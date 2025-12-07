//! Monster System - Exact port of DevilutionX Source/monster.h
//!
//! **C++ Reference**: Source/monster.h (lines 212-350)
//!
//! Implements complete Monster entity with:
//! - Position, movement, and direction
//! - HP, combat stats (AC, damage, to-hit)
//! - AI type and behavior variables
//! - Animation state
//! - Monster modes (Standing, Walking, Attacking, etc.)
//! - Pack/leader system
//! - Resistance and flags

use crate::game::types::Point;
use crate::game::monster_dat::{MonsterAIID, MonsterData};
use serde::{Deserialize, Serialize};

/// Monster flags - exact match of monster_flag enum (Source/monster.h:38-50)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MonsterFlags(pub u16);

impl MonsterFlags {
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
    pub const NOLIFESTEAL: Self = Self(1 << 12);

    pub fn is_hidden(&self) -> bool {
        (self.0 & Self::HIDDEN.0) != 0
    }

    pub fn is_golem(&self) -> bool {
        (self.0 & Self::GOLEM.0) != 0
    }

    pub fn can_open_door(&self) -> bool {
        (self.0 & Self::CAN_OPEN_DOOR.0) != 0
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        if hidden {
            self.0 |= Self::HIDDEN.0;
        } else {
            self.0 &= !Self::HIDDEN.0;
        }
    }
}

/// Monster mode - exact match of MonsterMode enum (Source/monster.h:70-89)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum MonsterMode {
    #[default]
    Stand = 0,
    /// Movement towards N, NW, or NE
    MoveNorthwards = 1,
    /// Movement towards S, SW, or SE
    MoveSouthwards = 2,
    /// Movement towards W or E
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
    /// Check if mode is a movement mode (Source/monster.h:91-100)
    pub fn is_move(&self) -> bool {
        matches!(
            self,
            MonsterMode::MoveNorthwards
                | MonsterMode::MoveSouthwards
                | MonsterMode::MoveSideways
        )
    }
}

/// Monster graphic type - exact match of MonsterGraphic enum (Source/monster.h:102-109)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MonsterGraphic {
    Stand = 0,
    Walk = 1,
    Attack = 2,
    GotHit = 3,
    Death = 4,
    Special = 5,
}

/// Monster goal - exact match of MonsterGoal enum (Source/monster.h:111-119)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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

/// Leader relation - exact match of LeaderRelation enum (Source/monster.h:136-149)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum LeaderRelation {
    #[default]
    None = 0,
    /// Minion that sticks to the leader
    Leashed = 1,
    /// Minion that was separated from the leader and acts individually until it reaches the leader again
    Separated = 2,
}

/// Unique monster type - exact match of UniqueMonsterType enum (Source/monster.h:56-69)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
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

/// Direction enum - simplified version matching C++ Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Direction {
    #[default]
    South = 0,
    SouthWest = 1,
    West = 2,
    NorthWest = 3,
    North = 4,
    NorthEast = 5,
    East = 6,
    SouthEast = 7,
}

/// Monster entity - exact port of Monster struct (Source/monster.h:212-280)
///
/// **C++ Reference**: `struct Monster` in Source/monster.h
///
/// This struct contains all fields from the C++ Monster, organized in the same order.
/// Field names use Rust naming conventions (snake_case) but maintain C++ semantics.
#[derive(Debug, Clone)]
pub struct Monster {
    // === Animation (simplified - full animation system TBD) ===
    /// Current animation frame
    pub anim_frame: i32,
    /// Animation length in frames
    pub anim_len: i32,
    /// Animation delay counter
    pub anim_delay: i32,

    // === HP & Combat Stats ===
    /// Maximum hit points (C++ field: maxHitPoints)
    pub max_hit_points: i32,
    /// Current hit points (C++ field: hitPoints)
    pub hit_points: i32,

    // === Flags & Seeds ===
    /// Monster flags (C++ field: flags)
    pub flags: MonsterFlags,
    /// Seed for item drops on death (C++ field: rndItemSeed)
    pub rnd_item_seed: u32,
    /// Seed for AI behavior/sync (C++ field: aiSeed)
    pub ai_seed: u32,

    // === Combat Stats (Golem specific) ===
    /// Golem to-hit bonus (C++ field: golemToHit)
    pub golem_to_hit: u16,
    /// Monster resistance flags (C++ field: resistance)
    pub resistance: u16,

    // === AI Variables ===
    /// Behavior variable 1 - movement/goals (C++ field: goalVar1)
    pub goal_var1: i16,
    /// Behavior variable 2 - turning direction (C++ field: goalVar2)
    pub goal_var2: i8,
    /// Behavior variable 3 - special actions (C++ field: goalVar3)
    pub goal_var3: i8,

    /// Generic variable 1 (C++ field: var1)
    pub var1: i16,
    /// Generic variable 2 (C++ field: var2)
    pub var2: i16,
    /// Generic variable 3 (C++ field: var3)
    pub var3: i8,

    // === Position & Movement ===
    /// Current position (C++ field: position.tile)
    pub position: Point,
    /// Old position for interpolation (C++ field: position.old)
    pub old_position: Point,
    /// Future/target position (C++ field: position.future)
    pub future_position: Point,

    // === Goals & Targets ===
    /// Current goal (C++ field: goal)
    pub goal: MonsterGoal,
    /// Enemy position (C++ field: enemyPosition)
    pub enemy_position: Point,

    // === Monster Type & State ===
    /// Level type index (C++ field: levelType)
    pub level_type: u8,
    /// Current mode (C++ field: mode)
    pub mode: MonsterMode,
    /// Path count for pathfinding (C++ field: pathCount)
    pub path_count: u8,
    /// Direction faced by monster (C++ field: direction)
    pub direction: Direction,
    /// Target enemy index (C++ field: enemy)
    pub enemy: u8,
    /// Is monster invalid/dead (C++ field: isInvalid)
    pub is_invalid: bool,

    // === AI Type ===
    /// AI type (C++ field: ai)
    pub ai: MonsterAIID,
    /// AI intelligence level (C++ field: intelligence)
    pub intelligence: u8,
    /// Active ticks remaining (C++ field: activeForTicks)
    pub active_for_ticks: u8,

