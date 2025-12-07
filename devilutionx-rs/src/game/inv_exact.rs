//! Inventory System - Exact port from C++ Source/inv.cpp
//!
//! 精确移植C++ inv.cpp 物品栏系统
//! 包含：物品栏布局、装备操作、物品放置、金币处理

use super::types::Point;
use super::player_exact::SpellId;

// ============================================================================
// 常量定义
// ============================================================================

/// 物品栏槽位总数
pub const NUM_XY_SLOTS: usize = 55;

/// 背包格子总数 (10x4)
pub const INVENTORY_GRID_CELLS: usize = 40;

/// 最大腰带物品数
pub const MAX_BELT_ITEMS: usize = 8;

/// 物品栏槽位像素尺寸
pub const INV_SLOT_SIZE_PX: i32 = 29;
pub const INV_SLOT_HALF_SIZE_PX: i32 = 14;

/// 最大金币堆叠数量
pub const MAX_GOLD: i32 = 5000;

// ============================================================================
// 物品栏槽位枚举
// ============================================================================

/// 物品栏XY槽位索引
/// Reference: C++ inv.h enum inv_xy_slot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InvXYSlot {
    Head = 0,
    RingLeft = 1,
    RingRight = 2,
    Amulet = 3,
    HandLeft = 4,
    HandRight = 5,
    Chest = 6,
    InvFirst = 7,
    InvLast = 46,
    BeltFirst = 47,
    BeltLast = 54,
}

impl InvXYSlot {
    pub const SLOTXY_HEAD: u8 = 0;
    pub const SLOTXY_RING_LEFT: u8 = 1;
    pub const SLOTXY_RING_RIGHT: u8 = 2;
    pub const SLOTXY_AMULET: u8 = 3;
    pub const SLOTXY_HAND_LEFT: u8 = 4;
    pub const SLOTXY_HAND_RIGHT: u8 = 5;
    pub const SLOTXY_CHEST: u8 = 6;
    pub const SLOTXY_INV_FIRST: u8 = 7;
    pub const SLOTXY_INV_LAST: u8 = 46;
    pub const SLOTXY_BELT_FIRST: u8 = 47;
    pub const SLOTXY_BELT_LAST: u8 = 54;
    pub const SLOTXY_EQUIPPED_FIRST: u8 = 0;
    pub const SLOTXY_EQUIPPED_LAST: u8 = 6;
}

/// 身体装备位置
/// Reference: C++ inv.h enum inv_body_loc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum InvBodyLoc {
    #[default]
    Head = 0,
    RingLeft = 1,
    RingRight = 2,
    Amulet = 3,
    HandLeft = 4,
    HandRight = 5,
    Chest = 6,
}

impl InvBodyLoc {
    pub const NUM_INVLOC: usize = 7;

    pub const INVLOC_HEAD: u8 = 0;
    pub const INVLOC_RING_LEFT: u8 = 1;
    pub const INVLOC_RING_RIGHT: u8 = 2;
    pub const INVLOC_AMULET: u8 = 3;
    pub const INVLOC_HAND_LEFT: u8 = 4;
    pub const INVLOC_HAND_RIGHT: u8 = 5;
    pub const INVLOC_CHEST: u8 = 6;
}

/// 物品装备位置类型
/// Reference: C++ items.h item_equip_type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ItemEquipType {
    #[default]
    None = 0,
    OneHand = 1,
    TwoHand = 2,
    Armor = 3,
    Helm = 4,
    Ring = 5,
    Amulet = 6,
    Unequipable = 7,
    Belt = 8,
    Invalid = 255,
}

/// 物品索引常量
/// Reference: C++ inv.h INVITEM_*
pub const INVITEM_INV_FIRST: i8 = 7;
pub const INVITEM_INV_LAST: i8 = 46;
pub const INVITEM_BELT_FIRST: i8 = 47;
pub const INVITEM_BELT_LAST: i8 = 54;

// ============================================================================
// 矩形结构 (物品栏槽位位置)
// ============================================================================

/// 矩形区域
#[derive(Debug, Clone, Copy, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            position: Point { x, y },
            size: Size { width: w, height: h },
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.position.x
            && p.x < self.position.x + self.size.width
            && p.y >= self.position.y
            && p.y < self.position.y + self.size.height
    }
}

/// 尺寸
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

// ============================================================================
// 物品栏槽位位置表
// ============================================================================

