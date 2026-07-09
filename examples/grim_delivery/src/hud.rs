//! Game HUD rendered with egui, following the same three-phase
//! prepare/upload/paint pattern as the engine's DebugOverlay.
//!
//! UI voice per the GDD: everything reads like corporate delivery-app copy.

use winit::window::Window;

use crate::game::{BannerKind, Game, Phase};

pub struct GameHud {
    pub egui_ctx: egui::Context,
    pub egui_winit_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
}

const INK: egui::Color32 = egui::Color32::from_rgb(30, 28, 34);
const PAPER: egui::Color32 = egui::Color32::from_rgb(246, 242, 226);
const GOOD: egui::Color32 = egui::Color32::from_rgb(70, 160, 90);
const BAD: egui::Color32 = egui::Color32::from_rgb(190, 60, 50);

impl GameHud {
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
        }
    }

    pub fn handle_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        self.egui_winit_state
            .on_window_event(window, event)
            .consumed
    }

    pub fn prepare(
        &mut self,
        window: &Window,
        game: &Game,
        fps: f32,
    ) -> (Vec<egui::ClippedPrimitive>, egui::TexturesDelta) {
        let raw_input = self.egui_winit_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| match game.phase {
            Phase::Intro => draw_intro(ctx, game),
            Phase::Riding => {
                draw_clipboard(ctx, game, fps);
                draw_banners(ctx, game);
            }
            Phase::Summary => draw_summary(ctx, game),
            Phase::Final => draw_final(ctx, game),
        });

        self.egui_winit_state
            .handle_platform_output(window, full_output.platform_output);
        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        (primitives, full_output.textures_delta)
    }

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

    pub fn paint(
        &self,
        render_pass: &mut wgpu::RenderPass<'static>,
        primitives: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        self.egui_renderer
            .render(render_pass, primitives, screen_descriptor);
    }

    pub fn cleanup(&mut self, textures_delta: &egui::TexturesDelta) {
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }
}

fn paper_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PAPER)
        .stroke(egui::Stroke::new(2.0, INK))
        .inner_margin(egui::Margin::same(14))
        .corner_radius(egui::CornerRadius::same(4))
}

fn ink_label(ui: &mut egui::Ui, text: impl Into<String>, size: f32, strong: bool) {
    let mut rich = egui::RichText::new(text.into()).size(size).color(INK);
    if strong {
        rich = rich.strong();
    }
    ui.label(rich);
}

fn draw_clipboard(ctx: &egui::Context, game: &Game, fps: f32) {
    egui::Window::new("clipboard")
        .title_bar(false)
        .resizable(false)
        .frame(paper_frame())
        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
        .show(ctx, |ui| {
            ink_label(ui, "☠ GRIM DELIVERY CO.", 16.0, true);
            ink_label(ui, game.level().route_label, 13.0, false);
            ui.separator();
            ink_label(
                ui,
                format!(
                    "SOULS COLLECTED:  {} / {}",
                    game.stats.souls, game.stats.quota
                ),
                15.0,
                true,
            );
            ink_label(
                ui,
                format!("SCORE: {}", game.total_score + game.stats.score),
                14.0,
                false,
            );
            if game.combo > 1 {
                ui.label(
                    egui::RichText::new(format!("STREAK x{}", game.combo))
                        .size(13.0)
                        .color(GOOD)
                        .strong(),
                );
            }
            if game.stats.wrong > 0 {
                ui.label(
                    egui::RichText::new(format!("MISFILINGS: {}", game.stats.wrong))
                        .size(12.0)
                        .color(BAD),
                );
            }
            ui.separator();
            ink_label(ui, "Porch light OFF = scheduled stop.", 11.0, false);
            ink_label(ui, "Porch light ON  = DO NOT SERVICE.", 11.0, false);
            ui.label(
                egui::RichText::new(format!("{fps:.0} fps"))
                    .size(9.0)
                    .color(egui::Color32::from_rgb(120, 116, 108)),
            );
        });
}

fn draw_banners(ctx: &egui::Context, game: &Game) {
    if game.banners.is_empty() {
        return;
    }
    egui::Window::new("banners")
        .title_bar(false)
        .resizable(false)
        .frame(egui::Frame::new())
        .anchor(egui::Align2::CENTER_TOP, [0.0, 16.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                for banner in &game.banners {
                    let color = match banner.kind {
                        BannerKind::Good => GOOD,
                        BannerKind::Bad => BAD,
                        BannerKind::Neutral => INK,
                    };
                    let alpha = (banner.ttl / 0.5).clamp(0.0, 1.0);
                    let bg = PAPER.gamma_multiply(alpha);
                    egui::Frame::new()
                        .fill(bg)
                        .stroke(egui::Stroke::new(1.5, color.gamma_multiply(alpha)))
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .corner_radius(egui::CornerRadius::same(3))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&banner.text)
                                    .size(14.0)
                                    .strong()
                                    .color(color.gamma_multiply(alpha)),
                            );
                        });
                    ui.add_space(4.0);
                }
            });
        });
}

