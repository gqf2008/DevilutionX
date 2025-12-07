/// Level generation types and constants
///
/// Ported from Source/levels/gendung.h and Source/levels/gendung_defs.hpp

use bitflags::bitflags;
use crate::engine::Point;

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of tiles in tile set
pub const MAXTILES: usize = 1379;

/// Maximum number of theme rooms
pub const MAXTHEMES: usize = 50;

/// Maximum dungeon X coordinate (in tile space)
pub const DMAXX: usize = 112;

/// Maximum dungeon Y coordinate (in tile space)
pub const DMAXY: usize = 112;

/// Maximum dungeon X coordinate (in subtile/piece space, 2x of tile space)
pub const MAXDUNX: usize = 112;

/// Maximum dungeon Y coordinate (in subtile/piece space, 2x of tile space)
pub const MAXDUNY: usize = 112;

// ============================================================================
// Dungeon Type
// ============================================================================

/// Specifies the active dungeon type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DungeonType {
    Town = 0,
    Cathedral = 1,    // L1-L4
    Catacombs = 2,    // L5-L8
    Caves = 3,        // L9-L12
    Hell = 4,         // L13-L16
    Crypt = 5,        // Quest levels
    Nest = 6,         // Hellfire expansion
}

impl DungeonType {
    /// Get the dungeon type for a given level number
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => DungeonType::Town,
            1..=4 => DungeonType::Cathedral,
            5..=8 => DungeonType::Catacombs,
            9..=12 => DungeonType::Caves,
            13..=16 => DungeonType::Hell,
            _ => DungeonType::Town,
        }
    }
}

// ============================================================================
// Set Levels (Quest Levels)
// ============================================================================

/// Special quest levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum SetLevel {
    None = 0,
    SkelKing = 1,      // Skeleton King level
    BoneChamber = 2,   // Chamber of Bone
    Maze = 3,          // Maze
    PoisonWater = 4,   // Poisoned Water Supply
    VileBetrayer = 5,  // Archbishop Lazarus

    // Arena levels (Hellfire)
    ArenaChurch = 6,
    ArenaHell = 7,
    ArenaCircleOfLife = 8,
}

impl SetLevel {
    pub fn is_arena(&self) -> bool {
        matches!(self, SetLevel::ArenaChurch | SetLevel::ArenaHell | SetLevel::ArenaCircleOfLife)
    }
}

// ============================================================================
// Dungeon Flags
// ============================================================================

bitflags! {
    /// Flags for dungeon tiles
    ///
    /// C++ source: `enum class DungeonFlag`
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DungeonFlag: u8 {
        const NONE                  = 0;
        const MISSILE               = 1 << 0;  // Tile contains missile
        const VISIBLE               = 1 << 1;  // Tile is visible to player
        const DEAD_PLAYER           = 1 << 2;  // Tile contains dead player corpse
        const POPULATED             = 1 << 3;  // Tile contains set piece/stairs
        const MISSILE_FIREWALL      = 1 << 4;  // Tile contains firewall
        const MISSILE_LIGHTNING_WALL = 1 << 5; // Tile contains lightning wall
        const LIT                   = 1 << 6;  // Tile is lit
        const EXPLORED              = 1 << 7;  // Tile has been explored (automap)
    }
}

impl DungeonFlag {
    pub const SAVED_FLAGS: DungeonFlag = DungeonFlag::POPULATED
        .union(DungeonFlag::LIT)
        .union(DungeonFlag::EXPLORED);

    pub const LOADED_FLAGS: DungeonFlag = DungeonFlag::MISSILE
        .union(DungeonFlag::VISIBLE)
        .union(DungeonFlag::DEAD_PLAYER)
        .union(DungeonFlag::POPULATED)
        .union(DungeonFlag::LIT)
        .union(DungeonFlag::EXPLORED);
}

// ============================================================================
// Difficulty
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Difficulty {
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

// ============================================================================
// Level Entry
// ============================================================================

/// How the player enters a level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LevelEntry {
    MainEntry = 0,   // Town entrance or new game
    StairsUp = 1,    // Coming from level below
    StairsDown = 2,  // Coming from level above
    Portal = 3,      // Through a portal
    TownWarp = 4,    // Town portal
}

// ============================================================================
// Theme Room
// ============================================================================

/// Theme room location and type
#[derive(Debug, Clone)]
pub struct ThemeLocation {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
    pub theme_type: i8,  // Negative for unused slots
}

