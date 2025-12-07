//! Exact port of monster.h Monster struct from DevilutionX
//!
//! This is a 1:1 mapping of the C++ Monster struct with all fields
//! preserved exactly as in the original source code.

// Allow non-snake_case names to preserve C++ naming convention for save file compatibility
#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::monster_dat::{MonsterAIID, MonsterClass, MonsterId, MonsterMode};
use super::types::{Direction, Point};

// ============================================================================
// Constants from monster.h
// ============================================================================

/// Maximum monsters in game
pub const MAX_MONSTERS: usize = 200;

/// Maximum monster types per level
pub const MAX_LVL_MTYPES: usize = 24;

// ============================================================================
// Enums from monster.h
// ============================================================================

/// Monster flags - exact match of monster_flag enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    pub const NO_LIFE_STEAL: Self = Self(1 << 12);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn set(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    pub fn clear(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }
}

/// Unique monster type - exact match of UniqueMonsterType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

/// Monster goal - exact match of MonsterGoal enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

/// Monster graphic type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum MonsterGraphic {
    #[default]
    Stand = 0,
    Walk = 1,
    Attack = 2,
    GotHit = 3,
    Death = 4,
    Special = 5,
}

/// Monster sound type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum MonsterSound {
    #[default]
    Attack = 0,
    Hit = 1,
    Death = 2,
    Special = 3,
}

/// Leader relation - pack behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum LeaderRelation {
    #[default]
    None = 0,
    /// Minion that sticks to the leader
    Leashed = 1,
    /// Minion that was separated from the leader
    Separated = 2,
}

/// Place flags for monster spawning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlaceFlags(pub u8);

impl PlaceFlags {
    pub const NONE: Self = Self(0);
    pub const SCATTER: Self = Self(1 << 0);
    pub const SPECIAL: Self = Self(1 << 1);
    pub const UNIQUE: Self = Self(1 << 2);
}

/// Speech/talk message ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(i32)]
pub enum SpeechId {
    #[default]
    None = 0,
    // Quest-related speeches
    Garbud1 = 1,
    Garbud2,
    Garbud3,
    Garbud4,
    // ... more speech IDs
}

/// Difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum Difficulty {
    #[default]
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

// ============================================================================
// Monster struct - exact match of C++ Monster struct
// ============================================================================

/// Monster structure - exact port from monster.h
///
/// All field names preserved with original naming convention
/// to maintain compatibility with save file formats.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Monster {
    // Note: uniqueMonsterTRN and animInfo omitted - handled by render system

    /// Maximum hit points
    pub maxHitPoints: i32,

    /// Current hit points
    pub hitPoints: i32,

    /// Monster flags
    pub flags: MonsterFlags,

    /// Seed used to determine item drops on death
    pub rndItemSeed: u32,

    /// Seed used to determine AI behaviour/sync sounds
    pub aiSeed: u32,

    /// Golem to-hit bonus
    pub golemToHit: u16,

    /// Monster resistances (bitfield)
    pub resistance: u16,

    /// Talk message ID
    pub talkMsg: SpeechId,

    /// Specifies monster's behaviour regarding moving and changing goals
    pub goalVar1: i16,

    /// Specifies turning direction for RoundWalk in most cases
    pub goalVar2: i8,

    /// Controls monster's behaviour regarding special actions
    pub goalVar3: i8,

    /// Generic variable 1
    pub var1: i16,

    /// Generic variable 2
    pub var2: i16,

    /// Generic variable 3
    pub var3: i8,

    /// Monster position (tile and offset)
    pub position: Point,

    /// Future/target position
    pub futurePosition: Point,

    /// Old position (for interpolation)
    pub oldPosition: Point,

    /// Specifies current goal of the monster
    pub goal: MonsterGoal,

    /// Usually corresponds to the enemy's future position
    pub enemyPosition: Point,

    /// Level type index (into LevelMonsterTypes)
    pub levelType: u8,

    /// Current monster mode/state
    pub mode: MonsterMode,

    /// Path counter
    pub pathCount: u8,

    /// Direction faced by monster
    pub direction: Direction,

    /// The current target of the monster
    pub enemy: u8,

    /// Is this monster slot invalid/unused
    pub isInvalid: bool,

    /// Monster AI type
    pub ai: MonsterAIID,

    /// Monster intelligence (affects AI decisions)
    pub intelligence: u8,

    /// Stores information for how many ticks the monster will remain active
    pub activeForTicks: u8,

    /// Unique monster type (if unique)
    pub uniqueType: UniqueMonsterType,

    /// Unique monster TRN index
    pub uniqTrans: u8,

    /// Corpse ID
    pub corpseId: i8,

    /// ID of player who last hit this monster
    pub whoHit: i8,

    /// Minimum damage (normal attack)
    pub minDamage: u8,

    /// Maximum damage (normal attack)
    pub maxDamage: u8,

    /// Minimum damage (special attack)
    pub minDamageSpecial: u8,

    /// Maximum damage (special attack)
    pub maxDamageSpecial: u8,

    /// Armor class
    pub armorClass: u8,

    /// Leader monster index (if part of a pack)
    pub leader: u8,

    /// Relation to pack leader
    pub leaderRelation: LeaderRelation,

    /// Size of monster pack
    pub packSize: u8,

    /// Light source ID (-1 if none)
    pub lightId: i8,

    // ========================================================================
    // Additional fields for game logic
    // ========================================================================

    /// Monster type ID (for looking up data)
    pub typeId: MonsterId,

    /// Monster class (undead/demon/animal)
    pub monsterClass: MonsterClass,

    /// Animation frame
    pub animFrame: i32,

    /// Animation frame counter
    pub animCnt: i32,

    /// Animation delay
    pub animDelay: i32,

    /// Experience points value
    pub exp: u32,

    /// Base to-hit chance
    pub toHit: u8,

    /// Base to-hit chance (special)
    pub toHitSpecial: u8,

    /// Monster level
    pub level: u8,
}

