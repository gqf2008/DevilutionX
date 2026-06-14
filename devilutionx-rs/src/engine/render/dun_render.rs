//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Dungeon Tile Rendering - level tile rendering functionality
//!
//! Ported from Source/engine/render/dun_render.hpp and dun_render.cpp

use crate::engine::Point;
use crate::engine::surface::Surface;
use super::light_render::{Lightmap, LIGHT_TABLE_SIZE, NUM_LIGHTING_LEVELS};
use super::primitive_render::get_palette_transparency_lookup;

// ============================================================================
// Constants - matching C++ dun_render.cpp
// ============================================================================

/// Tile width in pixels
pub const TILE_WIDTH: i32 = 64;
/// Tile height in pixels
pub const TILE_HEIGHT: i32 = 32;

/// Width of a tile rendering primitive (half of TILE_WIDTH).
pub const DUN_FRAME_WIDTH: i16 = 32;

/// Height of a tile rendering primitive (except triangles).
pub const DUN_FRAME_HEIGHT: i16 = 32;

/// Height of triangle frames.
pub const DUN_FRAME_TRIANGLE_HEIGHT: i16 = 31;

/// Height of the lower triangle portion.
const LOWER_HEIGHT: i16 = DUN_FRAME_HEIGHT / 2;  // 16

/// Height of the upper triangle of a triangular tile.
const TRIANGLE_UPPER_HEIGHT: i16 = DUN_FRAME_HEIGHT / 2 - 1;  // 15

/// Height of the upper rectangle of a trapezoid tile.
const TRAPEZOID_UPPER_HEIGHT: i16 = DUN_FRAME_HEIGHT / 2;  // 16

/// For triangles, pixels drawn horizontally per pixel drawn vertically.
const X_STEP: i16 = 2;

/// Size of re-encoded triangle frame data.
pub const REENCODED_TRIANGLE_FRAME_SIZE: usize = 544 - 32;

// ============================================================================
// Types - from levels/dun_tile.hpp
// ============================================================================

/// Level tile type - determines data encoding and shape
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TileType {
    /// 32x32 square, stored as array of pixels
    #[default]
    Square = 0,
    /// 32x32 square with transparency, RLE encoded
    TransparentSquare = 1,
    /// Left-pointing 32x31 triangle
    LeftTriangle = 2,
    /// Right-pointing 32x31 triangle
    RightTriangle = 3,
    /// Left-pointing 32x32 trapezoid
    LeftTrapezoid = 4,
    /// Right-pointing 32x32 trapezoid
    RightTrapezoid = 5,
}

impl TileType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => TileType::Square,
            1 => TileType::TransparentSquare,
            2 => TileType::LeftTriangle,
            3 => TileType::RightTriangle,
            4 => TileType::LeftTrapezoid,
            5 => TileType::RightTrapezoid,
            _ => TileType::Square,
        }
    }
}

/// Level CEL block - specifies MIN block of level CEL file
#[derive(Clone, Copy, Debug, Default)]
pub struct LevelCelBlock {
    pub data: u16,
}

impl LevelCelBlock {
    pub fn new(data: u16) -> Self {
        Self { data }
    }

    pub fn has_value(&self) -> bool {
        self.data != 0
    }

    /// Returns tile type (matching C++ LevelCelBlock::type())
    pub fn tile_type(&self) -> TileType {
        TileType::from_u8(((self.data & 0x7000) >> 12) as u8)
    }

    /// Returns 1-based frame index
    pub fn frame(&self) -> u16 {
        self.data & 0x0FFF
    }
}

/// Specifies the mask type to use for rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MaskType {
    /// The entire tile is opaque.
    Solid = 0,
    /// The entire tile is blended with transparency.
    Transparent = 1,
    /// Upper-right triangle is blended with transparency.
    Right = 2,
    /// Upper-left triangle is blended with transparency.
    Left = 3,
}

/// Clipping information for tile rendering.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clip {
    pub top: i16,
    pub bottom: i16,
    pub left: i16,
    pub right: i16,
    pub width: i16,
    pub height: i16,
}

/// Light type for rendering dispatch
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LightType {
    FullyDark,
    PartiallyLit,
    FullyLit,
    PerPixel,
}

// ============================================================================
// Internal helper functions
// ============================================================================

#[inline]
fn calculate_clip(x: i16, y: i16, w: i16, h: i16, out: &Surface) -> Clip {
    let out_w = out.w() as i16;
    let out_h = out.h() as i16;
    
    let top = if y + 1 < h { h - (y + 1) } else { 0 };
    let bottom = if y + 1 > out_h { (y + 1) - out_h } else { 0 };
    let left = if x < 0 { -x } else { 0 };
    let right = if x + w > out_w { x + w - out_w } else { 0 };

    Clip {
        top,
        bottom,
        left,
        right,
        width: w - left - right,
        height: h - top - bottom,
    }
}

/// Prefix increment values for mask types
#[inline]
fn prefix_increment(mask: MaskType) -> i8 {
    match mask {
        MaskType::Left => 2,
        MaskType::Right => -2,
        _ => 0,
    }
}

