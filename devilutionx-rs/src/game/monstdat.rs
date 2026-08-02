//! Monster Data - Exact port of DevilutionX Source/monstdat.h
//!
//! Contains monster types, AI IDs, and data structures.

use serde::{Deserialize, Serialize};
use super::types::DungeonType;

/// Monster AI ID - exact match of MonsterAIID enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum MonsterAIID {
    Zombie = 0,
    Fat = 1,
    SkeletonMelee = 2,
    SkeletonRanged = 3,
    Scavenger = 4,
    Rhino = 5,
    GoatMelee = 6,
    GoatRanged = 7,
    Fallen = 8,
    Magma = 9,
    SkeletonKing = 10,
    Bat = 11,
    Gargoyle = 12,
    Butcher = 13,
    Succubus = 14,
    Sneak = 15,
    Storm = 16,
    FireMan = 17,
    Gharbad = 18,
    Acid = 19,
    AcidUnique = 20,
    Golem = 21,
    Zhar = 22,
    Snotspill = 23,
    Snake = 24,
    Counselor = 25,
    Mega = 26,
    Diablo = 27,
    Lazarus = 28,
    LazarusSuccubus = 29,
    Lachdanan = 30,
    Warlord = 31,
    FireBat = 32,
    Torchant = 33,
    HorkDemon = 34,
    Lich = 35,
    ArchLich = 36,
    Psychorb = 37,
    Necromorb = 38,
    BoneDemon = 39,
    #[default]
    Invalid = -1,
}

/// Monster class - exact match of MonsterClass enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum MonsterClass {
    #[default]
    Undead = 0,
    Demon = 1,
    Animal = 2,
}

/// Monster resistance flags - exact match of monster_resistance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MonsterResistance(pub u8);

impl MonsterResistance {
    pub const NONE: Self = Self(0);
    pub const RESIST_MAGIC: Self = Self(1 << 0);
    pub const RESIST_FIRE: Self = Self(1 << 1);
    pub const RESIST_LIGHTNING: Self = Self(1 << 2);
    pub const IMMUNE_MAGIC: Self = Self(1 << 3);
    pub const IMMUNE_FIRE: Self = Self(1 << 4);
    pub const IMMUNE_LIGHTNING: Self = Self(1 << 5);
    pub const IMMUNE_ACID: Self = Self(1 << 7);

    pub fn resists_magic(&self) -> bool {
        (self.0 & Self::RESIST_MAGIC.0) != 0
    }

    pub fn resists_fire(&self) -> bool {
        (self.0 & Self::RESIST_FIRE.0) != 0
    }

    pub fn resists_lightning(&self) -> bool {
        (self.0 & Self::RESIST_LIGHTNING.0) != 0
    }

    pub fn immune_magic(&self) -> bool {
        (self.0 & Self::IMMUNE_MAGIC.0) != 0
    }

    pub fn immune_fire(&self) -> bool {
        (self.0 & Self::IMMUNE_FIRE.0) != 0
    }

    pub fn immune_lightning(&self) -> bool {
        (self.0 & Self::IMMUNE_LIGHTNING.0) != 0
    }
}

impl std::ops::BitOr for MonsterResistance {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Monster flags - exact match of monster_flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MonsterFlag(pub u16);

impl MonsterFlag {
    pub const NONE: Self = Self(0);
    pub const HIDDEN: Self = Self(1 << 0);
    pub const LOCK_ANIMATION: Self = Self(1 << 1);
    pub const ALLOW_SPECIAL: Self = Self(1 << 2);
    pub const TARGETS_MONSTER: Self = Self(1 << 4);
    pub const GOLEM: Self = Self(1 << 5);
    pub const QUEST_COMPLETE: Self = Self(1 << 6);
    pub const KNOCKBACK: Self = Self(1 << 7);
    pub const SEARCH: Self = Self(1 << 8);
    pub const CAN_OPEN_DOOR: Self = Self(1 << 9);
    pub const NO_ENEMY: Self = Self(1 << 10);
    pub const BERSERK: Self = Self(1 << 11);
    pub const NO_LIFE_STEAL: Self = Self(1 << 12);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for MonsterFlag {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for MonsterFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Monster mode - exact match of MonsterMode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum MonsterMode {
    #[default]
    Stand = 0,
    MoveNorthwards = 1,
    MoveSouthwards = 2,
    MoveSideways = 3,
    MeleeAttack = 4,
    HitRecovery = 5,
    Death = 6,
    SpecialMeleeAttack = 7,
    FadeIn = 8,
    FadeOut = 9,
    RangedAttack = 10,
    SpecialStand = 11,
    SpecialRangedAttack = 12,
    Delay = 13,
    Charge = 14,
    Petrified = 15,
    Heal = 16,
    Talk = 17,
}

impl MonsterMode {
    pub fn is_moving(&self) -> bool {
        matches!(
            self,
            MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways
        )
    }
}

/// Monster goal - exact match of MonsterGoal enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum MonsterGoal {
    #[default]
    None = 0,
    Normal = 1,
    Retreat = 2,
    Healing = 3,
    Move = 4,
    Attack = 5,
    Inquiring = 6,
    Talking = 7,
}

/// Monster graphic type - exact match of MonsterGraphic enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MonsterGraphic {
    Stand = 0,
    Walk = 1,
    Attack = 2,
    GotHit = 3,
    Death = 4,
    Special = 5,
}

/// Leader relation - exact match of LeaderRelation enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum LeaderRelation {
    #[default]
    None = 0,
    Leashed = 1,
    Separated = 2,
}

/// Unique monster type - exact match of UniqueMonsterType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum UniqueMonsterType {
    Garbud = 0,
    SkeletonKing = 1,
    Zhar = 2,
    SnotSpill = 3,
    Lazarus = 4,
    RedVex = 5,
    BlackJade = 6,
    Lachdan = 7,
    WarlordOfBlood = 8,
    Butcher = 9,
    HorkDemon = 10,
    Defiler = 11,
    NaKrul = 12,
    #[default]
    None = 255,
}

/// Monster type IDs - exact port of _monster_id enum from Source/monstdat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i16)]
pub enum MonsterId {
    // Zombies (0-3)
    ZombieN = 0,        // MT_NZOMBIE
    ZombieB = 1,        // MT_BZOMBIE
    ZombieG = 2,        // MT_GZOMBIE
    ZombieY = 3,        // MT_YZOMBIE

    // Fallen (Spear) (4-7)
    FallenRSpear = 4,   // MT_RFALLSP
    FallenDSpear = 5,   // MT_DFALLSP
    FallenYSpear = 6,   // MT_YFALLSP
    FallenBSpear = 7,   // MT_BFALLSP

    // Skeletons (Axe) (8-11)
    SkeletonAxeW = 8,   // MT_WSKELAX
    SkeletonAxeT = 9,   // MT_TSKELAX
    SkeletonAxeR = 10,  // MT_RSKELAX
    SkeletonAxeX = 11,  // MT_XSKELAX

    // Fallen (Sword) (12-15)
    FallenRSword = 12,  // MT_RFALLSD
    FallenDSword = 13,  // MT_DFALLSD
    FallenYSword = 14,  // MT_YFALLSD
    FallenBSword = 15,  // MT_BFALLSD

    // Scavengers (16-19)
    ScavengerN = 16,    // MT_NSCAV
    ScavengerB = 17,    // MT_BSCAV
    ScavengerW = 18,    // MT_WSCAV
    ScavengerY = 19,    // MT_YSCAV

    // Skeletons (Bow) (20-23)
    SkeletonBowW = 20,  // MT_WSKELBW
    SkeletonBowT = 21,  // MT_TSKELBW
    SkeletonBowR = 22,  // MT_RSKELBW
    SkeletonBowX = 23,  // MT_XSKELBW

    // Skeletons (Sword) (24-27)
    SkeletonSwordW = 24, // MT_WSKELSD
    SkeletonSwordT = 25, // MT_TSKELSD
    SkeletonSwordR = 26, // MT_RSKELSD
    SkeletonSwordX = 27, // MT_XSKELSD

    // Hidden/Invisible (28-33)
    InvisibleLord = 28,  // MT_INVILORD
    Sneak = 29,          // MT_SNEAK
    Stalker = 30,        // MT_STALKER
    Unseen = 31,         // MT_UNSEEN
    IllusionWeaver = 32, // MT_ILLWEAV
    LordSayter = 33,     // MT_LRDSAYTR

    // Goat Men (Melee) (34-37)
    GoatManN = 34,       // MT_NGOATMC
    GoatManB = 35,       // MT_BGOATMC
    GoatManR = 36,       // MT_RGOATMC
    GoatManG = 37,       // MT_GGOATMC

    // Succubi (38-41)
    Fiend = 38,          // MT_FIEND
    Blink = 39,          // MT_BLINK
    Gloom = 40,          // MT_GLOOM
    Familiar = 41,       // MT_FAMILIAR

    // Goat Men (Bow) (42-45)
    GoatArcherN = 42,    // MT_NGOATBW
    GoatArcherB = 43,    // MT_BGOATBW
    GoatArcherR = 44,    // MT_RGOATBW
    GoatArcherG = 45,    // MT_GGOATBW

