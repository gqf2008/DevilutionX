//! Main Panel - Bottom Game HUD (M64)
//!
//! Provides the main game panel at the bottom of the screen including:
//! - Health and Mana orbs
//! - Action buttons (Character, Quests, Map, Menu, Inventory, Spells)
//! - Belt item slots
//! - Experience bar
//!
//! ## C++ References
//! - Source/panels/mainpanel.cpp
//! - Source/panels/mainpanel.hpp
//! - Source/control.cpp (panel rendering)

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

/// Panel button indices - matches C++ MainPanelButton enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MainPanelButton {
    /// Character panel button
    Character = 0,
    /// Quests panel button
    Quests = 1,
    /// Automap button
    Map = 2,
    /// Game menu button
    Menu = 3,
    /// Inventory panel button
    Inventory = 4,
    /// Spell book button
    Spells = 5,
}

impl MainPanelButton {
    /// Get the button label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Character => "char",
            Self::Quests => "quests",
            Self::Map => "map",
            Self::Menu => "menu",
            Self::Inventory => "inv",
            Self::Spells => "spells",
        }
    }

    /// Get button index
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// From index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Character),
            1 => Some(Self::Quests),
            2 => Some(Self::Map),
            3 => Some(Self::Menu),
            4 => Some(Self::Inventory),
            5 => Some(Self::Spells),
            _ => None,
        }
    }

    /// Total number of buttons
    pub const COUNT: usize = 6;
}

/// Panel button state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Normal state
    Normal,
    /// Hovered by mouse
    Hovered,
    /// Being pressed
    Pressed,
    /// Disabled
    Disabled,
}

/// A point in 2D space
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Point = Point { x: 0, y: 0 };
}

/// A 2D size
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

/// A rectangle
#[derive(Debug, Clone, Copy, Default)]
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

    pub fn right(&self) -> i32 {
        self.position.x + self.size.width
    }

    pub fn bottom(&self) -> i32 {
        self.position.y + self.size.height
    }
}

/// Main panel button rectangles - matches C++ MainPanelButtonRect array
/// These are relative to the main panel position
pub const MAIN_PANEL_BUTTON_RECTS: [Rectangle; MainPanelButton::COUNT] = [
    Rectangle::new(9, 9, 71, 19),    // Character
    Rectangle::new(9, 35, 71, 19),   // Quests
    Rectangle::new(9, 61, 71, 19),   // Map
    Rectangle::new(560, 9, 71, 19),  // Menu
    Rectangle::new(560, 35, 71, 19), // Inventory
    Rectangle::new(560, 61, 71, 19), // Spells
];

/// Main panel dimensions
pub const MAIN_PANEL_WIDTH: i32 = 640;
pub const MAIN_PANEL_HEIGHT: i32 = 128;

/// Orb dimensions
pub const ORB_WIDTH: i32 = 88;
pub const ORB_HEIGHT: i32 = 88;

/// Health orb position (relative to panel)
pub const HEALTH_ORB_POSITION: Point = Point::new(96, 16);

/// Mana orb position (relative to panel)
pub const MANA_ORB_POSITION: Point = Point::new(456, 16);

/// Belt slot dimensions
pub const BELT_SLOT_WIDTH: i32 = 29;
pub const BELT_SLOT_HEIGHT: i32 = 29;
pub const BELT_SLOT_COUNT: usize = 8;

/// Belt position (relative to panel)
pub const BELT_POSITION: Point = Point::new(205, 21);

/// Flask button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlaskType {
    /// Health flask (red)
    Health,
    /// Mana flask (blue)
    Mana,
}

/// Main panel button state tracking
#[derive(Debug, Clone)]
pub struct PanelButtonState {
    /// Button type
    pub button: MainPanelButton,
    /// Current state
    pub state: ButtonState,
    /// Is the associated panel open?
    pub panel_open: bool,
}

impl PanelButtonState {
    pub fn new(button: MainPanelButton) -> Self {
        Self {
            button,
            state: ButtonState::Normal,
            panel_open: false,
        }
    }
}

/// Orb fill level (0-100)
#[derive(Debug, Clone, Copy)]
pub struct OrbLevel {
    /// Current value
    pub current: i32,
    /// Maximum value
    pub max: i32,
}