/// 物品栏槽位到屏幕位置的映射表
/// Reference: C++ inv.cpp InvRect[55]
///
/// 槽位布局:
/// ```text
///                          00 00
///                          00 00   03
///
///              04 04       06 06       05 05
///              04 04       06 06       05 05
///              04 04       06 06       05 05
///
///                 01                   02
///
///              07 08 09 10 11 12 13 14 15 16
///              17 18 19 20 21 22 23 24 25 26
///              27 28 29 30 31 32 33 34 35 36
///              37 38 39 40 41 42 43 44 45 46
///
/// 47 48 49 50 51 52 53 54
/// ```
pub const INV_RECT: [Rectangle; 55] = [
    // 头盔 (0)
    Rectangle::new(132, 2, 58, 59),
    // 左戒指 (1)
    Rectangle::new(47, 177, 28, 29),
    // 右戒指 (2)
    Rectangle::new(248, 177, 28, 29),
    // 护身符 (3)
    Rectangle::new(205, 32, 28, 29),
    // 左手 (4)
    Rectangle::new(17, 75, 58, 86),
    // 右手 (5)
    Rectangle::new(248, 75, 58, 87),
    // 胸甲 (6)
    Rectangle::new(132, 75, 58, 87),
    // 背包第1行 (7-16)
    Rectangle::new(17, 222, 29, 29),
    Rectangle::new(46, 222, 29, 29),
    Rectangle::new(75, 222, 29, 29),
    Rectangle::new(104, 222, 29, 29),
    Rectangle::new(133, 222, 29, 29),
    Rectangle::new(162, 222, 29, 29),
    Rectangle::new(191, 222, 29, 29),
    Rectangle::new(220, 222, 29, 29),
    Rectangle::new(249, 222, 29, 29),
    Rectangle::new(278, 222, 29, 29),
    // 背包第2行 (17-26)
    Rectangle::new(17, 251, 29, 29),
    Rectangle::new(46, 251, 29, 29),
    Rectangle::new(75, 251, 29, 29),
    Rectangle::new(104, 251, 29, 29),
    Rectangle::new(133, 251, 29, 29),
    Rectangle::new(162, 251, 29, 29),
    Rectangle::new(191, 251, 29, 29),
    Rectangle::new(220, 251, 29, 29),
    Rectangle::new(249, 251, 29, 29),
    Rectangle::new(278, 251, 29, 29),
    // 背包第3行 (27-36)
    Rectangle::new(17, 280, 29, 29),
    Rectangle::new(46, 280, 29, 29),
    Rectangle::new(75, 280, 29, 29),
    Rectangle::new(104, 280, 29, 29),
    Rectangle::new(133, 280, 29, 29),
    Rectangle::new(162, 280, 29, 29),
    Rectangle::new(191, 280, 29, 29),
    Rectangle::new(220, 280, 29, 29),
    Rectangle::new(249, 280, 29, 29),
    Rectangle::new(278, 280, 29, 29),
    // 背包第4行 (37-46)
    Rectangle::new(17, 309, 29, 29),
    Rectangle::new(46, 309, 29, 29),
    Rectangle::new(75, 309, 29, 29),
    Rectangle::new(104, 309, 29, 29),
    Rectangle::new(133, 309, 29, 29),
    Rectangle::new(162, 309, 29, 29),
    Rectangle::new(191, 309, 29, 29),
    Rectangle::new(220, 309, 29, 29),
    Rectangle::new(249, 309, 29, 29),
    Rectangle::new(278, 309, 29, 29),
    // 腰带 (47-54)
    Rectangle::new(205, 5, 29, 29),
    Rectangle::new(234, 5, 29, 29),
    Rectangle::new(263, 5, 29, 29),
    Rectangle::new(292, 5, 29, 29),
    Rectangle::new(321, 5, 29, 29),
    Rectangle::new(350, 5, 29, 29),
    Rectangle::new(379, 5, 29, 29),
    Rectangle::new(408, 5, 29, 29),
];

// ============================================================================
// 物品结构体 (简化版)
// ============================================================================

/// 简化物品结构用于物品栏操作
#[derive(Debug, Clone, Default)]
pub struct InvItem {
    pub _i_type: ItemType,
    pub _i_curs: i32,
    pub _i_value: i32,
    pub _i_max_dur: i32,
    pub _i_dur: i32,
    pub _i_charges: i32,
    pub _i_max_charges: i32,
    pub _i_loc: ItemEquipType,
    pub _i_class: ItemClass,
    pub _i_stat_flag: bool,
    pub _i_identified: bool,
    pub _i_spell: SpellId,
    pub _i_misc_id: i32,
    // 更多字段...
}

