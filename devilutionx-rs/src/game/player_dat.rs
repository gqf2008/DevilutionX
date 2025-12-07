//! Player Data - Exact port of DevilutionX Source/playerdat.hpp
//!
//! Contains class attributes, combat data, and starting loadouts.

use serde::{Deserialize, Serialize};

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

/// Get class attributes for a hero class
/// Values from Source/playerdat.cpp
pub fn get_class_attributes(class: HeroClass) -> ClassAttributes {
    match class {
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
            adj_life: 2048,  // 32 << 6 (fixed point)
            adj_mana: 0,
            lvl_life: 128,   // 2 << 6
            lvl_mana: 64,    // 1 << 6
            chr_life: 128,   // 2 << 6
            chr_mana: 64,    // 1 << 6
            itm_life: 128,   // 2 << 6
            itm_mana: 64,    // 1 << 6
        },
        HeroClass::Rogue => ClassAttributes {
            class_flags: PlayerClassFlag::None as u8,
            base_str: 20,
            base_mag: 15,
            base_dex: 30,
            base_vit: 20,
            max_str: 55,
            max_mag: 70,
            max_dex: 250,
            max_vit: 80,
            adj_life: 1024,  // 16 << 6
            adj_mana: 1408,  // 22 << 6
            lvl_life: 128,   // 2 << 6
            lvl_mana: 128,   // 2 << 6
            chr_life: 64,    // 1 << 6
            chr_mana: 128,   // 2 << 6
            itm_life: 64,    // 1 << 6
            itm_mana: 128,   // 2 << 6
        },
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
            adj_life: 576,   // 9 << 6
            adj_mana: 4480,  // 70 << 6
            lvl_life: 64,    // 1 << 6
            lvl_mana: 128,   // 2 << 6
            chr_life: 64,    // 1 << 6
            chr_mana: 128,   // 2 << 6
            itm_life: 64,    // 1 << 6
            itm_mana: 128,   // 2 << 6
        },
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
            adj_life: 1024,  // 16 << 6
            adj_mana: 1408,  // 22 << 6
            lvl_life: 128,   // 2 << 6
            lvl_mana: 128,   // 2 << 6
            chr_life: 96,    // 1.5 << 6
            chr_mana: 96,    // 1.5 << 6
            itm_life: 96,    // 1.5 << 6
            itm_mana: 96,    // 1.5 << 6
        },
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
            adj_life: 1024,  // 16 << 6
            adj_mana: 2240,  // 35 << 6
            lvl_life: 128,   // 2 << 6
            lvl_mana: 128,   // 2 << 6
            chr_life: 96,    // 1.5 << 6
            chr_mana: 96,    // 1.5 << 6
            itm_life: 96,    // 1.5 << 6
            itm_mana: 96,    // 1.5 << 6
        },
        HeroClass::Barbarian => ClassAttributes {
            class_flags: PlayerClassFlag::IronSkin as u8 | PlayerClassFlag::NaturalResistance as u8,
            base_str: 40,
            base_mag: 0,
            base_dex: 20,
            base_vit: 25,
            max_str: 255,
            max_mag: 0,
            max_dex: 55,
            max_vit: 150,
            adj_life: 2048,  // 32 << 6
            adj_mana: 0,
            lvl_life: 128,   // 2 << 6
            lvl_mana: 0,
            chr_life: 160,   // 2.5 << 6
            chr_mana: 0,
            itm_life: 160,   // 2.5 << 6
            itm_mana: 0,
        },
    }
}

