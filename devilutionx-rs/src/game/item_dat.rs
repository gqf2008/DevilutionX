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
pub const ITEMS_DATA: [ItemData; 168] = [
    //   0 IDI_GOLD
    idat!(1, ItemClass::Gold, ItemEquipType::Unequipable, 4, ItemType::Gold,
          UniqueBaseItem::None, "Gold", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, true, 0),
    //   1 IDI_WARRIOR
    idat!(0, ItemClass::Weapon, ItemEquipType::OneHand, 64, ItemType::Sword,
          UniqueBaseItem::None, "Short Sword", "", 2, 24, 2, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 120),
    //   2 IDI_WARRSHLD
    idat!(0, ItemClass::Armor, ItemEquipType::OneHand, 83, ItemType::Shield,
          UniqueBaseItem::None, "Buckler", "", 2, 16, 0, 0, 3, 3, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 30),
    //   3 IDI_WARRCLUB
    idat!(0, ItemClass::Weapon, ItemEquipType::OneHand, 66, ItemType::Mace,
          UniqueBaseItem::SpikedClub, "Club", "", 1, 20, 1, 6, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 20),
    //   4 IDI_ROGUE
    idat!(0, ItemClass::Weapon, ItemEquipType::TwoHand, 118, ItemType::Bow,
          UniqueBaseItem::None, "Short Bow", "", 1, 30, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 100),
    //   5 IDI_SORCERER
    idat!(0, ItemClass::Weapon, ItemEquipType::TwoHand, 109, ItemType::Staff,
          UniqueBaseItem::None, "Short Staff of Mana", "", 1, 25, 2, 4, 0, 0, 0, 17, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::Mana, false, 210),
    //   6 IDI_CLEAVER
    idat!(0, ItemClass::Weapon, ItemEquipType::TwoHand, 106, ItemType::Axe,
          UniqueBaseItem::Cleaver, "Cleaver", "", 10, 10, 4, 24, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 2000),
    //   7 IDI_SKCROWN
    idat!(0, ItemClass::Armor, ItemEquipType::Helm, 78, ItemType::Helm,
          UniqueBaseItem::SkeletonCrown, "The Undead Crown", "", 0, 50, 0, 0, 15, 15, 0, 0, 0,
          ItemSpecialEffect::RANDOM_STEAL_LIFE, ItemMiscId::Unique, SpellID::Null, false, 10000),
    //   8 IDI_INFRARING
    idat!(0, ItemClass::Misc, ItemEquipType::Ring, 18, ItemType::Ring,
          UniqueBaseItem::InfraRing, "Empyrean Band", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 8000),
    //   9 IDI_ROCK
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 76, ItemType::Misc,
          UniqueBaseItem::None, "Magic Rock", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  10 IDI_OPTAMULET
    idat!(0, ItemClass::Misc, ItemEquipType::Amulet, 44, ItemType::Amulet,
          UniqueBaseItem::OpticAmulet, "Optic Amulet", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 5000),
    //  11 IDI_TRING
    idat!(0, ItemClass::Misc, ItemEquipType::Ring, 10, ItemType::Ring,
          UniqueBaseItem::TRing, "Ring of Truth", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 1000),
    //  12 IDI_BANNER
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 126, ItemType::Misc,
          UniqueBaseItem::None, "Tavern Sign", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  13 IDI_HARCREST
    idat!(0, ItemClass::Armor, ItemEquipType::Helm, 81, ItemType::Helm,
          UniqueBaseItem::HarlequinCrest, "Harlequin Crest", "", 0, 15, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 15),
    //  14 IDI_STEELVEIL
    idat!(0, ItemClass::Armor, ItemEquipType::Helm, 85, ItemType::Helm,
          UniqueBaseItem::SteelVeil, "Veil of Steel", "", 0, 60, 0, 0, 18, 18, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 0),
    //  15 IDI_GLDNELIX
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 17, ItemType::Misc,
          UniqueBaseItem::Elixir, "Golden Elixir", "", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  16 IDI_ANVIL
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 140, ItemType::Misc,
          UniqueBaseItem::None, "Anvil of Fury", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  17 IDI_MUSHROOM
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 89, ItemType::Misc,
          UniqueBaseItem::None, "Black Mushroom", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  18 IDI_BRAIN
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 40, ItemType::Misc,
          UniqueBaseItem::None, "Brain", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  19 IDI_FUNGALTM
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 97, ItemType::Misc,
          UniqueBaseItem::None, "Fungal Tome", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  20 IDI_SPECELIX
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 15, ItemType::Misc,
          UniqueBaseItem::Elixir, "Spectral Elixir", "", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::SpecElixir, SpellID::Null, true, 0),
    //  21 IDI_BLDSTONE
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 25, ItemType::Misc,
          UniqueBaseItem::None, "Blood Stone", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  22 IDI_MAPOFDOOM
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 96, ItemType::Misc,
          UniqueBaseItem::MapOfDoom, "Cathedral Map", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::MapOfDoom, SpellID::Null, true, 0),
    //  23 IDI_EAR
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 255, ItemType::Misc,
          UniqueBaseItem::None, "Ear", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ear, SpellID::Null, false, 0),
    //  24 IDI_HEAL
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 32, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Healing", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Heal, SpellID::Null, true, 50),
    //  25 IDI_MANA
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 39, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Mana", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Mana, SpellID::Null, true, 50),
    //  26 IDI_IDENTIFY
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Identify", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Identify, true, 200),
    //  27 IDI_PORTAL
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Town Portal", "", 4, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::TownPortal, true, 200),
    //  28 IDI_ARMOFVAL
    idat!(0, ItemClass::Armor, ItemEquipType::Armor, 157, ItemType::MediumArmor,
          UniqueBaseItem::ArmorOfValor, "Arkaine's Valor", "", 0, 40, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 0),
    //  29 IDI_FULLHEAL
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 35, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Healing", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullHeal, SpellID::Null, true, 150),
    //  30 IDI_FULLMANA
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 0, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Mana", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullMana, SpellID::Null, true, 150),
    //  31 IDI_GRISWOLD
    idat!(0, ItemClass::Weapon, ItemEquipType::OneHand, 61, ItemType::Sword,
          UniqueBaseItem::Griswold, "Griswold's Edge", "", 8, 50, 4, 12, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 750),
    //  32 IDI_LGTFORGE
    idat!(0, ItemClass::Armor, ItemEquipType::Armor, 226, ItemType::HeavyArmor,
          UniqueBaseItem::Bovine, "Bovine Plate", "", 0, 40, 0, 0, 0, 0, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Unique, SpellID::Null, false, 0),
    //  33 IDI_LAZSTAFF
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 155, ItemType::Misc,
          UniqueBaseItem::LazarusStaff, "Staff of Lazarus", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  34 IDI_RESURRECT
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Resurrect", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Resurrect, true, 250),
    //  35 IDI_OIL
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 30, ItemType::Misc,
          UniqueBaseItem::None, "Blacksmith Oil", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilBSmith, SpellID::Null, true, 100),
    //  36 IDI_SHORTSTAFF
    idat!(0, ItemClass::Weapon, ItemEquipType::TwoHand, 109, ItemType::Staff,
          UniqueBaseItem::None, "Short Staff", "", 1, 25, 2, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 20),
    //  37 IDI_BARDSWORD
    idat!(0, ItemClass::Weapon, ItemEquipType::OneHand, 64, ItemType::Sword,
          UniqueBaseItem::None, "Sword", "", 2, 8, 1, 5, 0, 0, 15, 0, 20,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 20),
    //  38 IDI_BARDDAGGER
    idat!(0, ItemClass::Weapon, ItemEquipType::OneHand, 51, ItemType::Sword,
          UniqueBaseItem::None, "Dagger", "", 1, 16, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 20),
    //  39 IDI_RUNEBOMB
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 187, ItemType::Misc,
          UniqueBaseItem::None, "Rune Bomb", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  40 IDI_THEODORE
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 188, ItemType::Misc,
          UniqueBaseItem::None, "Theodore", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  41 IDI_AURIC
    idat!(0, ItemClass::Misc, ItemEquipType::Amulet, 180, ItemType::Misc,
          UniqueBaseItem::None, "Auric Amulet", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::AuricAmulet, SpellID::Null, false, 100),
    //  42 IDI_NOTE1
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 189, ItemType::Misc,
          UniqueBaseItem::None, "Torn Note 1", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  43 IDI_NOTE2
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 190, ItemType::Misc,
          UniqueBaseItem::None, "Torn Note 2", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  44 IDI_NOTE3
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 191, ItemType::Misc,
          UniqueBaseItem::None, "Torn Note 3", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  45 IDI_FULLNOTE
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 192, ItemType::Misc,
          UniqueBaseItem::None, "Reconstructed Note", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Note, SpellID::Null, true, 0),
    //  46 IDI_BROWNSUIT
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 199, ItemType::Misc,
          UniqueBaseItem::None, "Brown Suit", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  47 IDI_GREYSUIT
    idat!(0, ItemClass::Quest, ItemEquipType::Unequipable, 198, ItemType::Misc,
          UniqueBaseItem::None, "Grey Suit", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    //  48 Cap
    idat!(1, ItemClass::Armor, ItemEquipType::Helm, 91, ItemType::Helm,
          UniqueBaseItem::None, "Cap", "Cap", 1, 15, 0, 0, 1, 3, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 15),
    //  49 Skull Cap
    idat!(1, ItemClass::Armor, ItemEquipType::Helm, 90, ItemType::Helm,
          UniqueBaseItem::SkullCap, "Skull Cap", "Cap", 4, 20, 0, 0, 2, 4, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 25),
    //  50 Helm
    idat!(1, ItemClass::Armor, ItemEquipType::Helm, 82, ItemType::Helm,
          UniqueBaseItem::Helm, "Helm", "Helm", 8, 30, 0, 0, 4, 6, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 40),
    //  51 Full Helm
    idat!(1, ItemClass::Armor, ItemEquipType::Helm, 75, ItemType::Helm,
          UniqueBaseItem::None, "Full Helm", "Helm", 12, 35, 0, 0, 6, 8, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 90),
    //  52 Crown
    idat!(1, ItemClass::Armor, ItemEquipType::Helm, 95, ItemType::Helm,
          UniqueBaseItem::Crown, "Crown", "Crown", 16, 40, 0, 0, 8, 12, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    //  53 Great Helm
    idat!(1, ItemClass::Armor, ItemEquipType::Helm, 98, ItemType::Helm,
          UniqueBaseItem::GreatHelm, "Great Helm", "Helm", 20, 60, 0, 0, 10, 15, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 400),
    //  54 Cape
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 150, ItemType::LightArmor,
          UniqueBaseItem::Cape, "Cape", "Cape", 1, 12, 0, 0, 1, 5, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 10),
    //  55 Rags
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 128, ItemType::LightArmor,
          UniqueBaseItem::Rags, "Rags", "Rags", 1, 6, 0, 0, 2, 6, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 5),
    //  56 Cloak
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 149, ItemType::LightArmor,
          UniqueBaseItem::Cloak, "Cloak", "Cloak", 2, 18, 0, 0, 3, 7, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 40),
    //  57 Robe
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 137, ItemType::LightArmor,
          UniqueBaseItem::Robe, "Robe", "Robe", 3, 24, 0, 0, 4, 7, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 75),
    //  58 Quilted Armor
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 129, ItemType::LightArmor,
          UniqueBaseItem::None, "Quilted Armor", "Armor", 4, 30, 0, 0, 7, 10, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    //  59 Leather Armor
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 135, ItemType::LightArmor,
          UniqueBaseItem::LeatherArmor, "Leather Armor", "Armor", 6, 35, 0, 0, 10, 13, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 300),
    //  60 Hard Leather Armor
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 127, ItemType::LightArmor,
          UniqueBaseItem::None, "Hard Leather Armor", "Armor", 7, 40, 0, 0, 11, 14, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    //  61 Studded Leather Armor
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 107, ItemType::LightArmor,
          UniqueBaseItem::StuddedArmor, "Studded Leather Armor", "Armor", 9, 45, 0, 0, 15, 17, 20, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 700),
    //  62 Ring Mail
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 154, ItemType::MediumArmor,
          UniqueBaseItem::None, "Ring Mail", "Mail", 11, 50, 0, 0, 17, 20, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 900),
    //  63 Chain Mail
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 111, ItemType::MediumArmor,
          UniqueBaseItem::ChainMail, "Chain Mail", "Mail", 13, 55, 0, 0, 18, 22, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1250),
    //  64 Scale Mail
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 114, ItemType::MediumArmor,
          UniqueBaseItem::None, "Scale Mail", "Mail", 15, 60, 0, 0, 23, 28, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2300),
    //  65 Breast Plate
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 153, ItemType::HeavyArmor,
          UniqueBaseItem::BreastPlate, "Breast Plate", "Plate", 16, 80, 0, 0, 20, 24, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2800),
    //  66 Splint Mail
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 136, ItemType::MediumArmor,
          UniqueBaseItem::None, "Splint Mail", "Mail", 17, 65, 0, 0, 30, 35, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 3250),
    //  67 Plate Mail
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 103, ItemType::HeavyArmor,
          UniqueBaseItem::PlateMail, "Plate Mail", "Plate", 19, 75, 0, 0, 42, 50, 60, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 4600),
    //  68 Field Plate
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 103, ItemType::HeavyArmor,
          UniqueBaseItem::None, "Field Plate", "Plate", 21, 80, 0, 0, 40, 45, 65, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 5800),
    //  69 Gothic Plate
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 152, ItemType::HeavyArmor,
          UniqueBaseItem::None, "Gothic Plate", "Plate", 23, 100, 0, 0, 50, 60, 80, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 8000),
    //  70 Full Plate Mail
    idat!(1, ItemClass::Armor, ItemEquipType::Armor, 151, ItemType::HeavyArmor,
          UniqueBaseItem::FullPlate, "Full Plate Mail", "Plate", 25, 90, 0, 0, 60, 75, 90, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 6500),
    //  71 Buckler
    idat!(1, ItemClass::Armor, ItemEquipType::OneHand, 83, ItemType::Shield,
          UniqueBaseItem::Buckler, "Buckler", "Shield", 1, 16, 0, 0, 1, 5, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 30),
    //  72 Small Shield
    idat!(1, ItemClass::Armor, ItemEquipType::OneHand, 105, ItemType::Shield,
          UniqueBaseItem::SmallShield, "Small Shield", "Shield", 5, 24, 0, 0, 3, 8, 25, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 90),
    //  73 Large Shield
    idat!(1, ItemClass::Armor, ItemEquipType::OneHand, 147, ItemType::Shield,
          UniqueBaseItem::LargeShield, "Large Shield", "Shield", 9, 32, 0, 0, 5, 10, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    //  74 Kite Shield
    idat!(1, ItemClass::Armor, ItemEquipType::OneHand, 113, ItemType::Shield,
          UniqueBaseItem::KiteShield, "Kite Shield", "Shield", 14, 40, 0, 0, 8, 15, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 400),
    //  75 Tower Shield
    idat!(1, ItemClass::Armor, ItemEquipType::OneHand, 132, ItemType::Shield,
          UniqueBaseItem::GothicShield, "Tower Shield", "Shield", 20, 50, 0, 0, 12, 20, 60, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 850),
    //  76 Gothic Shield
    idat!(1, ItemClass::Armor, ItemEquipType::OneHand, 148, ItemType::Shield,
          UniqueBaseItem::GothicShield, "Gothic Shield", "Shield", 23, 60, 0, 0, 14, 18, 80, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2300),
    //  77 Potion of Healing
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 32, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Healing", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Heal, SpellID::Null, true, 50),
    //  78 Potion of Full Healing
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 35, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Healing", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullHeal, SpellID::Null, true, 150),
    //  79 Potion of Mana
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 39, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Mana", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Mana, SpellID::Null, true, 50),
    //  80 Potion of Full Mana
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 0, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Mana", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullMana, SpellID::Null, true, 150),
    //  81 Potion of Rejuvenation
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 37, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Rejuvenation", "", 3, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Rejuv, SpellID::Null, true, 120),
    //  82 Potion of Full Rejuvenation
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 33, ItemType::Misc,
          UniqueBaseItem::None, "Potion of Full Rejuvenation", "", 7, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::FullRejuv, SpellID::Null, true, 600),
    //  83 Blacksmith Oil
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 30, ItemType::Misc,
          UniqueBaseItem::None, "Blacksmith Oil", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilBSmith, SpellID::Null, true, 100),
    //  84 Oil of Accuracy
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 30, ItemType::Misc,
          UniqueBaseItem::None, "Oil of Accuracy", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilAcc, SpellID::Null, true, 500),
    //  85 Oil of Sharpness
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 30, ItemType::Misc,
          UniqueBaseItem::None, "Oil of Sharpness", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilSharp, SpellID::Null, true, 500),
    //  86 Oil
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 30, ItemType::Misc,
          UniqueBaseItem::None, "Oil", "", 10, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::OilOf, SpellID::Null, true, 0),
    //  87 Elixir of Strength
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 38, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Strength", "", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirStr, SpellID::Null, true, 5000),
    //  88 Elixir of Magic
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 34, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Magic", "", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirMag, SpellID::Null, true, 5000),
    //  89 Elixir of Dexterity
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 36, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Dexterity", "", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirDex, SpellID::Null, true, 5000),
    //  90 Elixir of Vitality
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 31, ItemType::Misc,
          UniqueBaseItem::None, "Elixir of Vitality", "", 20, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ElixirVit, SpellID::Null, true, 5000),
    //  91 Scroll of Healing
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Healing", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Healing, true, 50),
    //  92 Scroll of Search
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Search", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Search, true, 50),
    //  93 Scroll of Lightning
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Lightning", "", 4, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Lightning, true, 150),
    //  94 Scroll of Identify
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Identify", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Identify, true, 100),
    //  95 Scroll of Resurrect
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Resurrect", "", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Resurrect, true, 250),
    //  96 Scroll of Fire Wall
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Fire Wall", "", 4, 0, 0, 0, 0, 0, 0, 17, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::FireWall, true, 400),
    //  97 Scroll of Inferno
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Inferno", "", 1, 0, 0, 0, 0, 0, 0, 19, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Inferno, true, 100),
    //  98 Scroll of Town Portal
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Town Portal", "", 4, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::TownPortal, true, 200),
    //  99 Scroll of Flash
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Flash", "", 6, 0, 0, 0, 0, 0, 0, 21, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Flash, true, 500),
    // 100 Scroll of Infravision
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Infravision", "", 8, 0, 0, 0, 0, 0, 0, 23, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Infravision, true, 600),
    // 101 Scroll of Phasing
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Phasing", "", 6, 0, 0, 0, 0, 0, 0, 25, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Phasing, true, 200),
    // 102 Scroll of Mana Shield
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Mana Shield", "", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::ManaShield, true, 1200),
    // 103 Scroll of Flame Wave
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Flame Wave", "", 10, 0, 0, 0, 0, 0, 0, 29, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::FlameWave, true, 650),
    // 104 Scroll of Fireball
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Fireball", "", 8, 0, 0, 0, 0, 0, 0, 31, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Fireball, true, 300),
    // 105 Scroll of Stone Curse
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Stone Curse", "", 6, 0, 0, 0, 0, 0, 0, 33, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::StoneCurse, true, 800),
    // 106 Scroll of Chain Lightning
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Chain Lightning", "", 10, 0, 0, 0, 0, 0, 0, 35, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::ChainLightning, true, 750),
    // 107 Scroll of Guardian
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Guardian", "", 12, 0, 0, 0, 0, 0, 0, 47, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Guardian, true, 950),
    // 108 
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 255, ItemType::Misc,
          UniqueBaseItem::None, "", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    // 109 Scroll of Nova
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Nova", "", 14, 0, 0, 0, 0, 0, 0, 57, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Nova, true, 1300),
    // 110 Scroll of Golem
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Golem", "", 10, 0, 0, 0, 0, 0, 0, 51, 0,
          ItemSpecialEffect::NONE, ItemMiscId::ScrollT, SpellID::Golem, true, 1100),
    // 111 
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 255, ItemType::Misc,
          UniqueBaseItem::None, "", "", 0, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 0),
    // 112 Scroll of Teleport
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Teleport", "", 14, 0, 0, 0, 0, 0, 0, 81, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Teleport, true, 3000),
    // 113 Scroll of Apocalypse
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 1, ItemType::Misc,
          UniqueBaseItem::None, "Scroll of Apocalypse", "", 22, 0, 0, 0, 0, 0, 0, 117, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Scroll, SpellID::Apocalypse, true, 2000),
    // 114 IDI_BOOK1
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 88, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "", 2, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    // 115 IDI_BOOK2
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 88, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    // 116 IDI_BOOK3
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 88, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "", 14, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    // 117 IDI_BOOK4
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 88, ItemType::Misc,
          UniqueBaseItem::None, "Book of ", "", 20, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Book, SpellID::Null, true, 0),
    // 118 Dagger
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 51, ItemType::Sword,
          UniqueBaseItem::Dagger, "Dagger", "Dagger", 1, 16, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 60),
    // 119 Short Sword
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 64, ItemType::Sword,
          UniqueBaseItem::None, "Short Sword", "Sword", 1, 24, 2, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 120),
    // 120 Falchion
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 62, ItemType::Sword,
          UniqueBaseItem::Falchion, "Falchion", "Sword", 2, 20, 4, 8, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 250),
    // 121 Scimitar
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 72, ItemType::Sword,
          UniqueBaseItem::Scimitar, "Scimitar", "Sword", 4, 28, 3, 7, 0, 0, 23, 0, 23,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    // 122 Claymore
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 65, ItemType::Sword,
          UniqueBaseItem::Claymore, "Claymore", "Sword", 5, 36, 1, 12, 0, 0, 35, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    // 123 Blade
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 56, ItemType::Sword,
          UniqueBaseItem::None, "Blade", "Blade", 4, 30, 3, 8, 0, 0, 25, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 280),
    // 124 Sabre
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 67, ItemType::Sword,
          UniqueBaseItem::Sabre, "Sabre", "Sabre", 1, 45, 1, 8, 0, 0, 17, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 170),
    // 125 Long Sword
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 60, ItemType::Sword,
          UniqueBaseItem::LongSword, "Long Sword", "Sword", 6, 40, 2, 10, 0, 0, 30, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    // 126 Broad Sword
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 61, ItemType::Sword,
          UniqueBaseItem::BroadSword, "Broad Sword", "Sword", 8, 50, 4, 12, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 750),
    // 127 Bastard Sword
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 57, ItemType::Sword,
          UniqueBaseItem::BastardSword, "Bastard Sword", "Sword", 10, 60, 6, 15, 0, 0, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    // 128 Two-Handed Sword
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 110, ItemType::Sword,
          UniqueBaseItem::TwoHandSword, "Two-Handed Sword", "Sword", 14, 75, 8, 16, 0, 0, 65, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1800),
    // 129 Great Sword
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 134, ItemType::Sword,
          UniqueBaseItem::GreatSword, "Great Sword", "Sword", 17, 100, 10, 20, 0, 0, 75, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 3000),
    // 130 Small Axe
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 112, ItemType::Axe,
          UniqueBaseItem::SmallAxe, "Small Axe", "Axe", 2, 24, 2, 10, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 150),
    // 131 Axe
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 144, ItemType::Axe,
          UniqueBaseItem::None, "Axe", "Axe", 4, 32, 4, 12, 0, 0, 22, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 450),
    // 132 Large Axe
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 142, ItemType::Axe,
          UniqueBaseItem::LargeAxe, "Large Axe", "Axe", 6, 40, 6, 16, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 750),
    // 133 Broad Axe
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 141, ItemType::Axe,
          UniqueBaseItem::BroadAxe, "Broad Axe", "Axe", 8, 50, 8, 20, 0, 0, 50, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    // 134 Battle Axe
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 101, ItemType::Axe,
          UniqueBaseItem::BattleAxe, "Battle Axe", "Axe", 10, 60, 10, 25, 0, 0, 65, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1500),
    // 135 Great Axe
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 143, ItemType::Axe,
          UniqueBaseItem::GreatAxe, "Great Axe", "Axe", 12, 75, 12, 30, 0, 0, 80, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2500),
    // 136 Mace
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 59, ItemType::Mace,
          UniqueBaseItem::Mace, "Mace", "Mace", 2, 32, 1, 8, 0, 0, 16, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 200),
    // 137 Morning Star
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 63, ItemType::Mace,
          UniqueBaseItem::MorningStar, "Morning Star", "Mace", 3, 40, 1, 10, 0, 0, 26, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 300),
    // 138 War Hammer
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 121, ItemType::Mace,
          UniqueBaseItem::WarHammer, "War Hammer", "Hammer", 5, 50, 5, 9, 0, 0, 40, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 600),
    // 139 IDI_BARBARIAN
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 70, ItemType::Mace,
          UniqueBaseItem::SpikedClub, "Spiked Club", "Club", 4, 20, 3, 6, 0, 0, 18, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 225),
    // 140 Club
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 66, ItemType::Mace,
          UniqueBaseItem::SpikedClub, "Club", "Club", 1, 20, 1, 6, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 20),
    // 141 Flail
    idat!(1, ItemClass::Weapon, ItemEquipType::OneHand, 131, ItemType::Mace,
          UniqueBaseItem::Flail, "Flail", "Flail", 7, 36, 2, 12, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 500),
    // 142 Maul
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 122, ItemType::Mace,
          UniqueBaseItem::Maul, "Maul", "Maul", 10, 50, 6, 20, 0, 0, 55, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 900),
    // 143 Short Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 118, ItemType::Bow,
          UniqueBaseItem::ShortBow, "Short Bow", "Bow", 1, 30, 1, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 100),
    // 144 Hunter's Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 102, ItemType::Bow,
          UniqueBaseItem::HunterBow, "Hunter's Bow", "Bow", 3, 40, 2, 5, 0, 0, 20, 0, 35,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 350),
    // 145 Long Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 102, ItemType::Bow,
          UniqueBaseItem::LongBow, "Long Bow", "Bow", 5, 35, 1, 6, 0, 0, 25, 0, 30,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 250),
    // 146 Composite Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 133, ItemType::Bow,
          UniqueBaseItem::CompositeBow, "Composite Bow", "Bow", 7, 45, 3, 6, 0, 0, 25, 0, 40,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 600),
    // 147 Short Battle Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 167, ItemType::Bow,
          UniqueBaseItem::None, "Short Battle Bow", "Bow", 9, 45, 3, 7, 0, 0, 30, 0, 50,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 750),
    // 148 IDI_SHORT_BATTLE_BOW
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 119, ItemType::Bow,
          UniqueBaseItem::BattleBow, "Long Battle Bow", "Bow", 11, 50, 1, 10, 0, 0, 30, 0, 60,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1000),
    // 149 Short War Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 165, ItemType::Bow,
          UniqueBaseItem::None, "Short War Bow", "Bow", 15, 55, 4, 8, 0, 0, 35, 0, 70,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 1500),
    // 150 Long War Bow
    idat!(2, ItemClass::Weapon, ItemEquipType::TwoHand, 120, ItemType::Bow,
          UniqueBaseItem::WarBow, "Long War Bow", "Bow", 19, 60, 1, 14, 0, 0, 45, 0, 80,
          ItemSpecialEffect::NONE, ItemMiscId::None, SpellID::Null, false, 2000),
    // 151 Short Staff
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 109, ItemType::Staff,
          UniqueBaseItem::ShortStaff, "Short Staff", "Staff", 1, 25, 2, 4, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::Null, false, 30),
    // 152 Long Staff
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 123, ItemType::Staff,
          UniqueBaseItem::LongStaff, "Long Staff", "Staff", 4, 35, 4, 8, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::Null, false, 100),
    // 153 Composite Staff
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 166, ItemType::Staff,
          UniqueBaseItem::CompositeStaff, "Composite Staff", "Staff", 6, 45, 5, 10, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::Null, false, 500),
    // 154 Quarter Staff
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 109, ItemType::Staff,
          UniqueBaseItem::QuarterStaff, "Quarter Staff", "Staff", 9, 55, 6, 12, 0, 0, 20, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::Null, false, 1000),
    // 155 War Staff
    idat!(1, ItemClass::Weapon, ItemEquipType::TwoHand, 124, ItemType::Staff,
          UniqueBaseItem::WarStaff, "War Staff", "Staff", 12, 75, 8, 16, 0, 0, 30, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::Null, false, 1500),
    // 156 Ring
    idat!(1, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "Ring", 5, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 1000),
    // 157 Ring
    idat!(1, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "Ring", 10, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 1000),
    // 158 Ring
    idat!(1, ItemClass::Misc, ItemEquipType::Ring, 12, ItemType::Ring,
          UniqueBaseItem::Ring, "Ring", "Ring", 15, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Ring, SpellID::Null, false, 1000),
    // 159 Amulet
    idat!(1, ItemClass::Misc, ItemEquipType::Amulet, 45, ItemType::Amulet,
          UniqueBaseItem::Amulet, "Amulet", "Amulet", 8, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Amulet, SpellID::Null, false, 1200),
    // 160 Amulet
    idat!(1, ItemClass::Misc, ItemEquipType::Amulet, 45, ItemType::Amulet,
          UniqueBaseItem::Amulet, "Amulet", "Amulet", 16, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Amulet, SpellID::Null, false, 1200),
    // 161 Rune of Fire
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 193, ItemType::Misc,
          UniqueBaseItem::None, "Rune of Fire", "Rune", 1, 0, 0, 0, 0, 0, 0, 0, 0,
          ItemSpecialEffect::NONE, ItemMiscId::RuneF, SpellID::Null, true, 100),
    // 162 Rune of Lightning
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 195, ItemType::Misc,
          UniqueBaseItem::None, "Rune of Lightning", "Rune", 3, 0, 0, 0, 0, 0, 0, 13, 0,
          ItemSpecialEffect::NONE, ItemMiscId::RuneL, SpellID::Null, true, 200),
    // 163 Greater Rune of Fire
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 194, ItemType::Misc,
          UniqueBaseItem::None, "Greater Rune of Fire", "Rune", 7, 0, 0, 0, 0, 0, 0, 42, 0,
          ItemSpecialEffect::NONE, ItemMiscId::GrRuneF, SpellID::Null, true, 400),
    // 164 Greater Rune of Lightning
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 196, ItemType::Misc,
          UniqueBaseItem::None, "Greater Rune of Lightning", "Rune", 7, 0, 0, 0, 0, 0, 0, 42, 0,
          ItemSpecialEffect::NONE, ItemMiscId::GrRuneL, SpellID::Null, true, 500),
    // 165 IDI_RUNEOFSTONE
    idat!(1, ItemClass::Misc, ItemEquipType::Unequipable, 197, ItemType::Misc,
          UniqueBaseItem::None, "Rune of Stone", "Rune", 7, 0, 0, 0, 0, 0, 0, 25, 0,
          ItemSpecialEffect::NONE, ItemMiscId::RuneS, SpellID::Null, true, 300),
    // 166 IDI_SORCERER_DIABLO
    idat!(0, ItemClass::Weapon, ItemEquipType::TwoHand, 109, ItemType::Staff,
          UniqueBaseItem::None, "Short Staff of Charged Bolt", "", 1, 25, 2, 4, 0, 0, 0, 25, 0,
          ItemSpecialEffect::NONE, ItemMiscId::Staff, SpellID::ChargedBolt, false, 470),
    // 167 IDI_ARENAPOT
    idat!(0, ItemClass::Misc, ItemEquipType::Unequipable, 16, ItemType::Misc,
          UniqueBaseItem::None, "Arena Potion", "", 7, 0, 0, 0, 0, 0, 0, 0, 0,
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
    //   0 The Butcher's Cleaver
    udat!("The Butcher's Cleaver", 255, UniqueBaseItem::Cleaver, 1, 3, 3650,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::SetDam, 4, 24,
          ItemEffectType::SetDur, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //   1 The Undead Crown
    udat!("The Undead Crown", 77, UniqueBaseItem::SkeletonCrown, 1, 2, 16650,
          ItemEffectType::RndStealLife, 0, 0,
          ItemEffectType::SetAC, 8, 8,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //   2 Empyrean Band
    udat!("Empyrean Band", 255, UniqueBaseItem::InfraRing, 1, 4, 8000,
          ItemEffectType::Attribs, 2, 2,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::FastRecover, 1, 1,
          ItemEffectType::AbsHalfTrap, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //   3 Optic Amulet
    udat!("Optic Amulet", 255, UniqueBaseItem::OpticAmulet, 1, 4, 9750,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::LightRes, 20, 20,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //   4 Ring of Truth
    udat!("Ring of Truth", 255, UniqueBaseItem::TRing, 1, 3, 9100,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //   5 Harlequin Crest
    udat!("Harlequin Crest", 255, UniqueBaseItem::HarlequinCrest, 1, 5, 4000,
          ItemEffectType::ACCurse, 3, 3,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::Attribs, 2, 2,
          ItemEffectType::Life, 7, 7,
          ItemEffectType::Mana, 7, 7,
          ItemEffectType::Invalid, 0, 0),
    //   6 Veil of Steel
    udat!("Veil of Steel", 255, UniqueBaseItem::SteelVeil, 1, 6, 63800,
          ItemEffectType::AllRes, 50, 50,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::ArmorPercent, 60, 60,
          ItemEffectType::ManaCurse, 30, 30,
          ItemEffectType::Str, 15, 15,
          ItemEffectType::Vit, 15, 15),
    //   7 Arkaine's Valor
    udat!("Arkaine's Valor", 255, UniqueBaseItem::ArmorOfValor, 1, 4, 42000,
          ItemEffectType::SetAC, 25, 25,
          ItemEffectType::Vit, 10, 10,
          ItemEffectType::GetHit, 3, 3,
          ItemEffectType::FastRecover, 3, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //   8 Griswold's Edge
    udat!("Griswold's Edge", 255, UniqueBaseItem::Griswold, 1, 6, 42000,
          ItemEffectType::FireDam, 1, 10,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::FastAttack, 2, 2,
          ItemEffectType::Knockback, 0, 0,
          ItemEffectType::Mana, 20, 20,
          ItemEffectType::LifeCurse, 20, 20),
    //   9 Lightforge
    udat!("Lightforge", 255, UniqueBaseItem::LightningForge, 1, 6, 26675,
          ItemEffectType::Light, 4, 4,
          ItemEffectType::Damage, 150, 150,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::FireDam, 10, 20,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Attribs, 8, 8),
    //  10 The Rift Bow
    udat!("The Rift Bow", 255, UniqueBaseItem::ShortBow, 1, 3, 1800,
          ItemEffectType::RndArrowVel, 0, 0,
          ItemEffectType::DamMod, 2, 2,
          ItemEffectType::DexCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  11 The Needler
    udat!("The Needler", 158, UniqueBaseItem::ShortBow, 2, 3, 8900,
          ItemEffectType::ToHit, 50, 50,
          ItemEffectType::SetDam, 1, 3,
          ItemEffectType::FastAttack, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  12 The Celestial Bow
    udat!("The Celestial Bow", 133, UniqueBaseItem::LongBow, 2, 3, 1200,
          ItemEffectType::NoMinStr, 0, 0,
          ItemEffectType::DamMod, 2, 2,
          ItemEffectType::SetAC, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  13 Deadly Hunter
    udat!("Deadly Hunter", 108, UniqueBaseItem::CompositeBow, 3, 3, 8750,
          ItemEffectType::TripleDemonDamage, 0, 0,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  14 Bow of the Dead
    udat!("Bow of the Dead", 108, UniqueBaseItem::CompositeBow, 5, 5, 2500,
          ItemEffectType::ToHit, 10, 10,
          ItemEffectType::Dex, 4, 4,
          ItemEffectType::VitCurse, 3, 3,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::SetDur, 30, 30,
          ItemEffectType::Invalid, 0, 0),
    //  15 The Blackoak Bow
    udat!("The Blackoak Bow", 255, UniqueBaseItem::LongBow, 5, 4, 2500,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::VitCurse, 10, 10,
          ItemEffectType::Damage, 50, 50,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  16 Flamedart
    udat!("Flamedart", 255, UniqueBaseItem::HunterBow, 10, 3, 14250,
          ItemEffectType::FireArrows, 1, 6,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::FireRes, 40, 40,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  17 Fleshstinger
    udat!("Fleshstinger", 255, UniqueBaseItem::LongBow, 13, 4, 16500,
          ItemEffectType::Dex, 15, 15,
          ItemEffectType::ToHit, 40, 40,
          ItemEffectType::Damage, 80, 80,
          ItemEffectType::Durability, 6, 6,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  18 Windforce
    udat!("Windforce", 164, UniqueBaseItem::WarBow, 17, 3, 37750,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::Damage, 200, 200,
          ItemEffectType::Knockback, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  19 Eaglehorn
    udat!("Eaglehorn", 108, UniqueBaseItem::BattleBow, 26, 4, 42500,
          ItemEffectType::Dex, 20, 20,
          ItemEffectType::ToHit, 50, 50,
          ItemEffectType::Damage, 100, 100,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  20 Gonnagal's Dirk
    udat!("Gonnagal's Dirk", 54, UniqueBaseItem::Dagger, 1, 4, 7040,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::DamMod, 4, 4,
          ItemEffectType::FastAttack, 2, 2,
          ItemEffectType::FireRes, 25, 25,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  21 The Defender
    udat!("The Defender", 255, UniqueBaseItem::Sabre, 1, 3, 2000,
          ItemEffectType::SetAC, 5, 5,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::ToHitCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  22 Gryphon's Claw
    udat!("Gryphon's Claw", 68, UniqueBaseItem::Falchion, 1, 3, 1000,
          ItemEffectType::Damage, 100, 100,
          ItemEffectType::MagCurse, 2, 2,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  23 Black Razor
    udat!("Black Razor", 53, UniqueBaseItem::Dagger, 1, 3, 2000,
          ItemEffectType::Damage, 150, 150,
          ItemEffectType::Vit, 2, 2,
          ItemEffectType::SetDur, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  24 Gibbous Moon
    udat!("Gibbous Moon", 255, UniqueBaseItem::BroadSword, 2, 4, 6660,
          ItemEffectType::Attribs, 2, 2,
          ItemEffectType::Damage, 25, 25,
          ItemEffectType::Mana, 15, 15,
          ItemEffectType::LightCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  25 Ice Shank
    udat!("Ice Shank", 255, UniqueBaseItem::LongSword, 3, 3, 5250,
          ItemEffectType::FireRes, 40, 40,
          ItemEffectType::SetDur, 15, 15,
          ItemEffectType::Str, 5, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  26 The Executioner's Blade
    udat!("The Executioner's Blade", 58, UniqueBaseItem::Falchion, 3, 4, 7080,
          ItemEffectType::Damage, 150, 150,
          ItemEffectType::LifeCurse, 10, 10,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::Durability, 200, 200,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  27 The Bonesaw
    udat!("The Bonesaw", 255, UniqueBaseItem::Claymore, 6, 6, 4400,
          ItemEffectType::DamMod, 10, 10,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::ManaCurse, 10, 10),
    //  28 Shadowhawk
    udat!("Shadowhawk", 255, UniqueBaseItem::BroadSword, 8, 4, 13750,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::StealLife, 5, 5,
          ItemEffectType::ToHit, 15, 15,
          ItemEffectType::AllRes, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  29 Wizardspike
    udat!("Wizardspike", 50, UniqueBaseItem::Dagger, 11, 4, 12920,
          ItemEffectType::Mag, 15, 15,
          ItemEffectType::Mana, 35, 35,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::AllRes, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  30 Lightsabre
    udat!("Lightsabre", 255, UniqueBaseItem::Sabre, 13, 4, 19150,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::LightDam, 1, 10,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::LightRes, 50, 50,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  31 The Falcon's Talon
    udat!("The Falcon's Talon", 68, UniqueBaseItem::Scimitar, 15, 4, 7867,
          ItemEffectType::FastAttack, 4, 4,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::DamageCurse, 33, 33,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  32 Inferno
    udat!("Inferno", 255, UniqueBaseItem::LongSword, 17, 4, 34600,
          ItemEffectType::FireDam, 2, 12,
          ItemEffectType::Light, 3, 3,
          ItemEffectType::Mana, 20, 20,
          ItemEffectType::FireRes, 80, 80,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  33 Doombringer
    udat!("Doombringer", 255, UniqueBaseItem::BastardSword, 19, 5, 18250,
          ItemEffectType::ToHit, 25, 25,
          ItemEffectType::Damage, 250, 250,
          ItemEffectType::AttribsCurse, 5, 5,
          ItemEffectType::LifeCurse, 25, 25,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::Invalid, 0, 0),
    //  34 The Grizzly
    udat!("The Grizzly", 160, UniqueBaseItem::TwoHandSword, 23, 5, 50000,
          ItemEffectType::Str, 20, 20,
          ItemEffectType::VitCurse, 5, 5,
          ItemEffectType::Damage, 200, 200,
          ItemEffectType::Knockback, 0, 0,
          ItemEffectType::Durability, 100, 100,
          ItemEffectType::Invalid, 0, 0),
    //  35 The Grandfather
    udat!("The Grandfather", 161, UniqueBaseItem::GreatSword, 27, 5, 119800,
          ItemEffectType::OneHand, 0, 0,
          ItemEffectType::Attribs, 5, 5,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::Damage, 70, 70,
          ItemEffectType::Life, 20, 20,
          ItemEffectType::Invalid, 0, 0),
    //  36 The Mangler
    udat!("The Mangler", 144, UniqueBaseItem::LargeAxe, 2, 4, 2850,
          ItemEffectType::Damage, 200, 200,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::ManaCurse, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  37 Sharp Beak
    udat!("Sharp Beak", 143, UniqueBaseItem::LargeAxe, 2, 3, 2850,
          ItemEffectType::Life, 20, 20,
          ItemEffectType::MagCurse, 10, 10,
          ItemEffectType::ManaCurse, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  38 Bloodslayer
    udat!("Bloodslayer", 144, UniqueBaseItem::BroadAxe, 3, 4, 2500,
          ItemEffectType::Damage, 100, 100,
          ItemEffectType::TripleDemonDamage, 0, 0,
          ItemEffectType::AttribsCurse, 5, 5,
          ItemEffectType::SpellLevelAdd, -1, -1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  39 The Celestial Axe
    udat!("The Celestial Axe", 255, UniqueBaseItem::BattleAxe, 4, 4, 14100,
          ItemEffectType::NoMinStr, 0, 0,
          ItemEffectType::ToHit, 15, 15,
          ItemEffectType::Life, 15, 15,
          ItemEffectType::StrCurse, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  40 Wicked Axe
    udat!("Wicked Axe", 143, UniqueBaseItem::LargeAxe, 5, 5, 31150,
          ItemEffectType::ToHit, 30, 30,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::VitCurse, 10, 10,
          ItemEffectType::GetHit, 1, 6,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  41 Stonecleaver
    udat!("Stonecleaver", 104, UniqueBaseItem::BroadAxe, 7, 4, 23900,
          ItemEffectType::Life, 30, 30,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::Damage, 50, 50,
          ItemEffectType::LightRes, 40, 40,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  42 Aguinara's Hatchet
    udat!("Aguinara's Hatchet", 255, UniqueBaseItem::SmallAxe, 12, 3, 24800,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Mag, 10, 10,
          ItemEffectType::MagicRes, 80, 80,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  43 Hellslayer
    udat!("Hellslayer", 255, UniqueBaseItem::BattleAxe, 15, 5, 26200,
          ItemEffectType::Str, 8, 8,
          ItemEffectType::Vit, 8, 8,
          ItemEffectType::Damage, 100, 100,
          ItemEffectType::Life, 25, 25,
          ItemEffectType::ManaCurse, 25, 25,
          ItemEffectType::Invalid, 0, 0),
    //  44 Messerschmidt's Reaver
    udat!("Messerschmidt's Reaver", 163, UniqueBaseItem::GreatAxe, 25, 5, 58000,
          ItemEffectType::Damage, 200, 200,
          ItemEffectType::DamMod, 15, 15,
          ItemEffectType::Attribs, 5, 5,
          ItemEffectType::LifeCurse, 50, 50,
          ItemEffectType::FireDam, 2, 12,
          ItemEffectType::Invalid, 0, 0),
    //  45 Crackrust
    udat!("Crackrust", 255, UniqueBaseItem::Mace, 1, 5, 11375,
          ItemEffectType::Attribs, 2, 2,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::AllRes, 15, 15,
          ItemEffectType::Damage, 50, 50,
          ItemEffectType::SpellLevelAdd, -1, -1,
          ItemEffectType::Invalid, 0, 0),
    //  46 Hammer of Jholm
    udat!("Hammer of Jholm", 255, UniqueBaseItem::Maul, 1, 4, 8700,
          ItemEffectType::SetDam, 4, 10,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Str, 3, 3,
          ItemEffectType::ToHit, 15, 15,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  47 Civerb's Cudgel
    udat!("Civerb's Cudgel", 255, UniqueBaseItem::Mace, 1, 3, 2000,
          ItemEffectType::TripleDemonDamage, 0, 0,
          ItemEffectType::DexCurse, 5, 5,
          ItemEffectType::MagCurse, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  48 The Celestial Star
    udat!("The Celestial Star", 255, UniqueBaseItem::Flail, 2, 4, 7810,
          ItemEffectType::NoMinStr, 0, 0,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::DamMod, 10, 10,
          ItemEffectType::ACCurse, 8, 8,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  49 Baranar's Star
    udat!("Baranar's Star", 255, UniqueBaseItem::MorningStar, 5, 6, 6850,
          ItemEffectType::ToHit, 12, 12,
          ItemEffectType::Damage, 80, 80,
          ItemEffectType::FastAttack, 1, 1,
          ItemEffectType::Vit, 4, 4,
          ItemEffectType::DexCurse, 4, 4,
          ItemEffectType::SetDur, 60, 60),
    //  50 Gnarled Root
    udat!("Gnarled Root", 255, UniqueBaseItem::SpikedClub, 9, 6, 9820,
          ItemEffectType::ToHit, 20, 20,
          ItemEffectType::Damage, 300, 300,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::ACCurse, 10, 10),
    //  51 The Cranium Basher
    udat!("The Cranium Basher", 255, UniqueBaseItem::Maul, 12, 5, 36500,
          ItemEffectType::DamMod, 20, 20,
          ItemEffectType::Str, 15, 15,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::ManaCurse, 150, 150,
          ItemEffectType::AllRes, 5, 5,
          ItemEffectType::Invalid, 0, 0),
    //  52 Schaefer's Hammer
    udat!("Schaefer's Hammer", 255, UniqueBaseItem::WarHammer, 16, 6, 56125,
          ItemEffectType::DamageCurse, 100, 100,
          ItemEffectType::LightDam, 1, 50,
          ItemEffectType::Life, 50, 50,
          ItemEffectType::ToHit, 30, 30,
          ItemEffectType::LightRes, 80, 80,
          ItemEffectType::Light, 1, 1),
    //  53 Dreamflange
    udat!("Dreamflange", 255, UniqueBaseItem::Mace, 26, 5, 26450,
          ItemEffectType::Mag, 30, 30,
          ItemEffectType::Mana, 50, 50,
          ItemEffectType::MagicRes, 50, 50,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    //  54 Staff of Shadows
    udat!("Staff of Shadows", 255, UniqueBaseItem::LongStaff, 2, 5, 1250,
          ItemEffectType::MagCurse, 10, 10,
          ItemEffectType::ToHit, 10, 10,
          ItemEffectType::Damage, 60, 60,
          ItemEffectType::LightCurse, 2, 2,
          ItemEffectType::FastAttack, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    //  55 Immolator
    udat!("Immolator", 255, UniqueBaseItem::LongStaff, 4, 4, 3900,
          ItemEffectType::FireRes, 20, 20,
          ItemEffectType::FireDam, 4, 4,
          ItemEffectType::Mana, 10, 10,
          ItemEffectType::VitCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  56 Storm Spire
    udat!("Storm Spire", 255, UniqueBaseItem::WarStaff, 8, 4, 22500,
          ItemEffectType::LightRes, 50, 50,
          ItemEffectType::LightDam, 2, 8,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::MagCurse, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  57 Gleamsong
    udat!("Gleamsong", 255, UniqueBaseItem::ShortStaff, 8, 4, 6520,
          ItemEffectType::Mana, 25, 25,
          ItemEffectType::StrCurse, 3, 3,
          ItemEffectType::VitCurse, 3, 3,
          ItemEffectType::Spell, 10, 76,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  58 Thundercall
    udat!("Thundercall", 255, UniqueBaseItem::CompositeStaff, 14, 5, 22250,
          ItemEffectType::ToHit, 35, 35,
          ItemEffectType::LightDam, 1, 10,
          ItemEffectType::Spell, 3, 76,
          ItemEffectType::LightRes, 30, 30,
          ItemEffectType::Light, 2, 2,
          ItemEffectType::Invalid, 0, 0),
    //  59 The Protector
    udat!("The Protector", 162, UniqueBaseItem::ShortStaff, 16, 5, 17240,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::GetHit, 5, 5,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::Spell, 2, 86,
          ItemEffectType::Thorns, 1, 3,
          ItemEffectType::Invalid, 0, 0),
    //  60 Naj's Puzzler
    udat!("Naj's Puzzler", 255, UniqueBaseItem::LongStaff, 18, 5, 34000,
          ItemEffectType::Mag, 20, 20,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Spell, 23, 57,
          ItemEffectType::LifeCurse, 25, 25,
          ItemEffectType::Invalid, 0, 0),
    //  61 Mindcry
    udat!("Mindcry", 255, UniqueBaseItem::QuarterStaff, 20, 4, 41500,
          ItemEffectType::Mag, 15, 15,
          ItemEffectType::Spell, 13, 69,
          ItemEffectType::AllRes, 15, 15,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  62 Rod of Onan
    udat!("Rod of Onan", 255, UniqueBaseItem::WarStaff, 22, 3, 44167,
          ItemEffectType::Spell, 21, 50,
          ItemEffectType::Damage, 100, 100,
          ItemEffectType::Attribs, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  63 Helm of Spirits
    udat!("Helm of Spirits", 77, UniqueBaseItem::Helm, 1, 1, 7525,
          ItemEffectType::StealLife, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  64 Thinking Cap
    udat!("Thinking Cap", 93, UniqueBaseItem::SkullCap, 6, 4, 2020,
          ItemEffectType::Mana, 30, 30,
          ItemEffectType::SpellLevelAdd, 2, 2,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::SetDur, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  65 OverLord's Helm
    udat!("OverLord's Helm", 99, UniqueBaseItem::Helm, 7, 5, 12500,
          ItemEffectType::Str, 20, 20,
          ItemEffectType::Dex, 15, 15,
          ItemEffectType::Vit, 5, 5,
          ItemEffectType::MagCurse, 20, 20,
          ItemEffectType::SetDur, 15, 15,
          ItemEffectType::Invalid, 0, 0),
    //  66 Fool's Crest
    udat!("Fool's Crest", 80, UniqueBaseItem::Helm, 12, 4, 10150,
          ItemEffectType::AttribsCurse, 4, 4,
          ItemEffectType::Life, 100, 100,
          ItemEffectType::GetHitCurse, 1, 6,
          ItemEffectType::Thorns, 1, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  67 Gotterdamerung
    udat!("Gotterdamerung", 85, UniqueBaseItem::GreatHelm, 21, 5, 54900,
          ItemEffectType::Attribs, 20, 20,
          ItemEffectType::SetAC, 60, 60,
          ItemEffectType::GetHit, 4, 4,
          ItemEffectType::AllResZero, 0, 0,
          ItemEffectType::LightCurse, 4, 4,
          ItemEffectType::Invalid, 0, 0),
    //  68 Royal Circlet
    udat!("Royal Circlet", 79, UniqueBaseItem::Crown, 27, 4, 24875,
          ItemEffectType::Attribs, 10, 10,
          ItemEffectType::Mana, 40, 40,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::Light, 1, 1,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  69 Torn Flesh of Souls
    udat!("Torn Flesh of Souls", 92, UniqueBaseItem::Rags, 2, 4, 4825,
          ItemEffectType::SetAC, 8, 8,
          ItemEffectType::Vit, 10, 10,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  70 The Gladiator's Bane
    udat!("The Gladiator's Bane", 255, UniqueBaseItem::StuddedArmor, 6, 4, 3450,
          ItemEffectType::SetAC, 25, 25,
          ItemEffectType::GetHit, 2, 2,
          ItemEffectType::Durability, 200, 200,
          ItemEffectType::AttribsCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  71 The Rainbow Cloak
    udat!("The Rainbow Cloak", 138, UniqueBaseItem::Cloak, 2, 5, 4900,
          ItemEffectType::SetAC, 10, 10,
          ItemEffectType::Attribs, 1, 1,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::Life, 5, 5,
          ItemEffectType::Durability, 50, 50,
          ItemEffectType::Invalid, 0, 0),
    //  72 Leather of Aut
    udat!("Leather of Aut", 255, UniqueBaseItem::LeatherArmor, 4, 5, 10550,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::Dex, 5, 5,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  73 Wisdom's Wrap
    udat!("Wisdom's Wrap", 138, UniqueBaseItem::Robe, 5, 5, 6200,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::Mana, 10, 10,
          ItemEffectType::LightRes, 25, 25,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::GetHit, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    //  74 Sparking Mail
    udat!("Sparking Mail", 255, UniqueBaseItem::ChainMail, 9, 2, 15750,
          ItemEffectType::SetAC, 30, 30,
          ItemEffectType::LightDam, 1, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  75 Scavenger Carapace
    udat!("Scavenger Carapace", 255, UniqueBaseItem::BreastPlate, 13, 4, 14000,
          ItemEffectType::GetHit, 15, 15,
          ItemEffectType::ACCurse, 30, 30,
          ItemEffectType::Dex, 5, 5,
          ItemEffectType::LightRes, 40, 40,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  76 Nightscape
    udat!("Nightscape", 138, UniqueBaseItem::Cape, 16, 5, 11600,
          ItemEffectType::FastRecover, 2, 2,
          ItemEffectType::LightCurse, 4, 4,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::Dex, 3, 3,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::Invalid, 0, 0),
    //  77 Naj's Light Plate
    udat!("Naj's Light Plate", 159, UniqueBaseItem::PlateMail, 19, 5, 78700,
          ItemEffectType::NoMinStr, 0, 0,
          ItemEffectType::Mag, 5, 5,
          ItemEffectType::Mana, 20, 20,
          ItemEffectType::AllRes, 20, 20,
          ItemEffectType::SpellLevelAdd, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    //  78 Demonspike Coat
    udat!("Demonspike Coat", 255, UniqueBaseItem::FullPlate, 25, 5, 251175,
          ItemEffectType::SetAC, 100, 100,
          ItemEffectType::GetHit, 6, 6,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::FireRes, 50, 50,
          ItemEffectType::Invalid, 0, 0),
    //  79 The Deflector
    udat!("The Deflector", 255, UniqueBaseItem::Buckler, 1, 4, 1500,
          ItemEffectType::SetAC, 7, 7,
          ItemEffectType::AllRes, 10, 10,
          ItemEffectType::DamageCurse, 20, 20,
          ItemEffectType::ToHitCurse, 5, 5,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  80 Split Skull Shield
    udat!("Split Skull Shield", 116, UniqueBaseItem::Buckler, 1, 5, 2025,
          ItemEffectType::SetAC, 10, 10,
          ItemEffectType::Life, 10, 10,
          ItemEffectType::Str, 2, 2,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::SetDur, 15, 15,
          ItemEffectType::Invalid, 0, 0),
    //  81 Dragon's Breach
    udat!("Dragon's Breach", 117, UniqueBaseItem::KiteShield, 2, 5, 19200,
          ItemEffectType::FireRes, 25, 25,
          ItemEffectType::Str, 5, 5,
          ItemEffectType::SetAC, 20, 20,
          ItemEffectType::MagCurse, 5, 5,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  82 Blackoak Shield
    udat!("Blackoak Shield", 146, UniqueBaseItem::SmallShield, 4, 5, 5725,
          ItemEffectType::Dex, 10, 10,
          ItemEffectType::VitCurse, 10, 10,
          ItemEffectType::SetAC, 18, 18,
          ItemEffectType::LightCurse, 1, 1,
          ItemEffectType::Durability, 150, 150,
          ItemEffectType::Invalid, 0, 0),
    //  83 Holy Defender
    udat!("Holy Defender", 146, UniqueBaseItem::LargeShield, 10, 5, 13800,
          ItemEffectType::SetAC, 15, 15,
          ItemEffectType::GetHit, 2, 2,
          ItemEffectType::FireRes, 20, 20,
          ItemEffectType::Durability, 200, 200,
          ItemEffectType::FastBlock, 1, 1,
          ItemEffectType::Invalid, 0, 0),
    //  84 Stormshield
    udat!("Stormshield", 148, UniqueBaseItem::GothicShield, 24, 6, 49000,
          ItemEffectType::SetAC, 40, 40,
          ItemEffectType::GetHitCurse, 4, 4,
          ItemEffectType::Str, 10, 10,
          ItemEffectType::Indestructible, 0, 0,
          ItemEffectType::FastBlock, 1, 1,
          ItemEffectType::LightRes, 50, 50),
    //  85 Bramble
    udat!("Bramble", 9, UniqueBaseItem::Ring, 1, 3, 1000,
          ItemEffectType::AttribsCurse, 2, 2,
          ItemEffectType::DamMod, 3, 3,
          ItemEffectType::Mana, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  86 Ring of Regha
    udat!("Ring of Regha", 11, UniqueBaseItem::Ring, 1, 5, 4175,
          ItemEffectType::Mag, 10, 10,
          ItemEffectType::MagicRes, 10, 10,
          ItemEffectType::Light, 1, 1,
          ItemEffectType::StrCurse, 3, 3,
          ItemEffectType::DexCurse, 3, 3,
          ItemEffectType::Invalid, 0, 0),
    //  87 The Bleeder
    udat!("The Bleeder", 8, UniqueBaseItem::Ring, 2, 3, 8500,
          ItemEffectType::MagicRes, 20, 20,
          ItemEffectType::Mana, 30, 30,
          ItemEffectType::LifeCurse, 10, 10,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  88 Constricting Ring
    udat!("Constricting Ring", 14, UniqueBaseItem::Ring, 5, 2, 62000,
          ItemEffectType::AllRes, 75, 75,
          ItemEffectType::DrainLife, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
    //  89 Ring of Engagement
    udat!("Ring of Engagement", 13, UniqueBaseItem::Ring, 11, 4, 12476,
          ItemEffectType::GetHit, 1, 2,
          ItemEffectType::Thorns, 1, 3,
          ItemEffectType::SetAC, 5, 5,
          ItemEffectType::TargetAC, 2, 2,
          ItemEffectType::Invalid, 0, 0,
          ItemEffectType::Invalid, 0, 0),
];
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

    /// C++ ``_item_indexes`` values equal the itemdat.tsv row indices:
    /// ``GetItemAttrs(item, IDI_X, lvl)`` does ``AllItemsList[IDI_X]`` (items.cpp:3117)
    /// and the TSV rows 0..47 are authored in enum order, so
    /// ``get_item_data(ItemId::X as usize)`` must return the matching row.
    #[test]
    fn test_item_id_equals_tsv_row() {
        assert_eq!(get_item_data(ItemId::Gold as usize).unwrap().name, "Gold");
        assert_eq!(get_item_data(ItemId::Heal as usize).unwrap().name, "Potion of Healing");
        assert_eq!(get_item_data(ItemId::Mana as usize).unwrap().name, "Potion of Mana");
        assert_eq!(get_item_data(ItemId::Mushroom as usize).unwrap().name, "Black Mushroom");
        assert_eq!(get_item_data(ItemId::Anvil as usize).unwrap().name, "Anvil of Fury");
        // Row indices beyond the named quest/consumable block are base items:
        assert_eq!(get_item_data(118).unwrap().name, "Dagger");
        assert_eq!(get_item_data(71).unwrap().name, "Buckler");
    }

    #[test]
    fn test_item_data_gold() {
        let gold = get_item_data(0).unwrap();
        assert_eq!(gold.name, "Gold");
        assert_eq!(gold.class, ItemClass::Gold);
        assert_eq!(gold.item_type, ItemType::Gold);
    }

    #[test]
    fn test_item_data_healing_potion() {
        // TSV row 77 = Potion of Healing (ItemId::Heal = 24 points at row 24, IDI_HEAL).
        let heal = get_item_data(77).unwrap();
        assert_eq!(heal.name, "Potion of Healing");
        assert_eq!(heal.misc_id, ItemMiscId::Heal);
        assert_eq!(heal.usable, true);
        assert_eq!(heal.value, 50);
    }

    #[test]
    fn test_item_data_weapons() {
        let dagger = get_item_data(118).unwrap();
        assert_eq!(dagger.name, "Dagger");
        assert_eq!(dagger.class, ItemClass::Weapon);
        assert_eq!(dagger.min_damage, 1);
        assert_eq!(dagger.max_damage, 4);

        let two_hand = get_item_data(128).unwrap();
        assert_eq!(two_hand.name, "Two-Handed Sword");
        assert_eq!(two_hand.equip_type, ItemEquipType::TwoHand);
        assert!(two_hand.value > 1000);
    }

    #[test]
    fn test_item_data_armor() {
        let cloak = get_item_data(56).unwrap();
        assert_eq!(cloak.name, "Cloak");
        assert_eq!(cloak.class, ItemClass::Armor);
        assert_eq!(cloak.min_ac, 3);
        assert_eq!(cloak.max_ac, 7);

        let plate = get_item_data(70).unwrap();
        assert_eq!(plate.name, "Full Plate Mail");
        assert_eq!(plate.min_ac, 60);
        assert_eq!(plate.max_ac, 75);
        assert_eq!(plate.min_str, 90);
    }

    #[test]
    fn test_item_data_shields() {
        let buckler = get_item_data(71).unwrap();
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
        // unique_itemdat.tsv row 18 (UniqueItemId::Windforce).
        let windforce = get_unique_item_data(UniqueItemId::Windforce).unwrap();
        assert_eq!(windforce.name, "Windforce");
        assert_eq!(windforce.base_item_id, UniqueBaseItem::WarBow);
        assert_eq!(windforce.min_level, 17);
        assert_eq!(windforce.num_powers, 3);
        assert_eq!(windforce.value, 37750);
        assert_eq!(windforce.powers[0].effect_type, ItemEffectType::Str);
        assert_eq!(windforce.powers[0].param1, 5);
        assert_eq!(windforce.powers[1].effect_type, ItemEffectType::Damage);
        assert_eq!(windforce.powers[2].effect_type, ItemEffectType::Knockback);
    }

    #[test]
    fn test_unique_item_grandfather() {
        // unique_itemdat.tsv row 35 (UniqueItemId::Grandfather).
        let gf = get_unique_item_data(UniqueItemId::Grandfather).unwrap();
        assert_eq!(gf.name, "The Grandfather");
        assert_eq!(gf.base_item_id, UniqueBaseItem::GreatSword);
        assert_eq!(gf.min_level, 27);
        assert_eq!(gf.value, 119800);
        assert_eq!(gf.num_powers, 5);
        assert_eq!(gf.powers[0].effect_type, ItemEffectType::OneHand);
        assert_eq!(gf.powers[1].effect_type, ItemEffectType::Attribs);
        assert_eq!(gf.powers[2].effect_type, ItemEffectType::ToHit);
    }

    #[test]
    fn test_unique_item_arkaines_valor() {
        // unique_itemdat.tsv row 7 (UniqueItemId::ArmorOfVal).
        let arkaine = get_unique_item_data(UniqueItemId::ArmorOfVal).unwrap();
        assert_eq!(arkaine.name, "Arkaine's Valor");
        assert_eq!(arkaine.base_item_id, UniqueBaseItem::ArmorOfValor);
        assert_eq!(arkaine.min_level, 1);
        assert_eq!(arkaine.value, 42000);
        assert_eq!(arkaine.num_powers, 4);
    }

    #[test]
    fn test_unique_item_veil_of_steel() {
        // unique_itemdat.tsv row 6 (UniqueItemId::SteelVeil).
        let veil = get_unique_item_data(UniqueItemId::SteelVeil).unwrap();
        assert_eq!(veil.name, "Veil of Steel");
        assert_eq!(veil.base_item_id, UniqueBaseItem::SteelVeil);
        assert_eq!(veil.min_level, 1);
        assert_eq!(veil.value, 63800);
        assert_eq!(veil.num_powers, 6);
    }

    #[test]
    fn test_unique_item_ring_of_truth() {
        // unique_itemdat.tsv row 4 (UniqueItemId::TRing).
        let ring = get_unique_item_data(UniqueItemId::TRing).unwrap();
        assert_eq!(ring.name, "Ring of Truth");
        assert_eq!(ring.base_item_id, UniqueBaseItem::TRing);
        assert_eq!(ring.value, 9100);
    }

    #[test]
    fn test_unique_item_optic_amulet() {
        // unique_itemdat.tsv row 3 (UniqueItemId::OptAmulet).
        let amulet = get_unique_item_data(UniqueItemId::OptAmulet).unwrap();
        assert_eq!(amulet.name, "Optic Amulet");
        assert_eq!(amulet.base_item_id, UniqueBaseItem::OpticAmulet);
        assert_eq!(amulet.min_level, 1);
        assert_eq!(amulet.value, 9750);
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
