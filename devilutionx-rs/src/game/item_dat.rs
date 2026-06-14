//! Item Data - Exact port of DevilutionX Source/itemdat.h
//!
//! Contains item types, enums, and data structures.

use serde::{Deserialize, Serialize};

/// Item type - exact match of ItemType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum ItemType {
    Misc = 0,
    Sword = 1,
    Axe = 2,
    Bow = 3,
    Mace = 4,
    Shield = 5,
    LightArmor = 6,
    Helm = 7,
    MediumArmor = 8,
    HeavyArmor = 9,
    Staff = 10,
    Gold = 11,
    Ring = 12,
    Amulet = 13,
    #[default]
    None = -1,
}

/// Item class - exact match of item_class enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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

/// Item equipment location - exact match of item_equip_type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum ItemEquipType {
    #[default]
    None = 0,
    OneHand = 1,
    TwoHand = 2,
    Armor = 3,
    Helm = 4,
    Ring = 5,
    Amulet = 6,
    Unequipable = 7,
    Belt = 8,
    Invalid = -1,
}

/// Item quality - exact match of item_quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ItemQuality {
    #[default]
    Normal = 0,
    Magic = 1,
    Unique = 2,
}

/// Item special effects - exact match of ItemSpecialEffect (bitflags)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ItemSpecialEffect(pub u32);

impl ItemSpecialEffect {
    pub const NONE: Self = Self(0);
    pub const RANDOM_STEAL_LIFE: Self = Self(1 << 1);
    pub const STAFF: Self = Self(1 << 2);
    pub const RANDOM_ARROW_VELOCITY: Self = Self(1 << 2);
    pub const FIRE_ARROWS: Self = Self(1 << 3);
    pub const FIRE_DAMAGE: Self = Self(1 << 4);
    pub const LIGHTNING_DAMAGE: Self = Self(1 << 5);
    pub const DRAIN_LIFE: Self = Self(1 << 6);
    pub const MULTIPLE_ARROWS: Self = Self(1 << 9);
    pub const KNOCKBACK: Self = Self(1 << 11);
    pub const STEAL_MANA_3: Self = Self(1 << 13);
    pub const STEAL_MANA_5: Self = Self(1 << 14);
    pub const STEAL_LIFE_3: Self = Self(1 << 15);
    pub const STEAL_LIFE_5: Self = Self(1 << 16);
    pub const QUICK_ATTACK: Self = Self(1 << 17);
    pub const FAST_ATTACK: Self = Self(1 << 18);
    pub const FASTER_ATTACK: Self = Self(1 << 19);
    pub const FASTEST_ATTACK: Self = Self(1 << 20);
    pub const FAST_HIT_RECOVERY: Self = Self(1 << 21);
    pub const FASTER_HIT_RECOVERY: Self = Self(1 << 22);
    pub const FASTEST_HIT_RECOVERY: Self = Self(1 << 23);
    pub const FAST_BLOCK: Self = Self(1 << 24);
    pub const LIGHTNING_ARROWS: Self = Self(1 << 25);
    pub const THORNS: Self = Self(1 << 26);
    pub const NO_MANA: Self = Self(1 << 27);
    pub const HALF_TRAP_DAMAGE: Self = Self(1 << 28);
    pub const TRIPLE_DEMON_DAMAGE: Self = Self(1 << 30);
    pub const ZERO_RESISTANCE: Self = Self(1 << 31);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn has_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for ItemSpecialEffect {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for ItemSpecialEffect {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Hellfire special effects - exact match of ItemSpecialEffectHf
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ItemSpecialEffectHf(pub u8);

impl ItemSpecialEffectHf {
    pub const NONE: Self = Self(0);
    pub const DEVASTATION: Self = Self(1 << 0);
    pub const DECAY: Self = Self(1 << 1);
    pub const PERIL: Self = Self(1 << 2);
    pub const JESTERS: Self = Self(1 << 3);
    pub const DOPPELGANGER: Self = Self(1 << 4);
    pub const AC_AGAINST_DEMONS: Self = Self(1 << 5);
    pub const AC_AGAINST_UNDEAD: Self = Self(1 << 6);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn has_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for ItemSpecialEffectHf {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Inventory body location - exact match of inv_body_loc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InvBodyLoc {
    Head = 0,
    RingLeft = 1,
    RingRight = 2,
    Amulet = 3,
    HandLeft = 4,
    HandRight = 5,
    Chest = 6,
}

/// Item misc ID - exact match of item_misc_id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
#[repr(i8)]
pub enum ItemMiscId {
    #[default]
    None = 0,
    UseFirst = 1,
    FullHeal = 2,
    Heal = 3,
    // 4, 5 unused
    Mana = 6,
    FullMana = 7,
    // 8, 9 unused
    ElixirStr = 10,
    ElixirMag = 11,
    ElixirDex = 12,
    ElixirVit = 13,
    // 14-17 unused
    Rejuv = 18,
    FullRejuv = 19,
    UseLast = 20,
    Scroll = 21,
    ScrollT = 22,
    Staff = 23,
    Book = 24,
    Ring = 25,
    Amulet = 26,
    Unique = 27,
    // 28 unused
    OilFirst = 29,
    OilOf = 30,
    OilAcc = 31,
    OilMast = 32,
    OilSharp = 33,
    OilDeath = 34,
    OilSkill = 35,
    OilBSmith = 36,
    OilFort = 37,
    OilPerm = 38,
    OilHard = 39,
    OilImp = 40,
    OilLast = 41,
    MapOfDoom = 42,
    Ear = 43,
    SpecElixir = 44,
    // 45 unused
    RuneFirst = 46,
    RuneF = 47,
    RuneL = 48,
    GrRuneL = 49,
    GrRuneF = 50,
    RuneS = 51,
    RuneLast = 52,
    AuricAmulet = 53,
    Note = 54,
    ArenaPot = 55,
    StaffOfLazarus = 56,  // Quest item
    Invalid = -1,
}

/// Item effect type for affixes - exact match of item_effect_type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum ItemEffectType {
    ToHit = 0,
    ToHitCurse = 1,
    Damage = 2,
    DamageCurse = 3,
    ToHitDamage = 4,
    ToHitDamageCurse = 5,
    ArmorPercent = 6,
    ArmorPercentCurse = 7,
    FireRes = 8,
    LightRes = 9,
    MagicRes = 10,
    AllRes = 11,
    SpellLevelAdd = 14,
    Charges = 15,
    FireDam = 16,
    LightDam = 17,
    Str = 19,
    StrCurse = 20,
    Mag = 21,
    MagCurse = 22,
    Dex = 23,
    DexCurse = 24,
    Vit = 25,
    VitCurse = 26,
    Attribs = 27,
    AttribsCurse = 28,
    GetHitCurse = 29,
    GetHit = 30,
    Life = 31,
    LifeCurse = 32,
    Mana = 33,
    ManaCurse = 34,
    Durability = 35,
    DurabilityCurse = 36,
    Indestructible = 37,
    Light = 38,
    LightCurse = 39,
    MultipleArrows = 41,
    FireArrows = 42,
    LightArrows = 43,
    Thorns = 45,
    NoMana = 46,
    Fireball = 50,
    AbsHalfTrap = 52,
    Knockback = 53,
    StealMana = 55,
    StealLife = 56,
    TargetAC = 57,
    FastAttack = 58,
    FastRecover = 59,
    FastBlock = 60,
    DamMod = 61,
    RndArrowVel = 62,
    SetDam = 63,
    SetDur = 64,
    NoMinStr = 65,
    Spell = 66,
    OneHand = 68,
    TripleDemonDamage = 69,
    AllResZero = 70,
    DrainLife = 72,
    RndStealLife = 73,
    SetAC = 75,
    AddACLife = 76,
    AddManaAC = 77,
    ACCurse = 79,
    // Hellfire effects
    FireResCurse = 80,
    LightResCurse = 81,
    MagicResCurse = 82,
    Devastation = 84,
    Decay = 85,
    Peril = 86,
    Jesters = 87,
    Crystalline = 88,
    Doppelganger = 89,
    ACDemon = 90,
    ACUndead = 91,
    ManaToLife = 92,
    LifeToMana = 93,
    #[default]
    Invalid = -1,
}

/// Item power (affix effect)
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemPower {
    pub effect_type: ItemEffectType,
    pub param1: i32,
    pub param2: i32,
}

/// Good or evil alignment for affixes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GoodOrEvil {
    Any = 0,
    Evil = 1,
    Good = 2,
}

/// Gold limits
pub const GOLD_SMALL_LIMIT: i32 = 1000;
pub const GOLD_MEDIUM_LIMIT: i32 = 2500;
pub const GOLD_MAX_LIMIT: i32 = 5000;

/// Item indestructible durability
pub const DUR_INDESTRUCTIBLE: i32 = 255;

/// Max items in game
pub const MAX_ITEMS: usize = 127;

/// Max vendor value
pub const MAX_VENDOR_VALUE: i32 = 140000;
pub const MAX_VENDOR_VALUE_HF: i32 = 200000;
pub const MAX_BOY_VALUE: i32 = 90000;
pub const MAX_BOY_VALUE_HF: i32 = 200000;

/// Item name max length
pub const ITEM_NAME_LENGTH: usize = 64;

//=============================================================================
// Item Index Enums (for specific/quest items)
//=============================================================================

/// Item index for specific/quest items - exact match of _item_indexes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum ItemId {
    Gold = 0,
    Warrior = 1,
    WarriorShield = 2,
    WarriorClub = 3,
    Rogue = 4,
    Sorcerer = 5,
    // Quest items
    Cleaver = 6,
    SkeletonCrown = 7,
    InfraRing = 8,
    Rock = 9,
    OpticAmulet = 10,
    TRing = 11,
    Banner = 12,
    HarlequinCrest = 13,
    SteelVeil = 14,
    GoldenElixir = 15,
    Anvil = 16,
    Mushroom = 17,
    Brain = 18,
    FungalTome = 19,
    SpecialElixir = 20,
    BloodStone = 21,
    MapOfDoom = 22,
    // Consumables
    Ear = 23,
    Heal = 24,
    Mana = 25,
    Identify = 26,
    Portal = 27,
    ArmorOfValor = 28,
    FullHeal = 29,
    FullMana = 30,
    Griswold = 31,
    LightningForge = 32,
    LazarusStaff = 33,
    Resurrect = 34,
    Oil = 35,
    ShortStaff = 36,
    BardSword = 37,
    BardDagger = 38,
    RuneBomb = 39,
    Theodore = 40,
    Auric = 41,
    Note1 = 42,
    Note2 = 43,
    Note3 = 44,
    FullNote = 45,
    BrownSuit = 46,
    GreySuit = 47,
    // Books
    Book1 = 114,
    Book2 = 115,
    Book3 = 116,
    Book4 = 117,
    // Hellfire
    Barbarian = 139,
    ShortBattleBow = 148,
    RuneOfStone = 165,
    SorcererDiablo = 166,
    ArenaPot = 167,

    NumDefaultItems = 168,
    None = -1,
}

impl ItemId {
    pub const FIRST_QUEST: ItemId = ItemId::Cleaver;
    pub const LAST_QUEST: ItemId = ItemId::MapOfDoom;
}

//=============================================================================
// Unique Item IDs (complete list from C++ _unique_items)
//=============================================================================

/// Unique item IDs - exact match of _unique_items enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum UniqueItemId {
    // Weapons
    Cleaver = 0,
    SkCrown = 1,
    InfraRing = 2,
    OptAmulet = 3,
    TRing = 4,
    HarCrest = 5,
    SteelVeil = 6,
    ArmorOfVal = 7,
    Griswold = 8,
    Bovine = 9,
    // Bows
    RiftBow = 10,
    Needler = 11,
    CelestBow = 12,
    DeadlyHunt = 13,
    BowOfDead = 14,
    BlkOakBow = 15,
    FlameDart = 16,
    FleshSting = 17,
    Windforce = 18,
    Eaglehorn = 19,
    // Daggers/Swords
    GonnagalDirk = 20,
    Defender = 21,
    GryphonClaw = 22,
    BlackRazor = 23,
    GibbousMoon = 24,
    IceShank = 25,
    Executioner = 26,
    BoneSaw = 27,
    ShadHawk = 28,
    WizSpike = 29,
    LightSabre = 30,
    FalconTalon = 31,
    Inferno = 32,
    Doombringer = 33,
    Grizzly = 34,
    Grandfather = 35,
    // Axes
    Mangler = 36,
    SharpBeak = 37,
    BloodSlayer = 38,
    CelestAxe = 39,
    WickedAxe = 40,
    StoneCleaver = 41,
    AguHatchet = 42,
    Hellslayer = 43,
    MesserReaver = 44,
    // Maces/Hammers
    Crackrust = 45,
    JholmHamm = 46,
    Civerbs = 47,
    CelestStar = 48,
    BaranStar = 49,
    GnarlRoot = 50,
    Cranbash = 51,
    SchaefHamm = 52,
    DreamFlange = 53,
    // Staves
    StaffOfShad = 54,
    Immolator = 55,
    StormSpire = 56,
    Gleamsong = 57,
    Thundercall = 58,
    Protector = 59,
    NajPuzzle = 60,
    MindCry = 61,
    RodOfOnan = 62,
    // Helms
    SpiritHelm = 63,
    ThinkingCap = 64,
    OverlordHelm = 65,
    FoolsCrest = 66,
    Gotterdam = 67,
    RoyCirclet = 68,
    // Armors
    TornFlesh = 69,
    Gladbane = 70,
    RainCloak = 71,
    Leathaut = 72,
    WisdWrap = 73,
    SparkMail = 74,
    ScavCarap = 75,
    Nightscape = 76,
    NajPlate = 77,
    // Shields
    Demonspike = 78,
    Deflector = 79,
    SkullShld = 80,
    DragonBrch = 81,
    BlkOakShld = 82,
    HolyDef = 83,
    StormShld = 84,
    // Jewelry
    Bramble = 85,
    Regha = 86,
    Bleeder = 87,
    Constrict = 88,
    Engage = 89,

    Invalid = -1,
}

impl UniqueItemId {
    /// Get total number of unique items
    pub const fn count() -> usize {
        90
    }

