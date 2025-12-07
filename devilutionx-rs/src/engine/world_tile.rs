//! World Tile - 世界坐标与角色位置
//!
//! 移植自 Source/engine/world_tile.hpp 和 actor_position.hpp
//!
//! 世界坐标使用 u8 表示地块位置 (地图最大 112x112)

use super::animation::{AnimationInfo, BASE_VALUE_FRACTION};
use super::types::Direction;
use std::hash::{Hash, Hasher};

/// 世界地块坐标 (0-255)
pub type WorldTileCoord = u8;

/// 世界地块位置
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldTilePosition {
    pub x: WorldTileCoord,
    pub y: WorldTileCoord,
}

impl WorldTilePosition {
    pub const fn new(x: WorldTileCoord, y: WorldTileCoord) -> Self {
        Self { x, y }
    }

    /// 计算两点间的距离
    pub fn distance(&self, other: WorldTilePosition) -> f32 {
        let dx = (self.x as i32 - other.x as i32) as f32;
        let dy = (self.y as i32 - other.y as i32) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// 计算曼哈顿距离
    pub fn manhattan_distance(&self, other: WorldTilePosition) -> u16 {
        let dx = (self.x as i16 - other.x as i16).abs() as u16;
        let dy = (self.y as i16 - other.y as i16).abs() as u16;
        dx + dy
    }

    /// 偏移位置
    pub fn offset(&self, dx: i8, dy: i8) -> Self {
        Self {
            x: (self.x as i16 + dx as i16).clamp(0, 255) as u8,
            y: (self.y as i16 + dy as i16).clamp(0, 255) as u8,
        }
    }
}

impl Hash for WorldTilePosition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash_value: u16 = ((self.x as u16) << 8) | (self.y as u16);
        hash_value.hash(state);
    }
}

/// 世界地块偏移量
pub type WorldTileOffset = i8;

/// 世界地块位移
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldTileDisplacement {
    pub dx: WorldTileOffset,
    pub dy: WorldTileOffset,
}

impl WorldTileDisplacement {
    pub const fn new(dx: WorldTileOffset, dy: WorldTileOffset) -> Self {
        Self { dx, dy }
    }
}

/// 世界地块尺寸
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldTileSize {
    pub width: WorldTileCoord,
    pub height: WorldTileCoord,
}

impl WorldTileSize {
    pub const fn new(width: WorldTileCoord, height: WorldTileCoord) -> Self {
        Self { width, height }
    }
}

/// 世界地块矩形区域
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldTileRectangle {
    pub position: WorldTilePosition,
    pub size: WorldTileSize,
}

impl WorldTileRectangle {
    pub const fn new(x: WorldTileCoord, y: WorldTileCoord, width: WorldTileCoord, height: WorldTileCoord) -> Self {
        Self {
            position: WorldTilePosition::new(x, y),
            size: WorldTileSize::new(width, height),
        }
    }

    /// 检查点是否在矩形内
    pub fn contains(&self, pos: WorldTilePosition) -> bool {
        pos.x >= self.position.x
            && pos.x < self.position.x.saturating_add(self.size.width)
            && pos.y >= self.position.y
            && pos.y < self.position.y.saturating_add(self.size.height)
    }
}

/// 速度使用类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VelocityToUse {
    None,
    Full,
    NegativeFull,
    Half,
    NegativeHalf,
    Quarter,
    NegativeQuarter,
}

/// 圆整后的行走速度
#[derive(Clone, Copy)]
struct RoundedWalkVelocity {
    quarter: i16,
    half: i16,
    full: i16,
}

impl RoundedWalkVelocity {
    const fn get_velocity(&self, velocity_to_use: VelocityToUse) -> i16 {
        match velocity_to_use {
            VelocityToUse::Quarter => self.quarter,
            VelocityToUse::NegativeQuarter => -self.quarter,
            VelocityToUse::Half => self.half,
            VelocityToUse::NegativeHalf => -self.half,
            VelocityToUse::Full => self.full,
            VelocityToUse::NegativeFull => -self.full,
            VelocityToUse::None => 0,
        }
    }
}

