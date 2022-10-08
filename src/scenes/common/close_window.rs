use std::sync::mpsc::Sender;

use winit::{
    event::{Event, WindowEvent},
    window::WindowId,
};

use crate::exec::msg::ELGLMMsg;

pub(crate) struct CloseWindowScene;
impl CloseWindowScene {
    pub fn handle_event(e: &Event<()>, wid: WindowId, sender: &Sender<ELGLMMsg>) -> bool {
        match e {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if *window_id == wid => {
                sender.send(ELGLMMsg::Stop).unwrap();
                true
            }
            _ => false,
        }
    }
}
