//! Exact Monster system module (adapter).
//!
//! During the module consolidation the original `monster_exact.rs` (1,743
//! lines) was merged into [`crate::game::monster`]. This file re-exports the
//! consolidated types so existing `use crate::game::monster_exact::...`
//! imports keep compiling, and provides [`MonsterManager`] — the collection
//! that owns the active monster slots (max 200, matching C++).
//!
//! **C++ Reference**: `Monsters[MAXMONSTERS]` array in `Source/monster.cpp`.

// Re-export the consolidated monster types (merged in from this module).
pub use crate::game::monster::{Monster, MonsterMode};

use crate::game::types::Point;

/// Owner of the active monster slots (C++ `Monsters[]`, max 200).
pub struct MonsterManager {
    /// Active monsters (max 200 in C++).
    monsters: Vec<Option<Monster>>,
    /// Maximum number of monsters.
    max_monsters: usize,
    /// Count of active monsters.
    active_count: usize,
}

impl MonsterManager {
    /// Maximum monsters constant (Source/monster.h:35).
    pub const MAX_MONSTERS: usize = 200;

    /// Create a new `MonsterManager` pre-allocated with `max_count` empty slots.
    ///
    /// * `max_count` - Maximum monsters (default 200).
    pub fn new(max_count: usize) -> Self {
        let mut monsters = Vec::with_capacity(max_count);
        for _ in 0..max_count {
            monsters.push(None);
        }
        Self {
            monsters,
            max_monsters: max_count,
            active_count: 0,
        }
    }

    /// Add a monster to the first free slot.
    ///
    /// Returns the index it was placed at, or `None` if the manager is full.
    pub fn add_monster(&mut self, monster: Monster) -> Option<usize> {
        for (i, slot) in self.monsters.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(monster);
                self.active_count += 1;
                return Some(i);
            }
        }
        None
    }

    /// Remove the monster at `index` (no-op if empty/out of range).
    pub fn remove_monster(&mut self, index: usize) {
        if index < self.monsters.len() && self.monsters[index].is_some() {
            self.monsters[index] = None;
            self.active_count = self.active_count.saturating_sub(1);
        }
    }

    /// Immutable reference to the monster at `index`.
    pub fn get_monster(&self, index: usize) -> Option<&Monster> {
        self.monsters.get(index)?.as_ref()
    }

    /// Mutable reference to the monster at `index`.
    pub fn get_monster_mut(&mut self, index: usize) -> Option<&mut Monster> {
        self.monsters.get_mut(index)?.as_mut()
    }

    /// Find the index of the living monster at `pos`, if any.
    pub fn find_monster_at(&self, pos: Point) -> Option<usize> {
        for (i, monster_opt) in self.monsters.iter().enumerate() {
            if let Some(monster) = monster_opt {
                if monster.is_alive() && monster.x == pos.x && monster.y == pos.y {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Indices of all living monsters within `range` (Manhattan-ish) of `center`.
    pub fn get_monsters_in_range(&self, center: Point, range: i32) -> Vec<usize> {
        let mut result = Vec::new();
        for (i, monster_opt) in self.monsters.iter().enumerate() {
            if let Some(monster) = monster_opt {
                if monster.is_alive() && monster.distance_to(center.x, center.y) <= range {
                    result.push(i);
                }
            }
        }
        result
    }

    /// Remove all dead monsters from their slots.
    pub fn cleanup_dead(&mut self) {
        for slot in self.monsters.iter_mut() {
            if let Some(monster) = slot {
                if !monster.is_alive() {
                    *slot = None;
                    self.active_count = self.active_count.saturating_sub(1);
                }
            }
        }
    }

    /// Number of active monsters.
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Whether the manager is at capacity.
    pub fn is_full(&self) -> bool {
        self.active_count >= self.max_monsters
    }

    /// Maximum capacity.
    pub fn capacity(&self) -> usize {
        self.max_monsters
    }

    /// Clear all monsters.
    pub fn clear(&mut self) {
        for slot in self.monsters.iter_mut() {
            *slot = None;
        }
        self.active_count = 0;
    }

    /// Iterate over `(index, &Monster)` for all active monsters.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Monster)> {
        self.monsters
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|m| (i, m)))
    }

    /// Iterate over `(index, &mut Monster)` for all active monsters.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut Monster)> {
        self.monsters
            .iter_mut()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_mut().map(|m| (i, m)))
    }
}
