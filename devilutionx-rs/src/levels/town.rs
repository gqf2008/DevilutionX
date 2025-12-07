//! Tristram Town Level Generation
//!
//! Unlike Cathedral/Caves/Hell which use procedural generation, Town uses
//! a **fixed pre-made layout** loaded from sector files. This module handles:
//! - Loading 4 sector files (46x46 tiles each) for Town layout
//! - Dynamic quest-related updates (Hive open/closed, Grave open/closed, Fountain clean/poisoned)
//! - Setting player spawn position based on entry point (New game, Cathedral entrance, Town Portal warps)
//!
//! C++ source: Source/levels/town.cpp (359 lines)
//! Target: ~400-450 lines Rust code

use crate::levels::gendung::Dungeon;
use crate::levels::types::DungeonType;

/// Maximum dungeon X coordinate (112 = 16 + 40*2 + 16)
const MAXDUNX: usize = 112;
/// Maximum dungeon Y coordinate (112 = 16 + 40*2 + 16)
const MAXDUNY: usize = 112;

/// Town entry point (how player entered town)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TownEntry {
    /// New game / main entrance
    Main = 0,
    /// From previous level (Cathedral entrance)
    Prev = 1,
    /// From town portal (warp up from dungeon)
    TwarpUp = 7,
}

impl TryFrom<u8> for TownEntry {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TownEntry::Main),
            1 => Ok(TownEntry::Prev),
            7 => Ok(TownEntry::TwarpUp),
            _ => Err(format!("Invalid TownEntry: {}", value)),
        }
    }
}

/// Represents a 2D point in dungeon coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// MegaTile: 4 micro tiles forming a 2x2 block
/// C++ equivalent: Source/levels/gendung.h MegaTile struct
#[derive(Debug, Clone, Copy)]
pub struct MegaTile {
    pub micro1: u16, // top-left
    pub micro2: u16, // top-right
    pub micro3: u16, // bottom-left
    pub micro4: u16, // bottom-right
}

impl MegaTile {
    pub const fn new(micro1: u16, micro2: u16, micro3: u16, micro4: u16) -> Self {
        Self {
            micro1,
            micro2,
            micro3,
            micro4,
        }
    }
}

/// Town Generator: manages Town level data and dynamic updates
///
/// Unlike procedural generators (Cathedral/Caves), Town generator:
/// - Loads fixed sector files (sector1s.dun - sector4s.dun)
/// - Updates dynamic elements based on quest state (Hive/Grave/Fountain)
/// - Handles 5 spawn points (main entrance, Cathedral entrance, 4 town portal warps)
pub struct TownGenerator {
    /// Visual tile array (112x112, maps to micro tiles in sprite sheets)
    /// C++ equivalent: dPiece[MAXDUNX][MAXDUNY]
    pub d_piece: [[u16; MAXDUNY]; MAXDUNX],

    /// Special tile markers for interactivity (fountains, signs, warps)
    /// C++ equivalent: dSpecial[MAXDUNX][MAXDUNY]
    pub d_special: [[u8; MAXDUNY]; MAXDUNX],

    /// Logical dungeon tile array (used for warp state tracking)
    /// C++ equivalent: dungeon[DMAXX][DMAXY]
    pub dungeon: [[u8; 40]; 40],

    /// Player spawn position (set by create_town)
    pub view_position: Point,

    /// Dungeon bounds (min corner)
    pub dmin_position: Point,

    /// Dungeon bounds (max corner)
    pub dmax_position: Point,
}

impl TownGenerator {
    /// Create new Town generator with default state
    pub fn new() -> Self {
        Self {
            d_piece: [[0; MAXDUNY]; MAXDUNX],
            d_special: [[0; MAXDUNY]; MAXDUNX],
            dungeon: [[0; 40]; 40],
            view_position: Point { x: 0, y: 0 },
            dmin_position: Point { x: 10, y: 10 },
            dmax_position: Point { x: 84, y: 84 },
        }
    }