impl ThemeLocation {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            theme_type: -1,
        }
    }

    pub fn is_active(&self) -> bool {
        self.theme_type >= 0
    }
}

// ============================================================================
// Mega Tile
// ============================================================================

/// A mega tile contains 4 micro tiles (2x2 grid)
/// Used for mapping tile IDs to their micro-tile components
#[derive(Debug, Clone, Copy, Default)]
pub struct MegaTile {
    pub micro1: u16,
    pub micro2: u16,
    pub micro3: u16,
    pub micro4: u16,
}

// ============================================================================
// Tile Properties
// ============================================================================

bitflags! {
    /// Properties of a dungeon tile
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TileProperties: u8 {
        const NONE           = 0;
        const SOLID          = 1 << 0;  // Cannot walk through
        const BLOCK_LIGHT    = 1 << 1;  // Blocks light
        const BLOCK_PLAYER   = 1 << 2;  // Player cannot pass
        const BLOCK_MONSTER  = 1 << 3;  // Monster cannot pass
        const BLOCK_MISSILE  = 1 << 4;  // Missile cannot pass
        const TRAP           = 1 << 5;  // Trap tile
        const TRANSPARENT    = 1 << 6;  // Transparent (for rendering)
    }
}

// ============================================================================
// Micros (Subtile data)
// ============================================================================

/// Micro tile (subtile) data structure
/// Each full tile is composed of 4 micros in a 2x2 grid
#[derive(Debug, Clone, Copy, Default)]
pub struct Micros {
    pub mt: [u16; 16],  // 16 subtiles (4x4 for rendering)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a position is within dungeon bounds
#[inline]
pub fn in_dungeon_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32
}

/// Check if a position is within dungeon bounds (Point version)
#[inline]
pub fn in_dungeon_bounds_point(pos: Point) -> bool {
    in_dungeon_bounds(pos.x, pos.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_type_from_level() {
        assert_eq!(DungeonType::from_level(0), DungeonType::Town);
        assert_eq!(DungeonType::from_level(1), DungeonType::Cathedral);
        assert_eq!(DungeonType::from_level(4), DungeonType::Cathedral);
        assert_eq!(DungeonType::from_level(5), DungeonType::Catacombs);
        assert_eq!(DungeonType::from_level(9), DungeonType::Caves);
        assert_eq!(DungeonType::from_level(13), DungeonType::Hell);
    }

    #[test]
    fn test_set_level_is_arena() {
        assert!(!SetLevel::SkelKing.is_arena());
        assert!(SetLevel::ArenaChurch.is_arena());
        assert!(SetLevel::ArenaHell.is_arena());
    }

    #[test]
    fn test_dungeon_flags() {
        let flags = DungeonFlag::LIT | DungeonFlag::EXPLORED;
        assert!(flags.contains(DungeonFlag::LIT));
        assert!(flags.contains(DungeonFlag::EXPLORED));
        assert!(!flags.contains(DungeonFlag::VISIBLE));
    }

    #[test]
    fn test_in_dungeon_bounds() {
        assert!(in_dungeon_bounds(0, 0));
        assert!(in_dungeon_bounds(50, 50));
        assert!(in_dungeon_bounds(MAXDUNX as i32 - 1, MAXDUNY as i32 - 1));
        assert!(!in_dungeon_bounds(-1, 0));
        assert!(!in_dungeon_bounds(0, -1));
        assert!(!in_dungeon_bounds(MAXDUNX as i32, 0));
        assert!(!in_dungeon_bounds(0, MAXDUNY as i32));
    }

    #[test]
    fn test_theme_location() {
        let theme = ThemeLocation::new();
        assert_eq!(theme.theme_type, -1);
        assert!(!theme.is_active());
    }

    #[test]
    fn test_tile_properties() {
        let props = TileProperties::SOLID | TileProperties::BLOCK_LIGHT;
        assert!(props.contains(TileProperties::SOLID));
        assert!(props.contains(TileProperties::BLOCK_LIGHT));
        assert!(!props.contains(TileProperties::TRANSPARENT));
    }

    #[test]
    fn test_dungeon_constants() {
        assert_eq!(DMAXX, 112);
        assert_eq!(DMAXY, 112);
        assert_eq!(MAXDUNX, 112);
        assert_eq!(MAXDUNY, 112);
        assert_eq!(MAXTILES, 1379);
        assert_eq!(MAXTHEMES, 50);
    }
}
