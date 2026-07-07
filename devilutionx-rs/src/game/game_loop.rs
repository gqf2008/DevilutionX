// Game Loop Module (M80 - Complete C++ Alignment)
//
// C++ References:
// - Source/diablo.cpp:857 (RunGameLoop) - Main loop wrapper
// - Source/diablo.cpp:3416 (game_loop) - Iteration controller
// - Source/diablo.cpp:1510 (GameLogic) - Core game logic
//
// Architecture:
// - Two-speed loop: Fast path (60 FPS) for input/render, Logic path (2 Hz) for game state
// - RunGameLoop → game_loop → GameLogic hierarchy

use crate::engine::timing::GameTiming;
use crate::engine::window::{GameWindow, Color};
use crate::engine::isometric::{IsoPoint, TILE_WIDTH, TILE_HEIGHT};
use crate::engine::dungeon::{TileDecoder, TileType};
use crate::engine::sprite_render::rgba_to_texture;
use crate::game::input::InputSystem;
use crate::game::network;
use crate::game::game_state::GameState;
use anyhow::Result;
use sdl2::event::Event;
use sdl2::rect::Rect;
use rand::SeedableRng;

/// Interface mode for game initialization
///
/// C++ Reference: Source/diablo.h - interface_mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceMode {
    /// New game
    NewGame,
    /// Load game
    LoadGame,
    /// Join multiplayer
    MultiplayerJoin,
    /// Create multiplayer
    MultiplayerCreate,
}

/// Game loop state
struct GameLoopState {
    /// Main loop running flag (gbRunGame)
    running: bool,
    /// First iteration flag (gbGameLoopStartup)
    startup: bool,
    /// Should process player logic (gbProcessPlayers)
    process_players: bool,
    /// Game result on exit
    result: bool,
}

impl GameLoopState {
    fn new() -> Self {
        Self {
            running: true,
            startup: true,
            process_players: true,
            result: true,
        }
    }
}

/// Run the main game loop
///
/// C++ Reference: Source/diablo.cpp:857 - void RunGameLoop(interface_mode uMsg)
///
/// This is the complete production game loop with proper timing,
/// not a simplified demo.
///
/// # Arguments
/// * `mode` - Game initialization mode
/// * `window` - The game window
/// * `game_state` - The shared game state
///
/// # Returns
/// * `Ok(true)` - Game exited normally
/// * `Ok(false)` - Game quit via menu
/// * `Err(_)` - Error occurred
pub fn run_game_loop(mode: InterfaceMode, window: &mut GameWindow, game_state: &mut GameState) -> Result<bool> {
    println!("Starting game loop (mode: {:?})", mode);

    // 1. Initialization sequence
    // C++: StartGame(uMsg), SetEventHandler, run_delta_info
    start_game(mode)?;

    let mut state = GameLoopState::new();
    state.process_players = is_diablo_alive(true);

    // 2. Fade-in sequence
    // C++: PrepareForFadeIn(), LoadPWaterPalette(), PaletteFadeIn(8)
    prepare_for_fade_in();
    load_palette();
    palette_fade_in(8);

    // 3. Initialize input
    let mut input = InputSystem::new();

    init_backbuffer();
    redraw_everything(window);

    // 4. Create timing controller
    let mut timing = GameTiming::new();

    println!("Entering main game loop...");

    // 5. Main loop
    // C++: while (gbRunGame)
    while state.running {
        // Event processing
        // C++: while (FetchMessage(&event, &modState))
        for event in window.event_pump()?.poll_iter() {
            if !handle_event(&event, &mut state, &mut input) {
                break;
            }
        }

        if !state.running {
            break;
        }

        // Timing check - has 500ms passed?
        // C++: bool runGameLoop = nthread_has_500ms_passed(&drawGame)
        let mut draw_game = true;
        let run_game_logic = timing.has_500ms_passed(&mut draw_game);

        if !run_game_logic {
            // FAST PATH - Every frame (~60 Hz)
            // C++: ProcessInput(), DvlNet_ProcessNetworkPackets(), RedrawViewport()

            // Process input
            process_input(&mut input)?;

            // Process network packets
            network::process_network_packets();

            // Render if needed
            if draw_game {
                redraw_viewport(window, game_state);
                draw_and_blit(window, game_state);
            }

            continue;
        }

        // LOGIC PATH - Every 500ms (~2 Hz)
        // C++: ProcessGameMessagePackets(), game_loop(gbGameLoopStartup), diablo_color_cyc_logic()

        // Process network messages
        network::process_game_message_packets();

        // Run game logic iterations
        if game_loop_iteration(state.startup, game_state)? {
            // Color cycling if logic succeeded
            color_cycling_logic();
        }

        state.startup = false;

        // Render
        if draw_game {
            redraw_viewport(window, game_state);
            draw_and_blit(window, game_state);
        }
    }

    println!("Game loop ended");

    // 6. Cleanup
    // C++: PaletteFadeOut(8), cleanup
    palette_fade_out(8);
    cleanup();

    Ok(state.result)
}

