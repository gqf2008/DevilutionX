//! CLX 渲染 - 移植自 Source/engine/render/clx_render.cpp
//!
//! 将 CLX 精灵渲染到 8-bit indexed Surface

use super::clx_sprite::{
    get_clx_opaque_fill_width, get_clx_opaque_pixels_width, is_clx_opaque,
    is_clx_opaque_fill, ClxSprite,
};
use super::surface::Surface;
use super::types::Point;

/// 渲染 CLX 精灵到 8-bit indexed surface
///
/// # Arguments
/// * `out` - 目标 surface (8-bit 索引色)
/// * `position` - 目标位置（精灵左下角）
/// * `clx` - CLX 精灵
/// * `light_table` - 可选的光照表 (256 字节)
pub fn clx_draw(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    light_table: Option<&[u8; 256]>,
) {
    let width = clx.width() as i32;
    let height = clx.height() as i32;
    let src = clx.pixel_data();
    let out_w = out.w();
    let out_h = out.h();

    // 计算起始位置（从底部开始向上渲染）
    let start_y = position.y;
    let start_x = position.x;

    // 裁剪检查
    if start_x + width <= 0 || start_x >= out_w {
        return;
    }
    if start_y - height + 1 >= out_h || start_y < 0 {
        return;
    }

    let mut src_offset = 0;
    let mut y = start_y;

    while src_offset < src.len() && y >= 0 {
        let mut x = start_x;
        let mut remaining_width = width;

        while remaining_width > 0 && src_offset < src.len() {
            let control = src[src_offset];

            if !is_clx_opaque(control) {
                // 透明运行 - 跳过 N 个像素
                let skip = control as i32;
                x += skip;
                remaining_width -= skip;
                src_offset += 1;
            } else if is_clx_opaque_fill(control) {
                // 填充 - 用单个颜色填充 N 个像素
                let fill_width = get_clx_opaque_fill_width(control) as i32;
                let mut color_index = src[src_offset + 1];
                src_offset += 2;

                // 应用光照表
                if let Some(tbl) = light_table {
                    color_index = tbl[color_index as usize];
                }

                // 绘制填充
                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        let draw_start = x.max(0) as usize;
                        let draw_end = (x + fill_width).min(out_w) as usize;
                        if draw_start < draw_end && draw_end <= row.len() {
                            row[draw_start..draw_end].fill(color_index);
                        }
                    }
                }

                x += fill_width;
                remaining_width -= fill_width;
            } else {
                // 像素数据 - 复制 N 个像素
                let pixel_width = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1;

                // 绘制像素
                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        for i in 0..pixel_width {
                            let px = x + i;
                            if px >= 0 && (px as usize) < row.len() {
                                let src_idx = src_offset + i as usize;
                                if src_idx < src.len() {
                                    let mut color_index = src[src_idx];
                                    if let Some(tbl) = light_table {
                                        color_index = tbl[color_index as usize];
                                    }
                                    row[px as usize] = color_index;
                                }
                            }
                        }
                    }
                }

                src_offset += pixel_width as usize;
                x += pixel_width;
                remaining_width -= pixel_width;
            }
        }

        // 处理行结束的溢出
        while remaining_width < 0 {
            remaining_width += width;
            y -= 1;
        }

        if remaining_width == 0 {
            y -= 1;
        }
    }
}

