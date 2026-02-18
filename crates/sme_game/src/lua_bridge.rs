//! Rust <-> Lua bridge for gameplay scripting.
//!
//! Design contract: Lua scripts provide **movement intents** (direction + jump),
//! never direct mutation of physics state. Rust owns the simulation truth --
//! Lua only reads actor state (grounded, velocity) and writes into a
//! `_intent` table that Rust reads back after `on_update(dt)` returns.
//!
//! Input is exposed via lookup tables (`_held` / `_just_pressed`) rather than
//! per-key functions, so the Rust side can bulk-set the entire input snapshot
//! in one pass without creating closures per key.
//!
//! Reload strategy: on file change (mtime polling) or manual trigger (R key),
//! a **fresh Lua state** is created and the script is re-executed from scratch.
//! This avoids stale globals and leaked state at the cost of losing any
//! in-memory Lua variables -- acceptable because all persistent state lives
//! in Rust (CharacterController, etc.).

use std::path::PathBuf;
use std::time::SystemTime;

use mlua::prelude::*;

/// Intent returned by Lua's on_update — describes desired motion, not direct mutation.
#[derive(Debug, Clone, Default)]
pub struct LuaIntent {
    pub move_x: f32,
    pub jump_pressed: bool,
    pub play_animation: Option<String>,
    pub stop_animation: bool,
}

/// Status of the Lua runtime for display in the debug overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaStatus {
    /// Script loaded and running normally.
    Loaded,
    /// Script had an error; engine is using Rust fallback controller.
    Error,
    /// No script file found; engine is using Rust fallback controller.
    Fallback,
}

impl LuaStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Loaded => "Lua: loaded",
            Self::Error => "Lua: ERROR",
            Self::Fallback => "Lua: fallback",
        }
    }
}

impl std::fmt::Display for LuaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Snapshot of engine state passed to Lua each frame.
pub struct ActorSnapshot {
    pub grounded: bool,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub current_animation: Option<String>,
    pub animation_finished: bool,
}

/// Snapshot of input state passed to Lua each frame.
pub struct InputSnapshot {
    pub held_keys: Vec<String>,
    pub just_pressed_keys: Vec<String>,
}

pub struct LuaBridge {
    lua: Lua,
    script_path: PathBuf,
    last_modified: Option<SystemTime>,
    status: LuaStatus,
    last_error: Option<String>,
}

impl LuaBridge {
    /// Create a new LuaBridge. If the script file doesn't exist, starts in Fallback mode.
    pub fn new(script_path: PathBuf) -> Self {
        let lua = Lua::new();
        let mut bridge = Self {
            lua,
            script_path,
            last_modified: None,
            status: LuaStatus::Fallback,
            last_error: None,
        };
        bridge.try_load_script();
        bridge
    }

    pub fn status(&self) -> LuaStatus {
        self.status
    }

    #[allow(dead_code)]
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Check if the script file has been modified and reload if needed.
    /// Call this once per frame at a safe boundary (between frames, not mid-step).
    pub fn check_reload(&mut self) {
        let current_mtime = match std::fs::metadata(&self.script_path) {
            Ok(meta) => meta.modified().ok(),
            Err(_) => return,
        };

        if current_mtime != self.last_modified {
            log::info!(
                "Lua script changed, reloading: {}",
                self.script_path.display()
            );
            self.try_load_script();
        }
    }

    /// Force a reload of the script (e.g. when user presses R).
    pub fn force_reload(&mut self) {
        log::info!("Lua script force reload: {}", self.script_path.display());
        self.try_load_script();
    }

    /// Call the Lua on_update(dt) function with current engine state.
    /// Returns the intent from Lua, or None if Lua is not available.
    pub fn call_update(
        &self,
        dt: f32,
        input: &InputSnapshot,
        actor: &ActorSnapshot,
    ) -> Option<LuaIntent> {
        if self.status != LuaStatus::Loaded {
            return None;
        }

        match self.call_update_inner(dt, input, actor) {
            Ok(intent) => Some(intent),
            Err(err) => {
                log::error!("Lua on_update error: {}", err);
                None
            }
        }
    }