/// Initial prefix value for mask types
#[inline]
fn initial_prefix(mask: MaskType, y: i8) -> i8 {
    let inc = prefix_increment(mask);
    let init = if inc >= 0 { -32 } else { 64 };
    init + inc * y
}

// ============================================================================
// Line rendering functions
// ============================================================================

/// Render opaque line - fully dark (fill with 0)
#[inline]
fn render_line_opaque_dark(dst: &mut [u8], n: usize) {
    if n > 0 && dst.len() >= n {
        dst[..n].fill(0);
    }
}

/// Render opaque line - fully lit (copy directly)
#[inline]
fn render_line_opaque_lit(dst: &mut [u8], src: &[u8], n: usize) {
    if n > 0 && dst.len() >= n && src.len() >= n {
        dst[..n].copy_from_slice(&src[..n]);
    }
}

/// Render opaque line - with light table mapping
#[inline]
fn render_line_opaque_mapped(dst: &mut [u8], src: &[u8], n: usize, tbl: &[u8; 256]) {
    if n > 0 && dst.len() >= n && src.len() >= n {
        for i in 0..n {
            dst[i] = tbl[src[i] as usize];
        }
    }
}

/// Render opaque line - per pixel lighting
#[inline]
fn render_line_opaque_per_pixel(dst: &mut [u8], src: &[u8], n: usize, lightmap: &Lightmap, dst_offset: usize) {
    if n > 0 && dst.len() >= n && src.len() >= n {
        for i in 0..n {
            let light_level = lightmap.get_light_level_at_offset(dst_offset + i);
            dst[i] = lightmap.adjust_color(src[i], light_level);
        }
    }
}

/// Render transparent line - fully dark
#[inline]
fn render_line_transparent_dark(dst: &mut [u8], n: usize) {
    let lookup = get_palette_transparency_lookup();
    if n > 0 && dst.len() >= n {
        for i in 0..n {
            dst[i] = lookup[dst[i] as usize][0];
        }
    }
}

/// Render transparent line - fully lit
#[inline]
fn render_line_transparent_lit(dst: &mut [u8], src: &[u8], n: usize) {
    let lookup = get_palette_transparency_lookup();
    if n > 0 && dst.len() >= n && src.len() >= n {
        for i in 0..n {
            dst[i] = lookup[src[i] as usize][dst[i] as usize];
        }
    }
}

/// Render transparent line - with light table mapping
#[inline]
fn render_line_transparent_mapped(dst: &mut [u8], src: &[u8], n: usize, tbl: &[u8; 256]) {
    let lookup = get_palette_transparency_lookup();
    if n > 0 && dst.len() >= n && src.len() >= n {
        for i in 0..n {
            let mapped = tbl[src[i] as usize];
            dst[i] = lookup[dst[i] as usize][mapped as usize];
        }
    }
}

/// Render transparent line - per pixel lighting
#[inline]
fn render_line_transparent_per_pixel(dst: &mut [u8], src: &[u8], n: usize, lightmap: &Lightmap, dst_offset: usize) {
    let lookup = get_palette_transparency_lookup();
    if n > 0 && dst.len() >= n && src.len() >= n {
        for i in 0..n {
            let light_level = lightmap.get_light_level_at_offset(dst_offset + i);
            let lit_color = lightmap.adjust_color(src[i], light_level);
            dst[i] = lookup[dst[i] as usize][lit_color as usize];
        }
    }
}

// ============================================================================
// Dispatch helpers for light type
// ============================================================================

#[inline]
fn render_line_opaque(
    light: LightType, 
    dst: &mut [u8], 
    src: &[u8], 
    n: usize, 
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    dst_offset: usize,
) {
    match light {
        LightType::FullyDark => render_line_opaque_dark(dst, n),
        LightType::FullyLit => render_line_opaque_lit(dst, src, n),
        LightType::PartiallyLit => {
            if let Some(tbl) = tbl {
                render_line_opaque_mapped(dst, src, n, tbl);
            } else {
                render_line_opaque_lit(dst, src, n);
            }
        }
        LightType::PerPixel => render_line_opaque_per_pixel(dst, src, n, lightmap, dst_offset),
    }
}

#[inline]
fn render_line_transparent(
    light: LightType,
    dst: &mut [u8],
    src: &[u8],
    n: usize,
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    dst_offset: usize,
) {
    match light {
        LightType::FullyDark => render_line_transparent_dark(dst, n),
        LightType::FullyLit => render_line_transparent_lit(dst, src, n),
        LightType::PartiallyLit => {
            if let Some(tbl) = tbl {
                render_line_transparent_mapped(dst, src, n, tbl);
            } else {
                render_line_transparent_lit(dst, src, n);
            }
        }
        LightType::PerPixel => render_line_transparent_per_pixel(dst, src, n, lightmap, dst_offset),
    }
}

