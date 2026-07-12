use devilutionx_rs::engine::mpq::AssetManager;
use devilutionx_rs::engine::dungeon::DungeonLevelData;
use devilutionx_rs::engine::dungeon::DungeonType;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = AssetManager::new();
    mpq.load_mpq("spawn.mpq", 0)?;
    let level = DungeonLevelData::load_from_asset_manager(&mut mpq, DungeonType::Town)?;
    let cel = &level.level_cel;
    // Check if first offset points right past the header
    let nf = u32::from_le_bytes([cel[0],cel[1],cel[2],cel[3]]);
    let expected_header_end = 4 + (nf as usize + 1) * 4;
    let off0 = u32::from_le_bytes([cel[4],cel[5],cel[6],cel[7]]) as usize;
    println!("nf={} header_end={} offset[0]={} match={}", nf, expected_header_end, off0, expected_header_end == off0);
    // What if offsets are RELATIVE to offset array start (not file start)?
    // Some CEL formats use relative offsets. Check if off0 == (nf+1)*4 (relative)
    let relative = (nf as usize + 1) * 4;
    println!("relative offset[0] would be {} actual={} match={}", relative, off0, relative == off0);
    Ok(())
}
