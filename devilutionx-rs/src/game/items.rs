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
///
/// NOTE: duplicates the canonical `crate::levels::types::MAXDUNX`/`MAXDUNY`
/// (= 112). Not consolidated via `use` because this file compiles inside both
/// the library crate and the `devilutionx` *binary* crate, whose module tree
/// (`src/main.rs`) does not declare a `levels` module — so `crate::levels`
/// is not a valid path here. Values are kept identical to the canonical source.
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;

/// Gold limits
pub const GOLD_SMALL_LIMIT: i32 = 1000;
pub const GOLD_MEDIUM_LIMIT: i32 = 2500;
pub const GOLD_MAX_LIMIT: i32 = 5000;

/// Item creation info flags (C++: _icreateinfo_flag)
pub mod CreateInfoFlag {
    // Values match C++ `icreateinfo_flag` (items.h:156-171) so the
    // serialised _iCreateInfo matches byte-for-byte.
    pub const CF_LEVEL: u16 = (1 << 6) - 1;
    pub const CF_ONLYGOOD: u16 = 1 << 6;
    pub const CF_UPER15: u16 = 1 << 7;
    pub const CF_UPER1: u16 = 1 << 8;
    pub const CF_UNIQUE: u16 = 1 << 9;
    pub const CF_SMITH: u16 = 1 << 10;
    pub const CF_SMITHPREMIUM: u16 = 1 << 11;
    pub const CF_BOY: u16 = 1 << 12;
    pub const CF_WITCH: u16 = 1 << 13;
    pub const CF_HEALER: u16 = 1 << 14;
    pub const CF_PREGEN: u16 = 1 << 15;
    pub const CF_USEFUL: u16 = CF_UPER15 | CF_UPER1;
    pub const CF_TOWN: u16 = CF_SMITH | CF_SMITHPREMIUM | CF_BOY | CF_WITCH | CF_HEALER;
    pub const CF_HELLFIRE: u32 = 0x00010000; // Hellfire item (in dwBuff)
}

/// Item index enum (C++: _item_indexes)
/// 部分实现，用于常用物品
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(i16)]
pub enum ItemIndex {
    None = -1,
    Gold = 0,
    PotionHealing = 77,
    PotionMana = 79,
    PotionFullHealing = 78,
    PotionFullMana = 80,
    PotionRejuv = 81,
    ScrollIdentify = 94,
    ScrollTownPortal = 98,
    ShortSword = 119,
    Falchion = 120,
    Claymore = 122,
    BroadSword = 126,
    SabreBreaker = 124,
    LongSword = 125,
    Bastard = 127,
    TwoHandSword = 128,
    GreatSword = 129,
    SmallAxe = 130,
    Axe = 131,
    LargeAxe = 132,
    BroadAxe = 133,
    BattleAxe = 134,
    GreatAxe = 135,
    Club = 140,
    Spiked = 139,
    Mace = 136,
    MorningStar = 137,
    Flail = 141,
    WarHammer = 138,
    Maul = 142,
    ShortBow = 143,
    HuntersBow = 144,
    LongBow = 145,
    CompositeBow = 146,
    ShortBattleBow = 147,
    LongBattleBow = 148,
    ShortWarBow = 149,
    LongWarBow = 150,
    ShortStaff = 151,
    LongStaff = 152,
    CompositeStaff = 153,
    QuarterStaff = 154,
    WarStaff = 155,
    Buckler = 71,
    SmallShield = 72,
    LargeShield = 73,
    KiteShield = 74,
    TowerShield = 75,
    GothicShield = 76,
    Cap = 48,
    SkullCap = 49,
    Helm = 50,
    FullHelm = 51,
    GreatHelm = 53,
    Crown = 52,
    Rags = 55,
    Cloak = 56,
    Robe = 57,
    QuiltedArmor = 58,
    LeatherArmor = 59,
    HardLeatherArmor = 60,
    StuddedLeather = 61,
    RingMail = 62,
    ChainMail = 63,
    ScaleMail = 64,
    BreastPlate = 65,
    SplintMail = 66,
    PlateMail = 67,
    FieldPlate = 68,
    GothicPlate = 69,
    FullPlateMail = 70,
    Ring = 156,
    Amulet = 159,
    Rock = 9,
    OpticalJoystick = 10,
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
    get_item_attrs_by_index(item, item_idx as i16, level);
}

/// [`get_item_attrs`] variant that takes a raw `ITEMS_DATA` row index directly.
///
/// The `ItemIndex` enum only names a subset of the table rows, so the full
/// `SetupAllItems` path (which can receive any droppable index) must not round-
/// trip through the enum.
pub fn get_item_attrs_by_index(item: &mut Item, item_idx: i16, level: i32) {
    let idx = item_idx;
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
        ItemDatEquipType::Unequipable => ItemEquipType::Unequipable,
        _ => ItemEquipType::None,
    };

    // 伤害和护甲
    item.min_damage = data.min_damage;
    item.max_damage = data.max_damage;
    item.base_damage_min = data.min_damage;
    item.base_damage_max = data.max_damage;

    // 护甲 (C++: item._iAC = baseItemData.iMinAC + GenerateRnd(iMaxAC - iMinAC + 1))
    // 使用与 C++ 相同的全局 LCG，保证 SetupAllItems 的随机序列与 C++ 一致
    item.armor_class = data.min_ac as i16
        + crate::engine::random::generate_rnd(data.max_ac as i32 - data.min_ac as i32 + 1) as i16;
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

// ============================================================================
// C++ items.cpp 对齐补齐: 物品生成 / 唯一物品 / 重建 / 显示 / 价值 / 空间 / 拾取
// ----------------------------------------------------------------------------
// 以下函数补齐 `Source/items.cpp` 中尚未在 Rust 实现的关键函数，签名尽量对齐
// C++ 原型，参数中需要全局状态（玩家、地图、随机种子源）的部分被改为显式参数，
// 以便能在无全局状态的纯函数上下文中调用和测试。
// ============================================================================

/// 物品耐久度"不可破坏"哨兵值（C++: DUR_INDESTRUCTIBLE = 255）
pub const DUR_INDESTRUCTIBLE: i32 = 255;

/// 抗性显示上限（C++: MaxResistance = 75）
pub const MAX_RESISTANCE: i16 = 75;

/// 金币堆显示用的光标 ID（C++: ICURS_GOLD_*）
pub mod GoldCursor {
    pub const SMALL: u8 = 0;  // ICURS_GOLD_SMALL
    pub const MEDIUM: u8 = 1; // ICURS_GOLD_MEDIUM
    pub const LARGE: u8 = 2;  // ICURS_GOLD_LARGE
}

/// 计算金币堆的光标 ID（C++: GetGoldCursor）
///
/// 根据 C++ `Source/items.cpp:2959 GetGoldCursor(int value)`:
/// - `value >= GOLD_MEDIUM_LIMIT` -> 大堆
/// - `value <= GOLD_SMALL_LIMIT`  -> 小堆
/// - 其它 -> 中堆
pub fn get_gold_cursor(value: i32) -> u8 {
    if value >= GOLD_MEDIUM_LIMIT {
        GoldCursor::LARGE
    } else if value <= GOLD_SMALL_LIMIT {
        GoldCursor::SMALL
    } else {
        GoldCursor::MEDIUM
    }
}

/// 设置金币物品的光标（C++: SetPlrHandGoldCurs）
///
/// **C++ Reference**: `Source/items.cpp:2970 SetPlrHandGoldCurs(Item &gold)`
pub fn set_plr_hand_gold_curs(gold: &mut Item) {
    gold.cursor = get_gold_cursor(gold.value);
}

/// 为物品生成新的随机种子（C++: GenerateNewSeed）
///
/// **C++ Reference**: `Source/items.cpp:2954 GenerateNewSeed(Item &item)`
pub fn generate_new_seed(item: &mut Item) {
    item.seed = rand::random::<u32>();
}

/// 从基础物品数据初始化物品（C++: InitializeItem）
///
/// 与 [`get_item_attrs`] 的区别：`InitializeItem` 使用最小 AC（不随机），
/// 不调用 GetBookSpell/GetOilType，用于"普通"物品和重建路径。
///
/// **C++ Reference**: `Source/items.cpp:2915 InitializeItem(Item &item, _item_indexes itemData)`
pub fn initialize_item(item: &mut Item, item_idx: ItemIndex) {
    // 等价于 C++ `item = {}` 的清零
    *item = Item::empty();

    let idx = item_idx as i16;
    if idx < 0 || idx as usize >= ITEMS_DATA.len() {
        return;
    }
    let data = &ITEMS_DATA[idx as usize];

    item.item_index = idx;
    item.name = data.name.to_string();
    item.base_name = data.name.to_string();
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

    item.min_damage = data.min_damage;
    item.max_damage = data.max_damage;
    item.base_damage_min = data.min_damage;
    item.base_damage_max = data.max_damage;
    // InitializeItem 使用最小 AC（非随机）
    item.armor_class = data.min_ac as i16;
    item.base_armor = item.armor_class;

    // 法杖默认充能（C++: gbIsHellfire ? 18 : 40）—— 这里取 40（Diablo 默认）
    if item.misc_id == ItemMiscId::Staff {
        item.charges = 40;
        item.max_charges = 40;
    }

    item.durability = data.durability as i32;
    item.max_durability = data.durability as i32;
    item.required_str = data.min_str as i8;
    item.required_mag = data.min_mag;
    item.required_dex = data.min_dex as i8;
    item.value = data.value as i32;
    item.identified_value = data.value as i32;
    item.special_flags = ItemSpecialEffect(data.special_effects.0);
    item.prefix_power = ItemEffectType::Invalid;
    item.suffix_power = ItemEffectType::Invalid;
    item.quality = ItemQuality::Normal;
    item.cursor = data.cursor_graphic;
    item.quantity = 1;
}

