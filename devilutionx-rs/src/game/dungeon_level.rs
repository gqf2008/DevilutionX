//! L1 Cathedral dungeon level: generation + dPiece grid construction.
//!
//! Bridges the faithful Cathedral generator (`levels::drlg_l1::CathedralGenerator`,
//! which produces a 40×40 logical `Tile` enum grid) with the rendering layer,
//! which needs a 112×112 `dPiece` grid of micro-tile indices.
//!
//! The conversion mirrors C++ `Source/levels/gendung.cpp::DRLG_LPass3`:
//!   1. Fill the whole MAXDUN×MAXDUN grid with a background mega-tile's four
//!      micro values (C++ uses `pMegaTiles[Dirt - 1]`; here we use the L1
//!      "dirt" tile id).
//!   2. For each of the 40×40 logical tiles, look up its L1 TIL mega-tile id
//!      and stamp the four micro values into the grid at offset
//!      `(16 + i*2, 16 + j*2)`.
//!
//! The logical `Tile` enum produced by drlg_l1 keeps the *exact* C++
//! discriminants (drlg_l1.cpp `enum Tile`), which are 1-based L1 TIL mega
//! indices. `DRLG_LPass3` therefore maps each logical tile straight into the
//! TIL table with `tileId = dungeon[i][j] - 1` — no remapping step is needed.
//! (`tile_to_l1_til_index` remains only as a legacy reference for the enum's
//! semantic groups.)

use crate::engine::dungeon::DungeonLevelData;
use crate::game::game_state::DungeonLayout;
use crate::levels::drlg_l1::{CathedralGenerator, Tile};
use crate::levels::gendung::Dungeon;
use crate::levels::types::{DMAXX, DMAXY, MAXDUNX, MAXDUNY};

/// The L1 TIL index used as the "dirt background" for `DRLG_LPass3`'s first
/// pass. C++ `Pass3()` calls `DRLG_LPass3(Dirt - 1)` with `Dirt = 22`
/// (drlg_l1.cpp enum), so the background mega is TIL entry 21 (0-based).
/// Its four micros are all solid dirt (verified against l1.til/l1.sol).
const L1_BG_TIL_INDEX: usize = 21;

/// Generate an L1 Cathedral dungeon with the given seed and produce a
/// render-ready `DungeonLayout` (dPiece grid) by mapping the generator's
/// logical `Tile` grid through `level.til` mega definitions.
///
/// Returns the built layout (always non-empty: the 40×40 logical grid is
/// populated and the background fill guarantees micro values everywhere).
pub fn generate_l1_cathedral(seed: u32, level: &DungeonLevelData) -> DungeonLayout {
    // 1. Run the faithful Cathedral generator.
    let mut gen = CathedralGenerator::new();
    gen.generate(crate::levels::types::DungeonType::Cathedral, seed);

    // 2. Build the 112×112 dPiece grid via DRLG_LPass3-style stamping.
    build_dungeon_layout(&gen, level)
}


/// The background mega index for each dungeon's `DRLG_LPass3` first pass
/// (C++ `Pass3()` per level): L1 = 8-1, L2 = 12-1, L3 = 8-1, L4 = 30-1.
pub const DUNGEON_BG_TIL_INDEX: [usize; 5] = [0, 7, 11, 7, 29];

/// Stamp a render-ready `DungeonLayout` from a generated `Dungeon` whose
/// `tiles` hold 1-based TIL tile ids, mirroring C++ `DRLG_LPass3(bgIndex)`:
/// the background mega fills the whole MAXDUN×MAXDUN grid, then each logical
/// tile maps straight to its mega (`tile_id - 1`). No logical-enum remapping
/// is needed for L2-L4 (unlike L1's Cathedral generator).
pub fn stamp_dungeon_layout(
    dungeon: &Dungeon,
    level: &DungeonLevelData,
    bg_til_index: usize,
) -> DungeonLayout {
    let mut layout = DungeonLayout::default();

    // Background fill (C++ DRLG_LPass3 first pass).
    if let Some((m1, m2, m3, m4)) = mega_for_til_index(level, bg_til_index) {
        let mut j = 0;
        while j + 1 < MAXDUNY {
            let mut i = 0;
            while i + 1 < MAXDUNX {
                layout.d_piece[j * MAXDUNX + i] = m1;
                layout.d_piece[j * MAXDUNX + i + 1] = m2;
                layout.d_piece[(j + 1) * MAXDUNX + i] = m3;
                layout.d_piece[(j + 1) * MAXDUNX + i + 1] = m4;
                i += 2;
            }
            j += 2;
        }
    }

    // Stamp the 40×40 logical grid at offset (16, 16) (C++ DRLG_LPass3).
    let mut yy = 16usize;
    for j in 0..DMAXY {
        let mut xx = 16usize;
        for i in 0..DMAXX {
            let tile_id = dungeon.tiles[i][j] as usize;
            if tile_id > 0 {
                if let Some((m1, m2, m3, m4)) = mega_for_til_index(level, tile_id - 1) {
                    let b = yy * MAXDUNX + xx;
                    if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                        layout.d_piece[b] = m1;
                        layout.d_piece[b + 1] = m2;
                        layout.d_piece[b + MAXDUNX] = m3;
                        layout.d_piece[b + MAXDUNX + 1] = m4;
                    }
                }
            }
            xx += 2;
        }
        yy += 2;
    }

    // Copy the generator's dTransVal (the L2 generator already runs
    // fix_transparency; FloodTransparencyValues remains a follow-up).
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            layout.trans_val[y * MAXDUNX + x] = dungeon.trans_val[x][y];
        }
    }

    // Bake per-level static lights into dPreLight (C++ calls DoLighting during
    // level generation, then SavePreLighting() snapshots dLight -> dPreLight).
    apply_static_lights(level, &mut layout);

    // Keep the level's SOL table with the layout (C++ `SOLData`).
    layout.sol = level.sol.properties.clone();

    layout
}

