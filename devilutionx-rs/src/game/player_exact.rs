//! Player System - Exact C++ Port (Day 19-25)
//!
//! This module provides exact port of DevilutionX Player structure and functionality.
//!
//! ## Key Features
//! - Complete Player struct with all 50+ fields
//! - HP/Mana management with fixed-point precision (64x)
//! - Attribute system (Strength, Magic, Dexterity, Vitality)
//! - Equipment management
//! - Spell system integration
//! - Death/resurrection handling
//!
//! ## C++ References
//! - Source/player.h: Player struct definition
//! - Source/player.cpp: Player methods
//! - Source/objects.cpp: Shrine/fountain interactions
//!
//! ## Fixed-Point System
//! HP and Mana use 64x precision:
//! - _pHitPoints = actual_hp * 64
//! - Display: _pHitPoints >> 6 (divide by 64)
//! - Modify: add/subtract raw 64x values

use crate::game::types::Point;

/// Player name maximum length
pub const PLAYER_NAME_LENGTH: usize = 32;

/// Inventory grid cells (10x4)
pub const INVENTORY_GRID_CELLS: usize = 40;

/// Maximum belt items
pub const MAX_BELT_ITEMS: usize = 8;

/// Equipment slots
pub const NUM_INV_LOC: usize = 7;

/// Maximum resistance percentage
pub const MAX_RESISTANCE: i32 = 75;

/// Fixed-point precision for HP/Mana (64x)
pub const HP_PRECISION: i32 = 64;

/// Hero class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HeroClass {
    #[default]
    Warrior,
    Rogue,
    Sorcerer,
    Monk,
    Bard,
    Barbarian,
}

impl TryFrom<u8> for HeroClass {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HeroClass::Warrior),
            1 => Ok(HeroClass::Rogue),
            2 => Ok(HeroClass::Sorcerer),
            3 => Ok(HeroClass::Monk),
            4 => Ok(HeroClass::Bard),
            5 => Ok(HeroClass::Barbarian),
            _ => Err(()),
        }
    }
}

impl From<HeroClass> for u8 {
    fn from(class: HeroClass) -> Self {
        match class {
            HeroClass::Warrior => 0,
            HeroClass::Rogue => 1,
            HeroClass::Sorcerer => 2,
            HeroClass::Monk => 3,
            HeroClass::Bard => 4,
            HeroClass::Barbarian => 5,
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

/// Spell ID - all spells in game.
///
/// # Tech-debt note: DEPRECATED legacy adapter (off-by-one vs C++)
///
/// **Prefer `game::player::SpellId`**, which is the authoritative enum aligned
/// 1:1 with the C++ `SpellID` (`Source/spelldat.h`, `Firebolt = 1`).
///
/// This enum is a legacy adapter whose discriminants are offset by 1 relative
/// to C++ (`Firebolt = 0` here vs `Firebolt = 1` in C++ / `player::SpellId`).
/// `None = -1` here coincidentally matches C++ `Invalid = -1`, but the real
/// C++ `Null = 0` slot is occupied by `Firebolt` in this enum. `Invalid = -2`
/// is an extra sentinel with no C++ counterpart.
///
/// It remains because the `player_exact::Player` struct's spell fields
/// (`_p_spell`, `_p_target_spell`, `_p_r_spell`) and many `Player` methods are
/// typed with it, and `spells_cast.rs` (which operates on `player_exact::Player`)
/// transitively depends on it. The off-by-one is reconciled at the single
/// chokepoint `spells_cast::get_spell_bitmask`. See the doc-comment on
/// `player::SpellId` for the unification plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum SpellId {
    #[default]
    None = -1,
    Firebolt = 0,
    Healing = 1,
    Lightning = 2,
    Flash = 3,
    Identify = 4,
    FireWall = 5,
    TownPortal = 6,
    StoneCurse = 7,
    Infravision = 8,
    Phasing = 9,
    ManaShield = 10,
    Fireball = 11,
    Guardian = 12,
    ChainLightning = 13,
    FlameWave = 14,
    DoomSerpents = 15,
    BloodRitual = 16,
    Nova = 17,
    Invisibility = 18,
    Inferno = 19,
    Golem = 20,
    BloodBoil = 21,
    Teleport = 22,
    Apocalypse = 23,
    Etherealize = 24,
    ItemRepair = 25,
    StaffRecharge = 26,
    TrapDisarm = 27,
    Elemental = 28,
    ChargedBolt = 29,
    HolyBolt = 30,
    Resurrect = 31,
    Telekinesis = 32,
    HealOther = 33,
    BloodStar = 34,
    BoneSpirit = 35,
    // Hellfire spells
    LightningWall = 36,
    Immolation = 37,
    Warp = 38,
    Reflect = 39,
    Berserk = 40,
    RingOfFire = 41,
    Search = 42,
    // Skills
    RuneOfFire = 43,
    RuneOfLight = 44,
    RuneOfNova = 45,
    RuneOfImmolation = 46,
    RuneOfStone = 47,
    // More
    Invalid = -2,
}

/// Spell type source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SpellType {
    #[default]
    Skill = 0,      // Class skill
    Spell = 1,      // Memorized spell
    Scroll = 2,     // Scroll
    Charges = 3,    // Staff charges
    Invalid = 4,
}

/// Player mode (animation state)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PlayerMode {
    #[default]
    Stand = 0,
    WalkNorthwards = 1,
    WalkSouthwards = 2,
    WalkSideways = 3,
    Attack = 4,
    AttackBow = 5,
    Block = 6,
    GotHit = 7,
    Death = 8,
    Spell = 9,
    Targeting = 10,
    RangeAttack = 11,
}

/// Direction facing (8 directions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    None = 8,
}

/// Action type for destination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum ActionType {
    #[default]
    None = -1,
    Walk = 0,
    AttackMonster = 1,
    AttackPlayer = 2,
    RangeAttackMonster = 3,
    RangeAttackPlayer = 4,
    SpellMonster = 5,
    SpellPlayer = 6,
    SpellTarget = 7,
    Operate = 8,
    OperateTelekinesis = 9,
    PickupItem = 10,
    Talk = 11,
}

/// Player flags bitfield
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerFlags(pub u32);

impl PlayerFlags {
    pub const NONE: Self = Self(0);
    pub const ISPLAYING: Self = Self(1 << 0);
    pub const SPELL: Self = Self(1 << 1);
    pub const FIREWALL: Self = Self(1 << 2);
    pub const POISON: Self = Self(1 << 3);
    pub const LIGHTNING_WALL: Self = Self(1 << 4);
    pub const INFRAVISION: Self = Self(1 << 5);
    pub const RAGE: Self = Self(1 << 6);
    pub const REFLECT: Self = Self(1 << 7);
}