impl InvItem {
    pub fn is_empty(&self) -> bool {
        self._i_type == ItemType::None
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self._i_loc,
            ItemEquipType::OneHand | ItemEquipType::TwoHand |
            ItemEquipType::Armor | ItemEquipType::Helm |
            ItemEquipType::Ring | ItemEquipType::Amulet)
    }

    pub fn is_usable(&self) -> bool {
        // 简化实现
        !self.is_empty()
    }

    pub fn is_scroll_of(&self, spell: SpellId) -> bool {
        self._i_misc_id == 21 && self._i_spell == spell // IMISC_SCROLL
    }

    pub fn is_rune_of(&self, spell: SpellId) -> bool {
        self._i_misc_id >= 37 && self._i_misc_id <= 43 && self._i_spell == spell
    }
}

/// 物品类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ItemType {
    #[default]
    None = 0,
    Sword = 1,
    Axe = 2,
    Bow = 3,
    Mace = 4,
    Shield = 5,
    LightArmor = 6,
    Helm = 7,
    MediumArmor = 8,
    HeavyArmor = 9,
    Staff = 10,
    Gold = 11,
    Ring = 12,
    Amulet = 13,
    Misc = 14,
}

/// 物品类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ItemClass {
    #[default]
    None = 0,
    Weapon = 1,
    Armor = 2,
    Misc = 3,
    Gold = 4,
}

// ============================================================================
// 物品栏结构体
// ============================================================================

/// 物品栏状态
#[derive(Debug, Clone)]
pub struct InventoryState {
    /// 物品栏是否打开
    pub inv_flag: bool,
    /// 当前高亮的物品槽位
    pub pcurs_inv_item: i8,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            inv_flag: false,
            pcurs_inv_item: -1,
        }
    }
}

// ============================================================================
// 玩家结构 (物品栏专用)
// ============================================================================

/// 物品栏操作用的玩家结构体
/// 包含物品栏操作所需的最小字段集
#[derive(Debug, Clone)]
pub struct Player {
    /// 装备物品 (7槽位)
    pub inv_body: [InvItem; InvBodyLoc::NUM_INVLOC],
    /// 背包物品 (40槽位)
    pub inv_list: [InvItem; INVENTORY_GRID_CELLS],
    /// 背包网格 (正数=物品ID，负数=被占用，0=空)
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],
    /// 腰带物品 (8槽位)
    pub spd_list: [InvItem; MAX_BELT_ITEMS],
    /// 手持物品
    pub hold_item: InvItem,
    /// 背包物品数量
    pub _p_num_inv: i8,
    /// 金币总数
    pub _p_gold: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            inv_body: [
                InvItem::default(), InvItem::default(), InvItem::default(),
                InvItem::default(), InvItem::default(), InvItem::default(),
                InvItem::default(),
            ],
            inv_list: core::array::from_fn(|_| InvItem::default()),
            inv_grid: [0; INVENTORY_GRID_CELLS],
            spd_list: [
                InvItem::default(), InvItem::default(), InvItem::default(),
                InvItem::default(), InvItem::default(), InvItem::default(),
                InvItem::default(), InvItem::default(),
            ],
            hold_item: InvItem::default(),
            _p_num_inv: 0,
            _p_gold: 0,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 核心函数
// ============================================================================

/// 添加物品到背包网格
/// Reference: C++ inv.cpp AddItemToInvGrid()
pub fn add_item_to_inv_grid(
    player: &mut Player,
    inv_grid_index: usize,
    inv_list_index: i8,
    item_size: Size,
    _send_network_message: bool,
) {
    const PITCH: usize = 10;

    for y in 0..item_size.height as usize {
        let row_grid_index = inv_grid_index + PITCH * y;
        for x in 0..item_size.width as usize {
            let grid_index = row_grid_index + x;
            if grid_index < player.inv_grid.len() {
                if x == 0 && y == item_size.height as usize - 1 {
                    // 物品左下角存储正索引
                    player.inv_grid[grid_index] = inv_list_index;
                } else {
                    // 其他格子存储负索引表示被占用
                    player.inv_grid[grid_index] = -inv_list_index;
                }
            }
        }
    }

    // TODO: if send_network_message { NetSendCmdChInvItem(false, inv_grid_index); }
}

