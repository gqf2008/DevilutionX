//! Control Panel System - M21
//!
//! Precision port of DevilutionX control.cpp
//! Handles character and main control panels, UI buttons, info display

use bitflags::bitflags;

// ============================================================================
// Constants
// ============================================================================

/// Maximum length of chat/send message
pub const MAX_SEND_STR_LEN: usize = 80;

/// Number of belt item slots
pub const BELT_ITEMS: usize = 8;

/// Inventory slot size in pixels
pub const INV_SLOT_SIZE_PX: i32 = 28;

/// Panel padding height
pub const PANEL_PADDING_HEIGHT: i32 = 16;

// ============================================================================
// Types
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

/// Displacement (offset)
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
// Panel Button IDs
// ============================================================================

/// Main panel button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PanelButtonId {
    /// Character information panel
    Charinfo = 0,
    /// Quest log panel
    Qlog = 1,
    /// Automap toggle
    Automap = 2,
    /// Main menu
    Mainmenu = 3,
    /// Inventory panel
    Inventory = 4,
    /// Spellbook panel
    Spellbook = 5,
    /// Send message (chat)
    Sendmsg = 6,
    /// Friendly fire toggle
    Friendly = 7,
}

impl PanelButtonId {
    pub const FIRST: Self = Self::Charinfo;
    pub const LAST: Self = Self::Friendly;

    /// Get hotkey string for button
    pub fn hotkey(&self) -> &'static str {
        match self {
            Self::Charinfo => "'c'",
            Self::Qlog => "'q'",
            Self::Automap => "Tab",
            Self::Mainmenu => "Esc",
            Self::Inventory => "'i'",
            Self::Spellbook => "'b'",
            Self::Sendmsg => "Enter",
            Self::Friendly => "",
        }
    }

    /// Get description for button
    pub fn description(&self) -> &'static str {
        match self {
            Self::Charinfo => "Character Information",
            Self::Qlog => "Quests log",
            Self::Automap => "Automap",
            Self::Mainmenu => "Main Menu",
            Self::Inventory => "Inventory",
            Self::Spellbook => "Spell book",
            Self::Sendmsg => "Send Message",
            Self::Friendly => "", // Player attack
        }
    }

    /// Get all button IDs
    pub fn all() -> &'static [Self] {
        &[
            Self::Charinfo,
            Self::Qlog,
            Self::Automap,
            Self::Mainmenu,
            Self::Inventory,
            Self::Spellbook,
            Self::Sendmsg,
            Self::Friendly,
        ]
    }
}

/// Character attribute for stat increase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CharacterAttribute {
    Strength = 0,
    Magic = 1,
    Dexterity = 2,
    Vitality = 3,
}

impl CharacterAttribute {
    pub fn all() -> &'static [Self] {
        &[Self::Strength, Self::Magic, Self::Dexterity, Self::Vitality]
    }
}

// ============================================================================
// UI Panels Enum
// ============================================================================

/// UI Panel types for positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiPanels {
    Main,
    Character,
    Inventory,
    Spellbook,
    Quest,
}

// ============================================================================
// UI Flags
// ============================================================================

bitflags! {
    /// UI rendering flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct UiFlags: u32 {
        const COLOR_WHITE = 1 << 0;
        const COLOR_BLUE = 1 << 1;
        const COLOR_RED = 1 << 2;
        const COLOR_GOLD = 1 << 3;
        const COLOR_BLACK = 1 << 4;
        const COLOR_WHITEGOLD = 1 << 5;
        const ALIGN_CENTER = 1 << 8;
        const ALIGN_RIGHT = 1 << 9;
        const VERTICAL_CENTER = 1 << 10;
        const KERNING_FIT_SPACING = 1 << 11;
        const PENDING = 1 << 12;
    }
}

// ============================================================================
// Panel Draw Components
// ============================================================================

/// Components that can be redrawn
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelDrawComponent {
    ControlButtons,
    Health,
    Mana,
    Belt,
    Spell,
    InfoBox,
    All,
}

// ============================================================================
// Button Rectangles (Layout Constants)
// ============================================================================

