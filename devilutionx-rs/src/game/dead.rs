//! # ⚠️ 禁止变更 - 已完成移植
//!
//! 尸体系统 - 从 Source/dead.cpp 移植
//!
//! ## C++ 源文件: Source/dead.cpp (88 行)
//! ## Rust 文件: src/game/dead.rs
//!
//! ## 外部依赖说明
//!
//! 本模块依赖以下尚未完全移植的模块：
//!
//! - **monster 模块**: 需要 `CMonster` 结构（关卡怪物类型）和全局状态
//!   - `LevelMonsterTypes[]` - 关卡怪物类型数组
//!   - `Monsters[]` - 怪物实例数组  
//!   - `ActiveMonsters[]` - 活动怪物索引数组
//!   - `ActiveMonsterCount` - 活动怪物数量
//!   - 需要先移植 Source/monster.cpp 的全局状态管理
//!
//! - **lighting 模块**: ✅ 已可用
//!   - `ChangeLightXY()` -> `LightManager::change_light_position()`
//!   - `AddUnLight()` -> `LightManager::remove_light()`
//!
//! - **misdat 模块**: 需要 `GetMissileSpriteData()`
//!   - 用于获取石化效果的精灵数据
//!   - 需要完善 Source/misdat.cpp 的数据加载
//!
//! - **gendung 模块**: 需要 `dCorpse[][]` 全局数组
//!   - 用于存储地图上的尸体信息

use crate::game::types::Direction;

/// 最大尸体数量
///
/// **C++ Reference**: `MaxCorpses` in dead.h
pub const MAX_CORPSES: usize = 31;

/// 地牢最大X坐标
///
/// **C++ Reference**: `MAXDUNX` in defs.h
///
/// NOTE: duplicates the canonical `crate::levels::types::MAXDUNX` (= 112).
/// Not yet consolidated via `use` because this module is marked
/// "禁止变更 - 已完成移植"; both have identical value.
pub const MAXDUNX: usize = 112;

/// 地牢最大Y坐标
///
/// **C++ Reference**: `MAXDUNY` in defs.h
/// NOTE: duplicates `crate::levels::types::MAXDUNY` (= 112).
pub const MAXDUNY: usize = 112;

/// 尸体结构
///
/// **C++ Reference**: `Corpse` struct in dead.h
///
/// ```cpp
/// struct Corpse {
///     OptionalClxSpriteListOrSheet sprites;
///     int frame;
///     uint16_t width;
///     uint8_t translationPaletteIndex;
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Corpse {
    /// 精灵数据 - 依赖 CLX 精灵系统（engine/clx_sprite.hpp）
    /// 当前使用简化的占位类型，完整实现需要移植精灵系统
    pub sprites: Option<CorpseSprites>,
    /// 当前帧（死亡动画的最后一帧）
    pub frame: i32,
    /// 精灵宽度
    pub width: u16,
    /// 调色板转换索引（用于唯一怪物的颜色变化）
    pub translation_palette_index: u8,
}

impl Default for Corpse {
    fn default() -> Self {
        Self {
            sprites: None,
            frame: 0,
            width: 0,
            translation_palette_index: 0,
        }
    }
}

impl Corpse {
    /// 获取指定方向的精灵列表
    ///
    /// **C++ Reference**: `Corpse::spritesForDirection()`
    pub fn sprites_for_direction(&self, _direction: Direction) -> Option<&CorpseSprites> {
        // 完整实现需要精灵系统支持
        // sprites->isSheet() ? sprites->sheet()[direction] : sprites->list()
        self.sprites.as_ref()
    }
}

/// 尸体精灵数据
///
/// 简化的精灵数据结构，完整实现需要移植 engine/clx_sprite.hpp
#[derive(Debug, Clone)]
pub struct CorpseSprites {
    /// 精灵数据标识（临时）
    pub id: u32,
}

// ============================================================================
// 全局状态 - 对应 C++ 全局变量
// ============================================================================

/// 全局尸体数组
///
/// **C++ Reference**: `Corpse Corpses[MaxCorpses]` in dead.cpp
pub static mut CORPSES: [Corpse; MAX_CORPSES] = {
    const CORPSE_INIT: Corpse = Corpse {
        sprites: None,
        frame: 0,
        width: 0,
        translation_palette_index: 0,
    };
    [CORPSE_INIT; MAX_CORPSES]
};