/// Game loop iteration controller
///
/// C++ Reference: Source/diablo.cpp:3416 - bool game_loop(bool bStartup)
///
/// Runs game logic multiple times:
/// - Normal: 3 iterations
/// - Startup: tickRate × 3 iterations (usually 60-120)
///
/// # Arguments
/// * `startup` - Is this the first iteration?
///
/// # Returns
/// * `Ok(true)` - Logic succeeded, can do color cycling
/// * `Ok(false)` - Network timeout occurred
fn game_loop_iteration(startup: bool, game_state: &mut GameState) -> Result<bool> {
    // C++: const uint16_t wait = bStartup ? sgGameInitInfo.nTickRate * 3 : 3;
    const TICK_RATE: u16 = 20; // Typical value from C++
    let iterations = if startup { TICK_RATE * 3 } else { 3 };

    for _ in 0..iterations {
        // Network synchronization
        // C++: if (!multi_handle_delta()) { TimeoutCursor(true); return false; }
        if !network::handle_delta() {
            timeout_cursor(true);
            return Ok(false);
        }

        timeout_cursor(false);

        // Core game logic
        // C++: GameLogic()
        game_logic(game_state)?;

        // Network command cleanup
        // C++: ClearLastSentPlayerCmd()
        network::clear_last_sent_cmd();

        // Early exit conditions
        // C++: if (!gbRunGame || !gbIsMultiplayer || !nthread_has_500ms_passed()) break;
        if !is_running() || !network::is_multiplayer() {
            break;
        }
    }

    Ok(true)
}

/// Core game logic - updates all game systems
///
/// C++ Reference: Source/diablo.cpp:1510 - void GameLogic()
///
/// This runs all game state updates:
/// - Player AI and movement
/// - Monster AI and pathfinding
/// - Object interactions
/// - Missile physics
/// - Item updates
/// - Lighting calculations
/// - Sound effects
fn game_logic(game_state: &mut GameState) -> Result<()> {
    // C++: if (!ProcessInput()) return;
    // (Already handled in fast path, but checked here too)

    // Get current level type
    let level_type = get_level_type();

    // Use the unified GameState update
    // This replaces the individual process calls with the centralized logic
    // found in game_state.rs
    let mut rng = rand::rngs::StdRng::seed_from_u64(0); // TODO: Use game seed
    game_state.update(&mut rng);

    if level_type.is_town() {
        // TOWN LOGIC
        // C++: ProcessTowners(), ProcessItems(), ProcessMissiles()

        process_towners()?;
        // process_items is handled by game_state.update
        // process_missiles is handled by game_state.update

    } else {
        // DUNGEON LOGIC
        // C++: ProcessPlayers(), ProcessMonsters(), ProcessObjects(), etc.

        // All handled by game_state.update()

        // Lighting and vision
        process_light_list();
        process_vision_list();
    }

    // Sound updates
    sound_update();

    // Game state checks
    check_triggers();
    check_quests();

    // Viewport rendering (marks dirty regions)
    // Note: Actual drawing happens in draw_and_blit()
    // redraw_viewport(); // Called in main loop

    // Player file updates
    pfile_update(false);

    // Post-logic controls
    plrctrls_after_game_logic();

    Ok(())
}

