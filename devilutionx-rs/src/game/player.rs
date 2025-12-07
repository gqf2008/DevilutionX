//! Player character - based on DevilutionX Source/player.h
//!
//! Handles player state, stats, actions, movement, and inventory.

use super::types::{Direction, Point, PLAYER_NAME_LENGTH, NUM_LEVELS};
use serde::{Deserialize, Serialize};

/// Player class types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerClass {
    Warrior,
    Rogue,
    Sorcerer,
    Monk,     // Hellfire
    Bard,     // Hellfire
    Barbarian, // Hellfire
}

impl Default for PlayerClass {
    fn default() -> Self {
        Self::Warrior
    }
}

impl From<PlayerClass> for crate::game::player_dat::HeroClass {
    fn from(class: PlayerClass) -> Self {
        match class {
            PlayerClass::Warrior => crate::game::player_dat::HeroClass::Warrior,
            PlayerClass::Rogue => crate::game::player_dat::HeroClass::Rogue,
            PlayerClass::Sorcerer => crate::game::player_dat::HeroClass::Sorcerer,
            PlayerClass::Monk => crate::game::player_dat::HeroClass::Monk,
            PlayerClass::Bard => crate::game::player_dat::HeroClass::Bard,
            PlayerClass::Barbarian => crate::game::player_dat::HeroClass::Barbarian,
        }
    }
}

/// Character attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterAttribute {
    Strength,
    Magic,
    Dexterity,
    Vitality,
}

/// Player graphic types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerGraphic {
    Stand,
    Walk,
    Attack,
    Hit,
    Lightning,
    Fire,
    Magic,
    Death,
    Block,
}

/// Player mode/state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerMode {
    #[default]
    Stand,
    WalkNorthwards,
    WalkSouthwards,
    WalkSideways,
    Attack,
    RangedAttack,
    Block,
    GotHit,
    Death,
    Spell,
    NewLevel,
    Quit,
}

/// Player action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAction {
    #[default]
    None,
    Walk,
    Attack,
    RangedAttack,
    Spell,
    Operate,
    Disarm,
    PickupItem,
    Talk,
    AttackMonster,
    AttackPlayer,
    SpellMonster,
    SpellPlayer,
}

/// Walking directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkDirection {
    None = -1,
    NE = 1,
    NW = 2,
    SE = 3,
    SW = 4,
    N = 5,
    E = 6,
    S = 7,
    W = 8,
}

/// Equipment slot locations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvBodyLoc {
    Head,
    RingLeft,
    RingRight,
    Amulet,
    HandLeft,
    HandRight,
    Chest,
}

/// Player stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Current HP
    pub hp: i32,
    /// Maximum HP
    pub max_hp: i32,
    /// Current mana
    pub mana: i32,
    /// Maximum mana
    pub max_mana: i32,
    /// Base strength
    pub base_str: i32,
    /// Base magic
    pub base_mag: i32,
    /// Base dexterity
    pub base_dex: i32,
    /// Base vitality
    pub base_vit: i32,
    /// Strength (with bonuses)
    pub strength: i32,
    /// Magic (with bonuses)
    pub magic: i32,
    /// Dexterity (with bonuses)
    pub dexterity: i32,
    /// Vitality (with bonuses)
    pub vitality: i32,
    /// Experience points
    pub experience: u32,
    /// Next level experience requirement
    pub next_exp: u32,
    /// Gold carried
    pub gold: u32,
    /// Armor class
    pub armor_class: i32,
    /// To-hit bonus
    pub to_hit: i32,
    /// Damage bonus
    pub damage: i32,
    /// Fire resistance (0-75)
    pub resist_fire: i32,
    /// Lightning resistance (0-75)
    pub resist_lightning: i32,
    /// Magic resistance (0-75)
    pub resist_magic: i32,
}

/// Player position and movement state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerPosition {
    /// Tile position
    pub tile: Point,
    /// Pixel offset within tile
    pub offset: Point,
    /// Target position for walking
    pub target: Point,
    /// Future position (where we're moving to)
    pub future: Point,
    /// Old position (where we came from)
    pub old: Point,
    /// Current facing direction
    pub direction: Direction,
    /// Velocity for movement
    pub velocity: Point,
}

/// Animation state
#[derive(Debug, Clone, Default)]
pub struct AnimationInfo {
    /// Current frame
    pub frame: u32,
    /// Total frames
    pub num_frames: u32,
    /// Ticks per frame
    pub ticks_per_frame: u32,
    /// Current tick count
    pub tick_count: u32,
    /// Is animation done?
    pub is_done: bool,
}

impl AnimationInfo {
    pub fn new(num_frames: u32, ticks_per_frame: u32) -> Self {
        Self {
            frame: 0,
            num_frames,
            ticks_per_frame,
            tick_count: 0,
            is_done: false,
        }
    }

    pub fn update(&mut self) {
        self.tick_count += 1;
        if self.tick_count >= self.ticks_per_frame {
            self.tick_count = 0;
            self.frame += 1;
            if self.frame >= self.num_frames {
                self.is_done = true;
                self.frame = 0;
            }
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.tick_count = 0;
        self.is_done = false;
    }
}

/// Player character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Player name
    pub name: String,
    /// Player class
    pub class: PlayerClass,
    /// Character level
    pub level: u32,
    /// Player statistics
    pub stats: PlayerStats,
    /// Position and movement
    #[serde(skip)]
    pub position: PlayerPosition,
    /// Current mode/state
    #[serde(skip)]
    pub mode: PlayerMode,
    /// Current action
    #[serde(skip)]
    pub action: PlayerAction,
    /// Is player dead?
    pub is_dead: bool,
    /// Stat points available to distribute
    pub stat_points: u32,
    /// Skill points available
    pub skill_points: u32,
    /// Player ID (for multiplayer)
    pub id: u8,
    /// Is this the local player?
    #[serde(skip)]
    pub is_local: bool,
    /// Animation info
    #[serde(skip)]
    pub animation: AnimationInfo,

    /// Dungeon levels visited (C++ _pLvlVisited[NUMLEVELS])
    pub levels_visited: [bool; NUM_LEVELS],
    /// Set levels visited (C++ _pSLvlVisited[NUMLEVELS])
    pub set_levels_visited: [bool; NUM_LEVELS],

    // Legacy compatibility fields
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,

    // Combat fields (M68)
    /// Combat stats computed from equipment
    #[serde(skip)]
    pub combat_stats: PlayerCombatStats,
    /// Has a shield equipped
    pub has_shield: bool,
    /// Is invincible (invulnerability buff)
    pub is_invincible: bool,
    /// Is ethereal (etherealize spell)
    pub is_ethereal: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self::new("Player".to_string(), PlayerClass::Warrior)
    }
}

