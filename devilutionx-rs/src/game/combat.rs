//! Combat System - Damage calculation, attacks, and death handling

use rand::Rng;
use super::player::{Player, PlayerClass};
use super::monster::Monster;

/// Damage types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Fire,
    Lightning,
    Magic,
    Holy,
}

/// Attack result
#[derive(Debug, Clone)]
pub struct AttackResult {
    pub hit: bool,
    pub damage: i32,
    pub critical: bool,
    pub killed: bool,
    pub blocked: bool,
    pub damage_type: DamageType,
}

impl AttackResult {
    pub fn miss() -> Self {
        Self {
            hit: false,
            damage: 0,
            critical: false,
            killed: false,
            blocked: false,
            damage_type: DamageType::Physical,
        }
    }
}

/// Combat calculator
pub struct Combat;

impl Combat {
    /// Calculate player attack against monster
    pub fn player_attack_monster(player: &Player, monster: &mut Monster, rng: &mut impl Rng) -> AttackResult {
        // Hit chance calculation (base 50% + dex bonus + to_hit - monster evasion)
        let hit_chance = 50 + player.stats.dexterity / 2 + player.stats.to_hit - monster.evasion;
        let hit_roll = rng.gen_range(0..100);

        if hit_roll >= hit_chance.clamp(5, 95) {
            return AttackResult::miss();
        }

        // Damage calculation
        let base_damage = Self::calculate_player_damage(player, rng);

        // Critical hit (based on dex)
        let crit_chance = 5 + player.stats.dexterity / 10;
        let critical = rng.gen_range(0..100) < crit_chance;
        let damage = if critical { base_damage * 2 } else { base_damage };

        // Apply damage
        let actual_damage = (damage - monster.armor).max(1);
        monster.hp -= actual_damage;
        let killed = monster.hp <= 0;

        AttackResult {
            hit: true,
            damage: actual_damage,
            critical,
            killed,
            blocked: false,
            damage_type: DamageType::Physical,
        }
    }

    /// Calculate player base damage
    fn calculate_player_damage(player: &Player, rng: &mut impl Rng) -> i32 {
        let str_bonus = player.stats.strength / 5;
        let weapon_damage = player.stats.damage.max(1);

        // Class-specific bonuses
        let class_multiplier = match player.class {
            PlayerClass::Warrior => 1.2,
            PlayerClass::Barbarian => 1.3,
            PlayerClass::Rogue => 1.0,
            PlayerClass::Monk => 1.1,
            _ => 0.8,
        };

        let base = (weapon_damage + str_bonus) as f32 * class_multiplier;
        let variance = rng.gen_range(0.8..1.2);

        (base * variance) as i32
    }

    /// Calculate monster attack against player
    pub fn monster_attack_player(monster: &Monster, player: &mut Player, rng: &mut impl Rng) -> AttackResult {
        // Hit chance
        let hit_chance = 50 + monster.to_hit - player.stats.dexterity / 3;
        let hit_roll = rng.gen_range(0..100);

        if hit_roll >= hit_chance.clamp(5, 95) {
            return AttackResult::miss();
        }

        // Block check (warriors/monks have better block)
        let block_chance = match player.class {
            PlayerClass::Warrior => player.stats.dexterity / 4,
            PlayerClass::Monk => player.stats.dexterity / 3,
            _ => player.stats.dexterity / 6,
        };

        if rng.gen_range(0..100) < block_chance {
            return AttackResult {
                hit: true,
                damage: 0,
                critical: false,
                killed: false,
                blocked: true,
                damage_type: DamageType::Physical,
            };
        }

        // Damage calculation
        let base_damage = monster.damage + rng.gen_range(0..monster.damage / 2 + 1);
        let actual_damage = (base_damage - player.stats.armor_class).max(1);

        player.stats.hp -= actual_damage;
        player.hp = player.stats.hp;
        let killed = player.stats.hp <= 0;

        if killed {
            player.is_dead = true;
        }

        AttackResult {
            hit: true,
            damage: actual_damage,
            critical: false,
            killed,
            blocked: false,
            damage_type: DamageType::Physical,
        }
    }

    /// Calculate spell damage
    pub fn spell_damage(player: &Player, spell_level: i32, base_damage: i32, damage_type: DamageType, rng: &mut impl Rng) -> i32 {
        let magic_bonus = player.stats.magic / 4;
        let level_bonus = spell_level * 2;

        let base = base_damage + magic_bonus + level_bonus;
        let variance = rng.gen_range(0.9..1.1);

        (base as f32 * variance) as i32
    }

    /// Calculate healing amount
    pub fn heal_amount(player: &Player, spell_level: i32, base_heal: i32, rng: &mut impl Rng) -> i32 {
        let magic_bonus = player.stats.magic / 5;
        let level_bonus = spell_level * 3;

        let base = base_heal + magic_bonus + level_bonus;
        let variance = rng.gen_range(0.9..1.1);

        (base as f32 * variance) as i32
    }

    /// Apply resistance to damage
    pub fn apply_resistance(damage: i32, resistance: i32) -> i32 {
        let reduction = (damage as f32 * (resistance as f32 / 100.0)) as i32;
        (damage - reduction).max(0)
    }

    /// Calculate experience gained from killing monster
    pub fn experience_gained(monster: &Monster, player_level: u32) -> u32 {
        let base_exp = monster.experience;
        let level_diff = monster.level as i32 - player_level as i32;

        // Bonus/penalty based on level difference
        let multiplier = match level_diff {
            d if d >= 5 => 2.0,
            d if d >= 3 => 1.5,
            d if d >= 0 => 1.0,
            d if d >= -3 => 0.75,
            d if d >= -5 => 0.5,
            _ => 0.25,
        };

        (base_exp as f32 * multiplier) as u32
    }
}

/// Damage number display (for UI)
#[derive(Debug, Clone)]
pub struct DamageNumber {
    pub value: i32,
    pub x: f32,
    pub y: f32,
    pub velocity_y: f32,
    pub lifetime: f32,
    pub is_critical: bool,
    pub is_heal: bool,
}

impl DamageNumber {
    pub fn new(value: i32, x: i32, y: i32, is_critical: bool, is_heal: bool) -> Self {
        Self {
            value,
            x: x as f32,
            y: y as f32,
            velocity_y: -2.0,
            lifetime: 1.0,
            is_critical,
            is_heal,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.y += self.velocity_y;
        self.velocity_y += 0.1; // Gravity
        self.lifetime -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}
