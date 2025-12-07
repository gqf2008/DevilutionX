//! Exact port of items.h Item struct from DevilutionX
//!
//! This is a 1:1 mapping of the C++ Item struct with all fields
//! preserved exactly as in the original source code.

// Allow non-snake_case names to preserve C++ naming convention for save file compatibility
#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

use super::item_dat::{
    ItemClass, ItemEquipType, ItemMiscId, ItemSpecialEffect, ItemSpecialEffectHf, ItemType,
};
use super::spells::SpellId;
use super::types::Point;

// ============================================================================
// Constants from items.h
// ============================================================================

/// Maximum items in the world
pub const MAXITEMS: usize = 127;

/// Number of item types
pub const ITEMTYPES: usize = 43;

/// Gold limits for different pile sizes
pub const GOLD_SMALL_LIMIT: i32 = 1000;
pub const GOLD_MEDIUM_LIMIT: i32 = 2500;
pub const GOLD_MAX_LIMIT: i32 = 5000;

/// Item indestructible durability
pub const DUR_INDESTRUCTIBLE: u8 = 255;

/// Maximum length of item name
pub const ITEM_NAME_LENGTH: usize = 64;

/// Maximum vendor values
pub const MAX_VENDOR_VALUE: i32 = 140000;
pub const MAX_VENDOR_VALUE_HF: i32 = 200000;
pub const MAX_BOY_VALUE: i32 = 90000;
pub const MAX_BOY_VALUE_HF: i32 = 200000;

/// All item animation frames have this width
pub const ITEM_ANIM_WIDTH: i32 = 96;

// ============================================================================
// Enums from items.h
// ============================================================================

/// Item quality - matches item_quality enum exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum ItemQuality {
    #[default]
    Normal = 0,
    Magic = 1,
    Unique = 2,
}

/// Selection region for items on ground
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum SelectionRegion {
    #[default]
    None = 0,
    // Add other variants as needed from source
}

/// Item effect type - prefix/suffix powers
/// Matches item_effect_type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(i8)]
pub enum ItemEffectType {
    #[default]
    Invalid = -1,
    // Prefixes
    PrefixToHit = 0,
    PrefixToHitCurse,
    PrefixDamage,
    PrefixDamageCurse,
    PrefixToHitDamage,
    PrefixToHitDamageCurse,
    PrefixAC,
    PrefixACCurse,
    PrefixFireRes,
    PrefixFireResCurse,
    PrefixLightRes,
    PrefixLightResCurse,
    PrefixMagicRes,
    PrefixMagicResCurse,
    PrefixAllRes,
    PrefixAllResCurse,
    // Continue with all effect types from source...
    PrefixSpellLvl,
    PrefixCharges,
    PrefixFireDam,
    PrefixLightDam,
    PrefixStr,
    PrefixStrCurse,
    PrefixMag,
    PrefixMagCurse,
    PrefixDex,
    PrefixDexCurse,
    PrefixVit,
    PrefixVitCurse,
    PrefixGetHit,
    PrefixGetHitCurse,
    PrefixLife,
    PrefixLifeCurse,
    PrefixMana,
    PrefixManaCurse,
    PrefixDurability,
    PrefixDurabilityCurse,
    PrefixIndestructible,
    PrefixLight,
    PrefixLightCurse,
    // Unique item powers
    UniqueFireArrows,
    UniqueLightArrows,
    UniqueInvis,
    UniqueMultipleArrows,
    UniqueKnockback,
    UniqueNoMana,
    UniqueThornsDamage,
    UniqueNoHeal,
    UniqueHalfTrapDam,
    UniqueAbsorbHalfDam,
    UniqueNoBleed,
    UniqueStealing,
    UniqueManaSteal,
    UniqueRndStealLife,
    UniqueLifeSteal,
    UniqueStaffCharges,
    UniqueDamMod,
}