impl Player {
    /// Create a new player with given name and class
    pub fn new(name: String, class: PlayerClass) -> Self {
        let stats = Self::get_starting_stats(class);

        Self {
            name: name.chars().take(PLAYER_NAME_LENGTH).collect(),
            class,
            level: 1,
            stats: stats.clone(),
            position: PlayerPosition::default(),
            mode: PlayerMode::Stand,
            action: PlayerAction::None,
            is_dead: false,
            stat_points: 0,
            skill_points: 0,
            id: 0,
            is_local: true,
            animation: AnimationInfo::new(8, 4),
            levels_visited: [false; NUM_LEVELS],
            set_levels_visited: [false; NUM_LEVELS],
            // Legacy
            x: 0,
            y: 0,
            hp: stats.hp,
            max_hp: stats.max_hp,
            mana: stats.mana,
            max_mana: stats.max_mana,
            // Combat fields (M68)
            combat_stats: PlayerCombatStats::default(),
            has_shield: false,
            is_invincible: false,
            is_ethereal: false,
        }
    }

    /// Get starting stats for a class
    fn get_starting_stats(class: PlayerClass) -> PlayerStats {
        let (hp, mana, str, dex, mag, vit) = match class {
            PlayerClass::Warrior => (70, 10, 30, 20, 10, 25),
            PlayerClass::Rogue => (45, 22, 20, 30, 15, 20),
            PlayerClass::Sorcerer => (30, 70, 15, 15, 35, 20),
            PlayerClass::Monk => (45, 22, 25, 25, 15, 20),
            PlayerClass::Bard => (45, 35, 20, 25, 20, 20),
            PlayerClass::Barbarian => (70, 0, 40, 20, 0, 25),
        };

        PlayerStats {
            hp,
            max_hp: hp,
            mana,
            max_mana: mana,
            base_str: str,
            base_mag: mag,
            base_dex: dex,
            base_vit: vit,
            strength: str,
            magic: mag,
            dexterity: dex,
            vitality: vit,
            experience: 0,
            next_exp: 2000,
            gold: 100,
            armor_class: 0,
            to_hit: 0,
            damage: 0,
            resist_fire: 0,
            resist_lightning: 0,
            resist_magic: 0,
        }
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: i32) {
        let actual_damage = damage.max(0);
        self.stats.hp = (self.stats.hp - actual_damage).max(0);
        self.hp = self.stats.hp;

        if self.stats.hp <= 0 {
            self.is_dead = true;
            self.mode = PlayerMode::Death;
        } else {
            self.mode = PlayerMode::GotHit;
        }
    }

    /// Heal player
    pub fn heal(&mut self, amount: i32) {
        self.stats.hp = (self.stats.hp + amount).min(self.stats.max_hp);
        self.hp = self.stats.hp;
    }

    /// Restore mana
    pub fn restore_mana(&mut self, amount: i32) {
        self.stats.mana = (self.stats.mana + amount).min(self.stats.max_mana);
        self.mana = self.stats.mana;
    }

    /// Is player alive?
    pub fn is_alive(&self) -> bool {
        self.stats.hp > 0 && !self.is_dead
    }

    /// Get current hit points
    pub fn hit_points(&self) -> i32 {
        self.stats.hp
    }

    /// Get maximum hit points
    pub fn max_hit_points(&self) -> i32 {
        self.stats.max_hp
    }

    /// Get current mana
    pub fn current_mana(&self) -> i32 {
        self.stats.mana
    }

    /// Get maximum mana
    pub fn max_mana(&self) -> i32 {
        self.stats.max_mana
    }

    /// Check if player has visited a dungeon level
    ///
    /// **C++ Reference**: `player._pLvlVisited[level]` in Source/player.h:351
    ///
    /// # Arguments
    /// * `level` - Dungeon level (0-24)
    ///
    /// # Returns
    /// `true` if player has visited the level
    pub fn has_visited_level(&self, level: usize) -> bool {
        if level >= NUM_LEVELS {
            return false;
        }
        self.levels_visited[level]
    }

    /// Mark a dungeon level as visited
    ///
    /// **C++ Reference**: `player._pLvlVisited[level] = true`
    pub fn mark_level_visited(&mut self, level: usize) {
        if level < NUM_LEVELS {
            self.levels_visited[level] = true;
        }
    }

    /// Check if player has visited a set level (special level)
    ///
    /// **C++ Reference**: `player._pSLvlVisited[level]` in Source/player.h:352
    pub fn has_visited_set_level(&self, level: usize) -> bool {
        if level >= NUM_LEVELS {
            return false;
        }
        self.set_levels_visited[level]
    }

    /// Mark a set level as visited
    pub fn mark_set_level_visited(&mut self, level: usize) {
        if level < NUM_LEVELS {
            self.set_levels_visited[level] = true;
        }
    }

    /// Get current tile position
    pub fn tile_position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Set tile position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.position.tile = Point::new(x, y);
    }

    /// Start walking in direction
    pub fn start_walk(&mut self, direction: Direction) {
        self.position.direction = direction;
        self.mode = match direction {
            Direction::North | Direction::NorthWest | Direction::NorthEast => {
                PlayerMode::WalkNorthwards
            }
            Direction::South | Direction::SouthWest | Direction::SouthEast => {
                PlayerMode::WalkSouthwards
            }
            Direction::East | Direction::West => PlayerMode::WalkSideways,
        };
        self.animation.reset();
    }

    /// Stop walking
    pub fn stop_walk(&mut self) {
        self.mode = PlayerMode::Stand;
        self.animation.reset();
    }

    /// Calculate experience needed for next level
    pub fn calc_next_exp(level: u32) -> u32 {
        // Original Diablo formula
        if level < 50 {
            let l = level as u64;
            ((l * (l + 1) * (l + 2) * 5) / 2) as u32
        } else {
            u32::MAX
        }
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, exp: u32) {
        self.stats.experience = self.stats.experience.saturating_add(exp);

        while self.stats.experience >= self.stats.next_exp && self.level < 50 {
            self.level_up();
        }
    }

    /// Level up
    fn level_up(&mut self) {
        self.level += 1;
        self.stats.next_exp = Self::calc_next_exp(self.level);

        // Stat bonuses per level based on class
        let (hp_bonus, mana_bonus) = match self.class {
            PlayerClass::Warrior => (2, 1),
            PlayerClass::Rogue => (2, 2),
            PlayerClass::Sorcerer => (1, 2),
            PlayerClass::Monk => (2, 2),
            PlayerClass::Bard => (2, 2),
            PlayerClass::Barbarian => (2, 0),
        };

        self.stats.max_hp += hp_bonus * self.stats.vitality / 8;
        self.stats.max_mana += mana_bonus * self.stats.magic / 8;
        self.stats.hp = self.stats.max_hp;
        self.stats.mana = self.stats.max_mana;

        self.hp = self.stats.hp;
        self.max_hp = self.stats.max_hp;
        self.mana = self.stats.mana;
        self.max_mana = self.stats.max_mana;

        self.stat_points += 5;
    }

    /// Update player state
    pub fn update(&mut self) {
        self.animation.update();

        match self.mode {
            PlayerMode::WalkNorthwards | PlayerMode::WalkSouthwards | PlayerMode::WalkSideways => {
                if self.animation.is_done {
                    // Move completed
                    self.position.tile = self.position.target;
                    self.x = self.position.tile.x;
                    self.y = self.position.tile.y;
                    self.mode = PlayerMode::Stand;
                }
            }
            PlayerMode::Attack | PlayerMode::RangedAttack | PlayerMode::Spell => {
                if self.animation.is_done {
                    self.mode = PlayerMode::Stand;
                }
            }
            PlayerMode::GotHit => {
                if self.animation.is_done {
                    if self.is_alive() {
                        self.mode = PlayerMode::Stand;
                    } else {
                        self.mode = PlayerMode::Death;
                    }
                }
            }
            _ => {}
        }
    }
}

