//! 自动地图渲染 - 移植自 Source/engine/render/automap_render.cpp
//!
//! 提供自动地图线条绘制功能，支持等距投影的斜线

use super::primitive_render::{set_half_transparent_pixel, TransparencyLookup};
use super::surface::Surface;
use super::types::Point;

/// 自动地图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomapType {
    /// 完全不透明
    #[default]
    Opaque,
    /// 半透明
    Transparent,
    /// 小地图
    Minimap,
}

/// 自动地图渲染上下文
pub struct AutomapRenderer {
    /// 自动地图类型
    pub automap_type: AutomapType,
    /// 小地图裁剪区域
    pub minimap_rect: Option<super::types::Rectangle>,
    /// 透明度查找表
    pub transparency_lookup: TransparencyLookup,
}

impl AutomapRenderer {
    pub fn new() -> Self {
        Self {
            automap_type: AutomapType::Opaque,
            minimap_rect: None,
            transparency_lookup: TransparencyLookup::new(),
        }
    }

    /// 设置地图像素
    pub fn set_map_pixel(&self, out: &mut Surface, position: Point, color: u8) {
        // 小地图裁剪检查
        if self.automap_type == AutomapType::Minimap {
            if let Some(rect) = &self.minimap_rect {
                if !rect.contains(position) {
                    return;
                }
            }
        }

        if self.automap_type == AutomapType::Transparent {
            set_half_transparent_pixel(out, position, color, &self.transparency_lookup);
        } else {
            out.set_pixel(position, color);
        }
    }

    /// 绘制南北方向线 (垂直)
    pub fn draw_map_line_ns(&self, out: &mut Surface, from: Point, height: i32, color_index: u8) {
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

        for i in 0..height {
            self.set_map_pixel(out, Point::new(from.x, from.y + i), color_index);
        }
    }

    /// 绘制东西方向线 (水平)
    pub fn draw_map_line_we(&self, out: &mut Surface, from: Point, width: i32, color_index: u8) {
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

        for i in 0..width {
            self.set_map_pixel(out, Point::new(from.x + i, from.y), color_index);
        }
    }

    /// 绘制东北方向线 (atan(1/2) 角度)
    ///
    /// 每垂直移动一个像素，水平移动两个像素
    /// 终点: { from.x + 2 * height + 1, from.y - height }
    pub fn draw_map_line_ne(&self, out: &mut Surface, from: Point, height: i32, color_index: u8) {
        self.draw_map_line_internal(out, from, height, color_index, 1, -1);
    }

    /// 绘制东南方向线
    /// 终点: { from.x + 2 * height + 1, from.y + height }
    pub fn draw_map_line_se(&self, out: &mut Surface, from: Point, height: i32, color_index: u8) {
        self.draw_map_line_internal(out, from, height, color_index, 1, 1);
    }

    /// 绘制西北方向线
    /// 终点: { from.x - 2 * height + 1, from.y - height }
    pub fn draw_map_line_nw(&self, out: &mut Surface, from: Point, height: i32, color_index: u8) {
        self.draw_map_line_internal(out, from, height, color_index, -1, -1);
    }

    /// 绘制西南方向线
    /// 终点: { from.x - 2 * height + 1, from.y + height }
    pub fn draw_map_line_sw(&self, out: &mut Surface, from: Point, height: i32, color_index: u8) {
        self.draw_map_line_internal(out, from, height, color_index, -1, 1);
    }

    /// 内部方法：绘制等距投影线 (2:1 比例)
    fn draw_map_line_internal(
        &self,
        out: &mut Surface,
        mut from: Point,
        mut height: i32,
        color_index: u8,
        dir_x: i32,
        dir_y: i32,
    ) {
        while height > 0 {
            height -= 1;

            // 绘制阴影像素
            self.set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
            // 绘制主像素
            self.set_map_pixel(out, from, color_index);

            from.x += dir_x;

            self.set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
            self.set_map_pixel(out, from, color_index);

            from.x += dir_x;
            from.y += dir_y;
        }

        // 最后一个像素
        self.set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
        self.set_map_pixel(out, from, color_index);
    }

    /// 绘制陡峭的东北方向线 (atan(2) 角度)
    ///
    /// 每水平移动一个像素，垂直移动两个像素
    /// 终点: { from.x + width + 1, from.y - 2 * width }
    pub fn draw_map_line_steep_ne(
        &self,
        out: &mut Surface,
        from: Point,
        width: i32,
        color_index: u8,
    ) {
        self.draw_map_line_steep_internal(out, from, width, color_index, 1, -1);
    }

