//! Player Data - Port of DevilutionX Source/playerdat.hpp + assets/txtdata/classes/*.tsv
//!
//! Class attributes, combat data, starting loadouts, and experience table.
//!
//! ## Data source
//!
//! The authoritative values live in TSV files under `assets/txtdata/classes/`:
//!   - `<class>/attributes.tsv` — `ClassAttributes` + `PlayerCombatData`
//!   - `<class>/starting_loadout.tsv` — `PlayerStartingLoadoutData`
//!   - `classdat.tsv` — `PlayerData` (name/folder/portrait/inv)
//!   - `../Experience.tsv` — `EXP_LEVELS`
//!
//! The C++ runtime parses these via `LoadClassData` / `parseFixed6` (see
//! `Source/playerdat.cpp` and `Source/utils/parse_int.cpp`). Fixed-point
//! fields use a 2.6 encoding: `value << 6` plus a rounded 1/64 fraction,
//! so `5.5` -> `5*64 + 32 = 352` and `-1` -> `-64`.

use serde::{Deserialize, Serialize};

/// Fixed-point (2.6) conversion of a decimal value, matching `ParseFixed6`.
///
/// `fixed6(int_part, frac_num, frac_den)` encodes `(int_part << 6) + round(frac_num/frac_den * 64)`,
/// rounding half-up. Negative numbers subtract the fraction (see `ParseFixed6`).
const fn fixed6(int_part: i32, frac_num: u32, frac_den: u32) -> i16 {
    // Round half-up to nearest 1/64. frac_den is a power of two in practice (1,2,4,8).
    let frac = (frac_num * 64 + frac_den / 2) / frac_den;
    if int_part >= 0 {
        ((int_part << 6) + frac as i32) as i16
    } else {
        ((int_part << 6) - frac as i32) as i16
    }
}

/// Hero classes - exact match of HeroClass enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum HeroClass {
    #[default]
    Warrior = 0,
    Rogue = 1,
    Sorcerer = 2,
    Monk = 3,      // Hellfire
    Bard = 4,      // Hellfire
    Barbarian = 5, // Hellfire
}

/// Player class flags - exact match of PlayerClassFlag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerClassFlag {
    None = 0,
    CriticalStrike = 1 << 0,
    DualWield = 1 << 1,
    IronSkin = 1 << 2,
    NaturalResistance = 1 << 3,
    TrapSense = 1 << 4,
}

/// Class attributes - exact match of ClassAttributes struct
#[derive(Debug, Clone)]
pub struct ClassAttributes {
    /// Class flags (bitfield of PlayerClassFlag)
    pub class_flags: u8,
    /// Starting strength
    pub base_str: u8,
    /// Starting magic
    pub base_mag: u8,
    /// Starting dexterity
    pub base_dex: u8,
    /// Starting vitality
    pub base_vit: u8,
    /// Maximum strength
    pub max_str: u8,
    /// Maximum magic
    pub max_mag: u8,
    /// Maximum dexterity
    pub max_dex: u8,
    /// Maximum vitality
    pub max_vit: u8,
    /// Life adjustment (added to base life)
    pub adj_life: i16,
    /// Mana adjustment (added to base mana)
    pub adj_mana: i16,
    /// Life gained per level
    pub lvl_life: i16,
    /// Mana gained per level
    pub lvl_mana: i16,
    /// Life from base vitality
    pub chr_life: i16,
    /// Mana from base magic
    pub chr_mana: i16,
    /// Life from item bonus vitality
    pub itm_life: i16,
    /// Mana from item bonus magic
    pub itm_mana: i16,
}

/// Player combat data - exact match of PlayerCombatData
#[derive(Debug, Clone)]
pub struct PlayerCombatData {
    /// Base block chance (percentage)
    pub base_to_block: u8,
    /// Base melee to-hit (percentage)
    pub base_melee_to_hit: u8,
    /// Base ranged to-hit (percentage)
    pub base_ranged_to_hit: u8,
    /// Base magic to-hit (percentage)
    pub base_magic_to_hit: u8,
}