/// Wide panel button size
pub const WIDE_PANEL_BUTTON_SIZE: Size = Size::new(71, 20);

/// Standard panel button size
pub const PANEL_BUTTON_SIZE: Size = Size::new(33, 32);

/// Increment attribute button size
pub const INCREMENT_ATTR_BUTTON_SIZE: Size = Size::new(41, 22);

/// Main panel button rectangles
pub const MAIN_PANEL_BUTTON_RECTS: [Rectangle; 8] = [
    Rectangle::new(9, 9, 71, 20),    // char button
    Rectangle::new(9, 35, 71, 20),   // quests button
    Rectangle::new(9, 75, 71, 20),   // map button
    Rectangle::new(9, 101, 71, 20),  // menu button
    Rectangle::new(560, 9, 71, 20),  // inv button
    Rectangle::new(560, 35, 71, 20), // spells button
    Rectangle::new(87, 91, 33, 32),  // chat button
    Rectangle::new(527, 91, 33, 32), // friendly fire button
];

/// Character panel attribute button rectangles
pub const CHAR_PANEL_BUTTON_RECTS: [Rectangle; 4] = [
    Rectangle::new(137, 138, 41, 22), // Strength
    Rectangle::new(137, 166, 41, 22), // Magic
    Rectangle::new(137, 195, 41, 22), // Dexterity
    Rectangle::new(137, 223, 41, 22), // Vitality
];

/// Level up button rectangle (relative to main panel)
pub const LEVEL_BUTTON_RECT: Rectangle = Rectangle::new(40, -39, 41, 22);

/// Belt rectangle
pub const BELT_RECT: Rectangle = Rectangle::new(205, 5, (INV_SLOT_SIZE_PX + 1) * BELT_ITEMS as i32, INV_SLOT_SIZE_PX);

/// Spell button rectangle
pub const SPELL_BUTTON_RECT: Rectangle = Rectangle::new(565, 64, 56, 56);

/// Flask top rectangle
pub const FLASK_TOP_RECT: Rectangle = Rectangle::new(11, 3, 62, 13);

/// Flask bottom rectangle
pub const FLASK_BOTTOM_RECT: Rectangle = Rectangle::new(0, 16, 88, 69);

/// Info box rectangle
pub const INFO_BOX_RECT: Rectangle = Rectangle::new(177, 46, 288, 60);

/// Mute button rectangle
pub const MUTE_BUTTON_RECT: Rectangle = Rectangle::new(172, 69, 61, 16);

// ============================================================================
// Panel State
// ============================================================================

/// Main control panel state
#[derive(Debug, Default)]
pub struct PanelState {
    /// Character panel is open
    pub char_flag: bool,
    /// Inventory panel is open
    pub inv_flag: bool,
    /// Spellbook panel is open
    pub spellbook_flag: bool,
    /// Quest log is open
    pub quest_log_open: bool,
    /// Stash is open
    pub stash_open: bool,
    /// Chat input is active
    pub chat_flag: bool,
    /// Spell selection is active
    pub spell_select_flag: bool,
    /// Gold drop dialog is open
    pub drop_gold_flag: bool,
    /// Main panel is active
    pub main_panel_flag: bool,
    /// Main panel button is held down
    pub main_panel_button_down: bool,
    /// Level button is held down
    pub level_button_down: bool,
    /// Character panel button is active
    pub char_panel_button_active: bool,
    /// Currently selected spellbook tab (0-3)
    pub spellbook_tab: i32,

    /// Main panel button states
    pub main_panel_buttons: [bool; 8],
    /// Character panel button states (attribute increase)
    pub char_panel_buttons: [bool; 4],

    /// Talk button states
    pub talk_buttons_down: [bool; 3],
    /// Whisper list for multiplayer
    pub whisper_list: [bool; 4], // MAX_PLRS

    /// Info string to display
    pub info_string: String,
    /// Floating info string
    pub floating_info_string: String,
    /// Info text color
    pub info_color: UiFlags,

    /// Gold drop text
    pub gold_drop_text: String,
    /// Gold drop inventory index
    pub gold_drop_inv_index: i8,