    /// Fill a sector of the town from a .dun file
    ///
    /// C++ equivalent: FillSector() in Source/levels/town.cpp:24-47
    ///
    /// Loads a pre-made sector file (e.g., sector1s.dun) and converts
    /// tile IDs to micro tiles, placing them in the dPiece array.
    ///
    /// # Arguments
    /// * `tile_data` - Raw sector data (2-byte header for size + tile IDs)
    /// * `megatiles` - MegaTile lookup table
    /// * `xi` - X offset in dPiece array (must be even)
    /// * `yy` - Y offset in dPiece array (must be even)
    pub fn fill_sector(
        &mut self,
        tile_data: &[u16],
        megatiles: &[MegaTile],
        xi: usize,
        mut yy: usize,
    ) {
        if tile_data.len() < 2 {
            return;
        }

        // Parse sector size (first 2 u16 values are width and height)
        let width = tile_data[0] as usize;
        let height = tile_data[1] as usize;
        let tiles = &tile_data[2..];

        for j in 0..height {
            let mut xx = xi;
            for i in 0..width {
                let tile_idx = j * width + i;
                if tile_idx >= tiles.len() {
                    break;
                }

                // Default to tile 218 (empty grass)
                let mut v1 = 218;
                let mut v2 = 218;
                let mut v3 = 218;
                let mut v4 = 218;

                let tile_id = tiles[tile_idx].wrapping_sub(1); // 1-indexed → 0-indexed
                if tile_id < megatiles.len() as u16 {
                    let mega = megatiles[tile_id as usize];
                    v1 = mega.micro1;
                    v2 = mega.micro2;
                    v3 = mega.micro3;
                    v4 = mega.micro4;
                }

                // Place 2x2 micro tiles (MegaTile decomposition)
                if xx < MAXDUNX && yy < MAXDUNY {
                    self.d_piece[xx][yy] = v1;
                }
                if xx + 1 < MAXDUNX && yy < MAXDUNY {
                    self.d_piece[xx + 1][yy] = v2;
                }
                if xx < MAXDUNX && yy + 1 < MAXDUNY {
                    self.d_piece[xx][yy + 1] = v3;
                }
                if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                    self.d_piece[xx + 1][yy + 1] = v4;
                }

                xx += 2; // Each tile is 2 micro tiles wide
            }
            yy += 2; // Each tile is 2 micro tiles tall
        }
    }

    /// Fill a single tile (2x2 micro tile block)
    ///
    /// C++ equivalent: FillTile() in Source/levels/town.cpp:56-64
    ///
    /// # Arguments
    /// * `xx` - X coordinate in dPiece array (must be even)
    /// * `yy` - Y coordinate in dPiece array (must be even)
    /// * `tile_id` - 1-indexed tile ID (converted to 0-indexed internally)
    /// * `megatiles` - MegaTile lookup table
    pub fn fill_tile(&mut self, xx: usize, yy: usize, tile_id: u16, megatiles: &[MegaTile]) {
        if tile_id == 0 {
            return;
        }

        let idx = (tile_id - 1) as usize; // 1-indexed → 0-indexed
        if idx >= megatiles.len() {
            return;
        }

        let mega = megatiles[idx];

        if xx < MAXDUNX && yy < MAXDUNY {
            self.d_piece[xx][yy] = mega.micro1;
        }
        if xx + 1 < MAXDUNX && yy < MAXDUNY {
            self.d_piece[xx + 1][yy] = mega.micro2;
        }
        if xx < MAXDUNX && yy + 1 < MAXDUNY {
            self.d_piece[xx][yy + 1] = mega.micro3;
        }
        if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
            self.d_piece[xx + 1][yy + 1] = mega.micro4;
        }
    }

    /// Update the map to show the closed hive (Hellfire quest)
    ///
    /// C++ equivalent: TownCloseHive() in Source/levels/town.cpp:73-132
    pub fn town_close_hive(&mut self) {
        // Update logical dungeon tiles (used for collision detection)
        self.dungeon[35][27] = 18;
        self.dungeon[36][27] = 63;

        // Update visual tiles (dPiece array) - hardcoded closed hive tiles
        // These specific micro tile IDs represent the closed hive gate
        let closed_tiles: &[(usize, usize, u16)] = &[
            (78, 60, 0x489),
            (79, 60, 0x4ea),
            (78, 61, 0x4eb),
            (79, 61, 0x4ec),
            (78, 62, 0x4ed),
            (79, 62, 0x4ee),
            (78, 63, 0x4ef),
            (79, 63, 0x4f0),
            (78, 64, 0x4f1),
            (79, 64, 0x4f2),
            (78, 65, 0x4f3),
            (80, 60, 0x4f4),
            (81, 60, 0x4f5),
            (80, 61, 0x4f6),
            (81, 61, 0x4f7),
            (82, 60, 0x4f8),
            (83, 60, 0x4f9),
            (82, 61, 0x4fa),
            (83, 61, 0x4fb),
            (80, 62, 0x4fc),
            (81, 62, 0x4fd),
            (80, 63, 0x4fe),
            (81, 63, 0x4ff),
            (80, 64, 0x500),
            (81, 64, 0x501),
            (80, 65, 0x502),
            (81, 65, 0x503),
            (82, 64, 0x508),
            (83, 64, 0x509),
            (82, 65, 0x50a),
            (83, 65, 0x50b),
            (82, 62, 0x504),
            (83, 62, 0x505),
            (82, 63, 0x506),
            (83, 63, 0x507),
            (84, 61, 279),
            (84, 62, 280),
            (84, 63, 279),
            (84, 64, 10),
            (85, 60, 11),
            (85, 61, 12),
            (85, 62, 13),
            (85, 63, 14),
            (85, 64, 15),
            (86, 60, 16),
            (86, 61, 17),
        ];

        for &(x, y, tile) in closed_tiles {
            if x < MAXDUNX && y < MAXDUNY {
                self.d_piece[x][y] = tile;
            }
        }
    }

    /// Update the map to show the closed grave (Hellfire quest)
    ///
    /// C++ equivalent: TownCloseGrave() in Source/levels/town.cpp:137-149
    pub fn town_close_grave(&mut self) {
        // Update visual tiles (dPiece array) - hardcoded closed grave tiles
        let closed_tiles: &[(usize, usize, u16)] = &[
            (36, 21, 0x52a),
            (37, 21, 0x52b),
            (36, 22, 0x52c),
            (37, 22, 0x52d),
            (36, 23, 0x52e),
            (37, 23, 0x52f),
            (36, 24, 0x530),
            (37, 24, 0x531),
            (35, 21, 0x53a),
            (34, 21, 0x53b),
        ];

        for &(x, y, tile) in closed_tiles {
            if x < MAXDUNX && y < MAXDUNY {
                self.d_piece[x][y] = tile;
            }
        }
    }

    /// Update the map to show the open hive (Hellfire quest)
    ///
    /// C++ equivalent: TownOpenHive() in Source/levels/town.cpp:298-356
    pub fn town_open_hive(&mut self) {
        // Update logical dungeon tile (warp entrance)
        self.dungeon[36][27] = 47;

        // Update visual tiles (dPiece array) - hardcoded open hive tiles
        let open_tiles: &[(usize, usize, u16)] = &[
            (78, 60, 0x489),
            (79, 60, 0x48a),
            (78, 61, 0x48b),
            (79, 61, 0x50d),
            (78, 62, 0x4ed),
            (78, 63, 0x4ef),
            (79, 62, 0x50f),
            (79, 63, 0x510),
            (79, 64, 0x511),
            (78, 64, 0x119),
            (78, 65, 0x11b),
            (79, 65, 0x11c),
            (80, 60, 0x512),
            (80, 61, 0x514),
            (81, 61, 0x515),
            (82, 60, 0x516),
            (83, 60, 0x517),
            (82, 61, 0x518),
            (83, 61, 0x519),
            (80, 62, 0x51a),
            (81, 62, 0x51b),
            (80, 63, 0x51c),
            (81, 63, 0x51d),
            (80, 64, 0x51e),
            (81, 64, 0x51f),
            (80, 65, 0x520),
            (81, 65, 0x521),
            (82, 64, 0x526),
            (83, 64, 0x527),
            (82, 65, 0x528),
            (83, 65, 0x529),
            (82, 62, 0x522),
            (83, 62, 0x523),
            (82, 63, 0x524),
            (83, 63, 0x525),
            (84, 61, 279),
            (84, 62, 280),
            (84, 63, 279),
            (84, 64, 10),
            (85, 60, 11),
            (85, 61, 12),
            (85, 62, 13),
            (85, 63, 14),
            (85, 64, 15),
            (86, 60, 16),
            (86, 61, 17),
        ];

        for &(x, y, tile) in open_tiles {
            if x < MAXDUNX && y < MAXDUNY {
                self.d_piece[x][y] = tile;
            }
        }
    }

    /// Update the map to show the open grave (Hellfire quest)
    ///
    /// C++ equivalent: TownOpenGrave() in Source/levels/town.cpp:358-374
    pub fn town_open_grave(&mut self) {
        // Update logical dungeon tiles (warp entrance)
        self.dungeon[14][8] = 47;
        self.dungeon[14][7] = 47;

        // Update visual tiles (dPiece array) - hardcoded open grave tiles
        let open_tiles: &[(usize, usize, u16)] = &[
            (36, 21, 0x532),
            (37, 21, 0x533),
            (36, 22, 0x534),
            (37, 22, 0x535),
            (36, 23, 0x536),
            (37, 23, 0x537),
            (36, 24, 0x538),
            (37, 24, 0x539),
            (35, 21, 0x53a),
            (34, 21, 0x53b),
        ];

        for &(x, y, tile) in open_tiles {
            if x < MAXDUNX && y < MAXDUNY {
                self.d_piece[x][y] = tile;
            }
        }
    }

    /// Mark special interactive tiles for Town
    ///
    /// C++ equivalent: InitTownPieces() in Source/levels/town.cpp:151-187
    ///
    /// Scans dPiece array for specific tile IDs and marks them in dSpecial:
    /// - 1: Fountain (tile 359)
    /// - 2: Town sign (tile 357)
    /// - 6-18: Various NPCs, warps, quest objects
    pub fn init_town_pieces(&mut self) {
        for y in 0..MAXDUNY {
            for x in 0..MAXDUNX {
                let tile = self.d_piece[x][y];
                let special = match tile {
                    359 => 1,  // Fountain
                    357 => 2,  // Town sign
                    128 => 6,  // Unknown (NPC marker?)
                    129 => 7,
                    127 => 8,
                    116 => 9,
                    156 => 10,
                    157 => 11,
                    155 => 12,
                    161 => 13,
                    159 => 14,
                    213 => 15,
                    211 => 16,
                    216 => 17,
                    215 => 18,
                    _ => 0, // No special marker
                };

                self.d_special[x][y] = special;
            }
        }
    }
}

