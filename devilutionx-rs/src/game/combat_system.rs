//! Combat System - Exact port of DevilutionX Source/player.cpp PlrHitMonst
//!
//! Precise implementation of combat formulas from the original game.

use rand::Rng;
use crate::game::player_dat::{get_class_attributes, get_player_combat_data, HeroClass, PlayerClassFlag};
use crate::game::item_dat::{ItemType, ItemSpecialEffect, ItemSpecialEffectHf};
use crate::game::monster_dat::{MonsterClass, MonsterMode};

/// Damage types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Fire,
    Lightning,
    Magic,
}

/// Result of a combat attack
#[derive(Debug, Clone)]
pub struct CombatResult {
    /// Did the attack hit?
    pub hit: bool,
    /// Damage dealt (in fixed point, shift right 6 to get actual value)
    pub damage: i32,
    /// Was it a critical hit?
    pub critical: bool,
    /// Was the target killed?
    pub killed: bool,
    /// Was the attack blocked?
    pub blocked: bool,
    /// Type of damage dealt
    pub damage_type: DamageType,
    /// Life stolen
    pub life_stolen: i32,
    /// Mana stolen
    pub mana_stolen: i32,
    /// Peril self-damage (Hellfire effect)
    pub peril_self_damage: i32,
}

impl CombatResult {
    pub fn miss() -> Self {
        Self {
            hit: false,
            damage: 0,
            critical: false,
            killed: false,
            blocked: false,
            damage_type: DamageType::Physical,
            life_stolen: 0,
            mana_stolen: 0,
            peril_self_damage: 0,
        }
    }

    /// Get actual damage value (not fixed point)
    pub fn actual_damage(&self) -> i32 {
        self.damage >> 6
    }
}

/// Player combat stats needed for calculations
/// Mirrors relevant fields from C++ Player struct
#[derive(Debug, Clone, Default)]
pub struct PlayerCombatStats {
    /// Character level
    pub level: u8,
    /// Hero class
    pub class: HeroClass,
    /// Current dexterity (with bonuses)
    pub dexterity: i32,
    /// Minimum weapon damage
    pub min_dam: i32,
    /// Maximum weapon damage
    pub max_dam: i32,
    /// Damage bonus percentage
    pub bonus_dam_percent: i32,
    /// Damage bonus flat
    pub bonus_dam_mod: i32,
    /// Damage modifier from stats
    pub damage_mod: i32,
    /// To-hit bonus from items
    pub bonus_to_hit: i32,
    /// Armor class
    pub armor_class: i32,
    /// Armor piercing value
    pub armor_pierce: i32,
    /// Item special effects
    pub item_flags: ItemSpecialEffect,
    /// Hellfire special effects
    pub hf_flags: ItemSpecialEffectHf,
    /// Current hit points
    pub hit_points: i32,
    /// Maximum hit points
    pub max_hp: i32,
    /// Current mana
    pub mana: i32,
    /// Maximum mana
    pub max_mana: i32,
    /// Get hit modifier (damage taken)
    pub get_hit: i32,
    /// Left hand weapon type
    pub left_hand_type: ItemType,
    /// Right hand weapon type
    pub right_hand_type: ItemType,
    /// Fire min damage
    pub fire_min_dam: i32,
    /// Fire max damage
    pub fire_max_dam: i32,
}

impl PlayerCombatStats {
    /// Get melee to-hit value
    /// From Player::GetMeleeToHit() in player.h
    pub fn get_melee_to_hit(&self) -> i32 {
        let combat_data = get_player_combat_data(self.class);
        self.level as i32 + self.dexterity / 2 + self.bonus_to_hit + combat_data.base_melee_to_hit as i32
    }

    /// Get melee to-hit with armor piercing
    /// From Player::GetMeleePiercingToHit() in player.h
    pub fn get_melee_piercing_to_hit(&self, is_hellfire: bool) -> i32 {
        let mut hper = self.get_melee_to_hit();
        // In Hellfire, armor piercing ignores % of enemy armor instead
        if !is_hellfire {
            hper += self.armor_pierce;
        }
        hper
    }