    /// Chat message being composed
    pub talk_message: String,
    /// Saved chat messages
    pub talk_save: [String; 8],
    /// Current talk save index
    pub talk_save_index: u8,
    /// Next talk save slot
    pub next_talk_save: u8,

    /// Talk player table
    pub sgb_plr_talk_tbl: i32,
}

impl PanelState {
    pub fn new() -> Self {
        Self {
            info_color: UiFlags::COLOR_WHITE,
            ..Default::default()
        }
    }

    /// Check if left panel is open (character/quest/stash)
    pub fn is_left_panel_open(&self) -> bool {
        self.char_flag || self.quest_log_open || self.stash_open
    }

    /// Check if right panel is open (inventory/spellbook)
    pub fn is_right_panel_open(&self) -> bool {
        self.inv_flag || self.spellbook_flag
    }

    /// Open character panel
    pub fn open_char_panel(&mut self) {
        self.char_flag = true;
    }

    /// Close character panel
    pub fn close_char_panel(&mut self) {
        self.char_flag = false;
    }

    /// Toggle character panel
    pub fn toggle_char_panel(&mut self) {
        self.char_flag = !self.char_flag;
    }

    /// Open inventory panel
    pub fn open_inventory(&mut self) {
        self.inv_flag = true;
    }

    /// Close inventory panel
    pub fn close_inventory(&mut self) {
        self.inv_flag = false;
    }

    /// Toggle inventory panel
    pub fn toggle_inventory(&mut self) {
        self.inv_flag = !self.inv_flag;
    }

    /// Open spellbook panel
    pub fn open_spellbook(&mut self) {
        self.spellbook_flag = true;
    }

    /// Close spellbook panel
    pub fn close_spellbook(&mut self) {
        self.spellbook_flag = false;
    }

    /// Toggle spellbook panel
    pub fn toggle_spellbook(&mut self) {
        self.spellbook_flag = !self.spellbook_flag;
    }

    /// Open quest log
    pub fn open_quest_log(&mut self) {
        self.quest_log_open = true;
    }

    /// Close quest log
    pub fn close_quest_log(&mut self) {
        self.quest_log_open = false;
    }

    /// Toggle quest log
    pub fn toggle_quest_log(&mut self) {
        self.quest_log_open = !self.quest_log_open;
    }

    /// Start chat input
    pub fn start_chat(&mut self) {
        self.chat_flag = true;
        self.talk_message.clear();
    }

    /// End chat input
    pub fn end_chat(&mut self) {
        self.chat_flag = false;
    }

    /// Toggle chat
    pub fn toggle_chat(&mut self) {
        if self.chat_flag {
            self.end_chat();
        } else {
            self.start_chat();
        }
    }

    /// Clear info string
    pub fn clear_info(&mut self) {
        self.info_string.clear();
        self.info_color = UiFlags::COLOR_WHITE;
    }

    /// Set info string
    pub fn set_info(&mut self, text: &str, color: UiFlags) {
        self.info_string = text.to_string();
        self.info_color = color;
    }

    /// Add line to info string
    pub fn add_info_line(&mut self, text: &str) {
        if !self.info_string.is_empty() {
            self.info_string.push('\n');
        }
        self.info_string.push_str(text);
    }

    /// Start gold drop dialog
    pub fn start_gold_drop(&mut self, inv_index: i8) {
        self.drop_gold_flag = true;
        self.gold_drop_inv_index = inv_index;
        self.gold_drop_text.clear();
    }

    /// End gold drop dialog
    pub fn end_gold_drop(&mut self) {
        self.drop_gold_flag = false;
        self.gold_drop_inv_index = -1;
    }

    /// Save current chat message
    pub fn save_talk_message(&mut self) {
        if self.talk_message.is_empty() {
            return;
        }
        self.talk_save[self.next_talk_save as usize] = self.talk_message.clone();
        self.next_talk_save = (self.next_talk_save + 1) % 8;
    }