/// Bake per-level static lights into `dPreLight`, mirroring the C++ engine's
/// `PlaceCaveLights()` (drlg_l3.cpp:2150) and the object lights placed by
/// `AddL1Objs` (objects.cpp:3756) + `AddObjectLight` (objects.cpp:1247).
///
/// * Caves: every lava dPiece tile (MIN mega index ranges 55-146, 153-160 and
///   149/151) casts a radius-7 light.
/// * Cathedral: dPiece 269 (lava) gets `OBJ_L1LIGHT` in C++, a radius-5 light.
///
/// L2/L4 static lights come from placed torches/braziers (object placement),
/// which the Rust object subsystem does not place during generation yet.
fn apply_static_lights(level: &DungeonLevelData, layout: &mut DungeonLayout) {
    let is_lava = |piece: u16| -> bool {
        (55..=146).contains(&piece) || (153..=160).contains(&piece) || piece == 149 || piece == 151
    };
    let mut lm = crate::engine::lighting::LightManager::new();
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            let piece = layout.d_piece[y * MAXDUNX + x];
            match level.dungeon_type {
                crate::engine::dungeon::DungeonType::Caves if is_lava(piece) => {
                    lm.do_lighting(
                        &mut layout.pre_light,
                        MAXDUNX,
                        crate::engine::lighting::Point::new(x as i32, y as i32),
                        7,
                    );
                }
                crate::engine::dungeon::DungeonType::Cathedral if piece == 269 => {
                    // C++ AddL1Objs → OBJ_L1LIGHT → AddObjectLight radius 5.
                    lm.do_lighting(
                        &mut layout.pre_light,
                        MAXDUNX,
                        crate::engine::lighting::Point::new(x as i32, y as i32),
                        5,
                    );
                }
                _ => {}
            }
        }
    }
}

/// C++ `AddL2Torches` (objects.cpp:452-478) + `AddObjectLight` (objects.cpp:1247):
/// scan the L2 dPiece grid; at specific wall/floor micro values roll `FlipCoin`
/// and place a torch, then bake a radius-8 static light into dPreLight.
///
/// The RNG stream mirrors C++ `InitObjects`: the gameplay RNG is re-seeded to
/// `DungeonSeeds[currlevel]` (`SetRndSeedForDungeonLevel`) and InitObjects
/// discards one value (objects.cpp:3823) before object placement. The Rust
/// simplified level numbering uses `dungeon_seeds[level]` (level 2) in place of
/// C++ `DungeonSeeds[5..8]`; quest-gated placements (e.g. the L2-4 storybook)
/// that consume RNG before the torches are not yet replicated.
fn add_l2_torch_lights(layout: &mut DungeonLayout, seed: u32) {
    use crate::engine::lighting::Point;
    use crate::engine::random::DiabloGenerator;
    let mut rng = DiabloGenerator::new(seed);
    rng.discard(1); // C++ InitObjects DiscardRandomValues(1)
    let mut lm = crate::engine::lighting::LightManager::new();
    // Objects placed so far this pass (C++ dObject / IsObjectAtPosition).
    let mut placed: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            let pn = layout.d_piece[y * MAXDUNX + x];
            let (px, py) = (x as i32, y as i32);
            if pn == 0 && rng.flip_coin(3) {
                // OBJ_TORCHL2 at the floor tile itself.
                placed.insert((px, py));
                lm.do_lighting(&mut layout.pre_light, MAXDUNX, Point::new(px, py), 8);
            }
            if pn == 4 && rng.flip_coin(3) {
                // OBJ_TORCHR2 at the tile itself.
                placed.insert((px, py));
                lm.do_lighting(&mut layout.pre_light, MAXDUNX, Point::new(px, py), 8);
            }
            if pn == 36 && rng.flip_coin(10) && !placed.contains(&(px - 1, py)) {
                // OBJ_TORCHL at Direction::NorthWest = (-1, 0).
                placed.insert((px - 1, py));
                lm.do_lighting(&mut layout.pre_light, MAXDUNX, Point::new(px - 1, py), 8);
            }
            if pn == 40 && rng.flip_coin(10) && !placed.contains(&(px, py - 1)) {
                // OBJ_TORCHR at Direction::NorthEast = (0, -1).
                placed.insert((px, py - 1));
                lm.do_lighting(&mut layout.pre_light, MAXDUNX, Point::new(px, py - 1), 8);
            }
        }
    }
}

/// Scan a generated layout for door and light objects, mirroring C++
/// `AddL1Objs`, `AddL2Objs` and `AddL3Objs` (objects.cpp:3756-3795). Those
/// functions place objects from pure dPiece micro-value scans (no RNG), so the
/// scan here is deterministic and byte-faithful given a C++-aligned d_piece
/// grid:
///
/// * Cathedral (L1): 269 -> OBJ_L1LIGHT, 43/50/213 -> OBJ_L1LDOOR,
///   45/55 -> OBJ_L1RDOOR
/// * Catacombs (L2): 12/540 -> OBJ_L2LDOOR, 16/541 -> OBJ_L2RDOOR
/// * Caves (L3): 530 -> OBJ_L3LDOOR, 533 -> OBJ_L3RDOOR
///
/// Returns `(x, y, object_type)` in C++ scan order (row-major y then x, light
/// before doors within a cell, matching `AddL1Objs`).
pub fn scan_level_doors(level: u8, layout: &DungeonLayout) -> Vec<(i32, i32, crate::game::objdat::ObjectId)> {
    use crate::game::objdat::ObjectId;
    let mut objects = Vec::new();
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            let pn = layout.d_piece[y * MAXDUNX + x];
            let otype = match level {
                1 => match pn {
                    269 => Some(ObjectId::L1Light),
                    43 | 50 | 213 => Some(ObjectId::L1LDoor),
                    45 | 55 => Some(ObjectId::L1RDoor),
                    _ => None,
                },
                2 => match pn {
                    12 | 540 => Some(ObjectId::L2LDoor),
                    16 | 541 => Some(ObjectId::L2RDoor),
                    _ => None,
                },
                3 => match pn {
                    530 => Some(ObjectId::L3LDoor),
                    533 => Some(ObjectId::L3RDoor),
                    _ => None,
                },
                _ => None,
            };
            if let Some(otype) = otype {
                objects.push((x as i32, y as i32, otype));
            }
        }
    }
    objects
}