/// 行走动画帧数到速度的映射表
const WALK_VELOCITY_FOR_FRAMES: [RoundedWalkVelocity; 24] = [
    RoundedWalkVelocity { quarter: 256, half: 512, full: 1024 },
    RoundedWalkVelocity { quarter: 128, half: 256, full: 512 },
    RoundedWalkVelocity { quarter: 85, half: 170, full: 341 },
    RoundedWalkVelocity { quarter: 64, half: 128, full: 256 },
    RoundedWalkVelocity { quarter: 51, half: 102, full: 204 },
    RoundedWalkVelocity { quarter: 42, half: 85, full: 170 },
    RoundedWalkVelocity { quarter: 36, half: 73, full: 146 },
    RoundedWalkVelocity { quarter: 32, half: 64, full: 128 },
    RoundedWalkVelocity { quarter: 28, half: 56, full: 113 },
    RoundedWalkVelocity { quarter: 26, half: 51, full: 102 },
    RoundedWalkVelocity { quarter: 23, half: 46, full: 93 },
    RoundedWalkVelocity { quarter: 21, half: 42, full: 85 },
    RoundedWalkVelocity { quarter: 19, half: 39, full: 78 },
    RoundedWalkVelocity { quarter: 18, half: 36, full: 73 },
    RoundedWalkVelocity { quarter: 17, half: 34, full: 68 },
    RoundedWalkVelocity { quarter: 16, half: 32, full: 64 },
    RoundedWalkVelocity { quarter: 15, half: 30, full: 60 },
    RoundedWalkVelocity { quarter: 14, half: 28, full: 57 },
    RoundedWalkVelocity { quarter: 13, half: 26, full: 54 },
    RoundedWalkVelocity { quarter: 12, half: 25, full: 51 },
    RoundedWalkVelocity { quarter: 12, half: 24, full: 48 },
    RoundedWalkVelocity { quarter: 11, half: 23, full: 46 },
    RoundedWalkVelocity { quarter: 11, half: 22, full: 44 },
    RoundedWalkVelocity { quarter: 10, half: 21, full: 42 },
];

/// 行走参数
#[derive(Clone, Copy)]
struct WalkParameter {
    velocity_x: VelocityToUse,
    velocity_y: VelocityToUse,
}

impl WalkParameter {
    fn get_velocity(&self, number_of_frames: i8) -> (i16, i16) {
        let index = (number_of_frames.max(1) as usize - 1).min(23);
        let walk_velocity = &WALK_VELOCITY_FOR_FRAMES[index];
        (
            walk_velocity.get_velocity(self.velocity_x),
            walk_velocity.get_velocity(self.velocity_y),
        )
    }
}

/// 各方向的行走参数
const WALK_PARAMETERS: [WalkParameter; 8] = [
    // South
    WalkParameter { velocity_x: VelocityToUse::None, velocity_y: VelocityToUse::Half },
    // SouthWest
    WalkParameter { velocity_x: VelocityToUse::NegativeHalf, velocity_y: VelocityToUse::Quarter },
    // West
    WalkParameter { velocity_x: VelocityToUse::NegativeFull, velocity_y: VelocityToUse::None },
    // NorthWest
    WalkParameter { velocity_x: VelocityToUse::NegativeHalf, velocity_y: VelocityToUse::NegativeQuarter },
    // North
    WalkParameter { velocity_x: VelocityToUse::None, velocity_y: VelocityToUse::NegativeHalf },
    // NorthEast
    WalkParameter { velocity_x: VelocityToUse::Half, velocity_y: VelocityToUse::NegativeQuarter },
    // East
    WalkParameter { velocity_x: VelocityToUse::Full, velocity_y: VelocityToUse::None },
    // SouthEast
    WalkParameter { velocity_x: VelocityToUse::Half, velocity_y: VelocityToUse::Quarter },
];

/// 16 位位移
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Displacement16 {
    pub delta_x: i16,
    pub delta_y: i16,
}

impl Displacement16 {
    pub const fn new(delta_x: i16, delta_y: i16) -> Self {
        Self { delta_x, delta_y }
    }

    /// 乘法
    pub fn mul(&self, factor: i16) -> Self {
        Self {
            delta_x: self.delta_x.saturating_mul(factor),
            delta_y: self.delta_y.saturating_mul(factor),
        }
    }
}

/// 8 位位移
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Displacement8 {
    pub delta_x: i8,
    pub delta_y: i8,
}

impl Displacement8 {
    pub const fn new(delta_x: i8, delta_y: i8) -> Self {
        Self { delta_x, delta_y }
    }
}

/// 角色位置
///
/// 用于跟踪角色在世界中的各种位置状态
#[derive(Clone, Copy, Debug, Default)]
pub struct ActorPosition {
    /// 当前地块位置
    pub tile: WorldTilePosition,
    /// 未来地块位置（行走动画开始时设置）
    pub future: WorldTilePosition,
    /// 最后地块位置（通过网络设置）
    pub last: WorldTilePosition,
    /// 最近在 dPlayer 中的位置
    pub old: WorldTilePosition,
    /// 用于完成移动一个地块时的参考位置
    pub temp: WorldTilePosition,
}

impl ActorPosition {
    /// 创建新的角色位置
    pub fn new(tile: WorldTilePosition) -> Self {
        Self {
            tile,
            future: tile,
            last: tile,
            old: tile,
            temp: tile,
        }
    }

