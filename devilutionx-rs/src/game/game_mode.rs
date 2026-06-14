//! Game Mode - Game mode flags and state
//!
//! C++ Reference: Source/game_mode.cpp, Source/game_mode.hpp
//!
//! Contains global game mode flags.

use std::sync::atomic::{AtomicBool, Ordering};

/// Are we in-game? If false, we're in the main menu.
///
/// C++ Reference: `gbRunGame`
static GB_RUN_GAME: AtomicBool = AtomicBool::new(false);

/// Indicate if we only have access to demo data (Shareware/Spawn version)
///
/// C++ Reference: `gbIsSpawn`
static GB_IS_SPAWN: AtomicBool = AtomicBool::new(false);

/// Indicate if we have loaded the Hellfire expansion data
///
/// C++ Reference: `gbIsHellfire`
static GB_IS_HELLFIRE: AtomicBool = AtomicBool::new(false);

/// Indicate if we want vanilla savefiles
///
/// C++ Reference: `gbVanilla`
static GB_VANILLA: AtomicBool = AtomicBool::new(false);

/// Whether the Hellfire mode is required (forced)
///
/// C++ Reference: `forceHellfire`
static FORCE_HELLFIRE: AtomicBool = AtomicBool::new(false);

/// Check if we are in-game
pub fn is_run_game() -> bool {
    GB_RUN_GAME.load(Ordering::SeqCst)
}

/// Set the in-game flag
pub fn set_run_game(value: bool) {
    GB_RUN_GAME.store(value, Ordering::SeqCst);
}

/// Check if we are running Shareware/Spawn version
pub fn is_spawn() -> bool {
    GB_IS_SPAWN.load(Ordering::SeqCst)
}

/// Set the Shareware/Spawn flag
pub fn set_spawn(value: bool) {
    GB_IS_SPAWN.store(value, Ordering::SeqCst);
}

/// Check if Hellfire expansion is loaded
pub fn is_hellfire() -> bool {
    GB_IS_HELLFIRE.load(Ordering::SeqCst)
}

/// Set the Hellfire expansion flag
pub fn set_hellfire(value: bool) {
    GB_IS_HELLFIRE.store(value, Ordering::SeqCst);
}

/// Check if vanilla savefiles are enabled
pub fn is_vanilla() -> bool {
    GB_VANILLA.load(Ordering::SeqCst)
}

/// Set the vanilla savefiles flag
pub fn set_vanilla(value: bool) {
    GB_VANILLA.store(value, Ordering::SeqCst);
}

/// Check if Hellfire mode is forced
pub fn is_force_hellfire() -> bool {
    FORCE_HELLFIRE.load(Ordering::SeqCst)
}

/// Set the force Hellfire flag
pub fn set_force_hellfire(value: bool) {
    FORCE_HELLFIRE.store(value, Ordering::SeqCst);
}

/// Game mode configuration
#[derive(Debug, Clone, Copy, Default)]
pub struct GameMode {
    /// Running the game (vs main menu)
    pub run_game: bool,
    /// Shareware/Spawn version
    pub is_spawn: bool,
    /// Hellfire expansion
    pub is_hellfire: bool,
    /// Vanilla savefiles
    pub vanilla: bool,
    /// Force Hellfire
    pub force_hellfire: bool,
}

impl GameMode {
    /// Create a new game mode from current global state
    pub fn from_globals() -> Self {
        Self {
            run_game: is_run_game(),
            is_spawn: is_spawn(),
            is_hellfire: is_hellfire(),
            vanilla: is_vanilla(),
            force_hellfire: is_force_hellfire(),
        }
    }
    
    /// Apply this game mode to global state
    pub fn apply_to_globals(&self) {
        set_run_game(self.run_game);
        set_spawn(self.is_spawn);
        set_hellfire(self.is_hellfire);
        set_vanilla(self.vanilla);
        set_force_hellfire(self.force_hellfire);
    }
    
    /// Check if this is the full version (not spawn)
    pub fn is_full_version(&self) -> bool {
        !self.is_spawn
    }
    
    /// Get the game ID based on mode flags
    ///
    /// Returns the 4-character game ID used for network and save compatibility
    pub fn game_id(&self) -> u32 {
        match (self.is_hellfire, self.is_spawn) {
            (true, true) => u32::from_be_bytes(*b"HSHR"),   // Hellfire Spawn
            (true, false) => u32::from_be_bytes(*b"HRTL"),  // Hellfire Full
            (false, true) => u32::from_be_bytes(*b"DSHR"),  // Diablo Spawn
            (false, false) => u32::from_be_bytes(*b"DRTL"), // Diablo Full
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_mode_flags() {
        // Reset state
        set_run_game(false);
        set_spawn(false);
        set_hellfire(false);
        
        assert!(!is_run_game());
        assert!(!is_spawn());
        assert!(!is_hellfire());
        
        set_run_game(true);
        assert!(is_run_game());
        
        set_spawn(true);
        assert!(is_spawn());
        
        set_hellfire(true);
        assert!(is_hellfire());
        
        // Reset
        set_run_game(false);
        set_spawn(false);
        set_hellfire(false);
    }
    
    #[test]
    fn test_game_id() {
        let diablo_full = GameMode {
            is_hellfire: false,
            is_spawn: false,
            ..Default::default()
        };
        assert_eq!(diablo_full.game_id(), u32::from_be_bytes(*b"DRTL"));
        
        let diablo_spawn = GameMode {
            is_hellfire: false,
            is_spawn: true,
            ..Default::default()
        };
        assert_eq!(diablo_spawn.game_id(), u32::from_be_bytes(*b"DSHR"));
        
        let hellfire_full = GameMode {
            is_hellfire: true,
            is_spawn: false,
            ..Default::default()
        };
        assert_eq!(hellfire_full.game_id(), u32::from_be_bytes(*b"HRTL"));
        
        let hellfire_spawn = GameMode {
            is_hellfire: true,
            is_spawn: true,
            ..Default::default()
        };
        assert_eq!(hellfire_spawn.game_id(), u32::from_be_bytes(*b"HSHR"));
    }
}
