use std::sync::mpsc::Sender;

use winit::{
    event::{Event, WindowEvent},
    window::WindowId,
};

use crate::exec::msg::ELRLMsg;

pub(crate) struct ResizeWindowScene;
impl ResizeWindowScene {
    pub fn handle_event(e: &Event<()>, wid: WindowId, sender: &Sender<ELRLMsg>) -> bool {
        match e {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
            } if *window_id == wid => {
                sender.send(ELRLMsg::Resize(*size)).unwrap();
                true
            }
            _ => false,
        }
    }
}
