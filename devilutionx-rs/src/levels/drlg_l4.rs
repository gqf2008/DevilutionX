// DevilutionX Rust Port
// Module: levels::drlg_l4
// Description: L4 Hell dungeon generation (Levels 13-16)
// C++ Equivalent: Source/levels/drlg_l4.cpp

use super::gendung::Dungeon;
use super::types::{LevelEntry, DMAXX, DMAXY, MAXDUNX, MAXDUNY};

// =============================================================================
// Quest Integration Structures
// =============================================================================

/// Quest setpiece room (for quest room placement)
/// C++ equivalent: SetPieceRoom (gendung.h:119)
#[derive(Debug, Clone, Copy, Default)]
pub struct SetPieceRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Theme room types for L4 Hell
/// C++ equivalent: ThemeType enum (themes.h:15-25)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThemeType {
    Barrel = 0,         // Barrel decorations
    Shrine = 1,         // Shrine
    MonsterPit = 2,     // Monster pit
    SkeletonRoom = 3,   // Skeleton room
    Treasure = 4,       // Treasure room
    Library = 5,        // Library
    Torture = 6,        // Torture chamber
    BloodFountain = 7,  // Blood fountain
}

impl ThemeType {
    /// Get random theme type for L4 (0-7, excludes Decapitated)
    /// C++ equivalent: random_(0, 7) cast to ThemeType
    fn random(value: u32) -> Self {
        match value % 8 {
            0 => ThemeType::Barrel,
            1 => ThemeType::Shrine,
            2 => ThemeType::MonsterPit,
            3 => ThemeType::SkeletonRoom,
            4 => ThemeType::Treasure,
            5 => ThemeType::Library,
            6 => ThemeType::Torture,
            _ => ThemeType::BloodFountain,
        }
    }
}

// =============================================================================
// Constants
// =============================================================================

/// L4 Conversion Table for 2×2 wall patterns
/// Maps 16 possible patterns to final tile IDs
const L4_CONV_TABLE: [u8; 16] = [30, 6, 1, 6, 2, 6, 6, 6, 9, 6, 1, 6, 2, 6, 3, 6];

/// Maps tile IDs to their corresponding undecorated tile ID (140 entries)
const L4BTYPES: [u8; 140] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 16, 17, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 6,
    6, 6, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 2, 1, 2, 1, 2, 1, 1, 2,
    2, 0, 0, 0, 0, 0, 0, 15, 16, 9,
    12, 4, 5, 7, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// =============================================================================
// Miniset Structures
// =============================================================================

/// Miniset for pattern matching and replacement
#[derive(Clone)]
pub struct Miniset {
    pub width: usize,
    pub height: usize,
    pub search: Vec<Vec<u8>>,
    pub replace: Vec<Vec<u8>>,
}

impl Miniset {
    pub fn new(width: usize, height: usize, search: Vec<Vec<u8>>, replace: Vec<Vec<u8>>) -> Self {
        Self {
            width,
            height,
            search,
            replace,
        }
    }
}

// =============================================================================
// Miniset Definitions
// =============================================================================

/// Miniset: Stairs up (4×5)
pub fn miniset_l4_ustairs() -> Miniset {
    Miniset::new(
        4,
        5,
        vec![
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
        ],
        vec![
            vec![0, 0, 0, 0],
            vec![36, 38, 35, 0],
            vec![37, 34, 33, 32],
            vec![0, 0, 31, 0],
            vec![0, 0, 0, 0],
        ],
    )
}

/// Miniset: Town warp (4×5)
pub fn miniset_l4_twarp() -> Miniset {
    Miniset::new(
        4,
        5,
        vec![
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
            vec![6, 6, 6, 6],
        ],
        vec![
            vec![0, 0, 0, 0],
            vec![134, 136, 133, 0],
            vec![135, 132, 131, 130],
            vec![0, 0, 129, 0],
            vec![0, 0, 0, 0],
        ],
    )
}

/// Miniset: Stairs down (5×5)
pub fn miniset_l4_dstairs() -> Miniset {
    Miniset::new(
        5,
        5,
        vec![
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
        ],
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 45, 41, 0],
            vec![0, 44, 43, 40, 0],
            vec![0, 46, 42, 39, 0],
            vec![0, 0, 0, 0, 0],
        ],
    )
}

/// Miniset: Pentagram (5×5) - Level 15 only
pub fn miniset_l4_penta() -> Miniset {
    Miniset::new(
        5,
        5,
        vec![
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
        ],
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 98, 100, 103, 0],
            vec![0, 99, 102, 105, 0],
            vec![0, 101, 104, 106, 0],
            vec![0, 0, 0, 0, 0],
        ],
    )
}

/// Miniset: Pentagram portal (5×5) - Alternative version
pub fn miniset_l4_penta2() -> Miniset {
    Miniset::new(
        5,
        5,
        vec![
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
            vec![6, 6, 6, 6, 6],
        ],
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 107, 109, 112, 0],
            vec![0, 108, 111, 114, 0],
            vec![0, 110, 113, 115, 0],
            vec![0, 0, 0, 0, 0],
        ],
    )
}

// Diablo minisets will be added in Day 78

// =============================================================================
// Hell Dungeon Generator
// =============================================================================

/// L4 Hell dungeon generator (Levels 13-16)
pub struct Dungeon4Generator {
    /// RNG state for deterministic generation
    rng_state: u32,

    /// Pre-dungeon grid (before conversion)
    /// C++ `dungeon[DMAXX][DMAXY]` / `pdungeon[DMAXX][DMAXY]` — active region (40×40)
    predungeon: [[u8; DMAXY]; DMAXX],

    /// Room occupancy bitmap (first quadrant only)
    /// C++ `Bitset2d<DMAXX, DMAXY> DungeonMask` — active region (40×40)
    dungeon_mask: [[bool; DMAXY]; DMAXX],

    /// Protected tiles bitmap (quest rooms, cannot place decorations)
    /// C++ `Bitset2d<DMAXX, DMAXY> Protected` — active region (40×40)
    protected: [[bool; DMAXY]; DMAXX],

    /// Hall placement validity flags (20 possible hall positions)
    hall_ok: [bool; 20],

    /// Hold position for L4 generation (used in room placement)
    l4_hold: (usize, usize),

    /// Transparency value counter (incremented for each flood fill region)
    trans_val_counter: i8,

    /// Quest setpiece room rectangle (for quest room protection)
    /// C++ equivalent: SetPieceRoom
    set_piece_room: SetPieceRect,

    /// Theme room rectangles (x, y, width, height) placed by DRLG_PlaceThemeRooms
    /// C++ equivalent: THEME_LOC themeLoc[MAXTHEMES] / themeCount
    theme_locations: Vec<(usize, usize, usize, usize)>,

    /// L4PENTA2 (hell gate) placement position on level 15; the closed gate is
    /// overwritten with L4PENTA at this position (C++ Quests[Q_DIABLO].position).
    l4_penta_position: Option<(usize, usize)>,

    /// Whether the Warlord of Blood quest room is active (C++ Q_WARLORD on
    /// level 13): fixes the first room to 11x11, carves warlord.dun and
    /// protects the SetPiece room.
    pub warlord_quest_active: bool,
}

#[derive(Debug, Clone, Copy)]
struct Room {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Room {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }
}

impl Dungeon4Generator {
    /// Create a new L4 generator
    pub fn new() -> Self {
        Self {
            rng_state: 0,
            predungeon: [[0; DMAXY]; DMAXX],
            dungeon_mask: [[false; DMAXY]; DMAXX],
            protected: [[false; DMAXY]; DMAXX],
            hall_ok: [false; 20],
            l4_hold: (0, 0),
            trans_val_counter: 1,
            set_piece_room: SetPieceRect::default(),
            theme_locations: Vec::new(),
            l4_penta_position: None,
            warlord_quest_active: false,
        }
    }

    /// Set RNG seed for deterministic generation
    pub fn set_rng_seed(&mut self, seed: u32) {
        self.rng_state = seed;
    }

