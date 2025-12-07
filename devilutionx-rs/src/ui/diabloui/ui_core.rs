//! UI Core - Main UI Loop and Event Handling
//!
//! This module provides the core UI framework including event handling,
//! focus management, and the main UI loop.
//!
//! ## C++ Alignment
//!
//! - `diabloui.cpp` (UiInitList, UiPollAndRender, UiFocus, etc.)

use super::ui_item::{UiFlags, UiItem, UiItemBase, UiList, UiRect, UiScrollbar};

/// Sound effects used in UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSfx {
    /// Menu item move/scroll sound
    MenuMove,
    /// Menu item selection sound
    MenuSelect,
}

/// Key codes for UI input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum UiKeyCode {
    Unknown = 0,
    Return = 13,
    Escape = 27,
    Space = 32,
    Up = 0x40000052,
    Down = 0x40000051,
    Left = 0x40000050,
    Right = 0x4000004F,
    Tab = 9,
    Backspace = 8,
    Delete = 127,
    Home = 0x4000004A,
    End = 0x4000004D,
    PageUp = 0x4000004B,
    PageDown = 0x4000004E,
}

impl UiKeyCode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            13 => Self::Return,
            27 => Self::Escape,
            32 => Self::Space,
            0x40000052 => Self::Up,
            0x40000051 => Self::Down,
            0x40000050 => Self::Left,
            0x4000004F => Self::Right,
            9 => Self::Tab,
            8 => Self::Backspace,
            127 => Self::Delete,
            0x4000004A => Self::Home,
            0x4000004D => Self::End,
            0x4000004B => Self::PageUp,
            0x4000004E => Self::PageDown,
            _ => Self::Unknown,
        }
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// UI input event
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Key pressed
    KeyDown {
        keycode: UiKeyCode,
        character: Option<char>,
    },
    /// Key released
    KeyUp { keycode: UiKeyCode },
    /// Mouse moved
    MouseMotion { x: i32, y: i32 },
    /// Mouse button pressed
    MouseDown {
        button: MouseButton,
        x: i32,
        y: i32,
    },
    /// Mouse button released
    MouseUp {
        button: MouseButton,
        x: i32,
        y: i32,
    },
    /// Mouse wheel scrolled
    MouseWheel { delta: i32 },
    /// Text input event (for edit fields)
    TextInput { text: String },
    /// Window quit request
    Quit,
}

/// Result of handling a UI event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEventResult {
    /// Event was not handled
    NotHandled,
    /// Event was handled, continue
    Handled,
    /// Event triggered a selection
    Selected(usize),
    /// Event triggered escape/back
    Escape,
    /// Event triggered yes action
    Yes,
    /// Event triggered no action
    No,
}

/// Callback function types
pub type FocusCallback = Box<dyn Fn(usize)>;
pub type SelectCallback = Box<dyn Fn(usize)>;
pub type EscapeCallback = Box<dyn Fn()>;
pub type YesNoCallback = Box<dyn Fn() -> bool>;

/// UI event handler trait
pub trait UiEventHandler {
    /// Handle a UI event
    fn handle_event(&mut self, event: &UiEvent) -> UiEventResult;

    /// Called when focus changes
    fn on_focus_change(&mut self, _index: usize) {}

    /// Called when item is selected
    fn on_select(&mut self, _index: usize) {}

    /// Called on escape
    fn on_escape(&mut self) {}
}

/// UI renderer trait for drawing UI items
pub trait UiRenderer {
    /// Clear the screen
    fn clear(&mut self);

    /// Draw a UI item
    fn render_item(&mut self, item: &UiItem);

    /// Draw all items
    fn render_items(&mut self, items: &[UiItem]) {
        for item in items {
            if !item.is_hidden() {
                self.render_item(item);
            }
        }
    }

    /// Draw the mouse cursor
    fn draw_cursor(&mut self, x: i32, y: i32);

    /// Present the rendered frame
    fn present(&mut self);

