/// Core dungeon generation module
///
/// Ported from Source/levels/gendung.cpp and gendung.h
///
/// This module provides the fundamental dungeon data structures and operations
/// for Diablo's procedural level generation system.

use crate::engine::Point;
use crate::levels::types::*;
use std::collections::HashMap;

// ============================================================================
// Dungeon Structure
// ============================================================================

/// Main dungeon data structure
///
/// This represents the complete state of a generated dungeon level.
///
/// C++ equivalent: Multiple global arrays (dungeon[][], dPiece[][], dFlags[][], etc.)
#[derive(Clone)]
pub struct Dungeon {
    /// Current dungeon type (Town, Cathedral, etc.)
    pub level_type: DungeonType,

    /// Current level number (0 = Town, 1-16 = dungeon levels)
    pub current_level: u8,

    /// Tile IDs (what type of tile is at each position)
    /// C++ equivalent: `dungeon[DMAXX][DMAXY]`
    pub tiles: [[u8; DMAXY]; DMAXX],

    /// Backup of tiles (used during generation)
    /// C++ equivalent: `pdungeon[DMAXX][DMAXY]`
    pub tiles_backup: [[u8; DMAXY]; DMAXX],

    /// Dungeon flags (visibility, lighting, etc.)
    /// C++ equivalent: `dFlags[MAXDUNX][MAXDUNY]`
    pub flags: [[DungeonFlag; MAXDUNY]; MAXDUNX],

    /// Piece IDs (micro-tile indices for rendering)
    /// C++ equivalent: `dPiece[MAXDUNX][MAXDUNY]`
    pub pieces: [[u16; MAXDUNY]; MAXDUNX],

    /// Transparency values for lighting
    /// C++ equivalent: `dTransVal[MAXDUNX][MAXDUNY]`
    pub trans_val: [[i8; MAXDUNY]; MAXDUNX],

    /// Current light levels
    /// C++ equivalent: `dLight[MAXDUNX][MAXDUNY]`
    pub light: [[u8; MAXDUNY]; MAXDUNX],

    /// Pre-calculated static lights
    /// C++ equivalent: `dPreLight[MAXDUNX][MAXDUNY]`
    pub pre_light: [[u8; MAXDUNY]; MAXDUNX],

    /// Player layer (which player is at each tile)
    /// C++ equivalent: `dPlayer[MAXDUNX][MAXDUNY]`
    pub player_layer: [[i8; MAXDUNY]; MAXDUNX],

    /// Monster layer (which monster is at each tile)
    /// C++ equivalent: `dMonster[MAXDUNX][MAXDUNY]`
    pub monster_layer: [[i16; MAXDUNY]; MAXDUNX],

    /// Corpse layer (dead player corpses)
    /// C++ equivalent: `dCorpse[MAXDUNX][MAXDUNY]`
    pub corpse_layer: [[i8; MAXDUNY]; MAXDUNX],

    /// Object layer (which object is at each tile)
    /// C++ equivalent: `dObject[MAXDUNX][MAXDUNY]`
    pub object_layer: [[i8; MAXDUNY]; MAXDUNX],

    /// Special tile layer (arches, trees)
    /// C++ equivalent: `dSpecial[MAXDUNX][MAXDUNY]`
    pub special_layer: [[i8; MAXDUNY]; MAXDUNX],

    /// Minimum dungeon coordinates (usable area)
    /// C++ equivalent: `dminPosition`
    pub min_position: Point,

    /// Maximum dungeon coordinates (usable area)
    /// C++ equivalent: `dmaxPosition`
    pub max_position: Point,

    /// Theme room locations
    /// C++ equivalent: `themeLoc[MAXTHEMES]`
    pub theme_locations: Vec<ThemeLocation>,

    /// Number of active theme rooms
    /// C++ equivalent: `themeCount`
    pub theme_count: usize,

