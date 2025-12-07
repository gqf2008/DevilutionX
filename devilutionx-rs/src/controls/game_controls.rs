//! Game Controls - Action bindings and key mappings
//!
//! # M63: Game Controls
//!
//! This module handles game action bindings:
//! - Keyboard key bindings
//! - Controller button bindings
//! - Action mapping
//!
//! ## C++ References
//! - `Source/controls/game_controls.cpp`
//! - `Source/controls/game_controls.h`
//! - `Source/controls/keymapper.cpp`

#![allow(dead_code)]

use super::controller::{ControllerButton, ControllerButtonCombo};
use super::player_controls::GameActionType;

// =============================================================================
// Keyboard Keys
// =============================================================================

/// Keyboard key codes (SDL-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyCode {
    Unknown = 0,
    
    // Letters
    A = 97, B = 98, C = 99, D = 100, E = 101, F = 102, G = 103, H = 104,
    I = 105, J = 106, K = 107, L = 108, M = 109, N = 110, O = 111, P = 112,
    Q = 113, R = 114, S = 115, T = 116, U = 117, V = 118, W = 119, X = 120,
    Y = 121, Z = 122,
    
    // Numbers
    Num0 = 48, Num1 = 49, Num2 = 50, Num3 = 51, Num4 = 52,
    Num5 = 53, Num6 = 54, Num7 = 55, Num8 = 56, Num9 = 57,
    
    // Function keys
    F1 = 282, F2 = 283, F3 = 284, F4 = 285, F5 = 286, F6 = 287,
    F7 = 288, F8 = 289, F9 = 290, F10 = 291, F11 = 292, F12 = 293,
    
    // Special keys
    Return = 13,
    Escape = 27,
    Backspace = 8,
    Tab = 9,
    Space = 32,
    
    // Arrow keys
    Up = 273, Down = 274, Left = 276, Right = 275,
    
    // Navigation
    Home = 278, End = 279, PageUp = 280, PageDown = 281,
    Insert = 277, Delete = 127,
    
    // Modifiers
    LShift = 304, RShift = 303,
    LCtrl = 306, RCtrl = 305,
    LAlt = 308, RAlt = 307,
    
    // Numpad
    Kp0 = 256, Kp1 = 257, Kp2 = 258, Kp3 = 259, Kp4 = 260,
    Kp5 = 261, Kp6 = 262, Kp7 = 263, Kp8 = 264, Kp9 = 265,
    KpPeriod = 266, KpDivide = 267, KpMultiply = 268,
    KpMinus = 269, KpPlus = 270, KpEnter = 271,
}

impl Default for KeyCode {
    fn default() -> Self {
        Self::Unknown
    }
}

// =============================================================================
// Game Action
// =============================================================================

/// Game action with associated binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameAction {
    /// Action type
    pub action_type: GameActionType,
    /// Spell slot (for belt/spell actions)
    pub param: i32,
}

impl GameAction {
    pub fn new(action_type: GameActionType) -> Self {
        Self {
            action_type,
            param: 0,
        }
    }
    
    pub fn with_param(action_type: GameActionType, param: i32) -> Self {
        Self { action_type, param }
    }
    
    pub fn none() -> Self {
        Self::new(GameActionType::None)
    }
    
    pub fn is_none(&self) -> bool {
        self.action_type == GameActionType::None
    }
}

impl Default for GameAction {
    fn default() -> Self {
        Self::none()
    }
}

// =============================================================================
// Action Mapping
// =============================================================================

/// Key modifier state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyModifiers {
    pub fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }
    
    pub fn shift() -> Self {
        Self {
            shift: true,
            ctrl: false,
            alt: false,
        }
    }
    
    pub fn ctrl() -> Self {
        Self {
            shift: false,
            ctrl: true,
            alt: false,
        }
    }
    
    pub fn is_none(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt
    }
}

/// Keyboard action mapping
#[derive(Debug, Clone, Copy)]
pub struct KeyBinding {
    /// Key code
    pub key: KeyCode,
    /// Required modifiers
    pub modifiers: KeyModifiers,
    /// Action to perform
    pub action: GameAction,
}

impl KeyBinding {
    pub fn new(key: KeyCode, action: GameAction) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::none(),
            action,
        }
    }
    
    pub fn with_shift(key: KeyCode, action: GameAction) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::shift(),
            action,
        }
    }
    
    pub fn with_ctrl(key: KeyCode, action: GameAction) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::ctrl(),
            action,
        }
    }
    
    pub fn matches(&self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        self.key == key && self.modifiers == modifiers
    }
}

