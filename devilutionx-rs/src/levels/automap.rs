//! Automap exploration state (C++ `Source/automap.cpp`).
//!
//! Ports the `AutomapTile` shape table, `GetAutomapTileType`,
//! `HasAutomapFlag`, `UpdateAutomapExplorer` and `SetAutomapView` so the
//! saved `AutomapView[DMAXX][DMAXY]` matches the C++ engine byte-for-byte.

/// Automap exploration levels (C++ `MapExplorationType`).
pub const MAP_EXP_NONE: u8 = 0;
pub const MAP_EXP_OLD: u8 = 1;
pub const MAP_EXP_SHRINE: u8 = 2;
pub const MAP_EXP_OTHERS: u8 = 3;
pub const MAP_EXP_SELF: u8 = 4;

/// A tile's automap shape (C++ `AutomapTile`, automap.cpp:60).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutomapTile {
    /// `AutomapTile::Types` discriminant (automap.cpp:62-115).
    pub kind: u8,
    /// `AutomapTile::Flags` bitmask (automap.cpp:117-129).
    pub flags: u8,
}

impl Default for AutomapTile {
    fn default() -> Self {
        Self { kind: 0, flags: 0 }
    }
}

impl AutomapTile {
    /// `AutomapTile::Flags::Dirt` = 1 << 6.
    pub const DIRT: u8 = 1 << 6;
    /// `AutomapTile::Flags::VerticalArch` = 1 << 2.
    pub const VERTICAL_ARCH: u8 = 1 << 2;
    /// `AutomapTile::Flags::HorizontalArch` = 1 << 3.
    pub const HORIZONTAL_ARCH: u8 = 1 << 3;

    pub const fn has_flag(self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }
}

/// `AutomapTile::Types` discriminants used by `SetAutomapView`.
pub const TYPE_NONE: u8 = 0;
pub const TYPE_DIAMOND: u8 = 1;
pub const TYPE_VERTICAL: u8 = 2;
pub const TYPE_HORIZONTAL: u8 = 3;
pub const TYPE_CROSS: u8 = 4;
pub const TYPE_FENCE_VERTICAL: u8 = 5;
pub const TYPE_FENCE_HORIZONTAL: u8 = 6;
pub const TYPE_CORNER: u8 = 7;

/// Parse a `.amp` file into per-tile automap shapes.
///
/// C++ `InitAutomap` (automap.cpp:1579-1700): the raw `AutomapTile` array is
/// installed at index `i + 1` (the logical tile ids are 1-based).
pub fn parse_amp(data: &[u8]) -> Vec<AutomapTile> {
    data.chunks_exact(2)
        .map(|c| AutomapTile {
            kind: c[0],
            flags: c[1],
        })
        .collect()
}

/// Build the 256-entry `AutomapTypeTiles` table (C++ automap.cpp:162,1692).
pub fn build_type_tiles(amp: &[AutomapTile]) -> [AutomapTile; 256] {
    let mut table = [AutomapTile::default(); 256];
    for (i, tile) in amp.iter().enumerate() {
        if i + 1 < 256 {
            table[i + 1] = *tile;
        }
    }
    table
}

/// C++ `GetAutomapTileType` (automap.cpp:927-941): look up the shape of a
/// logical tile, upgrading `Corner` to `Diamond` next to arch passages.
pub fn get_automap_tile_type(dungeon: &[[u8; 40]; 40], table: &[AutomapTile; 256], x: i32, y: i32) -> AutomapTile {
    if x < 0 || x >= 40 || y < 0 || y >= 40 {
        return AutomapTile::default();
    }
    let tv = dungeon[y as usize][x as usize] as usize;
    let mut tile = *table.get(tv).unwrap_or(&AutomapTile::default());
    if tile.kind == TYPE_CORNER {
        if x - 1 >= 0
            && get_automap_tile_type(dungeon, table, x - 1, y).has_flag(AutomapTile::HORIZONTAL_ARCH)
            && y - 1 >= 0
            && get_automap_tile_type(dungeon, table, x, y - 1).has_flag(AutomapTile::VERTICAL_ARCH)
        {
            tile.kind = TYPE_DIAMOND;
        }
    }
    tile
}

/// C++ `HasAutomapFlag` (automap.cpp:915-924).
fn has_automap_flag(dungeon: &[[u8; 40]; 40], table: &[AutomapTile; 256], x: i32, y: i32, flag: u8) -> bool {
    if x < 0 || x >= 40 || y < 0 || y >= 40 {
        return false;
    }
    let tv = dungeon[y as usize][x as usize] as usize;
    table
        .get(tv)
        .map_or(false, |t| t.has_flag(flag))
}

/// C++ `UpdateAutomapExplorer` (automap.cpp:1877-1881).
fn update_explorer(view: &mut [[u8; 40]; 40], x: i32, y: i32, explorer: u8) {
    if x >= 0 && x < 40 && y >= 0 && y < 40 && view[y as usize][x as usize] < explorer {
        view[y as usize][x as usize] = explorer;
    }
}

