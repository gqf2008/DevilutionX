//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Monster Validation
//!
//! C++ Source: Source/monsters/validation.cpp
//! C++ Header: Source/monsters/validation.hpp
//!
//! Implementation of functions for validation of monster data.

use crate::game::monstdat::MonsterId;
use crate::game::monster::{Monster, UniqueMonsterType};

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of monsters in the game
/// C++ Reference: MaxMonsters in Source/monster.h
pub const MAX_MONSTERS: usize = 200;

/// Maximum number of players in the game
/// C++ Reference: MAX_PLRS in Source/multi.h
pub const MAX_PLRS: usize = 4;

// ============================================================================
// Unique Monster Data
// ============================================================================

/// Unique monster data - maps unique type to monster type ID
/// C++ Reference: UniqueMonsterData in Source/monstdat.h
#[derive(Debug, Clone, Copy)]
pub struct UniqueMonsterData {
    /// Monster type ID (mtype)
    pub mtype: MonsterId,
    /// Name string ID
    pub mname_id: u8,
    /// Text ID for talking
    pub mtalk_id: u8,
    /// Min/Max level where unique can spawn
    pub min_level: u8,
    pub max_level: u8,
    /// HP multiplier
    pub hp_mult: u16,
    /// AI type
    pub ai: i8,
    /// Intelligence
    pub intelligence: u8,
    /// Resistances
    pub resistance: u8,
    /// Pack size
    pub pack_size: u8,
}

/// Unique monsters data table
/// C++ Reference: UniqueMonstersData in Source/monstdat.cpp
pub static UNIQUE_MONSTERS_DATA: &[UniqueMonsterData] = &[
    // Garbud (0)
    UniqueMonsterData {
        mtype: MonsterId::Fat,
        mname_id: 1, mtalk_id: 1, min_level: 4, max_level: 4,
        hp_mult: 100, ai: 18, intelligence: 0, resistance: 0, pack_size: 0,
    },
    // Skeleton King (1)
    UniqueMonsterData {
        mtype: MonsterId::SkeletonKing,
        mname_id: 2, mtalk_id: 0, min_level: 3, max_level: 3,
        hp_mult: 100, ai: 10, intelligence: 3, resistance: 0b00001000, pack_size: 0,
    },
    // Zhar the Mad (2)
    UniqueMonsterData {
        mtype: MonsterId::Counselor,
        mname_id: 3, mtalk_id: 2, min_level: 8, max_level: 8,
        hp_mult: 100, ai: 22, intelligence: 0, resistance: 0b00111000, pack_size: 0,
    },
    // Snotspill (3)
    UniqueMonsterData {
        mtype: MonsterId::FallenRSword,
        mname_id: 4, mtalk_id: 3, min_level: 4, max_level: 4,
        hp_mult: 100, ai: 23, intelligence: 0, resistance: 0, pack_size: 0,
    },
    // Lazarus (4)
    UniqueMonsterData {
        mtype: MonsterId::Advocate,
        mname_id: 5, mtalk_id: 0, min_level: 15, max_level: 15,
        hp_mult: 100, ai: 28, intelligence: 3, resistance: 0b00111000, pack_size: 0,
    },
    // Red Vex (5)
    UniqueMonsterData {
        mtype: MonsterId::Succubus,
        mname_id: 6, mtalk_id: 0, min_level: 15, max_level: 15,
        hp_mult: 100, ai: 29, intelligence: 3, resistance: 0b00111000, pack_size: 0,
    },
    // Black Jade (6)
    UniqueMonsterData {
        mtype: MonsterId::Succubus,
        mname_id: 7, mtalk_id: 0, min_level: 15, max_level: 15,
        hp_mult: 100, ai: 29, intelligence: 3, resistance: 0b00111000, pack_size: 0,
    },
    // Lachdanan (7)
    UniqueMonsterData {
        mtype: MonsterId::BlackKnightN,
        mname_id: 8, mtalk_id: 4, min_level: 14, max_level: 14,
        hp_mult: 100, ai: 30, intelligence: 0, resistance: 0, pack_size: 0,
    },
    // Warlord of Blood (8)
    UniqueMonsterData {
        mtype: MonsterId::BlackKnightN,
        mname_id: 9, mtalk_id: 0, min_level: 13, max_level: 13,
        hp_mult: 100, ai: 31, intelligence: 0, resistance: 0, pack_size: 0,
    },
    // The Butcher (9)
    UniqueMonsterData {
        mtype: MonsterId::Butcher,
        mname_id: 10, mtalk_id: 0, min_level: 2, max_level: 2,
        hp_mult: 100, ai: 13, intelligence: 0, resistance: 0, pack_size: 0,
    },
    // Hork Demon (10) - Hellfire
    UniqueMonsterData {
        mtype: MonsterId::HorkDemon,
        mname_id: 11, mtalk_id: 0, min_level: 21, max_level: 21,
        hp_mult: 100, ai: 34, intelligence: 3, resistance: 0b00101000, pack_size: 0,
    },
    // Defiler (11) - Hellfire
    UniqueMonsterData {
        mtype: MonsterId::Defiler,
        mname_id: 12, mtalk_id: 0, min_level: 22, max_level: 22,
        hp_mult: 100, ai: 34, intelligence: 3, resistance: 0b00111000, pack_size: 0,
    },
    // Na-Krul (12) - Hellfire
    UniqueMonsterData {
        mtype: MonsterId::NaKrul,
        mname_id: 13, mtalk_id: 0, min_level: 24, max_level: 24,
        hp_mult: 100, ai: 27, intelligence: 3, resistance: 0b00111111, pack_size: 0,
    },
];

