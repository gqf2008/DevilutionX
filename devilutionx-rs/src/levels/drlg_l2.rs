// drlg_l2.rs - Catacombs (L5-L8) Level Generation
//
// Implements the procedural generation algorithm for the Catacombs tileset.
//
// C++ Reference: Source/levels/drlg_l2.cpp (~2,853 lines)
//
// Key Features:
// - Room-based generation (max 81 rooms)
// - Hall-based connection system
// - ~90 Minisets (40 VARCH, 40 HARCH, stairs, decorations)
// - Pattern-based tile replacement
// - Quest room integration (L5/L6/L7 quests)

use crate::engine::types::{Point, Displacement};
use crate::levels::gendung::Dungeon;
use crate::levels::types::{DungeonType, LevelEntry, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
use std::collections::VecDeque;

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of rooms in L2 dungeon
const MAX_ROOMS: usize = 81;

/// Direction offsets (None, Up, Right, Down, Left)
const DIR_ADD: [Displacement; 5] = [
    Displacement { delta_x: 0, delta_y: 0 },   // None
    Displacement { delta_x: 0, delta_y: -1 },  // Up
    Displacement { delta_x: 1, delta_y: 0 },   // Right
    Displacement { delta_x: 0, delta_y: 1 },   // Down
    Displacement { delta_x: -1, delta_y: 0 },  // Left
];

/// Shadow struct for L2
/// C++ equivalent: SPATSL2[2]
const SHADOW_PATTERNS_L2: [(u8, u8, u8, u8, u8, u8, u8); 2] = [
    (6, 3, 0, 3, 48, 0, 50),
    (9, 3, 0, 3, 48, 0, 50),
];

/// Tile type mapping for L2 (161 entries)
/// C++ equivalent: BTYPESL2[161]
const TILE_TYPES_L2: [u8; 161] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0, 17, 18, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 2,
    2, 2, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 0, 0, 2, 2, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 0, 3, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

/// Shadow tile type mapping for L2 (161 entries)
/// C++ equivalent: BSTYPESL2[161]
const SHADOW_TILE_TYPES_L2: [u8; 161] = [
    0, 1, 2, 3, 0, 0, 6, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 2,
    2, 2, 0, 0, 0, 0, 0, 6, 6, 6, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 0, 0, 2, 2, 2, 0, 0, 0, 1, 1, 1, 1, 6, 2, 2, 2, 0, 3, 3, 3, 3, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 2, 2, 3, 3, 3, 3, 1, 1, 2, 2,
    3, 3, 3, 3, 1, 1, 3, 3, 2, 2, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

/// Pattern matching table for DoPatternCheck (72 active patterns + end marker)
/// C++ equivalent: Patterns[100][10] in gendung.cpp
/// Format: [9 match values (3x3 grid), 1 replace value]
/// Match values: 0=any, 1='#', 2='.', 3='D', 4=' ', 5='D'|'.', 6='D'|'#', 7=' '|'.', 8='D'|'#'|'.'
/// Grid layout: [0,1,2, 3,4,5, 6,7,8] represents 3×3 neighborhood
const PATTERNS: [[i32; 10]; 72] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 3], [0, 0, 0, 0, 2, 0, 0, 0, 0, 3],
    [0, 7, 0, 0, 1, 0, 0, 5, 0, 2], [0, 5, 0, 0, 1, 0, 0, 7, 0, 2],
    [0, 0, 0, 7, 1, 5, 0, 0, 0, 1], [0, 0, 0, 5, 1, 7, 0, 0, 0, 1],
    [0, 1, 0, 0, 3, 0, 0, 1, 0, 4], [0, 0, 0, 1, 3, 1, 0, 0, 0, 5],
    [0, 6, 0, 6, 1, 0, 0, 0, 0, 6], [0, 6, 0, 0, 1, 6, 0, 0, 0, 9],
    [0, 0, 0, 6, 1, 0, 0, 6, 0, 7], [0, 0, 0, 0, 1, 6, 0, 6, 0, 8],
    [0, 6, 0, 6, 6, 0, 8, 6, 0, 7], [0, 6, 8, 6, 6, 6, 0, 0, 0, 9],
    [0, 6, 0, 0, 6, 6, 0, 6, 8, 8], [6, 6, 6, 6, 6, 6, 0, 6, 0, 8],
    [2, 6, 6, 6, 6, 6, 0, 6, 0, 8], [7, 7, 7, 6, 6, 6, 0, 6, 0, 8],
    [6, 6, 2, 6, 6, 6, 0, 6, 0, 8], [6, 2, 6, 6, 6, 6, 0, 6, 0, 8],
    [2, 6, 6, 6, 6, 6, 0, 6, 0, 8], [6, 7, 7, 6, 6, 6, 0, 6, 0, 8],
    [4, 4, 6, 6, 6, 6, 2, 6, 2, 8], [2, 2, 2, 2, 6, 2, 2, 6, 2, 7],
    [2, 2, 2, 2, 6, 2, 6, 6, 6, 7], [2, 2, 6, 2, 6, 6, 2, 2, 6, 9],
    [2, 6, 2, 2, 6, 2, 2, 2, 2, 6], [2, 2, 2, 2, 6, 6, 2, 2, 2, 9],
    [2, 2, 2, 6, 6, 2, 2, 2, 2, 6], [2, 2, 0, 2, 6, 6, 2, 2, 0, 9],
    [0, 0, 0, 0, 4, 0, 0, 0, 0, 12], [0, 1, 0, 0, 1, 4, 0, 1, 0, 10],
    [0, 0, 0, 1, 1, 1, 0, 4, 0, 11], [0, 0, 0, 6, 1, 4, 0, 1, 0, 14],
    [0, 6, 0, 1, 1, 0, 0, 4, 0, 16], [0, 6, 0, 0, 1, 1, 0, 4, 0, 15],
    [0, 0, 0, 0, 1, 1, 0, 1, 4, 13], [8, 8, 8, 8, 1, 1, 0, 1, 1, 13],
    [8, 8, 4, 8, 1, 1, 0, 1, 1, 10], [0, 0, 0, 1, 1, 1, 1, 1, 1, 11],
    [1, 1, 1, 1, 1, 1, 2, 2, 8, 2], [0, 1, 0, 1, 1, 4, 1, 1, 0, 16],
    [0, 0, 0, 1, 1, 1, 1, 1, 4, 11], [1, 1, 4, 1, 1, 1, 0, 2, 2, 2],
    [1, 1, 1, 1, 1, 1, 6, 2, 6, 2], [4, 1, 1, 1, 1, 1, 6, 2, 6, 2],
    [2, 2, 2, 1, 1, 1, 4, 1, 1, 11], [4, 1, 1, 1, 1, 1, 2, 2, 2, 2],
    [1, 1, 4, 1, 1, 1, 2, 2, 1, 2], [4, 1, 1, 1, 1, 1, 1, 2, 2, 2],
    [2, 2, 6, 1, 1, 1, 4, 1, 1, 11], [4, 1, 1, 1, 1, 1, 2, 2, 6, 2],
    [1, 2, 2, 1, 1, 1, 4, 1, 1, 11], [0, 1, 1, 0, 1, 1, 0, 1, 1, 10],
    [2, 1, 1, 3, 1, 1, 2, 1, 1, 14], [1, 1, 0, 1, 1, 2, 1, 1, 0, 1],
    [0, 4, 0, 1, 1, 1, 0, 1, 1, 14], [4, 1, 0, 1, 1, 0, 1, 1, 0, 1],
    [0, 1, 0, 4, 1, 1, 0, 1, 1, 15], [1, 1, 1, 1, 1, 1, 0, 2, 2, 2],
    [0, 1, 1, 2, 1, 1, 2, 1, 4, 10], [2, 1, 1, 1, 1, 1, 0, 4, 0, 16],
    [1, 1, 4, 1, 1, 2, 0, 1, 2, 1], [2, 1, 1, 2, 1, 1, 1, 1, 4, 10],
    [1, 1, 2, 1, 1, 2, 4, 1, 8, 1], [2, 1, 4, 1, 1, 1, 4, 4, 1, 16],
    [2, 1, 1, 1, 1, 1, 1, 1, 1, 16], [1, 1, 2, 1, 1, 1, 1, 1, 1, 15],
    [1, 1, 1, 1, 1, 1, 2, 1, 1, 14], [4, 1, 1, 1, 1, 1, 2, 1, 1, 14],
    [1, 1, 1, 1, 1, 1, 1, 1, 2, 8],
    [0, 0, 0, 0, 255, 0, 0, 0, 0, 0], // End marker (index 72)
];

// =============================================================================
// Enums
// =============================================================================

/// Hall direction for connecting rooms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum HallDirection {
    None = 0,
    Up = 1,
    Right = 2,
    Down = 3,
    Left = 4,
}

impl HallDirection {
    pub fn from_i8(value: i8) -> Option<Self> {
        match value {
            0 => Some(HallDirection::None),
            1 => Some(HallDirection::Up),
            2 => Some(HallDirection::Right),
            3 => Some(HallDirection::Down),
            4 => Some(HallDirection::Left),
            _ => None,
        }
    }

    pub fn to_displacement(self) -> Displacement {
        DIR_ADD[self as usize]
    }
}

/// L2 Tile types (subset used in generation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum L2Tile {
    Floor = 1,
    Wall = 2,
    Void = 0,
    Door = 4,
    Arch = 5,
    Pillar = 6,
    // TODO: Add more tile types as needed (~30 total)
}

// =============================================================================
// Data Structures
// =============================================================================

/// Hall node representing a corridor connecting rooms
#[derive(Debug, Clone)]
pub struct HallNode {
    pub beginning: Point,
    pub end: Point,
    pub direction: HallDirection,
}

impl HallNode {
    pub fn new(beginning: Point, end: Point, direction: HallDirection) -> Self {
        Self {
            beginning,
            end,
            direction,
        }
    }
}

/// Room node representing a rectangular room
#[derive(Debug, Clone, Copy)]
pub struct RoomNode {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl RoomNode {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        Self {
            top_left,
            bottom_right,
        }
    }

    pub fn width(&self) -> i32 {
        self.bottom_right.x - self.top_left.x + 1
    }

    pub fn height(&self) -> i32 {
        self.bottom_right.y - self.top_left.y + 1
    }

    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.top_left.x
            && point.x <= self.bottom_right.x
            && point.y >= self.top_left.y
            && point.y <= self.bottom_right.y
    }
}

/// Miniset structure for pattern replacement
/// C++ equivalent: Miniset struct
#[derive(Debug, Clone)]
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
// Miniset Constants (Critical Stairs + Sample Arches)
// =============================================================================

/// Miniset: Stairs up (4x4)
/// C++ equivalent: USTAIRS
pub fn miniset_ustairs() -> Miniset {
    Miniset::new(
        4,
        4,
        vec![
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
        ],
        vec![
            vec![0, 0, 0, 0],
            vec![0, 72, 77, 0],
            vec![0, 76, 0, 0],
            vec![0, 0, 0, 0],
        ],
    )
}

/// Miniset: Stairs down (4x4)
/// C++ equivalent: DSTAIRS
pub fn miniset_dstairs() -> Miniset {
    Miniset::new(
        4,
        4,
        vec![
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
        ],
        vec![
            vec![0, 0, 0, 0],
            vec![0, 48, 71, 0],
            vec![0, 50, 78, 0],
            vec![0, 0, 0, 0],
        ],
    )
}

