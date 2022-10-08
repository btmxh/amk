use std::sync::mpsc::Sender;

use winit::{event::Event, window::WindowId};

use crate::exec::msg::ELGLMMsg;

use super::common::close_window::CloseWindowScene;

pub struct RootScene;

impl RootScene {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_event(&self, e: Event<()>, wid: WindowId, sender: &Sender<ELGLMMsg>) {
        let _handle = CloseWindowScene::handle_event(&e, wid, sender);
    }
}