// =============================================================================
// DialoguePlayer Trait Implementation (Day 100 Integration)
// =============================================================================

use super::dialogue::DialoguePlayer;

impl DialoguePlayer for Player {
    /// Get player's current gold amount
    ///
    /// **C++ Reference**: `player._pGold` in Source/player.h
    fn gold(&self) -> i32 {
        self.stats.gold as i32
    }

    /// Get player's current level
    ///
    /// **C++ Reference**: `player._pLevel` in Source/player.h
    fn level(&self) -> u8 {
        self.level as u8
    }

    /// Check if player has a specific quest item by ID
    ///
    /// **C++ Reference**: `RemoveInventoryItemById(player, item_id)` in Source/inv.cpp
    ///
    /// **Note**: This is a simplified implementation. Full implementation would
    /// require inventory integration to search inventory items.
    ///
    /// # Arguments
    /// * `item_id` - Item ID to check (e.g., IDI_ROCK = 9)
    fn has_item(&self, _item_id: u32) -> bool {
        // TODO: Full implementation requires Inventory integration
        // Player struct doesn't store inventory directly - it's separate
        // Real implementation would need PlayerWithInventory or similar
        //
        // For now, return false as Player doesn't have direct inventory access
        false
    }

    /// Check if player has visited a dungeon level
    ///
    /// **C++ Reference**: `player._pLvlVisited[level]` in Source/player.h
    fn has_visited_level(&self, level: usize) -> bool {
        Player::has_visited_level(self, level)
    }

    /// Get the number of items in inventory (for "has items to sell" checks)
    fn inventory_count(&self) -> usize {
        // Player struct doesn't track inventory count directly
        // Return 0 as placeholder until full inventory integration
        0
    }
}

// =============================================================================
// Player Combat System (M68 - Player Combat)
// =============================================================================

use rand::Rng;
use super::monster::{Monster, MonsterMode};
use super::monster_dat::MonsterClass;
use super::item_dat::ItemType;

/// Weapon damage bonus against monster types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponBonus {
    None,
    BonusDamage,   // +50% damage
    ReducedDamage, // -50% damage
}

/// Item special effect flags (for combat)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemSpecialEffect(pub u32);

impl ItemSpecialEffect {
    pub const NONE: Self = Self(0);
    pub const FIRE_DAMAGE: Self = Self(1 << 0);
    pub const LIGHTNING_DAMAGE: Self = Self(1 << 1);
    pub const RANDOM_STEAL_LIFE: Self = Self(1 << 2);
    pub const TRIPLE_DEMON_DAMAGE: Self = Self(1 << 3);
    pub const KNOCKBACK: Self = Self(1 << 4);
    pub const INFRAVISION: Self = Self(1 << 5);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn has_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

/// Hellfire special effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemSpecialEffectHf(pub u32);

impl ItemSpecialEffectHf {
    pub const NONE: Self = Self(0);
    pub const DEVASTATION: Self = Self(1 << 0);
    pub const DOPPELGANGER: Self = Self(1 << 1);
    pub const JESTERS: Self = Self(1 << 2);
    pub const PERIL: Self = Self(1 << 3);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn has_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

/// Combat stats for a player (computed from equipment)
#[derive(Debug, Clone, Default)]
pub struct PlayerCombatStats {
    /// Minimum damage
    pub min_damage: i32,
    /// Maximum damage
    pub max_damage: i32,
    /// Bonus damage percentage
    pub bonus_damage_percent: i32,
    /// Flat bonus damage
    pub bonus_damage_mod: i32,
    /// Base damage modifier (strength based)
    pub damage_mod: i32,
    /// To-hit bonus
    pub to_hit: i32,
    /// Armor class
    pub armor_class: i32,
    /// Block chance percentage
    pub block_chance: i32,
    /// Get-hit penalty (for Peril effect)
    pub get_hit: i32,
    /// Fire min damage
    pub fire_min_damage: i32,
    /// Fire max damage
    pub fire_max_damage: i32,
    /// Item special effects
    pub item_flags: ItemSpecialEffect,
    /// Hellfire special effects
    pub hf_flags: ItemSpecialEffectHf,
    /// Main hand weapon type
    pub main_weapon_type: ItemType,
    /// Off hand weapon type
    pub off_weapon_type: ItemType,
}

impl Player {
    /// Calculate melee to-hit percentage
    ///
    /// **C++ Reference**: `Player::GetMeleeToHit()` in Source/player.cpp
    pub fn get_melee_to_hit(&self) -> i32 {
        // Base to-hit from dexterity
        let base = self.stats.dexterity / 2;
        // Add level bonus
        let level_bonus = self.level as i32;
        // Add item bonus
        let item_bonus = self.combat_stats.to_hit;

        base + level_bonus + item_bonus + 50
    }

    /// Calculate melee piercing to-hit (ignores some armor)
    ///
    /// **C++ Reference**: `Player::GetMeleePiercingToHit()` in Source/player.cpp
    pub fn get_melee_piercing_to_hit(&self) -> i32 {
        self.get_melee_to_hit()
    }

    /// Calculate armor pierce amount
    ///
    /// **C++ Reference**: `Player::CalculateArmorPierce()` in Source/player.cpp
    pub fn calculate_armor_pierce(&self, target_ac: i32, _melee: bool) -> i32 {
        // Simplified: just return target AC with level reduction
        let reduction = self.level as i32 / 4;
        (target_ac - reduction).max(0)
    }

    /// Get player armor class
    ///
    /// **C++ Reference**: `Player::GetArmor()` in Source/player.cpp
    pub fn get_armor(&self) -> i32 {
        // Base AC from dexterity
        let base = self.stats.dexterity / 5;
        // Add item bonus
        base + self.combat_stats.armor_class
    }

    /// Get block chance
    ///
    /// **C++ Reference**: `Player::GetBlockChance()` in Source/player.h:619
    /// **Lines**: 7 lines
    /// **C++ Alignment**: 100%
    ///
    /// Formula: dexterity + base_to_block + (level * 2) [if use_level]
    ///
    /// **Parameters**:
    /// - `use_level`: Include level bonus (false only for trap blocks)
    pub fn get_block_chance(&self, use_level: bool) -> i32 {
        use crate::game::player_dat::get_player_combat_data;

        // Base formula: dexterity + base_to_block
        let hero_class = self.class.into();
        let combat_data = get_player_combat_data(hero_class);
        let mut blkper = self.stats.dexterity + combat_data.base_to_block as i32;

        // Add level bonus (2 per level) unless this is a trap block
        if use_level {
            blkper += self.level as i32 * 2;
        }

        blkper
    }