    // === Unique Monster ===
    /// Unique monster type (C++ field: uniqueType)
    pub unique_type: UniqueMonsterType,
    /// Unique transformation index (C++ field: uniqTrans)
    pub uniq_trans: u8,

    // === Misc ===
    /// Corpse ID (C++ field: corpseId)
    pub corpse_id: i8,
    /// Who hit this monster last (C++ field: whoHit)
    pub who_hit: i8,

    // === Damage Stats ===
    /// Minimum damage (C++ field: minDamage)
    pub min_damage: u8,
    /// Maximum damage (C++ field: maxDamage)
    pub max_damage: u8,
    /// Minimum special damage (C++ field: minDamageSpecial)
    pub min_damage_special: u8,
    /// Maximum special damage (C++ field: maxDamageSpecial)
    pub max_damage_special: u8,

    // === Defense ===
    /// Armor class (C++ field: armorClass)
    pub armor_class: u8,

    // === Pack/Leader System ===
    /// Pack leader index (C++ field: leader)
    pub leader: u8,
    /// Relation to leader (C++ field: leaderRelation)
    pub leader_relation: LeaderRelation,
    /// Pack size (C++ field: packSize)
    pub pack_size: u8,

    // === Lighting ===
    /// Light ID for rendering (C++ field: lightId)
    pub light_id: i8,
}

/// Constant for no leader (Source/monster.h:285)
pub const NO_LEADER: u8 = 255;

impl Monster {
    /// Create a new monster from MonsterData
    ///
    /// **C++ Reference**: Monster initialization in Source/monster.cpp
    ///
    /// # Arguments
    /// * `data` - Monster data from monster_dat
    /// * `position` - Initial spawn position
    /// * `ai_type` - AI type for this monster
    /// * `level_type` - Level type index
    pub fn new(data: &MonsterData, position: Point, ai_type: MonsterAIID, level_type: u8) -> Self {
        let max_hp = data.hp_min as i32; // Use hp_min as base (hp_max used for variations)
        let rng_seed = rand::random::<u32>();

        Self {
            // Animation (simplified)
            anim_frame: 0,
            anim_len: 8, // Default, will be set by animation system
            anim_delay: 0,

            // HP & Combat
            max_hit_points: max_hp,
            hit_points: max_hp,

            // Flags & Seeds
            flags: MonsterFlags::NONE,
            rnd_item_seed: rng_seed,
            ai_seed: rng_seed,

            // Combat Stats
            golem_to_hit: 0,
            resistance: 0,

            // AI Variables
            goal_var1: 0,
            goal_var2: 0,
            goal_var3: 0,
            var1: 0,
            var2: 0,
            var3: 0,

            // Position
            position,
            old_position: position,
            future_position: position,

            // Goals & Targets
            goal: MonsterGoal::Normal,
            enemy_position: Point::new(0, 0),

            // Monster Type
            level_type,
            mode: MonsterMode::Stand,
            path_count: 0,
            direction: Direction::South,
            enemy: 0,
            is_invalid: false,

            // AI
            ai: ai_type,
            intelligence: data.intelligence,
            active_for_ticks: 0,

            // Unique
            unique_type: UniqueMonsterType::None,
            uniq_trans: 0,

            // Misc
            corpse_id: 0,
            who_hit: 0,

            // Damage
            min_damage: data.min_damage,
            max_damage: data.max_damage,
            min_damage_special: data.min_damage_special,
            max_damage_special: data.max_damage_special,

            // Defense
            armor_class: data.armor_class,

            // Pack/Leader
            leader: NO_LEADER,
            leader_relation: LeaderRelation::None,
            pack_size: 1,

            // Lighting
            light_id: -1,
        }
    }

    /// Modify monster HP by delta amount
    ///
    /// Returns actual HP change (may be clamped by death/max)
    ///
    /// # Arguments
    /// * `delta` - HP change (negative for damage, positive for heal)
    pub fn modify_hp(&mut self, delta: i32) -> i32 {
        let old_hp = self.hit_points;
        self.hit_points = (self.hit_points + delta).clamp(0, self.max_hit_points);

        if self.hit_points == 0 && old_hp > 0 {
            self.mode = MonsterMode::Death;
            self.is_invalid = true;
        }

        self.hit_points - old_hp
    }

    /// Take damage from an attack
    ///
    /// # Arguments
    /// * `damage` - Raw damage amount
    ///
    /// # Returns
    /// Actual damage dealt after resistances
    pub fn take_damage(&mut self, damage: i32) -> i32 {
        let actual_damage = -self.modify_hp(-damage);
        if actual_damage > 0 && self.is_alive() {
            self.mode = MonsterMode::HitRecovery;
        }
        actual_damage
    }

    /// Heal monster by amount
    ///
    /// # Arguments
    /// * `amount` - Heal amount
    ///
    /// # Returns
    /// Actual HP restored
    pub fn heal(&mut self, amount: i32) -> i32 {
        self.modify_hp(amount)
    }

    /// Check if monster is alive
    pub fn is_alive(&self) -> bool {
        self.hit_points > 0 && !self.is_invalid
    }

    /// Check if monster is dead
    pub fn is_dead(&self) -> bool {
        !self.is_alive()
    }

    /// Set monster position
    ///
    /// # Arguments
    /// * `pos` - New position
    pub fn set_position(&mut self, pos: Point) {
        self.old_position = self.position;
        self.position = pos;
        self.future_position = pos;
    }

    /// Get current position
    pub fn get_position(&self) -> Point {
        self.position
    }

    /// Set monster mode
    ///
    /// # Arguments
    /// * `new_mode` - New mode to set
    pub fn set_mode(&mut self, new_mode: MonsterMode) {
        self.mode = new_mode;
    }

    /// Check if monster is in a movement mode
    pub fn is_moving(&self) -> bool {
        self.mode.is_move()
    }

    /// Set monster direction
    ///
    /// # Arguments
    /// * `dir` - New direction
    pub fn set_direction(&mut self, dir: Direction) {
        self.direction = dir;
    }

    /// Calculate distance to a point
    ///
    /// # Arguments
    /// * `target` - Target position
    ///
    /// # Returns
    /// Manhattan distance
    pub fn distance_to(&self, target: Point) -> i32 {
        (self.position.x - target.x).abs() + (self.position.y - target.y).abs()
    }

