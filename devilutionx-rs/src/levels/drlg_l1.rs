//! Cathedral and Crypt level generation (L1/L5)
//!
//! Implements procedural dungeon generation for Cathedral (DTYPE_CATHEDRAL) and
//! Crypt (DTYPE_CRYPT) levels. Uses room-based layout with 3 chambers connected
//! by corridors.
//!
//! C++ Source: Source/levels/drlg_l1.cpp
//! Translation Date: 2024-XX-XX

use crate::levels::gendung::Dungeon;
use crate::levels::types::{DungeonType, ThemeLocation, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
use crate::utils::random::Rng;

// ===================================================================
// CONSTANTS
// ===================================================================

/// Dungeon array size (matches C++ DMAXX/DMAXY)
pub const DUNGEON_SIZE: usize = 40;

/// Tile size in micro-tiles (2x2 micro-tiles per dungeon tile)
pub const TILE_SIZE: usize = 2;

// ===================================================================
// TILE ENUM
// ===================================================================

/// Cathedral dungeon tiles (C++ enum Tile in drlg_l1.cpp)
///
/// Represents different tile types used during Cathedral generation.
/// These are logical tiles that get converted to visual tiles (Floor/Dirt)
/// during the generation process.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Tile {
    /// C++ miniset wildcard / empty cell (0)
    Invalid = 0,
    /// Vertical wall (left side)
    VWall = 1,
    /// Horizontal wall (top side)
    HWall = 2,
    /// C++ `Corner = 3` (wall corner; used by AddWall/FixTilesPatterns)
    Corner = 3,
    /// C++ `DWall = 4` (diagonal wall written by MakeDmt)
    SECorner = 4,
    /// C++ `DArch = 5` (door arch)
    DArch = 5,
    /// C++ `VWallEnd = 6` (vertical wall end)
    VWallEnd = 6,
    /// C++ `HWallEnd = 7` (horizontal wall end)
    HWallEnd = 7,
    /// C++ `HArchEnd = 8` (horizontal arch end)
    HArchEnd = 8,
    /// C++ `VArchEnd = 9` (vertical arch end)
    VArchEnd = 9,
    /// C++ `HArchVWall = 10` (horizontal arch + vertical wall)
    HArchVWall = 10,
    /// C++ `VArch = 11` (vertical arch)
    ArchV1 = 11,
    /// C++ `HArch = 12` (horizontal arch)
    ArchH1 = 12,
    /// Floor tile
    Floor = 13,
    /// C++ `HWallVArch = 14` (horizontal wall + vertical arch)
    HWallVArch = 14,
    /// C++ `Pillar = 15` (room pillar)
    Pillar = 15,
    /// C++ `VCorner = 16` (corner written by MakeDmt)
    NWCorner = 16,
    /// C++ `HCorner = 17` (horizontal corner)
    HCorner = 17,
    /// C++ `DirtHwall = 18` (dirt horizontal wall)
    DirtHwall = 18,
    /// C++ `DirtVwall = 19` (dirt vertical wall)
    DirtVwall = 19,
    /// C++ `VDirtCorner = 20` (vertical dirt corner)
    VDirtCorner = 20,
    /// C++ `HDirtCorner = 21` (horizontal dirt corner)
    HDirtCorner = 21,
    /// C++ `Dirt = 22` (the Cathedral background tile)
    Dirt = 22,
    /// C++ `DirtHwallEnd = 23` (dirt horizontal wall end)
    DirtHwallEnd = 23,
    /// C++ `DirtVwallEnd = 24` (dirt vertical wall end)
    DirtVwallEnd = 24,
    /// C++ `VDoor = 25` (vertical wall with door)
    VWallDoor = 25,
    /// C++ `HDoor = 26` (horizontal wall with door)
    HWallDoor = 26,
    /// C++ `HFenceVWall = 27` (horizontal fence + vertical wall)
    HFenceVWall = 27,
    /// C++ `HDoorVDoor = 28` (horizontal door + vertical door)
    HDoorVDoor = 28,
    /// C++ `DFence = 29` (diagonal fence)
    DFence = 29,
    /// C++ `VDoorEnd = 30` (vertical door end)
    VDoorEnd = 30,
    /// C++ `HDoorEnd = 31` (horizontal door end)
    HDoorEnd = 31,
    /// C++ `VFenceEnd = 32` (vertical fence end)
    VFenceEnd = 32,
    /// C++ `VArchEnd2 = 33`
    VArchEnd2 = 33,
    /// C++ `HArchVWall2 = 34`
    HArchVWall2 = 34,
    /// C++ `VFence = 35` (vertical fence)
    VFence = 35,
    /// C++ `HFence = 36` (horizontal fence)
    HFence = 36,
    /// C++ `HWallVFence = 37` (horizontal wall + vertical fence)
    HWallVFence = 37,
    /// C++ `HArchVFence = 38` (horizontal arch + vertical fence)
    HArchVFence = 38,
    /// C++ `HArchVDoor = 39` (horizontal arch + vertical door)
    HArchVDoor = 39,
    /// C++ `HArchVWall3 = 40`
    HArchVWall3 = 40,
    /// C++ `DWall2 = 41`
    DWall2 = 41,
    /// C++ `HWallVArch2 = 42`
    HWallVArch2 = 42,
    /// C++ `DWall3 = 43`
    DWall3 = 43,
    /// Lava pool (crypt only)
    Lava = 56,
    /// Raw l1.til mega-tiles used by the stairs minisets (C++ STAIRSUP/STAIRSDOWN)
    StairTile57 = 57,
    StairTile58 = 58,
    StairTile59 = 59,
    StairTile60 = 60,
    StairTile61 = 61,
    StairTile62 = 62,
    StairTile63 = 63,
    /// C++ `EntranceStairs = 64`
    EntranceStairs = 64,
    StairTile65 = 65,
    StairTile66 = 66,
    StairTile67 = 67,
    StairTile68 = 68,
    /// C++ `VWall2 = 79`
    VWall2 = 79,
    /// C++ `HWall2 = 80`
    HWall2 = 80,
    /// C++ `DWall4 = 82`
    DWall4 = 82,
    /// C++ `VWallEnd2 = 84`
    VWallEnd2 = 84,
    /// C++ `VWall4 = 89`
    VWall4 = 89,
    /// C++ `VWall5 = 90`
    VWall5 = 90,
    /// C++ `HWall4 = 91`
    HWall4 = 91,
    /// C++ `HWall5 = 92`
    HWall5 = 92,
    /// C++ `VWall8 = 100`
    VWall8 = 100,
    /// Raw l1.til mega-tiles that Substitution can write (TileDecorations non-zero)
    Mega94 = 94,
    Mega95 = 95,
    Mega97 = 97,
    Mega98 = 98,
    Mega99 = 99,
    Mega101 = 101,
    Mega102 = 102,
    Mega103 = 103,
    Mega104 = 104,
    Mega105 = 105,
    Mega106 = 106,
    Mega107 = 107,
    Mega108 = 108,
    Mega109 = 109,
    Mega110 = 110,
    Mega111 = 111,
    Mega112 = 112,
    Mega113 = 113,
    Mega114 = 114,
    Mega115 = 115,
    Mega116 = 116,
    Mega117 = 117,
    Mega118 = 118,
    Mega121 = 121,
    Mega122 = 122,
    Mega123 = 123,
    Mega124 = 124,
    Mega125 = 125,
    Mega133 = 133,
    Mega136 = 136,
    Mega137 = 137,
    Mega138 = 138,
    /// Raw l1.til mega-tiles used by LAMPS/PWATERIN minisets
    Mega81 = 81,
    Mega83 = 83,
    Mega85 = 85,
    Mega128 = 128,
    Mega129 = 129,
    Mega130 = 130,
    Mega134 = 134,
    Mega135 = 135,
    Mega203 = 203,
    Mega206 = 206,
    /// C++ `Floor12 = 139` .. `Floor23 = 163` (floor/shadow variations)
    Floor12 = 139,
    Floor13 = 140,
    Floor14 = 141,
    Floor15 = 142,
    Floor16 = 143,
    Floor17 = 144,
    Floor18 = 145,
    VWall17 = 146,
    VArch5 = 147,
    HWallShadow = 148,
    HArchShadow = 149,
    Floor19 = 150,
    Floor20 = 151,
    Floor21 = 152,
    HArchShadow2 = 153,
    HWallShadow2 = 154,
    Floor22 = 162,
    Floor23 = 163,
    /// C++ `DirtHWall2 = 199` .. `DirtVWallEnd2 = 205`
    DirtHWall2 = 199,
    DirtVWall2 = 200,
    DirtCorner2 = 202,
    DirtHWallEnd2 = 204,
    DirtVWallEnd2 = 205,
    // --- revalued self-invented / crypt-only variants (not used by L1) ---
    NECorner = 210,
    SWCorner = 211,
    ArchH2 = 212,
    ArchV2 = 213,
    DirtVWallToSouth = 214,
    DirtVWallToNorth = 215,
    DirtHWallToEast = 216,
    DirtHWallToWest = 217,
    DirtNWCorner = 218,
    DirtNECorner = 219,
    DirtSWCorner = 220,
    DirtSECorner = 221,
    DirtCross = 222,
    DirtHWall = 223,
    DirtVWall = 224,
    CryptArchH1 = 225,
    CryptArchH2 = 226,
    CryptArchV1 = 227,
    CryptArchV2 = 228,
    CryptHWallDoor = 229,
    CryptVWallDoor = 230,
    CryptNWCorner = 231,
    CryptNECorner = 232,
    CryptSWCorner = 233,
    CryptSECorner = 234,
    CryptHWall = 235,
    CryptVWall = 236,
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Invalid
    }
}

impl From<Tile> for u8 {
    fn from(tile: Tile) -> u8 {
        tile as u8
    }
}

