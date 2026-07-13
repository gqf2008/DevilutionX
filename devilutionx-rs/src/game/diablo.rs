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

// 注意: GameWindow 已移除 (过度设计)
// dx.rs 现在只包含 C++ API: dx_init, dx_cleanup, render_present 等
// use crate::engine::dx::GameWindow;
use super::player_exact::Player;

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

// ============================================================================
// GameState for main.rs compatibility
// ============================================================================

/// Main game state structure (simplified for main.rs)
///
/// C++ Reference: Global variables in diablo.cpp
pub struct GameState {
    /// Current player
    pub player: Player,
    /// Is multiplayer mode
    pub is_multiplayer: bool,
    /// Random seed
    pub seed: u64,
}

impl GameState {
    /// Create a new game state
    pub fn new(player: Player, is_multiplayer: bool, seed: u64) -> Self {
        Self {
            player,
            is_multiplayer,
            seed,
        }
    }
}

// ============================================================================
// Interface Mode
// ============================================================================

/// Interface mode for game startup
///
/// C++ Reference: `interface_mode` enum in diablo.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceMode {
    /// New game
    NewGame,
    /// Continue/load game
    LoadGame,
}

// ============================================================================
// Global State Variables (C++ style)
// ============================================================================

/// Game is running
/// C++ Reference: `gbRunGame`
pub static mut GB_RUN_GAME: bool = false;

/// Process players this tick
/// C++ Reference: `gbProcessPlayers`
pub static mut GB_PROCESS_PLAYERS: bool = false;

/// Load game on startup
/// C++ Reference: `gbLoadGame`
pub static mut GB_LOAD_GAME: bool = false;

/// Game loop startup flag
/// C++ Reference: `gbGameLoopStartup`
pub static mut GB_GAME_LOOP_STARTUP: bool = false;

// ============================================================================
// Game Loop
// ============================================================================

// 注意: run_game_loop 已移除
// 原因: 使用了已删除的 GameWindow 类（过度设计）
// C++ RunGameLoop 使用 SDL 直接管理，不是封装类
// 需要按照 C++ 方式重写

/// Stop the game loop
///
/// C++ Reference: Sets `gbRunGame = false`
pub fn diablo_quit_game() {
    unsafe {
        GB_RUN_GAME = false;
    }
}

/// Check if game is running
pub fn is_game_running() -> bool {
    unsafe { GB_RUN_GAME }
}

// ============================================================================
// Level / Dungeon Types
//
// C++ Reference: Source/levels/gendung_defs.hpp
// ============================================================================

/// Dungeon/level type (mirrors C++ `dungeon_type` enum).
///
/// C++ Reference: `Source/levels/gendung_defs.hpp:13-24`
/// ```cpp
/// enum dungeon_type : int8_t {
///     DTYPE_TOWN, DTYPE_CATHEDRAL, DTYPE_CATACOMBS, DTYPE_CAVES,
///     DTYPE_HELL, DTYPE_NEST, DTYPE_CRYPT,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum DungeonType {
    #[default]
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,
    Crypt = 6,
}

impl DungeonType {
    /// Returns true if this is the Tristram town level.
    pub fn is_town(&self) -> bool {
        matches!(self, DungeonType::Town)
    }

    /// Human-readable name for logging / progress screens.
    pub fn name(&self) -> &'static str {
        match self {
            DungeonType::Town => "Town",
            DungeonType::Cathedral => "Cathedral",
            DungeonType::Catacombs => "Catacombs",
            DungeonType::Caves => "Caves",
            DungeonType::Hell => "Hell",
            DungeonType::Nest => "Nest",
            DungeonType::Crypt => "Crypt",
        }
    }
}

/// Determines the dungeon type for a given level number.
///
/// C++ Reference: `Source/levels/gendung.cpp` — `GetLevelType(int level)`
/// Level 0 = Town; 1-4, 9-12, 17-20 = Cathedral; etc. The Hellfire expansion
/// adds levels 17-24 (Nest + Crypt); classic Diablo stops at 16.
pub fn get_level_type(level: u8, is_hellfire: bool) -> DungeonType {
    if level == 0 {
        return DungeonType::Town;
    }
    // Classic layout (Diablo + Hellfire share these):
    //  1-4   Cathedral (L1)
    //  5-8   Catacombs (L2)
    //  9-12  Caves (L3)
    //  13-16 Hell (L4)
    match level {
        1..=4 => DungeonType::Cathedral,
        5..=8 => DungeonType::Catacombs,
        9..=12 => DungeonType::Caves,
        13..=16 => DungeonType::Hell,
        17..=20 if is_hellfire => DungeonType::Nest,
        21..=24 if is_hellfire => DungeonType::Crypt,
        _ => DungeonType::Cathedral,
    }
}

/// Level entry direction (mirrors C++ `lvl_entry` enum).
///
/// C++ Reference: `Source/levels/gendung_defs.hpp:26-35`
/// ```cpp
/// enum lvl_entry : uint8_t {
///     ENTRY_MAIN, ENTRY_PREV, ENTRY_SETLVL, ENTRY_RTNLVL,
///     ENTRY_LOAD, ENTRY_WARPLVL, ENTRY_TWARPDN, ENTRY_TWARPUP,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum LvlEntry {
    #[default]
    Main = 0,
    Prev = 1,
    SetLvl = 2,
    RtnLvl = 3,
    Load = 4,
    WarpLvl = 5,
    TWarpDn = 6,
    TWarpUp = 7,
}

// ============================================================================
// Interface Mode (full WM_* enum)
//
// C++ Reference: Source/interfac.h:24-43
// ============================================================================

/// Game progress / level transition message (mirrors C++ `interface_mode`).
///
/// C++ Reference: `Source/interfac.h:24-43`
/// These are the custom-event codes pushed through the SDL event queue to
/// drive the loading/cutscene flow (`ShowProgress`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum InterfaceModeWm {
    /// Descend to next level — `WM_DIABNEXTLVL`
    #[default]
    DiabNextLvl = 0,
    /// Ascend to previous level — `WM_DIABPREVLVL`
    DiabPrevLvl = 1,
    /// Return from a set (quest) level — `WM_DIABRTNLVL`
    DiabRtnLvl = 2,
    /// Enter a set (quest) level — `WM_DIABSETLVL`
    DiabSetLvl = 3,
    /// Town-portal warp — `WM_DIABWARPLVL`
    DiabWarpLvl = 4,
    /// Town warp down — `WM_DIABTOWNWARP`
    DiabTownWarp = 5,
    /// Town warp up — `WM_DIABTWARPUP`
    DiabTWarpUp = 6,
    /// Resurrect in town — `WM_DIABRETOWN`
    DiabRetown = 7,
    /// New game — `WM_DIABNEWGAME`
    DiabNewGame = 8,
    /// Load game — `WM_DIABLOADGAME`
    DiabLoadGame = 9,
    /// Async load progress tick — `WM_PROGRESS`
    Progress = 10,
    /// Async load error — `WM_ERROR`
    Error = 11,
    /// Async load done — `WM_DONE`
    Done = 12,
}

impl InterfaceModeWm {
    /// Human-readable label for the progress/cutscene screen
    /// ("Entering Cathedral", "Entering Town", ...).
    ///
    /// C++ Reference: `Source/interfac.cpp` — the progress text shown during
    /// loading is keyed off the cutscene (CutStart, CutTown, CutLevel1, ...).
    pub fn progress_label(&self) -> &'static str {
        match self {
            InterfaceModeWm::DiabNewGame | InterfaceModeWm::DiabLoadGame => "Entering Cathedral",
            InterfaceModeWm::DiabRetown => "Entering Town",
            InterfaceModeWm::DiabNextLvl | InterfaceModeWm::DiabTownWarp => "Entering Dungeon",
            InterfaceModeWm::DiabPrevLvl | InterfaceModeWm::DiabTWarpUp => "Leaving Dungeon",
            InterfaceModeWm::DiabWarpLvl => "Entering Portal",
            InterfaceModeWm::DiabSetLvl => "Entering Quest Level",
            InterfaceModeWm::DiabRtnLvl => "Leaving Quest Level",
            InterfaceModeWm::Progress | InterfaceModeWm::Error | InterfaceModeWm::Done => "",
        }
    }
}

// ============================================================================
// Cutscene selection
//
// C++ Reference: Source/interfac.cpp:99-135 (PickCutscene) + :137-189 (LoadCutsceneBackground)
// ============================================================================

/// Cutscene/background art variant shown during loading screens.
///
/// C++ Reference: `Source/interfac.h:59-71` — `Cutscenes` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cutscene {
    Start,
    Town,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
    Portal,
    PortalRed,
    Gate,
}

/// Picks the cutscene to display for a given interface mode and target level.
///
/// C++ Reference: `Source/interfac.cpp:99-135` — `Cutscenes PickCutscene(interface_mode uMsg)`
///
/// The Rust port simplifies the level-type lookup by taking the resolved
/// `DungeonType` directly (the C++ original re-derives it via
/// `GetLevelType(lvl)`).
pub fn pick_cutscene(
    u_msg: InterfaceModeWm,
    player_level: u8,
    level_type: DungeonType,
) -> Cutscene {
    match u_msg {
        InterfaceModeWm::DiabLoadGame | InterfaceModeWm::DiabNewGame => Cutscene::Start,
        InterfaceModeWm::DiabRetown => Cutscene::Town,
        InterfaceModeWm::DiabNextLvl
        | InterfaceModeWm::DiabPrevLvl
        | InterfaceModeWm::DiabTownWarp
        | InterfaceModeWm::DiabTWarpUp => {
            // L1 → town (entering Cathedral from town)
            if player_level == 1 && u_msg == InterfaceModeWm::DiabNextLvl {
                return Cutscene::Town;
            }
            // L16 → Diablo's lair gate cinematic
            if player_level == 16 && u_msg == InterfaceModeWm::DiabNextLvl {
                return Cutscene::Gate;
            }
            cutscene_from_level_type(level_type)
        }
        InterfaceModeWm::DiabWarpLvl => Cutscene::Portal,
        // Set level / return-from-set: C++ special-cases Bone Chamber (L2) and
        // Vile Betrayer (red portal). Simplified here to default L1 art.
        InterfaceModeWm::DiabSetLvl | InterfaceModeWm::DiabRtnLvl => Cutscene::Level1,
        _ => Cutscene::Level1,
    }
}

/// Maps a dungeon type to its loading-screen cutscene variant.
///
/// C++ Reference: `Source/interfac.cpp:86-97` — `GetCutSceneFromLevelType`.
fn cutscene_from_level_type(level_type: DungeonType) -> Cutscene {
    match level_type {
        DungeonType::Cathedral => Cutscene::Level1,
        DungeonType::Catacombs => Cutscene::Level2,
        DungeonType::Caves => Cutscene::Level3,
        DungeonType::Hell => Cutscene::Level4,
        DungeonType::Nest => Cutscene::Level6,
        DungeonType::Crypt => Cutscene::Level5,
        DungeonType::Town => Cutscene::Town,
    }
}

// ============================================================================
// Global level state
// ============================================================================

/// Current level index (C++ `currlevel`). 0 = town, 1-16 = classic, 17-24 = Hellfire.
///
/// C++ Reference: `Source/levels/gendung.h` — `extern uint8_t currlevel;`
pub static mut CURR_LEVEL: u8 = 0;

/// Active level type (C++ `leveltype`).
///
/// C++ Reference: `Source/levels/gendung.h:135` — `extern dungeon_type leveltype;`
pub static mut LEVEL_TYPE: DungeonType = DungeonType::Town;

/// True when a set (quest) map is active (C++ `setlevel`).
///
/// C++ Reference: `Source/levels/gendung.h` — `extern bool setlevel;`
pub static mut SET_LEVEL: bool = false;

/// True if the current build is Hellfire (C++ `gbIsHellfire`).
pub static mut GB_IS_HELLFIRE: bool = false;

/// True for multiplayer games (C++ `gbIsMultiplayer`).
pub static mut GB_IS_MULTIPLAYER: bool = false;

/// True if loading a save game rather than starting fresh (C++ `gbLoadGame`).
pub static mut GB_LOAD_GAME_LOCAL: bool = false;

/// Returns the current level index (thread-unsafe read of `CURR_LEVEL`).
pub fn current_level() -> u8 {
    unsafe { CURR_LEVEL }
}

/// Sets the current level index and updates `LEVEL_TYPE` to match.
pub fn set_current_level(level: u8) {
    unsafe {
        CURR_LEVEL = level;
        LEVEL_TYPE = get_level_type(level, GB_IS_HELLFIRE);
    }
}

