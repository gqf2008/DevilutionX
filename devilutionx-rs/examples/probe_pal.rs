//! Dump town.pal palette RGB values to check brightness.
use devilutionx_rs::engine::mpq::MpqArchive;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = MpqArchive::open("spawn.mpq")?;
    let pal = mpq.read_file(r"levels\towndata\town.pal")?;
    println!("town.pal: {} bytes", pal.len());
    // PAL is 256 RGB triples = 768 bytes
    for i in 0..10usize {
        let off = i*3;
        if off+2 < pal.len() {
            println!("  idx {}: R={} G={} B={}", i, pal[off], pal[off+1], pal[off+2]);
        }
    }
    // Also check some middle/high indices
    for &i in &[100, 128, 200, 255] {
        let off = i*3;
        if off+2 < pal.len() {
            println!("  idx {}: R={} G={} B={}", i, pal[off], pal[off+1], pal[off+2]);
        }
    }
    // Max value in palette?
    let maxv = pal.iter().take(768).copied().max().unwrap_or(0);
    println!("  max byte value in palette: {} (if <64, it's 6-bit; if 255, it's 8-bit)", maxv);
    Ok(())
}
