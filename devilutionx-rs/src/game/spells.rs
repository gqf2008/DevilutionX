/// Spell system - Diablo-style magic
/// Exact port of DevilutionX Source/spelldat.h
use serde::{Deserialize, Serialize};

/// Spell ID - exact match of SpellID enum from spelldat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(i8)]
pub enum SpellId {
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
    // Hellfire spells
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

/// Spell type - exact match of SpellType enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum SpellType {
    Skill = 0,
    Spell = 1,
    Scroll = 2,
    Charges = 3,
    #[default]
    Invalid = 4,
}

/// Magic type for spells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MagicType {
    Fire = 0,
    Lightning = 1,
    Magic = 2,
}

/// Legacy spell type enum (for compatibility with old code)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Spell {
    // Attack spells
    Firebolt,
    ChargedBolt,
    HolyBolt,
    Lightning,
    Fireball,
    FlameWave,
    ChainLightning,
    Inferno,

    // Utility spells
    Heal,
    HealOther,
    TownPortal,
    Phasing,
    Teleport,
    Identify,

    // Defense spells
    ManaShield,
    StoneCurse,
    Telekinesis,

    // Apocalypse-level
    Apocalypse,
}

impl Spell {
    /// Get spell display name
    pub fn name(&self) -> &'static str {
        match self {
            Spell::Firebolt => "Firebolt",
            Spell::ChargedBolt => "Charged Bolt",
            Spell::HolyBolt => "Holy Bolt",
            Spell::Lightning => "Lightning",
            Spell::Fireball => "Fireball",
            Spell::FlameWave => "Flame Wave",
            Spell::ChainLightning => "Chain Lightning",
            Spell::Inferno => "Inferno",
            Spell::Heal => "Heal",
            Spell::HealOther => "Heal Other",
            Spell::TownPortal => "Town Portal",
            Spell::Phasing => "Phasing",
            Spell::Teleport => "Teleport",
            Spell::Identify => "Identify",
            Spell::ManaShield => "Mana Shield",
            Spell::StoneCurse => "Stone Curse",
            Spell::Telekinesis => "Telekinesis",
            Spell::Apocalypse => "Apocalypse",
        }
    }

    /// Get spell description
    pub fn description(&self) -> &'static str {
        match self {
            Spell::Firebolt => "Fires a bolt of flame",
            Spell::ChargedBolt => "Fires multiple lightning bolts",
            Spell::HolyBolt => "Damages undead enemies",
            Spell::Lightning => "A bolt of lightning",
            Spell::Fireball => "Explosive ball of fire",
            Spell::FlameWave => "Wave of flame",
            Spell::ChainLightning => "Lightning jumps between enemies",
            Spell::Inferno => "Stream of fire",
            Spell::Heal => "Restore your health",
            Spell::HealOther => "Heal an ally",
            Spell::TownPortal => "Open portal to town",
            Spell::Phasing => "Short range teleport",
            Spell::Teleport => "Teleport anywhere",
            Spell::Identify => "Identify an item",
            Spell::ManaShield => "Damage absorbed by mana",
            Spell::StoneCurse => "Turn enemy to stone",
            Spell::Telekinesis => "Pick up items at distance",
            Spell::Apocalypse => "Devastating magic attack",
        }
    }

    /// Is this an offensive spell?
    pub fn is_offensive(&self) -> bool {
        matches!(self,
            Spell::Firebolt | Spell::ChargedBolt | Spell::HolyBolt |
            Spell::Lightning | Spell::Fireball | Spell::FlameWave |
            Spell::ChainLightning | Spell::Inferno | Spell::Apocalypse |
            Spell::StoneCurse
        )
    }

    /// Does this spell require a target?
    pub fn requires_target(&self) -> bool {
        matches!(self,
            Spell::Firebolt | Spell::Lightning | Spell::Fireball |
            Spell::ChainLightning | Spell::StoneCurse | Spell::HealOther |
            Spell::HolyBolt | Spell::Telekinesis
        )
    }
}