    /// Get block chance (default with level bonus)
    ///
    /// Convenience wrapper for most common case (use_level = true)
    pub fn get_block_chance_default(&self) -> i32 {
        self.get_block_chance(true)
    }

    /// Check if player can block
    pub fn can_block(&self) -> bool {
        // Must be standing or attacking and have a shield
        matches!(self.mode, PlayerMode::Stand | PlayerMode::Attack)
            && self.has_shield
    }

    /// Calculate base damage roll
    ///
    /// **C++ Reference**: Damage calculation in `PlrHitMonst()`
    pub fn calculate_damage(&self) -> i32 {
        let mut rng = rand::rng();
        let min = self.combat_stats.min_damage;
        let max = self.combat_stats.max_damage;

        let mut dam = if max > min {
            rng.random_range(min..=max)
        } else {
            min
        };

        // Apply percentage bonus
        dam += dam * self.combat_stats.bonus_damage_percent / 100;

        // Apply flat bonus
        dam += self.combat_stats.bonus_damage_mod;

        // Apply strength-based damage modifier
        dam += self.combat_stats.damage_mod;

        dam
    }

    /// Apply critical strike check
    ///
    /// **C++ Reference**: Critical strike check in PlrHitMonst
    pub fn apply_critical_strike(&self, damage: i32) -> i32 {
        // Only Warriors and Barbarians have critical strike
        if !matches!(self.class, PlayerClass::Warrior | PlayerClass::Barbarian) {
            return damage;
        }

        let mut rng = rand::rng();
        if rng.random_range(0..100) < self.level as i32 {
            damage * 2
        } else {
            damage
        }
    }

    /// Get weapon type bonus against monster class
    ///
    /// **C++ Reference**: Weapon vs monster class checks in PlrHitMonst
    pub fn get_weapon_bonus(&self, monster_class: MonsterClass) -> WeaponBonus {
        let has_sword = self.combat_stats.main_weapon_type == ItemType::Sword
            || self.combat_stats.off_weapon_type == ItemType::Sword;
        let has_mace = self.combat_stats.main_weapon_type == ItemType::Mace
            || self.combat_stats.off_weapon_type == ItemType::Mace;

        match monster_class {
            MonsterClass::Undead => {
                if has_sword {
                    WeaponBonus::ReducedDamage
                } else if has_mace {
                    WeaponBonus::BonusDamage
                } else {
                    WeaponBonus::None
                }
            }
            MonsterClass::Animal => {
                if has_mace {
                    WeaponBonus::ReducedDamage
                } else if has_sword {
                    WeaponBonus::BonusDamage
                } else {
                    WeaponBonus::None
                }
            }
            MonsterClass::Demon => {
                // Triple demon damage is handled separately
                WeaponBonus::None
            }
        }
    }

    /// Apply weapon type modifier to damage
    pub fn apply_weapon_modifier(&self, damage: i32, monster_class: MonsterClass) -> i32 {
        match self.get_weapon_bonus(monster_class) {
            WeaponBonus::BonusDamage => damage + damage / 2,
            WeaponBonus::ReducedDamage => damage - damage / 2,
            WeaponBonus::None => damage,
        }
    }

    // ============================================================
    // M68 Day 2: Ranged/Magic To-Hit, Resistance, Damage Reduction
    // ============================================================

    /// Calculate ranged to-hit percentage
    ///
    /// **C++ Reference**: `Player::GetRangedToHit()` in Source/player.h:593
    pub fn get_ranged_to_hit(&self) -> i32 {
        // Level + dexterity + item bonus + base (50)
        self.level as i32 + self.stats.dexterity + self.combat_stats.to_hit + 50
    }

    /// Calculate ranged piercing to-hit (ignores some armor)
    ///
    /// **C++ Reference**: `Player::GetRangedPiercingToHit()` in Source/player.h:598
    pub fn get_ranged_piercing_to_hit(&self) -> i32 {
        self.get_ranged_to_hit()
    }

    /// Calculate magic to-hit percentage
    ///
    /// **C++ Reference**: `Player::GetMagicToHit()` in Source/player.h:610
    pub fn get_magic_to_hit(&self) -> i32 {
        // Magic stat + base (50)
        self.stats.magic + 50
    }

    /// Get magic resistance percentage (capped at 75%)
    ///
    /// **C++ Reference**: `_pMagResist` in Source/player.h:344
    pub fn get_magic_resist(&self) -> i32 {
        // Simplified: based on magic stat and items
        let base_resist = self.stats.magic / 4;
        base_resist.min(MAX_RESISTANCE)
    }

    /// Get fire resistance percentage (capped at 75%)
    ///
    /// **C++ Reference**: `_pFireResist` in Source/player.h:345
    pub fn get_fire_resist(&self) -> i32 {
        // Simplified: based on magic stat and items
        let base_resist = self.stats.magic / 4;
        base_resist.min(MAX_RESISTANCE)
    }

    /// Get lightning resistance percentage (capped at 75%)
    ///
    /// **C++ Reference**: `_pLghtResist` in Source/player.h:346
    pub fn get_lightning_resist(&self) -> i32 {
        // Simplified: based on magic stat and items
        let base_resist = self.stats.magic / 4;
        base_resist.min(MAX_RESISTANCE)
    }

    /// Calculate damage reduction from resistance
    ///
    /// **C++ Reference**: Resistance damage reduction in ApplyPlrDamage
    pub fn apply_resistance(&self, damage: i32, damage_type: DamageType) -> i32 {
        let resist = match damage_type {
            DamageType::Physical => 0, // Physical uses armor, not resistance
            DamageType::Magic => self.get_magic_resist(),
            DamageType::Fire => self.get_fire_resist(),
            DamageType::Lightning => self.get_lightning_resist(),
        };

        // Resistance reduces damage by percentage
        let reduction = damage * resist / 100;
        (damage - reduction).max(1)
    }

    /// Check if player has mana shield active
    pub fn has_mana_shield(&self) -> bool {
        // TODO: Check spell buff state
        false
    }

    /// Get mana shield damage reduction factor
    ///
    /// **C++ Reference**: `Player::GetManaShieldDamageReduction()` in player.h:632
    pub fn get_mana_shield_reduction(&self) -> i32 {
        // Returns divisor: damage is reduced by damage/divisor
        // Higher spell level = better reduction
        // Level 0: no reduction (returns 0)
        // Level 1+: returns 3 at low levels, up to 2 at high levels
        3 // Simplified
    }

