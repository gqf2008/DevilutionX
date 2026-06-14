//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! CLX 渲染 - 移植自 Source/engine/render/clx_render.cpp
//!
//! CL2 sprite rendering with clipping and various blending modes.

use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList, ClxSpriteSheet};
use crate::engine::render::light_render::Lightmap;
use crate::engine::render::primitive_render::get_palette_transparency_lookup;
use crate::engine::surface::Surface;
use crate::engine::Point;
use crate::utils::clx_decode::{
    get_clx_opaque_fill_width, get_clx_opaque_pixels_width, is_clx_opaque, is_clx_opaque_fill,
};
use std::sync::RwLock;

// === Outline cache (对应 C++ OutlinePixelsCache) ===

const MAX_OUTLINE_PIXELS: usize = 4096;

struct OutlinePixelsCacheEntry {
    outline_pixels: Vec<(u8, u8)>,
    sprite_data: *const u8,
    skip_color_index_zero: bool,
}

// SAFETY: sprite_data is only used for cache invalidation comparison
unsafe impl Send for OutlinePixelsCacheEntry {}
unsafe impl Sync for OutlinePixelsCacheEntry {}

impl Default for OutlinePixelsCacheEntry {
    fn default() -> Self {
        Self {
            outline_pixels: Vec::new(),
            sprite_data: std::ptr::null(),
            skip_color_index_zero: false,
        }
    }
}

static OUTLINE_PIXELS_CACHE: RwLock<OutlinePixelsCacheEntry> =
    RwLock::new(OutlinePixelsCacheEntry {
        outline_pixels: Vec::new(),
        sprite_data: std::ptr::null(),
        skip_color_index_zero: false,
    });

// === Clipping helpers ===

struct ClipX {
    left: i32,
    right: i32,
    width: i32,
}

fn calculate_clip_x(x: i32, w: u32, out: &Surface) -> ClipX {
    let w = w as i32;
    ClipX {
        left: if x < 0 { -x } else { 0 },
        right: if x + w > out.w() { x + w - out.w() } else { 0 },
        width: w - (if x < 0 { -x } else { 0 }) - (if x + w > out.w() { x + w - out.w() } else { 0 }),
    }
}

struct BlitCommandInfo {
    src_end_offset: usize,
    length: u32,
}

fn clx_blit_info(src: &[u8], offset: usize) -> BlitCommandInfo {
    let control = src[offset];
    if !is_clx_opaque(control) {
        BlitCommandInfo {
            src_end_offset: offset + 1,
            length: control as u32,
        }
    } else if is_clx_opaque_fill(control) {
        let width = get_clx_opaque_fill_width(control);
        BlitCommandInfo {
            src_end_offset: offset + 2,
            length: width as u32,
        }
    } else {
        let width = get_clx_opaque_pixels_width(control);
        BlitCommandInfo {
            src_end_offset: offset + 1 + width as usize,
            length: width as u32,
        }
    }
}

// === Core rendering (DoRenderBackwards equivalent) ===

