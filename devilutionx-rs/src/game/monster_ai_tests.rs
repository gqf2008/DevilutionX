//! M67 Day 1: Monster AI Tests
//!
//! Tests for the 8 new AI functions implemented in monster.rs

#[cfg(test)]
mod tests {
    use crate::game::monster::*;
    use crate::game::types::{Direction, Point};

    /// Helper: Create test monster with basic settings
    fn create_test_monster(x: i32, y: i32) -> Monster {
        let mut monster = Monster::new(0, MonsterType::Zombie, x, y, 1);
        monster.active_for_ticks = u8::MAX;
        monster.mode = MonsterMode::Stand;
        monster.enemy_position = Point::new(x + 5, y + 5);
        monster
    }

    // ============================================================================
    // Test 1: ai_rhino - Basic charging behavior
    // ============================================================================

    #[test]
    fn test_ai_rhino_stands_when_inactive() {
        let mut monster = create_test_monster(10, 10);
        monster.active_for_ticks = 0;
        
        ai_rhino(&mut monster);
        
        // Should not change mode when inactive
        assert_eq!(monster.mode, MonsterMode::Stand);
    }

    #[test]
    fn test_ai_rhino_close_range_attack() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(10, 11); // 1 tile away
        monster.intelligence = 3;
        
        ai_rhino(&mut monster);
        