    fn call_update_inner(
        &self,
        dt: f32,
        input: &InputSnapshot,
        actor: &ActorSnapshot,
    ) -> LuaResult<LuaIntent> {
        // Set up the engine.input table
        let engine: LuaTable = self.lua.globals().get("engine")?;
        let input_table: LuaTable = engine.get("input")?;
        let actor_table: LuaTable = engine.get("actor")?;

        // Update held keys set
        let held_set = self.lua.create_table()?;
        for key in &input.held_keys {
            held_set.set(key.as_str(), true)?;
        }
        input_table.set("_held", held_set)?;

        // Update just_pressed keys set
        let pressed_set = self.lua.create_table()?;
        for key in &input.just_pressed_keys {
            pressed_set.set(key.as_str(), true)?;
        }
        input_table.set("_just_pressed", pressed_set)?;

        // Update actor state
        actor_table.set("grounded", actor.grounded)?;
        actor_table.set("velocity_x", actor.velocity_x)?;
        actor_table.set("velocity_y", actor.velocity_y)?;
        match &actor.current_animation {
            Some(name) => actor_table.set("current_animation", name.as_str())?,
            None => actor_table.set("current_animation", LuaValue::Nil)?,
        }
        actor_table.set("animation_finished", actor.animation_finished)?;

        // Reset intent
        let intent_table: LuaTable = engine.get("_intent")?;
        intent_table.set("move_x", 0.0f32)?;
        intent_table.set("jump_pressed", false)?;
        intent_table.set("play_animation", LuaValue::Nil)?;
        intent_table.set("stop_animation", false)?;

        // Call on_update(dt)
        let on_update: LuaFunction = self.lua.globals().get("on_update")?;
        on_update.call::<()>(dt)?;

        // Read back intent
        let move_x: f32 = intent_table.get("move_x")?;
        let jump_pressed: bool = intent_table.get("jump_pressed")?;
        let play_animation: Option<String> = intent_table.get("play_animation").ok();
        let stop_animation: bool = intent_table.get("stop_animation").unwrap_or(false);

        Ok(LuaIntent {
            move_x,
            jump_pressed,
            play_animation,
            stop_animation,
        })
    }

    fn try_load_script(&mut self) {
        if !self.script_path.exists() {
            log::warn!(
                "Lua script not found: {}. Using Rust fallback.",
                self.script_path.display()
            );
            self.status = LuaStatus::Fallback;
            self.last_error = None;
            self.last_modified = None;
            return;
        }

        // Record mtime before loading
        self.last_modified = std::fs::metadata(&self.script_path)
            .ok()
            .and_then(|m| m.modified().ok());

        // Create a fresh Lua state to avoid stale globals
        self.lua = Lua::new();

        if let Err(err) = self.setup_engine_api() {
            let msg = format!("Failed to setup Lua engine API: {}", err);
            log::error!("{}", msg);
            self.status = LuaStatus::Error;
            self.last_error = Some(msg);
            return;
        }

        match std::fs::read_to_string(&self.script_path) {
            Ok(source) => {
                match self
                    .lua
                    .load(&source)
                    .set_name(self.script_path.to_string_lossy())
                    .exec()
                {
                    Ok(()) => {
                        self.status = LuaStatus::Loaded;
                        self.last_error = None;
                        log::info!("Lua script loaded: {}", self.script_path.display());

                        // Call on_init() if present
                        if let Ok(on_init) = self.lua.globals().get::<LuaFunction>("on_init") {
                            if let Err(err) = on_init.call::<()>(()) {
                                log::error!("Lua on_init error: {}", err);
                                // Don't fail the whole load over on_init error
                            }
                        }
                    }
                    Err(err) => {
                        let msg = format!("Lua script load error: {}", err);
                        log::error!("{}", msg);
                        self.status = LuaStatus::Error;
                        self.last_error = Some(msg);
                    }
                }
            }
            Err(err) => {
                let msg = format!("Failed to read Lua script: {}", err);
                log::error!("{}", msg);
                self.status = LuaStatus::Error;
                self.last_error = Some(msg);
            }
        }
    }