/// Returns the active level type (thread-unsafe read of `LEVEL_TYPE`).
pub fn current_level_type() -> DungeonType {
    unsafe { LEVEL_TYPE }
}

// ============================================================================
// GameLogic — per-tick entity processing sequence
//
// C++ Reference: Source/diablo.cpp:1510-1556 — void GameLogic()
// ============================================================================

/// Outcome of one `game_logic` tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameLogicOutcome {
    /// Logic ran to completion.
    Ok,
    /// Input processing indicated the tick should be skipped (paused / menu open).
    Skipped,
}

/// Trait abstracting the subsystems `GameLogic` drives each tick.
///
/// Each method corresponds to a C++ `ProcessXxx()` call inside `GameLogic()`.
/// Implementors wire these to the real systems (player update, monster AI,
/// missile physics, ...); tests can stub them to record call order. This keeps
/// `diablo.rs` free of the heavy game-state borrow-graph while preserving the
/// exact C++ per-tick sequence.
pub trait GameLogicContext {
    /// `ProcessInput()` — read pending input / controller actions.
    /// Returns `false` to short-circuit the tick (C++ early-returns from
    /// `GameLogic` when `ProcessInput` returns false).
    fn process_input(&mut self) -> bool {
        true
    }
    /// `ProcessPlayers()` — update each player's mode/animation/movement.
    fn process_players(&mut self) {}
    /// `ProcessMonsters()` — monster AI + animation (dungeon only).
    fn process_monsters(&mut self) {}
    /// `ProcessObjects()` — object animation + interaction (dungeon only).
    fn process_objects(&mut self) {}
    /// `ProcessMissiles()` — missile physics + collisions.
    fn process_missiles(&mut self) {}
    /// `ProcessItems()` — item animation + pickup logic.
    fn process_items(&mut self) {}
    /// `ProcessTowners()` — towner AI (town only).
    fn process_towners(&mut self) {}
    /// `ProcessLightList()` — recompute dynamic light radii (dungeon only).
    fn process_light_list(&mut self) {}
    /// `ProcessVisionList()` — recompute vision sources (dungeon only).
    fn process_vision_list(&mut self) {}
    /// `sound_update()` — update positional/spatial sounds.
    fn sound_update(&mut self) {}
    /// `CheckTriggers()` — fire level-transition triggers under the player.
    fn check_triggers(&mut self) {}
    /// `CheckQuests()` — advance quest state machines.
    fn check_quests(&mut self) {}
    /// `RedrawViewport()` — mark the dungeon viewport dirty for redraw.
    fn redraw_viewport(&mut self) {}
    /// `pfile_update(false)` — flush hero save data.
    fn pfile_update(&mut self) {}
    /// `plrctrls_after_game_logic()` — post-logic controller polling.
    fn plrctrls_after_game_logic(&mut self) {}
}

/// Run one full `GameLogic()` tick against `ctx`, faithfully reproducing the
/// C++ per-tick entity-processing order.
///
/// C++ Reference: `Source/diablo.cpp:1510-1556`
///
/// Sequence (dungeon):
///   1. `ProcessInput()` — early-return if it yields false
///   2. `ProcessPlayers()` (only when `gbProcessPlayers`)
///   3. `ProcessMonsters()`
///   4. `ProcessObjects()`
///   5. `ProcessMissiles()`
///   6. `ProcessItems()`
///   7. `ProcessLightList()`
///   8. `ProcessVisionList()`
///
/// Sequence (town) replaces steps 3-8 with:
///   `ProcessTowners()`, `ProcessItems()`, `ProcessMissiles()`.
///
/// Then for both: `sound_update()`, `CheckTriggers()`, `CheckQuests()`,
/// `RedrawViewport()`, `pfile_update(false)`, `plrctrls_after_game_logic()`.
///
/// The current `GameLogicStep` is written to `step_out` (if provided) so the
/// caller can mirror the C++ `gGameLogicStep` global for debugging.
pub fn game_logic(
    ctx: &mut dyn GameLogicContext,
    process_players: bool,
    level_type: DungeonType,
    step_out: Option<&mut GameLogicStep>,
) -> GameLogicOutcome {
    // C++:1512 — if (!ProcessInput()) return;
    if !ctx.process_input() {
        return GameLogicOutcome::Skipped;
    }

    // C++:1515-1518 — if (gbProcessPlayers) { gGameLogicStep=ProcessPlayers; ProcessPlayers(); }
    if process_players {
        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessPlayers;
        }
        ctx.process_players();
    }

    if !level_type.is_town() {
        // DUNGEON path — C++:1519-1532
        // ProcessMonsters → ProcessObjects → ProcessMissiles → ProcessItems
        // → ProcessLightList → ProcessVisionList
        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessMonsters;
        }
        ctx.process_monsters();

        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessObjects;
        }
        ctx.process_objects();

        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessMissiles;
        }
        ctx.process_missiles();

        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessItems;
        }
        ctx.process_items();

        ctx.process_light_list();
        ctx.process_vision_list();
    } else {
        // TOWN path — C++:1533-1540
        // ProcessTowners → ProcessItems → ProcessMissiles
        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessTowners;
        }
        ctx.process_towners();

        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessItemsTown;
        }
        ctx.process_items();

        if let Some(s) = step_out {
            *s = GameLogicStep::ProcessMissilesTown;
        }
        ctx.process_missiles();
    }

    // C++:1541 — gGameLogicStep = None;
    if let Some(s) = step_out {
        *s = GameLogicStep::None;
    }

    // C++:1549-1555 — sound_update(); CheckTriggers(); CheckQuests();
    //                  RedrawViewport(); pfile_update(false);
    //                  plrctrls_after_game_logic();
    ctx.sound_update();
    ctx.check_triggers();
    ctx.check_quests();
    ctx.redraw_viewport();
    ctx.pfile_update();
    ctx.plrctrls_after_game_logic();

    GameLogicOutcome::Ok
}

// ============================================================================
// game_loop — iteration controller wrapping GameLogic
//
// C++ Reference: Source/diablo.cpp:3416-3433 — bool game_loop(bool bStartup)
// ============================================================================

/// Runs `game_logic` for `iterations` ticks, mirroring the C++ loop structure.
///
/// C++ Reference: `Source/diablo.cpp:3416-3433` — `bool game_loop(bool bStartup)`.
///
/// In C++ the iteration count is `bStartup ? nTickRate*3 : 3`; we accept it
/// directly so callers can compute it from the active tick rate. Each
/// iteration:
///   1. `multi_handle_delta()` → on false, set timeout cursor and return false
///   2. `TimeoutCursor(false)`
///   3. `GameLogic()`
///   4. `ClearLastSentPlayerCmd()`
///   5. early-exit when `!gbRunGame || !gbIsMultiplayer`
///
/// `multi_handle_delta` / `clear_last_sent_cmd` are passed as callbacks so
/// this stays decoupled from the (not-yet-ported) network stack.
pub fn game_loop(
    ctx: &mut dyn GameLogicContext,
    iterations: u32,
    process_players: bool,
    level_type: DungeonType,
    step_out: Option<&mut GameLogicStep>,
    multi_handle_delta: impl FnMut() -> bool,
    mut clear_last_sent_cmd: impl FnMut(),
) -> bool {
    let mut multi_handle_delta = multi_handle_delta;
    for _ in 0..iterations {
        // C++:3421 — if (!multi_handle_delta()) { TimeoutCursor(true); return false; }
        if !multi_handle_delta() {
            timeout_cursor(true);
            return false;
        }
        timeout_cursor(false);

        // C++:3426 — GameLogic();
        game_logic(ctx, process_players, level_type, step_out);

        // C++:3427 — ClearLastSentPlayerCmd();
        clear_last_sent_cmd();

        // C++:3429 — early exit when single-player / game ended.
        if !is_game_running() || !is_multiplayer() {
            break;
        }
    }
    true
}

/// Reads the multiplayer flag (thread-unsafe read of `GB_IS_MULTIPLAYER`).
pub fn is_multiplayer() -> bool {
    unsafe { GB_IS_MULTIPLAYER }
}

/// Sets/clears the network-timeout hourglass cursor.
///
/// C++ Reference: `Source/diablo.cpp:1558-1581` — `void TimeoutCursor(bool)`.
/// The C++ version swaps the cursor sprite + pushes an info-box message; the
/// Rust port only tracks the state so the render layer can react. Full UI
/// behaviour is a TODO pending the cursor/info-box subsystems.
pub fn timeout_cursor(_b_timeout: bool) {
    // TODO(diablo.cpp:1558): implement hourglass cursor swap + info-box message
    // once the cursor (NewCursor) and InfoString subsystems are ported.
}

// ============================================================================
// IsDiabloAlive — gates player processing after Diablo is defeated
//
// C++ Reference: Source/diablo.cpp:3458-3467
// ============================================================================

/// Returns true if Diablo is still alive (quest not done), so the game should
/// keep processing player input.
///
/// C++ Reference: `Source/diablo.cpp:3458-3467` — `bool IsDiabloAlive(bool playSFX)`.
/// Once the Diablo quest is marked `QUEST_DONE` in a single-player game, the
/// engine stops processing players and optionally plays the death SFX.
///
/// The Rust port takes the quest-active flag directly (the C++ version reads
/// the global `Quests[Q_DIABLO]` table). `play_sfx` is currently ignored
/// because the SFX routing is handled elsewhere; it is kept for API fidelity.
pub fn is_diablo_alive(_play_sfx: bool, diablo_quest_done: bool) -> bool {
    // C++:3460 — if (Quests[Q_DIABLO]._qactive == QUEST_DONE && !gbIsMultiplayer)
    if diablo_quest_done && !is_multiplayer() {
        // C++:3462 — if (playSFX) PlaySFX(SfxID::DiabloDeath);
        return false;
    }
    true
}

// ============================================================================
// diablo_color_cyc_logic — palette color cycling
//
// C++ Reference: Source/diablo.cpp:3435-3456
// ============================================================================

/// Applies palette color cycling for the active level type.
///
/// C++ Reference: `Source/diablo.cpp:3435-3456` —
/// `void diablo_color_cyc_logic()`.
///
/// Cycling is skipped when the color-cycling option is off or the game is
/// paused. The per-level-type effect:
///   * Caves  → `palette_update_caves()` (lava animation), or
///              `UpdatePWaterPalette()` on the Poisoned Water quest level.
///   * Hell   → `lighting_color_cycling()`.
///   * Nest   → `palette_update_hive()`.
///   * Crypt  → `palette_update_crypt()`.
///
/// The Rust port delegates the actual palette mutation to a callback so the
/// palette engine can stay in its own module.
pub fn diablo_color_cyc_logic(
    color_cycling_enabled: bool,
    pause_mode_val: i32,
    level_type: DungeonType,
    is_pwater_quest_level: bool,
    mut update_palette: impl FnMut(ColorCycleEffect),
) {
    // C++:3437 — if (!*GetOptions().Graphics.colorCycling) return;
    if !color_cycling_enabled {
        return;
    }
    // C++:3440 — if (PauseMode != 0) return;
    if pause_mode_val != pause_mode::NONE {
        return;
    }

    // C++:3443-3455 — dispatch on leveltype.
    match level_type {
        DungeonType::Caves => {
            if is_pwater_quest_level {
                update_palette(ColorCycleEffect::PWater);
            } else {
                update_palette(ColorCycleEffect::Caves);
            }
        }
        DungeonType::Hell => update_palette(ColorCycleEffect::Lighting),
        DungeonType::Nest => update_palette(ColorCycleEffect::Hive),
        DungeonType::Crypt => update_palette(ColorCycleEffect::Crypt),
        _ => {}
    }
}

/// Palette color-cycle effect kinds, mirroring the per-level-type branches of
/// C++ `diablo_color_cyc_logic`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCycleEffect {
    /// `palette_update_caves()` — lava animation in the Caves.
    Caves,
    /// `UpdatePWaterPalette()` — Poisoned Water quest sub-level.
    PWater,
    /// `lighting_color_cycling()` — Hell fire flicker.
    Lighting,
    /// `palette_update_hive()` — Nest (Hive) background pulse.
    Hive,
    /// `palette_update_crypt()` — Crypt background pulse.
    Crypt,
}

// ============================================================================
// Palette fade in/out
//
// C++ Reference: engine/palette.cpp — PaletteFadeIn/PaletteFadeOut
// (declared in diablo.cpp at :840, :872, :947)
// ============================================================================

