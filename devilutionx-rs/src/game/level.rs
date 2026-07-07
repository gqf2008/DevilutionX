//! Dungeon level management - based on DevilutionX Source/levels/gendung.h
//!
//! Handles dungeon tile data, level generation, and level state.

use super::types::{DungeonType, Point, Direction, NUM_LEVELS};
use serde::{Deserialize, Serialize};
use bitflags::bitflags;

/// Dungeon map dimensions (in tiles)
///
/// This is the *correct* C++ value (`Source/levels/gendung_defs.hpp: DMAXX 40`).
/// It cannot be imported from `crate::levels::types` because (a) that module's
/// `DMAXX`/`DMAXY` are still incorrectly 112 (see the bug note there), and
/// (b) this file also compiles inside the `devilutionx` binary crate, whose
/// `src/main.rs` does not declare a `levels` module.
pub const DMAXX: usize = 40;
pub const DMAXY: usize = 40;

/// Full dungeon dimensions (in world units) - 4x map tiles
///
/// NOTE: duplicates `crate::levels::types::MAXDUNX`/`MAXDUNY` (= 112); see the
/// note on `DMAXX` for why it is not consolidated via `use`.
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;

/// Maximum number of themes per level
pub const MAX_THEMES: usize = 50;

/// Maximum tile types
pub const MAX_TILES: usize = 1379;

bitflags! {
    /// Dungeon tile flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct DungeonFlag: u8 {
        const NONE = 0;
        const MISSILE = 1 << 0;
        const VISIBLE = 1 << 1;
        const DEAD_PLAYER = 1 << 2;
        const POPULATED = 1 << 3;
        const MISSILE_FIRE_WALL = 1 << 4;
        const MISSILE_LIGHTNING_WALL = 1 << 5;
        const LIT = 1 << 6;
        const EXPLORED = 1 << 7;
    }
}

/// Special set levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SetLevel {
    #[default]
    None,
    SkeletonKing,
    BoneChamber,
    Maze,
    PoisonWater,
    VileBetrayer,
    ArenaChurch,
    ArenaHell,
    ArenaCircleOfLife,
}

/// A single dungeon tile
#[derive(Debug, Clone, Copy, Default)]
pub struct DungeonTile {
    /// Base tile ID
    pub tile_id: u16,
    /// Piece ID for rendering
    pub piece_id: u16,
    /// Tile flags
    pub flags: DungeonFlag,
    /// Monster index at this tile (-1 if none)
    pub monster: i16,
    /// Item index at this tile (-1 if none)
    pub item: i16,
    /// Object index at this tile (-1 if none)
    pub object: i16,
    /// Player index at this tile (-1 if none)
    pub player: i8,
    /// Light level (0-15)
    pub light: u8,
    /// Dead body/corpse ID
    pub dead: u8,
    /// Transparency value
    pub trans_val: i8,
}

/// Level state and data
#[derive(Debug)]
pub struct Level {
    /// Current dungeon type
    pub dungeon_type: DungeonType,
    /// Current level number (0 = town, 1-16 = dungeon)
    pub level_num: u8,
    /// Is this a set level (quest dungeon)?
    pub is_set_level: bool,
    /// Set level type
    pub set_level: SetLevel,

    /// Dungeon layout tiles (40x40)
    pub dungeon: [[u8; DMAXY]; DMAXX],
    /// Full tile data (112x112)
    pub tiles: [[DungeonTile; MAXDUNY]; MAXDUNX],

    /// Player view position
    pub view_position: Point,

    /// Level seeds for each dungeon level
    pub dungeon_seeds: [u32; NUM_LEVELS],

    /// Minimum valid position
    pub min_position: Point,
    /// Maximum valid position
    pub max_position: Point,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            dungeon_type: DungeonType::None,
            level_num: 0,
            is_set_level: false,
            set_level: SetLevel::None,
            dungeon: [[0; DMAXY]; DMAXX],
            tiles: [[DungeonTile::default(); MAXDUNY]; MAXDUNX],
            view_position: Point::ZERO,
            dungeon_seeds: [0; NUM_LEVELS],
            min_position: Point::ZERO,
            max_position: Point::new(MAXDUNX as i32, MAXDUNY as i32),
        }
    }
}

impl Level {
    /// Create a new level
    pub fn new() -> Self {
        Self::default()
    }

    /// Get tile at position
    pub fn get_tile(&self, pos: Point) -> Option<&DungeonTile> {
        if self.is_valid_position(pos) {
            Some(&self.tiles[pos.x as usize][pos.y as usize])
        } else {
            None
        }
    }

    /// Get mutable tile at position
    pub fn get_tile_mut(&mut self, pos: Point) -> Option<&mut DungeonTile> {
        if self.is_valid_position(pos) {
            Some(&mut self.tiles[pos.x as usize][pos.y as usize])
        } else {
            None
        }
    }

    /// Check if position is valid
    pub fn is_valid_position(&self, pos: Point) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < MAXDUNX as i32
            && pos.y < MAXDUNY as i32
    }

    /// Check if tile is walkable
    pub fn is_walkable(&self, pos: Point) -> bool {
        if let Some(tile) = self.get_tile(pos) {
            // TODO: Check actual tile properties from SOLData
            tile.tile_id != 0 && tile.monster < 0 && tile.object < 0
        } else {
            false
        }
    }

    /// Check if tile is visible
    pub fn is_visible(&self, pos: Point) -> bool {
        if let Some(tile) = self.get_tile(pos) {
            tile.flags.contains(DungeonFlag::VISIBLE)
        } else {
            false
        }
    }

    /// Set tile visibility
    pub fn set_visible(&mut self, pos: Point, visible: bool) {
        if let Some(tile) = self.get_tile_mut(pos) {
            if visible {
                tile.flags |= DungeonFlag::VISIBLE | DungeonFlag::EXPLORED;
            } else {
                tile.flags.remove(DungeonFlag::VISIBLE);
            }
        }
    }

    /// Get dungeon type for a level number
    pub fn get_dungeon_type_for_level(level: u8) -> DungeonType {
        match level {
            0 => DungeonType::Town,
            1..=4 => DungeonType::Cathedral,
            5..=8 => DungeonType::Catacombs,
            9..=12 => DungeonType::Caves,
            13..=16 => DungeonType::Hell,
            17..=20 => DungeonType::Nest,  // Hellfire
            21..=24 => DungeonType::Crypt, // Hellfire
            _ => DungeonType::None,
        }
    }

    /// Initialize level for given dungeon level
    pub fn init_level(&mut self, level_num: u8) {
        self.level_num = level_num;
        self.dungeon_type = Self::get_dungeon_type_for_level(level_num);
        self.is_set_level = false;
        self.set_level = SetLevel::None;

        // Clear tiles
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.tiles[x][y] = DungeonTile::default();
            }
        }

        // Clear dungeon layout
        for x in 0..DMAXX {
            for y in 0..DMAXY {
                self.dungeon[x][y] = 0;
            }
        }

        // Set default view position
        self.view_position = Point::new(MAXDUNX as i32 / 2, MAXDUNY as i32 / 2);
    }
}

/// Tile properties (solid, transparent, etc.)
#[derive(Debug, Clone, Copy, Default)]
pub struct TileProperties {
    pub solid: bool,
    pub block_light: bool,
    pub block_missile: bool,
    pub transparent: bool,
    pub trap: bool,
}