/// Get class attributes for a hero class.
///
/// Values from `assets/txtdata/classes/<class>/attributes.tsv`, with the
/// fixed-point (2.6) fields decoded the same way C++ `parseFixed6` does.
pub fn get_class_attributes(class: HeroClass) -> ClassAttributes {
    match class {
        // attributes.tsv: adjLife 18, adjMana -1, lvl 2/1, chr 2/1, itm 2/1
        HeroClass::Warrior => ClassAttributes {
            class_flags: PlayerClassFlag::CriticalStrike as u8,
            base_str: 30,
            base_mag: 10,
            base_dex: 20,
            base_vit: 25,
            max_str: 250,
            max_mag: 50,
            max_dex: 60,
            max_vit: 100,
            adj_life: fixed6(18, 0, 1),   // 1152
            adj_mana: fixed6(-1, 0, 1),   // -64
            lvl_life: fixed6(2, 0, 1),    // 128
            lvl_mana: fixed6(1, 0, 1),    // 64
            chr_life: fixed6(2, 0, 1),    // 128
            chr_mana: fixed6(1, 0, 1),    // 64
            itm_life: fixed6(2, 0, 1),    // 128
            itm_mana: fixed6(1, 0, 1),    // 64
        },
        // attributes.tsv: adjLife 23, adjMana 5.5, lvl 2/2, chr 1/1, itm 1.5/1.5
        HeroClass::Rogue => ClassAttributes {
            class_flags: PlayerClassFlag::TrapSense as u8,
            base_str: 20,
            base_mag: 15,
            base_dex: 30,
            base_vit: 20,
            max_str: 55,
            max_mag: 70,
            max_dex: 250,
            max_vit: 80,
            adj_life: fixed6(23, 0, 1),   // 1472
            adj_mana: fixed6(5, 1, 2),    // 5.5 -> 352
            lvl_life: fixed6(2, 0, 1),    // 128
            lvl_mana: fixed6(2, 0, 1),    // 128
            chr_life: fixed6(1, 0, 1),    // 64
            chr_mana: fixed6(1, 0, 1),    // 64
            itm_life: fixed6(1, 1, 2),    // 1.5 -> 96
            itm_mana: fixed6(1, 1, 2),    // 1.5 -> 96
        },
        // attributes.tsv: adjLife 9, adjMana -2, lvl 1/2, chr 1/2, itm 1/2
        HeroClass::Sorcerer => ClassAttributes {
            class_flags: PlayerClassFlag::None as u8,
            base_str: 15,
            base_mag: 35,
            base_dex: 15,
            base_vit: 20,
            max_str: 45,
            max_mag: 250,
            max_dex: 85,
            max_vit: 80,
            adj_life: fixed6(9, 0, 1),    // 576
            adj_mana: fixed6(-2, 0, 1),   // -128
            lvl_life: fixed6(1, 0, 1),    // 64
            lvl_mana: fixed6(2, 0, 1),    // 128
            chr_life: fixed6(1, 0, 1),    // 64
            chr_mana: fixed6(2, 0, 1),    // 128
            itm_life: fixed6(1, 0, 1),    // 64
            itm_mana: fixed6(2, 0, 1),    // 128
        },
        // attributes.tsv: adjLife 23, adjMana 5.5, lvl 2/2, chr 1/1, itm 1.5/1.5
        HeroClass::Monk => ClassAttributes {
            class_flags: PlayerClassFlag::None as u8,
            base_str: 25,
            base_mag: 15,
            base_dex: 25,
            base_vit: 20,
            max_str: 150,
            max_mag: 80,
            max_dex: 150,
            max_vit: 80,
            adj_life: fixed6(23, 0, 1),   // 1472
            adj_mana: fixed6(5, 1, 2),    // 352
            lvl_life: fixed6(2, 0, 1),    // 128
            lvl_mana: fixed6(2, 0, 1),    // 128
            chr_life: fixed6(1, 0, 1),    // 64
            chr_mana: fixed6(1, 0, 1),    // 64
            itm_life: fixed6(1, 1, 2),    // 96
            itm_mana: fixed6(1, 1, 2),    // 96
        },
        // attributes.tsv: adjLife 23, adjMana 3, lvl 2/2, chr 1/1.5, itm 1.5/1.75
        HeroClass::Bard => ClassAttributes {
            class_flags: PlayerClassFlag::DualWield as u8,
            base_str: 20,
            base_mag: 20,
            base_dex: 25,
            base_vit: 20,
            max_str: 120,
            max_mag: 120,
            max_dex: 120,
            max_vit: 100,
            adj_life: fixed6(23, 0, 1),   // 1472
            adj_mana: fixed6(3, 0, 1),    // 192
            lvl_life: fixed6(2, 0, 1),    // 128
            lvl_mana: fixed6(2, 0, 1),    // 128
            chr_life: fixed6(1, 0, 1),    // 64
            chr_mana: fixed6(1, 1, 2),    // 1.5 -> 96
            itm_life: fixed6(1, 1, 2),    // 96
            itm_mana: fixed6(1, 3, 4),    // 1.75 -> 112
        },
        // attributes.tsv: adjLife 18, adjMana 0, lvl 2/0, chr 2/1, itm 2.5/1
        HeroClass::Barbarian => ClassAttributes {
            class_flags: PlayerClassFlag::CriticalStrike as u8
                | PlayerClassFlag::IronSkin as u8
                | PlayerClassFlag::NaturalResistance as u8,
            base_str: 40,
            base_mag: 0,
            base_dex: 20,
            base_vit: 25,
            max_str: 255,
            max_mag: 0,
            max_dex: 55,
            max_vit: 150,
            adj_life: fixed6(18, 0, 1),   // 1152
            adj_mana: fixed6(0, 0, 1),    // 0
            lvl_life: fixed6(2, 0, 1),    // 128
            lvl_mana: fixed6(0, 0, 1),    // 0
            chr_life: fixed6(2, 0, 1),    // 128
            chr_mana: fixed6(1, 0, 1),    // 64
            itm_life: fixed6(2, 1, 2),    // 2.5 -> 160
            itm_mana: fixed6(1, 0, 1),    // 64
        },
    }
}

