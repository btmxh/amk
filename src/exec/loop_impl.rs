use std::sync::{mpsc::Sender, Arc};

use winit::{event::Event, window::Window};

use crate::{scenes::root::RootScene, graphics::context::RenderContext};

use super::{loops::GameLoop, msg::ELGLMMsg};

pub struct UpdateLoop {
    pub root_scene: Arc<RootScene>,
}
impl GameLoop for UpdateLoop {}
pub struct RenderLoop {
    pub root_scene: Arc<RootScene>,
    pub render_ctx: RenderContext,
}
impl GameLoop for RenderLoop {}
pub struct AudioLoop {
    pub root_scene: Arc<RootScene>,
}
impl GameLoop for AudioLoop {}
pub struct EventLoop {
    pub window: Window,
    pub root_scene: Arc<RootScene>,
}
impl EventLoop {
    pub(crate) fn run(&self, event: Event<()>, el_glm_sender: &Sender<ELGLMMsg>) {
        let wid = self.window.id();
        self.root_scene.handle_event(event, wid, el_glm_sender);
    }
}
