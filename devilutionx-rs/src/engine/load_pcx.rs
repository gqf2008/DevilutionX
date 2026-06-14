//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! PCX sprite loading (via CLX)
//!
//! In UNPACKED_MPQS mode, PCX files are pre-converted to CLX format.
//! This module loads the CLX files and optionally loads the associated palette.

use sdl2::pixels::Color;

use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::load_file::load_optional_file_in_mem;
use crate::engine::load_clx::load_optional_clx;

/// Optional owned CLX sprite list
pub type OptionalOwnedClxSpriteList = Option<ClxSpriteList>;

/// PCX file extension (with UNPACKED_MPQS, we load .clx instead of .pcx)
pub const DEVILUTIONX_PCX_EXT: &str = ".clx";

/// Load a PCX sprite list (actually loads pre-converted CLX file)
///
/// # Arguments
/// * `filename` - Base filename without extension
/// * `_num_frames_or_frame_height` - Number of frames or frame height (unused with UNPACKED_MPQS)
/// * `_transparent_color` - Transparent color index (unused with UNPACKED_MPQS)  
/// * `out_palette` - Optional output palette buffer (256 colors)
/// * `log_error` - Whether to log errors for missing files
///
/// # Returns
/// The loaded CLX sprite list, or None if not found
pub fn load_pcx_sprite_list(
    filename: &str,
    _num_frames_or_frame_height: i32,
    _transparent_color: Option<u8>,
    out_palette: Option<&mut [Color; 256]>,
    log_error: bool,
) -> OptionalOwnedClxSpriteList {
    // With UNPACKED_MPQS, load the pre-converted .clx file
    let path = format!("{}{}", filename, DEVILUTIONX_PCX_EXT);
    
    let result = load_optional_clx(&path);
    if result.is_none() {
        if log_error {
            log::error!("Missing file: {}", path);
        }
        return None;
    }
    
    // Load palette if requested
    if let Some(palette) = out_palette {
        let pal_path = format!("{}.pal", filename);
        if let Some(pal_data) = load_optional_file_in_mem(&pal_path) {
            if pal_data.len() >= 256 * 3 {
                for i in 0..256 {
                    palette[i] = Color::RGBA(
                        pal_data[i * 3],
                        pal_data[i * 3 + 1],
                        pal_data[i * 3 + 2],
                        255,
                    );
                }
            }
        }
    }
    
    result
}

/// Load a PCX sprite list (convenience wrapper without palette)
pub fn load_pcx(
    filename: &str,
    num_frames_or_frame_height: i32,
    transparent_color: Option<u8>,
) -> OptionalOwnedClxSpriteList {
    load_pcx_sprite_list(filename, num_frames_or_frame_height, transparent_color, None, true)
}