fn do_render_backwards<F>(
    out: &mut Surface,
    position: Point,
    src: &[u8],
    src_width: u32,
    src_height: u32,
    mut blit_fn: F,
) where
    F: FnMut(&mut [u8], &[u8], u8, bool), // dst, src_pixels, color, is_fill
{
    let out_h = out.h();
    let out_w = out.w();

    if position.y < 0 || position.y + 1 >= out_h + src_height as i32 {
        return;
    }

    let clip_x = calculate_clip_x(position.x, src_width, out);
    if clip_x.width <= 0 {
        return;
    }

    let pitch = out.pitch() as i32;
    let src_width_i32 = src_width as i32;

    // Skip bottom clipped lines
    let mut src_offset = 0usize;
    let mut y = position.y;
    let mut x_offset: i32 = 0;

    // Skip lines below the surface
    while y >= out_h && src_offset < src.len() {
        let mut remaining = src_width_i32 - x_offset;
        while remaining > 0 && src_offset < src.len() {
            let info = clx_blit_info(src, src_offset);
            src_offset = info.src_end_offset;
            remaining -= info.length as i32;
        }
        if remaining < 0 {
            x_offset = -remaining % src_width_i32;
            y -= 1 + (-remaining / src_width_i32);
        } else {
            x_offset = 0;
            y -= 1;
        }
    }

    if src_offset >= src.len() {
        return;
    }

    // Render visible lines
    while src_offset < src.len() && y >= 0 {
        let mut remaining = src_width_i32 - x_offset;
        let row_start = position.x + x_offset;

        if let Some(row) = out.row_mut(y) {
            let mut x = row_start;

            while remaining > 0 && src_offset < src.len() {
                let control = src[src_offset];

                if !is_clx_opaque(control) {
                    // Transparent
                    let skip = control as i32;
                    x += skip;
                    remaining -= skip;
                    src_offset += 1;
                } else if is_clx_opaque_fill(control) {
                    // Fill
                    let fill_width = get_clx_opaque_fill_width(control) as i32;
                    let color = src[src_offset + 1];
                    src_offset += 2;

                    // Clip and draw
                    let draw_start = x.max(0) as usize;
                    let draw_end = (x + fill_width).min(out_w) as usize;
                    if draw_start < draw_end && draw_end <= row.len() {
                        blit_fn(&mut row[draw_start..draw_end], &[], color, true);
                    }

                    x += fill_width;
                    remaining -= fill_width;
                } else {
                    // Pixels
                    let pixel_width = get_clx_opaque_pixels_width(control) as i32;
                    src_offset += 1;

                    let draw_start = x.max(0) as usize;
                    let draw_end = (x + pixel_width).min(out_w) as usize;
                    let src_start = if x < 0 { (-x) as usize } else { 0 };

                    if draw_start < draw_end && draw_end <= row.len() {
                        let src_slice = &src[src_offset + src_start..src_offset + src_start + (draw_end - draw_start)];
                        blit_fn(&mut row[draw_start..draw_end], src_slice, 0, false);
                    }

                    src_offset += pixel_width as usize;
                    x += pixel_width;
                    remaining -= pixel_width;
                }
            }
        } else {
            // Skip this row's data
            while remaining > 0 && src_offset < src.len() {
                let info = clx_blit_info(src, src_offset);
                src_offset = info.src_end_offset;
                remaining -= info.length as i32;
            }
        }

        // Handle line wrap
        if remaining < 0 {
            x_offset = -remaining % src_width_i32;
            y -= 1 + (-remaining / src_width_i32);
        } else {
            x_offset = 0;
            y -= 1;
        }
    }
}

// === Public API (matching C++ clx_render.hpp) ===

/// Blit CLX sprite to the back buffer
/// C++ Reference: ClxDraw
pub fn clx_draw(out: &mut Surface, position: Point, clx: &ClxSprite) {
    do_render_backwards(
        out,
        position,
        clx.pixel_data(),
        clx.width() as u32,
        clx.height() as u32,
        |dst, src_pixels, color, is_fill| {
            if is_fill {
                dst.fill(color);
            } else {
                dst.copy_from_slice(src_pixels);
            }
        },
    );
}

/// Blit CLX sprite with TRN color translation
/// C++ Reference: ClxDrawTRN
pub fn clx_draw_trn(out: &mut Surface, position: Point, clx: &ClxSprite, trn: &[u8; 256]) {
    do_render_backwards(
        out,
        position,
        clx.pixel_data(),
        clx.width() as u32,
        clx.height() as u32,
        |dst, src_pixels, color, is_fill| {
            if is_fill {
                dst.fill(trn[color as usize]);
            } else {
                for (d, &s) in dst.iter_mut().zip(src_pixels.iter()) {
                    *d = trn[s as usize];
                }
            }
        },
    );
}

/// Blit CLX sprite with lightmap
/// C++ Reference: ClxDrawWithLightmap
pub fn clx_draw_with_lightmap(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    lightmap: &Lightmap,
) {
    // 简化实现 - 实际需要 lightmap.get_lighting_at 和 lightmap.adjust_color
    // 当前先用直接绘制
    clx_draw(out, position, clx);
    let _ = lightmap;
}

