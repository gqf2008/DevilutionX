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

    /// The C++ `Names[6][10]` table (selhero.cpp): random suggested names per
    /// class. Bard reuses the Rogue names (C++ "Bard (uses Rogue names)").
    pub const DEFAULT_NAMES: [&'static [&'static str]; 6] = [
        &["Aidan", "Qarak", "Born", "Cathan", "Halbu", "Lenalas", "Maximus", "Vane", "Myrdgar", "Rothat"],
        &["Moreina", "Akara", "Kashya", "Flavie", "Divo", "Oriana", "Iantha", "Shikha", "Basanti", "Elexa"],
        &["Jazreth", "Drognan", "Armin", "Fauztin", "Jere", "Kazzulk", "Ranslor", "Sarnakyle", "Valthek", "Horazon"],
        &["Akyev", "Dvorak", "Kekegi", "Kharazim", "Mikulov", "Shenlong", "Vedenin", "Vhalit", "Vylnas", "Zhota"],
        &["Moreina", "Akara", "Kashya", "Flavie", "Divo", "Oriana", "Iantha", "Shikha", "Basanti", "Elexa"],
        &["Alaric", "Barloc", "Egtheow", "Guthlaf", "Heorogar", "Hrothgar", "Oslaf", "Qual-Kehk", "Ragnar", "Ulf"],
    ];

    /// Suggested name for a class at the given index (C++ `GetRandomName`),
    /// wrapping `index % 10`.
    pub fn default_name(self, index: usize) -> &'static str {
        let names = Self::DEFAULT_NAMES[self as usize % Self::COUNT];
        names[index % names.len()]
    }

    /// C++ `InitClassList`: whether the class appears in the hero-class
    /// selection list (Monk requires Hellfire; Bard/Barbarian require their
    /// assets or the test flags).
    pub fn is_listed(self, avail: &ClassAvailability) -> bool {
        match self {
            Self::Monk => avail.is_hellfire,
            Self::Bard => avail.has_bard_assets || avail.test_bard,
            Self::Barbarian => avail.has_barbarian_assets || avail.test_barbarian,
            _ => true,
        }
    }

    /// C++ `SelheroClassSelectorSelect`: whether the class can be selected in
    /// the shareware build (Rogue/Sorcerer need the full retail game; Bard
    /// additionally needs its assets).
    pub fn is_selectable(self, avail: &ClassAvailability) -> bool {
        if avail.is_spawn {
            match self {
                Self::Rogue | Self::Sorcerer => return false,
                Self::Bard => return avail.has_bard_assets,
                _ => {}
            }
        }
        true
    }
}

/// C++ class-availability state (gbIsSpawn, gbIsHellfire, asset flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassAvailability {
    pub is_spawn: bool,
    pub is_hellfire: bool,
    pub has_bard_assets: bool,
    pub has_barbarian_assets: bool,
    pub test_bard: bool,
    pub test_barbarian: bool,
}

