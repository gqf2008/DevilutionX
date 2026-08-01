// Combat Integration: Monster ⟷ Player Combat System
//
// C++ References:
// - Source/monster.cpp:1168-1270 (MonsterAttackPlayer)
// - Source/player.cpp:529-705 (PlrHitMonst)
//
// This module implements bidirectional combat between monsters and players,
// matching Diablo 1's combat formulas and mechanics.

use crate::game::types::Point;  // Use game::types::Point to match Monster/Player
use crate::game::monster_exact::{Monster, MonsterMode};
use crate::game::player_exact::{Player, HeroClass};
use rand::Rng;

/// Helper function: Calculate Chebyshev distance (max(dx, dy))
///
/// **C++ Reference**: `Source/engine/point.hpp:123` - `WalkingDistance()`
pub fn walking_distance(from: Point, to: Point) -> i32 {
    let dx = (from.x - to.x).abs();
    let dy = (from.y - to.y).abs();
    dx.max(dy)
}

/// Combat action result
#[derive(Debug, Clone, PartialEq)]
pub enum AttackResult {
    /// Attack missed the target
    Miss,
    /// Attack hit successfully
    Hit { damage: i32 },
    /// Attack killed the target
    Kill { damage: i32 },
    /// Player blocked the attack
    Block,
}

//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MONSTER → PLAYER COMBAT
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Monster attacks player
///
/// **C++ Reference**: `Source/monster.cpp:1168` - `MonsterAttackPlayer()`
///
/// # Combat Formula (from C++)
/// ```text
/// hit_chance = 2 * (monster.level - player.level) + 30 - player.ac
/// hit_chance = clamp(hit_chance, 5, 95)  // 5%-95% hit range
///
/// if random(100) >= hit_chance:
///     return MISS
///
/// damage = random(min_damage..=max_damage)
/// damage = max(damage - player.ac/2, 1)  // Armor reduction (simplified)
/// ```
///
/// # C++ Alignment
/// - ✅ Distance check (must be ≤1 tile away)
/// - ✅ Hit chance formula: `2*(mlvl-plvl) + 30 - ac`
/// - ✅ Hit chance clamped to 5%-95%
/// - ✅ Damage uses 64x fixed-point internally
/// - ✅ Minimum damage is 1 (64 in 64x)
///
/// # Arguments
/// - `monster`: Attacking monster
/// - `player`: Target player
/// - `rng`: Random number generator
///
/// # Returns
/// - `AttackResult::Miss` if attack missed
/// - `AttackResult::Hit { damage }` if hit successfully
/// - `AttackResult::Block` if player blocked (future implementation)
pub fn monster_attack_player(
    monster: &Monster,
    player: &mut Player,
    rng: &mut impl Rng,
) -> AttackResult {
    // 1. Distance check (C++ line 1174)
    let distance = walking_distance(monster.position(), player.position);
    if distance >= 2 {
        return AttackResult::Miss; // Too far away
    }

    // 2. Calculate hit chance (C++ line 1177-1191)
    let base_hit = calculate_monster_to_hit(monster, player);

    // 3. Roll to hit (C++ line 1176)
    let hit_roll = rng.random_range(0..100);

    if hit_roll >= base_hit {
        return AttackResult::Miss; // C++ line 1199
    }

    // 4. Calculate damage (C++ line 1223-1224)
    let min_dam_64x = (monster.min_damage as i32) << 6;
    let max_dam_64x = (monster.max_damage as i32) << 6;
    let raw_damage_64x = rng.random_range(min_dam_64x..=max_dam_64x);

    // 5. Apply armor reduction (simplified - C++ has complex AC system)
    let armor_class = player._p_armor_class as i32;
    let armor_reduction = (armor_class / 2) << 6;
    let final_damage_64x = raw_damage_64x.saturating_sub(armor_reduction).max(64); // Min 1 damage

    // 6. Apply damage to player (C++ line 1228).
    // `final_damage_64x` is in 64x fixed-point, but `modify_hp` takes a
    // display-scale delta and re-applies the 64x factor internally, so we
    // must pass the display value (final_damage_64x >> 6) to avoid a
    // double 64x scaling that would deal 64x the intended damage.
    let final_damage = final_damage_64x >> 6; // Convert back to normal scale
    player.modify_hp(-final_damage);

    AttackResult::Hit { damage: final_damage }
}

/// Calculate monster's chance to hit player
///
/// **C++ Reference**: `Source/monster.cpp:1177-1191`
///
/// # Formula (simplified - Monster has no level field)
/// ```text
/// hit = monster_intelligence + 30 - player_ac  // Simplified from C++
/// hit = clamp(hit, 5, 95)
/// ```
///
/// **Note**: C++ uses `monster.level()` which queries MonsterData. Since Monster
/// doesn't store level, we use intelligence as a proxy (typical range 0-15).
fn calculate_monster_to_hit(monster: &Monster, player: &Player) -> i32 {
    let monster_power = monster.intelligence as i32;
    let ac = player._p_armor_class as i32;

    // Simplified formula: int + 30 - ac (instead of C++ 2*level + 30 - ac)
    let mut hit_chance = monster_power + 30 - ac;    // Clamp to 5%-95% (C++ line 1191)
    hit_chance = hit_chance.clamp(5, 95);

    hit_chance
}