    /// 绘制陡峭的东南方向线
    /// 终点: { from.x + width + 1, from.y + 2 * width }
    pub fn draw_map_line_steep_se(
        &self,
        out: &mut Surface,
        from: Point,
        width: i32,
        color_index: u8,
    ) {
        self.draw_map_line_steep_internal(out, from, width, color_index, 1, 1);
    }

    /// 绘制陡峭的西北方向线
    /// 终点: { from.x - (width + 1), from.y - 2 * width }
    pub fn draw_map_line_steep_nw(
        &self,
        out: &mut Surface,
        from: Point,
        width: i32,
        color_index: u8,
    ) {
        self.draw_map_line_steep_internal(out, from, width, color_index, -1, -1);
    }

    /// 绘制陡峭的西南方向线
    /// 终点: { from.x - (width + 1), from.y + 2 * width }
    pub fn draw_map_line_steep_sw(
        &self,
        out: &mut Surface,
        from: Point,
        width: i32,
        color_index: u8,
    ) {
        self.draw_map_line_steep_internal(out, from, width, color_index, -1, 1);
    }

    /// 内部方法：绘制陡峭线 (1:2 比例)
    fn draw_map_line_steep_internal(
        &self,
        out: &mut Surface,
        mut from: Point,
        mut width: i32,
        color_index: u8,
        dir_x: i32,
        dir_y: i32,
    ) {
        while width > 0 {
            width -= 1;

            self.set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
            self.set_map_pixel(out, from, color_index);

            from.y += dir_y;

            self.set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
            self.set_map_pixel(out, from, color_index);

            from.y += dir_y;
            from.x += dir_x;
        }

        self.set_map_pixel(out, Point::new(from.x, from.y + 1), 0);
        self.set_map_pixel(out, from, color_index);
    }

    /// 绘制自由直线 (Bresenham 算法，不包含阴影)
    pub fn draw_map_free_line(
        &self,
        out: &mut Surface,
        mut from: Point,
        to: Point,
        color_index: u8,
    ) {
        let dx = (to.x - from.x).abs();
        let dy = (to.y - from.y).abs();
        let sx = if from.x < to.x { 1 } else { -1 };
        let sy = if from.y < to.y { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            self.set_map_pixel(out, from, color_index);

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
}

impl Default for AutomapRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// 独立函数版本 (不需要 AutomapRenderer 实例)

/// 绘制南北方向线
pub fn draw_map_line_ns(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_line_ns(out, from, height, color_index);
}

/// 绘制东西方向线
pub fn draw_map_line_we(out: &mut Surface, from: Point, width: i32, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_line_we(out, from, width, color_index);
}

/// 绘制东北方向线
pub fn draw_map_line_ne(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_line_ne(out, from, height, color_index);
}

/// 绘制东南方向线
pub fn draw_map_line_se(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_line_se(out, from, height, color_index);
}

/// 绘制西北方向线
pub fn draw_map_line_nw(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_line_nw(out, from, height, color_index);
}

/// 绘制西南方向线
pub fn draw_map_line_sw(out: &mut Surface, from: Point, height: i32, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_line_sw(out, from, height, color_index);
}

/// 绘制自由直线
pub fn draw_map_free_line(out: &mut Surface, from: Point, to: Point, color_index: u8) {
    let renderer = AutomapRenderer::new();
    renderer.draw_map_free_line(out, from, to, color_index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automap_renderer_creation() {
        let renderer = AutomapRenderer::new();
        assert_eq!(renderer.automap_type, AutomapType::Opaque);
        assert!(renderer.minimap_rect.is_none());
    }

    #[test]
    fn test_draw_map_line_ns() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        draw_map_line_ns(&mut surface, Point::new(50, 10), 20, 5);

        // 检查垂直线
        for y in 10..30 {
            assert_eq!(surface.row(y).unwrap()[50], 5);
        }
    }

    #[test]
    fn test_draw_map_line_we() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        draw_map_line_we(&mut surface, Point::new(10, 50), 20, 7);

        // 检查水平线
        let row = surface.row(50).unwrap();
        for x in 10..30 {
            assert_eq!(row[x], 7);
        }
    }

    #[test]
    fn test_draw_map_free_line() {
        let mut data = vec![0u8; 100 * 100];
        let mut surface = Surface::new(&mut data, 100, 100, 100);

        // 绘制对角线
        draw_map_free_line(&mut surface, Point::new(10, 10), Point::new(20, 20), 3);

        // 起点和终点应该被绘制
        assert_eq!(surface.row(10).unwrap()[10], 3);
        assert_eq!(surface.row(20).unwrap()[20], 3);
    }
}