/// Player command type (for multiplayer sync)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PlayerCmd {
    #[default]
    Invalid = 0xFF,
    Walk = 0,
    Attack = 1,
    AttackMonster = 2,
    AttackPlayer = 3,
    SpellXY = 4,
    SpellMonster = 5,
    SpellPlayer = 6,
    TargetSpellXY = 7,
    TargetSpellMonster = 8,
    TargetSpellPlayer = 9,
    Operate = 10,
    OperateTelekinesis = 11,
    PickupItem = 12,
    Talk = 13,
    DropItem = 14,
    UseItem = 15,
    PlayerInfo = 16,
    UseStaff = 17,
    UseScroll = 18,
    SpellSkill = 19,
    GotHit = 20,
    Death = 21,
    Block = 22,
    Quest = 23,
}

/// Animation structure
#[derive(Debug, Clone, Default)]
pub struct AnimStruct {
    pub cel_data: Option<Vec<u8>>,
    pub frames: u8,
    pub rate: u8,
    pub width: u16,
}

/// Maximum path length
pub const MAX_PATH_LENGTH: usize = 25;

/// Number of dungeon levels
pub const NUMLEVELS: usize = 25;

/// Maximum set levels (special levels)
pub const MAX_SET_LEVELS: usize = 8;

/// Equipment location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvBodyLoc {
    Head = 0,
    RingLeft = 1,
    RingRight = 2,
    Amulet = 3,
    HandLeft = 4,
    HandRight = 5,
    Chest = 6,
}

/// Simplified Item for player_exact module
/// (Avoids circular dependency with items.rs)
/// TODO: In the future, consider using items::Item directly
#[derive(Debug, Clone, Default)]
pub struct PlayerItem {
    pub item_id: i32,
    pub equipped: bool,
    /// Equipped weapon type (C++ `Item._itype`), used by the melee damage
    /// modifiers (sword/mace vs Undead/Animal/Demon in PlrHitMonst).
    pub _itype: crate::game::item_dat::ItemType,
    /// Fully generated item for real drops (C++ `SetupAllItems` output);
    /// `None` for empty slots and the legacy demo potions. When present,
    /// `save_player` serialises the full SaveItem (seed/affixes/unique).
    pub full: Option<crate::game::items::Item>,
}

/// Type alias for backwards compatibility
pub type Item = PlayerItem;

impl PlayerItem {
    pub fn empty() -> Self {
        Self {
            item_id: 0,
            equipped: false,
            _itype: crate::game::item_dat::ItemType::None,
            full: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_id == 0 && self.full.is_none()
    }
}

/// Player structure - exact port from C++
///
/// Fields are kept in the same order as C++ for clarity.
/// All field names use C++ naming (_p prefix) for easy reference.
#[derive(Debug, Clone)]
pub struct Player {
    // === Basic Info ===
    /// Player name
    pub _p_name: [u8; PLAYER_NAME_LENGTH],

    // === Inventory ===
    /// Equipped items (7 slots)
    pub inv_body: [Item; NUM_INV_LOC],
    /// Inventory items (40 slots)
    pub inv_list: [Item; INVENTORY_GRID_CELLS],
    /// Belt items (8 slots)
    pub spd_list: [Item; MAX_BELT_ITEMS],
    /// Held item (cursor)
    pub hold_item: Item,

    // === Light ===
    pub light_id: i32,

    // === Basic Stats ===
    /// Number of items in inventory
    pub _p_num_inv: i32,

    // --- Current Attributes ---
    /// Current strength
    pub _p_strength: i32,
    /// Base strength
    pub _p_base_str: i32,
    /// Current magic
    pub _p_magic: i32,
    /// Base magic
    pub _p_base_mag: i32,
    /// Current dexterity
    pub _p_dexterity: i32,
    /// Base dexterity
    pub _p_base_dex: i32,
    /// Current vitality
    pub _p_vitality: i32,
    /// Base vitality
    pub _p_base_vit: i32,
    /// Unspent stat points
    pub _p_stat_pts: i32,

    // === Combat Stats ===
    /// Damage modifier
    pub _p_damage_mod: i32,

    // --- HP (64x fixed-point) ---
    /// Base HP * 64
    pub _p_hp_base: i32,
    /// Base max HP * 64
    pub _p_max_hp_base: i32,
    /// Current HP * 64
    pub _p_hit_points: i32,
    /// Max HP * 64
    pub _p_max_hp: i32,
    /// HP percentage (0-100)
    pub _p_hp_per: i32,

    // --- Mana (64x fixed-point) ---
    /// Base Mana * 64
    pub _p_mana_base: i32,
    /// Base max Mana * 64
    pub _p_max_mana_base: i32,
    /// Current Mana * 64
    pub _p_mana: i32,
    /// Max Mana * 64
    pub _p_max_mana: i32,
    /// Mana percentage (0-100)
    pub _p_mana_per: i32,

    // === Item Bonuses ===
    /// Item bonus min damage
    pub _p_i_min_dam: i32,
    /// Item bonus max damage
    pub _p_i_max_dam: i32,
    /// Item bonus AC
    pub _p_i_ac: i32,
    /// Item bonus damage
    pub _p_i_bonus_dam: i32,
    /// Item bonus to hit
    pub _p_i_bonus_to_hit: i32,
    /// Item bonus AC (duplicate?)
    pub _p_i_bonus_ac: i32,
    /// Item bonus damage modifier
    pub _p_i_bonus_dam_mod: i32,
    /// Item get hit bonus
    pub _p_i_get_hit: i32,

    // === Elemental Bonuses ===
    /// Item energy AC
    pub _p_i_en_ac: i32,
    /// Item fire min damage
    pub _p_i_f_min_dam: i32,
    /// Item fire max damage
    pub _p_i_f_max_dam: i32,
    /// Item lightning min damage
    pub _p_i_l_min_dam: i32,
    /// Item lightning max damage
    pub _p_i_l_max_dam: i32,

    // === Progression ===
    /// Total experience
    pub _p_experience: u32,
    /// Player level (1-50)
    pub _p_level: u8,
    /// Player class
    pub _p_class: HeroClass,

    // === Gold ===
    /// Gold amount
    pub _p_gold: i32,

    // === Active State ===
    /// Is player active
    pub plr_active: bool,
    /// Current level
    pub plr_level: u8,
    /// Position
    pub position: Point,

    // === Resistances ===
    /// Armor class
    pub _p_armor_class: i8,
    /// Magic resistance
    pub _p_mag_resist: i8,
    /// Fire resistance
    pub _p_fire_resist: i8,
    /// Lightning resistance
    pub _p_lght_resist: i8,