    /// Protected tiles (cannot be overwritten by generator)
    /// C++ equivalent: `Protected` bitset
    pub protected: [[bool; DMAXY]; DMAXX],

    /// Dungeon mask (which tiles are in use)
    /// C++ equivalent: `DungeonMask` bitset
    pub mask: [[bool; DMAXY]; DMAXX],
}

impl Dungeon {
    /// Create a new empty dungeon
    pub fn new() -> Self {
        Self {
            level_type: DungeonType::Town,
            current_level: 0,
            tiles: [[0; DMAXY]; DMAXX],
            tiles_backup: [[0; DMAXY]; DMAXX],
            flags: [[DungeonFlag::NONE; MAXDUNY]; MAXDUNX],
            pieces: [[0; MAXDUNY]; MAXDUNX],
            trans_val: [[0; MAXDUNY]; MAXDUNX],
            light: [[0; MAXDUNY]; MAXDUNX],
            pre_light: [[0; MAXDUNY]; MAXDUNX],
            player_layer: [[0; MAXDUNY]; MAXDUNX],
            monster_layer: [[0; MAXDUNY]; MAXDUNX],
            corpse_layer: [[0; MAXDUNY]; MAXDUNX],
            object_layer: [[0; MAXDUNY]; MAXDUNX],
            special_layer: [[0; MAXDUNY]; MAXDUNX],
            min_position: Point::new(0, 0),
            max_position: Point::new(MAXDUNX as i32, MAXDUNY as i32),
            theme_locations: vec![ThemeLocation::new(); MAXTHEMES],
            theme_count: 0,
            protected: [[false; DMAXY]; DMAXX],
            mask: [[false; DMAXY]; DMAXX],
        }
    }

    /// Reset light/vision grids to the C++ level default.
    ///
    /// Mirrors `Source/levels/gendung.cpp` `InitGlobals()`:
    /// ```cpp
    /// uint8_t defaultLight = leveltype == DTYPE_TOWN ? 0 : 15;
    /// memset(dLight, defaultLight, sizeof(dLight));
    /// ```
    /// Town is fully bright (0); every dungeon level starts fully dark (15)
    /// and static level lights (lava, braziers, ...) later raise tiles via
    /// `DoLighting` before `SavePreLighting()` snapshots `dPreLight`.
    /// `trans_val` is zeroed like C++ `DRLG_InitTrans()`.
    pub fn init_lighting_defaults(&mut self) {
        let default_light: u8 = if self.level_type == DungeonType::Town { 0 } else { 15 };
        for y in 0..MAXDUNY {
            for x in 0..MAXDUNX {
                self.light[x][y] = default_light;
                self.pre_light[x][y] = default_light;
                self.trans_val[x][y] = 0;
            }
        }
    }

    /// Reset all dungeon data
    ///
    /// C++ equivalent: Part of `CreateDungeon()` initialization
    pub fn reset(&mut self) {
        self.tiles = [[0; DMAXY]; DMAXX];
        self.tiles_backup = [[0; DMAXY]; DMAXX];
        self.flags = [[DungeonFlag::NONE; MAXDUNY]; MAXDUNX];
        self.pieces = [[0; MAXDUNY]; MAXDUNX];
        self.trans_val = [[0; MAXDUNY]; MAXDUNX];
        self.light = [[0; MAXDUNY]; MAXDUNX];
        self.pre_light = [[0; MAXDUNY]; MAXDUNX];
        self.init_lighting_defaults();
        self.player_layer = [[0; MAXDUNY]; MAXDUNX];
        self.monster_layer = [[0; MAXDUNY]; MAXDUNX];
        self.corpse_layer = [[0; MAXDUNY]; MAXDUNX];
        self.object_layer = [[0; MAXDUNY]; MAXDUNX];
        self.special_layer = [[0; MAXDUNY]; MAXDUNX];
        self.theme_locations = vec![ThemeLocation::new(); MAXTHEMES];
        self.theme_count = 0;
        self.protected = [[false; DMAXY]; DMAXX];
        self.mask = [[false; DMAXY]; DMAXX];
    }