    /// Recall previous chat message
    pub fn recall_talk_message(&mut self, direction: i8) {
        let new_index = if direction > 0 {
            (self.talk_save_index + 1) % 8
        } else {
            (self.talk_save_index + 7) % 8
        };
        self.talk_save_index = new_index;
        self.talk_message = self.talk_save[new_index as usize].clone();
    }
}

// ============================================================================
// Control Manager
// ============================================================================

/// Main control system manager
pub struct ControlManager {
    /// Panel state
    pub state: PanelState,

    /// Main panel rectangle
    pub main_panel: Rectangle,
    /// Left panel rectangle
    pub left_panel: Rectangle,
    /// Right panel rectangle
    pub right_panel: Rectangle,

    /// Screen dimensions
    screen_width: i32,
    screen_height: i32,

    /// Number of main panel buttons (single player vs multiplayer)
    total_main_panel_buttons: i32,

    /// Mute button count
    mute_buttons: i32,
    /// Mute button padding
    mute_button_padding: i32,

    /// Current mouse position
    mouse_position: Point,
}

impl Default for ControlManager {
    fn default() -> Self {
        Self::new(640, 480)
    }
}

impl ControlManager {
    /// Create new control manager
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
            state: PanelState::new(),
            main_panel,
            left_panel,
            right_panel,
            screen_width,
            screen_height,
            total_main_panel_buttons: 6, // Single player default
            mute_buttons: 3,
            mute_button_padding: 2,
            mouse_position: Point::default(),
        }
    }

    /// Initialize for multiplayer (more buttons)
    pub fn init_multiplayer(&mut self) {
        self.total_main_panel_buttons = 8;
    }

    /// Initialize for single player
    pub fn init_single_player(&mut self) {
        self.total_main_panel_buttons = 6;
    }

    /// Update mouse position
    pub fn set_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_position = Point::new(x, y);
    }

    /// Get panel position for object placement
    pub fn get_panel_position(&self, panel: UiPanels, offset: Point) -> Point {
        match panel {
            UiPanels::Main => self.main_panel.position.offset(offset.x, offset.y),
            UiPanels::Character => self.left_panel.position.offset(offset.x, offset.y),
            UiPanels::Inventory => self.right_panel.position.offset(offset.x, offset.y),
            UiPanels::Spellbook => self.right_panel.position.offset(offset.x, offset.y),
            UiPanels::Quest => self.left_panel.position.offset(offset.x, offset.y),
        }
    }

    /// Check if main panel button is clicked
    pub fn check_main_panel_button(&mut self) -> Option<PanelButtonId> {
        for i in 0..self.total_main_panel_buttons as usize {
            let mut button = MAIN_PANEL_BUTTON_RECTS[i];
            button.position = self.get_panel_position(UiPanels::Main, button.position);

            if button.contains(self.mouse_position) {
                self.state.main_panel_buttons[i] = true;
                self.state.main_panel_button_down = true;
                return Some(match i {
                    0 => PanelButtonId::Charinfo,
                    1 => PanelButtonId::Qlog,
                    2 => PanelButtonId::Automap,
                    3 => PanelButtonId::Mainmenu,
                    4 => PanelButtonId::Inventory,
                    5 => PanelButtonId::Spellbook,
                    6 => PanelButtonId::Sendmsg,
                    7 => PanelButtonId::Friendly,
                    _ => continue,
                });
            }
        }
        None
    }

    /// Release main panel button
    pub fn release_main_panel_button(&mut self) {
        for i in 0..8 {
            self.state.main_panel_buttons[i] = false;
        }
        self.state.main_panel_button_down = false;
    }

    /// Execute main panel button action
    pub fn do_main_panel_button(&mut self, button: PanelButtonId) {
        match button {
            PanelButtonId::Charinfo => {
                self.state.toggle_char_panel();
            }
            PanelButtonId::Qlog => {
                self.state.toggle_quest_log();
            }
            PanelButtonId::Automap => {
                // Automap toggle handled by automap module
                // TODO: integrate with automap
            }
            PanelButtonId::Mainmenu => {
                // Open game menu
                // TODO: integrate with game menu
            }
            PanelButtonId::Inventory => {
                self.state.toggle_inventory();
            }
            PanelButtonId::Spellbook => {
                self.state.toggle_spellbook();
            }
            PanelButtonId::Sendmsg => {
                self.state.toggle_chat();
            }
            PanelButtonId::Friendly => {
                // Toggle friendly fire
                // TODO: integrate with combat system
            }
        }
    }

    /// Check character panel attribute buttons
    pub fn check_char_buttons(&mut self, player_stat_points: i32) -> Option<CharacterAttribute> {
        if self.state.char_panel_button_active || player_stat_points == 0 {
            return None;
        }

        for (i, attr) in CharacterAttribute::all().iter().enumerate() {
            let mut button = CHAR_PANEL_BUTTON_RECTS[i];
            button.position = self.get_panel_position(UiPanels::Character, button.position);

            if button.contains(self.mouse_position) {
                self.state.char_panel_buttons[i] = true;
                self.state.char_panel_button_active = true;
                return Some(*attr);
            }
        }
        None
    }

    /// Release character panel buttons
    pub fn release_char_buttons(&mut self) {
        self.state.char_panel_button_active = false;
        for i in 0..4 {
            self.state.char_panel_buttons[i] = false;
        }
    }

    /// Check level up button
    pub fn check_level_button(&mut self, is_visible: bool) -> bool {
        if !is_visible {
            return false;
        }

        let mut button = LEVEL_BUTTON_RECT;
        button.position = self.get_panel_position(UiPanels::Main, button.position);

        if !self.state.level_button_down && button.contains(self.mouse_position) {
            self.state.level_button_down = true;
            return true;
        }
        false
    }

    /// Release level button and check if clicked
    pub fn release_level_button(&mut self) -> bool {
        let was_down = self.state.level_button_down;
        self.state.level_button_down = false;

        if !was_down {
            return false;
        }

        let mut button = LEVEL_BUTTON_RECT;
        button.position = self.get_panel_position(UiPanels::Main, button.position);
        button.contains(self.mouse_position)
    }

    /// Check spell button
    pub fn check_spell_button(&mut self) -> bool {
        let mut button = SPELL_BUTTON_RECT;
        button.position = self.get_panel_position(UiPanels::Main, button.position);
        button.contains(self.mouse_position)
    }

    /// Check belt slot click
    pub fn check_belt(&mut self) -> Option<usize> {
        let mut belt = BELT_RECT;
        belt.position = self.get_panel_position(UiPanels::Main, belt.position);

        if !belt.contains(self.mouse_position) {
            return None;
        }

        let slot_width = INV_SLOT_SIZE_PX + 1;
        let relative_x = self.mouse_position.x - belt.position.x;
        let slot = (relative_x / slot_width) as usize;

        if slot < BELT_ITEMS {
            Some(slot)
        } else {
            None
        }
    }

    /// Check if panels can cover view area
    pub fn can_panels_cover_view(&self) -> bool {
        // Check if there's room for panels without overlapping game view
        let available_width = self.screen_width - self.left_panel.size.width - self.right_panel.size.width;
        available_width < 320 // Minimum game view width
    }

    /// Get life flask fill percentage (0-80)
    pub fn get_life_fill(&self, current_hp: i32, max_hp: i32) -> i32 {
        if max_hp <= 0 {
            return 0;
        }
        ((current_hp as i64 * 80) / max_hp as i64).clamp(0, 80) as i32
    }

    /// Get mana flask fill percentage (0-80)
    pub fn get_mana_fill(&self, current_mana: i32, max_mana: i32) -> i32 {
        if max_mana <= 0 {
            return 0;
        }
        ((current_mana as i64 * 80) / max_mana as i64).clamp(0, 80) as i32
    }

    /// Calculate info box position
    pub fn get_info_box_rect(&self) -> Rectangle {
        let mut rect = INFO_BOX_RECT;
        rect.position = self.get_panel_position(UiPanels::Main, rect.position);
        rect.position.y += PANEL_PADDING_HEIGHT;
        rect
    }

    /// Check if mouse is in info box
    pub fn is_in_info_box(&self) -> bool {
        self.get_info_box_rect().contains(self.mouse_position)
    }

    /// Get screen center for UI positioning
    pub fn get_screen_center(&self) -> Point {
        Point::new(self.screen_width / 2, self.screen_height / 2)
    }
}