/// Spell data with costs and effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    pub spell: Spell,
    pub level: i32,        // Player's level in this spell (1-15)
    pub base_mana: i32,    // Base mana cost
    pub min_damage: i32,   // Base minimum damage/effect
    pub max_damage: i32,   // Base maximum damage/effect
    pub required_magic: i32, // Minimum magic stat to learn
    pub required_level: i32, // Character level to learn
}

impl SpellData {
    /// Create a new spell with base stats
    pub fn new(spell: Spell) -> Self {
        let (base_mana, min_dam, max_dam, req_mag, req_lvl) = match spell {
            Spell::Firebolt => (6, 3, 6, 15, 1),
            Spell::ChargedBolt => (6, 1, 2, 25, 1),
            Spell::HolyBolt => (7, 3, 6, 20, 1),
            Spell::Lightning => (10, 4, 10, 30, 4),
            Spell::Fireball => (16, 10, 16, 48, 8),
            Spell::FlameWave => (20, 8, 12, 54, 10),
            Spell::ChainLightning => (30, 8, 16, 54, 10),
            Spell::Inferno => (11, 3, 6, 20, 3),
            Spell::Heal => (8, 0, 0, 17, 1),
            Spell::HealOther => (8, 0, 0, 17, 1),
            Spell::TownPortal => (35, 0, 0, 35, 3),
            Spell::Phasing => (12, 0, 0, 39, 7),
            Spell::Teleport => (35, 0, 0, 105, 14),
            Spell::Identify => (10, 0, 0, 23, 1),
            Spell::ManaShield => (33, 0, 0, 25, 5),
            Spell::StoneCurse => (60, 0, 0, 51, 8),
            Spell::Telekinesis => (15, 0, 0, 33, 5),
            Spell::Apocalypse => (150, 50, 100, 149, 15),
        };

        Self {
            spell,
            level: 0,  // Not learned yet
            base_mana,
            min_damage: min_dam,
            max_damage: max_dam,
            required_magic: req_mag,
            required_level: req_lvl,
        }
    }

    /// Get actual mana cost based on spell level
    pub fn mana_cost(&self) -> i32 {
        // Higher level = more efficient (lower cost)
        let reduction = (self.level - 1).max(0) * 2;
        (self.base_mana - reduction).max(self.base_mana / 2)
    }

    /// Get damage range based on spell level and magic stat
    pub fn damage_range(&self, magic: i32) -> (i32, i32) {
        let level_bonus = self.level * 2;
        let magic_bonus = magic / 4;

        let min = self.min_damage + level_bonus + magic_bonus;
        let max = self.max_damage + level_bonus * 2 + magic_bonus;

        (min, max.max(min))
    }

    /// Get healing amount for heal spells
    pub fn heal_amount(&self, magic: i32) -> i32 {
        // Healing formula similar to original
        let base = magic + self.level * 2;
        base + (base / 4)
    }

    /// Can this spell be learned with given stats?
    pub fn can_learn(&self, magic: i32, char_level: i32) -> bool {
        magic >= self.required_magic && char_level >= self.required_level
    }

    /// Learn spell or increase level
    pub fn learn(&mut self) {
        if self.level < 15 {
            self.level += 1;
        }
    }

    /// Is spell learned?
    pub fn is_learned(&self) -> bool {
        self.level > 0
    }
}

/// Player's spellbook containing all known spells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spellbook {
    pub spells: Vec<SpellData>,
    pub selected_spell: Option<usize>,
    pub mana_shield_active: bool,
}

impl Default for Spellbook {
    fn default() -> Self {
        Self::new()
    }
}

impl Spellbook {
    pub fn new() -> Self {
        // Initialize all spells (unlearned)
        let all_spells = vec![
            Spell::Firebolt,
            Spell::ChargedBolt,
            Spell::HolyBolt,
            Spell::Lightning,
            Spell::Fireball,
            Spell::FlameWave,
            Spell::ChainLightning,
            Spell::Heal,
            Spell::TownPortal,
            Spell::Phasing,
            Spell::Teleport,
            Spell::ManaShield,
            Spell::StoneCurse,
            Spell::Inferno,
            Spell::Apocalypse,
        ];

        Self {
            spells: all_spells.into_iter().map(SpellData::new).collect(),
            selected_spell: None,
            mana_shield_active: false,
        }
    }