/// Controller action mapping
#[derive(Debug, Clone, Copy)]
pub struct ControllerBinding {
    /// Button combo
    pub combo: ControllerButtonCombo,
    /// Action to perform
    pub action: GameAction,
}

impl ControllerBinding {
    pub fn new(button: ControllerButton, action: GameAction) -> Self {
        Self {
            combo: ControllerButtonCombo::new(button),
            action,
        }
    }
    
    pub fn with_modifier(button: ControllerButton, modifier: ControllerButton, action: GameAction) -> Self {
        Self {
            combo: ControllerButtonCombo::with_modifier(button, modifier),
            action,
        }
    }
}

// =============================================================================
// Game Action Mapping
// =============================================================================

/// Game action mapping - binds inputs to actions
pub struct GameActionMapping {
    /// Keyboard bindings
    key_bindings: Vec<KeyBinding>,
    /// Controller bindings
    controller_bindings: Vec<ControllerBinding>,
}

impl GameActionMapping {
    pub fn new() -> Self {
        let mut mapping = Self {
            key_bindings: Vec::new(),
            controller_bindings: Vec::new(),
        };
        mapping.setup_default_bindings();
        mapping
    }
    
    /// Setup default key bindings
    fn setup_default_bindings(&mut self) {
        // Movement keys (handled separately in player controls)
        
        // Toggle screens
        self.key_bindings.push(KeyBinding::new(
            KeyCode::I,
            GameAction::new(GameActionType::ToggleInventory),
        ));
        self.key_bindings.push(KeyBinding::new(
            KeyCode::C,
            GameAction::new(GameActionType::ToggleCharacter),
        ));
        self.key_bindings.push(KeyBinding::new(
            KeyCode::Q,
            GameAction::new(GameActionType::ToggleQuestLog),
        ));
        self.key_bindings.push(KeyBinding::new(
            KeyCode::S,
            GameAction::new(GameActionType::ToggleSpellBook),
        ));
        self.key_bindings.push(KeyBinding::new(
            KeyCode::Tab,
            GameAction::new(GameActionType::ToggleMap),
        ));
        
        // Quick save/load
        self.key_bindings.push(KeyBinding::new(
            KeyCode::F5,
            GameAction::new(GameActionType::QuickSave),
        ));
        self.key_bindings.push(KeyBinding::new(
            KeyCode::F8,
            GameAction::new(GameActionType::QuickLoad),
        ));
        
        // Pause
        self.key_bindings.push(KeyBinding::new(
            KeyCode::P,
            GameAction::new(GameActionType::Pause),
        ));
        self.key_bindings.push(KeyBinding::new(
            KeyCode::Escape,
            GameAction::new(GameActionType::OpenMenu),
        ));
        
        // Belt slots (1-8)
        for i in 0..8 {
            self.key_bindings.push(KeyBinding::new(
                match i {
                    0 => KeyCode::Num1,
                    1 => KeyCode::Num2,
                    2 => KeyCode::Num3,
                    3 => KeyCode::Num4,
                    4 => KeyCode::Num5,
                    5 => KeyCode::Num6,
                    6 => KeyCode::Num7,
                    7 => KeyCode::Num8,
                    _ => KeyCode::Unknown,
                },
                GameAction::with_param(GameActionType::UseBeltItem, i),
            ));
        }
        
        // Spell slots (F1-F8)
        for i in 0..8 {
            self.key_bindings.push(KeyBinding::new(
                match i {
                    0 => KeyCode::F1,
                    1 => KeyCode::F2,
                    2 => KeyCode::F3,
                    3 => KeyCode::F4,
                    4 => KeyCode::F5,
                    5 => KeyCode::F6,
                    6 => KeyCode::F7,
                    7 => KeyCode::F8,
                    _ => KeyCode::Unknown,
                },
                GameAction::with_param(GameActionType::SpeedBook, i),
            ));
        }
        
        // Controller bindings
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::A,
            GameAction::new(GameActionType::PrimaryAction),
        ));
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::B,
            GameAction::new(GameActionType::SecondaryAction),
        ));
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::Y,
            GameAction::new(GameActionType::ToggleInventory),
        ));
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::X,
            GameAction::new(GameActionType::CastSpell),
        ));
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::Start,
            GameAction::new(GameActionType::OpenMenu),
        ));
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::Back,
            GameAction::new(GameActionType::ToggleMap),
        ));
        
        // Shoulder buttons for belt items
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::LeftShoulder,
            GameAction::with_param(GameActionType::UseBeltItem, 0),
        ));
        self.controller_bindings.push(ControllerBinding::new(
            ControllerButton::RightShoulder,
            GameAction::with_param(GameActionType::UseBeltItem, 1),
        ));
    }
    
    /// Get action for keyboard input
    pub fn get_key_action(&self, key: KeyCode, modifiers: KeyModifiers) -> GameAction {
        for binding in &self.key_bindings {
            if binding.matches(key, modifiers) {
                return binding.action;
            }
        }
        GameAction::none()
    }
    
    /// Get action for controller input
    pub fn get_controller_action(&self, button: ControllerButton, modifier_pressed: Option<ControllerButton>) -> GameAction {
        for binding in &self.controller_bindings {
            if binding.combo.button == button {
                // Check modifier
                if binding.combo.has_modifier() {
                    if let Some(pressed) = modifier_pressed {
                        if binding.combo.modifier == pressed {
                            return binding.action;
                        }
                    }
                } else if modifier_pressed.is_none() {
                    return binding.action;
                }
            }
        }
        GameAction::none()
    }
    
    /// Add keyboard binding
    pub fn add_key_binding(&mut self, binding: KeyBinding) {
        self.key_bindings.push(binding);
    }
    
    /// Add controller binding
    pub fn add_controller_binding(&mut self, binding: ControllerBinding) {
        self.controller_bindings.push(binding);
    }
    
    /// Remove keyboard binding
    pub fn remove_key_binding(&mut self, key: KeyCode) {
        self.key_bindings.retain(|b| b.key != key);
    }
    
    /// Get all key bindings for an action
    pub fn get_keys_for_action(&self, action_type: GameActionType) -> Vec<KeyCode> {
        self.key_bindings
            .iter()
            .filter(|b| b.action.action_type == action_type)
            .map(|b| b.key)
            .collect()
    }
}

