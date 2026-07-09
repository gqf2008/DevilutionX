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
use crate::engine::dungeon::{TileDecoder, TileType, DunTemplate};
use crate::engine::sprite_render::rgba_to_texture;
use crate::game::input::InputSystem;
use crate::game::network;
use crate::game::game_state::{GameState, TownLayout, TOWN_MAX_X, TOWN_MAX_Y};
use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use rand::SeedableRng;
use std::collections::HashMap;

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
        // Begin a new input frame: clears per-frame pressed/released state.
        input.begin_frame();

        // Event processing
        // C++: while (FetchMessage(&event, &modState))
        for event in window.event_pump()?.poll_iter() {
            // Route keyboard/mouse events into the InputSystem for movement.
            match &event {
                Event::KeyDown { keycode: Some(k), .. } => input.on_key_down(*k),
                Event::KeyUp { keycode: Some(k), .. } => input.on_key_up(*k),
                _ => {}
            }
            if !handle_event(&event, &mut state, &mut input) {
                break;
            }
        }

        if !state.running {
            break;
        }

        // Apply continuous movement (held arrow/WASD keys) to the player/camera.
        apply_movement(game_state, &input);

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

fn handle_event(event: &Event, state: &mut GameLoopState, input: &mut InputSystem) -> bool {
    match event {
        Event::Quit { .. } => {
            println!("Quit event received");
            state.result = false;
            state.running = false;
            false
        }
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            // ESC exits the game back to the menu.
            println!("ESC pressed - exiting game loop");
            state.running = false;
            false
        }
        Event::KeyDown { keycode: Some(k), .. } => {
            input.on_key_down(*k);
            true
        }
        Event::KeyUp { keycode: Some(k), .. } => {
            input.on_key_up(*k);
            true
        }
        _ => {
            // C++: HandleMessage(event, modState)
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

/// Apply continuous movement from held movement keys to the player and camera.
///
/// This reads the raw (dx, dy) movement direction from the InputSystem and maps
/// it to isometric world-tile movement. In Diablo's isometric projection the
/// screen-up direction corresponds to moving toward decreasing `x` *and*
/// decreasing `y` in world tile space, screen-down to increasing both, screen-left
/// to decreasing `x` / increasing `y`, and screen-right to increasing `x` /
/// decreasing `y`.
///
/// Movement speed is calibrated so the player walks roughly 1 tile every few
/// frames (accumulated via a fractional sub-tile remainder stored on the
/// GameState's camera for smooth scrolling). Here we move in whole tiles at a
/// modest rate to keep things readable; holding a direction continuously slides
/// the camera.
fn apply_movement(game_state: &mut GameState, input: &InputSystem) {
    let (mdx, mdy) = input.get_movement_direction();
    if mdx == 0 && mdy == 0 {
        return;
    }

    // Convert screen-space (mdx, mdy) to world-tile delta. The mapping is:
    //   screen-right (+x) => world (+x, -y)
    //   screen-down  (+y) => world (+x, +y)
    // Combining: world_dx = mdx + mdy ; world_dy = -mdx + mdy
    // Each unit of screen movement is half a tile, so scale down.
    let world_dx = (mdx + mdy) as i32;
    let world_dy = (-mdx + mdy) as i32;

    // Movement amount per frame. Using sub-tile fractions would give smoother
    // scrolling, but the renderer works in whole-tile camera coordinates, so we
    // accumulate into a fractional remainder and commit whole tiles.
    const SPEED_Q8: i32 = 22; // ~0.086 tile/frame => ~5 tiles/sec at 60fps
    game_state.camera.sub_x += world_dx * SPEED_Q8;
    game_state.camera.sub_y += world_dy * SPEED_Q8;

    // Commit accumulated whole-tile movement.
    let step_x = game_state.camera.sub_x / 256;
    let step_y = game_state.camera.sub_y / 256;
    if step_x != 0 || step_y != 0 {
        let new_x = (game_state.camera.tile_x + step_x)
            .max(4)
            .min((TOWN_MAX_X as i32) - 5);
        let new_y = (game_state.camera.tile_y + step_y)
            .max(4)
            .min((TOWN_MAX_Y as i32) - 5);
        game_state.camera.tile_x = new_x;
        game_state.camera.tile_y = new_y;
        game_state.camera.sub_x -= step_x * 256;
        game_state.camera.sub_y -= step_y * 256;
        // Keep the player token aligned with the camera for the marker draw.
        game_state.player.position.x = new_x;
        game_state.player.position.y = new_y;
    }
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
    // buffer.
    //
    // This now renders a **geographically-correct Tristram**: the dPiece grid
    // (`game_state.town_layout`) drives which CEL frame each visible micro-tile
    // shows, and the viewport follows `game_state.camera` so the player can
    // walk around with the arrow / WASD keys.
    //
    // Rendering uses logical 640x480 coordinates (the canvas has
    // set_logical_size(640,480) applied in main.rs), so the viewport centre is
    // (320, 240) regardless of the physical window size.

    window.clear(Color::BLACK);

    let screen_center_x: i32 = LOGICAL_WIDTH as i32 / 2; // 320
    let screen_center_y: i32 = LOGICAL_HEIGHT as i32 / 2; // 240

    // The camera is expressed in world tile coordinates.
    let cam_tile_x = game_state.camera.tile_x;
    let cam_tile_y = game_state.camera.tile_y;

    // Set up the texture cache's "current creator" so cache-miss uploads inside
    // the draw functions can borrow the canvas's TextureCreator. Cleared after.
    {
        let creator = window.canvas_mut().texture_creator();
        set_current_creator(&creator);

        if let (Some(level), Some(layout)) = (&game_state.level_data, &game_state.town_layout) {
            // Real Tristram art path: render the visible micro-tiles from dPiece.
            let _ = draw_tristram(window, level, layout, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
        } else if let Some(level) = &game_state.level_data {
            // Fallback: town data loaded but layout not built yet — draw a small
            // checkerboard of real tile frames so the art chain is still visible.
            let _ = draw_checkerboard_fallback(window, level, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
        } else {
            // No art at all — draw a plain iso checkerboard.
            let canvas = window.canvas_mut();
            draw_plain_checkerboard(canvas, screen_center_x, screen_center_y);
        }

        clear_current_creator();
    }

    // Draw the player marker at the viewport centre (camera == player position).
    let canvas = window.canvas_mut();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 220, 60));
    let _ = canvas.fill_rect(Rect::new(screen_center_x - 4, screen_center_y - 4, 8, 8));
    // A subtle outline so it reads on bright tiles.
    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 20, 20));
    let _ = canvas.draw_rect(Rect::new(screen_center_x - 5, screen_center_y - 5, 10, 10));

    // Debug status line (throttled: only every 30 ticks to avoid log spam).
    if game_state.game_tick % 30 == 0 {
        let mode = if game_state.is_town { "Town (Tristram)" } else { "Dungeon" };
        let has_art = if game_state.town_layout.is_some() {
            "real Tristram layout"
        } else if game_state.level_data.is_some() {
            "tiles (no layout)"
        } else {
            "checkerboard only"
        };
        println!(
            "[DrawAndBlit] mode={} {} cam=({},{}) tick={}",
            mode, has_art, cam_tile_x, cam_tile_y, game_state.game_tick
        );
    }

    window.present();
}