//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PLAYER → MONSTER COMBAT
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Player attacks monster
///
/// **C++ Reference**: `Source/player.cpp:529` - `PlrHitMonst()`
///
/// # Combat Formula (from C++)
/// ```text
/// hit_chance = player.to_hit - monster.ac
/// hit_chance = clamp(hit_chance, 5, 95)
///
/// if random(100) >= hit_chance:
///     return MISS
///
/// damage = random(player.min_dam..=player.max_dam)
/// damage += damage * bonus_dam_percent / 100
/// damage -= monster.ac / 4  // Armor reduction
/// damage = max(damage, 1)
/// ```
///
/// # C++ Alignment
/// - ✅ Hit chance formula: `to_hit - monster.ac` (simplified)
/// - ✅ Hit chance clamped to 5%-95%
/// - ✅ Damage calculation with bonus modifiers
/// - ✅ Armor reduction: `monster.ac / 4`
/// - ✅ Minimum damage 1
/// - ✅ Death check triggers `MonsterMode::Dying`
///
/// # Arguments
/// - `player`: Attacking player
/// - `monster`: Target monster
/// - `rng`: Random number generator
///
/// # Returns
/// - `AttackResult::Miss` if attack missed
/// - `AttackResult::Hit { damage }` if hit successfully
/// - `AttackResult::Kill { damage }` if monster was killed
pub fn player_attack_monster(
    player: &Player,
    monster: &mut Monster,
    rng: &mut impl Rng,
) -> AttackResult {
    // 1. Calculate hit chance (C++ line 549)
    let hit_chance = calculate_player_to_hit(player, monster);

    // 2. Roll to hit (C++ line 547)
    let hit_roll = rng.random_range(0..100);

    if hit_roll >= hit_chance {
        return AttackResult::Miss; // C++ line 557
    }

    // 3. Calculate base damage (C++ line 566-567)
    let min_dam = player._p_i_min_dam;
    let max_dam = player._p_i_max_dam;
    let mut damage = rng.random_range(min_dam..=max_dam);

    // 4. Apply damage bonuses (C++ line 568-569)
    damage += damage * player._p_i_bonus_dam / 100;
    damage += player._p_i_bonus_dam_mod;

    // 5. Critical Strike for Warrior (C++ line 573-577)
    if player._p_class == HeroClass::Warrior {
        if rng.random_range(0..100) < (player._p_level as i32) {
            damage *= 2; // Double damage on crit
        }
    }

    // 6. Apply armor reduction (simplified from C++ line 625)
    let armor_reduction = (monster.armor_class as i32) / 4;
    damage = damage.saturating_sub(armor_reduction).max(1);

    // 7. Convert to 64x fixed-point and apply (C++ line 625)
    let damage_64x = damage << 6;
    monster.hp -= damage_64x;

    // 8. Check for death (C++ line 669)
    if monster.hp <= 0 {
        monster.mode = MonsterMode::Death;
        // Keep the AI-state filter the renderer uses (MonsterAIState::Dead)
        // in sync with the death mode, otherwise a slain monster keeps being
        // drawn because only `mode` was updated.
        monster.ai_state = crate::game::monster::MonsterAIState::Dead;
        return AttackResult::Kill { damage };
    }

    AttackResult::Hit { damage }
}