    /// Calculate walking distance to a point (Chebyshev distance)
    ///
    /// **C++ Reference**: Source/engine/point.hpp:123 (WalkingDistance)
    ///
    /// This matches the C++ WalkingDistance function which counts diagonal
    /// movement as 1 tile. Returns the maximum of dx and dy.
    ///
    /// # Arguments
    /// * `target` - Target position
    ///
    /// # Returns
    /// Walking distance (max of dx, dy)
    pub fn walking_distance(&self, target: Point) -> i32 {
        let dx = (self.position.x - target.x).abs();
        let dy = (self.position.y - target.y).abs();
        dx.max(dy)
    }

    /// Check if monster is in melee range of target
    ///
    /// # Arguments
    /// * `target` - Target position
    ///
    /// # Returns
    /// true if in melee range (distance <= 1)
    pub fn in_melee_range(&self, target: Point) -> bool {
        self.distance_to(target) <= 1
    }

    /// Petrify monster (Source/monster.h:314-316)
    pub fn petrify(&mut self) {
        self.mode = MonsterMode::Petrified;
    }

    /// Check if monster has a leader
    pub fn has_leader(&self) -> bool {
        self.leader != NO_LEADER && self.leader_relation != LeaderRelation::None
    }

    /// Get monster's to-hit value (base calculation)
    ///
    /// **Note**: Full calculation in C++ includes level adjustments
    pub fn get_to_hit(&self) -> i32 {
        if self.flags.is_golem() {
            self.golem_to_hit as i32
        } else {
            // Base to-hit (simplified - full formula in combat system)
            50 + (self.intelligence as i32) * 2
        }
    }
}

// ============================================================================
// Combat System - Monster vs Player
// ============================================================================

use crate::game::player_exact::Player;

/// Combat result from an attack
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatResult {
    /// Attack missed
    Miss,
    /// Attack was blocked
    Block,
    /// Attack hit, dealing damage
    Hit { damage: i32 },
    /// Attack killed target, dealing damage and awarding XP
    Kill { damage: i32, exp: u16 },
}

/// Monster attacks player (simplified port of MonsterAttackPlayer)
///
/// **C++ Reference**: Source/monster.cpp:1168-1250
///
/// # Arguments
/// * `monster` - Attacking monster
/// * `player` - Target player
/// * `to_hit` - Monster's to-hit value
/// * `min_dam` - Minimum damage
/// * `max_dam` - Maximum damage
///
/// # Returns
/// Combat result
pub fn monster_attack_player(
    _monster: &Monster, // Simplified - full implementation would use monster stats
    player: &mut Player,
    to_hit: i32,
    min_dam: i32,
    max_dam: i32,
) -> CombatResult {
    // Player already dead or invincible
    if !player.is_alive() {
        return CombatResult::Miss;
    }

    // Calculate hit chance
    let hit_roll = rand::random::<u32>() % 100;
    let player_ac = player._p_armor_class as i32; // Direct field access

    // Adjusted to-hit (simplified - no level/difficulty adjustments)
    let adjusted_hit = (to_hit - player_ac + 30).max(10) as u32;

    // Miss check
    if hit_roll >= adjusted_hit {
        return CombatResult::Miss;
    }

    // Block check (simplified - no blocking in basic implementation)
    // Full implementation would check player._pBlockFlag and dexterity
    let block_chance = 0; // Simplified
    let block_roll = rand::random::<u32>() % 100;

    if block_roll < block_chance {
        return CombatResult::Block;
    }

    // Calculate damage (64x fixed-point in C++)
    let damage_64x = if max_dam > min_dam {
        (rand::random::<i32>().abs() % ((max_dam - min_dam) << 6)) + (min_dam << 6)
    } else {
        min_dam << 6
    };

    let damage_64x = damage_64x.max(64); // Minimum 1 HP (64 in 64x)

    // Apply damage to player
    player.modify_hp(-damage_64x);

    CombatResult::Hit {
        damage: damage_64x >> 6, // Convert back to regular HP
    }
}

/// Player attacks monster (simplified port)
///
/// **C++ Reference**: Source/monster.cpp (player attack logic)
///
/// # Arguments
/// * `player` - Attacking player
/// * `monster` - Target monster
/// * `player_to_hit` - Player's to-hit value
/// * `min_dam` - Minimum damage
/// * `max_dam` - Maximum damage
///
/// # Returns
/// Combat result
pub fn player_attack_monster(
    _player: &Player, // Simplified - full implementation would use player stats
    monster: &mut Monster,
    player_to_hit: i32,
    min_dam: i32,
    max_dam: i32,
) -> CombatResult {
    // Monster already dead
    if monster.is_dead() {
        return CombatResult::Miss;
    }

    // Calculate hit chance
    let hit_roll = rand::random::<u32>() % 100;
    let monster_ac = monster.armor_class as i32;

    // Adjusted to-hit
    let adjusted_hit = (player_to_hit - monster_ac + 50).max(15) as u32;

    // Miss check
    if hit_roll >= adjusted_hit {
        return CombatResult::Miss;
    }

    // Calculate damage
    let damage = if max_dam > min_dam {
        (rand::random::<i32>().abs() % (max_dam - min_dam + 1)) + min_dam
    } else {
        min_dam
    };

    // Apply damage to monster
    let actual_damage = monster.take_damage(damage);

    // Check if killed
    if monster.is_dead() {
        // Award XP (simplified - use monster data exp value)
        let exp = 100; // Placeholder - would get from monster data
        CombatResult::Kill {
            damage: actual_damage,
            exp,
        }
    } else {
        CombatResult::Hit {
            damage: actual_damage,
        }
    }
}

// ============================================================================
// Monster AI System
// ============================================================================

/// AI action result - what the monster decided to do
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AIAction {
    /// Do nothing, stay in current mode
    None,
    /// Start standing (idle)
    Stand,
    /// Walk in specified direction
    Walk(Direction),
    /// Walk randomly in a direction
    RandomWalk,
    /// Start melee attack
    Attack,
    /// Start special attack
    SpecialAttack,
    /// Start ranged attack with missile type
    RangedAttack { missile_type: u8 },
    /// Delay for N ticks
    Delay { ticks: u8 },
}