/// Build a render-ready `DungeonLayout` for an L2 Catacombs level
/// (C++ `drlg_l2.cpp Pass3()` → `DRLG_LPass3(11)`).
pub fn build_catacombs_layout(dungeon: &Dungeon, level: &DungeonLevelData) -> DungeonLayout {
    stamp_dungeon_layout(dungeon, level, DUNGEON_BG_TIL_INDEX[2])
}

/// Build a render-ready `DungeonLayout` for an L3 Caves level
/// (C++ `drlg_l3.cpp Pass3()` → `DRLG_LPass3(7)`).
pub fn build_caves_layout(dungeon: &Dungeon, level: &DungeonLevelData) -> DungeonLayout {
    stamp_dungeon_layout(dungeon, level, DUNGEON_BG_TIL_INDEX[3])
}

/// Build a render-ready `DungeonLayout` for an L4 Hell level
/// (C++ `drlg_l4.cpp Pass3()` → `DRLG_LPass3(29)`).
pub fn build_hell_layout(dungeon: &Dungeon, level: &DungeonLevelData) -> DungeonLayout {
    stamp_dungeon_layout(dungeon, level, DUNGEON_BG_TIL_INDEX[4])
}


/// Generate a render-ready layout for any dungeon level, dispatching to the
/// faithful per-level generator (mirrors C++ `CreateL1..L4Dungeon` dispatch in
/// `drlg_l1..l4.cpp`).
///
/// Level numbering is the simple 1=L1 Cathedral, 2=L2 Catacombs, 3=L3 Caves,
/// 4=L4 Hell. `seed` drives the Diablo LCG so layouts are reproducible.
/// Generate a dungeon layout headlessly (no MPQ art) for the replay/save
/// pipeline. The logical 40x40 grid, floor tiles and trans_val come from
/// the generator (C++-exact); the dPiece micro grid uses a synthetic
/// identity TIL since no l*.til art is loaded, and dPreLight stays 0
/// (fully lit, matching L1). Only L1 (Cathedral) is supported.
pub fn generate_level_headless(level: u8, seed: u32) -> Result<DungeonLayout, String> {
    use crate::engine::dungeon::{DungeonLevelData, DungeonType, TilEntry};
    use crate::levels::drlg_l1::CathedralGenerator;
    if level != 1 {
        return Err(format!("headless level generation only supports L1, got {level}"));
    }
    let mut art = DungeonLevelData::new(DungeonType::Cathedral);
    // Synthetic identity TIL: entry i -> micros (i*4, i*4+1, i*4+2, i*4+3)
    // so build_dungeon_layout can stamp a deterministic dPiece grid. Door
    // detection (scan_level_doors) needs the real L1 micros, so doors are
    // only detected when real art is loaded.
    art.til.tiles = (0..256u16)
        .map(|i| TilEntry {
            micro1: i * 4,
            micro2: i * 4 + 1,
            micro3: i * 4 + 2,
            micro4: i * 4 + 3,
        })
        .collect();
    let mut gen = CathedralGenerator::new();
    gen.generate(crate::levels::types::DungeonType::Cathedral, seed);
    Ok(build_dungeon_layout(&gen, &art))
}

pub fn generate_dungeon_layout(
    level: u8,
    seed: u32,
    art: &DungeonLevelData,
) -> Result<DungeonLayout, String> {
    use crate::levels::drlg_l1::CathedralGenerator;
    use crate::levels::drlg_l2::CatacombsGenerator;
    use crate::levels::drlg_l3::CavesGenerator;
    use crate::levels::drlg_l4::Dungeon4Generator;
    use crate::levels::types::{DungeonType as LvType, LevelEntry};

    match level {
        1 => {
            let mut gen = CathedralGenerator::new();
            gen.generate(LvType::Cathedral, seed);
            Ok(build_dungeon_layout(&gen, art))
        }
        2 => {
            let mut dungeon = Dungeon::new();
            dungeon.level_type = LvType::Catacombs;
            let mut gen = CatacombsGenerator::new();
            gen.generate(&mut dungeon, seed, 5)
                .then_some(())
                .ok_or_else(|| "L2 Catacombs generation failed".to_string())?;
            let mut layout = build_catacombs_layout(&dungeon, art);
            // C++ InitObjects (objects.cpp:3854-3860): AddL2Objs then AddL2Torches.
            // The torch scan bakes static lights into dPreLight via AddObjectLight.
            add_l2_torch_lights(&mut layout, seed);
            Ok(layout)
        }
        3 => {
            let mut dungeon = Dungeon::new();
            dungeon.level_type = LvType::Caves;
            let mut gen = CavesGenerator::new();
            gen.generate(&mut dungeon, seed, 9, LevelEntry::Main)
                .then_some(())
                .ok_or_else(|| "L3 Caves generation failed".to_string())?;
            Ok(build_caves_layout(&dungeon, art))
        }
        4 => {
            let mut dungeon = Dungeon::new();
            dungeon.level_type = LvType::Hell;
            let mut gen = Dungeon4Generator::new();
            gen.generate(&mut dungeon, seed, 13, LevelEntry::Main)
                .then_some(())
                .ok_or_else(|| "L4 Hell generation failed".to_string())?;
            Ok(build_hell_layout(&dungeon, art))
        }
        other => Err(format!("unsupported dungeon level: {other}")),
    }
}