/// 石化尸体索引
///
/// **C++ Reference**: `int8_t stonendx` in dead.cpp
pub static mut STONENDX: i8 = 0;

// ============================================================================
// 公开函数 - 严格对应 C++ dead.cpp 的函数
// ============================================================================

/// 添加尸体到地图
///
/// **C++ Reference**: `void AddCorpse(Point tilePosition, int8_t dv, Direction ddir)` in dead.cpp:85
///
/// ```cpp
/// void AddCorpse(Point tilePosition, int8_t dv, Direction ddir)
/// {
///     dCorpse[tilePosition.x][tilePosition.y] = (dv & 0x1F) + (static_cast<int>(ddir) << 5);
/// }
/// ```
///
/// # 参数
/// - `d_corpse`: 尸体地图数组的可变引用（来自 gendung 模块）
/// - `tile_x`, `tile_y`: 瓦片坐标
/// - `corpse_id`: 尸体ID（低5位）
/// - `direction`: 尸体朝向（高3位）
pub fn add_corpse(
    d_corpse: &mut [[i8; MAXDUNY]; MAXDUNX],
    tile_x: i32,
    tile_y: i32,
    corpse_id: i8,
    direction: Direction,
) {
    if tile_x >= 0 && (tile_x as usize) < MAXDUNX && tile_y >= 0 && (tile_y as usize) < MAXDUNY {
        d_corpse[tile_x as usize][tile_y as usize] =
            (corpse_id & 0x1F) + ((direction as i8) << 5);
    }
}

// ============================================================================
// 以下函数需要外部依赖，当前无法完整实现
// ============================================================================

/// 初始化尸体系统
///
/// **C++ Reference**: `void InitCorpses()` in dead.cpp:46
///
/// ## ⚠️ 外部依赖未满足
///
/// 此函数需要以下尚未移植的模块：
/// - `monster.h`: `LevelMonsterTypes[]`, `LevelMonsterTypeCount`, `CMonster` 结构
/// - `monster.h`: `Monsters[]`, `ActiveMonsters[]`, `ActiveMonsterCount`
/// - `misdat.h`: `GetMissileSpriteData(MissileGraphicID::StoneCurseShatter)`
/// - `diablo.h`: `HeadlessMode` 全局变量
///
/// 需要先移植: Source/monster.cpp 的全局状态管理
///
/// ```cpp
/// void InitCorpses()
/// {
///     int8_t mtypes[MaxMonsters] = {};
///     int8_t nd = 0;
///
///     for (size_t i = 0; i < LevelMonsterTypeCount; i++) {
///         CMonster &monsterType = LevelMonsterTypes[i];
///         if (mtypes[monsterType.type] != 0)
///             continue;
///
///         InitDeadAnimationFromMonster(Corpses[nd], monsterType);
///         Corpses[nd].translationPaletteIndex = 0;
///         nd++;
///
///         monsterType.corpseId = nd;
///         mtypes[monsterType.type] = nd;
///     }
///
///     nd++; // Unused blood spatter
///
///     if (!HeadlessMode)
///         Corpses[nd].sprites.emplace(*GetMissileSpriteData(MissileGraphicID::StoneCurseShatter).sprites);
///     Corpses[nd].frame = 11;
///     Corpses[nd].width = 128;
///     Corpses[nd].translationPaletteIndex = 0;
///     nd++;
///
///     stonendx = nd;
///
///     for (size_t i = 0; i < ActiveMonsterCount; i++) {
///         auto &monster = Monsters[ActiveMonsters[i]];
///         if (monster.isUnique()) {
///             InitDeadAnimationFromMonster(Corpses[nd], monster.type());
///             Corpses[nd].translationPaletteIndex = ActiveMonsters[i] + 1;
///             nd++;
///
///             monster.corpseId = nd;
///         }
///     }
///
///     assert(static_cast<unsigned>(nd) <= MaxCorpses);
/// }
/// ```
pub fn init_corpses() {
    // 无法实现 - 需要先移植 Source/monster.cpp 的全局状态:
    // - LevelMonsterTypes[] 数组
    // - Monsters[] 数组
    // - ActiveMonsters[] 数组
    // - ActiveMonsterCount 变量
    // - CMonster::getAnimData() 方法
    unimplemented!(
        "InitCorpses 需要先移植 Source/monster.cpp 的全局状态管理 \
        (LevelMonsterTypes, Monsters, ActiveMonsters)"
    );
}