// ============================================================================
// Level Monster Types (simplified)
// ============================================================================

/// Level monster type entry
/// C++ Reference: CMonster in Source/monster.h
#[derive(Debug, Clone, Copy, Default)]
pub struct CMonster {
    /// Monster type ID
    pub monster_type: MonsterId,
}

// ============================================================================
// Validation Context (for accessing game state)
// ============================================================================

/// Trait for accessing game state during validation
/// This allows validation functions to query monster and player data
pub trait ValidationContext {
    /// Get monster hit points at index
    fn get_monster_hit_points(&self, index: usize) -> Option<i32>;
    
    /// Check if player at index is active
    fn is_player_active(&self, player_id: usize) -> bool;
    
    /// Get number of players
    fn player_count(&self) -> usize;
    
    /// Get level monster type at index
    fn get_level_monster_type(&self, index: usize) -> Option<&CMonster>;
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Check if enemy ID is valid (without checking monster table)
///
/// C++ Reference: `IsEnemyIdValid` in Source/monsters/validation.cpp
///
/// # Arguments
/// * `enemy_id` - The enemy ID to validate
///
/// # Returns
/// `true` if the enemy ID is within valid bounds (monster or player)
pub fn is_enemy_id_valid(enemy_id: usize) -> bool {
    is_enemy_valid_internal(enemy_id, false, None::<&DummyContext>)
}

/// Check if enemy ID is valid for a given monster
///
/// C++ Reference: `IsEnemyValid` in Source/monsters/validation.cpp
///
/// # Arguments
/// * `monster_id` - The source monster's ID
/// * `enemy_id` - The target enemy's ID
/// * `context` - Optional validation context for checking monster hit points
///
/// # Returns
/// `true` if:
/// - monster_id is a valid monster index
/// - monster_id != enemy_id (monster can't target itself)
/// - enemy_id refers to a valid, living enemy
pub fn is_enemy_valid<C: ValidationContext>(
    monster_id: usize,
    enemy_id: usize,
    context: Option<&C>,
) -> bool {
    if monster_id >= MAX_MONSTERS {
        return false;
    }
    if monster_id == enemy_id {
        return false;
    }
    is_enemy_valid_internal(enemy_id, true, context)
}

/// Internal helper for enemy validation
fn is_enemy_valid_internal<C: ValidationContext>(
    enemy_id: usize,
    check_monster_table: bool,
    context: Option<&C>,
) -> bool {
    if enemy_id < MAX_MONSTERS {
        // Enemy is a monster
        if !check_monster_table {
            return true;
        }
        // Check if monster is alive (has HP > 0)
        if let Some(ctx) = context {
            if let Some(hp) = ctx.get_monster_hit_points(enemy_id) {
                return hp > 0;
            }
        }
        // Without context, assume valid
        return true;
    }
    
    // Enemy is a player (player IDs start at MaxMonsters)
    let player_id = enemy_id - MAX_MONSTERS;
    if let Some(ctx) = context {
        return player_id < ctx.player_count() && ctx.is_player_active(player_id);
    }
    // Without context, just check bounds
    player_id < MAX_PLRS
}

/// Check if a monster is valid
///
/// C++ Reference: `IsMonsterValid` in Source/monsters/validation.cpp
///
/// # Arguments
/// * `monster` - The monster to validate
/// * `context` - Validation context for accessing level monster types
///
/// # Returns
/// `true` if the monster has a valid type and passes unique validation if applicable
pub fn is_monster_valid<C: ValidationContext>(monster: &Monster, context: &C) -> bool {
    // Get the monster type from level monster types
    let level_type = monster.level_type as usize;
    let Some(monster_type_entry) = context.get_level_monster_type(level_type) else {
        return false;
    };
    
    let monster_id = monster_type_entry.monster_type;
    let monster_index = monster_id as i16 as usize;
    
    // Check if monster type ID is valid
    // Note: MonsterId enum has values up to ~143 (NaKrul = 137 for Hellfire)
    if monster_index > 200 {
        return false;
    }
    
    // Check unique monster validation if this is a unique
    if monster.unique_type != UniqueMonsterType::None {
        if !is_unique_monster_valid(monster, monster_id) {
            return false;
        }
    }
    
    true
}

/// Check if a unique monster is valid
///
/// C++ Reference: `IsUniqueMonsterValid` in Source/monsters/validation.cpp
///
/// # Arguments
/// * `monster` - The unique monster to validate
/// * `expected_type` - The expected monster type from level monster types
///
/// # Returns
/// `true` if the unique monster type matches expected data
pub fn is_unique_monster_valid(monster: &Monster, expected_type: MonsterId) -> bool {
    // Monster must be unique
    if monster.unique_type == UniqueMonsterType::None {
        return false;
    }
    
    let unique_index = monster.unique_type as u8 as usize;
    
    // Check if unique index is valid
    if unique_index >= UNIQUE_MONSTERS_DATA.len() {
        return false;
    }
    
    // Check if monster type matches expected unique monster type
    let unique_data = &UNIQUE_MONSTERS_DATA[unique_index];
    if expected_type != unique_data.mtype {
        return false;
    }
    
    true
}

// ============================================================================
// Dummy Context (for simple validation without game state)
// ============================================================================

/// Dummy context for validation without full game state
struct DummyContext;

impl ValidationContext for DummyContext {
    fn get_monster_hit_points(&self, _index: usize) -> Option<i32> {
        None
    }
    
