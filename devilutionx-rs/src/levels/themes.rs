//! Theme Room System - Special themed rooms in dungeons
//!
//! Theme rooms are special decorated areas (shrines, treasure rooms, skeleton rooms, etc.)
//! placed in dungeons to add variety. This module manages theme room placement and creation.
//!
//! C++ source: Source/levels/themes.cpp (985 lines)
//! Target: ~600-700 lines Rust code (core framework, Theme_* functions deferred)

use rand::SeedableRng;

/// Maximum number of themes per level
pub const MAXTHEMES: usize = 50;

/// Theme type identifier
///
/// C++ equivalent: theme_id enum in Source/objdat.h:17-36
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum ThemeId {
    Barrel = 0,
    Shrine = 1,
    MonstPit = 2,
    SkelRoom = 3,
    Treasure = 4,
    Library = 5,
    Torture = 6,
    BloodFountain = 7,
    Decapitated = 8,
    PurifyingFountain = 9,
    ArmorStand = 10,
    GoatShrine = 11,
    Cauldron = 12,
    MurkyFountain = 13,
    TearFountain = 14,
    BrnCross = 15,
    WeaponRack = 16,
    None = -1,
}

impl ThemeId {
    /// Convert i8 to ThemeId
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => ThemeId::Barrel,
            1 => ThemeId::Shrine,
            2 => ThemeId::MonstPit,
            3 => ThemeId::SkelRoom,
            4 => ThemeId::Treasure,
            5 => ThemeId::Library,
            6 => ThemeId::Torture,
            7 => ThemeId::BloodFountain,
            8 => ThemeId::Decapitated,
            9 => ThemeId::PurifyingFountain,
            10 => ThemeId::ArmorStand,
            11 => ThemeId::GoatShrine,
            12 => ThemeId::Cauldron,
            13 => ThemeId::MurkyFountain,
            14 => ThemeId::TearFountain,
            15 => ThemeId::BrnCross,
            16 => ThemeId::WeaponRack,
            _ => ThemeId::None,
        }
    }

    /// Check if theme is a "good" theme (higher placement priority)
    ///
    /// C++ equivalent: ThemeGood array in Source/levels/themes.cpp:177-178
    pub fn is_good_theme(self) -> bool {
        matches!(
            self,
            ThemeId::Shrine
                | ThemeId::Library
                | ThemeId::Treasure
                | ThemeId::SkelRoom
        )
    }
}

/// Theme structure - defines a themed room
///
/// C++ equivalent: ThemeStruct in Source/levels/themes.h:12-15
#[derive(Debug, Clone, Copy)]
pub struct ThemeStruct {
    /// Theme type
    pub ttype: ThemeId,
    /// Theme transparency value (region ID in dTransVal)
    pub ttval: i8,
}

impl ThemeStruct {
    pub fn new(ttype: ThemeId, ttval: i8) -> Self {
        Self { ttype, ttval }
    }
}

/// Theme manager - stores all active themes for current level
///
/// C++ equivalent: Global variables in Source/levels/themes.cpp:27-30
pub struct ThemeManager {
    /// Array of active themes (max 50 per level)
    pub themes: [ThemeStruct; MAXTHEMES],
    /// Number of active themes (0-50)
    pub numthemes: usize,
    /// Armor stand placement flag
    pub armor_flag: bool,
    /// Weapon rack placement flag
    pub weapon_flag: bool,
    /// Zhar library theme index (-1 if not present)
    pub zharlib: i32,

    // Internal state for theme placement (C++ uses globals)
    /// Current theme X position during placement
    themex: i32,
    /// Current theme Y position during placement
    themey: i32,
    /// Theme variant (used by some themes for orientation/type)
    theme_var1: usize,

