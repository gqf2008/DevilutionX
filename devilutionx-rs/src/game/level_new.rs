//! Exact port of gendung.h and related level generation from DevilutionX
//!
//! This is a 1:1 mapping of the C++ dungeon generation system.

#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ============================================================================
// Constants from gendung_defs.hpp
// ============================================================================

/// Dungeon map width (tile-based) — the active-region dimension.
///
/// This is the *correct* C++ value (`Source/levels/gendung_defs.hpp: DMAXX 40`).
/// It cannot be imported from `crate::levels::types` because (a) that module's
/// `DMAXX`/`DMAXY` are still incorrectly 112 (see the bug note there), and
/// (b) this file also compiles inside the `devilutionx` binary crate, whose
/// `src/main.rs` does not declare a `levels` module.
pub const DMAXX: usize = 40;
/// Dungeon map height (tile-based)
pub const DMAXY: usize = 40;

/// Maximum dungeon X coordinate (world tiles)
///
/// NOTE: duplicates `crate::levels::types::MAXDUNX` (= 112); kept as a formula
/// of `DMAXX` for internal consistency. See `DMAXX` for why no `use`.
pub const MAXDUNX: usize = 16 + DMAXX * 2 + 16;  // 112
/// Maximum dungeon Y coordinate (world tiles)
pub const MAXDUNY: usize = 16 + DMAXY * 2 + 16;  // 112

/// Maximum themes per level
pub const MAXTHEMES: usize = 50;
/// Maximum tiles
pub const MAXTILES: usize = 1379;

/// Tile width in pixels
pub const TILE_WIDTH: i32 = 64;
/// Tile height in pixels
pub const TILE_HEIGHT: i32 = 32;

// ============================================================================
// Dungeon type - exact match of dungeon_type enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(i8)]
pub enum DungeonType {
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,      // Hellfire
    Crypt = 6,     // Hellfire
    #[default]
    None = -1,
}

impl DungeonType {
    /// Get the dungeon type for a given level
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => DungeonType::Town,
            1..=4 => DungeonType::Cathedral,
            5..=8 => DungeonType::Catacombs,
            9..=12 => DungeonType::Caves,
            13..=16 => DungeonType::Hell,
            17..=20 => DungeonType::Nest,   // Hellfire Hive
            21..=24 => DungeonType::Crypt,  // Hellfire Crypt
            _ => DungeonType::None,
        }
    }

    /// Get the tileset name for this dungeon type
    pub fn tileset_name(&self) -> &'static str {
        match self {
            DungeonType::Town => "levels/towndata/town",
            DungeonType::Cathedral => "levels/l1data/l1",
            DungeonType::Catacombs => "levels/l2data/l2",
            DungeonType::Caves => "levels/l3data/l3",
            DungeonType::Hell => "levels/l4data/l4",
            DungeonType::Nest => "nlevels/l6data/l6",
            DungeonType::Crypt => "nlevels/l5data/l5",
            DungeonType::None => "",
        }
    }

    /// Get number of levels in this dungeon type
    pub fn level_count(&self) -> u8 {
        match self {
            DungeonType::Town => 1,
            DungeonType::Cathedral => 4,
            DungeonType::Catacombs => 4,
            DungeonType::Caves => 4,
            DungeonType::Hell => 4,
            DungeonType::Nest => 4,
            DungeonType::Crypt => 4,
            DungeonType::None => 0,
        }
    }
}

// ============================================================================
// Set levels (special quest levels) - exact match of _setlevels enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(i8)]
pub enum SetLevel {
    #[default]
    None = 0,
    SkeletonKing = 1,
    BoneChamber = 2,
    Maze = 3,
    PoisonWater = 4,
    VileBetrayer = 5,
    ArenaChurch = 6,
    ArenaHell = 7,
    ArenaCircleOfLife = 8,
}

impl SetLevel {
    pub fn is_arena(&self) -> bool {
        matches!(
            self,
            SetLevel::ArenaChurch | SetLevel::ArenaHell | SetLevel::ArenaCircleOfLife
        )
    }
}