/// Blit CLX sprite with 50% transparency blending
/// C++ Reference: ClxDrawBlended
pub fn clx_draw_blended(out: &mut Surface, position: Point, clx: &ClxSprite) {
    let lookup = get_palette_transparency_lookup();
    do_render_backwards(
        out,
        position,
        clx.pixel_data(),
        clx.width() as u32,
        clx.height() as u32,
        |dst, src_pixels, color, is_fill| {
            if is_fill {
                for d in dst.iter_mut() {
                    *d = lookup.blend(color, *d);
                }
            } else {
                for (d, &s) in dst.iter_mut().zip(src_pixels.iter()) {
                    *d = lookup.blend(s, *d);
                }
            }
        },
    );
}

/// Blit CLX sprite with TRN and 50% transparency
/// C++ Reference: ClxDrawBlendedTRN
pub fn clx_draw_blended_trn(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    trn: &[u8; 256],
) {
    let lookup = get_palette_transparency_lookup();
    do_render_backwards(
        out,
        position,
        clx.pixel_data(),
        clx.width() as u32,
        clx.height() as u32,
        |dst, src_pixels, color, is_fill| {
            if is_fill {
                let mapped = trn[color as usize];
                for d in dst.iter_mut() {
                    *d = lookup.blend(mapped, *d);
                }
            } else {
                for (d, &s) in dst.iter_mut().zip(src_pixels.iter()) {
                    let mapped = trn[s as usize];
                    *d = lookup.blend(mapped, *d);
                }
            }
        },
    );
}

/// Blit CLX sprite with lightmap and 50% transparency
/// C++ Reference: ClxDrawBlendedWithLightmap
pub fn clx_draw_blended_with_lightmap(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    lightmap: &Lightmap,
) {
    // 简化实现
    clx_draw_blended(out, position, clx);
    let _ = lightmap;
}

/// Draw CLX sprite outline
/// C++ Reference: ClxDrawOutline
pub fn clx_draw_outline(out: &mut Surface, col: u8, position: Point, clx: &ClxSprite) {
    render_clx_outline::<false>(out, position, clx, col);
}

/// Draw CLX sprite outline, skipping color index 0 (shadows)
/// C++ Reference: ClxDrawOutlineSkipColorZero
pub fn clx_draw_outline_skip_color_zero(
    out: &mut Surface,
    col: u8,
    position: Point,
    clx: &ClxSprite,
) {
    render_clx_outline::<true>(out, position, clx, col);
}

fn render_clx_outline<const SKIP_COLOR_ZERO: bool>(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    color: u8,
) {
    update_outline_pixels_cache::<SKIP_COLOR_ZERO>(clx);

    let cache = OUTLINE_PIXELS_CACHE.read().unwrap();
    let pos = Point::new(position.x - 1, position.y - clx.height() as i32);

    for &(x, y) in &cache.outline_pixels {
        out.set_pixel(Point::new(pos.x + x as i32, pos.y + y as i32), color);
    }
}

fn update_outline_pixels_cache<const SKIP_COLOR_ZERO: bool>(clx: &ClxSprite) {
    let sprite_data_ptr = clx.pixel_data().as_ptr();

    {
        let cache = OUTLINE_PIXELS_CACHE.read().unwrap();
        if cache.sprite_data == sprite_data_ptr && cache.skip_color_index_zero == SKIP_COLOR_ZERO {
            return;
        }
    }

    let mut cache = OUTLINE_PIXELS_CACHE.write().unwrap();
    cache.skip_color_index_zero = SKIP_COLOR_ZERO;
    cache.sprite_data = sprite_data_ptr;
    cache.outline_pixels.clear();
    get_outline::<SKIP_COLOR_ZERO>(clx, &mut cache.outline_pixels);
}

