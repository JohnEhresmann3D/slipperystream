mod scene;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use scene::{load_scene_from_path, SceneFile, SceneWatcher, SortMode};
use sme_core::input::{InputState, Key};
use sme_core::time::TimeState;
use sme_devtools::DebugOverlay;
use sme_platform::window::PlatformConfig;
use sme_render::{Camera2D, GpuContext, SpritePipeline, SpriteVertex, Texture};

const CAMERA_SPEED: f32 = 200.0;
const SCENE_PATH: &str = "assets/scenes/m2_scene.json";
const FALLBACK_TEXTURE_BYTES: &[u8] = include_bytes!("../../../assets/textures/test_sprite.png");

#[derive(Debug, Clone)]
struct DrawCall {
    asset: String,
    index_start: u32,
    index_count: u32,
}

struct GpuSpriteTexture {
    texture: Texture,
    bind_group: wgpu::BindGroup,
}

struct EngineState {
    window: Arc<Window>,
    gpu: GpuContext,
    time: TimeState,
    input: InputState,
    camera: Camera2D,
    sprite_pipeline: SpritePipeline,
    debug_overlay: DebugOverlay,

    scene_path: std::path::PathBuf,
    scene_watcher: SceneWatcher,
    scene: SceneFile,
    textures: HashMap<String, GpuSpriteTexture>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    mesh_vertex_capacity: usize,
    mesh_index_capacity: usize,
    draw_calls: Vec<DrawCall>,
}

impl EngineState {
    fn new(window: Arc<Window>) -> Self {
        let gpu = GpuContext::new(window.clone());
        let time = TimeState::new();
        let input = InputState::new();
        let sprite_pipeline = SpritePipeline::new(&gpu.device, gpu.surface_format);
        let debug_overlay = DebugOverlay::new(&gpu.device, gpu.surface_format, &window);

        let scene_path = std::path::PathBuf::from(SCENE_PATH);
        let scene_watcher = SceneWatcher::new(scene_path.clone());
        let scene = load_scene_from_path(&scene_path).unwrap_or_else(|err| {
            panic!(
                "Failed to load initial scene '{}': {}",
                scene_path.display(),
                err
            );
        });

        let mut camera = Camera2D::new(gpu.size.0, gpu.size.1);
        if let Some(scene_camera) = &scene.camera {
            camera.position.x = scene_camera.start_x;
            camera.position.y = scene_camera.start_y;
            camera.zoom = scene_camera.zoom;
        }

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
        let vertex_buffer = create_vertex_buffer(&gpu.device, 1);
        let index_buffer = create_index_buffer(&gpu.device, 1);

        let mut state = Self {
            window,
            gpu,
            time,
            input,
            camera,
            sprite_pipeline,
            debug_overlay,
            scene_path,
            scene_watcher,
            scene,
            textures: HashMap::new(),
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            mesh_vertex_capacity: 0,
            mesh_index_capacity: 0,
            draw_calls: Vec::new(),
        };

        state.ensure_textures_for_scene();
        state.ensure_mesh_capacity(4, 6);
        state.rebuild_scene_mesh();
        state
    }

    fn reload_scene(&mut self, reason: &str) {
        match load_scene_from_path(&self.scene_path) {
            Ok(scene) => {
                self.scene = scene;
                if let Some(scene_camera) = &self.scene.camera {
                    self.camera.position.x = scene_camera.start_x;
                    self.camera.position.y = scene_camera.start_y;
                    self.camera.zoom = scene_camera.zoom;
                }
                self.ensure_textures_for_scene();
                self.rebuild_scene_mesh();
                log::info!(
                    "Scene reloaded ({reason}): {} ({})",
                    self.scene.scene_id,
                    self.scene.version
                );
            }
            Err(err) => {
                log::error!("Scene reload failed ({reason}): {err}");
            }
        }
    }

    fn ensure_textures_for_scene(&mut self) {
        let mut required_assets = HashSet::new();
        for layer in &self.scene.layers {
            for sprite in &layer.sprites {
                required_assets.insert(sprite.asset.clone());
            }
        }

        for asset_path in required_assets {
            if self.textures.contains_key(&asset_path) {
                continue;
            }
            let texture = load_texture_asset(
                &self.gpu.device,
                &self.gpu.queue,
                &self.sprite_pipeline,
                &asset_path,
            );
            self.textures.insert(asset_path, texture);
        }
    }

    fn rebuild_scene_mesh(&mut self) {
        let (vertices, indices, draw_calls) = self.build_mesh();
        self.ensure_mesh_capacity(vertices.len(), indices.len());
        self.draw_calls = draw_calls;

        if !vertices.is_empty() {
            self.gpu
                .queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }
        if !indices.is_empty() {
            self.gpu
                .queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
        }
    }

