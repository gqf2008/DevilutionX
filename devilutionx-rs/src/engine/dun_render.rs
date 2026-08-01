//! Dungeon Tile Rendering - level tile rendering functionality
//!
//! Ported from Source/engine/render/dun_render.hpp and dun_render.cpp

use crate::engine::types::Point;
use crate::engine::surface::Surface;

// Re-export from dungeon module
pub use crate::engine::dungeon::{TileType, TileProperties, LevelCelBlock};

/// Tile width in pixels
pub const TILE_WIDTH: i32 = 64;
/// Tile height in pixels
pub const TILE_HEIGHT: i32 = 32;

/// Width of a tile rendering primitive (half of TILE_WIDTH).
pub const DUN_FRAME_WIDTH: i32 = TILE_WIDTH / 2;  // 32

/// Height of a tile rendering primitive (except triangles).
pub const DUN_FRAME_HEIGHT: i32 = TILE_HEIGHT;    // 32

/// Height of triangle frames.
pub const DUN_FRAME_TRIANGLE_HEIGHT: i32 = 31;

/// Height of the lower triangle portion.
pub const LOWER_HEIGHT: i32 = DUN_FRAME_HEIGHT / 2;  // 16

/// Height of the upper triangle portion.
pub const TRIANGLE_UPPER_HEIGHT: i32 = DUN_FRAME_HEIGHT / 2 - 1;  // 15

/// Height of the upper trapezoid portion.
pub const TRAPEZOID_UPPER_HEIGHT: i32 = DUN_FRAME_HEIGHT / 2;  // 16

/// Size of re-encoded triangle frame data (after padding removal).
pub const REENCODED_TRIANGLE_FRAME_SIZE: usize = 544 - 32;

/// Size of re-encoded trapezoid frame data (after padding removal).
pub const REENCODED_TRAPEZOID_FRAME_SIZE: usize = 800 - 16;

/// For triangles, pixels drawn horizontally per pixel drawn vertically.
pub const X_STEP: i32 = 2;

/// Specifies the mask type to use for rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskType {
    /// The entire tile is opaque.
    Solid,
    /// The entire tile is blended with transparency.
    Transparent,
    /// Upper-right triangle is blended with transparency.
    Right,
    /// Upper-left triangle is blended with transparency.
    Left,
}

/// Clipping information for tile rendering.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clip {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
    pub width: i32,
    pub height: i32,
}

impl Clip {
    /// Calculate clipping parameters for a tile.
    pub fn calculate(x: i32, y: i32, w: i32, h: i32, out_w: i32, out_h: i32) -> Self {
        let top = if y + 1 < h { h - (y + 1) } else { 0 };
        let bottom = if y + 1 > out_h { (y + 1) - out_h } else { 0 };
        let left = if x < 0 { -x } else { 0 };
        let right = if x + w > out_w { x + w - out_w } else { 0 };

        Self {
            top,
            bottom,
            left,
            right,
            width: w - left - right,
            height: h - top - bottom,
        }
    }
}

