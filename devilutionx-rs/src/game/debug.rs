//! Debug - Debug functions and grid display
//!
//! C++ Reference: Source/debug.cpp, Source/debug.h
//!
//! Contains debugging functionality for development builds.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;

use crate::engine::Point;

/// Number of levels in the game
pub const NUMLEVELS: usize = 25;

/// Test map path for level loading
pub static TEST_MAP_PATH: Mutex<String> = Mutex::new(String::new());

/// Debug toggle flag
pub static DEBUG_TOGGLE: AtomicBool = AtomicBool::new(false);

/// God mode flag
pub static DEBUG_GOD_MODE: AtomicBool = AtomicBool::new(false);

/// Invisibility flag
pub static DEBUG_INVISIBLE: AtomicBool = AtomicBool::new(false);

/// Vision debugging flag
pub static DEBUG_VISION: AtomicBool = AtomicBool::new(false);

/// Path debugging flag
pub static DEBUG_PATH: AtomicBool = AtomicBool::new(false);

/// Grid debugging flag
pub static DEBUG_GRID: AtomicBool = AtomicBool::new(false);

/// Scroll view enabled flag
pub static DEBUG_SCROLL_VIEW_ENABLED: AtomicBool = AtomicBool::new(false);

/// Current debug monster ID
static DEBUG_MONSTER_ID: AtomicI32 = AtomicI32::new(0);

/// Selected debug grid text item
static SELECTED_DEBUG_GRID_TEXT_ITEM: Mutex<DebugGridTextItem> = Mutex::new(DebugGridTextItem::None);

/// Debug coordinates map
pub static DEBUG_COORDS_MAP: Mutex<Option<HashMap<i32, Point>>> = Mutex::new(None);

/// Debug TRN (translation) string
pub static DEBUG_TRN: Mutex<String> = Mutex::new(String::new());

/// Mid1 seeds for level generation debugging
pub static GL_MID1_SEED: Mutex<[u32; NUMLEVELS]> = Mutex::new([0; NUMLEVELS]);

/// Mid2 seeds for level generation debugging
pub static GL_MID2_SEED: Mutex<[u32; NUMLEVELS]> = Mutex::new([0; NUMLEVELS]);

/// Mid3 seeds for level generation debugging
pub static GL_MID3_SEED: Mutex<[u32; NUMLEVELS]> = Mutex::new([0; NUMLEVELS]);

/// End seeds for level generation debugging
pub static GL_END_SEED: Mutex<[u32; NUMLEVELS]> = Mutex::new([0; NUMLEVELS]);

/// Search monsters list for automap highlighting
static SEARCH_MONSTERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Search items list for automap highlighting
static SEARCH_ITEMS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Search objects list for automap highlighting
static SEARCH_OBJECTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Debug grid text items - what to display in the debug grid
///
/// C++ Reference: `DebugGridTextItem`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum DebugGridTextItem {
    #[default]
    None = 0,
    MicroTiles,
    DPiece,
    DTransVal,
    DLight,
    DPreLight,
    DFlags,
    DPlayer,
    DMonster,
    Missiles,
    DCorpse,
    DObject,
    DItem,
    DSpecial,
    
    Coords,
    CursorCoords,
    ObjectIndex,
    
    // Take dPiece as index
    Solid,
    Transparent,
    Trap,
    
    // Megatiles
    AutomapView,
    Dungeon,
    PDungeon,
    Protected,
}

impl DebugGridTextItem {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "microtiles" => Some(Self::MicroTiles),
            "dpiece" => Some(Self::DPiece),
            "dtransval" => Some(Self::DTransVal),
            "dlight" => Some(Self::DLight),
            "dprelight" => Some(Self::DPreLight),
            "dflags" => Some(Self::DFlags),
            "dplayer" => Some(Self::DPlayer),
            "dmonster" => Some(Self::DMonster),
            "missiles" => Some(Self::Missiles),
            "dcorpse" => Some(Self::DCorpse),
            "dobject" => Some(Self::DObject),
            "ditem" => Some(Self::DItem),
            "dspecial" => Some(Self::DSpecial),
            "coords" => Some(Self::Coords),
            "cursorcoords" => Some(Self::CursorCoords),
            "objectindex" => Some(Self::ObjectIndex),
            "solid" => Some(Self::Solid),
            "transparent" => Some(Self::Transparent),
            "trap" => Some(Self::Trap),
            "automapview" => Some(Self::AutomapView),
            "dungeon" => Some(Self::Dungeon),
            "pdungeon" => Some(Self::PDungeon),
            "protected" => Some(Self::Protected),
            _ => None,
        }
    }
}

