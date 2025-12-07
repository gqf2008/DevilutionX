//! Controller Interface - Game controller and joystick support
//!
//! # M63: Controller System
//!
//! This module provides abstract controller input handling for:
//! - Keyboard (as controller)
//! - Game controllers (Xbox, PlayStation, etc.)
//! - Joysticks
//!
//! ## C++ References
//! - `Source/controls/controller.cpp` (122 lines)
//! - `Source/controls/controller.h`
//! - `Source/controls/controller_buttons.cpp`
//! - `Source/controls/axis_direction.cpp`

#![allow(dead_code)]

// =============================================================================
// Controller Button Enumeration
// =============================================================================

/// Controller button enumeration
///
/// **C++ Reference**: `Source/controls/controller_buttons.h` - `ControllerButton`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ControllerButton {
    /// No button
    None = 0,
    
    // Face buttons
    /// A button (Xbox) / Cross (PlayStation)
    A = 1,
    /// B button (Xbox) / Circle (PlayStation)
    B = 2,
    /// X button (Xbox) / Square (PlayStation)
    X = 3,
    /// Y button (Xbox) / Triangle (PlayStation)
    Y = 4,
    
    // Shoulder buttons
    /// Left bumper
    LeftShoulder = 5,
    /// Right bumper
    RightShoulder = 6,
    
    // Triggers
    /// Left trigger (as button)
    LeftTrigger = 7,
    /// Right trigger (as button)
    RightTrigger = 8,
    
    // Center buttons
    /// Start button
    Start = 9,
    /// Back/Select button
    Back = 10,
    /// Guide/Home button
    Guide = 11,
    
    // Stick clicks
    /// Left stick press
    LeftStick = 12,
    /// Right stick press
    RightStick = 13,
    
    // D-Pad
    /// D-Pad Up
    DPadUp = 14,
    /// D-Pad Down
    DPadDown = 15,
    /// D-Pad Left
    DPadLeft = 16,
    /// D-Pad Right
    DPadRight = 17,
    
    // Analog axis as buttons
    /// Left stick up
    AxisLeftStickUp = 18,
    /// Left stick down
    AxisLeftStickDown = 19,
    /// Left stick left
    AxisLeftStickLeft = 20,
    /// Left stick right
    AxisLeftStickRight = 21,
    
    /// Right stick up
    AxisRightStickUp = 22,
    /// Right stick down
    AxisRightStickDown = 23,
    /// Right stick left
    AxisRightStickLeft = 24,
    /// Right stick right
    AxisRightStickRight = 25,
    
    /// Left trigger axis (analog)
    AxisTriggerLeft = 26,
    /// Right trigger axis (analog)
    AxisTriggerRight = 27,
}

impl ControllerButton {
    /// Check if this is a directional button
    pub fn is_directional(&self) -> bool {
        matches!(
            self,
            Self::DPadUp
                | Self::DPadDown
                | Self::DPadLeft
                | Self::DPadRight
                | Self::AxisLeftStickUp
                | Self::AxisLeftStickDown
                | Self::AxisLeftStickLeft
                | Self::AxisLeftStickRight
        )
    }
    
    /// Check if this is a trigger
    pub fn is_trigger(&self) -> bool {
        matches!(
            self,
            Self::LeftTrigger | Self::RightTrigger | Self::AxisTriggerLeft | Self::AxisTriggerRight
        )
    }
    
    /// Get opposite direction (for D-Pad and sticks)
    pub fn opposite(&self) -> Option<Self> {
        match self {
            Self::DPadUp => Some(Self::DPadDown),
            Self::DPadDown => Some(Self::DPadUp),
            Self::DPadLeft => Some(Self::DPadRight),
            Self::DPadRight => Some(Self::DPadLeft),
            Self::AxisLeftStickUp => Some(Self::AxisLeftStickDown),
            Self::AxisLeftStickDown => Some(Self::AxisLeftStickUp),
            Self::AxisLeftStickLeft => Some(Self::AxisLeftStickRight),
            Self::AxisLeftStickRight => Some(Self::AxisLeftStickLeft),
            _ => None,
        }
    }
}