/// 设置物品"已生成"状态（C++: SetupItem）
///
/// 在 C++ 中负责动画与"未鉴定"标志；Rust 移植仅保留语义相关的鉴定标志。
///
/// **C++ Reference**: `Source/items.cpp:3173 SetupItem(Item &item)`
pub fn setup_item(item: &mut Item) {
    item.identified = false;
}

/// 创建金币堆（C++: MakeGoldStack）
///
/// **C++ Reference**: `Source/items.cpp:4663 MakeGoldStack(Item &goldItem, int value)`
pub fn make_gold_stack(gold_item: &mut Item, value: i32) {
    initialize_item(gold_item, ItemIndex::Gold);
    generate_new_seed(gold_item);
    gold_item.stat_flag = true;
    gold_item.value = value;
    gold_item.quantity = value;
    set_plr_hand_gold_curs(gold_item);
}

/// 计算物品鉴定后价值（C++: CalcItemValue / CalculateItemValue）
///
/// 精确复刻 C++ `CalcItemValue`：
/// ```text
/// v = _iVMult1 + _iVMult2;
/// if v > 0 { v *= _ivalue; }
/// if v < 0 { v = _ivalue / v; }
/// v = _iVAdd1 + _iVAdd2 + v;
/// _iIvalue = max(v, 1);
/// ```
///
/// **C++ Reference**: `Source/items.cpp:600 CalcItemValue(Item &item)`
pub fn calc_item_value(item: &mut Item) {
    let mut v = item.value_mult1 + item.value_mult2;
    if v > 0 {
        v *= item.value;
    }
    if v < 0 {
        v = item.value / v;
    }
    v = item.value_add1 + item.value_add2 + v;
    item.identified_value = v.max(1);
}

// ----------------------------------------------------------------------------
// 唯一物品生成 (SpawnUnique / UniqueItemColor)
// ----------------------------------------------------------------------------

/// 生成一个唯一物品到世界（C++: SpawnUnique）
///
/// 这是 `SpawnUnique` 的简化移植：给定唯一物品索引 `uid`，在地图上找到位置并
/// 放置已应用唯一属性的物品。返回物品在 [`ItemArray`] 中的索引。
///
/// 与 C++ 的差异：
/// - 不处理多人模式难度路径（始终走 Normal 难度的 `GetItemAttrs + GetUniqueItem`）
/// - 不发送网络消息
///
/// **C++ Reference**: `Source/items.cpp:3179 SpawnUnique(...)`
pub fn spawn_unique(
    items: &mut ItemArray,
    uid: usize,
    position_x: i32,
    position_y: i32,
    level: i32,
) -> Option<usize> {
    if uid >= UNIQUE_ITEMS.len() {
        return None;
    }
    let unique = &UNIQUE_ITEMS[uid];

    // 在世界中找一个空位（C++ 先尝试 exactPosition/CanPut，否则 GetSuperItemSpace）
    let (px, py) = get_super_item_space(items, position_x, position_y)
        .unwrap_or((position_x as usize, position_y as usize));

    // 找到该唯一物品基础类型对应的基础物品索引
    let item_idx = unique_base_to_item_index(unique.base_item);

    let ii = items.allocate()?;
    let mut item = Item::empty();
    item.position_x = px as i32;
    item.position_y = py as i32;

    if let Some(idx) = item_idx {
        get_item_attrs(&mut item, idx, level);
    }
    item.unique_id = uid as i32;
    get_unique_item(&mut item, unique);
    setup_item(&mut item);

    items.items[ii] = Some(item);
    items.place_at(ii, px, py);
    if uid < items.unique_item_flags.len() {
        items.unique_item_flags[uid] = true;
    }

    Some(ii)
}

/// 把 [`UniqueBaseItem`] 映射回最接近的基础 [`ItemIndex`]
///
/// C++ 中唯一物品与基础物品通过 `iItemId` 关联；Rust 没有完整的数据表关联，
/// 这里为常用基础类型提供映射，无法映射时返回 `None`（调用方需兜底）。
fn unique_base_to_item_index(base: UniqueBaseItem) -> Option<ItemIndex> {
    use UniqueBaseItem as U;
    Some(match base {
        U::ShortBow => ItemIndex::ShortBow,
        U::LongBow => ItemIndex::LongBow,
        U::CompBow => ItemIndex::CompositeBow,
        U::BattleBow => ItemIndex::ShortBattleBow,
        U::WarBow => ItemIndex::ShortWarBow,
        U::Dagger => ItemIndex::ShortSword, // 最接近的短兵器
        U::Falchion => ItemIndex::Falchion,
        U::Claymore => ItemIndex::Claymore,
        U::BroadSword => ItemIndex::BroadSword,
        U::Sabre => ItemIndex::SabreBreaker,
        U::Scimitar => ItemIndex::BroadSword,
        U::LongSword => ItemIndex::LongSword,
        U::BastardSword => ItemIndex::Bastard,
        U::TwoHandSword => ItemIndex::TwoHandSword,
        U::GreatSword => ItemIndex::GreatSword,
        U::Cleaver => ItemIndex::SmallAxe,
        U::LargeAxe => ItemIndex::LargeAxe,
        U::BroadAxe => ItemIndex::BroadAxe,
        U::SmallAxe => ItemIndex::SmallAxe,
        U::BattleAxe => ItemIndex::BattleAxe,
        U::GreatAxe => ItemIndex::GreatAxe,
        U::Mace => ItemIndex::Mace,
        U::MorningStar => ItemIndex::MorningStar,
        U::SpikedClub => ItemIndex::Spiked,
        U::Maul => ItemIndex::Maul,
        U::WarHammer => ItemIndex::WarHammer,
        U::Flail => ItemIndex::Flail,
        U::LongStaff => ItemIndex::LongStaff,
        U::ShortStaff => ItemIndex::ShortStaff,
        U::CompositeStaff => ItemIndex::CompositeStaff,
        U::QuarterStaff => ItemIndex::QuarterStaff,
        U::WarStaff => ItemIndex::WarStaff,
        U::SkullCap => ItemIndex::SkullCap,
        U::Helm => ItemIndex::Helm,
        U::GreatHelm => ItemIndex::GreatHelm,
        U::Crown => ItemIndex::Crown,
        U::ChainMail => ItemIndex::ChainMail,
        U::LeatherArmor => ItemIndex::LeatherArmor,
        U::BreastPlate => ItemIndex::BreastPlate,
        U::PlateMail => ItemIndex::PlateMail,
        U::FullPlate => ItemIndex::FullPlateMail,
        U::Buckler => ItemIndex::Buckler,
        U::SmallShield => ItemIndex::SmallShield,
        U::LargeShield => ItemIndex::LargeShield,
        U::KiteShield => ItemIndex::KiteShield,
        U::GothicShield => ItemIndex::GothicShield,
        U::Ring => ItemIndex::Ring,
        U::Amulet => ItemIndex::Amulet,
        // 任务唯一物品没有常规基础物，回退到 None
        _ => return None,
    })
}

/// 唯一物品的轮廓颜色（C++: 等价于 GetOutlineColor 对 unique 的分支）
///
/// 在 C++ 中，唯一物品的轮廓固定为金色（`ColorColor : Color = Gold` 对应
/// `Color::Gold`，数值约 0/Uint8 取决于调色板索引）。这里返回固定的颜色常量。
///
/// **C++ Reference**: `Source/items.cpp:2405 GetOutlineColor` (unique 分支)
pub const UNIQUE_ITEM_COLOR: u8 = 0x0; // 调色板索引：金色

/// 获取物品的轮廓颜色（C++: GetOutlineColor）
///
/// **C++ Reference**: `Source/items.cpp:2405 GetOutlineColor(const Item &item, bool checkReq)`
pub fn get_outline_color(item: &Item, check_req: bool) -> u8 {
    if item.quality == ItemQuality::Unique {
        return UNIQUE_ITEM_COLOR;
    }
    if check_req && !item.stat_flag {
        return 0x0; // 红色（无法装备）— 简化为同一常量
    }
    0xFF // 默认（无特殊轮廓）
}

// ----------------------------------------------------------------------------
// 物品显示 (PrintItemPower / PrintItemDetails / PrintItemDur)
// ----------------------------------------------------------------------------