/// Get combat data for a hero class.
///
/// `base_to_block` comes from `blockBonus`; the three to-hit fields come from
/// `baseMagicToHit` / `baseMeleeToHit` / `baseRangedToHit` in
/// `assets/txtdata/classes/<class>/attributes.tsv`.
pub fn get_player_combat_data(class: HeroClass) -> PlayerCombatData {
    match class {
        HeroClass::Warrior => PlayerCombatData {
            base_to_block: 30,
            base_melee_to_hit: 70,
            base_ranged_to_hit: 60,
            base_magic_to_hit: 50,
        },
        HeroClass::Rogue => PlayerCombatData {
            base_to_block: 20,
            base_melee_to_hit: 50,
            base_ranged_to_hit: 70,
            base_magic_to_hit: 50,
        },
        HeroClass::Sorcerer => PlayerCombatData {
            base_to_block: 10,
            base_melee_to_hit: 50,
            base_ranged_to_hit: 50,
            base_magic_to_hit: 70,
        },
        HeroClass::Monk => PlayerCombatData {
            base_to_block: 25,
            base_melee_to_hit: 50,
            base_ranged_to_hit: 50,
            base_magic_to_hit: 50,
        },
        HeroClass::Bard => PlayerCombatData {
            base_to_block: 25,
            base_melee_to_hit: 50,
            base_ranged_to_hit: 60,
            base_magic_to_hit: 60,
        },
        HeroClass::Barbarian => PlayerCombatData {
            base_to_block: 30,
            base_melee_to_hit: 50,
            base_ranged_to_hit: 50,
            base_magic_to_hit: 50,
        },
    }
}

/// Player metadata - exact match of PlayerData
///
/// Values from `assets/txtdata/classes/classdat.tsv`.
#[derive(Debug, Clone)]
pub struct PlayerData {
    /// Class Name
    pub class_name: &'static str,
    /// Class Folder Name
    pub folder_name: &'static str,
    /// Class Portrait Index
    pub portrait: u8,
    /// Class Inventory UI File
    pub inv: &'static str,
}

/// Get player metadata (name/folder/portrait/inv) for a hero class.
///
/// Values from `assets/txtdata/classes/classdat.tsv`.
pub fn get_player_data_for_class(class: HeroClass) -> PlayerData {
    match class {
        HeroClass::Warrior => PlayerData {
            class_name: "Warrior",
            folder_name: "warrior",
            portrait: 0,
            inv: "inv",
        },
        HeroClass::Rogue => PlayerData {
            class_name: "Rogue",
            folder_name: "rogue",
            portrait: 1,
            inv: "inv_rog",
        },
        HeroClass::Sorcerer => PlayerData {
            class_name: "Sorcerer",
            folder_name: "sorcerer",
            portrait: 2,
            inv: "inv_sor",
        },
        HeroClass::Monk => PlayerData {
            class_name: "Monk",
            folder_name: "monk",
            portrait: 2,
            inv: "inv_sor",
        },
        HeroClass::Bard => PlayerData {
            class_name: "Bard",
            folder_name: "bard",
            portrait: 1,
            inv: "inv_rog",
        },
        HeroClass::Barbarian => PlayerData {
            class_name: "Barbarian",
            folder_name: "barbarian",
            portrait: 0,
            inv: "inv",
        },
    }
}

/// Known class skill / starting spell identifiers.
///
/// Subset of `SpellID` relevant to starting loadouts, from
/// `assets/txtdata/classes/<class>/starting_loadout.tsv` (`skill` / `spell`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartSpell {
    Null,
    ItemRepair,       // Warrior skill
    TrapDisarm,       // Rogue skill
    StaffRecharge,    // Sorcerer skill
    Search,           // Monk skill
    Identify,         // Bard skill
    Rage,             // Barbarian skill
    Firebolt,         // Sorcerer starting spell
}

/// Starting item index identifiers.
///
/// Subset of `_item_indexes` (`IDI_*`) relevant to starting loadouts.
/// Values from `assets/txtdata/classes/<class>/starting_loadout.tsv` (`item0`..`item4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartItem {
    None,
    // Warrior
    IdiWarrior,
    IdiWarrshld,
    IdiWarrclub,
    // Rogue
    IdiRogue,
    // Sorcerer
    IdiSorcererDiablo,
    // Monk
    IdiShortstaff,
    // Bard
    IdiBardsword,
    IdiBarddagger,
    // Barbarian
    IdiBarbarian,
    // Consumables shared across classes
    IdiHeal,
    IdiMana,
}

impl StartItem {
    /// String identifier as it appears in the TSV (e.g. `"IDI_WARRIOR"`).
    pub fn tsv_id(self) -> &'static str {
        match self {
            StartItem::None => "IDI_NONE",
            StartItem::IdiWarrior => "IDI_WARRIOR",
            StartItem::IdiWarrshld => "IDI_WARRSHLD",
            StartItem::IdiWarrclub => "IDI_WARRCLUB",
            StartItem::IdiRogue => "IDI_ROGUE",
            StartItem::IdiSorcererDiablo => "IDI_SORCERER_DIABLO",
            StartItem::IdiShortstaff => "IDI_SHORTSTAFF",
            StartItem::IdiBardsword => "IDI_BARDSWORD",
            StartItem::IdiBarddagger => "IDI_BARDDAGGER",
            StartItem::IdiBarbarian => "IDI_BARBARIAN",
            StartItem::IdiHeal => "IDI_HEAL",
            StartItem::IdiMana => "IDI_MANA",
        }
    }
}