    fn build_mesh(&self) -> (Vec<SpriteVertex>, Vec<u32>, Vec<DrawCall>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut draw_calls = Vec::new();

        for layer in &self.scene.layers {
            if !layer.visible {
                continue;
            }

            let mut sprites = layer.sprites.clone();
            if matches!(layer.sort_mode, SortMode::Y) {
                sprites.sort_by(|a, b| {
                    a.y.partial_cmp(&b.y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
                });
            }

            if layer.occlusion {
                log::trace!("Rendering occlusion layer '{}'", layer.id);
            }

            let parallax_offset = self.camera.position * (1.0 - layer.parallax);
            for sprite in &sprites {
                let Some(texture) = self.textures.get(&sprite.asset) else {
                    log::warn!(
                        "Skipping sprite '{}' due to missing texture '{}'",
                        sprite.id,
                        sprite.asset
                    );
                    continue;
                };

                let center_x = sprite.x + parallax_offset.x;
                let center_y = sprite.y + parallax_offset.y;
                let half_w = (texture.texture.size.0 as f32) * 0.5 * sprite.scale_x;
                let half_h = (texture.texture.size.1 as f32) * 0.5 * sprite.scale_y;
                let base_index = vertices.len() as u32;

                let mut corners = [
                    [-half_w, -half_h],
                    [half_w, -half_h],
                    [half_w, half_h],
                    [-half_w, half_h],
                ];
                let radians = sprite.rotation_deg.to_radians();
                if radians != 0.0 {
                    let cos_r = radians.cos();
                    let sin_r = radians.sin();
                    for c in &mut corners {
                        let x = c[0];
                        let y = c[1];
                        c[0] = x * cos_r - y * sin_r;
                        c[1] = x * sin_r + y * cos_r;
                    }
                }

                vertices.push(SpriteVertex {
                    position: [center_x + corners[0][0], center_y + corners[0][1]],
                    tex_coords: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(SpriteVertex {
                    position: [center_x + corners[1][0], center_y + corners[1][1]],
                    tex_coords: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(SpriteVertex {
                    position: [center_x + corners[2][0], center_y + corners[2][1]],
                    tex_coords: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                vertices.push(SpriteVertex {
                    position: [center_x + corners[3][0], center_y + corners[3][1]],
                    tex_coords: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                });

                let draw_start = indices.len() as u32;
                indices.extend_from_slice(&[
                    base_index,
                    base_index + 1,
                    base_index + 2,
                    base_index,
                    base_index + 2,
                    base_index + 3,
                ]);

                draw_calls.push(DrawCall {
                    asset: sprite.asset.clone(),
                    index_start: draw_start,
                    index_count: 6,
                });
            }
        }

        (vertices, indices, draw_calls)
    }

    fn ensure_mesh_capacity(&mut self, vertex_count: usize, index_count: usize) {
        let needed_vertices = vertex_count.max(1);
        if needed_vertices > self.mesh_vertex_capacity {
            self.mesh_vertex_capacity = needed_vertices.next_power_of_two();
            self.vertex_buffer = create_vertex_buffer(&self.gpu.device, self.mesh_vertex_capacity);
        }

        let needed_indices = index_count.max(1);
        if needed_indices > self.mesh_index_capacity {
            self.mesh_index_capacity = needed_indices.next_power_of_two();
            self.index_buffer = create_index_buffer(&self.gpu.device, self.mesh_index_capacity);
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
                if state.gpu.size.0 == 0 || state.gpu.size.1 == 0 {
                    return;
                }

                state.time.begin_frame();
                let mut scene_changed = false;
                while state.time.should_step() {
                    if state.input.is_just_pressed(Key::Escape) {
                        event_loop.exit();
                        return;
                    }
                    if state.input.is_just_pressed(Key::F3) {
                        state.debug_overlay.toggle();
                    }

                    if state.input.is_just_pressed(Key::R) {
                        state.reload_scene("manual trigger (R)");
                        scene_changed = true;
                    } else if state.scene_watcher.should_reload() {
                        state.reload_scene("file watcher");
                        scene_changed = true;
                    }

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

                if scene_changed || state.time.steps_this_frame > 0 {
                    state.rebuild_scene_mesh();
                }

                let camera_uniform = state.camera.build_uniform();
                state.gpu.queue.write_buffer(
                    &state.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[camera_uniform]),
                );

                let Some((output, view)) = state.gpu.begin_frame() else {
                    return;
                };

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

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Scene Render Pass"),
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
                    render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                    for draw in &state.draw_calls {
                        if let Some(texture) = state.textures.get(&draw.asset) {
                            render_pass.set_bind_group(1, &texture.bind_group, &[]);
                            render_pass.draw_indexed(
                                draw.index_start..(draw.index_start + draw.index_count),
                                0,
                                0..1,
                            );
                        }
                    }
                }

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
                state.input.end_frame();
            }

            _ => {}
        }
    }
}

fn create_vertex_buffer(device: &wgpu::Device, vertex_capacity: usize) -> wgpu::Buffer {
    let byte_len = (vertex_capacity * std::mem::size_of::<SpriteVertex>()).max(1) as u64;
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Scene Vertex Buffer"),
        size: byte_len,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_index_buffer(device: &wgpu::Device, index_capacity: usize) -> wgpu::Buffer {
    let byte_len = (index_capacity * std::mem::size_of::<u32>()).max(1) as u64;
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Scene Index Buffer"),
        size: byte_len,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn load_texture_asset(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &SpritePipeline,
    asset_path: &str,
) -> GpuSpriteTexture {
    let bytes = std::fs::read(asset_path).unwrap_or_else(|err| {
        log::warn!(
            "Failed to read texture '{}': {}. Falling back to test sprite.",
            asset_path,
            err
        );
        FALLBACK_TEXTURE_BYTES.to_vec()
    });
    let texture = Texture::from_bytes(device, queue, &bytes, asset_path);
    let bind_group = pipeline.create_texture_bind_group(device, &texture);
    GpuSpriteTexture {
        texture,
        bind_group,
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
        KeyCode::KeyR => Some(Key::R),
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
