//! Menu System - Main menu, dialogs, and UI elements
//!
//! # M62: UI System - Menu, Dialogs, HUD
//!
//! Rust port of `Source/DiabloUI/*.cpp` - Game menus and UI elements.
//!
//! ## C++ References
//! - `Source/DiabloUI/diabloui.cpp` (1,214 lines) - Core UI framework
//! - `Source/DiabloUI/mainmenu.cpp` (133 lines) - Main menu
//! - `Source/DiabloUI/dialogs.cpp` - Dialog system
//! - `Source/DiabloUI/button.cpp` - Button widgets
//!
//! ## Components
//! - MainMenu: Game main menu with Single/Multi/Settings options
//! - Dialog: Generic dialog boxes (Yes/No, OK, etc.)
//! - UiItem: Base UI element types
//! - UiList: Scrollable list widget
//! - Button: Clickable button widget

#![allow(dead_code)]

use std::time::{Duration, Instant};

// =============================================================================
// Constants
// =============================================================================

/// UI fade duration in milliseconds
pub const FADE_DURATION_MS: u64 = 500;

/// List double-click timeout in milliseconds
pub const LIST_DOUBLE_CLICK_MS: u64 = 500;

/// Maximum list viewport size
pub const MAX_LIST_VIEWPORT: usize = 10;

/// Screen width for UI calculations
pub const UI_SCREEN_WIDTH: i32 = 640;

/// Screen height for UI calculations
pub const UI_SCREEN_HEIGHT: i32 = 480;

// =============================================================================
// UI Flags
// =============================================================================

/// UI rendering and behavior flags
///
/// **C++ Reference**: `Source/DiabloUI/ui_flags.hpp`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFlags(u32);

impl UiFlags {
    pub const NONE: Self = Self(0);

    // Font sizes
    pub const FONT_SIZE_12: Self = Self(1 << 0);
    pub const FONT_SIZE_24: Self = Self(1 << 1);
    pub const FONT_SIZE_30: Self = Self(1 << 2);
    pub const FONT_SIZE_42: Self = Self(1 << 3);
    pub const FONT_SIZE_46: Self = Self(1 << 4);

    // Alignment
    pub const ALIGN_LEFT: Self = Self(1 << 5);
    pub const ALIGN_CENTER: Self = Self(1 << 6);
    pub const ALIGN_RIGHT: Self = Self(1 << 7);
    pub const VERTICAL_CENTER: Self = Self(1 << 8);

    // Colors
    pub const COLOR_UI_GOLD: Self = Self(1 << 9);
    pub const COLOR_UI_SILVER: Self = Self(1 << 10);
    pub const COLOR_UI_SILVER_DARK: Self = Self(1 << 11);
    pub const COLOR_DIALOG_WHITE: Self = Self(1 << 12);
    pub const COLOR_DIALOG_DARK: Self = Self(1 << 13);
    pub const COLOR_RED: Self = Self(1 << 14);
    pub const COLOR_BLUE: Self = Self(1 << 15);

    // Behavior
    pub const HIGHLIGHTED: Self = Self(1 << 16);
    pub const DISABLED: Self = Self(1 << 17);
    pub const PENDING_FOCUS: Self = Self(1 << 18);
    pub const ELEMENT_HIDDEN: Self = Self(1 << 19);
    pub const KNOWN_ART_SIZE: Self = Self(1 << 20);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn combine(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for UiFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for UiFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

// =============================================================================
// Geometry Types
// =============================================================================

/// Point in screen coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// Rectangle in screen coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.w
            && point.y >= self.y
            && point.y < self.y + self.h
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

// =============================================================================
// Main Menu
// =============================================================================

/// Main menu selection options
///
/// **C++ Reference**: `Source/DiabloUI/diabloui.h` - `_mainmenu_selections`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MainMenuSelection {
    /// No selection (placeholder)
    None = 255,
    /// Single player game
    SinglePlayer = 0,
    /// Multi player game
    MultiPlayer = 1,
    /// Settings menu
    Settings = 2,
    /// Show support information
    ShowSupport = 3,
    /// Show credits
    ShowCredits = 4,
    /// Exit game
    ExitDiablo = 5,
    /// Attract mode timeout
    AttractMode = 6,
}

/// Alias for compatibility
pub const Multiplayer: MainMenuSelection = MainMenuSelection::MultiPlayer;

impl MainMenuSelection {
    /// Convert index to selection
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::SinglePlayer),
            1 => Some(Self::MultiPlayer),
            2 => Some(Self::ShowSupport),
            3 => Some(Self::Settings),
            4 => Some(Self::ShowCredits),
            5 => Some(Self::ExitDiablo),
            _ => None,
        }
    }
}

