//! 传送门系统 (M35)
//!
//! 从 Source/portal.cpp 移植
//! 处理城镇传送门的创建、激活和管理

use crate::game::types::{DungeonType, Point};

/// 最大传送门数量（等于最大玩家数）
pub const MAX_PORTAL: usize = 4;

/// 传送门状态
#[derive(Debug, Clone, Default)]
pub struct Portal {
    /// 是否开启
    pub open: bool,
    /// 传送门位置
    pub position: Point,
    /// 目标关卡
    pub level: i32,
    /// 关卡类型
    pub ltype: DungeonType,
    /// 是否为设定关卡（特殊关卡）
    pub setlvl: bool,
}

impl Portal {
    /// 创建新传送门
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查传送门是否有效
    pub fn is_valid(&self) -> bool {
        self.open && self.level > 0
    }

    /// 重置传送门
    pub fn reset(&mut self) {
        self.open = false;
        self.position = Point::default();
        self.level = 0;
        self.ltype = DungeonType::default();
        self.setlvl = false;
    }
}

/// 城镇中每个玩家传送门的位置
pub const PORTAL_TOWN_POSITIONS: [Point; MAX_PORTAL] = [
    Point { x: 57, y: 40 },
    Point { x: 59, y: 40 },
    Point { x: 61, y: 40 },
    Point { x: 63, y: 40 },
];

/// 传送门管理器
#[derive(Debug)]
pub struct PortalManager {
    /// 所有传送门
    pub portals: [Portal; MAX_PORTAL],
    /// 当前传送门索引
    pub current_index: usize,
}

impl Default for PortalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PortalManager {
    /// 创建新的传送门管理器
    pub fn new() -> Self {
        Self {
            portals: std::array::from_fn(|_| Portal::new()),
            current_index: 0,
        }
    }

    /// 初始化所有传送门
    ///
    /// 从 InitPortals 移植
    pub fn init(&mut self) {
        for portal in &mut self.portals {
            portal.reset();
        }
        self.current_index = 0;
    }

    /// 设置传送门状态
    ///
    /// 从 SetPortalStats 移植
    pub fn set_portal_stats(
        &mut self,
        index: usize,
        open: bool,
        position: Point,
        level: i32,
        ltype: DungeonType,
        is_set_level: bool,
    ) {
        if index >= MAX_PORTAL {
            return;
        }

        let portal = &mut self.portals[index];
        portal.open = open;
        portal.position = position;
        portal.level = level;
        portal.ltype = ltype;
        portal.setlvl = is_set_level;
    }

    /// 激活传送门
    ///
    /// 从 ActivatePortal 移植
    pub fn activate(
        &mut self,
        player_id: usize,
        position: Point,
        level: i32,
        dungeon_type: DungeonType,
        is_set_level: bool,
    ) {
        if player_id >= MAX_PORTAL {
            return;
        }

        let portal = &mut self.portals[player_id];
        portal.open = true;

        if level != 0 {
            portal.position = position;
            portal.level = level;
            portal.ltype = dungeon_type;
            portal.setlvl = is_set_level;
        }
    }

    /// 停用传送门
    ///
    /// 从 DeactivatePortal 移植
    pub fn deactivate(&mut self, player_id: usize) {
        if player_id < MAX_PORTAL {
            self.portals[player_id].open = false;
        }
    }

    /// 检查玩家的传送门是否在当前关卡
    ///
    /// 从 PortalOnLevel 移植
    pub fn portal_on_level(
        &self,
        player_id: usize,
        current_level: i32,
        set_level: bool,
        set_level_num: i32,
        is_town: bool,
    ) -> bool {
        if player_id >= MAX_PORTAL {
            return false;
        }

        let portal = &self.portals[player_id];

        // 在城镇总是返回true
        if is_town {
            return true;
        }

        // 检查传送门是否在当前关卡
        if portal.setlvl == set_level {
            let target_level = if set_level { set_level_num } else { current_level };
            if portal.level == target_level {
                return true;
            }
        }

        false
    }

    /// 设置当前传送门索引
    ///
    /// 从 SetCurrentPortal 移植
    pub fn set_current(&mut self, index: usize) {
        if index < MAX_PORTAL {
            self.current_index = index;
        }
    }

    /// 获取当前传送门
    pub fn get_current(&self) -> Option<&Portal> {
        self.portals.get(self.current_index)
    }

    /// 获取传送门位置
    ///
    /// 从 GetPortalLvlPos 移植
    pub fn get_portal_position(&self, is_town: bool, is_my_player: bool) -> Point {
        if is_town {
            let pos = PORTAL_TOWN_POSITIONS[self.current_index];
            Point {
                x: pos.x + 1,
                y: pos.y + 1,
            }
        } else if let Some(portal) = self.get_current() {
            if is_my_player {
                portal.position
            } else {
                Point {
                    x: portal.position.x + 1,
                    y: portal.position.y + 1,
                }
            }
        } else {
            Point::default()
        }
    }