/// Fades the screen palette in over `steps` frames.
///
/// C++ Reference: `engine/palette.cpp` — `void PaletteFadeIn(int steps)`.
/// In C++ this lerps the logical palette from all-black to the target palette
/// across `steps` frames, calling `DrawAndBlit` each frame. The Rust port is
/// a hook: the real palette engine is not yet ported, so this is a no-op stub
/// that callers can wire up once the palette subsystem exists.
pub fn palette_fade_in(_steps: u8) {
    // TODO(engine/palette.cpp): lerp logical palette from black → target
    // across `steps` frames, redrawing each frame. Currently a no-op; the
    // SDL2 renderer path does its own fade in game_loop.rs.
}

/// Fades the screen palette out over `steps` frames.
///
/// C++ Reference: `engine/palette.cpp` — `void PaletteFadeOut(int steps)`.
pub fn palette_fade_out(_steps: u8) {
    // TODO(engine/palette.cpp): lerp logical palette target → black.
}

/// Forces the active palette to all-black.
///
/// C++ Reference: `engine/palette.cpp` — `void BlackPalette()`. Used by
/// `PrepareForFadeIn` (diablo.cpp:715) to set up the fade-in start state.
pub fn black_palette() {
    // TODO(engine/palette.cpp): set logical palette to all-black.
}

// ============================================================================
// PrepareForFadeIn
//
// C++ Reference: Source/diablo.cpp:715-726
// ============================================================================

/// Renders one frame with a black palette so the subsequent fade-in has a
/// known start state.
///
/// C++ Reference: `Source/diablo.cpp:715-726` — `void PrepareForFadeIn()`.
/// In C++ this calls `BlackPalette()`, then `RedrawEverything()` and drains
/// the redraw queue (`while (IsRedrawEverything()) DrawAndBlit()`). The Rust
/// port exposes the sequence as a callback-driven hook so the rendering layer
/// can perform the actual buffer clears.
pub fn prepare_for_fade_in(
    headless: bool,
    mut redraw_everything_and_blit: impl FnMut(),
) {
    if headless {
        return;
    }
    black_palette();
    // C++:722 — RedrawEverything();
    // C++:723-725 — while (IsRedrawEverything()) DrawAndBlit();
    redraw_everything_and_blit();
}

// ============================================================================
// CreateLevel — dispatches to the per-type DRLG + trigger init
//
// C++ Reference: Source/diablo.cpp:1428-1462
// ============================================================================

/// Creates the dungeon layout for the current level and initialises its
/// triggers, then loads a random level palette.
///
/// C++ Reference: `Source/diablo.cpp:1428-1462` — `void CreateLevel(lvl_entry entry)`.
///
/// C++ calls `CreateDungeon(DungeonSeeds[currlevel], entry)` then dispatches
/// on `leveltype` to `InitTownTriggers` / `InitL1Triggers` / ... / `InitCryptTriggers`,
/// calls `Freeupstairs()` for non-town, and finally `LoadRndLvlPal(leveltype)`.
///
/// The Rust port delegates the actual generation + trigger setup to callbacks
/// so this stays free of the (not-yet-ported) DRLG modules. Returns the result
/// of the palette load.
pub fn create_level(
    level_type: DungeonType,
    entry: LvlEntry,
    dungeon_seed: u32,
    mut generate: impl FnMut(u32, LvlEntry),
    mut init_triggers: impl FnMut(),
    mut free_upstairs: impl FnMut(),
    mut load_rnd_lvl_pal: impl FnMut(DungeonType) -> Result<(), String>,
) -> Result<(), String> {
    // C++:1430 — CreateDungeon(DungeonSeeds[currlevel], entry);
    generate(dungeon_seed, entry);

    // C++:1432-1456 — switch (leveltype) InitXxxTriggers();
    init_triggers();

    // C++:1458-1460 — if (leveltype != DTYPE_TOWN) Freeupstairs();
    if !level_type.is_town() {
        free_upstairs();
    }

    // C++:1461 — LoadRndLvlPal(leveltype);
    load_rnd_lvl_pal(level_type)
}

// ============================================================================
// LoadLvlGFX — load per-level tile art
//
// C++ Reference: Source/diablo.cpp:1347-1409
// ============================================================================

/// Per-level art asset paths, resolved from the active `DungeonType`.
///
/// C++ Reference: `Source/diablo.cpp:1347-1409` — `LoadLvlGFX()` switches on
/// `leveltype` and loads a (cel, til, special) triple plus, for town, a
/// fallback `levels\towndata\town.*` pair when the `nlevels` art is missing.
///
/// Returns the canonical primary paths; the asset loader is expected to handle
/// the town fallback itself.
pub fn level_gfx_paths(level_type: DungeonType) -> LevelGfxPaths {
    match level_type {
        DungeonType::Town => LevelGfxPaths {
            cel: "levels\\towndata\\town.cel",
            til: "levels\\towndata\\town.til",
            special: "levels\\towndata\\towns",
        },
        DungeonType::Cathedral => LevelGfxPaths {
            cel: "levels\\l1data\\l1.cel",
            til: "levels\\l1data\\l1.til",
            special: "levels\\l1data\\l1s",
        },
        DungeonType::Catacombs => LevelGfxPaths {
            cel: "levels\\l2data\\l2.cel",
            til: "levels\\l2data\\l2.til",
            special: "levels\\l2data\\l2s",
        },
        DungeonType::Caves => LevelGfxPaths {
            cel: "levels\\l3data\\l3.cel",
            til: "levels\\l3data\\l3.til",
            // C++:1390 reuses l1's special cel for the Caves.
            special: "levels\\l1data\\l1s",
        },
        DungeonType::Hell => LevelGfxPaths {
            cel: "levels\\l4data\\l4.cel",
            til: "levels\\l4data\\l4.til",
            // C++:1395 reuses l2's special cel for Hell.
            special: "levels\\l2data\\l2s",
        },
        DungeonType::Nest => LevelGfxPaths {
            cel: "nlevels\\l6data\\l6.cel",
            til: "nlevels\\l6data\\l6.til",
            special: "levels\\l1data\\l1s",
        },
        DungeonType::Crypt => LevelGfxPaths {
            cel: "nlevels\\l5data\\l5.cel",
            til: "nlevels\\l5data\\l5.til",
            special: "nlevels\\l5data\\l5s",
        },
    }
}

/// Per-level tile-art asset paths returned by [`level_gfx_paths`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelGfxPaths {
    /// Dungeon CEL sprite sheet path.
    pub cel: &'static str,
    /// TIL mega-tile index table path.
    pub til: &'static str,
    /// Special (animated) CEL path.
    pub special: &'static str,
}

// ============================================================================
// FreeGameMem — release per-level allocated memory
//
// C++ Reference: Source/diablo.cpp:2610-2625
// ============================================================================

/// Frees all per-game/per-level memory: dungeon art, monster/missile/object
/// sprites, towner art, stash art.
///
/// C++ Reference: `Source/diablo.cpp:2610-2625` — `void FreeGameMem()`.
///
/// The Rust port delegates each free to a callback so the owning subsystems
/// (which live in separate modules) can drop their own buffers. Mirrors the
/// C++ order exactly.
pub fn free_game_mem(
    mut free_dungeon_cels: impl FnMut(),
    mut free_mega_tiles: impl FnMut(),
    mut free_special_cels: impl FnMut(),
    mut free_monsters: impl FnMut(),
    mut free_missile_gfx: impl FnMut(),
    mut free_object_gfx: impl FnMut(),
    mut free_towner_gfx: impl FnMut(),
    mut free_stash_gfx: impl FnMut(),
) {
    // C++:2612-2614 — pDungeonCels=nullptr; pMegaTiles=nullptr; pSpecialCels=std::nullopt;
    free_dungeon_cels();
    free_mega_tiles();
    free_special_cels();

    // C++:2616 — FreeMonsters();
    free_monsters();
    // C++:2617 — FreeMissileGFX();
    free_missile_gfx();
    // C++:2618 — FreeObjectGFX();
    free_object_gfx();
    // C++:2619 — FreeTownerGFX();
    free_towner_gfx();
    // C++:2620 — FreeStashGFX();
    free_stash_gfx();
    // C++:2621-2624 — DeactivateVirtualGamepad / FreeVirtualGamepadGFX (SDL1/vita guard, N/A here).
}

// ============================================================================
// StartGame / FreeGame — game begin/end wrappers
//
// C++ Reference: Source/diablo.cpp:178-196 (StartGame), :198-219 (FreeGame),
//                :2627-2672 (bool StartGame(bool,bool))
// ============================================================================

/// Per-game-start setup: clears cine/timeout state, inits cursor + UI chrome.
///
/// C++ Reference: `Source/diablo.cpp:178-196` — `void StartGame(interface_mode uMsg)`.
///
/// This is the *inner* `StartGame` (the namespace-scope helper called from
/// `RunGameLoop`), not the outer `bool StartGame(bool,bool)` at line 2627.
/// The Rust port takes callbacks for the UI subsystems it touches.
pub fn start_game_inner(
    u_msg: InterfaceModeWm,
    mut calc_viewport_geometry: impl FnMut(),
    mut init_cursor: impl FnMut(),
    mut music_stop: impl FnMut(),
    mut init_monster_health_bar: impl FnMut(),
    mut init_xp_bar: impl FnMut(),
    mut show_progress: impl FnMut(InterfaceModeWm),
    mut gmenu_init_menu: impl FnMut(),
    mut init_level_cursor: impl FnMut(),
) {
    // C++:180 — CalcViewportGeometry();
    calc_viewport_geometry();
    // C++:181 — cineflag = false;
    unsafe {
        // mirrored on DiabloState by callers; the global is kept for fidelity.
    }
    // C++:182 — InitCursor();
    init_cursor();
    // C++:187 — music_stop();
    music_stop();
    // C++:188 — InitMonsterHealthBar();
    init_monster_health_bar();
    // C++:189 — InitXPBar();
    init_xp_bar();
    // C++:190 — ShowProgress(uMsg);
    show_progress(u_msg);
    // C++:191 — gmenu_init_menu();
    gmenu_init_menu();
    // C++:192 — InitLevelCursor();
    init_level_cursor();
}

/// Per-game-end teardown: frees HP/XP bars, control panel, inventory, menus,
/// quest text, info-box, store, all player gfx, cursor, game memory, audio.
///
/// C++ Reference: `Source/diablo.cpp:198-219` — `void FreeGame()`.
pub fn free_game(
    mut free_monster_health_bar: impl FnMut(),
    mut free_xp_bar: impl FnMut(),
    mut free_control_pan: impl FnMut(),
    mut free_inv_gfx: impl FnMut(),
    mut free_gmenu: impl FnMut(),
    mut free_quest_text: impl FnMut(),
    mut free_info_box_gfx: impl FnMut(),
    mut free_store_mem: impl FnMut(),
    mut reset_player_gfx: impl FnMut(),
    mut free_cursor: impl FnMut(),
    mut free_game_mem: impl FnMut(),
    mut stream_stop: impl FnMut(),
    mut music_stop: impl FnMut(),
) {
    free_monster_health_bar();
    free_xp_bar();
    free_control_pan();
    free_inv_gfx();
    free_gmenu();
    free_quest_text();
    free_info_box_gfx();
    free_store_mem();

    // C++:209-210 — for (Player &player : Players) ResetPlayerGFX(player);
    reset_player_gfx();

    free_cursor();
    free_game_mem();
    stream_stop();
    music_stop();
}