    /// Play a sound effect
    fn play_sound(&mut self, sfx: UiSfx);
}

/// UI list state for managing selectable lists
#[derive(Debug)]
pub struct UiListState {
    /// Currently selected item index
    pub selected_index: usize,
    /// Maximum selectable index
    pub max_index: usize,
    /// Number of visible items
    pub viewport_size: usize,
    /// Scroll offset
    pub scroll_offset: usize,
    /// Whether list wraps around
    pub wraps: bool,
    /// Double-click tracking
    pub last_click_index: Option<usize>,
    pub last_click_time: u32,
}

impl Default for UiListState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            max_index: 0,
            viewport_size: 1,
            scroll_offset: 0,
            wraps: false,
            last_click_index: None,
            last_click_time: 0,
        }
    }
}

impl UiListState {
    /// Create new list state
    pub fn new(max_index: usize, viewport_size: usize, wraps: bool) -> Self {
        Self {
            max_index,
            viewport_size,
            wraps,
            ..Default::default()
        }
    }

    /// Adjust scroll offset to show selected item
    pub fn adjust_scroll(&mut self) {
        if self.selected_index >= self.scroll_offset + self.viewport_size {
            self.scroll_offset = self.selected_index.saturating_sub(self.viewport_size - 1);
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) -> bool {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll();
            true
        } else if self.wraps && self.max_index > 0 {
            self.selected_index = self.max_index;
            self.adjust_scroll();
            true
        } else {
            false
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) -> bool {
        if self.selected_index < self.max_index {
            self.selected_index += 1;
            self.adjust_scroll();
            true
        } else if self.wraps {
            self.selected_index = 0;
            self.adjust_scroll();
            true
        } else {
            false
        }
    }

    /// Move selection by page
    pub fn page_up(&mut self) -> bool {
        if self.selected_index > 0 {
            self.selected_index = self.selected_index.saturating_sub(self.viewport_size);
            self.adjust_scroll();
            true
        } else {
            false
        }
    }

    /// Move selection by page down
    pub fn page_down(&mut self) -> bool {
        if self.selected_index < self.max_index {
            self.selected_index = (self.selected_index + self.viewport_size).min(self.max_index);
            self.adjust_scroll();
            true
        } else {
            false
        }
    }

    /// Check for double-click (within 500ms)
    pub fn check_double_click(&mut self, index: usize, current_time: u32) -> bool {
        const DOUBLE_CLICK_TIME_MS: u32 = 500;

        let is_double = if let Some(last_index) = self.last_click_index {
            last_index == index && current_time - self.last_click_time < DOUBLE_CLICK_TIME_MS
        } else {
            false
        };

        self.last_click_index = Some(index);
        self.last_click_time = current_time;

        is_double
    }
}

/// Main UI context managing the current UI state
#[derive(Debug)]
pub struct UiContext {
    /// Current items being displayed
    pub items: Vec<UiItem>,
    /// List state if a list is active
    pub list_state: Option<UiListState>,
    /// Whether text input is active
    pub text_input_active: bool,
    /// Allow empty text input
    pub allow_empty_input: bool,
    /// Fade state (0-256)
    pub fade_value: i32,
    /// Fade start time
    pub fade_start: Option<u32>,
    /// Current mouse position
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl Default for UiContext {
    fn default() -> Self {
        Self::new()
    }
}

impl UiContext {
    /// Create new UI context
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            list_state: None,
            text_input_active: false,
            allow_empty_input: false,
            fade_value: 256,
            fade_start: None,
            mouse_x: 0,
            mouse_y: 0,
        }
    }

    /// Initialize the UI list with items
    pub fn init_list(
        &mut self,
        items: Vec<UiItem>,
        max_index: usize,
        viewport_size: usize,
        wraps: bool,
        selected: usize,
    ) {
        self.items = items;
        self.list_state = Some(UiListState {
            selected_index: selected,
            max_index,
            viewport_size,
            wraps,
            ..Default::default()
        });
        if let Some(state) = &mut self.list_state {
            state.adjust_scroll();
        }
    }

