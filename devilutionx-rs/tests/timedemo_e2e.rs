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
    load_save_archive, parse_demo, DemoEventType, DemoParseError, ReplayDriver,
};

/// Path to a file in the upstream `test/fixtures/timedemo/WarriorLevel1to2/` dir,
/// resolved from this crate's manifest dir (so it works regardless of CWD).
fn fixture_path(name: &str) -> String {
    format!(
        "{}/../test/fixtures/timedemo/WarriorLevel1to2/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
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

    // TODO(Tier 1.5): once the inner save-structure names are known, probe them
    // here with `save.has_entry("<hero>")` etc. Diablo saves typically omit the
    // MPQ `(listfile)`, so `list_entries()` is expected to fail — that's fine,
    // the block-table count above is the real acceptance signal.
    // TODO(Tier 2): feed the decoded inner bytes into the engine's save loader.
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