    // Acid Beasts (46-49)
    AcidBeastN = 46,     // MT_NACID
    AcidBeastR = 47,     // MT_RACID
    AcidBeastB = 48,     // MT_BACID
    AcidBeastX = 49,     // MT_XACID

    // Unique Bosses (50-51)
    SkeletonKing = 50,   // MT_SKING
    Butcher = 51,        // MT_CLEAVER

    // Overlord/Fat (52-56)
    Fat = 52,            // MT_FAT
    Mudman = 53,         // MT_MUDMAN
    Toad = 54,           // MT_TOAD
    Flayed = 55,         // MT_FLAYED
    Wyrm = 56,           // MT_WYRM

    // Cave Slug/Devil Wyrm (57-59)
    CaveSlug = 57,       // MT_CAVSLUG
    DevilWyrm = 58,      // MT_DVLWYRM
    Devour = 59,         // MT_DEVOUR

    // Magma Demons (60-63)
    MagmaN = 60,         // MT_NMAGMA
    MagmaY = 61,         // MT_YMAGMA
    MagmaB = 62,         // MT_BMAGMA
    MagmaW = 63,         // MT_WMAGMA

    // Horned Demons (64-67)
    Horned = 64,         // MT_HORNED
    Mudrun = 65,         // MT_MUDRUN
    FrostCharger = 66,   // MT_FROSTC
    ObsidianLord = 67,   // MT_OBLORD

    // Death/Demon Lords (68-75)
    BoneDemon = 68,      // MT_BONEDMN
    RedDeath = 69,       // MT_REDDTH
    LitchDemon = 70,     // MT_LTCHDMN
    UndeadBalrog = 71,   // MT_UDEDBLRG
    Incinerator = 72,    // MT_INCIN
    FlameLord = 73,      // MT_FLAMLRD
    DoomFire = 74,       // MT_DOOMFIRE
    HellBurn = 75,       // MT_HELLBURN

    // Storm Demons (76-79)
    Storm = 76,          // MT_STORM
    StormR = 77,         // MT_RSTORM
    StormLord = 78,      // MT_STORML
    Maelstrom = 79,      // MT_MAEL

    // Fallen/Gargoyle (80-83)
    BigFallen = 80,      // MT_BIGFALL
    Winged = 81,         // MT_WINGED
    Gargoyle = 82,       // MT_GARGOYLE
    BloodClaw = 83,      // MT_BLOODCLW

    // Death Wing/Mega Demon (84-87)
    DeathWing = 84,      // MT_DEATHW
    Mega = 85,           // MT_MEGA
    Guard = 86,          // MT_GUARD
    VortexLord = 87,     // MT_VTEXLRD

    // Balrog/Snakes (88-91)
    Balrog = 88,         // MT_BALROG
    SnakeN = 89,         // MT_NSNAKE
    SnakeR = 90,         // MT_RSNAKE
    SnakeB = 91,         // MT_BSNAKE

    // Golden Snake/Black Knights (92-95)
    SnakeG = 92,         // MT_GSNAKE
    BlackKnightN = 93,   // MT_NBLACK
    BlackKnightT = 94,   // MT_RTBLACK
    BlackKnightB = 95,   // MT_BTBLACK

    // Red Black Knight/Counselors (96-103)
    BlackKnightR = 96,   // MT_RBLACK
    Unraveler = 97,      // MT_UNRAV
    HollowOne = 98,      // MT_HOLOWONE
    PainMaster = 99,     // MT_PAINMSTR
    RealityWeaver = 100, // MT_REALWEAV
    Succubus = 101,      // MT_SUCCUBUS
    SnowWitch = 102,     // MT_SNOWWICH
    HellSpawn = 103,     // MT_HLSPWN

    // Mages (104-108)
    SoulBurner = 104,    // MT_SOLBRNR
    Counselor = 105,     // MT_COUNSLR
    Magistrate = 106,    // MT_MAGISTR
    Cabalist = 107,      // MT_CABALIST
    Advocate = 108,      // MT_ADVOCATE

    // Player Summon/Final Boss (109-110)
    Golem = 109,         // MT_GOLEM
    Diablo = 110,        // MT_DIABLO

    // Hellfire Expansion Monsters (111-143)
    DarkMage = 111,      // MT_DARKMAGE
    HellBoar = 112,      // MT_HELLBOAR
    Stinger = 113,       // MT_STINGER
    Psychorb = 114,      // MT_PSYCHORB
    Arachnon = 115,      // MT_ARACHNON
    FellTwin = 116,      // MT_FELLTWIN
    HorkSpawn = 117,     // MT_HORKSPWN
    VenomTail = 118,     // MT_VENMTAIL
    Necromorb = 119,     // MT_NECRMORB
    SpiderLord = 120,    // MT_SPIDLORD
    LashWorm = 121,      // MT_LASHWORM
    Torchant = 122,      // MT_TORCHANT
    HorkDemon = 123,     // MT_HORKDMN
    Defiler = 124,       // MT_DEFILER
    GraveDigger = 125,   // MT_GRAVEDIG
    TombRat = 126,       // MT_TOMBRAT
    FireBat = 127,       // MT_FIREBAT
    SkullWing = 128,     // MT_SKLWING
    Lich = 129,          // MT_LICH
    CryptDemon = 130,    // MT_CRYPTDMN
    HellBat = 131,       // MT_HELLBAT
    BoneDemon2 = 132,    // MT_BONEDEMN
    ArchLich = 133,      // MT_ARCHLICH
    Biclops = 134,       // MT_BICLOPS
    FleshThing = 135,    // MT_FLESTHNG
    Reaper = 136,        // MT_REAPER
    NaKrul = 137,        // MT_NAKRUL (Final Hellfire boss)

    #[default]
    Invalid = -1,        // MT_INVALID
}

/// Total default monster types (excluding expansion slots)
pub const NUM_DEFAULT_MTYPES: usize = 138;

/// Maximum monster types (for save game compatibility)
pub const NUM_MAX_MTYPES: usize = 200;

/// Max monsters in game
pub const MAX_MONSTERS: usize = 200;

/// Max monster types per level
pub const MAX_LVL_MTYPES: usize = 24;

/// Monster data structure
#[derive(Debug, Clone)]
pub struct MonsterData {
    pub name: &'static str,
    pub min_dungeon_level: i8,
    pub max_dungeon_level: i8,
    pub level: i8,
    pub hp_min: u16,
    pub hp_max: u16,
    pub ai: MonsterAIID,
    pub intelligence: u8,
    pub to_hit: u8,
    pub min_damage: u8,
    pub max_damage: u8,
    pub to_hit_special: u8,
    pub min_damage_special: u8,
    pub max_damage_special: u8,
    pub armor_class: u8,
    pub monster_class: MonsterClass,
    pub resistance: MonsterResistance,
    pub resistance_hell: MonsterResistance,
    pub experience: u16,
    /// Sprite memory size (C++ `MonsterData::image`, used by `GetLevelMTypes`' image budget).
    pub image: u16,
    /// Frames per animation sequence (C++ `MonsterData::frames[6]`); index = MonsterGraphic.
    pub frames: [i8; 6],
    /// Frame rate per animation sequence (C++ `MonsterData::rate[6]`); index = MonsterGraphic.
    pub rate: [i8; 6],
}

/// Macro to create monster data entry
macro_rules! mdat {
    ($name:expr, $dlvl_min:expr, $dlvl_max:expr, $lvl:expr, $hp_min:expr, $hp_max:expr,
     $ai:expr, $int:expr, $hit:expr, $dmg_min:expr, $dmg_max:expr,
     $hit_sp:expr, $dmg_sp_min:expr, $dmg_sp_max:expr, $ac:expr,
     $class:expr, $res:expr, $res_hell:expr, $exp:expr, $image:expr,
     $frames:expr, $rate:expr) => {
        MonsterData {
            name: $name,
            min_dungeon_level: $dlvl_min,
            max_dungeon_level: $dlvl_max,
            level: $lvl,
            hp_min: $hp_min,
            hp_max: $hp_max,
            ai: $ai,
            intelligence: $int,
            to_hit: $hit,
            min_damage: $dmg_min,
            max_damage: $dmg_max,
            to_hit_special: $hit_sp,
            min_damage_special: $dmg_sp_min,
            max_damage_special: $dmg_sp_max,
            armor_class: $ac,
            monster_class: $class,
            resistance: $res,
            resistance_hell: $res_hell,
            experience: $exp,
            image: $image,
            frames: $frames,
            rate: $rate,
        }
    };
}

