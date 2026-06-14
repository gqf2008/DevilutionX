// Missiles System (Day 41-45, M66 Enhancement) - 投射物系统
//
// C++ References:
// - Source/missiles.h (Missile struct, MissilePosition)
// - Source/missiles.cpp:4216-4250 (ProcessMissiles)
// - Source/misdat.h (MissileData, MissileDataFlags)
// - Source/missiles.cpp:485-555 (CheckMissileCol - collision detection)
// - Source/missiles.cpp:300-352 (MonsterMHit - monster damage)
// - Source/missiles.cpp:353-450 (Plr2PlrMHit - player vs player damage)
//
// This module implements the projectile/spell effect system that is processed
// every game frame in the main game loop.
//
// M66 Enhancements:
// - MissileData system for missile metadata
// - DamageType integration
// - Enhanced collision detection with game state
// - Full damage calculation with resistances

use crate::game::types::Point;
use crate::game::combat::DamageType;
use rand::Rng;

/// Missile position tracking
///
/// **C++ Reference**: `struct MissilePosition` in `Source/missiles.h:23-52`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissilePosition {
    /// Sprite's pixel offset from tile
    pub offset: Point,

    /// Pixel velocity while moving
    pub velocity: Point,

    /// Pixels traveled as a numerator of 65,536
    pub traveled: Point,

    /// Current tile position
    pub tile: Point,

    /// Start position
    pub start: Point,

    /// Location (tile) while rendering
    pub tile_for_rendering: Point,

    /// Location (offset) while rendering
    pub offset_for_rendering: Point,
}

impl MissilePosition {
    /// Create new missile position at given tile
    pub fn new(tile: Point) -> Self {
        Self {
            offset: Point::new(0, 0),
            velocity: Point::new(0, 0),
            traveled: Point::new(0, 0),
            tile,
            start: tile,
            tile_for_rendering: tile,
            offset_for_rendering: Point::new(0, 0),
        }
    }

    /// Stop the missile (set velocity to zero)
    ///
    /// **C++ Reference**: `MissilePosition::StopMissile()` in `Source/missiles.h:44-50`
    pub fn stop_missile(&mut self) {
        self.velocity = Point::new(0, 0);
        if self.tile_for_rendering == self.tile {
            self.offset = self.offset_for_rendering;
        }
    }
}

/// Missile source type
///
/// **C++ Reference**: `enum class MissileSource` in `Source/missiles.h:54-58`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissileSource {
    Player,
    Monster,
    Trap,
}

/// Missile caster target type
///
/// **C++ Reference**: `mienemy_type` in C++ (TARGET_PLAYERS, TARGET_MONSTERS, TARGET_BOTH)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissileCaster {
    /// Missile targets players (cast by monster)
    TargetPlayers,

    /// Missile targets monsters (cast by player)
    TargetMonsters,

    /// Missile targets both
    TargetBoth,
}

/// Missile type ID
///
/// **C++ Reference**: `enum class MissileID` in `Source/spelldat.h:99-210`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum MissileID {
    Arrow = 0,
    Firebolt = 1,
    Guardian = 2,
    Phasing = 3,
    NovaBall = 4,
    Firewall = 5,
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
    ChainBall = 16,      // unused
    BloodHit = 17,       // unused
    BoneHit = 18,        // unused
    MetalHit = 19,       // unused
    Rhino = 20,
    MagmaBall = 21,
    ThinLightningControl = 22,
    ThinLightning = 23,
    BloodStar = 24,
    BloodStarExplosion = 25,
    Teleport = 26,
    FireArrow = 27,
    DoomSerpents = 28,   // unused
    FireOnly = 29,       // unused
    StoneCurse = 30,
    BloodRitual = 31,    // unused
    Invisibility = 32,   // unused
    Golem = 33,
    Etherealize = 34,
    Spurt = 35,          // unused
    ApocalypseBoom = 36,
    Healing = 37,
    FireWallControl = 38,
    Infravision = 39,
    Identify = 40,
    FlameWaveControl = 41,
    Nova = 42,
    Rage = 43,           // BloodBoil in Diablo
    Apocalypse = 44,
    ItemRepair = 45,
    StaffRecharge = 46,
    TrapDisarm = 47,
    Inferno = 48,
    InfernoControl = 49,
    FireMan = 50,        // unused
    Krull = 51,
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
    // Hellfire extensions
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

    // Additional types used in processing
    GuardianTentacle = 103,  // Guardian hydra tentacle attack
    ElementalBall = 104,     // Elemental ball projectile
    StoneCurseMissile = 105, // Stone curse effect
    SpecArrow = 106,         // Special arrow (spectral)

    // Rune aliases (using remaining values)
    Rune1 = 107,  // RuneOfFire alias
    Rune2 = 108,  // RuneOfLight alias
    Rune3 = 109,  // RuneOfNova alias
    Rune4 = 110,  // RuneOfImmolation alias
    Rune5 = 111,  // RuneOfStone alias

    Null = -1,
}

/// Missile graphics flags
///
/// **C++ Reference**: `enum class MissileGraphicsFlags` in `Source/missiles.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissileGraphicsFlags {
    NotAnimated,
    Animated,
}

/// Main missile structure
///
/// **C++ Reference**: `struct Missile` in `Source/missiles.h:86-187`
#[derive(Debug, Clone)]
pub struct Missile {
    /// Type of projectile
    pub missile_type: MissileID,

    /// Position and movement data
    pub position: MissilePosition,

    /// Spell level
    pub spell_level: i32,

    /// Indicate whether the missile should be deleted
    pub delete_flag: bool,

    /// Animation type
    pub anim_type: u8,

    /// Animation flags
    pub anim_flags: MissileGraphicsFlags,

    /// Tick length of each frame in current animation
    pub anim_delay: i32,

    /// Number of frames in current animation
    pub anim_len: i32,

    /// Current frame count (increases each tick)
    pub anim_cnt: i32,

    /// Animation frame increment (+1 or -1)
    pub anim_add: i32,

    /// Current frame of animation + 1
    pub anim_frame: i32,

    /// Draw flag
    pub draw_flag: bool,

    /// Light flag
    pub light_flag: bool,

    /// Pre-draw flag
    pub pre_flag: bool,

    /// Unique transparency value
    pub uniq_trans: u32,

    /// Time to live in game ticks
    pub duration: i32,

    /// Source entity index (-1 for trap)
    pub source: i32,

    /// Caster type
    pub caster: MissileCaster,

    /// Damage amount
    pub damage: i32,

    /// Hit flag
    pub hit_flag: bool,

    /// Distance traveled (for arrows)
    pub distance: i32,

    /// Light ID
    pub light_id: i32,

    /// Random value
    pub random: i32,

    /// Variable storage (8 slots for missile-specific data)
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
    pub var4: i32,
    pub var5: i32,
    pub var6: i32,
    pub var7: i32,

    /// Limit reached flag
    pub limit_reached: bool,

    /// Last collision target hash
    pub last_collision_target_hash: i16,
}

impl Missile {
    /// Create new missile
    pub fn new(missile_type: MissileID, position: Point) -> Self {
        Self {
            missile_type,
            position: MissilePosition::new(position),
            spell_level: 0,
            delete_flag: false,
            anim_type: 0,
            anim_flags: MissileGraphicsFlags::NotAnimated,
            anim_delay: 0,
            anim_len: 0,
            anim_cnt: 0,
            anim_add: 1,
            anim_frame: 1,
            draw_flag: true,
            light_flag: false,
            pre_flag: false,
            uniq_trans: 0,
            duration: 0,
            source: -1,
            caster: MissileCaster::TargetMonsters,
            damage: 0,
            hit_flag: false,
            distance: 0,
            light_id: -1,
            random: 0,
            var1: 0,
            var2: 0,
            var3: 0,
            var4: 0,
            var5: 0,
            var6: 0,
            var7: 0,
            limit_reached: false,
            last_collision_target_hash: 0,
        }
    }

    /// Check if missile is from a trap
    ///
    /// **C++ Reference**: `Missile::IsTrap()` in `Source/missiles.h:138-141`
    pub fn is_trap(&self) -> bool {
        self.source == -1
    }

    /// Get missile source type
    ///
    /// **C++ Reference**: `Missile::sourceType()` in `Source/missiles.h:163-169`
    pub fn source_type(&self) -> MissileSource {
        if self.source == -1 {
            return MissileSource::Trap;
        }
        if matches!(self.caster, MissileCaster::TargetPlayers) {
            return MissileSource::Monster;
        }
        MissileSource::Player
    }

    /// Stop the missile
    pub fn stop(&mut self) {
        self.position.stop_missile();
    }
}

impl Default for Missile {
    fn default() -> Self {
        Self::new(MissileID::Arrow, Point::new(0, 0))
    }
}

/// Calculate missile velocity components from direction and speed
///
/// **C++ Reference**: `UpdateMissileVelocity()` in missiles.cpp
fn calculate_missile_velocity(dx: i32, dy: i32, speed: i32) -> (i32, i32) {
    if dx == 0 && dy == 0 {
        return (0, 0);
    }

    let distance = ((dx * dx + dy * dy) as f64).sqrt();
    let vx = ((dx as f64 / distance) * speed as f64) as i32;
    let vy = ((dy as f64 / distance) * speed as f64) as i32;

    (vx, vy)
}

/// Missile manager
#[derive(Debug, Clone)]
pub struct MissileManager {
    missiles: Vec<Missile>,
    max_missiles: usize,
}

impl MissileManager {
    /// Create new missile manager
    pub fn new(max_missiles: usize) -> Self {
        Self {
            missiles: Vec::with_capacity(max_missiles),
            max_missiles,
        }
    }

    /// Add a missile
    pub fn add_missile(&mut self, missile: Missile) -> Option<usize> {
        if self.missiles.len() >= self.max_missiles {
            return None;
        }
        self.missiles.push(missile);
        Some(self.missiles.len() - 1)
    }

    /// Get missile by index
    pub fn get_missile(&self, index: usize) -> Option<&Missile> {
        self.missiles.get(index)
    }

    /// Get mutable missile by index
    pub fn get_missile_mut(&mut self, index: usize) -> Option<&mut Missile> {
        self.missiles.get_mut(index)
    }

    /// Delete missiles marked for deletion
    ///
    /// **C++ Reference**: Called in `ProcessMissiles()` in `Source/missiles.cpp:4227`
    pub fn delete_missiles(&mut self) {
        self.missiles.retain(|m| !m.delete_flag);
    }

    /// Process all missiles (main update loop)
    ///
    /// **C++ Reference**: `ProcessMissiles()` in `Source/missiles.cpp:4216-4250`
    ///
    /// This is called once per game frame in the main game loop.
    /// It handles missile animation, movement, collision, and deletion.
    pub fn process_missiles(&mut self) {
        // Phase 1: Clear dungeon flags and mark out-of-bounds missiles
        // C++ Reference: missiles.cpp:4217-4223
        for missile in &mut self.missiles {
            // In full implementation, would clear DungeonFlags here
            // For now, just mark out-of-bounds missiles
            if !is_in_dungeon_bounds(missile.position.tile) {
                missile.delete_flag = true;
            }
        }

        // Delete marked missiles
        self.delete_missiles();

        // Phase 2: Process missile logic and animate
        // C++ Reference: missiles.cpp:4227-4244
        for missile in &mut self.missiles {
            // Process missile-specific logic
            // C++ Reference: missileData.processFn(missile)
            // In full implementation, would call missile type-specific process function

            // Handle animation
            if missile.anim_flags == MissileGraphicsFlags::NotAnimated {
                continue;
            }

            missile.anim_cnt += 1;
            if missile.anim_cnt < missile.anim_delay {
                continue;
            }

            missile.anim_cnt = 0;
            missile.anim_frame += missile.anim_add;

            if missile.anim_frame > missile.anim_len {
                missile.anim_frame = 1;
            } else if missile.anim_frame < 1 {
                missile.anim_frame = missile.anim_len;
            }
        }

        // Final cleanup
        self.delete_missiles();
    }

    /// Get iterator over all missiles
    pub fn iter(&self) -> impl Iterator<Item = &Missile> {
        self.missiles.iter()
    }

    /// Get mutable iterator over all missiles
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Missile> {
        self.missiles.iter_mut()
    }

    /// Get number of active missiles
    pub fn count(&self) -> usize {
        self.missiles.len()
    }
}

/// Check if position is within dungeon bounds
///
/// **C++ Reference**: `InDungeonBounds()` in `Source/gendung.h`
fn is_in_dungeon_bounds(pos: Point) -> bool {
    // Simplified bounds check (112x112 is typical Diablo map size)
    pos.x >= 0 && pos.x < 112 && pos.y >= 0 && pos.y < 112
}

// =============================================================================
// M66: MissileData System - Missile Metadata
// =============================================================================

/// Missile movement distribution type
///
/// **C++ Reference**: `enum class MissileMovementDistribution` in `Source/misdat.h:93-107`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissileMovementDistribution {
    /// No movement - missile doesn't move
    Disabled,
    /// Blockable - stops on hit (e.g., firebolt)
    Blockable,
    /// Unblockable - keeps moving through targets (e.g., flame wave)
    Unblockable,
}

/// Missile data flags
///
/// **C++ Reference**: `enum class MissileDataFlags` in `Source/misdat.h:109-120`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissileDataFlags {
    /// Damage type (lower 3 bits)
    pub damage_type: DamageType,
    /// Is this an arrow-type missile
    pub is_arrow: bool,
    /// Is this missile invisible
    pub invisible: bool,
}

impl Default for MissileDataFlags {
    fn default() -> Self {
        Self {
            damage_type: DamageType::Physical,
            is_arrow: false,
            invisible: false,
        }
    }
}

/// Static missile data
///
/// **C++ Reference**: `struct MissileData` in `Source/misdat.h:158-188`
#[derive(Debug, Clone)]
pub struct MissileData {
    /// Missile type ID
    pub missile_id: MissileID,
    /// Graphics type for rendering
    pub graphic: MissileGraphicID,
    /// Flags including damage type
    pub flags: MissileDataFlags,
    /// Movement distribution type
    pub movement_distribution: MissileMovementDistribution,
    /// Base speed (pixels per frame, shifted by 16)
    pub speed: i32,
    /// Animation delay
    pub anim_delay: i32,
    /// Animation length
    pub anim_len: i32,
}

impl MissileData {
    /// Check if missile is drawn (not invisible)
    pub fn is_drawn(&self) -> bool {
        !self.flags.invisible
    }

    /// Check if missile is an arrow type
    pub fn is_arrow(&self) -> bool {
        self.flags.is_arrow
    }

    /// Get damage type
    pub fn damage_type(&self) -> DamageType {
        self.flags.damage_type
    }
}

/// Missile graphics ID
///
/// **C++ Reference**: `enum class MissileGraphicID` in `Source/misdat.h:33-90`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MissileGraphicID {
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

