//! Player Controls - Character movement and actions
//!
//! # M63: Player Controls
//!
//! This module handles player input processing:
//! - Movement (keyboard, controller stick/d-pad)
//! - Combat actions (attack, cast spell)
//! - Inventory interaction
//! - Object interaction
//!
//! ## C++ References
//! - `Source/controls/plrctrls.cpp` (2,260 lines)
//! - `Source/controls/plrctrls.h`

#![allow(dead_code)]

use super::controller::{AxisDirection, AxisDirectionX, AxisDirectionY, ControllerButton};

// =============================================================================
// Constants
// =============================================================================

/// Inventory slot constants
pub const SLOTXY_INV_FIRST: i32 = 7;
pub const SLOTXY_INV_LAST: i32 = 46;
pub const SLOTXY_BELT_FIRST: i32 = 47;
pub const SLOTXY_BELT_LAST: i32 = 54;

/// Invalid stash point
pub const INVALID_STASH_POINT: (i32, i32) = (-1, -1);

// =============================================================================
// Direction Mapping
// =============================================================================

/// Direction enumeration (matches C++ Direction enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    South = 0,
    SouthWest = 1,
    West = 2,
    NorthWest = 3,
    North = 4,
    NorthEast = 5,
    East = 6,
    SouthEast = 7,
}

impl Direction {
    /// Get dx for this direction
    pub fn dx(&self) -> i32 {
        match self {
            Self::South => 0,
            Self::SouthWest => -1,
            Self::West => -1,
            Self::NorthWest => -1,
            Self::North => 0,
            Self::NorthEast => 1,
            Self::East => 1,
            Self::SouthEast => 1,
        }
    }
    
    /// Get dy for this direction
    pub fn dy(&self) -> i32 {
        match self {
            Self::South => 1,
            Self::SouthWest => 0,
            Self::West => -1,
            Self::NorthWest => -1,
            Self::North => -1,
            Self::NorthEast => 0,
            Self::East => 1,
            Self::SouthEast => 1,
        }
    }
    
    /// Convert from axis direction
    pub fn from_axis(axis: AxisDirection) -> Option<Self> {
        match (axis.x, axis.y) {
            (AxisDirectionX::None, AxisDirectionY::None) => None,
            (AxisDirectionX::None, AxisDirectionY::Up) => Some(Self::North),
            (AxisDirectionX::None, AxisDirectionY::Down) => Some(Self::South),
            (AxisDirectionX::Left, AxisDirectionY::None) => Some(Self::West),
            (AxisDirectionX::Right, AxisDirectionY::None) => Some(Self::East),
            (AxisDirectionX::Left, AxisDirectionY::Up) => Some(Self::NorthWest),
            (AxisDirectionX::Right, AxisDirectionY::Up) => Some(Self::NorthEast),
            (AxisDirectionX::Left, AxisDirectionY::Down) => Some(Self::SouthWest),
            (AxisDirectionX::Right, AxisDirectionY::Down) => Some(Self::SouthEast),
        }
    }
    
    /// Get direction from two points
    pub fn from_offset(dx: i32, dy: i32) -> Self {
        // Normalize to -1, 0, 1
        let nx = dx.signum();
        let ny = dy.signum();
        
        match (nx, ny) {
            (0, 1) => Self::South,
            (-1, 0) => Self::West,
            (-1, 1) => Self::SouthWest,
            (-1, -1) => Self::NorthWest,
            (0, -1) => Self::North,
            (1, -1) => Self::NorthEast,
            (1, 0) => Self::East,
            (1, 1) => Self::SouthEast,
            _ => Self::South, // Default
        }
    }
    
    /// Opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            Self::South => Self::North,
            Self::SouthWest => Self::NorthEast,
            Self::West => Self::East,
            Self::NorthWest => Self::SouthEast,
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::East => Self::West,
            Self::SouthEast => Self::NorthWest,
        }
    }
}

/// Face direction lookup table based on input
///
/// **C++ Reference**: `FaceDir` in plrctrls.cpp
const FACE_DIR: [[Direction; 3]; 3] = [
    // X: NONE, Y: NONE/UP/DOWN
    [Direction::South, Direction::North, Direction::South],
    // X: LEFT
    [Direction::West, Direction::NorthWest, Direction::SouthWest],
    // X: RIGHT
    [Direction::East, Direction::NorthEast, Direction::SouthEast],
];

