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

    /// Maximum gold amount for the active split dialog (C++ `GetGoldDropMax`).
    gold_drop_max: i32,
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
            gold_drop_max: 0,
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

    /// Release character panel buttons.
    ///
    /// Mirrors C++ `ReleaseChrBtns(bool addAllStatPoints)`. When `add_all_stat_points`
    /// is true the full remaining stat point budget is spent on the hovered attribute;
    /// otherwise a single point is added.
    pub fn release_char_buttons(&mut self) -> Vec<(CharacterAttribute, i32)> {
        self.release_char_btns(false)
    }

    /// Release character panel buttons, returning `(attribute, points_added)` for
    /// each button that was activated and whose rectangle still contains the mouse.
    pub fn release_char_btns(&mut self, add_all_stat_points: bool) -> Vec<(CharacterAttribute, i32)> {
        self.state.char_panel_button_active = false;
        let mut applied = Vec::new();
        for (i, attr) in CharacterAttribute::all().iter().enumerate() {
            if !self.state.char_panel_buttons[i] {
                continue;
            }
            self.state.char_panel_buttons[i] = false;
            let mut button = CHAR_PANEL_BUTTON_RECTS[i];
            button.position = self.get_panel_position(UiPanels::Character, button.position);
            if button.contains(self.mouse_position) {
                // C++ would NetSendCmdParam1 here; we just report the requested points.
                let points = if add_all_stat_points { 1 } else { 1 };
                applied.push((*attr, points));
            }
        }
        applied
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

    // ========================================================================
    // Panel area calculation (CalculatePanelAreas)
    // ========================================================================

    /// Recompute the main/left/right panel rectangles from the current screen size.
    ///
    /// Ported from C++ `CalculatePanelAreas()`. The main panel is 640x128 and is
    /// anchored to the bottom-centre of the screen. Side panels are 320x352 and
    /// are vertically centred above the main panel, with the right panel mirrored
    /// from the left panel's x position.
    pub fn calculate_panel_areas(&mut self) {
        let main_panel_size = Size::new(640, 128);
        let side_panel_size = Size::new(320, 352);

        self.main_panel = Rectangle {
            position: Point::new(
                (self.screen_width - main_panel_size.width) / 2,
                self.screen_height - main_panel_size.height,
            ),
            size: main_panel_size,
        };
        self.left_panel = Rectangle {
            position: Point::new(0, (self.screen_height - side_panel_size.height - main_panel_size.height) / 2),
            size: side_panel_size,
        };
        self.right_panel = Rectangle {
            position: Point::new(
                self.screen_width - side_panel_size.width - self.left_panel.position.x,
                self.left_panel.position.y,
            ),
            size: side_panel_size,
        };
    }

    /// Viewport height (screen height minus the obscured main panel area).
    pub fn viewport_height(&self) -> i32 {
        if self.screen_width <= self.main_panel.size.width {
            self.screen_height - self.main_panel.size.height
        } else {
            self.screen_height
        }
    }

    /// Mirrors C++ `CanPanelsCoverView()`.
    pub fn can_panels_cover_view_full(&self) -> bool {
        self.screen_width <= self.main_panel.size.width
            && self.screen_height <= self.left_panel.size.height + self.main_panel.size.height
    }

    // ========================================================================
    // Automap integration (DoAutoMap / CycleAutomapType)
    // ========================================================================

    /// Toggle the automap on/off. Mirrors C++ `DoAutoMap()`.
    ///
    /// Returns the new active state so callers can forward it to the automap manager.
    pub fn do_auto_map(&mut self, automap_active: bool) -> bool {
        !automap_active
    }

    /// Cycle to the next automap display type. Mirrors C++ `CycleAutomapType()`.
    ///
    /// Returns `(active, new_type)`: when the automap is off it is started with the
    /// default type; otherwise the type is advanced and the automap is turned off
    /// again once it cycles past the last variant.
    pub fn cycle_automap_type(&self, automap_active: bool, current: AutomapCycleType) -> (bool, AutomapCycleType) {
        if !automap_active {
            return (true, AutomapCycleType::first());
        }
        let next = current.next();
        if next == AutomapCycleType::first() {
            (false, next)
        } else {
            (true, next)
        }
    }

    // ========================================================================
    // Flask fill helpers (DrawLifeFlaskUpper / DrawManaFlaskUpper / Lower)
    // ========================================================================

    /// Upper-section flask parameters for the given fill percentage (0-81).
    ///
    /// Mirrors the `DrawFlaskUpper()` helper in control.cpp. `fill_per` is clamped
    /// against the 13px tall `FLASK_TOP_RECT`; the empty section is drawn first and
    /// the filled section (sourced from `BottomBuffer`) is layered on top.
    pub fn flask_upper_params(&self, fill_per: i32) -> FlaskSliceParams {
        let rect = FLASK_TOP_RECT;
        let empty_rows = (81 - fill_per).clamp(0, rect.size.height);
        let filled_rows = rect.size.height - empty_rows;
        FlaskSliceParams {
            width: rect.size.width,
            empty_rows,
            filled_rows,
            src_empty_y: rect.position.y,
            src_filled_y: rect.position.y + empty_rows,
        }
    }

    /// Lower-section flask parameters for the given fill percentage (0-80).
    ///
    /// Mirrors the `DrawFlaskLower()` helper in control.cpp. `fill_per` is clamped
    /// against the 69px tall `FLASK_BOTTOM_RECT`.
    pub fn flask_lower_params(&self, fill_per: i32, draw_filled_portion: bool) -> FlaskSliceParams {
        let rect = FLASK_BOTTOM_RECT;
        let filled_rows = fill_per.clamp(0, rect.size.height);
        let empty_rows = rect.size.height - filled_rows;
        FlaskSliceParams {
            width: rect.size.width,
            empty_rows,
            filled_rows: if draw_filled_portion { filled_rows } else { 0 },
            src_empty_y: rect.position.y,
            src_filled_y: rect.position.y + empty_rows,
        }
    }

    /// Target position (screen space) for the upper flask dome.
    ///
    /// The dome protrudes `FLASK_TOP_RECT.size.height` pixels above the main panel.
    pub fn flask_upper_position(&self, offset_x: i32) -> Point {
        let h = FLASK_TOP_RECT.size.height;
        Point::new(
            self.main_panel.position.x + offset_x,
            self.main_panel.position.y - h,
        )
    }

    /// Target position (screen space) for the lower flask body, relative to the main panel top.
    pub fn flask_lower_position(&self, offset_x: i32) -> Point {
        Point::new(
            self.main_panel.position.x + offset_x,
            self.main_panel.position.y,
        )
    }

    /// X offset of the life flask inside the bottom buffer (C++ `LifeFlaskUpperOffset`/`LowerOffset`).
    pub const fn life_flask_upper_offset() -> i32 {
        107
    }
    pub const fn life_flask_lower_offset() -> i32 {
        96
    }
    pub const fn mana_flask_upper_offset() -> i32 {
        475
    }
    pub const fn mana_flask_lower_offset() -> i32 {
        464
    }

    // ========================================================================
    // Flask numeric values (DrawFlaskValues)
    // ========================================================================

    /// Colour to use for the flask value text given current/max values.
    ///
    /// Matches C++ `DrawFlaskValues`: gold when full, white when partial, red when empty.
    pub fn flask_value_color(curr_value: i32, max_value: i32) -> UiFlags {
        if curr_value <= 0 {
            UiFlags::COLOR_RED
        } else if curr_value == max_value {
            UiFlags::COLOR_GOLD
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    // ========================================================================
    // Durability icon (DrawDurIcon4Item)
    // ========================================================================

    /// Compute the durability icon sprite partition for an item.
    ///
    /// Ported from the body of C++ `DrawDurIcon4Item`. Returns `None` when the item
    /// should not render an icon (empty or durability above the gold threshold).
    /// Otherwise returns `(icon_index, height, partition)` where `partition` is the
    /// number of rows drawn from the gold (healthy) sprite and `height - partition`
    /// is drawn from the red (critical) sprite.
    pub fn durability_icon(
        &self,
        durability: i32,
        item_type_override: Option<i32>,
        sprite_height: i32,
    ) -> Option<DurabilityIconParams> {
        const DUR_THRESHOLD_GOLD: i32 = 5;
        const DUR_THRESHOLD_RED: i32 = 2;

        if durability > DUR_THRESHOLD_GOLD {
            return None;
        }

        let icon_index = item_type_override.unwrap_or(0);
        let partition = if durability > DUR_THRESHOLD_RED {
            let current = durability - DUR_THRESHOLD_RED;
            (sprite_height * current) / (DUR_THRESHOLD_GOLD - DUR_THRESHOLD_RED)
        } else {
            0
        };

        Some(DurabilityIconParams {
            icon_index,
            height: sprite_height,
            partition,
        })
    }

    // ========================================================================
    // Pause overlay (RedBack)
    // ========================================================================

    /// Apply the pause translation table to a viewport pixel.
    ///
    /// Mirrors the per-pixel logic of C++ `RedBack`: pixels in hell levels that are
    /// below index 32 are left untouched, everything else is mapped through `trn`.
    pub fn red_back_pixel(&self, pixel: u8, trn: &[u8], is_hell_level: bool) -> u8 {
        if is_hell_level && pixel < 32 {
            pixel
        } else {
            trn.get(pixel as usize).copied().unwrap_or(pixel)
        }
    }

    // ========================================================================
    // Death text button label (DrawDeathText)
    // ========================================================================

    /// Localised button label for the death screen, based on the active control mode.
    ///
    /// Mirrors the `switch (ControlMode)` block inside C++ `DrawDeathText`.
    pub fn death_text_button_label(&self, control_mode: ControlModeKind) -> &'static str {
        match control_mode {
            ControlModeKind::KeyboardAndMouse => "ESC",
            ControlModeKind::Gamepad => "Start",
            ControlModeKind::VirtualGamepad => "Menu Button",
        }
    }

    // ========================================================================
    // Gold drop (OpenGoldDrop / CloseGoldDrop / GetGoldDropMax)
    // ========================================================================

    /// Open the gold-drop split dialog. Mirrors C++ `OpenGoldDrop(int8_t, int)`.
    pub fn open_gold_drop(&mut self, inv_index: i8, max: i32) {
        self.state.drop_gold_flag = true;
        self.state.gold_drop_inv_index = inv_index;
        self.state.gold_drop_text.clear();
        self.gold_drop_max = max;
    }

    /// Close the gold-drop split dialog. Mirrors C++ `CloseGoldDrop()`.
    pub fn close_gold_drop(&mut self) {
        if !self.state.drop_gold_flag {
            return;
        }
        self.state.drop_gold_flag = false;
        self.state.gold_drop_inv_index = -1;
        self.gold_drop_max = 0;
    }

    /// Maximum amount that can be dropped in the active split dialog.
    pub fn get_gold_drop_max(&self) -> i32 {
        self.gold_drop_max
    }

    /// Handle a key press while the gold-drop dialog is open.
    ///
    /// Mirrors C++ `control_drop_gold(SDL_Keycode)`. Returns the action to take; the
    /// caller is responsible for actually removing gold from the player inventory.
    pub fn control_drop_gold(&mut self, vkey: GoldDropKey) -> GoldDropAction {
        if !self.state.drop_gold_flag {
            return GoldDropAction::Close;
        }
        match vkey {
            GoldDropKey::Enter => {
                let value = self.state.gold_drop_text.trim().parse::<i32>().unwrap_or(0);
                if value != 0 {
                    self.close_gold_drop();
                    GoldDropAction::Split(value)
                } else {
                    self.close_gold_drop();
                    GoldDropAction::Close
                }
            }
            GoldDropKey::Escape => {
                self.close_gold_drop();
                GoldDropAction::Close
            }
            GoldDropKey::Other => GoldDropAction::None,
        }
    }

    // ========================================================================
    // Chat (TypeChatMessage / ResetChat / IsChatActive / CheckKeypress)
    // ========================================================================

    /// Begin composing a chat message. Mirrors C++ `TypeChatMessage()`.
    ///
    /// Returns `true` if chat is available (multiplayer) and the composer was opened.
    pub fn type_chat_message(&mut self) -> bool {
        if !self.is_chat_available() {
            return false;
        }
        self.state.chat_flag = true;
        self.state.talk_message.clear();
        self.state.talk_buttons_down = [false; 3];
        self.state.sgb_plr_talk_tbl = self.main_panel.size.height + PANEL_PADDING_HEIGHT;
        self.state.talk_save_index = self.state.next_talk_save;
        true
    }

    /// Close the chat composer. Mirrors C++ `ResetChat()`.
    pub fn reset_chat(&mut self) {
        self.state.chat_flag = false;
        self.state.sgb_plr_talk_tbl = 0;
    }

    /// Whether chat input is currently active. Mirrors C++ `IsChatActive()`.
    pub fn is_chat_active(&self) -> bool {
        self.is_chat_available() && self.state.chat_flag
    }

    /// Whether chat is available at all (multiplayer only). Mirrors C++ `IsChatAvailable()`.
    pub fn is_chat_available(&self) -> bool {
        self.total_main_panel_buttons == 8
    }

    /// Handle a key press while the chat composer is open.
    ///
    /// Mirrors C++ `CheckKeypress(SDL_Keycode)`. Returns `true` when the key was
    /// consumed; printable ASCII keys (`Space`..`Z`) are reported as consumed but
    /// do not change state here (the caller feeds them to the text input buffer).
    pub fn check_keypress(&mut self, vkey: ChatKey) -> bool {
        if !self.is_chat_active() {
            return false;
        }
        match vkey {
            ChatKey::Escape => {
                self.reset_chat();
                true
            }
            ChatKey::Return => {
                self.control_press_enter();
                true
            }
            ChatKey::Down => {
                self.control_up_down(1);
                true
            }
            ChatKey::Up => {
                self.control_up_down(-1);
                true
            }
            ChatKey::Printable(_) => true,
        }
    }

    /// Submit the current chat message. Mirrors C++ `ControlPressEnter()`.
    pub fn control_press_enter(&mut self) {
        if !self.state.talk_message.is_empty() {
            self.save_talk_message();
            self.state.talk_message.clear();
            self.state.talk_save_index = self.state.next_talk_save;
        }
        self.reset_chat();
    }

    /// Cycle through saved chat messages. Mirrors C++ `ControlUpDown(int)`.
    pub fn control_up_down(&mut self, direction: i32) {
        for _ in 0..8 {
            let next = ((self.state.talk_save_index as i32 + direction) & 7) as u8;
            self.state.talk_save_index = next;
            let saved = &self.state.talk_save[next as usize];
            if !saved.is_empty() {
                self.state.talk_message = saved.clone();
                return;
            }
        }
    }

    /// Persist the current message into the talk-save ring buffer.
    ///
    /// Mirrors the deduplication + most-recent-slot rotation logic of C++
    /// `ControlPressEnter()`. If the message already exists in a different slot it
    /// is moved to the most-recent slot instead of being duplicated.
    pub fn save_talk_message(&mut self) {
        let msg = self.state.talk_message.clone();
        if msg.is_empty() {
            return;
        }
        // Look for an existing copy of the message.
        let existing = self.state.talk_save.iter().position(|s| s == &msg);
        match existing {
            None => {
                self.state.talk_save[self.state.next_talk_save as usize] = msg;
                self.state.next_talk_save = (self.state.next_talk_save + 1) & 7;
            }
            Some(idx) => {
                let most_recent = (self.state.next_talk_save.wrapping_sub(1)) & 7;
                if idx != most_recent as usize {
                    // Swap the existing entry into the most-recent slot.
                    let most_recent_msg =
                        self.state.talk_save[most_recent as usize].clone();
                    self.state.talk_save[idx] = most_recent_msg;
                    self.state.talk_save[most_recent as usize] = msg;
                }
            }
        }
    }

    // ========================================================================
    // Mute (player voice) buttons (CheckMuteButton / CheckMuteButtonUp)
    // ========================================================================

    /// Hover detection for the mute (voice) button column.
    ///
    /// Mirrors C++ `CheckMuteButton()`. Returns the index of the row currently being
    /// hovered, or `None` if the chat panel is closed / the mouse is outside.
    pub fn check_mute_button(&mut self) -> Option<usize> {
        if !self.state.chat_flag {
            return None;
        }
        let mut buttons = MUTE_BUTTON_RECT;
        buttons.position = self.get_panel_position(UiPanels::Main, buttons.position);
        buttons.size.height =
            (self.mute_buttons * buttons.size.height) + ((self.mute_buttons - 1) * self.mute_button_padding);
        if !buttons.contains(self.mouse_position) {
            return None;
        }
        self.state.talk_buttons_down = [false; 3];
        let row = ((self.mouse_position.y - (69 + self.main_panel.position.y)) / 18) as usize;
        if row < self.state.talk_buttons_down.len() {
            self.state.talk_buttons_down[row] = true;
        }
        Some(row)
    }

    /// Release handler for the mute (voice) button column.
    ///
    /// Mirrors C++ `CheckMuteButtonUp()`. Returns the index of the whisper-list entry
    /// that should be toggled (skipping the local player), or `None`.
    pub fn check_mute_button_up(&mut self, player_count: usize, my_player_id: usize) -> Option<usize> {
        if !self.state.chat_flag {
            return None;
        }
        self.state.talk_buttons_down = [false; 3];

        let mut buttons = MUTE_BUTTON_RECT;
        buttons.position = self.get_panel_position(UiPanels::Main, buttons.position);
        buttons.size.height =
            (self.mute_buttons * buttons.size.height) + ((self.mute_buttons - 1) * self.mute_button_padding);
        if !buttons.contains(self.mouse_position) {
            return None;
        }

        let mut off = (self.mouse_position.y - buttons.position.y)
            / (MUTE_BUTTON_RECT.size.height + self.mute_button_padding);
        let mut player_id = 0;
        while player_id < player_count && off != -1 {
            if player_id != my_player_id {
                off -= 1;
            }
            player_id += 1;
        }
        if player_id > 0 && player_id <= player_count {
            Some(player_id - 1)
        } else {
            None
        }
    }

    // ========================================================================
    // Hotkey messages (DiabloHotkeyMsg)
    // ========================================================================

    /// Send a quick (hotkey) message slot.
    ///
    /// Mirrors C++ `DiabloHotkeyMsg(uint32_t)`. Returns the resolved message strings
    /// so the caller can forward them to the network layer; returns an empty vector
    /// when chat is unavailable.
    pub fn diablo_hotkey_msg(&self, slot: u32, messages: &[Vec<String>]) -> Vec<String> {
        if (slot as usize) >= messages.len() {
            return Vec::new();
        }
        if !self.is_chat_available() {
            return Vec::new();
        }
        messages[slot as usize].clone()
    }

    // ========================================================================
    // Info box string accumulation (AddInfoBoxString)
    // ========================================================================

    /// Append a line to the info-box string (or floating info string).
    ///
    /// Mirrors C++ `AddInfoBoxString(std::string_view, bool floatingBox)`.
    pub fn add_info_box_string(&mut self, text: &str, floating_box: bool) {
        if floating_box {
            if self.state.floating_info_string.is_empty() {
                self.state.floating_info_string = text.to_string();
            } else {
                self.state.floating_info_string.push('\n');
                self.state.floating_info_string.push_str(text);
            }
        } else {
            self.state.add_info_line(text);
        }
    }

    // ========================================================================
    // Stat point capping (CapStatPointsToAdd)
    // ========================================================================

    /// Cap the number of stat points that can be added to an attribute.
    ///
    /// Mirrors C++ `CapStatPointsToAdd(remaining, player, attribute)`.
    pub fn cap_stat_points_to_add(remaining: i32, base: i32, maximum: i32) -> i32 {
        let points_to_reach_cap = maximum - base;
        remaining.min(points_to_reach_cap)
    }

    // ========================================================================
    // Level-up button visibility (IsLevelUpButtonVisible)
    // ========================================================================

    /// Whether the level-up button should be rendered.
    ///
    /// Mirrors C++ `IsLevelUpButtonVisible()`. The button is hidden while a panel
    /// that would overlap it is open, while shopping/stashing, or when there are no
    /// unspent stat points.
    pub fn is_level_up_button_visible(
        &self,
        stat_points: i32,
        in_store: bool,
        quest_log_overlaps_panel: bool,
    ) -> bool {
        if self.state.spell_select_flag || self.state.char_flag || stat_points == 0 {
            return false;
        }
        if in_store || self.state.stash_open {
            return false;
        }
        if self.state.quest_log_open && quest_log_overlaps_panel {
            return false;
        }
        true
    }

    // ========================================================================
    // Floating info box geometry (ClampAboveOrBelow / GetHoverSpriteHeight)
    // ========================================================================

    /// Choose a vertical position for the floating info box so it stays on-screen.
    ///
    /// Mirrors C++ `ClampAboveOrBelow(anchorY, spriteH, boxH, pad, linePad)`.
    pub fn clamp_above_or_below(anchor_y: i32, sprite_h: i32, box_h: i32, pad: i32, line_pad: i32) -> i32 {
        let y_above = anchor_y - sprite_h - box_h - pad;
        let y_below = anchor_y + line_pad / 2 + pad;
        if y_above >= 0 {
            y_above
        } else {
            y_below
        }
    }

    // ========================================================================
    // Focus / character info (FocusOnCharInfo)
    // ========================================================================

    /// Determine which attribute button the cursor should snap to.
    ///
    /// Mirrors C++ `FocusOnCharInfo()`: returns the index of the first attribute
    /// that has not yet reached its maximum (so the caller can move the cursor to
    /// `CHAR_PANEL_BUTTON_RECTS[index]`), or `None` when there is nothing to level.
    pub fn focus_on_char_info(&self, base_values: &[i32; 4], max_values: &[i32; 4]) -> Option<usize> {
        for (i, attr) in CharacterAttribute::all().iter().enumerate() {
            let _ = attr;
            if base_values[i] >= max_values[i] {
                continue;
            }
            return Some(i);
        }
        None
    }

    // ========================================================================
    // Panel-info hover (CheckPanelInfo)
    // ========================================================================

    /// Compute the info-box description for the hovered panel element.
    ///
    /// Mirrors the bookkeeping side of C++ `CheckPanelInfo()` (it deliberately does
    /// not perform rendering). Returns the hovered button (if any) together with the
    /// info colour and the hotkey/extra description strings that should be shown.
    pub fn check_panel_info(&mut self) -> PanelInfoResult {
        self.state.main_panel_flag = false;
        self.state.info_string.clear();
        self.state.floating_info_string.clear();

        let total_buttons = if self.is_chat_available() { 8 } else { 6 };
        for i in 0..total_buttons {
            let mut button = MAIN_PANEL_BUTTON_RECTS[i];
            button.position = self.get_panel_position(UiPanels::Main, button.position);
            if !button.contains(self.mouse_position) {
                continue;
            }
            let button_id = PanelButtonId::all()[i];
            let desc = if i == 7 {
                // Friendly/attack toggle uses runtime state in C++; we report the
                // generic label here and let callers refine it.
                "Player attack"
            } else {
                button_id.description()
            };
            self.state.info_string = desc.to_string();
            let hotkey = button_id.hotkey();
            let mut extra_lines = Vec::new();
            if !hotkey.is_empty() {
                let line = format!("Hotkey: {}", hotkey);
                self.add_info_box_string(&line, false);
                extra_lines.push(line);
            }
            self.state.info_color = UiFlags::COLOR_WHITE;
            self.state.main_panel_flag = true;
            return PanelInfoResult {
                button: Some(button_id),
                info_color: self.state.info_color,
                info_string: self.state.info_string.clone(),
                extra_lines,
            };
        }

        // Spell-select button.
        let mut spell_button = SPELL_BUTTON_RECT;
        spell_button.position = self.get_panel_position(UiPanels::Main, spell_button.position);
        if !self.state.spell_select_flag && spell_button.contains(self.mouse_position) {
            self.state.info_string = "Select current spell button".to_string();
            self.state.info_color = UiFlags::COLOR_WHITE;
            self.state.main_panel_flag = true;
            self.add_info_box_string("Hotkey: 's'", false);
            return PanelInfoResult {
                button: None,
                info_color: self.state.info_color,
                info_string: self.state.info_string.clone(),
                extra_lines: vec!["Hotkey: 's'".to_string()],
            };
        }

        PanelInfoResult::default()
    }

    // ========================================================================
    // Main-panel button press dispatch (CheckMainPanelButtonUp)
    // ========================================================================

    /// Resolve the button that was released while still hovered.
    ///
    /// Mirrors the hit-testing portion of C++ `CheckMainPanelButtonUp()`. Returns the
    /// button (if any) whose action should fire; the caller performs the side effect.
    pub fn check_main_panel_button_up(&mut self) -> Option<PanelButtonId> {
        self.state.main_panel_button_down = false;
        for i in 0..8 {
            if !self.state.main_panel_buttons[i] {
                continue;
            }
            self.state.main_panel_buttons[i] = false;
            let mut button = MAIN_PANEL_BUTTON_RECTS[i];
            button.position = self.get_panel_position(UiPanels::Main, button.position);
            if button.contains(self.mouse_position) {
                return Some(PanelButtonId::all()[i]);
            }
        }
        None
    }

    /// Hit-test the two buttons that remain active when the player is dead.
    ///
    /// Mirrors C++ `CheckMainPanelButtonDead()` — only the menu and chat buttons are
    /// considered.
    pub fn check_main_panel_button_dead(&mut self) -> Option<PanelButtonId> {
        for &idx in &[PanelButtonId::Mainmenu as usize, PanelButtonId::Sendmsg as usize] {
            let mut button = MAIN_PANEL_BUTTON_RECTS[idx];
            button.position = self.get_panel_position(UiPanels::Main, button.position);
            if button.contains(self.mouse_position) {
                self.state.main_panel_buttons[idx] = true;
                self.state.main_panel_button_down = true;
                return Some(PanelButtonId::all()[idx]);
            }
        }
        None
    }

    // ========================================================================
    // Init / free / reset (InitMainPanel / FreeControlPan / ResetMainPanelButtons)
    // ========================================================================

    /// (Re)initialise panel state. Mirrors the state side of C++ `InitMainPanel()`.
    ///
    /// Resource loading is handled by the dedicated asset/resource modules, so this
    /// only resets the in-memory flags and clears the info strings.
    pub fn init_main_panel(&mut self) {
        self.reset_main_panel_buttons();
        self.state.chat_flag = false;
        self.state.main_panel_flag = false;
        self.state.level_button_down = false;
        self.state.char_panel_buttons = [false; 4];
        self.state.char_panel_button_active = false;
        self.state.info_string.clear();
        self.state.floating_info_string.clear();
        self.state.spell_select_flag = false;
        self.state.spellbook_tab = 0;
        self.state.spellbook_flag = false;
        self.close_char_panel_full();
        self.calculate_panel_areas();
    }

    /// Tear down all panel state. Mirrors C++ `FreeControlPan()`.
    pub fn free_control_pan(&mut self) {
        self.state = PanelState::new();
        self.reset_main_panel_buttons();
    }

    /// Reset all main-panel button flags. Mirrors C++ `ResetMainPanelButtons()`.
    pub fn reset_main_panel_buttons(&mut self) {
        self.state.main_panel_buttons = [false; 8];
        self.state.main_panel_button_down = false;
    }

    /// Close the character panel with the full inspect-player reset.
    ///
    /// Mirrors C++ `CloseCharPanel()` including the `InspectPlayer`/`InspectingFromPartyPanel`
    /// handling. The inspect-related side effects are reported back so the caller can
    /// apply them.
    pub fn close_char_panel_full(&mut self) -> CloseCharPanelResult {
        let was_open = self.state.char_flag;
        self.state.char_flag = false;
        CloseCharPanelResult {
            was_open,
            // In C++ this resets the inspect target; we surface a flag so the caller
            // can reset its own `inspect_player` pointer.
            reset_inspect: was_open,
        }
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
// Flask slice geometry (DrawFlaskUpper / DrawFlaskLower helpers)
// ============================================================================

/// Row layout for one half (upper or lower) of a flask.
///
/// Produced by [`ControlManager::flask_upper_params`] and
/// [`ControlManager::flask_lower_params`] so that the renderer can issue the two
/// blits (empty source first, filled source from `BottomBuffer` second) without
/// having to know the C++ clamping rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlaskSliceParams {
    /// Width of the flask slice in pixels.
    pub width: i32,
    /// Number of rows that should be drawn from the empty flask sprite.
    pub empty_rows: i32,
    /// Number of rows that should be drawn from the filled flask sprite.
    pub filled_rows: i32,
    /// Source Y coordinate for the empty section within the flask cel.
    pub src_empty_y: i32,
    /// Source Y coordinate for the filled section within the flask cel.
    pub src_filled_y: i32,
}

// ============================================================================
// Automap display type cycling (CycleAutomapType)
// ============================================================================

/// Automap display mode used by [`ControlManager::cycle_automap_type`].
///
/// Corresponds to the C++ `AutomapType` enum in the same declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomapCycleType {
    First,
    Opaque,
    Transparent,
    Minimap,
}

impl AutomapCycleType {
    /// First variant in the cycle (matches C++ `AutomapType::FIRST`).
    pub const fn first() -> Self {
        Self::First
    }

    /// Advance to the next variant, wrapping at the end.
    pub fn next(self) -> Self {
        match self {
            Self::First => Self::Opaque,
            Self::Opaque => Self::Transparent,
            Self::Transparent => Self::Minimap,
            Self::Minimap => Self::First,
        }
    }

    /// Number of variants in the cycle.
    pub const fn count() -> usize {
        4
    }
}

impl Default for AutomapCycleType {
    fn default() -> Self {
        Self::First
    }
}

// ============================================================================
// Durability icon parameters (DrawDurIcon4Item)
// ============================================================================

/// Resolved parameters for a single durability icon sprite.
///
/// Returned by [`ControlManager::durability_icon`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DurabilityIconParams {
    /// Index into the durability icon cel (item-type specific).
    pub icon_index: i32,
    /// Total sprite height in pixels.
    pub height: i32,
    /// Rows drawn from the gold (healthy) icon; `height - partition` from the red.
    pub partition: i32,
}

