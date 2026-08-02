//! Monster sprite loading + caching for dungeon rendering.
//!
//! Loads the per-monster-type "neutral" (stand) CL2 sprite from the game MPQ
//! archives, decodes direction 0 / frame 0 to RGBA (matching the player-sprite
//! path in `main.rs`), and exposes:
//!   * [`MonsterSprite`] — a decoded RGBA sprite ready for texture upload.
//!   * [`load_monster_sprites`] — bulk-load sprites for a set of monster types.
//!   * [`MonsterSpriteSet`] — indexed by `MonsterType`, used by the renderer.
//!
//! Spawn.mpq (shareware) monster CL2 layout follows the C++ convention:
//!   `monsters\<dirname>\<prefix>n.cl2` = neutral/stand (8 walk/stand dirs),
//!   where `<dirname>` and `<prefix>` come from `monstdat.tsv`'s
//!   `assetsSuffix`. Frame width is 128 for all monsters (per `monstdat.tsv`).
//!
//! If a monster's real sprite is unavailable (missing asset / parse failure),
//! the renderer falls back to a per-type coloured block (see
//! [`MonsterType::debug_color`]) so the monster still has a visible presence.
//!
//! This module is intentionally asset-read-only; it never touches SDL textures
//! (those are created lazily in the render thread, like the player sprite).

use crate::engine::cl2_sheet::parse_cl2_sheet;
use crate::engine::mpq::MpqArchive;
use crate::game::game_state::MonsterSprite;
use crate::game::monster::MonsterType;
use std::collections::HashMap;

/// Per-frame pixel width for monster CL2 sprites. All monsters in
/// `monstdat.tsv` use width 128.
pub const MONSTER_FRAME_WIDTH: u16 = 128;

/// MPQ path + palette for a monster type's stand sprite.
///
/// Returns `(cl2_path, palette_path)`. The palette is `levels\\l1data\\l1.pal`
/// for dungeon monsters (the L1 Cathedral palette); town uses `town.pal`. We
/// use `l1.pal` so dungeon monsters render with dungeon-appropriate colours.
fn sprite_asset_paths(monster_type: &MonsterType) -> Option<(&'static str, &'static str)> {
    // Map each `MonsterType` to its spawn.mpq CL2 folder + neutral prefix.
    // The `<prefix>n.cl2` file holds the 8-direction stand animation.
    let (dir, prefix) = match *monster_type {
        MonsterType::Zombie => ("zombie", "zombie"),
        MonsterType::FallenOne => ("falsword", "fall"),
        MonsterType::Skeleton => ("skelaxe", "sklax"),
        MonsterType::SkeletonArcher => ("skelbow", "sklbw"),
        MonsterType::Scavenger => ("scav", "scav"),
        MonsterType::Gargoyle | MonsterType::Balrog => ("bat", "bat"),
        MonsterType::Golem => ("golem", "golem"),
        // Types without a dedicated spawn.mpq sprite fall back to the block.
        _ => return None,
    };
    // Build the path at runtime to keep backslashes consistent on all platforms.
    let path: &'static str = match (dir, prefix) {
        ("zombie", "zombie") => "monsters\\zombie\\zombien.cl2",
        ("falsword", "fall") => "monsters\\falsword\\falln.cl2",
        ("skelaxe", "sklax") => "monsters\\skelaxe\\sklaxn.cl2",
        ("skelbow", "sklbw") => "monsters\\skelbow\\sklbwn.cl2",
        ("scav", "scav") => "monsters\\scav\\scavn.cl2",
        ("bat", "bat") => "monsters\\bat\\batn.cl2",
        ("golem", "golem") => "monsters\\golem\\golemn.cl2",
        _ => return None,
    };
    Some((path, "levels\\l1data\\l1.pal"))
}

/// Load a single monster type's stand sprite (direction 0, frame 0) as RGBA.
///
/// Returns `None` (non-fatal) if the asset is missing or fails to decode; the
/// caller then falls back to the coloured-block marker.
pub fn load_monster_sprite(archive: &mut MpqArchive, monster_type: &MonsterType) -> Option<MonsterSprite> {
    let (cl2_path, pal_path) = sprite_asset_paths(monster_type)?;

    // Palette: 768 bytes RGB. Try the dungeon palette, then fall back to the
    // town palette (shareware may only ship town.pal).
    let pal_data = archive
        .read_file(pal_path)
        .or_else(|_| archive.read_file("levels\\towndata\\town.pal"))
        .ok()?;
    if pal_data.len() < 768 {
        return None;
    }
    let mut palette = [0u8; 768];
    palette.copy_from_slice(&pal_data[..768]);

    let cl2_data = archive.read_file(cl2_path).ok()?;
    let sheet = parse_cl2_sheet(&cl2_data, MONSTER_FRAME_WIDTH)?;
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

    let sprite = MonsterSprite {
        width: frame.width,
        height: frame.height,
        rgba,
        // Keep the palette-indexed frame so the renderer can draw the monster
        // into the 8-bit backbuffer with the light table (C++ RenderCl2Sprite).
        frame: Some(clx_frame),
    };
    Some(sprite)
}

/// A set of decoded monster sprites indexed by `MonsterType`, plus a fast lookup
/// of which `MonsterType` each monster id maps to (for the renderer).
#[derive(Debug, Clone, Default)]
pub struct MonsterSpriteSet {
    sprites: HashMap<MonsterType, MonsterSprite>,
}

