//! M68 Day 1: Player Combat Tests
//!
//! Tests for attack functions implemented in player.rs

#[cfg(test)]
mod tests {
    use crate::game::player::*;
    use crate::game::types::{Direction, Point};

    /// Helper: Create test player with basic settings
    fn create_test_player(class: PlayerClass, level: u32) -> Player {
        let mut player = Player::new("TestPlayer".to_string(), class);
        player.level = level;
        player.x = 10;
        player.y = 10;
        player.position.tile = Point::new(10, 10);
        player.stats.hp = 100;
        player.stats.max_hp = 100;
        player.stats.mana = 50;
        player.stats.max_mana = 50;
        player.stats.dexterity = 30;
        player
    }

    /// Helper: Create attacker with specific combat stats
    fn create_attacker_player() -> Player {
        let mut player = create_test_player(PlayerClass::Warrior, 5);
        player.combat_stats.min_damage = 10;
        player.combat_stats.max_damage = 20;
        player.combat_stats.bonus_damage_percent = 0;
        player.combat_stats.bonus_damage_mod = 0;
        player.combat_stats.damage_mod = 0;
        player.combat_stats.to_hit = 50;
        player
    }

    /// Helper: Create defender with shield
    fn create_defender_player() -> Player {
        let mut player = create_test_player(PlayerClass::Warrior, 5);
        player.has_shield = true;
        player.stats.dexterity = 40;
        player.combat_stats.armor_class = 30;
        player
    }

    // ============================================================================
    // Test 1: start_attack - Attack animation initialization
    // ============================================================================

