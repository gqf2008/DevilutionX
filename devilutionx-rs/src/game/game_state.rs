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
use rand::Rng;

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
        }
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
