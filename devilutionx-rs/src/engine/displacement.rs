//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Displacement - 2D vector/offset type
//!
//! Ported from Source/engine/displacement.hpp

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shl, Shr, Sub, SubAssign};
use super::size::Size;
use super::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Displacement {
    pub delta_x: i32,
    pub delta_y: i32,
}

impl Displacement {
    #[inline]
    pub const fn new(delta_x: i32, delta_y: i32) -> Self {
        Self { delta_x, delta_y }
    }

    #[inline]
    pub const fn uniform(delta: i32) -> Self {
        Self {
            delta_x: delta,
            delta_y: delta,
        }
    }

    #[inline]
    pub const fn from_size(size: Size) -> Self {
        Self {
            delta_x: size.width,
            delta_y: size.height,
        }
    }

    #[inline]
    pub const fn from_direction(dir: Direction) -> Self {
        match dir {
            Direction::South => Displacement::new(1, 1),
            Direction::SouthWest => Displacement::new(0, 1),
            Direction::West => Displacement::new(-1, 1),
            Direction::NorthWest => Displacement::new(-1, 0),
            Direction::North => Displacement::new(-1, -1),
            Direction::NorthEast => Displacement::new(0, -1),
            Direction::East => Displacement::new(1, -1),
            Direction::SouthEast => Displacement::new(1, 0),
            Direction::NoDirection => Displacement::new(0, 0),
        }
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        ((self.delta_x * self.delta_x + self.delta_y * self.delta_y) as f32).sqrt()
    }

    /// Returns a new Displacement object in screen coordinates.
    #[inline]
    pub const fn world_to_screen(&self) -> Displacement {
        Displacement::new(
            (self.delta_y - self.delta_x) * 32,
            (self.delta_y + self.delta_x) * -16,
        )
    }

    /// Returns a new Displacement object in world coordinates.
    #[inline]
    pub const fn screen_to_world(&self) -> Displacement {
        Displacement::new(
            (2 * self.delta_y + self.delta_x) / -64,
            (2 * self.delta_y - self.delta_x) / -64,
        )
    }

    /// Missiles flip the axes for some reason
    pub const fn screen_to_missile(&self) -> Displacement {
        let x_numerator = 2 * self.delta_y + self.delta_x;
        let y_numerator = 2 * self.delta_y - self.delta_x;
        let x_offset = if x_numerator >= 0 { 32 } else { -32 };
        let y_offset = if y_numerator >= 0 { 32 } else { -32 };
        Displacement::new(
            (x_numerator + x_offset) / 64,
            (y_numerator + y_offset) / 64,
        )
    }

    pub const fn screen_to_light(&self) -> Displacement {
        Displacement::new(
            (2 * self.delta_y + self.delta_x) / 8,
            (2 * self.delta_y - self.delta_x) / 8,
        )
    }

    /// Returns a 16 bit fixed point normalised displacement in isometric projection
    pub fn world_to_normal_screen(&self) -> Displacement {
        let rotated = Displacement::new(
            self.delta_y - self.delta_x,
            -(self.delta_y + self.delta_x),
        );
        let rotated_and_normalized = rotated.normalized();
        Displacement::new(
            rotated_and_normalized.delta_x,
            rotated_and_normalized.delta_y / 2,
        )
    }

    /// Calculates a 16 bit fixed point normalized displacement (having magnitude of ~1.0)
    pub fn normalized(&self) -> Displacement {
        let mag = self.magnitude();
        let shifted = *self << 16;
        Displacement::new(
            (shifted.delta_x as f32 / mag) as i32,
            (shifted.delta_y as f32 / mag) as i32,
        )
    }

    pub const fn rotate(&self, quadrants: i32) -> Displacement {
        const SINES: [i32; 4] = [0, 1, 0, -1];
        let q = ((quadrants % 4) + 4) % 4;
        let sine = SINES[q as usize];
        let cosine = SINES[((q + 1) % 4) as usize];
        Displacement::new(
            self.delta_x * cosine - self.delta_y * sine,
            self.delta_x * sine + self.delta_y * cosine,
        )
    }

    #[inline]
    pub const fn flip_x(&self) -> Displacement {
        Displacement::new(-self.delta_x, self.delta_y)
    }

    #[inline]
    pub const fn flip_y(&self) -> Displacement {
        Displacement::new(self.delta_x, -self.delta_y)
    }

