use devilutionx_rs::engine::dungeon::{DungeonLevelData, DungeonType, TileDecoder, TileType};
use devilutionx_rs::engine::mpq::AssetManager;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = AssetManager::new();
    mpq.load_mpq("spawn.mpq", 0)?;
    let level = DungeonLevelData::load_from_asset_manager(&mut mpq, DungeonType::Town)?;
    // CEL header: first u32 = frame count
    let nframes = if level.level_cel.len() >= 4 {
        u32::from_le_bytes([level.level_cel[0], level.level_cel[1], level.level_cel[2], level.level_cel[3]])
    } else { 0 };
    println!("town.cel header claims {} frames", nframes);
    for &f in &[1u16, 2, 5, 10, 20, 50, 100, 150, 200, 250, 300] {
        if let Some(rgba) = TileDecoder::decode_tile(&level.level_cel, f, TileType::Square, &level.palette) {
            let maxr = rgba.iter().step_by(4).copied().max().unwrap_or(0);
            let opaque = rgba.iter().skip(3).step_by(4).filter(|&&a| a > 0).count();
            println!("  frame {:4}: maxR={} opaque={}", f, maxr, opaque);
        } else {
            println!("  frame {:4}: None", f);
        }
    }
    Ok(())
}
