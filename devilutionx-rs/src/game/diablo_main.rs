//! Diablo Main Program Core Types and State
//!
//! This module contains the core types and state management for the Diablo game.
//! It is a port of the essential types from diablo.h/diablo.cpp without SDL dependencies.
//!
//! # C++ Source Reference
//! - Source/diablo.cpp
//! - Source/diablo.h
//!
//! # Module Organization
//!
//! This module focuses on platform-independent game state and logic types.
//! SDL-specific code will be handled separately in platform abstraction layers.

/// Number of dungeon levels
pub const NUMLEVELS: usize = 25;

/// Game ID constants (big-endian encoded ASCII)
pub mod game_id {
    /// Diablo Full version: "DRTL"
    pub const DIABLO_FULL: u32 = u32::from_be_bytes(*b"DRTL");

    /// Diablo Shareware/Spawn version: "DSHR"
    pub const DIABLO_SPAWN: u32 = u32::from_be_bytes(*b"DSHR");

    /// Hellfire Full version: "HRTL"
    pub const HELLFIRE_FULL: u32 = u32::from_be_bytes(*b"HRTL");

    /// Hellfire Shareware/Spawn version: "HSHR"
    pub const HELLFIRE_SPAWN: u32 = u32::from_be_bytes(*b"HSHR");

    /// Gets the game ID based on game mode flags
    pub fn get_game_id(is_hellfire: bool, is_spawn: bool) -> u32 {
        match (is_hellfire, is_spawn) {
            (true, true) => HELLFIRE_SPAWN,
            (true, false) => HELLFIRE_FULL,
            (false, true) => DIABLO_SPAWN,
            (false, false) => DIABLO_FULL,
        }
    }
}

/// Mouse click type
///
/// Specifies which mouse button was pressed.
///
/// # C++ Reference
/// ```cpp
/// enum clicktype : int8_t {
///     CLICK_NONE,
///     CLICK_LEFT,
///     CLICK_RIGHT,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum ClickType {
    /// No click
    #[default]
    None = 0,
    /// Left mouse button
    Left = 1,
    /// Right mouse button
    Right = 2,
}

impl ClickType {
    /// Returns true if any button is pressed
    pub fn is_pressed(&self) -> bool {
        !matches!(self, ClickType::None)
    }

    /// Returns true if left button is pressed
    pub fn is_left(&self) -> bool {
        matches!(self, ClickType::Left)
    }

    /// Returns true if right button is pressed
    pub fn is_right(&self) -> bool {
        matches!(self, ClickType::Right)
    }
}

/// Game logic step enumeration
///
/// Specifies what game logic step is currently being executed.
/// Used for debugging and synchronization purposes.
///
/// # C++ Reference
/// ```cpp
/// enum class GameLogicStep : uint8_t {
///     None,
///     ProcessPlayers,
///     ProcessMonsters,
///     // ...
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum GameLogicStep {
    /// No step being executed
    #[default]
    None = 0,
    /// Processing player actions
    ProcessPlayers = 1,
    /// Processing monster AI and actions
    ProcessMonsters = 2,
    /// Processing object interactions
    ProcessObjects = 3,
    /// Processing missile movement and collisions
    ProcessMissiles = 4,
    /// Processing item updates
    ProcessItems = 5,
    /// Processing towner (NPC) logic
    ProcessTowners = 6,
    /// Processing items in town
    ProcessItemsTown = 7,
    /// Processing missiles in town
    ProcessMissilesTown = 8,
}

impl GameLogicStep {
    /// Returns the name of the current step
    pub fn name(&self) -> &'static str {
        match self {
            GameLogicStep::None => "None",
            GameLogicStep::ProcessPlayers => "ProcessPlayers",
            GameLogicStep::ProcessMonsters => "ProcessMonsters",
            GameLogicStep::ProcessObjects => "ProcessObjects",
            GameLogicStep::ProcessMissiles => "ProcessMissiles",
            GameLogicStep::ProcessItems => "ProcessItems",
            GameLogicStep::ProcessTowners => "ProcessTowners",
            GameLogicStep::ProcessItemsTown => "ProcessItemsTown",
            GameLogicStep::ProcessMissilesTown => "ProcessMissilesTown",
        }
    }

    /// Returns true if currently processing gameplay entities
    pub fn is_processing(&self) -> bool {
        !matches!(self, GameLogicStep::None)
    }
}

