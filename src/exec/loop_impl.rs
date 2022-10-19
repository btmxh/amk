use std::sync::{
    mpsc::{Receiver, Sender, TryRecvError},
    Arc,
};

use winit::{dpi::PhysicalSize, event::Event, window::Window};

use crate::{graphics::context::RenderContext, scenes::root::RootScene};

use super::{
    loops::GameLoop,
    msg::{ELGLMMsg, ELRLMsg},
};

pub struct UpdateLoop {
    pub root_scene: Arc<RootScene>,
}
impl GameLoop for UpdateLoop {}
pub struct RenderLoop {
    pub root_scene: Arc<RootScene>,
    pub render_ctx: RenderContext,
    pub new_size: Option<PhysicalSize<u32>>,
    pub elrl_receiver: Receiver<ELRLMsg>,
}
impl GameLoop for RenderLoop {
    fn run(&mut self) -> anyhow::Result<()> {
        match self.elrl_receiver.try_recv() {
            Ok(msg) => match msg {
                ELRLMsg::Resize(size) => self.new_size = Some(size),
            },
            Err(TryRecvError::Empty) => {}
            e => {
                e.unwrap();
            }
        }
        self.render_ctx.wait_for_done();
        let done_resizing = if let Some(size) = &self.new_size {
            self.render_ctx.resize(*size)?
        } else {
            false
        };
        if done_resizing {
            self.new_size = None
        }
        self.render_ctx.render()?;
        Ok(())
    }
}
pub struct AudioLoop {
    pub root_scene: Arc<RootScene>,
}
impl GameLoop for AudioLoop {}
pub struct EventLoop {
    pub window: Window,
    pub root_scene: Arc<RootScene>,
    pub elrl_sender: Sender<ELRLMsg>,
    pub elglm_sender: Sender<ELGLMMsg>,
}
impl EventLoop {
    pub(crate) fn run(&self, event: Event<()>) {
        let wid = self.window.id();
        self.root_scene
            .handle_event(event, wid, &self.elglm_sender, &self.elrl_sender);
    }
}