/// 检查物品是否适合腰带槽位 (1x1)
/// Reference: C++ inv.cpp FitsInBeltSlot()
pub fn fits_in_belt_slot(item: &InvItem) -> bool {
    let size = get_inventory_size(item);
    size == Size::new(1, 1)
}

/// 检查物品是否可以放置在腰带上
/// Reference: C++ inv.cpp CanBePlacedOnBelt()
pub fn can_be_placed_on_belt(player: &Player, item: &InvItem) -> bool {
    fits_in_belt_slot(item)
        && item._i_type != ItemType::Gold
        && can_use_item(player, item)
        && item.is_usable()
}

/// 检查玩家是否可以使用物品
/// Reference: C++ player.h Player::CanUseItem()
pub fn can_use_item(_player: &Player, item: &InvItem) -> bool {
    item._i_stat_flag
}

/// 检查物品是否可以装备
/// Reference: C++ inv.cpp CanEquip()
pub fn can_equip(item: &InvItem) -> bool {
    item.is_equipment() && item._i_stat_flag
}

/// 检查武器是否可以持握
/// Reference: C++ inv.cpp CanWield()
pub fn can_wield(player: &Player, item: &InvItem) -> bool {
    if !can_equip(item) {
        return false;
    }

    let item_loc = item._i_loc;
    if item_loc != ItemEquipType::OneHand && item_loc != ItemEquipType::TwoHand {
        return false;
    }

    let left_hand_empty = player.inv_body[InvBodyLoc::HandLeft as usize].is_empty();
    let right_hand_empty = player.inv_body[InvBodyLoc::HandRight as usize].is_empty();

    // 双手都空
    if left_hand_empty && right_hand_empty {
        return true;
    }

    // 双手都不空
    if !left_hand_empty && !right_hand_empty {
        return false;
    }

    // 一只手空，一只手不空
    // 单手武器可以装备到空闲的手
    if item_loc == ItemEquipType::OneHand {
        let occupied_hand = if !left_hand_empty {
            &player.inv_body[InvBodyLoc::HandLeft as usize]
        } else {
            &player.inv_body[InvBodyLoc::HandRight as usize]
        };

        // 检查已装备物品是否是单手武器
        if occupied_hand._i_loc == ItemEquipType::OneHand {
            // 不能装备同类物品
            return item._i_class != occupied_hand._i_class;
        }
    }

    false
}

/// 检查是否可以在指定位置装备物品
/// Reference: C++ inv.cpp CanEquip(player, item, bodyLocation)
pub fn can_equip_at_location(
    player: &Player,
    item: &InvItem,
    body_location: InvBodyLoc,
) -> bool {
    if !can_equip(item) {
        return false;
    }

    // 检查目标位置是否为空
    if !player.inv_body[body_location as usize].is_empty() {
        return false;
    }

    match body_location {
        InvBodyLoc::Amulet => item._i_loc == ItemEquipType::Amulet,
        InvBodyLoc::Chest => item._i_loc == ItemEquipType::Armor,
        InvBodyLoc::HandLeft | InvBodyLoc::HandRight => can_wield(player, item),
        InvBodyLoc::Head => item._i_loc == ItemEquipType::Helm,
        InvBodyLoc::RingLeft | InvBodyLoc::RingRight => item._i_loc == ItemEquipType::Ring,
    }
}

/// 自动装备物品到身体
/// Reference: C++ inv.cpp AutoEquip()
pub fn auto_equip(
    player: &mut Player,
    item: &InvItem,
    persist_item: bool,
    _send_network_message: bool,
) -> bool {
    if !can_equip(item) {
        return false;
    }

    for body_loc_idx in 0..InvBodyLoc::NUM_INVLOC {
        let body_loc = match body_loc_idx {
            0 => InvBodyLoc::Head,
            1 => InvBodyLoc::RingLeft,
            2 => InvBodyLoc::RingRight,
            3 => InvBodyLoc::Amulet,
            4 => InvBodyLoc::HandLeft,
            5 => InvBodyLoc::HandRight,
            6 => InvBodyLoc::Chest,
            _ => continue,
        };

        if can_equip_at_location(player, item, body_loc) {
            if persist_item {
                player.inv_body[body_loc as usize] = item.clone();
                // TODO: 播放音效，计算属性
            }
            return true;
        }
    }

    false
}

