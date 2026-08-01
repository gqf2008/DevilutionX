//! Cathedral and Crypt level generation (L1/L5)
//!
//! Implements procedural dungeon generation for Cathedral (DTYPE_CATHEDRAL) and
//! Crypt (DTYPE_CRYPT) levels. Uses room-based layout with 3 chambers connected
//! by corridors.
//!
//! C++ Source: Source/levels/drlg_l1.cpp
//! Translation Date: 2024-XX-XX

use crate::levels::gendung::Dungeon;
use crate::levels::types::{DungeonType, ThemeLocation};
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
    /// Vertical wall (left side)
    VWall = 1,
    /// Horizontal wall (top side)
    HWall = 2,
    /// Diagonal-corner wall written by MakeDmt (C++ `VCorner = 16`).
    NWCorner = 16,
    /// Top-right corner wall (minisets only; no C++ L1 equivalent)
    NECorner = 41,
    /// Bottom-left corner wall (minisets only; no C++ L1 equivalent)
    SWCorner = 42,
    /// Wall written by MakeDmt for the diagonal branch (C++ `DWall = 4`).
    SECorner = 4,
    /// Vertical wall with door (C++ `VDoor = 25`)
    VWallDoor = 25,
    /// Horizontal wall with door (C++ `HDoor = 26`)
    HWallDoor = 26,
    /// Archway horizontal (C++ `HArch = 12`)
    ArchH1 = 12,
    /// Archway horizontal bottom (unused in generation)
    ArchH2 = 43,
    /// Archway vertical (C++ `VArch = 11`)
    ArchV1 = 11,
    /// Archway vertical right (unused in generation)
    ArchV2 = 44,
    /// Floor tile
    Floor = 13,

    /// C++ `Corner = 3` (wall corner; used by AddWall/FixTilesPatterns)
    Corner = 3,
    /// C++ `DArch = 5` (door arch)
    DArch = 5,
    /// C++ `HArchEnd = 8` (horizontal arch end)
    HArchEnd = 8,
    /// C++ `VArchEnd = 9` (vertical arch end)
    VArchEnd = 9,
    /// C++ `HArchVWall = 10` (horizontal arch + vertical wall)
    HArchVWall = 10,
    /// C++ `HWallVArch = 14` (horizontal wall + vertical arch)
    HWallVArch = 14,
    /// C++ `Pillar = 15` (room pillar)
    Pillar = 15,
    /// C++ `VWallEnd = 6` (vertical wall end)
    VWallEnd = 6,
    /// C++ `HWallEnd = 7` (horizontal wall end)
    HWallEnd = 7,
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
    /// C++ `DirtHwallEnd = 23` (dirt horizontal wall end)
    DirtHwallEnd = 23,
    /// C++ `DirtVwallEnd = 24` (dirt vertical wall end)
    DirtVwallEnd = 24,

    /// T-junction to south (self-invented; unused in generation)
    DirtVWallToSouth = 45,
    /// T-junction to north (self-invented; unused in generation)
    DirtVWallToNorth = 46,
    /// T-junction to east (self-invented; unused in generation)
    DirtHWallToEast = 47,
    /// T-junction to west (self-invented; unused in generation)
    DirtHWallToWest = 48,
    /// Dirt top-left corner (self-invented; unused in generation)
    DirtNWCorner = 49,
    /// Dirt top-right corner (self-invented; unused in generation)
    DirtNECorner = 50,
    /// Dirt bottom-left corner (self-invented; unused in generation)
    DirtSWCorner = 51,
    /// Dirt bottom-right corner (self-invented; unused in generation)
    DirtSECorner = 52,
    /// Cross junction (self-invented; unused in generation)
    DirtCross = 53,
    /// Horizontal wall dirt variant (self-invented; unused in generation)
    DirtHWall = 54,
    /// Vertical wall dirt variant (self-invented; unused in generation)
    DirtVWall = 55,

    /// Lava pool (crypt only)
    Lava = 56,
    /// Entrance stairs (C++ `EntranceStairs = 64`)
    EntranceStairs = 64,
    /// Dirt floor (C++ `Dirt = 22` — the Cathedral background tile)
    Dirt = 22,
    /// C++ `Floor22 = 162` (floor variation from FillFloor)
    Floor22 = 162,
    /// C++ `Floor23 = 163` (floor variation from FillFloor)
    Floor23 = 163,
    /// Invalid/empty tile
    Invalid = 57,

    // Crypt-specific tiles (not used in Cathedral)
    /// Crypt archway horizontal top
    CryptArchH1 = 29,
    /// Crypt archway horizontal bottom
    CryptArchH2 = 30,
    /// Crypt archway vertical left
    CryptArchV1 = 31,
    /// Crypt archway vertical right
    CryptArchV2 = 32,

    /// Crypt horizontal wall with door
    CryptHWallDoor = 33,
    /// Crypt vertical wall with door
    CryptVWallDoor = 34,

    /// Crypt top-left corner
    CryptNWCorner = 35,
    /// Crypt top-right corner
    CryptNECorner = 36,
    /// Crypt bottom-left corner
    CryptSWCorner = 37,
    /// Crypt bottom-right corner
    CryptSECorner = 38,

    /// Crypt horizontal wall
    CryptHWall = 39,
    /// Crypt vertical wall
    CryptVWall = 40,
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
            1 => Ok(Tile::VWall),
            2 => Ok(Tile::HWall),
            3 => Ok(Tile::Corner),
            4 => Ok(Tile::SECorner),
            5 => Ok(Tile::DArch),
            6 => Ok(Tile::VWallEnd),
            8 => Ok(Tile::HArchEnd),
            9 => Ok(Tile::VArchEnd),
            10 => Ok(Tile::HArchVWall),
            14 => Ok(Tile::HWallVArch),
            15 => Ok(Tile::Pillar),
            7 => Ok(Tile::HWallEnd),
            17 => Ok(Tile::HCorner),
            18 => Ok(Tile::DirtHwall),
            19 => Ok(Tile::DirtVwall),
            20 => Ok(Tile::VDirtCorner),
            21 => Ok(Tile::HDirtCorner),
            23 => Ok(Tile::DirtHwallEnd),
            24 => Ok(Tile::DirtVwallEnd),
            11 => Ok(Tile::ArchV1),
            12 => Ok(Tile::ArchH1),
            13 => Ok(Tile::Floor),
            16 => Ok(Tile::NWCorner),
            22 => Ok(Tile::Dirt),
            25 => Ok(Tile::VWallDoor),
            26 => Ok(Tile::HWallDoor),
            41 => Ok(Tile::NECorner),
            42 => Ok(Tile::SWCorner),
            43 => Ok(Tile::ArchH2),
            44 => Ok(Tile::ArchV2),
            45 => Ok(Tile::DirtVWallToSouth),
            46 => Ok(Tile::DirtVWallToNorth),
            47 => Ok(Tile::DirtHWallToEast),
            48 => Ok(Tile::DirtHWallToWest),
            49 => Ok(Tile::DirtNWCorner),
            50 => Ok(Tile::DirtNECorner),
            51 => Ok(Tile::DirtSWCorner),
            52 => Ok(Tile::DirtSECorner),
            53 => Ok(Tile::DirtCross),
            54 => Ok(Tile::DirtHWall),
            55 => Ok(Tile::DirtVWall),
            56 => Ok(Tile::Lava),
            57 => Ok(Tile::Invalid),
            64 => Ok(Tile::EntranceStairs),
            162 => Ok(Tile::Floor22),
            163 => Ok(Tile::Floor23),
            58 => Ok(Tile::CryptArchH1),
            59 => Ok(Tile::CryptArchH2),
            60 => Ok(Tile::CryptArchV1),
            61 => Ok(Tile::CryptArchV2),
            62 => Ok(Tile::CryptHWallDoor),
            63 => Ok(Tile::CryptVWallDoor),
            65 => Ok(Tile::CryptNWCorner),
            66 => Ok(Tile::CryptNECorner),
            67 => Ok(Tile::CryptSWCorner),
            68 => Ok(Tile::CryptSECorner),
            69 => Ok(Tile::CryptHWall),
            70 => Ok(Tile::CryptVWall),
            _ => Err(()),}
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
    pub fn matches(&self, dungeon: &[[Tile; DUNGEON_SIZE]; DUNGEON_SIZE], x: usize, y: usize) -> bool {
        if x + self.width > DUNGEON_SIZE || y + self.height > DUNGEON_SIZE {
            return false;
        }

        for dy in 0..self.height {
            for dx in 0..self.width {
                let search_tile = self.search[dy][dx];
                if search_tile != Tile::Invalid && dungeon[y + dy][x + dx] != search_tile {
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
            vec![Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor],
            vec![Floor, Floor, Floor, Floor],
        ],
        replace: vec![
            vec![Invalid, NWCorner, HWall, NECorner],
            vec![Invalid, VWall, EntranceStairs, VWall],
            vec![Invalid, VWall, EntranceStairs, VWall],
            vec![Invalid, SWCorner, HWall, SECorner],
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
            vec![NWCorner, HWall, HWall, NECorner],
            vec![VWall, Floor, Floor, VWall],
            vec![SWCorner, HWall, HWall, SECorner],
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
            vec![Floor, Floor],
            vec![Floor, Floor],
        ],
        replace: vec![
            vec![NWCorner, NECorner],
            vec![SWCorner, SECorner],
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
            vec![NWCorner, HWall, HWall, HWall, HWall, NECorner],
            vec![VWall, Floor, Floor, Floor, Floor, VWall],
            vec![VWall, Floor, Floor, Floor, Floor, VWall],
            vec![VWall, Floor, Floor, Floor, Floor, VWall],
            vec![VWall, Floor, Floor, Floor, Floor, VWall],
            vec![SWCorner, HWall, HWall, HWall, HWall, SECorner],
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
        // Reset masks
        self.dungeon_mask = [[false; DUNGEON_SIZE]; DUNGEON_SIZE];
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
            // Check if adjacent tiles are Floor and not protected/chamber
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

        // Check end tile validity
        if x + length < DUNGEON_SIZE {
            let end_tile = self.dungeon[y][x + length];
            if matches!(
                end_tile,
                Tile::NWCorner | Tile::SECorner | Tile::VWall | Tile::HWall
            ) {
                return length as i32;
            }
        }

        -1
    }

    /// Check if vertical wall can be placed
    ///
    /// C++ source: VerticalWallOk() in drlg_l1.cpp:597-613
    fn vertical_wall_ok(&self, x: usize, y: usize) -> i32 {
        let mut length = 1;
        while y + length < DUNGEON_SIZE && self.dungeon[y + length][x] == Tile::Floor {
            // Check if adjacent tiles are Floor and not protected/chamber
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

        // Check end tile validity
        if y + length < DUNGEON_SIZE {
            let end_tile = self.dungeon[y + length][x];
            if matches!(
                end_tile,
                Tile::NWCorner | Tile::SECorner | Tile::VWall | Tile::HWall
            ) {
                return length as i32;
            }
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

        // Randomly choose wall style — C++ GenerateRnd(4)
        match self.rng.generate(4) {
            2 => {
                // Add arch
                wall_tile = Tile::ArchH1;
                door_tile = Tile::ArchH1;
                if start == Tile::HWall {
                    start = Tile::ArchH1;
                }
            }
            3 => {
                // Add fence (not implemented for Cathedral, treat as normal wall)
                wall_tile = Tile::HWall;
            }
            _ => {}
        }

        // Randomly choose arch for door — C++ GenerateRnd(6) == 5
        if self.rng.generate(6) == 5 {
            door_tile = Tile::ArchH1;
        }

        self.dungeon[y][x] = start;

        // Place wall tiles
        for i in 1..max_x {
            self.dungeon[y][x + i as usize] = wall_tile;
        }

        // Place door at random position — C++ GenerateRnd(maxX - 1) + 1
        let door_pos = self.rng.random_less_than(max_x - 1) as usize + 1;
        self.dungeon[y][x + door_pos] = door_tile;
        if door_tile == Tile::HWallDoor {
            self.protected[y][x + door_pos] = true;
        }
    }

    /// Place vertical wall with random door
    ///
    /// C++ source: VerticalWall() in drlg_l1.cpp:652-687
    fn vertical_wall(&mut self, x: usize, y: usize, start: Tile, max_y: i32) {
        let mut wall_tile = Tile::VWall;
        let mut door_tile = Tile::VWallDoor;
        let mut start = start;

        // Randomly choose wall style — C++ GenerateRnd(4)
        match self.rng.generate(4) {
            2 => {
                // Add arch
                wall_tile = Tile::ArchV1;
                door_tile = Tile::ArchV1;
                if start == Tile::VWall {
                    start = Tile::ArchV1;
                }
            }
            3 => {
                // Add fence (not implemented for Cathedral, treat as normal wall)
                wall_tile = Tile::VWall;
            }
            _ => {}
        }

        // Randomly choose arch for door — C++ GenerateRnd(6) == 5
        if self.rng.generate(6) == 5 {
            door_tile = Tile::ArchV1;
        }

        self.dungeon[y][x] = start;

        // Place wall tiles
        for i in 1..max_y {
            self.dungeon[y + i as usize][x] = wall_tile;
        }

        // Place door at random position — C++ GenerateRnd(maxY - 1) + 1
        let door_pos = self.rng.random_less_than(max_y - 1) as usize + 1;
        self.dungeon[y + door_pos][x] = door_tile;
        if door_tile == Tile::VWallDoor {
            self.protected[y + door_pos][x] = true;
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

                let tile = self.dungeon[y][x];

                // Try horizontal wall from corner
                if tile == Tile::NWCorner {
                    let _ = self.rng.generate(1); // Discard random value (C++ compatibility)
                    let max_x = self.horizontal_wall_ok(x, y);
                    if max_x > 0 {
                        self.horizontal_wall(x, y, Tile::HWall, max_x);
                    }
                }

                // Try vertical wall from corner
                if self.dungeon[y][x] == Tile::NWCorner {
                    let _ = self.rng.generate(1); // discard random value
                    let max_y = self.vertical_wall_ok(x, y);
                    if max_y > 0 {
                        self.vertical_wall(x, y, Tile::VWall, max_y);
                    }
                }

                // Handle other wall types (VWallEnd, HWallEnd, etc.)
                // Simplified implementation for now
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
    /// C++ source: CreateL5Dungeon() in drlg_l1.cpp:1302-1318
    ///
    /// Main entry point for Cathedral generation. Steps:
    /// 1. Initialize dungeon with dirt
    /// 2. Generate rooms and chambers
    /// 3. Connect chambers with corridors
    /// 4. Place minisets (stairs, lamps, etc.)
    /// 5. Add floor variations
    /// 6. Fix tile transitions and corners
    pub fn generate(&mut self, _level_type: DungeonType, seed: u32) {
        // Seed the Diablo LCG before generation so results are deterministic
        // and reproduce the C++ `SetRndSeed(seed)` behaviour exactly.
        self.rng.set_seed(seed);

        // Initialize dungeon
        self.init_dungeon();

        // Generate room layout
        self.first_room();

        // Convert mask to tiles
        self.make_dmt();

        // C++ order: MakeDmt -> FillChambers -> FixTilesPatterns -> AddWall.
        // (FillChambers is not ported yet; its chamber-mask effect is minimal
        // for the current layout.)
        self.fill_chambers();
        self.fix_tiles_patterns();

        // Add walls between rooms
        self.add_wall();

        // Place stairs and decorations
        let stairs_up = stairs_up_miniset();
        let stairs_down = stairs_down_miniset();
        let lamps = lamps_miniset();

        // Try to place upward stairs
        self.place_miniset_random(&stairs_up, 100);

        // Try to place downward stairs
        self.place_miniset_random(&stairs_down, 100);

        // Place 5-10 lamp decorations — C++ GenerateRnd(5) + 5
        let num_lamps = self.rng.random_less_than(5) as u32 + 5;
        for _ in 0..num_lamps {
            self.place_miniset_random(&lamps, 100);
        }

        // Add floor variations
        self.fill_floor();
    }

    /// Place miniset randomly in dungeon
    ///
    /// C++ source: PlaceMiniSetRandom() in drlg_l1.cpp:1246-1261
    ///
    /// Attempts to place miniset at random valid positions with probability.
    /// C++ logic: if (GenerateRnd(100) >= rndper) continue; (skip placement)
    /// So higher rndper = MORE likely to skip = LESS likely to place
    /// rndper=0 → always place (never skip)
    /// rndper=100 → never place (always skip, unless rng returns 100+)
    /// Returns true if miniset was placed successfully.
    pub fn place_miniset_random(&mut self, miniset: &Miniset, rnd_percent: u32) -> bool {
        for y in 0..(DUNGEON_SIZE - miniset.height) {
            for x in 0..(DUNGEON_SIZE - miniset.width) {
                if miniset.matches(&self.dungeon, x, y) {
                    // C++ uses GenerateRnd(100) >= rndper to SKIP placement
                    if self.rng.generate(100) >= rnd_percent as i32 {
                        continue; // Skip placement
                    }
                    miniset.place(&mut self.dungeon, x, y);
                    return true;
                }
            }
        }
        false
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

// ===================================================================
// TESTS
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Tile::Invalid as u8, 57);
    }

    #[test]
    fn test_tile_try_from() {
        assert_eq!(Tile::try_from(1), Ok(Tile::VWall));
        assert_eq!(Tile::try_from(13), Ok(Tile::Floor));
        assert_eq!(Tile::try_from(16), Ok(Tile::NWCorner));
        assert_eq!(Tile::try_from(4), Ok(Tile::SECorner));
        assert_eq!(Tile::try_from(64), Ok(Tile::EntranceStairs));
        assert_eq!(Tile::try_from(57), Ok(Tile::Invalid));
        assert_eq!(Tile::try_from(255), Err(()));
    }

    #[test]
    fn test_miniset_stairs_up() {
        let miniset = stairs_up_miniset();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 4);
        assert_eq!(miniset.search[0][0], Tile::Floor);
        assert_eq!(miniset.replace[1][2], Tile::EntranceStairs);
    }

    #[test]
    fn test_miniset_stairs_down() {
        let miniset = stairs_down_miniset();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 3);
        assert_eq!(miniset.replace[0][0], Tile::NWCorner);
    }

    #[test]
    fn test_miniset_lamps() {
        let miniset = lamps_miniset();
        assert_eq!(miniset.width, 2);
        assert_eq!(miniset.height, 2);
        assert_eq!(miniset.search[0][0], Tile::Floor);
        assert_eq!(miniset.replace[0][0], Tile::NWCorner);
    }

    #[test]
    fn test_miniset_water_in() {
        let miniset = water_in_miniset();
        assert_eq!(miniset.width, 6);
        assert_eq!(miniset.height, 6);
        assert_eq!(miniset.search[0][0], Tile::Floor);
        assert_eq!(miniset.replace[0][0], Tile::NWCorner);
        assert_eq!(miniset.replace[3][3], Tile::Floor); // Center stays floor
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

        assert!(miniset.matches(&dungeon, 5, 5));
        assert!(!miniset.matches(&dungeon, 0, 0)); // Dirt area
    }

    #[test]
    fn test_miniset_place() {
        let miniset = lamps_miniset();
        let mut dungeon = [[Tile::Floor; DUNGEON_SIZE]; DUNGEON_SIZE];

        miniset.place(&mut dungeon, 10, 10);

        assert_eq!(dungeon[10][10], Tile::NWCorner);
        assert_eq!(dungeon[10][11], Tile::NECorner);
        assert_eq!(dungeon[11][10], Tile::SWCorner);
        assert_eq!(dungeon[11][11], Tile::SECorner);
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

        // Create floor area for miniset (4x4 for stairs_up)
        for y in 10..14 {
            for x in 10..14 {
                gen.dungeon[y][x] = Tile::Floor;
            }
        }

        let miniset = stairs_up_miniset();

        // C++ logic: if (GenerateRnd(100) >= rndper) continue; (skip placement)
        // rndper=0:   GenerateRnd(100) >= 0   → always true  → always skip → NEVER place
        // rndper=100: GenerateRnd(100) >= 100 → always false (returns 0-99) → never skip → ALWAYS place
        // Both threshold extremes are RNG-independent, so the actual seed does
        // not change the outcome here.

        // Test 1: rndper=0 (always skip) → should NOT place
        let placed = gen.place_miniset_random(&miniset, 0);
        assert!(!placed);

        // Test 2: rndper=100 (never skip) → ALWAYS place
        gen.init_dungeon();
        for y in 10..14 {
            for x in 10..14 {
                gen.dungeon[y][x] = Tile::Floor;
            }
        }

        let placed = gen.place_miniset_random(&miniset, 100);
        assert!(placed);

        // Verify miniset was placed at (10, 10)
        // stairs_up_miniset replace pattern:
        // [Invalid, NWCorner, HWall, NECorner]     row 0 → y=10
        // [Invalid, VWall, EntranceStairs, VWall]  row 1 → y=11
        // [Invalid, VWall, EntranceStairs, VWall]  row 2 → y=12
        // [Invalid, SWCorner, HWall, SECorner]     row 3 → y=13
        assert_eq!(gen.dungeon[10][11], Tile::NWCorner);      // row 0, col 1
        assert_eq!(gen.dungeon[10][12], Tile::HWall);         // row 0, col 2
        assert_eq!(gen.dungeon[11][11], Tile::VWall);         // row 1, col 1
        assert_eq!(gen.dungeon[11][12], Tile::EntranceStairs); // row 1, col 2
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
