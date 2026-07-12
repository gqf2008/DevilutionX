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
use crate::engine::dungeon::{TileDecoder, DunTemplate};
use crate::engine::sprite_render::rgba_to_texture;
use crate::game::input::InputSystem;
use crate::game::network;
use crate::game::game_state::{GameState, TownLayout, TOWN_MAX_X, TOWN_MAX_Y};
use crate::game::hud;
use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use rand::SeedableRng;
use rand::Rng;

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
    /// Current mouse position in logical (640x480) coords. Updated from
    /// MouseMotion events (SDL2 logical_size auto-converts them). Used to
    /// draw the in-game cursor so the player can see where they're pointing.
    /// C++ equivalent: `MousePosition` global (diablo.cpp:136).
    mouse_pos: (i32, i32),
}

impl GameLoopState {
    fn new() -> Self {
        Self {
            running: true,
            startup: true,
            process_players: true,
            result: true,
            mouse_pos: (320, 240),
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
pub fn run_game_loop(mode: InterfaceMode, window: &mut GameWindow, game_state: &mut GameState, event_pump: &mut sdl2::EventPump) -> Result<bool> {
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
        // Reuse the single shared EventPump created at startup — SDL2 allows
        // only one EventPump alive at a time, so we must not create another
        // via window.event_pump() here (that was the cause of the "an
        // EventPump instance is already alive" panic on entering the game).
        for event in event_pump.poll_iter() {
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

        // Town <-> Dungeon toggle keys. 'D' descends into L1 Cathedral (only
        // when currently in town); 'T' returns to town (only when in the
        // dungeon). These are the simplified staircase triggers; once true
        // stair-tile detection is wired in they can be replaced by it.
        if input.is_key_pressed(Keycode::D) && !game_state.in_dungeon {
            if let Err(e) = descend_to_dungeon(game_state) {
                println!("[GameLoop] descend_to_dungeon failed: {}", e);
            }
        }
        if input.is_key_pressed(Keycode::T) && game_state.in_dungeon {
            return_to_town(game_state);
        }

        // F5 = quick-save the current game to slot 0, F9 = quick-load from
        // slot 0. These exercise the full save/load round-trip (player vitals
        // in 64x fixed-point, position, level, tick). Used edge-triggered so a
        // single key tap saves/loads exactly once.
        if input.is_key_pressed(Keycode::F5) {
            match game_state.save_to_slot(0) {
                Ok(path) => {
                    println!(
                        "[Save] saved hero '{}' (L{}, XP {}, HP {}/{}) to {}",
                        game_state.player.get_name(),
                        game_state.player._p_level,
                        game_state.player._p_experience,
                        game_state.player._p_hit_points,
                        game_state.player._p_max_hp,
                        path
                    );
                }
                Err(e) => println!("[Save] FAILED: {}", e),
            }
        }
        if input.is_key_pressed(Keycode::F9) {
            match game_state.load_from_slot(0) {
                Ok(()) => {
                    println!(
                        "[Load] restored hero '{}' (L{}, XP {}, HP {}/{}) @ ({},{})",
                        game_state.player.get_name(),
                        game_state.player._p_level,
                        game_state.player._p_experience,
                        game_state.player._p_hit_points,
                        game_state.player._p_max_hp,
                        game_state.player.position.x,
                        game_state.player.position.y,
                    );
                }
                Err(e) => println!("[Load] FAILED: {}", e),
            }
        }

        // 'F' = cast Firebolt toward the nearest monster (demo spell-casting).
        // Edge-triggered so one tap fires one bolt. Mana cost is applied
        // inside the cast; nothing happens if mana is insufficient.
        if input.is_key_pressed(Keycode::F) && game_state.in_dungeon {
            let _ = game_state.cast_firebolt_at_nearest();
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
                draw_and_blit(window, game_state, state.mouse_pos);
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
            draw_and_blit(window, game_state, state.mouse_pos);
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
        // Track the mouse in logical coords so we can draw an in-game cursor.
        // SDL2 logical_size auto-converts event coords, matching C++ (which
        // is a no-op on SDL2 in events.cpp). C++ stores this in the global
        // `MousePosition` (diablo.cpp:136) and uses it for click-to-move.
        Event::MouseMotion { x, y, .. } => {
            state.mouse_pos = (*x, *y);
            true
        }
        Event::MouseButtonDown { x, y, .. } => {
            state.mouse_pos = (*x, *y);
            true
        }
        Event::MouseButtonUp { x, y, .. } => {
            state.mouse_pos = (*x, *y);
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

/// Descend from town into the L1 Cathedral dungeon.
///
/// Generates the Cathedral layout from `game_state.dungeon_level_data` (loaded
/// once in `start_game`) using the faithful `drlg_l1::CathedralGenerator`, sets
/// `in_dungeon = true`, and re-centres the camera on the generated dungeon.
/// Prints diagnostics about the generation. Returns an error string if the L1
/// art is unavailable.
pub fn descend_to_dungeon(game_state: &mut GameState) -> Result<(), String> {
    println!("[Descend] Generating L1 Cathedral...");
    let level = match &game_state.dungeon_level_data {
        Some(l) => l,
        None => return Err("L1 Cathedral art not loaded (dungeon_level_data is None)".to_string()),
    };

    // Generate + map to dPiece grid. Use the game seed so the level is stable.
    let seed = (game_state.game_tick as u32).wrapping_add(0xC0FFEE);
    let layout = crate::game::dungeon_level::generate_l1_cathedral(seed, level);
    let filled = crate::game::dungeon_level::count_filled(&layout);
    println!(
        "[Descend] L1 Cathedral generated (seed {}): {}x{}, {} non-zero dPiece cells, {} TIL megas",
        seed, layout.width, layout.height, filled, level.til.tiles.len()
    );

    game_state.dungeon_layout = Some(layout);
    game_state.in_dungeon = true;
    // Keep is_town in sync so GameState::update's monster/item logic matches the
    // active mode (dungeon processes monsters; town skips them).
    game_state.is_town = false;

    // Centre the camera on the middle of the 40×40 dungeon active region. The
    // DRLG_LPass3 stamping starts at micro offset (16,16), so tile (20,20)
    // (logical) maps to roughly (16 + 20*2, 16 + 20*2) = (56, 56) in micro
    // space. We place the camera there so the player starts in the dungeon
    // interior.
    let center_x = 16 + 20 * 2;
    let center_y = 16 + 20 * 2;
    game_state.camera.tile_x = center_x;
    game_state.camera.tile_y = center_y;
    game_state.camera.sub_x = 0;
    game_state.camera.sub_y = 0;
    game_state.player.position.x = center_x;
    game_state.player.position.y = center_y;

    println!("[Descend] Entered L1 Cathedral at ({}, {})", center_x, center_y);

    // Step 1: place monsters on the dungeon floor. We clear any stale monsters
    // (e.g. from a previous descent) and scatter a small pack across walkable
    // floor tiles away from the player spawn. Also pre-load their stand sprites
    // so the renderer can draw real CL2 art (with a coloured-block fallback).
    place_dungeon_monsters(game_state, center_x, center_y);

    Ok(())
}

/// Number of monsters to scatter across an L1 Cathedral level. Kept small so
/// the dungeon feels populated without crowding the shareware-sized rooms.
const DUNGEON_MONSTER_COUNT: usize = 8;

/// Place a small pack of monsters on the dungeon floor, away from the player
/// spawn, and pre-load their stand sprites for rendering.
///
/// Monsters are drawn from the Cathedral-appropriate types (Zombie, Fallen One,
/// Skeleton, Scavenger, ...). Each is placed on a random walkable floor tile
/// (from `dungeon_layout.floor_tiles`) at least ~8 micro-tiles from the spawn
/// point so the player doesn't immediately aggro the whole pack. The monsters
/// live in `game_state.monster_manager`; their sprites are cached in
/// `game_state.monster_sprites`.
///
/// Non-fatal: if the layout has no floor tiles or the MPQ can't be opened, we
/// simply place fewer/no monsters and the renderer falls back to coloured blocks.
fn place_dungeon_monsters(game_state: &mut GameState, spawn_x: i32, spawn_y: i32) {
    // Start each descent with a clean monster roster.
    game_state.monster_manager.clear();
    game_state.monster_sprites = None;

    // Candidate spawn tiles: floor tiles far enough from the player spawn.
    let floor_tiles: Vec<(i32, i32)> = match &game_state.dungeon_layout {
        Some(l) => l
            .floor_tiles
            .iter()
            .copied()
            .filter(|(x, y)| (x - spawn_x).abs() + (y - spawn_y).abs() >= 8)
            .collect(),
        None => Vec::new(),
    };

    if floor_tiles.is_empty() {
        println!("[Monsters] no floor tiles available; placing no monsters");
        return;
    }

    // Cathedral-appropriate monster types (matches MonsterType::for_dungeon).
    use crate::game::monster::MonsterType;
    let types = [
        MonsterType::Zombie,
        MonsterType::FallenOne,
        MonsterType::Skeleton,
        MonsterType::Scavenger,
        MonsterType::SkeletonArcher,
    ];

    let mut rng = rand::rng();
    let mut placed = 0usize;
    let mut used_types: Vec<MonsterType> = Vec::new();
    let mut occupied: Vec<(i32, i32)> = Vec::new();

    for _ in 0..DUNGEON_MONSTER_COUNT {
        // Pick a random floor tile not already occupied by another monster.
        let mut attempts = 0;
        let (tx, ty) = loop {
            let idx = rng.random_range(0..floor_tiles.len());
            let p = floor_tiles[idx];
            if !occupied.contains(&p) {
                break p;
            }
            attempts += 1;
            if attempts > 16 {
                break p;
            }
        };
        occupied.push((tx, ty));

        let monster_type = types[rng.random_range(0..types.len())];
        // Build the monster on its spawn tile. `Monster::new` seeds enemy/target
        // with the monster's own tile (a safe no-op until `update_ai` repoints
        // them at the player) and home_x/home_y with the spawn tile (used to
        // bound idle wandering).
        let mut m = crate::game::monster::Monster::new(
            placed as u32 + 1,
            monster_type,
            tx,
            ty,
            1, // level modifier
        );
        // Seed the enemy position with the player spawn so the first AI tick can
        // compute distance to the player.
        m.enemy_position = crate::game::types::Point::new(spawn_x, spawn_y);
        // Start idle (stand) — the AI step flips to Chasing when the player is
        // within aggro_range (8 tiles).
        m.ai_state = crate::game::monster::MonsterAIState::Idle;
        m.mode = crate::game::monster::MonsterMode::Stand;

        game_state.monster_manager.add_monster(m);
        used_types.push(monster_type);
        placed += 1;
    }

    println!(
        "[Monsters] placed {} monsters on floor tiles (types: {:?})",
        placed,
        used_types
            .iter()
            .map(|t| t.name())
            .collect::<std::collections::BTreeSet<_>>()
    );

    // Pre-load stand sprites for the placed types. Opens the local MPQ directly
    // (spawn.mpq / diabdat.mpq) since the game loop doesn't own the asset
    // manager. Non-fatal: if it fails, the renderer uses coloured blocks.
    if let Some(sheet) = load_monster_sprite_set(&used_types) {
        game_state.monster_sprites = Some(sheet);
    } else {
        println!("[Monsters] monster sprites unavailable; renderer will use coloured blocks");
    }
}

/// Open the local game MPQ (spawn.mpq shareware, or diabdat.mpq full) and load
/// the stand sprites for the given monster types into a [`MonsterSpriteSet`].
///
/// Searches a few candidate paths (cargo root, parent dir, current dir) so this
/// works whether the binary runs from the cargo root or a build output dir.
/// Returns `None` if no MPQ can be opened.
fn load_monster_sprite_set(types: &[crate::game::monster::MonsterType]) -> Option<crate::game::monster_sprites::MonsterSpriteSet> {
    use std::path::PathBuf;

    // Candidate MPQ locations. The binary may run from the cargo root, a
    // subdirectory, or an absolute install path.
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("spawn.mpq"));
            candidates.push(dir.join("diabdat.mpq"));
            candidates.push(dir.join("DIABDAT.MPQ"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("spawn.mpq"));
        candidates.push(cwd.join("diabdat.mpq"));
        candidates.push(cwd.join("DIABDAT.MPQ"));
    }

    let mpq_path = candidates.into_iter().find(|p| p.exists())?;
    let mut archive = crate::engine::mpq::MpqArchive::open(&mpq_path).ok()?;
    Some(crate::game::monster_sprites::MonsterSpriteSet::load(
        &mut archive,
        types,
    ))
}

/// Return from the dungeon to Tristram town. Restores the town camera spawn
/// and clears the dungeon layout so a fresh one is generated next descent.
pub fn return_to_town(game_state: &mut GameState) {
    println!("[Return] Leaving dungeon, returning to Tristram");
    game_state.in_dungeon = false;
    game_state.is_town = true;
    game_state.dungeon_layout = None;
    // Clear dungeon monsters + their sprites so a fresh pack is generated on the
    // next descent.
    game_state.monster_manager.clear();
    game_state.monster_sprites = None;
    // Drop the uploaded monster-sprite textures so a fresh pack re-uploads on
    // the next descent (the monsters may be different types).
    clear_monster_sprite_cache();
    // Restore the town spawn (C++ ENTRY_MAIN ViewPosition {75, 68}).
    game_state.init_town_camera();
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

fn draw_and_blit(window: &mut GameWindow, game_state: &GameState, mouse_pos: (i32, i32)) {
    // C++: DrawAndBlit() - renders the dungeon viewport then flips the back
    // buffer.
    //
    // This now renders a **geographically-correct Tristram**: the dPiece grid
    // (`game_state.town_layout`) drives which CEL frame each visible micro-tile
    // shows, and the viewport follows `game_state.camera` so the player can
    // walk around with the arrow / WASD keys.
    //
    // Rendering uses logical 640x480 coordinates (the canvas has
    // set_logical_size(640,480) applied in main.rs).

    window.clear(Color::BLACK);

    let screen_center_x: i32 = LOGICAL_WIDTH as i32 / 2; // 320
    // C++ scrolls the dungeon so the player sits at the vertical centre of the
    // *viewport* (screen height minus the bottom panel), not the screen centre
    // (scrollrt.cpp CalcViewportGeometry: playerPosition.y = viewportHeight/2).
    // Our HUD panel is 144px tall, so the viewport centre is (480-144)/2 = 168.
    // Drawing the player at y=240 (screen centre) would leave it half-hidden
    // behind the panel.
    const PANEL_HEIGHT: i32 = 144;
    let screen_center_y: i32 = (LOGICAL_HEIGHT as i32 - PANEL_HEIGHT) / 2; // 168

    // The camera is expressed in world tile coordinates.
    let cam_tile_x = game_state.camera.tile_x;
    let cam_tile_y = game_state.camera.tile_y;

    // Render the world. Each draw function now creates its own short-lived
    // `TextureCreator` per texture it needs and blits immediately — no
    // cross-frame texture cache and no `TextureCreator` borrow held across
    // other `canvas_mut()` calls. This is Plan A from the bug report: every
    // texture is rebuilt each frame, which is slower but provably safe (no
    // dangling textures, no aliasing violations).
    if game_state.in_dungeon {
        // DUNGEON MODE (L1 Cathedral). Requires the L1 level data + a
        // generated dungeon_layout. Falls back gracefully if either is
        // missing (e.g. shareware build without L1 art).
        if let (Some(level), Some(layout)) = (&game_state.dungeon_level_data, &game_state.dungeon_layout) {
            // Collect the living dungeon monsters (id + position + type)
            // for the renderer. Borrowing through a small Vec avoids
            // holding a borrow on monster_manager across the draw call.
            let monsters: Vec<(usize, i32, i32, crate::game::monster::MonsterType, crate::game::monster::MonsterAIState)> =
                game_state.monster_manager.iter().map(|(id, m)| {
                    (id, m.x, m.y, m.monster_type, m.ai_state)
                }).collect();
            let sprites = game_state.monster_sprites.as_ref();
            let _ = draw_dungeon(window, level, layout, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y, &monsters, sprites);
        } else if let Some(level) = &game_state.level_data {
            let _ = draw_checkerboard_fallback(window, level, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
        } else {
            let canvas = window.canvas_mut();
            draw_plain_checkerboard(canvas, screen_center_x, screen_center_y);
        }
    } else if let (Some(level), Some(layout)) = (&game_state.level_data, &game_state.town_layout) {
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

    // Draw the player at the viewport centre (camera == player position).
    // Prefer the real Warrior town-walk sprite; fall back to the yellow
    // marker if no sprite was loaded. The sprite texture is rebuilt every
    // frame (Plan A): safe, no dangling handles.
    draw_player_sprite(window, game_state, screen_center_x, screen_center_y);

    // Draw the bottom HUD panel (life/mana spheres, XP bar, belt, stats) on top
    // of the rendered world + player sprite. Pure canvas drawing (no textures).
    draw_simple_missiles(window, game_state, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
    hud::draw_hud(window, game_state);

    // Debug status line (throttled: only every 30 ticks to avoid log spam).
    if game_state.game_tick % 30 == 0 {
        let mode = if game_state.in_dungeon {
            "Dungeon (L1 Cathedral)"
        } else if game_state.is_town {
            "Town (Tristram)"
        } else {
            "Dungeon"
        };
        let has_art = if game_state.in_dungeon && game_state.dungeon_layout.is_some() {
            "Cathedral layout"
        } else if game_state.town_layout.is_some() {
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

    // Draw a simple in-game cursor so the player can see where they're
    // pointing. (C++ uses a hardware cursor or drawn cursor sprite; we use a
    // small crosshair for now. The OS cursor is hidden globally in main.rs
    // for the menu's drawn cursor, so we must draw one here too.)
    {
        let canvas = window.canvas_mut();
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 0));
        let (mx, my) = mouse_pos;
        for &(dx, dy) in &[(-5, 0), (5, 0), (0, -5), (0, 5), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
            let _ = canvas.draw_point(sdl2::rect::Point::new(mx + dx, my + dy));
        }
    }

    // Screenshot dump hook: RS_SHOT=1 dumps one game frame for verification.
    if std::env::var("RS_SHOT").is_ok() && game_state.game_tick >= 10 {
        static SHOT_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !SHOT_DONE.swap(true, std::sync::atomic::Ordering::SeqCst) {
            let canvas = window.canvas_mut();
            if let Ok(pix) = canvas.read_pixels(None, sdl2::pixels::PixelFormatEnum::ABGR8888) {
                let _ = std::fs::write("game_shot.rgba", &pix);
                eprintln!("[SHOT] game {} bytes tick={}", pix.len(), game_state.game_tick);
            }
        }
    }

    window.present();
}

/// Draw active spell projectiles (Firebolt) as small orange diamonds at their
/// world-tile position projected through the same isometric transform the
/// monsters use. Pure canvas drawing (no textures), so it runs outside the
/// TextureCreator borrow block.
fn draw_simple_missiles(
    window: &mut GameWindow,
    game_state: &GameState,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) {
    if game_state.simple_missiles.is_empty() {
        return;
    }
    let canvas = window.canvas_mut();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 140, 0));
    for m in &game_state.simple_missiles {
        let rel_x = (m.x - cam_tile_x - (m.y - cam_tile_y)) * (TILE_WIDTH / 2);
        let rel_y = (m.x - cam_tile_x + (m.y - cam_tile_y)) * (TILE_HEIGHT / 2);
        let cx = screen_center_x + rel_x;
        let cy = screen_center_y + rel_y;
        // Draw a small diamond (4 triangles of 1px wide) as a stand-in sprite.
        for (dx, dy) in [(-3, 0), (3, 0), (0, -3), (0, 3), (-2, -1), (2, -1), (-2, 1), (2, 1), (0, 0)] {
            let _ = canvas.draw_point(sdl2::rect::Point::new(cx + dx, cy + dy));
        }
    }
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
/// 3. Decode the frame to RGBA and upload it into a fresh SDL `Texture` via a
///    short-lived `TextureCreator` borrowed from the canvas.
/// 4. Blit the two 32x32 textures at the tile's screen position, offset so the
///    pair forms the 64x32 diamond.
///
/// **Safety note (Plan A rewrite):** textures are rebuilt every frame. The old
/// code cached `Texture<'static>` in a thread-local via `transmute`, which was
/// unsound (the `TextureCreator` it borrowed was dropped at the end of the
/// block, leaving dangling SDL handles). Each draw now scopes its
/// `TextureCreator` + `Texture` to a single `copy()` so the lifetimes are
/// provably valid and there is no aliasing violation.
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

            // dPiece is a 1-based index into the MIN mega-tile table (mirrors
            // C++ pMegaTiles[dPiece - 1]). The MIN array is 0-based, so we
            // subtract 1. Without this, every tile reads the wrong mega (off by
            // one) and the town renders dark/garbled.
            let mega_idx = dpiece.saturating_sub(1) as usize;
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

            // C++ scrollrt.cpp (DrawTile): for each dPiece, draw mt[0] (left
            // half) and mt[1] (right half). Each is a 32x32 CEL frame drawn as
            // a 16x32 half-diamond. Together they form the full 32x32 tile.
            // We do NOT select by (wx%2,wy%2) — the dPiece grid already maps
            // each micro-tile to the correct mega entry.
            let canvas = window.canvas_mut();
            for (slot, block) in [(0usize, &mega.blocks[0]), (1, &mega.blocks[1])] {
                if !block.has_value() { continue; }
                let rgba = match TileDecoder::decode_tile(
                    &level.level_cel, block.frame(), block.tile_type(), &level.palette,
                ) {
                    Some(r) => r,
                    None => continue,
                };
                let creator = canvas.texture_creator();
                let tex = match rgba_to_texture(&creator, &rgba, 32, 32) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                // Left half: right edge at dst_cx. Right half: left edge at dst_cx.
                // Both sit with top at dst_cy - 32 (bottom-centre anchor).
                let (tx, ty) = if slot == 0 {
                    (dst_cx - (TILE_WIDTH / 2), dst_cy - TILE_HEIGHT)
                } else {
                    (dst_cx, dst_cy - TILE_HEIGHT)
                };
                let _ = canvas.copy(&tex, None, Rect::new(tx, ty, (TILE_WIDTH / 2) as u32, TILE_HEIGHT as u32));
                drawn += 1;
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

/// Render the visible L1 Cathedral micro-tiles from the dungeon dPiece grid.
///
/// Structurally identical to `draw_tristram`, but reads
/// `game_state.dungeon_layout` (the L1 grid built by `dungeon_level`).
///
/// For each world tile `(wx, wy)` within the camera's view radius we look up
/// `dPiece = layout.get(wx, wy)`, index into `level.min.mega_tiles[dPiece]` to
/// get the two floor sub-tiles, and blit their decoded CEL frames at the tile's
/// screen position. Textures are rebuilt every frame (Plan A — see
/// `draw_tristram` for the safety rationale).
fn draw_dungeon(
    window: &mut GameWindow,
    level: &crate::engine::dungeon::DungeonLevelData,
    layout: &crate::game::game_state::DungeonLayout,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
    monsters: &[(usize, i32, i32, crate::game::monster::MonsterType, crate::game::monster::MonsterAIState)],
    sprites: Option<&crate::game::monster_sprites::MonsterSpriteSet>,
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

            let mega_idx = dpiece.saturating_sub(1) as usize;
            let mega = match level.min.mega_tiles.get(mega_idx) {
                Some(m) => m,
                None => {
                    skipped += 1;
                    continue;
                }
            };

            let rel_x = (wx_off - wy_off) * (TILE_WIDTH / 2);
            let rel_y = (wx_off + wy_off) * (TILE_HEIGHT / 2);
            let dst_cx = screen_center_x + rel_x;
            let dst_cy = screen_center_y + rel_y;

            if dst_cx < -(TILE_WIDTH) || dst_cx > (LOGICAL_WIDTH as i32 + TILE_WIDTH)
                || dst_cy < -(TILE_HEIGHT * 2) || dst_cy > (LOGICAL_HEIGHT as i32 + TILE_HEIGHT)
            {
                continue;
            }

            let canvas = window.canvas_mut();
            for (slot, block) in [(0usize, &mega.blocks[0]), (1, &mega.blocks[1])] {
                if !block.has_value() { continue; }
                let rgba = match TileDecoder::decode_tile(
                    &level.level_cel, block.frame(), block.tile_type(), &level.palette,
                ) {
                    Some(r) => r,
                    None => continue,
                };
                let creator = canvas.texture_creator();
                let tex = match rgba_to_texture(&creator, &rgba, 32, 32) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                let (tx, ty) = if slot == 0 {
                    (dst_cx - (TILE_WIDTH / 2), dst_cy - TILE_HEIGHT)
                } else {
                    (dst_cx, dst_cy - TILE_HEIGHT)
                };
                let _ = canvas.copy(&tex, None, Rect::new(tx, ty, (TILE_WIDTH / 2) as u32, TILE_HEIGHT as u32));
                drawn += 1;
            }
        }
    }

    if drawn == 0 {
        println!(
            "[DrawDungeon] WARNING: drew 0 tiles ({} skipped) cam=({},{}). \
             L1 layout may be empty or dPiece indices out of MIN range.",
            skipped, cam_tile_x, cam_tile_y
        );
    }

    // Step 2: draw the living dungeon monsters on top of the floor. Each
    // monster is placed at its world tile's screen position, with its sprite
    // anchored foot-first (like the player sprite). Monsters with a loaded CL2
    // sprite use it; the rest fall back to a per-type coloured block so every
    // monster still has a visible presence.
    draw_dungeon_monsters(window, monsters, sprites, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);

    Ok(())
}

//------------------------------------------------------------------------------
// Monster Sprite Rendering (Step 2)
//------------------------------------------------------------------------------

/// Draw all living dungeon monsters on top of the rendered floor.
///
/// For each monster we compute its world-tile → screen position using the same
/// isometric transform as the tiles (`rel_x = (wx-wy)*32`, `rel_y = (wx+wy)*16`,
/// minus the camera transform). Monsters off-screen are skipped.
///
/// Rendering path per monster:
/// 1. If `sprites` has a decoded sprite for the monster's type, rebuild its
///    texture from the cached RGBA buffer and blit it anchored foot-first
///    (bottom-centre on the tile centre) — exactly like the player sprite.
///    The texture is created fresh each frame (Plan A); the RGBA decode lives
///    in `MonsterSpriteSet` and is reused, only the SDL upload is repeated.
/// 2. Otherwise draw a per-type coloured diamond block at the tile so the
///    monster is still visible.
///
/// Monsters are drawn in order of increasing `(wx+wy)` so lower (further-down)
/// monsters correctly overlap monsters above them (painter's order).
fn draw_dungeon_monsters(
    window: &mut GameWindow,
    monsters: &[(usize, i32, i32, crate::game::monster::MonsterType, crate::game::monster::MonsterAIState)],
    sprites: Option<&crate::game::monster_sprites::MonsterSpriteSet>,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) {
    if monsters.is_empty() {
        return;
    }

    // Sort by depth (wx+wy ascending) so monsters further down the screen are
    // drawn last and overlap monsters behind them.
    let mut order: Vec<&(usize, i32, i32, crate::game::monster::MonsterType, crate::game::monster::MonsterAIState)> =
        monsters.iter().collect();
    order.sort_by_key(|m| m.1 + m.2);

    let mut drawn = 0u32;
    for (_, wx, wy, mtype, aistate) in order {
        // Skip dead monsters — they're not rendered (no corpse art yet).
        if *aistate == crate::game::monster::MonsterAIState::Dead {
            continue;
        }

        let wx = *wx;
        let wy = *wy;
        let rel_x = (wx - cam_tile_x - (wy - cam_tile_y)) * (TILE_WIDTH / 2);
        let rel_y = (wx - cam_tile_x + (wy - cam_tile_y)) * (TILE_HEIGHT / 2);
        let dst_cx = screen_center_x + rel_x;
        let dst_cy = screen_center_y + rel_y;

        // Cull off-screen monsters.
        if dst_cx < -(TILE_WIDTH * 2) || dst_cx > (LOGICAL_WIDTH as i32 + TILE_WIDTH * 2)
            || dst_cy < -(TILE_HEIGHT * 6) || dst_cy > (LOGICAL_HEIGHT as i32 + TILE_HEIGHT * 4)
        {
            continue;
        }

        // Try the real sprite path first: rebuild the texture from the cached
        // RGBA (Plan A — safe, no dangling handles). Scope the creator+texture
        // to this single blit so the canvas borrow is released right after copy.
        let mut used_sprite = false;
        if let Some(set) = sprites {
            if let Some(sprite) = set.get(mtype) {
                // Anchor feet at the tile centre: bottom-centre of the
                // sprite on (dst_cx, dst_cy). Same convention as the
                // player sprite so monsters stand on their tile.
                let dst = Rect::new(
                    dst_cx - sprite.width as i32 / 2,
                    dst_cy - sprite.height as i32,
                    sprite.width as u32,
                    sprite.height as u32,
                );
                let canvas = window.canvas_mut();
                let creator = canvas.texture_creator();
                // Bind the Result first so the Texture's borrow of `creator`
                // outlives the match temporaries (avoids E0597 drop-order).
                let tex_result =
                    rgba_to_texture(&creator, &sprite.rgba, sprite.width, sprite.height);
                if let Ok(tex) = tex_result {
                    let _ = canvas.copy(&tex, None, dst);
                    used_sprite = true;
                }
            }
        }

        if !used_sprite {
            // Coloured-block fallback: a small filled diamond in the monster
            // type's colour, plus a darker outline so it reads against the floor.
            let (r, g, b) = crate::game::monster_sprites::monster_display_color(mtype);
            let canvas = window.canvas_mut();
            fill_diamond(canvas, dst_cx, dst_cy, sdl2::pixels::Color::RGB(r, g, b));
            canvas.set_draw_color(sdl2::pixels::Color::RGB(
                r / 3,
                g / 3,
                b / 3,
            ));
            // Outline the diamond for contrast.
            let half_w = TILE_WIDTH / 2;
            let half_h = TILE_HEIGHT / 2;
            let _ = canvas.draw_line((dst_cx, dst_cy - half_h), (dst_cx + half_w, dst_cy));
            let _ = canvas.draw_line((dst_cx + half_w, dst_cy), (dst_cx, dst_cy + half_h));
            let _ = canvas.draw_line((dst_cx, dst_cy + half_h), (dst_cx - half_w, dst_cy));
            let _ = canvas.draw_line((dst_cx - half_w, dst_cy), (dst_cx, dst_cy - half_h));
        }
        drawn += 1;
    }
    let _ = drawn;
}

/// Clear the monster-sprite texture cache. Retained as a public no-op for API
/// compatibility with `return_to_town` / `main.rs` — there is no texture cache
/// any more (textures are rebuilt every frame, Plan A), so this is a safe no-op.
pub fn clear_monster_sprite_cache() {}

/// Fallback renderer: draw a checkerboard of real town tile frames when the
/// layout grid hasn't been built yet. Each visible tile decodes the first
/// mega-tile's left block and uploads a fresh texture (Plan A — no cross-frame
/// cache, no unsafe).
fn draw_checkerboard_fallback(
    window: &mut GameWindow,
    level: &crate::engine::dungeon::DungeonLevelData,
    _cam_tile_x: i32,
    _cam_tile_y: i32,
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
                    // Decode + upload per tile (Plan A). The same frame is
                    // re-decoded/re-uploaded for every visible tile; acceptable
                    // for this fallback path.
                    let rgba = match TileDecoder::decode_tile(
                        &level.level_cel,
                        block.frame(),
                        block.tile_type(),
                        &level.palette,
                    ) {
                        Some(r) => r,
                        None => continue,
                    };
                    let canvas = window.canvas_mut();
                    let creator = canvas.texture_creator();
                    let tex = match rgba_to_texture(&creator, &rgba, 32, 32) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let _ = canvas.copy(
                        &tex,
                        None,
                        Rect::new(
                            dst_cx - (TILE_WIDTH / 2),
                            dst_cy - TILE_HEIGHT,
                            (TILE_WIDTH / 2) as u32,
                            TILE_HEIGHT as u32,
                        ),
                    );
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
// Texture Creation (safe, per-frame — Plan A)
//------------------------------------------------------------------------------
//
// Previous design (REMOVED — caused the segfault): textures were stored in
// thread-local caches as `Texture<'static>` via `transmute`. The lifetime erase
// was unsound for two reasons:
//   1. The `TextureCreator` they borrowed was a local in `draw_and_blit` and
//      was dropped at the end of its block, so cached textures pointed at
//      destroyed/freed SDL state on the next frame (use-after-free → segfault).
//   2. Holding the creator's `&mut Canvas` borrow while `draw_*` called
//      `canvas_mut()` again was a Rust aliasing violation (runtime UB).
//
// New design (Plan A): each draw rebuilds its texture from the already-decoded
// RGBA buffer (which is plain safe `Vec<u8>` data living in `GameState`) and
// blits it immediately. The `TextureCreator`/`Texture` are short-lived locals
// scoped to a single `canvas.copy(&tex, ...)`:
//
//     let canvas = window.canvas_mut();   // &mut Canvas
//     let creator = canvas.texture_creator(); // owned TextureCreator (holds an
//                                             //   Rc<RendererContext>, does NOT
//                                             //   keep borrowing the canvas)
//     let tex = rgba_to_texture(&creator, rgba, w, h)?; // tex borrows creator
//     canvas.copy(&tex, None, dst)?;       // &Texture + &mut Canvas: the
//                                             //   creator borrow is independent,
//                                             //   so no aliasing violation.
//     // tex, creator dropped here — handle freed, nothing to dangle.
//
// This is slower than caching (the RGBA→texture upload is repeated every
// frame) but is provably safe and gets the game loop running.

/// Clear the tile texture cache. Retained as a public no-op for API
/// compatibility with `main.rs` — there is no texture cache any more (Plan A),
/// so this is a safe no-op.
pub fn clear_tile_texture_cache() {}

//------------------------------------------------------------------------------
// Player Sprite Rendering (Step 2)
//------------------------------------------------------------------------------

/// Draw the player at the viewport centre.
///
/// If `GameState::player_sprite` is set (a real Warrior town-walk sprite was
/// decoded in `start_game`), rebuild its texture from the cached RGBA buffer and
/// blit it anchored so the sprite's feet sit on the camera tile (top-left at
/// `centre_x - width/2`, `centre_y - height`). Otherwise fall back to the old
/// yellow marker so the game is still playable without assets.
///
/// The texture is rebuilt every frame (Plan A); the RGBA decode is cached in
/// `GameState::player_sprite` and only the SDL upload is repeated.
fn draw_player_sprite(
    window: &mut GameWindow,
    game_state: &GameState,
    centre_x: i32,
    centre_y: i32,
) {
    if let Some(sprite) = &game_state.player_sprite {
        // Rebuild the texture from the cached RGBA each frame (Plan A). The
        // creator holds an Rc into the renderer and does not keep borrowing the
        // canvas, so `creator` + `tex` + `canvas.copy(&tex, ...)` is sound.
        let canvas = window.canvas_mut();
        let creator = canvas.texture_creator();
        // Bind the Result to a local first so the Texture's borrow of `creator`
        // outlives the match temporaries (avoids the E0597 drop-order problem).
        let tex_result = rgba_to_texture(&creator, &sprite.rgba, sprite.width, sprite.height);
        match tex_result {
            Ok(tex) => {
                // Anchor feet at the tile centre: bottom-centre of the sprite on
                // (centre_x, centre_y). This matches how Diablo positions actor
                // sprites on their tile.
                let dst = Rect::new(
                    centre_x - sprite.width as i32 / 2,
                    centre_y - sprite.height as i32,
                    sprite.width as u32,
                    sprite.height as u32,
                );
                let _ = canvas.copy(&tex, None, dst);
                return;
            }
            Err(e) => {
                eprintln!("[PlayerSprite] texture upload failed: {:?}", e);
            }
        }
    }

    // Fallback: yellow marker (kept for asset-less runs / tests).
    let canvas = window.canvas_mut();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 220, 60));
    let _ = canvas.fill_rect(Rect::new(centre_x - 4, centre_y - 4, 8, 8));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 20, 20));
    let _ = canvas.draw_rect(Rect::new(centre_x - 5, centre_y - 5, 10, 10));
}

/// Clear the player-sprite texture cache. Retained as a public no-op for API
/// compatibility with `main.rs` — there is no texture cache any more (Plan A).
pub fn clear_player_sprite_cache() {}

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
        TilData, TileType, TilEntry,
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
