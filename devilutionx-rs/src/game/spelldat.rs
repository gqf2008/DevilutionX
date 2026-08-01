//! Spell Data - Exact port of DevilutionX Source/spelldat.h/cpp
//!
//! Contains spell IDs, missile IDs, spell data structures, and the complete spell data table.

use serde::{Deserialize, Serialize};

/// Spell type - exact match of SpellType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum SpellType {
    #[default]
    Skill = 0,
    Spell = 1,
    Scroll = 2,
    Charges = 3,
    Invalid = 4,
}

/// Spell ID - exact match of SpellID enum from spelldat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum SpellID {
    #[default]
    Null = 0,
    Firebolt = 1,
    Healing = 2,
    Lightning = 3,
    Flash = 4,
    Identify = 5,
    FireWall = 6,
    TownPortal = 7,
    StoneCurse = 8,
    Infravision = 9,
    Phasing = 10,
    ManaShield = 11,
    Fireball = 12,
    Guardian = 13,
    ChainLightning = 14,
    FlameWave = 15,
    DoomSerpents = 16,
    BloodRitual = 17,
    Nova = 18,
    Invisibility = 19,
    Inferno = 20,
    Golem = 21,
    Rage = 22,
    Teleport = 23,
    Apocalypse = 24,
    Etherealize = 25,
    ItemRepair = 26,
    StaffRecharge = 27,
    TrapDisarm = 28,
    Elemental = 29,
    ChargedBolt = 30,
    HolyBolt = 31,
    Resurrect = 32,
    Telekinesis = 33,
    HealOther = 34,
    BloodStar = 35,
    BoneSpirit = 36,
    // Hellfire spells start here
    Mana = 37,
    Magi = 38,
    Jester = 39,
    LightningWall = 40,
    Immolation = 41,
    Warp = 42,
    Reflect = 43,
    Berserk = 44,
    RingOfFire = 45,
    Search = 46,
    RuneOfFire = 47,
    RuneOfLight = 48,
    RuneOfNova = 49,
    RuneOfImmolation = 50,
    RuneOfStone = 51,
    Invalid = -1,
}

impl SpellID {
    /// Total number of spells (52 including Null)
    pub const COUNT: usize = 52;

    /// Last Diablo (non-Hellfire) spell
    pub const LAST_DIABLO: SpellID = SpellID::BoneSpirit;

    /// Check if this is a valid spell
    pub fn is_valid(self) -> bool {
        let id = self as i8;
        id >= 0 && id <= SpellID::RuneOfStone as i8
    }

    /// Check if this is a Hellfire-only spell
    pub fn is_hellfire(self) -> bool {
        let id = self as i8;
        id >= SpellID::Mana as i8 && id <= SpellID::RuneOfStone as i8
    }
}

/// Magic type for damage calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MagicType {
    Fire = 0,
    Lightning = 1,
    Magic = 2,
}

