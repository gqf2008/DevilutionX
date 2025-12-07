//! World/level management - exact port from DevilutionX
//!
//! This module provides the World struct which combines level data,
//! players, monsters, objects, and items into a single game state container.
//! It wraps the exact LevelData from level_new.rs with game entity management.

#![allow(dead_code)]

use super::level_new::{
    DungeonFlags, DungeonType, Difficulty, LevelData, SetLevel,
    MAXDUNX, MAXDUNY,
};
use super::{Player, Monster};

/// Maximum number of players
pub const MAX_PLRS: usize = 4;

/// World state containing all game data for a level
/// This is the high-level container that owns level data and entities
pub struct World {
    /// Level/dungeon data (tiles, flags, lighting, etc.)
    pub level: LevelData,

    /// Players in the game
    pub players: Vec<Player>,

    /// Monsters in the current level
    pub monsters: Vec<Monster>,

    /// Active player index (for single player, this is 0)
    pub my_player_num: usize,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            level: LevelData::new(),
            players: Vec::new(),
            monsters: Vec::new(),
            my_player_num: 0,
        }
    }

    /// Create world for a specific level
    pub fn for_level(level_num: u8, difficulty: Difficulty) -> Self {
        let mut world = Self::new();
        world.level.current_level = level_num;
        world.level.level_type = DungeonType::from_level(level_num);
        world.level.difficulty = difficulty;
        world
    }

    /// Create world for a set/quest level
    pub fn for_set_level(set_level: SetLevel, dungeon_type: DungeonType) -> Self {
        let mut world = Self::new();
        world.level.is_set_level = true;
        world.level.set_level_num = set_level;
        world.level.set_level_type = dungeon_type;
        world.level.level_type = dungeon_type;
        world
    }

    // ========================================================================
    // Tile access helpers (delegate to LevelData)
    // ========================================================================

    /// Check if position is within dungeon bounds
    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.level.in_dungeon_bounds(x, y)
    }

    /// Check if tile is solid (blocks movement)
    #[inline]
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        self.level.is_solid(x, y)
    }

    /// Check if tile blocks missiles
    #[inline]
    pub fn blocks_missile(&self, x: i32, y: i32) -> bool {
        self.level.blocks_missile(x, y)
    }

    /// Check if tile is visible to any player
    pub fn is_tile_visible(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        self.level.dflags[x as usize][y as usize].contains(DungeonFlags::VISIBLE)
    }

    /// Check if tile contains a missile
    pub fn tile_contains_missile(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        self.level.dflags[x as usize][y as usize].contains(DungeonFlags::MISSILE)
    }

    /// Check if tile contains a set piece
    pub fn tile_contains_set_piece(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        self.level.dflags[x as usize][y as usize].contains(DungeonFlags::POPULATED)
    }

    /// Check if tile has been explored
    pub fn is_tile_explored(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        self.level.dflags[x as usize][y as usize].contains(DungeonFlags::EXPLORED)
    }

    /// Check if tile is lit
    pub fn is_tile_lit(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        self.level.dflags[x as usize][y as usize].contains(DungeonFlags::LIT)
    }

    // ========================================================================
    // Entity position tracking
    // ========================================================================

    /// Get player at position (if any)
    pub fn get_player_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        let player_id = self.level.dplayer[x as usize][y as usize];
        if player_id > 0 {
            Some((player_id - 1) as usize)
        } else if player_id < 0 {
            // Negative means player is moving, but still there
            Some(((-player_id) - 1) as usize)
        } else {
            None
        }
    }

    /// Get monster at position (if any)
    pub fn get_monster_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        let monster_id = self.level.dmonster[x as usize][y as usize];
        if monster_id > 0 {
            Some((monster_id - 1) as usize)
        } else if monster_id < 0 {
            // Negative means monster is moving
            Some(((-monster_id) - 1) as usize)
        } else {
            None
        }
    }

    /// Get object at position (if any)
    pub fn get_object_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        let object_id = self.level.dobject[x as usize][y as usize];
        if object_id > 0 {
            Some((object_id - 1) as usize)
        } else if object_id < 0 {
            // Negative means large object extended area
            Some(((-object_id) - 1) as usize)
        } else {
            None
        }
    }

    /// Get item at position (if any)
    pub fn get_item_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        let item_id = self.level.ditem[x as usize][y as usize];
        if item_id > 0 {
            Some((item_id - 1) as usize)
        } else {
            None
        }
    }

    /// Set player position in dplayer array
    pub fn set_player_position(&mut self, player_idx: usize, x: i32, y: i32, is_moving: bool) {
        if !self.in_bounds(x, y) {
            return;
        }
        let id = (player_idx + 1) as i8;
        self.level.dplayer[x as usize][y as usize] = if is_moving { -id } else { id };
    }

    /// Clear player from position
    pub fn clear_player_position(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) {
            self.level.dplayer[x as usize][y as usize] = 0;
        }
    }

    /// Set monster position in dmonster array
    pub fn set_monster_position(&mut self, monster_idx: usize, x: i32, y: i32, is_moving: bool) {
        if !self.in_bounds(x, y) {
            return;
        }
        let id = (monster_idx + 1) as i16;
        self.level.dmonster[x as usize][y as usize] = if is_moving { -id } else { id };
    }

    /// Clear monster from position
    pub fn clear_monster_position(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) {
            self.level.dmonster[x as usize][y as usize] = 0;
        }
    }

    // ========================================================================
    // Visibility and lighting
    // ========================================================================

    /// Set tile visibility flag
    pub fn set_tile_visible(&mut self, x: i32, y: i32, visible: bool) {
        if !self.in_bounds(x, y) {
            return;
        }
        if visible {
            self.level.dflags[x as usize][y as usize].set(DungeonFlags::VISIBLE);
        } else {
            self.level.dflags[x as usize][y as usize].clear(DungeonFlags::VISIBLE);
        }
    }

    /// Set tile explored flag
    pub fn set_tile_explored(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) {
            self.level.dflags[x as usize][y as usize].set(DungeonFlags::EXPLORED);
        }
    }

    /// Get light level at position (0-15)
    pub fn get_light_level(&self, x: i32, y: i32) -> u8 {
        if !self.in_bounds(x, y) {
            return 0;
        }
        self.level.dlight[x as usize][y as usize]
    }

    /// Set light level at position
    pub fn set_light_level(&mut self, x: i32, y: i32, level: u8) {
        if self.in_bounds(x, y) {
            self.level.dlight[x as usize][y as usize] = level.min(15);
        }
    }

    // ========================================================================
    // Game update
    // ========================================================================

    /// Update all monsters in the world
    pub fn update(&mut self, player_x: i32, player_y: i32) {
        // First collect visibility info to avoid borrow conflicts
        let visibility: Vec<bool> = self.monsters.iter()
            .map(|m| self.level.dflags.get(m.x as usize)
                .and_then(|row| row.get(m.y as usize))
                .map(|f| f.contains(DungeonFlags::VISIBLE))
                .unwrap_or(false))
            .collect();

        // Update all monsters
        for (monster, can_see) in self.monsters.iter_mut().zip(visibility.iter()) {
            monster.update_ai(player_x, player_y, *can_see);
        }
    }

    /// Clear all visibility flags (called at start of each frame)
    pub fn clear_visibility(&mut self) {
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.level.dflags[x][y].clear(DungeonFlags::VISIBLE);
            }
        }
    }

    /// Clear all entity positions (players, monsters, items)
    pub fn clear_entity_positions(&mut self) {
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.level.dplayer[x][y] = 0;
                self.level.dmonster[x][y] = 0;
                self.level.ditem[x][y] = 0;
            }
        }
    }

    // ========================================================================
    // Level info getters
    // ========================================================================

    /// Get current level number
    pub fn current_level(&self) -> u8 {
        self.level.current_level
    }

    /// Get current dungeon type
    pub fn dungeon_type(&self) -> DungeonType {
        self.level.level_type
    }

    /// Get current difficulty
    pub fn difficulty(&self) -> Difficulty {
        self.level.difficulty
    }

    /// Check if this is a set/quest level
    pub fn is_set_level(&self) -> bool {
        self.level.is_set_level
    }

    /// Get set level type
    pub fn set_level_num(&self) -> SetLevel {
        self.level.set_level_num
    }

    /// Check if in town
    pub fn is_town(&self) -> bool {
        self.level.current_level == 0 || self.level.level_type == DungeonType::Town
    }

    /// Get dungeon dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (MAXDUNX, MAXDUNY)
    }

    /// Get view position
    pub fn view_position(&self) -> (i32, i32) {
        self.level.view_position
    }

    /// Set view position
    pub fn set_view_position(&mut self, x: i32, y: i32) {
        self.level.view_position = (x, y);
    }
}
