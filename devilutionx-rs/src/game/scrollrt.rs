//! Scroll/Render Types and Utilities
//!
//! This module contains core rendering types and coordinate transformation utilities
//! for the Diablo isometric projection system.
//!
//! # C++ Source Reference
//! - Source/engine/render/scrollrt.h
//! - Source/engine/render/scrollrt.cpp
//! - Source/engine/displacement.hpp
//! - Source/engine/point.hpp
//! - Source/engine/direction.hpp
//! - Source/engine/size.hpp
//!
//! # Coordinate Systems
//!
//! The game uses multiple coordinate systems:
//! - **World coordinates**: Tile-based grid coordinates (x, y)
//! - **Screen coordinates**: Pixel coordinates on screen
//! - **Isometric projection**: 2:1 aspect ratio diamond tiles (64x32 pixels)

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Direction enumeration for 8-directional movement
///
/// Uses clockwise ordering starting from South.
///
/// # C++ Reference
/// ```cpp
/// enum class Direction : std::uint8_t {
///     South, SouthWest, West, NorthWest,
///     North, NorthEast, East, SouthEast,
///     NoDirection
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Direction {
    /// South (down-right in isometric view)
    #[default]
    South = 0,
    /// South-West
    SouthWest = 1,
    /// West (down-left in isometric view)
    West = 2,
    /// North-West
    NorthWest = 3,
    /// North (up-left in isometric view)
    North = 4,
    /// North-East
    NorthEast = 5,
    /// East (up-right in isometric view)
    East = 6,
    /// South-East
    SouthEast = 7,
    /// No direction / invalid
    NoDirection = 8,
}

impl Direction {
    /// Returns the direction after turning left (counter-clockwise)
    pub fn left(&self) -> Direction {
        match self {
            Direction::South => Direction::SouthEast,
            Direction::SouthWest => Direction::South,
            Direction::West => Direction::SouthWest,
            Direction::NorthWest => Direction::West,
            Direction::North => Direction::NorthWest,
            Direction::NorthEast => Direction::North,
            Direction::East => Direction::NorthEast,
            Direction::SouthEast => Direction::East,
            Direction::NoDirection => Direction::NoDirection,
        }
    }

    /// Returns the direction after turning right (clockwise)
    pub fn right(&self) -> Direction {
        match self {
            Direction::South => Direction::SouthWest,
            Direction::SouthWest => Direction::West,
            Direction::West => Direction::NorthWest,
            Direction::NorthWest => Direction::North,
            Direction::North => Direction::NorthEast,
            Direction::NorthEast => Direction::East,
            Direction::East => Direction::SouthEast,
            Direction::SouthEast => Direction::South,
            Direction::NoDirection => Direction::NoDirection,
        }
    }