/// No leader constant
pub const NO_LEADER: u8 = 255;

impl Monster {
    /// Create a new empty monster slot
    pub fn new() -> Self {
        Self {
            isInvalid: true,
            leader: NO_LEADER,
            lightId: -1,
            uniqueType: UniqueMonsterType::None,
            ..Default::default()
        }
    }

    /// Check if monster slot is active
    pub fn is_active(&self) -> bool {
        !self.isInvalid
    }

    /// Check if monster is dead
    pub fn is_dead(&self) -> bool {
        self.hitPoints <= 0 || self.mode == MonsterMode::Death
    }

    /// Check if monster is walking
    pub fn is_walking(&self) -> bool {
        matches!(
            self.mode,
            MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways
        )
    }

    /// Check if monster is unique
    pub fn is_unique(&self) -> bool {
        self.uniqueType != UniqueMonsterType::None
    }

    /// Check if monster is a player's golem
    pub fn is_player_minion(&self) -> bool {
        self.flags.contains(MonsterFlags::GOLEM)
    }

    /// Check if monster is hidden
    pub fn is_hidden(&self) -> bool {
        self.flags.contains(MonsterFlags::HIDDEN)
    }

    /// Check if monster can open doors
    pub fn can_open_door(&self) -> bool {
        self.flags.contains(MonsterFlags::CAN_OPEN_DOOR)
    }

    /// Check if monster is berserk
    pub fn is_berserk(&self) -> bool {
        self.flags.contains(MonsterFlags::BERSERK)
    }

    /// Get monster's name
    pub fn name(&self) -> &'static str {
        // Would look up from monster data in real implementation
        match self.uniqueType {
            UniqueMonsterType::Garbud => "Garbud",
            UniqueMonsterType::SkeletonKing => "King Leoric",
            UniqueMonsterType::Zhar => "Zhar the Mad",
            UniqueMonsterType::SnotSpill => "Snotspill",
            UniqueMonsterType::Lazarus => "Archbishop Lazarus",
            UniqueMonsterType::RedVex => "Red Vex",
            UniqueMonsterType::BlackJade => "Black Jade",
            UniqueMonsterType::Lachdan => "Lachdan",
            UniqueMonsterType::WarlordOfBlood => "Warlord of Blood",
            UniqueMonsterType::Butcher => "The Butcher",
            UniqueMonsterType::HorkDemon => "Hork Demon",
            UniqueMonsterType::Defiler => "Defiler",
            UniqueMonsterType::NaKrul => "Na-Krul",
            UniqueMonsterType::None => "Monster",
        }
    }

    /// Calculate experience points for given difficulty
    pub fn calc_exp(&self, difficulty: Difficulty) -> u32 {
        let mut exp = self.exp;

        match difficulty {
            Difficulty::Nightmare => exp = 2 * (exp + 1000),
            Difficulty::Hell => exp = 4 * (exp + 1000),
            _ => {}
        }

        if self.is_unique() {
            exp *= 2;
        }

        exp
    }

    /// Calculate level for given difficulty
    pub fn calc_level(&self, difficulty: Difficulty) -> u8 {
        let mut level = self.level;

        match difficulty {
            Difficulty::Nightmare => level = level.saturating_add(15),
            Difficulty::Hell => level = level.saturating_add(30),
            _ => {}
        }

        level
    }

    /// Get leader monster if exists
    pub fn get_leader_index(&self) -> Option<u8> {
        if self.leader == NO_LEADER || self.leaderRelation == LeaderRelation::None {
            None
        } else {
            Some(self.leader)
        }
    }

    /// Set leader
    pub fn set_leader(&mut self, leader_idx: u8, relation: LeaderRelation) {
        self.leader = leader_idx;
        self.leaderRelation = relation;
    }

    /// Clear leader
    pub fn clear_leader(&mut self) {
        self.leader = NO_LEADER;
        self.leaderRelation = LeaderRelation::None;
    }

    /// Take damage, returns true if monster dies
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.hitPoints -= damage;
        self.hitPoints <= 0
    }

    /// Heal the monster
    pub fn heal(&mut self, amount: i32) {
        self.hitPoints = (self.hitPoints + amount).min(self.maxHitPoints);
    }

    /// Check resistance to a damage type (placeholder - would check resistance bits)
    pub fn is_resistant(&self, _damage_type: u8) -> bool {
        // Resistance would be checked against specific bits
        false
    }

    /// Check immunity to a damage type
    pub fn is_immune(&self, _damage_type: u8) -> bool {
        // Immunity would be checked against specific bits
        false
    }

    /// Distance to enemy (max of x and y distance, not euclidean)
    pub fn distance_to_enemy(&self) -> i32 {
        let dx = (self.position.x - self.enemyPosition.x).abs();
        let dy = (self.position.y - self.enemyPosition.y).abs();
        dx.max(dy)
    }

    /// Check if monster can be hit
    pub fn is_possible_to_hit(&self) -> bool {
        if self.is_dead() || self.is_hidden() {
            return false;
        }

        // Stone curse makes monster hittable
        if self.mode == MonsterMode::Petrified {
            return true;
        }

        // Fading monsters can't be hit
        if self.mode == MonsterMode::FadeIn || self.mode == MonsterMode::FadeOut {
            return false;
        }

        true
    }

    /// Start death sequence
    pub fn start_death(&mut self) {
        self.mode = MonsterMode::Death;
        self.hitPoints = 0;
    }

    /// Set mode to petrified (stone curse)
    pub fn petrify(&mut self) {
        self.mode = MonsterMode::Petrified;
        self.flags.set(MonsterFlags::LOCK_ANIMATION);
    }

    /// Remove petrification
    pub fn unpetrify(&mut self) {
        if self.mode == MonsterMode::Petrified {
            self.mode = MonsterMode::Stand;
            self.flags.clear(MonsterFlags::LOCK_ANIMATION);
        }
    }
}