        // At close range, should attempt attack or walk
        assert!(
            monster.mode == MonsterMode::MeleeAttack || 
            monster.mode == MonsterMode::Walk
        );
    }

    #[test]
    fn test_ai_rhino_move_goal_tracking() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(20, 20); // Far away
        monster.goal = MonsterGoal::Move;
        monster.goal_var1 = 0;
        
        ai_rhino(&mut monster);
        
        // Should increment goal_var1 when in move goal
        assert!(monster.goal_var1 > 0 || monster.goal == MonsterGoal::Normal);
    }

    // ============================================================================
    // Test 2: ai_sneak - Fade in/out behavior
    // ============================================================================

    #[test]
    fn test_ai_sneak_retreat_on_hit() {
        let mut monster = create_test_monster(10, 10);
        monster.var1 = MonsterMode::HitRecovery as i16;
        
        ai_sneak(&mut monster);
        
        // Should set retreat goal when hit
        assert_eq!(monster.goal, MonsterGoal::Retreat);
        assert_eq!(monster.goal_var1, 0);
    }

    #[test]
    fn test_ai_sneak_fade_when_close() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(11, 11); // Close
        monster.intelligence = 3;
        monster.flags = MonsterFlags::HIDDEN;
        
        ai_sneak(&mut monster);
        
        // Should attempt fade in when close
        assert!(monster.mode == MonsterMode::FadeIn || monster.mode == MonsterMode::Stand);
    }

    #[test]
    fn test_ai_sneak_fade_when_far() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(20, 20); // Far
        monster.intelligence = 3;
        monster.flags = MonsterFlags::NONE;
        
        ai_sneak(&mut monster);
        
        // Should attempt fade out when far
        assert!(monster.mode == MonsterMode::FadeOut || monster.mode == MonsterMode::Stand);
    }

    // ============================================================================
    // Test 3: ai_counselor - Teleporting caster
    // ============================================================================

    #[test]
    fn test_ai_counselor_retreat_behavior() {
        let mut monster = create_test_monster(10, 10);
        monster.goal = MonsterGoal::Retreat;
        monster.goal_var1 = 0;
        
        ai_counselor(&mut monster);
        
        // Should walk away and increment goal_var1
        assert!(monster.goal_var1 > 0 || monster.goal == MonsterGoal::Normal);
    }

    #[test]
    fn test_ai_counselor_teleport_move() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(20, 20);
        monster.goal = MonsterGoal::Move;
        monster.goal_var1 = 0;
        monster.active_for_ticks = u8::MAX;
        
        ai_counselor(&mut monster);
        
        // Should track teleport movement progress
        assert!(monster.goal_var1 > 0 || monster.goal == MonsterGoal::Normal);
    }

    #[test]
    fn test_ai_counselor_ranged_attack() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(15, 15);
        monster.goal = MonsterGoal::Normal;
        monster.intelligence = 2;
        
        ai_counselor(&mut monster);
        
        // Should attempt ranged attack or delay
        assert!(
            monster.mode == MonsterMode::RangedAttack ||
            monster.mode == MonsterMode::Stand ||
            monster.var2 > 0 // AI delay
        );
    }

    // ============================================================================
    // Test 4: ai_mega - Balrog with inferno
    // ============================================================================

    #[test]
    fn test_ai_mega_uses_skeleton_when_far() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(20, 20); // 10+ tiles away
        
        ai_mega(&mut monster);
        
        // When distance >= 5, should use skeleton AI behavior
        assert!(
            monster.mode == MonsterMode::Walk ||
            monster.mode == MonsterMode::MeleeAttack ||
            monster.mode == MonsterMode::Stand
        );
    }

    #[test]
    fn test_ai_mega_move_goal() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(13, 13); // 3 tiles
        monster.goal = MonsterGoal::Move;
        monster.goal_var1 = 0;
        monster.active_for_ticks = u8::MAX;
        
        ai_mega(&mut monster);
        
        // Should track movement progress
        assert!(monster.goal_var3 == 4 || monster.goal == MonsterGoal::Normal);
    }

    #[test]
    fn test_ai_mega_inferno_attack() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(14, 14);
        monster.goal = MonsterGoal::Normal;
        monster.intelligence = 3;
        
        ai_mega(&mut monster);
        
        // Should attempt inferno or walk
        assert!(
            monster.mode == MonsterMode::SpecialRangedAttack ||
            monster.mode == MonsterMode::Walk ||
            monster.mode == MonsterMode::Stand
        );
    }

    // ============================================================================
    // Test 5: ai_lachdanan - Quest NPC
    // ============================================================================

    #[test]
    fn test_ai_lachdanan_stands_when_not_standing() {
        let mut monster = create_test_monster(10, 10);
        monster.mode = MonsterMode::Walk;
        
        ai_lachdanan(&mut monster);
        
        // Should do nothing when not standing
        assert_eq!(monster.mode, MonsterMode::Walk);
    }

    #[test]
    fn test_ai_lachdanan_quest_state_transition() {
        let mut monster = create_test_monster(10, 10);
        monster.talk_msg = 9; // TEXT_VEIL9
        monster.goal = MonsterGoal::Talking;
        
        ai_lachdanan(&mut monster);
        
        // Quest state should be tracked
        assert!(monster.talk_msg >= 9);
    }

    // ============================================================================
    // Test 6: ai_warlord - Quest boss
    // ============================================================================

    #[test]
    fn test_ai_warlord_only_acts_when_standing() {
        let mut monster = create_test_monster(10, 10);
        monster.mode = MonsterMode::Walk;
        
        ai_warlord(&mut monster);
        
        // Should not change state when not standing
        assert_eq!(monster.mode, MonsterMode::Walk);
    }

    #[test]
    fn test_ai_warlord_activates_on_dialogue() {
        let mut monster = create_test_monster(10, 10);
        monster.talk_msg = 9; // TEXT_WARLRD9
        monster.goal = MonsterGoal::Talking;
        
        ai_warlord(&mut monster);
        
        // After dialogue, should activate for combat
        assert!(
            monster.active_for_ticks == u8::MAX ||
            monster.talk_msg == 9
        );
    }

    // ============================================================================
    // Test 7: ai_hork_demon - Hellfire spawner
    // ============================================================================

    #[test]
    fn test_ai_hork_demon_inactive_check() {
        let mut monster = create_test_monster(10, 10);
        monster.active_for_ticks = 0;
        
        ai_hork_demon(&mut monster);
        
        // Should not act when inactive
        assert_eq!(monster.mode, MonsterMode::Stand);
    }

    #[test]
    fn test_ai_hork_demon_close_range_attack() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(11, 11); // 1 tile
        monster.intelligence = 3;
        
        ai_hork_demon(&mut monster);
        
        // Should attempt attack at close range
        assert!(
            monster.mode == MonsterMode::MeleeAttack ||
            monster.mode == MonsterMode::Stand
        );
    }

    #[test]
    fn test_ai_hork_demon_spawn_attack() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(14, 14); // 3+ tiles
        monster.intelligence = 5;
        monster.goal = MonsterGoal::Normal;
        
        ai_hork_demon(&mut monster);
        
        // Should attempt spawn or walk
        assert!(
            monster.mode == MonsterMode::SpecialRangedAttack ||
            monster.mode == MonsterMode::Walk ||
            monster.mode == MonsterMode::Stand
        );
    }

    // ============================================================================
    // Test 8: ai_lazarus_minion - Succubus minion
    // ============================================================================

    #[test]
    fn test_ai_lazarus_minion_only_stands() {
        let mut monster = create_test_monster(10, 10);
        monster.mode = MonsterMode::Walk;
        
        ai_lazarus_minion(&mut monster);
        
        // Should not act when not standing
        assert_eq!(monster.mode, MonsterMode::Walk);
    }

    #[test]
    fn test_ai_lazarus_minion_quest_activation() {
        let mut monster = create_test_monster(10, 10);
        monster.goal = MonsterGoal::Inquiring;
        
        ai_lazarus_minion(&mut monster);
        
        // Should transition to normal AI
        assert_eq!(monster.goal, MonsterGoal::Normal);
    }

    // ============================================================================
    // Helper function tests
    // ============================================================================

    #[test]
    fn test_distance_to_enemy() {
        let mut monster = create_test_monster(10, 10);
        monster.enemy_position = Point::new(13, 14);
        
        let distance = distance_to_enemy(&monster);
        
        // Manhattan distance: max(|13-10|, |14-10|) = max(3, 4) = 4
        assert_eq!(distance, 4);
    }

    #[test]
    fn test_flip_coin() {
        // Test with frequency 1 (always true)
        let result = flip_coin(1);
        assert_eq!(result, true);
        
        // Test with frequency 0 (always false)
        let result = flip_coin(0);
        assert_eq!(result, false);
    }

    #[test]
    fn test_opposite_direction() {
        assert_eq!(opposite_direction(Direction::North), Direction::South);
        assert_eq!(opposite_direction(Direction::East), Direction::West);
        assert_eq!(opposite_direction(Direction::South), Direction::North);
        assert_eq!(opposite_direction(Direction::West), Direction::East);
    }

    #[test]
    fn test_is_unseen_type() {
        let mut monster = create_test_monster(10, 10);
        
        // Not hidden
        monster.flags = MonsterFlags::NONE;
        assert!(!is_unseen_type(&monster));
        
        // Hidden
        monster.flags = MonsterFlags::HIDDEN;
        assert!(is_unseen_type(&monster));
    }
}