/// Main menu state
///
/// **C++ Reference**: `Source/DiabloUI/mainmenu.cpp`
pub struct MainMenu {
    /// Current menu items
    items: Vec<MenuItem>,
    /// Currently selected item index
    selected_index: usize,
    /// Menu result after selection
    result: Option<MainMenuSelection>,
    /// Whether this is Hellfire mode
    is_hellfire: bool,
    /// Whether this is spawn/shareware mode
    is_spawn: bool,
    /// Attract mode timeout in seconds
    attract_timeout: u32,
    /// Last activity time for attract mode
    last_activity: Instant,
    /// UI rectangle position
    ui_position: Point,
    /// Fade state
    fade_state: FadeState,
    /// Player name for display
    player_name: String,
}

impl MainMenu {
    pub fn new(is_hellfire: bool, is_spawn: bool) -> Self {
        let mut menu = Self {
            items: Vec::new(),
            selected_index: 0,
            result: None,
            is_hellfire,
            is_spawn,
            attract_timeout: 30,
            last_activity: Instant::now(),
            ui_position: Point::new(0, 0),
            fade_state: FadeState::new(),
            player_name: String::new(),
        };
        menu.load_items();
        menu
    }
    
    /// Create from player name (compatibility constructor for main.rs)
    pub fn from_name(name: &str) -> Self {
        let mut menu = Self::new(false, false);
        menu.player_name = name.to_string();
        menu
    }
    
    /// Get player name
    pub fn player_name(&self) -> &str {
        &self.player_name
    }
    
    /// Set selection by index
    pub fn set_selection(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = index;
        }
    }
    
    /// Move selection by delta (for keyboard navigation)
    pub fn move_selection(&mut self, delta: i32) {
        let len = self.items.len() as i32;
        if len == 0 {
            return;
        }
        let new_index = (self.selected_index as i32 + delta).rem_euclid(len);
        self.selected_index = new_index as usize;
    }
    
    /// Get current selection
    pub fn get_selected(&self) -> MainMenuSelection {
        MainMenuSelection::from_index(self.selected_index)
            .unwrap_or(MainMenuSelection::None)
    }

    /// Load menu items
    ///
    /// **C++ Reference**: `MainmenuLoad()` in mainmenu.cpp:50-91
    fn load_items(&mut self) {
        self.items.clear();

        self.items.push(MenuItem::new(
            "Single Player",
            MainMenuSelection::SinglePlayer as i32,
        ));
        self.items.push(MenuItem::new(
            "Multi Player",
            MainMenuSelection::MultiPlayer as i32,
        ));
        self.items.push(MenuItem::new(
            "Settings",
            MainMenuSelection::Settings as i32,
        ));
        self.items.push(MenuItem::new(
            "Support",
            MainMenuSelection::ShowSupport as i32,
        ));
        self.items.push(MenuItem::new(
            "Show Credits",
            MainMenuSelection::ShowCredits as i32,
        ));

        let exit_text = if self.is_hellfire {
            "Exit Hellfire"
        } else {
            "Exit Diablo"
        };
        self.items.push(MenuItem::new(
            exit_text,
            MainMenuSelection::ExitDiablo as i32,
        ));
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        self.last_activity = Instant::now();

        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.selected_index = self.items.len() - 1;
                }
                true
            }
            KeyCode::Down => {
                if self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                } else {
                    self.selected_index = 0;
                }
                true
            }
            KeyCode::Return | KeyCode::Space => {
                self.select_current();
                true
            }
            KeyCode::Escape => {
                // Go to exit option
                self.selected_index = self.items.len() - 1;
                true
            }
            _ => false,
        }
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, pos: Point) -> bool {
        self.last_activity = Instant::now();

        for (index, item) in self.items.iter().enumerate() {
            if item.bounds.contains(pos) {
                self.selected_index = index;
                self.select_current();
                return true;
            }
        }
        false
    }

    /// Handle mouse movement
    pub fn handle_mouse_move(&mut self, pos: Point) {
        self.last_activity = Instant::now();

        for (index, item) in self.items.iter().enumerate() {
            if item.bounds.contains(pos) {
                self.selected_index = index;
                break;
            }
        }
    }

    /// Select current item
    fn select_current(&mut self) {
        if let Some(item) = self.items.get(self.selected_index) {
            self.result = match item.value {
                0 => Some(MainMenuSelection::SinglePlayer),
                1 => Some(MainMenuSelection::MultiPlayer),
                2 => Some(MainMenuSelection::Settings),
                3 => Some(MainMenuSelection::ShowSupport),
                4 => Some(MainMenuSelection::ShowCredits),
                5 => Some(MainMenuSelection::ExitDiablo),
                _ => None,
            };
        }
    }

    /// Update menu state
    pub fn update(&mut self) {
        // Update fade
        self.fade_state.update();

        // Check attract mode timeout
        if self.last_activity.elapsed() > Duration::from_secs(self.attract_timeout as u64) {
            self.result = Some(MainMenuSelection::AttractMode);
        }
    }

    /// Get menu result (None if no selection yet)
    pub fn get_result(&self) -> Option<MainMenuSelection> {
        self.result
    }

    /// Get selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get menu items
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Start fade in effect
    pub fn start_fade_in(&mut self) {
        self.fade_state.start_fade_in();
    }

    /// Get current fade value (0-255)
    pub fn fade_value(&self) -> u8 {
        self.fade_state.value()
    }

    /// Set UI position
    pub fn set_position(&mut self, pos: Point) {
        self.ui_position = pos;
        self.update_item_bounds();
    }

    /// Update item bounds based on UI position
    fn update_item_bounds(&mut self) {
        let start_y = self.ui_position.y + 192;
        let item_height = 43;

        for (index, item) in self.items.iter_mut().enumerate() {
            item.bounds = Rect::new(
                self.ui_position.x + 64,
                start_y + (index as i32 * item_height),
                510,
                item_height,
            );
        }
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new(false, false)
    }
}

/// Menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Display text
    pub text: String,
    /// Value associated with item
    pub value: i32,
    /// Bounding rectangle for hit testing
    pub bounds: Rect,
    /// UI flags
    pub flags: UiFlags,
}

impl MenuItem {
    pub fn new(text: &str, value: i32) -> Self {
        Self {
            text: text.to_string(),
            value,
            bounds: Rect::default(),
            flags: UiFlags::FONT_SIZE_42 | UiFlags::COLOR_UI_GOLD | UiFlags::ALIGN_CENTER,
        }
    }
}

// =============================================================================
// Dialog System
// =============================================================================

/// Dialog type
///
/// **C++ Reference**: `Source/DiabloUI/dialogs.h`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// OK button only
    Ok,
    /// Yes/No buttons
    YesNo,
    /// Cancel button only
    Cancel,
    /// Custom buttons
    Custom,
}

/// Dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    /// No result yet
    None,
    /// OK/Yes selected
    Ok,
    /// Cancel/No selected
    Cancel,
    /// Button 1 selected
    Button1,
    /// Button 2 selected
    Button2,
    /// Button 3 selected
    Button3,
}

/// Dialog box
pub struct Dialog {
    /// Dialog title
    pub title: String,
    /// Dialog message
    pub message: String,
    /// Dialog type
    pub dialog_type: DialogType,
    /// Buttons
    pub buttons: Vec<Button>,
    /// Selected button index
    pub selected_button: usize,
    /// Dialog result
    pub result: DialogResult,
    /// Dialog bounds
    pub bounds: Rect,
    /// Is dialog visible
    pub visible: bool,
}

impl Dialog {
    /// Create OK dialog
    pub fn ok(title: &str, message: &str) -> Self {
        let mut dialog = Self {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: DialogType::Ok,
            buttons: vec![Button::new("OK", 0)],
            selected_button: 0,
            result: DialogResult::None,
            bounds: Rect::new(160, 140, 320, 200),
            visible: true,
        };
        dialog.update_button_bounds();
        dialog
    }

    /// Create Yes/No dialog
    pub fn yes_no(title: &str, message: &str) -> Self {
        let mut dialog = Self {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: DialogType::YesNo,
            buttons: vec![
                Button::new("Yes", 0),
                Button::new("No", 1),
            ],
            selected_button: 0,
            result: DialogResult::None,
            bounds: Rect::new(160, 140, 320, 200),
            visible: true,
        };
        dialog.update_button_bounds();
        dialog
    }

