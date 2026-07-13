//! Player Inventory System
//!
//! C++ Reference: `Source/inv.cpp` (2279 lines) and `Source/inv.h`.
//!
//! This module is a port of the DevilutionX player-inventory logic:
//! - Equipment slots (head, chest, hands, rings, amulet)
//! - Inventory grid (10x4 = 40 slots)
//! - Belt slots (8 slots for consumables)
//! - Item placement / auto-placement / paste validation
//! - Equipping, wielding, dual-wield, two-handed weapon swapping
//! - Gold stacking and accounting
//! - Hit-testing against the on-screen inventory rectangles
//!
//! To remain self-contained and unit-testable without dragging in the full
//! `Player` struct (which lives in `player_exact.rs` / `player_new.rs` and has
//! unrelated fields), inventory state is carried by [`InventoryData`]. All
//! algorithms mirror the C++ line-for-line; line numbers are cited in comments.

use crate::engine::{Point, Rectangle, Size};
use super::item_new::{Item, ItemIndex, ItemQuality};
use super::item_dat::{
    ItemClass, ItemEquipType, ItemMiscId, ItemType, InvBodyLoc,
};

// ============================================================================
// Constants  — C++ Reference: inv.h / inv.cpp
// ============================================================================

/// Inventory slot pixel size (C++ uses 29x29 per cell).
/// C++ Reference: `INV_SLOT_SIZE_PX` in inv.h.
pub const INV_SLOT_SIZE_PX: i32 = 29;
/// Half of an inventory slot, used for centering even-sized items.
pub const INV_SLOT_HALF_SIZE_PX: i32 = INV_SLOT_SIZE_PX / 2;
/// Inventory grid dimensions in cells (10 wide, 4 tall).
/// C++ Reference: `InventorySizeInSlots` in player.h.
pub const INVENTORY_SIZE_IN_SLOTS: Size = Size { width: 10, height: 4 };
/// Number of cells in the inventory grid.
pub const INVENTORY_GRID_CELLS: usize = (INVENTORY_SIZE_IN_SLOTS.width * INVENTORY_SIZE_IN_SLOTS.height) as usize;
/// Row pitch (cells per row).
pub const INV_ROW_SLOT_SIZE: i32 = INVENTORY_SIZE_IN_SLOTS.width;
/// Maximum belt items.
/// C++ Reference: `MaxBeltItems` in player.h.
pub const MAX_BELT_ITEMS: usize = 8;
/// Maximum inventory list items.
/// C++ Reference: `_pNumInv` upper bound.
pub const MAX_INV_LIST: usize = 40;
/// Maximum gold per stack.
/// C++ Reference: `MaxGold` in inv.h.
pub const MAX_GOLD: i32 = 5000;
/// Total number of on-screen inventory slot rectangles.
/// C++ Reference: `NUM_XY_SLOTS` in inv.h.
pub const NUM_XY_SLOTS: usize = 55;

// ---------------------------------------------------------------------------
// inv_xy_slot constants — C++ Reference: `inv_xy_slot` enum in inv.h
// ---------------------------------------------------------------------------
// Equipment slots
pub const SLOTXY_HEAD: u8 = 0;
pub const SLOTXY_RING_LEFT: u8 = 1;
pub const SLOTXY_RING_RIGHT: u8 = 2;
pub const SLOTXY_AMULET: u8 = 3;
pub const SLOTXY_HAND_LEFT: u8 = 4;
pub const SLOTXY_HAND_RIGHT: u8 = 5;
pub const SLOTXY_CHEST: u8 = 6;
pub const SLOTXY_EQUIPPED_FIRST: u8 = 0;
pub const SLOTXY_EQUIPPED_LAST: u8 = 6;
// Inventory grid (4 rows of 10 slots each)
pub const SLOTXY_INV_FIRST: u8 = 7;
pub const SLOTXY_INV_LAST: u8 = 46;
// Belt slots (8 slots)
pub const SLOTXY_BELT_FIRST: u8 = 47;
pub const SLOTXY_BELT_LAST: u8 = 54;

// ---------------------------------------------------------------------------
// inv_item indices — C++ Reference: `inv_item` enum in inv.h
// ---------------------------------------------------------------------------
pub const INVITEM_INV_FIRST: i8 = 7;
pub const INVITEM_INV_LAST: i8 = 46;
pub const INVITEM_BELT_FIRST: i8 = 47;
pub const INVITEM_BELT_LAST: i8 = 54;

/// Inventory slot size in pixels (legacy alias kept for callers).
pub const INV_ROW_SLOT_SIZE_PX: i32 = INV_SLOT_SIZE_PX;

// ============================================================================
// On-screen inventory slot rectangles
// ============================================================================

/// Maps from inventory slot index to its on-screen rectangle.
///
/// C++ Reference: `InvRect[]` in inv.cpp lines 78-137.
///
/// Layout:
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
pub const INV_RECT: [Rectangle; NUM_XY_SLOTS] = [
    // clang-format off
    // { x, y }, { w, h }
    Rectangle { position: Point { x: 132, y:   2 }, size: Size { width: 58, height: 59 } }, // 0  helmet
    Rectangle { position: Point { x:  47, y: 177 }, size: Size { width: 28, height: 29 } }, // 1  left ring
    Rectangle { position: Point { x: 248, y: 177 }, size: Size { width: 28, height: 29 } }, // 2  right ring
    Rectangle { position: Point { x: 205, y:  32 }, size: Size { width: 28, height: 29 } }, // 3  amulet
    Rectangle { position: Point { x:  17, y:  75 }, size: Size { width: 58, height: 86 } }, // 4  left hand
    Rectangle { position: Point { x: 248, y:  75 }, size: Size { width: 58, height: 87 } }, // 5  right hand
    Rectangle { position: Point { x: 132, y:  75 }, size: Size { width: 58, height: 87 } }, // 6  chest
    Rectangle { position: Point { x:  17, y: 222 }, size: Size { width: 29, height: 29 } }, // 7  inv row 1
    Rectangle { position: Point { x:  46, y: 222 }, size: Size { width: 29, height: 29 } }, // 8
    Rectangle { position: Point { x:  75, y: 222 }, size: Size { width: 29, height: 29 } }, // 9
    Rectangle { position: Point { x: 104, y: 222 }, size: Size { width: 29, height: 29 } }, // 10
    Rectangle { position: Point { x: 133, y: 222 }, size: Size { width: 29, height: 29 } }, // 11
    Rectangle { position: Point { x: 162, y: 222 }, size: Size { width: 29, height: 29 } }, // 12
    Rectangle { position: Point { x: 191, y: 222 }, size: Size { width: 29, height: 29 } }, // 13
    Rectangle { position: Point { x: 220, y: 222 }, size: Size { width: 29, height: 29 } }, // 14
    Rectangle { position: Point { x: 249, y: 222 }, size: Size { width: 29, height: 29 } }, // 15
    Rectangle { position: Point { x: 278, y: 222 }, size: Size { width: 29, height: 29 } }, // 16
    Rectangle { position: Point { x:  17, y: 251 }, size: Size { width: 29, height: 29 } }, // 17 inv row 2
    Rectangle { position: Point { x:  46, y: 251 }, size: Size { width: 29, height: 29 } }, // 18
    Rectangle { position: Point { x:  75, y: 251 }, size: Size { width: 29, height: 29 } }, // 19
    Rectangle { position: Point { x: 104, y: 251 }, size: Size { width: 29, height: 29 } }, // 20
    Rectangle { position: Point { x: 133, y: 251 }, size: Size { width: 29, height: 29 } }, // 21
    Rectangle { position: Point { x: 162, y: 251 }, size: Size { width: 29, height: 29 } }, // 22
    Rectangle { position: Point { x: 191, y: 251 }, size: Size { width: 29, height: 29 } }, // 23
    Rectangle { position: Point { x: 220, y: 251 }, size: Size { width: 29, height: 29 } }, // 24
    Rectangle { position: Point { x: 249, y: 251 }, size: Size { width: 29, height: 29 } }, // 25
    Rectangle { position: Point { x: 278, y: 251 }, size: Size { width: 29, height: 29 } }, // 26
    Rectangle { position: Point { x:  17, y: 280 }, size: Size { width: 29, height: 29 } }, // 27 inv row 3
    Rectangle { position: Point { x:  46, y: 280 }, size: Size { width: 29, height: 29 } }, // 28
    Rectangle { position: Point { x:  75, y: 280 }, size: Size { width: 29, height: 29 } }, // 29
    Rectangle { position: Point { x: 104, y: 280 }, size: Size { width: 29, height: 29 } }, // 30
    Rectangle { position: Point { x: 133, y: 280 }, size: Size { width: 29, height: 29 } }, // 31
    Rectangle { position: Point { x: 162, y: 280 }, size: Size { width: 29, height: 29 } }, // 32
    Rectangle { position: Point { x: 191, y: 280 }, size: Size { width: 29, height: 29 } }, // 33
    Rectangle { position: Point { x: 220, y: 280 }, size: Size { width: 29, height: 29 } }, // 34
    Rectangle { position: Point { x: 249, y: 280 }, size: Size { width: 29, height: 29 } }, // 35
    Rectangle { position: Point { x: 278, y: 280 }, size: Size { width: 29, height: 29 } }, // 36
    Rectangle { position: Point { x:  17, y: 309 }, size: Size { width: 29, height: 29 } }, // 37 inv row 4
    Rectangle { position: Point { x:  46, y: 309 }, size: Size { width: 29, height: 29 } }, // 38
    Rectangle { position: Point { x:  75, y: 309 }, size: Size { width: 29, height: 29 } }, // 39
    Rectangle { position: Point { x: 104, y: 309 }, size: Size { width: 29, height: 29 } }, // 40
    Rectangle { position: Point { x: 133, y: 309 }, size: Size { width: 29, height: 29 } }, // 41
    Rectangle { position: Point { x: 162, y: 309 }, size: Size { width: 29, height: 29 } }, // 42
    Rectangle { position: Point { x: 191, y: 309 }, size: Size { width: 29, height: 29 } }, // 43
    Rectangle { position: Point { x: 220, y: 309 }, size: Size { width: 29, height: 29 } }, // 44
    Rectangle { position: Point { x: 249, y: 309 }, size: Size { width: 29, height: 29 } }, // 45
    Rectangle { position: Point { x: 278, y: 309 }, size: Size { width: 29, height: 29 } }, // 46
    Rectangle { position: Point { x: 205, y:   5 }, size: Size { width: 29, height: 29 } }, // 47 belt
    Rectangle { position: Point { x: 234, y:   5 }, size: Size { width: 29, height: 29 } }, // 48 belt
    Rectangle { position: Point { x: 263, y:   5 }, size: Size { width: 29, height: 29 } }, // 49 belt
    Rectangle { position: Point { x: 292, y:   5 }, size: Size { width: 29, height: 29 } }, // 50 belt
    Rectangle { position: Point { x: 321, y:   5 }, size: Size { width: 29, height: 29 } }, // 51 belt
    Rectangle { position: Point { x: 350, y:   5 }, size: Size { width: 29, height: 29 } }, // 52 belt
    Rectangle { position: Point { x: 379, y:   5 }, size: Size { width: 29, height: 29 } }, // 53 belt
    Rectangle { position: Point { x: 408, y:   5 }, size: Size { width: 29, height: 29 } }, // 54 belt
    // clang-format on
];

/// Backwards-compatible getter for the inventory slot rectangles.
pub fn get_inv_rect() -> [Rectangle; NUM_XY_SLOTS] {
    INV_RECT
}

/// Global inventory-panel-open flag.
/// C++ Reference: `invflag` in inv.cpp.
pub static mut INVFLAG: bool = false;

// ============================================================================
// Inventory state container
// ============================================================================