    // Theme placement flags (track what's been placed)
    cauldron_flag: bool,
    b_fountain_flag: bool, // Blood fountain
    m_fountain_flag: bool, // Murky fountain
    p_fountain_flag: bool, // Purifying fountain
    t_fountain_flag: bool, // Tear fountain
    treasure_flag: bool,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: [ThemeStruct::new(ThemeId::None, 0); MAXTHEMES],
            numthemes: 0,
            armor_flag: false,
            weapon_flag: false,
            zharlib: -1,
            themex: 0,
            themey: 0,
            theme_var1: 0,
            cauldron_flag: false,
            b_fountain_flag: false,
            m_fountain_flag: false,
            p_fountain_flag: false,
            t_fountain_flag: false,
            treasure_flag: false,
        }
    }

    /// C++ `CheckThemeReqs` (themes.cpp:204-222): a theme is available when
    /// its one-per-level flag (if any) is still set. The flags start `true`
    /// and are cleared once the theme is selected/placed.
    pub fn check_theme_reqs(&self, t: ThemeId, level: u8) -> bool {
        match t {
            ThemeId::Shrine | ThemeId::SkelRoom | ThemeId::Library => !matches!(level, 3 | 4),
            ThemeId::ArmorStand | ThemeId::WeaponRack => level != 1,
            ThemeId::Cauldron => level == 4 && self.cauldron_flag,
            ThemeId::BloodFountain => self.b_fountain_flag,
            ThemeId::PurifyingFountain => self.p_fountain_flag,
            ThemeId::MurkyFountain => self.m_fountain_flag,
            ThemeId::TearFountain => self.t_fountain_flag,
            ThemeId::Treasure => self.treasure_flag,
            _ => true,
        }
    }

    /// Reset the one-per-level theme flags to `true` (C++ `InitThemes`
    /// top-of-function state) and clear the theme list.
    pub fn reset(&mut self) {
        self.numthemes = 0;
        self.zharlib = -1;
        self.armor_flag = true;
        self.weapon_flag = true;
        self.cauldron_flag = true;
        self.b_fountain_flag = true;
        self.m_fountain_flag = true;
        self.p_fountain_flag = true;
        self.t_fountain_flag = true;
        self.treasure_flag = true;
        self.themex = 0;
        self.themey = 0;
        self.theme_var1 = 0;
    }

    /// C++ `HoldThemeRooms` (themes.cpp:902-918) for Cathedral: mark every
    /// tile of each theme room as Populated so normal monsters/objects avoid
    /// them. (Runs before InitObjects in LoadGameLevelDungeon.)
    ///
    /// Populated tiles are rejected by `RndLocOk`/`CanPlaceMonster` through
    /// C++ `TileContainsSetPiece` (`dFlags & DungeonFlag::Populated`). The
    /// caller passes the dungeon layout; this method stores the marker in
    /// `layout.populated` so placement checks can consult it.
    pub fn hold_theme_rooms(&mut self, layout: &mut crate::game::game_state::DungeonLayout) {
        for theme in self.themes.iter().take(self.numthemes) {
            // C++ dTransVal 0 is the opaque/unassigned region, never a valid
            // theme room (the flood starts at 1 and `CheckThemeRoom` requires
            // a bounded 9..100 tile room), so skip it defensively.
            let tv = theme.ttval;
            if tv <= 0 {
                continue;
            }
            for (i, t) in layout.trans_val.iter().enumerate() {
                if *t == tv {
                    layout.populated[i] = true;
                }
            }
        }
    }

    /// Add a theme to the manager
    ///
    /// # Returns
    /// `true` if added successfully, `false` if theme array is full
    pub fn add_theme(&mut self, ttype: ThemeId, ttval: i8) -> bool {
        if self.numthemes >= MAXTHEMES {
            return false;
        }

        self.themes[self.numthemes] = ThemeStruct::new(ttype, ttval);
        self.numthemes += 1;

        // Update flags
        match ttype {
            ThemeId::ArmorStand => self.armor_flag = true,
            ThemeId::WeaponRack => self.weapon_flag = true,
            ThemeId::Cauldron => self.cauldron_flag = true,
            ThemeId::BloodFountain => self.b_fountain_flag = true,
            ThemeId::PurifyingFountain => self.p_fountain_flag = true,
            ThemeId::MurkyFountain => self.m_fountain_flag = true,
            ThemeId::TearFountain => self.t_fountain_flag = true,
            ThemeId::Treasure => self.treasure_flag = true,
            _ => {}
        }

        true
    }

    /// Get theme at index
    pub fn get_theme(&self, index: usize) -> Option<&ThemeStruct> {
        if index < self.numthemes {
            Some(&self.themes[index])
        } else {
            None
        }
    }
}

// ============================================================================
// Real theme selection + placement (C++ Source/levels/themes.cpp)
// ============================================================================

const THEME_GOOD: [ThemeId; 4] = [ThemeId::GoatShrine, ThemeId::Shrine, ThemeId::SkelRoom, ThemeId::Library];

/// C++ `IsTileNotSolid(pos)` / `TileHasAny(pos, property)` helpers over the
/// layout's SOL table.
fn tile_props(game_state: &crate::game::game_state::GameState, x: i32, y: i32) -> crate::engine::dungeon::TileProperties {
    let Some(layout) = &game_state.dungeon_layout else { return crate::engine::dungeon::TileProperties::NONE };
    if x < 0 || y < 0 || x >= layout.width as i32 || y >= layout.height as i32 {
        return crate::engine::dungeon::TileProperties::NONE;
    }
    let pn = layout.d_piece[y as usize * layout.width + x as usize] as usize;
    layout.sol.get(pn).copied().unwrap_or_default()
}

fn is_tile_not_solid(game_state: &crate::game::game_state::GameState, x: i32, y: i32) -> bool {
    let Some(layout) = &game_state.dungeon_layout else { return false };
    if x < 0 || y < 0 || x >= layout.width as i32 || y >= layout.height as i32 {
        return false; // C++ IsTileNotSolid: out-of-bounds is not "not solid".
    }
    !tile_props(game_state, x, y).contains(crate::engine::dungeon::TileProperties::SOLID)
}

fn tile_has_any(game_state: &crate::game::game_state::GameState, x: i32, y: i32, prop: crate::engine::dungeon::TileProperties) -> bool {
    tile_props(game_state, x, y).contains(prop)
}

fn trans_val(game_state: &crate::game::game_state::GameState, x: i32, y: i32) -> i8 {
    let Some(layout) = &game_state.dungeon_layout else { return 0 };
    if x < 0 || y < 0 || x >= layout.width as i32 || y >= layout.height as i32 {
        return 0;
    }
    layout.trans_val[y as usize * layout.width + x as usize]
}

fn max_trans_val(game_state: &crate::game::game_state::GameState) -> i8 {
    let Some(layout) = &game_state.dungeon_layout else { return 0 };
    layout.trans_val.iter().copied().max().unwrap_or(0)
}

fn is_object_at(game_state: &crate::game::game_state::GameState, x: i32, y: i32) -> bool {
    game_state.objects.iter().any(|o| o.position.x == x && o.position.y == y)
}

fn add_object(game_state: &mut crate::game::game_state::GameState, otype: crate::game::objdat::ObjectId, x: i32, y: i32) {
    game_state.objects.push(crate::game::objects::Object::new(otype, crate::game::types::Point::new(x, y)));
}

/// C++ `CheckThemeObj3` (themes.cpp:141-156): a 1x1 object fits when in bounds,
/// not solid, same region, no object, and (optionally) not FlipCoin(frequency).
fn check_theme_obj3(
    game_state: &crate::game::game_state::GameState,
    x: i32,
    y: i32,
    region: i8,
    frequency: u32,
) -> bool {
    let Some(layout) = &game_state.dungeon_layout else { return false };
    if x < 0 || y < 0 || x >= layout.width as i32 || y >= layout.height as i32 {
        return false;
    }
    if !is_tile_not_solid(game_state, x, y) {
        return false;
    }
    if trans_val(game_state, x, y) != region {
        return false;
    }
    if is_object_at(game_state, x, y) {
        return false;
    }
    if frequency > 0 && crate::engine::random::gameplay_flip_coin(frequency as i32) {
        return false;
    }
    true
}

