use std::sync::mpsc::Sender;

use winit::{event::Event, window::WindowId};

use crate::exec::msg::{ELGLMMsg, ELRLMsg};

use super::common::{close_window::CloseWindowScene, resize_window::ResizeWindowScene};

pub struct RootScene;

impl RootScene {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_event(&self, e: Event<()>, wid: WindowId, elglm_sender: &Sender<ELGLMMsg>, elrl_sender: &Sender<ELRLMsg>) {
        let _handle = CloseWindowScene::handle_event(&e, wid, elglm_sender)
            || ResizeWindowScene::handle_event(&e, wid, elrl_sender);
    }
}