/// Starting loadout - exact match of PlayerStartingLoadoutData
///
/// Values from `assets/txtdata/classes/<class>/starting_loadout.tsv`.
#[derive(Debug, Clone)]
pub struct PlayerStartingLoadoutData {
    /// Class Skill
    pub skill: StartSpell,
    /// Starting Spell (Null if none)
    pub spell: StartSpell,
    /// Initial level of the starting spell
    pub spell_level: u8,
    /// Initial items (created in order; 5 slots, padded with None)
    pub items: [StartItem; 5],
    /// Initial gold amount (up to a single 5000-gold stack)
    pub gold: u16,
}

/// Get the starting loadout (skill, spell, items, gold) for a hero class.
///
/// Values from `assets/txtdata/classes/<class>/starting_loadout.tsv`.
pub fn get_player_starting_loadout(class: HeroClass) -> PlayerStartingLoadoutData {
    match class {
        HeroClass::Warrior => PlayerStartingLoadoutData {
            skill: StartSpell::ItemRepair,
            spell: StartSpell::Null,
            spell_level: 0,
            items: [
                StartItem::IdiWarrior,
                StartItem::IdiWarrshld,
                StartItem::IdiWarrclub,
                StartItem::IdiHeal,
                StartItem::IdiHeal,
            ],
            gold: 100,
        },
        HeroClass::Rogue => PlayerStartingLoadoutData {
            skill: StartSpell::TrapDisarm,
            spell: StartSpell::Null,
            spell_level: 0,
            items: [
                StartItem::IdiRogue,
                StartItem::IdiHeal,
                StartItem::IdiHeal,
                StartItem::None,
                StartItem::None,
            ],
            gold: 100,
        },
        HeroClass::Sorcerer => PlayerStartingLoadoutData {
            skill: StartSpell::StaffRecharge,
            spell: StartSpell::Firebolt,
            spell_level: 2,
            items: [
                StartItem::IdiSorcererDiablo,
                StartItem::IdiMana,
                StartItem::IdiMana,
                StartItem::None,
                StartItem::None,
            ],
            gold: 100,
        },
        HeroClass::Monk => PlayerStartingLoadoutData {
            skill: StartSpell::Search,
            spell: StartSpell::Null,
            spell_level: 0,
            items: [
                StartItem::IdiShortstaff,
                StartItem::IdiHeal,
                StartItem::IdiHeal,
                StartItem::None,
                StartItem::None,
            ],
            gold: 100,
        },
        HeroClass::Bard => PlayerStartingLoadoutData {
            skill: StartSpell::Identify,
            spell: StartSpell::Null,
            spell_level: 0,
            items: [
                StartItem::IdiBardsword,
                StartItem::IdiBarddagger,
                StartItem::IdiHeal,
                StartItem::IdiHeal,
                StartItem::None,
            ],
            gold: 100,
        },
        HeroClass::Barbarian => PlayerStartingLoadoutData {
            skill: StartSpell::Rage,
            spell: StartSpell::Null,
            spell_level: 0,
            items: [
                StartItem::IdiBarbarian,
                StartItem::IdiWarrshld,
                StartItem::IdiHeal,
                StartItem::IdiHeal,
                StartItem::None,
            ],
            gold: 100,
        },
    }
}

/// Experience thresholds per level (1-50)
/// From assets/txtdata/Experience.tsv
pub const EXP_LEVELS: [u32; 50] = [
    2000,        // Level 1
    4620,        // Level 2
    8040,        // Level 3
    12489,       // Level 4
    18258,       // Level 5
    25712,       // Level 6
    35309,       // Level 7
    47622,       // Level 8
    63364,       // Level 9
    83419,       // Level 10
    108879,      // Level 11
    141086,      // Level 12
    181683,      // Level 13
    231075,      // Level 14
    313656,      // Level 15
    424067,      // Level 16
    571190,      // Level 17
    766569,      // Level 18
    1025154,     // Level 19
    1366227,     // Level 20
    1814568,     // Level 21
    2401895,     // Level 22
    3168651,     // Level 23
    4166200,     // Level 24
    5459523,     // Level 25
    7130496,     // Level 26
    9281874,     // Level 27
    12042092,    // Level 28
    15571031,    // Level 29
    20066900,    // Level 30
    25774405,    // Level 31
    32994399,    // Level 32
    42095202,    // Level 33
    53525811,    // Level 34
    67831218,    // Level 35
    85670061,    // Level 36
    107834823,   // Level 37
    135274799,   // Level 38
    169122009,   // Level 39
    210720231,   // Level 40
    261657253,   // Level 41
    323800420,   // Level 42
    399335440,   // Level 43
    490808349,   // Level 44
    601170414,   // Level 45
    733825617,   // Level 46
    892680222,   // Level 47
    1082908612,  // Level 48
    1310707109,  // Level 49
    1583495809,  // Level 50
];

/// Get experience threshold for a given level (1-50)
/// Returns 0 for level 0, max u32 for levels beyond 50
pub fn get_next_experience_threshold(level: u32) -> u32 {
    match level {
        0 => 0,
        1..=50 => EXP_LEVELS[(level - 1) as usize],
        _ => u32::MAX,
    }
}