/// C++ `CheckThemeObj5` (themes.cpp:98-113): a 2x2 object fits when all four
/// tiles are non-solid and in the same region.
fn check_theme_obj5(game_state: &crate::game::game_state::GameState, origin: (i32, i32), region: i8) -> bool {
    for dy in 0..2i32 {
        for dx in 0..2i32 {
            let (x, y) = (origin.0 + dx, origin.1 + dy);
            if !is_tile_not_solid(game_state, x, y) {
                return false;
            }
            if trans_val(game_state, x, y) != region {
                return false;
            }
        }
    }
    true
}

/// C++ `TFit_Shrine` (themes.cpp:50-96): scan for the shrine pattern (a trap
/// tile flanked by walkable tiles). Returns `(x, y, variant)`.
fn tfit_shrine(game_state: &crate::game::game_state::GameState, region: i8) -> Option<(i32, i32, usize)> {
    let mut position = (0i32, 0i32);
    let mut found: usize = 0;
    while found == 0 {
        let (px, py) = position;
        if trans_val(game_state, px, py) == region {
            // C++ Direction deltas (displacement.hpp): NE=(0,-1), NW=(-1,0),
            // SE=(1,0), SW=(0,1), N=(-1,-1), E=(1,-1), W=(-1,1).
            if tile_has_any(game_state, px, py - 1, crate::engine::dungeon::TileProperties::TRAP)
                && is_tile_not_solid(game_state, px - 1, py)
                && is_tile_not_solid(game_state, px + 1, py)
                && trans_val(game_state, px - 1, py) == region
                && trans_val(game_state, px + 1, py) == region
                && !is_object_at(game_state, px - 1, py - 1)
                && !is_object_at(game_state, px + 1, py - 1)
            {
                found = 1;
            }
            if found == 0
                && tile_has_any(game_state, px - 1, py, crate::engine::dungeon::TileProperties::TRAP)
                && is_tile_not_solid(game_state, px, py - 1)
                && is_tile_not_solid(game_state, px, py + 1)
                && trans_val(game_state, px, py - 1) == region
                && trans_val(game_state, px, py + 1) == region
                && !is_object_at(game_state, px - 1, py - 1)
                && !is_object_at(game_state, px - 1, py + 1)
            {
                found = 2;
            }
        }
        if found == 0 {
            position.0 += 1;
            if position.0 == crate::levels::types::MAXDUNX as i32 {
                position.0 = 0;
                position.1 += 1;
                if position.1 == crate::levels::types::MAXDUNY as i32 {
                    return None;
                }
            }
        }
    }
    Some((position.0, position.1, found))
}

/// C++ `TFit_Obj5` (themes.cpp:115-139): pick a random target candidate count
/// (`GenerateRnd(5)` — one RNG draw), then scan for a valid 2x2 area.
fn tfit_obj5(game_state: &crate::game::game_state::GameState, region: i8) -> Option<(i32, i32)> {
    let target = crate::engine::random::gameplay_generate_rnd(5);
    if target < 0 {
        return Some((0, 0));
    }
    let mut candidates = 0usize;
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y)
                && check_theme_obj5(game_state, (x, y), region)
            {
                candidates += 1;
                if candidates > target as usize {
                    return Some((x, y));
                }
            }
        }
    }
    if candidates > 0 { Some((0, 0)) } else { None }
}

/// C++ `IsSkel` (monstdat): any of the 12 skeleton monster types.
fn is_skel(id: crate::game::monstdat::MonsterId) -> bool {
    use crate::game::monstdat::MonsterId;
    matches!(
        id,
        MonsterId::SkeletonAxeW | MonsterId::SkeletonAxeT | MonsterId::SkeletonAxeR | MonsterId::SkeletonAxeX
            | MonsterId::SkeletonBowW | MonsterId::SkeletonBowT | MonsterId::SkeletonBowR | MonsterId::SkeletonBowX
            | MonsterId::SkeletonSwordW | MonsterId::SkeletonSwordT | MonsterId::SkeletonSwordR | MonsterId::SkeletonSwordX
    )
}

/// C++ `IsGoat` (monstdat): the 8 goat-men types.
fn is_goat(id: crate::game::monstdat::MonsterId) -> bool {
    use crate::game::monstdat::MonsterId;
    matches!(
        id,
        MonsterId::GoatManN | MonsterId::GoatManB | MonsterId::GoatManR | MonsterId::GoatManG
            | MonsterId::GoatArcherN | MonsterId::GoatArcherB | MonsterId::GoatArcherR | MonsterId::GoatArcherG
    )
}

/// C++ `TFit_SkelRoom` (themes.cpp:141-154): needs a skeleton scatter type.
fn tfit_skel_room(game_state: &crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, region: i8) -> Option<(usize, (i32, i32))> {
    for i in 0..level_types.count() {
        if is_skel(level_types.get(i).map(|e| e.monster_type).unwrap_or(crate::game::monstdat::MonsterId::Invalid)) {
            let pos = tfit_obj5(game_state, region)?;
            return Some((i, pos));
        }
    }
    None
}

/// C++ `TFit_GoatShrine` (themes.cpp:156-165): needs a goat scatter type.
fn tfit_goat_shrine(game_state: &crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, region: i8) -> Option<(usize, (i32, i32))> {
    for i in 0..level_types.count() {
        if is_goat(level_types.get(i).map(|e| e.monster_type).unwrap_or(crate::game::monstdat::MonsterId::Invalid)) {
            let pos = tfit_obj5(game_state, region)?;
            return Some((i, pos));
        }
    }
    None
}