// ============================================================================
// Flask Drawing Support
// ============================================================================

/// Flask type for drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlaskType {
    Life,
    Mana,
}

/// Calculate flask drawing parameters
pub struct FlaskDrawParams {
    /// X offset for flask
    pub offset: i32,
    /// Empty rows in top section
    pub top_empty_rows: i32,
    /// Filled rows in top section
    pub top_filled_rows: i32,
    /// Empty rows in bottom section
    pub bottom_empty_rows: i32,
    /// Filled rows in bottom section
    pub bottom_filled_rows: i32,
}

impl FlaskDrawParams {
    /// Calculate flask draw parameters from fill percentage (0-80)
    pub fn from_fill(fill_percent: i32, flask_type: FlaskType) -> Self {
        let offset = match flask_type {
            FlaskType::Life => 0,
            FlaskType::Mana => 552, // Right side offset
        };

        // Top section is 13 pixels, bottom is 69 pixels
        // fill_percent 0-80 maps to total 82 pixels (13+69)
        let total_fill = fill_percent + 1; // 1-81 range

        let top_height = FLASK_TOP_RECT.size.height;
        let bottom_height = FLASK_BOTTOM_RECT.size.height;

        let top_empty_rows = (81 - total_fill).clamp(0, top_height);
        let top_filled_rows = top_height - top_empty_rows;

        let bottom_filled = (total_fill - top_height).max(0);
        let bottom_filled_rows = bottom_filled.clamp(0, bottom_height);
        let bottom_empty_rows = bottom_height - bottom_filled_rows;

        Self {
            offset,
            top_empty_rows,
            top_filled_rows,
            bottom_empty_rows,
            bottom_filled_rows,
        }
    }
}