    /// Update button positions
    fn update_button_bounds(&mut self) {
        let button_y = self.bounds.y + self.bounds.h - 50;
        let button_width = 80;
        let button_height = 30;
        let total_width = button_width * self.buttons.len() as i32 + 20 * (self.buttons.len() as i32 - 1);
        let start_x = self.bounds.x + (self.bounds.w - total_width) / 2;

        for (index, button) in self.buttons.iter_mut().enumerate() {
            button.bounds = Rect::new(
                start_x + (button_width + 20) * index as i32,
                button_y,
                button_width,
                button_height,
            );
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Left | KeyCode::Tab => {
                if self.selected_button > 0 {
                    self.selected_button -= 1;
                } else {
                    self.selected_button = self.buttons.len() - 1;
                }
                true
            }
            KeyCode::Right => {
                if self.selected_button < self.buttons.len() - 1 {
                    self.selected_button += 1;
                } else {
                    self.selected_button = 0;
                }
                true
            }
            KeyCode::Return | KeyCode::Space => {
                self.select_current();
                true
            }
            KeyCode::Escape => {
                self.result = DialogResult::Cancel;
                self.visible = false;
                true
            }
            KeyCode::Y => {
                if self.dialog_type == DialogType::YesNo {
                    self.result = DialogResult::Ok;
                    self.visible = false;
                    return true;
                }
                false
            }
            KeyCode::N => {
                if self.dialog_type == DialogType::YesNo {
                    self.result = DialogResult::Cancel;
                    self.visible = false;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, pos: Point) -> bool {
        for (index, button) in self.buttons.iter().enumerate() {
            if button.bounds.contains(pos) {
                self.selected_button = index;
                self.select_current();
                return true;
            }
        }
        false
    }

    /// Select current button
    fn select_current(&mut self) {
        match self.selected_button {
            0 => self.result = DialogResult::Ok,
            1 => self.result = DialogResult::Cancel,
            2 => self.result = DialogResult::Button1,
            3 => self.result = DialogResult::Button2,
            4 => self.result = DialogResult::Button3,
            _ => {}
        }
        self.visible = false;
    }

    /// Get dialog result
    pub fn get_result(&self) -> DialogResult {
        self.result
    }

    /// Is dialog still visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

// =============================================================================
// Button Widget
// =============================================================================

/// Button widget
#[derive(Debug, Clone)]
pub struct Button {
    /// Button text
    pub text: String,
    /// Button value
    pub value: i32,
    /// Button bounds
    pub bounds: Rect,
    /// Is button pressed
    pub pressed: bool,
    /// Is button hovered
    pub hovered: bool,
    /// Is button enabled
    pub enabled: bool,
    /// UI flags
    pub flags: UiFlags,
}

impl Button {
    pub fn new(text: &str, value: i32) -> Self {
        Self {
            text: text.to_string(),
            value,
            bounds: Rect::default(),
            pressed: false,
            hovered: false,
            enabled: true,
            flags: UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD | UiFlags::ALIGN_CENTER,
        }
    }

    pub fn with_bounds(text: &str, value: i32, bounds: Rect) -> Self {
        let mut button = Self::new(text, value);
        button.bounds = bounds;
        button
    }

    /// Handle mouse down
    pub fn handle_mouse_down(&mut self, pos: Point) -> bool {
        if self.enabled && self.bounds.contains(pos) {
            self.pressed = true;
            true
        } else {
            false
        }
    }

    /// Handle mouse up
    pub fn handle_mouse_up(&mut self, pos: Point) -> bool {
        let was_pressed = self.pressed;
        self.pressed = false;
        was_pressed && self.bounds.contains(pos)
    }

    /// Handle mouse move
    pub fn handle_mouse_move(&mut self, pos: Point) {
        self.hovered = self.bounds.contains(pos);
    }
}

// =============================================================================
// Scrollable List
// =============================================================================

/// List item
#[derive(Debug, Clone)]
pub struct ListItem {
    /// Item text
    pub text: String,
    /// Item value
    pub value: i32,
}

impl ListItem {
    pub fn new(text: &str, value: i32) -> Self {
        Self {
            text: text.to_string(),
            value,
        }
    }
}

/// Scrollable list widget
pub struct UiList {
    /// List items
    pub items: Vec<ListItem>,
    /// Selected item index
    pub selected_index: usize,
    /// Viewport offset
    pub viewport_offset: usize,
    /// Viewport size (number of visible items)
    pub viewport_size: usize,
    /// List bounds
    pub bounds: Rect,
    /// Item height
    pub item_height: i32,
    /// UI flags for items
    pub flags: UiFlags,
    /// Last click index (for double-click detection)
    last_click_index: usize,
    /// Last click time
    last_click_time: Instant,
}

impl UiList {
    pub fn new(items: Vec<ListItem>, viewport_size: usize) -> Self {
        Self {
            items,
            selected_index: 0,
            viewport_offset: 0,
            viewport_size,
            bounds: Rect::default(),
            item_height: 43,
            flags: UiFlags::FONT_SIZE_42 | UiFlags::COLOR_UI_GOLD,
            last_click_index: usize::MAX,
            last_click_time: Instant::now(),
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode, wraps: bool) -> bool {
        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else if wraps {
                    self.selected_index = self.items.len().saturating_sub(1);
                }
                self.adjust_viewport();
                true
            }
            KeyCode::Down => {
                if self.selected_index < self.items.len().saturating_sub(1) {
                    self.selected_index += 1;
                } else if wraps {
                    self.selected_index = 0;
                }
                self.adjust_viewport();
                true
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(self.viewport_size);
                self.adjust_viewport();
                true
            }
            KeyCode::PageDown => {
                self.selected_index = (self.selected_index + self.viewport_size)
                    .min(self.items.len().saturating_sub(1));
                self.adjust_viewport();
                true
            }
            KeyCode::Home => {
                self.selected_index = 0;
                self.viewport_offset = 0;
                true
            }
            KeyCode::End => {
                self.selected_index = self.items.len().saturating_sub(1);
                self.adjust_viewport();
                true
            }
            _ => false,
        }
    }

    /// Handle mouse click
    ///
    /// Returns true if double-click detected on an item
    pub fn handle_click(&mut self, pos: Point) -> bool {
        if !self.bounds.contains(pos) {
            return false;
        }

        let relative_y = pos.y - self.bounds.y;
        let item_index = self.viewport_offset + (relative_y / self.item_height) as usize;

        if item_index >= self.items.len() {
            return false;
        }

        // Check for double-click
        let is_double_click = item_index == self.last_click_index
            && self.last_click_time.elapsed() < Duration::from_millis(LIST_DOUBLE_CLICK_MS);

        self.selected_index = item_index;
        self.last_click_index = item_index;
        self.last_click_time = Instant::now();

        is_double_click
    }

    /// Handle scroll wheel
    pub fn handle_scroll(&mut self, delta: i32) {
        if delta < 0 {
            self.viewport_offset = self.viewport_offset.saturating_add(1);
            let max_offset = self.items.len().saturating_sub(self.viewport_size);
            if self.viewport_offset > max_offset {
                self.viewport_offset = max_offset;
            }
        } else if delta > 0 {
            self.viewport_offset = self.viewport_offset.saturating_sub(1);
        }
    }

    /// Adjust viewport to ensure selected item is visible
    fn adjust_viewport(&mut self) {
        if self.selected_index < self.viewport_offset {
            self.viewport_offset = self.selected_index;
        } else if self.selected_index >= self.viewport_offset + self.viewport_size {
            self.viewport_offset = self.selected_index - self.viewport_size + 1;
        }
    }

    /// Get visible items
    pub fn visible_items(&self) -> impl Iterator<Item = (usize, &ListItem)> {
        self.items
            .iter()
            .enumerate()
            .skip(self.viewport_offset)
            .take(self.viewport_size)
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&ListItem> {
        self.items.get(self.selected_index)
    }

    /// Set bounds
    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.viewport_size = (bounds.h / self.item_height) as usize;
    }

    /// Scroll bar thumb position (0.0 to 1.0)
    pub fn scroll_position(&self) -> f32 {
        if self.items.len() <= self.viewport_size {
            return 0.0;
        }
        let max_offset = self.items.len() - self.viewport_size;
        self.viewport_offset as f32 / max_offset as f32
    }

    /// Scroll bar thumb size (0.0 to 1.0)
    pub fn scroll_thumb_size(&self) -> f32 {
        if self.items.is_empty() {
            return 1.0;
        }
        (self.viewport_size as f32 / self.items.len() as f32).min(1.0)
    }
}

// =============================================================================
// Text Input
// =============================================================================

/// Text input state
pub struct TextInput {
    /// Current text
    pub text: String,
    /// Maximum length
    pub max_length: usize,
    /// Cursor position
    pub cursor_pos: usize,
    /// Is input focused
    pub focused: bool,
    /// Input bounds
    pub bounds: Rect,
    /// Allow empty input
    pub allow_empty: bool,
}

impl TextInput {
    pub fn new(max_length: usize) -> Self {
        Self {
            text: String::new(),
            max_length,
            cursor_pos: 0,
            focused: false,
            bounds: Rect::default(),
            allow_empty: false,
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) -> bool {
        if !self.focused || !c.is_ascii() || c.is_ascii_control() {
            return false;
        }

        if self.text.len() < self.max_length {
            self.text.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
            true
        } else {
            false
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        if !self.focused {
            return false;
        }

        match key {
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.text.remove(self.cursor_pos);
                    true
                } else {
                    false
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.text.len() {
                    self.text.remove(self.cursor_pos);
                    true
                } else {
                    false
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.text.len() {
                    self.cursor_pos += 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.text.len();
                true
            }
            _ => false,
        }
    }

    /// Set focus
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        if focused {
            self.cursor_pos = self.text.len();
        }
    }

    /// Get current text
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Set text
    pub fn set_text(&mut self, text: &str) {
        self.text = text.chars().take(self.max_length).collect();
        self.cursor_pos = self.text.len();
    }

    /// Is input valid (not empty or allow_empty is true)
    pub fn is_valid(&self) -> bool {
        self.allow_empty || !self.text.is_empty()
    }
}

// =============================================================================
// Fade Effect
// =============================================================================

/// Fade state for UI transitions
pub struct FadeState {
    /// Current fade value (0-255)
    value: u8,
    /// Target value
    target: u8,
    /// Fade start time
    start_time: Option<Instant>,
    /// Fade duration
    duration: Duration,
}

impl FadeState {
    pub fn new() -> Self {
        Self {
            value: 255,
            target: 255,
            start_time: None,
            duration: Duration::from_millis(FADE_DURATION_MS),
        }
    }

    /// Start fade in (0 -> 255)
    pub fn start_fade_in(&mut self) {
        self.value = 0;
        self.target = 255;
        self.start_time = Some(Instant::now());
    }

    /// Start fade out (255 -> 0)
    pub fn start_fade_out(&mut self) {
        self.value = 255;
        self.target = 0;
        self.start_time = Some(Instant::now());
    }

    /// Update fade state
    pub fn update(&mut self) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            if elapsed >= self.duration {
                self.value = self.target;
                self.start_time = None;
            } else {
                let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
                if self.target > self.value {
                    self.value = (255.0 * progress) as u8;
                } else {
                    self.value = (255.0 * (1.0 - progress)) as u8;
                }
            }
        }
    }

    /// Get current fade value (0-255)
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Is fade complete
    pub fn is_complete(&self) -> bool {
        self.start_time.is_none()
    }
}

impl Default for FadeState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Key Codes
// =============================================================================

/// Key codes for input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Unknown,
    Return,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,
}

impl KeyCode {
    /// Convert from SDL scancode
    pub fn from_sdl_scancode(scancode: u32) -> Self {
        match scancode {
            13 => Self::Return,
            27 => Self::Escape,
            32 => Self::Space,
            9 => Self::Tab,
            8 => Self::Backspace,
            127 => Self::Delete,
            273 => Self::Up,
            274 => Self::Down,
            276 => Self::Left,
            275 => Self::Right,
            278 => Self::Home,
            279 => Self::End,
            280 => Self::PageUp,
            281 => Self::PageDown,
            _ => Self::Unknown,
        }
    }
}

// =============================================================================
// UI Manager
// =============================================================================

/// Main UI manager
pub struct Menu {
    /// Current main menu
    pub main_menu: Option<MainMenu>,
    /// Current dialog
    pub dialog: Option<Dialog>,
    /// Is UI active
    pub active: bool,
    /// UI position offset
    pub position: Point,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            main_menu: None,
            dialog: None,
            active: false,
            position: Point::new(0, 0),
        }
    }

