//! Debug overlay rendered via egui on top of the game scene.
//!
//! Integration pattern: egui requires a three-phase render split because
//! `egui_wgpu::Renderer::render()` needs a `RenderPass<'static>`, while
//! `begin_render_pass` borrows the encoder. The phases are:
//!
//!   1. `prepare()` -- run egui UI logic, produce tessellated primitives
//!   2. `upload()`  -- upload textures and update GPU buffers (borrows encoder mutably)
//!   3. `paint()`   -- render into a new render pass with `forget_lifetime()`
//!   4. `cleanup()` -- free textures egui no longer references
//!
//! The overlay only runs UI logic when `visible` is true (toggled by F3),
//! but egui event handling is always active so the overlay can intercept
//! clicks when it is shown.

use sme_core::time::TimeState;
use winit::window::Window;

#[derive(Debug, Clone, Default)]
pub struct OverlayStats {
    pub draw_calls: u32,
    pub atlas_binds: u32,
    pub sprite_count: u32,
    /// Estimated GPU memory usage in megabytes
    pub memory_estimate_mb: f32,
    /// Current fidelity tier label (e.g. "Tier 0 (Mobile)")
    pub tier_label: String,
    /// Lua runtime status label (e.g. "Lua: loaded")
    pub lua_status_label: String,
    /// Whether simulation is paused
    pub paused: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OverlayActions {
    /// User clicked the tier cycle button
    pub cycle_tier: bool,
    /// User clicked the pause toggle
    pub toggle_pause: bool,
    /// User clicked the single-step button (advance one fixed step while paused)
    pub single_step: bool,
}

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
        stats: Option<OverlayStats>,
    ) -> (
        Vec<egui::ClippedPrimitive>,
        egui::TexturesDelta,
        OverlayActions,
    ) {
        let mut actions = OverlayActions::default();
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
                        if let Some(ref stats) = stats {
                            ui.separator();
                            ui.label(format!("Draw calls: {}", stats.draw_calls));
                            ui.label(format!("Atlas binds: {}", stats.atlas_binds));
                            ui.label(format!("Sprites: {}", stats.sprite_count));
                            ui.label(format!("Memory: {:.1} MB", stats.memory_estimate_mb));
                        }

                        // --- M5: Fidelity Tier ---
                        if let Some(ref stats) = stats {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(format!("Fidelity: {}", stats.tier_label));
                                if ui.button("Cycle").clicked() {
                                    actions.cycle_tier = true;
                                }
                            });

                            // --- M5: Lua Status ---
                            ui.label(&stats.lua_status_label);

                            // --- M5: Simulation Controls ---
                            ui.separator();
                            ui.horizontal(|ui| {
                                let pause_label = if stats.paused { "Resume" } else { "Pause" };
                                if ui.button(pause_label).clicked() {
                                    actions.toggle_pause = true;
                                }
                                if stats.paused && ui.button("Step").clicked() {
                                    actions.single_step = true;
                                }
                            });
                            if stats.paused {
                                ui.label("\u{23f8} PAUSED");
                            }
                        }
                    });
            }
        });

        self.egui_winit_state
            .handle_platform_output(window, full_output.platform_output);

        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        (primitives, full_output.textures_delta, actions)
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
