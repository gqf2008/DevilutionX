//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Monsters Validation
//!
//! C++ Source: Source/monsters/
//!
//! Note: Main monster logic is in Source/monster.cpp (in src/game/),
//! this directory only contains validation utilities.

pub mod validation;

pub use validation::{
    // Constants
    MAX_MONSTERS,
    MAX_PLRS,
    // Data structures
    UniqueMonsterData,
    CMonster,
    UNIQUE_MONSTERS_DATA,
    // Traits
    ValidationContext,
    // Functions
    is_enemy_id_valid,
    is_enemy_valid,
    is_monster_valid,
    is_unique_monster_valid,
};