/// C++ `TFit_Obj3` (themes.cpp:158-172): scan for a 1x1 fit in the region.
fn tfit_obj3(game_state: &crate::game::game_state::GameState, region: i8) -> Option<(i32, i32)> {
    for y in 1..(crate::levels::types::MAXDUNY as i32 - 1) {
        for x in 1..(crate::levels::types::MAXDUNX as i32 - 1) {
            if check_theme_obj3(game_state, x, y, region, 0) {
                return Some((x, y));
            }
        }
    }
    None
}

/// C++ `SpecialThemeFit` (themes.cpp:224-301). The helper scans only read the
/// game state; the found position/variant is written to the theme manager
/// after each immutable borrow ends.
fn special_theme_fit(
    game_state: &mut crate::game::game_state::GameState,
    level_types: &crate::game::monster::LevelMonsterTypes,
    i: usize,
    t: ThemeId,
) -> bool {
    let level = game_state.current_dungeon_level;
    let mut rv = game_state.theme_manager.check_theme_reqs(t, level);
    let region = game_state.theme_manager.themes[i].ttval;
    match t {
        ThemeId::Shrine | ThemeId::Library => {
            if rv {
                if let Some((x, y, variant)) = tfit_shrine(game_state, region) {
                    game_state.theme_manager.themex = x;
                    game_state.theme_manager.themey = y;
                    game_state.theme_manager.theme_var1 = variant;
                } else {
                    rv = false;
                }
            }
        }
        ThemeId::SkelRoom => {
            if rv {
                if let Some((skel, (x, y))) = tfit_skel_room(game_state, level_types, region) {
                    game_state.theme_manager.theme_var1 = skel;
                    game_state.theme_manager.themex = x;
                    game_state.theme_manager.themey = y;
                } else {
                    rv = false;
                }
            }
        }
        ThemeId::BloodFountain => {
            if rv {
                rv = tfit_obj5(game_state, region).is_some();
            }
            if rv {
                game_state.theme_manager.b_fountain_flag = false;
            }
        }
        ThemeId::PurifyingFountain => {
            if rv {
                rv = tfit_obj5(game_state, region).is_some();
            }
            if rv {
                game_state.theme_manager.p_fountain_flag = false;
            }
        }
        ThemeId::MurkyFountain => {
            if rv {
                rv = tfit_obj5(game_state, region).is_some();
            }
            if rv {
                game_state.theme_manager.m_fountain_flag = false;
            }
        }
        ThemeId::TearFountain => {
            if rv {
                rv = tfit_obj5(game_state, region).is_some();
            }
            if rv {
                game_state.theme_manager.t_fountain_flag = false;
            }
        }
        ThemeId::Cauldron => {
            if rv {
                rv = tfit_obj5(game_state, region).is_some();
            }
            if rv {
                game_state.theme_manager.cauldron_flag = false;
            }
        }
        ThemeId::GoatShrine => {
            if rv {
                rv = tfit_goat_shrine(game_state, level_types, region).is_some();
            }
        }
        ThemeId::Torture
        | ThemeId::Decapitated
        | ThemeId::ArmorStand
        | ThemeId::BrnCross
        | ThemeId::WeaponRack => {
            if rv {
                rv = tfit_obj3(game_state, region).is_some();
            }
        }
        ThemeId::Treasure => {
            if rv {
                game_state.theme_manager.treasure_flag = false;
            }
        }
        _ => {}
    }
    rv
}

/// C++ `CheckThemeRoom` (themes.cpp:305-351).
fn check_theme_room(game_state: &crate::game::game_state::GameState, tv: i8) -> bool {
    for t in 0..game_state.triggers.numtrigs {
        let pos = game_state.triggers.trigs[t].position;
        if trans_val(game_state, pos.x, pos.y) == tv {
            return false;
        }
    }
    let mut area = 0usize;
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) != tv {
                continue;
            }
            // TileContainsSetPiece: quest-free — empty.
            area += 1;
        }
    }
    if game_state.current_dungeon_level == 1 && (area < 9 || area > 100) {
        return false;
    }
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) != tv || !is_tile_not_solid(game_state, x, y) {
                continue;
            }
            if trans_val(game_state, x - 1, y) != tv && is_tile_not_solid(game_state, x - 1, y) {
                return false;
            }
            if trans_val(game_state, x + 1, y) != tv && is_tile_not_solid(game_state, x + 1, y) {
                return false;
            }
            if trans_val(game_state, x, y - 1) != tv && is_tile_not_solid(game_state, x, y - 1) {
                return false;
            }
            if trans_val(game_state, x, y + 1) != tv && is_tile_not_solid(game_state, x, y + 1) {
                return false;
            }
        }
    }
    true
}

/// C++ `InitThemes` (themes.cpp:823-893), Cathedral branch only.
pub fn init_themes(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes) {
    game_state.theme_manager.reset();
    let level = game_state.current_dungeon_level;
    if level == 16 || matches!(level, 5 | 6) {
        return;
    }
    let max_tv = max_trans_val(game_state);
    let mut i: i32 = 0;
    while game_state.theme_manager.numthemes < MAXTHEMES {
        let tv = i as i8;
        if check_theme_room(game_state, tv) {
            let idx = game_state.theme_manager.numthemes;
            game_state.theme_manager.themes[idx].ttval = tv;
            let mut j = THEME_GOOD[crate::engine::random::gameplay_generate_rnd(4) as usize];
            while !special_theme_fit(game_state, level_types, idx, j) {
                j = ThemeId::from_i8(crate::engine::random::gameplay_generate_rnd(17) as i8);
            }
            game_state.theme_manager.themes[idx].ttype = j;
            game_state.theme_manager.numthemes += 1;
        }
        i += 1;
        if i > max_tv as i32 {
            break;
        }
    }
    println!("[Themes] numthemes={} max_tv={}", game_state.theme_manager.numthemes, max_tv);
}


// ============================================================================
// Theme room content placement (C++ Theme_* + CreateThemeRooms)
// ============================================================================