impl OrbLevel {
    pub fn new(current: i32, max: i32) -> Self {
        Self { current, max }
    }

    /// Get fill percentage (0.0 - 1.0)
    pub fn percentage(&self) -> f32 {
        if self.max <= 0 {
            return 0.0;
        }
        (self.current as f32 / self.max as f32).clamp(0.0, 1.0)
    }

    /// Get fill height in pixels
    pub fn fill_height(&self) -> i32 {
        (self.percentage() * ORB_HEIGHT as f32) as i32
    }
}

/// Experience bar state
#[derive(Debug, Clone, Copy)]
pub struct ExperienceBar {
    /// Current experience
    pub current_exp: u32,
    /// Experience needed for current level
    pub level_start_exp: u32,
    /// Experience needed for next level
    pub next_level_exp: u32,
}

impl ExperienceBar {
    pub fn new(current: u32, level_start: u32, next_level: u32) -> Self {
        Self {
            current_exp: current,
            level_start_exp: level_start,
            next_level_exp: next_level,
        }
    }

    /// Get progress percentage (0.0 - 1.0)
    pub fn percentage(&self) -> f32 {
        if self.next_level_exp <= self.level_start_exp {
            return 1.0;
        }
        let progress = self.current_exp.saturating_sub(self.level_start_exp);
        let needed = self.next_level_exp - self.level_start_exp;
        (progress as f32 / needed as f32).clamp(0.0, 1.0)
    }
}

/// Belt item slot
#[derive(Debug, Clone, Copy, Default)]
pub struct BeltSlot {
    /// Whether this slot has an item
    pub has_item: bool,
    /// Item frame/icon index
    pub item_frame: u16,
    /// Item count (for stacking)
    pub count: u8,
    /// Is this slot selected?
    pub selected: bool,
}

/// Main game panel - the HUD at bottom of screen
#[derive(Debug, Clone)]
pub struct MainPanel {
    /// Panel position on screen
    pub position: Point,
    /// Panel size
    pub size: Size,
    /// Is panel visible?
    pub visible: bool,
    /// Button states
    pub buttons: [PanelButtonState; MainPanelButton::COUNT],
    /// Health orb level
    pub health: OrbLevel,
    /// Mana orb level
    pub mana: OrbLevel,
    /// Experience bar
    pub experience: ExperienceBar,
    /// Belt slots
    pub belt: [BeltSlot; BELT_SLOT_COUNT],
    /// Currently hovered button (if any)
    pub hovered_button: Option<MainPanelButton>,
    /// Is chat available (multiplayer)?
    pub chat_available: bool,
}

impl MainPanel {
    /// Create a new main panel
    pub fn new() -> Self {
        Self {
            position: Point::new(0, 0),
            size: Size::new(MAIN_PANEL_WIDTH, MAIN_PANEL_HEIGHT),
            visible: true,
            buttons: [
                PanelButtonState::new(MainPanelButton::Character),
                PanelButtonState::new(MainPanelButton::Quests),
                PanelButtonState::new(MainPanelButton::Map),
                PanelButtonState::new(MainPanelButton::Menu),
                PanelButtonState::new(MainPanelButton::Inventory),
                PanelButtonState::new(MainPanelButton::Spells),
            ],
            health: OrbLevel::new(100, 100),
            mana: OrbLevel::new(50, 100),
            experience: ExperienceBar::new(0, 0, 1000),
            belt: [BeltSlot::default(); BELT_SLOT_COUNT],
            hovered_button: None,
            chat_available: false,
        }
    }

