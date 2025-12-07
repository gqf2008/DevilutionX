//! 多人游戏同步系统
//!
//! 移植自 Source/sync.cpp
//! 负责多玩家游戏中的怪物状态同步、物品同步和玩家库存同步

use crate::game::types::Point;

/// 最大怪物数量
pub const MAX_MONSTERS: usize = 200;
/// 最大玩家数量
pub const MAX_PLRS: usize = 4;
/// 最大物品数量
pub const MAX_ITEMS: usize = 127;
/// 装备位置数量
pub const NUM_INVLOC: usize = 7;

/// 同步命令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncCommand {
    /// 同步数据命令
    SyncData,
    /// 物品同步
    ItemSync,
    /// 怪物同步
    MonsterSync,
}

/// 怪物同步数据包
#[derive(Debug, Clone, Copy, Default)]
pub struct TSyncMonster {
    /// 怪物索引
    pub monster_index: u16,
    /// X坐标
    pub x: u8,
    /// Y坐标
    pub y: u8,
    /// 敌人编码
    pub enemy: u8,
    /// 优先级增量
    pub delta: u8,
    /// 击中者掩码
    pub who_hit: u8,
    /// 生命值
    pub hit_points: i32,
}

impl TSyncMonster {
    /// 创建新的怪物同步数据
    pub fn new(index: u16, x: u8, y: u8) -> Self {
        Self {
            monster_index: index,
            x,
            y,
            enemy: 0,
            delta: 0,
            who_hit: 0,
            hit_points: 0,
        }
    }

    /// 获取位置
    pub fn position(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }

    /// 验证怪物同步数据是否有效
    pub fn is_valid(&self) -> bool {
        (self.monster_index as usize) < MAX_MONSTERS
            && self.x < 112
            && self.y < 112
    }
}

/// 同步头部数据
#[derive(Debug, Clone, Default)]
pub struct TSyncHeader {
    /// 命令类型
    pub cmd: u8,
    /// 关卡
    pub level: u8,
    /// 数据长度
    pub len: u16,
    /// 物品索引
    pub item_index: i8,
    /// 物品X坐标
    pub item_x: u8,
    /// 物品Y坐标
    pub item_y: u8,
    /// 物品ID
    pub item_id: u16,
    /// 物品创建信息
    pub item_ci: u16,
    /// 物品种子
    pub item_seed: u32,
    /// 物品鉴定状态
    pub item_identified: u8,
    /// 物品耐久度
    pub item_durability: u8,
    /// 物品最大耐久度
    pub item_max_durability: u8,
    /// 物品充能
    pub item_charges: u8,
    /// 物品最大充能
    pub item_max_charges: u8,
    /// 物品价值
    pub item_value: u16,
    /// 物品buff
    pub item_buff: u32,
    /// 玩家库存位置
    pub player_inv_loc: i8,
    /// 玩家库存物品索引
    pub player_inv_index: u16,
    /// 玩家库存创建信息
    pub player_inv_ci: u16,
    /// 玩家库存种子
    pub player_inv_seed: u32,
    /// 玩家库存鉴定状态
    pub player_inv_identified: u8,
}

/// 怪物同步优先级管理器
#[derive(Debug)]
pub struct MonsterSyncPriority {
    /// 怪物优先级数组
    priorities: [u16; MAX_MONSTERS],
    /// LRU缓存
    lru: [u16; MAX_MONSTERS],
    /// 当前同步怪物索引
    current_index: usize,
    /// 同步物品索引
    sync_item: usize,
    /// 同步玩家库存索引
    sync_inv: usize,
}

impl Default for MonsterSyncPriority {
    fn default() -> Self {
        Self::new()
    }
}

impl MonsterSyncPriority {
    /// 创建新的优先级管理器
    pub fn new() -> Self {
        Self {
            priorities: [0; MAX_MONSTERS],
            lru: [0xFFFF; MAX_MONSTERS],
            current_index: 0,
            sync_item: 0,
            sync_inv: 0,
        }
    }

    /// 初始化同步状态
    pub fn init(&mut self, player_id: usize) {
        self.current_index = 16 * player_id;
        self.lru = [0xFFFF; MAX_MONSTERS];
    }