    /// Initialize dungeon flags
    ///
    /// C++ equivalent: Part of `CreateDungeon()` - initializes dFlags
    pub fn init_flags(&mut self) {
        for y in 0..MAXDUNY {
            for x in 0..MAXDUNX {
                self.flags[x][y] = DungeonFlag::NONE;
            }
        }
    }

    /// Set a tile at the given position
    ///
    /// C++ equivalent: Direct array access `dungeon[x][y] = tile_id`
    #[inline]
    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u8) {
        if x < DMAXX && y < DMAXY {
            self.tiles[x][y] = tile_id;
        }
    }

    /// Get a tile at the given position
    ///
    /// C++ equivalent: Direct array access `dungeon[x][y]`
    #[inline]
    pub fn get_tile(&self, x: usize, y: usize) -> u8 {
        if x < DMAXX && y < DMAXY {
            self.tiles[x][y]
        } else {
            0
        }
    }

    /// Set a piece (micro-tile) at the given position
    ///
    /// C++ equivalent: Direct array access `dPiece[x][y] = piece_id`
    #[inline]
    pub fn set_piece(&mut self, x: usize, y: usize, piece_id: u16) {
        if x < MAXDUNX && y < MAXDUNY {
            self.pieces[x][y] = piece_id;
        }
    }

    /// Get a piece (micro-tile) at the given position
    ///
    /// C++ equivalent: Direct array access `dPiece[x][y]`
    #[inline]
    pub fn get_piece(&self, x: usize, y: usize) -> u16 {
        if x < MAXDUNX && y < MAXDUNY {
            self.pieces[x][y]
        } else {
            0
        }
    }

    /// Set flags at the given position
    #[inline]
    pub fn set_flags(&mut self, x: usize, y: usize, flags: DungeonFlag) {
        if x < MAXDUNX && y < MAXDUNY {
            self.flags[x][y] = flags;
        }
    }

    /// Get flags at the given position
    #[inline]
    pub fn get_flags(&self, x: usize, y: usize) -> DungeonFlag {
        if x < MAXDUNX && y < MAXDUNY {
            self.flags[x][y]
        } else {
            DungeonFlag::NONE
        }
    }

    /// Add flags to a tile
    #[inline]
    pub fn add_flags(&mut self, x: usize, y: usize, flags: DungeonFlag) {
        if x < MAXDUNX && y < MAXDUNY {
            self.flags[x][y] |= flags;
        }
    }

    /// Remove flags from a tile
    #[inline]
    pub fn remove_flags(&mut self, x: usize, y: usize, flags: DungeonFlag) {
        if x < MAXDUNX && y < MAXDUNY {
            self.flags[x][y] &= !flags;
        }
    }

    /// Check if a tile has specific flags
    #[inline]
    pub fn has_flags(&self, x: usize, y: usize, flags: DungeonFlag) -> bool {
        if x < MAXDUNX && y < MAXDUNY {
            self.flags[x][y].contains(flags)
        } else {
            false
        }
    }

    /// Check if tile contains a missile
    ///
    /// C++ equivalent: `TileContainsMissile()`
    #[inline]
    pub fn tile_contains_missile(&self, pos: Point) -> bool {
        in_dungeon_bounds_point(pos) && self.has_flags(pos.x as usize, pos.y as usize, DungeonFlag::MISSILE)
    }

    /// Check if tile contains a dead player
    ///
    /// C++ equivalent: `TileContainsDeadPlayer()`
    #[inline]
    pub fn tile_contains_dead_player(&self, pos: Point) -> bool {
        in_dungeon_bounds_point(pos) && self.has_flags(pos.x as usize, pos.y as usize, DungeonFlag::DEAD_PLAYER)
    }

    /// Check if tile contains a set piece (stairs, etc.)
    ///
    /// C++ equivalent: `TileContainsSetPiece()`
    #[inline]
    pub fn tile_contains_set_piece(&self, pos: Point) -> bool {
        in_dungeon_bounds_point(pos) && self.has_flags(pos.x as usize, pos.y as usize, DungeonFlag::POPULATED)
    }

    /// Check if tile is visible to any player
    ///
    /// C++ equivalent: `IsTileVisible()`
    #[inline]
    pub fn is_tile_visible(&self, pos: Point) -> bool {
        in_dungeon_bounds_point(pos) && self.has_flags(pos.x as usize, pos.y as usize, DungeonFlag::VISIBLE)
    }

    /// Check if tile is lit
    ///
    /// C++ equivalent: `IsTileLit()`
    #[inline]
    pub fn is_tile_lit(&self, pos: Point) -> bool {
        in_dungeon_bounds_point(pos) && self.has_flags(pos.x as usize, pos.y as usize, DungeonFlag::LIT)
    }

    /// Protect a tile from being overwritten
    #[inline]
    pub fn protect_tile(&mut self, x: usize, y: usize) {
        if x < DMAXX && y < DMAXY {
            self.protected[x][y] = true;
        }
    }

    /// Check if a tile is protected
    #[inline]
    pub fn is_tile_protected(&self, x: usize, y: usize) -> bool {
        if x < DMAXX && y < DMAXY {
            self.protected[x][y]
        } else {
            false
        }
    }

    /// Set dungeon mask bit
    #[inline]
    pub fn set_mask(&mut self, x: usize, y: usize, value: bool) {
        if x < DMAXX && y < DMAXY {
            self.mask[x][y] = value;
        }
    }

    /// Get dungeon mask bit
    #[inline]
    pub fn get_mask(&self, x: usize, y: usize) -> bool {
        if x < DMAXX && y < DMAXY {
            self.mask[x][y]
        } else {
            false
        }
    }

    /// Backup current tiles
    ///
    /// C++ equivalent: `memcpy(pdungeon, dungeon, sizeof(dungeon))`
    pub fn backup_tiles(&mut self) {
        self.tiles_backup = self.tiles;
    }

    /// Restore tiles from backup
    ///
    /// C++ equivalent: `memcpy(dungeon, pdungeon, sizeof(dungeon))`
    pub fn restore_tiles(&mut self) {
        self.tiles = self.tiles_backup;
    }
}

