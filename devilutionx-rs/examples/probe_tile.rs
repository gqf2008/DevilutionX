//! Decode one town tile and dump its RGBA to verify palette is applied.
use devilutionx_rs::engine::dungeon::{DungeonLevelData, DungeonType, TileDecoder, TileType};
use devilutionx_rs::engine::mpq::AssetManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = AssetManager::new();
    mpq.load_mpq("spawn.mpq", 0)?;
    let level = DungeonLevelData::load_from_asset_manager(&mut mpq, DungeonType::Town)?;
    println!("palette first 5 entries:");
    for i in 0..5 {
        let (r, g, b) = level.palette.get_rgb(i as u8);
        println!("  idx {}: ({},{},{})", i, r, g, b);
    }
    // Decode tile frame 1
    if let Some(rgba) = TileDecoder::decode_tile(&level.level_cel, 1, TileType::Square, &level.palette) {
        println!("\ntile frame 1: {} bytes", rgba.len());
        let mut count = 0;
        for i in 0..32 * 32 {
            let off = i * 4;
            if rgba[off] + rgba[off + 1] + rgba[off + 2] > 0 {
                println!("  pixel {}: R={} G={} B={} A={}", i, rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3]);
                count += 1;
                if count >= 10 { break; }
            }
        }
        let maxr = rgba.iter().step_by(4).copied().max().unwrap_or(0);
        let maxg = rgba.iter().skip(1).step_by(4).copied().max().unwrap_or(0);
        let maxb = rgba.iter().skip(2).step_by(4).copied().max().unwrap_or(0);
        println!("  max in tile RGBA: R={} G={} B={}", maxr, maxg, maxb);
    } else {
        println!("decode_tile returned None!");
    }
    Ok(())
}