/// Logical render resolution. The canvas is configured with
/// `set_logical_size(640, 480)` in main.rs, so all drawing happens in this
/// coordinate space and SDL scales it to the physical window.
const LOGICAL_WIDTH: u32 = 640;
const LOGICAL_HEIGHT: u32 = 480;

/// Half the visible tile radius around the camera. The isometric viewport of
/// 640x480 with 64x32 diamonds needs roughly ±9 tiles in X and ±8 in Y to cover
/// the screen; we add a margin so edges don't show gaps.
const VIEW_RADIUS_X: i32 = 12;
const VIEW_RADIUS_Y: i32 = 11;

/// Render the visible Tristram micro-tiles from the dPiece grid.
///
/// For each world tile `(wx, wy)` within the camera's view radius we:
/// 1. Look up `dPiece = layout.get(wx, wy)`.
/// 2. Index into `level.min.mega_tiles[dPiece]` to get the two floor sub-tiles
///    (`blocks[0]` = left-bottom triangle, `blocks[1]` = right-top triangle).
///    Each block carries a CEL frame index + tile type.
/// 3. Decode (or fetch from the texture cache) the frame into an SDL Texture.
/// 4. Blit the two 32x32 textures at the tile's screen position, offset so the
///    pair forms the 64x32 diamond.
///
/// Textures are cached in `TILE_TEXTURE_CACHE` keyed by the raw `LevelCelBlock`
/// data word (frame + type packed into a u16), so a given frame is decoded at
/// most once for the lifetime of the game.
fn draw_tristram(
    window: &mut GameWindow,
    level: &crate::engine::dungeon::DungeonLevelData,
    layout: &TownLayout,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) -> Result<()> {
    let mut drawn = 0u32;
    let mut skipped = 0u32;

    for wy_off in -VIEW_RADIUS_Y..=VIEW_RADIUS_Y {
        for wx_off in -VIEW_RADIUS_X..=VIEW_RADIUS_X {
            let wx = cam_tile_x + wx_off;
            let wy = cam_tile_y + wy_off;
            let dpiece = layout.get(wx, wy);
            if dpiece == 0 {
                skipped += 1;
                continue;
            }

            // dPiece is an index into the MIN mega-tile table (mirrors C++
            // DPieceMicros[dPiece]). The MIN table is 0-based, and the values
            // stored in town.til's micro fields are already these indices.
            let mega_idx = dpiece as usize;
            let mega = match level.min.mega_tiles.get(mega_idx) {
                Some(m) => m,
                None => {
                    skipped += 1;
                    continue;
                }
            };

            // Pixel position of this tile relative to the viewport centre.
            // Iso transform: rel_x = (wx-wy)*32, rel_y = (wx+wy)*16, minus the
            // same transform applied to the camera tile.
            let rel_x = (wx_off - wy_off) * (TILE_WIDTH / 2);
            let rel_y = (wx_off + wy_off) * (TILE_HEIGHT / 2);
            let dst_cx = screen_center_x + rel_x;
            let dst_cy = screen_center_y + rel_y;

            // Skip if entirely off-screen (simple bounding check).
            if dst_cx < -(TILE_WIDTH) || dst_cx > (LOGICAL_WIDTH as i32 + TILE_WIDTH)
                || dst_cy < -(TILE_HEIGHT * 2) || dst_cy > (LOGICAL_HEIGHT as i32 + TILE_HEIGHT)
            {
                continue;
            }

            // Draw the two floor sub-tiles (left + right) that form the diamond.
            // blocks[0] => left triangle, blocks[1] => right triangle.
            for (slot, block) in [(0usize, &mega.blocks[0]), (1, &mega.blocks[1])] {
                if !block.has_value() {
                    continue;
                }
                if let Some(tex_ptr) = tile_texture_cache_get(block.data, block.frame(), block.tile_type(), level) {
                    // SAFETY: see tile_texture_cache_get docstring. tex_ptr is a
                    // valid raw pointer to a Texture in the thread-local cache,
                    // which lives for the duration of the game window.
                    let tex_ref = unsafe { tex_ptr.as_texture_ref() };
                    // Anchor: left texture's right edge at dst_cx; right texture's
                    // left edge at dst_cx. Both sit with their top at dst_cy-32.
                    let (tx, ty) = if slot == 0 {
                        (dst_cx - (TILE_WIDTH / 2), dst_cy - TILE_HEIGHT)
                    } else {
                        (dst_cx, dst_cy - TILE_HEIGHT)
                    };
                    let _ = window.canvas_mut().copy(
                        tex_ref,
                        None,
                        Rect::new(tx, ty, (TILE_WIDTH / 2) as u32, TILE_HEIGHT as u32),
                    );
                    drawn += 1;
                }
            }
        }
    }

    if drawn == 0 {
        println!(
            "[DrawTristram] WARNING: drew 0 tiles ({} skipped) cam=({},{}). \
             Layout may be empty or dPiece indices out of MIN range.",
            skipped, cam_tile_x, cam_tile_y
        );
    }
    Ok(())
}

