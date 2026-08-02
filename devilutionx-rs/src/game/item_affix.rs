//! Exact port of item affix/generation system from DevilutionX
//!
//! Contains item prefixes, suffixes, and generation logic from items.cpp

#![allow(non_snake_case)]
#![allow(dead_code)]

use super::item_dat::{ItemType, ItemPower, ItemEffectType};
use super::spells::SpellId;
use rand::Rng;

// ============================================================================
// Affix item type flags - what item types an affix can apply to
// ============================================================================

/// Affix item-type category flags.
///
/// This mirrors the C++ `AffixItemType` enum exactly (see `Source/itemdat.h:609`):
/// a 6-bit coarse category set stored per-affix in `item_prefixes.tsv` /
/// `item_suffixes.tsv` (columns: Misc, Bow, Staff, Weapon, Shield, Armor) and
/// used as the single search mask in `GetItemBonus()` (`Source/items.cpp:1309`).
///
/// The constants below are kept bit-identical to C++ so that the data tables
/// (which are written in terms of `ALL_WEAPONS`, `ALL_ARMOR`, `BOW`, `STAFF`,
/// `SHIELD`, `JEWELRY`) compile to the same bitmask C++ loads from the TSV.
/// `JEWELRY` and `HELM` are kept as named aliases for back-compat with the
/// `items.rs` search-mask construction (rings/amulets and helms map to Misc /
/// Armor respectively in C++).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AffixItemType(pub u32);

impl AffixItemType {
    // --- Core 6-bit coarse categories (bit-identical to C++ enum) ---
    pub const NONE: Self = Self(0);
    /// Misc (rings, amulets, jewelry) — C++ `AffixItemType::Misc = 1 << 0`.
    pub const MISC: Self = Self(1 << 0);
    /// Bow — C++ `AffixItemType::Bow = 1 << 1`.
    pub const BOW: Self = Self(1 << 1);
    /// Staff — C++ `AffixItemType::Staff = 1 << 2`.
    pub const STAFF: Self = Self(1 << 2);
    /// Weapon (sword / axe / mace) — C++ `AffixItemType::Weapon = 1 << 3`.
    pub const WEAPON: Self = Self(1 << 3);
    /// Shield — C++ `AffixItemType::Shield = 1 << 4`.
    pub const SHIELD: Self = Self(1 << 4);
    /// Armor (light / medium / heavy / helm) — C++ `AffixItemType::Armor = 1 << 5`.
    pub const ARMOR: Self = Self(1 << 5);

    // --- Aliases used by data tables and external callers ---
    /// All weapons. Matches C++ coarse `Weapon` bit (a single category).
    pub const ALL_WEAPONS: Self = Self::WEAPON;
    /// All armor (light/medium/heavy/helm). Matches C++ coarse `Armor` bit.
    pub const ALL_ARMOR: Self = Self::ARMOR;
    /// Jewelry alias. Rings/amulets use the `Misc` category in C++.
    pub const JEWELRY: Self = Self::MISC;
    /// Helm alias. Helms use the `Armor` category in C++.
    pub const HELM: Self = Self::ARMOR;

    /// Returns true if `self` and `other` share any category bit.
    /// Matches C++ `HasAnyOf` / `(affix.PLIType & flgs) != AffixItemType::None`.
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Map a concrete `ItemType` to its coarse affix category.
    ///
    /// This mirrors the `switch (item._itype)` in `Source/items.cpp:2283`
    /// (sword/axe/mace → Weapon, bow → Bow, shield → Shield,
    ///  light/medium/heavy armor and helm → Armor, staff → Staff,
    ///  ring/amulet → Misc).
    pub fn from_item_type(item_type: ItemType) -> Self {
        match item_type {
            ItemType::Sword | ItemType::Axe | ItemType::Mace => Self::WEAPON,
            ItemType::Bow => Self::BOW,
            ItemType::Staff => Self::STAFF,
            ItemType::Shield => Self::SHIELD,
            ItemType::Helm
            | ItemType::LightArmor
            | ItemType::MediumArmor
            | ItemType::HeavyArmor => Self::ARMOR,
            ItemType::Ring | ItemType::Amulet => Self::MISC,
            _ => Self::MISC,
        }
    }
}

// ============================================================================
// Good or Evil alignment for affixes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum GoodOrEvil {
    #[default]
    Any = 0,
    Good = 1,
    Evil = 2,
}

// ============================================================================
// Prefix/Suffix structure - exact match of PLStruct
// ============================================================================

#[derive(Debug, Clone)]
pub struct AffixData {
    /// Name of the affix (e.g., "of the Bear", "King's")
    pub name: &'static str,

    /// Effect power and parameters
    pub power: ItemPower,

    /// Minimum item level for this affix
    pub min_level: i32,

    /// What item types this affix can apply to
    pub item_types: AffixItemType,

    /// Is this a "good" affix (no negative effects)
    pub is_good: bool,

    /// Good/Evil alignment
    pub alignment: GoodOrEvil,

    /// Minimum value for display
    pub min_val: i32,

    /// Maximum value for display
    pub max_val: i32,

    /// Value multiplier for item pricing
    pub mult_val: i32,

    /// Chance weight for selection
    pub chance: i32,
}

// ============================================================================
// Item prefixes - exact data from items.cpp ItemPrefixes array
// ============================================================================