/// Player action type
///
/// Specifies the type of action the player is performing.
/// Used for click-and-hold action handling.
///
/// # C++ Reference
/// ```cpp
/// enum class PlayerActionType : uint8_t {
///     None,
///     Walk,
///     Spell,
///     // ...
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum PlayerActionType {
    /// No action
    #[default]
    None = 0,
    /// Walking movement
    Walk = 1,
    /// Casting a spell (ground target)
    Spell = 2,
    /// Casting a spell at a monster
    SpellMonsterTarget = 3,
    /// Casting a spell at another player
    SpellPlayerTarget = 4,
    /// Basic attack (ground target)
    Attack = 5,
    /// Attacking a monster
    AttackMonsterTarget = 6,
    /// Attacking another player
    AttackPlayerTarget = 7,
    /// Operating an object (door, chest, etc.)
    OperateObject = 8,
}

impl PlayerActionType {
    /// Returns the name of the action
    pub fn name(&self) -> &'static str {
        match self {
            PlayerActionType::None => "None",
            PlayerActionType::Walk => "Walk",
            PlayerActionType::Spell => "Spell",
            PlayerActionType::SpellMonsterTarget => "SpellMonsterTarget",
            PlayerActionType::SpellPlayerTarget => "SpellPlayerTarget",
            PlayerActionType::Attack => "Attack",
            PlayerActionType::AttackMonsterTarget => "AttackMonsterTarget",
            PlayerActionType::AttackPlayerTarget => "AttackPlayerTarget",
            PlayerActionType::OperateObject => "OperateObject",
        }
    }

    /// Returns true if this is an attack action
    pub fn is_attack(&self) -> bool {
        matches!(
            self,
            PlayerActionType::Attack
                | PlayerActionType::AttackMonsterTarget
                | PlayerActionType::AttackPlayerTarget
        )
    }

    /// Returns true if this is a spell action
    pub fn is_spell(&self) -> bool {
        matches!(
            self,
            PlayerActionType::Spell
                | PlayerActionType::SpellMonsterTarget
                | PlayerActionType::SpellPlayerTarget
        )
    }

    /// Returns true if this action has a target
    pub fn has_target(&self) -> bool {
        matches!(
            self,
            PlayerActionType::SpellMonsterTarget
                | PlayerActionType::SpellPlayerTarget
                | PlayerActionType::AttackMonsterTarget
                | PlayerActionType::AttackPlayerTarget
        )
    }
}

/// Pause mode values
pub mod pause_mode {
    /// Game is not paused
    pub const NONE: i32 = 0;
    /// Game is paused (can still render)
    pub const PAUSED: i32 = 1;
    /// Game is fully paused (no input processing)
    pub const FULL_PAUSE: i32 = 2;
}

/// Point structure for coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Creates a new point
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Zero point (origin)
    pub const ZERO: Point = Point { x: 0, y: 0 };

    /// Calculates Manhattan distance to another point
    pub fn walking_distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    /// Adds a displacement to this point
    pub fn offset(&self, dx: i32, dy: i32) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Main Diablo game state
///
/// Contains all the global state variables needed for the game.
/// This is a consolidation of the global variables from diablo.cpp.
#[derive(Debug, Clone)]
pub struct DiabloState {
    /// Dungeon seeds for each level
    pub dungeon_seeds: [u32; NUMLEVELS],

    /// Level-specific seeds (optional override)
    pub level_seeds: [Option<u32>; NUMLEVELS],

    /// Current mouse position
    pub mouse_position: Point,

    /// Result of the game run
    pub run_game_result: bool,

    /// Flag to return to main menu
    pub return_to_main_menu: bool,

    /// Flag to process players
    pub process_players: bool,

    /// Flag indicating game is being loaded
    pub load_game: bool,

    /// Flag indicating cinematic is playing
    pub cine_flag: bool,

    /// Current pause mode
    pub pause_mode: i32,

    /// Current mouse button state
    pub mouse_down: ClickType,

    /// Game tick delay in milliseconds
    pub tick_delay: u16,

    /// Current game logic step
    pub game_logic_step: GameLogicStep,

    /// Last player action type
    pub last_player_action: PlayerActionType,

    /// Product name string
    pub product_name: String,

    /// Version number string
    pub version_number: String,

    /// Flag indicating game loop is starting up
    pub game_loop_startup: bool,

    /// Timeout cursor state
    pub timeout_cursor: i32,

    /// Show intro flag
    pub show_intro: bool,
}

impl Default for DiabloState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiabloState {
    /// Creates a new DiabloState with default values
    pub fn new() -> Self {
        Self {
            dungeon_seeds: [0; NUMLEVELS],
            level_seeds: [None; NUMLEVELS],
            mouse_position: Point::ZERO,
            run_game_result: false,
            return_to_main_menu: false,
            process_players: true,
            load_game: false,
            cine_flag: false,
            pause_mode: pause_mode::NONE,
            mouse_down: ClickType::None,
            tick_delay: 50, // Default 50ms = 20 FPS game logic
            game_logic_step: GameLogicStep::None,
            last_player_action: PlayerActionType::None,
            product_name: "DevilutionX vUnknown".to_string(),
            version_number: "internal version unknown".to_string(),
            game_loop_startup: false,
            timeout_cursor: -1, // CURSOR_NONE
            show_intro: true,
        }
    }

