//! Main Menu System
//!
//! Ported from Source/menu.cpp (173 lines)
//!
//! This module handles the main menu interface including:
//! - Single player game initialization
//! - Multiplayer game initialization
//! - Hero selection dialog
//! - Intro movie playback
//! - Credits and settings menus
//!
//! ## C++ Reference
//! - Source/menu.cpp: Main menu implementation
//! - Source/menu.h: Function declarations
//!
//! ## Key Functions
//! - `mainmenu_loop()` - Main menu event loop
//! - `mainmenu_select_hero_dialog()` - Hero selection UI
//! - `RefreshMusic()` - Background music management

use std::time::Duration;

// ============================================================================
// Constants
// ============================================================================

/// Current save number for the active player
pub static mut G_SAVE_NUMBER: u32 = 0;

/// Button press sound delay (ms)
pub const BUTTON_SOUND_DELAY_MS: u64 = 350;

// ============================================================================
// Music Track System
// ============================================================================

/// Music track identifiers
///
/// C++ Reference: `_music_id` enum in Source/effects.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum MusicId {
    #[default]
    None = -1,
    /// Main menu / town music
    Intro = 0,
    /// Cathedral levels (L1-L4)
    Cathedral = 1,
    /// Catacombs levels (L5-L8)
    Catacombs = 2,
    /// Caves levels (L9-L12)
    Caves = 3,
    /// Hell levels (L13-L16)
    Hell = 4,
    /// Hellfire: Nest levels
    Nest = 5,
    /// Hellfire: Crypt levels
    Crypt = 6,
}

/// Global current music track
static mut SGN_MUSIC_TRACK: MusicId = MusicId::Intro;

/// Get the next music track in sequence
///
/// C++ Reference: `NextTrack()` in menu.cpp
fn next_track(is_spawn: bool, is_hellfire: bool) -> MusicId {
    unsafe {
        if is_spawn {
            return MusicId::Intro;
        }

        match SGN_MUSIC_TRACK {
            MusicId::Intro => MusicId::Catacombs,
            MusicId::Catacombs => MusicId::Caves,
            MusicId::Caves => MusicId::Hell,
            MusicId::Hell => {
                if is_hellfire {
                    MusicId::Nest
                } else {
                    MusicId::Intro
                }
            }
            MusicId::Nest => {
                if is_hellfire {
                    MusicId::Crypt
                } else {
                    MusicId::Intro
                }
            }
            _ => MusicId::Intro,
        }
    }
}

/// Refresh background music
///
/// C++ Reference: `RefreshMusic()` in menu.cpp
pub fn refresh_music(is_spawn: bool, is_hellfire: bool) {
    let track = next_track(is_spawn, is_hellfire);
    music_start(track);
}

// ============================================================================
// Hero Selection Types
// ============================================================================

/// Hero selection dialog result
///
/// C++ Reference: `_selhero_selections` enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum SelHeroSelection {
    #[default]
    /// Create new dungeon game
    NewDungeon = 0,
    /// Continue existing game
    Continue = 1,
    /// Connect to multiplayer
    Connect = 2,
    /// Go back to previous menu
    Previous = 3,
}

/// Main menu selection options
///
/// C++ Reference: `_mainmenu_selections` enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum MainMenuSelection {
    #[default]
    /// No selection
    None = 0,
    /// Start single player game
    SinglePlayer = 1,
    /// Start multiplayer game
    Multiplayer = 2,
    /// Play intro/attract mode
    AttractMode = 3,
    /// Show credits
    ShowCredits = 4,
    /// Show support info
    ShowSupport = 5,
    /// Exit game
    ExitDiablo = 6,
    /// Open settings menu
    Settings = 7,
}

// ============================================================================
// Game Data
// ============================================================================

/// Game difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Difficulty {
    #[default]
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

/// Game session data passed to menu system
///
/// C++ Reference: `GameData` struct
#[derive(Debug, Clone, Default)]
pub struct GameData {
    /// Selected difficulty
    pub difficulty: Difficulty,
    /// Whether this is a multiplayer game
    pub is_multiplayer: bool,
    /// Whether to load an existing save
    pub load_game: bool,
}

// ============================================================================
// Menu State
// ============================================================================

