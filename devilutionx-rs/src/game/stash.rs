//! Shared stash QoL (C++ `Source/qol/stash.h` / `stash.cpp`).
//!
//! Ports the `StashStruct` data model: a per-page 10x10 item grid, the item
//! list, gold, page navigation, and grid placement/lookup. The drag/drop UI
//! and rendering are follow-ups.

use std::collections::HashMap;

use crate::game::items::Item;

/// C++ `StashStruct::EmptyCell`.
pub const EMPTY_CELL: i16 = -1;
/// C++ `StashGridSize { 10, 10 }`.
pub const STASH_GRID_SIZE: usize = 10;
/// C++ `CountStashPages`.
pub const STASH_PAGES: u32 = 100;

/// A stash grid cell value: `-1` = empty, otherwise `item_list_index + 1`.
pub type StashCell = i16;
pub type StashGrid = [[StashCell; STASH_GRID_SIZE]; STASH_GRID_SIZE];

/// C++ `StashStruct`.
#[derive(Debug, Clone)]
pub struct Stash {
    /// Per-page grids (C++ `stashGrids`), created on first access.
    grids: HashMap<u32, StashGrid>,
    /// The stash items (C++ `stashList`).
    pub list: Vec<Item>,
    /// C++ `gold`.
    pub gold: i32,
    /// C++ `dirty`.
    pub dirty: bool,
    /// Current page (C++ private `page`).
    page: u32,
}

impl Default for Stash {
    fn default() -> Self {
        Self::new()
    }
}

impl Stash {
    pub fn new() -> Self {
        Self {
            grids: HashMap::new(),
            list: Vec::new(),
            gold: 0,
            dirty: false,
            page: 0,
        }
    }

    pub fn page(&self) -> u32 {
        self.page
    }

    /// C++ `GetCurrentGrid()`.
    pub fn current_grid(&self) -> &StashGrid {
        self.grids.get(&self.page).unwrap_or(&EMPTY_GRID)
    }

    /// C++ `GetCurrentGrid()` (mutable).
    pub fn current_grid_mut(&mut self) -> &mut StashGrid {
        self.grids.entry(self.page).or_insert(EMPTY_GRID)
    }

    /// C++ `GetItemIdAtPosition`: the 0-based item-list index at (x, y), or
    /// `EMPTY_CELL` when empty/out of bounds.
    pub fn get_item_id_at_position(&self, x: usize, y: usize) -> StashCell {
        if x >= STASH_GRID_SIZE || y >= STASH_GRID_SIZE {
            return EMPTY_CELL;
        }
        self.current_grid()[x][y] - 1
    }

    /// C++ `IsItemAtPosition`.
    pub fn is_item_at_position(&self, x: usize, y: usize) -> bool {
        self.get_item_id_at_position(x, y) != EMPTY_CELL
    }

    /// C++ `AddItemToStashGrid`: fill the item's rectangle with
    /// `list_index + 1`.
    pub fn add_item_to_grid(&mut self, position_x: usize, position_y: usize, list_index: usize, width: usize, height: usize) {
        let grid = self.current_grid_mut();
        for dy in 0..height {
            for dx in 0..width {
                let gx = position_x + dx;
                let gy = position_y + dy;
                if gx < STASH_GRID_SIZE && gy < STASH_GRID_SIZE {
                    grid[gx][gy] = list_index as StashCell + 1;
                }
            }
        }
    }

    /// C++ `RemoveStashItem(iv)`: clear the item's grid cells.
    pub fn remove_stash_item(&mut self, list_index: usize) {
        let value = list_index as StashCell + 1;
        let grid = self.current_grid_mut();
        for cell in grid.iter_mut().flat_map(|col| col.iter_mut()) {
            if *cell == value {
                *cell = 0;
            }
        }
    }

    /// C++ `SetPage(newPage)` (clamped to [0, CountStashPages)).
    pub fn set_page(&mut self, new_page: u32) {
        self.page = new_page % STASH_PAGES;
    }

    /// C++ `NextPage(offset)`.
    pub fn next_page(&mut self, offset: u32) {
        self.set_page(self.page + offset);
    }

    /// C++ `PreviousPage(offset)` (wraps down through zero).
    pub fn previous_page(&mut self, offset: u32) {
        self.set_page(self.page + STASH_PAGES - (offset % STASH_PAGES));
    }
}

/// C++ stores `0` in empty cells; `GetItemIdAtPosition` subtracts one, so an
/// empty stored cell becomes `-1` (`EMPTY_CELL`).
const EMPTY_GRID: StashGrid = [[0; STASH_GRID_SIZE]; STASH_GRID_SIZE];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::items::{Item, ItemType};

    #[test]
    fn test_empty_stash_lookups() {
        let stash = Stash::new();
        assert_eq!(stash.page(), 0);
        assert!(!stash.is_item_at_position(0, 0));
        assert_eq!(stash.get_item_id_at_position(0, 0), EMPTY_CELL);
        assert_eq!(stash.get_item_id_at_position(99, 99), EMPTY_CELL);
    }

    #[test]
    fn test_add_and_lookup_item() {
        let mut stash = Stash::new();
        stash.list.push(Item {
            item_type: ItemType::Sword,
            ..Default::default()
        });
        // A 2x1 item anchored at (1, 2).
        stash.add_item_to_grid(1, 2, 0, 2, 1);
        assert_eq!(stash.get_item_id_at_position(1, 2), 0);
        assert_eq!(stash.get_item_id_at_position(2, 2), 0);
        assert!(stash.is_item_at_position(2, 2));
        assert_eq!(stash.get_item_id_at_position(3, 2), EMPTY_CELL);
    }

    #[test]
    fn test_remove_item_clears_cells() {
        let mut stash = Stash::new();
        stash.list.push(Item::default());
        stash.add_item_to_grid(0, 0, 0, 2, 2);
        assert!(stash.is_item_at_position(1, 1));
        stash.remove_stash_item(0);
        assert!(!stash.is_item_at_position(0, 0));
        assert!(!stash.is_item_at_position(1, 1));
    }

    #[test]
    fn test_page_navigation_wraps() {
        let mut stash = Stash::new();
        stash.next_page(1);
        assert_eq!(stash.page(), 1);
        stash.previous_page(1);
        assert_eq!(stash.page(), 0);
        // Wrap backwards from page 0.
        stash.previous_page(1);
        assert_eq!(stash.page(), STASH_PAGES - 1);
        // Wrap forwards past the last page.
        stash.next_page(2);
        assert_eq!(stash.page(), 1);
        // Pages have independent grids.
        stash.set_page(2);
        stash.list.push(Item::default());
        stash.add_item_to_grid(0, 0, 0, 1, 1);
        assert!(stash.is_item_at_position(0, 0));
        stash.set_page(0);
        assert!(!stash.is_item_at_position(0, 0), "page 0 grid is separate");
    }
}