    /// Checks if the game is paused
    pub fn is_paused(&self) -> bool {
        self.pause_mode != pause_mode::NONE
    }

    /// Checks if the game is fully paused
    pub fn is_fully_paused(&self) -> bool {
        self.pause_mode == pause_mode::FULL_PAUSE
    }

    /// Pauses the game
    pub fn pause(&mut self) {
        self.pause_mode = pause_mode::PAUSED;
    }

    /// Fully pauses the game
    pub fn full_pause(&mut self) {
        self.pause_mode = pause_mode::FULL_PAUSE;
    }

    /// Unpauses the game
    pub fn unpause(&mut self) {
        self.pause_mode = pause_mode::NONE;
    }

    /// Resets the last player action
    pub fn reset_action(&mut self) {
        self.last_player_action = PlayerActionType::None;
    }

    /// Sets the current game logic step
    pub fn set_logic_step(&mut self, step: GameLogicStep) {
        self.game_logic_step = step;
    }

    /// Gets the seed for a specific level
    pub fn get_level_seed(&self, level: usize) -> Option<u32> {
        if level < NUMLEVELS {
            self.level_seeds[level].or(Some(self.dungeon_seeds[level]))
        } else {
            None
        }
    }

    /// Sets the seed for a specific level
    pub fn set_level_seed(&mut self, level: usize, seed: u32) {
        if level < NUMLEVELS {
            self.level_seeds[level] = Some(seed);
        }
    }

    /// Clears the level seed override for a specific level
    pub fn clear_level_seed(&mut self, level: usize) {
        if level < NUMLEVELS {
            self.level_seeds[level] = None;
        }
    }
}

/// Cursor constants
pub mod cursor {
    /// No cursor / invalid cursor
    pub const NONE: i32 = -1;
    /// Hand cursor
    pub const HAND: i32 = 0;
    /// Disarm cursor
    pub const DISARM: i32 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_type_values() {
        assert_eq!(ClickType::None as i8, 0);
        assert_eq!(ClickType::Left as i8, 1);
        assert_eq!(ClickType::Right as i8, 2);
    }

    #[test]
    fn test_click_type_methods() {
        assert!(!ClickType::None.is_pressed());
        assert!(ClickType::Left.is_pressed());
        assert!(ClickType::Right.is_pressed());

        assert!(ClickType::Left.is_left());
        assert!(!ClickType::Left.is_right());

        assert!(!ClickType::Right.is_left());
        assert!(ClickType::Right.is_right());
    }

    #[test]
    fn test_game_logic_step_values() {
        assert_eq!(GameLogicStep::None as u8, 0);
        assert_eq!(GameLogicStep::ProcessPlayers as u8, 1);
        assert_eq!(GameLogicStep::ProcessMonsters as u8, 2);
        assert_eq!(GameLogicStep::ProcessObjects as u8, 3);
        assert_eq!(GameLogicStep::ProcessMissiles as u8, 4);
        assert_eq!(GameLogicStep::ProcessItems as u8, 5);
        assert_eq!(GameLogicStep::ProcessTowners as u8, 6);
        assert_eq!(GameLogicStep::ProcessItemsTown as u8, 7);
        assert_eq!(GameLogicStep::ProcessMissilesTown as u8, 8);
    }

    #[test]
    fn test_game_logic_step_methods() {
        assert!(!GameLogicStep::None.is_processing());
        assert!(GameLogicStep::ProcessPlayers.is_processing());
        assert!(GameLogicStep::ProcessMonsters.is_processing());

        assert_eq!(GameLogicStep::ProcessPlayers.name(), "ProcessPlayers");
    }

    #[test]
    fn test_player_action_type_values() {
        assert_eq!(PlayerActionType::None as u8, 0);
        assert_eq!(PlayerActionType::Walk as u8, 1);
        assert_eq!(PlayerActionType::Spell as u8, 2);
        assert_eq!(PlayerActionType::Attack as u8, 5);
        assert_eq!(PlayerActionType::OperateObject as u8, 8);
    }

    #[test]
    fn test_player_action_type_methods() {
        assert!(PlayerActionType::Attack.is_attack());
        assert!(PlayerActionType::AttackMonsterTarget.is_attack());
        assert!(!PlayerActionType::Walk.is_attack());

        assert!(PlayerActionType::Spell.is_spell());
        assert!(PlayerActionType::SpellMonsterTarget.is_spell());
        assert!(!PlayerActionType::Attack.is_spell());

        assert!(PlayerActionType::AttackMonsterTarget.has_target());
        assert!(PlayerActionType::SpellPlayerTarget.has_target());
        assert!(!PlayerActionType::Walk.has_target());
        assert!(!PlayerActionType::Attack.has_target());
    }