/// 自动放置物品到腰带
/// Reference: C++ inv.cpp AutoPlaceItemInBelt()
pub fn auto_place_item_in_belt(
    player: &mut Player,
    item: &InvItem,
    persist_item: bool,
    _send_network_message: bool,
) -> bool {
    if !can_be_placed_on_belt(player, item) {
        return false;
    }

    for i in 0..MAX_BELT_ITEMS {
        if player.spd_list[i].is_empty() {
            if persist_item {
                player.spd_list[i] = item.clone();
                // TODO: player.calc_scrolls();
                // TODO: if send_network_message { NetSendCmdChBeltItem(false, i); }
            }
            return true;
        }
    }

    false
}

/// 检查物品是否能放入背包
/// Reference: C++ inv.cpp CanFitItemInInventory()
pub fn can_fit_item_in_inventory(player: &Player, item: &InvItem) -> bool {
    find_slot_for_item(player, get_inventory_size(item)).is_some()
}

/// 自动放置物品到背包
/// Reference: C++ inv.cpp AutoPlaceItemInInventory()
pub fn auto_place_item_in_inventory(
    player: &mut Player,
    item: &InvItem,
    _send_network_message: bool,
) -> bool {
    let item_size = get_inventory_size(item);

    if let Some(target_slot) = find_slot_for_item(player, item_size) {
        player.inv_list[player._p_num_inv as usize] = item.clone();
        player._p_num_inv += 1;

        add_item_to_inv_grid(
            player,
            target_slot,
            player._p_num_inv,
            item_size,
            _send_network_message,
        );

        // TODO: player.calc_scrolls();

        return true;
    }

    false
}

/// 查找可以放置指定尺寸物品的槽位
/// Reference: C++ inv.cpp FindSlotForItem()
pub fn find_slot_for_item(player: &Player, item_size: Size) -> Option<usize> {
    find_slot_for_item_ignore(player, item_size, None)
}

/// 查找可以放置指定尺寸物品的槽位 (可忽略指定物品)
fn find_slot_for_item_ignore(
    player: &Player,
    item_size: Size,
    item_index_to_ignore: Option<usize>,
) -> Option<usize> {
    let ignore_idx = item_index_to_ignore.map(|i| i as i32).unwrap_or(-1);

    if item_size.height == 1 {
        // 优先放在最后一行
        for i in 30..=39 {
            if check_item_fits_in_inventory_slot(player, i, item_size, ignore_idx) {
                return Some(i);
            }
        }
        // 然后从右到左、从下到上
        for x in (0..=9).rev() {
            for y in (0..=2).rev() {
                let slot = 10 * y + x;
                if check_item_fits_in_inventory_slot(player, slot, item_size, ignore_idx) {
                    return Some(slot);
                }
            }
        }
        return None;
    }

    if item_size.height == 2 {
        for x in (0..=(10 - item_size.width)).rev() {
            for y in 0..3 {
                let slot = (10 * y + x) as usize;
                if check_item_fits_in_inventory_slot(player, slot, item_size, ignore_idx) {
                    return Some(slot);
                }
            }
        }
        return None;
    }

    if item_size == Size::new(1, 3) {
        for i in 0..20 {
            if check_item_fits_in_inventory_slot(player, i, item_size, ignore_idx) {
                return Some(i);
            }
        }
        return None;
    }

    if item_size == Size::new(2, 3) {
        for i in 0..9 {
            if check_item_fits_in_inventory_slot(player, i, item_size, ignore_idx) {
                return Some(i);
            }
        }
        for i in 10..19 {
            if check_item_fits_in_inventory_slot(player, i, item_size, ignore_idx) {
                return Some(i);
            }
        }
        return None;
    }

    None
}

/// 检查指定尺寸物品是否能放入指定槽位
/// Reference: C++ inv.cpp CheckItemFitsInInventorySlot()
fn check_item_fits_in_inventory_slot(
    player: &Player,
    slot_index: usize,
    item_size: Size,
    item_index_to_ignore: i32,
) -> bool {
    let mut yy = if slot_index > 0 { (slot_index / 10) * 10 } else { 0 };

    for _j in 0..item_size.height {
        if yy >= INVENTORY_GRID_CELLS {
            return false;
        }

        let mut xx = if slot_index > 0 { slot_index % 10 } else { 0 };

        for _i in 0..item_size.width {
            let grid_idx = xx + yy;
            if xx >= 10 || grid_idx >= player.inv_grid.len() {
                return false;
            }

            let grid_val = player.inv_grid[grid_idx];
            let occupied_idx = grid_val.abs() as i32 - 1;

            if grid_val != 0 && occupied_idx != item_index_to_ignore {
                return false;
            }

            xx += 1;
        }
        yy += 10;
    }

    true
}