/// Get current level from experience points
/// Returns 0 for 0 exp, capped at MAX_CHARACTER_LEVEL
pub fn get_level_from_experience(exp: u32) -> u8 {
    if exp == 0 {
        return 0;
    }

    for (idx, &threshold) in EXP_LEVELS.iter().enumerate() {
        if exp < threshold {
            return idx as u8;
        }
    }

    MAX_CHARACTER_LEVEL
}

/// Maximum character level
pub const MAX_CHARACTER_LEVEL: u8 = 50;

/// Maximum resistance value
pub const MAX_RESISTANCE: i32 = 75;

/// Maximum spell level
pub const MAX_SPELL_LEVEL: u8 = 15;

/// Player name max length
pub const PLAYER_NAME_LENGTH: usize = 32;

/// Inventory grid cells
pub const INVENTORY_GRID_CELLS: usize = 40;

/// Max belt items
pub const MAX_BELT_ITEMS: usize = 8;

/// Number of hotkeys
pub const NUM_HOTKEYS: usize = 12;

/// Number of inventory body locations
pub const NUM_INV_LOC: usize = 7;

#[cfg(test)]
mod tests {
    use super::*;
    /// Class base/max attributes + combat data must match the upstream
    /// `classes/<class>/attributes.tsv` rows (baseStr/baseMag/baseDex/baseVit,
    /// maxStr/maxMag/maxDex/maxVit, blockBonus, baseMelee/Ranged/MagicToHit).
    #[test]
    fn test_class_attributes_match_tsv() {
        // Warrior: 30/10/20/25, max 250/50/60/100, block 30, melee 70, ranged 60, magic 50.
        let w = get_class_attributes(HeroClass::Warrior);
        assert_eq!((w.base_str, w.base_mag, w.base_dex, w.base_vit), (30, 10, 20, 25));
        assert_eq!((w.max_str, w.max_mag, w.max_dex, w.max_vit), (250, 50, 60, 100));
        let wc = get_player_combat_data(HeroClass::Warrior);
        assert_eq!((wc.base_to_block, wc.base_melee_to_hit, wc.base_ranged_to_hit, wc.base_magic_to_hit), (30, 70, 60, 50));
        // Barbarian: 40/0/20/25, max 255/0/55/150, block 30, melee/ranged/magic 50.
        let b = get_class_attributes(HeroClass::Barbarian);
        assert_eq!((b.base_str, b.base_mag, b.base_dex, b.base_vit), (40, 0, 20, 25));
        assert_eq!((b.max_str, b.max_mag, b.max_dex, b.max_vit), (255, 0, 55, 150));
        // Rogue: 20/15/30/20, max 55/70/250/80, block 20, melee 50, ranged 70, magic 50.
        let r = get_class_attributes(HeroClass::Rogue);
        assert_eq!((r.base_str, r.base_mag, r.base_dex, r.base_vit), (20, 15, 30, 20));
        assert_eq!((r.max_str, r.max_mag, r.max_dex, r.max_vit), (55, 70, 250, 80));
        let rc = get_player_combat_data(HeroClass::Rogue);
        assert_eq!((rc.base_to_block, rc.base_melee_to_hit, rc.base_ranged_to_hit, rc.base_magic_to_hit), (20, 50, 70, 50));
    }


    #[test]
    fn test_fixed6_encoding() {
        // Sanity for the fixed6() helper, matching C++ parseFixed6.
        assert_eq!(fixed6(0, 0, 1), 0);
        assert_eq!(fixed6(1, 0, 1), 64);
        assert_eq!(fixed6(2, 0, 1), 128);
        assert_eq!(fixed6(-1, 0, 1), -64);
        assert_eq!(fixed6(-2, 0, 1), -128);
        // 0.5 -> 32, 0.75 -> 48, 0.25 -> 16
        assert_eq!(fixed6(1, 1, 2), 64 + 32); // 96
        assert_eq!(fixed6(1, 3, 4), 64 + 48); // 112
        assert_eq!(fixed6(2, 1, 2), 128 + 32); // 160
        assert_eq!(fixed6(5, 1, 2), 5 * 64 + 32); // 352
    }

    #[test]
    fn test_warrior_attributes() {
        let attr = get_class_attributes(HeroClass::Warrior);
        assert_eq!(attr.class_flags, PlayerClassFlag::CriticalStrike as u8);
        assert_eq!(attr.base_str, 30);
        assert_eq!(attr.base_mag, 10);
        assert_eq!(attr.base_dex, 20);
        assert_eq!(attr.base_vit, 25);
        assert_eq!(attr.max_str, 250);
        assert_eq!(attr.max_mag, 50);
        assert_eq!(attr.max_dex, 60);
        assert_eq!(attr.max_vit, 100);
        // adjLife 18, adjMana -1, lvl 2/1, chr 2/1, itm 2/1
        assert_eq!(attr.adj_life, 18 * 64);
        assert_eq!(attr.adj_mana, -64);
        assert_eq!(attr.lvl_life, 128);
        assert_eq!(attr.lvl_mana, 64);
        assert_eq!(attr.chr_life, 128);
        assert_eq!(attr.chr_mana, 64);
        assert_eq!(attr.itm_life, 128);
        assert_eq!(attr.itm_mana, 64);
    }

