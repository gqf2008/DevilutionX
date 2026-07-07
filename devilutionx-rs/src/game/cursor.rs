//! Cursor System - M22
//!
//! Precision port of DevilutionX cursor.cpp
//! Handles cursor tracking, selection, and item cursor rendering

// ============================================================================
// Constants
// ============================================================================

/// Cursor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum CursorType {
    /// Default hand cursor
    #[default]
    Hand = 0,
    /// Hourglass (loading)
    Hourglass = 1,
    /// No interaction possible
    None = 2,
    /// Identify item
    Identify = 3,
    /// Repair item
    Repair = 4,
    /// Recharge staff
    Recharge = 5,
    /// Disarm trap
    Disarm = 6,
    /// Weapon curse
    Oil = 7,
    /// Telekinesis spell
    Telekinesis = 8,
    /// Heal other player
    HealOther = 9,
    /// Resurrect player
    Resurrect = 10,
    /// First item cursor index
    FirstItem = 11,
}

impl CursorType {
    /// Check if cursor is an item
    pub fn is_item(&self) -> bool {
        (*self as i32) >= CursorType::FirstItem as i32
    }

    /// Check if cursor is a spell target cursor
    pub fn is_spell_target(&self) -> bool {
        matches!(self, Self::HealOther | Self::Resurrect | Self::Telekinesis)
    }

    /// Check if cursor is a player-only target
    pub fn is_player_target_only(&self) -> bool {
        matches!(self, Self::HealOther | Self::Resurrect)
    }

    /// Create from raw value
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Hand,
            1 => Self::Hourglass,
            2 => Self::None,
            3 => Self::Identify,
            4 => Self::Repair,
            5 => Self::Recharge,
            6 => Self::Disarm,
            7 => Self::Oil,
            8 => Self::Telekinesis,
            9 => Self::HealOther,
            10 => Self::Resurrect,
            v if v >= 11 => Self::FirstItem,
            _ => Self::Hand,
        }
    }
}

/// Monster selection region flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SelectionRegion {
    /// Top part of monster
    Top = 1,
    /// Middle part of monster
    Middle = 2,
    /// Bottom part of monster
    Bottom = 4,
}

impl SelectionRegion {
    pub fn has_any(&self, flags: u8) -> bool {
        (*self as u8) & flags != 0
    }
}

// ============================================================================
// Geometric Types
// ============================================================================

/// 2D Point
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// 2D Displacement
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Displacement {
    pub dx: i32,
    pub dy: i32,
}

impl Displacement {
    pub const fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }
}

/// 2D Size
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

/// Rectangle
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            position: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y < self.position.y + self.size.height
    }
}

// ============================================================================
// Tile Constants
// ============================================================================

/// Tile width in pixels
pub const TILE_WIDTH: i32 = 64;

/// Tile height in pixels
pub const TILE_HEIGHT: i32 = 32;

/// Maximum dungeon X dimension (i32, for cursor coordinate arithmetic).
///
/// This mirrors the canonical `crate::levels::types::MAXDUNX` (= 112) but is
/// kept as `i32` here because cursor code performs signed coordinate math.
/// The `usize`/`i32` split is intentional — do not unify the types.
pub const MAXDUNX: i32 = 112;

/// Maximum dungeon Y dimension (i32, see `MAXDUNX` above).
pub const MAXDUNY: i32 = 112;

// ============================================================================
// Cursor State
// ============================================================================

