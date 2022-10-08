#![allow(dead_code)]
#![allow(irrefutable_let_patterns)]
#![allow(clippy::new_without_default)]

use std::sync::Arc;

use exec::loop_impl::RenderLoop;
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

    let render_loop = RenderLoop {
        root_scene: root_scene.clone(),
        render_ctx: RenderContext::new(&window)?,
    };
    let event_loop = EventLoop {
        window,
        root_scene: root_scene.clone(),
    };
    let update_loop = UpdateLoop {
        root_scene: root_scene.clone(),
    };
    let audio_loop = AudioLoop { root_scene };

    let manager = GameLoopManager::new(event_loop, update_loop, render_loop, audio_loop);
    manager.run(window_event_loop);
}
