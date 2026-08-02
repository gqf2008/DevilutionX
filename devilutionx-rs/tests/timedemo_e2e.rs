//! Timedemo end-to-end harness — Tier 0/1 scaffold.
//!
//! Mirrors the C++ gold standard `test/timedemo_test.cpp::RunTimedemo("WarriorLevel1to2")`,
//! which replays `demo_0.dmo` over `spawn_0.sv` and byte-compares the final save
//! against `demo_0_reference_spawn_0.sv`.
//!
//! - `parse_real_demo_fixture` / `spawn_save_is_mpq` — Tier 0: validate the demo
//!   parser and save detection against the real fixtures. These PASS today.
//! - `replay_warrior_level1to2` — Tier 1..3: full headless replay + byte-compare.
//!   `#[ignore]`d until the engine's game loop / save systems are ported.

use devilutionx_rs::engine::demo_reader::{
    load_save_archive, parse_demo, DemoEventType, DemoParseError, DemoPayload, ReplayDriver,
};

/// Path to a file in the upstream `test/fixtures/timedemo/WarriorLevel1to2/` dir,
/// resolved from this crate's manifest dir (so it works regardless of CWD).
fn fixture_path(name: &str) -> String {
    format!(
        "{}/../test/fixtures/timedemo/WarriorLevel1to2/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Tier 1 headless driver: run the demo's GameTick events through a real
/// `GameState` update, headlessly (no SDL). The input events are not yet
/// translated into actions (that is Tier 2), but this proves the engine loop
/// can be driven deterministically from the demo stream.
#[test]
fn headless_replay_drives_game_ticks() {
    let data = match std::fs::read(fixture_path("demo_0.dmo")) {
        Ok(d) => d,
        Err(e) => panic!("demo fixture missing ({}): {e}", fixture_path("demo_0.dmo")),
    };
    let demo = parse_demo(&data).expect("demo should parse");
    let mut driver = ReplayDriver::new(demo);
    let total_ticks = driver.game_tick_count();
    assert!(total_ticks > 1000, "WarriorLevel1to2 drives thousands of game ticks, got {total_ticks}");

    // Bounded headless run: the full 4.8k ticks are slow; 200 ticks exercise
    // player/monster/simple-missile processing deterministically.
    fn run_ticks(driver: &mut ReplayDriver, limit: usize) -> devilutionx_rs::game::game_state::GameState {
        use devilutionx_rs::game::game_state::GameState;
        use devilutionx_rs::game::player_exact::{HeroClass, Player};
        use rand::SeedableRng;
        let mut player = Player::new();
        player.init_class_stats();
        player._p_class = HeroClass::Warrior;
        let mut gs = GameState::new(player, false, 12345);
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mut tick = 0;
        driver.reset();
        while tick < limit {
            match driver.peek() {
                Some(ev) if ev.event_type == DemoEventType::GameTick => {
                    gs.update(&mut rng);
                    tick += 1;
                }
                Some(_) => {
                    driver.step();
                }
                None => break,
            }
        }
        assert_eq!(tick, limit, "demo has enough GameTick events");
        gs
    }

    let mut d1 = ReplayDriver::new(parse_demo(&data).unwrap());
    let mut d2 = ReplayDriver::new(parse_demo(&data).unwrap());
    let gs1 = run_ticks(&mut d1, 200);
    let gs2 = run_ticks(&mut d2, 200);
    assert_eq!(
        (gs1.player.position.x, gs1.player.position.y, gs1.player._p_hit_points, gs1.game_tick),
        (gs2.player.position.x, gs2.player.position.y, gs2.player._p_hit_points, gs2.game_tick),
        "headless replay is deterministic across two runs"
    );
}

/// Tier 2 headless driver: translate the demo's MouseButtonDown events into
/// click-to-move targets and walk toward them on GameTicks (mirroring the
/// game loop's `tick_move_target`), then run `GameState::update`. This is the
/// input->action layer of the replay; byte-compare against C++ remains Tier 3.
#[test]
fn headless_replay_applies_click_to_move() {
    let data = match std::fs::read(fixture_path("demo_0.dmo")) {
        Ok(d) => d,
        Err(e) => panic!("demo fixture missing ({}): {e}", fixture_path("demo_0.dmo")),
    };
    let demo = parse_demo(&data).expect("demo should parse");
    let mut driver = ReplayDriver::new(demo);

    fn run(driver: &mut ReplayDriver, limit: usize) -> (i32, i32, u32) {
        use devilutionx_rs::game::game_loop::{convert_screen_to_tile, tick_move_target};
        use devilutionx_rs::game::game_state::GameState;
        use devilutionx_rs::game::player_exact::{HeroClass, Player};
        use rand::SeedableRng;
        let mut player = Player::new();
        player.init_class_stats();
        player._p_class = HeroClass::Warrior;
        let mut gs = GameState::new(player, false, 12345);
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mut move_target: Option<(i32, i32)> = None;
        let mut ticks = 0;
        let mut clicks = 0;
        driver.reset();
        while ticks < limit {
            match driver.peek() {
                Some(ev) => {
                    match ev.event_type {
                        DemoEventType::MouseButtonDown => {
                            if let DemoPayload::MouseButton { x, y, .. } = ev.payload {
                                let cam = gs.camera;
                                move_target = Some(convert_screen_to_tile(
                                    x as i32, y as i32, cam.tile_x, cam.tile_y,
                                ));
                                clicks += 1;
                            }
                        }
                        DemoEventType::GameTick => {
                            if let Some(target) = move_target {
                                tick_move_target(&mut gs, target, &mut move_target);
                            }
                            gs.update(&mut rng);
                            ticks += 1;
                        }
                        _ => {}
                    }
                    driver.step();
                }
                None => break,
            }
        }
        assert!(clicks > 0, "demo should contain mouse clicks");
        assert_eq!(ticks, limit, "demo has enough GameTick events");
        (gs.player.position.x, gs.player.position.y, gs.game_tick)
    }

    let mut d1 = ReplayDriver::new(parse_demo(&data).unwrap());
    let mut d2 = ReplayDriver::new(parse_demo(&data).unwrap());
    let s1 = run(&mut d1, 200);
    let s2 = run(&mut d2, 200);
    assert_eq!(s1, s2, "click-to-move replay is deterministic");
    // The player must have moved from the start (clicks drive movement).
    assert!(
        s1.0 != 56 || s1.1 != 56,
        "player moved away from the initial spawn, got ({},{})",
        s1.0,
        s1.1
    );
}

/// Tier 3 prerequisite: run the ENTIRE WarriorLevel1to2 demo stream (all
/// game ticks + every input) through `GameState::update` headlessly with
/// click-to-move, and assert the pipeline completes deterministically with the
/// player making real progress. Byte-comparing the final save against
/// `demo_0_reference_spawn_0.sv` is the remaining Tier 3 step (the engine
/// save writer must match C++ byte-for-byte).
#[test]
fn full_replay_runs_to_completion() {
    use devilutionx_rs::engine::demo_reader::DemoPayload;
    use devilutionx_rs::game::game_loop::{convert_screen_to_tile, tick_move_target};
    use devilutionx_rs::game::game_state::GameState;
    use devilutionx_rs::game::player_exact::{HeroClass, Player};
    use rand::SeedableRng;

    let data = match std::fs::read(fixture_path("demo_0.dmo")) {
        Ok(d) => d,
        Err(e) => panic!("demo fixture missing ({}): {e}", fixture_path("demo_0.dmo")),
    };
    let total_ticks = {
        let mut driver = ReplayDriver::new(parse_demo(&data).unwrap());
        driver.game_tick_count()
    };
    assert!(total_ticks > 4000, "WarriorLevel1to2 drives >4k ticks, got {total_ticks}");

    fn run(
        driver: &mut ReplayDriver,
        total_ticks: usize,
    ) -> (i32, i32, u32, usize) {
        // Reset the gameplay LCG per run so both runs are byte-identical even
        // for LCG-dependent state.
        devilutionx_rs::engine::random::seed_gameplay_rng(12345);
        let mut player = Player::new();
        player.init_class_stats();
        player._p_class = HeroClass::Warrior;
        let mut gs = GameState::new(player, false, 12345);
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mut move_target: Option<(i32, i32)> = None;
        let mut ticks = 0usize;
        let mut clicks = 0usize;
        driver.reset();
        while let Some(ev) = driver.peek() {
            match ev.event_type {
                DemoEventType::MouseButtonDown => {
                    if let DemoPayload::MouseButton { x, y, .. } = ev.payload {
                        let cam = gs.camera;
                        move_target =
                            Some(convert_screen_to_tile(x as i32, y as i32, cam.tile_x, cam.tile_y));
                        clicks += 1;
                    }
                }
                DemoEventType::GameTick => {
                    if let Some(target) = move_target {
                        tick_move_target(&mut gs, target, &mut move_target);
                    }
                    gs.update(&mut rng);
                    ticks += 1;
                }
                _ => {}
            }
            driver.step();
        }
        assert!(clicks > 0, "demo should contain mouse clicks");
        (gs.player.position.x, gs.player.position.y, gs.game_tick, ticks)
    }

    let mut d1 = ReplayDriver::new(parse_demo(&data).unwrap());
    let mut d2 = ReplayDriver::new(parse_demo(&data).unwrap());
    let s1 = run(&mut d1, total_ticks);
    let s2 = run(&mut d2, total_ticks);
    assert_eq!(s1.3, total_ticks, "all game ticks processed");
    assert_eq!(s1, s2, "full replay is deterministic across two runs");
    // The player made real progress from the initial spawn (56, 56).
    let moved = (s1.0 - 56).abs() + (s1.1 - 56).abs();
    assert!(moved > 0, "player moved from the initial spawn");
    println!(
        "[FullReplay] ticks={} final=({}, {}) moved={}",
        s1.3, s1.0, s1.1, moved
    );
}

#[test]
fn parse_real_demo_fixture() {
    // Sanity: the fixture must be present. The upstream repo ships it.
    let data = match std::fs::read(fixture_path("demo_0.dmo")) {
        Ok(d) => d,
        Err(e) => panic!("demo fixture missing ({}): {e}", fixture_path("demo_0.dmo")),
    };

    let demo = parse_demo(&data).expect("demo should parse");

    // Header — fixed by the recorded format.
    assert_eq!(demo.header.version, 3, "fixture is demo version 3");
    assert_eq!(demo.header.save_number, 0, "gSaveNumber for this fixture");
    assert!(demo.header.graphics_width > 0);
    assert!(demo.header.graphics_height > 0);
    assert_eq!(
        demo.header.settings_bytes.len(),
        23,
        "version>0 settings = 17 bools + 6 potion bytes"
    );

    // A real level-1-to-2 run is thousands of events (ticks + inputs).
    assert!(
        demo.events.len() > 500,
        "expected a substantial event stream, got {}",
        demo.events.len()
    );

    // It must contain at least game-ticks and some player input.
    let has_ticks = demo.events.iter().any(|e| e.event_type == DemoEventType::GameTick);
    let has_rendering = demo.events.iter().any(|e| e.event_type == DemoEventType::Rendering);
    assert!(has_ticks, "demo should contain GameTick events");
    assert!(has_rendering, "demo should contain Rendering events");
}

#[test]
fn driver_runs_full_event_stream() {
    let data = std::fs::read(fixture_path("demo_0.dmo")).expect("demo fixture");
    let demo = parse_demo(&data).expect("parse");
    let total = demo.events.len();

    let mut driver = ReplayDriver::new(demo);
    assert_eq!(driver.event_count(), total);

    let summary = driver.run_to_completion();
    assert_eq!(summary.events_processed, total, "driver should consume every event");
    assert!(!summary.by_type.is_empty());

    // After completion the cursor is exhausted.
    assert!(driver.peek().is_none());
    driver.reset();
    assert!(driver.peek().is_some());
}

#[test]
fn spawn_save_is_mpq_archive() {
    // C++ timedemo loads `spawn_0.sv` as a save game. Diablo save files are MPQ
    // archives (magic "MPQ\x1a"), so Tier 1 save-loading goes through the MPQ
    // reader + the inner save-structure parser.
    let data = std::fs::read(fixture_path("spawn_0.sv")).expect("save fixture");
    assert!(data.len() > 4, "save file truncated");
    assert_eq!(&data[0..4], b"MPQ\x1a", "spawn_0.sv must be an MPQ archive");

    // The reference save (what the final state must byte-match in Tier 3) is
    // present and also an MPQ of the same size.
    let reference = std::fs::read(fixture_path("demo_0_reference_spawn_0.sv")).expect("reference fixture");
    assert_eq!(&reference[0..4], b"MPQ\x1a");
    assert_eq!(reference.len(), data.len(), "reference save should match initial save size");
}

#[test]
fn rejects_unsupported_demo_version() {
    // A lone version byte of 0 (too old) or 5 (too new) must be rejected.
    let v0 = [0u8, 0, 0, 0, 0];
    assert!(matches!(
        parse_demo(&v0),
        Err(DemoParseError::UnsupportedVersion(0))
    ));
    let v5 = [5u8];
    assert!(matches!(
        parse_demo(&v5),
        Err(DemoParseError::UnsupportedVersion(5))
    ));
}

/// Tier 1: the save archive opens as an MPQ and we can see its structure.
///
/// This proves the save-loading pipeline works end-to-end against the real
/// fixture: `engine::mpq::MpqArchive` reads the header, decrypts the hash and
/// block tables, and exposes block-table entries. We don't yet parse the inner
/// save structure (hero/level blobs) — that's Tier 2+.
#[test]
fn loads_save_archive_as_mpq() {
    let save = load_save_archive(fixture_path("spawn_0.sv"))
        .expect("spawn_0.sv should open as an MPQ archive");

    // A real Diablo save has many block-table entries (hero data, per-level
    // blobs, quest state, etc.). Non-zero means the archive parsed and we can
    // reach its contents.
    let blocks = save.block_count();
    assert!(blocks > 0, "save archive should have inner entries, got {blocks}");

    // Sanity: the reference save opens the same way and has the same structure.
    let reference = load_save_archive(fixture_path("demo_0_reference_spawn_0.sv"))
        .expect("reference save should open as an MPQ archive");
    assert_eq!(
        reference.block_count(),
        blocks,
        "initial and reference saves should share block-table layout"
    );

    // TODO(Tier 2): feed the decoded inner bytes into the engine's save loader.
}

/// Tier 1.5: byte-exact decode of a real C++ save via the vanilla codec.
///
/// `spawn_0.sv` is a genuine C++ save (spawn single-player, password
/// `adslhfb1` — Source/pfile.cpp PASSWORD_SPAWN_SINGLE). The `game` entry
/// decrypts to the C++ `SaveGameData` header: magic `"SHAR"` (spawn
/// non-Hellfire, Source/loadsave.cpp:2766-2773), then setlevel u8, setlvlnum
/// BE u32, currlevel BE u32, leveltype BE u32. The fixture is the opening
/// save of WarriorLevel1to2, so currlevel must be 1.
#[test]
fn decodes_real_cpp_save_game_entry() {
    use devilutionx_rs::game::codec::codec_decode;

    const PASSWORD_SPAWN_SINGLE: &str = "adslhfb1";

    for (name, save_path) in [
        ("initial", "spawn_0.sv"),
        ("reference", "demo_0_reference_spawn_0.sv"),
    ] {
        let mut save = load_save_archive(fixture_path(save_path))
            .expect("save should open as an MPQ archive");
        assert!(save.has_entry("game"), "{name} save has a game entry");
        let raw = save.read_entry("game").expect("read game entry");
        let decoded = codec_decode(&raw, PASSWORD_SPAWN_SINGLE);
        assert!(decoded.len() >= 43, "{name} game entry decodes to a header");
        let header = devilutionx_rs::game::loadsave::CppGameHeader::parse(&decoded)
            .expect("C++ SaveGameData header parses");
        assert_eq!(
            &header.magic,
            b"SHAR",
            "{name} game magic must be SHAR (spawn, non-Hellfire)"
        );
        assert_eq!(header.setlevel, 0, "{name} setlevel must be 0");
        assert_eq!(header.currlevel, 1, "{name} fixture starts at level 1");
        assert_eq!(
            header.leveltype, 1,
            "{name} level 1 is DTYPE_CATHEDRAL (=1) via getHellfireLevelType"
        );
        assert!(header.active_monster_count >= 0, "{name} monster count sane");
        assert!(header.active_item_count >= 0, "{name} item count sane");
        // Level-seed table (17 classic levels): town (level 0) is DTYPE_TOWN,
        // level 1 is DTYPE_CATHEDRAL, per getHellfireLevelType.
        let seeds = devilutionx_rs::game::loadsave::CppGameHeader::parse_level_seeds(&decoded, 17)
            .expect("level seed table parses");
        assert_eq!(seeds.len(), 17, "{name} classic game has 17 levels");
        assert_eq!(seeds[0].1, 0, "{name} level 0 is DTYPE_TOWN (=0)");
        assert_eq!(seeds[1].1, 1, "{name} level 1 is DTYPE_CATHEDRAL (=1)");
        // The active level's seed must be non-zero (the generator reseeds it).
        assert!(seeds[header.currlevel as usize].0 != 0, "{name} active level seed set");
        // A non-spawn password must fail the checksum (proves the password gate).
        let wrong = codec_decode(&raw, "xrgyrkj1");
        assert!(wrong.is_empty(), "wrong password must be rejected by checksum");
    }

    // The hero entry decrypts to the C++ `PlayerPack` (pfile.cpp EncodeHero:
    // a `#pragma pack(1)` struct codec-encrypted with the save password). The
    // Rust `pack::PlayerPack::from_bytes` parses that packed layout, so the
    // real fixture hero must decode to a Warrior with a name.
    let mut save = load_save_archive(fixture_path("spawn_0.sv")).expect("open save");
    assert!(save.has_entry("hero"), "save has a hero entry");
    let raw = save.read_entry("hero").expect("read hero entry");
    let hero = codec_decode(&raw, PASSWORD_SPAWN_SINGLE);
    assert!(hero.len() > 100, "hero blob is substantial, got {}", hero.len());
    let pack = devilutionx_rs::game::pack::PlayerPack::from_bytes(&hero);
    assert_eq!(pack.class, 0, "fixture hero is a Warrior (PC_WARRIOR=0)");
    assert_eq!(pack.name, "timedemo", "fixture hero name is the timedemo Warrior");
    assert_eq!(pack.plr_level, 1, "fixture hero starts at dungeon level 1");
}

/// Tier 1..3 acceptance: load `spawn_0.sv`, replay `demo_0.dmo` headlessly through
/// the engine game loop, and byte-compare the final save against the reference.
///
/// Ignored until the engine systems (headless `game::game_loop`, MPQ save loading,
/// state capture) are ported. Un-ignore and implement in `ReplayDriver::step` /
/// `run_to_completion` as those land.
#[test]
#[ignore = "Tier 1..3: headless replay + byte-compare pending engine port"]
fn replay_warrior_level1to2() {
    let data = std::fs::read(fixture_path("demo_0.dmo")).expect("demo fixture");
    let mut driver = ReplayDriver::from_bytes(&data).expect("parse");

    // TODO(Tier 1): load spawn_0.sv (MPQ) into engine state.
    // TODO(Tier 1): drive game::game_loop headlessly over the demo event stream.
    let summary = driver.run_to_completion();
    assert!(summary.events_processed > 0);

    // TODO(Tier 3): compare final engine save state to demo_0_reference_spawn_0.sv
    // byte-for-byte (HeroCompareResult::Same in C++).
}

/// SaveQuest/SavePortal writers must match the C++ byte layout
/// (loadsave.cpp:1762-1789 / 1811-1818): 44-byte quests and 24-byte portals
/// that round-trip through the parser.
#[test]
fn quest_portal_writers_round_trip() {
    use devilutionx_rs::game::loadsave::{
        SaveHelper, parse_portal, parse_quest, write_portal, write_quest,
    };
    use devilutionx_rs::game::quest_new::{Quest, QuestId, QuestState, SpeechId};

    let mut q = Quest::default();
    q._qlevel = 3;
    q._qidx = QuestId::Mushroom;
    q._qactive = QuestState::Active;
    q.position = (10, 20);
    q._qmsg = SpeechId::Mush10;
    q._qvar1 = 5;
    q._qlog = true;

    let mut helper = SaveHelper::new(44);
    write_quest(&mut helper, &q, (5, 6), 2, 1);
    let data = helper.into_data();
    assert_eq!(data.len(), 44, "classic SaveQuest is 44 bytes");
    let (parsed, next) = parse_quest(&data, 0).expect("quest parses");
    assert_eq!(next, 44);
    assert_eq!(parsed._qlevel, q._qlevel);
    assert_eq!(parsed._qidx, q._qidx);
    assert_eq!(parsed._qactive, q._qactive);
    assert_eq!(parsed.position, q.position);
    assert_eq!(parsed._qslvl, q._qslvl);
    assert_eq!(parsed._qmsg, q._qmsg);
    assert_eq!(parsed._qvar1, q._qvar1);
    assert_eq!(parsed._qvar2, q._qvar2);
    assert_eq!(parsed._qlog, q._qlog);

    let mut helper = SaveHelper::new(24);
    write_portal(&mut helper, true, (30, 31), 2, 1, false);
    let data = helper.into_data();
    assert_eq!(data.len(), 24, "SavePortal is 24 bytes");
    let ((open, pos, level, ltype, setlvl), next) = parse_portal(&data, 0).expect("portal parses");
    assert_eq!(next, 24);
    assert!(open);
    assert_eq!(pos, (30, 31));
    assert_eq!(level, 2);
    assert_eq!(ltype, 1);
    assert!(!setlvl);
}

/// Tier 3 foundation: the Rust `CppGameHeader` writer must reproduce the
/// C++ `SaveGameData` header + level-seed table byte-for-byte. Decodes the
/// real C++ reference save, re-serialises the parsed header/seeds, and
/// compares against the original decoded bytes.
#[test]
fn save_writer_header_matches_cpp_reference_bytes() {
    use devilutionx_rs::game::codec::codec_decode;
    use devilutionx_rs::game::loadsave::CppGameHeader;

    const PASSWORD_SPAWN_SINGLE: &str = "adslhfb1";
    let mut save = load_save_archive(fixture_path("demo_0_reference_spawn_0.sv"))
        .expect("reference save opens as MPQ");
    let raw = save.read_entry("game").expect("game entry");
    let decoded = codec_decode(&raw, PASSWORD_SPAWN_SINGLE);
    assert!(decoded.len() >= 43 + 17 * 8, "decoded entry has header + seed table");

    let header = CppGameHeader::parse(&decoded).expect("header parses");
    let written_header = header.write();
    assert_eq!(
        &written_header[..],
        &decoded[..43],
        "header round-trip is byte-exact vs C++ SaveGameData"
    );

    let seeds = CppGameHeader::parse_level_seeds(&decoded, 17).expect("seeds parse");
    let written_seeds = CppGameHeader::write_level_seeds(&seeds);
    assert_eq!(
        &written_seeds[..],
        &decoded[43..43 + 17 * 8],
        "level-seed table round-trip is byte-exact vs C++"
    );
}

/// PlayerPack write direction: decoding the C++ reference hero entry and
/// re-encoding with `PlayerPack::to_bytes` must be byte-stable (the pack
/// layout round-trips exactly), so the save writer can emit the `hero`
/// entry byte-for-byte from engine state.
#[test]
fn player_pack_round_trip_is_byte_stable() {
    use devilutionx_rs::game::codec::codec_decode;
    use devilutionx_rs::game::pack::PlayerPack;

    const PASSWORD_SPAWN_SINGLE: &str = "adslhfb1";
    let mut save = load_save_archive(fixture_path("demo_0_reference_spawn_0.sv"))
        .expect("reference save opens as MPQ");
    let raw = save.read_entry("hero").expect("hero entry");
    let hero = codec_decode(&raw, PASSWORD_SPAWN_SINGLE);
    assert!(hero.len() > 100, "hero blob is substantial");

    let pack = PlayerPack::from_bytes(&hero);
    let reencoded = pack.to_bytes();
    assert_eq!(
        reencoded, hero,
        "PlayerPack round-trip must be byte-stable vs C++ EncodeHero (len {} vs {})",
        reencoded.len(),
        hero.len()
    );
}

/// Byte-exact decoding against the C++ `demomode.cpp` record layout
/// (`WriteDemoMsgHeader` + per-type payload, version 3):
///   header: [version u8][save u32le][w u16le][h u16le][23 settings bytes]
///   record: [type u8][progress u8][payload]  (Rendering <=127: single byte)
///   Key:    [sym u32le][mod u16le]
///   MouseButton: [button u8][x u16le][y u16le][mod u16le]
///   MouseMotion: [x u16le][y u16le]
#[test]
fn decodes_synthetic_version3_events_byte_exactly() {
    use devilutionx_rs::engine::demo_reader::DemoPayload;

    let mut data = Vec::new();
    // Header (version 3, save 0, 640x480, 23 settings bytes).
    data.push(3);
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&640u16.to_le_bytes());
    data.extend_from_slice(&480u16.to_le_bytes());
    data.extend_from_slice(&[0u8; 23]);

    // Rendering with progress 5 (inline single byte, high bit set).
    data.push(0x80 | 5);
    // GameTick progress 0 (no payload).
    data.push(0);
    data.push(0);
    // KeyDown type 13 progress 1: sym 0x11223344 LE, mod 0x0102 LE.
    data.push(13);
    data.push(1);
    data.extend_from_slice(&0x1122_3344u32.to_le_bytes());
    data.extend_from_slice(&0x0102u16.to_le_bytes());
    // MouseButtonDown type 10 progress 2: button 1, x 100, y 200, mod 3.
    data.push(10);
    data.push(2);
    data.push(1);
    data.extend_from_slice(&100u16.to_le_bytes());
    data.extend_from_slice(&200u16.to_le_bytes());
    data.extend_from_slice(&3u16.to_le_bytes());
    // MouseMotion type 9 progress 3: x 640, y 320.
    data.push(9);
    data.push(3);
    data.extend_from_slice(&640u16.to_le_bytes());
    data.extend_from_slice(&320u16.to_le_bytes());

    let demo = parse_demo(&data).expect("synthetic version-3 demo parses");
    assert_eq!(demo.header.version, 3);
    assert_eq!(demo.header.graphics_width, 640);
    assert_eq!(demo.events.len(), 5);

    // [0] Rendering (inline).
    assert_eq!(demo.events[0].event_type, DemoEventType::Rendering);
    assert_eq!(demo.events[0].progress_to_next_game_tick, 5);
    // [1] GameTick.
    assert_eq!(demo.events[1].event_type, DemoEventType::GameTick);
    // [2] KeyDown with C++ payload.
    assert_eq!(demo.events[2].event_type, DemoEventType::KeyDown);
    assert_eq!(
        demo.events[2].payload,
        DemoPayload::Key { sym: 0x1122_3344, mod_state: 0x0102 }
    );
    // [3] MouseButtonDown with C++ payload (button first).
    assert_eq!(demo.events[3].event_type, DemoEventType::MouseButtonDown);
    assert_eq!(
        demo.events[3].payload,
        DemoPayload::MouseButton { button: 1, x: 100, y: 200, mod_state: 3 }
    );
    // [4] MouseMotion.
    assert_eq!(demo.events[4].event_type, DemoEventType::MouseMotion);
    assert_eq!(
        demo.events[4].payload,
        DemoPayload::MouseMotion { x: 640, y: 320 }
    );
}