    /// Clear the UI list
    pub fn clear_list(&mut self) {
        self.items.clear();
        self.list_state = None;
        self.text_input_active = false;
    }

    /// Start fade-in effect
    pub fn start_fade_in(&mut self, current_time: u32) {
        self.fade_value = 0;
        self.fade_start = Some(current_time);
    }

    /// Update fade effect
    pub fn update_fade(&mut self, current_time: u32) -> bool {
        if self.fade_value >= 256 {
            return false;
        }

        if let Some(start) = self.fade_start {
            // 32 frames @ 60hz = ~533ms
            self.fade_value = ((current_time - start) as f32 / 2.083) as i32;
            if self.fade_value >= 256 {
                self.fade_value = 256;
                self.fade_start = None;
                return true; // Fade complete
            }
        }
        false
    }

    /// Check if fade is complete
    pub fn is_fade_complete(&self) -> bool {
        self.fade_value >= 256
    }

    /// Get fade alpha (0.0 - 1.0)
    pub fn fade_alpha(&self) -> f32 {
        (self.fade_value as f32 / 256.0).min(1.0)
    }

    /// Handle UI event
    pub fn handle_event(&mut self, event: &UiEvent) -> UiEventResult {
        match event {
            UiEvent::KeyDown { keycode, character } => {
                self.handle_key_down(*keycode, *character)
            }
            UiEvent::MouseMotion { x, y } => {
                self.mouse_x = *x;
                self.mouse_y = *y;
                self.handle_mouse_motion(*x, *y)
            }
            UiEvent::MouseDown { button, x, y } => {
                self.handle_mouse_down(*button, *x, *y)
            }
            UiEvent::MouseWheel { delta } => {
                self.handle_mouse_wheel(*delta)
            }
            UiEvent::TextInput { text } => {
                self.handle_text_input(text)
            }
            _ => UiEventResult::NotHandled,
        }
    }

    fn handle_key_down(&mut self, keycode: UiKeyCode, _character: Option<char>) -> UiEventResult {
        if let Some(state) = &mut self.list_state {
            match keycode {
                UiKeyCode::Up => {
                    if state.move_up() {
                        return UiEventResult::Handled;
                    }
                }
                UiKeyCode::Down => {
                    if state.move_down() {
                        return UiEventResult::Handled;
                    }
                }
                UiKeyCode::PageUp => {
                    if state.page_up() {
                        return UiEventResult::Handled;
                    }
                }
                UiKeyCode::PageDown => {
                    if state.page_down() {
                        return UiEventResult::Handled;
                    }
                }
                UiKeyCode::Home => {
                    state.selected_index = 0;
                    state.adjust_scroll();
                    return UiEventResult::Handled;
                }
                UiKeyCode::End => {
                    state.selected_index = state.max_index;
                    state.adjust_scroll();
                    return UiEventResult::Handled;
                }
                UiKeyCode::Return | UiKeyCode::Space => {
                    return UiEventResult::Selected(state.selected_index);
                }
                UiKeyCode::Escape => {
                    return UiEventResult::Escape;
                }
                _ => {}
            }
        }
        UiEventResult::NotHandled
    }

    fn handle_mouse_motion(&mut self, x: i32, y: i32) -> UiEventResult {
        // Check if mouse is over a list item
        if let Some(state) = &mut self.list_state {
            for item in &self.items {
                if let UiItem::List(list) = item {
                    let rect = list.rect();
                    if rect.contains_point(x, y) {
                        // Calculate which item was hovered
                        let relative_y = y - rect.y;
                        let item_index = state.scroll_offset + (relative_y / list.item_height) as usize;
                        if item_index <= state.max_index && item_index != state.selected_index {
                            state.selected_index = item_index;
                            return UiEventResult::Handled;
                        }
                    }
                }
            }
        }
        UiEventResult::NotHandled
    }