impl Default for GameActionMapping {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_action() {
        let action = GameAction::new(GameActionType::ToggleInventory);
        assert_eq!(action.action_type, GameActionType::ToggleInventory);
        assert_eq!(action.param, 0);
        
        let belt = GameAction::with_param(GameActionType::UseBeltItem, 3);
        assert_eq!(belt.param, 3);
    }

    #[test]
    fn test_key_modifiers() {
        let none = KeyModifiers::none();
        assert!(none.is_none());
        
        let shift = KeyModifiers::shift();
        assert!(!shift.is_none());
        assert!(shift.shift);
    }

    #[test]
    fn test_key_binding() {
        let binding = KeyBinding::new(KeyCode::I, GameAction::new(GameActionType::ToggleInventory));
        
        assert!(binding.matches(KeyCode::I, KeyModifiers::none()));
        assert!(!binding.matches(KeyCode::I, KeyModifiers::shift()));
        assert!(!binding.matches(KeyCode::C, KeyModifiers::none()));
    }

    #[test]
    fn test_key_binding_with_modifier() {
        let binding = KeyBinding::with_shift(KeyCode::S, GameAction::new(GameActionType::QuickSave));
        
        assert!(binding.matches(KeyCode::S, KeyModifiers::shift()));
        assert!(!binding.matches(KeyCode::S, KeyModifiers::none()));
    }

    #[test]
    fn test_game_action_mapping() {
        let mapping = GameActionMapping::new();
        
        // Check inventory toggle
        let action = mapping.get_key_action(KeyCode::I, KeyModifiers::none());
        assert_eq!(action.action_type, GameActionType::ToggleInventory);
        
        // Check unknown key
        let unknown = mapping.get_key_action(KeyCode::Z, KeyModifiers::none());
        assert!(unknown.is_none());
    }

    #[test]
    fn test_belt_bindings() {
        let mapping = GameActionMapping::new();
        
        // Belt slot 1
        let action = mapping.get_key_action(KeyCode::Num1, KeyModifiers::none());
        assert_eq!(action.action_type, GameActionType::UseBeltItem);
        assert_eq!(action.param, 0);
        
        // Belt slot 5
        let action = mapping.get_key_action(KeyCode::Num5, KeyModifiers::none());
        assert_eq!(action.action_type, GameActionType::UseBeltItem);
        assert_eq!(action.param, 4);
    }

    #[test]
    fn test_controller_binding() {
        let mapping = GameActionMapping::new();
        
        let action = mapping.get_controller_action(ControllerButton::A, None);
        assert_eq!(action.action_type, GameActionType::PrimaryAction);
    }

    #[test]
    fn test_get_keys_for_action() {
        let mapping = GameActionMapping::new();
        
        let keys = mapping.get_keys_for_action(GameActionType::ToggleInventory);
        assert!(keys.contains(&KeyCode::I));
    }
}