    /// Returns the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::NoDirection => Direction::NoDirection,
        }
    }

    /// Returns the direction name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Direction::South => "South",
            Direction::SouthWest => "SouthWest",
            Direction::West => "West",
            Direction::NorthWest => "NorthWest",
            Direction::North => "North",
            Direction::NorthEast => "NorthEast",
            Direction::East => "East",
            Direction::SouthEast => "SouthEast",
            Direction::NoDirection => "NoDirection",
        }
    }

    /// Returns the displacement for this direction
    pub fn to_displacement(&self) -> Displacement {
        match self {
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

    /// All cardinal and diagonal directions (excluding NoDirection)
    pub const ALL: [Direction; 8] = [
        Direction::South,
        Direction::SouthWest,
        Direction::West,
        Direction::NorthWest,
        Direction::North,
        Direction::NorthEast,
        Direction::East,
        Direction::SouthEast,
    ];
}

/// Size structure for dimensions
///
/// # C++ Reference
/// ```cpp
/// template <typename SizeT>
/// struct SizeOf {
///     SizeT width;
///     SizeT height;
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    /// Creates a new size
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Creates a square size
    pub const fn square(size: i32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    /// Zero size
    pub const ZERO: Size = Size {
        width: 0,
        height: 0,
    };

    /// Returns the area (width * height)
    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    /// Returns true if either dimension is zero or negative
    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

impl Mul<i32> for Size {
    type Output = Size;

    fn mul(self, factor: i32) -> Size {
        Size {
            width: self.width * factor,
            height: self.height * factor,
        }
    }
}

impl Div<i32> for Size {
    type Output = Size;

    fn div(self, factor: i32) -> Size {
        Size {
            width: self.width / factor,
            height: self.height / factor,
        }
    }
}

/// Displacement structure for offsets and vectors
///
/// Used for representing movement, offsets, and coordinate transformations.
///
/// # C++ Reference
/// ```cpp
/// template <typename DeltaT>
/// struct DisplacementOf {
///     DeltaT deltaX;
///     DeltaT deltaY;
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Displacement {
    pub delta_x: i32,
    pub delta_y: i32,
}

impl Displacement {
    /// Creates a new displacement
    pub const fn new(delta_x: i32, delta_y: i32) -> Self {
        Self { delta_x, delta_y }
    }

    /// Zero displacement
    pub const ZERO: Displacement = Displacement {
        delta_x: 0,
        delta_y: 0,
    };

    /// Creates a displacement from a Size
    pub fn from_size(size: Size) -> Self {
        Self {
            delta_x: size.width,
            delta_y: size.height,
        }
    }

    /// Calculates the magnitude (length) of the displacement
    pub fn magnitude(&self) -> f32 {
        ((self.delta_x * self.delta_x + self.delta_y * self.delta_y) as f32).sqrt()
    }

    /// Transforms from world coordinates to screen coordinates (isometric projection)
    ///
    /// Uses the rotation matrix for -135° with scaling for 64x32 pixel tiles:
    /// ```text
    /// [-32,  32] [dx]   [ 32(dy - dx)]
    /// [-16, -16] [dy] = [-16(dy + dx)]
    /// ```
    pub fn world_to_screen(&self) -> Displacement {
        Displacement {
            delta_x: (self.delta_y - self.delta_x) * 32,
            delta_y: (self.delta_y + self.delta_x) * -16,
        }
    }

    /// Transforms from screen coordinates to world coordinates
    ///
    /// Inverse of world_to_screen transformation.
    pub fn screen_to_world(&self) -> Displacement {
        Displacement {
            delta_x: (2 * self.delta_y + self.delta_x) / -64,
            delta_y: (2 * self.delta_y - self.delta_x) / -64,
        }
    }

    /// Transforms from screen coordinates to missile world coordinates
    ///
    /// Similar to screen_to_world but with rounding.
    pub fn screen_to_missile(&self) -> Displacement {
        let x_numerator = 2 * self.delta_y + self.delta_x;
        let y_numerator = 2 * self.delta_y - self.delta_x;
        let x_offset = if x_numerator >= 0 { 32 } else { -32 };
        let y_offset = if y_numerator >= 0 { 32 } else { -32 };
        Displacement {
            delta_x: (x_numerator + x_offset) / 64,
            delta_y: (y_numerator + y_offset) / 64,
        }
    }

    /// Transforms to light coordinates
    pub fn screen_to_light(&self) -> Displacement {
        Displacement {
            delta_x: (2 * self.delta_y + self.delta_x) / 8,
            delta_y: (2 * self.delta_y - self.delta_x) / 8,
        }
    }

    /// Rotates the displacement by 90° increments
    ///
    /// # Arguments
    /// * `quadrants` - Number of 90° rotations (positive = counter-clockwise)
    pub fn rotate(&self, quadrants: i32) -> Displacement {
        const SINES: [i32; 4] = [0, 1, 0, -1];
        let q = ((quadrants % 4) + 4) as usize % 4;
        let sine = SINES[q];
        let cosine = SINES[(q + 1) % 4];
        Displacement {
            delta_x: self.delta_x * cosine - self.delta_y * sine,
            delta_y: self.delta_x * sine + self.delta_y * cosine,
        }
    }

    /// Flips the X component
    pub fn flip_x(&self) -> Displacement {
        Displacement {
            delta_x: -self.delta_x,
            delta_y: self.delta_y,
        }
    }

    /// Flips the Y component
    pub fn flip_y(&self) -> Displacement {
        Displacement {
            delta_x: self.delta_x,
            delta_y: -self.delta_y,
        }
    }

    /// Flips both components
    pub fn flip_xy(&self) -> Displacement {
        Displacement {
            delta_x: -self.delta_x,
            delta_y: -self.delta_y,
        }
    }

    /// Returns absolute values of both components
    pub fn abs(&self) -> Displacement {
        Displacement {
            delta_x: self.delta_x.abs(),
            delta_y: self.delta_y.abs(),
        }
    }

    /// Returns a normalized displacement (16-bit fixed point)
    pub fn normalized(&self) -> Displacement {
        let mag = self.magnitude();
        if mag == 0.0 {
            return Displacement::ZERO;
        }
        Displacement {
            delta_x: ((self.delta_x << 16) as f32 / mag) as i32,
            delta_y: ((self.delta_y << 16) as f32 / mag) as i32,
        }
    }
}

impl Add for Displacement {
    type Output = Displacement;

    fn add(self, other: Displacement) -> Displacement {
        Displacement {
            delta_x: self.delta_x + other.delta_x,
            delta_y: self.delta_y + other.delta_y,
        }
    }
}

impl AddAssign for Displacement {
    fn add_assign(&mut self, other: Displacement) {
        self.delta_x += other.delta_x;
        self.delta_y += other.delta_y;
    }
}

impl Sub for Displacement {
    type Output = Displacement;

    fn sub(self, other: Displacement) -> Displacement {
        Displacement {
            delta_x: self.delta_x - other.delta_x,
            delta_y: self.delta_y - other.delta_y,
        }
    }
}

impl SubAssign for Displacement {
    fn sub_assign(&mut self, other: Displacement) {
        self.delta_x -= other.delta_x;
        self.delta_y -= other.delta_y;
    }
}

impl Mul<i32> for Displacement {
    type Output = Displacement;

    fn mul(self, factor: i32) -> Displacement {
        Displacement {
            delta_x: self.delta_x * factor,
            delta_y: self.delta_y * factor,
        }
    }
}

impl MulAssign<i32> for Displacement {
    fn mul_assign(&mut self, factor: i32) {
        self.delta_x *= factor;
        self.delta_y *= factor;
    }
}

impl Div<i32> for Displacement {
    type Output = Displacement;

    fn div(self, factor: i32) -> Displacement {
        Displacement {
            delta_x: self.delta_x / factor,
            delta_y: self.delta_y / factor,
        }
    }
}

impl DivAssign<i32> for Displacement {
    fn div_assign(&mut self, factor: i32) {
        self.delta_x /= factor;
        self.delta_y /= factor;
    }
}

impl Neg for Displacement {
    type Output = Displacement;

    fn neg(self) -> Displacement {
        Displacement {
            delta_x: -self.delta_x,
            delta_y: -self.delta_y,
        }
    }
}

impl From<Direction> for Displacement {
    fn from(dir: Direction) -> Self {
        dir.to_displacement()
    }
}

impl From<Size> for Displacement {
    fn from(size: Size) -> Self {
        Displacement::from_size(size)
    }
}

/// Point structure for coordinates
///
/// # C++ Reference
/// ```cpp
/// template <typename CoordT>
/// struct PointOf {
///     CoordT x;
///     CoordT y;
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Creates a new point
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Origin point (0, 0)
    pub const ZERO: Point = Point { x: 0, y: 0 };

    /// Fast approximate distance to another point
    ///
    /// Uses integer arithmetic with less than ~5% error.
    pub fn approx_distance(&self, other: Point) -> i32 {
        let offset = (*self - other).abs();
        let (min, max) = if offset.delta_x < offset.delta_y {
            (offset.delta_x, offset.delta_y)
        } else {
            (offset.delta_y, offset.delta_x)
        };

        let mut approx = max * 1007 + min * 441;
        if max < min * 16 {
            approx -= max * 40;
        }

        (approx + 512) / 1024
    }

    /// Exact distance to another point
    pub fn exact_distance(&self, other: Point) -> i32 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// Manhattan distance (sum of absolute differences)
    pub fn manhattan_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Walking distance (Chebyshev distance, max of absolute differences)
    pub fn walking_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    /// Converts megatile coordinates to world tile coordinates
    pub fn mega_to_world(&self) -> Point {
        Point {
            x: 16 + 2 * self.x,
            y: 16 + 2 * self.y,
        }
    }

    /// Converts world tile coordinates to megatile coordinates
    pub fn world_to_mega(&self) -> Point {
        Point {
            x: (self.x - 16) / 2,
            y: (self.y - 16) / 2,
        }
    }
}

impl Add<Displacement> for Point {
    type Output = Point;

    fn add(self, displacement: Displacement) -> Point {
        Point {
            x: self.x + displacement.delta_x,
            y: self.y + displacement.delta_y,
        }
    }
}

impl AddAssign<Displacement> for Point {
    fn add_assign(&mut self, displacement: Displacement) {
        self.x += displacement.delta_x;
        self.y += displacement.delta_y;
    }
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, direction: Direction) -> Point {
        self + direction.to_displacement()
    }
}