    /// 计算行走动画偏移量
    pub fn calculate_walking_offset(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement8 {
        let offset = self.calculate_walking_offset_shifted4(dir, anim_info);
        Displacement8 {
            delta_x: (offset.delta_x >> 4) as i8,
            delta_y: (offset.delta_y >> 4) as i8,
        }
    }

    /// 计算行走动画偏移量（左移 4 位）
    pub fn calculate_walking_offset_shifted4(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let velocity_progress = (anim_info.get_animation_progress() as i32
            * anim_info.number_of_frames as i32
            / BASE_VALUE_FRACTION as i32) as i16;

        let dir_index = direction_to_index(dir);
        let walk_param = &WALK_PARAMETERS[dir_index];
        let (vx, vy) = walk_param.get_velocity(anim_info.number_of_frames);

        Displacement16 {
            delta_x: vx.saturating_mul(velocity_progress),
            delta_y: vy.saturating_mul(velocity_progress),
        }
    }

    /// 计算行走动画偏移量（左移 8 位）
    pub fn calculate_walking_offset_shifted8(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let mut offset = self.calculate_walking_offset_shifted4(dir, anim_info);
        offset.delta_x = offset.delta_x.saturating_mul(16);
        offset.delta_y = offset.delta_y.saturating_mul(16);
        offset
    }

    /// 获取行走速度（左移 4 位）
    pub fn get_walking_velocity_shifted4(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let dir_index = direction_to_index(dir);
        let walk_param = &WALK_PARAMETERS[dir_index];
        let (vx, vy) = walk_param.get_velocity(anim_info.number_of_frames);
        Displacement16::new(vx, vy)
    }

    /// 获取行走速度（左移 8 位）
    pub fn get_walking_velocity_shifted8(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let mut velocity = self.get_walking_velocity_shifted4(dir, anim_info);
        velocity.delta_x = velocity.delta_x.saturating_mul(16);
        velocity.delta_y = velocity.delta_y.saturating_mul(16);
        velocity
    }

    /// 设置所有位置为同一值
    pub fn set_all(&mut self, pos: WorldTilePosition) {
        self.tile = pos;
        self.future = pos;
        self.last = pos;
        self.old = pos;
        self.temp = pos;
    }
}

/// 将方向转换为索引（0-7）
fn direction_to_index(dir: Direction) -> usize {
    match dir {
        Direction::South => 0,
        Direction::SouthWest => 1,
        Direction::West => 2,
        Direction::NorthWest => 3,
        Direction::North => 4,
        Direction::NorthEast => 5,
        Direction::East => 6,
        Direction::SouthEast => 7,
        Direction::NoDirection => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_tile_position() {
        let pos1 = WorldTilePosition::new(10, 20);
        let pos2 = WorldTilePosition::new(13, 24);

        let dist = pos1.distance(pos2);
        assert!(dist > 4.9 && dist < 5.1);

        let manhattan = pos1.manhattan_distance(pos2);
        assert_eq!(manhattan, 7);
    }

    #[test]
    fn test_world_tile_position_offset() {
        let pos = WorldTilePosition::new(50, 50);

        let moved = pos.offset(5, -10);
        assert_eq!(moved.x, 55);
        assert_eq!(moved.y, 40);

        // 测试边界
        let at_edge = WorldTilePosition::new(255, 0);
        let clamped = at_edge.offset(10, -10);
        assert_eq!(clamped.x, 255);
        assert_eq!(clamped.y, 0);
    }

    #[test]
    fn test_world_tile_rectangle_contains() {
        let rect = WorldTileRectangle::new(10, 10, 20, 20);

        assert!(rect.contains(WorldTilePosition::new(15, 15)));
        assert!(rect.contains(WorldTilePosition::new(10, 10)));
        assert!(!rect.contains(WorldTilePosition::new(30, 30)));
        assert!(!rect.contains(WorldTilePosition::new(5, 15)));
    }

    #[test]
    fn test_actor_position() {
        let start = WorldTilePosition::new(50, 50);
        let actor = ActorPosition::new(start);

        assert_eq!(actor.tile, start);
        assert_eq!(actor.future, start);
        assert_eq!(actor.last, start);
    }

    #[test]
    fn test_walk_velocity() {
        let actor = ActorPosition::new(WorldTilePosition::new(50, 50));
        let anim = AnimationInfo::new();

        // 测试不同方向的速度
        let vel_south = actor.get_walking_velocity_shifted4(Direction::South, &anim);
        let vel_east = actor.get_walking_velocity_shifted4(Direction::East, &anim);
        let vel_north = actor.get_walking_velocity_shifted4(Direction::North, &anim);

        // 南向：只有 Y 速度
        assert_eq!(vel_south.delta_x, 0);
        assert!(vel_south.delta_y > 0);

        // 东向：只有 X 速度
        assert!(vel_east.delta_x > 0);
        assert_eq!(vel_east.delta_y, 0);

        // 北向：只有负 Y 速度
        assert_eq!(vel_north.delta_x, 0);
        assert!(vel_north.delta_y < 0);
    }

    #[test]
    fn test_walk_velocity_table() {
        // 检查速度表的正确性
        for i in 0..24 {
            let vel = &WALK_VELOCITY_FOR_FRAMES[i];
            // 半速应该是全速的一半
            assert!((vel.half - vel.full / 2).abs() <= 1);
            // 四分之一速应该是半速的一半
            assert!((vel.quarter - vel.half / 2).abs() <= 1);
        }
    }
}
