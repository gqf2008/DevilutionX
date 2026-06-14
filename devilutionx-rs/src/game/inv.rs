//! Player Inventory System
//!
//! C++ Reference: Source/inv.cpp, Source/inv.h
//!
//! Interface of player inventory management, including:
//! - Equipment slots (head, chest, hands, rings, amulet)
//! - Inventory grid (10x4 slots)
//! - Belt slots (8 slots for consumables)
//! - Item placement and validation

use crate::engine::{Point, Size, Rectangle};
use super::itemdat::ItemType;

/// Inventory slot size in pixels
pub const INV_SLOT_SIZE_PX: i32 = 28;
/// Half of inventory slot size
pub const INV_SLOT_HALF_SIZE_PX: i32 = INV_SLOT_SIZE_PX / 2;
/// Inventory grid dimensions
pub const INVENTORY_SIZE_IN_SLOTS: Size = Size { width: 10, height: 4 };
/// Row slot size
pub const INV_ROW_SLOT_SIZE: i32 = INVENTORY_SIZE_IN_SLOTS.width;

/// Inventory item indices
/// 
/// C++ Reference: `inv_item` enum in inv.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum InvItem {
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

/// Inventory XY slot constants
/// 
/// Since Rust enums can't have duplicate discriminant values,
/// these are defined as constants instead.
/// C++ Reference: `inv_xy_slot` enum in inv.h
pub mod inv_xy_slot {
    // Equipment slots
    pub const HEAD: u8 = 0;
    pub const RING_LEFT: u8 = 1;
    pub const RING_RIGHT: u8 = 2;
    pub const AMULET: u8 = 3;
    pub const HAND_LEFT: u8 = 4;
    pub const HAND_RIGHT: u8 = 5;
    pub const CHEST: u8 = 6;
    
    // Inventory grid (4 rows of 10 slots each)
    pub const INV_FIRST: u8 = 7;
    pub const INV_ROW1_FIRST: u8 = 7;
    pub const INV_ROW1_LAST: u8 = 16;
    pub const INV_ROW2_FIRST: u8 = 17;
    pub const INV_ROW2_LAST: u8 = 26;
    pub const INV_ROW3_FIRST: u8 = 27;
    pub const INV_ROW3_LAST: u8 = 36;
    pub const INV_ROW4_FIRST: u8 = 37;
    pub const INV_ROW4_LAST: u8 = 46;
    pub const INV_LAST: u8 = 46;
    
    // Belt slots (8 slots)
    pub const BELT_FIRST: u8 = 47;
    pub const BELT_LAST: u8 = 54;
}

/// Total number of inventory XY slots
pub const NUM_XY_SLOTS: usize = 55;

/// Item color for rendering
///
/// C++ Reference: `item_color` enum in inv.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ItemColor {
    Yellow = 0x2D, // PAL16_YELLOW + 5
    White = 0x4D,  // PAL16_GRAY + 5
    Blue = 0x6D,   // PAL16_BLUE + 5
    Red = 0x8D,    // PAL16_RED + 5
}

/// Global inventory flag - whether inventory panel is open
pub static mut INVFLAG: bool = false;

/// Inventory slot rectangles for hit testing
/// 
/// TODO: Initialize with actual positions from C++
/// Using lazy_static or OnceCell for proper initialization
pub fn get_inv_rect() -> [Rectangle; NUM_XY_SLOTS] {
    [Rectangle::default(); NUM_XY_SLOTS]
}

/// Check if an item can be placed on the belt
///
/// C++ Reference: `CanBePlacedOnBelt()` in inv.cpp
///
/// Items that can be placed on belt:
/// - Potions, scrolls, oils
/// - Single-slot items only
/// - Not gold or quest items
pub fn can_be_placed_on_belt(item: &super::item_new::Item) -> bool {
    // TODO: Implement full logic based on item type and size
    // For now, just check if it's a consumable type
    matches!(
        item._itype,
        ItemType::Misc | ItemType::Gold
    ) && item._iStatFlag
}

/// Draw inventory slot background based on item quality
///
/// C++ Reference: `InvDrawSlotBack()` in inv.cpp
pub fn inv_draw_slot_back(
    _target_position: Point,
    _size: Size,
    _item_quality: super::item_new::ItemQuality,
) {
    // TODO: Implement rendering
}

/// Check if cursor is in an inventory slot
///
/// Returns the slot index if cursor is in a slot, None otherwise
pub fn check_inv_slot(cursor_pos: Point) -> Option<u8> {
    let inv_rect = get_inv_rect();
    for (i, rect) in inv_rect.iter().enumerate() {
        if rect.contains(cursor_pos) {
            return Some(i as u8);
        }
    }
    None
}

/// Get the inventory slot rectangle for a given slot
pub fn get_inv_slot_rect(slot: u8) -> Rectangle {
    let inv_rect = get_inv_rect();
    inv_rect[slot as usize]
}

/// Calculate inventory grid position from slot index
pub fn inv_slot_to_grid_pos(slot: u8) -> Option<(i32, i32)> {
    if slot < inv_xy_slot::INV_FIRST || slot > inv_xy_slot::INV_LAST {
        return None;
    }
    
    let inv_index = slot - inv_xy_slot::INV_FIRST;
    let row = inv_index as i32 / INV_ROW_SLOT_SIZE;
    let col = inv_index as i32 % INV_ROW_SLOT_SIZE;
    
    Some((col, row))
}

/// Calculate inventory slot index from grid position
pub fn grid_pos_to_inv_slot(col: i32, row: i32) -> Option<u8> {
    if col < 0 || col >= INV_ROW_SLOT_SIZE || row < 0 || row >= INVENTORY_SIZE_IN_SLOTS.height {
        return None;
    }
    
    let slot = inv_xy_slot::INV_FIRST + (row * INV_ROW_SLOT_SIZE + col) as u8;
    Some(slot)
}

/// Check if a multi-slot item can fit at the given inventory position
pub fn can_item_fit_at_position(
    col: i32,
    row: i32,
    item_width: i32,
    item_height: i32,
    _inventory: &[i8; 40],
) -> bool {
    // Check bounds
    if col < 0 || col + item_width > INV_ROW_SLOT_SIZE {
        return false;
    }
    if row < 0 || row + item_height > INVENTORY_SIZE_IN_SLOTS.height {
        return false;
    }
    
    // TODO: Check if slots are empty
    true
}

/// Find an empty slot for an item in the inventory
pub fn find_empty_inv_slot(
    item_width: i32,
    item_height: i32,
    _inventory: &[i8; 40],
) -> Option<(i32, i32)> {
    for row in 0..INVENTORY_SIZE_IN_SLOTS.height {
        for col in 0..INV_ROW_SLOT_SIZE {
            if can_item_fit_at_position(col, row, item_width, item_height, _inventory) {
                return Some((col, row));
            }
        }
    }
    None
}

/// Auto-equip an item if possible
///
/// C++ Reference: `AutoEquip()` in inv.cpp
pub fn auto_equip(_player_id: usize, _item: &super::item_new::Item) -> bool {
    // TODO: Implement auto-equip logic
    false
}

/// Auto-place an item in belt if possible
///
/// C++ Reference: `AutoPlaceBelt()` in inv.cpp  
pub fn auto_place_belt(_player_id: usize, _item: &super::item_new::Item) -> bool {
    // TODO: Implement belt placement
    false
}

/// Auto-place an item in inventory
///
/// C++ Reference: `AutoPlaceInv()` in inv.cpp
pub fn auto_place_inv(_player_id: usize, _item: &super::item_new::Item) -> bool {
    // TODO: Implement inventory placement
    false
}
