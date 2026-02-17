use sme_core::time::TimeState;
use winit::window::Window;

pub struct DebugOverlay {
    pub egui_ctx: egui::Context,
    pub egui_winit_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
    pub visible: bool,
}

impl DebugOverlay {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let egui_ctx = egui::Context::default();
        let egui_winit_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui_ctx.viewport_id(),
            window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1, false);

        Self {
            egui_ctx,
            egui_winit_state,
            egui_renderer,
            visible: false,
        }
    }

    pub fn handle_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let response = self.egui_winit_state.on_window_event(window, event);
        response.consumed
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        log::info!("Debug overlay: {}", if self.visible { "ON" } else { "OFF" });
    }

    pub fn prepare(
        &mut self,
        window: &Window,
        time: &TimeState,
    ) -> (Vec<egui::ClippedPrimitive>, egui::TexturesDelta) {
        let raw_input = self.egui_winit_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            if self.visible {
                egui::Window::new("Debug")
                    .default_pos([10.0, 10.0])
                    .show(ctx, |ui| {
                        ui.label(format!("FPS: {:.1}", time.smoothed_fps));
                        ui.label(format!("Frame time: {:.2} ms", time.smoothed_frame_time_ms));
                        ui.label(format!("Steps this frame: {}", time.steps_this_frame));
                        ui.label(format!("Total steps: {}", time.fixed_step_count));
                        ui.label(format!("Frame: {}", time.frame_count));
                    });
            }
        });

        self.egui_winit_state
            .handle_platform_output(window, full_output.platform_output);

        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        (primitives, full_output.textures_delta)
    }

    /// Upload textures and update buffers. Call before creating the egui render pass.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.egui_renderer
            .update_buffers(device, queue, encoder, primitives, screen_descriptor);
    }

    /// Render into an existing render pass. Call after `upload()`.
    pub fn paint(
        &self,
        render_pass: &mut wgpu::RenderPass<'static>,
        primitives: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        self.egui_renderer
            .render(render_pass, primitives, screen_descriptor);
    }

    /// Free textures that egui no longer needs. Call after rendering.
    pub fn cleanup(&mut self, textures_delta: &egui::TexturesDelta) {
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }
}