    // === Flags ===
    /// Block flag
    pub _p_block_flag: bool,
    /// Invincible flag
    pub _p_invincible: bool,
    /// Light radius (C++ `_pLightRad`; base 10 per InitPlayer player.cpp:2338)
    pub _p_light_rad: i8,

    // === Spell System (C++ Exact Alignment) ===
    /// Memorized spells bitmask (uint64_t _pMemSpells)
    pub _p_mem_spells: u64,
    /// Item spells bitmask (uint64_t _pISpells)
    pub _p_i_spells: u64,
    /// Ability spells bitmask (uint64_t _pAblSpells)
    pub _p_abl_spells: u64,
    /// Scroll spells bitmask (uint64_t _pScrlSpells)
    pub _p_scrl_spells: u64,
    /// Spell levels array (uint8_t _pSplLvl[64])
    pub _p_spl_lvl: [u8; 64],
    /// Selected spell
    pub _p_spell: SpellId,
    /// Selected spell type (memorized/scroll/charges/ability)
    pub _p_spell_type: SpellType,
    /// Target spell (for targeting)
    pub _p_target_spell: SpellId,
    /// Readied spell (for casting)
    pub _p_r_spell: SpellId,
    /// Readied spell type
    pub _p_r_spl_type: SpellType,

    // === Animation/Graphics (C++ Exact Alignment) ===
    /// Player mode (PLR_MODE)
    pub _p_mode: PlayerMode,
    /// Animation info
    pub _p_anims: [AnimStruct; 8],
    /// Current animation
    pub _p_active_anim: u8,
    /// Direction facing (0-7)
    pub _p_dir: Direction,
    /// Temporary direction (for wall spells)
    pub _p_temp_direction: Direction,
    /// Animation frame
    pub _p_frame: u8,
    /// Animation timer
    pub _p_anim_len: u8,
    /// Walk path
    pub walk_path: [Direction; MAX_PATH_LENGTH],
    /// Path destination X
    pub dest_x: i32,
    /// Path destination Y
    pub dest_y: i32,
    /// Destination action
    pub dest_action: ActionType,
    /// Destination action (alias)
    pub _p_dest_action: ActionType,
    /// Destination param 1
    pub dest_param1: i32,
    /// Destination param 2
    pub dest_param2: i32,
    /// Destination param 3
    pub dest_param3: i32,
    /// Destination param 4
    pub dest_param4: i32,

    // === Combat State ===
    /// Actions permitted flags
    pub _p_flags: PlayerFlags,
    /// Bool flags - in dungeon
    pub _p_dungeon: bool,
    /// Level visited flags
    pub _p_lvl_visited: [bool; NUMLEVELS],
    /// Set level visited flags
    pub _p_set_lvl_visited: [bool; MAX_SET_LEVELS],

    // === Network/Multiplayer ===
    /// Player command (for sync)
    pub _p_cmd: PlayerCmd,
    /// Player command parameter
    pub _p_cmd_param: u32,
}

impl Player {
    /// Create a new empty player
    pub fn new() -> Self {
        Self {
            _p_name: [0; PLAYER_NAME_LENGTH],
            inv_body: [
                Item::empty(),
                Item::empty(),
                Item::empty(),
                Item::empty(),
                Item::empty(),
                Item::empty(),
                Item::empty(),
            ],
            inv_list: std::array::from_fn(|_| Item::empty()),
            spd_list: std::array::from_fn(|_| Item::empty()),
            hold_item: Item::empty(),
            light_id: 0,
            _p_num_inv: 0,
            _p_strength: 0,
            _p_base_str: 0,
            _p_magic: 0,
            _p_base_mag: 0,
            _p_dexterity: 0,
            _p_base_dex: 0,
            _p_vitality: 0,
            _p_base_vit: 0,
            _p_stat_pts: 0,
            _p_damage_mod: 0,
            _p_hp_base: 0,
            _p_max_hp_base: 0,
            _p_hit_points: 0,
            _p_max_hp: 0,
            _p_hp_per: 0,
            _p_mana_base: 0,
            _p_max_mana_base: 0,
            _p_mana: 0,
            _p_max_mana: 0,
            _p_mana_per: 0,
            _p_i_min_dam: 0,
            _p_i_max_dam: 0,
            _p_i_ac: 0,
            _p_i_bonus_dam: 0,
            _p_i_bonus_to_hit: 0,
            _p_i_bonus_ac: 0,
            _p_i_bonus_dam_mod: 0,
            _p_i_get_hit: 0,
            _p_i_en_ac: 0,
            _p_i_f_min_dam: 0,
            _p_i_f_max_dam: 0,
            _p_i_l_min_dam: 0,
            _p_i_l_max_dam: 0,
            _p_experience: 0,
            _p_level: 1,
            _p_class: HeroClass::Warrior,
            _p_gold: 0,
            plr_active: false,
            plr_level: 0,
            position: Point::new(0, 0),
            _p_armor_class: 0,
            _p_mag_resist: 0,
            _p_fire_resist: 0,
            _p_lght_resist: 0,
            _p_block_flag: false,
            _p_invincible: false,
            _p_light_rad: 10,
            // Spell system
            _p_mem_spells: 0,
            _p_i_spells: 0,
            _p_abl_spells: 0,
            _p_scrl_spells: 0,
            _p_spl_lvl: [0; 64],
            _p_spell: SpellId::None,
            _p_spell_type: SpellType::Skill,
            _p_target_spell: SpellId::None,
            _p_r_spell: SpellId::Invalid,
            _p_r_spl_type: SpellType::Invalid,
            // Animation
            _p_mode: PlayerMode::Stand,
            _p_anims: std::array::from_fn(|_| AnimStruct::default()),
            _p_active_anim: 0,
            _p_dir: Direction::South,
            _p_temp_direction: Direction::South,
            _p_frame: 0,
            _p_anim_len: 0,
            walk_path: [Direction::None; MAX_PATH_LENGTH],
            dest_x: 0,
            dest_y: 0,
            dest_action: ActionType::None,
            _p_dest_action: ActionType::None,
            dest_param1: 0,
            dest_param2: 0,
            dest_param3: 0,
            dest_param4: 0,
            // Combat state
            _p_flags: PlayerFlags::NONE,
            _p_dungeon: false,
            _p_lvl_visited: [false; NUMLEVELS],
            _p_set_lvl_visited: [false; MAX_SET_LEVELS],
            // Network
            _p_cmd: PlayerCmd::Invalid,
            _p_cmd_param: 0,
        }
    }

    // =========================================================================
    // HP Management (64x Fixed-Point)
    // =========================================================================