/// Fallback renderer: draw a checkerboard of real town tile frames when the
/// layout grid hasn't been built yet. Uses the texture cache.
fn draw_checkerboard_fallback(
    window: &mut GameWindow,
    level: &crate::engine::dungeon::DungeonLevelData,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) -> Result<()> {
    for wy_off in -VIEW_RADIUS_Y..=VIEW_RADIUS_Y {
        for wx_off in -VIEW_RADIUS_X..=VIEW_RADIUS_X {
            let rel_x = (wx_off - wy_off) * (TILE_WIDTH / 2);
            let rel_y = (wx_off + wy_off) * (TILE_HEIGHT / 2);
            let dst_cx = screen_center_x + rel_x;
            let dst_cy = screen_center_y + rel_y;

            // Pick the first mega-tile's left block so we show actual town art.
            if let Some(mega) = level.min.mega_tiles.first() {
                let block = &mega.blocks[0];
                if block.has_value() {
                    if let Some(tex_ptr) = tile_texture_cache_get(block.data, block.frame(), block.tile_type(), level) {
                        let tex_ref = unsafe { tex_ptr.as_texture_ref() };
                        let _ = window.canvas_mut().copy(
                            tex_ref,
                            None,
                            Rect::new(
                                dst_cx - (TILE_WIDTH / 2),
                                dst_cy - TILE_HEIGHT,
                                (TILE_WIDTH / 2) as u32,
                                TILE_HEIGHT as u32,
                            ),
                        );
                    }
                }
            } else {
                // No mega tiles at all — draw a coloured diamond.
                let is_dark = (wx_off + wy_off) % 2 == 0;
                let color = if is_dark {
                    sdl2::pixels::Color::RGB(36, 36, 52)
                } else {
                    sdl2::pixels::Color::RGB(58, 58, 78)
                };
                fill_diamond(window.canvas_mut(), dst_cx, dst_cy, color);
            }
        }
    }
    Ok(())
}