    #[test]
    fn test_game_id_constants() {
        assert_eq!(game_id::DIABLO_FULL, u32::from_be_bytes(*b"DRTL"));
        assert_eq!(game_id::DIABLO_SPAWN, u32::from_be_bytes(*b"DSHR"));
        assert_eq!(game_id::HELLFIRE_FULL, u32::from_be_bytes(*b"HRTL"));
        assert_eq!(game_id::HELLFIRE_SPAWN, u32::from_be_bytes(*b"HSHR"));
    }

    #[test]
    fn test_game_id_function() {
        assert_eq!(game_id::get_game_id(false, false), game_id::DIABLO_FULL);
        assert_eq!(game_id::get_game_id(false, true), game_id::DIABLO_SPAWN);
        assert_eq!(game_id::get_game_id(true, false), game_id::HELLFIRE_FULL);
        assert_eq!(game_id::get_game_id(true, true), game_id::HELLFIRE_SPAWN);
    }

    #[test]
    fn test_point_basic() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);

        assert_eq!(Point::ZERO.x, 0);
        assert_eq!(Point::ZERO.y, 0);
    }

    #[test]
    fn test_point_walking_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(p1.walking_distance(p2), 4); // max(3, 4) = 4

        let p3 = Point::new(5, 2);
        assert_eq!(p1.walking_distance(p3), 5); // max(5, 2) = 5
    }

    #[test]
    fn test_point_arithmetic() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(5, 8);

        let sum = p1 + p2;
        assert_eq!(sum.x, 15);
        assert_eq!(sum.y, 28);

        let diff = p1 - p2;
        assert_eq!(diff.x, 5);
        assert_eq!(diff.y, 12);
    }

    #[test]
    fn test_point_offset() {
        let p = Point::new(10, 20);
        let p2 = p.offset(5, -3);
        assert_eq!(p2.x, 15);
        assert_eq!(p2.y, 17);
    }

    #[test]
    fn test_diablo_state_default() {
        let state = DiabloState::new();

        assert_eq!(state.dungeon_seeds, [0; NUMLEVELS]);
        assert_eq!(state.mouse_position, Point::ZERO);
        assert!(!state.run_game_result);
        assert!(!state.return_to_main_menu);
        assert!(state.process_players);
        assert!(!state.load_game);
        assert!(!state.cine_flag);
        assert_eq!(state.pause_mode, pause_mode::NONE);
        assert_eq!(state.mouse_down, ClickType::None);
        assert_eq!(state.tick_delay, 50);
        assert_eq!(state.game_logic_step, GameLogicStep::None);
        assert_eq!(state.last_player_action, PlayerActionType::None);
        assert!(state.show_intro);
    }

    #[test]
    fn test_diablo_state_pause() {
        let mut state = DiabloState::new();

        assert!(!state.is_paused());
        assert!(!state.is_fully_paused());

        state.pause();
        assert!(state.is_paused());
        assert!(!state.is_fully_paused());

        state.full_pause();
        assert!(state.is_paused());
        assert!(state.is_fully_paused());

        state.unpause();
        assert!(!state.is_paused());
        assert!(!state.is_fully_paused());
    }

    #[test]
    fn test_diablo_state_level_seeds() {
        let mut state = DiabloState::new();

        // Set dungeon seed
        state.dungeon_seeds[0] = 12345;
        assert_eq!(state.get_level_seed(0), Some(12345));

        // Override with level seed
        state.set_level_seed(0, 99999);
        assert_eq!(state.get_level_seed(0), Some(99999));

        // Clear override
        state.clear_level_seed(0);
        assert_eq!(state.get_level_seed(0), Some(12345));

        // Invalid level
        assert_eq!(state.get_level_seed(100), None);
    }

    #[test]
    fn test_diablo_state_logic_step() {
        let mut state = DiabloState::new();

        assert_eq!(state.game_logic_step, GameLogicStep::None);

        state.set_logic_step(GameLogicStep::ProcessPlayers);
        assert_eq!(state.game_logic_step, GameLogicStep::ProcessPlayers);

        state.set_logic_step(GameLogicStep::ProcessMonsters);
        assert_eq!(state.game_logic_step, GameLogicStep::ProcessMonsters);
    }

    #[test]
    fn test_diablo_state_reset_action() {
        let mut state = DiabloState::new();

        state.last_player_action = PlayerActionType::Attack;
        assert_eq!(state.last_player_action, PlayerActionType::Attack);

        state.reset_action();
        assert_eq!(state.last_player_action, PlayerActionType::None);
    }

    #[test]
    fn test_cursor_constants() {
        assert_eq!(cursor::NONE, -1);
        assert_eq!(cursor::HAND, 0);
        assert_eq!(cursor::DISARM, 1);
    }
}