/// Build the dPiece grid from a generated Cathedral grid + the L1 TIL data.
///
/// This is the dungeon-mode analogue of `build_town_layout`: it mirrors C++
/// `DRLG_LPass3`.
pub fn build_dungeon_layout(gen: &CathedralGenerator, level: &DungeonLevelData) -> DungeonLayout {
    let mut layout = DungeonLayout::default();

    // Background fill: stamp the background mega's four micro values into every
    // 2×2 block across MAXDUN×MAXDUN (C++: `dPiece[i][j] = v1..v4` for the
    // whole grid using pMegaTiles[bg]).
    if let Some((m1, m2, m3, m4)) = mega_for_til_index(level, L1_BG_TIL_INDEX) {
        let mut j = 0;
        while j + 1 < MAXDUNY {
            let mut i = 0;
            while i + 1 < MAXDUNX {
                layout.d_piece[j * MAXDUNX + i] = m1;
                layout.d_piece[j * MAXDUNX + i + 1] = m2;
                layout.d_piece[(j + 1) * MAXDUNX + i] = m3;
                layout.d_piece[(j + 1) * MAXDUNX + i + 1] = m4;
                i += 2;
            }
            j += 2;
        }
    }

    // Stamp the 40×40 logical grid at offset (16, 16), expanding each tile to a
    // 2×2 block of micro values from its L1 TIL mega (C++: `xx = 16; yy = 16`,
    // `dPiece[xx][yy] = mega.micro1..4`, `xx += 2` / `yy += 2`).
    let mut yy = 16usize;
    for j in 0..DMAXY {
        let mut xx = 16usize;
        for i in 0..DMAXX {
            // C++ DRLG_LPass3: `tileId = dungeon[i][j] - 1` — the Cathedral
            // Tile enum discriminants *are* 1-based L1 TIL mega indices, so
            // the logical tile maps straight into pMegaTiles (no remapping).
            // The generator stores the grid transposed (dungeon[row][col] here,
            // dungeon[col][row] in C++), so read the transposed index.
            let tile = gen.dungeon[j][i];
            let tile_id = tile as u8 as usize;
            if let Some((m1, m2, m3, m4)) = tile_id.checked_sub(1).and_then(|idx| mega_for_til_index(level, idx)) {
                let b = yy * MAXDUNX + xx;
                if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                    layout.d_piece[b] = m1;
                    layout.d_piece[b + 1] = m2;
                    layout.d_piece[b + MAXDUNX] = m3;
                    layout.d_piece[b + MAXDUNX + 1] = m4;
                }
            }

            // Collect the 2×2 micro-tile footprint of every *floor* logical tile
            // as a walkable spawn candidate (used by monster placement).
            if tile.is_floor_like() {
                if xx + 1 < MAXDUNX && yy + 1 < MAXDUNY {
                    layout.floor_tiles.push((xx as i32, yy as i32));
                    layout.floor_tiles.push(((xx + 1) as i32, yy as i32));
                    layout.floor_tiles.push((xx as i32, (yy + 1) as i32));
                    layout.floor_tiles.push(((xx + 1) as i32, (yy + 1) as i32));
                }
            }
            xx += 2;
        }
        yy += 2;
    }

    // 3. Compute dTransVal (C++ `FloodTransparencyValues(13)`) so the renderer
    //    can make doors/arches see-through. The generator's 40x40 logical Tile
    //    grid is read with the same [x][y] indexing as the dPiece stamping
    //    above; Tile::Floor == 13 matches the C++ floor id.
    let mut tiles = [[0u8; DMAXY]; DMAXX];
    let flood_src = gen.pre_variation_dungeon.as_ref().unwrap_or(&gen.dungeon);
    // The generator stores the grid transposed vs C++ (`dungeon[row][col]`
    // here, `dungeon[col][row]` in C++), so build the flood tiles in the C++
    // orientation: tiles[col][row] = grid[row][col].
    for y in 0..DMAXY {
        for x in 0..DMAXX {
            tiles[x][y] = flood_src[y][x] as u8;
        }
    }
    let mut trans_val = [[0i8; MAXDUNY]; MAXDUNX];
    crate::levels::gendung::flood_transparency_values(&tiles, Tile::Floor as u8, &mut trans_val);
    // C++ GenerateLevel (drlg_l1.cpp): after the retry loop, copy the TransVal
    // below each EntranceStairs tile into the stairs row, then run
    // FixTransparency() to spread the region value over Dirt wall footprints.
    // Both use the post-stairs, pre-variation grid (C++ runs them before
    // Substitution/FillFloor).
    let fix_src = gen.post_stairs_dungeon.as_ref().unwrap_or(&gen.dungeon);
    crate::levels::drlg_l1::copy_stairs_transparency(fix_src, &mut trans_val);
    crate::levels::drlg_l1::fix_transparency(fix_src, &mut trans_val);
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            layout.trans_val[y * MAXDUNX + x] = trans_val[x][y];
        }
    }

    // Keep the level's SOL table with the layout so monster placement and
    // tile queries can use C++ `IsTileSolid` (SOLData[dPiece]) exactly.
    let mut sol = level.sol.properties.clone();
    // C++ `LoadLevelSOLData` (gendung.cpp:444-472) applies per-level SOL
    // corrections after loading l1.sol; without them vision/light queries
    // diverge (arched/wall tiles miss their BlockLight|BlockMissile bits).
    {
        use crate::engine::dungeon::TileProperties;
        for idx in [9usize, 15, 16, 20, 21, 51, 56, 58, 61, 63, 65, 72, 208, 247, 253, 257, 323, 450] {
            if let Some(p) = sol.get_mut(idx) {
                *p |= TileProperties::BLOCK_LIGHT | TileProperties::BLOCK_MISSILE;
            }
        }
        for idx in [27usize, 28] {
            if let Some(p) = sol.get_mut(idx) {
                *p |= TileProperties::BLOCK_MISSILE;
            }
        }
        if let Some(p) = sol.get_mut(403) {
            *p |= TileProperties::BLOCK_LIGHT;
        }
        if let Some(p) = sol.get_mut(24) {
            *p |= TileProperties::BLOCK_LIGHT;
        }
    }
    layout.sol = sol;

    layout
}

/// Look up the four micro values (dPiece indices) for a 0-based TIL mega index
/// from the loaded L1 TIL data. Returns None if the index is out of range.
fn mega_for_til_index(level: &DungeonLevelData, til_idx: usize) -> Option<(u16, u16, u16, u16)> {
    let entry = level.til.tiles.get(til_idx)?;
    // TIL micro fields are 1-based MIN mega indices in the file. The renderer
    // indexes into `level.min.mega_tiles[dPiece]`, so the dPiece values we store
    // are these MIN mega indices directly (matching how town.til feeds
    // build_town_layout).
    Some((entry.micro1, entry.micro2, entry.micro3, entry.micro4))
}