impl MonsterSpriteSet {
    /// Bulk-load stand sprites for the given monster types from the MPQ archive.
    /// Types whose sprites can't be loaded are silently skipped (renderer falls
    /// back to a coloured block for those).
    pub fn load(archive: &mut MpqArchive, types: &[MonsterType]) -> Self {
        let mut sprites = HashMap::new();
        for ty in types {
            if sprites.contains_key(ty) {
                continue;
            }
            match load_monster_sprite(archive, ty) {
                Some(s) => {
                    println!(
                        "[MonsterSprite] loaded {:?} {} ({}x{}, {} opaque px)",
                        ty,
                        sprite_asset_paths(ty).map(|(p, _)| p).unwrap_or("?"),
                        s.width,
                        s.height,
                        s.rgba.chunks_exact(4).filter(|c| c[3] > 0).count()
                    );
                    sprites.insert(*ty, s);
                }
                None => {
                    println!(
                        "[MonsterSprite] {:?} sprite unavailable; will use coloured block",
                        ty
                    );
                }
            }
        }
        Self { sprites }
    }

    /// Look up the decoded sprite for a monster type, if loaded.
    pub fn get(&self, monster_type: &MonsterType) -> Option<&MonsterSprite> {
        self.sprites.get(monster_type)
    }

    /// Number of successfully loaded sprites.
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// Whether any sprites were loaded.
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }
}

/// A stable RGB display colour per monster type, used for the coloured-block
/// fallback when no real sprite is available. Keeps each type visually distinct.
pub fn monster_display_color(monster_type: &MonsterType) -> (u8, u8, u8) {
    match *monster_type {
        MonsterType::Zombie => (110, 150, 90),   // sickly green
        MonsterType::FallenOne => (180, 70, 60), // red
        MonsterType::Skeleton => (220, 215, 190),// bone white
        MonsterType::SkeletonArcher => (180, 175, 150),
        MonsterType::Scavenger => (150, 110, 60),// brown
        MonsterType::Gargoyle => (120, 120, 140),
        MonsterType::Ghoul => (90, 120, 80),
        MonsterType::BlackKnight => (60, 60, 70),
        MonsterType::Overlord => (140, 60, 100),
        MonsterType::Golem => (170, 140, 90),
        MonsterType::FlayerDemon => (200, 90, 40),
        MonsterType::StormRider => (90, 120, 200),
        MonsterType::VenomSpitter => (80, 170, 90),
        MonsterType::SuccubusBlack => (170, 90, 150),
        MonsterType::Balrog => (200, 60, 40),
        MonsterType::VileOne => (120, 50, 130),
        MonsterType::MageHell => (90, 70, 170),
        MonsterType::Butcher => (170, 30, 30),
        MonsterType::SkeletonKing => (240, 230, 200),
        MonsterType::Lazarus => (140, 60, 180),
        MonsterType::Diablo => (220, 40, 30),
        // All remaining types get a stable colour derived from the C++
        // `_monster_id` so every monster stays visually distinct.
        other => {
            let idx = other as i16;
            let r = 90 + (idx.wrapping_mul(7).rem_euclid(120)) as u8;
            let g = 90 + (idx.wrapping_mul(13).rem_euclid(120)) as u8;
            let b = 90 + (idx.wrapping_mul(29).rem_euclid(120)) as u8;
            (r, g, b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_display_color_distinct() {
        // Each L1 monster type gets a non-black colour.
        for ty in [
            MonsterType::Zombie,
            MonsterType::FallenOne,
            MonsterType::Skeleton,
            MonsterType::SkeletonArcher,
            MonsterType::Scavenger,
        ] {
            let (r, g, b) = monster_display_color(&ty);
            assert!((r as u32 + g as u32 + b as u32) > 0, "{:?} colour should be non-black", ty);
        }
    }

    #[test]
    fn test_sprite_asset_paths_l1_monsters() {
        // All L1 monster types that should have spawn.mpq sprites resolve.
        assert_eq!(
            sprite_asset_paths(&MonsterType::Zombie),
            Some(("monsters\\zombie\\zombien.cl2", "levels\\l1data\\l1.pal"))
        );
        assert_eq!(
            sprite_asset_paths(&MonsterType::Skeleton),
            Some(("monsters\\skelaxe\\sklaxn.cl2", "levels\\l1data\\l1.pal"))
        );
        assert_eq!(
            sprite_asset_paths(&MonsterType::FallenOne),
            Some(("monsters\\falsword\\falln.cl2", "levels\\l1data\\l1.pal"))
        );
    }

    /// Integration: load the real Zombie stand sprite from spawn.mpq and confirm
    /// it decodes to a non-trivial RGBA buffer. Skipped if the MPQ is absent.
    #[test]
    fn load_real_zombie_sprite() {
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
                eprintln!("[monster_sprites] spawn.mpq not present; skipping");
                return;
            }
        };
        let mut archive = crate::engine::mpq::MpqArchive::open(&mpq_path).expect("open spawn.mpq");
        let sprite = load_monster_sprite(&mut archive, &MonsterType::Zombie)
            .expect("decode zombie stand sprite");
        assert_eq!(sprite.width, 128, "zombie frame width should be 128");
        assert!(sprite.height > 0, "zombie frame height computed");
        let opaque = sprite.rgba.chunks_exact(4).filter(|c| c[3] > 0).count();
        assert!(opaque > 0, "expected some opaque pixels, got {}", opaque);
    }
}