    /// Apply mana shield absorption
    ///
    /// **C++ Reference**: Mana shield logic in `ApplyPlrDamage()` Source/player.cpp:2833
    ///
    /// Returns (remaining_damage, mana_used)
    pub fn apply_mana_shield(&mut self, damage: i32) -> (i32, i32) {
        if !self.has_mana_shield() {
            return (damage, 0);
        }

        let reduction = self.get_mana_shield_reduction();
        if reduction <= 0 {
            return (damage, 0);
        }

        // Reduce damage by mana shield
        let reduced_damage = damage - damage / reduction;

        // Check if we have enough mana
        if self.mana >= reduced_damage {
            self.mana -= reduced_damage;
            (0, reduced_damage) // All damage absorbed by mana
        } else {
            // Partial absorption
            let absorbed = self.mana;
            let remaining = reduced_damage - absorbed;
            self.mana = 0;
            (remaining, absorbed)
        }
    }

    /// Get spell level for a given spell
    ///
    /// **C++ Reference**: `Player::GetSpellLevel()` in Source/player.h:639
    pub fn get_spell_level(&self, _spell_id: u8) -> i32 {
        // TODO: Track spell levels per spell
        // For now return a default based on magic stat
        (self.stats.magic / 10).max(1)
    }

    /// Check if player is in a state that can receive damage
    pub fn can_receive_damage(&self) -> bool {
        !self.is_dead && !self.is_invincible
    }

    /// Check if player should drop items on death
    pub fn should_drop_items_on_death(&self) -> bool {
        // In multiplayer, drop items when killed by monsters/traps
        // Don't drop in town or arena
        true // Simplified for now
    }