impl Default for ControllerButton {
    fn default() -> Self {
        Self::None
    }
}

// =============================================================================
// Controller Button Event
// =============================================================================

/// Controller button event (press or release)
///
/// **C++ Reference**: `Source/controls/controller.h` - `ControllerButtonEvent`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerButtonEvent {
    /// The button involved
    pub button: ControllerButton,
    /// True if button was released, false if pressed
    pub up: bool,
}

impl ControllerButtonEvent {
    pub fn new(button: ControllerButton, up: bool) -> Self {
        Self { button, up }
    }
    
    pub fn pressed(button: ControllerButton) -> Self {
        Self { button, up: false }
    }
    
    pub fn released(button: ControllerButton) -> Self {
        Self { button, up: true }
    }
    
    pub fn is_pressed(&self) -> bool {
        !self.up
    }
    
    pub fn is_released(&self) -> bool {
        self.up
    }
}

impl Default for ControllerButtonEvent {
    fn default() -> Self {
        Self {
            button: ControllerButton::None,
            up: false,
        }
    }
}

// =============================================================================
// Controller Button Combo
// =============================================================================

/// Button combination (button + optional modifier)
///
/// **C++ Reference**: `Source/controls/controller.h` - `ControllerButtonCombo`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerButtonCombo {
    /// Primary button
    pub button: ControllerButton,
    /// Modifier button (None if no modifier required)
    pub modifier: ControllerButton,
}

impl ControllerButtonCombo {
    pub fn new(button: ControllerButton) -> Self {
        Self {
            button,
            modifier: ControllerButton::None,
        }
    }
    
    pub fn with_modifier(button: ControllerButton, modifier: ControllerButton) -> Self {
        Self { button, modifier }
    }
    
    pub fn has_modifier(&self) -> bool {
        self.modifier != ControllerButton::None
    }
}

// =============================================================================
// Axis Direction
// =============================================================================

/// Axis direction enumeration
///
/// **C++ Reference**: `Source/controls/axis_direction.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AxisDirection {
    pub x: AxisDirectionX,
    pub y: AxisDirectionY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum AxisDirectionX {
    #[default]
    None = 0,
    Left = -1,
    Right = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum AxisDirectionY {
    #[default]
    None = 0,
    Up = -1,
    Down = 1,
}

impl AxisDirection {
    pub const NONE: Self = Self { x: AxisDirectionX::None, y: AxisDirectionY::None };
    
    pub fn new(x: AxisDirectionX, y: AxisDirectionY) -> Self {
        Self { x, y }
    }
    
    pub fn is_none(&self) -> bool {
        self.x == AxisDirectionX::None && self.y == AxisDirectionY::None
    }
    
    pub fn has_horizontal(&self) -> bool {
        self.x != AxisDirectionX::None
    }
    
    pub fn has_vertical(&self) -> bool {
        self.y != AxisDirectionY::None
    }
}

// =============================================================================
// Controller State
// =============================================================================

/// Controller state tracking
pub struct ControllerState {
    /// Currently pressed buttons
    pub buttons: [bool; 28],
    /// Left stick position (-1.0 to 1.0)
    pub left_stick: (f32, f32),
    /// Right stick position (-1.0 to 1.0)
    pub right_stick: (f32, f32),
    /// Left trigger value (0.0 to 1.0)
    pub left_trigger: f32,
    /// Right trigger value (0.0 to 1.0)
    pub right_trigger: f32,
    /// Trigger lock state (for preventing double-fire)
    pub trigger_locked: (bool, bool),
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            buttons: [false; 28],
            left_stick: (0.0, 0.0),
            right_stick: (0.0, 0.0),
            left_trigger: 0.0,
            right_trigger: 0.0,
            trigger_locked: (false, false),
        }
    }
    
    /// Check if button is pressed
    pub fn is_pressed(&self, button: ControllerButton) -> bool {
        let index = button as usize;
        if index < self.buttons.len() {
            self.buttons[index]
        } else {
            false
        }
    }
    
    /// Set button state
    pub fn set_pressed(&mut self, button: ControllerButton, pressed: bool) {
        let index = button as usize;
        if index < self.buttons.len() {
            self.buttons[index] = pressed;
        }
    }
    
    /// Get axis direction from left stick
    pub fn get_left_stick_direction(&self, deadzone: f32) -> AxisDirection {
        let x = if self.left_stick.0 < -deadzone {
            AxisDirectionX::Left
        } else if self.left_stick.0 > deadzone {
            AxisDirectionX::Right
        } else {
            AxisDirectionX::None
        };
        
        let y = if self.left_stick.1 < -deadzone {
            AxisDirectionY::Up
        } else if self.left_stick.1 > deadzone {
            AxisDirectionY::Down
        } else {
            AxisDirectionY::None
        };
        
        AxisDirection::new(x, y)
    }
    
    /// Get axis direction from right stick
    pub fn get_right_stick_direction(&self, deadzone: f32) -> AxisDirection {
        let x = if self.right_stick.0 < -deadzone {
            AxisDirectionX::Left
        } else if self.right_stick.0 > deadzone {
            AxisDirectionX::Right
        } else {
            AxisDirectionX::None
        };
        
        let y = if self.right_stick.1 < -deadzone {
            AxisDirectionY::Up
        } else if self.right_stick.1 > deadzone {
            AxisDirectionY::Down
        } else {
            AxisDirectionY::None
        };
        
        AxisDirection::new(x, y)
    }
    
    /// Unlock trigger state
    pub fn unlock_trigger_state(&mut self) {
        self.trigger_locked = (false, false);
    }
    
    /// Reset all state
    pub fn reset(&mut self) {
        self.buttons = [false; 28];
        self.left_stick = (0.0, 0.0);
        self.right_stick = (0.0, 0.0);
        self.left_trigger = 0.0;
        self.right_trigger = 0.0;
        self.trigger_locked = (false, false);
    }
}

