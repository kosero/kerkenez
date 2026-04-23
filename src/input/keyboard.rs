use std::collections::HashSet;

use winit::{event::ElementState, keyboard::KeyCode};

#[derive(Default)]
pub struct KeyboardState {
    held: HashSet<KeyCode>,
    pressed: HashSet<KeyCode>,
    released: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if !self.held.contains(&key) {
                    self.pressed.insert(key);
                }
                self.held.insert(key);
            }
            ElementState::Released => {
                self.held.remove(&key);
                self.released.insert(key);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn is_down(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn is_released(&self, key: KeyCode) -> bool {
        self.released.contains(&key)
    }
}