impl TryFrom<u8> for Tile {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Tile::Invalid),
            1 => Ok(Tile::VWall),
            2 => Ok(Tile::HWall),
            3 => Ok(Tile::Corner),
            4 => Ok(Tile::SECorner),
            5 => Ok(Tile::DArch),
            6 => Ok(Tile::VWallEnd),
            7 => Ok(Tile::HWallEnd),
            8 => Ok(Tile::HArchEnd),
            9 => Ok(Tile::VArchEnd),
            10 => Ok(Tile::HArchVWall),
            11 => Ok(Tile::ArchV1),
            12 => Ok(Tile::ArchH1),
            13 => Ok(Tile::Floor),
            14 => Ok(Tile::HWallVArch),
            15 => Ok(Tile::Pillar),
            16 => Ok(Tile::NWCorner),
            17 => Ok(Tile::HCorner),
            18 => Ok(Tile::DirtHwall),
            19 => Ok(Tile::DirtVwall),
            20 => Ok(Tile::VDirtCorner),
            21 => Ok(Tile::HDirtCorner),
            22 => Ok(Tile::Dirt),
            23 => Ok(Tile::DirtHwallEnd),
            24 => Ok(Tile::DirtVwallEnd),
            25 => Ok(Tile::VWallDoor),
            26 => Ok(Tile::HWallDoor),
            27 => Ok(Tile::HFenceVWall),
            28 => Ok(Tile::HDoorVDoor),
            29 => Ok(Tile::DFence),
            30 => Ok(Tile::VDoorEnd),
            31 => Ok(Tile::HDoorEnd),
            32 => Ok(Tile::VFenceEnd),
            33 => Ok(Tile::VArchEnd2),
            34 => Ok(Tile::HArchVWall2),
            35 => Ok(Tile::VFence),
            36 => Ok(Tile::HFence),
            37 => Ok(Tile::HWallVFence),
            38 => Ok(Tile::HArchVFence),
            39 => Ok(Tile::HArchVDoor),
            40 => Ok(Tile::HArchVWall3),
            41 => Ok(Tile::DWall2),
            42 => Ok(Tile::HWallVArch2),
            43 => Ok(Tile::DWall3),
            56 => Ok(Tile::Lava),
            57 => Ok(Tile::StairTile57),
            58 => Ok(Tile::StairTile58),
            59 => Ok(Tile::StairTile59),
            60 => Ok(Tile::StairTile60),
            61 => Ok(Tile::StairTile61),
            62 => Ok(Tile::StairTile62),
            63 => Ok(Tile::StairTile63),
            64 => Ok(Tile::EntranceStairs),
            65 => Ok(Tile::StairTile65),
            66 => Ok(Tile::StairTile66),
            67 => Ok(Tile::StairTile67),
            68 => Ok(Tile::StairTile68),
            79 => Ok(Tile::VWall2),
            80 => Ok(Tile::HWall2),
            81 => Ok(Tile::Mega81),
            82 => Ok(Tile::DWall4),
            83 => Ok(Tile::Mega83),
            84 => Ok(Tile::VWallEnd2),
            85 => Ok(Tile::Mega85),
            89 => Ok(Tile::VWall4),
            90 => Ok(Tile::VWall5),
            91 => Ok(Tile::HWall4),
            92 => Ok(Tile::HWall5),
            100 => Ok(Tile::VWall8),
            128 => Ok(Tile::Mega128),
            129 => Ok(Tile::Mega129),
            130 => Ok(Tile::Mega130),
            134 => Ok(Tile::Mega134),
            135 => Ok(Tile::Mega135),
            94 => Ok(Tile::Mega94),
            95 => Ok(Tile::Mega95),
            97 => Ok(Tile::Mega97),
            98 => Ok(Tile::Mega98),
            99 => Ok(Tile::Mega99),
            101 => Ok(Tile::Mega101),
            102 => Ok(Tile::Mega102),
            103 => Ok(Tile::Mega103),
            104 => Ok(Tile::Mega104),
            105 => Ok(Tile::Mega105),
            106 => Ok(Tile::Mega106),
            107 => Ok(Tile::Mega107),
            108 => Ok(Tile::Mega108),
            109 => Ok(Tile::Mega109),
            110 => Ok(Tile::Mega110),
            111 => Ok(Tile::Mega111),
            112 => Ok(Tile::Mega112),
            113 => Ok(Tile::Mega113),
            114 => Ok(Tile::Mega114),
            115 => Ok(Tile::Mega115),
            116 => Ok(Tile::Mega116),
            117 => Ok(Tile::Mega117),
            118 => Ok(Tile::Mega118),
            121 => Ok(Tile::Mega121),
            122 => Ok(Tile::Mega122),
            123 => Ok(Tile::Mega123),
            124 => Ok(Tile::Mega124),
            125 => Ok(Tile::Mega125),
            133 => Ok(Tile::Mega133),
            136 => Ok(Tile::Mega136),
            137 => Ok(Tile::Mega137),
            138 => Ok(Tile::Mega138),
            139 => Ok(Tile::Floor12),
            140 => Ok(Tile::Floor13),
            141 => Ok(Tile::Floor14),
            142 => Ok(Tile::Floor15),
            143 => Ok(Tile::Floor16),
            144 => Ok(Tile::Floor17),
            145 => Ok(Tile::Floor18),
            146 => Ok(Tile::VWall17),
            147 => Ok(Tile::VArch5),
            148 => Ok(Tile::HWallShadow),
            149 => Ok(Tile::HArchShadow),
            150 => Ok(Tile::Floor19),
            151 => Ok(Tile::Floor20),
            152 => Ok(Tile::Floor21),
            153 => Ok(Tile::HArchShadow2),
            154 => Ok(Tile::HWallShadow2),
            162 => Ok(Tile::Floor22),
            163 => Ok(Tile::Floor23),
            199 => Ok(Tile::DirtHWall2),
            200 => Ok(Tile::DirtVWall2),
            203 => Ok(Tile::Mega203),
            202 => Ok(Tile::DirtCorner2),
            204 => Ok(Tile::DirtHWallEnd2),
            205 => Ok(Tile::DirtVWallEnd2),
            206 => Ok(Tile::Mega206),
            _ => Err(()),
        }
    }
}

// ===================================================================
// MINISET STRUCTURE
// ===================================================================

/// Miniset pattern for dungeon decoration
///
/// C++ equivalent: struct Miniset in drlg_l1.cpp
/// Minisets are small tile patterns (e.g., stairs, lamps) that get placed
/// into the dungeon matching specific tile configurations.
#[derive(Debug, Clone)]
pub struct Miniset {
    /// Width of the miniset
    pub width: usize,
    /// Height of the miniset
    pub height: usize,
    /// Search pattern (what tiles to match)
    pub search: Vec<Vec<Tile>>,
    /// Replacement pattern (what tiles to place)
    pub replace: Vec<Vec<Tile>>,
}

impl Miniset {
    /// Check if miniset matches at position (x, y) in dungeon
    /// Check if miniset matches at position (x, y) in dungeon
    ///
    /// Mirrors C++ `Miniset::matches()`: every cell must match `search`
    /// (0/Invalid = wildcard) and none may be `protected`.
    pub fn matches(
        &self,
        dungeon: &[[Tile; DUNGEON_SIZE]; DUNGEON_SIZE],
        protected: &[[bool; DUNGEON_SIZE]; DUNGEON_SIZE],
        x: usize,
        y: usize,
    ) -> bool {
        if x + self.width > DUNGEON_SIZE || y + self.height > DUNGEON_SIZE {
            return false;
        }

        for dy in 0..self.height {
            for dx in 0..self.width {
                let search_tile = self.search[dy][dx];
                if search_tile != Tile::Invalid && dungeon[y + dy][x + dx] != search_tile {
                    return false;
                }
                if protected[y + dy][x + dx] {
                    return false;
                }
            }
        }
        true
    }

    /// Place miniset at position (x, y) in dungeon
    pub fn place(&self, dungeon: &mut [[Tile; DUNGEON_SIZE]; DUNGEON_SIZE], x: usize, y: usize) {
        for dy in 0..self.height {
            for dx in 0..self.width {
                let replace_tile = self.replace[dy][dx];
                if replace_tile != Tile::Invalid {
                    dungeon[y + dy][x + dx] = replace_tile;
                }
            }
        }
    }
}

// ===================================================================
// MINISET DEFINITIONS
// ===================================================================

/// Upward stairs miniset (4x4)
///
/// C++ source: STAIRSUP in drlg_l1.cpp:36-43
pub fn stairs_up_miniset() -> Miniset {
    use Tile::*;
    Miniset {
        width: 4,
        height: 4,
        search: vec![
            vec![Floor, Floor, Floor, Floor],
            vec![HWall, HWall, HWall, HWall],
            vec![Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor],
        ],
        replace: vec![
            vec![Invalid, StairTile66, VWallEnd, Invalid],
            vec![StairTile63, EntranceStairs, StairTile65, Invalid],
            vec![Invalid, StairTile67, StairTile68, Invalid],
            vec![Invalid, Invalid, Invalid, Invalid],
        ],
    }
}

/// Original-cathedral upward stairs miniset (4x4).
///
/// C++ source: L5STAIRSUP in crypt.cpp:19-33 — used when the legacy
/// `pOriginalCathedral` option is on (matches the C++ test fixtures).
pub fn l5_stairs_up_miniset() -> Miniset {
    use Tile::*;
    Miniset {
        width: 4,
        height: 4,
        search: vec![
            vec![Dirt, Dirt, Dirt, Dirt],
            vec![HWall, HWall, HWall, HWall],
            vec![Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor],
        ],
        replace: vec![
            vec![Invalid, StairTile66, DirtHwallEnd, Invalid],
            vec![StairTile63, EntranceStairs, StairTile65, Invalid],
            vec![Invalid, StairTile67, StairTile68, Invalid],
            vec![Invalid, Invalid, Invalid, Invalid],
        ],
    }
}

/// Downward stairs miniset (4x3)
///
/// C++ source: STAIRSDOWN in drlg_l1.cpp:50-56
pub fn stairs_down_miniset() -> Miniset {
    use Tile::*;
    Miniset {
        width: 4,
        height: 3,
        search: vec![
            vec![Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor],
        ],
        replace: vec![
            vec![StairTile62, StairTile57, StairTile58, Invalid],
            vec![StairTile61, StairTile59, StairTile60, Invalid],
            vec![Invalid, Invalid, Invalid, Invalid],
        ],
    }
}

/// Lamp miniset (2x2)
///
/// C++ source: LAMPS in drlg_l1.cpp:63-68
pub fn lamps_miniset() -> Miniset {
    use Tile::*;
    Miniset {
        width: 2,
        height: 2,
        search: vec![
            vec![Floor, Invalid],
            vec![Floor, Floor],
        ],
        replace: vec![
            vec![Mega129, Invalid],
            vec![Mega130, Mega128],
        ],
    }
}

/// Water pool entrance miniset (6x6)
///
/// C++ source: PWATERIN in drlg_l1.cpp:75-87
pub fn water_in_miniset() -> Miniset {
    use Tile::*;
    Miniset {
        width: 6,
        height: 6,
        search: vec![
            vec![Floor, Floor, Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor, Floor, Floor],
        ],
        replace: vec![
            vec![Invalid, Invalid, Invalid, Invalid, Invalid, Invalid],
            vec![Invalid, DirtCorner2, DirtVWall2, DirtVWall2, VWallEnd2, Invalid],
            vec![Invalid, DirtHWall2, Mega203, Mega203, Mega83, Invalid],
            vec![Invalid, Mega85, Mega206, HWall2, Mega81, Invalid],
            vec![Invalid, Invalid, Mega134, Mega135, Invalid, Invalid],
            vec![Invalid, Invalid, Invalid, Invalid, Invalid, Invalid],
        ],
    }
}

// ===================================================================
// CHAMBER STATE
// ===================================================================

/// Chamber availability state
///
/// C++ equivalent: HasChamber1/HasChamber2/HasChamber3 globals
/// Tracks which chambers (1=top/left, 2=center, 3=bottom/right) exist
/// in the current dungeon layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChamberState {
    pub has_chamber_1: bool,
    pub has_chamber_2: bool,
    pub has_chamber_3: bool,
}

impl Default for ChamberState {
    fn default() -> Self {
        ChamberState {
            has_chamber_1: false,
            has_chamber_2: false,
            has_chamber_3: false,
        }
    }
}

impl ChamberState {
    /// Select random chamber position based on availability
    ///
    /// C++ source: SelectChamber() in drlg_l1.cpp:1263-1292
    ///
    /// Returns (x, y) position for chamber center in dungeon coordinates.
    /// Layout depends on vertical_layout flag:
    /// - Vertical: Chamber 1=(16,2), 2=(16,16), 3=(16,30)
    /// - Horizontal: Chamber 1=(2,16), 2=(16,16), 3=(30,16)
    pub fn select_chamber(&self, vertical_layout: bool, rng: &mut Rng) -> (usize, usize) {
        let chamber = if self.has_chamber_1 && self.has_chamber_2 && self.has_chamber_3 {
            // All 3 chambers available: pick randomly — C++ GenerateRnd(3) + 1
            rng.random_less_than(3) as usize + 1
        } else if self.has_chamber_1 && self.has_chamber_2 {
            // Chambers 1 & 2: reverse order to match vanilla — PickRandomlyAmong({2,1})
            if rng.generate(2) == 0 { 2 } else { 1 }
        } else if self.has_chamber_1 && self.has_chamber_3 {
            // Chambers 1 & 3: reverse order to match vanilla — PickRandomlyAmong({3,1})
            if rng.generate(2) == 0 { 3 } else { 1 }
        } else if self.has_chamber_2 && self.has_chamber_3 {
            // Chambers 2 & 3 — PickRandomlyAmong({2,3})
            if rng.generate(2) == 0 { 2 } else { 3 }
        } else {
            // Default to chamber 2 (always available if only 1 chamber exists)
            2
        };

        match chamber {
            1 => if vertical_layout { (16, 2) } else { (2, 16) },
            3 => if vertical_layout { (16, 30) } else { (30, 16) },
            _ => (16, 16), // Chamber 2 (center)
        }
    }
}

// ===================================================================
// RECTANGLE HELPER
// ===================================================================

/// Rectangle for room placement
///
/// C++ equivalent: Rectangle struct in drlg_l1.cpp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rectangle { x, y, width, height }
    }
}

// ===================================================================
// DUNGEON GENERATION STATE
// ===================================================================

