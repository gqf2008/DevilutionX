//! Player Structure - Exact port of DevilutionX Source/player.h Player struct
//!
//! Contains all player fields matching the C++ implementation.

use serde::{Deserialize, Serialize};
use crate::game::player_dat::{
    HeroClass, get_class_attributes, get_player_combat_data,
    PLAYER_NAME_LENGTH, INVENTORY_GRID_CELLS, MAX_BELT_ITEMS, NUM_HOTKEYS, NUM_INV_LOC,
    MAX_SPELL_LEVEL, MAX_RESISTANCE,
};
use crate::game::item_dat::{ItemSpecialEffect, ItemSpecialEffectHf, ItemMiscId};
use crate::game::spells::{SpellId, SpellType};

/// Number of dungeon levels
pub const NUM_LEVELS: usize = 25;

/// Maximum path length for player movement
pub const MAX_PATH_LENGTH_PLAYER: usize = 25;

/// Player mode/state - exact match of PLR_MODE enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerMode {
    #[default]
    Stand = 0,
    WalkNorthwards = 1,
    WalkSouthwards = 2,
    WalkSideways = 3,
    Attack = 4,
    RangedAttack = 5,
    Block = 6,
    GotHit = 7,
    Death = 8,
    Spell = 9,
    NewLevel = 10,
    Quit = 11,
}

/// Player action - exact match of action_id enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(i8)]
pub enum ActionId {
    Walk = -2,
    #[default]
    None = -1,
    Attack = 9,
    RangedAttack = 10,
    Spell = 12,
    Operate = 13,
    Disarm = 14,
    PickupItem = 15,
    PickupAutoItem = 16,
    Talk = 17,
    OperateTelekinesis = 18,
    AttackMonster = 20,
    AttackPlayer = 21,
    RangedAttackMonster = 22,
    RangedAttackPlayer = 23,
    SpellMonster = 24,
    SpellPlayer = 25,
    SpellWall = 26,
}

/// Spell flags - exact match of SpellFlag enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellFlags(pub u8);

impl SpellFlags {
    pub const NONE: Self = Self(0);
    pub const ETHEREALIZE: Self = Self(1 << 0);
    pub const RAGE_ACTIVE: Self = Self(1 << 1);
    pub const RAGE_COOLDOWN: Self = Self(1 << 2);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Direction enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

/// Player weapon graphic type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PlayerWeaponGraphic {
    #[default]
    Unarmed = 0,
    UnarmedShield = 1,
    Sword = 2,
    SwordShield = 3,
    Bow = 4,
    Axe = 5,
    Mace = 6,
    MaceShield = 7,
    Staff = 8,
}

/// Player graphic type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerGraphic {
    Stand = 0,
    Walk = 1,
    Attack = 2,
    Hit = 3,
    Lightning = 4,
    Fire = 5,
    Magic = 6,
    Death = 7,
    Block = 8,
}

/// Spell cast info - exact match of SpellCastInfo struct
#[derive(Debug, Clone, Copy, Default)]
pub struct SpellCastInfo {
    pub spell_id: SpellId,
    pub spell_type: SpellType,
    pub spell_from: i8,
    pub spell_level: i32,
}

/// Actor position - matches ActorPosition
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ActorPosition {
    /// Tile position
    pub tile_x: i32,
    pub tile_y: i32,
    /// Pixel offset within tile
    pub offset_x: i32,
    pub offset_y: i32,
    /// Old position
    pub old_x: i32,
    pub old_y: i32,
    /// Future position (target)
    pub future_x: i32,
    pub future_y: i32,
    /// Velocity
    pub velocity_x: i32,
    pub velocity_y: i32,
    /// Temporary position (for attacks/spells)
    pub temp_x: i32,
    pub temp_y: i32,
}

/// Animation info
#[derive(Debug, Clone, Default)]
pub struct AnimationInfo {
    pub current_frame: i32,
    pub number_of_frames: i32,
    pub ticks_per_frame: i32,
    pub tick_count: i32,
    pub tick_count_realized: f32,
}

/// Player structure - exact port of C++ Player struct
/// Contains all 80+ fields from player.h
#[derive(Debug, Clone)]
pub struct Player {
    // === Identity ===
    /// Player name (max 32 chars)
    pub name: [u8; PLAYER_NAME_LENGTH],
    /// Hero class
    pub class: HeroClass,
    /// Character level (1-50)
    level: u8,
    /// Player ID (for multiplayer)
    pub id: u8,
    /// Is player active?
    pub plr_active: bool,

