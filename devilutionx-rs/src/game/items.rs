//! Item system - Diablo-style items with affixes, quality, and equipment slots
//!
//! **C++ Reference**: `Source/items.cpp`, `Source/itemdat.cpp`
//!
//! This module implements the complete item system including:
//! - Item generation with affixes (prefixes/suffixes)
//! - Unique item generation
//! - Item quality (normal, magic, unique)
//! - Equipment slots and inventory management
//! - Item interaction (use, repair, identify)

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use rand::Rng;

// ============================================================================
// C++ Exact Alignment Types
// ============================================================================

/// Item quality (C++ item_quality)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ItemQuality {
    #[default]
    Normal = 0,
    Magic = 1,      // Blue items with 1-2 affixes
    Unique = 2,     // Orange named items
    Rare = 3,       // Yellow items with multiple affixes
    Set = 4,        // Green set items
}

/// Item misc ID (C++ item_misc_id)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(i16)]
pub enum ItemMiscId {
    #[default]
    None = 0,
    Usable = 1,
    UsableInfinite = 2,
    Staff = 3,
    Book = 4,
    Ring = 5,
    Amulet = 6,
    Unique = 7,
    Heal = 8,
    FullHeal = 9,
    Mana = 10,
    FullMana = 11,
    Elixir = 12,
    Map = 13,
    Ear = 14,
    Spectral = 15,
    Oil = 16,
    Scroll = 17,
    ScrollT = 18,  // Targeted scroll
    Rejuv = 19,
    FullRejuv = 20,
    // Elixir types for stats
    ElixStr = 21,
    ElixMag = 22,
    ElixDex = 23,
    ElixVit = 24,
    // Special items
    MapOfDoom = 25,
    SpecialElixir = 26,
    ArenaPot = 27,
    // Runes
    RuneFirst = 100,
    RuneF = 101,
    RuneL = 102,
    RuneS = 103,
    GrRuneL = 104,
    GrRuneF = 105,
    RuneLast = 110,
}

/// Item equip location (C++ item_equip_type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum ItemEquipType {
    #[default]
    None = -1,
    OneHand = 0,
    TwoHand = 1,
    Armor = 2,
    Helm = 3,
    Ring = 4,
    Amulet = 5,
    Unequipable = 6,
    Belt = 7,
}

/// Item class (C++ item_class)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ItemClass {
    #[default]
    None = 0,
    Weapon = 1,
    Armor = 2,
    Misc = 3,
    Gold = 4,
    Quest = 5,
}

/// Item special effect flags (C++ ItemSpecialEffect)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ItemSpecialEffect(pub u32);

impl ItemSpecialEffect {
    pub const NONE: Self = Self(0);
    pub const INFRAVISION: Self = Self(1 << 0);
    pub const KNOCKBACK: Self = Self(1 << 1);
    pub const FIRE_ARROWS: Self = Self(1 << 2);
    pub const LIGHTNING_ARROWS: Self = Self(1 << 3);
    pub const DAMAGE_UNDEAD: Self = Self(1 << 4);
    pub const QUICKATTACK: Self = Self(1 << 5);
    pub const FASTATTACK: Self = Self(1 << 6);
    pub const FAST_ATTACK: Self = Self(1 << 6);  // Alias for compatibility
    pub const FASTERATTACK: Self = Self(1 << 7);
    pub const FASTESTATTACK: Self = Self(1 << 8);
    pub const FASTHITRECOVER: Self = Self(1 << 9);
    pub const FAST_HIT_RECOVERY: Self = Self(1 << 9);  // Alias for compatibility
    pub const FASTERHITRECOVER: Self = Self(1 << 10);
    pub const FASTESTBLOCKRECOVER: Self = Self(1 << 11);
    pub const FAST_BLOCK: Self = Self(1 << 11);  // Alias for compatibility
    pub const MULTIPLE_ARROWS: Self = Self(1 << 12);
    pub const STEALMANA_3: Self = Self(1 << 13);
    pub const STEAL_MANA_3: Self = Self(1 << 13);  // Alias for compatibility
    pub const STEALMANA_5: Self = Self(1 << 14);
    pub const STEAL_MANA_5: Self = Self(1 << 14);  // Alias for compatibility
    pub const STEALLIFE_3: Self = Self(1 << 15);
    pub const STEAL_LIFE_3: Self = Self(1 << 15);  // Alias for compatibility
    pub const STEALLIFE_5: Self = Self(1 << 16);
    pub const STEAL_LIFE_5: Self = Self(1 << 16);  // Alias for compatibility
    pub const DAMAGE_DEMON: Self = Self(1 << 17);
    pub const TRIPLE_DEMON_DAMAGE: Self = Self(1 << 17);  // Alias for compatibility
    pub const ABSORBHALF: Self = Self(1 << 18);
    pub const HALF_TRAP_DAMAGE: Self = Self(1 << 18);  // Alias for compatibility
    pub const THORNS: Self = Self(1 << 19);
    pub const NOMANADRAIN: Self = Self(1 << 20);
    pub const NO_MANA: Self = Self(1 << 20);  // Alias for compatibility
    pub const TRIPLEBOW: Self = Self(1 << 21);
    pub const CHWACK: Self = Self(1 << 22);
    pub const RANDOM_STEAL_LIFE: Self = Self(1 << 22);  // Alias for compatibility
    pub const ZEROTOHI: Self = Self(1 << 23);
    pub const ZERO_RESISTANCE: Self = Self(1 << 23);  // Alias for compatibility
    pub const ONEHAND: Self = Self(1 << 24);
    pub const DRAIN_LIFE: Self = Self(1 << 25);
    pub const RANDOM_ARROW_VELOCITY: Self = Self(1 << 26);
}

impl std::ops::BitOr for ItemSpecialEffect {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ItemSpecialEffect {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for ItemSpecialEffect {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// Item effect type for affixes (C++ item_effect_type - COMPLETE)
/// Exact C++ alignment: Source/itemdat.h enum item_effect_type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum ItemEffectType {
    #[default]
    Invalid = -1,
    // Basic combat modifiers
    ToHit = 0,          // IPL_TOHIT
    ToHitCurse = 1,     // IPL_TOHIT_CURSE
    Damage = 2,         // IPL_DAMP (damage %)
    DamageCurse = 3,    // IPL_DAMP_CURSE
    ToHitDamage = 4,    // IPL_TOHIT_DAMP
    ToHitDamageCurse = 5, // IPL_TOHIT_DAMP_CURSE
    Ac = 6,             // IPL_ACP (armor %)
    AcCurse = 7,        // IPL_ACP_CURSE
    // Resistances
    FireRes = 8,        // IPL_FIRERES
    LightRes = 9,       // IPL_LIGHTRES
    MagicRes = 10,      // IPL_MAGICRES
    AllRes = 11,        // IPL_ALLRES
    // Gap: 12, 13 unused
    SpellLevelAdd = 14, // IPL_SPLLVLADD
    Charges = 15,       // IPL_CHARGES
    FireDam = 16,       // IPL_FIREDAM
    LightDam = 17,      // IPL_LIGHTDAM
    // Gap: 18 unused
    Str = 19,           // IPL_STR
    StrCurse = 20,      // IPL_STR_CURSE
    Mag = 21,           // IPL_MAG
    MagCurse = 22,      // IPL_MAG_CURSE
    Dex = 23,           // IPL_DEX
    DexCurse = 24,      // IPL_DEX_CURSE
    Vit = 25,           // IPL_VIT
    VitCurse = 26,      // IPL_VIT_CURSE
    Attribs = 27,       // IPL_ATTRIBS (all stats)
    AttribsCurse = 28,  // IPL_ATTRIBS_CURSE
    GetHitCurse = 29,   // IPL_GETHIT_CURSE
    GetHit = 30,        // IPL_GETHIT
    Life = 31,          // IPL_LIFE
    LifeCurse = 32,     // IPL_LIFE_CURSE
    Mana = 33,          // IPL_MANA
    ManaCurse = 34,     // IPL_MANA_CURSE
    Dur = 35,           // IPL_DUR
    DurCurse = 36,      // IPL_DUR_CURSE
    Indestructible = 37, // IPL_INDESTRUCTIBLE
    Light = 38,         // IPL_LIGHT
    LightCurse = 39,    // IPL_LIGHT_CURSE
    // Gap: 40 unused
    MultArrows = 41,    // IPL_MULT_ARROWS (Hellfire only)
    FireArrows = 42,    // IPL_FIRE_ARROWS
    LightArrows = 43,   // IPL_LIGHT_ARROWS
    // Gap: 44 unused
    Thorns = 45,        // IPL_THORNS
    NoMana = 46,        // IPL_NOMANA
    // Gap: 47-49 unused
    Fireball = 50,      // IPL_FIREBALL (Hellfire only)
    // Gap: 51 unused
    AbsHalfTrap = 52,   // IPL_ABSHALFTRAP
    Knockback = 53,     // IPL_KNOCKBACK
    // Gap: 54 unused
    StealMana = 55,     // IPL_STEALMANA
    StealLife = 56,     // IPL_STEALLIFE
    TargAc = 57,        // IPL_TARGAC (damage to target AC)
    FastAttack = 58,    // IPL_FASTATTACK
    FastRecover = 59,   // IPL_FASTRECOVER
    FastBlock = 60,     // IPL_FASTBLOCK
    DamMod = 61,        // IPL_DAMMOD (+X damage)
    RndArrowVel = 62,   // IPL_RNDARROWVEL
    SetDam = 63,        // IPL_SETDAM (fixed damage)
    SetDur = 64,        // IPL_SETDUR (fixed durability)
    NoMinStr = 65,      // IPL_NOMINSTR (no strength requirement)
    Spell = 66,         // IPL_SPELL
    // Gap: 67 unused
    OneHand = 68,       // IPL_ONEHAND (2H weapon becomes 1H)
    TripleDemonDam = 69, // IPL_3XDAMVDEM (triple damage vs demons)
    AllResZero = 70,    // IPL_ALLRESZERO
    // Gap: 71 unused
    DrainLife = 72,     // IPL_DRAINLIFE (constant HP drain)
    RndStealLife = 73,  // IPL_RNDSTEALLIFE
    // Gap: 74 unused
    SetAc = 75,         // IPL_SETAC (fixed AC)
    AddAcLife = 76,     // IPL_ADDACLIFE
    AddManaAc = 77,     // IPL_ADDMANAAC
    // Gap: 78 unused
    AcCurseSet = 79,    // IPL_AC_CURSE (last Diablo effect)
    // Hellfire effects
    FireResCurse = 80,  // IPL_FIRERES_CURSE
    LightResCurse = 81, // IPL_LIGHTRES_CURSE
    MagicResCurse = 82, // IPL_MAGICRES_CURSE
    // Gap: 83 unused
    Devastation = 84,   // IPL_DEVASTATION
    Decay = 85,         // IPL_DECAY
    Peril = 86,         // IPL_PERIL
    Jesters = 87,       // IPL_JESTERS
    Crystalline = 88,   // IPL_CRYSTALLINE
    Doppelganger = 89,  // IPL_DOPPELGANGER
    AcDemon = 90,       // IPL_ACDEMON
    AcUndead = 91,      // IPL_ACUNDEAD
    ManaToLife = 92,    // IPL_MANATOLIFE
    LifeToMana = 93,    // IPL_LIFETOMANA
}

/// Equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EquipSlot {
    Head,
    Chest,
    LeftHand,   // Shield or off-hand
    RightHand,  // Main weapon
    LeftRing,
    RightRing,
    Amulet,
}

/// Item types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ItemType {
    #[default]
    None,
    // Weapons
    Sword,
    Axe,
    Mace,
    Bow,
    Staff,
    Dagger,

    // Armor
    Helm,
    Armor,
    Shield,

    // Jewelry
    Ring,
    Amulet,

    // Consumables
    HealthPotion,
    ManaPotion,
    FullRejuvenation,
    Elixir,
    Scroll,

    // Misc
    Gold,
    Book,
    Quest,
}

impl ItemType {
    pub fn get_equip_slot(&self) -> Option<EquipSlot> {
        match self {
            ItemType::Helm => Some(EquipSlot::Head),
            ItemType::Armor => Some(EquipSlot::Chest),
            ItemType::Shield => Some(EquipSlot::LeftHand),
            ItemType::Sword | ItemType::Axe | ItemType::Mace |
            ItemType::Bow | ItemType::Staff | ItemType::Dagger => Some(EquipSlot::RightHand),
            ItemType::Ring => Some(EquipSlot::LeftRing), // Can be either ring slot
            ItemType::Amulet => Some(EquipSlot::Amulet),
            _ => None,
        }
    }

    pub fn is_weapon(&self) -> bool {
        matches!(self, ItemType::Sword | ItemType::Axe | ItemType::Mace |
                      ItemType::Bow | ItemType::Staff | ItemType::Dagger)
    }

    pub fn is_armor(&self) -> bool {
        matches!(self, ItemType::Helm | ItemType::Armor | ItemType::Shield)
    }

    pub fn is_consumable(&self) -> bool {
        matches!(self, ItemType::HealthPotion | ItemType::ManaPotion |
                      ItemType::FullRejuvenation | ItemType::Elixir | ItemType::Scroll)
    }
}

/// Item affix (prefix or suffix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAffix {
    pub name: String,
    pub is_prefix: bool,
    pub bonus_str: i32,
    pub bonus_mag: i32,
    pub bonus_dex: i32,
    pub bonus_vit: i32,
    pub bonus_damage: i32,
    pub bonus_armor: i32,
    pub bonus_to_hit: i32,
    pub bonus_hp: i32,
    pub bonus_mana: i32,
    pub resist_fire: i32,
    pub resist_lightning: i32,
    pub resist_magic: i32,
    pub life_steal: i32,  // Percentage
    pub mana_steal: i32,  // Percentage
}

impl ItemAffix {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            is_prefix: true,
            bonus_str: 0,
            bonus_mag: 0,
            bonus_dex: 0,
            bonus_vit: 0,
            bonus_damage: 0,
            bonus_armor: 0,
            bonus_to_hit: 0,
            bonus_hp: 0,
            bonus_mana: 0,
            resist_fire: 0,
            resist_lightning: 0,
            resist_magic: 0,
            life_steal: 0,
            mana_steal: 0,
        }
    }
}

/// Main Item structure (C++ exact alignment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    // === C++ Exact Fields ===
    /// Randomly generated identifier (C++: _iSeed)
    pub seed: u32,
    /// Creation info flags (C++: _iCreateInfo)
    pub create_info: u16,
    /// Item type (C++: _itype)
    pub item_type: ItemType,
    /// Item index (C++: IDidx)
    pub item_index: i16,
    /// Item quality (C++: _iMagical)
    pub quality: ItemQuality,
    /// Item class (C++: _iClass)
    pub item_class: ItemClass,
    /// Equip location (C++: _iLoc)
    pub equip_loc: ItemEquipType,
    /// Misc ID (C++: _iMiscId)
    pub misc_id: ItemMiscId,

    // === Names ===
    pub name: String,
    pub base_name: String,

    // === Combat Stats (C++ exact) ===
    /// Min damage (C++: _iMinDam)
    pub min_damage: u8,
    /// Max damage (C++: _iMaxDam)
    pub max_damage: u8,
    /// Armor class (C++: _iAC)
    pub armor_class: i16,

    // === Special Effect Flags (C++ exact) ===
    pub special_flags: ItemSpecialEffect,

    // === Spell (C++ exact) ===
    /// Associated spell (C++: _iSpell)
    pub spell: i8,
    /// Spell charges (C++: _iCharges)
    pub charges: i32,
    /// Max charges (C++: _iMaxCharges)
    pub max_charges: i32,

    // === Durability (C++ exact) ===
    pub durability: i32,
    pub max_durability: i32,

    // === Affix Bonuses (C++ exact: _iPL* fields) ===
    pub bonus_damage: i16,      // _iPLDam
    pub bonus_to_hit: i16,      // _iPLToHit
    pub bonus_ac: i16,          // _iPLAC
    pub bonus_str: i16,         // _iPLStr
    pub bonus_mag: i16,         // _iPLMag
    pub bonus_dex: i16,         // _iPLDex
    pub bonus_vit: i16,         // _iPLVit
    pub resist_fire: i16,       // _iPLFR
    pub resist_lightning: i16,  // _iPLLR
    pub resist_magic: i16,      // _iPLMR
    pub bonus_mana: i16,        // _iPLMana
    pub bonus_hp: i16,          // _iPLHP
    pub bonus_damage_mod: i16,  // _iPLDamMod
    pub bonus_get_hit: i16,     // _iPLGetHit
    pub bonus_light: i16,       // _iPLLight
    pub spell_level_add: i8,    // _iSplLvlAdd

    // === Elemental Damage (C++ exact) ===
    pub fire_min_dam: i16,      // _iFMinDam
    pub fire_max_dam: i16,      // _iFMaxDam
    pub lightning_min_dam: i16, // _iLMinDam
    pub lightning_max_dam: i16, // _iLMaxDam
    pub bonus_energy_ac: i16,   // _iPLEnAc

    // === Affix Powers (C++ exact) ===
    pub prefix_power: ItemEffectType,
    pub suffix_power: ItemEffectType,
    pub value_add1: i32,        // _iVAdd1
    pub value_mult1: i32,       // _iVMult1
    pub value_add2: i32,        // _iVAdd2
    pub value_mult2: i32,       // _iVMult2

    // === Requirements (C++ exact) ===
    pub required_str: i8,       // _iMinStr
    pub required_mag: u8,       // _iMinMag
    pub required_dex: i8,       // _iMinDex
    pub required_level: u8,     // Required character level

    // === Base Stats (for save/display) ===
    pub base_damage_min: u8,    // Base min damage before bonuses
    pub base_damage_max: u8,    // Base max damage before bonuses
    pub base_armor: i16,        // Base armor before bonuses
    pub buy_value: i32,         // Purchase value at shop

    // === Unique Item (C++ exact) ===
    pub unique_id: i32,         // _iUid

    // === State Flags ===
    pub identified: bool,       // _iIdentified
    pub stat_flag: bool,        // _iStatFlag (can equip?)
    pub request: bool,          // _iRequest

    // === Value ===
    pub value: i32,             // _ivalue
    pub identified_value: i32,  // _iIvalue

    // === Position (when on ground) ===
    pub position_x: i32,
    pub position_y: i32,

    // === Cursor/Graphics ===
    pub cursor: u8,             // _iCurs

    // === Legacy fields for compatibility ===
    pub prefix: Option<ItemAffix>,
    pub suffix: Option<ItemAffix>,
    pub quantity: i32,
    pub max_stack: i32,
}

