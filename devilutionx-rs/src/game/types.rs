//! Core game types - based on DevilutionX Source/diablo.h
//!
//! Contains fundamental type definitions used throughout the game.

use serde::{Deserialize, Serialize};

/// Number of dungeon levels
pub const NUM_LEVELS: usize = 25;

/// Maximum number of monsters in a level
pub const MAX_MONSTERS: usize = 200;

/// Maximum number of items on ground
pub const MAX_ITEMS: usize = 127;

/// Maximum number of missiles
pub const MAX_MISSILES: usize = 125;

/// Maximum number of objects
pub const MAX_OBJECTS: usize = 127;

/// Player name maximum length
pub const PLAYER_NAME_LENGTH: usize = 32;

/// Inventory grid cells
pub const INVENTORY_GRID_CELLS: usize = 40;

/// Maximum belt items
pub const MAX_BELT_ITEMS: usize = 8;

/// Maximum resistance value
pub const MAX_RESISTANCE: i32 = 75;

/// Maximum spell level
pub const MAX_SPELL_LEVEL: u8 = 15;

/// Click type for mouse input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClickType {
    #[default]
    None,
    Left,
    Right,
}

/// Game logic processing step (for debugging/profiling)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameLogicStep {
    #[default]
    None,
    ProcessPlayers,
    ProcessMonsters,
    ProcessObjects,
    ProcessMissiles,
    ProcessItems,
    ProcessTowners,
    ProcessItemsTown,
    ProcessMissilesTown,
}

/// Player action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerActionType {
    #[default]
    None,
    Walk,
    Spell,
    SpellMonsterTarget,
    SpellPlayerTarget,
    Attack,
    AttackMonsterTarget,
    AttackPlayerTarget,
    OperateObject,
}

/// Dungeon types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DungeonType {
    None,
    Town,
    Cathedral,  // L1
    Catacombs,  // L2
    Caves,      // L3
    Hell,       // L4
    Nest,       // Hellfire
    Crypt,      // Hellfire
}

impl Default for DungeonType {
    fn default() -> Self {
        Self::None
    }
}

/// Game difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Difficulty {
    #[default]
    Normal,
    Nightmare,
    Hell,
}

/// Direction enum (8 directions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    South,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
}

impl Direction {
    /// All 8 directions
    pub const ALL: [Direction; 8] = [
        Direction::South,
        Direction::SouthWest,
        Direction::West,
        Direction::NorthWest,
        Direction::North,
        Direction::NorthEast,
        Direction::East,
        Direction::SouthEast,
    ];

    /// Get direction index (0-7)
    pub fn index(self) -> usize {
        self as usize
    }

    /// Get delta X for this direction
    /// Reference: C++ DisplacementOf::fromDirection() in Source/engine/displacement.hpp:247-270
    /// Diablo uses an isometric coordinate system where:
    /// - X axis points to the bottom-right
    /// - Y axis points to the bottom-left
    /// - Both axes form a 45-degree rotated diamond grid
    pub fn dx(self) -> i32 {
        match self {
            Direction::South => 1,        // { 1, 1 }
            Direction::SouthWest => 0,    // { 0, 1 }
            Direction::West => -1,        // {-1, 1 }
            Direction::NorthWest => -1,   // {-1, 0 }
            Direction::North => -1,       // {-1,-1 }
            Direction::NorthEast => 0,    // { 0,-1 }
            Direction::East => 1,         // { 1,-1 }
            Direction::SouthEast => 1,    // { 1, 0 }
        }
    }

    /// Get delta Y for this direction
    /// Reference: C++ DisplacementOf::fromDirection() in Source/engine/displacement.hpp:247-270
    pub fn dy(self) -> i32 {
        match self {
            Direction::South => 1,        // { 1, 1 }
            Direction::SouthWest => 1,    // { 0, 1 }
            Direction::West => 1,         // {-1, 1 }
            Direction::NorthWest => 0,    // {-1, 0 }
            Direction::North => -1,       // {-1,-1 }
            Direction::NorthEast => -1,   // { 0,-1 }
            Direction::East => -1,        // { 1,-1 }
            Direction::SouthEast => 0,    // { 1, 0 }
        }
    }