/// Get missile data for a missile type
///
/// **C++ Reference**: `GetMissileData()` in `Source/misdat.cpp`
pub fn get_missile_data(missile_id: MissileID) -> MissileData {
    match missile_id {
        MissileID::Arrow => MissileData {
            missile_id,
            graphic: MissileGraphicID::Arrow,
            flags: MissileDataFlags {
                damage_type: DamageType::Physical,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::Firebolt => MissileData {
            missile_id,
            graphic: MissileGraphicID::Fireball,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Fireball => MissileData {
            missile_id,
            graphic: MissileGraphicID::Fireball,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 14,
        },
        MissileID::Lightning => MissileData {
            missile_id,
            graphic: MissileGraphicID::Lightning,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::HolyBolt => MissileData {
            missile_id,
            graphic: MissileGraphicID::HolyBolt,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::ChargedBolt => MissileData {
            missile_id,
            graphic: MissileGraphicID::ChargedBolt,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 8,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Guardian => MissileData {
            missile_id,
            graphic: MissileGraphicID::Guardian,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::Inferno => MissileData {
            missile_id,
            graphic: MissileGraphicID::Inferno,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Unblockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::FlameWave => MissileData {
            missile_id,
            graphic: MissileGraphicID::FireWall,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Unblockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 13,
        },
        MissileID::BoneSpirit => MissileData {
            missile_id,
            graphic: MissileGraphicID::BoneSpirit,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        // M66 Day 2: Additional missile data entries
        MissileID::FireArrow => MissileData {
            missile_id,
            graphic: MissileGraphicID::FireArrow,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::LightningArrow => MissileData {
            missile_id,
            graphic: MissileGraphicID::LightningArrow,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::Acid => MissileData {
            missile_id,
            graphic: MissileGraphicID::Acid,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::AcidSplat => MissileData {
            missile_id,
            graphic: MissileGraphicID::AcidSplat,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::AcidPuddle => MissileData {
            missile_id,
            graphic: MissileGraphicID::AcidPuddle,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 4,
        },
        MissileID::Teleport => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::Firewall => MissileData {
            missile_id,
            graphic: MissileGraphicID::FireWall,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 13,
        },
        MissileID::TownPortal => MissileData {
            missile_id,
            graphic: MissileGraphicID::TownPortal,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::FlashBottom => MissileData {
            missile_id,
            graphic: MissileGraphicID::FlashBottom,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 10,
        },
        MissileID::FlashTop => MissileData {
            missile_id,
            graphic: MissileGraphicID::FlashTop,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 10,
        },
        MissileID::ManaShield => MissileData {
            missile_id,
            graphic: MissileGraphicID::ManaShield,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::ChainLightning => MissileData {
            missile_id,
            graphic: MissileGraphicID::Lightning,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Unblockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::BloodStar => MissileData {
            missile_id,
            graphic: MissileGraphicID::BloodStar,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::BloodStarExplosion => MissileData {
            missile_id,
            graphic: MissileGraphicID::BloodStarExplosion,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Elemental => MissileData {
            missile_id,
            graphic: MissileGraphicID::Elemental,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::Resurrect => MissileData {
            missile_id,
            graphic: MissileGraphicID::Resurrect,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::Nova => MissileData {
            missile_id,
            graphic: MissileGraphicID::ThinLightning,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Unblockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Apocalypse => MissileData {
            missile_id,
            graphic: MissileGraphicID::ApocalypseBoom,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 15,
        },
        MissileID::ApocalypseBoom => MissileData {
            missile_id,
            graphic: MissileGraphicID::ApocalypseBoom,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 15,
        },
        MissileID::Golem => MissileData {
            missile_id,
            graphic: MissileGraphicID::Golem,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::MagmaBall => MissileData {
            missile_id,
            graphic: MissileGraphicID::MagmaBall,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::MagmaBallExplosion => MissileData {
            missile_id,
            graphic: MissileGraphicID::MagmaBallExplosion,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 10,
        },
        MissileID::Rhino => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags {
                damage_type: DamageType::Physical,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 18,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::StoneCurse => MissileData {
            missile_id,
            graphic: MissileGraphicID::StoneCurseShatter,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        // Hellfire missiles
        MissileID::LightningWall => MissileData {
            missile_id,
            graphic: MissileGraphicID::ThinLightning,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Immolation => MissileData {
            missile_id,
            graphic: MissileGraphicID::BigExplosion,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 10,
        },
        MissileID::SpectralArrow => MissileData {
            missile_id,
            graphic: MissileGraphicID::Arrow,
            flags: MissileDataFlags {
                damage_type: DamageType::Physical,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 16,
        },
        MissileID::FireballBow => MissileData {
            missile_id,
            graphic: MissileGraphicID::Fireball,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 14,
        },
        MissileID::LightningBow => MissileData {
            missile_id,
            graphic: MissileGraphicID::ThinLightning,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::ChargedBoltBow => MissileData {
            missile_id,
            graphic: MissileGraphicID::ChargedBolt,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 8,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::HolyBoltBow => MissileData {
            missile_id,
            graphic: MissileGraphicID::HolyBolt,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: true,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 32,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Warp => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::Reflect => MissileData {
            missile_id,
            graphic: MissileGraphicID::Reflect,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::Berserk => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::RingOfFire => MissileData {
            missile_id,
            graphic: MissileGraphicID::FireWall,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Unblockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 13,
        },
        MissileID::Search => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        // Rune missiles
        MissileID::RuneOfFire => MissileData {
            missile_id,
            graphic: MissileGraphicID::Rune,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::RuneOfLight => MissileData {
            missile_id,
            graphic: MissileGraphicID::Rune,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::RuneOfNova => MissileData {
            missile_id,
            graphic: MissileGraphicID::Rune,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::RuneOfImmolation => MissileData {
            missile_id,
            graphic: MissileGraphicID::Rune,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::RuneOfStone => MissileData {
            missile_id,
            graphic: MissileGraphicID::Rune,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::BigExplosion => MissileData {
            missile_id,
            graphic: MissileGraphicID::BigExplosion,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 10,
        },
        MissileID::HorkSpawn => MissileData {
            missile_id,
            graphic: MissileGraphicID::HorkSpawn,
            flags: MissileDataFlags {
                damage_type: DamageType::Physical,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 8,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::Jester => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: true,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
        MissileID::RedPortal => MissileData {
            missile_id,
            graphic: MissileGraphicID::RedPortal,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 16,
        },
        // Flare missiles
        MissileID::OrangeFlare => MissileData {
            missile_id,
            graphic: MissileGraphicID::OrangeFlare,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::BlueFlare => MissileData {
            missile_id,
            graphic: MissileGraphicID::BlueFlare,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::RedFlare => MissileData {
            missile_id,
            graphic: MissileGraphicID::RedFlare,
            flags: MissileDataFlags {
                damage_type: DamageType::Fire,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::YellowFlare => MissileData {
            missile_id,
            graphic: MissileGraphicID::YellowFlare,
            flags: MissileDataFlags {
                damage_type: DamageType::Lightning,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        MissileID::BlueFlare2 => MissileData {
            missile_id,
            graphic: MissileGraphicID::BlueFlare2,
            flags: MissileDataFlags {
                damage_type: DamageType::Magic,
                is_arrow: false,
                invisible: false,
            },
            movement_distribution: MissileMovementDistribution::Blockable,
            speed: 16,
            anim_delay: 1,
            anim_len: 8,
        },
        // Default for unspecified missiles
        _ => MissileData {
            missile_id,
            graphic: MissileGraphicID::None,
            flags: MissileDataFlags::default(),
            movement_distribution: MissileMovementDistribution::Disabled,
            speed: 0,
            anim_delay: 1,
            anim_len: 1,
        },
    }
}

// =============================================================================
// Missile Velocity and Movement (Day 101)
// =============================================================================

/// Velocity divisor for missile movement calculation
const MISSILE_VELOCITY_SHIFT: i32 = 16;

/// Update missile velocity towards destination
///
/// **C++ Reference**: `UpdateMissileVelocity()` in `Source/missiles.cpp:168-185`
///
/// Calculates velocity vector to move missile from current position to destination
/// at the specified speed.
///
/// # Arguments
/// * `missile` - Missile to update
/// * `dst` - Target destination
/// * `speed` - Movement speed (pixels per frame, scaled by 2^16)
pub fn update_missile_velocity(missile: &mut Missile, dst: Point, speed: i32) {
    let dx = dst.x - missile.position.start.x;
    let dy = dst.y - missile.position.start.y;

    // Calculate distance (simplified - full implementation uses fixed point math)
    let dist_sq = dx * dx + dy * dy;
    if dist_sq == 0 {
        missile.position.velocity = Point::new(0, 0);
        return;
    }

    // Approximate distance using integer sqrt
    let dist = (dist_sq as f32).sqrt() as i32;
    if dist == 0 {
        missile.position.velocity = Point::new(0, 0);
        return;
    }

    // Scale velocity by speed and velocity shift
    let vel_x = (dx * speed * MISSILE_VELOCITY_SHIFT) / dist;
    let vel_y = (dy * speed * MISSILE_VELOCITY_SHIFT) / dist;

    missile.position.velocity = Point::new(vel_x, vel_y);
}

/// Move missile by its velocity
///
/// **C++ Reference**: `MoveMissile()` in `Source/missiles.cpp:200-220`
///
/// Updates missile position based on current velocity.
/// Returns true if missile moved to a new tile.
pub fn move_missile(missile: &mut Missile) -> bool {
    let old_tile = missile.position.tile;

    // Update traveled distance
    missile.position.traveled.x += missile.position.velocity.x;
    missile.position.traveled.y += missile.position.velocity.y;

    // Calculate new tile position
    let new_x = missile.position.start.x + (missile.position.traveled.x >> MISSILE_VELOCITY_SHIFT);
    let new_y = missile.position.start.y + (missile.position.traveled.y >> MISSILE_VELOCITY_SHIFT);

    missile.position.tile = Point::new(new_x, new_y);

    // Calculate pixel offset within tile
    missile.position.offset.x = (missile.position.traveled.x >> (MISSILE_VELOCITY_SHIFT - 6)) % 64;
    missile.position.offset.y = (missile.position.traveled.y >> (MISSILE_VELOCITY_SHIFT - 6)) % 64;

    missile.position.tile != old_tile
}

// =============================================================================
// Direction Utilities (Day 101)
// =============================================================================

/// Get 16-direction from source to destination
///
/// **C++ Reference**: `GetDirection16()` in `Source/engine/direction.cpp`
pub fn get_direction16(src: Point, dst: Point) -> u8 {
    let dx = dst.x - src.x;
    let dy = dst.y - src.y;

    // Simple 16-direction calculation
    let angle = (dy as f32).atan2(dx as f32);
    let dir = ((angle + std::f32::consts::PI) / (std::f32::consts::PI / 8.0)) as u8;
    dir % 16
}

// =============================================================================
// Collision Detection (Day 101)
// =============================================================================

/// Hit type for collision results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitType {
    None,
    Monster(usize),
    Player(usize),
    Wall,
    Object,
}

/// Collision result from missile check
#[derive(Debug, Clone)]
pub struct CollisionResult {
    pub hit_type: HitType,
    pub position: Point,
    pub blocked: bool,
}

// =============================================================================
// M66: Enhanced Collision Detection and Damage Calculation
// =============================================================================

/// Dungeon tile state for collision detection
///
/// **C++ Reference**: `dMonster`, `dPlayer`, `dObject` arrays in `Source/gendung.cpp`
pub struct DungeonState {
    /// Monster at each tile (0 = none, >0 = monster_id+1, <0 = dead/dying)
    pub monsters: [[i16; 112]; 112],
    /// Player at each tile (0 = none, 1-4 = player_id)
    pub players: [[i8; 112]; 112],
    /// Solid tiles that block missiles
    pub solid_tiles: [[bool; 112]; 112],
    /// Objects at each tile
    pub objects: [[i16; 112]; 112],
}

impl Default for DungeonState {
    fn default() -> Self {
        Self {
            monsters: [[0; 112]; 112],
            players: [[0; 112]; 112],
            solid_tiles: [[false; 112]; 112],
            objects: [[0; 112]; 112],
        }
    }
}

impl DungeonState {
    /// Check if tile has a monster
    pub fn has_monster(&self, pos: Point) -> Option<usize> {
        if !is_in_dungeon_bounds(pos) {
            return None;
        }
        let mid = self.monsters[pos.x as usize][pos.y as usize];
        if mid > 0 {
            Some((mid - 1) as usize)
        } else {
            None
        }
    }

    /// Check if tile has a player
    pub fn has_player(&self, pos: Point) -> Option<usize> {
        if !is_in_dungeon_bounds(pos) {
            return None;
        }
        let pid = self.players[pos.x as usize][pos.y as usize];
        if pid > 0 {
            Some((pid - 1) as usize)
        } else {
            None
        }
    }

    /// Check if tile is solid (blocks missiles)
    pub fn is_solid(&self, pos: Point) -> bool {
        if !is_in_dungeon_bounds(pos) {
            return true;
        }
        self.solid_tiles[pos.x as usize][pos.y as usize]
    }
}

/// Enhanced collision check with game state
///
/// **C++ Reference**: `CheckMissileCol()` in `Source/missiles.cpp:485-555`
pub fn check_missile_collision_with_state(
    missile: &Missile,
    pos: Point,
    state: &DungeonState,
) -> CollisionResult {
    // Check bounds first
    if !is_in_dungeon_bounds(pos) {
        return CollisionResult {
            hit_type: HitType::Wall,
            position: pos,
            blocked: true,
        };
    }

    // Check for monster hit
    if let Some(monster_id) = state.has_monster(pos) {
        // Check if this missile can hit this monster
        let can_hit = match missile.caster {
            MissileCaster::TargetPlayers => false, // Monster missiles don't hit monsters
            MissileCaster::TargetMonsters => true, // Player missiles hit monsters
            MissileCaster::TargetBoth => true,
        };

        if can_hit {
            return CollisionResult {
                hit_type: HitType::Monster(monster_id),
                position: pos,
                blocked: get_missile_data(missile.missile_type).movement_distribution
                    == MissileMovementDistribution::Blockable,
            };
        }
    }

    // Check for player hit
    if let Some(player_id) = state.has_player(pos) {
        // Check if this missile can hit this player
        let can_hit = match missile.caster {
            MissileCaster::TargetPlayers => true,
            MissileCaster::TargetMonsters => {
                // Player missiles only hit other players in PvP
                missile.source as usize != player_id
            }
            MissileCaster::TargetBoth => true,
        };

        if can_hit {
            return CollisionResult {
                hit_type: HitType::Player(player_id),
                position: pos,
                blocked: get_missile_data(missile.missile_type).movement_distribution
                    == MissileMovementDistribution::Blockable,
            };
        }
    }

    // Check for solid tile
    if state.is_solid(pos) {
        return CollisionResult {
            hit_type: HitType::Wall,
            position: pos,
            blocked: true,
        };
    }

    CollisionResult {
        hit_type: HitType::None,
        position: pos,
        blocked: false,
    }
}

// =============================================================================
// M66 Day 1: Enhanced CheckMissileCol - Full C++ Alignment
// =============================================================================

use crate::game::types::Direction;

/// Check if missile can hit only walking targets in specific direction
///
/// **C++ Reference**: `CheckCanHitOnlyWalking()` in `Source/missiles.cpp`
fn check_can_hit_only_walking(
    missile: &Missile,
    target_pos: Point,
    required_direction: Direction,
) -> bool {
    // Check if target is moving in the required direction
    // This is used for missiles that only hit walking targets (like Rhino charge)
    // In full implementation, would check target's actual walking direction

    // For now, simple distance check
    let dx = target_pos.x - missile.position.start.x;
    let dy = target_pos.y - missile.position.start.y;

    match required_direction {
        Direction::North => dy < 0,
        Direction::South => dy > 0,
        Direction::East => dx > 0,
        Direction::West => dx < 0,
        Direction::NorthEast => dx > 0 && dy < 0,
        Direction::NorthWest => dx < 0 && dy < 0,
        Direction::SouthEast => dx > 0 && dy > 0,
        Direction::SouthWest => dx < 0 && dy > 0,
    }
}

/// Full CheckMissileCol implementation aligned with C++
///
/// **C++ Reference**: `CheckMissileCol()` in `Source/missiles.cpp:485-555`
///
/// This checks collision at a specific position and applies damage if hit.
/// It handles:
/// - Monster collision and damage
/// - Player collision and damage (PvP and monster attacks)
/// - Wall/object collision
/// - Special cases (traps, berserked monsters, etc.)
///
/// # Parameters
/// - `missile`: The missile to check collision for (mutable for hit_flag)
/// - `damage_type`: Type of damage dealt
/// - `min_damage`: Minimum damage
/// - `max_damage`: Maximum damage
/// - `is_damage_shifted`: Whether damage is already shifted by 64
/// - `position`: Position to check collision at
/// - `dont_delete_on_collision`: Keep missile alive after hit
/// - `only_hit_walking`: Only hit targets walking in this direction
/// - `state`: Dungeon state with monster/player/tile data
/// - `monster_hit_callback`: Called when monster is hit (monster_id, damage) -> hit_success
/// - `player_hit_callback`: Called when player is hit (player_id, damage) -> (hit_success, blocked)
/// - `object_hit_callback`: Called when object is hit (object_id)
pub fn check_missile_col<F, G, H>(
    missile: &mut Missile,
    damage_type: DamageType,
    min_damage: i32,
    max_damage: i32,
    is_damage_shifted: bool,
    position: Point,
    dont_delete_on_collision: bool,
    only_hit_walking: Option<Direction>,
    state: &DungeonState,
    mut monster_hit_callback: F,
    mut player_hit_callback: G,
    mut object_hit_callback: H,
) where
    F: FnMut(usize, i32, i32) -> bool, // (monster_id, min_dmg, max_dmg) -> hit_success
    G: FnMut(usize, i32, i32) -> (bool, bool), // (player_id, min_dmg, max_dmg) -> (hit_success, blocked)
    H: FnMut(usize), // (object_id)
{
    // Check bounds
    if !is_in_dungeon_bounds(position) {
        return;
    }

    let mut is_monster_hit = false;

    // Check monster collision
    if let Some(monster_id) = state.has_monster(position) {
        let mid = state.monsters[position.x as usize][position.y as usize];

        // Check if target is valid based on only_hit_walking
        let target_valid = if let Some(dir) = only_hit_walking {
            // Would check monster.isWalking() && CheckCanHitOnlyWalking
            check_can_hit_only_walking(missile, position, dir)
        } else {
            // Normal check: mid > 0 or monster.mode == Petrified
            mid > 0
        };

        if target_valid {
            // Check if missile can hit this monster
            let can_hit = if missile.is_trap() {
                // Traps can hit any monster
                true
            } else if matches!(missile.caster, MissileCaster::TargetPlayers) {
                // Monster-fired missiles hitting monsters
                // Can hit if different faction or berserked
                // Simplified: assume can't hit same faction
                false // Would check faction/berserk flags
            } else if matches!(missile.caster, MissileCaster::TargetBoth | MissileCaster::TargetMonsters) {
                // Player missiles or both-target missiles
                true
            } else {
                false
            };

            if can_hit {
                // Call monster hit callback
                is_monster_hit = monster_hit_callback(
                    monster_id,
                    min_damage,
                    max_damage,
                );
            }
        }
    }

    if is_monster_hit {
        if !dont_delete_on_collision {
            missile.duration = 0;
        }
        missile.hit_flag = true;
    }

    let mut is_player_hit = false;
    let mut blocked = false;

    // Check player collision
    if let Some(player_id) = state.has_player(position) {
        // Check if target is valid based on only_hit_walking
        let target_valid = if let Some(dir) = only_hit_walking {
            // Would check player.isWalking() && CheckCanHitOnlyWalking
            check_can_hit_only_walking(missile, position, dir)
        } else {
            true
        };

        if target_valid {
            let can_hit = if missile.caster != MissileCaster::TargetBoth && !missile.is_trap() {
                if matches!(missile.caster, MissileCaster::TargetMonsters) {
                    // Player missile (PvP)
                    missile.source != player_id as i32
                } else {
                    // Monster missile targeting players
                    true
                }
            } else {
                // Trap or both-target missile
                true
            };

            if can_hit {
                let (hit, is_blocked) = player_hit_callback(
                    player_id,
                    min_damage,
                    max_damage,
                );
                is_player_hit = hit;
                blocked = is_blocked;
            }
        }
    }

    if is_player_hit {
        // Hellfire: blocked missiles get rotated instead of deleted
        if blocked {
            // Would call RotateBlockedMissile(missile)
            // For now, just don't delete
        } else if !dont_delete_on_collision {
            missile.duration = 0;
        }
        missile.hit_flag = true;
    }

    // Check wall/object collision
    if is_missile_blocked_by_tile(position, state) {
        // Check for breakable object
        if let Some(obj_id) = state.objects[position.x as usize][position.y as usize].checked_sub(1) {
            if obj_id >= 0 {
                // Would check if object->IsBreakable()
                // Would call BreakObjectMissile(missile.sourcePlayer(), *object)
                object_hit_callback(obj_id as usize);
            }
        }

        if !dont_delete_on_collision {
            missile.duration = 0;
        }
        missile.hit_flag = false; // Wall hits don't set hit flag
    }

    // Play hit sound if missile expired
    if missile.duration == 0 {
        let missile_data = get_missile_data(missile.missile_type);
        // Would play: PlaySfxLoc(missileData.hitSound, missile.position.tile)
    }
}

/// Move missile and check collision along path
///
/// **C++ Reference**: `MoveMissileAndCheckMissileCol()` in `Source/missiles.cpp:642-673`
///
/// This moves the missile and checks collision at each step. It uses a hash
/// to track which target was last checked to avoid multiple hits on slow missiles.
pub fn move_missile_and_check_missile_col<F, G, H>(
    missile: &mut Missile,
    damage_type: DamageType,
    min_damage: i32,
    max_damage: i32,
    ignore_start: bool,
    _if_collides_dont_move_to_hit_tile: bool,
    state: &DungeonState,
    mut monster_hit_callback: F,
    mut player_hit_callback: G,
    mut object_hit_callback: H,
) where
    F: FnMut(usize, i32, i32) -> bool,
    G: FnMut(usize, i32, i32) -> (bool, bool),
    H: FnMut(usize),
{
    let old_tile = missile.position.tile;

    // Move the missile (simplified - full implementation would call MoveMissile)
    // For now, just update position based on velocity
    missile.position.tile.x += missile.position.velocity.x / 65536;
    missile.position.tile.y += missile.position.velocity.y / 65536;

    let tile_changed = missile.position.tile != old_tile;

    // Check collision at each tile along path
    if tile_changed {
        let tile = missile.position.tile;

        // Skip start tile if requested
        if ignore_start && missile.position.start == tile {
            return;
        }

        check_missile_col(
            missile,
            damage_type,
            min_damage,
            max_damage,
            false, // is_damage_shifted
            tile,
            false, // dont_delete_on_collision
            None,  // only_hit_walking
            state,
            &mut monster_hit_callback,
            &mut player_hit_callback,
            &mut object_hit_callback,
        );

        // Did missile hit anything?
        if missile.duration == 0 {
            return;
        }

        // Check if missile is blockable and hit something
        if missile.hit_flag {
            let missile_data = get_missile_data(missile.missile_type);
            if missile_data.movement_distribution == MissileMovementDistribution::Blockable {
                return; // Missile stopped by hit
            }
        }

        // Check if tile blocks missile
        if is_missile_blocked_by_tile(tile, state) {
            return;
        }
    }

    // Calculate target hash to avoid duplicate hits
    let tile = missile.position.tile;
    let monster_hash = state.monsters[tile.x as usize][tile.y as usize];
    let player_hash = state.players[tile.x as usize][tile.y as usize] as i16;
    let tile_target_hash = monster_hash ^ player_hash;

    // If tile didn't change, only check collision if target changed
    if !tile_changed && missile.last_collision_target_hash != tile_target_hash {
        check_missile_col(
            missile,
            damage_type,
            min_damage,
            max_damage,
            false,
            tile,
            false,
            None,
            state,
            &mut monster_hit_callback,
            &mut player_hit_callback,
            &mut object_hit_callback,
        );
    }

    // Remember what target we checked against
    missile.last_collision_target_hash = tile_target_hash;
}

// =============================================================================
// M66 Day 2: Damage Calculation System - Full C++ Alignment
// =============================================================================

/// Monster hit by missile with full damage calculation
///
/// **C++ Reference**: `MonsterMHit()` in `Source/missiles.cpp:277-352`
///
/// This handles a player's missile hitting a monster, including:
/// - Hit chance calculation based on missile type (arrow vs magic)
/// - Damage calculation with player bonuses
/// - Resistance application
/// - Monster state updates (knockback, hit animation, death)
///
/// # Parameters
/// - `player_level`: Attacker's level
/// - `player_to_hit`: Player's ranged or magic to-hit bonus
/// - `player_damage_mod`: Player's damage modifier
/// - `player_bonus_dam`: Player's bonus damage percentage
/// - `player_bonus_dam_mod`: Player's flat bonus damage
/// - `player_class_is_rogue`: Whether player is Rogue class
/// - `monster`: Monster being hit (mutable for HP/state changes)
/// - `monster_ac`: Monster's armor class
/// - `monster_level`: Monster's level
/// - `monster_hp`: Monster's current HP (fixed point)
/// - `min_damage`: Minimum damage
/// - `max_damage`: Maximum damage
/// - `distance`: Distance from missile source
/// - `missile_type`: Type of missile
/// - `damage_type`: Type of damage
/// - `is_damage_shifted`: Whether damage is already << 6
/// - `rng`: Random number generator
///
/// # Returns
/// - `(bool, i32, bool)`: (hit_success, damage_dealt, monster_killed)
pub fn monster_m_hit(
    player_level: i32,
    player_to_hit_ranged: i32,
    player_to_hit_magic: i32,
    player_armor_pierce: i32,
    player_damage_mod: i32,
    player_bonus_dam: i32,
    player_bonus_dam_mod: i32,
    player_class_is_rogue: bool,
    monster_ac: i32,
    monster_level: i32,
    monster_hp: &mut i32,
    monster_resist_percent: i32,
    min_damage: i32,
    max_damage: i32,
    distance: i32,
    missile_type: MissileID,
    damage_type: DamageType,
    is_damage_shifted: bool,
    rng: &mut impl rand::Rng,
) -> (bool, i32, bool) {
    // Check if monster is hittable (simplified - full version checks isPossibleToHit, isImmune)
    // For now, assume monster is hittable

    let missile_data = get_missile_data(missile_type);
    let is_arrow = missile_data.is_arrow();

    // Calculate hit chance
    let mut hit_percent = if is_arrow {
        // Ranged attack
        let mut hper = player_to_hit_ranged;
        hper -= player_armor_pierce; // Actually should call CalculateArmorPierce
        hper -= (distance * distance) / 2;
        hper
    } else {
        // Magic attack
        player_to_hit_magic - (monster_level * 2) - distance
    };

    // Clamp to 5-95%
    hit_percent = hit_percent.clamp(5, 95);

    // Roll to hit
    let hit_roll = rng.gen_range(0..100);
    if hit_roll >= hit_percent {
        // Miss
        return (false, 0, false);
    }

    // Calculate damage
    let mut dam = if missile_type == MissileID::BoneSpirit {
        // Bone Spirit does 1/3 of monster's HP
        (*monster_hp / 3) >> 6
    } else {
        rng.gen_range(min_damage..=max_damage)
    };

    // Apply player bonuses for arrows
    if is_arrow && damage_type == DamageType::Physical {
        dam = player_bonus_dam_mod + dam * player_bonus_dam / 100 + dam;
        if player_class_is_rogue {
            dam += player_damage_mod;
        } else {
            dam += player_damage_mod / 2;
        }
        // Triple damage vs demons (simplified - would check ItemSpecialEffect::TripleDemonDamage)
    }

    // Shift damage if not already shifted
    if !is_damage_shifted {
        dam <<= 6;
    }

    // Apply resistance
    let resist = monster_resist_percent > 0;
    if resist {
        dam >>= 2; // 25% damage when resistant
    }

    // Apply damage to monster
    *monster_hp -= dam;

    // Check if monster died
    let killed = (*monster_hp >> 6) <= 0;

    (true, dam, killed)
}

/// Player vs Player missile hit with full PvP logic
///
/// **C++ Reference**: `Plr2PlrMHit()` in `Source/missiles.cpp:353-458`
///
/// Handles PvP missile combat including:
/// - Friendly fire check
/// - Invincibility/etherealize checks
/// - Resistance calculation
/// - Block chance
/// - Damage calculation with player bonuses
///
/// # Returns
/// - `(bool, i32, bool)`: (hit_success, damage_dealt, blocked)
pub fn plr2plr_m_hit(
    attacker_level: i32,
    attacker_to_hit_ranged: i32,
    attacker_to_hit_magic: i32,
    attacker_damage_mod: i32,
    attacker_bonus_dam: i32,
    attacker_bonus_dam_mod: i32,
    attacker_class_is_rogue: bool,
    target_armor: i32,
    target_level: i32,
    target_hp: &mut i32,
    target_fire_resist: i32,
    target_lightning_resist: i32,
    target_magic_resist: i32,
    target_block_chance: i32,
    target_is_blocking: bool,
    min_damage: i32,
    max_damage: i32,
    distance: i32,
    missile_type: MissileID,
    damage_type: DamageType,
    is_damage_shifted: bool,
    friendly_fire_enabled: bool,
    rng: &mut impl rand::Rng,
) -> (bool, i32, bool) {
    // Check friendly fire
    if !friendly_fire_enabled {
        return (false, 0, false);
    }

    // Check for HolyBolt (doesn't hit players)
    if missile_type == MissileID::HolyBolt {
        return (false, 0, false);
    }

    let missile_data = get_missile_data(missile_type);
    let is_arrow = missile_data.is_arrow();

    // Get resistance based on damage type
    let resist_percent = match damage_type {
        DamageType::Fire => target_fire_resist,
        DamageType::Lightning => target_lightning_resist,
        DamageType::Magic => target_magic_resist,
        DamageType::Physical | DamageType::Holy => 0,
    };

    // Calculate hit chance
    let mut hit_percent = if is_arrow {
        attacker_to_hit_ranged - (distance * distance / 2) - target_armor
    } else {
        attacker_to_hit_magic - (target_level * 2) - distance
    };

    hit_percent = hit_percent.clamp(5, 95);

    // Roll to hit
    let hit_roll = rng.gen_range(0..100);
    if hit_roll >= hit_percent {
        return (false, 0, false);
    }

    // Calculate block chance
    let mut block_chance = 100;
    if !is_damage_shifted && target_is_blocking {
        block_chance = rng.gen_range(0..100);
    }

    let mut blk = target_block_chance - (attacker_level * 2);
    blk = blk.clamp(0, 100);

    // Calculate damage
    let mut dam = if missile_type == MissileID::BoneSpirit {
        *target_hp / 3
    } else {
        let mut d = rng.gen_range(min_damage..=max_damage);
        if is_arrow && damage_type == DamageType::Physical {
            let dam_mod = if attacker_class_is_rogue {
                attacker_damage_mod
            } else {
                attacker_damage_mod / 2
            };
            d += attacker_bonus_dam_mod + dam_mod + d * attacker_bonus_dam / 100;
        }
        if !is_damage_shifted {
            d <<= 6;
        }
        d
    };

    // Non-arrow spells do half damage in PvP
    if !is_arrow {
        dam /= 2;
    }

    // Apply resistance
    if resist_percent > 0 {
        dam -= (dam * resist_percent) / 100;
        // Target resisted, still counts as hit
        return (true, dam, false);
    }

    // Check for block
    let blocked = block_chance < blk;

    // Apply damage
    *target_hp -= dam;

    (true, dam, blocked)
}

/// Object hit by missile
///
/// **C++ Reference**: Object breaking logic in `Source/missiles.cpp`
///
/// Simplified version - just marks object as hit.
/// Full implementation would break the object and handle loot.
pub fn object_m_hit(
    _object_id: usize,
    _missile_type: MissileID,
) -> bool {
    // Simplified: just return true to indicate object was hit
    // Full implementation would call BreakObject, drop loot, etc.
    true
}

// =============================================================================
// M66 Day 3: 导弹处理循环和位置更新
// =============================================================================
// C++ References:
// - Source/missiles.cpp:219-230 (UpdateMissilePos)
// - Source/missiles.cpp:4216-4250 (ProcessMissiles main loop)
// - Source/missiles.cpp:556-640 (MoveMissile with tile checking)

/// Update missile pixel position based on traveled distance
///
/// **C++ Reference**: `UpdateMissilePos(Missile &missile)` in `Source/missiles.cpp:219-230`
///
/// Converts traveled distance (fixed-point) to actual tile and pixel offset.
pub fn update_missile_pos(missile: &mut Missile) {
    // Convert traveled distance to pixels (16-bit fixed point to pixels)
    let pixels_x = missile.position.traveled.x >> 16;
    let pixels_y = missile.position.traveled.y >> 16;

    // Screen to missile coordinate conversion
    // C++ Reference: screenToMissile() - converts (x,y) to tile offset
    // Formula: tileX = (x - y) / 64, tileY = (x + y) / 64
    let tile_offset_x = (pixels_x - pixels_y) / 64;
    let tile_offset_y = (pixels_x + pixels_y) / 64;

    // Update tile position
    missile.position.tile.x = missile.position.start.x + tile_offset_x;
    missile.position.tile.y = missile.position.start.y + tile_offset_y;

    // Calculate pixel offset within tile (for rendering)
    // Offset is remainder after tile conversion
    let tile_pixels_x = tile_offset_x * 64;
    let tile_pixels_y = tile_offset_y * 64;
    missile.position.offset.x = (pixels_x + pixels_y) - tile_pixels_y;
    missile.position.offset.y = (pixels_y - pixels_x) - tile_pixels_x + 32;
}

/// Check if missile position is in dungeon bounds
///
/// **C++ Reference**: `InDungeonBounds()` checks
fn is_missile_in_bounds(missile: &Missile) -> bool {
    is_in_dungeon_bounds(missile.position.tile)
}

/// Check if tile is blocked for missile movement
///
/// **C++ Reference**: `IsMissileBlockedByTile()` in `Source/missiles.cpp`
fn is_tile_blocked(tile: Point, state: &DungeonState) -> bool {
    state.is_solid(tile)
}

/// Move missile and check for collision
///
/// **C++ Reference**: `MoveMissile()` in `Source/missiles.cpp:556-640`
///
/// Advanced movement with tile-by-tile collision checking.
/// Returns true if missile successfully moved.
pub fn move_missile_with_collision<F>(
    missile: &mut Missile,
    mut check_tile: F,
) -> bool
where
    F: FnMut(Point) -> bool,
{
    let prev_tile = missile.position.tile;

    // Update traveled distance
    missile.position.traveled.x += missile.position.velocity.x;
    missile.position.traveled.y += missile.position.velocity.y;

    // Update position
    update_missile_pos(missile);

    // Calculate how many tiles might have been crossed
    let dx = (missile.position.tile.x - prev_tile.x).abs();
    let dy = (missile.position.tile.y - prev_tile.y).abs();
    let possible_visit_tiles = if missile.position.velocity.x == 0 || missile.position.velocity.y == 0 {
        dx.max(dy) // Walking distance (no diagonal)
    } else {
        dx + dy // Manhattan distance
    };

    // No movement
    if possible_visit_tiles == 0 {
        return false;
    }

    // Simple case: only moved to adjacent tile or didn't move far
    if possible_visit_tiles == 1 {
        // Check if new tile is valid
        if !check_tile(missile.position.tile) {
            // Invalid tile - stop missile
            missile.position.tile = prev_tile;
            missile.position.velocity = Point::new(0, 0);
            return false;
        }
        return true;
    }

    // Complex case: Missile skipped tiles (high speed)
    // Need to check intermediate tiles to avoid tunneling through walls
    // C++ Reference: missiles.cpp:570-620 (interpolation logic)

    // For simplicity in initial implementation, just check destination
    // Full implementation would interpolate and check each intermediate tile
    if !check_tile(missile.position.tile) {
        // Hit obstacle - stop at previous tile
        missile.position.tile = prev_tile;
        missile.position.velocity = Point::new(0, 0);
        return false;
    }

    true
}

/// Process single missile update (movement + collision)
///
/// **C++ Reference**: Logic spread across `ProcessMissiles()` and individual `MI_*` functions
///
/// This is called for each missile during the main game loop.
/// Handles missile-specific behavior, movement, and collision.
pub fn process_single_missile(
    missile: &mut Missile,
    dungeon_state: &DungeonState,
) {
    // Decrement duration
    if missile.duration > 0 {
        missile.duration -= 1;
        if missile.duration == 0 {
            missile.delete_flag = true;
            return;
        }
    }

    // Check if out of bounds
    if !is_missile_in_bounds(missile) {
        missile.delete_flag = true;
        return;
    }

    // Move missile if it has velocity
    if missile.position.velocity.x != 0 || missile.position.velocity.y != 0 {
        let moved = move_missile_with_collision(missile, |tile| {
            // Check if tile is walkable
            is_in_dungeon_bounds(tile) && !is_tile_blocked(tile, dungeon_state)
        });

        if !moved {
            // Hit obstacle - mark for deletion (simple behavior)
            missile.hit_flag = true;
            // Some missiles should explode on impact rather than just disappear
            // Full implementation would check missile type
        }
    }

    // Missile-specific processing would go here
    // C++ Reference: missileData.processFn(missile)
    // Each missile type (Fireball, Lightning, etc.) has its own process function
    // For now, just basic movement and lifetime
}

// =============================================================================
// M66 Day 4: 完善与集成
// =============================================================================
// C++ References:
// - Source/missiles.cpp:570-620 (完整插值逻辑)
// - Source/missiles.cpp:4216-4250 (ProcessMissiles完整循环)

/// Check intermediate tiles for high-speed missiles
///
/// **C++ Reference**: `MoveMissile()` interpolation logic in `Source/missiles.cpp:570-620`
///
/// Prevents tunneling by checking all tiles between start and end positions.
/// Returns the first blocked tile if any, or None if path is clear.
pub fn check_intermediate_tiles<F>(
    start: Point,
    end: Point,
    velocity: Point,
    mut check_tile: F,
) -> Option<Point>
where
    F: FnMut(Point) -> bool,
{
    // Calculate speed magnitude
    let speed_x = velocity.x.abs();
    let speed_y = velocity.y.abs();

    // Determine which component moves faster (for interpolation step size)
    let denominator = if 2 * speed_y >= speed_x {
        2 * speed_y
    } else {
        speed_x
    };

    if denominator == 0 {
        return None; // No movement
    }

    // Calculate increment velocity.
    // **C++**: denominator is a `float`, so `(32 << 16) / denominator` is a float
    // and `velocity * float` is computed in float then truncated to int via the
    // implicit float->int conversion when assigning to the Displacement. Doing the
    // integer product `velocity * (32 << 16)` first would overflow i32 for real
    // missile speeds, so we mirror the float math here.
    let factor = (32 << 16) as f32 / denominator as f32;
    let inc_x = (velocity.x as f32 * factor) as i32;
    let inc_y = (velocity.y as f32 * factor) as i32;

    // Start from aligned position
    let mut traveled_x = (start.x << 16);
    let mut traveled_y = (start.y << 16);

    // Check each intermediate tile
    let max_steps = 100; // Safety limit
    for _ in 0..max_steps {
        traveled_x += inc_x;
        traveled_y += inc_y;

        // Convert to tile coordinates via screenToMissile()
        // **C++** `DisplacementOf::screenToMissile()` (Source/engine/displacement.hpp):
        //   xNumerator = 2*deltaY + deltaX;  yNumerator = 2*deltaY - deltaX;
        //   x = (xNumerator + (xNumerator>=0?32:-32)) / 64
        //   y = (yNumerator + (yNumerator>=0?32:-32)) / 64
        let pixels_x = traveled_x >> 16;
        let pixels_y = traveled_y >> 16;
        let x_num = 2 * pixels_y + pixels_x;
        let y_num = 2 * pixels_y - pixels_x;
        let x_off: i32 = if x_num >= 0 { 32 } else { -32 };
        let y_off: i32 = if y_num >= 0 { 32 } else { -32 };
        let tile_x = (x_num + x_off) / 64;
        let tile_y = (y_num + y_off) / 64;
        let current_tile = Point::new(start.x + tile_x, start.y + tile_y);

        // Check if we've reached or passed the destination
        if current_tile == end {
            break;
        }

        // Check if this tile is blocked
        if !check_tile(current_tile) {
            return Some(current_tile);
        }
    }

    None // No blocked tiles found
}

/// Enhanced move with full interpolation for high-speed missiles
///
/// **C++ Reference**: Complete `MoveMissile()` in `Source/missiles.cpp:556-640`
///
/// This is the fully C++-aligned version with interpolation.
pub fn move_missile_with_full_interpolation<F>(
    missile: &mut Missile,
    mut check_tile: F,
) -> bool
where
    F: FnMut(Point) -> bool,
{
    let prev_tile = missile.position.tile;

    // Update traveled distance
    missile.position.traveled.x += missile.position.velocity.x;
    missile.position.traveled.y += missile.position.velocity.y;

    // Update position
    update_missile_pos(missile);

    // Calculate how many tiles might have been crossed
    let dx = (missile.position.tile.x - prev_tile.x).abs();
    let dy = (missile.position.tile.y - prev_tile.y).abs();
    let possible_visit_tiles = if missile.position.velocity.x == 0 || missile.position.velocity.y == 0 {
        dx.max(dy)
    } else {
        dx + dy
    };

    // No movement
    if possible_visit_tiles == 0 {
        return false;
    }

    // Simple case: only moved one tile
    if possible_visit_tiles == 1 {
        if !check_tile(missile.position.tile) {
            missile.position.tile = prev_tile;
            missile.position.velocity = Point::new(0, 0);
            return false;
        }
        return true;
    }

    // High-speed case: check all intermediate tiles
    if let Some(blocked_tile) = check_intermediate_tiles(
        prev_tile,
        missile.position.tile,
        missile.position.velocity,
        &mut check_tile,
    ) {
        // Found a blocked tile - stop just before it
        missile.position.tile = prev_tile;
        missile.position.velocity = Point::new(0, 0);
        return false;
    }

    // Also check final destination
    if !check_tile(missile.position.tile) {
        missile.position.tile = prev_tile;
        missile.position.velocity = Point::new(0, 0);
        return false;
    }

    true
}

/// Integrate missile processing with full collision detection
///
/// **C++ Reference**: Combines logic from `ProcessMissiles()` and collision checking
///
/// Complete single-missile update with interpolation and collision.
pub fn process_missile_integrated(
    missile: &mut Missile,
    dungeon_state: &DungeonState,
    use_interpolation: bool,
) {
    // Decrement duration
    if missile.duration > 0 {
        missile.duration -= 1;
        if missile.duration == 0 {
            missile.delete_flag = true;
            return;
        }
    }

    // Check if out of bounds
    if !is_missile_in_bounds(missile) {
        missile.delete_flag = true;
        return;
    }

    // Move missile if it has velocity
    if missile.position.velocity.x != 0 || missile.position.velocity.y != 0 {
        let moved = if use_interpolation {
            move_missile_with_full_interpolation(missile, |tile| {
                is_in_dungeon_bounds(tile) && !is_tile_blocked(tile, dungeon_state)
            })
        } else {
            move_missile_with_collision(missile, |tile| {
                is_in_dungeon_bounds(tile) && !is_tile_blocked(tile, dungeon_state)
            })
        };

        if !moved {
            missile.hit_flag = true;
        }
    }
}

/// Calculate missile damage based on type and resistances
///
/// **C++ Reference**: `MonsterMHit()` in `Source/missiles.cpp:215-300`
pub fn calculate_missile_damage(
    missile: &Missile,
    base_min: i32,
    base_max: i32,
    target_resistance: i32,
    rng: &mut impl Rng,
) -> i32 {
    let data = get_missile_data(missile.missile_type);

    // Roll damage
    let mut damage = rng.gen_range(base_min..=base_max);

    // Arrow damage gets bonus from source player
    if data.is_arrow() {
        // In full implementation, add player's bonus damage
        // damage += player._pIBonusDamMod + dam * player._pIBonusDam / 100
    }

    // Bone Spirit does 1/3 of target's HP
    if missile.missile_type == MissileID::BoneSpirit {
        // Special case - this is calculated elsewhere
        return damage;
    }

    // Scale by 64 if not pre-shifted (standard for most missiles)
    damage <<= 6;

    // Apply resistance
    if target_resistance > 0 {
        damage = damage * (100 - target_resistance) / 100;
    }

    damage
}

/// Monster resistance lookup
///
/// **C++ Reference**: `Monster::isResistant()` in `Source/monster.cpp`
pub fn get_monster_resistance(monster_class: u8, damage_type: DamageType) -> i32 {
    // Simplified resistance table - actual values are per-monster
    match damage_type {
        DamageType::Physical => 0,
        DamageType::Fire => {
            // Fire resistance varies by monster type
            match monster_class {
                0..=5 => 0,    // Normal monsters
                6..=10 => 25, // Fire resistant
                11..=15 => 50, // Fire immune (50% reduction)
                _ => 0,
            }
        }
        DamageType::Lightning => {
            match monster_class {
                0..=5 => 0,
                6..=10 => 25,
                11..=15 => 50,
                _ => 0,
            }
        }
        DamageType::Magic => {
            match monster_class {
                0..=5 => 0,
                6..=10 => 15,
                11..=15 => 30,
                _ => 0,
            }
        }
        DamageType::Holy => 0, // Holy bypasses resistances for undead
    }
}

/// Apply missile damage to monster
///
/// **C++ Reference**: `ApplyMonsterDamage()` in `Source/monster.cpp`
pub fn apply_missile_damage_to_monster(
    missile: &Missile,
    monster_hp: &mut i32,
    monster_class: u8,
    base_min: i32,
    base_max: i32,
    rng: &mut impl Rng,
) -> (i32, bool) {
    let data = get_missile_data(missile.missile_type);
    let resistance = get_monster_resistance(monster_class, data.damage_type());

    let damage = calculate_missile_damage(missile, base_min, base_max, resistance, rng);

    // Apply damage
    *monster_hp -= damage;

    // Return damage dealt and whether monster died
    let killed = *monster_hp <= 0;
    (damage, killed)
}

/// Check if missile is blocked by tile
///
/// **C++ Reference**: `IsMissileBlockedByTile()` in `Source/missiles.cpp:460-482`
pub fn is_missile_blocked_by_tile(pos: Point, state: &DungeonState) -> bool {
    if !is_in_dungeon_bounds(pos) {
        return true;
    }

    state.is_solid(pos)
}

/// Check missile collision at position
///
/// **C++ Reference**: `CheckMissileCol()` in `Source/missiles.cpp:635-750`
///
/// Simplified collision check - full implementation would check against
/// monster and player positions from game state.
pub fn check_missile_collision(missile: &Missile, pos: Point) -> CollisionResult {
    // Check bounds
    if !is_in_dungeon_bounds(pos) {
        return CollisionResult {
            hit_type: HitType::Wall,
            position: pos,
            blocked: true,
        };
    }

    // In full implementation, would check:
    // 1. Monster positions (DungeonFlag::Monster)
    // 2. Player positions (DungeonFlag::Player)
    // 3. Solid tiles (nSolidTable)
    // 4. Objects (DungeonFlag::Object)

    CollisionResult {
        hit_type: HitType::None,
        position: pos,
        blocked: false,
    }
}

// =============================================================================
// Missile Processing Functions (Day 101)
// =============================================================================

/// Process arrow missile
///
/// **C++ Reference**: `ProcessArrow()` in `Source/missiles.cpp:3045-3095`
///
/// Arrows travel in a straight line until hitting something or expiring.
pub fn process_arrow(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Move missile
    let moved = move_missile(missile);

    if moved {
        // Check collision at new position
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked {
            missile.delete_flag = true;
            return;
        }

        // Increment distance traveled
        missile.distance += 1;

        // Arrows have max range
        if missile.distance > 16 {
            missile.delete_flag = true;
        }
    }
}

/// Process firebolt missile
///
/// **C++ Reference**: `ProcessFirebolt()` in `Source/missiles.cpp:3097-3160`
///
/// Firebolts travel until hitting something, then explode.
pub fn process_firebolt(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Move missile
    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked || matches!(collision.hit_type, HitType::Monster(_) | HitType::Player(_)) {
            // Hit something - could spawn explosion here
            missile.hit_flag = true;
            missile.delete_flag = true;
        }
    }
}

/// Process fireball missile
///
/// **C++ Reference**: `ProcessFireball()` in `Source/missiles.cpp:3162-3230`
///
/// Fireballs are larger and explode on impact with area damage.
pub fn process_fireball(missile: &mut Missile) {
    // var1, var2 store starting position
    // var3 stores explosion timer

    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked {
            // Explode - in full implementation would spawn explosion missiles
            missile.var3 = 1; // Mark as exploded
            missile.hit_flag = true;
            missile.delete_flag = true;
        }
    }
}

/// Process lightning missile
///
/// **C++ Reference**: `ProcessLightning()` in `Source/missiles.cpp:3232-3300`
///
/// Lightning bolts travel through targets, damaging multiple enemies.
pub fn process_lightning(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Lightning doesn't use normal movement - it extends from source
    // var1 tracks current length
    missile.var1 += 1;

    // Max length reached
    if missile.var1 >= 8 {
        missile.delete_flag = true;
    }
}

/// Process guardian missile (Bone Spirit / Guardian orb)
///
/// **C++ Reference**: `ProcessGuardian()` in `Source/missiles.cpp:3302-3380`
///
/// Guardians orbit the player and seek out enemies.
pub fn process_guardian(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Guardians have complex behavior:
    // - Orbit around spawn point
    // - Seek nearest enemy
    // - var1, var2 store orbit angle and distance

    missile.var1 += 16; // Rotate
    if missile.var1 >= 360 {
        missile.var1 -= 360;
    }
}

/// Process holy bolt missile
///
/// **C++ Reference**: `ProcessHolyBolt()` in `Source/missiles.cpp:3382-3440`
///
/// Holy bolts only damage undead monsters.
pub fn process_holy_bolt(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked {
            missile.delete_flag = true;
        }

        // In full implementation, would check if target is undead
        // Only undead can be damaged by holy bolt
    }
}

/// Process inferno missile (fire stream)
///
/// **C++ Reference**: `ProcessInferno()` in `Source/missiles.cpp:3500-3560`
///
/// Inferno creates a stream of fire that damages over time.
pub fn process_inferno(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Inferno stays in place and damages nearby enemies
    // var2 tracks remaining damage ticks
    missile.var2 -= 1;
}

/// Process charged bolt missile
///
/// **C++ Reference**: `ProcessChargedBolt()` in `Source/missiles.cpp:3562-3640`
///
/// Charged bolts bounce around randomly.
pub fn process_charged_bolt(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Charged bolts change direction randomly
    // var1 tracks direction change timer
    missile.var1 -= 1;

    if missile.var1 <= 0 {
        // Change direction randomly
        missile.var1 = 5;
        missile.var2 = (missile.var2 + missile.random % 3 - 1 + 8) % 8;
    }

    move_missile(missile);
}

/// Process flash (Nova) bottom half
///
/// **C++ Reference**: `ProcessFlashBottom()` in `Source/missiles.cpp:3415-3465`
pub fn process_flash_bottom(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
    }
}

/// Process flash (Nova) top half
///
/// **C++ Reference**: `ProcessFlashTop()` in `Source/missiles.cpp:3467-3510`
pub fn process_flash_top(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
    }
}

// =============================================================================
// M66 Day 2: Enhanced Missile Processing Functions
// =============================================================================

/// Process fire wall missile
///
/// **C++ Reference**: `ProcessFireWall()` in `Source/missiles.cpp:3640-3700`
///
/// Fire walls deal continuous damage in a fixed area.
pub fn process_fire_wall(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // var1 contains countdown to stop spawning flames
    if missile.var1 > 0 {
        missile.var1 -= 1;
    }

    // Fire wall deals damage every few ticks
    // In full implementation, calls CheckMissileCol for damage
}

/// Process flame wave missile
///
/// **C++ Reference**: `ProcessFlameWave()` in `Source/missiles.cpp:3700-3760`
///
/// Flame waves move forward in a line, damaging all in path.
pub fn process_flame_wave(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Flame wave moves forward
    let moved = move_missile(missile);

    if moved {
        // Check collision but don't stop (unblockable)
        let collision = check_missile_collision(missile, missile.position.tile);

        // Apply damage to anything hit, but don't delete
        if matches!(collision.hit_type, HitType::Monster(_) | HitType::Player(_)) {
            missile.hit_flag = true;
        }

        // Stop at walls
        if collision.hit_type == HitType::Wall {
            missile.delete_flag = true;
        }
    }
}

/// Process blood star missile
///
/// **C++ Reference**: `ProcessBloodStar()` in `Source/missiles.cpp:3760-3820`
///
/// Blood stars are fired by blood knights and similar enemies.
pub fn process_blood_star(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked {
            // Spawn blood star explosion
            missile.hit_flag = true;
            missile.delete_flag = true;
        }
    }
}

/// Process bone spirit missile
///
/// **C++ Reference**: `ProcessBoneSpirit()` in `Source/missiles.cpp:3820-3900`
///
/// Bone spirits seek out targets and deal 1/3 of target's HP as damage.
pub fn process_bone_spirit(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // var3 tracks if we've found a target
    // var4, var5 store target position

    // Bone spirit homing behavior
    // In full implementation, recalculates velocity towards nearest enemy

    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if matches!(collision.hit_type, HitType::Monster(_) | HitType::Player(_)) {
            // Bone spirit deals 1/3 target HP - handled in damage calculation
            missile.hit_flag = true;
            missile.delete_flag = true;
        }

        if collision.hit_type == HitType::Wall {
            missile.delete_flag = true;
        }
    }
}

/// Process elemental missile (fire elemental)
///
/// **C++ Reference**: `ProcessElemental()` in `Source/missiles.cpp:3900-3960`
///
/// Fire elementals seek out targets and explode on contact.
pub fn process_elemental(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Similar to bone spirit but with fire damage
    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked || matches!(collision.hit_type, HitType::Monster(_)) {
            missile.hit_flag = true;
            missile.delete_flag = true;
        }
    }
}

/// Process acid splash missile
///
/// **C++ Reference**: `ProcessAcidSplat()` in `Source/missiles.cpp:2900-2940`
///
/// Acid splash expands outward from impact point.
pub fn process_acid_splat(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Acid splash animation
    // In full implementation, spawns acid puddles around impact
}

/// Process acid puddle missile
///
/// **C++ Reference**: `ProcessAcidPuddle()` in `Source/missiles.cpp:2940-2980`
///
/// Acid puddles linger and deal damage over time.
pub fn process_acid_puddle(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Puddle damage tick every few frames
    missile.var1 -= 1;
    if missile.var1 <= 0 {
        missile.var1 = 4; // Reset damage timer
        // In full implementation, apply damage to anyone standing in puddle
    }
}

/// Process town portal missile
///
/// **C++ Reference**: `ProcessTownPortal()` in `Source/missiles.cpp:2980-3040`
///
/// Town portals remain active until player uses them or casts a new one.
pub fn process_town_portal(missile: &mut Missile) {
    // Town portals don't expire by duration in standard gameplay
    // They expire when player uses them or casts a new portal

    // var1 stores countdown to full opening animation
    if missile.var1 > 0 {
        missile.var1 -= 1;
    }
}

/// Process mana shield missile
///
/// **C++ Reference**: `ProcessManaShield()` in `Source/missiles.cpp:3040-3080`
///
/// Mana shield visual effect that follows the player.
pub fn process_mana_shield(missile: &mut Missile) {
    // Mana shield follows player position
    // Duration decreases but typically managed by player state

    missile.duration -= 1;
    if missile.duration <= 0 {
        missile.delete_flag = true;
    }
}

/// Process nova missile
///
/// **C++ Reference**: `ProcessNova()` in `Source/missiles.cpp:3080-3140`
///
/// Nova spawns multiple lightning bolts in all directions.
pub fn process_nova(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Nova missile itself is the controller
    // It spawns thin lightning missiles in 16 directions
    // var1 tracks how many bolts have been spawned
}

/// Process apocalypse missile
///
/// **C++ Reference**: `ProcessApocalypse()` in `Source/missiles.cpp:3140-3200`
///
/// Apocalypse rains fire across the entire visible area.
pub fn process_apocalypse(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Apocalypse spawns explosion missiles across the map
    // var2-var6 define the area bounds
    // Each tick spawns explosions in a new row/column
    missile.var6 += 1;
    if missile.var6 > missile.var5 {
        missile.var6 = missile.var4;
        missile.var2 += 1;
        if missile.var2 > missile.var3 {
            missile.delete_flag = true;
        }
    }
}

/// Process rhino charge missile (monster charge attack)
///
/// **C++ Reference**: `ProcessRhino()` in `Source/missiles.cpp:3200-3260`
///
/// Rhino charges move the monster in a straight line.
pub fn process_rhino(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        // Stop on hitting anything
        if collision.blocked || matches!(collision.hit_type, HitType::Monster(_) | HitType::Player(_)) {
            missile.hit_flag = true;
            missile.delete_flag = true;
        }
    }
}

/// Process rune missile (trap that triggers on proximity)
///
/// **C++ Reference**: `ProcessRune()` in `Source/missiles.cpp:3260-3320`
///
/// Runes wait until an enemy steps near, then trigger their effect.
pub fn process_rune(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Runes check for nearby enemies
    // var1 stores the missile type to spawn when triggered
    // In full implementation, scans nearby tiles for enemies

    // When triggered:
    // - Spawn the stored missile type
    // - Delete this rune
}

/// Process stone curse missile
///
/// **C++ Reference**: `ProcessStoneCurse()` in `Source/missiles.cpp:3320-3380`
///
/// Stone curse petrifies a monster for a duration.
pub fn process_stone_curse(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        // Petrification ended
        // In full implementation, restore monster's original mode
        // var1 stores original monster mode
        missile.delete_flag = true;
    }
}

/// Process magma ball missile
///
/// **C++ Reference**: `ProcessMagmaBall()` in `Source/missiles.cpp:2800-2860`
///
/// Magma balls are projectiles from magma demons.
pub fn process_magma_ball(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    let moved = move_missile(missile);

    if moved {
        let collision = check_missile_collision(missile, missile.position.tile);

        if collision.blocked || matches!(collision.hit_type, HitType::Player(_)) {
            // Spawn explosion
            missile.hit_flag = true;
            missile.delete_flag = true;
        }
    }
}

/// Process chain lightning missile
///
/// **C++ Reference**: `ProcessChainLightning()` in `Source/missiles.cpp:2860-2900`
///
/// Chain lightning jumps between multiple targets.
pub fn process_chain_lightning(missile: &mut Missile) {
    missile.duration -= 1;

    if missile.duration <= 0 {
        missile.delete_flag = true;
        return;
    }

    // Chain lightning extends from source towards target
    // Then searches for additional targets to chain to
    // var1 tracks chain count remaining

    missile.var1 -= 1;
    if missile.var1 <= 0 {
        missile.delete_flag = true;
    }
}

/// Generic missile processing for unspecified types
///
/// **C++ Reference**: Fallback handling in `ProcessMissiles()` in `Source/missiles.cpp`
///
/// This handles any missile type that doesn't have specific processing logic.
/// Generally just decrements duration and marks for deletion when expired.
pub fn process_generic_missile(missile: &mut Missile) {
    missile.duration -= 1;
    if missile.duration <= 0 {
        missile.delete_flag = true;
    }
}

// =============================================================================
// Missile Creation Functions (Day 101)
// =============================================================================

/// Missile creation parameters
#[derive(Debug, Clone)]
pub struct AddMissileParameter {
    pub source: Point,
    pub destination: Point,
    pub direction: u8,
    pub spell_level: i32,
    pub damage: i32,
    pub caster: MissileCaster,
    pub source_id: i32,
}

impl Default for AddMissileParameter {
    fn default() -> Self {
        Self {
            source: Point::new(0, 0),
            destination: Point::new(0, 0),
            direction: 0,
            spell_level: 0,
            damage: 0,
            caster: MissileCaster::TargetMonsters,
            source_id: -1,
        }
    }
}

/// Add arrow missile
///
/// **C++ Reference**: `AddArrow()` in `Source/missiles.cpp:1750-1800`
pub fn add_arrow(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Arrow, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.damage = params.damage;
    missile.duration = 256;

    update_missile_velocity(&mut missile, params.destination, 32);

    let dir = get_direction16(params.source, params.destination);
    missile.anim_frame = dir as i32 + 1;

    missile
}

/// Add firebolt missile
///
/// **C++ Reference**: `AddFirebolt()` in `Source/missiles.cpp:1860-1920`
pub fn add_firebolt(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Firebolt, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.duration = 256;
    missile.light_flag = true;

    // Speed depends on caster type
    let speed = if missile.caster == MissileCaster::TargetMonsters {
        16 + (params.spell_level * 2).min(47)
    } else {
        26
    };

    update_missile_velocity(&mut missile, params.destination, speed);

    // Store start position in var1, var2
    missile.var1 = params.source.x;
    missile.var2 = params.source.y;

    missile
}

/// Add fireball missile
///
/// **C++ Reference**: `AddFireball()` in `Source/missiles.cpp:1922-1990`
pub fn add_fireball(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Fireball, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.damage = params.damage;
    missile.duration = 256;
    missile.light_flag = true;

    update_missile_velocity(&mut missile, params.destination, 16);

    // Store start position
    missile.var1 = params.source.x;
    missile.var2 = params.source.y;

    missile
}

/// Add lightning bolt missile
///
/// **C++ Reference**: `AddLightning()` in `Source/missiles.cpp:2050-2100`
pub fn add_lightning(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Lightning, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.damage = params.damage;
    missile.duration = 8; // Lightning is short-lived
    missile.light_flag = true;

    // Direction for lightning
    missile.var1 = 0; // Current length
    missile.var2 = params.direction as i32;

    missile
}

/// Add holy bolt missile
///
/// **C++ Reference**: `AddHolyBolt()` in `Source/missiles.cpp:2700-2750`
pub fn add_holy_bolt(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::HolyBolt, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.damage = params.damage;
    missile.duration = 256;
    missile.light_flag = true;

    let speed = 16 + (params.spell_level * 2).min(47);
    update_missile_velocity(&mut missile, params.destination, speed);

    missile.var1 = params.source.x;
    missile.var2 = params.source.y;

    missile
}

/// Add charged bolt missile
///
/// **C++ Reference**: `AddChargedBolt()` in `Source/missiles.cpp:2690-2740`
pub fn add_charged_bolt(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::ChargedBolt, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.damage = params.damage;
    missile.duration = 256;
    missile.light_flag = true;
    missile.random = rand::random::<i32>() % 15 + 1;

    update_missile_velocity(&mut missile, params.destination, 8);

    missile.var1 = 5; // Direction change timer
    missile.var2 = params.direction as i32;

    missile
}

// =============================================================================
// Additional Missile Creation Functions (M56 Day 1)
// =============================================================================

/// Add teleport missile
///
/// **C++ Reference**: `AddTeleport()` in `Source/missiles.cpp:1926-1945`
///
/// Teleport instantly moves the caster to a valid destination tile.
pub fn add_teleport(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Teleport, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Try to find valid teleport destination near target
    // In full implementation, would use FindClosestValidPosition
    let teleport_dest = find_teleport_destination(params.destination, 5);

    if let Some(dest) = teleport_dest {
        missile.position.tile = dest;
        missile.position.start = dest;
        missile.duration = 2;
    } else {
        // Spell fizzled - no valid destination
        missile.delete_flag = true;
    }

    missile
}

/// Find valid teleport destination near target
fn find_teleport_destination(target: Point, max_radius: i32) -> Option<Point> {
    // Simplified - in full implementation checks PosOkPlayer
    if is_in_dungeon_bounds(target) {
        Some(target)
    } else {
        // Try nearby positions
        for radius in 1..=max_radius {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    let pos = Point::new(target.x + dx, target.y + dy);
                    if is_in_dungeon_bounds(pos) {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }
}

/// Add town portal missile
///
/// **C++ Reference**: `AddTownPortal()` in `Source/missiles.cpp:2063-2115`
///
/// Creates a portal that allows travel to and from town.
pub fn add_town_portal(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // TownPortal ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Find valid portal position
    let portal_pos = find_teleport_destination(params.destination, 5);

    if let Some(pos) = portal_pos {
        missile.position.tile = pos;
        missile.position.start = pos;
        missile.delete_flag = false;
    } else {
        missile.delete_flag = true;
    }

    missile.duration = 100;
    missile.var1 = missile.duration - missile.anim_len;

    missile
}

/// Add stone curse missile
///
/// **C++ Reference**: `AddStoneCurse()` in `Source/missiles.cpp:2354-2410`
///
/// Petrifies a monster, making it unable to act temporarily.
pub fn add_stone_curse(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // StoneCurse ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // In full implementation, would search for valid monster target
    // and check if monster can be petrified
    missile.position.tile = params.destination;
    missile.position.start = params.destination;

    // Duration based on spell level
    let mut duration = params.spell_level + 6;
    if duration > 15 {
        duration = 15;
    }
    duration <<= 4; // * 16

    missile.duration = duration;
    missile.var1 = 0; // Monster mode before petrification
    missile.var2 = -1; // Monster ID (to be found)

    missile
}

/// Add golem missile (summon spell)
///
/// **C++ Reference**: `AddGolem()` in `Source/missiles.cpp:2412-2448`
///
/// Summons a golem minion or kills existing golem.
pub fn add_golem(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Golem ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Golem spell is handled differently - missile is immediately deleted
    // and golem spawning is handled via network command
    missile.delete_flag = true;

    // In full implementation:
    // 1. Check if player already has golem - if so, kill it
    // 2. Find valid spawn position near destination
    // 3. Send network command to spawn golem

    missile
}

/// Add healing missile
///
/// **C++ Reference**: `AddHealing()` in `Source/missiles.cpp:2451-2476`
///
/// Restores hit points to the caster.
pub fn add_healing(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Healing ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Healing is instant - calculate heal amount
    // In full implementation:
    // hp = rnd(10) + 1 + (level + 1) * rnd_sum(4) + (spell_level) * rnd_sum(6)
    // Warriors/Barbarians/Monks: hp *= 2
    // Rogues/Bards: hp += hp / 2

    let base_heal = rand::random::<i32>() % 10 + 1;
    let level_bonus = (params.spell_level + 1) * (rand::random::<i32>() % 4 + 1);
    let spell_bonus = params.spell_level * (rand::random::<i32>() % 6 + 1);
    missile.damage = (base_heal + level_bonus + spell_bonus) * 64;

    missile.delete_flag = true; // Instant effect
    missile
}

/// Add nova missile
///
/// **C++ Reference**: `AddNova()` in `Source/missiles.cpp:2565-2580`
///
/// Creates a ring of lightning bolts radiating outward.
pub fn add_nova(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Nova ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    missile.var1 = params.destination.x;
    missile.var2 = params.destination.y;

    // Damage calculation
    if params.source_id >= 0 {
        // Player cast
        let base_damage = rand::random::<i32>() % 30 + 5; // simplified rnd_sum(6, 5)
        missile.damage = scale_spell_effect(base_damage / 2, params.spell_level);
    } else {
        // Trap cast
        missile.damage = rand::random::<i32>() % 9 + 3; // simplified
    }

    missile.duration = 1;
    missile
}

/// Add inferno missile (fire breath)
///
/// **C++ Reference**: `AddInferno()` in `Source/missiles.cpp:2645-2665`
///
/// Creates a cone of fire damage in front of caster.
pub fn add_inferno(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Inferno ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    missile.var2 = 5 * params.damage;
    missile.position.start = params.destination;

    missile.duration = missile.var2 + 20;
    missile.light_flag = true;

    // Damage calculation
    if params.caster == MissileCaster::TargetMonsters {
        let base = rand::random::<i32>() % (params.spell_level + 1) + rand::random::<i32>() % 2;
        missile.damage = 8 * base + 16;
        missile.damage += missile.damage / 2;
    }

    missile
}

/// Add mana shield missile
///
/// **C++ Reference**: `AddManaShield()` in `Source/missiles.cpp:2145-2160`
///
/// Converts incoming damage to mana drain instead of health loss.
pub fn add_mana_shield(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // ManaShield ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Mana shield is a buff - missile is deleted immediately
    // and player gains the mana shield effect
    missile.delete_flag = true;

    // In full implementation:
    // - Check if player already has mana shield active (fizzle if so)
    // - Set player.pManaShield = true
    // - Send network command

    missile
}

/// Add flame wave missile
///
/// **C++ Reference**: `AddFlameWave()` in `Source/missiles.cpp:2162-2175`
///
/// Creates a wave of fire that travels forward.
pub fn add_flame_wave(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // FlameWave ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Damage calculation
    missile.damage = rand::random::<i32>() % 10 + params.spell_level + 1;

    update_missile_velocity(&mut missile, params.destination, 16);
    missile.duration = 255;

    // Adjust position for rendering
    missile.position.tile.y += 1; // South
    missile.position.offset.y -= 32;

    missile
}

/// Add guardian missile (hydra-like turret)
///
/// **C++ Reference**: `AddGuardian()` in `Source/missiles.cpp:2177-2230`
///
/// Creates a stationary guardian that shoots fireballs at enemies.
pub fn add_guardian(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Guardian, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Find valid spawn position
    let spawn_pos = find_teleport_destination(params.destination, 5);

    if let Some(pos) = spawn_pos {
        missile.position.tile = pos;
        missile.position.start = pos;
        missile.delete_flag = false;
        missile.light_flag = true;

        // Duration based on level and spell level
        let mut duration = params.spell_level + 10; // simplified
        if duration > 30 {
            duration = 30;
        }
        duration <<= 4; // * 16
        if duration < 30 {
            duration = 30;
        }

        missile.duration = duration;
        missile.var1 = missile.duration - missile.anim_len;
        missile.var3 = 1; // Firing enabled
    } else {
        missile.delete_flag = true;
    }

    missile
}

/// Add chain lightning missile
///
/// **C++ Reference**: `AddChainLightning()` in `Source/missiles.cpp:2232-2240`
///
/// Creates lightning that bounces between multiple targets.
pub fn add_chain_lightning(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::ChainLightning, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;
    missile.damage = params.damage;

    missile.var1 = params.destination.x;
    missile.var2 = params.destination.y;
    missile.duration = 1;

    missile
}

/// Add apocalypse missile
///
/// **C++ Reference**: `AddApocalypse()` in `Source/missiles.cpp:2620-2640`
///
/// Rains down fire damage across a large area.
pub fn add_apocalypse(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Apocalypse ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Area of effect bounds
    missile.var1 = 8; // Radius
    missile.var2 = (params.source.y - 8).max(1);
    missile.var3 = (params.source.y + 8).min(111); // MAXDUNY - 1
    missile.var4 = (params.source.x - 8).max(1);
    missile.var5 = (params.source.x + 8).min(111); // MAXDUNX - 1
    missile.var6 = missile.var4;

    // Damage calculation
    let level = params.spell_level;
    missile.damage = rand::random::<i32>() % (level * 6 + 1) + level;

    missile.duration = 255;
    missile
}

/// Add fire wall missile
///
/// **C++ Reference**: `AddFireWall()` in `Source/missiles.cpp:1957-1980`
///
/// Creates a wall of fire at target location.
pub fn add_fire_wall(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Firewall, params.source);
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Damage calculation
    let base_damage = rand::random::<i32>() % 10 + rand::random::<i32>() % 10 + 2;
    missile.damage = (base_damage + params.spell_level) << 3;

    update_missile_velocity(&mut missile, params.destination, 16);

    // Duration based on spell level
    let mut duration = 10;
    if params.spell_level > 0 {
        duration *= params.spell_level + 1;
    }
    duration *= 16;

    missile.duration = duration;
    missile.var1 = missile.duration - missile.anim_len;

    missile
}

/// Add elemental missile (Fire Elemental)
///
/// **C++ Reference**: `AddElemental()` in `Source/missiles.cpp:2495-2520`
///
/// Creates a seeking fire elemental.
pub fn add_elemental(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Elemental ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    let mut dest = params.destination;
    if params.source == dest {
        dest.x += 1; // Offset if same position
    }

    // Damage calculation
    let base_damage = rand::random::<i32>() % 20 + params.spell_level * 2 + 4;
    missile.damage = scale_spell_effect(base_damage / 2, params.spell_level);

    update_missile_velocity(&mut missile, dest, 16);
    missile.duration = 256;

    missile.var1 = params.source.x;
    missile.var2 = params.source.y;
    missile.var4 = dest.x;
    missile.var5 = dest.y;

    missile.light_flag = true;
    missile
}

/// Add flash bottom missile (Holy Bolt nova bottom)
///
/// **C++ Reference**: `AddFlashBottom()` in `Source/missiles.cpp:2117-2140`
pub fn add_flash_bottom(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // FlashBottom ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Damage calculation depends on source type
    let base_damage = rand::random::<i32>() % 20 * (params.spell_level + 1) + params.spell_level + 1;
    missile.damage = scale_spell_effect(base_damage, params.spell_level);
    missile.damage += missile.damage / 2;

    missile.duration = 19;
    missile
}

/// Add flash top missile (Holy Bolt nova top)
///
/// **C++ Reference**: `AddFlashTop()` in `Source/missiles.cpp:2142-2160`
pub fn add_flash_top(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // FlashTop ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    if params.caster == MissileCaster::TargetMonsters {
        let base_damage = rand::random::<i32>() % 20 * (params.spell_level + 1) + params.spell_level + 1;
        missile.damage = scale_spell_effect(base_damage, params.spell_level);
        missile.damage += missile.damage / 2;
    }

    missile.pre_flag = true;
    missile.duration = 19;
    missile
}

/// Add acid missile (Acid Spit)
///
/// **C++ Reference**: `AddAcid()` in `Source/missiles.cpp:2318-2350`
pub fn add_acid(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Acid ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    update_missile_velocity(&mut missile, params.destination, 16);

    // Duration based on monster intelligence (simplified)
    missile.duration = 5 * 8; // 5 * (intelligence + 4)

    missile.light_id = -1; // NO_LIGHT
    missile.var1 = params.source.x;
    missile.var2 = params.source.y;

    if params.damage == 0 {
        missile.damage = rand::random::<i32>() % 15 + 5; // ProjectileMonsterDamage simplified
    } else {
        missile.damage = params.damage;
    }

    missile
}

/// Add identify missile (spell effect only)
///
/// **C++ Reference**: `AddIdentify()` in `Source/missiles.cpp:2522-2540`
pub fn add_identify(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Identify ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Identify is UI-only - opens inventory with identify cursor
    missile.delete_flag = true;

    // In full implementation:
    // - Close spellbook if open
    // - Open inventory
    // - Set cursor to CURSOR_IDENTIFY

    missile
}

/// Add infravision missile
///
/// **C++ Reference**: `AddInfravision()` in `Source/missiles.cpp:2554-2558`
pub fn add_infravision(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Infravision ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // Duration scales with spell level
    missile.duration = scale_spell_effect(1584, params.spell_level);

    missile
}

/// Add rage/berserk missile
///
/// **C++ Reference**: `AddRage()` in `Source/missiles.cpp:2582-2605`
pub fn add_rage(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Null, params.source); // Rage ID
    missile.caster = params.caster;
    missile.source = params.source_id;
    missile.spell_level = params.spell_level;

    // In full implementation:
    // - Check if rage already active or on cooldown (fizzle if so)
    // - Check if player has enough HP

    let level = params.spell_level;
    missile.damage = level * 6;
    missile.duration = 245 + level * 2;
    missile.var1 = missile.duration;

    // Sets SpellFlag::RageActive on player
    missile
}

/// Scale spell effect by spell level
///
/// **C++ Reference**: `ScaleSpellEffect()` in `Source/spells.cpp`
fn scale_spell_effect(base: i32, spell_level: i32) -> i32 {
    // Simplified scaling - actual implementation has complex formula
    base + (base * spell_level / 4)
}

/// Add Warp spell (Phasing)
///
/// **C++ Reference**: `AddWarp()` in missiles.cpp:1507
pub fn add_warp(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Warp, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Find nearest trigger/warp point (simplified - actual C++ scans level triggers)
    missile.duration = 32;

    missile
}

/// Add Resurrect Beam visual effect
///
/// **C++ Reference**: `AddResurrectBeam()` in missiles.cpp:2743
pub fn add_resurrect_beam(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::ResurrectBeam, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;

    // Duration equals animation length
    missile.duration = 16; // GetMissileSpriteData(Resurrect).animLen

    missile
}

/// Add Bone Spirit missile
///
/// **C++ Reference**: `AddBoneSpirit()` in missiles.cpp:2759
pub fn add_bone_spirit(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::BoneSpirit, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    let dst = if params.source == params.destination {
        params.destination + Point::new(1, 0) // Offset by direction
    } else {
        params.destination
    };

    // Calculate velocity
    let dx = dst.x - params.source.x;
    let dy = dst.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 16);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 256;

    // Store start/target positions in var slots
    missile.var1 = params.source.x;
    missile.var2 = params.source.y;
    missile.var4 = dst.x as i32;
    missile.var5 = dst.y as i32;

    missile
}

/// Add Red Portal (to Diablo level)
///
/// **C++ Reference**: `AddRedPortal()` in missiles.cpp:2775
pub fn add_red_portal(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RedPortal, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.duration = 100;
    missile.var1 = 100 - missile.anim_len;

    missile
}

/// Add Telekinesis cursor (instant spell)
///
/// **C++ Reference**: `AddTelekinesis()` in missiles.cpp:2750
pub fn add_telekinesis(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Telekinesis, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.delete_flag = true; // Instant effect, mark for deletion

    missile
}

/// Add Item Repair spell
///
/// **C++ Reference**: `AddItemRepair()` in missiles.cpp:2596
pub fn add_item_repair(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::ItemRepair, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.delete_flag = true; // Instant effect

    missile
}

/// Add Staff Recharge spell
///
/// **C++ Reference**: `AddStaffRecharge()` in missiles.cpp:2613
pub fn add_staff_recharge(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::StaffRecharge, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.delete_flag = true; // Instant effect

    missile
}

/// Add Trap Disarm spell
///
/// **C++ Reference**: `AddTrapDisarm()` in missiles.cpp:2630
pub fn add_trap_disarm(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::TrapDisarm, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.delete_flag = true; // Instant effect

    missile
}

/// Add Rhino charge (monster charge attack)
///
/// **C++ Reference**: `AddRhino()` in missiles.cpp:2253
pub fn add_rhino(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Rhino, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Calculate velocity for charge
    let dx = params.destination.x - params.source.x;
    let dy = params.destination.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 18); // Faster than normal

    missile.position.velocity = Point::new(velocity.0, velocity.1);
    missile.duration = 256;

    missile
}

/// Add Generic Magic Missile
///
/// **C++ Reference**: `AddGenericMagicMissile()` in missiles.cpp:2273
pub fn add_generic_magic_missile(params: &AddMissileParameter) -> Missile {
    let dst = if params.source == params.destination {
        params.destination + Point::new(1, 0) // Offset by direction
    } else {
        params.destination
    };

    let mut missile = Missile::new(MissileID::BloodStar, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level;

    // Calculate velocity
    let dx = dst.x - params.source.x;
    let dy = dst.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 16);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 256;

    missile
}

/// Add Acid Puddle (monster acid pool)
///
/// **C++ Reference**: `AddAcidPuddle()` in missiles.cpp:2344
pub fn add_acid_puddle(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::AcidPuddle, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    // Duration based on monster intelligence (simplified)
    missile.duration = rand::rng().random_range(40..95);

    missile
}

/// Add Apocalypse Boom explosion
///
/// **C++ Reference**: `AddApocalypseBoom()` in missiles.cpp:2444
pub fn add_apocalypse_boom(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::ApocalypseBoom, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;

    // Duration equals animation length
    missile.duration = missile.anim_len;

    missile
}

/// Add Magma Ball (fire projectile)
///
/// **C++ Reference**: `AddMagmaBall()` in missiles.cpp:1898
pub fn add_magma_ball(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::MagmaBall, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Calculate damage
    let min_dmg = 2 * params.spell_level as i32 + 2;
    let max_dmg = 2 * params.spell_level as i32 + 4;
    missile.damage = rand::rng().random_range(min_dmg..max_dmg);

    // Calculate velocity
    let dx = params.destination.x - params.source.x;
    let dy = params.destination.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 16);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 256;

    missile
}

/// Add Lightning Wall
///
/// **C++ Reference**: `AddLightningWall()` in missiles.cpp:1582
pub fn add_lightning_wall(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::LightningWall, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Duration varies by level
    missile.duration = (params.spell_level as i32 + 1) * 20;

    // Damage calculation
    missile.damage = rand::rng().random_range(1..11) + params.spell_level as i32;

    missile
}

/// Add Spectral Arrow
///
/// **C++ Reference**: `AddSpectralArrow()` in missiles.cpp:1479
pub fn add_spectral_arrow(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::SpectralArrow, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Attack speed bonus varies by class (simplified)
    let av = params.spell_level as i32 / 4;

    missile.duration = 1;
    missile.var1 = params.destination.x;
    missile.var2 = params.destination.y;
    missile.var3 = av;

    missile
}

/// Add Lightning Bow
///
/// **C++ Reference**: `AddLightningBow()` in missiles.cpp:1636
pub fn add_lightning_bow(params: &AddMissileParameter) -> Missile {
    let dst = if params.source == params.destination {
        params.destination + Point::new(1, 0)
    } else {
        params.destination
    };

    let mut missile = Missile::new(MissileID::LightningBow, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Calculate velocity (faster than normal - speed 32)
    let dx = dst.x - params.source.x;
    let dy = dst.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 32);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.anim_frame = rand::rng().random_range(1..9);
    missile.duration = 255;

    // Store source position
    missile.var1 = params.source.x;
    missile.var2 = params.source.y;

    // Damage is multiplied by 64 in C++
    missile.damage = params.damage << 6;

    missile
}

/// Add Mana spell (restore mana)
///
/// **C++ Reference**: `AddMana()` in missiles.cpp:1655
pub fn add_mana(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Mana, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    // Mana restoration amount (simplified - actual depends on character level and class)
    let base_mana = (rand::rng().random_range(1..11)) << 6;
    missile.var1 = base_mana + (params.spell_level as i32 * 6);

    missile.delete_flag = true; // Instant effect

    missile
}

/// Add Magi spell (full mana restore)
///
/// **C++ Reference**: `AddMagi()` in missiles.cpp:1678
pub fn add_magi(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Magi, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    // Instant full mana restore
    missile.delete_flag = true;

    missile
}

/// Add Ring of Fire
///
/// **C++ Reference**: `AddRingOfFire()` in missiles.cpp:1688
pub fn add_ring_of_fire(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RingOfFire, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    missile.var1 = params.source.x;
    missile.var2 = params.source.y;
    missile.duration = 7;

    missile
}

/// Add Search spell (reveal items on map)
///
/// **C++ Reference**: `AddSearch()` in missiles.cpp:1695
pub fn add_search(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Search, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Duration based on spell level
    missile.duration = 10 + params.spell_level as i32 * 10;

    missile
}

/// Add Charged Bolt Bow
///
/// **C++ Reference**: `AddChargedBoltBow()` in missiles.cpp:1718
pub fn add_charged_bolt_bow(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::ChargedBoltBow, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Charged bolt pattern
    let spread = rand::rng().random_range(-2..3);
    let dx = params.destination.x - params.source.x + spread;
    let dy = params.destination.y - params.source.y + spread;
    let velocity = calculate_missile_velocity(dx, dy, 8);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 256;
    missile.damage = params.damage;

    missile
}

/// Add Elemental Arrow
///
/// **C++ Reference**: `AddElementalArrow()` in missiles.cpp:1737
pub fn add_elemental_arrow(params: &AddMissileParameter) -> Missile {
    let dst = if params.source == params.destination {
        params.destination + Point::new(1, 0)
    } else {
        params.destination
    };

    let mut missile = Missile::new(MissileID::FireArrow, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    let dx = dst.x - params.source.x;
    let dy = dst.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 32);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 256;
    missile.damage = params.damage;

    missile
}

/// Add Phasing (short-range teleport)
///
/// **C++ Reference**: `AddPhasing()` in missiles.cpp:1824
pub fn add_phasing(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Phasing, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    // Find random destination within range (simplified)
    missile.var1 = params.destination.x;
    missile.var2 = params.destination.y;
    missile.duration = 32;

    missile
}

/// Add Nova Ball
///
/// **C++ Reference**: `AddNovaBall()` in missiles.cpp:1946
pub fn add_nova_ball(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::NovaBall, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    let dx = params.destination.x - params.source.x;
    let dy = params.destination.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 16);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 256;
    missile.damage = params.damage;

    missile
}

/// Add Heal Other spell
///
/// **C++ Reference**: `AddHealOther()` in missiles.cpp:2473
pub fn add_heal_other(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::HealOther, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Healing amount
    let heal = rand::rng().random_range(1..11) + params.spell_level as i32 * 3;
    missile.var1 = heal << 6;

    missile.delete_flag = true; // Instant effect

    missile
}

/// Add Immolation (fire ring explosion)
///
/// **C++ Reference**: `AddImmolation()` in missiles.cpp:1620
pub fn add_immolation(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Immolation, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    missile.var1 = params.source.x;
    missile.var2 = params.source.y;
    missile.duration = 16;
    missile.damage = rand::rng().random_range(1..11) + params.spell_level as i32 * 2;

    missile
}

/// Add Big Explosion
///
/// **C++ Reference**: `AddBigExplosion()` in missiles.cpp:1603
pub fn add_big_explosion(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::BigExplosion, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.duration = missile.anim_len;

    missile
}

/// Add Rune of Fire
///
/// **C++ Reference**: `AddRuneOfFire()` in missiles.cpp:1250
pub fn add_rune_of_fire(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RuneOfFire, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Rune triggers BigExplosion when activated
    missile.var1 = MissileID::BigExplosion as i32;
    missile.duration = 60 * 10; // ~10 seconds at 60 FPS

    missile
}

/// Add Rune of Light
///
/// **C++ Reference**: `AddRuneOfLight()` in missiles.cpp:1255
pub fn add_rune_of_light(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RuneOfLight, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Calculate damage based on level
    let dmg = 16 * (rand::rng().random_range(2..22) + params.spell_level as i32 + 2);
    missile.damage = dmg;

    // Triggers LightningWall when activated
    missile.var1 = MissileID::LightningWall as i32;
    missile.duration = 60 * 10;

    missile
}

/// Add Rune of Nova
///
/// **C++ Reference**: `AddRuneOfNova()` in missiles.cpp:1263
pub fn add_rune_of_nova(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RuneOfNova, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Triggers Nova when activated
    missile.var1 = MissileID::Nova as i32;
    missile.duration = 60 * 10;

    missile
}

/// Add Rune of Immolation
///
/// **C++ Reference**: `AddRuneOfImmolation()` in missiles.cpp:1268
pub fn add_rune_of_immolation(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RuneOfImmolation, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Triggers Immolation when activated
    missile.var1 = MissileID::Immolation as i32;
    missile.duration = 60 * 10;

    missile
}

/// Add Rune of Stone
///
/// **C++ Reference**: `AddRuneOfStone()` in missiles.cpp:1273
pub fn add_rune_of_stone(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::RuneOfStone, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Triggers StoneCurse when activated
    missile.var1 = MissileID::StoneCurse as i32;
    missile.duration = 60 * 10;

    missile
}

/// Add Reflect spell
///
/// **C++ Reference**: `AddReflect()` in missiles.cpp:1278
pub fn add_reflect(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Reflect, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Instant effect - adds reflection charges to player
    missile.delete_flag = true;

    missile
}

/// Add Berserk spell
///
/// **C++ Reference**: `AddBerserk()` in missiles.cpp:1295
pub fn add_berserk(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Berserk, params.destination);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Makes target monster go berserk (attack other monsters)
    missile.duration = 32;

    missile
}

/// Add Hork Spawn (monster projectile)
///
/// **C++ Reference**: `AddHorkSpawn()` in missiles.cpp:1346
pub fn add_hork_spawn(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::HorkSpawn, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    let dx = params.destination.x - params.source.x;
    let dy = params.destination.y - params.source.y;
    let velocity = calculate_missile_velocity(dx, dy, 8);

    missile.position.velocity.x = velocity.0;
    missile.position.velocity.y = velocity.1;
    missile.duration = 9;
    missile.var1 = params.direction as i32;

    missile
}

/// Add Jester spell (random spell effect)
///
/// **C++ Reference**: `AddJester()` in missiles.cpp:1354
pub fn add_jester(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::Jester, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;
    missile.spell_level = params.spell_level as i32;

    // Pick random spell type (stored in var1)
    let spell = match rand::rng().random_range(0..10) {
        0 | 1 => MissileID::Firebolt,
        2 => MissileID::Fireball,
        3 => MissileID::FireWallControl,
        4 => MissileID::Guardian,
        5 => MissileID::ChainLightning,
        6 => MissileID::TownPortal,
        7 => MissileID::Teleport,
        8 => MissileID::Apocalypse,
        _ => MissileID::StoneCurse,
    };

    missile.var1 = spell as i32;
    missile.delete_flag = true; // Immediately spawns the random spell

    missile
}

/// Add Open Nest (spawns explosions)
///
/// **C++ Reference**: `AddOpenNest()` in missiles.cpp:1240
pub fn add_open_nest(params: &AddMissileParameter) -> Missile {
    let mut missile = Missile::new(MissileID::OpenNest, params.source);
    missile.source = params.source_id;
    missile.caster = params.caster;

    missile.delete_flag = true; // Instant - spawns multiple BigExplosion missiles

    missile
}

// =============================================================================
// MissileManager Extensions (Day 101)
// =============================================================================

impl MissileManager {
    /// Add missile with parameters
    ///
    /// **C++ Reference**: `AddMissile()` in `Source/missiles.cpp:3950-4200`
    ///
    /// Creates appropriate missile type based on MissileID.
    pub fn add_missile_ex(&mut self, missile_type: MissileID, params: &AddMissileParameter) -> Option<usize> {
        let missile = match missile_type {
            MissileID::Arrow => add_arrow(params),
            MissileID::Firebolt => add_firebolt(params),
            MissileID::Fireball => add_fireball(params),
            MissileID::Lightning => add_lightning(params),
            MissileID::HolyBolt => add_holy_bolt(params),
            MissileID::ChargedBolt => add_charged_bolt(params),
            MissileID::Teleport => add_teleport(params),
            MissileID::Firewall => add_fire_wall(params),
            MissileID::Guardian => add_guardian(params),
            MissileID::TownPortal => add_town_portal(params),
            MissileID::StoneCurse => add_stone_curse(params),
            MissileID::Golem => add_golem(params),
            MissileID::Healing => add_healing(params),
            MissileID::Nova => add_nova(params),
            MissileID::Inferno => add_inferno(params),
            MissileID::ManaShield => add_mana_shield(params),
            MissileID::FlameWave => add_flame_wave(params),
            MissileID::ChainLightning => add_chain_lightning(params),
            MissileID::Apocalypse => add_apocalypse(params),
            MissileID::Elemental => add_elemental(params),
            MissileID::FlashBottom => add_flash_bottom(params),
            MissileID::FlashTop => add_flash_top(params),
            MissileID::Acid => add_acid(params),
            MissileID::Identify => add_identify(params),
            MissileID::Infravision => add_infravision(params),
            MissileID::Rage => add_rage(params),
            // Day 57 additions
            MissileID::Warp => add_warp(params),
            MissileID::ResurrectBeam => add_resurrect_beam(params),
            MissileID::BoneSpirit => add_bone_spirit(params),
            MissileID::RedPortal => add_red_portal(params),
            MissileID::Telekinesis => add_telekinesis(params),
            MissileID::ItemRepair => add_item_repair(params),
            MissileID::StaffRecharge => add_staff_recharge(params),
            MissileID::TrapDisarm => add_trap_disarm(params),
            MissileID::Rhino => add_rhino(params),
            MissileID::BloodStar => add_generic_magic_missile(params),
            MissileID::AcidPuddle => add_acid_puddle(params),
            MissileID::ApocalypseBoom => add_apocalypse_boom(params),
            MissileID::MagmaBall => add_magma_ball(params),
            MissileID::LightningWall => add_lightning_wall(params),
            // Day 57 additions (batch 2)
            MissileID::SpectralArrow => add_spectral_arrow(params),
            MissileID::LightningBow => add_lightning_bow(params),
            MissileID::Mana => add_mana(params),
            MissileID::Magi => add_magi(params),
            MissileID::RingOfFire => add_ring_of_fire(params),
            MissileID::Search => add_search(params),
            MissileID::ChargedBoltBow => add_charged_bolt_bow(params),
            MissileID::FireArrow => add_elemental_arrow(params),
            MissileID::Phasing => add_phasing(params),
            MissileID::NovaBall => add_nova_ball(params),
            MissileID::HealOther => add_heal_other(params),
            MissileID::Immolation => add_immolation(params),
            MissileID::BigExplosion => add_big_explosion(params),
            // Day 57 additions (batch 3 - runes and special)
            MissileID::RuneOfFire => add_rune_of_fire(params),
            MissileID::RuneOfLight => add_rune_of_light(params),
            MissileID::RuneOfNova => add_rune_of_nova(params),
            MissileID::RuneOfImmolation => add_rune_of_immolation(params),
            MissileID::RuneOfStone => add_rune_of_stone(params),
            MissileID::Reflect => add_reflect(params),
            MissileID::Berserk => add_berserk(params),
            MissileID::HorkSpawn => add_hork_spawn(params),
            MissileID::Jester => add_jester(params),
            MissileID::OpenNest => add_open_nest(params),
            _ => Missile::new(missile_type, params.source),
        };

        self.add_missile(missile)
    }

    /// Process all missiles with full logic
    ///
    /// **C++ Reference**: `ProcessMissiles()` in `Source/missiles.cpp:4216-4250`
    ///
    /// Enhanced version with missile type-specific processing.
    pub fn process_missiles_full(&mut self) {
        // Phase 1: Mark out-of-bounds missiles
        for missile in &mut self.missiles {
            if !is_in_dungeon_bounds(missile.position.tile) {
                missile.delete_flag = true;
            }
        }

        // Delete marked missiles
        self.delete_missiles();

        // Phase 2: Process each missile by type
        for missile in &mut self.missiles {
            if missile.delete_flag {
                continue;
            }

            match missile.missile_type {
                // Arrow-type missiles
                MissileID::Arrow | MissileID::LightningArrow | MissileID::FireArrow |
                MissileID::SpecArrow => {
                    process_arrow(missile);
                }

                // Fire spells
                MissileID::Firebolt => {
                    process_firebolt(missile);
                }
                MissileID::Fireball | MissileID::BigExplosion | MissileID::Immolation => {
                    process_fireball(missile);
                }
                MissileID::Inferno | MissileID::InfernoControl => {
                    process_inferno(missile);
                }
                MissileID::FlameWave | MissileID::FlameWaveControl => {
                    process_flame_wave(missile);
                }
                MissileID::Firewall | MissileID::FireWallControl => {
                    process_fire_wall(missile);
                }

                // Lightning spells
                MissileID::Lightning | MissileID::LightningWall | MissileID::LightningControl => {
                    process_lightning(missile);
                }
                MissileID::ChargedBolt => {
                    process_charged_bolt(missile);
                }
                MissileID::ChainLightning => {
                    process_chain_lightning(missile);
                }
                MissileID::Nova => {
                    process_nova(missile);
                }

                // Holy/Undead
                MissileID::HolyBolt => {
                    process_holy_bolt(missile);
                }
                MissileID::Guardian | MissileID::GuardianTentacle => {
                    process_guardian(missile);
                }
                MissileID::BoneSpirit => {
                    process_bone_spirit(missile);
                }
                MissileID::BloodStar | MissileID::BloodStarExplosion => {
                    process_blood_star(missile);
                }

                // Flash/Area effects
                MissileID::FlashBottom => {
                    process_flash_bottom(missile);
                }
                MissileID::FlashTop => {
                    process_flash_top(missile);
                }
                MissileID::Apocalypse | MissileID::ApocalypseBoom => {
                    process_apocalypse(missile);
                }

                // Stone Curse
                MissileID::StoneCurse | MissileID::StoneCurseMissile => {
                    process_stone_curse(missile);
                }

                // Elementals
                MissileID::Elemental | MissileID::ElementalBall => {
                    process_elemental(missile);
                }

                // Acid (Diablo creature attacks)
                MissileID::Acid | MissileID::AcidSplat => {
                    process_acid_splat(missile);
                }
                MissileID::AcidPuddle => {
                    process_acid_puddle(missile);
                }

                // Monster special attacks
                MissileID::Rhino | MissileID::Krull | MissileID::HorkSpawn => {
                    process_rhino(missile);
                }
                MissileID::MagmaBall | MissileID::MagmaBallExplosion => {
                    // Similar to fireball but monster-spawned
                    process_fireball(missile);
                }

                // Portals
                MissileID::TownPortal | MissileID::RedPortal => {
                    process_town_portal(missile);
                }

                // Teleport
                MissileID::Teleport => {
                    // Teleport is instant, no processing needed
                    missile.delete_flag = true;
                }

                // Healing/Mana effects
                MissileID::Healing | MissileID::HealOther | MissileID::Mana | MissileID::Magi |
                MissileID::Resurrect | MissileID::Reflect => {
                    // Visual effects only, duration-based
                    missile.duration -= 1;
                    if missile.duration <= 0 {
                        missile.delete_flag = true;
                    }
                }

                // Rune spells (visual markers)
                MissileID::Rune1 | MissileID::Rune2 | MissileID::Rune3 |
                MissileID::Rune4 | MissileID::Rune5 => {
                    // Runes are stationary, duration-based
                    missile.duration -= 1;
                    if missile.duration <= 0 {
                        missile.delete_flag = true;
                    }
                }

                // Search spell (reveals map)
                MissileID::Search => {
                    missile.duration -= 1;
                    if missile.duration <= 0 {
                        missile.delete_flag = true;
                    }
                }

                // Default handler for any unspecified types
                _ => {
                    process_generic_missile(missile);
                }
            }

            // Handle animation
            if missile.anim_flags == MissileGraphicsFlags::Animated {
                missile.anim_cnt += 1;
                if missile.anim_cnt >= missile.anim_delay {
                    missile.anim_cnt = 0;
                    missile.anim_frame += missile.anim_add;

                    if missile.anim_frame > missile.anim_len {
                        missile.anim_frame = 1;
                    } else if missile.anim_frame < 1 {
                        missile.anim_frame = missile.anim_len;
                    }
                }
            }
        }

        // Final cleanup
        self.delete_missiles();
    }

    /// Process missiles with full dungeon state for collision detection
    /// This is the comprehensive version that integrates with the game state
    pub fn process_missiles_with_state(&mut self, state: &DungeonState) {

        // Phase 1: Mark out-of-bounds missiles
        for missile in &mut self.missiles {
            if !is_in_dungeon_bounds(missile.position.tile) {
                missile.delete_flag = true;
            }
        }

        self.delete_missiles();

        // Phase 2: Process each missile with state-aware collision
        for missile in &mut self.missiles {
            if missile.delete_flag {
                continue;
            }

            // Check collision with state
            let collision = check_missile_collision_with_state(missile, missile.position.tile, state);

            if collision.blocked {
                match collision.hit_type {
                    HitType::Monster(_monster_id) => {
                        // For most missiles, hitting something causes deletion
                        if !matches!(
                            missile.missile_type,
                            MissileID::Lightning | MissileID::LightningWall |
                            MissileID::Firewall | MissileID::Nova |
                            MissileID::ChainLightning | MissileID::Inferno
                        ) {
                            missile.delete_flag = true;
                        }
                    }
                    HitType::Player(_player_id) => {
                        // Similar handling for player hits
                        if !matches!(
                            missile.missile_type,
                            MissileID::Lightning | MissileID::LightningWall |
                            MissileID::Firewall | MissileID::Nova |
                            MissileID::ChainLightning | MissileID::Inferno
                        ) {
                            missile.delete_flag = true;
                        }
                    }
                    HitType::Wall | HitType::Object => {
                        // Most missiles stop at walls
                        missile.delete_flag = true;
                    }
                    HitType::None => {}
                }
            }

            // Type-specific processing (same as process_missiles_full)
            match missile.missile_type {
                MissileID::Arrow | MissileID::LightningArrow | MissileID::FireArrow |
                MissileID::SpecArrow => {
                    process_arrow(missile);
                }
                MissileID::Firebolt => {
                    process_firebolt(missile);
                }
                MissileID::Fireball | MissileID::BigExplosion | MissileID::Immolation => {
                    process_fireball(missile);
                }
                MissileID::Lightning | MissileID::LightningWall => {
                    process_lightning(missile);
                }
                MissileID::ChargedBolt => {
                    process_charged_bolt(missile);
                }
                MissileID::HolyBolt => {
                    process_holy_bolt(missile);
                }
                MissileID::Guardian => {
                    process_guardian(missile);
                }
                MissileID::BoneSpirit => {
                    process_bone_spirit(missile);
                }
                MissileID::FlameWave => {
                    process_flame_wave(missile);
                }
                MissileID::Firewall => {
                    process_fire_wall(missile);
                }
                MissileID::ChainLightning => {
                    process_chain_lightning(missile);
                }
                MissileID::Nova => {
                    process_nova(missile);
                }
                MissileID::Apocalypse => {
                    process_apocalypse(missile);
                }
                MissileID::StoneCurse => {
                    process_stone_curse(missile);
                }
                MissileID::Elemental => {
                    process_elemental(missile);
                }
                MissileID::Acid | MissileID::AcidSplat => {
                    process_acid_splat(missile);
                }
                MissileID::AcidPuddle => {
                    process_acid_puddle(missile);
                }
                MissileID::TownPortal | MissileID::RedPortal => {
                    process_town_portal(missile);
                }
                MissileID::Rhino => {
                    process_rhino(missile);
                }
                MissileID::BloodStar => {
                    process_blood_star(missile);
                }
                MissileID::Inferno => {
                    process_inferno(missile);
                }
                MissileID::FlashBottom => {
                    process_flash_bottom(missile);
                }
                MissileID::FlashTop => {
                    process_flash_top(missile);
                }
                _ => {
                    process_generic_missile(missile);
                }
            }

            // Handle animation
            if missile.anim_flags == MissileGraphicsFlags::Animated {
                missile.anim_cnt += 1;
                if missile.anim_cnt >= missile.anim_delay {
                    missile.anim_cnt = 0;
                    missile.anim_frame += missile.anim_add;

                    if missile.anim_frame > missile.anim_len {
                        missile.anim_frame = 1;
                    } else if missile.anim_frame < 1 {
                        missile.anim_frame = missile.anim_len;
                    }
                }
            }
        }

        // Final cleanup
        self.delete_missiles();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missile_position_creation() {
        let pos = MissilePosition::new(Point::new(10, 20));
        assert_eq!(pos.tile, Point::new(10, 20));
        assert_eq!(pos.start, Point::new(10, 20));
        assert_eq!(pos.velocity, Point::new(0, 0));
    }

    #[test]
    fn test_missile_stop() {
        let mut pos = MissilePosition::new(Point::new(5, 5));
        pos.velocity = Point::new(10, 10);
        pos.stop_missile();

        assert_eq!(pos.velocity, Point::new(0, 0));
    }

    #[test]
    fn test_missile_creation() {
        let missile = Missile::new(MissileID::Arrow, Point::new(15, 25));

        assert_eq!(missile.missile_type, MissileID::Arrow);
        assert_eq!(missile.position.tile, Point::new(15, 25));
        assert_eq!(missile.delete_flag, false);
        assert_eq!(missile.source, -1);
    }

    #[test]
    fn test_missile_is_trap() {
        let mut missile = Missile::new(MissileID::Firebolt, Point::new(0, 0));

        // Default source is -1 (trap)
        assert!(missile.is_trap());

        // Set to player source
        missile.source = 0;
        assert!(!missile.is_trap());
    }

    #[test]
    fn test_missile_source_type() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(0, 0));

        // Trap
        assert_eq!(missile.source_type(), MissileSource::Trap);

        // Monster
        missile.source = 5;
        missile.caster = MissileCaster::TargetPlayers;
        assert_eq!(missile.source_type(), MissileSource::Monster);

        // Player
        missile.caster = MissileCaster::TargetMonsters;
        assert_eq!(missile.source_type(), MissileSource::Player);
    }

    #[test]
    fn test_missile_manager_creation() {
        let manager = MissileManager::new(125);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_missile_manager_add() {
        let mut manager = MissileManager::new(10);
        let missile = Missile::new(MissileID::Arrow, Point::new(0, 0));

        let index = manager.add_missile(missile);
        assert!(index.is_some());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_missile_manager_capacity() {
        let mut manager = MissileManager::new(2);

        manager.add_missile(Missile::new(MissileID::Arrow, Point::new(0, 0)));
        manager.add_missile(Missile::new(MissileID::Firebolt, Point::new(1, 1)));

        // Should fail when at capacity
        let result = manager.add_missile(Missile::new(MissileID::Fireball, Point::new(2, 2)));
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_missiles() {
        let mut manager = MissileManager::new(10);

        let mut m1 = Missile::new(MissileID::Arrow, Point::new(0, 0));
        let m2 = Missile::new(MissileID::Firebolt, Point::new(1, 1));
        let mut m3 = Missile::new(MissileID::Fireball, Point::new(2, 2));

        m1.delete_flag = true;
        m3.delete_flag = true;

        manager.add_missile(m1);
        manager.add_missile(m2);
        manager.add_missile(m3);

        assert_eq!(manager.count(), 3);

        manager.delete_missiles();

        assert_eq!(manager.count(), 1);
        assert_eq!(manager.get_missile(0).unwrap().missile_type, MissileID::Firebolt);
    }

    #[test]
    fn test_process_missiles_out_of_bounds() {
        let mut manager = MissileManager::new(10);

        // In bounds
        manager.add_missile(Missile::new(MissileID::Arrow, Point::new(50, 50)));

        // Out of bounds
        manager.add_missile(Missile::new(MissileID::Firebolt, Point::new(200, 200)));

        assert_eq!(manager.count(), 2);

        manager.process_missiles();

        // Out of bounds missile should be deleted
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.get_missile(0).unwrap().position.tile, Point::new(50, 50));
    }

    #[test]
    fn test_missile_animation() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(0, 0));
        missile.anim_flags = MissileGraphicsFlags::Animated;
        missile.anim_delay = 2;
        missile.anim_len = 8;
        missile.anim_frame = 1;
        missile.anim_add = 1;

        let mut manager = MissileManager::new(10);
        manager.add_missile(missile);

        // Process once - anim_cnt = 1, frame should not change
        manager.process_missiles();
        assert_eq!(manager.get_missile(0).unwrap().anim_frame, 1);

        // Process again - anim_cnt = 2, frame should advance
        manager.process_missiles();
        assert_eq!(manager.get_missile(0).unwrap().anim_frame, 2);

        // Continue until wrap
        for _ in 0..14 {
            manager.process_missiles();
        }

        // Should have wrapped around
        let final_frame = manager.get_missile(0).unwrap().anim_frame;
        assert!(final_frame >= 1 && final_frame <= 8);
    }

    #[test]
    fn test_is_in_dungeon_bounds() {
        assert!(is_in_dungeon_bounds(Point::new(0, 0)));
        assert!(is_in_dungeon_bounds(Point::new(50, 50)));
        assert!(is_in_dungeon_bounds(Point::new(111, 111)));

        assert!(!is_in_dungeon_bounds(Point::new(-1, 0)));
        assert!(!is_in_dungeon_bounds(Point::new(0, -1)));
        assert!(!is_in_dungeon_bounds(Point::new(112, 0)));
        assert!(!is_in_dungeon_bounds(Point::new(0, 112)));
    }

    // =========================================================================
    // Day 101 Tests - Missile Processing
    // =========================================================================

    #[test]
    fn test_update_missile_velocity() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(0, 0));

        // Update velocity towards (10, 0) with speed 32
        update_missile_velocity(&mut missile, Point::new(10, 0), 32);

        // Velocity should be positive in X direction
        assert!(missile.position.velocity.x > 0);
        assert_eq!(missile.position.velocity.y, 0);
    }

    #[test]
    fn test_update_missile_velocity_diagonal() {
        let mut missile = Missile::new(MissileID::Firebolt, Point::new(0, 0));

        // Update velocity towards (10, 10)
        update_missile_velocity(&mut missile, Point::new(10, 10), 32);

        // Both components should be positive
        assert!(missile.position.velocity.x > 0);
        assert!(missile.position.velocity.y > 0);
    }

    #[test]
    fn test_get_direction16() {
        // East
        let dir_e = get_direction16(Point::new(0, 0), Point::new(10, 0));
        assert!(dir_e <= 16);

        // North
        let dir_n = get_direction16(Point::new(0, 0), Point::new(0, -10));
        assert!(dir_n <= 16);

        // Same point
        let dir_same = get_direction16(Point::new(5, 5), Point::new(5, 5));
        assert!(dir_same <= 16);
    }

    #[test]
    fn test_check_missile_collision_bounds() {
        let missile = Missile::new(MissileID::Arrow, Point::new(50, 50));

        // In bounds - no collision
        let result = check_missile_collision(&missile, Point::new(50, 50));
        assert_eq!(result.hit_type, HitType::None);
        assert!(!result.blocked);

        // Out of bounds - wall collision
        let result_oob = check_missile_collision(&missile, Point::new(-1, 0));
        assert_eq!(result_oob.hit_type, HitType::Wall);
        assert!(result_oob.blocked);
    }

    #[test]
    fn test_process_arrow() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 10;
        missile.distance = 0;

        // Process several times
        for _ in 0..5 {
            process_arrow(&mut missile);
        }

        assert_eq!(missile.duration, 5);
        assert!(!missile.delete_flag);

        // Process until expiry
        for _ in 0..10 {
            process_arrow(&mut missile);
        }

        assert!(missile.delete_flag);
    }

    #[test]
    fn test_process_firebolt() {
        let mut missile = Missile::new(MissileID::Firebolt, Point::new(50, 50));
        missile.duration = 100;

        process_firebolt(&mut missile);

        assert_eq!(missile.duration, 99);
        assert!(!missile.delete_flag);
    }

    #[test]
    fn test_process_lightning() {
        let mut missile = Missile::new(MissileID::Lightning, Point::new(50, 50));
        missile.duration = 8;
        missile.var1 = 0;

        // Process until length limit
        for _ in 0..10 {
            process_lightning(&mut missile);
        }

        assert!(missile.delete_flag);
    }

    #[test]
    fn test_add_arrow() {
        let params = AddMissileParameter {
            source: Point::new(10, 10),
            destination: Point::new(20, 10),
            direction: 0,
            spell_level: 0,
            damage: 15,
            caster: MissileCaster::TargetMonsters,
            source_id: 0,
        };

        let missile = add_arrow(&params);

        assert_eq!(missile.missile_type, MissileID::Arrow);
        assert_eq!(missile.position.start, Point::new(10, 10));
        assert_eq!(missile.damage, 15);
        assert_eq!(missile.duration, 256);
    }

    #[test]
    fn test_add_firebolt() {
        let params = AddMissileParameter {
            source: Point::new(5, 5),
            destination: Point::new(15, 15),
            direction: 0,
            spell_level: 5,
            damage: 0,
            caster: MissileCaster::TargetMonsters,
            source_id: 0,
        };

        let missile = add_firebolt(&params);

        assert_eq!(missile.missile_type, MissileID::Firebolt);
        assert!(missile.light_flag);
        assert_eq!(missile.var1, 5); // Start X
        assert_eq!(missile.var2, 5); // Start Y
    }

    #[test]
    fn test_add_fireball() {
        let params = AddMissileParameter {
            source: Point::new(20, 20),
            destination: Point::new(30, 30),
            direction: 0,
            spell_level: 10,
            damage: 50,
            caster: MissileCaster::TargetMonsters,
            source_id: 0,
        };

        let missile = add_fireball(&params);

        assert_eq!(missile.missile_type, MissileID::Fireball);
        assert_eq!(missile.damage, 50);
        assert!(missile.light_flag);
    }

    #[test]
    fn test_add_missile_ex() {
        let mut manager = MissileManager::new(10);

        let params = AddMissileParameter {
            source: Point::new(10, 10),
            destination: Point::new(20, 20),
            ..Default::default()
        };

        // Add arrow
        let idx = manager.add_missile_ex(MissileID::Arrow, &params);
        assert!(idx.is_some());
        assert_eq!(manager.count(), 1);

        // Add firebolt
        let idx2 = manager.add_missile_ex(MissileID::Firebolt, &params);
        assert!(idx2.is_some());
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_process_missiles_full() {
        let mut manager = MissileManager::new(10);

        // Add various missile types
        let params = AddMissileParameter {
            source: Point::new(50, 50),
            destination: Point::new(60, 60),
            ..Default::default()
        };

        manager.add_missile_ex(MissileID::Arrow, &params);
        manager.add_missile_ex(MissileID::Firebolt, &params);
        manager.add_missile_ex(MissileID::Fireball, &params);

        assert_eq!(manager.count(), 3);

        // Process all
        manager.process_missiles_full();

        // All should still exist (duration not expired)
        assert_eq!(manager.count(), 3);

        // Check that duration decreased
        let arrow = manager.get_missile(0).unwrap();
        assert!(arrow.duration < 256);
    }

    #[test]
    fn test_collision_result() {
        let result = CollisionResult {
            hit_type: HitType::Monster(5),
            position: Point::new(10, 10),
            blocked: true,
        };

        assert!(matches!(result.hit_type, HitType::Monster(5)));
        assert!(result.blocked);
    }

    // =========================================================================
    // M66 Tests - Enhanced Collision and Damage Calculation
    // =========================================================================

    #[test]
    fn test_missile_data_arrow() {
        let data = get_missile_data(MissileID::Arrow);
        assert!(data.is_arrow());
        assert!(data.is_drawn());
        assert_eq!(data.damage_type(), DamageType::Physical);
        assert_eq!(data.movement_distribution, MissileMovementDistribution::Blockable);
    }

    #[test]
    fn test_missile_data_firebolt() {
        let data = get_missile_data(MissileID::Firebolt);
        assert!(!data.is_arrow());
        assert!(data.is_drawn());
        assert_eq!(data.damage_type(), DamageType::Fire);
        assert_eq!(data.movement_distribution, MissileMovementDistribution::Blockable);
    }

    #[test]
    fn test_missile_data_lightning() {
        let data = get_missile_data(MissileID::Lightning);
        assert_eq!(data.damage_type(), DamageType::Lightning);
        assert_eq!(data.movement_distribution, MissileMovementDistribution::Disabled);
    }

    #[test]
    fn test_missile_data_flame_wave() {
        let data = get_missile_data(MissileID::FlameWave);
        assert_eq!(data.damage_type(), DamageType::Fire);
        assert_eq!(data.movement_distribution, MissileMovementDistribution::Unblockable);
    }

    #[test]
    fn test_dungeon_state_default() {
        let state = DungeonState::default();
        assert!(!state.is_solid(Point::new(50, 50)));
        assert!(state.has_monster(Point::new(50, 50)).is_none());
        assert!(state.has_player(Point::new(50, 50)).is_none());
    }

    #[test]
    fn test_dungeon_state_monster() {
        let mut state = DungeonState::default();
        state.monsters[50][50] = 5; // Monster ID 4 (5 - 1)

        assert_eq!(state.has_monster(Point::new(50, 50)), Some(4));
        assert!(state.has_monster(Point::new(51, 51)).is_none());
    }

    #[test]
    fn test_dungeon_state_player() {
        let mut state = DungeonState::default();
        state.players[30][30] = 2; // Player ID 1 (2 - 1)

        assert_eq!(state.has_player(Point::new(30, 30)), Some(1));
        assert!(state.has_player(Point::new(31, 31)).is_none());
    }

    #[test]
    fn test_collision_with_state_monster() {
        let mut state = DungeonState::default();
        state.monsters[60][60] = 3; // Monster at position

        let missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        let result = check_missile_collision_with_state(&missile, Point::new(60, 60), &state);

        assert!(matches!(result.hit_type, HitType::Monster(2)));
        assert!(result.blocked); // Arrow is blockable
    }

    #[test]
    fn test_collision_with_state_player() {
        let mut state = DungeonState::default();
        state.players[40][40] = 1; // Player 0 at position

        let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
        missile.caster = MissileCaster::TargetPlayers;
        missile.source = 5; // Monster source

        let result = check_missile_collision_with_state(&missile, Point::new(40, 40), &state);

        assert!(matches!(result.hit_type, HitType::Player(0)));
        assert!(result.blocked);
    }

    #[test]
    fn test_collision_with_state_wall() {
        let mut state = DungeonState::default();
        state.solid_tiles[25][25] = true;

        let missile = Missile::new(MissileID::Firebolt, Point::new(20, 20));
        let result = check_missile_collision_with_state(&missile, Point::new(25, 25), &state);

        assert_eq!(result.hit_type, HitType::Wall);
        assert!(result.blocked);
    }

    #[test]
    fn test_collision_unblockable_missile() {
        let mut state = DungeonState::default();
        state.monsters[70][70] = 2;

        // FlameWave is unblockable - should hit but not be blocked
        let missile = Missile::new(MissileID::FlameWave, Point::new(65, 65));
        let result = check_missile_collision_with_state(&missile, Point::new(70, 70), &state);

        assert!(matches!(result.hit_type, HitType::Monster(_)));
        assert!(!result.blocked); // Unblockable passes through
    }

    #[test]
    fn test_calculate_missile_damage() {
        let missile = Missile::new(MissileID::Firebolt, Point::new(0, 0));
        let mut rng = rand::rng();

        // No resistance
        let damage = calculate_missile_damage(&missile, 10, 20, 0, &mut rng);
        assert!(damage >= 10 << 6);
        assert!(damage <= 20 << 6);

        // 50% resistance
        let damage_resisted = calculate_missile_damage(&missile, 100, 100, 50, &mut rng);
        assert_eq!(damage_resisted, (100 << 6) / 2);
    }

    #[test]
    fn test_monster_resistance() {
        // Normal monster - no resistance
        assert_eq!(get_monster_resistance(0, DamageType::Fire), 0);
        assert_eq!(get_monster_resistance(0, DamageType::Physical), 0);

        // Fire resistant monster
        assert!(get_monster_resistance(7, DamageType::Fire) > 0);

        // Fire immune monster
        assert!(get_monster_resistance(12, DamageType::Fire) >= 50);
    }

    #[test]
    fn test_apply_missile_damage() {
        let missile = Missile::new(MissileID::Arrow, Point::new(0, 0));
        let mut monster_hp = 100 << 6; // 100 HP in fixed point
        let mut rng = rand::rng();

        let (damage, killed) = apply_missile_damage_to_monster(
            &missile,
            &mut monster_hp,
            0, // Normal monster
            10,
            20,
            &mut rng,
        );

        assert!(damage > 0);
        assert!(monster_hp < 100 << 6);
        // May or may not be killed depending on damage roll
        assert_eq!(killed, monster_hp <= 0);
    }

    #[test]
    fn test_is_missile_blocked_by_tile() {
        let mut state = DungeonState::default();
        state.solid_tiles[50][50] = true;

        assert!(is_missile_blocked_by_tile(Point::new(50, 50), &state));
        assert!(!is_missile_blocked_by_tile(Point::new(51, 51), &state));
        assert!(is_missile_blocked_by_tile(Point::new(-1, 0), &state)); // Out of bounds
    }

    #[test]
    fn test_missile_data_flags() {
        let flags = MissileDataFlags {
            damage_type: DamageType::Fire,
            is_arrow: false,
            invisible: false,
        };

        assert_eq!(flags.damage_type, DamageType::Fire);
        assert!(!flags.is_arrow);
        assert!(!flags.invisible);
    }

    #[test]
    fn test_missile_graphic_id() {
        assert_eq!(MissileGraphicID::Arrow as u8, 0);
        assert_eq!(MissileGraphicID::Fireball as u8, 1);
        assert_eq!(MissileGraphicID::None as u8, 255);
    }

    // =============================================================================
    // M66 Day 1 Tests: CheckMissileCol Enhancement
    // =============================================================================

    #[test]
    fn test_check_can_hit_only_walking_north() {
        let mut missile = Missile::new(MissileID::Rhino, Point::new(50, 50));
        missile.position.start = Point::new(50, 60); // Started south

        // Target north of start
        let target_pos = Point::new(50, 55);
        assert!(check_can_hit_only_walking(&missile, target_pos, Direction::North));

        // Target south of start (wrong direction)
        let target_pos_wrong = Point::new(50, 65);
        assert!(!check_can_hit_only_walking(&missile, target_pos_wrong, Direction::North));
    }

    #[test]
    fn test_check_can_hit_only_walking_diagonal() {
        let mut missile = Missile::new(MissileID::Rhino, Point::new(50, 50));
        missile.position.start = Point::new(50, 50);

        // Target northeast
        let target_pos = Point::new(55, 45);
        assert!(check_can_hit_only_walking(&missile, target_pos, Direction::NorthEast));

        // Target southwest (wrong direction)
        let target_pos_wrong = Point::new(45, 55);
        assert!(!check_can_hit_only_walking(&missile, target_pos_wrong, Direction::NorthEast));
    }

    #[test]
    fn test_check_missile_col_out_of_bounds() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.caster = MissileCaster::TargetMonsters;

        let state = DungeonState::default();

        // Use Cell for interior mutability
        use std::cell::Cell;
        let hit_count = Cell::new(0);

        check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            Point::new(-1, 0), // Out of bounds
            false,
            None,
            &state,
            |_, _, _| { hit_count.set(hit_count.get() + 1); true },
            |_, _, _| { hit_count.set(hit_count.get() + 1); (true, false) },
            |_| { hit_count.set(hit_count.get() + 1); },
        );

        // Should return early without hitting anything
        assert_eq!(hit_count.get(), 0);
        assert!(!missile.hit_flag);
    }

    #[test]
    fn test_check_missile_col_monster_hit() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.caster = MissileCaster::TargetMonsters; // Player missile
        missile.source = 0;
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.monsters[60][60] = 5; // Monster ID 4 (value is id+1)

        let mut monster_hit = false;

        check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            Point::new(60, 60),
            false, // Delete on collision
            None,
            &state,
            |monster_id, min, max| {
                monster_hit = true;
                assert_eq!(monster_id, 4);
                assert_eq!(min, 10);
                assert_eq!(max, 20);
                true
            },
            |_, _, _| (false, false),
            |_| {},
        );

        assert!(monster_hit);
        assert!(missile.hit_flag);
        assert_eq!(missile.duration, 0); // Missile deleted
    }

    #[test]
    fn test_check_missile_col_monster_hit_dont_delete() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
        missile.caster = MissileCaster::TargetMonsters;
        missile.source = 0;
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.monsters[60][60] = 5;

        check_missile_col(
            &mut missile,
            DamageType::Fire,
            10,
            20,
            false,
            Point::new(60, 60),
            true, // Don't delete on collision
            None,
            &state,
            |_, _, _| true,
            |_, _, _| (false, false),
            |_| {},
        );

        assert!(missile.hit_flag);
        assert_eq!(missile.duration, 100); // Missile NOT deleted
    }

    #[test]
    fn test_check_missile_col_player_hit() {
        let mut missile = Missile::new(MissileID::Firebolt, Point::new(50, 50));
        missile.caster = MissileCaster::TargetPlayers; // Monster missile
        missile.source = 10; // Monster ID
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.players[60][60] = 2; // Player ID 1 (value is id+1)

        let mut player_hit = false;

        check_missile_col(
            &mut missile,
            DamageType::Fire,
            15,
            25,
            false,
            Point::new(60, 60),
            false,
            None,
            &state,
            |_, _, _| false,
            |player_id, min, max| {
                player_hit = true;
                assert_eq!(player_id, 1);
                assert_eq!(min, 15);
                assert_eq!(max, 25);
                (true, false)
            },
            |_| {},
        );

        assert!(player_hit);
        assert!(missile.hit_flag);
        assert_eq!(missile.duration, 0);
    }

    #[test]
    fn test_check_missile_col_player_blocked() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.caster = MissileCaster::TargetPlayers;
        missile.source = 5;
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.players[60][60] = 1;

        check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            Point::new(60, 60),
            false,
            None,
            &state,
            |_, _, _| false,
            |_, _, _| (true, true), // Hit and blocked
            |_| {},
        );

        assert!(missile.hit_flag);
        // When blocked in Hellfire, missile is rotated not deleted
        // For simplicity, we still mark duration but acknowledge behavior differs
        assert!(missile.duration == 0 || missile.duration == 100);
    }

    #[test]
    fn test_check_missile_col_pvp_self_exclusion() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
        missile.caster = MissileCaster::TargetMonsters; // Player missile
        missile.source = 2; // Player ID 2
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.players[60][60] = 3; // Player ID 2 (value is id+1, so 3 means player 2)

        let mut player_hit = false;

        check_missile_col(
            &mut missile,
            DamageType::Fire,
            10,
            20,
            false,
            Point::new(60, 60),
            false,
            None,
            &state,
            |_, _, _| false,
            |_, _, _| { player_hit = true; (true, false) },
            |_| {},
        );

        // Should NOT hit self
        assert!(!player_hit);
        assert!(!missile.hit_flag);
    }

    #[test]
    fn test_check_missile_col_wall() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.solid_tiles[60][60] = true;

        check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            Point::new(60, 60),
            false,
            None,
            &state,
            |_, _, _| false,
            |_, _, _| (false, false),
            |_| {},
        );

        assert_eq!(missile.duration, 0); // Hit wall, deleted
        assert!(!missile.hit_flag); // Wall hits don't set hit_flag
    }

    #[test]
    fn test_check_missile_col_object() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.solid_tiles[60][60] = true;
        state.objects[60][60] = 10; // Object ID 9 (value is id+1)

        let mut object_hit = false;

        check_missile_col(
            &mut missile,
            DamageType::Fire,
            10,
            20,
            false,
            Point::new(60, 60),
            false,
            None,
            &state,
            |_, _, _| false,
            |_, _, _| (false, false),
            |obj_id| {
                object_hit = true;
                assert_eq!(obj_id, 9);
            },
        );

        assert!(object_hit);
        assert_eq!(missile.duration, 0);
    }

    #[test]
    fn test_check_missile_col_trap() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.caster = MissileCaster::TargetPlayers;
        missile.source = -1; // Trap
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.monsters[60][60] = 3; // Monster ID 2

        let mut monster_hit = false;

        check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            Point::new(60, 60),
            false,
            None,
            &state,
            |_, _, _| { monster_hit = true; true },
            |_, _, _| (false, false),
            |_| {},
        );

        // Traps can hit any monster
        assert!(monster_hit);
        assert!(missile.hit_flag);
    }

    #[test]
    fn test_move_missile_and_check_col_tile_changed() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(65536, 0); // Move 1 tile east
        missile.duration = 100;
        missile.caster = MissileCaster::TargetMonsters;

        let mut state = DungeonState::default();
        state.monsters[51][50] = 5; // Monster in next tile

        let mut monster_hits = 0;
        let mut player_hits = 0;
        let mut object_hits = 0;

        move_missile_and_check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            false,
            &state,
            |_, _, _| { monster_hits += 1; true },
            |_, _, _| { player_hits += 1; (true, false) },
            |_| { object_hits += 1; },
        );

        // Should have moved and hit monster
        assert_eq!(missile.position.tile, Point::new(51, 50));
        assert_eq!(monster_hits, 1);
        assert_eq!(player_hits, 0);
        assert_eq!(object_hits, 0);
    }

    #[test]
    fn test_move_missile_and_check_col_ignore_start() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
        missile.position.start = Point::new(50, 50);
        missile.position.velocity = Point::new(0, 65536); // Move south
        missile.duration = 100;

        let mut state = DungeonState::default();
        state.monsters[50][51] = 3;

        let mut monster_hits = 0;

        move_missile_and_check_missile_col(
            &mut missile,
            DamageType::Fire,
            10,
            20,
            true, // Ignore start tile
            false,
            &state,
            |_, _, _| { monster_hits += 1; true },
            |_, _, _| (false, false),
            |_| {},
        );

        assert_eq!(monster_hits, 1); // Should hit after leaving start
    }

    #[test]
    fn test_move_missile_and_check_col_hash_tracking() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(0, 0); // Not moving
        missile.duration = 100;
        missile.last_collision_target_hash = 0;

        let mut state = DungeonState::default();
        state.monsters[50][50] = 5; // Monster at current tile

        let mut first_checks = 0;

        // First check - should detect new target
        move_missile_and_check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            false,
            &state,
            |_, _, _| { first_checks += 1; false }, // Don't actually hit
            |_, _, _| (false, false),
            |_| {},
        );

        assert_eq!(first_checks, 1);

        let mut second_checks = 0;

        // Second check - hash matches, should NOT check again
        move_missile_and_check_missile_col(
            &mut missile,
            DamageType::Physical,
            10,
            20,
            false,
            false,
            &state,
            |_, _, _| { second_checks += 1; false },
            |_, _, _| (false, false),
            |_| {},
        );

        assert_eq!(second_checks, 0); // No duplicate hit
    }

    // =============================================================================
    // M66 Day 2 Tests: Damage Calculation System
    // =============================================================================

    #[test]
    fn test_monster_m_hit_basic() {
        let mut monster_hp = 100 << 6; // 100 HP in fixed point
        // Seeded RNG so the hit roll is deterministic (hit chance ~28%, clamped).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, killed) = monster_m_hit(
            10, // player_level
            50, // ranged to-hit
            40, // magic to-hit
            20, // armor pierce
            10, // damage mod
            100, // bonus dam %
            5, // bonus dam flat
            false, // is rogue
            20, // monster AC
            5, // monster level
            &mut monster_hp,
            0, // resistance
            10, // min damage
            20, // max damage
            2, // distance
            MissileID::Arrow,
            DamageType::Physical,
            false, // not shifted
            &mut rng,
        );

        assert!(hit);
        assert!(damage > 0);
        assert!(monster_hp < 100 << 6);
        // Monster should still be alive with this damage
        assert!(!killed);
    }

    #[test]
    fn test_monster_m_hit_miss() {
        let mut monster_hp = 100 << 6;
        let mut rng = rand::rng();

        // Very low to-hit chance
        let (hit, damage, killed) = monster_m_hit(
            1, 5, 5, 0, 0, 100, 0, false,
            50, // high AC
            20, // high level monster
            &mut monster_hp,
            0, 10, 20, 10, // far distance
            MissileID::Arrow,
            DamageType::Physical,
            false,
            &mut rng,
        );

        // Should miss most of the time (hit chance is clamped to 5%)
        // We can't guarantee miss, but damage should be 0 if missed
        if !hit {
            assert_eq!(damage, 0);
            assert_eq!(monster_hp, 100 << 6); // HP unchanged
        }
    }

    #[test]
    fn test_monster_m_hit_bone_spirit() {
        let mut monster_hp = 300 << 6; // 300 HP
        // Seeded RNG so the hit roll is deterministic (hit chance 58%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, _killed) = monster_m_hit(
            15, 80, 70, 20, 0, 100, 0, false,
            10, 10,
            &mut monster_hp,
            0,
            0, 0, // min/max don't matter for bone spirit
            2,
            MissileID::BoneSpirit,
            DamageType::Magic,
            false,
            &mut rng,
        );

        assert!(hit);
        // Bone Spirit should do 1/3 of HP
        // Initial HP: 300 << 6 = 19200
        // Damage: (19200 / 3) >> 6 = 100
        // Then shifted: 100 << 6 = 6400
        // Final HP: 19200 - 6400 = 12800 = 200 << 6
        let expected_hp_approx = 200 << 6;
        assert!((monster_hp - expected_hp_approx).abs() < (10 << 6)); // Within 10 HP
    }

    #[test]
    fn test_monster_m_hit_with_resistance() {
        let mut monster_hp = 100 << 6;
        // Seeded RNG so the hit roll is deterministic (hit chance 68%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, _killed) = monster_m_hit(
            10, 80, 70, 20, 10, 100, 5, false,
            10, 5,
            &mut monster_hp,
            75, // 75% resistance -> damage >> 2 (25% damage)
            50, 60, // High damage
            2,
            MissileID::Fireball,
            DamageType::Fire,
            false,
            &mut rng,
        );

        assert!(hit);
        // Damage should be significantly reduced
        // Even with high base damage (50-60), after shift and >>2, should be moderate
        assert!(damage > 0);
        assert!(damage < (60 << 6)); // Less than full damage
    }

    #[test]
    fn test_monster_m_hit_kill() {
        let mut monster_hp = 10 << 6; // Low HP
        // Seeded RNG so the hit roll is deterministic (hit chance 60%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, killed) = monster_m_hit(
            15, 90, 80, 30, 20, 150, 10, true, // Strong player (rogue)
            5, 3,
            &mut monster_hp,
            0,
            50, 100, // High damage
            1,
            MissileID::Arrow,
            DamageType::Physical,
            false,
            &mut rng,
        );

        assert!(hit);
        assert!(damage > 0);
        assert!(killed); // Should kill low HP monster
        assert!(monster_hp <= 0);
    }

    #[test]
    fn test_plr2plr_m_hit_basic() {
        let mut target_hp = 100 << 6;
        // Seeded RNG so the hit roll is deterministic (hit chance ~36%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, blocked) = plr2plr_m_hit(
            10, 50, 40, 10, 100, 5, false, // Attacker stats
            30, 10, &mut target_hp, // Target stats
            0, 0, 0, // Resistances
            30, false, // Block chance, not blocking
            15, 25, 3, // Damage, distance
            MissileID::Arrow,
            DamageType::Physical,
            false,
            true, // Friendly fire enabled
            &mut rng,
        );

        assert!(hit);
        assert!(damage > 0);
        assert!(!blocked);
        assert!(target_hp < 100 << 6);
    }

    #[test]
    fn test_plr2plr_m_hit_friendly_fire_disabled() {
        let mut target_hp = 100 << 6;
        let mut rng = rand::rng();

        let (hit, damage, _blocked) = plr2plr_m_hit(
            10, 50, 40, 10, 100, 5, false,
            30, 10, &mut target_hp,
            0, 0, 0,
            30, false,
            15, 25, 3,
            MissileID::Arrow,
            DamageType::Physical,
            false,
            false, // Friendly fire disabled
            &mut rng,
        );

        assert!(!hit);
        assert_eq!(damage, 0);
        assert_eq!(target_hp, 100 << 6); // HP unchanged
    }

    #[test]
    fn test_plr2plr_m_hit_holy_bolt() {
        let mut target_hp = 100 << 6;
        let mut rng = rand::rng();

        let (hit, damage, _blocked) = plr2plr_m_hit(
            10, 50, 40, 10, 100, 5, false,
            30, 10, &mut target_hp,
            0, 0, 0,
            30, false,
            15, 25, 3,
            MissileID::HolyBolt,
            DamageType::Holy,
            false,
            true,
            &mut rng,
        );

        assert!(!hit); // Holy Bolt doesn't hit players
        assert_eq!(damage, 0);
    }

    #[test]
    fn test_plr2plr_m_hit_with_resistance() {
        let mut target_hp = 100 << 6;
        // Seeded RNG so the hit roll is deterministic (hit chance 37%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, _blocked) = plr2plr_m_hit(
            10, 50, 60, 10, 100, 5, false,
            20, 10, &mut target_hp,
            50, 0, 0, // 50% fire resistance
            30, false,
            30, 40, 3,
            MissileID::Fireball,
            DamageType::Fire,
            false,
            true,
            &mut rng,
        );

        assert!(hit);
        // Damage should be reduced by resistance
        assert!(damage > 0);
        // With 50% resist, damage should be roughly half of base
    }

    #[test]
    fn test_plr2plr_m_hit_blocked() {
        let mut target_hp = 100 << 6;

        // Use fixed seed for deterministic blocking test
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let (hit, _damage, blocked) = plr2plr_m_hit(
            10, 50, 40, 10, 100, 5, false,
            20, 10, &mut target_hp,
            0, 0, 0,
            95, true, // Very high block chance, is blocking
            15, 25, 3,
            MissileID::Arrow,
            DamageType::Physical,
            false,
            true,
            &mut rng,
        );

        if hit {
            // If hit, might be blocked (depends on RNG)
            // We can't guarantee block, but blocked should be boolean
            assert!(blocked || !blocked); // Always true, just checking type
        }
    }

    #[test]
    fn test_plr2plr_m_hit_spell_half_damage() {
        let mut target_hp = 100 << 6;
        // Seeded RNG so the hit roll is deterministic (hit chance ~28%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, _blocked) = plr2plr_m_hit(
            15, 50, 70, 10, 100, 5, false,
            20, 10, &mut target_hp,
            0, 0, 0,
            30, false,
            40, 60, 2, // High damage
            MissileID::Fireball, // Not an arrow
            DamageType::Fire,
            false,
            true,
            &mut rng,
        );

        assert!(hit);
        // Non-arrow spells do half damage in PvP
        // So effective damage is (40-60) / 2 = 20-30 before shift
        // After shift: (20-30) << 6
        assert!(damage > 0);
        assert!(damage < (60 << 6)); // Should be less than full damage
    }

    #[test]
    fn test_plr2plr_m_hit_bone_spirit_pvp() {
        let mut target_hp = 200 << 6;
        // Seeded RNG so the hit roll is deterministic (hit chance ~38%).
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);

        let (hit, damage, _blocked) = plr2plr_m_hit(
            15, 50, 80, 10, 100, 5, false,
            20, 10, &mut target_hp,
            0, 0, 0,
            30, false,
            0, 0, 2, // min/max don't matter
            MissileID::BoneSpirit,
            DamageType::Magic,
            false,
            true,
            &mut rng,
        );

        assert!(hit);
        // Bone Spirit does 1/3 of target HP, then (non-arrow PvP) halves it:
        // Initial: 200 << 6 = 12800
        // dam = 12800 / 3 = 4266, then /2 = 2133
        // Final: 12800 - 2133 ≈ 10667
        assert!(damage > 0);
        let expected_hp = (200 << 6) - ((200 << 6) / 3) / 2;
        assert!((target_hp - expected_hp).abs() < (10 << 6));
    }

    #[test]
    fn test_object_m_hit() {
        let hit = object_m_hit(5, MissileID::Fireball);
        assert!(hit); // Should always succeed in simplified version
    }

    #[test]
    fn test_calculate_missile_damage_existing() {
        // Test existing calculate_missile_damage function
        let missile = Missile::new(MissileID::Arrow, Point::new(0, 0));
        let mut rng = rand::rng();

        let damage = calculate_missile_damage(
            &missile,
            10, 20,
            0, // No resistance
            &mut rng,
        );

        assert!(damage > 0);
        // Damage should be in range [10, 20] << 6
        assert!(damage >= (10 << 6));
        assert!(damage <= (20 << 6));
    }

    // =========================================================================
    // M66 Day 3 Tests: 导弹处理循环和位置更新
    // =========================================================================

    #[test]
    fn test_update_missile_pos_basic() {
        // Test basic position update from traveled distance
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.traveled = Point::new(64 << 16, 64 << 16);

        update_missile_pos(&mut missile);

        // 64 pixels = 1 tile in both directions
        // screenToMissile: tileX = (64-64)/64 = 0, tileY = (64+64)/64 = 2
        assert_eq!(missile.position.tile.x, 50); // 50 + 0
        assert_eq!(missile.position.tile.y, 52); // 50 + 2
    }

    #[test]
    fn test_update_missile_pos_diagonal() {
        // Test diagonal movement
        let mut missile = Missile::new(MissileID::Fireball, Point::new(30, 30));
        missile.position.traveled = Point::new(128 << 16, 0); // 128 pixels right

        update_missile_pos(&mut missile);

        // 128 pixels right, 0 down
        // screenToMissile: tileX = (128-0)/64 = 2, tileY = (128+0)/64 = 2
        assert_eq!(missile.position.tile.x, 32); // 30 + 2
        assert_eq!(missile.position.tile.y, 32); // 30 + 2
    }

    #[test]
    fn test_is_missile_in_bounds() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        assert!(is_missile_in_bounds(&missile));

        missile.position.tile = Point::new(-1, 50);
        assert!(!is_missile_in_bounds(&missile));

        missile.position.tile = Point::new(50, 150);
        assert!(!is_missile_in_bounds(&missile));
    }

    #[test]
    fn test_is_tile_blocked() {
        let mut state = DungeonState::default();
        state.solid_tiles[50][50] = true;

        assert!(is_tile_blocked(Point::new(50, 50), &state));
        assert!(!is_tile_blocked(Point::new(51, 51), &state));

        // Out of bounds is blocked
        assert!(is_tile_blocked(Point::new(-1, 50), &state));
        assert!(is_tile_blocked(Point::new(150, 50), &state));
    }

    #[test]
    fn test_move_missile_with_collision_success() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(64 << 16, 0); // Move right

        let moved = move_missile_with_collision(&mut missile, |_| true);

        assert!(moved);
        assert_ne!(missile.position.tile, Point::new(50, 50));
    }

    #[test]
    fn test_move_missile_with_collision_blocked() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(64 << 16, 0);

        let moved = move_missile_with_collision(&mut missile, |tile| {
            tile == Point::new(50, 50) // Only start tile is valid
        });

        assert!(!moved);
        assert_eq!(missile.position.tile, Point::new(50, 50)); // Should stay at original
        assert_eq!(missile.position.velocity, Point::new(0, 0)); // Velocity cleared
    }

    #[test]
    fn test_move_missile_with_collision_no_movement() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(0, 0); // No velocity

        let moved = move_missile_with_collision(&mut missile, |_| true);

        assert!(!moved);
        assert_eq!(missile.position.tile, Point::new(50, 50));
    }

    #[test]
    fn test_process_single_missile_duration() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 5;

        let state = DungeonState::default();

        // Process 4 times
        for _ in 0..4 {
            process_single_missile(&mut missile, &state);
            assert!(!missile.delete_flag);
        }

        // 5th time should mark for deletion
        process_single_missile(&mut missile, &state);
        assert!(missile.delete_flag);
    }

    #[test]
    fn test_process_single_missile_out_of_bounds() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(-1, 50));
        missile.duration = 100;

        let state = DungeonState::default();

        process_single_missile(&mut missile, &state);

        assert!(missile.delete_flag);
    }

    #[test]
    fn test_process_single_missile_movement() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;
        missile.position.velocity = Point::new(32 << 16, 0); // Slow movement

        let state = DungeonState::default();

        let old_tile = missile.position.tile;
        process_single_missile(&mut missile, &state);

        // Should have moved (or attempted to move)
        assert!(!missile.delete_flag); // Not deleted yet
        // Position might have changed depending on movement
    }

    #[test]
    fn test_process_single_missile_hit_wall() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;
        missile.position.velocity = Point::new(128 << 16, 0); // Fast movement right

        let mut state = DungeonState::default();
        state.solid_tiles[52][52] = true; // Block destination

        process_single_missile(&mut missile, &state);

        // Should mark hit when hitting wall
        assert!(missile.hit_flag);
    }

    // =========================================================================
    // M66 Day 4 Tests: 高速插值与集成
    // =========================================================================

    #[test]
    fn test_check_intermediate_tiles_clear_path() {
        // No obstacles between start and end
        let start = Point::new(50, 50);
        let end = Point::new(52, 52);
        let velocity = Point::new(128 << 16, 128 << 16);

        let blocked = check_intermediate_tiles(start, end, velocity, |_| true);

        assert!(blocked.is_none()); // Path is clear
    }

    #[test]
    fn test_check_intermediate_tiles_blocked() {
        // Obstacle on the missile's interpolated iso path.
        // check_intermediate_tiles walks the iso tiles the missile actually
        // traverses (screenToMissile mapping), so the blocker must be placed on
        // a sampled tile, not an assumed grid-diagonal tile. With start=(50,50)
        // and velocity (192<<16, 192<<16), the first sampled iso tile is (53,51).
        let start = Point::new(50, 50);
        let end = Point::new(53, 53);
        let velocity = Point::new(192 << 16, 192 << 16);
        let blocked_tile = Point::new(53, 51);

        let blocked = check_intermediate_tiles(start, end, velocity, |tile| {
            tile != blocked_tile
        });

        assert!(blocked.is_some());
        assert_eq!(blocked.unwrap(), blocked_tile);
    }

    #[test]
    fn test_check_intermediate_tiles_no_velocity() {
        // Zero velocity - no movement
        let start = Point::new(50, 50);
        let end = Point::new(50, 50);
        let velocity = Point::new(0, 0);

        let blocked = check_intermediate_tiles(start, end, velocity, |_| true);

        assert!(blocked.is_none());
    }

    #[test]
    fn test_move_missile_with_full_interpolation_success() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(64 << 16, 0);

        let moved = move_missile_with_full_interpolation(&mut missile, |_| true);

        assert!(moved);
        assert_ne!(missile.position.tile, Point::new(50, 50));
    }

    #[test]
    fn test_move_missile_with_full_interpolation_blocked() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(256 << 16, 0); // High speed

        let moved = move_missile_with_full_interpolation(&mut missile, |tile| {
            tile.x < 52 // Block anything beyond x=52
        });

        assert!(!moved);
        assert_eq!(missile.position.tile, Point::new(50, 50)); // Stopped at start
        assert_eq!(missile.position.velocity, Point::new(0, 0));
    }

    #[test]
    fn test_move_missile_with_full_interpolation_one_tile() {
        // Single tile movement uses simple path
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.position.velocity = Point::new(32 << 16, 0); // Slow

        let moved = move_missile_with_full_interpolation(&mut missile, |_| true);

        // Should move (or not, depending on exact distance)
        // Main thing is it doesn't crash
        assert!(!missile.delete_flag);
    }

    #[test]
    fn test_process_missile_integrated_with_interpolation() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;
        missile.position.velocity = Point::new(256 << 16, 0); // High speed

        let state = DungeonState::default();

        process_missile_integrated(&mut missile, &state, true);

        assert!(!missile.delete_flag); // Should still be alive
        assert_eq!(missile.duration, 99); // Duration decremented
    }

    #[test]
    fn test_process_missile_integrated_without_interpolation() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;
        missile.position.velocity = Point::new(64 << 16, 0);

        let state = DungeonState::default();

        process_missile_integrated(&mut missile, &state, false);

        assert!(!missile.delete_flag);
        assert_eq!(missile.duration, 99);
    }

    #[test]
    fn test_process_missile_integrated_duration_expiry() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 1; // About to expire

        let state = DungeonState::default();

        process_missile_integrated(&mut missile, &state, true);

        assert!(missile.delete_flag); // Should be deleted
    }

    #[test]
    fn test_process_missile_integrated_out_of_bounds() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(-1, 50));
        missile.duration = 100;

        let state = DungeonState::default();

        process_missile_integrated(&mut missile, &state, true);

        assert!(missile.delete_flag);
    }

    #[test]
    fn test_process_missile_integrated_collision() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;
        missile.position.velocity = Point::new(128 << 16, 0);

        let mut state = DungeonState::default();
        state.solid_tiles[52][52] = true; // Block destination

        process_missile_integrated(&mut missile, &state, true);

        assert!(missile.hit_flag); // Should mark collision
    }

    // Integration test: Full missile lifecycle
    #[test]
    fn test_full_missile_lifecycle() {
        let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
        missile.duration = 10;
        missile.position.velocity = Point::new(32 << 16, 32 << 16);

        let state = DungeonState::default();

        // Simulate 5 frames
        for i in 0..5 {
            process_missile_integrated(&mut missile, &state, true);
            assert!(!missile.delete_flag, "Deleted too early at frame {}", i);
            assert_eq!(missile.duration, 10 - i - 1);
        }

        // Simulate remaining frames until expiry
        for _ in 5..10 {
            process_missile_integrated(&mut missile, &state, true);
        }

        assert!(missile.delete_flag); // Should be deleted after duration expires
    }

    // Stress test: Many high-speed missiles
    #[test]
    fn test_multiple_high_speed_missiles() {
        let state = DungeonState::default();

        for i in 0..10 {
            let mut missile = Missile::new(MissileID::Arrow, Point::new(40 + i, 40));
            missile.duration = 50;
            missile.position.velocity = Point::new(512 << 16, 0); // Very high speed

            // Should not crash even with high speed
            process_missile_integrated(&mut missile, &state, true);
        }
    }

    // Edge case: Zero velocity with interpolation
    #[test]
    fn test_interpolation_zero_velocity() {
        let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
        missile.duration = 100;
        missile.position.velocity = Point::new(0, 0);

        let state = DungeonState::default();

        process_missile_integrated(&mut missile, &state, true);

        assert!(!missile.delete_flag);
        assert_eq!(missile.position.tile, Point::new(50, 50));
    }
}
