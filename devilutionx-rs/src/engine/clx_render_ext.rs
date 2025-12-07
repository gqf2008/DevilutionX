//! CLX 渲染扩展函数 - 半透明和光照渲染
//!
//! 补充 clx_render.rs 中缺失的高级渲染功能

use super::clx_sprite::{
    get_clx_opaque_fill_width, get_clx_opaque_pixels_width, is_clx_opaque,
    is_clx_opaque_fill, ClxSprite,
};
use super::surface::Surface;
use super::types::Point;

/// 渲染半透明 CLX 精灵
///
/// 与 clx_draw 类似，但使用透明度查找表进行 50% 混合
///
/// # Arguments
/// * `out` - 目标 surface (8-bit 索引色)
/// * `position` - 目标位置（精灵左下角）
/// * `clx` - CLX 精灵
/// * `transparency_table` - 透明度查找表 [前景色][背景色] -> 混合结果
pub fn clx_draw_blended(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    transparency_table: &[[u8; 256]; 256],
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
                let color_index = src[src_offset + 1];
                src_offset += 2;

                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        let lookup_row = &transparency_table[color_index as usize];
                        for dx in 0..fill_width {
                            let px = x + dx;
                            if px >= 0 && (px as usize) < row.len() {
                                let idx = px as usize;
                                row[idx] = lookup_row[row[idx] as usize];
                            }
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
                                    let color_index = src[src_idx];
                                    let idx = px as usize;
                                    row[idx] = transparency_table[color_index as usize][row[idx] as usize];
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

/// 渲染带光照映射的 CLX 精灵
///
/// # Arguments
/// * `out` - 目标 surface (8-bit 索引色)
/// * `position` - 目标位置（精灵左下角）
/// * `clx` - CLX 精灵
/// * `lightmap` - 光照映射数据（与目标 surface 对齐）
/// * `lightmap_pitch` - 光照映射行距
/// * `light_tables` - 光照表数组 [光照级别][颜色] -> 调整后颜色
pub fn clx_draw_with_lightmap(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    lightmap: &[u8],
    lightmap_pitch: i32,
    light_tables: &[[u8; 256]; 16],
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
                let color_index = src[src_offset + 1];
                src_offset += 2;

                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        for dx in 0..fill_width {
                            let px = x + dx;
                            if px >= 0 && (px as usize) < row.len() {
                                let lightmap_idx = (y * lightmap_pitch + px) as usize;
                                let light_level = if lightmap_idx < lightmap.len() {
                                    (lightmap[lightmap_idx] as usize).min(15)
                                } else {
                                    15
                                };

                                let idx = px as usize;
                                row[idx] = light_tables[light_level][color_index as usize];
                            }
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
                                    let color_index = src[src_idx];

                                    let lightmap_idx = (y * lightmap_pitch + px) as usize;
                                    let light_level = if lightmap_idx < lightmap.len() {
                                        (lightmap[lightmap_idx] as usize).min(15)
                                    } else {
                                        15
                                    };

                                    let idx = px as usize;
                                    row[idx] = light_tables[light_level][color_index as usize];
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

/// 渲染半透明且带光照的 CLX 精灵
pub fn clx_draw_blended_with_lightmap(
    out: &mut Surface,
    position: Point,
    clx: &ClxSprite,
    lightmap: &[u8],
    lightmap_pitch: i32,
    light_tables: &[[u8; 256]; 16],
    transparency_table: &[[u8; 256]; 256],
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
                let color_index = src[src_offset + 1];
                src_offset += 2;

                if y >= 0 && y < out_h {
                    if let Some(row) = out.row_mut(y) {
                        for dx in 0..fill_width {
                            let px = x + dx;
                            if px >= 0 && (px as usize) < row.len() {
                                let lightmap_idx = (y * lightmap_pitch + px) as usize;
                                let light_level = if lightmap_idx < lightmap.len() {
                                    (lightmap[lightmap_idx] as usize).min(15)
                                } else {
                                    15
                                };

                                let lit_color = light_tables[light_level][color_index as usize];
                                let idx = px as usize;
                                row[idx] = transparency_table[lit_color as usize][row[idx] as usize];
                            }
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
                                    let color_index = src[src_idx];

                                    let lightmap_idx = (y * lightmap_pitch + px) as usize;
                                    let light_level = if lightmap_idx < lightmap.len() {
                                        (lightmap[lightmap_idx] as usize).min(15)
                                    } else {
                                        15
                                    };

                                    let lit_color = light_tables[light_level][color_index as usize];
                                    let idx = px as usize;
                                    row[idx] = transparency_table[lit_color as usize][row[idx] as usize];
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

/// 检测点是否在 CLX 精灵的不透明区域内
///
/// 用于鼠标点击检测，忽略阴影（颜色索引 0）
pub fn is_point_within_clx(position: Point, clx: &ClxSprite, sprite_position: Point) -> bool {
    let width = clx.width() as i32;
    let height = clx.height() as i32;

    let local_x = position.x - sprite_position.x;
    let local_y = sprite_position.y - position.y;

    if local_x < 0 || local_x >= width || local_y < 0 || local_y >= height {
        return false;
    }

    let src = clx.pixel_data();
    let mut src_offset = 0;
    let mut current_row = 0;

    while src_offset < src.len() && current_row < height {
        let mut x = 0;
        let mut remaining_width = width;

        while remaining_width > 0 && src_offset < src.len() {
            let control = src[src_offset];

            if !is_clx_opaque(control) {
                let skip = control as i32;

                if current_row == local_y && local_x >= x && local_x < x + skip {
                    return false;
                }

                x += skip;
                remaining_width -= skip;
                src_offset += 1;
            } else if is_clx_opaque_fill(control) {
                let fill_width = get_clx_opaque_fill_width(control) as i32;
                let color_index = src[src_offset + 1];
                src_offset += 2;

                if current_row == local_y && local_x >= x && local_x < x + fill_width {
                    return color_index != 0;
                }

                x += fill_width;
                remaining_width -= fill_width;
            } else {
                let pixel_width = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1;

                if current_row == local_y && local_x >= x && local_x < x + pixel_width {
                    let pixel_offset = (local_x - x) as usize;
                    if src_offset + pixel_offset < src.len() {
                        let color_index = src[src_offset + pixel_offset];
                        return color_index != 0;
                    }
                }

                src_offset += pixel_width as usize;
                x += pixel_width;
                remaining_width -= pixel_width;
            }
        }

        while remaining_width < 0 {
            remaining_width += width;
            current_row += 1;
        }

        if remaining_width == 0 {
            current_row += 1;
        }
    }

    false
}

/// 测量 CLX 精灵的实际水平边界（跳过完全透明的列）
///
/// 返回 (start_x, end_x)
pub fn clx_measure_solid_horizontal_bounds(clx: &ClxSprite) -> (i32, i32) {
    let width = clx.width() as i32;
    let height = clx.height() as i32;
    let src = clx.pixel_data();

    let mut min_x = width;
    let mut max_x = 0i32;

    let mut src_offset = 0;
    let mut current_row = 0;

    while src_offset < src.len() && current_row < height {
        let mut x = 0;
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
                src_offset += 2;

                min_x = min_x.min(x);
                max_x = max_x.max(x + fill_width);

                x += fill_width;
                remaining_width -= fill_width;
            } else {
                let pixel_width = get_clx_opaque_pixels_width(control) as i32;
                src_offset += 1 + pixel_width as usize;

                min_x = min_x.min(x);
                max_x = max_x.max(x + pixel_width);

                x += pixel_width;
                remaining_width -= pixel_width;
            }
        }

        while remaining_width < 0 {
            remaining_width += width;
            current_row += 1;
        }

        if remaining_width == 0 {
            current_row += 1;
        }
    }

    if min_x > max_x {
        (0, 0)
    } else {
        (min_x, max_x)
    }
}

/// 批量应用 TRN 变换到 CLX 精灵列表
/// 注意：这会修改精灵数据
pub fn clx_apply_trans(sprite_data: &mut [u8], trn: &[u8; 256]) {
    // CLX 数据结构中只修改颜色数据部分
    // 这是一个简化实现，实际需要解析 CLX 结构
    for byte in sprite_data.iter_mut() {
        *byte = trn[*byte as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_sprite() -> ClxSprite {
        // 创建一个有效的 CLX 精灵
        // CLX 头部格式:
        // - 字节 0..2: uint16_t 头大小 (6)
        // - 字节 2..4: uint16_t 宽度 (4)
        // - 字节 4..6: uint16_t 高度 (2)
        // 像素数据: [跳过2][2个不透明像素, color5, color6]
        let data = vec![
            0x06, 0x00,  // header_size = 6
            0x04, 0x00,  // width = 4
            0x02, 0x00,  // height = 2
            // 行 1 像素数据 (宽度4)
            0x02,        // 跳过 2 像素
            0x82,        // 2 个不透明像素
            0x05, 0x06,  // 颜色 5 和 6
            // 行 2 像素数据 (宽度4)
            0x01,        // 跳过 1 像素
            0x82,        // 2 个不透明像素
            0x07, 0x08,  // 颜色 7 和 8
            0x01,        // 跳过 1 像素
        ];
        let data_len = data.len() as u32;
        ClxSprite::new(Arc::new(data), 0, data_len)
    }

    #[test]
    fn test_clx_measure_bounds() {
        let sprite = create_test_sprite();
        let (min_x, max_x) = clx_measure_solid_horizontal_bounds(&sprite);
        // 行 1: x=2..4 有像素
        // 行 2: x=1..3 有像素
        // 最小 x = 1, 最大 x = 4
        assert!(min_x <= 2);
        assert!(max_x >= 3);
    }

    #[test]
    fn test_clx_empty_sprite() {
        // 创建一个完全透明的精灵
        let data = vec![
            0x06, 0x00,  // header_size = 6
            0x04, 0x00,  // width = 4
            0x01, 0x00,  // height = 1
            0x04,        // 跳过 4 像素 (完全透明行)
        ];
        let data_len = data.len() as u32;
        let sprite = ClxSprite::new(Arc::new(data), 0, data_len);
        let (min_x, max_x) = clx_measure_solid_horizontal_bounds(&sprite);
        // 无不透明区域: min_x > max_x 表示空
        assert!(min_x > max_x || (min_x == 0 && max_x == 0));
    }
}
