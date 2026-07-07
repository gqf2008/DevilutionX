// drlg_l3.rs - Caves (L9-L12) & Hive (L15) Level Generation
//
// Implements the procedural generation algorithm for the Caves tileset.
//
// C++ Reference: Source/levels/drlg_l3.cpp (~2,220 lines)
//
// Key Features:
// - Cellular automata-based cave generation
// - River system (lava or water based on level)
// - ~35 Minisets (stairs, stalactites, crevices, islands)
// - L3 (Caves, levels 9-12) and L6 (Nest/Hive, level 15)
// - Pool/island formation algorithm

use crate::engine::types::{Point, Displacement};
use crate::levels::gendung::Dungeon;
use crate::levels::types::{DungeonType, LevelEntry, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
use crate::utils::random::Rng;

// =============================================================================
// Constants
// =============================================================================

/// Conversion table for 2x2 wall patterns (16 patterns)
/// C++ equivalent: L3ConvTbl[16]
const L3_CONV_TABLE: [u8; 16] = [
    8, 11, 3, 10, 1, 9, 12, 12, 6, 13, 4, 13, 2, 14, 5, 7
];

// =============================================================================
// Miniset Definitions
// =============================================================================

/// Miniset data structure for L3
#[derive(Debug, Clone)]
pub struct Miniset {
    width: usize,
    height: usize,
    search: Vec<Vec<u8>>,
    replace: Vec<Vec<u8>>,
}

impl Miniset {
    pub fn new(width: usize, height: usize, search: Vec<Vec<u8>>, replace: Vec<Vec<u8>>) -> Self {
        Miniset {
            width,
            height,
            search,
            replace,
        }
    }
}

// =============================================================================
// Stairs Minisets (L3: Caves)
// =============================================================================

/// Stairs up to previous level (L3)
pub fn miniset_l3_up() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![8,8,0],vec![10,10,0],vec![7,7,0]],
        vec![vec![51,50,0],vec![48,49,0],vec![0,0,0]])
}

/// Stairs down to next level (L3)
pub fn miniset_l3_down() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![8,9,7],vec![8,9,7],vec![0,0,0]],
        vec![vec![0,47,0],vec![0,46,0],vec![0,0,0]])
}

/// Stairs up to town/previous (L3 Hold Warp)
pub fn miniset_l3_holdwarp() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![8,8,0],vec![10,10,0],vec![7,7,0]],
        vec![vec![125,125,0],vec![125,125,0],vec![0,0,0]])
}

// =============================================================================
// Stairs Minisets (L6: Hive/Nest)
// =============================================================================

/// Stairs up to previous level (L6 Hive)
pub fn miniset_l6_up() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![8,8,0],vec![10,10,0],vec![7,7,0]],
        vec![vec![20,19,0],vec![17,18,0],vec![0,0,0]])
}

/// Stairs down to next level (L6 Hive)
pub fn miniset_l6_down() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![8,9,7],vec![8,9,7],vec![0,0,0]],
        vec![vec![0,16,0],vec![0,15,0],vec![0,0,0]])
}

/// Stairs up to previous (L6 Hold Warp)
pub fn miniset_l6_holdwarp() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![8,8,0],vec![10,10,0],vec![7,7,0]],
        vec![vec![24,23,0],vec![21,22,0],vec![0,0,0]])
}

// =============================================================================
// Stalactite Minisets (L3TITE1-13)
// =============================================================================

/// Stalactite variant 1 (4x4)
pub fn miniset_l3_tite1() -> Miniset {
    Miniset::new(4, 4,
        vec![vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7]],
        vec![vec![0,0,0,0],vec![0,57,58,0],vec![0,56,55,0],vec![0,0,0,0]])
}

/// Stalactite variant 2 (4x4)
pub fn miniset_l3_tite2() -> Miniset {
    Miniset::new(4, 4,
        vec![vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7]],
        vec![vec![0,0,0,0],vec![0,61,62,0],vec![0,60,59,0],vec![0,0,0,0]])
}

/// Stalactite variant 3 (4x4)
pub fn miniset_l3_tite3() -> Miniset {
    Miniset::new(4, 4,
        vec![vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7]],
        vec![vec![0,0,0,0],vec![0,65,66,0],vec![0,64,63,0],vec![0,0,0,0]])
}

// Stalactite variants 6-13 (similar structure, different tiles)
pub fn miniset_l3_tite6() -> Miniset {
    Miniset::new(4, 4,
        vec![vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7],vec![7,7,7,7]],
        vec![vec![0,0,0,0],vec![0,69,70,0],vec![0,68,67,0],vec![0,0,0,0]])
}

pub fn miniset_l3_tite7() -> Miniset {
    Miniset::new(5, 5,
        vec![vec![7,7,7,7,7],vec![7,7,7,7,7],vec![7,7,7,7,7],vec![7,7,7,7,7],vec![7,7,7,7,7]],
        vec![vec![0,0,0,0,0],vec![0,0,0,0,0],vec![0,0,72,73,0],vec![0,0,71,74,0],vec![0,0,0,0,0]])
}

pub fn miniset_l3_tite8() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![7,7,7],vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,77,0],vec![0,0,0]])
}

pub fn miniset_l3_tite9() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![7,7,7],vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,79,0],vec![0,0,0]])
}

pub fn miniset_l3_tite10() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![7,7,7],vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,80,0],vec![0,0,0]])
}

pub fn miniset_l3_tite11() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![7,7,7],vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,81,0],vec![0,0,0]])
}

pub fn miniset_l3_tite12() -> Miniset {
    Miniset::new(2, 2,
        vec![vec![7,7],vec![7,7]],
        vec![vec![0,0],vec![0,83]])
}

pub fn miniset_l3_tite13() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![7,7,7],vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,0,0],vec![0,0,167]])
}

// =============================================================================
// Crevice Minisets (L3CREV1-11)
// =============================================================================

pub fn miniset_l3_crev1() -> Miniset {
    Miniset::new(2, 2, vec![vec![7,7],vec![7,7]], vec![vec![0,0],vec![0,85]])
}

pub fn miniset_l3_crev2() -> Miniset {
    Miniset::new(2, 2, vec![vec![7,7],vec![7,7]], vec![vec![0,0],vec![0,87]])
}

pub fn miniset_l3_crev3() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,89,0]])
}

pub fn miniset_l3_crev4() -> Miniset {
    Miniset::new(2, 2, vec![vec![7,7],vec![7,7]], vec![vec![0,0],vec![0,91]])
}

pub fn miniset_l3_crev5() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,93,0]])
}

pub fn miniset_l3_crev6() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,95,0]])
}

pub fn miniset_l3_crev7() -> Miniset {
    Miniset::new(2, 2, vec![vec![7,7],vec![7,7]], vec![vec![0,0],vec![0,97]])
}

pub fn miniset_l3_crev8() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,99,0]])
}

pub fn miniset_l3_crev9() -> Miniset {
    Miniset::new(2, 2, vec![vec![7,7],vec![7,7]], vec![vec![0,0],vec![0,101]])
}

pub fn miniset_l3_crev10() -> Miniset {
    Miniset::new(2, 2, vec![vec![7,7],vec![7,7]], vec![vec![0,0],vec![0,103]])
}

