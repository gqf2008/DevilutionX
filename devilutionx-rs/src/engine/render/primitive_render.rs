//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! 基础图形渲染 - 移植自 Source/engine/render/primitive_render.cpp
//!
//! 提供矩形填充、直线绘制、半透明渲染等基础图形功能

use crate::engine::surface::Surface;
use crate::engine::{Point, Rectangle};
use std::ops::Index;
use std::sync::RwLock;

/// 透明度查找表 (256x256)
/// 用于调色板混合，实现半透明效果
/// 对应 C++ paletteTransparencyLookup
pub struct TransparencyLookup {
    /// 256x256 查找表: [前景色][背景色] -> 混合结果
    table: [[u8; 256]; 256],
}

// === 全局透明度查找表 (对应 C++ paletteTransparencyLookup) ===
static PALETTE_TRANSPARENCY_LOOKUP: RwLock<TransparencyLookup> = RwLock::new(TransparencyLookup::empty());

/// 黑色混合的 16 位查找表 (用于 32 位对齐优化)
/// 对应 C++ paletteTransparencyLookupBlack16
#[cfg(feature = "palette_transparency_black_16_lut")]
static PALETTE_TRANSPARENCY_LOOKUP_BLACK_16: RwLock<[u16; 65536]> = RwLock::new([0u16; 65536]);

/// 获取全局透明度查找表引用
pub fn get_palette_transparency_lookup() -> std::sync::RwLockReadGuard<'static, TransparencyLookup> {
    PALETTE_TRANSPARENCY_LOOKUP.read().unwrap()
}

/// 设置全局透明度查找表
pub fn set_palette_transparency_lookup(lookup: TransparencyLookup) {
    *PALETTE_TRANSPARENCY_LOOKUP.write().unwrap() = lookup;
}

impl TransparencyLookup {
    /// 创建空的透明度查找表 (const fn for static init)
    pub const fn empty() -> Self {
        Self {
            table: [[0; 256]; 256],
        }
    }

    /// 创建新的透明度查找表
    pub fn new() -> Self {
        Self::empty()
    }

    /// 从预计算数据创建
    pub fn from_data(data: &[u8]) -> Option<Self> {
        if data.len() < 256 * 256 {
            return None;
        }
        let mut table = [[0u8; 256]; 256];
        for (i, chunk) in data.chunks_exact(256).enumerate().take(256) {
            table[i].copy_from_slice(chunk);
        }
        Some(Self { table })
    }

    /// 获取混合结果
    #[inline]
    pub fn blend(&self, foreground: u8, background: u8) -> u8 {
        self.table[foreground as usize][background as usize]
    }

    /// 获取整个前景色的查找行
    #[inline]
    pub fn lookup_row(&self, foreground: u8) -> &[u8; 256] {
        &self.table[foreground as usize]
    }
}