/// C++ `PlaceThemeMonsts` (themes.cpp:353-375): place `1/f`-frequent scatter
/// monsters across the theme region. `GenerateRnd(numscattypes)` is one draw;
/// each placed monster consumes its InitMonster draws.
fn place_theme_monsts(
    game_state: &mut crate::game::game_state::GameState,
    level_types: &crate::game::monster::LevelMonsterTypes,
    t: usize,
    f: i32,
) {
    let scatter = level_types.scatter_indices();
    if scatter.is_empty() {
        return;
    }
    let mtype = scatter[crate::engine::random::gameplay_generate_rnd(scatter.len() as i32) as usize];
    let region = game_state.theme_manager.themes[t].ttval;
    let mut spawns = Vec::new();
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region
                && is_tile_not_solid(game_state, x, y)
                && !game_state.ground_items.iter().any(|gi| gi.x == x && gi.y == y)
                && !is_object_at(game_state, x, y)
                && crate::engine::random::gameplay_flip_coin(f)
            {
                spawns.push((x, y));
            }
        }
    }
    for (x, y) in spawns {
        let id = (game_state.monster_manager.active_count() + 1) as u32;
        let mtype_id = level_types.get(mtype).map(|e| e.monster_type).unwrap_or(crate::game::monstdat::MonsterId::Zombie);
        let dir = crate::engine::random::gameplay_generate_rnd(8);
        let mut m = crate::game::monster::Monster::new_with_rng(
            id,
            mtype_id,
            x,
            y,
            game_state.current_dungeon_level,
            &mut rand::rngs::StdRng::seed_from_u64(0),
        );
        m.facing = crate::game::types::Direction::ALL[(dir as usize) % 8];
        m.level_type = mtype as u8;
        m.ai_state = crate::game::monster::MonsterAIState::Idle;
        m.mode = crate::game::monster::MonsterMode::Stand;
        game_state.monster_manager.add_monster(m);
    }
}

/// C++ `AddSkeleton` + `GetRandomSkeletonTypeIndex` (monster.cpp): pick a
/// random skeleton scatter slot (one draw) and spawn it via InitMonster draws.
fn spawn_skeleton(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, x: i32, y: i32) {
    let mut skeleton_indexes: Vec<usize> = Vec::new();
    for i in 0..level_types.count() {
        let id = level_types.get(i).map(|e| e.monster_type).unwrap_or(crate::game::monstdat::MonsterId::Invalid);
        if is_skel(id) {
            skeleton_indexes.push(i);
        }
    }
    if skeleton_indexes.is_empty() {
        return;
    }
    let type_index = skeleton_indexes[crate::engine::random::gameplay_generate_rnd(skeleton_indexes.len() as i32) as usize];
    let id = (game_state.monster_manager.active_count() + 1) as u32;
    let mtype_id = level_types.get(type_index).map(|e| e.monster_type).unwrap_or(crate::game::monstdat::MonsterId::SkeletonAxeW);
    let mut m = crate::game::monster::Monster::new_with_rng(
        id,
        mtype_id,
        x,
        y,
        game_state.current_dungeon_level,
        &mut rand::rngs::StdRng::seed_from_u64(0),
    );
    m.facing = crate::game::types::Direction::SouthWest;
    m.level_type = type_index as u8;
    m.ai_state = crate::game::monster::MonsterAIState::Idle;
    m.mode = crate::game::monster::MonsterMode::Stand;
    game_state.monster_manager.add_monster(m);
}

/// C++ `SpawnObjectOrSkeleton` (themes.cpp:469-479).
fn spawn_object_or_skeleton(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, frequency: u32, otype: crate::game::objdat::ObjectId, x: i32, y: i32) {
    if crate::engine::random::gameplay_flip_coin(frequency as i32) {
        add_object(game_state, otype, x, y);
    } else {
        spawn_skeleton(game_state, level_types, x, y);
    }
}