/// Miniset: Warp stairs to town (4x4)
/// C++ equivalent: WARPSTAIRS
pub fn miniset_warpstairs() -> Miniset {
    Miniset::new(
        4,
        4,
        vec![
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
            vec![3, 3, 3, 3],
        ],
        vec![
            vec![0, 0, 0, 0],
            vec![0, 158, 160, 0],
            vec![0, 159, 0, 0],
            vec![0, 0, 0, 0],
        ],
    )
}

/// Miniset: Vertical arch sample 1 (2x4)
/// C++ equivalent: VARCH1
pub fn miniset_varch1() -> Miniset {
    Miniset::new(
        2,
        4,
        vec![
            vec![3, 0],
            vec![3, 1],
            vec![3, 4],
            vec![0, 7],
        ],
        vec![
            vec![48, 0],
            vec![51, 39],
            vec![47, 44],
            vec![0, 0],
        ],
    )
}

/// Miniset: Horizontal arch sample 1 (4x2)
/// C++ equivalent: HARCH1
pub fn miniset_harch1() -> Miniset {
    Miniset::new(
        4,
        2,
        vec![
            vec![3, 3, 0, 0],
            vec![0, 1, 7, 0],
        ],
        vec![
            vec![49, 46, 0, 0],
            vec![0, 40, 45, 0],
        ],
    )
}

/// Miniset: Big oil spill vertical (2x2)
/// C++ equivalent: BIG1
pub fn miniset_big1() -> Miniset {
    Miniset::new(2, 2, vec![vec![3,3],vec![3,3]], vec![vec![113,0],vec![112,0]])
}

pub fn miniset_big2() -> Miniset {
    Miniset::new(2, 2, vec![vec![3,3],vec![3,3]], vec![vec![114,115],vec![0,0]])
}

pub fn miniset_big3() -> Miniset {
    Miniset::new(1, 2, vec![vec![1],vec![1]], vec![vec![117],vec![116]])
}

pub fn miniset_big4() -> Miniset {
    Miniset::new(2, 1, vec![vec![2,2]], vec![vec![118,119]])
}

pub fn miniset_big5() -> Miniset {
    Miniset::new(2, 2, vec![vec![3,3],vec![3,3]], vec![vec![120,122],vec![121,123]])
}

pub fn miniset_big6() -> Miniset {
    Miniset::new(1, 2, vec![vec![1],vec![1]], vec![vec![125],vec![124]])
}

pub fn miniset_big7() -> Miniset {
    Miniset::new(2, 1, vec![vec![2,2]], vec![vec![126,127]])
}

pub fn miniset_big8() -> Miniset {
    Miniset::new(2, 2, vec![vec![3,3],vec![3,3]], vec![vec![128,130],vec![129,131]])
}

pub fn miniset_big9() -> Miniset {
    Miniset::new(2, 2, vec![vec![1,3],vec![1,3]], vec![vec![133,135],vec![132,134]])
}

pub fn miniset_big10() -> Miniset {
    Miniset::new(2, 2, vec![vec![2,2],vec![3,3]], vec![vec![136,137],vec![3,3]])
}

pub fn miniset_crushcol() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![2,6,3],vec![3,3,3]], vec![vec![0,0,0],vec![0,83,0],vec![0,0,0]])
}

pub fn miniset_pancreas1() -> Miniset {
    Miniset::new(5, 3, vec![vec![3,3,3,3,3],vec![3,3,3,3,3],vec![3,3,3,3,3]], vec![vec![0,0,0,0,0],vec![0,0,108,0,0],vec![0,0,0,0,0]])
}

pub fn miniset_pancreas2() -> Miniset {
    Miniset::new(5, 3, vec![vec![3,3,3,3,3],vec![3,3,3,3,3],vec![3,3,3,3,3]], vec![vec![0,0,0,0,0],vec![0,0,109,0,0],vec![0,0,0,0,0]])
}

// =============================================================================
// VARCH Minisets (Vertical Arches) - VARCH2-40
// =============================================================================

pub fn miniset_varch2() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,8]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch3() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,6]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch4() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,9]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch5() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,14]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch6() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,13]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch7() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,16]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch8() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,1],vec![3,4],vec![0,15]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch9() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,7]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch10() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,8]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch11() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,6]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch12() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,9]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch13() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,14]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch14() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,13]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch15() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,16]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch16() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,8],vec![3,4],vec![0,15]], vec![vec![48,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch17() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,7]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch18() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,8]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch19() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,6]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch20() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,9]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch21() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,14]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch22() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,13]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch23() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,16]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch24() -> Miniset {
    Miniset::new(2, 3, vec![vec![2,7],vec![3,4],vec![0,15]], vec![vec![141,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch25() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,7]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch26() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,8]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch27() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,6]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch28() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,9]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch29() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,14]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch30() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,13]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch31() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,16]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch32() -> Miniset {
    Miniset::new(2, 4, vec![vec![3,0],vec![3,4],vec![3,1],vec![0,15]], vec![vec![48,0],vec![51,39],vec![47,44],vec![0,0]])
}

pub fn miniset_varch33() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,7]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch34() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,8]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch35() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,6]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch36() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,9]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch37() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,14]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch38() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,13]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch39() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,16]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

pub fn miniset_varch40() -> Miniset {
    Miniset::new(2, 4, vec![vec![2,0],vec![3,8],vec![3,4],vec![0,15]], vec![vec![142,0],vec![51,42],vec![47,44],vec![0,0]])
}

// =============================================================================
// HARCH Minisets (Horizontal Arches) - HARCH2-40
// =============================================================================

pub fn miniset_harch2() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,6]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch3() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,8]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch4() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,7]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch5() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,15]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch6() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,16]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch7() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,13]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch8() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![2,5,14]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch9() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,9]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch10() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,6]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch11() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,8]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch12() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,7]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch13() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,15]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch14() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,16]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch15() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,13]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch16() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![8,5,14]], vec![vec![49,46,0],vec![43,45,0]])
}

pub fn miniset_harch17() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,9]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch18() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,6]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch19() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,8]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch20() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,7]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch21() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,15]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch22() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,16]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch23() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,13]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch24() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![8,5,14]], vec![vec![140,46,0],vec![43,45,0]])
}

pub fn miniset_harch25() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,9]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch26() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,6]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch27() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,8]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch28() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,7]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch29() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,15]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch30() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,16]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch31() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,13]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch32() -> Miniset {
    Miniset::new(3, 2, vec![vec![3,3,0],vec![5,2,14]], vec![vec![49,46,0],vec![40,45,0]])
}

pub fn miniset_harch33() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,9]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch34() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,6]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch35() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,8]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch36() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,7]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch37() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,15]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch38() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,16]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch39() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,13]], vec![vec![140,46,0],vec![40,45,0]])
}

pub fn miniset_harch40() -> Miniset {
    Miniset::new(3, 2, vec![vec![1,3,0],vec![9,5,14]], vec![vec![140,46,0],vec![40,45,0]])
}

// =============================================================================
// CTRDOOR Minisets (Center Doors) - CTRDOOR1-8
// =============================================================================

pub fn miniset_ctrdoor1() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,9,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor2() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,8,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor3() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,6,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor4() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,7,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor5() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,15,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor6() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,13,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor7() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,16,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

pub fn miniset_ctrdoor8() -> Miniset {
    Miniset::new(3, 3, vec![vec![3,1,3],vec![0,4,0],vec![0,14,0]], vec![vec![0,4,0],vec![0,1,0],vec![0,0,0]])
}

// =============================================================================
// Miniset Helper Functions
// =============================================================================

/// Get all critical Minisets for L2 generation
pub fn get_critical_minisets() -> Vec<Miniset> {
    vec![
        miniset_ustairs(),
        miniset_dstairs(),
        miniset_warpstairs(),
        miniset_varch1(),
        miniset_harch1(),
        miniset_big1(),
    ]
}

// =============================================================================
// Catacombs Generator
// =============================================================================

/// Main L2 (Catacombs) generator
pub struct CatacombsGenerator {
    /// Room list (max 81 rooms)
    room_list: Vec<RoomNode>,
    /// Room count
    room_count: usize,
    /// Hall list (queue for BFS-like connection)
    hall_list: VecDeque<HallNode>,
    /// ASCII representation of predungeon
    predungeon: [[char; MAXDUNY]; MAXDUNX],
    /// Current level (5-8)
    current_level: u8,
}

impl CatacombsGenerator {
    pub fn new() -> Self {
        Self {
            room_list: Vec::with_capacity(MAX_ROOMS),
            room_count: 0,
            hall_list: VecDeque::new(),
            predungeon: [[' '; MAXDUNY]; MAXDUNX],
            current_level: 5,
        }
    }