/// Global cursor state
#[derive(Debug, Default)]
pub struct CursorState {
    /// Current highlighted monster (-1 if none)
    pub monster_id: i32,
    /// Current highlighted item in inventory (-1 if none)
    pub inv_item: i8,
    /// Current highlighted stash item
    pub stash_item: u16,
    /// Current highlighted ground item (-1 if none)
    pub item_id: i8,
    /// Current highlighted object (index or -1)
    pub object_id: i32,
    /// Current highlighted player ID (-1 if none)
    pub player_id: i32,
    /// Current highlighted tile position
    pub position: Point,
    /// Previously highlighted monster (for target retention)
    pub prev_monster: i32,
    /// Current cursor type
    pub cursor_type: CursorType,
    /// Current item cursor ID (if holding item)
    pub item_cursor_id: i32,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            monster_id: -1,
            inv_item: -1,
            stash_item: 0xFFFF, // Empty cell
            item_id: -1,
            object_id: -1,
            player_id: -1,
            position: Point::default(),
            prev_monster: -1,
            cursor_type: CursorType::Hand,
            item_cursor_id: 0,
        }
    }

    /// Clear all cursor targeting info
    pub fn clear_targets(&mut self) {
        self.prev_monster = self.monster_id;
        self.monster_id = -1;
        self.object_id = -1;
        self.item_id = -1;
        self.inv_item = -1;
        self.stash_item = 0xFFFF;
        self.player_id = -1;
    }

    /// Check if any target is selected
    pub fn has_target(&self) -> bool {
        self.monster_id != -1
            || self.object_id != -1
            || self.item_id != -1
            || self.player_id != -1
    }

    /// Check if cursor is over a monster
    pub fn has_monster(&self) -> bool {
        self.monster_id != -1
    }

    /// Check if cursor is over an object
    pub fn has_object(&self) -> bool {
        self.object_id != -1
    }

    /// Check if cursor is over an item
    pub fn has_item(&self) -> bool {
        self.item_id != -1
    }

    /// Check if cursor is over inventory item
    pub fn has_inv_item(&self) -> bool {
        self.inv_item != -1
    }

    /// Check if cursor is over player
    pub fn has_player(&self) -> bool {
        self.player_id != -1
    }

    /// Set cursor to hold an item
    pub fn set_item_cursor(&mut self, item_cursor_id: i32) {
        self.item_cursor_id = item_cursor_id;
        self.cursor_type = CursorType::FirstItem;
    }

    /// Clear item cursor
    pub fn clear_item_cursor(&mut self) {
        self.item_cursor_id = 0;
        self.cursor_type = CursorType::Hand;
    }
}

// ============================================================================
// Cursor Manager
// ============================================================================

/// Cursor management system
pub struct CursorManager {
    /// Current cursor state
    pub state: CursorState,
    /// Mouse position
    pub mouse_position: Point,
    /// Screen dimensions
    screen_width: i32,
    screen_height: i32,
    /// View position (camera)
    pub view_position: Point,
    /// Main panel rectangle
    pub main_panel: Rectangle,
    /// Left panel rectangle
    pub left_panel: Rectangle,
    /// Right panel rectangle
    pub right_panel: Rectangle,
    /// Is inventory open
    pub inv_flag: bool,
    /// Is spellbook open
    pub spellbook_flag: bool,
    /// Is stash open
    pub stash_open: bool,
    /// Spell select mode active
    pub spell_select_flag: bool,
    /// Current level type (0=town, 1-6=dungeon)
    pub level_type: i32,
    /// Is game zoomed
    pub zoom_enabled: bool,
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new(640, 480)
    }
}

impl CursorManager {
    /// Create new cursor manager
    pub fn new(screen_width: i32, screen_height: i32) -> Self {
        let panel_height = 128;
        let main_panel = Rectangle::new(
            (screen_width - 640) / 2,
            screen_height - panel_height,
            640,
            panel_height,
        );

        let side_panel_width = 320;
        let side_panel_height = 352;
        let left_panel = Rectangle::new(
            0,
            (screen_height - side_panel_height - panel_height) / 2,
            side_panel_width,
            side_panel_height,
        );
        let right_panel = Rectangle::new(
            screen_width - side_panel_width,
            (screen_height - side_panel_height - panel_height) / 2,
            side_panel_width,
            side_panel_height,
        );

        Self {
            state: CursorState::new(),
            mouse_position: Point::default(),
            screen_width,
            screen_height,
            view_position: Point::default(),
            main_panel,
            left_panel,
            right_panel,
            inv_flag: false,
            spellbook_flag: false,
            stash_open: false,
            spell_select_flag: false,
            level_type: 0,
            zoom_enabled: false,
        }
    }

