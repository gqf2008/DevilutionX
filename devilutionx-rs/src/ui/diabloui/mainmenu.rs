//! Main Menu - Title Screen and Main Menu Navigation
//!
//! This module handles the main menu UI including:
//! - Title screen display
//! - Main menu options
//! - Credits and support screens
//! - Attract mode (demo playback)
//!
//! ## C++ Alignment
//!
//! - `DiabloUI/mainmenu.cpp` (135 lines)
//! - `DiabloUI/title.cpp`
//! - `DiabloUI/credits.cpp`

use super::ui_core::{UiContext, UiEvent, UiEventResult, UiKeyCode};
use super::ui_item::{UiArtText, UiArtTextButton, UiFlags, UiItem, UiList, UiListItem, UiRect};

/// Main menu selection options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MainMenuSelection {
    /// No selection yet
    None = 0,
    /// Single Player game
    SinglePlayer = 1,
    /// Multiplayer game
    Multiplayer = 2,
    /// Show support/donate screen
    ShowSupport = 3,
    /// Open settings menu
    Settings = 4,
    /// Show credits
    ShowCredits = 5,
    /// Exit the game
    ExitDiablo = 6,
    /// Attract mode (demo)
    AttractMode = 7,
}

impl MainMenuSelection {
    /// Get menu item text
    pub fn text(self) -> &'static str {
        match self {
            Self::None => "",
            Self::SinglePlayer => "Single Player",
            Self::Multiplayer => "Multi Player",
            Self::ShowSupport => "Support",
            Self::Settings => "Settings",
            Self::ShowCredits => "Credits",
            Self::ExitDiablo => "Exit Diablo",
            Self::AttractMode => "",
        }
    }

    /// Get selection from index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::SinglePlayer),
            1 => Some(Self::Multiplayer),
            2 => Some(Self::ShowSupport),
            3 => Some(Self::Settings),
            4 => Some(Self::ShowCredits),
            5 => Some(Self::ExitDiablo),
            _ => None,
        }
    }
}

/// Main menu result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuResult {
    /// No result yet
    None,
    /// User made a selection
    Selected(MainMenuSelection),
    /// Menu timed out (attract mode)
    Timeout,
}

/// Main menu state
#[derive(Debug)]
pub struct MainMenu {
    /// Player name for display
    player_name: String,
    /// Menu items
    items: Vec<UiItem>,
    /// Currently selected item
    selected_index: usize,
    /// Result
    result: MainMenuResult,
    /// Attract mode timeout (in ticks)
    attract_timeout: Option<u32>,
    /// Current tick count
    current_tick: u32,
    /// UI context
    ctx: UiContext,
}

impl MainMenu {
    /// Create a new main menu
    pub fn new(player_name: impl Into<String>) -> Self {
        let player_name = player_name.into();
        let mut menu = Self {
            player_name: player_name.clone(),
            items: Vec::new(),
            selected_index: 0,
            result: MainMenuResult::None,
            attract_timeout: None,
            current_tick: 0,
            ctx: UiContext::new(),
        };
        menu.build_items(&player_name);
        menu
    }

    /// Create main menu with attract mode timeout
    pub fn with_attract_timeout(player_name: impl Into<String>, timeout_ticks: u32) -> Self {
        let mut menu = Self::new(player_name);
        menu.attract_timeout = Some(timeout_ticks);
        menu
    }

