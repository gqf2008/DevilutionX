/// Level Manager - Orchestrates level generation and loading
///
/// This module provides a high-level interface for managing game levels,
/// including procedural generation (Cathedral, Catacombs, etc.) and
/// loading special quest maps.
///
/// C++ equivalent: Multiple files in Source/levels/ coordinated by diablo.cpp
use crate::levels::drlg_l1::CathedralGenerator;
use crate::levels::gendung::Dungeon;
use crate::levels::setmaps::{SetLevel, SetMapManager};
use crate::levels::themes::ThemeManager;
use crate::levels::town::TownGenerator;
use crate::levels::trigs::TriggerManager;
use crate::levels::types::DungeonType;

/// Maximum level number in the game
pub const MAX_LEVEL: u8 = 24;

/// Level type categorization
///
/// Groups levels by their visual theme and generation algorithm.
///
/// # C++ Equivalent
/// Multiple `_dungeon_type` checks scattered across the codebase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelType {
    /// Town (Level 0)
    Town = 0,
    /// Cathedral (Levels 1-4)
    Cathedral = 1,
    /// Catacombs (Levels 5-8)
    Catacombs = 2,
    /// Caves (Levels 9-12)
    Caves = 3,
    /// Hell (Levels 13-16)
    Hell = 4,
    /// Special quest levels (Skeleton King, Lazarus, etc.)
    SetLevel = 5,
}

impl LevelType {
    /// Get the level type for a given level number
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => LevelType::Town,
            1..=4 => LevelType::Cathedral,
            5..=8 => LevelType::Catacombs,
            9..=12 => LevelType::Caves,
            13..=16 => LevelType::Hell,
            _ => LevelType::Town, // Quest levels handled separately
        }
    }

    /// Get the corresponding DungeonType
    pub fn to_dungeon_type(&self) -> DungeonType {
        match self {
            LevelType::Town => DungeonType::Town,
            LevelType::Cathedral => DungeonType::Cathedral,
            LevelType::Catacombs => DungeonType::Catacombs,
            LevelType::Caves => DungeonType::Caves,
            LevelType::Hell => DungeonType::Hell,
            LevelType::SetLevel => DungeonType::Cathedral, // Default for quest levels
        }
    }
}

/// Level entry point (how player enters the level)
///
/// Determines where the player spawns when entering a level.
///
/// # C++ Equivalent
/// `entry` parameter in `CreateLevel()` and related functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelEntry {
    /// Main entry (from town or down stairs)
    Main = 0,
    /// Coming from stairs up
    StairsUp = 1,
    /// Coming from stairs down
    StairsDown = 2,
    /// Coming from portal
    Portal = 3,
    /// Special warp points
    Warp = 4,
}

/// Level Manager
///
/// Coordinates all level generation and loading operations.
/// This is the main entry point for creating game levels.
///
/// # C++ Equivalent
/// Functionality scattered across:
/// - `CreateLevel()` in levels/gendung.cpp
/// - Level generation functions in diablo.cpp
/// - Individual DRLG_* functions
pub struct LevelManager {
    /// Current level number (0 = Town, 1-16 = Dungeon)
    current_level: u8,

    /// Current set level (if in a quest level)
    current_set_level: SetLevel,

    /// The dungeon data structure
    dungeon: Dungeon,

    /// Cathedral generator (L1-L4)
    cathedral_generator: Option<CathedralGenerator>,

    /// Town generator
    town_generator: Option<TownGenerator>,

    /// Trigger manager
    trigger_manager: TriggerManager,

    /// Theme manager
    theme_manager: Option<ThemeManager>,

    /// Set map manager (quest levels)
    set_map_manager: SetMapManager,
}

impl LevelManager {
    /// Create a new level manager
    pub fn new() -> Self {
        LevelManager {
            current_level: 0,
            current_set_level: SetLevel::None,
            dungeon: Dungeon::new(),
            cathedral_generator: None,
            town_generator: None,
            trigger_manager: TriggerManager::new(),
            theme_manager: None,
            set_map_manager: SetMapManager::new(),
        }
    }

