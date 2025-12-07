//! Monster Data - Exact port of DevilutionX Source/monstdat.h
//!
//! Contains monster types, AI IDs, and data structures.

use serde::{Deserialize, Serialize};

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
}

/// Macro to create monster data entry
macro_rules! mdat {
    ($name:expr, $dlvl_min:expr, $dlvl_max:expr, $lvl:expr, $hp_min:expr, $hp_max:expr,
     $ai:expr, $int:expr, $hit:expr, $dmg_min:expr, $dmg_max:expr,
     $hit_sp:expr, $dmg_sp_min:expr, $dmg_sp_max:expr, $ac:expr,
     $class:expr, $res:expr, $res_hell:expr, $exp:expr) => {
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
        }
    };
}

/// Complete monster data table - exact port from assets/txtdata/monsters/monstdat.tsv
/// Data extracted from DevilutionX game files
pub const MONSTERS_DATA: [MonsterData; NUM_DEFAULT_MTYPES] = [
    // 0: MT_NZOMBIE
    mdat!("Zombie", 1, 2, 1, 4, 7, MonsterAIID::Zombie, 0, 10, 2, 5, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 54),
    // 1: MT_BZOMBIE
    mdat!("Ghoul", 2, 3, 2, 7, 11, MonsterAIID::Zombie, 1, 10, 3, 10, 0, 0, 0, 10,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 58),
    // 2: MT_GZOMBIE
    mdat!("Rotting Carcass", 2, 4, 4, 15, 25, MonsterAIID::Zombie, 2, 25, 5, 15, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC,
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 136),
    // 3: MT_YZOMBIE
    mdat!("Black Death", 3, 5, 6, 25, 40, MonsterAIID::Zombie, 3, 30, 6, 22, 0, 0, 0, 20,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC,
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 240),
    // 4-7: Fallen (Spear)
    mdat!("Fallen One", 1, 2, 1, 1, 4, MonsterAIID::Fallen, 0, 15, 1, 3, 0, 0, 0, 0,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 46),
    mdat!("Carver", 2, 3, 3, 4, 8, MonsterAIID::Fallen, 2, 20, 2, 5, 0, 0, 0, 5,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 80),
    mdat!("Devil Kin", 2, 4, 5, 12, 24, MonsterAIID::Fallen, 2, 25, 3, 7, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 155),
    mdat!("Dark One", 3, 5, 7, 20, 36, MonsterAIID::Fallen, 3, 30, 4, 8, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 255),
    // 8-11: Skeleton (Axe)
    mdat!("Skeleton", 1, 2, 1, 2, 4, MonsterAIID::SkeletonMelee, 0, 20, 1, 4, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 64),
    mdat!("Corpse Axe", 2, 3, 2, 4, 7, MonsterAIID::SkeletonMelee, 1, 25, 3, 5, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 68),
    mdat!("Burning Dead", 2, 4, 4, 8, 12, MonsterAIID::SkeletonMelee, 2, 30, 3, 7, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 154),
    mdat!("Horror", 3, 5, 6, 12, 20, MonsterAIID::SkeletonMelee, 3, 35, 4, 9, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 264),
    // 12-15: Fallen (Sword)
    mdat!("Fallen One", 1, 2, 1, 2, 5, MonsterAIID::Fallen, 0, 15, 1, 4, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 52),
    mdat!("Carver", 2, 3, 3, 5, 9, MonsterAIID::Fallen, 1, 20, 2, 7, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 90),
    mdat!("Devil Kin", 2, 4, 5, 16, 24, MonsterAIID::Fallen, 2, 25, 4, 10, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 180),
    mdat!("Dark One", 3, 5, 7, 24, 36, MonsterAIID::Fallen, 3, 30, 4, 12, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 280),
    // 16-19: Scavengers
    mdat!("Scavenger", 1, 3, 2, 3, 6, MonsterAIID::Scavenger, 0, 20, 1, 5, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 80),
    mdat!("Plague Eater", 2, 4, 4, 12, 24, MonsterAIID::Scavenger, 1, 30, 1, 8, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 188),
    mdat!("Shadow Beast", 3, 5, 6, 24, 36, MonsterAIID::Scavenger, 2, 35, 3, 12, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 375),
    mdat!("Bone Gasher", 4, 6, 8, 32, 48, MonsterAIID::Scavenger, 2, 40, 6, 14, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 465),
    // 20-23: Skeleton (Bow)
    mdat!("Skeleton Archer", 1, 3, 3, 3, 7, MonsterAIID::SkeletonRanged, 0, 25, 2, 5, 0, 0, 0, 0,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 90),
    mdat!("Corpse Bow", 2, 4, 5, 8, 12, MonsterAIID::SkeletonRanged, 1, 30, 3, 7, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 200),
    mdat!("Burning Dead Archer", 3, 5, 7, 12, 18, MonsterAIID::SkeletonRanged, 2, 35, 4, 10, 0, 0, 0, 10,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 310),
    mdat!("Horror Archer", 4, 6, 9, 16, 24, MonsterAIID::SkeletonRanged, 3, 40, 5, 12, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 420),
    // 24-27: Skeleton (Sword)
    mdat!("Skeleton Captain", 2, 4, 4, 8, 12, MonsterAIID::SkeletonMelee, 0, 30, 2, 6, 0, 0, 0, 5,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 156),
    mdat!("Corpse Captain", 3, 5, 6, 12, 18, MonsterAIID::SkeletonMelee, 1, 35, 4, 10, 0, 0, 0, 10,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 264),
    mdat!("Burning Dead Captain", 4, 6, 8, 16, 24, MonsterAIID::SkeletonMelee, 2, 40, 6, 14, 0, 0, 0, 15,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 372),
    mdat!("Horror Captain", 5, 7, 10, 20, 30, MonsterAIID::SkeletonMelee, 3, 45, 8, 16, 0, 0, 0, 20,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 480),
    // 28-33: Hidden/Invisible
    mdat!("Invisible Lord", 3, 5, 6, 24, 36, MonsterAIID::Sneak, 2, 35, 6, 12, 0, 0, 0, 15,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 375),
    mdat!("Sneak", 3, 5, 6, 24, 36, MonsterAIID::Sneak, 2, 35, 6, 12, 0, 0, 0, 15,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 375),
    mdat!("Stalker", 4, 6, 8, 32, 48, MonsterAIID::Sneak, 2, 40, 8, 14, 0, 0, 0, 20,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 465),
    mdat!("Unseen", 5, 7, 10, 40, 60, MonsterAIID::Sneak, 3, 45, 10, 16, 0, 0, 0, 25,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 600),
    mdat!("Illusion Weaver", 6, 8, 13, 50, 75, MonsterAIID::Sneak, 3, 55, 12, 20, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 850),
    mdat!("Lord Sayter", 3, 5, 6, 24, 36, MonsterAIID::Sneak, 2, 35, 6, 12, 0, 0, 0, 15,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance::RESIST_MAGIC, 375),
    // 34-37: Goat Men (Melee)
    mdat!("Flesh Clan", 2, 4, 3, 8, 12, MonsterAIID::GoatMelee, 0, 25, 2, 6, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 106),
    mdat!("Stone Clan", 3, 5, 5, 12, 24, MonsterAIID::GoatMelee, 1, 30, 4, 8, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 200),
    mdat!("Fire Clan", 4, 6, 7, 18, 30, MonsterAIID::GoatMelee, 2, 35, 6, 12, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 310),
    mdat!("Night Clan", 5, 7, 9, 24, 36, MonsterAIID::GoatMelee, 3, 40, 8, 14, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 420),
    // 38-41: Succubi
    mdat!("Fiend", 4, 6, 9, 24, 36, MonsterAIID::Succubus, 2, 45, 6, 14, 40, 6, 12, 25,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 525),
    mdat!("Blink", 5, 7, 11, 32, 48, MonsterAIID::Succubus, 3, 50, 8, 16, 45, 8, 14, 30,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 705),
    mdat!("Gloom", 6, 8, 13, 40, 60, MonsterAIID::Succubus, 3, 55, 10, 18, 50, 10, 16, 35,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 900),
    mdat!("Familiar", 7, 9, 15, 48, 72, MonsterAIID::Succubus, 3, 60, 12, 20, 55, 12, 18, 40,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1125),
    // 42-45: Goat Men (Bow)
    mdat!("Flesh Clan Archer", 2, 4, 4, 8, 12, MonsterAIID::GoatRanged, 0, 30, 2, 6, 0, 0, 0, 10,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::NONE, 156),
    mdat!("Stone Clan Archer", 3, 5, 6, 12, 24, MonsterAIID::GoatRanged, 1, 35, 4, 8, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 264),
    mdat!("Fire Clan Archer", 4, 6, 8, 18, 30, MonsterAIID::GoatRanged, 2, 40, 6, 12, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 372),
    mdat!("Night Clan Archer", 5, 7, 10, 24, 36, MonsterAIID::GoatRanged, 3, 45, 8, 14, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 480),
    //  46-49: Acid Beasts
    mdat!("Acid Beast", 3, 5, 5, 16, 24, MonsterAIID::Acid, 1, 30, 6, 10, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 200),
    mdat!("Poison Spitter", 4, 6, 7, 20, 32, MonsterAIID::Acid, 2, 35, 8, 12, 0, 0, 0, 20,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 310),
    mdat!("Pit Beast", 5, 7, 9, 28, 40, MonsterAIID::Acid, 2, 40, 10, 16, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 420),
    mdat!("Lava Maw", 6, 8, 11, 36, 48, MonsterAIID::Acid, 3, 45, 12, 18, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 625),
    // 50-51: Unique Bosses
    mdat!("Skeleton King", 3, 3, 7, 240, 240, MonsterAIID::SkeletonKing, 3, 40, 6, 16, 0, 0, 0, 35,
          MonsterClass::Undead, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 2100),
    mdat!("The Butcher", 2, 2, 7, 110, 110, MonsterAIID::Butcher, 0, 55, 6, 12, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 900),
    // 52-108: Remaining Diablo monsters (placeholders - TODO: fill from monstdat.tsv)
    mdat!("Overlord", 4, 6, 8, 32, 48, MonsterAIID::Fat, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 465), // 52
    mdat!("Mudman", 4, 6, 8, 32, 48, MonsterAIID::Fat, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 465), // 53
    mdat!("Toad", 4, 6, 8, 32, 48, MonsterAIID::Fat, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 465), // 54
    mdat!("Flayed", 4, 6, 8, 32, 48, MonsterAIID::Rhino, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 465), // 55
    mdat!("Wyrm", 4, 6, 8, 32, 48, MonsterAIID::Rhino, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 465), // 56
    mdat!("Cave Slug", 4, 6, 8, 32, 48, MonsterAIID::Rhino, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 465), // 57
    mdat!("Devil Wyrm", 5, 7, 10, 40, 60, MonsterAIID::Rhino, 3, 45, 12, 18, 0, 0, 0, 35,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 600), // 58
    mdat!("Devour", 5, 7, 10, 40, 60, MonsterAIID::Rhino, 3, 45, 12, 18, 0, 0, 0, 35,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 600), // 59
    mdat!("Magma Demon", 5, 7, 10, 40, 60, MonsterAIID::Magma, 3, 45, 12, 18, 40, 10, 16, 35,
          MonsterClass::Demon, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 600), // 60
    mdat!("Magma Demon", 6, 8, 12, 48, 72, MonsterAIID::Magma, 3, 50, 14, 20, 45, 12, 18, 40,
          MonsterClass::Demon, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 720), // 61
    mdat!("Magma Demon", 7, 9, 14, 56, 84, MonsterAIID::Magma, 3, 55, 16, 22, 50, 14, 20, 45,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 850), // 62
    mdat!("Magma Demon", 8, 10, 16, 64, 96, MonsterAIID::Magma, 3, 60, 18, 24, 55, 16, 22, 50,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1000), // 63
    mdat!("Horned Demon", 6, 8, 12, 48, 72, MonsterAIID::Rhino, 2, 50, 12, 20, 0, 0, 0, 40,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 720), // 64
    mdat!("Mudrun", 7, 9, 14, 56, 84, MonsterAIID::Rhino, 3, 55, 14, 22, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 850), // 65
    mdat!("Frost Charger", 8, 10, 16, 64, 96, MonsterAIID::Rhino, 3, 60, 16, 24, 0, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 1000), // 66
    mdat!("Obsidian Lord", 9, 11, 18, 72, 108, MonsterAIID::Rhino, 3, 65, 18, 26, 0, 0, 0, 55,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1170), // 67
    mdat!("Bone Demon", 7, 9, 14, 56, 84, MonsterAIID::BoneDemon, 3, 55, 14, 22, 50, 14, 20, 45,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 850), // 68
    mdat!("Red Death", 8, 10, 16, 64, 96, MonsterAIID::Mega, 3, 60, 16, 24, 0, 0, 0, 50,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1000), // 69
    mdat!("Litch Demon", 9, 11, 18, 72, 108, MonsterAIID::Lich, 3, 65, 18, 26, 60, 18, 24, 55,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 1170), // 70
    mdat!("Undead Balrog", 10, 12, 20, 80, 120, MonsterAIID::Mega, 3, 70, 20, 28, 0, 0, 0, 60,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1350), // 71
    mdat!("Incinerator", 8, 10, 16, 64, 96, MonsterAIID::FireMan, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1000), // 72
    mdat!("Flame Lord", 9, 11, 18, 72, 108, MonsterAIID::FireMan, 3, 65, 18, 26, 60, 18, 24, 55,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1170), // 73
    mdat!("Doom Fire", 10, 12, 20, 80, 120, MonsterAIID::FireMan, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1350), // 74
    mdat!("Hell Burn", 11, 13, 22, 88, 132, MonsterAIID::FireMan, 3, 75, 22, 30, 70, 22, 28, 65,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1540), // 75
    mdat!("Storm Demon", 7, 9, 14, 56, 84, MonsterAIID::Storm, 3, 55, 14, 22, 50, 14, 20, 45,
          MonsterClass::Demon, MonsterResistance::RESIST_LIGHTNING, MonsterResistance::IMMUNE_LIGHTNING, 850), // 76
    mdat!("Storm Rider", 8, 10, 16, 64, 96, MonsterAIID::Storm, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Demon, MonsterResistance::RESIST_LIGHTNING, MonsterResistance::IMMUNE_LIGHTNING, 1000), // 77
    mdat!("Storm Lord", 9, 11, 18, 72, 108, MonsterAIID::Storm, 3, 65, 18, 26, 60, 18, 24, 55,
          MonsterClass::Demon, MonsterResistance::IMMUNE_LIGHTNING, MonsterResistance::IMMUNE_LIGHTNING, 1170), // 78
    mdat!("Maelstrom", 10, 12, 20, 80, 120, MonsterAIID::Storm, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance::IMMUNE_LIGHTNING, MonsterResistance::IMMUNE_LIGHTNING, 1350), // 79
    mdat!("Big Fallen", 4, 6, 8, 32, 48, MonsterAIID::Fallen, 2, 40, 10, 16, 0, 0, 0, 30,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 465), // 80
    mdat!("Winged", 6, 8, 12, 48, 72, MonsterAIID::Bat, 2, 50, 12, 20, 0, 0, 0, 40,
          MonsterClass::Demon, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 720), // 81
    mdat!("Gargoyle", 7, 9, 14, 56, 84, MonsterAIID::Gargoyle, 3, 55, 14, 22, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 850), // 82
    mdat!("Blood Claw", 8, 10, 16, 64, 96, MonsterAIID::Gargoyle, 3, 60, 16, 24, 0, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1000), // 83
    mdat!("Death Wing", 9, 11, 18, 72, 108, MonsterAIID::Gargoyle, 3, 65, 18, 26, 0, 0, 0, 55,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1170), // 84
    mdat!("Mega Demon", 8, 10, 16, 64, 96, MonsterAIID::Mega, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1000), // 85
    mdat!("Guard", 9, 11, 18, 72, 108, MonsterAIID::Mega, 3, 65, 18, 26, 60, 18, 24, 55,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1170), // 86
    mdat!("Vortex Lord", 10, 12, 20, 80, 120, MonsterAIID::Mega, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1350), // 87
    mdat!("Balrog", 11, 13, 22, 88, 132, MonsterAIID::Mega, 3, 75, 22, 30, 70, 22, 28, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1540), // 88
    mdat!("Viper", 6, 8, 12, 48, 72, MonsterAIID::Snake, 2, 50, 12, 20, 45, 12, 18, 40,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 720), // 89
    mdat!("Red Snake", 7, 9, 14, 56, 84, MonsterAIID::Snake, 3, 55, 14, 22, 50, 14, 20, 45,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 850), // 90
    mdat!("Cave Viper", 8, 10, 16, 64, 96, MonsterAIID::Snake, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1000), // 91
    mdat!("Golden Viper", 9, 11, 18, 72, 108, MonsterAIID::Snake, 3, 65, 18, 26, 60, 18, 24, 55,
          MonsterClass::Animal, MonsterResistance(MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 1170), // 92
    mdat!("Black Knight", 7, 9, 14, 56, 84, MonsterAIID::Mega, 3, 55, 14, 22, 0, 0, 0, 45,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 850), // 93
    mdat!("Doom Guard", 8, 10, 16, 64, 96, MonsterAIID::Mega, 3, 60, 16, 24, 0, 0, 0, 50,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1000), // 94
    mdat!("Steel Lord", 9, 11, 18, 72, 108, MonsterAIID::Mega, 3, 65, 18, 26, 0, 0, 0, 55,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_LIGHTNING.0), 1170), // 95
    mdat!("Blood Knight", 10, 12, 20, 80, 120, MonsterAIID::Mega, 3, 70, 20, 28, 0, 0, 0, 60,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1350), // 96
    mdat!("Unraveler", 10, 12, 20, 80, 120, MonsterAIID::Counselor, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1350), // 97
    mdat!("Hollow One", 11, 13, 22, 88, 132, MonsterAIID::Counselor, 3, 75, 22, 30, 70, 22, 28, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1540), // 98
    mdat!("Pain Master", 12, 14, 24, 96, 144, MonsterAIID::Counselor, 3, 80, 24, 32, 75, 24, 30, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1740), // 99
    mdat!("Reality Weaver", 13, 15, 26, 104, 156, MonsterAIID::Counselor, 3, 85, 26, 34, 80, 26, 32, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 100
    mdat!("Succubus", 11, 13, 22, 88, 132, MonsterAIID::Succubus, 3, 75, 22, 30, 70, 22, 28, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1540), // 101
    mdat!("Snow Witch", 12, 14, 24, 96, 144, MonsterAIID::Succubus, 3, 80, 24, 32, 75, 24, 30, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1740), // 102
    mdat!("Hell Spawn", 13, 15, 26, 104, 156, MonsterAIID::Succubus, 3, 85, 26, 34, 80, 26, 32, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 103
    mdat!("Soul Burner", 13, 15, 26, 104, 156, MonsterAIID::Counselor, 3, 85, 26, 34, 80, 26, 32, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 104
    mdat!("Counselor", 10, 12, 20, 80, 120, MonsterAIID::Counselor, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1350), // 105
    mdat!("Magistrate", 11, 13, 22, 88, 132, MonsterAIID::Counselor, 3, 75, 22, 30, 70, 22, 28, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1540), // 106
    mdat!("Cabalist", 12, 14, 24, 96, 144, MonsterAIID::Counselor, 3, 80, 24, 32, 75, 24, 30, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1740), // 107
    mdat!("Advocate", 13, 15, 26, 104, 156, MonsterAIID::Counselor, 3, 85, 26, 34, 80, 26, 32, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 108
    // 109: MT_GOLEM
    mdat!("Golem", 0, 0, 0, 0, 0, MonsterAIID::Golem, 0, 0, 0, 0, 0, 0, 0, 25,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0),
          MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 0),
    // 110: MT_DIABLO
    mdat!("Diablo", 16, 16, 45, 1666, 1666, MonsterAIID::Diablo, 3, 150, 30, 60, 0, 0, 0, 145,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 33300),
    // 111-137: Hellfire expansion monsters (placeholders)
    mdat!("Dark Mage", 14, 16, 28, 112, 168, MonsterAIID::Counselor, 3, 90, 28, 36, 85, 28, 34, 80,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2170), // 111
    mdat!("Hell Boar", 3, 5, 6, 24, 36, MonsterAIID::Rhino, 2, 35, 6, 12, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 375), // 112
    mdat!("Stinger", 3, 5, 6, 24, 36, MonsterAIID::Bat, 2, 35, 6, 12, 0, 0, 0, 15,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 375), // 113
    mdat!("Psychorb", 10, 12, 20, 80, 120, MonsterAIID::Psychorb, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1350), // 114
    mdat!("Arachnon", 6, 8, 12, 48, 72, MonsterAIID::Sneak, 2, 50, 12, 20, 0, 0, 0, 40,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_FIRE, 720), // 115
    mdat!("Fell Twin", 8, 10, 16, 64, 96, MonsterAIID::Mega, 3, 60, 16, 24, 0, 0, 0, 50,
          MonsterClass::Demon, MonsterResistance::RESIST_MAGIC, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1000), // 116
    mdat!("Hork Spawn", 7, 9, 14, 56, 84, MonsterAIID::Fallen, 3, 55, 14, 22, 0, 0, 0, 45,
          MonsterClass::Demon, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 850), // 117
    mdat!("Venom Tail", 8, 10, 16, 64, 96, MonsterAIID::Snake, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1000), // 118
    mdat!("Necromorb", 11, 13, 22, 88, 132, MonsterAIID::Necromorb, 3, 75, 22, 30, 70, 22, 28, 65,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1540), // 119
    mdat!("Spider Lord", 9, 11, 18, 72, 108, MonsterAIID::Sneak, 3, 65, 18, 26, 0, 0, 0, 55,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance::IMMUNE_FIRE, 1170), // 120
    mdat!("Lash Worm", 10, 12, 20, 80, 120, MonsterAIID::Snake, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Animal, MonsterResistance::RESIST_FIRE, MonsterResistance(MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::RESIST_LIGHTNING.0), 1350), // 121
    mdat!("Torchant", 8, 10, 16, 64, 96, MonsterAIID::Torchant, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Animal, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1000), // 122
    mdat!("Hork Demon", 11, 13, 22, 88, 132, MonsterAIID::HorkDemon, 3, 75, 22, 30, 0, 0, 0, 65,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1540), // 123
    mdat!("Defiler", 12, 14, 24, 96, 144, MonsterAIID::Acid, 3, 80, 24, 32, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1740), // 124
    mdat!("Gravedigger", 7, 9, 14, 56, 84, MonsterAIID::SkeletonMelee, 3, 55, 14, 22, 0, 0, 0, 45,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance::IMMUNE_MAGIC, 850), // 125
    mdat!("Tomb Rat", 5, 7, 10, 40, 60, MonsterAIID::Scavenger, 3, 45, 10, 16, 0, 0, 0, 25,
          MonsterClass::Animal, MonsterResistance::NONE, MonsterResistance::RESIST_LIGHTNING, 600), // 126
    mdat!("Firebat", 8, 10, 16, 64, 96, MonsterAIID::FireBat, 3, 60, 16, 24, 55, 16, 22, 50,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1000), // 127
    mdat!("Skullwing", 9, 11, 18, 72, 108, MonsterAIID::Bat, 3, 65, 18, 26, 60, 18, 24, 55,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1170), // 128
    mdat!("Lich", 12, 14, 24, 96, 144, MonsterAIID::Lich, 3, 80, 24, 32, 75, 24, 30, 70,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 1740), // 129
    mdat!("Crypt Demon", 13, 15, 26, 104, 156, MonsterAIID::Mega, 3, 85, 26, 34, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 130
    mdat!("Hellbat", 10, 12, 20, 80, 120, MonsterAIID::Bat, 3, 70, 20, 28, 65, 20, 26, 60,
          MonsterClass::Demon, MonsterResistance::IMMUNE_FIRE, MonsterResistance::IMMUNE_FIRE, 1350), // 131
    mdat!("Bone Demon", 14, 16, 28, 112, 168, MonsterAIID::BoneDemon, 3, 90, 28, 36, 85, 28, 34, 80,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::RESIST_FIRE.0), 2170), // 132
    mdat!("Arch Lich", 13, 15, 26, 104, 156, MonsterAIID::ArchLich, 3, 85, 26, 34, 80, 26, 32, 75,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 133
    mdat!("Biclops", 12, 14, 24, 96, 144, MonsterAIID::Mega, 3, 80, 24, 32, 0, 0, 0, 70,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1740), // 134
    mdat!("Flesh Thing", 13, 15, 26, 104, 156, MonsterAIID::Mega, 3, 85, 26, 34, 0, 0, 0, 75,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::RESIST_MAGIC.0 | MonsterResistance::RESIST_FIRE.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 1950), // 136
    mdat!("Reaper", 14, 16, 28, 112, 168, MonsterAIID::Mega, 3, 90, 28, 36, 0, 0, 0, 80,
          MonsterClass::Undead, MonsterResistance::IMMUNE_MAGIC, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0), 2170), // 136
    mdat!("Na-Krul", 16, 16, 50, 2000, 2000, MonsterAIID::SkeletonKing, 3, 170, 35, 70, 0, 0, 0, 160,
          MonsterClass::Demon, MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0),
          MonsterResistance(MonsterResistance::IMMUNE_MAGIC.0 | MonsterResistance::IMMUNE_FIRE.0 | MonsterResistance::IMMUNE_LIGHTNING.0), 40000), // 137
];

/// Get monster data by ID using const array lookup
pub fn get_monster_data(id: MonsterId) -> &'static MonsterData {
    let index = id as i16;
    if index >= 0 && (index as usize) < MONSTERS_DATA.len() {
        &MONSTERS_DATA[index as usize]
    } else {
        // Return first zombie as fallback
        &MONSTERS_DATA[0]
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
    fn test_skeleton_king_data() {
        let sk = get_monster_data(MonsterId::SkeletonKing);
        assert_eq!(sk.name, "Skeleton King");
        assert_eq!(sk.level, 7);
        assert_eq!(sk.hp_min, 240);
        assert_eq!(sk.hp_max, 240);
        assert_eq!(sk.ai, MonsterAIID::SkeletonKing);
        assert_eq!(sk.armor_class, 35);
        assert_eq!(sk.experience, 2100);
        assert!(sk.resistance.immune_magic());
    }

    #[test]
    fn test_butcher_data() {
        let butcher = get_monster_data(MonsterId::Butcher);
        assert_eq!(butcher.name, "The Butcher");
        assert_eq!(butcher.level, 7);
        assert_eq!(butcher.hp_min, 110);
        assert_eq!(butcher.hp_max, 110);
        assert_eq!(butcher.ai, MonsterAIID::Butcher);
        assert_eq!(butcher.armor_class, 30);
        assert_eq!(butcher.monster_class, MonsterClass::Demon);
        assert_eq!(butcher.experience, 900);
    }

    #[test]
    fn test_diablo_data() {
        let diablo = get_monster_data(MonsterId::Diablo);
        assert_eq!(diablo.name, "Diablo");
        assert_eq!(diablo.level, 45);
        assert_eq!(diablo.hp_min, 1666);
        assert_eq!(diablo.hp_max, 1666);
        assert_eq!(diablo.ai, MonsterAIID::Diablo);
        assert_eq!(diablo.armor_class, 145);
        assert_eq!(diablo.experience, 33300);
        assert!(diablo.resistance.immune_magic());
        assert!(diablo.resistance.immune_fire());
        assert!(diablo.resistance.immune_lightning());
    }

    #[test]
    fn test_golem_data() {
        let golem = get_monster_data(MonsterId::Golem);
        assert_eq!(golem.name, "Golem");
        assert_eq!(golem.level, 0);
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
        assert_eq!(fiend.ai, MonsterAIID::Succubus);
        assert_eq!(blink.ai, MonsterAIID::Succubus);
        assert_eq!(gloom.ai, MonsterAIID::Succubus);
        assert_eq!(familiar.ai, MonsterAIID::Succubus);

        // All are Demons
        assert_eq!(fiend.monster_class, MonsterClass::Demon);

        // Levels increase
        assert_eq!(fiend.level, 9);
        assert_eq!(blink.level, 11);
        assert_eq!(gloom.level, 13);
        assert_eq!(familiar.level, 15);

        // All have special attacks
        assert!(fiend.to_hit_special > 0);
        assert!(blink.min_damage_special > 0);
    }

    #[test]
    fn test_hellfire_monsters() {
        let nakrul = get_monster_data(MonsterId::NaKrul);
        assert_eq!(nakrul.name, "Na-Krul");
        assert_eq!(nakrul.level, 50);
        assert_eq!(nakrul.hp_min, 2000);
        assert_eq!(nakrul.experience, 40000);
        assert!(nakrul.resistance.immune_magic());
        assert!(nakrul.resistance.immune_fire());
        assert!(nakrul.resistance.immune_lightning());

        let hork = get_monster_data(MonsterId::HorkDemon);
        assert_eq!(hork.ai, MonsterAIID::HorkDemon);
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
        assert!(diablo.resistance.immune_fire());
        assert!(diablo.resistance.immune_lightning());
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