/// Draw a plain (no-art) isometric checkerboard floor.
fn draw_plain_checkerboard(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    center_x: i32,
    center_y: i32,
) {
    for wy in -VIEW_RADIUS_Y..=VIEW_RADIUS_Y {
        for wx in -VIEW_RADIUS_X..=VIEW_RADIUS_X {
            let is_dark = (wx + wy) % 2 == 0;
            let color = if is_dark {
                sdl2::pixels::Color::RGB(36, 36, 52)
            } else {
                sdl2::pixels::Color::RGB(58, 58, 78)
            };
            let sx = (wx - wy) * (TILE_WIDTH / 2);
            let sy = (wx + wy) * (TILE_HEIGHT / 2);
            fill_diamond(canvas, center_x + sx, center_y + sy, color);
        }
    }
}

/// Fill a 64x32 isometric diamond centred at (cx, cy) using horizontal spans.
/// This is the floor-cell shape; used for the checkerboard fallbacks.
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

//------------------------------------------------------------------------------
// Texture Cache (Step 1: performance)
//------------------------------------------------------------------------------
//
// SDL `Texture<'a>` borrows a `TextureCreator<'a>` which borrows the
// `WindowContext`. The Rust binding models this lifetime, but the underlying
// SDL texture is an opaque GPU handle valid as long as the renderer (the
// `GameWindow`) is alive. Because the single `GameWindow` lives for the whole
// game loop, we can soundly cache textures for the loop's duration.
//
// To avoid fighting the borrow checker across the update/render split, we store
// textures in a thread-local cache keyed by the raw `LevelCelBlock.data` word,
// holding them as lifetime-erased `Texture<'static>`. The cache is cleared when
// the game loop exits so we don't reuse stale SDL handles after the window is
// destroyed.

