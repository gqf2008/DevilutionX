//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Items Validation
//!
//! C++ Source: Source/items/
//!
//! Note: Main item logic is in Source/items.cpp (in src/game/),
//! this directory only contains validation utilities.

pub mod validation;

// Re-export commonly used items
pub use validation::{
    // Constants
    CF_BOY, CF_HEALER, CF_HELLFIRE, CF_LEVEL, CF_ONLYGOOD, CF_PREGEN, CF_SMITH,
    CF_SMITHPREMIUM, CF_TOWN, CF_UNIQUE, CF_UPER1, CF_UPER15, CF_USEFUL, CF_WITCH,
    IDI_EAR, IDI_GOLD, MAX_BOY_VALUE, MAX_BOY_VALUE_HF, MAX_VENDOR_VALUE, MAX_VENDOR_VALUE_HF,
    // Functions
    is_creation_flag_combo_valid, is_dungeon_item_valid, is_hellfire_spell_book_valid,
    is_item_valid, is_item_valid_default, is_shop_price_valid, is_town_item_valid,
    is_unique_monster_item_valid,
};
