mod atlas;
mod collision;
mod controller;
mod lua_bridge;
#[cfg(test)]
mod replay;
mod scene;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use atlas::{load_atlas_from_path, AtlasRegistry, AtlasSpriteEntry};
use collision::{load_collision_from_path, Aabb, CollisionGrid};
use controller::{CharacterController, ControllerInput};
use lua_bridge::{ActorSnapshot, InputSnapshot, LuaBridge};
use scene::{load_scene_from_path, SceneFile, SceneWatcher, SortMode};
use sme_core::input::{InputState, Key};
use sme_core::tier::FidelityTier;
use sme_core::time::TimeState;
use sme_devtools::{DebugOverlay, OverlayStats};
use sme_platform::window::PlatformConfig;
use sme_render::{Camera2D, GpuContext, SpritePipeline, SpriteVertex, Texture};

const LUA_SCRIPT_PATH: &str = "assets/scripts/controller.lua";
const SCENE_PATH: &str = "assets/scenes/m4_scene.json";
const COLLISION_PATH: &str = "assets/collision/m3_collision.json";
const ATLAS_PATH: &str = "assets/generated/m4_sample_atlas.json";
const STRICT_SPRITE_ID_RESOLUTION: bool = true;
const FALLBACK_TEXTURE_BYTES: &[u8] = include_bytes!("../../../assets/textures/test_sprite.png");
const DEBUG_WHITE_ASSET: &str = "__debug_white";
const PLAYER_ASSET: &str = "__player";

#[derive(Debug, Clone)]
struct DrawCall {
    texture_key: Arc<str>,
    index_start: u32,
    index_count: u32,
}