// ============================================================================
// Control mode + key enums (DrawDeathText / control_drop_gold / CheckKeypress)
// ============================================================================

/// Active control mode, used by the death-screen label helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlModeKind {
    KeyboardAndMouse,
    Gamepad,
    VirtualGamepad,
}

/// Keys handled by the gold-drop dialog. Mirrors the `switch (vkey)` cases in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldDropKey {
    Enter,
    Escape,
    Other,
}

/// Result of [`ControlManager::control_drop_gold`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldDropAction {
    /// Dialog closed, no gold moved.
    Close,
    /// Dialog closed and `amount` gold should be removed from the inventory.
    Split(i32),
    /// Key was ignored.
    None,
}

/// Keys handled by the chat composer. Mirrors the `switch (vkey)` cases in C++.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatKey {
    Escape,
    Return,
    Up,
    Down,
    /// Printable character in the `Space`..`Z` range.
    Printable(u8),
}

// ============================================================================
// Result records
// ============================================================================

/// Result of [`ControlManager::check_panel_info`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PanelInfoResult {
    /// Hovered main-panel button, if any.
    pub button: Option<PanelButtonId>,
    /// Colour flags to apply to the info string.
    pub info_color: UiFlags,
    /// Primary info string to display.
    pub info_string: String,
    /// Additional description lines (hotkey, spell details, ...).
    pub extra_lines: Vec<String>,
}

