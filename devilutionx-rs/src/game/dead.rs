//! 尸体系统 (M34)
//!
//! 从 Source/dead.cpp 移植
//! 处理死亡怪物尸体的放置和管理

use crate::game::types::Direction;

/// 从u8转换Direction的辅助trait
trait DirectionFromU8 {
    fn from_u8(value: u8) -> Self;
}

impl DirectionFromU8 for Direction {
    fn from_u8(value: u8) -> Self {
        match value & 0x07 {
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
}

/// 最大尸体数量
pub const MAX_CORPSES: usize = 31;

/// 最大怪物数量
pub const MAX_MONSTERS: usize = 200;

/// 地牢最大X坐标
pub const MAXDUNX: usize = 112;

/// 地牢最大Y坐标
pub const MAXDUNY: usize = 112;

/// 尸体结构
///
/// 从 C++ Corpse 结构移植
#[derive(Debug, Clone)]
pub struct Corpse {
    /// 精灵数据（占位）
    pub sprites: Option<CorpseSprites>,
    /// 当前帧
    pub frame: i32,
    /// 宽度
    pub width: i32,
    /// 调色板索引
    pub translation_palette_index: i32,
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

/// 尸体精灵数据（占位）
#[derive(Debug, Clone)]
pub struct CorpseSprites {
    /// 精灵数据标识
    pub id: u32,
}

/// 怪物类型（占位）
#[derive(Debug, Clone, Default)]
pub struct CMonster {
    /// 怪物类型ID
    pub monster_type: i32,
    /// 尸体ID
    pub corpse_id: i8,
}

/// 怪物实例（占位）
#[derive(Debug, Clone, Default)]
pub struct Monster {
    /// 是否唯一怪物
    pub unique: bool,
    /// 尸体ID
    pub corpse_id: i8,
    /// 光源ID
    pub light_id: i32,
}

impl Monster {
    /// 检查是否为唯一怪物
    pub fn is_unique(&self) -> bool {
        self.unique
    }
}

/// 动画结构（占位）
#[derive(Debug, Clone, Default)]
pub struct AnimStruct {
    /// 精灵数据
    pub sprites: Option<CorpseSprites>,
    /// 帧数
    pub frames: i32,
    /// 宽度
    pub width: i32,
}

/// 全局尸体数组
pub static mut CORPSES: [Corpse; MAX_CORPSES] = {
    const CORPSE_INIT: Corpse = Corpse {
        sprites: None,
        frame: 0,
        width: 0,
        translation_palette_index: 0,
    };
    [CORPSE_INIT; MAX_CORPSES]
};

/// 石化索引
pub static mut STONENDX: i8 = 0;

/// 尸体地图数据
pub static mut D_CORPSE: [[u8; MAXDUNY]; MAXDUNX] = [[0; MAXDUNY]; MAXDUNX];

/// 尸体管理器
#[derive(Debug)]
pub struct CorpseManager {
    /// 尸体数组
    pub corpses: Vec<Corpse>,
    /// 石化索引
    pub stone_index: i8,
    /// 尸体地图
    pub corpse_map: Vec<Vec<u8>>,
}

impl Default for CorpseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CorpseManager {
    /// 创建新的尸体管理器
    pub fn new() -> Self {
        Self {
            corpses: vec![Corpse::default(); MAX_CORPSES],
            stone_index: 0,
            corpse_map: vec![vec![0u8; MAXDUNY]; MAXDUNX],
        }
    }

    /// 初始化尸体系统
    ///
    /// 从 InitCorpses 移植
    pub fn init(&mut self, level_monster_types: &[CMonster], monsters: &[Monster], active_monsters: &[usize]) {
        let mut mtypes = [0i8; MAX_MONSTERS];
        let mut nd: i8 = 0;

        // 初始化关卡怪物类型的尸体
        for mon in level_monster_types {
            if mtypes[mon.monster_type as usize] != 0 {
                continue;
            }

            // 初始化尸体动画
            self.init_corpse_from_monster(nd as usize, mon);
            self.corpses[nd as usize].translation_palette_index = 0;
            nd += 1;

            mtypes[mon.monster_type as usize] = nd;
        }

        nd += 1; // 未使用的血迹

        // 石化效果
        self.corpses[nd as usize] = Corpse {
            sprites: None, // 实际应加载 MissileGraphicID::StoneCurseShatter
            frame: 11,
            width: 128,
            translation_palette_index: 0,
        };
        nd += 1;

        self.stone_index = nd;

        // 处理唯一怪物
        for &monster_idx in active_monsters {
            if monster_idx >= monsters.len() {
                continue;
            }
            let monster = &monsters[monster_idx];
            if monster.is_unique() {
                // 初始化唯一怪物的尸体
                self.corpses[nd as usize] = Corpse {
                    sprites: None,
                    frame: 0,
                    width: 0,
                    translation_palette_index: monster_idx as i32 + 1,
                };
                nd += 1;
            }
        }

        assert!((nd as usize) <= MAX_CORPSES);
    }

    /// 从怪物初始化尸体动画
    fn init_corpse_from_monster(&mut self, corpse_idx: usize, _monster: &CMonster) {
        // 实际实现需要从怪物获取死亡动画数据
        // const AnimStruct &animData = mon.getAnimData(MonsterGraphic::Death);
        if corpse_idx < self.corpses.len() {
            self.corpses[corpse_idx] = Corpse {
                sprites: None,
                frame: 0,
                width: 0,
                translation_palette_index: 0,
            };
        }
    }

    /// 添加尸体
    ///
    /// 从 AddCorpse 移植
    pub fn add_corpse(&mut self, x: usize, y: usize, corpse_id: i8, direction: Direction) {
        if x < MAXDUNX && y < MAXDUNY {
            // dCorpse[tilePosition.x][tilePosition.y] = (dv & 0x1F) + (static_cast<int>(ddir) << 5);
            self.corpse_map[x][y] = ((corpse_id & 0x1F) as u8) + ((direction as u8) << 5);
        }
    }

    /// 获取指定位置的尸体ID
    pub fn get_corpse_at(&self, x: usize, y: usize) -> Option<(i8, Direction)> {
        if x >= MAXDUNX || y >= MAXDUNY {
            return None;
        }

        let value = self.corpse_map[x][y];
        if value == 0 {
            return None;
        }

        let corpse_id = (value & 0x1F) as i8;
        let direction = DirectionFromU8::from_u8(value >> 5);
        Some((corpse_id, direction))
    }

    /// 移除指定位置的尸体
    pub fn remove_corpse(&mut self, x: usize, y: usize) {
        if x < MAXDUNX && y < MAXDUNY {
            self.corpse_map[x][y] = 0;
        }
    }

    /// 清除所有尸体
    pub fn clear(&mut self) {
        for row in &mut self.corpse_map {
            for cell in row {
                *cell = 0;
            }
        }
    }

    /// 获取尸体
    pub fn get_corpse(&self, id: i8) -> Option<&Corpse> {
        if id > 0 && (id as usize) <= self.corpses.len() {
            Some(&self.corpses[(id - 1) as usize])
        } else {
            None
        }
    }

    /// 统计尸体数量
    pub fn count_corpses(&self) -> usize {
        let mut count = 0;
        for row in &self.corpse_map {
            for &cell in row {
                if cell != 0 {
                    count += 1;
                }
            }
        }
        count
    }
}

/// 添加尸体（全局函数）
///
/// 从 AddCorpse 移植
pub fn add_corpse(x: usize, y: usize, corpse_id: i8, direction: Direction) {
    unsafe {
        if x < MAXDUNX && y < MAXDUNY {
            D_CORPSE[x][y] = ((corpse_id & 0x1F) as u8) + ((direction as u8) << 5);
        }
    }
}

/// 初始化尸体系统（全局函数）
///
/// 从 InitCorpses 移植
///
/// # Safety
/// 必须在游戏初始化时调用
pub unsafe fn init_corpses() {
    STONENDX = 0;
    for corpse in &mut CORPSES {
        *corpse = Corpse::default();
    }
}

/// 移动光源到尸体位置（占位实现）
///
/// 从 MoveLightsToCorpses 移植
pub fn move_lights_to_corpses(_monsters: &[Monster], _active_monsters: &[usize]) {
    // 实际实现需要与光照系统集成
    // for (size_t i = 0; i < ActiveMonsterCount; i++) {
    //     auto &monster = Monsters[ActiveMonsters[i]];
    //     if (!monster.isUnique())
    //         continue;
    //     MoveLightToCorpse(monster);
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpse_default() {
        let corpse = Corpse::default();
        assert!(corpse.sprites.is_none());
        assert_eq!(corpse.frame, 0);
        assert_eq!(corpse.width, 0);
    }

    #[test]
    fn test_corpse_manager_creation() {
        let manager = CorpseManager::new();
        assert_eq!(manager.corpses.len(), MAX_CORPSES);
        assert_eq!(manager.stone_index, 0);
    }

    #[test]
    fn test_add_corpse() {
        let mut manager = CorpseManager::new();

        manager.add_corpse(10, 20, 5, Direction::South);

        let result = manager.get_corpse_at(10, 20);
        assert!(result.is_some());
        let (id, dir) = result.unwrap();
        assert_eq!(id, 5);
        assert_eq!(dir, Direction::South);
    }

    #[test]
    fn test_remove_corpse() {
        let mut manager = CorpseManager::new();

        manager.add_corpse(10, 20, 5, Direction::South);
        assert!(manager.get_corpse_at(10, 20).is_some());

        manager.remove_corpse(10, 20);
        assert!(manager.get_corpse_at(10, 20).is_none());
    }

    #[test]
    fn test_corpse_count() {
        let mut manager = CorpseManager::new();

        assert_eq!(manager.count_corpses(), 0);

        manager.add_corpse(10, 20, 1, Direction::South);
        manager.add_corpse(30, 40, 2, Direction::North);

        assert_eq!(manager.count_corpses(), 2);
    }

    #[test]
    fn test_clear_corpses() {
        let mut manager = CorpseManager::new();

        manager.add_corpse(10, 20, 1, Direction::South);
        manager.add_corpse(30, 40, 2, Direction::North);

        manager.clear();
        assert_eq!(manager.count_corpses(), 0);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut manager = CorpseManager::new();

        // 边界外操作应该安全处理
        manager.add_corpse(MAXDUNX + 1, MAXDUNY + 1, 1, Direction::South);
        assert!(manager.get_corpse_at(MAXDUNX + 1, MAXDUNY + 1).is_none());
    }
}