    /// Show main menu
    pub fn show_main_menu(&mut self, is_hellfire: bool, is_spawn: bool) {
        let mut menu = MainMenu::new(is_hellfire, is_spawn);
        menu.set_position(self.position);
        menu.start_fade_in();
        self.main_menu = Some(menu);
        self.active = true;
    }

    /// Hide main menu
    pub fn hide_main_menu(&mut self) {
        self.main_menu = None;
        if self.dialog.is_none() {
            self.active = false;
        }
    }

    /// Show OK dialog
    pub fn show_ok_dialog(&mut self, title: &str, message: &str) {
        self.dialog = Some(Dialog::ok(title, message));
        self.active = true;
    }

    /// Show Yes/No dialog
    pub fn show_yes_no_dialog(&mut self, title: &str, message: &str) {
        self.dialog = Some(Dialog::yes_no(title, message));
        self.active = true;
    }

    /// Hide dialog
    pub fn hide_dialog(&mut self) {
        self.dialog = None;
        if self.main_menu.is_none() {
            self.active = false;
        }
    }

    /// Update UI state
    pub fn update(&mut self) {
        if let Some(menu) = &mut self.main_menu {
            menu.update();
        }

        if let Some(dialog) = &self.dialog {
            if !dialog.is_visible() {
                self.dialog = None;
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        // Dialog takes priority
        if let Some(dialog) = &mut self.dialog {
            return dialog.handle_key(key);
        }

        if let Some(menu) = &mut self.main_menu {
            return menu.handle_key(key);
        }

        false
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, pos: Point) -> bool {
        // Dialog takes priority
        if let Some(dialog) = &mut self.dialog {
            return dialog.handle_click(pos);
        }

        if let Some(menu) = &mut self.main_menu {
            return menu.handle_click(pos);
        }

        false
    }

    /// Handle mouse movement
    pub fn handle_mouse_move(&mut self, pos: Point) {
        if let Some(menu) = &mut self.main_menu {
            menu.handle_mouse_move(pos);
        }
    }

    /// Render UI
    pub fn render(&self) {
        // Render main menu
        if let Some(_menu) = &self.main_menu {
            // TODO: Actual rendering via engine
        }

        // Render dialog on top
        if let Some(_dialog) = &self.dialog {
            // TODO: Actual rendering via engine
        }
    }

    /// Get main menu result
    pub fn get_main_menu_result(&self) -> Option<MainMenuSelection> {
        self.main_menu.as_ref().and_then(|m| m.get_result())
    }

    /// Get dialog result
    pub fn get_dialog_result(&self) -> Option<DialogResult> {
        self.dialog.as_ref().map(|d| d.get_result())
    }

    /// Set UI position
    pub fn set_position(&mut self, pos: Point) {
        self.position = pos;
        if let Some(menu) = &mut self.main_menu {
            menu.set_position(pos);
        }
    }

    /// Is UI active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Progress Bar
// =============================================================================

/// Progress bar for loading screens
pub struct ProgressBar {
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Progress bar bounds
    pub bounds: Rect,
    /// Message text
    pub message: String,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            bounds: Rect::new(160, 350, 320, 30),
            message: String::new(),
        }
    }

    /// Set progress (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Set message
    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    /// Get progress percentage (0-100)
    pub fn percentage(&self) -> u32 {
        (self.progress * 100.0) as u32
    }
}

impl Default for ProgressBar {
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
    fn test_ui_flags() {
        let flags = UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD;
        assert!(flags.contains(UiFlags::FONT_SIZE_24));
        assert!(flags.contains(UiFlags::COLOR_UI_GOLD));
        assert!(!flags.contains(UiFlags::FONT_SIZE_42));
    }

