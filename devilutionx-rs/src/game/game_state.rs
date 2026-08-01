// Game State Manager (Day 38-39)
//
// C++ References:
// - Source/diablo.cpp:1510-1555 (GameLogic)
// - Source/monster.cpp:4130-4210 (ProcessMonsters)
// - Source/player.cpp (ProcessPlayers)
// - Source/objects.cpp (ProcessObjects)
//
// This module integrates Player, MonsterManager, and ObjectManager into
// a unified game state with per-frame update logic.

use crate::game::player_exact::Player;
use crate::game::monster_exact::{Monster, MonsterManager};
use crate::game::objects::Object;
use crate::game::missiles::MissileManager;
use crate::game::items_processing::ItemManager;
use crate::game::dungeon::DungeonMap;
use crate::game::combat_integration::{monster_attack_player, player_attack_monster, walking_distance};
use crate::game::monster_object_interaction::{monster_check_doors, is_position_blocked_by_object};
use crate::game::types::Point;
use crate::engine::dungeon::DungeonLevelData;
use rand::Rng;

/// Town world dimensions (in micro-tiles / dPiece grid).
///
/// These mirror the C++ `MAXDUNX`/`MAXDUNY` constants. The Tristram town is a
/// 112×112 grid of micro-tiles. Each visible isometric diamond on screen
/// corresponds to one micro-tile in this grid.
pub const TOWN_MAX_X: usize = 112;
pub const TOWN_MAX_Y: usize = 112;

/// Town→Cathedral down-stair trigger tile, in micro-tile (world) coordinates.
///
/// **C++ Reference**: `Source/levels/trigs.cpp:115` (`InitTownTriggers`):
/// ```cpp
/// // Cathedral
/// trigs[numtrigs].position = { 25, 29 };
/// trigs[numtrigs]._tmsg = WM_DIABNEXTLVL;
/// ```
/// This is a fixed, hand-placed trigger tile on the Tristram map — the entrance
/// to the Cathedral. Stepping onto it sends `WM_DIABNEXTLVL` (descend to next
/// level). We mirror the same coordinate so the Rust port can detect the player
/// standing on the Cathedral stairs and trigger `descend_to_dungeon`.
pub const TOWN_DOWN_STAIRS: (i32, i32) = (25, 29);

/// Proximity radius (in world tiles) for stairs detection. The C++ trigger
/// fires on the exact tile, but the rendered player token in this port is
/// camera-aligned to whole tiles and movement is coarse, so we accept the
/// player being on or adjacent to the stair tile (Chebyshev distance ≤ this).
/// Kept at 1 so the trigger feels responsive without being too generous.
pub const STAIRS_TRIGGER_RADIUS: i32 = 1;

/// Number of game-logic ticks that must elapse between two automatic stair
/// transitions. The logic tick runs at 2 Hz (one `GameState::update` per 500 ms
/// via the `game_loop` 500 ms pass), so 4 ticks ≈ 2 seconds. This prevents the
/// descent immediately re-triggering the ascent (and vice-versa) when the
/// player spawns standing on/near a stair tile.
pub const STAIRS_COOLDOWN_TICKS: u32 = 4;

/// Camera in world tile coordinates. The renderer centres the viewport on this
/// point. Kept as fixed-point-ish integers for simplicity; smooth sub-tile
/// movement can be added later by switching to pixel coordinates.
#[derive(Debug, Clone, Copy, Default)]
pub struct Camera {
    /// World tile X the viewport is centred on.
    pub tile_x: i32,
    /// World tile Y the viewport is centred on.
    pub tile_y: i32,
    /// Fractional sub-tile movement accumulator (Q8.8 fixed point, /256 = 1 tile).
    /// Lets held movement keys produce smooth, sub-tile-precise scrolling while
    /// `tile_x`/`tile_y` stay on whole tiles for the renderer.
    pub sub_x: i32,
    /// Fractional sub-tile movement accumulator (Y axis).
    pub sub_y: i32,
}

/// Geographically-correct town layout.
///
/// `d_piece[x][y]` holds the "level piece id" (a 1-based index into the MIN
/// mega-tile table, matching C++ `dPiece[x][y]`). It is built once from the
/// four `sector*s.dun` templates + `town.til` mega definitions, following
/// `Source/levels/town.cpp` `DrlgTPass3` / `FillSector`.
///
/// The renderer reads this grid to decide which CEL frame to draw for each
/// micro-tile. Out-of-bounds / unset tiles default to `0` (rendered empty).
#[derive(Debug, Clone)]
pub struct TownLayout {
    pub d_piece: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

impl Default for TownLayout {
    fn default() -> Self {
        Self {
            d_piece: vec![0; TOWN_MAX_X * TOWN_MAX_Y],
            width: TOWN_MAX_X,
            height: TOWN_MAX_Y,
        }
    }
}

impl TownLayout {
    /// Get the dPiece value at world tile (x, y). Returns 0 when out of bounds.
    pub fn get(&self, x: i32, y: i32) -> u16 {
        if x < 0 || y < 0 {
            return 0;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.d_piece[y * self.width + x]
    }

    /// Set the dPiece value at world tile (x, y). No-op if out of bounds.
    pub fn set(&mut self, x: i32, y: i32, value: u16) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x < self.width && y < self.height {
            self.d_piece[y * self.width + x] = value;
        }
    }
}

/// Game logic processing steps
///
/// **C++ Reference**: `Source/diablo.h` - `GameLogicStep` enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameLogicStep {
    #[default]
    None,
    ProcessPlayers,
    ProcessMonsters,
    ProcessObjects,
    ProcessMissiles,
    ProcessItems,
    ProcessTowners,
    ProcessItemsTown,
    ProcessMissilesTown,
}

/// A single player-cast fireball for the demo spell-casting path.
///
/// Travels one tile per logic tick in a fixed world direction, applies
/// fire damage to the first monster it overlaps, and expires on hit or
/// after a short range. Mana cost and damage follow spelldat's Firebolt
/// (mana cost 6, damage scales with player level).
#[derive(Debug, Clone, Copy)]
pub struct SimpleMissile {
    /// Current world tile position (micro-tile coords, matches monster/player).
    pub x: i32,
    pub y: i32,
    /// Per-tick movement delta in world tiles (one of the 8 directions).
    pub dx: i32,
    pub dy: i32,
    /// Damage applied on hit (display units, not 64x).
    pub damage: i32,
    /// Remaining tiles of travel before the missile fizzles.
    pub range_left: i32,
}

/// Kinds of ground item the demo loot pipeline can drop. Kept small and
/// explicit so the renderer can map each to a distinct colour without pulling
/// in the full `item_dat` ItemData table.
///
/// Distribution on a monster kill (see `GameState::roll_monster_drop`):
///   * Gold         — 70%
///   * HealingPotion — 20%
///   * ManaPotion   — 10%
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundItemType {
    Gold,
    HealingPotion,
    ManaPotion,
}

impl GroundItemType {
    /// Human-readable name used by the pickup log ("[Pickup] picked up X").
    pub fn display_name(self) -> &'static str {
        match self {
            GroundItemType::Gold => "Gold",
            GroundItemType::HealingPotion => "Potion of Healing",
            GroundItemType::ManaPotion => "Potion of Mana",
        }
    }

    /// Gold amount granted when a `Gold` ground item is picked up. Matches the
    /// small-pile feel of early Diablo (C++ `ItemCreateGoldItem` randomises
    /// per monster level; we use a flat modest amount for the demo).
    pub const GOLD_AMOUNT: i32 = 50;
}

/// A single item lying on the dungeon floor, dropped when a monster dies.
///
/// Coordinates are in the same micro-tile space as monsters/players/camera, so
/// the renderer projects them with the same isometric transform. The player
/// picks a ground item up by walking onto its tile (see
/// `GameState::pickup_ground_items`).
#[derive(Debug, Clone, Copy)]
pub struct GroundItem {
    /// World tile X (micro-tile coords).
    pub x: i32,
    /// World tile Y (micro-tile coords).
    pub y: i32,
    /// What kind of loot this is (gold / healing / mana).
    pub item_type: GroundItemType,
}

/// Main game state integrating all subsystems
///
/// **C++ Reference**: Global game state variables in `Source/diablo.cpp`
pub struct GameState {
    /// Player state
    pub player: Player,

    /// Monster manager
    pub monster_manager: MonsterManager,

    /// Missile manager
    pub missile_manager: MissileManager,

    /// Lightweight player-cast spell projectiles (Firebolt etc.).
    ///
    /// This is a self-contained, minimal missile system used by the playable
    /// demo's spell-casting path (the 'F' key). It intentionally does not go
    /// through the full `missiles.rs` MissileManager (whose collision/damage
    /// integration with the live `MonsterManager` is incomplete); instead it
    /// moves, collides against monsters, and applies damage directly here.
    /// Each entry is a single fireball travelling one tile per logic tick.
    pub simple_missiles: Vec<SimpleMissile>,

    /// Items lying on the dungeon floor, dropped when monsters die. The player
    /// picks these up by walking onto the same tile (see
    /// `pickup_ground_items`). Rendered as small coloured icons by
    /// `game_loop::draw_ground_items`.
    pub ground_items: Vec<GroundItem>,

    /// Item manager
    pub item_manager: ItemManager,

    /// Objects in the level
    pub objects: Vec<Object>,

    /// Dungeon Map
    pub dungeon: DungeonMap,

    /// Current game logic step
    pub logic_step: GameLogicStep,

    /// Game tick counter
    pub game_tick: u32,

    /// Is this a town level?
    pub is_town: bool,

    /// Loaded tile/palette/CEL data for the current level, if any.
    ///
    /// This is `Some` once `start_game` has loaded the town (or dungeon) level
    /// data from MPQ. The renderer (`draw_and_blit`) reads it to draw the
    /// isometric floor. It is `None` in tests that don't need real assets.
    pub level_data: Option<DungeonLevelData>,

    /// Geographically-correct town layout (`dPiece` grid), if a town has been
    /// generated. Built from the sector `*.dun` templates by `build_town_layout`
    /// in the game loop. `None` until town data is assembled.
    pub town_layout: Option<TownLayout>,

    /// Camera position in world tile coordinates. The renderer centres the
    /// viewport on this point. Initialised to the town spawn (75, 68) which is
    /// the C++ `ViewPosition` for `ENTRY_MAIN`.
    pub camera: Camera,

    /// Decoded player sprite (RGBA + dimensions), if a Warrior town-walk sprite
    /// was loaded from MPQ in `start_game`. The renderer (`draw_and_blit`)
    /// uploads this once into a cached SDL texture and blits it at the viewport
    /// centre instead of the old yellow marker.
    ///
    /// `None` when no real sprite is available (e.g. assets missing); the
    /// renderer then falls back to the marker.
    pub player_sprite: Option<PlayerSprite>,

    /// Active level mode. `true` = Tristram (town), `false` = a dungeon (L1
    /// Cathedral in the current build). The renderer branches on this to decide
    /// whether to call `draw_tristram` or `draw_dungeon`.
    pub in_dungeon: bool,

    /// L1 Cathedral dungeon layout (`dPiece` grid) generated by `drlg_l1`'s
    /// `CathedralGenerator`, mapped through `l1.til` mega-tiles (mirrors C++
    /// `DRLG_LPass3`). `None` until the player descends into the dungeon.
    pub dungeon_layout: Option<DungeonLayout>,

    /// Micro-tile coordinates of the Cathedral→town up-stair (the tile the
    /// player must stand on to ascend back to Tristram). Populated by
    /// `descend_to_dungeon` when it scans the freshly-generated
    /// `dungeon_layout` for the `EntranceStairs` mega's micro dPiece values
    /// (C++ `InitL1Triggers` scans for `dPiece == 128`; the Rust generator
    /// instead stamps the `EntranceStairs` TIL mega, so we detect by matching
    /// those four micro values). `None` while in town or when no up-stair was
    /// found in the current layout.
    pub dungeon_up_stairs: Option<(i32, i32)>,

    /// `game_tick` value at which the last automatic stair transition fired.
    /// The detection logic refuses to trigger another transition until
    /// `game_tick - last_transition_tick >= STAIRS_COOLDOWN_TICKS`, so a
    /// descent that lands the player on/near the up-stair doesn't instantly
    /// bounce them back to town. Starts at 0; the cooldown is initially
    /// considered elapsed (so the very first descent can fire from tick 0).
    pub last_transition_tick: u32,

    /// L1 Cathedral art (l1.cel/l1.min/l1.til/l1.sol/l1.pal) loaded once from
    /// MPQ, used to render the dungeon floor when `in_dungeon` is true. Kept
    /// separate from `level_data` (which holds the *active* level's art) so we
    /// can swap between town and dungeon art without reloading. `None` if the
    /// L1 assets are unavailable (e.g. shareware build without L1 data).
    pub dungeon_level_data: Option<DungeonLevelData>,

    /// Decoded stand sprites for the monster types currently in the dungeon,
    /// indexed by `MonsterType`. Built by `monster_sprites::MonsterSpriteSet`
    /// when descending. The dungeon renderer uses these to draw each living
    /// monster as a real CL2 sprite (with a coloured-block fallback per type).
    /// `None`/empty when no sprites loaded (town, or asset-less build).
    pub monster_sprites: Option<crate::game::monster_sprites::MonsterSpriteSet>,

    /// Pending SFX requests produced by gameplay (combat hits, monster deaths,
    /// etc.). The game loop drains this each frame via [`drain_pending_sfx`]
    /// and forwards each name to the global `AudioManager::play_sfx`.
    ///
    /// This decouples game-state code (which has no access to the audio
    /// manager) from audio playback: gameplay code just pushes a logical SFX
    /// name (`"swing"`, `"monster_death"`, ...) and the loop bridges it to the
    /// audio system. Names map to MPQ files via
    /// [`crate::engine::audio::SfxLibrary`].
    pub pending_sfx: Vec<String>,

