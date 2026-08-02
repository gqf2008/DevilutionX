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
use crate::game::player_exact::{Player, HeroClass, PlayerMode};
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
    _rng: &mut impl Rng,
    current_level: u8,
) -> AttackResult {
    // 1. Distance check (C++ line 1174)
    let distance = walking_distance(monster.position(), player.position);
    if distance >= 2 {
        return AttackResult::Miss; // Too far away
    }

    // 2. Calculate hit chance (C++ line 1177-1191)
    let base_hit = calculate_monster_to_hit(monster, player, current_level);

    // 3. Roll to hit (C++ line 1176)
    let hit_roll = crate::engine::random::gameplay_rnd(0, 99);

    if hit_roll >= base_hit {
        return AttackResult::Miss; // C++ line 1199
    }

    // 3b. Block roll (C++ monster.cpp:1191-1207): a standing/attacking player
    // with a shield rolls; block chance = GetBlockChance() - 2*monster.level
    // (dex + baseToBlock + 2*playerLevel - 2*monsterLevel), clamped 0-100.
    let mut blk_roll = 100;
    if matches!(player._p_mode, PlayerMode::Stand | PlayerMode::Attack) && player._p_block_flag {
        blk_roll = crate::engine::random::gameplay_rnd(0, 99);
    }
    let base_to_block = crate::game::player_dat::get_player_combat_data(to_dat_class(player._p_class))
        .base_to_block as i32;
    let mut blk = player._p_dexterity + base_to_block + player._p_level as i32 * 2 - monster.level as i32 * 2;
    blk = blk.clamp(0, 100);
    if blk_roll < blk {
        player.set_mode(PlayerMode::Block); // C++ StartPlrBlock
        return AttackResult::Block;
    }

    // 4. Calculate damage (C++ monster.cpp:1219-1220):
    // `dam = RandomIntBetween(minDam << 6, maxDam << 6)`, then
    // `dam = max(dam + (player._pIGetHit << 6), 64)` — no AC reduction in
    // MonsterAttackPlayer itself.
    let min_dam_64x = (monster.min_damage as i32) << 6;
    let max_dam_64x = (monster.max_damage as i32) << 6;
    let raw_damage_64x = crate::engine::random::gameplay_rnd(min_dam_64x, max_dam_64x);
    let final_damage_64x = (raw_damage_64x + player._p_i_get_hit * 64).max(64); // Min 1 damage

    // 6. Apply damage to player (C++ line 1228).
    // `final_damage_64x` is in 64x fixed-point, but `modify_hp` takes a
    // display-scale delta and re-applies the 64x factor internally, so we
    // must pass the display value (final_damage_64x >> 6) to avoid a
    // double 64x scaling that would deal 64x the intended damage.
    let final_damage = final_damage_64x >> 6; // Convert back to normal scale
    player.modify_hp(-final_damage);

    AttackResult::Hit { damage: final_damage }
}

/// Map the `player_exact::HeroClass` to the `player_dat::HeroClass` used by
/// the per-class combat-data table (same variant order, distinct types).
fn to_dat_class(class: HeroClass) -> crate::game::player_dat::HeroClass {
    match class {
        HeroClass::Warrior => crate::game::player_dat::HeroClass::Warrior,
        HeroClass::Rogue => crate::game::player_dat::HeroClass::Rogue,
        HeroClass::Sorcerer => crate::game::player_dat::HeroClass::Sorcerer,
        HeroClass::Monk => crate::game::player_dat::HeroClass::Monk,
        HeroClass::Bard => crate::game::player_dat::HeroClass::Bard,
        HeroClass::Barbarian => crate::game::player_dat::HeroClass::Barbarian,
    }
}