impl AddAssign<Direction> for Point {
    fn add_assign(&mut self, direction: Direction) {
        *self += direction.to_displacement();
    }
}

impl Sub<Displacement> for Point {
    type Output = Point;

    fn sub(self, displacement: Displacement) -> Point {
        Point {
            x: self.x - displacement.delta_x,
            y: self.y - displacement.delta_y,
        }
    }
}

impl SubAssign<Displacement> for Point {
    fn sub_assign(&mut self, displacement: Displacement) {
        self.x -= displacement.delta_x;
        self.y -= displacement.delta_y;
    }
}

impl Sub for Point {
    type Output = Displacement;

    fn sub(self, other: Point) -> Displacement {
        Displacement {
            delta_x: self.x - other.x,
            delta_y: self.y - other.y,
        }
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<i32> for Point {
    type Output = Point;

    fn mul(self, factor: i32) -> Point {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl Div<i32> for Point {
    type Output = Point;

    fn div(self, factor: i32) -> Point {
        Point {
            x: self.x / factor,
            y: self.y / factor,
        }
    }
}

/// Calculate the best fit direction between two points
pub fn get_direction(start: Point, destination: Point) -> Direction {
    let mx = destination.x - start.x;
    let my = destination.y - start.y;

    if mx >= 0 {
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return Direction::SouthWest;
            }
            if 5 * my <= mx * 2 {
                return Direction::SouthEast;
            }
            Direction::South
        } else {
            let my = -my;
            if 5 * mx <= my * 2 {
                return Direction::NorthEast;
            }
            if 5 * my <= mx * 2 {
                return Direction::SouthEast;
            }
            Direction::East
        }
    } else {
        let mx = -mx;
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return Direction::SouthWest;
            }
            if 5 * my <= mx * 2 {
                return Direction::NorthWest;
            }
            Direction::West
        } else {
            let my = -my;
            if 5 * mx <= my * 2 {
                return Direction::NorthEast;
            }
            if 5 * my <= mx * 2 {
                return Direction::NorthWest;
            }
            Direction::North
        }
    }
}

