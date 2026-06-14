//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Points in Rectangle Range Iterator
//!
//! Ported from Source/engine/points_in_rectangle_range.hpp

use super::{Point, Rectangle, Displacement};

/// 矩形范围内的点迭代器（行主序）
/// Row-major iteration: {0,0}, {1,0}, {2,0}, {0,1}, {1,1}, ...
pub struct PointsInRectangleRange {
    origin: Point,
    major_dimension: i32,
    major_index: i32,
    minor_index: i32,
    total: i32,
}

impl PointsInRectangleRange {
    pub fn new(rect: Rectangle) -> Self {
        Self {
            origin: rect.position,
            major_dimension: rect.size.width,
            major_index: 0,
            minor_index: 0,
            total: rect.size.width * rect.size.height,
        }
    }
}

impl Iterator for PointsInRectangleRange {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.major_index * self.major_dimension + self.minor_index;
        if index >= self.total {
            return None;
        }

        let result = self.origin + Displacement::new(self.minor_index, self.major_index);

        // Increment
        self.minor_index += 1;
        if self.minor_index >= self.major_dimension {
            self.major_index += 1;
            self.minor_index = 0;
        }

        Some(result)
    }
}

/// 矩形范围内的点迭代器（列主序）
/// Col-major iteration: {0,0}, {0,1}, {0,2}, {1,0}, {1,1}, ...
pub struct PointsInRectangleColMajor {
    origin: Point,
    major_dimension: i32,
    major_index: i32,
    minor_index: i32,
    total: i32,
}

impl PointsInRectangleColMajor {
    pub fn new(rect: Rectangle) -> Self {
        Self {
            origin: rect.position,
            major_dimension: rect.size.height,
            major_index: 0,
            minor_index: 0,
            total: rect.size.width * rect.size.height,
        }
    }
}

impl Iterator for PointsInRectangleColMajor {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.major_index * self.major_dimension + self.minor_index;
        if index >= self.total {
            return None;
        }

        // Col-major: x = majorIndex, y = minorIndex
        let result = self.origin + Displacement::new(self.major_index, self.minor_index);

        // Increment
        self.minor_index += 1;
        if self.minor_index >= self.major_dimension {
            self.major_index += 1;
            self.minor_index = 0;
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::size::Size;

    #[test]
    fn test_points_in_rectangle_row_major() {
        let rect = Rectangle::new(Point::new(0, 0), Size::new(2, 2));
        let points: Vec<Point> = PointsInRectangleRange::new(rect).collect();
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], Point::new(0, 0));
        assert_eq!(points[1], Point::new(1, 0));
        assert_eq!(points[2], Point::new(0, 1));
        assert_eq!(points[3], Point::new(1, 1));
    }

    #[test]
    fn test_points_in_rectangle_col_major() {
        let rect = Rectangle::new(Point::new(0, 0), Size::new(2, 2));
        let points: Vec<Point> = PointsInRectangleColMajor::new(rect).collect();
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], Point::new(0, 0));
        assert_eq!(points[1], Point::new(0, 1));
        assert_eq!(points[2], Point::new(1, 0));
        assert_eq!(points[3], Point::new(1, 1));
    }
}
