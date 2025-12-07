//! Panels System - Game HUD and UI Panels (M64)
//!
//! This module provides the game's panel system for HUD elements.
//!
//! ## C++ References
//! - Source/panels/mainpanel.cpp: Main game panel
//! - Source/panels/charpanel.cpp: Character info panel
//! - Source/panels/spell_book.cpp: Spell book panel
//! - Source/panels/spell_icons.cpp: Spell icon rendering
//! - Source/panels/spell_list.cpp: Spell list panel
//! - Source/panels/info_box.cpp: Info box panel
//! - Source/panels/ui_panels.hpp: Panel definitions
//!
//! ## Architecture
//! - MainPanel: Bottom game HUD with buttons and orbs
//! - CharPanel: Character statistics display
//! - SpellBook: Spell learning and casting interface
//! - SpellList: Quick spell selection
//! - InfoBox: Contextual information display
//! - PartyPanel: Multiplayer party information

pub mod main_panel;
pub mod char_panel;
pub mod spell_book;
pub mod spell_icons;
pub mod spell_list;
pub mod info_box;

pub use main_panel::*;
pub use char_panel::*;
pub use spell_book::*;
pub use spell_icons::*;
pub use spell_list::*;
pub use info_box::*;
