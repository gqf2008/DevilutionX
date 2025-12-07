//! 基础类型定义 - 移植自 Source/engine/*.hpp
//!
//! 包含 Point, Size, Displacement, Rectangle, Direction 等基础几何类型

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 方向枚举 (8方向 + 无方向)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Direction {
    South = 0,
    SouthWest = 1,
    West = 2,
    NorthWest = 3,
    North = 4,
    NorthEast = 5,
    East = 6,
    SouthEast = 7,
    #[default]
    NoDirection = 8,
}

impl Direction {
    /// 左转
    #[inline]
    pub const fn left(self) -> Direction {
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

    /// 右转
    #[inline]
    pub const fn right(self) -> Direction {
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

    /// 反向
    #[inline]
    pub const fn opposite(self) -> Direction {
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

    /// 转换为位移
    #[inline]
    pub const fn to_displacement(self) -> Displacement {
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

    /// 从索引转换
    pub const fn from_index(index: u8) -> Option<Direction> {
        match index {
            0 => Some(Direction::South),
            1 => Some(Direction::SouthWest),
            2 => Some(Direction::West),
            3 => Some(Direction::NorthWest),
            4 => Some(Direction::North),
            5 => Some(Direction::NorthEast),
            6 => Some(Direction::East),
            7 => Some(Direction::SouthEast),
            8 => Some(Direction::NoDirection),
            _ => None,
        }
    }

    /// 转换为索引
    pub const fn to_index(self) -> u8 {
        self as u8
    }
}

/// 尺寸结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    #[inline]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub const fn square(size: i32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

impl Mul<i32> for Size {
    type Output = Size;
    fn mul(self, factor: i32) -> Self::Output {
        Size::new(self.width * factor, self.height * factor)
    }
}

impl Div<i32> for Size {
    type Output = Size;
    fn div(self, factor: i32) -> Self::Output {
        Size::new(self.width / factor, self.height / factor)
    }
}

/// 位移结构体 (向量)
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

    /// 从尺寸创建
    #[inline]
    pub const fn from_size(size: Size) -> Self {
        Self {
            delta_x: size.width,
            delta_y: size.height,
        }
    }

    /// 从方向创建
    #[inline]
    pub const fn from_direction(dir: Direction) -> Self {
        dir.to_displacement()
    }

    /// 计算向量长度
    #[inline]
    pub fn magnitude(&self) -> f32 {
        ((self.delta_x * self.delta_x + self.delta_y * self.delta_y) as f32).sqrt()
    }

    /// 世界坐标转屏幕坐标
    ///
    /// 等距投影变换:
    /// screen_x = (delta_y - delta_x) * 32
    /// screen_y = (delta_y + delta_x) * -16
    #[inline]
    pub const fn world_to_screen(&self) -> Displacement {
        Displacement::new(
            (self.delta_y - self.delta_x) * 32,
            (self.delta_y + self.delta_x) * -16,
        )
    }

    /// 屏幕坐标转世界坐标
    #[inline]
    pub const fn screen_to_world(&self) -> Displacement {
        Displacement::new(
            (2 * self.delta_y + self.delta_x) / -64,
            (2 * self.delta_y - self.delta_x) / -64,
        )
    }

    /// 旋转 (quadrants: 0-3 表示 0°, 90°, 180°, 270°)
    pub const fn rotate(&self, quadrants: i32) -> Displacement {
        let q = ((quadrants % 4) + 4) % 4;
        match q {
            0 => *self,
            1 => Displacement::new(-self.delta_y, self.delta_x),
            2 => Displacement::new(-self.delta_x, -self.delta_y),
            3 => Displacement::new(self.delta_y, -self.delta_x),
            _ => *self,
        }
    }

    /// X轴翻转
    #[inline]
    pub const fn flip_x(&self) -> Displacement {
        Displacement::new(-self.delta_x, self.delta_y)
    }

    /// Y轴翻转
    #[inline]
    pub const fn flip_y(&self) -> Displacement {
        Displacement::new(self.delta_x, -self.delta_y)
    }

    /// 取绝对值
    #[inline]
    pub fn abs(&self) -> Displacement {
        Displacement::new(self.delta_x.abs(), self.delta_y.abs())
    }
}

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

impl Neg for Displacement {
    type Output = Displacement;
    fn neg(self) -> Self::Output {
        Displacement::new(-self.delta_x, -self.delta_y)
    }
}

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

    /// 从位移创建
    #[inline]
    pub const fn from_displacement(d: Displacement) -> Self {
        Self {
            x: d.delta_x,
            y: d.delta_y,
        }
    }

    /// 转换为位移
    #[inline]
    pub const fn to_displacement(&self) -> Displacement {
        Displacement::new(self.x, self.y)
    }

    /// 快速近似距离 (误差 < 5%)
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

    /// 精确距离
    pub fn exact_distance(&self, other: Point) -> i32 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// 曼哈顿距离
    #[inline]
    pub fn manhattan_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// 行走距离 (切比雪夫距离)
    #[inline]
    pub fn walking_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    /// mega-tile 坐标转世界瓦片坐标
    #[inline]
    pub const fn mega_to_world(&self) -> Point {
        Point::new(16 + 2 * self.x, 16 + 2 * self.y)
    }

    /// 世界瓦片坐标转 mega-tile 坐标
    #[inline]
    pub const fn world_to_mega(&self) -> Point {
        Point::new((self.x - 16) / 2, (self.y - 16) / 2)
    }
}

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

impl Add<Direction> for Point {
    type Output = Point;
    fn add(self, dir: Direction) -> Self::Output {
        self + dir.to_displacement()
    }
}

impl AddAssign<Direction> for Point {
    fn add_assign(&mut self, dir: Direction) {
        *self += dir.to_displacement();
    }
}

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

impl Sub<Point> for Point {
    type Output = Displacement;
    fn sub(self, other: Point) -> Self::Output {
        Displacement::new(self.x - other.x, self.y - other.y)
    }
}

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

impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Self::Output {
        Point::new(-self.x, -self.y)
    }
}

/// 矩形结构体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    #[inline]
    pub const fn new(position: Point, size: Size) -> Self {
        Self { position, size }
    }