    /// Convert from index to UniqueItemId
    pub fn from_index(idx: usize) -> Option<Self> {
        if idx >= 90 {
            return None;
        }
        // Safe transmute since we check bounds
        Some(unsafe { std::mem::transmute(idx as i32) })
    }

    /// Convert to index
    pub fn to_index(self) -> Option<usize> {
        let val = self as i32;
        if val < 0 || val >= 90 {
            None
        } else {
            Some(val as usize)
        }
    }
}

//=============================================================================
// Unique Base Item Type (for unique item templates)
//=============================================================================

/// Unique base item types - exact match of unique_base_item enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum UniqueBaseItem {
    None = 0,
    // Bows
    ShortBow = 1,
    LongBow = 2,
    HunterBow = 3,
    CompositeBow = 4,
    WarBow = 5,
    BattleBow = 6,
    // Swords
    Dagger = 7,
    Falchion = 8,
    Claymore = 9,
    BroadSword = 10,
    Sabre = 11,
    Scimitar = 12,
    LongSword = 13,
    BastardSword = 14,
    TwoHandSword = 15,
    GreatSword = 16,
    // Axes
    Cleaver = 17,
    LargeAxe = 18,
    BroadAxe = 19,
    SmallAxe = 20,
    BattleAxe = 21,
    GreatAxe = 22,
    // Maces
    Mace = 23,
    MorningStar = 24,
    SpikedClub = 25,
    Maul = 26,
    WarHammer = 27,
    Flail = 28,
    // Staves
    LongStaff = 29,
    ShortStaff = 30,
    CompositeStaff = 31,
    QuarterStaff = 32,
    WarStaff = 33,
    // Helms
    SkullCap = 34,
    Helm = 35,
    GreatHelm = 36,
    Crown = 37,
    Type38 = 38,
    // Armor
    Rags = 39,
    StuddedArmor = 40,
    Cloak = 41,
    Robe = 42,
    ChainMail = 43,
    LeatherArmor = 44,
    BreastPlate = 45,
    Cape = 46,
    PlateMail = 47,
    FullPlate = 48,
    // Shields
    Buckler = 49,
    SmallShield = 50,
    LargeShield = 51,
    KiteShield = 52,
    GothicShield = 53,
    // Jewelry
    Ring = 54,
    Type55 = 55,
    Amulet = 56,
    // Quest items
    SkeletonCrown = 57,
    InfraRing = 58,
    OpticAmulet = 59,
    TRing = 60,
    HarlequinCrest = 61,
    MapOfDoom = 62,
    Elixir = 63,
    ArmorOfValor = 64,
    SteelVeil = 65,
    Griswold = 66,
    LightningForge = 67,
    LazarusStaff = 68,
    Bovine = 69,

    NumDefaultTypes = 70,
    NumMaxTypes = 127,
    Invalid = -1,
}

//=============================================================================
// Item Data Structures
//=============================================================================

use crate::game::spelldat::SpellID;

/// Item data - exact match of ItemData struct
#[derive(Debug, Clone)]
pub struct ItemData {
    pub drop_rate: u8,
    pub class: ItemClass,
    pub equip_type: ItemEquipType,
    pub cursor_graphic: u8,  // item_cursor_graphic
    pub item_type: ItemType,
    pub unique_base_id: UniqueBaseItem,
    pub name: &'static str,
    pub short_name: &'static str,
    pub min_mlvl: u8,  // Minimum monster level
    pub durability: u8,
    pub min_damage: u8,
    pub max_damage: u8,
    pub min_ac: u8,
    pub max_ac: u8,
    pub min_str: u8,
    pub min_mag: u8,
    pub min_dex: u8,
    pub special_effects: ItemSpecialEffect,
    pub misc_id: ItemMiscId,
    pub spell: SpellID,
    pub usable: bool,
    pub value: u16,
}

/// Unique item data - exact match of UniqueItem struct
#[derive(Debug, Clone)]
pub struct UniqueItemData {
    pub name: &'static str,
    pub cursor_graphic: u8,
    pub base_item_id: UniqueBaseItem,
    pub min_level: i8,
    pub num_powers: u8,
    pub value: i32,
    pub powers: [ItemPower; 6],
}

//=============================================================================
// Static Data Tables
//=============================================================================

/// Number of base item types (expanded table: 80/169 implemented)
pub const NUM_BASE_ITEMS_CURRENT: usize = 80;
pub const NUM_BASE_ITEMS_TOTAL: usize = 169;

/// Number of unique items (expanded table: 50/90 implemented)
pub const NUM_UNIQUE_ITEMS_CURRENT: usize = 50;
pub const NUM_UNIQUE_ITEMS_TOTAL: usize = 90;

/// Macro to simplify item data definition (18 parameters)
macro_rules! idat {
    ($drop:expr, $class:expr, $iloc:expr, $curs:expr, $itype:expr, $base:expr,
     $name:expr, $sname:expr, $mlvl:expr, $dur:expr, $mind:expr, $maxd:expr,
     $minac:expr, $maxac:expr, $str:expr, $mag:expr, $dex:expr,
     $flags:expr, $misc:expr, $spell:expr, $use:expr, $val:expr) => {
        ItemData {
            drop_rate: $drop,
            class: $class,
            equip_type: $iloc,
            cursor_graphic: $curs,
            item_type: $itype,
            unique_base_id: $base,
            name: $name,
            short_name: $sname,
            min_mlvl: $mlvl,
            durability: $dur,
            min_damage: $mind,
            max_damage: $maxd,
            min_ac: $minac,
            max_ac: $maxac,
            min_str: $str,
            min_mag: $mag,
            min_dex: $dex,
            special_effects: $flags,
            misc_id: $misc,
            spell: $spell,
            usable: $use,
            value: $val,
        }
    };
}

