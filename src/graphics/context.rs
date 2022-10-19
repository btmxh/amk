use std::{cell::Cell, collections::HashSet, sync::Arc};

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use vulkano::{
    command_buffer::{
        allocator::{
            CommandBufferAllocator, StandardCommandBufferAllocator,
            StandardCommandBufferBuilderAlloc,
        },
        AutoCommandBufferBuilder, CommandBufferLevel, CommandBufferUsage, RenderPassBeginInfo,
        SubpassContents,
    },
    device::{physical::PhysicalDevice, Device},
    device::{
        physical::PhysicalDeviceType, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
    },
    format::Format,
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCreateInfo,
        },
        Instance, InstanceCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        acquire_next_image, AcquireError, ColorSpace, Surface, Swapchain, SwapchainAbstract,
        SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
    },
    sync::{FlushError, GpuFuture, Sharing},
    VulkanLibrary,
};
use vulkano_win::{create_surface_from_handle, required_extensions};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct SendSyncWindowHandle {
    pub window: RawWindowHandle,
    pub display: RawDisplayHandle,
}

unsafe impl Send for SendSyncWindowHandle {}
unsafe impl Sync for SendSyncWindowHandle {}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum QueueKind {
    Graphics,
    Present,
}

unsafe impl HasRawWindowHandle for SendSyncWindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window
    }
}

unsafe impl HasRawDisplayHandle for SendSyncWindowHandle {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display
    }
}

pub struct RenderContext {
    pub lib: Arc<VulkanLibrary>,
    pub instance: Arc<Instance>,
    pub debug_messenger: Option<DebugUtilsMessenger>,
    pub surface: Arc<Surface<SendSyncWindowHandle>>,
    pub phys_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub present_queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain<SendSyncWindowHandle>>,
    pub images: Vec<Arc<SwapchainImage<SendSyncWindowHandle>>>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub main_command_buffer: StandardCommandBufferBuilderAlloc,
    pub render_extent: Cell<PhysicalSize<u32>>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub prev_frame_end: Option<Box<dyn GpuFuture>>,
}

unsafe impl Send for RenderContext {}