impl Default for Dungeon {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tile Property Manager
// ============================================================================

/// Manages tile properties for a dungeon level
///
/// C++ equivalent: `SOLData[MAXTILES]` global array
pub struct TilePropertyManager {
    /// Tile properties indexed by tile ID
    properties: [TileProperties; MAXTILES],
}

impl TilePropertyManager {
    /// Create a new tile property manager
    pub fn new() -> Self {
        Self {
            properties: [TileProperties::NONE; MAXTILES],
        }
    }

    /// Get properties for a tile
    #[inline]
    pub fn get_properties(&self, tile_id: u16) -> TileProperties {
        if (tile_id as usize) < MAXTILES {
            self.properties[tile_id as usize]
        } else {
            TileProperties::NONE
        }
    }

    /// Set properties for a tile
    #[inline]
    pub fn set_properties(&mut self, tile_id: u16, props: TileProperties) {
        if (tile_id as usize) < MAXTILES {
            self.properties[tile_id as usize] = props;
        }
    }

    /// Check if a tile is solid (cannot walk through)
    #[inline]
    pub fn is_solid(&self, tile_id: u16) -> bool {
        self.get_properties(tile_id).contains(TileProperties::SOLID)
    }

    /// Check if a tile blocks light
    #[inline]
    pub fn blocks_light(&self, tile_id: u16) -> bool {
        self.get_properties(tile_id).contains(TileProperties::BLOCK_LIGHT)
    }