// ============================================================================
// Level entry type - exact match of lvl_entry enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum LevelEntry {
    #[default]
    Main = 0,
    Prev = 1,
    SetLevel = 2,
    ReturnLevel = 3,
    Load = 4,
    WarpLevel = 5,
    TownWarpDown = 6,
    TownWarpUp = 7,
}

// ============================================================================
// Difficulty - exact match of _difficulty enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum Difficulty {
    #[default]
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

impl Difficulty {
    /// Get monster level bonus for this difficulty
    pub fn monster_level_bonus(&self) -> i32 {
        match self {
            Difficulty::Normal => 0,
            Difficulty::Nightmare => 15,
            Difficulty::Hell => 30,
        }
    }

    /// Get experience multiplier
    pub fn exp_multiplier(&self) -> i32 {
        match self {
            Difficulty::Normal => 1,
            Difficulty::Nightmare => 2,
            Difficulty::Hell => 4,
        }
    }
}

// ============================================================================
// Dungeon flags - exact match of DungeonFlag enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DungeonFlags(pub u8);

impl DungeonFlags {
    pub const NONE: Self = Self(0);
    pub const MISSILE: Self = Self(1 << 0);
    pub const VISIBLE: Self = Self(1 << 1);
    pub const DEAD_PLAYER: Self = Self(1 << 2);
    pub const POPULATED: Self = Self(1 << 3);
    pub const MISSILE_FIRE_WALL: Self = Self(1 << 4);
    pub const MISSILE_LIGHTNING_WALL: Self = Self(1 << 5);
    pub const LIT: Self = Self(1 << 6);
    pub const EXPLORED: Self = Self(1 << 7);

    /// Flags that are saved
    pub const SAVED_FLAGS: Self = Self(Self::POPULATED.0 | Self::LIT.0 | Self::EXPLORED.0);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn set(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    pub fn clear(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }
}

// ============================================================================
// Tile properties - exact match of TileProperties enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TileProperties(pub u8);

impl TileProperties {
    pub const NONE: Self = Self(0);
    pub const SOLID: Self = Self(1 << 0);
    pub const BLOCK_LIGHT: Self = Self(1 << 1);
    pub const BLOCK_MISSILE: Self = Self(1 << 2);
    pub const TRANSPARENT: Self = Self(1 << 3);
    pub const TRANSPARENT_LEFT: Self = Self(1 << 4);
    pub const TRANSPARENT_RIGHT: Self = Self(1 << 5);
    pub const TRAP: Self = Self(1 << 7);

    pub fn is_solid(&self) -> bool {
        self.contains(Self::SOLID)
    }

    pub fn blocks_light(&self) -> bool {
        self.contains(Self::BLOCK_LIGHT)
    }

    pub fn blocks_missile(&self) -> bool {
        self.contains(Self::BLOCK_MISSILE)
    }

    pub fn is_trap(&self) -> bool {
        self.contains(Self::TRAP)
    }

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

// ============================================================================
// Tile type for rendering - exact match of TileType enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum TileType {
    #[default]
    Square = 0,
    TransparentSquare = 1,
    LeftTriangle = 2,
    RightTriangle = 3,
    LeftTrapezoid = 4,
    RightTrapezoid = 5,
}

// ============================================================================
// Theme location struct
// ============================================================================

#[derive(Debug, Clone, Copy, Default)]
pub struct ThemeLocation {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
    pub theme_type: i8,
}

// ============================================================================
// MegaTile (4 micro tiles form one dungeon tile)
// ============================================================================

#[derive(Debug, Clone, Copy, Default)]
pub struct MegaTile {
    pub micro1: u16,
    pub micro2: u16,
    pub micro3: u16,
    pub micro4: u16,
}

// ============================================================================
// Level cell block for rendering
// ============================================================================

#[derive(Debug, Clone, Copy, Default)]
pub struct LevelCelBlock {
    pub data: u16,
}

impl LevelCelBlock {
    pub fn has_value(&self) -> bool {
        self.data != 0
    }

    pub fn tile_type(&self) -> TileType {
        match (self.data & 0x7000) >> 12 {
            0 => TileType::Square,
            1 => TileType::TransparentSquare,
            2 => TileType::LeftTriangle,
            3 => TileType::RightTriangle,
            4 => TileType::LeftTrapezoid,
            5 => TileType::RightTrapezoid,
            _ => TileType::Square,
        }
    }