    /// Set panel position (usually centered at bottom)
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
    }

    /// Center the panel horizontally at the bottom of screen
    pub fn center_at_bottom(&mut self, screen_width: i32, screen_height: i32) {
        self.position.x = (screen_width - self.size.width) / 2;
        self.position.y = screen_height - self.size.height;
    }

    /// Update health orb
    pub fn set_health(&mut self, current: i32, max: i32) {
        self.health = OrbLevel::new(current, max);
    }

    /// Update mana orb
    pub fn set_mana(&mut self, current: i32, max: i32) {
        self.mana = OrbLevel::new(current, max);
    }

    /// Update experience bar
    pub fn set_experience(&mut self, current: u32, level_start: u32, next_level: u32) {
        self.experience = ExperienceBar::new(current, level_start, next_level);
    }

    /// Set belt slot item
    pub fn set_belt_item(&mut self, slot: usize, item_frame: u16, count: u8) {
        if slot < BELT_SLOT_COUNT {
            self.belt[slot] = BeltSlot {
                has_item: true,
                item_frame,
                count,
                selected: false,
            };
        }
    }

    /// Clear belt slot
    pub fn clear_belt_slot(&mut self, slot: usize) {
        if slot < BELT_SLOT_COUNT {
            self.belt[slot] = BeltSlot::default();
        }
    }

    /// Get button rectangle in screen coordinates
    pub fn get_button_rect(&self, button: MainPanelButton) -> Rectangle {
        let base = MAIN_PANEL_BUTTON_RECTS[button.index()];
        Rectangle::new(
            self.position.x + base.position.x,
            self.position.y + base.position.y,
            base.size.width,
            base.size.height,
        )
    }

    /// Get health orb rectangle
    pub fn get_health_orb_rect(&self) -> Rectangle {
        Rectangle::new(
            self.position.x + HEALTH_ORB_POSITION.x,
            self.position.y + HEALTH_ORB_POSITION.y,
            ORB_WIDTH,
            ORB_HEIGHT,
        )
    }

    /// Get mana orb rectangle
    pub fn get_mana_orb_rect(&self) -> Rectangle {
        Rectangle::new(
            self.position.x + MANA_ORB_POSITION.x,
            self.position.y + MANA_ORB_POSITION.y,
            ORB_WIDTH,
            ORB_HEIGHT,
        )
    }

    /// Get belt slot rectangle
    pub fn get_belt_slot_rect(&self, slot: usize) -> Option<Rectangle> {
        if slot >= BELT_SLOT_COUNT {
            return None;
        }
        Some(Rectangle::new(
            self.position.x + BELT_POSITION.x + (slot as i32 * BELT_SLOT_WIDTH),
            self.position.y + BELT_POSITION.y,
            BELT_SLOT_WIDTH,
            BELT_SLOT_HEIGHT,
        ))
    }

    /// Handle mouse movement
    pub fn on_mouse_move(&mut self, mouse_x: i32, mouse_y: i32) {
        let mouse = Point::new(mouse_x, mouse_y);
        self.hovered_button = None;

        // Pre-calculate all rects to avoid borrow issues
        let rects: Vec<Rectangle> = (0..MainPanelButton::COUNT)
            .map(|i| {
                let base = MAIN_PANEL_BUTTON_RECTS[i];
                Rectangle::new(
                    self.position.x + base.position.x,
                    self.position.y + base.position.y,
                    base.size.width,
                    base.size.height,
                )
            })
            .collect();

        for (i, button) in self.buttons.iter_mut().enumerate() {
            if rects[i].contains(mouse) {
                button.state = if button.state == ButtonState::Pressed {
                    ButtonState::Pressed
                } else {
                    ButtonState::Hovered
                };
                self.hovered_button = Some(button.button);
            } else if button.state == ButtonState::Hovered {
                button.state = ButtonState::Normal;
            }
        }
    }

    /// Handle mouse button down
    pub fn on_mouse_down(&mut self, mouse_x: i32, mouse_y: i32) -> Option<MainPanelButton> {
        let mouse = Point::new(mouse_x, mouse_y);

        // Pre-calculate all rects to avoid borrow issues
        let rects: Vec<Rectangle> = (0..MainPanelButton::COUNT)
            .map(|i| {
                let base = MAIN_PANEL_BUTTON_RECTS[i];
                Rectangle::new(
                    self.position.x + base.position.x,
                    self.position.y + base.position.y,
                    base.size.width,
                    base.size.height,
                )
            })
            .collect();

        for (i, button) in self.buttons.iter_mut().enumerate() {
            if rects[i].contains(mouse) {
                button.state = ButtonState::Pressed;
                return Some(button.button);
            }
        }

        None
    }

    /// Handle mouse button up
    pub fn on_mouse_up(&mut self, mouse_x: i32, mouse_y: i32) -> Option<MainPanelButton> {
        let mouse = Point::new(mouse_x, mouse_y);
        let mut clicked = None;

        // Pre-calculate all rects to avoid borrow issues
        let rects: Vec<Rectangle> = (0..MainPanelButton::COUNT)
            .map(|i| {
                let base = MAIN_PANEL_BUTTON_RECTS[i];
                Rectangle::new(
                    self.position.x + base.position.x,
                    self.position.y + base.position.y,
                    base.size.width,
                    base.size.height,
                )
            })
            .collect();

        for (i, button) in self.buttons.iter_mut().enumerate() {
            if button.state == ButtonState::Pressed {
                if rects[i].contains(mouse) {
                    clicked = Some(button.button);
                }
                button.state = ButtonState::Normal;
            }
        }

        clicked
    }

    /// Toggle panel for button
    pub fn toggle_panel(&mut self, button: MainPanelButton) {
        let idx = button.index();
        self.buttons[idx].panel_open = !self.buttons[idx].panel_open;
    }

    /// Check if panel is open
    pub fn is_panel_open(&self, button: MainPanelButton) -> bool {
        self.buttons[button.index()].panel_open
    }

    /// Close all panels
    pub fn close_all_panels(&mut self) {
        for button in &mut self.buttons {
            button.panel_open = false;
        }
    }

    /// Check if clicking on health orb
    pub fn hit_test_health_orb(&self, x: i32, y: i32) -> bool {
        self.get_health_orb_rect().contains(Point::new(x, y))
    }

    /// Check if clicking on mana orb
    pub fn hit_test_mana_orb(&self, x: i32, y: i32) -> bool {
        self.get_mana_orb_rect().contains(Point::new(x, y))
    }

    /// Check if clicking on belt slot
    pub fn hit_test_belt_slot(&self, x: i32, y: i32) -> Option<usize> {
        for i in 0..BELT_SLOT_COUNT {
            if let Some(rect) = self.get_belt_slot_rect(i) {
                if rect.contains(Point::new(x, y)) {
                    return Some(i);
                }
            }
        }
        None
    }
}

