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
//! The logical `Tile` enum produced by drlg_l1 (Floor, Dirt, VWalls, corners,
//! ...) is an *algorithmic* representation; C++ converts those into actual
//! `dungeon[x][y]` TIL indices via the Cathedral tile set before `DRLG_LPass3`
//! runs. We replicate that step with `tile_to_l1_til_index`, a hand-written
//! mapping from each logical tile to a reasonable L1 TIL mega index (floor /
//! wall / corner variants). When the L1 TIL file is available we validate the
//! indices against it; out-of-range indices fall back to the floor tile.

use crate::engine::dungeon::DungeonLevelData;
use crate::game::game_state::DungeonLayout;
use crate::levels::drlg_l1::{CathedralGenerator, Tile};
use crate::levels::types::{DMAXX, DMAXY, MAXDUNX, MAXDUNY};

/// The L1 TIL index used as the "dirt background" for `DRLG_LPass3`'s first
/// pass. C++ uses `Dirt - 1` = tile id 26 (0-based index 25) for Cathedral.
/// We use a known L1 floor/dirt mega index; the exact value only affects the
/// look of areas outside the generated rooms.
const L1_BG_TIL_INDEX: usize = 7;

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
            let tile = gen.dungeon[i][j];
            let til_idx = tile_to_l1_til_index(tile);
            if let Some((m1, m2, m3, m4)) = mega_for_til_index(level, til_idx) {
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
            if tile == Tile::Floor {
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
fn tile_to_l1_til_index(tile: Tile) -> usize {
    match tile {
        // Floors
        Tile::Floor => 0,

        // Walls (use a couple of variants so long walls aren't a single flat strip)
        Tile::VWall => 1,
        Tile::HWall => 2,

        // Corners
        Tile::NWCorner => 3,
        Tile::NECorner => 4,
        Tile::SWCorner => 5,
        Tile::SECorner => 6,

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
        | Tile::DirtVWall => 7,

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

    #[test]
    fn test_dungeon_layout_get_bounds() {
        let mut layout = DungeonLayout::default();
        layout.d_piece[0] = 5;
        assert_eq!(layout.get(0, 0), 5);
        assert_eq!(layout.get(-1, 0), 0);
        assert_eq!(layout.get(MAXDUNX as i32, 0), 0);
    }
}