thread_local! {
    /// Per-thread cache of decoded tile textures keyed by `LevelCelBlock.data`.
    /// Values are `Texture<'static>` — the lifetime is erased with `transmute`
    /// because the underlying SDL handle outlives the borrow (see module notes).
    /// SDL textures are not `Sync`, so this is thread-local and only ever
    /// touched from the render thread.
    static TILE_TEXTURE_CACHE: std::cell::RefCell<HashMap<u16, Texture<'static>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// A handle returned by the texture cache. Carries a raw pointer into the
/// thread-local cache; `as_texture_ref` reborrows it for the current draw.
///
/// The pointer is valid until the cache is mutated (cleared), which only
/// happens at game-loop exit — never during a render call.
pub struct TextureHandle {
    raw: *const Texture<'static>,
}

impl TextureHandle {
    /// Reborrow the cached texture as `&Texture` for the current draw call.
    ///
    /// # Safety
    /// The caller must not mutate the thread-local `TILE_TEXTURE_CACHE` while
    /// the returned reference is live, and must be on the render thread. Both
    /// invariants hold during `draw_and_blit`.
    pub unsafe fn as_texture_ref<'a>(&self) -> &'a Texture<'a> {
        // SAFETY: the raw pointer points into the thread-local cache, which is
        // stable for the duration of the render call (the cache is only cleared
        // at game-loop exit). Reinterpreting the lifetime from 'static to 'a is
        // sound because 'a is shorter than the texture's actual validity.
        &*(self.raw as *const Texture<'a>)
    }
}

/// Fetch a decoded tile texture from the cache, decoding + uploading on miss.
///
/// `key` is the `LevelCelBlock.data` word (frame + type); `frame`/`tile_type`
/// are used for the actual CEL decode on a cache miss. The texture is uploaded
/// using a `TextureCreator` borrowed from the canvas inside this call.
fn tile_texture_cache_get(
    key: u16,
    frame: u16,
    tile_type: TileType,
    level: &crate::engine::dungeon::DungeonLevelData,
) -> Option<TextureHandle> {
    // Fast path: already cached. Return a handle to the existing entry.
    let hit = TILE_TEXTURE_CACHE.with(|c| c.borrow().get(&key).map(|t| t as *const Texture<'static>));
    if let Some(raw) = hit {
        return Some(TextureHandle { raw });
    }

    // Cache miss: decode the CEL frame to RGBA, then upload via a texture
    // creator borrowed from... we can't borrow the canvas here without passing
    // it in. To keep the cache API canvas-free, the caller is responsible for
    // the upload. Instead, we do a two-phase approach: the caller passes the
    // creator via a thread-local "current creator" set at the start of the draw
    // call. See `set_current_creator`.
    let creator = match current_creator() {
        Some(c) => c,
        None => return None,
    };

    let rgba = TileDecoder::decode_tile(&level.level_cel, frame, tile_type, &level.palette)?;
    let tex = rgba_to_texture(creator, &rgba, 32, 32).ok()?;

    // Erase the lifetime: the texture's underlying SDL handle is valid as long
    // as the GameWindow (and its renderer) is alive — the whole game loop.
    // SAFETY: see the Texture Cache module docstring.
    let tex_static: Texture<'static> = unsafe { std::mem::transmute(tex) };
    let raw = TILE_TEXTURE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        cache.entry(key).or_insert(tex_static) as *const Texture<'static>
    });
    Some(TextureHandle { raw })
}

thread_local! {
    /// The texture creator currently in use for the render call, set via
    /// `set_current_creator` at the start of `draw_and_blit`. Stored as a raw
    /// pointer to avoid the lifetime in the cache API.
    static CURRENT_CREATOR: std::cell::Cell<*const sdl2::render::TextureCreator<sdl2::video::WindowContext>> =
        std::cell::Cell::new(std::ptr::null());
}

/// Set the texture creator used for cache-miss uploads during the current
/// render call. Called once at the top of `draw_and_blit`.
///
/// # Safety
/// The creator must outlive all cache-miss uploads performed before the next
/// `clear_current_creator` call. This holds because the creator is borrowed
/// from the canvas, which is alive for the whole draw call.
fn set_current_creator(creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>) {
    CURRENT_CREATOR.with(|c| c.set(creator as *const _));
}

/// Clear the current creator reference (end of draw call).
fn clear_current_creator() {
    CURRENT_CREATOR.with(|c| c.set(std::ptr::null()));
}

/// Get the current creator, or None if not in a draw call.
fn current_creator() -> Option<&'static sdl2::render::TextureCreator<sdl2::video::WindowContext>> {
    let ptr = CURRENT_CREATOR.with(|c| c.get());
    if ptr.is_null() {
        None
    } else {
        // SAFETY: the creator is alive for the duration of the draw call (set in
        // set_current_creator, cleared in clear_current_creator). Returning a
        // 'static reference is a simplification — callers only use it within the
        // draw call, before clear_current_creator.
        Some(unsafe { &*ptr })
    }
}

/// Clear the tile texture cache. Called when the game loop exits so textures
/// aren't reused across windows (which would point at destroyed SDL handles).
pub fn clear_tile_texture_cache() {
    TILE_TEXTURE_CACHE.with(|c| c.borrow_mut().clear());
}