/// 获取物品在背包中的尺寸
/// Reference: C++ inv.cpp GetInventorySize()
pub fn get_inventory_size(item: &InvItem) -> Size {
    // 简化实现：基于物品类型返回常见尺寸
    // 实际实现需要查询物品光标图像尺寸
    match item._i_type {
        ItemType::None => Size::new(1, 1),
        ItemType::Gold => Size::new(1, 1),
        ItemType::Ring => Size::new(1, 1),
        ItemType::Amulet => Size::new(1, 1),
        ItemType::Misc => Size::new(1, 1),
        ItemType::Sword | ItemType::Staff => Size::new(1, 3),
        ItemType::Axe | ItemType::Mace | ItemType::Bow => Size::new(2, 3),
        ItemType::Shield => Size::new(2, 2),
        ItemType::Helm => Size::new(2, 2),
        ItemType::LightArmor => Size::new(2, 2),
        ItemType::MediumArmor | ItemType::HeavyArmor => Size::new(2, 3),
    }
}

/// 计算玩家金币总数
/// Reference: C++ inv.cpp CalculateGold()
pub fn calculate_gold(player: &Player) -> i32 {
    let mut gold = 0;

    for i in 0..player._p_num_inv as usize {
        if player.inv_list[i]._i_type == ItemType::Gold {
            gold += player.inv_list[i]._i_value;
        }
    }

    gold
}

/// 背包中可以容纳的金币数量
/// Reference: C++ inv.cpp RoomForGold()
pub fn room_for_gold(player: &Player) -> i32 {
    let mut amount = 0;

    for &item_index in &player.inv_grid {
        if item_index < 0 {
            continue;
        }

        if item_index == 0 {
            amount += MAX_GOLD;
            continue;
        }

        let gold_item = &player.inv_list[(item_index - 1) as usize];
        if gold_item._i_type != ItemType::Gold || gold_item._i_value == MAX_GOLD {
            continue;
        }

        amount += MAX_GOLD - gold_item._i_value;
    }

    amount
}

/// 添加金币到背包
/// Reference: C++ inv.cpp AddGoldToInventory()
pub fn add_gold_to_inventory(player: &mut Player, mut value: i32) -> i32 {
    // 先填满现有金币堆
    for i in 0..player._p_num_inv as usize {
        if value <= 0 {
            break;
        }

        let gold_item = &mut player.inv_list[i];
        if gold_item._i_type != ItemType::Gold || gold_item._i_value >= MAX_GOLD {
            continue;
        }

        let can_add = MAX_GOLD - gold_item._i_value;
        if value >= can_add {
            gold_item._i_value = MAX_GOLD;
            value -= can_add;
        } else {
            gold_item._i_value += value;
            value = 0;
        }

        // TODO: 更新金币光标
    }

    // 创建新金币堆 - 最后一行从右到左
    for i in (30..=39).rev() {
        if value <= 0 {
            break;
        }
        value = create_gold_item_in_inventory_slot(player, i, value);
    }

    // 其余位置
    for x in (0..=9).rev() {
        for y in (0..=2).rev() {
            if value <= 0 {
                break;
            }
            let slot = 10 * y + x;
            value = create_gold_item_in_inventory_slot(player, slot, value);
        }
    }

    value
}

/// 在指定槽位创建金币物品
/// Reference: C++ inv.cpp CreateGoldItemInInventorySlot()
fn create_gold_item_in_inventory_slot(player: &mut Player, slot_index: usize, value: i32) -> i32 {
    if slot_index >= player.inv_grid.len() || player.inv_grid[slot_index] != 0 {
        return value;
    }

    let gold_value = value.min(MAX_GOLD);

    let mut gold_item = InvItem::default();
    gold_item._i_type = ItemType::Gold;
    gold_item._i_value = gold_value;
    // TODO: SetPlrHandGoldCurs()

    player.inv_list[player._p_num_inv as usize] = gold_item;
    player._p_num_inv += 1;
    player.inv_grid[slot_index] = player._p_num_inv;

    // TODO: if MyPlayer { NetSendCmdChInvItem(false, slot_index); }

    value - gold_value
}

