//! Diagnostic: decode reference timedemo saves and dump post-replay Rust entry.
//! Not part of the crate's public API surface; kept for serialization alignment.
use devilutionx_rs::engine::demo_reader::{load_save_archive, parse_demo, DemoEventType, DemoPayload, ReplayDriver};
use devilutionx_rs::game::codec::codec_decode;
use devilutionx_rs::game::game_loop::{adjust_cursor_for_walk, convert_screen_to_tile};
use devilutionx_rs::game::game_state::GameState;
use devilutionx_rs::game::loadsave::CppGameHeader;
use devilutionx_rs::game::pack::PlayerPack;
use devilutionx_rs::game::player_exact::{HeroClass, Player};
use rand::SeedableRng;

fn fixture(name: &str) -> String {
    format!("{}/../test/fixtures/timedemo/WarriorLevel1to2/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn dump_save(label: &str, path: &str, out: &str) {
    const PASSWORD: &str = "adslhfb1";
    let mut save = load_save_archive(path).expect("open save");
    let decoded = codec_decode(&save.read_entry("game").unwrap(), PASSWORD);
    std::fs::write(out, &decoded).expect("write decoded");
    let hdr = CppGameHeader::parse(&decoded).expect("header");
    println!("[{label}] len={} vx={} vy={} curr={} ltype={} amc={} aic={} amsc={} aoc={} setlevel={}",
        decoded.len(), hdr.view_position_x, hdr.view_position_y, hdr.currlevel, hdr.leveltype,
        hdr.active_monster_count, hdr.active_item_count, hdr.active_missile_count, hdr.active_object_count, hdr.setlevel);
}

fn main() {
    dump_save("RefPre", &fixture("spawn_0.sv"), r"E:\tmp\ref_pre.bin");
    dump_save("RefPost", &fixture("demo_0_reference_spawn_0.sv"), r"E:\tmp\ref_post.bin");

    // Replicate the e2e replay: load save, prep dungeon, replay demo, write v3.
    const PASSWORD: &str = "adslhfb1";
    let mut save = load_save_archive(&fixture("spawn_0.sv")).expect("open save");
    let hero = codec_decode(&save.read_entry("hero").unwrap(), PASSWORD);
    let pack = PlayerPack::from_bytes(&hero);
    let decoded = codec_decode(&save.read_entry("game").unwrap(), PASSWORD);
    let header = CppGameHeader::parse(&decoded).expect("game header parses");
    let seeds = CppGameHeader::parse_level_seeds(&decoded, 17).expect("seed table");
    devilutionx_rs::engine::random::seed_gameplay_rng(12345);
    let mut player = Player::new();
    player.init_class_stats();
    player._p_class = HeroClass::Warrior;
    let mut gs = GameState::new(player, false, 12345);
    gs.load_from_save(&pack, &header, &seeds, Some(&decoded));
    assert!(devilutionx_rs::game::game_loop::prepare_dungeon_for_replay(&mut gs, 1), "prep");
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let mut driver = ReplayDriver::new(parse_demo(&std::fs::read(fixture("demo_0.dmo")).unwrap()).unwrap());
    while let Some(ev) = driver.peek() {
        match ev.event_type {
            DemoEventType::MouseButtonDown => {
                if let DemoPayload::MouseButton { x, y, .. } = ev.payload {
                    let cam = gs.camera;
                    let (ax, ay) = devilutionx_rs::game::game_loop::adjust_cursor_for_walk(&gs, x as i32, y as i32);
                    gs.handle_click_tile(convert_screen_to_tile(ax, ay, cam.tile_x, cam.tile_y, 768, 480));
                }
            }
            DemoEventType::GameTick => {
                gs.update(&mut rng);
            }
            _ => {}
        }
        driver.step();
    }
    let actual = gs.write_save_game_v3();
    std::fs::write(r"E:\tmp\actual_post.bin", &actual).expect("write actual");
    println!("[ActualPost] len={} vx={} vy={}", actual.len(), gs.player.position.x, gs.player.position.y);
    let refb = std::fs::read(r"E:\tmp\ref_post.bin").unwrap();
    let n = actual.len().min(refb.len());
    let first = (0..n).find(|&i| actual[i] != refb[i]);
    println!("[ActualPost] first_diff={:?} (offset, actual, ref)", first.map(|i| (i, actual[i], refb[i])));
}
