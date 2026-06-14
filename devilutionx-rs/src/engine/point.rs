//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Point - 2D coordinate type
//!
//! Ported from Source/engine/point.hpp

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg};
use super::displacement::Displacement;
use super::direction::Direction;

/// 点结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn from_displacement(d: Displacement) -> Self {
        Self {
            x: d.delta_x,
            y: d.delta_y,
        }
    }

    /// Fast approximate distance between two points, using only integer arithmetic, with less than ~5% error
    pub fn approx_distance(&self, other: Point) -> i32 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let (min, max) = if dx < dy { (dx, dy) } else { (dy, dx) };
        let mut approx = max * 1007 + min * 441;
        if max < min * 16 {
            approx -= max * 40;
        }
        (approx + 512) / 1024
    }

    /// Calculates the exact distance between two points
    pub fn exact_distance(&self, other: Point) -> i32 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// Manhattan distance (sum of absolute differences)
    #[inline]
    pub fn manhattan_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Walking distance (Chebyshev distance - max of absolute differences)
    #[inline]
    pub fn walking_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    /// Converts a coordinate in megatiles to the northmost of the 4 corresponding world tiles
    #[inline]
    pub const fn mega_to_world(&self) -> Point {
        Point::new(16 + 2 * self.x, 16 + 2 * self.y)
    }

    /// Converts a coordinate in world tiles back to the corresponding megatile
    #[inline]
    pub const fn world_to_mega(&self) -> Point {
        Point::new((self.x - 16) / 2, (self.y - 16) / 2)
    }
}

// Point + Displacement -> Point
impl Add<Displacement> for Point {
    type Output = Point;
    fn add(self, d: Displacement) -> Self::Output {
        Point::new(self.x + d.delta_x, self.y + d.delta_y)
    }
}

impl AddAssign<Displacement> for Point {
    fn add_assign(&mut self, d: Displacement) {
        self.x += d.delta_x;
        self.y += d.delta_y;
    }
}

// Point + Direction -> Point
impl Add<Direction> for Point {
    type Output = Point;
    fn add(self, dir: Direction) -> Self::Output {
        self + Displacement::from_direction(dir)
    }
}

impl AddAssign<Direction> for Point {
    fn add_assign(&mut self, dir: Direction) {
        *self += Displacement::from_direction(dir);
    }
}

// Point - Point -> Displacement
impl Sub for Point {
    type Output = Displacement;
    fn sub(self, other: Point) -> Self::Output {
        Displacement::new(self.x - other.x, self.y - other.y)
    }
}

// Point - Displacement -> Point
impl Sub<Displacement> for Point {
    type Output = Point;
    fn sub(self, d: Displacement) -> Self::Output {
        Point::new(self.x - d.delta_x, self.y - d.delta_y)
    }
}

impl SubAssign<Displacement> for Point {
    fn sub_assign(&mut self, d: Displacement) {
        self.x -= d.delta_x;
        self.y -= d.delta_y;
    }
}

// Point * int -> Point
impl Mul<i32> for Point {
    type Output = Point;
    fn mul(self, factor: i32) -> Self::Output {
        Point::new(self.x * factor, self.y * factor)
    }
}

impl MulAssign<i32> for Point {
    fn mul_assign(&mut self, factor: i32) {
        self.x *= factor;
        self.y *= factor;
    }
}

// Point * float -> Point
impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, factor: f32) -> Self::Output {
        Point::new((self.x as f32 * factor) as i32, (self.y as f32 * factor) as i32)
    }
}

impl MulAssign<f32> for Point {
    fn mul_assign(&mut self, factor: f32) {
        self.x = (self.x as f32 * factor) as i32;
        self.y = (self.y as f32 * factor) as i32;
    }
}

// Point / int -> Point
impl Div<i32> for Point {
    type Output = Point;
    fn div(self, factor: i32) -> Self::Output {
        Point::new(self.x / factor, self.y / factor)
    }
}

impl DivAssign<i32> for Point {
    fn div_assign(&mut self, factor: i32) {
        self.x /= factor;
        self.y /= factor;
    }
}

// -Point -> Point
impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Self::Output {
        Point::new(-self.x, -self.y)
    }
}

/// Calculate the best fit direction between two points
pub fn get_direction(start: Point, destination: Point) -> Direction {
    let mx = destination.x - start.x;
    let my = destination.y - start.y;
    
    let md;
    if mx >= 0 {
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return Direction::SouthWest;
            }
            md = Direction::South;
        } else {
            let my = -my;
            if 5 * mx <= my * 2 {
                return Direction::NorthEast;
            }
            md = Direction::East;
        }
        if 5 * my.abs() <= mx * 2 {
            return Direction::SouthEast;
        }
    } else {
        let mx = -mx;
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return Direction::SouthWest;
            }
            md = Direction::West;
        } else {
            let my = -my;
            if 5 * mx <= my * 2 {
                return Direction::NorthEast;
            }
            md = Direction::North;
        }
        if 5 * my.abs() <= mx * 2 {
            return Direction::NorthWest;
        }
    }
    md
}

/// Returns absolute value of point coordinates
#[inline]
pub fn abs(p: Point) -> Point {
    Point::new(p.x.abs(), p.y.abs())
}