    /// Get opposite direction
    pub fn opposite(self) -> Direction {
        match self {
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
        }
    }

    /// Rotate direction 45 degrees to the left (counter-clockwise)
    pub fn left(self) -> Direction {
        Direction::ALL[(self as usize + 7) % 8]
    }

    /// Rotate direction 45 degrees to the right (clockwise)
    pub fn right(self) -> Direction {
        Direction::ALL[(self as usize + 1) % 8]
    }

    /// Get offset Point for this direction
    pub fn offset(self) -> Point {
        Point::new(self.dx(), self.dy())
    }

    /// Get direction from delta X and Y
    ///
    /// **C++ Reference**: `GetDirection()` in Source/engine/direction.cpp
    pub fn from_delta(dx: i32, dy: i32) -> Direction {
        // Normalize to -1, 0, 1
        let ndx = if dx > 0 { 1 } else if dx < 0 { -1 } else { 0 };
        let ndy = if dy > 0 { 1 } else if dy < 0 { -1 } else { 0 };

        match (ndx, ndy) {
            (0, -1) => Direction::NorthEast,
            (0, 1) => Direction::SouthWest,
            (-1, 0) => Direction::NorthWest,
            (1, 0) => Direction::SouthEast,
            (-1, -1) => Direction::North,
            (1, -1) => Direction::East,
            (-1, 1) => Direction::West,
            (1, 1) => Direction::South,
            _ => Direction::South, // Default
        }
    }
}

/// 2D Point structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Point = Point::new(0, 0);

    /// Manhattan distance to another point
    pub fn distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Move in direction
    pub fn shift(&self, dir: Direction) -> Point {
        Point::new(self.x + dir.dx(), self.y + dir.dy())
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, rhs: Self) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

/// Rectangle structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rectangle {
    pub position: Point,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            position: Point::new(x, y),
            width,
            height,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.width
            && point.y >= self.position.y
            && point.y < self.position.y + self.height
    }
}

// =============================================================================
// Phase 1: 核心类型对齐 - 与 C++ DevilutionX 精确对齐
// Reference: Source/engine/*.hpp
// =============================================================================

/// 位移结构体 - 用于偏移计算
/// Reference: C++ Source/engine/displacement.hpp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Displacement {
    pub delta_x: i32,
    pub delta_y: i32,
}

impl Displacement {
    pub const fn new(delta_x: i32, delta_y: i32) -> Self {
        Self { delta_x, delta_y }
    }

    pub const ZERO: Displacement = Displacement::new(0, 0);

    /// 从方向创建位移
    pub fn from_direction(dir: Direction) -> Self {
        Self::new(dir.dx(), dir.dy())
    }

    /// 计算长度
    pub fn magnitude(&self) -> f32 {
        ((self.delta_x * self.delta_x + self.delta_y * self.delta_y) as f32).sqrt()
    }

    /// 世界坐标转屏幕坐标
    /// Reference: C++ displacement.hpp worldToScreen()
    pub fn world_to_screen(&self) -> Displacement {
        Displacement::new(
            32 * (self.delta_y - self.delta_x),
            -16 * (self.delta_y + self.delta_x),
        )
    }

    /// 屏幕坐标转世界坐标
    /// Reference: C++ displacement.hpp screenToWorld()
    pub fn screen_to_world(&self) -> Displacement {
        Displacement::new(
            (self.delta_y - self.delta_x / 2) / 16,
            (self.delta_y + self.delta_x / 2) / 16,
        )
    }
}

impl std::ops::Add for Displacement {
    type Output = Displacement;
    fn add(self, rhs: Self) -> Self::Output {
        Displacement::new(self.delta_x + rhs.delta_x, self.delta_y + rhs.delta_y)
    }
}

impl std::ops::Sub for Displacement {
    type Output = Displacement;
    fn sub(self, rhs: Self) -> Self::Output {
        Displacement::new(self.delta_x - rhs.delta_x, self.delta_y - rhs.delta_y)
    }
}