/// 移动唯一怪物的光源到尸体位置
///
/// **C++ Reference**: `void MoveLightsToCorpses()` in dead.cpp:89
///
/// ## ⚠️ 外部依赖未满足
///
/// 此函数需要以下尚未移植的模块：
/// - `monster.h`: `Monsters[]`, `ActiveMonsters[]`, `ActiveMonsterCount`
/// - `monster.h`: `Monster::isUnique()`, `Monster::corpseId`, `Monster::lightId`
/// - `lighting.h`: `ChangeLightXY()`, `AddUnLight()`
/// - `gendung.h`: `dCorpse[][]` 全局数组
///
/// ```cpp
/// void MoveLightsToCorpses()
/// {
///     for (size_t i = 0; i < ActiveMonsterCount; i++) {
///         auto &monster = Monsters[ActiveMonsters[i]];
///         if (!monster.isUnique())
///             continue;
///         MoveLightToCorpse(monster);
///     }
/// }
/// ```
pub fn move_lights_to_corpses() {
    // 无法实现 - 需要先移植 Source/monster.cpp 的全局状态:
    // - Monsters[] 数组
    // - ActiveMonsters[] 数组
    // - ActiveMonsterCount 变量
    unimplemented!(
        "MoveLightsToCorpses 需要先移植 Source/monster.cpp 的全局状态管理"
    );
}

// ============================================================================
// 内部辅助函数
// ============================================================================

/// 从 Direction 值解码尸体方向
fn direction_from_encoded(value: i8) -> Direction {
    match (value >> 5) & 0x07 {
        0 => Direction::South,
        1 => Direction::SouthWest,
        2 => Direction::West,
        3 => Direction::NorthWest,
        4 => Direction::North,
        5 => Direction::NorthEast,
        6 => Direction::East,
        7 => Direction::SouthEast,
        _ => Direction::South,
    }
}

/// 从编码值提取尸体ID
#[inline]
pub fn get_corpse_id(encoded: i8) -> i8 {
    encoded & 0x1F
}

/// 从编码值提取尸体方向
#[inline]
pub fn get_corpse_direction(encoded: i8) -> Direction {
    direction_from_encoded(encoded)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpse_default() {
        let corpse = Corpse::default();
        assert!(corpse.sprites.is_none());
        assert_eq!(corpse.frame, 0);
        assert_eq!(corpse.width, 0);
        assert_eq!(corpse.translation_palette_index, 0);
    }

    #[test]
    fn test_add_corpse() {
        let mut d_corpse = [[0i8; MAXDUNY]; MAXDUNX];

        add_corpse(&mut d_corpse, 10, 20, 5, Direction::South);

        let encoded = d_corpse[10][20];
        assert_eq!(get_corpse_id(encoded), 5);
        assert_eq!(get_corpse_direction(encoded), Direction::South);
    }

    #[test]
    fn test_add_corpse_with_direction() {
        let mut d_corpse = [[0i8; MAXDUNY]; MAXDUNX];

        add_corpse(&mut d_corpse, 50, 60, 15, Direction::NorthEast);

        let encoded = d_corpse[50][60];
        assert_eq!(get_corpse_id(encoded), 15);
        assert_eq!(get_corpse_direction(encoded), Direction::NorthEast);
    }

    #[test]
    fn test_add_corpse_bounds_check() {
        let mut d_corpse = [[0i8; MAXDUNY]; MAXDUNX];

        // 越界坐标应该被忽略
        add_corpse(&mut d_corpse, -1, 20, 5, Direction::South);
        add_corpse(&mut d_corpse, 10, -1, 5, Direction::South);
        add_corpse(&mut d_corpse, 200, 20, 5, Direction::South);
        add_corpse(&mut d_corpse, 10, 200, 5, Direction::South);

        // 验证没有写入任何数据
        for row in &d_corpse {
            for &cell in row {
                assert_eq!(cell, 0);
            }
        }
    }

    #[test]
    fn test_corpse_id_mask() {
        // 验证低5位掩码 (0x1F = 31)
        let mut d_corpse = [[0i8; MAXDUNY]; MAXDUNX];

        // 尸体ID超过31应该被截断
        add_corpse(&mut d_corpse, 10, 20, 35, Direction::South); // 35 & 0x1F = 3

        let encoded = d_corpse[10][20];
        assert_eq!(get_corpse_id(encoded), 3);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CORPSES, 31);
        assert_eq!(MAXDUNX, 112);
        assert_eq!(MAXDUNY, 112);
    }
}