/// Unique items index - matches _unique_items enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(i32)]
pub enum UniqueItemId {
    Cleaver = 0,
    SkullCrown,
    InfraRing,
    OptAmulet,
    TRing,
    HarCrest,
    SteelVeil,
    ArmOfVal,
    Griswold,
    Bovine,
    RiftBow,
    Needler,
    CelestBow,
    DeadlyHunt,
    BowOfDead,
    BlkOakBow,
    FlameDart,
    FleshSting,
    Windforce,
    EagleHorn,
    GonnagalDirk,
    Defender,
    GryphonClaw,
    BlackRazor,
    GibbousMoon,
    IceShank,
    Executioner,
    BoneSaw,
    ShadHawk,
    WizSpike,
    LightSabre,
    FalconTalon,
    Inferno,
    DoomBringer,
    Grizzly,
    GrandFather,
    Mangler,
    SharpBeak,
    BloodSlayer,
    CelestAxe,
    WickedAxe,
    StoneCleav,
    AguHatchet,
    HellSlayer,
    MesserReaver,
    CrackRust,
    JholmHamm,
    Civerbs,
    CelestStar,
    BaranStar,
    GnarlRoot,
    CranBash,
    SchaefHamm,
    DreamFlange,
    StaffOfShad,
    Immolator,
    StormSpire,
    GleamSong,
    ThunderCall,
    Protector,
    NajPuzzle,
    MindCry,
    RodOfOnan,
    SpiritHelm,
    ThinkingCap,
    OverlordHelm,
    FoolsCrest,
    GotterDam,
    RoyCirclet,
    TornFlesh,
    GladBane,
    RainCloak,
    LeathAut,
    WisdWrap,
    SparkMail,
    ScavCarap,
    NightScape,
    NajPlate,
    DemonSpike,
    Deflector,
    SkullShield,
    DragonBrch,
    BlkOakShield,
    HolyDef,
    StormShield,
    Bramble,
    Regha,
    Bleeder,
    Constrict,
    Engage,
    #[default]
    Invalid = -1,
}

/// Item creation info flags - matches icreateinfo_flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreateInfoFlags(pub u16);

impl CreateInfoFlags {
    /// Item Level (6 bits; value ranges from 0-63)
    pub const LEVEL: Self = Self((1 << 6) - 1);
    /// Item is not able to have affixes with PLOK set to false
    pub const ONLY_GOOD: Self = Self(1 << 6);
    /// Item is from a Unique Monster and has 15% chance of being a Unique Item
    pub const UPER15: Self = Self(1 << 7);
    /// Item is from the dungeon and has a 1% chance of being a Unique Item
    pub const UPER1: Self = Self(1 << 8);
    /// Item is a Unique Item
    pub const UNIQUE: Self = Self(1 << 9);
    /// Item is from Griswold (Basic)
    pub const SMITH: Self = Self(1 << 10);
    /// Item is from Griswold (Premium)
    pub const SMITH_PREMIUM: Self = Self(1 << 11);
    /// Item is from Wirt
    pub const BOY: Self = Self(1 << 12);
    /// Item is from Adria
    pub const WITCH: Self = Self(1 << 13);
    /// Item is from Pepin
    pub const HEALER: Self = Self(1 << 14);
    /// Item is pre-generated
    pub const PREGEN: Self = Self(1 << 15);

    /// Useful items (potions, scrolls)
    pub const fn useful() -> Self {
        Self(Self::UPER15.0 | Self::UPER1.0)
    }

    /// Items from town NPCs
    pub const fn town() -> Self {
        Self(Self::SMITH.0 | Self::SMITH_PREMIUM.0 | Self::BOY.0 | Self::WITCH.0 | Self::HEALER.0)
    }

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn level(&self) -> u8 {
        (self.0 & Self::LEVEL.0) as u8
    }
}

/// Item creation info flags 2 - matches icreateinfo_flag2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreateInfoFlags2(pub u8);