    // === Core Stats (Base values) ===
    /// Base strength
    pub base_str: i32,
    /// Base magic
    pub base_mag: i32,
    /// Base dexterity
    pub base_dex: i32,
    /// Base vitality
    pub base_vit: i32,
    /// Stat points available
    pub stat_pts: i32,

    // === Core Stats (Current values with bonuses) ===
    /// Current strength
    pub strength: i32,
    /// Current magic
    pub magic: i32,
    /// Current dexterity
    pub dexterity: i32,
    /// Current vitality
    pub vitality: i32,

    // === Health & Mana (Fixed point - shift right 6 for actual value) ===
    /// Base HP (without vitality bonus)
    pub hp_base: i32,
    /// Maximum base HP
    pub max_hp_base: i32,
    /// Current hit points
    pub hit_points: i32,
    /// Maximum hit points
    pub max_hp: i32,
    /// HP percentage (0-80)
    pub hp_per: i32,
    /// Base mana
    pub mana_base: i32,
    /// Maximum base mana
    pub max_mana_base: i32,
    /// Current mana
    pub mana: i32,
    /// Maximum mana
    pub max_mana: i32,
    /// Mana percentage (0-80)
    pub mana_per: i32,

    // === Item Stats ===
    /// Minimum weapon damage
    pub i_min_dam: i32,
    /// Maximum weapon damage
    pub i_max_dam: i32,
    /// Item armor class
    pub i_ac: i32,
    /// Bonus damage percentage
    pub i_bonus_dam: i32,
    /// Bonus to-hit
    pub i_bonus_to_hit: i32,
    /// Bonus armor class
    pub i_bonus_ac: i32,
    /// Bonus damage flat modifier
    pub i_bonus_dam_mod: i32,
    /// Damage taken modifier
    pub i_get_hit: i32,
    /// Armor piercing value
    pub i_en_ac: i32,
    /// Fire damage min
    pub i_f_min_dam: i32,
    /// Fire damage max
    pub i_f_max_dam: i32,
    /// Lightning damage min
    pub i_l_min_dam: i32,
    /// Lightning damage max
    pub i_l_max_dam: i32,
    /// Item special effects
    pub i_flags: ItemSpecialEffect,
    /// Spell level bonus from items
    pub i_spl_lvl_add: i8,
    /// Item spells bitmask
    pub i_spells: u64,

    // === Damage Modifier ===
    /// Damage modifier from stats
    pub damage_mod: i32,

    // === Experience & Gold ===
    /// Total experience
    pub experience: u32,
    /// Gold carried
    pub gold: i32,

    // === Position & Movement ===
    /// Current position
    pub position: ActorPosition,
    /// Current facing direction
    pub dir: Direction,
    /// Walk path (max 25 steps)
    pub walk_path: [i8; MAX_PATH_LENGTH_PLAYER],

    // === State ===
    /// Current mode
    pub mode: PlayerMode,
    /// Destination action
    pub dest_action: ActionId,
    /// Action parameters
    pub dest_param1: i32,
    pub dest_param2: i32,
    pub dest_param3: i32,
    pub dest_param4: i32,

    // === Animation ===
    /// Animation info
    pub anim_info: AnimationInfo,
    /// Standing frames
    pub n_frames: i8,
    /// Walking frames
    pub w_frames: i8,
    /// Attack frames
    pub a_frames: i8,
    /// Attack action frame
    pub a_f_num: i8,
    /// Spell frames
    pub s_frames: i8,
    /// Spell action frame
    pub s_f_num: i8,
    /// Hit recovery frames
    pub h_frames: i8,
    /// Death frames
    pub d_frames: i8,
    /// Block frames
    pub b_frames: i8,
    /// Graphics number (weapon + armor bitmask)
    pub gfx_num: u8,

    // === Spell System ===
    /// Queued spell info
    pub queued_spell: SpellCastInfo,
    /// Currently executing spell
    pub executed_spell: SpellCastInfo,
    /// Ready spell
    pub r_spell: SpellId,
    /// Ready spell type
    pub r_spl_type: SpellType,
    /// Spell book spell
    pub sb_spell: SpellId,
    /// Spell levels (64 spells max)
    pub spl_lvl: [u8; 64],
    /// Learned spells bitmask
    pub mem_spells: u64,
    /// Ability spells bitmask
    pub abl_spells: u64,
    /// Scroll spells bitmask
    pub scrl_spells: u64,
    /// Spell flags
    pub spell_flags: SpellFlags,
    /// Spell hotkeys
    pub spl_hot_key: [SpellId; NUM_HOTKEYS],
    /// Spell hotkey types
    pub spl_t_hot_key: [SpellType; NUM_HOTKEYS],

