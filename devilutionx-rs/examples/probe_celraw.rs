use devilutionx_rs::engine::mpq::AssetManager;
use devilutionx_rs::engine::dungeon::{DungeonLevelData, DungeonType, TileDecoder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = AssetManager::new();
    mpq.load_mpq("spawn.mpq", 0)?;
    let level = DungeonLevelData::load_from_asset_manager(&mut mpq, DungeonType::Town)?;
    for &idx in &[0usize, 1, 425, 426, 500, 1000] {
        if idx < level.min.mega_tiles.len() {
            let mt = &level.min.mega_tiles[idx];
            let b = &mt.blocks[0];
            if b.has_value() {
                if let Some(rgba) = TileDecoder::decode_tile(&level.level_cel, b.frame(), b.tile_type(), &level.palette) {
                    let maxr = rgba.iter().step_by(4).copied().max().unwrap_or(0);
                    println!("mega[{}] (dPiece={}): frame={} type={:?} maxR={}",
                        idx, idx + 1, b.frame(), b.tile_type(), maxr);
                }
            } else { println!("mega[{}]: empty", idx); }
        }
    }
    let mut best = (0u8, 0usize);
    for (mi, mt) in level.min.mega_tiles.iter().enumerate() {
        let b = &mt.blocks[0];
        if b.has_value() {
            if let Some(rgba) = TileDecoder::decode_tile(&level.level_cel, b.frame(), b.tile_type(), &level.palette) {
                let maxr = rgba.iter().step_by(4).copied().max().unwrap_or(0);
                if maxr > best.0 { best = (maxr, mi); }
            }
        }
    }
    println!("brightest mega[{}] maxR={}", best.1, best.0);
    Ok(())
}