    /// 从中心点和半径创建正方形
    pub const fn from_center_radius(center: Point, radius: i32) -> Self {
        Self {
            position: Point::new(center.x - radius, center.y - radius),
            size: Size::square(2 * radius + 1),
        }
    }

    /// 检查点是否在矩形内
    #[inline]
    pub const fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y < self.position.y + self.size.height
    }

    /// 获取中心点
    #[inline]
    pub const fn center(&self) -> Point {
        Point::new(
            self.position.x + self.size.width / 2,
            self.position.y + self.size.height / 2,
        )
    }

    /// 内缩矩形
    pub const fn inset(&self, d: Displacement) -> Rectangle {
        Rectangle {
            position: Point::new(self.position.x + d.delta_x, self.position.y + d.delta_y),
            size: Size::new(
                self.size.width - d.delta_x * 2,
                self.size.height - d.delta_y * 2,
            ),
        }
    }

    /// 左边界
    #[inline]
    pub const fn left(&self) -> i32 {
        self.position.x
    }

    /// 右边界
    #[inline]
    pub const fn right(&self) -> i32 {
        self.position.x + self.size.width
    }

    /// 上边界
    #[inline]
    pub const fn top(&self) -> i32 {
        self.position.y
    }

    /// 下边界
    #[inline]
    pub const fn bottom(&self) -> i32 {
        self.position.y + self.size.height
    }
}

/// 计算从起点到目标点的最佳方向
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_turn() {
        assert_eq!(Direction::South.left(), Direction::SouthEast);
        assert_eq!(Direction::South.right(), Direction::SouthWest);
        assert_eq!(Direction::South.opposite(), Direction::North);
    }

    #[test]
    fn test_displacement_world_to_screen() {
        let world = Displacement::new(1, 1);
        let screen = world.world_to_screen();
        assert_eq!(screen, Displacement::new(0, -32));
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(p1.exact_distance(p2), 5);
        assert_eq!(p1.manhattan_distance(p2), 7);
        assert_eq!(p1.walking_distance(p2), 4);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(Point::new(0, 0), Size::new(10, 10));
        assert!(rect.contains(Point::new(5, 5)));
        assert!(rect.contains(Point::new(0, 0)));
        assert!(!rect.contains(Point::new(10, 10)));
        assert!(!rect.contains(Point::new(-1, 5)));
    }
}