    // === Combat ===
    /// Can block?
    pub block_flag: bool,
    /// Is invincible?
    pub invincible: bool,
    /// Armor class
    pub armor_class: i8,
    /// Magic resistance (0-75)
    pub mag_resist: i8,
    /// Fire resistance (0-75)
    pub fire_resist: i8,
    /// Lightning resistance (0-75)
    pub lght_resist: i8,

    // === Inventory ===
    /// Equipped items (7 slots)
    // pub inv_body: [Item; NUM_INV_LOC],
    /// Inventory items (40 slots)
    // pub inv_list: [Item; INVENTORY_GRID_CELLS],
    /// Belt items (8 slots)
    // pub spd_list: [Item; MAX_BELT_ITEMS],
    /// Item being held
    // pub hold_item: Item,
    /// Number of inventory items
    pub num_inv: i32,
    /// Inventory grid (item placement)
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],

    // === Level & Dungeon ===
    /// Current dungeon level
    pub plr_level: u8,
    /// Is on set level?
    pub plr_is_on_set_level: bool,
    /// Levels visited
    pub lvl_visited: [bool; NUM_LEVELS],
    /// Set levels visited
    pub s_lvl_visited: [bool; NUM_LEVELS],
    /// Is changing level?
    pub lvl_changing: bool,
    /// Light ID
    pub light_id: i32,
    /// Light radius
    pub light_rad: i8,
    /// Infrared vision
    pub infra_flag: bool,

    // === Multiplayer ===
    /// Friendly mode (non-PvP)
    pub friendly_mode: bool,

    // === Misc ===
    /// Oil type applied
    pub oil_type: ItemMiscId,
    /// Town warps unlocked
    pub town_warps: u8,
    /// Dungeon messages seen
    pub dung_msgs: u8,
    /// Level load state
    pub lvl_load: u8,
    /// Mana shield active?
    pub mana_shield: bool,
    /// Original cathedral flag
    pub original_cathedral: bool,
    /// Diablo kill level
    pub diablo_kill_level: u8,
    /// Reflections (for reflect damage)
    pub w_reflections: u16,
    /// Hellfire damage/AC flags
    pub dam_ac_flags: ItemSpecialEffectHf,
}

impl Default for Player {
    fn default() -> Self {
        Self::new("Player", HeroClass::Warrior)
    }
}