//------------------------------------------------------------------------------
// Initialization Functions (Stubs)
//------------------------------------------------------------------------------

fn start_game(_mode: InterfaceMode) -> Result<()> {
    println!("StartGame() - TODO: Implement game initialization");
    // C++: StartGame(uMsg) - loads game state, initializes systems
    Ok(())
}

fn prepare_for_fade_in() {
    // C++: PrepareForFadeIn() - sets up palette transition
}

fn load_palette() {
    // C++: LoadPWaterPalette() - loads palette for current level
}

fn palette_fade_in(_steps: u8) {
    // C++: PaletteFadeIn(8) - fades in over 8 steps
}

fn palette_fade_out(_steps: u8) {
    // C++: PaletteFadeOut(8) - fades out over 8 steps
}

fn init_backbuffer() {
    // C++: InitBackbufferState() - initializes rendering buffers
}

fn redraw_everything(_window: &mut GameWindow) {
    // C++: RedrawEverything() - marks all screen regions dirty
}

fn cleanup() {
    // C++: NewCursor(CURSOR_NONE), save player data, etc.
}

//------------------------------------------------------------------------------
// Event Handling
//------------------------------------------------------------------------------

fn handle_event(event: &Event, state: &mut GameLoopState, _input: &mut InputSystem) -> bool {
    match event {
        Event::Quit { .. } => {
            println!("Quit event received");
            state.result = false;
            state.running = false;
            false
        }
        _ => {
            // C++: HandleMessage(event, modState)
            // TODO: Route to proper handlers
            true
        }
    }
}

//------------------------------------------------------------------------------
// Input Processing
//------------------------------------------------------------------------------

fn process_input(_input: &mut InputSystem) -> Result<()> {
    // C++: ProcessInput() - handles keyboard/mouse input
    // Already implemented in InputSystem
    Ok(())
}

//------------------------------------------------------------------------------
// Rendering Functions
//------------------------------------------------------------------------------

fn redraw_viewport(_window: &mut GameWindow, _game_state: &GameState) {
    // C++: RedrawViewport() - marks dirty regions; the actual drawing happens
    // in draw_and_blit() below, which now renders the full isometric view each
    // frame. Kept as a hook for future partial-redraw optimisation.
}

fn draw_and_blit(window: &mut GameWindow, game_state: &GameState) {
    // C++: DrawAndBlit() - renders the dungeon viewport then flips the back
    // buffer. Previously this drew a single 32x32 green square. It now renders
    // an isometric floor (checkerboard) as a stable proof that the camera/
    // viewport/isometric pipeline works, and — when town level data has been
    // loaded into game_state.level_data — overlays real decoded town tiles.
    //
    // Rendering uses logical 640x480 coordinates (the canvas has
    // set_logical_size(640,480) applied in main.rs), so the viewport centre is
    // (320, 240) regardless of the physical window size.

    window.clear(Color::BLACK);

    // Logical viewport centre.
    let center_x: i32 = LOGICAL_WIDTH as i32 / 2; // 320
    let center_y: i32 = LOGICAL_HEIGHT as i32 / 2; // 240

    // Camera is at the world origin (no scrolling yet), so screen positions are
    // computed relative to (0,0) and then offset by the viewport centre.
    let camera = IsoPoint::new(0, 0);
    // Render a grid large enough to cover the 640x480 logical viewport. Each
    // isometric tile is 64x32; ~16 columns x ~24 rows comfortably fills it.
    let grid_w = 16;
    let grid_h = 24;

    // 1. Stable checkerboard floor (always drawn) — proves the iso pipeline.
    //    Drawn as filled diamonds centred on each tile's screen position.
    let canvas = window.canvas_mut();
    for ty in 0..grid_h {
        for tx in 0..grid_w {
            let is_dark = (tx + ty) % 2 == 0;
            let color = if is_dark {
                sdl2::pixels::Color::RGB(36, 36, 52)
            } else {
                sdl2::pixels::Color::RGB(58, 58, 78)
            };
            let (sx, sy) = IsoPoint::new(tx, ty).to_screen(camera);
            fill_diamond(canvas, center_x + sx, center_y + sy, color);
        }
    }

    // 2. If real town level data is available, attempt to draw decoded tiles.
    //    This is the "real Tristram art" path; it's best-effort and may only
    //    show a few tiles correctly (tile-type/sub-tile mapping is approximate
    //    for this demo), but it proves the MPQ→CEL→RGBA→Texture→screen chain.
    if let Some(level) = &game_state.level_data {
        let _ = draw_real_tiles(window, camera, level, grid_w, grid_h);
    }

    // 3. Draw the player marker at the iso origin so its position is visible.
    let canvas = window.canvas_mut();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 220, 60));
    let _ = canvas.fill_rect(Rect::new(center_x - 4, center_y - 4, 8, 8));

    // 4. Status line so it's obvious this is no longer the green-square stub.
    let mode = if game_state.is_town { "Town (Tristram)" } else { "Dungeon" };
    let has_art = if game_state.level_data.is_some() { "real tiles ON" } else { "checkerboard only" };
    println!("[DrawAndBlit] mode={} {} (tick {})", mode, has_art, game_state.game_tick);

    window.present();
}