impl std::ops::Mul<i32> for Displacement {
    type Output = Displacement;
    fn mul(self, rhs: i32) -> Self::Output {
        Displacement::new(self.delta_x * rhs, self.delta_y * rhs)
    }
}

/// 世界瓦片位置 - 用于精确的世界坐标
/// Reference: C++ Source/engine/world_tile.hpp WorldTilePosition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorldTilePosition {
    pub x: i16,
    pub y: i16,
}

impl WorldTilePosition {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const ZERO: WorldTilePosition = WorldTilePosition::new(0, 0);

    /// 转换为Point
    pub fn to_point(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }

    /// 从Point创建
    pub fn from_point(p: Point) -> Self {
        Self::new(p.x as i16, p.y as i16)
    }

    /// 曼哈顿距离
    pub fn manhattan_distance(&self, other: &WorldTilePosition) -> i32 {
        (self.x as i32 - other.x as i32).abs() + (self.y as i32 - other.y as i32).abs()
    }

    /// 行走距离 (切比雪夫距离)
    pub fn walking_distance(&self, other: &WorldTilePosition) -> i32 {
        let dx = (self.x as i32 - other.x as i32).abs();
        let dy = (self.y as i32 - other.y as i32).abs();
        dx.max(dy)
    }
}

impl std::ops::Add<Displacement> for WorldTilePosition {
    type Output = WorldTilePosition;
    fn add(self, rhs: Displacement) -> Self::Output {
        WorldTilePosition::new(
            self.x + rhs.delta_x as i16,
            self.y + rhs.delta_y as i16,
        )
    }
}

/// 角色位置系统 - 完整的位置追踪
/// Reference: C++ Source/engine/actor_position.hpp ActorPosition
#[derive(Debug, Clone, Copy, Default)]
pub struct ActorPosition {
    /// 当前瓦片位置
    pub tile: WorldTilePosition,
    /// 未来瓦片位置 (行走动画开始时设置)
    pub future: WorldTilePosition,
    /// 上一个瓦片位置 (用于网络同步)
    pub last: WorldTilePosition,
    /// 最近的 dPlayer 位置
    pub old: WorldTilePosition,
    /// 临时位置 (用于法术/远程攻击目标)
    pub temp: WorldTilePosition,
}

impl ActorPosition {
    pub fn new(x: i16, y: i16) -> Self {
        let pos = WorldTilePosition::new(x, y);
        Self {
            tile: pos,
            future: pos,
            last: pos,
            old: pos,
            temp: pos,
        }
    }

    /// 设置所有位置为同一点
    pub fn set_all(&mut self, pos: WorldTilePosition) {
        self.tile = pos;
        self.future = pos;
        self.last = pos;
        self.old = pos;
        self.temp = pos;
    }
}

// =============================================================================
// 物品特殊效果位标志 - 精确对齐 C++ ItemSpecialEffect
// Reference: C++ Source/itemdat.h
// =============================================================================

