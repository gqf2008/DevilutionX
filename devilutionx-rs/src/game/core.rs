//! Core game state and logic - based on DevilutionX diablo.cpp
//!
//! This module manages the main game state machine, game loop logic,
//! and coordinates all game subsystems.

use super::player::{Player, PlayerClass, PlayerMode};
use super::types::{Direction, DungeonType, Difficulty, Point, GameLogicStep};
use super::level::Level;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Tick rate (game logic updates per second)
pub const TICK_RATE: u32 = 20;
/// Tick duration in milliseconds
pub const TICK_DURATION_MS: u64 = 1000 / TICK_RATE as u64;

/// Main game states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// Main menu / title screen
    MainMenu,
    /// Character selection
    CharacterSelect,
    /// In-game playing
    Playing,
    /// Game paused
    Paused,
    /// Loading screen
    Loading,
    /// Death screen
    Death,
    /// Victory/credits
    Victory,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::MainMenu
    }
}

/// Game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Current difficulty
    pub difficulty: Difficulty,
    /// Is this a multiplayer game?
    pub is_multiplayer: bool,
    /// Is Hellfire expansion enabled?
    pub is_hellfire: bool,
    /// Screen width
    pub screen_width: u32,
    /// Screen height
    pub screen_height: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            is_multiplayer: false,
            is_hellfire: false,
            screen_width: 800,
            screen_height: 600,
        }
    }
}

/// Mouse state
#[derive(Debug, Clone, Default)]
pub struct MouseState {
    /// Current mouse position (screen coordinates)
    pub position: Point,
    /// Mouse position in world/tile coordinates
    pub world_position: Point,
    /// Left button held
    pub left_held: bool,
    /// Right button held
    pub right_held: bool,
    /// Left button just clicked this frame
    pub left_clicked: bool,
    /// Right button just clicked this frame
    pub right_clicked: bool,
}

/// Keyboard state for continuous input
#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub space: bool,
}

/// Input state aggregator
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
}

/// Camera/viewport for rendering
#[derive(Debug, Clone)]
pub struct Camera {
    /// Center position in world coordinates
    pub position: Point,
    /// Viewport width in pixels
    pub width: i32,
    /// Viewport height in pixels
    pub height: i32,
    /// Zoom level (1.0 = normal)
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point::new(0, 0),
            width: 800,
            height: 600,
            zoom: 1.0,
        }
    }
}

impl Camera {
    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Point) -> Point {
        let scale = 1.0 / self.zoom;
        Point::new(
            self.position.x + ((screen_pos.x - self.width / 2) as f32 * scale) as i32,
            self.position.y + ((screen_pos.y - self.height / 2) as f32 * scale) as i32,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Point) -> Point {
        Point::new(
            (self.width / 2) + ((world_pos.x - self.position.x) as f32 * self.zoom) as i32,
            (self.height / 2) + ((world_pos.y - self.position.y) as f32 * self.zoom) as i32,
        )
    }

    /// Center camera on a world position
    pub fn center_on(&mut self, pos: Point) {
        self.position = pos;
    }
}

/// The main game state container
#[derive(Debug)]
pub struct Game {
    /// Current game mode
    pub mode: GameMode,
    /// Game configuration
    pub config: GameConfig,
    /// Current game tick (logic frame counter)
    pub tick: u64,
    /// Last tick time
    pub last_tick: Instant,
    /// Accumulated time for fixed timestep
    pub accumulator: Duration,
    /// Current logic step being processed
    pub logic_step: GameLogicStep,
    /// Is the game running?
    pub running: bool,
    /// Should return to main menu?
    pub return_to_menu: bool,

    // Game entities
    /// Local player
    pub player: Player,
    /// Current level
    pub level: Level,
    /// Camera/viewport
    pub camera: Camera,
    /// Input state
    pub input: InputState,

