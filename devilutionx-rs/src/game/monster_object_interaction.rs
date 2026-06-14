//! Monster ⟷ Object interaction (doors and movement blocking).
//!
//! **C++ References**:
//! - `Source/objects.cpp:4302-4316` — `MonstCheckDoors`
//! - `Source/objects.cpp:1762-1784` — `OperateDoor`
//! - `Source/monster.cpp:1729-1741` — `IsTileAccessible`

use crate::game::types::{Direction, Point};
use crate::game::monster::Monster;
use crate::game::objects::Object;
use crate::game::objdat::ObjectId;

/// Door state: closed (C++ objects.cpp).
const DOOR_CLOSED: i32 = 0;
/// Door state: open.
const DOOR_OPEN: i32 = 1;
/// Door state: blocked.
const DOOR_BLOCKED: i32 = 2;

// ────────────────────────────────────────────────────────────────────────────
// Monster ⟷ Door interaction
// ────────────────────────────────────────────────────────────────────────────

/// Check if a monster can open doors (C++ `MFLAG_CAN_OPEN_DOOR`).
pub fn can_monster_open_doors(monster: &Monster) -> bool {
    monster.flags.can_open_door()
}

/// Check all adjacent doors and open the closed ones the monster can use.
///
/// **C++ Reference**: `MonstCheckDoors()` — `Source/objects.cpp:4302`.
///
/// Returns the number of doors opened.
pub fn monster_check_doors(monster: &Monster, objects: &mut [Object]) -> usize {
    if !can_monster_open_doors(monster) {
        return 0;
    }

    let mut doors_opened = 0;

    // C++ iterates { NE, SW, N, E, S, W, NW, SE }.
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
        let check_pos = Point::new(monster.x + dx, monster.y + dy);

        if let Some(door) = find_door_at_position(objects, check_pos) {
            if door.door_state == DOOR_CLOSED && is_door_object(door.otype) {
                // Open the door (C++ calls OperateDoor with sound effects).
                door.door_state = DOOR_OPEN;
                door.solid = false;
                doors_opened += 1;
            }
        }
    }

    doors_opened
}

/// Whether `position` is blocked for `monster` by a solid object.
///
/// **C++ Reference**: `IsTileAccessible()` — `Source/monster.cpp:1729`.
pub fn is_position_blocked_by_object(monster: &Monster, objects: &[Object], position: Point) -> bool {
    for object in objects {
        if object.position == position && does_object_block_monster(monster, object) {
            return true;
        }
    }
    false
}

/// Whether `object` blocks movement for `monster`.
pub fn does_object_block_monster(monster: &Monster, object: &Object) -> bool {
    // 1. Deleted objects don't block.
    if object.del_flag {
        return false;
    }
    // 2. Non-solid objects don't block.
    if !object.solid {
        return false;
    }
    // 3. Doors can be opened by some monsters.
    if is_door_object(object.otype) {
        if can_monster_open_doors(monster) {
            // Closed doors are passable for door-opening monsters.
            return object.door_state == DOOR_BLOCKED;
        }
        // Other monsters are blocked by non-open doors.
        return object.door_state != DOOR_OPEN;
    }
    // 4. All other solid objects block movement.
    true
}

/// Whether `otype` is any dungeon door.
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

/// First non-deleted door object at `position`, if any.
fn find_door_at_position(objects: &mut [Object], position: Point) -> Option<&mut Object> {
    objects
        .iter_mut()
        .find(|obj| obj.position == position && is_door_object(obj.otype) && !obj.del_flag)
}