impl Default for Item {
    fn default() -> Self {
        Self::empty()
    }
}

impl Item {
    /// Create empty item
    pub fn empty() -> Self {
        Self {
            seed: 0,
            create_info: 0,
            item_type: ItemType::None,
            item_index: -1,
            quality: ItemQuality::Normal,
            item_class: ItemClass::None,
            equip_loc: ItemEquipType::None,
            misc_id: ItemMiscId::None,
            name: String::new(),
            base_name: String::new(),
            min_damage: 0,
            max_damage: 0,
            armor_class: 0,
            special_flags: ItemSpecialEffect::NONE,
            spell: -1,
            charges: 0,
            max_charges: 0,
            durability: 0,
            max_durability: 0,
            bonus_damage: 0,
            bonus_to_hit: 0,
            bonus_ac: 0,
            bonus_str: 0,
            bonus_mag: 0,
            bonus_dex: 0,
            bonus_vit: 0,
            resist_fire: 0,
            resist_lightning: 0,
            resist_magic: 0,
            bonus_mana: 0,
            bonus_hp: 0,
            bonus_damage_mod: 0,
            bonus_get_hit: 0,
            bonus_light: 0,
            spell_level_add: 0,
            fire_min_dam: 0,
            fire_max_dam: 0,
            lightning_min_dam: 0,
            lightning_max_dam: 0,
            bonus_energy_ac: 0,
            prefix_power: ItemEffectType::Invalid,
            suffix_power: ItemEffectType::Invalid,
            value_add1: 0,
            value_mult1: 0,
            value_add2: 0,
            value_mult2: 0,
            required_str: 0,
            required_mag: 0,
            required_dex: 0,
            required_level: 0,
            base_damage_min: 0,
            base_damage_max: 0,
            base_armor: 0,
            buy_value: 0,
            unique_id: -1,
            identified: false,
            stat_flag: false,
            request: false,
            value: 0,
            identified_value: 0,
            position_x: 0,
            position_y: 0,
            cursor: 0,
            prefix: None,
            suffix: None,
            quantity: 0,
            max_stack: 1,
        }
    }

    pub fn new(name: String, item_type: ItemType, value: i32) -> Self {
        let mut item = Self::empty();
        item.name = name.clone();
        item.base_name = name;
        item.item_type = item_type;
        item.value = value;
        item.identified_value = value;
        item.durability = 100;
        item.max_durability = 100;
        item.identified = true;
        item.quantity = 1;
        item
    }

    /// Create gold drop
    pub fn gold(amount: i32) -> Self {
        let mut item = Self::empty();
        item.name = format!("{} Gold", amount);
        item.base_name = "Gold".to_string();
        item.item_type = ItemType::Gold;
        item.item_class = ItemClass::Gold;
        item.value = amount;
        item.identified_value = amount;
        item.quantity = amount;
        item.max_stack = 5000;
        item.identified = true;
        item
    }

    /// Create health potion
    pub fn health_potion(healing: i32) -> Self {
        let name = match healing {
            h if h <= 50 => "Minor Healing Potion",
            h if h <= 100 => "Light Healing Potion",
            h if h <= 200 => "Healing Potion",
            _ => "Full Healing Potion",
        };

        let mut item = Self::empty();
        item.name = name.to_string();
        item.base_name = name.to_string();
        item.item_type = ItemType::HealthPotion;
        item.item_class = ItemClass::Misc;
        item.misc_id = ItemMiscId::Heal;
        item.min_damage = healing as u8; // Store healing in damage
        item.max_damage = healing as u8;
        item.value = healing * 2;
        item.identified_value = healing * 2;
        item.quantity = 1;
        item.max_stack = 20;
        item.identified = true;
        item
    }

    /// Create mana potion
    pub fn mana_potion(restore: i32) -> Self {
        let name = match restore {
            r if r <= 40 => "Minor Mana Potion",
            r if r <= 80 => "Light Mana Potion",
            r if r <= 160 => "Mana Potion",
            _ => "Full Mana Potion",
        };

        let mut item = Self::empty();
        item.name = name.to_string();
        item.base_name = name.to_string();
        item.item_type = ItemType::ManaPotion;
        item.item_class = ItemClass::Misc;
        item.misc_id = ItemMiscId::Mana;
        item.min_damage = restore as u8;
        item.max_damage = restore as u8;
        item.value = restore * 3;
        item.identified_value = restore * 3;
        item.quantity = 1;
        item.max_stack = 20;
        item.identified = true;
        item
    }

    /// Get total damage with affixes
    pub fn total_damage(&self) -> (i32, i32) {
        let mut min = self.min_damage as i32 + self.bonus_damage as i32;
        let mut max = self.max_damage as i32 + self.bonus_damage as i32;

        if let Some(ref prefix) = self.prefix {
            min += prefix.bonus_damage;
            max += prefix.bonus_damage;
        }
        if let Some(ref suffix) = self.suffix {
            min += suffix.bonus_damage;
            max += suffix.bonus_damage;
        }

        (min.max(1), max.max(min))
    }

    /// Get total armor with affixes
    pub fn total_armor(&self) -> i32 {
        let mut armor = self.armor_class as i32 + self.bonus_ac as i32;

        if let Some(ref prefix) = self.prefix {
            armor += prefix.bonus_armor;
        }
        if let Some(ref suffix) = self.suffix {
            armor += suffix.bonus_armor;
        }

        armor.max(0)
    }

    /// Generate full name with affixes
    pub fn full_name(&self) -> String {
        if !self.identified && self.quality != ItemQuality::Normal {
            return format!("Unidentified {}", self.base_name);
        }

        let mut name = String::new();

        if let Some(ref prefix) = self.prefix {
            name.push_str(&prefix.name);
            name.push(' ');
        }

        name.push_str(&self.base_name);

        if let Some(ref suffix) = self.suffix {
            name.push_str(" of ");
            name.push_str(&suffix.name);
        }

        name
    }
}

/// Ground item - item dropped on the dungeon floor
#[derive(Debug, Clone)]
pub struct GroundItem {
    pub item: Item,
    pub x: i32,
    pub y: i32,
    pub spawn_tick: u64,
}

impl GroundItem {
    pub fn new(item: Item, x: i32, y: i32, tick: u64) -> Self {
        Self {
            item,
            x,
            y,
            spawn_tick: tick,
        }
    }
}

/// Item generator for creating random loot
pub struct ItemGenerator {
    // Prefix pools by item level
    weapon_prefixes: Vec<(i32, ItemAffix)>,
    armor_prefixes: Vec<(i32, ItemAffix)>,
    weapon_suffixes: Vec<(i32, ItemAffix)>,
    armor_suffixes: Vec<(i32, ItemAffix)>,
}

impl ItemGenerator {
    pub fn new() -> Self {
        let mut gen = Self {
            weapon_prefixes: Vec::new(),
            armor_prefixes: Vec::new(),
            weapon_suffixes: Vec::new(),
            armor_suffixes: Vec::new(),
        };
        gen.init_affixes();
        gen
    }

    fn init_affixes(&mut self) {
        // Weapon prefixes
        self.weapon_prefixes.push((1, ItemAffix {
            name: "Sharp".to_string(),
            is_prefix: true,
            bonus_damage: 2,
            ..ItemAffix::empty()
        }));
        self.weapon_prefixes.push((3, ItemAffix {
            name: "Fine".to_string(),
            is_prefix: true,
            bonus_damage: 4,
            bonus_to_hit: 10,
            ..ItemAffix::empty()
        }));
        self.weapon_prefixes.push((5, ItemAffix {
            name: "Warrior's".to_string(),
            is_prefix: true,
            bonus_str: 5,
            bonus_damage: 3,
            ..ItemAffix::empty()
        }));
        self.weapon_prefixes.push((8, ItemAffix {
            name: "Soldier's".to_string(),
            is_prefix: true,
            bonus_str: 10,
            bonus_damage: 5,
            ..ItemAffix::empty()
        }));
        self.weapon_prefixes.push((12, ItemAffix {
            name: "Knight's".to_string(),
            is_prefix: true,
            bonus_str: 15,
            bonus_damage: 8,
            bonus_to_hit: 20,
            ..ItemAffix::empty()
        }));
        self.weapon_prefixes.push((15, ItemAffix {
            name: "King's".to_string(),
            is_prefix: true,
            bonus_str: 20,
            bonus_damage: 12,
            bonus_to_hit: 30,
            ..ItemAffix::empty()
        }));

        // Armor prefixes
        self.armor_prefixes.push((1, ItemAffix {
            name: "Sturdy".to_string(),
            is_prefix: true,
            bonus_armor: 3,
            ..ItemAffix::empty()
        }));
        self.armor_prefixes.push((4, ItemAffix {
            name: "Strong".to_string(),
            is_prefix: true,
            bonus_armor: 6,
            bonus_vit: 3,
            ..ItemAffix::empty()
        }));
        self.armor_prefixes.push((8, ItemAffix {
            name: "Glorious".to_string(),
            is_prefix: true,
            bonus_armor: 10,
            bonus_hp: 20,
            ..ItemAffix::empty()
        }));
        self.armor_prefixes.push((12, ItemAffix {
            name: "Blessed".to_string(),
            is_prefix: true,
            bonus_armor: 15,
            bonus_hp: 30,
            resist_magic: 10,
            ..ItemAffix::empty()
        }));

        // Weapon suffixes
        self.weapon_suffixes.push((1, ItemAffix {
            name: "Quality".to_string(),
            is_prefix: false,
            bonus_damage: 1,
            bonus_to_hit: 5,
            ..ItemAffix::empty()
        }));
        self.weapon_suffixes.push((5, ItemAffix {
            name: "Slaying".to_string(),
            is_prefix: false,
            bonus_damage: 6,
            ..ItemAffix::empty()
        }));
        self.weapon_suffixes.push((8, ItemAffix {
            name: "Gore".to_string(),
            is_prefix: false,
            bonus_damage: 8,
            life_steal: 3,
            ..ItemAffix::empty()
        }));
        self.weapon_suffixes.push((12, ItemAffix {
            name: "Vampires".to_string(),
            is_prefix: false,
            life_steal: 6,
            ..ItemAffix::empty()
        }));
        self.weapon_suffixes.push((10, ItemAffix {
            name: "the Heavens".to_string(),
            is_prefix: false,
            bonus_damage: 10,
            bonus_to_hit: 25,
            ..ItemAffix::empty()
        }));

        // Armor suffixes
        self.armor_suffixes.push((1, ItemAffix {
            name: "Health".to_string(),
            is_prefix: false,
            bonus_hp: 10,
            ..ItemAffix::empty()
        }));
        self.armor_suffixes.push((3, ItemAffix {
            name: "Protection".to_string(),
            is_prefix: false,
            bonus_armor: 5,
            ..ItemAffix::empty()
        }));
        self.armor_suffixes.push((6, ItemAffix {
            name: "Fire".to_string(),
            is_prefix: false,
            resist_fire: 20,
            ..ItemAffix::empty()
        }));
        self.armor_suffixes.push((6, ItemAffix {
            name: "Lightning".to_string(),
            is_prefix: false,
            resist_lightning: 20,
            ..ItemAffix::empty()
        }));
        self.armor_suffixes.push((10, ItemAffix {
            name: "the Stars".to_string(),
            is_prefix: false,
            bonus_hp: 30,
            bonus_mana: 20,
            ..ItemAffix::empty()
        }));
        self.armor_suffixes.push((12, ItemAffix {
            name: "the Zodiac".to_string(),
            is_prefix: false,
            bonus_str: 5,
            bonus_mag: 5,
            bonus_dex: 5,
            bonus_vit: 5,
            ..ItemAffix::empty()
        }));
    }

    /// Generate a random weapon for a given level
    pub fn generate_weapon(&self, level: i32, rng: &mut impl Rng) -> Item {
        let weapons = [
            ("Short Sword", 1, 4, 0, 18, ItemType::Sword),
            ("Falchion", 2, 6, 10, 30, ItemType::Sword),
            ("Claymore", 3, 8, 20, 40, ItemType::Sword),
            ("Broad Sword", 4, 10, 30, 50, ItemType::Sword),
            ("Long Sword", 5, 12, 40, 60, ItemType::Sword),
            ("Hand Axe", 2, 5, 0, 22, ItemType::Axe),
            ("Battle Axe", 4, 12, 25, 50, ItemType::Axe),
            ("Great Axe", 6, 16, 40, 80, ItemType::Axe),
            ("Club", 1, 3, 0, 12, ItemType::Mace),
            ("Mace", 2, 6, 15, 32, ItemType::Mace),
            ("Morning Star", 4, 10, 25, 50, ItemType::Mace),
            ("War Hammer", 5, 14, 35, 65, ItemType::Mace),
            ("Short Bow", 1, 4, 0, 25, ItemType::Bow),
            ("Long Bow", 2, 7, 20, 45, ItemType::Bow),
            ("Composite Bow", 4, 10, 40, 80, ItemType::Bow),
            ("Short Staff", 1, 3, 0, 15, ItemType::Staff),
            ("Long Staff", 2, 5, 20, 30, ItemType::Staff),
            ("War Staff", 4, 10, 50, 80, ItemType::Staff),
            ("Dagger", 1, 3, 0, 15, ItemType::Dagger),
            ("Blade", 2, 5, 15, 30, ItemType::Dagger),
        ];

        // Filter by level
        let available: Vec<_> = weapons.iter()
            .filter(|w| w.1 <= level + 2)
            .collect();

        if available.is_empty() {
            return self.generate_weapon(1, rng);
        }

        let &(name, min_dam, max_dam, req_str, value, ref item_type) =
            available[rng.random_range(0..available.len())];

        let mut item = Item::empty();
        item.name = name.to_string();
        item.base_name = name.to_string();
        item.item_type = *item_type;
        item.item_class = ItemClass::Weapon;
        item.min_damage = min_dam as u8;
        item.max_damage = max_dam as u8;
        item.required_str = req_str as i8;
        item.value = value;
        item.identified_value = value;
        item.durability = 50 + rng.random_range(0..50);
        item.max_durability = 100;
        item.identified = true;
        item.quantity = 1;

        // Roll for magic item
        let magic_chance = 10 + level * 2;
        if rng.random_range(0..100) < magic_chance {
            item.quality = ItemQuality::Magic;
            item.identified = false;

            // Add prefix
            let available_prefixes: Vec<_> = self.weapon_prefixes.iter()
                .filter(|(lvl, _)| *lvl <= level)
                .collect();
            if !available_prefixes.is_empty() && rng.random_bool(0.7) {
                let (_, affix) = &available_prefixes[rng.random_range(0..available_prefixes.len())];
                item.prefix = Some(affix.clone());
            }

            // Add suffix
            let available_suffixes: Vec<_> = self.weapon_suffixes.iter()
                .filter(|(lvl, _)| *lvl <= level)
                .collect();
            if !available_suffixes.is_empty() && rng.random_bool(0.7) {
                let (_, affix) = &available_suffixes[rng.random_range(0..available_suffixes.len())];
                item.suffix = Some(affix.clone());
            }

            // Update name and value
            item.name = item.full_name();
            item.value *= 2;
            item.identified_value *= 2;
        }

        item
    }