    /// Build the `engine` global table that Lua scripts interact with.
    ///
    /// Layout:
    ///   engine.input._held        -- table of key->true for currently held keys
    ///   engine.input._just_pressed -- table of key->true for edge-triggered presses
    ///   engine.input.is_held(key)  -- convenience wrapper over _held lookup
    ///   engine.input.is_just_pressed(key) -- convenience wrapper over _just_pressed
    ///   engine.actor.grounded     -- read-only bool, set by Rust each frame
    ///   engine.actor.velocity_x/y -- read-only floats, set by Rust each frame
    ///   engine.actor.set_intent(move_x, jump_pressed) -- Lua writes intent here
    ///   engine._intent            -- internal table read by Rust after on_update
    fn setup_engine_api(&self) -> LuaResult<()> {
        let lua = &self.lua;
        let engine = lua.create_table()?;

        // engine.input table with helper methods
        let input_table = lua.create_table()?;
        let held_set = lua.create_table()?;
        let pressed_set = lua.create_table()?;
        input_table.set("_held", held_set)?;
        input_table.set("_just_pressed", pressed_set)?;

        // engine.input.is_held(key) -> bool
        let is_held = lua.create_function(|lua_ctx, key: String| {
            let engine: LuaTable = lua_ctx.globals().get("engine")?;
            let input: LuaTable = engine.get("input")?;
            let held: LuaTable = input.get("_held")?;
            let result: bool = held.get::<bool>(key.as_str()).unwrap_or(false);
            Ok(result)
        })?;
        input_table.set("is_held", is_held)?;

        // engine.input.is_just_pressed(key) -> bool
        let is_just_pressed = lua.create_function(|lua_ctx, key: String| {
            let engine: LuaTable = lua_ctx.globals().get("engine")?;
            let input: LuaTable = engine.get("input")?;
            let pressed: LuaTable = input.get("_just_pressed")?;
            let result: bool = pressed.get::<bool>(key.as_str()).unwrap_or(false);
            Ok(result)
        })?;
        input_table.set("is_just_pressed", is_just_pressed)?;

        engine.set("input", input_table)?;

        // engine.actor table (read-only state, updated each frame from Rust)
        let actor_table = lua.create_table()?;
        actor_table.set("grounded", false)?;
        actor_table.set("velocity_x", 0.0f32)?;
        actor_table.set("velocity_y", 0.0f32)?;

        // engine.actor.set_intent(move_x, jump_pressed)
        let set_intent = lua.create_function(|lua_ctx, (move_x, jump_pressed): (f32, bool)| {
            let engine: LuaTable = lua_ctx.globals().get("engine")?;
            let intent: LuaTable = engine.get("_intent")?;
            intent.set("move_x", move_x)?;
            intent.set("jump_pressed", jump_pressed)?;
            Ok(())
        })?;
        actor_table.set("set_intent", set_intent)?;

        // engine.actor.play_animation(name)
        let play_animation = lua.create_function(|lua_ctx, name: String| {
            let engine: LuaTable = lua_ctx.globals().get("engine")?;
            let intent: LuaTable = engine.get("_intent")?;
            intent.set("play_animation", name)?;
            Ok(())
        })?;
        actor_table.set("play_animation", play_animation)?;

        // engine.actor.stop_animation()
        let stop_animation = lua.create_function(|lua_ctx, ()| {
            let engine: LuaTable = lua_ctx.globals().get("engine")?;
            let intent: LuaTable = engine.get("_intent")?;
            intent.set("stop_animation", true)?;
            Ok(())
        })?;
        actor_table.set("stop_animation", stop_animation)?;

        // Read-only animation state
        actor_table.set("current_animation", LuaValue::Nil)?;
        actor_table.set("animation_finished", false)?;

        engine.set("actor", actor_table)?;

        // engine._intent (internal, read by Rust after on_update)
        let intent_table = lua.create_table()?;
        intent_table.set("move_x", 0.0f32)?;
        intent_table.set("jump_pressed", false)?;
        engine.set("_intent", intent_table)?;

        lua.globals().set("engine", engine)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const VALID_LUA_SCRIPT: &str = r#"
function on_update(dt)
    engine.actor.set_intent(1.0, true)
end
"#;

    const INVALID_LUA_SCRIPT: &str = "this is not valid lua !@#$";

    fn temp_lua_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "sme_test_lua_bridge_{}_{}.lua",
            name,
            std::process::id()
        ));
        path
    }

    fn write_temp_script(path: &PathBuf, content: &str) {
        let mut f = std::fs::File::create(path).expect("failed to create temp script");
        f.write_all(content.as_bytes())
            .expect("failed to write temp script");
        f.flush().expect("failed to flush temp script");
    }

    fn make_input() -> InputSnapshot {
        InputSnapshot {
            held_keys: vec![],
            just_pressed_keys: vec![],
        }
    }

    fn make_actor() -> ActorSnapshot {
        ActorSnapshot {
            grounded: false,
            velocity_x: 0.0,
            velocity_y: 0.0,
            current_animation: None,
            animation_finished: false,
        }
    }

    #[test]
    fn lua_status_labels() {
        let variants = [LuaStatus::Loaded, LuaStatus::Error, LuaStatus::Fallback];
        for variant in &variants {
            let label = variant.label();
            assert!(
                !label.is_empty(),
                "{:?} should have a non-empty label",
                variant
            );
        }
    }

    #[test]
    fn lua_status_display() {
        let variants = [LuaStatus::Loaded, LuaStatus::Error, LuaStatus::Fallback];
        for variant in &variants {
            let display = format!("{}", variant);
            assert_eq!(
                display,
                variant.label(),
                "Display for {:?} should match label()",
                variant
            );
        }
    }

    #[test]
    fn lua_intent_default() {
        let intent = LuaIntent::default();
        assert_eq!(intent.move_x, 0.0);
        assert!(!intent.jump_pressed);
    }

    #[test]
    fn bridge_fallback_when_no_script() {
        let path = PathBuf::from("__nonexistent_script_for_test_42__.lua");
        let bridge = LuaBridge::new(path);
        assert_eq!(bridge.status(), LuaStatus::Fallback);
    }

    #[test]
    fn bridge_loads_valid_script() {
        let path = temp_lua_path("valid");
        write_temp_script(&path, VALID_LUA_SCRIPT);

        let bridge = LuaBridge::new(path.clone());
        assert_eq!(
            bridge.status(),
            LuaStatus::Loaded,
            "Expected Loaded, got {:?}. Error: {:?}",
            bridge.status(),
            bridge.last_error()
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bridge_error_on_invalid_script() {
        let path = temp_lua_path("invalid");
        write_temp_script(&path, INVALID_LUA_SCRIPT);

        let bridge = LuaBridge::new(path.clone());
        assert_eq!(bridge.status(), LuaStatus::Error);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bridge_call_update_returns_intent() {
        let path = temp_lua_path("intent");
        write_temp_script(&path, VALID_LUA_SCRIPT);

        let bridge = LuaBridge::new(path.clone());
        assert_eq!(bridge.status(), LuaStatus::Loaded);

        let input = make_input();
        let actor = make_actor();
        let intent = bridge
            .call_update(1.0 / 60.0, &input, &actor)
            .expect("call_update should return Some(intent)");

        assert!(
            (intent.move_x - 1.0).abs() < f32::EPSILON,
            "move_x should be 1.0, got {}",
            intent.move_x
        );
        assert!(intent.jump_pressed, "jump_pressed should be true");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bridge_call_update_returns_none_when_fallback() {
        let path = PathBuf::from("__nonexistent_script_for_test_none__.lua");
        let bridge = LuaBridge::new(path);
        assert_eq!(bridge.status(), LuaStatus::Fallback);

        let input = make_input();
        let actor = make_actor();
        let result = bridge.call_update(1.0 / 60.0, &input, &actor);
        assert!(
            result.is_none(),
            "call_update should return None in Fallback mode"
        );
    }

    #[test]
    fn bridge_force_reload() {
        let path = temp_lua_path("reload");
        // Start with no file -- should be Fallback
        let _ = std::fs::remove_file(&path);
        let mut bridge = LuaBridge::new(path.clone());
        assert_eq!(bridge.status(), LuaStatus::Fallback);

        // Now write a valid script and force reload
        write_temp_script(&path, VALID_LUA_SCRIPT);
        bridge.force_reload();
        assert_eq!(
            bridge.status(),
            LuaStatus::Loaded,
            "After force_reload with valid script, status should be Loaded. Error: {:?}",
            bridge.last_error()
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn lua_determinism_same_inputs_same_outputs() {
        // Write a Lua script that makes decisions based on input state
        let path = temp_lua_path("determinism");
        write_temp_script(
            &path,
            r#"
function on_update(dt)
    local move_x = 0
    if engine.input.is_held("right") then
        move_x = 1
    elseif engine.input.is_held("left") then
        move_x = -1
    end
    local jump = engine.input.is_just_pressed("space")
    engine.actor.set_intent(move_x, jump)
end
"#,
        );

        let dt = 1.0 / 60.0;

        // Define a fixed input sequence (simulating gameplay)
        let input_sequence: Vec<InputSnapshot> = vec![
            // Standing still
            InputSnapshot {
                held_keys: vec![],
                just_pressed_keys: vec![],
            },
            InputSnapshot {
                held_keys: vec![],
                just_pressed_keys: vec![],
            },
            // Start moving right
            InputSnapshot {
                held_keys: vec!["right".to_string()],
                just_pressed_keys: vec!["right".to_string()],
            },
            InputSnapshot {
                held_keys: vec!["right".to_string()],
                just_pressed_keys: vec![],
            },
            InputSnapshot {
                held_keys: vec!["right".to_string()],
                just_pressed_keys: vec![],
            },
            // Jump while moving
            InputSnapshot {
                held_keys: vec!["right".to_string(), "space".to_string()],
                just_pressed_keys: vec!["space".to_string()],
            },
            InputSnapshot {
                held_keys: vec!["right".to_string()],
                just_pressed_keys: vec![],
            },
            InputSnapshot {
                held_keys: vec!["right".to_string()],
                just_pressed_keys: vec![],
            },
            // Stop moving
            InputSnapshot {
                held_keys: vec![],
                just_pressed_keys: vec![],
            },
            InputSnapshot {
                held_keys: vec![],
                just_pressed_keys: vec![],
            },
            // Move left
            InputSnapshot {
                held_keys: vec!["left".to_string()],
                just_pressed_keys: vec!["left".to_string()],
            },
            InputSnapshot {
                held_keys: vec!["left".to_string()],
                just_pressed_keys: vec![],
            },
        ];

        let actor = make_actor();

        // Run A
        let bridge_a = LuaBridge::new(path.clone());
        assert_eq!(bridge_a.status(), LuaStatus::Loaded);
        let mut results_a = Vec::new();
        for input in &input_sequence {
            let intent = bridge_a.call_update(dt, input, &actor).unwrap();
            results_a.push((intent.move_x, intent.jump_pressed));
        }

        // Run B (fresh bridge, same script, same inputs)
        let bridge_b = LuaBridge::new(path.clone());
        assert_eq!(bridge_b.status(), LuaStatus::Loaded);
        let mut results_b = Vec::new();
        for input in &input_sequence {
            let intent = bridge_b.call_update(dt, input, &actor).unwrap();
            results_b.push((intent.move_x, intent.jump_pressed));
        }

        // Verify identical outputs
        assert_eq!(results_a.len(), results_b.len());
        for (i, (a, b)) in results_a.iter().zip(results_b.iter()).enumerate() {
            assert!(
                (a.0 - b.0).abs() < f32::EPSILON && a.1 == b.1,
                "Determinism failure at step {}: run_a=({}, {}), run_b=({}, {})",
                i,
                a.0,
                a.1,
                b.0,
                b.1
            );
        }

        // Also verify expected values for sanity
        assert_eq!(results_a[0], (0.0, false), "step 0: idle");
        assert_eq!(results_a[2], (1.0, false), "step 2: moving right");
        assert_eq!(results_a[5], (1.0, true), "step 5: jump while moving right");
        assert_eq!(results_a[8], (0.0, false), "step 8: stopped");
        assert_eq!(results_a[10], (-1.0, false), "step 10: moving left");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn lua_play_animation_returns_intent() {
        let path = temp_lua_path("play_anim");
        write_temp_script(
            &path,
            r#"
function on_update(dt)
    engine.actor.set_intent(0.0, false)
    engine.actor.play_animation("run")
end
"#,
        );

        let bridge = LuaBridge::new(path.clone());
        assert_eq!(bridge.status(), LuaStatus::Loaded);

        let intent = bridge
            .call_update(1.0 / 60.0, &make_input(), &make_actor())
            .expect("should return intent");
        assert_eq!(intent.play_animation.as_deref(), Some("run"));
        assert!(!intent.stop_animation);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn lua_stop_animation_returns_intent() {
        let path = temp_lua_path("stop_anim");
        write_temp_script(
            &path,
            r#"
function on_update(dt)
    engine.actor.set_intent(0.0, false)
    engine.actor.stop_animation()
end
"#,
        );

        let bridge = LuaBridge::new(path.clone());
        assert_eq!(bridge.status(), LuaStatus::Loaded);

        let intent = bridge
            .call_update(1.0 / 60.0, &make_input(), &make_actor())
            .expect("should return intent");
        assert!(intent.stop_animation);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn lua_reads_animation_state() {
        let path = temp_lua_path("read_anim_state");
        write_temp_script(
            &path,
            r#"
function on_update(dt)
    local anim = engine.actor.current_animation
    local finished = engine.actor.animation_finished
    -- Use animation state to drive movement
    if anim == "idle" and not finished then
        engine.actor.set_intent(0.0, false)
    else
        engine.actor.set_intent(1.0, false)
    end
end
"#,
        );

        let bridge = LuaBridge::new(path.clone());
        assert_eq!(bridge.status(), LuaStatus::Loaded);

        // With "idle" animation, should get move_x = 0
        let mut actor = make_actor();
        actor.current_animation = Some("idle".to_string());
        actor.animation_finished = false;
        let intent = bridge
            .call_update(1.0 / 60.0, &make_input(), &actor)
            .expect("should return intent");
        assert_eq!(intent.move_x, 0.0);

        // With no animation, should get move_x = 1
        let actor2 = make_actor(); // current_animation = None
        let intent2 = bridge
            .call_update(1.0 / 60.0, &make_input(), &actor2)
            .expect("should return intent");
        assert_eq!(intent2.move_x, 1.0);

        let _ = std::fs::remove_file(&path);
    }
}
