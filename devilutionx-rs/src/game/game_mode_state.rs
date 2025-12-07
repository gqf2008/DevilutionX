//! Game Mode State
//!
//! This module manages global game mode flags that determine which version
//! of the game is running and what features are available.
//!
//! # C++ Source Reference
//! - Source/game_mode.cpp
//! - Source/game_mode.hpp
//!
//! # Game Modes
//! - **Spawn/Shareware**: Limited demo version
//! - **Hellfire**: The expansion pack
//! - **Vanilla**: Original Diablo without modifications
//! - **Full Game**: Standard Diablo with all features

/// Game mode state container
///
/// Tracks the current game mode and variant flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameModeState {
    /// Game is currently running
    pub run_game: bool,
    /// Shareware/spawn (demo) mode
    pub is_spawn: bool,
    /// Hellfire expansion is active
    pub is_hellfire: bool,
    /// Vanilla mode (no QoL modifications)
    pub is_vanilla: bool,
    /// Force Hellfire mode
    pub force_hellfire: bool,
}

impl Default for GameModeState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameModeState {
    /// Creates a new game mode state with default values
    pub const fn new() -> Self {
        Self {
            run_game: false,
            is_spawn: false,
            is_hellfire: false,
            is_vanilla: false,
            force_hellfire: false,
        }
    }

    /// Creates state for the full Diablo game
    pub const fn diablo() -> Self {
        Self {
            run_game: false,
            is_spawn: false,
            is_hellfire: false,
            is_vanilla: false,
            force_hellfire: false,
        }
    }

    /// Creates state for the Hellfire expansion
    pub const fn hellfire() -> Self {
        Self {
            run_game: false,
            is_spawn: false,
            is_hellfire: true,
            is_vanilla: false,
            force_hellfire: false,
        }
    }

    /// Creates state for shareware/demo mode
    pub const fn shareware() -> Self {
        Self {
            run_game: false,
            is_spawn: true,
            is_hellfire: false,
            is_vanilla: false,
            force_hellfire: false,
        }
    }

    /// Creates state for vanilla mode (original behavior)
    pub const fn vanilla() -> Self {
        Self {
            run_game: false,
            is_spawn: false,
            is_hellfire: false,
            is_vanilla: true,
            force_hellfire: false,
        }
    }

    /// Checks if the game is the full version (not shareware)
    pub const fn is_full_game(&self) -> bool {
        !self.is_spawn
    }

    /// Checks if quality-of-life features are enabled
    pub const fn has_qol_features(&self) -> bool {
        !self.is_vanilla
    }

    /// Checks if Hellfire content is available
    pub const fn has_hellfire_content(&self) -> bool {
        self.is_hellfire || self.force_hellfire
    }

    /// Sets the shareware/spawn mode
    pub fn set_spawn(&mut self, spawn: bool) {
        self.is_spawn = spawn;
    }

    /// Sets the Hellfire mode
    pub fn set_hellfire(&mut self, hellfire: bool) {
        self.is_hellfire = hellfire;
    }

    /// Sets the vanilla mode
    pub fn set_vanilla(&mut self, vanilla: bool) {
        self.is_vanilla = vanilla;
    }

    /// Sets the game running state
    pub fn set_running(&mut self, running: bool) {
        self.run_game = running;
    }

    /// Starts the game
    pub fn start(&mut self) {
        self.run_game = true;
    }

    /// Stops the game
    pub fn stop(&mut self) {
        self.run_game = false;
    }
}

/// Global game mode state (thread-local for safety)
///
/// In actual game use, this would likely need synchronization
/// for multi-threaded access.
#[derive(Debug, Default)]
pub struct GlobalGameMode {
    state: GameModeState,
}

impl GlobalGameMode {
    /// Creates a new global game mode
    pub const fn new() -> Self {
        Self {
            state: GameModeState::new(),
        }
    }

    /// Gets the current state
    pub fn state(&self) -> &GameModeState {
        &self.state
    }

