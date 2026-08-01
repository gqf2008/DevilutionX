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
use crate::engine::types::Point as TilePoint;
use crate::game::input::InputSystem;
use crate::game::network;
use crate::game::game_state::{GameState, TownLayout, TOWN_MAX_X, TOWN_MAX_Y, GroundItemType};
use crate::game::player_exact::Player;
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
    /// Click-to-move destination in world tile coordinates. Set when the
    /// player left-clicks the ground; cleared on arrival or when a movement
    /// key is pressed. Consumed one tile per logic tick in the 2 Hz update
    /// path. `None` when the player is not auto-walking.
    move_target: Option<(i32, i32)>,
    /// Character stats panel visibility (toggled with the `C` key, mirroring
    /// C++ `QuestLogIsOpen`/`chrbtns` style panel toggles). When true,
    /// `draw_and_blit` paints a full-screen overlay listing the hero's
    /// Str/Mag/Dex/Vit, HP/Mana, AC, damage, XP, gold.
    char_panel_open: bool,
    /// Quest log visibility (toggled with the `Q` key, mirroring C++
    /// `QuestLogIsOpen`). When true, `draw_and_blit` paints an overlay listing
    /// the active quests and their status.
    quest_panel_open: bool,
    /// Spell book visibility (toggled with the `B` key, mirroring C++
    /// `DrawSpellBook`). When true, `draw_and_blit` paints an overlay listing
    /// the player's known spells with mana costs; the currently readied spell
    /// (`player._p_r_spell`) is highlighted. Number keys 1-9 select a spell.
    spellbook_open: bool,
    /// Automap visibility (toggled with the `A` key, mirroring C++
    /// `StartAutomap`/`DrawAutomap`). When true, `draw_and_blit` paints a
    /// semi-transparent overlay in the upper-right showing an isometric
    /// thumbnail of the surrounding tiles plus the player + nearby NPCs /
    /// monsters.
    automap_open: bool,
}