pub fn miniset_l3_crev11() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![7,7,7],vec![7,7,7]],
        vec![vec![0,0,0],vec![0,105,0]])
}

// =============================================================================
// Island Minisets (L3ISLE1-5, L6ISLE1-5)
// =============================================================================

/// Island variant 1 for L3 (3x3)
pub fn miniset_l3_isle1() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![15,15,15],vec![15,7,15],vec![15,15,15]],
        vec![vec![0,0,0],vec![0,143,0],vec![0,0,0]])
}

/// Island variant 2 for L3 (3x3)
pub fn miniset_l3_isle2() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![15,15,15],vec![15,7,15]],
        vec![vec![0,0,0],vec![0,145,0]])
}

/// Island variant 3 for L3 (3x3)
pub fn miniset_l3_isle3() -> Miniset {
    Miniset::new(3, 3,
        vec![vec![15,15,15],vec![7,7,15],vec![15,15,15]],
        vec![vec![0,0,0],vec![0,147,0],vec![0,0,0]])
}

/// Island variant 4 for L3 (3x3)
pub fn miniset_l3_isle4() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![15,15,7],vec![15,7,15]],
        vec![vec![0,0,0],vec![0,149,0]])
}

/// Island variant 5 for L3 (3x2)
pub fn miniset_l3_isle5() -> Miniset {
    Miniset::new(2, 3,
        vec![vec![15,15],vec![7,15],vec![15,15]],
        vec![vec![0,0],vec![151,0],vec![0,0]])
}

// =============================================================================
// Island Minisets for Hive/Nest (L6ISLE1-5)
// =============================================================================

/// Island variant 1 for L6 Hive (2x3)
pub fn miniset_l6_isle1() -> Miniset {
    Miniset::new(2, 3,
        vec![vec![5,14],vec![4,9],vec![13,12]],
        vec![vec![7,7],vec![7,7],vec![7,7]])
}

/// Island variant 2 for L6 Hive (3x2)
pub fn miniset_l6_isle2() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![5,2,14],vec![13,10,12]],
        vec![vec![7,7,7],vec![7,7,7]])
}

/// Island variant 3 for L6 Hive (2x3, with decorative tiles)
pub fn miniset_l6_isle3() -> Miniset {
    Miniset::new(2, 3,
        vec![vec![5,14],vec![4,9],vec![13,12]],
        vec![vec![107,115],vec![119,122],vec![131,123]])
}

/// Island variant 4 for L6 Hive (3x2, with decorative tiles)
pub fn miniset_l6_isle4() -> Miniset {
    Miniset::new(3, 2,
        vec![vec![5,2,14],vec![13,10,12]],
        vec![vec![107,120,115],vec![131,121,123]])
}

/// Island variant 5 for L6 Hive (2x2)
pub fn miniset_l6_isle5() -> Miniset {
    Miniset::new(2, 2,
        vec![vec![5,14],vec![13,12]],
        vec![vec![7,7],vec![7,7]])
}

// =============================================================================
// Caves Generator
// =============================================================================

pub struct CavesGenerator {
    /// Seeded RNG (matches Diablo's `SetRndSeed`/`GenerateRnd`/`FlipCoin`).
    ///
    /// All randomness in the Caves generator flows through the three helpers
    /// `random_range`, `flip_coin` and `flip_coin_n`, which delegate to this
    /// `Rng`. This makes level generation bit-for-bit reproducible for a given
    /// seed, matching the C++ engine (`Source/engine/random.cpp`).
    rng: Rng,
    /// Predungeon grid (cellular automata workspace)
    predungeon: [[u8; MAXDUNY]; MAXDUNX],
    /// Lockout counter (for door placement)
    lockout_count: i32,
}

impl CavesGenerator {
    pub fn new() -> Self {
        CavesGenerator {
            rng: Rng::with_default_seed(),
            predungeon: [[0; MAXDUNY]; MAXDUNX],
            lockout_count: 0,
        }
    }

