use winit::dpi::PhysicalSize;

use super::{loops::{GameLoop, GameLoopKind}, mode::Mode};

pub(crate) enum ToRunnerMsg {
    RequestLoop(GameLoopKind),
    SendLoop(GameLoopKind, Box<dyn GameLoop>, f64),
    SetRelativeFrequency(GameLoopKind, f64),
    Stop,
}

pub(crate) enum FromRunnerMsg {
    SendLoop(Box<dyn GameLoop>),
}

pub enum ELGLMMsg {
    SetMode(Mode),
    Stop,
}

pub enum ELRLMsg {
    Resize(PhysicalSize<u32>)
}
