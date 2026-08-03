use devilutionx_rs::engine::demo_reader::load_save_archive;
use devilutionx_rs::game::codec::codec_decode;
use devilutionx_rs::game::loadsave::{CppGameHeader, LoadHelper, BinaryObjectData};
fn main() {
    let path = r"E:\Users\gxh\Documents\GitHub\DevilutionX\build_rs14\test\fixtures\timedemo\WarriorLevel1to2\spawn_0.sv";
    let mut save = load_save_archive(&path).expect("open save");
    let decoded = codec_decode(&save.read_entry("game").unwrap(), "adslhfb1");
    let hdr = CppGameHeader::parse(&decoded).expect("header");
    let count = hdr.active_monster_count as usize;
    let obj_ids = 23459 + 200*4 + count*216 + 250;
    let mut helper = LoadHelper::new(decoded[obj_ids + 254..].to_vec());
    let ocount = hdr.active_object_count as usize;
    let mut sarcs = Vec::new();
    for i in 0..ocount {
        let o = BinaryObjectData::from_binary(&mut helper);
        if o.object_type == 48 { sarcs.push((o.position_x, o.position_y)); }
    }
    println!("INITIAL save sarcs (placement order): {:?}", sarcs);
    // also print all object types/positions
    let mut helper2 = LoadHelper::new(decoded[obj_ids + 254..].to_vec());
    let mut all = Vec::new();
    for i in 0..ocount {
        let o = BinaryObjectData::from_binary(&mut helper2);
        all.push((o.object_type, o.position_x, o.position_y));
    }
    println!("INITIAL all objects ({}): {:?}", all.len(), all);
    println!("has 55,69: {}", sarcs.contains(&(55,69)));
}