    #[test]
    fn test_point() {
        let p = Point::new(10, 20);
        let p2 = p.offset(5, -5);
        assert_eq!(p2.x, 15);
        assert_eq!(p2.y, 15);
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(10, 10, 100, 100);
        assert!(r.contains(Point::new(50, 50)));
        assert!(r.contains(Point::new(10, 10)));
        assert!(!r.contains(Point::new(110, 110)));
        assert!(!r.contains(Point::new(5, 50)));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0, 0, 100, 100);
        let r2 = Rect::new(50, 50, 100, 100);
        let r3 = Rect::new(200, 200, 50, 50);

        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_main_menu_creation() {
        let menu = MainMenu::new(false, false);
        assert_eq!(menu.items().len(), 6);
        assert_eq!(menu.selected_index(), 0);
        assert!(menu.get_result().is_none());
    }

    #[test]
    fn test_main_menu_hellfire() {
        let menu = MainMenu::new(true, false);
        let exit_item = menu.items().last().unwrap();
        assert!(exit_item.text.contains("Hellfire"));
    }

    #[test]
    fn test_main_menu_navigation() {
        let mut menu = MainMenu::new(false, false);

        menu.handle_key(KeyCode::Down);
        assert_eq!(menu.selected_index(), 1);

        menu.handle_key(KeyCode::Down);
        assert_eq!(menu.selected_index(), 2);

        menu.handle_key(KeyCode::Up);
        assert_eq!(menu.selected_index(), 1);
    }

