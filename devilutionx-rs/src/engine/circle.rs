//! Circle - 圆形几何
//!
//! 移植自 Source/engine/circle.hpp

use super::types::{Displacement, Point, Size, Rectangle};

/// 圆形
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Circle {
    /// 圆心位置
    pub position: Point,
    /// 半径
    pub radius: i32,
}

impl Circle {
    /// 创建新圆形
    pub const fn new(position: Point, radius: i32) -> Self {
        Self { position, radius }
    }

    /// 从圆心坐标和半径创建
    pub const fn from_coords(x: i32, y: i32, radius: i32) -> Self {
        Self {
            position: Point::new(x, y),
            radius,
        }
    }

    /// 检查点是否在圆内
    pub fn contains(&self, point: Point) -> bool {
        let diff = Displacement::new(
            point.x - self.position.x,
            point.y - self.position.y,
        );
        let x = diff.delta_x;
        let y = diff.delta_y;
        x * x + y * y < self.radius * self.radius
    }

    /// 检查点是否在圆上或圆内
    pub fn contains_inclusive(&self, point: Point) -> bool {
        let diff = Displacement::new(
            point.x - self.position.x,
            point.y - self.position.y,
        );
        let x = diff.delta_x;
        let y = diff.delta_y;
        x * x + y * y <= self.radius * self.radius
    }

    /// 计算到点的距离
    pub fn distance_to(&self, point: Point) -> f32 {
        let dx = (point.x - self.position.x) as f32;
        let dy = (point.y - self.position.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// 检查两个圆是否相交
    pub fn intersects(&self, other: &Circle) -> bool {
        let dx = (other.position.x - self.position.x) as f32;
        let dy = (other.position.y - self.position.y) as f32;
        let dist_sq = dx * dx + dy * dy;
        let radius_sum = (self.radius + other.radius) as f32;
        dist_sq < radius_sum * radius_sum
    }

    /// 获取圆的边界矩形
    pub fn bounding_rect(&self) -> Rectangle {
        Rectangle::new(
            Point::new(self.position.x - self.radius, self.position.y - self.radius),
            Size::new(self.radius * 2, self.radius * 2),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_contains() {
        let circle = Circle::new(Point::new(10, 10), 5);

        // 圆心在圆内
        assert!(circle.contains(Point::new(10, 10)));

        // 圆内的点
        assert!(circle.contains(Point::new(12, 12)));

        // 圆边上的点 (不包含)
        assert!(!circle.contains(Point::new(15, 10)));

        // 圆外的点
        assert!(!circle.contains(Point::new(20, 20)));
    }

    #[test]
    fn test_circle_contains_inclusive() {
        let circle = Circle::new(Point::new(10, 10), 5);

        // 圆边上的点 (包含)
        assert!(circle.contains_inclusive(Point::new(15, 10)));
        assert!(circle.contains_inclusive(Point::new(10, 15)));
    }

    #[test]
    fn test_circle_intersects() {
        let circle1 = Circle::new(Point::new(0, 0), 5);
        let circle2 = Circle::new(Point::new(8, 0), 5);
        let circle3 = Circle::new(Point::new(20, 0), 5);

        // 相交
        assert!(circle1.intersects(&circle2));

        // 不相交
        assert!(!circle1.intersects(&circle3));
    }

    #[test]
    fn test_circle_bounding_rect() {
        let circle = Circle::new(Point::new(10, 10), 5);
        let rect = circle.bounding_rect();

        assert_eq!(rect.position.x, 5);
        assert_eq!(rect.position.y, 5);
        assert_eq!(rect.size.width, 10);
        assert_eq!(rect.size.height, 10);
    }
}