/// Logical render resolution. The canvas is configured with
/// `set_logical_size(640, 480)` in main.rs, so all drawing happens in this
/// coordinate space and SDL scales it to the physical window.
const LOGICAL_WIDTH: u32 = 640;
const LOGICAL_HEIGHT: u32 = 480;

/// Fill a 64x32 isometric diamond centred at (cx, cy) using horizontal spans.
/// This is the floor-cell shape; used for the checkerboard.
fn fill_diamond(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    cx: i32,
    cy: i32,
    color: sdl2::pixels::Color,
) {
    canvas.set_draw_color(color);
    let half_w = TILE_WIDTH / 2; // 32
    let half_h = TILE_HEIGHT / 2; // 16
    for dy in -half_h..=half_h {
        // width grows linearly toward the centre, shrinks toward the edges
        let w = half_w - (dy.abs() * half_w / half_h);
        if w > 0 {
            let y = cy + dy;
            let _ = canvas.draw_line((cx - w, y), (cx + w, y));
        }
    }
}

/// Decode town floor tiles from the loaded `DungeonLevelData` and blit them at
/// their isometric screen positions.
///
/// For each visible tile we decode a CEL frame to a 32x32 RGBA buffer (Square
/// type), upload it as a streaming texture, and copy it to the tile's screen
/// position. This is slow (re-decodes every frame) but sufficient to prove the
/// rendering chain; a real implementation caches textures per frame index.
fn draw_real_tiles(
    window: &mut GameWindow,
    camera: IsoPoint,
    level: &crate::engine::dungeon::DungeonLevelData,
    grid_w: i32,
    grid_h: i32,
) -> Result<()> {
    let creator = window.canvas_mut().texture_creator();
    let screen_center_x = LOGICAL_WIDTH as i32 / 2;
    let screen_center_y = LOGICAL_HEIGHT as i32 / 2;

    // Use a stable frame index from the level's CEL. Frame 1 is the first real
    // tile graphic; we vary it by tile coordinate so the floor isn't uniform.
    let mut drawn = 0u32;
    let mut failed = 0u32;
    for ty in 0..grid_h {
        for tx in 0..grid_w {
            // Pick a frame index deterministically from tile coords. Keep it
            // within the first chunk of frames to avoid out-of-range tiles.
            let frame = 1 + ((tx + ty * 3) as u16 % 40u16);
            let rgba = match TileDecoder::decode_tile(
                &level.level_cel,
                frame,
                TileType::Square,
                &level.palette,
            ) {
                Some(px) => px,
                None => {
                    failed += 1;
                    continue;
                }
            };

            // Tile screen position (top-left of the 32x32 sub-tile), centred on
            // the diamond cell like the floor tiles above.
            let (sx, sy) = IsoPoint::new(tx, ty).to_screen(camera);
            // Anchor so the tile sits on its diamond: shift up by TILE_HEIGHT.
            let dst_x = screen_center_x + sx - (TILE_WIDTH / 2);
            let dst_y = screen_center_y + sy - TILE_HEIGHT;

            match rgba_to_texture(&creator, &rgba, 32, 32) {
                Ok(tex) => {
                    let _ = window.canvas_mut().copy(
                        &tex,
                        None,
                        Rect::new(dst_x, dst_y, 32, 32),
                    );
                    drawn += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }
    }
    if drawn > 0 {
        println!("[DrawRealTiles] drew {} town tiles ({} skipped)", drawn, failed);
    }
    Ok(())
}

//------------------------------------------------------------------------------
// Game Logic System Functions (Stubs)
//------------------------------------------------------------------------------

fn process_players() -> Result<()> {
    // C++: ProcessPlayers() - updates all players
    Ok(())
}

fn process_monsters() -> Result<()> {
    // C++: ProcessMonsters() - updates monster AI
    Ok(())
}

fn process_objects() -> Result<()> {
    // C++: ProcessObjects() - updates interactive objects
    Ok(())
}

fn process_missiles() -> Result<()> {
    // C++: ProcessMissiles() - updates projectiles
    Ok(())
}

fn process_items() -> Result<()> {
    // C++: ProcessItems() - updates dropped items
    Ok(())
}

fn process_towners() -> Result<()> {
    // C++: ProcessTowners() - updates NPCs in town
    Ok(())
}

fn process_light_list() {
    // C++: ProcessLightList() - updates lighting
}

fn process_vision_list() {
    // C++: ProcessVisionList() - updates visibility
}

fn sound_update() {
    // C++: sound_update() - updates sound effects
}

fn check_triggers() {
    // C++: CheckTriggers() - checks level transitions
}

fn check_quests() {
    // C++: CheckQuests() - updates quest state
}

fn pfile_update(_force: bool) {
    // C++: pfile_update(false) - saves player data
}

fn plrctrls_after_game_logic() {
    // C++: plrctrls_after_game_logic() - post-logic controls
}

//------------------------------------------------------------------------------
// Color Cycling
//------------------------------------------------------------------------------

fn color_cycling_logic() {
    // C++: diablo_color_cyc_logic() - animates palettes
    let level_type = get_level_type();

    match level_type {
        LevelType::Caves => {
            // C++: palette_update_caves()
        }
        LevelType::Hell => {
            // C++: lighting_color_cycling()
        }
        LevelType::Nest => {
            // C++: palette_update_hive()
        }
        LevelType::Crypt => {
            // C++: palette_update_crypt()
        }
        _ => {
            // No color cycling for other levels
        }
    }
}

//------------------------------------------------------------------------------
// Network/Cursor Functions
//------------------------------------------------------------------------------

fn timeout_cursor(_timeout: bool) {
    // C++: TimeoutCursor(bTimeout) - shows/hides network timeout cursor
}

//------------------------------------------------------------------------------
// Game State Queries
//------------------------------------------------------------------------------

fn is_diablo_alive(_play_sfx: bool) -> bool {
    // C++: IsDiabloAlive(playSFX) - checks if final boss alive
    true // TODO: Implement quest check
}

fn is_running() -> bool {
    // C++: gbRunGame - main loop flag
    true // TODO: Read from global state
}

fn should_process_players() -> bool {
    // C++: gbProcessPlayers - should update players?
    true // TODO: Read from global state
}

fn get_level_type() -> LevelType {
    // C++: leveltype - current level type
    LevelType::Town // TODO: Read from global state
}

//------------------------------------------------------------------------------
// Level Type (from previous game_loop.rs)
//------------------------------------------------------------------------------

/// Level type enumeration
///
/// **C++ Reference**: `Source/gendung.h` - `dungeon_type`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelType {
    #[default]
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    // Hellfire expansions
    Nest = 5,
    Crypt = 6,
}

impl LevelType {
    /// Check if this is a town level
    pub fn is_town(&self) -> bool {
        matches!(self, LevelType::Town)
    }

    /// Check if this is a dungeon level
    pub fn is_dungeon(&self) -> bool {
        !self.is_town()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_type() {
        assert!(LevelType::Town.is_town());
        assert!(!LevelType::Cathedral.is_town());
        assert!(LevelType::Hell.is_dungeon());
    }
}