impl Player {
    /// Create a new player with given name and class
    pub fn new(name: &str, class: HeroClass) -> Self {
        let attrs = get_class_attributes(class);

        // Calculate starting HP and Mana
        // HP = adjLife + chrLife * baseVit
        // Mana = adjMana + chrMana * baseMag
        let hp = (attrs.adj_life as i32) + (attrs.chr_life as i32 * attrs.base_vit as i32) / 64;
        let mana = (attrs.adj_mana as i32) + (attrs.chr_mana as i32 * attrs.base_mag as i32) / 64;

        // Copy name into fixed array
        let mut name_arr = [0u8; PLAYER_NAME_LENGTH];
        for (i, b) in name.bytes().take(PLAYER_NAME_LENGTH - 1).enumerate() {
            name_arr[i] = b;
        }

        Self {
            name: name_arr,
            class,
            level: 1,
            id: 0,
            plr_active: true,

            // Base stats from class
            base_str: attrs.base_str as i32,
            base_mag: attrs.base_mag as i32,
            base_dex: attrs.base_dex as i32,
            base_vit: attrs.base_vit as i32,
            stat_pts: 0,

            // Current stats (same as base initially)
            strength: attrs.base_str as i32,
            magic: attrs.base_mag as i32,
            dexterity: attrs.base_dex as i32,
            vitality: attrs.base_vit as i32,

            // HP/Mana (fixed point)
            hp_base: hp,
            max_hp_base: hp,
            hit_points: hp,
            max_hp: hp,
            hp_per: 80,
            mana_base: mana,
            max_mana_base: mana,
            mana: mana,
            max_mana: mana,
            mana_per: 80,

            // Item stats (initially 0/default)
            i_min_dam: 1,
            i_max_dam: 1,
            i_ac: 0,
            i_bonus_dam: 0,
            i_bonus_to_hit: 0,
            i_bonus_ac: 0,
            i_bonus_dam_mod: 0,
            i_get_hit: 0,
            i_en_ac: 0,
            i_f_min_dam: 0,
            i_f_max_dam: 0,
            i_l_min_dam: 0,
            i_l_max_dam: 0,
            i_flags: ItemSpecialEffect::NONE,
            i_spl_lvl_add: 0,
            i_spells: 0,

            damage_mod: 0,
            experience: 0,
            gold: 100,

            position: ActorPosition::default(),
            dir: Direction::South,
            walk_path: [0; MAX_PATH_LENGTH_PLAYER],

            mode: PlayerMode::Stand,
            dest_action: ActionId::None,
            dest_param1: 0,
            dest_param2: 0,
            dest_param3: 0,
            dest_param4: 0,

            anim_info: AnimationInfo::default(),
            n_frames: 8,
            w_frames: 8,
            a_frames: 16,
            a_f_num: 9,
            s_frames: 16,
            s_f_num: 12,
            h_frames: 8,
            d_frames: 20,
            b_frames: 6,
            gfx_num: 0,

            queued_spell: SpellCastInfo::default(),
            executed_spell: SpellCastInfo::default(),
            r_spell: SpellId::Null,
            r_spl_type: SpellType::Invalid,
            sb_spell: SpellId::Null,
            spl_lvl: [0; 64],
            mem_spells: 0,
            abl_spells: 0,
            scrl_spells: 0,
            spell_flags: SpellFlags::NONE,
            spl_hot_key: [SpellId::Null; NUM_HOTKEYS],
            spl_t_hot_key: [SpellType::Invalid; NUM_HOTKEYS],

            block_flag: false,
            invincible: false,
            armor_class: 0,
            mag_resist: 0,
            fire_resist: 0,
            lght_resist: 0,

            num_inv: 0,
            inv_grid: [0; INVENTORY_GRID_CELLS],

            plr_level: 0,
            plr_is_on_set_level: false,
            lvl_visited: [false; NUM_LEVELS],
            s_lvl_visited: [false; NUM_LEVELS],
            lvl_changing: false,
            light_id: -1,
            light_rad: 10,
            infra_flag: false,

            friendly_mode: true,

            oil_type: ItemMiscId::None,
            town_warps: 0,
            dung_msgs: 0,
            lvl_load: 0,
            mana_shield: false,
            original_cathedral: false,
            diablo_kill_level: 0,
            w_reflections: 0,
            dam_ac_flags: ItemSpecialEffectHf::NONE,
        }
    }

    /// Get character level
    pub fn get_character_level(&self) -> u8 {
        self.level
    }

    /// Set character level (clamped to valid range)
    pub fn set_character_level(&mut self, level: u8) {
        self.level = level.clamp(1, 50);
    }

    /// Get player name as string
    pub fn get_name(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(PLAYER_NAME_LENGTH);
        std::str::from_utf8(&self.name[..len]).unwrap_or("Unknown")
    }

    /// Get melee to-hit value
    /// From Player::GetMeleeToHit() in player.h
    pub fn get_melee_to_hit(&self) -> i32 {
        let combat_data = get_player_combat_data(self.class);
        self.level as i32 + self.dexterity / 2 + self.i_bonus_to_hit + combat_data.base_melee_to_hit as i32
    }

    /// Get melee to-hit with armor piercing
    pub fn get_melee_piercing_to_hit(&self, is_hellfire: bool) -> i32 {
        let mut hper = self.get_melee_to_hit();
        if !is_hellfire {
            hper += self.i_en_ac;
        }
        hper
    }

    /// Get ranged to-hit value
    pub fn get_ranged_to_hit(&self) -> i32 {
        let combat_data = get_player_combat_data(self.class);
        self.level as i32 + self.dexterity + self.i_bonus_to_hit + combat_data.base_ranged_to_hit as i32
    }

    /// Get magic to-hit value
    pub fn get_magic_to_hit(&self) -> i32 {
        let combat_data = get_player_combat_data(self.class);
        self.magic + combat_data.base_magic_to_hit as i32
    }

    /// Get armor value
    pub fn get_armor(&self) -> i32 {
        self.i_bonus_ac + self.i_ac + self.dexterity / 5
    }