// ============================================================================
// Durability Icon Support
// ============================================================================

/// Equipment slot for durability icons
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EquipSlot {
    Head,
    Chest,
    LeftHand,
    RightHand,
}

/// Durability state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityState {
    /// Item is in good condition
    Good,
    /// Item needs repair (yellow)
    NeedsRepair,
    /// Item is about to break (red)
    Critical,
    /// Item is broken
    Broken,
}

impl DurabilityState {
    /// Get durability state from current/max values
    pub fn from_durability(current: i32, max: i32) -> Self {
        if current <= 0 {
            Self::Broken
        } else if max > 0 {
            let percent = (current * 100) / max;
            if percent <= 10 {
                Self::Critical
            } else if percent <= 25 {
                Self::NeedsRepair
            } else {
                Self::Good
            }
        } else {
            Self::Good
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_button_id() {
        assert_eq!(PanelButtonId::Charinfo.hotkey(), "'c'");
        assert_eq!(PanelButtonId::Automap.hotkey(), "Tab");
        assert_eq!(PanelButtonId::Mainmenu.description(), "Main Menu");
        assert_eq!(PanelButtonId::all().len(), 8);
    }

    #[test]
    fn test_character_attribute() {
        assert_eq!(CharacterAttribute::all().len(), 4);
        assert_eq!(CharacterAttribute::Strength as u8, 0);
        assert_eq!(CharacterAttribute::Vitality as u8, 3);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(10, 20, 100, 50);
        assert!(rect.contains(Point::new(50, 40)));
        assert!(rect.contains(Point::new(10, 20)));
        assert!(!rect.contains(Point::new(110, 70)));
        assert!(!rect.contains(Point::new(9, 20)));
    }

    #[test]
    fn test_panel_state_init() {
        let state = PanelState::new();
        assert!(!state.char_flag);
        assert!(!state.inv_flag);
        assert!(!state.chat_flag);
        assert_eq!(state.info_color, UiFlags::COLOR_WHITE);
    }

    #[test]
    fn test_panel_state_toggles() {
        let mut state = PanelState::new();

        assert!(!state.char_flag);
        state.toggle_char_panel();
        assert!(state.char_flag);
        state.toggle_char_panel();
        assert!(!state.char_flag);

        assert!(!state.inv_flag);
        state.toggle_inventory();
        assert!(state.inv_flag);
    }

    #[test]
    fn test_panel_state_left_right() {
        let mut state = PanelState::new();

        assert!(!state.is_left_panel_open());
        assert!(!state.is_right_panel_open());

        state.char_flag = true;
        assert!(state.is_left_panel_open());

        state.inv_flag = true;
        assert!(state.is_right_panel_open());
    }

    #[test]
    fn test_panel_state_info() {
        let mut state = PanelState::new();

        state.set_info("Test info", UiFlags::COLOR_GOLD);
        assert_eq!(state.info_string, "Test info");
        assert_eq!(state.info_color, UiFlags::COLOR_GOLD);

        state.add_info_line("Line 2");
        assert!(state.info_string.contains('\n'));

        state.clear_info();
        assert!(state.info_string.is_empty());
    }

    #[test]
    fn test_panel_state_chat() {
        let mut state = PanelState::new();

        assert!(!state.chat_flag);
        state.start_chat();
        assert!(state.chat_flag);
        assert!(state.talk_message.is_empty());

        state.end_chat();
        assert!(!state.chat_flag);
    }

    #[test]
    fn test_control_manager_init() {
        let manager = ControlManager::new(640, 480);
        assert_eq!(manager.screen_width, 640);
        assert_eq!(manager.screen_height, 480);
        assert_eq!(manager.total_main_panel_buttons, 6);
    }

    #[test]
    fn test_control_manager_multiplayer() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();
        assert_eq!(manager.total_main_panel_buttons, 8);

        manager.init_single_player();
        assert_eq!(manager.total_main_panel_buttons, 6);
    }

    #[test]
    fn test_control_manager_flask_fill() {
        let manager = ControlManager::new(640, 480);

        assert_eq!(manager.get_life_fill(100, 100), 80);
        assert_eq!(manager.get_life_fill(50, 100), 40);
        assert_eq!(manager.get_life_fill(0, 100), 0);
        assert_eq!(manager.get_life_fill(100, 0), 0);
    }

    #[test]
    fn test_flask_draw_params() {
        let params = FlaskDrawParams::from_fill(80, FlaskType::Life);
        assert_eq!(params.offset, 0);
        assert_eq!(params.top_filled_rows, 13);

        let params = FlaskDrawParams::from_fill(40, FlaskType::Mana);
        assert_eq!(params.offset, 552);
    }

    #[test]
    fn test_durability_state() {
        assert_eq!(DurabilityState::from_durability(100, 100), DurabilityState::Good);
        assert_eq!(DurabilityState::from_durability(25, 100), DurabilityState::NeedsRepair);
        assert_eq!(DurabilityState::from_durability(5, 100), DurabilityState::Critical);
        assert_eq!(DurabilityState::from_durability(0, 100), DurabilityState::Broken);
    }

    #[test]
    fn test_button_rects() {
        assert_eq!(MAIN_PANEL_BUTTON_RECTS.len(), 8);
        assert_eq!(CHAR_PANEL_BUTTON_RECTS.len(), 4);

        // Check first button position
        assert_eq!(MAIN_PANEL_BUTTON_RECTS[0].position.x, 9);
        assert_eq!(MAIN_PANEL_BUTTON_RECTS[0].position.y, 9);
    }

    #[test]
    fn test_belt_check() {
        let mut manager = ControlManager::new(640, 480);

        // Set mouse outside belt
        manager.set_mouse_position(0, 0);
        assert!(manager.check_belt().is_none());

        // Calculate belt position
        let belt_x = manager.main_panel.position.x + BELT_RECT.position.x;
        let belt_y = manager.main_panel.position.y + BELT_RECT.position.y;

        // Set mouse in first belt slot
        manager.set_mouse_position(belt_x + 5, belt_y + 5);
        assert_eq!(manager.check_belt(), Some(0));

        // Set mouse in second belt slot
        manager.set_mouse_position(belt_x + INV_SLOT_SIZE_PX + 5, belt_y + 5);
        assert_eq!(manager.check_belt(), Some(1));
    }
}