/// Load debug graphics
pub fn load_debug_gfx() {
    // TODO: Load data/square.cel for debug display
}

/// Free debug graphics
pub fn free_debug_gfx() {
    // TODO: Free pSquareCel
}

/// Get debug monster info
pub fn get_debug_monster() {
    // TODO: Get current cursor monster or debug monster and print info
    let monster_id = DEBUG_MONSTER_ID.load(Ordering::SeqCst);
    log::debug!("Debug monster ID: {}", monster_id);
}

/// Switch to next debug monster
pub fn next_debug_monster() {
    let id = DEBUG_MONSTER_ID.fetch_add(1, Ordering::SeqCst) + 1;
    let new_id = if id >= 200 { 0 } else { id };
    DEBUG_MONSTER_ID.store(new_id, Ordering::SeqCst);
    log::debug!("Current debug monster = {}", new_id);
}

/// Set debug level seed info for current level
///
/// # Arguments
///
/// * `level` - Level index
/// * `mid1_seed` - First mid-point seed
/// * `mid2_seed` - Second mid-point seed
/// * `mid3_seed` - Third mid-point seed
/// * `end_seed` - End seed
pub fn set_debug_level_seed_infos(
    level: usize,
    mid1_seed: u32,
    mid2_seed: u32,
    mid3_seed: u32,
    end_seed: u32,
) {
    if level < NUMLEVELS {
        GL_MID1_SEED.lock().unwrap()[level] = mid1_seed;
        GL_MID2_SEED.lock().unwrap()[level] = mid2_seed;
        GL_MID3_SEED.lock().unwrap()[level] = mid3_seed;
        GL_END_SEED.lock().unwrap()[level] = end_seed;
    }
}

/// Check if debug grid text is needed
pub fn is_debug_grid_text_needed() -> bool {
    *SELECTED_DEBUG_GRID_TEXT_ITEM.lock().unwrap() != DebugGridTextItem::None
}

/// Check if debug grid is in megatile mode
pub fn is_debug_grid_in_megatiles() -> bool {
    match *SELECTED_DEBUG_GRID_TEXT_ITEM.lock().unwrap() {
        DebugGridTextItem::AutomapView
        | DebugGridTextItem::Dungeon
        | DebugGridTextItem::PDungeon
        | DebugGridTextItem::Protected => true,
        _ => false,
    }
}

/// Get current debug grid text type
pub fn get_debug_grid_text_type() -> DebugGridTextItem {
    *SELECTED_DEBUG_GRID_TEXT_ITEM.lock().unwrap()
}

/// Set debug grid text type
pub fn set_debug_grid_text_type(value: DebugGridTextItem) {
    *SELECTED_DEBUG_GRID_TEXT_ITEM.lock().unwrap() = value;
}

/// Get debug grid text for a specific dungeon coordinate
///
/// # Arguments
///
/// * `dungeon_coords` - The dungeon coordinates
///
/// # Returns
///
/// Some(text) if there's debug text to display, None otherwise
pub fn get_debug_grid_text(dungeon_coords: Point) -> Option<String> {
    let item_type = *SELECTED_DEBUG_GRID_TEXT_ITEM.lock().unwrap();
    
    match item_type {
        DebugGridTextItem::None => None,
        DebugGridTextItem::Coords => Some(format!("{}:{}", dungeon_coords.x, dungeon_coords.y)),
        DebugGridTextItem::CursorCoords => {
            // TODO: Check if dungeon_coords == cursor_position
            Some(format!("{}:{}", dungeon_coords.x, dungeon_coords.y))
        }
        _ => {
            // TODO: Implement other debug grid text types
            // These require access to game state arrays (dPiece, dLight, etc.)
            None
        }
    }
}

/// Check if debug automap highlight is needed
pub fn is_debug_automap_highlight_needed() -> bool {
    let monsters = SEARCH_MONSTERS.lock().unwrap();
    let items = SEARCH_ITEMS.lock().unwrap();
    let objects = SEARCH_OBJECTS.lock().unwrap();
    
    !monsters.is_empty() || !items.is_empty() || !objects.is_empty()
}

/// Check if a tile should be highlighted in debug automap
///
/// # Arguments
///
/// * `position` - The tile position to check
///
/// # Returns
///
/// true if the tile should be highlighted
pub fn should_highlight_debug_automap_tile(_position: Point) -> bool {
    // TODO: Implement monster/item/object name matching
    // Requires access to game state
    false
}

/// Add a monster name to the debug automap highlight list
pub fn add_debug_automap_monster_highlight(name: &str) {
    let mut monsters = SEARCH_MONSTERS.lock().unwrap();
    monsters.push(name.to_lowercase());
}

