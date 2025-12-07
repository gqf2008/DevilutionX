//! Input System - Keyboard and Mouse Event Handling
//!
//! Provides unified input management for game actions.
//!
//! C++ Reference: Source/control.cpp, Source/engine/events.cpp

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::collections::HashSet;

/// Game-level input actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    // Movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveUpLeft,
    MoveUpRight,
    MoveDownLeft,
    MoveDownRight,

    // Actions
    Attack,
    Interact,
    Inventory,
    CharacterSheet,
    SpellBook,
    Automap,
    QuestLog,

    // UI
    Pause,
    Quit,
    ToggleFullscreen,

    // Mouse
    PrimaryAction,    // Left click
    SecondaryAction,  // Right click
}

/// Point for mouse position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Input state manager
///
/// Tracks keyboard and mouse state, converts to game actions.
/// C++ Reference: control.cpp ProcessInput()
pub struct InputSystem {
    /// Currently pressed keys
    keys_down: HashSet<Keycode>,
    /// Keys pressed this frame
    keys_pressed: HashSet<Keycode>,
    /// Keys released this frame
    keys_released: HashSet<Keycode>,
    /// Mouse position
    mouse_pos: Point,
    /// Previous mouse position
    prev_mouse_pos: Point,
    /// Mouse buttons currently down
    mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons pressed this frame
    mouse_pressed: HashSet<MouseButton>,
    /// Mouse buttons released this frame
    mouse_released: HashSet<MouseButton>,
}