    /// Get block chance
    pub fn get_block_chance(&self, use_level: bool) -> i32 {
        let combat_data = get_player_combat_data(self.class);
        let mut blk = self.dexterity + combat_data.base_to_block as i32;
        if use_level {
            blk += self.level as i32 * 2;
        }
        blk
    }

    /// Calculate armor pierce against monster
    pub fn calculate_armor_pierce(&self, monster_armor: i32, is_melee: bool, is_hellfire: bool) -> i32 {
        let mut tmac = monster_armor;
        if self.i_en_ac > 0 {
            if is_hellfire {
                let pierce = self.i_en_ac - 1;
                if pierce > 0 {
                    tmac >>= pierce;
                } else {
                    tmac -= tmac / 4;
                }
            }
            if is_melee && self.class == HeroClass::Barbarian {
                tmac -= monster_armor / 8;
            }
        }
        tmac.max(0)
    }

    /// Apply damage to player
    pub fn apply_damage(&mut self, damage: i32) {
        self.hit_points = (self.hit_points - damage).max(0);
        self.hp_base = (self.hp_base - damage).max(0);
        self.update_hp_percentage();
    }

    /// Heal player
    pub fn heal(&mut self, amount: i32) {
        self.hit_points = (self.hit_points + amount).min(self.max_hp);
        self.hp_base = (self.hp_base + amount).min(self.max_hp_base);
        self.update_hp_percentage();
    }

    /// Restore mana
    pub fn restore_mana(&mut self, amount: i32) {
        if !self.i_flags.contains(ItemSpecialEffect::NO_MANA) {
            self.mana = (self.mana + amount).min(self.max_mana);
            self.mana_base = (self.mana_base + amount).min(self.max_mana_base);
            self.update_mana_percentage();
        }
    }

    /// Update HP percentage
    fn update_hp_percentage(&mut self) {
        if self.max_hp <= 0 {
            self.hp_per = 0;
        } else {
            self.hp_per = (self.hit_points * 81 / self.max_hp).clamp(0, 81);
        }
    }

    /// Update mana percentage
    fn update_mana_percentage(&mut self) {
        if self.max_mana <= 0 {
            self.mana_per = 0;
        } else {
            self.mana_per = (self.mana * 81 / self.max_mana).clamp(0, 81);
        }
    }

    /// Check if player is alive
    pub fn is_alive(&self) -> bool {
        self.hit_points > 0
    }

    /// Check if player is walking
    pub fn is_walking(&self) -> bool {
        matches!(
            self.mode,
            PlayerMode::WalkNorthwards | PlayerMode::WalkSouthwards | PlayerMode::WalkSideways
        )
    }

    /// Get actual HP (not fixed point)
    pub fn get_hp(&self) -> i32 {
        self.hit_points >> 6
    }

    /// Get actual max HP (not fixed point)
    pub fn get_max_hp(&self) -> i32 {
        self.max_hp >> 6
    }

    /// Get actual mana (not fixed point)
    pub fn get_mana(&self) -> i32 {
        self.mana >> 6
    }

    /// Get actual max mana (not fixed point)
    pub fn get_max_mana(&self) -> i32 {
        self.max_mana >> 6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warrior_starting_stats() {
        let player = Player::new("TestWarrior", HeroClass::Warrior);
        assert_eq!(player.base_str, 30);
        assert_eq!(player.base_mag, 10);
        assert_eq!(player.base_dex, 20);
        assert_eq!(player.base_vit, 25);
        assert_eq!(player.get_character_level(), 1);
    }

    #[test]
    fn test_sorcerer_starting_stats() {
        let player = Player::new("TestSorcerer", HeroClass::Sorcerer);
        assert_eq!(player.base_str, 15);
        assert_eq!(player.base_mag, 35);
        assert_eq!(player.base_dex, 15);
        assert_eq!(player.base_vit, 20);
    }

    #[test]
    fn test_melee_to_hit() {
        let player = Player::new("Test", HeroClass::Warrior);
        // Warrior base melee to-hit is 70 (assets/txtdata/classes/warrior/attributes.tsv)
        // level(1) + dex/2 (20/2=10) + i_bonus_to_hit(0) + base(70) = 81
        assert_eq!(player.get_melee_to_hit(), 81);
    }

    #[test]
    fn test_damage() {
        let mut player = Player::new("Test", HeroClass::Warrior);
        let initial_hp = player.hit_points;
        player.apply_damage(100);
        assert_eq!(player.hit_points, initial_hp - 100);
    }
}