impl Default for ClassAvailability {
    fn default() -> Self {
        Self {
            is_spawn: false,
            is_hellfire: false,
            has_bard_assets: false,
            has_barbarian_assets: false,
            test_bard: false,
            test_barbarian: false,
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

/// C++ `_mainmenu_selections` (DiabloUI/diabloui.h): the main-menu actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MainMenuAction {
    #[default]
    None = 0,
    SinglePlayer = 1,
    Multiplayer = 2,
    ShowSupport = 3,
    Settings = 4,
    ShowCredits = 5,
    ExitDiablo = 6,
    AttractMode = 7,
}

/// One main-menu option (C++ `mainmenu.cpp` `vecMenuItems`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainMenuOption {
    pub label: &'static str,
    pub action: MainMenuAction,
}

/// The main-menu options in display order, mirroring C++ `UiMainMenu`:
/// Single Player, Multi Player, Settings, Support, Show Credits, Exit.
pub const MAIN_MENU_OPTIONS: &[MainMenuOption] = &[
    MainMenuOption { label: "Single Player", action: MainMenuAction::SinglePlayer },
    MainMenuOption { label: "Multi Player", action: MainMenuAction::Multiplayer },
    MainMenuOption { label: "Settings", action: MainMenuAction::Settings },
    MainMenuOption { label: "Support", action: MainMenuAction::ShowSupport },
    MainMenuOption { label: "Show Credits", action: MainMenuAction::ShowCredits },
    MainMenuOption { label: "Exit Diablo", action: MainMenuAction::ExitDiablo },
];

/// The action for a menu index (bounds-safe), `None` when out of range.
pub fn main_menu_action(index: usize) -> Option<MainMenuAction> {
    MAIN_MENU_OPTIONS.get(index).map(|o| o.action)
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
    #[test]
    fn test_default_names_match_cpp_table() {
        use super::HeroClass;
        // Warrior / Rogue / Sorcerer first names match selhero.cpp.
        assert_eq!(HeroClass::Warrior.default_name(0), "Aidan");
        assert_eq!(HeroClass::Rogue.default_name(0), "Moreina");
        assert_eq!(HeroClass::Sorcerer.default_name(0), "Jazreth");
        assert_eq!(HeroClass::Barbarian.default_name(0), "Alaric");
        // Bard reuses Rogue names (C++ "Bard (uses Rogue names)").
        assert_eq!(HeroClass::Bard.default_name(0), HeroClass::Rogue.default_name(0));
        // Index wraps modulo 10.
        assert_eq!(HeroClass::Warrior.default_name(10), "Aidan");
        assert_eq!(HeroClass::Warrior.default_name(5), "Lenalas");
    }


    #[test]
    fn test_class_listing_rules() {
        use super::{ClassAvailability, HeroClass};
        let base = ClassAvailability::default();
        // Non-Hellfire, no assets: only the three base classes listed.
        for class in [HeroClass::Warrior, HeroClass::Rogue, HeroClass::Sorcerer] {
            assert!(class.is_listed(&base));
        }
        assert!(!HeroClass::Monk.is_listed(&base), "Monk needs Hellfire");
        assert!(!HeroClass::Bard.is_listed(&base), "Bard needs assets");
        assert!(!HeroClass::Barbarian.is_listed(&base), "Barbarian needs assets");

        // Hellfire enables Monk; test flags / assets enable Bard/Barbarian.
        let hellfire = ClassAvailability { is_hellfire: true, ..base };
        assert!(HeroClass::Monk.is_listed(&hellfire));
        let test = ClassAvailability { test_bard: true, test_barbarian: true, ..base };
        assert!(HeroClass::Bard.is_listed(&test));
        assert!(HeroClass::Barbarian.is_listed(&test));
    }

    #[test]
    fn test_class_selectability_in_shareware() {
        use super::{ClassAvailability, HeroClass};
        let spawn = ClassAvailability { is_spawn: true, ..Default::default() };
        // Rogue/Sorcerer unavailable in shareware (full retail message).
        assert!(!HeroClass::Rogue.is_selectable(&spawn));
        assert!(!HeroClass::Sorcerer.is_selectable(&spawn));
        // Warrior is fine.
        assert!(HeroClass::Warrior.is_selectable(&spawn));
        // Bard is selectable only with assets.
        assert!(!HeroClass::Bard.is_selectable(&spawn));
        let spawn_bard = ClassAvailability { is_spawn: true, has_bard_assets: true, ..Default::default() };
        assert!(HeroClass::Bard.is_selectable(&spawn_bard));
        // Outside shareware everything is selectable.
        let full = ClassAvailability::default();
        assert!(HeroClass::Rogue.is_selectable(&full));
        assert!(HeroClass::Barbarian.is_selectable(&full));
    }


    #[test]
    fn test_main_menu_options_match_cpp() {
        use super::{MainMenuAction, MAIN_MENU_OPTIONS, main_menu_action};
        // Display order and actions mirror C++ UiMainMenu.
        assert_eq!(MAIN_MENU_OPTIONS[0].action, MainMenuAction::SinglePlayer);
        assert_eq!(MAIN_MENU_OPTIONS[1].action, MainMenuAction::Multiplayer);
        assert_eq!(MAIN_MENU_OPTIONS[2].action, MainMenuAction::Settings);
        assert_eq!(MAIN_MENU_OPTIONS[3].action, MainMenuAction::ShowSupport);
        assert_eq!(MAIN_MENU_OPTIONS[4].action, MainMenuAction::ShowCredits);
        assert_eq!(MAIN_MENU_OPTIONS[5].action, MainMenuAction::ExitDiablo);
        assert_eq!(main_menu_action(0), Some(MainMenuAction::SinglePlayer));
        assert_eq!(main_menu_action(99), None);
        // Enum values match C++ _mainmenu_selections.
        assert_eq!(MainMenuAction::SinglePlayer as u8, 1);
        assert_eq!(MainMenuAction::ExitDiablo as u8, 6);
    }


}
