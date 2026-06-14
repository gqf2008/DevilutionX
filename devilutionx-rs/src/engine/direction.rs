//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Direction - 8方向枚举
//!
//! 移植自 Source/engine/direction.hpp/cpp

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

/// 向左转
/// C++ API: `Direction Left(Direction facing)`
#[inline]
pub const fn left(facing: Direction) -> Direction {
    match facing {
        Direction::NoDirection => Direction::NoDirection,
        _ => {
            // (facing + 7) % 8
            let val = facing as u8;
            let new_val = (val + 7) % 8;
            // Safety: new_val is always 0-7
            match new_val {
                0 => Direction::South,
                1 => Direction::SouthWest,
                2 => Direction::West,
                3 => Direction::NorthWest,
                4 => Direction::North,
                5 => Direction::NorthEast,
                6 => Direction::East,
                7 => Direction::SouthEast,
                _ => Direction::NoDirection, // unreachable
            }
        }
    }
}

/// 向右转
/// C++ API: `Direction Right(Direction facing)`
#[inline]
pub const fn right(facing: Direction) -> Direction {
    match facing {
        Direction::NoDirection => Direction::NoDirection,
        _ => {
            // (facing + 1) % 8
            let val = facing as u8;
            let new_val = (val + 1) % 8;
            match new_val {
                0 => Direction::South,
                1 => Direction::SouthWest,
                2 => Direction::West,
                3 => Direction::NorthWest,
                4 => Direction::North,
                5 => Direction::NorthEast,
                6 => Direction::East,
                7 => Direction::SouthEast,
                _ => Direction::NoDirection, // unreachable
            }
        }
    }
}

/// 反方向
/// C++ API: `Direction Opposite(Direction facing)`
#[inline]
pub const fn opposite(facing: Direction) -> Direction {
    match facing {
        Direction::NoDirection => Direction::NoDirection,
        _ => {
            // (facing + 4) % 8
            let val = facing as u8;
            let new_val = (val + 4) % 8;
            match new_val {
                0 => Direction::South,
                1 => Direction::SouthWest,
                2 => Direction::West,
                3 => Direction::NorthWest,
                4 => Direction::North,
                5 => Direction::NorthEast,
                6 => Direction::East,
                7 => Direction::SouthEast,
                _ => Direction::NoDirection, // unreachable
            }
        }
    }
}

/// 方向转字符串
/// C++ API: `std::string_view DirectionToString(Direction direction)`
pub fn direction_to_string(direction: Direction) -> &'static str {
    match direction {
        Direction::South => "South",
        Direction::SouthWest => "SouthWest",
        Direction::West => "West",
        Direction::NorthWest => "NorthWest",
        Direction::North => "North",
        Direction::NorthEast => "NorthEast",
        Direction::East => "East",
        Direction::SouthEast => "SouthEast",
        Direction::NoDirection => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left() {
        assert_eq!(left(Direction::South), Direction::SouthEast);
        assert_eq!(left(Direction::North), Direction::NorthWest);
    }

    #[test]
    fn test_right() {
        assert_eq!(right(Direction::South), Direction::SouthWest);
        assert_eq!(right(Direction::North), Direction::NorthEast);
    }

    #[test]
    fn test_opposite() {
        assert_eq!(opposite(Direction::South), Direction::North);
        assert_eq!(opposite(Direction::East), Direction::West);
    }

    #[test]
    fn test_direction_to_string() {
        assert_eq!(direction_to_string(Direction::South), "South");
        assert_eq!(direction_to_string(Direction::NoDirection), "");
    }
}