#[inline]
fn render_line_dispatch(
    light: LightType,
    mask: MaskType,
    dst: &mut [u8],
    src: &[u8],
    n: usize,
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    dst_offset: usize,
    prefix: i8,
) {
    match mask {
        MaskType::Solid => {
            render_line_opaque(light, dst, src, n, tbl, lightmap, dst_offset);
        }
        MaskType::Transparent => {
            render_line_transparent(light, dst, src, n, tbl, lightmap, dst_offset);
        }
        MaskType::Left | MaskType::Right => {
            let n_i8 = n as i8;
            if prefix >= n_i8 {
                // All opaque (Right) or all transparent (Left)
                if mask == MaskType::Right {
                    render_line_opaque(light, dst, src, n, tbl, lightmap, dst_offset);
                } else {
                    render_line_transparent(light, dst, src, n, tbl, lightmap, dst_offset);
                }
            } else if prefix <= 0 {
                // All transparent (Right) or all opaque (Left)
                if mask == MaskType::Left {
                    render_line_opaque(light, dst, src, n, tbl, lightmap, dst_offset);
                } else {
                    render_line_transparent(light, dst, src, n, tbl, lightmap, dst_offset);
                }
            } else {
                // Mixed - split at prefix
                let prefix_w = prefix as usize;
                if mask == MaskType::Right {
                    // Opaque first, then transparent
                    render_line_opaque(light, &mut dst[..prefix_w], &src[..prefix_w], prefix_w, tbl, lightmap, dst_offset);
                    render_line_transparent(light, &mut dst[prefix_w..], &src[prefix_w..], n - prefix_w, tbl, lightmap, dst_offset + prefix_w);
                } else {
                    // Transparent first, then opaque
                    render_line_transparent(light, &mut dst[..prefix_w], &src[..prefix_w], prefix_w, tbl, lightmap, dst_offset);
                    render_line_opaque(light, &mut dst[prefix_w..], &src[prefix_w..], n - prefix_w, tbl, lightmap, dst_offset + prefix_w);
                }
            }
        }
    }
}

// ============================================================================
// Square rendering
// ============================================================================

fn render_square_full(
    light: LightType,
    transparent: bool,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    mut dst_offset: usize,
) {
    let width = DUN_FRAME_WIDTH as usize;
    let mut src_offset = 0;
    
    for _ in 0..DUN_FRAME_HEIGHT {
        if transparent {
            render_line_transparent(light, &mut dst[dst_offset..], &src[src_offset..], width, tbl, lightmap, dst_offset);
        } else {
            render_line_opaque(light, &mut dst[dst_offset..], &src[src_offset..], width, tbl, lightmap, dst_offset);
        }
        src_offset += width;
        if dst_offset >= dst_pitch {
            dst_offset -= dst_pitch;
        } else {
            break;
        }
    }
}

fn render_square_clipped(
    light: LightType,
    transparent: bool,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    mut dst_offset: usize,
) {
    let width = DUN_FRAME_WIDTH as usize;
    let mut src_offset = clip.bottom as usize * width + clip.left as usize;
    let clip_width = clip.width as usize;
    
    for _ in 0..clip.height {
        if transparent {
            render_line_transparent(light, &mut dst[dst_offset..], &src[src_offset..], clip_width, tbl, lightmap, dst_offset);
        } else {
            render_line_opaque(light, &mut dst[dst_offset..], &src[src_offset..], clip_width, tbl, lightmap, dst_offset);
        }
        src_offset += width;
        if dst_offset >= dst_pitch {
            dst_offset -= dst_pitch;
        } else {
            break;
        }
    }
}