bitflags::bitflags! {
    /// 物品特殊效果
    /// Reference: C++ Source/itemdat.h ItemSpecialEffect
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ItemSpecialEffect: u32 {
        /// 无效果
        const NONE = 0;
        /// 红外视觉
        const INFRAVISION = 1 << 0;
        /// 随机箭矢速度
        const RANDOM_ARROW_VELOCITY = 1 << 1;
        /// 火焰箭矢
        const FIRE_ARROWS = 1 << 2;
        /// 火焰伤害
        const FIRE_DAMAGE = 1 << 3;
        /// 闪电伤害
        const LIGHTNING_DAMAGE = 1 << 4;
        /// 龙之呼吸 (火墙攻击)
        const DRAGON_BREATH = 1 << 5;
        /// 10%窃取生命
        const STEAL_LIFE_3 = 1 << 6;
        /// 5%窃取生命
        const STEAL_LIFE_5 = 1 << 7;
        /// 5%窃取法力
        const STEAL_MANA_3 = 1 << 8;
        /// 3%窃取法力
        const STEAL_MANA_5 = 1 << 9;
        /// 快速恢复
        const FAST_RECOVERY = 1 << 10;
        /// 更快恢复
        const FASTER_RECOVERY = 1 << 11;
        /// 最快恢复
        const FASTEST_RECOVERY = 1 << 12;
        /// 快速格挡
        const FAST_BLOCK = 1 << 13;
        /// 增加伤害
        const THORNS = 1 << 14;
        /// 击退
        const KNOCKBACK = 1 << 15;
        /// 3倍对恶魔伤害
        const TRIPLE_DEMON_DAMAGE = 1 << 16;
        /// 无法被击退
        const NO_KNOCKBACK = 1 << 17;
        /// +200%对不死伤害
        const UNDEAD_DAMAGE_BONUS = 1 << 18;
        /// 零魔法
        const ZERO_MANA = 1 << 19;
        /// 快速攻击
        const FAST_ATTACK = 1 << 20;
        /// 更快攻击
        const FASTER_ATTACK = 1 << 21;
        /// 最快攻击
        const FASTEST_ATTACK = 1 << 22;
        /// 快速命中恢复
        const FAST_HIT_RECOVERY = 1 << 23;
        /// 更快命中恢复
        const FASTER_HIT_RECOVERY = 1 << 24;
        /// 最快命中恢复
        const FASTEST_HIT_RECOVERY = 1 << 25;
        /// 穿透护甲
        const PIERCE_ARMOR = 1 << 26;
        /// 无耐久度损失
        const NO_DURABILITY_LOSS = 1 << 27;
        /// 吸血鬼
        const VAMPIRIC = 1 << 28;
        /// 法术等级加成
        const SPELL_LEVEL_BONUS = 1 << 29;
        /// 快速行走
        const FAST_WALK = 1 << 30;
        /// 更快行走
        const FASTER_WALK = 1 << 31;
    }
}

// =============================================================================
// 法术标志 - 精确对齐 C++ SpellFlag
// Reference: C++ Source/player.h
// =============================================================================

bitflags::bitflags! {
    /// 法术状态标志
    /// Reference: C++ Source/player.h SpellFlag
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct SpellFlag: u8 {
        /// 无标志
        const NONE = 0;
        /// 以太化
        const ETHEREALIZE = 1 << 0;
        /// 狂怒激活
        const RAGE_ACTIVE = 1 << 1;
        /// 狂怒冷却
        const RAGE_COOLDOWN = 1 << 2;
    }
}

// =============================================================================
// 物品创建信息标志 - 精确对齐 C++ icreateinfo_flag
// Reference: C++ Source/items.h
// =============================================================================

bitflags::bitflags! {
    /// 物品创建信息标志
    /// Reference: C++ Source/items.h icreateinfo_flag
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ItemCreateInfo: u16 {
        /// 等级掩码 (0-63)
        const LEVEL_MASK = (1 << 6) - 1;
        /// 只有好属性
        const ONLY_GOOD = 1 << 6;
        /// 15%唯一物品几率 (来自唯一怪物)
        const UPER_15 = 1 << 7;
        /// 1%唯一物品几率 (来自地牢)
        const UPER_1 = 1 << 8;
        /// 是唯一物品
        const UNIQUE = 1 << 9;
        /// 来自Griswold (基本)
        const SMITH = 1 << 10;
        /// 来自Griswold (高级)
        const SMITH_PREMIUM = 1 << 11;
        /// 来自Wirt
        const BOY = 1 << 12;
        /// 来自Adria
        const WITCH = 1 << 13;
        /// 来自Pepin
        const HEALER = 1 << 14;
        /// 预生成物品
        const PREGEN = 1 << 15;
    }
}

impl ItemCreateInfo {
    /// 获取物品等级 (0-63)
    pub fn level(&self) -> u8 {
        (self.bits() & Self::LEVEL_MASK.bits()) as u8
    }

    /// 是否来自城镇NPC
    pub fn is_from_town(&self) -> bool {
        self.intersects(Self::SMITH | Self::SMITH_PREMIUM | Self::BOY | Self::WITCH | Self::HEALER)
    }

    /// 是否有用 (药水/卷轴)
    pub fn is_useful(&self) -> bool {
        self.intersects(Self::UPER_15 | Self::UPER_1)
    }
}