/// 金币自动放置
/// Reference: C++ inv.cpp GoldAutoPlace()
pub fn gold_auto_place(player: &mut Player, gold_stack: &mut InvItem) -> bool {
    gold_stack._i_value = add_gold_to_inventory(player, gold_stack._i_value);
    // TODO: SetPlrHandGoldCurs(gold_stack);

    player._p_gold = calculate_gold(player);

    gold_stack._i_value == 0
}

/// 移除装备
/// Reference: C++ inv.cpp RemoveEquipment()
pub fn remove_equipment(
    player: &mut Player,
    body_location: InvBodyLoc,
    _hi_pri: bool,
) {
    // TODO: if MyPlayer { NetSendCmdDelItem(hi_pri, body_location); }
    player.inv_body[body_location as usize].clear();
}

/// 检查是否可以使用卷轴
/// Reference: C++ inv.cpp CanUseScroll()
pub fn can_use_scroll(player: &Player, spell: SpellId) -> bool {
    // TODO: 检查城镇法术限制

    has_inventory_or_belt_item(player, |item| {
        item.is_scroll_of(spell) || item.is_rune_of(spell)
    })
}

/// 检查玩家背包或腰带是否有满足条件的物品
fn has_inventory_or_belt_item<F>(player: &Player, predicate: F) -> bool
where
    F: Fn(&InvItem) -> bool,
{
    for i in 0..player._p_num_inv as usize {
        if predicate(&player.inv_list[i]) {
            return true;
        }
    }

    for i in 0..MAX_BELT_ITEMS {
        if predicate(&player.spd_list[i]) {
            return true;
        }
    }

    false
}

/// 消耗卷轴
/// Reference: C++ inv.cpp ConsumeScroll()
pub fn consume_scroll(player: &mut Player, spell_id: SpellId, item_slot: i8) {
    // 尝试从指定位置移除
    if item_slot >= INVITEM_INV_FIRST && item_slot <= INVITEM_INV_LAST {
        let item_index = (item_slot - INVITEM_INV_FIRST) as usize;
        let item = &player.inv_list[item_index];
        if !item.is_empty() && (item.is_scroll_of(spell_id) || item.is_rune_of(spell_id)) {
            remove_inv_item(player, item_index);
            return;
        }
    } else if item_slot >= INVITEM_BELT_FIRST && item_slot <= INVITEM_BELT_LAST {
        let item_index = (item_slot - INVITEM_BELT_FIRST) as usize;
        let item = &player.spd_list[item_index];
        if !item.is_empty() && (item.is_scroll_of(spell_id) || item.is_rune_of(spell_id)) {
            remove_spd_bar_item(player, item_index);
            return;
        }
    }

    // 没找到，移除第一个匹配的卷轴
    remove_inventory_or_belt_item(player, |item| {
        item.is_scroll_of(spell_id) || item.is_rune_of(spell_id)
    });
}

/// 从背包移除物品
/// Reference: C++ player.cpp Player::RemoveInvItem()
pub fn remove_inv_item(player: &mut Player, item_index: usize) {
    // 清除网格引用
    let target_idx = (item_index + 1) as i8;
    for grid_val in &mut player.inv_grid {
        if *grid_val == target_idx || *grid_val == -target_idx {
            *grid_val = 0;
        }
    }

    // 移动最后一个物品填补空位
    player._p_num_inv -= 1;
    if item_index < player._p_num_inv as usize {
        let last_idx = player._p_num_inv as usize;
        player.inv_list[item_index] = player.inv_list[last_idx].clone();

        // 更新网格引用
        let old_idx = (last_idx + 1) as i8;
        let new_idx = (item_index + 1) as i8;
        for grid_val in &mut player.inv_grid {
            if *grid_val == old_idx {
                *grid_val = new_idx;
            } else if *grid_val == -old_idx {
                *grid_val = -new_idx;
            }
        }
    }

    player.inv_list[player._p_num_inv as usize].clear();
}

/// 从腰带移除物品
/// Reference: C++ player.cpp Player::RemoveSpdBarItem()
pub fn remove_spd_bar_item(player: &mut Player, item_index: usize) {
    player.spd_list[item_index].clear();
    // TODO: CalcScrolls()
    // TODO: force_redraw = 255
}