    fn build_items(&mut self, player_name: &str) {
        self.items.clear();

        // Logo would be added by renderer

        // Player name title
        self.items.push(UiItem::ArtText(UiArtText::new(
            player_name,
            UiRect::new(0, 161, 640, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_42 | UiFlags::COLOR_GOLD,
        )));

        // Menu list
        let list_items = vec![
            UiListItem::new("Single Player", 0, UiFlags::NONE),
            UiListItem::new("Multi Player", 1, UiFlags::NONE),
            UiListItem::new("Support", 2, UiFlags::NONE),
            UiListItem::new("Settings", 3, UiFlags::NONE),
            UiListItem::new("Credits", 4, UiFlags::NONE),
            UiListItem::new("Exit Diablo", 5, UiFlags::NONE),
        ];

        let list = UiList::new(list_items, UiRect::new(64, 211, 510, 210), UiFlags::NONE, 6);
        self.items.push(UiItem::List(list));

        // Initialize context
        self.ctx.init_list(Vec::new(), 5, 6, true, 0);
    }

    /// Handle input event
    pub fn handle_event(&mut self, event: &UiEvent) -> MainMenuResult {
        if self.result != MainMenuResult::None {
            return self.result;
        }

        // Reset attract timer on any input
        self.current_tick = 0;

        match self.ctx.handle_event(event) {
            UiEventResult::Selected(index) => {
                if let Some(selection) = MainMenuSelection::from_index(index) {
                    self.result = MainMenuResult::Selected(selection);
                }
            }
            UiEventResult::Escape => {
                self.result = MainMenuResult::Selected(MainMenuSelection::ExitDiablo);
            }
            UiEventResult::Handled => {
                // Update selected index from context
                if let Some(state) = &self.ctx.list_state {
                    self.selected_index = state.selected_index;
                }
            }
            _ => {}
        }

        self.result
    }

    /// Update menu (for attract mode timeout)
    pub fn update(&mut self, delta_ticks: u32) -> MainMenuResult {
        if self.result != MainMenuResult::None {
            return self.result;
        }

        if let Some(timeout) = self.attract_timeout {
            self.current_tick += delta_ticks;
            if self.current_tick >= timeout {
                self.result = MainMenuResult::Timeout;
            }
        }

        self.result
    }

    /// Get current selection index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get player name
    pub fn player_name(&self) -> &str {
        &self.player_name
    }

    /// Get menu items for rendering
    pub fn items(&self) -> &[UiItem] {
        &self.items
    }

    /// Check if result is ready
    pub fn is_done(&self) -> bool {
        self.result != MainMenuResult::None
    }

    /// Get result
    pub fn get_result(&self) -> MainMenuResult {
        self.result
    }

    /// Move selection up/down
    pub fn move_selection(&mut self, delta: i32) {
        let len = 6; // Number of menu items
        if delta > 0 {
            self.selected_index = (self.selected_index + 1) % len;
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = len - 1;
        }
    }

    /// Set selection directly by index
    pub fn set_selection(&mut self, index: usize) {
        let len = 6; // Number of menu items
        if index < len {
            self.selected_index = index;
        }
    }

    /// Get currently selected option
    pub fn get_selected(&self) -> MainMenuSelection {
        MainMenuSelection::from_index(self.selected_index).unwrap_or(MainMenuSelection::None)
    }

    /// Render menu to SDL canvas
    pub fn render(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> Result<(), String> {
        use sdl2::pixels::Color;
        use sdl2::rect::Rect as SdlRect;

        canvas.set_draw_color(Color::RGB(20, 20, 40));
        canvas.clear();

        // Title
        let title_y = 100;
        // Logo/title would be rendered here if we had the assets loaded

        // Menu items
        let menu_texts = ["Single Player", "Multi Player", "Settings", "Credits", "Exit Diablo"];
        for (i, text) in menu_texts.iter().enumerate() {
            let y = 200 + i as i32 * 50;
            let x = 150;

            if i == self.selected_index {
                canvas.set_draw_color(Color::RGB(80, 60, 100));
                let _ = canvas.fill_rect(SdlRect::new(x - 10, y - 5, 350, 40));
            }

            // Simple text rendering would go here
            // For now just show selection indicator
            if i == self.selected_index {
                canvas.set_draw_color(Color::RGB(255, 200, 50));
                let _ = canvas.fill_rect(SdlRect::new(x - 30, y + 10, 20, 20));
            }
        }

        // Product name at bottom
        canvas.present();
        Ok(())
    }

    /// Reset menu to initial state
    pub fn reset(&mut self) {
        self.result = MainMenuResult::None;
        self.selected_index = 0;
        self.current_tick = 0;
        self.ctx.init_list(Vec::new(), 4, 5, true, 0);
    }
}

/// Title screen manager
#[derive(Debug)]
pub struct TitleScreen {
    /// Whether to show title animation
    #[allow(dead_code)]
    show_animation: bool,
    /// Animation frame
    animation_frame: u32,
    /// Total animation frames
    total_frames: u32,
    /// Whether animation is complete
    animation_complete: bool,
}

impl TitleScreen {
    /// Create title screen with animation
    pub fn new(animated: bool) -> Self {
        Self {
            show_animation: animated,
            animation_frame: 0,
            total_frames: if animated { 30 } else { 0 },
            animation_complete: !animated,
        }
    }

    /// Create static title screen (no animation)
    pub fn static_screen() -> Self {
        Self::new(false)
    }

    /// Update animation
    pub fn update(&mut self) -> bool {
        if self.animation_complete {
            return true;
        }

        self.animation_frame += 1;
        if self.animation_frame >= self.total_frames {
            self.animation_complete = true;
        }

        self.animation_complete
    }

    /// Skip animation
    pub fn skip_animation(&mut self) {
        self.animation_complete = true;
        self.animation_frame = self.total_frames;
    }

    /// Handle input (skip on any key/click)
    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::KeyDown { .. } | UiEvent::MouseDown { .. } => {
                if !self.animation_complete {
                    self.skip_animation();
                    false // Don't proceed yet
                } else {
                    true // Animation done, proceed
                }
            }
            _ => false,
        }
    }

    /// Get animation progress (0.0 - 1.0)
    pub fn animation_progress(&self) -> f32 {
        if self.total_frames == 0 {
            1.0
        } else {
            self.animation_frame as f32 / self.total_frames as f32
        }
    }