/// Self-contained inventory state used by all functions in this module.
///
/// This mirrors the inventory-relevant subset of the C++ `Player` struct
/// (`InvBody`, `InvList`, `InvGrid`, `SpdList`, `HoldItem`, `_pNumInv`,
/// `_pGold`). Keeping it separate lets the algorithms be unit-tested without
/// the full `Player`. Callers that own a real `Player` can populate this from
/// the player's fields.
///
/// C++ Reference: `Player` in player.h.
#[derive(Debug, Clone)]
pub struct InventoryData {
    /// Equipped body items, indexed by [`InvBodyLoc`].
    /// C++ Reference: `Player::InvBody[NUM_INVLOC]`.
    pub inv_body: [Item; 7],
    /// Backpack items list (compact, first `_p_num_inv` are live).
    /// C++ Reference: `Player::InvList[]`.
    pub inv_list: [Item; MAX_INV_LIST],
    /// Backpack grid (10x4). `0` = empty, `+n` = top-left cell of item n,
    /// `-n` = occupied non-top-left cell of item n (1-based).
    /// C++ Reference: `Player::InvGrid[40]`.
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],
    /// Belt items (consumables).
    /// C++ Reference: `Player::SpdList[MaxBeltItems]`.
    pub spd_list: [Item; MAX_BELT_ITEMS],
    /// Item currently held by the mouse cursor.
    /// C++ Reference: `Player::HoldItem`.
    pub hold_item: Item,
    /// Number of live items in `inv_list`.
    /// C++ Reference: `Player::_pNumInv`.
    pub p_num_inv: usize,
    /// Total gold carried (inventory gold only).
    /// C++ Reference: `Player::_pGold`.
    pub p_gold: i32,
}

impl Default for InventoryData {
    fn default() -> Self {
        Self {
            inv_body: core::array::from_fn(|_| Item::new()),
            inv_list: core::array::from_fn(|_| Item::new()),
            inv_grid: [0; INVENTORY_GRID_CELLS],
            spd_list: core::array::from_fn(|_| Item::new()),
            hold_item: Item::new(),
            p_num_inv: 0,
            p_gold: 0,
        }
    }
}

impl InventoryData {
    /// Create an empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the equipped item at a body location.
    #[inline]
    pub fn body(&self, loc: InvBodyLoc) -> &Item {
        &self.inv_body[loc as usize]
    }

    /// Mutably borrow the equipped item at a body location.
    #[inline]
    pub fn body_mut(&mut self, loc: InvBodyLoc) -> &mut Item {
        &mut self.inv_body[loc as usize]
    }

    /// Convenience: is the given body slot empty?
    #[inline]
    pub fn body_is_empty(&self, loc: InvBodyLoc) -> bool {
        self.inv_body[loc as usize].is_empty()
    }
}

// ============================================================================
// Item helpers
// ============================================================================

/// Whether the item is "usable" (can be activated/consumed by the player).
///
/// C++ Reference: `Item::isUsable()` in items.cpp line 4784, which queries the
/// base-item-data `iUsable` flag. We approximate that flag with the canonical
/// set of usable misc categories: potions (1-19), scrolls (21-22), books (24),
/// oils (29-41), special elixir (44), runes (46-52), and arena potions (55).
pub fn item_is_usable(item: &Item) -> bool {
    use ItemMiscId::*;
    matches!(
        item._iMiscId,
        UseFirst
            | FullHeal
            | Heal
            | Mana
            | FullMana
            | ElixirStr
            | ElixirMag
            | ElixirDex
            | ElixirVit
            | Rejuv
            | FullRejuv
            | UseLast
            | Scroll
            | ScrollT
            | Book
            | OilOf
            | OilAcc
            | OilMast
            | OilSharp
            | OilDeath
            | OilSkill
            | OilBSmith
            | OilFort
            | OilPerm
            | OilHard
            | OilImp
            | SpecElixir
            | RuneF
            | RuneL
            | GrRuneL
            | GrRuneF
            | RuneS
            | ArenaPot
    )
}

/// Inventory cell size of an item.
///
/// C++ Reference: `GetInventorySize()` in inv.cpp lines 2272-2277. The C++
/// version derives this from the item's cursor-graphic dimensions; we use the
/// canonical item-type → cell-size mapping (matches the original graphics).
pub fn get_inventory_size(item: &Item) -> Size {
    match item._itype {
        ItemType::None | ItemType::Gold | ItemType::Ring | ItemType::Amulet | ItemType::Misc => {
            Size::new(1, 1)
        }
        ItemType::Sword | ItemType::Staff => Size::new(1, 3),
        ItemType::Axe | ItemType::Mace | ItemType::Bow => Size::new(2, 3),
        ItemType::Shield | ItemType::Helm | ItemType::LightArmor => Size::new(2, 2),
        ItemType::MediumArmor | ItemType::HeavyArmor => Size::new(2, 3),
    }
}

/// Returns the location an item wants to occupy on the body.
///
/// C++ Reference: `Player::GetItemLocation()` in player.cpp — collapses the
/// item's `_iLoc` to one of `ILOC_ONEHAND`/`ILOC_TWOHAND`/etc.
pub fn get_item_location(item: &Item) -> ItemEquipType {
    match item._iLoc {
        ItemEquipType::TwoHand => ItemEquipType::TwoHand,
        ItemEquipType::OneHand => ItemEquipType::OneHand,
        other => other,
    }
}

/// Adjust a gold item's cursor based on its value.
///
/// C++ Reference: `SetPlrHandGoldCurs()` in items.cpp. The cursor id encodes
/// the displayed pile size (small/medium/large).
pub fn set_plr_hand_gold_curs(item: &mut Item) {
    item._iCurs = if item._ivalue <= super::item_new::GOLD_SMALL_LIMIT {
        0 // ICURS_GOLD_SMALL
    } else if item._ivalue <= super::item_new::GOLD_MEDIUM_LIMIT {
        1 // ICURS_GOLD_MEDIUM
    } else {
        2 // ICURS_GOLD_LARGE
    };
}

// ============================================================================
// Grid placement
// ============================================================================

/// Add an item's occupancy markers to `inv_grid`.
///
/// C++ Reference: `AddItemToInvGrid()` in inv.cpp lines 150-166.
///
/// The bottom-left cell stores the positive 1-based item index; every other
/// covered cell stores the negated index to mark "occupied but not anchor".
pub fn add_item_to_inv_grid(
    inv: &mut InventoryData,
    inv_grid_index: usize,
    inv_list_index: i8,
    item_size: Size,
) {
    const PITCH: usize = 10;
    let h = item_size.height as usize;
    let w = item_size.width as usize;
    for y in 0..h {
        let row = inv_grid_index + PITCH * y;
        for x in 0..w {
            let cell = row + x;
            if cell >= inv.inv_grid.len() {
                break;
            }
            // C++ stores positive index on (x==0 && y==height-1) — the bottom-left cell.
            if x == 0 && y == h - 1 {
                inv.inv_grid[cell] = inv_list_index;
            } else {
                inv.inv_grid[cell] = -inv_list_index;
            }
        }
    }
}

/// Whether an item of the given size can occupy `slot_index` (top-left cell).
///
/// C++ Reference: `CheckItemFitsInInventorySlot()` in inv.cpp lines 659-678.
///
/// `item_index_to_ignore` lets the caller ask "would it fit if item N were
/// removed?" by treating that item's cells as free.
pub fn check_item_fits_in_inventory_slot(
    inv: &InventoryData,
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
            if xx >= 10 {
                return false;
            }
            let cell = xx + yy;
            if cell >= inv.inv_grid.len() {
                return false;
            }
            let grid_val = inv.inv_grid[cell] as i32;
            let occupied_idx = grid_val.abs() - 1;
            // Cell must be empty, or occupied only by the item we're ignoring.
            if grid_val != 0 && occupied_idx != item_index_to_ignore {
                return false;
            }
            xx += 1;
        }
        yy += 10;
    }
    true
}

/// Find the first slot that can fit an item of `item_size`.
///
/// C++ Reference: `FindSlotForItem()` in inv.cpp lines 687-735. Uses the
/// original per-height search ordering:
/// - height 1: last row left→right, then bottom-to-top/right-to-left
/// - height 2: right-to-left columns, top-to-bottom rows
/// - 1x3 / 2x3: row-major over the valid region
pub fn find_slot_for_item(inv: &InventoryData, item_size: Size) -> Option<usize> {
    find_slot_for_item_ignore(inv, item_size, None)
}

/// As [`find_slot_for_item`] but treats `item_index_to_ignore`'s cells as free.
fn find_slot_for_item_ignore(
    inv: &InventoryData,
    item_size: Size,
    item_index_to_ignore: Option<usize>,
) -> Option<usize> {
    let ignore = item_index_to_ignore.map(|i| i as i32).unwrap_or(-1);

    if item_size.height == 1 {
        for i in 30..=39 {
            if check_item_fits_in_inventory_slot(inv, i, item_size, ignore) {
                return Some(i);
            }
        }
        for x in (0..=9).rev() {
            for y in (0..=2).rev() {
                let slot = 10 * y + x;
                if check_item_fits_in_inventory_slot(inv, slot, item_size, ignore) {
                    return Some(slot as usize);
                }
            }
        }
        return None;
    }

    if item_size.height == 2 {
        let x_max = 10 - item_size.width;
        for x in (0..=x_max).rev() {
            for y in 0..3 {
                let slot = (10 * y + x) as usize;
                if check_item_fits_in_inventory_slot(inv, slot, item_size, ignore) {
                    return Some(slot);
                }
            }
        }
        return None;
    }

    if item_size == Size::new(1, 3) {
        for i in 0..20 {
            if check_item_fits_in_inventory_slot(inv, i, item_size, ignore) {
                return Some(i);
            }
        }
        return None;
    }

    if item_size == Size::new(2, 3) {
        for i in 0..9 {
            if check_item_fits_in_inventory_slot(inv, i, item_size, ignore) {
                return Some(i);
            }
        }
        for i in 10..19 {
            if check_item_fits_in_inventory_slot(inv, i, item_size, ignore) {
                return Some(i);
            }
        }
        return None;
    }

    None
}

/// Could `item` fit if the item at `item_index_to_ignore` were removed?
///
/// C++ Reference: `CouldFitItemInInventory()` in inv.cpp lines 744-747.
pub fn could_fit_item_in_inventory(
    inv: &InventoryData,
    item: &Item,
    item_index_to_ignore: Option<usize>,
) -> bool {
    find_slot_for_item_ignore(inv, get_inventory_size(item), item_index_to_ignore).is_some()
}

/// Whether an item can fit anywhere in the backpack right now.
///
/// C++ Reference: `CanFitItemInInventory()` in inv.cpp lines 1390-1393.
pub fn can_fit_item_in_inventory(inv: &InventoryData, item: &Item) -> bool {
    find_slot_for_item(inv, get_inventory_size(item)).is_some()
}

// ============================================================================
// Belt placement
// ============================================================================

/// Whether the item's footprint is a single cell (so it could sit in a belt).
///
/// C++ Reference: `FitsInBeltSlot()` in inv.cpp lines 173-176.
pub fn fits_in_belt_slot(item: &Item) -> bool {
    get_inventory_size(item) == Size::new(1, 1)
}

/// Whether the item may be placed on the belt.
///
/// C++ Reference: `CanBePlacedOnBelt()` in inv.cpp lines 1156-1162. The C++
/// version also calls `player.CanUseItem(item)`, which checks the cached
/// `_iStatFlag` (set when stats requirements are evaluated); we honour that
/// here via `item._iStatFlag`. The `inv` parameter is accepted for signature
/// parity with the C++ `Player&` overload and for future hooks.
pub fn can_be_placed_on_belt(_inv: &InventoryData, item: &Item) -> bool {
    fits_in_belt_slot(item)
        && item._itype != ItemType::Gold
        && item._iStatFlag
        && item_is_usable(item)
}