    /// Update mouse position
    pub fn set_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_position = Point::new(x, y);
    }

    /// Check if left panel is open
    pub fn is_left_panel_open(&self) -> bool {
        self.stash_open
    }

    /// Check if right panel is open
    pub fn is_right_panel_open(&self) -> bool {
        self.inv_flag || self.spellbook_flag
    }

    /// Convert screen position to tile grid
    pub fn screen_to_tile(&self, screen_pos: Point) -> Point {
        let mut pos = screen_pos;

        // Adjust for zoom
        if self.zoom_enabled {
            pos.y -= TILE_HEIGHT / 4;
        }

        let tx = pos.x / TILE_WIDTH;
        let ty = pos.y / TILE_HEIGHT;

        Point::new(self.view_position.x + tx - ty, self.view_position.y + tx + ty)
    }

    /// Check if position is in dungeon bounds
    pub fn in_dungeon_bounds(&self, pos: Point) -> bool {
        pos.x >= 0 && pos.x < MAXDUNX && pos.y >= 0 && pos.y < MAXDUNY
    }

    /// Shift position to diamond grid alignment
    pub fn align_to_diamond(&self, screen_pos: Point, tile: &mut Point) -> bool {
        let px = screen_pos.x % TILE_WIDTH;
        let py = screen_pos.y % TILE_HEIGHT;

        let flip_y = py < (px / 2);
        if flip_y {
            tile.y -= 1;
        }

        let flip_x = py >= TILE_HEIGHT - (px / 2);
        if flip_x {
            tile.x += 1;
        }

        // Clamp to bounds
        tile.x = tile.x.clamp(0, MAXDUNX - 1);
        tile.y = tile.y.clamp(0, MAXDUNY - 1);

        // Calculate flip flag
        (flip_y && flip_x) || ((flip_y || flip_x) && px < TILE_WIDTH / 2)
    }

    /// Check if mouse is over main panel
    pub fn is_over_main_panel(&self) -> bool {
        self.main_panel.contains(self.mouse_position)
    }

    /// Check if mouse is over left panel
    pub fn is_over_left_panel(&self) -> bool {
        self.left_panel.contains(self.mouse_position)
    }

    /// Check if mouse is over right panel
    pub fn is_over_right_panel(&self) -> bool {
        self.right_panel.contains(self.mouse_position)
    }

    /// Check panels and return true if cursor is over UI
    pub fn check_panels(&mut self) -> bool {
        if self.is_over_main_panel() {
            return true;
        }

        if self.inv_flag && self.is_over_right_panel() {
            // Check inventory highlight
            // self.state.inv_item = self.check_inv_highlight();
            return true;
        }

        if self.stash_open && self.is_over_left_panel() {
            // Check stash highlight
            // self.state.stash_item = self.check_stash_highlight();
            return true;
        }

        if self.spellbook_flag && self.is_over_right_panel() {
            return true;
        }

        if self.is_left_panel_open() && self.is_over_left_panel() {
            return true;
        }

        false
    }

    /// Update cursor for current frame
    pub fn update(&mut self) {
        // Clear previous targeting
        self.state.clear_targets();

        // Check if over UI panels
        if self.check_panels() {
            return;
        }

        // Convert screen position to tile
        let adjusted_pos = Point::new(
            self.mouse_position.x - self.screen_width / 2,
            self.mouse_position.y - self.screen_height / 2,
        );

        let mut tile = self.screen_to_tile(adjusted_pos);
        let _flip_flag = self.align_to_diamond(adjusted_pos, &mut tile);

        if !self.in_dungeon_bounds(tile) {
            return;
        }

        self.state.position = tile;

        // Target selection would happen here based on game state
        // This requires integration with monster/object/item systems
    }

    /// Get cursor size for current cursor type
    pub fn get_cursor_size(&self) -> Size {
        match self.state.cursor_type {
            CursorType::Hand => Size::new(33, 32),
            CursorType::Hourglass => Size::new(33, 32),
            CursorType::None => Size::new(33, 32),
            CursorType::Identify => Size::new(33, 32),
            CursorType::Repair => Size::new(33, 32),
            CursorType::Recharge => Size::new(33, 32),
            CursorType::Disarm => Size::new(33, 32),
            CursorType::Oil => Size::new(33, 32),
            CursorType::Telekinesis => Size::new(33, 32),
            CursorType::HealOther => Size::new(33, 32),
            CursorType::Resurrect => Size::new(33, 32),
            CursorType::FirstItem => {
                // Item cursor size varies by item
                self.get_item_cursor_size(self.state.item_cursor_id)
            }
        }
    }

    /// Get item cursor size by ID
    pub fn get_item_cursor_size(&self, _cursor_id: i32) -> Size {
        // Default item cursor size (1x1 slot)
        Size::new(28, 28)
    }

    /// Check if cursor is in town
    pub fn is_in_town(&self) -> bool {
        self.level_type == 0
    }
}

