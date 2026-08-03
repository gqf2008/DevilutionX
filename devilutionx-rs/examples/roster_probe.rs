use devilutionx_rs::game::codec::codec_decode;
use devilutionx_rs::game::game_state::GameState;
use devilutionx_rs::game::loadsave::CppGameHeader;
use devilutionx_rs::game::pack::PlayerPack;
use devilutionx_rs::game::player_exact::Player;
use devilutionx_rs::engine::demo_reader::load_save_archive;
fn main() {
    const PASSWORD: &str = "adslhfb1";
    let path = format!("{}/../test/fixtures/timedemo/WarriorLevel1to2/spawn_0.sv", env!("CARGO_MANIFEST_DIR"));
    let mut save = load_save_archive(&path).expect("open save");
    let hero = codec_decode(&save.read_entry("hero").unwrap(), PASSWORD);
    let pack = PlayerPack::from_bytes(&hero);
    let decoded = codec_decode(&save.read_entry("game").unwrap(), PASSWORD);
    let header = CppGameHeader::parse(&decoded).expect("header");
    let seeds = CppGameHeader::parse_level_seeds(&decoded, 17).expect("seeds");
    let mut gs = GameState::new(Player::new(), true, 12345);
    gs.load_from_save(&pack, &header, &seeds);
    let ok = devilutionx_rs::game::game_loop::prepare_dungeon_for_replay(&mut gs, 1);
    println!("[Probe] prep={ok} monsters={} objects={} numtrigs={} state={}", gs.monster_manager.active_count(), gs.objects.len(), gs.triggers.numtrigs, devilutionx_rs::engine::random::gameplay_rng_state());
    let sv = gs.write_save_game_v3();
    println!("[Probe] save_game_v3 len={}", sv.len());
    let out = std::env::var("PROBE_SAVE_OUT").unwrap_or_else(|_| "rust_game.bin".into());
    std::fs::write(&out, &sv).expect("write");
    for t in 0..gs.triggers.numtrigs {
        let p = gs.triggers.trigs[t].position;
        println!("  trig {t}: ({},{})", p.x, p.y);
    }
    println!("[Probe] themes={} state={}", gs.theme_manager.numthemes, devilutionx_rs::engine::random::gameplay_rng_state());
    for i in 0..gs.theme_manager.numthemes {
        let th = gs.theme_manager.themes[i];
        println!("  {i}: ttval={} ttype={:?}", th.ttval, th.ttype);
    }
    println!("[Monsters] count={}", gs.monster_manager.active_count());
    for i in 102..gs.monster_manager.active_count() {
        if let Some(m) = gs.monster_manager.get_monster(i) {
            println!("  THEME {i}: lt={} pos=({},{})", m.level_type, m.x, m.y);
        }
    }
    for i in 4..gs.monster_manager.active_count() {
        if let Some(m) = gs.monster_manager.get_monster(i) {
            println!("  {i}: lt={} pos=({},{})", m.level_type, m.x, m.y);
        }
    }
    println!("[Objs] count={}", gs.objects.len());
    for (i, o) in gs.objects.iter().enumerate() {
        println!("  {i}: ({},{}) type={}", o.position.x, o.position.y, o.otype as i32);
    }
}
