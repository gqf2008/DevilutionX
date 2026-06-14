//! DiabloUI System - Rust Implementation
//!
//! This module provides the main UI framework for the game,
//! including menus, dialogs, hero selection, and UI item rendering.
//!
//! ## Architecture
//!
//! The DiabloUI system consists of several interconnected components:
//!
//! - **UI Items**: Base components for building UI (text, buttons, images, lists)
//! - **UI Core**: Main UI loop and event handling
//! - **Hero Selection**: Character creation and selection screens
//! - **Dialogs**: Confirmation, progress, and message dialogs
//! - **Main Menu**: Title screen and main menu navigation
//!
//! ## C++ Alignment
//!
//! This module corresponds to the following C++ files:
//! - `DiabloUI/diabloui.cpp` (1,214 lines)
//! - `DiabloUI/ui_item.h` (460 lines)
//! - `DiabloUI/button.cpp` (62 lines)
//! - `DiabloUI/scrollbar.cpp` (53 lines)
//! - `DiabloUI/dialogs.cpp` (120 lines)
//! - `DiabloUI/mainmenu.cpp` (135 lines)
//!
//! ## Module Structure
//!
//! ```text
//! diabloui/
//! ├── mod.rs          - Module root and exports
//! ├── ui_item.rs      - UI item types and base classes
//! ├── ui_core.rs      - Main UI loop and event handling
//! ├── dialogs.rs      - Dialog boxes (yes/no, ok, progress)
//! ├── hero_select.rs  - Hero creation and selection
//! └── mainmenu.rs     - Main menu implementation
//! ```

pub mod dialogs;
pub mod hero_select;
pub mod mainmenu;
pub mod ui_core;
pub mod ui_item;

// Re-export commonly used types
pub use dialogs::{DialogResult, DialogType, ProgressDialog, SelectDialog};
pub use hero_select::{HeroInfo, HeroSelection, SelHeroResult};
pub use mainmenu::{MainMenu, MainMenuResult, MainMenuSelection};
pub use ui_core::{UiContext, UiEvent, UiEventHandler, UiEventResult, UiKeyCode, UiListState, UiRenderer};
pub use ui_item::{
    UiArtText, UiArtTextButton, UiButton, UiEdit, UiFlags, UiImageClx, UiItem, UiItemBase, UiList,
    UiListItem, UiRect, UiScrollbar, UiText, UiType,
};

/// Focus indicator sizes used for menu item highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArtFocus {
    /// Small focus indicator
    Small = 0,
    /// Medium focus indicator
    Medium = 1,
    /// Big focus indicator
    Big = 2,
}

impl ArtFocus {
    /// Get the index for art focus array
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Default hero stats for each class
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultStats {
    pub strength: u16,
    pub magic: u16,
    pub dexterity: u16,
    pub vitality: u16,
}

impl DefaultStats {
    pub const WARRIOR: Self = Self {
        strength: 30,
        magic: 10,
        dexterity: 20,
        vitality: 25,
    };

    pub const ROGUE: Self = Self {
        strength: 20,
        magic: 15,
        dexterity: 30,
        vitality: 20,
    };

    pub const SORCERER: Self = Self {
        strength: 15,
        magic: 35,
        dexterity: 15,
        vitality: 20,
    };

    /// Get default stats for a hero class
    pub fn for_class(class: HeroClass) -> Self {
        match class {
            HeroClass::Warrior => Self::WARRIOR,
            HeroClass::Rogue => Self::ROGUE,
            HeroClass::Sorcerer => Self::SORCERER,
            HeroClass::Monk => Self {
                strength: 25,
                magic: 15,
                dexterity: 25,
                vitality: 20,
            },
            HeroClass::Bard => Self {
                strength: 20,
                magic: 20,
                dexterity: 25,
                vitality: 20,
            },
            HeroClass::Barbarian => Self {
                strength: 40,
                magic: 0,
                dexterity: 20,
                vitality: 25,
            },
        }
    }
}

/// Hero class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum HeroClass {
    #[default]
    Warrior = 0,
    Rogue = 1,
    Sorcerer = 2,
    Monk = 3,
    Bard = 4,
    Barbarian = 5,
}

impl HeroClass {
    /// Total number of hero classes
    pub const COUNT: usize = 6;

    /// Get class name
    pub fn name(self) -> &'static str {
        match self {
            Self::Warrior => "Warrior",
            Self::Rogue => "Rogue",
            Self::Sorcerer => "Sorcerer",
            Self::Monk => "Monk",
            Self::Bard => "Bard",
            Self::Barbarian => "Barbarian",
        }
    }

    /// Get class from index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Warrior),
            1 => Some(Self::Rogue),
            2 => Some(Self::Sorcerer),
            3 => Some(Self::Monk),
            4 => Some(Self::Bard),
            5 => Some(Self::Barbarian),
            _ => None,
        }
    }
}

/// Game difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Difficulty {
    #[default]
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

impl Difficulty {
    /// Get difficulty name
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Nightmare => "Nightmare",
            Self::Hell => "Hell",
        }
    }

    /// Get difficulty from index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Normal),
            1 => Some(Self::Nightmare),
            2 => Some(Self::Hell),
            _ => None,
        }
    }

    /// Check if difficulty is unlocked based on player level
    pub fn is_unlocked(self, max_level_completed: u8) -> bool {
        match self {
            Self::Normal => true,
            Self::Nightmare => max_level_completed >= 16,
            Self::Hell => max_level_completed >= 32, // Completed nightmare
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_art_focus() {
        assert_eq!(ArtFocus::Small.index(), 0);
        assert_eq!(ArtFocus::Medium.index(), 1);
        assert_eq!(ArtFocus::Big.index(), 2);
    }

    #[test]
    fn test_default_stats() {
        let warrior_stats = DefaultStats::for_class(HeroClass::Warrior);
        assert_eq!(warrior_stats.strength, 30);
        assert_eq!(warrior_stats.magic, 10);

        let sorc_stats = DefaultStats::for_class(HeroClass::Sorcerer);
        assert_eq!(sorc_stats.magic, 35);
    }

    #[test]
    fn test_hero_class() {
        assert_eq!(HeroClass::Warrior.name(), "Warrior");
        assert_eq!(HeroClass::from_index(2), Some(HeroClass::Sorcerer));
        assert_eq!(HeroClass::from_index(10), None);
    }

    #[test]
    fn test_difficulty() {
        assert_eq!(Difficulty::Normal.name(), "Normal");
        assert!(Difficulty::Normal.is_unlocked(0));
        assert!(!Difficulty::Nightmare.is_unlocked(0));
        assert!(Difficulty::Nightmare.is_unlocked(16));
        assert!(!Difficulty::Hell.is_unlocked(16));
        assert!(Difficulty::Hell.is_unlocked(32));
    }
}
