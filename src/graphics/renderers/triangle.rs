use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    impl_vertex,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState, vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline,
    },
    render_pass::{RenderPass, Subpass},
};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
        #version 450
        layout (location = 0) in vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
        #version 450

        layout (location = 0) out vec4 color;

        void main() {
            color = vec4(1.0);
        }
        "
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub(crate) struct Vertex {
    position: [f32; 2],
}

impl_vertex!(Vertex, position);

pub struct TriangleRenderer {
    pub(crate) pipeline: Arc<GraphicsPipeline>,
    pub(crate) vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

impl TriangleRenderer {
    pub fn new(device: Arc<Device>, render_pass: Arc<RenderPass>) -> anyhow::Result<Self> {
        Ok(Self {
            pipeline: GraphicsPipeline::start()
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
                .input_assembly_state(InputAssemblyState::new())
                .vertex_shader(vs::load(device.clone())?.entry_point("main").unwrap(), ())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .fragment_shader(fs::load(device.clone())?.entry_point("main").unwrap(), ())
                .build(device.clone())?,
            vertex_buffer: CpuAccessibleBuffer::from_iter(
                device,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                [
                    Vertex {
                        position: [-0.5, -0.25],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                    },
                    Vertex {
                        position: [0.25, -0.1],
                    },
                ],
            )?,
        })
    }
}