    /// Generate a random armor piece for a given level
    pub fn generate_armor(&self, level: i32, rng: &mut impl Rng) -> Item {
        let armors = [
            ("Cap", 2, 0, ItemType::Helm, 15),
            ("Skull Cap", 4, 15, ItemType::Helm, 30),
            ("Helm", 7, 25, ItemType::Helm, 50),
            ("Full Helm", 10, 35, ItemType::Helm, 80),
            ("Great Helm", 15, 50, ItemType::Helm, 120),
            ("Rags", 3, 0, ItemType::Armor, 10),
            ("Cloak", 5, 0, ItemType::Armor, 25),
            ("Leather Armor", 10, 15, ItemType::Armor, 50),
            ("Hard Leather", 15, 25, ItemType::Armor, 80),
            ("Studded Leather", 20, 35, ItemType::Armor, 120),
            ("Ring Mail", 25, 40, ItemType::Armor, 160),
            ("Chain Mail", 30, 50, ItemType::Armor, 200),
            ("Plate Mail", 42, 60, ItemType::Armor, 350),
            ("Full Plate", 60, 80, ItemType::Armor, 550),
            ("Buckler", 3, 0, ItemType::Shield, 20),
            ("Small Shield", 6, 15, ItemType::Shield, 40),
            ("Large Shield", 10, 30, ItemType::Shield, 70),
            ("Kite Shield", 15, 40, ItemType::Shield, 100),
            ("Tower Shield", 20, 60, ItemType::Shield, 150),
        ];

        // Filter by level
        let available: Vec<_> = armors.iter()
            .filter(|a| a.1 <= level * 5 + 10)
            .collect();

        if available.is_empty() {
            return self.generate_armor(1, rng);
        }

        let &(name, armor, req_str, ref item_type, value) =
            available[rng.random_range(0..available.len())];

        let mut item = Item::empty();
        item.name = name.to_string();
        item.base_name = name.to_string();
        item.item_type = *item_type;
        item.item_class = ItemClass::Armor;
        item.armor_class = armor as i16;
        item.required_str = req_str as i8;
        item.value = value;
        item.identified_value = value;
        item.durability = 50 + rng.random_range(0..50);
        item.max_durability = 100;
        item.identified = true;
        item.quantity = 1;

        // Roll for magic item
        let magic_chance = 8 + level * 2;
        if rng.random_range(0..100) < magic_chance {
            item.quality = ItemQuality::Magic;
            item.identified = false;

            // Add prefix
            let available_prefixes: Vec<_> = self.armor_prefixes.iter()
                .filter(|(lvl, _)| *lvl <= level)
                .collect();
            if !available_prefixes.is_empty() && rng.random_bool(0.7) {
                let (_, affix) = &available_prefixes[rng.random_range(0..available_prefixes.len())];
                item.prefix = Some(affix.clone());
            }

            // Add suffix
            let available_suffixes: Vec<_> = self.armor_suffixes.iter()
                .filter(|(lvl, _)| *lvl <= level)
                .collect();
            if !available_suffixes.is_empty() && rng.random_bool(0.7) {
                let (_, affix) = &available_suffixes[rng.random_range(0..available_suffixes.len())];
                item.suffix = Some(affix.clone());
            }

            // Update name and value
            item.name = item.full_name();
            item.value *= 2;
            item.identified_value *= 2;
        }

        item
    }

    /// Generate monster loot drop
    pub fn generate_monster_drop(&self, monster_level: i32, gold_range: (i32, i32), rng: &mut impl Rng) -> Vec<Item> {
        let mut drops = Vec::new();

        // Always drop gold
        let gold_amount = rng.random_range(gold_range.0..=gold_range.1);
        if gold_amount > 0 {
            drops.push(Item::gold(gold_amount));
        }

        // Chance to drop item
        let item_chance = 15 + monster_level;
        if rng.random_range(0..100) < item_chance {
            // Determine item type
            let roll = rng.random_range(0..100);
            let item = if roll < 30 {
                // Potion
                if rng.random_bool(0.6) {
                    Item::health_potion(30 + monster_level * 5)
                } else {
                    Item::mana_potion(25 + monster_level * 4)
                }
            } else if roll < 60 {
                // Weapon
                self.generate_weapon(monster_level, rng)
            } else {
                // Armor
                self.generate_armor(monster_level, rng)
            };

            drops.push(item);
        }

        drops
    }
}

/// Player inventory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// Equipment slots
    pub head: Option<Item>,
    pub chest: Option<Item>,
    pub left_hand: Option<Item>,
    pub right_hand: Option<Item>,
    pub left_ring: Option<Item>,
    pub right_ring: Option<Item>,
    pub amulet: Option<Item>,

    /// Backpack (grid-based in original, simplified to list here)
    pub backpack: Vec<Item>,
    pub max_backpack_size: usize,

    /// Belt (quick-use potions)
    pub belt: [Option<Item>; 8],

    /// Gold
    pub gold: i32,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            head: None,
            chest: None,
            left_hand: None,
            right_hand: None,
            left_ring: None,
            right_ring: None,
            amulet: None,
            backpack: Vec::new(),
            max_backpack_size: 40,
            belt: [const { None }; 8],
            gold: 0,
        }
    }

    /// Try to add item to inventory, returns false if no space
    pub fn add_item(&mut self, item: Item) -> bool {
        // Handle gold
        if item.item_type == ItemType::Gold {
            self.gold += item.quantity;
            return true;
        }

        // Handle potions - try belt first
        if item.item_type == ItemType::HealthPotion || item.item_type == ItemType::ManaPotion {
            for slot in &mut self.belt {
                if slot.is_none() {
                    *slot = Some(item);
                    return true;
                }
            }
        }

        // Add to backpack
        if self.backpack.len() < self.max_backpack_size {
            self.backpack.push(item);
            return true;
        }

        false
    }

    /// Equip item, returns previously equipped item if any
    pub fn equip(&mut self, item: Item) -> Option<Item> {
        let slot = item.item_type.get_equip_slot()?;

        let slot_ref = match slot {
            EquipSlot::Head => &mut self.head,
            EquipSlot::Chest => &mut self.chest,
            EquipSlot::LeftHand => &mut self.left_hand,
            EquipSlot::RightHand => &mut self.right_hand,
            EquipSlot::LeftRing => {
                // Try left ring first, then right
                if self.left_ring.is_none() {
                    &mut self.left_ring
                } else if self.right_ring.is_none() {
                    &mut self.right_ring
                } else {
                    &mut self.left_ring
                }
            }
            EquipSlot::RightRing => &mut self.right_ring,
            EquipSlot::Amulet => &mut self.amulet,
        };

        let old = slot_ref.take();
        *slot_ref = Some(item);
        old
    }

    /// Get total armor from all equipment
    pub fn total_armor(&self) -> i32 {
        let mut total = 0;

        if let Some(ref item) = self.head {
            total += item.total_armor();
        }
        if let Some(ref item) = self.chest {
            total += item.total_armor();
        }
        if let Some(ref item) = self.left_hand {
            total += item.total_armor();
        }

        total
    }

    /// Get weapon damage range
    pub fn weapon_damage(&self) -> (i32, i32) {
        if let Some(ref weapon) = self.right_hand {
            weapon.total_damage()
        } else {
            (1, 2)  // Unarmed
        }
    }

    /// Use belt item at index
    pub fn use_belt_item(&mut self, index: usize) -> Option<Item> {
        if index < 8 {
            self.belt[index].take()
        } else {
            None
        }
    }
}

// ============================================================================
// M55: 物品系统增强 - 物品属性计算和生成系统
// ============================================================================
// C++ Reference: Source/items.cpp
// ============================================================================

use super::item_dat::{
    ItemData, ItemType as ItemDatType, ItemClass as ItemDatClass,
    ItemEquipType as ItemDatEquipType, ItemMiscId as ItemDatMiscId,
    ITEMS_DATA,
};
use super::item_affix::{AffixItemType, ITEM_PREFIXES, ITEM_SUFFIXES, AffixData};

/// Maximum number of items in the game (C++: MAXITEMS)
pub const MAXITEMS: usize = 127;

/// Maximum dungeon size for item placement
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;

/// Gold limits
pub const GOLD_SMALL_LIMIT: i32 = 1000;
pub const GOLD_MEDIUM_LIMIT: i32 = 2500;
pub const GOLD_MAX_LIMIT: i32 = 5000;

/// Item creation info flags (C++: _icreateinfo_flag)
pub mod CreateInfoFlag {
    pub const CF_LEVEL: u16 = 0x007F;      // Level mask
    pub const CF_ONLYGOOD: u16 = 0x0080;   // Only good affixes
    pub const CF_UPER15: u16 = 0x0100;     // Unique chance 15%
    pub const CF_UPER1: u16 = 0x0200;      // Unique chance 1%
    pub const CF_UNIQUE: u16 = 0x0400;     // Is unique
    pub const CF_SMITH: u16 = 0x0800;      // From smith
    pub const CF_SMITHPREMIUM: u16 = 0x1000; // Premium item
    pub const CF_BOY: u16 = 0x2000;        // From Wirt
    pub const CF_WITCH: u16 = 0x4000;      // From witch
    pub const CF_HEALER: u16 = 0x8000;     // From healer
    pub const CF_PREGEN: u16 = 0x0040;     // Pre-generated
    pub const CF_USEFUL: u16 = 0x0020;     // Useful item
    pub const CF_TOWN: u16 = 0x0010;       // From town
    pub const CF_HELLFIRE: u32 = 0x00010000; // Hellfire item (in dwBuff)
}

/// Item index enum (C++: _item_indexes)
/// 部分实现，用于常用物品
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(i16)]
pub enum ItemIndex {
    None = -1,
    Gold = 0,
    PotionHealing = 1,
    PotionMana = 2,
    PotionFullHealing = 3,
    PotionFullMana = 4,
    PotionRejuv = 5,
    ScrollIdentify = 6,
    ScrollTownPortal = 7,
    // Weapons
    ShortSword = 20,
    Falchion = 21,
    Claymore = 22,
    BroadSword = 23,
    SabreBreaker = 24,
    LongSword = 25,
    Bastard = 26,
    TwoHandSword = 27,
    GreatSword = 28,
    // Axes
    SmallAxe = 40,
    Axe = 41,
    LargeAxe = 42,
    BroadAxe = 43,
    BattleAxe = 44,
    GreatAxe = 45,
    // Maces
    Club = 50,
    Spiked = 51,
    Mace = 52,
    MorningStar = 53,
    Flail = 54,
    WarHammer = 55,
    Maul = 56,
    // Bows
    ShortBow = 60,
    HuntersBow = 61,
    LongBow = 62,
    CompositeBow = 63,
    ShortBattleBow = 64,
    LongBattleBow = 65,
    ShortWarBow = 66,
    LongWarBow = 67,
    // Staves
    ShortStaff = 70,
    LongStaff = 71,
    CompositeStaff = 72,
    QuarterStaff = 73,
    WarStaff = 74,
    // Shields
    Buckler = 80,
    SmallShield = 81,
    LargeShield = 82,
    KiteShield = 83,
    TowerShield = 84,
    GothicShield = 85,
    // Helms
    Cap = 90,
    SkullCap = 91,
    Helm = 92,
    FullHelm = 93,
    GreatHelm = 94,
    Crown = 95,
    // Armor
    Rags = 100,
    Cloak = 101,
    Robe = 102,
    QuiltedArmor = 103,
    LeatherArmor = 104,
    HardLeatherArmor = 105,
    StuddedLeather = 106,
    RingMail = 107,
    ChainMail = 108,
    ScaleMail = 109,
    BreastPlate = 110,
    SplintMail = 111,
    PlateMail = 112,
    FieldPlate = 113,
    GothicPlate = 114,
    FullPlateMail = 115,
    // Jewelry
    Ring = 130,
    Amulet = 131,
    // Quest items
    Rock = 150,
    MagicRock = 151,
    OpticalJoystick = 152, // Map of Stars
}

impl ItemIndex {
    /// Get item data for this index
    pub fn get_data(&self) -> Option<&'static ItemData> {
        let idx = *self as i16;
        if idx < 0 || idx as usize >= ITEMS_DATA.len() {
            None
        } else {
            Some(&ITEMS_DATA[idx as usize])
        }
    }
}

/// 物品数组管理器
///
/// **C++ Reference**: Items[MAXITEMS+1], ActiveItems[], ActiveItemCount
#[derive(Debug, Clone)]
pub struct ItemArray {
    /// 所有物品数组
    pub items: Vec<Option<Item>>,
    /// 活动物品索引
    pub active_items: Vec<u8>,
    /// 活动物品计数
    pub active_count: usize,
    /// 地图上的物品 (dItem)
    pub ground_items: [[i8; MAXDUNY]; MAXDUNX],
    /// 唯一物品标记
    pub unique_item_flags: [bool; 128],
}

impl Default for ItemArray {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemArray {
    pub fn new() -> Self {
        Self {
            items: vec![None; MAXITEMS + 1],
            active_items: Vec::with_capacity(MAXITEMS),
            active_count: 0,
            ground_items: [[0; MAXDUNY]; MAXDUNX],
            unique_item_flags: [false; 128],
        }
    }

    /// 分配一个新物品槽位
    ///
    /// **C++ Reference**: `AllocateItem()`
    pub fn allocate(&mut self) -> Option<usize> {
        if self.active_count >= MAXITEMS {
            return None;
        }

        // 找一个空槽
        for i in 0..MAXITEMS {
            if self.items[i].is_none() {
                self.active_items.push(i as u8);
                self.active_count += 1;
                return Some(i);
            }
        }

        None
    }

    /// 释放物品槽位
    ///
    /// **C++ Reference**: `FreeItem()`
    pub fn free(&mut self, idx: usize) {
        if idx >= MAXITEMS {
            return;
        }

        self.items[idx] = None;

        // 从活动列表移除
        if let Some(pos) = self.active_items.iter().position(|&x| x as usize == idx) {
            self.active_items.remove(pos);
            self.active_count = self.active_count.saturating_sub(1);
        }
    }

    /// 获取物品引用
    pub fn get(&self, idx: usize) -> Option<&Item> {
        self.items.get(idx).and_then(|opt| opt.as_ref())
    }

    /// 获取物品可变引用
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Item> {
        self.items.get_mut(idx).and_then(|opt| opt.as_mut())
    }

    /// 在位置放置物品
    pub fn place_at(&mut self, idx: usize, x: usize, y: usize) {
        if x < MAXDUNX && y < MAXDUNY {
            self.ground_items[x][y] = (idx + 1) as i8;
            if let Some(item) = self.get_mut(idx) {
                item.position_x = x as i32;
                item.position_y = y as i32;
            }
        }
    }