/// Map a logical Cathedral `Tile` (the algorithmic enum from drlg_l1) to a
/// 0-based L1 TIL mega index.
///
/// The C++ Cathedral generator writes raw TIL tile ids into `dungeon[][]`
/// directly (it works in the engine's own tile-set vocabulary). The Rust port
/// instead produces a logical enum, so we re-map here. The mapping is curated
/// to pick visually-appropriate L1 megas:
///   - Floor / floor variants → L1 floor megas
///   - VWalls / HWalls / corners → L1 wall megas
///   - Dirt → L1 dirt (background-ish) mega
///   - Arches / doors → L1 arch/door megas
///   - Entrance stairs → L1 stairs-down mega
///
/// All indices are kept small (< ~12) so they stay within the L1 TIL table
/// even in shareware builds; out-of-range lookups safely fall back.
#[allow(dead_code)]
fn tile_to_l1_til_index(tile: Tile) -> usize {
    match tile {
        // Floors
        Tile::Floor | Tile::Floor22 | Tile::Floor23 => 0,

        // Walls (use a couple of variants so long walls aren't a single flat strip)
        Tile::VWall => 1,
        Tile::HWall => 2,

        // Corners
        Tile::NWCorner => 3,
        Tile::Corner => 3,
        Tile::HCorner => 3,
        Tile::NECorner => 4,
        Tile::SWCorner => 5,
        Tile::SECorner => 6,

        // Wall ends (C++ VWallEnd/HWallEnd)
        Tile::VWallEnd => 1,
        Tile::HWallEnd => 2,

        // Dirt / background
        Tile::Dirt => 7,

        // Doors
        Tile::VWallDoor => 8,
        Tile::HWallDoor => 9,

        // Arches
        Tile::ArchH1 | Tile::ArchH2 => 10,
        Tile::ArchV1 | Tile::ArchV2 => 11,

        // Dirt wall variants — reuse the dirt mega
        Tile::DirtVWallToSouth
        | Tile::DirtVWallToNorth
        | Tile::DirtHWallToEast
        | Tile::DirtHWallToWest
        | Tile::DirtNWCorner
        | Tile::DirtNECorner
        | Tile::DirtSWCorner
        | Tile::DirtSECorner
        | Tile::DirtCross
        | Tile::DirtHWall
        | Tile::DirtVWall
        // C++ dirt-wall tiles produced by FixTilesPatterns
        | Tile::DirtHwall
        | Tile::DirtVwall
        | Tile::VDirtCorner
        | Tile::HDirtCorner
        | Tile::DirtHwallEnd
        | Tile::DirtVwallEnd => 7,

        // Stairs down (entrance) — Cathedral uses a dedicated stairs mega
        Tile::EntranceStairs => 12,

        // Everything else (Lava, crypt tiles, Invalid) → floor
        _ => 0,
    }
}