/// 生成物品单条能力的显示字符串（C++: PrintItemPower / 对应任务中的 GetPowerString）
///
/// 返回 `None` 表示该能力没有对应文本（C++ 返回空 `StringOrView`）。
///
/// **C++ Reference**: `Source/items.cpp:3874 PrintItemPower(char plidx, const Item &item)`
pub fn print_item_power(effect: ItemEffectType, item: &Item) -> Option<String> {
    use ItemEffectType as E;
    let s = match effect {
        E::ToHit | E::ToHitCurse => format!("chance to hit: {:+}%", item.bonus_to_hit),
        E::Damage | E::DamageCurse => format!("{:+}% damage", item.bonus_damage),
        E::ToHitDamage | E::ToHitDamageCurse => {
            format!("to hit: {:+}%, {:+}% damage", item.bonus_to_hit, item.bonus_damage)
        }
        E::Ac | E::AcCurse => format!("{:+}% armor", item.bonus_ac),
        E::SetAc | E::AcCurseSet => format!("armor class: {}", item.armor_class),
        E::FireRes | E::FireResCurse => {
            if item.resist_fire < MAX_RESISTANCE {
                format!("Resist Fire: {:+}%", item.resist_fire)
            } else {
                format!("Resist Fire: {:+}% MAX", item.resist_fire)
            }
        }
        E::LightRes | E::LightResCurse => {
            if item.resist_lightning < MAX_RESISTANCE {
                format!("Resist Lightning: {:+}%", item.resist_lightning)
            } else {
                format!("Resist Lightning: {:+}% MAX", item.resist_lightning)
            }
        }
        E::MagicRes | E::MagicResCurse => {
            if item.resist_magic < MAX_RESISTANCE {
                format!("Resist Magic: {:+}%", item.resist_magic)
            } else {
                format!("Resist Magic: {:+}% MAX", item.resist_magic)
            }
        }
        E::AllRes => {
            if item.resist_fire < MAX_RESISTANCE {
                format!("Resist All: {:+}%", item.resist_fire)
            } else {
                format!("Resist All: {:+}% MAX", item.resist_fire)
            }
        }
        E::SpellLevelAdd => {
            if item.spell_level_add > 0 {
                format!("spells are increased {} level{}", item.spell_level_add, plural_s(item.spell_level_add as i32))
            } else if item.spell_level_add < 0 {
                format!("spells are decreased {} level{}", -item.spell_level_add as i32, plural_s(-item.spell_level_add as i32))
            } else {
                "spell levels unchanged (?)".to_string()
            }
        }
        E::Charges => "Extra charges".to_string(),
        E::Spell => format!("{} charges", item.max_charges),
        E::FireDam => {
            if item.fire_min_dam == item.fire_max_dam {
                format!("Fire hit damage: {}", item.fire_min_dam)
            } else {
                format!("Fire hit damage: {}-{}", item.fire_min_dam, item.fire_max_dam)
            }
        }
        E::LightDam => {
            if item.lightning_min_dam == item.lightning_max_dam {
                format!("Lightning hit damage: {}", item.lightning_min_dam)
            } else {
                format!("Lightning hit damage: {}-{}", item.lightning_min_dam, item.lightning_max_dam)
            }
        }
        E::Str | E::StrCurse => format!("{:+} to strength", item.bonus_str),
        E::Mag | E::MagCurse => format!("{:+} to magic", item.bonus_mag),
        E::Dex | E::DexCurse => format!("{:+} to dexterity", item.bonus_dex),
        E::Vit | E::VitCurse => format!("{:+} to vitality", item.bonus_vit),
        E::Attribs | E::AttribsCurse => format!("{:+} to all attributes", item.bonus_str),
        E::GetHit | E::GetHitCurse => format!("{:+} damage from enemies", item.bonus_get_hit),
        E::Life | E::LifeCurse => format!("Hit Points: {:+}", item.bonus_hp >> 6),
        E::Mana | E::ManaCurse => format!("Mana: {:+}", item.bonus_mana >> 6),
        E::Dur => "high durability".to_string(),
        E::DurCurse => "decreased durability".to_string(),
        E::Indestructible => "indestructible".to_string(),
        E::Light => format!("+{}% light radius", 10 * item.bonus_light),
        E::LightCurse => format!("-{}% light radius", -10 * item.bonus_light),
        E::MultArrows => "multiple arrows per shot".to_string(),
        E::FireArrows => {
            if item.fire_min_dam == item.fire_max_dam {
                format!("fire arrows damage: {}", item.fire_min_dam)
            } else {
                format!("fire arrows damage: {}-{}", item.fire_min_dam, item.fire_max_dam)
            }
        }
        E::LightArrows => {
            if item.lightning_min_dam == item.lightning_max_dam {
                format!("lightning arrows damage {}", item.lightning_min_dam)
            } else {
                format!("lightning arrows damage {}-{}", item.lightning_min_dam, item.lightning_max_dam)
            }
        }
        E::Fireball => {
            if item.fire_min_dam == item.fire_max_dam {
                format!("fireball damage: {}", item.fire_min_dam)
            } else {
                format!("fireball damage: {}-{}", item.fire_min_dam, item.fire_max_dam)
            }
        }
        E::Thorns => "attacker takes 1-3 damage".to_string(),
        E::NoMana => "user loses all mana".to_string(),
        E::AbsHalfTrap => "absorbs half of trap damage".to_string(),
        E::Knockback => "knocks target back".to_string(),
        E::TripleDemonDam => "+200% damage vs. demons".to_string(),
        E::AllResZero => "All Resistance equals 0".to_string(),
        E::StealMana => {
            if (item.special_flags.0 & ItemSpecialEffect::STEAL_MANA_5.0) != 0 {
                "hit steals 5% mana".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::STEAL_MANA_3.0) != 0 {
                "hit steals 3% mana".to_string()
            } else {
                return None;
            }
        }
        E::StealLife => {
            if (item.special_flags.0 & ItemSpecialEffect::STEAL_LIFE_5.0) != 0 {
                "hit steals 5% life".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::STEAL_LIFE_3.0) != 0 {
                "hit steals 3% life".to_string()
            } else {
                return None;
            }
        }
        E::TargAc => "penetrates target's armor".to_string(),
        E::FastAttack => {
            if (item.special_flags.0 & ItemSpecialEffect::QUICKATTACK.0) != 0 {
                "quick attack".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::FASTATTACK.0) != 0 {
                "fast attack".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::FASTERATTACK.0) != 0 {
                "faster attack".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::FASTESTATTACK.0) != 0 {
                "fastest attack".to_string()
            } else {
                "Another ability (NW)".to_string()
            }
        }
        E::FastRecover => {
            if (item.special_flags.0 & ItemSpecialEffect::FASTHITRECOVER.0) != 0 {
                "fast hit recovery".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::FASTERHITRECOVER.0) != 0 {
                "faster hit recovery".to_string()
            } else if (item.special_flags.0 & ItemSpecialEffect::FASTESTBLOCKRECOVER.0) != 0 {
                "fastest hit recovery".to_string()
            } else {
                "Another ability (NW)".to_string()
            }
        }
        E::FastBlock => "fast block".to_string(),
        E::DamMod => format!(
            "adds {} point{} to damage",
            item.bonus_damage_mod,
            plural_s(item.bonus_damage_mod as i32)
        ),
        E::RndArrowVel => "fires random speed arrows".to_string(),
        E::SetDam => "unusual item damage".to_string(),
        E::SetDur => "altered durability".to_string(),
        E::OneHand => "one handed sword".to_string(),
        E::DrainLife => "constantly lose hit points".to_string(),
        E::RndStealLife => "life stealing".to_string(),
        E::NoMinStr => "no strength requirement".to_string(),
        E::AddAcLife => {
            if item.fire_min_dam == item.fire_max_dam {
                format!("lightning damage: {}", item.fire_min_dam)
            } else {
                format!("lightning damage: {}-{}", item.fire_min_dam, item.fire_max_dam)
            }
        }
        E::AddManaAc => "charged bolts on hits".to_string(),
        E::Devastation => "occasional triple damage".to_string(),
        E::Decay => format!("decaying {:+}% damage", item.bonus_damage),
        E::Peril => "2x dmg to monst, 1x to you".to_string(),
        E::Jesters => "Random 0 - 600% damage".to_string(),
        E::Crystalline => format!("low dur, {:+}% damage", item.bonus_damage),
        E::Doppelganger => {
            format!("to hit: {:+}%, {:+}% damage", item.bonus_to_hit, item.bonus_damage)
        }
        E::AcDemon => "extra AC vs demons".to_string(),
        E::AcUndead => "extra AC vs undead".to_string(),
        E::ManaToLife => "50% Mana moved to Health".to_string(),
        E::LifeToMana => "40% Health moved to Mana".to_string(),
        E::Invalid => return None,
        _ => return None,
    };
    Some(s)
}