/// Complete monster data table - exact port from assets/txtdata/monsters/monstdat.tsv
/// Data extracted from DevilutionX game files
pub const MONSTERS_DATA: [MonsterData; NUM_DEFAULT_MTYPES] = [
    // 0: MT_NZOMBIE
    mdat!("Zombie", 1, 2, 1, 4, 7, MonsterAIID::Zombie, 0, 10, 2, 5, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 54, 799,
          [11,24,12,6,16,0], [4,1,1,1,1,1]),
    // 1: MT_BZOMBIE
    mdat!("Ghoul", 2, 3, 2, 7, 11, MonsterAIID::Zombie, 1, 10, 3, 10, 0, 0, 0, 10,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 58, 799,
          [11,24,12,6,16,0], [4,1,1,1,1,1]),
    // 2: MT_GZOMBIE
    mdat!("Rotting Carcass", 2, 4, 4, 15, 25, MonsterAIID::Zombie, 2, 25, 5, 15, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 136, 799,
          [11,24,12,6,16,0], [4,1,1,1,1,1]),
    // 3: MT_YZOMBIE
    mdat!("Black Death", 3, 5, 6, 25, 40, MonsterAIID::Zombie, 3, 30, 6, 22, 0, 0, 0, 20,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 240, 799,
          [11,24,12,6,16,0], [4,1,1,1,1,1]),
    // 4: MT_RFALLSP
    mdat!("Fallen One", 1, 2, 1, 1, 4, MonsterAIID::Fallen, 0, 15, 1, 3, 0, 0, 0, 0,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 46, 543,
          [11,11,13,11,18,13], [3,1,1,1,1,1]),
    // 5: MT_DFALLSP
    mdat!("Carver", 2, 3, 3, 4, 8, MonsterAIID::Fallen, 2, 20, 2, 5, 0, 0, 0, 5,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 80, 543,
          [11,11,13,11,18,13], [3,1,1,1,1,1]),
    // 6: MT_YFALLSP
    mdat!("Devil Kin", 2, 4, 5, 12, 24, MonsterAIID::Fallen, 2, 25, 3, 7, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 155, 543,
          [11,11,13,11,18,13], [3,1,1,1,1,1]),
    // 7: MT_BFALLSP
    mdat!("Dark One", 3, 5, 7, 20, 36, MonsterAIID::Fallen, 3, 30, 4, 8, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 255, 543,
          [11,11,13,11,18,13], [3,1,1,1,1,1]),
    // 8: MT_WSKELAX
    mdat!("Skeleton", 1, 2, 1, 2, 4, MonsterAIID::SkeletonMelee, 0, 20, 1, 4, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 64, 553,
          [12,8,13,6,17,16], [5,1,1,1,1,1]),
    // 9: MT_TSKELAX
    mdat!("Corpse Axe", 2, 3, 2, 4, 7, MonsterAIID::SkeletonMelee, 1, 25, 3, 5, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 68, 553,
          [12,8,13,6,17,16], [4,1,1,1,1,1]),
    // 10: MT_RSKELAX
    mdat!("Burning Dead", 2, 4, 4, 8, 12, MonsterAIID::SkeletonMelee, 2, 30, 3, 7, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 154, 553,
          [12,8,13,6,17,16], [2,1,1,1,1,1]),
    // 11: MT_XSKELAX
    mdat!("Horror", 3, 5, 6, 12, 20, MonsterAIID::SkeletonMelee, 3, 35, 4, 9, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 264, 553,
          [12,8,13,6,17,16], [3,1,1,1,1,1]),
    // 12: MT_RFALLSD
    mdat!("Fallen One", 1, 2, 1, 2, 5, MonsterAIID::Fallen, 0, 15, 1, 4, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 52, 623,
          [12,12,13,11,14,15], [3,1,1,1,1,1]),
    // 13: MT_DFALLSD
    mdat!("Carver", 2, 3, 3, 5, 9, MonsterAIID::Fallen, 1, 20, 2, 7, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 90, 623,
          [12,12,13,11,14,15], [3,1,1,1,1,1]),
    // 14: MT_YFALLSD
    mdat!("Devil Kin", 2, 4, 5, 16, 24, MonsterAIID::Fallen, 2, 25, 4, 10, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 180, 623,
          [12,12,13,11,14,15], [3,1,1,1,1,1]),
    // 15: MT_BFALLSD
    mdat!("Dark One", 3, 5, 7, 24, 36, MonsterAIID::Fallen, 3, 30, 4, 12, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 280, 623,
          [12,12,13,11,14,15], [3,1,1,1,1,1]),
    // 16: MT_NSCAV
    mdat!("Scavenger", 1, 3, 2, 3, 6, MonsterAIID::Scavenger, 0, 20, 1, 5, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 80, 410,
          [12,8,12,6,20,11], [2,1,1,1,1,1]),
    // 17: MT_BSCAV
    mdat!("Plague Eater", 2, 4, 4, 12, 24, MonsterAIID::Scavenger, 1, 30, 1, 8, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 188, 410,
          [12,8,12,6,20,11], [2,1,1,1,1,1]),
    // 18: MT_WSCAV
    mdat!("Shadow Beast", 3, 5, 6, 24, 36, MonsterAIID::Scavenger, 2, 35, 3, 12, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 375, 410,
          [12,8,12,6,20,11], [2,1,1,1,1,1]),
    // 19: MT_YSCAV
    mdat!("Bone Gasher", 4, 6, 8, 28, 40, MonsterAIID::Scavenger, 3, 35, 5, 15, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_LIGHTNING, 552, 410,
          [12,8,12,6,20,11], [2,1,1,1,1,1]),
    // 20: MT_WSKELBW
    mdat!("Skeleton", 2, 3, 3, 2, 4, MonsterAIID::SkeletonRanged, 0, 15, 1, 2, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 110, 567,
          [9,8,16,5,16,16], [4,1,1,1,1,1]),
    // 21: MT_TSKELBW
    mdat!("Corpse Bow", 2, 4, 5, 8, 16, MonsterAIID::SkeletonRanged, 1, 25, 1, 4, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 210, 567,
          [9,8,16,5,16,16], [4,1,1,1,1,1]),
    // 22: MT_RSKELBW
    mdat!("Burning Dead", 3, 5, 7, 10, 24, MonsterAIID::SkeletonRanged, 2, 30, 1, 6, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 364, 567,
          [9,8,16,5,16,16], [2,1,1,1,1,1]),
    // 23: MT_XSKELBW
    mdat!("Horror", 4, 6, 9, 15, 45, MonsterAIID::SkeletonRanged, 3, 35, 2, 9, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 594, 567,
          [9,8,16,5,16,16], [3,1,1,1,1,1]),
    // 24: MT_WSKELSD
    mdat!("Skeleton Captain", 1, 3, 2, 3, 6, MonsterAIID::SkeletonMelee, 0, 20, 2, 7, 0, 0, 0, 10,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 90, 575,
          [13,8,12,7,15,16], [4,1,1,1,1,1]),
    // 25: MT_TSKELSD
    mdat!("Corpse Captain", 2, 4, 4, 12, 20, MonsterAIID::SkeletonMelee, 1, 30, 3, 9, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 200, 575,
          [13,8,12,7,15,16], [4,1,1,1,1,1]),
    // 26: MT_RSKELSD
    mdat!("Burning Dead Captain", 3, 5, 6, 16, 30, MonsterAIID::SkeletonMelee, 2, 35, 4, 10, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 393, 575,
          [13,8,12,7,15,16], [4,1,1,1,1,1]),
    // 27: MT_XSKELSD
    mdat!("Horror Captain", 4, 6, 8, 35, 50, MonsterAIID::SkeletonMelee, 3, 40, 5, 14, 0, 0, 0, 30,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 604, 575,
          [13,8,12,7,15,16], [4,1,1,1,1,1]),
    // 28: MT_INVILORD
    mdat!("Invisible Lord", 19, 20, 14, 278, 278, MonsterAIID::SkeletonMelee, 3, 65, 16, 30, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 2000, 800,
          [13,13,15,11,16,0], [2,1,1,1,1,1]),
    // 29: MT_SNEAK
    mdat!("Hidden", 2, 5, 5, 8, 24, MonsterAIID::Sneak, 0, 35, 3, 6, 0, 0, 0, 25,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::NONE, 278, 992,
          [16,8,12,8,24,15], [2,1,1,1,1,1]),
    // 30: MT_STALKER
    mdat!("Stalker", 5, 7, 9, 30, 45, MonsterAIID::Sneak, 1, 40, 8, 16, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::NONE, 630, 992,
          [16,8,12,8,24,15], [2,1,1,1,1,1]),
    // 31: MT_UNSEEN
    mdat!("Unseen", 6, 8, 11, 35, 50, MonsterAIID::Sneak, 2, 45, 12, 20, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::IMMUNE_MAGIC, 935, 992,
          [16,8,12,8,24,15], [2,1,1,1,1,1]),
    // 32: MT_ILLWEAV
    mdat!("Illusion Weaver", 8, 10, 13, 40, 60, MonsterAIID::Sneak, 3, 60, 16, 24, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1500, 992,
          [16,8,12,8,24,15], [2,1,1,1,1,1]),
    // 33: MT_LRDSAYTR
    mdat!("Satyr Lord", 21, 22, 28, 160, 200, MonsterAIID::SkeletonMelee, 3, 90, 20, 30, 0, 0, 0, 70,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2800, 800,
          [13,13,14,9,16,0], [2,1,1,1,1,1]),
    // 34: MT_NGOATMC
    mdat!("Flesh Clan", 4, 6, 8, 30, 45, MonsterAIID::GoatMelee, 0, 50, 4, 10, 0, 0, 0, 40,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::NONE, 460, 1030,
          [12,8,12,6,20,12], [2,1,1,1,1,1]),
    // 35: MT_BGOATMC
    mdat!("Stone Clan", 5, 7, 10, 40, 55, MonsterAIID::GoatMelee, 1, 60, 6, 12, 0, 0, 0, 40,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::IMMUNE_MAGIC, 685, 1030,
          [12,8,12,6,20,12], [2,1,1,1,1,1]),
    // 36: MT_RGOATMC
    mdat!("Fire Clan", 6, 8, 12, 50, 65, MonsterAIID::GoatMelee, 2, 70, 8, 16, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 906, 1030,
          [12,8,12,6,20,12], [2,1,1,1,1,1]),
    // 37: MT_GGOATMC
    mdat!("Night Clan", 7, 9, 14, 55, 70, MonsterAIID::GoatMelee, 3, 80, 10, 20, 15, 30, 30, 50,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::IMMUNE_MAGIC, 1190, 1030,
          [12,8,12,6,20,12], [2,1,1,1,1,1]),
    // 38: MT_FIEND
    mdat!("Fiend", 2, 3, 3, 3, 6, MonsterAIID::Bat, 0, 35, 1, 6, 0, 0, 0, 0,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 102, 364,
          [9,13,10,9,13,0], [1,1,1,1,1,1]),
    // 39: MT_BLINK
    mdat!("Blink", 3, 5, 7, 12, 28, MonsterAIID::Bat, 1, 45, 1, 8, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 340, 364,
          [9,13,10,9,13,0], [1,1,1,1,1,1]),
    // 40: MT_GLOOM
    mdat!("Gloom", 4, 6, 9, 28, 36, MonsterAIID::Bat, 2, 70, 4, 12, 0, 0, 0, 35,
          MonsterClass::Animal, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 509, 364,
          [9,13,10,9,13,0], [1,1,1,1,1,1]),
    // 41: MT_FAMILIAR
    mdat!("Familiar", 6, 8, 13, 20, 35, MonsterAIID::Bat, 3, 50, 4, 16, 0, 0, 0, 35,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 448, 364,
          [9,13,10,9,13,0], [1,1,1,1,1,1]),
    // 42: MT_NGOATBW
    mdat!("Flesh Clan", 4, 6, 8, 20, 35, MonsterAIID::GoatRanged, 0, 35, 1, 7, 0, 0, 0, 35,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::NONE, 448, 1040,
          [12,8,16,6,20,0], [3,1,1,1,1,1]),
    // 43: MT_BGOATBW
    mdat!("Stone Clan", 5, 7, 10, 30, 40, MonsterAIID::GoatRanged, 1, 40, 2, 9, 0, 0, 0, 35,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::IMMUNE_MAGIC, 645, 1040,
          [12,8,16,6,20,0], [3,1,1,1,1,1]),
    // 44: MT_RGOATBW
    mdat!("Fire Clan", 6, 8, 12, 40, 50, MonsterAIID::GoatRanged, 2, 45, 3, 11, 0, 0, 0, 35,
          MonsterClass::Demon, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 822, 1040,
          [12,8,16,6,20,0], [3,1,1,1,1,1]),
    // 45: MT_GGOATBW
    mdat!("Night Clan", 7, 9, 14, 50, 65, MonsterAIID::GoatRanged, 3, 50, 4, 13, 15, 0, 0, 40,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::IMMUNE_MAGIC, 1092, 1040,
          [12,8,16,6,20,0], [3,1,1,1,1,1]),
    // 46: MT_NACID
    mdat!("Acid Beast", 6, 8, 11, 40, 66, MonsterAIID::Acid, 0, 40, 4, 12, 25, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::IMMUNE_ACID, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_ACID.0), 846, 716,
          [13,8,12,8,16,12], [1,1,1,1,1,1]),
    // 47: MT_RACID
    mdat!("Poison Spitter", 8, 10, 15, 60, 85, MonsterAIID::Acid, 1, 45, 4, 16, 25, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::IMMUNE_ACID, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_ACID.0), 1248, 716,
          [13,8,12,8,16,12], [1,1,1,1,1,1]),
    // 48: MT_BACID
    mdat!("Pit Beast", 10, 12, 21, 80, 110, MonsterAIID::Acid, 2, 55, 8, 18, 35, 0, 0, 35,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_ACID.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0 | MonsterResistance::IMMUNE_ACID.0), 2060, 716,
          [13,8,12,8,16,12], [1,1,1,1,1,1]),
    // 49: MT_XACID
    mdat!("Lava Maw", 12, 14, 25, 100, 150, MonsterAIID::Acid, 3, 65, 10, 20, 40, 0, 0, 35,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_ACID.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_ACID.0), 2940, 716,
          [13,8,12,8,16,12], [1,1,1,1,1,1]),
    // 50: MT_SKING
    mdat!("Skeleton King", 4, 4, 9, 140, 140, MonsterAIID::SkeletonKing, 3, 60, 6, 16, 0, 0, 0, 70,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 570, 1010,
          [8,6,16,6,16,6], [2,1,1,1,1,2]),
    // 51: MT_CLEAVER
    mdat!("The Butcher", 1, 1, 1, 320, 320, MonsterAIID::Butcher, 3, 50, 6, 12, 0, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 710, 980,
          [10,8,12,6,16,0], [1,1,1,1,1,1]),
    // 52: MT_FAT
    mdat!("Overlord", 5, 7, 10, 60, 80, MonsterAIID::Fat, 0, 55, 6, 12, 0, 0, 0, 55,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 635, 1130,
          [8,10,15,6,16,10], [4,1,1,1,1,1]),
    // 53: MT_MUDMAN
    mdat!("Mud Man", 7, 9, 14, 100, 125, MonsterAIID::Fat, 1, 60, 8, 16, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::IMMUNE_LIGHTNING, 1165, 1130,
          [8,10,15,6,16,10], [4,1,1,1,1,1]),
    // 54: MT_TOAD
    mdat!("Toad Demon", 8, 10, 16, 135, 160, MonsterAIID::Fat, 2, 70, 8, 16, 40, 8, 20, 65,
          MonsterClass::Demon, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 1380, 1130,
          [8,10,15,6,16,10], [4,1,1,1,1,1]),
    // 55: MT_FLAYED
    mdat!("Flayed One", 10, 12, 20, 160, 200, MonsterAIID::Fat, 3, 85, 10, 20, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2058, 1130,
          [8,10,15,6,16,10], [4,1,1,1,1,1]),
    // 56: MT_WYRM
    mdat!("Wyrm", 5, 7, 11, 60, 90, MonsterAIID::SkeletonMelee, 0, 40, 4, 10, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 660, 2420,
          [13,13,13,11,19,0], [1,1,1,1,1,1]),
    // 57: MT_CAVSLUG
    mdat!("Cave Slug", 6, 8, 13, 75, 110, MonsterAIID::SkeletonMelee, 1, 50, 6, 13, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 994, 2420,
          [13,13,13,11,19,0], [1,1,1,1,1,1]),
    // 58: MT_DVLWYRM
    mdat!("Devil Wyrm", 7, 9, 15, 100, 140, MonsterAIID::SkeletonMelee, 2, 55, 8, 16, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1320, 2420,
          [13,13,13,11,19,0], [1,1,1,1,1,1]),
    // 59: MT_DEVOUR
    mdat!("Devourer", 8, 10, 17, 125, 200, MonsterAIID::SkeletonMelee, 3, 60, 10, 20, 0, 0, 0, 35,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1827, 2420,
          [13,13,13,11,19,0], [1,1,1,1,1,1]),
    // 60: MT_NMAGMA
    mdat!("Magma Demon", 8, 9, 13, 50, 70, MonsterAIID::Magma, 0, 45, 2, 10, 50, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1076, 1680,
          [8,10,14,7,18,18], [2,1,1,1,1,1]),
    // 61: MT_YMAGMA
    mdat!("Blood Stone", 8, 10, 14, 55, 75, MonsterAIID::Magma, 1, 50, 2, 12, 50, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1309, 1680,
          [8,10,14,7,18,18], [2,1,1,1,1,1]),
    // 62: MT_BMAGMA
    mdat!("Hell Stone", 9, 11, 16, 60, 80, MonsterAIID::Magma, 2, 60, 2, 20, 60, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1680, 1680,
          [8,10,14,7,18,18], [2,1,1,1,1,1]),
    // 63: MT_WMAGMA
    mdat!("Lava Lord", 9, 11, 18, 70, 85, MonsterAIID::Magma, 3, 75, 4, 24, 60, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2124, 1680,
          [8,10,14,7,18,18], [2,1,1,1,1,1]),
    // 64: MT_HORNED
    mdat!("Horned Demon", 7, 9, 13, 40, 80, MonsterAIID::Rhino, 0, 60, 2, 16, 100, 5, 32, 40,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 1172, 1630,
          [8,8,14,6,16,6], [2,1,1,1,1,1]),
    // 65: MT_MUDRUN
    mdat!("Mud Runner", 8, 10, 15, 50, 90, MonsterAIID::Rhino, 1, 70, 6, 18, 100, 12, 36, 45,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 1404, 1630,
          [8,8,14,6,16,6], [2,1,1,1,1,1]),
    // 66: MT_FROSTC
    mdat!("Frost Charger", 9, 11, 17, 60, 100, MonsterAIID::Rhino, 2, 80, 8, 20, 100, 20, 40, 50,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 1720, 1630,
          [8,8,14,6,16,6], [2,1,1,1,1,1]),
    // 67: MT_OBLORD
    mdat!("Obsidian Lord", 10, 12, 19, 70, 110, MonsterAIID::Rhino, 3, 90, 10, 22, 100, 20, 50, 55,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 1809, 1630,
          [8,8,14,6,16,6], [2,1,1,1,1,1]),
    // 68: MT_BONEDMN
    mdat!("oldboned", 24, 24, 12, 70, 70, MonsterAIID::Storm, 0, 60, 6, 14, 12, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 1344, 1740,
          [10,8,20,6,24,16], [3,1,1,1,1,1]),
    // 69: MT_REDDTH
    mdat!("Red Death", 8, 10, 16, 96, 96, MonsterAIID::Storm, 1, 75, 10, 20, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2168, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 70: MT_LTCHDMN
    mdat!("Litch Demon", 9, 11, 18, 110, 110, MonsterAIID::Storm, 2, 80, 10, 24, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2736, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 71: MT_UDEDBLRG
    mdat!("Undead Balrog", 11, 13, 22, 130, 130, MonsterAIID::Storm, 3, 85, 12, 30, 0, 0, 0, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 3575, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 72: MT_INCIN
    mdat!("Incinerator", 21, 22, 16, 30, 45, MonsterAIID::FireMan, 0, 75, 8, 16, 0, 0, 0, 25,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1888, 1460,
          [14,19,20,8,14,23], [1,1,1,1,1,1]),
    // 73: MT_FLAMLRD
    mdat!("Flame Lord", 22, 23, 18, 40, 55, MonsterAIID::FireMan, 1, 75, 10, 20, 0, 0, 0, 25,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2250, 1460,
          [14,19,20,8,14,23], [1,1,1,1,1,1]),
    // 74: MT_DOOMFIRE
    mdat!("Doom Fire", 23, 24, 20, 50, 65, MonsterAIID::FireMan, 2, 80, 12, 24, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 2740, 1460,
          [14,19,20,8,14,23], [1,1,1,1,1,1]),
    // 75: MT_HELLBURN
    mdat!("Hell Burner", 24, 24, 22, 60, 80, MonsterAIID::FireMan, 3, 85, 15, 30, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 3355, 1460,
          [14,19,20,8,14,23], [1,1,1,1,1,1]),
    // 76: MT_STORM
    mdat!("Red Storm", 9, 11, 18, 55, 110, MonsterAIID::Storm, 0, 80, 8, 18, 75, 4, 16, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2160, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 77: MT_RSTORM
    mdat!("Storm Rider", 10, 12, 20, 60, 120, MonsterAIID::Storm, 1, 80, 8, 18, 80, 4, 16, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2391, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 78: MT_STORML
    mdat!("Storm Lord", 11, 13, 22, 75, 135, MonsterAIID::Storm, 2, 85, 12, 24, 75, 4, 16, 35,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2775, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 79: MT_MAEL
    mdat!("Maelstrom", 12, 14, 24, 90, 150, MonsterAIID::Storm, 3, 90, 12, 28, 75, 4, 16, 40,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 3177, 1740,
          [8,8,18,4,17,14], [3,1,1,1,1,1]),
    // 80: MT_BIGFALL
    mdat!("Devil Kin Brute", 21, 22, 27, 120, 160, MonsterAIID::SkeletonMelee, 3, 100, 18, 24, 0, 0, 0, 70,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 2400, 800,
          [10,8,11,8,17,0], [1,1,1,1,2,2]),
    // 81: MT_WINGED
    mdat!("Winged-Demon", 5, 7, 9, 45, 60, MonsterAIID::Gargoyle, 0, 50, 10, 16, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 662, 1650,
          [14,14,14,10,18,14], [1,1,1,1,1,2]),
    // 82: MT_GARGOYLE
    mdat!("Gargoyle", 7, 9, 13, 60, 90, MonsterAIID::Gargoyle, 1, 65, 10, 16, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 1205, 1650,
          [14,14,14,10,18,14], [1,1,1,1,1,2]),
    // 83: MT_BLOODCLW
    mdat!("Blood Claw", 9, 11, 19, 75, 125, MonsterAIID::Gargoyle, 2, 80, 14, 22, 0, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1873, 1650,
          [14,14,14,10,18,14], [1,1,1,1,1,1]),
    // 84: MT_DEATHW
    mdat!("Death Wing", 10, 12, 23, 90, 150, MonsterAIID::Gargoyle, 3, 95, 16, 28, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2278, 1650,
          [14,14,14,10,18,14], [1,1,1,1,1,1]),
    // 85: MT_MEGA
    mdat!("Slayer", 10, 12, 20, 120, 140, MonsterAIID::Mega, 0, 100, 12, 20, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2300, 2220,
          [6,7,14,1,24,5], [3,1,1,1,2,1]),
    // 86: MT_GUARD
    mdat!("Guardian", 11, 13, 22, 140, 160, MonsterAIID::Mega, 1, 110, 14, 22, 0, 0, 0, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2714, 2220,
          [6,7,14,1,24,5], [3,1,1,1,2,1]),
    // 87: MT_VTEXLRD
    mdat!("Vortex Lord", 12, 14, 24, 160, 180, MonsterAIID::Mega, 2, 120, 18, 24, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 3252, 2220,
          [6,7,14,1,24,5], [3,1,1,1,2,1]),
    // 88: MT_BALROG
    mdat!("Balrog", 13, 15, 26, 180, 200, MonsterAIID::Mega, 3, 130, 22, 30, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 3643, 2220,
          [6,7,14,1,24,5], [3,1,1,1,2,1]),
    // 89: MT_NSNAKE
    mdat!("Cave Viper", 11, 13, 21, 100, 150, MonsterAIID::Snake, 0, 90, 8, 20, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 2725, 1270,
          [12,11,13,5,18,0], [2,1,1,1,1,1]),
    // 90: MT_RSNAKE
    mdat!("Fire Drake", 12, 14, 23, 120, 170, MonsterAIID::Snake, 1, 105, 12, 24, 0, 0, 0, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 3139, 1270,
          [12,11,13,5,18,0], [2,1,1,1,1,1]),
    // 91: MT_BSNAKE
    mdat!("Gold Viper", 13, 14, 25, 140, 180, MonsterAIID::Snake, 2, 120, 15, 26, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 3540, 1270,
          [12,11,13,5,18,0], [2,1,1,1,1,1]),
    // 92: MT_GSNAKE
    mdat!("Azure Drake", 15, 16, 27, 160, 200, MonsterAIID::Snake, 3, 130, 18, 30, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 3791, 1270,
          [12,11,13,5,18,0], [2,1,1,1,1,1]),
    // 93: MT_NBLACK
    mdat!("Black Knight", 12, 14, 24, 150, 150, MonsterAIID::SkeletonMelee, 0, 110, 15, 20, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 3360, 2120,
          [8,8,16,4,24,0], [2,1,1,1,1,1]),
    // 94: MT_RTBLACK
    mdat!("Doom Guard", 13, 15, 26, 165, 165, MonsterAIID::SkeletonMelee, 0, 130, 18, 25, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 3650, 2120,
          [8,8,16,4,24,0], [2,1,1,1,1,1]),
    // 95: MT_BTBLACK
    mdat!("Steel Lord", 14, 16, 28, 180, 180, MonsterAIID::SkeletonMelee, 1, 120, 20, 30, 0, 0, 0, 80,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4252, 2120,
          [8,8,16,4,24,0], [2,1,1,1,1,1]),
    // 96: MT_RBLACK
    mdat!("Blood Knight", 13, 14, 30, 200, 200, MonsterAIID::SkeletonMelee, 1, 130, 25, 35, 0, 0, 0, 85,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 5130, 2120,
          [8,8,16,4,24,0], [2,1,1,1,1,1]),
    // 97: MT_UNRAV
    mdat!("The Shredded", 17, 18, 23, 70, 90, MonsterAIID::SkeletonMelee, 0, 75, 4, 12, 0, 0, 0, 65,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 900, 484,
          [10,10,12,5,16,0], [1,1,1,1,1,1]),
    // 98: MT_HOLOWONE
    mdat!("Hollow One", 18, 19, 27, 135, 240, MonsterAIID::SkeletonMelee, 1, 75, 12, 24, 0, 0, 0, 75,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4374, 484,
          [10,10,12,5,16,0], [1,1,1,1,1,1]),
    // 99: MT_PAINMSTR
    mdat!("Pain Master", 19, 20, 29, 110, 200, MonsterAIID::SkeletonMelee, 2, 80, 16, 30, 0, 0, 0, 80,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 5147, 484,
          [10,10,12,5,16,0], [1,1,1,1,1,1]),
    // 100: MT_REALWEAV
    mdat!("Reality Weaver", 20, 20, 30, 135, 240, MonsterAIID::SkeletonMelee, 3, 85, 20, 35, 0, 0, 0, 85,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 5925, 484,
          [10,10,12,5,16,0], [1,1,1,1,1,1]),
    // 101: MT_SUCCUBUS
    mdat!("Succubus", 12, 14, 24, 120, 150, MonsterAIID::Succubus, 0, 100, 1, 20, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 3696, 980,
          [14,8,16,7,24,0], [1,1,1,1,1,1]),
    // 102: MT_SNOWWICH
    mdat!("Snow Witch", 13, 15, 26, 135, 175, MonsterAIID::Succubus, 1, 110, 1, 24, 0, 0, 0, 65,
          MonsterClass::Demon, MonsterResistance::RESIST_LIGHTNING, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 4084, 980,
          [14,8,16,7,24,0], [1,1,1,1,1,1]),
    // 103: MT_HLSPWN
    mdat!("Hell Spawn", 14, 16, 28, 150, 200, MonsterAIID::Succubus, 2, 115, 1, 30, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4480, 980,
          [14,8,16,7,24,0], [1,1,1,1,1,1]),
    // 104: MT_SOLBRNR
    mdat!("Soul Burner", 15, 16, 30, 140, 225, MonsterAIID::Succubus, 3, 120, 1, 35, 0, 0, 0, 85,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 4644, 980,
          [14,8,16,7,24,0], [1,1,1,1,1,1]),
    // 105: MT_COUNSLR
    mdat!("Counselor", 13, 14, 25, 70, 70, MonsterAIID::Counselor, 0, 90, 8, 20, 0, 0, 0, 0,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4070, 2000,
          [12,1,20,8,28,20], [1,1,1,1,1,1]),
    // 106: MT_MAGISTR
    mdat!("Magistrate", 14, 15, 27, 85, 85, MonsterAIID::Counselor, 1, 100, 10, 24, 0, 0, 0, 0,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4478, 2000,
          [12,1,20,8,28,20], [1,1,1,1,1,1]),
    // 107: MT_CABALIST
    mdat!("Cabalist", 15, 16, 29, 120, 120, MonsterAIID::Counselor, 2, 110, 14, 30, 0, 0, 0, 0,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 4929, 2000,
          [12,1,20,8,28,20], [1,1,1,1,1,1]),
    // 108: MT_ADVOCATE
    mdat!("Advocate", 16, 16, 30, 145, 145, MonsterAIID::Counselor, 3, 120, 15, 25, 0, 0, 0, 0,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 4968, 2000,
          [12,1,20,8,28,20], [1,1,1,1,1,1]),
    // 109: MT_GOLEM
    mdat!("Golem", 1, 1, 12, 1, 1, MonsterAIID::Golem, 0, 0, 1, 1, 0, 0, 0, 1,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance::NONE, 0, 386,
          [0,16,12,0,12,20], [1,1,1,1,1,1]),
    // 110: MT_DIABLO
    mdat!("The Dark Lord", 26, 26, 30, 1666, 1666, MonsterAIID::Diablo, 3, 220, 30, 60, 0, 0, 0, 90,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 31666, 2000,
          [16,6,16,6,16,16], [1,1,1,1,1,1]),
    // 111: MT_DARKMAGE
    mdat!("The Arch-Litch Malignus", 21, 21, 30, 160, 160, MonsterAIID::Counselor, 3, 120, 20, 40, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 4968, 1060,
          [6,1,21,6,23,18], [1,1,1,1,1,1]),
    // 112: MT_HELLBOAR
    mdat!("Hellboar", 17, 18, 23, 80, 100, MonsterAIID::SkeletonMelee, 2, 70, 16, 24, 0, 0, 0, 60,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 750, 800,
          [10,10,15,6,16,0], [2,1,1,1,1,1]),
    // 113: MT_STINGER
    mdat!("Stinger", 17, 18, 22, 30, 40, MonsterAIID::SkeletonMelee, 3, 85, 1, 20, 0, 0, 0, 50,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 500, 305,
          [10,10,12,6,15,0], [2,1,1,1,1,1]),
    // 114: MT_PSYCHORB
    mdat!("Psychorb", 17, 18, 22, 20, 30, MonsterAIID::Psychorb, 3, 80, 10, 10, 0, 0, 0, 40,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 450, 800,
          [12,13,13,7,21,0], [2,1,1,1,1,1]),
    // 115: MT_ARACHNON
    mdat!("Arachnon", 17, 18, 22, 60, 80, MonsterAIID::SkeletonMelee, 3, 50, 5, 15, 0, 0, 0, 50,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 500, 800,
          [12,10,15,6,20,0], [2,1,1,1,1,1]),
    // 116: MT_FELLTWIN
    mdat!("Felltwin", 17, 18, 22, 50, 70, MonsterAIID::SkeletonMelee, 3, 70, 10, 18, 0, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance::NONE, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 600, 800,
          [13,13,15,11,16,0], [2,1,1,1,1,1]),
    // 117: MT_HORKSPWN
    mdat!("Hork Spawn", 18, 19, 22, 30, 30, MonsterAIID::SkeletonMelee, 3, 60, 10, 25, 0, 0, 0, 25,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 250, 520,
          [15,12,14,11,14,0], [1,1,1,1,1,1]),
    // 118: MT_VENMTAIL
    mdat!("Venomtail", 19, 20, 24, 40, 50, MonsterAIID::SkeletonMelee, 3, 85, 1, 30, 0, 0, 0, 60,
          MonsterClass::Animal, MonsterResistance::RESIST_LIGHTNING, MonsterResistance::IMMUNE_LIGHTNING, 1000, 305,
          [10,10,12,6,15,0], [2,1,1,1,1,1]),
    // 119: MT_NECRMORB
    mdat!("Necromorb", 19, 20, 24, 30, 40, MonsterAIID::Necromorb, 3, 80, 20, 20, 0, 0, 0, 50,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1100, 800,
          [12,13,13,7,21,0], [2,1,1,1,1,1]),
    // 120: MT_SPIDLORD
    mdat!("Spider Lord", 19, 20, 24, 80, 100, MonsterAIID::Acid, 3, 60, 8, 20, 75, 10, 10, 60,
          MonsterClass::Animal, MonsterResistance::RESIST_LIGHTNING, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 1250, 800,
          [12,10,15,6,20,10], [2,1,1,1,1,1]),
    // 121: MT_LASHWORM
    mdat!("Lashworm", 19, 20, 20, 30, 30, MonsterAIID::SkeletonMelee, 3, 90, 12, 20, 0, 0, 0, 50,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 600, 800,
          [10,12,15,6,16,0], [1,1,1,1,1,1]),
    // 122: MT_TORCHANT
    mdat!("Torchant", 19, 20, 22, 60, 80, MonsterAIID::Torchant, 3, 75, 20, 30, 0, 0, 0, 70,
          MonsterClass::Animal, MonsterResistance::IMMUNE_FIRE, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1250, 800,
          [14,12,12,6,20,0], [2,1,1,1,1,1]),
    // 123: MT_HORKDMN
    mdat!("Hork Demon", 19, 19, 27, 120, 160, MonsterAIID::SkeletonMelee, 3, 60, 20, 35, 80, 0, 0, 80,
          MonsterClass::Demon, MonsterResistance::RESIST_LIGHTNING, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2000, 800,
          [15,8,16,6,16,9], [2,1,1,1,1,2]),
    // 124: MT_DEFILER
    mdat!("Hell Bug", 20, 20, 30, 240, 240, MonsterAIID::SkeletonMelee, 3, 110, 20, 30, 90, 50, 60, 80,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 5000, 800,
          [8,8,14,6,14,12], [1,1,1,1,1,1]),
    // 125: MT_GRAVEDIG
    mdat!("Gravedigger", 21, 21, 26, 120, 240, MonsterAIID::Scavenger, 3, 80, 2, 12, 0, 0, 0, 20,
          MonsterClass::Undead, MonsterResistance::IMMUNE_LIGHTNING, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 2000, 800,
          [24,24,12,6,16,16], [2,1,1,1,1,1]),
    // 126: MT_TOMBRAT
    mdat!("Tomb Rat", 21, 22, 24, 80, 120, MonsterAIID::SkeletonMelee, 3, 120, 12, 25, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1800, 550,
          [11,8,12,6,20,0], [2,1,1,1,1,1]),
    // 127: MT_FIREBAT
    mdat!("Firebat", 21, 22, 24, 60, 80, MonsterAIID::FireBat, 3, 100, 15, 20, 0, 0, 0, 70,
          MonsterClass::Animal, MonsterResistance::IMMUNE_FIRE, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 2400, 550,
          [18,16,14,6,18,11], [2,1,1,1,1,1]),
    // 128: MT_SKLWING
    mdat!("Skullwing", 21, 22, 27, 70, 70, MonsterAIID::SkeletonMelee, 0, 75, 15, 20, 75, 15, 20, 80,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 3000, 1740,
          [10,8,20,6,24,16], [3,1,1,1,1,1]),
    // 129: MT_LICH
    mdat!("Lich", 21, 22, 25, 80, 100, MonsterAIID::Lich, 3, 100, 15, 20, 0, 0, 0, 60,
          MonsterClass::Undead, MonsterResistance::RESIST_LIGHTNING, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 3000, 800,
          [12,10,10,7,21,0], [2,1,1,1,2,1]),
    // 130: MT_CRYPTDMN
    mdat!("Crypt Demon", 22, 23, 28, 200, 240, MonsterAIID::SkeletonMelee, 3, 100, 20, 40, 0, 0, 0, 85,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 3200, 800,
          [8,18,12,8,21,0], [3,1,1,1,1,1]),
    // 131: MT_HELLBAT
    mdat!("Hellbat", 23, 24, 29, 100, 140, MonsterAIID::Torchant, 3, 110, 30, 30, 0, 0, 0, 80,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 3600, 550,
          [18,16,14,6,18,11], [2,1,1,1,1,1]),
    // 132: MT_BONEDEMN
    mdat!("Bone Demon", 23, 24, 30, 240, 280, MonsterAIID::BoneDemon, 0, 100, 40, 50, 160, 50, 50, 50,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 5000, 1740,
          [10,8,20,6,24,16], [3,1,1,1,1,1]),
    // 133: MT_ARCHLICH
    mdat!("Arch Lich", 23, 24, 30, 180, 200, MonsterAIID::ArchLich, 3, 120, 30, 30, 0, 0, 0, 75,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 4000, 800,
          [12,10,10,7,21,0], [2,1,1,1,2,1]),
    // 134: MT_BICLOPS
    mdat!("Biclops", 23, 24, 30, 200, 240, MonsterAIID::SkeletonMelee, 3, 90, 40, 50, 0, 0, 0, 80,
          MonsterClass::Demon, MonsterResistance::RESIST_LIGHTNING, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4000, 800,
          [10,11,16,6,16,0], [2,1,1,1,2,1]),
    // 135: MT_FLESTHNG
    mdat!("Flesh Thing", 23, 24, 28, 300, 400, MonsterAIID::SkeletonMelee, 3, 150, 12, 18, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 4000, 800,
          [15,24,15,6,16,0], [1,1,1,1,1,1]),
    // 136: MT_REAPER
    mdat!("Reaper", 23, 24, 30, 260, 300, MonsterAIID::SkeletonMelee, 3, 120, 30, 35, 0, 0, 0, 90,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 6000, 800,
          [12,10,14,6,16,0], [2,1,1,1,1,1]),
    // 137: MT_NAKRUL
    mdat!("Na-Krul", 31, 31, 40, 1332, 1332, MonsterAIID::SkeletonMelee, 3, 150, 40, 50, 150, 40, 50, 125,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 13333, 1200,
          [2,6,16,3,16,16], [1,1,1,1,1,1]),
];

/// Get monster data by ID using const array lookup
/// Build a `MonsterId` from its C++ `_monster_id` value (0..NUM_DEFAULT_MTYPES).
///
/// The enum is `#[repr(i16)]` with contiguous discriminants 0..137, so the
/// transmute below is safe for the checked range (mirrors the index cast in
/// `get_monster_data`).
pub fn monster_id_from_index(index: i16) -> Option<MonsterId> {
    if index >= 0 && (index as usize) < NUM_DEFAULT_MTYPES {
        Some(unsafe { std::mem::transmute::<i16, MonsterId>(index) })
    } else {
        None
    }
}

pub fn get_monster_data(id: MonsterId) -> &'static MonsterData {
    let index = id as i16;
    if index >= 0 && (index as usize) < MONSTERS_DATA.len() {
        &MONSTERS_DATA[index as usize]
    } else {
        // Return first zombie as fallback
        &MONSTERS_DATA[0]
    }
}

/// Engine-level summary of a monster's base stats (C++ `InitMonster` inputs).
#[derive(Debug, Clone, Copy)]
pub struct MonsterStats {
    pub hp: i32,
    /// Lower bound of the hit-point range (C++ `hitPointsMinimum`).
    pub hp_min: i32,
    pub min_damage: i32,
    pub max_damage: i32,
    pub armor: i32,
    pub to_hit: i32,
    pub experience: u32,
}

#[allow(non_upper_case_globals)] // legacy simplified-type aliases
impl MonsterId {
    // ------------------------------------------------------------------
    // Legacy simplified-type aliases (original engine `monster.rs` enum).
    // Each maps to its closest C++ `_monster_id`; new code should use the
    // full `MonsterId` variants directly.
    // ------------------------------------------------------------------
    pub const Zombie: Self = Self::ZombieN;
    pub const FallenOne: Self = Self::FallenRSpear;
    pub const Skeleton: Self = Self::SkeletonAxeW;
    pub const SkeletonArcher: Self = Self::SkeletonBowW;
    pub const Scavenger: Self = Self::ScavengerN;
    pub const Ghoul: Self = Self::ZombieB;
    pub const BlackKnight: Self = Self::BlackKnightN;
    pub const Overlord: Self = Self::Fat;
    pub const FlayerDemon: Self = Self::Flayed;
    pub const StormRider: Self = Self::StormR;
    pub const VenomSpitter: Self = Self::AcidBeastR;
    pub const SuccubusBlack: Self = Self::Succubus;
    pub const VileOne: Self = Self::HollowOne;
    pub const MageHell: Self = Self::Counselor;
    /// Arch-Bishop Lazarus: set-level unique boss whose base type is
    /// `MT_ADVOCATE` (unique_monstdat.tsv row 6).
    pub const Lazarus: Self = Self::Advocate;

    /// Base stats from the authoritative `monstdat.tsv` row
    /// (`MonsterData`, C++ `InitMonster`).
    pub fn base_stats(&self) -> MonsterStats {
        let d = get_monster_data(*self);
        MonsterStats {
            hp: d.hp_max as i32,
            hp_min: d.hp_min as i32,
            min_damage: d.min_damage as i32,
            max_damage: d.max_damage as i32,
            armor: d.armor_class as i32,
            to_hit: d.to_hit as i32,
            experience: d.experience as u32,
        }
    }

    /// Display name from `monstdat.tsv` (`MonsterData::name`).
    pub fn name(&self) -> &'static str {
        get_monster_data(*self).name
    }

    /// C++ `MonsterData::monsterClass`.
    pub fn monster_class(&self) -> MonsterClass {
        get_monster_data(*self).monster_class
    }

    /// C++ `MonsterData::ai` (`MonsterAIID`).
    pub fn monster_ai(&self) -> MonsterAIID {
        get_monster_data(*self).ai
    }

    /// True for boss/unique encounters (used to scale aggro range and
    /// intelligence in the simplified spawner; the real game uses
    /// `UniqueMonstersData`).
    pub fn is_boss(&self) -> bool {
        matches!(
            *self,
            Self::Butcher | Self::SkeletonKing | Self::Diablo | Self::NaKrul | Self::Advocate
        )
    }

    /// Per-dungeon monster table (kept from the original simplified engine;
    /// each entry now uses its real `_monster_id`). The C++ game instead
    /// selects per-level in `GetLevelMTypes` using `MonstersData` level ranges.
    pub fn for_dungeon(dungeon_type: DungeonType) -> Vec<MonsterId> {
        match dungeon_type {
            DungeonType::Cathedral => vec![
                MonsterId::ZombieN,
                MonsterId::FallenRSpear,
                MonsterId::SkeletonAxeW,
                MonsterId::SkeletonBowW,
                MonsterId::ScavengerN,
            ],
            DungeonType::Catacombs => vec![
                MonsterId::ZombieB,
                MonsterId::BlackKnightN,
                MonsterId::Gargoyle,
                MonsterId::Fat,
            ],
            DungeonType::Caves => vec![
                MonsterId::Golem,
                MonsterId::Flayed,
                MonsterId::StormR,
                MonsterId::AcidBeastR,
            ],
            DungeonType::Hell => vec![
                MonsterId::Succubus,
                MonsterId::Balrog,
                MonsterId::HollowOne,
                MonsterId::Counselor,
            ],
            _ => vec![MonsterId::ZombieN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_count() {
        assert_eq!(MONSTERS_DATA.len(), NUM_DEFAULT_MTYPES);
        assert_eq!(NUM_DEFAULT_MTYPES, 138);
    }

    #[test]
    fn test_zombie_data() {
        let zombie = get_monster_data(MonsterId::ZombieN);
        assert_eq!(zombie.name, "Zombie");
        assert_eq!(zombie.level, 1);
        assert_eq!(zombie.hp_min, 4);
        assert_eq!(zombie.hp_max, 7);
        assert_eq!(zombie.ai, MonsterAIID::Zombie);
        assert_eq!(zombie.monster_class, MonsterClass::Undead);
        assert_eq!(zombie.experience, 54);
    }

    #[test]
    fn test_image_budget_values() {
        // C++ GetLevelMTypes filters by `image` against the 4000-byte
        // sprite budget (monster.cpp:3483-3493); verify the TSV column
        // landed in the table.
        assert_eq!(get_monster_data(MonsterId::ZombieN).image, 799);
        assert_eq!(get_monster_data(MonsterId::SkeletonKing).image, 1010);
        assert_eq!(get_monster_data(MonsterId::Diablo).image, 2000);
    }

    #[test]
    fn test_skeleton_king_data() {
        let sk = get_monster_data(MonsterId::SkeletonKing);
        assert_eq!(sk.name, "Skeleton King");
        // monstdat.tsv row 50 (MT_SKING).
        assert_eq!(sk.level, 9);
        assert_eq!(sk.hp_min, 140);
        assert_eq!(sk.hp_max, 140);
        assert_eq!(sk.ai, MonsterAIID::SkeletonKing);
        assert_eq!(sk.armor_class, 70);
        assert_eq!(sk.experience, 570);
        assert!(sk.resistance.immune_magic());
    }

    #[test]
    fn test_butcher_data() {
        let butcher = get_monster_data(MonsterId::Butcher);
        assert_eq!(butcher.name, "The Butcher");
        // monstdat.tsv row 51 (MT_CLEAVER).
        assert_eq!(butcher.level, 1);
        assert_eq!(butcher.hp_min, 320);
        assert_eq!(butcher.hp_max, 320);
        assert_eq!(butcher.ai, MonsterAIID::Butcher);
        assert_eq!(butcher.armor_class, 50);
        assert_eq!(butcher.monster_class, MonsterClass::Demon);
        assert_eq!(butcher.experience, 710);
    }

    #[test]
    fn test_diablo_data() {
        let diablo = get_monster_data(MonsterId::Diablo);
        // monstdat.tsv row 110 (MT_DIABLO "The Dark Lord").
        assert_eq!(diablo.name, "The Dark Lord");
        assert_eq!(diablo.level, 30);
        assert_eq!(diablo.hp_min, 1666);
        assert_eq!(diablo.hp_max, 1666);
        assert_eq!(diablo.ai, MonsterAIID::Diablo);
        assert_eq!(diablo.armor_class, 90);
        assert_eq!(diablo.experience, 31666);
        assert!(diablo.resistance.immune_magic());
        assert!(diablo.resistance.resists_fire());
        assert!(diablo.resistance.resists_lightning());
    }

    #[test]
    fn test_golem_data() {
        let golem = get_monster_data(MonsterId::Golem);
        assert_eq!(golem.name, "Golem");
        // monstdat.tsv row 109 (MT_GOLEM): level 12, 1 hp, no XP.
        assert_eq!(golem.level, 12);
        assert_eq!(golem.hp_min, 1);
        assert_eq!(golem.hp_max, 1);
        assert_eq!(golem.ai, MonsterAIID::Golem);
        assert_eq!(golem.monster_class, MonsterClass::Demon);
        // Golem doesn't give XP
        assert_eq!(golem.experience, 0);
    }

    #[test]
    fn test_fallen_variants() {
        let fallen_r = get_monster_data(MonsterId::FallenRSpear);
        let fallen_d = get_monster_data(MonsterId::FallenDSpear);
        let fallen_y = get_monster_data(MonsterId::FallenYSpear);
        let fallen_b = get_monster_data(MonsterId::FallenBSpear);

        // All use Fallen AI
        assert_eq!(fallen_r.ai, MonsterAIID::Fallen);
        assert_eq!(fallen_d.ai, MonsterAIID::Fallen);
        assert_eq!(fallen_y.ai, MonsterAIID::Fallen);
        assert_eq!(fallen_b.ai, MonsterAIID::Fallen);

        // Levels increase
        assert_eq!(fallen_r.level, 1);
        assert_eq!(fallen_d.level, 3);
        assert_eq!(fallen_y.level, 5);
        assert_eq!(fallen_b.level, 7);
    }

    #[test]
    fn test_skeleton_variants() {
        let skel_axe_w = get_monster_data(MonsterId::SkeletonAxeW);
        let skel_bow_w = get_monster_data(MonsterId::SkeletonBowW);
        let skel_sword_w = get_monster_data(MonsterId::SkeletonSwordW);

        // Different AI types
        assert_eq!(skel_axe_w.ai, MonsterAIID::SkeletonMelee);
        assert_eq!(skel_bow_w.ai, MonsterAIID::SkeletonRanged);
        assert_eq!(skel_sword_w.ai, MonsterAIID::SkeletonMelee);

        // All are Undead with magic immunity
        assert_eq!(skel_axe_w.monster_class, MonsterClass::Undead);
        assert!(skel_axe_w.resistance.immune_magic());
        assert!(skel_bow_w.resistance.immune_magic());
        assert!(skel_sword_w.resistance.immune_magic());
    }

    #[test]
    fn test_succubus_variants() {
        let fiend = get_monster_data(MonsterId::Fiend);
        let blink = get_monster_data(MonsterId::Blink);
        let gloom = get_monster_data(MonsterId::Gloom);
        let familiar = get_monster_data(MonsterId::Familiar);

        // All use Succubus AI
        // monstdat.tsv rows 38-41: Fiend/Blink/Gloom/Familiar are Bats (Animal),
        // not Succubi; Familiar is a Demon.
        assert_eq!(fiend.ai, MonsterAIID::Bat);
        assert_eq!(blink.ai, MonsterAIID::Bat);
        assert_eq!(gloom.ai, MonsterAIID::Bat);
        assert_eq!(familiar.ai, MonsterAIID::Bat);

        // All are Demons
        assert_eq!(fiend.monster_class, MonsterClass::Animal);
        assert_eq!(familiar.monster_class, MonsterClass::Demon);

        // Levels increase
        assert_eq!(fiend.level, 3);
        assert_eq!(blink.level, 7);
        assert_eq!(gloom.level, 9);
        assert_eq!(familiar.level, 13);

        // All have special attacks
        // Current monstdat.tsv gives bats no special attacks.
        assert_eq!(fiend.to_hit_special, 0);
        assert_eq!(blink.min_damage_special, 0);
    }

    #[test]
    fn test_hellfire_monsters() {
        let nakrul = get_monster_data(MonsterId::NaKrul);
        // monstdat.tsv row 137 (MT_NAKRUL).
        assert_eq!(nakrul.name, "Na-Krul");
        assert_eq!(nakrul.level, 40);
        assert_eq!(nakrul.hp_min, 1332);
        assert_eq!(nakrul.experience, 13333);
        assert!(nakrul.resistance.immune_magic());
        assert!(nakrul.resistance.immune_fire());
        assert!(nakrul.resistance.resists_lightning());

        let hork = get_monster_data(MonsterId::HorkDemon);
        // monstdat.tsv row 123 (MT_HORKDMN) uses the SkeletonMelee AI.
        assert_eq!(hork.ai, MonsterAIID::SkeletonMelee);
        assert_eq!(hork.monster_class, MonsterClass::Demon);
    }

    #[test]
    fn test_monster_resistances() {
        // Fire immunity
        let magma = get_monster_data(MonsterId::MagmaW);
        assert!(magma.resistance_hell.immune_fire());

        // Lightning immunity
        let storm = get_monster_data(MonsterId::StormLord);
        assert!(storm.resistance_hell.immune_lightning());

        // Multiple immunities
        let diablo = get_monster_data(MonsterId::Diablo);
        assert!(diablo.resistance.immune_magic());
        assert!(diablo.resistance.resists_fire());
        assert!(diablo.resistance.resists_lightning());
    }

    #[test]
    fn test_monster_id_enum_range() {
        // Test first and last valid IDs
        assert_eq!(MonsterId::ZombieN as i16, 0);
        assert_eq!(MonsterId::NaKrul as i16, 137);
        assert_eq!(MonsterId::Invalid as i16, -1);
    }

    #[test]
    fn test_ai_id_mapping() {
        // Verify AI IDs match expected monster types
        let zombie = get_monster_data(MonsterId::ZombieN);
        assert_eq!(zombie.ai, MonsterAIID::Zombie);

        let scav = get_monster_data(MonsterId::ScavengerN);
        assert_eq!(scav.ai, MonsterAIID::Scavenger);

        let goat = get_monster_data(MonsterId::GoatManN);
        assert_eq!(goat.ai, MonsterAIID::GoatMelee);

        let goat_archer = get_monster_data(MonsterId::GoatArcherN);
        assert_eq!(goat_archer.ai, MonsterAIID::GoatRanged);
    }

    #[test]
    fn test_monster_class_distribution() {
        let zombie = get_monster_data(MonsterId::ZombieN);
        assert_eq!(zombie.monster_class, MonsterClass::Undead);

        let fallen = get_monster_data(MonsterId::FallenRSpear);
        assert_eq!(fallen.monster_class, MonsterClass::Animal);

        let succubus = get_monster_data(MonsterId::Succubus);
        assert_eq!(succubus.monster_class, MonsterClass::Demon);
    }
}
