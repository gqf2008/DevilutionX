//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Sound Position - 声音位置计算
//!
//! 移植自 Source/engine/sound_position.hpp/cpp
//!
//! ## 模块依赖
//! - point.rs: Point 类型
//! - sound_defs.rs: PAN_MIN, PAN_MAX, ATTENUATION_MIN

use super::point::Point;
use super::sound_defs::{ATTENUATION_MIN, PAN_MAX, PAN_MIN};

/// 计算声音位置的音量和声像
///
/// 根据声音源位置计算：
/// - 音量衰减 (基于距离)
/// - 左右声像 (基于相对位置)
///
/// C++ API: `bool CalculateSoundPosition(Point soundPosition, int *plVolume, int *plPan)`
/// C++ 从全局 `MyPlayer->position.tile` 获取玩家位置，
/// Rust 版本需要显式传入玩家位置。
///
/// # 参数
/// - `sound_position`: 声音源位置
/// - `player_position`: 玩家位置 (C++ 从全局状态获取)
/// - `pl_volume`: 输出音量
/// - `pl_pan`: 输出声像
///
/// # 返回
/// - `true`: 声音应该播放
/// - `false`: 声音太远，不应播放
pub fn calculate_sound_position(
    sound_position: Point,
    player_position: Point,
    pl_volume: &mut i32,
    pl_pan: &mut i32,
) -> bool {
    let delta_x = sound_position.x - player_position.x;
    let delta_y = sound_position.y - player_position.y;

    // 计算声像 (左右定位)
    let pan = (delta_x - delta_y) * 256;
    *pl_pan = pan.clamp(PAN_MIN, PAN_MAX);

    // 计算音量衰减 (基于近似距离)
    let volume = player_position.approx_distance(sound_position) * -64;

    if volume <= ATTENUATION_MIN {
        return false;
    }

    *pl_volume = volume;

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_at_player_position() {
        let pos = Point::new(10, 10);
        let mut volume = 0;
        let mut pan = 0;
        let result = calculate_sound_position(pos, pos, &mut volume, &mut pan);
        assert!(result);
        assert_eq!(volume, 0); // 距离为0
        assert_eq!(pan, 0);    // 同一位置
    }

    #[test]
    fn test_sound_too_far() {
        let sound_pos = Point::new(1000, 1000);
        let player_pos = Point::new(0, 0);
        let mut volume = 0;
        let mut pan = 0;
        let result = calculate_sound_position(sound_pos, player_pos, &mut volume, &mut pan);
        assert!(!result); // 太远
    }
}