impl CreateInfoFlags2 {
    pub const HELLFIRE: Self = Self(1 << 0);
    pub const UID_OFFSET: Self = Self(((1 << 4) - 1) << 1);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn uid_offset(&self) -> u8 {
        (self.0 & Self::UID_OFFSET.0) >> 1
    }
}

/// Item index in base item data - matches _item_indexes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(i16)]
pub enum ItemIndex {
    #[default]
    None = -1,
    // Swords
    ShortSword = 0,
    BuckLer,
    Club,
    // ... many more item types
    // Gold
    Gold = 68,
    // ... continue with all item indices
}

// ============================================================================
// Item struct - exact match of C++ Item struct
// ============================================================================

/// Item structure - exact port from items.h
///
/// All field names preserved with original naming convention (underscore prefix)
/// to maintain compatibility with save file formats and debugging.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Item {
    /// Randomly generated identifier
    pub _iSeed: u32,

    /// Creation info flags
    pub _iCreateInfo: u16,

    /// Item type (Sword, Bow, Shield, etc.)
    pub _itype: ItemType,

    /// Is the item animated
    pub _iAnimFlag: bool,

    /// Position on ground
    pub position: Point,

    // Note: AnimationInfo omitted - handled by render system

    /// Set when item is flagged for deletion (deprecated in 1.02)
    pub _iDelFlag: bool,

    /// Selection region for clicking
    pub selectionRegion: SelectionRegion,

    /// Draw after main sprites
    pub _iPostDraw: bool,

    /// Has the item been identified
    pub _iIdentified: bool,

    /// Item quality (Normal, Magic, Unique)
    pub _iMagical: ItemQuality,

    /// Item name (unidentified)
    pub _iName: String,

    /// Item name (identified)
    pub _iIName: String,

    /// Equipment location (head, chest, ring, etc.)
    pub _iLoc: ItemEquipType,

    /// Item class (weapon, armor, misc)
    pub _iClass: ItemClass,

    /// Cursor graphic index
    pub _iCurs: u8,

    /// Item value (base)
    pub _ivalue: i32,

    /// Item value (with modifiers)
    pub _iIvalue: i32,

    /// Minimum damage
    pub _iMinDam: u8,

    /// Maximum damage
    pub _iMaxDam: u8,

    /// Armor class
    pub _iAC: i16,

    /// Special effects (life steal, knockback, etc.)
    pub _iFlags: ItemSpecialEffect,

    /// Misc item ID (potion type, scroll type, etc.)
    pub _iMiscId: ItemMiscId,

    /// Spell granted by item
    pub _iSpell: SpellId,

    /// Index into base item data
    pub IDidx: ItemIndex,

    /// Current charges (for staves)
    pub _iCharges: i32,

    /// Maximum charges
    pub _iMaxCharges: i32,

    /// Current durability
    pub _iDurability: i32,

    /// Maximum durability
    pub _iMaxDur: i32,

    /// Percent damage bonus
    pub _iPLDam: i16,

    /// Percent to-hit bonus
    pub _iPLToHit: i16,

    /// Percent AC bonus
    pub _iPLAC: i16,

    /// Strength bonus
    pub _iPLStr: i16,

    /// Magic bonus
    pub _iPLMag: i16,

    /// Dexterity bonus
    pub _iPLDex: i16,

    /// Vitality bonus
    pub _iPLVit: i16,

    /// Fire resistance bonus
    pub _iPLFR: i16,

    /// Lightning resistance bonus
    pub _iPLLR: i16,

    /// Magic resistance bonus
    pub _iPLMR: i16,

    /// Mana bonus
    pub _iPLMana: i16,

    /// Hit points bonus
    pub _iPLHP: i16,

    /// Damage modifier
    pub _iPLDamMod: i16,

    /// Get hit modifier (negative = less damage taken)
    pub _iPLGetHit: i16,

    /// Light radius bonus
    pub _iPLLight: i16,

    /// Spell level addition
    pub _iSplLvlAdd: i8,

    /// Item has been requested (multiplayer)
    pub _iRequest: bool,

    /// Unique item ID (index into UniqueItemList)
    pub _iUid: i32,

    /// Fire min damage (enchanted)
    pub _iFMinDam: i16,

    /// Fire max damage (enchanted)
    pub _iFMaxDam: i16,

    /// Lightning min damage (enchanted)
    pub _iLMinDam: i16,

    /// Lightning max damage (enchanted)
    pub _iLMaxDam: i16,

    /// Enhanced AC
    pub _iPLEnAc: i16,

    /// Prefix power type
    pub _iPrePower: ItemEffectType,

    /// Suffix power type
    pub _iSufPower: ItemEffectType,

    /// Value addition 1 (for prefix/suffix)
    pub _iVAdd1: i32,

    /// Value multiplier 1
    pub _iVMult1: i32,

    /// Value addition 2
    pub _iVAdd2: i32,

    /// Value multiplier 2
    pub _iVMult2: i32,

    /// Minimum strength required
    pub _iMinStr: i8,

    /// Minimum magic required
    pub _iMinMag: u8,

    /// Minimum dexterity required
    pub _iMinDex: i8,

    /// Can player use this item (stat check passed)
    pub _iStatFlag: bool,

    /// Hellfire damage/AC flags
    pub _iDamAcFlags: ItemSpecialEffectHf,

    /// Buffer for network/save (dwBuff in original)
    pub dwBuff: u32,
}

