//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! 自动地图渲染 - 移植自 Source/engine/render/automap_render.cpp
//!
//! 提供自动地图线条绘制功能，支持等距投影的斜线

use super::primitive_render::set_half_transparent_pixel;
use crate::engine::surface::Surface;
use crate::engine::{Point, Rectangle, Size};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;

/// 自动地图类型 - 对应 C++ AutomapType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AutomapType {
    /// 完全不透明
    #[default]
    Opaque = 0,
    /// 半透明
    Transparent = 1,
    /// 小地图
    Minimap = 2,
}

impl From<u8> for AutomapType {
    fn from(v: u8) -> Self {
        match v {
            1 => AutomapType::Transparent,
            2 => AutomapType::Minimap,
            _ => AutomapType::Opaque,
        }
    }
}

// === 全局状态 (对应 C++ automap.h 中的全局变量) ===

static AUTOMAP_TYPE: AtomicU8 = AtomicU8::new(0);
static MINIMAP_RECT: RwLock<Rectangle> = RwLock::new(Rectangle::new(Point::new(0, 0), Size::new(0, 0)));

/// 获取自动地图类型 - 对应 C++ GetAutomapType()
pub fn get_automap_type() -> AutomapType {
    AutomapType::from(AUTOMAP_TYPE.load(Ordering::Relaxed))
}

/// 设置自动地图类型
pub fn set_automap_type(automap_type: AutomapType) {
    AUTOMAP_TYPE.store(automap_type as u8, Ordering::Relaxed);
}

/// 获取小地图区域 - 对应 C++ MinimapRect
pub fn get_minimap_rect() -> Rectangle {
    *MINIMAP_RECT.read().unwrap()
}

/// 设置小地图区域
pub fn set_minimap_rect(rect: Rectangle) {
    *MINIMAP_RECT.write().unwrap() = rect;
}

// === 方向枚举 (对应 C++ 匿名命名空间中的 DirectionX/DirectionY) ===

#[derive(Clone, Copy)]
enum DirectionX {
    East = 1,
    West = -1,
}

#[derive(Clone, Copy)]
enum DirectionY {
    South = 1,
    North = -1,
}

// === 内部绘制函数 (对应 C++ 模板函数) ===

fn draw_map_line_impl(
    out: &mut Surface,
    mut from: Point,
    mut height: i32,
    color_index: u8,
    dir_x: DirectionX,
    dir_y: DirectionY,
) {
    let dx = dir_x as i32;
    let dy = dir_y as i32;

    while height > 0 {
        height -= 1;
        set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
        set_map_pixel(out, from, color_index);
        from.x += dx;
        set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
        set_map_pixel(out, from, color_index);
        from.x += dx;
        from.y += dy;
    }
    set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
    set_map_pixel(out, from, color_index);
}

fn draw_map_line_steep_impl(
    out: &mut Surface,
    mut from: Point,
    mut width: i32,
    color_index: u8,
    dir_x: DirectionX,
    dir_y: DirectionY,
) {
    let dx = dir_x as i32;
    let dy = dir_y as i32;

    while width > 0 {
        width -= 1;
        set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
        set_map_pixel(out, from, color_index);
        from.y += dy;
        set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
        set_map_pixel(out, from, color_index);
        from.y += dy;
        from.x += dx;
    }
    set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
    set_map_pixel(out, from, color_index);
}

// === 公开 API (对应 C++ automap_render.hpp) ===

/// 绘制南北方向线 (垂直)
pub fn draw_map_line_ns(out: &mut Surface, mut from: Point, mut height: i32, color_index: u8) {
    let out_w = out.w();
    let out_h = out.h();

    if from.x < 0 || from.x >= out_w || from.y >= out_h || height <= 0 || from.y + height <= 0 {
        return;
    }

    if from.y < 0 {
        height += from.y;
        from.y = 0;
    }

    if from.y + height > out_h {
        height = out_h - from.y;
    }

    for i in 0..height {
        set_map_pixel(out, Point::new(from.x, from.y + i), color_index);
    }
}

/// 绘制东西方向线 (水平)
pub fn draw_map_line_we(out: &mut Surface, mut from: Point, mut width: i32, color_index: u8) {
    let out_w = out.w();
    let out_h = out.h();

    if from.y < 0 || from.y >= out_h || from.x >= out_w || width <= 0 || from.x + width <= 0 {
        return;
    }

    if from.x < 0 {
        width += from.x;
        from.x = 0;
    }

    if from.x + width > out_w {
        width = out_w - from.x;
    }

    for i in 0..width {
        set_map_pixel(out, Point::new(from.x + i, from.y), color_index);
    }
}

/// 绘制东北方向线 (atan(1/2) 角度)
/// 终点: { from.x + 2 * height + 1, from.y - height }
pub fn draw_map_line_ne(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line_impl(out, from, height, color_index, DirectionX::East, DirectionY::North);
}

/// 绘制东南方向线
/// 终点: { from.x + 2 * height + 1, from.y + height }
pub fn draw_map_line_se(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line_impl(out, from, height, color_index, DirectionX::East, DirectionY::South);
}

/// 绘制西北方向线
/// 终点: { from.x - 2 * height + 1, from.y - height }
pub fn draw_map_line_nw(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line_impl(out, from, height, color_index, DirectionX::West, DirectionY::North);
}

/// 绘制西南方向线
/// 终点: { from.x - 2 * height + 1, from.y + height }
pub fn draw_map_line_sw(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line_impl(out, from, height, color_index, DirectionX::West, DirectionY::South);
}

/// 绘制陡峭的东北方向线 (atan(2) 角度)
/// 终点: { from.x + width + 1, from.y - 2 * width }
pub fn draw_map_line_steep_ne(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep_impl(out, from, width, color_index, DirectionX::East, DirectionY::North);
}

/// 绘制陡峭的东南方向线
/// 终点: { from.x + width + 1, from.y + 2 * width }
pub fn draw_map_line_steep_se(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep_impl(out, from, width, color_index, DirectionX::East, DirectionY::South);
}

/// 绘制陡峭的西北方向线
/// 终点: { from.x - (width + 1), from.y - 2 * width }
pub fn draw_map_line_steep_nw(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep_impl(out, from, width, color_index, DirectionX::West, DirectionY::North);
}

/// 绘制陡峭的西南方向线
/// 终点: { from.x - (width + 1), from.y + 2 * width }
pub fn draw_map_line_steep_sw(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep_impl(out, from, width, color_index, DirectionX::West, DirectionY::South);
}

/// 绘制自由直线 (Bresenham 算法，不包含阴影)
pub fn draw_map_free_line(out: &mut Surface, mut from: Point, to: Point, color_index: u8) {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        set_map_pixel(out, from, color_index);

        if from.x == to.x && from.y == to.y {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            from.x += sx;
        }
        if e2 < dx {
            err += dx;
            from.y += sy;
        }
    }
}

/// 设置地图像素 - 对应 C++ SetMapPixel
pub fn set_map_pixel(out: &mut Surface, position: Point, color: u8) {
    let automap_type = get_automap_type();

    if automap_type == AutomapType::Minimap {
        let minimap_rect = get_minimap_rect();
        if !minimap_rect.contains(position) {
            return;
        }
    }

    if automap_type == AutomapType::Transparent {
        set_half_transparent_pixel(out, position, color);
    } else {
        out.set_pixel(position, color);
    }
}
