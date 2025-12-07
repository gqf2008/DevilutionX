// Monster-Object Interaction System (Day 37)
//
// C++ References:
// - Source/objects.cpp:4302-4316 (MonstCheckDoors)
// - Source/objects.cpp:1762-1784 (OperateDoor)
// - Source/monster.cpp:1729-1741 (IsTileAccessible)
//
// This module implements interactions between monsters and objects,
// particularly doors and destructible objects.

use crate::game::types::{Point, Direction};
use crate::game::monster_exact::{Monster, MonsterFlags};
use crate::game::objects::Object;
use crate::game::data::objdat::ObjectId;

/// Door state constants (from C++ objects.cpp)
const DOOR_CLOSED: i32 = 0;
const DOOR_OPEN: i32 = 1;
const DOOR_BLOCKED: i32 = 2;

//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MONSTER-DOOR INTERACTION
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Check if a monster can open doors
///
/// **C++ Reference**: Monster flags `MFLAG_CAN_OPEN_DOOR` (Source/monster.h:55)
pub fn can_monster_open_doors(monster: &Monster) -> bool {
    // Check monster flags for door-opening ability
    // In C++: (monster.flags & MFLAG_CAN_OPEN_DOOR) != 0
    monster.flags.can_open_door()
}

/// Check all adjacent doors and open them if possible
///
/// **C++ Reference**: `Source/objects.cpp:4302` - `MonstCheckDoors()`
///
/// # C++ Implementation
/// ```cpp
/// void MonstCheckDoors(const Monster &monster)
/// {
///     for (const Direction dir : { NE, SW, N, E, S, W, NW, SE }) {
///         Object *object = FindObjectAtPosition(monster.position.tile + dir);
///         if (object == nullptr) continue;
///
///         Object &door = *object;
///         if (!door.isDoor() || door._oVar4 != DOOR_CLOSED)
///             continue;
///
///         OperateDoor(door, true);
///     }
/// }
/// ```
///
/// # C++ Alignment
/// - ✅ Checks all 8 directions around monster
/// - ✅ Only operates closed doors (door_state == DOOR_CLOSED)
/// - ✅ Calls OperateDoor() to handle door opening
///
/// # Arguments
/// - `monster`: Monster attempting to open doors
/// - `objects`: Mutable slice of all objects in the level
///
/// # Returns
/// Number of doors opened
pub fn monster_check_doors(monster: &Monster, objects: &mut [Object]) -> usize {
    if !can_monster_open_doors(monster) {
        return 0;
    }

    let mut doors_opened = 0;

    // Check all 8 directions (C++ uses NE, SW, N, E, S, W, NW, SE)
    let directions = [
        Direction::NorthEast,
        Direction::SouthWest,
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::NorthWest,
        Direction::SouthEast,
    ];

    for &dir in &directions {
        let dx = dir.dx();
        let dy = dir.dy();
        let check_pos = Point::new(monster.position.x + dx, monster.position.y + dy);

        // Find door at this position
        if let Some(door) = find_door_at_position(objects, check_pos) {
            if door.door_state == DOOR_CLOSED && is_door_object(door.otype) {
                // Open the door (simplified - C++ calls OperateDoor with sound effects)
                door.door_state = DOOR_OPEN;
                door.solid = false;
                doors_opened += 1;
            }
        }
    }

    doors_opened
}

/// Try to open a specific door
///
/// **C++ Reference**: `Source/objects.cpp:1762` - `OperateDoor()`
///
/// # C++ Implementation
/// ```cpp
/// void OperateDoor(Object &door, bool sendflag)
/// {
///     const bool openDoor = door._oVar4 == DOOR_CLOSED;
///
///     if (!openDoor && !IsDoorClear(door)) {
///         door._oVar4 = DOOR_BLOCKED;
///         return;
///     }
///
///     if (openDoor) {
///         PlaySfxLoc(SfxID::DoorOpen, door.position);
///         OpenDoor(door);
///     } else {
///         PlaySfxLoc(SfxID::DoorClose, door.position);
///         CloseDoor(door);
///     }
/// }
/// ```
///
/// # Arguments
/// - `monster`: Monster opening the door
/// - `door`: Door object to open
///
/// # Returns
/// - `true` if door was opened successfully
/// - `false` if door cannot be opened
pub fn monster_open_door(monster: &Monster, door: &mut Object) -> bool {
    // 1. Check monster can open doors
    if !can_monster_open_doors(monster) {
        return false;
    }

    // 2. Check this is actually a door
    if !is_door_object(door.otype) {
        return false;
    }

    // 3. Check door is closed
    if door.door_state != DOOR_CLOSED {
        return false;
    }

    // 4. Open the door (C++ line 1774)
    door.door_state = DOOR_OPEN;
    door.solid = false;
    door.anim_frame = 0; // Start open animation

    true
}

