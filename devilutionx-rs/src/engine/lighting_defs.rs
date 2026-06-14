//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Lighting Definitions - 光照常量定义
//!
//! 移植自 Source/engine/lighting_defs.hpp
//!
//! 注意: 完整的光照系统在 Source/lighting.h/cpp 中，
//! 本模块只包含基础常量定义

/// 最大光源数量
pub const MAX_LIGHTS: usize = 32;

/// 最大视野光源数量
pub const MAX_VISION: usize = 4;

/// 无光源标记
pub const NO_LIGHT: i32 = -1;

/// 最大光照级别 (15 = 最暗)
/// C++ 原型: constexpr char LightsMax = 15;
pub const LIGHTS_MAX: i8 = 15;

/// 光照表大小 (等于调色板大小)
/// C++ 原型: constexpr size_t LightTableSize = 256;
pub const LIGHT_TABLE_SIZE: usize = 256;

/// 支持的光照级别数量 (0-15 共 16 级)
/// C++ 原型: constexpr size_t NumLightingLevels = LightsMax + 1;
pub const NUM_LIGHTING_LEVELS: usize = LIGHTS_MAX as usize + 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(NUM_LIGHTING_LEVELS, 16);
        assert_eq!(LIGHT_TABLE_SIZE, 256);
    }
}