    pub fn frame(&self) -> u16 {
        self.data & 0xFFF
    }
}

// ============================================================================
// Micro tile array
// ============================================================================

#[derive(Debug, Clone, Copy, Default)]
pub struct Micros {
    pub mt: [LevelCelBlock; 16],
}

// ============================================================================
// Level data structure
// ============================================================================

/// Level/Dungeon state - contains all dungeon generation data
#[derive(Clone)]
pub struct LevelData {
    /// Current dungeon type
    pub level_type: DungeonType,

    /// Current level number (0 = town, 1-16 = main dungeon, 17-24 = Hellfire)
    pub current_level: u8,

    /// Is this a set/quest level
    pub is_set_level: bool,

    /// Set level number
    pub set_level_num: SetLevel,

    /// Set level dungeon type
    pub set_level_type: DungeonType,

    /// Current difficulty
    pub difficulty: Difficulty,

    /// Tile IDs for the dungeon (DMAXX x DMAXY)
    pub dungeon: [[u8; DMAXY]; DMAXX],

    /// Backup of tile IDs
    pub pdungeon: [[u8; DMAXY]; DMAXX],

    /// Piece IDs for world tiles (MAXDUNX x MAXDUNY)
    pub dpiece: [[u16; MAXDUNY]; MAXDUNX],

    /// Dungeon flags per tile
    pub dflags: [[DungeonFlags; MAXDUNY]; MAXDUNX],

    /// Light level per tile (0 = dark, 15 = bright)
    pub dlight: [[u8; MAXDUNY]; MAXDUNX],

    /// Pre-calculated static light
    pub dprelight: [[u8; MAXDUNY]; MAXDUNX],

    /// Transparency value per tile
    pub dtransval: [[i8; MAXDUNY]; MAXDUNX],

    /// Player index per tile (negative = moving)
    pub dplayer: [[i8; MAXDUNY]; MAXDUNX],

    /// Monster index per tile (negative = moving)
    pub dmonster: [[i16; MAXDUNY]; MAXDUNX],

    /// Object index per tile (negative = large object extended area)
    pub dobject: [[i8; MAXDUNY]; MAXDUNX],

    /// Item index per tile
    pub ditem: [[i8; MAXDUNY]; MAXDUNX],

    /// Corpse/dead index per tile
    pub dcorpse: [[i8; MAXDUNY]; MAXDUNX],

    /// Special tileset frame per tile
    pub dspecial: [[i8; MAXDUNY]; MAXDUNX],

    /// Tile properties (solid, block light, etc.)
    pub sol_data: [TileProperties; MAXTILES],

    /// Protected tiles (cannot be overwritten by generator)
    pub protected: [[bool; DMAXY]; DMAXX],

    /// Dungeon mask (which tiles are used)
    pub dungeon_mask: [[bool; DMAXY]; DMAXX],

    /// View position (player's view center)
    pub view_position: (i32, i32),

    /// Minimum valid position
    pub dmin_position: (i32, i32),

    /// Maximum valid position
    pub dmax_position: (i32, i32),

    /// Theme locations
    pub theme_locs: [ThemeLocation; MAXTHEMES],
    pub theme_count: usize,

    /// Set piece room bounds
    pub set_piece_room: (i32, i32, i32, i32),

    /// Set piece bounds
    pub set_piece: (i32, i32, i32, i32),

    /// Transparency index
    pub trans_val: i8,

    /// Transparency list
    pub trans_list: [bool; 256],