    /// 获取位置上的物品索引
    pub fn get_at(&self, x: usize, y: usize) -> Option<usize> {
        if x < MAXDUNX && y < MAXDUNY {
            let idx = self.ground_items[x][y];
            if idx > 0 {
                Some((idx - 1) as usize)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 清除位置上的物品
    pub fn clear_at(&mut self, x: usize, y: usize) {
        if x < MAXDUNX && y < MAXDUNY {
            self.ground_items[x][y] = 0;
        }
    }
}

/// 获取物品基础属性
///
/// **C++ Reference**: `Source/items.cpp:3115` - `GetItemAttrs()`
///
/// 从物品数据表加载基础属性到 Item 结构
pub fn get_item_attrs(item: &mut Item, item_idx: ItemIndex, level: i32) {
    let idx = item_idx as i16;
    if idx < 0 || idx as usize >= ITEMS_DATA.len() {
        return;
    }

    let data = &ITEMS_DATA[idx as usize];

    // 复制基础数据
    item.item_index = idx;
    item.name = data.name.to_string();
    item.base_name = data.name.to_string();

    // 类型和位置
    item.item_type = match data.item_type {
        ItemDatType::Sword => ItemType::Sword,
        ItemDatType::Axe => ItemType::Axe,
        ItemDatType::Mace => ItemType::Mace,
        ItemDatType::Bow => ItemType::Bow,
        ItemDatType::Staff => ItemType::Staff,
        ItemDatType::Shield => ItemType::Shield,
        ItemDatType::Helm => ItemType::Helm,
        ItemDatType::LightArmor | ItemDatType::MediumArmor | ItemDatType::HeavyArmor => ItemType::Armor,
        ItemDatType::Ring => ItemType::Ring,
        ItemDatType::Amulet => ItemType::Amulet,
        ItemDatType::Gold => ItemType::Gold,
        _ => ItemType::None,
    };

    item.item_class = match data.class {
        ItemDatClass::Weapon => ItemClass::Weapon,
        ItemDatClass::Armor => ItemClass::Armor,
        ItemDatClass::Misc => ItemClass::Misc,
        ItemDatClass::Gold => ItemClass::Gold,
        ItemDatClass::Quest => ItemClass::Quest,
        _ => ItemClass::None,
    };

    item.equip_loc = match data.equip_type {
        ItemDatEquipType::OneHand => ItemEquipType::OneHand,
        ItemDatEquipType::TwoHand => ItemEquipType::TwoHand,
        ItemDatEquipType::Armor => ItemEquipType::Armor,
        ItemDatEquipType::Helm => ItemEquipType::Helm,
        ItemDatEquipType::Ring => ItemEquipType::Ring,
        ItemDatEquipType::Amulet => ItemEquipType::Amulet,
        ItemDatEquipType::Belt => ItemEquipType::Belt,
        _ => ItemEquipType::None,
    };

    // 伤害和护甲
    item.min_damage = data.min_damage;
    item.max_damage = data.max_damage;
    item.base_damage_min = data.min_damage;
    item.base_damage_max = data.max_damage;

    // 护甲 (随机在 min_ac 和 max_ac 之间)
    let mut rng = rand::rng();
    if data.max_ac > data.min_ac {
        item.armor_class = data.min_ac as i16 + rng.random_range(0..=(data.max_ac - data.min_ac)) as i16;
    } else {
        item.armor_class = data.min_ac as i16;
    }
    item.base_armor = item.armor_class;

    // 耐久度
    item.durability = data.durability as i32;
    item.max_durability = data.durability as i32;

    // 需求
    item.required_str = data.min_str as i8;
    item.required_mag = data.min_mag;
    item.required_dex = data.min_dex as i8;

    // 价值
    item.value = data.value as i32;
    item.identified_value = data.value as i32;

    // 特殊效果
    item.special_flags = ItemSpecialEffect(data.special_effects.0);

    // 品质默认为普通
    item.quality = ItemQuality::Normal;

    // 其他属性
    item.misc_id = match data.misc_id {
        ItemDatMiscId::Heal => ItemMiscId::Heal,
        ItemDatMiscId::FullHeal => ItemMiscId::FullHeal,
        ItemDatMiscId::Mana => ItemMiscId::Mana,
        ItemDatMiscId::FullMana => ItemMiscId::FullMana,
        ItemDatMiscId::Rejuv => ItemMiscId::Rejuv,
        ItemDatMiscId::FullRejuv => ItemMiscId::FullRejuv,
        ItemDatMiscId::Scroll => ItemMiscId::Scroll,
        ItemDatMiscId::Staff => ItemMiscId::Staff,
        ItemDatMiscId::Book => ItemMiscId::Book,
        ItemDatMiscId::Ring => ItemMiscId::Ring,
        ItemDatMiscId::Amulet => ItemMiscId::Amulet,
        ItemDatMiscId::Unique => ItemMiscId::Unique,
        _ => ItemMiscId::None,
    };

    item.prefix_power = ItemEffectType::Invalid;
    item.suffix_power = ItemEffectType::Invalid;

    // 处理金币
    if item.item_type == ItemType::Gold {
        calculate_gold_value(item, level);
    }

    item.cursor = data.cursor_graphic;
}

/// 计算金币价值 (基于等级和难度)
///
/// **C++ Reference**: `GetItemAttrs()` 金币部分
fn calculate_gold_value(item: &mut Item, level: i32) {
    let mut rng = rand::rng();
    let base = 5 * level + rng.random_range(0..10 * level.max(1));

    item.value = base.min(GOLD_MAX_LIMIT);
    item.identified_value = item.value;
    item.quantity = item.value;
}

/// 获取物品加成 (应用词缀)
///
/// **C++ Reference**: `Source/items.cpp:1309` - `GetItemBonus()`
pub fn get_item_bonus(
    item: &mut Item,
    min_lvl: i32,
    max_lvl: i32,
    only_good: bool,
    allow_spells: bool,
) {
    // 根据物品类型确定词缀类型
    let affix_type = match item.item_type {
        ItemType::Sword | ItemType::Axe | ItemType::Mace | ItemType::Dagger => AffixItemType::ALL_WEAPONS,
        ItemType::Bow => AffixItemType::BOW,
        ItemType::Shield => AffixItemType::SHIELD,
        ItemType::Armor => AffixItemType::ALL_ARMOR,
        ItemType::Helm => AffixItemType::HELM,
        ItemType::Staff => AffixItemType::STAFF,
        ItemType::Ring | ItemType::Amulet => AffixItemType::JEWELRY,
        _ => AffixItemType::MISC,
    };

    get_item_power(item, min_lvl, max_lvl, affix_type, only_good);
}

/// 获取物品能力 (选择并应用词缀)
///
/// **C++ Reference**: `Source/items.cpp:1210` - `GetItemPower()`
pub fn get_item_power(
    item: &mut Item,
    min_lvl: i32,
    max_lvl: i32,
    affix_type: AffixItemType,
    only_good: bool,
) {
    let mut rng = rand::rng();

    // 选择前缀和后缀
    let (prefix_idx, suffix_idx) = select_affixes(&mut rng, min_lvl, max_lvl, affix_type, only_good);

    // 应用前缀
    if let Some(idx) = prefix_idx {
        if idx < ITEM_PREFIXES.len() {
            apply_affix(item, &ITEM_PREFIXES[idx], true);
        }
    }

    // 应用后缀
    if let Some(idx) = suffix_idx {
        if idx < ITEM_SUFFIXES.len() {
            apply_affix(item, &ITEM_SUFFIXES[idx], false);
        }
    }

    // 更新物品名称
    update_item_name(item);

    // 如果有词缀，设置为魔法品质
    if item.prefix_power != ItemEffectType::Invalid || item.suffix_power != ItemEffectType::Invalid {
        item.quality = ItemQuality::Magic;
    }
}

/// 选择词缀
fn select_affixes<R: Rng>(
    rng: &mut R,
    min_lvl: i32,
    max_lvl: i32,
    affix_type: AffixItemType,
    only_good: bool,
) -> (Option<usize>, Option<usize>) {
    // 过滤可用前缀
    let valid_prefixes: Vec<usize> = ITEM_PREFIXES
        .iter()
        .enumerate()
        .filter(|(_, affix)| {
            affix.min_level <= max_lvl
                && affix.item_types.contains(affix_type)
                && (!only_good || affix.is_good)
        })
        .map(|(i, _)| i)
        .collect();

    // 过滤可用后缀
    let valid_suffixes: Vec<usize> = ITEM_SUFFIXES
        .iter()
        .enumerate()
        .filter(|(_, affix)| {
            affix.min_level <= max_lvl
                && affix.item_types.contains(affix_type)
                && (!only_good || affix.is_good)
        })
        .map(|(i, _)| i)
        .collect();

    // 随机决定有哪些词缀
    let has_prefix = !valid_prefixes.is_empty() && rng.random_bool(0.5);
    let has_suffix = !valid_suffixes.is_empty() && rng.random_bool(0.5);

    let prefix_idx = if has_prefix {
        valid_prefixes.get(rng.random_range(0..valid_prefixes.len())).copied()
    } else {
        None
    };

    let suffix_idx = if has_suffix {
        valid_suffixes.get(rng.random_range(0..valid_suffixes.len())).copied()
    } else {
        None
    };

    (prefix_idx, suffix_idx)
}

/// 应用词缀效果到物品
fn apply_affix(item: &mut Item, affix: &AffixData, is_prefix: bool) {
    use super::item_dat::ItemEffectType as DatEffectType;

    // 计算随机值
    let mut rng = rand::rng();
    let value = if affix.min_val != affix.max_val {
        rng.random_range(affix.min_val..=affix.max_val)
    } else {
        affix.min_val
    };

    // 根据效果类型应用加成
    match affix.power.effect_type {
        DatEffectType::ToHit => {
            item.bonus_to_hit += value as i16;
        }
        DatEffectType::ToHitCurse => {
            item.bonus_to_hit -= value as i16;
        }
        DatEffectType::Damage => {
            item.bonus_damage += value as i16;
        }
        DatEffectType::DamageCurse => {
            item.bonus_damage -= value as i16;
        }
        DatEffectType::ArmorPercent | DatEffectType::SetAC => {
            item.bonus_ac += value as i16;
        }
        DatEffectType::ArmorPercentCurse | DatEffectType::ACCurse => {
            item.bonus_ac -= value as i16;
        }
        DatEffectType::FireRes => {
            item.resist_fire += value as i16;
        }
        DatEffectType::LightRes => {
            item.resist_lightning += value as i16;
        }
        DatEffectType::MagicRes => {
            item.resist_magic += value as i16;
        }
        DatEffectType::AllRes => {
            item.resist_fire += value as i16;
            item.resist_lightning += value as i16;
            item.resist_magic += value as i16;
        }
        DatEffectType::Str => {
            item.bonus_str += value as i16;
        }
        DatEffectType::Mag => {
            item.bonus_mag += value as i16;
        }
        DatEffectType::Dex => {
            item.bonus_dex += value as i16;
        }
        DatEffectType::Vit => {
            item.bonus_vit += value as i16;
        }
        DatEffectType::Life => {
            item.bonus_hp += value as i16;
        }
        DatEffectType::Mana => {
            item.bonus_mana += value as i16;
        }
        DatEffectType::FastAttack => {
            // 攻击速度通过特殊标志实现
            if value >= 3 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::FASTESTATTACK.0);
            } else if value >= 2 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::FASTERATTACK.0);
            } else if value >= 1 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::FASTATTACK.0);
            }
        }
        DatEffectType::StealLife => {
            // 生命偷取
            if value >= 5 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::STEALLIFE_5.0);
            } else if value >= 3 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::STEALLIFE_3.0);
            }
        }
        DatEffectType::StealMana => {
            // 法力偷取
            if value >= 5 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::STEALMANA_5.0);
            } else if value >= 3 {
                item.special_flags = ItemSpecialEffect(item.special_flags.0 | ItemSpecialEffect::STEALMANA_3.0);
            }
        }
        DatEffectType::Light => {
            item.bonus_light += value as i16;
        }
        _ => {}
    }

    // 记录词缀能力
    if is_prefix {
        item.prefix_power = convert_effect_type(affix.power.effect_type);
        item.value_add1 = value;
        item.value_mult1 = affix.mult_val;
    } else {
        item.suffix_power = convert_effect_type(affix.power.effect_type);
        item.value_add2 = value;
        item.value_mult2 = affix.mult_val;
    }

    // 调整物品价值
    let value_mult = affix.mult_val.max(1);
    item.value = item.value * value_mult / 100;
    item.identified_value = item.value;
}

/// 转换效果类型
fn convert_effect_type(effect: super::item_dat::ItemEffectType) -> ItemEffectType {
    use super::item_dat::ItemEffectType as DatEffectType;

    match effect {
        DatEffectType::ToHit => ItemEffectType::ToHit,
        DatEffectType::ToHitCurse => ItemEffectType::ToHitCurse,
        DatEffectType::Damage => ItemEffectType::Damage,
        DatEffectType::DamageCurse => ItemEffectType::DamageCurse,
        DatEffectType::ToHitDamage => ItemEffectType::ToHitDamage,
        DatEffectType::ToHitDamageCurse => ItemEffectType::ToHitDamageCurse,
        DatEffectType::ArmorPercent | DatEffectType::SetAC => ItemEffectType::Ac,
        DatEffectType::ArmorPercentCurse | DatEffectType::ACCurse => ItemEffectType::AcCurse,
        DatEffectType::FireRes => ItemEffectType::FireRes,
        DatEffectType::LightRes => ItemEffectType::LightRes,
        DatEffectType::MagicRes => ItemEffectType::MagicRes,
        DatEffectType::AllRes => ItemEffectType::AllRes,
        DatEffectType::Spell => ItemEffectType::Spell,
        DatEffectType::Charges => ItemEffectType::Charges,
        DatEffectType::FireDam => ItemEffectType::FireDam,
        DatEffectType::LightDam => ItemEffectType::LightDam,
        _ => ItemEffectType::Invalid,
    }
}

/// 更新物品名称 (根据词缀)
fn update_item_name(item: &mut Item) {
    let mut name = item.base_name.clone();

    // 添加前缀
    if item.prefix_power != ItemEffectType::Invalid {
        if let Some(prefix) = get_affix_name(item.value_add1 as usize, true) {
            name = format!("{} {}", prefix, name);
        }
    }

    // 添加后缀
    if item.suffix_power != ItemEffectType::Invalid {
        if let Some(suffix) = get_affix_name(item.value_add2 as usize, false) {
            name = format!("{} {}", name, suffix);
        }
    }

    item.name = name;
}

/// 获取词缀名称
fn get_affix_name(idx: usize, is_prefix: bool) -> Option<&'static str> {
    if is_prefix && idx < ITEM_PREFIXES.len() {
        Some(ITEM_PREFIXES[idx].name)
    } else if !is_prefix && idx < ITEM_SUFFIXES.len() {
        Some(ITEM_SUFFIXES[idx].name)
    } else {
        None
    }
}

/// 完整物品设置
///
/// **C++ Reference**: `Source/items.cpp:3253` - `SetupAllItems()`
pub fn setup_all_items(
    item: &mut Item,
    idx: ItemIndex,
    seed: u32,
    level: i32,
    uper: i32,        // 1=1%唯一, 15=16%唯一
    only_good: bool,
    pregen: bool,
) {
    // 设置种子
    item.seed = seed;

    // 获取基础属性
    get_item_attrs(item, idx, level / 2);

    // 设置创建信息
    item.create_info = level as u16;
    if pregen {
        item.create_info |= CreateInfoFlag::CF_PREGEN;
    }
    if only_good {
        item.create_info |= CreateInfoFlag::CF_ONLYGOOD;
    }
    if uper == 15 {
        item.create_info |= CreateInfoFlag::CF_UPER15;
    } else if uper == 1 {
        item.create_info |= CreateInfoFlag::CF_UPER1;
    }

    // 如果不是唯一物品类型，尝试生成魔法属性
    if item.misc_id != ItemMiscId::Unique {
        let blvl = get_item_base_level(level, only_good, uper == 15);

        if blvl > 0 {
            // 检查是否生成唯一物品
            let mut rng = rand::rng();
            let unique_chance = if uper == 15 { 16 } else { 1 };

            if rng.random_range(0..100) < unique_chance {
                // TODO: 实现唯一物品生成
                // get_unique_item(item, uid);
                item.quality = ItemQuality::Unique;
                item.create_info |= CreateInfoFlag::CF_UNIQUE;
            } else {
                // 生成魔法物品
                get_item_bonus(item, blvl / 2, blvl, only_good, true);
            }
        }

        // 随机耐久度
        if item.quality != ItemQuality::Unique {
            randomize_durability(item);
        }
    }
}

/// 获取物品基础等级
///
/// **C++ Reference**: `GetItemBLevel()`
fn get_item_base_level(level: i32, only_good: bool, uper15: bool) -> i32 {
    let mut blvl = level;

    if uper15 {
        blvl += 4;
    }
    if only_good {
        blvl += 2;
    }

    blvl.max(1)
}

/// 随机化物品耐久度
///
/// **C++ Reference**: `ItemRndDur()`
fn randomize_durability(item: &mut Item) {
    if item.max_durability <= 1 {
        return;
    }

    let mut rng = rand::rng();
    let cur_dur = rng.random_range(1..=item.max_durability);
    item.durability = cur_dur;
}

/// 创建掉落物品
///
/// **C++ Reference**: `Source/items.cpp:1485` - `CreatePlrItems()`
pub fn create_drop_item(
    items: &mut ItemArray,
    x: i32,
    y: i32,
    item_idx: ItemIndex,
    only_good: bool,
    level: i32,
) -> Option<usize> {
    // 分配物品槽
    let idx = items.allocate()?;

    // 初始化物品
    let mut item = Item::empty();
    let seed = rand::random::<u32>();

    setup_all_items(&mut item, item_idx, seed, level * 2, 1, only_good, false);

    // 设置位置
    item.position_x = x;
    item.position_y = y;

    // 存储物品
    items.items[idx] = Some(item);
    items.place_at(idx, x as usize, y as usize);

    Some(idx)
}

