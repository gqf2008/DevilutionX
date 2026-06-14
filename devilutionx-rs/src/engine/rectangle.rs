//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Rectangle - 2D 矩形区域类型
//!
//! 移植自 Source/engine/rectangle.hpp

use super::displacement::Displacement;
use super::point::Point;
use super::size::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    /// 从位置和尺寸创建
    /// C++ API: `constexpr RectangleOf(PointOf<CoordT> position, SizeOf<SizeT> size)`
    #[inline]
    pub const fn new(position: Point, size: Size) -> Self {
        Self { position, size }
    }

    /// 从中心点和半径创建 (正方形)
    /// C++ API: `explicit constexpr RectangleOf(PointOf<CoordT> center, SizeT radius)`
    #[inline]
    pub const fn from_center(center: Point, radius: i32) -> Self {
        Self {
            position: Point::new(center.x - radius, center.y - radius),
            size: Size::uniform(2 * radius + 1),
        }
    }

    /// 检查点是否在矩形内
    /// C++ API: `bool contains(PointOf<PointCoordT> point) const`
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y < self.position.y + self.size.height
    }

    /// 计算矩形中心
    /// C++ API: `constexpr PointOf<CoordT> Center() const`
    pub const fn center(&self) -> Point {
        Point::new(
            self.position.x + self.size.width / 2,
            self.position.y + self.size.height / 2,
        )
    }

    /// 收缩矩形 (向内缩小边界)
    /// C++ API: `constexpr RectangleOf<CoordT, SizeT> inset(DisplacementOf<SizeT> factor) const`
    pub const fn inset(&self, factor: Displacement) -> Rectangle {
        Rectangle {
            position: Point::new(
                self.position.x + factor.delta_x,
                self.position.y + factor.delta_y,
            ),
            size: Size::new(
                self.size.width - factor.delta_x * 2,
                self.size.height - factor.delta_y * 2,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(Point::new(10, 10), Size::new(20, 20));
        assert!(rect.contains(Point::new(15, 15)));
        assert!(!rect.contains(Point::new(5, 5)));
        assert!(!rect.contains(Point::new(30, 30))); // 边界外
    }

    #[test]
    fn test_rectangle_center() {
        let rect = Rectangle::new(Point::new(10, 20), Size::new(100, 50));
        let c = rect.center();
        assert_eq!(c, Point::new(60, 45));
    }

    #[test]
    fn test_rectangle_inset() {
        let rect = Rectangle::new(Point::new(0, 0), Size::new(100, 100));
        let inset = rect.inset(Displacement::new(10, 10));
        assert_eq!(inset.position, Point::new(10, 10));
        assert_eq!(inset.size, Size::new(80, 80));
    }
}