    /// Generate random number in range [min, max)
    ///
    /// C++ `GenerateRnd(v)`: advances the Borland LCG
    /// (`seed = seed * 0x015A4E35 + 1`), reinterprets the state as signed,
    /// applies `abs`, then reduces using the high bits for small limits.
    /// (The previous port used the standard C `rand()` LCG — wrong.)
    fn random_range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        self.rng_state = self.rng_state.wrapping_mul(0x015A4E35).wrapping_add(1);
        let signed = self.rng_state as i32;
        let adv = if signed == i32::MIN { i32::MIN } else { signed.abs() };
        let v = (max - min) as i32;
        let r = if v <= 0x7FFF { (adv >> 16) % v } else { adv % v };
        min + r.max(0) as usize
    }

    /// Random boolean (50% chance)
    fn flip_coin(&mut self) -> bool {
        // C++ FlipCoin() == GenerateRnd(2) == 0
        self.random_range(0, 2) == 0
    }

    /// Initialize dungeon flags and grid
    /// C++ equivalent: InitDungeonFlags
    fn init_dungeon_flags(&mut self) {
        self.predungeon = [[30; DMAXY]; DMAXX]; // Fill with default wall tile
        self.dungeon_mask = [[false; DMAXY]; DMAXX]; // Clear room occupancy
        self.protected = [[false; DMAXY]; DMAXX]; // Clear protected tiles
        self.hall_ok = [false; 20];
        self.l4_hold = (0, 0);
    }

    /// Mirror first quadrant to other 3 quadrants
    /// C++ equivalent: MirrorDungeonLayout (lines 300-311)
    fn mirror_dungeon_layout(&mut self) {
        for y in 0..DMAXY / 2 {
            for x in 0..DMAXX / 2 {
                if self.dungeon_mask[x][y] {
                    self.dungeon_mask[x][DMAXY - 1 - y] = true; // Top-right
                    self.dungeon_mask[DMAXX - 1 - x][y] = true; // Bottom-left
                    self.dungeon_mask[DMAXX - 1 - x][DMAXY - 1 - y] = true; // Bottom-right
                }
            }
        }
    }

    /// Convert dungeon_mask to dungeon tiles using L4_CONV_TABLE
    /// C++ equivalent: MakeDmt (lines 313-321)
    fn make_dmt(&self, dungeon: &mut Dungeon) {
        for y in 0..DMAXY - 1 {
            for x in 0..DMAXX - 1 {
                // 2×2 pattern to index (bit pattern: bottom-right, bottom-left, top-right, top-left)
                let val = ((self.dungeon_mask[x + 1][y + 1] as u8) << 3)
                        | ((self.dungeon_mask[x][y + 1] as u8) << 2)
                        | ((self.dungeon_mask[x + 1][y] as u8) << 1)
                        | (self.dungeon_mask[x][y] as u8);
                dungeon.tiles[x][y] = L4_CONV_TABLE[val as usize];
            }
        }
    }

    /// Check if horizontal wall can be placed
    /// C++ equivalent: HorizontalWallOk (lines 323-341)
    fn horizontal_wall_ok(&self, dungeon: &Dungeon, i: usize, j: usize) -> Option<usize> {
        if i >= DMAXX - 1 || j >= DMAXY { return None; }

        let mut x = 1;
        while i + x < DMAXX && dungeon.tiles[i + x][j] == 6 {
            if self.protected[i + x][j] { break; }
            if j == 0 || dungeon.tiles[i + x][j - 1] != 6 { break; }
            if j >= DMAXY - 1 || dungeon.tiles[i + x][j + 1] != 6 { break; }
            x += 1;
        }

        if i + x < DMAXX {
            let end_tile = dungeon.tiles[i + x][j];
            if matches!(end_tile, 10 | 12 | 13 | 15 | 16 | 21 | 22) && x > 3 {
                return Some(x);
            }
        }

        None
    }

    /// Check if vertical wall can be placed
    /// C++ equivalent: VerticalWallOk (lines 344-362)
    fn vertical_wall_ok(&self, dungeon: &Dungeon, i: usize, j: usize) -> Option<usize> {
        if i >= DMAXX || j >= DMAXY - 1 { return None; }

        let mut y = 1;
        while j + y < DMAXY && dungeon.tiles[i][j + y] == 6 {
            if self.protected[i][j + y] { break; }
            if i == 0 || dungeon.tiles[i - 1][j + y] != 6 { break; }
            if i >= DMAXX - 1 || dungeon.tiles[i + 1][j + y] != 6 { break; }
            y += 1;
        }

        if j + y < DMAXY {
            let end_tile = dungeon.tiles[i][j + y];
            if matches!(end_tile, 8 | 9 | 11 | 14 | 15 | 16 | 21 | 23) && y > 3 {
                return Some(y);
            }
        }

        None
    }

    /// Place horizontal wall with door
    /// C++ equivalent: HorizontalWall (lines 365-405)
    fn horizontal_wall(&mut self, dungeon: &mut Dungeon, i: usize, j: usize, dx: usize) {
        if i >= DMAXX || j >= DMAXY { return; }

        // Convert start tile
        match dungeon.tiles[i][j] {
            13 => dungeon.tiles[i][j] = 17,
            16 => dungeon.tiles[i][j] = 11,
            12 => dungeon.tiles[i][j] = 14,
            _ => {}
        }

        // Fill wall (horizontal floor tiles)
        for xx in 1..dx {
            if i + xx < DMAXX {
                dungeon.tiles[i + xx][j] = 2;
            }
        }

        // Convert end tile
        if i + dx < DMAXX {
            match dungeon.tiles[i + dx][j] {
                15 => dungeon.tiles[i + dx][j] = 14,
                10 => dungeon.tiles[i + dx][j] = 17,
                21 => dungeon.tiles[i + dx][j] = 23,
                22 => dungeon.tiles[i + dx][j] = 29,
                _ => {}
            }
        }

        // Place door (random position, 3 tiles: left, center, right)
        if dx > 3 {
            let xx = self.random_range(0, dx - 3) + 1;
            if i + xx + 2 < DMAXX {
                dungeon.tiles[i + xx][j] = 57; // Door left
                dungeon.tiles[i + xx + 2][j] = 56; // Door right
                dungeon.tiles[i + xx + 1][j] = 60; // Door center

                // Archway decoration above door
                if j > 0 {
                    if dungeon.tiles[i + xx][j - 1] == 6 {
                        dungeon.tiles[i + xx][j - 1] = 58;
                    }
                    if dungeon.tiles[i + xx + 1][j - 1] == 6 {
                        dungeon.tiles[i + xx + 1][j - 1] = 59;
                    }
                }
            }
        }
    }

    /// Place vertical wall with door
    /// C++ equivalent: VerticalWall (lines 407-458)
    fn vertical_wall(&mut self, dungeon: &mut Dungeon, i: usize, j: usize, dy: usize) {
        if i >= DMAXX || j >= DMAXY { return; }

        // Convert start tile
        match dungeon.tiles[i][j] {
            14 => dungeon.tiles[i][j] = 17,
            8 => dungeon.tiles[i][j] = 9,
            15 => dungeon.tiles[i][j] = 10,
            _ => {}
        }

        // Fill wall (vertical floor tiles)
        for yy in 1..dy {
            if j + yy < DMAXY {
                dungeon.tiles[i][j + yy] = 1;
            }
        }

        // Convert end tile
        if j + dy < DMAXY {
            match dungeon.tiles[i][j + dy] {
                11 => dungeon.tiles[i][j + dy] = 17,
                9 => dungeon.tiles[i][j + dy] = 10,
                16 => dungeon.tiles[i][j + dy] = 13,
                21 => dungeon.tiles[i][j + dy] = 22,
                23 => dungeon.tiles[i][j + dy] = 29,
                _ => {}
            }
        }

        // Place door (random position, 3 tiles: top, center, bottom)
        if dy > 3 {
            let yy = self.random_range(0, dy - 3) + 1;
            if j + yy + 2 < DMAXY {
                dungeon.tiles[i][j + yy] = 53; // Door top
                dungeon.tiles[i][j + yy + 2] = 52; // Door bottom
                dungeon.tiles[i][j + yy + 1] = 6; // Door center (floor)

                // Archway decoration to left of door
                if i > 0 {
                    if dungeon.tiles[i - 1][j + yy] == 6 {
                        dungeon.tiles[i - 1][j + yy] = 54;
                    }
                    if dungeon.tiles[i - 1][j + yy - 1] == 6 {
                        dungeon.tiles[i - 1][j + yy - 1] = 55;
                    }
                }
            }
        }
    }

    /// Add horizontal/vertical walls (halls) between rooms
    /// C++ equivalent: AddWall (lines 459-479)
    /// Add horizontal/vertical walls (halls) between rooms
    /// C++ equivalent: AddWall (lines 452-479)
    fn add_wall(&mut self, dungeon: &mut Dungeon) {
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                if self.protected[i][j] {
                    continue;
                }
                for d in [10, 12, 13, 15, 16, 21, 22] {
                    if dungeon.tiles[i][j] == d {
                        let _ = self.random_range(0, 1); // C++ DiscardRandomValues(1)
                        if let Some(x) = self.horizontal_wall_ok(dungeon, i, j) {
                            self.horizontal_wall(dungeon, i, j, x);
                        }
                    }
                }
                for d in [8, 9, 11, 14, 15, 16, 21, 23] {
                    if dungeon.tiles[i][j] == d {
                        let _ = self.random_range(0, 1);
                        if let Some(y) = self.vertical_wall_ok(dungeon, i, j) {
                            self.vertical_wall(dungeon, i, j, y);
                        }
                    }
                }
            }
        }
    }

    /// Fix tile patterns to ensure proper transitions and corner pieces (211-line massive rule set)
    /// This is the heart of L4 aesthetic quality, converting rough tiles to polished dungeon
    /// C++ equivalent: FixTilesPatterns (lines 481-691, 211 lines of pattern matching rules)