/// Get combat data for a hero class
/// Values from Source/playerdat.cpp
pub fn get_player_combat_data(class: HeroClass) -> PlayerCombatData {
    match class {
        HeroClass::Warrior => PlayerCombatData {
            base_to_block: 30,
            base_melee_to_hit: 20,
            base_ranged_to_hit: 10,
            base_magic_to_hit: 50,
        },
        HeroClass::Rogue => PlayerCombatData {
            base_to_block: 20,
            base_melee_to_hit: 15,
            base_ranged_to_hit: 20,
            base_magic_to_hit: 50,
        },
        HeroClass::Sorcerer => PlayerCombatData {
            base_to_block: 10,
            base_melee_to_hit: 10,
            base_ranged_to_hit: 10,
            base_magic_to_hit: 20,
        },
        HeroClass::Monk => PlayerCombatData {
            base_to_block: 25,
            base_melee_to_hit: 15,
            base_ranged_to_hit: 15,
            base_magic_to_hit: 40,
        },
        HeroClass::Bard => PlayerCombatData {
            base_to_block: 20,
            base_melee_to_hit: 15,
            base_ranged_to_hit: 15,
            base_magic_to_hit: 35,
        },
        HeroClass::Barbarian => PlayerCombatData {
            base_to_block: 30,
            base_melee_to_hit: 20,
            base_ranged_to_hit: 10,
            base_magic_to_hit: 50,
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

    #[test]
    fn test_warrior_attributes() {
        let attr = get_class_attributes(HeroClass::Warrior);
        assert_eq!(attr.base_str, 30);
        assert_eq!(attr.base_mag, 10);
        assert_eq!(attr.base_dex, 20);
        assert_eq!(attr.base_vit, 25);
        assert_eq!(attr.max_str, 250);
        assert_eq!(attr.max_mag, 50);
        assert_eq!(attr.max_dex, 60);
        assert_eq!(attr.max_vit, 100);
    }

    #[test]
    fn test_rogue_attributes() {
        let attr = get_class_attributes(HeroClass::Rogue);
        assert_eq!(attr.base_str, 20);
        assert_eq!(attr.base_mag, 15);
        assert_eq!(attr.base_dex, 30);
        assert_eq!(attr.base_vit, 20);
        assert_eq!(attr.max_dex, 250);
    }

    #[test]
    fn test_sorcerer_attributes() {
        let attr = get_class_attributes(HeroClass::Sorcerer);
        assert_eq!(attr.base_str, 15);
        assert_eq!(attr.base_mag, 35);
        assert_eq!(attr.max_mag, 250);
    }

    #[test]
    fn test_hellfire_classes() {
        // Test Monk
        let monk = get_class_attributes(HeroClass::Monk);
        assert_eq!(monk.base_str, 25);
        assert_eq!(monk.base_mag, 15);
        assert_eq!(monk.max_str, 150);
        assert_eq!(monk.max_dex, 150);

        // Test Bard with DualWield flag
        let bard = get_class_attributes(HeroClass::Bard);
        assert_eq!(bard.class_flags, PlayerClassFlag::DualWield as u8);
        assert_eq!(bard.base_str, 20);
        assert_eq!(bard.base_mag, 20);

        // Test Barbarian with multiple flags
        let barb = get_class_attributes(HeroClass::Barbarian);
        assert_eq!(
            barb.class_flags,
            PlayerClassFlag::IronSkin as u8 | PlayerClassFlag::NaturalResistance as u8
        );
        assert_eq!(barb.base_str, 40);
        assert_eq!(barb.max_str, 255);
        assert_eq!(barb.base_mag, 0);
        assert_eq!(barb.max_mag, 0);
    }

    #[test]
    fn test_combat_data() {
        let warrior_combat = get_player_combat_data(HeroClass::Warrior);
        assert_eq!(warrior_combat.base_to_block, 30);
        assert_eq!(warrior_combat.base_melee_to_hit, 20);

        let rogue_combat = get_player_combat_data(HeroClass::Rogue);
        assert_eq!(rogue_combat.base_ranged_to_hit, 20);

        let sorc_combat = get_player_combat_data(HeroClass::Sorcerer);
        assert_eq!(sorc_combat.base_magic_to_hit, 20);
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
        // Test fixed-point life/mana values
        let warrior = get_class_attributes(HeroClass::Warrior);
        assert_eq!(warrior.adj_life, 2048);  // 32 << 6
        assert_eq!(warrior.lvl_life, 128);   // 2 << 6
        assert_eq!(warrior.chr_life, 128);   // 2 << 6

        let sorc = get_class_attributes(HeroClass::Sorcerer);
        assert_eq!(sorc.adj_mana, 4480);     // 70 << 6
        assert_eq!(sorc.lvl_mana, 128);      // 2 << 6

        // Test Monk's 1.5 multipliers
        let monk = get_class_attributes(HeroClass::Monk);
        assert_eq!(monk.chr_life, 96);       // 1.5 << 6
        assert_eq!(monk.chr_mana, 96);       // 1.5 << 6
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