    /// Random seed for level generation
    pub level_seed: u32,
}

impl Default for LevelData {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelData {
    pub fn new() -> Self {
        Self {
            level_type: DungeonType::None,
            current_level: 0,
            is_set_level: false,
            set_level_num: SetLevel::None,
            set_level_type: DungeonType::None,
            difficulty: Difficulty::Normal,
            dungeon: [[0; DMAXY]; DMAXX],
            pdungeon: [[0; DMAXY]; DMAXX],
            dpiece: [[0; MAXDUNY]; MAXDUNX],
            dflags: [[DungeonFlags::NONE; MAXDUNY]; MAXDUNX],
            dlight: [[0; MAXDUNY]; MAXDUNX],
            dprelight: [[0; MAXDUNY]; MAXDUNX],
            dtransval: [[0; MAXDUNY]; MAXDUNX],
            dplayer: [[0; MAXDUNY]; MAXDUNX],
            dmonster: [[0; MAXDUNY]; MAXDUNX],
            dobject: [[0; MAXDUNY]; MAXDUNX],
            ditem: [[0; MAXDUNY]; MAXDUNX],
            dcorpse: [[0; MAXDUNY]; MAXDUNX],
            dspecial: [[0; MAXDUNY]; MAXDUNX],
            sol_data: [TileProperties::NONE; MAXTILES],
            protected: [[false; DMAXY]; DMAXX],
            dungeon_mask: [[false; DMAXY]; DMAXX],
            view_position: (0, 0),
            dmin_position: (0, 0),
            dmax_position: (MAXDUNX as i32, MAXDUNY as i32),
            theme_locs: [ThemeLocation::default(); MAXTHEMES],
            theme_count: 0,
            set_piece_room: (0, 0, 0, 0),
            set_piece: (0, 0, 0, 0),
            trans_val: 0,
            trans_list: [false; 256],
            level_seed: 0,
        }
    }

    /// Check if position is within dungeon bounds
    pub fn in_dungeon_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < MAXDUNX as i32 && y >= 0 && y < MAXDUNY as i32
    }

    /// Check if tile is solid
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if !self.in_dungeon_bounds(x, y) {
            return true;
        }
        let piece = self.dpiece[x as usize][y as usize] as usize;
        if piece >= MAXTILES {
            return true;
        }
        self.sol_data[piece].is_solid()
    }

    /// Check if tile blocks missiles
    pub fn blocks_missile(&self, x: i32, y: i32) -> bool {
        if !self.in_dungeon_bounds(x, y) {
            return true;
        }
        let piece = self.dpiece[x as usize][y as usize] as usize;
        if piece >= MAXTILES {
            return true;
        }
        self.sol_data[piece].blocks_missile()
    }