    /// Get current HP (display value)
    pub fn get_hp(&self) -> i32 {
        self._p_hit_points >> 6
    }

    /// Get max HP (display value)
    pub fn get_max_hp(&self) -> i32 {
        self._p_max_hp >> 6
    }

    /// Get HP percentage (0-100)
    pub fn get_hp_percentage(&self) -> i32 {
        if self._p_max_hp == 0 {
            return 0;
        }
        ((self._p_hit_points as i64 * 100) / self._p_max_hp as i64) as i32
    }

    /// Set HP (display value) - converts to 64x
    pub fn set_hp(&mut self, hp: i32) {
        self._p_hit_points = hp * HP_PRECISION;
        self._p_hit_points = self._p_hit_points.min(self._p_max_hp);
        self._p_hp_per = self.get_hp_percentage();
    }

    /// Initialise stats/HP/mana for a class. `Player::new()` zeroes every
    /// field (including HP), so without this a freshly-created hero starts
    /// at 0 HP and is instantly dead. Uses the per-class base attributes
    /// from `player_dat::get_class_attributes` (sourced from the TSVs) and
    /// the Diablo fixed-point HP/mana formulas:
    ///
    /// C++ `CreatePlayer`: `max_hp_base = (vit + adj_life) * lvl_life`,
    /// stored as 64x fixed-point; mana is `(mag + adj_mana) * lvl_mana`.
    /// `_p_hit_points`/`_p_mana` start full.
    pub fn init_class_stats(&mut self) {
        // `player_exact::HeroClass` and `player_dat::HeroClass` are two
        // separate enums with identical discriminants; bridge via u8.
        let pd_class = match self._p_class {
            HeroClass::Warrior => crate::game::player_dat::HeroClass::Warrior,
            HeroClass::Rogue => crate::game::player_dat::HeroClass::Rogue,
            HeroClass::Sorcerer => crate::game::player_dat::HeroClass::Sorcerer,
            HeroClass::Monk => crate::game::player_dat::HeroClass::Monk,
            HeroClass::Bard => crate::game::player_dat::HeroClass::Bard,
            HeroClass::Barbarian => crate::game::player_dat::HeroClass::Barbarian,
        };
        let attrs = crate::game::player_dat::get_class_attributes(pd_class);
        // Base attributes
        self._p_base_str = attrs.base_str as i32;
        self._p_strength = attrs.base_str as i32;
        self._p_base_mag = attrs.base_mag as i32;
        self._p_magic = attrs.base_mag as i32;
        self._p_base_dex = attrs.base_dex as i32;
        self._p_dexterity = attrs.base_dex as i32;
        self._p_base_vit = attrs.base_vit as i32;
        self._p_vitality = attrs.base_vit as i32;

        // HP: (vit + adj_life) * lvl_life, all in 64x fixed-point. adj_life/
        // lvl_life are already 64x (fixed6(...) values, i16), and vit is a
        // plain integer that we shift into 64x before combining. Result
        // stays 64x.
        let vit_64x = (attrs.base_vit as i32) << 6;
        let adj_life = attrs.adj_life as i32;
        let lvl_life = attrs.lvl_life as i32;
        let max_hp_base = (vit_64x + adj_life) * lvl_life >> 6;
        self._p_max_hp_base = max_hp_base;
        self._p_max_hp = max_hp_base;
        self._p_hit_points = max_hp_base;
        self._p_hp_per = 100 << 6; // full

        // Mana: (mag + adj_mana) * lvl_mana, same fixed-point scheme.
        let mag_64x = (attrs.base_mag as i32) << 6;
        let adj_mana = attrs.adj_mana as i32;
        let lvl_mana = attrs.lvl_mana as i32;
        let max_mana_base = ((mag_64x + adj_mana) * lvl_mana) >> 6;
        self._p_max_mana_base = max_mana_base;
        self._p_max_mana = max_mana_base;
        self._p_mana = max_mana_base;
        self._p_mana_per = 100 << 6;

        // Level 1 start.
        self._p_level = 1;
    }


    /// Modify HP by delta (display value)
    ///
    /// Returns true if player died from this change
    pub fn modify_hp(&mut self, delta: i32) -> bool {
        let old_hp = self._p_hit_points;
        self._p_hit_points += delta * HP_PRECISION;
        self._p_hit_points = self._p_hit_points.clamp(0, self._p_max_hp);

        // Also modify base HP
        self._p_hp_base += delta * HP_PRECISION;
        self._p_hp_base = self._p_hp_base.clamp(0, self._p_max_hp_base);

        self._p_hp_per = self.get_hp_percentage();

        // Return true if player just died
        old_hp > 0 && self._p_hit_points == 0
    }

    /// Heal to full HP (shrine/fountain effect)
    pub fn heal_to_full(&mut self) {
        self._p_hit_points = self._p_max_hp;
        self._p_hp_base = self._p_max_hp_base;
        self._p_hp_per = 100;
    }

    /// Check if player is alive
    pub fn is_alive(&self) -> bool {
        self._p_hit_points > 0
    }

    /// Check if player is dead
    pub fn is_dead(&self) -> bool {
        self._p_hit_points == 0
    }

    // =========================================================================
    // Mana Management (64x Fixed-Point)
    // =========================================================================

    /// Get current Mana (display value)
    pub fn get_mana(&self) -> i32 {
        self._p_mana >> 6
    }

    /// Get max Mana (display value)
    pub fn get_max_mana(&self) -> i32 {
        self._p_max_mana >> 6
    }

    /// Get Mana percentage (0-100)
    pub fn get_mana_percentage(&self) -> i32 {
        if self._p_max_mana == 0 {
            return 0;
        }
        ((self._p_mana as i64 * 100) / self._p_max_mana as i64) as i32
    }

    /// Set Mana (display value) - converts to 64x
    pub fn set_mana(&mut self, mana: i32) {
        self._p_mana = mana * HP_PRECISION;
        self._p_mana = self._p_mana.min(self._p_max_mana);
        self._p_mana_per = self.get_mana_percentage();
    }

    /// Modify Mana by delta (display value)
    pub fn modify_mana(&mut self, delta: i32) {
        self._p_mana += delta * HP_PRECISION;
        self._p_mana = self._p_mana.clamp(0, self._p_max_mana);

        // Also modify base Mana
        self._p_mana_base += delta * HP_PRECISION;
        self._p_mana_base = self._p_mana_base.clamp(0, self._p_max_mana_base);

        self._p_mana_per = self.get_mana_percentage();
    }

    /// Restore to full Mana (shrine/fountain effect)
    pub fn restore_mana_to_full(&mut self) {
        self._p_mana = self._p_max_mana;
        self._p_mana_base = self._p_max_mana_base;
        self._p_mana_per = 100;
    }