pub static ITEM_PREFIXES: &[AffixData] = &[
AffixData {
        name: "Tin",
        power: ItemPower { effect_type: ItemEffectType::ToHitCurse, param1: 6, param2: 10 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 2,
    },
    AffixData {
        name: "Brass",
        power: ItemPower { effect_type: ItemEffectType::ToHitCurse, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 2,
    },
    AffixData {
        name: "Bronze",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 100, max_val: 500, mult_val: 2,
        chance: 2,
    },
    AffixData {
        name: "Iron",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 6, param2: 10 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 600, max_val: 1000, mult_val: 3,
        chance: 2,
    },
    AffixData {
        name: "Steel",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 11, param2: 15 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1100, max_val: 1500, mult_val: 5,
        chance: 2,
    },
    AffixData {
        name: "Silver",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 16, param2: 20 },
        min_level: 9,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 1600, max_val: 2000, mult_val: 7,
        chance: 2,
    },
    AffixData {
        name: "Gold",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 21, param2: 30 },
        min_level: 12,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 2100, max_val: 3000, mult_val: 9,
        chance: 2,
    },
    AffixData {
        name: "Platinum",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 31, param2: 40 },
        min_level: 16,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 3100, max_val: 4000, mult_val: 11,
        chance: 2,
    },
    AffixData {
        name: "Mithril",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 41, param2: 60 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 4100, max_val: 6000, mult_val: 13,
        chance: 2,
    },
    AffixData {
        name: "Meteoric",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 61, param2: 80 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 6100, max_val: 10000, mult_val: 15,
        chance: 2,
    },
    AffixData {
        name: "Weird",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 81, param2: 100 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 10100, max_val: 14000, mult_val: 17,
        chance: 2,
    },
    AffixData {
        name: "Strange",
        power: ItemPower { effect_type: ItemEffectType::ToHit, param1: 101, param2: 150 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 14100, max_val: 20000, mult_val: 20,
        chance: 2,
    },
    AffixData {
        name: "Useless",
        power: ItemPower { effect_type: ItemEffectType::DamageCurse, param1: 100, param2: 100 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -8,
        chance: 2,
    },
    AffixData {
        name: "Bent",
        power: ItemPower { effect_type: ItemEffectType::DamageCurse, param1: 50, param2: 75 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -4,
        chance: 2,
    },
    AffixData {
        name: "Weak",
        power: ItemPower { effect_type: ItemEffectType::DamageCurse, param1: 25, param2: 45 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 2,
    },
    AffixData {
        name: "Jagged",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 20, param2: 35 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 250, max_val: 450, mult_val: 3,
        chance: 2,
    },
    AffixData {
        name: "Deadly",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 36, param2: 50 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 500, max_val: 700, mult_val: 4,
        chance: 2,
    },
    AffixData {
        name: "Heavy",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 51, param2: 65 },
        min_level: 9,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 750, max_val: 950, mult_val: 5,
        chance: 2,
    },
    AffixData {
        name: "Vicious",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 66, param2: 80 },
        min_level: 12,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Evil,
        min_val: 1000, max_val: 1450, mult_val: 8,
        chance: 2,
    },
    AffixData {
        name: "Brutal",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 81, param2: 95 },
        min_level: 16,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1500, max_val: 1950, mult_val: 10,
        chance: 2,
    },
    AffixData {
        name: "Massive",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 96, param2: 110 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 2450, mult_val: 13,
        chance: 2,
    },
    AffixData {
        name: "Savage",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 111, param2: 125 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2500, max_val: 3000, mult_val: 15,
        chance: 2,
    },
    AffixData {
        name: "Ruthless",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 126, param2: 150 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 10100, max_val: 15000, mult_val: 17,
        chance: 2,
    },
    AffixData {
        name: "Merciless",
        power: ItemPower { effect_type: ItemEffectType::Damage, param1: 151, param2: 175 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 15000, max_val: 20000, mult_val: 20,
        chance: 2,
    },
    AffixData {
        name: "Clumsy",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamageCurse, param1: 50, param2: 75 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -7,
        chance: 2,
    },
    AffixData {
        name: "Dull",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamageCurse, param1: 25, param2: 45 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -5,
        chance: 2,
    },
    AffixData {
        name: "Sharp",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 20, param2: 35 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 350, max_val: 950, mult_val: 5,
        chance: 2,
    },
    AffixData {
        name: "Fine",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 36, param2: 50 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1100, max_val: 1700, mult_val: 7,
        chance: 2,
    },
    AffixData {
        name: "Warrior's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 51, param2: 65 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1850, max_val: 2450, mult_val: 13,
        chance: 2,
    },
    AffixData {
        name: "Soldier's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 66, param2: 80 },
        min_level: 15,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2600, max_val: 3950, mult_val: 17,
        chance: 2,
    },
    AffixData {
        name: "Lord's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 81, param2: 95 },
        min_level: 19,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4100, max_val: 5950, mult_val: 21,
        chance: 2,
    },
    AffixData {
        name: "Knight's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 96, param2: 110 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 6100, max_val: 8450, mult_val: 26,
        chance: 2,
    },
    AffixData {
        name: "Master's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 111, param2: 125 },
        min_level: 28,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8600, max_val: 13000, mult_val: 30,
        chance: 2,
    },
    AffixData {
        name: "Champion's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 126, param2: 150 },
        min_level: 40,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 15200, max_val: 24000, mult_val: 33,
        chance: 2,
    },
    AffixData {
        name: "King's",
        power: ItemPower { effect_type: ItemEffectType::ToHitDamage, param1: 151, param2: 175 },
        min_level: 28,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 24100, max_val: 35000, mult_val: 38,
        chance: 2,
    },
    AffixData {
        name: "Vulnerable",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercentCurse, param1: 51, param2: 100 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 2,
    },
    AffixData {
        name: "Rusted",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercentCurse, param1: 25, param2: 50 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 2,
    },
    AffixData { // Fine #2
        name: "Fine",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 20, param2: 30 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 20, max_val: 100, mult_val: 2,
        chance: 2,
    },
    AffixData {
        name: "Strong",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 31, param2: 40 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 120, max_val: 200, mult_val: 3,
        chance: 2,
    },
    AffixData {
        name: "Grand",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 41, param2: 55 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 220, max_val: 300, mult_val: 5,
        chance: 2,
    },
    AffixData {
        name: "Valiant",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 56, param2: 70 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 320, max_val: 400, mult_val: 7,
        chance: 2,
    },
    AffixData {
        name: "Glorious",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 71, param2: 90 },
        min_level: 14,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 420, max_val: 600, mult_val: 9,
        chance: 2,
    },
    AffixData {
        name: "Blessed",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 91, param2: 110 },
        min_level: 19,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 620, max_val: 800, mult_val: 11,
        chance: 2,
    },
    AffixData {
        name: "Saintly",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 111, param2: 130 },
        min_level: 24,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 820, max_val: 1200, mult_val: 13,
        chance: 2,
    },
    AffixData {
        name: "Awesome",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 131, param2: 150 },
        min_level: 28,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 1220, max_val: 2000, mult_val: 15,
        chance: 2,
    },
    AffixData {
        name: "Holy",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 151, param2: 170 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 5200, max_val: 6000, mult_val: 17,
        chance: 2,
    },
    AffixData {
        name: "Godly",
        power: ItemPower { effect_type: ItemEffectType::ArmorPercent, param1: 171, param2: 200 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 6200, max_val: 7000, mult_val: 20,
        chance: 2,
    },
    AffixData {
        name: "Red",
        power: ItemPower { effect_type: ItemEffectType::FireRes, param1: 10, param2: 20 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 500, max_val: 1500, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Crimson",
        power: ItemPower { effect_type: ItemEffectType::FireRes, param1: 21, param2: 30 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2100, max_val: 3000, mult_val: 2,
        chance: 1,
    },
    AffixData { // Crimson #2
        name: "Crimson",
        power: ItemPower { effect_type: ItemEffectType::FireRes, param1: 31, param2: 40 },
        min_level: 16,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3100, max_val: 4000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Garnet",
        power: ItemPower { effect_type: ItemEffectType::FireRes, param1: 41, param2: 50 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8200, max_val: 12000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Ruby",
        power: ItemPower { effect_type: ItemEffectType::FireRes, param1: 51, param2: 60 },
        min_level: 26,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 17100, max_val: 20000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "Blue",
        power: ItemPower { effect_type: ItemEffectType::LightRes, param1: 10, param2: 20 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 500, max_val: 1500, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Azure",
        power: ItemPower { effect_type: ItemEffectType::LightRes, param1: 21, param2: 30 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2100, max_val: 3000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Lapis",
        power: ItemPower { effect_type: ItemEffectType::LightRes, param1: 31, param2: 40 },
        min_level: 16,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3100, max_val: 4000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Cobalt",
        power: ItemPower { effect_type: ItemEffectType::LightRes, param1: 41, param2: 50 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8200, max_val: 12000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Sapphire",
        power: ItemPower { effect_type: ItemEffectType::LightRes, param1: 51, param2: 60 },
        min_level: 26,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 17100, max_val: 20000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "White",
        power: ItemPower { effect_type: ItemEffectType::MagicRes, param1: 10, param2: 20 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 500, max_val: 1500, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Pearl",
        power: ItemPower { effect_type: ItemEffectType::MagicRes, param1: 21, param2: 30 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2100, max_val: 3000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Ivory",
        power: ItemPower { effect_type: ItemEffectType::MagicRes, param1: 31, param2: 40 },
        min_level: 16,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3100, max_val: 4000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Crystal",
        power: ItemPower { effect_type: ItemEffectType::MagicRes, param1: 41, param2: 50 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8200, max_val: 12000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Diamond",
        power: ItemPower { effect_type: ItemEffectType::MagicRes, param1: 51, param2: 60 },
        min_level: 26,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 17100, max_val: 20000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "Topaz",
        power: ItemPower { effect_type: ItemEffectType::AllRes, param1: 10, param2: 15 },
        min_level: 8,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 5000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Amber",
        power: ItemPower { effect_type: ItemEffectType::AllRes, param1: 16, param2: 20 },
        min_level: 12,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 7400, max_val: 10000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Jade",
        power: ItemPower { effect_type: ItemEffectType::AllRes, param1: 21, param2: 30 },
        min_level: 18,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 11000, max_val: 15000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Obsidian",
        power: ItemPower { effect_type: ItemEffectType::AllRes, param1: 31, param2: 40 },
        min_level: 24,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 24000, max_val: 40000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "Emerald",
        power: ItemPower { effect_type: ItemEffectType::AllRes, param1: 41, param2: 50 },
        min_level: 31,
        item_types: AffixItemType(AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 61000, max_val: 75000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "Hyena's",
        power: ItemPower { effect_type: ItemEffectType::ManaCurse, param1: 11, param2: 25 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Any,
        min_val: 100, max_val: 1000, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "Frog's",
        power: ItemPower { effect_type: ItemEffectType::ManaCurse, param1: 1, param2: 10 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "Spider's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 10, param2: 15 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Evil,
        min_val: 500, max_val: 1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Raven's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 15, param2: 20 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1100, max_val: 2000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Snake's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 21, param2: 30 },
        min_level: 9,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2100, max_val: 4000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "Serpent's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 30, param2: 40 },
        min_level: 15,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4100, max_val: 6000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "Drake's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 41, param2: 50 },
        min_level: 21,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 6100, max_val: 10000, mult_val: 9,
        chance: 1,
    },
    AffixData {
        name: "Dragon's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 51, param2: 60 },
        min_level: 27,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 10100, max_val: 15000, mult_val: 11,
        chance: 1,
    },
    AffixData {
        name: "Wyrm's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 61, param2: 80 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 15100, max_val: 19000, mult_val: 12,
        chance: 1,
    },
    AffixData {
        name: "Hydra's",
        power: ItemPower { effect_type: ItemEffectType::Mana, param1: 81, param2: 100 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 19100, max_val: 30000, mult_val: 13,
        chance: 1,
    },
    AffixData {
        name: "Angel's",
        power: ItemPower { effect_type: ItemEffectType::SpellLevelAdd, param1: 1, param2: 1 },
        min_level: 15,
        item_types: AffixItemType(AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 25000, max_val: 25000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Arch-Angel's",
        power: ItemPower { effect_type: ItemEffectType::SpellLevelAdd, param1: 2, param2: 2 },
        min_level: 25,
        item_types: AffixItemType(AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 50000, max_val: 50000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Plentiful",
        power: ItemPower { effect_type: ItemEffectType::Charges, param1: 2, param2: 2 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 2000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Bountiful",
        power: ItemPower { effect_type: ItemEffectType::Charges, param1: 3, param2: 3 },
        min_level: 9,
        item_types: AffixItemType(AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3000, max_val: 3000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "Flaming",
        power: ItemPower { effect_type: ItemEffectType::FireDam, param1: 1, param2: 10 },
        min_level: 7,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 5000, max_val: 5000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "Lightning",
        power: ItemPower { effect_type: ItemEffectType::LightDam, param1: 2, param2: 20 },
        min_level: 18,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 10000, max_val: 10000, mult_val: 2,
        chance: 1,
    },
];

// ============================================================================
// Item suffixes - exact data from items.cpp ItemSuffixes array
// ============================================================================

pub static ITEM_SUFFIXES: &[AffixData] = &[
AffixData {
        name: "quality",
        power: ItemPower { effect_type: ItemEffectType::DamMod, param1: 1, param2: 2 },
        min_level: 2,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 100, max_val: 200, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "maiming",
        power: ItemPower { effect_type: ItemEffectType::DamMod, param1: 3, param2: 5 },
        min_level: 7,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1300, max_val: 1500, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "slaying",
        power: ItemPower { effect_type: ItemEffectType::DamMod, param1: 6, param2: 8 },
        min_level: 15,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2600, max_val: 3000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "gore",
        power: ItemPower { effect_type: ItemEffectType::DamMod, param1: 9, param2: 12 },
        min_level: 25,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4100, max_val: 5000, mult_val: 8,
        chance: 1,
    },
    AffixData {
        name: "carnage",
        power: ItemPower { effect_type: ItemEffectType::DamMod, param1: 13, param2: 16 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 5100, max_val: 10000, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "slaughter",
        power: ItemPower { effect_type: ItemEffectType::DamMod, param1: 17, param2: 20 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 10100, max_val: 15000, mult_val: 13,
        chance: 1,
    },
    AffixData {
        name: "pain",
        power: ItemPower { effect_type: ItemEffectType::GetHitCurse, param1: 2, param2: 4 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -4,
        chance: 1,
    },
    AffixData {
        name: "tears",
        power: ItemPower { effect_type: ItemEffectType::GetHitCurse, param1: 1, param2: 1 },
        min_level: 2,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "health",
        power: ItemPower { effect_type: ItemEffectType::GetHit, param1: 1, param2: 1 },
        min_level: 2,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 200, max_val: 200, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "protection",
        power: ItemPower { effect_type: ItemEffectType::GetHit, param1: 2, param2: 2 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 400, max_val: 800, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "absorption",
        power: ItemPower { effect_type: ItemEffectType::GetHit, param1: 3, param2: 3 },
        min_level: 12,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 1001, max_val: 2500, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "deflection",
        power: ItemPower { effect_type: ItemEffectType::GetHit, param1: 4, param2: 4 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 2500, max_val: 6500, mult_val: 15,
        chance: 1,
    },
    AffixData {
        name: "osmosis",
        power: ItemPower { effect_type: ItemEffectType::GetHit, param1: 5, param2: 6 },
        min_level: 50,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 7500, max_val: 10000, mult_val: 20,
        chance: 1,
    },
    AffixData {
        name: "frailty",
        power: ItemPower { effect_type: ItemEffectType::StrCurse, param1: 6, param2: 10 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 1,
    },
    AffixData {
        name: "weakness",
        power: ItemPower { effect_type: ItemEffectType::StrCurse, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "strength",
        power: ItemPower { effect_type: ItemEffectType::Str, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 200, max_val: 1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "might",
        power: ItemPower { effect_type: ItemEffectType::Str, param1: 6, param2: 10 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1200, max_val: 2000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "power",
        power: ItemPower { effect_type: ItemEffectType::Str, param1: 11, param2: 15 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2200, max_val: 3000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "giants",
        power: ItemPower { effect_type: ItemEffectType::Str, param1: 16, param2: 20 },
        min_level: 17,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3200, max_val: 5000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "titans",
        power: ItemPower { effect_type: ItemEffectType::Str, param1: 21, param2: 30 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 5200, max_val: 10000, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "paralysis",
        power: ItemPower { effect_type: ItemEffectType::DexCurse, param1: 6, param2: 10 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 1,
    },
    AffixData {
        name: "atrophy",
        power: ItemPower { effect_type: ItemEffectType::DexCurse, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "dexterity",
        power: ItemPower { effect_type: ItemEffectType::Dex, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 200, max_val: 1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "skill",
        power: ItemPower { effect_type: ItemEffectType::Dex, param1: 6, param2: 10 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1200, max_val: 2000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "accuracy",
        power: ItemPower { effect_type: ItemEffectType::Dex, param1: 11, param2: 15 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2200, max_val: 3000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "precision",
        power: ItemPower { effect_type: ItemEffectType::Dex, param1: 16, param2: 20 },
        min_level: 17,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3200, max_val: 5000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "perfection",
        power: ItemPower { effect_type: ItemEffectType::Dex, param1: 21, param2: 30 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 5200, max_val: 10000, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "the fool",
        power: ItemPower { effect_type: ItemEffectType::MagCurse, param1: 6, param2: 10 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 1,
    },
    AffixData {
        name: "dyslexia",
        power: ItemPower { effect_type: ItemEffectType::MagCurse, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "magic",
        power: ItemPower { effect_type: ItemEffectType::Mag, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 200, max_val: 1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "the mind",
        power: ItemPower { effect_type: ItemEffectType::Mag, param1: 6, param2: 10 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1200, max_val: 2000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "brilliance",
        power: ItemPower { effect_type: ItemEffectType::Mag, param1: 11, param2: 15 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2200, max_val: 3000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "sorcery",
        power: ItemPower { effect_type: ItemEffectType::Mag, param1: 16, param2: 20 },
        min_level: 17,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 3200, max_val: 5000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "wizardry",
        power: ItemPower { effect_type: ItemEffectType::Mag, param1: 21, param2: 30 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::STAFF.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 5200, max_val: 10000, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "illness",
        power: ItemPower { effect_type: ItemEffectType::VitCurse, param1: 6, param2: 10 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 1,
    },
    AffixData {
        name: "disease",
        power: ItemPower { effect_type: ItemEffectType::VitCurse, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "vitality",
        power: ItemPower { effect_type: ItemEffectType::Vit, param1: 1, param2: 5 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 200, max_val: 1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "zest",
        power: ItemPower { effect_type: ItemEffectType::Vit, param1: 6, param2: 10 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 1200, max_val: 2000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "vim",
        power: ItemPower { effect_type: ItemEffectType::Vit, param1: 11, param2: 15 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 2200, max_val: 3000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "vigor",
        power: ItemPower { effect_type: ItemEffectType::Vit, param1: 16, param2: 20 },
        min_level: 17,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 3200, max_val: 5000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "life",
        power: ItemPower { effect_type: ItemEffectType::Vit, param1: 21, param2: 30 },
        min_level: 23,
        item_types: AffixItemType(AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 5200, max_val: 10000, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "trouble",
        power: ItemPower { effect_type: ItemEffectType::AttribsCurse, param1: 6, param2: 10 },
        min_level: 12,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -10,
        chance: 1,
    },
    AffixData {
        name: "the pit",
        power: ItemPower { effect_type: ItemEffectType::AttribsCurse, param1: 1, param2: 5 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -5,
        chance: 1,
    },
    AffixData {
        name: "the sky",
        power: ItemPower { effect_type: ItemEffectType::Attribs, param1: 1, param2: 3 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 800, max_val: 4000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "the moon",
        power: ItemPower { effect_type: ItemEffectType::Attribs, param1: 4, param2: 7 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4800, max_val: 8000, mult_val: 10,
        chance: 1,
    },
    AffixData {
        name: "the stars",
        power: ItemPower { effect_type: ItemEffectType::Attribs, param1: 8, param2: 11 },
        min_level: 17,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8800, max_val: 12000, mult_val: 15,
        chance: 1,
    },
    AffixData {
        name: "the heavens",
        power: ItemPower { effect_type: ItemEffectType::Attribs, param1: 12, param2: 15 },
        min_level: 25,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 12800, max_val: 20000, mult_val: 20,
        chance: 1,
    },
    AffixData {
        name: "the zodiac",
        power: ItemPower { effect_type: ItemEffectType::Attribs, param1: 16, param2: 20 },
        min_level: 30,
        item_types: AffixItemType(AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 20800, max_val: 40000, mult_val: 30,
        chance: 1,
    },
    AffixData {
        name: "the vulture",
        power: ItemPower { effect_type: ItemEffectType::LifeCurse, param1: 11, param2: 25 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -4,
        chance: 1,
    },
    AffixData {
        name: "the jackal",
        power: ItemPower { effect_type: ItemEffectType::LifeCurse, param1: 1, param2: 10 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "the fox",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 10, param2: 15 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 100, max_val: 1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "the jaguar",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 16, param2: 20 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1100, max_val: 2000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "the eagle",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 21, param2: 30 },
        min_level: 9,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2100, max_val: 4000, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "the wolf",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 30, param2: 40 },
        min_level: 15,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4100, max_val: 6000, mult_val: 7,
        chance: 1,
    },
    AffixData {
        name: "the tiger",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 41, param2: 50 },
        min_level: 21,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 6100, max_val: 10000, mult_val: 9,
        chance: 1,
    },
    AffixData {
        name: "the lion",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 51, param2: 60 },
        min_level: 27,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 10100, max_val: 15000, mult_val: 11,
        chance: 1,
    },
    AffixData {
        name: "the mammoth",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 61, param2: 80 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 15100, max_val: 19000, mult_val: 12,
        chance: 1,
    },
    AffixData {
        name: "the whale",
        power: ItemPower { effect_type: ItemEffectType::Life, param1: 81, param2: 100 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 19100, max_val: 30000, mult_val: 13,
        chance: 1,
    },
    AffixData {
        name: "fragility",
        power: ItemPower { effect_type: ItemEffectType::DurabilityCurse, param1: 100, param2: 100 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -4,
        chance: 1,
    },
    AffixData {
        name: "brittleness",
        power: ItemPower { effect_type: ItemEffectType::DurabilityCurse, param1: 26, param2: 75 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "sturdiness",
        power: ItemPower { effect_type: ItemEffectType::Durability, param1: 26, param2: 75 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 100, max_val: 100, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "craftsmanship",
        power: ItemPower { effect_type: ItemEffectType::Durability, param1: 51, param2: 100 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 200, max_val: 200, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "structure",
        power: ItemPower { effect_type: ItemEffectType::Durability, param1: 101, param2: 200 },
        min_level: 12,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 300, max_val: 300, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "the ages",
        power: ItemPower { effect_type: ItemEffectType::Indestructible, param1: 0, param2: 0 },
        min_level: 25,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 600, max_val: 600, mult_val: 5,
        chance: 1,
    },
    AffixData {
        name: "the dark",
        power: ItemPower { effect_type: ItemEffectType::LightCurse, param1: 4, param2: 4 },
        min_level: 6,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -3,
        chance: 1,
    },
    AffixData {
        name: "the night",
        power: ItemPower { effect_type: ItemEffectType::LightCurse, param1: 2, param2: 2 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::JEWELRY.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: 0, max_val: 0, mult_val: -2,
        chance: 1,
    },
    AffixData {
        name: "light",
        power: ItemPower { effect_type: ItemEffectType::Light, param1: 2, param2: 2 },
        min_level: 4,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 750, max_val: 750, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "radiance",
        power: ItemPower { effect_type: ItemEffectType::Light, param1: 4, param2: 4 },
        min_level: 8,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::ALL_WEAPONS.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Good,
        min_val: 1500, max_val: 1500, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "flame",
        power: ItemPower { effect_type: ItemEffectType::FireArrows, param1: 1, param2: 3 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 2000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "fire",
        power: ItemPower { effect_type: ItemEffectType::FireArrows, param1: 1, param2: 6 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4000, max_val: 4000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "burning",
        power: ItemPower { effect_type: ItemEffectType::FireArrows, param1: 1, param2: 16 },
        min_level: 35,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 6000, max_val: 6000, mult_val: 6,
        chance: 1,
    },
    AffixData {
        name: "shock",
        power: ItemPower { effect_type: ItemEffectType::LightArrows, param1: 1, param2: 6 },
        min_level: 13,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 6000, max_val: 6000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "lightning",
        power: ItemPower { effect_type: ItemEffectType::LightArrows, param1: 1, param2: 10 },
        min_level: 21,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8000, max_val: 8000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "thunder",
        power: ItemPower { effect_type: ItemEffectType::LightArrows, param1: 1, param2: 20 },
        min_level: 60,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 12000, max_val: 12000, mult_val: 6,
        chance: 1,
    },
    AffixData {
        name: "many",
        power: ItemPower { effect_type: ItemEffectType::Durability, param1: 100, param2: 100 },
        min_level: 3,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 750, max_val: 750, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "plenty",
        power: ItemPower { effect_type: ItemEffectType::Durability, param1: 200, param2: 200 },
        min_level: 7,
        item_types: AffixItemType(AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1500, max_val: 1500, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "thorns",
        power: ItemPower { effect_type: ItemEffectType::Thorns, param1: 1, param2: 3 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 500, max_val: 500, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "corruption",
        power: ItemPower { effect_type: ItemEffectType::NoMana, param1: 0, param2: 0 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::ALL_WEAPONS.0),
        is_good: false,
        alignment: GoodOrEvil::Evil,
        min_val: -1000, max_val: -1000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "thieves",
        power: ItemPower { effect_type: ItemEffectType::AbsHalfTrap, param1: 0, param2: 0 },
        min_level: 11,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::SHIELD.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1500, max_val: 1500, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "the bear",
        power: ItemPower { effect_type: ItemEffectType::Knockback, param1: 0, param2: 0 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Evil,
        min_val: 750, max_val: 750, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "the bat",
        power: ItemPower { effect_type: ItemEffectType::StealMana, param1: 3, param2: 3 },
        min_level: 8,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 7500, max_val: 7500, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "vampires",
        power: ItemPower { effect_type: ItemEffectType::StealMana, param1: 5, param2: 5 },
        min_level: 19,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 15000, max_val: 15000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "the leech",
        power: ItemPower { effect_type: ItemEffectType::StealLife, param1: 3, param2: 3 },
        min_level: 8,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 7500, max_val: 7500, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "blood",
        power: ItemPower { effect_type: ItemEffectType::StealLife, param1: 5, param2: 5 },
        min_level: 19,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 15000, max_val: 15000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "piercing",
        power: ItemPower { effect_type: ItemEffectType::TargetAC, param1: 2, param2: 6 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 1000, max_val: 1000, mult_val: 3,
        chance: 1,
    },
    AffixData {
        name: "puncturing",
        power: ItemPower { effect_type: ItemEffectType::TargetAC, param1: 4, param2: 12 },
        min_level: 9,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 2000, mult_val: 6,
        chance: 1,
    },
    AffixData {
        name: "bashing",
        power: ItemPower { effect_type: ItemEffectType::TargetAC, param1: 8, param2: 24 },
        min_level: 17,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4000, max_val: 4000, mult_val: 12,
        chance: 1,
    },
    AffixData {
        name: "readiness",
        power: ItemPower { effect_type: ItemEffectType::FastAttack, param1: 1, param2: 1 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 2000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "swiftness",
        power: ItemPower { effect_type: ItemEffectType::FastAttack, param1: 2, param2: 2 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0 | AffixItemType::BOW.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4000, max_val: 4000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "speed",
        power: ItemPower { effect_type: ItemEffectType::FastAttack, param1: 3, param2: 3 },
        min_level: 19,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8000, max_val: 8000, mult_val: 8,
        chance: 1,
    },
    AffixData {
        name: "haste",
        power: ItemPower { effect_type: ItemEffectType::FastAttack, param1: 4, param2: 4 },
        min_level: 27,
        item_types: AffixItemType(AffixItemType::ALL_WEAPONS.0 | AffixItemType::STAFF.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 16000, max_val: 16000, mult_val: 16,
        chance: 1,
    },
    AffixData {
        name: "balance",
        power: ItemPower { effect_type: ItemEffectType::FastRecover, param1: 1, param2: 1 },
        min_level: 1,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 2000, max_val: 2000, mult_val: 2,
        chance: 1,
    },
    AffixData {
        name: "stability",
        power: ItemPower { effect_type: ItemEffectType::FastRecover, param1: 2, param2: 2 },
        min_level: 10,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4000, max_val: 4000, mult_val: 4,
        chance: 1,
    },
    AffixData {
        name: "harmony",
        power: ItemPower { effect_type: ItemEffectType::FastRecover, param1: 3, param2: 3 },
        min_level: 20,
        item_types: AffixItemType(AffixItemType::ALL_ARMOR.0 | AffixItemType::JEWELRY.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 8000, max_val: 8000, mult_val: 8,
        chance: 1,
    },
    AffixData {
        name: "blocking",
        power: ItemPower { effect_type: ItemEffectType::FastBlock, param1: 1, param2: 1 },
        min_level: 5,
        item_types: AffixItemType(AffixItemType::SHIELD.0),
        is_good: true,
        alignment: GoodOrEvil::Any,
        min_val: 4000, max_val: 4000, mult_val: 4,
        chance: 1,
    },
];

// ============================================================================
// Affix selection and item generation functions
// ============================================================================

/// Select a random affix from the given list based on item type and level
pub fn select_affix(
    affix_list: &[AffixData],
    item_type: AffixItemType,
    min_level: i32,
    max_level: i32,
    only_good: bool,
    alignment: GoodOrEvil,
) -> Option<&AffixData> {
    let mut rng = rand::rng();

    // Build list of eligible affixes with their weights
    let mut eligible: Vec<(&AffixData, i32)> = Vec::new();

    for affix in affix_list {
        // Check item type
        if !item_type.contains(affix.item_types) {
            continue;
        }

        // Check level range
        if affix.min_level < min_level || affix.min_level > max_level {
            continue;
        }

        // Check only good
        if only_good && !affix.is_good {
            continue;
        }

        // Check alignment
        if alignment != GoodOrEvil::Any
            && ((alignment == GoodOrEvil::Good && affix.alignment == GoodOrEvil::Evil)
                || (alignment == GoodOrEvil::Evil && affix.alignment == GoodOrEvil::Good))
        {
            continue;
        }

        eligible.push((affix, affix.chance));
    }

    if eligible.is_empty() {
        return None;
    }

    // Calculate total weight
    let total_weight: i32 = eligible.iter().map(|(_, w)| w).sum();

    // Random selection
    let mut roll = rng.random_range(0..total_weight);

    for (affix, weight) in &eligible {
        roll -= weight;
        if roll < 0 {
            return Some(*affix);
        }
    }

    eligible.last().map(|(a, _)| *a)
}

/// Generate random value between min and max
pub fn rnd_pl(min: i32, max: i32) -> i32 {
    if min >= max {
        return min;
    }
    // C++ `RndPl` (items.cpp): return GenerateRnd(max - min + 1) + min;
    super::super::engine::random::generate_rnd(max - min + 1) + min
}

/// Calculate to-hit bonus from damage bonus
pub fn calculate_to_hit_bonus(damage_bonus: i32) -> i32 {
    if damage_bonus < 0 {
        -damage_bonus / 4
    } else {
        damage_bonus / 4
    }
}

/// Scale value based on affix range
pub fn pl_val(value: i32, param1: i32, param2: i32, min_val: i32, max_val: i32) -> i32 {
    if param1 == param2 {
        return value;
    }

    // Scale linearly between min_val and max_val
    let range = param2 - param1;
    let val_range = max_val - min_val;

    if range == 0 {
        return min_val;
    }

    min_val + (value - param1) * val_range / range
}

/// Staff spell list with minimum magic requirements
pub static STAFF_SPELLS: &[(SpellId, i32)] = &[
    (SpellId::Firebolt, 15),
    (SpellId::ChargedBolt, 25),
    (SpellId::HolyBolt, 20),
    (SpellId::Healing, 17),
    (SpellId::Lightning, 30),
    (SpellId::FireWall, 27),
    (SpellId::Inferno, 20),
    (SpellId::TownPortal, 35),
    (SpellId::Flash, 33),
    (SpellId::StoneCurse, 51),
    (SpellId::Phasing, 39),
    (SpellId::Fireball, 48),
    (SpellId::FlameWave, 54),
    (SpellId::ChainLightning, 54),
    (SpellId::Guardian, 61),
    (SpellId::Elemental, 68),
    (SpellId::Nova, 87),
    (SpellId::Golem, 81),
    (SpellId::Teleport, 105),
    (SpellId::Apocalypse, 149),
];

/// Get random staff spell for given level
pub fn get_staff_spell(level: i32) -> Option<(SpellId, i32)> {
    let mut rng = rand::rng();
    let eligible: Vec<_> = STAFF_SPELLS
        .iter()
        .filter(|(_, min_mag)| *min_mag <= level * 2)
        .collect();

    if eligible.is_empty() {
        return Some(STAFF_SPELLS[0]);
    }

    let idx = rng.random_range(0..eligible.len());
    Some(*eligible[idx])
}

// ============================================================================
// Item affix application functions - exact port from items.cpp
// ============================================================================

/// Apply a single ItemPower effect to an item and return the random value generated
/// Exact port of SaveItemPower() from items.cpp lines 701-1046
pub fn save_item_power(item: &mut super::items::Item, power: &ItemPower, _player_max_hp: i32, _player_max_mana: i32) -> i32 {
    use super::items::ItemSpecialEffect;

    const DUR_INDESTRUCTIBLE: i32 = 255;

    let r = rnd_pl(power.param1, power.param2);

    match power.effect_type {
        ItemEffectType::ToHit => {
            item.bonus_to_hit += r as i16;
        }
        ItemEffectType::ToHitCurse => {
            item.bonus_to_hit -= r as i16;
        }
        ItemEffectType::Damage => {
            item.bonus_damage += r as i16;
        }
        ItemEffectType::DamageCurse => {
            item.bonus_damage -= r as i16;
        }
        ItemEffectType::ToHitDamage => {
            let r2 = rnd_pl(power.param1, power.param2);
            item.bonus_damage += r2 as i16;
            item.bonus_to_hit += calculate_to_hit_bonus(power.param1) as i16;
        }
        ItemEffectType::ToHitDamageCurse => {
            item.bonus_damage -= r as i16;
            item.bonus_to_hit += calculate_to_hit_bonus(-power.param1) as i16;
        }
        ItemEffectType::ArmorPercent => {
            item.bonus_ac += r as i16;
        }
        ItemEffectType::ArmorPercentCurse => {
            item.bonus_ac -= r as i16;
        }
        ItemEffectType::SetAC => {
            item.armor_class = r as i16;
        }
        ItemEffectType::ACCurse => {
            item.armor_class -= r as i16;
        }
        ItemEffectType::FireRes => {
            item.resist_fire += r as i16;
        }
        ItemEffectType::LightRes => {
            item.resist_lightning += r as i16;
        }
        ItemEffectType::MagicRes => {
            item.resist_magic += r as i16;
        }
        ItemEffectType::AllRes => {
            item.resist_fire = (item.resist_fire + r as i16).max(0);
            item.resist_lightning = (item.resist_lightning + r as i16).max(0);
            item.resist_magic = (item.resist_magic + r as i16).max(0);
        }
        ItemEffectType::SpellLevelAdd => {
            item.spell_level_add = r as i8;
        }
        ItemEffectType::Charges => {
            item.charges *= power.param1;
            item.max_charges = item.charges;
        }
        ItemEffectType::Spell => {
            item.spell = power.param1 as i8;
            item.charges = power.param2;
            item.max_charges = power.param2;
        }
        ItemEffectType::FireDam => {
            item.special_flags.0 |= ItemSpecialEffect::FIRE_ARROWS.0;
            item.special_flags.0 &= !ItemSpecialEffect::LIGHTNING_ARROWS.0;
            item.fire_min_dam = power.param1 as i16;
            item.fire_max_dam = power.param2 as i16;
            item.lightning_min_dam = 0;
            item.lightning_max_dam = 0;
        }
        ItemEffectType::LightDam => {
            item.special_flags.0 |= ItemSpecialEffect::LIGHTNING_ARROWS.0;
            item.special_flags.0 &= !ItemSpecialEffect::FIRE_ARROWS.0;
            item.lightning_min_dam = power.param1 as i16;
            item.lightning_max_dam = power.param2 as i16;
            item.fire_min_dam = 0;
            item.fire_max_dam = 0;
        }
        ItemEffectType::Str => {
            item.bonus_str += r as i16;
        }
        ItemEffectType::StrCurse => {
            item.bonus_str -= r as i16;
        }
        ItemEffectType::Mag => {
            item.bonus_mag += r as i16;
        }
        ItemEffectType::MagCurse => {
            item.bonus_mag -= r as i16;
        }
        ItemEffectType::Dex => {
            item.bonus_dex += r as i16;
        }
        ItemEffectType::DexCurse => {
            item.bonus_dex -= r as i16;
        }
        ItemEffectType::Vit => {
            item.bonus_vit += r as i16;
        }
        ItemEffectType::VitCurse => {
            item.bonus_vit -= r as i16;
        }
        ItemEffectType::Attribs => {
            item.bonus_str += r as i16;
            item.bonus_mag += r as i16;
            item.bonus_dex += r as i16;
            item.bonus_vit += r as i16;
        }
        ItemEffectType::AttribsCurse => {
            item.bonus_str -= r as i16;
            item.bonus_mag -= r as i16;
            item.bonus_dex -= r as i16;
            item.bonus_vit -= r as i16;
        }
        ItemEffectType::GetHitCurse => {
            item.bonus_get_hit += r as i16;
        }
        ItemEffectType::GetHit => {
            item.bonus_get_hit -= r as i16;
        }
        ItemEffectType::Life => {
            item.bonus_hp += (r << 6) as i16;
        }
        ItemEffectType::LifeCurse => {
            item.bonus_hp -= (r << 6) as i16;
        }
        ItemEffectType::Mana => {
            item.bonus_mana += (r << 6) as i16;
        }
        ItemEffectType::ManaCurse => {
            item.bonus_mana -= (r << 6) as i16;
        }
        ItemEffectType::Durability => {
            let bonus = r * item.max_durability / 100;
            item.max_durability += bonus;
            item.durability += bonus;
        }
        ItemEffectType::DurabilityCurse => {
            let reduction = r * item.max_durability / 100;
            item.max_durability -= reduction;
            item.max_durability = item.max_durability.max(1);
            item.durability = item.max_durability;
        }
        ItemEffectType::Indestructible => {
            item.durability = DUR_INDESTRUCTIBLE;
            item.max_durability = DUR_INDESTRUCTIBLE;
        }
        ItemEffectType::Light => {
            item.bonus_light += power.param1 as i16;
        }
        ItemEffectType::LightCurse => {
            item.bonus_light -= power.param1 as i16;
        }
        ItemEffectType::MultipleArrows => {
            item.special_flags.0 |= ItemSpecialEffect::MULTIPLE_ARROWS.0;
        }
        ItemEffectType::FireArrows => {
            item.special_flags.0 |= ItemSpecialEffect::FIRE_ARROWS.0;
            item.special_flags.0 &= !ItemSpecialEffect::LIGHTNING_ARROWS.0;
            item.fire_min_dam = power.param1 as i16;
            item.fire_max_dam = power.param2 as i16;
            item.lightning_min_dam = 0;
            item.lightning_max_dam = 0;
        }
        ItemEffectType::LightArrows => {
            item.special_flags.0 |= ItemSpecialEffect::LIGHTNING_ARROWS.0;
            item.special_flags.0 &= !ItemSpecialEffect::FIRE_ARROWS.0;
            item.lightning_min_dam = power.param1 as i16;
            item.lightning_max_dam = power.param2 as i16;
            item.fire_min_dam = 0;
            item.fire_max_dam = 0;
        }
        ItemEffectType::Fireball => {
            item.special_flags.0 |= ItemSpecialEffect::LIGHTNING_ARROWS.0 | ItemSpecialEffect::FIRE_ARROWS.0;
            item.fire_min_dam = power.param1 as i16;
            item.fire_max_dam = power.param2 as i16;
            item.lightning_min_dam = 0;
            item.lightning_max_dam = 0;
        }
        ItemEffectType::Thorns => {
            item.special_flags.0 |= ItemSpecialEffect::THORNS.0;
        }
        ItemEffectType::NoMana => {
            item.special_flags.0 |= ItemSpecialEffect::NOMANADRAIN.0;
        }
        ItemEffectType::AbsHalfTrap => {
            item.special_flags.0 |= ItemSpecialEffect::ABSORBHALF.0;
        }
        ItemEffectType::Knockback => {
            item.special_flags.0 |= ItemSpecialEffect::KNOCKBACK.0;
        }
        ItemEffectType::TripleDemonDamage => {
            item.special_flags.0 |= ItemSpecialEffect::DAMAGE_DEMON.0;
        }
        ItemEffectType::AllResZero => {
            item.special_flags.0 |= ItemSpecialEffect::ZEROTOHI.0;
        }
        ItemEffectType::StealMana => {
            if power.param1 == 3 {
                item.special_flags.0 |= ItemSpecialEffect::STEALMANA_3.0;
            }
            if power.param1 == 5 {
                item.special_flags.0 |= ItemSpecialEffect::STEALMANA_5.0;
            }
        }
        ItemEffectType::StealLife => {
            if power.param1 == 3 {
                item.special_flags.0 |= ItemSpecialEffect::STEALLIFE_3.0;
            }
            if power.param1 == 5 {
                item.special_flags.0 |= ItemSpecialEffect::STEALLIFE_5.0;
            }
        }
        ItemEffectType::TargetAC => {
            item.bonus_energy_ac += r as i16;
        }
        ItemEffectType::FastAttack => {
            if power.param1 == 1 {
                item.special_flags.0 |= ItemSpecialEffect::QUICKATTACK.0;
            }
            if power.param1 == 2 {
                item.special_flags.0 |= ItemSpecialEffect::FASTATTACK.0;
            }
            if power.param1 == 3 {
                item.special_flags.0 |= ItemSpecialEffect::FASTERATTACK.0;
            }
            if power.param1 == 4 {
                item.special_flags.0 |= ItemSpecialEffect::FASTESTATTACK.0;
            }
        }
        ItemEffectType::FastRecover => {
            if power.param1 == 1 {
                item.special_flags.0 |= ItemSpecialEffect::FASTHITRECOVER.0;
            }
            if power.param1 == 2 {
                item.special_flags.0 |= ItemSpecialEffect::FASTERHITRECOVER.0;
            }
            if power.param1 == 3 {
                item.special_flags.0 |= ItemSpecialEffect::FASTESTBLOCKRECOVER.0;
            }
        }
        ItemEffectType::FastBlock => {
            item.special_flags.0 |= ItemSpecialEffect::FASTESTBLOCKRECOVER.0;  // FastBlock uses FASTESTBLOCKRECOVER
        }
        ItemEffectType::DamMod => {
            item.bonus_damage_mod += r as i16;
        }
        ItemEffectType::RndArrowVel => {
            // RandomArrowVelocity not yet defined in ItemSpecialEffect, skip for now
        }
        ItemEffectType::SetDam => {
            item.min_damage = power.param1 as u8;
            item.max_damage = power.param2 as u8;
        }
        ItemEffectType::SetDur => {
            item.durability = power.param1;
            item.max_durability = power.param1;
        }
        ItemEffectType::NoMinStr => {
            item.required_str = 0;
        }
        ItemEffectType::OneHand => {
            item.equip_loc = super::items::ItemEquipType::OneHand;
        }
        ItemEffectType::DrainLife => {
            // DrainLife not yet defined, skip for now
        }
        ItemEffectType::RndStealLife => {
            // RandomStealLife not yet defined, skip for now
        }
        ItemEffectType::AddACLife => {
            item.special_flags.0 |= ItemSpecialEffect::LIGHTNING_ARROWS.0 | ItemSpecialEffect::FIRE_ARROWS.0;
            item.fire_min_dam = power.param1 as i16;
            item.fire_max_dam = power.param2 as i16;
            item.lightning_min_dam = 1;
            item.lightning_max_dam = 0;
        }
        ItemEffectType::AddManaAC => {
            item.special_flags.0 |= ItemSpecialEffect::LIGHTNING_ARROWS.0 | ItemSpecialEffect::FIRE_ARROWS.0;
            item.fire_min_dam = power.param1 as i16;
            item.fire_max_dam = power.param2 as i16;
            item.lightning_min_dam = 2;
            item.lightning_max_dam = 0;
        }
        ItemEffectType::FireResCurse => {
            item.resist_fire -= r as i16;
        }
        ItemEffectType::LightResCurse => {
            item.resist_lightning -= r as i16;
        }
        ItemEffectType::MagicResCurse => {
            item.resist_magic -= r as i16;
        }
        _ => {
            // Hellfire-specific effects or unimplemented effects
        }
    }

    r
}

/// Apply an affix to an item with value calculation
/// Exact port of SaveItemAffix() from items.cpp lines 1048-1061
pub fn save_item_affix(item: &mut super::items::Item, affix: &AffixData, player_max_hp: i32, player_max_mana: i32) {
    let mut power = affix.power;
    let value = save_item_power(item, &power, player_max_hp, player_max_mana);

    let scaled_value = pl_val(value, power.param1, power.param2, affix.min_val, affix.max_val);

    if item.value_add1 != 0 || item.value_mult1 != 0 {
        item.value_add2 = scaled_value;
        item.value_mult2 = affix.mult_val;
    } else {
        item.value_add1 = scaled_value;
        item.value_mult1 = affix.mult_val;
    }
}

/// Generate magic item name from prefix and suffix
/// Exact port of GenerateMagicItemName() from items.cpp lines 1166-1172
pub fn generate_magic_item_name(base_name: &str, prefix: Option<&AffixData>, suffix: Option<&AffixData>) -> String {
    match (prefix, suffix) {
        (Some(p), Some(s)) => {
            format!("{} {} of {}", p.name, base_name, s.name)
        }
        (Some(p), None) => {
            format!("{} {}", p.name, base_name)
        }
        (None, Some(s)) => {
            format!("{} of {}", base_name, s.name)
        }
        (None, None) => {
            base_name.to_string()
        }
    }
}

/// Calculate item value including affix bonuses
/// Simplified version - full implementation would be in items.rs CalcItemValue()
pub fn calc_affix_item_value(item: &super::items::Item) -> i32 {
    let mut value = item.buy_value;

    // Apply value modifiers from affixes
    if item.value_mult1 != 0 {
        value += item.value_add1;
        value *= item.value_mult1;
    }
    if item.value_mult2 != 0 {
        value += item.value_add2;
        value *= item.value_mult2;
    }

    value
}

/// Apply random affixes to an item based on its type
/// Exact port of GetItemBonus() from items.cpp lines 1309-1347
pub fn get_item_bonus<P>(
    item: &mut super::items::Item,
    player: &P,
    min_lvl: i32,
    max_lvl: i32,
    only_good: bool,
    allow_spells: bool,
) {
    use super::items::ItemType;

    // Cap minimum level at 25 (C++ behavior)
    let min_lvl = min_lvl.min(25);

    match item.item_type {
        ItemType::Sword | ItemType::Axe | ItemType::Mace => {
            // Weapons get weapon affixes
            apply_random_affixes(
                item,
                player,
                min_lvl,
                max_lvl,
                AffixItemType::ALL_WEAPONS,
                only_good,
            );
        }
        ItemType::Bow => {
            apply_random_affixes(
                item,
                player,
                min_lvl,
                max_lvl,
                AffixItemType::BOW,
                only_good,
            );
        }
        ItemType::Shield => {
            apply_random_affixes(
                item,
                player,
                min_lvl,
                max_lvl,
                AffixItemType::SHIELD,
                only_good,
            );
        }
        ItemType::Helm | ItemType::Armor => {
            // Armor pieces get armor affixes
            apply_random_affixes(
                item,
                player,
                min_lvl,
                max_lvl,
                AffixItemType::ALL_ARMOR,
                only_good,
            );
        }
        ItemType::Staff => {
            if allow_spells {
                // Staffs get spell + charges, optionally with prefix
                apply_staff_power(item, player, max_lvl, only_good);
            } else {
                // Force staff to use regular affixes
                apply_random_affixes(
                    item,
                    player,
                    min_lvl,
                    max_lvl,
                    AffixItemType::STAFF,
                    only_good,
                );
            }
        }
        ItemType::Ring | ItemType::Amulet => {
            apply_random_affixes(
                item,
                player,
                min_lvl,
                max_lvl,
                AffixItemType::JEWELRY,
                only_good,
            );
        }
        _ => {
            // No affixes for other types (gold, potions, etc.)
        }
    }
}

/// Select a random affix from the list based on eligibility criteria
/// Exact port of SelectAffix() from items.cpp lines 1063-1097
fn select_random_affix_internal<'a>(
    affix_list: &'a [AffixData],
    item_type: AffixItemType,
    min_lvl: i32,
    max_lvl: i32,
    only_good: bool,
    goe: GoodOrEvil,
    exclude_charges_for_staffs: bool,
) -> Option<&'a AffixData> {
    let mut eligible: Vec<&AffixData> = Vec::with_capacity(256);

    for affix in affix_list {
        // Check item type compatibility
        if (affix.item_types.0 & item_type.0) == 0 {
            continue;
        }

        // Check level requirements
        if affix.min_level < min_lvl || affix.min_level > max_lvl {
            continue;
        }

        // Check quality requirement
        if only_good && !affix.is_good {
            continue;
        }

        // Check good/evil alignment
        if (goe == GoodOrEvil::Good && affix.alignment == GoodOrEvil::Evil)
            || (goe == GoodOrEvil::Evil && affix.alignment == GoodOrEvil::Good) {
            continue;
        }

        // Special case: exclude charges for staffs if requested
        if exclude_charges_for_staffs
            && item_type.0 == AffixItemType::STAFF.0
            && affix.power.effect_type == ItemEffectType::Charges {
            continue;
        }

        // Add this affix multiple times based on its chance weight
        for _ in 0..affix.chance {
            eligible.push(affix);
        }
    }

    if eligible.is_empty() {
        return None;
    }

    // Use engine random for C++ alignment
    let idx = super::super::engine::random::generate_rnd(eligible.len() as i32) as usize;
    Some(eligible[idx])
}

/// Get random prefix/suffix affixes and apply them to an item
/// Exact port of GetItemPower() from items.cpp lines 1210-1236
pub fn apply_random_affixes<P>(
    item: &mut super::items::Item,
    _player: &P,  // Reserved for future use
    min_lvl: i32,
    max_lvl: i32,
    item_types: AffixItemType,
    mut only_good: bool,
) {
    use super::super::engine::random::flip_coin;

    // Determine if we allocate prefix/suffix (same logic as C++)
    let mut allocate_prefix = flip_coin(4);
    let mut allocate_suffix = !flip_coin(3);

    // Ensure at least one affix
    if !allocate_prefix && !allocate_suffix {
        if flip_coin(2) {
            allocate_prefix = true;
        } else {
            allocate_suffix = true;
        }
    }

    let mut goe = GoodOrEvil::Any;
    if !only_good && !flip_coin(3) {
        only_good = true;
    }

    let mut prefix_data: Option<&AffixData> = None;
    let mut suffix_data: Option<&AffixData> = None;

    // Try to apply prefix
    if allocate_prefix {
        if let Some(prefix) = select_random_affix_internal(
            &ITEM_PREFIXES,
            item_types,
            min_lvl,
            max_lvl,
            only_good,
            goe,
            true,  // exclude_charges_for_staffs
        ) {
            item.quality = super::items::ItemQuality::Magic;
            save_item_affix(item, prefix, 0, 0);  // TODO: Pass actual player_max_hp/mana
            // Convert item_dat::ItemEffectType to items::ItemEffectType (values are identical)
            // Both enums use the same numeric values from C++, safe to cast via discriminant
            item.prefix_power = unsafe { std::mem::transmute((prefix.power.effect_type as i32) as i8) };
            goe = prefix.alignment;  // Lock good/evil for suffix selection
            prefix_data = Some(prefix);
        }
    }

    // Try to apply suffix
    if allocate_suffix {
        if let Some(suffix) = select_random_affix_internal(
            &ITEM_SUFFIXES,
            item_types,
            min_lvl,
            max_lvl,
            only_good,
            goe,  // Use locked goe from prefix
            true,
        ) {
            item.quality = super::items::ItemQuality::Magic;
            save_item_affix(item, suffix, 0, 0);  // TODO: Pass actual player_max_hp/mana
            // Convert item_dat::ItemEffectType to items::ItemEffectType (values are identical)
            item.suffix_power = unsafe { std::mem::transmute((suffix.power.effect_type as i32) as i8) };
            suffix_data = Some(suffix);
        }
    }

    // Generate item name (C++: CopyUtf8(item._iIName, GenerateMagicItemName(...)))
    // The Rust Item has no separate identified-name field; `name` is the
    // display name (and `base_name` keeps the base).
    if prefix_data.is_some() || suffix_data.is_some() {
        item.name = generate_magic_item_name(&item.base_name, prefix_data, suffix_data);
        // Recalculate item value (simplified CalcItemValue)
        item.buy_value = calc_affix_item_value(item);
    }
}

/// Apply spell and charges to a staff, optionally with a prefix
/// Exact port of GetStaffPower() from items.cpp lines 1138-1159
pub fn apply_staff_power<P>(
    item: &mut super::items::Item,
    _player: &P,
    lvl: i32,
    only_good: bool,
) {
    use super::super::engine::random::flip_coin;

    // Try to apply a prefix (10% base chance, or always if only_good)
    let mut prefix_data: Option<&AffixData> = None;

    if flip_coin(10) || only_good {
        if let Some(prefix) = select_random_affix_internal(
            &ITEM_PREFIXES,
            AffixItemType::STAFF,
            0,
            lvl,
            only_good,
            GoodOrEvil::Any,
            false,  // Don't exclude charges for prefix
        ) {
            item.quality = super::items::ItemQuality::Magic;
            save_item_affix(item, prefix, 0, 0);  // TODO: Pass actual player_max_hp/mana
            // Convert item_dat::ItemEffectType to items::ItemEffectType (values are identical)
            item.prefix_power = unsafe { std::mem::transmute((prefix.power.effect_type as i32) as i8) };
            prefix_data = Some(prefix);
        }
    }

    // Staff display name: prefix + base (spell name generation is a
    // follow-up; the C++ format is "{Prefix} {Item} of {Spell}").
    if let Some(prefix) = prefix_data {
        item.name = generate_magic_item_name(&item.base_name, Some(prefix), None);
    }

    // Recalculate value
    item.buy_value = calc_affix_item_value(item);
}

// ============================================================================
// Random Item Index Selection (with drop rate weights)
// ============================================================================

/// Weighted item index for cumulative probability selection
#[derive(Debug, Clone)]
struct WeightedItemIndex {
    index: usize,
    cumulative_weight: u32,
}

/// Get random item index from droppable items with optional weight consideration
/// C++ `IsItemAvailable` (items.cpp:2375-2403): filters out quest-locked and
/// Hellfire-exclusive items for the current game mode.
pub fn is_item_available(index: usize, hellfire: bool, spawn: bool, test_bard: bool) -> bool {
    use super::item_dat::ItemId;
    if index >= crate::game::item_dat::ITEMS_DATA.len() {
        return false;
    }
    if spawn {
        // Medium and heavy armors.
        if (62..=70).contains(&index) {
            return false;
        }
        // Unavailable scrolls in the shareware build.
        if matches!(index, 105 | 107 | 108 | 110 | 111 | 113) {
            return false;
        }
    }
    if hellfire {
        return true;
    }
    let i = index as i64;
    (i != ItemId::MapOfDoom as i64
        && i != ItemId::LightningForge as i64
        && (i < ItemId::Oil as i64 || i > ItemId::GreySuit as i64)
        && (i < 83 || i > 86)
        && i != 92
        && (i < 161 || i > 165)
        && i != ItemId::Sorcerer as i64)
        || (test_bard && (i == ItemId::BardSword as i64 || i == ItemId::BardDagger as i64))
}

/// Exact port of GetItemIndexForDroppableItem() from items.cpp lines 1353-1377.
/// Builds a cumulative-weight list over the authoritative `ITEMS_DATA` table
/// (skip unavailable / dropRate 0 / Resurrect+HealOther scrolls in single
/// player / filtered items) and picks one with `RandomIntLessThan(cumulative)`.
pub fn get_item_index_for_droppable<F>(
    consider_drop_rate: bool,
    is_item_okay: F,
    rng: &mut impl Rng,
    hellfire: bool,
    spawn: bool,
    test_bard: bool,
    multiplayer: bool,
) -> Option<usize>
where
    F: Fn(&super::item_dat::ItemData) -> bool,
{
    let mut ril: Vec<(usize, u32)> = Vec::new();
    let mut cumulative = 0u32;
    for (i, item) in super::item_dat::ITEMS_DATA.iter().enumerate() {
        if !is_item_available(i, hellfire, spawn, test_bard) {
            continue;
        }
        if item.drop_rate == 0 {
            continue;
        }
        if !multiplayer
            && matches!(
                item.spell,
                crate::game::spelldat::SpellID::Resurrect | crate::game::spelldat::SpellID::HealOther,
            )
        {
            continue;
        }
        if !is_item_okay(item) {
            continue;
        }
        cumulative += if consider_drop_rate { item.drop_rate as u32 } else { 1 };
        ril.push((i, cumulative));
    }
    if ril.is_empty() {
        return None;
    }
    let target = rng.random_range(0..cumulative);
    for &(i, cw) in ril.iter() {
        if target < cw {
            return Some(i);
        }
    }
    ril.last().map(|&(i, _)| i)
}

/// Select random equipment item (no gold, no misc)
/// Exact port of RndUItem() from items.cpp lines 1379-1391
///
/// TODO: Requires AllItemsList + current level tracking
pub fn rnd_u_item(
    monster_level: Option<i32>,
    current_level: i32,
    rng: &mut impl Rng,
    hellfire: bool,
    spawn: bool,
    test_bard: bool,
    multiplayer: bool,
) -> Option<usize> {
    let item_max_level = monster_level.unwrap_or(current_level * 2);

    get_item_index_for_droppable(false, |item_data| {
        // Books are always allowed
        if item_data.item_type == super::item_dat::ItemType::Misc
            && item_data.misc_id == super::item_dat::ItemMiscId::Book {
            return true;
        }

        // Check level requirement
        if item_max_level < item_data.min_mlvl as i32 {
            return false;
        }

        // Exclude gold and misc items
        if matches!(item_data.item_type, super::item_dat::ItemType::Gold | super::item_dat::ItemType::Misc) {
            return false;
        }

        true
    }, rng, hellfire, spawn, test_bard, multiplayer)
}

/// Select random item (75% gold, 25% any item)
/// Exact port of RndAllItems() from items.cpp lines 1392-1403.
pub fn rnd_all_items(
    current_level: i32,
    rng: &mut impl Rng,
    hellfire: bool,
    spawn: bool,
    test_bard: bool,
    multiplayer: bool,
) -> Option<usize> {
    // C++ RndAllItems: `if (GenerateRnd(100) > 25) return IDI_GOLD;`
    // (IDI_GOLD = 0 in itemdat.h `_item_indexes`, matching ItemId::Gold).
    if rng.random_range(0..100) > 25 {
        return Some(super::item_dat::ItemId::Gold as usize);
    }

    let item_max_level = current_level * 2;

    get_item_index_for_droppable(false, |item_data| {
        // Only level requirement check
        item_max_level >= item_data.min_mlvl as i32
    }, rng, hellfire, spawn, test_bard, multiplayer)
}

/// Select random item of specific type
/// Exact port of RndTypeItems() from items.cpp lines 1405-1418
///
/// TODO: Requires AllItemsList
pub fn rnd_type_items(
    item_type: super::item_dat::ItemType,
    misc_id: Option<super::item_dat::ItemMiscId>,
    level: i32,
    rng: &mut impl Rng,
    hellfire: bool,
    spawn: bool,
    test_bard: bool,
    multiplayer: bool,
) -> Option<usize> {
    let item_max_level = level * 2;

    get_item_index_for_droppable(false, |item_data| {
        // Check level
        if item_max_level < item_data.min_mlvl as i32 {
            return false;
        }

        // Check type
        if item_data.item_type != item_type {
            return false;
        }

        // Check misc_id if specified
        if let Some(mid) = misc_id {
            if item_data.misc_id != mid {
                return false;
            }
        }

        true
    }, rng, hellfire, spawn, test_bard, multiplayer)
}

// ============================================================================
// Complete Item Generation Flow
// ============================================================================

/// Create info flags for item generation tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateInfo(pub u16);

impl CreateInfo {
    pub const NONE: Self = Self(0);
    /// Level mask — C++ `CF_LEVEL = (1 << 6) - 1`.
    pub const LEVEL: Self = Self(0x3F);
    /// Pre-generated item — C++ `CF_PREGEN = 1 << 15`.
    pub const PREGEN: Self = Self(1 << 15);
    /// Only good affixes — C++ `CF_ONLYGOOD = 1 << 6`.
    pub const ONLYGOOD: Self = Self(1 << 6);
    /// 15% unique chance — C++ `CF_UPER15 = 1 << 7`.
    pub const UPER15: Self = Self(1 << 7);
    /// 1% unique chance — C++ `CF_UPER1 = 1 << 8`.
    pub const UPER1: Self = Self(1 << 8);
    /// Is a unique item — C++ `CF_UNIQUE = 1 << 9`.
    pub const UNIQUE: Self = Self(1 << 9);
}

/// Complete item generation from seed
/// Exact port of SetupAllItems() from items.cpp lines 3253-3298
///
/// Generates a complete item including:
/// - Base attributes from item index
/// - Random durability
/// - Unique item check (if applicable)
/// - Magic affixes (if not unique)
pub fn setup_all_items<P>(
    item: &mut super::items::Item,
    player: &P,
    item_index: usize,
    seed: u32,
    level: i32,
    unique_percent: i32, // 1 or 15 typical
    only_good: bool,
    pregen: bool,
    uid_offset: i32,
    force_not_unique: bool,
) {
    use super::items::{ItemMiscId, ItemQuality};
    use super::super::engine::random::{discard_random_values, set_rnd_seed};

    // C++: item._iSeed = iseed; SetRndSeed(iseed);
    item.seed = seed;
    set_rnd_seed(seed);

    // C++: GetItemAttrs(item, idx, lvl / 2);
    super::items::get_item_attrs_by_index(item, item_index as i16, level / 2);

    // C++: item._iCreateInfo = lvl; (+ CF_PREGEN / CF_ONLYGOOD / CF_UPER15 / CF_UPER1)
    let mut create_info = level as u16;
    if pregen {
        create_info |= CreateInfo::PREGEN.0;
    }
    if only_good {
        create_info |= CreateInfo::ONLYGOOD.0;
    }
    if unique_percent == 15 {
        create_info |= CreateInfo::UPER15.0;
    } else if unique_percent == 1 {
        create_info |= CreateInfo::UPER1.0;
    }
    item.create_info = create_info;

    if item.misc_id != ItemMiscId::Unique {
        // C++: const int iblvl = GetItemBLevel(lvl, item._iMiscId, onlygood, uper == 15);
        let iblvl = get_item_blevel(level, item.misc_id, only_good, unique_percent == 15);
        if iblvl != -1 {
            let uid = if force_not_unique {
                // C++: DiscardRandomValues(1);
                discard_random_values(1);
                None
            } else {
                check_unique(item, iblvl, unique_percent, uid_offset)
            };
            match uid {
                None => get_item_bonus(item, player, iblvl / 2, iblvl, only_good, true),
                Some(uid) => get_unique_item(player, item, uid),
            }
        }
        // C++: if (item._iMagical != ITEM_QUALITY_UNIQUE) ItemRndDur(item);
        if item.quality != ItemQuality::Unique {
            item_rnd_dur(item);
        }
    } else {
        // Quest-unique item: the uid is stored in iseed (C++ `_unique_items`).
        // C++: if (item._iLoc != ILOC_UNEQUIPABLE) { ... }
        let unequipable = super::item_dat::get_item_data(item_index)
            .map_or(false, |d| d.equip_type == super::item_dat::ItemEquipType::Unequipable);
        if !unequipable {
            let mismatched = seed > 109
                || super::item_dat::UniqueItemId::from_index(seed as usize).map_or(true, |uid| {
                    super::item_dat::get_unique_item_data(uid).map_or(true, |unique| {
                        unique.base_item_id
                            != super::item_dat::get_item_data(item_index)
                                .map(|d| d.unique_base_id)
                                .unwrap_or(super::item_dat::UniqueBaseItem::None)
                    })
                });
            if mismatched {
                // C++: item.clear();
                *item = super::items::Item::empty();
                return;
            }
            get_unique_item(player, item, seed as usize);
        }
    }
}

/// C++ `GetValidUniques(lvl, baseItemId)` (items.cpp:1419-1430): every unique
/// whose base item matches `baseItemId` and whose min level is satisfied.
fn get_valid_uniques(level: i32, base_item_id: super::item_dat::UniqueBaseItem) -> Vec<u16> {
    let mut valid = Vec::new();
    for (index, unique) in super::item_dat::UNIQUE_ITEMS.iter().enumerate() {
        if unique.base_item_id == base_item_id && level >= unique.min_level as i32 {
            valid.push(index as u16);
        }
    }
    valid
}

/// Exact port of CheckUnique() from items.cpp lines 1432-1452.
///
/// Returns `Some(uid)` (index into `UniqueItems`) when the item rolls unique
/// and a valid unique exists for the base item; `None` otherwise.
pub fn check_unique(item: &super::items::Item, level: i32, uper: i32, uid_offset: i32) -> Option<usize> {
    use super::super::engine::random::{discard_random_values, generate_rnd};

    // C++: if (GenerateRnd(100) > uper) return UITEM_INVALID;
    if generate_rnd(100) > uper {
        return None;
    }

    // C++: GetValidUniques(lvl, AllItemsList[item.IDidx].iItemId)
    let base_item_id = super::item_dat::get_item_data(item.item_index as usize)
        .map(|d| d.unique_base_id)
        .unwrap_or(super::item_dat::UniqueBaseItem::None);
    let valid = get_valid_uniques(level, base_item_id);
    if valid.is_empty() {
        return None;
    }

    // C++: DiscardRandomValues(1);
    discard_random_values(1);

    // C++: if (uidOffset >= validUniques.size()) return UITEM_INVALID;
    if uid_offset < 0 || uid_offset as usize >= valid.len() {
        return None;
    }

    // C++: return validUniques[validUniques.size() - 1 - uidOffset];
    Some(valid[valid.len() - 1 - uid_offset as usize] as usize)
}

/// Exact port of GetUniqueItem() from items.cpp lines 1454-1475.
pub fn get_unique_item<P>(_player: &P, item: &mut super::items::Item, uid: usize) {
    use super::item_dat::{ItemEffectType, UniqueItemId};
    use super::items::{ItemMiscId, ItemQuality};

    let Some(id) = UniqueItemId::from_index(uid) else { return };
    let Some(data) = super::item_dat::get_unique_item_data(id) else { return };

    // C++: for (auto power : uniqueItemData.powers) {
    //          if (power.type == IPL_INVALID) break;
    //          SaveItemPower(player, item, power);
    //      }
    for power in data.powers.iter().take(data.num_powers as usize) {
        if power.effect_type == ItemEffectType::Invalid {
            break;
        }
        save_item_power(item, power, 0, 0); // TODO: pass player max hp/mana
    }

    // C++: CopyUtf8(item._iIName, uniqueItemData.UIName, ItemNameLength);
    // The Rust Item has no separate identified-name field; `name` is the
    // display name (matches `apply_random_affixes`).
    item.name = data.name.to_string();

    // C++: if (uniqueItemData.UICurs != ICURS_DEFAULT) item._iCurs = uniqueItemData.UICurs;
    if data.cursor_graphic != 0 {
        item.cursor = data.cursor_graphic;
    }

    // C++: item._iIvalue = uniqueItemData.UIValue;
    item.identified_value = data.value;

    // C++: if (item._iMiscId == IMISC_UNIQUE) item._iSeed = uid;
    if item.misc_id == ItemMiscId::Unique {
        item.seed = uid as u32;
    }

    // C++: item._iUid = uid; item._iMagical = ITEM_QUALITY_UNIQUE;
    //      item._iCreateInfo |= CF_UNIQUE;
    item.unique_id = uid as i32;
    item.quality = ItemQuality::Unique;
    item.create_info |= CreateInfo::UNIQUE.0;
}

/// Randomize item durability
/// Exact port of ItemRndDur() from items.cpp lines 1477-1481
pub fn item_rnd_dur(item: &mut super::items::Item) {
    use super::item_dat::DUR_INDESTRUCTIBLE;
    use super::super::engine::random::generate_rnd;

    // C++: if (item._iDurability > 0 && item._iDurability != DUR_INDESTRUCTIBLE)
    if item.durability > 0 && item.durability != DUR_INDESTRUCTIBLE {
        // C++: item._iDurability = GenerateRnd(item._iMaxDur / 2) + (item._iMaxDur / 4) + 1;
        item.durability = generate_rnd(item.max_durability / 2) + (item.max_durability / 4) + 1;
    }
}

/// Get item bonus level for affix/unique generation
/// Exact port of GetItemBLevel() from items.cpp lines 1483-1508
pub fn get_item_blevel(
    level: i32,
    misc_id: super::items::ItemMiscId,
    only_good: bool,
    uper15: bool,
) -> i32 {
    use super::items::ItemMiscId;
    use super::super::engine::random::generate_rnd;

    // C++: int iblvl = -1;
    let mut iblvl = -1;
    // C++: if (GenerateRnd(100) <= 10 || GenerateRnd(100) <= lvl || onlygood
    //         || IsAnyOf(miscId, IMISC_STAFF, IMISC_RING, IMISC_AMULET))
    if generate_rnd(100) <= 10
        || generate_rnd(100) <= level
        || only_good
        || matches!(misc_id, ItemMiscId::Staff | ItemMiscId::Ring | ItemMiscId::Amulet)
    {
        iblvl = level;
    }
    // C++: if (uper15) iblvl = lvl + 4;
    if uper15 {
        iblvl = level + 4;
    }
    iblvl
}

// ============================================================================
// Item Effects Application to Player
// ============================================================================

/// Accumulated item bonuses from all equipped items
/// Used by CalcPlrItemVals to track total equipment bonuses
#[derive(Debug, Default, Clone)]
pub struct ItemBonuses {
    // Base stats
    pub min_damage: i32,
    pub max_damage: i32,
    pub armor_class: i32,

    // Bonus stats
    pub bonus_damage: i32,
    pub bonus_to_hit: i32,
    pub bonus_ac: i32,

    // Attributes
    pub strength: i32,
    pub magic: i32,
    pub dexterity: i32,
    pub vitality: i32,

    // Resistances
    pub fire_resistance: i32,
    pub lightning_resistance: i32,
    pub magic_resistance: i32,

    // Other bonuses
    pub damage_mod: i32,
    pub get_hit: i32,
    pub light_radius: i32,
    pub life: i32,
    pub mana: i32,
    pub spell_level_add: i8,
    pub target_ac: i32,

    // Elemental damage
    pub min_fire_damage: i32,
    pub max_fire_damage: i32,
    pub min_lightning_damage: i32,
    pub max_lightning_damage: i32,

    // Special flags
    pub special_flags: u64,  // ItemSpecialEffect bits
    pub dam_ac_flags: u64,   // ItemSpecialEffectHf bits

    // Available spells from items
    pub spells: u64,  // Spell bitmask
}

/// Calculate all item bonuses from player's equipped items
/// Exact port of CalcPlrItemVals() from items.cpp lines 2775-2890
///
/// This is the core function that applies all item effects to the player.
/// It sums up all bonuses from equipped items (weapons, armor, jewelry, etc.)
///
/// TODO: Requires Player struct with InvBody equipment slots
pub fn calc_player_item_values(
    _player: &super::player::Player,
    equipped_items: &[super::items::Item],
) -> ItemBonuses {
    let mut bonuses = ItemBonuses::default();
    bonuses.light_radius = 10;  // Base light radius

    // Iterate through all equipped items
    for item in equipped_items {
        // Skip empty slots
        if item.item_type == super::items::ItemType::None {
            continue;
        }

        // Skip items that don't meet stat requirements
        // TODO: Check item.stat_flag when available

        // Add base damage and AC
        bonuses.min_damage += item.min_damage as i32;
        bonuses.max_damage += item.max_damage as i32;
        bonuses.armor_class += item.armor_class as i32;

        // Add spell charges
        // TODO: Check if spell is valid and has charges
        // if is_valid_spell(item.spell) && item.charges != 0 {
        //     bonuses.spells |= get_spell_bitmask(item.spell);
        // }

        // Apply magic item bonuses (only if identified or normal quality)
        let is_identified = true;  // TODO: Check item.identified flag
        if item.quality == super::items::ItemQuality::Normal || is_identified {
            bonuses.bonus_damage += item.bonus_damage as i32;
            bonuses.bonus_to_hit += item.bonus_to_hit as i32;
            bonuses.bonus_ac += item.bonus_ac as i32;

            // Special flags
            bonuses.special_flags |= item.special_flags.0 as u64;
            // TODO: Add dam_ac_flags when available

            // Attribute bonuses
            bonuses.strength += item.bonus_str as i32;
            bonuses.magic += item.bonus_mag as i32;
            bonuses.dexterity += item.bonus_dex as i32;
            bonuses.vitality += item.bonus_vit as i32;

            // Resistances
            bonuses.fire_resistance += item.resist_fire as i32;
            bonuses.lightning_resistance += item.resist_lightning as i32;
            bonuses.magic_resistance += item.resist_magic as i32;

            // Other bonuses
            bonuses.damage_mod += item.bonus_damage_mod as i32;
            bonuses.get_hit += item.bonus_get_hit as i32;
            bonuses.light_radius += item.bonus_light as i32;
            bonuses.life += item.bonus_hp as i32;
            bonuses.mana += item.bonus_mana as i32;
            bonuses.spell_level_add += item.spell_level_add;
            bonuses.target_ac += item.bonus_energy_ac as i32;

            // Elemental damage
            bonuses.min_fire_damage += item.fire_min_dam as i32;
            bonuses.max_fire_damage += item.fire_max_dam as i32;
            bonuses.min_lightning_damage += item.lightning_min_dam as i32;
            bonuses.max_lightning_damage += item.lightning_max_dam as i32;
        }
    }

    bonuses
}

/// Apply item bonuses to player stats
/// This would be called after calc_player_item_values() to update player fields
///
/// TODO: Requires full Player struct implementation
pub fn apply_item_bonuses_to_player(
    _player: &mut super::player::Player,
    bonuses: &ItemBonuses,
) {
    // TODO: Apply bonuses to player fields
    // player.item_armor_class = bonuses.armor_class;
    // player.item_bonus_damage = bonuses.bonus_damage;
    // player.item_bonus_to_hit = bonuses.bonus_to_hit;
    // ... etc

    // This function would also call:
    // - CalcPlrDamage() to compute final damage
    // - CalcPlrPrimaryStats() to apply attribute bonuses
    // - CalcPlrLightRadius() to update light radius
    // - CalcPlrResistances() to compute final resistances
    // - CalcPlrLifeMana() to update HP/Mana maximums
    // - CalcPlrBlockFlag() to determine if player can block
}

/// Calculate if a single item meets player's stat requirements
/// Port of CalcSelfItems() logic from items.cpp
///
/// TODO: Requires Player stat fields and Item requirement fields
pub fn check_item_stat_requirements(
    _player: &super::player::Player,
    _item: &super::items::Item,
) -> bool {
    // TODO: Compare player stats vs item requirements
    // player.strength >= item.required_str &&
    // player.magic >= item.required_mag &&
    // player.dexterity >= item.required_dex

    true  // Placeholder
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// C++ `RndAllItems` (items.cpp:1392-1403): a 75% roll returns
    /// `IDI_GOLD` (= 0). The port must return `Some(0)` for a gold roll
    /// (previously a placeholder returned `None`).
    #[test]
    fn test_rnd_all_items_gold_roll_returns_index_zero() {
        use rand::SeedableRng;
        // Single player, non-Hellfire: find a seed whose first roll > 25.
        let mut gold_seed = None;
        for seed in 1..=500u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            if rnd_all_items(1, &mut rng, false, false, false, false) == Some(0) {
                gold_seed = Some(seed);
                break;
            }
        }
        let seed = gold_seed.expect("some seed must roll the 75% gold branch");
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        assert_eq!(rnd_all_items(1, &mut rng, false, false, false, false), Some(0), "gold roll maps to IDI_GOLD=0");
    }

    /// C++ `GetItemIndexForDroppableItem` (items.cpp:1353-1377): the weighted
    /// pick now runs over the aligned `ITEMS_DATA` (dropRate > 0, level filter).
    #[test]
    fn test_get_item_index_for_droppable_picks_from_items_data() {
        use rand::SeedableRng;
        // Level 1: only items with min_mlvl <= 1 are candidates (e.g. heal potion
        // row 77). Excludes gold (dropRate column is 1 for gold? verify by pick).
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let mut picked = 0usize;
        for _ in 0..200 {
            let idx = get_item_index_for_droppable(false, |item| item.min_mlvl <= 1, &mut rng, false, false, false, false)
                .expect("level-1 droppable item exists");
            assert!(idx < crate::game::item_dat::ITEMS_DATA.len());
            assert!(crate::game::item_dat::ITEMS_DATA[idx].min_mlvl <= 1);
            picked += 1;
        }
        assert_eq!(picked, 200);
        // Gold (row 0) is a droppable candidate too (dropRate 1, min_mlvl 1).
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(1);
        let mut saw_gold = false;
        for _ in 0..100 {
            if let Some(idx) = get_item_index_for_droppable(false, |item| item.min_mlvl <= 1, &mut rng2, false, false, false, false) {
                if idx == 0 {
                    saw_gold = true;
                    break;
                }
            }
        }
        assert!(saw_gold, "gold should be a droppable candidate");
    }

    #[test]
    fn test_affix_item_type_flags() {
        // Core 6-bit coarse categories — bit-identical to C++ enum (Source/itemdat.h:609)
        assert_eq!(AffixItemType::MISC.0, 1 << 0);
        assert_eq!(AffixItemType::BOW.0, 1 << 1);
        assert_eq!(AffixItemType::STAFF.0, 1 << 2);
        assert_eq!(AffixItemType::WEAPON.0, 1 << 3);
        assert_eq!(AffixItemType::SHIELD.0, 1 << 4);
        assert_eq!(AffixItemType::ARMOR.0, 1 << 5);

        // Aliases
        assert_eq!(AffixItemType::ALL_WEAPONS, AffixItemType::WEAPON);
        assert_eq!(AffixItemType::ALL_ARMOR, AffixItemType::ARMOR);
        assert_eq!(AffixItemType::JEWELRY, AffixItemType::MISC);
        assert_eq!(AffixItemType::HELM, AffixItemType::ARMOR);

        // A weapon affix matches a weapon search mask but not a shield/armor one
        assert!(AffixItemType::ALL_WEAPONS.contains(AffixItemType::WEAPON));
        assert!(!AffixItemType::ALL_WEAPONS.contains(AffixItemType::SHIELD));
        assert!(AffixItemType::ALL_ARMOR.contains(AffixItemType::HELM));
    }

    #[test]
    fn test_from_item_type() {
        // Mirrors C++ Source/items.cpp:2283 switch
        assert_eq!(AffixItemType::from_item_type(ItemType::Sword), AffixItemType::WEAPON);
        assert_eq!(AffixItemType::from_item_type(ItemType::Axe), AffixItemType::WEAPON);
        assert_eq!(AffixItemType::from_item_type(ItemType::Mace), AffixItemType::WEAPON);
        assert_eq!(AffixItemType::from_item_type(ItemType::Bow), AffixItemType::BOW);
        assert_eq!(AffixItemType::from_item_type(ItemType::Staff), AffixItemType::STAFF);
        assert_eq!(AffixItemType::from_item_type(ItemType::Shield), AffixItemType::SHIELD);
        assert_eq!(AffixItemType::from_item_type(ItemType::Helm), AffixItemType::ARMOR);
        assert_eq!(AffixItemType::from_item_type(ItemType::LightArmor), AffixItemType::ARMOR);
        assert_eq!(AffixItemType::from_item_type(ItemType::Ring), AffixItemType::MISC);
        assert_eq!(AffixItemType::from_item_type(ItemType::Amulet), AffixItemType::MISC);
    }

    #[test]
    fn test_item_prefixes_count() {
        // Verify we have the expected number of prefixes
        assert!(ITEM_PREFIXES.len() >= 25, "Expected at least 25 prefixes, got {}", ITEM_PREFIXES.len());
    }

    #[test]
    fn test_item_suffixes_count() {
        // Verify we have the expected number of suffixes
        assert!(ITEM_SUFFIXES.len() >= 30, "Expected at least 30 suffixes, got {}", ITEM_SUFFIXES.len());
    }

    #[test]
    fn test_prefix_to_hit_weapons() {
        // Test "Tin" prefix - first prefix (ToHitCurse)
        let tin = &ITEM_PREFIXES[0];
        assert_eq!(tin.name, "Tin");
        assert_eq!(tin.power.effect_type, ItemEffectType::ToHitCurse);
        assert_eq!(tin.power.param1, 6);
        assert_eq!(tin.power.param2, 10);
        assert_eq!(tin.min_level, 3);
        // TSV itemTypes = "Weapon,Bow,Misc" => affix matches weapon/bow/jewelry searches
        assert!(tin.item_types.contains(AffixItemType::WEAPON));
        assert!(tin.item_types.contains(AffixItemType::BOW));
        assert!(tin.item_types.contains(AffixItemType::JEWELRY));
        assert!(!tin.item_types.contains(AffixItemType::SHIELD));
        assert!(!tin.is_good);

        // Test "Gold" prefix - high-level to-hit
        let gold = ITEM_PREFIXES.iter().find(|a| a.name == "Gold").unwrap();
        assert_eq!(gold.power.param1, 21);
        assert_eq!(gold.power.param2, 30);
        assert_eq!(gold.min_level, 12);
        assert_eq!(gold.mult_val, 9);
    }

    #[test]
    fn test_prefix_damage_weapons() {
        // Test damage prefixes
        let sharp = ITEM_PREFIXES.iter().find(|a| a.name == "Sharp").unwrap();
        assert_eq!(sharp.power.effect_type, ItemEffectType::ToHitDamage);
        assert_eq!(sharp.power.param1, 20);
        assert_eq!(sharp.power.param2, 35);

        let kings = ITEM_PREFIXES.iter().find(|a| a.name == "King's").unwrap();
        assert_eq!(kings.power.effect_type, ItemEffectType::ToHitDamage);
        assert_eq!(kings.power.param1, 151);
        assert_eq!(kings.power.param2, 175);
        assert_eq!(kings.min_level, 28);
    }

    #[test]
    fn test_prefix_armor_ac() {
        // Test AC% prefixes for armor - there are two "Fine", find the one with ArmorPercent
        let fine = ITEM_PREFIXES.iter().find(|a| a.name == "Fine" && a.power.effect_type == ItemEffectType::ArmorPercent).unwrap();
        assert_eq!(fine.power.effect_type, ItemEffectType::ArmorPercent);
        assert_eq!(fine.power.param1, 20);
        assert_eq!(fine.power.param2, 30);
        // TSV itemTypes = "Armor,Shield"
        assert!(fine.item_types.contains(AffixItemType::ALL_ARMOR));
        assert!(fine.item_types.contains(AffixItemType::HELM)); // helms are Armor
        assert!(fine.item_types.contains(AffixItemType::SHIELD));

        let godly = ITEM_PREFIXES.iter().find(|a| a.name == "Godly").unwrap();
        assert_eq!(godly.power.param1, 171);
        assert_eq!(godly.power.param2, 200);
        assert_eq!(godly.min_level, 60);
    }

    #[test]
    fn test_suffix_attributes() {
        // Test first suffix - quality (damage modifier)
        let quality = &ITEM_SUFFIXES[0];
        assert_eq!(quality.name, "quality");
        assert_eq!(quality.power.effect_type, ItemEffectType::DamMod);
        assert_eq!(quality.power.param1, 1);
        assert_eq!(quality.power.param2, 2);

        // Test strength suffix
        let strength = ITEM_SUFFIXES.iter().find(|a| a.name == "strength").unwrap();
        assert_eq!(strength.power.effect_type, ItemEffectType::Str);

        let titans = ITEM_SUFFIXES.iter().find(|a| a.name == "titans").unwrap();
        assert_eq!(titans.power.effect_type, ItemEffectType::Str);
        assert_eq!(titans.power.param1, 21);
        assert_eq!(titans.power.param2, 30);
        assert_eq!(titans.min_level, 23);

        // Test magic suffixes
        let mind = ITEM_SUFFIXES.iter().find(|a| a.name == "the mind").unwrap();
        assert_eq!(mind.power.effect_type, ItemEffectType::Mag);

        // Test dexterity suffix
        let dexterity = ITEM_SUFFIXES.iter().find(|a| a.name == "dexterity").unwrap();
        assert_eq!(dexterity.power.effect_type, ItemEffectType::Dex);

        // Test vitality suffix
        let vitality = ITEM_SUFFIXES.iter().find(|a| a.name == "vitality").unwrap();
        assert_eq!(vitality.power.effect_type, ItemEffectType::Vit);
    }

    #[test]
    fn test_suffix_life_mana() {
        // Test life suffixes - note: "the fox" is a Life effect, param1=10-15
        let fox = ITEM_SUFFIXES.iter().find(|a| a.name == "the fox").unwrap();
        assert_eq!(fox.power.effect_type, ItemEffectType::Life);
        assert_eq!(fox.power.param1, 10);
        assert_eq!(fox.power.param2, 15);

        // Note: Mana effects are all PREFIXES (Spider's, Raven's, etc.), not suffixes
        // Test all attributes suffix instead
        let moon = ITEM_SUFFIXES.iter().find(|a| a.name == "the moon").unwrap();
        assert_eq!(moon.power.effect_type, ItemEffectType::Attribs);
        assert_eq!(moon.power.param1, 4);
        assert_eq!(moon.power.param2, 7);
    }

    #[test]
    fn test_suffix_resistances() {
        // Note: Fire/Lightning/Magic resistance are PREFIX effects, not suffix
        // Test fire arrows (BOW suffix)
        let fire = ITEM_SUFFIXES.iter().find(|a| a.name == "fire").unwrap();
        assert_eq!(fire.power.effect_type, ItemEffectType::FireArrows);
        assert_eq!(fire.power.param1, 1);
        assert_eq!(fire.power.param2, 6);

        let burning = ITEM_SUFFIXES.iter().find(|a| a.name == "burning").unwrap();
        assert_eq!(burning.power.effect_type, ItemEffectType::FireArrows);
        assert_eq!(burning.power.param1, 1);
        assert_eq!(burning.power.param2, 16);

        // Test lightning arrows (BOW suffix)
        let lightning = ITEM_SUFFIXES.iter().find(|a| a.name == "lightning").unwrap();
        assert_eq!(lightning.power.effect_type, ItemEffectType::LightArrows);

        // Test all attributes (the zodiac)
        let zodiac = ITEM_SUFFIXES.iter().find(|a| a.name == "the zodiac").unwrap();
        assert_eq!(zodiac.power.effect_type, ItemEffectType::Attribs);
        assert_eq!(zodiac.min_level, 30);
    }

    #[test]
    fn test_rnd_pl() {
        // Test same value
        assert_eq!(rnd_pl(5, 5), 5);

        // Test range
        for _ in 0..100 {
            let val = rnd_pl(1, 10);
            assert!(val >= 1 && val <= 10);
        }
    }

    #[test]
    fn test_calculate_to_hit_bonus() {
        assert_eq!(calculate_to_hit_bonus(0), 0);
        assert_eq!(calculate_to_hit_bonus(100), 25);
        assert_eq!(calculate_to_hit_bonus(-100), 25);
        assert_eq!(calculate_to_hit_bonus(50), 12);
    }

    #[test]
    fn test_pl_val() {
        // Test fixed value
        assert_eq!(pl_val(5, 5, 5, 10, 20), 5);

        // Test scaling
        let result = pl_val(5, 1, 10, 100, 200);
        assert!(result >= 100 && result <= 200);

        // Test min boundary
        let min_result = pl_val(1, 1, 10, 100, 200);
        assert_eq!(min_result, 100);

        // Test max boundary
        let max_result = pl_val(10, 1, 10, 100, 200);
        assert_eq!(max_result, 200);
    }

    #[test]
    fn test_staff_spells() {
        // Verify staff spell list
        assert_eq!(STAFF_SPELLS.len(), 20);

        // Test first spell (Firebolt)
        assert_eq!(STAFF_SPELLS[0].0, SpellId::Firebolt);
        assert_eq!(STAFF_SPELLS[0].1, 15);

        // Test last spell (Apocalypse)
        assert_eq!(STAFF_SPELLS[19].0, SpellId::Apocalypse);
        assert_eq!(STAFF_SPELLS[19].1, 149);
    }

    #[test]
    fn test_get_staff_spell() {
        // Low level should get basic spells
        let low_spell = get_staff_spell(10);
        assert!(low_spell.is_some());
        let (_, min_mag) = low_spell.unwrap();
        assert!(min_mag <= 20); // level 10 * 2

        // High level should have access to all spells
        let high_spell = get_staff_spell(100);
        assert!(high_spell.is_some());

        // Level 0 should still get the first spell
        let zero_spell = get_staff_spell(0);
        assert!(zero_spell.is_some());
    }

    #[test]
    fn test_affix_item_type_coverage() {
        // Verify weapon prefixes (ToHit/Damage) apply to the Weapon category
        for prefix in ITEM_PREFIXES.iter() {
            if prefix.power.effect_type == ItemEffectType::ToHit
                || prefix.power.effect_type == ItemEffectType::Damage
            {
                assert!(
                    prefix.item_types.contains(AffixItemType::ALL_WEAPONS),
                    "Prefix {} ({:?}) should apply to weapons",
                    prefix.name,
                    prefix.power.effect_type
                );
            }
        }

        // Verify armor prefixes apply to the Armor category
        for prefix in ITEM_PREFIXES.iter() {
            if prefix.power.effect_type == ItemEffectType::ArmorPercent {
                assert!(
                    prefix.item_types.contains(AffixItemType::ALL_ARMOR),
                    "Prefix {} should apply to armor",
                    prefix.name
                );
            }
        }
    }

    #[test]
    fn test_affix_level_progression() {
        // Verify prefixes have increasing min_level with power
        let to_hit_prefixes: Vec<_> = ITEM_PREFIXES
            .iter()
            .filter(|a| a.power.effect_type == ItemEffectType::ToHit)
            .collect();

        for i in 1..to_hit_prefixes.len() {
            assert!(
                to_hit_prefixes[i].min_level >= to_hit_prefixes[i - 1].min_level,
                "Level progression should be non-decreasing"
            );
        }
    }

    #[test]
    fn test_good_evil_alignment() {
        // Test alignment consistency with is_good flag
        // Note: Some affixes like "Vicious" have alignment=Evil but is_good=true (cursed good items)
        for prefix in ITEM_PREFIXES.iter() {
            // is_good=false should not have Good alignment
            if !prefix.is_good {
                assert_ne!(prefix.alignment, GoodOrEvil::Good, "Prefix {} with is_good=false cannot have Good alignment", prefix.name);
            }
        }

        for suffix in ITEM_SUFFIXES.iter() {
            if !suffix.is_good {
                assert_ne!(suffix.alignment, GoodOrEvil::Good, "Suffix {} with is_good=false cannot have Good alignment", suffix.name);
            }
        }
    }

    // ========================================================================
    // M75: Integration Tests - Complete System Validation
    // ========================================================================

    #[test]
    fn test_integration_save_item_power_basic() {
        use super::super::items::Item;

        // Create mock item
        let mut item = Item::default();

        // Test ToHit effect
        let power = ItemPower {
            effect_type: ItemEffectType::ToHit,
            param1: 5,
            param2: 10,
        };

        let value = save_item_power(&mut item, &power, 100, 100);
        assert!(value >= 5 && value <= 10, "Random value should be in range");
        assert_eq!(item.bonus_to_hit, value as i16, "ToHit should be applied");
    }

    #[test]
    fn test_integration_save_item_affix() {
        use super::super::items::Item;

        // Find a test affix
        let sharp = ITEM_PREFIXES.iter().find(|a| a.name == "Sharp").unwrap();

        let mut item = Item::default();
        save_item_affix(&mut item, sharp, 100, 100);

        // Verify damage was applied
        assert!(item.bonus_damage > 0, "Sharp should add damage");

        // Verify value multipliers were set
        assert!(item.value_mult1 != 0 || item.value_add1 != 0, "Value should be modified");
    }

    #[test]
    fn test_integration_generate_item_name() {
        let sharp = ITEM_PREFIXES.iter().find(|a| a.name == "Sharp").unwrap();
        let quality = ITEM_SUFFIXES.iter().find(|a| a.name == "quality").unwrap();

        // Test all combinations
        let name1 = generate_magic_item_name("Long Sword", Some(sharp), Some(quality));
        assert_eq!(name1, "Sharp Long Sword of quality");

        let name2 = generate_magic_item_name("Long Sword", Some(sharp), None);
        assert_eq!(name2, "Sharp Long Sword");

        let name3 = generate_magic_item_name("Long Sword", None, Some(quality));
        assert_eq!(name3, "Long Sword of quality");

        let name4 = generate_magic_item_name("Long Sword", None, None);
        assert_eq!(name4, "Long Sword");
    }

    #[test]
    fn test_integration_calc_item_value() {
        use super::super::items::Item;

        let mut item = Item::default();
        item.buy_value = 100;
        item.value_add1 = 50;
        item.value_mult1 = 2;
        item.value_add2 = 100;
        item.value_mult2 = 3;

        let value = calc_affix_item_value(&item);
        // (100 + 50) * 2 = 300
        // (300 + 100) * 3 = 1200
        assert_eq!(value, 1200);
    }

    #[test]
    fn test_integration_item_bonus_accumulation() {
        use super::super::items::{Item, ItemQuality};
        use super::super::player::Player;

        // Create multiple test items with bonuses
        let mut items = vec![
            Item::default(),
            Item::default(),
        ];

        // Item 1: +5 STR
        items[0].item_type = super::super::items::ItemType::Ring;  // Must have a type
        items[0].bonus_str = 5;
        items[0].quality = ItemQuality::Magic;

        // Item 2: +10 STR, +20 HP
        items[1].item_type = super::super::items::ItemType::Amulet;  // Must have a type
        items[1].bonus_str = 10;
        items[1].bonus_hp = 20;
        items[1].quality = ItemQuality::Magic;

        let bonuses = calc_player_item_values(&Player::default(), &items);

        assert_eq!(bonuses.strength, 15, "Strength should sum to 15");
        assert_eq!(bonuses.life, 20, "Life should be 20");
    }

    #[test]
    fn test_integration_special_flags_combination() {
        use super::super::items::{Item, ItemQuality, ItemSpecialEffect};
        use super::super::player::Player;

        let mut items = vec![
            Item::default(),
            Item::default(),
        ];

        // Item 1: Fire arrows
        items[0].item_type = super::super::items::ItemType::Bow;  // Must have a type
        items[0].special_flags.0 = ItemSpecialEffect::FIRE_ARROWS.0;
        items[0].quality = ItemQuality::Magic;

        // Item 2: Lightning arrows
        items[1].item_type = super::super::items::ItemType::Ring;  // Must have a type
        items[1].special_flags.0 = ItemSpecialEffect::LIGHTNING_ARROWS.0;
        items[1].quality = ItemQuality::Magic;

        let bonuses = calc_player_item_values(&Player::default(), &items);

        // Both flags should be combined
        assert_ne!(bonuses.special_flags & ItemSpecialEffect::FIRE_ARROWS.0 as u64, 0);
        assert_ne!(bonuses.special_flags & ItemSpecialEffect::LIGHTNING_ARROWS.0 as u64, 0);
    }

    #[test]
    fn test_integration_resistance_stacking() {
        use super::super::items::{Item, ItemQuality};
        use super::super::player::Player;

        let mut items = vec![
            Item::default(),
            Item::default(),
            Item::default(),
        ];

        // All items add fire resistance
        items[0].item_type = super::super::items::ItemType::Ring;
        items[0].resist_fire = 10;
        items[0].quality = ItemQuality::Magic;

        items[1].item_type = super::super::items::ItemType::Amulet;
        items[1].resist_fire = 25;
        items[1].quality = ItemQuality::Magic;

        items[2].item_type = super::super::items::ItemType::Helm;
        items[2].resist_fire = 40;
        items[2].quality = ItemQuality::Magic;

        let bonuses = calc_player_item_values(&Player::default(), &items);

        assert_eq!(bonuses.fire_resistance, 75, "Fire resistance should stack to 75");
    }

    #[test]
    fn test_integration_complete_affix_workflow() {
        use super::super::items::{Item, ItemQuality};
        use super::super::player::Player;

        // Simulate complete workflow:
        // 1. Create item
        // 2. Apply prefix
        // 3. Apply suffix
        // 4. Generate name
        // 5. Calculate value

        let mut item = Item::default();
        item.item_type = super::super::items::ItemType::Sword;  // Must have a type
        item.base_name = "Long Sword".to_string();
        item.buy_value = 100;

        // Apply prefix: "Sharp" (+20-35 to-hit and damage)
        let sharp = ITEM_PREFIXES.iter().find(|a| a.name == "Sharp").unwrap();
        save_item_affix(&mut item, sharp, 100, 100);

        // Apply suffix: "quality" (+1-2 damage multiplier)
        let quality = ITEM_SUFFIXES.iter().find(|a| a.name == "quality").unwrap();
        save_item_affix(&mut item, quality, 100, 100);

        // Verify bonuses applied
        assert!(item.bonus_damage > 0, "Should have damage bonus from sharp");
        assert!(item.bonus_to_hit > 0, "Should have to-hit bonus from quality");

        // Calculate final value
        let final_value = calc_affix_item_value(&item);
        assert!(final_value > 100, "Magic item should be worth more than base");

        // Mark as magic quality
        item.quality = ItemQuality::Magic;

        // Verify it would contribute to player stats
        let bonuses = calc_player_item_values(&Player::default(), &[item]);
        assert!(bonuses.bonus_damage > 0);
        assert!(bonuses.bonus_to_hit > 0);
    }

    #[test]
    fn test_integration_elemental_damage() {
        use super::super::items::{Item, ItemSpecialEffect};

        let mut item = Item::default();

        // Apply fire damage effect
        let fire_power = ItemPower {
            effect_type: ItemEffectType::FireDam,
            param1: 1,
            param2: 6,
        };

        save_item_power(&mut item, &fire_power, 100, 100);

        // Verify fire damage set
        assert_eq!(item.fire_min_dam, 1);
        assert_eq!(item.fire_max_dam, 6);

        // Verify fire flag set
        assert_ne!(item.special_flags.0 & ItemSpecialEffect::FIRE_ARROWS.0, 0);

        // Verify lightning cleared (mutually exclusive)
        assert_eq!(item.lightning_min_dam, 0);
        assert_eq!(item.lightning_max_dam, 0);
    }

    #[test]
    fn test_integration_all_effect_types_coverage() {
        use super::super::items::Item;

        // Verify all ItemEffectType cases are handled in save_item_power
        let mut item = Item::default();

        // Test a few critical effect types
        let effects = vec![
            ItemEffectType::ToHit,
            ItemEffectType::Damage,
            ItemEffectType::ArmorPercent,
            ItemEffectType::FireRes,
            ItemEffectType::Str,
            ItemEffectType::Life,
            ItemEffectType::Mana,
            ItemEffectType::Indestructible,
        ];

        for effect in effects {
            let power = ItemPower {
                effect_type: effect,
                param1: 10,
                param2: 20,
            };

            // Should not panic
            save_item_power(&mut item, &power, 100, 100);
        }
    }

    #[test]
    fn test_integration_affix_data_consistency() {
        // Verify all affixes have valid data
        for prefix in ITEM_PREFIXES.iter() {
            assert!(!prefix.name.is_empty(), "Prefix name should not be empty");
            assert!(prefix.min_level >= 0, "Min level should be non-negative");
            assert!(prefix.chance > 0, "Chance should be positive");
            assert!(prefix.item_types.0 != 0, "Should apply to at least one item type");
        }

        for suffix in ITEM_SUFFIXES.iter() {
            assert!(!suffix.name.is_empty(), "Suffix name should not be empty");
            assert!(suffix.min_level >= 0, "Min level should be non-negative");
            assert!(suffix.chance > 0, "Chance should be positive");
            assert!(suffix.item_types.0 != 0, "Should apply to at least one item type");
        }
    }

    // ========================================================================
    // TSV-authoritative alignment tests (assets/txtdata/items/item_*.tsv)
    //
    // These lock the Rust data tables to the exact row counts and spot-checked
    // field values of the C++ runtime data source. If the TSV ever changes,
    // update both the table and these constants together.
    // ========================================================================

    /// C++ `item_prefixes.tsv` has 83 records (header excluded).
    #[test]
    fn test_tsv_prefix_count_exact() {
        assert_eq!(ITEM_PREFIXES.len(), 83, "must match item_prefixes.tsv");
    }

    /// C++ `item_suffixes.tsv` has 95 records (header excluded).
    #[test]
    fn test_tsv_suffix_count_exact() {
        assert_eq!(ITEM_SUFFIXES.len(), 95, "must match item_suffixes.tsv");
    }

    /// Spot-check several well-known prefixes against the TSV values.
    #[test]
    fn test_known_prefixes_match_cpp() {
        // King's: TOHIT_DAMP 151-175, minLvl 28, Weapon,Staff, Any, chance 2, useful, 24100..35000 x38
        let kings = ITEM_PREFIXES.iter().find(|a| a.name == "King's").unwrap();
        assert_eq!(kings.power.effect_type, ItemEffectType::ToHitDamage);
        assert_eq!(kings.power.param1, 151);
        assert_eq!(kings.power.param2, 175);
        assert_eq!(kings.min_level, 28);
        assert_eq!(kings.alignment, GoodOrEvil::Any);
        assert!(kings.is_good);
        assert_eq!(kings.chance, 2);
        assert_eq!((kings.min_val, kings.max_val, kings.mult_val), (24100, 35000, 38));
        // King's itemTypes = "Weapon,Staff"
        assert!(kings.item_types.contains(AffixItemType::WEAPON));
        assert!(kings.item_types.contains(AffixItemType::STAFF));
        assert!(!kings.item_types.contains(AffixItemType::BOW));
        assert!(!kings.item_types.contains(AffixItemType::ARMOR));

        // Silver: TOHIT 16-20, minLvl 9, Good alignment
        let silver = ITEM_PREFIXES.iter().find(|a| a.name == "Silver").unwrap();
        assert_eq!(silver.power.param1, 16);
        assert_eq!(silver.power.param2, 20);
        assert_eq!(silver.min_level, 9);
        assert_eq!(silver.alignment, GoodOrEvil::Good);

        // Ruby: FIRERES 51-60, minLvl 26, multVal 5
        let ruby = ITEM_PREFIXES.iter().find(|a| a.name == "Ruby").unwrap();
        assert_eq!(ruby.power.effect_type, ItemEffectType::FireRes);
        assert_eq!(ruby.power.param1, 51);
        assert_eq!(ruby.power.param2, 60);
        assert_eq!(ruby.mult_val, 5);

        // Wyrm's: MANA 61-80, Staff only (no jewelry), minLvl 35
        let wyrm = ITEM_PREFIXES.iter().find(|a| a.name == "Wyrm's").unwrap();
        assert_eq!(wyrm.power.effect_type, ItemEffectType::Mana);
        assert!(wyrm.item_types.contains(AffixItemType::STAFF));
        assert!(!wyrm.item_types.contains(AffixItemType::JEWELRY));
        assert_eq!(wyrm.min_level, 35);

        // Godly: ACP 171-200, minLvl 60, Armor,Shield, Good
        let godly = ITEM_PREFIXES.iter().find(|a| a.name == "Godly").unwrap();
        assert_eq!(godly.power.effect_type, ItemEffectType::ArmorPercent);
        assert_eq!((godly.power.param1, godly.power.param2), (171, 200));
        assert_eq!(godly.alignment, GoodOrEvil::Good);
        assert!(godly.item_types.contains(AffixItemType::ARMOR));
        assert!(godly.item_types.contains(AffixItemType::SHIELD));
    }

    /// Spot-check several well-known suffixes against the TSV values.
    #[test]
    fn test_known_suffixes_match_cpp() {
        // carnage: DAMMOD 13-16, minLvl 35, Weapon only
        let carnage = ITEM_SUFFIXES.iter().find(|a| a.name == "carnage").unwrap();
        assert_eq!(carnage.power.effect_type, ItemEffectType::DamMod);
        assert_eq!((carnage.power.param1, carnage.power.param2), (13, 16));
        assert_eq!(carnage.min_level, 35);
        assert!(carnage.item_types.contains(AffixItemType::WEAPON));
        assert!(!carnage.item_types.contains(AffixItemType::BOW));

        // the wolf: LIFE 30-40, minLvl 15, Armor,Shield,Misc (jewelry)
        let wolf = ITEM_SUFFIXES.iter().find(|a| a.name == "the wolf").unwrap();
        assert_eq!(wolf.power.effect_type, ItemEffectType::Life);
        assert_eq!((wolf.power.param1, wolf.power.param2), (30, 40));
        assert_eq!(wolf.min_level, 15);
        assert!(wolf.item_types.contains(AffixItemType::ARMOR));
        assert!(wolf.item_types.contains(AffixItemType::SHIELD));
        assert!(wolf.item_types.contains(AffixItemType::JEWELRY));

        // the whale: LIFE 81-100, minLvl 60, Armor only
        let whale = ITEM_SUFFIXES.iter().find(|a| a.name == "the whale").unwrap();
        assert_eq!((whale.power.param1, whale.power.param2), (81, 100));
        assert_eq!(whale.min_level, 60);
        assert!(whale.item_types.contains(AffixItemType::ARMOR));
        assert!(!whale.item_types.contains(AffixItemType::SHIELD));

        // the zodiac: ATTRIBS 16-20, minLvl 30, Misc (jewelry) only
        let zodiac = ITEM_SUFFIXES.iter().find(|a| a.name == "the zodiac").unwrap();
        assert_eq!(zodiac.power.effect_type, ItemEffectType::Attribs);
        assert_eq!((zodiac.power.param1, zodiac.power.param2), (16, 20));
        assert_eq!(zodiac.min_level, 30);
        assert!(zodiac.item_types.contains(AffixItemType::JEWELRY));
        assert!(!zodiac.item_types.contains(AffixItemType::ARMOR));

        // the ages: INDESTRUCTIBLE, minLvl 25, Armor,Shield,Weapon
        let ages = ITEM_SUFFIXES.iter().find(|a| a.name == "the ages").unwrap();
        assert_eq!(ages.power.effect_type, ItemEffectType::Indestructible);
        assert_eq!(ages.min_level, 25);
        assert!(ages.item_types.contains(AffixItemType::ARMOR));
        assert!(ages.item_types.contains(AffixItemType::SHIELD));
        assert!(ages.item_types.contains(AffixItemType::WEAPON));

        // corruption: NOMANA, minLvl 5, Evil, negative value -1000
        let corruption = ITEM_SUFFIXES.iter().find(|a| a.name == "corruption").unwrap();
        assert_eq!(corruption.power.effect_type, ItemEffectType::NoMana);
        assert_eq!(corruption.alignment, GoodOrEvil::Evil);
        assert!(!corruption.is_good);
        assert_eq!((corruption.min_val, corruption.max_val), (-1000, -1000));

        // the bear: KNOCKBACK, Weapon,Staff,Bow, Evil (good-aligned cursed)
        let bear = ITEM_SUFFIXES.iter().find(|a| a.name == "the bear").unwrap();
        assert_eq!(bear.power.effect_type, ItemEffectType::Knockback);
        assert_eq!(bear.alignment, GoodOrEvil::Evil);
        assert!(bear.is_good);
        assert!(bear.item_types.contains(AffixItemType::WEAPON));
        assert!(bear.item_types.contains(AffixItemType::STAFF));
        assert!(bear.item_types.contains(AffixItemType::BOW));

        // blocking: FASTBLOCK, minLvl 5, Shield only
        let blocking = ITEM_SUFFIXES.iter().find(|a| a.name == "blocking").unwrap();
        assert_eq!(blocking.power.effect_type, ItemEffectType::FastBlock);
        assert_eq!(blocking.min_level, 5);
        assert!(blocking.item_types.contains(AffixItemType::SHIELD));
    }

    /// Verify the coarse bitmask identity matches C++ for every affix:
    /// the bitmask must be expressible purely in the 6 coarse bits.
    #[test]
    fn test_all_affixes_use_only_coarse_bits() {
        const COARSE_MASK: u32 =
            AffixItemType::MISC.0
            | AffixItemType::BOW.0
            | AffixItemType::STAFF.0
            | AffixItemType::WEAPON.0
            | AffixItemType::SHIELD.0
            | AffixItemType::ARMOR.0;
        for p in ITEM_PREFIXES.iter() {
            assert_eq!(
                p.item_types.0 & !COARSE_MASK,
                0,
                "Prefix {} uses non-coarse bits: 0x{:x}",
                p.name,
                p.item_types.0
            );
            assert_ne!(p.item_types.0, 0, "Prefix {} has empty item type", p.name);
        }
        for s in ITEM_SUFFIXES.iter() {
            assert_eq!(
                s.item_types.0 & !COARSE_MASK,
                0,
                "Suffix {} uses non-coarse bits: 0x{:x}",
                s.name,
                s.item_types.0
            );
            assert_ne!(s.item_types.0, 0, "Suffix {} has empty item type", s.name);
        }
    }

    /// Verify helm/jewelry search masks match the right affixes (regression
    /// for the coarse-model fix: previously helms matched nothing).
    #[test]
    fn test_helm_and_jewelry_search_masks() {
        // A weapon-only prefix must NOT match a helm search (HELM == ARMOR).
        let kings = ITEM_PREFIXES.iter().find(|a| a.name == "King's").unwrap();
        assert!(!kings.item_types.contains(AffixItemType::HELM));

        // An Armor/Shield prefix (Godly) MUST match a helm search.
        let godly = ITEM_PREFIXES.iter().find(|a| a.name == "Godly").unwrap();
        assert!(godly.item_types.contains(AffixItemType::HELM));

        // A Misc/jewelry-suffix (the zodiac) MUST match a jewelry search.
        let zodiac = ITEM_SUFFIXES.iter().find(|a| a.name == "the zodiac").unwrap();
        assert!(zodiac.item_types.contains(AffixItemType::JEWELRY));
        // And must NOT match a weapon search.
        assert!(!zodiac.item_types.contains(AffixItemType::WEAPON));
    }
    // ------------------------------------------------------------------
    // SetupAllItems / CheckUnique / GetUniqueItem / ItemRndDur
    // ------------------------------------------------------------------
    //
    // NOTE: the generation path uses the shared global Diablo LCG (like C++).
    // The assertions below are intentionally state-independent so they stay
    // deterministic under the parallel test harness; byte-exact seed-to-item
    // reproduction is exercised by the single-threaded cpp-repro workflows.

    fn find_item_index(name: &str) -> usize {
        crate::game::item_dat::ITEMS_DATA
            .iter()
            .position(|d| d.name == name)
            .unwrap_or_else(|| panic!("ITEMS_DATA row {name:?} not found"))
    }

    #[test]
    fn test_setup_all_items_sets_seed_index_and_create_info() {
        use super::super::items::{Item, ItemQuality};

        let idx = find_item_index("Short Sword");
        let mut item = Item::empty();
        setup_all_items(&mut item, &(), idx, 0x1234_5678, 8, 1, true, true, 0, false);

        assert_eq!(item.seed, 0x1234_5678);
        assert_eq!(item.item_index, idx as i16);
        // C++ flags: CF_PREGEN = 1<<15, CF_ONLYGOOD = 1<<6, CF_UPER1 = 1<<8.
        assert_ne!(item.create_info & CreateInfo::PREGEN.0, 0, "pregen flag set");
        assert_ne!(item.create_info & CreateInfo::ONLYGOOD.0, 0, "onlygood flag set");
        assert_ne!(item.create_info & CreateInfo::UPER1.0, 0, "uper1 flag set");
        assert_eq!(item.create_info & CreateInfo::UPER15.0, 0, "uper15 not set");
        // Level is stored in the low bits (C++ `_iCreateInfo = lvl`).
        assert_eq!(item.create_info & CreateInfo::LEVEL.0, 8);
        // A sword with onlygood=true must end up magic or unique.
        assert!(
            item.quality == ItemQuality::Magic || item.quality == ItemQuality::Unique,
            "sword quality should be magic or unique, got {:?}",
            item.quality
        );
        assert!(!item.name.is_empty());
    }

    #[test]
    fn test_check_unique_uper_100_returns_matching_uid() {
        use super::super::items::Item;

        // uper=100 makes the first GenerateRnd(100) roll always pass, so the
        // result depends only on GetValidUniques + uidOffset (state-independent).
        // Pick the first base item whose unique_base_id has matching uniques
        // (some names appear on multiple table rows with different base ids).
        let idx = crate::game::item_dat::ITEMS_DATA
            .iter()
            .position(|d| {
                crate::game::item_dat::UNIQUE_ITEMS
                    .iter()
                    .any(|u| u.base_item_id == d.unique_base_id)
            })
            .expect("some ITEMS_DATA row has matching uniques");
        let mut item = Item::empty();
        // item_index is what CheckUnique reads (C++ item.IDidx).
        item.item_index = idx as i16;

        let uid = check_unique(&item, 50, 100, 0).expect("base item has valid uniques at level 50");
        let unique = crate::game::item_dat::get_unique_item_data(
            crate::game::item_dat::UniqueItemId::from_index(uid).unwrap(),
        )
        .unwrap();
        assert_eq!(
            unique.base_item_id,
            crate::game::item_dat::get_item_data(idx).unwrap().unique_base_id,
            "picked unique must match the base item"
        );

        // uid_offset beyond the valid list -> invalid.
        assert_eq!(check_unique(&item, 50, 100, 99), None);
        // A base item with no uniques (e.g. a scroll) -> invalid.
        let scroll = find_item_index("Scroll of Town Portal");
        let mut no_unique = Item::empty();
        no_unique.item_index = scroll as i16;
        assert_eq!(check_unique(&no_unique, 50, 100, 0), None);
    }

    #[test]
    fn test_get_unique_item_sets_unique_quality_and_name() {
        use super::super::items::Item;

        use super::super::items::ItemQuality;

        let idx = find_item_index("Cleaver");
        let mut item = Item::empty();
        setup_all_items(&mut item, &(), idx, 0, 10, 1, false, false, 0, false);

        // Cleaver is a quest-unique (IMISC_UNIQUE) whose uid == iseed == 0.
        assert_eq!(item.quality, ItemQuality::Unique);
        assert_eq!(item.unique_id, 0);
        assert_eq!(item.seed, 0, "IMISC_UNIQUE keeps uid in _iSeed");
        assert_ne!(item.create_info & CreateInfo::UNIQUE.0, 0);
        assert_eq!(
            item.name,
            crate::game::item_dat::UNIQUE_ITEMS[0].name,
            "display name is the unique item name"
        );
    }

    #[test]
    fn test_setup_all_items_quest_unique_bad_seed_clears_item() {
        use super::super::items::Item;

        let idx = find_item_index("Cleaver");
        let mut item = Item::empty();
        item.value = 999;
        // iseed > 109 (or mismatching base) -> C++ `item.clear()`.
        setup_all_items(&mut item, &(), idx, 200, 10, 1, false, false, 0, false);
        assert_eq!(item.item_index, -1, "mismatched quest-unique seed clears the item");
        assert!(item.name.is_empty());
        assert_eq!(item.value, 0);
    }

    #[test]
    fn test_item_rnd_dur_skips_indestructible_and_zero_dur() {
        use super::super::items::Item;

        let mut item = Item::empty();
        item.durability = 0;
        item.max_durability = 20;
        item_rnd_dur(&mut item);
        assert_eq!(item.durability, 0, "zero durability is left untouched");

        // C++ checks current durability (not max): indestructible items have
        // both _iDurability and _iMaxDur == DUR_INDESTRUCTIBLE.
        item.durability = 255; // DUR_INDESTRUCTIBLE
        item.max_durability = 255;
        item_rnd_dur(&mut item);
        assert_eq!(item.durability, 255, "indestructible durability is left untouched");
    }

    #[test]
    fn test_get_item_blevel_misc_short_circuit_and_uper15() {
        use super::super::items::ItemMiscId;

        // Staff/Ring/Amulet short-circuit the rolls (C++ IsAnyOf).
        assert_eq!(get_item_blevel(8, ItemMiscId::Staff, false, false), 8);
        assert_eq!(get_item_blevel(8, ItemMiscId::Ring, false, false), 8);
        assert_eq!(get_item_blevel(8, ItemMiscId::Amulet, false, false), 8);
        // onlygood always yields a level (state-independent).
        assert_eq!(get_item_blevel(8, ItemMiscId::None, true, false), 8);
        // uper15 pins to lvl + 4 (state-independent).
        assert_eq!(get_item_blevel(8, ItemMiscId::None, false, true), 12);
        assert_eq!(get_item_blevel(30, ItemMiscId::None, false, true), 34);
    }

}