/// Missile ID - exact match of MissileID enum from spelldat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i8)]
pub enum MissileID {
    Arrow = 0,
    Firebolt = 1,
    Guardian = 2,
    Phasing = 3,
    NovaBall = 4,
    FireWall = 5,
    Fireball = 6,
    LightningControl = 7,
    Lightning = 8,
    MagmaBallExplosion = 9,
    TownPortal = 10,
    FlashBottom = 11,
    FlashTop = 12,
    ManaShield = 13,
    FlameWave = 14,
    ChainLightning = 15,
    ChainBall = 16,        // unused
    BloodHit = 17,         // unused
    BoneHit = 18,          // unused
    MetalHit = 19,         // unused
    Rhino = 20,
    MagmaBall = 21,
    ThinLightningControl = 22,
    ThinLightning = 23,
    BloodStar = 24,
    BloodStarExplosion = 25,
    Teleport = 26,
    FireArrow = 27,
    DoomSerpents = 28,     // unused
    FireOnly = 29,         // unused
    StoneCurse = 30,
    BloodRitual = 31,      // unused
    Invisibility = 32,     // unused
    Golem = 33,
    Etherealize = 34,
    Spurt = 35,            // unused
    ApocalypseBoom = 36,
    Healing = 37,
    FireWallControl = 38,
    Infravision = 39,
    Identify = 40,
    FlameWaveControl = 41,
    Nova = 42,
    Rage = 43,
    Apocalypse = 44,
    ItemRepair = 45,
    StaffRecharge = 46,
    TrapDisarm = 47,
    Inferno = 48,
    InfernoControl = 49,
    FireMan = 50,          // unused
    Krull = 51,            // unused
    ChargedBolt = 52,
    HolyBolt = 53,
    Resurrect = 54,
    Telekinesis = 55,
    LightningArrow = 56,
    Acid = 57,
    AcidSplat = 58,
    AcidPuddle = 59,
    HealOther = 60,
    Elemental = 61,
    ResurrectBeam = 62,
    BoneSpirit = 63,
    WeaponExplosion = 64,
    RedPortal = 65,
    DiabloApocalypseBoom = 66,
    DiabloApocalypse = 67,
    // Hellfire missiles
    Mana = 68,
    Magi = 69,
    LightningWall = 70,
    LightningWallControl = 71,
    Immolation = 72,
    SpectralArrow = 73,
    FireballBow = 74,
    LightningBow = 75,
    ChargedBoltBow = 76,
    HolyBoltBow = 77,
    Warp = 78,
    Reflect = 79,
    Berserk = 80,
    RingOfFire = 81,
    StealPotions = 82,
    StealMana = 83,
    RingOfLightning = 84,  // unused
    Search = 85,
    Aura = 86,             // unused
    Aura2 = 87,            // unused
    SpiralFireball = 88,   // unused
    RuneOfFire = 89,
    RuneOfLight = 90,
    RuneOfNova = 91,
    RuneOfImmolation = 92,
    RuneOfStone = 93,
    BigExplosion = 94,
    HorkSpawn = 95,
    Jester = 96,
    OpenNest = 97,
    OrangeFlare = 98,
    BlueFlare = 99,
    RedFlare = 100,
    YellowFlare = 101,
    BlueFlare2 = 102,
    YellowExplosion = 103,
    RedExplosion = 104,
    BlueExplosion = 105,
    BlueExplosion2 = 106,
    OrangeExplosion = 107,
    #[default]
    Null = -1,
}

/// Sound effect ID (simplified, full enum would be very long)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum SfxID {
    None = -1,
    CastFire = 72,
    CastLightning = 73,
    CastSkill = 74,
    CastHealing = 76,
}

/// Spell data flags - exact match of SpellDataFlags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellDataFlags(pub u8);

impl SpellDataFlags {
    // Lower 2 bits store MagicType
    pub const FIRE: Self = Self(0);
    pub const LIGHTNING: Self = Self(1);
    pub const MAGIC: Self = Self(2);
    pub const TARGETED: Self = Self(1 << 2);
    pub const ALLOWED_IN_TOWN: Self = Self(1 << 3);

    pub fn magic_type(&self) -> MagicType {
        match self.0 & 0b11 {
            0 => MagicType::Fire,
            1 => MagicType::Lightning,
            _ => MagicType::Magic,
        }
    }

    pub fn is_targeted(&self) -> bool {
        (self.0 & Self::TARGETED.0) != 0
    }

    pub fn is_allowed_in_town(&self) -> bool {
        (self.0 & Self::ALLOWED_IN_TOWN.0) != 0
    }

    pub fn with_magic_type(magic_type: MagicType) -> Self {
        Self(magic_type as u8)
    }

    pub const fn with_flags(magic_type: MagicType, targeted: bool, allowed_in_town: bool) -> Self {
        let mut flags = magic_type as u8;
        if targeted {
            flags |= Self::TARGETED.0;
        }
        if allowed_in_town {
            flags |= Self::ALLOWED_IN_TOWN.0;
        }
        Self(flags)
    }
}

impl std::ops::BitOr for SpellDataFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Spell data structure - exact match of SpellData struct
#[derive(Debug, Clone)]
pub struct SpellData {
    /// Spell name (text key)
    pub name: &'static str,
    /// Sound effect ID
    pub sfx: SfxID,
    /// Book cost (× 10)
    pub book_cost_10: u16,
    /// Staff cost (× 10)
    pub staff_cost_10: u8,
    /// Mana cost
    pub mana_cost: u8,
    /// Spell flags (magic type + targeted + town allowed)
    pub flags: SpellDataFlags,
    /// Required level to read book
    pub book_level: i8,
    /// Required level to use staff
    pub staff_level: i8,
    /// Minimum intelligence required
    pub min_int: u8,
    /// Missile IDs (up to 2)
    pub missiles: [MissileID; 2],
    /// Mana cost multiplier per spell level
    pub mana_adj: u8,
    /// Minimum mana cost
    pub min_mana: u8,
    /// Staff min charges
    pub staff_min: u8,
    /// Staff max charges
    pub staff_max: u8,
}