    #[test]
    fn test_rogue_attributes() {
        let attr = get_class_attributes(HeroClass::Rogue);
        // TSV: classFlags = TrapSense
        assert_eq!(attr.class_flags, PlayerClassFlag::TrapSense as u8);
        assert_eq!(attr.base_str, 20);
        assert_eq!(attr.base_mag, 15);
        assert_eq!(attr.base_dex, 30);
        assert_eq!(attr.base_vit, 20);
        assert_eq!(attr.max_str, 55);
        assert_eq!(attr.max_mag, 70);
        assert_eq!(attr.max_dex, 250);
        assert_eq!(attr.max_vit, 80);
        // adjLife 23, adjMana 5.5, lvl 2/2, chr 1/1, itm 1.5/1.5
        assert_eq!(attr.adj_life, 23 * 64);
        assert_eq!(attr.adj_mana, 5 * 64 + 32);
        assert_eq!(attr.lvl_life, 128);
        assert_eq!(attr.lvl_mana, 128);
        assert_eq!(attr.chr_life, 64);
        assert_eq!(attr.chr_mana, 64);
        assert_eq!(attr.itm_life, 96);
        assert_eq!(attr.itm_mana, 96);
    }

    #[test]
    fn test_sorcerer_attributes() {
        let attr = get_class_attributes(HeroClass::Sorcerer);
        assert_eq!(attr.class_flags, PlayerClassFlag::None as u8);
        assert_eq!(attr.base_str, 15);
        assert_eq!(attr.base_mag, 35);
        assert_eq!(attr.base_dex, 15);
        assert_eq!(attr.base_vit, 20);
        assert_eq!(attr.max_str, 45);
        assert_eq!(attr.max_mag, 250);
        assert_eq!(attr.max_dex, 85);
        assert_eq!(attr.max_vit, 80);
        // adjLife 9, adjMana -2, lvl 1/2, chr 1/2, itm 1/2
        assert_eq!(attr.adj_life, 9 * 64);
        assert_eq!(attr.adj_mana, -128);
        assert_eq!(attr.lvl_life, 64);
        assert_eq!(attr.lvl_mana, 128);
        assert_eq!(attr.chr_life, 64);
        assert_eq!(attr.chr_mana, 128);
        assert_eq!(attr.itm_life, 64);
        assert_eq!(attr.itm_mana, 128);
    }

    #[test]
    fn test_hellfire_classes() {
        // Monk
        let monk = get_class_attributes(HeroClass::Monk);
        assert_eq!(monk.class_flags, PlayerClassFlag::None as u8);
        assert_eq!(monk.base_str, 25);
        assert_eq!(monk.base_mag, 15);
        assert_eq!(monk.base_dex, 25);
        assert_eq!(monk.base_vit, 20);
        assert_eq!(monk.max_str, 150);
        assert_eq!(monk.max_mag, 80);
        assert_eq!(monk.max_dex, 150);
        assert_eq!(monk.max_vit, 80);
        // adjLife 23, adjMana 5.5, lvl 2/2, chr 1/1, itm 1.5/1.5
        assert_eq!(monk.adj_life, 23 * 64);
        assert_eq!(monk.adj_mana, 5 * 64 + 32);
        assert_eq!(monk.itm_life, 96);
        assert_eq!(monk.itm_mana, 96);

        // Bard with DualWield
        let bard = get_class_attributes(HeroClass::Bard);
        assert_eq!(bard.class_flags, PlayerClassFlag::DualWield as u8);
        assert_eq!(bard.base_str, 20);
        assert_eq!(bard.base_mag, 20);
        // adjLife 23, adjMana 3, lvl 2/2, chr 1/1.5, itm 1.5/1.75
        assert_eq!(bard.adj_life, 23 * 64);
        assert_eq!(bard.adj_mana, 3 * 64);
        assert_eq!(bard.chr_mana, 96); // 1.5
        assert_eq!(bard.itm_life, 96); // 1.5
        assert_eq!(bard.itm_mana, 112); // 1.75

        // Barbarian: CriticalStrike|IronSkin|NaturalResistance
        let barb = get_class_attributes(HeroClass::Barbarian);
        assert_eq!(
            barb.class_flags,
            PlayerClassFlag::CriticalStrike as u8
                | PlayerClassFlag::IronSkin as u8
                | PlayerClassFlag::NaturalResistance as u8
        );
        assert_eq!(barb.base_str, 40);
        assert_eq!(barb.base_mag, 0);
        assert_eq!(barb.max_str, 255);
        assert_eq!(barb.max_mag, 0);
        // adjLife 18, adjMana 0, lvl 2/0, chr 2/1, itm 2.5/1
        assert_eq!(barb.adj_life, 18 * 64);
        assert_eq!(barb.adj_mana, 0);
        assert_eq!(barb.lvl_mana, 0);
        assert_eq!(barb.chr_mana, 64);
        assert_eq!(barb.itm_life, 160); // 2.5
        assert_eq!(barb.itm_mana, 64);
    }