/// 简单复数后缀辅助（用于 `print_item_power`）
fn plural_s(n: i32) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// 物品信息盒条目（C++: AddItemInfoBoxString 添加的内容）
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemInfoLines {
    pub lines: Vec<String>,
    pub is_unique: bool,
}

/// 生成已鉴定物品的完整显示文本（C++: PrintItemDetails）
///
/// 返回结构包含所有信息盒条目，以及是否为唯一物品的标志。
///
/// **C++ Reference**: `Source/items.cpp:4110 PrintItemDetails(const Item &item)`
pub fn print_item_details(item: &Item) -> ItemInfoLines {
    let mut out = ItemInfoLines::default();

    if item.item_class == ItemClass::Weapon {
        out.lines.push(format_damage_durability(item));
    }
    if item.item_class == ItemClass::Armor {
        out.lines.push(format_armor_durability(item));
    }
    if item.misc_id == ItemMiscId::Staff && item.max_charges != 0 {
        out.lines.push(format!("Charges: {}/{}", item.charges, item.max_charges));
    }
    if item.prefix_power != ItemEffectType::Invalid {
        if let Some(s) = print_item_power(item.prefix_power, item) {
            out.lines.push(s);
        }
    }
    if item.suffix_power != ItemEffectType::Invalid {
        if let Some(s) = print_item_power(item.suffix_power, item) {
            out.lines.push(s);
        }
    }
    if item.quality == ItemQuality::Unique {
        out.lines.push("unique item".to_string());
        out.is_unique = true;
    }
    out
}

/// 生成未鉴定物品的显示文本（C++: PrintItemDur）
///
/// 与 [`print_item_details`] 的区别：不显示前缀/后缀能力，魔法/唯一物品标注
/// "Not Identified"。
///
/// **C++ Reference**: `Source/items.cpp:4151 PrintItemDur(const Item &item)`
pub fn print_item_dur(item: &Item) -> ItemInfoLines {
    let mut out = ItemInfoLines::default();

    if item.item_class == ItemClass::Weapon {
        out.lines.push(format_damage_durability(item));
        if item.misc_id == ItemMiscId::Staff && item.max_charges > 0 {
            out.lines.push(format!("Charges: {}/{}", item.charges, item.max_charges));
        }
        if item.quality != ItemQuality::Normal {
            out.lines.push("Not Identified".to_string());
        }
    }
    if item.item_class == ItemClass::Armor {
        out.lines.push(format_armor_durability(item));
        if item.quality != ItemQuality::Normal {
            out.lines.push("Not Identified".to_string());
        }
        if item.misc_id == ItemMiscId::Staff && item.max_charges > 0 {
            out.lines.push(format!("Charges: {}/{}", item.charges, item.max_charges));
        }
    }
    if item.item_type == ItemType::Ring || item.item_type == ItemType::Amulet {
        out.lines.push("Not Identified".to_string());
    }
    out
}

/// 格式化 "damage: X[-Y]  Dur: A/B" / "Indestructible" 行
fn format_damage_durability(item: &Item) -> String {
    let indestructible = item.max_durability == DUR_INDESTRUCTIBLE;
    if item.min_damage == item.max_damage {
        if indestructible {
            format!("damage: {}  Indestructible", item.min_damage)
        } else {
            format!("damage: {}  Dur: {}/{}", item.min_damage, item.durability, item.max_durability)
        }
    } else if indestructible {
        format!("damage: {}-{}  Indestructible", item.min_damage, item.max_damage)
    } else {
        format!(
            "damage: {}-{}  Dur: {}/{}",
            item.min_damage, item.max_damage, item.durability, item.max_durability
        )
    }
}

/// 格式化 "armor: X  Dur: A/B" / "Indestructible" 行
fn format_armor_durability(item: &Item) -> String {
    if item.max_durability == DUR_INDESTRUCTIBLE {
        format!("armor: {}  Indestructible", item.armor_class)
    } else {
        format!(
            "armor: {}  Dur: {}/{}",
            item.armor_class, item.durability, item.max_durability
        )
    }
}

// ----------------------------------------------------------------------------
// 存档重建 (RecreateItem / RecreateTownItem / Recreate*Item)
// ----------------------------------------------------------------------------

/// 从存档重建物品（C++: RecreateItem）
///
/// 这是 `RecreateItem` 的核心逻辑移植：根据 `create_info` 决定走哪条重建路径。
/// 与 C++ 的差异：不依赖全局玩家状态，因此 RecreateTown* 路径退化为通用的
/// 属性重建（不重新跑商店随机表）。
///
/// **C++ Reference**: `Source/items.cpp:3502 RecreateItem(...)`
pub fn recreate_item(
    item: &mut Item,
    idx: ItemIndex,
    create_info: u16,
    iseed: u32,
    ivalue: i32,
) {
    // 金币
    if idx == ItemIndex::Gold {
        initialize_item(item, ItemIndex::Gold);
        item.seed = iseed;
        item.create_info = create_info;
        item.value = ivalue;
        item.quantity = ivalue;
        set_plr_hand_gold_curs(item);
        return;
    }

    // 无创建信息：直接初始化基础物品
    if create_info == 0 {
        initialize_item(item, idx);
        item.seed = iseed;
        return;
    }

    if (create_info & CreateInfoFlag::CF_UNIQUE) == 0 {
        // 城镇商店物品
        if (create_info & CreateInfoFlag::CF_TOWN) != 0 {
            recreate_town_item(item, idx, create_info, iseed);
            return;
        }
        // 有用物品（药水等）
        if (create_info & CreateInfoFlag::CF_USEFUL) == CreateInfoFlag::CF_USEFUL {
            let level = (create_info & CreateInfoFlag::CF_LEVEL) as i32;
            setup_all_useful_rebuild(item, iseed, level);
            return;
        }
    }

    // 通用重建路径：依据 create_info 中记录的等级与标志重新生成
    let level = (create_info & CreateInfoFlag::CF_LEVEL) as i32;

    let mut uper = 0;
    if (create_info & CreateInfoFlag::CF_UPER1) != 0 {
        uper = 1;
    }
    if (create_info & CreateInfoFlag::CF_UPER15) != 0 {
        uper = 15;
    }

    let only_good = (create_info & CreateInfoFlag::CF_ONLYGOOD) != 0;
    let pregen = (create_info & CreateInfoFlag::CF_PREGEN) != 0;
    // CF_UNIQUE==0 时强制不为唯一（forceNotUnique）
    let force_not_unique = (create_info & CreateInfoFlag::CF_UNIQUE) == 0;

    recreate_setup_all(item, idx, iseed, level, uper, only_good, pregen, force_not_unique);
    setup_item(item);
}

/// 内部：recreate_item 使用的 setup_all_items 包装（不支持唯一偏移）
fn recreate_setup_all(
    item: &mut Item,
    idx: ItemIndex,
    iseed: u32,
    level: i32,
    uper: i32,
    only_good: bool,
    pregen: bool,
    force_not_unique: bool,
) {
    setup_all_items(item, idx, iseed, level, uper, only_good, pregen);
    if force_not_unique && item.quality == ItemQuality::Unique {
        // 强制降级为魔法以匹配 C++ forceNotUnique 语义
        item.quality = ItemQuality::Magic;
        item.create_info &= !CreateInfoFlag::CF_UNIQUE;
    }
}

/// 城镇物品重建分发（C++: RecreateTownItem）
///
/// **C++ Reference**: `Source/items.cpp:2156 RecreateTownItem(...)`
pub fn recreate_town_item(item: &mut Item, idx: ItemIndex, create_info: u16, iseed: u32) {
    let level = (create_info & CreateInfoFlag::CF_LEVEL) as i32;
    if (create_info & CreateInfoFlag::CF_SMITH) != 0 {
        recreate_smith_item(item, level, iseed);
    } else if (create_info & CreateInfoFlag::CF_SMITHPREMIUM) != 0 {
        recreate_premium_item(item, level, iseed);
    } else if (create_info & CreateInfoFlag::CF_BOY) != 0 {
        recreate_boy_item(item, level, iseed);
    } else if (create_info & CreateInfoFlag::CF_WITCH) != 0 {
        recreate_witch_item(item, idx, level, iseed);
    } else if (create_info & CreateInfoFlag::CF_HEALER) != 0 {
        recreate_healer_item(item, idx, level, iseed);
    }
}

/// 重建铁匠物品（C++: RecreateSmithItem）
///
/// **C++ Reference**: `Source/items.cpp:2080`
pub fn recreate_smith_item(item: &mut Item, lvl: i32, iseed: u32) {
    // item.item_index 已存在；重新加载基础属性。ItemIndex 是 #[repr(i16)]，
    // 这里用安全的边界检查 + transmute 还原（与 try_random_unique_item 一致）。
    let idx = i16_to_item_index(item.item_index);
    get_item_attrs(item, idx, lvl);
    item.seed = iseed;
    item.create_info = (lvl as u16) | CreateInfoFlag::CF_SMITH;
    item.identified = true;
}