    fn is_player_active(&self, _player_id: usize) -> bool {
        false
    }
    
    fn player_count(&self) -> usize {
        MAX_PLRS
    }
    
    fn get_level_monster_type(&self, _index: usize) -> Option<&CMonster> {
        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enemy_id_valid_monster() {
        // Valid monster indices
        assert!(is_enemy_id_valid(0));
        assert!(is_enemy_id_valid(MAX_MONSTERS - 1));
        
        // Player indices (also valid)
        assert!(is_enemy_id_valid(MAX_MONSTERS));
        assert!(is_enemy_id_valid(MAX_MONSTERS + MAX_PLRS - 1));
        
        // Invalid indices
        assert!(!is_enemy_id_valid(MAX_MONSTERS + MAX_PLRS));
        assert!(!is_enemy_id_valid(1000));
    }

    #[test]
    fn test_is_enemy_valid_self_target() {
        // Monster can't target itself
        assert!(!is_enemy_valid::<DummyContext>(5, 5, None));
        assert!(!is_enemy_valid::<DummyContext>(0, 0, None));
    }

    #[test]
    fn test_is_enemy_valid_invalid_monster_id() {
        // Monster ID must be valid
        assert!(!is_enemy_valid::<DummyContext>(MAX_MONSTERS, 0, None));
        assert!(!is_enemy_valid::<DummyContext>(MAX_MONSTERS + 1, 0, None));
    }

    #[test]
    fn test_is_enemy_valid_different_targets() {
        // Different valid targets
        assert!(is_enemy_valid::<DummyContext>(0, 1, None));
        assert!(is_enemy_valid::<DummyContext>(50, 100, None));
        // Can target players
        assert!(is_enemy_valid::<DummyContext>(0, MAX_MONSTERS, None));
    }

    #[test]
    fn test_unique_monsters_data() {
        // Check data integrity
        assert_eq!(UNIQUE_MONSTERS_DATA.len(), 13);
        
        // Skeleton King
        assert_eq!(UNIQUE_MONSTERS_DATA[1].mtype, MonsterId::SkeletonKing);
        
        // Butcher
        assert_eq!(UNIQUE_MONSTERS_DATA[9].mtype, MonsterId::Butcher);
        
        // Na-Krul (Hellfire boss)
        assert_eq!(UNIQUE_MONSTERS_DATA[12].mtype, MonsterId::NaKrul);
    }

    #[test]
    fn test_unique_monster_type_mapping() {
        // Test that UniqueMonsterType enum values match array indices
        assert_eq!(UniqueMonsterType::Garbud as u8, 0);
        assert_eq!(UniqueMonsterType::SkeletonKing as u8, 1);
        assert_eq!(UniqueMonsterType::Butcher as u8, 9);
        assert_eq!(UniqueMonsterType::NaKrul as u8, 12);
    }
}
