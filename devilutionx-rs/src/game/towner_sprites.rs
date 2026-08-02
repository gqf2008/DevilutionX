//! Towner sprite loading (C++ `LoadTownerAnimations`, towners.cpp:106-129).
//!
//! Loads the first frame of each NPC's CL2 animation from the MPQ archives
//! (`towners\<dir>\<prefix>.cl2`, the towners.tsv `animPath` + `.cl2`), keeping
//! the palette-indexed frame for palette-surface rendering (`clx_draw`) and an
//! RGBA copy. Missing assets fall back to the coloured-marker path in the
//! renderer.

use crate::engine::cl2_sheet::parse_cl2_sheet;
use crate::engine::mpq::MpqArchive;
use crate::game::towner::{TownerType, towner_anim_path, towner_anim_width};
use std::collections::HashMap;

/// A decoded towner sprite (CL2 frame 0 of the walk/stand animation).
#[derive(Debug, Clone)]
pub struct TownerSprite {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
    /// Palette-indexed CLX frame for `clx_draw` into the 8-bit backbuffer.
    pub frame: Option<crate::engine::clx_sprite::ClxSprite>,
}

/// Sprites indexed by `TownerType`, loaded once per town entry.
#[derive(Debug, Clone, Default)]
pub struct TownerSpriteSet {
    sprites: HashMap<TownerType, TownerSprite>,
}

impl TownerSpriteSet {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
        }
    }

    pub fn get(&self, towner_type: &TownerType) -> Option<&TownerSprite> {
        self.sprites.get(towner_type)
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// Bulk-load the first animation frame for each towner type in `types`.
    /// Types without a sprite (empty animPath / missing asset) are skipped;
    /// the renderer keeps the coloured-marker fallback for those.
    pub fn load(archive: &mut MpqArchive, types: &[TownerType]) -> Self {
        let mut sprites = HashMap::new();
        for ty in types {
            if sprites.contains_key(ty) {
                continue;
            }
            if let Some(sprite) = load_towner_sprite(archive, *ty) {
                sprites.insert(*ty, sprite);
            }
        }
        Self { sprites }
    }
}

/// Load a single towner's first animation frame.
///
/// Returns `None` (non-fatal) when the towner has no animPath or the asset
/// fails to decode; the caller falls back to the coloured marker.
pub fn load_towner_sprite(
    archive: &mut MpqArchive,
    towner_type: TownerType,
) -> Option<TownerSprite> {
    let anim = towner_anim_path(towner_type);
    if anim.is_empty() {
        return None;
    }
    let cl2_path = format!("{}.cl2", anim);
    let width = towner_anim_width(towner_type);

    // Palette: town.pal first (Tristram), then the dungeon palette fallback.
    let pal_data = archive
        .read_file("levels\\towndata\\town.pal")
        .or_else(|_| archive.read_file("levels\\l1data\\l1.pal"))
        .ok()?;
    if pal_data.len() < 768 {
        return None;
    }
    let mut palette = [0u8; 768];
    palette.copy_from_slice(&pal_data[..768]);

    let cl2_data = archive.read_file(&cl2_path).ok()?;
    let sheet = parse_cl2_sheet(&cl2_data, width)?;
    let frame = sheet.first_frame(0)?;

    let rgba = frame.decode_rgba(&palette);

    // Rebuild the frame as the Arc-backed `clx_sprite::ClxSprite` that
    // `clx_draw` consumes (header: [header_size][width][height] + RLE data).
    let mut clx_buf = Vec::with_capacity(6 + frame.pixel_data.len());
    clx_buf.extend_from_slice(&6u16.to_le_bytes());
    clx_buf.extend_from_slice(&frame.width.to_le_bytes());
    clx_buf.extend_from_slice(&frame.height.to_le_bytes());
    clx_buf.extend_from_slice(&frame.pixel_data);
    let clx_frame = crate::engine::clx_sprite::ClxSprite::new(
        std::sync::Arc::new(clx_buf),
        0,
        (6 + frame.pixel_data.len()) as u32,
    );

    Some(TownerSprite {
        width: frame.width,
        height: frame.height,
        rgba,
        frame: Some(clx_frame),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_towner_sprite_paths_from_tsv() {
        // towners.tsv animPath for the classic NPCs.
        assert_eq!(towner_anim_path(TownerType::Smith), "towners\\smith\\smithn");
        assert_eq!(towner_anim_path(TownerType::Witch), "towners\\townwmn1\\witch");
        assert_eq!(towner_anim_path(TownerType::PegBoy), "towners\\townboy\\pegkid1");
        assert_eq!(towner_anim_path(TownerType::Cow), "");
        assert_eq!(towner_anim_width(TownerType::Smith), 96);
    }

    /// Integration: load a real towner sprite from the MPQ and confirm it
    /// decodes to non-trivial pixels. Skipped when the MPQ is absent.
    #[test]
    fn load_real_smith_sprite() {
        use std::path::PathBuf;
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let candidates = [
            manifest.join("spawn.mpq"),
            manifest
                .parent()
                .unwrap()
                .join("devilutionx-rs")
                .join("spawn.mpq"),
        ];
        let mpq_path = match candidates.iter().find(|p| p.exists()) {
            Some(p) => p.clone(),
            None => {
                eprintln!("[towner_sprites] spawn.mpq not present; skipping");
                return;
            }
        };
        let mut archive = crate::engine::mpq::MpqArchive::open(&mpq_path).expect("open spawn.mpq");
        let sprite = load_towner_sprite(&mut archive, TownerType::Smith);
        let sprite = match sprite {
            Some(s) => s,
            None => {
                eprintln!("[towner_sprites] smith sprite unavailable; skipping");
                return;
            }
        };
        assert!(sprite.width > 0, "smith frame width computed");
        assert!(sprite.height > 0, "smith frame height computed");
        let opaque = sprite.rgba.chunks_exact(4).filter(|c| c[3] > 0).count();
        assert!(opaque > 0, "expected opaque pixels, got {}", opaque);
    }
}