/// Place an item into the first free belt slot.
///
/// C++ Reference: `AutoPlaceItemInBelt()` in inv.cpp lines 1323-1346.
pub fn auto_place_item_in_belt(
    inv: &mut InventoryData,
    item: &Item,
    persist_item: bool,
) -> bool {
    if !can_be_placed_on_belt(inv, item) {
        return false;
    }
    for slot in inv.spd_list.iter_mut() {
        if slot.is_empty() {
            if persist_item {
                *slot = item.clone();
            }
            return true;
        }
    }
    false
}

// ============================================================================
// Inventory placement
// ============================================================================

/// Place an item into the backpack at the first fitting slot.
///
/// C++ Reference: `AutoPlaceItemInInventory()` in inv.cpp lines 1395-1411.
pub fn auto_place_item_in_inventory(inv: &mut InventoryData, item: &Item) -> bool {
    let item_size = get_inventory_size(item);
    if let Some(target_slot) = find_slot_for_item(inv, item_size) {
        let idx = inv.p_num_inv;
        inv.inv_list[idx] = item.clone();
        inv.p_num_inv += 1;
        add_item_to_inv_grid(inv, target_slot, inv.p_num_inv as i8, item_size);
        return true;
    }
    false
}

// ============================================================================
// Equipment / wielding
// ============================================================================

/// Whether an item is generally equippable.
///
/// C++ Reference: `CanEquip(const Item&)` in inv.cpp lines 184-188.
pub fn can_equip(item: &Item) -> bool {
    item.is_equipment() && item._iStatFlag
}

/// Whether the player can wield `item` in either hand.
///
/// C++ Reference: `CanWield()` in inv.cpp lines 198-234. Dual-wield (Bard) and
/// two-hand dispositions follow the original logic; the Bard class flag is
/// passed via `dual_wield` since class data lives outside this module.
pub fn can_wield(inv: &InventoryData, item: &Item, dual_wield: bool) -> bool {
    if !can_equip(item) {
        return false;
    }
    let loc = get_item_location(item);
    if !matches!(loc, ItemEquipType::OneHand | ItemEquipType::TwoHand) {
        return false;
    }

    let left_empty = inv.inv_body[InvBodyLoc::HandLeft as usize].is_empty();
    let right_empty = inv.inv_body[InvBodyLoc::HandRight as usize].is_empty();

    if left_empty && right_empty {
        return true;
    }
    if !left_empty && !right_empty {
        return false;
    }

    let occupied = if !left_empty {
        &inv.inv_body[InvBodyLoc::HandLeft as usize]
    } else {
        &inv.inv_body[InvBodyLoc::HandRight as usize]
    };

    // Bard-style dual wield: two one-handed swords/maces.
    if dual_wield {
        let occ_one_handed_blade =
            get_item_location(occupied) == ItemEquipType::OneHand
                && matches!(occupied._itype, ItemType::Sword | ItemType::Mace);
        let new_one_handed_blade =
            loc == ItemEquipType::OneHand && matches!(item._itype, ItemType::Sword | ItemType::Mace);
        if occ_one_handed_blade && new_one_handed_blade {
            return true;
        }
    }

    loc == ItemEquipType::OneHand
        && get_item_location(occupied) == ItemEquipType::OneHand
        && item._iClass != occupied._iClass
}

/// Whether `item` can be equipped at the specific body location.
///
/// C++ Reference: `CanEquip(Player&, Item&, inv_body_loc)` in inv.cpp lines 244-271.
/// `dual_wield` is the player's dual-wield class flag (see [`can_wield`]).
/// The C++ `player._pmode > PM_WALK_SIDEWAYS` walk-state gate is represented
/// by `player_busy` (true means the player is mid-action and can't equip).
pub fn can_equip_at(
    inv: &InventoryData,
    item: &Item,
    body_location: InvBodyLoc,
    dual_wield: bool,
    player_busy: bool,
) -> bool {
    if !can_equip(item) || player_busy {
        return false;
    }
    if !inv.inv_body[body_location as usize].is_empty() {
        return false;
    }
    match body_location {
        InvBodyLoc::Amulet => item._iLoc == ItemEquipType::Amulet,
        InvBodyLoc::Chest => item._iLoc == ItemEquipType::Armor,
        InvBodyLoc::HandLeft | InvBodyLoc::HandRight => can_wield(inv, item, dual_wield),
        InvBodyLoc::Head => item._iLoc == ItemEquipType::Helm,
        InvBodyLoc::RingLeft | InvBodyLoc::RingRight => item._iLoc == ItemEquipType::Ring,
    }
}

/// Place `item` into a body slot, overwriting whatever was there.
///
/// C++ Reference: `ChangeEquipment()` in inv.cpp lines 273-280.
pub fn change_equipment(inv: &mut InventoryData, body_location: InvBodyLoc, item: &Item) {
    inv.inv_body[body_location as usize] = item.clone();
}

/// Try to equip `item` at a specific body slot.
///
/// C++ Reference: `AutoEquip(Player&, Item&, inv_body_loc, ...)` in inv.cpp lines 282-299.
pub fn auto_equip_at(
    inv: &mut InventoryData,
    item: &Item,
    body_location: InvBodyLoc,
    persist_item: bool,
    dual_wield: bool,
    player_busy: bool,
) -> bool {
    if !can_equip_at(inv, item, body_location, dual_wield, player_busy) {
        return false;
    }
    if persist_item {
        change_equipment(inv, body_location, item);
    }
    true
}

/// Try to equip `item` at any valid body slot.
///
/// C++ Reference: `AutoEquip(Player&, Item&, ...)` in inv.cpp lines 1348-1361.
pub fn auto_equip(
    inv: &mut InventoryData,
    item: &Item,
    persist_item: bool,
    dual_wield: bool,
    player_busy: bool,
) -> bool {
    if !can_equip(item) {
        return false;
    }
    for loc in [
        InvBodyLoc::Head,
        InvBodyLoc::RingLeft,
        InvBodyLoc::RingRight,
        InvBodyLoc::Amulet,
        InvBodyLoc::HandLeft,
        InvBodyLoc::HandRight,
        InvBodyLoc::Chest,
    ] {
        if auto_equip_at(inv, item, loc, persist_item, dual_wield, player_busy) {
            return true;
        }
    }
    false
}

/// Clear a body slot (no inventory placement).
///
/// C++ Reference: `RemoveEquipment()` in inv.cpp lines 1314-1321.
pub fn remove_equipment(inv: &mut InventoryData, body_location: InvBodyLoc) {
    inv.inv_body[body_location as usize].clear();
}

/// Convenience: alias matching the task's `UnequipItem` naming. Clears the slot.
pub fn unequip_item(inv: &mut InventoryData, body_location: InvBodyLoc) {
    remove_equipment(inv, body_location);
}

// ============================================================================
// CheckInvPaste helpers
// ============================================================================

/// Map a slot index to the item-equip category it represents.
///
/// C++ Reference: `GetItemEquipType()` in inv.cpp lines 540-559.
pub fn get_item_equip_type(slot: u8, desired_location: ItemEquipType) -> ItemEquipType {
    match slot {
        SLOTXY_HEAD => ItemEquipType::Helm,
        SLOTXY_RING_LEFT | SLOTXY_RING_RIGHT => ItemEquipType::Ring,
        SLOTXY_AMULET => ItemEquipType::Amulet,
        SLOTXY_HAND_LEFT | SLOTXY_HAND_RIGHT => {
            if desired_location == ItemEquipType::TwoHand {
                ItemEquipType::TwoHand
            } else {
                ItemEquipType::OneHand
            }
        }
        SLOTXY_CHEST => ItemEquipType::Armor,
        s if s >= SLOTXY_BELT_FIRST => ItemEquipType::Belt,
        _ => ItemEquipType::Unequipable,
    }
}

/// Determine which already-placed item (if any) the held item would overlap.
///
/// Returns `Some(-1)` (encoded as a sentinel) when two *different* items would
/// be displaced; otherwise the 1-based index of the single overlapping item, or
/// 0 for no overlap.
///
/// C++ Reference: `CheckOverlappingItems()` in inv.cpp lines 420-447.
pub fn check_overlapping_items(
    inv: &InventoryData,
    slot: u8,
    item_size: Size,
) -> i8 {
    let origin = (slot - SLOTXY_INV_FIRST) as usize;
    let mut overlapping_id: i8 = 0;
    let row_stride = INVENTORY_SIZE_IN_SLOTS.width as usize;
    let mut row_offset = 0usize;
    while row_offset < item_size.height as usize * row_stride {
        let mut col_offset = 0usize;
        while col_offset < item_size.width as usize {
            let test_cell = origin + row_offset + col_offset;
            if test_cell >= inv.inv_grid.len() {
                col_offset += 1;
                continue;
            }
            let v = inv.inv_grid[test_cell];
            if v != 0 {
                let iv = v.abs();
                if overlapping_id != 0 {
                    if overlapping_id != iv {
                        return -1; // two different items would be displaced
                    }
                } else {
                    overlapping_id = iv;
                }
            }
            col_offset += 1;
        }
        row_offset += row_stride;
    }
    overlapping_id
}

/// Decide the previous-item id for `ChangeInvItem`'s gold vs. normal paths.
///
/// C++ Reference: `GetPrevItemId()` in inv.cpp lines 449-461.
pub fn get_prev_item_id(inv: &InventoryData, slot: u8, item_size: Size) -> i8 {
    if inv.hold_item._itype != ItemType::Gold {
        return check_overlapping_items(inv, slot, item_size);
    }
    let ii = (slot - SLOTXY_INV_FIRST) as usize;
    let item_cell_begin = inv.inv_grid[ii];
    if item_cell_begin == 0 {
        return 0;
    }
    if item_cell_begin <= 0 {
        return -item_cell_begin;
    }
    let inv_index = (item_cell_begin - 1) as usize;
    if inv.inv_list[inv_index]._itype != ItemType::Gold {
        return item_cell_begin;
    }
    0
}

/// Paste the held item into the backpack at `slot`. Returns false if the paste
/// is rejected.
///
/// C++ Reference: `ChangeInvItem()` in inv.cpp lines 463-521. The held item is
/// consumed (cleared) on success.
pub fn change_inv_item(inv: &mut InventoryData, slot: u8, item_size: Size) -> bool {
    let prev_item_id = get_prev_item_id(inv, slot, item_size);
    if prev_item_id < 0 {
        return false;
    }

    // Gold pasted onto an empty cell or existing gold pile has bespoke stacking
    // logic that bypasses the swap path below.
    // C++ Reference: inv.cpp lines 468-496.
    if inv.hold_item._itype == ItemType::Gold && prev_item_id == 0 {
        let ii = (slot - SLOTXY_INV_FIRST) as usize;
        if inv.inv_grid[ii] > 0 {
            // Stack onto an existing gold pile anchored here.
            let inv_index = (inv.inv_grid[ii] - 1) as usize;
            let gt = inv.inv_list[inv_index]._ivalue;
            let ig = inv.hold_item._ivalue + gt;
            if ig <= MAX_GOLD {
                inv.inv_list[inv_index]._ivalue = ig;
                set_plr_hand_gold_curs(&mut inv.inv_list[inv_index]);
                inv.p_gold += inv.hold_item._ivalue;
                inv.hold_item.clear();
            } else {
                let added = MAX_GOLD - gt;
                inv.p_gold += added;
                inv.hold_item._ivalue -= added;
                set_plr_hand_gold_curs(&mut inv.hold_item);
                inv.inv_list[inv_index]._ivalue = MAX_GOLD;
                inv.inv_list[inv_index]._iCurs = 2; // ICURS_GOLD_LARGE
            }
        } else {
            // New gold pile in a free cell.
            let inv_index = inv.p_num_inv;
            inv.p_gold += inv.hold_item._ivalue;
            inv.inv_list[inv_index] = inv.hold_item.clone();
            inv.hold_item.clear();
            inv.p_num_inv += 1;
            inv.inv_grid[ii] = inv.p_num_inv as i8;
        }
        return true;
    }

    // Non-gold, or gold landing on a non-gold item: append or swap.
    // C++ Reference: inv.cpp lines 497-518.
    let prev_id = if prev_item_id == 0 {
        // Brand-new item: append to inv_list and consume the held item
        // (C++ `player.HoldItem.pop()` moves + clears).
        let idx = inv.p_num_inv;
        inv.inv_list[idx] = inv.hold_item.clone();
        inv.hold_item.clear();
        inv.p_num_inv += 1;
        inv.p_num_inv as i8
    } else {
        // Swap the held item with the existing occupant of this slot.
        let inv_index = (prev_item_id - 1) as usize;
        if inv.hold_item._itype == ItemType::Gold {
            inv.p_gold += inv.hold_item._ivalue;
        }
        std::mem::swap(&mut inv.inv_list[inv_index], &mut inv.hold_item);
        if inv.hold_item._itype == ItemType::Gold {
            inv.p_gold = calculate_gold(inv);
        }
        // Clear every grid cell that referenced the swapped-out item; the
        // incoming item will re-mark its footprint below.
        let target = prev_item_id;
        let neg = -target;
        for v in inv.inv_grid.iter_mut() {
            if *v == target || *v == neg {
                *v = 0;
            }
        }
        prev_item_id
    };
    add_item_to_inv_grid(inv, (slot - SLOTXY_INV_FIRST) as usize, prev_id, item_size);
    true
}