/// Add an item name to the debug automap highlight list
pub fn add_debug_automap_item_highlight(name: &str) {
    let mut items = SEARCH_ITEMS.lock().unwrap();
    items.push(name.to_lowercase());
}

/// Add an object name to the debug automap highlight list
pub fn add_debug_automap_object_highlight(name: &str) {
    let mut objects = SEARCH_OBJECTS.lock().unwrap();
    objects.push(name.to_lowercase());
}

/// Clear all debug automap highlights
pub fn clear_debug_automap_highlights() {
    SEARCH_MONSTERS.lock().unwrap().clear();
    SEARCH_ITEMS.lock().unwrap().clear();
    SEARCH_OBJECTS.lock().unwrap().clear();
}

/// Debug configuration
#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    pub toggle: bool,
    pub god_mode: bool,
    pub invisible: bool,
    pub vision: bool,
    pub path: bool,
    pub grid: bool,
    pub scroll_view_enabled: bool,
    pub grid_text_item: DebugGridTextItem,
}

impl DebugConfig {
    /// Create a new debug config from current global state
    pub fn from_globals() -> Self {
        Self {
            toggle: DEBUG_TOGGLE.load(Ordering::SeqCst),
            god_mode: DEBUG_GOD_MODE.load(Ordering::SeqCst),
            invisible: DEBUG_INVISIBLE.load(Ordering::SeqCst),
            vision: DEBUG_VISION.load(Ordering::SeqCst),
            path: DEBUG_PATH.load(Ordering::SeqCst),
            grid: DEBUG_GRID.load(Ordering::SeqCst),
            scroll_view_enabled: DEBUG_SCROLL_VIEW_ENABLED.load(Ordering::SeqCst),
            grid_text_item: get_debug_grid_text_type(),
        }
    }
    
    /// Apply this debug config to global state
    pub fn apply_to_globals(&self) {
        DEBUG_TOGGLE.store(self.toggle, Ordering::SeqCst);
        DEBUG_GOD_MODE.store(self.god_mode, Ordering::SeqCst);
        DEBUG_INVISIBLE.store(self.invisible, Ordering::SeqCst);
        DEBUG_VISION.store(self.vision, Ordering::SeqCst);
        DEBUG_PATH.store(self.path, Ordering::SeqCst);
        DEBUG_GRID.store(self.grid, Ordering::SeqCst);
        DEBUG_SCROLL_VIEW_ENABLED.store(self.scroll_view_enabled, Ordering::SeqCst);
        set_debug_grid_text_type(self.grid_text_item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_flags() {
        DEBUG_GOD_MODE.store(false, Ordering::SeqCst);
        assert!(!DEBUG_GOD_MODE.load(Ordering::SeqCst));
        
        DEBUG_GOD_MODE.store(true, Ordering::SeqCst);
        assert!(DEBUG_GOD_MODE.load(Ordering::SeqCst));
        
        DEBUG_GOD_MODE.store(false, Ordering::SeqCst);
    }
    
    #[test]
    fn test_debug_grid_text_item() {
        set_debug_grid_text_type(DebugGridTextItem::Coords);
        assert_eq!(get_debug_grid_text_type(), DebugGridTextItem::Coords);
        assert!(is_debug_grid_text_needed());
        
        set_debug_grid_text_type(DebugGridTextItem::None);
        assert!(!is_debug_grid_text_needed());
    }
    
    #[test]
    fn test_debug_grid_text() {
        set_debug_grid_text_type(DebugGridTextItem::Coords);
        let text = get_debug_grid_text(Point::new(10, 20));
        assert_eq!(text, Some("10:20".to_string()));
        
        set_debug_grid_text_type(DebugGridTextItem::None);
    }
    
    #[test]
    fn test_automap_highlights() {
        clear_debug_automap_highlights();
        assert!(!is_debug_automap_highlight_needed());
        
        add_debug_automap_monster_highlight("skeleton");
        assert!(is_debug_automap_highlight_needed());
        
        clear_debug_automap_highlights();
        assert!(!is_debug_automap_highlight_needed());
    }
    
    #[test]
    fn test_debug_grid_text_item_parse() {
        assert_eq!(
            DebugGridTextItem::from_str("coords"),
            Some(DebugGridTextItem::Coords)
        );
        assert_eq!(
            DebugGridTextItem::from_str("DPIECE"),
            Some(DebugGridTextItem::DPiece)
        );
        assert_eq!(DebugGridTextItem::from_str("invalid"), None);
    }
}