/// Get the frame data offset from dungeon CEL data.
pub fn get_dun_frame_offset(dungeon_cel_data: &[u8], frame: u32) -> Option<usize> {
    if frame == 0 {
        return None;
    }

    // Frame table is at the start, each entry is a 32-bit little-endian offset
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

/// Get the raw frame data for a dungeon frame.
pub fn get_dun_frame(dungeon_cel_data: &[u8], frame: u32) -> Option<&[u8]> {
    let offset = get_dun_frame_offset(dungeon_cel_data, frame)?;
    Some(&dungeon_cel_data[offset..])
}

/// Render a line of opaque pixels.
#[inline]
pub fn render_line_opaque(
    dst: &mut [u8],
    src: &[u8],
    width: usize,
    light_table: Option<&[u8; 256]>,
) {
    if width == 0 || dst.len() < width || src.len() < width {
        return;
    }

    match light_table {
        Some(tbl) => {
            for i in 0..width {
                dst[i] = tbl[src[i] as usize];
            }
        }
        None => {
            dst[..width].copy_from_slice(&src[..width]);
        }
    }
}

/// Fill a line with a solid color.
#[inline]
pub fn fill_line(dst: &mut [u8], width: usize, color: u8) {
    if width <= dst.len() {
        dst[..width].fill(color);
    }
}

/// Render a 32x32 square tile (TileType::Square).
pub fn render_square(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let clip = Clip::calculate(
        position.x,
        position.y,
        DUN_FRAME_WIDTH,
        DUN_FRAME_HEIGHT,
        out.w(),
        out.h(),
    );

    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let mut src_offset = (clip.bottom * DUN_FRAME_WIDTH + clip.left) as usize;
    let dst_x = (position.x + clip.left).max(0) as usize;

    for i in 0..clip.height {
        let dst_y = position.y - i - clip.bottom;
        if dst_y >= 0 && dst_y < out.h() {
            if let Some(dst_row) = out.row_mut(dst_y) {
                let dst_end = (dst_x + clip.width as usize).min(dst_row.len());
                if dst_x < dst_end && src_offset + clip.width as usize <= src.len() {
                    let dst_slice = &mut dst_row[dst_x..dst_end];
                    let src_slice = &src[src_offset..src_offset + clip.width as usize];
                    render_line_opaque(dst_slice, src_slice, clip.width as usize, light_table);
                }
            }
        }
        src_offset += DUN_FRAME_WIDTH as usize;
    }
}

/// Render a transparent square tile (TileType::TransparentSquare).
///
/// `height` is the frame height in rows (32 for a full tile, 16 for the
/// foliage sprite that sits on the tile's upper half).
pub fn render_transparent_square(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
    _mask_type: MaskType,
    height: i32,
) {
    let out_w = out.w();
    let out_h = out.h();

    let clip = Clip::calculate(
        position.x,
        position.y,
        DUN_FRAME_WIDTH,
        height,
        out_w,
        out_h,
    );

    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let dst_x = (position.x + clip.left).max(0);
    let mut src_offset = 0usize;

    // Skip bottom clipped lines
    for _ in 0..clip.bottom {
        let mut remaining = DUN_FRAME_WIDTH;
        while remaining > 0 {
            if src_offset >= src.len() {
                return;
            }
            let v = src[src_offset] as i8;
            src_offset += 1;
            if v > 0 {
                src_offset += v as usize;
                remaining -= v as i32;
            } else {
                remaining -= (-v) as i32;
            }
        }
    }

    // Render visible lines
    for i in 0..clip.height {
        let dst_y = position.y - i - clip.bottom;
        let mut x = 0i32;

        while x < DUN_FRAME_WIDTH {
            if src_offset >= src.len() {
                return;
            }
            let v = src[src_offset] as i8;
            src_offset += 1;

            if v > 0 {
                let pixel_count = v as i32;
                let start_x = x;
                let end_x = x + pixel_count;

                let vis_start = (start_x - clip.left).max(0);
                let vis_end = (end_x - clip.left).min(clip.width);

                if vis_start < vis_end && dst_y >= 0 && dst_y < out_h {
                    let skip = (clip.left - start_x).max(0) as usize;
                    let width = (vis_end - vis_start) as usize;

                    if let Some(dst_row) = out.row_mut(dst_y) {
                        let dst_start = (dst_x + vis_start) as usize;
                        let dst_end = dst_start + width;

                        if dst_end <= dst_row.len() && src_offset + skip + width <= src.len() {
                            let dst_slice = &mut dst_row[dst_start..dst_end];
                            let src_slice = &src[src_offset + skip..src_offset + skip + width];
                            render_line_opaque(dst_slice, src_slice, width, light_table);
                        }
                    }
                }

                src_offset += pixel_count as usize;
                x = end_x;
            } else {
                x += (-v) as i32;
            }
        }
    }
}

/// Render a left-pointing triangle (TileType::LeftTriangle).
pub fn render_left_triangle(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let out_w = out.w();
    let out_h = out.h();

    let clip = Clip::calculate(
        position.x,
        position.y,
        DUN_FRAME_WIDTH,
        DUN_FRAME_TRIANGLE_HEIGHT,
        out_w,
        out_h,
    );

    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let mut src_offset = 0usize;

    // Lower triangle (rows 0-15, widths 2, 4, 6, ..., 32)
    for row in 0..LOWER_HEIGHT {
        let width = ((row + 1) * X_STEP) as usize;
        let dst_y = position.y - row;
        let x_offset = DUN_FRAME_WIDTH - width as i32;

        if dst_y >= 0 && dst_y < out_h && row >= clip.bottom && row < DUN_FRAME_TRIANGLE_HEIGHT - clip.top {
            let vis_start = (clip.left - x_offset).max(0) as usize;
            let vis_end = (clip.left + clip.width - x_offset).min(width as i32).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + x_offset + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += width;
    }

    // Upper triangle (rows 16-30, widths 30, 28, 26, ..., 2)
    for row in 0..TRIANGLE_UPPER_HEIGHT {
        let width = (DUN_FRAME_WIDTH - (row + 1) * X_STEP) as usize;
        let actual_row = LOWER_HEIGHT + row;
        let dst_y = position.y - actual_row;
        let x_offset = (row + 1) * X_STEP;

        if dst_y >= 0 && dst_y < out_h && actual_row >= clip.bottom && actual_row < DUN_FRAME_TRIANGLE_HEIGHT - clip.top {
            let vis_start = (clip.left - x_offset).max(0) as usize;
            let vis_end = (clip.left + clip.width - x_offset).min(width as i32).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + x_offset + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += width;
    }
}

/// Render a right-pointing triangle (TileType::RightTriangle).
pub fn render_right_triangle(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let out_w = out.w();
    let out_h = out.h();

    let clip = Clip::calculate(
        position.x,
        position.y,
        DUN_FRAME_WIDTH,
        DUN_FRAME_TRIANGLE_HEIGHT,
        out_w,
        out_h,
    );

    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let mut src_offset = 0usize;

    // Lower triangle
    for row in 0..LOWER_HEIGHT {
        let width = ((row + 1) * X_STEP) as usize;
        let dst_y = position.y - row;

        if dst_y >= 0 && dst_y < out_h && row >= clip.bottom && row < DUN_FRAME_TRIANGLE_HEIGHT - clip.top {
            let vis_start = clip.left.max(0) as usize;
            let vis_end = (clip.left + clip.width).min(width as i32).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += width;
    }

    // Upper triangle
    for row in 0..TRIANGLE_UPPER_HEIGHT {
        let width = (DUN_FRAME_WIDTH - (row + 1) * X_STEP) as usize;
        let actual_row = LOWER_HEIGHT + row;
        let dst_y = position.y - actual_row;

        if dst_y >= 0 && dst_y < out_h && actual_row >= clip.bottom && actual_row < DUN_FRAME_TRIANGLE_HEIGHT - clip.top {
            let vis_start = clip.left.max(0) as usize;
            let vis_end = (clip.left + clip.width).min(width as i32).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += width;
    }
}

/// Render a left-pointing trapezoid (TileType::LeftTrapezoid).
pub fn render_left_trapezoid(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let out_w = out.w();
    let out_h = out.h();

    let clip = Clip::calculate(
        position.x,
        position.y,
        DUN_FRAME_WIDTH,
        DUN_FRAME_HEIGHT,
        out_w,
        out_h,
    );

    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let mut src_offset = 0usize;

    // Lower triangle (rows 0-15)
    for row in 0..LOWER_HEIGHT {
        let width = ((row + 1) * X_STEP) as usize;
        let dst_y = position.y - row;
        let x_offset = DUN_FRAME_WIDTH - width as i32;

        if dst_y >= 0 && dst_y < out_h && row >= clip.bottom && row < DUN_FRAME_HEIGHT - clip.top {
            let vis_start = (clip.left - x_offset).max(0) as usize;
            let vis_end = (clip.left + clip.width - x_offset).min(width as i32).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + x_offset + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += width;
    }

    // Upper rectangle (rows 16-31)
    for row in 0..TRAPEZOID_UPPER_HEIGHT {
        let actual_row = LOWER_HEIGHT + row;
        let dst_y = position.y - actual_row;

        if dst_y >= 0 && dst_y < out_h && actual_row >= clip.bottom && actual_row < DUN_FRAME_HEIGHT - clip.top {
            let vis_start = clip.left.max(0) as usize;
            let vis_end = (clip.left + clip.width).min(DUN_FRAME_WIDTH).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += DUN_FRAME_WIDTH as usize;
    }
}

/// Render a right-pointing trapezoid (TileType::RightTrapezoid).
pub fn render_right_trapezoid(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
) {
    let out_w = out.w();
    let out_h = out.h();

    let clip = Clip::calculate(
        position.x,
        position.y,
        DUN_FRAME_WIDTH,
        DUN_FRAME_HEIGHT,
        out_w,
        out_h,
    );

    if clip.width <= 0 || clip.height <= 0 {
        return;
    }

    let mut src_offset = 0usize;

    // Lower triangle (rows 0-15)
    for row in 0..LOWER_HEIGHT {
        let width = ((row + 1) * X_STEP) as usize;
        let dst_y = position.y - row;

        if dst_y >= 0 && dst_y < out_h && row >= clip.bottom && row < DUN_FRAME_HEIGHT - clip.top {
            let vis_start = clip.left.max(0) as usize;
            let vis_end = (clip.left + clip.width).min(width as i32).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += width;
    }

    // Upper rectangle (rows 16-31)
    for row in 0..TRAPEZOID_UPPER_HEIGHT {
        let actual_row = LOWER_HEIGHT + row;
        let dst_y = position.y - actual_row;

        if dst_y >= 0 && dst_y < out_h && actual_row >= clip.bottom && actual_row < DUN_FRAME_HEIGHT - clip.top {
            let vis_start = clip.left.max(0) as usize;
            let vis_end = (clip.left + clip.width).min(DUN_FRAME_WIDTH).max(0) as usize;

            if vis_start < vis_end {
                if let Some(dst_row) = out.row_mut(dst_y) {
                    let dst_start = (position.x + vis_start as i32).max(0) as usize;
                    let dst_end = dst_start + (vis_end - vis_start);

                    if dst_end <= dst_row.len() && src_offset + vis_end <= src.len() {
                        let dst_slice = &mut dst_row[dst_start..dst_end];
                        let src_slice = &src[src_offset + vis_start..src_offset + vis_end];
                        render_line_opaque(dst_slice, src_slice, vis_end - vis_start, light_table);
                    }
                }
            }
        }

        src_offset += DUN_FRAME_WIDTH as usize;
    }
}

/// Render a tile frame based on its type.
pub fn render_tile_frame(
    out: &mut Surface,
    position: Point,
    tile_type: TileType,
    src: &[u8],
    light_table: Option<&[u8; 256]>,
    mask_type: MaskType,
) {
    match tile_type {
        TileType::Square => {
            render_square(out, position, src, light_table);
        }
        TileType::TransparentSquare => {
            render_transparent_square(out, position, src, light_table, mask_type, DUN_FRAME_HEIGHT);
        }
        TileType::LeftTriangle => {
            render_left_triangle(out, position, src, light_table);
        }
        TileType::RightTriangle => {
            render_right_triangle(out, position, src, light_table);
        }
        TileType::LeftTrapezoid => {
            render_left_trapezoid(out, position, src, light_table);
        }
        TileType::RightTrapezoid => {
            render_right_trapezoid(out, position, src, light_table);
        }
    }
}

/// Render a complete tile from LevelCelBlock.
pub fn render_tile(
    out: &mut Surface,
    position: Point,
    dungeon_cel_data: &[u8],
    level_cel_block: LevelCelBlock,
    mask_type: MaskType,
    light_table: Option<&[u8; 256]>,
) {
    if !level_cel_block.has_value() {
        return;
    }

    let tile_type = level_cel_block.tile_type();
    let frame = level_cel_block.frame();

    if let Some(src) = get_dun_frame(dungeon_cel_data, frame as u32) {
        render_tile_frame(out, position, tile_type, src, light_table, mask_type);
    }
}

/// C++ `GetDunFrameFoliage()` (dun_render.hpp:132): the foliage sprite of a
/// dungeon frame lives `ReencodedTriangleFrameSize` bytes into the frame.
pub fn get_dun_frame_foliage(dungeon_cel_data: &[u8], frame: u32) -> Option<&[u8]> {
    let frame_data = get_dun_frame(dungeon_cel_data, frame)?;
    let foliage_start = REENCODED_TRIANGLE_FRAME_SIZE.min(frame_data.len());
    Some(&frame_data[foliage_start..])
}

/// C++ `RenderTileFoliage()` (dun_render.hpp:162): renders the 16-pixel-tall
/// foliage sprite of a floor tile one tile-half above the floor line.
pub fn render_tile_foliage(
    out: &mut Surface,
    position: Point,
    dungeon_cel_data: &[u8],
    level_cel_block: LevelCelBlock,
    light_table: Option<&[u8; 256]>,
) {
    if !level_cel_block.has_value() {
        return;
    }
    let frame = level_cel_block.frame();
    let Some(src) = get_dun_frame_foliage(dungeon_cel_data, frame as u32) else {
        return;
    };
    render_transparent_square(
        out,
        Point::new(position.x, position.y - 16),
        src,
        light_table,
        MaskType::Solid,
        16,
    );
}

/// Draw a black diamond tile (64x31 pixels).
pub fn draw_black_tile(out: &mut Surface, x: i32, y: i32) {
    let out_w = out.w();
    let out_h = out.h();

    for row in 0..16 {
        let width = 2 + row * 4;
        let offset = 31 - row * 2;
        let dst_y = y - row;

        if dst_y >= 0 && dst_y < out_h {
            if let Some(dst_row) = out.row_mut(dst_y) {
                let start = (x + offset).max(0) as usize;
                let end = (x + offset + width).min(out_w) as usize;
                if start < end && end <= dst_row.len() {
                    dst_row[start..end].fill(0);
                }
            }
        }
    }

    for row in 0..15 {
        let width = 58 - row * 4;
        let offset = 3 + row * 2;
        let dst_y = y - 16 - row;

        if dst_y >= 0 && dst_y < out_h {
            if let Some(dst_row) = out.row_mut(dst_y) {
                let start = (x + offset).max(0) as usize;
                let end = (x + offset + width).min(out_w) as usize;
                if start < end && end <= dst_row.len() {
                    dst_row[start..end].fill(0);
                }
            }
        }
    }
}

/// Calculate the center offset for a sprite relative to a tile.
#[inline]
pub fn calculate_sprite_tile_center_x(width: i32) -> i32 {
    (width - TILE_WIDTH) / 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::surface::Surface;

    /// Build a level CEL with a single frame: 512-byte reencoded triangle
    /// frame followed by a 16-row foliage sprite (each row = RLE run of 32
    /// pixels of color 200).
    fn foliage_cel() -> Vec<u8> {
        let mut cel = Vec::new();
        cel.extend_from_slice(&1u32.to_le_bytes()); // num frames
        cel.extend_from_slice(&8u32.to_le_bytes()); // frame 1 offset
        cel.extend_from_slice(&vec![0u8; 512]); // reencoded triangle frame
        for _ in 0..16 {
            cel.push(32u8); // run of 32 pixels
            cel.extend_from_slice(&[200u8; 32]);
        }
        cel
    }

    #[test]
    fn test_get_dun_frame_foliage_offset() {
        let cel = foliage_cel();
        let fol = get_dun_frame_foliage(&cel, 1).expect("foliage present");
        assert_eq!(fol.len(), 16 * 33);
        assert_eq!(fol[0], 32);
        assert_eq!(fol[1], 200);
    }

    #[test]
    fn test_render_tile_foliage_above_floor_line() {
        let cel = foliage_cel();
        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        let block = LevelCelBlock::new(0x0001);
        render_tile_foliage(&mut surface, Point::new(100, 200), &cel, block, None);

        // The 16-row foliage sprite sits above the floor line: rows y 184..=169
        // (bottom-up RLE), x 100..=131.
        assert_eq!(surface.at(100, 184).copied(), Some(200), "bottom foliage row");
        assert_eq!(surface.at(131, 184).copied(), Some(200), "foliage row right edge");
        assert_eq!(surface.at(100, 169).copied(), Some(200), "top foliage row");
        // The floor line itself and the row below stay untouched.
        assert_eq!(surface.at(100, 200).copied(), Some(0));
        assert_eq!(surface.at(100, 168).copied(), Some(0));
    }
}