//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MONSTER-OBJECT DESTRUCTION
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Check if a monster can break objects
///
/// In Diablo 1, most monsters don't actively break objects, but some bosses
/// and large monsters can destroy barrels, crates, etc. when moving through them.
pub fn can_monster_break_objects(monster: &Monster) -> bool {
    // Check monster size/type flags
    // In C++: Based on monster type (Golem, Diablo, etc.)
    // For now, return false (not implemented in base game)
    false
}

/// Monster attempts to break a destructible object
///
/// # Arguments
/// - `monster`: Monster attempting to break the object
/// - `object`: Object to potentially break
///
/// # Returns
/// - `true` if object was broken
/// - `false` if object cannot be broken
pub fn monster_break_object(monster: &Monster, object: &mut Object) -> bool {
    // 1. Check monster can break objects
    if !can_monster_break_objects(monster) {
        return false;
    }

    // 2. Check object is breakable
    if !object.breakable {
        return false;
    }

    // 3. Check object is not already broken
    if object.del_flag {
        return false;
    }

    // 4. Break the object
    object.del_flag = true;
    object.solid = false;

    true
}

//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PATH BLOCKING CHECKS
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Check if an object blocks a monster's path
///
/// **C++ Reference**: `Source/monster.cpp:1736` - `IsTileAccessible()`
///
/// # Arguments
/// - `monster`: Monster checking the path
/// - `object`: Object to check
///
/// # Returns
/// - `true` if object blocks the monster
/// - `false` if monster can pass (or open door)
pub fn does_object_block_monster(monster: &Monster, object: &Object) -> bool {
    // 1. Deleted objects don't block
    if object.del_flag {
        return false;
    }

    // 2. Non-solid objects don't block
    if !object.solid {
        return false;
    }

    // 3. Doors can be opened by some monsters
    if is_door_object(object.otype) {
        if can_monster_open_doors(monster) {
            // Closed doors are passable for door-opening monsters
            return object.door_state == DOOR_BLOCKED;
        }
        // Other monsters are blocked by closed doors
        return object.door_state != DOOR_OPEN;
    }

    // 4. All other solid objects block movement
    true
}

/// Check if a tile position contains a blocking object
///
/// # Arguments
/// - `monster`: Monster checking the path
/// - `objects`: Slice of all objects in the level
/// - `position`: Position to check
///
/// # Returns
/// - `true` if position is blocked by an object
/// - `false` if position is clear
pub fn is_position_blocked_by_object(
    monster: &Monster,
    objects: &[Object],
    position: Point,
) -> bool {
    for object in objects {
        if object.position == position && does_object_block_monster(monster, object) {
            return true;
        }
    }
    false
}

//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HELPER FUNCTIONS
//━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Check if an ObjectId is a door type
///
/// **C++ Reference**: `Object::isDoor()` (Source/objects.h)
fn is_door_object(otype: ObjectId) -> bool {
    matches!(
        otype,
        ObjectId::L1LDoor
            | ObjectId::L1RDoor
            | ObjectId::L2LDoor
            | ObjectId::L2RDoor
            | ObjectId::L3LDoor
            | ObjectId::L3RDoor
            | ObjectId::L5LDoor
            | ObjectId::L5RDoor
    )
}