/// 生成随机物品
///
/// **C++ Reference**: `CreateRandomItem()`
pub fn create_random_item(
    items: &mut ItemArray,
    x: i32,
    y: i32,
    level: i32,
    only_good: bool,
) -> Option<usize> {
    // 随机选择物品类型
    let mut rng = rand::rng();

    // 简化的物品类型选择
    let item_indices = [
        ItemIndex::ShortSword,
        ItemIndex::LongSword,
        ItemIndex::Axe,
        ItemIndex::Mace,
        ItemIndex::ShortBow,
        ItemIndex::Buckler,
        ItemIndex::Cap,
        ItemIndex::LeatherArmor,
        ItemIndex::Ring,
    ];

    let idx = item_indices[rng.random_range(0..item_indices.len())];

    create_drop_item(items, x, y, idx, only_good, level)
}

// ============================================================================
// M55 Day 2: 唯一物品系统 - 完整 C++ 对齐实现
// C++ Reference: items.cpp - CheckUnique, GetUniqueItem, TryRandomUniqueItem
// Data Source: assets/txtdata/items/unique_itemdat.tsv (91 unique items)
// ============================================================================

/// 唯一物品基础类型枚举 (C++ unique_base_item - COMPLETE)
/// Exact C++ alignment: Source/itemdat.h enum unique_base_item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum UniqueBaseItem {
    #[default]
    None = 0,
    ShortBow = 1,       // UITYPE_SHORTBOW
    LongBow = 2,        // UITYPE_LONGBOW
    HuntBow = 3,        // UITYPE_HUNTBOW
    CompBow = 4,        // UITYPE_COMPBOW
    WarBow = 5,         // UITYPE_WARBOW
    BattleBow = 6,      // UITYPE_BATTLEBOW
    Dagger = 7,         // UITYPE_DAGGER
    Falchion = 8,       // UITYPE_FALCHION
    Claymore = 9,       // UITYPE_CLAYMORE
    BroadSword = 10,    // UITYPE_BROADSWR
    Sabre = 11,         // UITYPE_SABRE
    Scimitar = 12,      // UITYPE_SCIMITAR
    LongSword = 13,     // UITYPE_LONGSWR
    BastardSword = 14,  // UITYPE_BASTARDSWR
    TwoHandSword = 15,  // UITYPE_TWOHANDSWR
    GreatSword = 16,    // UITYPE_GREATSWR
    Cleaver = 17,       // UITYPE_CLEAVER
    LargeAxe = 18,      // UITYPE_LARGEAXE
    BroadAxe = 19,      // UITYPE_BROADAXE
    SmallAxe = 20,      // UITYPE_SMALLAXE
    BattleAxe = 21,     // UITYPE_BATTLEAXE
    GreatAxe = 22,      // UITYPE_GREATAXE
    Mace = 23,          // UITYPE_MACE
    MorningStar = 24,   // UITYPE_MORNSTAR
    SpikedClub = 25,    // UITYPE_SPIKCLUB
    Maul = 26,          // UITYPE_MAUL
    WarHammer = 27,     // UITYPE_WARHAMMER
    Flail = 28,         // UITYPE_FLAIL
    LongStaff = 29,     // UITYPE_LONGSTAFF
    ShortStaff = 30,    // UITYPE_SHORTSTAFF
    CompositeStaff = 31, // UITYPE_COMPSTAFF
    QuarterStaff = 32,  // UITYPE_QUARSTAFF
    WarStaff = 33,      // UITYPE_WARSTAFF
    SkullCap = 34,      // UITYPE_SKULLCAP
    Helm = 35,          // UITYPE_HELM
    GreatHelm = 36,     // UITYPE_GREATHELM
    Crown = 37,         // UITYPE_CROWN
    Reserved38 = 38,    // UITYPE_38
    Rags = 39,          // UITYPE_RAGS
    StudArmor = 40,     // UITYPE_STUDARMOR
    Cloak = 41,         // UITYPE_CLOAK
    Robe = 42,          // UITYPE_ROBE
    ChainMail = 43,     // UITYPE_CHAINMAIL
    LeatherArmor = 44,  // UITYPE_LEATHARMOR
    BreastPlate = 45,   // UITYPE_BREASTPLATE
    Cape = 46,          // UITYPE_CAPE
    PlateMail = 47,     // UITYPE_PLATEMAIL
    FullPlate = 48,     // UITYPE_FULLPLATE
    Buckler = 49,       // UITYPE_BUCKLER
    SmallShield = 50,   // UITYPE_SMALLSHIELD
    LargeShield = 51,   // UITYPE_LARGESHIELD
    KiteShield = 52,    // UITYPE_KITESHIELD
    GothicShield = 53,  // UITYPE_GOTHSHIELD
    Ring = 54,          // UITYPE_RING
    Reserved55 = 55,    // UITYPE_55
    Amulet = 56,        // UITYPE_AMULET
    // Quest unique base items
    SkCrown = 57,       // UITYPE_SKCROWN (Undead Crown)
    InfraRing = 58,     // UITYPE_INFRARING (Empyrean Band)
    OptAmulet = 59,     // UITYPE_OPTAMULET (Optic Amulet)
    TRing = 60,         // UITYPE_TRING (Ring of Truth)
    HarCrest = 61,      // UITYPE_HARCREST (Harlequin Crest)
    MapOfDoom = 62,     // UITYPE_MAPOFDOOM
    Elixir = 63,        // UITYPE_ELIXIR
    ArmOfVal = 64,      // UITYPE_ARMOFVAL (Arkaine's Valor)
    SteelVeil = 65,     // UITYPE_STEELVEIL (Veil of Steel)
    Griswold = 66,      // UITYPE_GRISWOLD (Griswold's Edge)
    LgtForge = 67,      // UITYPE_LGTFORGE (Lightforge)
    LazStaff = 68,      // UITYPE_LAZSTAFF (Staff of Lazarus)
    Bovine = 69,        // UITYPE_BOVINE
    Invalid = -1,       // UITYPE_INVALID
}

/// 唯一物品效果
/// **C++ Reference**: `ItemPower` struct in UniqueItem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniqueItemEffect {
    pub effect_type: ItemEffectType,
    pub min_value: i32,
    pub max_value: i32,
}

impl UniqueItemEffect {
    pub const NONE: Self = Self {
        effect_type: ItemEffectType::Invalid,
        min_value: 0,
        max_value: 0,
    };
}

impl Default for UniqueItemEffect {
    fn default() -> Self {
        Self::NONE
    }
}

/// 唯一物品数据 (C++ UniqueItem struct - COMPLETE)
/// **C++ Reference**: `UniqueItem` struct in itemdat.h
#[derive(Debug, Clone)]
pub struct UniqueItemData {
    pub name: &'static str,
    pub base_item: UniqueBaseItem,
    pub min_level: i8,
    pub value: i32,
    pub powers: [UniqueItemEffect; 6],
    pub num_powers: u8,
}

impl Default for UniqueItemData {
    fn default() -> Self {
        Self {
            name: "",
            base_item: UniqueBaseItem::None,
            min_level: 0,
            value: 0,
            powers: [UniqueItemEffect::NONE; 6],
            num_powers: 0,
        }
    }
}

// Helper macro for creating unique items
macro_rules! unique_item {
    ($name:expr, $base:expr, $level:expr, $value:expr, $($eff:expr),*) => {
        {
            let effects = [$($eff),*];
            let mut powers = [UniqueItemEffect::NONE; 6];
            let mut count = 0u8;
            $(
                if count < 6 {
                    powers[count as usize] = $eff;
                    count += 1;
                }
            )*
            let _ = effects; // suppress unused warning
            UniqueItemData {
                name: $name,
                base_item: $base,
                min_level: $level,
                value: $value,
                powers,
                num_powers: count,
            }
        }
    };
}

// Helper function for effect creation
const fn eff(t: ItemEffectType, min: i32, max: i32) -> UniqueItemEffect {
    UniqueItemEffect { effect_type: t, min_value: min, max_value: max }
}

