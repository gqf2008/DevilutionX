/// Tile Properties System
///
/// Ported from Source/levels/tile_properties.hpp/cpp and dun_tile.hpp
///
/// This module handles tile walkability, solidity, and collision detection.

use crate::engine::Point;
use crate::levels::types::*;
use crate::levels::gendung::{Dungeon, TilePropertyManager};

// ============================================================================
// Tile Type (for rendering)
// ============================================================================

/// Level tile type - determines data encoding and shape
///
/// C++ equivalent: `enum class TileType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TileType {
    /// 32x32 square, stored as array of pixels
    Square = 0,

    /// 32x32 square with transparency, RLE encoded
    TransparentSquare = 1,

    /// Left-pointing 32x31 triangle
    LeftTriangle = 2,

    /// Right-pointing 32x31 triangle
    RightTriangle = 3,

    /// Left-pointing 32x32 trapezoid
    LeftTrapezoid = 4,

    /// Right-pointing 32x32 trapezoid
    RightTrapezoid = 5,
}

/// Level CEL block - represents a tile frame reference
///
/// C++ equivalent: `struct LevelCelBlock`
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelCelBlock {
    pub data: u16,
}

impl LevelCelBlock {
    /// Check if this block has a value
    #[inline]
    pub fn has_value(&self) -> bool {
        self.data != 0
    }

    /// Get the tile type (encoded in upper bits)
    #[inline]
    pub fn tile_type(&self) -> TileType {
        let type_bits = (self.data & 0x7000) >> 12;
        match type_bits {
            0 => TileType::Square,
            1 => TileType::TransparentSquare,
            2 => TileType::LeftTriangle,
            3 => TileType::RightTriangle,
            4 => TileType::LeftTrapezoid,
            5 => TileType::RightTrapezoid,
            _ => TileType::Square,
        }
    }

    /// Get the frame index (1-based)
    #[inline]
    pub fn frame(&self) -> u16 {
        self.data & 0xFFF
    }
}

// ============================================================================
// Tile Constants
// ============================================================================

pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

pub const DUN_FRAME_WIDTH: i32 = TILE_WIDTH / 2;
pub const DUN_FRAME_HEIGHT: i32 = TILE_HEIGHT;
pub const DUN_FRAME_TRIANGLE_HEIGHT: i32 = 31;

pub const REENCODED_TRIANGLE_FRAME_SIZE: usize = 544 - 32;
pub const REENCODED_TRAPEZOID_FRAME_SIZE: usize = 800 - 16;

/// Calculate sprite tile center X offset
#[inline]
pub fn calculate_sprite_tile_center_x(width: i32) -> i32 {
    (width - TILE_WIDTH) / 2
}

// ============================================================================
// Tile Property Queries
// ============================================================================

/// Check if a tile is not solid (walkable base check)
///
/// C++ equivalent: `IsTileNotSolid()`
#[inline]
pub fn is_tile_not_solid(dungeon: &Dungeon, tile_props: &TilePropertyManager, pos: Point) -> bool {
    if !in_dungeon_bounds_point(pos) {
        return false;
    }

    let piece_id = dungeon.get_piece(pos.x as usize, pos.y as usize);
    !tile_props.is_solid(piece_id)
}

/// Check if a tile is solid (not walkable)
///
/// C++ equivalent: `IsTileSolid()`
#[inline]
pub fn is_tile_solid(dungeon: &Dungeon, tile_props: &TilePropertyManager, pos: Point) -> bool {
    if !in_dungeon_bounds_point(pos) {
        return false;
    }

    let piece_id = dungeon.get_piece(pos.x as usize, pos.y as usize);
    tile_props.is_solid(piece_id)
}

/// Check if a tile is walkable (considers objects)
///
/// C++ equivalent: `IsTileWalkable()`
///
/// Note: Simplified version without object checking (objects will be added later)
pub fn is_tile_walkable(dungeon: &Dungeon, tile_props: &TilePropertyManager, pos: Point, _ignore_doors: bool) -> bool {
    // TODO: Check for objects when object system is integrated
    // Object *object = FindObjectAtPosition(position);
    // if (object != nullptr) {
    //     if (ignoreDoors && object->isDoor()) return true;
    //     if (object->_oSolidFlag) return false;
    // }

    is_tile_not_solid(dungeon, tile_props, pos)
}