    /// Gets mutable access to the state
    pub fn state_mut(&mut self) -> &mut GameModeState {
        &mut self.state
    }

    /// Checks if running
    pub fn is_running(&self) -> bool {
        self.state.run_game
    }

    /// Checks if spawn mode
    pub fn is_spawn(&self) -> bool {
        self.state.is_spawn
    }

    /// Checks if hellfire
    pub fn is_hellfire(&self) -> bool {
        self.state.is_hellfire
    }

    /// Checks if vanilla
    pub fn is_vanilla(&self) -> bool {
        self.state.is_vanilla
    }
}

/// Game version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameVersion {
    /// Major version number
    pub major: u8,
    /// Minor version number
    pub minor: u8,
    /// Patch version number
    pub patch: u8,
    /// Version string
    pub string: &'static str,
}

impl GameVersion {
    /// DevilutionX version
    pub const DEVILUTIONX: Self = Self {
        major: 1,
        minor: 5,
        patch: 3,
        string: "1.5.3",
    };

    /// Original Diablo version
    pub const DIABLO_ORIGINAL: Self = Self {
        major: 1,
        minor: 0,
        patch: 9,
        string: "1.09b",
    };

    /// Hellfire version
    pub const HELLFIRE: Self = Self {
        major: 1,
        minor: 0,
        patch: 1,
        string: "1.01",
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = GameModeState::new();
        assert!(!state.run_game);
        assert!(!state.is_spawn);
        assert!(!state.is_hellfire);
        assert!(!state.is_vanilla);
        assert!(!state.force_hellfire);
    }

    #[test]
    fn test_diablo_mode() {
        let state = GameModeState::diablo();
        assert!(!state.is_spawn);
        assert!(!state.is_hellfire);
        assert!(state.is_full_game());
    }

    #[test]
    fn test_hellfire_mode() {
        let state = GameModeState::hellfire();
        assert!(state.is_hellfire);
        assert!(state.has_hellfire_content());
    }

    #[test]
    fn test_shareware_mode() {
        let state = GameModeState::shareware();
        assert!(state.is_spawn);
        assert!(!state.is_full_game());
    }

    #[test]
    fn test_vanilla_mode() {
        let state = GameModeState::vanilla();
        assert!(state.is_vanilla);
        assert!(!state.has_qol_features());
    }

    #[test]
    fn test_force_hellfire() {
        let mut state = GameModeState::new();
        state.force_hellfire = true;
        assert!(state.has_hellfire_content());
        assert!(!state.is_hellfire); // is_hellfire itself is false
    }

    #[test]
    fn test_setters() {
        let mut state = GameModeState::new();

        state.set_spawn(true);
        assert!(state.is_spawn);

        state.set_hellfire(true);
        assert!(state.is_hellfire);

        state.set_vanilla(true);
        assert!(state.is_vanilla);

        state.set_running(true);
        assert!(state.run_game);
    }

    #[test]
    fn test_start_stop() {
        let mut state = GameModeState::new();

        state.start();
        assert!(state.run_game);

        state.stop();
        assert!(!state.run_game);
    }

    #[test]
    fn test_global_game_mode() {
        let mut global = GlobalGameMode::new();

        assert!(!global.is_running());

        global.state_mut().start();
        assert!(global.is_running());

        global.state_mut().set_hellfire(true);
        assert!(global.is_hellfire());
    }

    #[test]
    fn test_game_version() {
        assert_eq!(GameVersion::DEVILUTIONX.string, "1.5.3");
        assert_eq!(GameVersion::DIABLO_ORIGINAL.string, "1.09b");
        assert_eq!(GameVersion::HELLFIRE.string, "1.01");
    }

    #[test]
    fn test_qol_features() {
        let state = GameModeState::diablo();
        assert!(state.has_qol_features()); // Non-vanilla has QoL

        let vanilla = GameModeState::vanilla();
        assert!(!vanilla.has_qol_features()); // Vanilla doesn't
    }
}