struct QuadSpec<'a> {
    texture_key: &'a str,
    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
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
    collision_path: std::path::PathBuf,
    collision_watcher: SceneWatcher,
    collision_grid: CollisionGrid,
    atlas_path: std::path::PathBuf,
    atlas_watcher: SceneWatcher,
    atlas_registry: Option<AtlasRegistry>,
    character: CharacterController,
    show_collision_debug: bool,
    tier: FidelityTier,
    lua_bridge: LuaBridge,
    paused: bool,
    single_step_requested: bool,
    textures: HashMap<Arc<str>, GpuSpriteTexture>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    mesh_vertex_capacity: usize,
    mesh_index_capacity: usize,
    draw_calls: Vec<DrawCall>,
    sprite_count: usize,
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
        let collision_path = std::path::PathBuf::from(COLLISION_PATH);
        let collision_watcher = SceneWatcher::new(collision_path.clone());
        let collision_grid = load_collision_from_path(&collision_path).unwrap_or_else(|err| {
            panic!(
                "Failed to load initial collision '{}': {}",
                collision_path.display(),
                err
            );
        });
        let atlas_path = std::path::PathBuf::from(ATLAS_PATH);
        let atlas_watcher = SceneWatcher::new(atlas_path.clone());
        let atlas_registry = if atlas_path.exists() {
            match load_atlas_from_path(&atlas_path) {
                Ok(registry) => Some(registry),
                Err(err) => {
                    log::error!(
                        "Failed to load initial atlas '{}': {}",
                        atlas_path.display(),
                        err
                    );
                    None
                }
            }
        } else {
            log::warn!(
                "Atlas metadata '{}' was not found. sprite_id references will fail to resolve.",
                atlas_path.display()
            );
            None
        };
        if let Err(err) = validate_scene_sprite_references(&scene, atlas_registry.as_ref()) {
            panic!(
                "Initial scene '{}' failed sprite reference validation: {}",
                scene_path.display(),
                err
            );
        }
        if let Some(registry) = atlas_registry.as_ref() {
            if let Err(err) =
                preflight_atlas_textures(&gpu.device, &gpu.queue, &sprite_pipeline, registry)
            {
                panic!(
                    "Initial atlas '{}' failed texture preflight: {}",
                    atlas_path.display(),
                    err
                );
            }
        }

        let mut camera = Camera2D::new(gpu.size.0, gpu.size.1);
        if let Some(scene_camera) = &scene.camera {
            camera.position.x = scene_camera.start_x;
            camera.position.y = scene_camera.start_y;
            camera.zoom = scene_camera.zoom;
        }
        let cell_world = collision_grid.cell_size as f32;
        let character = CharacterController::new(Aabb {
            center_x: collision_grid.origin.x as f32 + cell_world * 2.0,
            center_y: collision_grid.origin.y as f32 + cell_world * 2.0,
            half_w: cell_world * 0.35,
            half_h: cell_world * 0.45,
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
            collision_path,
            collision_watcher,
            collision_grid,
            atlas_path,
            atlas_watcher,
            atlas_registry,
            character,
            show_collision_debug: true,
            tier: FidelityTier::default(),
            lua_bridge: LuaBridge::new(std::path::PathBuf::from(LUA_SCRIPT_PATH)),
            paused: false,
            single_step_requested: false,
            textures: HashMap::new(),
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            mesh_vertex_capacity: 0,
            mesh_index_capacity: 0,
            draw_calls: Vec::new(),
            sprite_count: 0,
        };

        // Startup order matters: load textures before building the first mesh.
        state.ensure_textures_for_scene();
        state.ensure_mesh_capacity(4, 6);
        state.rebuild_scene_mesh();
        state
    }

    fn reload_scene(&mut self, reason: &str) {
        match load_scene_from_path(&self.scene_path) {
            Ok(scene_candidate) => {
                if let Err(err) =
                    validate_scene_sprite_references(&scene_candidate, self.atlas_registry.as_ref())
                {
                    log::error!("Scene reload failed ({reason}): {err}");
                    return;
                }
                self.scene = scene_candidate;
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

    fn reload_collision(&mut self, reason: &str) {
        match load_collision_from_path(&self.collision_path) {
            Ok(grid) => {
                self.collision_grid = grid;
                self.rebuild_scene_mesh();
                log::info!(
                    "Collision reloaded ({reason}): {} ({})",
                    self.collision_grid.collision_id,
                    self.collision_grid.version
                );
            }
            Err(err) => {
                log::error!("Collision reload failed ({reason}): {err}");
            }
        }
    }

    fn reload_atlas(&mut self, reason: &str) {
        match load_atlas_from_path(&self.atlas_path) {
            Ok(registry_candidate) => {
                if let Err(err) =
                    validate_scene_sprite_references(&self.scene, Some(&registry_candidate))
                {
                    log::error!("Atlas reload failed ({reason}): {err}");
                    return;
                }
                if let Err(err) = preflight_atlas_textures(
                    &self.gpu.device,
                    &self.gpu.queue,
                    &self.sprite_pipeline,
                    &registry_candidate,
                ) {
                    log::error!("Atlas reload failed ({reason}): {err}");
                    return;
                }
                let atlas_id = registry_candidate.atlas_id.clone();
                self.atlas_registry = Some(registry_candidate);
                self.ensure_textures_for_scene();
                self.rebuild_scene_mesh();
                log::info!("Atlas reloaded ({reason}): {}", atlas_id);
            }
            Err(err) => {
                log::error!("Atlas reload failed ({reason}): {err}");
            }
        }
    }

    fn resolve_sprite_entry(&self, sprite: &scene::SceneSprite) -> Option<AtlasSpriteEntry> {
        if let Some(sprite_id) = &sprite.sprite_id {
            let Some(registry) = &self.atlas_registry else {
                log::warn!(
                    "Sprite '{}' references sprite_id '{}' but no atlas is loaded",
                    sprite.id,
                    sprite_id
                );
                return None;
            };
            let Some(entry) = registry.resolve(sprite_id) else {
                log::warn!(
                    "Sprite '{}' references missing sprite_id '{}' in atlas '{}'",
                    sprite.id,
                    sprite_id,
                    registry.atlas_id
                );
                return None;
            };
            return Some(entry.clone());
        }

        let Some(asset) = &sprite.asset else {
            return None;
        };
        Some(AtlasSpriteEntry {
            texture_path: asset.clone(),
            size_px: (0, 0),
            uv: [0.0, 0.0, 1.0, 1.0],
            pivot: (0.5, 0.5),
        })
    }

    fn ensure_textures_for_scene(&mut self) {
        let mut required_assets = HashSet::new();
        for layer in &self.scene.layers {
            for sprite in &layer.sprites {
                if let Some(entry) = self.resolve_sprite_entry(sprite) {
                    required_assets.insert(entry.texture_path);
                }
            }
        }

        for asset_path in required_assets {
            if self.textures.contains_key(asset_path.as_str()) {
                continue;
            }
            let texture = load_texture_asset(
                &self.gpu.device,
                &self.gpu.queue,
                &self.sprite_pipeline,
                &asset_path,
            );
            self.textures.insert(Arc::from(asset_path), texture);
        }

        if !self.textures.contains_key(DEBUG_WHITE_ASSET) {
            let texture = Texture::from_rgba8(
                &self.gpu.device,
                &self.gpu.queue,
                &[255, 255, 255, 255],
                1,
                1,
                "debug_white",
            );
            let bind_group = self
                .sprite_pipeline
                .create_texture_bind_group(&self.gpu.device, &texture);
            self.textures.insert(
                Arc::from(DEBUG_WHITE_ASSET),
                GpuSpriteTexture {
                    texture,
                    bind_group,
                },
            );
        }
        if !self.textures.contains_key(PLAYER_ASSET) {
            let texture = Texture::from_rgba8(
                &self.gpu.device,
                &self.gpu.queue,
                &[255, 64, 64, 255],
                1,
                1,
                "player_debug",
            );
            let bind_group = self
                .sprite_pipeline
                .create_texture_bind_group(&self.gpu.device, &texture);
            self.textures.insert(
                Arc::from(PLAYER_ASSET),
                GpuSpriteTexture {
                    texture,
                    bind_group,
                },
            );
        }
    }

    fn estimate_memory_mb(&self) -> f32 {
        let mut bytes: usize = 0;
        // Texture memory (width * height * 4 bytes per pixel)
        for tex in self.textures.values() {
            let (w, h) = tex.texture.size;
            bytes += (w as usize) * (h as usize) * 4;
        }
        // GPU buffer memory
        bytes += self.mesh_vertex_capacity * std::mem::size_of::<SpriteVertex>();
        bytes += self.mesh_index_capacity * std::mem::size_of::<u32>();
        bytes as f32 / (1024.0 * 1024.0)
    }

    fn rebuild_scene_mesh(&mut self) {
        // Build a single CPU-side mesh each frame from scene + debug overlays,
        // then stream it into GPU buffers.
        let (vertices, indices, draw_calls) = self.build_mesh();
        self.ensure_mesh_capacity(vertices.len(), indices.len());
        self.sprite_count = vertices.len() / 4;
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
        // Tier2 gets a subtle warm color boost for "PC polish" feel.
        let tier_color = match self.tier {
            FidelityTier::Tier0 => [1.0f32, 1.0, 1.0, 1.0],
            FidelityTier::Tier2 => [1.05f32, 1.02, 0.98, 1.0],
        };

        let sprite_count_estimate: usize = self
            .scene
            .layers
            .iter()
            .filter(|l| l.visible)
            .map(|l| l.sprites.len())
            .sum::<usize>()
            + 64; // padding for debug overlays + player
        let mut vertices = Vec::with_capacity(sprite_count_estimate * 4);
        let mut indices = Vec::with_capacity(sprite_count_estimate * 6);
        let mut draw_calls = Vec::with_capacity(16);

        // Visual scene layers render back-to-front according to authored order.
        for layer in &self.scene.layers {
            if !layer.visible {
                continue;
            }

            let sprite_indices: Vec<usize> = if matches!(layer.sort_mode, SortMode::Y) {
                let mut indices_vec: Vec<usize> = (0..layer.sprites.len()).collect();
                indices_vec.sort_by(|&a, &b| {
                    layer.sprites[a]
                        .y
                        .partial_cmp(&layer.sprites[b].y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            layer.sprites[a]
                                .z
                                .partial_cmp(&layer.sprites[b].z)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                });
                indices_vec
            } else {
                (0..layer.sprites.len()).collect()
            };

            if layer.occlusion {
                log::trace!("Rendering occlusion layer '{}'", layer.id);
            }

            // Parallax is implemented as a per-layer camera-space offset.
            let parallax_offset = self.camera.position * (1.0 - layer.parallax);
            for &sprite_idx in &sprite_indices {
                let sprite = &layer.sprites[sprite_idx];
                let Some(sprite_entry) = self.resolve_sprite_entry(sprite) else {
                    log::warn!(
                        "Skipping sprite '{}' due to unresolved asset reference",
                        sprite.id
                    );
                    continue;
                };
                let Some(texture) = self.textures.get(sprite_entry.texture_path.as_str()) else {
                    log::warn!("Skipping sprite '{}' due to missing texture", sprite.id);
                    continue;
                };

                let center_x = sprite.x + parallax_offset.x;
                let center_y = sprite.y + parallax_offset.y;
                let source_size = if sprite.sprite_id.is_some() {
                    sprite_entry.size_px
                } else {
                    texture.texture.size
                };
                let sprite_w = source_size.0 as f32 * sprite.scale_x;
                let sprite_h = source_size.1 as f32 * sprite.scale_y;
                let (pivot_x, pivot_y) = sprite_entry.pivot;
                let left = -sprite_w * pivot_x;
                let right = sprite_w * (1.0 - pivot_x);
                let bottom = -sprite_h * pivot_y;
                let top = sprite_h * (1.0 - pivot_y);
                let base_index = vertices.len() as u32;

                let mut corners = [[left, bottom], [right, bottom], [right, top], [left, top]];
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

                let [u0, v0, u1, v1] = sprite_entry.uv;
                vertices.push(SpriteVertex {
                    position: [center_x + corners[0][0], center_y + corners[0][1]],
                    tex_coords: [u0, v1],
                    color: tier_color,
                });
                vertices.push(SpriteVertex {
                    position: [center_x + corners[1][0], center_y + corners[1][1]],
                    tex_coords: [u1, v1],
                    color: tier_color,
                });
                vertices.push(SpriteVertex {
                    position: [center_x + corners[2][0], center_y + corners[2][1]],
                    tex_coords: [u1, v0],
                    color: tier_color,
                });
                vertices.push(SpriteVertex {
                    position: [center_x + corners[3][0], center_y + corners[3][1]],
                    tex_coords: [u0, v0],
                    color: tier_color,
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

                push_draw_call(
                    &mut draw_calls,
                    Arc::from(sprite_entry.texture_path.as_str()),
                    draw_start,
                    6,
                );
            }
        }

        // Debug collision overlay is rendered as translucent quads in world space.
        if self.show_collision_debug {
            let cell = self.collision_grid.cell_size as f32;
            for solid in self.collision_grid.solids_iter() {
                let center_x = self.collision_grid.origin.x as f32 + (solid.x as f32 + 0.5) * cell;
                let center_y = self.collision_grid.origin.y as f32 + (solid.y as f32 + 0.5) * cell;
                add_quad(
                    &mut vertices,
                    &mut indices,
                    &mut draw_calls,
                    QuadSpec {
                        texture_key: DEBUG_WHITE_ASSET,
                        center_x,
                        center_y,
                        width: cell,
                        height: cell,
                        color: [0.15, 0.9, 0.15, 0.35],
                    },
                );
            }
        }

        // Player visualization uses a simple debug quad driven by controller AABB.
        add_quad(
            &mut vertices,
            &mut indices,
            &mut draw_calls,
            QuadSpec {
                texture_key: PLAYER_ASSET,
                center_x: self.character.aabb.center_x,
                center_y: self.character.aabb.center_y,
                width: self.character.aabb.half_w * 2.0,
                height: self.character.aabb.half_h * 2.0,
                color: [1.0, 0.3, 0.3, 0.9],
            },
        );

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

                // Fixed-step simulation phase.
                state.time.begin_frame();
                let mut scene_changed = false;

                // Check for Lua script reload at frame boundary (safe point)
                state.lua_bridge.check_reload();
                if state.input.is_just_pressed(Key::R) {
                    state.lua_bridge.force_reload();
                }

                while state.time.should_step() {
                    if state.input.is_just_pressed(Key::Escape) {
                        event_loop.exit();
                        return;
                    }
                    if state.input.is_just_pressed(Key::F3) {
                        state.debug_overlay.toggle();
                    }
                    if state.input.is_just_pressed(Key::F4) {
                        state.show_collision_debug = !state.show_collision_debug;
                        scene_changed = true;
                        log::info!(
                            "Collision debug: {}",
                            if state.show_collision_debug {
                                "ON"
                            } else {
                                "OFF"
                            }
                        );
                    }
                    if state.input.is_just_pressed(Key::F5) {
                        state.tier = state.tier.next();
                        log::info!("Fidelity tier: {}", state.tier);
                    }

                    if state.input.is_just_pressed(Key::R) {
                        state.reload_scene("manual trigger (R)");
                        state.reload_collision("manual trigger (R)");
                        state.reload_atlas("manual trigger (R)");
                        scene_changed = true;
                    } else if state.scene_watcher.should_reload() {
                        state.reload_scene("file watcher");
                        scene_changed = true;
                    } else if state.collision_watcher.should_reload() {
                        state.reload_collision("file watcher");
                        scene_changed = true;
                    } else if state.atlas_watcher.should_reload() {
                        state.reload_atlas("file watcher");
                        scene_changed = true;
                    }

                    // Skip simulation update when paused (unless single-step requested)
                    if state.paused && !state.single_step_requested {
                        break;
                    }
                    state.single_step_requested = false;

                    // Build input snapshot for Lua
                    let input_snapshot = build_input_snapshot(&state.input);
                    let actor_snapshot = ActorSnapshot {
                        grounded: state.character.grounded,
                        velocity_x: state.character.velocity_x,
                        velocity_y: state.character.velocity_y,
                    };

                    // Try Lua controller first, fall back to Rust
                    let dt = state.time.fixed_dt as f32;
                    let controller_input = if let Some(intent) =
                        state
                            .lua_bridge
                            .call_update(dt, &input_snapshot, &actor_snapshot)
                    {
                        ControllerInput {
                            move_x: intent.move_x,
                            jump_pressed: intent.jump_pressed,
                        }
                    } else {
                        // Rust fallback controller (identical logic to the Lua script)
                        let mut move_x: f32 = 0.0;
                        if state.input.is_held(Key::Left) || state.input.is_held(Key::A) {
                            move_x -= 1.0;
                        }
                        if state.input.is_held(Key::Right) || state.input.is_held(Key::D) {
                            move_x += 1.0;
                        }
                        let jump_pressed = state.input.is_just_pressed(Key::Space)
                            || state.input.is_just_pressed(Key::W)
                            || state.input.is_just_pressed(Key::Up);
                        ControllerInput {
                            move_x,
                            jump_pressed,
                        }
                    };

                    state
                        .character
                        .step(controller_input, dt, &state.collision_grid);

                    state.camera.position.x = state.character.aabb.center_x;
                    state.camera.position.y = state.character.aabb.center_y;
                }
                state.time.end_frame();

                if scene_changed || state.time.steps_this_frame > 0 {
                    state.rebuild_scene_mesh();
                }

                // Render phase reads finalized simulation state from this frame.
                let camera_uniform = state.camera.build_uniform();
                state.gpu.queue.write_buffer(
                    &state.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[camera_uniform]),
                );

                let Some((output, view)) = state.gpu.begin_frame() else {
                    return;
                };

                let predicted_bind_count = count_texture_binds(&state.draw_calls);
                let (egui_primitives, egui_textures_delta, overlay_actions) =
                    state.debug_overlay.prepare(
                        &state.window,
                        &state.time,
                        Some(OverlayStats {
                            draw_calls: state.draw_calls.len() as u32,
                            atlas_binds: predicted_bind_count as u32,
                            sprite_count: state.sprite_count as u32,
                            memory_estimate_mb: state.estimate_memory_mb(),
                            tier_label: state.tier.label().to_string(),
                            lua_status_label: state.lua_bridge.status().label().to_string(),
                            paused: state.paused,
                        }),
                    );

                // Handle overlay button actions
                if overlay_actions.cycle_tier {
                    state.tier = state.tier.next();
                    log::info!("Fidelity tier (overlay): {}", state.tier);
                }
                if overlay_actions.toggle_pause {
                    state.paused = !state.paused;
                    log::info!(
                        "Simulation {}",
                        if state.paused { "PAUSED" } else { "RESUMED" }
                    );
                }
                if overlay_actions.single_step {
                    state.single_step_requested = true;
                }
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
                    let clear_color = match state.tier {
                        FidelityTier::Tier0 => wgpu::Color {
                            r: 0.392,
                            g: 0.584,
                            b: 0.929,
                            a: 1.0,
                        },
                        FidelityTier::Tier2 => wgpu::Color {
                            r: 0.35,
                            g: 0.55,
                            b: 0.95,
                            a: 1.0,
                        },
                    };
                    let mut last_bound_texture_key: Option<&Arc<str>> = None;
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Scene Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(clear_color),
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
                        if let Some(texture) = state.textures.get(&draw.texture_key) {
                            let need_rebind = match last_bound_texture_key {
                                Some(last) => **last != *draw.texture_key,
                                None => true,
                            };
                            if need_rebind {
                                render_pass.set_bind_group(1, &texture.bind_group, &[]);
                                last_bound_texture_key = Some(&draw.texture_key);
                            }
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

                // Only clear edge-triggered input (just_pressed / just_released)
                // after at least one fixed step consumed it. Otherwise a press
                // that lands on a frame with 0 simulation steps is silently lost.
                if state.time.steps_this_frame > 0 {
                    state.input.end_frame();
                }
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

fn add_quad(
    vertices: &mut Vec<SpriteVertex>,
    indices: &mut Vec<u32>,
    draw_calls: &mut Vec<DrawCall>,
    spec: QuadSpec<'_>,
) {
    let half_w = spec.width * 0.5;
    let half_h = spec.height * 0.5;
    let base_index = vertices.len() as u32;

    vertices.push(SpriteVertex {
        position: [spec.center_x - half_w, spec.center_y - half_h],
        tex_coords: [0.0, 1.0],
        color: spec.color,
    });
    vertices.push(SpriteVertex {
        position: [spec.center_x + half_w, spec.center_y - half_h],
        tex_coords: [1.0, 1.0],
        color: spec.color,
    });
    vertices.push(SpriteVertex {
        position: [spec.center_x + half_w, spec.center_y + half_h],
        tex_coords: [1.0, 0.0],
        color: spec.color,
    });
    vertices.push(SpriteVertex {
        position: [spec.center_x - half_w, spec.center_y + half_h],
        tex_coords: [0.0, 0.0],
        color: spec.color,
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

    push_draw_call(draw_calls, Arc::from(spec.texture_key), draw_start, 6);
}

fn push_draw_call(
    draw_calls: &mut Vec<DrawCall>,
    texture_key: Arc<str>,
    index_start: u32,
    index_count: u32,
) {
    if let Some(last) = draw_calls.last_mut() {
        let contiguous = last.index_start + last.index_count == index_start;
        if *last.texture_key == *texture_key && contiguous {
            last.index_count += index_count;
            return;
        }
    }
    draw_calls.push(DrawCall {
        texture_key,
        index_start,
        index_count,
    });
}

fn load_texture_asset(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &SpritePipeline,
    asset_path: &str,
) -> GpuSpriteTexture {
    let bytes_owned;
    let bytes: &[u8] = match std::fs::read(asset_path) {
        Ok(data) => {
            bytes_owned = data;
            &bytes_owned
        }
        Err(err) => {
            log::warn!(
                "Failed to read texture '{}': {}. Falling back to test sprite.",
                asset_path,
                err
            );
            FALLBACK_TEXTURE_BYTES
        }
    };
    let texture = Texture::from_bytes(device, queue, bytes, asset_path);
    let bind_group = pipeline.create_texture_bind_group(device, &texture);
    GpuSpriteTexture {
        texture,
        bind_group,
    }
}

fn load_texture_asset_strict(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &SpritePipeline,
    asset_path: &str,
) -> Result<GpuSpriteTexture, String> {
    let bytes = std::fs::read(asset_path)
        .map_err(|e| format!("Failed to read texture '{}': {e}", asset_path))?;
    let texture = Texture::from_bytes(device, queue, &bytes, asset_path);
    let bind_group = pipeline.create_texture_bind_group(device, &texture);
    Ok(GpuSpriteTexture {
        texture,
        bind_group,
    })
}

fn preflight_atlas_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &SpritePipeline,
    atlas_registry: &AtlasRegistry,
) -> Result<(), String> {
    let mut required_textures = std::collections::HashSet::new();
    for entry in atlas_registry.sprite_entries.values() {
        required_textures.insert(entry.texture_path.as_str());
    }
    for texture_path in required_textures {
        let _ = load_texture_asset_strict(device, queue, pipeline, texture_path)?;
    }
    Ok(())
}

fn count_texture_binds(draw_calls: &[DrawCall]) -> usize {
    let mut binds = 0usize;
    let mut current: Option<&str> = None;
    for draw in draw_calls {
        let key: &str = &draw.texture_key;
        if current != Some(key) {
            current = Some(key);
            binds += 1;
        }
    }
    binds
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
        KeyCode::F4 => Some(Key::F4),
        KeyCode::F5 => Some(Key::F5),
        KeyCode::KeyW => Some(Key::W),
        KeyCode::KeyA => Some(Key::A),
        KeyCode::KeyS => Some(Key::S),
        KeyCode::KeyD => Some(Key::D),
        KeyCode::KeyR => Some(Key::R),
        _ => None,
    }
}

fn build_input_snapshot(input: &InputState) -> InputSnapshot {
    let key_names: &[(Key, &str)] = &[
        (Key::Left, "left"),
        (Key::Right, "right"),
        (Key::Up, "up"),
        (Key::Down, "down"),
        (Key::Space, "space"),
        (Key::W, "w"),
        (Key::A, "a"),
        (Key::S, "s"),
        (Key::D, "d"),
    ];

    let mut held_keys = Vec::new();
    let mut just_pressed_keys = Vec::new();
    for &(key, name) in key_names {
        if input.is_held(key) {
            held_keys.push(name.to_string());
        }
        if input.is_just_pressed(key) {
            just_pressed_keys.push(name.to_string());
        }
    }

    InputSnapshot {
        held_keys,
        just_pressed_keys,
    }
}

fn validate_scene_sprite_references(
    scene: &SceneFile,
    atlas_registry: Option<&AtlasRegistry>,
) -> Result<(), String> {
    if !STRICT_SPRITE_ID_RESOLUTION {
        return Ok(());
    }

    for layer in &scene.layers {
        for sprite in &layer.sprites {
            let Some(sprite_id) = &sprite.sprite_id else {
                continue;
            };
            let registry = atlas_registry.ok_or_else(|| {
                format!(
                    "sprite '{}' references sprite_id '{}' but no atlas metadata is loaded",
                    sprite.id, sprite_id
                )
            })?;
            if registry.resolve(sprite_id).is_none() {
                return Err(format!(
                    "sprite '{}' references missing sprite_id '{}' in atlas '{}'",
                    sprite.id, sprite_id, registry.atlas_id
                ));
            }
        }
    }

    Ok(())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Saturday Morning Engine starting...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
