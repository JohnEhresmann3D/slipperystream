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

    // On the web, winit creates a detached <canvas>; it renders nothing and
    // receives no input until it is attached to the DOM. Append it to
    // #sme-container if the page provides one, else to <body>.
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let canvas = window.canvas().expect("winit window has no canvas");
        canvas
            .set_attribute("tabindex", "0")
            .expect("set canvas tabindex");
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no DOM document");
        let parent = document
            .get_element_by_id("sme-container")
            .or_else(|| document.body().map(|b| b.into()))
            .expect("no element to attach canvas to");
        parent
            .append_child(&canvas)
            .expect("failed to attach canvas");
        let _ = canvas.focus();

        // The size requested at window creation doesn't apply while the
        // canvas is detached from the DOM — it stays 1x1. Re-request it now
        // that the canvas is attached; this also fires the Resized event the
        // renderer uses to configure the surface.
        let _ = window.request_inner_size(winit::dpi::LogicalSize::new(
            config.width,
            config.height,
        ));
    }

    Arc::new(window)
}