impl Default for TransparencyLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for TransparencyLookup {
    type Output = [u8; 256];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

/// 填充矩形
///
/// # Arguments
/// * `out` - 目标 surface
/// * `x`, `y` - 起始坐标
/// * `width`, `height` - 矩形尺寸
/// * `color_index` - 调色板颜色索引
pub fn fill_rect(out: &mut Surface, x: i32, y: i32, width: i32, height: i32, color_index: u8) {
    for j in 0..height {
        draw_horizontal_line(out, Point::new(x, y + j), width, color_index);
    }
}

/// 绘制水平线 (从左到右)
///
/// # Arguments
/// * `out` - 目标 surface
/// * `from` - 起始点
/// * `width` - 线宽度
/// * `color_index` - 调色板颜色索引
pub fn draw_horizontal_line(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    let out_w = out.w();
    let out_h = out.h();

    if from.y < 0 || from.y >= out_h || from.x >= out_w || width <= 0 || from.x + width <= 0 {
        return;
    }

    let mut from = from;
    let mut width = width;

    if from.x < 0 {
        width += from.x;
        from.x = 0;
    }
    if from.x + width > out_w {
        width = out_w - from.x;
    }

    unsafe_draw_horizontal_line(out, from, width, color_index);
}

/// 绘制水平线（无边界检查）
pub fn unsafe_draw_horizontal_line(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    if let Some(row) = out.row_mut(from.y) {
        let start = from.x as usize;
        let end = (from.x + width) as usize;
        if end <= row.len() {
            row[start..end].fill(color_index);
        }
    }
}

/// 绘制垂直线 (从上到下)
///
/// # Arguments
/// * `out` - 目标 surface
/// * `from` - 起始点
/// * `height` - 线高度
/// * `color_index` - 调色板颜色索引
pub fn draw_vertical_line(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let out_w = out.w();
    let out_h = out.h();

    if from.x < 0 || from.x >= out_w || from.y >= out_h || height <= 0 || from.y + height <= 0 {
        return;
    }

    let mut from = from;
    let mut height = height;

    if from.y < 0 {
        height += from.y;
        from.y = 0;
    }
    if from.y + height > out_h {
        height = out_h - from.y;
    }

    unsafe_draw_vertical_line(out, from, height, color_index);
}

/// 绘制垂直线（无边界检查）
pub fn unsafe_draw_vertical_line(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let x = from.x as usize;
    for dy in 0..height {
        let y = from.y + dy;
        if let Some(row) = out.row_mut(y) {
            if x < row.len() {
                row[x] = color_index;
            }
        }
    }
}

/// 绘制半透明水平线 - 对应 C++ DrawHalfTransparentHorizontalLine
pub fn draw_half_transparent_horizontal_line(
    out: &mut Surface,
    from: Point,
    width: i32,
    color_index: u8,
) {
    let out_w = out.w();
    let out_h = out.h();

    // 完全超出边界？
    if from.y < 0 || from.y >= out_h || width <= 0 || from.x >= out_w || from.x + width <= 0 {
        return;
    }

    let x0 = from.x.max(0);
    let x1 = (from.x + width).min(out_w);

    for x in x0..x1 {
        set_half_transparent_pixel(out, Point::new(x, from.y), color_index);
    }
}

/// 绘制半透明垂直线 - 对应 C++ DrawHalfTransparentVerticalLine
pub fn draw_half_transparent_vertical_line(
    out: &mut Surface,
    from: Point,
    height: i32,
    color_index: u8,
) {
    let out_w = out.w();
    let out_h = out.h();

    // 完全超出边界？
    if from.x < 0 || from.x >= out_w || height <= 0 || from.y >= out_h || from.y + height <= 0 {
        return;
    }

    let y0 = from.y.max(0);
    let y1 = (from.y + height).min(out_h);

    for y in y0..y1 {
        set_half_transparent_pixel(out, Point::new(from.x, y), color_index);
    }
}

/// 绘制半透明水平线（带自定义 lookup）
pub fn draw_half_transparent_horizontal_line_with_lookup(
    out: &mut Surface,
    from: Point,
    width: i32,
    color_index: u8,
    lookup: &TransparencyLookup,
) {
    let out_w = out.w();
    let out_h = out.h();

    if from.y < 0 || from.y >= out_h || width <= 0 || from.x >= out_w || from.x + width <= 0 {
        return;
    }

    let x0 = from.x.max(0);
    let x1 = (from.x + width).min(out_w);

    for x in x0..x1 {
        set_half_transparent_pixel_with_lookup(out, Point::new(x, from.y), color_index, lookup);
    }
}

/// 绘制半透明垂直线（带自定义 lookup）
pub fn draw_half_transparent_vertical_line_with_lookup(
    out: &mut Surface,
    from: Point,
    height: i32,
    color_index: u8,
    lookup: &TransparencyLookup,
) {
    let out_w = out.w();
    let out_h = out.h();

    if from.x < 0 || from.x >= out_w || height <= 0 || from.y >= out_h || from.y + height <= 0 {
        return;
    }

    let y0 = from.y.max(0);
    let y1 = (from.y + height).min(out_h);

    for y in y0..y1 {
        set_half_transparent_pixel_with_lookup(out, Point::new(from.x, y), color_index, lookup);
    }
}

/// 绘制半透明矩形（与黑色混合）- 对应 C++ DrawHalfTransparentRectTo(out, sx, sy, w, h)
pub fn draw_half_transparent_rect(
    out: &mut Surface,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
) {
    draw_half_transparent_rect_impl(out, sx, sy, width, height, 0);
}

/// 绘制半透明矩形（与指定颜色混合）- 对应 C++ DrawHalfTransparentRectTo(out, sx, sy, w, h, color)
pub fn draw_half_transparent_rect_colored(
    out: &mut Surface,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    color: u8,
) {
    draw_half_transparent_rect_impl(out, sx, sy, width, height, color);
}

/// 半透明矩形的内部实现
fn draw_half_transparent_rect_impl(
    out: &mut Surface,
    mut sx: i32,
    mut sy: i32,
    mut width: i32,
    mut height: i32,
    color: u8,
) {
    let out_w = out.w();
    let out_h = out.h();

    if sx + width < 0 || sy + height < 0 || sx >= out_w || sy >= out_h {
        return;
    }

    if sx < 0 {
        width += sx;
        sx = 0;
    } else if sx + width >= out_w {
        width = out_w - sx;
    }

    if sy < 0 {
        height += sy;
        sy = 0;
    } else if sy + height >= out_h {
        height = out_h - sy;
    }

    draw_half_transparent_blended_rect(out, sx as u32, sy as u32, width as u32, height as u32, color);
}

/// 内部：非对齐的半透明矩形渲染
/// 对应 C++ DrawHalfTransparentUnalignedBlendedRectTo
fn draw_half_transparent_blended_rect(
    out: &mut Surface,
    sx: u32,
    sy: u32,
    width: u32,
    height: u32,
    color: u8,
) {
    let lookup = get_palette_transparency_lookup();
    let lookup_row = lookup.lookup_row(color);

    for y in sy..(sy + height) {
        if let Some(row) = out.row_mut(y as i32) {
            let start = sx as usize;
            let end = (sx + width) as usize;
            for x in start..end.min(row.len()) {
                row[x] = lookup_row[row[x] as usize];
            }
        }
    }
}

/// 绘制半透明矩形（带自定义 lookup）
pub fn draw_half_transparent_rect_with_lookup(
    out: &mut Surface,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    lookup: &TransparencyLookup,
) {
    draw_half_transparent_rect_colored_with_lookup(out, sx, sy, width, height, 0, lookup);
}

/// 绘制半透明矩形（带自定义 lookup 和颜色）
pub fn draw_half_transparent_rect_colored_with_lookup(
    out: &mut Surface,
    mut sx: i32,
    mut sy: i32,
    mut width: i32,
    mut height: i32,
    color: u8,
    lookup: &TransparencyLookup,
) {
    let out_w = out.w();
    let out_h = out.h();

    if sx + width < 0 || sy + height < 0 || sx >= out_w || sy >= out_h {
        return;
    }

    if sx < 0 {
        width += sx;
        sx = 0;
    } else if sx + width >= out_w {
        width = out_w - sx;
    }

    if sy < 0 {
        height += sy;
        sy = 0;
    } else if sy + height >= out_h {
        height = out_h - sy;
    }

    let lookup_row = lookup.lookup_row(color);

    for y in sy..(sy + height) {
        if let Some(row) = out.row_mut(y) {
            let start = sx as usize;
            let end = (sx + width) as usize;
            for x in start..end.min(row.len()) {
                row[x] = lookup_row[row[x] as usize];
            }
        }
    }
}

/// 设置半透明像素 (使用全局透明度查找表)
/// 对应 C++ SetHalfTransparentPixel
pub fn set_half_transparent_pixel(
    out: &mut Surface,
    position: Point,
    color: u8,
) {
    if position.x >= 0
        && position.x < out.w()
        && position.y >= 0
        && position.y < out.h()
    {
        let lookup = get_palette_transparency_lookup();
        if let Some(row) = out.row_mut(position.y) {
            let x = position.x as usize;
            if x < row.len() {
                row[x] = lookup.blend(color, row[x]);
            }
        }
    }
}

/// 设置半透明像素 (使用指定的透明度查找表)
pub fn set_half_transparent_pixel_with_lookup(
    out: &mut Surface,
    position: Point,
    color: u8,
    lookup: &TransparencyLookup,
) {
    if position.x >= 0
        && position.x < out.w()
        && position.y >= 0
        && position.y < out.h()
    {
        if let Some(row) = out.row_mut(position.y) {
            let x = position.x as usize;
            if x < row.len() {
                row[x] = lookup.blend(color, row[x]);
            }
        }
    }
}

/// 绘制 2 像素宽的边框（无边界检查）
pub fn unsafe_draw_border_2px(out: &mut Surface, rect: Rectangle, color: u8) {
    let width = rect.size.width as usize;
    let height = rect.size.height;

    // 顶部两行
    for dy in 0..2 {
        if let Some(row) = out.row_mut(rect.position.y + dy) {
            let start = rect.position.x as usize;
            let end = start + width;
            if end <= row.len() {
                row[start..end].fill(color);
            }
        }
    }

    // 左右边框
    for dy in 2..(height - 2) {
        if let Some(row) = out.row_mut(rect.position.y + dy) {
            let x = rect.position.x as usize;
            if x + 1 < row.len() {
                row[x] = color;
                row[x + 1] = color;
            }
            let x_right = (rect.position.x + rect.size.width - 2) as usize;
            if x_right + 1 < row.len() {
                row[x_right] = color;
                row[x_right + 1] = color;
            }
        }
    }

    // 底部两行
    for dy in (height - 2)..height {
        if let Some(row) = out.row_mut(rect.position.y + dy) {
            let start = rect.position.x as usize;
            let end = start + width;
            if end <= row.len() {
                row[start..end].fill(color);
            }
        }
    }
}

/// 绘制矩形边框
pub fn draw_rect_outline(out: &mut Surface, rect: Rectangle, color: u8) {
    let x = rect.position.x;
    let y = rect.position.y;
    let w = rect.size.width;
    let h = rect.size.height;

    // 上边
    draw_horizontal_line(out, Point::new(x, y), w, color);
    // 下边
    draw_horizontal_line(out, Point::new(x, y + h - 1), w, color);
    // 左边
    draw_vertical_line(out, Point::new(x, y), h, color);
    // 右边
    draw_vertical_line(out, Point::new(x + w - 1, y), h, color);
}

/// Bresenham 直线算法
pub fn draw_line(out: &mut Surface, from: Point, to: Point, color: u8) {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = from.x;
    let mut y = from.y;

    loop {
        out.set_pixel(Point::new(x, y), color);

        if x == to.x && y == to.y {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_rect() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        fill_rect(&mut surface, 10, 10, 20, 20, 5);

        // 检查填充区域
        assert_eq!(surface.row(15).unwrap()[15], 5);
        assert_eq!(surface.row(25).unwrap()[25], 5);

        // 检查未填充区域
        assert_eq!(surface.row(5).unwrap()[5], 0);
    }

    #[test]
    fn test_draw_horizontal_line() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        draw_horizontal_line(&mut surface, Point::new(10, 50), 30, 7);

        let row = surface.row(50).unwrap();
        assert_eq!(row[10], 7);
        assert_eq!(row[39], 7);
        assert_eq!(row[9], 0);
        assert_eq!(row[40], 0);
    }

    #[test]
    fn test_draw_vertical_line() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        draw_vertical_line(&mut surface, Point::new(50, 10), 30, 9);

        assert_eq!(surface.row(10).unwrap()[50], 9);
        assert_eq!(surface.row(39).unwrap()[50], 9);
        assert_eq!(surface.row(9).unwrap()[50], 0);
        assert_eq!(surface.row(40).unwrap()[50], 0);
    }

    #[test]
    fn test_transparency_lookup() {
        let mut data = vec![0u8; 256 * 256];
        // 简单的测试: 设置对角线
        for i in 0..256 {
            data[i * 256 + i] = i as u8;
        }

        let lookup = TransparencyLookup::from_data(&data).unwrap();
        assert_eq!(lookup.blend(100, 100), 100);
        assert_eq!(lookup.blend(0, 0), 0);
    }

    #[test]
    fn test_draw_line() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        draw_line(&mut surface, Point::new(10, 10), Point::new(20, 10), 3);

        // 水平线测试
        let row = surface.row(10).unwrap();
        for x in 10..=20 {
            assert_eq!(row[x], 3);
        }
    }
}