/// Paste the held item into a belt slot (swap if occupied).
///
/// C++ Reference: `ChangeBeltItem()` in inv.cpp lines 523-538.
pub fn change_belt_item(inv: &mut InventoryData, slot: u8) {
    let ii = (slot - SLOTXY_BELT_FIRST) as usize;
    if inv.spd_list[ii].is_empty() {
        inv.spd_list[ii] = inv.hold_item.clone();
        inv.hold_item.clear();
    } else {
        std::mem::swap(&mut inv.spd_list[ii], &mut inv.hold_item);
        if inv.hold_item._itype == ItemType::Gold {
            inv.p_gold = calculate_gold(inv);
        }
    }
}

/// Put the held item into a ring/amulet/helm/chest body slot, swapping out the
/// previous occupant to the hand.
///
/// C++ Reference: `ChangeBodyEquipment()` in inv.cpp lines 342-363.
pub fn change_body_equipment(inv: &mut InventoryData, slot: u8, location: ItemEquipType) {
    let body_location = match location {
        ItemEquipType::Helm => InvBodyLoc::Head,
        ItemEquipType::Ring => {
            if slot == SLOTXY_RING_LEFT {
                InvBodyLoc::RingLeft
            } else {
                InvBodyLoc::RingRight
            }
        }
        ItemEquipType::Amulet => InvBodyLoc::Amulet,
        ItemEquipType::Armor => InvBodyLoc::Chest,
        _ => InvBodyLoc::Head, // unreachable for valid inputs
    };
    let previously_equipped = inv.inv_body[body_location as usize].clone();
    inv.inv_body[body_location as usize] = inv.hold_item.clone();
    inv.hold_item.clear();
    if !previously_equipped.is_empty() {
        inv.hold_item = previously_equipped;
    }
}

/// Put the held item into a hand slot, choosing the appropriate hand and
/// handling two-handed-weapon swaps.
///
/// C++ Reference: `ChangeEquippedItem()` in inv.cpp lines 365-386. `dual_wield`
/// carries the player's Bard dual-wield class flag.
pub fn change_equipped_item(inv: &mut InventoryData, slot: u8, dual_wield: bool) {
    let selected_hand = if slot == SLOTXY_HAND_LEFT {
        InvBodyLoc::HandLeft
    } else {
        InvBodyLoc::HandRight
    };
    let other_hand = if slot == SLOTXY_HAND_LEFT {
        InvBodyLoc::HandRight
    } else {
        InvBodyLoc::HandLeft
    };

    let other = &inv.inv_body[other_hand as usize];
    let paste_into_selected = other.is_empty()
        || other._iClass != inv.hold_item._iClass
        || (dual_wield
            && other._iClass == ItemClass::Weapon
            && inv.hold_item._iClass == ItemClass::Weapon);

    let dequip_two_handed =
        !other.is_empty() && get_item_location(other) == ItemEquipType::TwoHand;

    let paste_hand = if paste_into_selected {
        selected_hand
    } else {
        other_hand
    };

    let previously_equipped = if dequip_two_handed {
        inv.inv_body[other_hand as usize].clone()
    } else {
        inv.inv_body[paste_hand as usize].clone()
    };

    if dequip_two_handed {
        remove_equipment(inv, other_hand);
    }
    inv.inv_body[paste_hand as usize] = inv.hold_item.clone();
    inv.hold_item.clear();
    if !previously_equipped.is_empty() {
        inv.hold_item = previously_equipped;
    }
}

/// Put a two-handed item into the left hand, displacing right/left occupants to
/// the backpack or hand as needed.
///
/// C++ Reference: `ChangeTwoHandItem()` in inv.cpp lines 388-418.
pub fn change_two_hand_item(inv: &mut InventoryData) -> bool {
    let left = inv.inv_body[InvBodyLoc::HandLeft as usize].clone();
    let right = inv.inv_body[InvBodyLoc::HandRight as usize].clone();

    if !left.is_empty() && !right.is_empty() {
        // Need to free a hand; prefer dropping the shield side.
        let mut loc_to_unequip = InvBodyLoc::HandLeft;
        if right._itype == ItemType::Shield {
            loc_to_unequip = InvBodyLoc::HandRight;
        }
        let to_place = inv.inv_body[loc_to_unequip as usize].clone();
        if !auto_place_item_in_inventory(inv, &to_place) {
            return false;
        }
        if loc_to_unequip == InvBodyLoc::HandRight {
            remove_equipment(inv, InvBodyLoc::HandRight);
        } else {
            inv.inv_body[InvBodyLoc::HandLeft as usize].clear();
        }
    }

    if inv.inv_body[InvBodyLoc::HandRight as usize].is_empty() {
        let previously_equipped = inv.inv_body[InvBodyLoc::HandLeft as usize].clone();
        inv.inv_body[InvBodyLoc::HandLeft as usize] = inv.hold_item.clone();
        inv.hold_item.clear();
        if !previously_equipped.is_empty() {
            inv.hold_item = previously_equipped;
        }
    } else {
        let previously_equipped = inv.inv_body[InvBodyLoc::HandRight as usize].clone();
        remove_equipment(inv, InvBodyLoc::HandRight);
        inv.inv_body[InvBodyLoc::HandLeft as usize] = inv.hold_item.clone();
        inv.hold_item = previously_equipped;
    }
    true
}

// ============================================================================
// Cursor → slot hit testing
// ============================================================================

/// Find the slot under the cursor, accounting for the held item's footprint.
///
/// C++ Reference: `FindTargetSlotUnderItemCursor()` in inv.cpp lines 301-340.
/// `right_panel_origin`/`main_panel_origin` are the screen origins of the right
/// (inventory) and main (belt) panels.
pub fn find_target_slot_under_item_cursor(
    cursor_position: Point,
    item_size: Size,
    right_panel_origin: Point,
    main_panel_origin: Point,
) -> u8 {
    let offset = Point {
        x: cursor_position.x - right_panel_origin.x,
        y: cursor_position.y - right_panel_origin.y,
    };
    for r in SLOTXY_EQUIPPED_FIRST..=SLOTXY_EQUIPPED_LAST {
        if INV_RECT[r as usize].contains(offset) {
            return r;
        }
    }
    for r in SLOTXY_INV_FIRST..=SLOTXY_INV_LAST {
        if INV_RECT[r as usize].contains(offset) {
            // 1x1: hot pixel is already the top-left cell.
            if item_size.height <= 1 && item_size.width <= 1 {
                return r;
            }
            // Center the item on the hot pixel, clamping into bounds.
            let hot_x = (item_size.width - 1) / 2;
            let hot_y = (item_size.height - 1) / 2;
            let hot_pixel_cell = (r - SLOTXY_INV_FIRST) as i32;
            let target_row = (hot_pixel_cell / INVENTORY_SIZE_IN_SLOTS.width - hot_y)
                .clamp(0, INVENTORY_SIZE_IN_SLOTS.height - item_size.height);
            let target_col = (hot_pixel_cell % INVENTORY_SIZE_IN_SLOTS.width - hot_x)
                .clamp(0, INVENTORY_SIZE_IN_SLOTS.width - item_size.width);
            return SLOTXY_INV_FIRST
                + (target_row * INVENTORY_SIZE_IN_SLOTS.width + target_col) as u8;
        }
    }

    let offset = Point {
        x: cursor_position.x - main_panel_origin.x,
        y: cursor_position.y - main_panel_origin.y,
    };
    for r in SLOTXY_BELT_FIRST..=SLOTXY_BELT_LAST {
        if INV_RECT[r as usize].contains(offset) {
            return r;
        }
    }
    // C++ returns NUM_XY_SLOTS as a sentinel; cast to u8 (== 55).
    NUM_XY_SLOTS as u8
}

/// Find any slot under the cursor (no item-size adjustment).
///
/// C++ Reference: `FindSlotUnderCursor()` in inv.cpp lines 629-649.
pub fn find_slot_under_cursor(
    cursor_position: Point,
    right_panel_origin: Point,
    main_panel_origin: Point,
) -> Option<u8> {
    let test = Point {
        x: cursor_position.x - right_panel_origin.x,
        y: cursor_position.y - right_panel_origin.y,
    };
    for r in SLOTXY_EQUIPPED_FIRST..SLOTXY_BELT_FIRST {
        if INV_RECT[r as usize].contains(test) {
            return Some(r);
        }
    }
    let test = Point {
        x: cursor_position.x - main_panel_origin.x,
        y: cursor_position.y - main_panel_origin.y,
    };
    for r in SLOTXY_BELT_FIRST..NUM_XY_SLOTS as u8 {
        if INV_RECT[r as usize].contains(test) {
            return Some(r);
        }
    }
    None
}

/// Map an equipment slot index to its body location (valid for slots 0..=6).
///
/// C++ Reference: `MapSlotToInvBodyLoc()` in inv.cpp lines 623-627.
pub fn map_slot_to_inv_body_loc(slot: u8) -> InvBodyLoc {
    match slot {
        0 => InvBodyLoc::Head,
        1 => InvBodyLoc::RingLeft,
        2 => InvBodyLoc::RingRight,
        3 => InvBodyLoc::Amulet,
        4 => InvBodyLoc::HandLeft,
        5 => InvBodyLoc::HandRight,
        6 => InvBodyLoc::Chest,
        _ => InvBodyLoc::Head,
    }
}

// ============================================================================
// CheckInvPaste — paste the held item under the cursor
// ============================================================================

/// Paste the held item under the cursor into the appropriate slot.
///
/// C++ Reference: `CheckInvPaste()` in inv.cpp lines 561-621. `player_busy`
/// represents the `player._pmode > PM_WALK_SIDEWAYS` gate; `dual_wield` is the
/// Bard flag. Returns `false` if nothing was pasted.
pub fn check_inv_paste(
    inv: &mut InventoryData,
    cursor_position: Point,
    right_panel_origin: Point,
    main_panel_origin: Point,
    dual_wield: bool,
    player_busy: bool,
) -> bool {
    let item_size = get_inventory_size(&inv.hold_item);
    let slot = find_target_slot_under_item_cursor(
        cursor_position,
        item_size,
        right_panel_origin,
        main_panel_origin,
    );
    if slot as usize == NUM_XY_SLOTS {
        return false;
    }

    let desired = get_item_location(&inv.hold_item);
    let location = get_item_equip_type(slot, desired);

    if location == ItemEquipType::Belt {
        if !can_be_placed_on_belt(inv, &inv.hold_item) {
            return false;
        }
    } else if location != ItemEquipType::Unequipable {
        if desired != location {
            return false;
        }
    }

    if !matches!(location, ItemEquipType::Unequipable | ItemEquipType::Belt) {
        if !inv.hold_item._iStatFlag || player_busy {
            return false;
        }
    }

    match location {
        ItemEquipType::Helm | ItemEquipType::Ring | ItemEquipType::Amulet | ItemEquipType::Armor => {
            change_body_equipment(inv, slot, location);
        }
        ItemEquipType::OneHand => {
            change_equipped_item(inv, slot, dual_wield);
        }
        ItemEquipType::TwoHand => {
            if !change_two_hand_item(inv) {
                return false;
            }
        }
        ItemEquipType::Unequipable => {
            if !change_inv_item(inv, slot, item_size) {
                return false;
            }
        }
        ItemEquipType::Belt => {
            change_belt_item(inv, slot);
        }
        _ => {}
    }
    true
}