fn draw_intro(ctx: &egui::Context, game: &Game) {
    egui::Window::new("intro")
        .title_bar(false)
        .resizable(false)
        .frame(paper_frame())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ink_label(ui, "☠ GRIM DELIVERY CO. ☠", 24.0, true);
                ink_label(ui, "\"Every notice finds its home. Eventually.\"", 12.0, false);
                ui.add_space(10.0);
                ink_label(
                    ui,
                    format!("SHIFT {} of 3", game.level_index + 1),
                    14.0,
                    false,
                );
                ink_label(ui, game.level().route_label, 18.0, true);
                ui.add_space(6.0);
                ink_label(
                    ui,
                    format!(
                        "{} stops scheduled. Porch light OFF marks the address.",
                        game.stats.quota
                    ),
                    13.0,
                    false,
                );
                ink_label(
                    ui,
                    "Deliver ONLY to scheduled addresses. Misfilings are\ncareer-limiting and mildly fatal to bystanders.",
                    12.0,
                    false,
                );
                ui.add_space(12.0);
                ink_label(ui, "◄ ► or A/D — change lane", 13.0, false);
                ink_label(ui, "SPACE — throw death notice (left side)", 13.0, false);
                ui.add_space(12.0);
                ink_label(ui, "PRESS SPACE TO CLOCK IN", 16.0, true);
            });
        });
}

fn draw_summary(ctx: &egui::Context, game: &Game) {
    let stats = &game.stats;
    let quota_met = stats.souls >= stats.quota;
    egui::Window::new("summary")
        .title_bar(false)
        .resizable(false)
        .frame(paper_frame())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ink_label(ui, "END-OF-ROUTE REPORT", 20.0, true);
                ink_label(ui, game.level().route_label, 13.0, false);
                ui.add_space(8.0);
                ink_label(
                    ui,
                    format!("Souls collected: {} / {}", stats.souls, stats.quota),
                    15.0,
                    true,
                );
                ink_label(
                    ui,
                    format!("Wrong deliveries: {}", stats.wrong),
                    13.0,
                    false,
                );
                ink_label(ui, format!("Stops missed: {}", stats.missed), 13.0, false);
                ink_label(
                    ui,
                    format!("Best streak: x{}", stats.best_combo.max(1)),
                    13.0,
                    false,
                );
                ink_label(ui, format!("Route score: {}", stats.score), 15.0, true);
                ui.add_space(8.0);
                let line = if quota_met {
                    game.level().manager_line_good
                } else {
                    game.level().manager_line_bad
                };
                ui.label(egui::RichText::new(line).size(12.0).italics().color(INK));
                ui.add_space(12.0);
                let next = if game.level_index + 1 < 3 {
                    "PRESS SPACE FOR NEXT ROUTE"
                } else {
                    "PRESS SPACE TO FILE FINAL PAPERWORK"
                };
                ink_label(ui, next, 15.0, true);
            });
        });
}

fn draw_final(ctx: &egui::Context, game: &Game) {
    let total_souls: u32 = game.completed_levels.iter().map(|s| s.souls).sum();
    let total_quota: u32 = game.completed_levels.iter().map(|s| s.quota).sum();
    let total_wrong: u32 = game.completed_levels.iter().map(|s| s.wrong).sum();
    egui::Window::new("final")
        .title_bar(false)
        .resizable(false)
        .frame(paper_frame())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ink_label(ui, "QUARTERLY MORTALITY REVIEW", 20.0, true);
                ui.add_space(8.0);
                ink_label(
                    ui,
                    format!("Souls collected: {total_souls} / {total_quota}"),
                    15.0,
                    true,
                );
                ink_label(ui, format!("Total misfilings: {total_wrong}"), 13.0, false);
                ink_label(ui, format!("FINAL SCORE: {}", game.total_score), 18.0, true);
                ui.add_space(8.0);
                ink_label(ui, game.final_rank(), 14.0, true);
                ui.add_space(4.0);
                ink_label(
                    ui,
                    "Thank you for delivering with Grim Delivery Co.\nYour performance has been noted in your permanent record.\nAll records are permanent.",
                    11.0,
                    false,
                );
                ui.add_space(12.0);
                ink_label(ui, "PRESS R FOR ANOTHER SHIFT  ·  ESC TO RESIGN", 14.0, true);
            });
        });
}