    /// 更新单个怪物的优先级
    pub fn update_priority(&mut self, monster_index: usize, distance: u16, is_active: bool) {
        if monster_index >= MAX_MONSTERS {
            return;
        }

        let mut priority = distance;
        if !is_active {
            priority = priority.saturating_add(0x1000);
        } else if self.lru[monster_index] > 0 {
            self.lru[monster_index] -= 1;
        }
        self.priorities[monster_index] = priority;
    }

    /// 选择下一个要同步的活跃怪物（基于优先级）
    pub fn select_active_monster(&mut self, active_monsters: &[usize]) -> Option<usize> {
        let mut best_index = None;
        let mut best_lru = 0xFFFFFFFF_u32;

        for &monster_index in active_monsters {
            if monster_index >= MAX_MONSTERS {
                continue;
            }
            let priority = self.priorities[monster_index] as u32;
            if priority < best_lru && self.lru[monster_index] < 0xFFFE {
                best_lru = priority;
                best_index = Some(monster_index);
            }
        }

        if let Some(index) = best_index {
            self.mark_synced(index);
        }

        best_index
    }

    /// 轮询选择下一个怪物（用于第二优先级同步）
    pub fn select_monster_round_robin(&mut self, active_monsters: &[usize]) -> Option<usize> {
        if active_monsters.is_empty() {
            return None;
        }

        let count = active_monsters.len();
        let mut best_index = None;
        let mut best_lru = 0xFFFE_u16;

        for _ in 0..count {
            if self.current_index >= count {
                self.current_index = 0;
            }
            let monster_index = active_monsters[self.current_index];
            if monster_index < MAX_MONSTERS && self.lru[monster_index] < best_lru {
                best_lru = self.lru[monster_index];
                best_index = Some(monster_index);
            }
            self.current_index += 1;
        }

        if let Some(index) = best_index {
            self.mark_synced(index);
        }

        best_index
    }

    /// 标记怪物已同步
    fn mark_synced(&mut self, monster_index: usize) {
        if monster_index < MAX_MONSTERS {
            self.priorities[monster_index] = 0xFFFF;
            // 活跃怪物设置为0xFFFE，非活跃设置为0xFFFF
            self.lru[monster_index] = 0xFFFE;
        }
    }

    /// 获取下一个同步物品索引
    pub fn next_sync_item(&mut self, active_item_count: usize) -> Option<usize> {
        if active_item_count == 0 {
            return None;
        }
        if self.sync_item >= active_item_count {
            self.sync_item = 0;
        }
        let index = self.sync_item;
        self.sync_item += 1;
        Some(index)
    }

    /// 获取下一个同步玩家库存位置
    pub fn next_sync_inv(&mut self) -> usize {
        let loc = self.sync_inv;
        self.sync_inv += 1;
        if self.sync_inv >= NUM_INVLOC {
            self.sync_inv = 0;
        }
        loc
    }
}