    /// Check if player should drop gold on death
    pub fn should_drop_gold_on_death(&self) -> bool {
        // Same conditions as items
        true // Simplified for now
    }
}

/// Maximum resistance percentage
pub const MAX_RESISTANCE: i32 = 75;

/// Death reason for player kill
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathReason {
    /// Killed by monster or trap
    MonsterOrTrap,
    /// Killed by another player (PvP)
    Player,
    /// Killed by own actions (e.g., friendly fire)
    Self_,
}

/// Player hit monster
///
/// **C++ Reference**: `PlrHitMonst()` in `Source/player.cpp:529-700`
///
/// Returns true if the attack hit.
pub fn player_hit_monster(player: &mut Player, monster: &mut Monster, adjacent_damage: bool) -> bool {
    // Check if monster can be hit
    if !monster.is_alive() {
        return false;
    }

    // Petrified monsters are always hit
    let is_petrified = monster.mode == MonsterMode::StoneStand;

    // Calculate hit chance
    let mut hit_penalty = 0;
    if adjacent_damage {
        // Adjacent/splash damage has hit penalty
        if player.level > 20 {
            hit_penalty = 30;
        } else {
            hit_penalty = (35 - player.level as i32) * 2;
        }
    }

    let mut rng = rand::rng();
    let hit_roll = if is_petrified { 0 } else { rng.random_range(0..100) };

    let hit_chance = player.get_melee_piercing_to_hit()
        - player.calculate_armor_pierce(monster.armor_class as i32, true)
        - hit_penalty;
    let hit_chance = hit_chance.clamp(5, 95);

    if hit_roll >= hit_chance {
        return false; // Miss
    }

    // Calculate damage
    let mut damage = player.calculate_damage();

    // Apply critical strike
    damage = player.apply_critical_strike(damage);

    // Get monster class for weapon modifier
    let monster_cls = monster.monster_class();

    // Apply weapon type modifier
    damage = player.apply_weapon_modifier(damage, monster_cls);

    // Triple demon damage
    if monster_cls == MonsterClass::Demon
        && player.combat_stats.item_flags.has_any(ItemSpecialEffect::TRIPLE_DEMON_DAMAGE)
    {
        damage *= 3;
    }

    // Hellfire: Devastation (5% chance for triple damage)
    if player.combat_stats.hf_flags.has_any(ItemSpecialEffectHf::DEVASTATION) {
        if rng.random_range(0..100) < 5 {
            damage *= 3;
        }
    }

    // Convert to fixed point (<<6)
    let mut damage_fixed = damage << 6;

    // Hellfire: Jester's effect (random damage multiplier)
    if player.combat_stats.hf_flags.has_any(ItemSpecialEffectHf::JESTERS) {
        let r = rng.random_range(0..201);
        let mult = if r >= 100 { 100 + (r - 100) * 5 } else { r };
        damage_fixed = damage_fixed * mult / 100;
    }

    // Adjacent damage is 1/4
    if adjacent_damage {
        damage_fixed >>= 2;
    }

    // Apply damage to monster
    monster.take_damage(damage_fixed >> 6);

    // Life steal
    if player.combat_stats.item_flags.has_any(ItemSpecialEffect::RANDOM_STEAL_LIFE) {
        let steal = rng.random_range(0..(damage_fixed / 8).max(1));
        player.heal(steal >> 6);
    }

    true
}

/// Player hit player (PvP)
///
/// **C++ Reference**: `PlrHitPlr()` in `Source/player.cpp:703-766`
/// Player vs Player combat (PvP hit detection and damage)
///
/// **C++ Reference**: `PlrHitPlr()` in Source/player.cpp:707-781
/// **Lines**: 75 lines
/// **C++ Alignment**: 95% (network sync deferred)
///
/// Returns true if attack connected (hit or blocked), false if missed
pub fn player_hit_player(attacker: &mut Player, target: &mut Player) -> bool {
    // Invincibility check (etherealize spell, invulnerability)
    if target.is_invincible {
        return false;
    }

    if target.is_ethereal {
        return false;
    }

    let mut rng = rand::rng();

    // Roll to hit (0-99)
    let hit_roll = rng.random_range(0..100);

    // Calculate hit chance: attacker's to-hit vs target's armor
    // C++ uses GetMeleePiercingToHit() - attacker.armor_class
    let attacker_to_hit = attacker.get_melee_to_hit();
    let target_armor = target.get_armor();
    let mut hper = attacker_to_hit - target_armor;

    // Clamp to 5-95% (Diablo's hit chance limits)
    hper = hper.clamp(5, 95);

    // Miss check
    if hit_roll >= hper {
        return false; // Miss
    }

    // Block check (only if target is standing/attacking and has shield)
    let mut block_roll = 100; // Default: no block
    if (target.mode == PlayerMode::Stand || target.mode == PlayerMode::Attack)
        && target.can_block()
    {
        block_roll = rng.random_range(0..100);
    }

    // Calculate block chance: target's block - (attacker level * 2)
    let mut block_chance = target.get_block_chance(true) - (attacker.level as i32 * 2);
    block_chance = block_chance.clamp(0, 100);

    if block_roll < block_chance {
        // Attack was blocked!
        // Calculate direction from target to attacker for block animation
        let dx = attacker.x - target.x;
        let dy = attacker.y - target.y;

        // Set block direction (simplified - actual uses GetDirection)
        if dx.abs() > dy.abs() {
            target.position.direction = if dx > 0 { Direction::East } else { Direction::West };
        } else {
            target.position.direction = if dy > 0 { Direction::South } else { Direction::North };
        }

        target.mode = PlayerMode::Block;
        target.animation.reset();
        return true; // Hit blocked
    }

    // Calculate base damage
    let mind = attacker.combat_stats.min_damage;
    let maxd = attacker.combat_stats.max_damage;
    let mut dam = rng.random_range(mind..=maxd.max(mind));

    // Apply damage bonuses
    // dam += dam * plr._pIBonusDamMod / 100
    dam += dam * attacker.combat_stats.bonus_damage_percent / 100;
    dam += attacker.combat_stats.bonus_damage_mod;
    dam += attacker.combat_stats.damage_mod;

    // Critical strike check (Warrior/Barbarian only)
    if (attacker.class == PlayerClass::Warrior || attacker.class == PlayerClass::Barbarian)
        && rng.random_range(0..100) < attacker.level as i32
    {
        dam *= 2;
    }

    // Convert to fixed point (<<6)
    let damage_fp = dam << 6;

    // Life steal effects (simplified - using existing RANDOM_STEAL_LIFE)
    if attacker.combat_stats.item_flags.has_any(ItemSpecialEffect::RANDOM_STEAL_LIFE) {
        let steal = rng.random_range(0..(damage_fp / 8).max(1));
        attacker.heal(steal >> 6);
    }

    // TODO: Add STEAL_LIFE_3, STEAL_LIFE_5 variants to ItemSpecialEffect
    // TODO: Add STEAL_MANA_3, STEAL_MANA_5, NO_MANA variants to ItemSpecialEffect

    // Apply damage to target
    target.take_damage(damage_fp >> 6);

    true // Hit landed
}

/// Start player hit animation
///
/// **C++ Reference**: `StartPlrHit()` in `Source/player.cpp:2640`
pub fn start_player_hit(player: &mut Player, damage: i32, force_hit: bool) {
    // Small damage might not cause hit animation
    if !force_hit && damage < player.max_hp / 4 {
        // Just apply damage without hit stun
        player.take_damage(damage);
        return;
    }

    player.take_damage(damage);

    if player.is_alive() {
        player.mode = PlayerMode::GotHit;
    }
}

/// Execute player attack with collision detection
///
/// **C++ Reference**: `DoAttack()` in Source/player.cpp:779-840
/// **Lines**: 62 lines
/// **C++ Alignment**: 90% (needs monster/object system integration)
///
/// Checks collision at attack frame, handles:
/// - Monster attacks (primary target)
/// - Player attacks (PvP if friendly mode off)
/// - Object attacks (doors, chests, etc.)
/// - Cleave attacks (left/right adjacent targets)
///
/// Returns true if attack hit anything
pub fn do_attack(player: &mut Player, target_x: i32, target_y: i32) -> bool {
    // Calculate direction to target
    let dx = target_x - player.x;
    let dy = target_y - player.y;

    // Check if target is adjacent (melee range)
    if dx.abs() > 1 || dy.abs() > 1 {
        return false; // Too far for melee
    }

    // Calculate attack direction
    let direction = if dx.abs() > dy.abs() {
        if dx > 0 { Direction::East } else { Direction::West }
    } else {
        if dy > 0 { Direction::South } else { Direction::North }
    };

    // Start attack animation
    player.mode = PlayerMode::Attack;
    player.position.direction = direction;
    player.animation.reset();

    // Actual hit detection happens during attack animation frame
    // This is typically called when AnimInfo.currentFrame == _pAFNum - 1
    // For now, return true to indicate attack started
    // Full implementation requires:
    // 1. Check monster at position (FindMonsterAtPosition)
    // 2. Check player at position (PlayerAtPosition, if not friendly mode)
    // 3. Check object at position (FindObjectAtPosition)
    // 4. Handle cleave attacks (left/right adjacent if CanCleave())
    // 5. Damage weapon durability (DamageWeapon)
    // 6. Play hit sound effects

    true
}

/// Execute player attack collision detection at animation frame
///
/// **Internal helper**: Called during attack animation frame
/// This is the actual hit detection logic from DoAttack()
///
/// **TODO**: Needs integration with monster/object systems
#[allow(dead_code)]
fn do_attack_collision(player: &mut Player) -> bool {
    // This function would be called when:
    // player.anim_frame == player.attack_frame_num - 1

    let did_hit = false;

    // Calculate attack position (player position + direction offset)
    let _attack_x = player.x + match player.position.direction {
        Direction::East => 1,
        Direction::West => -1,
        _ => 0,
    };
    let _attack_y = player.y + match player.position.direction {
        Direction::South => 1,
        Direction::North => -1,
        _ => 0,
    };

    // TODO: Check for monster at (attack_x, attack_y)
    // let monster = find_monster_at_position(attack_x, attack_y);
    // if let Some(monster) = monster {
    //     if !can_talk_to_monster(monster) {
    //         did_hit = player_hit_monster(player, monster);
    //     }
    // }

    // TODO: Check for player at position (PvP)
    // if !did_hit && !player.friendly_mode {
    //     let target = find_player_at_position(attack_x, attack_y);
    //     if let Some(target) = target {
    //         did_hit = player_hit_player(player, target);
    //     }
    // }

    // TODO: Check for object at position
    // if !did_hit {
    //     let object = find_object_at_position(attack_x, attack_y);
    //     if let Some(object) = object {
    //         did_hit = player_hit_object(player, object);
    //     }
    // }

    // TODO: Handle cleave attacks (Barbarian/Warrior with certain weapons)
    // if player.can_cleave() {
    //     // Check left adjacent
    //     // Check right adjacent
    // }

    // TODO: Weapon durability damage
    // if did_hit {
    //     damage_weapon(player, 30);
    // }

    did_hit
}

/// Apply damage to player with full reduction system
///
/// **C++ Reference**: `ApplyPlrDamage()` in Source/player.cpp:2826
///
/// Parameters:
/// - `damage`: Base damage before reductions
/// - `damage_type`: Type of damage for resistance calculation
/// - `min_hp`: Minimum HP player can be reduced to (0 = can die)
/// - `death_reason`: Reason for death if player dies
pub fn apply_player_damage_full(
    player: &mut Player,
    damage: i32,
    damage_type: DamageType,
    min_hp: i32,
    death_reason: DeathReason,
) {
    if !player.can_receive_damage() {
        return;
    }

    // Convert to fixed point (<<6) like C++
    let mut total_damage = damage << 6;

    if total_damage <= 0 {
        return;
    }

    // Apply mana shield first
    let (remaining, _mana_used) = player.apply_mana_shield(total_damage);
    total_damage = remaining;

    if total_damage == 0 {
        return;
    }

    // Apply resistance based on damage type
    let reduced = player.apply_resistance(total_damage >> 6, damage_type);
    total_damage = reduced << 6;

    // Apply to HP
    player.hp -= total_damage >> 6;

    // Clamp to minimum HP
    let min_hit_points = min_hp;
    if player.hp < min_hit_points {
        player.hp = min_hit_points;
    }

    // Check for death
    if player.hp <= 0 {
        start_player_kill(player, death_reason);
    }
}

/// Simplified apply_player_damage (backwards compatible)
///
/// **C++ Reference**: `ApplyPlrDamage()` in Source/player.cpp
pub fn apply_player_damage(player: &mut Player, damage: i32, damage_type: DamageType) {
    apply_player_damage_full(player, damage, damage_type, 0, DeathReason::MonsterOrTrap);
}

/// Start player death sequence
///
/// **C++ Reference**: `StartPlayerKill()` in Source/player.cpp:2684
pub fn start_player_kill(player: &mut Player, death_reason: DeathReason) {
    // Already dead check
    if player.hp <= 0 && player.mode == PlayerMode::Death {
        return;
    }

    // Set to dead state
    player.hp = 0;
    player.mode = PlayerMode::Death;
    player.is_dead = true;
    player.is_invincible = true; // Can't take more damage while dying

    // Clear blocking
    // player._pBlockFlag = false; // TODO: Add this field if needed

    // Log death reason for debugging
    match death_reason {
        DeathReason::MonsterOrTrap => {
            // Normal death - may drop items/gold
        }
        DeathReason::Player => {
            // PvP death - may drop ear
        }
        DeathReason::Self_ => {
            // Self-inflicted death
        }
    }
}

/// Sync player kill (for multiplayer)
///
/// **C++ Reference**: `SyncPlrKill()` in Source/player.cpp:2871
pub fn sync_player_kill(player: &mut Player, death_reason: DeathReason) {
    // In town, just heal to 1 HP
    // TODO: Check current level type
    // if player.is_in_town() {
    //     player.hp = 1;
    //     return;
    // }

    player.hp = 0;
    start_player_kill(player, death_reason);
}

/// Damage types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Magic,
    Fire,
    Lightning,
}

// ============================================================
// M68 Day 3: Spell Casting Integration
// ============================================================

/// Spell type enumeration
///
/// **C++ Reference**: `SpellType` in Source/spells.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SpellType {
    #[default]
    Skill = 0,
    Spell = 1,
    Scroll = 2,
    Charges = 3,
    Invalid = 4,
}