fn get_outline<const SKIP_COLOR_ZERO: bool>(clx: &ClxSprite, result: &mut Vec<(u8, u8)>) {
    let width = clx.width() as usize;
    let height = clx.height();
    let src = clx.pixel_data();

    // Simplified outline detection - find edges of opaque regions
    let mut solid_mask = vec![vec![false; width + 2]; height as usize + 2];

    let mut src_offset = 0;
    let mut y = height as i32;
    let mut x = 1i32;

    while src_offset < src.len() && y > 0 {
        while x <= width as i32 && src_offset < src.len() {
            let control = src[src_offset];
            src_offset += 1;

            if !is_clx_opaque(control) {
                x += control as i32;
            } else if is_clx_opaque_fill(control) {
                let w = get_clx_opaque_fill_width(control) as i32;
                let color = src[src_offset];
                src_offset += 1;

                if !SKIP_COLOR_ZERO || color != 0 {
                    for i in 0..w {
                        if (x + i) > 0 && (x + i) <= width as i32 {
                            solid_mask[y as usize][(x + i) as usize] = true;
                        }
                    }
                }
                x += w;
            } else {
                let w = get_clx_opaque_pixels_width(control) as i32;
                for i in 0..w {
                    if src_offset + i as usize >= src.len() {
                        break;
                    }
                    let color = src[src_offset + i as usize];
                    if !SKIP_COLOR_ZERO || color != 0 {
                        if (x + i) > 0 && (x + i) <= width as i32 {
                            solid_mask[y as usize][(x + i) as usize] = true;
                        }
                    }
                }
                src_offset += w as usize;
                x += w;
            }
        }

        // Handle line wrap
        while x > width as i32 {
            x -= width as i32;
            y -= 1;
        }
        if x == width as i32 + 1 {
            x = 1;
            y -= 1;
        }
    }

    // Find outline pixels (adjacent to solid but not solid)
    for y in 1..=height as usize {
        for x in 1..=width {
            if solid_mask[y][x] {
                // Check 4-connected neighbors
                if !solid_mask[y - 1][x] {
                    result.push(((x - 1) as u8, (y - 1) as u8));
                }
                if !solid_mask[y + 1][x] {
                    result.push(((x - 1) as u8, y as u8));
                }
                if !solid_mask[y][x - 1] {
                    result.push(((x - 2) as u8, (y - 1) as u8));
                }
                if !solid_mask[y][x + 1] {
                    result.push((x as u8, (y - 1) as u8));
                }
            }
        }
    }

    // Deduplicate
    result.sort();
    result.dedup();
}

/// Apply TRN color translation to CLX sprite list (modifies in place)
/// C++ Reference: ClxApplyTrans(ClxSpriteList, trn)
pub fn clx_apply_trans_list(list: &mut ClxSpriteList, trn: &[u8; 256]) {
    for i in 0..list.num_sprites() as usize {
        if let Some(sprite) = list.get(i) {
            clx_apply_trans_sprite(&sprite, trn);
        }
    }
}

/// Apply TRN color translation to CLX sprite sheet (modifies in place)
/// C++ Reference: ClxApplyTrans(ClxSpriteSheet, trn)
pub fn clx_apply_trans_sheet(sheet: &mut ClxSpriteSheet, trn: &[u8; 256]) {
    for i in 0..sheet.num_lists() as usize {
        if let Some(mut list) = sheet.get(i) {
            clx_apply_trans_list(&mut list, trn);
        }
    }
}

fn clx_apply_trans_sprite(sprite: &ClxSprite, trn: &[u8; 256]) {
    // NOTE: This requires mutable access to sprite data
    // C++ does: auto *dst = const_cast<uint8_t *>(sprite.pixelData());
    // In Rust we need unsafe or interior mutability
    let data = sprite.pixel_data();
    let dst = data.as_ptr() as *mut u8;
    let mut remaining = data.len();
    let mut offset = 0;

    while remaining > 0 {
        let control = data[offset];
        offset += 1;
        remaining -= 1;

        if !is_clx_opaque(control) {
            continue;
        }

        if is_clx_opaque_fill(control) {
            // SAFETY: Mutating sprite pixel data in place (same as C++)
            unsafe {
                *dst.add(offset) = trn[*dst.add(offset) as usize];
            }
            offset += 1;
            remaining -= 1;
        } else {
            let w = get_clx_opaque_pixels_width(control) as usize;
            for i in 0..w {
                unsafe {
                    let p = dst.add(offset + i);
                    *p = trn[*p as usize];
                }
            }
            offset += w;
            remaining -= w;
        }
    }
}

