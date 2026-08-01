//! Auto-pickup QoL feature (C++ `Source/qol/autopickup.cpp`).
//!
//! Ports the `DoPickup` predicate against the Rust `Item` model. The
//! inventory-dependent inputs (`has_room_for_gold`, `can_fit_in_inventory_or_belt`,
//! misc-item counts) are supplied by the caller so the decision logic stays a
//! pure, testable function; wiring them to `inventory.rs` is a follow-up.

use crate::game::items::{Item, ItemMiscId, ItemType};

/// Max gold a single pile can hold (C++ `MaxGold`).
pub const MAX_GOLD: i32 = 5000;

/// C++ `Gameplay` auto-pickup options (options.cpp).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoPickupOptions {
    pub auto_gold_pickup: bool,
    pub auto_elixir_pickup: bool,
    pub auto_oil_pickup: bool,
    pub auto_pickup_in_town: bool,
    pub num_heal_potion_pickup: i32,
    pub num_full_heal_potion_pickup: i32,
    pub num_mana_potion_pickup: i32,
    pub num_full_mana_potion_pickup: i32,
    pub num_reju_potion_pickup: i32,
    pub num_full_reju_potion_pickup: i32,
}

impl Default for AutoPickupOptions {
    fn default() -> Self {
        Self {
            auto_gold_pickup: true,
            auto_elixir_pickup: false,
            auto_oil_pickup: false,
            auto_pickup_in_town: true,
            num_heal_potion_pickup: 1,
            num_full_heal_potion_pickup: 1,
            num_mana_potion_pickup: 1,
            num_full_mana_potion_pickup: 1,
            num_reju_potion_pickup: 1,
            num_full_reju_potion_pickup: 1,
        }
    }
}

impl From<&crate::utils::options::GameplayOptions> for AutoPickupOptions {
    /// Build the pickup options from the live engine options (C++ options.cpp).
    fn from(o: &crate::utils::options::GameplayOptions) -> Self {
        Self {
            auto_gold_pickup: o.auto_gold_pickup,
            auto_elixir_pickup: o.auto_elixir_pickup,
            auto_oil_pickup: o.auto_oil_pickup,
            auto_pickup_in_town: o.auto_pickup_in_town,
            num_heal_potion_pickup: o.num_heal_potion_pickup,
            num_full_heal_potion_pickup: o.num_full_heal_potion_pickup,
            num_mana_potion_pickup: o.num_mana_potion_pickup,
            num_full_mana_potion_pickup: o.num_full_mana_potion_pickup,
            num_reju_potion_pickup: o.num_reju_potion_pickup,
            num_full_reju_potion_pickup: o.num_full_reju_potion_pickup,
        }
    }
}

/// C++ `DoPickup(item)` — decide whether the item is auto-picked.
///
/// `has_room_for_gold` mirrors C++ `HasRoomForGold()`, `can_fit` mirrors
/// `CanFitItemInInventory || AutoPlaceItemInBelt`, and `count_of_misc`
/// mirrors `NumMiscItemsInInv(miscId)`.
pub fn should_autopickup(
    item: &Item,
    options: &AutoPickupOptions,
    has_room_for_gold: bool,
    can_fit: bool,
    count_of_misc: impl Fn(ItemMiscId) -> usize,
) -> bool {
    if item.item_type == ItemType::Gold {
        return options.auto_gold_pickup && has_room_for_gold && item.value < MAX_GOLD;
    }
    if !can_fit {
        return false;
    }
    match item.misc_id {
        ItemMiscId::Heal => options.num_heal_potion_pickup as usize > count_of_misc(ItemMiscId::Heal),
        ItemMiscId::FullHeal => {
            options.num_full_heal_potion_pickup as usize > count_of_misc(ItemMiscId::FullHeal)
        }
        ItemMiscId::Mana => options.num_mana_potion_pickup as usize > count_of_misc(ItemMiscId::Mana),
        ItemMiscId::FullMana => {
            options.num_full_mana_potion_pickup as usize > count_of_misc(ItemMiscId::FullMana)
        }
        ItemMiscId::Rejuv => options.num_reju_potion_pickup as usize > count_of_misc(ItemMiscId::Rejuv),
        ItemMiscId::FullRejuv => {
            options.num_full_reju_potion_pickup as usize > count_of_misc(ItemMiscId::FullRejuv)
        }
        ItemMiscId::ElixStr | ItemMiscId::ElixMag | ItemMiscId::ElixDex | ItemMiscId::ElixVit => {
            options.auto_elixir_pickup
        }
        ItemMiscId::Oil => options.auto_oil_pickup,
        _ => false,
    }
}