    /// Calculate armor pierce against monster
    /// From Player::CalculateArmorPierce() in player.h
    pub fn calculate_armor_pierce(&self, monster_armor: i32, is_melee: bool, is_hellfire: bool) -> i32 {
        let mut tmac = monster_armor;
        if self.armor_pierce > 0 {
            if is_hellfire {
                let pierce = self.armor_pierce - 1;
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

    /// Get equipped weapon type (Sword or Mace takes priority)
    pub fn get_weapon_type(&self) -> ItemType {
        if self.left_hand_type == ItemType::Sword || self.right_hand_type == ItemType::Sword {
            return ItemType::Sword;
        }
        if self.left_hand_type == ItemType::Mace || self.right_hand_type == ItemType::Mace {
            return ItemType::Mace;
        }
        ItemType::None
    }
}

/// Monster combat stats needed for calculations
#[derive(Debug, Clone, Default)]
pub struct MonsterCombatStats {
    /// Monster's armor class
    pub armor_class: u8,
    /// Monster's current hit points (fixed point, >> 6 for actual)
    pub hit_points: i32,
    /// Monster's maximum hit points
    pub max_hit_points: i32,
    /// Monster's class (Undead, Demon, Animal)
    pub monster_class: MonsterClass,
    /// Monster's current mode
    pub mode: MonsterMode,
    /// Is this a unique monster?
    pub is_unique: bool,
    /// Monster type ID (for Diablo check)
    pub type_id: i32,
}

/// Combat calculator - exact port of PlrHitMonst from player.cpp
pub struct CombatSystem;

impl CombatSystem {
    /// Player melee attack against monster
    /// Exact port of PlrHitMonst() from Source/player.cpp lines 528-705
    pub fn player_hit_monster(
        player: &PlayerCombatStats,
        monster: &MonsterCombatStats,
        adjacent_damage: bool,
        is_hellfire: bool,
        rng: &mut impl Rng,
    ) -> CombatResult {
        // Check if monster can be hit (mode check)
        // From monster.isPossibleToHit()
        if !Self::monster_is_possible_to_hit(monster) {
            return CombatResult::miss();
        }

        // Calculate hit chance penalty for adjacent damage (cleave)
        let mut hper: i32 = 0;
        if adjacent_damage {
            if player.level > 20 {
                hper -= 30;
            } else {
                hper -= (35 - player.level as i32) * 2;
            }
        }

        // Roll to hit
        let mut hit: i32 = rng.gen_range(0..100);

        // Petrified monsters are auto-hit
        if monster.mode == MonsterMode::Petrified {
            hit = 0;
        }

        // Calculate final hit chance
        // hper += player.GetMeleePiercingToHit() - player.CalculateArmorPierce(monster.armorClass, true)
        let piercing_to_hit = player.get_melee_piercing_to_hit(is_hellfire);
        let monster_armor = player.calculate_armor_pierce(monster.armor_class as i32, true, is_hellfire);
        hper += piercing_to_hit - monster_armor;

        // Clamp hit chance to 5-95%
        hper = hper.clamp(5, 95);

        // Check for miss
        if hit >= hper {
            return CombatResult::miss();
        }

        // === DAMAGE CALCULATION ===

        // Base weapon damage
        let mind = player.min_dam;
        let maxd = player.max_dam;
        let mut dam = rng.gen_range(mind..=maxd.max(mind));

        // Apply percentage damage bonus
        dam += dam * player.bonus_dam_percent / 100;

        // Apply flat damage bonus
        dam += player.bonus_dam_mod;

        // Save damage for Peril calculation (before damage_mod)
        let dam2 = dam << 6;

        // Apply damage modifier from stats
        dam += player.damage_mod;

        // Critical strike check (Warrior class ability)
        let class_attrs = get_class_attributes(player.class);
        let mut critical = false;
        if (class_attrs.class_flags & PlayerClassFlag::CriticalStrike as u8) != 0 {
            // Crit chance = character level %
            if rng.gen_range(0..100) < player.level as i32 {
                dam *= 2;
                critical = true;
            }
        }

        // Weapon type vs monster class modifiers
        let weapon_type = player.get_weapon_type();
        match monster.monster_class {
            MonsterClass::Undead => {
                if weapon_type == ItemType::Sword {
                    // Swords deal half damage to undead
                    dam -= dam / 2;
                } else if weapon_type == ItemType::Mace {
                    // Maces deal 50% more damage to undead
                    dam += dam / 2;
                }
            }
            MonsterClass::Animal => {
                if weapon_type == ItemType::Mace {
                    // Maces deal half damage to animals
                    dam -= dam / 2;
                } else if weapon_type == ItemType::Sword {
                    // Swords deal 50% more damage to animals
                    dam += dam / 2;
                }
            }
            MonsterClass::Demon => {
                // Triple damage to demons with special item flag
                if player.item_flags.contains(ItemSpecialEffect::TRIPLE_DEMON_DAMAGE) {
                    dam *= 3;
                }
            }
        }

        // Hellfire: Devastation (5% chance for triple damage)
        if player.hf_flags.contains(ItemSpecialEffectHf::DEVASTATION) {
            if rng.gen_range(0..100) < 5 {
                dam *= 3;
            }
        }

        // Hellfire: Doppelganger (10% chance to summon copy)
        // C++ Reference: player.cpp:615-616
        // Only works on non-unique monsters and not Diablo (type_id == 110)
        if player.hf_flags.contains(ItemSpecialEffectHf::DOPPELGANGER)
            && monster.type_id != 110 // MT_DIABLO
            && !monster.is_unique
            && rng.gen_range(0..100) < 10
        {
            // TODO: AddDoppelganger(monster) - requires monster system integration
            // For now, this is a placeholder that will be implemented when
            // monster spawning system is integrated
        }

        // Convert to fixed point (multiply by 64)
        dam <<= 6;

        // Hellfire: Jester's effect (random damage multiplier)
        if player.hf_flags.contains(ItemSpecialEffectHf::JESTERS) {
            let mut r = rng.gen_range(0..201);
            if r >= 100 {
                r = 100 + (r - 100) * 5;
            }
            dam = dam * r / 100;
        }

        // Adjacent damage (cleave) does 1/4 damage
        if adjacent_damage {
            dam >>= 2;
        }

        // Hellfire: Peril (double damage but take self-damage)
        // C++ Reference: player.cpp:630-640
        let mut peril_self_damage = 0i32;
        if player.hf_flags.contains(ItemSpecialEffectHf::PERIL) {
            // Calculate self-damage using saved base damage (dam2) + get_hit penalty
            peril_self_damage = dam2 + (player.get_hit << 6);

            // Double the damage dealt to monster
            dam *= 2;
        }

        // Calculate life/mana steal
        let mut life_stolen = 0i32;
        let mut mana_stolen = 0i32;

        // Random steal life
        if player.item_flags.contains(ItemSpecialEffect::RANDOM_STEAL_LIFE) {
            life_stolen = rng.gen_range(0..(dam / 8).max(1));
        }

        // Steal mana 3% or 5%
        if !player.item_flags.contains(ItemSpecialEffect::NO_MANA) {
            if player.item_flags.contains(ItemSpecialEffect::STEAL_MANA_3) {
                mana_stolen = 3 * dam / 100;
            }
            if player.item_flags.contains(ItemSpecialEffect::STEAL_MANA_5) {
                mana_stolen = 5 * dam / 100;
            }
        }

        // Steal life 3% or 5%
        if player.item_flags.contains(ItemSpecialEffect::STEAL_LIFE_3) {
            life_stolen = life_stolen.max(3 * dam / 100);
        }
        if player.item_flags.contains(ItemSpecialEffect::STEAL_LIFE_5) {
            life_stolen = life_stolen.max(5 * dam / 100);
        }

        // Check if monster is killed
        let monster_hp_after = monster.hit_points - dam;
        let killed = (monster_hp_after >> 6) <= 0;

        CombatResult {
            hit: true,
            damage: dam,
            critical,
            killed,
            blocked: false,
            damage_type: DamageType::Physical,
            life_stolen,
            mana_stolen,
            peril_self_damage,
        }
    }

    /// Check if monster can be hit
    fn monster_is_possible_to_hit(monster: &MonsterCombatStats) -> bool {
        // Most modes can be hit, some special modes cannot
        !matches!(
            monster.mode,
            MonsterMode::Death | MonsterMode::FadeIn | MonsterMode::FadeOut | MonsterMode::Heal
        )
    }

    /// Monster attack against player
    /// Based on MonsterAttackPlayer logic
    pub fn monster_hit_player(
        monster_to_hit: i32,
        monster_min_dam: i32,
        monster_max_dam: i32,
        player: &PlayerCombatStats,
        can_block: bool,
        rng: &mut impl Rng,
    ) -> CombatResult {
        // Roll to hit
        let hit_roll = rng.gen_range(0..100);

        // Calculate hit chance
        // Monster to-hit vs player armor
        let hit_chance = (monster_to_hit - player.armor_class).clamp(5, 95);

        if hit_roll >= hit_chance {
            return CombatResult::miss();
        }

        // Block check
        if can_block {
            let combat_data = get_player_combat_data(player.class);
            let block_chance = player.dexterity + combat_data.base_to_block as i32 + (player.level as i32) * 2;
            let block_roll = rng.gen_range(0..100);

            if block_roll < block_chance.clamp(0, 100) {
                return CombatResult {
                    hit: true,
                    damage: 0,
                    critical: false,
                    killed: false,
                    blocked: true,
                    damage_type: DamageType::Physical,
                    life_stolen: 0,
                    mana_stolen: 0,
                    peril_self_damage: 0,
                };
            }
        }

        // Calculate damage
        let dam = rng.gen_range(monster_min_dam..=monster_max_dam.max(monster_min_dam));

        // Convert to fixed point
        let mut fixed_dam = dam << 6;

        // Apply player's armor reduction
        // In original, armor reduces damage but minimum 1
        let armor_reduction = (player.armor_class << 6) / 2;
        fixed_dam = (fixed_dam - armor_reduction).max(64); // Minimum 1 damage

        // Check for kill
        let killed = (player.hit_points - fixed_dam) <= 0;

        CombatResult {
            hit: true,
            damage: fixed_dam,
            critical: false,
            killed,
            blocked: false,
            damage_type: DamageType::Physical,
            life_stolen: 0,
            mana_stolen: 0,
            peril_self_damage: 0,
        }
    }

    /// Calculate ranged attack hit chance
    pub fn get_ranged_to_hit(player: &PlayerCombatStats) -> i32 {
        let combat_data = get_player_combat_data(player.class);
        player.level as i32 + player.dexterity + player.bonus_to_hit + combat_data.base_ranged_to_hit as i32
    }

    /// Calculate magic hit chance
    pub fn get_magic_to_hit(player: &PlayerCombatStats, magic_stat: i32) -> i32 {
        let combat_data = get_player_combat_data(player.class);
        magic_stat + combat_data.base_magic_to_hit as i32
    }
}

/// Damage number for visual display
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_hit_chance_clamping() {
        let mut rng = StdRng::seed_from_u64(12345);

        let player = PlayerCombatStats {
            level: 1,
            class: HeroClass::Warrior,
            dexterity: 20,
            min_dam: 2,
            max_dam: 8,
            bonus_to_hit: 0,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 100, // Very high AC
            hit_points: 100 << 6,
            monster_class: MonsterClass::Undead,
            ..Default::default()
        };

        // Even with very high AC, hit chance should be at least 5%
        let mut hits = 0;
        for _ in 0..1000 {
            let result = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);
            if result.hit {
                hits += 1;
            }
        }
        // Should hit roughly 5% of the time (with some variance)
        assert!(hits > 20 && hits < 100, "Hit rate should be around 5%, got {}%", hits / 10);
    }

    #[test]
    fn test_undead_weapon_modifier() {
        let mut rng = StdRng::seed_from_u64(12345);

        // Test mace vs undead (should do 50% more damage)
        let player_mace = PlayerCombatStats {
            level: 10,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 10,
            max_dam: 10, // Fixed damage for testing
            bonus_to_hit: 100, // Guaranteed hit
            left_hand_type: ItemType::Mace,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 1000 << 6,
            monster_class: MonsterClass::Undead,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_mace, &monster, false, false, &mut rng);
        // Base 10 damage + 50% = 15, then << 6 = 960
        assert!(result.hit);
        // Damage should be around 15 << 6 = 960 (may vary due to crit)
        let actual_dam = result.damage >> 6;
        assert!(actual_dam >= 10, "Mace vs undead should do at least base damage, got {}", actual_dam);
    }

    #[test]
    fn test_critical_strike_warrior() {
        let mut rng = StdRng::seed_from_u64(12345);

        // High level warrior has high crit chance
        let player = PlayerCombatStats {
            level: 50, // 50% crit chance
            class: HeroClass::Warrior,
            dexterity: 100,
            min_dam: 10,
            max_dam: 10,
            bonus_to_hit: 100,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            ..Default::default()
        };

        let mut crits = 0;
        for _ in 0..100 {
            let result = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);
            if result.critical {
                crits += 1;
            }
        }
        // Should crit roughly 50% of the time
        assert!(crits > 30 && crits < 70, "Crit rate should be around 50%, got {}%", crits);
    }