impl Default for ControllerState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Controller Interface
// =============================================================================

/// Abstract controller interface
pub struct Controller {
    /// Controller ID
    pub id: u32,
    /// Controller name
    pub name: String,
    /// Controller state
    pub state: ControllerState,
    /// Deadzone for analog sticks
    pub deadzone: f32,
    /// Connected status
    pub connected: bool,
}

impl Controller {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            state: ControllerState::new(),
            deadzone: 0.25,
            connected: true,
        }
    }
    
    /// Process button event
    pub fn process_button_event(&mut self, event: ControllerButtonEvent) {
        self.state.set_pressed(event.button, !event.up);
    }
    
    /// Set stick position
    pub fn set_left_stick(&mut self, x: f32, y: f32) {
        self.state.left_stick = (x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0));
        
        // Update axis buttons
        let dir = self.state.get_left_stick_direction(self.deadzone);
        self.state.set_pressed(ControllerButton::AxisLeftStickLeft, dir.x == AxisDirectionX::Left);
        self.state.set_pressed(ControllerButton::AxisLeftStickRight, dir.x == AxisDirectionX::Right);
        self.state.set_pressed(ControllerButton::AxisLeftStickUp, dir.y == AxisDirectionY::Up);
        self.state.set_pressed(ControllerButton::AxisLeftStickDown, dir.y == AxisDirectionY::Down);
    }
    
    /// Set right stick position
    pub fn set_right_stick(&mut self, x: f32, y: f32) {
        self.state.right_stick = (x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0));
        
        // Update axis buttons
        let dir = self.state.get_right_stick_direction(self.deadzone);
        self.state.set_pressed(ControllerButton::AxisRightStickLeft, dir.x == AxisDirectionX::Left);
        self.state.set_pressed(ControllerButton::AxisRightStickRight, dir.x == AxisDirectionX::Right);
        self.state.set_pressed(ControllerButton::AxisRightStickUp, dir.y == AxisDirectionY::Up);
        self.state.set_pressed(ControllerButton::AxisRightStickDown, dir.y == AxisDirectionY::Down);
    }
    
    /// Set left trigger value
    pub fn set_left_trigger(&mut self, value: f32) {
        self.state.left_trigger = value.clamp(0.0, 1.0);
        
        // Trigger as button (threshold 0.5)
        let pressed = self.state.left_trigger > 0.5;
        if pressed && !self.state.trigger_locked.0 {
            self.state.set_pressed(ControllerButton::AxisTriggerLeft, true);
            self.state.trigger_locked.0 = true;
        } else if !pressed && self.state.trigger_locked.0 {
            self.state.set_pressed(ControllerButton::AxisTriggerLeft, false);
            self.state.trigger_locked.0 = false;
        }
    }
    
    /// Set right trigger value
    pub fn set_right_trigger(&mut self, value: f32) {
        self.state.right_trigger = value.clamp(0.0, 1.0);
        
        // Trigger as button (threshold 0.5)
        let pressed = self.state.right_trigger > 0.5;
        if pressed && !self.state.trigger_locked.1 {
            self.state.set_pressed(ControllerButton::AxisTriggerRight, true);
            self.state.trigger_locked.1 = true;
        } else if !pressed && self.state.trigger_locked.1 {
            self.state.set_pressed(ControllerButton::AxisTriggerRight, false);
            self.state.trigger_locked.1 = false;
        }
    }
    
    /// Check if button is pressed
    pub fn is_pressed(&self, button: ControllerButton) -> bool {
        self.state.is_pressed(button)
    }
    
    /// Check if button combo is pressed
    pub fn is_combo_pressed(&self, combo: ControllerButtonCombo) -> bool {
        self.is_pressed(combo.button)
            && (combo.modifier == ControllerButton::None || self.is_pressed(combo.modifier))
    }
    
    /// Get movement direction
    pub fn get_move_direction(&self) -> AxisDirection {
        // Prefer D-Pad over stick
        let mut dir = AxisDirection::NONE;
        
        if self.is_pressed(ControllerButton::DPadLeft) {
            dir.x = AxisDirectionX::Left;
        } else if self.is_pressed(ControllerButton::DPadRight) {
            dir.x = AxisDirectionX::Right;
        }
        
        if self.is_pressed(ControllerButton::DPadUp) {
            dir.y = AxisDirectionY::Up;
        } else if self.is_pressed(ControllerButton::DPadDown) {
            dir.y = AxisDirectionY::Down;
        }
        
        // Fall back to left stick if no D-Pad input
        if dir.is_none() {
            dir = self.state.get_left_stick_direction(self.deadzone);
        }
        
        dir
    }
}