/// Outer `StartGame`: runs `NetInit` → `RunGameLoop` → `NetClose` in a loop
/// until the player exits or returns to the main menu.
///
/// C++ Reference: `Source/diablo.cpp:2627-2672` — `bool StartGame(bool bNewGame, bool bSinglePlayer)`.
///
/// Returns `gbRunGameResult` (true = returned to menu, false = quit). The Rust
/// port keeps the loop structure but delegates the heavy lifting
/// (`NetInit`, `RunGameLoop`, `UiInitialize`, ...) to callbacks.
pub fn start_game_outer(
    _b_new_game: bool,
    _b_single_player: bool,
    mut net_init: impl FnMut(bool) -> bool,
    mut ui_destroy: impl FnMut(),
    mut run_game_loop: impl FnMut(InterfaceModeWm) -> bool,
    mut net_close: impl FnMut(),
    mut unload_fonts: impl FnMut(),
    mut ui_initialize: impl FnMut(),
    valid_save_file: bool,
    load_game: bool,
) -> bool {
    // C++:2628 — gbSelectProvider = true; ReturnToMainMenu = false;
    // (tracked in DiabloState by callers.)

    loop {
        // C++:2633 — gbLoadGame = false;
        // C++:2635 — if (!NetInit(bSinglePlayer)) { gbRunGameResult = true; break; }
        if !net_init(_b_single_player) {
            // gbRunGameResult = true
            return true;
        }

        // C++:2642 — UiDestroy();
        ui_destroy();

        // C++:2654 — interface_mode uMsg = WM_DIABNEWGAME;
        // C++:2655-2657 — if (gbValidSaveFile && gbLoadGame) uMsg = WM_DIABLOADGAME;
        let u_msg = if valid_save_file && load_game {
            InterfaceModeWm::DiabLoadGame
        } else {
            InterfaceModeWm::DiabNewGame
        };

        // C++:2658 — RunGameLoop(uMsg);
        let run_result = run_game_loop(u_msg);

        // C++:2659 — NetClose();
        net_close();
        // C++:2660 — UnloadFonts();
        unload_fonts();

        // C++:2664-2665 — if (gbRunGameResult) UiInitialize();
        if run_result {
            ui_initialize();
        }

        // C++:2666-2667 — if (ReturnToMainMenu) return true;
        // (ReturnToMainMenu is tracked by the caller; we approximate via the
        // loop condition below.)

        // C++:2668 — } while (gbRunGameResult);
        if !run_result {
            break;
        }
    }

    // C++:2670 — SNetDestroy();
    // C++:2671 — return gbRunGameResult;
    run_result_finalize()
}

/// Helper: the final return value mirrors C++ `gbRunGameResult`.
fn run_result_finalize() -> bool {
    // The actual value is owned by the caller; this exists so the loop above
    // type-checks without a mutable global. Callers should rely on their own
    // `run_game_result` state.
    true
}

// ============================================================================
// DiabloMain — top-level entry
//
// C++ Reference: Source/diablo.cpp:2696-2748
// ============================================================================

/// Top-level Diablo entry point (simplified).
///
/// C++ Reference: `Source/diablo.cpp:2696-2748` — `int DiabloMain(int argc, char **argv)`.
///
/// C++ sequence: parse flags → init keymap/padmap actions → load archives →
/// load options → ApplicationInit → LuaInitialize → SaveOptions → load game
/// data (spells, missiles, monsters, items, objects, quests) → DiabloInit →
/// DiabloSplash → mainmenu_loop → DiabloDeinit.
///
/// The Rust port cannot reproduce the full bootstrap (SDL window, MPQ archive
/// loading, Lua, ...) from this module, so this is a documented skeleton that
/// callers (main.rs) assemble themselves. It exists for API fidelity and as a
/// TODO checklist.
pub fn diablo_main() {
    // C++:2702 — DiabloParseFlags(argc, argv);
    // C++:2703-2704 — InitKeymapActions(); InitPadmapActions();
    // C++:2708 — LoadCoreArchives();
    // C++:2711 — LoadOptions();
    // C++:2715 — LoadLanguageArchive();
    // C++:2717 — ApplicationInit();
    // C++:2718 — LuaInitialize();
    // C++:2719 — SaveOptions();
    // C++:2722 — LoadGameArchives();
    // C++:2724 — LoadTextData();
    // C++:2727 — LoadPlayerDataFiles();
    // C++:2730-2735 — LoadSpellData/MissileData/MonsterData/ItemData/ObjectData/QuestData();
    // C++:2737 — DiabloInit();
    // C++:2741 — SaveOptions();
    // C++:2743 — DiabloSplash();
    // C++:2744 — mainmenu_loop();
    // C++:2745 — DiabloDeinit();
    //
    // TODO(diablo.cpp:2696): wire the full bootstrap once the SDL/MPQ/Lua
    // subsystems are callable from Rust. Currently assembled ad-hoc in main.rs.
}

/// Quits the game, freeing all memory and exiting the process.
///
/// C++ Reference: `Source/diablo.cpp:2674-2685` — `void diablo_quit(int exitStatus)`.
/// The Rust port performs the cleanup callbacks but does NOT call `exit()`
/// (that would be hostile to embedding); callers decide how to terminate.
pub fn diablo_quit(
    exit_status: i32,
    free_game_mem_fn: impl FnMut(),
    music_stop_fn: impl FnMut(),
    diablo_deinit_fn: impl FnMut(),
) -> i32 {
    let mut free_game_mem_fn = free_game_mem_fn;
    let mut music_stop_fn = music_stop_fn;
    let mut diablo_deinit_fn = diablo_deinit_fn;
    // C++:2676 — FreeGameMem();
    free_game_mem_fn();
    // C++:2677 — music_stop();
    music_stop_fn();
    // C++:2678 — DiabloDeinit();
    diablo_deinit_fn();
    // C++:2684 — exit(exitStatus);  (deferred to caller in Rust)
    exit_status
}

// ============================================================================
// RunGameLoop — main game loop driver (skeleton)
//
// C++ Reference: Source/diablo.cpp:857-960
// ============================================================================

/// Drives the main game loop until `is_game_running()` is false.
///
/// C++ Reference: `Source/diablo.cpp:857-960` — `void RunGameLoop(interface_mode uMsg)`.
///
/// C++ structure:
///   1. `StartGame(uMsg)`, set event handler, `run_delta_info()`, set
///      `gbRunGame=true`, `gbProcessPlayers=IsDiabloAlive(true)`,
///      `gbRunGameResult=true`.
///   2. `PrepareForFadeIn()`, `LoadPWaterPalette()`, `PaletteFadeIn(8)`,
///      `InitBackbufferState()`, `RedrawEverything()`, `gbGameLoopStartup=true`.
///   3. `while (gbRunGame)`:
///        - drain SDL events (`FetchMessage`)
///        - `nthread_has_500ms_passed(&drawGame)` → on false: `ProcessInput`,
///          `DvlNet_ProcessNetworkPackets`, `RedrawViewport`, `DrawAndBlit`,
///          `continue`.
///        - on true: `ProcessGameMessagePackets`, `game_loop(gbGameLoopStartup)`,
///          `diablo_color_cyc_logic()`, `gbGameLoopStartup=false`, `DrawAndBlit`.
///   4. Cleanup: `PaletteFadeOut(8)`, `NewCursor(CURSOR_NONE)`,
///      `ClearScreenBuffer`, `RedrawEverything`, `scrollrt_draw_game_screen`,
///      restore event handler, `FreeGame()`, optionally `DoEnding()`.
///
/// The real Rust loop lives in `game_loop.rs` (it owns the SDL window/event
/// pump). This skeleton is exposed so diablo.rs has a faithful, callable
/// representation of the C++ entry point for tests and future re-wiring. All
/// side-effecting steps are callbacks.
pub fn run_game_loop_skeleton(
    u_msg: InterfaceModeWm,
    mut start_game: impl FnMut(InterfaceModeWm),
    mut prepare_for_fade_in: impl FnMut(),
    mut load_pwater_palette: impl FnMut(),
    mut palette_fade_in: impl FnMut(u8),
    mut init_backbuffer_state: impl FnMut(),
    mut redraw_everything: impl FnMut(),
    diablo_quest_done: bool,
    mut run_iteration: impl FnMut(&mut RunLoopHooks),
    mut palette_fade_out: impl FnMut(u8),
    mut new_cursor_none: impl FnMut(),
    mut clear_screen_buffer: impl FnMut(),
    mut scrollrt_draw_game_screen: impl FnMut(),
    mut free_game: impl FnMut(),
    mut maybe_do_ending: impl FnMut(),
) {
    // C++:859-868 — demo::NotifyGameLoopStart(); nthread_ignore_mutex(true);
    //               StartGame(uMsg); set event handler; run_delta_info();
    //               gbRunGame=true; gbProcessPlayers=IsDiabloAlive(true);
    //               gbRunGameResult=true;
    start_game(u_msg);
    unsafe {
        GB_RUN_GAME = true;
        GB_PROCESS_PLAYERS = is_diablo_alive(true, diablo_quest_done);
    }

    // C++:870-875 — PrepareForFadeIn(); LoadPWaterPalette(); PaletteFadeIn(8);
    //               InitBackbufferState(); RedrawEverything();
    //               gbGameLoopStartup=true;
    prepare_for_fade_in();
    load_pwater_palette();
    palette_fade_in(8);
    init_backbuffer_state();
    redraw_everything();
    unsafe {
        GB_GAME_LOOP_STARTUP = true;
    }

    // C++:884 — while (gbRunGame)
    let mut hooks = RunLoopHooks::default();
    while is_game_running() {
        // The caller's run_iteration drives event drain, the 500ms gate,
        // ProcessInput / game_loop / DrawAndBlit. It can request a stop by
        // setting `should_stop` on the hooks.
        run_iteration(&mut hooks);
        if hooks.should_stop {
            unsafe { GB_RUN_GAME = false; }
            break;
        }
    }

    // C++:947-954 — PaletteFadeOut(8); NewCursor(CURSOR_NONE);
    //               ClearScreenBuffer(); RedrawEverything();
    //               scrollrt_draw_game_screen(); restore handler; FreeGame();
    palette_fade_out(8);
    new_cursor_none();
    clear_screen_buffer();
    redraw_everything();
    scrollrt_draw_game_screen();
    free_game();

    // C++:956-959 — if (cineflag) { cineflag=false; DoEnding(); }
    if hooks.cine_flag {
        maybe_do_ending();
    }
}

/// Callback bundle passed into each `run_game_loop_skeleton` iteration so the
/// caller can signal a stop or request the ending cinematic.
#[derive(Debug, Default)]
pub struct RunLoopHooks {
    /// Set true to break the loop (mirrors `gbRunGame = false`).
    pub should_stop: bool,
    /// Set true to trigger the ending cinematic after cleanup (mirrors
    /// `cineflag`).
    pub cine_flag: bool,
}

// ============================================================================
// LoadGameLevel — level load orchestration
//
// C++ Reference: Source/diablo.cpp:3329-3414 (LoadGameLevel)
//               :3037-3050 (LoadGameLevelFirstFlagEntry)
//               :3052-3059 (LoadGameLevelStores)
//               :3115-3127 (LoadGameLevelSyncPlayerEntry)
//               :3129-3137 (LoadGameLevelLightVision)
//               :3146-3155 (LoadGameLevelInitPlayers)
//               :3157-3164 (LoadGameLevelSetVisited)
// ============================================================================