// =============================================================================
// 玩家动作ID - 精确对齐 C++ action_id
// Reference: C++ Source/player.h
// =============================================================================

/// 玩家动作ID
/// Reference: C++ Source/player.h action_id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum ActionId {
    /// 自动行走 (手柄)
    Walk = -2,
    /// 无动作
    #[default]
    None = -1,
    /// 攻击
    Attack = 9,
    /// 远程攻击
    RangedAttack = 10,
    /// 施法
    Spell = 12,
    /// 操作物体
    Operate = 13,
    /// 解除陷阱
    Disarm = 14,
    /// 拾取物品 (放到手中)
    PickupItem = 15,
    /// 自动拾取物品 (放到背包)
    PickupAutoItem = 16,
    /// 对话
    Talk = 17,
    /// 远程操作 (心灵遥控)
    OperateTelekinesis = 18,
    /// 攻击怪物
    AttackMonster = 20,
    /// 攻击玩家
    AttackPlayer = 21,
    /// 远程攻击怪物
    RangedAttackMonster = 22,
    /// 远程攻击玩家
    RangedAttackPlayer = 23,
    /// 对怪物施法
    SpellMonster = 24,
    /// 对玩家施法
    SpellPlayer = 25,
    /// 对墙施法
    SpellWall = 26,
}

// =============================================================================
// 玩家模式 - 精确对齐 C++ PLR_MODE
// Reference: C++ Source/player.h
// =============================================================================

/// 玩家模式
/// Reference: C++ Source/player.h PLR_MODE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PlayerMode {
    /// 站立
    #[default]
    Stand = 0,
    /// 向北行走
    WalkNorthwards = 1,
    /// 向南行走
    WalkSouthwards = 2,
    /// 横向行走
    WalkSideways = 3,
    /// 近战攻击
    Attack = 4,
    /// 远程攻击
    RangedAttack = 5,
    /// 格挡
    Block = 6,
    /// 受击
    GotHit = 7,
    /// 死亡
    Death = 8,
    /// 施法
    Spell = 9,
    /// 新关卡
    NewLevel = 10,
    /// 退出
    Quit = 11,
}

impl PlayerMode {
    /// 是否是行走模式
    pub fn is_walking(&self) -> bool {
        matches!(self,
            PlayerMode::WalkNorthwards |
            PlayerMode::WalkSouthwards |
            PlayerMode::WalkSideways
        )
    }

    /// 是否可以改变动作
    pub fn can_change_action(&self) -> bool {
        !matches!(self,
            PlayerMode::Death |
            PlayerMode::NewLevel |
            PlayerMode::Quit
        )
    }
}

// =============================================================================
// 怪物模式 - 精确对齐 C++ MonsterMode
// Reference: C++ Source/monster.h
// =============================================================================

/// 怪物模式
/// Reference: C++ Source/monster.h MonsterMode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MonsterMode {
    /// 站立
    #[default]
    Stand = 0,
    /// 移动中 (新瓦片)
    MoveNorthwards = 1,
    /// 移动中 (旧瓦片)
    MoveSouthwards = 2,
    /// 横向移动
    MoveSideways = 3,
    /// 近战攻击
    MeleeAttack = 4,
    /// 受击
    GotHit = 5,
    /// 死亡
    Death = 6,
    /// 特殊站立 (等待)
    SpecialStand = 7,
    /// 法术攻击延迟
    SpellDelay = 8,
    /// 特殊近战攻击
    SpecialMelee = 9,
    /// 特殊远程攻击
    SpecialRanged = 10,
    /// 远程攻击
    RangedAttack = 11,
    /// 石化
    Petrified = 12,
    /// 治疗中
    Healing = 13,
    /// 对话中
    Talking = 14,
    /// 冲锋
    Charge = 15,
}

impl MonsterMode {
    /// 是否是移动模式
    pub fn is_moving(&self) -> bool {
        matches!(self,
            MonsterMode::MoveNorthwards |
            MonsterMode::MoveSouthwards |
            MonsterMode::MoveSideways
        )
    }

