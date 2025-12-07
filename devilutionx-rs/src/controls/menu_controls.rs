//! Menu Controls - Menu navigation input handling
//!
//! # M63: Menu Controls
//!
//! This module handles menu input processing:
//! - Menu navigation (up/down/select)
//! - Dialog interaction
//! - List scrolling
//!
//! ## C++ References
//! - `Source/controls/menu_controls.cpp`
//! - `Source/controls/menu_controls.h`

#![allow(dead_code)]

use super::controller::{AxisDirection, AxisDirectionX, AxisDirectionY, ControllerButton};
use super::game_controls::KeyCode;

// =============================================================================
// Menu Action
// =============================================================================

/// Menu action enumeration
///
/// **C++ Reference**: `Source/controls/menu_controls.h` - `MenuAction`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MenuAction {
    /// No action
    None = 0,
    /// Move selection up
    Up = 1,
    /// Move selection down
    Down = 2,
    /// Move selection left
    Left = 3,
    /// Move selection right
    Right = 4,
    /// Select current item
    Select = 5,
    /// Cancel/back
    Back = 6,
    /// Delete character/item
    Delete = 7,
    /// Page up
    PageUp = 8,
    /// Page down
    PageDown = 9,
}

impl Default for MenuAction {
    fn default() -> Self {
        Self::None
    }
}

// =============================================================================
// Menu Controls State
// =============================================================================

/// Menu controls state
pub struct MenuControls {
    /// Current menu action
    pub current_action: MenuAction,
    /// Movement direction
    pub move_direction: AxisDirection,
    /// Key repeat delay (ms)
    pub key_repeat_delay: u32,
    /// Key repeat interval (ms)
    pub key_repeat_interval: u32,
    /// Time of last key press
    pub last_key_time: u32,
    /// Currently held key
    pub held_key: Option<KeyCode>,
    /// Currently held button
    pub held_button: Option<ControllerButton>,
}

impl MenuControls {
    pub fn new() -> Self {
        Self {
            current_action: MenuAction::None,
            move_direction: AxisDirection::NONE,
            key_repeat_delay: 500,
            key_repeat_interval: 100,
            last_key_time: 0,
            held_key: None,
            held_button: None,
        }
    }
    
    /// Process keyboard input
    pub fn process_key(&mut self, key: KeyCode, pressed: bool) -> MenuAction {
        if !pressed {
            if self.held_key == Some(key) {
                self.held_key = None;
            }
            return MenuAction::None;
        }
        
        self.held_key = Some(key);
        
        match key {
            KeyCode::Up => MenuAction::Up,
            KeyCode::Down => MenuAction::Down,
            KeyCode::Left => MenuAction::Left,
            KeyCode::Right => MenuAction::Right,
            KeyCode::Return | KeyCode::Space => MenuAction::Select,
            KeyCode::Escape => MenuAction::Back,
            KeyCode::Delete | KeyCode::Backspace => MenuAction::Delete,
            KeyCode::PageUp => MenuAction::PageUp,
            KeyCode::PageDown => MenuAction::PageDown,
            _ => MenuAction::None,
        }
    }
    
    /// Process controller button
    pub fn process_button(&mut self, button: ControllerButton, pressed: bool) -> MenuAction {
        if !pressed {
            if self.held_button == Some(button) {
                self.held_button = None;
            }
            return MenuAction::None;
        }
        
        self.held_button = Some(button);
        
        match button {
            ControllerButton::DPadUp | ControllerButton::AxisLeftStickUp => MenuAction::Up,
            ControllerButton::DPadDown | ControllerButton::AxisLeftStickDown => MenuAction::Down,
            ControllerButton::DPadLeft | ControllerButton::AxisLeftStickLeft => MenuAction::Left,
            ControllerButton::DPadRight | ControllerButton::AxisLeftStickRight => MenuAction::Right,
            ControllerButton::A => MenuAction::Select,
            ControllerButton::B => MenuAction::Back,
            ControllerButton::Y => MenuAction::Delete,
            ControllerButton::LeftShoulder => MenuAction::PageUp,
            ControllerButton::RightShoulder => MenuAction::PageDown,
            _ => MenuAction::None,
        }
    }
    
