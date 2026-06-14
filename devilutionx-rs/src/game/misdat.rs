//! Missile Data
//!
//! C++ Reference: Source/misdat.cpp, Source/misdat.h
//!
//! Data related to missiles (projectiles, spell effects, etc.)

/// Missile target type
///
/// C++ Reference: `mienemy_type` enum in misdat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MissileTargetType {
    #[default]
    Monsters = 0,
    Players = 1,
    Both = 2,
}

/// Damage type for missiles and attacks
///
/// C++ Reference: `DamageType` enum in misdat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DamageType {
    #[default]
    Physical = 0,
    Fire = 1,
    Lightning = 2,
    Magic = 3,
    Acid = 4,
}

/// Missile graphic IDs
///
/// C++ Reference: `MissileGraphicID` enum in misdat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MissileGraphicId {
    #[default]
    Arrow = 0,
    Fireball = 1,
    Guardian = 2,
    Lightning = 3,
    FireWall = 4,
    MagmaBallExplosion = 5,
    TownPortal = 6,
    FlashBottom = 7,
    FlashTop = 8,
    ManaShield = 9,
    BloodHit = 10,
    BoneHit = 11,
    MetalHit = 12,
    FireArrow = 13,
    DoomSerpents = 14,
    Golem = 15,
    Spurt = 16,
    ApocalypseBoom = 17,
    StoneCurseShatter = 18,
    BigExplosion = 19,
    Inferno = 20,
    ThinLightning = 21,
    BloodStar = 22,
    BloodStarExplosion = 23,
    MagmaBall = 24,
    Krull = 25,
    ChargedBolt = 26,
    HolyBolt = 27,
    HolyBoltExplosion = 28,
    LightningArrow = 29,
    FireArrowExplosion = 30,
    Acid = 31,
    AcidSplat = 32,
    AcidPuddle = 33,
    Etherealize = 34,
    Elemental = 35,
    Resurrect = 36,
    BoneSpirit = 37,
    RedPortal = 38,
    DiabloApocalypseBoom = 39,
    BloodStarBlue = 40,
    BloodStarBlueExplosion = 41,
    BloodStarYellow = 42,
    BloodStarYellowExplosion = 43,
    BloodStarRed = 44,
    BloodStarRedExplosion = 45,
    HorkSpawn = 46,
    Reflect = 47,
    OrangeFlare = 48,
    BlueFlare = 49,
    RedFlare = 50,
    YellowFlare = 51,
    Rune = 52,
    YellowFlareExplosion = 53,
    BlueFlareExplosion = 54,
    RedFlareExplosion = 55,
    BlueFlare2 = 56,
    OrangeFlareExplosion = 57,
    BlueFlareExplosion2 = 58,
    None = 255,
}

/// Missile movement distribution type
///
/// C++ Reference: `MissileMovementDistribution` enum in misdat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MissileMovementDistribution {
    /// No movement - missile is stationary
    #[default]
    Disabled = 0,
    /// Missile stops when hitting an enemy (e.g., firebolt)
    Blockable = 1,
    /// Missile continues through enemies (e.g., flame wave)
    Unblockable = 2,
}

/// 16-point compass direction for missile sprites
///
/// C++ Reference: `Direction16` enum in misdat.h
///
/// Used for projectiles like arrows that have 16 directional sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Direction16 {
    #[default]
    South = 0,
    SouthSouthWest = 1,
    SouthWest = 2,
    WestSouthWest = 3,
    West = 4,
    WestNorthWest = 5,
    NorthWest = 6,
    NorthNorthWest = 7,
    North = 8,
    NorthNorthEast = 9,
    NorthEast = 10,
    EastNorthEast = 11,
    East = 12,
    EastSouthEast = 13,
    SouthEast = 14,
    SouthSouthEast = 15,
}

impl Direction16 {
    /// Convert from 8-direction to 16-direction
    pub fn from_direction8(dir: crate::engine::Direction) -> Self {
        match dir {
            crate::engine::Direction::South => Self::South,
            crate::engine::Direction::SouthWest => Self::SouthWest,
            crate::engine::Direction::West => Self::West,
            crate::engine::Direction::NorthWest => Self::NorthWest,
            crate::engine::Direction::North => Self::North,
            crate::engine::Direction::NorthEast => Self::NorthEast,
            crate::engine::Direction::East => Self::East,
            crate::engine::Direction::SouthEast => Self::SouthEast,
            crate::engine::Direction::NoDirection => Self::South, // Default to South
        }
    }
    