    #[inline]
    pub const fn flip_xy(&self) -> Displacement {
        Displacement::new(-self.delta_x, -self.delta_y)
    }
}

// Displacement + Displacement
impl Add for Displacement {
    type Output = Displacement;
    fn add(self, other: Displacement) -> Self::Output {
        Displacement::new(self.delta_x + other.delta_x, self.delta_y + other.delta_y)
    }
}

impl AddAssign for Displacement {
    fn add_assign(&mut self, other: Displacement) {
        self.delta_x += other.delta_x;
        self.delta_y += other.delta_y;
    }
}

// Displacement - Displacement
impl Sub for Displacement {
    type Output = Displacement;
    fn sub(self, other: Displacement) -> Self::Output {
        Displacement::new(self.delta_x - other.delta_x, self.delta_y - other.delta_y)
    }
}

impl SubAssign for Displacement {
    fn sub_assign(&mut self, other: Displacement) {
        self.delta_x -= other.delta_x;
        self.delta_y -= other.delta_y;
    }
}

// Displacement * i32
impl Mul<i32> for Displacement {
    type Output = Displacement;
    fn mul(self, factor: i32) -> Self::Output {
        Displacement::new(self.delta_x * factor, self.delta_y * factor)
    }
}

impl MulAssign<i32> for Displacement {
    fn mul_assign(&mut self, factor: i32) {
        self.delta_x *= factor;
        self.delta_y *= factor;
    }
}

// Displacement * f32
impl Mul<f32> for Displacement {
    type Output = Displacement;
    fn mul(self, factor: f32) -> Self::Output {
        Displacement::new(
            (self.delta_x as f32 * factor) as i32,
            (self.delta_y as f32 * factor) as i32,
        )
    }
}

impl MulAssign<f32> for Displacement {
    fn mul_assign(&mut self, factor: f32) {
        self.delta_x = (self.delta_x as f32 * factor) as i32;
        self.delta_y = (self.delta_y as f32 * factor) as i32;
    }
}

// Displacement * Displacement (component-wise)
impl Mul<Displacement> for Displacement {
    type Output = Displacement;
    fn mul(self, factor: Displacement) -> Self::Output {
        Displacement::new(
            self.delta_x * factor.delta_x,
            self.delta_y * factor.delta_y,
        )
    }
}

impl MulAssign<Displacement> for Displacement {
    fn mul_assign(&mut self, factor: Displacement) {
        self.delta_x *= factor.delta_x;
        self.delta_y *= factor.delta_y;
    }
}

// Displacement / i32
impl Div<i32> for Displacement {
    type Output = Displacement;
    fn div(self, factor: i32) -> Self::Output {
        Displacement::new(self.delta_x / factor, self.delta_y / factor)
    }
}

impl DivAssign<i32> for Displacement {
    fn div_assign(&mut self, factor: i32) {
        self.delta_x /= factor;
        self.delta_y /= factor;
    }
}

// Displacement / f32
impl Div<f32> for Displacement {
    type Output = Displacement;
    fn div(self, factor: f32) -> Self::Output {
        Displacement::new(
            (self.delta_x as f32 / factor) as i32,
            (self.delta_y as f32 / factor) as i32,
        )
    }
}

impl DivAssign<f32> for Displacement {
    fn div_assign(&mut self, factor: f32) {
        self.delta_x = (self.delta_x as f32 / factor) as i32;
        self.delta_y = (self.delta_y as f32 / factor) as i32;
    }
}

// -Displacement
impl Neg for Displacement {
    type Output = Displacement;
    fn neg(self) -> Self::Output {
        Displacement::new(-self.delta_x, -self.delta_y)
    }
}

// Displacement << u32
impl Shl<u32> for Displacement {
    type Output = Displacement;
    fn shl(self, factor: u32) -> Self::Output {
        Displacement::new(self.delta_x << factor, self.delta_y << factor)
    }
}

// Displacement >> u32
impl Shr<u32> for Displacement {
    type Output = Displacement;
    fn shr(self, factor: u32) -> Self::Output {
        Displacement::new(self.delta_x >> factor, self.delta_y >> factor)
    }
}

/// Returns absolute value of displacement
#[inline]
pub fn abs(d: Displacement) -> Displacement {
    Displacement::new(d.delta_x.abs(), d.delta_y.abs())
}