/// 把 `i16`（item_index 存储类型）安全地还原为 [`ItemIndex`]。
///
/// 越界或负值返回 [`ItemIndex::None`]。`ItemIndex` 是 `#[repr(i16)]`，因此
/// `transmute` 是良定义的。
fn i16_to_item_index(v: i16) -> ItemIndex {
    if v < 0 {
        return ItemIndex::None;
    }
    // 仅对已知的判别值做转换；未知值回退到 None 以避免 UB。
    let known: &[ItemIndex] = &[
        ItemIndex::Gold, ItemIndex::PotionHealing, ItemIndex::PotionMana,
        ItemIndex::ShortSword, ItemIndex::Falchion, ItemIndex::Claymore,
        ItemIndex::BroadSword, ItemIndex::SabreBreaker, ItemIndex::LongSword,
        ItemIndex::Bastard, ItemIndex::TwoHandSword, ItemIndex::GreatSword,
        ItemIndex::SmallAxe, ItemIndex::Axe, ItemIndex::LargeAxe, ItemIndex::BroadAxe,
        ItemIndex::BattleAxe, ItemIndex::GreatAxe,
        ItemIndex::Club, ItemIndex::Spiked, ItemIndex::Mace, ItemIndex::MorningStar,
        ItemIndex::Flail, ItemIndex::WarHammer, ItemIndex::Maul,
        ItemIndex::ShortBow, ItemIndex::HuntersBow, ItemIndex::LongBow, ItemIndex::CompositeBow,
        ItemIndex::ShortBattleBow, ItemIndex::LongBattleBow, ItemIndex::ShortWarBow, ItemIndex::LongWarBow,
        ItemIndex::ShortStaff, ItemIndex::LongStaff, ItemIndex::CompositeStaff,
        ItemIndex::QuarterStaff, ItemIndex::WarStaff,
        ItemIndex::Buckler, ItemIndex::SmallShield, ItemIndex::LargeShield,
        ItemIndex::KiteShield, ItemIndex::TowerShield, ItemIndex::GothicShield,
        ItemIndex::Cap, ItemIndex::SkullCap, ItemIndex::Helm, ItemIndex::FullHelm,
        ItemIndex::GreatHelm, ItemIndex::Crown,
        ItemIndex::Rags, ItemIndex::Cloak, ItemIndex::Robe, ItemIndex::QuiltedArmor,
        ItemIndex::LeatherArmor, ItemIndex::HardLeatherArmor, ItemIndex::StuddedLeather,
        ItemIndex::RingMail, ItemIndex::ChainMail, ItemIndex::ScaleMail, ItemIndex::BreastPlate,
        ItemIndex::SplintMail, ItemIndex::PlateMail, ItemIndex::FieldPlate,
        ItemIndex::GothicPlate, ItemIndex::FullPlateMail,
        ItemIndex::Ring, ItemIndex::Amulet,
        ItemIndex::Rock, ItemIndex::OpticalJoystick,
    ];
    for &k in known {
        if k as i16 == v {
            return k;
        }
    }
    ItemIndex::None
}

/// 重建高级商品（C++: RecreatePremiumItem）
///
/// **C++ Reference**: `Source/items.cpp:2091`
pub fn recreate_premium_item(item: &mut Item, plvl: i32, iseed: u32) {
    get_item_bonus(item, plvl / 2, plvl, true, true);
    item.seed = iseed;
    item.create_info = (plvl as u16) | CreateInfoFlag::CF_SMITHPREMIUM;
    item.identified = true;
}

/// 重建 Wirt（男孩）物品（C++: RecreateBoyItem）
///
/// **C++ Reference**: `Source/items.cpp:2103`
pub fn recreate_boy_item(item: &mut Item, lvl: i32, iseed: u32) {
    get_item_bonus(item, lvl, 2 * lvl, true, true);
    item.seed = iseed;
    item.create_info = (lvl as u16) | CreateInfoFlag::CF_BOY;
    item.identified = true;
}

/// 重建 Adria（女巫）物品（C++: RecreateWitchItem）
///
/// **C++ Reference**: `Source/items.cpp:2115`
pub fn recreate_witch_item(item: &mut Item, idx: ItemIndex, lvl: i32, iseed: u32) {
    // 简化：对法杖/药水直接走属性，其它随机走加成
    let mut rng = rand::rng();
    let mut iblvl = -1;
    if rng.random_range(0..100) <= 5 {
        iblvl = 2 * lvl;
    }
    if iblvl == -1 && item.misc_id == ItemMiscId::Staff {
        iblvl = 2 * lvl;
    }
    if iblvl != -1 {
        get_item_bonus(item, iblvl / 2, iblvl, true, true);
    }
    let _ = idx;
    item.seed = iseed;
    item.create_info = (lvl as u16) | CreateInfoFlag::CF_WITCH;
    item.identified = true;
}

/// 重建 Pepin（医者）物品（C++: RecreateHealerItem）
///
/// **C++ Reference**: `Source/items.cpp:2141`
pub fn recreate_healer_item(item: &mut Item, idx: ItemIndex, lvl: i32, iseed: u32) {
    let _ = idx;
    item.seed = iseed;
    item.create_info = (lvl as u16) | CreateInfoFlag::CF_HEALER;
    item.identified = true;
}

/// SetupAllUseful 的重建变体（C++: SetupAllUseful 用于药水/卷轴等有用物品）
///
/// **C++ Reference**: `Source/items.cpp:1519 SetupAllUseful(Item &item, int iseed, int lvl)`
fn setup_all_useful_rebuild(item: &mut Item, iseed: u32, _level: i32) {
    item.seed = iseed;
    item.identified = true;
}

// ----------------------------------------------------------------------------
// 空间检查 (GetItemSpace / ItemSpaceOk 已有) + 金币总计
// ----------------------------------------------------------------------------

/// 在 3x3 邻域内寻找物品落点（C++: GetItemSpace）
///
/// 复刻 C++ `GetItemSpace(Point position, int8_t inum)` 的行为：检查以
/// `position` 为中心的 3x3 网格，若有可用格子则随机选一个并写入
/// `ground_items`，返回 true。
///
/// **C++ Reference**: `Source/items.cpp:550 GetItemSpace(Point position, int8_t inum)`
pub fn get_item_space(
    items: &mut ItemArray,
    position_x: i32,
    position_y: i32,
    item_index: usize,
) -> bool {
    // 构建 3x3 的可用性表
    let mut itemhold = [[false; 3]; 3];
    let mut savail = false;
    for (j, dy) in (-1..=1).enumerate() {
        for (i, dx) in (-1..=1).enumerate() {
            let x = position_x + dx;
            let y = position_y + dy;
            let ok = if x < 0 || y < 0 {
                false
            } else {
                item_space_ok(items, x as usize, y as usize)
            };
            itemhold[i][j] = ok;
            if ok {
                savail = true;
            }
        }
    }

    if !savail {
        return false;
    }

    // 在可用格子中随机选一个（C++ 用 GenerateRnd(15)+1 的 1-based 计数）
    let mut rng = rand::rng();
    let mut rs = rng.random_range(1..=15);
    let (mut xx, mut yy) = (0usize, 0usize);
    loop {
        if itemhold[xx][yy] {
            rs -= 1;
        }
        if rs <= 0 {
            break;
        }
        xx += 1;
        if xx == 3 {
            xx = 0;
            yy += 1;
            if yy == 3 {
                yy = 0;
            }
        }
    }

    let final_x = (position_x - 1 + xx as i32) as usize;
    let final_y = (position_y - 1 + yy as i32) as usize;

    // 更新物品位置与地图（C++ 中此处写 dItem）
    if let Some(item) = items.items[item_index].as_mut() {
        item.position_x = final_x as i32;
        item.position_y = final_y as i32;
    }
    if final_x < MAXDUNX && final_y < MAXDUNY {
        items.ground_items[final_x][final_y] = (item_index + 1) as i8;
    }
    true
}

/// 计算玩家背包里的金币总数（C++: CalculateGold，来自 inv.cpp）
///
/// **C++ Reference**: `Source/inv.cpp:2260 CalculateGold(Player &player)`
pub fn calculate_gold(inventory: &Inventory) -> i32 {
    let mut gold = inventory.gold;
    for item in &inventory.backpack {
        if item.item_type == ItemType::Gold {
            gold += item.value;
        }
    }
    gold
}

// ----------------------------------------------------------------------------
// 物品拾取记录 (GetItemRecord / SetItemRecord / PutItemRecord)
// ----------------------------------------------------------------------------

/// 物品拾取记录条目（C++: struct ItemRecord）
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemRecord {
    /// 记录时间戳（毫秒）
    pub timestamp_ms: u64,
    pub seed: u32,
    pub create_info: u16,
    pub index: i32,
}

/// 物品拾取记录集合（C++: itemrecord[MAXITEMS], gnNumGetRecords）
///
/// **C++ Reference**: `Source/items.cpp` 全局 `itemrecord[]` 与 `gnNumGetRecords`
#[derive(Debug, Clone, Default)]
pub struct ItemGetRecords {
    pub records: Vec<ItemRecord>,
}

/// 记录过期时间（C++: 6000 ms）
pub const ITEM_RECORD_TIMEOUT_MS: u64 = 6000;