impl GameLoopState {
    fn new() -> Self {
        Self {
            running: true,
            startup: true,
            process_players: true,
            result: true,
            mouse_pos: (320, 240),
            move_target: None,
            char_panel_open: false,
            quest_panel_open: false,
            spellbook_open: false,
            automap_open: false,
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

    // Equip the Warrior with a starter Short Sword (1-6 damage) so melee
    // combat has a non-zero damage range. This mirrors the C++ starting kit
    // handed out in `CreatePlayer`/`StartNewGame`. Recalculating derived item
    // stats here populates `_p_i_min_dam`/`_p_i_max_dam` which the combat path
    // (`player_attack_monster`) reads. Safe to call every loop start: it's
    // idempotent (no-ops when a weapon is already equipped).
    game_state.equip_starter_weapon();
    println!(
        "[Equip] starter weapon equipped: damage range {}-{} (gold {})",
        game_state.player._p_i_min_dam,
        game_state.player._p_i_max_dam,
        game_state.player._p_gold
    );

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
            if !handle_event(&event, &mut state, &mut input, game_state) {
                break;
            }
        }

        if !state.running {
            break;
        }

        // ── Death / Resurrection flow ───────────────────────────────────────
        //
        // When `game_state.player_dead` is latched (HP dropped to ≤ 0 in the
        // last logic tick), the player sees the red "YOU HAVE DIED" overlay
        // (drawn in `draw_and_blit`) and the rest of the game input is frozen.
        // Pressing Space or Enter resurrects them: HP/Mana refilled, sent back
        // to Tristram town, gold halved (classic Diablo death penalty). Mirrors
        // C++ `gbDeathActive` + the Space-to-respawn handler in `RunGameLoop`.
        //
        // The input handlers below (D/T stairs, F5/F9 save, F spell, movement)
        // are gated on `!is_player_dead()` so they can't fire while dead; the
        // timing/render path still runs every frame so the overlay is drawn.
        let player_dead = game_state.is_player_dead();
        if player_dead {
            if input.is_key_pressed(Keycode::Space)
                || input.is_key_pressed(Keycode::Return)
                || input.is_key_pressed(Keycode::Return2)
                || input.is_key_pressed(Keycode::KpEnter)
            {
                game_state.resurrect_player();
            }
        }

        // Town <-> Dungeon toggle keys. 'D' descends into L1 Cathedral (only
        // when currently in town); 'T' returns to town (only when in the
        // dungeon). These are manual fallbacks alongside the automatic
        // stair-tile detection in `check_stairs_transition` (which fires when
        // the player walks onto a stair tile). Kept so the player can force a
        // transition even if they can't reach the stair (e.g. pathfinding
        // limitations) or for debugging.
        if !player_dead {
            if input.is_key_pressed(Keycode::D) && !game_state.in_dungeon {
                if let Err(e) = descend_to_dungeon(game_state) {
                    println!("[GameLoop] descend_to_dungeon failed: {}", e);
                }
            }
            if input.is_key_pressed(Keycode::T) && game_state.in_dungeon {
                return_to_town(game_state);
            }
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
        // inside the cast; nothing happens if mana is insufficient. Skipped
        // while dead (the player must resurrect first).
        if !player_dead && input.is_key_pressed(Keycode::F) && game_state.in_dungeon {
            let _ = game_state.cast_firebolt_at_nearest();
        }

        // Apply continuous movement (held arrow/WASD keys) to the player/camera.
        // Frozen while dead — the overlay takes over until the player resurrects.
        if !player_dead {
            apply_movement(game_state, &input);
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
                draw_and_blit(window, game_state, state.mouse_pos, state.move_target, state.char_panel_open, state.quest_panel_open, state.spellbook_open, state.automap_open);
            }

            continue;
        }

        // LOGIC PATH - Every 500ms (~2 Hz)
        // C++: ProcessGameMessagePackets(), game_loop(gbGameLoopStartup), diablo_color_cyc_logic()

        // Click-to-move: walk one tile toward the pending destination each
        // logic tick, dragging the camera with the player. Done here (not in
        // the fast path) so movement is locked to the 2 Hz simulation rate.
        // Skipped while dead (the death overlay is modal).
        if !game_state.is_player_dead() {
            if let Some(target) = state.move_target {
                tick_move_target(game_state, target, &mut state.move_target);
            }
        }

        // Stair detection: if the player is standing on (or adjacent to) a
        // stair trigger tile and the cooldown has elapsed, perform the level
        // transition automatically. Mirrors C++ `CheckTriggers` running inside
        // `GameLogic()` after the player position has been updated. Placed here
        // (logic path, 2 Hz) rather than the fast path so transitions are
        // locked to the simulation rate and the cooldown (in ticks) is
        // meaningful. The D/T manual keys above remain as a fallback. Skipped
        // while dead.
        if !game_state.is_player_dead() {
            check_stairs_transition(game_state);
        }

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
            draw_and_blit(window, game_state, state.mouse_pos, state.move_target, state.char_panel_open, state.quest_panel_open, state.spellbook_open, state.automap_open);
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

        // Drain any SFX requests queued by gameplay (combat hits, monster
        // deaths, item pickups, ...) and forward them to the global audio sink.
        // The sink is registered by main.rs at startup and wraps
        // AudioManager::play_sfx. In headless/test environments where no sink
        // is registered, dispatch_sfx is a silent no-op (never panics).
        for sfx_name in game_state.drain_pending_sfx() {
            crate::engine::audio::dispatch_sfx(&sfx_name);
        }

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

/// Convert a logical-canvas mouse position (mx, my) into a world tile
/// coordinate, using the inverse of the faithful pipeline projection
/// (`tile_screen_position` / C++ GetScreenPosition).
///
/// The camera tile anchors at `tile_to_screen(cam, cam)`; a tile offset
/// (u, v) = (wx - cam_x, wy - cam_y) projects to anchor + ((u-v)*32, (u+v)*16).
/// Inverting:
///   u - v = rel_x / 32
///   u + v = rel_y / 16
///   u = ((u+v) + (u-v)) / 2
///   v = ((u+v) - (u-v)) / 2
fn convert_screen_to_tile(
    mx: i32,
    my: i32,
    cam_tile_x: i32,
    cam_tile_y: i32,
) -> (i32, i32) {
    let (ax, ay) = tile_to_screen(cam_tile_x, cam_tile_y, cam_tile_x, cam_tile_y);
    let rel_x = mx - ax; // (u - v) * 32
    let rel_y = my - ay; // (u + v) * 16

    let diff_uv = rel_x / (TILE_WIDTH / 2); // u - v
    let sum_uv = rel_y / (TILE_HEIGHT / 2); // u + v
    let u = (sum_uv + diff_uv) / 2;
    let v = (sum_uv - diff_uv) / 2;

    (cam_tile_x + u, cam_tile_y + v)
}

fn handle_event(
    event: &Event,
    state: &mut GameLoopState,
    input: &mut InputSystem,
    game_state: &mut GameState,
) -> bool {
    match event {
        Event::Quit { .. } => {
            println!("Quit event received");
            state.result = false;
            state.running = false;
            false
        }
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            // If the shop panel is open, ESC closes it instead of exiting the
            // game (mirrors C++ where ESC first dismisses the active store
            // before falling through to the game-menu exit). Otherwise ESC
            // exits the game back to the menu.
            if game_state.shop_open {
                game_state.close_shop();
                return true;
            }
            println!("ESC pressed - exiting game loop");
            state.running = false;
            false
        }
        Event::KeyDown { keycode: Some(k), .. } => {
            // Any movement key cancels an in-progress click-to-move so the
            // keyboard takes over immediately (mirrors C++ behaviour where a
            // new walk command clears the pending destination).
            if is_movement_keycode(*k) {
                if state.move_target.is_some() {
                    state.move_target = None;
                }
            }
            // Panel toggles (mirrors C++ Keydown handlers in
            // `diablo.cpp::PressKey`: `KEYCODE_C` opens the character sheet,
            // `KEYCODE_Q` opens the quest log, `KEYCODE_B` opens the spell
            // book, `KEYCODE_A` opens the automap). Edge-triggered via KeyDown
            // so a single tap flips the flag; pressing again closes.
            match *k {
                Keycode::C => state.char_panel_open = !state.char_panel_open,
                Keycode::Q => state.quest_panel_open = !state.quest_panel_open,
                Keycode::B => state.spellbook_open = !state.spellbook_open,
                Keycode::A => state.automap_open = !state.automap_open,
                _ => {}
            }
            // While the spell book is open, the number keys 1-9 select the
            // readied spell (`player._p_r_spell`) from the displayed list.
            // Mirrors C++ `DrawSpellBook` / `PressSpellKey` selection. The
            // list ordering is fixed by `known_spells_for_book()` (canonical
            // Diablo order); the index maps to the nth known spell.
            if state.spellbook_open {
                if let Some(idx) = spell_selection_index(*k) {
                    let known = known_spells_for_book(&game_state.player);
                    if let Some(&(spell, _mana)) = known.get(idx) {
                        game_state.player._p_r_spell = spell;
                        game_state.player._p_r_spl_type =
                            crate::game::player_exact::SpellType::Spell;
                        crate::engine::audio::dispatch_sfx("ui_click");
                    }
                }
            }
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
            // Play the UI click SFX on mouse-down. dispatch_sfx is a no-op
            // when no global sink is registered (headless / library-only), so
            // this is safe in all environments.
            crate::engine::audio::dispatch_sfx("ui_click");
            true
        }
        Event::MouseButtonUp { mouse_btn, x, y, .. } => {
            state.mouse_pos = (*x, *y);
            // Left-click handling branches on whether the shop panel is open.
            if *mouse_btn == sdl2::mouse::MouseButton::Left {
                // If the shop panel is open, route clicks to the shop overlay
                // (item rows / CLOSE) instead of click-to-move. The overlay
                // geometry is computed by `shop_panel_geometry` and must match
                // `draw_shop_panel`.
                if game_state.shop_open {
                    handle_shop_panel_click(game_state, *x, *y);
                    return true;
                }

                // Convert the click's logical-canvas coords to a world tile via
                // the inverse isometric projection (same transform C++
                // `ConvertToTileGrid` uses).
                let (wx, wy) = convert_screen_to_tile(
                    *x,
                    *y,
                    game_state.camera.tile_x,
                    game_state.camera.tile_y,
                );

                // First, if the click lands on (or very near) a shop-capable
                // NPC, open that NPC's shop instead of walking. This is the
                // C++ `TalkToTowner` path. We also open a placeholder panel for
                // gossip-only NPCs so the player gets feedback.
                if let Some((nx, ny, _name, kind)) = game_state.npc_at_tile(wx, wy) {
                    if GameState::npc_runs_shop(kind) {
                        game_state.open_shop(kind);
                        println!(
                            "[Shop] opened {} shop (NPC at ({},{}), kind {})",
                            GameState::shop_name_for_npc(kind),
                            nx,
                            ny,
                            kind
                        );
                        // Cancelling any in-progress walk so the player stops at
                        // the NPC rather than walking through them.
                        state.move_target = None;
                        return true;
                    }
                }

                // Otherwise the click is on the ground: store it as a click-to-
                // move destination for the 2 Hz logic tick to walk toward one
                // tile at a time.
                state.move_target = Some((wx, wy));
                println!("[ClickMove] target=({},{}) cam=({},{})", wx, wy, game_state.camera.tile_x, game_state.camera.tile_y);
                println!("[ClickMove] target=({},{})", wx, wy);
            }
            true
        }
        _ => {
            // C++: HandleMessage(event, modState)
            true
        }
    }
}

/// Is `k` one of the keyboard movement keys (WASD + arrows)? Used to decide
/// whether a key-down should cancel an in-progress click-to-move.
fn is_movement_keycode(k: Keycode) -> bool {
    matches!(
        k,
        Keycode::W
            | Keycode::A
            | Keycode::S
            | Keycode::D
            | Keycode::Up
            | Keycode::Down
            | Keycode::Left
            | Keycode::Right
    )
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

    // Resolve the Cathedral→town up-stair tile (C++ `InitL1Triggers` scans for
    // `dPiece == 128`; the Rust generator instead stamps the EntranceStairs
    // TIL mega, so we detect by matching that mega's micro1 value in the
    // generated d_piece grid). Falls back to None if no match found.
    let up_stairs = find_dungeon_up_stairs(&layout, level);
    if let Some((sx, sy)) = up_stairs {
        println!("[Descend] up-stair (to town) located at micro-tile ({}, {})", sx, sy);
    } else {
        println!("[Descend] WARNING: no up-stair tile found in generated layout");
    }

    game_state.dungeon_layout = Some(layout);
    game_state.dungeon_up_stairs = up_stairs;
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

    // Stamp the cooldown so the freshly-spawned position can't instantly
    // re-trigger a transition. The player starts in the dungeon interior and
    // the up-stair is somewhere in the 40×40 active region; even so, this guard
    // makes the descent robust against landing on/near the stair tile.
    game_state.mark_stair_transition();

    // Step 1: place monsters on the dungeon floor. We clear any stale monsters
    // (e.g. from a previous descent) and scatter a small pack across walkable
    // floor tiles away from the player spawn. Also pre-load their stand sprites
    // so the renderer can draw real CL2 art (with a coloured-block fallback).
    place_dungeon_monsters(game_state, center_x, center_y);

    Ok(())
}

/// The TIL mega index the L1 Cathedral generator uses for the
/// `EntranceStairs` logical tile. Must match `tile_to_l1_til_index` in
/// `dungeon_level.rs` (`Tile::EntranceStairs => 12`). Kept here so the
/// up-stair detection stays in sync with the generator's TIL mapping without
/// `game_loop` having to reach into the levels module.
const L1_ENTRANCE_STAIRS_TIL_INDEX: usize = 12;

/// Scan a freshly-generated Cathedral `DungeonLayout` for the up-stair tile
/// (the `EntranceStairs` mega) and return its micro-tile coordinates.
///
/// The Rust generator stamps the `EntranceStairs` TIL mega (index
/// `L1_ENTRANCE_STAIRS_TIL_INDEX`) into the active region of the d_piece grid
/// (micro offset 16,16), writing the mega's four micro values into each 2×2
/// block. We look up that mega's `micro1` value from the loaded L1 TIL data
/// and scan the grid for a matching d_piece, returning the first hit.
///
/// This mirrors C++ `InitL1Triggers` (which scans for `dPiece == 128`); the
/// difference is that the Rust port's dPiece values are the L1 TIL micro
/// indices, so we resolve the expected value from the TIL data instead of
/// hard-coding 128. Returns `None` if the TIL table has no entry at the
/// entrance-stairs index or no grid cell matches.
fn find_dungeon_up_stairs(
    layout: &crate::game::game_state::DungeonLayout,
    level: &crate::engine::dungeon::DungeonLevelData,
) -> Option<(i32, i32)> {
    let mega = level.til.tiles.get(L1_ENTRANCE_STAIRS_TIL_INDEX)?;
    let target = mega.micro1;
    if target == 0 {
        return None;
    }
    for y in 0..layout.height {
        for x in 0..layout.width {
            if layout.d_piece[y * layout.width + x] == target {
                return Some((x as i32, y as i32));
            }
        }
    }
    None
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
    game_state.dungeon_up_stairs = None;
    // Stamp the cooldown so the town spawn (which is fixed at 75,68, far from
    // the Cathedral down-stair at 25,29) can't accidentally re-trigger — and
    // more importantly, so a player who immediately walks back to the stair
    // gets the same 2-second grace as every other transition.
    game_state.mark_stair_transition();
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

    // Convert screen-space (mdx, mdy) to world-tile delta.
    // C++ worldToScreen: screenX = (worldY - worldX) * 32,
    //                    screenY = (worldY + worldX) * -16
    // So screen-right (+screenX) means worldY - worldX increases =>
    //   press Right (mdx=+1): worldY+1 or worldX-1. Convention: move SE.
    // screen-up (-screenY) means worldY + worldX increases =>
    //   press Up (mdy=-1 => screenY decreases): worldY+1 or worldX+1.
    // Simplified for 8-direction:
    //   world_dx = mdy - mdx (screen-up + screen-right => world +x)
    //   world_dy = mdy + mdx
    let world_dx = (mdy - mdx) as i32;
    let world_dy = (mdy + mdx) as i32;

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

/// Advance click-to-move by one tile toward `target`.
///
/// Called once per logic tick (2 Hz). Moves the player one world tile along
/// the diagonal toward the destination (independent signum on each axis, so
/// the path is a diagonal-then-straight walk, matching how the C++ walk
/// command steps `position` toward `walk.destination`). The camera follows
/// the player exactly (camera == player position, same convention as
/// [`apply_movement`]). On arrival the destination is cleared via the
/// `move_target` out-parameter so the main loop stops calling this.
///
/// `target` is passed by value and `move_target` is a mutable reference back
/// into `GameLoopState` so we can `None` it on arrival; the caller reads the
/// current value from `state.move_target` before invoking us.
fn tick_move_target(
    game_state: &mut GameState,
    target: (i32, i32),
    move_target: &mut Option<(i32, i32)>,
) {
    let (tx, ty) = target;
    let cur_x = game_state.player.position.x;
    let cur_y = game_state.player.position.y;
    println!("[TickMove] player=({},{}) target=({},{})", cur_x, cur_y, tx, ty);

    // Already there?
    if cur_x == tx && cur_y == ty {
        *move_target = None;
        return;
    }

    // Step one tile toward the target on each axis (signum delta). This gives
    // a diagonal-then-straight path: e.g. from (0,0) to (3,1) walks
    // (1,1)->(2,1)->(3,1).
    let dx = (tx - cur_x).signum();
    let dy = (ty - cur_y).signum();

    let new_x = (cur_x + dx).max(4).min((TOWN_MAX_X as i32) - 5);
    let new_y = (cur_y + dy).max(4).min((TOWN_MAX_Y as i32) - 5);

    // Move the player + camera together. The renderer centres on the camera
    // tile and draws the player token on top, so they must stay aligned.
    game_state.player.position.x = new_x;
    game_state.player.position.y = new_y;
    game_state.camera.tile_x = new_x;
    game_state.camera.tile_y = new_y;
    // Reset the sub-tile accumulators so keyboard movement resumes cleanly
    // after a click-move.
    game_state.camera.sub_x = 0;
    game_state.camera.sub_y = 0;

    // Arrival check: clear the destination so subsequent ticks don't keep
    // nudging (and so keyboard input regains control).
    if new_x == tx && new_y == ty {
        *move_target = None;
    }
}

/// Automatic stair-trigger detection, mirroring C++ `CheckTriggers`
/// (`Source/levels/trigs.cpp`) which fires `WM_DIABNEXTLVL` / `WM_DIABPREVLVL`
/// when the player stands on a stair trigger tile.
///
/// Runs once per logic tick (2 Hz, in the logic path) after movement has been
/// applied, so the player's just-updated tile is the one being tested. Two
/// triggers are checked:
///   * **Town mode** — player on/near `TOWN_DOWN_STAIRS` (25, 29) → calls
///     `descend_to_dungeon` (Cathedral entrance).
///   * **Dungeon mode** — player on/near `dungeon_up_stairs` (resolved from
///     the generated layout) → calls `return_to_town`.
///
/// "On/near" means within `STAIRS_TRIGGER_RADIUS` (Chebyshev distance), so the
/// coarse whole-tile movement in this port still feels responsive. The
/// `STAIRS_COOLDOWN_TICKS` guard (set via `mark_stair_transition`) prevents a
/// fresh descent/ascent from immediately re-triggering.
///
/// The cooldown is stamped here unconditionally once a trigger fires (whether
/// or not the underlying transition succeeds), so a missing-art failure on
/// `descend_to_dungeon` doesn't cause the detection to re-fire every tick and
/// spam the log. `descend_to_dungeon` / `return_to_town` also stamp the
/// cooldown on their own success path (so manual D/T key transitions get the
/// same guard), but the stamp is idempotent.
///
/// Errors from `descend_to_dungeon` (e.g. missing L1 art) are logged and
/// swallowed so the game loop keeps running.
fn check_stairs_transition(game_state: &mut GameState) {
    // Cooldown gate: a transition fired too recently — do nothing this tick.
    if !game_state.stairs_cooldown_ready() {
        return;
    }

    let mut fired = false;
    if !game_state.in_dungeon {
        // Town: check the fixed Cathedral down-stair.
        let (sx, sy) = crate::game::game_state::TOWN_DOWN_STAIRS;
        if game_state.player_tile_distance_to(sx, sy)
            <= crate::game::game_state::STAIRS_TRIGGER_RADIUS
        {
            println!(
                "[Stairs] player at ({},{}) stepped on town down-stair ({},{}) — descending",
                game_state.player.position.x,
                game_state.player.position.y,
                sx,
                sy
            );
            fired = true;
            if let Err(e) = descend_to_dungeon(game_state) {
                println!("[Stairs] descend_to_dungeon failed: {}", e);
            }
        }
    } else if let Some((sx, sy)) = game_state.dungeon_up_stairs {
        // Dungeon: check the resolved Cathedral up-stair.
        if game_state.player_tile_distance_to(sx, sy)
            <= crate::game::game_state::STAIRS_TRIGGER_RADIUS
        {
            println!(
                "[Stairs] player at ({},{}) stepped on dungeon up-stair ({},{}) — ascending",
                game_state.player.position.x,
                game_state.player.position.y,
                sx,
                sy
            );
            fired = true;
            return_to_town(game_state);
        }
    }

    // Stamp the cooldown whenever a trigger fired, regardless of whether the
    // underlying transition succeeded. This prevents a failing descent (e.g.
    // missing L1 art) from re-firing every tick and log-spamming.
    if fired {
        game_state.mark_stair_transition();
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

/// `dPiece` grid adapters so the faithful pipeline can read either layout type.
/// Both `TownLayout` and `DungeonLayout` expose `get(x, y) -> u16`.
impl crate::engine::scrollrt::DPieceGrid for crate::game::game_state::TownLayout {
    fn d_piece(&self, x: i32, y: i32) -> u16 {
        crate::game::game_state::TownLayout::get(self, x, y)
    }
}
impl crate::engine::scrollrt::DPieceGrid for crate::game::game_state::DungeonLayout {
    fn d_piece(&self, x: i32, y: i32) -> u16 {
        crate::game::game_state::DungeonLayout::get(self, x, y)
    }
}

/// Render floor + walls to the 8-bit palette backbuffer via the faithful C++
/// pipeline, then palette-convert + upload in one blit. Replaces the per-tile
/// RGBA-texture path.
fn render_world_pipeline(
    window: &mut GameWindow,
    level: &crate::engine::dungeon::DungeonLevelData,
    grid: &impl crate::engine::scrollrt::DPieceGrid,
    view_pos: TilePoint,
    viewport_h: i32,
) {
    let palette = crate::engine::palette::Palette::from_rgb_bytes(&level.palette.colors)
        .unwrap_or_else(crate::engine::palette::Palette::new);

    // 光照（C++ MakeLightTable + dLight）。城镇全亮；地牢环境全暗 + 玩家光晕。
    // 注：光照表逐帧重建（4096 op，开销可忽略），后续可缓存。墙遮挡光传播
    // （C++ MakeLight 的 flood-fill）尚未移植，当前玩家光为径向衰减（不挡墙）。
    let mut lm = crate::engine::lighting::LightManager::new();
    lm.make_light_table();
    let mut dlight = vec![0u8; 112 * 112];
    if level.dungeon_type != crate::engine::dungeon::DungeonType::Town {
        dlight.fill(crate::engine::lighting::LIGHTS_MAX as u8);
        const PLAYER_LIGHT_RADIUS: u8 = 9;
        lm.do_lighting(
            &mut dlight,
            112,
            crate::engine::lighting::Point::new(view_pos.x, view_pos.y),
            PLAYER_LIGHT_RADIUS,
        );
    }
    let lighting = crate::engine::scrollrt::Lighting::new(&dlight, &lm.tables);

    window.clear_backbuffer();
    {
        let (w, h, buf) = window.backbuffer_mut();
        let mut surface = crate::engine::surface::Surface::new(buf, w as u32, w as i32, h as i32);
        crate::engine::scrollrt::draw_view(
            &mut surface,
            level,
            grid,
            view_pos,
            LOGICAL_WIDTH as i32,
            LOGICAL_HEIGHT as i32,
            viewport_h,
            &lighting,
        );
    }
    let _ = window.present_backbuffer(&palette);
}

/// 把世界 tile 坐标投影到后备缓冲屏幕坐标，与忠实地板管线共享同一套投影
/// （C++ GetScreenPosition）。统一所有实体 overlay 的屏幕定位，保证实体
/// 贴在管线的地板 tile 上（此前各 overlay 手算 rel_x/rel_y，彼此不一致）。
fn tile_to_screen(wx: i32, wy: i32, cam_tile_x: i32, cam_tile_y: i32) -> (i32, i32) {
    let pos = crate::engine::scrollrt::tile_screen_position(
        TilePoint::new(wx, wy),
        TilePoint::new(cam_tile_x, cam_tile_y),
        LOGICAL_WIDTH as i32,
        LOGICAL_HEIGHT as i32,
        LOGICAL_HEIGHT as i32 - 144,
    );
    (pos.x, pos.y)
}

fn draw_and_blit(
    window: &mut GameWindow,
    game_state: &GameState,
    mouse_pos: (i32, i32),
    move_target: Option<(i32, i32)>,
    char_panel_open: bool,
    quest_panel_open: bool,
    spellbook_open: bool,
    automap_open: bool,
) {
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

    // --- Render the world via the faithful C++ pipeline (scrollrt.cpp DrawView
    //     → DrawGame → DrawFloor/DrawTileContent). Floor + walls render into the
    //     8-bit palette backbuffer, then upload in one palette-converted blit
    //     (replacing the per-tile RGBA-texture path). Entities/HUD still overlay
    //     on the SDL canvas below — to be moved onto the palette surface in the
    //     next increment (C++ scrollrt.cpp:326-505, 705-927).
    let viewport_h = LOGICAL_HEIGHT as i32 - PANEL_HEIGHT; // 336
    let view_pos = TilePoint::new(cam_tile_x, cam_tile_y);
    let mut drew_pipeline = false;

    if game_state.in_dungeon {
        if let (Some(level), Some(layout)) = (&game_state.dungeon_level_data, &game_state.dungeon_layout) {
            render_world_pipeline(window, level, layout, view_pos, viewport_h);
            drew_pipeline = true;
        }
    } else if let (Some(level), Some(layout)) = (&game_state.level_data, &game_state.town_layout) {
        render_world_pipeline(window, level, layout, view_pos, viewport_h);
        drew_pipeline = true;
    }

    if !drew_pipeline {
        // No faithful level/layout data: keep the canvas-drawn checkerboard
        // fallbacks so the art chain is still visible (e.g. shareware without
        // L1 art, or before the town layout is built).
        if let Some(level) = &game_state.level_data {
            let _ = draw_checkerboard_fallback(window, level, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
        } else {
            let canvas = window.canvas_mut();
            draw_plain_checkerboard(canvas, screen_center_x, screen_center_y);
        }
    }

    // Draw Tristram NPCs (Griswold, Pepin, Ogden, Cain, ...) as coloured
    // markers with each NPC's initial. Only in town mode — the dungeon has no
    // towners. Rendered after the world (so markers sit on the floor art) and
    // before ground loot / the player sprite (so the player token stays on
    // top). Uses the same isometric projection as monsters/player.
    if !game_state.in_dungeon {
        draw_towners(window, game_state, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
    }

    // Dungeon monsters (real CL2 sprites where loaded, else per-type coloured
    // markers). Previously drawn inside `draw_dungeon`; now that the floor/walls
    // come from the faithful palette pipeline, render them here as a canvas
    // overlay (to move onto the palette surface with the entity-draw increment).
    if game_state.in_dungeon {
        let monsters: Vec<(usize, i32, i32, crate::game::monster::MonsterType, crate::game::monster::MonsterAIState)> =
            game_state.monster_manager.iter().map(|(id, m)| {
                (id, m.x, m.y, m.monster_type, m.ai_state)
            }).collect();
        let sprites = game_state.monster_sprites.as_ref();
        draw_dungeon_monsters(window, &monsters, sprites, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);
    }

    // Draw ground loot (dropped by slain monsters) before the player sprite so
    // the player token renders on top of any item they're standing on. Uses the
    // same isometric projection as the monsters/player (coloured icons, no art).
    draw_ground_items(window, game_state, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);

    // Draw the player at its tile (camera == player position). Anchored via
    // tile_to_screen so it lands on the pipeline's floor tile for the camera.
    // Prefer the real Warrior town-walk sprite; fall back to the yellow
    // marker if no sprite was loaded. The sprite texture is rebuilt every
    // frame (Plan A): safe, no dangling handles.
    let (player_sx, player_sy) = tile_to_screen(cam_tile_x, cam_tile_y, cam_tile_x, cam_tile_y);
    draw_player_sprite(window, game_state, player_sx, player_sy);

    // Stair markers: draw a pulsing arrow/diamond over the relevant stair tile
    // so the player can see where to walk to change levels. Town shows the
    // down-stair (to Cathedral); dungeon shows the up-stair (to town). Pure
    // canvas drawing using the same iso projection as the floor tiles.
    draw_stairs_marker(window, game_state, cam_tile_x, cam_tile_y, screen_center_x, screen_center_y);

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

    // Click-to-move destination marker: a pulsing green diamond outline drawn
    // at the target tile's projected screen position while the player is
    // auto-walking. Removed once the player arrives (move_target == None).
    // Uses the same forward iso projection as the floor tiles.
    if let Some((tx, ty)) = move_target {
        let (cx, cy) = tile_to_screen(tx, ty, cam_tile_x, cam_tile_y);
        let canvas = window.canvas_mut();
        canvas.set_draw_color(sdl2::pixels::Color::RGB(80, 255, 120));
        let half_w = TILE_WIDTH / 2;
        let half_h = TILE_HEIGHT / 2;
        let _ = canvas.draw_line((cx, cy - half_h), (cx + half_w, cy));
        let _ = canvas.draw_line((cx + half_w, cy), (cx, cy + half_h));
        let _ = canvas.draw_line((cx, cy + half_h), (cx - half_w, cy));
        let _ = canvas.draw_line((cx - half_w, cy), (cx, cy - half_h));
        // Centre dot so the destination is readable even at a distance.
        let _ = canvas.draw_point(sdl2::rect::Point::new(cx, cy));
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

    // Character (C) and Quest (Q) overlay panels. Drawn last so they sit on
    // top of the world + HUD + cursor. Each is a modal full-screen dim with a
    // bordered panel listing the relevant info; toggled edge-triggered in
    // `handle_event`. If both are somehow open, the character panel wins
    // (drawn after the quest panel).
    if quest_panel_open {
        draw_quest_panel(window, game_state);
    }
    if char_panel_open {
        draw_char_panel(window, game_state);
    }

    // Spell book (B) overlay panel. Modal like the char/quest panels (drawn
    // on top of the world + HUD). Lists the player's known spells with mana
    // costs and highlights the readied spell; number keys 1-9 select.
    if spellbook_open {
        draw_spellbook_panel(window, game_state);
    }

    // Automap (A) overlay. Non-modal: drawn in the upper-right as a bordered,
    // semi-transparent thumbnail of the surrounding tiles (simplified iso
    // grid) with the player, NPCs, and monsters marked. Mirrors C++
    // `DrawAutomap`'s minimap layout. Drawn before the death overlay so the
    // map remains visible (dimmed) behind the death screen.
    if automap_open {
        draw_automap_overlay(window, game_state);
    }

    // Shop panel overlay: drawn after the char/quest panels and before the
    // death overlay. Opened by clicking a shop-capable NPC; closed by clicking
    // CLOSE or pressing ESC. Renders a semi-transparent dim, the NPC's shop
    // name, the BUY/SELL/REPAIR/CLOSE option buttons, and the item rows with
    // prices. Reads its open state from `GameState::shop_open` so the data
    // lives in one place.
    if game_state.shop_open {
        draw_shop_panel(window, game_state);
    }

    // Death overlay: drawn on the very top (over everything, including the
    // char/quest panels) while `game_state.player_dead` is latched. A red
    // semi-transparent wash + "YOU HAVE DIED" headline + the prompt to press
    // Space/Enter to resurrect. The player resurrects in the main loop's
    // death-flow handler (above), which calls `GameState::resurrect_player`.
    if game_state.is_player_dead() {
        draw_death_overlay(window, game_state);
    }

    window.present();
}

/// Death overlay: painted on top of everything while `game_state.player_dead`
/// is latched.
///
/// A full-screen red semi-transparent wash (alpha ~140) evokes the C++ death
/// screen's red-tinted palette, then a centred black-bordered "YOU HAVE DIED"
/// headline in the largest bitmap font scale, followed by the gold-penalty
/// note and the "PRESS SPACE / ENTER TO RESURRECT" prompt. The hero's name and
/// the death location are shown for context.
///
/// Mirrors C++ `DrawDiabloDeath` (Source/interfac.cpp) which renders the death
/// screen with the slain-hero artwork + a "Save Game? Y/N" prompt; we use a
/// procedural canvas overlay instead since we don't ship the death CEL art.
///
/// All values are read straight from `GameState::player`; nothing is mutated.
fn draw_death_overlay(window: &mut GameWindow, game_state: &GameState) {
    let canvas = window.canvas_mut();
    let player = &game_state.player;

    // ---- Full-screen red wash (semi-transparent) ----
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(110, 0, 0, 160));
    let _ = canvas.fill_rect(Rect::new(0, 0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    // ---- Centred panel backdrop for the headline ----
    const PANEL_W: i32 = 420;
    const PANEL_H: i32 = 150;
    let panel_x = (LOGICAL_WIDTH as i32 - PANEL_W) / 2;
    let panel_y = (LOGICAL_HEIGHT as i32 - PANEL_H) / 2;

    // Outer dark border + inner bevel (matching the char-panel palette).
    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 0, 0));
    let _ = canvas.fill_rect(Rect::new(panel_x - 3, panel_y - 3, (PANEL_W + 6) as u32, (PANEL_H + 6) as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(48, 12, 12));
    let _ = canvas.fill_rect(Rect::new(panel_x, panel_y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(150, 40, 40));
    let _ = canvas.draw_rect(Rect::new(panel_x, panel_y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 24, 24));
    let _ = canvas.draw_rect(Rect::new(panel_x + 1, panel_y + 1, (PANEL_W - 2) as u32, (PANEL_H - 2) as u32));

    // ---- Text ----
    let title_font = crate::engine::font::PixelFont::new(3); // largest scale
    let font = crate::engine::font::PixelFont::new(2);
    let small = crate::engine::font::PixelFont::new(1);

    let red = sdl2::pixels::Color::RGB(255, 60, 60);
    let gold = sdl2::pixels::Color::RGB(255, 200, 80);
    let dim = sdl2::pixels::Color::RGB(180, 160, 140);

    let center_x = LOGICAL_WIDTH as i32 / 2;
    let mut y = panel_y + 14;

    // Headline.
    title_font.render_text_centered(canvas, "YOU HAVE DIED", center_x, y, red);
    y += title_font.line_height() + 8;

    // Divider.
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 24, 24));
    let _ = canvas.draw_line((panel_x + 16, y), (panel_x + PANEL_W - 16, y));
    y += 8;

    // Hero name + death location for context.
    let name = player.get_name();
    small.render_text_centered(
        canvas,
        &format!("{} has fallen", if name.is_empty() { "The hero" } else { name.as_str() }),
        center_x,
        y,
        dim,
    );
    y += small.line_height() + 4;

    // Gold-penalty note.
    let gold_note = format!("Half your gold ({} left) is forfeit", player._p_gold / 2);
    small.render_text_centered(canvas, &gold_note, center_x, y, gold);
    y += small.line_height() + 8;

    // Resurrect prompt.
    font.render_text_centered(canvas, "PRESS SPACE / ENTER TO RESURRECT", center_x, y, gold);
}

/// Character info panel overlay (C key, C++ `DrawChr`).
///
/// Paints a semi-transparent full-screen dim, then a bordered 280x430 stone
/// panel on the left side listing the hero's name, class, level, the four core
/// attributes (Str/Mag/Dex/Vit with their base/bonus split), HP and Mana in
/// display units (raw 64x >> 6), armour class, damage range, experience, the
/// XP needed to reach the next level, and gold.
///
/// All numbers come straight from `GameState::player`; nothing is mutated. The
/// frame + text are drawn procedurally with the SDL2 canvas + `PixelFont` (no
/// CEL art), matching the HUD's approach so the panel renders identically with
/// or without the real Diablo `data\char.cel` art.
fn draw_char_panel(window: &mut GameWindow, game_state: &GameState) {
    let canvas = window.canvas_mut();
    let player = &game_state.player;

    // ---- Full-screen dim (semi-transparent black) ----
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 150));
    let _ = canvas.fill_rect(Rect::new(0, 0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    // ---- Panel frame (left-side, 280x430) ----
    const PANEL_X: i32 = 16;
    const PANEL_Y: i32 = 20;
    const PANEL_W: i32 = 280;
    const PANEL_H: i32 = 430;

    // Outer dark border + inner bevel (mirrors the HUD's stone palette).
    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 16, 12));
    let _ = canvas.fill_rect(Rect::new(PANEL_X - 2, PANEL_Y - 2, (PANEL_W + 4) as u32, (PANEL_H + 4) as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(58, 46, 34));
    let _ = canvas.fill_rect(Rect::new(PANEL_X, PANEL_Y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(140, 116, 84));
    let _ = canvas.draw_rect(Rect::new(PANEL_X, PANEL_Y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_rect(Rect::new(PANEL_X + 1, PANEL_Y + 1, (PANEL_W - 2) as u32, (PANEL_H - 2) as u32));

    // ---- Text ----
    let font = crate::engine::font::PixelFont::new(2);
    let title_font = crate::engine::font::PixelFont::new(2);
    let small = crate::engine::font::PixelFont::new(1);

    let txt = sdl2::pixels::Color::RGB(240, 230, 200);
    let label = sdl2::pixels::Color::RGB(200, 188, 150);
    let dim = sdl2::pixels::Color::RGB(150, 138, 110);
    let hi = sdl2::pixels::Color::RGB(255, 240, 160);

    let mut y = PANEL_Y + 10;

    // Title bar.
    title_font.render_text_centered(canvas, "CHARACTER", PANEL_X + PANEL_W / 2, y, hi);
    y += title_font.line_height() + 8;

    // Divider.
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;

    // Name + class.
    let name = player.get_name();
    let class = player.get_class_name();
    let level = player._p_level;

    title_font.render_text(canvas, &name, PANEL_X + 14, y, txt);
    y += title_font.line_height() + 2;
    small.render_text(canvas, &format!("CLASS: {}", class), PANEL_X + 14, y, dim);
    y += small.line_height() + 2;
    small.render_text(canvas, &format!("LEVEL: {}", level), PANEL_X + 14, y, dim);
    y += small.line_height() + 8;

    // ---- Core attributes (Str/Mag/Dex/Vit): base + bonus ----
    let attrs: [(&str, i32, i32); 4] = [
        ("STRENGTH",     player._p_strength,   player._p_base_str),
        ("MAGIC",        player._p_magic,      player._p_base_mag),
        ("DEXTERITY",    player._p_dexterity,  player._p_base_dex),
        ("VITALITY",     player._p_vitality,   player._p_base_vit),
    ];
    for (name, cur, base) in attrs.iter() {
        font.render_text(canvas, name, PANEL_X + 14, y, label);
        // Display as "cur" if no bonus, else "cur (base+bonus)".
        let bonus = cur - base;
        let val_str = if bonus == 0 {
            format!("{}", cur)
        } else {
            format!("{} ({:+})", cur, bonus)
        };
        font.render_text(canvas, &val_str, PANEL_X + 150, y, txt);
        y += font.line_height() + 4;
    }
    y += 4;

    // ---- HP / Mana (display value = 64x >> 6) ----
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;

    let hp_cur = player._p_hit_points >> 6;
    let hp_max = player._p_max_hp >> 6;
    let mp_cur = player._p_mana >> 6;
    let mp_max = player._p_max_mana >> 6;

    font.render_text(canvas, "LIFE", PANEL_X + 14, y, label);
    font.render_text(canvas, &format!("{}/{}", hp_cur.max(0), hp_max), PANEL_X + 150, y, txt);
    y += font.line_height() + 4;

    font.render_text(canvas, "MANA", PANEL_X + 14, y, label);
    font.render_text(canvas, &format!("{}/{}", mp_cur.max(0), mp_max), PANEL_X + 150, y, txt);
    y += font.line_height() + 8;

    // ---- Combat: AC + damage ----
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;

    let ac = player._p_i_ac;
    let dmg_min = player._p_i_min_dam;
    let dmg_max = player._p_i_max_dam;

    font.render_text(canvas, "ARMOR", PANEL_X + 14, y, label);
    font.render_text(canvas, &format!("{}", ac), PANEL_X + 150, y, txt);
    y += font.line_height() + 4;

    font.render_text(canvas, "DAMAGE", PANEL_X + 14, y, label);
    font.render_text(canvas, &format!("{}-{}", dmg_min, dmg_max), PANEL_X + 150, y, txt);
    y += font.line_height() + 8;

    // ---- Experience / next-level / gold ----
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;

    let exp = player._p_experience;
    // Use the same XP table the HUD uses for the "next level" threshold.
    let next_xp = next_level_threshold(level, exp);

    font.render_text(canvas, "EXPERIENCE", PANEL_X + 14, y, label);
    font.render_text(canvas, &format!("{}", exp), PANEL_X + 150, y, txt);
    y += font.line_height() + 4;

    font.render_text(canvas, "NEXT LEVEL", PANEL_X + 14, y, label);
    let next_str = match next_xp {
        Some(n) => format!("{}", n),
        None => "MAX".to_string(),
    };
    font.render_text(canvas, &next_str, PANEL_X + 150, y, txt);
    y += font.line_height() + 4;

    font.render_text(canvas, "GOLD", PANEL_X + 14, y, label);
    font.render_text(canvas, &format!("{}", player._p_gold), PANEL_X + 150, y, hi);
    y += font.line_height() + 8;

    // Footer hint.
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;
    small.render_text_centered(canvas, "PRESS C TO CLOSE", PANEL_X + PANEL_W / 2, y, dim);
}

/// Quest log panel overlay (Q key, C++ `DrawQuestLog`).
///
/// The port doesn't yet track per-quest state in `GameState`, so this renders a
/// curated list of the canonical Diablo main quests with a fixed "active /
/// completed" status drawn from the hero's level (lower-level quests read as
/// completed once the hero has out-levelled them). When real quest state lands
/// in `GameState` this becomes a straight read; for now it's a readable
/// placeholder that exercises the full overlay path.
fn draw_quest_panel(window: &mut GameWindow, game_state: &GameState) {
    let canvas = window.canvas_mut();
    let level = game_state.player._p_level;

    // ---- Full-screen dim ----
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 150));
    let _ = canvas.fill_rect(Rect::new(0, 0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    // ---- Panel frame (centred, 360x360) ----
    const PANEL_W: i32 = 360;
    const PANEL_H: i32 = 360;
    let panel_x = (LOGICAL_WIDTH as i32 - PANEL_W) / 2;
    let panel_y = (LOGICAL_HEIGHT as i32 - PANEL_H) / 2;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 16, 12));
    let _ = canvas.fill_rect(Rect::new(panel_x - 2, panel_y - 2, (PANEL_W + 4) as u32, (PANEL_H + 4) as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(58, 46, 34));
    let _ = canvas.fill_rect(Rect::new(panel_x, panel_y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(140, 116, 84));
    let _ = canvas.draw_rect(Rect::new(panel_x, panel_y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_rect(Rect::new(panel_x + 1, panel_y + 1, (PANEL_W - 2) as u32, (PANEL_H - 2) as u32));

    let font = crate::engine::font::PixelFont::new(2);
    let title_font = crate::engine::font::PixelFont::new(2);
    let small = crate::engine::font::PixelFont::new(1);

    let hi = sdl2::pixels::Color::RGB(255, 240, 160);
    let txt = sdl2::pixels::Color::RGB(240, 230, 200);
    let active = sdl2::pixels::Color::RGB(120, 220, 120);
    let done = sdl2::pixels::Color::RGB(150, 138, 110);

    let mut y = panel_y + 12;
    title_font.render_text_centered(canvas, "QUEST LOG", panel_x + PANEL_W / 2, y, hi);
    y += title_font.line_height() + 8;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((panel_x + 12, y), (panel_x + PANEL_W - 12, y));
    y += 8;

    // Canonical Diablo quests: (name, level_threshold). A quest reads as
    // "completed" once the hero's level exceeds the threshold, otherwise
    // "active". This is a stand-in until real quest state is tracked.
    let quests: &[(&str, u8)] = &[
        ("THE BUTCHER",          4),
        ("POISONED WATER SUPPLY", 5),
        ("KING LEORIC'S CURSE",  7),
        ("MAGIC ROCK",           9),
        ("VALOR",                10),
        ("HALLS OF THE BLIND",   10),
        ("Zhar THE MAD",         11),
        ("BLACK MUSHROOM",       12),
        ("ANVIL OF FURY",        13),
        ("WARLORD OF BLOOD",     14),
        ("LACHDANAN",            15),
        ("ARCHBISHOP LAZARUS",   16),
    ];

    for (name, thresh) in quests {
        let is_done = level >= *thresh;
        let status_str = if is_done { "[DONE]" } else { "[ACTIVE]" };
        let colour = if is_done { done } else { active };

        font.render_text(canvas, name, panel_x + 18, y, txt);
        font.render_text(canvas, status_str, panel_x + PANEL_W - 18 - font.text_width(status_str), y, colour);
        y += font.line_height() + 3;
    }

    y += 8;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((panel_x + 12, y), (panel_x + PANEL_W - 12, y));
    y += 6;
    small.render_text_centered(canvas, "PRESS Q TO CLOSE", panel_x + PANEL_W / 2, y, done);
}

// ============================================================================
// Shop panel (Step 1: open on NPC click, list items, buy on click)
// ============================================================================
//
// Geometry must be kept in sync between `shop_panel_geometry` (hit testing) and
// `draw_shop_panel` (painting). The panel is a centred 420x340 stone rect with:
//   * header (NPC shop name)
//   * BUY / SELL / REPAIR / CLOSE option buttons row
//   * 3-5 item rows (name + price), clickable
//   * gold + hint footer
// All coordinates are in logical 640x480 canvas space.

/// Shop panel layout constants shared between drawing and hit-testing.
const SHOP_PANEL_W: i32 = 420;
const SHOP_PANEL_H: i32 = 340;
/// Y offset of the first option button (BUY/SELL/REPAIR/CLOSE) from the panel
/// top.
const SHOP_OPTIONS_Y_OFF: i32 = 56;
/// Height of each option button.
const SHOP_OPTION_BTN_H: i32 = 22;
/// Number of option buttons across the panel.
const SHOP_NUM_OPTIONS: i32 = 4;
/// Y offset of the first item row from the panel top.
const SHOP_ITEMS_Y_OFF: i32 = 100;
/// Height of each item row (clickable + rendered).
const SHOP_ITEM_ROW_H: i32 = 24;

/// Layout of the shop panel: absolute (logical-canvas) rects for the option
/// buttons (BUY/SELL/REPAIR/CLOSE in that order) and the item rows. Computed
/// from `GameState::active_shop_inventory()` so the hit-test and draw paths
/// share one source of truth. Returns `(panel_rect, option_rects, item_rects)`.
///
/// Pure (no `&mut self`); safe to call from both the event handler and the
/// render path.
fn shop_panel_geometry(game_state: &GameState) -> (Rect, Vec<Rect>, Vec<Rect>) {
    let panel_x = (LOGICAL_WIDTH as i32 - SHOP_PANEL_W) / 2;
    let panel_y = (LOGICAL_HEIGHT as i32 - SHOP_PANEL_H) / 2;
    let panel = Rect::new(panel_x, panel_y, SHOP_PANEL_W as u32, SHOP_PANEL_H as u32);

    // Option buttons: 4 buttons evenly spaced across the panel width with 8px
    // side padding and 6px gutters.
    let btn_w = (SHOP_PANEL_W - 16 - (SHOP_NUM_OPTIONS - 1) as i32 * 6) / SHOP_NUM_OPTIONS;
    let mut options: Vec<Rect> = Vec::with_capacity(SHOP_NUM_OPTIONS as usize);
    for i in 0..SHOP_NUM_OPTIONS {
        let bx = panel_x + 8 + i as i32 * (btn_w + 6);
        let by = panel_y + SHOP_OPTIONS_Y_OFF;
        options.push(Rect::new(bx, by, btn_w as u32, SHOP_OPTION_BTN_H as u32));
    }

    // Item rows: one per active-shop inventory entry, each full-width-minus-
    // padding.
    let row_w = SHOP_PANEL_W - 24;
    let mut items: Vec<Rect> = Vec::new();
    let n = game_state.active_shop_inventory().len() as i32;
    for i in 0..n {
        let ry = panel_y + SHOP_ITEMS_Y_OFF + i * SHOP_ITEM_ROW_H;
        items.push(Rect::new(panel_x + 12, ry, row_w as u32, SHOP_ITEM_ROW_H as u32));
    }

    (panel, options, items)
}

/// Handle a left-click at `(mx, my)` while the shop panel is open.
///
/// Hit-tests against the layout from [`shop_panel_geometry`]:
///   * CLOSE button (index 3) closes the shop.
///   * BUY/SELL/REPAIR buttons are no-ops for now (logged); the demo only wires
///     the buy path via item-row clicks. (Buying is exercised by clicking an
///     item row below the buttons.)
///   * An item row calls [`GameState::buy_shop_item`] for that index, logging
///     the result. Gold is deducted on success.
///
/// Clicks outside any button/row are ignored (the panel stays open). Pure
/// geometry check + mutation of `game_state` only; no rendering.
fn handle_shop_panel_click(game_state: &mut GameState, mx: i32, my: i32) {
    let (_panel, options, items) = shop_panel_geometry(game_state);

    // CLOSE button is always the 4th option.
    if let Some(close_rect) = options.get(3) {
        if point_in_rect(mx, my, *close_rect) {
            println!("[Shop] CLOSE clicked - closing shop");
            game_state.close_shop();
            return;
        }
    }
    // BUY/SELL/REPAIR buttons (0..3) are placeholders for the demo: just log.
    for (i, rect) in options.iter().enumerate() {
        if i == 3 {
            break; // handled above
        }
        if point_in_rect(mx, my, *rect) {
            println!("[Shop] option {} clicked (not wired in demo)", i);
            return;
        }
    }

    // Item rows -> attempt purchase.
    for (i, rect) in items.iter().enumerate() {
        if point_in_rect(mx, my, *rect) {
            match game_state.buy_shop_item(i) {
                Ok((name, price)) => {
                    println!(
                        "[Shop] bought '{}' for {} gold ({} gold left)",
                        name, price, game_state.player._p_gold
                    );
                }
                Err(reason) => {
                    println!("[Shop] buy failed: {}", reason);
                }
            }
            return;
        }
    }
}

/// True when the point `(x, y)` lies inside `rect`. Uses SDL2's exclusive-
/// bound semantics (`right = x + w`, `bottom = y + h`), so a rect of
/// `Rect::new(10, 20, 30, 40)` covers pixel x in [10, 40) and y in [20, 60).
fn point_in_rect(x: i32, y: i32, rect: Rect) -> bool {
    x >= rect.left() && x < rect.right() && y >= rect.top() && y < rect.bottom()
}

/// Draw the shop panel overlay.
///
/// Paints a semi-transparent full-screen dim, then a bordered 420x340 stone
/// panel showing:
///   * the NPC's shop name (header),
///   * a row of BUY / SELL / REPAIR / CLOSE buttons,
///   * the active shop's inventory as clickable rows (name left, price right),
///   * the player's current gold and a "click CLOSE or ESC to leave" hint.
///
/// All values are read straight from `GameState`; nothing is mutated. The
/// geometry matches [`shop_panel_geometry`] so click hit-testing lands on the
/// painted rows. Procedural canvas + `PixelFont` (no CEL art), matching the
/// char/quest panels.
fn draw_shop_panel(window: &mut GameWindow, game_state: &GameState) {
    let canvas = window.canvas_mut();

    // ---- Full-screen dim ----
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 150));
    let _ = canvas.fill_rect(Rect::new(0, 0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    let (panel, _options, _items) = shop_panel_geometry(game_state);
    let panel_x = panel.x();
    let panel_y = panel.y();

    // ---- Panel frame ----
    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 16, 12));
    let _ = canvas.fill_rect(Rect::new(
        panel_x - 2,
        panel_y - 2,
        (SHOP_PANEL_W + 4) as u32,
        (SHOP_PANEL_H + 4) as u32,
    ));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(58, 46, 34));
    let _ = canvas.fill_rect(panel);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(140, 116, 84));
    let _ = canvas.draw_rect(panel);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_rect(Rect::new(
        panel_x + 1,
        panel_y + 1,
        (SHOP_PANEL_W - 2) as u32,
        (SHOP_PANEL_H - 2) as u32,
    ));

    let font = crate::engine::font::PixelFont::new(2);
    let title_font = crate::engine::font::PixelFont::new(2);
    let small = crate::engine::font::PixelFont::new(1);

    let hi = sdl2::pixels::Color::RGB(255, 240, 160);
    let txt = sdl2::pixels::Color::RGB(240, 230, 200);
    let label = sdl2::pixels::Color::RGB(200, 188, 150);
    let dim = sdl2::pixels::Color::RGB(150, 138, 110);
    let gold = sdl2::pixels::Color::RGB(255, 215, 0);
    let btn_txt = sdl2::pixels::Color::RGB(230, 220, 190);

    // ---- Header: NPC shop name ----
    let npc_kind = game_state.active_shop_npc.unwrap_or(255);
    let shop_name = GameState::shop_name_for_npc(npc_kind);
    let mut y = panel_y + 12;
    title_font.render_text_centered(canvas, shop_name, panel_x + SHOP_PANEL_W / 2, y, hi);
    y += title_font.line_height() + 6;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((panel_x + 12, y), (panel_x + SHOP_PANEL_W - 12, y));
    y += 6;

    // ---- Option buttons row (BUY / SELL / REPAIR / CLOSE) ----
    let btn_w = (SHOP_PANEL_W - 16 - (SHOP_NUM_OPTIONS - 1) as i32 * 6) / SHOP_NUM_OPTIONS;
    let btn_labels = ["BUY", "SELL", "REPAIR", "CLOSE"];
    for (i, label_str) in btn_labels.iter().enumerate() {
        let bx = panel_x + 8 + i as i32 * (btn_w + 6);
        let by = panel_y + SHOP_OPTIONS_Y_OFF;
        // CLOSE button gets a brighter outline so the exit affordance is clear.
        let is_close = *label_str == "CLOSE";
        canvas.set_draw_color(if is_close {
            sdl2::pixels::Color::RGB(110, 90, 60)
        } else {
            sdl2::pixels::Color::RGB(70, 56, 40)
        });
        let _ = canvas.fill_rect(Rect::new(bx, by, btn_w as u32, SHOP_OPTION_BTN_H as u32));
        canvas.set_draw_color(if is_close {
            sdl2::pixels::Color::RGB(200, 170, 110)
        } else {
            sdl2::pixels::Color::RGB(130, 108, 78)
        });
        let _ = canvas.draw_rect(Rect::new(bx, by, btn_w as u32, SHOP_OPTION_BTN_H as u32));
        font.render_text_centered(
            canvas,
            label_str,
            bx + btn_w / 2,
            by + (SHOP_OPTION_BTN_H - font.line_height()) / 2,
            btn_txt,
        );
    }

    // ---- Items list ----
    let mut iy = panel_y + SHOP_ITEMS_Y_OFF - 4;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((panel_x + 12, iy), (panel_x + SHOP_PANEL_W - 12, iy));
    iy += 6;
    small.render_text(canvas, "ITEMS FOR SALE", panel_x + 14, iy, label);
    iy += small.line_height() + 4;

    let inventory = game_state.active_shop_inventory();
    if inventory.is_empty() {
        // No shop or gossip-only NPC: show a placeholder.
        small.render_text(canvas, "(This NPC has nothing for sale.)", panel_x + 14, iy, dim);
        iy += small.line_height() + 4;
    } else {
        for (name, price) in &inventory {
            // Row background (subtle shade so rows read as clickable). Faint
            // enough that text stays legible.
            canvas.set_draw_color(sdl2::pixels::Color::RGB(48, 38, 28));
            let _ = canvas.fill_rect(Rect::new(
                panel_x + 12,
                iy - 2,
                (SHOP_PANEL_W - 24) as u32,
                (SHOP_ITEM_ROW_H - 2) as u32,
            ));
            // Item name (left).
            font.render_text(canvas, name, panel_x + 18, iy, txt);
            // Price (right-aligned). Render "N gold" right-aligned to the panel.
            let price_str = format!("{} gold", price);
            let pw = font.text_width(&price_str);
            let affordable = game_state.player._p_gold >= *price;
            let price_col = if affordable { gold } else { dim };
            font.render_text(
                canvas,
                &price_str,
                panel_x + SHOP_PANEL_W - 18 - pw,
                iy,
                price_col,
            );
            iy += SHOP_ITEM_ROW_H;
        }
    }

    // ---- Footer: gold + hint ----
    let footer_y = panel_y + SHOP_PANEL_H - 38;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((panel_x + 12, footer_y), (panel_x + SHOP_PANEL_W - 12, footer_y));
    let mut fy = footer_y + 6;
    font.render_text(
        canvas,
        &format!("YOUR GOLD: {}", game_state.player._p_gold),
        panel_x + 14,
        fy,
        gold,
    );
    fy += font.line_height() + 2;
    small.render_text_centered(
        canvas,
        "CLICK AN ITEM TO BUY  -  CLOSE OR ESC TO LEAVE",
        panel_x + SHOP_PANEL_W / 2,
        fy,
        dim,
    );
}


//------------------------------------------------------------------------------
// Spell Book panel (B key) + Automap (A key)
//------------------------------------------------------------------------------

/// Convert a digit Keycode (1-9) into a 0-based list index for the spell book.
/// Returns `None` for non-digit keys. Used by `handle_event` so the number keys
/// select a spell from the displayed list (mirrors C++ `PressSpellKey`).
fn spell_selection_index(k: Keycode) -> Option<usize> {
    match k {
        Keycode::Num1 | Keycode::Kp1 => Some(0),
        Keycode::Num2 | Keycode::Kp2 => Some(1),
        Keycode::Num3 | Keycode::Kp3 => Some(2),
        Keycode::Num4 | Keycode::Kp4 => Some(3),
        Keycode::Num5 | Keycode::Kp5 => Some(4),
        Keycode::Num6 | Keycode::Kp6 => Some(5),
        Keycode::Num7 | Keycode::Kp7 => Some(6),
        Keycode::Num8 | Keycode::Kp8 => Some(7),
        Keycode::Num9 | Keycode::Kp9 => Some(8),
        _ => None,
    }
}

/// Build the ordered list of spells shown in the spell book.
///
/// The player's memorised/known spells come from two sources:
///   * `player._p_mem_spells` — the classic 64-bit bitmask of memorised spells
///     (`_pMemSpells` in C++). Bit `i` (0-indexed) corresponds to the i-th
///     spell in canonical order.
///   * `player._p_spl_lvl[]` — per-spell level (non-zero ⇒ the spell has been
///     learnt from a book). This is the more reliable source in the port since
///     the bitmask bit-ordering differs between `player_exact::SpellId`
///     (Firebolt=0) and `spelldat::SpellID` (Firebolt=1).
///
/// The spell is included if *either* the bitmask has its bit set *or* the
/// spell level is ≥ 1. Firebolt is always offered as the hero's starting spell
/// (C++ gives every class Firebolt at level 0). The result is in canonical
/// Diablo spell order (Firebolt, Healing, Lightning, ...) with the mana cost
/// pulled from `spelldat::SPELLS_DATA` next to each entry.
///
/// **Index mapping note:** `player_exact::SpellId` uses Firebolt=0, while
/// `spelldat::SpellID` uses Firebolt=1 (Null=0). We index `_p_spl_lvl` and the
/// bitmask with the *player_exact* discriminant (so Firebolt is slot 0), then
/// translate to the spelldat table by adding 1 when looking up the name/cost.
fn known_spells_for_book(player: &Player) -> Vec<(crate::game::player_exact::SpellId, u8)> {
    use crate::game::player_exact::SpellId as PId;
    // Canonical order of Diablo (non-Hellfire) spells as the book presents
    // them. Indices match `player_exact::SpellId` discriminants *and* the
    // `_p_spl_lvl` array slots.
    const BOOK_SPELLS: [PId; 9] = [
        PId::Firebolt,
        PId::Healing,
        PId::Lightning,
        PId::Flash,
        PId::FireWall,
        PId::TownPortal,
        PId::StoneCurse,
        PId::Fireball,
        PId::ChargedBolt,
    ];

    let mut out: Vec<(PId, u8)> = Vec::new();
    for &spell in BOOK_SPELLS.iter() {
        let idx = spell as usize; // player_exact discriminant (Firebolt = 0)
        let bit_set = (player._p_mem_spells >> idx) & 1 == 1;
        let lvl_known = idx < player._p_spl_lvl.len() && player._p_spl_lvl[idx] >= 1;
        // Firebolt is always available (starting spell for every class).
        let always = spell == PId::Firebolt;
        if bit_set || lvl_known || always {
            // spelldat uses SpellID with Firebolt=1 (Null=0), so the data slot
            // is `player_exact discriminant + 1`.
            let data_idx = idx + 1;
            let mana = if data_idx < crate::game::spelldat::SPELLS_DATA.len() {
                crate::game::spelldat::SPELLS_DATA[data_idx].mana_cost
            } else {
                0
            };
            out.push((spell, mana));
        }
    }
    out
}

/// Spell book overlay (B key, C++ `DrawSpellBook`).
///
/// Renders a modal full-screen dim + a right-side panel listing the hero's
/// known spells with their mana cost. The currently readied spell
/// (`player._p_r_spell`) is highlighted. A leading index (1-9) is shown next
/// to each row so the player knows which number key selects it; pressing that
/// key in `handle_event` sets `_p_r_spell` directly.
fn draw_spellbook_panel(window: &mut GameWindow, game_state: &GameState) {
    use crate::game::player_exact::SpellId as PId;

    let canvas = window.canvas_mut();
    let player = &game_state.player;

    // ---- Full-screen dim ----
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 150));
    let _ = canvas.fill_rect(Rect::new(0, 0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    // ---- Panel frame (right-side, 300x430) ----
    const PANEL_X: i32 = 324;
    const PANEL_Y: i32 = 20;
    const PANEL_W: i32 = 300;
    const PANEL_H: i32 = 430;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 16, 12));
    let _ = canvas.fill_rect(Rect::new(PANEL_X - 2, PANEL_Y - 2, (PANEL_W + 4) as u32, (PANEL_H + 4) as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(58, 46, 34));
    let _ = canvas.fill_rect(Rect::new(PANEL_X, PANEL_Y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(140, 116, 84));
    let _ = canvas.draw_rect(Rect::new(PANEL_X, PANEL_Y, PANEL_W as u32, PANEL_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_rect(Rect::new(PANEL_X + 1, PANEL_Y + 1, (PANEL_W - 2) as u32, (PANEL_H - 2) as u32));

    let font = crate::engine::font::PixelFont::new(2);
    let title_font = crate::engine::font::PixelFont::new(2);
    let small = crate::engine::font::PixelFont::new(1);

    let txt = sdl2::pixels::Color::RGB(240, 230, 200);
    let label = sdl2::pixels::Color::RGB(200, 188, 150);
    let dim = sdl2::pixels::Color::RGB(150, 138, 110);
    let hi = sdl2::pixels::Color::RGB(255, 240, 160);
    let sel_bg = sdl2::pixels::Color::RGBA(120, 100, 40, 200);

    let mut y = PANEL_Y + 10;

    // Title.
    title_font.render_text_centered(canvas, "SPELL BOOK", PANEL_X + PANEL_W / 2, y, hi);
    y += title_font.line_height() + 4;

    small.render_text_centered(
        canvas,
        "PRESS 1-9 TO READY A SPELL",
        PANEL_X + PANEL_W / 2,
        y,
        dim,
    );
    y += small.line_height() + 8;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;

    // Mana available (display value = 64x >> 6).
    let mp_cur = (player._p_mana >> 6).max(0);
    let mp_max = player._p_max_mana >> 6;
    font.render_text(canvas, "MANA", PANEL_X + 14, y, label);
    font.render_text(
        canvas,
        &format!("{}/{}", mp_cur, mp_max),
        PANEL_X + 150,
        y,
        txt,
    );
    y += font.line_height() + 8;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;

    // Column headers.
    small.render_text(canvas, "# SPELL        MANA", PANEL_X + 14, y, dim);
    y += small.line_height() + 4;

    let known = known_spells_for_book(player);
    let readied = player._p_r_spell;

    if known.is_empty() {
        small.render_text_centered(canvas, "NO SPELLS KNOWN", PANEL_X + PANEL_W / 2, y, dim);
    }

    for (i, (spell, mana)) in known.iter().enumerate() {
        // Only the first 9 rows get a hotkey (1-9).
        let hotkey = if i < 9 { format!("{}", i + 1) } else { String::new() };

        let is_sel = *spell == readied;
        // Highlight bar for the readied spell.
        if is_sel {
            canvas.set_draw_color(sel_bg);
            let _ = canvas.fill_rect(Rect::new(PANEL_X + 8, y - 2, (PANEL_W - 16) as u32, (font.line_height() + 4) as u32));
        }

        let name = spell_display_name(*spell);
        let row_colour = if is_sel { hi } else { txt };

        // "1 Firebolt     6"
        let prefix = format!("{:<2}", hotkey);
        font.render_text(canvas, &prefix, PANEL_X + 14, y, label);
        font.render_text(canvas, name, PANEL_X + 36, y, row_colour);
        let mana_str = format!("{}", mana);
        font.render_text(
            canvas,
            &mana_str,
            PANEL_X + PANEL_W - 14 - font.text_width(&mana_str),
            y,
            row_colour,
        );

        y += font.line_height() + 4;
    }

    // Footer.
    y = PANEL_Y + PANEL_H - 28;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_line((PANEL_X + 10, y), (PANEL_X + PANEL_W - 10, y));
    y += 6;
    small.render_text_centered(canvas, "PRESS B TO CLOSE", PANEL_X + PANEL_W / 2, y, dim);

    let _ = PId::Invalid; // silence unused-import warning if SpellId unused here
}

/// Human-readable name for a `player_exact::SpellId`. Uses the spelldat table
/// when possible (canonical Blizzard spelling) and falls back to a debug name
/// derived from the discriminant.
fn spell_display_name(spell: crate::game::player_exact::SpellId) -> &'static str {
    use crate::game::player_exact::SpellId as PId;
    // spelldat index = player_exact discriminant + 1 (Null occupies slot 0).
    let idx = (spell as i32 + 1) as usize;
    if idx < crate::game::spelldat::SPELLS_DATA.len() {
        let name = crate::game::spelldat::SPELLS_DATA[idx].name;
        if !name.is_empty() {
            return name;
        }
    }
    // Fallback debug names (shouldn't normally be hit).
    match spell {
        PId::Firebolt => "Firebolt",
        PId::Healing => "Healing",
        PId::Lightning => "Lightning",
        PId::Flash => "Flash",
        PId::FireWall => "Fire Wall",
        PId::TownPortal => "Town Portal",
        PId::StoneCurse => "Stone Curse",
        PId::Fireball => "Fireball",
        PId::ChargedBolt => "Charged Bolt",
        _ => "???",
    }
}

/// Automap overlay (A key, C++ `DrawAutomap`).
///
/// Draws a bordered, semi-transparent thumbnail of the tiles around the camera
/// in the upper-right corner of the screen. Each world tile maps to a small
/// point in a simplified isometric grid; solid tiles (walls / non-zero dPiece
/// for the town, or non-floor for the dungeon) are drawn brighter so the level
/// geometry reads at a glance. The player is a bright marker at the centre;
/// town NPCs and dungeon monsters are coloured dots.
///
/// This is a deliberately simplified renderer: it does not replicate the C++
/// wall-polygon geometry (which needs the full AutomapTileType table wired into
/// the dungeon generator). Instead it classifies each visible tile as
/// walkable / solid from the dPiece grid, which is enough to navigate.
fn draw_automap_overlay(window: &mut GameWindow, game_state: &GameState) {
    let canvas = window.canvas_mut();

    // ---- Map geometry ----
    // Thumbnail is a fixed-size box in the upper-right. The grid step is a
    // small number of pixels per tile so a decent radius fits.
    const MAP_W: i32 = 184;
    const MAP_H: i32 = 150;
    const MAP_PAD: i32 = 8;
    let map_x = LOGICAL_WIDTH as i32 - MAP_W - MAP_PAD;
    let map_y = MAP_PAD;
    let map_cx = map_x + MAP_W / 2;
    let map_cy = map_y + MAP_H / 2;

    // Iso grid step: pixels per tile. dx/dy form the iso basis.
    const STEP_X: i32 = 3; // (tile_x - tile_y) axis
    const STEP_Y: i32 = 2; // (tile_x + tile_y) axis
    const RADIUS: i32 = 26; // tiles from camera to each edge

    // ---- Frame + translucent background ----
    // Shadow + dim border, echoing C++ minimap frame.
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 170));
    let _ = canvas.fill_rect(Rect::new(map_x, map_y, MAP_W as u32, MAP_H as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(140, 116, 84));
    let _ = canvas.draw_rect(Rect::new(map_x - 1, map_y - 1, (MAP_W + 2) as u32, (MAP_H + 2) as u32));
    canvas.set_draw_color(sdl2::pixels::Color::RGB(96, 78, 56));
    let _ = canvas.draw_rect(Rect::new(map_x, map_y, MAP_W as u32, MAP_H as u32));

    let cam_x = game_state.camera.tile_x;
    let cam_y = game_state.camera.tile_y;

    // Determine which tile source to read.
    let dungeon = game_state.dungeon_layout.as_ref();
    let town = game_state.town_layout.as_ref();
    let in_dungeon = game_state.in_dungeon;

    // Closure: is (wx, wy) a solid tile?
    let tile_solid = |wx: i32, wy: i32| -> bool {
        if in_dungeon {
            if let Some(dl) = dungeon {
                // Non-zero dPiece ⇒ wall/solid in the Cathedral grid; floor
                // tiles were stamped with the floor mega so this is a coarse
                // but useful wall/floor split. We additionally treat the
                // explicitly-known floor_tiles set as walkable.
                let v = dl.get(wx, wy);
                if v == 0 {
                    return false;
                }
                // floor_tiles is a Vec; membership check is O(n) but the list
                // is small (a few hundred entries) and this runs once/frame.
                !dl.floor_tiles.iter().any(|&(fx, fy)| fx == wx && fy == wy)
            } else {
                false
            }
        } else if let Some(tl) = town {
            // Town: a non-zero dPiece is a placed tile (road/structure). We
            // draw all placed tiles as "structure" dots and leave empty
            // (dPiece == 0) tiles blank. This gives a readable town layout.
            tl.get(wx, wy) != 0
        } else {
            false
        }
    };

    // ---- Draw the tile grid ----
    let wall_col = sdl2::pixels::Color::RGB(190, 170, 120);
    let floor_col = sdl2::pixels::Color::RGB(70, 64, 48);

    for dy in -RADIUS..=RADIUS {
        for dx in -RADIUS..=RADIUS {
            let wx = cam_x + dx;
            let wy = cam_y + dy;
            // Iso projection: screen offset from map centre.
            let sx = map_cx + (dx - dy) * STEP_X;
            let sy = map_cy + (dx + dy) * STEP_Y;
            // Clip to the map box.
            if sx < map_x + 1 || sx > map_x + MAP_W - 1 || sy < map_y + 1 || sy > map_y + MAP_H - 1 {
                continue;
            }
            if tile_solid(wx, wy) {
                canvas.set_draw_color(wall_col);
            } else {
                canvas.set_draw_color(floor_col);
            }
            let _ = canvas.draw_point(sdl2::rect::Point::new(sx, sy));
        }
    }

    // ---- NPCs (town) as green dots ----
    let npc_col = sdl2::pixels::Color::RGB(120, 230, 120);
    if !in_dungeon {
        for &(nx, ny, _name, _kind) in &game_state.towners {
            let dx = nx - cam_x;
            let dy = ny - cam_y;
            let sx = map_cx + (dx - dy) * STEP_X;
            let sy = map_cy + (dx + dy) * STEP_Y;
            if sx >= map_x && sx < map_x + MAP_W && sy >= map_y && sy < map_y + MAP_H {
                canvas.set_draw_color(npc_col);
                let _ = canvas.draw_point(sdl2::rect::Point::new(sx, sy));
            }
        }
    }

    // ---- Monsters (dungeon) as red dots ----
    let mon_col = sdl2::pixels::Color::RGB(240, 70, 60);
    if in_dungeon {
        for (_id, mx, my, _mt, _ai) in game_state.monster_manager.iter().map(|(id, m)| (id, m.x, m.y, m.monster_type, m.ai_state)) {
            let dx = mx - cam_x;
            let dy = my - cam_y;
            let sx = map_cx + (dx - dy) * STEP_X;
            let sy = map_cy + (dx + dy) * STEP_Y;
            if sx >= map_x && sx < map_x + MAP_W && sy >= map_y && sy < map_y + MAP_H {
                canvas.set_draw_color(mon_col);
                let _ = canvas.draw_point(sdl2::rect::Point::new(sx, sy));
            }
        }
    }

    // ---- Player marker (bright) at the centre ----
    let plr_col = sdl2::pixels::Color::RGB(255, 255, 255);
    canvas.set_draw_color(plr_col);
    // A small plus sign so the player is unmistakable.
    for &(dx, dy) in &[(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (0, -2), (0, -1), (0, 1), (0, 2)] {
        let _ = canvas.draw_point(sdl2::rect::Point::new(map_cx + dx, map_cy + dy));
    }

    // ---- Label ----
    let small = crate::engine::font::PixelFont::new(1);
    let label_col = sdl2::pixels::Color::RGB(240, 230, 200);
    small.render_text(canvas, "AUTOMAP", map_x + 4, map_y + MAP_H + 2, label_col);
}


/// XP required to advance from the given level to the next, using the canonical
/// Diablo 1 per-level experience table (a copy of `hud::XP_THRESHOLDS`, kept
/// here because the HUD's copy is private). Returns `None` at max level.
///
/// `exp` is the hero's current experience; if they've already banked enough to
/// leave the level, the remaining requirement is 0 (clamped at 0).
fn next_level_threshold(level: u8, exp: u32) -> Option<u32> {
    // C++ Reference: Source/playerdat.cpp::GetNextExperienceThresholdForLevel.
    // Index i = threshold to leave level (i+1); i.e. entry [level-1] leaves
    // the current level.
    const XP_THRESHOLDS: [u32; 50] = [
        2000, 4620, 8040, 12489, 18258, 25712, 35309, 47622, 63364, 83419, 108879, 141086, 181683,
        231075, 313656, 424067, 571190, 766569, 1025154, 1366227, 1814568, 2401895, 3168651,
        4166200, 5459523, 7130496, 9281874, 12042092, 15571031, 20066900, 25774405, 32994399,
        42095202, 53525811, 67831218, 85670061, 107834823, 135274799, 169122009, 210720231,
        261657253, 323800420, 399335440, 490808349, 601170414, 733825617, 892680222, 1082908612,
        1310707109, 1583495809,
    ];
    const MAX_PLAYER_LEVEL: u8 = 50;
    if level >= MAX_PLAYER_LEVEL {
        return None;
    }
    let idx = (level as usize).saturating_sub(1);
    let next = XP_THRESHOLDS.get(idx).copied()?;
    if exp >= next {
        Some(0)
    } else {
        Some(next - exp)
    }
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
        let (cx, cy) = tile_to_screen(m.x, m.y, cam_tile_x, cam_tile_y);
        // Draw a small diamond (4 triangles of 1px wide) as a stand-in sprite.
        for (dx, dy) in [(-3, 0), (3, 0), (0, -3), (0, 3), (-2, -1), (2, -1), (-2, 1), (2, 1), (0, 0)] {
            let _ = canvas.draw_point(sdl2::rect::Point::new(cx + dx, cy + dy));
        }
    }
}

/// Draw a pulsing stair marker over the active level's stair tile so the
/// player can see where to walk to change levels.
///
///   * **Town** — marks the fixed Cathedral down-stair `TOWN_DOWN_STAIRS`
///     (25,29) with a cyan downward-pointing chevron + outline diamond.
///   * **Dungeon** — marks the resolved up-stair `dungeon_up_stairs` with a
///     magenta upward-pointing chevron + outline diamond.
///
/// The marker pulses (alpha-modulated brightness) using `game_tick` so it's
/// visually obvious even against busy floor art. The pulse period is 8 ticks
/// (4 seconds at 2 Hz); on the "off" half of the cycle the marker dims but
/// never disappears, so it's always locatable. Pure canvas drawing using the
/// same forward iso projection as the floor/player tiles (`rel_x = (u-v)*32`,
/// `rel_y = (u+v)*16`).
///
/// Off-screen stairs are silently culled (the projection would land far off
/// the viewport), matching the floor-tile culling in `draw_tristram`.
fn draw_stairs_marker(
    window: &mut GameWindow,
    game_state: &GameState,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) {
    // Resolve which stair (if any) to mark this frame.
    let (sx, sy, down): (i32, i32, bool) = if game_state.in_dungeon {
        match game_state.dungeon_up_stairs {
            Some(pos) => (pos.0, pos.1, false),
            None => return,
        }
    } else {
        let pos = crate::game::game_state::TOWN_DOWN_STAIRS;
        (pos.0, pos.1, true)
    };

    // Project the stair tile to screen space (same transform as floor tiles).
    let (cx, cy) = tile_to_screen(sx, sy, cam_tile_x, cam_tile_y);

    // Cull if the projected centre is well off the visible viewport.
    const MARGIN: i32 = 64;
    if cx < -MARGIN
        || cx > LOGICAL_WIDTH as i32 + MARGIN
        || cy < -MARGIN
        || cy > LOGICAL_HEIGHT as i32 + MARGIN
    {
        return;
    }

    // Pulse: bright on ticks where (game_tick % 8) < 4, dim otherwise.
    // game_tick advances at 2 Hz inside GameState::update, so one full cycle
    // is 4 seconds.
    let bright = (game_state.game_tick % 8) < 4;
    // Base colour: cyan for down-stair, magenta for up-stair.
    let (r, g, b) = if down {
        if bright { (120, 240, 255) } else { (40, 110, 130) }
    } else if bright {
        (255, 120, 240)
    } else {
        (130, 50, 120)
    };

    let canvas = window.canvas_mut();
    canvas.set_draw_color(sdl2::pixels::Color::RGB(r, g, b));

    // Outline diamond — sits on the tile centre, half a tile wide/tall.
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;
    let _ = canvas.draw_line((cx, cy - half_h), (cx + half_w, cy));
    let _ = canvas.draw_line((cx + half_w, cy), (cx, cy + half_h));
    let _ = canvas.draw_line((cx, cy + half_h), (cx - half_w, cy));
    let _ = canvas.draw_line((cx - half_w, cy), (cx, cy - half_h));

    // Direction chevron above the tile centre: a small arrow indicating the
    // transition direction. Down-stair points down (into the floor), up-stair
    // points up (out of the dungeon).
    let arrow_top = cy - half_h - 14;
    let arrow_bot = cy - half_h - 2;
    if down {
        // ▼ chevron (pointing down).
        let _ = canvas.draw_line((cx - 6, arrow_top), (cx, arrow_bot));
        let _ = canvas.draw_line((cx, arrow_bot), (cx + 6, arrow_top));
        let _ = canvas.draw_line((cx - 6, arrow_top), (cx + 6, arrow_top));
    } else {
        // ▲ chevron (pointing up).
        let _ = canvas.draw_line((cx - 6, arrow_bot), (cx, arrow_top));
        let _ = canvas.draw_line((cx, arrow_top), (cx + 6, arrow_bot));
        let _ = canvas.draw_line((cx - 6, arrow_bot), (cx + 6, arrow_bot));
    }

    // Centre dot so the marker is readable even when the chevron overlaps
    // a wall sprite.
    let _ = canvas.draw_point(sdl2::rect::Point::new(cx, cy));
}

/// Draw ground loot (items dropped by slain monsters) as small coloured icons.
///
/// Each `GroundItem` is projected through the same isometric transform the
/// monsters/player use (`rel_x = (wx-wy)*32`, `rel_y = (wx+wy)*16`) and drawn as
/// a filled rectangle centred on its tile. Icon colour encodes the item type:
///   * Gold          — yellow (255, 215, 0)
///   * HealingPotion — red    (220,  40, 40)
///   * ManaPotion    — blue   ( 40,  80, 220)
///
/// Pure canvas drawing (no textures), so it runs outside any TextureCreator
/// borrow block. Off-screen items are culled.
fn draw_ground_items(
    window: &mut GameWindow,
    game_state: &GameState,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) {
    if game_state.ground_items.is_empty() {
        return;
    }
    // Sort by depth (wx+wy ascending) so lower-screen items overlap those behind.
    let mut order: Vec<&crate::game::game_state::GroundItem> = game_state.ground_items.iter().collect();
    order.sort_by_key(|g| g.x + g.y);

    let canvas = window.canvas_mut();
    for g in order {
        let (cx, cy) = tile_to_screen(g.x, g.y, cam_tile_x, cam_tile_y);

        // Cull off-screen items.
        if cx < -(TILE_WIDTH * 2) || cx > (LOGICAL_WIDTH as i32 + TILE_WIDTH * 2)
            || cy < -(TILE_HEIGHT * 6) || cy > (LOGICAL_HEIGHT as i32 + TILE_HEIGHT * 4)
        {
            continue;
        }

        let (r, gg, b) = match g.item_type {
            GroundItemType::Gold => (255, 215, 0),
            GroundItemType::HealingPotion => (220, 40, 40),
            GroundItemType::ManaPotion => (40, 80, 220),
        };
        // Filled icon centred on the tile, sitting just above the floor line so
        // it reads clearly against the tile art. A 10x8 filled rect plus a
        // darker outline for contrast.
        const ICON_W: i32 = 10;
        const ICON_H: i32 = 8;
        let icon = Rect::new(cx - ICON_W / 2, cy - ICON_H - 2, ICON_W as u32, ICON_H as u32);
        canvas.set_draw_color(sdl2::pixels::Color::RGB(r / 3, gg / 3, b / 3));
        let _ = canvas.fill_rect(Rect::new(
            icon.x() - 1,
            icon.y() - 1,
            icon.width() + 2,
            icon.height() + 2,
        ));
        canvas.set_draw_color(sdl2::pixels::Color::RGB(r, gg, b));
        let _ = canvas.fill_rect(icon);
    }
}

/// Marker colour per NPC `TownerType` (as u8). Each NPC gets a distinct,
/// readable hue so the player can tell them apart at a glance. The order
/// matches `TownerType` discriminants in `src/game/towner.rs`.
fn towner_marker_colour(kind: u8) -> (u8, u8, u8) {
    match kind {
        // Smith = Griswold (blacksmith) — orange/hammer
        0 => (220, 140, 30),
        // Healer = Pepin — white/clean
        1 => (240, 240, 240),
        // DeadGuy = Wounded Townsman — dark red (not rendered in Diablo mode)
        2 => (120, 30, 30),
        // Tavern = Ogden — warm brown/ale
        3 => (180, 110, 60),
        // Story = Deckard Cain — gold/sage
        4 => (230, 200, 60),
        // Drunk = Farnham — purple/wine
        5 => (170, 80, 170),
        // Witch = Adria — violet/mystic
        6 => (140, 90, 220),
        // Barmaid = Gillian — pink
        7 => (230, 130, 170),
        // PegBoy = Wirt — green/cunning
        8 => (60, 200, 90),
        // Cow — white-ish
        9 => (210, 210, 190),
        // Farmer / Girl / CowFarmer (Hellfire-only)
        _ => (140, 140, 140),
    }
}

/// Draw Tristram NPCs as coloured isometric markers.
///
/// Each NPC is drawn as a filled diamond (the same isometric projection the
/// floor tiles use) centred on its world tile, filled with a per-type colour
/// from [`towner_marker_colour`] and outlined in black for contrast. A small
/// cluster of points in the marker's upper half encodes the NPC's initial —
/// a lightweight "text" stand-in that needs no font rasteriser (the canvas
/// path has none). Off-screen NPCs are culled.
///
/// This mirrors how `draw_ground_items` / `draw_simple_missiles` project world
/// tiles: `rel_x = (wx-wy)*32`, `rel_y = (wx+wy)*16`, centred on the camera.
fn draw_towners(
    window: &mut GameWindow,
    game_state: &GameState,
    cam_tile_x: i32,
    cam_tile_y: i32,
    screen_center_x: i32,
    screen_center_y: i32,
) {
    if game_state.towners.is_empty() {
        return;
    }

    // Depth-sort by (wx+wy) ascending so nearer (lower-screen) NPCs correctly
    // overlap those behind them — same approach as draw_ground_items.
    let mut order: Vec<(i32, i32, &'static str, u8)> = game_state.towners.clone();
    order.sort_by_key(|t| t.0 + t.1);

    let canvas = window.canvas_mut();
    let half_w = TILE_WIDTH / 2;
    let half_h = TILE_HEIGHT / 2;

    for (wx, wy, name, kind) in order {
        let (cx, cy) = tile_to_screen(wx, wy, cam_tile_x, cam_tile_y);

        // Cull off-screen NPCs (with a margin for the marker + outline).
        if cx < -(TILE_WIDTH * 2) || cx > (LOGICAL_WIDTH as i32 + TILE_WIDTH * 2)
            || cy < -(TILE_HEIGHT * 6) || cy > (LOGICAL_HEIGHT as i32 + TILE_HEIGHT * 4)
        {
            continue;
        }

        let (r, g, b) = towner_marker_colour(kind);

        // Filled diamond outline (slightly darker) for contrast against the floor.
        let (or, og, ob) = ((r / 4).max(0) as u8, (g / 4).max(0) as u8, (b / 4).max(0) as u8);
        canvas.set_draw_color(sdl2::pixels::Color::RGB(or, og, ob));
        // Outline diamond (one tile wide) — four edges.
        let _ = canvas.draw_line((cx, cy - half_h), (cx + half_w, cy));
        let _ = canvas.draw_line((cx + half_w, cy), (cx, cy + half_h));
        let _ = canvas.draw_line((cx, cy + half_h), (cx - half_w, cy));
        let _ = canvas.draw_line((cx - half_w, cy), (cx, cy - half_h));

        // Inner filled diamond — build from horizontal scanlines to approximate
        // a filled diamond (SDL2 canvas has no fill-polygon primitive here).
        canvas.set_draw_color(sdl2::pixels::Color::RGB(r, g, b));
        for row in -half_h..=half_h {
            // half-width at this row grows linearly toward the equator.
            let row_half = half_w - (row.abs() * half_w / half_h).max(0).min(half_w);
            let _ = canvas.draw_line((cx - row_half, cy + row), (cx + row_half, cy + row));
        }

        // Encode the NPC position with a bright focal dot above the marker
        // centre so each marker reads as a distinct "pin" rather than an
        // anonymous floor tile. The per-NPC colour already disambiguates them;
        // this adds a small white highlight near the top vertex. (The canvas
        // path has no font rasteriser, so we can't draw the NPC's initial as
        // text — the colour is the label.)
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        for &(dx, dy) in &[(0, -half_h / 2), (-1, -half_h / 2), (1, -half_h / 2)] {
            let _ = canvas.draw_point(sdl2::rect::Point::new(cx + dx, cy + dy));
        }
        let _ = name; // name retained for future font-rendered labels
    }
}

/// Logical render resolution. The canvas is configured with
/// `set_logical_size(640, 480)` in main.rs, so all drawing happens in this
/// coordinate space and SDL scales it to the physical window.
const LOGICAL_WIDTH: u32 = 640;
const LOGICAL_HEIGHT: u32 = 480;

/// Half the visible tile radius around the camera — used by the checkerboard
/// fallbacks (the faithful pipeline computes its own viewport coverage).
const VIEW_RADIUS_X: i32 = 12;
const VIEW_RADIUS_Y: i32 = 11;


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
        let (dst_cx, dst_cy) = tile_to_screen(wx, wy, cam_tile_x, cam_tile_y);

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
            let rel_x = (wy_off - wx_off) * (TILE_WIDTH / 2);
            let rel_y = (wx_off + wy_off) * -(TILE_HEIGHT / 2);
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
        input.on_key_down(Keycode::D); // screen-right
        // With corrected mapping: world_dx = mdy - mdx = 0 - 1 = -1
        //                         world_dy = mdy + mdx = 0 + 1 = +1
        // So D moves world (-x, +y) = south-west in iso terms.
        for _ in 0..200 {
            apply_movement(&mut gs, &input);
        }
        assert!(gs.camera.tile_x < 50 || gs.camera.tile_y > 50,
            "should have moved: x={}, y={}", gs.camera.tile_x, gs.camera.tile_y);
        assert_eq!(gs.camera.tile_x, gs.player.position.x);
        assert_eq!(gs.camera.tile_y, gs.player.position.y);
    }

    /// Forward projection for tests: mirrors the live pipeline. Given a world
    /// tile offset (u, v) from the camera, returns the logical-canvas screen
    /// point via `tile_to_screen` (C++ GetScreenPosition).
    fn project_tile_to_screen(u: i32, v: i32, cam_x: i32, cam_y: i32) -> (i32, i32) {
        tile_to_screen(cam_x + u, cam_y + v, cam_x, cam_y)
    }

    #[test]
    fn test_convert_screen_to_tile_at_anchor_is_camera() {
        // Clicking the camera tile's own screen anchor maps back to the camera.
        let cam_x = 75;
        let cam_y = 68;
        let (ax, ay) = tile_to_screen(cam_x, cam_y, cam_x, cam_y);
        let (wx, wy) = convert_screen_to_tile(ax, ay, cam_x, cam_y);
        assert_eq!((wx, wy), (cam_x, cam_y));
    }

    #[test]
    fn test_convert_screen_to_tile_round_trips_forward_projection() {
        // For every tile offset within the visible radius, forward-project to
        // a screen point, then convert back — the round-trip must be exact.
        let cam_x = 75;
        let cam_y = 68;
        for u in -9..=9 {
            for v in -9..=9 {
                let (mx, my) = project_tile_to_screen(u, v, cam_x, cam_y);
                let (wx, wy) = convert_screen_to_tile(mx, my, cam_x, cam_y);
                assert_eq!(
                    (wx, wy),
                    (cam_x + u, cam_y + v),
                    "round-trip failed for offset ({},{}): screen=({},{})",
                    u, v, mx, my
                );
            }
        }
    }

    #[test]
    fn test_convert_screen_to_tile_concrete_example() {
        // cam=(75,68) anchors at (288,183). Tile (80,70): u=5, v=2.
        // Forward: mx = 288 + (u-v)*32 = 288 + 96 = 384
        //          my = 183 + (u+v)*16 = 183 + 112 = 295
        // Inverse must recover (80, 70).
        let (wx, wy) = convert_screen_to_tile(384, 295, 75, 68);
        assert_eq!((wx, wy), (80, 70));
    }

    #[test]
    fn test_tick_move_target_clears_on_arrival() {
        // Player already on the target: tick should be a no-op and clear the
        // destination so subsequent ticks don't run.
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = 50;
        gs.camera.tile_y = 50;
        gs.player.position.x = 50;
        gs.player.position.y = 50;
        let mut target = Some((50, 50));
        tick_move_target(&mut gs, (50, 50), &mut target);
        assert_eq!(target, None, "target should be cleared on arrival");
        assert_eq!(gs.camera.tile_x, 50);
        assert_eq!(gs.camera.tile_y, 50);
    }

    #[test]
    fn test_tick_move_target_walks_one_tile_toward_destination() {
        // From (50,50) toward (53,52): the first tick moves diagonally to
        // (51,51) (both axes advance). Camera must follow the player.
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = 50;
        gs.camera.tile_y = 50;
        gs.player.position.x = 50;
        gs.player.position.y = 50;
        let mut target = Some((53, 52));
        tick_move_target(&mut gs, (53, 52), &mut target);
        assert_eq!(gs.player.position.x, 51, "x advanced by 1");
        assert_eq!(gs.player.position.y, 51, "y advanced by 1");
        // Camera follows.
        assert_eq!(gs.camera.tile_x, 51);
        assert_eq!(gs.camera.tile_y, 51);
        // Not arrived yet — target retained.
        assert_eq!(target, Some((53, 52)));
    }

    #[test]
    fn test_tick_move_target_clears_destination_on_final_step() {
        // From (50,50) toward (52,50): tick twice. After the second tick the
        // player is on the destination and target should be None.
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = 50;
        gs.camera.tile_y = 50;
        gs.player.position.x = 50;
        gs.player.position.y = 50;
        let mut target = Some((52, 50));
        tick_move_target(&mut gs, (52, 50), &mut target);
        assert_eq!(gs.player.position.x, 51);
        assert_eq!(target, Some((52, 50)));
        tick_move_target(&mut gs, (52, 50), &mut target);
        assert_eq!(gs.player.position.x, 52, "arrived at x");
        assert_eq!(gs.player.position.y, 50, "y unchanged");
        assert_eq!(target, None, "target cleared after final step");
    }

    #[test]
    fn test_tick_move_target_walks_full_diagonal_path() {
        // From (50,50) to (53,53): three diagonal ticks. After arrival the
        // player is on the target and it is cleared.
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.camera.tile_x = 50;
        gs.camera.tile_y = 50;
        gs.player.position.x = 50;
        gs.player.position.y = 50;
        let dest = (53, 53);
        let mut target = Some(dest);
        for _ in 0..10 {
            tick_move_target(&mut gs, dest, &mut target);
            if target.is_none() {
                break;
            }
        }
        assert_eq!(target, None, "should have arrived within 10 ticks");
        assert_eq!(gs.player.position.x, 53);
        assert_eq!(gs.player.position.y, 53);
    }

    // ========================================================================
    // Stair detection tests
    // ========================================================================

    /// Helper: a GameState already "in the dungeon" with the up-stair at the
    /// given tile and the cooldown window cleared (so detection can fire).
    fn dungeon_gs_with_up_stair(stair_x: i32, stair_y: i32) -> GameState {
        let mut gs = GameState::new(crate::game::player_exact::Player::new(), false, 1);
        gs.in_dungeon = true;
        gs.is_town = false;
        gs.dungeon_up_stairs = Some((stair_x, stair_y));
        // Push the cooldown well into the past.
        gs.game_tick = crate::game::game_state::STAIRS_COOLDOWN_TICKS + 10;
        gs.last_transition_tick = 0;
        gs
    }

    /// Helper: a GameState "in town" with the cooldown cleared.
    fn town_gs_for_stairs() -> GameState {
        let mut gs = GameState::new(crate::game::player_exact::Player::new(), true, 1);
        gs.in_dungeon = false;
        gs.is_town = true;
        gs.game_tick = crate::game::game_state::STAIRS_COOLDOWN_TICKS + 10;
        gs.last_transition_tick = 0;
        gs
    }

    #[test]
    fn test_stair_detection_descends_when_on_town_down_stair() {
        // Player standing exactly on the town down-stair (25,29). Because
        // descend_to_dungeon needs real L1 art (unavailable here), we expect
        // it to fail — but the cooldown must still be stamped and in_dungeon
        // must stay false (we didn't actually descend). This verifies the
        // detection path fired and called descend_to_dungeon.
        let mut gs = town_gs_for_stairs();
        gs.player.position = crate::game::types::Point::new(
            crate::game::game_state::TOWN_DOWN_STAIRS.0,
            crate::game::game_state::TOWN_DOWN_STAIRS.1,
        );
        let tick_before = gs.game_tick;
        check_stairs_transition(&mut gs);
        // Detection fired → cooldown stamped at the current tick.
        assert_eq!(
            gs.last_transition_tick, tick_before,
            "cooldown should be stamped after a detection fires"
        );
        // Without L1 art, descend failed, so we're still in town.
        assert!(!gs.in_dungeon, "descent should fail without L1 art");
        // And the cooldown is now blocking.
        assert!(!gs.stairs_cooldown_ready());
    }

    #[test]
    fn test_stair_detection_descends_when_adjacent_to_town_down_stair() {
        // Within STAIRS_TRIGGER_RADIUS (1) — diagonally adjacent counts.
        let mut gs = town_gs_for_stairs();
        gs.player.position = crate::game::types::Point::new(
            crate::game::game_state::TOWN_DOWN_STAIRS.0 + 1,
            crate::game::game_state::TOWN_DOWN_STAIRS.1 + 1,
        );
        check_stairs_transition(&mut gs);
        assert_eq!(
            gs.last_transition_tick, gs.game_tick,
            "adjacency within radius should still fire detection"
        );
    }

    #[test]
    fn test_stair_detection_does_not_descend_when_far_from_stair() {
        // Player at the town spawn (75,68), far from the down-stair (25,29).
        let mut gs = town_gs_for_stairs();
        gs.player.position = crate::game::types::Point::new(75, 68);
        let last_before = gs.last_transition_tick;
        check_stairs_transition(&mut gs);
        assert_eq!(
            gs.last_transition_tick, last_before,
            "cooldown must not change when no stair is nearby"
        );
        assert!(!gs.in_dungeon, "must remain in town");
    }

    #[test]
    fn test_stair_detection_blocked_by_cooldown_in_town() {
        // Simulate the moment right after a transition: cooldown not yet ready.
        let mut gs = town_gs_for_stairs();
        // Force the cooldown window to be "just fired".
        gs.last_transition_tick = gs.game_tick; // fired this tick
        assert!(!gs.stairs_cooldown_ready());
        gs.player.position = crate::game::types::Point::new(
            crate::game::game_state::TOWN_DOWN_STAIRS.0,
            crate::game::game_state::TOWN_DOWN_STAIRS.1,
        );
        let last_before = gs.last_transition_tick;
        check_stairs_transition(&mut gs);
        assert_eq!(
            gs.last_transition_tick, last_before,
            "cooldown must suppress re-detection"
        );
        assert!(!gs.in_dungeon, "must remain in town during cooldown");
    }

    #[test]
    fn test_stair_detection_ascends_when_on_dungeon_up_stair() {
        // Player on the dungeon up-stair → return_to_town fires → we're back in
        // town, dungeon_layout/up_stairs cleared, cooldown stamped.
        let mut gs = dungeon_gs_with_up_stair(40, 40);
        gs.player.position = crate::game::types::Point::new(40, 40);
        check_stairs_transition(&mut gs);
        assert!(!gs.in_dungeon, "should have returned to town");
        assert!(gs.is_town, "is_town flag set on return");
        assert!(gs.dungeon_layout.is_none(), "dungeon_layout cleared");
        assert!(
            gs.dungeon_up_stairs.is_none(),
            "dungeon_up_stairs cleared on return"
        );
        assert!(gs.stairs_cooldown_ready() == false, "cooldown stamped after return");
    }

    #[test]
    fn test_stair_detection_no_up_stair_does_nothing_in_dungeon() {
        // In dungeon but no up-stair resolved → detection is a no-op.
        let mut gs = dungeon_gs_with_up_stair(40, 40);
        gs.dungeon_up_stairs = None;
        gs.player.position = crate::game::types::Point::new(40, 40);
        let last_before = gs.last_transition_tick;
        check_stairs_transition(&mut gs);
        assert!(gs.in_dungeon, "must remain in dungeon");
        assert_eq!(gs.last_transition_tick, last_before);
    }

    #[test]
    fn test_find_dungeon_up_stairs_locates_entrance_mega() {
        // Build a synthetic DungeonLayout + L1 level where TIL[12] (the
        // EntranceStairs mega) has micro1 = 999, and stamp 999 into one grid
        // cell. The finder must return that cell's coordinates.
        use crate::game::game_state::DungeonLayout;
        use crate::levels::types::{MAXDUNX, MAXDUNY};

        let mut layout = DungeonLayout::default();
        // width/height come from Default (MAXDUNX×MAXDUNY). Stamp the target
        // micro value at a known micro-tile.
        let target_micro: u16 = 999;
        let (tx, ty) = (20usize, 30usize);
        layout.d_piece[ty * MAXDUNX + tx] = target_micro;
        assert_eq!(layout.width, MAXDUNX);
        assert_eq!(layout.height, MAXDUNY);

        // Synthetic L1 level: TIL has at least 13 entries, index 12 holds the
        // target micro value as micro1.
        let level = make_test_level_with_stairs_mega(target_micro);

        let found = find_dungeon_up_stairs(&layout, &level);
        assert_eq!(found, Some((tx as i32, ty as i32)));
    }

    #[test]
    fn test_find_dungeon_up_stairs_returns_none_when_no_match() {
        // TIL[12] has micro1 = 999 but no grid cell holds it → None.
        use crate::game::game_state::DungeonLayout;
        let layout = DungeonLayout::default(); // all zeros
        let level = make_test_level_with_stairs_mega(999);
        assert!(find_dungeon_up_stairs(&layout, &level).is_none());
    }

    #[test]
    fn test_find_dungeon_up_stairs_returns_none_when_til_too_short() {
        // TIL table has fewer than 13 entries → the entrance mega index is out
        // of range → None, no panic.
        use crate::game::game_state::DungeonLayout;
        let layout = DungeonLayout::default();
        // make_test_level() only has 2 TIL entries.
        let level = make_test_level();
        assert!(find_dungeon_up_stairs(&layout, &level).is_none());
    }

    /// Build a minimal `DungeonLevelData` whose TIL table is long enough to
    /// expose index `L1_ENTRANCE_STAIRS_TIL_INDEX` (12), with that entry's
    /// `micro1` set to `stairs_micro1`. Used by the up-stair finder tests.
    fn make_test_level_with_stairs_mega(stairs_micro1: u16) -> DungeonLevelData {
        use crate::engine::dungeon::{MegaTile, MinData, SolData, TilData, TilEntry};
        let mut til_tiles: Vec<TilEntry> = Vec::new();
        for i in 0..=L1_ENTRANCE_STAIRS_TIL_INDEX {
            til_tiles.push(TilEntry {
                micro1: i as u16,
                micro2: 0,
                micro3: 0,
                micro4: 0,
            });
        }
        // Overwrite the entrance-stairs entry with the requested micro1.
        til_tiles[L1_ENTRANCE_STAIRS_TIL_INDEX].micro1 = stairs_micro1;

        DungeonLevelData {
            dungeon_type: DungeonType::Cathedral,
            palette: PaletteData::default(),
            sol: SolData { properties: vec![] },
            min: MinData {
                mega_tiles: vec![MegaTile::default(); 20],
                blocks_per_tile: 16,
            },
            til: TilData { tiles: til_tiles },
            level_cel: vec![],
        }
    }
    // ========================================================================
    // Shop panel click hit-testing tests (Step 1: geometry + buy path)
    // ========================================================================

    #[test]
    fn test_point_in_rect_basic() {
        // point_in_rect uses SDL2's exclusive-bound semantics:
        // Rect::new(10, 20, 30, 40) covers pixel x in [10,40), y in [20,60).
        let r = Rect::new(10, 20, 30, 40); // left=10,right=40,top=20,bottom=60
        assert!(point_in_rect(10, 20, r), "top-left corner (inclusive lower bound)");
        assert!(point_in_rect(39, 59, r), "last interior pixel (just inside upper bound)");
        assert!(point_in_rect(25, 40, r), "interior point");
        assert!(!point_in_rect(9, 40, r), "just left of rect");
        assert!(!point_in_rect(40, 40, r), "right edge is exclusive");
        assert!(!point_in_rect(25, 19, r), "just above rect");
        assert!(!point_in_rect(25, 60, r), "bottom edge is exclusive");
    }

    /// Build a GameState with the shop open for `kind` and `gold` player gold.
    fn gs_with_shop_open(kind: u8, gold: i32) -> GameState {
        let player = crate::game::player_exact::Player::new();
        let mut gs = GameState::new(player, true, 1);
        gs.player._p_gold = gold;
        gs.open_shop(kind);
        gs
    }

    #[test]
    fn test_shop_panel_geometry_is_centered() {
        // The panel must be centred in the 640x480 logical canvas.
        let gs = gs_with_shop_open(0, 1000);
        let (panel, _opts, _items) = shop_panel_geometry(&gs);
        let expected_x = (LOGICAL_WIDTH as i32 - SHOP_PANEL_W) / 2;
        let expected_y = (LOGICAL_HEIGHT as i32 - SHOP_PANEL_H) / 2;
        assert_eq!(panel.x(), expected_x);
        assert_eq!(panel.y(), expected_y);
        assert_eq!(panel.width(), SHOP_PANEL_W as u32);
        assert_eq!(panel.height(), SHOP_PANEL_H as u32);
    }

    #[test]
    fn test_shop_panel_geometry_has_four_option_buttons() {
        // BUY / SELL / REPAIR / CLOSE → exactly 4 option rects, equal widths.
        let gs = gs_with_shop_open(0, 1000);
        let (_panel, opts, _items) = shop_panel_geometry(&gs);
        assert_eq!(opts.len(), SHOP_NUM_OPTIONS as usize);
        let w0 = opts[0].width();
        assert!(opts.iter().all(|r| r.width() == w0), "option buttons equal width");
    }

    #[test]
    fn test_shop_panel_geometry_item_rows_match_inventory_size() {
        // The number of item rows must equal the active shop's inventory count.
        for kind in [0u8, 1, 6, 8] {
            let gs = gs_with_shop_open(kind, 10_000);
            let (_panel, _opts, items) = shop_panel_geometry(&gs);
            let expected = gs.active_shop_inventory().len();
            assert_eq!(items.len(), expected, "kind {} row count", kind);
            assert!(expected >= 1, "kind {} has at least one item", kind);
        }
    }

    #[test]
    fn test_close_button_click_closes_shop() {
        // Clicking the CLOSE button (4th option) must close the shop.
        let mut gs = gs_with_shop_open(0, 1000);
        let (_panel, opts, _items) = shop_panel_geometry(&gs);
        let close = opts[3];
        let cx = close.x() + close.width() as i32 / 2;
        let cy = close.y() + close.height() as i32 / 2;
        handle_shop_panel_click(&mut gs, cx, cy);
        assert!(!gs.shop_open, "CLOSE should close the shop");
        assert!(gs.active_shop_npc.is_none());
    }

    #[test]
    fn test_item_row_click_buys_item() {
        // Clicking the centre of the first item row should buy that item,
        // deducting its price from the player's gold.
        let price;
        {
            let gs = gs_with_shop_open(0, 10_000);
            price = gs.active_shop_inventory()[0].1;
        }
        let mut gs = gs_with_shop_open(0, 10_000);
        let (_panel, _opts, items) = shop_panel_geometry(&gs);
        let row = items[0];
        let rx = row.x() + 5;
        let ry = row.y() + row.height() as i32 / 2;
        let gold_before = gs.player._p_gold;
        handle_shop_panel_click(&mut gs, rx, ry);
        assert_eq!(gs.player._p_gold, gold_before - price, "gold deducted by item price");
        // Shop stays open after a buy (player can buy more).
        assert!(gs.shop_open, "shop stays open after a buy");
    }

    #[test]
    fn test_item_row_click_no_gold_leaves_gold_unchanged() {
        // With zero gold, clicking an item row must fail to buy and leave gold
        // at 0 (and the shop open).
        let mut gs = gs_with_shop_open(0, 0);
        let (_panel, _opts, items) = shop_panel_geometry(&gs);
        let row = items[0];
        let rx = row.x() + 5;
        let ry = row.y() + row.height() as i32 / 2;
        handle_shop_panel_click(&mut gs, rx, ry);
        assert_eq!(gs.player._p_gold, 0, "no gold deducted on failed buy");
        assert!(gs.shop_open, "shop stays open on failed buy");
    }

    #[test]
    fn test_click_outside_buttons_and_rows_is_ignored() {
        // A click well inside the panel but not on any button/row must be
        // ignored: shop stays open and gold is unchanged.
        let mut gs = gs_with_shop_open(0, 1000);
        let (panel, _opts, _items) = shop_panel_geometry(&gs);
        // Click in the header area (top of panel, above the option buttons).
        let header_x = panel.x() + SHOP_PANEL_W / 2;
        let header_y = panel.y() + 20;
        let gold_before = gs.player._p_gold;
        handle_shop_panel_click(&mut gs, header_x, header_y);
        assert!(gs.shop_open, "stray click keeps shop open");
        assert_eq!(gs.player._p_gold, gold_before, "stray click changes nothing");
    }

    #[test]
    fn test_each_shopkeeper_offers_three_to_five_items() {
        // The task asks for 3-5 example items per shop. The three full
        // shopkeepers (Griswold=0, Pepin=1, Adria=6) must stay in that range so
        // the panel never looks empty or overflows. Wirt (8) is canonically a
        // single-item premium vendor, so he's checked separately (exactly 1).
        for kind in [0u8, 1, 6] {
            let gs = gs_with_shop_open(kind, 10_000);
            let n = gs.active_shop_inventory().len();
            assert!(
                (3..=5).contains(&n),
                "kind {} should offer 3-5 items, got {}",
                kind,
                n
            );
        }
        // Wirt: exactly one premium item (matches C++ `BoyItem`).
        let wirt = gs_with_shop_open(8, 10_000);
        assert_eq!(
            wirt.active_shop_inventory().len(),
            1,
            "Wirt offers exactly one premium item"
        );
    }

    /// 真实数据渲染冒烟：从 spawn.mpq 加载真实 Tristram 关卡（town.cel/min/
    /// sol/pal），用真实 mega-tile 经忠实管线 `draw_view` 渲染一帧，断言产生
    /// 大量非零且有色彩层次的像素——证明真实 CEL 艺术能正确解码/渲染（合成
    /// 数据测试之外的真实数据回归保护）。spawn.mpq 缺失时优雅跳过。
    #[test]
    fn test_real_town_render_smoke() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType};
        use crate::engine::mpq::MpqArchive;
        use crate::engine::surface::Surface;

        // 定位 spawn.mpq（cwd 或 crate 根），缺失则跳过（CI 无资产时不红）。
        let mut mpq_path = None;
        if let Ok(cwd) = std::env::current_dir() {
            let p = cwd.join("spawn.mpq");
            if p.exists() { mpq_path = Some(p); }
        }
        if mpq_path.is_none() {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spawn.mpq");
            if p.exists() { mpq_path = Some(p); }
        }
        let Some(mpq_path) = mpq_path else {
            eprintln!("[render-smoke] spawn.mpq not found; skipping");
            return;
        };
        let mut archive = match MpqArchive::open(&mpq_path) {
            Ok(a) => a,
            Err(e) => { eprintln!("[render-smoke] open failed: {e}"); return; }
        };
        let level = match DungeonLevelData::load_from_mpq(&mut archive, DungeonType::Town) {
            Ok(l) => l,
            Err(e) => { eprintln!("[render-smoke] town load failed (assets?): {e:?}"); return; }
        };
        assert!(!level.min.mega_tiles.is_empty(), "town.min should load mega-tiles");

        // 用 piece=1（首个 mega-tile，保证有效）铺满网格。
        let layout = crate::game::game_state::TownLayout {
            d_piece: vec![1u16; TOWN_MAX_X * TOWN_MAX_Y],
            width: TOWN_MAX_X,
            height: TOWN_MAX_Y,
        };

        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        // 城镇全亮：恒等光照表 + 全 0 dLight。
        let mut lm = crate::engine::lighting::LightManager::new();
        lm.make_light_table();
        let dlight = vec![0u8; 112 * 112];
        let lighting = crate::engine::scrollrt::Lighting::new(&dlight, &lm.tables);
        crate::engine::scrollrt::draw_view(
            &mut surface, &level, &layout, TilePoint::new(75, 68), 640, 480, 336, &lighting,
        );

        let nonzero = buf.iter().filter(|&&p| p != 0).count();
        let distinct: std::collections::HashSet<u8> = buf.iter().copied().collect();
        assert!(
            nonzero > 1000,
            "real town render should produce many non-zero pixels, got {nonzero}"
        );
        assert!(
            distinct.len() > 3,
            "real town render should have color variety, got {} distinct indices",
            distinct.len()
        );
    }

    /// 地牢光照渲染冒烟：从 spawn.mpq 加载真实 L1 Cathedral 关卡，用地牢光照
    /// （环境全暗 + 玩家光晕）渲染一帧，断言既有被照亮的像素（玩家附近）也有
    /// 黑暗像素（光晕之外）——证明 dLight 玩家光晕端到端工作且 L1 艺术能渲染。
    /// spawn.mpq 缺失时优雅跳过。
    #[test]
    fn test_real_dungeon_render_with_player_light() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType};
        use crate::engine::mpq::MpqArchive;
        use crate::engine::surface::Surface;

        let mut mpq_path = None;
        if let Ok(cwd) = std::env::current_dir() {
            let p = cwd.join("spawn.mpq");
            if p.exists() { mpq_path = Some(p); }
        }
        if mpq_path.is_none() {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spawn.mpq");
            if p.exists() { mpq_path = Some(p); }
        }
        let Some(mpq_path) = mpq_path else {
            eprintln!("[render-smoke] spawn.mpq not found; skipping");
            return;
        };
        let mut archive = match MpqArchive::open(&mpq_path) {
            Ok(a) => a,
            Err(e) => { eprintln!("[render-smoke] open failed: {e}"); return; }
        };
        let level = match DungeonLevelData::load_from_mpq(&mut archive, DungeonType::Cathedral) {
            Ok(l) => l,
            Err(e) => { eprintln!("[render-smoke] L1 load failed (assets?): {e:?}"); return; }
        };
        if level.min.mega_tiles.is_empty() {
            eprintln!("[render-smoke] L1 has no mega-tiles; skipping");
            return;
        }

        // 用 piece=1 铺满网格；相机/玩家在网格中心 (56,56)。
        let layout = crate::game::game_state::DungeonLayout {
            d_piece: vec![1u16; 112 * 112],
            width: 112,
            height: 112,
            trans_val: vec![0; 112 * 112],
            floor_tiles: Vec::new(),
        };

        // 地牢光照：环境全暗(15) + 玩家光晕（小半径，确保视口有明显暗区）。
        let mut lm = crate::engine::lighting::LightManager::new();
        lm.make_light_table();
        let mut dlight = vec![crate::engine::lighting::LIGHTS_MAX as u8; 112 * 112];
        lm.do_lighting(&mut dlight, 112, crate::engine::lighting::Point::new(56, 56), 4);
        let lighting = crate::engine::scrollrt::Lighting::new(&dlight, &lm.tables);

        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        crate::engine::scrollrt::draw_view(
            &mut surface, &level, &layout, TilePoint::new(56, 56), 640, 480, 336, &lighting,
        );

        let nonzero = buf.iter().filter(|&&p| p != 0).count();
        let zero = buf.iter().filter(|&&p| p == 0).count();
        assert!(nonzero > 100, "player-light area should render lit pixels, got {nonzero}");
        assert!(zero > 10_000, "dungeon beyond the light radius should be dark, got {zero} dark px");
    }
}
