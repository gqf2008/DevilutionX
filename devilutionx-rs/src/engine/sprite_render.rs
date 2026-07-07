//! CLX/CEL Sprite → SDL Texture Rendering Bridge
//!
//! This module is the bridge between the palette-based RLE sprite formats
//! (CLX/CL2/CEL) used throughout Diablo's data and SDL2 hardware textures
//! that can be blitted to the screen.
//!
//! ## Pipeline
//! 1. A [`ClxSprite`] (owned, from `engine::clx`) holds RLE-encoded pixel data
//!    plus its `width`/`height`.
//! 2. [`clx_sprite_to_texture`] decodes the RLE data into an RGBA pixel buffer
//!    using a [`Palette`] (256 RGB colors). Palette index 0 is the colorkey
//!    (transparent) — those pixels get alpha = 0; all others get alpha = 255.
//! 3. The RGBA buffer is uploaded into a streaming SDL `Texture` (ABGR8888).
//!
//! The decode-to-RGBA step mirrors `ClxSprite::decode_rgba` from `clx.rs`, which
//! renders bottom-to-top (CL2 convention) and leaves colorkey pixels fully
//! transparent because the destination buffer is zero-initialised.
//!
//! C++ Reference: `Source/engine/render/clx_render.cpp` (`ClxDraw`) and the
//! PCX→Texture pattern in `main.rs::UiArtImage::to_texture`.

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;

use super::clx::ClxSprite;
use super::palette::{Color, Palette};

/// Error returned by the sprite→texture bridge.
#[derive(Debug)]
pub enum SpriteRenderError {
    /// SDL reported an error while creating or updating the texture.
    Sdl(String),
}

impl std::fmt::Display for SpriteRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpriteRenderError::Sdl(msg) => write!(f, "SDL texture error: {}", msg),
        }
    }
}

impl std::error::Error for SpriteRenderError {}

/// Convert a [`Palette`] (`colors: [Color; 256]`) into the flat 768-byte RGB
/// layout expected by `ClxSprite::decode_rgba`.
///
/// Public so other rendering code can reuse the same palette representation
/// when calling `decode_rgba` directly.
pub fn palette_to_rgb768(palette: &Palette) -> [u8; 768] {
    let mut out = [0u8; 768];
    for (i, c) in palette.colors.iter().enumerate() {
        out[i * 3] = c.r;
        out[i * 3 + 1] = c.g;
        out[i * 3 + 2] = c.b;
    }
    out
}

/// Decode a CLX sprite's RLE pixel data to an RGBA buffer using the given palette.
///
/// Index 0 is treated as transparent (colorkey): those pixels are left with
/// alpha = 0 (the buffer is zero-initialised, so skipped runs stay transparent).
/// All other indices are looked up in the palette and written with alpha = 255.
///
/// The returned buffer has length `width * height * 4` and is laid out
/// top-to-bottom, left-to-right (standard texture order), because `decode_rgba`
/// internally handles the CL2 bottom-to-top convention.
pub fn clx_sprite_to_rgba(sprite: &ClxSprite, palette: &Palette) -> Vec<u8> {
    let rgb = palette_to_rgb768(palette);
    sprite.decode_rgba(&rgb)
}

/// Convert a CLX sprite into a streaming SDL `Texture`.
///
/// This is the core of the CLX→SDL bridge (task R1). It:
/// 1. Decodes the RLE data with `clx_sprite_to_rgba` (index 0 → transparent).
/// 2. Creates a streaming ABGR8888 texture of the sprite's dimensions.
/// 3. Uploads the RGBA buffer (pitch = width * 4 bytes).
/// 4. Enables alpha blending so transparent pixels composite correctly.
///
/// # Returns
/// A textured owned by `creator`'s lifetime, or an error if SDL failed.
///
/// # Example
/// ```ignore
/// let texture = clx_sprite_to_texture(&canvas.texture_creator(), &sprite, &palette)?;
/// canvas.copy(&texture, None, Rect::new(x, y, sprite.width, sprite.height))?;
/// ```
pub fn clx_sprite_to_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    sprite: &ClxSprite,
    palette: &Palette,
) -> Result<Texture<'a>, SpriteRenderError> {
    let rgba = clx_sprite_to_rgba(sprite, palette);
    rgba_to_texture(creator, &rgba, sprite.width, sprite.height)
}