// ============================================================================
// Item removal (inv_list / spd_list compaction)
// ============================================================================

/// Remove the item at `item_index` from the backpack, compacting the list.
///
/// C++ Reference: `Player::RemoveInvItem()` in player.cpp. Clears grid refs and
/// moves the last live item into the freed slot.
pub fn remove_inv_item(inv: &mut InventoryData, item_index: usize) {
    let target_idx = (item_index + 1) as i8;
    // Clear this item's grid references.
    for v in inv.inv_grid.iter_mut() {
        if *v == target_idx || *v == -target_idx {
            *v = 0;
        }
    }

    // Compact: move last live item into the hole.
    if inv.p_num_inv == 0 {
        return;
    }
    inv.p_num_inv -= 1;
    if item_index < inv.p_num_inv {
        let last_idx = inv.p_num_inv;
        inv.inv_list[item_index] = inv.inv_list[last_idx].clone();
        // Re-point grid refs from the moved item's old index to its new one.
        let old_idx = (last_idx + 1) as i8;
        let new_idx = (item_index + 1) as i8;
        for v in inv.inv_grid.iter_mut() {
            if *v == old_idx {
                *v = new_idx;
            } else if *v == -old_idx {
                *v = -new_idx;
            }
        }
    }
    inv.inv_list[inv.p_num_inv].clear();
}

/// Remove the item at `item_index` from the belt.
///
/// C++ Reference: `Player::RemoveSpdBarItem()` in player.cpp.
pub fn remove_spd_bar_item(inv: &mut InventoryData, item_index: usize) {
    if item_index < MAX_BELT_ITEMS {
        inv.spd_list[item_index].clear();
    }
}

/// Borrow the inventory item referenced by a `location` code
/// (0..=6 body / 7..=46 backpack / 47..=54 belt).
///
/// C++ Reference: `GetInventoryItem()` in inv.cpp lines 2061-2070.
pub fn get_inventory_item<'a>(inv: &'a InventoryData, location: i8) -> &'a Item {
    if location < INVITEM_INV_FIRST {
        &inv.inv_body[location as usize]
    } else if location <= INVITEM_INV_LAST {
        &inv.inv_list[(location - INVITEM_INV_FIRST) as usize]
    } else {
        &inv.spd_list[(location - INVITEM_BELT_FIRST) as usize]
    }
}

// ============================================================================
// Gold accounting
// ============================================================================

/// Sum of all gold piles in the backpack.
///
/// C++ Reference: `CalculateGold()` in inv.cpp lines 2260-2270.
pub fn calculate_gold(inv: &InventoryData) -> i32 {
    let mut gold = 0;
    for i in 0..inv.p_num_inv {
        if inv.inv_list[i]._itype == ItemType::Gold {
            gold += inv.inv_list[i]._ivalue;
        }
    }
    gold
}

/// How much more gold the backpack can hold.
///
/// C++ Reference: `RoomForGold()` in inv.cpp lines 1479-1500.
pub fn room_for_gold(inv: &InventoryData) -> i32 {
    let mut amount = 0;
    for &item_index in inv.inv_grid.iter() {
        if item_index < 0 {
            continue;
        }
        if item_index == 0 {
            amount += MAX_GOLD;
            continue;
        }
        let gold_item = &inv.inv_list[(item_index - 1) as usize];
        if gold_item._itype != ItemType::Gold || gold_item._ivalue == MAX_GOLD {
            continue;
        }
        amount += MAX_GOLD - gold_item._ivalue;
    }
    amount
}

/// Create a gold pile in a free grid cell. Returns the leftover gold.
///
/// C++ Reference: `CreateGoldItemInInventorySlot()` in inv.cpp lines 1103-1120.
fn create_gold_item_in_inventory_slot(
    inv: &mut InventoryData,
    slot_index: usize,
    value: i32,
) -> i32 {
    if slot_index >= inv.inv_grid.len() || inv.inv_grid[slot_index] != 0 {
        return value;
    }
    let gold_value = value.min(MAX_GOLD);
    let mut gold_item = Item::new();
    gold_item._itype = ItemType::Gold;
    gold_item._ivalue = gold_value;
    gold_item.IDidx = ItemIndex::Gold;
    set_plr_hand_gold_curs(&mut gold_item);

    let idx = inv.p_num_inv;
    inv.inv_list[idx] = gold_item;
    inv.p_num_inv += 1;
    inv.inv_grid[slot_index] = inv.p_num_inv as i8;
    value - gold_value
}

/// Spread `value` gold across the backpack, topping off existing piles first.
/// Returns the leftover gold that did not fit.
///
/// C++ Reference: `AddGoldToInventory()` in inv.cpp lines 1502-1536.
pub fn add_gold_to_inventory(inv: &mut InventoryData, mut value: i32) -> i32 {
    // Top off existing piles.
    for i in 0..inv.p_num_inv {
        if value <= 0 {
            break;
        }
        let gold_item = &mut inv.inv_list[i];
        if gold_item._itype != ItemType::Gold || gold_item._ivalue >= MAX_GOLD {
            continue;
        }
        let can_add = MAX_GOLD - gold_item._ivalue;
        if value >= can_add {
            gold_item._ivalue = MAX_GOLD;
            value -= can_add;
        } else {
            gold_item._ivalue += value;
            value = 0;
        }
        set_plr_hand_gold_curs(gold_item);
    }

    // Last row, right to left.
    for i in (30..=39).rev() {
        if value <= 0 {
            break;
        }
        value = create_gold_item_in_inventory_slot(inv, i, value);
    }

    // Remaining columns, bottom to top, right to left.
    for x in (0..=9).rev() {
        for y in (0..=2).rev() {
            if value <= 0 {
                break;
            }
            let slot = 10 * y + x;
            value = create_gold_item_in_inventory_slot(inv, slot as usize, value);
        }
    }

    value
}

/// Add a held gold stack to the backpack. On success the stack is emptied.
///
/// C++ Reference: `GoldAutoPlace()` in inv.cpp lines 1538-1546.
pub fn gold_auto_place(inv: &mut InventoryData, gold_stack: &mut Item) -> bool {
    gold_stack._ivalue = add_gold_to_inventory(inv, gold_stack._ivalue);
    set_plr_hand_gold_curs(gold_stack);
    inv.p_gold = calculate_gold(inv);
    gold_stack._ivalue == 0
}

/// Task-facing alias: add `amount` gold to the backpack, returning leftover.
pub fn gold_add(inv: &mut InventoryData, amount: i32) -> i32 {
    let leftover = add_gold_to_inventory(inv, amount);
    inv.p_gold = calculate_gold(inv);
    leftover
}

/// Remove up to `amount` gold from the backpack. Returns the amount actually
/// removed (may be less if insufficient). Mirrors the withdrawal pattern used
/// by the stash/store flows.
pub fn gold_remove(inv: &mut InventoryData, amount: i32) -> i32 {
    let mut remaining = amount;
    // Drain from the largest piles first by iterating in list order; the C++
    // game removes specific piles via item indices, but for a self-contained
    // helper we sweep all gold items.
    for i in 0..inv.p_num_inv {
        if remaining <= 0 {
            break;
        }
        let gold_item = &mut inv.inv_list[i];
        if gold_item._itype != ItemType::Gold {
            continue;
        }
        let take = remaining.min(gold_item._ivalue);
        gold_item._ivalue -= take;
        remaining -= take;
        if gold_item._ivalue == 0 {
            gold_item.clear();
        } else {
            set_plr_hand_gold_curs(gold_item);
        }
    }
    inv.p_gold = calculate_gold(inv);
    amount - remaining
}

// ============================================================================
// Reorganization
// ============================================================================

/// Sort backpack items by footprint (tallest first, then widest), returning
/// the indices in sorted order.
///
/// C++ Reference: `SortItemsBySize()` in inv.cpp lines 1413-1438.
pub fn sort_items_by_size(inv: &InventoryData) -> Vec<usize> {
    let mut item_sizes: Vec<(Size, usize)> = Vec::with_capacity(inv.p_num_inv);
    for i in 0..inv.p_num_inv {
        let size = get_inventory_size(&inv.inv_list[i]);
        item_sizes.push((size, i));
    }
    // Sort by height desc, then width desc.
    item_sizes.sort_by(|a, b| {
        if a.0.height == b.0.height {
            b.0.width.cmp(&a.0.width)
        } else {
            b.0.height.cmp(&a.0.height)
        }
    });
    item_sizes.into_iter().map(|(_, idx)| idx).collect()
}

/// Re-pack the backpack, sorting items by size for tighter packing.
///
/// C++ Reference: `ReorganizeInventory()` in inv.cpp lines 1440-1477. If the
/// re-packing fails to fit everything, the original layout is restored.
pub fn reorganize_inventory(inv: &mut InventoryData) {
    let sorted_indices = sort_items_by_size(inv);

    // Snapshot.
    let temp_storage: Vec<Item> = inv.inv_list[..inv.p_num_inv].to_vec();
    let original_grid = inv.inv_grid;

    // Clear.
    for item in inv.inv_list.iter_mut().take(inv.p_num_inv) {
        item.clear();
    }
    inv.p_num_inv = 0;
    inv.inv_grid = [0; INVENTORY_GRID_CELLS];

    // Re-place.
    let mut failed = false;
    for &index in &sorted_indices {
        let item = &temp_storage[index];
        if !auto_place_item_in_inventory(inv, item) {
            failed = true;
            break;
        }
    }

    if failed {
        // Restore.
        for item in inv.inv_list.iter_mut().take(inv.p_num_inv) {
            item.clear();
        }
        inv.p_num_inv = 0;
        for item in &temp_storage {
            if !item.is_empty() {
                inv.inv_list[inv.p_num_inv] = item.clone();
                inv.p_num_inv += 1;
            }
        }
        inv.inv_grid = original_grid;
    }
}

// ============================================================================
// Rendering support (data-only; pixel output lives in the render layer)
// ============================================================================

/// Palette color-shift used to tint an inventory slot background.
///
/// C++ Reference: `InvDrawSlotBack()` in inv.cpp lines 1124-1154. The C++
/// version performs the in-place palette shift on a `Surface`; here we compute
/// the shift value so the renderer can apply it. `inspecting_other` toggles the
/// orange tint used when viewing another player's inventory.
pub fn inv_draw_slot_color_shift(item_quality: ItemQuality, inspecting_other: bool) -> u8 {
    // PAL16_GRAY = 224, PAL16_BLUE = 144, PAL16_ORANGE = 176, PAL16_YELLOW = 208, PAL16_BEIGE = 208
    const PAL16_GRAY: u8 = 224;
    const PAL16_BLUE: u8 = 144;
    const PAL16_ORANGE: u8 = 176;
    const PAL16_YELLOW: u8 = 208;
    const PAL16_BEIGE: u8 = 208;

    match item_quality {
        ItemQuality::Magic => {
            if inspecting_other {
                PAL16_GRAY - PAL16_ORANGE - 1
            } else {
                PAL16_GRAY - PAL16_BLUE - 1
            }
        }
        ItemQuality::Unique => {
            if inspecting_other {
                PAL16_GRAY - PAL16_ORANGE - 1
            } else {
                PAL16_GRAY - PAL16_YELLOW - 1
            }
        }
        _ => {
            if inspecting_other {
                PAL16_GRAY - PAL16_ORANGE - 1
            } else {
                PAL16_GRAY - PAL16_BEIGE - 1
            }
        }
    }
}

