#![allow(dead_code)]
#![allow(irrefutable_let_patterns)]
#![allow(clippy::new_without_default)]

use std::sync::Arc;

use exec::{loop_impl::RenderLoop, msg::{ELRLMsg, ELGLMMsg}};
use graphics::context::RenderContext;
use logging::init_log;
use scenes::root::RootScene;
use winit::{dpi::PhysicalSize, window::WindowBuilder};

use crate::exec::{
    loop_impl::{AudioLoop, EventLoop, UpdateLoop},
    manager::{GameLoopManager, WinitEventLoop},
};

pub mod exec;
pub mod graphics;
pub mod logging;
pub mod scenes;
pub mod utils;

fn main() -> anyhow::Result<()> {
    init_log()?;
    let window_event_loop = WinitEventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("hello")
        .build(&window_event_loop)?;

    let root_scene = Arc::new(RootScene::new());

    // ELRL communication channels
    let (elrl_sender, elrl_receiver) = std::sync::mpsc::channel::<ELRLMsg>();
    // EventLoop-GameLoopManager (ELGLM) communication channels
    let (elglm_sender, elglm_receiver) = std::sync::mpsc::channel::<ELGLMMsg>();
        
    let render_loop = RenderLoop {
        root_scene: root_scene.clone(),
        render_ctx: RenderContext::new(&window)?,
        new_size: None,
        elrl_receiver,
    };
    let event_loop = EventLoop {
        window,
        root_scene: root_scene.clone(),
        elrl_sender,
        elglm_sender,
    };
    let update_loop = UpdateLoop {
        root_scene: root_scene.clone(),
    };
    let audio_loop = AudioLoop { root_scene };

    let manager = GameLoopManager::new(event_loop, update_loop, render_loop, audio_loop);
    manager.run(window_event_loop, elglm_receiver);
}