    fn handle_mouse_down(&mut self, button: MouseButton, x: i32, y: i32) -> UiEventResult {
        if button != MouseButton::Left {
            return UiEventResult::NotHandled;
        }

        // Check clickable items
        for item in &self.items {
            let rect = item.rect();
            if !item.is_interactive() || !rect.contains_point(x, y) {
                continue;
            }

            match item {
                UiItem::Button(btn) => {
                    return UiEventResult::Selected(btn.action_id as usize);
                }
                UiItem::ArtTextButton(btn) => {
                    return UiEventResult::Selected(btn.action_id as usize);
                }
                UiItem::List(_) => {
                    if let Some(state) = &self.list_state {
                        return UiEventResult::Selected(state.selected_index);
                    }
                }
                _ => {}
            }
        }

        UiEventResult::NotHandled
    }

    fn handle_mouse_wheel(&mut self, delta: i32) -> UiEventResult {
        if let Some(state) = &mut self.list_state {
            if delta > 0 {
                for _ in 0..delta.abs() {
                    state.move_up();
                }
                return UiEventResult::Handled;
            } else if delta < 0 {
                for _ in 0..delta.abs() {
                    state.move_down();
                }
                return UiEventResult::Handled;
            }
        }
        UiEventResult::NotHandled
    }

    fn handle_text_input(&mut self, text: &str) -> UiEventResult {
        if !self.text_input_active {
            return UiEventResult::NotHandled;
        }

        // Find the edit field and update it
        for item in &mut self.items {
            if let UiItem::Edit(edit) = item {
                for c in text.chars() {
                    edit.insert_char(c);
                }
                return UiEventResult::Handled;
            }
        }

        UiEventResult::NotHandled
    }
}

/// Calculate center offset for centering content
pub fn get_center_offset(content_width: i32, container_width: i32) -> i32 {
    (container_width - content_width) / 2
}

/// Screen dimensions for UI layout
pub const SCREEN_WIDTH: i32 = 640;
pub const SCREEN_HEIGHT: i32 = 480;

/// Get the main UI rectangle (centered on screen)
pub fn get_ui_rectangle() -> UiRect {
    UiRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_list_state() {
        let mut state = UiListState::new(5, 3, true);

        assert_eq!(state.selected_index, 0);
        assert!(state.move_down());
        assert_eq!(state.selected_index, 1);

        // Test wrapping
        state.selected_index = 5;
        assert!(state.move_down()); // Should wrap to 0
        assert_eq!(state.selected_index, 0);

        // Test move up with wrap
        assert!(state.move_up()); // Should wrap to max
        assert_eq!(state.selected_index, 5);
    }

    #[test]
    fn test_ui_list_state_no_wrap() {
        let mut state = UiListState::new(3, 2, false);

        state.selected_index = 0;
        assert!(!state.move_up()); // Should not wrap
        assert_eq!(state.selected_index, 0);

        state.selected_index = 3;
        assert!(!state.move_down()); // Should not wrap
        assert_eq!(state.selected_index, 3);
    }

    #[test]
    fn test_ui_context() {
        let mut ctx = UiContext::new();
        ctx.init_list(Vec::new(), 5, 3, true, 0);

        let event = UiEvent::KeyDown {
            keycode: UiKeyCode::Down,
            character: None,
        };
        assert_eq!(ctx.handle_event(&event), UiEventResult::Handled);

        if let Some(state) = &ctx.list_state {
            assert_eq!(state.selected_index, 1);
        }
    }

    #[test]
    fn test_fade() {
        let mut ctx = UiContext::new();
        ctx.start_fade_in(0);

        assert_eq!(ctx.fade_value, 0);
        assert!(!ctx.is_fade_complete());

        // Simulate time passing
        ctx.fade_value = 256;
        assert!(ctx.is_fade_complete());
        assert_eq!(ctx.fade_alpha(), 1.0);
    }

    #[test]
    fn test_center_offset() {
        assert_eq!(get_center_offset(100, 640), 270);
        assert_eq!(get_center_offset(640, 640), 0);
    }
}