/// Layout description for one drawn inventory body slot.
///
/// C++ Reference: `DrawInv()` in inv.cpp lines 1179-1265. The C++ function
/// renders directly; this helper returns the slot's screen rectangle and
/// footprint so the renderer can draw it.
#[derive(Debug, Clone, Copy)]
pub struct InvSlotDrawInfo {
    pub body_location: InvBodyLoc,
    /// Cell footprint (e.g. 2x3 for chest).
    pub cell_size: Size,
    /// Top-left pixel within the inventory panel.
    pub panel_position: Point,
}

/// Per-slot sizes used by `DrawInv`.
///
/// C++ Reference: lines 1183-1191.
const DRAW_INV_SLOT_SIZE: [Size; 7] = [
    Size::new(2, 2), // head
    Size::new(1, 1), // left ring
    Size::new(1, 1), // right ring
    Size::new(1, 1), // amulet
    Size::new(2, 3), // left hand
    Size::new(2, 3), // right hand
    Size::new(2, 3), // chest
];

/// Per-slot panel positions used by `DrawInv`.
///
/// C++ Reference: lines 1193-1201.
const DRAW_INV_SLOT_POS: [Point; 7] = [
    Point::new(133, 59),  // head
    Point::new(48, 205),  // left ring
    Point::new(249, 205), // right ring
    Point::new(205, 60),  // amulet
    Point::new(17, 160),  // left hand
    Point::new(248, 160), // right hand
    Point::new(133, 160), // chest
];

/// Iterate the body slots that should be drawn, along with their layout.
///
/// C++ Reference: the per-slot loop in `DrawInv()` lines 1205-1239.
pub fn iter_draw_inv_slots() -> impl Iterator<Item = InvSlotDrawInfo> {
    [0usize, 1, 2, 3, 4, 5, 6].into_iter().map(|slot| {
        let loc = map_slot_to_inv_body_loc(slot as u8);
        InvSlotDrawInfo {
            body_location: loc,
            cell_size: DRAW_INV_SLOT_SIZE[slot],
            panel_position: DRAW_INV_SLOT_POS[slot],
        }
    })
}

/// Iterate the backpack cells that should be drawn (those with `inv_grid != 0`).
///
/// C++ Reference: the grid loop in `DrawInv()` lines 1241-1264. Returns
/// `(grid_index, panel_position, is_top_left)` so the renderer can draw the
/// background tint and, for top-left cells, the item sprite.
pub fn iter_draw_inv_grid(inv: &InventoryData) -> impl Iterator<Item = (usize, Point, bool)> + '_ {
    (0..INVENTORY_GRID_CELLS).filter_map(move |i| {
        if inv.inv_grid[i] != 0 {
            let slot = SLOTXY_INV_FIRST as usize + i;
            let pos = INV_RECT[slot].position;
            // The C++ panel adds one slot-height to y for the anchor.
            let pos = Point::new(pos.x, pos.y + INV_SLOT_SIZE_PX);
            Some((i, pos, inv.inv_grid[i] > 0))
        } else {
            None
        }
    })
}

/// Layout for a belt slot to draw.
///
/// C++ Reference: `DrawInvBelt()` in inv.cpp lines 1267-1312.
#[derive(Debug, Clone, Copy)]
pub struct BeltSlotDrawInfo {
    pub belt_index: usize,
    pub panel_position: Point,
}

/// Iterate the belt slots, with their on-panel positions.
///
/// C++ Reference: the loop in `DrawInvBelt()` lines 1279-1311.
pub fn iter_draw_inv_belt() -> impl Iterator<Item = BeltSlotDrawInfo> {
    (0..MAX_BELT_ITEMS).into_iter().map(|i| {
        let slot = SLOTXY_BELT_FIRST as usize + i;
        BeltSlotDrawInfo {
            belt_index: i,
            panel_position: INV_RECT[slot].position,
        }
    })
}

// ============================================================================
// Hover/lightbox lookup
// ============================================================================

/// Result of hovering over an inventory slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvHLightResult {
    /// The `inv_item` index to highlight (matches C++ `rv`), or `None`.
    pub item_index: Option<i8>,
}