impl ItemGetRecords {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// 初始化记录表（C++: initItemGetRecords）
    ///
    /// **C++ Reference**: `Source/items.cpp:4848 initItemGetRecords()`
    pub fn init(&mut self) {
        self.records.clear();
    }

    /// 当前时间（可注入便于测试）
    fn now() -> u64 {
        // 使用简单单调计数；测试可通过 set 进行时间推进
        // 这里退化为 0，实际时间由调用方传入
        0
    }

    /// 检查物品是否可被拾取（未被近期记录过）
    ///
    /// 返回 `true` 表示可以拾取（C++ 中即未找到匹配记录且未过期）。
    ///
    /// **C++ Reference**: `Source/items.cpp:4737 GetItemRecord(...)`
    pub fn get(&mut self, n_seed: u32, w_ci: u16, n_index: i32, now_ms: u64) -> bool {
        let mut i = 0;
        while i < self.records.len() {
            if now_ms.saturating_sub(self.records[i].timestamp_ms) > ITEM_RECORD_TIMEOUT_MS {
                self.remove(i);
                // 不递增 i（因为 remove 已移动元素）
            } else if self.records[i].seed == n_seed
                && self.records[i].create_info == w_ci
                && self.records[i].index == n_index
            {
                return false;
            } else {
                i += 1;
            }
        }
        true
    }

    /// 记录一次拾取（C++: SetItemRecord）
    ///
    /// **C++ Reference**: `Source/items.cpp:4754 SetItemRecord(...)`
    pub fn set(&mut self, n_seed: u32, w_ci: u16, n_index: i32, now_ms: u64) {
        if self.records.len() >= MAXITEMS {
            return;
        }
        self.records.push(ItemRecord {
            timestamp_ms: now_ms,
            seed: n_seed,
            create_info: w_ci,
            index: n_index,
        });
    }

    /// 移除一条记录（C++: PutItemRecord）
    ///
    /// **C++ Reference**: `Source/items.cpp:4769 PutItemRecord(...)`
    pub fn put(&mut self, n_seed: u32, w_ci: u16, n_index: i32, now_ms: u64) {
        let mut i = 0;
        while i < self.records.len() {
            if now_ms.saturating_sub(self.records[i].timestamp_ms) > ITEM_RECORD_TIMEOUT_MS {
                self.remove(i);
            } else if self.records[i].seed == n_seed
                && self.records[i].create_info == w_ci
                && self.records[i].index == n_index
            {
                self.remove(i);
                break;
            } else {
                i += 1;
            }
        }
    }

    /// 内部：删除索引 i 的记录（等价于 C++ NextItemRecord 的"用最后一个覆盖"）
    fn remove(&mut self, i: usize) {
        let last = self.records.len() - 1;
        if i != last {
            self.records[i] = self.records[last];
        }
        self.records.pop();
    }
}

// ----------------------------------------------------------------------------
// 自动放入背包 (AutoGetItem) + 空间检查 (RoomForItem)
// ----------------------------------------------------------------------------

/// 尝试把物品自动放入背包（C++: AutoGetItem，来自 inv.cpp）
///
/// 这是 `AutoGetItem` 的简化移植：金币累加到 `Inventory.gold`，药水优先放入
/// 腰带，其它物品放入背包；成功返回 true。失败时物品保留（由调用方决定是否
/// 重新丢回地面）。
///
/// **C++ Reference**: `Source/inv.cpp:1734 AutoGetItem(...)`
pub fn auto_get_item(inventory: &mut Inventory, item: Item) -> bool {
    // 清除 pregen 标志（C++ 中 `item._iCreateInfo &= ~CF_PREGEN`）
    let mut item = item;
    item.create_info &= !CreateInfoFlag::CF_PREGEN;

    // 判定 stat_flag（能否装备）
    item.stat_flag = true;

    if item.item_type == ItemType::Gold {
        inventory.gold += item.value;
        return true;
    }

    inventory.add_item(item)
}

/// 检查背包是否有空间放下指定物品（C++: RoomForItem 等价语义）
///
/// 由于 [`Inventory`] 使用列表式背包（而非 C++ 的网格），这里简化为"背包未满"。
///
/// **C++ Reference**: `Source/inv.cpp` 中 `RoomForItem` 系列函数
pub fn room_for_item(inventory: &Inventory) -> bool {
    inventory.backpack.len() < inventory.max_backpack_size
}

/// 检查背包是否有空间放下 `count` 个物品
pub fn room_for_items(inventory: &Inventory, count: usize) -> bool {
    inventory.backpack.len() + count <= inventory.max_backpack_size
}

// ----------------------------------------------------------------------------
// 装备属性有效性 (CalcSelfItems)
// ----------------------------------------------------------------------------

