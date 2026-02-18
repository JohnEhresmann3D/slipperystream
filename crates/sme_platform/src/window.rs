//! Platform window creation.
//!
//! Thin wrapper around winit that applies `PlatformConfig` defaults. The window
//! is created lazily when the event loop calls `ApplicationHandler::resumed`
//! and returned as `Arc<Window>` so both the GPU surface and egui can share it.

use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub struct PlatformConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            title: "Saturday Morning Engine".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

pub fn create_window(event_loop: &ActiveEventLoop, config: &PlatformConfig) -> Arc<Window> {
    let attrs = WindowAttributes::default()
        .with_title(&config.title)
        .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height));

    let window = event_loop
        .create_window(attrs)
        .expect("Failed to create window");
    Arc::new(window)
}