/// C++ `HasRoomForGold()` (autopickup.cpp): true when the inventory has an
/// empty cell or a gold pile with room.
pub fn has_room_for_gold(inventory: &crate::game::inventory::Inventory) -> bool {
    for &idx in &inventory.grid {
        if idx < 0 {
            continue; // continuation cell
        }
        if idx == 0 {
            return true; // empty cell
        }
        let item = inventory
            .items
            .get(idx as usize - 1)
            .and_then(|cell| cell.as_ref());
        if let Some(item) = item {
            if item.item_type == ItemType::Gold && item.value < MAX_GOLD {
                return true;
            }
        }
    }
    false
}

/// C++ `NumMiscItemsInInv(iMiscId)` — count matching misc items in the
/// inventory and belt.
pub fn num_misc_items_in_inv(
    inventory: &crate::game::inventory::Inventory,
    belt: &crate::game::inventory::Belt,
    misc_id: ItemMiscId,
) -> usize {
    let inv_count = inventory
        .items
        .iter()
        .filter(|cell| cell.as_ref().map(|i| i.misc_id == misc_id).unwrap_or(false))
        .count();
    let belt_count = belt
        .items
        .iter()
        .filter(|cell| cell.as_ref().map(|i| i.misc_id == misc_id).unwrap_or(false))
        .count();
    inv_count + belt_count
}

/// Simplified C++ `CanFitItemInInventory || AutoPlaceItemInBelt`: room in the
/// inventory grid or an empty belt slot.
pub fn can_fit_item(
    inventory: &crate::game::inventory::Inventory,
    belt: &crate::game::inventory::Belt,
) -> bool {
    !inventory.is_full() || belt.items.iter().any(Option::is_none)
}

