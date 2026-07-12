use devilutionx_rs::engine::mpq::AssetManager;
use devilutionx_rs::engine::dungeon::DungeonType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = AssetManager::new();
    mpq.load_mpq("spawn.mpq", 0)?;
    let min_data = mpq.read_file(r"levels\towndata\town.min")?;
    println!("town.min: {} bytes = {} u16 = {} megas", min_data.len(), min_data.len()/2, min_data.len()/32);
    // mega-tile 425 raw entries 12-15 (mapped to render blocks 0-3)
    let offset = 425 * 32;
    print!("mega[425] entries 12-15: ");
    for i in 12..16 {
        let off = offset + i * 2;
        let val = u16::from_le_bytes([min_data[off], min_data[off+1]]);
        print!("[{}]=0x{:04x}(f{},t{}) ", i, val, val & 0x0FFF, (val >> 12) & 7);
    }
    println!();
    // Check mega 0
    print!("mega[0] entries 12-15:   ");
    for i in 12..16 {
        let off = i * 2;
        let val = u16::from_le_bytes([min_data[off], min_data[off+1]]);
        print!("[{}]=0x{:04x} ", i, val);
    }
    println!();
    Ok(())
}