/// Rectangle structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    /// Creates a new rectangle
    pub const fn new(position: Point, size: Size) -> Self {
        Self { position, size }
    }

    /// Creates a rectangle from coordinates and dimensions
    pub const fn from_coords(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            position: Point { x, y },
            size: Size { width, height },
        }
    }

    /// Empty rectangle at origin
    pub const ZERO: Rectangle = Rectangle {
        position: Point::ZERO,
        size: Size::ZERO,
    };

    /// Returns true if the rectangle is empty
    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }

    /// Returns true if the point is inside the rectangle
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y < self.position.y + self.size.height
    }

    /// Returns the right edge x coordinate
    pub fn right(&self) -> i32 {
        self.position.x + self.size.width
    }

    /// Returns the bottom edge y coordinate
    pub fn bottom(&self) -> i32 {
        self.position.y + self.size.height
    }
}

/// Dungeon frame dimensions
pub mod frame {
    /// Width of a dungeon frame/tile in pixels
    pub const DUN_FRAME_WIDTH: i32 = 64;

    /// Height of a dungeon frame/tile in pixels
    pub const DUN_FRAME_HEIGHT: i32 = 32;

    /// Right frame displacement
    pub const RIGHT_FRAME_DISPLACEMENT: super::Displacement =
        super::Displacement::new(DUN_FRAME_WIDTH, 0);
}