    /// Generate L3 cave dungeon
    /// C++ equivalent: GenerateLevel
    pub fn generate(&mut self, dungeon: &mut Dungeon, seed: u32, level: u8, entry: LevelEntry) -> bool {
        // Seed the Diablo LCG before generation so results are deterministic
        // and reproduce the C++ `SetRndSeed(seed)` behaviour exactly.
        self.rng.set_seed(seed);

        // Generation loop (retry until valid dungeon)
        loop {
            self.init_dungeon_flags();

            // Create initial room
            let x1 = self.random_range(10, 30);
            let y1 = self.random_range(10, 30);
            let x2 = x1 + 2;
            let y2 = y1 + 2;
            self.fill_room(x1, y1, x2, y2);

            // Create blocks in 4 directions
            self.create_block(x1, y1, 2, 0);
            self.create_block(x2, y1, 2, 1);
            self.create_block(x1, y2, 2, 2);
            self.create_block(x1, y1, 2, 3);

            // TODO: Quest room handling (Q_ANVIL)

            // Apply cellular automata rules
            self.fill_diagonals();
            self.fill_singles();
            self.fill_straights();
            self.fill_diagonals();
            self.edges();

            // Copy to dungeon for validation
            self.copy_to_dungeon(dungeon);

            // Check floor area and connectivity
            let floor_area = self.get_floor_area(dungeon);
            if floor_area < 600 || !self.lockout(dungeon) {
                continue; // Retry generation
            }

            // Convert predungeon to final dungeon tiles (MakeMegas)
            self.make_megas(dungeon);

            // Place stairs
            if !self.place_stairs(dungeon, level, entry) {
                continue; // Retry if stairs placement failed
            }

            // River/Pool generation (for L3 Caves, not Hive)
            if level >= 9 && level <= 12 {
                self.river(dungeon);
                self.pool_fix(dungeon);

                // TODO: Warp (town warp special handling)

                // Place L3 Isle minisets
                self.place_miniset_random(dungeon, &miniset_l3_isle1(), 70);
                self.place_miniset_random(dungeon, &miniset_l3_isle2(), 70);
                self.place_miniset_random(dungeon, &miniset_l3_isle3(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_isle4(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_isle1(), 100);
                self.place_miniset_random(dungeon, &miniset_l3_isle2(), 100);
                self.place_miniset_random(dungeon, &miniset_l3_isle5(), 90);

                // TODO: HallOfHeroes, River (second call in C++)

                // Place TITE minisets (stalactites)
                self.place_miniset_random(dungeon, &miniset_l3_tite1(), 10);
                self.place_miniset_random(dungeon, &miniset_l3_tite2(), 10);
                self.place_miniset_random(dungeon, &miniset_l3_tite3(), 10);
                self.place_miniset_random(dungeon, &miniset_l3_tite6(), 20);
                self.place_miniset_random(dungeon, &miniset_l3_tite7(), 20);
                self.place_miniset_random(dungeon, &miniset_l3_tite8(), 20);
                self.place_miniset_random(dungeon, &miniset_l3_tite9(), 20);
                self.place_miniset_random(dungeon, &miniset_l3_tite10(), 20);
                self.place_miniset_random(dungeon, &miniset_l3_tite11(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_tite12(), 20);
                self.place_miniset_random(dungeon, &miniset_l3_tite13(), 20);

                // Place CREV minisets (crevices)
                self.place_miniset_random(dungeon, &miniset_l3_crev1(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev2(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev3(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev4(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev5(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev6(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev7(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev8(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev9(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev10(), 30);
                self.place_miniset_random(dungeon, &miniset_l3_crev11(), 30);

                // Place 1x1 minisets (single tile replacements)
                self.place_miniset_random_1x1(dungeon, 7, 106, 25);
                self.place_miniset_random_1x1(dungeon, 7, 107, 25);
                self.place_miniset_random_1x1(dungeon, 7, 108, 25);
                self.place_miniset_random_1x1(dungeon, 9, 109, 25);
                self.place_miniset_random_1x1(dungeon, 10, 110, 25);
            }

            // Place lava pools
            if self.place_lava_pool(dungeon) {
                break; // Successfully generated dungeon with pool
            }

            // If no pool placed, retry (matches C++ behavior)
        }

        true
    }

    /// Initialize dungeon grid to zeros
    /// C++ equivalent: InitDungeonFlags
    fn init_dungeon_flags(&mut self) {
        self.predungeon = [[0; MAXDUNY]; MAXDUNX];
        self.lockout_count = 0;
    }

    /// Fill a rectangular room with floor tiles
    /// C++ equivalent: FillRoom
    fn fill_room(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        // Check boundaries
        if x1 <= 1 || x2 >= DMAXX - 2 || y1 <= 1 || y2 >= DMAXY - 2 {
            return false;
        }

        // Check if area is empty
        let mut v = 0;
        for j in y1..=y2 {
            for i in x1..=x2 {
                v += self.predungeon[i][j];
            }
        }

        if v != 0 {
            return false;
        }

        // Fill interior
        for j in (y1 + 1)..y2 {
            for i in (x1 + 1)..x2 {
                self.predungeon[i][j] = 1;
            }
        }

        // Randomly fill borders
        for j in y1..=y2 {
            if self.flip_coin() {
                self.predungeon[x1][j] = 1;
            }
            if self.flip_coin() {
                self.predungeon[x2][j] = 1;
            }
        }

        for i in x1..=x2 {
            if self.flip_coin() {
                self.predungeon[i][y1] = 1;
            }
            if self.flip_coin() {
                self.predungeon[i][y2] = 1;
            }
        }

        true
    }

    /// Create a block in specified direction
    /// C++ equivalent: CreateBlock
    fn create_block(&mut self, x: usize, y: usize, obs: usize, dir: usize) {
        let blksizex = self.random_range(3, 5);
        let blksizey = self.random_range(3, 5);

        let (x1, y1, x2, y2) = match dir {
            0 => {
                // Up
                let y2 = y.saturating_sub(1);
                let y1 = y2.saturating_sub(blksizey);
                let x1 = if blksizex < obs {
                    x + self.random_range(0, blksizex)
                } else if blksizex == obs {
                    x
                } else {
                    x.saturating_sub(self.random_range(0, blksizex))
                };
                let x2 = x1 + blksizex;
                (x1, y1, x2, y2)
            }
            1 => {
                // Right
                let x1 = x + 1;
                let x2 = x1 + blksizex;
                let y1 = if blksizey < obs {
                    y + self.random_range(0, blksizey)
                } else if blksizey == obs {
                    y
                } else {
                    y.saturating_sub(self.random_range(0, blksizey))
                };
                let y2 = y1 + blksizey;
                (x1, y1, x2, y2)
            }
            2 => {
                // Down
                let y1 = y + 1;
                let y2 = y1 + blksizey;
                let x1 = if blksizex < obs {
                    x + self.random_range(0, blksizex)
                } else if blksizex == obs {
                    x
                } else {
                    x.saturating_sub(self.random_range(0, blksizex))
                };
                let x2 = x1 + blksizex;
                (x1, y1, x2, y2)
            }
            _ => {
                // Left (dir == 3)
                let x2 = x.saturating_sub(1);
                let x1 = x2.saturating_sub(blksizex);
                let y1 = if blksizey < obs {
                    y + self.random_range(0, blksizey)
                } else if blksizey == obs {
                    y
                } else {
                    y.saturating_sub(self.random_range(0, blksizey))
                };
                let y2 = y1 + blksizey;
                (x1, y1, x2, y2)
            }
        };

        if self.fill_room(x1, y1, x2, y2) {
            if self.flip_coin_n(4) {
                return;
            }

            // Recursively create blocks
            if dir != 2 {
                self.create_block(x1, y1, blksizey, 0);
            }
            if dir != 3 {
                self.create_block(x2, y1, blksizex, 1);
            }
            if dir != 0 {
                self.create_block(x1, y2, blksizey, 2);
            }
            if dir != 1 {
                self.create_block(x1, y1, blksizex, 3);
            }
        }
    }

    /// Fill diagonal connections in cellular automata
    /// C++ equivalent: FillDiagonals
    fn fill_diagonals(&mut self) {
        for j in 0..(DMAXY - 1) {
            for i in 0..(DMAXX - 1) {
                let v = self.predungeon[i + 1][j + 1]
                    + 2 * self.predungeon[i][j + 1]
                    + 4 * self.predungeon[i + 1][j]
                    + 8 * self.predungeon[i][j];

                if v == 6 {
                    if self.flip_coin() {
                        self.predungeon[i][j] = 1;
                    } else {
                        self.predungeon[i + 1][j + 1] = 1;
                    }
                } else if v == 9 {
                    if self.flip_coin() {
                        self.predungeon[i + 1][j] = 1;
                    } else {
                        self.predungeon[i][j + 1] = 1;
                    }
                }
            }
        }
    }

    /// Fill single isolated cells
    /// C++ equivalent: FillSingles
    fn fill_singles(&mut self) {
        for j in 1..(DMAXY - 1) {
            for i in 1..(DMAXX - 1) {
                if self.predungeon[i][j] == 0
                    && self.predungeon[i][j - 1] + self.predungeon[i - 1][j - 1] + self.predungeon[i + 1][j - 1] == 3
                    && self.predungeon[i + 1][j] + self.predungeon[i - 1][j] == 2
                    && self.predungeon[i][j + 1] + self.predungeon[i - 1][j + 1] + self.predungeon[i + 1][j + 1] == 3
                {
                    self.predungeon[i][j] = 1;
                }
            }
        }
    }

    /// Fill straight connections
    /// C++ equivalent: FillStraights
    fn fill_straights(&mut self) {
        for j in 1..(DMAXY - 1) {
            for i in 1..(DMAXX - 1) {
                if self.predungeon[i][j] == 0 {
                    let horiz = self.predungeon[i - 1][j] + self.predungeon[i + 1][j];
                    let vert = self.predungeon[i][j - 1] + self.predungeon[i][j + 1];

                    if horiz == 2 && vert == 0 {
                        self.predungeon[i][j] = 1;
                    }
                    if horiz == 0 && vert == 2 {
                        self.predungeon[i][j] = 1;
                    }
                }
            }
        }
    }

    /// Process edges of the dungeon
    /// C++ equivalent: Edges
    fn edges(&mut self) {
        // Set all edges to wall
        for i in 0..DMAXX {
            self.predungeon[i][0] = 0;
            self.predungeon[i][DMAXY - 1] = 0;
        }
        for j in 0..DMAXY {
            self.predungeon[0][j] = 0;
            self.predungeon[DMAXX - 1][j] = 0;
        }
    }

    /// Random integer in `[min, max)`.
    ///
    /// Mirrors C++ `GenerateRnd(max - min) + min`. `min >= max` yields `min`
    /// without advancing the engine (matching the old `GenerateRnd(<=0)` guard).
    /// Uses the deterministic Borland LCG via [`Rng`], so identical seeds
    /// produce identical sequences.
    fn random_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        let range = (max - min) as i32;
        min + self.rng.random_less_than(range) as usize
    }

    /// Flip coin with a 1-in-2 chance (`GenerateRnd(2) == 0`).
    ///
    /// Mirrors C++ `FlipCoin()` (default frequency 2).
    fn flip_coin(&mut self) -> bool {
        // FlipCoin() == FlipCoin(2) == (GenerateRnd(2) == 0)
        self.rng.generate(2) == 0
    }

    /// Flip coin with a 1-in-`n` chance (`GenerateRnd(n) == 0`).
    ///
    /// Mirrors C++ `FlipCoin(n)`.
    fn flip_coin_n(&mut self, n: usize) -> bool {
        self.rng.generate(n as i32) == 0
    }

    /// Copy predungeon to dungeon grid
    /// C++ equivalent: Inline copy in GenerateLevel
    fn copy_to_dungeon(&self, dungeon: &mut Dungeon) {
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                dungeon.tiles[i][j] = self.predungeon[i][j];
            }
        }
    }

    /// Calculate total floor area
    /// C++ equivalent: GetFloorArea
    fn get_floor_area(&self, dungeon: &Dungeon) -> i32 {
        let mut area = 0;
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                area += dungeon.tiles[i][j] as i32;
            }
        }
        area
    }

    /// Check dungeon connectivity (flood fill)
    /// C++ equivalent: Lockout + LockRectangle
    fn lockout(&mut self, dungeon: &mut Dungeon) -> bool {
        // Reset Protected array (used as visited mask)
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                dungeon.protected[i][j] = false;
            }
        }

        // Find first floor tile
        let mut fx = 0;
        let mut fy = 0;
        let mut total_tiles = 0;

        for j in 0..DMAXY {
            for i in 0..DMAXX {
                if dungeon.tiles[i][j] != 0 {
                    dungeon.protected[i][j] = true;
                    fx = i;
                    fy = j;
                    total_tiles += 1;
                }
            }
        }

        // Flood fill from first floor tile
        self.lockout_count = 0;
        self.lock_rectangle(dungeon, fx, fy);

        // Check if all floor tiles are reachable
        total_tiles == self.lockout_count
    }

    /// Recursive flood fill helper
    /// C++ equivalent: LockRectangle
    fn lock_rectangle(&mut self, dungeon: &mut Dungeon, x: usize, y: usize) {
        if x >= DMAXX || y >= DMAXY {
            return;
        }

        if !dungeon.protected[x][y] {
            return;
        }

        dungeon.protected[x][y] = false;
        self.lockout_count += 1;

        if y > 0 {
            self.lock_rectangle(dungeon, x, y - 1);
        }
        if y < DMAXY - 1 {
            self.lock_rectangle(dungeon, x, y + 1);
        }
        if x > 0 {
            self.lock_rectangle(dungeon, x - 1, y);
        }
        if x < DMAXX - 1 {
            self.lock_rectangle(dungeon, x + 1, y);
        }
    }

    /// Convert 2x2 predungeon patterns to final dungeon tiles
    /// C++ equivalent: MakeMegas
    fn make_megas(&mut self, dungeon: &mut Dungeon) {
        for j in 0..(DMAXY - 1) {
            for i in 0..(DMAXX - 1) {
                // Calculate 2x2 pattern value
                let mut v = (dungeon.tiles[i + 1][j + 1]
                    + 2 * dungeon.tiles[i][j + 1]
                    + 4 * dungeon.tiles[i + 1][j]
                    + 8 * dungeon.tiles[i][j]) as usize;

                // Handle diagonal patterns randomly
                if v == 6 {
                    v = if self.flip_coin() { 12 } else { 5 };
                }
                if v == 9 {
                    v = if self.flip_coin() { 13 } else { 14 };
                }

                // Apply conversion table
                dungeon.tiles[i][j] = L3_CONV_TABLE[v];
            }
            // Set right edge to wall
            dungeon.tiles[DMAXX - 1][j] = 8;
        }

        // Set bottom edge to wall
        for i in 0..DMAXX {
            dungeon.tiles[i][DMAXY - 1] = 8;
        }
    }

    /// Place stairs (up, down, and town warp)
    /// C++ equivalent: PlaceCaveStairs / PlaceNestStairs
    fn place_stairs(&mut self, dungeon: &mut Dungeon, level: u8, entry: LevelEntry) -> bool {
        // Determine if this is Hive/Nest level (15-16) or regular Caves (9-12)
        let is_hive = level == 15 || level == 16;

        // Place stairs up
        let stairs_up = if is_hive {
            miniset_l6_up()
        } else {
            miniset_l3_up()
        };

        if !self.try_place_miniset(dungeon, &stairs_up) {
            return false;
        }

        // Place stairs down (not on last level of Hive)
        if !is_hive || level != 16 {
            let stairs_down = if is_hive {
                miniset_l6_down()
            } else {
                miniset_l3_down()
            };

            if !self.try_place_miniset(dungeon, &stairs_down) {
                return false;
            }
        }

        // Place town warp on level 9
        if level == 9 && matches!(entry, LevelEntry::TownWarp) {
            let warp = miniset_l3_holdwarp();
            if !self.try_place_miniset(dungeon, &warp) {
                return false;
            }
        }

        true
    }

    /// Try to place a miniset at random valid location
    /// C++ equivalent: PlaceMiniSet (simplified)
    fn try_place_miniset(&mut self, dungeon: &mut Dungeon, miniset: &Miniset) -> bool {
        let sw = miniset.width;
        let sh = miniset.height;

        // Try random locations
        for _attempt in 0..100 {
            let sx = self.random_range(0, DMAXX - sw);
            let sy = self.random_range(0, DMAXY - sh);

            // Check if miniset matches
            if self.miniset_matches(dungeon, miniset, sx, sy) {
                // Place miniset
                self.place_miniset_at(dungeon, miniset, sx, sy);
                return true;
            }
        }

        false
    }

    /// Check if miniset matches at given position
    fn miniset_matches(&self, dungeon: &Dungeon, miniset: &Miniset, sx: usize, sy: usize) -> bool {
        for j in 0..miniset.height {
            for i in 0..miniset.width {
                let search_val = miniset.search[j][i];
                let dungeon_val = dungeon.tiles[sx + i][sy + j];

                // 0 in search pattern means "don't care"
                if search_val != 0 && search_val != dungeon_val {
                    return false;
                }
            }
        }
        true
    }

    /// Place miniset at given position
    fn place_miniset_at(&self, dungeon: &mut Dungeon, miniset: &Miniset, sx: usize, sy: usize) {
        for j in 0..miniset.height {
            for i in 0..miniset.width {
                let replace_val = miniset.replace[j][i];
                if replace_val != 0 {
                    dungeon.tiles[sx + i][sy + j] = replace_val;
                }
            }
        }
    }

    /// Generate river/lava system
    /// C++ equivalent: River
    fn river(&mut self, dungeon: &mut Dungeon) {
        let mut river_count = 0;
        let mut tries = 0;
        let mut pdir = -1i32; // Previous direction

        while tries < 200 && river_count < 4 {
            tries += 1;

            // Find starting point (tiles 25-28: directional edge tiles)
            let mut rx = 0usize;
            let mut ry = 0usize;
            let mut i = 0;

            // Find valid starting tile
            while i < 100 {
                rx = self.random_range(0, DMAXX);
                ry = self.random_range(0, DMAXY);
                i += 1;

                // Scan horizontally if needed
                while ry < DMAXY && (dungeon.tiles[rx][ry] < 25 || dungeon.tiles[rx][ry] > 28) {
                    rx += 1;
                    if rx >= DMAXX {
                        rx = 0;
                        ry += 1;
                    }
                }

                if ry < DMAXY && dungeon.tiles[rx][ry] >= 25 && dungeon.tiles[rx][ry] <= 28 {
                    break;
                }
            }

            if ry >= DMAXY || i >= 100 {
                continue;
            }

            // Initialize river path based on starting tile
            let mut river_x = [0usize; 100];
            let mut river_y = [0usize; 100];
            let mut river_tile = [0u8; 100];
            let mut riveramt = 0usize;

            let (mut dir, nodir) = match dungeon.tiles[rx][ry] {
                25 => (3, 2),
                26 => (0, 1),
                27 => (1, 0),
                28 => (2, 3),
                _ => continue,
            };

            // Set starting endpoint tile
            river_tile[0] = match dungeon.tiles[rx][ry] {
                25 => 40,
                26 => 38,
                27 => 41,
                28 => 39,
                _ => 0,
            };
            river_x[0] = rx;
            river_y[0] = ry;
            riveramt = 1;

            let mut nodir2 = 4; // Secondary blocked direction
            let mut dircheck = 0;

            // Trace river path
            while dircheck < 4 && riveramt < 100 {
                let px = rx;
                let py = ry;

                if dircheck == 0 {
                    dir = self.random_range(0, 4) as i32;
                } else {
                    dir = (dir + 1) & 3;
                }
                dircheck += 1;

                // Skip blocked directions
                while dir == nodir || dir == nodir2 {
                    dir = (dir + 1) & 3;
                    dircheck += 1;
                }

                // Move in direction
                match dir {
                    0 if ry > 0 => ry -= 1,
                    1 if ry < DMAXY - 1 => ry += 1,
                    2 if rx < DMAXX - 1 => rx += 1,
                    3 if rx > 0 => rx -= 1,
                    _ => {}
                }

                // Check if we hit floor tile
                if dungeon.tiles[rx][ry] == 7 {
                    dircheck = 0;

                    // Choose straight river tile based on direction
                    if dir < 2 {
                        river_tile[riveramt] = if self.flip_coin() { 17 } else { 18 };
                    } else {
                        river_tile[riveramt] = if self.flip_coin() { 15 } else { 16 };
                    }

                    river_x[riveramt] = rx;
                    river_y[riveramt] = ry;
                    riveramt += 1;

                    // Detect corners and update previous tile
                    if (dir == 0 && pdir == 2) || (dir == 3 && pdir == 1) {
                        if riveramt > 2 {
                            river_tile[riveramt - 2] = 22;
                        }
                        nodir2 = if dir == 0 { 1 } else { 2 };
                    }
                    if (dir == 0 && pdir == 3) || (dir == 2 && pdir == 1) {
                        if riveramt > 2 {
                            river_tile[riveramt - 2] = 21;
                        }
                        nodir2 = if dir == 0 { 1 } else { 3 };
                    }
                    if (dir == 1 && pdir == 2) || (dir == 3 && pdir == 0) {
                        if riveramt > 2 {
                            river_tile[riveramt - 2] = 20;
                        }
                        nodir2 = if dir == 1 { 0 } else { 2 };
                    }
                    if (dir == 1 && pdir == 3) || (dir == 2 && pdir == 0) {
                        if riveramt > 2 {
                            river_tile[riveramt - 2] = 19;
                        }
                        nodir2 = if dir == 1 { 0 } else { 3 };
                    }

                    pdir = dir;
                } else {
                    // Revert position if not floor
                    rx = px;
                    ry = py;
                }
            }

            // Check if river reached an endpoint
            let mut valid_endpoint = false;
            if dir == 0 && ry >= 2 && dungeon.tiles[rx][ry - 1] == 10 && dungeon.tiles[rx][ry - 2] == 8 {
                river_x[riveramt] = rx;
                river_y[riveramt] = ry - 1;
                river_tile[riveramt] = 24;
                riveramt += 1;
                valid_endpoint = true;
            }
            if dir == 1 && ry < DMAXY - 2 && dungeon.tiles[rx][ry + 1] == 10 && dungeon.tiles[rx][ry + 2] == 8 {
                river_x[riveramt] = rx;
                river_y[riveramt] = ry + 1;
                river_tile[riveramt] = 23;
                riveramt += 1;
                valid_endpoint = true;
            }
            if dir == 2 && rx < DMAXX - 2 && dungeon.tiles[rx + 1][ry] == 10 && dungeon.tiles[rx + 2][ry] == 8 {
                river_x[riveramt] = rx + 1;
                river_y[riveramt] = ry;
                river_tile[riveramt] = if self.flip_coin() { 42 } else { 43 };
                riveramt += 1;
                valid_endpoint = true;
            }
            if dir == 3 && rx >= 2 && dungeon.tiles[rx - 1][ry] == 10 && dungeon.tiles[rx - 2][ry] == 8 {
                river_x[riveramt] = rx - 1;
                river_y[riveramt] = ry;
                river_tile[riveramt] = if self.flip_coin() { 38 } else { 39 };
                riveramt += 1;
                valid_endpoint = true;
            }

            // Only place river if valid and long enough
            if valid_endpoint && riveramt >= 7 {
                // Place river tiles
                for k in 0..riveramt {
                    dungeon.tiles[river_x[k]][river_y[k]] = river_tile[k];
                }

                // Randomly place a bridge on one river segment
                if riveramt > 3 {
                    let bridge_idx = self.random_range(1, riveramt - 1);
                    let tile = river_tile[bridge_idx];
                    if (tile >= 15 && tile <= 18) || (tile >= 19 && tile <= 22) {
                        dungeon.tiles[river_x[bridge_idx]][river_y[bridge_idx]] = if tile <= 18 { 44 } else { 45 };
                    }
                }

                river_count += 1;
            }
        }
    }

    /// Fix pool edges (convert isolated ceiling tiles near lava to lava)
    /// C++ equivalent: PoolFix
    fn pool_fix(&self, dungeon: &mut Dungeon) {
        for j in 1..(DMAXY - 2) {
            for i in 1..(DMAXX - 2) {
                // Check if tile is default ceiling (tile 8)
                if dungeon.tiles[i][j] != 8 {
                    continue;
                }

                // Check 3x3 neighborhood for lava tiles (25-41)
                let mut has_lava = false;
                for dy in 0..3 {
                    for dx in 0..3 {
                        let nx = i + dx - 1;
                        let ny = j + dy - 1;
                        if nx < DMAXX && ny < DMAXY {
                            let tile = dungeon.tiles[nx][ny];
                            if tile >= 25 && tile <= 41 {
                                has_lava = true;
                                break;
                            }
                        }
                    }
                    if has_lava {
                        break;
                    }
                }

                // If adjacent to lava, convert to ground lava (tile 33)
                if has_lava {
                    dungeon.tiles[i][j] = 33;
                }
            }
        }
    }

    /// SpawnEdge helper for Spawn (checks boundaries and specific tiles)
    /// C++ equivalent: SpawnEdge
    fn spawn_edge(&self, dungeon: &mut Dungeon, x: i32, y: i32, totarea: &mut i32) -> bool {
        const SPAWN_TABLE: [u8; 15] = [0x00, 0x0A, 0x03, 0x05, 0x0C, 0x06, 0x09, 0x00, 0x00, 0x0C, 0x03, 0x06, 0x09, 0x0A, 0x05];

        if *totarea > 40 {
            return true;
        }
        if x < 0 || y < 0 || x >= DMAXX as i32 || y >= DMAXY as i32 {
            return true;
        }
        if (dungeon.tiles[x as usize][y as usize] & 0x80) != 0 {
            return false;
        }
        if dungeon.tiles[x as usize][y as usize] > 15 {
            return false;
        }

        let i = dungeon.tiles[x as usize][y as usize];
        dungeon.tiles[x as usize][y as usize] |= 0x80;
        *totarea += 1;

        let st = SPAWN_TABLE[i as usize];
        if (st & 0x80) != 0 && self.spawn_edge(dungeon, x, y - 1, totarea) {
            return true;
        }
        if (st & 0x40) != 0 && self.spawn_edge(dungeon, x, y + 1, totarea) {
            return true;
        }
        if (st & 0x20) != 0 && self.spawn_edge(dungeon, x + 1, y, totarea) {
            return true;
        }
        if (st & 0x10) != 0 && self.spawn_edge(dungeon, x - 1, y, totarea) {
            return true;
        }

        false
    }

    /// Spawn: flood fill for pool generation
    /// C++ equivalent: Spawn
    fn spawn(&self, dungeon: &mut Dungeon, x: i32, y: i32, totarea: &mut i32) -> bool {
        const SPAWN_TABLE: [u8; 15] = [0x00, 0x0A, 0x03, 0x05, 0x0C, 0x06, 0x09, 0x00, 0x00, 0x0C, 0x03, 0x06, 0x09, 0x0A, 0x05];

        if *totarea > 40 {
            return true;
        }
        if x < 0 || y < 0 || x >= DMAXX as i32 || y >= DMAXY as i32 {
            return true;
        }
        if (dungeon.tiles[x as usize][y as usize] & 0x80) != 0 {
            return false;
        }
        if dungeon.tiles[x as usize][y as usize] > 15 {
            return true;
        }

        let i = dungeon.tiles[x as usize][y as usize];
        dungeon.tiles[x as usize][y as usize] |= 0x80;
        *totarea += 1;

        if i != 8 {
            let st = SPAWN_TABLE[i as usize];
            if (st & 8) != 0 && self.spawn_edge(dungeon, x, y - 1, totarea) {
                return true;
            }
            if (st & 4) != 0 && self.spawn_edge(dungeon, x, y + 1, totarea) {
                return true;
            }
            if (st & 2) != 0 && self.spawn_edge(dungeon, x + 1, y, totarea) {
                return true;
            }
            if (st & 1) != 0 && self.spawn_edge(dungeon, x - 1, y, totarea) {
                return true;
            }
        } else {
            if self.spawn(dungeon, x + 1, y, totarea) {
                return true;
            }
            if self.spawn(dungeon, x - 1, y, totarea) {
                return true;
            }
            if self.spawn(dungeon, x, y + 1, totarea) {
                return true;
            }
            if self.spawn(dungeon, x, y - 1, totarea) {
                return true;
            }
        }

        false
    }

    /// Place lava pools (for L3 Caves)
    /// C++ equivalent: PlaceLavaPool
    fn place_lava_pool(&mut self, dungeon: &mut Dungeon) -> bool {
        const POOL_SUB: [u8; 15] = [0, 35, 26, 36, 25, 29, 34, 7, 33, 28, 27, 37, 32, 31, 30];

        let mut lava_pool_placed = false;

        for duny in 0..DMAXY {
            for dunx in 0..DMAXX {
                if dungeon.tiles[dunx][duny] != 8 {
                    continue;
                }

                dungeon.tiles[dunx][duny] |= 0x80;
                let mut totarea = 1;
                let mut found = true;

                if dunx + 1 < DMAXX {
                    found = self.spawn(dungeon, (dunx + 1) as i32, duny as i32, &mut totarea);
                }
                if dunx > 0 && !found {
                    found = self.spawn(dungeon, (dunx - 1) as i32, duny as i32, &mut totarea);
                } else {
                    found = true;
                }
                if duny + 1 < DMAXY && !found {
                    found = self.spawn(dungeon, dunx as i32, (duny + 1) as i32, &mut totarea);
                } else {
                    found = true;
                }
                if duny > 0 && !found {
                    found = self.spawn(dungeon, dunx as i32, (duny - 1) as i32, &mut totarea);
                } else {
                    found = true;
                }

                let place_pool = self.random_range(0, 100) < 25;

                for j in (duny.saturating_sub(totarea as usize))..(duny + totarea as usize).min(DMAXY) {
                    for i in (dunx.saturating_sub(totarea as usize))..(dunx + totarea as usize).min(DMAXX) {
                        if (dungeon.tiles[i][j] & 0x80) != 0 {
                            dungeon.tiles[i][j] &= !0x80;
                            if totarea > 4 && place_pool && !found {
                                let k = POOL_SUB[dungeon.tiles[i][j] as usize];
                                if k != 0 && k <= 37 {
                                    dungeon.tiles[i][j] = k;
                                }
                                lava_pool_placed = true;
                            }
                        }
                    }
                }
            }
        }

        lava_pool_placed
    }

    /// CanReplaceTile: Check if tile can be replaced (prevents overwriting special tiles)
    /// C++ equivalent: CanReplaceTile
    fn can_replace_tile(&self, dungeon: &Dungeon, replace: u8, tx: usize, ty: usize) -> bool {
        if replace < 84 || replace > 100 {
            return true;
        }

        // Check 3x3 neighborhood for existing special tiles (84-100)
        let check_tile = |x: i32, y: i32| -> bool {
            if x >= 0 && x < DMAXX as i32 && y >= 0 && y < DMAXY as i32 {
                let tile = dungeon.tiles[x as usize][y as usize];
                tile >= 84 && tile <= 100
            } else {
                false
            }
        };

        let tx = tx as i32;
        let ty = ty as i32;

        if check_tile(tx - 1, ty - 1) || check_tile(tx + 1, ty + 1)
            || check_tile(tx - 1, ty + 1) || check_tile(tx + 1, ty - 1)
            || check_tile(tx, ty - 1) || check_tile(tx, ty + 1)
            || check_tile(tx - 1, ty) || check_tile(tx + 1, ty)
        {
            return false;
        }

        true
    }

    /// PlaceMiniSetRandom with probability check
    /// C++ equivalent: PlaceMiniSetRandom
    fn place_miniset_random(&mut self, dungeon: &mut Dungeon, miniset: &Miniset, rndper: usize) -> bool {
        let sw = miniset.width;
        let sh = miniset.height;

        let mut placed = false;
        for sy in 0..(DMAXY - sh) {
            for sx in 0..(DMAXX - sw) {
                if !self.miniset_matches(dungeon, miniset, sx, sy) {
                    continue;
                }
                if !self.can_replace_tile(dungeon, miniset.replace[0][0], sx, sy) {
                    continue;
                }
                if self.random_range(0, 100) >= rndper {
                    continue;
                }
                self.place_miniset_at(dungeon, miniset, sx, sy);
                placed = true;
            }
        }

        placed
    }

    /// Place miniset 1x1 (single tile replacement)
    /// C++ equivalent: PlaceMiniSetRandom1x1
    fn place_miniset_random_1x1(&mut self, dungeon: &mut Dungeon, search: u8, replace: u8, rndper: usize) {
        let miniset = Miniset {
            width: 1,
            height: 1,
            search: vec![vec![search]],
            replace: vec![vec![replace]],
        };
        self.place_miniset_random(dungeon, &miniset, rndper);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l3_conv_table() {
        assert_eq!(L3_CONV_TABLE.len(), 16);
        assert_eq!(L3_CONV_TABLE[0], 8);
        assert_eq!(L3_CONV_TABLE[15], 7);
    }

    #[test]
    fn test_miniset_l3_up() {
        let miniset = miniset_l3_up();
        assert_eq!(miniset.width, 3);
        assert_eq!(miniset.height, 3);
        assert_eq!(miniset.search[0][0], 8);
        assert_eq!(miniset.replace[0][0], 51);
    }

    #[test]
    fn test_miniset_l3_down() {
        let miniset = miniset_l3_down();
        assert_eq!(miniset.width, 3);
        assert_eq!(miniset.height, 3);
        assert_eq!(miniset.replace[0][1], 47);
        assert_eq!(miniset.replace[1][1], 46);
    }

    #[test]
    fn test_miniset_l3_tite1() {
        let miniset = miniset_l3_tite1();
        assert_eq!(miniset.width, 4);
        assert_eq!(miniset.height, 4);
        assert_eq!(miniset.search[0][0], 7);
        assert_eq!(miniset.replace[1][1], 57);
    }

    #[test]
    fn test_miniset_l3_crev1() {
        let miniset = miniset_l3_crev1();
        assert_eq!(miniset.width, 2);
        assert_eq!(miniset.height, 2);
        assert_eq!(miniset.replace[1][1], 85);
    }

    #[test]
    fn test_miniset_l3_isle1() {
        let miniset = miniset_l3_isle1();
        assert_eq!(miniset.width, 3);
        assert_eq!(miniset.height, 3);
        assert_eq!(miniset.search[1][1], 7);
        assert_eq!(miniset.replace[1][1], 143);
    }

    #[test]
    fn test_caves_generator_creation() {
        let generator = CavesGenerator::new();
        assert_eq!(generator.rng.get_seed(), 0);
        assert_eq!(generator.lockout_count, 0);
    }

    #[test]
    fn test_random_range() {
        let mut generator = CavesGenerator::new();
        generator.rng.set_seed(12345);

        let val = generator.random_range(0, 10);
        assert!(val < 10);

        let val2 = generator.random_range(5, 15);
        assert!(val2 >= 5 && val2 < 15);
    }

    #[test]
    fn test_random_range_equal() {
        let mut generator = CavesGenerator::new();
        let val = generator.random_range(5, 5);
        assert_eq!(val, 5);
    }

    #[test]
    fn test_miniset_l6_isle1() {
        let miniset = miniset_l6_isle1();
        assert_eq!(miniset.width, 2);
        assert_eq!(miniset.height, 3);
        assert_eq!(miniset.search[0][0], 5);
        assert_eq!(miniset.replace[0][0], 7);
    }

    #[test]
    fn test_fill_room() {
        let mut generator = CavesGenerator::new();
        generator.rng.set_seed(12345);

        // Test valid room
        let result = generator.fill_room(5, 5, 10, 10);
        assert!(result);

        // Check interior is filled
        for j in 6..10 {
            for i in 6..10 {
                assert_eq!(generator.predungeon[i][j], 1);
            }
        }
    }

    #[test]
    fn test_fill_diagonals() {
        let mut generator = CavesGenerator::new();
        generator.rng.set_seed(12345);

        // Setup a diagonal pattern (value 6)
        generator.predungeon[5][5] = 0;
        generator.predungeon[6][5] = 1;
        generator.predungeon[5][6] = 1;
        generator.predungeon[6][6] = 0;

        generator.fill_diagonals();

        // One of the diagonals should be filled
        assert!(generator.predungeon[5][5] == 1 || generator.predungeon[6][6] == 1);
    }

    #[test]
    fn test_init_dungeon_flags() {
        let mut generator = CavesGenerator::new();
        generator.predungeon[5][5] = 1;
        generator.lockout_count = 10;

        generator.init_dungeon_flags();

        assert_eq!(generator.predungeon[5][5], 0);
        assert_eq!(generator.lockout_count, 0);
    }

    #[test]
    fn test_edges() {
        let mut generator = CavesGenerator::new();
        generator.predungeon[0][0] = 1;
        generator.predungeon[DMAXX-1][DMAXY-1] = 1;

        generator.edges();

        assert_eq!(generator.predungeon[0][0], 0);
        assert_eq!(generator.predungeon[DMAXX-1][DMAXY-1], 0);
    }

    #[test]
    fn test_get_floor_area() {
        let mut dungeon = Dungeon::new();
        dungeon.tiles[5][5] = 7;
        dungeon.tiles[6][6] = 7;
        dungeon.tiles[7][7] = 12;

        let generator = CavesGenerator::new();
        let area = generator.get_floor_area(&dungeon);

        assert_eq!(area, 7 + 7 + 12);
    }

    #[test]
    fn test_make_megas() {
        let mut generator = CavesGenerator::new();
        generator.rng.set_seed(42);
        let mut dungeon = Dungeon::new();

        // Setup 2x2 pattern (all 1s = value 15)
        dungeon.tiles[0][0] = 1;
        dungeon.tiles[1][0] = 1;
        dungeon.tiles[0][1] = 1;
        dungeon.tiles[1][1] = 1;

        generator.make_megas(&mut dungeon);

        // Pattern 15 should map to 7
        assert_eq!(dungeon.tiles[0][0], L3_CONV_TABLE[15]);
        assert_eq!(L3_CONV_TABLE[15], 7);
    }

    #[test]
    fn test_lockout_connected() {
        let mut generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Create simple connected area
        for i in 5..10 {
            dungeon.tiles[i][5] = 7;
        }

        let result = generator.lockout(&mut dungeon);
        assert!(result); // All tiles should be connected
    }

    #[test]
    fn test_lockout_disconnected() {
        let mut generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Create two separate islands
        dungeon.tiles[5][5] = 7;
        dungeon.tiles[10][10] = 7;

        let result = generator.lockout(&mut dungeon);
        assert!(!result); // Tiles are not connected
    }

    #[test]
    fn test_copy_to_dungeon() {
        let mut generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        generator.predungeon[5][5] = 1;
        generator.predungeon[6][6] = 7;

        generator.copy_to_dungeon(&mut dungeon);

        assert_eq!(dungeon.tiles[5][5], 1);
        assert_eq!(dungeon.tiles[6][6], 7);
    }

    #[test]
    fn test_miniset_matches() {
        let generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Setup dungeon pattern
        dungeon.tiles[5][5] = 8;
        dungeon.tiles[6][5] = 8;
        dungeon.tiles[5][6] = 10;
        dungeon.tiles[6][6] = 10;

        // Create matching miniset
        let miniset = Miniset::new(2, 2,
            vec![vec![8,8],vec![10,10]],
            vec![vec![1,2],vec![3,4]]);

        assert!(generator.miniset_matches(&dungeon, &miniset, 5, 5));
    }

    #[test]
    fn test_place_miniset_at() {
        let generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        let miniset = Miniset::new(2, 2,
            vec![vec![0,0],vec![0,0]],
            vec![vec![51,50],vec![48,49]]);

        generator.place_miniset_at(&mut dungeon, &miniset, 10, 10);

        assert_eq!(dungeon.tiles[10][10], 51);
        assert_eq!(dungeon.tiles[11][10], 50);
        assert_eq!(dungeon.tiles[10][11], 48);
        assert_eq!(dungeon.tiles[11][11], 49);
    }

    #[test]
    fn test_pool_fix() {
        let generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Setup: tile 8 (ceiling) adjacent to tile 25 (lava)
        dungeon.tiles[5][5] = 8;
        dungeon.tiles[6][5] = 25; // lava tile

        generator.pool_fix(&mut dungeon);

        // Should convert ceiling to ground lava (33)
        assert_eq!(dungeon.tiles[5][5], 33);
    }

    #[test]
    fn test_can_replace_tile() {
        let generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Tile < 84 should always be replaceable
        assert!(generator.can_replace_tile(&dungeon, 50, 5, 5));

        // Tile > 100 should always be replaceable
        assert!(generator.can_replace_tile(&dungeon, 110, 5, 5));

        // Tile 84-100 with no special neighbors should be replaceable
        dungeon.tiles[5][5] = 7; // normal floor
        assert!(generator.can_replace_tile(&dungeon, 90, 5, 5));

        // Tile 84-100 with special tile neighbor (84-100) should NOT be replaceable
        dungeon.tiles[6][5] = 95; // special tile
        assert!(!generator.can_replace_tile(&dungeon, 90, 5, 5));
    }

    #[test]
    fn test_place_miniset_random() {
        let mut generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Setup dungeon with matching pattern
        for i in 0..40 {
            for j in 0..40 {
                dungeon.tiles[i][j] = 7; // floor
            }
        }

        let miniset = Miniset::new(2, 2,
            vec![vec![7,7],vec![7,7]],
            vec![vec![51,50],vec![48,49]]);

        // With 100% probability, should place
        let placed = generator.place_miniset_random(&mut dungeon, &miniset, 100);
        assert!(placed);
    }

    #[test]
    fn test_spawn_edge() {
        let generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Setup simple spawn scenario
        dungeon.tiles[5][5] = 8; // ceiling
        dungeon.tiles[6][5] = 7; // floor

        let mut totarea = 0;
        let result = generator.spawn_edge(&mut dungeon, 5, 5, &mut totarea);

        // Should mark tile with 0x80
        assert!((dungeon.tiles[5][5] & 0x80) != 0);
        assert!(totarea > 0);
    }

    #[test]
    fn test_spawn() {
        let generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Setup spawn area
        for i in 3..8 {
            for j in 3..8 {
                dungeon.tiles[i][j] = 8; // ceiling
            }
        }

        let mut totarea = 0;
        let result = generator.spawn(&mut dungeon, 5, 5, &mut totarea);

        // Should have marked multiple tiles
        assert!(totarea > 0);
    }

    #[test]
    fn test_place_lava_pool() {
        let mut generator = CavesGenerator::new();
        let mut dungeon = Dungeon::new();

        // Setup small area with ceiling tiles
        for i in 5..10 {
            for j in 5..10 {
                dungeon.tiles[i][j] = 8; // ceiling
            }
        }

        // Surround with walls to make it disconnected
        for i in 4..11 {
            dungeon.tiles[i][4] = 10;
            dungeon.tiles[i][10] = 10;
        }
        for j in 4..11 {
            dungeon.tiles[4][j] = 10;
            dungeon.tiles[10][j] = 10;
        }

        // Try to place pool (result depends on random)
        let _result = generator.place_lava_pool(&mut dungeon);
        // Note: Since this uses RNG, we can't assert specific result
        // Just verify it runs without panic
    }

    /// Two generations with the same seed must produce identical dungeon tiles.
    ///
    /// This is the core determinism contract (save/replay compatibility): the
    /// Caves generator must be bit-for-bit reproducible for a given seed. It
    /// depends on the Borland LCG (`Rng`) being wired in correctly end-to-end
    /// via the `random_range`/`flip_coin`/`flip_coin_n` helpers.
    #[test]
    fn test_generate_is_deterministic_for_same_seed() {
        let seed = 12345u32;
        let level = 9u8;

        let mut a = CavesGenerator::new();
        let mut da = Dungeon::new();
        a.generate(&mut da, seed, level, LevelEntry::MainEntry);

        let mut b = CavesGenerator::new();
        let mut db = Dungeon::new();
        b.generate(&mut db, seed, level, LevelEntry::MainEntry);

        assert_eq!(da.tiles, db.tiles, "same seed must yield identical tiles");
    }

    /// The RNG helpers must be deterministic and seed-driven.
    ///
    /// Two generators seeded identically must draw the same `random_range` and
    /// `flip_coin` sequences, and the engine state must advance identically.
    /// Guards against the generator silently reverting to a non-deterministic
    /// RNG (the bug class the original custom LCG belonged to).
    #[test]
    fn test_rng_helpers_are_deterministic() {
        let mut a = CavesGenerator::new();
        let mut b = CavesGenerator::new();
        a.rng.set_seed(0xABCDEF01);
        b.rng.set_seed(0xABCDEF01);

        for _ in 0..100 {
            assert_eq!(a.random_range(0, 100), b.random_range(0, 100));
            assert_eq!(a.flip_coin(), b.flip_coin());
            assert_eq!(a.flip_coin_n(4), b.flip_coin_n(4));
        }
        assert_eq!(
            a.rng.get_seed(),
            b.rng.get_seed(),
            "identical draw sequences must leave the engine in the same state"
        );
    }

    /// Different seeds must drive the RNG into different end states.
    ///
    /// Guards against the generator accidentally ignoring its seed.
    #[test]
    fn test_different_seeds_drive_rng_differently() {
        let mut a = CavesGenerator::new();
        let mut da = Dungeon::new();
        a.generate(&mut da, 1, 9, LevelEntry::MainEntry);

        let mut b = CavesGenerator::new();
        let mut db = Dungeon::new();
        b.generate(&mut db, 2, 9, LevelEntry::MainEntry);

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