/// Get facing direction from axis input
pub fn get_face_direction(axis: AxisDirection) -> Direction {
    let x_index = match axis.x {
        AxisDirectionX::None => 0,
        AxisDirectionX::Left => 1,
        AxisDirectionX::Right => 2,
    };
    
    let y_index = match axis.y {
        AxisDirectionY::None => 0,
        AxisDirectionY::Up => 1,
        AxisDirectionY::Down => 2,
    };
    
    FACE_DIR[x_index][y_index]
}

// =============================================================================
// Game Action Type
// =============================================================================

/// Game action type enumeration
///
/// **C++ Reference**: `Source/controls/game_controls.h` - `GameActionType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GameActionType {
    /// No action
    None = 0,
    /// Use item at belt slot
    UseBeltItem = 1,
    /// Use item at slot
    UseItem = 2,
    /// Primary action (attack/interact)
    PrimaryAction = 3,
    /// Secondary action (spell)
    SecondaryAction = 4,
    /// Cast spell
    CastSpell = 5,
    /// Toggle inventory
    ToggleInventory = 6,
    /// Toggle character screen
    ToggleCharacter = 7,
    /// Toggle quest log
    ToggleQuestLog = 8,
    /// Toggle spell book
    ToggleSpellBook = 9,
    /// Toggle map
    ToggleMap = 10,
    /// Send message
    SendMessage = 11,
    /// Quick save
    QuickSave = 12,
    /// Quick load
    QuickLoad = 13,
    /// Pause game
    Pause = 14,
    /// Open menu
    OpenMenu = 15,
    /// Stand still toggle
    StandToggle = 16,
    /// Speed book selection
    SpeedBook = 17,
}

impl Default for GameActionType {
    fn default() -> Self {
        Self::None
    }
}

// =============================================================================
// Cursor Target
// =============================================================================

/// Cursor target type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorTargetType {
    /// No target
    None,
    /// Targeting a monster
    Monster,
    /// Targeting a player
    Player,
    /// Targeting an item
    Item,
    /// Targeting an object
    Object,
    /// Targeting a trigger (stairs, etc.)
    Trigger,
    /// Targeting a missile
    Missile,
    /// Targeting a towner (NPC)
    Towner,
}

/// Cursor target info
#[derive(Debug, Clone, Copy)]
pub struct CursorTarget {
    /// Target type
    pub target_type: CursorTargetType,
    /// Target ID (monster index, item index, etc.)
    pub target_id: i32,
    /// Target position
    pub position: (i32, i32),
}

impl CursorTarget {
    pub fn none() -> Self {
        Self {
            target_type: CursorTargetType::None,
            target_id: -1,
            position: (0, 0),
        }
    }
    
    pub fn monster(id: i32, pos: (i32, i32)) -> Self {
        Self {
            target_type: CursorTargetType::Monster,
            target_id: id,
            position: pos,
        }
    }
    
    pub fn player(id: i32, pos: (i32, i32)) -> Self {
        Self {
            target_type: CursorTargetType::Player,
            target_id: id,
            position: pos,
        }
    }
    
    pub fn item(id: i32, pos: (i32, i32)) -> Self {
        Self {
            target_type: CursorTargetType::Item,
            target_id: id,
            position: pos,
        }
    }
    
    pub fn object(id: i32, pos: (i32, i32)) -> Self {
        Self {
            target_type: CursorTargetType::Object,
            target_id: id,
            position: pos,
        }
    }
    
    pub fn is_none(&self) -> bool {
        self.target_type == CursorTargetType::None
    }
    
    pub fn is_monster(&self) -> bool {
        self.target_type == CursorTargetType::Monster
    }
    
    pub fn is_player(&self) -> bool {
        self.target_type == CursorTargetType::Player
    }
}

impl Default for CursorTarget {
    fn default() -> Self {
        Self::none()
    }
}

// =============================================================================
// Player Controls State
// =============================================================================