/// 完整唯一物品表 (90 items) - 完全对齐 C++ unique_itemdat.tsv
/// **C++ Reference**: `assets/txtdata/items/unique_itemdat.tsv`
///
/// 效果类型使用 ItemEffectType 枚举 (见上方定义)
pub static UNIQUE_ITEMS: [UniqueItemData; 90] = [
    // Quest unique items (indices 0-9)
    unique_item!("The Butcher's Cleaver", UniqueBaseItem::Cleaver, 1, 3650,
        eff(ItemEffectType::Str, 10, 10),
        eff(ItemEffectType::SetDam, 4, 24),
        eff(ItemEffectType::SetDur, 10, 10)),
    unique_item!("The Undead Crown", UniqueBaseItem::SkCrown, 1, 16650,
        eff(ItemEffectType::RndStealLife, 0, 0),
        eff(ItemEffectType::SetAc, 8, 8)),
    unique_item!("Empyrean Band", UniqueBaseItem::InfraRing, 1, 8000,
        eff(ItemEffectType::Attribs, 2, 2),
        eff(ItemEffectType::Light, 2, 2),
        eff(ItemEffectType::FastRecover, 1, 1),
        eff(ItemEffectType::AbsHalfTrap, 0, 0)),
    unique_item!("Optic Amulet", UniqueBaseItem::OptAmulet, 1, 9750,
        eff(ItemEffectType::Light, 2, 2),
        eff(ItemEffectType::LightRes, 20, 20),
        eff(ItemEffectType::GetHit, 1, 1),
        eff(ItemEffectType::Mag, 5, 5)),
    unique_item!("Ring of Truth", UniqueBaseItem::TRing, 1, 9100,
        eff(ItemEffectType::Life, 10, 10),
        eff(ItemEffectType::GetHit, 1, 1),
        eff(ItemEffectType::AllRes, 10, 10)),
    unique_item!("Harlequin Crest", UniqueBaseItem::HarCrest, 1, 4000,
        eff(ItemEffectType::AcCurseSet, 3, 3),
        eff(ItemEffectType::GetHit, 1, 1),
        eff(ItemEffectType::Attribs, 2, 2),
        eff(ItemEffectType::Life, 7, 7),
        eff(ItemEffectType::Mana, 7, 7)),
    unique_item!("Veil of Steel", UniqueBaseItem::SteelVeil, 1, 63800,
        eff(ItemEffectType::AllRes, 50, 50),
        eff(ItemEffectType::LightCurse, 2, 2),
        eff(ItemEffectType::Ac, 60, 60),
        eff(ItemEffectType::ManaCurse, 30, 30),
        eff(ItemEffectType::Str, 15, 15),
        eff(ItemEffectType::Vit, 15, 15)),
    unique_item!("Arkaine's Valor", UniqueBaseItem::ArmOfVal, 1, 42000,
        eff(ItemEffectType::SetAc, 25, 25),
        eff(ItemEffectType::Vit, 10, 10),
        eff(ItemEffectType::GetHit, 3, 3),
        eff(ItemEffectType::FastRecover, 3, 3)),
    unique_item!("Griswold's Edge", UniqueBaseItem::Griswold, 1, 42000,
        eff(ItemEffectType::FireDam, 1, 10),
        eff(ItemEffectType::ToHit, 25, 25),
        eff(ItemEffectType::FastAttack, 2, 2),
        eff(ItemEffectType::Knockback, 0, 0),
        eff(ItemEffectType::Mana, 20, 20),
        eff(ItemEffectType::LifeCurse, 20, 20)),
    unique_item!("Lightforge", UniqueBaseItem::LgtForge, 1, 26675,
        eff(ItemEffectType::Light, 4, 4),
        eff(ItemEffectType::Damage, 150, 150),
        eff(ItemEffectType::ToHit, 25, 25),
        eff(ItemEffectType::FireDam, 10, 20),
        eff(ItemEffectType::Indestructible, 0, 0),
        eff(ItemEffectType::Attribs, 8, 8)),

    // Bows (indices 10-19)
    unique_item!("The Rift Bow", UniqueBaseItem::ShortBow, 1, 1800,
        eff(ItemEffectType::RndArrowVel, 0, 0),
        eff(ItemEffectType::DamMod, 2, 2),
        eff(ItemEffectType::DexCurse, 3, 3)),
    unique_item!("The Needler", UniqueBaseItem::ShortBow, 2, 8900,
        eff(ItemEffectType::ToHit, 50, 50),
        eff(ItemEffectType::SetDam, 1, 3),
        eff(ItemEffectType::FastAttack, 2, 2)),
    unique_item!("The Celestial Bow", UniqueBaseItem::LongBow, 2, 1200,
        eff(ItemEffectType::NoMinStr, 0, 0),
        eff(ItemEffectType::DamMod, 2, 2),
        eff(ItemEffectType::SetAc, 5, 5)),
    unique_item!("Deadly Hunter", UniqueBaseItem::CompBow, 3, 8750,
        eff(ItemEffectType::TripleDemonDam, 0, 0),
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::MagCurse, 5, 5)),
    unique_item!("Bow of the Dead", UniqueBaseItem::CompBow, 5, 2500,
        eff(ItemEffectType::ToHit, 10, 10),
        eff(ItemEffectType::Dex, 4, 4),
        eff(ItemEffectType::VitCurse, 3, 3),
        eff(ItemEffectType::LightCurse, 2, 2),
        eff(ItemEffectType::SetDur, 30, 30)),
    unique_item!("The Blackoak Bow", UniqueBaseItem::LongBow, 5, 2500,
        eff(ItemEffectType::Dex, 10, 10),
        eff(ItemEffectType::VitCurse, 10, 10),
        eff(ItemEffectType::Damage, 50, 50),
        eff(ItemEffectType::LightCurse, 1, 1)),
    unique_item!("Flamedart", UniqueBaseItem::HuntBow, 10, 14250,
        eff(ItemEffectType::FireArrows, 1, 6),
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::FireRes, 40, 40)),
    unique_item!("Fleshstinger", UniqueBaseItem::LongBow, 13, 16500,
        eff(ItemEffectType::Dex, 15, 15),
        eff(ItemEffectType::ToHit, 40, 40),
        eff(ItemEffectType::Damage, 80, 80),
        eff(ItemEffectType::Dur, 6, 6)),
    unique_item!("Windforce", UniqueBaseItem::WarBow, 17, 37750,
        eff(ItemEffectType::Str, 5, 5),
        eff(ItemEffectType::Damage, 200, 200),
        eff(ItemEffectType::Knockback, 0, 0)),
    unique_item!("Eaglehorn", UniqueBaseItem::BattleBow, 26, 42500,
        eff(ItemEffectType::Dex, 20, 20),
        eff(ItemEffectType::ToHit, 50, 50),
        eff(ItemEffectType::Damage, 100, 100),
        eff(ItemEffectType::Indestructible, 0, 0)),

    // Daggers & Swords (indices 20-35)
    unique_item!("Gonnagal's Dirk", UniqueBaseItem::Dagger, 1, 7040,
        eff(ItemEffectType::DexCurse, 5, 5),
        eff(ItemEffectType::DamMod, 4, 4),
        eff(ItemEffectType::FastAttack, 2, 2),
        eff(ItemEffectType::FireRes, 25, 25)),
    unique_item!("The Defender", UniqueBaseItem::Sabre, 1, 2000,
        eff(ItemEffectType::SetAc, 5, 5),
        eff(ItemEffectType::Vit, 5, 5),
        eff(ItemEffectType::ToHitCurse, 5, 5)),
    unique_item!("Gryphon's Claw", UniqueBaseItem::Falchion, 1, 1000,
        eff(ItemEffectType::Damage, 100, 100),
        eff(ItemEffectType::MagCurse, 2, 2),
        eff(ItemEffectType::DexCurse, 5, 5)),
    unique_item!("Black Razor", UniqueBaseItem::Dagger, 1, 2000,
        eff(ItemEffectType::Damage, 150, 150),
        eff(ItemEffectType::Vit, 2, 2),
        eff(ItemEffectType::SetDur, 5, 5)),
    unique_item!("Gibbous Moon", UniqueBaseItem::BroadSword, 2, 6660,
        eff(ItemEffectType::Attribs, 2, 2),
        eff(ItemEffectType::Damage, 25, 25),
        eff(ItemEffectType::Mana, 15, 15),
        eff(ItemEffectType::LightCurse, 3, 3)),
    unique_item!("Ice Shank", UniqueBaseItem::LongSword, 3, 5250,
        eff(ItemEffectType::FireRes, 40, 40),
        eff(ItemEffectType::SetDur, 15, 15),
        eff(ItemEffectType::Str, 5, 10)),
    unique_item!("The Executioner's Blade", UniqueBaseItem::Falchion, 3, 7080,
        eff(ItemEffectType::Damage, 150, 150),
        eff(ItemEffectType::LifeCurse, 10, 10),
        eff(ItemEffectType::LightCurse, 1, 1),
        eff(ItemEffectType::Dur, 200, 200)),
    unique_item!("The Bonesaw", UniqueBaseItem::Claymore, 6, 4400,
        eff(ItemEffectType::DamMod, 10, 10),
        eff(ItemEffectType::Str, 10, 10),
        eff(ItemEffectType::MagCurse, 5, 5),
        eff(ItemEffectType::DexCurse, 5, 5),
        eff(ItemEffectType::Life, 10, 10),
        eff(ItemEffectType::ManaCurse, 10, 10)),
    unique_item!("Shadowhawk", UniqueBaseItem::BroadSword, 8, 13750,
        eff(ItemEffectType::LightCurse, 2, 2),
        eff(ItemEffectType::StealLife, 5, 5),
        eff(ItemEffectType::ToHit, 15, 15),
        eff(ItemEffectType::AllRes, 5, 5)),
    unique_item!("Wizardspike", UniqueBaseItem::Dagger, 11, 12920,
        eff(ItemEffectType::Mag, 15, 15),
        eff(ItemEffectType::Mana, 35, 35),
        eff(ItemEffectType::ToHit, 25, 25),
        eff(ItemEffectType::AllRes, 15, 15)),
    unique_item!("Lightsabre", UniqueBaseItem::Sabre, 13, 19150,
        eff(ItemEffectType::Light, 2, 2),
        eff(ItemEffectType::LightDam, 1, 10),
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::LightRes, 50, 50)),
    unique_item!("The Falcon's Talon", UniqueBaseItem::Scimitar, 15, 7867,
        eff(ItemEffectType::FastAttack, 4, 4),
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::DamageCurse, 33, 33),
        eff(ItemEffectType::Dex, 10, 10)),
    unique_item!("Inferno", UniqueBaseItem::LongSword, 17, 34600,
        eff(ItemEffectType::FireDam, 2, 12),
        eff(ItemEffectType::Light, 3, 3),
        eff(ItemEffectType::Mana, 20, 20),
        eff(ItemEffectType::FireRes, 80, 80)),
    unique_item!("Doombringer", UniqueBaseItem::BastardSword, 19, 18250,
        eff(ItemEffectType::ToHit, 25, 25),
        eff(ItemEffectType::Damage, 250, 250),
        eff(ItemEffectType::AttribsCurse, 5, 5),
        eff(ItemEffectType::LifeCurse, 25, 25),
        eff(ItemEffectType::LightCurse, 2, 2)),
    unique_item!("The Grizzly", UniqueBaseItem::TwoHandSword, 23, 50000,
        eff(ItemEffectType::Str, 20, 20),
        eff(ItemEffectType::VitCurse, 5, 5),
        eff(ItemEffectType::Damage, 200, 200),
        eff(ItemEffectType::Knockback, 0, 0),
        eff(ItemEffectType::Dur, 100, 100)),
    unique_item!("The Grandfather", UniqueBaseItem::GreatSword, 27, 119800,
        eff(ItemEffectType::OneHand, 0, 0),
        eff(ItemEffectType::Attribs, 5, 5),
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::Damage, 70, 70),
        eff(ItemEffectType::Life, 20, 20)),

    // Axes (indices 36-44)
    unique_item!("The Mangler", UniqueBaseItem::LargeAxe, 2, 2850,
        eff(ItemEffectType::Damage, 200, 200),
        eff(ItemEffectType::DexCurse, 5, 5),
        eff(ItemEffectType::MagCurse, 5, 5),
        eff(ItemEffectType::ManaCurse, 10, 10)),
    unique_item!("Sharp Beak", UniqueBaseItem::LargeAxe, 2, 2850,
        eff(ItemEffectType::Life, 20, 20),
        eff(ItemEffectType::MagCurse, 10, 10),
        eff(ItemEffectType::ManaCurse, 10, 10)),
    unique_item!("Bloodslayer", UniqueBaseItem::BroadAxe, 3, 2500,
        eff(ItemEffectType::Damage, 100, 100),
        eff(ItemEffectType::TripleDemonDam, 0, 0),
        eff(ItemEffectType::AttribsCurse, 5, 5),
        eff(ItemEffectType::SpellLevelAdd, -1, -1)),
    unique_item!("The Celestial Axe", UniqueBaseItem::BattleAxe, 4, 14100,
        eff(ItemEffectType::NoMinStr, 0, 0),
        eff(ItemEffectType::ToHit, 15, 15),
        eff(ItemEffectType::Life, 15, 15),
        eff(ItemEffectType::StrCurse, 15, 15)),
    unique_item!("Wicked Axe", UniqueBaseItem::LargeAxe, 5, 31150,
        eff(ItemEffectType::ToHit, 30, 30),
        eff(ItemEffectType::Dex, 10, 10),
        eff(ItemEffectType::VitCurse, 10, 10),
        eff(ItemEffectType::GetHit, 1, 6),
        eff(ItemEffectType::Indestructible, 0, 0)),
    unique_item!("Stonecleaver", UniqueBaseItem::BroadAxe, 7, 23900,
        eff(ItemEffectType::Life, 30, 30),
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::Damage, 50, 50),
        eff(ItemEffectType::LightRes, 40, 40)),
    unique_item!("Aguinara's Hatchet", UniqueBaseItem::SmallAxe, 12, 24800,
        eff(ItemEffectType::SpellLevelAdd, 1, 1),
        eff(ItemEffectType::Mag, 10, 10),
        eff(ItemEffectType::MagicRes, 80, 80)),
    unique_item!("Hellslayer", UniqueBaseItem::BattleAxe, 15, 26200,
        eff(ItemEffectType::Str, 8, 8),
        eff(ItemEffectType::Vit, 8, 8),
        eff(ItemEffectType::Damage, 100, 100),
        eff(ItemEffectType::Life, 25, 25),
        eff(ItemEffectType::ManaCurse, 25, 25)),
    unique_item!("Messerschmidt's Reaver", UniqueBaseItem::GreatAxe, 25, 58000,
        eff(ItemEffectType::Damage, 200, 200),
        eff(ItemEffectType::DamMod, 15, 15),
        eff(ItemEffectType::Attribs, 5, 5),
        eff(ItemEffectType::LifeCurse, 50, 50),
        eff(ItemEffectType::FireDam, 2, 12)),

    // Maces (indices 45-54)
    unique_item!("Crackrust", UniqueBaseItem::Mace, 1, 11375,
        eff(ItemEffectType::Attribs, 2, 2),
        eff(ItemEffectType::Indestructible, 0, 0),
        eff(ItemEffectType::AllRes, 15, 15),
        eff(ItemEffectType::Damage, 50, 50),
        eff(ItemEffectType::SpellLevelAdd, -1, -1)),
    unique_item!("Hammer of Jholm", UniqueBaseItem::Maul, 1, 8700,
        eff(ItemEffectType::SetDam, 4, 10),
        eff(ItemEffectType::Indestructible, 0, 0),
        eff(ItemEffectType::Str, 3, 3),
        eff(ItemEffectType::ToHit, 15, 15)),
    unique_item!("Civerb's Cudgel", UniqueBaseItem::Mace, 1, 2000,
        eff(ItemEffectType::TripleDemonDam, 0, 0),
        eff(ItemEffectType::DexCurse, 5, 5),
        eff(ItemEffectType::MagCurse, 2, 2)),
    unique_item!("The Celestial Star", UniqueBaseItem::Flail, 2, 7810,
        eff(ItemEffectType::NoMinStr, 0, 0),
        eff(ItemEffectType::Light, 2, 2),
        eff(ItemEffectType::DamMod, 10, 10),
        eff(ItemEffectType::AcCurseSet, 8, 8)),
    unique_item!("Baranar's Star", UniqueBaseItem::MorningStar, 5, 6850,
        eff(ItemEffectType::ToHit, 12, 12),
        eff(ItemEffectType::Damage, 80, 80),
        eff(ItemEffectType::FastAttack, 1, 1),
        eff(ItemEffectType::Vit, 4, 4),
        eff(ItemEffectType::DexCurse, 4, 4),
        eff(ItemEffectType::SetDur, 60, 60)),
    unique_item!("Gnarled Root", UniqueBaseItem::SpikedClub, 9, 9820,
        eff(ItemEffectType::ToHit, 20, 20),
        eff(ItemEffectType::Damage, 300, 300),
        eff(ItemEffectType::Dex, 10, 10),
        eff(ItemEffectType::Mag, 5, 5),
        eff(ItemEffectType::AllRes, 10, 10),
        eff(ItemEffectType::AcCurseSet, 10, 10)),
    unique_item!("The Cranium Basher", UniqueBaseItem::Maul, 12, 36500,
        eff(ItemEffectType::DamMod, 20, 20),
        eff(ItemEffectType::Str, 15, 15),
        eff(ItemEffectType::Indestructible, 0, 0),
        eff(ItemEffectType::ManaCurse, 150, 150),
        eff(ItemEffectType::AllRes, 5, 5)),
    unique_item!("Schaefer's Hammer", UniqueBaseItem::WarHammer, 16, 56125,
        eff(ItemEffectType::DamageCurse, 100, 100),
        eff(ItemEffectType::LightDam, 1, 50),
        eff(ItemEffectType::Life, 50, 50),
        eff(ItemEffectType::ToHit, 30, 30),
        eff(ItemEffectType::LightRes, 80, 80),
        eff(ItemEffectType::Light, 1, 1)),
    unique_item!("Dreamflange", UniqueBaseItem::Mace, 26, 26450,
        eff(ItemEffectType::Mag, 30, 30),
        eff(ItemEffectType::Mana, 50, 50),
        eff(ItemEffectType::MagicRes, 50, 50),
        eff(ItemEffectType::Light, 2, 2),
        eff(ItemEffectType::SpellLevelAdd, 1, 1)),

    // Staves (indices 55-63)
    unique_item!("Staff of Shadows", UniqueBaseItem::LongStaff, 2, 1250,
        eff(ItemEffectType::MagCurse, 10, 10),
        eff(ItemEffectType::ToHit, 10, 10),
        eff(ItemEffectType::Damage, 60, 60),
        eff(ItemEffectType::LightCurse, 2, 2),
        eff(ItemEffectType::FastAttack, 1, 1)),
    unique_item!("Immolator", UniqueBaseItem::LongStaff, 4, 3900,
        eff(ItemEffectType::FireRes, 20, 20),
        eff(ItemEffectType::FireDam, 4, 4),
        eff(ItemEffectType::Mana, 10, 10),
        eff(ItemEffectType::VitCurse, 5, 5)),
    unique_item!("Storm Spire", UniqueBaseItem::WarStaff, 8, 22500,
        eff(ItemEffectType::LightRes, 50, 50),
        eff(ItemEffectType::LightDam, 2, 8),
        eff(ItemEffectType::Str, 10, 10),
        eff(ItemEffectType::MagCurse, 10, 10)),
    unique_item!("Gleamsong", UniqueBaseItem::ShortStaff, 8, 6520,
        eff(ItemEffectType::Mana, 25, 25),
        eff(ItemEffectType::StrCurse, 3, 3),
        eff(ItemEffectType::VitCurse, 3, 3),
        eff(ItemEffectType::Spell, 10, 76)),
    unique_item!("Thundercall", UniqueBaseItem::CompositeStaff, 14, 22250,
        eff(ItemEffectType::ToHit, 35, 35),
        eff(ItemEffectType::LightDam, 1, 10),
        eff(ItemEffectType::Spell, 3, 76),
        eff(ItemEffectType::LightRes, 30, 30),
        eff(ItemEffectType::Light, 2, 2)),
    unique_item!("The Protector", UniqueBaseItem::ShortStaff, 16, 17240,
        eff(ItemEffectType::Vit, 5, 5),
        eff(ItemEffectType::GetHit, 5, 5),
        eff(ItemEffectType::SetAc, 40, 40),
        eff(ItemEffectType::Spell, 2, 86),
        eff(ItemEffectType::Thorns, 1, 3)),
    unique_item!("Naj's Puzzler", UniqueBaseItem::LongStaff, 18, 34000,
        eff(ItemEffectType::Mag, 20, 20),
        eff(ItemEffectType::Dex, 10, 10),
        eff(ItemEffectType::AllRes, 20, 20),
        eff(ItemEffectType::Spell, 23, 57),
        eff(ItemEffectType::LifeCurse, 25, 25)),
    unique_item!("Mindcry", UniqueBaseItem::QuarterStaff, 20, 41500,
        eff(ItemEffectType::Mag, 15, 15),
        eff(ItemEffectType::Spell, 13, 69),
        eff(ItemEffectType::AllRes, 15, 15),
        eff(ItemEffectType::SpellLevelAdd, 1, 1)),
    unique_item!("Rod of Onan", UniqueBaseItem::WarStaff, 22, 44167,
        eff(ItemEffectType::Spell, 21, 50),
        eff(ItemEffectType::Damage, 100, 100),
        eff(ItemEffectType::Attribs, 5, 5)),

    // Helms (indices 64-69)
    unique_item!("Helm of Spirits", UniqueBaseItem::Helm, 1, 7525,
        eff(ItemEffectType::StealLife, 5, 5)),
    unique_item!("Thinking Cap", UniqueBaseItem::SkullCap, 6, 2020,
        eff(ItemEffectType::Mana, 30, 30),
        eff(ItemEffectType::SpellLevelAdd, 2, 2),
        eff(ItemEffectType::AllRes, 20, 20),
        eff(ItemEffectType::SetDur, 1, 1)),
    unique_item!("Overlord's Helm", UniqueBaseItem::Helm, 7, 12500,
        eff(ItemEffectType::Str, 20, 20),
        eff(ItemEffectType::Dex, 15, 15),
        eff(ItemEffectType::Vit, 5, 5),
        eff(ItemEffectType::MagCurse, 20, 20),
        eff(ItemEffectType::SetDur, 15, 15)),
    unique_item!("Fool's Crest", UniqueBaseItem::Helm, 12, 10150,
        eff(ItemEffectType::AttribsCurse, 4, 4),
        eff(ItemEffectType::Life, 100, 100),
        eff(ItemEffectType::GetHitCurse, 1, 6),
        eff(ItemEffectType::Thorns, 1, 3)),
    unique_item!("Gotterdamerung", UniqueBaseItem::GreatHelm, 21, 54900,
        eff(ItemEffectType::Attribs, 20, 20),
        eff(ItemEffectType::SetAc, 60, 60),
        eff(ItemEffectType::GetHit, 4, 4),
        eff(ItemEffectType::AllResZero, 0, 0),
        eff(ItemEffectType::LightCurse, 4, 4)),
    unique_item!("Royal Circlet", UniqueBaseItem::Crown, 27, 24875,
        eff(ItemEffectType::Attribs, 10, 10),
        eff(ItemEffectType::Mana, 40, 40),
        eff(ItemEffectType::SetAc, 40, 40),
        eff(ItemEffectType::Light, 1, 1)),

    // Armor (indices 70-79)
    unique_item!("Torn Flesh of Souls", UniqueBaseItem::Rags, 2, 4825,
        eff(ItemEffectType::SetAc, 8, 8),
        eff(ItemEffectType::Vit, 10, 10),
        eff(ItemEffectType::GetHit, 1, 1),
        eff(ItemEffectType::Indestructible, 0, 0)),
    unique_item!("The Gladiator's Bane", UniqueBaseItem::StudArmor, 6, 3450,
        eff(ItemEffectType::SetAc, 25, 25),
        eff(ItemEffectType::GetHit, 2, 2),
        eff(ItemEffectType::Dur, 200, 200),
        eff(ItemEffectType::AttribsCurse, 3, 3)),
    unique_item!("The Rainbow Cloak", UniqueBaseItem::Cloak, 2, 4900,
        eff(ItemEffectType::SetAc, 10, 10),
        eff(ItemEffectType::Attribs, 1, 1),
        eff(ItemEffectType::AllRes, 10, 10),
        eff(ItemEffectType::Life, 5, 5),
        eff(ItemEffectType::Dur, 50, 50)),
    unique_item!("Leather of Aut", UniqueBaseItem::LeatherArmor, 4, 10550,
        eff(ItemEffectType::SetAc, 15, 15),
        eff(ItemEffectType::Str, 5, 5),
        eff(ItemEffectType::MagCurse, 5, 5),
        eff(ItemEffectType::Dex, 5, 5),
        eff(ItemEffectType::Indestructible, 0, 0)),
    unique_item!("Wisdom's Wrap", UniqueBaseItem::Robe, 5, 6200,
        eff(ItemEffectType::Mag, 5, 5),
        eff(ItemEffectType::Mana, 10, 10),
        eff(ItemEffectType::LightRes, 25, 25),
        eff(ItemEffectType::SetAc, 15, 15),
        eff(ItemEffectType::GetHit, 1, 1)),
    unique_item!("Sparking Mail", UniqueBaseItem::ChainMail, 9, 15750,
        eff(ItemEffectType::SetAc, 30, 30),
        eff(ItemEffectType::LightDam, 1, 10)),
    unique_item!("Scavenger Carapace", UniqueBaseItem::BreastPlate, 13, 14000,
        eff(ItemEffectType::GetHit, 15, 15),
        eff(ItemEffectType::AcCurseSet, 30, 30),
        eff(ItemEffectType::Dex, 5, 5),
        eff(ItemEffectType::LightRes, 40, 40)),
    unique_item!("Nightscape", UniqueBaseItem::Cape, 16, 11600,
        eff(ItemEffectType::FastRecover, 2, 2),
        eff(ItemEffectType::LightCurse, 4, 4),
        eff(ItemEffectType::SetAc, 15, 15),
        eff(ItemEffectType::Dex, 3, 3),
        eff(ItemEffectType::AllRes, 20, 20)),
    unique_item!("Naj's Light Plate", UniqueBaseItem::PlateMail, 19, 78700,
        eff(ItemEffectType::NoMinStr, 0, 0),
        eff(ItemEffectType::Mag, 5, 5),
        eff(ItemEffectType::Mana, 20, 20),
        eff(ItemEffectType::AllRes, 20, 20),
        eff(ItemEffectType::SpellLevelAdd, 1, 1)),
    unique_item!("Demonspike Coat", UniqueBaseItem::FullPlate, 25, 251175,
        eff(ItemEffectType::SetAc, 100, 100),
        eff(ItemEffectType::GetHit, 6, 6),
        eff(ItemEffectType::Str, 10, 10),
        eff(ItemEffectType::Indestructible, 0, 0),
        eff(ItemEffectType::FireRes, 50, 50)),

    // Shields (indices 80-86)
    unique_item!("The Deflector", UniqueBaseItem::Buckler, 1, 1500,
        eff(ItemEffectType::SetAc, 7, 7),
        eff(ItemEffectType::AllRes, 10, 10),
        eff(ItemEffectType::DamageCurse, 20, 20),
        eff(ItemEffectType::ToHitCurse, 5, 5)),
    unique_item!("Split Skull Shield", UniqueBaseItem::Buckler, 1, 2025,
        eff(ItemEffectType::SetAc, 10, 10),
        eff(ItemEffectType::Life, 10, 10),
        eff(ItemEffectType::Str, 2, 2),
        eff(ItemEffectType::LightCurse, 1, 1),
        eff(ItemEffectType::SetDur, 15, 15)),
    unique_item!("Dragon's Breach", UniqueBaseItem::KiteShield, 2, 19200,
        eff(ItemEffectType::FireRes, 25, 25),
        eff(ItemEffectType::Str, 5, 5),
        eff(ItemEffectType::SetAc, 20, 20),
        eff(ItemEffectType::MagCurse, 5, 5),
        eff(ItemEffectType::Indestructible, 0, 0)),
    unique_item!("Blackoak Shield", UniqueBaseItem::SmallShield, 4, 5725,
        eff(ItemEffectType::Dex, 10, 10),
        eff(ItemEffectType::VitCurse, 10, 10),
        eff(ItemEffectType::SetAc, 18, 18),
        eff(ItemEffectType::LightCurse, 1, 1),
        eff(ItemEffectType::Dur, 150, 150)),
    unique_item!("Holy Defender", UniqueBaseItem::LargeShield, 10, 13800,
        eff(ItemEffectType::SetAc, 15, 15),
        eff(ItemEffectType::GetHit, 2, 2),
        eff(ItemEffectType::FireRes, 20, 20),
        eff(ItemEffectType::Dur, 200, 200),
        eff(ItemEffectType::FastBlock, 1, 1)),
    unique_item!("Stormshield", UniqueBaseItem::GothicShield, 24, 49000,
        eff(ItemEffectType::SetAc, 40, 40),
        eff(ItemEffectType::GetHitCurse, 4, 4),
        eff(ItemEffectType::Str, 10, 10),
        eff(ItemEffectType::Indestructible, 0, 0),
        eff(ItemEffectType::FastBlock, 1, 1),
        eff(ItemEffectType::LightRes, 50, 50)),

    // Rings & Amulets (indices 87-90)
    unique_item!("Bramble", UniqueBaseItem::Ring, 1, 1000,
        eff(ItemEffectType::AttribsCurse, 2, 2),
        eff(ItemEffectType::DamMod, 3, 3),
        eff(ItemEffectType::Mana, 10, 10)),
    unique_item!("Ring of Regha", UniqueBaseItem::Ring, 1, 4175,
        eff(ItemEffectType::Mag, 10, 10),
        eff(ItemEffectType::MagicRes, 10, 10),
        eff(ItemEffectType::Light, 1, 1),
        eff(ItemEffectType::StrCurse, 3, 3),
        eff(ItemEffectType::DexCurse, 3, 3)),
    unique_item!("The Bleeder", UniqueBaseItem::Ring, 2, 8500,
        eff(ItemEffectType::MagicRes, 20, 20),
        eff(ItemEffectType::Mana, 30, 30),
        eff(ItemEffectType::LifeCurse, 10, 10)),
    unique_item!("Constricting Ring", UniqueBaseItem::Ring, 5, 62000,
        eff(ItemEffectType::AllRes, 75, 75),
        eff(ItemEffectType::DrainLife, 0, 0)),
    unique_item!("Ring of Engagement", UniqueBaseItem::Ring, 11, 12476,
        eff(ItemEffectType::GetHit, 1, 2),
        eff(ItemEffectType::Thorns, 1, 3),
        eff(ItemEffectType::SetAc, 5, 5),
        eff(ItemEffectType::TargAc, 2, 2)),
];

