//! GRIM DELIVERY — Paperboy, but you're the Grim Reaper and the paper route
//! is your job description.
//!
//! 48-hour-jam-scope prototype built on the Saturday Morning Engine crates:
//! sme_platform (window), sme_core (fixed timestep + input), sme_render
//! (camera + sprite pipeline). All art is vertex-colored quads on a single
//! 1x1 white texture, so the whole scene is one draw call; the HUD is egui.
//!
//! Controls: A/D or arrows change lane · SPACE throws (left side) · ESC quits.

mod game;
mod hud;
mod level;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use game::{Game, GameInput, Quad};
use hud::GameHud;
use sme_core::input::{InputState, Key};
use sme_core::time::TimeState;
use sme_platform::window::PlatformConfig;
use sme_render::{Camera2D, GpuContext, SpritePipeline, SpriteVertex, Texture};

const CAMERA_ZOOM: f32 = 1.4;
/// Camera x bias: keeps the street center-right so the house row stays on screen.
const CAMERA_X: f32 = -60.0;
/// Suburban lawn green — the cheerful backdrop to a morbid job.
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.28,
    g: 0.52,
    b: 0.22,
    a: 1.0,
};

struct GameApp {
    window: Arc<Window>,
    gpu: GpuContext,
    time: TimeState,
    input: InputState,
    camera: Camera2D,
    sprite_pipeline: SpritePipeline,
    hud: GameHud,
    game: Game,

    white_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    index_count: u32,
    quad_scratch: Vec<Quad>,
}