/// Find a door object at a specific position
///
/// # Arguments
/// - `objects`: Mutable slice of all objects
/// - `position`: Position to search
///
/// # Returns
/// - `Some(&mut Object)` if a door is found at position
/// - `None` if no door exists at position
fn find_door_at_position(objects: &mut [Object], position: Point) -> Option<&mut Object> {
    objects
        .iter_mut()
        .find(|obj| obj.position == position && is_door_object(obj.otype) && !obj.del_flag)
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::monster_dat::{get_monster_data, MonsterAIID, MonsterId};

    #[test]
    fn test_can_monster_open_doors() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);

        // Zombie without flag cannot open doors
        monster.flags = MonsterFlags::NONE;
        assert!(!can_monster_open_doors(&monster));

        // Zombie with MFLAG_CAN_OPEN_DOOR can open doors
        monster.flags = MonsterFlags::CAN_OPEN_DOOR;
        assert!(can_monster_open_doors(&monster));
    }

    #[test]
    fn test_monster_open_door_success() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);
        monster.flags = MonsterFlags::CAN_OPEN_DOOR;

        let mut door = Object::new(ObjectId::L1LDoor, Point::new(11, 10));
        door.door_state = DOOR_CLOSED;
        door.solid = true;

        let result = monster_open_door(&monster, &mut door);

        assert!(result, "Monster should open door");
        assert_eq!(door.door_state, DOOR_OPEN);
        assert!(!door.solid);
    }

    #[test]
    fn test_monster_open_door_cannot_open() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);
        monster.flags = MonsterFlags::NONE;

        let mut door = Object::new(ObjectId::L1LDoor, Point::new(11, 10));
        door.door_state = DOOR_CLOSED;

        let result = monster_open_door(&monster, &mut door);

        assert!(!result, "Monster without flag should not open door");
        assert_eq!(door.door_state, DOOR_CLOSED);
    }

    #[test]
    fn test_monster_open_door_already_open() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);
        monster.flags = MonsterFlags::CAN_OPEN_DOOR;

        let mut door = Object::new(ObjectId::L1LDoor, Point::new(11, 10));
        door.door_state = DOOR_OPEN; // Already open

        let result = monster_open_door(&monster, &mut door);

        assert!(!result, "Should not re-open already open door");
    }

    #[test]
    fn test_does_object_block_monster_solid() {
        let data = get_monster_data(MonsterId::ZombieN);
        let monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);

        let mut barrel = Object::new(ObjectId::BarrelEx, Point::new(11, 10));
        barrel.solid = true;
        barrel.del_flag = false;

        assert!(does_object_block_monster(&monster, &barrel));
    }

    #[test]
    fn test_does_object_block_monster_door_can_open() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);
        monster.flags = MonsterFlags::CAN_OPEN_DOOR;

        let mut door = Object::new(ObjectId::L1LDoor, Point::new(11, 10));
        door.door_state = DOOR_CLOSED;
        door.solid = true;

        // Monster can open doors, so closed door doesn't block
        assert!(!does_object_block_monster(&monster, &door));

        // But blocked door still blocks
        door.door_state = DOOR_BLOCKED;
        assert!(does_object_block_monster(&monster, &door));
    }

    #[test]
    fn test_does_object_block_monster_door_cannot_open() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);
        monster.flags = MonsterFlags::NONE;

        let mut door = Object::new(ObjectId::L1LDoor, Point::new(11, 10));
        door.door_state = DOOR_CLOSED;
        door.solid = true;

        // Monster cannot open doors, so closed door blocks
        assert!(does_object_block_monster(&monster, &door));

        // Open door doesn't block
        door.door_state = DOOR_OPEN;
        assert!(!does_object_block_monster(&monster, &door));
    }

    #[test]
    fn test_monster_check_doors_opens_adjacent() {
        let data = get_monster_data(MonsterId::ZombieN);
        let mut monster = Monster::new(data, Point::new(10, 10), MonsterAIID::Zombie, 0);
        monster.flags = MonsterFlags::CAN_OPEN_DOOR;

        let mut objects = vec![
            {
                let mut door = Object::new(ObjectId::L1LDoor, Point::new(11, 10)); // East
                door.door_state = DOOR_CLOSED;
                door.solid = true;
                door
            },
            {
                let mut door = Object::new(ObjectId::L2LDoor, Point::new(10, 9)); // North
                door.door_state = DOOR_CLOSED;
                door.solid = true;
                door
            },
            {
                let mut door = Object::new(ObjectId::L1LDoor, Point::new(9, 10)); // West (already open)
                door.door_state = DOOR_OPEN;
                door.solid = false;
                door
            },
        ];

        let opened = monster_check_doors(&monster, &mut objects);

        assert_eq!(opened, 2, "Should open 2 closed doors");
        assert_eq!(objects[0].door_state, DOOR_OPEN); // East door opened
        assert_eq!(objects[1].door_state, DOOR_OPEN); // North door opened
        assert_eq!(objects[2].door_state, DOOR_OPEN); // West door was already open
    }
}