/// Cathedral/Crypt level generator state
///
/// Encapsulates all mutable state for level generation.
/// C++ uses global variables; Rust encapsulates them here.
pub struct CathedralGenerator {
    /// Dungeon tile array (40x40 logical tiles)
    pub dungeon: [[Tile; DUNGEON_SIZE]; DUNGEON_SIZE],

    /// Dungeon mask (tracks which tiles are part of rooms)
    /// C++ equivalent: DungeonMask bitset
    pub dungeon_mask: [[bool; DUNGEON_SIZE]; DUNGEON_SIZE],

    /// Protected tiles (cannot be modified)
    /// C++ equivalent: Protected bitset
    pub protected: [[bool; DUNGEON_SIZE]; DUNGEON_SIZE],

    /// Chamber tiles (marks chamber areas)
    /// C++ equivalent: Chamber bitset
    pub chamber: [[bool; DUNGEON_SIZE]; DUNGEON_SIZE],

    /// Chamber availability state
    pub chamber_state: ChamberState,

    /// Vertical layout flag (true=vertical, false=horizontal)
    pub vertical_layout: bool,

    /// Theme locations (for special rooms)
    pub themes: Vec<ThemeLocation>,

    /// Legacy original-cathedral stairs (`pOriginalCathedral`): when true the
    /// C++ engine places L5STAIRSUP instead of STAIRSUP. Defaults to true to
    /// match the C++ test fixtures (TestInitGame sets it true).
    pub original_cathedral: bool,

    /// Seeded RNG (matches Diablo's `SetRndSeed`/`GenerateRnd`/`FlipCoin`).
    ///
    /// All randomness in Cathedral generation flows through this `Rng`, seeded
    /// once at the start of [`generate`](Self::generate). This makes level
    /// generation bit-for-bit reproducible for a given seed, matching the C++
    /// engine (`Source/levels/drlg_l1.cpp` calls `SetRndSeed` then `GenerateRnd`/
    /// `FlipCoin` throughout).
    rng: Rng,
}impl Default for CathedralGenerator {
    fn default() -> Self {
        CathedralGenerator {
            dungeon: [[Tile::Invalid; DUNGEON_SIZE]; DUNGEON_SIZE],
            dungeon_mask: [[false; DUNGEON_SIZE]; DUNGEON_SIZE],
            protected: [[false; DUNGEON_SIZE]; DUNGEON_SIZE],
            chamber: [[false; DUNGEON_SIZE]; DUNGEON_SIZE],
            chamber_state: ChamberState::default(),
            vertical_layout: false,
            themes: Vec::new(),
            rng: Rng::with_default_seed(),
            original_cathedral: true,
        }
    }
}