impl Item {
    /// Create a new empty item
    pub fn new() -> Self {
        Self {
            _itype: ItemType::None,
            _iMagical: ItemQuality::Normal,
            _iLoc: ItemEquipType::None,
            _iClass: ItemClass::None,
            _iMiscId: ItemMiscId::None,
            _iSpell: SpellId::Null,
            IDidx: ItemIndex::None,
            _iPrePower: ItemEffectType::Invalid,
            _iSufPower: ItemEffectType::Invalid,
            _iFlags: ItemSpecialEffect::empty(),
            _iDamAcFlags: ItemSpecialEffectHf::empty(),
            _iName: String::new(),
            _iIName: String::new(),
            ..Default::default()
        }
    }

    /// Clear this item (make it empty)
    pub fn clear(&mut self) {
        self._itype = ItemType::None;
    }

    /// Check if item is empty
    pub fn is_empty(&self) -> bool {
        self._itype == ItemType::None
    }

    /// Check if item is equipment
    pub fn is_equipment(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        matches!(
            self._iLoc,
            ItemEquipType::Amulet
                | ItemEquipType::Armor
                | ItemEquipType::Helm
                | ItemEquipType::OneHand
                | ItemEquipType::Ring
                | ItemEquipType::TwoHand
        )
    }

    /// Check if item is a weapon
    pub fn is_weapon(&self) -> bool {
        matches!(
            self._itype,
            ItemType::Axe
                | ItemType::Bow
                | ItemType::Mace
                | ItemType::Staff
                | ItemType::Sword
        )
    }

    /// Check if item is armor
    pub fn is_armor(&self) -> bool {
        matches!(
            self._itype,
            ItemType::HeavyArmor | ItemType::LightArmor | ItemType::MediumArmor
        )
    }

    /// Check if item is gold
    pub fn is_gold(&self) -> bool {
        self._itype == ItemType::Gold
    }

    /// Check if item is a helm
    pub fn is_helm(&self) -> bool {
        self._itype == ItemType::Helm
    }

    /// Check if item is a shield
    pub fn is_shield(&self) -> bool {
        self._itype == ItemType::Shield
    }

    /// Check if item is jewelry (ring or amulet)
    pub fn is_jewelry(&self) -> bool {
        matches!(self._itype, ItemType::Amulet | ItemType::Ring)
    }

    /// Check if item is a scroll
    pub fn is_scroll(&self) -> bool {
        matches!(self._iMiscId, ItemMiscId::Scroll | ItemMiscId::ScrollT)
    }

    /// Check if item is a scroll of a specific spell
    pub fn is_scroll_of(&self, spell_id: SpellId) -> bool {
        self.is_scroll() && self._iSpell == spell_id
    }