// =============================================================================
// Controller Manager
// =============================================================================

/// Controller manager - handles multiple controllers
pub struct ControllerManager {
    /// All connected controllers
    controllers: Vec<Controller>,
    /// Maximum number of controllers
    max_controllers: usize,
}

impl ControllerManager {
    pub fn new(max_controllers: usize) -> Self {
        Self {
            controllers: Vec::new(),
            max_controllers,
        }
    }
    
    /// Add a controller
    pub fn add_controller(&mut self, id: u32, name: &str) -> bool {
        if self.controllers.len() >= self.max_controllers {
            return false;
        }
        
        // Check if already exists
        if self.controllers.iter().any(|c| c.id == id) {
            return false;
        }
        
        self.controllers.push(Controller::new(id, name));
        true
    }
    
    /// Remove a controller
    pub fn remove_controller(&mut self, id: u32) {
        self.controllers.retain(|c| c.id != id);
    }
    
    /// Get controller by ID
    pub fn get_controller(&self, id: u32) -> Option<&Controller> {
        self.controllers.iter().find(|c| c.id == id)
    }
    
    /// Get mutable controller by ID
    pub fn get_controller_mut(&mut self, id: u32) -> Option<&mut Controller> {
        self.controllers.iter_mut().find(|c| c.id == id)
    }
    
    /// Check if any controller has button pressed
    pub fn is_pressed_on_any(&self, button: ControllerButton) -> bool {
        self.controllers.iter().any(|c| c.is_pressed(button))
    }
    
    /// Get first controller
    pub fn first(&self) -> Option<&Controller> {
        self.controllers.first()
    }
    
    /// Get first controller (mutable)
    pub fn first_mut(&mut self) -> Option<&mut Controller> {
        self.controllers.first_mut()
    }
    
    /// Number of connected controllers
    pub fn count(&self) -> usize {
        self.controllers.len()
    }
    