    // UI state
    /// Is inventory open?
    pub inventory_open: bool,
    /// Is character panel open?
    pub character_panel_open: bool,
    /// Is spell book open?
    pub spellbook_open: bool,
    /// Is quest log open?
    pub quest_log_open: bool,
    /// Is automap visible?
    pub automap_visible: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        Self {
            mode: GameMode::MainMenu,
            config: GameConfig::default(),
            tick: 0,
            last_tick: Instant::now(),
            accumulator: Duration::ZERO,
            logic_step: GameLogicStep::None,
            running: true,
            return_to_menu: false,
            player: Player::new("Hero".to_string(), PlayerClass::Warrior),
            level: Level::new(),
            camera: Camera::default(),
            input: InputState::default(),
            inventory_open: false,
            character_panel_open: false,
            spellbook_open: false,
            quest_log_open: false,
            automap_visible: false,
        }
    }

    /// Start a new game
    pub fn start_new_game(&mut self, class: PlayerClass, name: String, difficulty: Difficulty) {
        self.config.difficulty = difficulty;
        self.player = Player::new(name, class);
        self.level.init_level(0); // Start in town

        // Set player starting position (town center)
        self.player.set_position(75, 68);

        // Center camera on player
        self.camera.center_on(self.player.tile_position());

        self.mode = GameMode::Playing;
        self.tick = 0;
        self.running = true;
        self.return_to_menu = false;
    }

    /// Main game logic update (called at fixed timestep)
    pub fn update(&mut self) {
        if self.mode != GameMode::Playing {
            return;
        }

        self.tick += 1;

        // Process player input and movement
        self.logic_step = GameLogicStep::ProcessPlayers;
        self.process_player();

        // Process monsters (if not in town)
        if self.level.dungeon_type != DungeonType::Town {
            self.logic_step = GameLogicStep::ProcessMonsters;
            self.process_monsters();

            self.logic_step = GameLogicStep::ProcessMissiles;
            self.process_missiles();

            self.logic_step = GameLogicStep::ProcessObjects;
            self.process_objects();
        }

        self.logic_step = GameLogicStep::ProcessItems;
        self.process_items();

        self.logic_step = GameLogicStep::None;

        // Update camera to follow player
        self.camera.center_on(self.player.tile_position());
    }

    /// Process player input and update player state
    fn process_player(&mut self) {
        // Handle keyboard movement
        let mut move_dir: Option<Direction> = None;

        if self.input.keyboard.up && self.input.keyboard.left {
            move_dir = Some(Direction::NorthWest);
        } else if self.input.keyboard.up && self.input.keyboard.right {
            move_dir = Some(Direction::NorthEast);
        } else if self.input.keyboard.down && self.input.keyboard.left {
            move_dir = Some(Direction::SouthWest);
        } else if self.input.keyboard.down && self.input.keyboard.right {
            move_dir = Some(Direction::SouthEast);
        } else if self.input.keyboard.up {
            move_dir = Some(Direction::North);
        } else if self.input.keyboard.down {
            move_dir = Some(Direction::South);
        } else if self.input.keyboard.left {
            move_dir = Some(Direction::West);
        } else if self.input.keyboard.right {
            move_dir = Some(Direction::East);
        }

        // Handle mouse click movement
        if self.input.mouse.left_clicked {
            let target = self.input.mouse.world_position;
            let player_pos = self.player.tile_position();

            // Calculate direction to target
            let dx = target.x - player_pos.x;
            let dy = target.y - player_pos.y;

            if dx != 0 || dy != 0 {
                move_dir = Some(Self::direction_from_delta(dx, dy));
            }
        }

        // Apply movement
        if let Some(dir) = move_dir {
            if self.player.mode == PlayerMode::Stand {
                let new_pos = self.player.tile_position().shift(dir);

                // Check if new position is walkable
                if self.is_walkable(new_pos) {
                    self.player.start_walk(dir);
                    self.player.set_position(new_pos.x, new_pos.y);
                }
            }
        } else if self.player.mode != PlayerMode::Stand {
            self.player.stop_walk();
        }

        // Update player animation
        self.player.update();
    }

    /// Calculate direction from delta x/y
    fn direction_from_delta(dx: i32, dy: i32) -> Direction {
        // Normalize to -1, 0, 1
        let sx = dx.signum();
        let sy = dy.signum();

        match (sx, sy) {
            (0, -1) => Direction::North,
            (1, -1) => Direction::NorthEast,
            (1, 0) => Direction::East,
            (1, 1) => Direction::SouthEast,
            (0, 1) => Direction::South,
            (-1, 1) => Direction::SouthWest,
            (-1, 0) => Direction::West,
            (-1, -1) => Direction::NorthWest,
            _ => Direction::South,
        }
    }

    /// Check if a tile is walkable
    fn is_walkable(&self, pos: Point) -> bool {
        // Bounds check
        if pos.x < 0 || pos.y < 0 || pos.x >= 112 || pos.y >= 112 {
            return false;
        }

        // TODO: Check actual tile properties
        // For now, allow movement everywhere
        true
    }

    /// Process monsters (placeholder)
    fn process_monsters(&mut self) {
        // TODO: Implement monster AI
    }

    /// Process missiles (placeholder)
    fn process_missiles(&mut self) {
        // TODO: Implement missile updates
    }

    /// Process objects (placeholder)
    fn process_objects(&mut self) {
        // TODO: Implement object interactions
    }

    /// Process items (placeholder)
    fn process_items(&mut self) {
        // TODO: Implement item drops/pickup
    }

    /// Handle a single game loop iteration with fixed timestep
    pub fn run_frame(&mut self, delta: Duration) -> bool {
        self.accumulator += delta;

        let tick_duration = Duration::from_millis(TICK_DURATION_MS);

        // Run logic updates at fixed timestep
        while self.accumulator >= tick_duration {
            self.update();
            self.accumulator -= tick_duration;
        }

        self.running && !self.return_to_menu
    }

    /// Toggle inventory panel
    pub fn toggle_inventory(&mut self) {
        self.inventory_open = !self.inventory_open;
        if self.inventory_open {
            self.character_panel_open = false;
        }
    }

    /// Toggle character panel
    pub fn toggle_character_panel(&mut self) {
        self.character_panel_open = !self.character_panel_open;
        if self.character_panel_open {
            self.inventory_open = false;
        }
    }

    /// Toggle spell book
    pub fn toggle_spellbook(&mut self) {
        self.spellbook_open = !self.spellbook_open;
    }

    /// Toggle quest log
    pub fn toggle_quest_log(&mut self) {
        self.quest_log_open = !self.quest_log_open;
    }

    /// Toggle automap
    pub fn toggle_automap(&mut self) {
        self.automap_visible = !self.automap_visible;
    }

    /// Close all panels
    pub fn close_all_panels(&mut self) {
        self.inventory_open = false;
        self.character_panel_open = false;
        self.spellbook_open = false;
        self.quest_log_open = false;
    }

    /// Handle escape key
    pub fn handle_escape(&mut self) -> bool {
        // If any panel is open, close it
        if self.inventory_open || self.character_panel_open ||
           self.spellbook_open || self.quest_log_open {
            self.close_all_panels();
            return true;
        }

        // Otherwise, return to menu or show game menu
        if self.mode == GameMode::Playing {
            self.mode = GameMode::Paused;
            return true;
        }

        false
    }

    /// Get player screen position for rendering
    pub fn get_player_screen_pos(&self) -> Point {
        self.camera.world_to_screen(self.player.tile_position())
    }
}