    /// Get the sprite index for this direction
    pub fn sprite_index(self) -> usize {
        self as usize
    }
}

use bitflags::bitflags;

bitflags! {
    /// Missile data flags
    ///
    /// C++ Reference: `MissileDataFlags` enum in misdat.h
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MissileDataFlags: u8 {
        /// Physical damage type (bits 0-2)
        const PHYSICAL = 0;
        const FIRE = 1;
        const LIGHTNING = 2;
        const MAGIC = 3;
        const ACID = 4;
        /// Missile is an arrow (affected by arrow-specific bonuses)
        const ARROW = 1 << 4;
        /// Missile is invisible
        const INVISIBLE = 1 << 5;
    }
}

impl MissileDataFlags {
    /// Get the damage type from flags
    pub fn damage_type(self) -> DamageType {
        match self.bits() & 0x07 {
            0 => DamageType::Physical,
            1 => DamageType::Fire,
            2 => DamageType::Lightning,
            3 => DamageType::Magic,
            4 => DamageType::Acid,
            _ => DamageType::Physical,
        }
    }
    
    /// Check if missile is an arrow
    pub fn is_arrow(self) -> bool {
        self.contains(Self::ARROW)
    }
    
    /// Check if missile is invisible
    pub fn is_invisible(self) -> bool {
        self.contains(Self::INVISIBLE)
    }
}

/// Missile data entry
///
/// C++ Reference: `MissileData` struct in misdat.h
#[derive(Debug, Clone)]
pub struct MissileData {
    /// Graphic ID for this missile
    pub graphic: MissileGraphicId,
    /// Sound effect when missile is created
    pub fire_sound: i8,
    /// Sound effect when missile hits
    pub impact_sound: i8,
    /// Movement distribution type
    pub movement: MissileMovementDistribution,
    /// Target type
    pub target_type: MissileTargetType,
    /// Missile flags
    pub flags: MissileDataFlags,
    /// Number of animation frames
    pub anim_len: u8,
    /// Animation delay
    pub anim_delay: u8,
    /// Missile width
    pub anim_width: u16,
}

impl Default for MissileData {
    fn default() -> Self {
        Self {
            graphic: MissileGraphicId::None,
            fire_sound: -1,
            impact_sound: -1,
            movement: MissileMovementDistribution::Disabled,
            target_type: MissileTargetType::Monsters,
            flags: MissileDataFlags::empty(),
            anim_len: 1,
            anim_delay: 1,
            anim_width: 96,
        }
    }
}

/// Missile graphic data
///
/// C++ Reference: `MissileFileData` struct
#[derive(Debug, Clone)]
pub struct MissileFileData {
    /// Animation filename prefix
    pub name: &'static str,
    /// Number of animation frames
    pub anim_len: u8,
    /// Frame width
    pub width: u16,
}

/// Missile graphics data table
pub static MISSILE_FILE_DATA: &[MissileFileData] = &[
    MissileFileData { name: "arrows", anim_len: 16, width: 96 },
    MissileFileData { name: "fireba", anim_len: 16, width: 96 },
    MissileFileData { name: "guard", anim_len: 15, width: 96 },
    MissileFileData { name: "lghning", anim_len: 8, width: 96 },
    MissileFileData { name: "firewal", anim_len: 13, width: 128 },
    MissileFileData { name: "magblos", anim_len: 10, width: 128 },
    MissileFileData { name: "portal", anim_len: 16, width: 96 },
    MissileFileData { name: "bluexfr", anim_len: 19, width: 160 },
    MissileFileData { name: "bluexbk", anim_len: 19, width: 160 },
    MissileFileData { name: "manashld", anim_len: 1, width: 96 },
    // ... more entries would follow
];

/// Get missile graphic data by ID
pub fn get_missile_file_data(graphic: MissileGraphicId) -> Option<&'static MissileFileData> {
    let index = graphic as usize;
    if index < MISSILE_FILE_DATA.len() {
        Some(&MISSILE_FILE_DATA[index])
    } else {
        None
    }
}