impl GameApp {
    /// GPU init is async because the browser forbids blocking on wasm32;
    /// native callers wrap this in `pollster::block_on`.
    async fn new(window: Arc<Window>) -> Self {
        let gpu = GpuContext::new_async(window.clone()).await;
        let sprite_pipeline = SpritePipeline::new(&gpu.device, gpu.surface_format);
        let hud = GameHud::new(&gpu.device, gpu.surface_format, &window);

        let white = Texture::from_rgba8(
            &gpu.device,
            &gpu.queue,
            &[255, 255, 255, 255],
            1,
            1,
            "white",
        );
        let white_bind_group = sprite_pipeline.create_texture_bind_group(&gpu.device, &white);

        let mut camera = Camera2D::new(gpu.size.0, gpu.size.1);
        camera.zoom = CAMERA_ZOOM;
        camera.position.x = CAMERA_X;

        let camera_buffer = {
            use wgpu::util::DeviceExt;
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[camera.build_uniform()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
        };
        let camera_bind_group =
            sprite_pipeline.create_camera_bind_group(&gpu.device, &camera_buffer);
        let vertex_buffer = create_vertex_buffer(&gpu.device, 4);
        let index_buffer = create_index_buffer(&gpu.device, 6);

        Self {
            window,
            gpu,
            time: TimeState::new(),
            input: InputState::new(),
            camera,
            sprite_pipeline,
            hud,
            game: Game::new(),
            white_bind_group,
            camera_buffer,
            camera_bind_group,
            vertex_buffer,
            index_buffer,
            vertex_capacity: 4,
            index_capacity: 6,
            index_count: 0,
            quad_scratch: Vec::with_capacity(512),
        }
    }

    fn build_game_input(&self) -> GameInput {
        GameInput {
            lane_left_pressed: self.input.is_just_pressed(Key::Left)
                || self.input.is_just_pressed(Key::A),
            lane_right_pressed: self.input.is_just_pressed(Key::Right)
                || self.input.is_just_pressed(Key::D),
            throw_pressed: self.input.is_just_pressed(Key::Space),
            advance_pressed: self.input.is_just_pressed(Key::Space),
            restart_pressed: self.input.is_just_pressed(Key::R),
        }
    }

    fn rebuild_mesh(&mut self) {
        self.quad_scratch.clear();
        let mut quads = std::mem::take(&mut self.quad_scratch);
        self.game.build_quads(&mut quads);

        let mut vertices: Vec<SpriteVertex> = Vec::with_capacity(quads.len() * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(quads.len() * 6);
        for quad in &quads {
            let half_w = quad.w * 0.5;
            let half_h = quad.h * 0.5;
            let base = vertices.len() as u32;
            let corners = [
                [quad.x - half_w, quad.y - half_h],
                [quad.x + half_w, quad.y - half_h],
                [quad.x + half_w, quad.y + half_h],
                [quad.x - half_w, quad.y + half_h],
            ];
            let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            for (pos, uv) in corners.iter().zip(uvs.iter()) {
                vertices.push(SpriteVertex {
                    position: *pos,
                    tex_coords: *uv,
                    color: quad.color,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        self.quad_scratch = quads;

        if vertices.len() > self.vertex_capacity {
            self.vertex_capacity = vertices.len().next_power_of_two();
            self.vertex_buffer = create_vertex_buffer(&self.gpu.device, self.vertex_capacity);
        }
        if indices.len() > self.index_capacity {
            self.index_capacity = indices.len().next_power_of_two();
            self.index_buffer = create_index_buffer(&self.gpu.device, self.index_capacity);
        }
        self.index_count = indices.len() as u32;
        if !vertices.is_empty() {
            self.gpu
                .queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            self.gpu
                .queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.size.0 == 0 || self.gpu.size.1 == 0 {
            return;
        }

        self.time.begin_frame();
        while self.time.should_step() {
            if self.input.is_just_pressed(Key::Escape) {
                event_loop.exit();
                return;
            }
            let game_input = self.build_game_input();
            self.game
                .update_fixed(self.time.fixed_dt as f32, game_input);
        }
        self.time.end_frame();

        self.camera.position.x = CAMERA_X;
        self.camera.position.y = self.game.camera_y();
        self.rebuild_mesh();

        self.gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.build_uniform()]),
        );

        let Some((output, view)) = self.gpu.begin_frame() else {
            return;
        };

        let (egui_primitives, egui_textures_delta) =
            self.hud
                .prepare(&self.window, &self.game, self.time.smoothed_fps as f32);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.gpu.size.0, self.gpu.size.1],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Grim Delivery Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            render_pass.set_pipeline(&self.sprite_pipeline.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.white_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            if self.index_count > 0 {
                render_pass.draw_indexed(0..self.index_count, 0, 0..1);
            }
        }

        self.hud.upload(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &egui_primitives,
            &egui_textures_delta,
            &screen_descriptor,
        );

        {
            let mut egui_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("HUD Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                })
                .forget_lifetime();
            self.hud
                .paint(&mut egui_pass, &egui_primitives, &screen_descriptor);
        }

        self.hud.cleanup(&egui_textures_delta);
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // One-time diagnostic: confirms a frame actually reached present()
        // and with what geometry — the difference between "not rendering"
        // and "rendering somewhere we can't see".
        if self.time.frame_count <= 1 {
            log::info!(
                "first frame presented: surface {}x{}, {} indices, format {:?}",
                self.gpu.size.0,
                self.gpu.size.1,
                self.index_count,
                self.gpu.surface_format
            );
            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowExtWebSys;
                if let Some(canvas) = self.window.canvas() {
                    log::info!(
                        "canvas: attribute {}x{}, client {}x{}, connected: {}",
                        canvas.width(),
                        canvas.height(),
                        canvas.client_width(),
                        canvas.client_height(),
                        canvas.is_connected()
                    );
                }
            }
        }

        // Clear edge-triggered input only after a fixed step consumed it,
        // mirroring the engine's input-loss guard.
        if self.time.steps_this_frame > 0 {
            self.input.end_frame();
        }
    }
}

struct App {
    config: PlatformConfig,
    state: Option<GameApp>,
    /// Route for the wasm path: GPU init completes in a spawned future and
    /// delivers the finished `GameApp` back to the loop as a user event.
    /// Unused on native, where init happens synchronously in `resumed`.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    proxy: EventLoopProxy<GameApp>,
    /// Guards against `resumed` firing more than once while async init is
    /// still in flight on wasm.
    init_started: bool,
}

impl ApplicationHandler<GameApp> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() || self.init_started {
            return;
        }
        self.init_started = true;
        let window = sme_platform::window::create_window(event_loop, &self.config);

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(pollster::block_on(GameApp::new(window)));
        }
        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let app = GameApp::new(window).await;
                let _ = proxy.send_event(app);
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, app: GameApp) {
        app.window.request_redraw();
        self.state = Some(app);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let egui_consumed = state.hud.handle_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    state.gpu.resize(size.width, size.height);
                    state.camera.viewport = (size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } if !egui_consumed => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if let Some(engine_key) = map_key(key_code) {
                        match event.state {
                            ElementState::Pressed => state.input.key_down(engine_key),
                            ElementState::Released => state.input.key_up(engine_key),
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => state.redraw(event_loop),
            _ => {}
        }
    }
}

fn map_key(key_code: KeyCode) -> Option<Key> {
    match key_code {
        KeyCode::ArrowLeft => Some(Key::Left),
        KeyCode::ArrowRight => Some(Key::Right),
        KeyCode::ArrowUp => Some(Key::Up),
        KeyCode::ArrowDown => Some(Key::Down),
        KeyCode::Escape => Some(Key::Escape),
        KeyCode::Space => Some(Key::Space),
        KeyCode::KeyW => Some(Key::W),
        KeyCode::KeyA => Some(Key::A),
        KeyCode::KeyS => Some(Key::S),
        KeyCode::KeyD => Some(Key::D),
        KeyCode::KeyR => Some(Key::R),
        _ => None,
    }
}

fn create_vertex_buffer(device: &wgpu::Device, vertex_capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Grim Vertex Buffer"),
        size: (vertex_capacity * std::mem::size_of::<SpriteVertex>()).max(1) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_index_buffer(device: &wgpu::Device, index_capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Grim Index Buffer"),
        size: (index_capacity * std::mem::size_of::<u32>()).max(1) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("failed to init console logger");
    }
    log::info!("GRIM DELIVERY — clocking in.");

    let event_loop = EventLoop::<GameApp>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let app = App {
        config: PlatformConfig {
            title: "GRIM DELIVERY — Grim Delivery Co. Route Client v0.1".to_string(),
            width: 1280,
            height: 720,
        },
        state: None,
        proxy: event_loop.create_proxy(),
        init_started: false,
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop.run_app(&mut app).expect("Event loop error");
    }
    // On the web the loop must not block; spawn_app hands control to the
    // browser and drives frames via requestAnimationFrame.
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }
}