    /// Is animation complete?
    pub fn is_complete(&self) -> bool {
        self.animation_complete
    }
}

/// Credits screen manager
#[derive(Debug)]
pub struct CreditsScreen {
    /// Credit lines
    lines: Vec<String>,
    /// Current scroll position
    scroll_position: f32,
    /// Scroll speed (lines per second)
    scroll_speed: f32,
    /// Whether credits are done scrolling
    complete: bool,
}

impl CreditsScreen {
    /// Create credits screen
    pub fn new() -> Self {
        Self {
            lines: Self::default_credits(),
            scroll_position: 0.0,
            scroll_speed: 30.0, // Pixels per second
            complete: false,
        }
    }

    fn default_credits() -> Vec<String> {
        vec![
            "DevilutionX".to_string(),
            "".to_string(),
            "A Modern Diablo Port".to_string(),
            "".to_string(),
            "Original Game by Blizzard North".to_string(),
            "".to_string(),
            "Reverse Engineering by".to_string(),
            "Sanctuary Team".to_string(),
            "".to_string(),
            "Rust Port by".to_string(),
            "DevilutionX-RS Team".to_string(),
            "".to_string(),
            "Thanks for playing!".to_string(),
        ]
    }

    /// Set custom credits
    pub fn with_lines(lines: Vec<String>) -> Self {
        Self {
            lines,
            scroll_position: 0.0,
            scroll_speed: 30.0,
            complete: false,
        }
    }

    /// Update scroll position
    pub fn update(&mut self, delta_seconds: f32) {
        if self.complete {
            return;
        }

        self.scroll_position += self.scroll_speed * delta_seconds;

        // Check if all credits have scrolled past
        let total_height = self.lines.len() as f32 * 20.0; // Assume 20px per line
        if self.scroll_position > total_height + 480.0 {
            // Screen height
            self.complete = true;
        }
    }

    /// Handle input (skip on escape)
    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        if let UiEvent::KeyDown { keycode, .. } = event {
            if *keycode == UiKeyCode::Escape {
                self.complete = true;
                return true;
            }
        }
        false
    }

    /// Get visible lines for rendering
    pub fn visible_lines(&self) -> impl Iterator<Item = (f32, &str)> {
        let line_height = 20.0;
        let screen_height = 480.0;

        self.lines.iter().enumerate().filter_map(move |(i, line)| {
            let y = i as f32 * line_height - self.scroll_position + screen_height;
            if y > -line_height && y < screen_height + line_height {
                Some((y, line.as_str()))
            } else {
                None
            }
        })
    }

    /// Is credits complete?
    pub fn is_complete(&self) -> bool {
        self.complete
    }
}

impl Default for CreditsScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_menu_selection() {
        assert_eq!(MainMenuSelection::SinglePlayer.text(), "Single Player");
        assert_eq!(
            MainMenuSelection::from_index(0),
            Some(MainMenuSelection::SinglePlayer)
        );
        assert_eq!(
            MainMenuSelection::from_index(4),
            Some(MainMenuSelection::ExitDiablo)
        );
        assert_eq!(MainMenuSelection::from_index(10), None);
    }

    #[test]
    fn test_main_menu() {
        let mut menu = MainMenu::new("TestPlayer");
        assert_eq!(menu.player_name(), "TestPlayer");
        assert!(!menu.is_done());

        // Simulate pressing Enter on first item
        let event = UiEvent::KeyDown {
            keycode: UiKeyCode::Return,
            character: None,
        };
        menu.handle_event(&event);
        assert!(menu.is_done());

        if let MainMenuResult::Selected(selection) = menu.get_result() {
            assert_eq!(selection, MainMenuSelection::SinglePlayer);
        } else {
            panic!("Expected Selected result");
        }
    }

    #[test]
    fn test_main_menu_attract_timeout() {
        let mut menu = MainMenu::with_attract_timeout("Player", 100);

        // Update less than timeout
        menu.update(50);
        assert!(!menu.is_done());

        // Update past timeout
        menu.update(60);
        assert!(menu.is_done());
        assert_eq!(menu.get_result(), MainMenuResult::Timeout);
    }

    #[test]
    fn test_title_screen() {
        let mut title = TitleScreen::new(true);
        assert!(!title.is_complete());

        // Skip animation
        title.skip_animation();
        assert!(title.is_complete());
    }

    #[test]
    fn test_credits_screen() {
        let mut credits = CreditsScreen::new();
        assert!(!credits.is_complete());

        // Simulate escape
        let event = UiEvent::KeyDown {
            keycode: UiKeyCode::Escape,
            character: None,
        };
        credits.handle_event(&event);
        assert!(credits.is_complete());
    }

    #[test]
    fn test_title_screen_static() {
        let title = TitleScreen::static_screen();
        assert!(title.is_complete());
        assert_eq!(title.animation_progress(), 1.0);
    }
}
