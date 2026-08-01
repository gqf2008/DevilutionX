// M66 Day 1 Verification Script
//
// This is a standalone verification of the CheckMissileCol implementation
// Run this to verify the logic without full integration tests

use devilutionx_rs::game::missiles::*;
use devilutionx_rs::game::types::{Point, Direction};
use devilutionx_rs::game::combat::DamageType;

fn main() {
    println!("=== M66 Day 1: CheckMissileCol Verification ===\n");

    // Test 1: check_can_hit_only_walking
    println!("Test 1: check_can_hit_only_walking");
    let mut missile = Missile::new(MissileID::Rhino, Point::new(50, 50));
    missile.position.start = Point::new(50, 60);

    let target_north = Point::new(50, 55);
    let can_hit = check_can_hit_only_walking(&missile, target_north, Direction::North);
    println!("  Can hit north target: {} (expected: true)", can_hit);
    assert!(can_hit, "Should hit north target");

    let target_south = Point::new(50, 65);
    let cannot_hit = check_can_hit_only_walking(&missile, target_south, Direction::North);
    println!("  Can hit south target: {} (expected: false)", cannot_hit);
    assert!(!cannot_hit, "Should NOT hit south target");

    println!("  ✓ PASSED\n");

    // Test 2: check_missile_col - out of bounds
    println!("Test 2: check_missile_col - out of bounds");
    let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
    missile.caster = MissileCaster::TargetMonsters;
    let state = DungeonState::default();

    let mut callbacks_called = 0;
    check_missile_col(
        &mut missile,
        DamageType::Physical,
        10, 20, false,
        Point::new(-1, 0), // Out of bounds
        false, None,
        &state,
        |_, _, _| { callbacks_called += 1; true },
        |_, _, _| { callbacks_called += 1; (true, false) },
        |_| { callbacks_called += 1; },
    );

    println!("  Callbacks called: {} (expected: 0)", callbacks_called);
    println!("  Hit flag: {} (expected: false)", missile.hit_flag);
    assert_eq!(callbacks_called, 0, "No callbacks should be called for OOB");
    assert!(!missile.hit_flag, "Hit flag should be false");
    println!("  ✓ PASSED\n");

    // Test 3: check_missile_col - monster hit
    println!("Test 3: check_missile_col - monster hit");
    let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
    missile.caster = MissileCaster::TargetMonsters;
    missile.source = 0;
    missile.duration = 100;

    let mut state = DungeonState::default();
    state.monsters[60][60] = 5; // Monster ID 4 (value = id + 1)

    let mut monster_was_hit = false;
    check_missile_col(
        &mut missile,
        DamageType::Physical,
        10, 20, false,
        Point::new(60, 60),
        false, None,
        &state,
        |monster_id, min, max| {
            println!("  Monster hit callback: id={}, min={}, max={}", monster_id, min, max);
            monster_was_hit = true;
            assert_eq!(monster_id, 4, "Monster ID should be 4");
            assert_eq!(min, 10, "Min damage should be 10");
            assert_eq!(max, 20, "Max damage should be 20");
            true
        },
        |_, _, _| (false, false),
        |_| {},
    );

    println!("  Monster was hit: {} (expected: true)", monster_was_hit);
    println!("  Hit flag: {} (expected: true)", missile.hit_flag);
    println!("  Duration: {} (expected: 0)", missile.duration);
    assert!(monster_was_hit, "Monster should be hit");
    assert!(missile.hit_flag, "Hit flag should be true");
    assert_eq!(missile.duration, 0, "Missile should be deleted");
    println!("  ✓ PASSED\n");

    // Test 4: check_missile_col - don't delete
    println!("Test 4: check_missile_col - don't delete on collision");
    let mut missile = Missile::new(MissileID::Fireball, Point::new(50, 50));
    missile.caster = MissileCaster::TargetMonsters;
    missile.source = 0;
    missile.duration = 100;

    let mut state = DungeonState::default();
    state.monsters[60][60] = 5;

    check_missile_col(
        &mut missile,
        DamageType::Fire,
        10, 20, false,
        Point::new(60, 60),
        true, // Don't delete on collision
        None,
        &state,
        |_, _, _| true,
        |_, _, _| (false, false),
        |_| {},
    );

    println!("  Hit flag: {} (expected: true)", missile.hit_flag);
    println!("  Duration: {} (expected: 100)", missile.duration);
    assert!(missile.hit_flag, "Hit flag should be true");
    assert_eq!(missile.duration, 100, "Missile should NOT be deleted");
    println!("  ✓ PASSED\n");

    // Test 5: move_missile_and_check_missile_col
    println!("Test 5: move_missile_and_check_missile_col");
    let mut missile = Missile::new(MissileID::Arrow, Point::new(50, 50));
    missile.position.velocity = Point::new(65536, 0); // Move 1 tile east
    missile.duration = 100;
    missile.caster = MissileCaster::TargetMonsters;

    let mut state = DungeonState::default();
    state.monsters[51][50] = 5;

    let mut hit_monster = false;
    move_missile_and_check_missile_col(
        &mut missile,
        DamageType::Physical,
        10, 20,
        false, false,
        &state,
        |_, _, _| { hit_monster = true; true },
        |_, _, _| (false, false),
        |_| {},
    );

    println!("  New position: {:?} (expected: Point(51, 50))", missile.position.tile);
    println!("  Hit monster: {} (expected: true)", hit_monster);
    assert_eq!(missile.position.tile, Point::new(51, 50), "Should move to (51, 50)");
    assert!(hit_monster, "Should hit monster");
    println!("  ✓ PASSED\n");

    println!("=== All verification tests PASSED! ===");
}
