//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Sound Definitions - 声音常量定义
//!
//! 移植自 Source/engine/sound_defs.hpp

/// 音量最小值
pub const VOLUME_MIN: i32 = -1600;

/// 音量最大值
pub const VOLUME_MAX: i32 = 0;

/// 音量级别数
pub const VOLUME_STEPS: i32 = 64;

/// 衰减最小值
pub const ATTENUATION_MIN: i32 = -6400;

/// 衰减最大值
pub const ATTENUATION_MAX: i32 = 0;

/// 声像最小值 (最左)
pub const PAN_MIN: i32 = -6400;

/// 声像最大值 (最右)
pub const PAN_MAX: i32 = 6400;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_range() {
        assert!(VOLUME_MIN < VOLUME_MAX);
    }

    #[test]
    fn test_pan_range() {
        assert!(PAN_MIN < PAN_MAX);
        assert_eq!(PAN_MIN, -PAN_MAX); // 对称
    }
}