/// Count non-zero dPiece cells in a layout (diagnostic / logging helper).
pub fn count_filled(layout: &DungeonLayout) -> usize {
    layout.d_piece.iter().filter(|&&v| v != 0).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_l1_cathedral_produces_nonempty_grid() {
        // Build a synthetic L1 level: a TIL with enough entries for our mapping.
        use crate::engine::dungeon::{
            DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry,
        };
        let mut til_tiles = Vec::new();
        for i in 0..16u16 {
            til_tiles.push(TilEntry {
                micro1: i,
                micro2: i + 1,
                micro3: i + 2,
                micro4: i + 3,
            });
        }
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Cathedral,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData {
                mega_tiles: vec![],
                blocks_per_tile: 10,
            },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        let layout = generate_l1_cathedral(12345, &level);
        assert_eq!(layout.width, MAXDUNX);
        assert_eq!(layout.height, MAXDUNY);
        assert_eq!(layout.d_piece.len(), MAXDUNX * MAXDUNY);
        // The background fill guarantees non-zero cells everywhere.
        assert!(count_filled(&layout) > 0);
        // The Cathedral generator always carves at least one room, so we should
        // collect a non-empty set of walkable floor micro-tiles (used by monster
        // spawning).
        assert!(
            !layout.floor_tiles.is_empty(),
            "expected some floor tiles for monster spawning"
        );
        // All collected floor tiles must lie inside the active dungeon region
        // (micro offset 16..96) and be unique.
        for &(x, y) in &layout.floor_tiles {
            assert!(x >= 16 && x < 96, "floor tile x {} out of active region", x);
            assert!(y >= 16 && y < 96, "floor tile y {} out of active region", y);
        }
    }

    #[test]
    fn test_tile_to_l1_til_index_covers_all_variants() {
        // Ensure every logical Tile maps to a sane index (no panic).
        for value in 1..=40u8 {
            if let Ok(tile) = Tile::try_from(value) {
                let idx = tile_to_l1_til_index(tile);
                assert!(idx < 16, "tile {:?} mapped to implausibly large index {}", tile, idx);
            }
        }
    }

    /// C++-exact dPiece alignment for the Timedemo L1 seed (1545811660):
    /// with the real l1.til/l1.sol from spawn.mpq, `build_dungeon_layout`
    /// must reproduce the door/light object scan (AddL1Objs) and the
    /// `InitMonsters` non-solid count (`na` = 2462 -> 82 monsters), both of
    /// which the old hand-written tile mapping got wrong (0 objects, 49).
    #[test]
    fn test_l1_dpiece_matches_cpp_timedemo_seed() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType};
        use crate::engine::mpq::MpqArchive;
        use crate::levels::drlg_l1::CathedralGenerator;
        use crate::levels::types::DungeonType as LvType;

        let mut archive = match MpqArchive::open("spawn.mpq") {
            Ok(a) => a,
            Err(_) => return, // skip without shareware assets
        };
        let art = match DungeonLevelData::load_from_mpq(&mut archive, DungeonType::Cathedral) {
            Ok(a) => a,
            Err(_) => return,
        };
        let mut gen = CathedralGenerator::new();
        gen.generate(LvType::Cathedral, 1545811660);
        let layout = build_dungeon_layout(&gen, &art);

        // AddL1Objs scan (aligned with the C++-exact L1 generator): 8 door
        // tiles + 7 lava lights.
        let objects = scan_level_doors(1, &layout);
        let doors = objects.iter().filter(|(_, _, t)| {
            matches!(*t, crate::game::objdat::ObjectId::L1LDoor | crate::game::objdat::ObjectId::L1RDoor)
        }).count();
        let lights = objects.iter().filter(|(_, _, t)| matches!(*t, crate::game::objdat::ObjectId::L1Light)).count();
        assert_eq!(doors, 8, "L1 door micro scan (dPiece 43/50/213/45/55)");
        assert_eq!(lights, 7, "L1 light micro scan (dPiece 269)");

        // InitMonsters na = non-solid micros in 16..96 via SOLData[dPiece].
        let na = (16usize..96).flat_map(|t| (16usize..96).map(move |s| (s, t)))
            .filter(|&(s, t)| {
                let pn = layout.d_piece[t * layout.width + s] as usize;
                !layout.sol.get(pn).copied().unwrap_or_default().contains(
                    crate::engine::dungeon::TileProperties::SOLID)
            }).count();
        assert_eq!(na, 2868, "InitMonsters non-solid count");
        assert_eq!(na / 30, 95, "numplacemonsters = na/30");
    }

    #[test]
    fn test_dungeon_layout_get_bounds() {
        let mut layout = DungeonLayout::default();
        layout.d_piece[0] = 5;
        assert_eq!(layout.get(0, 0), 5);
        assert_eq!(layout.get(-1, 0), 0);
        assert_eq!(layout.get(MAXDUNX as i32, 0), 0);
    }
    #[test]
    fn test_generate_l1_populates_trans_val() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        let mut til_tiles = Vec::new();
        for i in 0..16u16 {
            til_tiles.push(TilEntry { micro1: i, micro2: i + 1, micro3: i + 2, micro4: i + 3 });
        }
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Cathedral,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };
        let layout = generate_l1_cathedral(12345, &level);

        // FloodTransparencyValues(13) assigns a TransVal to every connected
        // floor region, so a Cathedral with at least one room must contain
        // non-zero trans_val entries (the floor + surrounding walls).
        let nonzero = layout.trans_val.iter().filter(|&&v| v != 0).count();
        assert!(nonzero > 0, "expected flood-filled trans_val entries, got {}", nonzero);
        // The max TransVal equals the number of distinct floor regions.
        let max_val = layout.trans_val.iter().max().copied().unwrap_or(0);
        assert!(max_val >= 1, "TransVal starts at 1");
        // Core Floor(13) tiles belong to a flooded region (non-zero). The
        // floor *variants* (fill_floor output, after FloodTransparencyValues)
        // carry dTransVal 0 in C++, so only require at least one floor tile
        // to be in a region.
        let region_floor = layout
            .floor_tiles
            .iter()
            .filter(|(x, y)| layout.trans_val[(*y as usize) * MAXDUNX + *x as usize] != 0)
            .count();
        assert!(region_floor > 0, "expected floor tiles in a flooded region");
    }


    #[test]
    fn test_build_catacombs_layout_stamps_dpiece() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        use crate::levels::drlg_l2::CatacombsGenerator;
        use crate::levels::gendung::Dungeon;

        // Synthetic L2 TIL table large enough for every L2 tile id (<=161).
        let mut til_tiles = Vec::new();
        for i in 0..200u16 {
            til_tiles.push(TilEntry { micro1: i, micro2: i + 1, micro3: i + 2, micro4: i + 3 });
        }
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Catacombs,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        let mut dungeon = Dungeon::new();
        dungeon.level_type = crate::levels::types::DungeonType::Catacombs;
        let mut gen = CatacombsGenerator::new();
        let ok = gen.generate(&mut dungeon, 0x13572468, 5);
        assert!(ok, "L2 generation should succeed");

        let layout = build_catacombs_layout(&dungeon, &level);
        assert_eq!(layout.width, MAXDUNX);
        assert_eq!(layout.height, MAXDUNY);
        assert!(count_filled(&layout) > 0, "stamped dPiece grid is non-empty");

        // Spot check every stamped logical tile: the 2x2 micro block must equal
        // the TIL mega for `tile_id - 1` (C++ DRLG_LPass3).
        let mut yy = 16usize;
        for j in 0..DMAXY {
            let mut xx = 16usize;
            for i in 0..DMAXX {
                let tile_id = dungeon.tiles[i][j] as usize;
                if tile_id > 0 && tile_id - 1 < level.til.tiles.len() {
                    let e = &level.til.tiles[tile_id - 1];
                    let b = yy * MAXDUNX + xx;
                    assert_eq!(layout.d_piece[b], e.micro1, "tile ({},{}) micro1", i, j);
                    assert_eq!(layout.d_piece[b + 1], e.micro2, "tile ({},{}) micro2", i, j);
                    assert_eq!(layout.d_piece[b + MAXDUNX], e.micro3, "tile ({},{}) micro3", i, j);
                    assert_eq!(layout.d_piece[b + MAXDUNX + 1], e.micro4, "tile ({},{}) micro4", i, j);
                }
                xx += 2;
            }
            yy += 2;
        }
    }


    #[test]
    fn test_build_caves_layout_stamps_dpiece() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        use crate::levels::drlg_l3::CavesGenerator;
        use crate::levels::gendung::Dungeon;
        use crate::levels::types::LevelEntry;

        let mut til_tiles = Vec::new();
        for i in 0..200u16 {
            til_tiles.push(TilEntry { micro1: i, micro2: i + 1, micro3: i + 2, micro4: i + 3 });
        }
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Caves,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        let mut dungeon = Dungeon::new();
        dungeon.level_type = crate::levels::types::DungeonType::Caves;
        let mut gen = CavesGenerator::new();
        let ok = gen.generate(&mut dungeon, 0x13572468, 9, LevelEntry::Main);
        assert!(ok, "L3 generation should succeed");

        let layout = build_caves_layout(&dungeon, &level);
        assert!(count_filled(&layout) > 0, "L3 layout non-empty");

        let mut yy = 16usize;
        for j in 0..DMAXY {
            let mut xx = 16usize;
            for i in 0..DMAXX {
                let tile_id = dungeon.tiles[i][j] as usize;
                if tile_id > 0 && tile_id - 1 < level.til.tiles.len() {
                    let e = &level.til.tiles[tile_id - 1];
                    let b = yy * MAXDUNX + xx;
                    assert_eq!(layout.d_piece[b], e.micro1, "L3 tile ({},{})", i, j);
                    assert_eq!(layout.d_piece[b + MAXDUNX + 1], e.micro4, "L3 tile ({},{})", i, j);
                }
                xx += 2;
            }
            yy += 2;
        }
    }

    #[test]
    fn test_build_hell_layout_stamps_dpiece() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        use crate::levels::drlg_l4::Dungeon4Generator;
        use crate::levels::gendung::Dungeon;
        use crate::levels::types::LevelEntry;

        let mut til_tiles = Vec::new();
        for i in 0..256u16 {
            til_tiles.push(TilEntry { micro1: i, micro2: i + 1, micro3: i + 2, micro4: i + 3 });
        }
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Hell,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        let mut dungeon = Dungeon::new();
        dungeon.level_type = crate::levels::types::DungeonType::Hell;
        let mut gen = Dungeon4Generator::new();
        let ok = gen.generate(&mut dungeon, 0x13572468, 13, LevelEntry::Main);
        assert!(ok, "L4 generation should succeed");

        let layout = build_hell_layout(&dungeon, &level);
        assert!(count_filled(&layout) > 0, "L4 layout non-empty");

        let mut yy = 16usize;
        for j in 0..DMAXY {
            let mut xx = 16usize;
            for i in 0..DMAXX {
                let tile_id = dungeon.tiles[i][j] as usize;
                if tile_id > 0 && tile_id - 1 < level.til.tiles.len() {
                    let e = &level.til.tiles[tile_id - 1];
                    let b = yy * MAXDUNX + xx;
                    assert_eq!(layout.d_piece[b], e.micro1, "L4 tile ({},{})", i, j);
                    assert_eq!(layout.d_piece[b + MAXDUNX + 1], e.micro4, "L4 tile ({},{})", i, j);
                }
                xx += 2;
            }
            yy += 2;
        }
    }


    #[test]
    fn test_generate_dungeon_layout_dispatches_all_levels() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};

        // A synthetic TIL large enough for every level (L4 uses tile ids < 256).
        let mut til_tiles = Vec::new();
        for i in 0..300u16 {
            til_tiles.push(TilEntry { micro1: i, micro2: i + 1, micro3: i + 2, micro4: i + 3 });
        }
        let art = DungeonLevelData {
            dungeon_type: DungeonType::Cathedral,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        for level in 1..=4u8 {
            let layout = generate_dungeon_layout(level, 0x13572468, &art)
                .unwrap_or_else(|e| panic!("L{} generation failed: {}", level, e));
            assert!(count_filled(&layout) > 0, "L{} layout non-empty", level);
        }

        assert!(generate_dungeon_layout(0, 1, &art).is_err());
        assert!(generate_dungeon_layout(5, 1, &art).is_err());
    }


    #[test]
    fn test_caves_layout_bakes_lava_pre_light() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        use crate::levels::gendung::Dungeon;

        // TIL entry 59 has micro1 = 60 — a C++ L3 lava MIN mega index.
        let mut til_tiles = Vec::new();
        for _ in 0..200u16 {
            til_tiles.push(TilEntry { micro1: 1, micro2: 2, micro3: 3, micro4: 4 });
        }
        til_tiles[59] = TilEntry { micro1: 60, micro2: 60, micro3: 60, micro4: 60 };
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Caves,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        let mut dungeon = Dungeon::new();
        dungeon.level_type = crate::levels::types::DungeonType::Caves;
        dungeon.tiles[20][20] = 60; // logical tile -> TIL 59 -> lava micros 60
        let layout = stamp_dungeon_layout(&dungeon, &level, DUNGEON_BG_TIL_INDEX[3]);

        // The lava micro tile at (16+20*2, 16+20*2) = (56, 56) is lit by
        // PlaceCaveLights (radius 7): dPreLight drops below the ambient 15.
        let lit = layout.pre_light[56 * MAXDUNX + 56];
        assert!(lit < 15, "lava tile pre_light {} should be < 15", lit);
        // A far-away tile keeps the fully-dark dungeon default.
        assert_eq!(layout.pre_light[80 * MAXDUNX + 80], 15);
    }

    /// Cathedral lava (dPiece 269) gets OBJ_L1LIGHT in C++
    /// (AddL1Objs objects.cpp:3756 + AddObjectLight radius 5); the Rust must
    /// bake the same glow into dPreLight.
    #[test]
    fn test_cathedral_layout_bakes_lava_pre_light() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        use crate::levels::gendung::Dungeon;

        // Logical tile 255 maps to TIL[254], whose micros are dPiece 269 (lava).
        let mut til_tiles = Vec::new();
        for _ in 0..300u16 {
            til_tiles.push(TilEntry { micro1: 1, micro2: 2, micro3: 3, micro4: 4 });
        }
        til_tiles[254] = TilEntry { micro1: 269, micro2: 269, micro3: 269, micro4: 269 };
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Cathedral,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };

        let mut dungeon = Dungeon::new();
        dungeon.level_type = crate::levels::types::DungeonType::Cathedral;
        dungeon.tiles[20][20] = 255; // logical tile -> TIL[254] -> lava micros 269
        let layout = stamp_dungeon_layout(&dungeon, &level, DUNGEON_BG_TIL_INDEX[1]);

        // The lava micro tile at (56, 56) is lit by the radius-5 OBJ_L1LIGHT.
        let lit = layout.pre_light[56 * MAXDUNX + 56];
        assert!(lit < 15, "L1 lava tile pre_light {} should be < 15", lit);
        // A far-away tile keeps the fully-dark dungeon default.
        assert_eq!(layout.pre_light[80 * MAXDUNX + 80], 15);
    }

    #[test]
    fn test_non_caves_layout_keeps_dark_pre_light() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MinData, PaletteData, SolData, TilData, TilEntry};
        use crate::levels::gendung::Dungeon;

        let mut til_tiles = Vec::new();
        for _ in 0..200u16 {
            til_tiles.push(TilEntry { micro1: 60, micro2: 60, micro3: 60, micro4: 60 });
        }
        let level = DungeonLevelData {
            dungeon_type: DungeonType::Catacombs,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles: vec![], blocks_per_tile: 10 },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        };
        let mut dungeon = Dungeon::new();
        dungeon.level_type = crate::levels::types::DungeonType::Catacombs;
        dungeon.tiles[20][20] = 60;
        let layout = stamp_dungeon_layout(&dungeon, &level, DUNGEON_BG_TIL_INDEX[2]);
        // L2 has no baked static lights: everything stays fully dark.
        assert!(layout.pre_light.iter().all(|&v| v == 15));
    }

    /// C++ `AddL2Torches` (objects.cpp:452-478): dPiece 0/4/36/40 trigger a
    /// FlipCoin roll; successful rolls place a torch and bake a radius-8 light.
    /// Seed 0 (verified against the Diablo LCG): after InitObjects'
    /// DiscardRandomValues(1), the four rolls are F, T, F, T.
    #[test]
    fn test_l2_torch_lights_place_and_lit() {
        let mut layout = DungeonLayout::default();
        // Fill with a non-trigger micro so only the four probe tiles roll.
        for v in layout.d_piece.iter_mut() {
            *v = 60;
        }
        layout.d_piece[10 * MAXDUNX + 10] = 0; // pn==0  -> TORCHL2 roll (fails)
        layout.d_piece[20 * MAXDUNX + 10] = 4; // pn==4  -> TORCHR2 roll (succeeds)
        layout.d_piece[30 * MAXDUNX + 10] = 36; // pn==36 -> TORCHL at NW (fails)
        layout.d_piece[40 * MAXDUNX + 10] = 40; // pn==40 -> TORCHR at NE (succeeds)
        add_l2_torch_lights(&mut layout, 0);

        // Torch placed at (20,10): TORCHR2 lights the tile itself.
        assert!(layout.pre_light[20 * MAXDUNX + 10] < 15, "TORCHR2 at (20,10) lit");
        // Torch placed at (40, 10-1) = (40,9): TORCHR at Direction::NorthEast.
        assert!(layout.pre_light[40 * MAXDUNX + 9] < 15, "TORCHR at (40,9) lit");
        // Failed rolls leave the tiles fully dark.
        assert_eq!(layout.pre_light[10 * MAXDUNX + 10], 15, "failed flip(3) stays dark");
        assert_eq!(layout.pre_light[29 * MAXDUNX + 10], 15, "failed flip(10) stays dark");
        // A far-away untouched tile keeps the ambient default.
        assert_eq!(layout.pre_light[80 * MAXDUNX + 80], 15);
    }

    /// C++ AddL1Objs/AddL2Objs/AddL3Objs (objects.cpp:3756-3795): doors come
    /// from pure dPiece micro-value scans, so scan_level_doors must map the
    /// exact C++ values to the correct ObjectId at the same position.
    #[test]
    fn test_scan_level_doors_matches_cpp_micro_values() {
        use crate::game::objdat::ObjectId;

        let mut layout = DungeonLayout::default();
        for v in layout.d_piece.iter_mut() {
            *v = 99; // non-door filler
        }
        // L1: 43/50/213 -> L1LDoor, 45/55 -> L1RDoor.
        // d_piece is indexed [y * MAXDUNX + x]; the probe tiles sit at (10,10),
        // (11,10), (12,10), (13,10), (14,10).
        layout.d_piece[10 * MAXDUNX + 10] = 43;
        layout.d_piece[10 * MAXDUNX + 11] = 50;
        layout.d_piece[10 * MAXDUNX + 12] = 213;
        layout.d_piece[10 * MAXDUNX + 13] = 45;
        layout.d_piece[10 * MAXDUNX + 14] = 55;
        let doors = scan_level_doors(1, &layout);
        assert_eq!(doors, vec![
            (10, 10, ObjectId::L1LDoor),
            (11, 10, ObjectId::L1LDoor),
            (12, 10, ObjectId::L1LDoor),
            (13, 10, ObjectId::L1RDoor),
            (14, 10, ObjectId::L1RDoor),
        ]);

        // L2: 12/540 -> L2LDoor, 16/541 -> L2RDoor.
        let mut layout2 = DungeonLayout::default();
        for v in layout2.d_piece.iter_mut() {
            *v = 99;
        }
        layout2.d_piece[10 * MAXDUNX + 10] = 12;
        layout2.d_piece[10 * MAXDUNX + 11] = 540;
        layout2.d_piece[10 * MAXDUNX + 12] = 16;
        layout2.d_piece[10 * MAXDUNX + 13] = 541;
        let doors2 = scan_level_doors(2, &layout2);
        assert_eq!(doors2, vec![
            (10, 10, ObjectId::L2LDoor),
            (11, 10, ObjectId::L2LDoor),
            (12, 10, ObjectId::L2RDoor),
            (13, 10, ObjectId::L2RDoor),
        ]);

        // L3: 530 -> L3LDoor, 533 -> L3RDoor.
        let mut layout3 = DungeonLayout::default();
        for v in layout3.d_piece.iter_mut() {
            *v = 99;
        }
        layout3.d_piece[10 * MAXDUNX + 10] = 530;
        layout3.d_piece[10 * MAXDUNX + 11] = 533;
        let doors3 = scan_level_doors(3, &layout3);
        assert_eq!(doors3, vec![
            (10, 10, ObjectId::L3LDoor),
            (11, 10, ObjectId::L3RDoor),
        ]);

        // L4 has no AddL4Objs door scan (C++ hell places no door objects).
        assert!(scan_level_doors(4, &layout).is_empty());
    }

    /// A grid with no trigger micros consumes no RNG and bakes no lights.
    #[test]
    fn test_l2_grid_without_triggers_stays_dark() {
        let mut layout = DungeonLayout::default();
        for v in layout.d_piece.iter_mut() {
            *v = 60;
        }
        add_l2_torch_lights(&mut layout, 12345);
        assert!(layout.pre_light.iter().all(|&v| v == 15));
    }


}