impl CathedralGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize dungeon array with Dirt tiles
    ///
    /// C++ source: InitDungeon() in drlg_l1.cpp
    pub fn init_dungeon(&mut self) {
        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                self.dungeon[y][x] = Tile::Dirt;
            }
        }
        // C++ InitDungeonFlags() only clears dungeon + Protected/Chamber;
        // DungeonMask is reset by FirstRoom() and must survive into MakeDmt().
        self.protected = [[false; DUNGEON_SIZE]; DUNGEON_SIZE];
        self.chamber = [[false; DUNGEON_SIZE]; DUNGEON_SIZE];
    }

    /// Map room area to dungeon mask
    ///
    /// C++ source: MapRoom() in drlg_l1.cpp:437-443
    fn map_room(&mut self, room: Rectangle) {
        for y in 0..room.height {
            for x in 0..room.width {
                let px = (room.x + x) as usize;
                let py = (room.y + y) as usize;
                if px < DUNGEON_SIZE && py < DUNGEON_SIZE {
                    self.dungeon_mask[py][px] = true;
                }
            }
        }
    }

    /// Check if room fits without overlapping
    ///
    /// C++ source: CheckRoom() in drlg_l1.cpp:445-459
    fn check_room(&self, room: Rectangle) -> bool {
        for y in 0..room.height {
            for x in 0..room.width {
                let px = room.x + x;
                let py = room.y + y;
                if px < 0 || px >= DUNGEON_SIZE as i32 || py < 0 || py >= DUNGEON_SIZE as i32 {
                    return false;
                }
                if self.dungeon_mask[py as usize][px as usize] {
                    return false;
                }
            }
        }
        true
    }

    /// Generate recursive room subdivision
    ///
    /// C++ source: GenerateRoom() in drlg_l1.cpp:461-503
    fn generate_room(&mut self, area: Rectangle, vertical_layout: bool) {
        // Randomly flip layout orientation — C++ FlipCoin(4)
        let rotate = self.rng.generate(4) == 0;
        let vertical_layout = (!vertical_layout && rotate) || (vertical_layout && !rotate);

        let mut room1 = Rectangle::new(0, 0, 0, 0);
        let mut place_room1 = false;

        // Try to place first room (20 attempts)
        for _ in 0..20 {
            let random_width = ((self.rng.generate(5) + 2) & !1) as i32; // Even width
            let random_height = ((self.rng.generate(5) + 2) & !1) as i32; // Even height

            room1.width = random_width;
            room1.height = random_height;
            room1.x = area.x;
            room1.y = area.y;

            if vertical_layout {
                room1.x -= room1.width;
                room1.y += area.height / 2 - room1.height / 2;
                // C++ bug: swaps width/height in bounds check (we replicate bug for compatibility)
                let check_rect = Rectangle::new(
                    room1.x - 1,
                    room1.y - 1,
                    room1.height + 2, // Intentional swap
                    room1.width + 1,
                );
                place_room1 = self.check_room(check_rect);
            } else {
                room1.x += area.width / 2 - room1.width / 2;
                room1.y -= room1.height;
                let check_rect = Rectangle::new(
                    room1.x - 1,
                    room1.y - 1,
                    room1.width + 2,
                    room1.height + 1,
                );
                place_room1 = self.check_room(check_rect);
            }

            if place_room1 {
                break;
            }
        }

        // Place room1 if valid
        if place_room1 {
            let clamped_width = (DUNGEON_SIZE as i32 - room1.x).min(room1.width);
            let clamped_height = (DUNGEON_SIZE as i32 - room1.y).min(room1.height);
            self.map_room(Rectangle::new(room1.x, room1.y, clamped_width, clamped_height));
        }

        // Try to place room2 (mirror of room1)
        let mut room2 = room1;
        let place_room2;

        if vertical_layout {
            room2.x = area.x + area.width;
            let check_rect = Rectangle::new(
                room2.x,
                room2.y - 1,
                room2.width + 1,
                room2.height + 2,
            );
            place_room2 = self.check_room(check_rect);
        } else {
            room2.y = area.y + area.height;
            let check_rect = Rectangle::new(
                room2.x - 1,
                room2.y,
                room2.width + 2,
                room2.height + 1,
            );
            place_room2 = self.check_room(check_rect);
        }

        if place_room2 {
            self.map_room(room2);
        }

        // Recursively subdivide rooms
        if place_room1 {
            self.generate_room(room1, !vertical_layout);
        }
        if place_room2 {
            self.generate_room(room2, !vertical_layout);
        }
    }

    /// Generate initial chamber layout
    ///
    /// C++ source: FirstRoom() in drlg_l1.cpp:508-550
    pub fn first_room(&mut self) {
        // Reset dungeon mask
        self.dungeon_mask = [[false; DUNGEON_SIZE]; DUNGEON_SIZE];

        // Randomly determine layout — C++ FlipCoin() / !FlipCoin()
        self.vertical_layout = self.rng.generate(2) == 0;
        self.chamber_state.has_chamber_1 = self.rng.generate(2) != 0;
        self.chamber_state.has_chamber_2 = self.rng.generate(2) != 0;
        self.chamber_state.has_chamber_3 = self.rng.generate(2) != 0;

        // Ensure at least 2 chambers (center always exists if only 1 chamber)
        if !self.chamber_state.has_chamber_1 || !self.chamber_state.has_chamber_3 {
            self.chamber_state.has_chamber_2 = true;
        }

        // Define chamber positions
        let mut chamber1 = Rectangle::new(1, 15, 10, 10);
        let chamber2 = Rectangle::new(15, 15, 10, 10);
        let mut chamber3 = Rectangle::new(29, 15, 10, 10);
        let mut hallway = Rectangle::new(1, 17, 38, 6);

        // Adjust hallway based on chamber availability
        if !self.chamber_state.has_chamber_1 {
            hallway.x += 17;
            hallway.width -= 17;
        }
        if !self.chamber_state.has_chamber_3 {
            hallway.width -= 16;
        }

        // Swap coordinates for vertical layout
        if self.vertical_layout {
            std::mem::swap(&mut chamber1.x, &mut chamber1.y);
            std::mem::swap(&mut chamber3.x, &mut chamber3.y);
            std::mem::swap(&mut hallway.x, &mut hallway.y);
            std::mem::swap(&mut hallway.width, &mut hallway.height);
        }

        // Map chambers and hallway
        if self.chamber_state.has_chamber_1 {
            self.map_room(chamber1);
        }
        if self.chamber_state.has_chamber_2 {
            self.map_room(chamber2);
        }
        if self.chamber_state.has_chamber_3 {
            self.map_room(chamber3);
        }
        self.map_room(hallway);

        // Recursively generate rooms within chambers
        if self.chamber_state.has_chamber_1 {
            self.generate_room(chamber1, self.vertical_layout);
        }
        if self.chamber_state.has_chamber_2 {
            self.generate_room(chamber2, self.vertical_layout);
        }
        if self.chamber_state.has_chamber_3 {
            self.generate_room(chamber3, self.vertical_layout);
        }
    }

    /// Count number of tiles in dungeon mask
    ///
    /// C++ source: FindArea() in drlg_l1.cpp:555-558
    pub fn find_area(&self) -> usize {
        let mut count = 0;
        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                if self.dungeon_mask[y][x] {
                    count += 1;
                }
            }
        }
        count
    }

    /// C++ `GenerateChamber()` (drlg_l1.cpp): fills a 10x10 chamber interior
    /// with floor, marks the Chamber mask, places pillars, and connects to
    /// neighbouring chambers with arch/door tiles.
    fn generate_chamber(
        &mut self,
        position_x: i32,
        position_y: i32,
        connect_previous: bool,
        connect_next: bool,
        vertical_layout: bool,
    ) {
        let (px, py) = (position_x as usize, position_y as usize);
        if connect_previous {
            if vertical_layout {
                self.dungeon[py][px + 2] = Tile::ArchH1;
                self.dungeon[py][px + 3] = Tile::ArchH1;
                self.dungeon[py][px + 4] = Tile::Corner;
                self.dungeon[py][px + 7] = Tile::VArchEnd;
                self.dungeon[py][px + 8] = Tile::ArchH1;
                self.dungeon[py][px + 9] = Tile::HWall;
            } else {
                self.dungeon[py + 2][px] = Tile::ArchV1;
                self.dungeon[py + 3][px] = Tile::ArchV1;
                self.dungeon[py + 4][px] = Tile::Corner;
                self.dungeon[py + 7][px] = Tile::HArchEnd;
                self.dungeon[py + 8][px] = Tile::ArchV1;
                self.dungeon[py + 9][px] = Tile::VWall;
            }
        }
        if connect_next {
            if vertical_layout {
                let y = py + 11;
                self.dungeon[y][px + 2] = Tile::HArchVWall;
                self.dungeon[y][px + 3] = Tile::ArchH1;
                self.dungeon[y][px + 4] = Tile::HArchEnd;
                self.dungeon[y][px + 7] = Tile::DArch;
                self.dungeon[y][px + 8] = Tile::ArchH1;
                if self.dungeon[y][px + 9] != Tile::SECorner {
                    self.dungeon[y][px + 9] = Tile::HDirtCorner;
                }
            } else {
                let x = px + 11;
                self.dungeon[py + 2][x] = Tile::HWallVArch;
                self.dungeon[py + 3][x] = Tile::ArchV1;
                self.dungeon[py + 4][x] = Tile::VArchEnd;
                self.dungeon[py + 7][x] = Tile::DArch;
                self.dungeon[py + 8][x] = Tile::ArchV1;
                if self.dungeon[py + 9][x] != Tile::SECorner {
                    self.dungeon[py + 9][x] = Tile::HDirtCorner;
                }
            }
        }
        for y in 1..11 {
            for x in 1..11 {
                self.dungeon[py + y][px + x] = Tile::Floor;
                self.chamber[py + y][px + x] = true;
            }
        }
        self.dungeon[py + 4][px + 4] = Tile::Pillar;
        self.dungeon[py + 4][px + 7] = Tile::Pillar;
        self.dungeon[py + 7][px + 4] = Tile::Pillar;
        self.dungeon[py + 7][px + 7] = Tile::Pillar;
    }

    /// C++ `GenerateHall()` (drlg_l1.cpp): draws the arch rows connecting
    /// chambers through the hallway.
    fn generate_hall(&mut self, start_x: i32, start_y: i32, length: i32, vertical_layout: bool) {
        if vertical_layout {
            for i in start_y..(start_y + length) {
                self.dungeon[i as usize][start_x as usize] = Tile::ArchV1;
                self.dungeon[i as usize][start_x as usize + 3] = Tile::ArchV1;
            }
        } else {
            for i in start_x..(start_x + length) {
                self.dungeon[start_y as usize][i as usize] = Tile::ArchH1;
                self.dungeon[start_y as usize + 3][i as usize] = Tile::ArchH1;
            }
        }
    }

    /// C++ `FillChambers()` (drlg_l1.cpp): builds the chamber walls/arches on
    /// top of the MakeDmt output so `AddWall` can complete the dungeon. Quest
    /// set pieces (`InitSetPiece`) are skipped — no quest wiring and the
    /// set-piece .dun data is not loaded.
    fn fill_chambers(&mut self) {
        let (mut chamber1x, mut chamber1y) = (0i32, 14i32);
        let (mut chamber3x, mut chamber3y) = (28i32, 14i32);
        let (mut hall1x, mut hall1y) = (12i32, 18i32);
        let (mut hall2x, mut hall2y) = (26i32, 18i32);
        if self.vertical_layout {
            std::mem::swap(&mut chamber1x, &mut chamber1y);
            std::mem::swap(&mut chamber3x, &mut chamber3y);
            std::mem::swap(&mut hall1x, &mut hall1y);
            std::mem::swap(&mut hall2x, &mut hall2y);
        }

        if self.chamber_state.has_chamber_1 {
            self.generate_chamber(chamber1x, chamber1y, false, true, self.vertical_layout);
        }
        if self.chamber_state.has_chamber_2 {
            self.generate_chamber(
                14,
                14,
                self.chamber_state.has_chamber_1,
                self.chamber_state.has_chamber_3,
                self.vertical_layout,
            );
        }
        if self.chamber_state.has_chamber_3 {
            self.generate_chamber(chamber3x, chamber3y, true, false, self.vertical_layout);
        }

        if self.chamber_state.has_chamber_2 {
            if self.chamber_state.has_chamber_1 {
                self.generate_hall(hall1x, hall1y, 2, self.vertical_layout);
            }
            if self.chamber_state.has_chamber_3 {
                self.generate_hall(hall2x, hall2y, 2, self.vertical_layout);
            }
        } else {
            self.generate_hall(hall1x, hall1y, 16, self.vertical_layout);
        }
    }

    /// C++ `FixTilesPatterns()` (drlg_l1.cpp) — three passes converting the
    /// raw MakeDmt walls into the dirt-wall / corner vocabulary that AddWall
    /// and the renderer expect. Tile names follow the Rust enum; values match
    /// C++ (`SECorner` = DWall=4, `NWCorner` = VCorner=16).
    fn fix_tiles_patterns(&mut self) {
        for j in 0..DUNGEON_SIZE {
            for i in 0..DUNGEON_SIZE {
                if i + 1 < DUNGEON_SIZE {
                    if self.dungeon[j][i] == Tile::HWall && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::DirtHwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::DirtHwall;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j][i + 1] == Tile::HWall {
                        self.dungeon[j][i + 1] = Tile::HWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::DirtVwallEnd;
                    }
                }
                if j + 1 < DUNGEON_SIZE {
                    if self.dungeon[j][i] == Tile::VWall && self.dungeon[j + 1][i] == Tile::Dirt {
                        self.dungeon[j + 1][i] = Tile::DirtVwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j + 1][i] == Tile::VWall {
                        self.dungeon[j + 1][i] = Tile::VWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j + 1][i] == Tile::Dirt {
                        self.dungeon[j + 1][i] = Tile::DirtVwall;
                    }
                }
            }
        }

        for j in 0..DUNGEON_SIZE {
            for i in 0..DUNGEON_SIZE {
                if i + 1 < DUNGEON_SIZE {
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j][i + 1] == Tile::DirtVwall {
                        self.dungeon[j][i + 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::VDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::HWallEnd && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::DirtHwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j][i + 1] == Tile::DirtVwallEnd {
                        self.dungeon[j][i + 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::DirtVwall && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::VDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::HWall && self.dungeon[j][i + 1] == Tile::DirtVwall {
                        self.dungeon[j][i + 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::DirtVwall && self.dungeon[j][i + 1] == Tile::VWall {
                        self.dungeon[j][i + 1] = Tile::VWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::HWallEnd && self.dungeon[j][i + 1] == Tile::DirtVwall {
                        self.dungeon[j][i + 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::HWall && self.dungeon[j][i + 1] == Tile::VWall {
                        self.dungeon[j][i + 1] = Tile::VWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::Corner && self.dungeon[j][i + 1] == Tile::Dirt {
                        self.dungeon[j][i + 1] = Tile::DirtVwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::HDirtCorner && self.dungeon[j][i + 1] == Tile::VWall {
                        self.dungeon[j][i + 1] = Tile::VWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::HWallEnd && self.dungeon[j][i + 1] == Tile::VWall {
                        self.dungeon[j][i + 1] = Tile::VWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::HWallEnd && self.dungeon[j][i + 1] == Tile::DirtVwallEnd {
                        self.dungeon[j][i + 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::SECorner && self.dungeon[j][i + 1] == Tile::NWCorner {
                        self.dungeon[j][i + 1] = Tile::HCorner;
                    }
                    if self.dungeon[j][i] == Tile::HWallEnd && self.dungeon[j][i + 1] == Tile::Floor {
                        self.dungeon[j][i + 1] = Tile::HCorner;
                    }
                    if self.dungeon[j][i] == Tile::HWall && self.dungeon[j][i + 1] == Tile::DirtVwallEnd {
                        self.dungeon[j][i + 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::HWall && self.dungeon[j][i + 1] == Tile::Floor {
                        self.dungeon[j][i + 1] = Tile::HCorner;
                    }
                }
                if i > 0 {
                    if self.dungeon[j][i] == Tile::DirtHwallEnd && self.dungeon[j][i - 1] == Tile::Dirt {
                        self.dungeon[j][i - 1] = Tile::DirtVwall;
                    }
                    if self.dungeon[j][i] == Tile::DirtVwall && self.dungeon[j][i - 1] == Tile::DirtHwallEnd {
                        self.dungeon[j][i - 1] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j][i - 1] == Tile::Dirt {
                        self.dungeon[j][i - 1] = Tile::DirtVwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j][i - 1] == Tile::DirtHwallEnd {
                        self.dungeon[j][i - 1] = Tile::HDirtCorner;
                    }
                }
                if j + 1 < DUNGEON_SIZE {
                    if self.dungeon[j][i] == Tile::VWall && self.dungeon[j + 1][i] == Tile::HWall {
                        self.dungeon[j + 1][i] = Tile::HWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j + 1][i] == Tile::DirtHwall {
                        self.dungeon[j + 1][i] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::DirtHwall && self.dungeon[j + 1][i] == Tile::HWall {
                        self.dungeon[j + 1][i] = Tile::HWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j + 1][i] == Tile::HWall {
                        self.dungeon[j + 1][i] = Tile::HWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::HDirtCorner && self.dungeon[j + 1][i] == Tile::HWall {
                        self.dungeon[j + 1][i] = Tile::HWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j + 1][i] == Tile::Dirt {
                        self.dungeon[j + 1][i] = Tile::DirtVwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j + 1][i] == Tile::Floor {
                        self.dungeon[j + 1][i] = Tile::NWCorner;
                    }
                    if self.dungeon[j][i] == Tile::VWall && self.dungeon[j + 1][i] == Tile::Floor {
                        self.dungeon[j + 1][i] = Tile::NWCorner;
                    }
                    if self.dungeon[j][i] == Tile::Floor && self.dungeon[j + 1][i] == Tile::NWCorner {
                        self.dungeon[j + 1][i] = Tile::HCorner;
                    }
                }
                if j > 0 {
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j - 1][i] == Tile::Dirt {
                        self.dungeon[j - 1][i] = Tile::HWallEnd;
                    }
                    if self.dungeon[j][i] == Tile::VWallEnd && self.dungeon[j - 1][i] == Tile::Dirt {
                        self.dungeon[j - 1][i] = Tile::DirtVwallEnd;
                    }
                    if self.dungeon[j][i] == Tile::HWallEnd && self.dungeon[j - 1][i] == Tile::DirtVwallEnd {
                        self.dungeon[j - 1][i] = Tile::HDirtCorner;
                    }
                    if self.dungeon[j][i] == Tile::DirtHwall && self.dungeon[j - 1][i] == Tile::DirtVwallEnd {
                        self.dungeon[j - 1][i] = Tile::HDirtCorner;
                    }
                }
            }
        }

        for j in 0..DUNGEON_SIZE {
            for i in 0..DUNGEON_SIZE {
                if j + 1 < DUNGEON_SIZE && self.dungeon[j][i] == Tile::SECorner && self.dungeon[j + 1][i] == Tile::HWall {
                    self.dungeon[j + 1][i] = Tile::HWallEnd;
                }
                if i + 1 < DUNGEON_SIZE && self.dungeon[j][i] == Tile::HWall && self.dungeon[j][i + 1] == Tile::DirtVwall {
                    self.dungeon[j][i + 1] = Tile::HDirtCorner;
                }
                if j + 1 < DUNGEON_SIZE && self.dungeon[j][i] == Tile::DirtHwall && self.dungeon[j + 1][i] == Tile::Dirt {
                    self.dungeon[j + 1][i] = Tile::VDirtCorner;
                }
            }
        }
    }

    /// Convert dungeon mask to tile types
    ///
    /// C++ source: MakeDmt() in drlg_l1.cpp:560-577
    pub fn make_dmt(&mut self) {
        for y in 0..(DUNGEON_SIZE - 1) {
            for x in 0..(DUNGEON_SIZE - 1) {
                let curr = self.dungeon_mask[y][x];
                let right = self.dungeon_mask[y][x + 1];
                let down = self.dungeon_mask[y + 1][x];
                let diag = self.dungeon_mask[y + 1][x + 1];

                if curr {
                    self.dungeon[y][x] = Tile::Floor;
                } else if !diag && down && right {
                    // Remove diagonal corners
                    self.dungeon[y][x] = Tile::Floor;
                } else if diag && down && right {
                    self.dungeon[y][x] = Tile::NWCorner; // VCorner in C++
                } else if down {
                    self.dungeon[y][x] = Tile::HWall;
                } else if right {
                    self.dungeon[y][x] = Tile::VWall;
                } else if diag {
                    self.dungeon[y][x] = Tile::SECorner; // DWall in C++
                } else {
                    self.dungeon[y][x] = Tile::Dirt;
                }
            }
        }
    }

    /// Check if horizontal wall can be placed
    ///
    /// C++ source: HorizontalWallOk() in drlg_l1.cpp:579-595
    fn horizontal_wall_ok(&self, x: usize, y: usize) -> i32 {
        let mut length = 1;
        while x + length < DUNGEON_SIZE && self.dungeon[y][x + length] == Tile::Floor {
            // C++ has no bounds checks here; the dungeon border is Dirt so the
            // walk always stops before the edge in practice. Guard anyway.
            if y == 0 || y >= DUNGEON_SIZE - 1 {
                break;
            }
            if self.dungeon[y - 1][x + length] != Tile::Floor
                || self.dungeon[y + 1][x + length] != Tile::Floor
                || self.protected[y][x + length]
                || self.chamber[y][x + length]
            {
                break;
            }
            length += 1;
        }

        if length == 1 {
            return -1;
        }
        if x + length >= DUNGEON_SIZE {
            return -1;
        }

        // C++: the terminating tile must be one of the wall/corner variants.
        let end_tile = self.dungeon[y][x + length];
        if matches!(
            end_tile,
            Tile::Corner
                | Tile::SECorner
                | Tile::DArch
                | Tile::VWallEnd
                | Tile::HWallEnd
                | Tile::NWCorner
                | Tile::HCorner
                | Tile::DirtHwall
                | Tile::DirtVwall
                | Tile::VDirtCorner
                | Tile::HDirtCorner
                | Tile::DirtHwallEnd
                | Tile::DirtVwallEnd
        ) {
            return length as i32;
        }

        -1
    }

    /// Check if vertical wall can be placed
    ///
    /// C++ source: VerticalWallOk() in drlg_l1.cpp:597-613
    fn vertical_wall_ok(&self, x: usize, y: usize) -> i32 {
        let mut length = 1;
        while y + length < DUNGEON_SIZE && self.dungeon[y + length][x] == Tile::Floor {
            if x == 0 || x >= DUNGEON_SIZE - 1 {
                break;
            }
            if self.dungeon[y + length][x - 1] != Tile::Floor
                || self.dungeon[y + length][x + 1] != Tile::Floor
                || self.protected[y + length][x]
                || self.chamber[y + length][x]
            {
                break;
            }
            length += 1;
        }

        if length == 1 {
            return -1;
        }
        if y + length >= DUNGEON_SIZE {
            return -1;
        }

        let end_tile = self.dungeon[y + length][x];
        if matches!(
            end_tile,
            Tile::Corner
                | Tile::SECorner
                | Tile::DArch
                | Tile::VWallEnd
                | Tile::HWallEnd
                | Tile::NWCorner
                | Tile::HCorner
                | Tile::DirtHwall
                | Tile::DirtVwall
                | Tile::VDirtCorner
                | Tile::HDirtCorner
                | Tile::DirtHwallEnd
                | Tile::DirtVwallEnd
        ) {
            return length as i32;
        }

        -1
    }

    /// Place horizontal wall with random door
    ///
    /// C++ source: HorizontalWall() in drlg_l1.cpp:615-650
    fn horizontal_wall(&mut self, x: usize, y: usize, start: Tile, max_x: i32) {
        let mut wall_tile = Tile::HWall;
        let mut door_tile = Tile::HWallDoor;
        let mut start = start;

        // C++ GenerateRnd(4): 2 = arch, 3 = fence
        match self.rng.generate(4) {
            2 => {
                wall_tile = Tile::ArchH1;
                door_tile = Tile::ArchH1;
                if start == Tile::HWall {
                    start = Tile::ArchH1;
                } else if start == Tile::SECorner {
                    start = Tile::HArchVWall;
                }
            }
            3 => {
                wall_tile = Tile::HFence;
                if start == Tile::HWall {
                    start = Tile::HFence;
                } else if start == Tile::SECorner {
                    start = Tile::HFenceVWall;
                }
            }
            _ => {}
        }

        // C++ GenerateRnd(6) == 5 -> arch door
        if self.rng.generate(6) == 5 {
            door_tile = Tile::ArchH1;
        }

        self.dungeon[y][x] = start;

        for i in 1..max_x {
            self.dungeon[y][x + i as usize] = wall_tile;
        }

        // C++ GenerateRnd(maxX - 1) + 1
        let door_pos = self.rng.generate(max_x - 1) + 1;
        self.dungeon[y][x + door_pos as usize] = door_tile;
        if door_tile == Tile::HWallDoor {
            self.protected[y][x + door_pos as usize] = true;
        }
    }

    /// Place vertical wall with random door
    ///
    /// C++ source: VerticalWall() in drlg_l1.cpp:652-687
    fn vertical_wall(&mut self, x: usize, y: usize, start: Tile, max_y: i32) {
        let mut wall_tile = Tile::VWall;
        let mut door_tile = Tile::VWallDoor;
        let mut start = start;

        match self.rng.generate(4) {
            2 => {
                wall_tile = Tile::ArchV1;
                door_tile = Tile::ArchV1;
                if start == Tile::VWall {
                    start = Tile::ArchV1;
                } else if start == Tile::SECorner {
                    start = Tile::HWallVArch;
                }
            }
            3 => {
                wall_tile = Tile::VFence;
                if start == Tile::VWall {
                    start = Tile::VFence;
                } else if start == Tile::SECorner {
                    start = Tile::HWallVFence;
                }
            }
            _ => {}
        }

        if self.rng.generate(6) == 5 {
            door_tile = Tile::ArchV1;
        }

        self.dungeon[y][x] = start;

        for i in 1..max_y {
            self.dungeon[y + i as usize][x] = wall_tile;
        }

        let door_pos = self.rng.generate(max_y - 1) + 1;
        self.dungeon[y + door_pos as usize][x] = door_tile;
        if door_tile == Tile::VWallDoor {
            self.protected[y + door_pos as usize][x] = true;
        }
    }

    /// Add walls between rooms
    ///
    /// C++ source: AddWall() in drlg_l1.cpp:689-750
    fn add_wall(&mut self) {
        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                if self.protected[y][x] || self.chamber[y][x] {
                    continue;
                }

                if self.dungeon[y][x] == Tile::Corner {
                    let _ = self.rng.generate(1); // C++ DiscardRandomValues(1)
                    let max_x = self.horizontal_wall_ok(x, y);
                    if max_x != -1 {
                        self.horizontal_wall(x, y, Tile::HWall, max_x);
                    }
                }
                if self.dungeon[y][x] == Tile::Corner {
                    let _ = self.rng.generate(1);
                    let max_y = self.vertical_wall_ok(x, y);
                    if max_y != -1 {
                        self.vertical_wall(x, y, Tile::VWall, max_y);
                    }
                }
                if self.dungeon[y][x] == Tile::VWallEnd {
                    let _ = self.rng.generate(1);
                    let max_x = self.horizontal_wall_ok(x, y);
                    if max_x != -1 {
                        self.horizontal_wall(x, y, Tile::SECorner, max_x);
                    }
                }
                if self.dungeon[y][x] == Tile::HWallEnd {
                    let _ = self.rng.generate(1);
                    let max_y = self.vertical_wall_ok(x, y);
                    if max_y != -1 {
                        self.vertical_wall(x, y, Tile::SECorner, max_y);
                    }
                }
                if self.dungeon[y][x] == Tile::HWall {
                    let _ = self.rng.generate(1);
                    let max_x = self.horizontal_wall_ok(x, y);
                    if max_x != -1 {
                        self.horizontal_wall(x, y, Tile::HWall, max_x);
                    }
                }
                if self.dungeon[y][x] == Tile::VWall {
                    let _ = self.rng.generate(1);
                    let max_y = self.vertical_wall_ok(x, y);
                    if max_y != -1 {
                        self.vertical_wall(x, y, Tile::VWall, max_y);
                    }
                }
            }
        }
    }

    /// C++ `BASE_TYPES` (drlg_l1.cpp) - 207 entries, indexed by tile value.
    const BASE_TYPES: [u8; 207] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        16, 17, 0, 0, 0, 0, 0, 0, 0, 1, 2, 10, 4, 5, 6, 7,
        8, 9, 10, 11, 12, 14, 5, 14, 10, 4, 14, 4, 5, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        2, 3, 4, 1, 6, 7, 16, 17, 2, 1, 1, 2, 2, 1, 1, 2,
        2, 2, 2, 2, 1, 1, 11, 1, 13, 13, 13, 1, 2, 1, 2, 1,
        2, 1, 2, 2, 2, 2, 12, 0, 0, 11, 1, 11, 1, 13, 0, 0,
        0, 0, 0, 0, 0, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13,
        13, 13, 1, 11, 2, 12, 13, 13, 13, 12, 2, 1, 2, 2, 4, 14,
        4, 10, 13, 13, 4, 4, 1, 1, 4, 2, 2, 13, 13, 13, 13, 25,
        26, 28, 30, 31, 41, 43, 40, 41, 42, 43, 25, 41, 43, 28, 28, 1,
        2, 25, 26, 22, 22, 25, 26, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    /// C++ `TILE_DECORATIONS` (drlg_l1.cpp) - 207 entries, indexed by tile value.
    const TILE_DECORATIONS: [u8; 207] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        16, 17, 0, 0, 0, 0, 0, 0, 0, 25, 26, 0, 28, 0, 30, 31,
        0, 0, 0, 0, 0, 0, 0, 0, 40, 41, 42, 43, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 79,
        80, 0, 82, 0, 0, 0, 0, 0, 0, 79, 0, 80, 0, 0, 79, 80,
        0, 2, 2, 2, 1, 1, 11, 25, 13, 13, 13, 1, 2, 1, 2, 1,
        2, 1, 2, 2, 2, 2, 12, 0, 0, 11, 1, 11, 1, 13, 0, 0,
        0, 0, 0, 0, 0, 13, 13, 13, 13, 13, 13, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    /// C++ `ShadowPatterns[37]` (drlg_l1.cpp) - (strig, s1, s2, s3, nv1, nv2, nv3).
    const SHADOW_PATTERNS: [(u8, u8, u8, u8, u8, u8, u8); 37] = [
        (7, 13, 0, 13, 144, 0, 142),
        (16, 13, 0, 13, 144, 0, 142),
        (15, 13, 0, 13, 145, 0, 142),
        (5, 13, 13, 13, 152, 140, 139),
        (5, 13, 1, 13, 143, 146, 139),
        (5, 13, 13, 2, 143, 140, 148),
        (5, 0, 1, 2, 0, 146, 148),
        (5, 13, 11, 13, 143, 147, 139),
        (5, 13, 13, 12, 143, 140, 149),
        (5, 13, 11, 12, 150, 147, 149),
        (5, 13, 1, 12, 143, 146, 149),
        (5, 13, 11, 2, 143, 147, 148),
        (9, 13, 13, 13, 144, 140, 142),
        (9, 13, 1, 13, 144, 146, 142),
        (9, 13, 11, 13, 151, 147, 142),
        (8, 13, 0, 13, 144, 0, 139),
        (8, 13, 0, 12, 143, 0, 149),
        (8, 0, 0, 2, 0, 0, 148),
        (11, 0, 0, 13, 0, 0, 139),
        (11, 13, 0, 13, 139, 0, 139),
        (11, 2, 0, 13, 148, 0, 139),
        (11, 12, 0, 13, 149, 0, 139),
        (11, 13, 11, 12, 139, 0, 149),
        (14, 0, 0, 13, 0, 0, 139),
        (14, 13, 0, 13, 139, 0, 139),
        (14, 2, 0, 13, 148, 0, 139),
        (14, 12, 0, 13, 149, 0, 139),
        (14, 13, 11, 12, 139, 0, 149),
        (10, 0, 13, 0, 0, 140, 0),
        (10, 13, 13, 0, 140, 140, 0),
        (10, 0, 1, 0, 0, 146, 0),
        (10, 13, 11, 0, 140, 147, 0),
        (12, 0, 13, 0, 0, 140, 0),
        (12, 13, 13, 0, 140, 140, 0),
        (12, 0, 1, 0, 0, 146, 0),
        (12, 13, 11, 0, 140, 147, 0),
        (3, 13, 11, 12, 150, 0, 0),
    ];

    /// Fix dirt-tile transitions to their decorated variants.
    ///
    /// C++ source: FixDirtTiles() in drlg_l1.cpp:1100-1115
    fn fix_dirt_tiles(&mut self) {
        for y in 0..(DUNGEON_SIZE - 1) {
            for x in 0..(DUNGEON_SIZE - 1) {
                if self.dungeon[y][x] == Tile::HDirtCorner && self.dungeon[y][x + 1] != Tile::DirtVwall {
                    self.dungeon[y][x] = Tile::DirtCorner2;
                }
                if self.dungeon[y][x] == Tile::DirtVwall && self.dungeon[y][x + 1] != Tile::DirtVwall {
                    self.dungeon[y][x] = Tile::DirtVWall2;
                }
                if self.dungeon[y][x] == Tile::DirtVwallEnd && self.dungeon[y][x + 1] != Tile::DirtVwall {
                    self.dungeon[y][x] = Tile::DirtVWallEnd2;
                }
                if self.dungeon[y][x] == Tile::DirtHwall && self.dungeon[y + 1][x] != Tile::DirtHwall {
                    self.dungeon[y][x] = Tile::DirtHWall2;
                }
                if self.dungeon[y][x] == Tile::HDirtCorner && self.dungeon[y + 1][x] != Tile::DirtHwall {
                    self.dungeon[y][x] = Tile::DirtCorner2;
                }
                if self.dungeon[y][x] == Tile::DirtHwallEnd && self.dungeon[y + 1][x] != Tile::DirtHwall {
                    self.dungeon[y][x] = Tile::DirtHWallEnd2;
                }
            }
        }
    }

    /// Fix corner tiles.
    ///
    /// C++ source: FixCornerTiles() in drlg_l1.cpp:1117-1130
    fn fix_corner_tiles(&mut self) {
        for y in 1..(DUNGEON_SIZE - 1) {
            for x in 1..(DUNGEON_SIZE - 1) {
                if !self.protected[y][x]
                    && self.dungeon[y][x] == Tile::HCorner
                    && self.dungeon[y][x - 1] == Tile::Floor
                    && self.dungeon[y - 1][x] == Tile::VWall
                {
                    self.dungeon[y][x] = Tile::NWCorner; // C++ VCorner
                }
                if self.dungeon[y][x] == Tile::DirtCorner2
                    && self.dungeon[y][x + 1] == Tile::Floor
                    && self.dungeon[y + 1][x] == Tile::VWall
                {
                    self.dungeon[y][x] = Tile::HArchEnd;
                }
                if self.dungeon[y][x] == Tile::DirtCorner2
                    && self.dungeon[y + 1][x] == Tile::Floor
                    && self.dungeon[y][x + 1] == Tile::HWall
                {
                    self.dungeon[y][x] = Tile::VArchEnd;
                }
            }
        }
    }

    /// Randomly substitute decorated tile variants.
    ///
    /// C++ source: Substitution() in drlg_l1.cpp:1014-1045
    fn substitution(&mut self) {
        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                if self.rng.generate(4) != 0 {
                    continue; // C++ FlipCoin(4)
                }
                let c = Self::TILE_DECORATIONS[self.dungeon[y][x] as usize];
                if c == 0 || self.protected[y][x] {
                    continue;
                }
                let mut rv = self.rng.generate(16);
                let mut i: i32 = -1;
                while rv >= 0 {
                    i += 1;
                    if i == Self::TILE_DECORATIONS.len() as i32 {
                        i = 0;
                    }
                    if c == Self::TILE_DECORATIONS[i as usize] {
                        rv -= 1;
                    }
                }
                let mut i = i as usize;
                if i == Tile::VWall4 as usize && y > 0 {
                    if Self::TILE_DECORATIONS[self.dungeon[y - 1][x] as usize] != Tile::VWall2 as u8
                        || self.protected[y - 1][x]
                    {
                        i = Tile::VWall2 as usize;
                    } else {
                        self.dungeon[y - 1][x] = Tile::VWall5;
                    }
                }
                if i == Tile::HWall4 as usize && x + 1 < DUNGEON_SIZE {
                    if Self::TILE_DECORATIONS[self.dungeon[y][x + 1] as usize] != Tile::HWall2 as u8
                        || self.protected[y][x + 1]
                    {
                        i = Tile::HWall2 as usize;
                    } else {
                        self.dungeon[y][x + 1] = Tile::HWall5;
                    }
                }
                self.dungeon[y][x] = Tile::try_from(i as u8).unwrap_or(Tile::Invalid);
            }
        }
    }

    /// Apply the Cathedral shadow patterns.
    ///
    /// C++ source: ApplyShadowsPatterns() in drlg_l1.cpp:280-346
    fn apply_shadows_patterns(&mut self) {
        for y in 1..DUNGEON_SIZE {
            for x in 1..DUNGEON_SIZE {
                let s00 = Self::BASE_TYPES[self.dungeon[y][x] as usize];
                let s10 = Self::BASE_TYPES[self.dungeon[y][x - 1] as usize];
                let s01 = Self::BASE_TYPES[self.dungeon[y - 1][x] as usize];
                let s11 = Self::BASE_TYPES[self.dungeon[y - 1][x - 1] as usize];

                for &(strig, s1, s2, s3, nv1, nv2, nv3) in Self::SHADOW_PATTERNS.iter() {
                    if strig != s00 {
                        continue;
                    }
                    if s1 != 0 && s1 != s11 {
                        continue;
                    }
                    if s2 != 0 && s2 != s01 {
                        continue;
                    }
                    if s3 != 0 && s3 != s10 {
                        continue;
                    }
                    if nv1 != 0 && !self.protected[y - 1][x - 1] {
                        self.dungeon[y - 1][x - 1] = Tile::try_from(nv1).unwrap_or(Tile::Invalid);
                    }
                    if nv2 != 0 && !self.protected[y - 1][x] {
                        self.dungeon[y - 1][x] = Tile::try_from(nv2).unwrap_or(Tile::Invalid);
                    }
                    if nv3 != 0 && !self.protected[y][x - 1] {
                        self.dungeon[y][x - 1] = Tile::try_from(nv3).unwrap_or(Tile::Invalid);
                    }
                }
            }
        }

        for y in 1..DUNGEON_SIZE {
            for x in 1..DUNGEON_SIZE {
                if self.protected[y][x - 1] {
                    continue;
                }
                let tile = self.dungeon[y][x - 1];
                let right = self.dungeon[y][x];
                let fence = matches!(
                    right,
                    Tile::DFence
                        | Tile::VFenceEnd
                        | Tile::VFence
                        | Tile::HWallVFence
                        | Tile::HArchVFence
                        | Tile::HArchVDoor
                );
                match tile {
                    Tile::Floor12 => {
                        self.dungeon[y][x - 1] = if fence { Tile::Floor14 } else { Tile::Floor12 };
                    }
                    Tile::HArchShadow => {
                        self.dungeon[y][x - 1] = if fence { Tile::HArchShadow2 } else { Tile::HArchShadow };
                    }
                    Tile::HWallShadow => {
                        self.dungeon[y][x - 1] = if fence { Tile::HWallShadow2 } else { Tile::HWallShadow };
                    }
                    _ => {}
                }
            }
        }
    }
    /// Randomly add floor variations
    ///
    /// C++ source: FillFloor() in drlg_l1.cpp:368-380
    fn fill_floor(&mut self) {
        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                if self.dungeon[y][x] != Tile::Floor || self.protected[y][x] {
                    continue;
                }

                // C++ uses RandomIntLessThan(3) which returns 0, 1, or 2
                let rv = self.rng.random_less_than(3) as u32;
                if rv == 1 {
                    // C++ FillFloor: rv==1 -> Floor22 (162)
                    self.dungeon[y][x] = Tile::Floor22;
                } else if rv == 2 {
                    // C++ FillFloor: rv==2 -> Floor23 (163)
                    self.dungeon[y][x] = Tile::Floor23;
                }
            }
        }
    }


    /// Generate Cathedral level
    ///
    /// C++ source: CreateL5Dungeon() / GenerateLevel() in drlg_l1.cpp
    ///
    /// Faithfully reproduces the C++ pipeline: re-roll the room layout until
    /// it is large enough and stairs can be placed, then run the final tile
    /// fix-up passes (dirt/corner substitution, shadows, lamps, floor fill).
    pub fn generate(&mut self, _level_type: DungeonType, seed: u32) {
        // Seed the Diablo LCG before generation so results are deterministic
        // and reproduce the C++ `SetRndSeed(seed)` behaviour exactly.
        self.rng.set_seed(seed);

        // C++ GenerateLevel(): minarea for currlevel 1 is 533.
        loop {
            loop {
                self.first_room();
                if self.find_area() >= 533 {
                    break;
                }
                            }
                        self.init_dungeon();
            self.make_dmt();
            self.fill_chambers();
            self.fix_tiles_patterns();
            self.add_wall();
            // FloodTransparencyValues(13) only touches dTransVal; skip.
                        let ok = self.place_stairs();
                        if ok {
                break;
            }
        }

        // FixTransparency() only touches dTransVal; skip.
        self.fix_dirt_tiles();
        self.fix_corner_tiles();
        self.substitution();
        self.apply_shadows_patterns();

        // C++: numt = GenerateRnd(5) + 5 lamp minisets.
        let num_lamps = self.rng.generate(5) + 5;
        for _ in 0..num_lamps {
            self.place_miniset(&lamps_miniset(), DUNGEON_SIZE * DUNGEON_SIZE, true);
        }

        // Add floor variations
        self.fill_floor();
    }

    /// Place stairs for the Cathedral level.
    ///
    /// C++ source: PlaceCathedralStairs() in drlg_l1.cpp:1132-1174
    ///
    /// Quest-free port: PWATER / LTBANNER quests are not active, so only
    /// STAIRSUP and STAIRSDOWN are placed (L5STAIRSUP is only used with the
    /// legacy `pOriginalCathedral` flag).
    fn place_stairs(&mut self) -> bool {
        let mut success = true;
        let stairs_up = if self.original_cathedral {
            l5_stairs_up_miniset()
        } else {
            stairs_up_miniset()
        };
        if self.place_miniset(&stairs_up, DUNGEON_SIZE * DUNGEON_SIZE, true).is_none() {
            success = false;
        }
        if self.place_miniset(&stairs_down_miniset(), DUNGEON_SIZE * DUNGEON_SIZE, true).is_none() {
            success = false;
        }
        success
    }

    /// Place a miniset using the C++ `PlaceMiniSet` scan algorithm.
    ///
    /// C++ source: PlaceMiniSet() in gendung.cpp:648-683
    ///
    /// Starts from a random position, then scans with wrap-around until the
    /// miniset matches. `drlg1_quirk` replicates the Cathedral bias that
    /// skips positions with x/y <= 12. Returns the placed position.
    fn place_miniset(&mut self, miniset: &Miniset, tries: usize, drlg1_quirk: bool) -> Option<(usize, usize)> {
        let sw = miniset.width as i32;
        let sh = miniset.height as i32;
        let mut x = self.rng.generate(DUNGEON_SIZE as i32 - sw);
        let mut y = self.rng.generate(DUNGEON_SIZE as i32 - sh);
        let mut i = 0usize;
        while i < tries {
            if x == DUNGEON_SIZE as i32 - sw {
                x = 0;
                y += 1;
                if y == DUNGEON_SIZE as i32 - sh {
                    y = 0;
                }
            }
            if drlg1_quirk {
                let mut valid = true;
                if x <= 12 {
                    x += 1;
                    valid = false;
                }
                if y <= 12 {
                    y += 1;
                    valid = false;
                }
                if !valid {
                    i += 1;
                    x += 1;
                    continue;
                }
            }
            // SetPieceRoom is empty for quest-free Cathedral generation.
            if miniset.matches(&self.dungeon, &self.protected, x as usize, y as usize) {
                miniset.place(&mut self.dungeon, x as usize, y as usize);
                return Some((x as usize, y as usize));
            }
            i += 1;
            x += 1;
        }
        None
    }

    /// Convert dungeon tiles to final output (Dungeon struct)
    ///
    /// C++ source: Uses memcpy(pdungeon, dungeon) after generation
    ///
    /// Converts logical Cathedral tiles to actual game tiles that the engine uses.
    pub fn write_to_dungeon(&self, dungeon: &mut Dungeon) {
        // TODO: Implement tile conversion logic
        // Cathedral tiles → Floor/Dirt visual tiles
    }
}

// -------------------------------------------------------------------
// Transparency helpers (C++ drlg_l1.cpp)
// -------------------------------------------------------------------

/// C++ `FixTransparency()` (drlg_l1.cpp:1031-1064): propagate the floor
/// region's `TransVal` into the 2x2 micro-tile footprint of Dirt walls
/// (DirtHwall/DirtVwall/ends/corners) so the renderer treats those walls as
/// see-through over the floor.
pub fn fix_transparency(
    tiles: &[[Tile; DUNGEON_SIZE]; DUNGEON_SIZE],
    trans_val: &mut [[i8; MAXDUNY]; MAXDUNX],
) {
    let mut yy = 16usize;
    for j in 0..DMAXY {
        let mut xx = 16usize;
        for i in 0..DMAXX {
            let t = tiles[i][j];
            // BUGFIX: `j > 0` is checked after the tile test in C++; keep the
            // same guarded semantics.
            if t == Tile::DirtHwallEnd && j > 0 && tiles[i][j - 1] == Tile::DirtHwall {
                trans_val[xx + 1][yy] = trans_val[xx][yy];
                trans_val[xx + 1][yy + 1] = trans_val[xx][yy];
            }
            if t == Tile::DirtVwallEnd && i + 1 < DMAXY && tiles[i + 1][j] == Tile::DirtVwall {
                trans_val[xx][yy + 1] = trans_val[xx][yy];
                trans_val[xx + 1][yy + 1] = trans_val[xx][yy];
            }
            if t == Tile::DirtHwall {
                trans_val[xx + 1][yy] = trans_val[xx][yy];
                trans_val[xx + 1][yy + 1] = trans_val[xx][yy];
            }
            if t == Tile::DirtVwall {
                trans_val[xx][yy + 1] = trans_val[xx][yy];
                trans_val[xx + 1][yy + 1] = trans_val[xx][yy];
            }
            if t == Tile::VDirtCorner {
                trans_val[xx + 1][yy] = trans_val[xx][yy];
                trans_val[xx][yy + 1] = trans_val[xx][yy];
                trans_val[xx + 1][yy + 1] = trans_val[xx][yy];
            }
            xx += 2;
        }
        yy += 2;
    }
}

/// C++ drlg_l1.cpp:1202-1211 (`DRLG_CopyTrans(xx, yy + 1, xx, yy)` for every
/// `EntranceStairs` tile): copy the TransVal from the micro-tile row below each
/// stairs tile into the stairs row so the stairs render over the drop.
pub fn copy_stairs_transparency(
    tiles: &[[Tile; DUNGEON_SIZE]; DUNGEON_SIZE],
    trans_val: &mut [[i8; MAXDUNY]; MAXDUNX],
) {
    for j in 0..DMAXY {
        for i in 0..DMAXX {
            if tiles[i][j] == Tile::EntranceStairs {
                let xx = 2 * i + 16;
                let yy = 2 * j + 16;
                trans_val[xx][yy] = trans_val[xx][yy + 1];
                trans_val[xx + 1][yy] = trans_val[xx + 1][yy + 1];
            }
        }
    }
}

// ===================================================================
// TESTS
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// C++ `FixTransparency` (drlg_l1.cpp:1031-1064): a flood-filled floor
    /// region (value 7) adjacent to Dirt walls propagates its TransVal across
    /// the wall's 2x2 micro footprint (right/bottom copies).
    #[test]
    fn test_fix_transparency_propagates_into_dirt_walls() {
        let mut tiles = [[Tile::Dirt; DUNGEON_SIZE]; DUNGEON_SIZE];
        let mut trans_val = [[0i8; MAXDUNY]; MAXDUNX];
        // 2x2 floor region at logical (10,10)-(11,11), region id 7.
        for y in 10..12 {
            for x in 10..12 {
                tiles[x][y] = Tile::Floor;
                let xx = 2 * x + 16;
                let yy = 2 * y + 16;
                trans_val[xx][yy] = 7;
                trans_val[xx + 1][yy] = 7;
                trans_val[xx][yy + 1] = 7;
                trans_val[xx + 1][yy + 1] = 7;
            }
        }
        // Dirt walls on the floor region's east/south borders. The top-left
        // micro of each wall is a diagonal/edge neighbour of a floor tile, so
        // after the C++ flood fill it already carries the region id (simulate
        // that here).
        tiles[12][10] = Tile::DirtHwall; // east of (11,10)
        tiles[12][11] = Tile::DirtHwallEnd; // east of (11,11)
        tiles[10][12] = Tile::DirtVwallEnd; // south of (10,11)
        tiles[11][12] = Tile::DirtVwall; // south of (11,11)
        tiles[12][12] = Tile::VDirtCorner; // SE diagonal of (11,11)
        // Wall micros adjacent to the floor inherit the region id (flood).
        trans_val[2 * 12 + 16][2 * 10 + 16] = 7; // DirtHwall (12,10) top-left
        trans_val[2 * 12 + 16][2 * 11 + 16] = 7; // DirtHwallEnd (12,11) top-left
        trans_val[2 * 10 + 16][2 * 12 + 16] = 7; // DirtVwallEnd (10,12) top-left
        trans_val[2 * 11 + 16][2 * 12 + 16] = 7; // DirtVwall (11,12) top-left
        trans_val[2 * 12 + 16][2 * 12 + 16] = 7; // VDirtCorner (12,12) top-left

        fix_transparency(&tiles, &mut trans_val);

        // DirtHwall (12,10): copies (xx+1,yy) and (xx+1,yy+1) from (xx,yy).
        let xx = 2 * 12 + 16;
        let yy = 2 * 10 + 16;
        assert_eq!(trans_val[xx + 1][yy], 7, "DirtHwall right micro");
        assert_eq!(trans_val[xx + 1][yy + 1], 7, "DirtHwall bottom-right micro");
        // DirtHwallEnd (12,11) with DirtHwall above at (12,10): same copies.
        let xx = 2 * 12 + 16;
        let yy = 2 * 11 + 16;
        assert_eq!(trans_val[xx + 1][yy], 7, "DirtHwallEnd right micro");
        assert_eq!(trans_val[xx + 1][yy + 1], 7, "DirtHwallEnd bottom-right micro");
        // DirtVwallEnd (10,12) with DirtVwall at (10+1,12): copies (xx,yy+1),(xx+1,yy+1).
        let xx = 2 * 10 + 16;
        let yy = 2 * 12 + 16;
        assert_eq!(trans_val[xx][yy + 1], 7, "DirtVwallEnd bottom micro");
        assert_eq!(trans_val[xx + 1][yy + 1], 7, "DirtVwallEnd bottom-right micro");
        // DirtVwall (11,12): copies (xx,yy+1) and (xx+1,yy+1).
        let xx = 2 * 11 + 16;
        let yy = 2 * 12 + 16;
        assert_eq!(trans_val[xx][yy + 1], 7, "DirtVwall bottom micro");
        assert_eq!(trans_val[xx + 1][yy + 1], 7, "DirtVwall bottom-right micro");
        // VDirtCorner (12,12): copies all three neighbors from its top-left micro.
        let xx = 2 * 12 + 16;
        let yy = 2 * 12 + 16;
        assert_eq!(trans_val[xx + 1][yy], 7, "VDirtCorner right micro");
        assert_eq!(trans_val[xx][yy + 1], 7, "VDirtCorner bottom micro");
        assert_eq!(trans_val[xx + 1][yy + 1], 7, "VDirtCorner bottom-right micro");
    }

    /// C++ drlg_l1.cpp:1202-1211: each EntranceStairs tile copies the TransVal
    /// from the micro-tile row below into its own row.
    #[test]
    fn test_copy_stairs_transparency() {
        let mut tiles = [[Tile::Dirt; DUNGEON_SIZE]; DUNGEON_SIZE];
        let mut trans_val = [[0i8; MAXDUNY]; MAXDUNX];
        tiles[20][20] = Tile::EntranceStairs;
        // The bottom micro row of the stairs tile (yy+1) carries region id 5;
        // C++ copies it up into the top row (yy).
        let xx = 2 * 20 + 16;
        let yy = 2 * 20 + 16;
        trans_val[xx][yy + 1] = 5;
        trans_val[xx + 1][yy + 1] = 5;

        copy_stairs_transparency(&tiles, &mut trans_val);

        assert_eq!(trans_val[xx][yy], 5, "stairs left micro inherits bottom row");
        assert_eq!(trans_val[xx + 1][yy], 5, "stairs right micro inherits bottom row");
    }

    #[test]
    fn test_tile_enum_values() {
        assert_eq!(Tile::VWall as u8, 1);
        assert_eq!(Tile::HWall as u8, 2);
        assert_eq!(Tile::Floor as u8, 13);
        assert_eq!(Tile::Dirt as u8, 22); // C++ Dirt=22
        assert_eq!(Tile::NWCorner as u8, 16); // C++ VCorner
        assert_eq!(Tile::SECorner as u8, 4); // C++ DWall
        assert_eq!(Tile::VWallDoor as u8, 25); // C++ VDoor
        assert_eq!(Tile::HWallDoor as u8, 26); // C++ HDoor
        assert_eq!(Tile::ArchH1 as u8, 12); // C++ HArch
        assert_eq!(Tile::EntranceStairs as u8, 64); // C++ EntranceStairs
        assert_eq!(Tile::Invalid as u8, 0); // C++ miniset wildcard
    }

    #[test]
    fn test_tile_try_from() {
        assert_eq!(Tile::try_from(1), Ok(Tile::VWall));
        assert_eq!(Tile::try_from(13), Ok(Tile::Floor));
        assert_eq!(Tile::try_from(16), Ok(Tile::NWCorner));
        assert_eq!(Tile::try_from(4), Ok(Tile::SECorner));
        assert_eq!(Tile::try_from(64), Ok(Tile::EntranceStairs));
        assert_eq!(Tile::try_from(0), Ok(Tile::Invalid));
        assert_eq!(Tile::try_from(57), Ok(Tile::StairTile57));
        assert_eq!(Tile::try_from(255), Err(()));
    }

    #[test]
    fn test_miniset_stairs_up() {
        let miniset = stairs_up_miniset();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 4);
        assert_eq!(miniset.search[0][0], Tile::Floor);
        assert_eq!(miniset.search[1][0], Tile::HWall); // C++ row 1 is HWall
        assert_eq!(miniset.replace[1][1], Tile::EntranceStairs); // C++ {63,64,65,0}
        assert_eq!(miniset.replace[1][2], Tile::StairTile65);
    }

    #[test]
    fn test_miniset_stairs_down() {
        let miniset = stairs_down_miniset();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 3);
        assert_eq!(miniset.replace[0][0], Tile::StairTile62); // C++ STAIRSDOWN raw tile
    }

    #[test]
    fn test_miniset_lamps() {
        let miniset = lamps_miniset();
        assert_eq!(miniset.width, 2);
        assert_eq!(miniset.height, 2);
        assert_eq!(miniset.search[0][0], Tile::Floor);
        assert_eq!(miniset.replace[0][0], Tile::Mega129); // C++ LAMPS raw tile
    }

    #[test]
    fn test_miniset_water_in() {
        let miniset = water_in_miniset();
        assert_eq!(miniset.width, 6);
        assert_eq!(miniset.height, 6);
        assert_eq!(miniset.search[0][0], Tile::Floor);
        assert_eq!(miniset.replace[0][0], Tile::Invalid); // C++ PWATERIN edge is 0
        assert_eq!(miniset.replace[3][2], Tile::Mega206); // C++ PWATERIN row 3: {0,85,206,80,81,0}
        assert_eq!(miniset.replace[3][3], Tile::HWall2);
    }

    #[test]
    fn test_miniset_matches() {
        let miniset = lamps_miniset();
        let mut dungeon = [[Tile::Dirt; DUNGEON_SIZE]; DUNGEON_SIZE];

        // Create 2x2 floor area
        dungeon[5][5] = Tile::Floor;
        dungeon[5][6] = Tile::Floor;
        dungeon[6][5] = Tile::Floor;
        dungeon[6][6] = Tile::Floor;

        let protected = [[false; DUNGEON_SIZE]; DUNGEON_SIZE];
        assert!(miniset.matches(&dungeon, &protected, 5, 5));
        assert!(!miniset.matches(&dungeon, &protected, 0, 0)); // Dirt area
    }

    #[test]
    fn test_miniset_place() {
        let miniset = lamps_miniset();
        let mut dungeon = [[Tile::Floor; DUNGEON_SIZE]; DUNGEON_SIZE];

        miniset.place(&mut dungeon, 10, 10);

        // C++ LAMPS replace: {129, 0}, {130, 128}
        assert_eq!(dungeon[10][10], Tile::Mega129);
        assert_eq!(dungeon[10][11], Tile::Floor); // wildcard (0) leaves tile unchanged
        assert_eq!(dungeon[11][10], Tile::Mega130);
        assert_eq!(dungeon[11][11], Tile::Mega128);
    }

    #[test]
    fn test_chamber_state_select_all_chambers() {
        let state = ChamberState {
            has_chamber_1: true,
            has_chamber_2: true,
            has_chamber_3: true,
        };

        // With all three chambers available the result must be one of the
        // three valid vertical-layout chamber centers.
        let mut rng = Rng::with_default_seed();
        let (x, y) = state.select_chamber(true, &mut rng);
        assert!(
            [(16, 2), (16, 16), (16, 30)].contains(&(x, y)),
            "select_chamber returned an invalid vertical chamber center ({x}, {y})"
        );
    }

    #[test]
    fn test_chamber_state_select_two_chambers() {
        let state = ChamberState {
            has_chamber_1: true,
            has_chamber_2: true,
            has_chamber_3: false,
        };

        // Chambers 1 & 2 only: horizontal layout centers are (2,16) or (16,16).
        let mut rng = Rng::with_default_seed();
        let (x, y) = state.select_chamber(false, &mut rng);
        assert!(
            [(2, 16), (16, 16)].contains(&(x, y)),
            "select_chamber returned an invalid horizontal chamber center ({x}, {y})"
        );
    }

    #[test]
    fn test_chamber_state_select_single_chamber() {
        let state = ChamberState {
            has_chamber_1: false,
            has_chamber_2: true,
            has_chamber_3: false,
        };

        // Single chamber always falls through to chamber 2 (center) regardless
        // of RNG state — this path never consumes the engine.
        let mut rng = Rng::with_default_seed();
        let seed_before = rng.get_seed();
        let (x, y) = state.select_chamber(true, &mut rng);
        assert_eq!((x, y), (16, 16)); // Always center chamber
        assert_eq!(
            rng.get_seed(),
            seed_before,
            "single-chamber path must not consume RNG state"
        );
    }

    #[test]
    fn test_cathedral_generator_init() {
        let mut gen = CathedralGenerator::new();
        gen.init_dungeon();

        // All tiles should be Dirt
        assert_eq!(gen.dungeon[0][0], Tile::Dirt);
        assert_eq!(gen.dungeon[20][20], Tile::Dirt);
        assert_eq!(gen.dungeon[39][39], Tile::Dirt);
    }

    #[test]
    fn test_cathedral_generator_place_miniset() {
        let mut gen = CathedralGenerator::new();
        gen.init_dungeon();

        // Fill a floor area with an HWall run so the scan-based C++
        // PlaceMiniSet (with the drlg1_quirk bias) finds the STAIRSUP pattern:
        // search rows [Floor..], [HWall..], [Floor..], [Floor..].
        for y in 14..20 {
            for x in 14..20 {
                gen.dungeon[y][x] = Tile::Floor;
            }
        }
        for x in 14..20 {
            gen.dungeon[15][x] = Tile::HWall;
        }

        let miniset = stairs_up_miniset();
        let placed = gen.place_miniset(&miniset, DUNGEON_SIZE * DUNGEON_SIZE, true);
        assert!(placed.is_some(), "stairs-up miniset should be placed");

        let (px, py) = placed.unwrap();
        assert!(px >= 13 && py >= 13, "drlg1 quirk biases placement past x/y=12");
        // C++ STAIRSUP replace: {0,66,6,0},{63,64,65,0},{0,67,68,0},{0,0,0,0}
        assert_eq!(gen.dungeon[py][px + 2], Tile::VWallEnd);
        assert_eq!(gen.dungeon[py + 1][px + 1], Tile::EntranceStairs);
    }

    #[test]
    fn test_default_instances() {
        let tile = Tile::default();
        assert_eq!(tile, Tile::Invalid);

        let state = ChamberState::default();
        assert!(!state.has_chamber_1);
        assert!(!state.has_chamber_2);
        assert!(!state.has_chamber_3);

        let gen = CathedralGenerator::default();
        assert_eq!(gen.dungeon[0][0], Tile::Invalid);
        assert!(!gen.vertical_layout);
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::new(5, 10, 20, 15);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 10);
        assert_eq!(rect.width, 20);
        assert_eq!(rect.height, 15);
    }

    #[test]
    fn test_map_room() {
        let mut gen = CathedralGenerator::new();
        let room = Rectangle::new(5, 5, 3, 3);

        gen.map_room(room);

        assert!(gen.dungeon_mask[5][5]);
        assert!(gen.dungeon_mask[7][7]);
        assert!(!gen.dungeon_mask[4][4]);
        assert!(!gen.dungeon_mask[8][8]);
    }

    #[test]
    fn test_check_room() {
        let mut gen = CathedralGenerator::new();
        let room1 = Rectangle::new(5, 5, 3, 3);

        assert!(gen.check_room(room1));

        gen.map_room(room1);

        // Overlapping room should fail
        let room2 = Rectangle::new(6, 6, 3, 3);
        assert!(!gen.check_room(room2));

        // Non-overlapping room should succeed
        let room3 = Rectangle::new(10, 10, 3, 3);
        assert!(gen.check_room(room3));
    }

    #[test]
    fn test_find_area() {
        let mut gen = CathedralGenerator::new();
        assert_eq!(gen.find_area(), 0);

        gen.map_room(Rectangle::new(5, 5, 3, 3));
        assert_eq!(gen.find_area(), 9); // 3x3 = 9 tiles

        gen.map_room(Rectangle::new(10, 10, 2, 2));
        assert_eq!(gen.find_area(), 13); // 9 + 4 = 13 tiles
    }

    #[test]
    fn test_make_dmt() {
        let mut gen = CathedralGenerator::new();

        // Create simple 3x3 room
        for y in 5..8 {
            for x in 5..8 {
                gen.dungeon_mask[y][x] = true;
            }
        }

        gen.make_dmt();

        // Interior should be Floor
        assert_eq!(gen.dungeon[5][5], Tile::Floor);
        assert_eq!(gen.dungeon[6][6], Tile::Floor);

        // Outside should be Dirt
        assert_eq!(gen.dungeon[0][0], Tile::Dirt);
        assert_eq!(gen.dungeon[9][9], Tile::Dirt);
    }

    #[test]
    fn test_first_room_basic() {
        let mut gen = CathedralGenerator::new();

        gen.first_room();

        // Should have at least some tiles mapped
        assert!(gen.find_area() > 0);

        // Should have set chamber state
        assert!(
            gen.chamber_state.has_chamber_1
                || gen.chamber_state.has_chamber_2
                || gen.chamber_state.has_chamber_3
        );
    }

    #[test]
    fn test_first_room_ensures_two_chambers() {
        let mut gen = CathedralGenerator::new();

        // The first-room logic guarantees at least two chambers: if chamber1 or
        // chamber3 is absent, chamber2 is forced on regardless of the RNG draw.
        // Use a fixed seed so the four FlipCoin draws are reproducible.
        gen.rng.set_seed(1);
        gen.first_room();

        // The hallway/chamber layout always reserves chamber 2 when either of
        // the side chambers is missing. At least two chambers must be present.
        let active = [
            gen.chamber_state.has_chamber_1,
            gen.chamber_state.has_chamber_2,
            gen.chamber_state.has_chamber_3,
        ]
        .iter()
        .filter(|&&b| b)
        .count();
        assert!(
            active >= 2,
            "first_room must keep at least two chambers active (got {active})"
        );
        // And specifically: if a side chamber is missing, chamber2 is forced on.
        if !gen.chamber_state.has_chamber_1 || !gen.chamber_state.has_chamber_3 {
            assert!(gen.chamber_state.has_chamber_2);
        }
    }

    #[test]
    fn test_generate_full_dungeon() {
        use crate::levels::types::DungeonType;

        let mut gen = CathedralGenerator::new();

        gen.generate(DungeonType::Cathedral, 12345);

        // Should have generated room layout
        assert!(gen.find_area() > 0);

        // Should have some floor tiles
        let mut floor_count = 0;
        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                if gen.dungeon[y][x] == Tile::Floor {
                    floor_count += 1;
                }
            }
        }
        assert!(floor_count > 0);
    }

    #[test]
    fn test_fill_floor() {
        let mut gen = CathedralGenerator::new();

        // Create some floor tiles
        for y in 5..10 {
            for x in 5..10 {
                gen.dungeon[y][x] = Tile::Floor;
            }
        }

        // C++ FillFloor: rv==0 stays Floor, rv==1 -> Floor22, rv==2 -> Floor23.
        gen.fill_floor();

        // Every input Floor tile must now be Floor, Floor22 or Floor23.
        for y in 5..10 {
            for x in 5..10 {
                assert!(
                    matches!(gen.dungeon[y][x], Tile::Floor | Tile::Floor22 | Tile::Floor23),
                    "fill_floor must only emit Floor/Floor22/Floor23, got {:?}",
                    gen.dungeon[y][x]
                );
            }
        }
    }

    #[test]
    fn test_complete_generation_pipeline() {
        use crate::levels::types::DungeonType;

        let mut gen = CathedralGenerator::new();

        gen.generate(DungeonType::Cathedral, 42);

        // Verify dungeon was generated
        assert!(gen.find_area() > 0, "Dungeon should have mapped area");

        // Count different tile types
        let mut floor_count = 0;
        let mut wall_count = 0;
        let mut dirt_count = 0;

        for y in 0..DUNGEON_SIZE {
            for x in 0..DUNGEON_SIZE {
                match gen.dungeon[y][x] {
                    Tile::Floor => floor_count += 1,
                    Tile::HWall | Tile::VWall => wall_count += 1,
                    Tile::Dirt => dirt_count += 1,
                    _ => {}
                }
            }
        }

        assert!(floor_count > 0, "Should have floor tiles");
        assert!(dirt_count > 0, "Should have dirt tiles");
        // Walls might be 0 if random placement didn't trigger
    }

    /// Two generations with the same seed must produce identical dungeon tiles.
    ///
    /// This is the core determinism contract (save/replay compatibility): the
    /// Cathedral generator must be bit-for-bit reproducible for a given seed.
    /// It depends on the Borland LCG (`Rng`) being wired in correctly
    /// end-to-end through `first_room`/`generate_room`/`add_wall`/etc.
    #[test]
    fn test_generate_is_deterministic_for_same_seed() {
        let seed = 0x24681357u32;

        let mut a = CathedralGenerator::new();
        a.generate(DungeonType::Cathedral, seed);

        let mut b = CathedralGenerator::new();
        b.generate(DungeonType::Cathedral, seed);

        assert_eq!(a.dungeon, b.dungeon, "same seed must yield identical tiles");
        assert_eq!(
            a.dungeon_mask, b.dungeon_mask,
            "same seed must yield identical dungeon_mask"
        );
        assert_eq!(
            a.vertical_layout, b.vertical_layout,
            "same seed must yield identical vertical_layout"
        );
        assert_eq!(
            a.chamber_state.has_chamber_1,
            b.chamber_state.has_chamber_1
        );
    }

    /// Different seeds must drive the RNG into different end states.
    ///
    /// Guards against the generator accidentally ignoring its seed.
    #[test]
    fn test_different_seeds_drive_rng_differently() {
        let mut a = CathedralGenerator::new();
        a.generate(DungeonType::Cathedral, 1);

        let mut b = CathedralGenerator::new();
        b.generate(DungeonType::Cathedral, 2);

        assert_ne!(
            a.rng.get_seed(),
            b.rng.get_seed(),
            "different seeds should leave the engine in different states"
        );

        // Sanity: a seed must actually have been consumed (default seed is 0).
        assert_ne!(
            a.rng.get_seed(),
            Rng::with_default_seed().get_seed(),
            "generate() must advance the RNG past its initial state"
        );
    }
}