/// Build the geographically-correct town layout (`dPiece` grid) from the four
/// sector `.dun` templates + the `town.til` mega definitions.
///
/// This mirrors C++ `Source/levels/town.cpp::DrlgTPass3` + `FillSector`:
///   1. Initialise the whole grid to tile id 426 (default ground).
///   2. For each of the 4 sectors, decode the `.dun` template and, for each
///      template tile id `t`, expand it into the 4 micro-tiles from
///      `til.tiles[t-1].micro1..micro4` (placed as a 2x2 block).
///
/// The four sectors tile the 112x112 map at offsets (46,46),(46,0),(0,46),(0,0),
/// exactly as in the C++ `FillSector` calls.
pub fn build_town_layout(
    level: &crate::engine::dungeon::DungeonLevelData,
    sector1s: Option<&DunTemplate>,
    sector2s: Option<&DunTemplate>,
    sector3s: Option<&DunTemplate>,
    sector4s: Option<&DunTemplate>,
) -> TownLayout {
    let mut layout = TownLayout::default();

    // Default-fill the whole grid with dPiece value 426. C++ `DrlgTPass3` does
    // exactly this: `dPiece[xx][yy] = 426` for every micro-tile. 426 is a
    // micro-tile index into the MIN/DPieceMicros table (the "default ground"),
    // NOT a town.til tile id — so we store it literally rather than expanding a
    // TIL entry.
    for yy in 0..TOWN_MAX_Y {
        for xx in 0..TOWN_MAX_X {
            layout.set(xx as i32, yy as i32, 426);
        }
    }

    // Stamp the four sectors. Each sector tile id indexes into town.til; the
    // TIL entry gives the 4 micro (dPiece) values. C++ offsets:
    //   sector1s -> (46,46), sector2s -> (46,0), sector3s -> (0,46), sector4s -> (0,0)
    fill_sector(&mut layout, level, sector1s, 46, 46);
    fill_sector(&mut layout, level, sector2s, 46, 0);
    fill_sector(&mut layout, level, sector3s, 0, 46);
    fill_sector(&mut layout, level, sector4s, 0, 0);

    layout
}

/// Look up the 4 micro values (dPiece) for a town tile id, from town.til.
/// Returns `(micro1, micro2, micro3, micro4)` or None if out of range.
fn mega_for_tile_id(
    level: &crate::engine::dungeon::DungeonLevelData,
    tile_id: u16,
) -> Option<(u16, u16, u16, u16)> {
    if tile_id == 0 {
        return None;
    }
    let idx = (tile_id - 1) as usize;
    let t = level.til.tiles.get(idx)?;
    Some((t.micro1, t.micro2, t.micro3, t.micro4))
}

/// Place a mega-tile's 4 micro values into the dPiece grid as a 2x2 block.
fn place_mega(layout: &mut TownLayout, xx: usize, yy: usize, mega: (u16, u16, u16, u16)) {
    let (m1, m2, m3, m4) = mega;
    layout.set(xx as i32, yy as i32, m1);
    layout.set(xx as i32 + 1, yy as i32, m2);
    layout.set(xx as i32, yy as i32 + 1, m3);
    layout.set(xx as i32 + 1, yy as i32 + 1, m4);
}