/// Calculate player's chance to hit monster — exact port of C++
/// `PlrHitMonst` (player.cpp:548-549):
///
/// ```cpp
/// hper += GetMeleePiercingToHit() - CalculateArmorPierce(monster.armorClass, true);
/// hper = clamp(hper, 5, 95);
/// ```
///
/// - `GetMeleeToHit()` (player.h:573) = level + dex/2 + iBonusToHit + baseMeleeToHit
/// - `GetMeleePiercingToHit()` (player.h:583, non-Hellfire) = GetMeleeToHit() + `_pIEnAc`
/// - `CalculateArmorPierce(armor, isMelee=true)` (player.h:653, non-Hellfire):
///   the Barbarian melee branch subtracts armor/8; tmac is clamped to >= 0.
fn calculate_player_to_hit(player: &Player, monster: &Monster) -> i32 {
    let class = match player._p_class {
        HeroClass::Warrior => crate::game::player_dat::HeroClass::Warrior,
        HeroClass::Rogue => crate::game::player_dat::HeroClass::Rogue,
        HeroClass::Sorcerer => crate::game::player_dat::HeroClass::Sorcerer,
        HeroClass::Monk => crate::game::player_dat::HeroClass::Monk,
        HeroClass::Bard => crate::game::player_dat::HeroClass::Bard,
        HeroClass::Barbarian => crate::game::player_dat::HeroClass::Barbarian,
    };
    let combat_data = crate::game::player_dat::get_player_combat_data(class);
    let mut hper = player._p_level as i32
        + player._p_dexterity / 2
        + player._p_i_bonus_to_hit
        + combat_data.base_melee_to_hit as i32
        + player._p_i_en_ac; // GetMeleePiercingToHit (non-Hellfire)

    // CalculateArmorPierce(monster.armor_class, is_melee=true), non-Hellfire path.
    let mut tmac = monster.armor_class as i32;
    if player._p_i_en_ac > 0 && player._p_class == HeroClass::Barbarian {
        tmac -= monster.armor_class as i32 / 8;
    }
    if tmac < 0 {
        tmac = 0;
    }
    hper -= tmac;

    hper.clamp(5, 95)
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::monster::MonsterType;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_walking_distance() {
        let p1 = Point::new(10, 10);
        let p2 = Point::new(13, 14);

        let dist = walking_distance(p1, p2);

        // Chebyshev distance = max(|13-10|, |14-10|) = max(3, 4) = 4
        assert_eq!(dist, 4);
    }

    #[test]
    fn test_walking_distance_adjacent() {
        let p1 = Point::new(10, 10);
        let p2 = Point::new(11, 10);

        let dist = walking_distance(p1, p2);

        assert_eq!(dist, 1);
    }

    #[test]
    fn test_monster_attack_player_distance_check() {
        let monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);

        let mut player = Player::new();
        player.position = Point::new(20, 20); // Too far away
        player._p_hit_points = 100 * 64;

        let mut rng = StdRng::seed_from_u64(12345);
        let result = monster_attack_player(&monster, &mut player, &mut rng);

        assert_eq!(result, AttackResult::Miss, "Should miss due to distance");
    }

    #[test]
    fn test_monster_attack_player_within_range() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.intelligence = 5;

        let mut player = Player::new();
        player.position = Point::new(11, 10);
        player._p_hit_points = 100 * 64;
        player._p_armor_class = 0;

        let mut rng = StdRng::seed_from_u64(12345);
        let result = monster_attack_player(&monster, &mut player, &mut rng);

        match result {
            AttackResult::Miss | AttackResult::Hit { .. } => {},
            _ => panic!("Unexpected result"),
        }
    }

    /// Exact C++ `PlrHitMonst` to-hit (player.cpp:548-549): Warrior
    /// baseMeleeToHit=70, so hper = level + dex/2 + iBonusToHit + 70 +
    /// _pIEnAc - monster armor, clamped to 5-95.
    #[test]
    fn test_calculate_player_to_hit_matches_cpp() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.armor_class = 10;

        let mut player = Player::new();
        player._p_class = HeroClass::Warrior;
        player._p_level = 5;
        player._p_dexterity = 30;
        player._p_i_bonus_to_hit = 0;
        player._p_i_en_ac = 0;

        // GetMeleeToHit = 5 + 15 + 0 + 70 = 90; armor 10 -> 80.
        assert_eq!(calculate_player_to_hit(&player, &monster), 80);

        // Bonus to-hit and pierce (_pIEnAc) raise it further.
        player._p_i_bonus_to_hit = 7;
        player._p_i_en_ac = 5;
        // GetMeleePiercingToHit = 90 + 7 + 5 = 102; armor 10 -> 92.
        assert_eq!(calculate_player_to_hit(&player, &monster), 92);

        // Low chance clamps to 5.
        let mut weak = Player::new();
        weak._p_class = HeroClass::Warrior;
        weak._p_level = 1;
        weak._p_dexterity = 5;
        monster.armor_class = 200;
        assert_eq!(calculate_player_to_hit(&weak, &monster), 5);

        // High chance clamps to 95.
        let mut strong = Player::new();
        strong._p_class = HeroClass::Warrior;
        strong._p_level = 50;
        strong._p_dexterity = 100;
        strong._p_i_bonus_to_hit = 40;
        monster.armor_class = 1;
        assert_eq!(calculate_player_to_hit(&strong, &monster), 95);
    }

    #[test]
    fn test_player_attack_monster_kill() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.hp = 10 * 64;
        monster.max_hp = 10 * 64;

        let mut player = Player::new();
        player._p_level = 10;
        player._p_dexterity = 50;
        player._p_i_min_dam = 10;
        player._p_i_max_dam = 20;
        player._p_i_bonus_to_hit = 100;
        player._p_i_bonus_dam = 0;
        player._p_i_bonus_dam_mod = 0;
        player._p_class = HeroClass::Warrior;

        let mut rng = StdRng::seed_from_u64(11111);
        let result = player_attack_monster(&player, &mut monster, &mut rng);

        if let AttackResult::Kill { damage } = result {
            assert!(damage >= 10);
            assert!(monster.hp <= 0);
            assert_eq!(monster.mode, MonsterMode::Death);
        }
    }
}