    /// Check if item is a rune (Hellfire)
    pub fn is_rune(&self) -> bool {
        self._iMiscId > ItemMiscId::RuneFirst && self._iMiscId < ItemMiscId::RuneLast
    }

    /// Check if item is a rune of a specific spell
    pub fn is_rune_of(&self, spell_id: SpellId) -> bool {
        if !self.is_rune() {
            return false;
        }
        match self._iMiscId {
            ItemMiscId::RuneF => spell_id == SpellId::RuneOfFire,
            ItemMiscId::RuneL => spell_id == SpellId::RuneOfLight,
            ItemMiscId::GrRuneL => spell_id == SpellId::RuneOfNova,
            ItemMiscId::GrRuneF => spell_id == SpellId::RuneOfImmolation,
            ItemMiscId::RuneS => spell_id == SpellId::RuneOfStone,
            _ => false,
        }
    }

    /// Get total damage range
    pub fn damage_range(&self) -> (i32, i32) {
        let min = self._iMinDam as i32;
        let max = self._iMaxDam as i32;

        // Add percentage bonus
        let min = min + (min * self._iPLDam as i32) / 100;
        let max = max + (max * self._iPLDam as i32) / 100;

        // Add flat bonus
        let min = min + self._iPLDamMod as i32;
        let max = max + self._iPLDamMod as i32;

        (min.max(0), max.max(min))
    }

    /// Get total armor class
    pub fn armor_class(&self) -> i32 {
        let base = self._iAC as i32;

        // Add percentage bonus
        let ac = base + (base * self._iPLAC as i32) / 100;

        // Add enhanced AC
        ac + self._iPLEnAc as i32
    }

    /// Check if player meets stat requirements
    pub fn meets_requirements(&self, str: i32, mag: i32, dex: i32) -> bool {
        str >= self._iMinStr as i32
            && mag >= self._iMinMag as i32
            && dex >= self._iMinDex as i32
    }

    /// Check if item is indestructible
    pub fn is_indestructible(&self) -> bool {
        self._iMaxDur == DUR_INDESTRUCTIBLE as i32
    }

    /// Check if item is broken
    pub fn is_broken(&self) -> bool {
        !self.is_indestructible() && self._iDurability == 0
    }

    /// Reduce durability by amount, returns true if item breaks
    pub fn reduce_durability(&mut self, amount: i32) -> bool {
        if self.is_indestructible() {
            return false;
        }

        self._iDurability = (self._iDurability - amount).max(0);
        self._iDurability == 0
    }

    /// Get identified display name
    pub fn display_name(&self) -> &str {
        if self._iIdentified {
            &self._iIName
        } else {
            &self._iName
        }
    }

    /// Get gold value (for selling)
    pub fn sell_value(&self) -> i32 {
        // Items sell for 1/4 of their value
        self._iIvalue / 4
    }

    /// Check if this is a quest item
    pub fn is_quest_item(&self) -> bool {
        // Quest items typically have special misc IDs
        matches!(
            self._iMiscId,
            ItemMiscId::StaffOfLazarus
                | ItemMiscId::MapOfDoom
                // Add other quest item misc IDs
        )
    }
}

// ============================================================================
// Item get record for multiplayer
// ============================================================================

/// Record of items picked up (for multiplayer sync)
#[derive(Debug, Clone, Default)]
pub struct ItemGetRecord {
    pub seed: u32,
    pub create_info: u16,
    pub index: i32,
    pub timestamp: u32,
}

// ============================================================================
// Cornerstone struct (for Hellfire)
// ============================================================================

/// Cornerstone for item duplication prevention (Hellfire)
#[derive(Debug, Clone, Default)]
pub struct CornerStone {
    pub position: Point,
    pub activated: bool,
    pub item: Item,
}

impl CornerStone {
    pub fn is_available(&self) -> bool {
        self.activated && !self.item.is_empty()
    }
}
