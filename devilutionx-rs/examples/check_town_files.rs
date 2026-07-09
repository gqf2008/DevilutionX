//! Quick recon: which town files exist in spawn.mpq and does build_town_layout
//! produce a non-empty dPiece grid whose values map to valid MIN mega-tiles?
use devilutionx_rs::engine::dungeon::{DungeonLevelData, DungeonType, DunTemplate};
use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::game::game_loop::build_town_layout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = MpqArchive::open("spawn.mpq")
        .or_else(|_| MpqArchive::open("DIABDAT.MPQ"))?;
    println!("[recon] MPQ opened");

    let files = [
        "levels/towndata/sector1s.dun",
        "levels/towndata/sector2s.dun",
        "levels/towndata/sector3s.dun",
        "levels/towndata/sector4s.dun",
        "levels/towndata/town.til",
        "levels/towndata/town.min",
        "levels/towndata/town.cel",
        "levels/towndata/town.sol",
        "levels/towndata/town.pal",
    ];
    for f in files {
        let exists = mpq.has_file(f);
        let bytes = if exists { mpq.read_file(f).map(|d| d.len()).ok() } else { None };
        println!("  [{}] {} {:?}", if exists { "Y" } else { "n" }, f, bytes);
    }

    // Load town level data
    let level = DungeonLevelData::load_from_mpq(&mut mpq, DungeonType::Town)?;
    println!(
        "\n[recon] town data: {} MIN mega-tiles, {} TIL entries, {} bytes CEL",
        level.min.mega_tiles.len(),
        level.til.tiles.len(),
        level.level_cel.len()
    );

    // Build the town layout from the four sector templates.
    let s1 = read_dun(&mut mpq, "levels\\towndata\\sector1s.dun");
    let s2 = read_dun(&mut mpq, "levels\\towndata\\sector2s.dun");
    let s3 = read_dun(&mut mpq, "levels\\towndata\\sector3s.dun");
    let s4 = read_dun(&mut mpq, "levels\\towndata\\sector4s.dun");

    let layout = build_town_layout(&level, s1.as_ref(), s2.as_ref(), s3.as_ref(), s4.as_ref());

    // Sanity stats on the dPiece grid.
    let total = layout.d_piece.len();
    let non_zero = layout.d_piece.iter().filter(|&&v| v != 0).count();
    let distinct: std::collections::HashSet<u16> = layout.d_piece.iter().copied().collect();
    println!(
        "\n[layout] {}x{} = {} cells, {} non-zero, {} distinct dPiece values",
        layout.width, layout.height, total, non_zero, distinct.len()
    );

    // Sample the spawn position (75, 68) and a few neighbours.
    for &(x, y) in &[(75, 68), (50, 50), (0, 0), (10, 10), (84, 84)] {
        let v = layout.get(x, y);
        let in_min = (v as usize) < level.min.mega_tiles.len();
        println!(
            "  dPiece[{},{}] = {} {}",
            x, y, v,
            if in_min { "(in MIN range)" } else { "(OUT of MIN range!)" }
        );
    }

    // Verify a representative dPiece value decodes to a visible tile.
    let dp = layout.get(75, 68);
    if let Some(mega) = level.min.mega_tiles.get(dp as usize) {
        let b0 = &mega.blocks[0];
        println!(
            "\n[decode] MIN[{}] block0: frame={} type={:?} has_value={}",
            dp,
            b0.frame(),
            b0.tile_type(),
            b0.has_value()
        );
    }

    // Count how many grid cells have dPiece values that are out of MIN range.
    let oob = layout
        .d_piece
        .iter()
        .filter(|&&v| v != 0 && (v as usize) >= level.min.mega_tiles.len())
        .count();
    println!("\n[out-of-range] {} cells have dPiece >= MIN len ({})", oob, level.min.mega_tiles.len());

    Ok(())
}

fn read_dun(mpq: &mut MpqArchive, path: &str) -> Option<DunTemplate> {
    let data = mpq.read_file(path).ok()?;
    DunTemplate::from_bytes(&data)
}