    #[test]
    fn test_combat_data() {
        // blockBonus / baseMagicToHit / baseMeleeToHit / baseRangedToHit
        let w = get_player_combat_data(HeroClass::Warrior);
        assert_eq!(w.base_to_block, 30);
        assert_eq!(w.base_magic_to_hit, 50);
        assert_eq!(w.base_melee_to_hit, 70);
        assert_eq!(w.base_ranged_to_hit, 60);

        let r = get_player_combat_data(HeroClass::Rogue);
        assert_eq!(r.base_to_block, 20);
        assert_eq!(r.base_ranged_to_hit, 70);
        assert_eq!(r.base_magic_to_hit, 50);
        assert_eq!(r.base_melee_to_hit, 50);

        let s = get_player_combat_data(HeroClass::Sorcerer);
        assert_eq!(s.base_to_block, 10);
        assert_eq!(s.base_magic_to_hit, 70);
        assert_eq!(s.base_melee_to_hit, 50);
        assert_eq!(s.base_ranged_to_hit, 50);

        let m = get_player_combat_data(HeroClass::Monk);
        assert_eq!(m.base_to_block, 25);
        assert_eq!(m.base_magic_to_hit, 50);

        let b = get_player_combat_data(HeroClass::Bard);
        assert_eq!(b.base_to_block, 25);
        assert_eq!(b.base_magic_to_hit, 60);
        assert_eq!(b.base_ranged_to_hit, 60);

        let bar = get_player_combat_data(HeroClass::Barbarian);
        assert_eq!(bar.base_to_block, 30);
        assert_eq!(bar.base_melee_to_hit, 50);
    }

    #[test]
    fn test_player_data_for_class() {
        // classdat.tsv
        let w = get_player_data_for_class(HeroClass::Warrior);
        assert_eq!(w.class_name, "Warrior");
        assert_eq!(w.folder_name, "warrior");
        assert_eq!(w.portrait, 0);
        assert_eq!(w.inv, "inv");

        let r = get_player_data_for_class(HeroClass::Rogue);
        assert_eq!(r.portrait, 1);
        assert_eq!(r.inv, "inv_rog");

        let s = get_player_data_for_class(HeroClass::Sorcerer);
        assert_eq!(s.portrait, 2);
        assert_eq!(s.inv, "inv_sor");

        // Monk reuses sorcerer portrait/inv
        let m = get_player_data_for_class(HeroClass::Monk);
        assert_eq!(m.portrait, 2);
        assert_eq!(m.inv, "inv_sor");

        // Bard reuses rogue portrait/inv
        let b = get_player_data_for_class(HeroClass::Bard);
        assert_eq!(b.portrait, 1);
        assert_eq!(b.inv, "inv_rog");

        let bar = get_player_data_for_class(HeroClass::Barbarian);
        assert_eq!(bar.portrait, 0);
        assert_eq!(bar.inv, "inv");
    }

    #[test]
    fn test_starting_loadouts() {
        // Warrior: skill ItemRepair, 3 items + 2 heal, gold 100
        let w = get_player_starting_loadout(HeroClass::Warrior);
        assert_eq!(w.skill, StartSpell::ItemRepair);
        assert_eq!(w.spell, StartSpell::Null);
        assert_eq!(w.spell_level, 0);
        assert_eq!(w.gold, 100);
        assert_eq!(w.items[0].tsv_id(), "IDI_WARRIOR");
        assert_eq!(w.items[1].tsv_id(), "IDI_WARRSHLD");
        assert_eq!(w.items[2].tsv_id(), "IDI_WARRCLUB");
        assert_eq!(w.items[3], StartItem::IdiHeal);
        assert_eq!(w.items[4], StartItem::IdiHeal);

        // Rogue: skill TrapDisarm, 1 item + 2 heal, 2 None
        let r = get_player_starting_loadout(HeroClass::Rogue);
        assert_eq!(r.skill, StartSpell::TrapDisarm);
        assert_eq!(r.items[0], StartItem::IdiRogue);
        assert_eq!(r.items[3], StartItem::None);
        assert_eq!(r.items[4], StartItem::None);

        // Sorcerer: skill StaffRecharge, spell Firebolt level 2, 2 mana
        let s = get_player_starting_loadout(HeroClass::Sorcerer);
        assert_eq!(s.skill, StartSpell::StaffRecharge);
        assert_eq!(s.spell, StartSpell::Firebolt);
        assert_eq!(s.spell_level, 2);
        assert_eq!(s.items[0], StartItem::IdiSorcererDiablo);
        assert_eq!(s.items[1], StartItem::IdiMana);
        assert_eq!(s.items[2], StartItem::IdiMana);

        // Monk: skill Search, shortstaff + 2 heal
        let m = get_player_starting_loadout(HeroClass::Monk);
        assert_eq!(m.skill, StartSpell::Search);
        assert_eq!(m.items[0], StartItem::IdiShortstaff);

        // Bard: skill Identify, sword + dagger + 2 heal
        let b = get_player_starting_loadout(HeroClass::Bard);
        assert_eq!(b.skill, StartSpell::Identify);
        assert_eq!(b.items[0], StartItem::IdiBardsword);
        assert_eq!(b.items[1], StartItem::IdiBarddagger);

        // Barbarian: skill Rage, barbarian + warrshld + 2 heal
        let bar = get_player_starting_loadout(HeroClass::Barbarian);
        assert_eq!(bar.skill, StartSpell::Rage);
        assert_eq!(bar.items[0], StartItem::IdiBarbarian);
        assert_eq!(bar.items[1], StartItem::IdiWarrshld);

        // All classes start with 100 gold.
        for c in [
            HeroClass::Warrior,
            HeroClass::Rogue,
            HeroClass::Sorcerer,
            HeroClass::Monk,
            HeroClass::Bard,
            HeroClass::Barbarian,
        ] {
            assert_eq!(get_player_starting_loadout(c).gold, 100, "gold mismatch");
            // All loadouts use all 5 item slots.
            assert_eq!(
                get_player_starting_loadout(c).items.len(),
                5,
                "items array must be 5 slots"
            );
        }
    }

