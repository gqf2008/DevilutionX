//! Controls Module - Input handling and player controls
//!
//! # M63: Controls System
//!
//! This module handles all input processing for DevilutionX:
//! - Keyboard and mouse input
//! - Game controller/joystick support
//! - Player movement controls
//! - Menu navigation controls
//!
//! ## C++ References
//! - `Source/controls/plrctrls.cpp` (2,260 lines) - Player controls
//! - `Source/controls/controller.cpp` (122 lines) - Controller interface
//! - `Source/controls/game_controls.cpp` - Game action bindings
//! - `Source/controls/menu_controls.cpp` - Menu input handling

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod controller;
pub mod player_controls;
pub mod game_controls;
pub mod menu_controls;
pub mod keymapper;

// Re-export main types
pub use controller::{
    ControllerButton,
    ControllerButtonEvent,
    ControllerButtonCombo,
    AxisDirection,
    Controller,
};

pub use player_controls::{
    PlayerControls,
    GameActionType,
    CursorTarget,
};

pub use game_controls::{
    GameAction,
    GameActionMapping,
};

pub use menu_controls::{
    MenuAction,
    MenuControls,
};

pub use keymapper::{
    KeyAction,
    PadAction,
    default_key_actions,
    default_pad_actions,
    key_id_to_name,
    combo_description,
};