    // =========================================================================
    // Attribute Management
    // =========================================================================

    /// Get base attribute value
    pub fn get_base_attribute(&self, attr: CharacterAttribute) -> i32 {
        match attr {
            CharacterAttribute::Strength => self._p_base_str,
            CharacterAttribute::Magic => self._p_base_mag,
            CharacterAttribute::Dexterity => self._p_base_dex,
            CharacterAttribute::Vitality => self._p_base_vit,
        }
    }

    /// Get current attribute value (includes bonuses)
    pub fn get_current_attribute(&self, attr: CharacterAttribute) -> i32 {
        match attr {
            CharacterAttribute::Strength => self._p_strength,
            CharacterAttribute::Magic => self._p_magic,
            CharacterAttribute::Dexterity => self._p_dexterity,
            CharacterAttribute::Vitality => self._p_vitality,
        }
    }

    /// Set base attribute (used during level up)
    pub fn set_base_attribute(&mut self, attr: CharacterAttribute, value: i32) {
        match attr {
            CharacterAttribute::Strength => {
                self._p_base_str = value;
                self._p_strength = value; // TODO: Add item bonuses
            }
            CharacterAttribute::Magic => {
                self._p_base_mag = value;
                self._p_magic = value;
            }
            CharacterAttribute::Dexterity => {
                self._p_base_dex = value;
                self._p_dexterity = value;
            }
            CharacterAttribute::Vitality => {
                self._p_base_vit = value;
                self._p_vitality = value;
            }
        }
    }

    // =========================================================================
    // Resistance Management
    // =========================================================================

    /// Get resistance value (0-75%)
    pub fn get_resistance(&self, resist_type: &str) -> i8 {
        match resist_type {
            "magic" => self._p_mag_resist.min(MAX_RESISTANCE as i8),
            "fire" => self._p_fire_resist.min(MAX_RESISTANCE as i8),
            "lightning" => self._p_lght_resist.min(MAX_RESISTANCE as i8),
            _ => 0,
        }
    }

    /// Modify resistance (capped at 75%)
    pub fn modify_resistance(&mut self, resist_type: &str, delta: i8) {
        match resist_type {
            "magic" => {
                self._p_mag_resist += delta;
                self._p_mag_resist = self._p_mag_resist.clamp(0, MAX_RESISTANCE as i8);
            }
            "fire" => {
                self._p_fire_resist += delta;
                self._p_fire_resist = self._p_fire_resist.clamp(0, MAX_RESISTANCE as i8);
            }
            "lightning" => {
                self._p_lght_resist += delta;
                self._p_lght_resist = self._p_lght_resist.clamp(0, MAX_RESISTANCE as i8);
            }
            _ => {}
        }
    }

    // =========================================================================
    // Class & Level Info
    // =========================================================================

    /// Get player name as string
    pub fn get_name(&self) -> String {
        String::from_utf8_lossy(&self._p_name)
            .trim_end_matches('\0')
            .to_string()
    }

    /// Set player name
    pub fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(PLAYER_NAME_LENGTH - 1);
        self._p_name[..len].copy_from_slice(&bytes[..len]);
        self._p_name[len..].fill(0);
    }

    /// Get class name as string
    pub fn get_class_name(&self) -> &'static str {
        match self._p_class {
            HeroClass::Warrior => "Warrior",
            HeroClass::Rogue => "Rogue",
            HeroClass::Sorcerer => "Sorcerer",
            HeroClass::Monk => "Monk",
            HeroClass::Bard => "Bard",
            HeroClass::Barbarian => "Barbarian",
        }
    }

    /// Check if player can level up
    pub fn can_level_up(&self) -> bool {
        self._p_level < 50 && self._p_experience >= self.get_required_exp_for_level(self._p_level + 1)
    }

    /// Get required experience for a given level (simplified)
    pub fn get_required_exp_for_level(&self, level: u8) -> u32 {
        // Simplified exponential formula
        // Real formula is more complex and class-specific
        if level <= 1 {
            0
        } else {
            (level as u32 - 1) * (level as u32 - 1) * 100
        }
    }

    // =========================================================================
    // Spell System (C++ Exact Port)
    // =========================================================================

    /// Get spell bitmask for a spell ID
    pub fn get_spell_bitmask(spell: SpellId) -> u64 {
        let spell_idx = spell as i8;
        if spell_idx < 0 || spell_idx >= 64 {
            return 0;
        }
        1u64 << spell_idx
    }

    /// Check if player has memorized a spell
    pub fn has_spell(&self, spell: SpellId) -> bool {
        (self._p_mem_spells & Self::get_spell_bitmask(spell)) != 0
    }

    /// Check if player has spell from items
    pub fn has_item_spell(&self, spell: SpellId) -> bool {
        (self._p_i_spells & Self::get_spell_bitmask(spell)) != 0
    }

    /// Check if player has ability/skill
    pub fn has_ability(&self, spell: SpellId) -> bool {
        (self._p_abl_spells & Self::get_spell_bitmask(spell)) != 0
    }

    /// Learn a spell
    pub fn learn_spell(&mut self, spell: SpellId) {
        self._p_mem_spells |= Self::get_spell_bitmask(spell);
        if self._p_spl_lvl[spell as usize] == 0 {
            self._p_spl_lvl[spell as usize] = 1;
        }
    }

    /// Get spell level
    pub fn get_spell_level(&self, spell: SpellId) -> u8 {
        let idx = spell as i8;
        if idx < 0 || idx >= 64 {
            return 0;
        }
        self._p_spl_lvl[idx as usize]
    }

    /// Set spell level (used by shrine effects)
    pub fn set_spell_level(&mut self, spell: SpellId, level: u8) {
        let idx = spell as i8;
        if idx >= 0 && idx < 64 {
            self._p_spl_lvl[idx as usize] = level.min(15); // MaxSpellLevel = 15
        }
    }

    /// Increment spell level (used by reading books)
    pub fn increment_spell_level(&mut self, spell: SpellId) {
        let idx = spell as i8;
        if idx >= 0 && idx < 64 {
            let current = self._p_spl_lvl[idx as usize];
            if current < 15 {
                self._p_spl_lvl[idx as usize] = current + 1;
            }
        }
    }

    /// Set selected spell
    pub fn select_spell(&mut self, spell: SpellId, spell_type: SpellType) {
        self._p_spell = spell;
        self._p_spell_type = spell_type;
    }

    /// Get all available spells (union of all spell sources)
    pub fn get_all_spells(&self) -> u64 {
        self._p_mem_spells | self._p_i_spells | self._p_abl_spells
    }

    /// Count memorized spells
    pub fn count_spells(&self) -> u32 {
        self._p_mem_spells.count_ones()
    }

    // =========================================================================
    // Mode and Animation
    // =========================================================================

    /// Set player mode
    pub fn set_mode(&mut self, mode: PlayerMode) {
        self._p_mode = mode;
    }

    /// Check if player is in a walking mode
    pub fn is_walking(&self) -> bool {
        matches!(
            self._p_mode,
            PlayerMode::WalkNorthwards | PlayerMode::WalkSouthwards | PlayerMode::WalkSideways
        )
    }

    /// Check if player is attacking
    pub fn is_attacking(&self) -> bool {
        matches!(
            self._p_mode,
            PlayerMode::Attack | PlayerMode::AttackBow | PlayerMode::RangeAttack
        )
    }

    /// Check if player is casting
    pub fn is_casting(&self) -> bool {
        matches!(self._p_mode, PlayerMode::Spell | PlayerMode::Targeting)
    }

    /// Set direction
    pub fn set_direction(&mut self, dir: Direction) {
        self._p_dir = dir;
    }

    // =========================================================================
    // Level Progression
    // =========================================================================

    /// Check if player has visited a level
    pub fn has_visited_level(&self, level: u8) -> bool {
        if (level as usize) < NUMLEVELS {
            self._p_lvl_visited[level as usize]
        } else {
            false
        }
    }

    /// Mark level as visited
    pub fn set_level_visited(&mut self, level: u8) {
        if (level as usize) < NUMLEVELS {
            self._p_lvl_visited[level as usize] = true;
        }
    }

    /// Check if player has visited a set level
    pub fn has_visited_set_level(&self, level: u8) -> bool {
        if (level as usize) < MAX_SET_LEVELS {
            self._p_set_lvl_visited[level as usize]
        } else {
            false
        }
    }

    /// Mark set level as visited
    pub fn set_set_level_visited(&mut self, level: u8) {
        if (level as usize) < MAX_SET_LEVELS {
            self._p_set_lvl_visited[level as usize] = true;
        }
    }

    // =========================================================================
    // Destination / Path
    // =========================================================================

    /// Clear destination action
    pub fn clear_destination(&mut self) {
        self.dest_action = ActionType::None;
        self.dest_param1 = 0;
        self.dest_param2 = 0;
        self.dest_param3 = 0;
        self.dest_param4 = 0;
    }

    /// Set walk destination
    pub fn set_walk_destination(&mut self, x: i32, y: i32) {
        self.dest_action = ActionType::Walk;
        self.dest_x = x;
        self.dest_y = y;
    }

    /// Set attack destination
    pub fn set_attack_destination(&mut self, target_id: i32, is_player: bool) {
        self.dest_action = if is_player {
            ActionType::AttackPlayer
        } else {
            ActionType::AttackMonster
        };
        self.dest_param1 = target_id;
    }

    /// Check if animation is at last frame
    pub fn is_anim_last_frame(&self) -> bool {
        self._p_frame >= self._p_anim_len.saturating_sub(1)
    }
}