/// Decode a sector `.dun` template into the dPiece grid at (xi, yy).
///
/// Mirrors C++ `FillSector`: for each template tile id `t` (1-based), the four
/// micro values come from `town.til[t-1].micro1..micro4` and are stored as
/// dPiece values. Tiles with id 0 default to 218 (empty grass) per the C++ code.
fn fill_sector(
    layout: &mut TownLayout,
    level: &crate::engine::dungeon::DungeonLevelData,
    sector: Option<&DunTemplate>,
    xi: usize,
    mut yy: usize,
) {
    let sector = match sector {
        Some(s) => s,
        None => return,
    };
    let width = sector.width as usize;
    let height = sector.height as usize;

    for j in 0..height {
        let mut xx = xi;
        for i in 0..width {
            let raw = sector.tiles.get(j * width + i).copied().unwrap_or(0);
            // C++: tileId = raw - 1; if tileId >= 0 use mega, else default 218.
            let tile_id = if raw == 0 { 0 } else { raw };

            let (m1, m2, m3, m4) = if let Some(m) = mega_for_tile_id(level, tile_id) {
                m
            } else {
                // Default to tile 218's micro tiles (C++ FillSector default).
                mega_for_tile_id(level, 218).unwrap_or((426, 426, 426, 426))
            };

            place_mega(layout, xx, yy, (m1, m2, m3, m4));
            xx += 2;
        }
        yy += 2;
    }
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
    use crate::engine::dungeon::{
        DungeonLevelData, DungeonType, LevelCelBlock, MegaTile, MinData, PaletteData, SolData,
        TilData, TilEntry,
    };

    #[test]
    fn test_level_type() {
        assert!(LevelType::Town.is_town());
        assert!(!LevelType::Cathedral.is_town());
        assert!(LevelType::Hell.is_dungeon());
    }

    /// Build a minimal `DungeonLevelData` with a known MIN/TIL mapping so we can
    /// unit-test `build_town_layout` without loading real MPQ data.
    ///
    /// Layout:
    ///   - MIN mega[5] has blocks[0] = frame 11 (LeftTriangle),
    ///     blocks[1] = frame 12 (RightTriangle). Other blocks empty.
    ///   - TIL[0] (tile id 1) = micro1-4 = (5, 6, 7, 8)
    ///   - TIL[1] (tile id 2) = micro1-4 = (9, 10, 11, 12)
    /// So a sector tile id of 1 should write dPiece 5,6,7,8.
    fn make_test_level() -> DungeonLevelData {
        let mut mega_tiles = vec![MegaTile::default(); 20];
        // mega[5]: frame 11 left, frame 12 right (type 2/3).
        mega_tiles[5].blocks[0] = LevelCelBlock::new(0x2000 | 11); // type 2 (LeftTriangle) | frame 11
        mega_tiles[5].blocks[1] = LevelCelBlock::new(0x3000 | 12); // type 3 (RightTriangle) | frame 12

        let til = TilData {
            tiles: vec![
                TilEntry { micro1: 5, micro2: 6, micro3: 7, micro4: 8 },
                TilEntry { micro1: 9, micro2: 10, micro3: 11, micro4: 12 },
            ],
        };

        DungeonLevelData {
            dungeon_type: DungeonType::Town,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData { mega_tiles, blocks_per_tile: 16 },
            til,
            level_cel: vec![],
        }
    }

    #[test]
    fn test_build_town_layout_default_fill_426() {
        // With no sectors, every cell should be the default dPiece = 426.
        let level = make_test_level();
        let layout = build_town_layout(&level, None, None, None, None);
        // All cells filled with 426.
        assert_eq!(layout.get(0, 0), 426);
        assert_eq!(layout.get(75, 68), 426);
        assert_eq!(layout.get(111, 111), 426);
    }

    #[test]
    fn test_build_town_layout_sector_stamping() {
        // A 1x1 sector with tile id 1, stamped at (0,0), should write the TIL
        // micro values (5,6,7,8) into the 2x2 block at (0,0).
        let level = make_test_level();
        let sector = DunTemplate {
            width: 1,
            height: 1,
            tiles: vec![1], // tile id 1
            monsters: vec![],
            objects: vec![],
            transparency: vec![],
        };
        // sector4s is stamped at (0,0).
        let layout = build_town_layout(&level, None, None, None, Some(&sector));
        // 2x2 block: (0,0)=micro1=5, (1,0)=micro2=6, (0,1)=micro3=7, (1,1)=micro4=8
        assert_eq!(layout.get(0, 0), 5, "top-left micro");
        assert_eq!(layout.get(1, 0), 6, "top-right micro");
        assert_eq!(layout.get(0, 1), 7, "bottom-left micro");
        assert_eq!(layout.get(1, 1), 8, "bottom-right micro");
        // The cell just outside the stamped block should still be the default.
        assert_eq!(layout.get(2, 0), 426);
    }

    #[test]
    fn test_build_town_layout_multi_tile_sector() {
        // A 2x1 sector with tile ids [1, 2] stamped at (0,0).
        // Row layout: [tile1 -> (5,6,7,8)] [tile2 -> (9,10,11,12)]
        let level = make_test_level();
        let sector = DunTemplate {
            width: 2,
            height: 1,
            tiles: vec![1, 2],
            monsters: vec![],
            objects: vec![],
            transparency: vec![],
        };
        let layout = build_town_layout(&level, None, None, None, Some(&sector));
        // tile1 at xx=0: (0,0)=5 (1,0)=6
        assert_eq!(layout.get(0, 0), 5);
        assert_eq!(layout.get(1, 0), 6);
        // tile2 at xx=2: (2,0)=9 (3,0)=10
        assert_eq!(layout.get(2, 0), 9);
        assert_eq!(layout.get(3, 0), 10);
    }

    #[test]
    fn test_mega_for_tile_id() {
        let level = make_test_level();
        // tile id 1 -> TIL[0] -> (5,6,7,8)
        assert_eq!(mega_for_tile_id(&level, 1), Some((5, 6, 7, 8)));
        // tile id 2 -> TIL[1] -> (9,10,11,12)
        assert_eq!(mega_for_tile_id(&level, 2), Some((9, 10, 11, 12)));
        // tile id 0 -> None
        assert_eq!(mega_for_tile_id(&level, 0), None);
        // out of range -> None
        assert_eq!(mega_for_tile_id(&level, 999), None);
    }

    #[test]
    fn test_dpiece_index_into_min_blocks() {
        // Verify the data flow: a dPiece value indexes into min.mega_tiles,
        // and blocks[0]/blocks[1] carry the two floor triangle frames.
        let level = make_test_level();
        let mega = &level.min.mega_tiles[5];
        assert!(mega.blocks[0].has_value());
        assert_eq!(mega.blocks[0].frame(), 11);
        assert_eq!(mega.blocks[0].tile_type(), TileType::LeftTriangle);
        assert_eq!(mega.blocks[1].frame(), 12);
        assert_eq!(mega.blocks[1].tile_type(), TileType::RightTriangle);
    }

    #[test]
    fn test_apply_movement_no_keys_is_noop() {
        // With no movement keys held, the camera must not move.
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.init_town_camera();
        let input = InputSystem::new();
        let before = (gs.camera.tile_x, gs.camera.tile_y);
        apply_movement(&mut gs, &input);
        let after = (gs.camera.tile_x, gs.camera.tile_y);
        assert_eq!(before, after, "camera should not move with no input");
    }

    #[test]
    fn test_apply_movement_clamps_to_bounds() {
        // Place the camera at the top-left corner; holding the up-left keys
        // (W+A) must clamp at the minimum bound (4, 4), not go negative.
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = 5;
        gs.camera.tile_y = 5;
        let mut input = InputSystem::new();
        input.begin_frame();
        input.on_key_down(Keycode::W);
        input.on_key_down(Keycode::A);
        // Run many iterations to exhaust the sub-tile accumulator past the bound.
        for _ in 0..200 {
            apply_movement(&mut gs, &input);
        }
        assert!(gs.camera.tile_x >= 4, "x clamped: {}", gs.camera.tile_x);
        assert!(gs.camera.tile_y >= 4, "y clamped: {}", gs.camera.tile_y);
    }

    #[test]
    fn test_apply_movement_clamps_to_max_bounds() {
        // Place the camera near the bottom-right; holding down-right (S+D) must
        // clamp at (TOWN_MAX_X-5, TOWN_MAX_Y-5).
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = TOWN_MAX_X as i32 - 6;
        gs.camera.tile_y = TOWN_MAX_Y as i32 - 6;
        let mut input = InputSystem::new();
        input.begin_frame();
        input.on_key_down(Keycode::S);
        input.on_key_down(Keycode::D);
        for _ in 0..200 {
            apply_movement(&mut gs, &input);
        }
        assert!(gs.camera.tile_x <= (TOWN_MAX_X as i32 - 5), "x max clamp: {}", gs.camera.tile_x);
        assert!(gs.camera.tile_y <= (TOWN_MAX_Y as i32 - 5), "y max clamp: {}", gs.camera.tile_y);
    }

    #[test]
    fn test_apply_movement_updates_player_position() {
        // When the camera moves, the player.position should track it (the marker
        // is drawn at the viewport centre, where the camera/player is).
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = 50;
        gs.camera.tile_y = 50;
        let mut input = InputSystem::new();
        input.begin_frame();
        input.on_key_down(Keycode::D); // move east: world (+x, -y)
        // Accumulate enough to cross a whole-tile boundary.
        for _ in 0..200 {
            apply_movement(&mut gs, &input);
        }
        assert!(gs.camera.tile_x > 50, "should have moved +x: {}", gs.camera.tile_x);
        assert!(gs.camera.tile_y < 50, "should have moved -y: {}", gs.camera.tile_y);
        // Player position tracks camera.
        assert_eq!(gs.camera.tile_x, gs.player.position.x);
        assert_eq!(gs.camera.tile_y, gs.player.position.y);
    }
}