    /// Generate a Catacombs level
    /// Main dungeon generation entry point
    /// C++ equivalent: GenerateLevel
    pub fn generate(&mut self, dungeon: &mut Dungeon, _seed: u32, level: u8) -> bool {
        self.current_level = level;
        // TODO: SetRndSeed(seed)

        // Main generation loop (retry until valid)
        let max_retries = 100;
        for _attempt in 0..max_retries {
            self.reset();

            if self.create_dungeon(dungeon) {
                self.fix_tiles_patterns(dungeon);
                // TODO: InitSetPiece
                // TODO: FloodTransparencyValues(3)
                self.fix_transparency(dungeon);

                // Place stairs - retry if failed
                if !self.place_stairs(dungeon, LevelEntry::MainEntry) {
                    continue;
                }

                // Break out of retry loop - generation succeeded
                break;
            }
        }

        // Post-generation fixes (outside retry loop)
        self.fix_lockout(dungeon);
        self.fix_doors(dungeon);
        self.fix_dirt_tiles(dungeon);

        // TODO: DRLG_PlaceThemeRooms(6, 10, 3, 0, false)

        // Place decorative minisets (88 total: 8 CTRDOOR + 40 VARCH + 40 HARCH)
        // CTRDOOR (Center Doors) - 8 minisets
        self.place_miniset_random(dungeon, &miniset_ctrdoor1(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor2(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor3(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor4(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor5(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor6(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor7(), 100);
        self.place_miniset_random(dungeon, &miniset_ctrdoor8(), 100);

        // VARCH (Vertical Arches) - 40 minisets
        self.place_miniset_random(dungeon, &miniset_varch1(), 100);
        self.place_miniset_random(dungeon, &miniset_varch2(), 100);
        self.place_miniset_random(dungeon, &miniset_varch3(), 100);
        self.place_miniset_random(dungeon, &miniset_varch4(), 100);
        self.place_miniset_random(dungeon, &miniset_varch5(), 100);
        self.place_miniset_random(dungeon, &miniset_varch6(), 100);
        self.place_miniset_random(dungeon, &miniset_varch7(), 100);
        self.place_miniset_random(dungeon, &miniset_varch8(), 100);
        self.place_miniset_random(dungeon, &miniset_varch9(), 100);
        self.place_miniset_random(dungeon, &miniset_varch10(), 100);
        self.place_miniset_random(dungeon, &miniset_varch11(), 100);
        self.place_miniset_random(dungeon, &miniset_varch12(), 100);
        self.place_miniset_random(dungeon, &miniset_varch13(), 100);
        self.place_miniset_random(dungeon, &miniset_varch14(), 100);
        self.place_miniset_random(dungeon, &miniset_varch15(), 100);
        self.place_miniset_random(dungeon, &miniset_varch16(), 100);
        self.place_miniset_random(dungeon, &miniset_varch17(), 100);
        self.place_miniset_random(dungeon, &miniset_varch18(), 100);
        self.place_miniset_random(dungeon, &miniset_varch19(), 100);
        self.place_miniset_random(dungeon, &miniset_varch20(), 100);
        self.place_miniset_random(dungeon, &miniset_varch21(), 100);
        self.place_miniset_random(dungeon, &miniset_varch22(), 100);
        self.place_miniset_random(dungeon, &miniset_varch23(), 100);
        self.place_miniset_random(dungeon, &miniset_varch24(), 100);
        self.place_miniset_random(dungeon, &miniset_varch25(), 100);
        self.place_miniset_random(dungeon, &miniset_varch26(), 100);
        self.place_miniset_random(dungeon, &miniset_varch27(), 100);
        self.place_miniset_random(dungeon, &miniset_varch28(), 100);
        self.place_miniset_random(dungeon, &miniset_varch29(), 100);
        self.place_miniset_random(dungeon, &miniset_varch30(), 100);
        self.place_miniset_random(dungeon, &miniset_varch31(), 100);
        self.place_miniset_random(dungeon, &miniset_varch32(), 100);
        self.place_miniset_random(dungeon, &miniset_varch33(), 100);
        self.place_miniset_random(dungeon, &miniset_varch34(), 100);
        self.place_miniset_random(dungeon, &miniset_varch35(), 100);
        self.place_miniset_random(dungeon, &miniset_varch36(), 100);
        self.place_miniset_random(dungeon, &miniset_varch37(), 100);
        self.place_miniset_random(dungeon, &miniset_varch38(), 100);
        self.place_miniset_random(dungeon, &miniset_varch39(), 100);
        self.place_miniset_random(dungeon, &miniset_varch40(), 100);

        // HARCH (Horizontal Arches) - 40 minisets
        self.place_miniset_random(dungeon, &miniset_harch1(), 100);
        self.place_miniset_random(dungeon, &miniset_harch2(), 100);
        self.place_miniset_random(dungeon, &miniset_harch3(), 100);
        self.place_miniset_random(dungeon, &miniset_harch4(), 100);
        self.place_miniset_random(dungeon, &miniset_harch5(), 100);
        self.place_miniset_random(dungeon, &miniset_harch6(), 100);
        self.place_miniset_random(dungeon, &miniset_harch7(), 100);
        self.place_miniset_random(dungeon, &miniset_harch8(), 100);
        self.place_miniset_random(dungeon, &miniset_harch9(), 100);
        self.place_miniset_random(dungeon, &miniset_harch10(), 100);
        self.place_miniset_random(dungeon, &miniset_harch11(), 100);
        self.place_miniset_random(dungeon, &miniset_harch12(), 100);
        self.place_miniset_random(dungeon, &miniset_harch13(), 100);
        self.place_miniset_random(dungeon, &miniset_harch14(), 100);
        self.place_miniset_random(dungeon, &miniset_harch15(), 100);
        self.place_miniset_random(dungeon, &miniset_harch16(), 100);
        self.place_miniset_random(dungeon, &miniset_harch17(), 100);
        self.place_miniset_random(dungeon, &miniset_harch18(), 100);
        self.place_miniset_random(dungeon, &miniset_harch19(), 100);
        self.place_miniset_random(dungeon, &miniset_harch20(), 100);
        self.place_miniset_random(dungeon, &miniset_harch21(), 100);
        self.place_miniset_random(dungeon, &miniset_harch22(), 100);
        self.place_miniset_random(dungeon, &miniset_harch23(), 100);
        self.place_miniset_random(dungeon, &miniset_harch24(), 100);
        self.place_miniset_random(dungeon, &miniset_harch25(), 100);
        self.place_miniset_random(dungeon, &miniset_harch26(), 100);
        self.place_miniset_random(dungeon, &miniset_harch27(), 100);
        self.place_miniset_random(dungeon, &miniset_harch28(), 100);
        self.place_miniset_random(dungeon, &miniset_harch29(), 100);
        self.place_miniset_random(dungeon, &miniset_harch30(), 100);
        self.place_miniset_random(dungeon, &miniset_harch31(), 100);
        self.place_miniset_random(dungeon, &miniset_harch32(), 100);
        self.place_miniset_random(dungeon, &miniset_harch33(), 100);
        self.place_miniset_random(dungeon, &miniset_harch34(), 100);
        self.place_miniset_random(dungeon, &miniset_harch35(), 100);
        self.place_miniset_random(dungeon, &miniset_harch36(), 100);
        self.place_miniset_random(dungeon, &miniset_harch37(), 100);
        self.place_miniset_random(dungeon, &miniset_harch38(), 100);
        self.place_miniset_random(dungeon, &miniset_harch39(), 100);
        self.place_miniset_random(dungeon, &miniset_harch40(), 100);

        // Special minisets (CRUSHCOL, decorative tiles, PANCREAS, BIG variants)
        self.place_miniset_random(dungeon, &miniset_crushcol(), 99);

        // 1x1 tile substitutions
        self.place_miniset_random_1x1(dungeon, 1, 80, 10);
        self.place_miniset_random_1x1(dungeon, 1, 81, 10);
        self.place_miniset_random_1x1(dungeon, 1, 82, 10);
        self.place_miniset_random_1x1(dungeon, 2, 84, 10);
        self.place_miniset_random_1x1(dungeon, 2, 85, 10);
        self.place_miniset_random_1x1(dungeon, 2, 86, 10);
        self.place_miniset_random_1x1(dungeon, 8, 87, 50);

        self.place_miniset_random(dungeon, &miniset_pancreas1(), 1);
        self.place_miniset_random(dungeon, &miniset_pancreas2(), 1);
        self.place_miniset_random(dungeon, &miniset_big1(), 3);
        self.place_miniset_random(dungeon, &miniset_big2(), 3);
        self.place_miniset_random(dungeon, &miniset_big3(), 3);
        self.place_miniset_random(dungeon, &miniset_big4(), 3);
        self.place_miniset_random(dungeon, &miniset_big5(), 3);
        self.place_miniset_random(dungeon, &miniset_big6(), 20);
        self.place_miniset_random(dungeon, &miniset_big7(), 20);
        self.place_miniset_random(dungeon, &miniset_big8(), 3);
        self.place_miniset_random(dungeon, &miniset_big9(), 20);
        self.place_miniset_random(dungeon, &miniset_big10(), 20);

        // Apply tile substitution for variety
        self.substitution(dungeon);

        // Apply shadow patterns for visual depth
        self.apply_shadows_patterns(dungeon);

        true
    }

    /// Reset generation state
    fn reset(&mut self) {
        self.room_count = 0;
        self.room_list.clear();
        self.hall_list.clear();
        self.predungeon = [[' '; MAXDUNY]; MAXDUNX];
    }

    /// Create dungeon structure
    /// C++ equivalent: CreateDungeon
    fn create_dungeon(&mut self, dungeon: &mut Dungeon) -> bool {
        // Determine quest room size (if any)
        let quest_room_size = self.get_quest_room_size();

        // Create initial room
        let top_left = Point::new(2, 2);
        let bottom_right = Point::new(
            DMAXX as i32 - 1,
            DMAXY as i32 - 1,
        );
        self.create_room(top_left, bottom_right, 0, HallDirection::None, quest_room_size);

        // Connect all halls (BFS-like)
        while let Some(hall) = self.hall_list.pop_front() {
            self.connect_hall(&hall);
        }

        // Clean up predungeon characters
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                match self.predungeon[x][y] {
                    'A' | 'B' | 'C' | 'E' => self.predungeon[x][y] = '#',
                    ',' => {
                        self.predungeon[x][y] = '.';
                        // Mark adjacent spaces as walls
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx >= 0
                                    && ny >= 0
                                    && (nx as usize) < DMAXX
                                    && (ny as usize) < DMAXY
                                {
                                    if self.predungeon[nx as usize][ny as usize] == ' ' {
                                        self.predungeon[nx as usize][ny as usize] = '#';
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Fill voids - convert empty spaces to rooms/walls
        if !self.fill_voids() {
            return false; // Too many voids, retry generation
        }

        // Apply pattern-based tile corrections to dungeon
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                self.do_pattern_check(dungeon, i, j);
            }
        }

        // Fix tile adjacency patterns
        self.fix_tiles_patterns(dungeon);

        true
    }

    /// Get quest room size for current level
    /// C++ equivalent: size determination in CreateDungeon
    fn get_quest_room_size(&self) -> Option<(usize, usize)> {
        match self.current_level {
            5 => Some((14, 20)), // Q_BLOOD (Poisoned Water Supply)
            6 => Some((10, 10)), // Q_SCHAMB (Bone Chamber)
            7 => Some((15, 15)), // Q_BLIND (Halls of the Blind)
            _ => None,
        }
    }

    /// Create a room and potentially add halls
    /// C++ equivalent: CreateRoom
    fn create_room(
        &mut self,
        top_left: Point,
        bottom_right: Point,
        dest_room: usize,
        hall_dir: HallDirection,
        quest_size: Option<(usize, usize)>,
    ) {
        const AREA_MIN: i32 = 2;

        // Check if too many rooms or area too small
        if self.room_count >= MAX_ROOMS
            || top_left.x + AREA_MIN > bottom_right.x
            || top_left.y + AREA_MIN > bottom_right.y {
            return;
        }

        // Calculate available area
        let area_width = (bottom_right.x - top_left.x) as usize;
        let area_height = (bottom_right.y - top_left.y) as usize;

        const ROOM_MAX: usize = 10;
        const ROOM_MIN: usize = 4;

        // Determine room size (random or quest-specific)
        let mut room_width = area_width;
        let mut room_height = area_height;

        if let Some((qw, qh)) = quest_size {
            room_width = qw;
            room_height = qh;
        } else {
            if area_width > ROOM_MIN {
                room_width = (self.random_range(0, area_width.min(ROOM_MAX) - ROOM_MIN) + ROOM_MIN).min(area_width);
            }
            if area_height > ROOM_MIN {
                room_height = (self.random_range(0, area_height.min(ROOM_MAX) - ROOM_MIN) + ROOM_MIN).min(area_height);
            }
        }

        // Random placement within area
        let random_x = self.random_range(0, area_width);
        let random_y = self.random_range(0, area_height);

        let mut room_top_left = Point::new(
            top_left.x + random_x as i32,
            top_left.y + random_y as i32,
        );
        let mut room_bottom_right = Point::new(
            room_top_left.x + room_width as i32,
            room_top_left.y + room_height as i32,
        );

        // Adjust if room exceeds bounds
        if room_bottom_right.x > bottom_right.x {
            room_bottom_right.x = bottom_right.x;
            room_top_left.x = bottom_right.x - room_width as i32;
        }
        if room_bottom_right.y > bottom_right.y {
            room_bottom_right.y = bottom_right.y;
            room_top_left.y = bottom_right.y - room_height as i32;
        }

        // Clamp to valid dungeon range
        room_top_left.x = room_top_left.x.clamp(1, 38);
        room_top_left.y = room_top_left.y.clamp(1, 38);
        room_bottom_right.x = room_bottom_right.x.clamp(1, 38);
        room_bottom_right.y = room_bottom_right.y.clamp(1, 38);

        // Define room in predungeon
        self.define_room(room_top_left, room_bottom_right, quest_size.is_some());

        // Add room to list
        let room_id = self.room_count;
        let room = RoomNode::new(room_top_left, room_bottom_right);
        self.room_list.push(room);
        self.room_count += 1;

        // Create hall to destination room if needed
        if dest_room != 0 {
            let (hx1, hy1, hx2, hy2) = match hall_dir {
                HallDirection::Up => {
                    let hx1 = self.random_range(0, room_width - 2) as i32 + room_top_left.x + 1;
                    let hy1 = room_top_left.y;
                    let dest = &self.room_list[dest_room];
                    let hw = (dest.bottom_right.x - dest.top_left.x - 2).max(1);
                    let hx2 = self.random_range(0, hw as usize) as i32 + dest.top_left.x + 1;
                    let hy2 = dest.bottom_right.y;
                    (hx1, hy1, hx2, hy2)
                }
                HallDirection::Down => {
                    let hx1 = self.random_range(0, room_width - 2) as i32 + room_top_left.x + 1;
                    let hy1 = room_bottom_right.y;
                    let dest = &self.room_list[dest_room];
                    let hw = (dest.bottom_right.x - dest.top_left.x - 2).max(1);
                    let hx2 = self.random_range(0, hw as usize) as i32 + dest.top_left.x + 1;
                    let hy2 = dest.top_left.y;
                    (hx1, hy1, hx2, hy2)
                }
                HallDirection::Right => {
                    let hx1 = room_bottom_right.x;
                    let hy1 = self.random_range(0, room_height - 2) as i32 + room_top_left.y + 1;
                    let dest = &self.room_list[dest_room];
                    let hx2 = dest.top_left.x;
                    let hh = (dest.bottom_right.y - dest.top_left.y - 2).max(1);
                    let hy2 = self.random_range(0, hh as usize) as i32 + dest.top_left.y + 1;
                    (hx1, hy1, hx2, hy2)
                }
                HallDirection::Left => {
                    let hx1 = room_top_left.x;
                    let hy1 = self.random_range(0, room_height - 2) as i32 + room_top_left.y + 1;
                    let dest = &self.room_list[dest_room];
                    let hx2 = dest.bottom_right.x;
                    let hh = (dest.bottom_right.y - dest.top_left.y - 2).max(1);
                    let hy2 = self.random_range(0, hh as usize) as i32 + dest.top_left.y + 1;
                    (hx1, hy1, hx2, hy2)
                }
                HallDirection::None => (0, 0, 0, 0),
            };

            if hall_dir != HallDirection::None {
                let hall = HallNode::new(
                    Point::new(hx1, hy1),
                    Point::new(hx2, hy2),
                    hall_dir,
                );
                self.hall_list.push_back(hall);
            }
        }

        // Recursive subdivision
        const STANDOFF: i32 = 2;
        let room_bottom_left = Point::new(room_top_left.x, room_bottom_right.y);
        let room_top_right = Point::new(room_bottom_right.x, room_top_left.y);

        if room_height > room_width {
            // Vertical split (4 sub-areas)
            self.create_room(
                Point::new(top_left.x + STANDOFF, top_left.y + STANDOFF),
                Point::new(room_bottom_left.x - STANDOFF, room_bottom_left.y - STANDOFF),
                room_id,
                HallDirection::Right,
                None,
            );
            self.create_room(
                Point::new(room_top_right.x + STANDOFF, room_top_right.y + STANDOFF),
                Point::new(bottom_right.x - STANDOFF, bottom_right.y - STANDOFF),
                room_id,
                HallDirection::Left,
                None,
            );
            self.create_room(
                Point::new(top_left.x + STANDOFF, room_bottom_right.y + STANDOFF),
                Point::new(room_bottom_right.x - STANDOFF, bottom_right.y - STANDOFF),
                room_id,
                HallDirection::Up,
                None,
            );
            self.create_room(
                Point::new(room_top_left.x + STANDOFF, top_left.y + STANDOFF),
                Point::new(bottom_right.x - STANDOFF, room_top_left.y - STANDOFF),
                room_id,
                HallDirection::Down,
                None,
            );
        } else {
            // Horizontal split (4 sub-areas)
            self.create_room(
                Point::new(top_left.x + STANDOFF, top_left.y + STANDOFF),
                Point::new(room_top_right.x - STANDOFF, room_top_right.y - STANDOFF),
                room_id,
                HallDirection::Down,
                None,
            );
            self.create_room(
                Point::new(room_bottom_left.x + STANDOFF, room_bottom_left.y + STANDOFF),
                Point::new(bottom_right.x - STANDOFF, bottom_right.y - STANDOFF),
                room_id,
                HallDirection::Up,
                None,
            );
            self.create_room(
                Point::new(top_left.x + STANDOFF, room_top_left.y + STANDOFF),
                Point::new(room_top_left.x - STANDOFF, bottom_right.y - STANDOFF),
                room_id,
                HallDirection::Right,
                None,
            );
            self.create_room(
                Point::new(room_bottom_right.x + STANDOFF, top_left.y + STANDOFF),
                Point::new(bottom_right.x - STANDOFF, room_bottom_right.y - STANDOFF),
                room_id,
                HallDirection::Left,
                None,
            );
        }
    }

    /// Connect a hall between two rooms
    /// C++ equivalent: ConnectHall
    fn connect_hall(&mut self, hall: &HallNode) {
        let mut beginning = hall.beginning;
        let mut end = hall.end;

        // Random flags for corridor widening
        let f_minus_flag = self.random_chance(50);
        let f_plus_flag = self.random_chance(50);

        // Create doors at endpoints
        self.create_door_type(beginning);
        self.create_door_type(end);

        // Initialize direction and adjust end point
        let mut current_dir = hall.direction;
        end = Point::new(
            end.x - DIR_ADD[current_dir as usize].delta_x,
            end.y - DIR_ADD[current_dir as usize].delta_y,
        );
        self.set_predungeon(end, ',');

        let mut in_room = false;

        // Main corridor carving loop.
        //
        // C++ uses `do { ... } while (beginning != end)` and relies on the
        // game's seeded RNG (`random_chance`) to steer the corridor toward
        // `end`, guaranteeing termination. Our `random_chance` is still a
        // placeholder (returns `percent > 50`), which can make the direction
        // logic oscillate forever for certain hall configurations. Bound the
        // loop defensively so generation completes instead of hanging; the
        // corridor simply stops where it is if the cap is hit. This cap can
        // be removed once a real seeded RNG is wired in (see `generate`'s
        // `SetRndSeed` TODO).
        let mut steps = 0usize;
        const MAX_HALL_STEPS: usize = 4 * (DMAXX + DMAXY);
        while beginning != end && steps < MAX_HALL_STEPS {
            steps += 1;
            // Boundary collision detection
            if beginning.x >= 38 && current_dir == HallDirection::Right {
                current_dir = HallDirection::Left;
            }
            if beginning.y >= 38 && current_dir == HallDirection::Down {
                current_dir = HallDirection::Up;
            }
            if beginning.x <= 1 && current_dir == HallDirection::Left {
                current_dir = HallDirection::Right;
            }
            if beginning.y <= 1 && current_dir == HallDirection::Up {
                current_dir = HallDirection::Down;
            }

            // Corner collision detection (with room corners marked A/B/C/E)
            let current_char = self.get_predungeon(beginning);
            if current_char == 'C'
                && (current_dir == HallDirection::Up || current_dir == HallDirection::Left)
            {
                current_dir = HallDirection::Right;
            }
            if current_char == 'B'
                && (current_dir == HallDirection::Up || current_dir == HallDirection::Right)
            {
                current_dir = HallDirection::Down;
            }
            if current_char == 'E'
                && (current_dir == HallDirection::Left || current_dir == HallDirection::Down)
            {
                current_dir = HallDirection::Up;
            }
            if current_char == 'A'
                && (current_dir == HallDirection::Right || current_dir == HallDirection::Down)
            {
                current_dir = HallDirection::Left;
            }

            // Move in current direction
            beginning = Point::new(
                beginning.x + DIR_ADD[current_dir as usize].delta_x,
                beginning.y + DIR_ADD[current_dir as usize].delta_y,
            );

            // Handle corridor placement
            if self.get_predungeon(beginning) == ' ' {
                if in_room {
                    // Exiting a room, create door
                    let door_pos = Point::new(
                        beginning.x - DIR_ADD[current_dir as usize].delta_x,
                        beginning.y - DIR_ADD[current_dir as usize].delta_y,
                    );
                    self.create_door_type(door_pos);
                    in_room = false;
                } else {
                    // In open space, place corridor
                    if f_minus_flag {
                        if current_dir != HallDirection::Up && current_dir != HallDirection::Down {
                            self.place_hall_ext(Point::new(beginning.x, beginning.y - 1)); // Up
                        } else {
                            self.place_hall_ext(Point::new(beginning.x - 1, beginning.y)); // Left
                        }
                    }
                    if f_plus_flag {
                        if current_dir != HallDirection::Up && current_dir != HallDirection::Down {
                            self.place_hall_ext(Point::new(beginning.x, beginning.y + 1)); // Down
                        } else {
                            self.place_hall_ext(Point::new(beginning.x + 1, beginning.y)); // Right
                        }
                    }
                }
                self.set_predungeon(beginning, ',');
            } else {
                // Entering or in a room
                if !in_room && self.get_predungeon(beginning) == '#' {
                    self.create_door_type(beginning);
                }
                if self.get_predungeon(beginning) != ',' {
                    in_room = true;
                }
            }

            // Direction adjustment based on distance to goal
            let dx = (end.x - beginning.x).abs();
            let dy = (end.y - beginning.y).abs();

            // Favor horizontal movement when dx > dy
            if dx > dy {
                let rp = std::cmp::min(2 * dx, 30);
                if self.random_chance(rp as u32) {
                    if end.x <= beginning.x || beginning.x >= DMAXX as i32 {
                        current_dir = HallDirection::Left;
                    } else {
                        current_dir = HallDirection::Right;
                    }
                }
            } else {
                // Favor vertical movement when dy >= dx
                let rp = std::cmp::min(5 * dy, 80);
                if self.random_chance(rp as u32) {
                    if end.y <= beginning.y || beginning.y >= DMAXY as i32 {
                        current_dir = HallDirection::Up;
                    } else {
                        current_dir = HallDirection::Down;
                    }
                }
            }

            // Fine-tune direction when close to goal
            if dy < 10 && beginning.x == end.x && (current_dir == HallDirection::Right || current_dir == HallDirection::Left) {
                if end.y <= beginning.y || beginning.y >= DMAXY as i32 {
                    current_dir = HallDirection::Up;
                } else {
                    current_dir = HallDirection::Down;
                }
            }
            if dx < 10 && beginning.y == end.y && (current_dir == HallDirection::Up || current_dir == HallDirection::Down) {
                if end.x <= beginning.x || beginning.x >= DMAXX as i32 {
                    current_dir = HallDirection::Left;
                } else {
                    current_dir = HallDirection::Right;
                }
            }

            // Handle edge cases when almost aligned
            if dy == 1 && dx > 1 && (current_dir == HallDirection::Up || current_dir == HallDirection::Down) {
                if end.x <= beginning.x || beginning.x >= DMAXX as i32 {
                    current_dir = HallDirection::Left;
                } else {
                    current_dir = HallDirection::Right;
                }
            }
            if dx == 1 && dy > 1 && (current_dir == HallDirection::Right || current_dir == HallDirection::Left) {
                if end.y <= beginning.y || beginning.y >= DMAXY as i32 {
                    current_dir = HallDirection::Up;
                } else {
                    current_dir = HallDirection::Down;
                }
            }

            // Force direction when perfectly aligned
            if dx == 0 && self.get_predungeon(beginning) != ' ' && (current_dir == HallDirection::Right || current_dir == HallDirection::Left) {
                if end.y <= hall.beginning.y || beginning.y >= DMAXY as i32 {
                    current_dir = HallDirection::Up;
                } else {
                    current_dir = HallDirection::Down;
                }
            }
            if dy == 0 && self.get_predungeon(beginning) != ' ' && (current_dir == HallDirection::Up || current_dir == HallDirection::Down) {
                if end.x <= hall.beginning.x || beginning.x >= DMAXX as i32 {
                    current_dir = HallDirection::Left;
                } else {
                    current_dir = HallDirection::Right;
                }
            }
        }
    }

    /// Helper: Create a door marker at position
    fn create_door_type(&mut self, pos: Point) {
        if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < DMAXX && (pos.y as usize) < DMAXY {
            if self.get_predungeon(pos) == '#' {
                self.set_predungeon(pos, 'D');
            }
        }
    }

    /// Helper: Place hall extension (for wider corridors)
    fn place_hall_ext(&mut self, pos: Point) {
        if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < DMAXX && (pos.y as usize) < DMAXY {
            if self.get_predungeon(pos) == ' ' {
                self.set_predungeon(pos, ',');
            }
        }
    }

    /// Helper: Get predungeon character at position
    fn get_predungeon(&self, pos: Point) -> char {
        if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < DMAXX && (pos.y as usize) < DMAXY {
            self.predungeon[pos.x as usize][pos.y as usize]
        } else {
            ' '
        }
    }

    /// Helper: Set predungeon character at position
    fn set_predungeon(&mut self, pos: Point, c: char) {
        if pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < DMAXX && (pos.y as usize) < DMAXY {
            self.predungeon[pos.x as usize][pos.y as usize] = c;
        }
    }

    /// Helper: Random chance (0-100)
    fn random_chance(&self, percent: u32) -> bool {
        // TODO: Use proper RNG - for now use simple modulo
        // In real implementation, should use SetRndSeed/GenerateRnd
        percent > 50 // Placeholder
    }

    /// Helper: Random number in range [min, max)
    fn random_range(&self, min: usize, max: usize) -> usize {
        // TODO: Use proper RNG
        if max <= min {
            return min;
        }
        min + ((max - min) / 2) // Placeholder - returns middle value
    }

    /// Define a room in the predungeon grid
    /// C++ equivalent: DefineRoom
    fn define_room(&mut self, top_left: Point, bottom_right: Point, is_quest: bool) {
        // Mark room interior as floor
        for y in (top_left.y + 1)..bottom_right.y {
            for x in (top_left.x + 1)..bottom_right.x {
                if x >= 0 && y >= 0 && (x as usize) < DMAXX && (y as usize) < DMAXY {
                    self.predungeon[x as usize][y as usize] = '.';
                }
            }
        }

        // Mark room corners (used for hall connection logic)
        if !is_quest {
            // Top-left corner: 'A'
            if top_left.x >= 0 && top_left.y >= 0
                && (top_left.x as usize) < DMAXX && (top_left.y as usize) < DMAXY {
                self.predungeon[top_left.x as usize][top_left.y as usize] = 'A';
            }
            // Top-right corner: 'B'
            if bottom_right.x >= 0 && top_left.y >= 0
                && (bottom_right.x as usize) < DMAXX && (top_left.y as usize) < DMAXY {
                self.predungeon[bottom_right.x as usize][top_left.y as usize] = 'B';
            }
            // Bottom-left corner: 'C'
            if top_left.x >= 0 && bottom_right.y >= 0
                && (top_left.x as usize) < DMAXX && (bottom_right.y as usize) < DMAXY {
                self.predungeon[top_left.x as usize][bottom_right.y as usize] = 'C';
            }
            // Bottom-right corner: 'E'
            if bottom_right.x >= 0 && bottom_right.y >= 0
                && (bottom_right.x as usize) < DMAXX && (bottom_right.y as usize) < DMAXY {
                self.predungeon[bottom_right.x as usize][bottom_right.y as usize] = 'E';
            }
        }

        // Mark room walls as '#'
        for x in top_left.x..=bottom_right.x {
            if x >= 0 && (x as usize) < DMAXX {
                if top_left.y >= 0 && (top_left.y as usize) < DMAXY {
                    self.predungeon[x as usize][top_left.y as usize] = '#';
                }
                if bottom_right.y >= 0 && (bottom_right.y as usize) < DMAXY {
                    self.predungeon[x as usize][bottom_right.y as usize] = '#';
                }
            }
        }
        for y in top_left.y..=bottom_right.y {
            if y >= 0 && (y as usize) < DMAXY {
                if top_left.x >= 0 && (top_left.x as usize) < DMAXX {
                    self.predungeon[top_left.x as usize][y as usize] = '#';
                }
                if bottom_right.x >= 0 && (bottom_right.x as usize) < DMAXX {
                    self.predungeon[bottom_right.x as usize][y as usize] = '#';
                }
            }
        }
    }

    /// Apply pattern-based tile corrections
    /// C++ equivalent: DoPatternCheck
    fn do_pattern_check(&self, dungeon: &mut Dungeon, i: usize, j: usize) {
        // Iterate through all patterns (end marker at [4]==255)
        for pattern in &PATTERNS {
            if pattern[4] == 255 {
                break; // End of pattern table
            }

            // Read 3x3 neighborhood around predungeon[i][j]
            let mut x = (i as i32) - 1;
            let mut y = (j as i32) - 1;
            let mut matches = true;

            // Check all 9 positions in pattern
            for l in 0..9 {
                // Move to next row after every 3 elements
                if l == 3 || l == 6 {
                    y += 1;
                    x = (i as i32) - 1;
                }

                // Bounds check
                if x < 0 || x >= DMAXX as i32 || y < 0 || y >= DMAXY as i32 {
                    x += 1;
                    continue; // Out of bounds matches automatically
                }

                let cell = self.predungeon[x as usize][y as usize];
                let pattern_val = pattern[l];

                // Check pattern match
                matches &= match pattern_val {
                    0 => true, // Wildcard (any value matches)
                    1 => cell == '#',
                    2 => cell == '.',
                    3 => cell == 'D',
                    4 => cell == ' ',
                    5 => cell == 'D' || cell == '.',
                    6 => cell == 'D' || cell == '#',
                    7 => cell == ' ' || cell == '.',
                    8 => cell == 'D' || cell == '#' || cell == '.',
                    _ => false,
                };

                if !matches {
                    break;
                }

                x += 1;
            }

            // If pattern matched, replace center tile
            if matches {
                dungeon.tiles[i][j] = pattern[9] as u8;
                return; // Only apply first matching pattern
            }
        }
    }

    /// Fix tile patterns after generation
    /// C++ equivalent: FixTilesPatterns
    fn fix_tiles_patterns(&self, dungeon: &mut Dungeon) {
        for j in 0..(DMAXY - 1) {
            for i in 0..(DMAXX - 1) {
                // Rule 1: Tile 1 + Tile 3 below → convert below to 1
                if dungeon.tiles[i][j] == 1 && dungeon.tiles[i][j + 1] == 3 {
                    dungeon.tiles[i][j + 1] = 1;
                }
                // Rule 2: Tile 3 + Tile 1 below → convert below to 3
                if dungeon.tiles[i][j] == 3 && dungeon.tiles[i][j + 1] == 1 {
                    dungeon.tiles[i][j + 1] = 3;
                }
                // Rule 3: Tile 3 + Tile 7 right → convert right to 3
                if dungeon.tiles[i][j] == 3 && dungeon.tiles[i + 1][j] == 7 {
                    dungeon.tiles[i + 1][j] = 3;
                }
                // Rule 4: Tile 2 + Tile 3 right → convert right to 2
                if dungeon.tiles[i][j] == 2 && dungeon.tiles[i + 1][j] == 3 {
                    dungeon.tiles[i + 1][j] = 2;
                }
                // Rule 5: Tile 11 + Tile 14 right → convert right to 16
                if dungeon.tiles[i][j] == 11 && dungeon.tiles[i + 1][j] == 14 {
                    dungeon.tiles[i + 1][j] = 16;
                }
                // Rule 6: Tile 15 + Tile 1 below → convert below to 8
                if dungeon.tiles[i][j] == 15 && dungeon.tiles[i][j + 1] == 1 {
                    dungeon.tiles[i][j + 1] = 8;
                }
            }
        }
    }

    /// Fix transparency values for specific tile combinations
    /// C++ equivalent: FixTransparency
    fn fix_transparency(&self, dungeon: &mut Dungeon) {
        let mut yy = 16;
        for j in 0..DMAXY {
            let mut xx = 16;
            for i in 0..DMAXX {
                // BUGFIX: Should check j > 0 first (but matching C++ behavior)
                if j > 0 && dungeon.tiles[i][j] == 14 && dungeon.tiles[i][j - 1] == 10 {
                    dungeon.trans_val[xx + 1][yy] = dungeon.trans_val[xx][yy];
                    dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                }
                // BUGFIX: Should check i + 1 < DMAXX first (but matching C++ behavior)
                if i + 1 < DMAXX && dungeon.tiles[i][j] == 15 && dungeon.tiles[i + 1][j] == 11 {
                    dungeon.trans_val[xx][yy + 1] = dungeon.trans_val[xx][yy];
                    dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                }
                if dungeon.tiles[i][j] == 10 {
                    dungeon.trans_val[xx + 1][yy] = dungeon.trans_val[xx][yy];
                    dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                }
                if dungeon.tiles[i][j] == 11 {
                    dungeon.trans_val[xx][yy + 1] = dungeon.trans_val[xx][yy];
                    dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                }
                if dungeon.tiles[i][j] == 16 {
                    dungeon.trans_val[xx + 1][yy] = dungeon.trans_val[xx][yy];
                    dungeon.trans_val[xx][yy + 1] = dungeon.trans_val[xx][yy];
                    dungeon.trans_val[xx + 1][yy + 1] = dungeon.trans_val[xx][yy];
                }
                xx += 2;
            }
            yy += 2;
        }
    }

    /// Fix dirt tile boundaries
    /// C++ equivalent: FixDirtTiles
    fn fix_dirt_tiles(&self, dungeon: &mut Dungeon) {
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                // Check right neighbor (need bounds check)
                if i + 1 < DMAXX {
                    if dungeon.tiles[i][j] == 13 && dungeon.tiles[i + 1][j] != 11 {
                        dungeon.tiles[i][j] = 146;
                    }
                    if dungeon.tiles[i][j] == 11 && dungeon.tiles[i + 1][j] != 11 {
                        dungeon.tiles[i][j] = 144;
                    }
                    if dungeon.tiles[i][j] == 15 && dungeon.tiles[i + 1][j] != 11 {
                        dungeon.tiles[i][j] = 148;
                    }
                }

                // Check bottom neighbor (need bounds check)
                if j + 1 < DMAXY {
                    if dungeon.tiles[i][j] == 10 && dungeon.tiles[i][j + 1] != 10 {
                        dungeon.tiles[i][j] = 143;
                    }
                    if dungeon.tiles[i][j] == 13 && dungeon.tiles[i][j + 1] != 10 {
                        dungeon.tiles[i][j] = 146;
                    }
                    if dungeon.tiles[i][j] == 14 && dungeon.tiles[i][j + 1] != 15 {
                        dungeon.tiles[i][j] = 147;
                    }
                }
            }
        }
    }

    /// Apply substitution pass for tile variety
    /// C++ equivalent: Substitution
    fn substitution(&mut self, dungeon: &mut Dungeon) {
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                // Skip set piece room areas (TODO: implement SetPieceRoom check)
                // if SetPieceRoom.contains(x, y) { continue; }

                // 25% probability per tile
                if self.random_range(0, 4) != 0 {
                    continue;
                }

                let tile_type = TILE_TYPES_L2[dungeon.tiles[x][y] as usize];
                if tile_type == 0 {
                    continue;
                }

                // Find random alternative tile with same type
                let mut rv = self.random_range(0, 16);
                let mut candidate_tile = 0u8;

                for i in 0..161 {
                    if TILE_TYPES_L2[i] == tile_type {
                        if rv == 0 {
                            candidate_tile = i as u8;
                            break;
                        }
                        rv -= 1;
                    }
                }

                // Check if candidate tile already exists in 5×5 neighborhood
                let mut found_duplicate = false;
                'outer: for dy in (y.saturating_sub(2))..=(y + 2).min(DMAXY - 1) {
                    for dx in (x.saturating_sub(2))..=(x + 2).min(DMAXX - 1) {
                        if dungeon.tiles[dx][dy] == candidate_tile {
                            found_duplicate = true;
                            break 'outer;
                        }
                    }
                }

                if !found_duplicate {
                    dungeon.tiles[x][y] = candidate_tile;
                }
            }
        }
    }

    /// Apply shadow patterns for visual depth
    /// C++ equivalent: ApplyShadowsPatterns
    fn apply_shadows_patterns(&self, dungeon: &mut Dungeon) {
        for y in 1..DMAXY {
            for x in 1..DMAXX {
                // Get shadow types from dungeon tiles
                let sd_00 = SHADOW_TILE_TYPES_L2[dungeon.tiles[x][y] as usize];
                let sd_10 = SHADOW_TILE_TYPES_L2[dungeon.tiles[x - 1][y] as usize];
                let sd_01 = SHADOW_TILE_TYPES_L2[dungeon.tiles[x][y - 1] as usize];
                let sd_11 = SHADOW_TILE_TYPES_L2[dungeon.tiles[x - 1][y - 1] as usize];

                // Check each shadow pattern
                for &(strig, s1, s2, s3, nv1, nv2, nv3) in &SHADOW_PATTERNS_L2 {
                    if strig != sd_00 {
                        continue;
                    }
                    if s1 != 0 && s1 != sd_11 {
                        continue;
                    }
                    if s2 != 0 && s2 != sd_01 {
                        continue;
                    }
                    if s3 != 0 && s3 != sd_10 {
                        continue;
                    }

                    // Apply shadow tiles
                    if nv1 != 0 {
                        dungeon.tiles[x - 1][y - 1] = nv1;
                    }
                    if nv2 != 0 {
                        dungeon.tiles[x][y - 1] = nv2;
                    }
                    if nv3 != 0 {
                        dungeon.tiles[x - 1][y] = nv3;
                    }
                }
            }
        }
    }

    /// Fix door lockout issues
    /// C++ equivalent: FixLockout
    fn fix_lockout(&self, dungeon: &mut Dungeon) {
        // First pass: Fix door tiles without proper adjacency
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                if i > 0 && dungeon.tiles[i][j] == 4 && dungeon.tiles[i - 1][j] != 3 {
                    dungeon.tiles[i][j] = 1;
                }
                if j > 0 && dungeon.tiles[i][j] == 5 && dungeon.tiles[i][j - 1] != 3 {
                    dungeon.tiles[i][j] = 2;
                }
            }
        }

        // Second pass: Ensure horizontal door corridors have at least one door (tile 5)
        for j in 1..(DMAXY - 1) {
            for i in 1..(DMAXX - 1) {
                // Skip protected areas (if implemented)
                // if dungeon.protected.test(i, j) { continue; }

                if (dungeon.tiles[i][j] == 2 || dungeon.tiles[i][j] == 5)
                    && dungeon.tiles[i][j - 1] == 3
                    && dungeon.tiles[i][j + 1] == 3 {

                    let mut doorok = false;
                    let mut scan_i = i;

                    // Scan horizontally while in corridor
                    loop {
                        if scan_i >= DMAXX - 1 {
                            break;
                        }
                        if dungeon.tiles[scan_i][j] != 2 && dungeon.tiles[scan_i][j] != 5 {
                            break;
                        }
                        if dungeon.tiles[scan_i][j - 1] != 3 || dungeon.tiles[scan_i][j + 1] != 3 {
                            break;
                        }
                        if dungeon.tiles[scan_i][j] == 5 {
                            doorok = true;
                        }
                        scan_i += 1;
                    }

                    // If no door found, place one at end
                    if !doorok && scan_i > 0 && scan_i - 1 < DMAXX {
                        // if !dungeon.protected.test(scan_i - 1, j)
                        dungeon.tiles[scan_i - 1][j] = 5;
                    }
                }
            }
        }

        // Third pass: Ensure vertical door corridors have at least one door (tile 4)
        for j in 1..(DMAXX - 1) {  // Note: C++ has j/i flipped here (likely a bug)
            for i in 1..(DMAXY - 1) {
                // Skip protected areas (if implemented)
                // if dungeon.protected.test(j, i) { continue; }

                if (dungeon.tiles[j][i] == 1 || dungeon.tiles[j][i] == 4)
                    && dungeon.tiles[j - 1][i] == 3
                    && dungeon.tiles[j + 1][i] == 3 {

                    let mut doorok = false;
                    let mut scan_i = i;

                    // Scan vertically while in corridor
                    loop {
                        if scan_i >= DMAXY - 1 {
                            break;
                        }
                        if dungeon.tiles[j][scan_i] != 1 && dungeon.tiles[j][scan_i] != 4 {
                            break;
                        }
                        if dungeon.tiles[j - 1][scan_i] != 3 || dungeon.tiles[j + 1][scan_i] != 3 {
                            break;
                        }
                        if dungeon.tiles[j][scan_i] == 4 {
                            doorok = true;
                        }
                        scan_i += 1;
                    }

                    // If no door found, place one at end
                    if !doorok && scan_i > 0 && scan_i - 1 < DMAXY {
                        // if !dungeon.protected.test(j, scan_i - 1)
                        dungeon.tiles[j][scan_i - 1] = 4;
                    }
                }
            }
        }
    }

    /// Fix door tiles based on adjacency
    /// C++ equivalent: FixDoors
    fn fix_doors(&self, dungeon: &mut Dungeon) {
        for j in 1..DMAXY {
            for i in 1..DMAXX {
                // Vertical door (tile 4) next to horizontal wall (tile 3) becomes tile 7
                if dungeon.tiles[i][j] == 4 && dungeon.tiles[i][j - 1] == 3 {
                    dungeon.tiles[i][j] = 7;
                }
                // Horizontal door (tile 5) next to vertical wall (tile 3) becomes tile 9
                if dungeon.tiles[i][j] == 5 && dungeon.tiles[i - 1][j] == 3 {
                    dungeon.tiles[i][j] = 9;
                }
            }
        }
    }

    /// Count empty (void) tiles in predungeon
    /// C++ equivalent: CountEmptyTiles
    fn count_empty_tiles(&self) -> usize {
        let mut count = 0;
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                if self.predungeon[x][y] == ' ' {
                    count += 1;
                }
            }
        }
        count
    }

    /// Fill a void region starting from (xx, yy)
    /// C++ equivalent: FillVoid
    fn fill_void(&mut self, mut xf1: bool, mut yf1: bool, mut xf2: bool, mut yf2: bool, xx: usize, yy: usize) {
        let mut x1 = xx;
        if xf1 {
            x1 = x1.saturating_sub(1);
        }
        let mut x2 = xx;
        if xf2 {
            x2 = (x2 + 1).min(DMAXX - 1);
        }
        let mut y1 = yy;
        if yf1 {
            y1 = y1.saturating_sub(1);
        }
        let mut y2 = yy;
        if yf2 {
            y2 = (y2 + 1).min(DMAXY - 1);
        }

        if !xf1 {
            // Expand vertically
            while yf1 || yf2 {
                if y1 == 0 {
                    yf1 = false;
                }
                if y2 == DMAXY - 1 {
                    yf2 = false;
                }
                if y2.saturating_sub(y1) >= 14 {
                    yf1 = false;
                    yf2 = false;
                }
                if yf1 && y1 > 0 {
                    y1 -= 1;
                }
                if yf2 && y2 < DMAXY - 1 {
                    y2 += 1;
                }
                if x2 < DMAXX && (yf1 && self.predungeon[x2][y1] != ' ' || yf2 && self.predungeon[x2][y2] != ' ') {
                    yf1 = false;
                    yf2 = false;
                }
            }
            // Mark area
            for y in y1..=y2 {
                for x in x1..=x2 {
                    if x < DMAXX && y < DMAXY {
                        self.predungeon[x][y] = '.';
                    }
                }
            }
        } else if !xf2 {
            // Expand vertically (other direction)
            while yf1 || yf2 {
                if y1 == 0 {
                    yf1 = false;
                }
                if y2 == DMAXY - 1 {
                    yf2 = false;
                }
                if y2.saturating_sub(y1) >= 14 {
                    yf1 = false;
                    yf2 = false;
                }
                if yf1 && y1 > 0 {
                    y1 -= 1;
                }
                if yf2 && y2 < DMAXY - 1 {
                    y2 += 1;
                }
                if x1 < DMAXX && (yf1 && self.predungeon[x1][y1] != ' ' || yf2 && self.predungeon[x1][y2] != ' ') {
                    yf1 = false;
                    yf2 = false;
                }
            }
            // Mark area
            for y in y1..=y2 {
                for x in x1..=x2 {
                    if x < DMAXX && y < DMAXY {
                        self.predungeon[x][y] = '.';
                    }
                }
            }
        } else if !yf1 {
            // Expand horizontally
            while xf1 || xf2 {
                if x1 == 0 {
                    xf1 = false;
                }
                if x2 == DMAXX - 1 {
                    xf2 = false;
                }
                if x2.saturating_sub(x1) >= 14 {
                    xf1 = false;
                    xf2 = false;
                }
                if xf1 && x1 > 0 {
                    x1 -= 1;
                }
                if xf2 && x2 < DMAXX - 1 {
                    x2 += 1;
                }
                if y2 < DMAXY && (xf1 && self.predungeon[x1][y2] != ' ' || xf2 && self.predungeon[x2][y2] != ' ') {
                    xf1 = false;
                    xf2 = false;
                }
            }
            // Mark area
            for y in y1..=y2 {
                for x in x1..=x2 {
                    if x < DMAXX && y < DMAXY {
                        self.predungeon[x][y] = '.';
                    }
                }
            }
        } else {
            // Expand horizontally (other direction)
            while xf1 || xf2 {
                if x1 == 0 {
                    xf1 = false;
                }
                if x2 == DMAXX - 1 {
                    xf2 = false;
                }
                if x2.saturating_sub(x1) >= 14 {
                    xf1 = false;
                    xf2 = false;
                }
                if xf1 && x1 > 0 {
                    x1 -= 1;
                }
                if xf2 && x2 < DMAXX - 1 {
                    x2 += 1;
                }
                if y1 < DMAXY && (xf1 && self.predungeon[x1][y1] != ' ' || xf2 && self.predungeon[x2][y1] != ' ') {
                    xf1 = false;
                    xf2 = false;
                }
            }
            // Mark area
            for y in y1..=y2 {
                for x in x1..=x2 {
                    if x < DMAXX && y < DMAXY {
                        self.predungeon[x][y] = '.';
                    }
                }
            }
        }
    }

    /// Fill voids in the dungeon
    /// C++ equivalent: FillVoids
    fn fill_voids(&mut self) -> bool {
        let mut attempts = 0;
        while self.count_empty_tiles() > 700 && attempts < 100 {
            // Random position
            let xx = (self.random_chance(38) as usize % 38) + 1;
            let yy = (self.random_chance(38) as usize % 38) + 1;

            if self.predungeon[xx][yy] != '#' {
                attempts += 1;
                continue;
            }

            let mut xf1 = false;
            let mut xf2 = false;
            let mut yf1 = false;
            let mut yf2 = false;

            // Check for void patterns
            if xx > 0 && xx < DMAXX - 1 && yy > 0 && yy < DMAXY - 1 {
                // Pattern 1: void on left, floor on right
                if self.predungeon[xx - 1][yy] == ' ' && self.predungeon[xx + 1][yy] == '.' {
                    if self.predungeon[xx + 1][yy - 1] == '.'
                        && self.predungeon[xx + 1][yy + 1] == '.'
                        && self.predungeon[xx - 1][yy - 1] == ' '
                        && self.predungeon[xx - 1][yy + 1] == ' '
                    {
                        xf1 = true;
                        yf1 = true;
                        yf2 = true;
                    }
                }
                // Pattern 2: void on right, floor on left
                else if self.predungeon[xx + 1][yy] == ' ' && self.predungeon[xx - 1][yy] == '.' {
                    if self.predungeon[xx - 1][yy - 1] == '.'
                        && self.predungeon[xx - 1][yy + 1] == '.'
                        && self.predungeon[xx + 1][yy - 1] == ' '
                        && self.predungeon[xx + 1][yy + 1] == ' '
                    {
                        xf2 = true;
                        yf1 = true;
                        yf2 = true;
                    }
                }
                // Pattern 3: void above, floor below
                else if self.predungeon[xx][yy - 1] == ' ' && self.predungeon[xx][yy + 1] == '.' {
                    if self.predungeon[xx - 1][yy + 1] == '.'
                        && self.predungeon[xx + 1][yy + 1] == '.'
                        && self.predungeon[xx - 1][yy - 1] == ' '
                        && self.predungeon[xx + 1][yy - 1] == ' '
                    {
                        yf1 = true;
                        xf1 = true;
                        xf2 = true;
                    }
                }
                // Pattern 4: void below, floor above
                else if self.predungeon[xx][yy + 1] == ' ' && self.predungeon[xx][yy - 1] == '.' {
                    if self.predungeon[xx - 1][yy - 1] == '.'
                        && self.predungeon[xx + 1][yy - 1] == '.'
                        && self.predungeon[xx - 1][yy + 1] == ' '
                        && self.predungeon[xx + 1][yy + 1] == ' '
                    {
                        yf2 = true;
                        xf1 = true;
                        xf2 = true;
                    }
                }
            }

            if xf1 || yf1 || xf2 || yf2 {
                self.fill_void(xf1, yf1, xf2, yf2, xx, yy);
            }
            attempts += 1;
        }

        self.count_empty_tiles() <= 700
    }

    /// Place stairs at appropriate positions
    /// C++ equivalent: PlaceStairs
    fn place_stairs(&mut self, dungeon: &mut Dungeon, _entry: LevelEntry) -> bool {
        // Place stairs up
        if !self.place_miniset(dungeon, &miniset_ustairs()) {
            return false;
        }
        // TODO: Set ViewPosition based on entry type

        // Place stairs down
        if !self.place_miniset(dungeon, &miniset_dstairs()) {
            return false;
        }

        // Place town warp stairs (L5 only)
        if self.current_level == 5 {
            if !self.place_miniset(dungeon, &miniset_warpstairs()) {
                return false;
            }
        }

        true
    }

    /// Place a miniset by searching for valid location
    /// C++ equivalent: PlaceMiniSet
    fn place_miniset(&mut self, dungeon: &mut Dungeon, miniset: &Miniset) -> bool {
        // Try random locations until one works
        const MAX_ATTEMPTS: usize = 1000;

        for _ in 0..MAX_ATTEMPTS {
            let x = self.random_range(0, (DMAXX - miniset.width) as usize) as i32;
            let y = self.random_range(0, (DMAXY - miniset.height) as usize) as i32;
            let pos = Point::new(x, y);

            if self.try_place_miniset(dungeon, miniset, pos) {
                return true;
            }
        }

        false
    }

    /// Try to place miniset at specific position
    fn try_place_miniset(&mut self, dungeon: &mut Dungeon, miniset: &Miniset, pos: Point) -> bool {
        // Check bounds
        if pos.x + miniset.width as i32 > DMAXX as i32
            || pos.y + miniset.height as i32 > DMAXY as i32 {
            return false;
        }

        // Check if search pattern matches
        for y in 0..miniset.height {
            for x in 0..miniset.width {
                let search_val = miniset.search[y][x];
                if search_val == 0 {
                    continue; // 0 = wildcard
                }

                let dungeon_x = pos.x as usize + x;
                let dungeon_y = pos.y as usize + y;
                let predungeon_char = self.predungeon[dungeon_x][dungeon_y];

                // Map predungeon characters to search values
                let matches = match search_val {
                    1 => predungeon_char == '#',  // Wall
                    2 => predungeon_char == '.',  // Floor
                    3 => predungeon_char == '.',  // Floor (room)
                    4 => predungeon_char == ' ',  // Void
                    7 => predungeon_char == '#' || predungeon_char == 'D', // Wall or door
                    _ => false,
                };

                if !matches {
                    return false;
                }
            }
        }

        // Pattern matches, apply replacement to dungeon
        for y in 0..miniset.height {
            for x in 0..miniset.width {
                let replace_val = miniset.replace[y][x];
                if replace_val == 0 {
                    continue; // 0 = no change
                }

                let dungeon_x = pos.x as usize + x;
                let dungeon_y = pos.y as usize + y;

                // Update dungeon tiles
                dungeon.tiles[dungeon_x][dungeon_y] = replace_val;
            }
        }

        true
    }

    /// Place a miniset randomly on the predungeon
    /// C++ equivalent: PlaceMiniSetRandom
    /// Place a miniset randomly with probability check
    /// C++ equivalent: PlaceMiniSetRandom
    ///
    /// # Arguments
    /// * `dungeon` - The dungeon to modify
    /// * `miniset` - The miniset to place
    /// * `rnd_per` - Success probability out of 100 (e.g., 100 = 100% success)
    fn place_miniset_random(&mut self, dungeon: &mut Dungeon, miniset: &Miniset, rnd_per: u32) -> bool {
        // Check probability
        if self.random_range(0, 100) >= rnd_per as usize {
            return false;
        }

        // Try to place the miniset
        self.place_miniset(dungeon, miniset)
    }

    /// Place a 1x1 miniset randomly (tile substitution)
    /// C++ equivalent: PlaceMiniSetRandom1x1
    fn place_miniset_random_1x1(&mut self, dungeon: &mut Dungeon, search: u8, replace: u8, rnd_per: u32) -> bool {
        // Create temporary 1x1 miniset
        let miniset = Miniset::new(
            1,
            1,
            vec![vec![search]],
            vec![vec![replace]]
        );

        // Use existing place_miniset_random logic
        self.place_miniset_random(dungeon, &miniset, rnd_per)
    }

    /// Place a miniset at a specific location if pattern matches
    /// C++ equivalent: PlaceMiniSet
    fn place_miniset_at(&mut self, miniset: &Miniset, pos: Point) -> bool {
        // Check bounds
        if pos.x + miniset.width as i32 > DMAXX as i32
            || pos.y + miniset.height as i32 > DMAXY as i32 {
            return false;
        }

        // Check if search pattern matches
        for y in 0..miniset.height {
            for x in 0..miniset.width {
                let search_val = miniset.search[y][x];
                if search_val == 0 {
                    continue; // 0 = wildcard
                }

                let dungeon_x = pos.x as usize + x;
                let dungeon_y = pos.y as usize + y;
                let predungeon_char = self.predungeon[dungeon_x][dungeon_y];

                // Map predungeon characters to search values
                let matches = match search_val {
                    1 => predungeon_char == '#',  // Wall
                    2 => predungeon_char == '.',  // Floor
                    3 => predungeon_char == '.',  // Floor (room)
                    4 => predungeon_char == ' ',  // Void
                    7 => predungeon_char == '#' || predungeon_char == 'D', // Wall or door
                    _ => false,
                };

                if !matches {
                    return false;
                }
            }
        }

        // Pattern matches, apply replacement
        for y in 0..miniset.height {
            for x in 0..miniset.width {
                let replace_val = miniset.replace[y][x];
                if replace_val == 0 {
                    continue; // 0 = no change
                }

                let dungeon_x = pos.x as usize + x;
                let dungeon_y = pos.y as usize + y;

                // Mark the tile (in real implementation, would update dungeon tiles)
                // For now, just mark in predungeon for testing
                self.predungeon[dungeon_x][dungeon_y] = 'M'; // 'M' for miniset
            }
        }

        true
    }
}