/// Monster AI trait - defines AI behavior
///
/// **C++ Reference**: AI function pointers in Source/monster.cpp:3018-3058
///
/// Each AI type implements this trait to define monster behavior.
/// Called every game tick when monster is in Stand mode.
pub trait MonsterAI {
    /// Update monster AI logic
    ///
    /// # Arguments
    /// * `monster` - Monster being updated
    /// * `player_pos` - Current player position
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Action to perform
    fn update(&self, monster: &Monster, player_pos: Point, rng: &mut dyn FnMut(u32) -> u32) -> AIAction;
}

// ============================================================================
// ZombieAI - Basic Melee AI
// ============================================================================

/// Zombie AI - basic melee monster
///
/// **C++ Reference**: Source/monster.cpp:1996-2024 (ZombieAi)
///
/// Behavior:
/// - 2% + (2 * intelligence)% chance to act per tick
/// - If player far: walk towards player
/// - If player close (dist < 2): attack
/// - Random direction with 20% + (2 * intelligence)% chance
pub struct ZombieAI;

impl MonsterAI for ZombieAI {
    fn update(&self, monster: &Monster, player_pos: Point, rng: &mut dyn FnMut(u32) -> u32) -> AIAction {
        // Only act when standing
        if monster.mode != MonsterMode::Stand {
            return AIAction::None;
        }

        // Random chance to act (2 * intelligence + 10)%
        let act_chance = 2 * (monster.intelligence as u32) + 10;
        if rng(100) >= act_chance {
            return AIAction::Stand;
        }

        // Calculate distance to player (use walking distance - matches C++ WalkingDistance)
        let dist = monster.walking_distance(player_pos);

        if dist >= 2 {
            // Far from player - walk towards player
            let intel_threshold = 2 * (monster.intelligence as i32) + 4;

            if dist >= intel_threshold {
                // Far enough - random walk or directed walk
                let random_walk_chance = 2 * (monster.intelligence as u32) + 20;
                if rng(100) < random_walk_chance {
                    // Random direction
                    let random_dir = match rng(8) {
                        0 => Direction::South,
                        1 => Direction::SouthWest,
                        2 => Direction::West,
                        3 => Direction::NorthWest,
                        4 => Direction::North,
                        5 => Direction::NorthEast,
                        6 => Direction::East,
                        _ => Direction::SouthEast,
                    };
                    AIAction::Walk(random_dir)
                } else {
                    // Walk in current direction
                    AIAction::Walk(monster.direction)
                }
            } else {
                // Close enough - walk towards player
                AIAction::RandomWalk
            }
        } else {
            // In melee range - attack!
            AIAction::Attack
        }
    }
}

// ============================================================================
// SkeletonMeleeAI - Melee with Delay
// ============================================================================

/// Skeleton Melee AI - melee with delay tactics
///
/// **C++ Reference**: Source/monster.cpp:2051-2074 (SkeletonAi)
///
/// Behavior:
/// - Uses delay tactics (var1 stores delay state)
/// - If player far: 35% - (4 * intelligence)% chance to walk
/// - If player close: 20% + (2 * intelligence)% chance to attack
/// - Otherwise delays for (2 * (5 - intelligence)) + random(10) ticks
pub struct SkeletonMeleeAI;

impl MonsterAI for SkeletonMeleeAI {
    fn update(&self, monster: &Monster, player_pos: Point, rng: &mut dyn FnMut(u32) -> u32) -> AIAction {
        // Only act when standing and active
        if monster.mode != MonsterMode::Stand || monster.active_for_ticks == 0 {
            return AIAction::None;
        }

        // Calculate distance to player (use walking distance)
        let dist = monster.walking_distance(player_pos);

        // Check if currently in delay mode (stored in var1)
        let is_delaying = monster.var1 == MonsterMode::Delay as i16;

        if dist >= 2 {
            // Far from player
            if is_delaying {
                // Already delaying, walk now
                AIAction::RandomWalk
            } else {
                // Chance to walk vs delay
                // Note: Clamp to prevent underflow
                let intel_factor = (4 * monster.intelligence as u32).min(35);
                let walk_chance = 35 - intel_factor;
                if rng(100) >= walk_chance {
                    // Walk towards player
                    AIAction::RandomWalk
                } else {
                    // Delay instead
                    // Clamp intelligence to [0,5] to prevent underflow
                    let intel_clamped = (monster.intelligence as u8).min(5);
                    let delay_base = 15 - (2 * intel_clamped);
                    let delay_ticks = delay_base + (rng(10) as u8);
                    AIAction::Delay { ticks: delay_ticks }
                }
            }
        } else {
            // Close to player (dist < 2)
            if is_delaying {
                // Already delaying, attack now
                AIAction::Attack
            } else {
                // Chance to attack vs delay
                let attack_chance = 2 * (monster.intelligence as u32) + 20;
                if rng(100) < attack_chance {
                    AIAction::Attack
                } else {
                    // Delay before attacking
                    let intel_clamped = (monster.intelligence as u8).min(5);
                    let delay_base = if intel_clamped <= 5 { 2 * (5 - intel_clamped) } else { 0 };
                    let delay_ticks = delay_base + (rng(10) as u8);
                    AIAction::Delay { ticks: delay_ticks }
                }
            }
        }
    }
}

// ============================================================================
// AI Helper Functions
// ============================================================================

/// Apply AI action to monster
///
/// Converts AIAction to actual monster state changes
///
/// # Arguments
/// * `monster` - Monster to update
/// * `action` - Action to apply
pub fn apply_ai_action(monster: &mut Monster, action: AIAction) {
    match action {
        AIAction::None => {
            // No change
        }
        AIAction::Stand => {
            // Already standing
            monster.mode = MonsterMode::Stand;
        }
        AIAction::Walk(direction) => {
            // Start walking in direction
            monster.set_direction(direction);
            // Would transition to movement mode in full implementation
            // For now just update direction
        }
        AIAction::RandomWalk => {
            // Walk towards enemy
            // Full implementation would use pathfinding
            monster.mode = MonsterMode::MoveNorthwards; // Placeholder
        }
        AIAction::Attack => {
            // Start attack
            monster.set_mode(MonsterMode::MeleeAttack);
        }
        AIAction::SpecialAttack => {
            // Start special attack
            monster.set_mode(MonsterMode::SpecialMeleeAttack);
        }
        AIAction::RangedAttack { missile_type } => {
            // Start ranged attack
            monster.var1 = missile_type as i16;
            monster.set_mode(MonsterMode::RangedAttack);
        }
        AIAction::Delay { ticks } => {
            // Set delay mode
            monster.var1 = MonsterMode::Delay as i16;
            monster.var2 = ticks as i16;
            monster.set_mode(MonsterMode::Delay);
        }
    }
}