/// Viewport/rendering state
#[derive(Debug, Clone, Default)]
pub struct ViewportState {
    /// Whether to show items on automap
    pub auto_map_show_items: bool,

    /// Frame flag for rendering
    pub frame_flag: bool,

    /// Previous cursor rectangle (for undraw)
    pub prev_cursor_rect: Rectangle,

    /// Last FPS update timestamp in milliseconds
    pub last_fps_update_ms: u32,
}

impl ViewportState {
    /// Creates a new viewport state
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_values() {
        assert_eq!(Direction::South as u8, 0);
        assert_eq!(Direction::SouthWest as u8, 1);
        assert_eq!(Direction::West as u8, 2);
        assert_eq!(Direction::NorthWest as u8, 3);
        assert_eq!(Direction::North as u8, 4);
        assert_eq!(Direction::NorthEast as u8, 5);
        assert_eq!(Direction::East as u8, 6);
        assert_eq!(Direction::SouthEast as u8, 7);
        assert_eq!(Direction::NoDirection as u8, 8);
    }

    #[test]
    fn test_direction_left() {
        assert_eq!(Direction::South.left(), Direction::SouthEast);
        assert_eq!(Direction::East.left(), Direction::NorthEast);
        assert_eq!(Direction::North.left(), Direction::NorthWest);
        assert_eq!(Direction::West.left(), Direction::SouthWest);
    }