fn render_square_dispatch(
    light: LightType,
    transparent: bool,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    if clip.width == DUN_FRAME_WIDTH && clip.height == DUN_FRAME_HEIGHT {
        render_square_full(light, transparent, dst, dst_pitch, src, tbl, lightmap, dst_offset);
    } else {
        render_square_clipped(light, transparent, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
    }
}

// ============================================================================
// Transparent square rendering (RLE)
// ============================================================================

fn render_transparent_square_full(
    light: LightType,
    mask: MaskType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    height: i16,
    mut dst_offset: usize,
) {
    let width = DUN_FRAME_WIDTH as usize;
    let mut src_idx = 0usize;
    let mut prefix = initial_prefix(mask, 0);
    let inc = prefix_increment(mask);
    
    for _ in 0..height {
        let mut x = 0usize;
        let line_start = dst_offset;
        
        while x < width {
            if src_idx >= src.len() { return; }
            let v = src[src_idx] as i8;
            src_idx += 1;
            
            if v > 0 {
                let count = v as usize;
                if src_idx + count > src.len() { return; }
                
                let adjusted_prefix = prefix - (width - x) as i8 + width as i8;
                render_line_dispatch(
                    light, mask,
                    &mut dst[dst_offset + x..],
                    &src[src_idx..],
                    count, tbl, lightmap,
                    dst_offset + x,
                    adjusted_prefix,
                );
                src_idx += count;
                x += count;
            } else {
                x += (-v) as usize;
            }
        }
        
        prefix += inc;
        if line_start >= dst_pitch + width {
            dst_offset = line_start - dst_pitch;
        } else {
            break;
        }
    }
}

fn render_transparent_square_clipped(
    light: LightType,
    mask: MaskType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    mut dst_offset: usize,
) {
    let width = DUN_FRAME_WIDTH as usize;
    let mut src_idx = 0usize;
    
    // Skip bottom clipped lines
    for _ in 0..clip.bottom {
        let mut remaining = width as i16;
        while remaining > 0 {
            if src_idx >= src.len() { return; }
            let v = src[src_idx] as i8;
            src_idx += 1;
            if v > 0 {
                src_idx += v as usize;
                remaining -= v as i16;
            } else {
                remaining -= (-v) as i16;
            }
        }
    }
    
    let mut prefix = initial_prefix(mask, clip.bottom as i8);
    let inc = prefix_increment(mask);
    
    for _ in 0..clip.height {
        let mut x = 0i16;
        let mut remaining_left = clip.left;
        
        // Skip left clipped pixels
        while remaining_left > 0 {
            if src_idx >= src.len() { return; }
            let v = src[src_idx] as i8;
            src_idx += 1;
            
            if v > 0 {
                let count = v as i16;
                if count > remaining_left {
                    let overshoot = (count - remaining_left) as usize;
                    let skip = remaining_left as usize;
                    let adjusted_prefix = prefix as i16 - (DUN_FRAME_WIDTH - remaining_left);
                    render_line_dispatch(
                        light, mask,
                        &mut dst[dst_offset..],
                        &src[src_idx + skip..],
                        overshoot, tbl, lightmap,
                        dst_offset,
                        adjusted_prefix as i8,
                    );
                    dst_offset += overshoot;
                    x += overshoot as i16;
                }
                src_idx += count as usize;
                remaining_left -= count;
            } else {
                let skip = (-v) as i16;
                if skip > remaining_left {
                    let overshoot = (skip - remaining_left) as usize;
                    dst_offset += overshoot;
                    x += overshoot as i16;
                }
                remaining_left -= skip;
            }
        }
        
        // Draw visible portion
        let mut draw_width = clip.width - x;
        while draw_width > 0 {
            if src_idx >= src.len() { return; }
            let v = src[src_idx] as i8;
            src_idx += 1;
            
            if v > 0 {
                let count = v as i16;
                let actual = count.min(draw_width);
                let adjusted_prefix = prefix as i16 - (DUN_FRAME_WIDTH - draw_width);
                render_line_dispatch(
                    light, mask,
                    &mut dst[dst_offset..],
                    &src[src_idx..],
                    actual as usize, tbl, lightmap,
                    dst_offset,
                    adjusted_prefix as i8,
                );
                src_idx += count as usize;
                dst_offset += actual as usize;
                draw_width -= count;
            } else {
                let skip = (-v) as i16;
                let actual = skip.min(draw_width);
                dst_offset += actual as usize;
                draw_width -= skip;
            }
        }
        
        // Skip right clipped pixels
        let mut remaining_right = clip.right + draw_width.min(0);
        while remaining_right > 0 {
            if src_idx >= src.len() { return; }
            let v = src[src_idx] as i8;
            src_idx += 1;
            if v > 0 {
                src_idx += v as usize;
                remaining_right -= v as i16;
            } else {
                remaining_right -= (-v) as i16;
            }
        }
        
        prefix += inc;
        dst_offset = dst_offset.saturating_sub(dst_pitch).saturating_sub(clip.width as usize);
    }
}

// ============================================================================
// Triangle rendering helpers
// ============================================================================

fn calculate_triangle_source_skip_lower_bottom(num_lines: i16) -> usize {
    (X_STEP * num_lines * (num_lines + 1) / 2) as usize
}

fn calculate_triangle_source_skip_upper_bottom(num_lines: i16) -> usize {
    (2 * TRIANGLE_UPPER_HEIGHT * num_lines - num_lines * (num_lines - 1)) as usize
}

/// Vertical clip for diamond tile (L/R TRIANGLE)
struct DiamondClipY {
    lower_bottom: i16,
    lower_top: i16,
    upper_bottom: i16,
    upper_top: i16,
}

fn calculate_diamond_clip_y(clip: &Clip, upper_height: i16) -> DiamondClipY {
    if clip.bottom > LOWER_HEIGHT {
        DiamondClipY {
            lower_bottom: LOWER_HEIGHT,
            upper_bottom: clip.bottom - LOWER_HEIGHT,
            lower_top: 0,
            upper_top: 0,
        }
    } else if clip.top > upper_height {
        DiamondClipY {
            upper_top: upper_height,
            lower_top: clip.top - upper_height,
            upper_bottom: 0,
            lower_bottom: 0,
        }
    } else {
        DiamondClipY {
            upper_top: clip.top,
            lower_bottom: clip.bottom,
            lower_top: 0,
            upper_bottom: 0,
        }
    }
}

// ============================================================================
// Public API functions - matching C++ dun_render.hpp
// ============================================================================

/// Get the frame data offset from dungeon CEL data.
pub fn get_dun_frame_offset(dungeon_cel_data: &[u8], frame: u32) -> Option<usize> {
    if frame == 0 {
        return None;
    }

    let frame_table_offset = (frame as usize) * 4;
    if frame_table_offset + 4 > dungeon_cel_data.len() {
        return None;
    }

    let offset = u32::from_le_bytes([
        dungeon_cel_data[frame_table_offset],
        dungeon_cel_data[frame_table_offset + 1],
        dungeon_cel_data[frame_table_offset + 2],
        dungeon_cel_data[frame_table_offset + 3],
    ]) as usize;

    if offset < dungeon_cel_data.len() {
        Some(offset)
    } else {
        None
    }
}

/// Get the raw frame data for a dungeon frame (matching C++ GetDunFrame).
pub fn get_dun_frame(dungeon_cel_data: &[u8], frame: u32) -> Option<&[u8]> {
    let offset = get_dun_frame_offset(dungeon_cel_data, frame)?;
    Some(&dungeon_cel_data[offset..])
}

/// Get the foliage frame data (matching C++ GetDunFrameFoliage).
pub fn get_dun_frame_foliage(dungeon_cel_data: &[u8], frame: u32) -> Option<&[u8]> {
    let offset = get_dun_frame_offset(dungeon_cel_data, frame)?;
    let foliage_offset = offset + REENCODED_TRIANGLE_FRAME_SIZE;
    if foliage_offset < dungeon_cel_data.len() {
        Some(&dungeon_cel_data[foliage_offset..])
    } else {
        None
    }
}

/// Determine light type from lightmap and light table
fn determine_light_type(lightmap: &Lightmap, tbl: Option<&[u8; 256]>, per_pixel_lighting: bool) -> LightType {
    if per_pixel_lighting {
        LightType::PerPixel
    } else if let Some(tbl) = tbl {
        if lightmap.is_fully_dark_table(tbl) {
            LightType::FullyDark
        } else if lightmap.is_fully_lit_table(tbl) {
            LightType::FullyLit
        } else {
            LightType::PartiallyLit
        }
    } else {
        LightType::FullyLit
    }
}

/// Low-level tile rendering function (matching C++ RenderTileFrame)
pub fn render_tile_frame(
    out: &mut Surface,
    lightmap: &Lightmap,
    position: Point,
    tile: TileType,
    src: &[u8],
    height: i16,
    mask_type: MaskType,
    tbl: Option<&[u8; 256]>,
) {
    let clip = calculate_clip(position.x as i16, position.y as i16, DUN_FRAME_WIDTH, height, out);
    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let dst_x = (position.x + clip.left as i32).max(0) as usize;
    let dst_y = (position.y - clip.bottom as i32).max(0) as i32;
    
    if dst_y < 0 || dst_y >= out.h() {
        return;
    }
    
    let dst_pitch = out.pitch() as usize;
    let dst_offset = dst_y as usize * dst_pitch + dst_x;
    
    // TODO: Read per_pixel_lighting from options
    let per_pixel_lighting = false;
    let light = determine_light_type(lightmap, tbl, per_pixel_lighting);
    
    // Get mutable access to buffer
    let dst = &mut out.pixels[..];
    
    match mask_type {
        MaskType::Solid => {
            render_tile_type_dispatch(light, false, tile, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        MaskType::Transparent => {
            render_tile_type_dispatch(light, true, tile, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        MaskType::Left => {
            render_left_masked_dispatch(light, tile, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        MaskType::Right => {
            render_right_masked_dispatch(light, tile, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
    }
}

fn render_tile_type_dispatch(
    light: LightType,
    transparent: bool,
    tile: TileType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    match tile {
        TileType::Square => {
            render_square_dispatch(light, transparent, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        TileType::TransparentSquare => {
            let mask = if transparent { MaskType::Transparent } else { MaskType::Solid };
            if clip.width == DUN_FRAME_WIDTH && clip.bottom == 0 && clip.top == 0 {
                render_transparent_square_full(light, mask, dst, dst_pitch, src, tbl, lightmap, clip.height, dst_offset);
            } else {
                render_transparent_square_clipped(light, mask, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
            }
        }
        TileType::LeftTriangle => {
            render_left_triangle_dispatch(light, transparent, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        TileType::RightTriangle => {
            render_right_triangle_dispatch(light, transparent, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        TileType::LeftTrapezoid => {
            let mask = if transparent { MaskType::Transparent } else { MaskType::Solid };
            render_left_trapezoid_dispatch(light, mask, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        TileType::RightTrapezoid => {
            let mask = if transparent { MaskType::Transparent } else { MaskType::Solid };
            render_right_trapezoid_dispatch(light, mask, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
    }
}

fn render_left_masked_dispatch(
    light: LightType,
    tile: TileType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    match tile {
        TileType::TransparentSquare => {
            if clip.width == DUN_FRAME_WIDTH && clip.bottom == 0 && clip.top == 0 {
                render_transparent_square_full(light, MaskType::Left, dst, dst_pitch, src, tbl, lightmap, clip.height, dst_offset);
            } else {
                render_transparent_square_clipped(light, MaskType::Left, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
            }
        }
        TileType::LeftTrapezoid => {
            render_left_trapezoid_dispatch(light, MaskType::Left, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        _ => {}
    }
}

fn render_right_masked_dispatch(
    light: LightType,
    tile: TileType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    match tile {
        TileType::TransparentSquare => {
            if clip.width == DUN_FRAME_WIDTH && clip.bottom == 0 && clip.top == 0 {
                render_transparent_square_full(light, MaskType::Right, dst, dst_pitch, src, tbl, lightmap, clip.height, dst_offset);
            } else {
                render_transparent_square_clipped(light, MaskType::Right, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
            }
        }
        TileType::RightTrapezoid => {
            render_right_trapezoid_dispatch(light, MaskType::Right, dst, dst_pitch, src, tbl, lightmap, clip, dst_offset);
        }
        _ => {}
    }
}

// Triangle dispatch functions (simplified for now)
fn render_left_triangle_dispatch(
    light: LightType,
    transparent: bool,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    // Simplified: render lower and upper halves
    let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
    let mut src_offset = calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom);
    let mut current_dst = dst_offset + (X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as usize;
    
    // Lower triangle
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = (X_STEP * i) as usize;
        if transparent {
            render_line_transparent(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        } else {
            render_line_opaque(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        }
        src_offset += width;
        current_dst = current_dst.saturating_sub(dst_pitch + X_STEP as usize);
    }
    
    // Upper triangle
    src_offset += calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom);
    current_dst = dst_offset.saturating_sub(dst_pitch * LOWER_HEIGHT as usize) + (2 * X_STEP + X_STEP * clip_y.upper_bottom) as usize;
    
    let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
    for i in (1 + clip_y.upper_bottom)..=upper_max {
        let width = (DUN_FRAME_WIDTH - X_STEP * i) as usize;
        if transparent {
            render_line_transparent(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        } else {
            render_line_opaque(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        }
        src_offset += width;
        if current_dst >= dst_pitch {
            current_dst -= dst_pitch;
            current_dst += X_STEP as usize;
        }
    }
}

fn render_right_triangle_dispatch(
    light: LightType,
    transparent: bool,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
    let mut src_offset = calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom);
    let mut current_dst = dst_offset;
    
    // Lower triangle
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = (X_STEP * i) as usize;
        if transparent {
            render_line_transparent(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        } else {
            render_line_opaque(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        }
        src_offset += width;
        current_dst = current_dst.saturating_sub(dst_pitch);
    }
    
    // Upper triangle
    src_offset += calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom);
    
    let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
    for i in (1 + clip_y.upper_bottom)..=upper_max {
        let width = (DUN_FRAME_WIDTH - X_STEP * i) as usize;
        if transparent {
            render_line_transparent(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        } else {
            render_line_opaque(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        }
        src_offset += width;
        current_dst = current_dst.saturating_sub(dst_pitch);
    }
}

fn render_left_trapezoid_dispatch(
    light: LightType,
    mask: MaskType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    let transparent = mask == MaskType::Transparent;
    let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
    let mut src_offset = calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom);
    let mut current_dst = dst_offset + (X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as usize;
    
    // Lower triangle
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = (X_STEP * i) as usize;
        if transparent {
            render_line_transparent(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        } else {
            render_line_opaque(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        }
        src_offset += width;
        current_dst = current_dst.saturating_sub(dst_pitch + X_STEP as usize);
    }
    
    // Upper rectangle with mask
    src_offset = 272 + clip_y.upper_bottom as usize * DUN_FRAME_WIDTH as usize;
    current_dst = dst_offset.saturating_sub(dst_pitch * LOWER_HEIGHT as usize) + X_STEP as usize;
    
    let mut prefix = initial_prefix(mask, clip.bottom as i8);
    let inc = prefix_increment(mask);
    
    let upper_max = TRAPEZOID_UPPER_HEIGHT - clip_y.upper_top;
    for _ in (1 + clip_y.upper_bottom)..=upper_max {
        render_line_dispatch(light, mask, &mut dst[current_dst..], &src[src_offset..], 
            DUN_FRAME_WIDTH as usize, tbl, lightmap, current_dst, prefix);
        src_offset += DUN_FRAME_WIDTH as usize;
        prefix += inc;
        current_dst = current_dst.saturating_sub(dst_pitch);
    }
}

fn render_right_trapezoid_dispatch(
    light: LightType,
    mask: MaskType,
    dst: &mut [u8],
    dst_pitch: usize,
    src: &[u8],
    tbl: Option<&[u8; 256]>,
    lightmap: &Lightmap,
    clip: Clip,
    dst_offset: usize,
) {
    let transparent = mask == MaskType::Transparent;
    let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
    let mut src_offset = calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom);
    let mut current_dst = dst_offset;
    
    // Lower triangle
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = (X_STEP * i) as usize;
        if transparent {
            render_line_transparent(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        } else {
            render_line_opaque(light, &mut dst[current_dst..], &src[src_offset..], width.min(clip.width as usize), tbl, lightmap, current_dst);
        }
        src_offset += width;
        current_dst = current_dst.saturating_sub(dst_pitch);
    }
    
    // Upper rectangle with mask
    src_offset = 272 + clip_y.upper_bottom as usize * DUN_FRAME_WIDTH as usize;
    
    let mut prefix = initial_prefix(mask, clip.bottom as i8);
    let inc = prefix_increment(mask);
    
    let upper_max = TRAPEZOID_UPPER_HEIGHT - clip_y.upper_top;
    for _ in (1 + clip_y.upper_bottom)..=upper_max {
        render_line_dispatch(light, mask, &mut dst[current_dst..], &src[src_offset..],
            DUN_FRAME_WIDTH as usize, tbl, lightmap, current_dst, prefix);
        src_offset += DUN_FRAME_WIDTH as usize;
        prefix += inc;
        current_dst = current_dst.saturating_sub(dst_pitch);
    }
}

/// Render a complete tile from LevelCelBlock (matching C++ RenderTile inline).
pub fn render_tile(
    out: &mut Surface,
    lightmap: &Lightmap,
    position: Point,
    dungeon_cel_data: &[u8],
    level_cel_block: LevelCelBlock,
    mask_type: MaskType,
    tbl: Option<&[u8; 256]>,
) {
    if !level_cel_block.has_value() {
        return;
    }

    let tile_type = level_cel_block.tile_type();
    let frame = level_cel_block.frame();

    if let Some(src) = get_dun_frame(dungeon_cel_data, frame as u32) {
        let height = match tile_type {
            TileType::LeftTriangle | TileType::RightTriangle => DUN_FRAME_TRIANGLE_HEIGHT,
            _ => DUN_FRAME_HEIGHT,
        };
        render_tile_frame(out, lightmap, position, tile_type, src, height, mask_type, tbl);
    }
}

/// Render a floor foliage tile (matching C++ RenderTileFoliage inline).
pub fn render_tile_foliage(
    out: &mut Surface,
    lightmap: &Lightmap,
    position: Point,
    dungeon_cel_data: &[u8],
    level_cel_block: LevelCelBlock,
    tbl: Option<&[u8; 256]>,
) {
    if !level_cel_block.has_value() {
        return;
    }

    let frame = level_cel_block.frame();

    if let Some(src) = get_dun_frame_foliage(dungeon_cel_data, frame as u32) {
        let foliage_position = Point::new(position.x, position.y - 16);
        render_tile_frame(
            out, lightmap, foliage_position,
            TileType::TransparentSquare, src, 16,
            MaskType::Solid, tbl,
        );
    }
}

/// Draw a black diamond tile 64x31 (matching C++ world_draw_black_tile)
pub fn world_draw_black_tile(out: &mut Surface, sx: i32, sy: i32) {
    let out_w = out.w();
    let out_h = out.h();
    
    let clip_left = calculate_clip(sx as i16, sy as i16, DUN_FRAME_WIDTH, DUN_FRAME_TRIANGLE_HEIGHT, out);
    if clip_left.height <= 0 {
        return;
    }
    
    let right_left = if (sx + DUN_FRAME_WIDTH as i32) < 0 { 
        -(sx + DUN_FRAME_WIDTH as i32) as i16 
    } else { 
        0 
    };
    let right_right = if (sx + 2 * DUN_FRAME_WIDTH as i32) > out_w { 
        (sx + 2 * DUN_FRAME_WIDTH as i32 - out_w) as i16 
    } else { 
        0 
    };
    
    let clip_right = Clip {
        top: clip_left.top,
        bottom: clip_left.bottom,
        left: right_left,
        right: right_right,
        width: DUN_FRAME_WIDTH - right_left - right_right,
        height: clip_left.height,
    };
    
    let dst_pitch = out.pitch() as usize;
    let buf = &mut out.pixels[..];
    
    // Left triangle (dark)
    if clip_left.width > 0 {
        let dst_x = (sx + clip_left.left as i32).max(0) as usize;
        let dst_y = (sy - clip_left.bottom as i32).max(0) as i32;
        if dst_y >= 0 && dst_y < out_h {
            let dst_offset = dst_y as usize * dst_pitch + dst_x;
            render_left_triangle_dark(buf, dst_pitch, clip_left, dst_offset);
        }
    }
    
    // Right triangle (dark)
    if clip_right.width > 0 {
        let dst_x = (sx + DUN_FRAME_WIDTH as i32 + clip_right.left as i32).max(0) as usize;
        let dst_y = (sy - clip_right.bottom as i32).max(0) as i32;
        if dst_y >= 0 && dst_y < out_h {
            let dst_offset = dst_y as usize * dst_pitch + dst_x;
            render_right_triangle_dark(buf, dst_pitch, clip_right, dst_offset);
        }
    }
}

fn render_left_triangle_dark(dst: &mut [u8], dst_pitch: usize, clip: Clip, mut dst_offset: usize) {
    // Lower half
    let mut width = X_STEP as usize;
    for _ in 0..LOWER_HEIGHT {
        let actual_width = width.min(clip.width as usize);
        if dst_offset + actual_width <= dst.len() {
            dst[dst_offset..dst_offset + actual_width].fill(0);
        }
        width += X_STEP as usize;
        dst_offset = dst_offset.saturating_sub(dst_pitch + X_STEP as usize);
    }
    
    // Upper half
    width = (DUN_FRAME_WIDTH - X_STEP) as usize;
    for _ in 0..TRIANGLE_UPPER_HEIGHT {
        let actual_width = width.min(clip.width as usize);
        if dst_offset + actual_width <= dst.len() {
            dst[dst_offset..dst_offset + actual_width].fill(0);
        }
        width -= X_STEP as usize;
        if dst_offset >= dst_pitch {
            dst_offset -= dst_pitch;
            dst_offset += X_STEP as usize;
        }
    }
}

fn render_right_triangle_dark(dst: &mut [u8], dst_pitch: usize, clip: Clip, mut dst_offset: usize) {
    // Lower half
    let mut width = X_STEP as usize;
    for _ in 0..LOWER_HEIGHT {
        let actual_width = width.min(clip.width as usize);
        if dst_offset + actual_width <= dst.len() {
            dst[dst_offset..dst_offset + actual_width].fill(0);
        }
        width += X_STEP as usize;
        dst_offset = dst_offset.saturating_sub(dst_pitch);
    }
    
    // Upper half
    width = (DUN_FRAME_WIDTH - X_STEP) as usize;
    for _ in 0..TRIANGLE_UPPER_HEIGHT {
        let actual_width = width.min(clip.width as usize);
        if dst_offset + actual_width <= dst.len() {
            dst[dst_offset..dst_offset + actual_width].fill(0);
        }
        width -= X_STEP as usize;
        dst_offset = dst_offset.saturating_sub(dst_pitch);
    }
}

// ============================================================================
// Convenience wrappers (matching old API for compatibility)
// ============================================================================

/// Render a square tile (for backward compatibility)
// ============================================================================
// Static dummy data for backward compatibility functions
// ============================================================================

/// Static dummy output buffer for backward compatibility
static DUMMY_OUT_BUFFER: [u8; 1] = [0];

/// Static dummy lightmap buffer for backward compatibility
static DUMMY_LIGHTMAP_BUFFER: [u8; 1] = [0];

/// Static identity light tables for backward compatibility
static IDENTITY_LIGHT_TABLES: [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS] = {
    // Create identity tables where each color maps to itself
    let mut tables = [[0u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS];
    let mut level = 0;
    while level < NUM_LIGHTING_LEVELS {
        let mut i = 0;
        while i < LIGHT_TABLE_SIZE {
            tables[level][i] = i as u8;
            i += 1;
        }
        level += 1;
    }
    tables
};

/// Create a dummy lightmap for backward compatibility functions
fn create_dummy_lightmap<'a>() -> Lightmap<'a> {
    Lightmap::new(
        &DUMMY_OUT_BUFFER,
        1,
        &DUMMY_LIGHTMAP_BUFFER,
        1,
        &IDENTITY_LIGHT_TABLES,
        IDENTITY_LIGHT_TABLES[0].as_ptr(),
        IDENTITY_LIGHT_TABLES[NUM_LIGHTING_LEVELS - 1].as_ptr(),
    )
}

pub fn render_square(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let lightmap = create_dummy_lightmap();
    render_tile_frame(out, &lightmap, position, TileType::Square, src, DUN_FRAME_HEIGHT, MaskType::Solid, light_table);
}

/// Render a transparent square tile (for backward compatibility)
pub fn render_transparent_square(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
    mask_type: MaskType,
) {
    let lightmap = create_dummy_lightmap();
    render_tile_frame(out, &lightmap, position, TileType::TransparentSquare, src, DUN_FRAME_HEIGHT, mask_type, light_table);
}

/// Render a left-pointing triangle (for backward compatibility)
pub fn render_left_triangle(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let lightmap = create_dummy_lightmap();
    render_tile_frame(out, &lightmap, position, TileType::LeftTriangle, src, DUN_FRAME_TRIANGLE_HEIGHT, MaskType::Solid, light_table);
}

/// Render a right-pointing triangle (for backward compatibility)
pub fn render_right_triangle(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let lightmap = create_dummy_lightmap();
    render_tile_frame(out, &lightmap, position, TileType::RightTriangle, src, DUN_FRAME_TRIANGLE_HEIGHT, MaskType::Solid, light_table);
}

/// Draw a black tile (for backward compatibility)
pub fn draw_black_tile(out: &mut Surface, x: i32, y: i32) {
    world_draw_black_tile(out, x, y);
}
