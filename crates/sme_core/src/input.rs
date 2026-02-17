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