    #[test]
    fn test_direction_right() {
        assert_eq!(Direction::South.right(), Direction::SouthWest);
        assert_eq!(Direction::East.right(), Direction::SouthEast);
        assert_eq!(Direction::North.right(), Direction::NorthEast);
        assert_eq!(Direction::West.right(), Direction::NorthWest);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::NorthWest.opposite(), Direction::SouthEast);
    }

    #[test]
    fn test_direction_to_displacement() {
        assert_eq!(Direction::South.to_displacement(), Displacement::new(1, 1));
        assert_eq!(Direction::North.to_displacement(), Displacement::new(-1, -1));
        assert_eq!(Direction::East.to_displacement(), Displacement::new(1, -1));
        assert_eq!(Direction::West.to_displacement(), Displacement::new(-1, 1));
        assert_eq!(
            Direction::NoDirection.to_displacement(),
            Displacement::new(0, 0)
        );
    }

    #[test]
    fn test_size_basic() {
        let s = Size::new(10, 20);
        assert_eq!(s.width, 10);
        assert_eq!(s.height, 20);
        assert_eq!(s.area(), 200);
        assert!(!s.is_empty());

        assert!(Size::ZERO.is_empty());
        assert!(Size::new(0, 10).is_empty());
    }

    #[test]
    fn test_size_arithmetic() {
        let s = Size::new(10, 20);
        assert_eq!(s * 2, Size::new(20, 40));
        assert_eq!(s / 2, Size::new(5, 10));
    }

    #[test]
    fn test_displacement_basic() {
        let d = Displacement::new(3, 4);
        assert_eq!(d.delta_x, 3);
        assert_eq!(d.delta_y, 4);
        assert!((d.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_displacement_world_to_screen() {
        // Test isometric projection
        let d = Displacement::new(1, 0);
        let screen = d.world_to_screen();
        // (0 - 1) * 32 = -32, (0 + 1) * -16 = -16
        assert_eq!(screen.delta_x, -32);
        assert_eq!(screen.delta_y, -16);

        let d2 = Displacement::new(0, 1);
        let screen2 = d2.world_to_screen();
        // (1 - 0) * 32 = 32, (1 + 0) * -16 = -16
        assert_eq!(screen2.delta_x, 32);
        assert_eq!(screen2.delta_y, -16);
    }

    #[test]
    fn test_displacement_rotate() {
        let d = Displacement::new(1, 0);

        let r1 = d.rotate(1); // 90° CCW
        assert_eq!(r1.delta_x, 0);
        assert_eq!(r1.delta_y, 1);

        let r2 = d.rotate(2); // 180°
        assert_eq!(r2.delta_x, -1);
        assert_eq!(r2.delta_y, 0);
    }

    #[test]
    fn test_displacement_arithmetic() {
        let d1 = Displacement::new(10, 20);
        let d2 = Displacement::new(3, 4);

        assert_eq!(d1 + d2, Displacement::new(13, 24));
        assert_eq!(d1 - d2, Displacement::new(7, 16));
        assert_eq!(d1 * 2, Displacement::new(20, 40));
        assert_eq!(d1 / 2, Displacement::new(5, 10));
        assert_eq!(-d1, Displacement::new(-10, -20));
    }

    #[test]
    fn test_displacement_flips() {
        let d = Displacement::new(3, 5);
        assert_eq!(d.flip_x(), Displacement::new(-3, 5));
        assert_eq!(d.flip_y(), Displacement::new(3, -5));
        assert_eq!(d.flip_xy(), Displacement::new(-3, -5));
    }

    #[test]
    fn test_point_basic() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    #[test]
    fn test_point_with_displacement() {
        let p = Point::new(10, 20);
        let d = Displacement::new(5, 3);

        assert_eq!(p + d, Point::new(15, 23));
        assert_eq!(p - d, Point::new(5, 17));
    }

    #[test]
    fn test_point_with_direction() {
        let p = Point::new(10, 10);

        assert_eq!(p + Direction::South, Point::new(11, 11));
        assert_eq!(p + Direction::North, Point::new(9, 9));
        assert_eq!(p + Direction::East, Point::new(11, 9));
        assert_eq!(p + Direction::West, Point::new(9, 11));
    }

    #[test]
    fn test_point_subtraction() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(3, 5);

        let d = p1 - p2;
        assert_eq!(d.delta_x, 7);
        assert_eq!(d.delta_y, 15);
    }

    #[test]
    fn test_point_distances() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);

        // Exact distance should be 5
        assert_eq!(p1.exact_distance(p2), 5);

        // Manhattan distance should be 7
        assert_eq!(p1.manhattan_distance(p2), 7);

        // Walking distance should be 4 (max of 3, 4)
        assert_eq!(p1.walking_distance(p2), 4);

        // Approximate distance should be close to 5
        let approx = p1.approx_distance(p2);
        assert!((approx - 5).abs() <= 1);
    }

    #[test]
    fn test_point_mega_world_conversion() {
        let mega = Point::new(5, 10);
        let world = mega.mega_to_world();
        assert_eq!(world, Point::new(26, 36));

        let back = world.world_to_mega();
        assert_eq!(back, mega);
    }

    #[test]
    fn test_get_direction() {
        let center = Point::new(10, 10);

        // Cardinal directions
        assert_eq!(
            get_direction(center, Point::new(10, 20)),
            Direction::SouthWest
        );
        assert_eq!(
            get_direction(center, Point::new(10, 0)),
            Direction::NorthEast
        );
        assert_eq!(
            get_direction(center, Point::new(20, 10)),
            Direction::SouthEast
        );
        assert_eq!(
            get_direction(center, Point::new(0, 10)),
            Direction::NorthWest
        );

        // Diagonal directions
        assert_eq!(get_direction(center, Point::new(20, 20)), Direction::South);
        assert_eq!(get_direction(center, Point::new(0, 0)), Direction::North);
    }

    #[test]
    fn test_rectangle_basic() {
        let r = Rectangle::from_coords(10, 20, 100, 50);
        assert_eq!(r.position, Point::new(10, 20));
        assert_eq!(r.size, Size::new(100, 50));
        assert_eq!(r.right(), 110);
        assert_eq!(r.bottom(), 70);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_rectangle_contains() {
        let r = Rectangle::from_coords(10, 10, 20, 20);

        assert!(r.contains(Point::new(15, 15)));
        assert!(r.contains(Point::new(10, 10)));
        assert!(!r.contains(Point::new(30, 30)));
        assert!(!r.contains(Point::new(5, 15)));
    }

    #[test]
    fn test_frame_constants() {
        assert_eq!(frame::DUN_FRAME_WIDTH, 64);
        assert_eq!(frame::DUN_FRAME_HEIGHT, 32);
    }
}