/// C++ `SetAutomapView` (automap.cpp:1883-1960): mark the logical tile under a
/// micro position explored and expand onto neighbouring wall tiles per the
/// automap shape rules.
pub fn set_automap_view(
    dungeon: &[[u8; 40]; 40],
    table: &[AutomapTile; 256],
    view: &mut [[u8; 40]; 40],
    px: i32,
    py: i32,
    explorer: u8,
) {
    let mx = (px - 16) / 2;
    let my = (py - 16) / 2;
    if mx < 0 || mx >= 40 || my < 0 || my >= 40 {
        return;
    }
    update_explorer(view, mx, my, explorer);
    let tile = get_automap_tile_type(dungeon, table, mx, my);
    let solid = tile.has_flag(AutomapTile::DIRT);
    match tile.kind {
        TYPE_VERTICAL => {
            if solid {
                let sw = get_automap_tile_type(dungeon, table, mx, my + 1);
                if sw.kind == TYPE_CORNER && sw.has_flag(AutomapTile::DIRT) {
                    update_explorer(view, mx, my + 1, explorer);
                }
            } else if has_automap_flag(dungeon, table, mx - 1, my, AutomapTile::DIRT) {
                update_explorer(view, mx - 1, my, explorer);
            }
        }
        TYPE_HORIZONTAL => {
            if solid {
                let se = get_automap_tile_type(dungeon, table, mx + 1, my);
                if se.kind == TYPE_CORNER && se.has_flag(AutomapTile::DIRT) {
                    update_explorer(view, mx + 1, my, explorer);
                }
            } else if has_automap_flag(dungeon, table, mx, my - 1, AutomapTile::DIRT) {
                update_explorer(view, mx, my - 1, explorer);
            }
        }
        TYPE_CROSS => {
            if solid {
                let sw = get_automap_tile_type(dungeon, table, mx, my + 1);
                if sw.kind == TYPE_CORNER && sw.has_flag(AutomapTile::DIRT) {
                    update_explorer(view, mx, my + 1, explorer);
                }
                let se = get_automap_tile_type(dungeon, table, mx + 1, my);
                if se.kind == TYPE_CORNER && se.has_flag(AutomapTile::DIRT) {
                    update_explorer(view, mx + 1, my, explorer);
                }
            } else {
                if has_automap_flag(dungeon, table, mx - 1, my, AutomapTile::DIRT) {
                    update_explorer(view, mx - 1, my, explorer);
                }
                if has_automap_flag(dungeon, table, mx, my - 1, AutomapTile::DIRT) {
                    update_explorer(view, mx, my - 1, explorer);
                }
                if has_automap_flag(dungeon, table, mx - 1, my - 1, AutomapTile::DIRT) {
                    update_explorer(view, mx - 1, my - 1, explorer);
                }
            }
        }
        TYPE_FENCE_VERTICAL => {
            if solid {
                if has_automap_flag(dungeon, table, mx, my - 1, AutomapTile::DIRT) {
                    update_explorer(view, mx, my - 1, explorer);
                }
                let sw = get_automap_tile_type(dungeon, table, mx, my + 1);
                if sw.kind == TYPE_CORNER && sw.has_flag(AutomapTile::DIRT) {
                    update_explorer(view, mx, my + 1, explorer);
                }
            } else if has_automap_flag(dungeon, table, mx - 1, my, AutomapTile::DIRT) {
                update_explorer(view, mx - 1, my, explorer);
            }
        }
        TYPE_FENCE_HORIZONTAL => {
            if solid {
                if has_automap_flag(dungeon, table, mx - 1, my, AutomapTile::DIRT) {
                    update_explorer(view, mx - 1, my, explorer);
                }
                let se = get_automap_tile_type(dungeon, table, mx + 1, my);
                if se.kind == TYPE_CORNER && se.has_flag(AutomapTile::DIRT) {
                    update_explorer(view, mx + 1, my, explorer);
                }
            } else if has_automap_flag(dungeon, table, mx, my - 1, AutomapTile::DIRT) {
                update_explorer(view, mx, my - 1, explorer);
            }
        }
        _ => {}
    }
}

/// Run `SetAutomapView(MAP_EXP_SELF)` for every explored micro-tile.
///
/// `explored` is the 112x112 `dFlags & (Lit|Explored)` set; `dungeon` is the
/// 40x40 logical tile grid. Returns the 40x40 `AutomapView` grid.
pub fn compute_automap_view(
    dungeon: &[[u8; 40]; 40],
    table: &[AutomapTile; 256],
    explored: &[bool],
) -> [[u8; 40]; 40] {
    let mut view = [[0u8; 40]; 40];
    for (i, &e) in explored.iter().enumerate() {
        if !e {
            continue;
        }
        let x = (i % 112) as i32;
        let y = (i / 112) as i32;
        set_automap_view(dungeon, table, &mut view, x, y, MAP_EXP_SELF);
    }
    view
}