/// 移除背包或腰带中满足条件的第一个物品
fn remove_inventory_or_belt_item<F>(player: &mut Player, predicate: F)
where
    F: Fn(&InvItem) -> bool,
{
    for i in 0..player._p_num_inv as usize {
        if predicate(&player.inv_list[i]) {
            remove_inv_item(player, i);
            return;
        }
    }

    for i in 0..MAX_BELT_ITEMS {
        if predicate(&player.spd_list[i]) {
            remove_spd_bar_item(player, i);
            return;
        }
    }
}

/// 获取物品栏中的物品
/// Reference: C++ inv.cpp GetInventoryItem()
pub fn get_inventory_item(player: &Player, location: i8) -> &InvItem {
    if location < INVITEM_INV_FIRST {
        &player.inv_body[location as usize]
    } else if location <= INVITEM_INV_LAST {
        &player.inv_list[(location - INVITEM_INV_FIRST) as usize]
    } else {
        &player.spd_list[(location - INVITEM_BELT_FIRST) as usize]
    }
}

/// 关闭物品栏
/// Reference: C++ inv.cpp CloseInventory()
pub fn close_inventory(state: &mut InventoryState) {
    // TODO: CloseGoldWithdraw()
    // TODO: CloseStash()
    state.inv_flag = false;
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_player() -> Player {
        Player::new()
    }

    fn create_gold_item(value: i32) -> InvItem {
        let mut item = InvItem::default();
        item._i_type = ItemType::Gold;
        item._i_value = value;
        item
    }

    fn create_potion_item() -> InvItem {
        let mut item = InvItem::default();
        item._i_type = ItemType::Misc;
        item._i_stat_flag = true;
        item
    }

    #[test]
    fn test_inv_rect_count() {
        assert_eq!(INV_RECT.len(), 55);
    }

    #[test]
    fn test_inv_rect_head() {
        let head = &INV_RECT[InvXYSlot::SLOTXY_HEAD as usize];
        assert_eq!(head.position.x, 132);
        assert_eq!(head.position.y, 2);
    }

    #[test]
    fn test_fits_in_belt_slot() {
        let gold = create_gold_item(100);
        assert!(fits_in_belt_slot(&gold));

        let mut sword = InvItem::default();
        sword._i_type = ItemType::Sword;
        assert!(!fits_in_belt_slot(&sword));
    }

    #[test]
    fn test_calculate_gold() {
        let mut player = create_test_player();

        player.inv_list[0] = create_gold_item(100);
        player.inv_list[1] = create_gold_item(200);
        player._p_num_inv = 2;

        assert_eq!(calculate_gold(&player), 300);
    }

    #[test]
    fn test_get_inventory_size() {
        let gold = create_gold_item(100);
        assert_eq!(get_inventory_size(&gold), Size::new(1, 1));

        let mut sword = InvItem::default();
        sword._i_type = ItemType::Sword;
        assert_eq!(get_inventory_size(&sword), Size::new(1, 3));
    }

    #[test]
    fn test_auto_place_item_in_belt() {
        let mut player = create_test_player();
        let potion = create_potion_item();

        assert!(auto_place_item_in_belt(&mut player, &potion, true, false));
        assert!(!player.spd_list[0].is_empty());
    }

    #[test]
    fn test_auto_place_item_in_inventory() {
        let mut player = create_test_player();
        let potion = create_potion_item();

        assert!(auto_place_item_in_inventory(&mut player, &potion, false));
    }

    #[test]
    fn test_room_for_gold_empty() {
        let player = create_test_player();
        // 40个空槽 * MAX_GOLD
        assert_eq!(room_for_gold(&player), 40 * MAX_GOLD);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(10, 20, 30, 40);

        assert!(rect.contains(Point::new(10, 20)));
        assert!(rect.contains(Point::new(39, 59)));
        assert!(!rect.contains(Point::new(9, 20)));
        assert!(!rect.contains(Point::new(40, 20)));
    }

    #[test]
    fn test_item_is_empty() {
        let item = InvItem::default();
        assert!(item.is_empty());

        let gold = create_gold_item(100);
        assert!(!gold.is_empty());
    }

    #[test]
    fn test_inv_body_loc_constants() {
        assert_eq!(InvBodyLoc::INVLOC_HEAD, 0);
        assert_eq!(InvBodyLoc::INVLOC_CHEST, 6);
        assert_eq!(InvBodyLoc::NUM_INVLOC, 7);
    }
}