impl Default for MainPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Talk button for multiplayer voice chat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkButtonState {
    /// Normal voice button
    Voice,
    /// Pressed voice button
    VoicePressed,
    /// Mute button
    Mute,
    /// Pressed mute button
    MutePressed,
}

/// Multiplayer talk panel
#[derive(Debug, Clone)]
pub struct TalkPanel {
    /// Is panel visible?
    pub visible: bool,
    /// Talk button states for each player (up to 3 other players)
    pub buttons: [TalkButtonState; 3],
    /// Player names
    pub player_names: [String; 3],
}

impl TalkPanel {
    pub fn new() -> Self {
        Self {
            visible: false,
            buttons: [TalkButtonState::Voice; 3],
            player_names: [String::new(), String::new(), String::new()],
        }
    }

    /// Set player info
    pub fn set_player(&mut self, index: usize, name: &str) {
        if index < 3 {
            self.player_names[index] = name.to_string();
        }
    }

    /// Toggle mute for player
    pub fn toggle_mute(&mut self, index: usize) {
        if index < 3 {
            self.buttons[index] = match self.buttons[index] {
                TalkButtonState::Voice | TalkButtonState::VoicePressed => TalkButtonState::Mute,
                TalkButtonState::Mute | TalkButtonState::MutePressed => TalkButtonState::Voice,
            };
        }
    }
}

impl Default for TalkPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Panel rendering order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PanelLayer {
    /// Background panels (main panel background)
    Background = 0,
    /// Orb fills
    Orbs = 1,
    /// Belt items
    Belt = 2,
    /// Buttons
    Buttons = 3,
    /// Experience bar
    Experience = 4,
    /// Overlays (highlights, selections)
    Overlay = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_panel_button_labels() {
        assert_eq!(MainPanelButton::Character.label(), "char");
        assert_eq!(MainPanelButton::Quests.label(), "quests");
        assert_eq!(MainPanelButton::Inventory.label(), "inv");
    }

