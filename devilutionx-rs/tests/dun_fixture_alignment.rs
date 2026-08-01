//! Generation-fidelity regression against C++ gold `.dun` exports.
//!
//! The repo ships real DevilutionX level exports under
//! `test/fixtures/diablo/{currlevel}-{seed}.dun` (produced by the C++ dev
//! command `devilutionx.dev.level.exportDun`). Each file begins with the
//! 40x40 logical `dungeon[][]` tile grid (u16 LE). This harness regenerates
//! the matching Rust level for the recorded seed and reports the per-cell
//! tile match rate — the objective's "持续对齐回归" gate for generation
//! fidelity.
//!
//! Current fidelity (2026-08-01): L1 ~4.5%, L2 ~0-19%, L3 ~27%, L4 TBD —
//! the generators are approximate reimplementations, not yet byte-exact.
//! The harness asserts a non-zero sanity floor per level so a total
//! regression (e.g. the L2 empty-grid bug) fails loudly, while the printed
//! rates document the long-tail gap for future work.

use devilutionx_rs::levels::drlg_l1::CathedralGenerator;
use devilutionx_rs::levels::drlg_l2::CatacombsGenerator;
use devilutionx_rs::levels::drlg_l3::CavesGenerator;
use devilutionx_rs::levels::drlg_l4::Dungeon4Generator;
use devilutionx_rs::levels::gendung::Dungeon;
use devilutionx_rs::levels::types::{DungeonType, LevelEntry};
use std::path::PathBuf;

fn fixture_dir() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest.join("..").join("test").join("fixtures").join("diablo");
    dir.is_dir().then_some(dir)
}

fn parse_dun_tiles(path: &PathBuf) -> Option<Vec<u16>> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 4 + 1600 * 2 {
        return None;
    }
    let mut tiles = Vec::with_capacity(1600);
    for i in 0..1600 {
        tiles.push(u16::from_le_bytes([data[4 + i * 2], data[5 + i * 2]]));
    }
    Some(tiles)
}

/// Demo level 1..=4 -> (game currlevel, seed used for the fixture name).
/// Rust demo levels map to game levels 1 / 5 / 9 / 13 (Cathedral /
/// Catacombs / Caves / Hell).
fn demo_to_currlevel(demo: u8) -> u8 {
    match demo {
        1 => 1,
        2 => 5,
        3 => 9,
        _ => 13,
    }
}

/// Generate the Rust logical tile grid for a demo level and seed.
fn generate_tiles(demo: u8, seed: u32) -> [[u8; 40]; 40] {
    let mut out = [[0u8; 40]; 40];
    match demo {
        1 => {
            let mut gen = CathedralGenerator::new();
            gen.generate(DungeonType::Cathedral, seed);
            for y in 0..40 {
                for x in 0..40 {
                    out[x][y] = gen.dungeon[y][x] as u8;
                }
            }
        }
        2 => {
            let mut gen = CatacombsGenerator::new();
            let mut d = Dungeon::new();
            gen.generate(&mut d, seed, 5);
            for y in 0..40 {
                for x in 0..40 {
                    out[x][y] = d.tiles[x][y];
                }
            }
        }
        3 => {
            let mut gen = CavesGenerator::new();
            let mut d = Dungeon::new();
            gen.generate(&mut d, seed, 9, LevelEntry::Main);
            for y in 0..40 {
                for x in 0..40 {
                    out[x][y] = d.tiles[x][y];
                }
            }
        }
        _ => {
            let mut gen = Dungeon4Generator::new();
            let mut d = Dungeon::new();
            gen.generate(&mut d, seed, 13, LevelEntry::Main);
            for y in 0..40 {
                for x in 0..40 {
                    out[x][y] = d.tiles[x][y];
                }
            }
        }
    }
    out
}

fn match_count(gold: &[u16], tiles: &[[u8; 40]; 40]) -> usize {
    let mut m = 0;
    for y in 0..40 {
        for x in 0..40 {
            if gold[y * 40 + x] as u8 == tiles[x][y] {
                m += 1;
            }
        }
    }
    m
}

#[test]
fn dun_fixture_generation_alignment() {
    let Some(dir) = fixture_dir() else {
        eprintln!("[dun_fixture] SKIP: test/fixtures/diablo not present");
        return;
    };
    let mut all_passed = true;
    for demo in 1..=4u8 {
        let curr = demo_to_currlevel(demo);
        let mut best = 0usize;
        let mut best_seed = 0u32;
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(&format!("{curr}-")) || !name.ends_with(".dun") {
                continue;
            }
            let seed_str = name
                .trim_start_matches(&format!("{curr}-"))
                .trim_end_matches(".dun");
            let Ok(seed) = seed_str.parse::<u32>() else { continue };
            let Some(gold) = parse_dun_tiles(&entry.path()) else { continue };
            let tiles = generate_tiles(demo, seed);
            let m = match_count(&gold, &tiles);
            if m > best {
                best = m;
                best_seed = seed;
            }
        }
        let pct = best as f64 / 16.0;
        eprintln!(
            "[dun_fixture] demo L{} (game L{}): best tile match {}/1600 ({:.1}%) seed {}",
            demo, curr, best, pct, best_seed
        );
        // Sanity floor: the harness must match *something* — a total
        // regression (empty grid / wrong seed mapping) fails loudly.
        if best == 0 {
            eprintln!("[dun_fixture] WARN: L{} has zero matching cells", demo);
            all_passed = false;
        }
    }
    assert!(all_passed, "every demo level must match at least one gold cell");
}