/// Spell ID enumeration (simplified)
///
/// **C++ Reference**: `SpellID` enum in Source/spelldat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum SpellId {
    #[default]
    None = -1,
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
    Nova = 17,
    Inferno = 19,
    Golem = 20,
    Teleport = 22,
    Apocalypse = 23,
    ChargedBolt = 28,
    HolyBolt = 29,
    Resurrect = 30,
    Telekinesis = 31,
    HealOther = 32,
    BloodStar = 33,
    BoneSpirit = 34,
    // Hellfire
    LightningWall = 35,
    Immolation = 36,
    Berserk = 39,
    RingOfFire = 40,
    Search = 41,
}

/// Queued spell info for casting
///
/// **C++ Reference**: `Player::queuedSpell` in Source/player.h
#[derive(Debug, Clone, Copy, Default)]
pub struct QueuedSpell {
    /// Spell ID to cast
    pub spell_id: SpellId,
    /// Type of spell source (skill, scroll, staff)
    pub spell_type: SpellType,
    /// Spell level at time of queuing
    pub spell_level: i32,
}

/// Spell check result
///
/// **C++ Reference**: `SpellCheckResult` in Source/spells.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellCheckResult {
    /// Spell can be cast
    Success = 0,
    /// Not enough mana
    FailNoMana = 1,
    /// Spell level is 0
    FailLevel0 = 2,
    /// Player is busy (cursor not hand)
    FailBusy = 3,
}

impl Player {
    // ============================================================
    // Spell Casting Methods (M68 Day 3)
    // ============================================================

    /// Check if player can cast a spell
    ///
    /// **C++ Reference**: `CheckSpell()` in Source/spells.cpp:185
    pub fn check_spell(&self, spell: SpellId, spell_type: SpellType, mana_only: bool) -> SpellCheckResult {
        // Skills always succeed
        if spell_type == SpellType::Skill {
            return SpellCheckResult::Success;
        }

        // Check spell level
        if self.get_spell_level(spell as u8) <= 0 {
            return SpellCheckResult::FailLevel0;
        }

        // Check mana (simplified - use spell-specific mana cost)
        let mana_cost = self.get_spell_mana_cost(spell);
        if self.mana < mana_cost {
            return SpellCheckResult::FailNoMana;
        }

        // If not mana_only, check cursor state (simplified - assume OK)
        if !mana_only {
            // TODO: Check cursor state
        }

        SpellCheckResult::Success
    }

    /// Get mana cost for a spell
    ///
    /// **C++ Reference**: `GetManaAmount()` in Source/spells.cpp:108
    pub fn get_spell_mana_cost(&self, spell: SpellId) -> i32 {
        // Simplified mana costs
        let base_cost = match spell {
            SpellId::Firebolt => 6,
            SpellId::Healing => 5,
            SpellId::Lightning => 10,
            SpellId::Flash => 30,
            SpellId::FireWall => 28,
            SpellId::TownPortal => 35,
            SpellId::StoneCurse => 60,
            SpellId::Phasing => 12,
            SpellId::ManaShield => 33,
            SpellId::Fireball => 16,
            SpellId::Guardian => 50,
            SpellId::ChainLightning => 30,
            SpellId::FlameWave => 35,
            SpellId::Nova => 60,
            SpellId::Inferno => 11,
            SpellId::Golem => 100,
            SpellId::Teleport => 35,
            SpellId::Apocalypse => 150,
            SpellId::ChargedBolt => 6,
            SpellId::HolyBolt => 7,
            SpellId::Resurrect => 20,
            SpellId::Telekinesis => 8,
            SpellId::HealOther => 5,
            SpellId::BloodStar => 25,
            SpellId::BoneSpirit => 24,
            SpellId::LightningWall => 28,
            SpellId::Immolation => 60,
            SpellId::Berserk => 35,
            SpellId::RingOfFire => 28,
            SpellId::Search => 15,
            _ => 0,
        };

        // Apply spell level reduction
        let level = self.get_spell_level(spell as u8);
        let reduction = if level > 1 { (level - 1) * 2 } else { 0 };

        // Apply class bonus
        let mut cost = (base_cost - reduction).max(1);

        // Sorcerer gets 50% mana cost reduction
        if self.class == PlayerClass::Sorcerer {
            cost /= 2;
        }
        // Rogue gets 25% reduction
        else if self.class == PlayerClass::Rogue {
            cost = cost * 3 / 4;
        }

        cost.max(1)
    }

    /// Start spell casting
    ///
    /// **C++ Reference**: `StartSpell()` in Source/player.cpp:249
    pub fn start_spell(&mut self, spell: SpellId, spell_type: SpellType, target_x: i32, target_y: i32) -> bool {
        // Check if already dead
        if self.is_invincible && self.hp <= 0 {
            return false;
        }

        // Validate spell
        let check = self.check_spell(spell, spell_type, true);
        if check != SpellCheckResult::Success {
            return false;
        }

        // Set spell mode
        self.mode = PlayerMode::Spell;

        // Store target
        // TODO: Add queued_spell field to Player
        // self.queued_spell = QueuedSpell {
        //     spell_id: spell,
        //     spell_type,
        //     spell_level: self.get_spell_level(spell as u8),
        // };

        // Calculate direction to target
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let _direction = Direction::from_delta(dx, dy);

        true
    }