/// C++ `AutoPickup(player)` decision for a single item, wired to the
/// inventory/belt state. `in_town` gates the whole feature: C++ skips
/// autopickup on town levels unless `autoPickupInTown` is enabled
/// (autopickup.cpp `AutoPickup`).
pub fn try_autopickup(
    item: &Item,
    inventory: &crate::game::inventory::Inventory,
    belt: &crate::game::inventory::Belt,
    options: &AutoPickupOptions,
    in_town: bool,
) -> bool {
    if in_town && !options.auto_pickup_in_town {
        return false;
    }
    should_autopickup(
        item,
        options,
        has_room_for_gold(inventory),
        can_fit_item(inventory, belt),
        |m| num_misc_items_in_inv(inventory, belt, m),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::items::{Item, ItemMiscId, ItemType};

    fn gold_item(value: i32) -> Item {
        Item {
            item_type: ItemType::Gold,
            value,
            ..Default::default()
        }
    }

    fn misc_item(misc_id: ItemMiscId) -> Item {
        Item {
            item_type: ItemType::HealthPotion,
            misc_id,
            ..Default::default()
        }
    }

    #[test]
    fn test_gold_pickup_rules() {
        let opts = AutoPickupOptions::default();
        // Room + below max -> pick up.
        assert!(should_autopickup(&gold_item(100), &opts, true, false, |_| 0));
        // No room -> no.
        assert!(!should_autopickup(&gold_item(100), &opts, false, false, |_| 0));
        // Full pile -> no.
        assert!(!should_autopickup(&gold_item(MAX_GOLD), &opts, true, false, |_| 0));
        // Disabled option -> no.
        let mut off = opts;
        off.auto_gold_pickup = false;
        assert!(!should_autopickup(&gold_item(100), &off, true, false, |_| 0));
    }

    /// C++ `AutoPickup` town gate: in town, autopickup only runs when the
    /// `autoPickupInTown` option is on (autopickup.cpp).
    #[test]
    fn test_town_gate_matches_cpp() {
        let inv = crate::game::inventory::Inventory::default();
        let belt = crate::game::inventory::Belt::default();
        let mut opts = AutoPickupOptions::default();
        opts.auto_pickup_in_town = false;
        // In town with autoPickupInTown off → nothing is picked up.
        assert!(!try_autopickup(&gold_item(100), &inv, &belt, &opts, true));
        assert!(!try_autopickup(&misc_item(ItemMiscId::Heal), &inv, &belt, &opts, true));
        // Same state in the dungeon → pickup proceeds.
        assert!(try_autopickup(&gold_item(100), &inv, &belt, &opts, false));
        // In town with autoPickupInTown on → pickup proceeds.
        opts.auto_pickup_in_town = true;
        assert!(try_autopickup(&gold_item(100), &inv, &belt, &opts, true));
    }

    #[test]
    fn test_potion_count_threshold() {
        let opts = AutoPickupOptions::default();
        // 1 heal potion configured; 0 carried -> pick up.
        assert!(should_autopickup(&misc_item(ItemMiscId::Heal), &opts, false, true, |m| {
            if m == ItemMiscId::Heal { 0 } else { 0 }
        }));
        // Already carrying the configured number -> no.
        assert!(!should_autopickup(&misc_item(ItemMiscId::Heal), &opts, false, true, |m| {
            if m == ItemMiscId::Heal { 1 } else { 0 }
        }));
        // No room in inventory/belt -> no.
        assert!(!should_autopickup(&misc_item(ItemMiscId::Mana), &opts, false, false, |_| 0));
    }

    #[test]
    fn test_elixir_and_oil_flags() {
        let mut opts = AutoPickupOptions::default();
        opts.auto_elixir_pickup = true;
        opts.auto_oil_pickup = true;
        assert!(should_autopickup(&misc_item(ItemMiscId::ElixStr), &opts, false, true, |_| 0));
        assert!(should_autopickup(&misc_item(ItemMiscId::Oil), &opts, false, true, |_| 0));

        opts.auto_elixir_pickup = false;
        assert!(!should_autopickup(&misc_item(ItemMiscId::ElixVit), &opts, false, true, |_| 0));
    }

    #[test]
    fn test_non_pickup_misc_rejected() {
        let opts = AutoPickupOptions::default();
        assert!(!should_autopickup(&misc_item(ItemMiscId::Scroll), &opts, false, true, |_| 0));
        assert!(!should_autopickup(&misc_item(ItemMiscId::None), &opts, false, true, |_| 0));
    }
    #[test]
    fn test_has_room_for_gold_empty_cell() {
        use crate::game::inventory::Inventory;
        let inv = Inventory::new();
        assert!(has_room_for_gold(&inv), "empty inventory has room");
    }

    #[test]
    fn test_has_room_for_gold_partial_pile() {
        use crate::game::inventory::{Inventory, INVENTORY_SIZE};
        // Fill every cell with a full gold pile: no room anywhere.
        let mut inv = Inventory::new();
        for i in 0..INVENTORY_SIZE {
            inv.items.push(Some(gold_item(MAX_GOLD)));
            inv.grid[i] = (i + 1) as i8;
        }
        assert!(!has_room_for_gold(&inv), "full pile + no empty cell");
        // A partial pile leaves room.
        inv.items[0] = Some(gold_item(100));
        assert!(has_room_for_gold(&inv));
    }

    #[test]
    fn test_num_misc_items_counts_inventory_and_belt() {
        use crate::game::inventory::{Belt, Inventory};
        let mut inv = Inventory::new();
        inv.items.push(Some(misc_item(ItemMiscId::Heal)));
        inv.grid[0] = 1;
        let mut belt = Belt::new();
        belt.items[0] = Some(misc_item(ItemMiscId::Heal));
        belt.items[1] = Some(misc_item(ItemMiscId::Mana));
        assert_eq!(num_misc_items_in_inv(&inv, &belt, ItemMiscId::Heal), 2);
        assert_eq!(num_misc_items_in_inv(&inv, &belt, ItemMiscId::Mana), 1);
        assert_eq!(num_misc_items_in_inv(&inv, &belt, ItemMiscId::Rejuv), 0);
    }

    #[test]
    fn test_try_autopickup_wired() {
        use crate::game::inventory::{Belt, Inventory};
        let opts = AutoPickupOptions::default();
        // Empty inventory -> gold picked.
        let inv2 = Inventory::new();
        let belt = Belt::new();
        assert!(try_autopickup(&gold_item(50), &inv2, &belt, &opts, false));
        // Potion threshold wired through inventory counts.
        let mut inv3 = Inventory::new();
        inv3.items.push(Some(misc_item(ItemMiscId::Heal)));
        inv3.grid[0] = 1;
        let mut belt3 = Belt::new();
        belt3.items[0] = Some(misc_item(ItemMiscId::Heal));
        // Carrying 2 with threshold 1 -> no pickup.
        assert!(!try_autopickup(&misc_item(ItemMiscId::Heal), &inv3, &belt3, &opts, false));
    }


    #[test]
    fn test_autopickup_from_gameplay_options() {
        use crate::utils::options::GameplayOptions;
        let mut opts = GameplayOptions::default();
        opts.auto_gold_pickup = true;
        opts.auto_oil_pickup = true;
        opts.num_mana_potion_pickup = 4;
        let ap: AutoPickupOptions = (&opts).into();
        assert!(ap.auto_gold_pickup);
        assert!(ap.auto_oil_pickup);
        assert_eq!(ap.num_mana_potion_pickup, 4);
    }


}
