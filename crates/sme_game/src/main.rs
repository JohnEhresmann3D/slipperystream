use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use sme_core::input::{InputState, Key};
use sme_core::time::TimeState;
use sme_devtools::DebugOverlay;
use sme_platform::window::PlatformConfig;
use sme_render::{Camera2D, GpuContext, SpritePipeline, SpriteVertex, Texture};

const CAMERA_SPEED: f32 = 200.0;
const SPRITE_SIZE: f32 = 128.0;

struct EngineState {
    window: Arc<Window>,
    gpu: GpuContext,
    time: TimeState,
    input: InputState,
    camera: Camera2D,
    sprite_pipeline: SpritePipeline,
    debug_overlay: DebugOverlay,

    // GPU resources
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
}

impl EngineState {
    fn new(window: Arc<Window>) -> Self {
        let gpu = GpuContext::new(window.clone());

        let time = TimeState::new();
        let input = InputState::new();
        let camera = Camera2D::new(gpu.size.0, gpu.size.1);

        let sprite_pipeline = SpritePipeline::new(&gpu.device, gpu.surface_format);

        // Load test sprite texture
        let sprite_bytes = include_bytes!("../../../assets/textures/test_sprite.png");
        let texture = Texture::from_bytes(&gpu.device, &gpu.queue, sprite_bytes, "test_sprite");

        // Create a quad centered at origin
        let half = SPRITE_SIZE / 2.0;
        let vertices = [
            SpriteVertex {
                position: [-half, -half],
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [half, -half],
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [half, half],
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            SpriteVertex {
                position: [-half, half],
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        use wgpu::util::DeviceExt;
        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sprite Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let camera_uniform = camera.build_uniform();
        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group =
            sprite_pipeline.create_camera_bind_group(&gpu.device, &camera_buffer);
        let texture_bind_group = sprite_pipeline.create_texture_bind_group(&gpu.device, &texture);

        let debug_overlay = DebugOverlay::new(&gpu.device, gpu.surface_format, &window);

        Self {
            window,
            gpu,
            time,
            input,
            camera,
            sprite_pipeline,
            debug_overlay,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            texture_bind_group,
        }
    }
}

struct App {
    config: PlatformConfig,
    state: Option<EngineState>,
}

impl App {
    fn new() -> Self {
        Self {
            config: PlatformConfig::default(),
            state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = sme_platform::window::create_window(event_loop, &self.config);
        log::info!(
            "Window created: {}x{}",
            self.config.width,
            self.config.height
        );
        self.state = Some(EngineState::new(window));
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Let egui handle events first
        let egui_consumed = state
            .debug_overlay
            .handle_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting.");
                event_loop.exit();
            }

            WindowEvent::Resized(physical_size) => {
                let w = physical_size.width;
                let h = physical_size.height;
                if w > 0 && h > 0 {
                    state.gpu.resize(w, h);
                    state.camera.viewport = (w, h);
                    log::info!("Resized to {}x{}", w, h);
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

            WindowEvent::CursorMoved { position, .. } => {
                state.input.mouse_position = (position.x, position.y);
            }

            WindowEvent::RedrawRequested => {
                // Skip rendering when minimized
                if state.gpu.size.0 == 0 || state.gpu.size.1 == 0 {
                    return;
                }

                // --- Time ---
                state.time.begin_frame();
                while state.time.should_step() {
                    // Fixed update
                    if state.input.is_just_pressed(Key::Escape) {
                        event_loop.exit();
                        return;
                    }
                    if state.input.is_just_pressed(Key::F3) {
                        state.debug_overlay.toggle();
                    }

                    // Camera movement
                    let dt = state.time.fixed_dt as f32;
                    if state.input.is_held(Key::Left) || state.input.is_held(Key::A) {
                        state.camera.position.x -= CAMERA_SPEED * dt;
                    }
                    if state.input.is_held(Key::Right) || state.input.is_held(Key::D) {
                        state.camera.position.x += CAMERA_SPEED * dt;
                    }
                    if state.input.is_held(Key::Up) || state.input.is_held(Key::W) {
                        state.camera.position.y += CAMERA_SPEED * dt;
                    }
                    if state.input.is_held(Key::Down) || state.input.is_held(Key::S) {
                        state.camera.position.y -= CAMERA_SPEED * dt;
                    }
                }
                state.time.end_frame();

                // --- Update camera uniform ---
                let camera_uniform = state.camera.build_uniform();
                state.gpu.queue.write_buffer(
                    &state.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[camera_uniform]),
                );

                // --- Render ---
                let Some((output, view)) = state.gpu.begin_frame() else {
                    return;
                };

                // Prepare egui
                let (egui_primitives, egui_textures_delta) =
                    state.debug_overlay.prepare(&state.window, &state.time);

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [state.gpu.size.0, state.gpu.size.1],
                    pixels_per_point: state.window.scale_factor() as f32,
                };

                let mut encoder =
                    state
                        .gpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                // Sprite render pass (clear)
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Sprite Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.392,
                                    g: 0.584,
                                    b: 0.929,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });

                    render_pass.set_pipeline(&state.sprite_pipeline.render_pipeline);
                    render_pass.set_bind_group(0, &state.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, &state.texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..1);
                }

                // Egui overlay
                state.debug_overlay.upload(
                    &state.gpu.device,
                    &state.gpu.queue,
                    &mut encoder,
                    &egui_primitives,
                    &egui_textures_delta,
                    &screen_descriptor,
                );

                {
                    let mut egui_pass = encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("egui Render Pass"),
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

                    state
                        .debug_overlay
                        .paint(&mut egui_pass, &egui_primitives, &screen_descriptor);
                }

                state.debug_overlay.cleanup(&egui_textures_delta);

                state.gpu.queue.submit(std::iter::once(encoder.finish()));
                output.present();

                // --- End of frame ---
                state.input.end_frame();
            }

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
        KeyCode::F3 => Some(Key::F3),
        KeyCode::KeyW => Some(Key::W),
        KeyCode::KeyA => Some(Key::A),
        KeyCode::KeyS => Some(Key::S),
        KeyCode::KeyD => Some(Key::D),
        _ => None,
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Saturday Morning Engine starting...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