    #[test]
    fn test_experience_thresholds() {
        // Level 0 should return 0
        assert_eq!(get_next_experience_threshold(0), 0);

        // Test exact values from Experience.tsv
        assert_eq!(get_next_experience_threshold(1), 2000);
        assert_eq!(get_next_experience_threshold(2), 4620);
        assert_eq!(get_next_experience_threshold(10), 83419);
        assert_eq!(get_next_experience_threshold(25), 5459523);
        assert_eq!(get_next_experience_threshold(50), 1583495809);

        // Beyond max level should return max
        assert_eq!(get_next_experience_threshold(51), u32::MAX);
        assert_eq!(get_next_experience_threshold(100), u32::MAX);
    }

    #[test]
    fn test_level_from_experience() {
        // 0 exp = level 0
        assert_eq!(get_level_from_experience(0), 0);

        // Just below level 1 threshold
        assert_eq!(get_level_from_experience(1999), 0);

        // At level 1 threshold
        assert_eq!(get_level_from_experience(2000), 1);

        // Between level 1 and 2
        assert_eq!(get_level_from_experience(3000), 1);

        // At level 10
        assert_eq!(get_level_from_experience(83419), 10);

        // Max level
        assert_eq!(get_level_from_experience(1583495809), 50);
        assert_eq!(get_level_from_experience(u32::MAX), 50);
    }

    #[test]
    fn test_exp_levels_array() {
        // Verify array length
        assert_eq!(EXP_LEVELS.len(), 50);

        // Verify monotonically increasing
        for i in 1..EXP_LEVELS.len() {
            assert!(
                EXP_LEVELS[i] > EXP_LEVELS[i - 1],
                "Level {} exp ({}) should be greater than level {} exp ({})",
                i + 1,
                EXP_LEVELS[i],
                i,
                EXP_LEVELS[i - 1]
            );
        }
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CHARACTER_LEVEL, 50);
        assert_eq!(MAX_RESISTANCE, 75);
        assert_eq!(MAX_SPELL_LEVEL, 15);
        assert_eq!(PLAYER_NAME_LENGTH, 32);
        assert_eq!(INVENTORY_GRID_CELLS, 40);
        assert_eq!(MAX_BELT_ITEMS, 8);
        assert_eq!(NUM_HOTKEYS, 12);
        assert_eq!(NUM_INV_LOC, 7);
    }

    #[test]
    fn test_class_life_mana_growth() {
        // Fixed-point (2.6) values decoded from attributes.tsv decimals.
        let warrior = get_class_attributes(HeroClass::Warrior);
        assert_eq!(warrior.adj_life, 18 * 64); // 18 -> 1152
        assert_eq!(warrior.adj_mana, -64); // -1 -> -64
        assert_eq!(warrior.lvl_life, 2 * 64); // 2
        assert_eq!(warrior.chr_life, 2 * 64); // 2

        let sorc = get_class_attributes(HeroClass::Sorcerer);
        assert_eq!(sorc.adj_mana, -128); // -2 -> -128
        assert_eq!(sorc.lvl_mana, 2 * 64); // 2

        // Rogue's 5.5 adjMana and 1.5 itm multipliers
        let rogue = get_class_attributes(HeroClass::Rogue);
        assert_eq!(rogue.adj_mana, 5 * 64 + 32); // 5.5 -> 352
        assert_eq!(rogue.itm_life, 96); // 1.5

        // Bard's 1.75 itmMana multiplier
        let bard = get_class_attributes(HeroClass::Bard);
        assert_eq!(bard.itm_mana, 112); // 1.75 -> 112

        // Barbarian's 2.5 itmLife multiplier
        let barb = get_class_attributes(HeroClass::Barbarian);
        assert_eq!(barb.itm_life, 160); // 2.5 -> 160
        assert_eq!(barb.lvl_mana, 0); // 0
    }

    #[test]
    fn test_hero_class_enum() {
        // Test enum values match C++
        assert_eq!(HeroClass::Warrior as u8, 0);
        assert_eq!(HeroClass::Rogue as u8, 1);
        assert_eq!(HeroClass::Sorcerer as u8, 2);
        assert_eq!(HeroClass::Monk as u8, 3);
        assert_eq!(HeroClass::Bard as u8, 4);
        assert_eq!(HeroClass::Barbarian as u8, 5);
    }
}