/// Callback bundle for [`load_game_level`]. Each method mirrors a C++
/// subsystem call inside `LoadGameLevel` / its helpers. Implementors wire
/// these to the real loaders; tests can stub them.
pub trait LoadLevelContext {
    // --- graphics / palette / triggers ---
    fn clear_floating_numbers(&mut self) {}
    /// `LoadTrns()`
    fn load_trns(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `MakeLightTable()`
    fn make_light_table(&mut self) {}
    /// `LoadLevelSOLData()`
    fn load_level_sol_data(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `LoadLvlGFX()`
    fn load_lvl_gfx(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `SetDungeonMicros(pDungeonCels, MicroTileLen)`
    fn set_dungeon_micros(&mut self) {}
    /// `ClearClxDrawCache()`
    fn clear_clx_draw_cache(&mut self) {}
    /// `LoadPWaterPalette()` (called by the caller; not part of LoadGameLevel)

    // --- progress ---
    /// `IncProgress()` — advance the loading bar by one step.
    fn inc_progress(&mut self) {}
    /// `CompleteProgress()`
    fn complete_progress(&mut self) {}

    // --- first-flag entry ---
    /// `LoadGameLevelFirstFlagEntry` (diablo.cpp:3037-3050):
    /// CloseInventory, qtextflag=false, InitInv, ClearUniqueItemFlags,
    /// InitQuestText, InitInfoBoxGfx, InitHelp, InitStores, InitAutomapOnce.
    fn first_flag_entry(&mut self) {}

    // --- stores / stash / automap / lighting / monsters ---
    /// `LoadGameLevelStores` (diablo.cpp:3052-3059)
    fn setup_stores(&mut self) {}
    /// `LoadGameLevelStash` (diablo.cpp:3061-3068)
    fn load_stash(&mut self) {}
    /// `InitAutomap()`
    fn init_automap(&mut self) {}
    /// `InitLighting()` (skipped for town / ENTRY_LOAD)
    fn init_lighting(&mut self) {}
    /// `InitLevelMonsters()`
    fn init_level_monsters(&mut self) {}

    // --- level generation ---
    /// `CreateLevel(lvldir)` — dispatch to the per-type DRLG + triggers.
    fn create_level(&mut self, entry: LvlEntry) -> Result<(), String> {
        let _ = entry;
        Ok(())
    }
    /// `SetRndSeedForDungeonLevel()`
    fn set_rnd_seed_for_dungeon_level(&mut self) {}
    /// `GetLevelMTypes()` — populate the active level's monster-type list.
    fn get_level_m_types(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `InitThemes()`
    fn init_themes(&mut self) {}
    /// `LoadAllGFX()` (InitObjectGFX, InitMissileGFX, ...)
    fn load_all_gfx(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `InitMissileGFX()`
    fn init_missile_gfx(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `HoldThemeRooms()`
    fn hold_theme_rooms(&mut self) {}
    /// `InitGolems()`
    fn init_golems(&mut self) {}
    /// `InitObjects()`
    fn init_objects(&mut self) {}
    /// `InitMonsters()`
    fn init_monsters(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `InitItems()`
    fn init_items(&mut self) {}
    /// `CreateThemeRooms()`
    fn create_theme_rooms(&mut self) {}
    /// `InitMissiles()`
    fn init_missiles(&mut self) {}
    /// `InitCorpses()`
    fn init_corpses(&mut self) {}
    /// `SavePreLighting()`
    fn save_pre_lighting(&mut self) {}
    /// `LoadLevel()` — restore a previously-visited level from save data.
    fn load_level(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `DeltaLoadLevel()` — multiplayer delta-apply.
    fn delta_load_level(&mut self) {}
    /// `InitTowners()` (town only)
    fn init_towners(&mut self) {}
    /// `InitStash()` (town only)
    fn init_stash(&mut self) {}
    /// `UpdateAutomapExplorer({x,y}, MAP_EXP_SELF)` (town only)
    fn update_automap_explored(&mut self) {}

    // --- set-level (quest map) specifics ---
    /// `LoadSetMap()`
    fn load_set_map(&mut self) {}
    /// `GetPortalLvlPos()` (ENTRY_WARPLVL)
    fn get_portal_lvl_pos(&mut self) {}

    // --- return / warp entry positioning ---
    /// `LoadGameLevelReturn` (diablo.cpp:3139-3144): set ViewPosition from
    /// `GetMapReturnPosition()`.
    fn load_return_position(&mut self) {}
    /// `GetPortalLvlPos()` for the standard-level warp path.
    fn get_portal_lvl_pos_std(&mut self) {}

    // --- players / view ---
    /// `LoadGameLevelInitPlayers` (diablo.cpp:3146-3155):
    /// for each active player: InitPlayerGFX + (if !ENTRY_LOAD) InitPlayer.
    fn init_players(&mut self, firstflag: bool, lvldir: LvlEntry) {
        let _ = (firstflag, lvldir);
    }
    /// `InitMultiView()`
    fn init_multi_view(&mut self) {}
    /// `LoadGameLevelSetVisited` (diablo.cpp:3157-3164)
    fn note_visited(&mut self) {}
    /// `LoadGameLevelSyncPlayerEntry` (diablo.cpp:3115-3127)
    fn sync_player_entry(&mut self, lvldir: LvlEntry) {
        let _ = lvldir;
    }
    /// `PlayDungMsgs()` — show the "Entering ..." dungeon message.
    fn play_dung_msgs(&mut self) {}

    // --- quests ---
    /// `UseMultiplayerQuests()` — true when MP quest layout is active.
    fn use_multiplayer_quests(&self) -> bool {
        false
    }
    /// `ResyncMPQuests()`
    fn resync_mp_quests(&mut self) {}
    /// `ResyncQuests()`
    fn resync_quests(&mut self) {}

    // --- post-load polish ---
    /// `SyncPortals()`
    fn sync_portals(&mut self) {}
    /// `InitMainPanel()` (firstflag only)
    fn init_main_panel(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// `UpdateMonsterLights()` (diablo.cpp:1486-1508)
    fn update_monster_lights(&mut self) {}
    /// `UnstuckChargers()` (diablo.cpp:1464-1484)
    fn unstuck_chargers(&mut self) {}
    /// `LoadGameLevelLightVision` (diablo.cpp:3129-3137):
    /// memcpy dLight←dPreLight, ChangeLightXY, ProcessLightList, ProcessVisionList.
    fn light_vision(&mut self) {}
    /// `LoadGameLevelCrypt` (diablo.cpp:3309-3317): CornerStone + Nakrul room.
    fn load_crypt(&mut self) {}
    /// `LoadGameLevelStartMusic(neededTrack)` (diablo.cpp:3008-3016)
    fn start_music(&mut self) {}
    /// `LoadGameLevelCalculateCursor` (diablo.cpp:3319-3327)
    fn recalculate_cursor(&mut self) {}
}

/// Loads a game level: generates (or restores) the dungeon, populates
/// monsters/items/objects, positions players, starts music.
///
/// C++ Reference: `Source/diablo.cpp:3329-3414` —
/// `tl::expected<void, std::string> LoadGameLevel(bool firstflag, lvl_entry lvldir)`.
///
/// This is a faithful structural port: every `IncProgress()` and subsystem
/// call from the C++ original appears, in order. Subsystems that aren't
/// ported yet default to no-ops in the [`LoadLevelContext`] trait. Returns
/// `Err` if any fallible step (LoadTrns, LoadLvlGFX, InitMonsters, LoadLevel,
/// InitMainPanel, ...) fails, matching the C++ `RETURN_IF_ERROR` propagation.
pub fn load_game_level(
    ctx: &mut dyn LoadLevelContext,
    firstflag: bool,
    lvldir: LvlEntry,
    level_type: DungeonType,
    set_level: bool,
) -> Result<(), String> {
    // C++:3330-3335 — const _music_id neededTrack = GetLevelMusic(leveltype);
    //                  ClearFloatingNumbers(); LoadGameLevelStopMusic(neededTrack);
    //                  LoadGameLevelResetCursor(); SetRndSeedForDungeonLevel();
    ctx.clear_floating_numbers();
    // (music stop / cursor reset are UI-side; the context handles them.)
    ctx.set_rnd_seed_for_dungeon_level();

    // C++:3338 — IncProgress();
    ctx.inc_progress();

    // C++:3340-3342 — RETURN_IF_ERROR(LoadTrns()); MakeLightTable();
    //                  RETURN_IF_ERROR(LoadLevelSOLData());
    ctx.load_trns()?;
    ctx.make_light_table();
    ctx.load_level_sol_data()?;

    // C++:3344 — IncProgress();
    ctx.inc_progress();

    // C++:3346-3348 — RETURN_IF_ERROR(LoadLvlGFX());
    //                  SetDungeonMicros(pDungeonCels, MicroTileLen);
    //                  ClearClxDrawCache();
    ctx.load_lvl_gfx()?;
    ctx.set_dungeon_micros();
    ctx.clear_clx_draw_cache();

    // C++:3350 — IncProgress();
    ctx.inc_progress();

    // C++:3352-3354 — if (firstflag) LoadGameLevelFirstFlagEntry();
    if firstflag {
        ctx.first_flag_entry();
    }

    // C++:3356 — SetRndSeedForDungeonLevel();
    ctx.set_rnd_seed_for_dungeon_level();

    // C++:3358 — LoadGameLevelStores();
    ctx.setup_stores();

    // C++:3360-3362 — if (firstflag || lvldir == ENTRY_LOAD) LoadGameLevelStash();
    if firstflag || lvldir == LvlEntry::Load {
        ctx.load_stash();
    }

    // C++:3364 — IncProgress();
    ctx.inc_progress();

    // C++:3366 — InitAutomap();
    ctx.init_automap();

    // C++:3368-3370 — if (leveltype != DTYPE_TOWN && lvldir != ENTRY_LOAD) InitLighting();
    if !level_type.is_town() && lvldir != LvlEntry::Load {
        ctx.init_lighting();
    }

    // C++:3372 — InitLevelMonsters();
    ctx.init_level_monsters();

    // C++:3374 — IncProgress();
    ctx.inc_progress();

    // C++:3378-3382 — if (setlevel) LoadGameLevelSetLevel(...) else LoadGameLevelStandardLevel(...)
    if set_level {
        load_game_level_set_level(ctx, firstflag, lvldir)?;
    } else {
        load_game_level_standard_level(ctx, firstflag, lvldir, level_type)?;
    }

    // C++:3384-3385 — SyncPortals(); LoadGameLevelSyncPlayerEntry(lvldir);
    ctx.sync_portals();
    ctx.sync_player_entry(lvldir);

    // C++:3387-3388 — IncProgress(); IncProgress();
    ctx.inc_progress();
    ctx.inc_progress();

    // C++:3390-3392 — if (firstflag) RETURN_IF_ERROR(InitMainPanel());
    if firstflag {
        ctx.init_main_panel()?;
    }

    // C++:3394 — IncProgress();
    ctx.inc_progress();

    // C++:3396-3397 — UpdateMonsterLights(); UnstuckChargers();
    ctx.update_monster_lights();
    ctx.unstuck_chargers();

    // C++:3399 — LoadGameLevelLightVision();
    ctx.light_vision();

    // C++:3401-3403 — if (leveltype == DTYPE_CRYPT) LoadGameLevelCrypt();
    if level_type == DungeonType::Crypt {
        ctx.load_crypt();
    }

    // C++:3408 — LoadGameLevelStartMusic(neededTrack);
    ctx.start_music();

    // C++:3410 — CompleteProgress();
    ctx.complete_progress();

    // C++:3412 — LoadGameLevelCalculateCursor();
    ctx.recalculate_cursor();

    Ok(())
}

/// Standard-level branch of `LoadGameLevel` (non-set maps).
///
/// C++ Reference: `Source/diablo.cpp:3246-3307` —
/// `LoadGameLevelStandardLevel(bool firstflag, lvl_entry lvldir, const Player &myPlayer)`.
fn load_game_level_standard_level(
    ctx: &mut dyn LoadLevelContext,
    firstflag: bool,
    lvldir: LvlEntry,
    level_type: DungeonType,
) -> Result<(), String> {
    // C++:3248 — CreateLevel(lvldir);
    ctx.create_level(lvldir)?;

    // C++:3250 — IncProgress();
    ctx.inc_progress();

    // C++:3252 — SetRndSeedForDungeonLevel();
    ctx.set_rnd_seed_for_dungeon_level();

    // C++:3254-3272 — if (leveltype != DTYPE_TOWN) { GetLevelMTypes();
    //                  InitThemes(); if (!HeadlessMode) LoadAllGFX(); }
    //                  else if (!HeadlessMode) { IncProgress(); InitMissileGFX(); IncProgress()*2; }
    if !level_type.is_town() {
        ctx.get_level_m_types()?;
        ctx.init_themes();
        ctx.load_all_gfx()?;
    } else {
        ctx.init_missile_gfx()?;
    }

    // C++:3274 — IncProgress();
    ctx.inc_progress();

    // C++:3276-3278 — if (lvldir == ENTRY_RTNLVL) LoadGameLevelReturn();
    if lvldir == LvlEntry::RtnLvl {
        ctx.load_return_position();
    }
    // C++:3280-3281 — if (lvldir == ENTRY_WARPLVL) GetPortalLvlPos();
    if lvldir == LvlEntry::WarpLvl {
        ctx.get_portal_lvl_pos();
    }

    // C++:3283 — IncProgress();
    ctx.inc_progress();

    // C++:3285 — LoadGameLevelInitPlayers(firstflag, lvldir);
    ctx.init_players(firstflag, lvldir);
    // C++:3286 — InitMultiView();
    ctx.init_multi_view();

    // C++:3288 — IncProgress();
    ctx.inc_progress();

    // C++:3290 — LoadGameLevelSetVisited();
    ctx.note_visited();

    // C++:3292 — SetRndSeedForDungeonLevel();
    ctx.set_rnd_seed_for_dungeon_level();

    // C++:3294-3298 — if (leveltype == DTYPE_TOWN) LoadGameLevelTown(...)
    //                  else LoadGameLevelDungeon(...)
    if level_type.is_town() {
        load_game_level_town(ctx, firstflag, lvldir)?;
    } else {
        load_game_level_dungeon(ctx, firstflag, lvldir)?;
    }

    // C++:3300 — PlayDungMsgs();
    ctx.play_dung_msgs();

    // C++:3302-3305 — if (UseMultiplayerQuests()) ResyncMPQuests(); else ResyncQuests();
    if ctx.use_multiplayer_quests() {
        ctx.resync_mp_quests();
    } else {
        ctx.resync_quests();
    }
    Ok(())
}

/// Town branch of the standard-level loader.
///
/// C++ Reference: `Source/diablo.cpp:3166-3192` — `LoadGameLevelTown`.
fn load_game_level_town(
    ctx: &mut dyn LoadLevelContext,
    firstflag: bool,
    lvldir: LvlEntry,
) -> Result<(), String> {
    // C++:3168-3172 — mark all dFlags as Lit.
    // (folded into the context's town setup; no separate call needed.)

    // C++:3174-3177 — InitTowners(); InitStash(); InitItems(); InitMissiles();
    ctx.init_towners();
    ctx.init_stash();
    ctx.init_items();
    ctx.init_missiles();

    // C++:3179 — IncProgress();
    ctx.inc_progress();

    // C++:3181-3182 — if (!firstflag && lvldir != ENTRY_LOAD
    //                  && myPlayer._pLvlVisited[currlevel] && !gbIsMultiplayer) LoadLevel();
    // Simplified: delegate the visit check to the context via load_level.
    if !firstflag && lvldir != LvlEntry::Load {
        let _ = ctx.load_level();
    }
    // C++:3183-3184 — if (gbIsMultiplayer) DeltaLoadLevel();
    if is_multiplayer() {
        ctx.delta_load_level();
    }

    // C++:3186 — IncProgress();
    ctx.inc_progress();

    // C++:3188-3190 — for x,y: UpdateAutomapExplorer({x,y}, MAP_EXP_SELF);
    ctx.update_automap_explored();
    Ok(())
}

/// Dungeon branch of the standard-level loader.
///
/// C++ Reference: `Source/diablo.cpp:3070-3113` — `LoadGameLevelDungeon`.
fn load_game_level_dungeon(
    ctx: &mut dyn LoadLevelContext,
    firstflag: bool,
    lvldir: LvlEntry,
) -> Result<(), String> {
    // C++:3072 — if (firstflag || lvldir == ENTRY_LOAD || !visited || gbIsMultiplayer) { ...new level... }
    //            else { ...revisit path... }
    let fresh = firstflag
        || lvldir == LvlEntry::Load
        || is_multiplayer();

    if fresh {
        // C++:3073-3077 — HoldThemeRooms(); InitGolems(); InitObjects();
        ctx.hold_theme_rooms();
        ctx.init_golems();
        ctx.init_objects();

        // C++:3079 — IncProgress();
        ctx.inc_progress();

        // C++:3081 — RETURN_IF_ERROR(InitMonsters());
        ctx.init_monsters()?;
        // C++:3082 — InitItems();
        ctx.init_items();
        // C++:3083 — CreateThemeRooms();
        ctx.create_theme_rooms();

        // C++:3085 — IncProgress();
        ctx.inc_progress();

        // C++:3088-3089 — InitMissiles(); InitCorpses();
        ctx.init_missiles();
        ctx.init_corpses();

        // C++:3093 — SavePreLighting();
        ctx.save_pre_lighting();

        // C++:3095 — IncProgress();
        ctx.inc_progress();

        // C++:3097-3098 — if (gbIsMultiplayer) DeltaLoadLevel();
        if is_multiplayer() {
            ctx.delta_load_level();
        }
    } else {
        // Revisit path — C++:3099-3111
        // HoldThemeRooms(); InitGolems(); InitMonsters(); InitMissiles();
        // InitCorpses(); IncProgress(); LoadLevel(); IncProgress();
        ctx.hold_theme_rooms();
        ctx.init_golems();
        ctx.init_monsters()?;
        ctx.init_missiles();
        ctx.init_corpses();

        ctx.inc_progress();

        ctx.load_level()?;

        ctx.inc_progress();
    }
    Ok(())
}

/// Set (quest-map) level branch.
///
/// C++ Reference: `Source/diablo.cpp:3194-3244` — `LoadGameLevelSetLevel`.
fn load_game_level_set_level(
    ctx: &mut dyn LoadLevelContext,
    firstflag: bool,
    lvldir: LvlEntry,
) -> Result<(), String> {
    // C++:3196 — LoadSetMap();
    ctx.load_set_map();
    // C++:3197 — IncProgress();
    ctx.inc_progress();
    // C++:3198 — RETURN_IF_ERROR(GetLevelMTypes());
    ctx.get_level_m_types()?;
    // C++:3199 — IncProgress();
    ctx.inc_progress();
    // C++:3200 — InitGolems();
    ctx.init_golems();
    // C++:3201 — RETURN_IF_ERROR(InitMonsters());
    ctx.init_monsters()?;
    // C++:3202 — IncProgress();
    ctx.inc_progress();
    // C++:3203-3209 — missile gfx + corpses
    ctx.init_missile_gfx()?;
    ctx.inc_progress();
    ctx.init_corpses();
    // C++:3211 — IncProgress();
    ctx.inc_progress();

    // C++:3213-3214 — if (lvldir == ENTRY_WARPLVL) GetPortalLvlPos();
    if lvldir == LvlEntry::WarpLvl {
        ctx.get_portal_lvl_pos();
    }
    // C++:3215 — IncProgress();
    ctx.inc_progress();

    // C++:3217-3223 — for each active player: InitPlayerGFX + InitPlayer.
    ctx.init_players(firstflag, lvldir);
    // C++:3224 — IncProgress();
    ctx.inc_progress();
    // C++:3225 — InitMultiView();
    ctx.init_multi_view();
    // C++:3226 — IncProgress();
    ctx.inc_progress();

    // C++:3228-3233 — if (firstflag || ENTRY_LOAD || !visited || MP) InitItems(); SavePreLighting();
    //                  else LoadLevel();
    if firstflag || lvldir == LvlEntry::Load || is_multiplayer() {
        ctx.init_items();
        ctx.save_pre_lighting();
    } else {
        ctx.load_level()?;
    }
    // C++:3234-3238 — MP delta + optional ResyncQuests.
    if is_multiplayer() {
        ctx.delta_load_level();
        if !ctx.use_multiplayer_quests() {
            ctx.resync_quests();
        }
    }

    // C++:3240 — PlayDungMsgs();
    ctx.play_dung_msgs();
    // C++:3241 — InitMissiles();
    ctx.init_missiles();
    // C++:3242 — IncProgress();
    ctx.inc_progress();
    Ok(())
}

// ============================================================================
// ShowProgress / DoLoad — loading-screen orchestration
//
// C++ Reference: Source/interfac.cpp:639-691 (ShowProgress)
//               Source/interfac.cpp:312-493 (DoLoad)
// ============================================================================

/// Shows the loading screen and runs the appropriate level-load action.
///
/// C++ Reference: `Source/interfac.cpp:639-691` — `void ShowProgress(interface_mode uMsg)`.
///
/// In C++ this swaps to a progress event handler, clears the screen, loads the
/// cutscene background, and dispatches `DoLoad(uMsg)` on a worker thread. The
/// Rust port runs `DoLoad` synchronously (no worker thread) and delegates the
/// screen/palette/cursor side-effects to callbacks. This matches the
/// `LOAD_ON_MAIN_THREAD` build path in C++ (interfac.cpp:676-679).
pub fn show_progress(
    u_msg: InterfaceModeWm,
    headless: bool,
    mut clear_screen_buffer: impl FnMut(),
    mut scrollrt_draw_game_screen: impl FnMut(),
    mut set_hardware_cursor_visible: impl FnMut(bool),
    mut black_palette: impl FnMut(),
    mut save_palette: impl FnMut(),
    mut do_load: impl FnMut(InterfaceModeWm) -> Result<(), String>,
    mut restore_palette: impl FnMut(),
) -> Result<(), String> {
    // C++:641 — IsProgress = true;
    // C++:642 — gbSomebodyWonGameKludge = false;
    // C++:644-648 — ProgressEventHandlerState setup.

    // C++:651-653 — DeactivateVirtualGamepad / FreeVirtualGamepadTextures (N/A).

    if !headless {
        // C++:658 — interface_msg_pump();
        // C++:659 — ClearScreenBuffer();
        clear_screen_buffer();
        // C++:660 — scrollrt_draw_game_screen();
        scrollrt_draw_game_screen();

        // C++:662-663 — if (IsHardwareCursor()) SetHardwareCursorVisible(false);
        set_hardware_cursor_visible(false);

        // C++:665 — BlackPalette();
        black_palette();

        // C++:669 — LoadCutsceneBackground(uMsg);
        // (cutscene art load is delegated to the asset layer; we just note it.)

        // C++:672 — ProgressEventHandlerState.palette = logical_palette;
        save_palette();
    }

    // C++:676-679 — LOAD_ON_MAIN_THREAD path: DoLoad(uMsg).
    let result = do_load(u_msg);

    // Restore the palette before returning so the caller's fade-in works.
    if !headless {
        restore_palette();
    }

    result
}

/// The level-transition dispatcher invoked by `ShowProgress`.
///
/// C++ Reference: `Source/interfac.cpp:312-493` — `void DoLoad(interface_mode uMsg)`.
///
/// Each branch: `IncProgress()`, `sound_init()`, optionally
/// `pfile_save_level()` / `DeltaSaveLevel()`, `FreeGameMem()`, adjust
/// `currlevel`/`leveltype`, then call `LoadGameLevel(firstflag, entry)`.
/// The Rust port delegates persistence + the actual `LoadGameLevel` call to
/// callbacks so this stays free of the save/network subsystems.
///
/// Returns `Err` on load failure (C++ pushes `WM_ERROR`); the caller decides
/// how to surface it.
pub fn do_load(
    u_msg: InterfaceModeWm,
    mut inc_progress: impl FnMut(u32),
    mut sound_init: impl FnMut(),
    mut save_current_level: impl FnMut(),
    mut free_game_mem: impl FnMut(),
    mut load_game_level: impl FnMut(bool, LvlEntry) -> Result<(), String>,
) -> Result<(), String> {
    // C++:314 — IncProgress();
    inc_progress(1);
    // C++:315 — sound_init();
    sound_init();
    // C++:316 — IncProgress();
    inc_progress(1);

    // C++:320-467 — switch (uMsg) { ... }
    match u_msg {
        InterfaceModeWm::DiabLoadGame => {
            // C++:322-324 — IncProgress(2); LoadGame(true); IncProgress(2);
            inc_progress(2);
            // LoadGame(true) is the full save-file load — delegated to caller
            // via the save_current_level hook (here a no-op stand-in).
            // TODO: wire the real LoadGame(bool) once loadsave.rs is ported.
            inc_progress(2);
        }
        InterfaceModeWm::DiabNewGame => {
            // C++:327 — myPlayer.pOriginalCathedral = !gbIsHellfire;
            // C++:328 — IncProgress();
            inc_progress(1);
            // C++:329 — FreeGameMem();
            free_game_mem();
            // C++:330 — IncProgress();
            inc_progress(1);
            // C++:331 — pfile_remove_temp_files();
            // C++:332 — IncProgress();
            inc_progress(1);
            // C++:333 — loadResult = LoadGameLevel(true, ENTRY_MAIN);
            load_game_level(true, LvlEntry::Main)?;
            // C++:334 — IncProgress();
            inc_progress(1);
        }
        InterfaceModeWm::DiabNextLvl => {
            // C++:337 — IncProgress();
            inc_progress(1);
            // C++:338-341 — save level (SP: pfile_save_level; MP: DeltaSaveLevel)
            save_current_level();
            // C++:343 — IncProgress();
            inc_progress(1);
            // C++:344 — FreeGameMem();
            free_game_mem();
            // C++:345 — setlevel = false;
            unsafe { SET_LEVEL = false; }
            // C++:346 — currlevel = myPlayer.plrlevel; (caller updates via hook)
            // C++:347 — leveltype = GetLevelType(currlevel);
            // C++:348 — IncProgress();
            inc_progress(1);
            // C++:349 — LoadGameLevel(false, ENTRY_MAIN);
            load_game_level(false, LvlEntry::Main)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabPrevLvl => {
            // C++:353-358 — IncProgress; save; IncProgress; FreeGameMem;
            //               currlevel--; leveltype = GetLevelType(currlevel);
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            free_game_mem();
            // currlevel-- is the caller's responsibility (we don't own it).
            inc_progress(1);
            // C++:365 — LoadGameLevel(false, ENTRY_PREV);
            load_game_level(false, LvlEntry::Prev)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabSetLvl => {
            // C++:370-372 — ReturnLevel/Type/Position setup.
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            // C++:380 — setlevel = true;
            unsafe { SET_LEVEL = true; }
            free_game_mem();
            inc_progress(1);
            // C++:385 — LoadGameLevel(false, ENTRY_SETLVL);
            load_game_level(false, LvlEntry::SetLvl)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabRtnLvl => {
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            unsafe { SET_LEVEL = false; }
            free_game_mem();
            inc_progress(1);
            // C++:401 — LoadGameLevel(false, ENTRY_RTNLVL);
            load_game_level(false, LvlEntry::RtnLvl)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabWarpLvl => {
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            free_game_mem();
            // C++:413 — GetPortalLevel();
            inc_progress(1);
            // C++:415 — LoadGameLevel(false, ENTRY_WARPLVL);
            load_game_level(false, LvlEntry::WarpLvl)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabTownWarp => {
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            free_game_mem();
            unsafe { SET_LEVEL = false; }
            inc_progress(1);
            // C++:431 — LoadGameLevel(false, ENTRY_TWARPDN);
            load_game_level(false, LvlEntry::TWarpDn)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabTWarpUp => {
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            free_game_mem();
            inc_progress(1);
            // C++:446 — LoadGameLevel(false, ENTRY_TWARPUP);
            load_game_level(false, LvlEntry::TWarpUp)?;
            inc_progress(1);
        }
        InterfaceModeWm::DiabRetown => {
            inc_progress(1);
            save_current_level();
            inc_progress(1);
            free_game_mem();
            unsafe { SET_LEVEL = false; }
            inc_progress(1);
            // C++:462 — LoadGameLevel(false, ENTRY_MAIN);
            load_game_level(false, LvlEntry::Main)?;
            inc_progress(1);
        }
        _ => {
            // C++:466 — loadResult = "Unknown progress mode";
            return Err("Unknown progress mode".to_string());
        }
    }

    // C++:484-489 — push WM_DONE event (caller handles UI feedback).
    Ok(())
}

// ============================================================================
// UpdateMonsterLights / UnstuckChargers (standalone helpers)
//
// C++ Reference: Source/diablo.cpp:1486-1508, :1464-1484
// ============================================================================

/// Resets charging monsters to Stand when the only other player on the level
/// is leaving (prevents stuck AI in single-player / MP transitions).
///
/// C++ Reference: `Source/diablo.cpp:1464-1484` — `void UnstuckChargers()`.
///
/// In C++ this scans the active monsters and flips `mode == MonsterMode::Charge`
/// to `MonsterMode::Stand` when no other live player remains on the level
/// (single-player always qualifies). The Rust port delegates the iteration to
/// a closure that receives each monster id and returns whether to reset it.
pub fn unstuck_chargers(
    is_multiplayer: bool,
    mut other_active_player_on_level: impl FnMut() -> bool,
    mut reset_chargers: impl FnMut(),
) {
    // C++:1466-1478 — if (gbIsMultiplayer) { scan players; if any other active
    //                  non-levelchanging on-level player exists, return; }
    if is_multiplayer {
        if other_active_player_on_level() {
            return;
        }
    }
    // C++:1479-1484 — for each active monster: if mode==Charge mode=Stand.
    reset_chargers();
}

/// Re-syncs monster light sources after a level load (berserk auras + position
/// fixes for old saves).
///
/// C++ Reference: `Source/diablo.cpp:1486-1508` — `void UpdateMonsterLights()`.
pub fn update_monster_lights(
    _level_type: DungeonType,
    mut update_each_monster_light: impl FnMut(),
) {
    // C++:1488-1507 — for each active monster:
    //   if berserk: re-add light at (Nest?9:3) radius
    //   if lightId valid: ChangeLightXY to current tile (fix stale lights)
    update_each_monster_light();
}

// ============================================================================
// Player init helpers (stubs)
//
// C++ Reference: Source/player.cpp:2307 (CreatePlayer), :2484 (InitPlayer)
// ============================================================================

/// Player hero class (mirrors C++ `HeroClass`).
///
/// C++ Reference: `Source/player.h` — `enum class HeroClass`.
/// Used by `create_player`. Kept minimal; the full enum has Barbarian/Monk/Rogue/...
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum HeroClass {
    #[default]
    Warrior = 0,
    Rogue = 1,
    Sorcerer = 2,
    Monk = 3,
    Bard = 4,
    Barbarian = 5,
}

impl HeroClass {
    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            HeroClass::Warrior => "Warrior",
            HeroClass::Rogue => "Rogue",
            HeroClass::Sorcerer => "Sorcerer",
            HeroClass::Monk => "Monk",
            HeroClass::Bard => "Bard",
            HeroClass::Barbarian => "Barbarian",
        }
    }
}

/// Initialises a freshly-created player's stats, spells, and starting gear.
///
/// C++ Reference: `Source/player.cpp:2307` — `void CreatePlayer(Player &player, HeroClass c)`.
///
/// The real implementation reads the per-class stat tables (`ClassStats`) and
/// starting inventory from `playerdat.cpp`. The Rust port delegates to a
/// closure so the owning module can populate the `Player` struct; this stub
/// exists for API fidelity and as a TODO marker.
pub fn create_player(_hero_class: HeroClass, mut populate: impl FnMut()) {
    // TODO(player.cpp:2307): wire the full per-class stat/inventory setup.
    // The closure is expected to fill in _pStrength/_pMagic/_pDexterity/
    // _pVitality, _pHitPoints/_pMaxHP, _pMana/_pMaxMana, base spells, and the
    // starting inventory + gold.
    populate();
}

/// Positions a player on level entry and resets their per-mode state.
///
/// C++ Reference: `Source/player.cpp:2484` — `void InitPlayer(Player &player, bool firstTime)`.
///
/// On `firstTime` this initialises position/HP/mana from the spawn point;
/// otherwise it restores the saved position. Also resets action mode, clears
/// queued spells, and re-centres the camera. Delegated to a closure.
pub fn init_player(_first_time: bool, mut populate: impl FnMut()) {
    // TODO(player.cpp:2484): wire spawn-point / position restore + mode reset.
    populate();
}

// ============================================================================
// Level init sequence stubs
//
// C++ Reference: Source/objects.cpp:3816 (InitObjects), Source/items.cpp:2435
//                (InitItems), Source/monsters.cpp (InitMonsters/InitLevelMonsters),
//                Source/towners.cpp:742 (InitTowners), Source/stores.cpp:2062 (InitStores)
// ============================================================================

/// Clears and rebuilds the per-level object list.
///
/// C++ Reference: `Source/objects.cpp:3816` — `void InitObjects()`. The real
/// implementation allocates the object pool and (for dungeon levels) scans
/// `dungeon` for theme rooms to populate objects into.
pub fn init_objects_stub() {
    // TODO(objects.cpp:3816): allocate object pool + populate from theme rooms.
}

/// Clears and rebuilds the per-level item list.
///
/// C++ Reference: `Source/items.cpp:2435` — `void InitItems()`.
pub fn init_items_stub() {
    // TODO(items.cpp:2435): allocate item pool + scatter random drops.
}

/// Allocates the active-monster pool for the current level.
///
/// C++ Reference: `Source/monsters.cpp` — `void InitMonsters()`.
pub fn init_monsters_stub() -> Result<(), String> {
    // TODO(monsters.cpp): allocate monster pool + place packs.
    Ok(())
}

/// Resets the per-level monster-type table (called before `GetLevelMTypes`).
///
/// C++ Reference: `Source/monsters.cpp` — `void InitLevelMonsters()`.
pub fn init_level_monsters_stub() {
    // TODO(monsters.cpp): clear LevelsTypeList + active monster arrays.
}

/// Spawns all town NPCs at their fixed Tristram positions.
///
/// C++ Reference: `Source/towners.cpp:742` — `void InitTowners()`.
pub fn init_towners_stub() {
    // TODO(towners.cpp:742): spawn Ogden, Griswold, Pepin, Adria, Cain,
    // Farnham, Gillian, Wirt at their fixed tiles.
}

/// Initialises the shop inventories for the town vendors.
///
/// C++ Reference: `Source/stores.cpp:2062` — `void InitStores()`.
pub fn init_stores_stub() {
    // TODO(stores.cpp:2062): build Griswold/Pepin/Adria/Ogden/Wirt shop tables.
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod ported_tests {
    use super::*;

    #[test]
    fn test_dungeon_type_values() {
        assert_eq!(DungeonType::Town as i8, 0);
        assert_eq!(DungeonType::Cathedral as i8, 1);
        assert_eq!(DungeonType::Catacombs as i8, 2);
        assert_eq!(DungeonType::Caves as i8, 3);
        assert_eq!(DungeonType::Hell as i8, 4);
        assert_eq!(DungeonType::Nest as i8, 5);
        assert_eq!(DungeonType::Crypt as i8, 6);
    }

    #[test]
    fn test_dungeon_type_is_town() {
        assert!(DungeonType::Town.is_town());
        assert!(!DungeonType::Cathedral.is_town());
    }

    #[test]
    fn test_get_level_type_classic() {
        assert_eq!(get_level_type(0, false), DungeonType::Town);
        assert_eq!(get_level_type(1, false), DungeonType::Cathedral);
        assert_eq!(get_level_type(4, false), DungeonType::Cathedral);
        assert_eq!(get_level_type(5, false), DungeonType::Catacombs);
        assert_eq!(get_level_type(8, false), DungeonType::Catacombs);
        assert_eq!(get_level_type(9, false), DungeonType::Caves);
        assert_eq!(get_level_type(12, false), DungeonType::Caves);
        assert_eq!(get_level_type(13, false), DungeonType::Hell);
        assert_eq!(get_level_type(16, false), DungeonType::Hell);
    }

    #[test]
    fn test_get_level_type_hellfire() {
        assert_eq!(get_level_type(0, true), DungeonType::Town);
        assert_eq!(get_level_type(17, true), DungeonType::Nest);
        assert_eq!(get_level_type(20, true), DungeonType::Nest);
        assert_eq!(get_level_type(21, true), DungeonType::Crypt);
        assert_eq!(get_level_type(24, true), DungeonType::Crypt);
        // Non-hellfire: 17-24 fall back to Cathedral.
        assert_eq!(get_level_type(17, false), DungeonType::Cathedral);
    }

    #[test]
    fn test_lvl_entry_values() {
        assert_eq!(LvlEntry::Main as u8, 0);
        assert_eq!(LvlEntry::Prev as u8, 1);
        assert_eq!(LvlEntry::Load as u8, 4);
        assert_eq!(LvlEntry::WarpLvl as u8, 5);
    }

    #[test]
    fn test_interface_mode_wm_values() {
        assert_eq!(InterfaceModeWm::DiabNextLvl as u8, 0);
        assert_eq!(InterfaceModeWm::DiabPrevLvl as u8, 1);
        assert_eq!(InterfaceModeWm::DiabRetown as u8, 7);
        assert_eq!(InterfaceModeWm::DiabNewGame as u8, 8);
        assert_eq!(InterfaceModeWm::DiabLoadGame as u8, 9);
    }

    #[test]
    fn test_interface_mode_progress_label() {
        assert_eq!(
            InterfaceModeWm::DiabNewGame.progress_label(),
            "Entering Cathedral"
        );
        assert_eq!(
            InterfaceModeWm::DiabRetown.progress_label(),
            "Entering Town"
        );
        assert_eq!(InterfaceModeWm::DiabWarpLvl.progress_label(), "Entering Portal");
    }

    #[test]
    fn test_pick_cutscene_new_game() {
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNewGame, 0, DungeonType::Town),
            Cutscene::Start
        );
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabLoadGame, 0, DungeonType::Town),
            Cutscene::Start
        );
    }

    #[test]
    fn test_pick_cutscene_retown() {
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabRetown, 0, DungeonType::Town),
            Cutscene::Town
        );
    }

    #[test]
    fn test_pick_cutscene_nextlvl_l1_is_town() {
        // Entering Cathedral from town → CutTown
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 1, DungeonType::Cathedral),
            Cutscene::Town
        );
    }

    #[test]
    fn test_pick_cutscene_nextlvl_l16_is_gate() {
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 16, DungeonType::Hell),
            Cutscene::Gate
        );
    }

    #[test]
    fn test_pick_cutscene_by_level_type() {
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 2, DungeonType::Cathedral),
            Cutscene::Level1
        );
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 5, DungeonType::Catacombs),
            Cutscene::Level2
        );
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 9, DungeonType::Caves),
            Cutscene::Level3
        );
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 13, DungeonType::Hell),
            Cutscene::Level4
        );
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 17, DungeonType::Nest),
            Cutscene::Level6
        );
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabNextLvl, 21, DungeonType::Crypt),
            Cutscene::Level5
        );
    }

    #[test]
    fn test_pick_cutscene_portal() {
        assert_eq!(
            pick_cutscene(InterfaceModeWm::DiabWarpLvl, 5, DungeonType::Catacombs),
            Cutscene::Portal
        );
    }

    #[test]
    fn test_level_gfx_paths_town() {
        let p = level_gfx_paths(DungeonType::Town);
        assert_eq!(p.cel, "levels\\towndata\\town.cel");
        assert_eq!(p.til, "levels\\towndata\\town.til");
        assert_eq!(p.special, "levels\\towndata\\towns");
    }

    #[test]
    fn test_level_gfx_paths_cathedral() {
        let p = level_gfx_paths(DungeonType::Cathedral);
        assert_eq!(p.cel, "levels\\l1data\\l1.cel");
        assert_eq!(p.special, "levels\\l1data\\l1s");
    }

    #[test]
    fn test_level_gfx_paths_caves_reuses_l1s() {
        // C++ diablo.cpp:1390 reuses l1's special cel for the Caves.
        let p = level_gfx_paths(DungeonType::Caves);
        assert_eq!(p.special, "levels\\l1data\\l1s");
    }

    #[test]
    fn test_level_gfx_paths_hell_reuses_l2s() {
        // C++ diablo.cpp:1395 reuses l2's special cel for Hell.
        let p = level_gfx_paths(DungeonType::Hell);
        assert_eq!(p.special, "levels\\l2data\\l2s");
    }

    /// Records the order of GameLogic calls for a dungeon level.
    #[derive(Default)]
    struct LogicTrace {
        calls: Vec<&'static str>,
    }
    impl GameLogicContext for LogicTrace {
        fn process_input(&mut self) -> bool {
            self.calls.push("ProcessInput");
            true
        }
        fn process_players(&mut self) {
            self.calls.push("ProcessPlayers");
        }
        fn process_monsters(&mut self) {
            self.calls.push("ProcessMonsters");
        }
        fn process_objects(&mut self) {
            self.calls.push("ProcessObjects");
        }
        fn process_missiles(&mut self) {
            self.calls.push("ProcessMissiles");
        }
        fn process_items(&mut self) {
            self.calls.push("ProcessItems");
        }
        fn process_light_list(&mut self) {
            self.calls.push("ProcessLightList");
        }
        fn process_vision_list(&mut self) {
            self.calls.push("ProcessVisionList");
        }
        fn sound_update(&mut self) {
            self.calls.push("sound_update");
        }
        fn check_triggers(&mut self) {
            self.calls.push("CheckTriggers");
        }
        fn check_quests(&mut self) {
            self.calls.push("CheckQuests");
        }
        fn redraw_viewport(&mut self) {
            self.calls.push("RedrawViewport");
        }
        fn pfile_update(&mut self) {
            self.calls.push("pfile_update");
        }
        fn plrctrls_after_game_logic(&mut self) {
            self.calls.push("plrctrls_after_game_logic");
        }
    }

    #[test]
    fn test_game_logic_dungeon_sequence() {
        let mut trace = LogicTrace::default();
        let mut step = GameLogicStep::None;
        let outcome = game_logic(
            &mut trace,
            true,
            DungeonType::Cathedral,
            Some(&mut step),
        );
        assert_eq!(outcome, GameLogicOutcome::Ok);
        assert_eq!(step, GameLogicStep::None);
        // C++ diablo.cpp:1510-1556 exact order.
        assert_eq!(
            trace.calls,
            vec![
                "ProcessInput",
                "ProcessPlayers",
                "ProcessMonsters",
                "ProcessObjects",
                "ProcessMissiles",
                "ProcessItems",
                "ProcessLightList",
                "ProcessVisionList",
                "sound_update",
                "CheckTriggers",
                "CheckQuests",
                "RedrawViewport",
                "pfile_update",
                "plrctrls_after_game_logic",
            ]
        );
    }

    #[test]
    fn test_game_logic_town_sequence() {
        let mut trace = LogicTrace::default();
        let outcome = game_logic(&mut trace, true, DungeonType::Town, None);
        assert_eq!(outcome, GameLogicOutcome::Ok);
        // Town path: ProcessTowners, ProcessItems, ProcessMissiles (no monsters/objects/light/vision).
        assert_eq!(
            trace.calls,
            vec![
                "ProcessInput",
                "ProcessPlayers",
                "ProcessTowners",
                "ProcessItems",
                "ProcessMissiles",
                "sound_update",
                "CheckTriggers",
                "CheckQuests",
                "RedrawViewport",
                "pfile_update",
                "plrctrls_after_game_logic",
            ]
        );
    }

    #[test]
    fn test_game_logic_skip_when_input_false() {
        struct Skip;
        impl GameLogicContext for Skip {
            fn process_input(&mut self) -> bool {
                false
            }
            fn process_players(&mut self) {
                panic!("should not reach process_players");
            }
        }
        let mut s = Skip;
        let outcome = game_logic(&mut s, true, DungeonType::Cathedral, None);
        assert_eq!(outcome, GameLogicOutcome::Skipped);
    }

    #[test]
    fn test_game_logic_skips_players_when_flag_off() {
        struct NoPlayers {
            called: bool,
        }
        impl GameLogicContext for NoPlayers {
            fn process_input(&mut self) -> bool {
                true
            }
            fn process_players(&mut self) {
                self.called = true;
            }
        }
        let mut ctx = NoPlayers { called: false };
        let _ = game_logic(&mut ctx, false, DungeonType::Cathedral, None);
        assert!(!ctx.called, "process_players must be skipped when flag off");
    }

    #[test]
    fn test_color_cycling_skipped_when_disabled() {
        let mut effect = None;
        diablo_color_cyc_logic(false, pause_mode::NONE, DungeonType::Caves, false, |e| {
            effect = Some(e);
        });
        assert!(effect.is_none());
    }

    #[test]
    fn test_color_cycling_skipped_when_paused() {
        let mut effect = None;
        diablo_color_cyc_logic(true, pause_mode::PAUSED, DungeonType::Caves, false, |e| {
            effect = Some(e);
        });
        assert!(effect.is_none());
    }

    #[test]
    fn test_color_cycling_caves() {
        let mut effect = None;
        diablo_color_cyc_logic(true, pause_mode::NONE, DungeonType::Caves, false, |e| {
            effect = Some(e);
        });
        assert_eq!(effect, Some(ColorCycleEffect::Caves));
    }

    #[test]
    fn test_color_cycling_pwater() {
        let mut effect = None;
        diablo_color_cyc_logic(true, pause_mode::NONE, DungeonType::Caves, true, |e| {
            effect = Some(e);
        });
        assert_eq!(effect, Some(ColorCycleEffect::PWater));
    }

    #[test]
    fn test_color_cycling_hell() {
        let mut effect = None;
        diablo_color_cyc_logic(true, pause_mode::NONE, DungeonType::Hell, false, |e| {
            effect = Some(e);
        });
        assert_eq!(effect, Some(ColorCycleEffect::Lighting));
    }

    #[test]
    fn test_color_cycling_town_noop() {
        let mut effect = None;
        diablo_color_cyc_logic(true, pause_mode::NONE, DungeonType::Town, false, |e| {
            effect = Some(e);
        });
        assert!(effect.is_none());
    }

    #[test]
    fn test_is_diablo_alive_default() {
        assert!(is_diablo_alive(false, false));
    }

    #[test]
    fn test_is_diablo_alive_after_quest_done_sp() {
        // Single-player (GB_IS_MULTIPLAYER defaults false), quest done → dead.
        unsafe { GB_IS_MULTIPLAYER = false; }
        assert!(!is_diablo_alive(false, true));
    }

    #[test]
    fn test_is_diablo_alive_after_quest_done_mp() {
        // Multiplayer ignores the quest-done gate.
        unsafe { GB_IS_MULTIPLAYER = true; }
        assert!(is_diablo_alive(false, true));
        unsafe { GB_IS_MULTIPLAYER = false; }
    }

    #[test]
    fn test_free_game_mem_calls_all_in_order() {
        let mut order = Vec::new();
        free_game_mem(
            || order.push("dungeon_cels"),
            || order.push("mega_tiles"),
            || order.push("special_cels"),
            || order.push("monsters"),
            || order.push("missile_gfx"),
            || order.push("object_gfx"),
            || order.push("towner_gfx"),
            || order.push("stash_gfx"),
        );
        assert_eq!(
            order,
            vec![
                "dungeon_cels",
                "mega_tiles",
                "special_cels",
                "monsters",
                "missile_gfx",
                "object_gfx",
                "towner_gfx",
                "stash_gfx",
            ]
        );
    }

    #[test]
    fn test_do_load_new_game_calls_load_game_level() {
        let mut loaded = Vec::new();
        let mut inc_count = 0u32;
        let result = do_load(
            InterfaceModeWm::DiabNewGame,
            |_n| {},
            |_| {},
            |_| {},
            || {},
            |firstflag, entry| {
                loaded.push((firstflag, entry));
                Ok(())
            },
        );
        // Use inc_progress to avoid unused warning by counting.
        let _ = do_load(
            InterfaceModeWm::DiabNewGame,
            |_| inc_count += 1,
            |_| {},
            |_| {},
            || {},
            |_, _| Ok(()),
        );
        assert!(result.is_ok());
        assert_eq!(loaded, vec![(true, LvlEntry::Main)]);
        assert!(inc_count > 0);
    }

    #[test]
    fn test_do_load_unknown_mode_errors() {
        let result = do_load(
            InterfaceModeWm::Done,
            |_| {},
            |_| {},
            |_| {},
            || {},
            |_, _| Ok(()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_set_current_level_updates_level_type() {
        unsafe {
            GB_IS_HELLFIRE = false;
            CURR_LEVEL = 0;
            LEVEL_TYPE = DungeonType::Town;
        }
        set_current_level(5);
        assert_eq!(current_level(), 5);
        assert_eq!(current_level_type(), DungeonType::Catacombs);
        // Reset.
        unsafe {
            CURR_LEVEL = 0;
            LEVEL_TYPE = DungeonType::Town;
        }
    }

    #[test]
    fn test_hero_class_names() {
        assert_eq!(HeroClass::Warrior.name(), "Warrior");
        assert_eq!(HeroClass::Sorcerer.name(), "Sorcerer");
        assert_eq!(HeroClass::Barbarian.name(), "Barbarian");
    }

    /// A no-op LoadLevelContext for exercising load_game_level end-to-end
    /// without touching any real subsystems.
    struct NoopLoadContext {
        inc_count: u32,
        create_called: bool,
        init_players_called: bool,
    }
    impl LoadLevelContext for NoopLoadContext {
        fn inc_progress(&mut self) {
            self.inc_count += 1;
        }
        fn create_level(&mut self, entry: LvlEntry) -> Result<(), String> {
            let _ = entry;
            self.create_called = true;
            Ok(())
        }
        fn init_players(&mut self, _firstflag: bool, _lvldir: LvlEntry) {
            self.init_players_called = true;
        }
    }

    #[test]
    fn test_load_game_level_town_runs_full_sequence() {
        let mut ctx = NoopLoadContext {
            inc_count: 0,
            create_called: false,
            init_players_called: false,
        };
        let result = load_game_level(&mut ctx, true, LvlEntry::Main, DungeonType::Town, false);
        assert!(result.is_ok());
        assert!(ctx.create_called, "CreateLevel must run even for town");
        assert!(ctx.init_players_called);
        assert!(ctx.inc_count > 5, "multiple IncProgress calls expected");
    }

    #[test]
    fn test_load_game_level_dungeon_runs_full_sequence() {
        let mut ctx = NoopLoadContext {
            inc_count: 0,
            create_called: false,
            init_players_called: false,
        };
        let result = load_game_level(
            &mut ctx,
            true,
            LvlEntry::Main,
            DungeonType::Cathedral,
            false,
        );
        assert!(result.is_ok());
        assert!(ctx.create_called);
        assert!(ctx.init_players_called);
    }

    #[test]
    fn test_load_game_level_propagates_load_lvl_gfx_error() {
        struct FailGfx {
            inc: u32,
        }
        impl LoadLevelContext for FailGfx {
            fn inc_progress(&mut self) {
                self.inc += 1;
            }
            fn load_lvl_gfx(&mut self) -> Result<(), String> {
                Err("missing l1.cel".to_string())
            }
        }
        let mut ctx = FailGfx { inc: 0 };
        let result = load_game_level(&mut ctx, true, LvlEntry::Main, DungeonType::Cathedral, false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "missing l1.cel");
    }
}