    /// 检查位置是否有传送门
    ///
    /// 从 PosOkPortal 移植
    pub fn pos_ok_portal(&self, level: i32, position: Point) -> bool {
        for portal in &self.portals {
            if portal.open && portal.level == level {
                if portal.position == position {
                    return true;
                }
                // 检查偏移位置
                if portal.position.x == position.x + 1 && portal.position.y == position.y + 1 {
                    return true;
                }
            }
        }
        false
    }

    /// 获取指定玩家的传送门
    pub fn get_portal(&self, player_id: usize) -> Option<&Portal> {
        self.portals.get(player_id)
    }

    /// 获取指定玩家的传送门（可变）
    pub fn get_portal_mut(&mut self, player_id: usize) -> Option<&mut Portal> {
        self.portals.get_mut(player_id)
    }

    /// 统计活跃传送门数量
    pub fn active_count(&self) -> usize {
        self.portals.iter().filter(|p| p.open).count()
    }

    /// 获取城镇传送门位置
    pub fn get_town_position(player_id: usize) -> Point {
        if player_id < MAX_PORTAL {
            PORTAL_TOWN_POSITIONS[player_id]
        } else {
            PORTAL_TOWN_POSITIONS[0]
        }
    }
}

/// 全局传送门管理器
pub static mut PORTAL_MANAGER: Option<PortalManager> = None;

/// 初始化传送门系统
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn init_portals() {
    PORTAL_MANAGER = Some(PortalManager::new());
    if let Some(ref mut manager) = PORTAL_MANAGER {
        manager.init();
    }
}

/// 激活传送门
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn activate_portal(
    player_id: usize,
    position: Point,
    level: i32,
    dungeon_type: DungeonType,
    is_set_level: bool,
) {
    if let Some(ref mut manager) = PORTAL_MANAGER {
        manager.activate(player_id, position, level, dungeon_type, is_set_level);
    }
}

/// 停用传送门
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn deactivate_portal(player_id: usize) {
    if let Some(ref mut manager) = PORTAL_MANAGER {
        manager.deactivate(player_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portal_default() {
        let portal = Portal::new();
        assert!(!portal.open);
        assert_eq!(portal.level, 0);
    }

    #[test]
    fn test_portal_manager_creation() {
        let manager = PortalManager::new();
        assert_eq!(manager.portals.len(), MAX_PORTAL);
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_portal_activation() {
        let mut manager = PortalManager::new();

        manager.activate(
            0,
            Point { x: 10, y: 20 },
            5,
            DungeonType::Cathedral,
            false,
        );

        assert!(manager.portals[0].open);
        assert_eq!(manager.portals[0].level, 5);
        assert_eq!(manager.portals[0].position.x, 10);
    }

    #[test]
    fn test_portal_deactivation() {
        let mut manager = PortalManager::new();

        manager.activate(
            0,
            Point { x: 10, y: 20 },
            5,
            DungeonType::Cathedral,
            false,
        );
        assert!(manager.portals[0].open);

        manager.deactivate(0);
        assert!(!manager.portals[0].open);
    }

    #[test]
    fn test_portal_on_level() {
        let mut manager = PortalManager::new();

        manager.activate(
            0,
            Point { x: 10, y: 20 },
            5,
            DungeonType::Cathedral,
            false,
        );

        // 在城镇总是返回true
        assert!(manager.portal_on_level(0, 0, false, 0, true));

        // 在正确的关卡
        assert!(manager.portal_on_level(0, 5, false, 0, false));

        // 在错误的关卡
        assert!(!manager.portal_on_level(0, 3, false, 0, false));
    }

    #[test]
    fn test_pos_ok_portal() {
        let mut manager = PortalManager::new();

        manager.activate(
            0,
            Point { x: 10, y: 20 },
            5,
            DungeonType::Cathedral,
            false,
        );

        assert!(manager.pos_ok_portal(5, Point { x: 10, y: 20 }));
        assert!(!manager.pos_ok_portal(5, Point { x: 15, y: 25 }));
        assert!(!manager.pos_ok_portal(3, Point { x: 10, y: 20 })); // 错误关卡
    }

    #[test]
    fn test_town_positions() {
        assert_eq!(PortalManager::get_town_position(0), Point { x: 57, y: 40 });
        assert_eq!(PortalManager::get_town_position(1), Point { x: 59, y: 40 });
        assert_eq!(PortalManager::get_town_position(2), Point { x: 61, y: 40 });
        assert_eq!(PortalManager::get_town_position(3), Point { x: 63, y: 40 });
    }

    #[test]
    fn test_active_count() {
        let mut manager = PortalManager::new();
        assert_eq!(manager.active_count(), 0);

        manager.activate(0, Point { x: 10, y: 20 }, 5, DungeonType::Cathedral, false);
        assert_eq!(manager.active_count(), 1);

        manager.activate(1, Point { x: 15, y: 25 }, 3, DungeonType::Caves, false);
        assert_eq!(manager.active_count(), 2);
    }
}