    /// Check if a tile blocks missiles
    #[inline]
    pub fn blocks_missile(&self, tile_id: u16) -> bool {
        self.get_properties(tile_id).contains(TileProperties::BLOCK_MISSILE)
    }

    /// Check if a tile blocks players
    #[inline]
    pub fn blocks_player(&self, tile_id: u16) -> bool {
        self.get_properties(tile_id).contains(TileProperties::BLOCK_PLAYER)
    }

    /// Check if a tile blocks monsters
    #[inline]
    pub fn blocks_monster(&self, tile_id: u16) -> bool {
        self.get_properties(tile_id).contains(TileProperties::BLOCK_MONSTER)
    }
}

impl Default for TilePropertyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_new() {
        let dungeon = Dungeon::new();
        assert_eq!(dungeon.level_type, DungeonType::Town);
        assert_eq!(dungeon.current_level, 0);
        assert_eq!(dungeon.theme_count, 0);
    }

    #[test]
    fn test_dungeon_reset() {
        let mut dungeon = Dungeon::new();
        dungeon.set_tile(10, 10, 42);
        dungeon.theme_count = 5;
        dungeon.reset();
        assert_eq!(dungeon.get_tile(10, 10), 0);
        assert_eq!(dungeon.theme_count, 0);
    }

    #[test]
    fn test_set_get_tile() {
        let mut dungeon = Dungeon::new();
        // Use a coordinate inside the active region (DMAXX/DMAXY == 40).
        dungeon.set_tile(20, 20, 123);
        assert_eq!(dungeon.get_tile(20, 20), 123);
    }

    #[test]
    fn test_set_get_piece() {
        let mut dungeon = Dungeon::new();
        dungeon.set_piece(50, 50, 999);
        assert_eq!(dungeon.get_piece(50, 50), 999);
    }

    #[test]
    fn test_tile_flags() {
        let mut dungeon = Dungeon::new();

        // Initially no flags
        assert!(!dungeon.has_flags(10, 10, DungeonFlag::LIT));

        // Add LIT flag
        dungeon.add_flags(10, 10, DungeonFlag::LIT);
        assert!(dungeon.has_flags(10, 10, DungeonFlag::LIT));

        // Add EXPLORED flag
        dungeon.add_flags(10, 10, DungeonFlag::EXPLORED);
        assert!(dungeon.has_flags(10, 10, DungeonFlag::LIT));
        assert!(dungeon.has_flags(10, 10, DungeonFlag::EXPLORED));

        // Remove LIT flag
        dungeon.remove_flags(10, 10, DungeonFlag::LIT);
        assert!(!dungeon.has_flags(10, 10, DungeonFlag::LIT));
        assert!(dungeon.has_flags(10, 10, DungeonFlag::EXPLORED));
    }

    #[test]
    fn test_tile_queries() {
        let mut dungeon = Dungeon::new();
        let pos = Point::new(20, 20);

        // Test missile detection
        dungeon.add_flags(20, 20, DungeonFlag::MISSILE);
        assert!(dungeon.tile_contains_missile(pos));

        // Test dead player detection
        dungeon.add_flags(20, 20, DungeonFlag::DEAD_PLAYER);
        assert!(dungeon.tile_contains_dead_player(pos));

        // Test set piece detection
        dungeon.add_flags(20, 20, DungeonFlag::POPULATED);
        assert!(dungeon.tile_contains_set_piece(pos));

        // Test visibility
        dungeon.add_flags(20, 20, DungeonFlag::VISIBLE);
        assert!(dungeon.is_tile_visible(pos));

        // Test lighting
        dungeon.add_flags(20, 20, DungeonFlag::LIT);
        assert!(dungeon.is_tile_lit(pos));
    }