    /// Process axis direction
    pub fn process_axis(&mut self, axis: AxisDirection) -> MenuAction {
        if axis == self.move_direction {
            return MenuAction::None;
        }
        
        self.move_direction = axis;
        
        // Convert axis to menu action
        match (axis.x, axis.y) {
            (AxisDirectionX::None, AxisDirectionY::Up) => MenuAction::Up,
            (AxisDirectionX::None, AxisDirectionY::Down) => MenuAction::Down,
            (AxisDirectionX::Left, AxisDirectionY::None) => MenuAction::Left,
            (AxisDirectionX::Right, AxisDirectionY::None) => MenuAction::Right,
            (AxisDirectionX::Left, AxisDirectionY::Up) => MenuAction::Up, // Prioritize vertical
            (AxisDirectionX::Right, AxisDirectionY::Up) => MenuAction::Up,
            (AxisDirectionX::Left, AxisDirectionY::Down) => MenuAction::Down,
            (AxisDirectionX::Right, AxisDirectionY::Down) => MenuAction::Down,
            _ => MenuAction::None,
        }
    }
    
    /// Check for key repeat
    pub fn check_repeat(&self, current_time: u32) -> bool {
        if self.held_key.is_none() && self.held_button.is_none() {
            return false;
        }
        
        let elapsed = current_time.saturating_sub(self.last_key_time);
        if elapsed < self.key_repeat_delay {
            return false;
        }
        
        let repeat_elapsed = elapsed - self.key_repeat_delay;
        repeat_elapsed % self.key_repeat_interval == 0
    }
    
    /// Get current held action
    pub fn get_held_action(&self) -> MenuAction {
        if let Some(key) = self.held_key {
            return self.key_to_action(key);
        }
        if let Some(button) = self.held_button {
            return self.button_to_action(button);
        }
        MenuAction::None
    }
    
    /// Convert key to action
    fn key_to_action(&self, key: KeyCode) -> MenuAction {
        match key {
            KeyCode::Up => MenuAction::Up,
            KeyCode::Down => MenuAction::Down,
            KeyCode::Left => MenuAction::Left,
            KeyCode::Right => MenuAction::Right,
            _ => MenuAction::None,
        }
    }
    
    /// Convert button to action
    fn button_to_action(&self, button: ControllerButton) -> MenuAction {
        match button {
            ControllerButton::DPadUp | ControllerButton::AxisLeftStickUp => MenuAction::Up,
            ControllerButton::DPadDown | ControllerButton::AxisLeftStickDown => MenuAction::Down,
            ControllerButton::DPadLeft | ControllerButton::AxisLeftStickLeft => MenuAction::Left,
            ControllerButton::DPadRight | ControllerButton::AxisLeftStickRight => MenuAction::Right,
            _ => MenuAction::None,
        }
    }
    
    /// Reset state
    pub fn reset(&mut self) {
        self.current_action = MenuAction::None;
        self.move_direction = AxisDirection::NONE;
        self.held_key = None;
        self.held_button = None;
    }
}

impl Default for MenuControls {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Menu Navigation
// =============================================================================

/// Menu navigation helper
pub struct MenuNavigation {
    /// Number of items in menu
    pub item_count: usize,
    /// Currently selected item
    pub selected: usize,
    /// Viewport offset (for scrolling)
    pub viewport_offset: usize,
    /// Viewport size
    pub viewport_size: usize,
    /// Wrap navigation
    pub wrap: bool,
}

impl MenuNavigation {
    pub fn new(item_count: usize, viewport_size: usize) -> Self {
        Self {
            item_count,
            selected: 0,
            viewport_offset: 0,
            viewport_size,
            wrap: true,
        }
    }
    
    /// Process menu action
    pub fn process(&mut self, action: MenuAction) -> bool {
        match action {
            MenuAction::Up => self.move_up(),
            MenuAction::Down => self.move_down(),
            MenuAction::PageUp => self.page_up(),
            MenuAction::PageDown => self.page_down(),
            _ => false,
        }
    }
    
    /// Move selection up
    pub fn move_up(&mut self) -> bool {
        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_viewport();
            true
        } else if self.wrap && self.item_count > 0 {
            self.selected = self.item_count - 1;
            self.adjust_viewport();
            true
        } else {
            false
        }
    }
    
    /// Move selection down
    pub fn move_down(&mut self) -> bool {
        if self.selected < self.item_count.saturating_sub(1) {
            self.selected += 1;
            self.adjust_viewport();
            true
        } else if self.wrap {
            self.selected = 0;
            self.adjust_viewport();
            true
        } else {
            false
        }
    }
    