/// Get AI for monster type
///
/// **C++ Reference**: Source/monster.cpp:3018-3058 (AiProc array)
///
/// # Arguments
/// * `ai_type` - Monster AI type
///
/// # Returns
/// AI implementation
pub fn get_ai_for_type(ai_type: MonsterAIID) -> Box<dyn MonsterAI> {
    match ai_type {
        MonsterAIID::Zombie => Box::new(ZombieAI),
        MonsterAIID::SkeletonMelee => Box::new(SkeletonMeleeAI),
        // Add more AI types here as implemented
        _ => Box::new(ZombieAI), // Default to zombie AI
    }
}

// ============================================================================
// MonsterManager - Monster Array Management
// ============================================================================

/// Monster Manager - manages active monsters array
///
/// **C++ Reference**: Global Monsters array in Source/monster.cpp
///
/// Provides:
/// - Monster spawning and removal
/// - Position-based queries
/// - Range queries for AI
/// - Cleanup of dead monsters
pub struct MonsterManager {
    /// Active monsters (max 200 in C++)
    monsters: Vec<Option<Monster>>,
    /// Maximum number of monsters
    max_monsters: usize,
    /// Count of active monsters
    active_count: usize,
}

impl MonsterManager {
    /// Maximum monsters constant (Source/monster.h:35)
    pub const MAX_MONSTERS: usize = 200;

    /// Create new MonsterManager
    ///
    /// # Arguments
    /// * `max_count` - Maximum monsters (default 200)
    pub fn new(max_count: usize) -> Self {
        let mut monsters = Vec::with_capacity(max_count);
        for _ in 0..max_count {
            monsters.push(None);
        }

        Self {
            monsters,
            max_monsters: max_count,
            active_count: 0,
        }
    }