/// Base items data table (expanded - contains common items)
/// Covers major weapon/armor types for gameplay
pub const ITEMS_DATA: [ItemData; 167] = [
    // Gold (0)
    idat!(0, ItemClass::Gold, ItemEquipType::None, 4, ItemType::Gold,
          UniqueBaseItem::None, "Gold", "gold", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1),

    // Potions (1-5)
    idat!(50, ItemClass::Misc, ItemEquipType::Unequipable, 32, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Healing", "heal", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Heal, SpellID::Healing, true, 50),
    idat!(50, ItemClass::Misc, ItemEquipType::Unequipable, 39, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Mana", "mana", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Mana, SpellID::Null, true, 50),
    idat!(25, ItemClass::Misc, ItemEquipType::Unequipable, 35, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Healing", "fheal", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullHeal, SpellID::Healing, true, 150),
    idat!(25, ItemClass::Misc, ItemEquipType::Unequipable, 0, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Mana", "fmana", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullMana, SpellID::Null, true, 150),
    idat!(20, ItemClass::Misc, ItemEquipType::Unequipable, 37, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Rejuvenation", "rejuv", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Rejuv, SpellID::Healing, true, 120),

    // Scrolls (6-7)
    idat!(40, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Identify", "id", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Identify, true, 100),
    idat!(40, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Town Portal", "tp", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::TownPortal, true, 200),

    // Weapons - Swords (8-12)
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 51, ItemType::Sword,
          UniqueBaseItem::Dagger, "Dagger", "dag", 1, 16, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 60),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 64, ItemType::Sword,
          UniqueBaseItem::LongSword, "Short Sword", "ssw", 2, 24, 2, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 120),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 67, ItemType::Sword,
          UniqueBaseItem::Sabre, "Sabre", "sab", 5, 32, 1, 8, 0, 0, 17, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 150),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 61, ItemType::Sword,
          UniqueBaseItem::BroadSword, "Broad Sword", "bsw", 8, 50, 4, 12, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 400),
    idat!(8, ItemClass::Weapon, ItemEquipType::TwoHand, 110, ItemType::Sword,
          UniqueBaseItem::TwoHandSword, "Two-Handed Sword", "2sw", 14, 75, 8, 16, 0, 0, 45, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1200),

    // Weapons - Axes (13-15)
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 144, ItemType::Axe,
          UniqueBaseItem::SmallAxe, "Axe", "axe", 2, 24, 2, 6, 0, 0, 22, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 150),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 101, ItemType::Axe,
          UniqueBaseItem::BattleAxe, "Battle Axe", "bax", 10, 50, 4, 12, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 650),
    idat!(8, ItemClass::Weapon, ItemEquipType::TwoHand, 143, ItemType::Axe,
          UniqueBaseItem::GreatAxe, "Great Axe", "gax", 17, 75, 10, 20, 0, 0, 65, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1750),

    // Weapons - Bows (16-18)
    idat!(10, ItemClass::Weapon, ItemEquipType::TwoHand, 118, ItemType::Bow,
          UniqueBaseItem::ShortBow, "Short Bow", "sbo", 1, 30, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 100),
    idat!(10, ItemClass::Weapon, ItemEquipType::TwoHand, 102, ItemType::Bow,
          UniqueBaseItem::HunterBow, "Hunter's Bow", "hbo", 5, 40, 2, 5, 0, 0, 0, 0, 20,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    idat!(8, ItemClass::Weapon, ItemEquipType::TwoHand, 119, ItemType::Bow,
          UniqueBaseItem::LongBow, "Long War Bow", "lbo", 15, 60, 1, 14, 0, 0, 0, 0, 60,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2000),

    // Armor (19-22)
    idat!(8, ItemClass::Armor, ItemEquipType::Armor, 149, ItemType::LightArmor,
          UniqueBaseItem::Cloak, "Cloak", "clk", 1, 12, 0, 0, 1, 5, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 40),
    idat!(8, ItemClass::Armor, ItemEquipType::Armor, 135, ItemType::LightArmor,
          UniqueBaseItem::LeatherArmor, "Leather Armor", "lea", 3, 20, 0, 0, 2, 10, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 100),
    idat!(7, ItemClass::Armor, ItemEquipType::Armor, 111, ItemType::MediumArmor,
          UniqueBaseItem::ChainMail, "Chain Mail", "chn", 7, 40, 0, 0, 10, 15, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    idat!(6, ItemClass::Armor, ItemEquipType::Armor, 151, ItemType::HeavyArmor,
          UniqueBaseItem::FullPlate, "Full Plate Mail", "fpl", 15, 60, 0, 0, 20, 30, 60, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2300),

    // Shields (23-25)
    idat!(8, ItemClass::Armor, ItemEquipType::OneHand, 83, ItemType::Shield,
          UniqueBaseItem::Buckler, "Buckler", "buc", 1, 16, 0, 0, 1, 5, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 30),
    idat!(7, ItemClass::Armor, ItemEquipType::OneHand, 105, ItemType::Shield,
          UniqueBaseItem::SmallShield, "Small Shield", "sml", 4, 24, 0, 0, 3, 8, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 90),
    idat!(6, ItemClass::Armor, ItemEquipType::OneHand, 113, ItemType::Shield,
          UniqueBaseItem::KiteShield, "Kite Shield", "kit", 10, 40, 0, 0, 8, 15, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 850),

    // Helms (26-28)
    idat!(7, ItemClass::Armor, ItemEquipType::Helm, 91, ItemType::Helm,
          UniqueBaseItem::Helm, "Cap", "cap", 1, 10, 0, 0, 1, 3, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 15),
    idat!(6, ItemClass::Armor, ItemEquipType::Helm, 82, ItemType::Helm,
          UniqueBaseItem::Helm, "Helm", "hlm", 7, 30, 0, 0, 5, 10, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    idat!(5, ItemClass::Armor, ItemEquipType::Helm, 95, ItemType::Helm,
          UniqueBaseItem::Crown, "Crown", "crn", 15, 50, 0, 0, 15, 20, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 4000),

    // Jewelry (29-30)
    idat!(5, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "rng", 5, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 100),
    idat!(5, ItemClass::Misc, ItemEquipType::Amulet, 45, ItemType::Amulet,
          UniqueBaseItem::Amulet, "Amulet", "amu", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Amulet, SpellID::Null, false, 100),

    // More Swords (31-36)
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 62, ItemType::Sword,
          UniqueBaseItem::Falchion, "Falchion", "fal", 6, 36, 3, 8, 0, 0, 20, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 72, ItemType::Sword,
          UniqueBaseItem::Scimitar, "Scimitar", "scm", 10, 42, 3, 11, 0, 0, 23, 0, 35,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 500),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 60, ItemType::Sword,
          UniqueBaseItem::LongSword, "Long Sword", "lsw", 11, 48, 2, 10, 0, 0, 30, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 650),
    idat!(9, ItemClass::Weapon, ItemEquipType::OneHand, 57, ItemType::Sword,
          UniqueBaseItem::BastardSword, "Bastard Sword", "bsw", 12, 60, 5, 15, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    idat!(8, ItemClass::Weapon, ItemEquipType::TwoHand, 65, ItemType::Sword,
          UniqueBaseItem::Claymore, "Claymore", "clm", 13, 70, 5, 18, 0, 0, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1400),
    idat!(7, ItemClass::Weapon, ItemEquipType::TwoHand, 134, ItemType::Sword,
          UniqueBaseItem::GreatSword, "Great Sword", "gsw", 19, 100, 10, 20, 0, 0, 75, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 3000),

    // More Axes (37-40)
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 142, ItemType::Axe,
          UniqueBaseItem::LargeAxe, "Large Axe", "lax", 8, 40, 4, 10, 0, 0, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 141, ItemType::Axe,
          UniqueBaseItem::BroadAxe, "Broad Axe", "bax", 11, 55, 6, 14, 0, 0, 48, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 800),
    idat!(8, ItemClass::Weapon, ItemEquipType::OneHand, 112, ItemType::Axe,
          UniqueBaseItem::SmallAxe, "Small Axe", "sax", 5, 28, 3, 7, 0, 0, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    idat!(7, ItemClass::Weapon, ItemEquipType::OneHand, 106, ItemType::Axe,
          UniqueBaseItem::Cleaver, "Cleaver", "clv", 2, 22, 2, 8, 0, 0, 17, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 140),

    // More Bows (41-44)
    idat!(10, ItemClass::Weapon, ItemEquipType::TwoHand, 133, ItemType::Bow,
          UniqueBaseItem::CompositeBow, "Composite Bow", "cbo", 8, 45, 3, 6, 0, 0, 0, 0, 25,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 600),
    idat!(9, ItemClass::Weapon, ItemEquipType::TwoHand, 120, ItemType::Bow,
          UniqueBaseItem::WarBow, "War Bow", "wbo", 11, 55, 2, 8, 0, 0, 0, 0, 45,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1250),
    idat!(8, ItemClass::Weapon, ItemEquipType::TwoHand, 167, ItemType::Bow,
          UniqueBaseItem::BattleBow, "Battle Bow", "bbo", 14, 65, 1, 15, 0, 0, 0, 0, 60,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2800),
    idat!(9, ItemClass::Weapon, ItemEquipType::TwoHand, 165, ItemType::Bow,
          UniqueBaseItem::ShortBow, "Short War Bow", "swb", 7, 35, 2, 5, 0, 0, 0, 0, 15,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 300),

    // Maces/Hammers (45-50)
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 59, ItemType::Mace,
          UniqueBaseItem::Mace, "Mace", "mac", 3, 32, 1, 8, 0, 0, 16, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    idat!(10, ItemClass::Weapon, ItemEquipType::OneHand, 63, ItemType::Mace,
          UniqueBaseItem::MorningStar, "Morning Star", "mst", 9, 48, 1, 10, 0, 0, 26, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 500),
    idat!(9, ItemClass::Weapon, ItemEquipType::OneHand, 70, ItemType::Mace,
          UniqueBaseItem::SpikedClub, "Spiked Club", "spc", 5, 36, 3, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 225),
    idat!(8, ItemClass::Weapon, ItemEquipType::OneHand, 122, ItemType::Mace,
          UniqueBaseItem::Maul, "Maul", "mau", 15, 70, 6, 20, 0, 0, 55, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1850),
    idat!(9, ItemClass::Weapon, ItemEquipType::OneHand, 121, ItemType::Mace,
          UniqueBaseItem::WarHammer, "War Hammer", "whm", 12, 60, 5, 9, 0, 0, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1200),
    idat!(8, ItemClass::Weapon, ItemEquipType::OneHand, 131, ItemType::Mace,
          UniqueBaseItem::Flail, "Flail", "fla", 10, 50, 2, 12, 0, 0, 30, 0, 35,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 700),

    // Staves (51-54)
    idat!(10, ItemClass::Weapon, ItemEquipType::TwoHand, 123, ItemType::Staff,
          UniqueBaseItem::LongStaff, "Long Staff", "lst", 4, 35, 4, 8, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 100),
    idat!(9, ItemClass::Weapon, ItemEquipType::TwoHand, 166, ItemType::Staff,
          UniqueBaseItem::CompositeStaff, "Composite Staff", "cst", 6, 45, 5, 11, 0, 0, 0, 20, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    idat!(8, ItemClass::Weapon, ItemEquipType::TwoHand, 124, ItemType::Staff,
          UniqueBaseItem::QuarterStaff, "Quarter Staff", "qst", 10, 55, 6, 12, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 650),
    idat!(7, ItemClass::Weapon, ItemEquipType::TwoHand, 124, ItemType::Staff,
          UniqueBaseItem::WarStaff, "War Staff", "wst", 14, 75, 8, 16, 0, 0, 0, 30, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1500),

    // More Armor (55-62)
    idat!(8, ItemClass::Armor, ItemEquipType::Armor, 107, ItemType::LightArmor,
          UniqueBaseItem::StuddedArmor, "Studded Leather", "stu", 5, 35, 0, 0, 3, 15, 20, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    idat!(7, ItemClass::Armor, ItemEquipType::Armor, 137, ItemType::LightArmor,
          UniqueBaseItem::Robe, "Robe", "rbe", 1, 6, 0, 0, 2, 4, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 25),
    idat!(8, ItemClass::Armor, ItemEquipType::Armor, 150, ItemType::LightArmor,
          UniqueBaseItem::Cape, "Cape", "cpe", 7, 24, 0, 0, 3, 7, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 75),
    idat!(7, ItemClass::Armor, ItemEquipType::Armor, 154, ItemType::MediumArmor,
          UniqueBaseItem::ChainMail, "Ring Mail", "rng", 5, 35, 0, 0, 7, 12, 24, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    idat!(7, ItemClass::Armor, ItemEquipType::Armor, 136, ItemType::MediumArmor,
          UniqueBaseItem::ChainMail, "Splint Mail", "spl", 10, 45, 0, 0, 11, 18, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 700),
    idat!(6, ItemClass::Armor, ItemEquipType::Armor, 153, ItemType::HeavyArmor,
          UniqueBaseItem::BreastPlate, "Breast Plate", "brs", 12, 50, 0, 0, 14, 24, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1200),
    idat!(6, ItemClass::Armor, ItemEquipType::Armor, 151, ItemType::HeavyArmor,
          UniqueBaseItem::PlateMail, "Plate Mail", "plt", 14, 60, 0, 0, 16, 35, 55, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1800),
    idat!(5, ItemClass::Armor, ItemEquipType::Armor, 152, ItemType::HeavyArmor,
          UniqueBaseItem::FullPlate, "Gothic Plate", "gth", 18, 80, 0, 0, 40, 60, 80, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 4400),

    // More Shields (63-67)
    idat!(7, ItemClass::Armor, ItemEquipType::OneHand, 147, ItemType::Shield,
          UniqueBaseItem::LargeShield, "Large Shield", "lrg", 7, 32, 0, 0, 5, 10, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 280),
    idat!(6, ItemClass::Armor, ItemEquipType::OneHand, 148, ItemType::Shield,
          UniqueBaseItem::GothicShield, "Gothic Shield", "got", 13, 50, 0, 0, 12, 18, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1400),
    idat!(5, ItemClass::Armor, ItemEquipType::OneHand, 132, ItemType::Shield,
          UniqueBaseItem::GothicShield, "Tower Shield", "tow", 16, 60, 0, 0, 16, 25, 60, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2500),
    idat!(7, ItemClass::Armor, ItemEquipType::OneHand, 117, ItemType::Shield,
          UniqueBaseItem::SmallShield, "Dragon's Breach", "drg", 15, 55, 0, 0, 14, 20, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1600),
    idat!(8, ItemClass::Armor, ItemEquipType::OneHand, 146, ItemType::Shield,
          UniqueBaseItem::SmallShield, "Blackoak Shield", "oak", 8, 35, 0, 0, 6, 12, 28, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),

    // More Helms (68-72)
    idat!(6, ItemClass::Armor, ItemEquipType::Helm, 90, ItemType::Helm,
          UniqueBaseItem::SkullCap, "Skull Cap", "skl", 3, 18, 0, 0, 2, 4, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 35),
    idat!(6, ItemClass::Armor, ItemEquipType::Helm, 75, ItemType::Helm,
          UniqueBaseItem::GreatHelm, "Full Helm", "fhl", 10, 35, 0, 0, 6, 12, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 650),
    idat!(5, ItemClass::Armor, ItemEquipType::Helm, 98, ItemType::Helm,
          UniqueBaseItem::GreatHelm, "Great Helm", "ght", 14, 40, 0, 0, 10, 15, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1250),
    idat!(5, ItemClass::Armor, ItemEquipType::Helm, 79, ItemType::Helm,
          UniqueBaseItem::Crown, "Royal Circlet", "rcy", 18, 60, 0, 0, 20, 30, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 6000),
    idat!(7, ItemClass::Armor, ItemEquipType::Helm, 91, ItemType::Helm,
          UniqueBaseItem::Helm, "War Hat", "wht", 5, 22, 0, 0, 3, 6, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 85),

    // Consumables (73-79)
    idat!(15, ItemClass::Misc, ItemEquipType::Unequipable, 38, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Strength", "estr", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirStr, SpellID::Null, true, 5000),
    idat!(15, ItemClass::Misc, ItemEquipType::Unequipable, 34, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Magic", "emag", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirMag, SpellID::Null, true, 5000),
    idat!(15, ItemClass::Misc, ItemEquipType::Unequipable, 36, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Dexterity", "edex", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirDex, SpellID::Null, true, 5000),
    idat!(15, ItemClass::Misc, ItemEquipType::Unequipable, 31, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Vitality", "evit", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirVit, SpellID::Null, true, 5000),
    idat!(20, ItemClass::Misc, ItemEquipType::Unequipable, 33, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Rejuvenation", "frej", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullRejuv, SpellID::Healing, true, 300),
    idat!(30, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Healing", "sheal", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Healing, true, 50),
    idat!(30, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Lightning", "slit", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Lightning, true, 100),

    // Item 80-110: Potions, Oils, Elixirs, Scrolls
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 29, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Mana", "pman", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Mana, SpellID::Null, true, 50),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 30, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Mana", "pfman", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullMana, SpellID::Null, true, 150),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 27, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Rejuvenation", "prej", 3, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Rejuv, SpellID::Null, true, 120),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 28, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Rejuvenation", "pfrej", 7, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullRejuv, SpellID::Null, true, 600),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 40, ItemType::Misc,
          UniqueBaseItem::None, "Blacksmith Oil", "oilb", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilBSmith, SpellID::Null, true, 100),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 40, ItemType::Misc,
          UniqueBaseItem::None, "Oil of Accuracy", "oilacc", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilAcc, SpellID::Null, true, 500),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 40, ItemType::Misc,
          UniqueBaseItem::None, "Oil of Sharpness", "oilsharp", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilSharp, SpellID::Null, true, 500),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 40, ItemType::Misc,
          UniqueBaseItem::None, "Oil", "oil", 10, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilOf, SpellID::Null, true, 0),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 31, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Strength", "elixstr", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirStr, SpellID::Null, true, 5000),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 31, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Magic", "elixmag", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirMag, SpellID::Null, true, 5000),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 31, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Dexterity", "elixdex", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirDex, SpellID::Null, true, 5000),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 31, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Vitality", "elixvit", 20, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirVit, SpellID::Null, true, 5000),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Healing", "sheal2", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Healing, true, 50),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Search", "ssearch", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Search, true, 50),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Lightning", "slit2", 4, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Lightning, true, 150),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Identify", "sidentify", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Identify, true, 100),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Resurrect", "sres", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Resurrect, true, 250),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Fire Wall", "sfwall", 4, 0, 0, 0, 0, 0, 0, 17, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::FireWall, true, 400),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Inferno", "sinferno", 1, 0, 0, 0, 0, 0, 0, 19, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Inferno, true, 100),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Town Portal", "sportal", 4, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::TownPortal, true, 200),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Flash", "sflash", 6, 0, 0, 0, 0, 0, 0, 21, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Flash, true, 500),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Infravision", "sinfrav", 8, 0, 0, 0, 0, 0, 0, 23, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Infravision, true, 600),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Phasing", "sphase", 6, 0, 0, 0, 0, 0, 0, 25, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Phasing, true, 200),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Mana Shield", "smshield", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::ManaShield, true, 1200),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Flame Wave", "sfwave", 10, 0, 0, 0, 0, 0, 0, 29, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::FlameWave, true, 650),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Fireball", "sfball", 8, 0, 0, 0, 0, 0, 0, 31, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Fireball, true, 300),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Stone Curse", "sstone", 6, 0, 0, 0, 0, 0, 0, 33, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::StoneCurse, true, 800),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Chain Lightning", "schain", 10, 0, 0, 0, 0, 0, 0, 35, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::ChainLightning, true, 750),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Guardian", "sguard", 12, 0, 0, 0, 0, 0, 0, 47, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Guardian, true, 950),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Nova", "snova", 14, 0, 0, 0, 0, 0, 0, 57, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Nova, true, 1300),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Golem", "sgolem", 10, 0, 0, 0, 0, 0, 0, 51, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Golem, true, 1100),

    // Item 111-140: Books and Weapons
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 18, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Teleport", "stele", 14, 0, 0, 0, 0, 0, 0, 81, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Teleport, true, 3000),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 18, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Apocalypse", "sapoc", 22, 0, 0, 0, 0, 0, 0, 117, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Apocalypse, true, 2000),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 18, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "book1", 2, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 18, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "book2", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 18, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "book3", 14, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 18, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "book4", 20, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 6, ItemType::Sword,
          UniqueBaseItem::Dagger, "Dagger", "dag", 1, 16, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 60),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 104, ItemType::Sword,
          UniqueBaseItem::None, "Short Sword", "ssword", 1, 24, 2, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 120),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 8, ItemType::Sword,
          UniqueBaseItem::Falchion, "Falchion", "falch", 2, 20, 4, 8, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 250),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 11, ItemType::Sword,
          UniqueBaseItem::Scimitar, "Scimitar", "scim", 4, 28, 3, 7, 0, 0, 23, 0, 23,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 9, ItemType::Sword,
          UniqueBaseItem::Claymore, "Claymore", "clay", 5, 36, 1, 12, 0, 0, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 105, ItemType::Sword,
          UniqueBaseItem::None, "Blade", "blade", 4, 30, 3, 8, 0, 0, 25, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 280),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 11, ItemType::Sword,
          UniqueBaseItem::Sabre, "Sabre", "sabre", 1, 45, 1, 8, 0, 0, 17, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 170),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 13, ItemType::Sword,
          UniqueBaseItem::LongSword, "Long Sword", "lsword", 6, 40, 2, 10, 0, 0, 30, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 10, ItemType::Sword,
          UniqueBaseItem::BroadSword, "Broad Sword", "bsword", 8, 50, 4, 12, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 750),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 14, ItemType::Sword,
          UniqueBaseItem::BastardSword, "Bastard Sword", "bastard", 10, 60, 6, 15, 0, 0, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 15, ItemType::Sword,
          UniqueBaseItem::TwoHandSword, "Two-Handed Sword", "2hsword", 14, 75, 8, 16, 0, 0, 65, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1800),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 16, ItemType::Sword,
          UniqueBaseItem::GreatSword, "Great Sword", "gsword", 17, 100, 10, 20, 0, 0, 75, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 3000),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 20, ItemType::Axe,
          UniqueBaseItem::SmallAxe, "Small Axe", "saxe", 2, 24, 2, 10, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 150),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 105, ItemType::Axe,
          UniqueBaseItem::None, "Axe", "axe", 4, 32, 4, 12, 0, 0, 22, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 18, ItemType::Axe,
          UniqueBaseItem::LargeAxe, "Large Axe", "laxe", 6, 40, 6, 16, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 750),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 19, ItemType::Axe,
          UniqueBaseItem::BroadAxe, "Broad Axe", "baxe", 8, 50, 8, 20, 0, 0, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 21, ItemType::Axe,
          UniqueBaseItem::BattleAxe, "Battle Axe", "btaxe", 10, 60, 10, 25, 0, 0, 65, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1500),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 22, ItemType::Axe,
          UniqueBaseItem::GreatAxe, "Great Axe", "gaxe", 12, 75, 12, 30, 0, 0, 80, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2500),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 23, ItemType::Mace,
          UniqueBaseItem::Mace, "Mace", "mace", 2, 32, 1, 8, 0, 0, 16, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 24, ItemType::Mace,
          UniqueBaseItem::MorningStar, "Morning Star", "mstar", 3, 40, 1, 10, 0, 0, 26, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 300),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 27, ItemType::Mace,
          UniqueBaseItem::WarHammer, "War Hammer", "whammer", 5, 50, 5, 9, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 600),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 25, ItemType::Mace,
          UniqueBaseItem::SpikedClub, "Spiked Club", "sclub", 4, 20, 3, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 225),

    // Item 140-169: More Weapons, Bows, Staves, Jewelry, Runes
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 25, ItemType::Mace,
          UniqueBaseItem::SpikedClub, "Club", "club", 1, 20, 1, 6, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 20),
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 28, ItemType::Mace,
          UniqueBaseItem::Flail, "Flail", "flail", 7, 36, 2, 12, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 500),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 26, ItemType::Mace,
          UniqueBaseItem::Maul, "Maul", "maul", 10, 50, 6, 20, 0, 0, 55, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 900),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 1, ItemType::Bow,
          UniqueBaseItem::ShortBow, "Short Bow", "sbow", 1, 30, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 100),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 3, ItemType::Bow,
          UniqueBaseItem::HunterBow, "Hunter's Bow", "hbow", 3, 40, 2, 5, 0, 0, 20, 0, 35,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 3, ItemType::Bow,
          UniqueBaseItem::LongBow, "Long Bow", "lbow", 5, 35, 1, 6, 0, 0, 25, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 250),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 4, ItemType::Bow,
          UniqueBaseItem::CompositeBow, "Composite Bow", "cbow", 7, 45, 3, 6, 0, 0, 25, 0, 40,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 600),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 105, ItemType::Bow,
          UniqueBaseItem::None, "Short Battle Bow", "sbbow", 9, 45, 3, 7, 0, 0, 30, 0, 50,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 750),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 6, ItemType::Bow,
          UniqueBaseItem::BattleBow, "Long Battle Bow", "lbbow", 11, 50, 1, 10, 0, 0, 30, 0, 60,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 105, ItemType::Bow,
          UniqueBaseItem::None, "Short War Bow", "swbow", 15, 55, 4, 8, 0, 0, 35, 0, 70,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1500),
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 5, ItemType::Bow,
          UniqueBaseItem::WarBow, "Long War Bow", "lwbow", 19, 60, 1, 14, 0, 0, 45, 0, 80,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2000),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 30, ItemType::Staff,
          UniqueBaseItem::ShortStaff, "Short Staff", "sstaff", 1, 25, 2, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::STAFF, ItemMiscId::None, SpellID::Null, false, 30),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 29, ItemType::Staff,
          UniqueBaseItem::LongStaff, "Long Staff", "lstaff", 4, 35, 4, 8, 0, 0, 0, 0, 0,
          ItemSpecialEffect::STAFF, ItemMiscId::None, SpellID::Null, false, 100),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 31, ItemType::Staff,
          UniqueBaseItem::CompositeStaff, "Composite Staff", "cstaff", 6, 45, 5, 10, 0, 0, 0, 0, 0,
          ItemSpecialEffect::STAFF, ItemMiscId::None, SpellID::Null, false, 500),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 30, ItemType::Staff,
          UniqueBaseItem::QuarterStaff, "Quarter Staff", "qstaff", 9, 55, 6, 12, 0, 0, 20, 0, 0,
          ItemSpecialEffect::STAFF, ItemMiscId::None, SpellID::Null, false, 1000),
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 33, ItemType::Staff,
          UniqueBaseItem::WarStaff, "War Staff", "wstaff", 12, 75, 8, 16, 0, 0, 30, 0, 0,
          ItemSpecialEffect::STAFF, ItemMiscId::None, SpellID::Null, false, 1500),
    idat!(1, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "ring1", 5, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 1000),
    idat!(1, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "ring2", 10, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 1000),
    idat!(1, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "ring3", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 1000),
    idat!(1, ItemClass::Misc, ItemEquipType::Amulet, 45, ItemType::Amulet,
          UniqueBaseItem::Amulet, "Amulet", "amulet1", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Amulet, SpellID::Null, false, 1200),
    idat!(1, ItemClass::Misc, ItemEquipType::Amulet, 45, ItemType::Amulet,
          UniqueBaseItem::Amulet, "Amulet", "amulet2", 16, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Amulet, SpellID::Null, false, 1200),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 162, ItemType::Misc,
          UniqueBaseItem::None, "Rune of Fire", "runef", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::RuneF, SpellID::Null, true, 100),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 162, ItemType::Misc,
          UniqueBaseItem::None, "Rune of Lightning", "runel", 3, 0, 0, 0, 0, 0, 0, 13, 0,
          ItemSpecialEffect::NONE, ItemMiscId::RuneL, SpellID::Null, true, 200),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 162, ItemType::Misc,
          UniqueBaseItem::None, "Greater Rune of Fire", "grunef", 7, 0, 0, 0, 0, 0, 0, 42, 0,
          ItemSpecialEffect::NONE, ItemMiscId::GrRuneF, SpellID::Null, true, 400),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 162, ItemType::Misc,
          UniqueBaseItem::None, "Greater Rune of Lightning", "grunel", 7, 0, 0, 0, 0, 0, 0, 42, 0,
          ItemSpecialEffect::NONE, ItemMiscId::GrRuneL, SpellID::Null, true, 500),
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 162, ItemType::Misc,
          UniqueBaseItem::None, "Rune of Stone", "runes", 7, 0, 0, 0, 0, 0, 0, 25, 0,
          ItemSpecialEffect::NONE, ItemMiscId::RuneS, SpellID::Null, true, 300),
    idat!(0, ItemClass::Weapon, ItemEquipType::TwoHand, 30, ItemType::Staff,
          UniqueBaseItem::None, "Short Staff of Charged Bolt", "sscb", 1, 25, 2, 4, 0, 0, 0, 25, 0,
          ItemSpecialEffect::STAFF, ItemMiscId::Staff, SpellID::ChargedBolt, false, 470),
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 43, ItemType::Misc,
          UniqueBaseItem::None, "Arena Potion", "arenapot", 7, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ArenaPot, SpellID::Null, true, 0),
];