    /// 是否是攻击模式
    pub fn is_attacking(&self) -> bool {
        matches!(self,
            MonsterMode::MeleeAttack |
            MonsterMode::SpecialMelee |
            MonsterMode::SpecialRanged |
            MonsterMode::RangedAttack
        )
    }
}

/// 怪物目标
/// Reference: C++ Source/monster.h MonsterGoal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MonsterGoal {
    /// 正常 (追逐/攻击)
    #[default]
    Normal = 0,
    /// 撤退
    Retreat = 1,
    /// 治疗
    Healing = 2,
    /// 移动到位置
    Move = 3,
    /// 攻击
    Attack = 4,
    /// 调查
    Inquiring = 5,
    /// 对话中
    Talking = 6,
}

// =============================================================================
// 唯一怪物类型 - 精确对齐 C++ UniqueMonsterType
// Reference: C++ Source/monstdat.h
// =============================================================================

/// 唯一怪物类型
/// Reference: C++ Source/monstdat.h UniqueMonsterType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum UniqueMonsterType {
    /// 非唯一怪物
    #[default]
    None = 0,
    /// Skeleton King
    SkeletonKing = 1,
    /// The Butcher
    Butcher = 2,
    /// Gharbad the Weak
    GharbadTheWeak = 3,
    /// Zhar the Mad
    ZharTheMad = 4,
    /// Snotspill
    Snotspill = 5,
    /// Arch-Bishop Lazarus
    Lazarus = 6,
    /// Lachdanan
    Lachdanan = 7,
    /// Warlord of Blood
    WarlordOfBlood = 8,
    /// Diablo
    Diablo = 9,
    /// The Defiler (Hellfire)
    Defiler = 10,
    /// Na-Krul (Hellfire)
    NaKrul = 11,
    /// Hork Demon (Hellfire)
    HorkDemon = 12,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_displacement() {
        let d = Displacement::new(3, 4);
        assert!((d.magnitude() - 5.0).abs() < 0.001);

        let d2 = Displacement::from_direction(Direction::South);
        assert_eq!(d2.delta_x, 1);
        assert_eq!(d2.delta_y, 1);
    }

    #[test]
    fn test_world_tile_position() {
        let p1 = WorldTilePosition::new(10, 10);
        let p2 = WorldTilePosition::new(15, 12);
        assert_eq!(p1.manhattan_distance(&p2), 7);
        assert_eq!(p1.walking_distance(&p2), 5);
    }

    #[test]
    fn test_actor_position() {
        let mut pos = ActorPosition::new(50, 50);
        assert_eq!(pos.tile.x, 50);

        pos.future = WorldTilePosition::new(51, 50);
        assert_eq!(pos.future.x, 51);
    }

    #[test]
    fn test_item_special_effect() {
        let flags = ItemSpecialEffect::FIRE_ARROWS | ItemSpecialEffect::KNOCKBACK;
        assert!(flags.contains(ItemSpecialEffect::FIRE_ARROWS));
        assert!(flags.contains(ItemSpecialEffect::KNOCKBACK));
        assert!(!flags.contains(ItemSpecialEffect::VAMPIRIC));
    }

    #[test]
    fn test_spell_flag() {
        let flags = SpellFlag::RAGE_ACTIVE | SpellFlag::RAGE_COOLDOWN;
        assert!(flags.contains(SpellFlag::RAGE_ACTIVE));
        assert!(!flags.contains(SpellFlag::ETHEREALIZE));
    }

    #[test]
    fn test_item_create_info() {
        let info = ItemCreateInfo::from_bits_truncate(0b10 | (1 << 10)); // level 2 + SMITH
        assert_eq!(info.level(), 2);
        assert!(info.is_from_town());
    }

    #[test]
    fn test_player_mode() {
        assert!(PlayerMode::WalkNorthwards.is_walking());
        assert!(!PlayerMode::Stand.is_walking());
        assert!(PlayerMode::Stand.can_change_action());
        assert!(!PlayerMode::Death.can_change_action());
    }

    #[test]
    fn test_monster_mode() {
        assert!(MonsterMode::MoveNorthwards.is_moving());
        assert!(MonsterMode::MeleeAttack.is_attacking());
        assert!(!MonsterMode::Stand.is_moving());
    }
}
