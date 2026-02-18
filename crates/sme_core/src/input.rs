//! Input state tracking with both edge-triggered and level-triggered queries.
//!
//! - **Level-triggered (held):** `is_held(key)` returns true every frame the key
//!   is physically down. Used for continuous actions like movement.
//!
//! - **Edge-triggered (just_pressed / just_released):** These are true only during
//!   the frame the transition happened. They are cleared by `end_frame()`, which
//!   the main loop calls only after at least one fixed simulation step has consumed
//!   them. This prevents a press from being silently lost on a frame that has zero
//!   simulation steps (when the accumulator hasn't built up enough time).

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Escape,
    Space,
    F3,
    F4,
    F5,
    W,
    A,
    S,
    D,
    R,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseBtn {
    Left,
    Right,
    Middle,
}

pub struct InputState {
    held: HashSet<Key>,
    just_pressed: HashSet<Key>,
    just_released: HashSet<Key>,

    mouse_held: HashSet<MouseBtn>,
    mouse_just_pressed: HashSet<MouseBtn>,
    mouse_just_released: HashSet<MouseBtn>,

    pub mouse_position: (f64, f64),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            held: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            mouse_held: HashSet::new(),
            mouse_just_pressed: HashSet::new(),
            mouse_just_released: HashSet::new(),
            mouse_position: (0.0, 0.0),
        }
    }

    pub fn key_down(&mut self, key: Key) {
        if self.held.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    pub fn key_up(&mut self, key: Key) {
        if self.held.remove(&key) {
            self.just_released.insert(key);
        }
    }

    pub fn mouse_down(&mut self, btn: MouseBtn) {
        if self.mouse_held.insert(btn) {
            self.mouse_just_pressed.insert(btn);
        }
    }

    pub fn mouse_up(&mut self, btn: MouseBtn) {
        if self.mouse_held.remove(&btn) {
            self.mouse_just_released.insert(btn);
        }
    }

    pub fn is_held(&self, key: Key) -> bool {
        self.held.contains(&key)
    }

    pub fn is_just_pressed(&self, key: Key) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn is_just_released(&self, key: Key) -> bool {
        self.just_released.contains(&key)
    }

    pub fn is_mouse_held(&self, btn: MouseBtn) -> bool {
        self.mouse_held.contains(&btn)
    }

    pub fn is_mouse_just_pressed(&self, btn: MouseBtn) -> bool {
        self.mouse_just_pressed.contains(&btn)
    }

    pub fn is_mouse_just_released(&self, btn: MouseBtn) -> bool {
        self.mouse_just_released.contains(&btn)
    }

    pub fn end_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_just_released.clear();
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_down_sets_held_and_just_pressed() {
        let mut input = InputState::new();
        input.key_down(Key::A);
        assert!(input.is_held(Key::A));
        assert!(input.is_just_pressed(Key::A));
    }

    #[test]
    fn test_key_up_clears_held_sets_just_released() {
        let mut input = InputState::new();
        input.key_down(Key::A);
        input.key_up(Key::A);
        assert!(!input.is_held(Key::A));
        assert!(input.is_just_released(Key::A));
    }

    #[test]
    fn test_key_down_repeat_does_not_double_just_pressed() {
        let mut input = InputState::new();
        input.key_down(Key::A);
        assert!(input.is_just_pressed(Key::A));
        // Second key_down for the same key should not alter state since
        // the key is already in held (HashSet::insert returns false).
        input.key_down(Key::A);
        assert!(input.is_held(Key::A));
        assert!(input.is_just_pressed(Key::A));
    }

    #[test]
    fn test_key_up_without_down_is_no_op() {
        let mut input = InputState::new();
        // key_up without a prior key_down: held.remove returns false,
        // so just_released should NOT be set.
        input.key_up(Key::A);
        assert!(!input.is_just_released(Key::A));
        assert!(!input.is_held(Key::A));
    }

    #[test]
    fn test_end_frame_clears_transient_state() {
        let mut input = InputState::new();
        input.key_down(Key::A);
        input.key_down(Key::Space);
        input.end_frame();
        // Transient just_pressed should be cleared.
        assert!(!input.is_just_pressed(Key::A));
        assert!(!input.is_just_pressed(Key::Space));
        // Held state should persist across frames.
        assert!(input.is_held(Key::A));
        assert!(input.is_held(Key::Space));
    }

    #[test]
    fn test_end_frame_clears_just_released() {
        let mut input = InputState::new();
        input.key_down(Key::A);
        input.key_up(Key::A);
        assert!(input.is_just_released(Key::A));
        input.end_frame();
        assert!(!input.is_just_released(Key::A));
    }

    #[test]
    fn test_mouse_down_sets_held_and_just_pressed() {
        let mut input = InputState::new();
        input.mouse_down(MouseBtn::Left);
        assert!(input.is_mouse_held(MouseBtn::Left));
        assert!(input.is_mouse_just_pressed(MouseBtn::Left));
    }

    #[test]
    fn test_mouse_up_sets_just_released() {
        let mut input = InputState::new();
        input.mouse_down(MouseBtn::Left);
        input.mouse_up(MouseBtn::Left);
        assert!(input.is_mouse_just_released(MouseBtn::Left));
        assert!(!input.is_mouse_held(MouseBtn::Left));
    }

    #[test]
    fn test_mouse_end_frame_clears_transients() {
        let mut input = InputState::new();
        input.mouse_down(MouseBtn::Left);
        input.end_frame();
        assert!(!input.is_mouse_just_pressed(MouseBtn::Left));

        input.mouse_up(MouseBtn::Left);
        assert!(input.is_mouse_just_released(MouseBtn::Left));
        input.end_frame();
        assert!(!input.is_mouse_just_released(MouseBtn::Left));
    }

    #[test]
    fn test_mouse_position_tracking() {
        let mut input = InputState::new();
        input.mouse_position = (100.0, 200.0);
        assert!((input.mouse_position.0 - 100.0).abs() < f64::EPSILON);
        assert!((input.mouse_position.1 - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_keys_independent() {
        let mut input = InputState::new();
        input.key_down(Key::A);
        input.key_down(Key::D);
        assert!(input.is_held(Key::A));
        assert!(input.is_held(Key::D));

        input.key_up(Key::A);
        assert!(!input.is_held(Key::A));
        assert!(input.is_just_released(Key::A));
        // D should remain held and unaffected.
        assert!(input.is_held(Key::D));
        assert!(!input.is_just_released(Key::D));
    }

    #[test]
    fn test_default_state_is_empty() {
        let input = InputState::new();
        assert!(!input.is_held(Key::A));
        assert!(!input.is_held(Key::Space));
        assert!(!input.is_just_pressed(Key::A));
        assert!(!input.is_just_released(Key::A));
        assert!(!input.is_mouse_held(MouseBtn::Left));
        assert!(!input.is_mouse_just_pressed(MouseBtn::Left));
        assert!(!input.is_mouse_just_released(MouseBtn::Left));
        assert!((input.mouse_position.0 - 0.0).abs() < f64::EPSILON);
        assert!((input.mouse_position.1 - 0.0).abs() < f64::EPSILON);
    }
}
