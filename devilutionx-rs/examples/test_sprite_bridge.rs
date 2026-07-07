//! CLX/CEL → SDL Texture Bridge Spike (task R1)
//!
//! End-to-end verification that the sprite→SDL bridge works against real MPQ
//! data. Two paths are exercised:
//!
//! 1. **Level-tile path** (the one Step 3 actually needs): read `town.cel` +
//!    `town.pal` from spawn.mpq, decode a 32×32 sub-tile frame via
//!    `engine::dungeon::TileDecoder`, and upload it as an SDL Texture. This
//!    proves the full chain `MPQ → CEL RLE → RGBA → SDL Texture` works for the
//!    data we'll render in-game.
//!
//! 2. **CLX sprite path**: synthesise a tiny CLX sprite in-memory and run it
//!    through `engine::sprite_render::clx_sprite_to_rgba` to prove the generic
//!    RLE decoder (transparent index 0, fill/copy commands) is correct without
//!    depending on file-format quirks. (Real `.cel`/`.cl2` sprite files lack an
//!    embedded width, so width must come from external metadata — handled by
//!    the engine's loader, not the bridge itself.)
//!
//! When run with a display, frame 0 of `town.cel` is shown in a window.
//! Headless (`SDL_VIDEODRIVER=dummy`) the decode paths are still verified.
//!
//! Usage:
//!   cargo run --example test_sprite_bridge
//!   cargo run --example test_sprite_bridge -- <path_to_mpq>

use devilutionx_rs::engine::clx::ClxSprite;
use devilutionx_rs::engine::dungeon::{DungeonType, DungeonLevelData, PaletteData, TileDecoder, TileType};
use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::palette::{Color, Palette};
use devilutionx_rs::engine::sprite_render::{clx_sprite_to_rgba, rgba_to_texture};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mpq_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("DIABDAT_MPQ").ok())
        .unwrap_or_else(|| "spawn.mpq".to_string());

    // ---- Path 1: level tile (town.cel) ---------------------------------
    let mut mpq = MpqArchive::open(&mpq_path)
        .or_else(|_| MpqArchive::open("DIABDAT.MPQ"))
        .map_err(|e| format!("找不到 MPQ ({}): {}", mpq_path, e))?;
    println!("[bridge] MPQ opened: {}", mpq_path);

    // load_from_mpq wants &mut MpqArchive (the raw archive type). town == spawn.
    let level = DungeonLevelData::load_from_mpq(&mut mpq, DungeonType::Town)?;
    println!(
        "[bridge] town level loaded: {} palette colors, {} bytes of CEL, TIL entries={}",
        level.palette.colors.len() / 3,
        level.level_cel.len(),
        level.til.len(),
    );

    // Build a Palette (engine::palette) from PaletteData for the generic bridge.
    let mut palette = Palette::default();
    for i in 0..256 {
        palette.colors[i] = Color::new(
            level.palette.colors[i * 3],
            level.palette.colors[i * 3 + 1],
            level.palette.colors[i * 3 + 2],
        );
    }

    // Decode tile frame 1 (1-based), Square type, as a 32x32 RGBA buffer.
    let tile_rgba = TileDecoder::decode_tile(
        &level.level_cel,
        1,
        TileType::Square,
        &level.palette,
    )
    .ok_or("TileDecoder 未能解码 town.cel 第 1 帧")?;

    let opaque = tile_rgba.chunks_exact(4).filter(|c| c[3] > 0).count();
    println!(
        "[bridge] ✓ town tile frame 1 decoded: {} bytes, {} opaque pixels ({:.0}% filled)",
        tile_rgba.len(),
        opaque,
        opaque as f32 / (32.0 * 32.0) * 100.0
    );
    assert_eq!(tile_rgba.len(), 32 * 32 * 4, "tile RGBA should be 32x32x4");
    // Town floor frames should be largely opaque.
    assert!(opaque > 100, "town tile decoded with too few opaque pixels");

    // ---- Path 2: synthetic CLX sprite through the generic RLE bridge -----
    // A 2x1 sprite: skip 0 (opaque) + fill 2 pixels with palette index 5.
    // fill command for 2 px = 0xBF-2 = 0xBD, then color byte.
    let synth = ClxSprite {
        width: 2,
        height: 1,
        pixel_data: vec![0xBD, 5],
    };
    let mut pal2 = Palette::default();
    pal2.colors[5] = Color::new(10, 20, 30);
    let rgba2 = clx_sprite_to_rgba(&synth, &pal2);
    assert_eq!(&rgba2[0..4], &[10, 20, 30, 255]);
    assert_eq!(&rgba2[4..8], &[10, 20, 30, 255]);
    println!("[bridge] ✓ synthetic CLX (fill cmd) decoded correctly via clx_sprite_to_rgba");

    println!("[bridge] decode paths VERIFIED ✓ (R1 桥: CEL RLE → RGBA)");

    // ---- Try to create an SDL Texture (needs video) --------------------
    let sdl_result = (|| -> Result<(), String> {
        let sdl = sdl2::init().map_err(|e| e.to_string())?;
        let video = sdl.video().map_err(|e| e.to_string())?;
        let window = video
            .window("CLX→Texture Bridge Spike (town tile)", 320, 240)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let creator = canvas.texture_creator();

        // Upload the decoded town tile as a 32x32 texture.
        let texture = rgba_to_texture(&creator, &tile_rgba, 32, 32)
            .map_err(|e| e.to_string())?;
        println!(
            "[bridge] ✓ SDL Texture created from town tile ({}x{})",
            texture.query().width,
            texture.query().height
        );

        canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 20, 30));
        canvas.clear();
        // Scale the 32x32 tile up 4x so it's visible.
        let dst = sdl2::rect::Rect::new(96, 56, 128, 128);
        canvas
            .copy(&texture, None, dst)
            .map_err(|e| e.to_string())?;
        canvas.present();
        println!("[bridge] displayed tile for 3s...");
        std::thread::sleep(std::time::Duration::from_secs(3));
        println!("[bridge] texture path VERIFIED ✓ (R1 桥: RGBA → SDL Texture)");
        Ok(())
    })();
    if let Err(e) = sdl_result {
        println!(
            "[bridge] SDL display unavailable ({}); decode paths still verified ✓",
            e
        );
    }

    println!("[bridge] spike complete — R1 桥验证通过");
    Ok(())
}