    /// Get all learned spells
    pub fn learned_spells(&self) -> Vec<&SpellData> {
        self.spells.iter().filter(|s| s.is_learned()).collect()
    }

    /// Learn a spell by type
    pub fn learn_spell(&mut self, spell: Spell) -> bool {
        if let Some(spell_data) = self.spells.iter_mut().find(|s| s.spell == spell) {
            spell_data.learn();
            if self.selected_spell.is_none() {
                // Auto-select first learned spell
                self.selected_spell = self.spells.iter().position(|s| s.spell == spell);
            }
            return true;
        }
        false
    }

    /// Get currently selected spell
    pub fn current_spell(&self) -> Option<&SpellData> {
        self.selected_spell.and_then(|i| {
            let spell = self.spells.get(i)?;
            if spell.is_learned() { Some(spell) } else { None }
        })
    }

    /// Select next learned spell
    pub fn select_next(&mut self) {
        let learned: Vec<_> = self.spells.iter()
            .enumerate()
            .filter(|(_, s)| s.is_learned())
            .map(|(i, _)| i)
            .collect();

        if learned.is_empty() {
            self.selected_spell = None;
            return;
        }

        let current = self.selected_spell.unwrap_or(0);
        let next_idx = learned.iter()
            .find(|&&i| i > current)
            .or(learned.first())
            .copied();

        self.selected_spell = next_idx;
    }

    /// Select previous learned spell
    pub fn select_prev(&mut self) {
        let learned: Vec<_> = self.spells.iter()
            .enumerate()
            .filter(|(_, s)| s.is_learned())
            .map(|(i, _)| i)
            .collect();

        if learned.is_empty() {
            self.selected_spell = None;
            return;
        }

        let current = self.selected_spell.unwrap_or(0);
        let prev_idx = learned.iter()
            .rev()
            .find(|&&i| i < current)
            .or(learned.last())
            .copied();

        self.selected_spell = prev_idx;
    }
}

/// Spell projectile for animated spell effects
#[derive(Debug, Clone)]
pub struct SpellProjectile {
    pub spell: Spell,
    pub x: f32,
    pub y: f32,
    pub target_x: i32,
    pub target_y: i32,
    pub speed: f32,
    pub damage: i32,
    pub lifetime: f32,
    pub caster_id: u32,  // To avoid self-damage
}

impl SpellProjectile {
    pub fn new(spell: Spell, start_x: i32, start_y: i32, target_x: i32, target_y: i32, damage: i32) -> Self {
        let speed = match spell {
            Spell::Firebolt => 8.0,
            Spell::Lightning => 16.0,
            Spell::Fireball => 6.0,
            Spell::ChargedBolt => 5.0,
            Spell::HolyBolt => 7.0,
            _ => 10.0,
        };

        Self {
            spell,
            x: start_x as f32,
            y: start_y as f32,
            target_x,
            target_y,
            speed,
            damage,
            lifetime: 3.0,
            caster_id: 0,
        }
    }

    /// Update projectile position, returns true if still alive
    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        if self.lifetime <= 0.0 {
            return false;
        }

        let dx = self.target_x as f32 - self.x;
        let dy = self.target_y as f32 - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 0.5 {
            return false;  // Reached target
        }

        let nx = dx / dist;
        let ny = dy / dist;

        self.x += nx * self.speed * dt * 60.0;
        self.y += ny * self.speed * dt * 60.0;

        true
    }

    /// Check if projectile hit a target at given position
    pub fn hit_check(&self, target_x: i32, target_y: i32) -> bool {
        let dx = (self.x - target_x as f32).abs();
        let dy = (self.y - target_y as f32).abs();
        dx < 1.0 && dy < 1.0
    }
}