    /// Check if tile is visible/explored
    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        if !self.in_dungeon_bounds(x, y) {
            return false;
        }
        self.dflags[x as usize][y as usize].contains(DungeonFlags::EXPLORED)
    }

    /// Mark tile as explored
    pub fn set_explored(&mut self, x: i32, y: i32) {
        if self.in_dungeon_bounds(x, y) {
            self.dflags[x as usize][y as usize].set(DungeonFlags::EXPLORED);
        }
    }

    /// Get player at position (0 = none)
    pub fn get_player(&self, x: i32, y: i32) -> i8 {
        if !self.in_dungeon_bounds(x, y) {
            return 0;
        }
        self.dplayer[x as usize][y as usize]
    }

    /// Get monster at position (0 = none)
    pub fn get_monster(&self, x: i32, y: i32) -> i16 {
        if !self.in_dungeon_bounds(x, y) {
            return 0;
        }
        self.dmonster[x as usize][y as usize]
    }

    /// Get object at position (0 = none)
    pub fn get_object(&self, x: i32, y: i32) -> i8 {
        if !self.in_dungeon_bounds(x, y) {
            return 0;
        }
        self.dobject[x as usize][y as usize]
    }

    /// Get item at position (0 = none)
    pub fn get_item(&self, x: i32, y: i32) -> i8 {
        if !self.in_dungeon_bounds(x, y) {
            return 0;
        }
        self.ditem[x as usize][y as usize]
    }

    /// Set player position
    pub fn set_player(&mut self, x: i32, y: i32, player_id: i8) {
        if self.in_dungeon_bounds(x, y) {
            self.dplayer[x as usize][y as usize] = player_id;
        }
    }

    /// Set monster position
    pub fn set_monster(&mut self, x: i32, y: i32, monster_id: i16) {
        if self.in_dungeon_bounds(x, y) {
            self.dmonster[x as usize][y as usize] = monster_id;
        }
    }

    /// Set object position
    pub fn set_object(&mut self, x: i32, y: i32, object_id: i8) {
        if self.in_dungeon_bounds(x, y) {
            self.dobject[x as usize][y as usize] = object_id;
        }
    }

    /// Set item position
    pub fn set_item(&mut self, x: i32, y: i32, item_id: i8) {
        if self.in_dungeon_bounds(x, y) {
            self.ditem[x as usize][y as usize] = item_id;
        }
    }

    /// Clear the level data
    pub fn clear(&mut self) {
        self.dungeon = [[0; DMAXY]; DMAXX];
        self.pdungeon = [[0; DMAXY]; DMAXX];
        self.dpiece = [[0; MAXDUNY]; MAXDUNX];
        self.dflags = [[DungeonFlags::NONE; MAXDUNY]; MAXDUNX];
        self.dlight = [[0; MAXDUNY]; MAXDUNX];
        self.dprelight = [[0; MAXDUNY]; MAXDUNX];
        self.dtransval = [[0; MAXDUNY]; MAXDUNX];
        self.dplayer = [[0; MAXDUNY]; MAXDUNX];
        self.dmonster = [[0; MAXDUNY]; MAXDUNX];
        self.dobject = [[0; MAXDUNY]; MAXDUNX];
        self.ditem = [[0; MAXDUNY]; MAXDUNX];
        self.dcorpse = [[0; MAXDUNY]; MAXDUNX];
        self.dspecial = [[0; MAXDUNY]; MAXDUNX];
        self.protected = [[false; DMAXY]; DMAXX];
        self.dungeon_mask = [[false; DMAXY]; DMAXX];
        self.theme_count = 0;
    }

    /// Get light level at position
    pub fn get_light(&self, x: i32, y: i32) -> u8 {
        if !self.in_dungeon_bounds(x, y) {
            return 0;
        }
        self.dlight[x as usize][y as usize]
    }

    /// Set light level at position
    pub fn set_light(&mut self, x: i32, y: i32, light: u8) {
        if self.in_dungeon_bounds(x, y) {
            self.dlight[x as usize][y as usize] = light;
        }
    }
}

// ============================================================================
// Level generation constants per dungeon type
// ============================================================================

/// Level parameters for each dungeon type
#[derive(Debug, Clone, Copy)]
pub struct LevelParams {
    /// Base monster level
    pub monster_level: i32,
    /// Minimum room size
    pub min_room_size: i32,
    /// Maximum room size
    pub max_room_size: i32,
    /// Number of rooms to generate
    pub num_rooms: i32,
    /// Whether to generate themes
    pub has_themes: bool,
}

impl LevelParams {
    pub fn for_type(dtype: DungeonType, level: u8) -> Self {
        match dtype {
            DungeonType::Town => Self {
                monster_level: 0,
                min_room_size: 0,
                max_room_size: 0,
                num_rooms: 0,
                has_themes: false,
            },
            DungeonType::Cathedral => Self {
                monster_level: level as i32,
                min_room_size: 2,
                max_room_size: 10,
                num_rooms: 40,
                has_themes: true,
            },
            DungeonType::Catacombs => Self {
                monster_level: level as i32,
                min_room_size: 2,
                max_room_size: 6,
                num_rooms: 80,
                has_themes: true,
            },
            DungeonType::Caves => Self {
                monster_level: level as i32,
                min_room_size: 3,
                max_room_size: 8,
                num_rooms: 60,
                has_themes: false,
            },
            DungeonType::Hell => Self {
                monster_level: level as i32,
                min_room_size: 4,
                max_room_size: 12,
                num_rooms: 50,
                has_themes: true,
            },
            DungeonType::Nest => Self {
                monster_level: level as i32,
                min_room_size: 3,
                max_room_size: 8,
                num_rooms: 60,
                has_themes: false,
            },
            DungeonType::Crypt => Self {
                monster_level: level as i32,
                min_room_size: 2,
                max_room_size: 10,
                num_rooms: 40,
                has_themes: true,
            },
            DungeonType::None => Self {
                monster_level: 0,
                min_room_size: 0,
                max_room_size: 0,
                num_rooms: 0,
                has_themes: false,
            },
        }
    }
}