/// Isometric coordinate conversion utilities
pub mod isometric {
    use super::Point;

    /// Tile size in pixels (for isometric rendering)
    pub const TILE_WIDTH: i32 = 64;
    pub const TILE_HEIGHT: i32 = 32;

    /// Convert tile coordinates to screen coordinates (isometric projection)
    pub fn tile_to_screen(tile_x: i32, tile_y: i32) -> Point {
        Point::new(
            (tile_x - tile_y) * (TILE_WIDTH / 2),
            (tile_x + tile_y) * (TILE_HEIGHT / 2),
        )
    }

    /// Convert screen coordinates to tile coordinates (isometric projection)
    pub fn screen_to_tile(screen_x: i32, screen_y: i32) -> Point {
        // Inverse of tile_to_screen
        let fx = screen_x as f32 / (TILE_WIDTH / 2) as f32;
        let fy = screen_y as f32 / (TILE_HEIGHT / 2) as f32;

        Point::new(
            ((fx + fy) / 2.0).floor() as i32,
            ((fy - fx) / 2.0).floor() as i32,
        )
    }

    /// Get the screen bounds for a visible area
    pub fn get_visible_tiles(
        camera_x: i32,
        camera_y: i32,
        screen_width: i32,
        screen_height: i32,
    ) -> (Point, Point) {
        // Calculate tile bounds with some padding
        let half_w = screen_width / 2 + TILE_WIDTH * 2;
        let half_h = screen_height / 2 + TILE_HEIGHT * 4;

        let top_left = screen_to_tile(camera_x - half_w, camera_y - half_h);
        let bottom_right = screen_to_tile(camera_x + half_w, camera_y + half_h);

        (top_left, bottom_right)
    }
}