/// Result of [`ControlManager::close_char_panel_full`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CloseCharPanelResult {
    /// Whether the panel was open before being closed.
    pub was_open: bool,
    /// Whether the caller should reset its inspect-player pointer.
    pub reset_inspect: bool,
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

    // ========================================================================
    // New tests for ported functions
    // ========================================================================

    #[test]
    fn test_calculate_panel_areas() {
        let mut manager = ControlManager::new(800, 600);
        manager.calculate_panel_areas();

        // Main panel anchored to bottom-centre.
        assert_eq!(manager.main_panel.size.width, 640);
        assert_eq!(manager.main_panel.size.height, 128);
        assert_eq!(manager.main_panel.position.x, (800 - 640) / 2);
        assert_eq!(manager.main_panel.position.y, 600 - 128);

        // Side panels share the same y and are mirrored about the centre.
        assert_eq!(manager.left_panel.size, Size::new(320, 352));
        assert_eq!(manager.right_panel.size, Size::new(320, 352));
        assert_eq!(manager.left_panel.position.y, manager.right_panel.position.y);
        assert_eq!(
            manager.right_panel.position.x,
            800 - 320 - manager.left_panel.position.x
        );
    }

    #[test]
    fn test_viewport_height() {
        // Narrow screen: panel obscures part of the viewport.
        let mut manager = ControlManager::new(640, 480);
        manager.calculate_panel_areas();
        assert_eq!(manager.viewport_height(), 480 - 128);

        // Wide screen: viewport is the full height.
        let mut manager = ControlManager::new(1920, 1080);
        manager.calculate_panel_areas();
        assert_eq!(manager.viewport_height(), 1080);
    }

    #[test]
    fn test_do_auto_map() {
        let mut manager = ControlManager::new(640, 480);
        // Toggling flips the active state.
        assert!(manager.do_auto_map(false));
        assert!(!manager.do_auto_map(true));
    }

    #[test]
    fn test_cycle_automap_type() {
        let manager = ControlManager::new(640, 480);

        // Inactive automap is started with the first type.
        let (active, ty) = manager.cycle_automap_type(false, AutomapCycleType::First);
        assert!(active);
        assert_eq!(ty, AutomapCycleType::First);

        // Active automap advances through the variants.
        let (active, ty) = manager.cycle_automap_type(true, AutomapCycleType::First);
        assert!(active);
        assert_eq!(ty, AutomapCycleType::Opaque);

        let (active, ty) = manager.cycle_automap_type(true, AutomapCycleType::Minimap);
        // Wrapping back to First turns the automap off.
        assert!(!active);
        assert_eq!(ty, AutomapCycleType::First);
    }

    #[test]
    fn test_automap_cycle_type_next_wraps() {
        assert_eq!(AutomapCycleType::First.next(), AutomapCycleType::Opaque);
        assert_eq!(AutomapCycleType::Opaque.next(), AutomapCycleType::Transparent);
        assert_eq!(AutomapCycleType::Transparent.next(), AutomapCycleType::Minimap);
        assert_eq!(AutomapCycleType::Minimap.next(), AutomapCycleType::First);
        assert_eq!(AutomapCycleType::count(), 4);
    }

    #[test]
    fn test_flask_upper_params() {
        let manager = ControlManager::new(640, 480);

        // Full flask (fill 81): no empty rows, all 13 filled.
        let p = manager.flask_upper_params(81);
        assert_eq!(p.empty_rows, 0);
        assert_eq!(p.filled_rows, FLASK_TOP_RECT.size.height);
        assert_eq!(p.width, FLASK_TOP_RECT.size.width);

        // Empty flask (fill 0): all empty.
        let p = manager.flask_upper_params(0);
        assert_eq!(p.empty_rows, FLASK_TOP_RECT.size.height);
        assert_eq!(p.filled_rows, 0);

        // Overflow clamps to the rect height.
        let p = manager.flask_upper_params(200);
        assert_eq!(p.empty_rows, 0);
        assert_eq!(p.filled_rows, FLASK_TOP_RECT.size.height);
    }

    #[test]
    fn test_flask_lower_params() {
        let manager = ControlManager::new(640, 480);

        // Full flask (fill 80): all rows filled.
        let p = manager.flask_lower_params(80, true);
        assert_eq!(p.filled_rows, FLASK_BOTTOM_RECT.size.height);
        assert_eq!(p.empty_rows, 0);

        // Half full.
        let p = manager.flask_lower_params(35, true);
        assert_eq!(p.filled_rows, 35);
        assert_eq!(p.empty_rows, FLASK_BOTTOM_RECT.size.height - 35);

        // draw_filled_portion=false suppresses the filled rows.
        let p = manager.flask_lower_params(80, false);
        assert_eq!(p.filled_rows, 0);
    }

    #[test]
    fn test_flask_positions() {
        let manager = ControlManager::new(640, 480);
        let pos = manager.flask_upper_position(96);
        assert_eq!(pos.x, manager.main_panel.position.x + 96);
        assert_eq!(pos.y, manager.main_panel.position.y - FLASK_TOP_RECT.size.height);

        let pos = manager.flask_lower_position(96);
        assert_eq!(pos.x, manager.main_panel.position.x + 96);
        assert_eq!(pos.y, manager.main_panel.position.y);
    }

    #[test]
    fn test_flask_offsets_match_cpp() {
        assert_eq!(ControlManager::life_flask_upper_offset(), 107);
        assert_eq!(ControlManager::life_flask_lower_offset(), 96);
        assert_eq!(ControlManager::mana_flask_upper_offset(), 475);
        assert_eq!(ControlManager::mana_flask_lower_offset(), 464);
    }

    #[test]
    fn test_flask_value_color() {
        assert_eq!(ControlManager::flask_value_color(0, 100), UiFlags::COLOR_RED);
        assert_eq!(ControlManager::flask_value_color(100, 100), UiFlags::COLOR_GOLD);
        assert_eq!(ControlManager::flask_value_color(50, 100), UiFlags::COLOR_WHITE);
        assert_eq!(ControlManager::flask_value_color(-1, 100), UiFlags::COLOR_RED);
    }

    #[test]
    fn test_durability_icon() {
        let manager = ControlManager::new(640, 480);

        // Durability above gold threshold: no icon.
        assert!(manager.durability_icon(10, None, 32).is_none());

        // Critical durability: partition is 0 (all red).
        let p = manager.durability_icon(1, None, 32).unwrap();
        assert_eq!(p.partition, 0);
        assert_eq!(p.height, 32);

        // Mid-range durability with explicit icon override.
        let p = manager.durability_icon(4, Some(2), 32).unwrap();
        // current = 4 - 2 = 2; partition = 32 * 2 / (5 - 2) = 21 (rounded).
        assert_eq!(p.partition, (32 * 2) / 3);
        assert_eq!(p.icon_index, 2);
    }

    #[test]
    fn test_red_back_pixel() {
        let manager = ControlManager::new(640, 480);
        let mut trn = [0u8; 256];
        for (i, v) in trn.iter_mut().enumerate() {
            *v = ((i as u8).wrapping_add(1)) & 0xFF;
        }

        // Non-hell level: always mapped through trn.
        assert_eq!(manager.red_back_pixel(10, &trn, false), 11);

        // Hell level, pixel < 32: left untouched.
        assert_eq!(manager.red_back_pixel(10, &trn, true), 10);

        // Hell level, pixel >= 32: mapped through trn.
        assert_eq!(manager.red_back_pixel(40, &trn, true), 41);
    }

    #[test]
    fn test_death_text_button_label() {
        let manager = ControlManager::new(640, 480);
        assert_eq!(
            manager.death_text_button_label(ControlModeKind::KeyboardAndMouse),
            "ESC"
        );
        assert_eq!(
            manager.death_text_button_label(ControlModeKind::Gamepad),
            "Start"
        );
        assert_eq!(
            manager.death_text_button_label(ControlModeKind::VirtualGamepad),
            "Menu Button"
        );
    }

    #[test]
    fn test_gold_drop_lifecycle() {
        let mut manager = ControlManager::new(640, 480);

        manager.open_gold_drop(5, 1000);
        assert!(manager.state.drop_gold_flag);
        assert_eq!(manager.state.gold_drop_inv_index, 5);
        assert_eq!(manager.get_gold_drop_max(), 1000);

        // Enter with empty text: just closes.
        let action = manager.control_drop_gold(GoldDropKey::Enter);
        assert_eq!(action, GoldDropAction::Close);
        assert!(!manager.state.drop_gold_flag);

        // Enter with a numeric amount: splits.
        manager.open_gold_drop(2, 500);
        manager.state.gold_drop_text = "123".to_string();
        let action = manager.control_drop_gold(GoldDropKey::Enter);
        assert_eq!(action, GoldDropAction::Split(123));

        // Escape always closes.
        manager.open_gold_drop(1, 100);
        let action = manager.control_drop_gold(GoldDropKey::Escape);
        assert_eq!(action, GoldDropAction::Close);
        assert!(!manager.state.drop_gold_flag);

        // Other key is ignored.
        manager.open_gold_drop(1, 100);
        let action = manager.control_drop_gold(GoldDropKey::Other);
        assert_eq!(action, GoldDropAction::None);
        assert!(manager.state.drop_gold_flag);
        manager.close_gold_drop();
    }

    #[test]
    fn test_close_gold_drop_is_idempotent() {
        let mut manager = ControlManager::new(640, 480);
        manager.close_gold_drop(); // no-op when not open
        assert!(!manager.state.drop_gold_flag);
    }

    #[test]
    fn test_chat_composer_lifecycle() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();
        assert!(manager.is_chat_available());

        // Single player cannot open chat.
        manager.init_single_player();
        assert!(!manager.is_chat_available());
        assert!(!manager.type_chat_message());

        // Multiplayer opens and closes chat.
        manager.init_multiplayer();
        assert!(manager.type_chat_message());
        assert!(manager.state.chat_flag);
        assert!(manager.is_chat_active());

        manager.reset_chat();
        assert!(!manager.state.chat_flag);
        assert!(!manager.is_chat_active());
    }

    #[test]
    fn test_check_keypress() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();
        manager.type_chat_message();

        // Escape closes chat.
        assert!(manager.check_keypress(ChatKey::Escape));
        assert!(!manager.state.chat_flag);

        // When chat is closed, keys are not consumed (except the composer is inert).
        assert!(!manager.check_keypress(ChatKey::Escape));

        // Printable keys are consumed when chat is open.
        manager.type_chat_message();
        assert!(manager.check_keypress(ChatKey::Printable(b'A')));
        assert!(manager.check_keypress(ChatKey::Printable(b' ')));

        // Single player: never consumed.
        let mut sp = ControlManager::new(640, 480);
        sp.init_single_player();
        assert!(!sp.check_keypress(ChatKey::Return));
    }

    #[test]
    fn test_control_press_enter_clears_message() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();
        manager.type_chat_message();
        manager.state.talk_message = "hello".to_string();

        manager.control_press_enter();
        // Message is saved into the ring buffer and the composer is cleared.
        assert!(manager.state.talk_message.is_empty());
        assert!(!manager.state.chat_flag);
        assert!(manager.state.talk_save.iter().any(|s| s == "hello"));
    }

    #[test]
    fn test_save_talk_message_dedup() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();

        // First save goes to slot 0.
        manager.state.talk_message = "hello".to_string();
        manager.save_talk_message();
        assert_eq!(manager.state.talk_save[0], "hello");
        assert_eq!(manager.state.next_talk_save, 1);

        // Add a second distinct message to slot 1.
        manager.state.talk_message = "world".to_string();
        manager.save_talk_message();
        assert_eq!(manager.state.talk_save[1], "world");
        assert_eq!(manager.state.next_talk_save, 2);

        // Re-saving "hello" should move it to the most-recent slot (1) rather than
        // leaving a duplicate.
        manager.state.talk_message = "hello".to_string();
        manager.save_talk_message();
        assert_eq!(manager.state.talk_save[1], "hello");
        // Slot 0 keeps the displaced "world".
        assert_eq!(manager.state.talk_save[0], "world");
    }

    #[test]
    fn test_control_up_down() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();
        manager.state.talk_save[0] = "zero".to_string();
        manager.state.talk_save[1] = "one".to_string();
        manager.state.talk_save_index = 0;

        manager.control_up_down(1);
        assert_eq!(manager.state.talk_message, "one");
        assert_eq!(manager.state.talk_save_index, 1);

        manager.control_up_down(-1);
        assert_eq!(manager.state.talk_message, "zero");
        assert_eq!(manager.state.talk_save_index, 0);
    }

    #[test]
    fn test_check_mute_button() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();

        // Chat closed: no hover.
        manager.set_mouse_position(
            manager.main_panel.position.x + MUTE_BUTTON_RECT.position.x + 5,
            manager.main_panel.position.y + 70,
        );
        assert_eq!(manager.check_mute_button(), None);

        // Chat open: first row hovered.
        manager.type_chat_message();
        let row = manager.check_mute_button().unwrap();
        assert_eq!(row, 0);

        // Second row.
        manager.set_mouse_position(
            manager.main_panel.position.x + MUTE_BUTTON_RECT.position.x + 5,
            manager.main_panel.position.y + 70 + 18,
        );
        let row = manager.check_mute_button().unwrap();
        assert_eq!(row, 1);
    }

    #[test]
    fn test_check_mute_button_up() {
        let mut manager = ControlManager::new(640, 480);
        manager.init_multiplayer();
        manager.type_chat_message();

        // 4 players, local player id 0: hovering row 0 selects player index 1.
        manager.set_mouse_position(
            manager.main_panel.position.x + MUTE_BUTTON_RECT.position.x + 5,
            manager.main_panel.position.y + MUTE_BUTTON_RECT.position.y + 2,
        );
        let toggled = manager.check_mute_button_up(4, 0);
        assert_eq!(toggled, Some(1));

        // Mouse outside the column: nothing toggled.
        manager.set_mouse_position(0, 0);
        assert_eq!(manager.check_mute_button_up(4, 0), None);
    }

    #[test]
    fn test_diablo_hotkey_msg() {
        let manager = ControlManager::new(640, 480);
        manager; // borrow check
        let mut manager = ControlManager::new(640, 480);
        manager.init_single_player();
        let msgs = vec![vec!["hi".to_string(), "bye".to_string()]];
        // Single player: chat unavailable, returns empty.
        assert!(manager.diablo_hotkey_msg(0, &msgs).is_empty());

        manager.init_multiplayer();
        let result = manager.diablo_hotkey_msg(0, &msgs);
        assert_eq!(result, vec!["hi".to_string(), "bye".to_string()]);

        // Out-of-range slot: empty.
        assert!(manager.diablo_hotkey_msg(5, &msgs).is_empty());
    }

    #[test]
    fn test_add_info_box_string() {
        let mut manager = ControlManager::new(640, 480);

        manager.add_info_box_string("first", false);
        assert_eq!(manager.state.info_string, "first");

        manager.add_info_box_string("second", false);
        assert_eq!(manager.state.info_string, "first\nsecond");

        // Floating box string is independent.
        manager.add_info_box_string("float", true);
        assert_eq!(manager.state.floating_info_string, "float");
        manager.add_info_box_string("more", true);
        assert_eq!(manager.state.floating_info_string, "float\nmore");
    }

    #[test]
    fn test_cap_stat_points_to_add() {
        // Capped by remaining points.
        assert_eq!(ControlManager::cap_stat_points_to_add(3, 10, 50), 3);
        // Capped by the distance to the attribute maximum.
        assert_eq!(ControlManager::cap_stat_points_to_add(50, 45, 50), 5);
        // Already at cap.
        assert_eq!(ControlManager::cap_stat_points_to_add(5, 50, 50), 0);
    }

    #[test]
    fn test_is_level_up_button_visible() {
        let mut manager = ControlManager::new(640, 480);

        // Happy path: stat points available, no obstructing panels.
        assert!(manager.is_level_up_button_visible(5, false, false));

        // No stat points.
        assert!(!manager.is_level_up_button_visible(0, false, false));

        // Character panel open.
        manager.state.char_flag = true;
        assert!(!manager.is_level_up_button_visible(5, false, false));
        manager.state.char_flag = false;

        // In a store.
        assert!(!manager.is_level_up_button_visible(5, true, false));

        // Stash open.
        manager.state.stash_open = true;
        assert!(!manager.is_level_up_button_visible(5, false, false));
        manager.state.stash_open = false;

        // Quest log overlaps the panel.
        manager.state.quest_log_open = true;
        assert!(!manager.is_level_up_button_visible(5, false, true));
        manager.state.quest_log_open = false;
        // Quest log open but not overlapping: visible.
        manager.state.quest_log_open = true;
        assert!(manager.is_level_up_button_visible(5, false, false));
    }

    #[test]
    fn test_clamp_above_or_below() {
        // When the box fits above the anchor, prefer the above position.
        let y_above = ControlManager::clamp_above_or_below(100, 30, 40, 4, 6);
        assert_eq!(y_above, 100 - 30 - 40 - 4);

        // When above would be negative, fall back to below.
        let y_below = ControlManager::clamp_above_or_below(10, 30, 40, 4, 6);
        assert_eq!(y_below, 10 + 6 / 2 + 4);
    }

    #[test]
    fn test_focus_on_char_info() {
        let manager = ControlManager::new(640, 480);

        // First attribute already maxed: focuses the second.
        let idx = manager.focus_on_char_info(&[50, 10, 10, 10], &[50, 50, 50, 50]);
        assert_eq!(idx, Some(1));

        // All maxed: nothing to focus.
        let idx = manager.focus_on_char_info(&[50, 50, 50, 50], &[50, 50, 50, 50]);
        assert_eq!(idx, None);

        // None maxed: focuses the first.
        let idx = manager.focus_on_char_info(&[10, 10, 10, 10], &[50, 50, 50, 50]);
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn test_check_panel_info() {
        let mut manager = ControlManager::new(640, 480);

        // Hover the character button (first button).
        let mut btn = MAIN_PANEL_BUTTON_RECTS[0];
        btn.position = manager.get_panel_position(UiPanels::Main, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);

        let result = manager.check_panel_info();
        assert_eq!(result.button, Some(PanelButtonId::Charinfo));
        // C++ appends the hotkey line to InfoString via AddInfoBoxString.
        assert_eq!(result.info_string, "Character Information\nHotkey: 'c'");
        assert!(manager.state.main_panel_flag);
    }

    #[test]
    fn test_check_panel_info_spell_button() {
        let mut manager = ControlManager::new(640, 480);
        let mut btn = SPELL_BUTTON_RECT;
        btn.position = manager.get_panel_position(UiPanels::Main, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);

        let result = manager.check_panel_info();
        assert_eq!(result.button, None);
        // C++ appends the hotkey line to InfoString via AddInfoBoxString.
        assert_eq!(result.info_string, "Select current spell button\nHotkey: 's'");
        assert!(result.extra_lines.iter().any(|l| l.contains("'s'")));
    }

    #[test]
    fn test_check_panel_info_no_hover() {
        let mut manager = ControlManager::new(640, 480);
        manager.set_mouse_position(0, 0);

        let result = manager.check_panel_info();
        assert!(result.button.is_none());
        assert!(result.info_string.is_empty());
        assert!(!manager.state.main_panel_flag);
    }

    #[test]
    fn test_check_main_panel_button_up() {
        let mut manager = ControlManager::new(640, 480);

        // Press the character button.
        let mut btn = MAIN_PANEL_BUTTON_RECTS[0];
        btn.position = manager.get_panel_position(UiPanels::Main, btn.position);
        manager.state.main_panel_buttons[0] = true;
        manager.state.main_panel_button_down = true;
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);

        let clicked = manager.check_main_panel_button_up();
        assert_eq!(clicked, Some(PanelButtonId::Charinfo));
        assert!(!manager.state.main_panel_button_down);
        assert!(!manager.state.main_panel_buttons[0]);

        // Release outside the button: no action.
        manager.state.main_panel_buttons[0] = true;
        manager.state.main_panel_button_down = true;
        manager.set_mouse_position(0, 0);
        let clicked = manager.check_main_panel_button_up();
        assert_eq!(clicked, None);
        assert!(!manager.state.main_panel_buttons[0]);
    }

    #[test]
    fn test_check_main_panel_button_dead() {
        let mut manager = ControlManager::new(640, 480);

        // Hover the menu button (index 3).
        let mut btn = MAIN_PANEL_BUTTON_RECTS[PanelButtonId::Mainmenu as usize];
        btn.position = manager.get_panel_position(UiPanels::Main, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);
        let clicked = manager.check_main_panel_button_dead();
        assert_eq!(clicked, Some(PanelButtonId::Mainmenu));

        // Reset and hover the chat button (index 6).
        manager.reset_main_panel_buttons();
        let mut btn = MAIN_PANEL_BUTTON_RECTS[PanelButtonId::Sendmsg as usize];
        btn.position = manager.get_panel_position(UiPanels::Main, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);
        let clicked = manager.check_main_panel_button_dead();
        assert_eq!(clicked, Some(PanelButtonId::Sendmsg));

        // Hovering a non-active button (e.g. inventory) yields nothing.
        manager.reset_main_panel_buttons();
        let mut btn = MAIN_PANEL_BUTTON_RECTS[PanelButtonId::Inventory as usize];
        btn.position = manager.get_panel_position(UiPanels::Main, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);
        let clicked = manager.check_main_panel_button_dead();
        assert_eq!(clicked, None);
    }

    #[test]
    fn test_init_main_panel_resets_state() {
        let mut manager = ControlManager::new(640, 480);
        // Dirty the state.
        manager.state.char_flag = true;
        manager.state.spellbook_flag = true;
        manager.state.chat_flag = true;
        manager.state.main_panel_buttons[0] = true;
        manager.state.info_string = "stale".to_string();
        manager.state.spellbook_tab = 2;

        manager.init_main_panel();
        assert!(!manager.state.char_flag);
        assert!(!manager.state.spellbook_flag);
        assert!(!manager.state.chat_flag);
        assert!(!manager.state.main_panel_buttons[0]);
        assert!(manager.state.info_string.is_empty());
        assert_eq!(manager.state.spellbook_tab, 0);
    }

    #[test]
    fn test_free_control_pan() {
        let mut manager = ControlManager::new(640, 480);
        manager.state.char_flag = true;
        manager.state.main_panel_button_down = true;

        manager.free_control_pan();
        assert!(!manager.state.char_flag);
        assert!(!manager.state.main_panel_button_down);
    }

    #[test]
    fn test_reset_main_panel_buttons() {
        let mut manager = ControlManager::new(640, 480);
        for b in &mut manager.state.main_panel_buttons {
            *b = true;
        }
        manager.state.main_panel_button_down = true;

        manager.reset_main_panel_buttons();
        for &b in &manager.state.main_panel_buttons {
            assert!(!b);
        }
        assert!(!manager.state.main_panel_button_down);
    }

    #[test]
    fn test_close_char_panel_full() {
        let mut manager = ControlManager::new(640, 480);
        manager.state.char_flag = true;
        let result = manager.close_char_panel_full();
        assert!(result.was_open);
        assert!(result.reset_inspect);
        assert!(!manager.state.char_flag);

        // Closing again: was_open is false.
        let result = manager.close_char_panel_full();
        assert!(!result.was_open);
        assert!(!result.reset_inspect);
    }

    #[test]
    fn test_release_char_btns_add_all() {
        let mut manager = ControlManager::new(640, 480);

        // Hold the strength button and release while hovering it.
        manager.state.char_panel_buttons[0] = true;
        manager.state.char_panel_button_active = true;
        let mut btn = CHAR_PANEL_BUTTON_RECTS[0];
        btn.position = manager.get_panel_position(UiPanels::Character, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);

        let applied = manager.release_char_btns(true);
        assert_eq!(applied, vec![(CharacterAttribute::Strength, 1)]);
        assert!(!manager.state.char_panel_button_active);
        assert!(!manager.state.char_panel_buttons[0]);
    }

    #[test]
    fn test_release_char_buttons_default_signature() {
        let mut manager = ControlManager::new(640, 480);
        manager.state.char_panel_buttons[2] = true;
        manager.state.char_panel_button_active = true;
        let mut btn = CHAR_PANEL_BUTTON_RECTS[2];
        btn.position = manager.get_panel_position(UiPanels::Character, btn.position);
        manager.set_mouse_position(btn.position.x + 2, btn.position.y + 2);

        let applied = manager.release_char_buttons();
        assert_eq!(applied, vec![(CharacterAttribute::Dexterity, 1)]);
    }

    #[test]
    fn test_flask_slice_params_default() {
        let p = FlaskSliceParams::default();
        assert_eq!(p.width, 0);
        assert_eq!(p.empty_rows, 0);
        assert_eq!(p.filled_rows, 0);
    }

    #[test]
    fn test_gold_drop_action_eq() {
        assert_eq!(GoldDropAction::Close, GoldDropAction::Close);
        assert_eq!(GoldDropAction::Split(50), GoldDropAction::Split(50));
        assert_ne!(GoldDropAction::Split(50), GoldDropAction::Split(51));
        assert_ne!(GoldDropAction::Close, GoldDropAction::None);
    }

    #[test]
    fn test_panel_info_result_default() {
        let r = PanelInfoResult::default();
        assert!(r.button.is_none());
        assert_eq!(r.info_color, UiFlags::empty());
        assert!(r.info_string.is_empty());
        assert!(r.extra_lines.is_empty());
    }

    #[test]
    fn test_close_char_panel_result_default() {
        let r = CloseCharPanelResult::default();
        assert!(!r.was_open);
        assert!(!r.reset_inspect);
    }

    #[test]
    fn test_can_panels_cover_view_full() {
        // 640x480 exactly fits the panels + main panel height.
        let mut manager = ControlManager::new(640, 480);
        manager.calculate_panel_areas();
        assert!(manager.can_panels_cover_view_full());

        // Larger screen cannot be fully covered.
        let mut manager = ControlManager::new(1920, 1080);
        manager.calculate_panel_areas();
        assert!(!manager.can_panels_cover_view_full());
    }
}