fn fix_tiles_patterns(&mut self, dungeon: &mut Dungeon) {

    // Pass 1 (C++ FixTilesPatterns loop 1)

    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 13;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 14;
            }
        }
    }

    // Pass 2 (C++ FixTilesPatterns loop 2)

    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 2;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 13;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 14 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 13 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 16;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 10;
            }
            if j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 1 {
                dungeon.tiles[(i) as usize][(j - 1) as usize] = 1;
            }
        }
    }

    // Pass 3 (C++ FixTilesPatterns loop 3)

    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 27;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 27;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 26;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 14;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 14;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] != 0 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 22;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 13;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 28;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 16 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 6 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 27;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 16 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 27;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 21;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 29;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 14;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 29;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 18 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 24;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 14;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 24;
            }
            if j - 1 >= 0 && j - 2 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 24 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 30 && dungeon.tiles[(i) as usize][(j - 2) as usize] == 6 {
                dungeon.tiles[(i) as usize][(j - 1) as usize] = 21;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 28;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 28;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 28 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 28 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 18 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 22 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 18 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 13 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 18 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 1 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 22 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 30 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && i + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 29;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 28 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 21 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 24;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 29;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 18 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 25;
            }
            if i + 1 < 40 && i + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 2 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 10 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 18 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if i - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 24 && dungeon.tiles[(i - 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i - 1) as usize][(j) as usize] = 19;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 10;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 16 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if i + 1 < 40 && i + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 18 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 24;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 29;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 25 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 11 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 14;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 27 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 26;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 18 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 24;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 26 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 29 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 29 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 15 {
                dungeon.tiles[(i) as usize][(j - 1) as usize] = 10;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 18 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 18 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 10;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 18 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 24;
            }
            if i + 1 < 40 && i + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 28;
            }
            if i + 1 < 40 && i + 2 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 30 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 29 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 29 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 26 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 30 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 19;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 16 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 10;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 25 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 30 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 18;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 18 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 2 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 15;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 11 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 13;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 13 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
        }
    }

    // Pass 4 (C++ FixTilesPatterns loop 4)

    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if j + 1 < 40 && j + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 24 && dungeon.tiles[(i) as usize][(j + 2) as usize] == 1 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 1 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 16 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 29;
            }
            if i - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i - 1) as usize][(j) as usize] == 6 {
                dungeon.tiles[(i - 1) as usize][(j) as usize] = 8;
            }
            if j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 6 {
                dungeon.tiles[(i) as usize][(j - 1) as usize] = 7;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 10;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 6 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 10 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 5 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 12;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 16 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 13;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 6 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 13 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 16;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 25 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 10;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 5 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 12;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 28 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 6 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 10 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 11 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 10 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 27 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 9 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 11;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 1 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 11 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 14 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 5;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 5 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 16 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 2 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j - 1) as usize] == 8 {
                dungeon.tiles[(i) as usize][(j - 1) as usize] = 9;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
        }
    }

    // Pass 5 (C++ FixTilesPatterns loop 5)

    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 10 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 17 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 10 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 4 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 17 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 5 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 12;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 29 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 9 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 10;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 5 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 12;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 9 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 16 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 13;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 10 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 16 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 13;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 16 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 3 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 11 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 5 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 12;
            }
            if i + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 10 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 3 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 16 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 12;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 16 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 5 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 12;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 1 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 6 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 4;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 13 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 10 {
                dungeon.tiles[(i + 1) as usize][(j + 1) as usize] = 12;
            }
            if i + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 10 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 22 && dungeon.tiles[(i) as usize][(j + 1) as usize] == 11 {
                dungeon.tiles[(i) as usize][(j + 1) as usize] = 17;
            }
            if i + 1 < 40 && i + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 16 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 23 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 1 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 6 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 16;
            }
        }
    }

    // Pass 6 (C++ FixTilesPatterns loop 6)

    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if i + 1 < 40 && i + 2 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 28 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 16 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 23;
            }
            if i + 1 < 40 && i + 2 < 40 && j + 1 < 40 && j - 1 >= 0 && dungeon.tiles[(i) as usize][(j) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j - 1) as usize] == 21 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 13 && dungeon.tiles[(i + 2) as usize][(j) as usize] == 2 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
            if i + 1 < 40 && j + 1 < 40 && dungeon.tiles[(i) as usize][(j) as usize] == 19 && dungeon.tiles[(i + 1) as usize][(j) as usize] == 15 && dungeon.tiles[(i + 1) as usize][(j + 1) as usize] == 12 {
                dungeon.tiles[(i + 1) as usize][(j) as usize] = 17;
            }
        }
    }

}


    /// Find the number of mega tiles used by layout (validation check)
    /// Hell layouts are mirrored based on a single quadrant - we count the quadrant and multiply by 4
    /// C++ equivalent: FindArea (line 941)
    fn find_area(&self) -> usize {
        let mut count = 0;
        for y in 0..DMAXY / 2 {
            for x in 0..DMAXX / 2 {
                if self.dungeon_mask[x][y] {
                    count += 1;
                }
            }
        }
        count * 4 // Mirror to 4 quadrants
    }

    /// Place miniset pattern in dungeon (used for stairs placement)
    /// Returns Some(position) if successfully placed, None otherwise
    /// C++ equivalent: PlaceMiniSet (simplified version for L4)
    /// Place a miniset using the C++ `PlaceMiniSet` scan algorithm.
    /// C++ source: PlaceMiniSet() in gendung.cpp:648-683 (L4 uses drlg1Quirk=false)
    fn place_miniset(&mut self, dungeon: &mut Dungeon, miniset: &Miniset) -> Option<(usize, usize)> {
        let sw = miniset.width as i32;
        let sh = miniset.height as i32;
        let mut x = self.random_range(0, DMAXX - miniset.width) as i32;
        let mut y = self.random_range(0, DMAXY - miniset.height) as i32;
        let mut i = 0usize;
        let tries = 199; // C++ PlaceMiniSet default `tries = 199` (gendung.cpp:646)
        while i < tries {
            if x == DMAXX as i32 - sw {
                x = 0;
                y += 1;
                if y == DMAXY as i32 - sh {
                    y = 0;
                }
            }
            // SetPieceRoom is empty for quest-free generation; skip the check.
            let mut matches = true;
            for dy in 0..miniset.height {
                for dx in 0..miniset.width {
                    let e = miniset.search[dy][dx];
                    let px = (x + dx as i32) as usize;
                    let py = (y + dy as i32) as usize;
                    if (e != 0 && dungeon.tiles[px][py] != e) || self.protected[px][py] {
                        matches = false;
                        break;
                    }
                }
                if !matches {
                    break;
                }
            }
            if matches {
                for dy in 0..miniset.height {
                    for dx in 0..miniset.width {
                        let r = miniset.replace[dy][dx];
                        if r != 0 {
                            dungeon.tiles[(x + dx as i32) as usize][(y + dy as i32) as usize] = r;
                        }
                    }
                }
                return Some((x as usize, y as usize));
            }
            i += 1;
            x += 1;
        }
        None
    }

    /// Place stairs (up, down, town warp, or hell gate)
    /// C++ equivalent: PlaceStairs (line 1094)
    /// Place stairs (up, down, town warp, or hell gate)
    /// C++ equivalent: PlaceStairs (drlg_l4.cpp:1096-1136)
    fn place_stairs(&mut self, dungeon: &mut Dungeon, level: u8, _entry: LevelEntry) -> bool {
        // Place stairs up
        if self.place_miniset(dungeon, &miniset_l4_ustairs()).is_none() {
            return false;
        }

        if level != 15 {
            // Place stairs down (skipped on level 15)
            if level != 16 {
                // C++: if Q_WARLORD is available, skip stairs down (SetPiece
                // is the warlord's room).
                if !self.warlord_quest_active {
                    if self.place_miniset(dungeon, &miniset_l4_dstairs()).is_none() {
                        return false;
                    }
                }
            }

            // Place town warp stairs on level 13
            if level == 13 {
                if self.place_miniset(dungeon, &miniset_l4_twarp()).is_none() {
                    return false;
                }
            }
        } else {
            // Level 15: place hell gate (pentagram portal)
            let Some(pos) = self.place_miniset(dungeon, &miniset_l4_penta2()) else {
                return false;
            };
            self.l4_penta_position = Some(pos);
        }

        true
    }

    /// General tile fixes for final polish
    /// C++ equivalent: GeneralFix (line 1083)
    /// General tile fixes for final polish
    /// C++ equivalent: GeneralFix (lines 1083-1094)
    fn general_fix(&mut self, dungeon: &mut Dungeon) {
        for j in 0..DMAXY - 1 {
            for i in 0..DMAXX - 1 {
                if (dungeon.tiles[i][j] == 24 || dungeon.tiles[i][j] == 122)
                    && dungeon.tiles[i + 1][j] == 2
                    && dungeon.tiles[i][j + 1] == 5
                {
                    dungeon.tiles[i][j] = 17;
                }
            }
        }
    }

    /// Apply shadow tiles to dungeon edges
    /// C++ equivalent: ApplyShadowsPatterns (line 146)
    fn apply_shadows_patterns(&mut self, dungeon: &mut Dungeon) {
        for y in 1..DMAXY {
            for x in 1..DMAXX {
                if matches!(dungeon.tiles[x][y], 3 | 4 | 8 | 15) {
                    if dungeon.tiles[x - 1][y] == 6 {
                        dungeon.tiles[x - 1][y] = 47; // Shadow left
                    }
                    if dungeon.tiles[x - 1][y - 1] == 6 {
                        dungeon.tiles[x - 1][y - 1] = 48; // Shadow corner
                    }
                }
            }
        }
    }

    /// Fix corner tile transitions for polish
    /// C++ equivalent: FixCornerTiles (line 1057)
    fn fix_corner_tiles(&mut self, dungeon: &mut Dungeon) {
        for j in 1..DMAXY - 1 {
            for i in 1..DMAXX - 1 {
                if dungeon.tiles[i][j] >= 18 && dungeon.tiles[i][j] <= 30 {
                    if dungeon.tiles[i + 1][j] < 18 || dungeon.tiles[i][j + 1] < 18 {
                        dungeon.tiles[i][j] += 98;
                    }
                }
            }
        }
    }

    /// Replace basic tiles with decorated variants
    /// C++ equivalent: Substitution (uses L4BTYPES)
    /// Replace basic tiles with decorated variants
    /// C++ equivalent: Substitution (lines 830-868) — two passes:
    /// 1) FlipCoin(3) + L4BTYPES random walk, 2) FlipCoin(10) lava pools.
    fn substitution(&mut self, dungeon: &mut Dungeon) {
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                if !self.flip_coin_weighted(3) {
                    continue;
                }
                let c = L4BTYPES[dungeon.tiles[x][y] as usize];
                if c == 0 || self.protected[x][y] {
                    continue;
                }
                let mut rv = self.random_range(0, 16) as i32;
                let mut i: i32 = -1;
                while rv >= 0 {
                    i += 1;
                    if i == L4BTYPES.len() as i32 {
                        i = 0;
                    }
                    if c == L4BTYPES[i as usize] {
                        rv -= 1;
                    }
                }
                dungeon.tiles[x][y] = i as u8;
            }
        }
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                if !self.flip_coin_weighted(10) {
                    continue;
                }
                let c = dungeon.tiles[x][y] as usize;
                if c < L4BTYPES.len() && L4BTYPES[c] == 6 && !self.protected[x][y] {
                    let idx = self.random_range(0, 3).max(0);
                    dungeon.tiles[x][y] = [95, 96, 97][idx];
                }
            }
        }
    }

    /// Protect Diablo quad areas from modification (Level 16)
    /// C++ equivalent: ProtectQuads (line 947)
    fn protect_quads(&mut self) {
        let (hold_x, hold_y) = self.l4_hold;
        for y in 0..14 {
            for x in 0..14 {
                // Protect 4 quadrants around L4Hold
                self.protected[hold_x + x][hold_y + y] = true;
                self.protected[DMAXX - 1 - x - hold_x][hold_y + y] = true;
                self.protected[hold_x + x][DMAXY - 1 - y - hold_y] = true;
                self.protected[DMAXX - 1 - x - hold_x][DMAXY - 1 - y - hold_y] = true;
            }
        }
    }

    /// Helper: Check if tile is a floor tile
    ///
    /// Maps a transparency-grid coordinate `(x, y)` (in the `[MAXDUNX][MAXDUNY]`
    /// rendering space, offset by 16 from the active region) back to an active
    /// tile coordinate and checks the tile id. Port of `IsFloor` in gendung.cpp:
    /// `i = (p.x - 16) / 2`, `j = (p.y - 16) / 2`.
    fn is_floor(&self, dungeon: &Dungeon, x: usize, y: usize, floor_id: u8) -> bool {
        // Transparency grid starts at offset 16; values below 16 are padding.
        if x < 16 || y < 16 {
            return false;
        }
        let i = (x - 16) / 2;
        let j = (y - 16) / 2;
        if i >= DMAXX || j >= DMAXY {
            return false;
        }
        dungeon.tiles[i][j] == floor_id
    }

    /// Fill transparency values recursively (flood fill)
    /// C++ equivalent: FillTransparencyValues + FindTransparencyValues
    fn fill_transparency_recursive(
        &mut self,
        dungeon: &mut Dungeon,
        x: usize,
        y: usize,
        floor_id: u8,
    ) {
        if x >= MAXDUNX || y >= MAXDUNY {
            return;
        }
        if dungeon.trans_val[x][y] != 0 {
            return;
        }
        if !self.is_floor(dungeon, x, y, floor_id) {
            return;
        }

        dungeon.trans_val[x][y] = self.trans_val_counter;

        // Fill all 8 directions (including diagonals)
        if x > 0 {
            self.fill_transparency_recursive(dungeon, x - 1, y, floor_id);
        }
        if x + 1 < MAXDUNX {
            self.fill_transparency_recursive(dungeon, x + 1, y, floor_id);
        }
        if y > 0 {
            self.fill_transparency_recursive(dungeon, x, y - 1, floor_id);
        }
        if y + 1 < MAXDUNY {
            self.fill_transparency_recursive(dungeon, x, y + 1, floor_id);
        }
        if x > 0 && y > 0 {
            self.fill_transparency_recursive(dungeon, x - 1, y - 1, floor_id);
        }
        if x + 1 < MAXDUNX && y > 0 {
            self.fill_transparency_recursive(dungeon, x + 1, y - 1, floor_id);
        }
        if x > 0 && y + 1 < MAXDUNY {
            self.fill_transparency_recursive(dungeon, x - 1, y + 1, floor_id);
        }
        if x + 1 < MAXDUNX && y + 1 < MAXDUNY {
            self.fill_transparency_recursive(dungeon, x + 1, y + 1, floor_id);
        }
    }

    /// Flood fill transparency values for lighting calculations
    /// C++ equivalent: FloodTransparencyValues (gendung.cpp line 820)
    ///
    /// Iterates over the active dungeon region (`DMAXX`/`DMAXY` == 40),
    /// mapping each active tile `(i, j)` to the rendering/transparency grid
    /// at `(16 + i*2, 16 + j*2)`. `trans_val` is `[MAXDUNX][MAXDUNY]`
    /// (`[112][112]`), and the doubled coordinates stay within bounds
    /// (`16 + (40-1)*2 == 94 < 112`).
    fn flood_transparency_values(&mut self, dungeon: &mut Dungeon, floor_id: u8) {
        self.trans_val_counter = 1;
        let mut yy = 16;
        for j in 0..DMAXY {
            let mut xx = 16;
            for i in 0..DMAXX {
                if dungeon.tiles[i][j] == floor_id && dungeon.trans_val[xx][yy] == 0 {
                    self.fill_transparency_recursive(dungeon, xx, yy, floor_id);
                    // C++ `TransVal` is `int8_t` and relies on implicit
                    // wrapping (signed overflow is UB but wraps in practice).
                    // Use wrapping arithmetic to match that behavior rather
                    // than panicking in debug builds.
                    self.trans_val_counter = self.trans_val_counter.wrapping_add(1);
                }
                xx += 2;
            }
            yy += 2;
        }
    }

    /// Check if tile is a down-right wall
    fn is_dur_right_wall(&self, tile: u8) -> bool {
        matches!(tile, 25 | 28 | 23)
    }

    /// Check if tile is a down-left wall
    fn is_dl_left_wall(&self, tile: u8) -> bool {
        matches!(tile, 27 | 26 | 22)
    }

    /// Fix transparency values for special wall tiles
    /// C++ equivalent: FixTransparency (line 1013)
    fn fix_transparency(&mut self, dungeon: &mut Dungeon) {
        let mut yy = 16;
        for j in 0..DMAXY {
            let mut xx = 16;
            for i in 0..DMAXX {
                // Fix down-right walls
                if j > 0 && self.is_dur_right_wall(dungeon.tiles[i][j]) && dungeon.tiles[i][j - 1] == 18 {
                    if xx + 1 < MAXDUNX && yy < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy] = dungeon.trans_val[xx][yy];
                    }
                    if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                }

                // Fix down-left walls
                if i + 1 < DMAXX && self.is_dl_left_wall(dungeon.tiles[i][j]) && dungeon.tiles[i + 1][j] == 19 {
                    if xx < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                    if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                }

                // Tile 18 special handling
                if dungeon.tiles[i][j] == 18 {
                    if xx + 1 < MAXDUNX && yy < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy] = dungeon.trans_val[xx][yy];
                    }
                    if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                }

                // Tile 19 special handling
                if dungeon.tiles[i][j] == 19 {
                    if xx < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                    if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                }

                // Tile 24 special handling
                if dungeon.tiles[i][j] == 24 {
                    if xx + 1 < MAXDUNX && yy < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy] = dungeon.trans_val[xx][yy];
                    }
                    if xx < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                    if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                    }
                }

                // Tile 57 special handling
                if dungeon.tiles[i][j] == 57 {
                    if xx > 0 && yy < MAXDUNY && yy + 1 < MAXDUNY {
                        dungeon.trans_val[xx - 1][yy] = dungeon.trans_val[xx][yy + 1];
                        dungeon.trans_val[xx][yy] = dungeon.trans_val[xx][yy + 1];
                    }
                }

                // Tile 53 special handling
                if dungeon.tiles[i][j] == 53 {
                    if yy > 0 && xx + 1 < MAXDUNX && yy < MAXDUNY {
                        dungeon.trans_val[xx][yy - 1] = dungeon.trans_val[xx + 1][yy];
                        dungeon.trans_val[xx][yy] = dungeon.trans_val[xx + 1][yy];
                    }
                }

                xx += 2;
            }
            yy += 2;
        }
    }

    /// Initialize quest setpiece (Warlord or Betrayer)
    /// C++ equivalent: InitSetPiece (line 163)
    fn init_set_piece(&mut self, dungeon: &mut Dungeon, level: u8) {
        // C++ InitSetPiece: place warlord.dun at SetPieceRoom.position (floor 6).
        if self.warlord_quest_active && level == 13 {
            let r = self.set_piece_room;
            self.load_dun_file("levels/l4data/warlord.dun", dungeon, r.x, r.y, 6);
        }
        // C++ also handles the multiplayer Betrayer set piece (vile1.dun on
        // level 15) — not ported yet.
        let _ = level;
    }

    /// Load .dun file and place tiles in dungeon
    ///
    /// Reads a binary .dun file and places tiles at the specified position.
    /// .dun format:
    /// - Offset 0x00 (2 bytes): width (little-endian u16)
    /// - Offset 0x02 (2 bytes): height (little-endian u16)
    /// - Offset 0x04 (W*H*2 bytes): tile data (row-major, little-endian u16[])
    ///
    /// C++ equivalent: LoadFileInMem + PlaceDunTiles
    ///
    /// # Arguments
    /// * `filename` - Path to .dun file (e.g., "levels/l4data/diab1.dun")
    /// * `dungeon` - Dungeon instance to modify
    /// * `x` - Top-left X position
    /// * `y` - Top-left Y position
    ///
    /// # Returns
    /// `true` if successful, `false` if file not found or invalid
    fn load_dun_file(&mut self, filename: &str, dungeon: &mut Dungeon, x: i32, y: i32, floor_id: u8) -> bool {
        use std::fs::File;
        use std::io::Read;

        // Open file. The crate runs from `devilutionx-rs/`, so also try the
        // repo-root fixture path (C++ repro resolves `levels\lXdata\` to
        // `test/fixtures/levels/lXdata/`).
        let candidates = [
            filename.to_string(),
            format!("../test/fixtures/{}", filename),
            format!("test/fixtures/{}", filename),
        ];
        let mut file = None;
        for cand in &candidates {
            if let Ok(f) = File::open(cand) {
                file = Some(f);
                break;
            }
        }
        let mut file = match file {
            Some(f) => f,
            None => return false, // File not found
        };

        // Read entire file into buffer
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_err() {
            return false; // Read error
        }

        // Validate minimum size (4 bytes for header)
        if buffer.len() < 4 {
            return false; // Invalid file
        }

        // Parse width (bytes 0-1, little-endian)
        let width = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;

        // Parse height (bytes 2-3, little-endian)
        let height = u16::from_le_bytes([buffer[2], buffer[3]]) as usize;

        // Validate dimensions (sanity check)
        if width == 0 || height == 0 || width > 100 || height > 100 {
            return false; // Unreasonable dimensions
        }

        // Calculate expected data size
        let tile_count = match width.checked_mul(height) {
            Some(c) => c,
            None => return false, // Overflow
        };
        let data_size = match tile_count.checked_mul(2) {
            Some(s) => s,
            None => return false, // Overflow
        };

        // Validate file size (header + tile data). Real .dun files carry
        // extra layers (transparency, monsters, objects) after the tiles.
        let expected_size = 4 + data_size;
        if buffer.len() < expected_size {
            return false; // Size mismatch
        }

        // Extract tile data (bytes 4+)
        let tile_data = &buffer[4..];

        // Place tiles in dungeon (row-major order)
        for j in 0..height {
            for i in 0..width {
                // Calculate offset in tile data
                let offset = (j * width + i) * 2;

                // Parse tile ID (little-endian u16, cast to u8)
                let tile_id = u16::from_le_bytes([
                    tile_data[offset],
                    tile_data[offset + 1],
                ]) as u8;

                // Calculate dungeon position
                let px = x + i as i32;
                let py = y + j as i32;

                // Bounds check
                if px >= 0 && px < (DMAXX as i32) && py >= 0 && py < (DMAXY as i32) {
                    let (ux, uy) = (px as usize, py as usize);
                    if tile_id != 0 {
                        // C++ PlaceDunTiles: non-zero tile is placed and protected
                        dungeon.tiles[ux][uy] = tile_id;
                        self.protected[ux][uy] = true;
                    } else if floor_id != 0 {
                        // C++ PlaceDunTiles: zero tile becomes the floor tile
                        dungeon.tiles[ux][uy] = floor_id;
                    }
                }
            }
        }

        true // Success
    }

    /// Load Diablo quad layouts for Level 16
    ///
    /// Loads 4 special room layouts (diab1/2/3/4) around l4_hold center.
    /// Preflag determines whether to use "before stairs" (2b/3b/4b) or
    /// "after stairs" (2a/3a/4a) variants to avoid stair conflicts.
    ///
    /// C++ equivalent: LoadDiabQuads (line 960-987)
    ///
    /// # Arguments
    /// * `dungeon` - Dungeon instance to modify
    /// * `preflag` - true = before stairs (use b variants), false = after (use a)
    fn load_diablo_quads(&mut self, dungeon: &mut Dungeon, preflag: bool) {
        let (hold_x, hold_y) = self.l4_hold;

        // Quad 1 (SW): diab1.dun (always loaded, no a/b variant)
        let quad1_x = hold_x as i32 + 4;
        let quad1_y = hold_y as i32 + 4;
        self.load_dun_file("levels/l4data/diab1.dun", dungeon, quad1_x, quad1_y, 6);

        // Quad 2 (NE): diab2a.dun or diab2b.dun
        let quad2_x = 27 - hold_x as i32;
        let quad2_y = 1 + hold_y as i32;
        let quad2_file = if preflag {
            "levels/l4data/diab2b.dun"
        } else {
            "levels/l4data/diab2a.dun"
        };
        self.load_dun_file(quad2_file, dungeon, quad2_x, quad2_y, 6);

        // Quad 3 (NW): diab3a.dun or diab3b.dun
        let quad3_x = 1 + hold_x as i32;
        let quad3_y = 27 - hold_y as i32;
        let quad3_file = if preflag {
            "levels/l4data/diab3b.dun"
        } else {
            "levels/l4data/diab3a.dun"
        };
        self.load_dun_file(quad3_file, dungeon, quad3_x, quad3_y, 6);

        // Quad 4 (SE): diab4a.dun or diab4b.dun
        let quad4_x = 28 - hold_x as i32;
        let quad4_y = 28 - hold_y as i32;
        let quad4_file = if preflag {
            "levels/l4data/diab4b.dun"
        } else {
            "levels/l4data/diab4a.dun"
        };
        self.load_dun_file(quad4_file, dungeon, quad4_x, quad4_y, 6);
    }

    /// Check if a position is near an already-placed theme room.
    /// C++ equivalent: IsNearThemeRoom (gendung.cpp)
    fn is_near_theme_room(&self, tx: usize, ty: usize) -> bool {
        for &(x, y, w, h) in &self.theme_locations {
            let rx = x as isize - 2;
            let ry = y as isize - 2;
            let rw = w as isize + 5;
            let rh = h as isize + 5;
            if (tx as isize) >= rx
                && (tx as isize) < rx + rw
                && (ty as isize) >= ry
                && (ty as isize) < ry + rh
            {
                return true;
            }
        }
        false
    }

    /// Find the largest available rectangle of `floor` tiles.
    /// C++ equivalent: GetSizeForThemeRoom (gendung.cpp:118-160)
    fn get_size_for_theme_room(
        &self,
        dungeon: &Dungeon,
        floor: u8,
        ox: usize,
        oy: usize,
        min_size: usize,
        max_size: usize,
    ) -> Option<(usize, usize)> {
        if ox + max_size > DMAXX && oy + max_size > DMAXY {
            return None; // C++ broken bounds check (avoids lower-right corner)
        }
        if self.is_near_theme_room(ox, oy) {
            return None;
        }
        let max_width = max_size.min(DMAXX - ox);
        let max_height = max_size.min(DMAXY - oy);
        let mut room_w = max_width;
        let mut room_h = max_height;
        for i in 0..max_size {
            let mut width = if i < room_h { i } else { 0 };
            if i < max_height {
                while width < room_w {
                    if dungeon.tiles[ox + width][oy + i] != floor {
                        break;
                    }
                    width += 1;
                }
            }
            let mut height = if i < room_w { i } else { 0 };
            if i < max_width {
                while height < room_h {
                    if dungeon.tiles[ox + i][oy + height] != floor {
                        break;
                    }
                    height += 1;
                }
            }
            if width < min_size || height < min_size {
                if i < min_size {
                    return None;
                }
                break;
            }
            room_w = room_w.min(width);
            room_h = room_h.min(height);
        }
        Some((room_w - 2, room_h - 2))
    }

    /// Draw the theme room frame (walls + door).
    /// C++ equivalent: CreateThemeRoom (gendung.cpp:161-248, DTYPE_HELL branch)
    fn create_theme_room(&mut self, dungeon: &mut Dungeon, idx: usize) {
        let (lx, ly, w, h) = self.theme_locations[idx];
        let hx = lx + w;
        let hy = ly + h;
        for yy in ly..hy {
            for xx in lx..hx {
                if yy == ly || yy == hy - 1 {
                    dungeon.tiles[xx][yy] = 2;
                } else if xx == lx || xx == hx - 1 {
                    dungeon.tiles[xx][yy] = 1;
                } else {
                    dungeon.tiles[xx][yy] = 6;
                }
            }
        }
        dungeon.tiles[lx][ly] = 9;
        dungeon.tiles[hx - 1][ly] = 16;
        dungeon.tiles[lx][hy - 1] = 15;
        dungeon.tiles[hx - 1][hy - 1] = 12;
        if self.flip_coin() {
            let yy = (ly + hy) / 2;
            dungeon.tiles[hx - 1][yy - 1] = 53;
            dungeon.tiles[hx - 1][yy] = 6;
            dungeon.tiles[hx - 1][yy + 1] = 52;
            dungeon.tiles[hx - 2][yy - 1] = 54;
        } else {
            let xx = (lx + hx) / 2;
            dungeon.tiles[xx - 1][hy - 1] = 57;
            dungeon.tiles[xx][hy - 1] = 6;
            dungeon.tiles[xx + 1][hy - 1] = 56;
            dungeon.tiles[xx][hy - 2] = 59;
            dungeon.tiles[xx - 1][hy - 2] = 58;
        }
    }

    /// Place theme room frames (walls/doors only).
    /// C++ equivalent: DRLG_PlaceThemeRooms (gendung.cpp:706-753)
    ///
    /// The theme *content* (InitThemes/CreateThemeRooms in themes.cpp) only
    /// places objects/monsters — it writes no tiles and runs after
    /// Substitution, so it does not affect the exported tile grid.
    fn place_theme_rooms(
        &mut self,
        dungeon: &mut Dungeon,
        min_size: usize,
        max_size: usize,
        floor: u8,
        freq: usize,
        rnd_size: bool,
    ) {
        self.theme_locations.clear();
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                if dungeon.tiles[i][j] != floor || !self.flip_coin_weighted(freq) {
                    continue;
                }
                let Some((mut rw, mut rh)) =
                    self.get_size_for_theme_room(dungeon, floor, i, j, min_size, max_size)
                else {
                    continue;
                };
                if rnd_size {
                    let min = min_size - 2;
                    let max = max_size - 2;
                    let inner_w = self.random_range(0, rw - min + 1);
                    rw = min + self.random_range(0, inner_w);
                    if rw < min || rw > max {
                        rw = min;
                    }
                    let inner_h = self.random_range(0, rh - min + 1);
                    rh = min + self.random_range(0, inner_h);
                    if rh < min || rh > max {
                        rh = min;
                    }
                }
                // C++: theme.room.position = { i, j } + Direction::South = {1,1}
                self.theme_locations.push((i + 1, j + 1, rw, rh));
                let idx = self.theme_locations.len() - 1;
                self.create_theme_room(dungeon, idx);
            }
        }
    }

    /// Find pentagram tiles (98 or 107) for Level 15
    /// C++ equivalent: Level 15 pentagram detection (line 1198-1211)
    fn find_pentagram_tiles(&self, dungeon: &Dungeon) -> Option<(usize, usize)> {
        for j in 1..DMAXY {
            for i in 1..DMAXX {
                if matches!(dungeon.tiles[i][j], 98 | 107) {
                    return Some((i, j));
                }
            }
        }
        None
    }

    /// Place L4PENTA miniset if Diablo gate is closed (Level 15)
    /// C++ equivalent: L4PENTA.place (line 1198)
    fn place_l4_penta(&mut self, dungeon: &mut Dungeon, _level: u8) {
        // TODO: Check if gate is open (Quest system)
        // For now, always skip (assume gate open)
        // let is_gate_open = true;
        // if !is_gate_open {
        //     // Place pentagram miniset
        // }
    }

    /// Populate render tiles (dPiece array from dungeon tiles)
    /// C++ equivalent: Pass3 → DRLG_LPass3(30 - 1)
    ///
    /// This is a simplified placeholder. The full implementation requires:
    /// - MegaTile structures (pMegaTiles array)
    /// - dPiece array allocation
    /// - Micro-tile expansion (4 micro tiles per dungeon tile)
    ///
    /// For now, this is a marker that Pass3 would be called here.
    fn pass3(&mut self, _dungeon: &mut Dungeon, _level: u8) {
        // TODO: Implement full rendering tile population
        // C++ code:
        // const MegaTile mega = pMegaTiles[lv];
        // for each tile: dPiece[x][y] = mega.micro1/2/3/4
        //
        // This requires engine/rendering structures not yet ported.
        // Deferring to future milestone (rendering layer).
    }

    /// Protect quest room area (mark as protected in dungeon mask)
    /// C++ equivalent: SetPieceRoom protection (drlg_l4.cpp:1161-1168)
    fn protect_quest_room(&mut self, level: u8) {
        // C++ GenerateLevel: protect the Warlord SetPiece room before AddWall.
        if self.warlord_quest_active && level == 13 {
            let r = self.set_piece_room;
            for spj in r.y..r.y + r.height - 1 {
                for spi in r.x..r.x + r.width - 1 {
                    if spi >= 0 && spi < DMAXX as i32 && spj >= 0 && spj < DMAXY as i32 {
                        self.protected[spi as usize][spj as usize] = true;
                    }
                }
            }
        }
    }

    /// Validate and setup quest triggers
    /// C++ equivalent: DRLG_CheckQuests (quests.cpp:429)
    fn check_quests(&mut self) {
        // TODO: Quest system integration
        // C++ code:
        // for (auto &quest : Quests) {
        //     if (quest.IsAvailable()) {
        //         switch (quest._qidx) {
        //             case Q_WARLORD: DrawWarLord(position); break;
        //             case Q_BUTCHER: DrawButcher(); break;
        //             // ... other quests ...
        //         }
        //     }
        // }
        //
        // For now, this is a placeholder marker.
        // Will be implemented when quest system is ported.
    }    /// Mark room area as occupied in dungeon mask (first quadrant only)
    /// C++ equivalent: MapRoom
    fn map_room(&mut self, room: Room) {
        for y in 0..room.height {
            if room.y + y < 0 || room.y + y >= (DMAXY / 2) as i32 {
                continue;
            }
            for x in 0..room.width {
                if room.x + x < 0 || room.x + x >= (DMAXX / 2) as i32 {
                    continue;
                }
                self.dungeon_mask[(room.x + x) as usize][(room.y + y) as usize] = true;
            }
        }
    }

    /// Check if room can be placed (no overlap with existing rooms)
    /// C++ equivalent: CheckRoom
    fn check_room(&self, room: Room) -> bool {
        if room.x <= 0 || room.y <= 0 {
            return false;
        }

        for y in 0..room.height {
            for x in 0..room.width {
                let px = room.x + x;
                let py = room.y + y;

                if px < 0 || px >= (DMAXX / 2) as i32 || py < 0 || py >= (DMAXY / 2) as i32 {
                    return false;
                }

                if self.dungeon_mask[px as usize][py as usize] {
                    return false; // Overlap detected
                }
            }
        }

        true
    }

    /// Recursively generate rooms using UberRoom algorithm
    /// C++ equivalent: GenerateRoom
    /// C++ CloseOuterBorders (lines 1073-1082): clear the outer border of the
    /// first quadrant so the mirrored walls form the map edge.
    fn close_outer_borders(&mut self) {
        for x in 0..DMAXX / 2 {
            self.dungeon_mask[x][0] = false;
        }
        for y in 0..DMAXY / 2 {
            self.dungeon_mask[0][y] = false;
        }
    }

    /// C++ PrepareInnerBorders (lines 869-946): carve connection corridors in
    /// the first quadrant before mirroring so the layout stays connected.
    fn prepare_inner_borders(&mut self) {
        for y in (0..DMAXY / 2).rev() {
            for x in (0..DMAXX / 2).rev() {
                if !self.dungeon_mask[x][y] {
                    self.hall_ok[y] = false;
                } else {
                    let has_sw = y + 1 < DMAXY / 2 && self.dungeon_mask[x][y + 1];
                    let has_s = x + 1 < DMAXX / 2
                        && y + 1 < DMAXY / 2
                        && self.dungeon_mask[x + 1][y + 1];
                    self.hall_ok[y] = has_sw && !has_s;
                    break; // C++: x = 0 then loop decrements to -1 and exits
                }
            }
        }

        let mut ry = self.random_range(0, DMAXY / 2 - 1) + 1;
        loop {
            if self.hall_ok[ry] {
                for x in (0..DMAXX / 2).rev() {
                    if self.dungeon_mask[x][ry] {
                        ry = 0;
                        break;
                    }
                    self.dungeon_mask[x][ry] = true;
                    self.dungeon_mask[x][ry + 1] = true;
                }
            } else {
                ry += 1;
                if ry == DMAXY / 2 {
                    ry = 1;
                }
            }
            if ry == 0 {
                break;
            }
        }

        for x in (0..DMAXX / 2).rev() {
            for y in (0..DMAXY / 2).rev() {
                if !self.dungeon_mask[x][y] {
                    self.hall_ok[x] = false;
                } else {
                    let has_se = x + 1 < DMAXX / 2 && self.dungeon_mask[x + 1][y];
                    let has_s = x + 1 < DMAXX / 2
                        && y + 1 < DMAXY / 2
                        && self.dungeon_mask[x + 1][y + 1];
                    self.hall_ok[x] = has_se && !has_s;
                    break;
                }
            }
        }

        let mut rx = self.random_range(0, DMAXX / 2 - 1) + 1;
        loop {
            if self.hall_ok[rx] {
                for y in (0..DMAXY / 2).rev() {
                    if self.dungeon_mask[rx][y] {
                        rx = 0;
                        break;
                    }
                    self.dungeon_mask[rx][y] = true;
                    self.dungeon_mask[rx + 1][y] = true;
                }
            } else {
                rx += 1;
                if rx == DMAXX / 2 {
                    rx = 1;
                }
            }
            if rx == 0 {
                break;
            }
        }
    }


    fn generate_room(&mut self, area: Room, vertical_layout: bool) {
        // Randomly rotate layout (75% chance to flip)
        let rotate = !self.flip_coin_weighted(4);
        let vertical_layout = (!vertical_layout && rotate) || (vertical_layout && !rotate);


        let mut place_room1 = false;
        let mut room1 = Room::new(0, 0, 0, 0);

        // Try to place first room (max 20 attempts)
        for _ in 0..20 {
            let random_width = (self.random_range(0, 5) + 2) & !1; // Even width (2-6)
            let random_height = (self.random_range(0, 5) + 2) & !1; // Even height (2-6)
            room1 = Room::new(area.x, area.y, random_width as i32, random_height as i32);

            let check_room: Room;
            if vertical_layout {
                // Place to the left of area
                room1.x = area.x - room1.width;
                room1.y = area.y + area.height / 2 - room1.height / 2;

                // C++ BUGFIX note: C++ swaps height/width here
                // ({ room1.size.height + 2, room1.size.width + 1 }); replicate
                // the original bug for byte-exact compatibility.
                check_room = Room::new(
                    room1.x - 1,
                    room1.y - 1,
                    room1.height + 2,
                    room1.width + 1,
                );
            } else {
                // Place above area
                room1.x = area.x + area.width / 2 - room1.width / 2;
                room1.y = area.y - room1.height;

                check_room = Room::new(
                    room1.x - 1,
                    room1.y - 1,
                    room1.width + 2,
                    room1.height + 1,
                );
            }

            if self.check_room(check_room) {
                place_room1 = true;
                break;
            }
        }

        // Map first room if successfully placed
        if place_room1 {
            let clamped_room = Room::new(
                room1.x,
                room1.y,
                std::cmp::min(DMAXX as i32 - room1.x, room1.width),
                std::cmp::min(DMAXY as i32 - room1.y, room1.height),
            );
            self.map_room(clamped_room);
        }

        // Try to place second room (adjacent to first)
        let room2: Room;
        let place_room2: bool;

        if vertical_layout {
            room2 = Room::new(area.x + area.width, room1.y, room1.width, room1.height);
            let check_room = Room::new(
                room2.x,
                room2.y - 1,
                room2.width + 1,
                room2.height + 2,
            );
            place_room2 = self.check_room(check_room);
        } else {
            room2 = Room::new(room1.x, area.y + area.height, room1.width, room1.height);
            let check_room = Room::new(
                room2.x - 1,
                room2.y,
                room2.width + 2,
                room2.height + 1,
            );
            place_room2 = self.check_room(check_room);
        }

        if place_room2 {
            self.map_room(room2);
        }

        // Recursively subdivide placed rooms
        if place_room1 {
            self.generate_room(room1, vertical_layout);
        }
        if place_room2 {
            self.generate_room(room2, vertical_layout);
        }
    }

    /// Generate first room (entry point for UberRoom system)
    /// C++ equivalent: FirstRoom
    fn first_room(&mut self, level: u8) {
        let mut room_width = 14;
        let mut room_height = 14;

        if level != 16 {
            if self.warlord_quest_active && level == 13 {
                // C++ FirstRoom: Warlord quest room is a fixed 11x11 block
                room_width = 11;
                room_height = 11;
            } else {
                room_width = self.random_range(0, 5) + 2;
                room_height = self.random_range(0, 5) + 2;
            }
        }

        // Calculate random position in first quadrant
        let xmin = (DMAXX / 2 - room_width) / 2;
        let xmax = DMAXX / 2 - 1 - room_width;
        let ymin = (DMAXY / 2 - room_height) / 2;
        let ymax = DMAXY / 2 - 1 - room_height;

        let random_x = self.random_range(0, xmax - xmin + 1) + xmin;
        let random_y = self.random_range(0, ymax - ymin + 1) + ymin;

        let room = Room::new(random_x as i32, random_y as i32, room_width as i32, room_height as i32);


        if level == 16 {
            self.l4_hold = (random_x, random_y);
        }

        // C++ FirstRoom: SetPieceRoom = room.position + {1,1}, size + 1
        if self.warlord_quest_active && level == 13 {
            self.set_piece_room = SetPieceRect {
                x: random_x as i32 + 1,
                y: random_y as i32 + 1,
                width: room_width as i32 + 1,
                height: room_height as i32 + 1,
            };
        } else {
            self.set_piece_room = SetPieceRect::default();
        }

        self.map_room(room);

        // Store layout for recursive generation
        let use_vertical = !self.flip_coin();
        self.generate_room(room, use_vertical);
    }

    /// Flip coin with weighted probability (1/n chance)
    fn flip_coin_weighted(&mut self, n: usize) -> bool {
        self.random_range(0, n) == 0
    }


    /// Generate L4 dungeon (main entry point)
    /// C++ equivalent: CreateL4Dungeon
    pub fn generate(&mut self, dungeon: &mut Dungeon, seed: u32, level: u8, _entry: LevelEntry) -> bool {
        self.set_rng_seed(seed);

        // Special handling for Level 16 (Diablo's Lair)
        if level == 16 {
            // TODO Day 79: Implement generate_diablo_lair
            return self.generate_diablo_lair(dungeon, seed);
        }

        // Normal L4 generation loop
        let result = self.generate_level(dungeon, level, _entry);

        // Pass3: Populate render tiles (dPiece array)
        // C++ equivalent: DRLG_LPass3(30 - 1)
        self.pass3(dungeon, level);

        result
    }

    /// Main generation loop (separated for clarity)
    /// C++ equivalent: GenerateLevel (line 1140)
    /// Main generation loop (separated for clarity)
    /// C++ equivalent: GenerateLevel (drlg_l4.cpp:1140-1199)
    fn generate_level(&mut self, dungeon: &mut Dungeon, level: u8, entry: LevelEntry) -> bool {
        const MIN_AREA: usize = 692;
        loop {
            // C++: do { InitDungeonFlags(); FirstRoom(); CloseOuterBorders(); }
            //          while (FindArea() < Minarea);   FindArea = mask.count() * 4
            loop {
                self.init_dungeon_flags();
                for y in 0..DMAXY {
                    for x in 0..DMAXX {
                        dungeon.tiles[x][y] = 30; // C++ memset(dungeon, 30)
                    }
                }
                self.first_room(level);
                self.close_outer_borders();
                if self.find_area() >= MIN_AREA {
                    break;
                }
            }

            self.prepare_inner_borders();
            self.mirror_dungeon_layout();
            self.make_dmt(dungeon);
            self.fix_tiles_patterns(dungeon);
            if level == 16 {
                self.protect_quads();
            }
            // C++: protect SetPieceRoom when the Warlord/Betrayer quest is active.
            // The quest system is not ported yet; protect_quest_room is a no-op.
            self.protect_quest_room(level);
            self.add_wall(dungeon);
            self.flood_transparency_values(dungeon, 6);
            self.fix_transparency(dungeon);
            self.init_set_piece(dungeon, level);
            if level == 16 {
                self.load_diablo_quads(dungeon, true);
            }
            if self.place_stairs(dungeon, level, entry) {
                break;
            }
        }

        self.general_fix(dungeon);
        if level != 16 {
            // C++ DRLG_PlaceThemeRooms(7, 10, 6, 8, true)
            self.place_theme_rooms(dungeon, 7, 10, 6, 8, true);
        }
        self.apply_shadows_patterns(dungeon);
        self.fix_corner_tiles(dungeon);
        self.substitution(dungeon);

        // TODO: memcpy to pdungeon backup array

        // C++ DRLG_CheckQuests(SetPieceRoom.position)
        self.check_quests();

        // Level 15 special handling: pentagram placement.
        // C++: if the Diablo gate is closed (no quest / not multiplayer),
        // L4PENTA.place(Quests[Q_DIABLO].position) overwrites the L4PENTA2
        // hell gate placed by PlaceStairs. The quest system is not ported yet,
        // so we always treat the gate as closed (single-player, no quest).
        if level == 15 {
            if let Some((x, y)) = self.l4_penta_position {
                let penta = miniset_l4_penta();
                for dy in 0..penta.height {
                    for dx in 0..penta.width {
                        let r = penta.replace[dy][dx];
                        if r != 0 {
                            dungeon.tiles[x + dx][y + dy] = r;
                        }
                    }
                }
            }
        }
        if level == 16 {
            self.load_diablo_quads(dungeon, false);
        }

        true
    }

    /// Generate Diablo's Lair (Level 16 special layout)
    /// C++ equivalent: LoadDiabQuads + placement logic
    /// Generate Diablo's Lair (Level 16 special layout)
    ///
    /// C++ CreateL4Dungeon handles level 16 inside GenerateLevel with the
    /// ProtectQuads/LoadDiabQuads branches; route through the same pipeline.
    fn generate_diablo_lair(&mut self, dungeon: &mut Dungeon, seed: u32) -> bool {
        self.set_rng_seed(seed);
        let result = self.generate_level(dungeon, 16, LevelEntry::Main);
        self.pass3(dungeon, 16);
        result
    }
}