impl RenderContext {
    pub fn new(window: &Window) -> anyhow::Result<Self> {
        let lib = VulkanLibrary::new()?;
        let layers = ["VK_LAYER_KHRONOS_validation"];
        let debug = cfg!(debug_assertions) && lib.supported_extensions().ext_debug_utils;
        let instance = Instance::new(lib.clone(), {
            let mut info = InstanceCreateInfo::application_from_cargo_toml();
            let required_exts = required_extensions(&lib);
            info.enabled_extensions = required_exts;
            info.enabled_extensions.ext_debug_utils = debug;
            info.enumerate_portability = true;
            info.enabled_layers = lib
                .layer_properties()?
                .map(|layer| String::from(layer.name().trim_end_matches('\0')))
                .filter(|layer_name| layers.contains(&layer_name.as_str()))
                .collect::<Vec<_>>();
            info
        })?;
        let debug_messenger = if debug {
            unsafe {
                Some(DebugUtilsMessenger::new(
                    instance.clone(),
                    DebugUtilsMessengerCreateInfo {
                        message_severity: DebugUtilsMessageSeverity {
                            error: true,
                            warning: true,
                            information: true,
                            verbose: true,
                            ..Default::default()
                        },
                        message_type: DebugUtilsMessageType {
                            general: true,
                            validation: true,
                            performance: true,
                            ..Default::default()
                        },
                        ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                            if msg.severity.error {
                                log::error!(target: "vk", "{}", msg.description);
                            } else if msg.severity.warning {
                                log::warn!(target: "vk", "{}", msg.description);
                            } else if msg.severity.information {
                                log::info!(target: "vk", "{}", msg.description);
                            } else if msg.severity.verbose {
                                log::debug!(target: "vk", "{}", msg.description);
                            }
                        }))
                    },
                )?)
            }
        } else {
            None
        };
        let window_handle = SendSyncWindowHandle {
            window: window.raw_window_handle(),
            display: window.raw_display_handle(),
        };
        let surface = create_surface_from_handle(window_handle, instance.clone())?;

        let device_exts = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (phys_device, graphics_queue_index, present_queue_index) = instance
            .enumerate_physical_devices()?
            .filter(|pd| pd.supported_extensions().contains(&device_exts))
            .filter_map(|pd| {
                let graphics_queue_index = pd
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(_, queue)| queue.queue_flags.graphics);
                let present_queue_index =
                    pd.queue_family_properties()
                        .iter()
                        .enumerate()
                        .position(|(i, _)| {
                            pd.surface_support(i.try_into().unwrap(), &surface)
                                .unwrap_or(false)
                        });
                graphics_queue_index
                    .and_then(|graphics| present_queue_index.map(|present| (pd, graphics, present)))
            })
            .max_by_key(|(pd, ..)| {
                let mut score = 0;
                match pd.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => score += 4,
                    PhysicalDeviceType::IntegratedGpu => score += 3,
                    PhysicalDeviceType::VirtualGpu => score += 2,
                    PhysicalDeviceType::Cpu => score += 1,
                    _ => {}
                }
                score
            })
            .ok_or_else(|| anyhow::anyhow!("No suitable physical device found"))?;

        let unique_queue_families = vec![graphics_queue_index, present_queue_index]
            .drain(..)
            .collect::<HashSet<_>>();
        let (device, queues) = Device::new(
            phys_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: unique_queue_families
                    .iter()
                    .map(|index| QueueCreateInfo {
                        queue_family_index: (*index).try_into().unwrap(),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                enabled_extensions: device_exts,
                ..Default::default()
            },
        )?;

        let queues = queues.collect::<Vec<_>>();
        let graphics_queue = queues
            .iter()
            .find(|q| graphics_queue_index == q.queue_family_index().try_into().unwrap())
            .unwrap()
            .clone();
        let present_queue = queues
            .iter()
            .find(|q| present_queue_index == q.queue_family_index().try_into().unwrap())
            .unwrap()
            .clone();

        let (swapchain, images) = {
            let surf_caps = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())?;
            let image_count = (surf_caps.min_image_count + 1).clamp(
                surf_caps.min_image_count,
                surf_caps.max_image_count.unwrap_or(u32::MAX),
            );
            let (format, color_space) = *phys_device
                .surface_formats(&surface, Default::default())?
                .iter()
                .max_by_key(|(fmt, cs)| {
                    let mut score = 0;
                    if *fmt == Format::R8G8B8A8_UNORM || *fmt == Format::B8G8R8A8_UNORM {
                        score += 1;
                    }
                    if *cs == ColorSpace::SrgbNonLinear {
                        score += 1;
                    }
                    score
                })
                .unwrap();
            let extent = window.inner_size().into();
            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    image_color_space: color_space,
                    image_format: Some(format),
                    image_extent: extent,
                    image_sharing: match unique_queue_families.len() {
                        1 => Sharing::Exclusive,
                        _ => Sharing::Concurrent(
                            unique_queue_families
                                .iter()
                                .map(|i| u32::try_from(*i).unwrap())
                                .collect(),
                        ),
                    },
                    image_usage: ImageUsage {
                        color_attachment: true,
                        ..Default::default()
                    },
                    min_image_count: image_count,
                    ..Default::default()
                },
            )?
        };

        let command_buffer_allocator =
            Arc::new(StandardCommandBufferAllocator::new(device.clone()));
        let main_command_buffer = command_buffer_allocator
            .allocate(
                graphics_queue_index.try_into().unwrap(),
                CommandBufferLevel::Primary,
                1,
            )?
            .next()
            .unwrap();

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )?;

        let framebuffers = Self::create_framebuffers(&images, &render_pass)?;

        let prev_frame_end = Some(vulkano::sync::now(device.clone()).boxed());

        Ok(Self {
            lib,
            instance,
            debug_messenger,
            surface,
            phys_device,
            device,
            graphics_queue,
            present_queue,
            swapchain,
            images,
            command_buffer_allocator,
            main_command_buffer,
            render_extent: Cell::new(window.inner_size()),
            render_pass,
            framebuffers,
            prev_frame_end,
        })
    }

    fn create_framebuffers(
        images: &[Arc<SwapchainImage<SendSyncWindowHandle>>],
        render_pass: &Arc<RenderPass>,
    ) -> anyhow::Result<Vec<Arc<Framebuffer>>> {
        images
            .iter()
            .map(|img| ImageView::new_default(img.clone()).map_err(anyhow::Error::from))
            .map(|iv| {
                iv.and_then(|iv| {
                    Framebuffer::new(
                        render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![iv],
                            ..Default::default()
                        },
                    )
                    .map_err(anyhow::Error::from)
                })
            })
            .collect::<Result<Vec<Arc<Framebuffer>>, _>>()
            .map_err(anyhow::Error::from)
    }

    pub fn wait_for_done(&mut self) {
        self.prev_frame_end.as_mut().unwrap().cleanup_finished();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> anyhow::Result<bool> {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: size.into(),
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return Ok(false),
            Err(e) => Err(e)?,
        };

        self.swapchain = new_swapchain;
        self.framebuffers = Self::create_framebuffers(&new_images, &self.render_pass)?;
        Ok(true)
    }

    pub fn render(&mut self) -> anyhow::Result<bool> {
        let (image_idx, mut recreate_swapchain, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => return Ok(false),
                Err(e) => Err(e)?,
            };
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.as_ref(),
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.2, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_idx as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )?
            .end_render_pass()?;
        let command_buffer = builder.build()?;
        let future = self
            .prev_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.present_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_idx),
            )
            .then_signal_fence_and_flush();
        match future {
            Ok(f) => self.prev_frame_end = Some(f.boxed()),
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                self.prev_frame_end = Some(vulkano::sync::now(self.device.clone()).boxed())
            }
            Err(e) => {
                self.prev_frame_end = Some(vulkano::sync::now(self.device.clone()).boxed());
                Err(e)?
            }
        }
        Ok(recreate_swapchain)
    }
}