/// 使用 TRN 颜色变换渲染 CLX 精灵
///
/// # Arguments
/// * `out` - 目标 surface
/// * `position` - 目标位置（精灵左下角）
/// * `clx` - CLX 精灵
/// * `trn` - 颜色变换表（256 字节）
/// * `light_table` - 可选的光照表
pub fn clx_draw_trn(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    trn: &[u8; 256],
    light_table: Option<&[u8; 256]>,
) {
    let width = clx.width() as i32;
    let height = clx.height() as i32;
    let src = clx.pixel_data();
    let out_w = out.w();
    let out_h = out.h();

    let start_y = position.y;
    let start_x = position.x;

    if start_x + width <= 0 || start_x >= out_w {
        return;
    }
    if start_y - height + 1 >= out_h || start_y < 0 {
        return;
    }

    let mut src_offset = 0;
    let mut y = start_y;

    while src_offset < src.len() && y >= 0 {
        let mut x = start_x;
        let mut remaining_width = width;

        while remaining_width > 0 && src_offset < src.len() {
            let control = src[src_offset];

            if !is_clx_opaque(control) {
                let skip = control as i32;
                x += skip;
                remaining_width -= skip;
                src_offset += 1;
            } else if is_clx_opaque_fill(control) {
                let fill_width = get_clx_opaque_fill_width(control) as i32;
                let mut color_index = trn[src[src_offset + 1] as usize];
                src_offset += 2;

                if let Some(tbl) = light_table {
                    color_index = tbl[color_index as usize];
                }

                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        let draw_start = x.max(0) as usize;
                        let draw_end = (x + fill_width).min(out_w) as usize;
                        if draw_start < draw_end && draw_end <= row.len() {
                            row[draw_start..draw_end].fill(color_index);
                        }
                    }
                }

                x += fill_width;
                remaining_width -= fill_width;
            } else {
                let pixel_width = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1;

                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        for i in 0..pixel_width {
                            let px = x + i;
                            if px >= 0 && (px as usize) < row.len() {
                                let src_idx = src_offset + i as usize;
                                if src_idx < src.len() {
                                    let mut color_index = trn[src[src_idx] as usize];
                                    if let Some(tbl) = light_table {
                                        color_index = tbl[color_index as usize];
                                    }
                                    row[px as usize] = color_index;
                                }
                            }
                        }
                    }
                }

                src_offset += pixel_width as usize;
                x += pixel_width;
                remaining_width -= pixel_width;
            }
        }

        while remaining_width < 0 {
            remaining_width += width;
            y -= 1;
        }

        if remaining_width == 0 {
            y -= 1;
        }
    }
}

/// 绘制 CLX 精灵轮廓
///
/// 只绘制非透明像素的外边缘
pub fn clx_draw_outline(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    outline_color: u8,
) {
    let width = clx.width() as i32;
    let height = clx.height() as i32;
    let src = clx.pixel_data();
    let out_w = out.w();
    let out_h = out.h();

    let start_y = position.y;
    let start_x = position.x;

    if start_x + width <= 0 || start_x >= out_w {
        return;
    }
    if start_y - height + 1 >= out_h || start_y < 0 {
        return;
    }

    let mut src_offset = 0;
    let mut y = start_y;

    while src_offset < src.len() && y >= 0 {
        let mut x = start_x;
        let mut remaining_width = width;

        while remaining_width > 0 && src_offset < src.len() {
            let control = src[src_offset];

            if !is_clx_opaque(control) {
                let skip = control as i32;
                
                // 在透明段之后绘制轮廓（如果有不透明像素）
                if y >= 0 && y < out_h && skip > 0 {
                    // 检查前一个像素是否为不透明
                    let px = x - 1;
                    if px >= 0 && (px as usize) < out_w as usize {
                        // 在透明区域的开始处绘制
                        out.set_pixel(Point::new(x, y), outline_color);
                    }
                    // 在透明区域的结束处绘制
                    let end_x = x + skip - 1;
                    if end_x >= 0 && (end_x as usize) < out_w as usize {
                        out.set_pixel(Point::new(end_x, y), outline_color);
                    }
                }
                
                x += skip;
                remaining_width -= skip;
                src_offset += 1;
            } else if is_clx_opaque_fill(control) {
                let fill_width = get_clx_opaque_fill_width(control) as i32;
                src_offset += 2;
                x += fill_width;
                remaining_width -= fill_width;
            } else {
                let pixel_width = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1 + pixel_width as usize;
                x += pixel_width;
                remaining_width -= pixel_width;
            }
        }

        while remaining_width < 0 {
            remaining_width += width;
            y -= 1;
        }

        if remaining_width == 0 {
            y -= 1;
        }
    }
}