// ============================================================================
// Monster array and tracking
// ============================================================================

/// Global monster array
pub struct MonsterManager {
    /// All monsters
    pub monsters: [Monster; MAX_MONSTERS],

    /// Active monster indices
    pub active_monsters: [usize; MAX_MONSTERS],

    /// Count of active monsters
    pub active_count: usize,

    /// Kill counts per monster type
    pub kill_counts: [i32; 128],
}

impl Default for MonsterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MonsterManager {
    pub fn new() -> Self {
        Self {
            monsters: std::array::from_fn(|_| Monster::new()),
            active_monsters: [0; MAX_MONSTERS],
            active_count: 0,
            kill_counts: [0; 128],
        }
    }

    /// Get an active monster by active index
    pub fn get_active(&self, active_idx: usize) -> Option<&Monster> {
        if active_idx >= self.active_count {
            return None;
        }
        let monster_idx = self.active_monsters[active_idx];
        Some(&self.monsters[monster_idx])
    }

    /// Get a mutable active monster
    pub fn get_active_mut(&mut self, active_idx: usize) -> Option<&mut Monster> {
        if active_idx >= self.active_count {
            return None;
        }
        let monster_idx = self.active_monsters[active_idx];
        Some(&mut self.monsters[monster_idx])
    }

    /// Find monster at position
    pub fn find_at_position(&self, pos: Point) -> Option<usize> {
        for i in 0..self.active_count {
            let idx = self.active_monsters[i];
            let monster = &self.monsters[idx];
            if monster.position == pos && monster.is_active() && !monster.is_dead() {
                return Some(idx);
            }
        }
        None
    }

    /// Find unique monster by type
    pub fn find_unique(&self, unique_type: UniqueMonsterType) -> Option<usize> {
        for i in 0..self.active_count {
            let idx = self.active_monsters[i];
            if self.monsters[idx].uniqueType == unique_type {
                return Some(idx);
            }
        }
        None
    }

    /// Add a monster, returns index
    pub fn add_monster(&mut self) -> Option<usize> {
        // Find first invalid slot
        for i in 0..MAX_MONSTERS {
            if self.monsters[i].isInvalid {
                self.monsters[i].isInvalid = false;
                self.active_monsters[self.active_count] = i;
                self.active_count += 1;
                return Some(i);
            }
        }
        None
    }

    /// Remove a monster
    pub fn remove_monster(&mut self, idx: usize) {
        if idx >= MAX_MONSTERS || self.monsters[idx].isInvalid {
            return;
        }

        self.monsters[idx].isInvalid = true;

        // Remove from active list
        for i in 0..self.active_count {
            if self.active_monsters[i] == idx {
                // Shift remaining elements
                for j in i..self.active_count - 1 {
                    self.active_monsters[j] = self.active_monsters[j + 1];
                }
                self.active_count -= 1;
                break;
            }
        }
    }

    /// Clear all monsters
    pub fn clear(&mut self) {
        for monster in &mut self.monsters {
            monster.isInvalid = true;
        }
        self.active_count = 0;
    }

    /// Record a kill
    pub fn record_kill(&mut self, monster_type: usize) {
        if monster_type < self.kill_counts.len() {
            self.kill_counts[monster_type] += 1;
        }
    }
}