    /// Get current level number
    pub fn current_level(&self) -> u8 {
        self.current_level
    }

    /// Get current set level
    pub fn current_set_level(&self) -> SetLevel {
        self.current_set_level
    }

    /// Check if a level is the town
    pub fn is_town(level: u8) -> bool {
        level == 0
    }

    /// Get the level type for a level number
    pub fn get_level_type(level: u8) -> LevelType {
        LevelType::from_level(level)
    }

    /// Get reference to the dungeon
    pub fn dungeon(&self) -> &Dungeon {
        &self.dungeon
    }

    /// Get mutable reference to the dungeon
    pub fn dungeon_mut(&mut self) -> &mut Dungeon {
        &mut self.dungeon
    }

    /// Get reference to the trigger manager
    pub fn trigger_manager(&self) -> &TriggerManager {
        &self.trigger_manager
    }

    /// Get mutable reference to the trigger manager
    pub fn trigger_manager_mut(&mut self) -> &mut TriggerManager {
        &mut self.trigger_manager
    }

    /// Generate a level (procedural generation)
    ///
    /// # Arguments
    /// * `level` - Level number to generate (0 = Town, 1-16 = Dungeon)
    /// * `entry` - Entry point for the player
    /// * `seed` - Random seed for deterministic generation
    ///
    /// # C++ Equivalent
    /// `CreateLevel()` in gendung.cpp + specific DRLG functions
    ///
    /// # Deferred Work
    /// Full implementation requires:
    /// - Random seed management
    /// - Level-specific generation dispatch
    /// - Monster/item placement
    /// - Trigger initialization
    pub fn generate_level(&mut self, level: u8, entry: LevelEntry, _seed: u32) {
        self.current_level = level;
        self.current_set_level = SetLevel::None;

        // Reset dungeon
        self.dungeon = Dungeon::new();
        self.trigger_manager = TriggerManager::new();

        match LevelType::from_level(level) {
            LevelType::Town => {
                // TODO: Generate town
                // self.town_generator = Some(TownGenerator::new());
                // self.town_generator.as_mut().unwrap().create_town(&mut self.dungeon, entry);
                // self.trigger_manager.init_town_triggers(...);
            }
            LevelType::Cathedral => {
                // TODO: Generate cathedral (L1)
                // self.cathedral_generator = Some(CathedralGenerator::new());
                // self.cathedral_generator.as_mut().unwrap().generate(&mut self.dungeon, level, seed);
                // self.trigger_manager.init_l1_triggers(...);
                // self.theme_manager = Some(ThemeManager::new());
                // self.theme_manager.as_mut().unwrap().init_themes(...);
            }
            LevelType::Catacombs => {
                // TODO: Generate catacombs (L2) - M7+
            }
            LevelType::Caves => {
                // TODO: Generate caves (L3) - M7+
            }
            LevelType::Hell => {
                // TODO: Generate hell (L4) - M7+
            }
            LevelType::SetLevel => {
                // Quest levels handled by load_set_level
            }
        }

        let _ = entry; // Suppress unused warning
    }

    /// Load a set level (quest level)
    ///
    /// # Arguments
    /// * `set_level` - The set level to load
    ///
    /// # C++ Equivalent
    /// `LoadSetMap()` in setmaps.cpp
    ///
    /// # Deferred Work
    /// Full implementation requires:
    /// - File I/O for .dun maps
    /// - Object initialization
    /// - Trigger setup
    /// - Quest state updates
    pub fn load_set_level(&mut self, set_level: SetLevel) {
        self.current_set_level = set_level;
        self.current_level = 0; // Set levels don't have standard level numbers

        // Reset dungeon
        self.dungeon = Dungeon::new();
        self.trigger_manager = TriggerManager::new();

        // TODO: Load set map
        // self.set_map_manager.load_set_map(set_level, &mut self.dungeon);
    }

    /// Reset the level manager
    pub fn reset(&mut self) {
        self.current_level = 0;
        self.current_set_level = SetLevel::None;
        self.dungeon = Dungeon::new();
        self.cathedral_generator = None;
        self.town_generator = None;
        self.trigger_manager = TriggerManager::new();
        self.theme_manager = None;
    }
}