/// 游戏同步管理器
#[derive(Debug)]
pub struct SyncManager {
    /// 怪物优先级管理
    pub monster_priority: MonsterSyncPriority,
    /// 本地玩家ID
    pub my_player_id: usize,
    /// 消息缓冲模式
    pub buffer_msgs: u8,
    /// 是否正在切换关卡
    pub level_changing: bool,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SyncManager {
    /// 创建新的同步管理器
    pub fn new(player_id: usize) -> Self {
        let mut manager = Self {
            monster_priority: MonsterSyncPriority::new(),
            my_player_id: player_id,
            buffer_msgs: 0,
            level_changing: false,
        };
        manager.monster_priority.init(player_id);
        manager
    }

    /// 初始化同步
    pub fn init(&mut self, player_id: usize) {
        self.my_player_id = player_id;
        self.monster_priority.init(player_id);
    }

    /// 同步所有怪物
    pub fn sync_all_monsters(
        &mut self,
        buffer: &mut Vec<u8>,
        max_len: usize,
        active_monsters: &[usize],
        player_position: Point,
        _current_level: u8,
        get_monster_info: impl Fn(usize) -> Option<MonsterSyncInfo>,
    ) -> usize {
        if active_monsters.is_empty() {
            return max_len;
        }

        let header_size = std::mem::size_of::<TSyncHeader>();
        let monster_size = std::mem::size_of::<TSyncMonster>();

        if max_len < header_size + monster_size {
            return max_len;
        }

        if self.level_changing {
            return max_len;
        }

        // 更新所有怪物的优先级
        for &monster_index in active_monsters {
            if let Some(info) = get_monster_info(monster_index) {
                // 使用自定义曼哈顿距离计算
                let dx = (player_position.x - info.position.x).abs();
                let dy = (player_position.y - info.position.y).abs();
                let distance = (dx + dy) as u16;
                self.monster_priority.update_priority(monster_index, distance, info.is_active);
            }
        }

        let mut remaining = max_len - header_size;
        let mut _synced_count = 0;

        // 同步怪物
        for i in 0..active_monsters.len() {
            if remaining < monster_size {
                break;
            }

            let monster_index = if i < 2 {
                // 前两个使用轮询
                self.monster_priority.select_monster_round_robin(active_monsters)
            } else {
                None
            };

            let monster_index = monster_index
                .or_else(|| self.monster_priority.select_active_monster(active_monsters));

            if let Some(index) = monster_index {
                if let Some(info) = get_monster_info(index) {
                    let sync = TSyncMonster {
                        monster_index: index as u16,
                        x: info.position.x as u8,
                        y: info.position.y as u8,
                        enemy: info.enemy_id,
                        delta: self.monster_priority.priorities[index].min(255) as u8,
                        who_hit: info.who_hit,
                        hit_points: info.hit_points,
                    };

                    // 序列化怪物数据（简化版）
                    buffer.extend_from_slice(&sync.monster_index.to_le_bytes());
                    buffer.push(sync.x);
                    buffer.push(sync.y);
                    buffer.push(sync.enemy);
                    buffer.push(sync.delta);
                    buffer.push(sync.who_hit);
                    buffer.extend_from_slice(&sync.hit_points.to_le_bytes());

                    remaining -= monster_size;
                    _synced_count += 1;
                }
            } else {
                break;
            }
        }

        remaining
    }

    /// 处理接收到的同步数据
    pub fn on_sync_data(
        &mut self,
        _header: &TSyncHeader,
        monster_syncs: &[TSyncMonster],
        sender_id: usize,
        sender_level: u8,
        my_level: u8,
    ) -> bool {
        if self.buffer_msgs == 2 {
            return false;
        }

        if self.buffer_msgs == 1 {
            return true;
        }

        if sender_id == self.my_player_id {
            return true;
        }

        let sync_local = !self.level_changing && my_level == sender_level;
        let is_owner = sender_id > self.my_player_id;

        for sync in monster_syncs {
            if !sync.is_valid() {
                continue;
            }

            if sync_local {
                // 同步本地怪物状态
                self.sync_monster(sync, is_owner);
            }

            // 保存到delta同步
            self.delta_sync_monster(sync, sender_level);
        }

        true
    }

    /// 同步单个怪物
    fn sync_monster(&self, _sync: &TSyncMonster, _is_owner: bool) {
        // 实际实现会更新本地怪物状态
        // 这里是简化版本
    }

    /// Delta同步怪物（用于关卡切换）
    fn delta_sync_monster(&self, _sync: &TSyncMonster, _level: u8) {
        // 保存到delta存储用于关卡切换时恢复
    }

    /// 验证同步怪物数据
    pub fn validate_sync_monster(sync: &TSyncMonster) -> bool {
        if (sync.monster_index as usize) >= MAX_MONSTERS {
            return false;
        }

        if sync.x >= 112 || sync.y >= 112 {
            return false;
        }

        true
    }
}

/// 怪物同步信息（用于回调）
#[derive(Debug, Clone)]
pub struct MonsterSyncInfo {
    /// 位置
    pub position: Point,
    /// 是否活跃
    pub is_active: bool,
    /// 敌人ID编码
    pub enemy_id: u8,
    /// 击中者掩码
    pub who_hit: u8,
    /// 生命值
    pub hit_points: i32,
}

/// 编码敌人ID
pub fn encode_enemy(target_type: u8, target_id: u8) -> u8 {
    (target_type << 5) | (target_id & 0x1F)
}

/// 解码敌人ID
pub fn decode_enemy(encoded: u8) -> (u8, u8) {
    let target_type = encoded >> 5;
    let target_id = encoded & 0x1F;
    (target_type, target_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_monster_new() {
        let sync = TSyncMonster::new(42, 10, 20);
        assert_eq!(sync.monster_index, 42);
        assert_eq!(sync.x, 10);
        assert_eq!(sync.y, 20);
    }

    #[test]
    fn test_sync_monster_position() {
        let sync = TSyncMonster::new(0, 15, 25);
        let pos = sync.position();
        assert_eq!(pos.x, 15);
        assert_eq!(pos.y, 25);
    }

    #[test]
    fn test_sync_monster_valid() {
        let valid = TSyncMonster::new(100, 50, 50);
        assert!(valid.is_valid());

        let invalid_index = TSyncMonster::new(250, 50, 50);
        assert!(!invalid_index.is_valid());

        let invalid_pos = TSyncMonster::new(0, 120, 50);
        assert!(!invalid_pos.is_valid());
    }

    #[test]
    fn test_monster_sync_priority_init() {
        let mut priority = MonsterSyncPriority::new();
        priority.init(2);
        assert_eq!(priority.current_index, 32);
    }

    #[test]
    fn test_monster_sync_priority_update() {
        let mut priority = MonsterSyncPriority::new();
        priority.update_priority(5, 100, true);
        assert_eq!(priority.priorities[5], 100);

        priority.update_priority(10, 50, false);
        // 非活跃怪物增加0x1000
        assert_eq!(priority.priorities[10], 50 + 0x1000);
    }

    #[test]
    fn test_monster_sync_priority_select() {
        let mut priority = MonsterSyncPriority::new();
        let active = vec![1, 2, 3, 4, 5];

        // 设置优先级
        for (i, &m) in active.iter().enumerate() {
            priority.priorities[m] = (i * 10) as u16;
            priority.lru[m] = 0;
        }

        // 应该选择优先级最低的
        let selected = priority.select_active_monster(&active);
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn test_sync_manager_new() {
        let manager = SyncManager::new(1);
        assert_eq!(manager.my_player_id, 1);
        assert_eq!(manager.buffer_msgs, 0);
        assert!(!manager.level_changing);
    }

    #[test]
    fn test_sync_manager_init() {
        let mut manager = SyncManager::new(0);
        manager.init(2);
        assert_eq!(manager.my_player_id, 2);
    }

    #[test]
    fn test_encode_decode_enemy() {
        let target_type = 2u8;
        let target_id = 15u8;
        let encoded = encode_enemy(target_type, target_id);
        let (decoded_type, decoded_id) = decode_enemy(encoded);
        assert_eq!(decoded_type, target_type);
        assert_eq!(decoded_id, target_id);
    }

    #[test]
    fn test_next_sync_item() {
        let mut priority = MonsterSyncPriority::new();

        // 空列表
        assert!(priority.next_sync_item(0).is_none());

        // 正常迭代
        assert_eq!(priority.next_sync_item(5), Some(0));
        assert_eq!(priority.next_sync_item(5), Some(1));
        assert_eq!(priority.next_sync_item(5), Some(2));
    }

    #[test]
    fn test_next_sync_inv() {
        let mut priority = MonsterSyncPriority::new();

        for i in 0..NUM_INVLOC {
            assert_eq!(priority.next_sync_inv(), i);
        }
        // 应该循环回到0
        assert_eq!(priority.next_sync_inv(), 0);
    }

    #[test]
    fn test_validate_sync_monster() {
        let valid = TSyncMonster::new(50, 60, 70);
        assert!(SyncManager::validate_sync_monster(&valid));

        let invalid = TSyncMonster::new(250, 60, 70);
        assert!(!SyncManager::validate_sync_monster(&invalid));
    }
}