    /// Iterate over controllers
    pub fn iter(&self) -> impl Iterator<Item = &Controller> {
        self.controllers.iter()
    }
}

impl Default for ControllerManager {
    fn default() -> Self {
        Self::new(4)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_button() {
        assert!(ControllerButton::DPadUp.is_directional());
        assert!(ControllerButton::AxisLeftStickLeft.is_directional());
        assert!(!ControllerButton::A.is_directional());
        
        assert!(ControllerButton::LeftTrigger.is_trigger());
        assert!(!ControllerButton::A.is_trigger());
    }

    #[test]
    fn test_controller_button_opposite() {
        assert_eq!(ControllerButton::DPadUp.opposite(), Some(ControllerButton::DPadDown));
        assert_eq!(ControllerButton::DPadLeft.opposite(), Some(ControllerButton::DPadRight));
        assert_eq!(ControllerButton::A.opposite(), None);
    }

    #[test]
    fn test_controller_button_event() {
        let press = ControllerButtonEvent::pressed(ControllerButton::A);
        assert!(press.is_pressed());
        assert!(!press.is_released());
        
        let release = ControllerButtonEvent::released(ControllerButton::A);
        assert!(release.is_released());
        assert!(!release.is_pressed());
    }

    #[test]
    fn test_controller_button_combo() {
        let simple = ControllerButtonCombo::new(ControllerButton::A);
        assert!(!simple.has_modifier());
        
        let combo = ControllerButtonCombo::with_modifier(
            ControllerButton::A,
            ControllerButton::LeftShoulder
        );
        assert!(combo.has_modifier());
    }

    #[test]
    fn test_axis_direction() {
        let none = AxisDirection::NONE;
        assert!(none.is_none());
        
        let left = AxisDirection::new(AxisDirectionX::Left, AxisDirectionY::None);
        assert!(left.has_horizontal());
        assert!(!left.has_vertical());
    }

    #[test]
    fn test_controller_state() {
        let mut state = ControllerState::new();
        
        assert!(!state.is_pressed(ControllerButton::A));
        state.set_pressed(ControllerButton::A, true);
        assert!(state.is_pressed(ControllerButton::A));
    }

    #[test]
    fn test_controller_stick_direction() {
        let mut state = ControllerState::new();
        state.left_stick = (-0.8, 0.0);
        
        let dir = state.get_left_stick_direction(0.25);
        assert_eq!(dir.x, AxisDirectionX::Left);
        assert_eq!(dir.y, AxisDirectionY::None);
    }

    #[test]
    fn test_controller() {
        let mut controller = Controller::new(0, "Test Controller");
        
        controller.process_button_event(ControllerButtonEvent::pressed(ControllerButton::A));
        assert!(controller.is_pressed(ControllerButton::A));
        
        controller.process_button_event(ControllerButtonEvent::released(ControllerButton::A));
        assert!(!controller.is_pressed(ControllerButton::A));
    }

    #[test]
    fn test_controller_stick() {
        let mut controller = Controller::new(0, "Test");
        
        controller.set_left_stick(-1.0, 0.0);
        assert!(controller.is_pressed(ControllerButton::AxisLeftStickLeft));
        assert!(!controller.is_pressed(ControllerButton::AxisLeftStickRight));
    }

    #[test]
    fn test_controller_manager() {
        let mut manager = ControllerManager::new(4);
        
        assert!(manager.add_controller(0, "Controller 1"));
        assert!(manager.add_controller(1, "Controller 2"));
        assert_eq!(manager.count(), 2);
        
        // Can't add duplicate
        assert!(!manager.add_controller(0, "Controller 1 Dup"));
        assert_eq!(manager.count(), 2);
        
        manager.remove_controller(0);
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_controller_manager_any_pressed() {
        let mut manager = ControllerManager::new(4);
        manager.add_controller(0, "Controller 1");
        manager.add_controller(1, "Controller 2");
        
        if let Some(c) = manager.get_controller_mut(1) {
            c.state.set_pressed(ControllerButton::A, true);
        }
        
        assert!(manager.is_pressed_on_any(ControllerButton::A));
        assert!(!manager.is_pressed_on_any(ControllerButton::B));
    }
}