/// Main menu state manager
#[derive(Debug, Clone)]
pub struct MainMenu {
    /// Whether the menu loop is done
    pub done: bool,
    /// Current game data
    pub game_data: GameData,
    /// Product name displayed in title
    pub product_name: String,
    /// Is this Hellfire expansion
    pub is_hellfire: bool,
    /// Is this Spawn (demo) version
    pub is_spawn: bool,
    /// Is game window active
    pub is_active: bool,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl MainMenu {
    /// Create a new main menu
    pub fn new() -> Self {
        Self {
            done: false,
            game_data: GameData::default(),
            product_name: "DevilutionX".to_string(),
            is_hellfire: false,
            is_spawn: false,
            is_active: true,
        }
    }

    /// Create main menu with configuration
    pub fn with_config(product_name: &str, is_hellfire: bool, is_spawn: bool) -> Self {
        Self {
            done: false,
            game_data: GameData::default(),
            product_name: product_name.to_string(),
            is_hellfire,
            is_spawn,
            is_active: true,
        }
    }

    /// Initialize single player menu
    ///
    /// C++ Reference: `InitSinglePlayerMenu()` in menu.cpp
    pub fn init_single_player(&mut self) -> bool {
        self.game_data.is_multiplayer = false;
        self.init_menu(SelHeroSelection::NewDungeon)
    }

    /// Initialize multiplayer menu
    ///
    /// C++ Reference: `InitMultiPlayerMenu()` in menu.cpp
    pub fn init_multiplayer(&mut self) -> bool {
        self.game_data.is_multiplayer = true;
        self.init_menu(SelHeroSelection::Connect)
    }

    /// Initialize menu with selection type
    ///
    /// C++ Reference: `InitMenu()` in menu.cpp
    fn init_menu(&mut self, selection: SelHeroSelection) -> bool {
        if selection == SelHeroSelection::Previous {
            return true;
        }

        let success = self.start_game(
            selection != SelHeroSelection::Continue,
            selection != SelHeroSelection::Connect,
        );

        if success {
            refresh_music(self.is_spawn, self.is_hellfire);
        }

        success
    }

    /// Start the game
    ///
    /// Placeholder - actual implementation would initialize game systems
    fn start_game(&self, _new_game: bool, _single_player: bool) -> bool {
        // TODO: Implement actual game start logic
        // This would call into the game initialization system
        true
    }

    /// Play intro movie
    ///
    /// C++ Reference: `PlayIntro()` in menu.cpp
    pub fn play_intro(&self) {
        music_stop();

        let movie_path = if self.is_hellfire {
            "gendata\\Hellfire.smk"
        } else {
            "gendata\\diablo1.smk"
        };

        play_movie(movie_path, true);
        refresh_music(self.is_spawn, self.is_hellfire);
    }

    /// Check if intro is available
    pub fn have_intro(&self) -> bool {
        // TODO: Check if intro movie file exists
        true
    }

    /// Handle main menu selection
    ///
    /// C++ Reference: `mainmenu_loop()` switch statement
    pub fn handle_selection(&mut self, selection: MainMenuSelection) -> bool {
        match selection {
            MainMenuSelection::None => {
                // No action
            }
            MainMenuSelection::SinglePlayer => {
                if !self.init_single_player() {
                    self.done = true;
                }
            }
            MainMenuSelection::Multiplayer => {
                if !self.init_multiplayer() {
                    self.done = true;
                }
            }
            MainMenuSelection::AttractMode => {
                if self.is_spawn && !self.have_intro() {
                    // Skip for spawn without intro
                } else if self.is_active {
                    self.play_intro();
                }
            }
            MainMenuSelection::ShowCredits => {
                self.show_credits();
            }
            MainMenuSelection::ShowSupport => {
                self.show_support();
            }
            MainMenuSelection::ExitDiablo => {
                self.wait_for_button_sound();
                self.done = true;
            }
            MainMenuSelection::Settings => {
                self.show_settings();
            }
        }

        self.done
    }

    /// Wait for button sound to finish
    ///
    /// C++ Reference: `mainmenu_wait_for_button_sound()` in menu.cpp
    pub fn wait_for_button_sound(&self) {
        // In actual implementation, this would:
        // 1. Clear the UI surface
        // 2. Fade in
        // 3. Delay for button sound
        std::thread::sleep(Duration::from_millis(BUTTON_SOUND_DELAY_MS));
    }

    /// Show credits dialog
    fn show_credits(&self) {
        // TODO: Implement credits UI
    }

    /// Show support dialog
    fn show_support(&self) {
        // TODO: Implement support UI
    }

    /// Show settings menu
    fn show_settings(&self) {
        // TODO: Implement settings UI
    }

    /// Run the main menu loop
    ///
    /// C++ Reference: `mainmenu_loop()` in menu.cpp
    pub fn run(&mut self) {
        refresh_music(self.is_spawn, self.is_hellfire);
        self.done = false;

        while !self.done {
            // In actual implementation, this would:
            // 1. Display main menu UI
            // 2. Wait for user selection
            // 3. Handle the selection

            // For now, just exit
            self.done = true;
        }

        music_stop();
    }
}

/// Hero selection dialog
///
/// C++ Reference: `mainmenu_select_hero_dialog()` in menu.cpp
pub fn select_hero_dialog(game_data: &mut GameData, is_multiplayer: bool, is_hellfire: bool) -> bool {
    unsafe {
        // Get save number from options (would be persisted)
        let save_number = G_SAVE_NUMBER;

        let mut selection = SelHeroSelection::NewDungeon;

        if !is_multiplayer {
            // Single player hero selection
            // TODO: Call UI hero selection dialog
            // UiSelHeroSingDialog(...)
            game_data.load_game = selection == SelHeroSelection::Continue;
        } else {
            // Multiplayer hero selection
            // TODO: Call UI hero selection dialog
            // UiSelHeroMultDialog(...)
        }

        if selection == SelHeroSelection::Previous {
            return false;
        }

        // Save the selected save number
        G_SAVE_NUMBER = save_number;
    }

    true
}

// ============================================================================
// Stub Functions - To be implemented with actual systems
// ============================================================================

/// Start playing music track
fn music_start(_track: MusicId) {
    // TODO: Implement with audio system
}

/// Stop current music
fn music_stop() {
    // TODO: Implement with audio system
}

/// Play a movie file
fn play_movie(_path: &str, _loop_mode: bool) {
    // TODO: Implement with movie playback system
}

// ============================================================================
// Public API
// ============================================================================

/// Run the main menu and return game data
pub fn mainmenu_loop(product_name: &str, is_hellfire: bool, is_spawn: bool) -> Option<GameData> {
    let mut menu = MainMenu::with_config(product_name, is_hellfire, is_spawn);
    menu.run();

    if menu.done {
        Some(menu.game_data)
    } else {
        None
    }
}

/// Get the current save number
pub fn get_save_number() -> u32 {
    unsafe { G_SAVE_NUMBER }
}

/// Set the current save number
pub fn set_save_number(save_num: u32) {
    unsafe { G_SAVE_NUMBER = save_num }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_music_id_default() {
        let track = MusicId::default();
        assert_eq!(track, MusicId::None);
    }

    #[test]
    fn test_next_track_spawn() {
        // Spawn version always returns intro
        let track = next_track(true, false);
        assert_eq!(track, MusicId::Intro);
    }

    #[test]
    fn test_sel_hero_selection_default() {
        let sel = SelHeroSelection::default();
        assert_eq!(sel, SelHeroSelection::NewDungeon);
    }

    #[test]
    fn test_main_menu_selection_default() {
        let sel = MainMenuSelection::default();
        assert_eq!(sel, MainMenuSelection::None);
    }

    #[test]
    fn test_difficulty_default() {
        let diff = Difficulty::default();
        assert_eq!(diff, Difficulty::Normal);
    }

    #[test]
    fn test_game_data_default() {
        let data = GameData::default();
        assert_eq!(data.difficulty, Difficulty::Normal);
        assert!(!data.is_multiplayer);
        assert!(!data.load_game);
    }

    #[test]
    fn test_main_menu_new() {
        let menu = MainMenu::new();
        assert!(!menu.done);
        assert!(!menu.is_hellfire);
        assert!(!menu.is_spawn);
        assert!(menu.is_active);
    }

    #[test]
    fn test_main_menu_with_config() {
        let menu = MainMenu::with_config("Test Game", true, false);
        assert_eq!(menu.product_name, "Test Game");
        assert!(menu.is_hellfire);
        assert!(!menu.is_spawn);
    }

    #[test]
    fn test_init_single_player() {
        let mut menu = MainMenu::new();
        let result = menu.init_single_player();
        assert!(result);
        assert!(!menu.game_data.is_multiplayer);
    }

    #[test]
    fn test_init_multiplayer() {
        let mut menu = MainMenu::new();
        let result = menu.init_multiplayer();
        assert!(result);
        assert!(menu.game_data.is_multiplayer);
    }

    #[test]
    fn test_handle_selection_exit() {
        let mut menu = MainMenu::new();
        let done = menu.handle_selection(MainMenuSelection::ExitDiablo);
        assert!(done);
        assert!(menu.done);
    }

    #[test]
    fn test_handle_selection_none() {
        let mut menu = MainMenu::new();
        let done = menu.handle_selection(MainMenuSelection::None);
        assert!(!done);
        assert!(!menu.done);
    }

    #[test]
    fn test_save_number() {
        set_save_number(42);
        assert_eq!(get_save_number(), 42);
        set_save_number(0);
    }

    #[test]
    fn test_mainmenu_loop() {
        let result = mainmenu_loop("Test", false, false);
        // Menu immediately exits in stub implementation
        assert!(result.is_some());
    }

    #[test]
    fn test_select_hero_dialog() {
        let mut game_data = GameData::default();
        let result = select_hero_dialog(&mut game_data, false, false);
        assert!(result);
    }
}