impl SpellData {
    /// Get book cost in gold
    pub fn book_cost(&self) -> u32 {
        self.book_cost_10 as u32 * 10
    }

    /// Get staff cost in gold
    pub fn staff_cost(&self) -> u16 {
        self.staff_cost_10 as u16 * 10
    }

    /// Get magic type
    pub fn magic_type(&self) -> MagicType {
        self.flags.magic_type()
    }

    /// Is spell targeted?
    pub fn is_targeted(&self) -> bool {
        self.flags.is_targeted()
    }

    /// Is spell allowed in town?
    pub fn is_allowed_in_town(&self) -> bool {
        self.flags.is_allowed_in_town()
    }
}

/// Complete spell data table - matches Source/spelldat.cpp LoadSpellData()
/// Based on txtdata/spells/spelldat.tsv
pub static SPELLS_DATA: [SpellData; SpellID::COUNT] = [
    // Null spell
    SpellData {
        name: "",
        sfx: SfxID::None,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Fire, false, false),
        book_level: 0,
        staff_level: 0,
        min_int: 0,
        missiles: [MissileID::Null, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 1: Firebolt
    SpellData {
        name: "Firebolt",
        sfx: SfxID::CastFire,
        book_cost_10: 100,
        staff_cost_10: 5,
        mana_cost: 6,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 1,
        staff_level: 1,
        min_int: 15,
        missiles: [MissileID::Firebolt, MissileID::Null],
        mana_adj: 1,
        min_mana: 3,
        staff_min: 40,
        staff_max: 80,
    },
    // 2: Healing
    SpellData {
        name: "Healing",
        sfx: SfxID::CastHealing,
        book_cost_10: 100,
        staff_cost_10: 5,
        mana_cost: 5,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: 1,
        staff_level: 1,
        min_int: 17,
        missiles: [MissileID::Healing, MissileID::Null],
        mana_adj: 3,
        min_mana: 1,
        staff_min: 20,
        staff_max: 40,
    },
    // 3: Lightning
    SpellData {
        name: "Lightning",
        sfx: SfxID::CastLightning,
        book_cost_10: 300,
        staff_cost_10: 15,
        mana_cost: 10,
        flags: SpellDataFlags::with_flags(MagicType::Lightning, true, false),
        book_level: 4,
        staff_level: 3,
        min_int: 20,
        missiles: [MissileID::LightningControl, MissileID::Null],
        mana_adj: 1,
        min_mana: 6,
        staff_min: 20,
        staff_max: 60,
    },
    // 4: Flash
    SpellData {
        name: "Flash",
        sfx: SfxID::CastLightning,
        book_cost_10: 750,
        staff_cost_10: 50,
        mana_cost: 30,
        flags: SpellDataFlags::with_flags(MagicType::Lightning, false, false),
        book_level: 5,
        staff_level: 4,
        min_int: 33,
        missiles: [MissileID::FlashBottom, MissileID::FlashTop],
        mana_adj: 2,
        min_mana: 16,
        staff_min: 20,
        staff_max: 40,
    },
    // 5: Identify
    SpellData {
        name: "Identify",
        sfx: SfxID::CastSkill,
        book_cost_10: 0,
        staff_cost_10: 10,
        mana_cost: 13,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: -1,
        staff_level: -1,
        min_int: 23,
        missiles: [MissileID::Identify, MissileID::Null],
        mana_adj: 2,
        min_mana: 1,
        staff_min: 8,
        staff_max: 12,
    },
    // 6: FireWall
    SpellData {
        name: "Fire Wall",
        sfx: SfxID::CastFire,
        book_cost_10: 600,
        staff_cost_10: 40,
        mana_cost: 28,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 3,
        staff_level: 2,
        min_int: 27,
        missiles: [MissileID::FireWallControl, MissileID::Null],
        mana_adj: 2,
        min_mana: 16,
        staff_min: 8,
        staff_max: 16,
    },
    // 7: TownPortal
    SpellData {
        name: "Town Portal",
        sfx: SfxID::CastSkill,
        book_cost_10: 300,
        staff_cost_10: 20,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: 3,
        staff_level: 3,
        min_int: 20,
        missiles: [MissileID::TownPortal, MissileID::Null],
        mana_adj: 3,
        min_mana: 18,
        staff_min: 8,
        staff_max: 12,
    },
    // 8: StoneCurse
    SpellData {
        name: "Stone Curse",
        sfx: SfxID::CastFire,
        book_cost_10: 1200,
        staff_cost_10: 80,
        mana_cost: 60,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: 6,
        staff_level: 5,
        min_int: 51,
        missiles: [MissileID::StoneCurse, MissileID::Null],
        mana_adj: 3,
        min_mana: 40,
        staff_min: 8,
        staff_max: 16,
    },
    // 9: Infravision
    SpellData {
        name: "Infravision",
        sfx: SfxID::CastHealing,
        book_cost_10: 0,
        staff_cost_10: 60,
        mana_cost: 40,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 36,
        missiles: [MissileID::Infravision, MissileID::Null],
        mana_adj: 5,
        min_mana: 20,
        staff_min: 0,
        staff_max: 0,
    },
    // 10: Phasing
    SpellData {
        name: "Phasing",
        sfx: SfxID::CastFire,
        book_cost_10: 350,
        staff_cost_10: 20,
        mana_cost: 12,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 7,
        staff_level: 6,
        min_int: 39,
        missiles: [MissileID::Phasing, MissileID::Null],
        mana_adj: 2,
        min_mana: 4,
        staff_min: 40,
        staff_max: 80,
    },
    // 11: ManaShield
    SpellData {
        name: "Mana Shield",
        sfx: SfxID::CastFire,
        book_cost_10: 1600,
        staff_cost_10: 120,
        mana_cost: 33,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 6,
        staff_level: 5,
        min_int: 25,
        missiles: [MissileID::ManaShield, MissileID::Null],
        mana_adj: 0,
        min_mana: 33,
        staff_min: 4,
        staff_max: 10,
    },
    // 12: Fireball
    SpellData {
        name: "Fireball",
        sfx: SfxID::CastFire,
        book_cost_10: 800,
        staff_cost_10: 30,
        mana_cost: 16,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 8,
        staff_level: 7,
        min_int: 48,
        missiles: [MissileID::Fireball, MissileID::Null],
        mana_adj: 1,
        min_mana: 10,
        staff_min: 40,
        staff_max: 80,
    },
    // 13: Guardian
    SpellData {
        name: "Guardian",
        sfx: SfxID::CastFire,
        book_cost_10: 1400,
        staff_cost_10: 95,
        mana_cost: 50,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 9,
        staff_level: 8,
        min_int: 61,
        missiles: [MissileID::Guardian, MissileID::Null],
        mana_adj: 2,
        min_mana: 30,
        staff_min: 16,
        staff_max: 32,
    },
    // 14: ChainLightning
    SpellData {
        name: "Chain Lightning",
        sfx: SfxID::CastFire,
        book_cost_10: 1100,
        staff_cost_10: 75,
        mana_cost: 30,
        flags: SpellDataFlags::with_flags(MagicType::Lightning, false, false),
        book_level: 8,
        staff_level: 7,
        min_int: 54,
        missiles: [MissileID::ChainLightning, MissileID::Null],
        mana_adj: 1,
        min_mana: 18,
        staff_min: 20,
        staff_max: 60,
    },
    // 15: FlameWave
    SpellData {
        name: "Flame Wave",
        sfx: SfxID::CastFire,
        book_cost_10: 1000,
        staff_cost_10: 65,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 9,
        staff_level: 8,
        min_int: 54,
        missiles: [MissileID::FlameWaveControl, MissileID::Null],
        mana_adj: 3,
        min_mana: 20,
        staff_min: 20,
        staff_max: 40,
    },
    // 16: DoomSerpents
    SpellData {
        name: "Doom Serpents",
        sfx: SfxID::CastFire,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Lightning, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 0,
        missiles: [MissileID::Null, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 17: BloodRitual
    SpellData {
        name: "Blood Ritual",
        sfx: SfxID::CastFire,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 0,
        missiles: [MissileID::Null, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 18: Nova
    SpellData {
        name: "Nova",
        sfx: SfxID::CastLightning,
        book_cost_10: 2100,
        staff_cost_10: 130,
        mana_cost: 60,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 14,
        staff_level: 10,
        min_int: 87,
        missiles: [MissileID::Nova, MissileID::Null],
        mana_adj: 3,
        min_mana: 35,
        staff_min: 16,
        staff_max: 32,
    },
    // 19: Invisibility
    SpellData {
        name: "Invisibility",
        sfx: SfxID::CastFire,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 0,
        missiles: [MissileID::Null, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 20: Inferno
    SpellData {
        name: "Inferno",
        sfx: SfxID::CastFire,
        book_cost_10: 200,
        staff_cost_10: 10,
        mana_cost: 11,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 3,
        staff_level: 2,
        min_int: 20,
        missiles: [MissileID::InfernoControl, MissileID::Null],
        mana_adj: 1,
        min_mana: 6,
        staff_min: 20,
        staff_max: 40,
    },
    // 21: Golem
    SpellData {
        name: "Golem",
        sfx: SfxID::CastFire,
        book_cost_10: 1800,
        staff_cost_10: 110,
        mana_cost: 100,
        flags: SpellDataFlags::with_flags(MagicType::Fire, true, false),
        book_level: 11,
        staff_level: 9,
        min_int: 81,
        missiles: [MissileID::Golem, MissileID::Null],
        mana_adj: 6,
        min_mana: 60,
        staff_min: 16,
        staff_max: 32,
    },
    // 22: Rage
    SpellData {
        name: "Rage",
        sfx: SfxID::CastHealing,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 15,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 0,
        missiles: [MissileID::Rage, MissileID::Null],
        mana_adj: 1,
        min_mana: 1,
        staff_min: 0,
        staff_max: 0,
    },
    // 23: Teleport
    SpellData {
        name: "Teleport",
        sfx: SfxID::CastSkill,
        book_cost_10: 2000,
        staff_cost_10: 125,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: 14,
        staff_level: 12,
        min_int: 105,
        missiles: [MissileID::Teleport, MissileID::Null],
        mana_adj: 3,
        min_mana: 15,
        staff_min: 16,
        staff_max: 32,
    },
    // 24: Apocalypse
    SpellData {
        name: "Apocalypse",
        sfx: SfxID::CastFire,
        book_cost_10: 3000,
        staff_cost_10: 200,
        mana_cost: 150,
        flags: SpellDataFlags::with_flags(MagicType::Fire, false, false),
        book_level: 19,
        staff_level: 15,
        min_int: 149,
        missiles: [MissileID::Apocalypse, MissileID::Null],
        mana_adj: 6,
        min_mana: 90,
        staff_min: 8,
        staff_max: 12,
    },
    // 25: Etherealize
    SpellData {
        name: "Etherealize",
        sfx: SfxID::CastFire,
        book_cost_10: 2600,
        staff_cost_10: 160,
        mana_cost: 100,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 93,
        missiles: [MissileID::Etherealize, MissileID::Null],
        mana_adj: 0,
        min_mana: 100,
        staff_min: 2,
        staff_max: 6,
    },
    // 26: ItemRepair
    SpellData {
        name: "Item Repair",
        sfx: SfxID::CastSkill,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: -1,
        staff_level: -1,
        min_int: 255,
        missiles: [MissileID::ItemRepair, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 27: StaffRecharge
    SpellData {
        name: "Staff Recharge",
        sfx: SfxID::CastSkill,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: -1,
        staff_level: -1,
        min_int: 255,
        missiles: [MissileID::StaffRecharge, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 28: TrapDisarm
    SpellData {
        name: "Trap Disarm",
        sfx: SfxID::CastSkill,
        book_cost_10: 0,
        staff_cost_10: 0,
        mana_cost: 0,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: -1,
        staff_level: -1,
        min_int: 255,
        missiles: [MissileID::TrapDisarm, MissileID::Null],
        mana_adj: 0,
        min_mana: 0,
        staff_min: 40,
        staff_max: 80,
    },
    // 29: Elemental
    SpellData {
        name: "Elemental",
        sfx: SfxID::CastFire,
        book_cost_10: 1050,
        staff_cost_10: 70,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Fire, false, false),
        book_level: 8,
        staff_level: 6,
        min_int: 68,
        missiles: [MissileID::Elemental, MissileID::Null],
        mana_adj: 2,
        min_mana: 20,
        staff_min: 20,
        staff_max: 60,
    },
    // 30: ChargedBolt
    SpellData {
        name: "Charged Bolt",
        sfx: SfxID::CastFire,
        book_cost_10: 100,
        staff_cost_10: 5,
        mana_cost: 6,
        flags: SpellDataFlags::with_flags(MagicType::Lightning, true, false),
        book_level: 1,
        staff_level: 1,
        min_int: 25,
        missiles: [MissileID::ChargedBolt, MissileID::Null],
        mana_adj: 1,
        min_mana: 6,
        staff_min: 40,
        staff_max: 80,
    },
    // 31: HolyBolt
    SpellData {
        name: "Holy Bolt",
        sfx: SfxID::CastFire,
        book_cost_10: 100,
        staff_cost_10: 5,
        mana_cost: 7,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: 1,
        staff_level: 1,
        min_int: 20,
        missiles: [MissileID::HolyBolt, MissileID::Null],
        mana_adj: 1,
        min_mana: 3,
        staff_min: 40,
        staff_max: 80,
    },
    // 32: Resurrect
    SpellData {
        name: "Resurrect",
        sfx: SfxID::CastHealing,
        book_cost_10: 400,
        staff_cost_10: 25,
        mana_cost: 20,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: -1,
        staff_level: 5,
        min_int: 30,
        missiles: [MissileID::Resurrect, MissileID::Null],
        mana_adj: 0,
        min_mana: 20,
        staff_min: 4,
        staff_max: 10,
    },
    // 33: Telekinesis
    SpellData {
        name: "Telekinesis",
        sfx: SfxID::CastFire,
        book_cost_10: 250,
        staff_cost_10: 20,
        mana_cost: 15,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 2,
        staff_level: 2,
        min_int: 33,
        missiles: [MissileID::Telekinesis, MissileID::Null],
        mana_adj: 2,
        min_mana: 8,
        staff_min: 20,
        staff_max: 40,
    },
    // 34: HealOther
    SpellData {
        name: "Heal Other",
        sfx: SfxID::CastHealing,
        book_cost_10: 100,
        staff_cost_10: 5,
        mana_cost: 5,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: 1,
        staff_level: 1,
        min_int: 17,
        missiles: [MissileID::HealOther, MissileID::Null],
        mana_adj: 3,
        min_mana: 1,
        staff_min: 20,
        staff_max: 40,
    },
    // 35: BloodStar
    SpellData {
        name: "Blood Star",
        sfx: SfxID::CastFire,
        book_cost_10: 2750,
        staff_cost_10: 180,
        mana_cost: 25,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 14,
        staff_level: 13,
        min_int: 70,
        missiles: [MissileID::BloodStar, MissileID::Null],
        mana_adj: 2,
        min_mana: 14,
        staff_min: 20,
        staff_max: 60,
    },
    // 36: BoneSpirit
    SpellData {
        name: "Bone Spirit",
        sfx: SfxID::CastFire,
        book_cost_10: 1150,
        staff_cost_10: 80,
        mana_cost: 24,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 9,
        staff_level: 7,
        min_int: 34,
        missiles: [MissileID::BoneSpirit, MissileID::Null],
        mana_adj: 1,
        min_mana: 12,
        staff_min: 20,
        staff_max: 60,
    },
    // 37: Mana
    SpellData {
        name: "Mana",
        sfx: SfxID::CastHealing,
        book_cost_10: 100,
        staff_cost_10: 5,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: -1,
        staff_level: 5,
        min_int: 17,
        missiles: [MissileID::Mana, MissileID::Null],
        mana_adj: 3,
        min_mana: 1,
        staff_min: 12,
        staff_max: 24,
    },
    // 38: Magi
    SpellData {
        name: "the Magi",
        sfx: SfxID::CastHealing,
        book_cost_10: 10000,
        staff_cost_10: 20,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, true),
        book_level: -1,
        staff_level: 20,
        min_int: 45,
        missiles: [MissileID::Magi, MissileID::Null],
        mana_adj: 3,
        min_mana: 1,
        staff_min: 15,
        staff_max: 30,
    },
    // 39: Jester
    SpellData {
        name: "the Jester",
        sfx: SfxID::CastHealing,
        book_cost_10: 10000,
        staff_cost_10: 20,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: -1,
        staff_level: 4,
        min_int: 30,
        missiles: [MissileID::Jester, MissileID::Null],
        mana_adj: 3,
        min_mana: 1,
        staff_min: 15,
        staff_max: 30,
    },
    // 40: LightningWall
    SpellData {
        name: "Lightning Wall",
        sfx: SfxID::CastLightning,
        book_cost_10: 600,
        staff_cost_10: 40,
        mana_cost: 28,
        flags: SpellDataFlags::with_flags(MagicType::Lightning, true, false),
        book_level: 3,
        staff_level: 2,
        min_int: 27,
        missiles: [MissileID::LightningWallControl, MissileID::Null],
        mana_adj: 2,
        min_mana: 16,
        staff_min: 8,
        staff_max: 16,
    },
    // 41: Immolation
    SpellData {
        name: "Immolation",
        sfx: SfxID::CastFire,
        book_cost_10: 2100,
        staff_cost_10: 130,
        mana_cost: 60,
        flags: SpellDataFlags::with_flags(MagicType::Fire, false, false),
        book_level: 14,
        staff_level: 10,
        min_int: 87,
        missiles: [MissileID::Immolation, MissileID::Null],
        mana_adj: 3,
        min_mana: 35,
        staff_min: 16,
        staff_max: 32,
    },
    // 42: Warp
    SpellData {
        name: "Warp",
        sfx: SfxID::CastSkill,
        book_cost_10: 300,
        staff_cost_10: 20,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 3,
        staff_level: 3,
        min_int: 25,
        missiles: [MissileID::Warp, MissileID::Null],
        mana_adj: 3,
        min_mana: 18,
        staff_min: 8,
        staff_max: 12,
    },
    // 43: Reflect
    SpellData {
        name: "Reflect",
        sfx: SfxID::CastSkill,
        book_cost_10: 300,
        staff_cost_10: 20,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 3,
        staff_level: 3,
        min_int: 25,
        missiles: [MissileID::Reflect, MissileID::Null],
        mana_adj: 3,
        min_mana: 15,
        staff_min: 8,
        staff_max: 12,
    },
    // 44: Berserk
    SpellData {
        name: "Berserk",
        sfx: SfxID::CastSkill,
        book_cost_10: 300,
        staff_cost_10: 20,
        mana_cost: 35,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: 3,
        staff_level: 3,
        min_int: 35,
        missiles: [MissileID::Berserk, MissileID::Null],
        mana_adj: 3,
        min_mana: 15,
        staff_min: 8,
        staff_max: 12,
    },
    // 45: RingOfFire
    SpellData {
        name: "Ring of Fire",
        sfx: SfxID::CastFire,
        book_cost_10: 600,
        staff_cost_10: 40,
        mana_cost: 28,
        flags: SpellDataFlags::with_flags(MagicType::Fire, false, false),
        book_level: 5,
        staff_level: 5,
        min_int: 27,
        missiles: [MissileID::RingOfFire, MissileID::Null],
        mana_adj: 2,
        min_mana: 16,
        staff_min: 8,
        staff_max: 16,
    },
    // 46: Search
    SpellData {
        name: "Search",
        sfx: SfxID::CastSkill,
        book_cost_10: 300,
        staff_cost_10: 20,
        mana_cost: 15,
        flags: SpellDataFlags::with_flags(MagicType::Magic, false, false),
        book_level: 1,
        staff_level: 3,
        min_int: 25,
        missiles: [MissileID::Search, MissileID::Null],
        mana_adj: 1,
        min_mana: 1,
        staff_min: 8,
        staff_max: 12,
    },
    // 47: RuneOfFire
    SpellData {
        name: "Rune of Fire",
        sfx: SfxID::CastHealing,
        book_cost_10: 800,
        staff_cost_10: 30,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: -1,
        staff_level: -1,
        min_int: 48,
        missiles: [MissileID::RuneOfFire, MissileID::Null],
        mana_adj: 1,
        min_mana: 10,
        staff_min: 40,
        staff_max: 80,
    },
    // 48: RuneOfLight
    SpellData {
        name: "Rune of Light",
        sfx: SfxID::CastHealing,
        book_cost_10: 800,
        staff_cost_10: 30,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: -1,
        staff_level: -1,
        min_int: 48,
        missiles: [MissileID::RuneOfLight, MissileID::Null],
        mana_adj: 1,
        min_mana: 10,
        staff_min: 40,
        staff_max: 80,
    },
    // 49: RuneOfNova
    SpellData {
        name: "Rune of Nova",
        sfx: SfxID::CastHealing,
        book_cost_10: 800,
        staff_cost_10: 30,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: -1,
        staff_level: -1,
        min_int: 48,
        missiles: [MissileID::RuneOfNova, MissileID::Null],
        mana_adj: 1,
        min_mana: 10,
        staff_min: 40,
        staff_max: 80,
    },
    // 50: RuneOfImmolation
    SpellData {
        name: "Rune of Immolation",
        sfx: SfxID::CastHealing,
        book_cost_10: 800,
        staff_cost_10: 30,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: -1,
        staff_level: -1,
        min_int: 48,
        missiles: [MissileID::RuneOfImmolation, MissileID::Null],
        mana_adj: 1,
        min_mana: 10,
        staff_min: 40,
        staff_max: 80,
    },
    // 51: RuneOfStone
    SpellData {
        name: "Rune of Stone",
        sfx: SfxID::CastHealing,
        book_cost_10: 800,
        staff_cost_10: 30,
        mana_cost: 255,
        flags: SpellDataFlags::with_flags(MagicType::Magic, true, false),
        book_level: -1,
        staff_level: -1,
        min_int: 48,
        missiles: [MissileID::RuneOfStone, MissileID::Null],
        mana_adj: 1,
        min_mana: 10,
        staff_min: 40,
        staff_max: 80,
    },
];

/// Get spell data by ID
pub fn get_spell_data(spell_id: SpellID) -> &'static SpellData {
    let index = spell_id as usize;
    if index < SPELLS_DATA.len() {
        &SPELLS_DATA[index]
    } else {
        &SPELLS_DATA[0] // Return Null spell for invalid IDs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_count() {
        assert_eq!(SPELLS_DATA.len(), SpellID::COUNT);
    }

    #[test]
    fn test_firebolt_data() {
        let firebolt = get_spell_data(SpellID::Firebolt);
        assert_eq!(firebolt.name, "Firebolt");
        assert_eq!(firebolt.mana_cost, 6);
        assert_eq!(firebolt.min_int, 15);
        assert_eq!(firebolt.book_cost(), 1000);
        assert_eq!(firebolt.staff_cost(), 50);
        assert_eq!(firebolt.magic_type(), MagicType::Fire);
        assert!(firebolt.is_targeted());
        assert!(!firebolt.is_allowed_in_town());
    }

    #[test]
    fn test_healing_data() {
        let healing = get_spell_data(SpellID::Healing);
        assert_eq!(healing.name, "Healing");
        assert_eq!(healing.mana_cost, 5);
        assert_eq!(healing.min_int, 17);
        assert!(healing.is_allowed_in_town());
        assert!(!healing.is_targeted());
    }

    #[test]
    fn test_apocalypse_data() {
        let apocalypse = get_spell_data(SpellID::Apocalypse);
        assert_eq!(apocalypse.name, "Apocalypse");
        assert_eq!(apocalypse.mana_cost, 150);
        assert_eq!(apocalypse.min_int, 149);
        assert_eq!(apocalypse.book_level, 19);
    }

    #[test]
    fn test_spell_flags() {
        let flags = SpellDataFlags::with_flags(MagicType::Fire, true, false);
        assert_eq!(flags.magic_type(), MagicType::Fire);
        assert!(flags.is_targeted());
        assert!(!flags.is_allowed_in_town());
    }

    #[test]
    fn test_spell_id_validation() {
        assert!(SpellID::Firebolt.is_valid());
        assert!(SpellID::RuneOfStone.is_valid());
        assert!(!SpellID::Invalid.is_valid());
    }

    #[test]
    fn test_hellfire_spells() {
        assert!(!SpellID::Firebolt.is_hellfire());
        assert!(SpellID::Mana.is_hellfire());
        assert!(SpellID::RuneOfFire.is_hellfire());
    }
}