    #[test]
    fn test_peril_doubles_damage_and_applies_self_damage() {
        let mut rng = StdRng::seed_from_u64(42);

        let player = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 20,
            max_dam: 20, // Fixed damage
            bonus_dam_percent: 0,
            bonus_dam_mod: 0,
            damage_mod: 0,
            bonus_to_hit: 100, // Guaranteed hit
            get_hit: 10, // Peril self-damage penalty
            hf_flags: ItemSpecialEffectHf::PERIL,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);

        assert!(result.hit, "Peril attack should hit");

        // Base damage is 20, should be doubled by Peril
        // Expected: 20 (base) -> << 6 = 1280, then * 2 = 2560
        let expected_damage = 20 << 6; // Base in fixed point
        let expected_damage_doubled = expected_damage * 2; // Peril doubles

        assert_eq!(result.damage, expected_damage_doubled,
            "Peril should double damage: expected {}, got {}",
            expected_damage_doubled, result.damage);

        // Peril self-damage = dam2 (saved base) + (get_hit << 6)
        // dam2 = 20 << 6 = 1280
        // get_hit << 6 = 10 << 6 = 640
        // Total = 1920
        let expected_self_damage = (20 << 6) + (10 << 6);
        assert_eq!(result.peril_self_damage, expected_self_damage,
            "Peril self-damage should be dam2 + (get_hit << 6): expected {}, got {}",
            expected_self_damage, result.peril_self_damage);
    }

    #[test]
    fn test_peril_with_critical_strike() {
        let mut rng = StdRng::seed_from_u64(999); // Seed that triggers crit

        let player = PlayerCombatStats {
            level: 50, // High crit chance
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 10,
            max_dam: 10,
            bonus_to_hit: 100,
            get_hit: 5,
            hf_flags: ItemSpecialEffectHf::PERIL,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);

        // Peril doubles damage AFTER critical strike
        // If crit: 10 * 2 (crit) = 20, then << 6 = 1280, then * 2 (peril) = 2560
        // If no crit: 10 << 6 = 640, then * 2 (peril) = 1280
        let damage_actual = result.damage >> 6;

        // Should be either 20 (no crit + peril) or 40 (crit + peril)
        assert!(damage_actual == 20 || damage_actual == 40,
            "Peril with possible crit should result in 20 or 40 damage, got {}", damage_actual);
    }

    #[test]
    fn test_doppelganger_conditions() {
        let mut rng = StdRng::seed_from_u64(123);

        // Test that Doppelganger doesn't trigger on Diablo
        let player = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 10,
            max_dam: 10,
            bonus_to_hit: 100,
            hf_flags: ItemSpecialEffectHf::DOPPELGANGER,
            ..Default::default()
        };

        let diablo = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 110, // MT_DIABLO
        };

        // Should not crash or cause issues (Doppelganger skipped for Diablo)
        let result = CombatSystem::player_hit_monster(&player, &diablo, false, false, &mut rng);
        assert!(result.hit, "Should still hit Diablo normally");

        // Test that Doppelganger doesn't trigger on unique monsters
        let unique_monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Undead,
            mode: MonsterMode::Stand,
            is_unique: true, // Unique monster
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player, &unique_monster, false, false, &mut rng);
        assert!(result.hit, "Should still hit unique monster normally");

        // Test normal monster (would trigger AddDoppelganger in C++, but we have placeholder)
        let normal_monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player, &normal_monster, false, false, &mut rng);
        assert!(result.hit, "Should hit normal monster (Doppelganger would trigger 10% of time)");
    }

    #[test]
    fn test_jesters_damage_multiplier() {
        let mut rng = StdRng::seed_from_u64(42);

        let player = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100, // Fixed damage for testing
            bonus_to_hit: 100,
            hf_flags: ItemSpecialEffectHf::JESTERS,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 100000 << 6,
            max_hit_points: 100000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        // Test multiple times to verify variance
        let mut damages = Vec::new();
        for _ in 0..20 {
            let result = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);
            damages.push(result.damage >> 6);
        }

        // Jesters should produce varying damage (0% to 500%)
        let min_damage = *damages.iter().min().unwrap();
        let max_damage = *damages.iter().max().unwrap();

        // Should have significant variance
        assert!(max_damage > min_damage * 2,
            "Jesters should create damage variance, min={} max={}", min_damage, max_damage);
    }

    #[test]
    fn test_devastation_triple_damage() {
        let mut rng = StdRng::seed_from_u64(999);

        let player = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 10,
            max_dam: 10,
            bonus_to_hit: 100,
            hf_flags: ItemSpecialEffectHf::DEVASTATION,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        // Test 100 times to verify ~5% trigger rate
        let mut triple_count = 0;
        for _ in 0..100 {
            let result = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);
            let damage = result.damage >> 6;

            // Base is 10, triple is 30 (before fixed point)
            if damage >= 28 && damage <= 32 { // Allow some variance
                triple_count += 1;
            }
        }

        // Should trigger roughly 5% of the time (2-10% with variance)
        assert!(triple_count >= 2 && triple_count <= 10,
            "Devastation should trigger ~5% of time, got {}%", triple_count);
    }

    #[test]
    fn test_life_steal_calculation() {
        let mut rng = StdRng::seed_from_u64(42);

        // Test RandomStealLife
        let player_random = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 80,
            max_dam: 80, // Fixed damage = 80 << 6 = 5120 fixed point
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::RANDOM_STEAL_LIFE,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player_random, &monster, false, false, &mut rng);

        // RandomStealLife: 0 to dam/8
        // dam = 80 << 6 = 5120, so max steal = 5120 / 8 = 640
        assert!(result.life_stolen >= 0 && result.life_stolen < 640,
            "RandomStealLife should be 0 to dam/8, got {}", result.life_stolen);

        // Test StealLife3
        let player_3 = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::STEAL_LIFE_3,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_3, &monster, false, false, &mut rng);
        let expected_steal = 3 * (100 << 6) / 100; // 3% of damage
        assert_eq!(result.life_stolen, expected_steal,
            "StealLife3 should be 3% of damage");

        // Test StealLife5
        let player_5 = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::STEAL_LIFE_5,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_5, &monster, false, false, &mut rng);
        let expected_steal = 5 * (100 << 6) / 100; // 5% of damage
        assert_eq!(result.life_stolen, expected_steal,
            "StealLife5 should be 5% of damage");
    }

    #[test]
    fn test_mana_steal_calculation() {
        let mut rng = StdRng::seed_from_u64(42);

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 10000 << 6,
            max_hit_points: 10000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        // Test StealMana3
        let player_3 = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::STEAL_MANA_3,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_3, &monster, false, false, &mut rng);
        let expected_steal = 3 * (100 << 6) / 100;
        assert_eq!(result.mana_stolen, expected_steal,
            "StealMana3 should be 3% of damage");

        // Test StealMana5
        let player_5 = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::STEAL_MANA_5,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_5, &monster, false, false, &mut rng);
        let expected_steal = 5 * (100 << 6) / 100;
        assert_eq!(result.mana_stolen, expected_steal,
            "StealMana5 should be 5% of damage");

        // Test NO_MANA flag blocks mana steal
        let player_no_mana = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::STEAL_MANA_5 | ItemSpecialEffect::NO_MANA,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_no_mana, &monster, false, false, &mut rng);
        assert_eq!(result.mana_stolen, 0,
            "NO_MANA flag should block mana steal");
    }

    #[test]
    fn test_weapon_type_modifiers() {
        let mut rng = StdRng::seed_from_u64(42);

        // Test Mace vs Undead (+50%)
        let player_mace = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            left_hand_type: ItemType::Mace,
            ..Default::default()
        };

        let undead = MonsterCombatStats {
            armor_class: 0,
            hit_points: 100000 << 6,
            max_hit_points: 100000 << 6,
            monster_class: MonsterClass::Undead,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player_mace, &undead, false, false, &mut rng);
        let damage = result.damage >> 6;

        // Base 100, +50% = 150
        assert_eq!(damage, 150, "Mace vs Undead should deal 150% damage");

        // Test Sword vs Undead (-50%)
        let player_sword = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            left_hand_type: ItemType::Sword,
            ..Default::default()
        };

        let result = CombatSystem::player_hit_monster(&player_sword, &undead, false, false, &mut rng);
        let damage = result.damage >> 6;

        // Base 100, -50% = 50
        assert_eq!(damage, 50, "Sword vs Undead should deal 50% damage");

        // Test triple demon damage
        let player_demon = PlayerCombatStats {
            level: 20,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            item_flags: ItemSpecialEffect::TRIPLE_DEMON_DAMAGE,
            ..Default::default()
        };

        let demon = MonsterCombatStats {
            armor_class: 0,
            hit_points: 100000 << 6,
            max_hit_points: 100000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        let result = CombatSystem::player_hit_monster(&player_demon, &demon, false, false, &mut rng);
        let damage = result.damage >> 6;

        // Base 100, *3 = 300
        assert_eq!(damage, 300, "Triple demon damage should deal 300% damage");
    }

    #[test]
    fn test_adjacent_damage_penalty() {
        let mut rng = StdRng::seed_from_u64(42);

        let player = PlayerCombatStats {
            level: 25,
            class: HeroClass::Warrior,
            dexterity: 50,
            min_dam: 100,
            max_dam: 100,
            bonus_to_hit: 100,
            ..Default::default()
        };

        let monster = MonsterCombatStats {
            armor_class: 0,
            hit_points: 100000 << 6,
            max_hit_points: 100000 << 6,
            monster_class: MonsterClass::Demon,
            mode: MonsterMode::Stand,
            is_unique: false,
            type_id: 50,
        };

        // Normal hit
        let result_normal = CombatSystem::player_hit_monster(&player, &monster, false, false, &mut rng);
        let damage_normal = result_normal.damage >> 6;

        // Adjacent hit (cleave)
        let result_adjacent = CombatSystem::player_hit_monster(&player, &monster, true, false, &mut rng);
        let damage_adjacent = result_adjacent.damage >> 6;

        // Adjacent damage should be 1/4 of normal
        assert_eq!(damage_adjacent, damage_normal / 4,
            "Adjacent damage should be 1/4, got {} vs {}", damage_adjacent, damage_normal);
    }
}