/// Upload a raw RGBA buffer (already laid out top-to-bottom) into a streaming
/// SDL texture. Shared by the CLX path and any other code that already has RGBA
/// pixels (e.g. `TileDecoder` output).
pub fn rgba_to_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    rgba: &[u8],
    width: u16,
    height: u16,
) -> Result<Texture<'a>, SpriteRenderError> {
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, width as u32, height as u32)
        .map_err(|e| SpriteRenderError::Sdl(e.to_string()))?;

    // Pitch is the number of bytes per row: width * 4 (RGBA).
    let pitch = (width as usize) * 4;
    texture
        .update(None, rgba, pitch)
        .map_err(|e| SpriteRenderError::Sdl(e.to_string()))?;

    // Enable alpha blending so index-0 (transparent) pixels composite over the
    // background instead of drawing as solid black.
    let _ = texture.set_blend_mode(BlendMode::Blend);

    Ok(texture)
}

/// Convenience: build a 1x1 solid-color texture from a [`Color`]. Useful for
/// debug fills / placeholder tiles without allocating a separate buffer path.
pub fn solid_color_texture<'a>(
    creator: &'a TextureCreator<WindowContext>,
    color: Color,
) -> Result<Texture<'a>, SpriteRenderError> {
    let rgba = [color.r, color.g, color.b, 255];
    rgba_to_texture(creator, &rgba, 1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a tiny CLX sprite by hand so we can assert the decode/bridge logic
    /// without depending on MPQ assets or SDL. The pixel data is CL2-style RLE:
    ///   - control byte 0x00..=0x7F: skip (transparent) that many pixels
    ///   - 0x80..=0xBE: fill (0xBF - control) pixels with the next byte
    ///   - 0xBF..=0xFF: copy (256 - control) literal pixels from the stream
    fn build_test_sprite(width: u16, height: u16, pixel_data: Vec<u8>) -> ClxSprite {
        ClxSprite {
            width,
            height,
            pixel_data,
        }
    }

    #[test]
    fn test_palette_to_rgb768_layout() {
        let mut palette = Palette::default();
        palette.colors[0] = Color::new(1, 2, 3);
        palette.colors[255] = Color::new(250, 251, 252);
        let rgb = palette_to_rgb768(&palette);
        assert_eq!(rgb[0..3], [1, 2, 3]);
        assert_eq!(rgb[255 * 3..255 * 3 + 3], [250, 251, 252]);
    }

    #[test]
    fn test_clx_sprite_to_rgba_transparent_index_zero() {
        // 2x2 sprite: all transparent (a single "skip 4" run = control 0x04).
        // decode_rgba renders bottom-to-top; a pure-skip run leaves the whole
        // buffer zeroed => fully transparent (alpha 0 everywhere), which is
        // exactly the colorkey behaviour we want for index 0.
        let sprite = build_test_sprite(2, 2, vec![0x04]);
        let palette = Palette::default();
        let rgba = clx_sprite_to_rgba(&sprite, &palette);
        assert_eq!(rgba.len(), 2 * 2 * 4);
        // Every pixel must be transparent black.
        for chunk in rgba.chunks_exact(4) {
            assert_eq!(chunk, [0, 0, 0, 0]);
        }
    }

    #[test]
    fn test_clx_sprite_to_rgba_fill_uses_palette() {
        // 2x1 sprite filled with palette index 5 via a fill command.
        // Fill command: 0xBF - count = control, so control 0xBE => fill 1 pixel.
        // We fill 2 pixels => control 0xBD (0xBF - 2), followed by color byte 5.
        let sprite = build_test_sprite(2, 1, vec![0xBD, 5]);
        let mut palette = Palette::default();
        palette.colors[5] = Color::new(10, 20, 30);
        let rgba = clx_sprite_to_rgba(&sprite, &palette);
        // decode_rgba starts at y = height-1 = 0 (bottom row) and writes both pixels.
        // Both pixels should be the palette color with full alpha.
        assert_eq!(&rgba[0..4], &[10, 20, 30, 255]);
        assert_eq!(&rgba[4..8], &[10, 20, 30, 255]);
    }

    #[test]
    fn test_clx_sprite_to_rgba_copy_command() {
        // 2x1 sprite via copy: control 0xFE => copy 2 literal pixels (256-0xFE=2).
        let sprite = build_test_sprite(2, 1, vec![0xFE, 7, 8]);
        let mut palette = Palette::default();
        palette.colors[7] = Color::new(100, 0, 0);
        palette.colors[8] = Color::new(0, 100, 0);
        let rgba = clx_sprite_to_rgba(&sprite, &palette);
        assert_eq!(&rgba[0..4], &[100, 0, 0, 255]);
        assert_eq!(&rgba[4..8], &[0, 100, 0, 255]);
    }

    #[test]
    fn test_clx_sprite_to_rgba_correct_size() {
        // A 3x4 sprite with all-transparent data must produce a 3*4*4 = 48 byte buffer.
        let sprite = build_test_sprite(3, 4, vec![0x0C]); // skip 12 pixels
        let palette = Palette::default();
        let rgba = clx_sprite_to_rgba(&sprite, &palette);
        assert_eq!(rgba.len(), 3 * 4 * 4);
    }
}