    #[test]
    fn test_main_menu_wrap() {
        let mut menu = MainMenu::new(false, false);

        // Go up from first item - should wrap to last
        menu.handle_key(KeyCode::Up);
        assert_eq!(menu.selected_index(), 5);

        // Go down from last item - should wrap to first
        menu.handle_key(KeyCode::Down);
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn test_dialog_ok() {
        let dialog = Dialog::ok("Test", "Test message");
        assert_eq!(dialog.buttons.len(), 1);
        assert_eq!(dialog.buttons[0].text, "OK");
        assert!(dialog.is_visible());
    }

    #[test]
    fn test_dialog_yes_no() {
        let dialog = Dialog::yes_no("Confirm", "Are you sure?");
        assert_eq!(dialog.buttons.len(), 2);
        assert_eq!(dialog.buttons[0].text, "Yes");
        assert_eq!(dialog.buttons[1].text, "No");
    }

    #[test]
    fn test_dialog_keyboard() {
        let mut dialog = Dialog::yes_no("Test", "Test");

        // Navigate between buttons
        dialog.handle_key(KeyCode::Right);
        assert_eq!(dialog.selected_button, 1);

        dialog.handle_key(KeyCode::Left);
        assert_eq!(dialog.selected_button, 0);

        // Confirm
        dialog.handle_key(KeyCode::Return);
        assert_eq!(dialog.get_result(), DialogResult::Ok);
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_dialog_shortcut_keys() {
        let mut dialog = Dialog::yes_no("Test", "Test");

        dialog.handle_key(KeyCode::Y);
        assert_eq!(dialog.get_result(), DialogResult::Ok);

        let mut dialog2 = Dialog::yes_no("Test", "Test");
        dialog2.handle_key(KeyCode::N);
        assert_eq!(dialog2.get_result(), DialogResult::Cancel);
    }

    #[test]
    fn test_button() {
        let mut button = Button::with_bounds("Test", 1, Rect::new(0, 0, 100, 30));
        assert!(!button.pressed);

        button.handle_mouse_down(Point::new(50, 15));
        assert!(button.pressed);

        let clicked = button.handle_mouse_up(Point::new(50, 15));
        assert!(clicked);
        assert!(!button.pressed);
    }

    #[test]
    fn test_button_outside_click() {
        let mut button = Button::with_bounds("Test", 1, Rect::new(0, 0, 100, 30));

        // Click outside
        button.handle_mouse_down(Point::new(200, 200));
        assert!(!button.pressed);
    }

    #[test]
    fn test_list() {
        let items = vec![
            ListItem::new("Item 1", 0),
            ListItem::new("Item 2", 1),
            ListItem::new("Item 3", 2),
        ];
        let mut list = UiList::new(items, 10);

        assert_eq!(list.selected_index, 0);

        list.handle_key(KeyCode::Down, true);
        assert_eq!(list.selected_index, 1);

        list.handle_key(KeyCode::End, true);
        assert_eq!(list.selected_index, 2);

        list.handle_key(KeyCode::Home, true);
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_list_viewport() {
        let items: Vec<_> = (0..20)
            .map(|i| ListItem::new(&format!("Item {}", i), i))
            .collect();
        let mut list = UiList::new(items, 5);

        // Move to item beyond viewport
        for _ in 0..10 {
            list.handle_key(KeyCode::Down, false);
        }

        assert_eq!(list.selected_index, 10);
        assert!(list.viewport_offset > 0);
    }

    #[test]
    fn test_text_input() {
        let mut input = TextInput::new(20);
        input.set_focus(true);

        input.handle_char('H');
        input.handle_char('e');
        input.handle_char('l');
        input.handle_char('l');
        input.handle_char('o');

        assert_eq!(input.get_text(), "Hello");
        assert_eq!(input.cursor_pos, 5);
    }

    #[test]
    fn test_text_input_backspace() {
        let mut input = TextInput::new(20);
        input.set_focus(true);
        input.set_text("Hello");

        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.get_text(), "Hell");
    }

    #[test]
    fn test_text_input_navigation() {
        let mut input = TextInput::new(20);
        input.set_focus(true);
        input.set_text("Hello");

        input.handle_key(KeyCode::Home);
        assert_eq!(input.cursor_pos, 0);

        input.handle_key(KeyCode::End);
        assert_eq!(input.cursor_pos, 5);

        input.handle_key(KeyCode::Left);
        assert_eq!(input.cursor_pos, 4);
    }

    #[test]
    fn test_text_input_max_length() {
        let mut input = TextInput::new(5);
        input.set_focus(true);

        for _ in 0..10 {
            input.handle_char('a');
        }

        assert_eq!(input.get_text().len(), 5);
    }

    #[test]
    fn test_fade_state() {
        let mut fade = FadeState::new();
        assert_eq!(fade.value(), 255);

        fade.start_fade_in();
        assert_eq!(fade.value(), 0);
        assert!(!fade.is_complete());
    }

    #[test]
    fn test_menu_manager() {
        let mut menu = Menu::new();
        assert!(!menu.is_active());

        menu.show_main_menu(false, false);
        assert!(menu.is_active());
        assert!(menu.main_menu.is_some());

        menu.hide_main_menu();
        assert!(!menu.is_active());
        assert!(menu.main_menu.is_none());
    }

    #[test]
    fn test_menu_dialog() {
        let mut menu = Menu::new();

        menu.show_ok_dialog("Title", "Message");
        assert!(menu.is_active());
        assert!(menu.dialog.is_some());
    }

    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new();
        assert_eq!(bar.progress, 0.0);
        assert_eq!(bar.percentage(), 0);

        bar.set_progress(0.5);
        assert_eq!(bar.percentage(), 50);

        bar.set_progress(1.5); // Should clamp
        assert_eq!(bar.progress, 1.0);
    }
}