    #[test]
    fn test_protect_tile() {
        let mut dungeon = Dungeon::new();
        assert!(!dungeon.is_tile_protected(15, 15));
        dungeon.protect_tile(15, 15);
        assert!(dungeon.is_tile_protected(15, 15));
    }

    #[test]
    fn test_dungeon_mask() {
        let mut dungeon = Dungeon::new();
        assert!(!dungeon.get_mask(25, 25));
        dungeon.set_mask(25, 25, true);
        assert!(dungeon.get_mask(25, 25));
    }

    #[test]
    fn test_backup_restore_tiles() {
        let mut dungeon = Dungeon::new();
        dungeon.set_tile(30, 30, 99);
        dungeon.backup_tiles();
        dungeon.set_tile(30, 30, 88);
        assert_eq!(dungeon.get_tile(30, 30), 88);
        dungeon.restore_tiles();
        assert_eq!(dungeon.get_tile(30, 30), 99);
    }

    #[test]
    fn test_bounds_checking() {
        let mut dungeon = Dungeon::new();

        // Out of bounds should not panic and should return default values
        dungeon.set_tile(200, 200, 42);
        assert_eq!(dungeon.get_tile(200, 200), 0);

        dungeon.set_piece(200, 200, 42);
        assert_eq!(dungeon.get_piece(200, 200), 0);
    }

    #[test]
    fn test_tile_property_manager() {
        let mut manager = TilePropertyManager::new();

        // Set properties for tile 100
        manager.set_properties(100, TileProperties::SOLID | TileProperties::BLOCK_LIGHT);

        assert!(manager.is_solid(100));
        assert!(manager.blocks_light(100));
        assert!(!manager.blocks_missile(100));
        assert!(!manager.blocks_player(100));
    }

    #[test]
    fn test_tile_property_queries() {
        let mut manager = TilePropertyManager::new();

        // Wall tile: solid, blocks everything
        manager.set_properties(1,
            TileProperties::SOLID |
            TileProperties::BLOCK_LIGHT |
            TileProperties::BLOCK_PLAYER |
            TileProperties::BLOCK_MONSTER |
            TileProperties::BLOCK_MISSILE
        );

        assert!(manager.is_solid(1));
        assert!(manager.blocks_light(1));
        assert!(manager.blocks_player(1));
        assert!(manager.blocks_monster(1));
        assert!(manager.blocks_missile(1));

        // Floor tile: walkable
        manager.set_properties(2, TileProperties::NONE);
        assert!(!manager.is_solid(2));
        assert!(!manager.blocks_light(2));
    }

    #[test]
    fn test_out_of_bounds_tile_properties() {
        let manager = TilePropertyManager::new();

        // Should return NONE for out of bounds
        assert_eq!(manager.get_properties(9999), TileProperties::NONE);
        assert!(!manager.is_solid(9999));
    }
    #[test]
    fn test_lighting_defaults_match_cpp_default_light() {
        // C++ gendung.cpp: `uint8_t defaultLight = leveltype == DTYPE_TOWN ? 0 : 15;`
        let mut town = Dungeon::new(); // level_type == Town
        town.init_lighting_defaults();
        assert_eq!(town.light[10][10], 0, "town is fully bright (0)");
        assert_eq!(town.pre_light[10][10], 0, "town dPreLight is fully bright");

        let mut cathedral = Dungeon::new();
        cathedral.level_type = DungeonType::Cathedral;
        cathedral.init_lighting_defaults();
        assert_eq!(cathedral.light[10][10], 15, "dungeon starts fully dark (15)");
        assert_eq!(cathedral.pre_light[10][10], 15, "dungeon dPreLight starts fully dark");
        assert_eq!(cathedral.trans_val[10][10], 0, "DRLG_InitTrans zeroes dTransVal");

        // reset() re-applies the defaults for the current level type.
        cathedral.reset();
        assert_eq!(cathedral.light[10][10], 15);
        assert_eq!(cathedral.trans_val[10][10], 0);
    }


}