/// C++ `Theme_Barrel` (themes.cpp:377-397).
fn theme_barrel(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let barrnd = [2i32, 6, 4, 8];
    let monstrnd = [5i32, 7, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y) {
                if crate::engine::random::gameplay_flip_coin(barrnd[level - 1]) {
                    let r = if crate::engine::random::gameplay_flip_coin(barrnd[level - 1]) { ObjectId::Barrel } else { ObjectId::BarrelEx };
                    add_object(game_state, r, x, y);
                }
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_Shrine` (themes.cpp:399-419).
fn theme_shrine(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let monstrnd = [6i32, 6, 3, 9];
    let region = game_state.theme_manager.themes[t].ttval;
    if let Some((x, y, variant)) = tfit_shrine(game_state, region) {
        if variant == 1 {
            add_object(game_state, ObjectId::Candle2, x - 1, y);
            add_object(game_state, ObjectId::ShrineR, x, y);
            add_object(game_state, ObjectId::Candle2, x + 1, y);
        } else {
            add_object(game_state, ObjectId::Candle2, x, y - 1);
            add_object(game_state, ObjectId::ShrineL, x, y);
            add_object(game_state, ObjectId::Candle2, x, y + 1);
        }
    }
    let level = game_state.current_dungeon_level as usize;
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_Treasure` (themes.cpp:512-551): gold/random item drops.
fn theme_treasure(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::items::Item;
    let treasrnd = [4i32, 9, 7, 10];
    let monstrnd = [6i32, 8, 3, 7];
    let level = game_state.current_dungeon_level as usize;
    let treasure_type = treasrnd[level - 1];
    let region = game_state.theme_manager.themes[t].ttval;
    crate::engine::random::gameplay_advance_rnd_seed(); // DiscardRandomValues(1)
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y) {
                let rv = crate::engine::random::gameplay_generate_rnd(treasure_type);
                if crate::engine::random::gameplay_flip_coin(treasure_type) {
                    let seed = crate::engine::random::gameplay_advance_rnd_seed() as u32;
                    let value = 15 * ((seed % 100) as i32) + 50;
                    game_state.ground_items.push(crate::game::game_state::GroundItem {
                        x, y,
                        item_type: crate::game::game_state::GroundItemType::Gold,
                        item_index: None,
                        item: Some(Item::gold(value)),
                    });
                }
                if rv == 0 {
                    let seed = crate::engine::random::gameplay_advance_rnd_seed() as u32;
                    let value = 15 * ((seed % 100) as i32) + 50;
                    game_state.ground_items.push(crate::game::game_state::GroundItem {
                        x, y,
                        item_type: crate::game::game_state::GroundItemType::Gold,
                        item_index: None,
                        item: Some(Item::gold(value)),
                    });
                }
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_Library` (themes.cpp:553-593).
fn theme_library(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let librnd = [1u32, 2, 2, 5];
    let monstrnd = [5i32, 7, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    if let Some((x, y, variant)) = tfit_shrine(game_state, region) {
        if variant == 1 {
            add_object(game_state, ObjectId::BookCandle, x - 1, y);
            add_object(game_state, ObjectId::BookcaseR, x, y);
            add_object(game_state, ObjectId::BookCandle, x + 1, y);
        } else {
            add_object(game_state, ObjectId::BookCandle, x, y - 1);
            add_object(game_state, ObjectId::BookcaseL, x, y);
            add_object(game_state, ObjectId::BookCandle, x, y + 1);
        }
    }
    for y in 1..(crate::levels::types::MAXDUNY as i32 - 1) {
        for x in 1..(crate::levels::types::MAXDUNX as i32 - 1) {
            if check_theme_obj3(game_state, x, y, region, 0)
                && !game_state.monster_manager.find_monster_at(crate::game::types::Point::new(x, y)).is_some()
                && crate::engine::random::gameplay_flip_coin(librnd[level - 1] as i32)
            {
                add_object(game_state, ObjectId::Bookstand, x, y);
                if !crate::engine::random::gameplay_flip_coin((2 * librnd[level - 1]) as i32) {
                    if let Some(o) = game_state.objects.last_mut() {
                        o.selection_region = crate::game::objdat::SelectionRegion::None;
                        o.anim_frame += 2;
                    }
                }
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_SkelRoom` (themes.cpp:481-510).
fn theme_skel_room(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let monstrnd = [6u32, 7, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    let Some((skel, (xp, yp))) = tfit_skel_room(game_state, level_types, region) else { return };
    game_state.theme_manager.theme_var1 = skel;
    game_state.theme_manager.themex = xp;
    game_state.theme_manager.themey = yp;
    add_object(game_state, ObjectId::SkFire, xp, yp);
    spawn_object_or_skeleton(game_state, level_types, monstrnd[level - 1], ObjectId::BannerL, xp - 1, yp - 1);
    spawn_skeleton(game_state, level_types, xp, yp - 1);
    spawn_object_or_skeleton(game_state, level_types, monstrnd[level - 1], ObjectId::BannerR, xp + 1, yp - 1);
    spawn_object_or_skeleton(game_state, level_types, monstrnd[level - 1], ObjectId::BannerM, xp - 1, yp);
    spawn_object_or_skeleton(game_state, level_types, monstrnd[level - 1], ObjectId::BannerM, xp + 1, yp);
    spawn_object_or_skeleton(game_state, level_types, monstrnd[level - 1], ObjectId::BannerR, xp - 1, yp + 1);
    spawn_skeleton(game_state, level_types, xp, yp + 1);
    spawn_object_or_skeleton(game_state, level_types, monstrnd[level - 1], ObjectId::BannerL, xp + 1, yp + 1);
    if !is_object_at(game_state, xp, yp - 3) {
        add_object(game_state, ObjectId::SkelBook, xp, yp - 2);
    }
    if !is_object_at(game_state, xp, yp + 3) {
        add_object(game_state, ObjectId::SkelBook, xp, yp + 2);
    }
}

/// C++ `Theme_Torture` (themes.cpp:595-613).
fn theme_torture(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let tortrnd = [6u32, 8, 3, 8];
    let monstrnd = [6i32, 8, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    for y in 1..(crate::levels::types::MAXDUNY as i32 - 1) {
        for x in 1..(crate::levels::types::MAXDUNX as i32 - 1) {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y)
                && check_theme_obj3(game_state, x, y, region, 0)
                && crate::engine::random::gameplay_flip_coin(tortrnd[level - 1] as i32)
            {
                add_object(game_state, ObjectId::TNudeM2, x, y);
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_BloodFountain`/`Theme_Cauldron`/fountains: TFit_Obj5 + one object.
fn theme_obj5_object(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize, otype: crate::game::objdat::ObjectId, monstrnd: [i32; 4]) {
    let region = game_state.theme_manager.themes[t].ttval;
    if let Some((x, y)) = tfit_obj5(game_state, region) {
        add_object(game_state, otype, x, y);
    }
    let level = game_state.current_dungeon_level as usize;
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_Decap` (themes.cpp:633-654).
fn theme_decap(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let decaprnd = [6u32, 8, 3, 8];
    let monstrnd = [6i32, 8, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    for y in 1..(crate::levels::types::MAXDUNY as i32 - 1) {
        for x in 1..(crate::levels::types::MAXDUNX as i32 - 1) {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y)
                && check_theme_obj3(game_state, x, y, region, 0)
                && crate::engine::random::gameplay_flip_coin(decaprnd[level - 1] as i32)
            {
                add_object(game_state, ObjectId::Decap, x, y);
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_ArmorStand` (themes.cpp:670-697).
fn theme_armor_stand(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let armorrnd = [6u32, 8, 3, 8];
    let monstrnd = [6i32, 7, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    if game_state.theme_manager.armor_flag {
        if let Some((x, y)) = tfit_obj3(game_state, region) {
            add_object(game_state, ObjectId::ArmorStand, x, y);
        }
    }
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y)
                && check_theme_obj3(game_state, x, y, region, 0)
                && crate::engine::random::gameplay_flip_coin(armorrnd[level - 1] as i32)
            {
                add_object(game_state, ObjectId::ArmorStandN, x, y);
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
    game_state.theme_manager.armor_flag = false;
}

/// C++ `Theme_GoatShrine` (themes.cpp:699-715).
fn theme_goat_shrine(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let region = game_state.theme_manager.themes[t].ttval;
    let Some((goat, (themex, themey))) = tfit_goat_shrine(game_state, level_types, region) else { return };
    game_state.theme_manager.theme_var1 = goat;
    game_state.theme_manager.themex = themex;
    game_state.theme_manager.themey = themey;
    add_object(game_state, ObjectId::GoatShrine, themex, themey);
    for yy in (themey - 1)..=(themey + 1) {
        for xx in (themex - 1)..=(themex + 1) {
            if trans_val(game_state, xx, yy) == region && is_tile_not_solid(game_state, xx, yy) && (xx != themex || yy != themey) {
                let id = (game_state.monster_manager.active_count() + 1) as u32;
                let goat_id = level_types.get(goat).map(|e| e.monster_type).unwrap_or(crate::game::monstdat::MonsterId::GoatManN);
                let mut m = crate::game::monster::Monster::new_with_rng(
                    id, goat_id, xx, yy, game_state.current_dungeon_level,
                    &mut rand::rngs::StdRng::seed_from_u64(0),
                );
                m.facing = crate::game::types::Direction::SouthWest;
                m.level_type = goat as u8;
                m.ai_state = crate::game::monster::MonsterAIState::Idle;
                m.mode = crate::game::monster::MonsterMode::Stand;
                game_state.monster_manager.add_monster(m);
            }
        }
    }
}

/// C++ `Theme_BrnCross` (themes.cpp:759-781).
fn theme_brn_cross(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let bcrossrnd = [5u32, 7, 3, 8];
    let monstrnd = [6i32, 8, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y)
                && check_theme_obj3(game_state, x, y, region, 0)
                && crate::engine::random::gameplay_flip_coin(bcrossrnd[level - 1] as i32)
            {
                add_object(game_state, ObjectId::TBCross, x, y);
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
}

/// C++ `Theme_WeaponRack` (themes.cpp:784-805).
fn theme_weapon_rack(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes, t: usize) {
    use crate::game::objdat::ObjectId;
    let weaponrnd = [6u32, 8, 5, 8];
    let monstrnd = [6i32, 7, 3, 9];
    let level = game_state.current_dungeon_level as usize;
    let region = game_state.theme_manager.themes[t].ttval;
    if game_state.theme_manager.weapon_flag {
        if let Some((x, y)) = tfit_obj3(game_state, region) {
            add_object(game_state, ObjectId::WeapRack, x, y);
        }
    }
    for y in 0..crate::levels::types::MAXDUNY as i32 {
        for x in 0..crate::levels::types::MAXDUNX as i32 {
            if trans_val(game_state, x, y) == region && is_tile_not_solid(game_state, x, y)
                && check_theme_obj3(game_state, x, y, region, 0)
                && crate::engine::random::gameplay_flip_coin(weaponrnd[level - 1] as i32)
            {
                add_object(game_state, ObjectId::WeaponRackN, x, y);
            }
        }
    }
    place_theme_monsts(game_state, level_types, t, monstrnd[level - 1]);
    game_state.theme_manager.weapon_flag = false;
}

/// C++ `CreateThemeRooms` (themes.cpp:912-...): dispatch every selected theme.
pub fn create_theme_rooms(game_state: &mut crate::game::game_state::GameState, level_types: &crate::game::monster::LevelMonsterTypes) {
    let level = game_state.current_dungeon_level;
    if level == 16 || matches!(level, 5 | 6) {
        return;
    }
    for i in 0..game_state.theme_manager.numthemes {
        game_state.theme_manager.themex = 0;
        game_state.theme_manager.themey = 0;
        let ttype = game_state.theme_manager.themes[i].ttype;
        match ttype {
            ThemeId::Barrel => theme_barrel(game_state, level_types, i),
            ThemeId::Shrine => theme_shrine(game_state, level_types, i),
            ThemeId::MonstPit => {
                // CreateRndItem drop (1 global draw per item); simplified gold.
                let region = game_state.theme_manager.themes[i].ttval;
                let monstrnd = [6i32, 7, 3, 9];
                let lvl = level as usize;
                let r = crate::engine::random::gameplay_generate_rnd(100) + 1;
                let mut remaining = r;
                let mut found = None;
                let mut ixp = 0i32;
                let mut iyp = 0i32;
                // C++ scans with wrap-around until the r-th matching tile; a
                // region with fewer than r non-solid tiles would spin forever
                // there too, so bound the scan at one full pass.
                let mut scanned = 0usize;
                while remaining > 0 && scanned < (crate::levels::types::MAXDUNX * crate::levels::types::MAXDUNY) {
                    if trans_val(game_state, ixp, iyp) == region && is_tile_not_solid(game_state, ixp, iyp) {
                        remaining -= 1;
                        found = Some((ixp, iyp));
                    }
                    if remaining <= 0 {
                        break;
                    }
                    ixp += 1;
                    if ixp == crate::levels::types::MAXDUNX as i32 {
                        ixp = 0;
                        iyp += 1;
                        if iyp == crate::levels::types::MAXDUNY as i32 {
                            iyp = 0;
                        }
                    }
                    scanned += 1;
                }
                if let Some((x, y)) = found {
                    let seed = crate::engine::random::gameplay_advance_rnd_seed() as u32;
                    let value = 15 * ((seed % 100) as i32) + 50;
                    game_state.ground_items.push(crate::game::game_state::GroundItem {
                        x, y,
                        item_type: crate::game::game_state::GroundItemType::Gold,
                        item_index: None,
                        item: Some(crate::game::items::Item::gold(value)),
                    });
                }
                place_theme_monsts(game_state, level_types, i, monstrnd[lvl - 1]);
            }
            ThemeId::SkelRoom => theme_skel_room(game_state, level_types, i),
            ThemeId::Treasure => theme_treasure(game_state, level_types, i),
            ThemeId::Library => theme_library(game_state, level_types, i),
            ThemeId::Torture => theme_torture(game_state, level_types, i),
            ThemeId::BloodFountain => theme_obj5_object(game_state, level_types, i, crate::game::objdat::ObjectId::BloodFtn, [6, 8, 3, 9]),
            ThemeId::Decapitated => theme_decap(game_state, level_types, i),
            ThemeId::PurifyingFountain => theme_obj5_object(game_state, level_types, i, crate::game::objdat::ObjectId::PurifyingFtn, [6, 7, 3, 9]),
            ThemeId::ArmorStand => theme_armor_stand(game_state, level_types, i),
            ThemeId::GoatShrine => theme_goat_shrine(game_state, level_types, i),
            ThemeId::Cauldron => theme_obj5_object(game_state, level_types, i, crate::game::objdat::ObjectId::Cauldron, [6, 7, 3, 9]),
            ThemeId::MurkyFountain => theme_obj5_object(game_state, level_types, i, crate::game::objdat::ObjectId::MurkyFtn, [6, 7, 3, 9]),
            ThemeId::TearFountain => theme_obj5_object(game_state, level_types, i, crate::game::objdat::ObjectId::TearFtn, [6, 7, 3, 9]),
            ThemeId::BrnCross => theme_brn_cross(game_state, level_types, i),
            ThemeId::WeaponRack => theme_weapon_rack(game_state, level_types, i),
            ThemeId::None => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_new_and_reset() {
        let mut mgr = ThemeManager::new();
        mgr.reset();
        assert_eq!(mgr.numthemes, 0);
        assert_eq!(mgr.zharlib, -1);
        assert!(mgr.armor_flag);
        assert!(mgr.weapon_flag);
        assert!(mgr.treasure_flag);
    }

    #[test]
    fn test_theme_id_from_i8() {
        assert_eq!(ThemeId::from_i8(0), ThemeId::Barrel);
        assert_eq!(ThemeId::from_i8(1), ThemeId::Shrine);
        assert_eq!(ThemeId::from_i8(16), ThemeId::WeaponRack);
        assert_eq!(ThemeId::from_i8(-1), ThemeId::None);
        assert_eq!(ThemeId::from_i8(99), ThemeId::None);
    }

    #[test]
    fn test_theme_id_is_good_theme() {
        assert!(ThemeId::Shrine.is_good_theme());
        assert!(ThemeId::Library.is_good_theme());
        assert!(ThemeId::Treasure.is_good_theme());
        assert!(ThemeId::SkelRoom.is_good_theme());
        assert!(!ThemeId::Barrel.is_good_theme());
    }

    #[test]
    fn test_add_theme() {
        let mut mgr = ThemeManager::new();
        assert!(mgr.add_theme(ThemeId::Shrine, 10));
        assert_eq!(mgr.numthemes, 1);
        assert_eq!(mgr.themes[0].ttype, ThemeId::Shrine);
        assert_eq!(mgr.themes[0].ttval, 10);
    }

    #[test]
    fn test_add_theme_max_limit() {
        let mut mgr = ThemeManager::new();
        for i in 0..MAXTHEMES {
            assert!(mgr.add_theme(ThemeId::Barrel, i as i8));
        }
        assert_eq!(mgr.numthemes, MAXTHEMES);
        assert!(!mgr.add_theme(ThemeId::Shrine, 99));
    }

    #[test]
    fn test_check_theme_reqs_cpp_semantics() {
        let mut mgr = ThemeManager::new();
        mgr.reset();
        // Level 1 (Cathedral): shrines/skel/library OK, armor/weapon not.
        assert!(mgr.check_theme_reqs(ThemeId::Shrine, 1));
        assert!(mgr.check_theme_reqs(ThemeId::SkelRoom, 1));
        assert!(mgr.check_theme_reqs(ThemeId::Library, 1));
        assert!(!mgr.check_theme_reqs(ThemeId::ArmorStand, 1));
        assert!(!mgr.check_theme_reqs(ThemeId::WeaponRack, 1));
        assert!(mgr.check_theme_reqs(ThemeId::Treasure, 1));
        // Treasure consumes its one-per-level flag.
        mgr.treasure_flag = false;
        assert!(!mgr.check_theme_reqs(ThemeId::Treasure, 1));
        // Armor stand allowed on level 2+.
        assert!(mgr.check_theme_reqs(ThemeId::ArmorStand, 2));
        // Cauldron only in Hell.
        assert!(!mgr.check_theme_reqs(ThemeId::Cauldron, 1));
        assert!(mgr.check_theme_reqs(ThemeId::Cauldron, 4));
    }

    #[test]
    fn test_is_skel_goat() {
        use crate::game::monstdat::MonsterId;
        assert!(is_skel(MonsterId::SkeletonAxeW));
        assert!(is_skel(MonsterId::SkeletonBowT));
        assert!(is_skel(MonsterId::SkeletonSwordR));
        assert!(!is_skel(MonsterId::GoatManN));
        assert!(is_goat(MonsterId::GoatManN));
        assert!(is_goat(MonsterId::GoatArcherB));
        assert!(!is_goat(MonsterId::SkeletonAxeW));
    }

    #[test]
    fn test_get_theme() {
        let mut mgr = ThemeManager::new();
        mgr.add_theme(ThemeId::Shrine, 10);
        mgr.add_theme(ThemeId::Library, 20);
        let t0 = mgr.get_theme(0).unwrap();
        assert_eq!(t0.ttype, ThemeId::Shrine);
        assert_eq!(t0.ttval, 10);
        assert!(mgr.get_theme(2).is_none());
    }
}