impl InputSystem {
    /// Create new input system
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_pos: Point::default(),
            prev_mouse_pos: Point::default(),
            mouse_buttons: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_released: HashSet::new(),
        }
    }

    /// Begin new frame - clear frame-specific state
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.prev_mouse_pos = self.mouse_pos;
    }

    /// Handle keyboard key down event
    pub fn on_key_down(&mut self, key: Keycode) {
        if self.keys_down.insert(key) {
            self.keys_pressed.insert(key);
        }
    }

    /// Handle keyboard key up event
    pub fn on_key_up(&mut self, key: Keycode) {
        if self.keys_down.remove(&key) {
            self.keys_released.insert(key);
        }
    }

    /// Handle mouse button down event
    pub fn on_mouse_button_down(&mut self, button: MouseButton) {
        if self.mouse_buttons.insert(button) {
            self.mouse_pressed.insert(button);
        }
    }

    /// Handle mouse button up event
    pub fn on_mouse_button_up(&mut self, button: MouseButton) {
        if self.mouse_buttons.remove(&button) {
            self.mouse_released.insert(button);
        }
    }

    /// Handle mouse move event
    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_pos = Point::new(x, y);
    }

    /// Check if key is currently down
    pub fn is_key_down(&self, key: Keycode) -> bool {
        self.keys_down.contains(&key)
    }

    /// Check if key was pressed this frame
    pub fn is_key_pressed(&self, key: Keycode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if key was released this frame
    pub fn is_key_released(&self, key: Keycode) -> bool {
        self.keys_released.contains(&key)
    }

    /// Check if mouse button is down
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Check if mouse button was pressed this frame
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    /// Get current mouse position
    pub fn mouse_pos(&self) -> Point {
        self.mouse_pos
    }

    /// Get mouse delta (movement since last frame)
    pub fn mouse_delta(&self) -> Point {
        Point::new(
            self.mouse_pos.x - self.prev_mouse_pos.x,
            self.mouse_pos.y - self.prev_mouse_pos.y,
        )
    }

    /// Get active game actions this frame
    ///
    /// C++ Reference: control.cpp ProcessInput() key mapping
    pub fn get_actions(&self) -> Vec<GameAction> {
        let mut actions = Vec::new();

        // Movement - WASD and arrow keys
        let mut move_x = 0;
        let mut move_y = 0;

        if self.is_key_down(Keycode::W) || self.is_key_down(Keycode::Up) {
            move_y -= 1;
        }
        if self.is_key_down(Keycode::S) || self.is_key_down(Keycode::Down) {
            move_y += 1;
        }
        if self.is_key_down(Keycode::A) || self.is_key_down(Keycode::Left) {
            move_x -= 1;
        }
        if self.is_key_down(Keycode::D) || self.is_key_down(Keycode::Right) {
            move_x += 1;
        }

        // Convert to diagonal or cardinal directions
        match (move_x, move_y) {
            (-1, -1) => actions.push(GameAction::MoveUpLeft),
            (0, -1) => actions.push(GameAction::MoveUp),
            (1, -1) => actions.push(GameAction::MoveUpRight),
            (-1, 0) => actions.push(GameAction::MoveLeft),
            (1, 0) => actions.push(GameAction::MoveRight),
            (-1, 1) => actions.push(GameAction::MoveDownLeft),
            (0, 1) => actions.push(GameAction::MoveDown),
            (1, 1) => actions.push(GameAction::MoveDownRight),
            _ => {}
        }

        // Action keys
        if self.is_key_pressed(Keycode::Space) {
            actions.push(GameAction::Attack);
        }
        if self.is_key_pressed(Keycode::E) {
            actions.push(GameAction::Interact);
        }
        if self.is_key_pressed(Keycode::I) {
            actions.push(GameAction::Inventory);
        }
        if self.is_key_pressed(Keycode::C) {
            actions.push(GameAction::CharacterSheet);
        }
        if self.is_key_pressed(Keycode::B) {
            actions.push(GameAction::SpellBook);
        }
        if self.is_key_pressed(Keycode::Tab) {
            actions.push(GameAction::Automap);
        }
        if self.is_key_pressed(Keycode::Q) {
            actions.push(GameAction::QuestLog);
        }

        // System keys
        if self.is_key_pressed(Keycode::Escape) {
            actions.push(GameAction::Quit);
        }
        if self.is_key_pressed(Keycode::P) {
            actions.push(GameAction::Pause);
        }
        if self.is_key_pressed(Keycode::F11) {
            actions.push(GameAction::ToggleFullscreen);
        }

        // Mouse actions
        if self.is_mouse_button_pressed(MouseButton::Left) {
            actions.push(GameAction::PrimaryAction);
        }
        if self.is_mouse_button_pressed(MouseButton::Right) {
            actions.push(GameAction::SecondaryAction);
        }

        actions
    }

    /// Get movement direction as normalized vector (-1, 0, 1)
    pub fn get_movement_direction(&self) -> (i32, i32) {
        let mut dx = 0;
        let mut dy = 0;

        if self.is_key_down(Keycode::W) || self.is_key_down(Keycode::Up) {
            dy -= 1;
        }
        if self.is_key_down(Keycode::S) || self.is_key_down(Keycode::Down) {
            dy += 1;
        }
        if self.is_key_down(Keycode::A) || self.is_key_down(Keycode::Left) {
            dx -= 1;
        }
        if self.is_key_down(Keycode::D) || self.is_key_down(Keycode::Right) {
            dx += 1;
        }

        (dx, dy)
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_press_detection() {
        let mut input = InputSystem::new();

        input.begin_frame();
        input.on_key_down(Keycode::W);

        assert!(input.is_key_down(Keycode::W));
        assert!(input.is_key_pressed(Keycode::W));

        input.begin_frame();
        assert!(input.is_key_down(Keycode::W));
        assert!(!input.is_key_pressed(Keycode::W)); // Only first frame
    }

    #[test]
    fn test_movement_actions() {
        let mut input = InputSystem::new();

        input.begin_frame();
        input.on_key_down(Keycode::W);
        input.on_key_down(Keycode::D);

        let actions = input.get_actions();
        assert!(actions.contains(&GameAction::MoveUpRight));
    }

    #[test]
    fn test_mouse_position() {
        let mut input = InputSystem::new();

        input.on_mouse_move(100, 200);
        assert_eq!(input.mouse_pos(), Point::new(100, 200));

        input.begin_frame();
        input.on_mouse_move(150, 250);
        let delta = input.mouse_delta();
        assert_eq!(delta.x, 50);
        assert_eq!(delta.y, 50);
    }
}