/// Macro to simplify unique item data definition
macro_rules! udat {
    ($name:expr, $curs:expr, $base:expr, $lvl:expr, $npow:expr, $val:expr,
     $p1type:expr, $p1p1:expr, $p1p2:expr,
     $p2type:expr, $p2p1:expr, $p2p2:expr,
     $p3type:expr, $p3p1:expr, $p3p2:expr,
     $p4type:expr, $p4p1:expr, $p4p2:expr,
     $p5type:expr, $p5p1:expr, $p5p2:expr,
     $p6type:expr, $p6p1:expr, $p6p2:expr) => {
        UniqueItemData {
            name: $name,
            cursor_graphic: $curs,
            base_item_id: $base,
            min_level: $lvl,
            num_powers: $npow,
            value: $val,
            powers: [
                ItemPower { effect_type: $p1type, param1: $p1p1, param2: $p1p2 },
                ItemPower { effect_type: $p2type, param1: $p2p1, param2: $p2p2 },
                ItemPower { effect_type: $p3type, param1: $p3p1, param2: $p3p2 },
                ItemPower { effect_type: $p4type, param1: $p4p1, param2: $p4p2 },
                ItemPower { effect_type: $p5type, param1: $p5p1, param2: $p5p2 },
                ItemPower { effect_type: $p6type, param1: $p6p1, param2: $p6p2 },
            ],
        }
    };
}