// ============================================================================
// Item Cursor Helpers
// ============================================================================

/// Inventory item size in grid cells
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemGridSize {
    pub width: u8,
    pub height: u8,
}

impl ItemGridSize {
    pub const fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }

    pub const ONE_BY_ONE: Self = Self::new(1, 1);
    pub const ONE_BY_TWO: Self = Self::new(1, 2);
    pub const ONE_BY_THREE: Self = Self::new(1, 3);
    pub const TWO_BY_TWO: Self = Self::new(2, 2);
    pub const TWO_BY_THREE: Self = Self::new(2, 3);
}

/// Get inventory grid size for cursor ID
pub fn get_inv_item_grid_size(cursor_id: i32) -> ItemGridSize {
    // Based on cursor ID ranges
    match cursor_id {
        // 1x1 items (rings, amulets, potions, scrolls, gold)
        1..=50 => ItemGridSize::ONE_BY_ONE,
        // 1x2 items (small weapons)
        51..=100 => ItemGridSize::ONE_BY_TWO,
        // 1x3 items (staves, bows)
        101..=150 => ItemGridSize::ONE_BY_THREE,
        // 2x2 items (shields, helms)
        151..=200 => ItemGridSize::TWO_BY_TWO,
        // 2x3 items (armor, large weapons)
        201..=250 => ItemGridSize::TWO_BY_THREE,
        _ => ItemGridSize::ONE_BY_ONE,
    }
}

// ============================================================================
// Selection Helpers
// ============================================================================

/// Target type for cursor selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    None,
    Monster,
    Player,
    Object,
    Item,
    Towner,
}

/// Selection result
#[derive(Debug, Clone, Copy)]
pub struct SelectionResult {
    pub target_type: TargetType,
    pub target_id: i32,
    pub position: Point,
}

impl Default for SelectionResult {
    fn default() -> Self {
        Self {
            target_type: TargetType::None,
            target_id: -1,
            position: Point::default(),
        }
    }
}

impl SelectionResult {
    pub fn monster(id: i32, pos: Point) -> Self {
        Self {
            target_type: TargetType::Monster,
            target_id: id,
            position: pos,
        }
    }

    pub fn player(id: i32, pos: Point) -> Self {
        Self {
            target_type: TargetType::Player,
            target_id: id,
            position: pos,
        }
    }

    pub fn object(id: i32, pos: Point) -> Self {
        Self {
            target_type: TargetType::Object,
            target_id: id,
            position: pos,
        }
    }

    pub fn item(id: i32, pos: Point) -> Self {
        Self {
            target_type: TargetType::Item,
            target_id: id,
            position: pos,
        }
    }

    pub fn towner(id: i32, pos: Point) -> Self {
        Self {
            target_type: TargetType::Towner,
            target_id: id,
            position: pos,
        }
    }

