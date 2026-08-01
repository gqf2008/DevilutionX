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
}