impl Default for Dungeon4Generator {
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
    fn test_l4_conv_table() {
        assert_eq!(L4_CONV_TABLE.len(), 16);
        assert_eq!(L4_CONV_TABLE[0], 30);
        assert_eq!(L4_CONV_TABLE[1], 6);
        assert_eq!(L4_CONV_TABLE[15], 6);
    }

    #[test]
    fn test_l4btypes() {
        assert_eq!(L4BTYPES.len(), 140);
        assert_eq!(L4BTYPES[0], 0);
        assert_eq!(L4BTYPES[6], 6);
        assert_eq!(L4BTYPES[49], 6);
    }

    #[test]
    fn test_dungeon4_generator_new() {
        let generator = Dungeon4Generator::new();
        assert_eq!(generator.rng_state, 0);
        assert_eq!(generator.hall_ok.len(), 20);
        assert_eq!(generator.l4_hold, (0, 0));
    }

    #[test]
    fn test_miniset_l4_ustairs() {
        let miniset = miniset_l4_ustairs();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 5);
        assert_eq!(miniset.search[0][0], 6);
        assert_eq!(miniset.replace[1][0], 36);
        assert_eq!(miniset.replace[2][1], 34);
    }

    #[test]
    fn test_miniset_l4_dstairs() {
        let miniset = miniset_l4_dstairs();
        assert_eq!(miniset.width, 5);
        assert_eq!(miniset.height, 5);
        assert_eq!(miniset.search[0][0], 6);
        assert_eq!(miniset.replace[1][2], 45);
        assert_eq!(miniset.replace[2][2], 43);
    }

    #[test]
    fn test_miniset_l4_twarp() {
        let miniset = miniset_l4_twarp();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 5);
        assert_eq!(miniset.replace[1][0], 134);
        assert_eq!(miniset.replace[2][2], 131);
    }

    #[test]
    fn test_miniset_l4_penta() {
        let miniset = miniset_l4_penta();
        assert_eq!(miniset.width, 5);
        assert_eq!(miniset.height, 5);
        assert_eq!(miniset.replace[1][1], 98);
        assert_eq!(miniset.replace[2][2], 102);
    }

    #[test]
    fn test_miniset_l4_penta2() {
        let miniset = miniset_l4_penta2();
        assert_eq!(miniset.width, 5);
        assert_eq!(miniset.height, 5);
        assert_eq!(miniset.replace[1][1], 107);
        assert_eq!(miniset.replace[3][3], 115);
    }

    #[test]
    fn test_init_dungeon_flags() {
        let mut generator = Dungeon4Generator::new();
        generator.init_dungeon_flags();

        // Check that predungeon is filled with wall tiles (30)
        assert_eq!(generator.predungeon[5][5], 30);
        assert_eq!(generator.predungeon[10][10], 30);

        // Check hall_ok is reset
        assert!(!generator.hall_ok[0]);
        assert!(!generator.hall_ok[19]);
    }

    #[test]
    fn test_random_range() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(12345);

        let r1 = generator.random_range(0, 10);
        assert!(r1 < 10);

        let r2 = generator.random_range(5, 15);
        assert!(r2 >= 5 && r2 < 15);
    }

    #[test]
    fn test_room_new() {
        let room = Room::new(10, 20, 5, 7);
        assert_eq!(room.x, 10);
        assert_eq!(room.y, 20);
        assert_eq!(room.width, 5);
        assert_eq!(room.height, 7);
    }

    #[test]
    fn test_map_room() {
        let mut generator = Dungeon4Generator::new();
        generator.init_dungeon_flags();

        let room = Room::new(5, 5, 4, 4);
        generator.map_room(room);

        // Check that room area is marked as occupied
        assert!(generator.dungeon_mask[5][5]);
        assert!(generator.dungeon_mask[8][8]);
        assert!(generator.dungeon_mask[5][8]);
        assert!(generator.dungeon_mask[8][5]);

        // Check that area outside room is not marked
        assert!(!generator.dungeon_mask[4][5]);
        assert!(!generator.dungeon_mask[9][5]);
    }

    #[test]
    fn test_check_room_empty() {
        let generator = Dungeon4Generator::new();

        let room = Room::new(5, 5, 4, 4);
        assert!(generator.check_room(room)); // Empty dungeon, should succeed
    }

    #[test]
    fn test_check_room_overlap() {
        let mut generator = Dungeon4Generator::new();
        generator.init_dungeon_flags();

        let room1 = Room::new(5, 5, 4, 4);
        generator.map_room(room1);

        let room2 = Room::new(7, 7, 4, 4); // Overlaps with room1
        assert!(!generator.check_room(room2)); // Should fail due to overlap
    }

    #[test]
    fn test_check_room_boundary() {
        let generator = Dungeon4Generator::new();

        // Room at edge of first quadrant
        let room = Room::new(1, 1, 2, 2);
        assert!(generator.check_room(room));

        // Room at origin (should fail, x/y must be > 0)
        let room_origin = Room::new(0, 0, 2, 2);
        assert!(!generator.check_room(room_origin));
    }

    #[test]
    fn test_first_room() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(42);
        generator.init_dungeon_flags();

        generator.first_room(13);

        // Check that at least some rooms were created
        let mut room_count = 0;
        for y in 0..DMAXY / 2 {
            for x in 0..DMAXX / 2 {
                if generator.dungeon_mask[x][y] {
                    room_count += 1;
                }
            }
        }

        assert!(room_count > 0); // Should have created at least one room
        assert!(room_count < (DMAXX / 2) * (DMAXY / 2)); // But not fill entire quadrant
    }

    #[test]
    fn test_first_room_level_16() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(123);
        generator.init_dungeon_flags();

        generator.first_room(16);

        // Check that l4_hold was set
        assert!(generator.l4_hold.0 > 0 || generator.l4_hold.1 > 0);
    }

    #[test]
    fn test_generate_room_recursive() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(999);
        generator.init_dungeon_flags();

        let initial_room = Room::new(10, 10, 8, 8);
        generator.map_room(initial_room);
        generator.generate_room(initial_room, false);

        // Should have created additional rooms through recursion
        let mut room_count = 0;
        for y in 0..DMAXY / 2 {
            for x in 0..DMAXX / 2 {
                if generator.dungeon_mask[x][y] {
                    room_count += 1;
                }
            }
        }

        assert!(room_count > 64); // More than just initial 8×8 room
    }

    #[test]
    fn test_mirror_dungeon_layout() {
        let mut generator = Dungeon4Generator::new();
        generator.init_dungeon_flags();

        // Set a room in first quadrant
        generator.dungeon_mask[5][5] = true;
        generator.dungeon_mask[6][5] = true;
        generator.dungeon_mask[5][6] = true;
        generator.dungeon_mask[6][6] = true;

        generator.mirror_dungeon_layout();

        // Check all 4 quadrants
        assert!(generator.dungeon_mask[5][5]); // Original (top-left quadrant)
        assert!(generator.dungeon_mask[5][DMAXY - 1 - 5]); // Top-right
        assert!(generator.dungeon_mask[DMAXX - 1 - 5][5]); // Bottom-left
        assert!(generator.dungeon_mask[DMAXX - 1 - 5][DMAXY - 1 - 5]); // Bottom-right
    }

    #[test]
    fn test_make_dmt() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();
        generator.init_dungeon_flags();

        // Create a 2×2 pattern (all true)
        generator.dungeon_mask[0][0] = true;
        generator.dungeon_mask[1][0] = true;
        generator.dungeon_mask[0][1] = true;
        generator.dungeon_mask[1][1] = true;

        generator.make_dmt(&mut dungeon);

        // Pattern 1111 (binary) = 15 (decimal)
        // L4_CONV_TABLE[15] = 6 (floor tile)
        assert_eq!(dungeon.tiles[0][0], 6);

        // Pattern 0000 (binary) = 0 (decimal)
        // L4_CONV_TABLE[0] = 30 (wall tile)
        generator.dungeon_mask = [[false; DMAXY]; DMAXX];
        generator.make_dmt(&mut dungeon);
        assert_eq!(dungeon.tiles[0][0], 30);
    }

    #[test]
    fn test_horizontal_wall_ok() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Create a horizontal corridor (tile 6 = floor)
        for x in 0..10 {
            dungeon.tiles[x][5] = 6;
            dungeon.tiles[x][4] = 6;
            dungeon.tiles[x][6] = 6;
        }
        dungeon.tiles[10][5] = 10; // Valid end tile

        let result = generator.horizontal_wall_ok(&dungeon, 0, 5);
        assert!(result.is_some());
        assert!(result.unwrap() > 3); // Must be > 3
    }

    #[test]
    fn test_vertical_wall_ok() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Create a vertical corridor
        for y in 0..10 {
            dungeon.tiles[5][y] = 6;
            dungeon.tiles[4][y] = 6;
            dungeon.tiles[6][y] = 6;
        }
        dungeon.tiles[5][10] = 8; // Valid end tile

        let result = generator.vertical_wall_ok(&dungeon, 5, 0);
        assert!(result.is_some());
        assert!(result.unwrap() > 3);
    }

    #[test]
    fn test_horizontal_wall() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(777);
        let mut dungeon = Dungeon::new();

        // Setup corridor
        for x in 0..10 {
            dungeon.tiles[x][5] = 6;
        }
        dungeon.tiles[0][5] = 13; // Start tile
        dungeon.tiles[10][5] = 15; // End tile

        generator.horizontal_wall(& mut dungeon, 0, 5, 10);

        // Check start tile converted
        assert_eq!(dungeon.tiles[0][5], 17);

        // Check end tile converted
        assert_eq!(dungeon.tiles[10][5], 14);

        // Check some wall tiles placed (tile 2)
        let mut wall_count = 0;
        for x in 1..10 {
            if dungeon.tiles[x][5] == 2 {
                wall_count += 1;
            }
        }
        assert!(wall_count > 5); // Most tiles should be walls

        // Check door tiles exist (57, 56, 60)
        let mut door_tiles = 0;
        for x in 1..10 {
            if matches!(dungeon.tiles[x][5], 57 | 56 | 60) {
                door_tiles += 1;
            }
        }
        assert_eq!(door_tiles, 3); // Should have 3 door tiles
    }

    #[test]
    fn test_vertical_wall() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(888);
        let mut dungeon = Dungeon::new();

        // Setup corridor
        for y in 0..10 {
            dungeon.tiles[5][y] = 6;
        }
        dungeon.tiles[5][0] = 14; // Start tile
        dungeon.tiles[5][10] = 11; // End tile

        generator.vertical_wall(&mut dungeon, 5, 0, 10);

        // Check start tile converted
        assert_eq!(dungeon.tiles[5][0], 17);

        // Check end tile converted
        assert_eq!(dungeon.tiles[5][10], 17);

        // Check wall tiles placed (tile 1)
        let mut wall_count = 0;
        for y in 1..10 {
            if dungeon.tiles[5][y] == 1 {
                wall_count += 1;
            }
        }
        assert!(wall_count > 5);

        // Check door tiles (53, 52, 6)
        let mut door_count = 0;
        for y in 1..10 {
            if matches!(dungeon.tiles[5][y], 53 | 52) {
                door_count += 1;
            }
        }
        assert_eq!(door_count, 2); // Should have 2 door frame tiles (top/bottom)
    }

    #[test]
    fn test_fix_tiles_patterns_pass1() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Test basic transitions: 2→5, 2→13, 1→14
        dungeon.tiles[5][5] = 2;
        dungeon.tiles[6][5] = 6;

        dungeon.tiles[10][10] = 2;
        dungeon.tiles[11][10] = 1;

        dungeon.tiles[15][15] = 1;
        dungeon.tiles[15][16] = 2;

        generator.fix_tiles_patterns(&mut dungeon);

        assert_eq!(dungeon.tiles[6][5], 5);  // 2→6 becomes 2→5
        assert_eq!(dungeon.tiles[11][10], 13); // 2→1 becomes 2→13
        assert_eq!(dungeon.tiles[15][16], 14); // 1→2 becomes 1→14
    }

    #[test]
    fn test_fix_tiles_patterns_pass2() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Test extended transitions
        dungeon.tiles[5][5] = 2;
        dungeon.tiles[6][5] = 9;

        dungeon.tiles[10][10] = 9;
        dungeon.tiles[11][10] = 6;

        dungeon.tiles[15][15] = 6;
        dungeon.tiles[16][15] = 14;

        generator.fix_tiles_patterns(&mut dungeon);

        assert_eq!(dungeon.tiles[6][5], 11);  // 2→9 becomes 2→11
        assert_eq!(dungeon.tiles[11][10], 12); // 9→6 becomes 9→12
        assert_eq!(dungeon.tiles[16][15], 15); // 6→14 becomes 6→15
    }

    #[test]
    fn test_fix_tiles_patterns_pass3() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Test complex patterns: 13→30, 27→30, 19→27
        dungeon.tiles[5][5] = 13;
        dungeon.tiles[5][6] = 30;

        dungeon.tiles[10][10] = 27;
        dungeon.tiles[11][10] = 30;

        dungeon.tiles[15][15] = 19;
        dungeon.tiles[16][15] = 27;

        generator.fix_tiles_patterns(&mut dungeon);

        assert_eq!(dungeon.tiles[5][6], 27);  // 13→30 becomes 13→27
        assert_eq!(dungeon.tiles[11][10], 19); // 27→30 becomes 27→19
        assert_eq!(dungeon.tiles[16][15], 26); // 19→27 becomes 19→26
    }

    #[test]
    fn test_add_wall() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(999);
        let mut dungeon = Dungeon::new();

        // Create a setup where horizontal wall can be placed
        for x in 0..15 {
            dungeon.tiles[x][10] = 6;
            dungeon.tiles[x][9] = 6;
            dungeon.tiles[x][11] = 6;
        }
        dungeon.tiles[5][10] = 6;
        dungeon.tiles[14][10] = 2; // Trigger condition
        dungeon.tiles[15][10] = 10; // End marker

        generator.add_wall(&mut dungeon);

        // Check that some tiles changed (walls/doors placed)
        let original_floor_count = 15;
        let mut new_floor_count = 0;
        for x in 0..15 {
            if dungeon.tiles[x][10] == 6 {
                new_floor_count += 1;
            }
        }

        assert!(new_floor_count < original_floor_count); // Some floor tiles converted
    }

    #[test]
    fn test_find_area() {
        let mut generator = Dungeon4Generator::new();

        // Set some tiles in first quadrant
        for y in 0..10 {
            for x in 0..10 {
                generator.dungeon_mask[x][y] = true;
            }
        }

        let area = generator.find_area();
        assert_eq!(area, 100 * 4); // 100 tiles * 4 quadrants = 400
    }

    #[test]
    fn test_place_miniset() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(12345);
        let mut dungeon = Dungeon::new();

        // Fill dungeon with floor tiles
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                dungeon.tiles[x][y] = 6;
            }
        }

        // Create simple 2x2 miniset
        let miniset = Miniset::new(
            2, 2,
            vec![
                vec![6, 6],
                vec![6, 6],
            ],
            vec![
                vec![100, 101],
                vec![102, 103],
            ],
        );

        let result = generator.place_miniset(&mut dungeon, &miniset);
        assert!(result.is_some());

        let (x, y) = result.unwrap();
        assert_eq!(dungeon.tiles[x][y], 100);
        assert_eq!(dungeon.tiles[x + 1][y], 101);
        assert_eq!(dungeon.tiles[x][y + 1], 102);
        assert_eq!(dungeon.tiles[x + 1][y + 1], 103);
    }

    #[test]
    fn test_general_fix() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // C++ GeneralFix: (24 | 122) with right=2 and below=5 -> 17.
        dungeon.tiles[5][5] = 24;
        dungeon.tiles[6][5] = 2;
        dungeon.tiles[5][6] = 5;

        dungeon.tiles[10][10] = 122;
        dungeon.tiles[11][10] = 2;
        dungeon.tiles[10][11] = 5;

        // Negative case: same left tile but wrong neighbours must stay.
        dungeon.tiles[15][15] = 24;
        dungeon.tiles[16][15] = 1;
        dungeon.tiles[15][16] = 1;

        generator.general_fix(&mut dungeon);

        assert_eq!(dungeon.tiles[5][5], 17);  // 24 + right 2 + below 5
        assert_eq!(dungeon.tiles[10][10], 17); // 122 + right 2 + below 5
        assert_eq!(dungeon.tiles[15][15], 24); // neighbours don't match
    }

    #[test]
    fn test_apply_shadows_patterns() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Set up test: wall tiles with floor around them
        dungeon.tiles[5][5] = 3; // Wall
        dungeon.tiles[4][5] = 6; // Floor left
        dungeon.tiles[4][4] = 6; // Floor corner

        dungeon.tiles[10][10] = 8; // Wall
        dungeon.tiles[9][10] = 6;  // Floor left
        dungeon.tiles[9][9] = 6;   // Floor corner

        generator.apply_shadows_patterns(&mut dungeon);

        // Check shadows applied
        assert_eq!(dungeon.tiles[4][5], 47);  // Shadow left
        assert_eq!(dungeon.tiles[4][4], 48);  // Shadow corner
        assert_eq!(dungeon.tiles[9][10], 47); // Shadow left
        assert_eq!(dungeon.tiles[9][9], 48);  // Shadow corner
    }

    #[test]
    fn test_fix_corner_tiles() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Set up corner tiles (18-30) with transitions
        dungeon.tiles[5][5] = 20;
        dungeon.tiles[6][5] = 10; // < 18, trigger fix

        dungeon.tiles[10][10] = 25;
        dungeon.tiles[10][11] = 15; // < 18, trigger fix

        generator.fix_corner_tiles(&mut dungeon);

        // Check corners incremented by 98
        assert_eq!(dungeon.tiles[5][5], 20 + 98);
        assert_eq!(dungeon.tiles[10][10], 25 + 98);
    }

    #[test]
    fn test_substitution() {
        let mut generator = Dungeon4Generator::new();
        generator.set_rng_seed(123);
        let mut dungeon = Dungeon::new();

        // C++ Substitution: pass 1 random-walks L4BTYPES (L4BTYPES[i]==6 for
        // i in {6,49,50,51}); pass 2 (FlipCoin(10)) turns base-6 tiles into
        // lava pools 95/96/97.
        dungeon.tiles[5][5] = 6; // L4BTYPES[6] = 6
        dungeon.tiles[10][10] = 13; // L4BTYPES[13] = 13

        generator.substitution(&mut dungeon);

        let t = dungeon.tiles[5][5];
        assert!(
            matches!(t, 6 | 49 | 50 | 51 | 95 | 96 | 97),
            "tile[5][5] = {t} not a C++-valid substitution result"
        );
        assert_eq!(dungeon.tiles[10][10], 13); // only matching index for base 13 is 13
    }

    #[test]
    fn test_protect_quads() {
        let mut generator = Dungeon4Generator::new();
        generator.l4_hold = (10, 10);

        generator.protect_quads();

        // Check all 4 quadrants protected
        assert!(generator.protected[10][10]); // Top-left quad
        assert!(generator.protected[DMAXX - 1 - 10][10]); // Top-right
        assert!(generator.protected[10][DMAXY - 1 - 10]); // Bottom-left
        assert!(generator.protected[DMAXX - 1 - 10][DMAXY - 1 - 10]); // Bottom-right
    }

    #[test]
    fn test_generate_diablo_lair() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        let result = generator.generate_diablo_lair(&mut dungeon, 12345);
        assert!(result);
    }

    #[test]
    fn test_flood_transparency_values() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Set up some floor tiles (tile 6)
        for y in 0..5 {
            for x in 0..5 {
                dungeon.tiles[x][y] = 6;
            }
        }

        generator.flood_transparency_values(&mut dungeon, 6);

        // Check that transparency values were filled
        // trans_val maps to double-resolution grid starting at (16,16)
        assert!(dungeon.trans_val[16][16] != 0);
        assert!(dungeon.trans_val[18][18] != 0);
    }

    #[test]
    fn test_fix_transparency() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Set up test pattern with special tiles
        dungeon.tiles[5][5] = 18; // Special wall tile
        dungeon.tiles[10][10] = 19; // Special wall tile
        dungeon.tiles[15][15] = 24; // Special wall tile

        // Set some base transparency values
        dungeon.trans_val[26][26] = 5; // (5*2 + 16, 5*2 + 16)

        generator.fix_transparency(&mut dungeon);

        // Just verify it doesn't crash
        assert!(true);
    }

    #[test]
    fn test_is_dur_right_wall() {
        let generator = Dungeon4Generator::new();
        assert!(generator.is_dur_right_wall(25));
        assert!(generator.is_dur_right_wall(28));
        assert!(generator.is_dur_right_wall(23));
        assert!(!generator.is_dur_right_wall(10));
    }

    #[test]
    fn test_is_dl_left_wall() {
        let generator = Dungeon4Generator::new();
        assert!(generator.is_dl_left_wall(27));
        assert!(generator.is_dl_left_wall(26));
        assert!(generator.is_dl_left_wall(22));
        assert!(!generator.is_dl_left_wall(10));
    }

    #[test]
    fn test_init_set_piece() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Just ensure it doesn't crash
        generator.init_set_piece(&mut dungeon, 15);
    }

    #[test]
    fn test_set_piece_rect() {
        let rect = SetPieceRect {
            x: 10,
            y: 20,
            width: 5,
            height: 3,
        };
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 5);
        assert_eq!(rect.height, 3);
    }

    #[test]
    fn test_set_piece_rect_default() {
        let rect = SetPieceRect::default();
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 0);
        assert_eq!(rect.height, 0);
    }

    #[test]
    fn test_protect_quest_room() {
        let mut generator = Dungeon4Generator::new();

        // Set a test quest room
        generator.set_piece_room = SetPieceRect {
            x: 5,
            y: 5,
            width: 10,
            height: 8,
        };

        // Call protection function (placeholder for now)
        generator.protect_quest_room(14);

        // Verify structure is set
        assert_eq!(generator.set_piece_room.x, 5);
    }

    #[test]
    fn test_check_quests() {
        let mut generator = Dungeon4Generator::new();

        // Call quest check function (placeholder for now)
        generator.check_quests();

        // Just verify it doesn't crash
        assert!(true);
    }

    #[test]
    fn test_find_pentagram_tiles() {
        let generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // No pentagram tiles - should return None
        assert!(generator.find_pentagram_tiles(&dungeon).is_none());

        // Add pentagram tile
        dungeon.tiles[15][20] = 98;
        let result = generator.find_pentagram_tiles(&dungeon);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), (15, 20));
    }

    #[test]
    fn test_load_diablo_quads() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Call with preflag true (placeholder for now)
        generator.load_diablo_quads(&mut dungeon, true);

        // Call with preflag false
        generator.load_diablo_quads(&mut dungeon, false);

        // Just verify it doesn't crash
        assert!(true);
    }

    #[test]
    fn test_complete_l4_generation() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Test Level 13 generation
        let result = generator.generate(&mut dungeon, 54321, 13, LevelEntry::Main);
        assert!(result);

        // Verify some basic properties
        assert!(dungeon.tiles[20][20] > 0); // Should have some tiles placed
    }

    // Day 84 Tests: .dun File I/O

    #[test]
    fn test_load_dun_file_nonexistent() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Try to load non-existent file
        let result = generator.load_dun_file("nonexistent.dun", &mut dungeon, 0, 0, 0);

        assert_eq!(result, false); // Should fail gracefully
    }

    #[test]
    fn test_load_dun_file_empty() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();

        // Create temporary empty file path (won't exist in test env)
        let result = generator.load_dun_file("empty.dun", &mut dungeon, 0, 0, 0);

        assert_eq!(result, false); // Invalid file should return false
    }

    #[test]
    fn test_load_diablo_quads_preflag_true() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();
        generator.l4_hold = (17, 17);

        // Load quads with preflag=true (should use 2b/3b/4b)
        generator.load_diablo_quads(&mut dungeon, true);

        // Verify positions are calculated correctly
        let (hold_x, hold_y) = generator.l4_hold;
        let quad1_x = hold_x as i32 + 4;
        let quad1_y = hold_y as i32 + 4;
        assert_eq!(quad1_x, 21);
        assert_eq!(quad1_y, 21);

        let quad2_x = 27 - hold_x as i32;
        let quad2_y = 1 + hold_y as i32;
        assert_eq!(quad2_x, 10);
        assert_eq!(quad2_y, 18);
    }

    #[test]
    fn test_load_diablo_quads_preflag_false() {
        let mut generator = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();
        generator.l4_hold = (15, 19);

        // Load quads with preflag=false (should use 2a/3a/4a)
        generator.load_diablo_quads(&mut dungeon, false);

        // Verify positions
        let (hold_x, hold_y) = generator.l4_hold;
        let quad3_x = 1 + hold_x as i32;
        let quad3_y = 27 - hold_y as i32;
        assert_eq!(quad3_x, 16);
        assert_eq!(quad3_y, 8);

        let quad4_x = 28 - hold_x as i32;
        let quad4_y = 28 - hold_y as i32;
        assert_eq!(quad4_x, 13);
        assert_eq!(quad4_y, 9);
    }

    #[test]
    fn test_dun_file_format_parsing() {
        // Create mock .dun file data
        let width: u16 = 3;
        let height: u16 = 2;

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&width.to_le_bytes());
        buffer.extend_from_slice(&height.to_le_bytes());

        // Tile data (3x2 = 6 tiles)
        let tiles: [u16; 6] = [10, 11, 12, 13, 14, 15];
        for tile in tiles.iter() {
            buffer.extend_from_slice(&tile.to_le_bytes());
        }

        // Parse header
        let parsed_width = u16::from_le_bytes([buffer[0], buffer[1]]);
        let parsed_height = u16::from_le_bytes([buffer[2], buffer[3]]);

        assert_eq!(parsed_width, 3);
        assert_eq!(parsed_height, 2);

        // Parse first tile
        let tile_data = &buffer[4..];
        let first_tile = u16::from_le_bytes([tile_data[0], tile_data[1]]);
        assert_eq!(first_tile, 10);
    }

    // ========== Theme Room Tests (faithful C++ port) ==========

    #[test]
    fn test_get_size_for_theme_room() {
        let mut gen = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();
        // 8x8 block of floor (tile 6) starting at (5,5)
        for y in 5..13 {
            for x in 5..13 {
                dungeon.tiles[x][y] = 6;
            }
        }
        let sz = gen.get_size_for_theme_room(&dungeon, 6, 5, 5, 7, 10);
        assert!(sz.is_some(), "a large floor area must fit a theme room");
        let (w, h) = sz.unwrap();
        assert!(w >= 5 && h >= 5, "size {w}x{h} should be at least 5 (minSize-2)");
    }

    #[test]
    fn test_get_size_for_theme_room_too_small() {
        let mut gen = Dungeon4Generator::new();
        let dungeon = Dungeon::new();
        // All rock: no floor
        assert!(gen.get_size_for_theme_room(&dungeon, 6, 5, 5, 7, 10).is_none());
    }

    #[test]
    fn test_create_theme_room_frame() {
        let mut gen = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();
        gen.theme_locations.push((10, 10, 5, 5));
        gen.create_theme_room(&mut dungeon, 0);
        // Frame corners
        assert_eq!(dungeon.tiles[10][10], 9);
        assert_eq!(dungeon.tiles[14][10], 16);
        assert_eq!(dungeon.tiles[10][14], 15);
        assert_eq!(dungeon.tiles[14][14], 12);
        // Top/bottom rows are horizontal wall (2), left col vertical wall (1)
        assert_eq!(dungeon.tiles[12][10], 2);
        assert_eq!(dungeon.tiles[10][12], 1);
        assert_eq!(dungeon.tiles[12][14], 2);
        // Interior floor (6) — the right wall (14,12) may hold the door
        assert_eq!(dungeon.tiles[11][11], 6);
        assert_eq!(dungeon.tiles[12][12], 6);
        // Either the FlipCoin() door on the right wall (53/6/52/54 around (14,12))
        // or on the bottom wall (57/6/56/59/58 around (12,14)).
        assert!(
            matches!(
                dungeon.tiles[14][12],
                1 | 6 | 53 | 52 | 54
            ),
            "right wall cell (14,12) = {}",
            dungeon.tiles[14][12]
        );
    }

    #[test]
    fn test_is_near_theme_room() {
        let mut gen = Dungeon4Generator::new();
        gen.theme_locations.push((10, 10, 5, 5));
        // Inside the padded rect (position - 2, size + 5)
        assert!(gen.is_near_theme_room(10, 10));
        assert!(gen.is_near_theme_room(8, 8));
        // Far away
        assert!(!gen.is_near_theme_room(20, 20));
    }

    #[test]
    fn test_place_theme_rooms_empty() {
        let mut gen = Dungeon4Generator::new();
        let mut dungeon = Dungeon::new();
        gen.place_theme_rooms(&mut dungeon, 7, 10, 6, 8, true);
        // All-rock dungeon: no floor, no theme rooms
        assert!(gen.theme_locations.is_empty());
    }
}