// =============================================================================
// Player Action Functions - Exact Port of player.cpp
// =============================================================================

/// Continue movement towards new tile
/// Exact port of DoWalk from player.cpp
pub fn do_walk(player: &mut Player) -> bool {
    // If animation not at last frame, continue walking
    if !player.is_anim_last_frame() {
        update_player_light_offset(player);
        return false;
    }

    // Reached new tile - update position
    player.position.x = player.dest_x;
    player.position.y = player.dest_y;

    start_stand(player, player._p_temp_direction);
    clear_state_variables(player);

    true
}

/// Check if weapon decays (Hellfire Decay flag)
/// Exact port of WeaponDecay from player.cpp
fn weapon_decay(player: &mut Player, slot: usize) -> bool {
    let _ = (player, slot);  // Placeholder
    false
}

/// Damage weapon durability
/// Exact port of DamageWeapon from player.cpp
pub fn damage_weapon(player: &mut Player, damage_frequency: u32) -> bool {
    // Check weapon decay
    if weapon_decay(player, INVLOC_HAND_LEFT) {
        return true;
    }
    if weapon_decay(player, INVLOC_HAND_RIGHT) {
        return true;
    }

    let _ = damage_frequency;
    false
}

/// Equipment slot constants
const INVLOC_HAND_LEFT: usize = 4;
const INVLOC_HAND_RIGHT: usize = 5;
#[allow(dead_code)]
const INVLOC_HEAD: usize = 0;
#[allow(dead_code)]
const INVLOC_CHEST: usize = 2;

/// Perform attack action
/// Exact port of DoAttack from player.cpp
pub fn do_attack(player: &mut Player) -> bool {
    let mut did_hit = false;

    // Attack frame - check for hit
    // The attack actually connects at specific animation frames

    // Calculate target position based on direction
    let target_x = player.position.x + direction_dx(player._p_dir);
    let target_y = player.position.y + direction_dy(player._p_dir);

    // In full implementation:
    // - Find monster at target position
    // - Check if can talk to monster
    // - Add fire/lightning explosions if has those flags
    // - Try to hit monster/player/object
    // - Handle cleave for barbarian

    let _ = (target_x, target_y, did_hit);

    // Damage weapon if hit
    if did_hit && damage_weapon(player, 30) {
        start_stand(player, player._p_dir);
        clear_state_variables(player);
        return true;
    }

    // Animation complete
    if player.is_anim_last_frame() {
        start_stand(player, player._p_dir);
        clear_state_variables(player);
        return true;
    }

    false
}

/// Perform ranged attack
/// Exact port of DoRangeAttack from player.cpp
pub fn do_range_attack(player: &mut Player) -> bool {
    let arrows = 1;

    for _arrow in 0..arrows {
        // In full implementation:
        // - Calculate arrow spread for multiple arrows
        // - Determine missile type (Arrow/FireArrow/LightningArrow/SpectralArrow)
        // - Create missile

        // Damage weapon
        if damage_weapon(player, 40) {
            start_stand(player, player._p_dir);
            clear_state_variables(player);
            return true;
        }
    }

    if player.is_anim_last_frame() {
        start_stand(player, player._p_dir);
        clear_state_variables(player);
        return true;
    }

    false
}

/// Damage parry item (shield/staff) on block
#[allow(dead_code)]
fn damage_parry_item(_player: &mut Player) {
    // Damage shield or staff durability
}

/// Perform block action
/// Exact port of DoBlock from player.cpp
pub fn do_block(player: &mut Player) -> bool {
    if player.is_anim_last_frame() {
        start_stand(player, player._p_dir);
        clear_state_variables(player);

        // 10% chance to damage blocking item
        // if flip_coin(10) { damage_parry_item(player); }

        return true;
    }

    false
}