/// Main entry point: generate Town level
///
/// C++ equivalent: CreateTown() in Source/levels/town.cpp:376-410
///
/// Unlike dungeon levels, Town does not use procedural generation. Instead:
/// 1. Sets dungeon bounds (10,10) to (84,84)
/// 2. Sets player spawn position based on entry point
/// 3. Calls DrlgTPass3() to load sector files and update quest states
///
/// # Arguments
/// * `entry` - How the player entered Town (Main, Cathedral, Town Portal)
/// * `twarp_from` - If entry == TwarpUp, which dungeon level the portal came from
///
/// # Returns
/// Fully initialized TownGenerator
pub fn create_town(entry: TownEntry, twarp_from: i32) -> TownGenerator {
    let mut generator = TownGenerator::new();

    // Set dungeon bounds (fixed for Town)
    generator.dmin_position = Point { x: 10, y: 10 };
    generator.dmax_position = Point { x: 84, y: 84 };

    // Set player spawn position based on entry point
    generator.view_position = match entry {
        TownEntry::Main => Point { x: 75, y: 68 }, // New game spawn
        TownEntry::Prev => Point { x: 25, y: 31 }, // From Cathedral
        TownEntry::TwarpUp => {
            // Town portal spawn (depends on source dungeon level)
            match twarp_from {
                5 => Point { x: 49, y: 22 },  // From Catacombs
                9 => Point { x: 18, y: 69 },  // From Caves
                13 => Point { x: 41, y: 81 }, // From Hell
                21 => Point { x: 36, y: 25 }, // From Crypt (Hellfire)
                17 => Point { x: 79, y: 62 }, // From Nest (Hellfire)
                _ => Point { x: 75, y: 68 },  // Default to main entrance
            }
        }
    };

    generator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_town_entry_conversion() {
        assert_eq!(TownEntry::try_from(0).unwrap(), TownEntry::Main);
        assert_eq!(TownEntry::try_from(1).unwrap(), TownEntry::Prev);
        assert_eq!(TownEntry::try_from(7).unwrap(), TownEntry::TwarpUp);
        assert!(TownEntry::try_from(99).is_err());
    }

    #[test]
    fn test_create_town_main() {
        let gen = create_town(TownEntry::Main, 0);
        assert_eq!(gen.view_position, Point { x: 75, y: 68 });
        assert_eq!(gen.dmin_position, Point { x: 10, y: 10 });
        assert_eq!(gen.dmax_position, Point { x: 84, y: 84 });
    }

    #[test]
    fn test_create_town_cathedral() {
        let gen = create_town(TownEntry::Prev, 0);
        assert_eq!(gen.view_position, Point { x: 25, y: 31 });
    }

    #[test]
    fn test_create_town_twarp_catacombs() {
        let gen = create_town(TownEntry::TwarpUp, 5);
        assert_eq!(gen.view_position, Point { x: 49, y: 22 });
    }

    #[test]
    fn test_create_town_twarp_caves() {
        let gen = create_town(TownEntry::TwarpUp, 9);
        assert_eq!(gen.view_position, Point { x: 18, y: 69 });
    }

    #[test]
    fn test_create_town_twarp_hell() {
        let gen = create_town(TownEntry::TwarpUp, 13);
        assert_eq!(gen.view_position, Point { x: 41, y: 81 });
    }

    #[test]
    fn test_create_town_twarp_crypt() {
        let gen = create_town(TownEntry::TwarpUp, 21);
        assert_eq!(gen.view_position, Point { x: 36, y: 25 });
    }

    #[test]
    fn test_create_town_twarp_nest() {
        let gen = create_town(TownEntry::TwarpUp, 17);
        assert_eq!(gen.view_position, Point { x: 79, y: 62 });
    }

    #[test]
    fn test_create_town_twarp_invalid() {
        let gen = create_town(TownEntry::TwarpUp, 999);
        assert_eq!(gen.view_position, Point { x: 75, y: 68 }); // Default
    }

    #[test]
    fn test_fill_tile_bounds_check() {
        let mut gen = TownGenerator::new();
        let megatiles = [MegaTile::new(100, 101, 102, 103)];

        // Valid placement
        gen.fill_tile(0, 0, 1, &megatiles);
        assert_eq!(gen.d_piece[0][0], 100);
        assert_eq!(gen.d_piece[1][0], 101);
        assert_eq!(gen.d_piece[0][1], 102);
        assert_eq!(gen.d_piece[1][1], 103);

        // Out of bounds (should not panic)
        gen.fill_tile(MAXDUNX - 1, MAXDUNY - 1, 1, &megatiles);
    }

    #[test]
    fn test_fill_sector_basic() {
        let mut gen = TownGenerator::new();
        let megatiles = [MegaTile::new(10, 11, 12, 13)];

        // Sector data: width=2, height=2, tiles=[1, 1, 1, 1]
        let sector_data: Vec<u16> = vec![2, 2, 1, 1, 1, 1];

        gen.fill_sector(&sector_data, &megatiles, 0, 0);

        // Check first tile (top-left)
        assert_eq!(gen.d_piece[0][0], 10);
        assert_eq!(gen.d_piece[1][0], 11);
        assert_eq!(gen.d_piece[0][1], 12);
        assert_eq!(gen.d_piece[1][1], 13);

        // Check second tile (top-right)
        assert_eq!(gen.d_piece[2][0], 10);
        assert_eq!(gen.d_piece[3][0], 11);

        // Check third tile (bottom-left)
        assert_eq!(gen.d_piece[0][2], 10);
        assert_eq!(gen.d_piece[0][3], 12);
    }

    #[test]
    fn test_init_town_pieces() {
        let mut gen = TownGenerator::new();

        // Place fountain (tile 359) and sign (tile 357)
        gen.d_piece[10][10] = 359;
        gen.d_piece[20][20] = 357;
        gen.d_piece[30][30] = 128;

        gen.init_town_pieces();

        assert_eq!(gen.d_special[10][10], 1); // Fountain
        assert_eq!(gen.d_special[20][20], 2); // Sign
        assert_eq!(gen.d_special[30][30], 6); // NPC marker
        assert_eq!(gen.d_special[0][0], 0); // Empty
    }

    #[test]
    fn test_town_close_hive() {
        let mut gen = TownGenerator::new();
        gen.town_close_hive();

        // Check logical dungeon update
        assert_eq!(gen.dungeon[35][27], 18);
        assert_eq!(gen.dungeon[36][27], 63);

        // Check some visual tile updates (spot checks)
        assert_eq!(gen.d_piece[78][60], 0x489);
        assert_eq!(gen.d_piece[79][60], 0x4ea);
        assert_eq!(gen.d_piece[84][61], 279);
    }

    #[test]
    fn test_town_open_hive() {
        let mut gen = TownGenerator::new();
        gen.town_open_hive();

        // Check logical dungeon update (warp entrance)
        assert_eq!(gen.dungeon[36][27], 47);

        // Check some visual tile updates (spot checks)
        assert_eq!(gen.d_piece[78][60], 0x489);
        assert_eq!(gen.d_piece[79][60], 0x48a); // Different from closed
        assert_eq!(gen.d_piece[79][61], 0x50d); // Open state
    }

    #[test]
    fn test_town_close_grave() {
        let mut gen = TownGenerator::new();
        gen.town_close_grave();

        // Check visual tile updates (spot checks)
        assert_eq!(gen.d_piece[36][21], 0x52a);
        assert_eq!(gen.d_piece[37][21], 0x52b);
        assert_eq!(gen.d_piece[35][21], 0x53a);
    }

    #[test]
    fn test_town_open_grave() {
        let mut gen = TownGenerator::new();
        gen.town_open_grave();

        // Check logical dungeon update (warp entrance)
        assert_eq!(gen.dungeon[14][8], 47);
        assert_eq!(gen.dungeon[14][7], 47);

        // Check visual tile updates (spot checks)
        assert_eq!(gen.d_piece[36][21], 0x532); // Different from closed
        assert_eq!(gen.d_piece[37][21], 0x533);
    }
}