    #[test]
    fn test_start_attack_sets_direction() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);

        start_attack(&mut player, Direction::East, false);

        // Should set direction
        assert_eq!(player.position.direction, Direction::East);
        // Should set attack mode
        assert_eq!(player.mode, PlayerMode::Attack);
    }

    #[test]
    fn test_start_attack_resets_animation() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.animation.frame = 5;
        player.animation.tick_count = 3;

        start_attack(&mut player, Direction::North, false);

        // Should reset animation when not including first frame
        assert_eq!(player.animation.frame, 0);
        assert_eq!(player.animation.tick_count, 0);
    }

    #[test]
    fn test_start_attack_preserves_animation_when_including_first_frame() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.animation.frame = 2;

        start_attack(&mut player, Direction::South, true);

        // Should NOT reset animation when including first frame
        assert_eq!(player.animation.frame, 2);
        // But should still set direction and mode
        assert_eq!(player.position.direction, Direction::South);
        assert_eq!(player.mode, PlayerMode::Attack);
    }

    // ============================================================================
    // Test 2: do_attack - Attack collision detection
    // ============================================================================

    #[test]
    fn test_do_attack_adjacent_target() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.x = 10;
        player.y = 10;

        // Adjacent target (1 tile away)
        let result = do_attack(&mut player, 11, 10);

        // Should return true for adjacent target
        assert!(result);
        // Should set attack mode
        assert_eq!(player.mode, PlayerMode::Attack);
    }

    #[test]
    fn test_do_attack_too_far() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.x = 10;
        player.y = 10;

        // Target too far (3 tiles away)
        let result = do_attack(&mut player, 13, 10);

        // Should return false for distant target
        assert!(!result);
    }

    #[test]
    fn test_do_attack_sets_direction_east() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.x = 10;
        player.y = 10;

        do_attack(&mut player, 11, 10); // East

        assert_eq!(player.position.direction, Direction::East);
    }

    #[test]
    fn test_do_attack_sets_direction_west() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.x = 10;
        player.y = 10;

        do_attack(&mut player, 9, 10); // West

        assert_eq!(player.position.direction, Direction::West);
    }

    #[test]
    fn test_do_attack_sets_direction_south() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.x = 10;
        player.y = 10;

        do_attack(&mut player, 10, 11); // South

        assert_eq!(player.position.direction, Direction::South);
    }

    #[test]
    fn test_do_attack_sets_direction_north() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.x = 10;
        player.y = 10;

        do_attack(&mut player, 10, 9); // North

        assert_eq!(player.position.direction, Direction::North);
    }

    // ============================================================================
    // Test 3: player_hit_player - PvP combat system
    // ============================================================================

    #[test]
    fn test_player_hit_player_invincible_target() {
        let mut attacker = create_attacker_player();
        let mut target = create_defender_player();
        target.is_invincible = true;

        let result = player_hit_player(&mut attacker, &mut target);

        // Should not hit invincible target
        assert!(!result);
    }

    #[test]
    fn test_player_hit_player_ethereal_target() {
        let mut attacker = create_attacker_player();
        let mut target = create_defender_player();
        target.is_ethereal = true;

        let result = player_hit_player(&mut attacker, &mut target);

        // Should not hit ethereal target
        assert!(!result);
    }

    #[test]
    fn test_player_hit_player_damages_target() {
        let mut attacker = create_attacker_player();
        let mut target = create_defender_player();
        target.has_shield = false; // No block

        let initial_hp = target.stats.hp;

        // Try multiple times to ensure at least one hit
        let mut hit = false;
        for _ in 0..100 {
            let result = player_hit_player(&mut attacker, &mut target);
            if result && target.stats.hp < initial_hp {
                hit = true;
                break;
            }
            target.stats.hp = initial_hp; // Reset HP
        }

        // Should have hit at least once in 100 attempts
        assert!(hit, "Player should hit at least once in 100 attempts");
    }

    #[test]
    fn test_player_hit_player_block_sets_mode() {
        let mut attacker = create_attacker_player();
        let mut target = create_defender_player();
        target.has_shield = true;
        target.mode = PlayerMode::Stand;

        // Try multiple times to trigger a block
        let mut blocked = false;
        for _ in 0..100 {
            target.mode = PlayerMode::Stand;
            let result = player_hit_player(&mut attacker, &mut target);
            if result && target.mode == PlayerMode::Block {
                blocked = true;
                break;
            }
        }

        // Should block at least once in 100 attempts
        assert!(blocked, "Player should block at least once in 100 attempts");
    }

    // ============================================================================
    // Test 4: get_block_chance - Block percentage calculation
    // ============================================================================

    #[test]
    fn test_get_block_chance_with_level() {
        let player = create_test_player(PlayerClass::Warrior, 5);

        let block_chance = player.get_block_chance(true);

        // Warrior base_to_block = 30, dexterity = 30, level = 5
        // Formula: 30 + 30 + (5 * 2) = 70
        assert_eq!(block_chance, 70);
    }

    #[test]
    fn test_get_block_chance_without_level() {
        let player = create_test_player(PlayerClass::Warrior, 5);

        let block_chance = player.get_block_chance(false);

        // Warrior base_to_block = 30, dexterity = 30, no level bonus
        // Formula: 30 + 30 = 60
        assert_eq!(block_chance, 60);
    }

    #[test]
    fn test_get_block_chance_different_classes() {
        let warrior = create_test_player(PlayerClass::Warrior, 10);
        let rogue = create_test_player(PlayerClass::Rogue, 10);
        let sorcerer = create_test_player(PlayerClass::Sorcerer, 10);

        let warrior_block = warrior.get_block_chance(true);
        let rogue_block = rogue.get_block_chance(true);
        let sorcerer_block = sorcerer.get_block_chance(true);

        // Warrior should have highest base_to_block (30)
        // Rogue has 20, Sorcerer has 10
        assert!(warrior_block > rogue_block);
        assert!(rogue_block > sorcerer_block);
    }

    #[test]
    fn test_get_block_chance_default_wrapper() {
        let player = create_test_player(PlayerClass::Warrior, 5);

        let block_default = player.get_block_chance_default();
        let block_with_level = player.get_block_chance(true);

        // Default should be same as with level = true
        assert_eq!(block_default, block_with_level);
    }

    // ============================================================================
    // Test 5: can_block - Block availability check
    // ============================================================================

    #[test]
    fn test_can_block_with_shield_standing() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.has_shield = true;
        player.mode = PlayerMode::Stand;

        assert!(player.can_block());
    }

    #[test]
    fn test_can_block_with_shield_attacking() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.has_shield = true;
        player.mode = PlayerMode::Attack;

        assert!(player.can_block());
    }

    #[test]
    fn test_can_block_without_shield() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.has_shield = false;
        player.mode = PlayerMode::Stand;

        assert!(!player.can_block());
    }

    #[test]
    fn test_can_block_wrong_mode() {
        let mut player = create_test_player(PlayerClass::Warrior, 1);
        player.has_shield = true;
        player.mode = PlayerMode::WalkNorthwards; // Not stand or attack

        assert!(!player.can_block());
    }

    // ============================================================================
    // Test 6: Hit chance verification (5-95% clamp)
    // ============================================================================

    #[test]
    fn test_hit_chance_clamps_to_minimum_5_percent() {
        let mut attacker = create_attacker_player();
        let mut target = create_defender_player();

        // Set very low to-hit vs very high armor
        attacker.combat_stats.to_hit = 0;
        target.combat_stats.armor_class = 200;
        target.has_shield = false;

        // Try 1000 times, should miss sometimes but hit at least ~5%
        let mut hits = 0;
        for _ in 0..1000 {
            target.stats.hp = 100; // Reset HP
            if player_hit_player(&mut attacker, &mut target)
                && target.stats.hp < 100 {
                hits += 1;
            }
        }

        // With 5% hit chance, expect ~50 hits in 1000 attempts
        // Allow range 20-100 to account for randomness
        assert!(hits >= 20, "Should hit at least 20 times (actual: {})", hits);
        assert!(hits <= 100, "Should not exceed ~10% hit rate with 5% cap (actual: {})", hits);
    }

    #[test]
    fn test_hit_chance_clamps_to_maximum_95_percent() {
        let mut attacker = create_attacker_player();
        let mut target = create_defender_player();

        // Set very high to-hit vs very low armor
        attacker.combat_stats.to_hit = 200;
        target.combat_stats.armor_class = 0;
        target.has_shield = false;

        // Try 1000 times, should miss sometimes (~5%)
        let mut hits = 0;
        for _ in 0..1000 {
            target.stats.hp = 100; // Reset HP
            if player_hit_player(&mut attacker, &mut target)
                && target.stats.hp < 100 {
                hits += 1;
            }
        }

        // With 95% hit chance, expect ~950 hits in 1000 attempts
        // Allow range 900-980 to account for randomness
        assert!(hits >= 900, "Should hit at least 900 times with 95% cap (actual: {})", hits);
        assert!(hits <= 980, "Should miss occasionally even with max to-hit (actual: {})", hits);
    }

    // ============================================================================
    // Test 7: Critical strike for Warrior/Barbarian
    // ============================================================================

    #[test]
    fn test_warrior_can_critical_strike() {
        let mut attacker = create_test_player(PlayerClass::Warrior, 10);
        attacker.combat_stats.min_damage = 10;
        attacker.combat_stats.max_damage = 10;
        let mut target = create_defender_player();
        target.has_shield = false;

        // Try many times to trigger a critical
        let mut max_damage = 0;
        for _ in 0..1000 {
            target.stats.hp = 1000;
            player_hit_player(&mut attacker, &mut target);
            let damage = 1000 - target.stats.hp;
            if damage > max_damage {
                max_damage = damage;
            }
        }

        // With fixed 10 damage, critical should do 20+ (2x)
        // Account for fixed-point conversion (>>6)
        assert!(max_damage > 10, "Warrior should occasionally critical strike (max damage: {})", max_damage);
    }

    #[test]
    fn test_barbarian_can_critical_strike() {
        let mut attacker = create_test_player(PlayerClass::Barbarian, 10);
        attacker.combat_stats.min_damage = 10;
        attacker.combat_stats.max_damage = 10;
        let mut target = create_defender_player();
        target.has_shield = false;

        // Try many times to trigger a critical
        let mut max_damage = 0;
        for _ in 0..1000 {
            target.stats.hp = 1000;
            player_hit_player(&mut attacker, &mut target);
            let damage = 1000 - target.stats.hp;
            if damage > max_damage {
                max_damage = damage;
            }
        }

        // Barbarian should also crit
        assert!(max_damage > 10, "Barbarian should occasionally critical strike (max damage: {})", max_damage);
    }

    #[test]
    fn test_sorcerer_cannot_critical_strike() {
        let mut attacker = create_test_player(PlayerClass::Sorcerer, 10);
        attacker.combat_stats.min_damage = 10;
        attacker.combat_stats.max_damage = 10;
        let mut target = create_defender_player();
        target.has_shield = false;

        // Try many times
        let mut max_damage = 0;
        for _ in 0..1000 {
            target.stats.hp = 1000;
            player_hit_player(&mut attacker, &mut target);
            let damage = 1000 - target.stats.hp;
            if damage > max_damage {
                max_damage = damage;
            }
        }

        // Sorcerer should never exceed base damage significantly
        // Allow small variance from damage calculation but no 2x crits
        assert!(max_damage <= 15, "Sorcerer should not critical strike (max damage: {})", max_damage);
    }

    // ============================================================================
    // Test 8: PlayerClass to HeroClass conversion
    // ============================================================================

    #[test]
    fn test_player_class_to_hero_class_conversion() {
        use crate::game::playerdat::HeroClass;

        let warrior: HeroClass = PlayerClass::Warrior.into();
        let rogue: HeroClass = PlayerClass::Rogue.into();
        let sorcerer: HeroClass = PlayerClass::Sorcerer.into();
        let monk: HeroClass = PlayerClass::Monk.into();
        let bard: HeroClass = PlayerClass::Bard.into();
        let barbarian: HeroClass = PlayerClass::Barbarian.into();

        assert_eq!(warrior, HeroClass::Warrior);
        assert_eq!(rogue, HeroClass::Rogue);
        assert_eq!(sorcerer, HeroClass::Sorcerer);
        assert_eq!(monk, HeroClass::Monk);
        assert_eq!(bard, HeroClass::Bard);
        assert_eq!(barbarian, HeroClass::Barbarian);
    }
}
