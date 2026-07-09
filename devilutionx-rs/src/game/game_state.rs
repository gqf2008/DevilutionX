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
            dungeon_level_data: None,
            monster_sprites: None,
        }
    }

    /// Initialise the camera/player position to the town spawn (C++ `ViewPosition`
    /// for `ENTRY_MAIN` = {75, 68}). Called by `start_game` after town data is
    /// loaded.
    pub fn init_town_camera(&mut self) {
        // C++ CreateTown ENTRY_MAIN: ViewPosition = { 75, 68 }
        self.camera = Camera { tile_x: 75, tile_y: 68, sub_x: 0, sub_y: 0 };
        self.player.position = Point::new(75, 68);
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

            // Process items (C++ line 1531)
            self.logic_step = GameLogicStep::ProcessItems;
            self.process_items();
        }

        self.logic_step = GameLogicStep::None;
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
        // on monster_manager/player resolve.
        let mut xp_gained: i32 = 0;
        for monster_id in monster_ids {
            if let Some(monster) = self.monster_manager.get_monster_mut(monster_id) {
                let dist = walking_distance(player_pos, monster.position());

                if dist <= 1 {
                    // Player attacks this monster
                    match player_attack_monster(&self.player, monster, rng) {
                        crate::game::combat_integration::AttackResult::Kill { .. } => {
                            // Award monster XP on kill (monster_dat xp reward).
                            xp_gained += monster.experience as i32;
                        }
                        _ => {}
                    }
                }
            }
        }

        if xp_gained > 0 {
            self.player._p_experience = self.player._p_experience.saturating_add(xp_gained as u32);
            // Level-up check: advance while XP exceeds the next level threshold.
            self.check_level_up();
        }
    }

    /// Advance the player's level while their experience exceeds the next
    /// threshold, raising base HP/mana per C++ NextLevel/CalcStats.
    fn check_level_up(&mut self) {
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
    pub fn save_to_slot(&self, slot: u32) -> std::result::Result<String, String> {
        use crate::game::save::{build_save_slot, SaveManager};
        let mgr = SaveManager::new();
        let save = build_save_slot(self, self);
        let path = mgr
            .save_slot(slot, &save)
            .map_err(|e| format!("save failed: {}", e))?;
        Ok(path)
    }

    /// Load a `SaveSlot` from disk and apply it to this `GameState` (player +
    /// world). Returns the loaded snapshot for inspection/logging.
    pub fn load_from_slot(&mut self, slot: u32) -> std::result::Result<(), String> {
        use crate::game::save::{SaveManager, PlayerSnapshotMut};
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