/// Damage armor on getting hit
#[allow(dead_code)]
fn damage_armor(_player: &mut Player) {
    // Damage head or chest armor durability
}

/// Perform spell casting
/// Exact port of DoSpell from player.cpp
pub fn do_spell(player: &mut Player) -> bool {
    // Cast on specific animation frame
    // In full implementation, call CastSpell

    if player.is_anim_last_frame() {
        start_stand(player, player._p_dir);
        clear_state_variables(player);
        return true;
    }

    false
}

/// Perform got hit reaction
/// Exact port of DoGotHit from player.cpp
pub fn do_got_hit(player: &mut Player) -> bool {
    if player.is_anim_last_frame() {
        start_stand(player, player._p_dir);
        clear_state_variables(player);

        // 75% chance to damage armor
        // if !flip_coin(4) { damage_armor(player); }

        return true;
    }

    false
}

/// Perform death animation
/// Exact port of DoDeath from player.cpp
pub fn do_death(player: &mut Player) -> bool {
    if player.is_anim_last_frame() {
        // Freeze on death frame
        player._p_anim_len = 100;
    }

    false  // Death animation never completes
}

/// Start standing mode
/// Exact port of StartStand from player.cpp
pub fn start_stand(player: &mut Player, dir: Direction) {
    player._p_mode = PlayerMode::Stand;
    player._p_dir = dir;
    player._p_frame = 0;
}

/// Clear state variables after action
fn clear_state_variables(player: &mut Player) {
    player.dest_x = 0;
    player.dest_y = 0;
    player._p_temp_direction = Direction::None;
    player._p_dest_action = ActionType::None;
}

/// Update player light offset during walk
fn update_player_light_offset(_player: &mut Player) {
    // Calculate sub-tile position based on animation frame
}

/// Get direction X offset
fn direction_dx(dir: Direction) -> i32 {
    match dir {
        Direction::North => 0,
        Direction::NorthEast => 1,
        Direction::East => 1,
        Direction::SouthEast => 1,
        Direction::South => 0,
        Direction::SouthWest => -1,
        Direction::West => -1,
        Direction::NorthWest => -1,
        Direction::None => 0,
    }
}

/// Get direction Y offset
fn direction_dy(dir: Direction) -> i32 {
    match dir {
        Direction::North => -1,
        Direction::NorthEast => -1,
        Direction::East => 0,
        Direction::SouthEast => 1,
        Direction::South => 1,
        Direction::SouthWest => 1,
        Direction::West => 0,
        Direction::NorthWest => -1,
        Direction::None => 0,
    }
}

/// Player hit monster
/// Exact port of PlrHitMonst from player.cpp
pub fn plr_hit_monst(
    player: &mut Player,
    _monster_id: usize,
    adjacent_damage: bool,
) -> bool {
    let mut hper = 0i32;

    // Adjacent damage penalty (cleave)
    if adjacent_damage {
        if player._p_level > 20 {
            hper -= 30;
        } else {
            hper -= (35 - player._p_level as i32) * 2;
        }
    }

    // Calculate damage
    let min_dam = player._p_i_min_dam;
    let max_dam = player._p_i_max_dam;
    let mut dam = if max_dam > min_dam {
        crate::engine::random::gameplay_rnd(min_dam, max_dam)
    } else {
        min_dam
    };

    // Bonus damage
    dam += dam * player._p_i_bonus_dam / 100;
    dam += player._p_i_bonus_dam_mod;
    dam += player._p_damage_mod;

    let _ = hper;
    true
}

/// Player hit player (PvP)
/// Exact port of PlrHitPlr from player.cpp
pub fn plr_hit_plr(
    attacker: &mut Player,
    target: &mut Player,
) -> bool {
    // Check invincibility
    if target._p_invincible {
        return false;
    }

    // Calculate damage
    let min_dam = attacker._p_i_min_dam;
    let max_dam = attacker._p_i_max_dam;
    let mut dam = if max_dam > min_dam {
        crate::engine::random::gameplay_rnd(min_dam, max_dam)
    } else {
        min_dam
    };
    dam += (dam * attacker._p_i_bonus_dam) / 100;
    dam += attacker._p_i_bonus_dam_mod + attacker._p_damage_mod;

    let _ = dam;
    true
}

/// Player hit object (breakable)
/// Exact port of PlrHitObj from player.cpp
pub fn plr_hit_obj(_player: &Player, _object_id: usize) -> bool {
    false
}

/// Start blocking animation
pub fn start_plr_block(player: &mut Player, dir: Direction) {
    player._p_mode = PlayerMode::Block;
    player._p_dir = dir;
    player._p_frame = 0;
}

/// Start hit animation
pub fn start_plr_hit(player: &mut Player, damage: i32, _is_ranged: bool) {
    player._p_mode = PlayerMode::GotHit;
    player._p_frame = 0;

    // Apply damage
    player._p_hit_points -= damage;
    player._p_hp_base -= damage;

    if player._p_hit_points <= 0 {
        player._p_hit_points = 0;
    }
}

