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
}

impl Default for DungeonLayout {
    fn default() -> Self {
        use crate::levels::types::MAXDUNX;
        use crate::levels::types::MAXDUNY;
        Self {
            d_piece: vec![0; MAXDUNX * MAXDUNY],
            width: MAXDUNX,
            height: MAXDUNY,
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

        for monster_id in monster_ids {
            if let Some(monster) = self.monster_manager.get_monster_mut(monster_id) {
                let dist = walking_distance(player_pos, monster.position());

                if dist <= 1 {
                    // Player attacks this monster
                    let _ = player_attack_monster(&self.player, monster, rng);
                }
            }
        }
    }

    /// Process monster-door interactions
    fn process_monster_doors(&mut self, monster_id: usize) {
        if let Some(monster) = self.monster_manager.get_monster(monster_id) {
            let _ = monster_check_doors(monster, &mut self.objects);
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
}