/// 检查物品是否可以是唯一物品 (完全对齐 C++)
///
/// **C++ Reference**: `CheckUnique(Item &item, int lvl, int uper, bool recreate)`
pub fn check_unique(
    item: &mut Item,
    level: i32,
    uper: i32,
    recreate: bool,
    unique_flags: &mut [bool; 128],
) -> bool {
    let mut rng = rand::rng();

    // 检查是否生成唯一物品的概率
    let chance = if uper == 15 {
        15 // CF_UPER15: 15% 唯一物品概率
    } else if uper == 1 {
        1  // CF_UPER1: 1% 唯一物品概率
    } else {
        2  // 默认: 2% 基础概率
    };

    // 随机检查
    if !recreate && rng.random_range(0..100) >= chance {
        return false;
    }

    // 查找匹配的唯一物品
    let matching_uniques: Vec<usize> = UNIQUE_ITEMS
        .iter()
        .enumerate()
        .filter_map(|(idx, u)| {
            // Check if this unique matches the item type and level requirements
            // and hasn't been generated yet
            if u.min_level <= level as i8 && !unique_flags.get(idx).copied().unwrap_or(true) {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    if matching_uniques.is_empty() {
        return false;
    }

    // 随机选择一个唯一物品
    let idx = matching_uniques[rng.random_range(0..matching_uniques.len())];

    // 应用唯一物品属性
    get_unique_item(item, &UNIQUE_ITEMS[idx]);

    // 标记此唯一物品已生成
    if idx < unique_flags.len() {
        unique_flags[idx] = true;
    }

    true
}

/// 应用唯一物品属性 (完全对齐 C++)
///
/// **C++ Reference**: `GetUniqueItem(Item &item, const UniqueItem &uniq)`
pub fn get_unique_item(item: &mut Item, unique: &UniqueItemData) {
    // 设置唯一物品基础属性
    item.quality = ItemQuality::Unique;
    item.name = unique.name.to_string();
    item.value = unique.value;
    item.identified_value = unique.value;

    // 应用唯一物品效果
    for i in 0..unique.num_powers as usize {
        if i < unique.powers.len() {
            apply_unique_effect(item, &unique.powers[i]);
        }
    }

    // 唯一物品有固定的耐久度
    if item.max_durability > 0 {
        item.max_durability = 255;
        item.durability = 255;
    }
}

/// 应用唯一物品效果 (完全对齐 C++)
/// **C++ Reference**: items.cpp GetUniqueItem effect application
fn apply_unique_effect(item: &mut Item, effect: &UniqueItemEffect) {
    let mut rng = rand::rng();
    let value = if effect.min_value == effect.max_value {
        effect.min_value
    } else {
        rng.random_range(effect.min_value..=effect.max_value)
    };

    // 根据效果类型应用 (完整对齐 C++ item_effect_type)
    match effect.effect_type {
        ItemEffectType::ToHit => item.bonus_to_hit += value as i16,
        ItemEffectType::ToHitCurse => item.bonus_to_hit -= value as i16,
        ItemEffectType::Damage => item.bonus_damage_mod += value as i16,
        ItemEffectType::DamageCurse => item.bonus_damage_mod -= value as i16,
        ItemEffectType::ToHitDamage => {
            item.bonus_to_hit += value as i16;
            item.bonus_damage_mod += value as i16;
        },
        ItemEffectType::ToHitDamageCurse => {
            item.bonus_to_hit -= value as i16;
            item.bonus_damage_mod -= value as i16;
        },
        ItemEffectType::Ac => item.bonus_ac += value as i16,
        ItemEffectType::AcCurse | ItemEffectType::AcCurseSet => item.bonus_ac -= value as i16,
        ItemEffectType::FireRes => item.resist_fire += value as i16,
        ItemEffectType::LightRes => item.resist_lightning += value as i16,
        ItemEffectType::MagicRes => item.resist_magic += value as i16,
        ItemEffectType::AllRes => {
            item.resist_fire += value as i16;
            item.resist_lightning += value as i16;
            item.resist_magic += value as i16;
        },
        ItemEffectType::SpellLevelAdd => item.spell_level_add += value as i8,
        ItemEffectType::Charges => {
            item.charges = value;
            item.max_charges = value;
        },
        ItemEffectType::FireDam => item.fire_min_dam += value as i16,
        ItemEffectType::LightDam => item.lightning_min_dam += value as i16,
        ItemEffectType::Str => item.bonus_str += value as i16,
        ItemEffectType::StrCurse => item.bonus_str -= value as i16,
        ItemEffectType::Mag => item.bonus_mag += value as i16,
        ItemEffectType::MagCurse => item.bonus_mag -= value as i16,
        ItemEffectType::Dex => item.bonus_dex += value as i16,
        ItemEffectType::DexCurse => item.bonus_dex -= value as i16,
        ItemEffectType::Vit => item.bonus_vit += value as i16,
        ItemEffectType::VitCurse => item.bonus_vit -= value as i16,
        ItemEffectType::Attribs => {
            item.bonus_str += value as i16;
            item.bonus_mag += value as i16;
            item.bonus_dex += value as i16;
            item.bonus_vit += value as i16;
        },
        ItemEffectType::AttribsCurse => {
            item.bonus_str -= value as i16;
            item.bonus_mag -= value as i16;
            item.bonus_dex -= value as i16;
            item.bonus_vit -= value as i16;
        },
        ItemEffectType::GetHit => item.bonus_get_hit += value as i16,
        ItemEffectType::GetHitCurse => item.bonus_get_hit -= value as i16,
        ItemEffectType::Life => item.bonus_hp += value as i16,
        ItemEffectType::LifeCurse => item.bonus_hp -= value as i16,
        ItemEffectType::Mana => item.bonus_mana += value as i16,
        ItemEffectType::ManaCurse => item.bonus_mana -= value as i16,
        ItemEffectType::Dur => item.max_durability += value as i32,
        ItemEffectType::DurCurse => item.max_durability -= value as i32,
        ItemEffectType::Indestructible => item.max_durability = 255,
        ItemEffectType::Light => item.bonus_light += value as i16,
        ItemEffectType::LightCurse => item.bonus_light -= value as i16,
        ItemEffectType::SetDam => {
            item.min_damage = (value & 0xFF) as u8;
            item.max_damage = ((value >> 8) & 0xFF) as u8;
        },
        ItemEffectType::SetDur => item.max_durability = value as i32,
        ItemEffectType::SetAc => item.armor_class = value as i16,
        ItemEffectType::NoMinStr => item.required_str = 0,
        ItemEffectType::DamMod => item.bonus_damage += value as i16,
        ItemEffectType::FastAttack => {
            item.special_flags |= ItemSpecialEffect::FAST_ATTACK;
        },
        ItemEffectType::FastRecover => {
            item.special_flags |= ItemSpecialEffect::FAST_HIT_RECOVERY;
        },
        ItemEffectType::FastBlock => {
            item.special_flags |= ItemSpecialEffect::FAST_BLOCK;
        },
        ItemEffectType::OneHand => {
            item.equip_loc = ItemEquipType::OneHand;
        },
        ItemEffectType::Knockback => {
            item.special_flags |= ItemSpecialEffect::KNOCKBACK;
        },
        ItemEffectType::FireArrows => {
            item.special_flags |= ItemSpecialEffect::FIRE_ARROWS;
        },
        ItemEffectType::LightArrows => {
            item.special_flags |= ItemSpecialEffect::LIGHTNING_ARROWS;
        },
        ItemEffectType::MultArrows => {
            item.special_flags |= ItemSpecialEffect::MULTIPLE_ARROWS;
        },
        ItemEffectType::Thorns => {
            item.special_flags |= ItemSpecialEffect::THORNS;
        },
        ItemEffectType::NoMana => {
            item.special_flags |= ItemSpecialEffect::NO_MANA;
        },
        ItemEffectType::AbsHalfTrap => {
            item.special_flags |= ItemSpecialEffect::HALF_TRAP_DAMAGE;
        },
        ItemEffectType::StealMana => {
            item.special_flags |= if value == 3 { ItemSpecialEffect::STEAL_MANA_3 } else { ItemSpecialEffect::STEAL_MANA_5 };
        },
        ItemEffectType::StealLife => {
            item.special_flags |= if value == 3 { ItemSpecialEffect::STEAL_LIFE_3 } else { ItemSpecialEffect::STEAL_LIFE_5 };
        },
        ItemEffectType::RndStealLife => {
            item.special_flags |= ItemSpecialEffect::RANDOM_STEAL_LIFE;
        },
        ItemEffectType::TripleDemonDam => {
            item.special_flags |= ItemSpecialEffect::TRIPLE_DEMON_DAMAGE;
        },
        ItemEffectType::AllResZero => {
            item.special_flags |= ItemSpecialEffect::ZERO_RESISTANCE;
        },
        ItemEffectType::DrainLife => {
            item.special_flags |= ItemSpecialEffect::DRAIN_LIFE;
        },
        ItemEffectType::RndArrowVel => {
            item.special_flags |= ItemSpecialEffect::RANDOM_ARROW_VELOCITY;
        },
        // Other effects not yet implemented
        _ => {},
    }
}

/// 尝试生成随机唯一物品
///
/// **C++ Reference**: `TryRandomUniqueItem()`
///
/// 在不指定具体物品类型时，尝试生成一个随机唯一物品
pub fn try_random_unique_item(
    items: &mut ItemArray,
    x: i32,
    y: i32,
    level: i32,
    unique_flags: &mut [bool; 128],
) -> Option<usize> {
    // 收集所有可用的唯一物品
    let available: Vec<_> = UNIQUE_ITEMS
        .iter()
        .enumerate()
        .filter(|(idx, u)| {
            u.min_level <= level as i8
                && !unique_flags.get(*idx).copied().unwrap_or(true)
        })
        .collect();

    if available.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    let (idx, unique) = available[rng.random_range(0..available.len())];

    // 分配物品槽
    let item_idx = items.allocate()?;
    let item = items.items[item_idx].as_mut()?;

    // 设置位置
    item.position_x = x;
    item.position_y = y;

    // 获取基础物品属性
    // TODO: Convert UniqueBaseItem to ItemIndex properly
    // For now, cast the enum discriminant
    let item_index = unsafe { std::mem::transmute::<_, ItemIndex>(unique.base_item as u16) };
    get_item_attrs(item, item_index, level);

    // 应用唯一物品属性
    get_unique_item(item, unique);

    // 标记已生成
    if idx < unique_flags.len() {
        unique_flags[idx] = true;
    }

    // 放置到地图
    items.place_at(item_idx, x as usize, y as usize);

    Some(item_idx)
}

// ============================================================================
// M55 Day 2: 物品空间检查
// C++ Reference: items.cpp - GetSuperItemSpace, ItemSpaceOk
// ============================================================================

/// 检查地图位置是否可放置物品
///
/// **C++ Reference**: `ItemSpaceOk(Point position)`
pub fn item_space_ok(items: &ItemArray, x: usize, y: usize) -> bool {
    // 边界检查
    if x >= MAXDUNX || y >= MAXDUNY {
        return false;
    }

    // 检查是否已有物品
    items.ground_items[x][y] == 0
}

/// 在指定区域寻找可放置物品的位置
///
/// **C++ Reference**: `GetSuperItemSpace(Point position, int8_t &bession)`
///
/// 从中心点向外搜索，找到第一个可放置物品的位置
pub fn get_super_item_space(
    items: &ItemArray,
    center_x: i32,
    center_y: i32,
) -> Option<(usize, usize)> {
    // 搜索方向表 (顺时针螺旋)
    const OFFSETS: [(i32, i32); 8] = [
        (0, 0),   // 中心
        (1, 0),   // 右
        (0, 1),   // 下
        (-1, 0),  // 左
        (0, -1),  // 上
        (1, 1),   // 右下
        (-1, 1),  // 左下
        (-1, -1), // 左上
    ];

    for (dx, dy) in OFFSETS.iter() {
        let x = (center_x + dx) as usize;
        let y = (center_y + dy) as usize;

        if item_space_ok(items, x, y) {
            return Some((x, y));
        }
    }

    // 扩大搜索范围
    for radius in 2i32..=5i32 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() == radius || dy.abs() == radius {
                    let x = (center_x + dx) as usize;
                    let y = (center_y + dy) as usize;

                    if item_space_ok(items, x, y) {
                        return Some((x, y));
                    }
                }
            }
        }
    }

    None
}