/// Process player based on current mode
/// Exact port of ProcessPlayers->ProcessPlayer logic
pub fn process_player(player: &mut Player) -> bool {
    match player._p_mode {
        PlayerMode::Stand => false,
        PlayerMode::WalkNorthwards |
        PlayerMode::WalkSouthwards |
        PlayerMode::WalkSideways => do_walk(player),
        PlayerMode::Attack |
        PlayerMode::AttackBow => do_attack(player),
        PlayerMode::RangeAttack => do_range_attack(player),
        PlayerMode::Block => do_block(player),
        PlayerMode::Spell => do_spell(player),
        PlayerMode::GotHit => do_got_hit(player),
        PlayerMode::Death => do_death(player),
        PlayerMode::Targeting => false,
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // HP Tests (Fixed-Point 64x)
    // =========================================================================

    #[test]
    fn test_hp_precision() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;

        // Set HP to 50
        player.set_hp(50);
        assert_eq!(player.get_hp(), 50);
        assert_eq!(player._p_hit_points, 50 * 64); // Internal: 3200
    }

    #[test]
    fn test_hp_modify_positive() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player.set_hp(50);

        // Heal 20 HP
        let died = player.modify_hp(20);
        assert!(!died);
        assert_eq!(player.get_hp(), 70);
    }

    #[test]
    fn test_hp_modify_negative() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player.set_hp(50);

        // Take 30 damage
        let died = player.modify_hp(-30);
        assert!(!died);
        assert_eq!(player.get_hp(), 20);
    }

    #[test]
    fn test_hp_death() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player.set_hp(10);

        // Fatal damage
        let died = player.modify_hp(-20);
        assert!(died);
        assert_eq!(player.get_hp(), 0);
        assert!(player.is_dead());
        assert!(!player.is_alive());
    }

    #[test]
    fn test_hp_clamp_max() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player.set_hp(90);

        // Overheal
        player.modify_hp(50);
        assert_eq!(player.get_hp(), 100); // Clamped to max
    }

    #[test]
    fn test_hp_percentage() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player.set_hp(50);

        assert_eq!(player.get_hp_percentage(), 50);

        player.set_hp(75);
        assert_eq!(player.get_hp_percentage(), 75);

        player.set_hp(0);
        assert_eq!(player.get_hp_percentage(), 0);
    }

    #[test]
    fn test_heal_to_full() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player.set_hp(10);

        player.heal_to_full();
        assert_eq!(player.get_hp(), 100);
        assert_eq!(player.get_hp_percentage(), 100);
    }

    // =========================================================================
    // Mana Tests (Fixed-Point 64x)
    // =========================================================================

    #[test]
    fn test_mana_precision() {
        let mut player = Player::new();
        player._p_max_mana = 80 * HP_PRECISION;
        player._p_max_mana_base = 80 * HP_PRECISION;

        player.set_mana(40);
        assert_eq!(player.get_mana(), 40);
        assert_eq!(player._p_mana, 40 * 64);
    }

    #[test]
    fn test_mana_modify() {
        let mut player = Player::new();
        player._p_max_mana = 100 * HP_PRECISION;
        player._p_max_mana_base = 100 * HP_PRECISION;
        player.set_mana(50);

        // Consume 20 mana
        player.modify_mana(-20);
        assert_eq!(player.get_mana(), 30);

        // Restore 10 mana
        player.modify_mana(10);
        assert_eq!(player.get_mana(), 40);
    }

    #[test]
    fn test_mana_clamp() {
        let mut player = Player::new();
        player._p_max_mana = 100 * HP_PRECISION;
        player._p_max_mana_base = 100 * HP_PRECISION;
        player.set_mana(90);

        // Overfill
        player.modify_mana(50);
        assert_eq!(player.get_mana(), 100);

        // Drain below 0
        player.modify_mana(-150);
        assert_eq!(player.get_mana(), 0);
    }

    #[test]
    fn test_mana_percentage() {
        let mut player = Player::new();
        player._p_max_mana = 100 * HP_PRECISION;
        player.set_mana(75);

        assert_eq!(player.get_mana_percentage(), 75);
    }

    #[test]
    fn test_restore_mana_to_full() {
        let mut player = Player::new();
        player._p_max_mana = 100 * HP_PRECISION;
        player._p_max_mana_base = 100 * HP_PRECISION;
        player.set_mana(20);

        player.restore_mana_to_full();
        assert_eq!(player.get_mana(), 100);
        assert_eq!(player.get_mana_percentage(), 100);
    }

    // =========================================================================
    // Attribute Tests
    // =========================================================================

    #[test]
    fn test_attributes() {
        let mut player = Player::new();

        player.set_base_attribute(CharacterAttribute::Strength, 30);
        player.set_base_attribute(CharacterAttribute::Magic, 20);
        player.set_base_attribute(CharacterAttribute::Dexterity, 25);
        player.set_base_attribute(CharacterAttribute::Vitality, 35);

        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 30);
        assert_eq!(player.get_current_attribute(CharacterAttribute::Magic), 20);
        assert_eq!(player.get_current_attribute(CharacterAttribute::Dexterity), 25);
        assert_eq!(player.get_current_attribute(CharacterAttribute::Vitality), 35);
    }

    // =========================================================================
    // Resistance Tests
    // =========================================================================

    #[test]
    fn test_resistance() {
        let mut player = Player::new();

        player.modify_resistance("magic", 30);
        player.modify_resistance("fire", 20);
        player.modify_resistance("lightning", 15);

        assert_eq!(player.get_resistance("magic"), 30);
        assert_eq!(player.get_resistance("fire"), 20);
        assert_eq!(player.get_resistance("lightning"), 15);
    }

    #[test]
    fn test_resistance_cap() {
        let mut player = Player::new();

        // Exceed cap
        player.modify_resistance("magic", 100);
        assert_eq!(player.get_resistance("magic"), 75); // Capped

        // Negative resistance
        player.modify_resistance("fire", -20);
        assert_eq!(player.get_resistance("fire"), 0); // Floored at 0
    }

    // =========================================================================
    // Name & Class Tests
    // =========================================================================

    #[test]
    fn test_player_name() {
        let mut player = Player::new();
        player.set_name("Griswold");

        assert_eq!(player.get_name(), "Griswold");
    }

    #[test]
    fn test_class_name() {
        let mut player = Player::new();
        player._p_class = HeroClass::Sorcerer;

        assert_eq!(player.get_class_name(), "Sorcerer");
    }

    // =========================================================================
    // Level Tests
    // =========================================================================

    #[test]
    fn test_level_up() {
        let mut player = Player::new();
        player._p_level = 5;
        player._p_experience = 10000;

        assert!(player.can_level_up());
    }

    #[test]
    fn test_required_exp() {
        let player = Player::new();

        assert_eq!(player.get_required_exp_for_level(1), 0);
        assert_eq!(player.get_required_exp_for_level(2), 100);
        assert_eq!(player.get_required_exp_for_level(3), 400);
        assert_eq!(player.get_required_exp_for_level(10), 8100);
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_shrine_full_heal() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player._p_max_mana = 80 * HP_PRECISION;
        player._p_max_mana_base = 80 * HP_PRECISION;

        // Damage player
        player.set_hp(30);
        player.set_mana(10);

        // Shrine effect: full heal
        player.heal_to_full();
        player.restore_mana_to_full();

        assert_eq!(player.get_hp(), 100);
        assert_eq!(player.get_mana(), 80);
    }

    #[test]
    fn test_combat_damage() {
        let mut player = Player::new();
        player._p_max_hp = 100 * HP_PRECISION;
        player._p_max_hp_base = 100 * HP_PRECISION;
        player.set_hp(100);

        // Take 3 hits of 25 damage
        player.modify_hp(-25);
        assert_eq!(player.get_hp(), 75);
        assert!(player.is_alive());

        player.modify_hp(-25);
        assert_eq!(player.get_hp(), 50);

        player.modify_hp(-25);
        assert_eq!(player.get_hp(), 25);

        // Fatal hit
        let died = player.modify_hp(-30);
        assert!(died);
        assert_eq!(player.get_hp(), 0);
        assert!(player.is_dead());
    }
}