/// Check if a tile is occupied (by monsters, players, objects, or solid tiles)
///
/// C++ equivalent: `IsTileOccupied()`
pub fn is_tile_occupied(dungeon: &Dungeon, tile_props: &TilePropertyManager, pos: Point) -> bool {
    if !in_dungeon_bounds_point(pos) {
        return true; // OOB positions are considered occupied
    }

    // Check if solid
    if is_tile_solid(dungeon, tile_props, pos) {
        return true;
    }

    let x = pos.x as usize;
    let y = pos.y as usize;

    // Check for monsters
    if dungeon.monster_layer[x][y] != 0 {
        return true;
    }

    // Check for players
    if dungeon.player_layer[x][y] != 0 {
        return true;
    }

    // Check for objects
    if dungeon.object_layer[x][y] != 0 {
        return true;
    }

    false
}

/// Check if stepping from start to destination cuts a corner
///
/// C++ equivalent: `CanStep()`
///
/// If you step from A to B, both Xs need to be clear:
/// ```text
///  AX
///  XB
/// ```
pub fn can_step(dungeon: &Dungeon, tile_props: &TilePropertyManager, start_pos: Point, dest_pos: Point) -> bool {
    // Get direction from start to destination
    let dx = dest_pos.x - start_pos.x;
    let dy = dest_pos.y - start_pos.y;

    // Determine path direction (simplified from GetPathDirection)
    // Direction mapping: 5=North, 6=East, 7=South, 8=West
    let direction = match (dx, dy) {
        (0, -1) => 5,  // North
        (1, 0) => 6,   // East
        (0, 1) => 7,   // South
        (-1, 0) => 8,  // West
        _ => return true, // Diagonal or same position, allow
    };

    match direction {
        5 => { // Stepping north
            let sw = Point::new(dest_pos.x - 1, dest_pos.y + 1);
            let se = Point::new(dest_pos.x + 1, dest_pos.y + 1);
            is_tile_not_solid(dungeon, tile_props, sw) && is_tile_not_solid(dungeon, tile_props, se)
        }
        6 => { // Stepping east
            let sw = Point::new(dest_pos.x - 1, dest_pos.y + 1);
            let nw = Point::new(dest_pos.x - 1, dest_pos.y - 1);
            is_tile_not_solid(dungeon, tile_props, sw) && is_tile_not_solid(dungeon, tile_props, nw)
        }
        7 => { // Stepping south
            let ne = Point::new(dest_pos.x + 1, dest_pos.y - 1);
            let nw = Point::new(dest_pos.x - 1, dest_pos.y - 1);
            is_tile_not_solid(dungeon, tile_props, ne) && is_tile_not_solid(dungeon, tile_props, nw)
        }
        8 => { // Stepping west
            let se = Point::new(dest_pos.x + 1, dest_pos.y + 1);
            let ne = Point::new(dest_pos.x + 1, dest_pos.y - 1);
            is_tile_not_solid(dungeon, tile_props, se) && is_tile_not_solid(dungeon, tile_props, ne)
        }
        _ => true,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_cel_block() {
        let block = LevelCelBlock { data: 0 };
        assert!(!block.has_value());

        // Test with value: type=Square(0), frame=42
        let block = LevelCelBlock { data: 42 };
        assert!(block.has_value());
        assert_eq!(block.frame(), 42);
        assert_eq!(block.tile_type(), TileType::Square);

        // Test with type=TransparentSquare(1), frame=100
        let block = LevelCelBlock { data: 0x1000 | 100 };
        assert_eq!(block.tile_type(), TileType::TransparentSquare);
        assert_eq!(block.frame(), 100);

        // Test with type=LeftTriangle(2), frame=255
        let block = LevelCelBlock { data: 0x2000 | 255 };
        assert_eq!(block.tile_type(), TileType::LeftTriangle);
        assert_eq!(block.frame(), 255);
    }

    #[test]
    fn test_tile_constants() {
        assert_eq!(TILE_WIDTH, 64);
        assert_eq!(TILE_HEIGHT, 32);
        assert_eq!(DUN_FRAME_WIDTH, 32);
        assert_eq!(DUN_FRAME_HEIGHT, 32);
        assert_eq!(DUN_FRAME_TRIANGLE_HEIGHT, 31);
    }

    #[test]
    fn test_calculate_sprite_tile_center_x() {
        assert_eq!(calculate_sprite_tile_center_x(64), 0);
        assert_eq!(calculate_sprite_tile_center_x(128), 32);
        assert_eq!(calculate_sprite_tile_center_x(96), 16);
    }

    #[test]
    fn test_is_tile_solid() {
        let mut dungeon = Dungeon::new();
        let mut tile_props = TilePropertyManager::new();

        // Set piece 100 as solid
        tile_props.set_properties(100, TileProperties::SOLID);
        dungeon.set_piece(10, 10, 100);

        let pos = Point::new(10, 10);
        assert!(is_tile_solid(&dungeon, &tile_props, pos));
        assert!(!is_tile_not_solid(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_is_tile_not_solid() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        // Set piece 50 (no properties = not solid)
        dungeon.set_piece(20, 20, 50);

        let pos = Point::new(20, 20);
        assert!(is_tile_not_solid(&dungeon, &tile_props, pos));
        assert!(!is_tile_solid(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_is_tile_walkable() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        // Floor tile (walkable)
        dungeon.set_piece(15, 15, 1);
        let pos = Point::new(15, 15);
        assert!(is_tile_walkable(&dungeon, &tile_props, pos, false));
    }

    #[test]
    fn test_is_tile_occupied_by_solid() {
        let mut dungeon = Dungeon::new();
        let mut tile_props = TilePropertyManager::new();

        // Solid wall
        tile_props.set_properties(99, TileProperties::SOLID);
        dungeon.set_piece(30, 30, 99);

        let pos = Point::new(30, 30);
        assert!(is_tile_occupied(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_is_tile_occupied_by_monster() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        // Place monster
        dungeon.monster_layer[25][25] = 1;

        let pos = Point::new(25, 25);
        assert!(is_tile_occupied(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_is_tile_occupied_by_player() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        // Place player
        dungeon.player_layer[35][35] = 1;

        let pos = Point::new(35, 35);
        assert!(is_tile_occupied(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_is_tile_occupied_by_object() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        // Place object
        dungeon.object_layer[40][40] = 1;

        let pos = Point::new(40, 40);
        assert!(is_tile_occupied(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_is_tile_occupied_oob() {
        let dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        // Out of bounds is considered occupied
        let pos = Point::new(-1, -1);
        assert!(is_tile_occupied(&dungeon, &tile_props, pos));

        let pos = Point::new(999, 999);
        assert!(is_tile_occupied(&dungeon, &tile_props, pos));
    }

    #[test]
    fn test_can_step_north() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        let start = Point::new(10, 10);
        let dest = Point::new(10, 9); // North

        // Both corner tiles clear
        assert!(can_step(&dungeon, &tile_props, start, dest));

        // Block southwest corner
        let mut tile_props2 = TilePropertyManager::new();
        tile_props2.set_properties(99, TileProperties::SOLID);
        dungeon.set_piece(9, 10, 99); // SW of dest

        assert!(!can_step(&dungeon, &tile_props2, start, dest));
    }

    #[test]
    fn test_can_step_east() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        let start = Point::new(10, 10);
        let dest = Point::new(11, 10); // East

        assert!(can_step(&dungeon, &tile_props, start, dest));

        // Block southwest corner
        let mut tile_props2 = TilePropertyManager::new();
        tile_props2.set_properties(99, TileProperties::SOLID);
        dungeon.set_piece(10, 11, 99); // SW of dest

        assert!(!can_step(&dungeon, &tile_props2, start, dest));
    }

    #[test]
    fn test_can_step_south() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        let start = Point::new(10, 10);
        let dest = Point::new(10, 11); // South

        assert!(can_step(&dungeon, &tile_props, start, dest));
    }

    #[test]
    fn test_can_step_west() {
        let mut dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        let start = Point::new(10, 10);
        let dest = Point::new(9, 10); // West

        assert!(can_step(&dungeon, &tile_props, start, dest));
    }

    #[test]
    fn test_can_step_diagonal() {
        let dungeon = Dungeon::new();
        let tile_props = TilePropertyManager::new();

        let start = Point::new(10, 10);
        let dest = Point::new(11, 11); // Diagonal - should allow

        assert!(can_step(&dungeon, &tile_props, start, dest));
    }
}