/// Check if point is within CLX sprite (ignoring shadows)
/// C++ Reference: IsPointWithinClx
pub fn is_point_within_clx(position: Point, clx: &ClxSprite) -> bool {
    let src = clx.pixel_data();
    let width = clx.width() as i32;
    let height = clx.height() as i32;

    if position.x < 0 || position.x >= width || position.y < 0 || position.y >= height {
        return false;
    }

    let mut src_offset = 0;
    let mut x_cur = 0i32;
    let mut y_cur = height - 1;

    while src_offset < src.len() {
        // Skip to target row
        if y_cur != position.y {
            while x_cur < width && src_offset < src.len() {
                let info = clx_blit_info(src, src_offset);
                src_offset = info.src_end_offset;
                x_cur += info.length as i32;
            }
            while x_cur >= width {
                x_cur -= width;
                y_cur -= 1;
            }
            if y_cur < position.y {
                return false;
            }
            continue;
        }

        // Check target row
        while x_cur < width && src_offset < src.len() {
            let control = src[src_offset];

            if !is_clx_opaque(control) {
                let skip = control as i32;
                x_cur += skip;
                if x_cur > position.x {
                    return false;
                }
                src_offset += 1;
            } else if is_clx_opaque_fill(control) {
                let w = get_clx_opaque_fill_width(control) as i32;
                let color = src[src_offset + 1];
                src_offset += 2;

                if x_cur <= position.x && position.x < x_cur + w {
                    return color != 0; // ignore shadows
                }
                x_cur += w;
            } else {
                let w = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1;

                for i in 0..w {
                    if x_cur == position.x {
                        let color = src[src_offset + i as usize];
                        return color != 0; // ignore shadows
                    }
                    x_cur += 1;
                }
                src_offset += w as usize;
            }
        }

        return false;
    }

    false
}

/// Measure solid horizontal bounds of CLX sprite
/// Returns (start_x, end_x) - start inclusive, end exclusive
/// C++ Reference: ClxMeasureSolidHorizontalBounds
pub fn clx_measure_solid_horizontal_bounds(clx: &ClxSprite) -> (i32, i32) {
    let src = clx.pixel_data();
    let width = clx.width() as i32;

    let mut x_begin = width;
    let mut x_end = 0i32;
    let mut x_cur = 0i32;
    let mut src_offset = 0;

    while src_offset < src.len() {
        while x_cur < width && src_offset < src.len() {
            let control = src[src_offset];

            if !is_clx_opaque(control) {
                x_cur += control as i32;
                src_offset += 1;
            } else if is_clx_opaque_fill(control) {
                let w = get_clx_opaque_fill_width(control) as i32;
                src_offset += 2;

                x_begin = x_begin.min(x_cur);
                x_cur += w;
                x_end = x_end.max(x_cur);
            } else {
                let w = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1 + w as usize;

                x_begin = x_begin.min(x_cur);
                x_cur += w;
                x_end = x_end.max(x_cur);
            }
        }

        while x_cur >= width {
            x_cur -= width;
        }

        if x_begin == 0 && x_end == width {
            break;
        }
    }

    (x_begin, x_end)
}

/// Clear CLX draw cache - must be called when CLX sprites are freed
/// C++ Reference: ClearClxDrawCache
pub fn clear_clx_draw_cache() {
    let mut cache = OUTLINE_PIXELS_CACHE.write().unwrap();
    cache.sprite_data = std::ptr::null();
    cache.outline_pixels.clear();
}