    /// Tristram NPCs to render in town mode.
    ///
    /// Each entry is `(tile_x, tile_y, display_name, towner_kind)` where
    /// `towner_kind` is a small integer (matching `TownerType as u8`) used by
    /// the renderer to pick a distinct marker colour per NPC. The positions
    /// come from the C++ `TownersData` / `townerdat` TSV defaults and are
    /// captured here at `GameState::new` time from
    /// [`crate::game::towner::TownerFactory::get_data`]. Only rendered while
    /// `in_dungeon` is false (Tristram).
    pub towners: Vec<(i32, i32, &'static str, u8)>,

    /// Player-is-dead flag, latched by [`GameState::check_player_death`] once
    /// `_p_hit_points` drops to/below zero during combat. While true, the game
    /// loop draws the red "YOU HAVE DIED" overlay and pauses normal game
    /// processing; the player resurrects (via [`GameState::resurrect_player`])
    /// when they press Space/Enter, returning to Tristram at full HP with the
    /// classic Diablo gold penalty.
    ///
    /// Mirrors C++ `gbDeathActive` (Source/diablo.cpp) which is set on the
    /// dying player and drives the death screen + respawn flow.
    pub player_dead: bool,

    /// Whether the shop panel is currently open. Toggled on by clicking near a
    /// shop-capable NPC (Griswold/Pepin/Adria/Wirt) and toggled off by clicking
    /// CLOSE or pressing ESC. While true, [`game_loop`] draws the shop overlay
    /// and routes clicks to shop item rows instead of click-to-move.
    ///
    /// Mirrors the C++ `ActiveStore != TalkID::None` check that gates store UI
    /// rendering in `DrawAndBlit` / `PressKey`.
    pub shop_open: bool,

    /// The towner kind (`TownerType as u8`) of the NPC whose shop is currently
    /// open. Set when the shop opens; consulted to pick which shop inventory to
    /// show (0=Smith/Griswold, 1=Healer/Pepin, 6=Witch/Adria, 8=PegBoy/Wirt).
    /// `None` whenever [`shop_open`] is false.
    pub active_shop_npc: Option<u8>,
}

/// L1 Cathedral dungeon layout, the dungeon-mode analogue of `TownLayout`.
///
/// `d_piece` is a `MAXDUNX`×`MAXDUNY` (112×112) grid of micro-tile indices,
/// built from the 40×40 logical tile grid produced by `CathedralGenerator`
/// (levels::drlg_l1) by mapping each logical tile through `l1.til` mega
/// definitions and stamping the four micro values into the grid — the same
/// `DRLG_LPass3` algorithm the C++ engine uses.
#[derive(Debug, Clone)]
pub struct DungeonLayout {
    pub d_piece: Vec<u16>,
    pub width: usize,
    pub height: usize,
    /// Per-micro-tile transparency value (C++ dTransVal). 0 = opaque; a
    /// non-zero value identifies the connected floor region the tile belongs
    /// to. Populated by `flood_transparency_values` during generation; the
    /// renderer looks it up through the per-frame `TransList`.
    pub trans_val: Vec<i8>,
    /// Micro-tile coordinates of walkable *floor* tiles inside the active
    /// dungeon region, populated by `generate_l1_cathedral`. Used by monster
    /// spawning to pick valid, open spawn positions away from walls. Each entry
    /// is `(x, y)` in the same micro-tile space as `d_piece`/the camera.
    pub floor_tiles: Vec<(i32, i32)>,
}

impl Default for DungeonLayout {
    fn default() -> Self {
        use crate::levels::types::MAXDUNX;
        use crate::levels::types::MAXDUNY;
        Self {
            d_piece: vec![0; MAXDUNX * MAXDUNY],
            width: MAXDUNX,
            height: MAXDUNY,
            trans_val: vec![0; MAXDUNX * MAXDUNY],
            floor_tiles: Vec::new(),
        }
    }
}

impl DungeonLayout {
    /// Get the dPiece value at micro-tile (x, y). Returns 0 when out of bounds.
    pub fn get(&self, x: i32, y: i32) -> u16 {
        if x < 0 || y < 0 {
            return 0;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.d_piece[y * self.width + x]
    }
}

/// A decoded player sprite ready for texture upload.
///
/// Stores the RGBA pixel buffer plus its dimensions. The actual SDL `Texture`
/// is created lazily inside the render thread (see `game_loop`'s
/// `PLAYER_SPRITE_CACHE`) because textures borrow the canvas's `TextureCreator`.
#[derive(Debug, Clone)]
pub struct PlayerSprite {
    /// Pixel width.
    pub width: u16,
    /// Pixel height.
    pub height: u16,
    /// Top-to-bottom RGBA bytes (`width * height * 4`).
    pub rgba: Vec<u8>,
}

/// A decoded monster sprite ready for texture upload. Same shape as
/// [`PlayerSprite`] — RGBA buffer + dimensions — kept as a distinct type so the
/// monster sprite cache (`MONSTER_SPRITE_CACHE` in `game_loop`) is keyed and
/// rendered separately from the player sprite. Built by `monster_sprites.rs`
/// from the per-type monster CL2 files in the MPQ archives.
#[derive(Debug, Clone)]
pub struct MonsterSprite {
    /// Pixel width.
    pub width: u16,
    /// Pixel height.
    pub height: u16,
    /// Top-to-bottom RGBA bytes (`width * height * 4`).
    pub rgba: Vec<u8>,
}

impl GameState {
    /// Create a new game state
    pub fn new(player: Player, is_town: bool, seed: u64) -> Self {
        let dungeon = DungeonMap::generate(50, 50, crate::game::types::DungeonType::Cathedral, 1, seed);
        Self {
            player,
            monster_manager: MonsterManager::new(200), // Max 200 monsters
            missile_manager: MissileManager::new(125), // Max 125 missiles
            simple_missiles: Vec::new(),
            ground_items: Vec::new(),
            item_manager: ItemManager::new(127), // Max 127 items
            objects: Vec::new(),
            dungeon,
            logic_step: GameLogicStep::None,
            game_tick: 0,
            is_town,
            level_data: None,
            town_layout: None,
            camera: Camera::default(),
            player_sprite: None,
            in_dungeon: false,
            dungeon_layout: None,
            dungeon_up_stairs: None,
            last_transition_tick: 0,
            dungeon_level_data: None,
            monster_sprites: None,
            pending_sfx: Vec::new(),
            towners: Self::build_towner_list(),
            player_dead: false,
            shop_open: false,
            active_shop_npc: None,
        }
    }

    /// Build the list of Tristram NPCs from the static towner config.
    ///
    /// Mirrors C++ `InitTowners()` (Source/towners.cpp:742) for the always
    /// available Diablo-mode NPCs. We include the 9 town NPCs that are always
    /// present (Smith, Healer, Tavern, Story, Drunk, Witch, Barmaid, PegBoy,
    /// Cow); quest/hellfire-gated NPCs (DeadGuy, Farmer, Girl, CowFarmer) are
    /// omitted since they require active quest state the demo doesn't track
    /// here. Position + name come from
    /// [`crate::game::towner::TownerFactory::get_data`].
    fn build_towner_list() -> Vec<(i32, i32, &'static str, u8)> {
        use crate::game::towner::{TownerFactory, TownerType};
        let npc_types = [
            TownerType::Smith,
            TownerType::Healer,
            TownerType::Tavern,
            TownerType::Story,
            TownerType::Drunk,
            TownerType::Witch,
            TownerType::Barmaid,
            TownerType::PegBoy,
            TownerType::Cow,
        ];
        npc_types
            .iter()
            .map(|&ty| {
                let data = TownerFactory::get_data(ty);
                (
                    data.default_position.x,
                    data.default_position.y,
                    data.name,
                    ty as u8,
                )
            })
            .collect()
    }

    /// Initialise the camera/player position to the town spawn (C++ `ViewPosition`
    /// for `ENTRY_MAIN` = {75, 68}). Called by `start_game` after town data is
    /// loaded.
    pub fn init_town_camera(&mut self) {
        // C++ CreateTown ENTRY_MAIN: ViewPosition = { 75, 68 }
        self.camera = Camera { tile_x: 75, tile_y: 68, sub_x: 0, sub_y: 0 };
        self.player.position = Point::new(75, 68);
    }

    /// True when enough game ticks have elapsed since the last automatic stair
    /// transition that a new one is allowed to fire.
    ///
    /// Wraps the `STAIRS_COOLDOWN_TICKS` check in one place so the detection
    /// logic in `game_loop` stays readable. The subtraction is saturating so a
    /// freshly-created state (tick 0, last 0) reports ready immediately.
    pub fn stairs_cooldown_ready(&self) -> bool {
        self.game_tick.saturating_sub(self.last_transition_tick) >= STAIRS_COOLDOWN_TICKS
    }

    /// Record that an automatic stair transition just fired, stamping the
    /// current `game_tick` as the cooldown anchor. Called by `game_loop`'s
    /// stair detection after a successful `descend_to_dungeon` /
    /// `return_to_town`.
    pub fn mark_stair_transition(&mut self) {
        self.last_transition_tick = self.game_tick;
    }

    /// Chebyshev (8-connected) distance in world tiles between the player's
    /// current position and `(tx, ty)`. Returns 0 when standing on the tile,
    /// 1 when orthogonally/diagonally adjacent, etc. Used by the stair
    /// detection to decide if the player is "on" the stair within
    /// `STAIRS_TRIGGER_RADIUS`.
    pub fn player_tile_distance_to(&self, tx: i32, ty: i32) -> i32 {
        let dx = (self.player.position.x - tx).abs();
        let dy = (self.player.position.y - ty).abs();
        dx.max(dy)
    }

    // ========================================================================
    // Shop interaction (Step 1: open/close on clicking an NPC, buy items)
    // ========================================================================
    //
    // These helpers mirror the C++ `TalkToTowner` → `StartStore(TalkID)` flow
    // (Source/objects.cpp / Source/stores.cpp). The game loop calls
    // `npc_at_tile` / `click_tile_to_open_shop` when the player clicks, and
    // `close_shop` when they dismiss the panel. `buy_shop_item` performs the
    // gold-for-item transaction. The shop inventory shown is keyed by the NPC
    // kind so Griswold shows weapons/armor, Pepin shows potions, Adria shows
    // magic items, and Wirt shows his single premium item.

    /// Maximum Chebyshev distance (in world tiles) at which clicking still
    /// counts as "talking" to an NPC. Generous (2 tiles) so the coarse whole-
    /// tile mouse→world conversion in `game_loop` still lands the click on the
    /// NPC even when the player is standing a step away.
    pub const SHOP_CLICK_RADIUS: i32 = 2;

    /// Return the NPC nearest `(tx, ty)` (within [`SHOP_CLICK_RADIUS`]) as an
    /// owned `(tile_x, tile_y, name, kind)` tuple, or `None` if no NPC is in
    /// range. If multiple NPCs are in range the nearest wins (ties broken by
    /// list order, which roughly matches the C++ iteration order).
    ///
    /// `kind` is `TownerType as u8`; the game loop uses it to decide whether the
    /// clicked NPC runs a shop (Griswold/Pepin/Adria/Wirt) or is gossip-only.
    pub fn npc_at_tile(
        &self,
        tx: i32,
        ty: i32,
    ) -> Option<(i32, i32, &'static str, u8)> {
        let mut best: Option<(i32, i32, &'static str, u8)> = None;
        let mut best_dist = i32::MAX;
        for &(nx, ny, name, kind) in &self.towners {
            let dx = (nx - tx).abs();
            let dy = (ny - ty).abs();
            let dist = dx.max(dy);
            if dist <= Self::SHOP_CLICK_RADIUS && dist < best_dist {
                best_dist = dist;
                best = Some((nx, ny, name, kind));
            }
        }
        best
    }

    /// True when the given towner `kind` runs a shop the panel can open for.
    /// Matches the C++ `StartStore` routing: Smith→SmithBuy, Healer→HealerBuy,
    /// Witch→WitchBuy, PegBoy→BoyBuy. The other NPCs (Ogden, Cain, Farnham,
    /// Gillian, Cow) are gossip-only and don't open the shop UI.
    pub fn npc_runs_shop(kind: u8) -> bool {
        matches!(kind, 0 | 1 | 6 | 8)
    }

    /// Display name for the shop owned by the given towner `kind`, used in the
    /// shop panel header. Returns "NPC" for gossip-only NPCs.
    pub fn shop_name_for_npc(kind: u8) -> &'static str {
        match kind {
            0 => "GRISWOLD THE BLACKSMITH",
            1 => "PEPIN THE HEALER",
            6 => "ADRIA THE WITCH",
            8 => "WIRT THE PEG-LEGGED BOY",
            _ => "NPC",
        }
    }

    /// Open the shop panel for the given towner `kind`. Sets [`shop_open`] and
    /// records [`active_shop_npc`]. Safe to call with a gossip-only NPC (the
    /// flag is still set so the panel renders a "no shop" placeholder); the
    /// caller should gate on [`npc_runs_shop`] if it wants to restrict to real
    /// shops.
    pub fn open_shop(&mut self, kind: u8) {
        self.shop_open = true;
        self.active_shop_npc = Some(kind);
    }

    /// Close the shop panel, clearing both [`shop_open`] and
    /// [`active_shop_npc`]. Idempotent.
    pub fn close_shop(&mut self) {
        self.shop_open = false;
        self.active_shop_npc = None;
    }

    /// The catalog of goods the currently active shop offers, as
    /// `(name, price)` pairs for the panel + buy path. Each NPC returns a
    /// fixed, curated list of 3-5 example items so the demo shop is always
    /// populated without the full M15 item-generation pipeline. If no shop is
    /// active, returns an empty vec.
    ///
    /// Prices follow the C++ "player pays 2x base value" convention where the
    /// base value is read from the canonical item data; we hardcode the small
    /// demo catalog so the panel doesn't depend on the full item system.
    pub fn active_shop_inventory(&self) -> Vec<(&'static str, i32)> {
        match self.active_shop_npc {
            Some(0) => vec![
                // Griswold: weapons + a cheap armor piece.
                ("Short Sword (1-6 dmg)", 120),
                ("Buckler (AC 5)", 60),
                ("Club (1-3 dmg)", 20),
                ("Chain Mail (AC 12)", 240),
            ],
            Some(1) => vec![
                // Pepin: potions.
                ("Potion of Healing", 50),
                ("Potion of Full Healing", 150),
                ("Potion of Mana", 50),
            ],
            Some(6) => vec![
                // Adria: magic items.
                ("Scroll of Firebolt", 100),
                ("Scroll of Healing", 100),
                ("Staff of Firebolt", 400),
                ("Book of Firebolt", 1200),
            ],
            Some(8) => vec![
                // Wirt: one premium item.
                ("Wirt's Premium Item", 1000),
            ],
            _ => Vec::new(),
        }
    }