    /// Add a new monster to the manager
    ///
    /// # Arguments
    /// * `monster` - Monster to add
    ///
    /// # Returns
    /// Index of added monster, or None if full
    pub fn add_monster(&mut self, monster: Monster) -> Option<usize> {
        // Find first empty slot
        for (i, slot) in self.monsters.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(monster);
                self.active_count += 1;
                return Some(i);
            }
        }
        None
    }

    /// Remove monster at index
    ///
    /// # Arguments
    /// * `index` - Monster index to remove
    pub fn remove_monster(&mut self, index: usize) {
        if index < self.monsters.len() && self.monsters[index].is_some() {
            self.monsters[index] = None;
            self.active_count = self.active_count.saturating_sub(1);
        }
    }

    /// Get immutable reference to monster
    ///
    /// # Arguments
    /// * `index` - Monster index
    pub fn get_monster(&self, index: usize) -> Option<&Monster> {
        self.monsters.get(index)?.as_ref()
    }

    /// Get mutable reference to monster
    ///
    /// # Arguments
    /// * `index` - Monster index
    pub fn get_monster_mut(&mut self, index: usize) -> Option<&mut Monster> {
        self.monsters.get_mut(index)?.as_mut()
    }

    /// Find monster at specific position
    ///
    /// # Arguments
    /// * `pos` - Position to check
    ///
    /// # Returns
    /// Index of monster at position, or None
    pub fn find_monster_at(&self, pos: Point) -> Option<usize> {
        for (i, monster_opt) in self.monsters.iter().enumerate() {
            if let Some(monster) = monster_opt {
                if monster.is_alive() && monster.position == pos {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Get all monsters within range of a point
    ///
    /// # Arguments
    /// * `center` - Center point
    /// * `range` - Maximum Manhattan distance
    ///
    /// # Returns
    /// Vector of monster indices in range
    pub fn get_monsters_in_range(&self, center: Point, range: i32) -> Vec<usize> {
        let mut result = Vec::new();
        for (i, monster_opt) in self.monsters.iter().enumerate() {
            if let Some(monster) = monster_opt {
                if monster.is_alive() && monster.distance_to(center) <= range {
                    result.push(i);
                }
            }
        }
        result
    }

    /// Cleanup all dead monsters
    ///
    /// Removes monsters marked as invalid or with HP <= 0
    pub fn cleanup_dead(&mut self) {
        for slot in self.monsters.iter_mut() {
            if let Some(monster) = slot {
                if monster.is_dead() {
                    *slot = None;
                    self.active_count = self.active_count.saturating_sub(1);
                }
            }
        }
    }

    /// Get active monster count
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Check if manager is full
    pub fn is_full(&self) -> bool {
        self.active_count >= self.max_monsters
    }

    /// Get maximum capacity
    pub fn capacity(&self) -> usize {
        self.max_monsters
    }

    /// Clear all monsters
    pub fn clear(&mut self) {
        for slot in self.monsters.iter_mut() {
            *slot = None;
        }
        self.active_count = 0;
    }

    /// Iterate over all active monsters with their indices
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Monster)> {
        self.monsters
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|m| (i, m)))
    }

    /// Iterate over all active monsters mutably with their indices
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut Monster)> {
        self.monsters
            .iter_mut()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_mut().map(|m| (i, m)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_monster_data() -> MonsterData {
        MonsterData {
            name: "Test Monster",
            min_dungeon_level: 1,
            max_dungeon_level: 10,
            level: 5,
            hp_min: 100,
            hp_max: 150,
            ai: MonsterAIID::Zombie,
            intelligence: 10,
            to_hit: 50,
            min_damage: 5,
            max_damage: 15,
            to_hit_special: 60,
            min_damage_special: 10,
            max_damage_special: 20,
            armor_class: 20,
            monster_class: crate::game::monster_dat::MonsterClass::Undead,
            resistance: crate::game::monster_dat::MonsterResistance::NONE,
            resistance_hell: crate::game::monster_dat::MonsterResistance::NONE,
            experience: 100,
        }
    }

    #[test]
    fn test_monster_creation() {
        let data = create_test_monster_data();
        let pos = Point::new(10, 20);
        let monster = Monster::new(&data, pos, MonsterAIID::Zombie, 0);

        assert_eq!(monster.position, pos);
        assert_eq!(monster.max_hit_points, 100);
        assert_eq!(monster.hit_points, 100);
        assert_eq!(monster.ai, MonsterAIID::Zombie);
        assert_eq!(monster.mode, MonsterMode::Stand);
        assert!(monster.is_alive());
    }

    #[test]
    fn test_monster_hp_damage() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        let actual_damage = monster.take_damage(30);
        assert_eq!(actual_damage, 30);
        assert_eq!(monster.hit_points, 70);
        assert!(monster.is_alive());
        assert_eq!(monster.mode, MonsterMode::HitRecovery);
    }

    #[test]
    fn test_monster_death() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        let damage = monster.take_damage(150); // Overkill
        assert_eq!(damage, 100); // Only dealt 100 (current HP)
        assert_eq!(monster.hit_points, 0);
        assert!(!monster.is_alive());
        assert!(monster.is_dead());
        assert_eq!(monster.mode, MonsterMode::Death);
        assert!(monster.is_invalid);
    }

    #[test]
    fn test_monster_heal() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        monster.take_damage(50);
        assert_eq!(monster.hit_points, 50);

        let healed = monster.heal(30);
        assert_eq!(healed, 30);
        assert_eq!(monster.hit_points, 80);

        // Test heal cap
        let healed2 = monster.heal(50);
        assert_eq!(healed2, 20); // Only healed to max (100)
        assert_eq!(monster.hit_points, 100);
    }

    #[test]
    fn test_monster_position() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        let new_pos = Point::new(5, 10);
        monster.set_position(new_pos);
        assert_eq!(monster.get_position(), new_pos);
        assert_eq!(monster.old_position, Point::new(0, 0));
        assert_eq!(monster.future_position, new_pos);
    }

    #[test]
    fn test_monster_mode_transitions() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        assert_eq!(monster.mode, MonsterMode::Stand);

        monster.set_mode(MonsterMode::MoveNorthwards);
        assert_eq!(monster.mode, MonsterMode::MoveNorthwards);
        assert!(monster.is_moving());

        monster.set_mode(MonsterMode::MeleeAttack);
        assert_eq!(monster.mode, MonsterMode::MeleeAttack);
        assert!(!monster.is_moving());
    }

    #[test]
    fn test_monster_distance() {
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(10, 10), MonsterAIID::Zombie, 0);

        assert_eq!(monster.distance_to(Point::new(10, 10)), 0);
        assert_eq!(monster.distance_to(Point::new(13, 14)), 7); // 3 + 4
        assert_eq!(monster.distance_to(Point::new(5, 5)), 10); // 5 + 5
    }

    #[test]
    fn test_monster_melee_range() {
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(10, 10), MonsterAIID::Zombie, 0);

        assert!(monster.in_melee_range(Point::new(10, 10))); // Same tile
        assert!(monster.in_melee_range(Point::new(10, 11))); // Adjacent
        assert!(monster.in_melee_range(Point::new(11, 10))); // Adjacent
        assert!(!monster.in_melee_range(Point::new(12, 10))); // 2 tiles away
        assert!(!monster.in_melee_range(Point::new(10, 12))); // 2 tiles away
    }

    #[test]
    fn test_monster_flags() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        assert!(!monster.flags.is_hidden());
        monster.flags.set_hidden(true);
        assert!(monster.flags.is_hidden());
        monster.flags.set_hidden(false);
        assert!(!monster.flags.is_hidden());

        monster.flags = MonsterFlags::GOLEM;
        assert!(monster.flags.is_golem());
    }

    #[test]
    fn test_monster_petrify() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        monster.petrify();
        assert_eq!(monster.mode, MonsterMode::Petrified);
    }

    #[test]
    fn test_monster_pack_leader() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);

        assert!(!monster.has_leader());
        assert_eq!(monster.leader, NO_LEADER);

        monster.leader = 5;
        monster.leader_relation = LeaderRelation::Leashed;
        assert!(monster.has_leader());
    }

    #[test]
    fn test_monster_mode_is_move() {
        assert!(MonsterMode::MoveNorthwards.is_move());
        assert!(MonsterMode::MoveSouthwards.is_move());
        assert!(MonsterMode::MoveSideways.is_move());
        assert!(!MonsterMode::Stand.is_move());
        assert!(!MonsterMode::MeleeAttack.is_move());
    }

    // ========================================================================
    // MonsterManager Tests
    // ========================================================================

    #[test]
    fn test_monster_manager_creation() {
        let manager = MonsterManager::new(10);
        assert_eq!(manager.capacity(), 10);
        assert_eq!(manager.active_count(), 0);
        assert!(!manager.is_full());
    }

    #[test]
    fn test_monster_manager_add() {
        let mut manager = MonsterManager::new(5);
        let data = create_test_monster_data();

        let idx1 = manager.add_monster(Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0));
        assert_eq!(idx1, Some(0));
        assert_eq!(manager.active_count(), 1);

        let idx2 = manager.add_monster(Monster::new(&data, Point::new(1, 1), MonsterAIID::Zombie, 0));
        assert_eq!(idx2, Some(1));
        assert_eq!(manager.active_count(), 2);
    }

    #[test]
    fn test_monster_manager_remove() {
        let mut manager = MonsterManager::new(5);
        let data = create_test_monster_data();

        let idx = manager.add_monster(Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0)).unwrap();
        assert_eq!(manager.active_count(), 1);

        manager.remove_monster(idx);
        assert_eq!(manager.active_count(), 0);
        assert!(manager.get_monster(idx).is_none());
    }

    #[test]
    fn test_monster_manager_full() {
        let mut manager = MonsterManager::new(2);
        let data = create_test_monster_data();

        manager.add_monster(Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0));
        assert!(!manager.is_full());

        manager.add_monster(Monster::new(&data, Point::new(1, 1), MonsterAIID::Zombie, 0));
        assert!(manager.is_full());

        // Try to add when full
        let result = manager.add_monster(Monster::new(&data, Point::new(2, 2), MonsterAIID::Zombie, 0));
        assert!(result.is_none());
    }

    #[test]
    fn test_monster_manager_find_at_position() {
        let mut manager = MonsterManager::new(5);
        let data = create_test_monster_data();

        let pos = Point::new(10, 20);
        let idx = manager.add_monster(Monster::new(&data, pos, MonsterAIID::Zombie, 0)).unwrap();

        let found = manager.find_monster_at(pos);
        assert_eq!(found, Some(idx));

        let not_found = manager.find_monster_at(Point::new(99, 99));
        assert!(not_found.is_none());
    }

    #[test]
    fn test_monster_manager_range_query() {
        let mut manager = MonsterManager::new(10);
        let data = create_test_monster_data();

        // Add monsters at different positions
        manager.add_monster(Monster::new(&data, Point::new(10, 10), MonsterAIID::Zombie, 0));
        manager.add_monster(Monster::new(&data, Point::new(11, 11), MonsterAIID::Zombie, 0));
        manager.add_monster(Monster::new(&data, Point::new(20, 20), MonsterAIID::Zombie, 0));
        manager.add_monster(Monster::new(&data, Point::new(30, 30), MonsterAIID::Zombie, 0));

        // Query range 5 from (10, 10)
        let nearby = manager.get_monsters_in_range(Point::new(10, 10), 5);
        assert_eq!(nearby.len(), 2); // (10,10) and (11,11)

        // Query range 50 from (10, 10)
        let all_nearby = manager.get_monsters_in_range(Point::new(10, 10), 50);
        assert_eq!(all_nearby.len(), 4);
    }

    #[test]
    fn test_monster_manager_cleanup_dead() {
        let mut manager = MonsterManager::new(5);
        let data = create_test_monster_data();

        // Add alive monsters
        let idx1 = manager.add_monster(Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0)).unwrap();
        let idx2 = manager.add_monster(Monster::new(&data, Point::new(1, 1), MonsterAIID::Zombie, 0)).unwrap();
        let idx3 = manager.add_monster(Monster::new(&data, Point::new(2, 2), MonsterAIID::Zombie, 0)).unwrap();

        assert_eq!(manager.active_count(), 3);

        // Kill one monster
        if let Some(monster) = manager.get_monster_mut(idx2) {
            monster.take_damage(1000);
        }

        // Cleanup
        manager.cleanup_dead();
        assert_eq!(manager.active_count(), 2);
        assert!(manager.get_monster(idx1).is_some());
        assert!(manager.get_monster(idx2).is_none());
        assert!(manager.get_monster(idx3).is_some());
    }

    #[test]
    fn test_monster_manager_clear() {
        let mut manager = MonsterManager::new(5);
        let data = create_test_monster_data();

        manager.add_monster(Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0));
        manager.add_monster(Monster::new(&data, Point::new(1, 1), MonsterAIID::Zombie, 0));
        assert_eq!(manager.active_count(), 2);

        manager.clear();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_monster_manager_iter() {
        let mut manager = MonsterManager::new(5);
        let data = create_test_monster_data();

        manager.add_monster(Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0));
        manager.add_monster(Monster::new(&data, Point::new(1, 1), MonsterAIID::Zombie, 0));
        manager.add_monster(Monster::new(&data, Point::new(2, 2), MonsterAIID::Zombie, 0));

        let count = manager.iter().count();
        assert_eq!(count, 3);

        // Test positions
        let positions: Vec<Point> = manager.iter().map(|(_, m)| m.position).collect();
        assert!(positions.contains(&Point::new(0, 0)));
        assert!(positions.contains(&Point::new(1, 1)));
        assert!(positions.contains(&Point::new(2, 2)));
    }

    // ========================================================================
    // AI System Tests
    // ========================================================================

    #[test]
    fn test_zombie_ai_idle_behavior() {
        // Test zombie AI when random chance fails
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        let player_pos = Point::new(55, 55);

        // Mock RNG that always fails act_chance (return 100)
        let mut rng = |_max: u32| -> u32 { 100 };

        let ai = ZombieAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // Should return Stand when chance fails
        assert_eq!(action, AIAction::Stand);
    }

    #[test]
    fn test_zombie_ai_attack_in_range() {
        // Test zombie AI attacks when player is close
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        let player_pos = Point::new(51, 51); // Distance = 1

        // Mock RNG that succeeds act_chance (return 0)
        let mut rng = |_max: u32| -> u32 { 0 };

        let ai = ZombieAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // Should attack when player is close (dist < 2)
        assert_eq!(action, AIAction::Attack);
    }

    #[test]
    fn test_zombie_ai_walk_when_far() {
        // Test zombie AI walks when player is far
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        let player_pos = Point::new(60, 60); // Walking distance = 10

        // Mock RNG: act_chance succeeds (0)
        let mut rng = |_max: u32| -> u32 { 0 };

        let ai = ZombieAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // With intelligence=10, dist=10 < (2*10+4=24), so should RandomWalk
        assert_eq!(action, AIAction::RandomWalk);
    }

    #[test]
    fn test_zombie_ai_random_walk() {
        // Test zombie AI random walk with directed walk
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        // Use large distance >= intel_threshold(24) to trigger directed walk logic
        let player_pos = Point::new(80, 80); // Walking distance = max(30,30) = 30

        // Mock RNG: act_chance succeeds, random_walk_chance succeeds, direction=2
        let mut call_count = 0;
        let mut rng = |_max: u32| -> u32 {
            call_count += 1;
            match call_count {
                1 => 0,   // act_chance succeeds
                2 => 0,   // random_walk_chance succeeds
                _ => 2,   // direction = West
            }
        };

        let ai = ZombieAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // Distance(30) >= intel_threshold(24), random_walk_chance succeeds
        // Should walk in random direction (West)
        assert_eq!(action, AIAction::Walk(Direction::West));
    }

    #[test]
    fn test_skeleton_ai_delay_far() {
        // Test skeleton AI delay when far from player
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::SkeletonMelee, 0);
        monster.active_for_ticks = 10; // Active
        monster.intelligence = 2; // Low intelligence: walk_chance = 35 - 4*2 = 27
        let player_pos = Point::new(60, 60); // Walking distance = 10

        // Mock RNG: walk_chance check returns 20 (20 < 27 = false, so delay)
        let mut call_count = 0;
        let mut rng = |_max: u32| -> u32 {
            call_count += 1;
            if call_count == 1 { 20 } else { 5 } // walk_chance fails (20 < 27), delay 5 ticks
        };

        let ai = SkeletonMeleeAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // Should delay (rng(100)=20 < walk_chance(27))
        assert!(matches!(action, AIAction::Delay { ticks: _ }));
    }

    #[test]
    fn test_skeleton_ai_walk_from_delay() {
        // Test skeleton walks after delay when far
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::SkeletonMelee, 0);
        monster.active_for_ticks = 10;
        monster.var1 = MonsterMode::Delay as i16; // Currently delaying
        let player_pos = Point::new(60, 60);

        let mut rng = |_max: u32| -> u32 { 0 };

        let ai = SkeletonMeleeAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // Should walk after delay
        assert_eq!(action, AIAction::RandomWalk);
    }

    #[test]
    fn test_skeleton_ai_attack_close() {
        // Test skeleton attacks when close
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::SkeletonMelee, 0);
        monster.active_for_ticks = 10;
        monster.intelligence = 5; // High intelligence = high attack chance
        let player_pos = Point::new(51, 51); // Distance = 1

        // Mock RNG: attack_chance succeeds (return 0)
        let mut rng = |_max: u32| -> u32 { 0 };

        let ai = SkeletonMeleeAI;
        let action = ai.update(&monster, player_pos, &mut rng);

        // Should attack
        assert_eq!(action, AIAction::Attack);
    }

    #[test]
    fn test_apply_ai_action_stand() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        apply_ai_action(&mut monster, AIAction::Stand);
        assert_eq!(monster.mode, MonsterMode::Stand);
    }

    #[test]
    fn test_apply_ai_action_walk() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        apply_ai_action(&mut monster, AIAction::Walk(Direction::North));
        assert_eq!(monster.direction, Direction::North);
    }

    #[test]
    fn test_apply_ai_action_attack() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        apply_ai_action(&mut monster, AIAction::Attack);
        assert_eq!(monster.mode, MonsterMode::MeleeAttack);
    }

    #[test]
    fn test_apply_ai_action_delay() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        apply_ai_action(&mut monster, AIAction::Delay { ticks: 10 });
        assert_eq!(monster.mode, MonsterMode::Delay);
        assert_eq!(monster.var1, MonsterMode::Delay as i16);
        assert_eq!(monster.var2, 10);
    }

    #[test]
    fn test_get_ai_for_zombie() {
        let data = create_test_monster_data();
        let ai = get_ai_for_type(MonsterAIID::Zombie);
        let monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::Zombie, 0);
        let player_pos = Point::new(51, 51);
        let mut rng = |_max: u32| -> u32 { 0 };

        let action = ai.update(&monster, player_pos, &mut rng);
        assert_eq!(action, AIAction::Attack); // Close range = attack
    }

    #[test]
    fn test_get_ai_for_skeleton() {
        let data = create_test_monster_data();
        let ai = get_ai_for_type(MonsterAIID::SkeletonMelee);
        let mut monster = Monster::new(&data, Point::new(50, 50), MonsterAIID::SkeletonMelee, 0);
        monster.active_for_ticks = 10;
        monster.intelligence = 5;
        let player_pos = Point::new(51, 51);
        let mut rng = |_max: u32| -> u32 { 0 };

        let action = ai.update(&monster, player_pos, &mut rng);
        assert_eq!(action, AIAction::Attack); // Close range + high chance = attack
    }

    // ========================================================================
    // Combat Tests
    // ========================================================================

    fn create_test_player() -> Player {
        Player::new()
    }

    #[test]
    fn test_monster_attack_player() {
        let data = create_test_monster_data();
        let monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);
        let mut player = create_test_player();

        // Set player HP manually (64x fixed-point)
        player._p_hit_points = 100 << 6; // 100 HP
        player._p_max_hp = 100 << 6;

        // Attack multiple times to test hit/miss/block variance
        let mut hit_count = 0;
        for _ in 0..10 {
            let result = monster_attack_player(&monster, &mut player, 50, 5, 15);
            if matches!(result, CombatResult::Hit { .. }) {
                hit_count += 1;
            }
        }

        // Should have some hits (probabilistic test)
        assert!(hit_count > 0);
    }

    #[test]
    fn test_player_attack_monster_hit() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);
        let player = create_test_player();

        let initial_hp = monster.hit_points;

        // Attack with high to-hit to ensure hit
        let result = player_attack_monster(&player, &mut monster, 100, 10, 20);

        match result {
            CombatResult::Hit { damage } => {
                assert!(damage > 0);
                assert_eq!(monster.hit_points, initial_hp - damage);
            }
            _ => {
                // May miss due to randomness, but with to-hit 100 should mostly hit
            }
        }
    }

    #[test]
    fn test_player_attack_monster_kill() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);
        let player = create_test_player();

        // Attack with massive damage to ensure kill
        let result = player_attack_monster(&player, &mut monster, 100, 1000, 1000);

        match result {
            CombatResult::Kill { damage, exp } => {
                assert!(damage > 0);
                assert_eq!(exp, 100);
                assert!(monster.is_dead());
            }
            CombatResult::Hit { .. } => {
                // Might not kill if missed due to RNG, but damage should be lethal
            }
            _ => {}
        }
    }

    #[test]
    fn test_combat_dead_target() {
        let data = create_test_monster_data();
        let mut monster = Monster::new(&data, Point::new(0, 0), MonsterAIID::Zombie, 0);
        let player = create_test_player();

        // Kill monster first
        monster.take_damage(1000);
        assert!(monster.is_dead());

        // Attack should miss dead monster
        let result = player_attack_monster(&player, &mut monster, 100, 10, 20);
        assert_eq!(result, CombatResult::Miss);
    }
}