    #[test]
    fn test_main_panel_button_index() {
        assert_eq!(MainPanelButton::Character.index(), 0);
        assert_eq!(MainPanelButton::Spells.index(), 5);
        assert_eq!(MainPanelButton::from_index(3), Some(MainPanelButton::Menu));
        assert_eq!(MainPanelButton::from_index(10), None);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(10, 10, 50, 50);
        assert!(rect.contains(Point::new(25, 25)));
        assert!(rect.contains(Point::new(10, 10)));
        assert!(!rect.contains(Point::new(60, 60)));
        assert!(!rect.contains(Point::new(5, 25)));
    }

    #[test]
    fn test_orb_level() {
        let orb = OrbLevel::new(50, 100);
        assert!((orb.percentage() - 0.5).abs() < 0.001);
        assert_eq!(orb.fill_height(), 44);

        let full = OrbLevel::new(100, 100);
        assert!((full.percentage() - 1.0).abs() < 0.001);

        let empty = OrbLevel::new(0, 100);
        assert!((empty.percentage() - 0.0).abs() < 0.001);

        let overflow = OrbLevel::new(150, 100);
        assert!((overflow.percentage() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_experience_bar() {
        let exp = ExperienceBar::new(500, 0, 1000);
        assert!((exp.percentage() - 0.5).abs() < 0.001);

        let exp2 = ExperienceBar::new(1500, 1000, 2000);
        assert!((exp2.percentage() - 0.5).abs() < 0.001);

        let max = ExperienceBar::new(1000, 1000, 1000);
        assert!((max.percentage() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_main_panel_new() {
        let panel = MainPanel::new();
        assert!(panel.visible);
        assert_eq!(panel.buttons.len(), 6);
        assert!(panel.hovered_button.is_none());
    }

    #[test]
    fn test_main_panel_center() {
        let mut panel = MainPanel::new();
        panel.center_at_bottom(800, 600);
        assert_eq!(panel.position.x, 80);
        assert_eq!(panel.position.y, 472);
    }

    #[test]
    fn test_main_panel_health_mana() {
        let mut panel = MainPanel::new();
        panel.set_health(75, 150);
        assert_eq!(panel.health.current, 75);
        assert_eq!(panel.health.max, 150);
        assert!((panel.health.percentage() - 0.5).abs() < 0.001);

        panel.set_mana(25, 100);
        assert_eq!(panel.mana.current, 25);
        assert_eq!(panel.mana.max, 100);
    }

    #[test]
    fn test_main_panel_belt() {
        let mut panel = MainPanel::new();
        panel.set_belt_item(0, 42, 5);
        assert!(panel.belt[0].has_item);
        assert_eq!(panel.belt[0].item_frame, 42);
        assert_eq!(panel.belt[0].count, 5);

        panel.clear_belt_slot(0);
        assert!(!panel.belt[0].has_item);
    }

    #[test]
    fn test_main_panel_toggle() {
        let mut panel = MainPanel::new();
        assert!(!panel.is_panel_open(MainPanelButton::Character));

        panel.toggle_panel(MainPanelButton::Character);
        assert!(panel.is_panel_open(MainPanelButton::Character));

        panel.toggle_panel(MainPanelButton::Character);
        assert!(!panel.is_panel_open(MainPanelButton::Character));
    }

    #[test]
    fn test_main_panel_close_all() {
        let mut panel = MainPanel::new();
        panel.toggle_panel(MainPanelButton::Character);
        panel.toggle_panel(MainPanelButton::Inventory);
        assert!(panel.is_panel_open(MainPanelButton::Character));
        assert!(panel.is_panel_open(MainPanelButton::Inventory));

        panel.close_all_panels();
        assert!(!panel.is_panel_open(MainPanelButton::Character));
        assert!(!panel.is_panel_open(MainPanelButton::Inventory));
    }

    #[test]
    fn test_talk_panel() {
        let mut talk = TalkPanel::new();
        assert!(!talk.visible);
        assert_eq!(talk.buttons[0], TalkButtonState::Voice);

        talk.set_player(0, "Player1");
        assert_eq!(talk.player_names[0], "Player1");

        talk.toggle_mute(0);
        assert_eq!(talk.buttons[0], TalkButtonState::Mute);

        talk.toggle_mute(0);
        assert_eq!(talk.buttons[0], TalkButtonState::Voice);
    }
}