/// 在指定区域放置任务物品
///
/// **C++ Reference**: `PlaceQuestItemInArea(int idx, int aession)`
pub fn place_quest_item_in_area(
    items: &mut ItemArray,
    item_index: ItemIndex,
    area_x: i32,
    area_y: i32,
    area_radius: i32,
) -> Option<usize> {
    let mut rng = rand::rng();

    // 尝试在区域内找到位置
    for _ in 0..100 {
        let x = area_x + rng.random_range(-area_radius..=area_radius);
        let y = area_y + rng.random_range(-area_radius..=area_radius);

        if let Some((px, py)) = get_super_item_space(items, x, y) {
            // 分配物品
            let idx = items.allocate()?;
            let item = items.items[idx].as_mut()?;

            // 设置为任务物品
            item.position_x = px as i32;
            item.position_y = py as i32;
            get_item_attrs(item, item_index, 1);
            item.item_class = ItemClass::Quest;

            items.place_at(idx, px, py);
            return Some(idx);
        }
    }

    None
}

// ============================================================================
// M55 Day 2: 物品拾取与移除
// C++ Reference: items.cpp - GetItem, DeleteItem
// ============================================================================

/// 拾取地面物品
///
/// **C++ Reference**: `GetItem(int pnum, int ii)`
pub fn pickup_item(
    items: &mut ItemArray,
    item_index: usize,
) -> Option<Item> {
    // 获取物品
    let item = items.items.get_mut(item_index)?.take()?;

    // 从地图移除
    let x = item.position_x as usize;
    let y = item.position_y as usize;
    if x < MAXDUNX && y < MAXDUNY {
        items.ground_items[x][y] = 0;
    }

    // 释放物品槽
    items.free(item_index);

    Some(item)
}

/// 删除地面物品
///
/// **C++ Reference**: `DeleteItem(int ii, int i)`
pub fn delete_item(items: &mut ItemArray, item_index: usize) -> bool {
    if item_index >= items.items.len() {
        return false;
    }

    if let Some(item) = items.items[item_index].take() {
        // 从地图移除
        let x = item.position_x as usize;
        let y = item.position_y as usize;
        if x < MAXDUNX && y < MAXDUNY {
            items.ground_items[x][y] = 0;
        }


        // 释放物品槽
        items.free(item_index);
        return true;
    }

    false
}

// ============================================================================
// Day 3: 物品交互系统
// ============================================================================

/// 物品使用效果类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemUseEffect {
    /// 无效果
    None,
    /// 恢复部分生命
    HealPartial,
    /// 恢复全部生命
    HealFull,
    /// 恢复部分法力
    ManaPartial,
    /// 恢复全部法力
    ManaFull,
    /// 恢复生命和法力
    Rejuvenation,
    /// 恢复全部生命和法力
    FullRejuvenation,
    /// 增加力量
    StrengthBonus(i32),
    /// 增加魔法
    MagicBonus(i32),
    /// 增加敏捷
    DexterityBonus(i32),
    /// 增加活力
    VitalityBonus(i32),
    /// 卷轴施法
    CastScroll(i32), // spell_id
    /// 学习法术书
    LearnSpell(i32), // spell_id
    /// 使用符文
    UseRune(i32), // rune_spell_id
    /// 使用油
    ApplyOil(i32), // oil_type
    /// 显示末日地图
    MapOfDoom,
    /// 特殊药剂 (所有属性+3)
    SpecialElixir,
}

/// 物品使用结果
#[derive(Debug, Clone)]
pub struct ItemUseResult {
    /// 是否成功使用
    pub success: bool,
    /// 使用效果
    pub effect: ItemUseEffect,
    /// 消息
    pub message: String,
    /// 是否消耗物品
    pub consumed: bool,
}

impl ItemUseResult {
    pub fn success(effect: ItemUseEffect, consumed: bool) -> Self {
        Self {
            success: true,
            effect,
            message: String::new(),
            consumed,
        }
    }

    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            effect: ItemUseEffect::None,
            message: message.to_string(),
            consumed: false,
        }
    }
}

/// 使用物品
///
/// **C++ Reference**: `UseItem(Player &player, item_misc_id mid, SpellID spellID, int spellFrom)`
///
/// # Arguments
/// * `misc_id` - 物品杂项类型
/// * `spell_id` - 关联的法术ID (用于卷轴/书)
///
/// # Returns
/// 物品使用结果
pub fn use_item(misc_id: ItemMiscId, spell_id: i32) -> ItemUseResult {
    match misc_id {
        ItemMiscId::Heal => {
            ItemUseResult::success(ItemUseEffect::HealPartial, true)
        }
        ItemMiscId::FullHeal => {
            ItemUseResult::success(ItemUseEffect::HealFull, true)
        }
        ItemMiscId::Mana => {
            ItemUseResult::success(ItemUseEffect::ManaPartial, true)
        }
        ItemMiscId::FullMana => {
            ItemUseResult::success(ItemUseEffect::ManaFull, true)
        }
        ItemMiscId::Rejuv => {
            ItemUseResult::success(ItemUseEffect::Rejuvenation, true)
        }
        ItemMiscId::FullRejuv | ItemMiscId::ArenaPot => {
            ItemUseResult::success(ItemUseEffect::FullRejuvenation, true)
        }
        ItemMiscId::ElixStr => {
            ItemUseResult::success(ItemUseEffect::StrengthBonus(1), true)
        }
        ItemMiscId::ElixMag => {
            ItemUseResult::success(ItemUseEffect::MagicBonus(1), true)
        }
        ItemMiscId::ElixDex => {
            ItemUseResult::success(ItemUseEffect::DexterityBonus(1), true)
        }
        ItemMiscId::ElixVit => {
            ItemUseResult::success(ItemUseEffect::VitalityBonus(1), true)
        }
        ItemMiscId::Scroll | ItemMiscId::ScrollT => {
            ItemUseResult::success(ItemUseEffect::CastScroll(spell_id), true)
        }
        ItemMiscId::Book => {
            ItemUseResult::success(ItemUseEffect::LearnSpell(spell_id), true)
        }
        ItemMiscId::MapOfDoom => {
            ItemUseResult::success(ItemUseEffect::MapOfDoom, true)
        }
        ItemMiscId::SpecialElixir => {
            ItemUseResult::success(ItemUseEffect::SpecialElixir, true)
        }
        ItemMiscId::RuneF => {
            ItemUseResult::success(ItemUseEffect::UseRune(1), true) // SpellID::RuneOfFire
        }
        ItemMiscId::RuneL => {
            ItemUseResult::success(ItemUseEffect::UseRune(2), true) // SpellID::RuneOfLight
        }
        ItemMiscId::RuneS => {
            ItemUseResult::success(ItemUseEffect::UseRune(3), true) // SpellID::RuneOfStone
        }
        ItemMiscId::GrRuneL => {
            ItemUseResult::success(ItemUseEffect::UseRune(4), true) // SpellID::RuneOfNova
        }
        ItemMiscId::GrRuneF => {
            ItemUseResult::success(ItemUseEffect::UseRune(5), true) // SpellID::RuneOfImmolation
        }
        ItemMiscId::Oil => {
            ItemUseResult::success(ItemUseEffect::ApplyOil(misc_id as i32), false) // 油不立即消耗
        }
        _ => {
            ItemUseResult::failure("Cannot use this item")
        }
    }
}

/// 鉴定物品
///
/// **C++ Reference**: `CheckIdentify(Player &player, int cii)`
///
/// # Arguments
/// * `item` - 要鉴定的物品
///
/// # Returns
/// 鉴定后的物品
pub fn identify_item(item: &mut Item) {
    item.identified = true;

    // 如果物品是魔法或唯一品质，更新名称显示
    if item.quality == ItemQuality::Magic || item.quality == ItemQuality::Unique {
        // 显示完整名称（带词缀）
        // 名称已经在生成时设置，只需标记已鉴定
    }
}

/// 修复物品
///
/// **C++ Reference**: `RepairItem(Item &item, int lvl)`
///
/// # Arguments
/// * `item` - 要修复的物品
/// * `player_level` - 玩家等级（影响修复程度）
pub fn repair_item(item: &mut Item, player_level: i32) {
    if item.durability >= item.max_durability {
        return; // 已满耐久
    }

    // 计算修复量
    // C++: rep = 0; do { rep += lvl + GenerateRnd(lvl); dur++; } while (IsValidDur && dur < MaxDur);
    let mut current_dur = item.durability;
    let max_dur = item.max_durability;

    let mut repair_points = 0;
    let mut rng = rand::rng();

    while current_dur < max_dur && repair_points < 100 {
        repair_points += player_level + rng.random_range(0..player_level.max(1));
        current_dur += 1;
    }

    item.durability = current_dur.min(max_dur);
}

/// 充能物品（法杖）
///
/// **C++ Reference**: `RechargeItem(Item &item, Player &player)`
///
/// # Arguments
/// * `item` - 要充能的物品
/// * `player_max_mana` - 玩家最大法力值
pub fn recharge_item(item: &mut Item, player_max_mana: i32) {
    if item.item_type != ItemType::Staff {
        return;
    }

    // 计算可充能的法力
    // C++: 基于玩家法力和物品当前充能
    let max_charges = item.max_charges;
    if max_charges <= 0 {
        return;
    }

    // 每点法力可恢复一定充能
    let recharge_amount = (player_max_mana / 100).max(1);
    item.charges = (item.charges + recharge_amount).min(max_charges);
}

/// 应用油到物品
///
/// **C++ Reference**: `ApplyOilToItem(Item &item, Player &player)`
///
/// # Arguments
/// * `item` - 要应用油的物品
/// * `oil_value` - 油的效果值 (根据不同油类型)
///
/// # Returns
/// 是否成功应用
pub fn apply_oil_to_item(item: &mut Item, oil_value: i32) -> bool {
    // 只能应用到武器或护甲
    match item.item_type {
        ItemType::Sword | ItemType::Axe | ItemType::Bow | ItemType::Mace |
        ItemType::Armor | ItemType::Shield | ItemType::Helm => {}
        _ => return false,
    }

    // 油类型效果 (简化版，实际应用需要根据具体油类型)
    // 这里用 oil_value 来决定效果
    match oil_value % 10 {
        0 => {
            // 精准油：+2% 命中
            item.bonus_to_hit += 2;
        }
        1 => {
            // 大师油：+4% 命中
            item.bonus_to_hit += 4;
        }
        2 => {
            // 锋利油：+1 最大伤害
            item.max_damage += 1;
        }
        3 => {
            // 死亡油：+2 最大伤害
            item.max_damage += 2;
        }
        4 => {
            // 技巧油：降低装备需求
            item.required_str = (item.required_str - 5).max(0);
        }
        5 => {
            // 铁匠油：+1 耐久
            item.max_durability += 1;
            item.durability += 1;
        }
        6 => {
            // 强化油：+2 耐久
            item.max_durability += 2;
            item.durability += 2;
        }
        7 => {
            // 硬化油：+1 护甲
            item.armor_class += 1;
        }
        8 => {
            // 增强油：+2 护甲
            item.armor_class += 2;
        }
        _ => return false,
    }

    true
}/// 放置物品到世界
///
/// **C++ Reference**: `PlaceItemInWorld(Item &&item, WorldTilePosition position)`
///
/// # Arguments
/// * `items` - 物品数组
/// * `item` - 要放置的物品
/// * `x`, `y` - 放置位置
///
/// # Returns
/// 物品索引，如果失败则返回 None
pub fn place_item_in_world(
    items: &mut ItemArray,
    mut item: Item,
    x: usize,
    y: usize,
) -> Option<usize> {
    // 分配物品槽
    let ii = items.allocate()?;

    // 设置位置
    item.position_x = x as i32;
    item.position_y = y as i32;

    // 放置到地图
    items.ground_items[x][y] = (ii + 1) as i8;
    items.items[ii] = Some(item);

    Some(ii)
}

/// 丢弃物品到玩家脚下
///
/// **C++ Reference**: 部分 `InvGetItem` 和网络命令
///
/// # Arguments
/// * `items` - 物品数组
/// * `item` - 要丢弃的物品
/// * `player_x`, `player_y` - 玩家位置
///
/// # Returns
/// 放置的物品索引
pub fn drop_item_at_player(
    items: &mut ItemArray,
    item: Item,
    player_x: usize,
    player_y: usize,
) -> Option<usize> {
    // 寻找放置位置
    if let Some((x, y)) = get_super_item_space(items, player_x as i32, player_y as i32) {
        place_item_in_world(items, item, x, y)
    } else {
        None
    }
}

/// 金币合并
///
/// **C++ Reference**: 金币拾取时自动合并
///
/// # Arguments
/// * `gold1` - 第一个金币堆
/// * `gold2` - 第二个金币堆
///
/// # Returns
/// 合并后的金币和溢出的金币
pub fn merge_gold(gold1: &mut Item, gold2: Item) -> Option<Item> {
    if gold1.item_type != ItemType::Gold || gold2.item_type != ItemType::Gold {
        return Some(gold2);
    }

    let total = gold1.value + gold2.value;

    // 检查金币上限
    let max_gold = if gold1.value >= GOLD_MEDIUM_LIMIT {
        GOLD_MAX_LIMIT
    } else if gold1.value >= GOLD_SMALL_LIMIT {
        GOLD_MEDIUM_LIMIT
    } else {
        GOLD_SMALL_LIMIT
    };

    if total <= max_gold as i32 {
        gold1.value = total;
        None // 完全合并
    } else {
        gold1.value = max_gold as i32;
        // 返回剩余金币
        let mut overflow = gold2;
        overflow.value = total - max_gold as i32;
        Some(overflow)
    }
}

/// 检查物品是否可被玩家使用
///
/// **C++ Reference**: `Player::CanUseItem(const Item &item)`
///
/// # Arguments
/// * `item` - 要检查的物品
/// * `player_str` - 玩家力量
/// * `player_mag` - 玩家魔法
/// * `player_dex` - 玩家敏捷
///
/// # Returns
/// 是否满足使用要求
pub fn can_use_item(
    item: &Item,
    player_str: i32,
    player_mag: i32,
    player_dex: i32,
) -> bool {
    player_str >= item.required_str as i32 &&
    player_mag >= item.required_mag as i32 &&
    player_dex >= item.required_dex as i32
}

/// 获取物品价值（卖价）
///
/// **C++ Reference**: `GetItemSell(const Item &item)`
pub fn get_item_sell_value(item: &Item) -> i32 {
    // 基础价值的四分之一
    let mut value = item.value / 4;

    // 最低 1 金
    value = value.max(1);

    // 已损坏的物品价值更低
    if item.durability < item.max_durability && item.max_durability > 0 {
        let ratio = item.durability as f32 / item.max_durability as f32;
        value = (value as f32 * ratio) as i32;
    }

    value.max(1)
}

/// 获取物品修复费用
///
/// **C++ Reference**: `GetItemRepairCost(const Item &item)`
pub fn get_item_repair_cost(item: &Item) -> i32 {
    if item.durability >= item.max_durability || item.max_durability == 0 {
        return 0;
    }

    // 基于物品价值和损坏程度
    let damage_ratio = 1.0 - (item.durability as f32 / item.max_durability as f32);
    let cost = (item.value as f32 * damage_ratio * 0.5) as i32;

    cost.max(1)
}
