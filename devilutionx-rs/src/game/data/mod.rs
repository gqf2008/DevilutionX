//! Game data tables module
//! Contains all static game data definitions

pub mod spelldat;
pub mod objdat;

pub use spelldat::*;

// Re-export commonly used types
pub use spelldat::{
    SpellID, SpellType, SpellData, MagicType, MissileID,
    SpellDataFlags, SfxID, get_spell_data, SPELLS_DATA,
};

pub use objdat::{
    ObjectId, ThemeId, QuestId, ObjectDataFlags, ObjectData,
    OBJ_TYPE_CONV, ALL_OBJECTS, get_object_data,
    is_shrine, is_chest, is_door, is_breakable,
};