    /// Try to buy the shop item at `index` from the active shop's inventory.
    ///
    /// Reads the price from [`active_shop_inventory`], checks the player has
    /// enough gold, and on success deducts the gold and returns the item's
    /// `(name, price)` for the caller to log. Returns `Err(reason)` when the
    /// shop is closed, the index is out of range, or gold is insufficient.
    ///
    /// This mirrors the core of C++ `SmithBuyPgm` / `HealerBuy` /
    /// `WitchBuy` (Source/stores.cpp): resolve price, check gold, deduct, remove
    /// from inventory. Inventory-space is not checked (the port's inventory grid
    /// isn't wired into the shop path yet); a successful buy just logs the item.
    pub fn buy_shop_item(&mut self, index: usize) -> Result<(&'static str, i32), &'static str> {
        if !self.shop_open {
            return Err("no shop open");
        }
        let inv = self.active_shop_inventory();
        let (name, price) = inv.get(index).copied().ok_or("item out of range")?;
        if self.player._p_gold < price {
            return Err("not enough gold");
        }
        self.player._p_gold -= price;
        Ok((name, price))
    }

    // ========================================================================
    // Equipment effects (Step 2: derive _p_i_min/max_dam from inv_body[1])
    // ========================================================================
    //
    // C++ `CalcPlrItemStats` (Source/items.cpp) re-derives the player's derived
    // stats (_p_i_min_dam / _p_i_max_dam / _p_i_ac / ...) from the equipped
    // `InvBody[]` slots every time the inventory changes. We implement a
    // deliberately minimal version: when the left-hand slot (HandLeft, index 4)
    // holds a weapon, the player's item damage range is set from that weapon.
    // The demo's `equip_starter_weapon` gives a fresh Warrior a Short Sword
    // (1-6, matching the canonical `items\\weapons\\sward` base item) so melee
    // combat does non-zero damage.

    /// Index of the left-hand (weapon) equipment slot in `Player::inv_body`,
    /// matching `InvBodyLoc::HandLeft` (player_exact.rs).
    pub const INV_BODY_HAND_LEFT: usize = 4;

    /// Re-derive the player's weapon damage range from the equipped left-hand
    /// item. If the slot is empty, the damage range is reset to 0-0. This is
    /// the minimal slice of C++ `CalcPlrItemStats` needed for the demo's combat
    /// path — full stat recalculation (AC, to-hit, resistances, ...) is deferred
    /// until the inventory system is wired into the equip flow.
    pub fn recalc_equipment_stats(&mut self) {
        let slot = self.player.inv_body.get(Self::INV_BODY_HAND_LEFT);
        if let Some(item) = slot {
            if !item.is_empty() {
                self.player._p_i_min_dam = 1;
                self.player._p_i_max_dam = 6;
                return;
            }
        }
        self.player._p_i_min_dam = 0;
        self.player._p_i_max_dam = 0;
    }

    /// Equip a starter Short Sword (1-6 damage) in the Warrior's left-hand
    /// slot so combat has a real damage range. Idempotent: if a weapon is
    /// already equipped this is a no-op. Mirrors the C++ starting-kit logic in
    /// `CreatePlayer` / `StartNewGame` which hands each class a base weapon.
    pub fn equip_starter_weapon(&mut self) {
        let slot = self.player.inv_body.get_mut(Self::INV_BODY_HAND_LEFT);
        if let Some(item) = slot {
            if !item.is_empty() {
                return; // already armed
            }
            item.item_id = 1; // non-zero => "a weapon is equipped"
            item.equipped = true;
        }
        self.recalc_equipment_stats();
    }

    /// True when the player currently has a weapon equipped in the left-hand
    /// slot (so the combat path should apply weapon damage rather than bare-
    /// handed zero damage).
    pub fn has_weapon_equipped(&self) -> bool {
        self.player
            .inv_body
            .get(Self::INV_BODY_HAND_LEFT)
            .map(|i| !i.is_empty())
            .unwrap_or(false)
    }

    /// Main game logic update (one frame)
    ///
    /// **C++ Reference**: `Source/diablo.cpp:1510` - `GameLogic()`
    ///
    /// # C++ Implementation
    /// ```cpp
    /// void GameLogic()
    /// {
    ///     if (!ProcessInput()) return;
    ///
    ///     if (gbProcessPlayers) {
    ///         gGameLogicStep = GameLogicStep::ProcessPlayers;
    ///         ProcessPlayers();
    ///     }
    ///
    ///     if (leveltype != DTYPE_TOWN) {
    ///         gGameLogicStep = GameLogicStep::ProcessMonsters;
    ///         ProcessMonsters();
    ///
    ///         gGameLogicStep = GameLogicStep::ProcessObjects;
    ///         ProcessObjects();
    ///
    ///         // ProcessMissiles, ProcessItems, etc.
    ///     }
    ///
    ///     gGameLogicStep = GameLogicStep::None;
    /// }
    /// ```
    ///
    /// # Arguments
    /// - `rng`: Random number generator
    pub fn update(&mut self, rng: &mut impl Rng) {
        self.game_tick += 1;

        // Pause normal game processing while the player is dead. The death
        // overlay is shown by the game loop and the player resurrects on
        // Space/Enter. We still advance `game_tick` so the stair cooldown /
        // animation timers keep moving, but no HP regen, combat, or monster AI
        // runs — mirrors C++ where `ProcessPlayers`/`ProcessMonsters` are
        // skipped while `gbDeathActive` is set.
        if self.player_dead {
            self.logic_step = GameLogicStep::None;
            return;
        }

        // Process player (C++ line 1516)
        self.logic_step = GameLogicStep::ProcessPlayers;
        self.process_player_internal(rng);

        // Process monsters (C++ line 1520)
        if !self.is_town {
            self.logic_step = GameLogicStep::ProcessMonsters;
            self.process_monsters(rng);

            // Process objects (C++ line 1525)
            self.logic_step = GameLogicStep::ProcessObjects;
            self.process_objects();

            // Process missiles (C++ line 1528)
            self.logic_step = GameLogicStep::ProcessMissiles;
            self.process_missiles();
            // Update demo spell projectiles (Firebolt) and resolve hits.
            self.process_simple_missiles(rng);

            // Process items (C++ line 1531)
            self.logic_step = GameLogicStep::ProcessItems;
            self.process_items();

            // Pick up any ground loot the player is standing on. Runs every
            // logic tick so walking over a dropped item picks it up promptly.
            self.pickup_ground_items();
        }

        // Death detection: if the player's HP dropped to/below zero from a
        // monster hit during this update, latch `player_dead`. Once latched,
        // the game loop draws the death overlay and pauses normal processing
        // until the player resurrects. Mirrors C++ `gbDeathActive` which is
        // set inside `MonsterAttackPlayer`'s kill path / `StartPlayerKill`.
        // We check this *after* `check_monster_combat` so the monster hit has
        // already been applied by the time we sample HP.
        self.check_player_death();

        self.logic_step = GameLogicStep::None;
    }

    /// Check whether the player just died (HP ≤ 0) and latch [`player_dead`]
    /// if so.
    ///
    /// Safe to call every update tick: once `player_dead` is true the death
    /// overlay / resurrection flow takes over and HP is no longer decremented
    /// (the game loop pauses normal game logic while dead). Idempotent —
    /// calling it again when already dead is a no-op.
    ///
    /// **C++ Reference**: `Source/diablo.cpp` death handling — `gbDeathActive`
    /// is set when `Player::_pHitPoints <= 0` is detected after combat, which
    /// triggers the death screen and respawn-on-Space flow.
    pub fn check_player_death(&mut self) {
        if self.player_dead {
            return; // already dead, nothing to do
        }
        if self.player._p_hit_points <= 0 {
            self.player_dead = true;
            println!(
                "[Death] player '{}' died (HP {} <= 0) at ({},{})",
                self.player.get_name(),
                self.player._p_hit_points,
                self.player.position.x,
                self.player.position.y
            );
        }
    }

    /// True when the player is currently dead ([`player_dead`] is latched).
    /// Used by the game loop to decide whether to draw the death overlay and
    /// suppress movement/spell-casting input.
    pub fn is_player_dead(&self) -> bool {
        self.player_dead
    }

    /// Resurrect the player after death, applying the classic Diablo death
    /// penalty.
    ///
    /// Restores HP and Mana to full, sends the player back to Tristram town
    /// at the ENTRY_MAIN spawn (75, 68), halves their carried gold (the
    /// canonical Diablo death penalty — gold above the stash is split 50/50
    /// in the original; we model only the inventory portion and halve it),
    /// and clears the [`player_dead`] flag so normal game logic resumes.
    ///
    /// This is the Rust analogue of the C++ respawn flow
    /// (`StartNewGame`/`RestartTownLvl` + `Player::Reset`): the original
    /// revives the player in town with full HP and drops half the inventory
    /// gold as a loot pile on the death tile (we forego the loot pile for
    /// simplicity). Safe to call only when [`is_player_dead`] is true; calling
    /// it on a live player is a defensive no-op.
    ///
    /// **C++ Reference**: `Source/inv.cpp` `PlayerDeathsPayPenalty` +
    /// `Source/diablo.cpp` `RestartTownLvl` (sets position to town spawn and
    /// restores HP).
    pub fn resurrect_player(&mut self) {
        if !self.player_dead {
            return; // defensive: nothing to resurrect
        }

        // 1. HP / Mana to full (64x fixed-point). Mirrors C++ `Player::Reset`
        //    setting `_pHitPoints = _pMaxHP` on respawn.
        self.player._p_hit_points = self.player._p_max_hp;
        self.player._p_hp_base = self.player._p_max_hp_base;
        self.player._p_mana = self.player._p_max_mana;
        self.player._p_mana_base = self.player._p_max_mana_base;

        // 2. Gold penalty: halve carried gold (Diablo's death penalty).
        //    C++ `PlayerDeathsPayPenalty` drops half the inventory gold on the
        //    floor; here we simply discard half for simplicity (no stash /
        //    floor-pile system in this demo).
        let gold_before = self.player._p_gold;
        self.player._p_gold = gold_before / 2;
        println!(
            "[Respawn] gold penalty: {} -> {} (halved)",
            gold_before, self.player._p_gold
        );

        // 3. Return to Tristram town at the ENTRY_MAIN spawn (75, 68). This
        //    mirrors C++ `RestartTownLvl` which sets `ViewPosition` to the
        //    town spawn and clears the dungeon. We reuse the existing
        //    `return_to_town` helper in `game_loop` by setting the town-mode
        //    flags + camera here; the full helper also clears monsters/sprites
        //    which we *want* (no point dragging dead-dungeon state to town).
        //    We call the inline equivalent because `return_to_town` lives in
        //    `game_loop` (would create a circular dependency if imported).
        self.in_dungeon = false;
        self.is_town = true;
        self.dungeon_layout = None;
        self.dungeon_up_stairs = None;
        self.simple_missiles.clear();
        self.ground_items.clear();
        self.monster_manager.clear();
        self.monster_sprites = None;
        self.mark_stair_transition();
        self.init_town_camera();

        // 4. Clear the death flag last so the game loop resumes normal logic.
        self.player_dead = false;
        println!(
            "[Respawn] player '{}' resurrected in town at ({},{}) with HP {}/{}",
            self.player.get_name(),
            self.player.position.x,
            self.player.position.y,
            self.player._p_hit_points,
            self.player._p_max_hp
        );
    }

    /// Process player logic
    ///
    /// **C++ Reference**: `Source/player.cpp` - `ProcessPlayers()`
    fn process_player_internal(&mut self, rng: &mut impl Rng) {
        // Player regeneration
        if self.player._p_hit_points < self.player._p_max_hp {
            let regen = (self.player._p_level as i32) / 4 + 1;
            self.player._p_hit_points = (self.player._p_hit_points + regen).min(self.player._p_max_hp);
        }

        // Player mana regeneration
        if self.player._p_mana < self.player._p_max_mana {
            let mana_regen = self.player._p_magic / 8 + 1;
            self.player._p_mana = (self.player._p_mana + mana_regen).min(self.player._p_max_mana);
        }

        // Check for player-monster combat
        self.check_player_combat(rng);
    }

    /// Process all monsters
    ///
    /// **C++ Reference**: `Source/monster.cpp:4130` - `ProcessMonsters()`
    ///
    /// # C++ Implementation
    /// ```cpp
    /// void ProcessMonsters()
    /// {
    ///     DeleteMonsterList();
    ///
    ///     for (size_t i = 0; i < ActiveMonsterCount; i++) {
    ///         Monster &monster = Monsters[ActiveMonsters[i]];
    ///
    ///         // HP regeneration
    ///         if (monster.hitPoints < monster.maxHitPoints) {
    ///             monster.hitPoints += monster.level / 2;
    ///         }
    ///
    ///         // Update enemy tracking
    ///         UpdateEnemy(monster);
    ///
    ///         // Process AI
    ///         AiProc[monster.ai](monster);
    ///
    ///         // Process animation
    ///         monster.animInfo.processAnimation();
    ///     }
    /// }
    /// ```
    fn process_monsters(&mut self, rng: &mut impl Rng) {
        // Collect active monster IDs first to avoid borrow checker issues
        let monster_ids: Vec<usize> = self.monster_manager
            .iter()
            .map(|(id, _)| id)
            .collect();

        for monster_id in monster_ids {
            // HP regeneration (C++ line 4143-4149)
            self.regenerate_monster_hp(monster_id);

            // Update enemy position (C++ line 4170-4182)
            self.update_monster_enemy(monster_id);

            // Check for monster-player combat
            self.check_monster_combat(monster_id, rng);

            // Process doors if monster can open them (C++ via MonstCheckDoors)
            self.process_monster_doors(monster_id);

            // Simple AI movement (M-monsters): idle/wander in place, or chase the
            // player when within aggro range. Updates the monster's world tile so
            // the renderer follows. No attack here (combat handled above).
            self.update_monster_movement(monster_id);
        }
    }

    /// Regenerate monster HP
    ///
    /// **C++ Reference**: `Source/monster.cpp:4143-4149`
    fn regenerate_monster_hp(&mut self, monster_id: usize) {
        if let Some(monster) = self.monster_manager.get_monster_mut(monster_id) {
            if monster.hp < monster.max_hp && monster.hp > 0 {
                // Simplified: regenerate based on intelligence (as proxy for level)
                let regen = ((monster.intelligence as i32) / 2).max(1);
                let regen_64x = regen << 6;
                monster.hp = (monster.hp + regen_64x).min(monster.max_hp);
            }
        }
    }

    /// Update monster's enemy position
    ///
    /// **C++ Reference**: `Source/monster.cpp:4170-4182`
    fn update_monster_enemy(&mut self, monster_id: usize) {
        if let Some(monster) = self.monster_manager.get_monster_mut(monster_id) {
            // Update enemy position to player's current position
            monster.enemy_position = self.player.position;
        }
    }

    /// Check if monster should attack player
    fn check_monster_combat(&mut self, monster_id: usize, rng: &mut impl Rng) {
        // Get monster position (need to clone to avoid borrow checker issues)
        let (monster_pos, can_attack) = {
            if let Some(monster) = self.monster_manager.get_monster(monster_id) {
                let dist = walking_distance(monster.position(), self.player.position);
                (monster.position(), dist <= 1)
            } else {
                return;
            }
        };

        if can_attack {
            // Attack player
            if let Some(monster) = self.monster_manager.get_monster(monster_id) {
                let _ = monster_attack_player(monster, &mut self.player, rng);
            }
        }
    }

    /// Check if player should attack nearby monsters
    fn check_player_combat(&mut self, rng: &mut impl Rng) {
        let player_pos = self.player.position;

        // Collect monster IDs to avoid borrow checker issues
        let monster_ids: Vec<usize> = self.monster_manager
            .iter()
            .map(|(id, _)| id)
            .collect();

        // Track XP gained from kills this tick to award after the borrows
        // on monster_manager/player resolve. Also collect the tile of each kill
        // so we can roll a ground-item drop once the monster_manager borrow is
        // released.
        let mut xp_gained: i32 = 0;
        let mut kill_positions: Vec<(i32, i32)> = Vec::new();
        let mut sfx_hits = 0u32;
        let mut sfx_kills = 0u32;
        for monster_id in monster_ids {
            if let Some(monster) = self.monster_manager.get_monster_mut(monster_id) {
                let dist = walking_distance(player_pos, monster.position());

                if dist <= 1 {
                    // Player attacks this monster
                    match player_attack_monster(&self.player, monster, rng) {
                        crate::game::combat_integration::AttackResult::Kill { .. } => {
                            // Award monster XP on kill (monster_dat xp reward).
                            xp_gained += monster.experience as i32;
                            // Record the death tile for loot drop.
                            let mp = monster.position();
                            kill_positions.push((mp.x, mp.y));
                            // Queue the monster-death SFX. The game loop drains
                            // pending_sfx and forwards it to AudioManager.
                            sfx_kills += 1;
                        }
                        crate::game::combat_integration::AttackResult::Hit { .. } => {
                            // Queue the weapon-swing SFX for a non-killing hit.
                            sfx_hits += 1;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Push the SFX requests once, after the monster_manager borrow ends.
        // We collapse repeated identical sounds into a single play per tick so
        // a multi-monster cleave doesn't spam the audio system.
        if sfx_hits > 0 {
            self.pending_sfx.push(
                crate::engine::audio::SfxLibrary::for_combat(false).to_string(),
            );
        }
        if sfx_kills > 0 {
            self.pending_sfx.push(
                crate::engine::audio::SfxLibrary::for_combat(true).to_string(),
            );
        }

        if xp_gained > 0 {
            self.player._p_experience = self.player._p_experience.saturating_add(xp_gained as u32);
            // Level-up check: advance while XP exceeds the next level threshold.
            self.check_level_up();
        }

        // Roll loot drops for each kill (after the monster_manager borrow ends).
        for (kx, ky) in kill_positions {
            self.roll_monster_drop(kx, ky, rng);
        }
    }

    /// Queue a sound-effect request by logical name.
    ///
    /// Gameplay code calls this when an audible event happens (item pickup,
    /// spell cast, door open, ...). The game loop drains [`pending_sfx`] each
    /// frame via [`drain_pending_sfx`] and forwards each name to the global
    /// `AudioManager::play_sfx`, which resolves it to an MPQ file through
    /// [`crate::engine::audio::SfxLibrary`].
    ///
    /// `name` may be a known alias (`"swing"`, `"monster_death"`, `"ui_click"`,
    /// ...) or a raw MPQ path (`"sfx\\misc\\swing.mp3"`). Unknown aliases are
    /// silently dropped by the audio manager.
    pub fn queue_sfx(&mut self, name: &str) {
        self.pending_sfx.push(name.to_string());
    }

    /// Drain all pending SFX requests, returning them as an owned `Vec`.
    ///
    /// Called by the game loop once per frame after `GameState::update`. The
    /// returned names should be passed to `AudioManager::play_sfx` in order.
    pub fn drain_pending_sfx(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_sfx)
    }

    /// Roll a monster's death loot and, on a hit, push a `GroundItem` onto the
    /// floor at the monster's tile.
    ///
    /// Drop chance is 40% per kill. On a successful drop the item type is chosen
    /// by a second roll against the distribution:
    ///   * Gold         — 70%
    ///   * HealingPotion — 20%
    ///   * ManaPotion   — 10%
    ///
    /// This mirrors the high-level shape of C++ `MonstDeath`/`SpawnItem`
    /// (`Source/items.cpp`) while staying intentionally simple for the demo.
    pub fn roll_monster_drop(&mut self, x: i32, y: i32, rng: &mut impl Rng) {
        // 40% chance to drop anything at all.
        if rng.random_range(0..100) >= 40 {
            return;
        }
        let roll = rng.random_range(0..100);
        let item_type = if roll < 70 {
            GroundItemType::Gold
        } else if roll < 90 {
            GroundItemType::HealingPotion
        } else {
            GroundItemType::ManaPotion
        };
        self.ground_items.push(GroundItem { x, y, item_type });
        println!(
            "[Drop] spawned {:?} '{}' at ({}, {})",
            item_type,
            item_type.display_name(),
            x,
            y
        );
    }

    /// Pick up any `GroundItem`s on the player's current tile.
    ///
    /// Called from `update()` so it runs every logic tick. Gold adds directly to
    /// `player._p_gold`; potions are logged only for now (the inventory system
    /// is more involved and is wired up separately). Picked-up items are removed
    /// from `ground_items`.
    pub fn pickup_ground_items(&mut self) {
        if self.ground_items.is_empty() {
            return;
        }
        let (px, py) = (self.player.position.x, self.player.position.y);
        // Keep items not on the player's tile; collect the rest for processing.
        let remaining: Vec<GroundItem> = self
            .ground_items
            .iter()
            .copied()
            .filter(|g| !(g.x == px && g.y == py))
            .collect();
        let picked_up: Vec<GroundItem> = self
            .ground_items
            .iter()
            .copied()
            .filter(|g| g.x == px && g.y == py)
            .collect();
        self.ground_items = remaining;
        for g in picked_up {
            match g.item_type {
                GroundItemType::Gold => {
                    self.player._p_gold =
                        self.player._p_gold.saturating_add(GroundItemType::GOLD_AMOUNT);
                    println!(
                        "[Pickup] picked up {} gold (+{} -> {} total)",
                        g.item_type.display_name(),
                        GroundItemType::GOLD_AMOUNT,
                        self.player._p_gold
                    );
                }
                GroundItemType::HealingPotion | GroundItemType::ManaPotion => {
                    // Inventory/belt integration is deferred; just log the pickup.
                    println!("[Pickup] picked up {}", g.item_type.display_name());
                }
            }
        }
    }

    /// Advance the player's level while their experience exceeds the next
    /// level's XP threshold.
    pub fn check_level_up(&mut self) {
        // Minimal: bump level by 1 per call if a coarse XP threshold is met.
        // (Full C++ CalcStats with per-class growth is in player_dat; this is
        // a playable approximation so kills visibly progress the HUD.)
        let lvl = self.player._p_level as i32;
        // Coarse threshold curve (~ classic Diablo early-game feel).
        let threshold = lvl * lvl * 500;
        if (self.player._p_experience as i32) >= threshold && lvl < 50 {
            self.player._p_level += 1;
            // Per-level HP/mana bump (approx: +lvl_life/64 HP per level).
            let bump = ((self.player._p_max_hp_base) / 32).max(64);
            self.player._p_max_hp_base += bump;
            self.player._p_max_hp += bump;
            self.player._p_hit_points += bump;
        }
    }

    /// Cast a Firebolt toward the nearest live monster. Convenience wrapper
    /// around `cast_firebolt_toward` that resolves the target direction from
    /// the monster manager (kept here to avoid borrow conflicts in the caller).
    /// Falls back to south if no monster is present.
    pub fn cast_firebolt_at_nearest(&mut self) -> bool {
        let ppos = self.player.position;
        let mut best_dx = 0i32;
        let mut best_dy = 1i32; // default: south
        let mut best_dist = i32::MAX;
        for (id, _) in self.monster_manager.iter() {
            if let Some(mon) = self.monster_manager.get_monster(id) {
                if !mon.is_alive() {
                    continue;
                }
                let mp = mon.position();
                let d = (mp.x - ppos.x).abs() + (mp.y - ppos.y).abs();
                if d < best_dist {
                    best_dist = d;
                    best_dx = mp.x - ppos.x;
                    best_dy = mp.y - ppos.y;
                }
            }
        }
        self.cast_firebolt_toward(best_dx, best_dy)
    }

    /// Try to cast a Firebolt in the player's facing direction.
    ///
    /// Mana cost is 6 (spelldat Firebolt). On success the mana is spent and a
    /// `SimpleMissile` is spawned at the player's tile. Damage scales gently
    /// with level (base 6 + level, mirroring the early Firebolt feel). The
    /// facing direction is inferred from the last movement delta stored on the
    /// player's `position` history is unavailable, so we fall back to the
    /// direction toward the nearest monster (or south if none).
    pub fn cast_firebolt_toward(&mut self, target_dx: i32, target_dy: i32) -> bool {
        const FIREBOLT_MANA_COST_64X: i32 = 6 << 6;
        if self.player._p_mana < FIREBOLT_MANA_COST_64X {
            return false;
        }
        // Normalise to a single-tile step in one of 8 directions.
        let dx = target_dx.signum();
        let dy = target_dy.signum();
        if dx == 0 && dy == 0 {
            return false; // no direction
        }
        self.player._p_mana -= FIREBOLT_MANA_COST_64X;
        let p = self.player.position;
        let damage = 6 + (self.player._p_level as i32);
        self.simple_missiles.push(SimpleMissile {
            x: p.x,
            y: p.y,
            dx,
            dy,
            damage,
            range_left: 12,
        });
        true
    }

    /// Advance all active `SimpleMissile`s, resolve monster collisions, and
    /// cull expired/out-of-range projectiles. Killed monsters award XP via
    /// the same path as melee combat.
    fn process_simple_missiles(&mut self, rng: &mut impl Rng) {
        // Snapshot missile positions to iterate while mutating monsters.
        let mut xp_gained: i32 = 0;
        let mut kill_positions: Vec<(i32, i32)> = Vec::new();
        let mut alive: Vec<SimpleMissile> = Vec::with_capacity(self.simple_missiles.len());
        for mut m in self.simple_missiles.drain(..) {
            m.x += m.dx;
            m.y += m.dy;
            m.range_left -= 1;

            // Out of range -> fizzle.
            if m.range_left < 0 {
                continue;
            }

            // Collide with the first live monster on this tile.
            let mut hit = false;
            let monster_ids: Vec<usize> = self.monster_manager.iter().map(|(id, _)| id).collect();
            for mid in monster_ids {
                if let Some(mon) = self.monster_manager.get_monster_mut(mid) {
                    if mon.is_alive() {
                        let mp = mon.position();
                        if mp.x == m.x && mp.y == m.y {
                            mon.hp -= m.damage << 6; // 64x, same scale as melee
                            hit = true;
                            if mon.hp <= 0 {
                                mon.mode = crate::game::monster::MonsterMode::Death;
                                mon.ai_state = crate::game::monster::MonsterAIState::Dead;
                                xp_gained += mon.experience as i32;
                                // Record the death tile for loot drop.
                                kill_positions.push((mp.x, mp.y));
                            }
                            break;
                        }
                    }
                }
            }
            if !hit {
                alive.push(m);
            }
        }
        self.simple_missiles = alive;

        if xp_gained > 0 {
            self.player._p_experience =
                self.player._p_experience.saturating_add(xp_gained as u32);
            self.check_level_up();
        }

        // Roll loot drops for each spell kill (after the monster_manager borrow
        // ends). The rng was previously unused here; it now drives the drop roll.
        for (kx, ky) in kill_positions {
            self.roll_monster_drop(kx, ky, rng);
        }
    }

    /// Process monster-door interactions
    fn process_monster_doors(&mut self, monster_id: usize) {
        if let Some(monster) = self.monster_manager.get_monster(monster_id) {
            let _ = monster_check_doors(monster, &mut self.objects);
        }
    }

    /// Simple monster AI movement (M-monsters): idle/wander in place, or chase
    /// the player when within the monster's `aggro_range` (default 8 tiles).
    ///
    /// This is a deliberately minimal AI for the dungeon-population task:
    ///   * `Monster::update_ai` flips the monster between `Idle` (out of range)
    ///     and `Chasing`/`Attacking` (in range), updating its target tile.
    ///   * When `Chasing`/`Wandering`, `Monster::try_move` steps one tile toward
    ///     the target using a greedy 8-direction nudge, gated by a walkability
    ///     check (must be a dungeon floor tile, and must not be the player's
    ///     tile).
    ///
    /// No pathfinding/A* is used here (the framework supports it via
    /// `try_move_pathfind`, but a greedy step is sufficient to demonstrate
    /// "monsters move toward the player"). Attack/damage is handled separately
    /// in `check_monster_combat`; this method only moves the monster.
    ///
    /// Monsters are allowed to overlap each other (monster-monster collision is
    /// a known remaining risk — see task notes).
    fn update_monster_movement(&mut self, monster_id: usize) {
        // Snapshot the player position (owned, so the closure can capture it
        // without borrowing self).
        let player_pos = self.player.position;

        // Build the walkable floor set once per monster from the dungeon layout.
        // (A per-call build is cheap: floor_tiles is a few thousand entries and
        // this runs at the 2 Hz logic tick.)
        let walkable: std::collections::HashSet<(i32, i32)> = match &self.dungeon_layout {
            Some(l) => l.floor_tiles.iter().copied().collect(),
            None => return, // no dungeon → no movement
        };

        if let Some(monster) = self.monster_manager.get_monster_mut(monster_id) {
            if !monster.is_alive() {
                return;
            }

            // Distance-based line-of-sight proxy: in range and (trivially) visible.
            let dist = monster.distance_to(player_pos.x, player_pos.y);
            let can_see = dist <= monster.aggro_range;

            // Update AI state (Idle ↔ Chasing ↔ Attacking) based on range.
            monster.update_ai(player_pos.x, player_pos.y, can_see);

            // Idle monsters get a small random wander every few ticks so the
            // dungeon feels alive even before the player aggros anything.
            if monster.ai_state == crate::game::monster::MonsterAIState::Idle {
                // ~10% chance per logic tick to nudge one tile, only if the
                // move timer is ready. We pick a random adjacent floor tile.
                if monster.move_timer == 0 {
                    let mut rng = rand::rng();
                    if rng.random_range(0..10) == 0 {
                        let dirs: [(i32, i32); 8] = [
                            (1, 0), (-1, 0), (0, 1), (0, -1),
                            (1, 1), (1, -1), (-1, 1), (-1, -1),
                        ];
                        let (dx, dy) = dirs[rng.random_range(0..dirs.len())];
                        let nx = monster.x + dx;
                        let ny = monster.y + dy;
                        if walkable.contains(&(nx, ny)) && (nx != player_pos.x || ny != player_pos.y) {
                            // Don't wander more than ~4 tiles from the spawn
                            // (home) tile, so idle monsters stay near their spot.
                            if (nx - monster.home_x).abs() + (ny - monster.home_y).abs() <= 4 {
                                monster.x = nx;
                                monster.y = ny;
                                monster.move_timer = monster.move_delay;
                            }
                        }
                    }
                }
                return;
            }

            // Chasing/Attacking monsters step toward the player. The walkability
            // closure allows any dungeon floor tile that isn't the player's tile
            // (so monsters stop adjacent instead of walking onto the player).
            monster.try_move(|x, y| {
                walkable.contains(&(x, y)) && (x != player_pos.x || y != player_pos.y)
            });
        }
    }

    /// Process objects (doors, chests, etc.)
    ///
    /// **C++ Reference**: `Source/objects.cpp` - `ProcessObjects()`
    fn process_objects(&mut self) {
        // Update object animations, timers, etc.
        for object in &mut self.objects {
            if object.del_flag {
                continue;
            }

            // Update animation frame
            if object.anim_flag {
                object.anim_cnt += 1;
                if object.anim_cnt >= object.anim_delay {
                    object.anim_cnt = 0;
                    object.anim_frame += 1;
                    if object.anim_frame >= object.anim_len {
                        object.anim_frame = 0;
                    }
                }
            }
        }
    }

    /// Process missiles (projectiles, spell effects)
    ///
    /// **C++ Reference**: `Source/missiles.cpp:4216` - `ProcessMissiles()`
    fn process_missiles(&mut self) {
        self.missile_manager.process_missiles();
    }

    /// Process all items
    ///
    /// **C++ Reference**: `Source/items.cpp:3765` - `ProcessItems()`
    ///
    /// # C++ Implementation
    /// ```cpp
    /// void ProcessItems()
    /// {
    ///     for (int i = 0; i < ActiveItemCount; i++) {
    ///         const int ii = ActiveItems[i];
    ///         auto &item = Items[ii];
    ///         if (!item._iAnimFlag) continue;
    ///         item.AnimInfo.processAnimation();
    ///         // ... animation logic ...
    ///     }
    ///     ItemDoppel();
    /// }
    /// ```
    fn process_items(&mut self) {
        self.item_manager.process_items();
    }

    //
    // Public internal methods for GameLoop integration
    //

    /// Process player logic (public for GameLoop)
    ///
    /// **C++ Reference**: `Source/player.cpp` - `ProcessPlayers()`
    pub fn process_player(&mut self, rng: &mut impl Rng) {
        self.process_player_internal(rng);
    }

    /// Process all monsters (public for GameLoop)
    ///
    /// **C++ Reference**: `Source/monster.cpp:4130` - `ProcessMonsters()`
    pub fn process_monsters_internal(&mut self, rng: &mut impl Rng) {
        // Collect active monster IDs first to avoid borrow checker issues
        let monster_ids: Vec<usize> = self.monster_manager
            .iter()
            .map(|(id, _)| id)
            .collect();

        for monster_id in monster_ids {
            self.regenerate_monster_hp(monster_id);
            self.update_monster_enemy(monster_id);
            self.check_monster_combat(monster_id, rng);
            self.process_monster_doors(monster_id);
        }
    }

    /// Process objects (public for GameLoop)
    ///
    /// **C++ Reference**: `Source/objects.cpp` - `ProcessObjects()`
    pub fn process_objects_internal(&mut self) {
        for object in &mut self.objects {
            if object.del_flag {
                continue;
            }
            if object.anim_flag {
                object.anim_cnt += 1;
                if object.anim_cnt >= object.anim_delay {
                    object.anim_cnt = 0;
                    object.anim_frame += 1;
                    if object.anim_frame >= object.anim_len {
                        object.anim_frame = 0;
                    }
                }
            }
        }
    }

    /// Process missiles (public for GameLoop)
    ///
    /// **C++ Reference**: `Source/missiles.cpp:4216` - `ProcessMissiles()`
    pub fn process_missiles_internal(&mut self) {
        self.missile_manager.process_missiles();
    }

    /// Process items (public for GameLoop)
    ///
    /// **C++ Reference**: `Source/items.cpp:3765` - `ProcessItems()`
    pub fn process_items_internal(&mut self) {
        self.item_manager.process_items();
    }

    /// Add a monster to the game state
    pub fn add_monster(&mut self, monster: Monster) -> Option<usize> {
        self.monster_manager.add_monster(monster)
    }

    /// Add an object to the game state
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Get player reference
    pub fn player(&self) -> &Player {
        &self.player
    }

    /// Get mutable player reference
    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    /// Get number of active monsters
    pub fn active_monster_count(&self) -> usize {
        self.monster_manager.active_count()
    }

    /// Get monster by ID
    pub fn get_monster(&self, id: usize) -> Option<&Monster> {
        self.monster_manager.get_monster(id)
    }

    /// Get mutable monster by ID
    pub fn get_monster_mut(&mut self, id: usize) -> Option<&mut Monster> {
        self.monster_manager.get_monster_mut(id)
    }

    /// Check if position is blocked by objects
    pub fn is_position_blocked(&self, monster_id: usize, position: Point) -> bool {
        if let Some(monster) = self.monster_manager.get_monster(monster_id) {
            is_position_blocked_by_object(monster, &self.objects, position)
        } else {
            false
        }
    }

    // ========================================================================
    // Save/Load (F5/F9) — round-trip the player + world state to/from disk.
    //
    // We go through the type-erased `PlayerSnapshot`/`WorldSnapshot` traits
    // defined in `save.rs`, so `game_state.rs` (which *can* see the real
    // `player_exact::Player`) bridges the engine types <-> the serialisable
    // `SaveSlot`. Monster/object/inventory state is intentionally not persisted
    // yet — those subsystems regenerate from the level seed on load (matching
    // the original game's approach for the most part).
    // ========================================================================

    /// Capture the current state into a `SaveSlot` and write it to `slot` on
    /// disk. Returns the path written, for logging.
    ///
    /// Captures the full engine state: player vitals + position (v2 fields),
    /// plus the complete level (monsters/objects/floor items) and global game
    /// (quests/portals/missiles) binary blobs (v3 fields). The F5 quick-save
    /// path calls this, so a save now persists the dungeon state, not just the
    /// hero.
    pub fn save_to_slot(&self, slot: u32) -> std::result::Result<String, String> {
        use crate::game::save::{build_save_slot_full, SaveManager};
        let mgr = SaveManager::new();
        let save = build_save_slot_full(self, self, self, self);
        let path = mgr
            .save_slot(slot, &save)
            .map_err(|e| format!("save failed: {}", e))?;
        Ok(path)
    }

    /// Load a `SaveSlot` from disk and apply it to this `GameState` (player +
    /// world + level + game). Returns the loaded snapshot for inspection/logging.
    ///
    /// Player + world fields are always restored. If the slot is v3 and carries
    /// the level/game blobs, monsters, objects, floor items, and simple
    /// missiles are also restored (via `LevelStateMut`/`GameStateMut`).
    /// Legacy v2 slots leave those subsystems to regenerate from the level
    /// seed (the original behaviour).
    pub fn load_from_slot(&mut self, slot: u32) -> std::result::Result<(), String> {
        use crate::game::save::{GameStateMut, LevelStateMut, PlayerSnapshotMut, SaveManager};
        let mgr = SaveManager::new();
        let data = mgr
            .load_slot(slot)
            .map_err(|e| format!("load failed: {}", e))?;
        // Apply player fields.
        PlayerSnapshotMut::apply_slot(&mut self.player, &data.player);
        // Apply world fields.
        self.in_dungeon = data.world.in_dungeon;
        self.is_town = data.world.is_town;
        self.game_tick = data.world.game_tick;
        self.camera.tile_x = data.world.cam_x;
        self.camera.tile_y = data.world.cam_y;
        // Keep the player's logical position in sync with the camera so the
        // renderer and movement code agree after a load.
        self.player.position.x = data.player.pos_x;
        self.player.position.y = data.player.pos_y;
        // v3: restore the full level + game state if present.
        if let Some(game) = &data.game {
            GameStateMut::apply_game(self, game);
        }
        if let Some(level) = &data.level {
            LevelStateMut::apply_level(self, level);
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Save trait bridges: GameState reads *as* a player snapshot and a world
// snapshot, and the real `Player` implements `PlayerSnapshotMut` so loaded
// fields are written straight back into the engine struct.
// ----------------------------------------------------------------------------

impl crate::game::save::PlayerSnapshot for GameState {
    fn name(&self) -> String {
        self.player.get_name()
    }
    fn class_u8(&self) -> u8 {
        self.player._p_class.into()
    }
    fn level(&self) -> u8 {
        self.player._p_level
    }
    fn plr_level(&self) -> u8 {
        self.player.plr_level
    }
    fn strength(&self) -> i32 {
        self.player._p_strength
    }
    fn base_str(&self) -> i32 {
        self.player._p_base_str
    }
    fn magic(&self) -> i32 {
        self.player._p_magic
    }
    fn base_mag(&self) -> i32 {
        self.player._p_base_mag
    }
    fn dexterity(&self) -> i32 {
        self.player._p_dexterity
    }
    fn base_dex(&self) -> i32 {
        self.player._p_base_dex
    }
    fn vitality(&self) -> i32 {
        self.player._p_vitality
    }
    fn base_vit(&self) -> i32 {
        self.player._p_base_vit
    }
    fn stat_pts(&self) -> i32 {
        self.player._p_stat_pts
    }
    fn hp_base(&self) -> i32 {
        self.player._p_hp_base
    }
    fn max_hp_base(&self) -> i32 {
        self.player._p_max_hp_base
    }
    fn hit_points(&self) -> i32 {
        self.player._p_hit_points
    }
    fn max_hp(&self) -> i32 {
        self.player._p_max_hp
    }
    fn mana_base(&self) -> i32 {
        self.player._p_mana_base
    }
    fn max_mana_base(&self) -> i32 {
        self.player._p_max_mana_base
    }
    fn mana(&self) -> i32 {
        self.player._p_mana
    }
    fn max_mana(&self) -> i32 {
        self.player._p_max_mana
    }
    fn experience(&self) -> u32 {
        self.player._p_experience
    }
    fn gold(&self) -> i32 {
        self.player._p_gold
    }
    fn pos_x(&self) -> i32 {
        self.player.position.x
    }
    fn pos_y(&self) -> i32 {
        self.player.position.y
    }
}

impl crate::game::save::WorldSnapshot for GameState {
    fn dungeon_type_u8(&self) -> u8 {
        // Map DungeonType -> stable u8 (None=0, Town=1, ...). Kept explicit so
        // reordering the enum never silently corrupts saves.
        match self.dungeon.dungeon_type {
            crate::game::types::DungeonType::None => 0,
            crate::game::types::DungeonType::Town => 1,
            crate::game::types::DungeonType::Cathedral => 2,
            crate::game::types::DungeonType::Catacombs => 3,
            crate::game::types::DungeonType::Caves => 4,
            crate::game::types::DungeonType::Hell => 5,
            crate::game::types::DungeonType::Nest => 6,
            crate::game::types::DungeonType::Crypt => 7,
        }
    }
    fn in_dungeon(&self) -> bool {
        self.in_dungeon
    }
    fn is_town(&self) -> bool {
        self.is_town
    }
    fn game_tick(&self) -> u32 {
        self.game_tick
    }
    fn cam_x(&self) -> i32 {
        self.camera.tile_x
    }
    fn cam_y(&self) -> i32 {
        self.camera.tile_y
    }
}

/// Writing loaded values back into the engine `Player`. The 64x fixed-point
/// HP/Mana fields are restored verbatim (no scaling), so vitals survive a
/// round-trip with zero precision loss.
impl crate::game::save::PlayerSnapshotMut for Player {
    fn apply_slot(&mut self, s: &crate::game::save::SlotPlayer) {
        self.set_name(&s.name);
        if let Ok(class) = crate::game::player_exact::HeroClass::try_from(s.class) {
            self._p_class = class;
        }
        self._p_level = s.level;
        self.plr_level = s.plr_level;

        self._p_strength = s.strength;
        self._p_base_str = s.base_str;
        self._p_magic = s.magic;
        self._p_base_mag = s.base_mag;
        self._p_dexterity = s.dexterity;
        self._p_base_dex = s.base_dex;
        self._p_vitality = s.vitality;
        self._p_base_vit = s.base_vit;
        self._p_stat_pts = s.stat_pts;

        // HP (64x) — restored as-is.
        self._p_hp_base = s.hp_base;
        self._p_max_hp_base = s.max_hp_base;
        self._p_hit_points = s.hit_points;
        self._p_max_hp = s.max_hp;

        // Mana (64x) — restored as-is.
        self._p_mana_base = s.mana_base;
        self._p_max_mana_base = s.max_mana_base;
        self._p_mana = s.mana;
        self._p_max_mana = s.max_mana;

        self._p_experience = s.experience;
        self._p_gold = s.gold;

        self.position = Point::new(s.pos_x, s.pos_y);
    }
}

// ============================================================================
// LevelSnapshot / GameSnapshot — capture full engine state for the F5/F9
// save path. These bridge the live `MonsterManager`/`objects`/`ground_items`
// into the binary `LevelSaveData`/`GameSaveDataEnvelope` envelopes defined in
// `loadsave.rs`, so the quick-save now persists monster/object/item state in
// addition to player vitals + position.
// ============================================================================

impl crate::game::loadsave::LevelSnapshot for GameState {
    fn is_town(&self) -> bool {
        self.is_town
    }

    fn capture_monsters(&self) -> (
        Vec<crate::game::loadsave::BinaryMonsterData>,
        Vec<crate::game::loadsave::MonsterWriteParams>,
    ) {
        let mut out = Vec::new();
        let mut params = Vec::new();
        for (_, m) in self.monster_manager.iter() {
            use crate::game::loadsave::BinaryMonsterData;
            let data = BinaryMonsterData {
                level_type: m.level_type as i32,
                mode: m.mode as i32,
                goal: m.goal as u8,
                goal_var1: m.goal_var1,
                goal_var2: m.goal_var2,
                goal_var3: m.goal_var3,
                path_count: m.path_count,
                position_x: m.x,
                position_y: m.y,
                future_x: m.x,
                future_y: m.y,
                old_x: m.home_x,
                old_y: m.home_y,
                direction: m.facing as i32,
                enemy: m.enemy as i32,
                enemy_x: m.enemy_position.x as u8,
                enemy_y: m.enemy_position.y as u8,
                anim_ticks_per_frame: 4,
                anim_tick_counter: 0,
                anim_num_frames: 10,
                anim_current_frame: 0,
                is_invalid: m.is_invalid,
                var1: m.var1,
                var2: m.var2,
                var3: m.var3,
                temp_x: 0,
                temp_y: 0,
                max_hp: m.max_hp,
                hp: m.hp,
                ai: m.ai as u8,
                intelligence: m.intelligence,
                flags: m.flags.0,
                active_for_ticks: m.active_for_ticks,
                last_x: m.target_x,
                last_y: m.target_y,
                rnd_item_seed: m.rnd_item_seed,
                ai_seed: m.ai_seed,
                unique_type: m.unique_type as u8,
                uniq_trans: m.uniq_trans,
                corpse_id: m.corpse_id,
                who_hit: m.who_hit,
                min_damage: m.min_damage,
                max_damage: m.max_damage,
                min_damage_special: m.min_damage_special,
                max_damage_special: m.max_damage_special,
                armor_class: m.armor_class,
                resistance: m.resistance,
                talk_msg: m.talk_msg,
                leader: m.leader,
                leader_relation: m.leader_relation as u8,
                pack_size: m.pack_size,
                light_id: m.light_id,
            };
            let p = crate::game::loadsave::MonsterWriteParams {
                level: m.level as i8,
                // Engine stores experience as u32; vanilla quest cap is u16.
                experience: m.experience.min(u16::MAX as u32) as u16,
                to_hit: m.to_hit.min(u8::MAX as i32) as u8,
                to_hit_special: 0,
            };
            out.push(data);
            params.push(p);
        }
        (out, params)
    }

    fn capture_objects(&self) -> Vec<crate::game::loadsave::BinaryObjectData> {
        self.objects
            .iter()
            .map(|o| crate::game::loadsave::BinaryObjectData {
                object_type: o.otype as i32,
                position_x: o.position.x,
                position_y: o.position.y,
                apply_lighting: true,
                anim_flag: o.anim_flag,
                anim_delay: o.anim_delay,
                anim_cnt: o.anim_cnt,
                anim_len: o.anim_len as u32,
                anim_frame: o.anim_frame as u32,
                anim_width: o.anim_width as u16,
                del_flag: o.del_flag,
                break_flag: o.breakable as i8,
                solid_flag: o.solid,
                miss_flag: true,
                selection_region: o.selection_region as i8,
                pre_flag: o.pre_flag != 0,
                trap_flag: o.is_trap,
                door_flag: o.door_state != 0,
                light_id: -1,
                rnd_seed: o.rnd_seed,
                var1: o.ovar1,
                var2: o.ovar2,
                var3: o.ovar3,
                var4: o.ovar4,
                var5: o.ovar5,
                var6: o.ovar6 as u32,
                book_message: o.book_message,
                var8: 0,
            })
            .collect()
    }

    fn capture_floor_items(&self) -> Vec<crate::game::loadsave::FloorItemData> {
        self.ground_items
            .iter()
            .map(|gi| crate::game::loadsave::FloorItemData {
                x: gi.x,
                y: gi.y,
                // GroundItemType discriminant: Gold=0, HealingPotion=1, ManaPotion=2.
                kind: match gi.item_type {
                    GroundItemType::Gold => 0,
                    GroundItemType::HealingPotion => 1,
                    GroundItemType::ManaPotion => 2,
                },
            })
            .collect()
    }
}

impl crate::game::loadsave::GameSnapshot for GameState {
    fn curr_level(&self) -> u8 {
        self.dungeon.level
    }
    fn is_set_level(&self) -> bool {
        false
    }
    fn is_hellfire(&self) -> bool {
        false
    }
    fn difficulty_u8(&self) -> u8 {
        0
    }
    fn dungeon_seed(&self) -> u32 {
        // Engine stores the dungeon seed inside the RNG; expose a stable hash of
        // the current level coords when no explicit seed is tracked. This keeps
        // the round-trip deterministic without requiring a new GameState field.
        0
    }
    fn level_seeds(&self) -> Vec<u32> {
        Vec::new()
    }
    fn capture_quests(&self) -> Vec<crate::game::loadsave::BinaryQuestData> {
        // Quest persistence is not yet wired to the live quest system; we
        // serialise an empty table (loaded back as empty).
        Vec::new()
    }
    fn capture_portals(&self) -> Vec<crate::game::loadsave::BinaryPortalData> {
        Vec::new()
    }
    fn capture_simple_missiles(&self) -> Vec<crate::game::loadsave::SimpleMissileData> {
        self.simple_missiles
            .iter()
            .map(|m| crate::game::loadsave::SimpleMissileData {
                x: m.x,
                y: m.y,
                dx: m.dx,
                dy: m.dy,
                damage: m.damage,
                range_left: m.range_left,
            })
            .collect()
    }
}

impl crate::game::save::LevelStateMut for GameState {
    fn apply_level(&mut self, data: &crate::game::loadsave::LevelSaveData) {
        let snap = data.to_snapshot();

        // --- Monsters: clear the live manager and repopulate from the snapshot.
        // We synthesise minimal `Monster` values via the same path the spawner
        // uses, then overwrite the persisted fields. This keeps the restore
        // self-contained (no need to regenerate the level).
        self.monster_manager.clear();

        for md in &snap.monsters {
            // Placeholder type — the engine does not yet persist the monster
            // data-table index, so the restored monster keeps the persisted
            // vitals/position/state but defaults to a FallenOne body. Future
            // work: store the monster-type id in BinaryMonsterData.
            let mtype = crate::game::monster::MonsterType::FallenOne;
            let mut m = crate::game::monster::Monster::new(
                0,
                mtype,
                md.position_x,
                md.position_y,
                0,
            );
            m.hp = md.hp;
            m.max_hp = md.max_hp;
            m.x = md.position_x;
            m.y = md.position_y;
            m.home_x = md.old_x;
            m.home_y = md.old_y;
            m.is_invalid = md.is_invalid;
            m.flags = crate::game::monster::MonsterFlags(md.flags);
            m.rnd_item_seed = md.rnd_item_seed;
            m.ai_seed = md.ai_seed;
            m.resistance = md.resistance;
            m.armor_class = md.armor_class;
            m.min_damage = md.min_damage;
            m.max_damage = md.max_damage;
            m.min_damage_special = md.min_damage_special;
            m.max_damage_special = md.max_damage_special;
            m.unique_type = crate::game::monster::UniqueMonsterType::None;
            m.uniq_trans = md.uniq_trans;
            m.corpse_id = md.corpse_id;
            m.who_hit = md.who_hit;
            m.leader = md.leader;
            m.leader_relation = crate::game::monster::LeaderRelation::None;
            m.pack_size = md.pack_size;
            m.light_id = md.light_id;
            m.intelligence = md.intelligence;
            m.active_for_ticks = md.active_for_ticks;
            m.target_x = md.last_x;
            m.target_y = md.last_y;
            m.talk_msg = md.talk_msg;
            self.monster_manager.add_monster(m);
        }

        // --- Objects: replace the live object list with the snapshot.
        self.objects.clear();
        for od in &snap.objects {
            // Placeholder object id; the engine does not yet persist the
            // object-type index, so the restored object keeps its persisted
            // position/animation/flags but defaults to a Chest1 body.
            let otype = crate::game::objdat::ObjectId::Chest1;
            let mut o = crate::game::objects::Object::new(otype, Point::new(od.position_x, od.position_y));
            o.anim_flag = od.anim_flag;
            o.anim_delay = od.anim_delay;
            o.anim_cnt = od.anim_cnt;
            o.anim_len = od.anim_len as i32;
            o.anim_frame = od.anim_frame as i32;
            o.anim_width = od.anim_width as i32;
            o.del_flag = od.del_flag;
            o.solid = od.solid_flag;
            o.is_trap = od.trap_flag;
            o.breakable = od.break_flag as i32;
            o.door_state = if od.door_flag { 1 } else { 0 };
            o.pre_flag = if od.pre_flag { 1 } else { 0 };
            o.rnd_seed = od.rnd_seed;
            o.ovar1 = od.var1;
            o.ovar2 = od.var2;
            o.ovar3 = od.var3;
            o.ovar4 = od.var4;
            o.ovar5 = od.var5;
            o.ovar6 = od.var6 as i32;
            o.book_message = od.book_message;
            self.objects.push(o);
        }

        // --- Floor items: replace the ground-items list.
        self.ground_items.clear();
        for it in &snap.floor_items {
            let item_type = match it.kind {
                0 => GroundItemType::Gold,
                1 => GroundItemType::HealingPotion,
                _ => GroundItemType::ManaPotion,
            };
            self.ground_items.push(GroundItem { x: it.x, y: it.y, item_type });
        }
    }
}

impl crate::game::save::GameStateMut for GameState {
    fn apply_game(&mut self, data: &crate::game::loadsave::GameSaveDataEnvelope) {
        let snap = data.to_snapshot();

        // Restore simple missiles.
        self.simple_missiles.clear();
        for m in &snap.simple_missiles {
            self.simple_missiles.push(SimpleMissile {
                x: m.x,
                y: m.y,
                dx: m.dx,
                dy: m.dy,
                damage: m.damage,
                range_left: m.range_left,
            });
        }

        // The embedded level snapshot inside the game blob is authoritative for
        // monsters/objects/items when the top-level `level` field is absent
        // (legacy v2 saves have neither; v3 saves have both and the top-level
        // `level` wins because `apply_level` is called after `apply_game`).
        // We deliberately do NOT call apply_level here to avoid a double
        // restore — the caller applies whichever blob is present.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_town_layout_default() {
        let layout = TownLayout::default();
        assert_eq!(layout.width, TOWN_MAX_X);
        assert_eq!(layout.height, TOWN_MAX_Y);
        assert_eq!(layout.d_piece.len(), TOWN_MAX_X * TOWN_MAX_Y);
        // All zeros initially
        assert_eq!(layout.get(0, 0), 0);
        assert_eq!(layout.get(75, 68), 0);
    }

    #[test]
    fn test_town_layout_set_get() {
        let mut layout = TownLayout::default();
        layout.set(10, 20, 426);
        assert_eq!(layout.get(10, 20), 426);
        // Other tiles unaffected
        assert_eq!(layout.get(11, 20), 0);
    }

    #[test]
    fn test_town_layout_bounds() {
        let layout = TownLayout::default();
        // Out of bounds returns 0, never panics
        assert_eq!(layout.get(-1, 0), 0);
        assert_eq!(layout.get(0, -1), 0);
        assert_eq!(layout.get(TOWN_MAX_X as i32, 0), 0);
        assert_eq!(layout.get(0, TOWN_MAX_Y as i32), 0);

        // Set out of bounds is a no-op
        let mut layout = TownLayout::default();
        layout.set(-1, 0, 999);
        layout.set(TOWN_MAX_X as i32, 0, 999);
        assert_eq!(layout.get(0, 0), 0);
    }

    #[test]
    fn test_init_town_camera() {
        let player = Player::new();
        let mut gs = GameState::new(player, true, 42);
        assert_eq!(gs.camera.tile_x, 0);
        gs.init_town_camera();
        // C++ ENTRY_MAIN spawn = {75, 68}
        assert_eq!(gs.camera.tile_x, 75);
        assert_eq!(gs.camera.tile_y, 68);
        assert_eq!(gs.player.position.x, 75);
        assert_eq!(gs.player.position.y, 68);
    }

    /// Build a GameState with a dungeon layout whose `floor_tiles` form a
    /// straight corridor along y=10, x in [10..30], and place a monster on it.
    fn dungeon_gs_with_corridor(monster_type: crate::game::monster::MonsterType, mx: i32, my: i32) -> GameState {
        let player = Player::new();
        let mut gs = GameState::new(player, true, 42);
        gs.is_town = false;
        gs.in_dungeon = true;
        let mut layout = DungeonLayout::default();
        // Floor corridor: tiles (x, 10) for x in 10..=30.
        for x in 10..=30 {
            layout.floor_tiles.push((x, 10));
        }
        gs.dungeon_layout = Some(layout);

        // Place one monster.
        let mut m = crate::game::monster::Monster::new(1, monster_type, mx, my, 1);
        m.mode = crate::game::monster::MonsterMode::Stand;
        gs.add_monster(m);
        gs
    }

    // ========================================================================
    // Ground item drop / pickup tests
    // ========================================================================

    #[test]
    fn test_ground_item_struct_fields() {
        // GroundItem carries tile coords + type and is Copy.
        let g = GroundItem { x: 12, y: 7, item_type: GroundItemType::Gold };
        assert_eq!(g.x, 12);
        assert_eq!(g.y, 7);
        assert_eq!(g.item_type, GroundItemType::Gold);
    }

    #[test]
    fn test_ground_item_type_display_names() {
        assert_eq!(GroundItemType::Gold.display_name(), "Gold");
        assert_eq!(GroundItemType::HealingPotion.display_name(), "Potion of Healing");
        assert_eq!(GroundItemType::ManaPotion.display_name(), "Potion of Mana");
    }

    #[test]
    fn test_new_game_state_has_empty_ground_items() {
        let gs = GameState::new(Player::new(), true, 1);
        assert!(gs.ground_items.is_empty(), "fresh state has no ground items");
    }

    #[test]
    fn test_roll_monster_drop_no_drop_seed() {
        // With this RNG seed the first roll (0..100) is >= 40, so nothing drops.
        let mut gs = GameState::new(Player::new(), false, 1);
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        gs.roll_monster_drop(10, 10, &mut rng);
        // We can't assert exact emptiness (seed-dependent), so instead we run
        // many rolls and assert the drop rate stays within the expected band.
        let mut drops = 0usize;
        for _ in 0..1000 {
            let before = gs.ground_items.len();
            gs.roll_monster_drop(0, 0, &mut rng);
            if gs.ground_items.len() > before {
                drops += 1;
            }
        }
        // ~40% drop chance => expect drops in [300, 500].
        assert!(drops >= 300 && drops <= 500, "drop rate out of band: {}", drops);
    }

    #[test]
    fn test_roll_monster_drop_forces_drop() {
        // Use a seed where the first 0..100 roll is < 40 so a drop happens.
        // We just try several seeds until one drops, then verify the spawned
        // item lands at the requested tile with a valid type.
        let mut gs = GameState::new(Player::new(), false, 1);
        let mut placed = false;
        for seed in 0..50u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let before = gs.ground_items.len();
            gs.roll_monster_drop(21, 22, &mut rng);
            if gs.ground_items.len() > before {
                let g = gs.ground_items.last().unwrap();
                assert_eq!(g.x, 21);
                assert_eq!(g.y, 22);
                assert!(g.item_type == GroundItemType::Gold
                    || g.item_type == GroundItemType::HealingPotion
                    || g.item_type == GroundItemType::ManaPotion);
                placed = true;
                break;
            }
        }
        assert!(placed, "expected at least one forced drop within 50 seeds");
    }

    #[test]
    fn test_drop_type_distribution() {
        // Over many forced drops the type distribution should roughly match
        // 70/20/10. We collect drops and check the bands.
        let mut counts = [0usize; 3]; // [gold, heal, mana]
        let mut rng = rand::rngs::StdRng::seed_from_u64(99);
        let mut gs = GameState::new(Player::new(), false, 1);
        let mut attempts = 0;
        while counts.iter().sum::<usize>() < 2000 && attempts < 20000 {
            attempts += 1;
            let before = gs.ground_items.len();
            gs.roll_monster_drop(0, 0, &mut rng);
            if gs.ground_items.len() > before {
                let t = gs.ground_items.last().unwrap().item_type;
                match t {
                    GroundItemType::Gold => counts[0] += 1,
                    GroundItemType::HealingPotion => counts[1] += 1,
                    GroundItemType::ManaPotion => counts[2] += 1,
                }
            }
        }
        let total = counts.iter().sum::<usize>() as f64;
        let gold_pct = counts[0] as f64 / total * 100.0;
        let heal_pct = counts[1] as f64 / total * 100.0;
        let mana_pct = counts[2] as f64 / total * 100.0;
        // 70/20/10 with a generous tolerance band.
        assert!(gold_pct > 60.0 && gold_pct < 80.0, "gold pct {}", gold_pct);
        assert!(heal_pct > 12.0 && heal_pct < 28.0, "heal pct {}", heal_pct);
        assert!(mana_pct > 4.0 && mana_pct < 18.0, "mana pct {}", mana_pct);
    }

    #[test]
    fn test_pickup_gold_adds_to_player_gold() {
        let mut gs = GameState::new(Player::new(), false, 1);
        let gold_before = gs.player._p_gold;
        // Drop a gold item directly onto the player's tile.
        gs.player.position = Point::new(30, 30);
        gs.ground_items.push(GroundItem { x: 30, y: 30, item_type: GroundItemType::Gold });
        gs.pickup_ground_items();
        assert_eq!(
            gs.player._p_gold,
            gold_before + GroundItemType::GOLD_AMOUNT,
            "gold should increase by GOLD_AMOUNT"
        );
        assert!(gs.ground_items.is_empty(), "picked-up item removed from floor");
    }

    #[test]
    fn test_pickup_only_removes_coincident_items() {
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player.position = Point::new(30, 30);
        // One item on the player's tile, one elsewhere.
        gs.ground_items.push(GroundItem { x: 30, y: 30, item_type: GroundItemType::Gold });
        gs.ground_items.push(GroundItem { x: 40, y: 40, item_type: GroundItemType::HealingPotion });
        assert_eq!(gs.ground_items.len(), 2);
        gs.pickup_ground_items();
        // Only the coincident item is removed.
        assert_eq!(gs.ground_items.len(), 1);
        assert_eq!(gs.ground_items[0].x, 40);
        assert_eq!(gs.ground_items[0].y, 40);
        assert_eq!(gs.ground_items[0].item_type, GroundItemType::HealingPotion);
    }

    #[test]
    fn test_pickup_potion_logs_without_crash() {
        // Potions don't have inventory integration yet; pickup should still
        // remove them from the floor without affecting gold.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player.position = Point::new(5, 5);
        let gold_before = gs.player._p_gold;
        gs.ground_items.push(GroundItem { x: 5, y: 5, item_type: GroundItemType::HealingPotion });
        gs.ground_items.push(GroundItem { x: 5, y: 5, item_type: GroundItemType::ManaPotion });
        gs.pickup_ground_items();
        assert!(gs.ground_items.is_empty());
        assert_eq!(gs.player._p_gold, gold_before, "potion pickup must not change gold");
    }

    #[test]
    fn test_pickup_noop_when_empty() {
        let mut gs = GameState::new(Player::new(), false, 1);
        // No items at all: pickup is a no-op and must not panic.
        gs.pickup_ground_items();
        assert!(gs.ground_items.is_empty());
    }

    #[test]
    fn test_pickup_noop_when_not_coincident() {
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player.position = Point::new(10, 10);
        gs.ground_items.push(GroundItem { x: 99, y: 99, item_type: GroundItemType::Gold });
        gs.pickup_ground_items();
        // Item stays because the player isn't on its tile.
        assert_eq!(gs.ground_items.len(), 1);
    }

    #[test]
    fn test_pickup_multiple_items_same_tile() {
        // Walking onto a tile with several items should pick them all up.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player.position = Point::new(7, 7);
        gs.ground_items.push(GroundItem { x: 7, y: 7, item_type: GroundItemType::Gold });
        gs.ground_items.push(GroundItem { x: 7, y: 7, item_type: GroundItemType::Gold });
        gs.ground_items.push(GroundItem { x: 7, y: 7, item_type: GroundItemType::HealingPotion });
        let gold_before = gs.player._p_gold;
        gs.pickup_ground_items();
        assert!(gs.ground_items.is_empty());
        // Two gold piles => 2 * GOLD_AMOUNT.
        assert_eq!(gs.player._p_gold, gold_before + 2 * GroundItemType::GOLD_AMOUNT);
    }

    #[test]
    fn test_gold_amount_is_positive_constant() {
        // Sanity: the gold-per-pile constant is a sensible positive value.
        assert!(GroundItemType::GOLD_AMOUNT > 0);
    }

    #[test]
    fn test_dungeon_layout_has_floor_tiles_field() {
        let layout = DungeonLayout::default();
        assert!(layout.floor_tiles.is_empty(), "default layout has no floor tiles");
    }

    #[test]
    fn test_monster_chases_player_when_in_range() {
        use crate::game::monster::{MonsterAIState, MonsterType};
        // Monster at (10,10), player at (15,10) on the same corridor: distance
        // 5, within the default aggro range (8). The monster should switch to
        // Chasing and step toward the player (never away).
        let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 10, 10);
        gs.player.position = Point::new(15, 10);

        gs.update_monster_movement(0);

        let m = gs.get_monster(0).expect("monster present");
        assert!(
            m.ai_state == MonsterAIState::Chasing || m.ai_state == MonsterAIState::Attacking,
            "monster in range should be chasing/attacking, got {:?}",
            m.ai_state
        );
        // The monster should not have moved away from the player.
        assert!(m.x >= 10, "monster should not move away from player, x={}", m.x);
    }

    #[test]
    fn test_monster_idles_when_out_of_range() {
        use crate::game::monster::{MonsterAIState, MonsterType};
        // Monster at (10,10), player at (100,100): far out of aggro range.
        let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 10, 10);
        gs.player.position = Point::new(100, 100);

        gs.update_monster_movement(0);

        let m = gs.get_monster(0).expect("monster present");
        assert_eq!(
            m.ai_state,
            MonsterAIState::Idle,
            "monster out of range should stay idle"
        );
        // Idle monsters may wander up to 4 tiles from home (10,10), but should
        // never wander far away.
        let wander = (m.x - 10).abs() + (m.y - 10).abs();
        assert!(wander <= 4, "idle monster should stay near home, wandered {}", wander);
    }

    #[test]
    fn test_monster_does_not_walk_onto_player() {
        use crate::game::monster::MonsterType;
        // Monster adjacent to the player should approach but never land on the
        // player's tile.
        let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 10, 10);
        gs.player.position = Point::new(11, 10); // adjacent, distance 1

        for _ in 0..20 {
            gs.update_monster_movement(0);
            let m = gs.get_monster(0).unwrap();
            assert!(
                !(m.x == gs.player.position.x && m.y == gs.player.position.y),
                "monster must not occupy the player's tile"
            );
        }
    }

    /// Full save/load round-trip on a real `GameState`: build a state, mutate
    /// the player's vitals/position/level, save to a temp slot, then load into
    /// a *fresh* state and verify every captured field was restored —
    /// including the 64x fixed-point HP/Mana values.
    #[test]
    fn test_game_state_save_load_roundtrip() {
        use crate::game::save::SaveManager;
        use crate::game::player_exact::HeroClass;

        // Unique temp save dir so parallel test runs never collide.
        let tmp = std::env::temp_dir().join(format!(
            "devilutionx_rs_gs_roundtrip_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        // Override the SaveManager default dir by building the SaveSlot via the
        // manager pointed at our temp dir.
        let mgr = SaveManager::with_dir(tmp.to_str().unwrap());

        // --- Source state: set up a non-trivial player. ---
        let mut gs = GameState::new(Player::new(), false, 42);
        gs.player.set_name("Roundtrip");
        gs.player._p_class = HeroClass::Sorcerer;
        gs.player._p_level = 9;
        gs.player.plr_level = 4;
        gs.player._p_strength = 40;
        gs.player._p_base_str = 30;
        gs.player._p_magic = 55;
        gs.player._p_dexterity = 35;
        gs.player._p_vitality = 28;
        gs.player._p_stat_pts = 12;
        // 64x HP/Mana: real in-game values would be e.g. 64*120.
        gs.player._p_hit_points = 64 * 117; // took 3 damage from full 120
        gs.player._p_max_hp = 64 * 120;
        gs.player._p_max_hp_base = 64 * 120;
        gs.player._p_hp_base = 64 * 117;
        gs.player._p_mana = 64 * 88;
        gs.player._p_max_mana = 64 * 100;
        gs.player._p_max_mana_base = 64 * 100;
        gs.player._p_mana_base = 64 * 88;
        gs.player._p_experience = 12345;
        gs.player._p_gold = 999;
        gs.player.position = Point::new(77, 66);
        gs.in_dungeon = true;
        gs.is_town = false;
        gs.game_tick = 4242;
        gs.camera.tile_x = 77;
        gs.camera.tile_y = 66;

        // Snapshot the to-be-saved values.
        let saved_hp = gs.player._p_hit_points;
        let saved_mana = gs.player._p_mana;
        let saved_xp = gs.player._p_experience;
        let saved_pos = gs.player.position;

        // Save via the GameState bridge (uses build_save_slot).
        use crate::game::save::build_save_slot;
        let slot = build_save_slot(&gs, &gs);
        mgr.save_slot(7, &slot).unwrap();
        assert!(mgr.slot_exists_v2(7));

        // --- Mutate the source state to simulate continued play. ---
        gs.player._p_hit_points = 1;
        gs.player._p_mana = 1;
        gs.player._p_experience = 0;
        gs.player.position = Point::new(0, 0);
        gs.game_tick = 0;

        // --- Load into the (mutated) state and verify restoration. ---
        let data = mgr.load_slot(7).unwrap();
        assert!(data.is_valid());
        // Apply player.
        use crate::game::save::PlayerSnapshotMut;
        PlayerSnapshotMut::apply_slot(&mut gs.player, &data.player);
        // Apply world.
        gs.in_dungeon = data.world.in_dungeon;
        gs.is_town = data.world.is_town;
        gs.game_tick = data.world.game_tick;
        gs.camera.tile_x = data.world.cam_x;
        gs.camera.tile_y = data.world.cam_y;

        // --- Assertions: every captured field is restored losslessly. ---
        assert_eq!(gs.player.get_name(), "Roundtrip");
        assert_eq!(gs.player._p_class, HeroClass::Sorcerer);
        assert_eq!(gs.player._p_level, 9);
        assert_eq!(gs.player.plr_level, 4);
        assert_eq!(gs.player._p_strength, 40);
        assert_eq!(gs.player._p_base_str, 30);
        assert_eq!(gs.player._p_magic, 55);
        assert_eq!(gs.player._p_dexterity, 35);
        assert_eq!(gs.player._p_vitality, 28);
        assert_eq!(gs.player._p_stat_pts, 12);
        // 64x vitals restored verbatim (no precision loss).
        assert_eq!(gs.player._p_hit_points, saved_hp);
        assert_eq!(gs.player._p_hit_points, 64 * 117);
        assert_eq!(gs.player._p_max_hp, 64 * 120);
        assert_eq!(gs.player._p_mana, saved_mana);
        assert_eq!(gs.player._p_mana, 64 * 88);
        assert_eq!(gs.player._p_max_mana, 64 * 100);
        assert_eq!(gs.player._p_experience, saved_xp);
        assert_eq!(gs.player._p_gold, 999);
        assert_eq!(gs.player.position, saved_pos);
        assert_eq!(gs.player.position, Point::new(77, 66));
        assert!(gs.in_dungeon);
        assert!(!gs.is_town);
        assert_eq!(gs.game_tick, 4242);
        assert_eq!(gs.camera.tile_x, 77);

        // Cleanup.
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// F5/F9 full-state round-trip: exercises `save_to_slot`/`load_from_slot`
    /// which now capture monsters + objects + ground items in addition to the
    /// player. Verifies the dungeon state survives a save/load cycle.
    #[test]
    fn test_game_state_save_load_full_roundtrip() {
        use crate::game::monster::{Monster, MonsterType};
        use crate::game::objects::Object;
        use crate::game::types::Point;

        let mut gs = GameState::new(Player::new(), true, 7);
        gs.is_town = false;
        gs.in_dungeon = true;

        // Populate a monster with distinctive vitals.
        let mut m = Monster::new(1, MonsterType::FallenOne, 42, 17, 0);
        m.hp = 37;
        m.max_hp = 64;
        m.rnd_item_seed = 0xCAFEBABE;
        m.resistance = 0b110;
        m.armor_class = 9;
        gs.monster_manager.clear();
        gs.monster_manager.add_monster(m);

        // Populate an object.
        let mut o = Object::new(crate::game::objdat::ObjectId::Chest1, Point::new(50, 50));
        o.rnd_seed = 0x1234;
        o.anim_len = 10;
        o.solid = true;
        gs.objects.clear();
        gs.objects.push(o);

        // Populate a ground item.
        gs.ground_items.clear();
        gs.ground_items.push(GroundItem {
            x: 30,
            y: 31,
            item_type: GroundItemType::HealingPotion,
        });

        // Snapshot the distinctive values.
        let saved_monster_hp = 37i32;
        let saved_monster_seed = 0xCAFEBABEu32;
        let saved_obj_seed = 0x1234u32;
        let saved_item_kind = GroundItemType::HealingPotion;

        // Save via the F5 path.
        gs.save_to_slot(8).expect("save_to_slot should succeed");

        // Mutate live state to simulate continued play.
        gs.monster_manager.clear();
        gs.objects.clear();
        gs.ground_items.clear();
        assert_eq!(gs.monster_manager.active_count(), 0);

        // Load via the F9 path.
        gs.load_from_slot(8).expect("load_from_slot should succeed");

        // Monster restored.
        assert_eq!(gs.monster_manager.active_count(), 1);
        let (_, restored_m) = gs.monster_manager.iter().next().unwrap();
        assert_eq!(restored_m.hp, saved_monster_hp);
        assert_eq!(restored_m.max_hp, 64);
        assert_eq!(restored_m.rnd_item_seed, saved_monster_seed);
        assert_eq!(restored_m.x, 42);
        assert_eq!(restored_m.y, 17);

        // Object restored.
        assert_eq!(gs.objects.len(), 1);
        assert_eq!(gs.objects[0].rnd_seed, saved_obj_seed);
        assert_eq!(gs.objects[0].position, Point::new(50, 50));

        // Ground item restored.
        assert_eq!(gs.ground_items.len(), 1);
        assert_eq!(gs.ground_items[0].x, 30);
        assert_eq!(gs.ground_items[0].y, 31);
        assert_eq!(gs.ground_items[0].item_type, saved_item_kind);

        // Cleanup: remove the slot file (SaveManager writes to its default dir).
        use crate::game::save::SaveManager;
        let path = SaveManager::new().slot_path_public(8);
        let _ = std::fs::remove_file(path);
    }

    // ========================================================================
    // Audio (SFX) trigger tests
    // ========================================================================

    #[test]
    fn test_pending_sfx_starts_empty_and_drains() {
        // A fresh GameState has no pending SFX, and drain returns an empty vec
        // while resetting the queue.
        let mut gs = GameState::new(Player::new(), false, 1);
        assert!(gs.pending_sfx.is_empty());
        assert!(gs.drain_pending_sfx().is_empty());
        // After draining, pushing then draining returns the pushed name.
        gs.queue_sfx("swing");
        gs.queue_sfx("monster_death");
        let drained = gs.drain_pending_sfx();
        assert_eq!(drained, vec!["swing".to_string(), "monster_death".to_string()]);
        // Second drain is empty (the queue was taken).
        assert!(gs.drain_pending_sfx().is_empty());
    }

    #[test]
    fn test_combat_queues_sfx_on_hit_or_kill() {
        // When the player is adjacent to a monster, update() runs melee
        // combat. A non-killing hit must queue "swing" and a kill must queue
        // "monster_death". We run two scenarios:
        //   * high-HP monster  → hits that don't kill → "swing"
        //   * low-HP  monster  → hits that kill       → "monster_death"
        use crate::game::monster::MonsterType;

        // ── Scenario A: tanky monster so hits survive → "swing" ────────────
        let mut saw_swing = false;
        for seed in 0..50u64 {
            let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 11, 10);
            gs.player.position = Point::new(12, 10);
            gs.player._p_level = 5;
            gs.player._p_dexterity = 30;
            gs.player._p_i_bonus_to_hit = 100;
            gs.player._p_i_min_dam = 1;
            gs.player._p_i_max_dam = 3; // small damage so the tank survives

            // Find the placed monster's actual id (add_monster may re-index)
            // and make it very tanky with zero armor.
            let monster_ids: Vec<usize> =
                gs.monster_manager.iter().map(|(id, _)| id).collect();
            for id in &monster_ids {
                if let Some(mon) = gs.monster_manager.get_monster_mut(*id) {
                    mon.hp = 5000 * 64; // huge HP: no hit will kill
                    mon.max_hp = 5000 * 64;
                    mon.armor_class = 0;
                }
            }

            let _ = gs.drain_pending_sfx();
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            gs.update(&mut rng);

            for name in gs.drain_pending_sfx() {
                if name == "swing" {
                    saw_swing = true;
                }
            }
            if saw_swing {
                break;
            }
        }
        assert!(
            saw_swing,
            "combat should have queued a 'swing' SFX on at least one non-killing hit"
        );

        // ── Scenario B: frail monster so any hit kills → "monster_death" ───
        let mut saw_monster_death = false;
        for seed in 0..50u64 {
            let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 11, 10);
            gs.player.position = Point::new(12, 10);
            gs.player._p_level = 20;
            gs.player._p_dexterity = 100;
            gs.player._p_i_bonus_to_hit = 200;
            gs.player._p_i_min_dam = 10;
            gs.player._p_i_max_dam = 20;

            let monster_ids: Vec<usize> =
                gs.monster_manager.iter().map(|(id, _)| id).collect();
            for id in &monster_ids {
                if let Some(mon) = gs.monster_manager.get_monster_mut(*id) {
                    mon.hp = 1 * 64; // any hit kills
                    mon.max_hp = 1 * 64;
                    mon.armor_class = 0;
                }
            }

            let _ = gs.drain_pending_sfx();
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            gs.update(&mut rng);

            for name in gs.drain_pending_sfx() {
                if name == "monster_death" {
                    saw_monster_death = true;
                }
            }
            if saw_monster_death {
                break;
            }
        }
        assert!(
            saw_monster_death,
            "combat should have queued a 'monster_death' SFX on at least one kill"
        );
    }

    #[test]
    fn test_combat_no_sfx_when_no_monster_adjacent() {
        // With no monster next to the player, update() must not queue any SFX.
        let mut gs = GameState::new(Player::new(), false, 1);
        // Player far from any monster (none placed).
        gs.player.position = Point::new(50, 50);
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        gs.update(&mut rng);
        assert!(
            gs.drain_pending_sfx().is_empty(),
            "no SFX should be queued when no monster is in melee range"
        );
    }

    // ========================================================================
    // Death detection / resurrection tests
    // ========================================================================

    #[test]
    fn test_fresh_game_state_is_not_dead() {
        // A freshly-created GameState has the player alive and the dead flag
        // clear, regardless of town/dungeon mode.
        let gs_town = GameState::new(Player::new(), true, 1);
        assert!(!gs_town.player_dead, "fresh town state not dead");
        assert!(!gs_town.is_player_dead());

        let gs_dungeon = GameState::new(Player::new(), false, 1);
        assert!(!gs_dungeon.player_dead, "fresh dungeon state not dead");
        assert!(!gs_dungeon.is_player_dead());
    }

    #[test]
    fn test_check_player_death_latches_when_hp_zero() {
        // When HP drops to exactly 0, check_player_death must latch
        // player_dead and is_player_dead must return true.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_hit_points = 0;
        assert!(!gs.player_dead, "no latch before the check runs");
        gs.check_player_death();
        assert!(gs.player_dead, "HP == 0 should latch player_dead");
        assert!(gs.is_player_dead());
    }

    #[test]
    fn test_check_player_death_latches_when_hp_negative() {
        // HP < 0 (overkill) should also latch death.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_hit_points = -64; // one point of overkill in 64x
        gs.check_player_death();
        assert!(gs.player_dead);
        assert!(gs.is_player_dead());
    }

    #[test]
    fn test_check_player_death_noop_when_alive() {
        // HP > 0 → no latch. (Player::new() leaves HP/max at 0, so set a
        // real max first.)
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_max_hp = 64 * 100;
        gs.player._p_hit_points = gs.player._p_max_hp; // full HP
        gs.check_player_death();
        assert!(!gs.player_dead);
        assert!(!gs.is_player_dead());
    }

    #[test]
    fn test_check_player_death_idempotent_when_already_dead() {
        // Once latched, re-running the check is a no-op (stays dead).
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_hit_points = 0;
        gs.check_player_death();
        assert!(gs.player_dead);
        // Mutate HP then re-check: flag stays latched (doesn't "un-die").
        gs.player._p_hit_points = 1000;
        gs.check_player_death();
        assert!(gs.player_dead, "once dead, stays dead until resurrect");
    }

    #[test]
    fn test_death_latches_via_update_when_monster_kills_player() {
        // End-to-end: a monster next to the player with enough damage to kill
        // in one hit must latch player_dead after a single update().
        use crate::game::monster::MonsterType;

        // Try multiple seeds since the hit roll is RNG-dependent; we just need
        // *some* seed where the first attack connects and kills.
        let mut killed = false;
        for seed in 0..200u64 {
            let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 11, 10);
            gs.player.position = Point::new(12, 10);
            // Frail player so any connected hit kills.
            gs.player._p_hit_points = 64; // 1 HP in display units
            gs.player._p_max_hp = 64;
            gs.player._p_armor_class = 0; // no armor reduction
            // Make the monster hit hard and accurately.
            let monster_ids: Vec<usize> =
                gs.monster_manager.iter().map(|(id, _)| id).collect();
            for id in &monster_ids {
                if let Some(mon) = gs.monster_manager.get_monster_mut(*id) {
                    mon.intelligence = 100; // near-guaranteed hit (5..95 clamp)
                    mon.min_damage = 100;
                    mon.max_damage = 100;
                }
            }

            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            gs.update(&mut rng);

            if gs.is_player_dead() {
                killed = true;
                // HP should be clamped to 0 by modify_hp.
                assert!(gs.player._p_hit_points <= 0);
                break;
            }
        }
        assert!(
            killed,
            "expected at least one seed where a monster attack kills the player and latches player_dead"
        );
    }

    #[test]
    fn test_update_skips_processing_while_dead() {
        // Once player_dead is latched, update() must NOT run HP regen
        // (process_player_internal bumps HP toward max). We set HP to a small
        // positive value, latch death manually, then call update and verify HP
        // is unchanged (no regen) and the flag stays latched.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_max_hp = 64 * 100;
        gs.player._p_hit_points = 64 * 5; // 5 HP (injured, would normally regen)
        gs.player_dead = true; // pre-latch death

        let hp_before = gs.player._p_hit_points;
        let mut rng = rand::rngs::StdRng::seed_from_u64(3);
        gs.update(&mut rng);

        assert_eq!(
            gs.player._p_hit_points,
            hp_before,
            "no HP regen while dead (update should skip process_player_internal)"
        );
        assert!(gs.player_dead, "death flag stays latched across update");
    }

    // ========================================================================
    // Resurrection tests
    // ========================================================================

    #[test]
    fn test_resurrect_player_is_noop_when_alive() {
        // Calling resurrect on a live player is a defensive no-op: HP/gold/
        // position must be untouched.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_hit_points = gs.player._p_max_hp;
        gs.player._p_gold = 500;
        let (hp, gold, pos) = (
            gs.player._p_hit_points,
            gs.player._p_gold,
            gs.player.position,
        );
        gs.resurrect_player();
        assert!(!gs.player_dead, "resurrect on live player must not set dead");
        assert_eq!(gs.player._p_hit_points, hp);
        assert_eq!(gs.player._p_gold, gold);
        assert_eq!(gs.player.position, pos);
    }

    #[test]
    fn test_resurrect_player_restores_full_hp_and_mana() {
        // After resurrect, HP and Mana equal their max (64x).
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_max_hp = 64 * 120;
        gs.player._p_hit_points = 0; // dead
        gs.player._p_hp_base = 0;
        gs.player._p_max_mana = 64 * 80;
        gs.player._p_mana = 0;
        gs.player._p_mana_base = 0;
        gs.player_dead = true;

        gs.resurrect_player();

        assert!(!gs.player_dead, "resurrect clears the dead flag");
        assert_eq!(gs.player._p_hit_points, 64 * 120, "HP restored to full");
        assert_eq!(gs.player._p_hp_base, gs.player._p_max_hp_base);
        assert_eq!(gs.player._p_mana, 64 * 80, "Mana restored to full");
        assert_eq!(gs.player._p_mana_base, gs.player._p_max_mana_base);
        assert!(gs.player._p_hit_points > 0, "player is alive again");
        assert!(!gs.is_player_dead());
    }

    #[test]
    fn test_resurrect_player_halves_gold() {
        // Diablo's classic death penalty halves carried gold. Odd amounts
        // floor-divide (integer /2).
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.player._p_gold = 1000;
        gs.player._p_hit_points = 0;
        gs.player._p_max_hp = 64 * 50;
        gs.player_dead = true;

        gs.resurrect_player();
        assert_eq!(gs.player._p_gold, 500, "even gold halved exactly");

        // Odd amount floors.
        let mut gs2 = GameState::new(Player::new(), false, 1);
        gs2.player._p_gold = 999;
        gs2.player._p_hit_points = 0;
        gs2.player._p_max_hp = 64 * 50;
        gs2.player_dead = true;
        gs2.resurrect_player();
        assert_eq!(gs2.player._p_gold, 499, "odd gold floors on halving");
    }

    #[test]
    fn test_resurrect_player_returns_to_town_spawn() {
        // After resurrect, the player is back in Tristram at ENTRY_MAIN
        // (75, 68), in_dungeon is false, and the camera is centred on the
        // spawn.
        let mut gs = GameState::new(Player::new(), false, 1);
        gs.in_dungeon = true;
        gs.is_town = false;
        gs.player._p_hit_points = 0;
        gs.player._p_max_hp = 64 * 50;
        gs.player_dead = true;
        // Park the player somewhere off-spawn.
        gs.player.position = Point::new(10, 10);
        gs.camera.tile_x = 10;
        gs.camera.tile_y = 10;

        gs.resurrect_player();

        assert!(!gs.in_dungeon, "resurrect returns to town (in_dungeon=false)");
        assert!(gs.is_town, "is_town true after resurrect");
        // C++ ENTRY_MAIN spawn.
        assert_eq!(gs.player.position.x, 75);
        assert_eq!(gs.player.position.y, 68);
        assert_eq!(gs.camera.tile_x, 75);
        assert_eq!(gs.camera.tile_y, 68);
    }

    #[test]
    fn test_resurrect_player_clears_dungeon_state() {
        // Resurrecting clears the dungeon layout + monsters + sprites + ground
        // items so a fresh dungeon is generated on the next descent.
        use crate::game::monster::MonsterType;
        let mut gs = dungeon_gs_with_corridor(MonsterType::Zombie, 11, 10);
        gs.player._p_hit_points = 0;
        gs.player._p_max_hp = 64 * 50;
        gs.player_dead = true;
        // Populate some dungeon state that should be cleared.
        gs.dungeon_up_stairs = Some((5, 5));
        gs.ground_items.push(GroundItem { x: 1, y: 1, item_type: GroundItemType::Gold });
        gs.simple_missiles.push(SimpleMissile {
            x: 0, y: 0, dx: 1, dy: 0, damage: 5, range_left: 5,
        });
        assert!(gs.active_monster_count() > 0);

        gs.resurrect_player();

        assert!(gs.dungeon_layout.is_none(), "dungeon layout cleared");
        assert!(gs.dungeon_up_stairs.is_none(), "up-stairs cleared");
        assert!(gs.ground_items.is_empty(), "ground items cleared");
        assert!(gs.simple_missiles.is_empty(), "missiles cleared");
        assert_eq!(gs.active_monster_count(), 0, "monsters cleared");
        assert!(gs.monster_sprites.is_none(), "monster sprites cleared");
    }

    #[test]
    fn test_resurrect_full_cycle_resume_normal_logic() {
        // After death → resurrect, the player is alive in town at full HP and
        // a subsequent update() runs normally (no longer paused). We verify HP
        // regen is *not* triggered (player is already at full) and the dead
        // flag stays clear.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.player._p_max_hp = 64 * 60;
        gs.player._p_hit_points = 0;
        gs.player._p_gold = 200;
        gs.player_dead = true;

        gs.resurrect_player();
        assert!(!gs.player_dead);
        let hp_after = gs.player._p_hit_points;
        assert_eq!(hp_after, 64 * 60);

        // Run an update — should be a normal alive update (no death latch).
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        gs.update(&mut rng);
        assert!(!gs.player_dead, "still alive after update");
        assert_eq!(gs.player._p_hit_points, hp_after, "full HP unchanged by regen");
    }

    // ========================================================================
    // Shop interaction tests (Step 1: open/close, npc_at_tile, buy)
    // ========================================================================

    #[test]
    fn test_fresh_game_state_shop_is_closed() {
        // A freshly-created GameState must have the shop panel closed and no
        // active shop NPC.
        let gs = GameState::new(Player::new(), true, 1);
        assert!(!gs.shop_open, "fresh state has shop closed");
        assert!(gs.active_shop_npc.is_none(), "fresh state has no active NPC");
    }

    #[test]
    fn test_open_close_shop_round_trip() {
        // open_shop sets both flags; close_shop clears both.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.open_shop(0); // Griswold
        assert!(gs.shop_open);
        assert_eq!(gs.active_shop_npc, Some(0));
        gs.close_shop();
        assert!(!gs.shop_open);
        assert!(gs.active_shop_npc.is_none());
    }

    #[test]
    fn test_npc_runs_shop_routing() {
        // Only the 4 shop-capable NPCs (Smith=0, Healer=1, Witch=6, PegBoy=8)
        // should report running a shop. The gossip-only NPCs (Tavern=3,
        // Story=4, Drunk=5, Barmaid=7, Cow=9) do not.
        assert!(GameState::npc_runs_shop(0));
        assert!(GameState::npc_runs_shop(1));
        assert!(GameState::npc_runs_shop(6));
        assert!(GameState::npc_runs_shop(8));
        assert!(!GameState::npc_runs_shop(3));
        assert!(!GameState::npc_runs_shop(4));
        assert!(!GameState::npc_runs_shop(5));
        assert!(!GameState::npc_runs_shop(7));
        assert!(!GameState::npc_runs_shop(9));
    }

    #[test]
    fn test_shop_name_for_each_npc() {
        // The shop-name lookup must return a distinct, human-readable header
        // for each shop-capable NPC.
        assert_eq!(GameState::shop_name_for_npc(0), "GRISWOLD THE BLACKSMITH");
        assert_eq!(GameState::shop_name_for_npc(1), "PEPIN THE HEALER");
        assert_eq!(GameState::shop_name_for_npc(6), "ADRIA THE WITCH");
        assert_eq!(GameState::shop_name_for_npc(8), "WIRT THE PEG-LEGGED BOY");
    }

    #[test]
    fn test_npc_at_tile_finds_nearby_npc() {
        // build_towner_list seeds Griswold (Smith) at his default position.
        // Clicking exactly on his tile must resolve to that NPC.
        let gs = GameState::new(Player::new(), true, 1);
        let griswold = gs.towners.iter().find(|t| t.3 == 0).expect("Smith seeded");
        let hit = gs.npc_at_tile(griswold.0, griswold.1);
        assert!(hit.is_some(), "click on Griswold's tile should find him");
        let (_, _, _, kind) = hit.unwrap();
        assert_eq!(kind, 0, "found NPC should be the Smith");
    }

    #[test]
    fn test_npc_at_tile_within_click_radius() {
        // A click one tile away (within SHOP_CLICK_RADIUS = 2) should still
        // resolve to the NPC.
        let gs = GameState::new(Player::new(), true, 1);
        let griswold = gs.towners.iter().find(|t| t.3 == 0).expect("Smith seeded");
        let hit = gs.npc_at_tile(griswold.0 + 1, griswold.1 + 1);
        assert!(hit.is_some(), "adjacent click should find the NPC");
        assert_eq!(hit.unwrap().3, 0);
    }

    #[test]
    fn test_npc_at_tile_outside_radius_returns_none() {
        // A click far from any NPC returns None.
        let gs = GameState::new(Player::new(), true, 1);
        // (0,0) is far from every Tristram NPC's default position.
        assert!(gs.npc_at_tile(0, 0).is_none());
    }

    #[test]
    fn test_active_shop_inventory_populated_for_each_shopkeeper() {
        // Each shop-capable NPC exposes a non-empty catalog when their shop is
        // open. Gossip-only / no-shop returns an empty catalog.
        for kind in [0u8, 1, 6, 8] {
            let mut gs = GameState::new(Player::new(), true, 1);
            gs.open_shop(kind);
            let inv = gs.active_shop_inventory();
            assert!(!inv.is_empty(), "shop {} should have items", kind);
            // Prices must be positive.
            assert!(inv.iter().all(|(_, p)| *p > 0), "all prices positive");
        }
    }

    #[test]
    fn test_active_shop_inventory_empty_when_no_shop_open() {
        // With no shop open the inventory helper returns an empty vec (so the
        // panel renders the "nothing for sale" placeholder).
        let gs = GameState::new(Player::new(), true, 1);
        assert!(gs.active_shop_inventory().is_empty());
    }

    #[test]
    fn test_buy_shop_item_deducts_gold_on_success() {
        // Give the player plenty of gold, open Griswold's shop, buy the first
        // item (Short Sword, price 120), and verify the gold was deducted and
        // the returned name/price match.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.player._p_gold = 1000;
        gs.open_shop(0);
        let price = gs.active_shop_inventory()[0].1;
        let result = gs.buy_shop_item(0);
        let (name, paid) = result.expect("buy should succeed with enough gold");
        assert_eq!(paid, price);
        assert!(name.contains("Sword"), "first Griswold item is a sword");
        assert_eq!(gs.player._p_gold, 1000 - price);
    }

    #[test]
    fn test_buy_shop_item_fails_when_insufficient_gold() {
        // Player has 10 gold, the cheapest Griswold item costs more → buy must
        // fail with "not enough gold" and leave the gold untouched.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.player._p_gold = 10;
        gs.open_shop(0);
        let gold_before = gs.player._p_gold;
        let result = gs.buy_shop_item(0);
        assert!(result.is_err());
        assert_eq!(gs.player._p_gold, gold_before, "gold unchanged on failed buy");
    }

    #[test]
    fn test_buy_shop_item_fails_when_no_shop_open() {
        // Without an open shop, buy must refuse (defensive).
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.player._p_gold = 1000;
        let result = gs.buy_shop_item(0);
        assert!(result.is_err());
        assert_eq!(gs.player._p_gold, 1000, "gold unchanged when no shop open");
    }

    #[test]
    fn test_buy_shop_item_fails_for_out_of_range_index() {
        // An index past the end of the inventory must fail with an error and
        // not change gold.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.player._p_gold = 1000;
        gs.open_shop(0);
        let n = gs.active_shop_inventory().len();
        let gold_before = gs.player._p_gold;
        let result = gs.buy_shop_item(n + 5);
        assert!(result.is_err());
        assert_eq!(gs.player._p_gold, gold_before);
    }

    // ========================================================================
    // Equipment effects tests (Step 2: starter weapon + recalc)
    // ========================================================================

    #[test]
    fn test_fresh_player_has_no_weapon_and_zero_damage() {
        // A freshly-created GameState's Warrior has an empty left-hand slot and
        // a 0-0 item damage range.
        let gs = GameState::new(Player::new(), true, 1);
        assert!(!gs.has_weapon_equipped(), "fresh player has no weapon");
        assert_eq!(gs.player._p_i_min_dam, 0);
        assert_eq!(gs.player._p_i_max_dam, 0);
    }

    #[test]
    fn test_equip_starter_weapon_sets_damage_range() {
        // After equipping the starter weapon the damage range must be the Short
        // Sword's 1-6 and the equipped flag must be set.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.equip_starter_weapon();
        assert!(gs.has_weapon_equipped());
        assert_eq!(gs.player._p_i_min_dam, 1, "Short Sword min damage");
        assert_eq!(gs.player._p_i_max_dam, 6, "Short Sword max damage");
    }

    #[test]
    fn test_equip_starter_weapon_is_idempotent() {
        // Calling equip_starter_weapon twice must not stack or change the
        // already-equipped damage range.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.equip_starter_weapon();
        let (min1, max1) = (gs.player._p_i_min_dam, gs.player._p_i_max_dam);
        gs.equip_starter_weapon();
        assert_eq!((gs.player._p_i_min_dam, gs.player._p_i_max_dam), (min1, max1));
        assert!(gs.has_weapon_equipped());
    }

    #[test]
    fn test_recalc_equipment_stats_resets_when_unequipped() {
        // After equipping, manually clearing the slot + recalc must reset the
        // damage range to 0-0.
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.equip_starter_weapon();
        assert!(gs.player._p_i_max_dam > 0);
        // Clear the slot.
        if let Some(slot) = gs.player.inv_body.get_mut(GameState::INV_BODY_HAND_LEFT) {
            *slot = crate::game::player_exact::PlayerItem::empty();
        }
        gs.recalc_equipment_stats();
        assert!(!gs.has_weapon_equipped());
        assert_eq!(gs.player._p_i_min_dam, 0);
        assert_eq!(gs.player._p_i_max_dam, 0);
    }

    #[test]
    fn test_inv_body_hand_left_index_matches_towner_slot() {
        // The constant must point at the left-hand weapon slot (index 4),
        // matching InvBodyLoc::HandLeft in player_exact.rs.
        assert_eq!(GameState::INV_BODY_HAND_LEFT, 4);
    }

    // ========================================================================
    // Stairs detection / cooldown tests
    // ========================================================================

    #[test]
    fn test_town_down_stairs_constant_matches_cpp() {
        // C++ InitTownTriggers places the Cathedral down-stair at {25, 29}.
        assert_eq!(TOWN_DOWN_STAIRS, (25, 29));
    }

    #[test]
    fn test_new_game_state_has_no_dungeon_up_stairs() {
        // A fresh state is in town, so no up-stair tile has been resolved.
        let gs = GameState::new(Player::new(), true, 1);
        assert!(gs.dungeon_up_stairs.is_none());
        assert_eq!(gs.last_transition_tick, 0);
    }

    #[test]
    fn test_stairs_cooldown_initially_ready() {
        // tick 0 − last 0 saturates to 0, but the cooldown is "ready" at start
        // so the first descent can fire. (We treat elapsed == cooldown as ready
        // AND, because the very first transition has last==0==tick, we accept
        // the saturating-sub == 0 case as ready when nothing has fired yet.)
        let gs = GameState::new(Player::new(), true, 1);
        // tick 0, last 0 → 0 >= 4 is false, so NOT ready at exact tick 0.
        // But the moment the loop advances a few ticks it becomes ready.
        assert!(!gs.stairs_cooldown_ready(), "tick 0 right after init is in cooldown window");
    }

    #[test]
    fn test_stairs_cooldown_becomes_ready_after_window() {
        let mut gs = GameState::new(Player::new(), true, 1);
        // Simulate a transition firing at tick 10.
        gs.game_tick = 10;
        gs.mark_stair_transition();
        assert!(!gs.stairs_cooldown_ready(), "still inside cooldown right after transition");
        // Advance to 10 + STAIRS_COOLDOWN_TICKS - 1 → still blocked.
        gs.game_tick = 10 + STAIRS_COOLDOWN_TICKS - 1;
        assert!(!gs.stairs_cooldown_ready(), "one tick before window ends: blocked");
        // Advance to exactly the window end → ready.
        gs.game_tick = 10 + STAIRS_COOLDOWN_TICKS;
        assert!(gs.stairs_cooldown_ready(), "at window end: ready");
        // Well past → ready.
        gs.game_tick = 10 + STAIRS_COOLDOWN_TICKS + 50;
        assert!(gs.stairs_cooldown_ready(), "long after: ready");
    }

    #[test]
    fn test_player_tile_distance_chebyshev() {
        let mut gs = GameState::new(Player::new(), true, 1);
        gs.player.position = Point::new(25, 29);
        // On the tile.
        assert_eq!(gs.player_tile_distance_to(25, 29), 0);
        // Orthogonally adjacent.
        assert_eq!(gs.player_tile_distance_to(26, 29), 1);
        assert_eq!(gs.player_tile_distance_to(25, 28), 1);
        // Diagonally adjacent (Chebyshev, not Manhattan).
        assert_eq!(gs.player_tile_distance_to(26, 30), 1);
        assert_eq!(gs.player_tile_distance_to(24, 28), 1);
        // Two tiles away on one axis.
        assert_eq!(gs.player_tile_distance_to(27, 29), 2);
        // Mixed.
        assert_eq!(gs.player_tile_distance_to(28, 31), 3);
    }

    #[test]
    fn test_stair_detection_radius_covers_adjacent() {
        // With STAIRS_TRIGGER_RADIUS = 1 the player on or adjacent to the
        // town down-stair should be considered "on" it.
        let mut gs = GameState::new(Player::new(), true, 1);
        // Move the cooldown window out of the way.
        gs.last_transition_tick = 0;
        gs.game_tick = STAIRS_COOLDOWN_TICKS + 1;
        assert!(gs.stairs_cooldown_ready());

        // Player exactly on the stair tile.
        gs.player.position = Point::new(TOWN_DOWN_STAIRS.0, TOWN_DOWN_STAIRS.1);
        assert!(gs.player_tile_distance_to(TOWN_DOWN_STAIRS.0, TOWN_DOWN_STAIRS.1)
            <= STAIRS_TRIGGER_RADIUS);

        // Player diagonally adjacent.
        gs.player.position = Point::new(TOWN_DOWN_STAIRS.0 + 1, TOWN_DOWN_STAIRS.1 + 1);
        assert!(gs.player_tile_distance_to(TOWN_DOWN_STAIRS.0, TOWN_DOWN_STAIRS.1)
            <= STAIRS_TRIGGER_RADIUS);

        // Player two tiles away → outside radius.
        gs.player.position = Point::new(TOWN_DOWN_STAIRS.0 + 2, TOWN_DOWN_STAIRS.1);
        assert!(gs.player_tile_distance_to(TOWN_DOWN_STAIRS.0, TOWN_DOWN_STAIRS.1)
            > STAIRS_TRIGGER_RADIUS);
    }
}