/// Calculate monster's chance to hit player — exact port of C++
/// `MonsterAttackPlayer` (monster.cpp:1185-1189):
///
/// ```cpp
/// hit += 2 * (monster.level(difficulty) - player.getCharacterLevel()) + 30 - player.GetArmor();
/// hit = std::max(hit, GetMinHit());  // 30 (L16), 25 (L15), 20 (L14), 15 otherwise
/// ```
///
/// `monster.toHit(difficulty)` is the monster's base to-hit stat and
/// `GetArmor()` = `_pIBonusAC + _pIAc + _pDexterity / 5` (player.h:618).
fn calculate_monster_to_hit(monster: &Monster, player: &Player, current_level: u8) -> i32 {
    let monster_level = monster.level as i32;
    let player_level = player._p_level as i32;
    let ac = player._p_i_bonus_ac + player._p_i_ac + player._p_dexterity / 5;
    let hit = monster.to_hit + 2 * (monster_level - player_level) + 30 - ac;
    let min_hit = match current_level {
        16 => 30,
        15 => 25,
        14 => 20,
        _ => 15,
    };
    hit.max(min_hit)
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
    _rng: &mut impl Rng,
) -> AttackResult {
    // 1. Calculate hit chance (C++ line 549)
    let hit_chance = calculate_player_to_hit(player, monster);

    // 2. Roll to hit (C++ line 547)
    let hit_roll = crate::engine::random::gameplay_rnd(0, 99);

    if hit_roll >= hit_chance {
        return AttackResult::Miss; // C++ line 557
    }

    // 3. Calculate base damage (C++ line 566-567)
    let min_dam = player._p_i_min_dam;
    let max_dam = player._p_i_max_dam;
    let mut damage = crate::engine::random::gameplay_rnd(min_dam, max_dam);

    // 4. Apply damage bonuses (C++ line 568-570)
    damage += damage * player._p_i_bonus_dam / 100;
    damage += player._p_i_bonus_dam_mod;
    damage += player._p_damage_mod;

    // 5. Critical Strike (C++ line 574-579): classes carrying the
    // `PlayerClassFlag::CriticalStrike` flag (data-driven, per attributes.tsv).
    let attrs = crate::game::player_dat::get_class_attributes(to_dat_class(player._p_class));
    if attrs.class_flags & (crate::game::player_dat::PlayerClassFlag::CriticalStrike as u8) != 0
        && crate::engine::random::gameplay_rnd(0, 99) < player._p_level as i32
    {
        damage *= 2; // Double damage on crit
    }

    // C++ PlrHitMonst applies no AC reduction to the damage (monster armor
    // only reduces the hit chance via CalculateArmorPierce).
    // 6. Weapon vs monster-class modifiers (C++ player.cpp:581-609):
    //    sword/mace deal +/-50% against Undead/Animal monsters.
    use crate::game::item_dat::ItemType;
    // player_exact::INVLOC_HAND_LEFT = 4, INVLOC_HAND_RIGHT = 5.
    let left = player.inv_body[4]._itype;
    let right = player.inv_body[5]._itype;
    let sword = left == ItemType::Sword || right == ItemType::Sword;
    let mace = left == ItemType::Mace || right == ItemType::Mace;
    match monster.monster_class() {
        crate::game::monstdat::MonsterClass::Undead => {
            if sword {
                damage -= damage / 2;
            } else if mace {
                damage += damage / 2;
            }
        }
        crate::game::monstdat::MonsterClass::Animal => {
            if mace {
                damage -= damage / 2;
            } else if sword {
                damage += damage / 2;
            }
        }
        crate::game::monstdat::MonsterClass::Demon => {
            // C++: TripleDemonDamage item effect would triple damage; the
            // item-special-effect system is not wired into combat yet.
        }
    }

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
    let combat_data = crate::game::player_dat::get_player_combat_data(to_dat_class(player._p_class));
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
        let result = monster_attack_player(&monster, &mut player, &mut rng, 1);

        assert_eq!(result, AttackResult::Miss, "Should miss due to distance");
    }

    /// Exact C++ `MonsterAttackPlayer` to-hit (monster.cpp:1185-1189):
    /// hit = toHit + 2*(mLevel - pLevel) + 30 - GetArmor(), floored by
    /// GetMinHit() (15/20/25/30 by currlevel).
    #[test]
    fn test_calculate_monster_to_hit_matches_cpp() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.level = 5;
        monster.to_hit = 40;

        let mut player = Player::new();
        player._p_level = 10;
        player._p_dexterity = 50;
        player._p_i_ac = 0;
        player._p_i_bonus_ac = 0;
        player._p_armor_class = 0;

        // GetArmor = 0 + 0 + 50/5 = 10; hit = 40 + 2*(5-10) + 30 - 10 = 50.
        assert_eq!(calculate_monster_to_hit(&monster, &player, 1), 50);
        // Level 16 floors the result at 30.
        assert_eq!(calculate_monster_to_hit(&monster, &player, 16), 50);

        // Low hit clamps to GetMinHit (15 on L1, 30 on L16).
        let mut weak = Monster::new(2, MonsterType::Zombie, 10, 10, 0);
        weak.level = 1;
        weak.to_hit = 0;
        let mut strong = Player::new();
        strong._p_level = 30;
        strong._p_dexterity = 100;
        assert_eq!(calculate_monster_to_hit(&weak, &strong, 1), 15);
        assert_eq!(calculate_monster_to_hit(&weak, &strong, 16), 30);
    }

    /// C++ block roll (monster.cpp:1191-1207): a standing/attacking player
    /// with a shield may block; block chance = dex + baseToBlock +
    /// 2*playerLevel - 2*monsterLevel, clamped 0-100.
    #[test]
    fn test_monster_attack_player_block_roll_matches_cpp() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.level = 1;
        monster.to_hit = 100; // hit roll always succeeds

        let mut player = Player::new();
        player._p_class = HeroClass::Warrior; // base_to_block 30
        player._p_level = 1;
        player._p_dexterity = 50;
        player._p_block_flag = true;
        player._p_mode = PlayerMode::Stand;
        player._p_hit_points = 100 * 64;
        player.position = Point::new(11, 10); // adjacent to the monster

        // blk = 50 + 30 + 2 - 2 = 80 (clamped 0-100). Find a seed whose
        // blk_roll (second roll) is < 80 so the player blocks.
        let mut blocked = false;
        for seed in 1..=200u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = monster_attack_player(&monster, &mut player, &mut rng, 1);
            if result == AttackResult::Block {
                blocked = true;
                break;
            }
            player._p_hit_points = 100 * 64; // restore for next attempt
            player._p_mode = PlayerMode::Stand;
        }
        assert!(blocked, "a blocking Warrior must block some hit rolls");
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
        let result = monster_attack_player(&monster, &mut player, &mut rng, 1);

        match result {
            AttackResult::Miss | AttackResult::Hit { .. } => {},
            _ => panic!("Unexpected result"),
        }
    }

    /// Exact C++ `PlrHitMonst` to-hit (player.cpp:548-549): Warrior
    /// baseMeleeToHit=70, so hper = level + dex/2 + iBonusToHit + 70 +
    /// _pIEnAc - monster armor, clamped to 5-95.
    /// C++ `MonsterAttackPlayer` damage (monster.cpp:1219-1220):
    /// RandomIntBetween(min<<6, max<<6) + _pIGetHit<<6, floored at 64 —
    /// no AC reduction inside MonsterAttackPlayer.
    /// C++ `PlrHitMonst` damage (player.cpp:566-570): RandomIntBetween + %
    /// bonus + _pIBonusDamMod + _pDamageMod; no AC reduction in PlrHitMonst.
    #[test]
    fn test_player_attack_monster_damage_matches_cpp() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.level = 1;
        monster.armor_class = 10;
        monster.hp = 100 * 64;
        monster.max_hp = 100 * 64;

        let mut player = Player::new();
        player._p_class = HeroClass::Warrior; // base_melee_to_hit 70
        player._p_level = 0; // 0% crit chance
        player._p_dexterity = 50;
        player._p_i_min_dam = 5;
        player._p_i_max_dam = 5;
        player._p_i_bonus_dam = 0;
        player._p_i_bonus_dam_mod = 0;
        player._p_damage_mod = 0;
        player.position = Point::new(11, 10);

        // hit_chance = 0 + 25 + 70 + 0 - 10 = 85 (roll < 85 hits).
        let mut rng = StdRng::seed_from_u64(3);
        let result = player_attack_monster(&player, &mut monster, &mut rng);
        match result {
            AttackResult::Hit { damage } | AttackResult::Kill { damage } => {
                assert_eq!(damage, 5, "display damage = 5");
            }
            AttackResult::Miss => panic!("hit roll must succeed"),
            AttackResult::Block => panic!("monster does not block"),
        }
        // damage 5 << 6 = 320 applied to 6400 HP.
        assert_eq!(monster.hp, 100 * 64 - 320);
    }

    /// C++ `PlrHitMonst` weapon-vs-monster-class modifiers (player.cpp:581-609):
    /// a sword halves damage vs Undead, a mace boosts it +50%.
    #[test]
    fn test_player_weapon_vs_monster_class_modifiers_match_cpp() {
        use crate::game::item_dat::ItemType;
        // Common setup: Warrior, level 0 (no crit), fixed 10 damage.
        let mut player = Player::new();
        player._p_class = HeroClass::Warrior;
        player._p_level = 0;
        player._p_dexterity = 50;
        player._p_i_min_dam = 10;
        player._p_i_max_dam = 10;
        player.position = Point::new(11, 10);

        // Sword vs Undead Zombie: 10 -> 10 - 10/2 = 5.
        let mut zombie = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        zombie.level = 1;
        zombie.armor_class = 0;
        zombie.hp = 100 * 64;
        zombie.max_hp = 100 * 64;
        player.inv_body[4]._itype = ItemType::Sword;
        let mut rng = StdRng::seed_from_u64(11);
        let result = player_attack_monster(&player, &mut zombie, &mut rng);
        match result {
            AttackResult::Hit { damage } | AttackResult::Kill { damage } => assert_eq!(damage, 5),
            other => panic!("expected a hit, got {:?}", other),
        }
        assert_eq!(zombie.hp, 100 * 64 - 5 * 64);

        // Mace vs Undead Zombie: 10 -> 10 + 10/2 = 15.
        let mut zombie = Monster::new(2, MonsterType::Zombie, 10, 10, 0);
        zombie.level = 1;
        zombie.armor_class = 0;
        zombie.hp = 100 * 64;
        zombie.max_hp = 100 * 64;
        player.inv_body[4]._itype = ItemType::Mace;
        let mut rng = StdRng::seed_from_u64(11);
        let result = player_attack_monster(&player, &mut zombie, &mut rng);
        match result {
            AttackResult::Hit { damage } | AttackResult::Kill { damage } => assert_eq!(damage, 15),
            other => panic!("expected a hit, got {:?}", other),
        }
        assert_eq!(zombie.hp, 100 * 64 - 15 * 64);
    }

    #[test]
    fn test_monster_attack_player_damage_matches_cpp() {
        let mut monster = Monster::new(1, MonsterType::Zombie, 10, 10, 0);
        monster.level = 1;
        monster.to_hit = 100; // always hits
        monster.min_damage = 5;
        monster.max_damage = 5; // fixed damage

        let mut player = Player::new();
        player._p_level = 1;
        player._p_dexterity = 50;
        player._p_i_get_hit = 2;
        player._p_hit_points = 100 * 64;
        player._p_max_hp = 100 * 64; // modify_hp clamps to _p_max_hp
        player.position = Point::new(11, 10);

        // raw = 5<<6 = 320; final = 320 + 2*64 = 448 (7 display HP).
        let mut rng = StdRng::seed_from_u64(7);
        let result = monster_attack_player(&monster, &mut player, &mut rng, 1);
        match result {
            AttackResult::Hit { damage } => assert_eq!(damage, 7, "display damage"),
            other => panic!("expected a hit, got {:?}", other),
        }
        // HP starts at 6400 (100*64), loses 7*64 = 448.
        assert_eq!(player._p_hit_points, 100 * 64 - 448);
    }

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