/// Player controls state
///
/// **C++ Reference**: Global state in plrctrls.cpp
pub struct PlayerControls {
    /// Currently held action
    pub action_held: GameActionType,
    /// Stand still toggle
    pub stand_toggle: bool,
    /// Current cursor target
    pub cursor_target: CursorTarget,
    /// Current cursor trigger
    pub cursor_trigger: i32,
    /// Current cursor missile
    pub cursor_missile: i32,
    /// Current cursor quest
    pub cursor_quest: i32,
    /// Current inventory slot
    pub inv_slot: i32,
    /// Active stash slot
    pub stash_slot: (i32, i32),
    /// Previous inventory column
    pub prev_inv_column: i32,
    /// Belt returns to stash flag
    pub belt_returns_to_stash: bool,
    /// Movement direction
    pub move_direction: AxisDirection,
}

impl PlayerControls {
    pub fn new() -> Self {
        Self {
            action_held: GameActionType::None,
            stand_toggle: false,
            cursor_target: CursorTarget::none(),
            cursor_trigger: -1,
            cursor_missile: -1,
            cursor_quest: -1,
            inv_slot: SLOTXY_INV_FIRST,
            stash_slot: INVALID_STASH_POINT,
            prev_inv_column: -1,
            belt_returns_to_stash: false,
            move_direction: AxisDirection::NONE,
        }
    }
    
    /// Check if in game menu
    ///
    /// **C++ Reference**: `InGameMenu()` in plrctrls.cpp
    pub fn in_game_menu(&self) -> bool {
        // This would check various game states
        false // Placeholder
    }
    
    /// Get rotary distance to target
    ///
    /// **C++ Reference**: `GetRotaryDistance()` in plrctrls.cpp
    pub fn get_rotary_distance(&self, player_pos: (i32, i32), player_dir: Direction, target: (i32, i32)) -> i32 {
        if player_pos == target {
            return -1;
        }
        
        let target_dir = Direction::from_offset(target.0 - player_pos.0, target.1 - player_pos.1);
        
        let d1 = player_dir as i32;
        let d2 = target_dir as i32;
        
        let d = (d1 - d2).abs();
        if d > 4 {
            4 - (d % 4)
        } else {
            d
        }
    }
    
    /// Get minimum distance to target
    ///
    /// **C++ Reference**: `GetMinDistance()` in plrctrls.cpp
    pub fn get_min_distance(&self, from: (i32, i32), to: (i32, i32)) -> i32 {
        let dx = (from.0 - to.0).abs();
        let dy = (from.1 - to.1).abs();
        dx.max(dy)
    }
    
    /// Reset state
    pub fn reset(&mut self) {
        self.action_held = GameActionType::None;
        self.cursor_target = CursorTarget::none();
        self.cursor_trigger = -1;
        self.cursor_missile = -1;
        self.cursor_quest = -1;
        self.move_direction = AxisDirection::NONE;
    }
    
    /// Set movement direction from controller input
    pub fn set_move_direction(&mut self, axis: AxisDirection) {
        self.move_direction = axis;
    }
    
    /// Get movement direction
    pub fn get_move_direction(&self) -> Option<Direction> {
        Direction::from_axis(self.move_direction)
    }
    
    /// Set cursor target to monster
    pub fn target_monster(&mut self, monster_id: i32, pos: (i32, i32)) {
        self.cursor_target = CursorTarget::monster(monster_id, pos);
    }
    
    /// Set cursor target to item
    pub fn target_item(&mut self, item_id: i32, pos: (i32, i32)) {
        self.cursor_target = CursorTarget::item(item_id, pos);
    }
    
    /// Clear cursor target
    pub fn clear_target(&mut self) {
        self.cursor_target = CursorTarget::none();
    }
    
    /// Toggle stand still mode
    pub fn toggle_stand(&mut self) {
        self.stand_toggle = !self.stand_toggle;
    }
    