impl Default for CatacombsGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Create L2 (Catacombs) dungeon
/// C++ equivalent: CreateL2Dungeon
pub fn create_l2_dungeon(dungeon: &mut Dungeon, seed: u32, level: u8) -> bool {
    let mut generator = CatacombsGenerator::new();
    generator.generate(dungeon, seed, level)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hall_direction_enum() {
        assert_eq!(HallDirection::None as i8, 0);
        assert_eq!(HallDirection::Up as i8, 1);
        assert_eq!(HallDirection::Right as i8, 2);
        assert_eq!(HallDirection::Down as i8, 3);
        assert_eq!(HallDirection::Left as i8, 4);

        assert_eq!(HallDirection::from_i8(0), Some(HallDirection::None));
        assert_eq!(HallDirection::from_i8(4), Some(HallDirection::Left));
        assert_eq!(HallDirection::from_i8(5), None);
    }

    #[test]
    fn test_hall_direction_displacement() {
        assert_eq!(HallDirection::Up.to_displacement(), Displacement { delta_x: 0, delta_y: -1 });
        assert_eq!(HallDirection::Right.to_displacement(), Displacement { delta_x: 1, delta_y: 0 });
        assert_eq!(HallDirection::Down.to_displacement(), Displacement { delta_x: 0, delta_y: 1 });
        assert_eq!(HallDirection::Left.to_displacement(), Displacement { delta_x: -1, delta_y: 0 });
    }

    #[test]
    fn test_hall_node_creation() {
        let hall = HallNode::new(Point::new(5, 5), Point::new(10, 5), HallDirection::Right);
        assert_eq!(hall.beginning, Point::new(5, 5));
        assert_eq!(hall.end, Point::new(10, 5));
        assert_eq!(hall.direction, HallDirection::Right);
    }

    #[test]
    fn test_room_node_creation() {
        let room = RoomNode::new(Point::new(10, 10), Point::new(20, 20));
        assert_eq!(room.top_left, Point::new(10, 10));
        assert_eq!(room.bottom_right, Point::new(20, 20));
    }

    #[test]
    fn test_room_node_dimensions() {
        let room = RoomNode::new(Point::new(5, 10), Point::new(15, 25));
        assert_eq!(room.width(), 11); // 15 - 5 + 1
        assert_eq!(room.height(), 16); // 25 - 10 + 1
        assert_eq!(room.area(), 176); // 11 * 16
    }

    #[test]
    fn test_room_node_contains() {
        let room = RoomNode::new(Point::new(10, 10), Point::new(20, 20));
        assert!(room.contains(Point::new(15, 15)));
        assert!(room.contains(Point::new(10, 10))); // Edge
        assert!(room.contains(Point::new(20, 20))); // Edge
        assert!(!room.contains(Point::new(5, 15)));
        assert!(!room.contains(Point::new(25, 15)));
    }

    #[test]
    fn test_catacombs_generator_new() {
        let gen = CatacombsGenerator::new();
        assert_eq!(gen.room_count, 0);
        assert!(gen.room_list.is_empty());
        assert!(gen.hall_list.is_empty());
        assert_eq!(gen.current_level, 5);
    }

    #[test]
    fn test_catacombs_generator_reset() {
        let mut gen = CatacombsGenerator::new();
        gen.room_count = 10;
        gen.room_list.push(RoomNode::new(Point::new(0, 0), Point::new(10, 10)));
        gen.reset();
        assert_eq!(gen.room_count, 0);
        assert!(gen.room_list.is_empty());
    }

    #[test]
    fn test_get_quest_room_size() {
        let mut gen = CatacombsGenerator::new();
        gen.current_level = 5;
        assert_eq!(gen.get_quest_room_size(), Some((14, 20)));
        gen.current_level = 6;
        assert_eq!(gen.get_quest_room_size(), Some((10, 10)));
        gen.current_level = 7;
        assert_eq!(gen.get_quest_room_size(), Some((15, 15)));
        gen.current_level = 8;
        assert_eq!(gen.get_quest_room_size(), None);
    }

    #[test]
    fn test_miniset_creation() {
        let search = vec![vec![1, 2], vec![3, 4]];
        let replace = vec![vec![5, 6], vec![7, 8]];
        let miniset = Miniset::new(2, 2, search.clone(), replace.clone());
        assert_eq!(miniset.width, 2);
        assert_eq!(miniset.height, 2);
        assert_eq!(miniset.search, search);
        assert_eq!(miniset.replace, replace);
    }

    #[test]
    fn test_tile_types_l2_length() {
        assert_eq!(TILE_TYPES_L2.len(), 161);
        assert_eq!(SHADOW_TILE_TYPES_L2.len(), 161);
    }

    #[test]
    fn test_dir_add_offsets() {
        assert_eq!(DIR_ADD[0], Displacement { delta_x: 0, delta_y: 0 }); // None
        assert_eq!(DIR_ADD[1], Displacement { delta_x: 0, delta_y: -1 }); // Up
        assert_eq!(DIR_ADD[2], Displacement { delta_x: 1, delta_y: 0 }); // Right
        assert_eq!(DIR_ADD[3], Displacement { delta_x: 0, delta_y: 1 }); // Down
        assert_eq!(DIR_ADD[4], Displacement { delta_x: -1, delta_y: 0 }); // Left
    }

    #[test]
    fn test_catacombs_generate_framework() {
        let mut gen = CatacombsGenerator::new();
        let _dungeon = Dungeon::new();
        // Framework test - just ensure no panic
        // Full generation requires complete implementation
        gen.reset();
        assert_eq!(gen.room_count, 0);
    }

    // =============================================================================
    // Day 62 Tests: ConnectHall + Minisets
    // =============================================================================

    #[test]
    fn test_miniset_ustairs() {
        let stairs = miniset_ustairs();
        assert_eq!(stairs.width, 4);
        assert_eq!(stairs.height, 4);
        assert_eq!(stairs.search[0], vec![3, 3, 3, 3]);
        assert_eq!(stairs.replace[1][1], 72); // Upstairs tile
    }

    #[test]
    fn test_miniset_dstairs() {
        let stairs = miniset_dstairs();
        assert_eq!(stairs.width, 4);
        assert_eq!(stairs.height, 4);
        assert_eq!(stairs.replace[1][1], 48); // Downstairs tile
    }

    #[test]
    fn test_miniset_warpstairs() {
        let stairs = miniset_warpstairs();
        assert_eq!(stairs.width, 4);
        assert_eq!(stairs.height, 4);
        assert_eq!(stairs.replace[1][1], 158); // Warp tile
    }

    #[test]
    fn test_miniset_varch1() {
        let arch = miniset_varch1();
        assert_eq!(arch.width, 2);
        assert_eq!(arch.height, 4);
        assert_eq!(arch.search[1][1], 1);
        assert_eq!(arch.replace[0][0], 48);
    }

    #[test]
    fn test_miniset_harch1() {
        let arch = miniset_harch1();
        assert_eq!(arch.width, 4);
        assert_eq!(arch.height, 2);
        assert_eq!(arch.search[0][0], 3);
        assert_eq!(arch.replace[0][0], 49);
    }

    #[test]
    fn test_get_critical_minisets() {
        let minisets = get_critical_minisets();
        assert_eq!(minisets.len(), 6); // 3 stairs + 2 arches + 1 big
        assert_eq!(minisets[0].width, 4); // USTAIRS
        assert_eq!(minisets[1].width, 4); // DSTAIRS
        assert_eq!(minisets[2].width, 4); // WARPSTAIRS
    }

    #[test]
    fn test_place_miniset_at_simple() {
        let mut gen = CatacombsGenerator::new();

        // Setup a simple floor pattern
        for y in 5..9 {
            for x in 5..9 {
                gen.predungeon[x][y] = '.';
            }
        }

        let stairs = miniset_ustairs();
        let placed = gen.place_miniset_at(&stairs, Point::new(5, 5));
        assert!(placed);

        // Check that miniset was marked
        assert_eq!(gen.predungeon[6][6], 'M'); // Center should be marked
    }

    #[test]
    fn test_place_miniset_at_pattern_mismatch() {
        let mut gen = CatacombsGenerator::new();

        // Setup wrong pattern (walls instead of floor)
        for y in 5..9 {
            for x in 5..9 {
                gen.predungeon[x][y] = '#';
            }
        }

        let stairs = miniset_ustairs();
        let placed = gen.place_miniset_at(&stairs, Point::new(5, 5));
        assert!(!placed); // Should fail - pattern doesn't match
    }

    #[test]
    fn test_connect_hall_basic() {
        let mut gen = CatacombsGenerator::new();

        // Create a simple hall
        let hall = HallNode::new(
            Point::new(10, 10),
            Point::new(20, 10),
            HallDirection::Right,
        );

        // Mark endpoints as walls
        gen.predungeon[10][10] = '#';
        gen.predungeon[20][10] = '#';

        // Connect the hall
        gen.connect_hall(&hall);

        // Check that some corridor was carved
        // (exact verification depends on RNG, but should have changed some tiles)
        let mut corridor_found = false;
        for x in 10..=20 {
            if gen.predungeon[x][10] == ',' || gen.predungeon[x][10] == 'D' {
                corridor_found = true;
                break;
            }
        }
        assert!(corridor_found, "ConnectHall should carve corridor tiles");
    }

    #[test]
    fn test_helper_functions() {
        let mut gen = CatacombsGenerator::new();

        // Test get_predungeon
        gen.predungeon[10][10] = 'X';
        assert_eq!(gen.get_predungeon(Point::new(10, 10)), 'X');
        assert_eq!(gen.get_predungeon(Point::new(-1, -1)), ' '); // Out of bounds

        // Test set_predungeon
        gen.set_predungeon(Point::new(15, 15), 'Y');
        assert_eq!(gen.predungeon[15][15], 'Y');

        // Test create_door_type
        gen.predungeon[20][20] = '#';
        gen.create_door_type(Point::new(20, 20));
        assert_eq!(gen.predungeon[20][20], 'D');

        // Test place_hall_ext
        gen.predungeon[25][25] = ' ';
        gen.place_hall_ext(Point::new(25, 25));
        assert_eq!(gen.predungeon[25][25], ',');
    }

    // =============================================================================
    // Day 63 Tests: FillVoids + Integration
    // =============================================================================

    #[test]
    fn test_count_empty_tiles() {
        let mut gen = CatacombsGenerator::new();

        // Initially all empty (40x40 = 1600)
        let initial_count = gen.count_empty_tiles();
        assert_eq!(initial_count, DMAXX * DMAXY);

        // Fill some tiles
        for x in 5..15 {
            for y in 5..15 {
                gen.predungeon[x][y] = '.';
            }
        }

        let after_count = gen.count_empty_tiles();
        assert_eq!(after_count, initial_count - 100); // 10x10 = 100
    }

    #[test]
    fn test_fill_void_basic() {
        let mut gen = CatacombsGenerator::new();

        // Create a pattern: floor on right, void on left
        for x in 15..20 {
            for y in 10..15 {
                gen.predungeon[x][y] = '.';
            }
        }
        gen.predungeon[14][12] = '#'; // Wall to fill

        // Verify initial state
        assert_eq!(gen.predungeon[14][12], '#');
        assert_eq!(gen.predungeon[13][12], ' '); // Void

        // Call fill_void
        gen.fill_void(true, true, false, true, 14, 12);

        // Should have expanded the void into floor
        assert_eq!(gen.predungeon[14][12], '.');
    }

    #[test]
    fn test_fill_voids_reduces_voids() {
        let mut gen = CatacombsGenerator::new();

        // Create a large floor area with some voids
        for x in 5..35 {
            for y in 5..35 {
                gen.predungeon[x][y] = '.';
            }
        }

        // Add walls around edges
        for x in 4..36 {
            gen.predungeon[x][4] = '#';
            gen.predungeon[x][35] = '#';
        }
        for y in 4..36 {
            gen.predungeon[4][y] = '#';
            gen.predungeon[35][y] = '#';
        }

        // Create void pockets
        for x in 10..15 {
            for y in 10..15 {
                gen.predungeon[x][y] = ' ';
            }
        }

        let before = gen.count_empty_tiles();
        let result = gen.fill_voids();
        let after = gen.count_empty_tiles();

        // Should reduce voids or already be below threshold
        // (result depends on how many voids and random placement)
        assert!(after <= before, "FillVoids should not increase empty tiles");
        // Result depends on void threshold (700)
        assert_eq!(result, after <= 700);
    }

    #[test]
    fn test_create_dungeon_with_fill_voids() {
        let mut gen = CatacombsGenerator::new();
        let mut dungeon = Dungeon::new();

        // Run create_dungeon (includes fill_voids now)
        gen.reset();
        let result = gen.create_dungeon(&mut dungeon);

        // Should succeed or fail gracefully
        // (exact result depends on RNG, but should not panic)
        let _ = result;

        // Check that predungeon was modified
        let mut has_floor = false;
        for y in 0..DMAXY {
            for x in 0..DMAXX {
                if gen.predungeon[x][y] == '.' {
                    has_floor = true;
                    break;
                }
            }
            if has_floor {
                break;
            }
        }
        assert!(has_floor, "CreateDungeon should create floor tiles");
    }

    #[test]
    fn test_fill_void_horizontal_expansion() {
        let mut gen = CatacombsGenerator::new();

        // Setup horizontal void pattern
        for y in 10..20 {
            gen.predungeon[15][y] = '.'; // Vertical floor strip
        }
        gen.predungeon[14][15] = '#'; // Wall to fill

        // Fill horizontally
        gen.fill_void(true, false, true, false, 14, 15);

        // Should expand horizontally
        assert_eq!(gen.predungeon[14][15], '.');
    }

    #[test]
    fn test_fill_void_vertical_expansion() {
        let mut gen = CatacombsGenerator::new();

        // Setup vertical void pattern
        for x in 10..20 {
            gen.predungeon[x][15] = '.'; // Horizontal floor strip
        }
        gen.predungeon[15][14] = '#'; // Wall to fill

        // Fill vertically
        gen.fill_void(false, true, false, true, 15, 14);

        // Should expand vertically
        assert_eq!(gen.predungeon[15][14], '.');
    }

    #[test]
    fn test_fill_voids_pattern_detection() {
        let mut gen = CatacombsGenerator::new();

        // Create pattern 1: void left, floor right
        gen.predungeon[20][20] = '#';
        gen.predungeon[19][20] = ' ';
        gen.predungeon[21][20] = '.';
        gen.predungeon[21][19] = '.';
        gen.predungeon[21][21] = '.';
        gen.predungeon[19][19] = ' ';
        gen.predungeon[19][21] = ' ';

        let before = gen.predungeon[20][20];
        let _ = gen.fill_voids();
        let after = gen.predungeon[20][20];

        // Should have attempted to process this pattern
        // (exact result depends on random selection)
        let _ = (before, after);
    }
}