    /// Page up
    pub fn page_up(&mut self) -> bool {
        if self.selected > 0 {
            self.selected = self.selected.saturating_sub(self.viewport_size);
            self.adjust_viewport();
            true
        } else {
            false
        }
    }
    
    /// Page down
    pub fn page_down(&mut self) -> bool {
        if self.selected < self.item_count.saturating_sub(1) {
            self.selected = (self.selected + self.viewport_size).min(self.item_count.saturating_sub(1));
            self.adjust_viewport();
            true
        } else {
            false
        }
    }
    
    /// Adjust viewport to keep selection visible
    fn adjust_viewport(&mut self) {
        if self.selected < self.viewport_offset {
            self.viewport_offset = self.selected;
        } else if self.selected >= self.viewport_offset + self.viewport_size {
            self.viewport_offset = self.selected - self.viewport_size + 1;
        }
    }
    
    /// Set item count
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        if self.selected >= count {
            self.selected = count.saturating_sub(1);
        }
        self.adjust_viewport();
    }
    
    /// Jump to item
    pub fn jump_to(&mut self, index: usize) -> bool {
        if index < self.item_count {
            self.selected = index;
            self.adjust_viewport();
            true
        } else {
            false
        }
    }
    
    /// Get visible range
    pub fn visible_range(&self) -> (usize, usize) {
        let end = (self.viewport_offset + self.viewport_size).min(self.item_count);
        (self.viewport_offset, end)
    }
    
    /// Is item visible
    pub fn is_visible(&self, index: usize) -> bool {
        index >= self.viewport_offset && index < self.viewport_offset + self.viewport_size
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_controls_key() {
        let mut controls = MenuControls::new();
        
        let action = controls.process_key(KeyCode::Up, true);
        assert_eq!(action, MenuAction::Up);
        
        let action = controls.process_key(KeyCode::Return, true);
        assert_eq!(action, MenuAction::Select);
        
        let action = controls.process_key(KeyCode::Escape, true);
        assert_eq!(action, MenuAction::Back);
    }

    #[test]
    fn test_menu_controls_button() {
        let mut controls = MenuControls::new();
        
        let action = controls.process_button(ControllerButton::DPadUp, true);
        assert_eq!(action, MenuAction::Up);
        
        let action = controls.process_button(ControllerButton::A, true);
        assert_eq!(action, MenuAction::Select);
    }

    #[test]
    fn test_menu_navigation() {
        let mut nav = MenuNavigation::new(10, 5);
        
        assert_eq!(nav.selected, 0);
        
        nav.move_down();
        assert_eq!(nav.selected, 1);
        
        nav.move_up();
        assert_eq!(nav.selected, 0);
    }

    #[test]
    fn test_menu_navigation_wrap() {
        let mut nav = MenuNavigation::new(5, 5);
        nav.wrap = true;
        
        // Wrap up from first item
        nav.move_up();
        assert_eq!(nav.selected, 4);
        
        // Wrap down from last item
        nav.move_down();
        assert_eq!(nav.selected, 0);
    }

    #[test]
    fn test_menu_navigation_no_wrap() {
        let mut nav = MenuNavigation::new(5, 5);
        nav.wrap = false;
        
        // Can't go up from first
        let moved = nav.move_up();
        assert!(!moved);
        assert_eq!(nav.selected, 0);
        
        // Go to end
        nav.selected = 4;
        
        // Can't go down from last
        let moved = nav.move_down();
        assert!(!moved);
        assert_eq!(nav.selected, 4);
    }

    #[test]
    fn test_menu_navigation_viewport() {
        let mut nav = MenuNavigation::new(20, 5);
        
        // Move to item 10
        for _ in 0..10 {
            nav.move_down();
        }
        
        assert_eq!(nav.selected, 10);
        assert!(nav.is_visible(10));
        assert!(!nav.is_visible(0));
    }

    #[test]
    fn test_menu_navigation_page() {
        let mut nav = MenuNavigation::new(20, 5);
        nav.selected = 10;
        
        nav.page_up();
        assert_eq!(nav.selected, 5);
        
        nav.page_down();
        assert_eq!(nav.selected, 10);
    }

    #[test]
    fn test_menu_navigation_visible_range() {
        let mut nav = MenuNavigation::new(20, 5);
        nav.selected = 10;
        nav.adjust_viewport();
        
        let (start, end) = nav.visible_range();
        assert!(start <= nav.selected);
        assert!(end > nav.selected);
        assert_eq!(end - start, nav.viewport_size.min(nav.item_count));
    }
}