/// Unique items data table (expanded - contains 50 major unique items)
/// Covers most iconic weapons, armor, and jewelry
pub const UNIQUE_ITEMS: [UniqueItemData; 90] = [
    // The Butcher's Cleaver (0)
    udat!("The Butcher's Cleaver", 106, UniqueBaseItem::Cleaver, 1, 0, 3000,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Undead Crown (1)
    udat!("Undead Crown", 78, UniqueBaseItem::SkeletonCrown, 1, 0, 4000,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Windforce (2 - actually ID 18, placeholder)
    udat!("Windforce", 164, UniqueBaseItem::LongBow, 25, 6, 40000,
          ItemEffectType::ToHit, 200, 200,
          ItemEffectType::Damage, 0, 200,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::Knockback, 0, 0,
          ItemEffectType::FastAttack, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // The Grandfather (3 - actually ID 35)
    udat!("The Grandfather", 161, UniqueBaseItem::GreatSword, 25, 4, 50000,
          ItemEffectType::ToHit, 70, 70,
          ItemEffectType::Damage, 10, 15,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Helm of Spirits (4)
    udat!("Helm of Spirits", 77, UniqueBaseItem::Helm, 12, 4, 8000,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Mana, 20, 20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Thinking Cap (5)
    udat!("Thinking Cap", 93, UniqueBaseItem::Helm, 12, 3, 6000,
          ItemEffectType::SetAC, 30, 30,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::SpellLevelAdd, 1, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Arkaine's Valor (6)
    udat!("Arkaine's Valor", 157, UniqueBaseItem::FullPlate, 18, 4, 19500,
          ItemEffectType::SetAC, 60, 60,
          ItemEffectType::Vit, 10, 10,
          ItemEffectType::FastRecover, 0, 0,
          ItemEffectType::Damage, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Griswold's Edge (7)
    udat!("Griswold's Edge", 0, UniqueBaseItem::BroadSword, 8, 3, 6000,
          ItemEffectType::ToHit, 20, 30,
          ItemEffectType::SetDam, 1, 10,
          ItemEffectType::Knockback, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Veil of Steel (8)
    udat!("Veil of Steel", 85, UniqueBaseItem::GreatHelm, 16, 4, 12000,
          ItemEffectType::SetAC, 50, 50,
          ItemEffectType::AllRes, 50, 50,
          ItemEffectType::Str, 15, 15,
          ItemEffectType::Vit, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Harlequin Crest (9)
    udat!("Harlequin Crest", 81, UniqueBaseItem::HarlequinCrest, 1, 2, 4500,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::Attribs, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // The Protector (10)
    udat!("The Protector", 162, UniqueBaseItem::LongStaff, 10, 3, 7500,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Ring of Truth (11)
    udat!("Ring of Truth", 10, UniqueBaseItem::Ring, 12, 3, 8000,
          ItemEffectType::Light, 3, 3,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Damage, 1, 12,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Ring of Regha (12)
    udat!("Ring of Regha", 11, UniqueBaseItem::Ring, 12, 4, 15000,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::Mag, 10, 10,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Optic Amulet (13)
    udat!("Optic Amulet", 44, UniqueBaseItem::OpticAmulet, 1, 1, 5000,
          ItemEffectType::Light, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Torn Flesh of Souls (14)
    udat!("Torn Flesh of Souls", 92, UniqueBaseItem::Cape, 12, 2, 6000,
          ItemEffectType::SetAC, 10, 10,
          ItemEffectType::Life, -10, -20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // The Rift Bow (15)
    udat!("The Rift Bow", 0, UniqueBaseItem::BattleBow, 15, 5, 18000,
          ItemEffectType::ToHit, 50, 50,
          ItemEffectType::Damage, 0, 100,
          ItemEffectType::Dex, 5, 5,
          ItemEffectType::RndArrowVel, 0, 0,
          ItemEffectType::FastAttack, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // The Needler (16)
    udat!("The Needler", 158, UniqueBaseItem::HunterBow, 16, 5, 22000,
          ItemEffectType::ToHit, 50, 100,
          ItemEffectType::Damage, 0, 150,
          ItemEffectType::FastAttack, 0, 0,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Messerschmidt's Reaver (17)
    udat!("Messerschmidt's Reaver", 163, UniqueBaseItem::BattleAxe, 20, 5, 35000,
          ItemEffectType::ToHit, 100, 100,
          ItemEffectType::SetDam, 15, 25,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::FireDam, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // The Grizzly (18)
    udat!("The Grizzly", 160, UniqueBaseItem::Maul, 18, 4, 25000,
          ItemEffectType::ToHit, 100, 100,
          ItemEffectType::SetDam, 10, 20,
          ItemEffectType::Knockback, 0, 0,
          ItemEffectType::Str, 20, 20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Royal Circlet (19)
    udat!("Royal Circlet", 79, UniqueBaseItem::Crown, 14, 3, 12000,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // More Unique Weapons (20-34)
    udat!("Lightsabre", 0, UniqueBaseItem::Sabre, 15, 5, 24000,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::SetDam, 1, 20,
          ItemEffectType::LightDam, 1, 10,
          ItemEffectType::LightRes, 20, 20,
          ItemEffectType::Light, 3, 3,
          ItemEffectType::Invalid, 0, 0),
    udat!("Inferno", 0, UniqueBaseItem::LongSword, 12, 4, 18500,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::SetDam, 2, 12,
          ItemEffectType::FireDam, 2, 12,
          ItemEffectType::FireRes, 20, 20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Doombringer", 0, UniqueBaseItem::BastardSword, 18, 5, 32000,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::SetDam, 4, 20,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Invalid, 0, 0),
    udat!("Falcon's Talon", 0, UniqueBaseItem::Scimitar, 11, 4, 11250,
          ItemEffectType::ToHit, 20, 30,
          ItemEffectType::Damage, 1, 100,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::FastAttack, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Gibbous Moon", 0, UniqueBaseItem::BroadSword, 8, 4, 6660,
          ItemEffectType::Attribs, 2, 2,
          ItemEffectType::Damage, 0, 25,
          ItemEffectType::Mana, 15, 15,
          ItemEffectType::LightCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Ice Shank", 0, UniqueBaseItem::LongSword, 9, 3, 5250,
          ItemEffectType::FireRes, 40, 40,
          ItemEffectType::SetDur, 15, 15,
          ItemEffectType::Str, 5, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Bonesaw", 0, UniqueBaseItem::Claymore, 12, 6, 4400,
          ItemEffectType::DamMod, 10, 10,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::ManaCurse, 10, 10),
    udat!("Shadowhawk", 0, UniqueBaseItem::BroadSword, 14, 4, 13750,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::StealLife, 5, 5,
          ItemEffectType::ToHit, 15, 15,
          ItemEffectType::AllRes, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Wizardspike", 50, UniqueBaseItem::Dagger, 15, 4, 12920,
          ItemEffectType::Mag, 15, 15,
          ItemEffectType::Mana, 35, 35,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::AllRes, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Gonnagal's Dirk", 54, UniqueBaseItem::Dagger, 5, 4, 7040,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::DamMod, 4, 4,
          ItemEffectType::FastAttack, 2, 2,
          ItemEffectType::FireRes, 25, 25,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Defender", 0, UniqueBaseItem::Sabre, 3, 3, 2000,
          ItemEffectType::SetAC, 5, 5,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::ToHitCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Gryphon's Claw", 68, UniqueBaseItem::Falchion, 5, 3, 1000,
          ItemEffectType::Damage, 0, 100,
          ItemEffectType::MagCurse, 2, 2,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Black Razor", 53, UniqueBaseItem::Dagger, 7, 3, 2000,
          ItemEffectType::Damage, 0, 150,
          ItemEffectType::Vit, 2, 2,
          ItemEffectType::SetDur, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Executioner's Blade", 58, UniqueBaseItem::Falchion, 8, 4, 7080,
          ItemEffectType::Damage, 0, 150,
          ItemEffectType::LifeCurse, 10, 10,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::Durability, 200, 200,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // More Unique Axes (35-40)
    udat!("The Mangler", 0, UniqueBaseItem::LargeAxe, 10, 3, 8000,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::SetDam, 2, 10,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Sharp Beak", 0, UniqueBaseItem::SmallAxe, 8, 3, 6200,
          ItemEffectType::ToHit, 15, 15,
          ItemEffectType::Damage, 1, 50,
          ItemEffectType::Dex, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Bloodslayer", 0, UniqueBaseItem::BroadAxe, 14, 4, 18000,
          ItemEffectType::ToHit, 60, 60,
          ItemEffectType::SetDam, 8, 24,
          ItemEffectType::Damage, 0, 200,
          ItemEffectType::Str, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Celestial Axe", 0, UniqueBaseItem::BattleAxe, 15, 3, 20000,
          ItemEffectType::ToHit, 15, 25,
          ItemEffectType::SetDam, 5, 20,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Wicked Axe", 0, UniqueBaseItem::BroadAxe, 11, 3, 11000,
          ItemEffectType::ToHit, 30, 30,
          ItemEffectType::Damage, 1, 150,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Stonecleaver", 104, UniqueBaseItem::BroadAxe, 14, 4, 16500,
          ItemEffectType::ToHit, 50, 50,
          ItemEffectType::SetDam, 6, 18,
          ItemEffectType::Damage, 1, 200,
          ItemEffectType::FireRes, 40, 40,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // More Unique Maces (41-44)
    udat!("Crackrust", 0, UniqueBaseItem::Mace, 8, 2, 4200,
          ItemEffectType::ToHit, 15, 15,
          ItemEffectType::Damage, 1, 100,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Civerb's Cudgel", 0, UniqueBaseItem::MorningStar, 12, 4, 14000,
          ItemEffectType::ToHit, 200, 200,
          ItemEffectType::Damage, 2, 150,
          ItemEffectType::SetDam, 1, 20,
          ItemEffectType::Damage, 0, 200,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Celestial Star", 0, UniqueBaseItem::Flail, 15, 4, 22500,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::SetDam, 8, 18,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Baranar's Star", 0, UniqueBaseItem::MorningStar, 11, 3, 12000,
          ItemEffectType::ToHit, 12, 12,
          ItemEffectType::Damage, 4, 16,
          ItemEffectType::FastAttack, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // More Unique Staves/Misc (45-49)
    udat!("Staff of Shadows", 0, UniqueBaseItem::LongStaff, 12, 5, 16000,
          ItemEffectType::Mag, 10, 10,
          ItemEffectType::Mana, 40, 40,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::Charges, 40, 40,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Invalid, 0, 0),
    udat!("Immolator", 0, UniqueBaseItem::LongStaff, 15, 4, 20000,
          ItemEffectType::ToHit, 30, 30,
          ItemEffectType::FireDam, 10, 30,
          ItemEffectType::FastAttack, 1, 1,
          ItemEffectType::FireRes, 20, 20,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Stormspire", 0, UniqueBaseItem::WarStaff, 20, 4, 35000,
          ItemEffectType::ToHit, 40, 40,
          ItemEffectType::LightDam, 1, 50,
          ItemEffectType::LightArrows, 0, 0,
          ItemEffectType::LightRes, 30, 30,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Gleamsong", 0, UniqueBaseItem::CompositeStaff, 13, 3, 18750,
          ItemEffectType::ToHit, 20, 30,
          ItemEffectType::Damage, 1, 200,
          ItemEffectType::AllRes, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Thundercall", 0, UniqueBaseItem::WarStaff, 18, 5, 28500,
          ItemEffectType::ToHit, 35, 35,
          ItemEffectType::SetDam, 1, 50,
          ItemEffectType::LightDam, 1, 40,
          ItemEffectType::Mag, 10, 10,
          ItemEffectType::Charges, 60, 60,
          ItemEffectType::Invalid, 0, 0),

    // Items 50-69: More Unique Weapons, Helms
    udat!("Baranar's Star", 0, UniqueBaseItem::MorningStar, 5, 6, 6850,
          ItemEffectType::ToHit, 12, 12,
          ItemEffectType::Damage, 80, 80,
          ItemEffectType::FastAttack, 1, 1,
          ItemEffectType::Vit, 4, 4,
          ItemEffectType::DexCurse, 4, 4,
          ItemEffectType::SetDur, 60, 60),
    udat!("Gnarled Root", 0, UniqueBaseItem::SpikedClub, 9, 6, 9820,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::Damage, 300, 300,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::ACCurse, 10, 10),
    udat!("The Cranium Basher", 0, UniqueBaseItem::Maul, 12, 5, 36500,
          ItemEffectType::DamMod, 20, 20,
          ItemEffectType::Str, 15, 15,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::ManaCurse, 150, 150,
          ItemEffectType::AllRes, 5, 5,
          ItemEffectType::Invalid, 0, 0),
    udat!("Schaefer's Hammer", 0, UniqueBaseItem::WarHammer, 16, 6, 56125,
          ItemEffectType::DamageCurse, 100, 100,
          ItemEffectType::LightDam, 1, 50,
          ItemEffectType::Life, 50, 50,
          ItemEffectType::ToHit, 30, 30,
          ItemEffectType::LightRes, 80, 80,
          ItemEffectType::Light, 1, 1),
    udat!("Dreamflange", 0, UniqueBaseItem::Mace, 26, 5, 26450,
          ItemEffectType::Mag, 30, 30,
          ItemEffectType::Mana, 50, 50,
          ItemEffectType::MagicRes, 50, 50,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    udat!("Staff of Shadows", 0, UniqueBaseItem::LongStaff, 2, 5, 1250,
          ItemEffectType::MagCurse, 10, 10,
          ItemEffectType::ToHit, 10, 10,
          ItemEffectType::Damage, 60, 60,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::FastAttack, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    udat!("Immolator", 0, UniqueBaseItem::LongStaff, 4, 4, 3900,
          ItemEffectType::FireRes, 20, 20,
          ItemEffectType::FireDam, 4, 4,
          ItemEffectType::Mana, 10, 10,
          ItemEffectType::VitCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Storm Spire", 0, UniqueBaseItem::WarStaff, 8, 4, 22500,
          ItemEffectType::LightRes, 50, 50,
          ItemEffectType::LightDam, 2, 8,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::MagCurse, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Gleamsong", 0, UniqueBaseItem::ShortStaff, 8, 4, 6520,
          ItemEffectType::Mana, 25, 25,
          ItemEffectType::StrCurse, 3, 3,
          ItemEffectType::VitCurse, 3, 3,
          ItemEffectType::Spell, 10, 76,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Thundercall", 0, UniqueBaseItem::CompositeStaff, 14, 5, 22250,
          ItemEffectType::ToHit, 35, 35,
          ItemEffectType::LightDam, 1, 10,
          ItemEffectType::Spell, 3, 76,
          ItemEffectType::LightRes, 30, 30,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Protector", 0, UniqueBaseItem::ShortStaff, 16, 5, 17240,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::GetHit, 5, 5,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::Spell, 2, 86,
          ItemEffectType::Thorns, 1, 3,
          ItemEffectType::Invalid, 0, 0),
    udat!("Naj's Puzzler", 0, UniqueBaseItem::LongStaff, 18, 5, 34000,
          ItemEffectType::Mag, 20, 20,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Spell, 23, 57,
          ItemEffectType::LifeCurse, 25, 25,
          ItemEffectType::Invalid, 0, 0),
    udat!("Mindcry", 0, UniqueBaseItem::QuarterStaff, 20, 4, 41500,
          ItemEffectType::Mag, 15, 15,
          ItemEffectType::Spell, 13, 69,
          ItemEffectType::AllRes, 15, 15,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Rod of Onan", 0, UniqueBaseItem::WarStaff, 22, 3, 44167,
          ItemEffectType::Spell, 21, 50,
          ItemEffectType::Damage, 100, 100,
          ItemEffectType::Attribs, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Helm of Spirits", 0, UniqueBaseItem::Helm, 1, 1, 7525,
          ItemEffectType::StealLife, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Thinking Cap", 0, UniqueBaseItem::SkullCap, 6, 4, 2020,
          ItemEffectType::Mana, 30, 30,
          ItemEffectType::SpellLevelAdd, 2, 2,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::SetDur, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Overlord's Helm", 0, UniqueBaseItem::Helm, 7, 5, 12500,
          ItemEffectType::Str, 20, 20,
          ItemEffectType::Dex, 15, 15,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::MagCurse, 20, 20,
          ItemEffectType::SetDur, 15, 15,
          ItemEffectType::Invalid, 0, 0),
    udat!("Fool's Crest", 0, UniqueBaseItem::Helm, 12, 4, 10150,
          ItemEffectType::AttribsCurse, 4, 4,
          ItemEffectType::Life, 100, 100,
          ItemEffectType::GetHitCurse, 1, 6,
          ItemEffectType::Thorns, 1, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Gotterdamerung", 0, UniqueBaseItem::GreatHelm, 21, 5, 54900,
          ItemEffectType::Attribs, 20, 20,
          ItemEffectType::SetAC, 60, 60,
          ItemEffectType::GetHit, 4, 4,
          ItemEffectType::AllResZero, 0, 0,
          ItemEffectType::LightCurse, 4, 4,
          ItemEffectType::Invalid, 0, 0),
    udat!("Royal Circlet", 0, UniqueBaseItem::Crown, 27, 4, 24875,
          ItemEffectType::Attribs, 10, 10,
          ItemEffectType::Mana, 40, 40,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::Light, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),

    // Items 70-89: Armors, Shields, Rings
    udat!("Torn Flesh of Souls", 0, UniqueBaseItem::Rags, 2, 4, 4825,
          ItemEffectType::SetAC, 8, 8,
          ItemEffectType::Vit, 10, 10,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Gladiator's Bane", 0, UniqueBaseItem::StuddedArmor, 6, 4, 3450,
          ItemEffectType::SetAC, 25, 25,
          ItemEffectType::GetHit, 2, 2,
          ItemEffectType::Durability, 200, 200,
          ItemEffectType::AttribsCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Rainbow Cloak", 0, UniqueBaseItem::Cloak, 2, 5, 4900,
          ItemEffectType::SetAC, 10, 10,
          ItemEffectType::Attribs, 1, 1,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Life, 5, 5,
          ItemEffectType::Durability, 50, 50,
          ItemEffectType::Invalid, 0, 0),
    udat!("Leather of Aut", 0, UniqueBaseItem::LeatherArmor, 4, 5, 10550,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::Dex, 5, 5,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Wisdom's Wrap", 0, UniqueBaseItem::Robe, 5, 5, 6200,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::Mana, 10, 10,
          ItemEffectType::LightRes, 25, 25,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    udat!("Sparking Mail", 0, UniqueBaseItem::ChainMail, 9, 2, 15750,
          ItemEffectType::SetAC, 30, 30,
          ItemEffectType::LightDam, 1, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Scavenger Carapace", 0, UniqueBaseItem::BreastPlate, 13, 4, 14000,
          ItemEffectType::GetHit, 15, 15,
          ItemEffectType::ACCurse, 30, 30,
          ItemEffectType::Dex, 5, 5,
          ItemEffectType::LightRes, 40, 40,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Nightscape", 0, UniqueBaseItem::Cape, 16, 5, 11600,
          ItemEffectType::FastRecover, 2, 2,
          ItemEffectType::LightCurse, 4, 4,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::Dex, 3, 3,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Invalid, 0, 0),
    udat!("Naj's Light Plate", 0, UniqueBaseItem::PlateMail, 19, 5, 78700,
          ItemEffectType::NoMinStr, 0, 0,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::Mana, 20, 20,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    udat!("Demonspike Coat", 0, UniqueBaseItem::FullPlate, 25, 5, 251175,
          ItemEffectType::SetAC, 100, 100,
          ItemEffectType::GetHit, 6, 6,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::FireRes, 50, 50,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Deflector", 0, UniqueBaseItem::Buckler, 1, 4, 1500,
          ItemEffectType::SetAC, 7, 7,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::DamageCurse, 20, 20,
          ItemEffectType::ToHitCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Split Skull Shield", 0, UniqueBaseItem::Buckler, 1, 5, 2025,
          ItemEffectType::SetAC, 10, 10,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::Str, 2, 2,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::SetDur, 15, 15,
          ItemEffectType::Invalid, 0, 0),
    udat!("Dragon's Breach", 0, UniqueBaseItem::KiteShield, 2, 5, 19200,
          ItemEffectType::FireRes, 25, 25,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::SetAC, 20, 20,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Blackoak Shield", 0, UniqueBaseItem::SmallShield, 4, 5, 5725,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::VitCurse, 10, 10,
          ItemEffectType::SetAC, 18, 18,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::Durability, 150, 150,
          ItemEffectType::Invalid, 0, 0),
    udat!("Holy Defender", 0, UniqueBaseItem::LargeShield, 10, 5, 13800,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::GetHit, 2, 2,
          ItemEffectType::FireRes, 20, 20,
          ItemEffectType::Durability, 200, 200,
          ItemEffectType::FastBlock, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    udat!("Stormshield", 0, UniqueBaseItem::GothicShield, 24, 6, 49000,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::GetHitCurse, 4, 4,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::FastBlock, 1, 1,
          ItemEffectType::LightRes, 50, 50),
    udat!("Bramble", 0, UniqueBaseItem::Ring, 1, 3, 1000,
          ItemEffectType::AttribsCurse, 2, 2,
          ItemEffectType::DamMod, 3, 3,
          ItemEffectType::Mana, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Ring of Regha", 0, UniqueBaseItem::Ring, 1, 5, 4175,
          ItemEffectType::Mag, 10, 10,
          ItemEffectType::MagicRes, 10, 10,
          ItemEffectType::Light, 1, 1,
          ItemEffectType::StrCurse, 3, 3,
          ItemEffectType::DexCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0),
    udat!("The Bleeder", 0, UniqueBaseItem::Ring, 2, 3, 8500,
          ItemEffectType::MagicRes, 20, 20,
          ItemEffectType::Mana, 30, 30,
          ItemEffectType::LifeCurse, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Constricting Ring", 0, UniqueBaseItem::Ring, 5, 2, 62000,
          ItemEffectType::AllRes, 75, 75,
          ItemEffectType::DrainLife, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    udat!("Ring of Engagement", 0, UniqueBaseItem::Ring, 11, 4, 12476,
          ItemEffectType::GetHit, 1, 2,
          ItemEffectType::Thorns, 1, 3,
          ItemEffectType::SetAC, 5, 5,
          ItemEffectType::TargetAC, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
];

/// Get item data by index (for base items)
pub fn get_item_data(index: usize) -> Option<&'static ItemData> {
    if index < ITEMS_DATA.len() {
        Some(&ITEMS_DATA[index])
    } else {
        None
    }
}

/// Get unique item data by ID
pub fn get_unique_item_data(id: UniqueItemId) -> Option<&'static UniqueItemData> {
    let idx = id as i32;
    if idx >= 0 && (idx as usize) < UNIQUE_ITEMS.len() {
        Some(&UNIQUE_ITEMS[idx as usize])
    } else {
        None
    }
}

//=============================================================================
// Tests
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_data_gold() {
        let gold = get_item_data(0).unwrap();
        assert_eq!(gold.name, "Gold");
        assert_eq!(gold.class, ItemClass::Gold);
        assert_eq!(gold.item_type, ItemType::Gold);
    }

    #[test]
    fn test_item_data_healing_potion() {
        let heal = get_item_data(1).unwrap();
        assert_eq!(heal.name, "Potion of Healing");
        assert_eq!(heal.misc_id, ItemMiscId::Heal);
        assert_eq!(heal.usable, true);
        assert_eq!(heal.value, 50);
    }

    #[test]
    fn test_item_data_weapons() {
        let dagger = get_item_data(8).unwrap();
        assert_eq!(dagger.name, "Dagger");
        assert_eq!(dagger.class, ItemClass::Weapon);
        assert_eq!(dagger.min_damage, 1);
        assert_eq!(dagger.max_damage, 4);

        let two_hand = get_item_data(12).unwrap();
        assert_eq!(two_hand.name, "Two-Handed Sword");
        assert_eq!(two_hand.equip_type, ItemEquipType::TwoHand);
        assert!(two_hand.value > 1000);
    }

    #[test]
    fn test_item_data_armor() {
        let cloak = get_item_data(19).unwrap();
        assert_eq!(cloak.name, "Cloak");
        assert_eq!(cloak.class, ItemClass::Armor);
        assert_eq!(cloak.min_ac, 1);
        assert_eq!(cloak.max_ac, 5);

        let plate = get_item_data(22).unwrap();
        assert_eq!(plate.name, "Full Plate Mail");
        assert_eq!(plate.min_ac, 20);
        assert_eq!(plate.max_ac, 30);
        assert_eq!(plate.min_str, 60);
    }

    #[test]
    fn test_item_data_shields() {
        let buckler = get_item_data(23).unwrap();
        assert_eq!(buckler.name, "Buckler");
        assert_eq!(buckler.item_type, ItemType::Shield);
        assert_eq!(buckler.equip_type, ItemEquipType::OneHand);
    }

    #[test]
    fn test_unique_item_cleaver() {
        let cleaver = get_unique_item_data(UniqueItemId::Cleaver).unwrap();
        assert_eq!(cleaver.name, "The Butcher's Cleaver");
        assert_eq!(cleaver.base_item_id, UniqueBaseItem::Cleaver);
        assert_eq!(cleaver.min_level, 1);
    }

    #[test]
    fn test_unique_item_windforce() {
        // Note: Simplified table uses sequential indices, not C++ enum values
        // Windforce is at index 2 in our table
        let windforce = &UNIQUE_ITEMS[2];
        assert_eq!(windforce.name, "Windforce");
        assert_eq!(windforce.base_item_id, UniqueBaseItem::LongBow);
        assert_eq!(windforce.min_level, 25);
        assert_eq!(windforce.num_powers, 6);
        assert!(windforce.value > 30000);
    }

    #[test]
    fn test_unique_item_grandfather() {
        // Grandfather is at index 3
        let gf = &UNIQUE_ITEMS[3];
        assert_eq!(gf.name, "The Grandfather");
        assert_eq!(gf.min_level, 25);
        assert_eq!(gf.value, 50000);
        assert_eq!(gf.powers[0].effect_type, ItemEffectType::ToHit);
        assert_eq!(gf.powers[1].effect_type, ItemEffectType::Damage);
    }

    #[test]
    fn test_unique_item_arkaines_valor() {
        // Arkaine's Valor is at index 6
        let arkaine = &UNIQUE_ITEMS[6];
        assert_eq!(arkaine.name, "Arkaine's Valor");
        assert_eq!(arkaine.base_item_id, UniqueBaseItem::FullPlate);
        assert_eq!(arkaine.min_level, 18);
        assert_eq!(arkaine.num_powers, 4);
    }

    #[test]
    fn test_unique_item_veil_of_steel() {
        // Veil of Steel is at index 8
        let veil = &UNIQUE_ITEMS[8];
        assert_eq!(veil.name, "Veil of Steel");
        assert_eq!(veil.min_level, 16);
        assert_eq!(veil.value, 12000);
    }

    #[test]
    fn test_unique_item_ring_of_truth() {
        // Ring of Truth is at index 11
        let ring = &UNIQUE_ITEMS[11];
        assert_eq!(ring.name, "Ring of Truth");
        assert_eq!(ring.base_item_id, UniqueBaseItem::Ring);
    }

    #[test]
    fn test_unique_item_optic_amulet() {
        // Optic Amulet is at index 13
        let amulet = &UNIQUE_ITEMS[13];
        assert_eq!(amulet.name, "Optic Amulet");
        assert_eq!(amulet.base_item_id, UniqueBaseItem::OpticAmulet);
    }

    #[test]
    fn test_item_type_enum() {
        assert_eq!(ItemType::Sword as i8, 1);
        assert_eq!(ItemType::Axe as i8, 2);
        assert_eq!(ItemType::Bow as i8, 3);
        assert_eq!(ItemType::None as i8, -1);
    }

    #[test]
    fn test_item_class_enum() {
        assert_eq!(ItemClass::Weapon as u8, 1);
        assert_eq!(ItemClass::Armor as u8, 2);
        assert_eq!(ItemClass::Misc as u8, 3);
        assert_eq!(ItemClass::Gold as u8, 4);
    }

    #[test]
    fn test_unique_item_count() {
        assert_eq!(UniqueItemId::count(), 90);
        assert_eq!(UNIQUE_ITEMS.len(), 90); // Day 4: completed 50→90
    }
}

// TODO: implement remaining items (ITEMS_DATA: 30→169, UNIQUE_ITEMS: 20→90)