    /// Navigate inventory slot
    ///
    /// The inventory is a flat 10-wide grid (slots `SLOTXY_INV_FIRST..=SLOTXY_INV_LAST`,
    /// 4 rows x 10 columns). The `Direction` enum uses isometric (iso) deltas where
    /// `East` is `(dx=1, dy=1)` and would jump `+1 + 1*10 = +11` (a diagonal move).
    /// For flat-grid navigation we instead map each direction to a grid
    /// (column, row) delta: East/West move one column (+/-1), South/North move one
    /// row (+/-10), and the diagonal directions combine both.
    pub fn navigate_inventory(&mut self, direction: Direction) {
        let (col_delta, row_delta): (i32, i32) = match direction {
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::South => (0, 1),
            Direction::North => (0, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
        };

        // Flat grid navigation: each row is 10 slots wide.
        let new_slot = self.inv_slot + col_delta + row_delta * 10;
        if new_slot >= SLOTXY_INV_FIRST && new_slot <= SLOTXY_INV_LAST {
            self.inv_slot = new_slot;
        }
    }
    
    /// Navigate to belt
    pub fn navigate_to_belt(&mut self, slot: i32) {
        if slot >= 0 && slot < 8 {
            self.inv_slot = SLOTXY_BELT_FIRST + slot;
        }
    }
    
    /// Is currently in inventory area
    pub fn in_inventory_area(&self) -> bool {
        self.inv_slot >= SLOTXY_INV_FIRST && self.inv_slot <= SLOTXY_INV_LAST
    }
    
    /// Is currently in belt area
    pub fn in_belt_area(&self) -> bool {
        self.inv_slot >= SLOTXY_BELT_FIRST && self.inv_slot <= SLOTXY_BELT_LAST
    }
}

impl Default for PlayerControls {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_dx_dy() {
        assert_eq!(Direction::North.dx(), 0);
        assert_eq!(Direction::North.dy(), -1);
        
        assert_eq!(Direction::East.dx(), 1);
        assert_eq!(Direction::East.dy(), 1);
        
        assert_eq!(Direction::NorthWest.dx(), -1);
        assert_eq!(Direction::NorthWest.dy(), -1);
    }

    #[test]
    fn test_direction_from_axis() {
        let axis = AxisDirection::new(AxisDirectionX::Left, AxisDirectionY::Up);
        assert_eq!(Direction::from_axis(axis), Some(Direction::NorthWest));
        
        let none = AxisDirection::NONE;
        assert_eq!(Direction::from_axis(none), None);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::NorthWest.opposite(), Direction::SouthEast);
    }

    #[test]
    fn test_get_face_direction() {
        let left_up = AxisDirection::new(AxisDirectionX::Left, AxisDirectionY::Up);
        assert_eq!(get_face_direction(left_up), Direction::NorthWest);
        
        let down = AxisDirection::new(AxisDirectionX::None, AxisDirectionY::Down);
        assert_eq!(get_face_direction(down), Direction::South);
    }

    #[test]
    fn test_cursor_target() {
        let none = CursorTarget::none();
        assert!(none.is_none());
        
        let monster = CursorTarget::monster(5, (10, 20));
        assert!(monster.is_monster());
        assert_eq!(monster.target_id, 5);
    }

    #[test]
    fn test_player_controls() {
        let mut controls = PlayerControls::new();
        
        controls.target_monster(3, (15, 25));
        assert!(controls.cursor_target.is_monster());
        assert_eq!(controls.cursor_target.target_id, 3);
        
        controls.clear_target();
        assert!(controls.cursor_target.is_none());
    }

    #[test]
    fn test_rotary_distance() {
        let controls = PlayerControls::new();
        
        // Same position
        assert_eq!(controls.get_rotary_distance((5, 5), Direction::North, (5, 5)), -1);
        
        // Facing target
        assert_eq!(controls.get_rotary_distance((5, 5), Direction::North, (5, 0)), 0);
    }

    #[test]
    fn test_min_distance() {
        let controls = PlayerControls::new();
        
        assert_eq!(controls.get_min_distance((0, 0), (3, 4)), 4);
        assert_eq!(controls.get_min_distance((0, 0), (5, 3)), 5);
    }

    #[test]
    fn test_inventory_navigation() {
        let mut controls = PlayerControls::new();
        controls.inv_slot = SLOTXY_INV_FIRST;
        
        controls.navigate_inventory(Direction::East);
        assert_eq!(controls.inv_slot, SLOTXY_INV_FIRST + 1);
    }

    #[test]
    fn test_belt_navigation() {
        let mut controls = PlayerControls::new();
        
        controls.navigate_to_belt(3);
        assert!(controls.in_belt_area());
        assert_eq!(controls.inv_slot, SLOTXY_BELT_FIRST + 3);
    }

    #[test]
    fn test_stand_toggle() {
        let mut controls = PlayerControls::new();
        assert!(!controls.stand_toggle);
        
        controls.toggle_stand();
        assert!(controls.stand_toggle);
        
        controls.toggle_stand();
        assert!(!controls.stand_toggle);
    }
}
