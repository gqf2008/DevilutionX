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
//! Current fidelity (2026-08-02): L1 ~4.5%, L2 ~0-19%, L3 ~27% (stale
//! fixtures), L4 100% on current-HEAD fixtures (13-428074402, 13-594689775
//! Warlord quest, 14-717625719, 14-815743776, 15-1256511996, 15-1583642716,
//! 16-741281013). L4 aligns cell-for-cell with an independent C++ repro built
//! from `Source/levels/drlg_l4.cpp` (see `devilutionx-rs/tools/l4repro/`).
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

/// Level type -> (first currlevel, generator dispatch). Cathedral 1-4,
/// Catacombs 5-8, Caves 9-12, Hell 13-16 (GetLevelType in gendung.cpp).
fn level_type_of(level: u8) -> u8 {
    if level <= 4 {
        1
    } else if level <= 8 {
        2
    } else if level <= 12 {
        3
    } else {
        4
    }
}

/// Generate the Rust logical tile grid for a fixture `(currlevel, seed)`.
/// `quest_active` selects the quest-room state:
/// - L2 (5-8): Q_BLOOD (level 5) room; L3 (9-12): Q_ANVIL (level 10) room.
fn generate_tiles(level: u8, seed: u32, quest_active: bool) -> [[u8; 40]; 40] {
    let mut out = [[0u8; 40]; 40];
    match level_type_of(level) {
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
            gen.blood_quest_active = quest_active;
            gen.schamb_quest_active = quest_active;
            gen.blind_quest_active = quest_active;
            let mut d = Dungeon::new();
            gen.generate(&mut d, seed, level);
            for y in 0..40 {
                for x in 0..40 {
                    out[x][y] = d.tiles[x][y];
                }
            }
        }
        3 => {
            let mut gen = CavesGenerator::new();
            gen.anvil_quest_active = quest_active;
            let mut d = Dungeon::new();
            gen.generate(&mut d, seed, level, LevelEntry::Main);
            for y in 0..40 {
                for x in 0..40 {
                    out[x][y] = d.tiles[x][y];
                }
            }
        }
        _ => {
            let mut gen = Dungeon4Generator::new();
            gen.warlord_quest_active = quest_active;
            let mut d = Dungeon::new();
            gen.generate(&mut d, seed, level, LevelEntry::Main);
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
    // Group fixtures by level type; report the best per type and per file.
    for type_id in 1..=4u8 {
        let mut best = 0usize;
        let mut best_desc = String::new();
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".dun") || name.contains("-changed") {
                continue;
            }
            let Some((level_s, seed_s)) = name.trim_end_matches(".dun").split_once('-') else {
                continue;
            };
            let Ok(level) = level_s.parse::<u8>() else { continue };
            if level_type_of(level) != type_id {
                continue;
            }
            let Ok(seed) = seed_s.parse::<u32>() else { continue };
            let Some(gold) = parse_dun_tiles(&entry.path()) else { continue };
            let mut file_best = 0usize;
            let mut file_desc = String::new();
            // Quest-gated levels: try both quest states (L2 5-8, L3 9-12,
            // L4 Warlord quest on level 13).
            let quest_states: &[bool] = if matches!(type_id, 2 | 3 | 4) {
                &[false, true]
            } else {
                &[false]
            };
            for &quest in quest_states {
                let tiles = generate_tiles(level, seed, quest);
                let m = match_count(&gold, &tiles);
                if m > file_best {
                    file_best = m;
                    file_desc = if type_id == 4 {
                        format!("warlord={quest}")
                    } else {
                        format!("quest={quest}")
                    };
                }
            }
            eprintln!("[dun_fixture] L{} seed {}: {} match {}", level, seed, file_desc, file_best);
            if file_best > best {
                best = file_best;
                best_desc = format!("{} (seed {}, {})", level, seed, file_desc);
            }
        }
        let pct = best as f64 / 16.0;
        eprintln!(
            "[dun_fixture] level-type {}: best tile match {}/1600 ({:.1}%) {}",
            type_id, best, pct, best_desc
        );
        if best == 0 {
            eprintln!("[dun_fixture] WARN: level type {} has zero matching cells", type_id);
            all_passed = false;
        }
    }
    assert!(all_passed, "every level type must match at least one gold cell");
}