/// Determine which inventory item the mouse is hovering over for the info box.
///
/// C++ Reference: `CheckInvHLight()` in inv.cpp lines 1915-2000. Returns the
/// item index (body loc, or `INVITEM_INV_FIRST`-based, or
/// `INVITEM_BELT_FIRST`-based) or `None` when no item is under the cursor.
///
/// `right_panel_origin`/`main_panel_origin` are the screen origins of the
/// inventory and main panels.
pub fn check_inv_hlight(
    inv: &InventoryData,
    cursor_position: Point,
    right_panel_origin: Point,
    main_panel_origin: Point,
) -> InvHLightResult {
    let mut r = 0u8;
    let mut found = false;
    while (r as usize) < NUM_XY_SLOTS {
        let origin = if r >= SLOTXY_BELT_FIRST {
            main_panel_origin
        } else {
            right_panel_origin
        };
        let test = Point {
            x: cursor_position.x - origin.x,
            y: cursor_position.y - origin.y,
        };
        if INV_RECT[r as usize].contains(test) {
            found = true;
            break;
        }
        r += 1;
    }

    if !found {
        return InvHLightResult { item_index: None };
    }

    // Resolve the candidate item index and a reference to the item (if any)
    // so we can apply the C++ `if (pi->isEmpty()) return -1;` gate uniformly.
    let (rv, item_empty): (Option<i8>, bool) = match r {
        SLOTXY_HEAD => {
            let loc = InvBodyLoc::Head;
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        SLOTXY_RING_LEFT => {
            let loc = InvBodyLoc::RingLeft;
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        SLOTXY_RING_RIGHT => {
            let loc = InvBodyLoc::RingRight;
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        SLOTXY_AMULET => {
            let loc = InvBodyLoc::Amulet;
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        SLOTXY_HAND_LEFT => {
            let loc = InvBodyLoc::HandLeft;
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        SLOTXY_HAND_RIGHT => {
            // A two-handed weapon in the left hand occupies the right slot visually.
            let left = inv.body(InvBodyLoc::HandLeft);
            let loc = if !left.is_empty()
                && get_item_location(left) == ItemEquipType::TwoHand
            {
                InvBodyLoc::HandLeft
            } else {
                InvBodyLoc::HandRight
            };
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        SLOTXY_CHEST => {
            let loc = InvBodyLoc::Chest;
            (Some(loc as i8), inv.body(loc).is_empty())
        }
        s if s >= SLOTXY_INV_FIRST && s <= SLOTXY_INV_LAST => {
            let item_id = inv.inv_grid[(s - SLOTXY_INV_FIRST) as usize].abs();
            if item_id == 0 {
                (None, true)
            } else {
                let ii = (item_id - 1) as i8;
                (Some(ii + INVITEM_INV_FIRST), false)
            }
        }
        s if s >= SLOTXY_BELT_FIRST => {
            let idx = (s - SLOTXY_BELT_FIRST) as usize;
            (Some(idx as i8 + INVITEM_BELT_FIRST), inv.spd_list[idx].is_empty())
        }
        _ => (None, true),
    };

    // C++ line 1981-1982: empty slots yield no highlight.
    InvHLightResult {
        item_index: if item_empty { None } else { rv },
    }
}

// ============================================================================
// High-level convenience: AutoGetItem / InvGetItem (backpack/belt only)
// ============================================================================

/// Try to place `item` into the inventory: auto-equip → belt → backpack.
///
/// This is the backpack/belt portion of `AutoGetItem` (the ground-item pickup
/// entry point). The full C++ function also handles the ground `Items[]` array
/// and quest hooks, which live outside this module.
///
/// C++ Reference: `AutoGetItem()` in inv.cpp lines 1734-1783.
pub fn auto_get_item_to_inventory(
    inv: &mut InventoryData,
    item: &Item,
    dual_wield: bool,
    player_busy: bool,
    auto_equip_enabled: bool,
) -> bool {
    if item._itype == ItemType::Gold {
        let mut gold = item.clone();
        return gold_auto_place(inv, &mut gold);
    }

    let mut done = false;
    if auto_equip_enabled {
        done = auto_equip(inv, item, true, dual_wield, player_busy);
    }
    if !done {
        done = auto_place_item_in_belt(inv, item, true);
    }
    if !done {
        done = auto_place_item_in_inventory(inv, item);
    }
    done
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::item_dat::{ItemEquipType, ItemMiscId, ItemType};
    use super::super::item_new::{Item, ItemIndex, ItemQuality};

    fn gold_item(value: i32) -> Item {
        let mut item = Item::new();
        item._itype = ItemType::Gold;
        item._ivalue = value;
        item.IDidx = ItemIndex::Gold;
        item._iStatFlag = true;
        item
    }

    fn potion_item() -> Item {
        let mut item = Item::new();
        item._itype = ItemType::Misc;
        item._iMiscId = ItemMiscId::Heal;
        item._iLoc = ItemEquipType::Unequipable;
        item._iStatFlag = true;
        item
    }

    fn sword_item() -> Item {
        let mut item = Item::new();
        item._itype = ItemType::Sword;
        item._iLoc = ItemEquipType::OneHand;
        item._iClass = ItemClass::Weapon;
        item._iStatFlag = true;
        item
    }

    fn shield_item() -> Item {
        let mut item = Item::new();
        item._itype = ItemType::Shield;
        item._iLoc = ItemEquipType::OneHand;
        item._iClass = ItemClass::Armor;
        item._iStatFlag = true;
        item
    }

    fn helm_item() -> Item {
        let mut item = Item::new();
        item._itype = ItemType::Helm;
        item._iLoc = ItemEquipType::Helm;
        item._iClass = ItemClass::Armor;
        item._iStatFlag = true;
        item
    }

    fn bow_item() -> Item {
        let mut item = Item::new();
        item._itype = ItemType::Bow;
        item._iLoc = ItemEquipType::TwoHand;
        item._iClass = ItemClass::Weapon;
        item._iStatFlag = true;
        item
    }

    // ---- constants / table ----

    #[test]
    fn test_inv_rect_count() {
        assert_eq!(INV_RECT.len(), NUM_XY_SLOTS);
        assert_eq!(NUM_XY_SLOTS, 55);
    }

    #[test]
    fn test_inv_rect_head_position() {
        let head = &INV_RECT[SLOTXY_HEAD as usize];
        assert_eq!(head.position.x, 132);
        assert_eq!(head.position.y, 2);
        assert_eq!(head.size.width, 58);
    }

    #[test]
    fn test_inv_rect_belt_range() {
        for i in 0..MAX_BELT_ITEMS {
            let slot = SLOTXY_BELT_FIRST as usize + i;
            assert_eq!(INV_RECT[slot].size.width, 29);
            assert_eq!(INV_RECT[slot].position.y, 5);
        }
    }

    // ---- item helpers ----

    #[test]
    fn test_get_inventory_size() {
        assert_eq!(get_inventory_size(&gold_item(1)), Size::new(1, 1));
        assert_eq!(get_inventory_size(&sword_item()), Size::new(1, 3));
        assert_eq!(get_inventory_size(&bow_item()), Size::new(2, 3));
        assert_eq!(get_inventory_size(&helm_item()), Size::new(2, 2));
    }

    #[test]
    fn test_item_is_usable() {
        assert!(item_is_usable(&potion_item()));
        let mut scroll = Item::new();
        scroll._iMiscId = ItemMiscId::Scroll;
        assert!(item_is_usable(&scroll));
        // Plain misc with None id is not usable.
        let mut none = Item::new();
        none._iMiscId = ItemMiscId::None;
        assert!(!item_is_usable(&none));
    }

    #[test]
    fn test_set_plr_hand_gold_curs() {
        let mut g = gold_item(500);
        set_plr_hand_gold_curs(&mut g);
        assert_eq!(g._iCurs, 0); // small
        g._ivalue = 2000;
        set_plr_hand_gold_curs(&mut g);
        assert_eq!(g._iCurs, 1); // medium
        g._ivalue = 4000;
        set_plr_hand_gold_curs(&mut g);
        assert_eq!(g._iCurs, 2); // large
    }

    // ---- belt ----

    #[test]
    fn test_fits_in_belt_slot() {
        assert!(fits_in_belt_slot(&potion_item()));
        assert!(!fits_in_belt_slot(&sword_item()));
    }

    #[test]
    fn test_auto_place_item_in_belt_fills_first_empty() {
        let mut inv = InventoryData::new();
        let potion = potion_item();
        assert!(auto_place_item_in_belt(&mut inv, &potion, true));
        assert!(!inv.spd_list[0].is_empty());
        assert!(inv.spd_list[1].is_empty());

        // Second item goes into the next slot.
        assert!(auto_place_item_in_belt(&mut inv, &potion, true));
        assert!(!inv.spd_list[1].is_empty());
    }

    #[test]
    fn test_auto_place_item_in_belt_rejects_non_usable() {
        let mut inv = InventoryData::new();
        // Sword is 1x3, doesn't fit a belt slot.
        assert!(!auto_place_item_in_belt(&mut inv, &sword_item(), true));
    }

    #[test]
    fn test_auto_place_item_in_belt_no_persist() {
        let mut inv = InventoryData::new();
        let potion = potion_item();
        assert!(auto_place_item_in_belt(&mut inv, &potion, false));
        // Not persisted: belt stays empty.
        assert!(inv.spd_list[0].is_empty());
    }

    // ---- inventory placement ----

    #[test]
    fn test_auto_place_item_in_inventory_potion() {
        let mut inv = InventoryData::new();
        let potion = potion_item();
        assert!(auto_place_item_in_inventory(&mut inv, &potion));
        assert_eq!(inv.p_num_inv, 1);
        // 1x1 items prefer the last row (slot 30..39).
        assert_eq!(inv.inv_grid[30], 1);
    }

    #[test]
    fn test_auto_place_item_in_inventory_sword() {
        let mut inv = InventoryData::new();
        let sword = sword_item();
        assert!(auto_place_item_in_inventory(&mut inv, &sword));
        assert_eq!(inv.p_num_inv, 1);
        // A 1x3 item placed at grid 0 occupies cells 0 (row 0), 10 (row 1),
        // and 20 (row 2). The anchor (positive index) is the bottom-left cell,
        // i.e. grid 20; cells 0 and 10 hold the negated index.
        assert_eq!(inv.inv_grid[0], -1);
        assert_eq!(inv.inv_grid[10], -1);
        assert_eq!(inv.inv_grid[20], 1);
    }

    #[test]
    fn test_auto_place_item_in_inventory_fills_up() {
        let mut inv = InventoryData::new();
        // Fill the grid with 2x3 bows. A 10x4 grid holds 5 such items: five
        // fit across rows 0-2 (columns 0-1, 2-3, ..., 8-9), and the single
        // remaining row 3 can't fit a height-3 item.
        let bow = bow_item();
        let mut placed = 0;
        while auto_place_item_in_inventory(&mut inv, &bow) {
            placed += 1;
        }
        assert_eq!(placed, 5);
        // No more room.
        assert!(!auto_place_item_in_inventory(&mut inv, &bow));
    }

    #[test]
    fn test_check_item_fits_and_find_slot() {
        let mut inv = InventoryData::new();
        // Occupy slot 0 with a 1x1.
        inv.inv_grid[0] = 1;
        inv.p_num_inv = 1;
        // A 1x1 still fits elsewhere.
        assert!(find_slot_for_item(&inv, Size::new(1, 1)).is_some());
        // Slot 0 itself is taken.
        assert!(!check_item_fits_in_inventory_slot(&inv, 0, Size::new(1, 1), -1));
    }

    #[test]
    fn test_can_fit_item_in_inventory() {
        let mut inv = InventoryData::new();
        assert!(can_fit_item_in_inventory(&mut inv, &bow_item()));
        // Fill the grid with 2x3 bows (5 fit in a 10x4 grid).
        for _ in 0..5 {
            auto_place_item_in_inventory(&mut inv, &bow_item());
        }
        assert!(!can_fit_item_in_inventory(&inv, &bow_item()));
    }

    // ---- equip / wield ----

    #[test]
    fn test_can_equip_basic() {
        assert!(can_equip(&helm_item()));
        // Stat flag false ⇒ not equippable.
        let mut h = helm_item();
        h._iStatFlag = false;
        assert!(!can_equip(&h));
        // Potions aren't equipment.
        assert!(!can_equip(&potion_item()));
    }

    #[test]
    fn test_can_wield_empty_hands() {
        let inv = InventoryData::new();
        assert!(can_wield(&inv, &sword_item(), false));
        assert!(can_wield(&inv, &bow_item(), false));
    }

    #[test]
    fn test_can_wield_sword_then_shield() {
        let mut inv = InventoryData::new();
        // Equip a sword in the left hand.
        inv.inv_body[InvBodyLoc::HandLeft as usize] = sword_item();
        // A shield (armor class) can go in the right hand.
        assert!(can_wield(&inv, &shield_item(), false));
        // A second sword cannot (same class).
        assert!(!can_wield(&inv, &sword_item(), false));
    }

    #[test]
    fn test_can_wield_dual_wield_bard() {
        let mut inv = InventoryData::new();
        inv.inv_body[InvBodyLoc::HandLeft as usize] = sword_item();
        // With dual_wield=true, a second sword is allowed.
        assert!(can_wield(&inv, &sword_item(), true));
    }

    #[test]
    fn test_can_wield_two_hands_blocked_when_one_occupied() {
        let mut inv = InventoryData::new();
        inv.inv_body[InvBodyLoc::HandLeft as usize] = sword_item();
        // Two-handed bow can't be wielded while a hand is occupied.
        assert!(!can_wield(&inv, &bow_item(), false));
    }

    #[test]
    fn test_auto_equip_helm_into_empty_slot() {
        let mut inv = InventoryData::new();
        let helm = helm_item();
        assert!(auto_equip_at(
            &mut inv,
            &helm,
            InvBodyLoc::Head,
            true,
            false,
            false
        ));
        assert!(!inv.body(InvBodyLoc::Head).is_empty());
    }

    #[test]
    fn test_auto_equip_finds_right_slot() {
        let mut inv = InventoryData::new();
        let helm = helm_item();
        assert!(auto_equip(&mut inv, &helm, true, false, false));
        assert!(!inv.body(InvBodyLoc::Head).is_empty());
        // All other slots remain empty.
        assert!(inv.body(InvBodyLoc::Chest).is_empty());
    }

    #[test]
    fn test_auto_equip_rejects_busy_player() {
        let mut inv = InventoryData::new();
        let helm = helm_item();
        assert!(!auto_equip(&mut inv, &helm, true, false, true));
        assert!(inv.body(InvBodyLoc::Head).is_empty());
    }

    #[test]
    fn test_remove_equipment_clears_slot() {
        let mut inv = InventoryData::new();
        inv.inv_body[InvBodyLoc::Head as usize] = helm_item();
        remove_equipment(&mut inv, InvBodyLoc::Head);
        assert!(inv.body(InvBodyLoc::Head).is_empty());
    }

    #[test]
    fn test_unequip_item_alias() {
        let mut inv = InventoryData::new();
        inv.inv_body[InvBodyLoc::Chest as usize] = helm_item();
        unequip_item(&mut inv, InvBodyLoc::Chest);
        assert!(inv.body(InvBodyLoc::Chest).is_empty());
    }

    // ---- gold ----

    #[test]
    fn test_calculate_gold_empty() {
        let inv = InventoryData::new();
        assert_eq!(calculate_gold(&inv), 0);
    }

    #[test]
    fn test_calculate_gold_sums_piles() {
        let mut inv = InventoryData::new();
        inv.inv_list[0] = gold_item(100);
        inv.inv_list[1] = gold_item(250);
        inv.p_num_inv = 2;
        assert_eq!(calculate_gold(&inv), 350);
    }

    #[test]
    fn test_room_for_gold_empty_inventory() {
        let inv = InventoryData::new();
        // 40 free cells × MAX_GOLD.
        assert_eq!(room_for_gold(&inv), 40 * MAX_GOLD);
    }

    #[test]
    fn test_add_gold_to_inventory_creates_piles() {
        let mut inv = InventoryData::new();
        let leftover = add_gold_to_inventory(&mut inv, 12_000);
        // 12000 / 5000 = 2 full piles + 2000 leftover in a third pile ⇒ 0 leftover.
        assert_eq!(leftover, 0);
        assert_eq!(calculate_gold(&inv), 12_000);
    }

    #[test]
    fn test_add_gold_to_inventory_tops_off() {
        let mut inv = InventoryData::new();
        inv.inv_list[0] = gold_item(4000);
        inv.p_num_inv = 1;
        inv.inv_grid[30] = 1; // anchor for the existing pile
        let leftover = add_gold_to_inventory(&mut inv, 2000);
        assert_eq!(leftover, 0);
        // The existing pile should now be full.
        assert_eq!(inv.inv_list[0]._ivalue, MAX_GOLD);
    }

    #[test]
    fn test_add_gold_to_inventory_overflow() {
        let mut inv = InventoryData::new();
        // 40 cells × 5000 = 200000 capacity.
        let leftover = add_gold_to_inventory(&mut inv, 250_000);
        assert_eq!(leftover, 50_000);
        assert_eq!(calculate_gold(&inv), 200_000);
    }

    #[test]
    fn test_gold_auto_place_empties_stack() {
        let mut inv = InventoryData::new();
        let mut stack = gold_item(3000);
        assert!(gold_auto_place(&mut inv, &mut stack));
        assert_eq!(stack._ivalue, 0);
        assert_eq!(calculate_gold(&inv), 3000);
    }

    #[test]
    fn test_gold_auto_place_partial_when_full() {
        let mut inv = InventoryData::new();
        // Fill the inventory completely.
        add_gold_to_inventory(&mut inv, 200_000);
        let mut stack = gold_item(1000);
        assert!(!gold_auto_place(&mut inv, &mut stack));
        // Nothing fit.
        assert_eq!(stack._ivalue, 1000);
    }

    #[test]
    fn test_gold_add_and_remove() {
        let mut inv = InventoryData::new();
        assert_eq!(gold_add(&mut inv, 10_000), 0);
        assert_eq!(inv.p_gold, 10_000);
        let removed = gold_remove(&mut inv, 6000);
        assert_eq!(removed, 6000);
        assert_eq!(inv.p_gold, 4000);
    }

    #[test]
    fn test_gold_remove_insufficient() {
        let mut inv = InventoryData::new();
        gold_add(&mut inv, 3000);
        let removed = gold_remove(&mut inv, 9000);
        assert_eq!(removed, 3000);
    }

    // ---- inv_list compaction ----

    #[test]
    fn test_remove_inv_item_compacts() {
        let mut inv = InventoryData::new();
        auto_place_item_in_inventory(&mut inv, &potion_item());
        auto_place_item_in_inventory(&mut inv, &potion_item());
        assert_eq!(inv.p_num_inv, 2);
        // Remove the first; the second should slide into slot 0.
        remove_inv_item(&mut inv, 0);
        assert_eq!(inv.p_num_inv, 1);
        assert!(!inv.inv_list[0].is_empty());
        // Grid references re-pointed to index 1.
        let mut refs = 0;
        for &v in inv.inv_grid.iter() {
            if v.abs() == 1 {
                refs += 1;
            }
        }
        assert_eq!(refs, 1);
    }

    #[test]
    fn test_remove_spd_bar_item() {
        let mut inv = InventoryData::new();
        inv.spd_list[2] = potion_item();
        remove_spd_bar_item(&mut inv, 2);
        assert!(inv.spd_list[2].is_empty());
    }

    // ---- paste / hit-testing ----

    #[test]
    fn test_get_item_equip_type_mapping() {
        assert_eq!(get_item_equip_type(SLOTXY_HEAD, ItemEquipType::None), ItemEquipType::Helm);
        assert_eq!(
            get_item_equip_type(SLOTXY_RING_LEFT, ItemEquipType::None),
            ItemEquipType::Ring
        );
        assert_eq!(
            get_item_equip_type(SLOTXY_HAND_LEFT, ItemEquipType::TwoHand),
            ItemEquipType::TwoHand
        );
        assert_eq!(
            get_item_equip_type(SLOTXY_HAND_LEFT, ItemEquipType::OneHand),
            ItemEquipType::OneHand
        );
        assert_eq!(get_item_equip_type(SLOTXY_CHEST, ItemEquipType::None), ItemEquipType::Armor);
        assert_eq!(
            get_item_equip_type(SLOTXY_BELT_FIRST, ItemEquipType::None),
            ItemEquipType::Belt
        );
        // An inventory-grid slot maps to Unequipable.
        assert_eq!(
            get_item_equip_type(SLOTXY_INV_FIRST, ItemEquipType::None),
            ItemEquipType::Unequipable
        );
    }

    #[test]
    fn test_find_slot_under_cursor_body_slot() {
        // Helm rectangle: x 132..190, y 2..61.
        let r = find_slot_under_cursor(
            Point::new(150, 30),
            Point::new(0, 0),
            Point::new(0, 0),
        );
        assert_eq!(r, Some(SLOTXY_HEAD));
    }

    #[test]
    fn test_find_slot_under_cursor_belt() {
        // Belt slot 47: x 205..234, y 5..34, on the main panel.
        let r = find_slot_under_cursor(
            Point::new(220, 20),
            Point::new(1000, 1000), // right panel far away
            Point::new(0, 0),
        );
        assert_eq!(r, Some(SLOTXY_BELT_FIRST));
    }

    #[test]
    fn test_find_slot_under_cursor_miss() {
        let r = find_slot_under_cursor(
            Point::new(5000, 5000),
            Point::new(0, 0),
            Point::new(0, 0),
        );
        assert_eq!(r, None);
    }

    #[test]
    fn test_find_target_slot_for_1x1_in_inventory() {
        // Slot 30 (inv row 4, col 0): x 17..46, y 309..338.
        let slot = find_target_slot_under_item_cursor(
            Point::new(30, 320),
            Size::new(1, 1),
            Point::new(0, 0),
            Point::new(0, 0),
        );
        // Should land somewhere in the inventory region.
        assert!((SLOTXY_INV_FIRST..=SLOTXY_INV_LAST).contains(&slot));
    }

    #[test]
    fn test_find_target_slot_returns_sentinel_on_miss() {
        let slot = find_target_slot_under_item_cursor(
            Point::new(5000, 5000),
            Size::new(1, 1),
            Point::new(0, 0),
            Point::new(0, 0),
        );
        assert_eq!(slot as usize, NUM_XY_SLOTS);
    }

    #[test]
    fn test_check_inv_hlight_head() {
        let inv = InventoryData::new();
        let res = check_inv_hlight(
            &inv,
            Point::new(150, 30),
            Point::new(0, 0),
            Point::new(0, 0),
        );
        // Head slot empty ⇒ no item.
        assert_eq!(res.item_index, None);

        // With an item equipped:
        let mut inv = inv;
        inv.inv_body[InvBodyLoc::Head as usize] = helm_item();
        let res = check_inv_hlight(&inv, Point::new(150, 30), Point::new(0, 0), Point::new(0, 0));
        assert_eq!(res.item_index, Some(InvBodyLoc::Head as i8));
    }

    #[test]
    fn test_check_inv_hlight_inventory_item() {
        let mut inv = InventoryData::new();
        auto_place_item_in_inventory(&mut inv, &potion_item());
        // A 1x1 potion lands at grid index 30 (last row, col 0), i.e. slot 37,
        // whose rectangle is x 17..46, y 309..338.
        let res = check_inv_hlight(&inv, Point::new(30, 320), Point::new(0, 0), Point::new(0, 0));
        // Should map to INVITEM_INV_FIRST + 0 (first item).
        assert!(res.item_index.is_some());
        assert_eq!(res.item_index.unwrap(), INVITEM_INV_FIRST);
    }

    // ---- paste flows ----

    #[test]
    fn test_check_inv_paste_potion_into_inventory() {
        let mut inv = InventoryData::new();
        inv.hold_item = potion_item();
        // Cursor over inventory slot 30.
        let ok = check_inv_paste(
            &mut inv,
            Point::new(115, 320),
            Point::new(0, 0),
            Point::new(0, 0),
            false,
            false,
        );
        assert!(ok);
        assert_eq!(inv.p_num_inv, 1);
        assert!(inv.hold_item.is_empty());
    }

    #[test]
    fn test_check_inv_paste_helm_into_head_slot() {
        let mut inv = InventoryData::new();
        inv.hold_item = helm_item();
        let ok = check_inv_paste(
            &mut inv,
            Point::new(150, 30),
            Point::new(0, 0),
            Point::new(0, 0),
            false,
            false,
        );
        assert!(ok);
        assert!(!inv.body(InvBodyLoc::Head).is_empty());
        assert!(inv.hold_item.is_empty());
    }

    #[test]
    fn test_check_inv_paste_into_belt() {
        let mut inv = InventoryData::new();
        inv.hold_item = potion_item();
        // Belt slot 47: x 205..234, y 5..34 on the main panel.
        let ok = check_inv_paste(
            &mut inv,
            Point::new(220, 20),
            Point::new(1000, 1000), // push right panel away
            Point::new(0, 0),
            false,
            false,
        );
        assert!(ok);
        assert!(!inv.spd_list[0].is_empty());
    }

    #[test]
    fn test_check_inv_paste_rejects_wrong_location() {
        let mut inv = InventoryData::new();
        // A helm can't go into the ring slot.
        inv.hold_item = helm_item();
        // Ring-left rect: x 47..75, y 177..206.
        let ok = check_inv_paste(
            &mut inv,
            Point::new(60, 190),
            Point::new(0, 0),
            Point::new(0, 0),
            false,
            false,
        );
        assert!(!ok);
        assert!(inv.body(InvBodyLoc::Head).is_empty());
    }

    // ---- reorganize ----

    #[test]
    fn test_reorganize_inventory_keeps_items() {
        let mut inv = InventoryData::new();
        // Scatter a few items.
        auto_place_item_in_inventory(&mut inv, &potion_item());
        auto_place_item_in_inventory(&mut inv, &sword_item());
        let count_before = inv.p_num_inv;
        reorganize_inventory(&mut inv);
        // Same number of items after reorg.
        assert_eq!(inv.p_num_inv, count_before);
    }

    #[test]
    fn test_sort_items_by_size_orders_by_height() {
        let mut inv = InventoryData::new();
        auto_place_item_in_inventory(&mut inv, &potion_item()); // 1x1
        auto_place_item_in_inventory(&mut inv, &bow_item()); // 2x3
        let order = sort_items_by_size(&inv);
        // Tallest first: bow (2x3) before potion (1x1).
        assert_eq!(order.len(), 2);
        let first_size = get_inventory_size(&inv.inv_list[order[0]]);
        assert!(first_size.height >= 2);
    }

    // ---- rendering data ----

    #[test]
    fn test_inv_draw_slot_color_shift_magic() {
        let shift = inv_draw_slot_color_shift(ItemQuality::Magic, false);
        // PAL16_GRAY(224) - PAL16_BLUE(144) - 1 = 79
        assert_eq!(shift, 79);
        let shift_inspect = inv_draw_slot_color_shift(ItemQuality::Magic, true);
        // PAL16_GRAY(224) - PAL16_ORANGE(176) - 1 = 47
        assert_eq!(shift_inspect, 47);
    }

    #[test]
    fn test_iter_draw_inv_slots_count() {
        assert_eq!(iter_draw_inv_slots().count(), 7);
    }

    #[test]
    fn test_iter_draw_inv_belt_count() {
        assert_eq!(iter_draw_inv_belt().count(), MAX_BELT_ITEMS);
    }

    #[test]
    fn test_iter_draw_inv_grid_empty() {
        let inv = InventoryData::new();
        assert_eq!(iter_draw_inv_grid(&inv).count(), 0);
    }

    #[test]
    fn test_iter_draw_inv_grid_with_items() {
        let mut inv = InventoryData::new();
        auto_place_item_in_inventory(&mut inv, &potion_item());
        let drawn: Vec<_> = iter_draw_inv_grid(&inv).collect();
        // 1x1 ⇒ one cell drawn, and it's the top-left (anchor).
        assert_eq!(drawn.len(), 1);
        assert!(drawn[0].2); // is_top_left
    }

    // ---- auto_get_item ----

    #[test]
    fn test_auto_get_item_potion_to_inventory() {
        let mut inv = InventoryData::new();
        let potion = potion_item();
        assert!(auto_get_item_to_inventory(&mut inv, &potion, false, false, false));
        // Should land in the belt (first free slot).
        assert!(!inv.spd_list[0].is_empty());
    }

    #[test]
    fn test_auto_get_item_helm_auto_equips() {
        let mut inv = InventoryData::new();
        let helm = helm_item();
        assert!(auto_get_item_to_inventory(&mut inv, &helm, false, false, true));
        assert!(!inv.body(InvBodyLoc::Head).is_empty());
    }

    #[test]
    fn test_auto_get_item_gold() {
        let mut inv = InventoryData::new();
        let g = gold_item(1500);
        assert!(auto_get_item_to_inventory(&mut inv, &g, false, false, false));
        assert_eq!(calculate_gold(&inv), 1500);
    }

    #[test]
    fn test_get_inventory_item_body() {
        let mut inv = InventoryData::new();
        inv.inv_body[InvBodyLoc::Head as usize] = helm_item();
        let item = get_inventory_item(&inv, InvBodyLoc::Head as i8);
        assert!(!item.is_empty());
    }

    #[test]
    fn test_get_inventory_item_belt() {
        let mut inv = InventoryData::new();
        inv.spd_list[3] = potion_item();
        let loc = INVITEM_BELT_FIRST + 3;
        let item = get_inventory_item(&inv, loc);
        assert!(!item.is_empty());
    }

    #[test]
    fn test_map_slot_to_inv_body_loc() {
        assert_eq!(map_slot_to_inv_body_loc(0), InvBodyLoc::Head);
        assert_eq!(map_slot_to_inv_body_loc(4), InvBodyLoc::HandLeft);
        assert_eq!(map_slot_to_inv_body_loc(6), InvBodyLoc::Chest);
    }

    #[test]
    fn test_could_fit_with_ignore() {
        let mut inv = InventoryData::new();
        // Fill the inventory with 2x3 bows (5 fit in a 10x4 grid).
        for _ in 0..5 {
            auto_place_item_in_inventory(&mut inv, &bow_item());
        }
        // No room for another bow.
        assert!(!can_fit_item_in_inventory(&inv, &bow_item()));
        // But if we ignore one of the existing items, there's room.
        assert!(could_fit_item_in_inventory(&inv, &bow_item(), Some(0)));
    }

    #[test]
    fn test_check_overlapping_items_empty() {
        let inv = InventoryData::new();
        // Slot 7 (inv index 0), 1x1, empty grid ⇒ 0.
        assert_eq!(check_overlapping_items(&inv, SLOTXY_INV_FIRST, Size::new(1, 1)), 0);
    }

    #[test]
    fn test_check_overlapping_items_single() {
        let mut inv = InventoryData::new();
        auto_place_item_in_inventory(&mut inv, &potion_item());
        // A 1x1 potion lands at grid index 30 (last row, col 0). The
        // corresponding slot index is SLOTXY_INV_FIRST + 30 = 37.
        let s = SLOTXY_INV_FIRST + 30;
        assert_eq!(check_overlapping_items(&inv, s, Size::new(1, 1)), 1);
    }
}
