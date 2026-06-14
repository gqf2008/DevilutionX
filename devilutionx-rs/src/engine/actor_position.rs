//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Actor Position - 角色位置
//!
//! 移植自 Source/engine/actor_position.hpp/cpp
//!
//! 角色 (玩家、怪物) 的位置跟踪，支持行走动画偏移和瓦片过渡。

use super::direction::Direction;
use super::animationinfo::{AnimationInfo, BASE_VALUE_FRACTION};
use super::world_tile::WorldTilePosition;

/// 16位位移 (用于行走偏移计算)
#[derive(Debug, Clone, Copy, Default)]
pub struct Displacement16 {
    pub delta_x: i16,
    pub delta_y: i16,
}

impl Displacement16 {
    pub const fn new(delta_x: i16, delta_y: i16) -> Self {
        Self { delta_x, delta_y }
    }
}

/// 8位位移 (用于行走偏移结果)
#[derive(Debug, Clone, Copy, Default)]
pub struct Displacement8 {
    pub delta_x: i8,
    pub delta_y: i8,
}

impl Displacement8 {
    pub const fn new(delta_x: i8, delta_y: i8) -> Self {
        Self { delta_x, delta_y }
    }
}

/// 角色位置，包含多个位置状态用于平滑移动
///
/// C++ Reference: `ActorPosition` struct in actor_position.hpp
#[derive(Debug, Clone, Copy, Default)]
pub struct ActorPosition {
    /// 当前瓦片位置
    pub tile: WorldTilePosition,
    /// 未来瓦片位置 (在行走动画开始时设置)
    pub future: WorldTilePosition,
    /// 最后输入位置 (通过网络在玩家输入时设置)
    pub last: WorldTilePosition,
    /// dPlayer 数组中的最近位置
    pub old: WorldTilePosition,
    /// 临时位置 (用于法术/攻击目标)
    pub temp: WorldTilePosition,
}

//=============================================================================
// 行走速度查找表
//=============================================================================

/// 行走速度 (根据帧数)
#[derive(Clone, Copy)]
struct RoundedWalkVelocity {
    quarter: i16,
    half: i16,
    full: i16,
}

impl RoundedWalkVelocity {
    const fn get_velocity(&self, velocity_type: VelocityToUse) -> i16 {
        match velocity_type {
            VelocityToUse::None => 0,
            VelocityToUse::Quarter => self.quarter,
            VelocityToUse::NegativeQuarter => -self.quarter,
            VelocityToUse::Half => self.half,
            VelocityToUse::NegativeHalf => -self.half,
            VelocityToUse::Full => self.full,
            VelocityToUse::NegativeFull => -self.full,
        }
    }
}

#[derive(Clone, Copy)]
enum VelocityToUse {
    None,
    Full,
    NegativeFull,
    Half,
    NegativeHalf,
    Quarter,
    NegativeQuarter,
}

/// 行走速度查找表 (按帧数索引，从 1 帧到 24 帧)
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

/// 行走参数 (每个方向的 X/Y 速度类型)
struct WalkParameter {
    velocity_x: VelocityToUse,
    velocity_y: VelocityToUse,
}

impl WalkParameter {
    fn get_velocity(&self, number_of_frames: i8) -> Displacement16 {
        let idx = (number_of_frames - 1).max(0).min(23) as usize;
        let walk_velocity = &WALK_VELOCITY_FOR_FRAMES[idx];
        Displacement16::new(
            walk_velocity.get_velocity(self.velocity_x),
            walk_velocity.get_velocity(self.velocity_y),
        )
    }
}

/// 8 个方向的行走参数
const WALK_PARAMETERS: [WalkParameter; 8] = [
    WalkParameter { velocity_x: VelocityToUse::None, velocity_y: VelocityToUse::Half },              // South
    WalkParameter { velocity_x: VelocityToUse::NegativeHalf, velocity_y: VelocityToUse::Quarter },   // SouthWest
    WalkParameter { velocity_x: VelocityToUse::NegativeFull, velocity_y: VelocityToUse::None },      // West
    WalkParameter { velocity_x: VelocityToUse::NegativeHalf, velocity_y: VelocityToUse::NegativeQuarter }, // NorthWest
    WalkParameter { velocity_x: VelocityToUse::None, velocity_y: VelocityToUse::NegativeHalf },      // North
    WalkParameter { velocity_x: VelocityToUse::Half, velocity_y: VelocityToUse::NegativeQuarter },   // NorthEast
    WalkParameter { velocity_x: VelocityToUse::Full, velocity_y: VelocityToUse::None },              // East
    WalkParameter { velocity_x: VelocityToUse::Half, velocity_y: VelocityToUse::Quarter },           // SouthEast
];

impl ActorPosition {
    /// 计算行走动画偏移
    /// C++ API: `DisplacementOf<int8_t> CalculateWalkingOffset(Direction dir, const AnimationInfo &animInfo) const`
    pub fn calculate_walking_offset(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement8 {
        let offset = self.calculate_walking_offset_shifted4(dir, anim_info);
        Displacement8::new(
            (offset.delta_x >> 4) as i8,
            (offset.delta_y >> 4) as i8,
        )
    }

    /// 计算行走动画偏移 (4位精度)
    /// C++ API: `DisplacementOf<int16_t> CalculateWalkingOffsetShifted4(Direction dir, const AnimationInfo &animInfo) const`
    pub fn calculate_walking_offset_shifted4(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let velocity_progress = (anim_info.get_animation_progress() as i32 * anim_info.number_of_frames as i32 
            / BASE_VALUE_FRACTION as i32) as i16;
        
        let dir_idx = match dir {
            Direction::NoDirection => return Displacement16::new(0, 0),
            d => d as usize,
        };
        
        let walk_param = &WALK_PARAMETERS[dir_idx];
        let velocity = walk_param.get_velocity(anim_info.number_of_frames);
        
        Displacement16::new(
            velocity.delta_x.saturating_mul(velocity_progress),
            velocity.delta_y.saturating_mul(velocity_progress),
        )
    }

    /// 计算行走动画偏移 (8位精度)
    /// C++ API: `DisplacementOf<int16_t> CalculateWalkingOffsetShifted8(Direction dir, const AnimationInfo &animInfo) const`
    pub fn calculate_walking_offset_shifted8(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let offset = self.calculate_walking_offset_shifted4(dir, anim_info);
        Displacement16::new(
            offset.delta_x << 4,
            offset.delta_y << 4,
        )
    }

    /// 获取行走像素速度 (4位精度)
    /// C++ API: `DisplacementOf<int16_t> GetWalkingVelocityShifted4(Direction dir, const AnimationInfo &animInfo) const`
    pub fn get_walking_velocity_shifted4(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let dir_idx = match dir {
            Direction::NoDirection => return Displacement16::new(0, 0),
            d => d as usize,
        };
        
        WALK_PARAMETERS[dir_idx].get_velocity(anim_info.number_of_frames)
    }

    /// 获取行走像素速度 (8位精度)
    /// C++ API: `DisplacementOf<int16_t> GetWalkingVelocityShifted8(Direction dir, const AnimationInfo &animInfo) const`
    pub fn get_walking_velocity_shifted8(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement16 {
        let velocity = self.get_walking_velocity_shifted4(dir, anim_info);
        Displacement16::new(
            velocity.delta_x << 4,
            velocity.delta_y << 4,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_position_default() {
        let pos = ActorPosition::default();
        assert_eq!(pos.tile.x, 0);
        assert_eq!(pos.tile.y, 0);
    }
}