    pub fn has_target(&self) -> bool {
        self.target_type != TargetType::None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_type() {
        assert!(!CursorType::Hand.is_item());
        assert!(CursorType::FirstItem.is_item());
        assert!(CursorType::HealOther.is_player_target_only());
        assert!(CursorType::Telekinesis.is_spell_target());
        assert!(!CursorType::Telekinesis.is_player_target_only());
    }

    #[test]
    fn test_cursor_type_from_i32() {
        assert_eq!(CursorType::from_i32(0), CursorType::Hand);
        assert_eq!(CursorType::from_i32(3), CursorType::Identify);
        assert_eq!(CursorType::from_i32(11), CursorType::FirstItem);
        assert_eq!(CursorType::from_i32(100), CursorType::FirstItem);
    }

    #[test]
    fn test_cursor_state_init() {
        let state = CursorState::new();
        assert_eq!(state.monster_id, -1);
        assert_eq!(state.item_id, -1);
        assert_eq!(state.cursor_type, CursorType::Hand);
        assert!(!state.has_target());
    }

    #[test]
    fn test_cursor_state_clear() {
        let mut state = CursorState::new();
        state.monster_id = 5;
        state.item_id = 3;

        state.clear_targets();

        assert_eq!(state.monster_id, -1);
        assert_eq!(state.item_id, -1);
        assert_eq!(state.prev_monster, 5);
    }

    #[test]
    fn test_cursor_state_item_cursor() {
        let mut state = CursorState::new();

        state.set_item_cursor(25);
        assert_eq!(state.item_cursor_id, 25);
        assert_eq!(state.cursor_type, CursorType::FirstItem);

        state.clear_item_cursor();
        assert_eq!(state.item_cursor_id, 0);
        assert_eq!(state.cursor_type, CursorType::Hand);
    }

    #[test]
    fn test_cursor_manager_init() {
        let manager = CursorManager::new(640, 480);
        assert_eq!(manager.screen_width, 640);
        assert_eq!(manager.screen_height, 480);
        assert!(!manager.inv_flag);
        assert!(!manager.spell_select_flag);
    }

    #[test]
    fn test_cursor_manager_panels() {
        let mut manager = CursorManager::new(640, 480);

        // Test main panel detection
        manager.set_mouse_position(320, 420);
        assert!(manager.is_over_main_panel());

        // Test outside panels
        manager.set_mouse_position(320, 200);
        assert!(!manager.is_over_main_panel());
    }

    #[test]
    fn test_in_dungeon_bounds() {
        let manager = CursorManager::new(640, 480);

        assert!(manager.in_dungeon_bounds(Point::new(50, 50)));
        assert!(manager.in_dungeon_bounds(Point::new(0, 0)));
        assert!(manager.in_dungeon_bounds(Point::new(111, 111)));
        assert!(!manager.in_dungeon_bounds(Point::new(-1, 0)));
        assert!(!manager.in_dungeon_bounds(Point::new(112, 0)));
    }

    #[test]
    fn test_item_grid_size() {
        assert_eq!(get_inv_item_grid_size(10), ItemGridSize::ONE_BY_ONE);
        assert_eq!(get_inv_item_grid_size(75), ItemGridSize::ONE_BY_TWO);
        assert_eq!(get_inv_item_grid_size(125), ItemGridSize::ONE_BY_THREE);
        assert_eq!(get_inv_item_grid_size(175), ItemGridSize::TWO_BY_TWO);
        assert_eq!(get_inv_item_grid_size(225), ItemGridSize::TWO_BY_THREE);
    }

    #[test]
    fn test_selection_result() {
        let result = SelectionResult::default();
        assert!(!result.has_target());

        let monster = SelectionResult::monster(5, Point::new(10, 20));
        assert!(monster.has_target());
        assert_eq!(monster.target_type, TargetType::Monster);
        assert_eq!(monster.target_id, 5);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(10, 20, 100, 50);
        assert!(rect.contains(Point::new(50, 40)));
        assert!(!rect.contains(Point::new(5, 40)));
        assert!(!rect.contains(Point::new(111, 40)));
    }

    #[test]
    fn test_is_in_town() {
        let mut manager = CursorManager::new(640, 480);

        manager.level_type = 0;
        assert!(manager.is_in_town());

        manager.level_type = 1;
        assert!(!manager.is_in_town());
    }
}
