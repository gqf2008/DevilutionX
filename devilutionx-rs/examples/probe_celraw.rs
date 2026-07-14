use devilutionx_rs::engine::mpq::AssetManager;
use devilutionx_rs::engine::dungeon::DungeonLevelData;
use devilutionx_rs::engine::dungeon::DungeonType;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = AssetManager::new();
    mpq.load_mpq("spawn.mpq", 0)?;
    let level = DungeonLevelData::load_from_asset_manager(&mut mpq, DungeonType::Town)?;
    // Check palette entries 125-130
    for i in 125..135 {
        let (r,g,b) = level.palette.get_rgb(i as u8);
        println!("idx {}: ({},{},{})", i, r, g, b);
    }
    // Check the brightest entries
    let mut brightest = vec![];
    for i in 0..=255u8 {
        let (r,g,b) = level.palette.get_rgb(i);
        brightest.push((i, r as u16 + g as u16 + b as u16, r, g, b));
    }
    brightest.sort_by(|a,b| b.1.cmp(&a.1));
    println!("\nBrightest palette entries:");
    for (idx, sum, r, g, b) in brightest.iter().take(10) {
        println!("  idx {}: ({},{},{}) sum={}", idx, r, g, b, sum);
    }
    Ok(())
}