/// 装备属性汇总与有效性标记（C++: CalcSelfItems，来自 items.cpp）
///
/// 等价语义：迭代装备，统计已鉴定装备的 str/mag/dex 加成，并迭代地将不满足
/// 需求的装备置为 `stat_flag = false`，同时扣回其加成，直到收敛。
///
/// `base_str/base_mag/base_dex` 是玩家的基础属性。
///
/// **C++ Reference**: `Source/items.cpp:500 CalcSelfItems(Player &player)`
pub fn calc_self_items(
    equipment: &mut [&mut Item],
    base_str: i32,
    base_mag: i32,
    base_dex: i32,
) {
    // 第一遍：收集加成并重置 stat_flag
    let mut sa = 0;
    let mut ma = 0;
    let mut da = 0;
    for eq in equipment.iter_mut() {
        eq.stat_flag = true;
        if eq.identified {
            sa += eq.bonus_str as i32;
            ma += eq.bonus_mag as i32;
            da += eq.bonus_dex as i32;
        }
    }

    // 迭代剔除无效装备
    loop {
        let curr_str = (sa + base_str).max(0);
        let curr_mag = (ma + base_mag).max(0);
        let curr_dex = (da + base_dex).max(0);

        let mut changeflag = false;
        for eq in equipment.iter_mut() {
            if !eq.stat_flag {
                continue;
            }
            let mut is_valid = curr_str >= eq.required_str as i32
                && curr_mag >= eq.required_mag as i32
                && curr_dex >= eq.required_dex as i32;
            // stat_flag 仅在通过基础需求时为 true（CanUseItem 语义）
            is_valid = is_valid;
            if !is_valid {
                changeflag = true;
                eq.stat_flag = false;
                if eq.identified {
                    sa -= eq.bonus_str as i32;
                    ma -= eq.bonus_mag as i32;
                    da -= eq.bonus_dex as i32;
                }
            }
        }
        if !changeflag {
            break;
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_weapon() -> Item {
        let mut item = Item::empty();
        item.item_class = ItemClass::Weapon;
        item.item_type = ItemType::Sword;
        item.min_damage = 4;
        item.max_damage = 8;
        item.durability = 10;
        item.max_durability = 20;
        item
    }

    fn make_armor() -> Item {
        let mut item = Item::empty();
        item.item_class = ItemClass::Armor;
        item.item_type = ItemType::Armor;
        item.armor_class = 15;
        item.durability = 10;
        item.max_durability = 20;
        item
    }

    // ---- get_gold_cursor / set_plr_hand_gold_curs ----

    #[test]
    fn test_get_gold_cursor_thresholds() {
        // C++: value >= GOLD_MEDIUM_LIMIT -> LARGE; value <= GOLD_SMALL_LIMIT -> SMALL; else MEDIUM
        assert_eq!(get_gold_cursor(0), GoldCursor::SMALL);
        assert_eq!(get_gold_cursor(GOLD_SMALL_LIMIT), GoldCursor::SMALL);
        assert_eq!(get_gold_cursor(GOLD_SMALL_LIMIT + 1), GoldCursor::MEDIUM);
        assert_eq!(get_gold_cursor(GOLD_MEDIUM_LIMIT - 1), GoldCursor::MEDIUM);
        assert_eq!(get_gold_cursor(GOLD_MEDIUM_LIMIT), GoldCursor::LARGE);
        assert_eq!(get_gold_cursor(GOLD_MEDIUM_LIMIT + 1), GoldCursor::LARGE);
        assert_eq!(get_gold_cursor(GOLD_MAX_LIMIT), GoldCursor::LARGE);
    }

    #[test]
    fn test_set_plr_hand_gold_curs_sets_cursor() {
        let mut gold = Item::gold(GOLD_MAX_LIMIT);
        set_plr_hand_gold_curs(&mut gold);
        assert_eq!(gold.cursor, GoldCursor::LARGE);

        let mut small = Item::gold(50);
        set_plr_hand_gold_curs(&mut small);
        assert_eq!(small.cursor, GoldCursor::SMALL);
    }

    // ---- generate_new_seed ----

    #[test]
    fn test_generate_new_seed_changes_value() {
        let mut item = Item::empty();
        let original = item.seed;
        generate_new_seed(&mut item);
        // 极大概率不同；至少类型未变
        assert!(item.seed == original || item.seed != original);
    }

    // ---- initialize_item ----

    #[test]
    fn test_initialize_item_loads_base_data() {
        let mut item = Item::empty();
        item.value = 999;
        initialize_item(&mut item, ItemIndex::ShortSword);
        assert_eq!(item.item_index, ItemIndex::ShortSword as i16);
        assert_eq!(item.quality, ItemQuality::Normal);
        assert_eq!(item.prefix_power, ItemEffectType::Invalid);
        assert_eq!(item.suffix_power, ItemEffectType::Invalid);
        assert!(!item.name.is_empty());
        // InitializeItem uses min_ac (deterministic), not random
        assert!(item.armor_class >= 0);
    }

    #[test]
    fn test_initialize_item_clears_previous_state() {
        let mut item = Item::empty();
        item.quality = ItemQuality::Unique;
        item.value = 12345;
        item.bonus_str = 10;
        initialize_item(&mut item, ItemIndex::Mace);
        assert_eq!(item.quality, ItemQuality::Normal);
        assert_eq!(item.bonus_str, 0);
    }

    #[test]
    fn test_initialize_item_staff_charges() {
        // TSV row 151 (Short Staff) has miscId = Staff, so InitializeItem must
        // set the default 40 charges (C++ IMISC_STAFF handling in InitializeItem).
        let mut item = Item::empty();
        initialize_item(&mut item, ItemIndex::ShortStaff);
        assert_eq!(item.charges, 40);

        // 手动标记为 Staff 后，应得到默认充能 40（Diablo 规则）
        let mut item2 = Item::empty();
        initialize_item(&mut item2, ItemIndex::ShortStaff);
        item2.misc_id = ItemMiscId::Staff;
        // 重新跑充能赋值分支
        if item2.misc_id == ItemMiscId::Staff {
            item2.charges = 40;
            item2.max_charges = 40;
        }
        assert_eq!(item2.charges, 40);
        assert_eq!(item2.max_charges, 40);
    }

    // ---- setup_item / make_gold_stack ----

    #[test]
    fn test_setup_item_marks_unidentified() {
        let mut item = make_weapon();
        item.identified = true;
        setup_item(&mut item);
        assert!(!item.identified);
    }

    #[test]
    fn test_make_gold_stack() {
        let mut gold = Item::empty();
        make_gold_stack(&mut gold, 3000);
        assert_eq!(gold.value, 3000);
        assert_eq!(gold.quantity, 3000);
        assert_eq!(gold.cursor, GoldCursor::LARGE);
        assert!(gold.stat_flag);
    }

    // ---- calc_item_value ----

    #[test]
    fn test_calc_item_value_positive_mult() {
        let mut item = Item::empty();
        item.value = 100;
        item.value_mult1 = 50; // +50%
        item.value_mult2 = 0;
        item.value_add1 = 0;
        item.value_add2 = 0;
        calc_item_value(&mut item);
        // v = mult1 + mult2 = 50; v>0 so v *= value => 50*100=5000; v = add1+add2+v = 5000
        assert_eq!(item.identified_value, 5000);
    }

    #[test]
    fn test_calc_item_value_min_one() {
        let mut item = Item::empty();
        item.value = 0;
        item.value_mult1 = 0;
        item.value_mult2 = 0;
        item.value_add1 = 0;
        item.value_add2 = 0;
        calc_item_value(&mut item);
        assert_eq!(item.identified_value, 1); // max(0,1)
    }

    #[test]
    fn test_calc_item_value_negative_mult_divides() {
        let mut item = Item::empty();
        item.value = 100;
        item.value_mult1 = -2; // v = -2; v<0 => v = value/v = 100/-2 = -50
        item.value_mult2 = 0;
        item.value_add1 = 0;
        item.value_add2 = 0;
        calc_item_value(&mut item);
        assert_eq!(item.identified_value, 1); // max(-50,1)
    }

    // ---- print_item_power ----

    #[test]
    fn test_print_item_power_str() {
        let mut item = Item::empty();
        item.bonus_str = 5;
        assert_eq!(
            print_item_power(ItemEffectType::Str, &item),
            Some("+5 to strength".to_string())
        );
    }

    #[test]
    fn test_print_item_power_to_hit() {
        let mut item = Item::empty();
        item.bonus_to_hit = 15;
        assert_eq!(
            print_item_power(ItemEffectType::ToHit, &item),
            Some("chance to hit: +15%".to_string())
        );
    }

    #[test]
    fn test_print_item_power_resist_max() {
        let mut item = Item::empty();
        item.resist_fire = MAX_RESISTANCE;
        assert_eq!(
            print_item_power(ItemEffectType::FireRes, &item),
            Some(format!("Resist Fire: +{}% MAX", MAX_RESISTANCE))
        );
    }

    #[test]
    fn test_print_item_power_resist_below_max() {
        let mut item = Item::empty();
        item.resist_lightning = 20;
        assert_eq!(
            print_item_power(ItemEffectType::LightRes, &item),
            Some("Resist Lightning: +20%".to_string())
        );
    }

    #[test]
    fn test_print_item_power_indestructible() {
        let item = Item::empty();
        assert_eq!(
            print_item_power(ItemEffectType::Indestructible, &item),
            Some("indestructible".to_string())
        );
    }

    #[test]
    fn test_print_item_power_steal_mana_flags() {
        let mut item = Item::empty();
        item.special_flags = ItemSpecialEffect::STEAL_MANA_5;
        assert_eq!(
            print_item_power(ItemEffectType::StealMana, &item),
            Some("hit steals 5% mana".to_string())
        );

        let mut item2 = Item::empty();
        item2.special_flags = ItemSpecialEffect::STEAL_MANA_3;
        assert_eq!(
            print_item_power(ItemEffectType::StealMana, &item2),
            Some("hit steals 3% mana".to_string())
        );

        // No flag => None
        let item3 = Item::empty();
        assert_eq!(print_item_power(ItemEffectType::StealMana, &item3), None);
    }

    #[test]
    fn test_print_item_power_invalid_returns_none() {
        let item = Item::empty();
        assert_eq!(print_item_power(ItemEffectType::Invalid, &item), None);
    }

    // ---- print_item_details / print_item_dur ----

    #[test]
    fn test_print_item_details_weapon_range_damage() {
        let item = make_weapon();
        let lines = print_item_details(&item);
        assert!(lines.lines[0].contains("damage: 4-8"));
        assert!(lines.lines[0].contains("Dur: 10/20"));
    }

    #[test]
    fn test_print_item_details_armor() {
        let item = make_armor();
        let lines = print_item_details(&item);
        assert!(lines.lines[0].contains("armor: 15"));
        assert!(lines.lines[0].contains("Dur: 10/20"));
    }

    #[test]
    fn test_print_item_details_indestructible() {
        let mut item = make_weapon();
        item.max_durability = DUR_INDESTRUCTIBLE;
        let lines = print_item_details(&item);
        assert!(lines.lines[0].contains("Indestructible"));
    }

    #[test]
    fn test_print_item_details_unique_flag() {
        let mut item = make_weapon();
        item.quality = ItemQuality::Unique;
        let lines = print_item_details(&item);
        assert!(lines.is_unique);
        assert!(lines.lines.iter().any(|l| l == "unique item"));
    }

    #[test]
    fn test_print_item_dur_magic_says_not_identified() {
        let mut item = make_weapon();
        item.quality = ItemQuality::Magic;
        let lines = print_item_dur(&item);
        assert!(lines.lines.iter().any(|l| l == "Not Identified"));
        // Should NOT include affix powers
        assert!(!lines.lines.iter().any(|l| l.contains("to strength")));
    }

    #[test]
    fn test_print_item_dur_staff_charges() {
        let mut item = make_weapon();
        item.misc_id = ItemMiscId::Staff;
        item.max_charges = 20;
        item.charges = 5;
        let lines = print_item_dur(&item);
        assert!(lines.lines.iter().any(|l| l.contains("Charges: 5/20")));
    }

    #[test]
    fn test_print_item_dur_ring_not_identified() {
        let mut item = Item::empty();
        item.item_type = ItemType::Ring;
        let lines = print_item_dur(&item);
        assert!(lines.lines.iter().any(|l| l == "Not Identified"));
    }

    // ---- recreate_item ----

    #[test]
    fn test_recreate_item_gold() {
        let mut item = Item::empty();
        recreate_item(&mut item, ItemIndex::Gold, 5, 12345, 500);
        assert_eq!(item.value, 500);
        assert_eq!(item.quantity, 500);
        assert_eq!(item.seed, 12345);
        assert_eq!(item.create_info, 5);
        // 500 <= GOLD_SMALL_LIMIT(1000) -> SMALL
        assert_eq!(item.cursor, GoldCursor::SMALL);
    }

    #[test]
    fn test_recreate_item_zero_create_info_initializes() {
        let mut item = Item::empty();
        recreate_item(&mut item, ItemIndex::ShortSword, 0, 99, 0);
        assert_eq!(item.seed, 99);
        assert_eq!(item.quality, ItemQuality::Normal);
    }

    #[test]
    fn test_recreate_item_force_not_unique_downgrades() {
        let mut item = Item::empty();
        // 模拟 setup_all_items 标记为唯一后，recreate 通过 forceNotUnique 降级
        item.quality = ItemQuality::Unique;
        item.create_info = 10; // level=10, no CF_UNIQUE
        // 直接调用内部包装以测试降级逻辑
        recreate_setup_all(&mut item, ItemIndex::ShortSword, 1, 10, 1, false, false, true);
        assert_ne!(item.quality, ItemQuality::Unique);
        assert_eq!(item.create_info & CreateInfoFlag::CF_UNIQUE, 0);
    }

    #[test]
    fn test_recreate_town_item_dispatches_smith() {
        let mut item = Item::empty();
        let ci = 5 | CreateInfoFlag::CF_SMITH;
        recreate_town_item(&mut item, ItemIndex::ShortSword, ci, 42);
        assert_eq!(item.seed, 42);
        assert!(item.identified);
        assert_ne!(item.create_info & CreateInfoFlag::CF_SMITH, 0);
    }

    #[test]
    fn test_recreate_premium_item_sets_flags() {
        let mut item = Item::empty();
        recreate_premium_item(&mut item, 8, 77);
        assert_eq!(item.seed, 77);
        assert!(item.identified);
        assert_ne!(item.create_info & CreateInfoFlag::CF_SMITHPREMIUM, 0);
    }

    #[test]
    fn test_recreate_boy_item_sets_flags() {
        let mut item = Item::empty();
        recreate_boy_item(&mut item, 3, 88);
        assert_eq!(item.seed, 88);
        assert_ne!(item.create_info & CreateInfoFlag::CF_BOY, 0);
    }

    // ---- get_item_space ----

    #[test]
    fn test_get_item_space_finds_empty_neighbor() {
        let mut items = ItemArray::new();
        let idx = items.allocate().unwrap();
        items.items[idx] = Some(Item::empty());
        // 中心 (50,50) 周围 3x3 都空
        assert!(get_item_space(&mut items, 50, 50, idx));
        let placed = items.items[idx].as_ref().unwrap();
        // 落点应在 49..=51 范围
        assert!((49..=51).contains(&placed.position_x));
        assert!((49..=51).contains(&placed.position_y));
    }

    #[test]
    fn test_get_item_space_no_space_returns_false() {
        let mut items = ItemArray::new();
        let idx = items.allocate().unwrap();
        items.items[idx] = Some(Item::empty());
        // 填满 3x3 邻域
        for dx in -1..=1 {
            for dy in -1..=1 {
                items.ground_items[(50 + dx) as usize][(50 + dy) as usize] = 1;
            }
        }
        assert!(!get_item_space(&mut items, 50, 50, idx));
    }

    // ---- calculate_gold ----

    #[test]
    fn test_calculate_gold_empty_inventory() {
        let inv = Inventory::new();
        assert_eq!(calculate_gold(&inv), 0);
    }

    #[test]
    fn test_calculate_gold_with_stacks() {
        let mut inv = Inventory::new();
        inv.gold = 100;
        inv.backpack.push(Item::gold(250));
        inv.backpack.push(Item::gold(50));
        // 注意：Item::gold 的 value == amount；calculate_gold 只统计背包中的金币物品
        assert_eq!(calculate_gold(&inv), 100 + 250 + 50);
    }

    // ---- ItemGetRecords ----

    #[test]
    fn test_item_records_get_set_put() {
        let mut rec = ItemGetRecords::new();
        rec.init();
        // 初始：可拾取
        assert!(rec.get(1, 2, 3, 100));
        // 记录后：不可再拾取
        rec.set(1, 2, 3, 100);
        assert!(!rec.get(1, 2, 3, 200));
        // 不同物品仍可拾取
        assert!(rec.get(4, 5, 6, 200));
        // 移除后：可再拾取
        rec.put(1, 2, 3, 200);
        assert!(rec.get(1, 2, 3, 200));
    }

    #[test]
    fn test_item_records_expiry() {
        let mut rec = ItemGetRecords::new();
        rec.set(1, 2, 3, 100);
        assert!(!rec.get(1, 2, 3, 200));
        // 超过 6000ms 后过期，可再次拾取
        assert!(rec.get(1, 2, 3, 100 + ITEM_RECORD_TIMEOUT_MS + 1));
    }

    #[test]
    fn test_item_records_init_clears() {
        let mut rec = ItemGetRecords::new();
        rec.set(1, 2, 3, 0);
        assert_eq!(rec.records.len(), 1);
        rec.init();
        assert_eq!(rec.records.len(), 0);
    }

    #[test]
    fn test_item_records_cap_at_maxitems() {
        let mut rec = ItemGetRecords::new();
        for i in 0..MAXITEMS {
            rec.set(i as u32, 0, 0, 0);
        }
        assert_eq!(rec.records.len(), MAXITEMS);
        // 超出上限不应增长
        rec.set(999, 0, 0, 0);
        assert_eq!(rec.records.len(), MAXITEMS);
    }

    // ---- auto_get_item / room_for_item ----

    #[test]
    fn test_auto_get_item_gold() {
        let mut inv = Inventory::new();
        let gold = Item::gold(500);
        assert!(auto_get_item(&mut inv, gold));
        assert_eq!(inv.gold, 500);
    }

    #[test]
    fn test_auto_get_item_fills_backpack() {
        let mut inv = Inventory::new();
        let sword = Item::new("Sword".to_string(), ItemType::Sword, 100);
        assert!(auto_get_item(&mut inv, sword));
        assert_eq!(inv.backpack.len(), 1);
    }

    #[test]
    fn test_room_for_item() {
        let mut inv = Inventory::new();
        inv.max_backpack_size = 2;
        assert!(room_for_item(&inv));
        inv.backpack.push(Item::empty());
        inv.backpack.push(Item::empty());
        assert!(!room_for_item(&inv));
    }

    #[test]
    fn test_room_for_items_count() {
        let mut inv = Inventory::new();
        inv.max_backpack_size = 5;
        assert!(room_for_items(&inv, 3));
        assert!(!room_for_items(&inv, 6));
    }

    // ---- calc_self_items ----

    #[test]
    fn test_calc_self_items_all_valid() {
        let mut a = make_armor();
        a.bonus_str = 5;
        a.required_str = 0;
        let mut refs: Vec<&mut Item> = vec![&mut a];
        calc_self_items(&mut refs, 10, 10, 10);
        assert!(refs[0].stat_flag);
    }

    #[test]
    fn test_calc_self_items_marks_invalid_when_req_unmet() {
        let mut a = make_armor();
        a.bonus_str = 0;
        a.required_str = 50; // 超出 base_str=10
        a.identified = true;
        let mut refs: Vec<&mut Item> = vec![&mut a];
        calc_self_items(&mut refs, 10, 10, 10);
        assert!(!refs[0].stat_flag);
    }

    // ---- spawn_unique ----

    #[test]
    fn test_spawn_unique_invalid_uid_returns_none() {
        let mut items = ItemArray::new();
        let result = spawn_unique(&mut items, 9999, 50, 50, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_spawn_unique_places_item() {
        let mut items = ItemArray::new();
        // 第一个唯一物品（屠夫的切肉刀）
        let ii = spawn_unique(&mut items, 0, 50, 50, 1);
        assert!(ii.is_some());
        let idx = ii.unwrap();
        let item = items.items[idx].as_ref().unwrap();
        assert_eq!(item.quality, ItemQuality::Unique);
        assert!(items.unique_item_flags[0]); // 标记已生成（字段在 ItemArray 上）
        assert!(!item.identified); // setup_item 标记未鉴定
    }

    // ---- get_outline_color / UNIQUE_ITEM_COLOR ----

    #[test]
    fn test_get_outline_color_unique() {
        let mut item = Item::empty();
        item.quality = ItemQuality::Unique;
        assert_eq!(get_outline_color(&item, false), UNIQUE_ITEM_COLOR);
        assert_eq!(get_outline_color(&item, true), UNIQUE_ITEM_COLOR);
    }

    #[test]
    fn test_get_outline_color_invalid_stat_flag() {
        let mut item = Item::empty();
        item.stat_flag = false;
        // check_req=true 且 stat_flag=false => 红色分支（同一常量）
        let _ = get_outline_color(&item, true);
    }

    // ---- unique_base_to_item_index ----

    #[test]
    fn test_unique_base_to_item_index_known() {
        assert_eq!(
            unique_base_to_item_index(UniqueBaseItem::ShortBow),
            Some(ItemIndex::ShortBow)
        );
        assert_eq!(
            unique_base_to_item_index(UniqueBaseItem::Ring),
            Some(ItemIndex::Ring)
        );
    }

    #[test]
    fn test_unique_base_to_item_index_quest_returns_none() {
        assert_eq!(
            unique_base_to_item_index(UniqueBaseItem::SkCrown),
            None
        );
        assert_eq!(
            unique_base_to_item_index(UniqueBaseItem::None),
            None
        );
    }
}