impl Default for LevelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_type_from_level() {
        assert_eq!(LevelType::from_level(0), LevelType::Town);
        assert_eq!(LevelType::from_level(1), LevelType::Cathedral);
        assert_eq!(LevelType::from_level(4), LevelType::Cathedral);
        assert_eq!(LevelType::from_level(5), LevelType::Catacombs);
        assert_eq!(LevelType::from_level(8), LevelType::Catacombs);
        assert_eq!(LevelType::from_level(9), LevelType::Caves);
        assert_eq!(LevelType::from_level(12), LevelType::Caves);
        assert_eq!(LevelType::from_level(13), LevelType::Hell);
        assert_eq!(LevelType::from_level(16), LevelType::Hell);
    }

    #[test]
    fn test_level_type_to_dungeon_type() {
        assert_eq!(LevelType::Town.to_dungeon_type(), DungeonType::Town);
        assert_eq!(
            LevelType::Cathedral.to_dungeon_type(),
            DungeonType::Cathedral
        );
        assert_eq!(
            LevelType::Catacombs.to_dungeon_type(),
            DungeonType::Catacombs
        );
        assert_eq!(LevelType::Caves.to_dungeon_type(), DungeonType::Caves);
        assert_eq!(LevelType::Hell.to_dungeon_type(), DungeonType::Hell);
    }

    #[test]
    fn test_level_manager_new() {
        let manager = LevelManager::new();
        assert_eq!(manager.current_level(), 0);
        assert_eq!(manager.current_set_level(), SetLevel::None);
    }

    #[test]
    fn test_level_manager_is_town() {
        assert!(LevelManager::is_town(0));
        assert!(!LevelManager::is_town(1));
        assert!(!LevelManager::is_town(16));
    }

    #[test]
    fn test_level_manager_get_level_type() {
        assert_eq!(LevelManager::get_level_type(0), LevelType::Town);
        assert_eq!(LevelManager::get_level_type(1), LevelType::Cathedral);
        assert_eq!(LevelManager::get_level_type(5), LevelType::Catacombs);
        assert_eq!(LevelManager::get_level_type(10), LevelType::Caves);
        assert_eq!(LevelManager::get_level_type(15), LevelType::Hell);
    }

    #[test]
    fn test_level_manager_generate_town() {
        let mut manager = LevelManager::new();
        manager.generate_level(0, LevelEntry::Main, 12345);
        assert_eq!(manager.current_level(), 0);
        assert_eq!(manager.current_set_level(), SetLevel::None);
    }

    #[test]
    fn test_level_manager_generate_cathedral() {
        let mut manager = LevelManager::new();
        manager.generate_level(1, LevelEntry::Main, 54321);
        assert_eq!(manager.current_level(), 1);
        assert_eq!(manager.current_set_level(), SetLevel::None);
    }

    #[test]
    fn test_level_manager_load_set_level() {
        let mut manager = LevelManager::new();
        manager.load_set_level(SetLevel::SkelKing);
        assert_eq!(manager.current_set_level(), SetLevel::SkelKing);
    }

    #[test]
    fn test_level_manager_reset() {
        let mut manager = LevelManager::new();
        manager.generate_level(5, LevelEntry::StairsDown, 99999);
        assert_eq!(manager.current_level(), 5);

        manager.reset();
        assert_eq!(manager.current_level(), 0);
        assert_eq!(manager.current_set_level(), SetLevel::None);
    }

    #[test]
    fn test_level_manager_dungeon_access() {
        let manager = LevelManager::new();
        let dungeon = manager.dungeon();
        // Dungeon should be initialized
        assert_eq!(dungeon.get_tile(0, 0), 0);
    }

    #[test]
    fn test_level_entry_variants() {
        assert_eq!(LevelEntry::Main as u8, 0);
        assert_eq!(LevelEntry::StairsUp as u8, 1);
        assert_eq!(LevelEntry::StairsDown as u8, 2);
        assert_eq!(LevelEntry::Portal as u8, 3);
        assert_eq!(LevelEntry::Warp as u8, 4);
    }
}