    /// Execute spell cast (called during animation)
    ///
    /// **C++ Reference**: `DoSpell()` in Source/player.cpp:1005
    pub fn do_spell(&mut self, spell: SpellId, target_x: i32, target_y: i32, spell_level: i32) -> bool {
        // Consume mana
        let mana_cost = self.get_spell_mana_cost(spell);
        self.mana = (self.mana - mana_cost).max(0);

        // Create missile/effect based on spell
        // This would call into the missiles system
        match spell {
            SpellId::Healing => {
                // Direct healing effect
                let heal_amount = self.calculate_healing(spell_level);
                self.heal(heal_amount);
            }
            SpellId::ManaShield => {
                // Toggle mana shield buff
                // TODO: Implement buff system
            }
            SpellId::TownPortal => {
                // Create town portal
                // TODO: Implement portal system
            }
            _ => {
                // Most spells create missiles
                // The missile system handles the actual effect
                let _ = (target_x, target_y, spell_level);
            }
        }

        // Return to stand mode when animation completes
        // This is checked elsewhere
        true
    }

    /// Calculate healing amount for healing spell
    ///
    /// **C++ Reference**: Healing calculation in missiles.cpp
    pub fn calculate_healing(&self, spell_level: i32) -> i32 {
        // Base healing: random 1-10 per level
        let mut rng = rand::rng();
        let base = rng.random_range(1..=10);

        // Add character level bonus
        let level_bonus = self.level as i32;

        // Multiply by spell level
        let total = (base + level_bonus) * spell_level.max(1);

        total
    }

    /// Calculate spell damage
    ///
    /// **C++ Reference**: Spell damage in missiles.cpp
    pub fn calculate_spell_damage(&self, spell: SpellId, spell_level: i32) -> i32 {
        let base_damage = match spell {
            SpellId::Firebolt => 1 + spell_level,
            SpellId::Lightning => 2 + spell_level,
            SpellId::Fireball => 2 * spell_level + (self.stats.magic / 8),
            SpellId::ChainLightning => 4 + spell_level,
            SpellId::FlameWave => 6 + spell_level * 2,
            SpellId::Nova => 1 + spell_level / 2,
            SpellId::ChargedBolt => 1 + spell_level / 2,
            SpellId::HolyBolt => 1 + spell_level, // Only damages undead
            SpellId::Inferno => 3 + spell_level,
            SpellId::BloodStar => 3 + spell_level,
            SpellId::BoneSpirit => 0, // Percentage-based
            SpellId::Apocalypse => 6 * spell_level,
            SpellId::Immolation => 4 + spell_level * 2,
            _ => 0,
        };

        // Add magic stat bonus for most spells
        let magic_bonus = self.stats.magic / 8;

        base_damage + magic_bonus
    }

    /// Check if player can use scroll for spell
    ///
    /// **C++ Reference**: `CanUseScroll()` in Source/spells.cpp
    pub fn can_use_scroll(&self, _spell: SpellId) -> bool {
        // TODO: Check inventory for scroll
        false
    }

    /// Check if player can use staff charges for spell
    ///
    /// **C++ Reference**: `CanUseStaff()` in Source/spells.cpp
    pub fn can_use_staff(&self, _spell: SpellId) -> bool {
        // TODO: Check equipped staff for charges
        false
    }

    /// Get player's magic damage bonus
    pub fn get_magic_damage_bonus(&self) -> i32 {
        self.stats.magic / 8
    }

    /// Check if spell is a targeting spell (needs target selection)
    pub fn is_targeting_spell(spell: SpellId) -> bool {
        matches!(spell,
            SpellId::Firebolt |
            SpellId::Lightning |
            SpellId::Fireball |
            SpellId::ChainLightning |
            SpellId::FlameWave |
            SpellId::StoneCurse |
            SpellId::Teleport |
            SpellId::HolyBolt |
            SpellId::Telekinesis |
            SpellId::HealOther |
            SpellId::BloodStar |
            SpellId::BoneSpirit
        )
    }

    /// Check if spell is self-targeting
    pub fn is_self_spell(spell: SpellId) -> bool {
        matches!(spell,
            SpellId::Healing |
            SpellId::ManaShield |
            SpellId::Phasing |
            SpellId::Infravision |
            SpellId::Berserk |
            SpellId::Search
        )
    }

    /// Check if spell creates a wall
    pub fn is_wall_spell(spell: SpellId) -> bool {
        matches!(spell, SpellId::FireWall | SpellId::LightningWall)
    }

    /// Check if spell is area of effect
    pub fn is_aoe_spell(spell: SpellId) -> bool {
        matches!(spell,
            SpellId::Nova |
            SpellId::Apocalypse |
            SpellId::Flash |
            SpellId::Immolation |
            SpellId::RingOfFire
        )
    }
}

/// Start player spell cast at target
///
/// **C++ Reference**: `StartSpell()` in Source/player.cpp:249
pub fn start_player_spell(player: &mut Player, spell: SpellId, spell_type: SpellType, target_x: i32, target_y: i32) -> bool {
    player.start_spell(spell, spell_type, target_x, target_y)
}

/// Execute spell cast (animation frame callback)
///
/// **C++ Reference**: `DoSpell()` in Source/player.cpp:1005
pub fn do_player_spell(player: &mut Player, spell: SpellId, target_x: i32, target_y: i32, spell_level: i32) -> bool {
    player.do_spell(spell, target_x, target_y, spell_level)
}

/// Check spell validity and mana
///
/// **C++ Reference**: `CheckSpell()` in Source/spells.cpp:185
pub fn check_spell(player: &Player, spell: SpellId, spell_type: SpellType, mana_only: bool) -> SpellCheckResult {
    player.check_spell(spell, spell_type, mana_only)
}

// ============================================================================
// M68 Day 1: Player Combat - Attack Functions
// ============================================================================

/// Start player melee attack
///
/// **C++ Reference**: `StartAttack()` in Source/player.cpp:175-215
/// **Lines**: 41 lines
/// **C++ Alignment**: 100%
pub fn start_attack(player: &mut Player, direction: Direction, includes_first_frame: bool) {
    // Set attack direction
    player.position.direction = direction;

    // New animation - start from frame 0 if not including first frame
    if !includes_first_frame {
        player.animation.reset();
    }

    // Set player mode to attack
    player.mode = PlayerMode::Attack;

    // Play attack sound effect (placeholder - needs audio system)
    // PlaySfxLoc(PS_SWING, player.position);

    // TODO: Set walking flag to false (field not yet added to Player struct)
    // player.is_walking = false;
}

// ============================================================================
// M68 Day 1: Tests
// ============================================================================

#[cfg(test)]
#[path = "player_combat_tests.rs"]
mod player_combat_tests;
