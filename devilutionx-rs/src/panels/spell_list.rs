//! Spell List - Quick Spell Selection Panel (M64)
//!
//! Provides the spell list panel for quick spell selection:
//! - List of available spells
//! - Quick selection for spell casting
//! - Spell filtering by type
//!
//! ## C++ References
//! - Source/panels/spell_list.cpp
//! - Source/panels/spell_list.hpp

#![allow(dead_code)]
#![allow(unused_imports)]

use super::main_panel::{Point, Size, Rectangle};
use super::spell_book::{SpellId, SpellType};
use super::spell_icons::get_spell_icon_frame;

/// Maximum number of spells in the spell list
pub const MAX_SPELL_LIST_ENTRIES: usize = 52;

/// Spell list display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellListMode {
    /// Show all available spells
    All,
    /// Show only learned spells
    Learned,
    /// Show only skills
    Skills,
    /// Show only scrolls/items
    Items,
}

/// A single entry in the spell list
#[derive(Debug, Clone, Copy)]
pub struct SpellListEntry {
    /// Spell ID
    pub spell: SpellId,
    /// How the spell is available
    pub spell_type: SpellType,
    /// Is this entry visible (has the spell)?
    pub visible: bool,
    /// Is this entry selected?
    pub selected: bool,
    /// Spell level (for learned spells)
    pub level: i32,
}

impl SpellListEntry {
    pub fn new(spell: SpellId) -> Self {
        Self {
            spell,
            spell_type: SpellType::Invalid,
            visible: false,
            selected: false,
            level: 0,
        }
    }

    pub fn set_available(&mut self, spell_type: SpellType, level: i32) {
        self.spell_type = spell_type;
        self.level = level;
        self.visible = spell_type != SpellType::Invalid;
    }

    pub fn clear(&mut self) {
        self.spell_type = SpellType::Invalid;
        self.visible = false;
        self.selected = false;
        self.level = 0;
    }
}

impl Default for SpellListEntry {
    fn default() -> Self {
        Self::new(SpellId::Null)
    }
}

/// Spell list panel dimensions
pub const SPELL_LIST_WIDTH: i32 = 56;
pub const SPELL_LIST_ICON_SIZE: i32 = 56;
pub const SPELL_LIST_SPACING: i32 = 2;

/// Spell list direction (where it expands from)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellListDirection {
    /// Expands upward from bottom
    Up,
    /// Expands downward from top
    Down,
    /// Expands left from right
    Left,
    /// Expands right from left
    Right,
}

/// Spell list panel
#[derive(Debug, Clone)]
pub struct SpellList {
    /// Panel position (anchor point)
    pub position: Point,
    /// Is panel visible?
    pub visible: bool,
    /// Current display mode
    pub mode: SpellListMode,
    /// Expansion direction
    pub direction: SpellListDirection,
    /// Spell entries
    pub entries: Vec<SpellListEntry>,
    /// Number of visible entries
    pub visible_count: usize,
    /// Currently selected entry index
    pub selected_index: Option<usize>,
    /// Is Hellfire mode?
    pub is_hellfire: bool,
    /// Maximum visible entries at once
    pub max_visible: usize,
    /// Scroll offset (for long lists)
    pub scroll_offset: usize,
}

impl SpellList {
    /// Create a new spell list
    pub fn new() -> Self {
        Self {
            position: Point::new(0, 0),
            visible: false,
            mode: SpellListMode::All,
            direction: SpellListDirection::Up,
            entries: Vec::with_capacity(MAX_SPELL_LIST_ENTRIES),
            visible_count: 0,
            selected_index: None,
            is_hellfire: false,
            max_visible: 8,
            scroll_offset: 0,
        }
    }

    /// Show the spell list
    pub fn show(&mut self) {
        self.visible = true;
        self.scroll_offset = 0;
    }

    /// Hide the spell list
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// Set display mode
    pub fn set_mode(&mut self, mode: SpellListMode) {
        self.mode = mode;
        self.update_visible_count();
    }

    /// Set expansion direction
    pub fn set_direction(&mut self, direction: SpellListDirection) {
        self.direction = direction;
    }

    /// Add a spell entry
    pub fn add_spell(&mut self, spell: SpellId, spell_type: SpellType, level: i32) {
        // Check if spell already exists
        for entry in &mut self.entries {
            if entry.spell == spell {
                entry.set_available(spell_type, level);
                self.update_visible_count();
                return;
            }
        }

        // Add new entry
        let mut entry = SpellListEntry::new(spell);
        entry.set_available(spell_type, level);
        self.entries.push(entry);
        self.update_visible_count();
    }

    /// Remove a spell entry
    pub fn remove_spell(&mut self, spell: SpellId) {
        for entry in &mut self.entries {
            if entry.spell == spell {
                entry.clear();
                break;
            }
        }
        self.update_visible_count();
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.visible_count = 0;
        self.selected_index = None;
    }

    /// Update visible entry count based on mode
    fn update_visible_count(&mut self) {
        self.visible_count = self.entries.iter().filter(|e| self.is_entry_visible(e)).count();
    }

    /// Check if entry is visible based on current mode
    fn is_entry_visible(&self, entry: &SpellListEntry) -> bool {
        if !entry.visible {
            return false;
        }

        match self.mode {
            SpellListMode::All => true,
            SpellListMode::Learned => entry.spell_type == SpellType::Spell,
            SpellListMode::Skills => entry.spell_type == SpellType::Skill,
            SpellListMode::Items => {
                entry.spell_type == SpellType::Scroll || entry.spell_type == SpellType::Charges
            }
        }
    }

    /// Get visible entries
    pub fn get_visible_entries(&self) -> Vec<&SpellListEntry> {
        self.entries
            .iter()
            .filter(|e| self.is_entry_visible(e))
            .skip(self.scroll_offset)
            .take(self.max_visible)
            .collect()
    }

    /// Select spell at index
    pub fn select(&mut self, index: usize) {
        // Deselect all
        for entry in &mut self.entries {
            entry.selected = false;
        }

        // Select at index
        let visible: Vec<usize> = self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.is_entry_visible(e))
            .map(|(i, _)| i)
            .collect();

        if let Some(&entry_idx) = visible.get(self.scroll_offset + index) {
            self.entries[entry_idx].selected = true;
            self.selected_index = Some(entry_idx);
        }
    }

    /// Get currently selected spell
    pub fn get_selected(&self) -> Option<(SpellId, SpellType)> {
        self.selected_index.and_then(|idx| {
            self.entries.get(idx).map(|e| (e.spell, e.spell_type))
        })
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.visible_count == 0 {
            return;
        }

        if let Some(idx) = self.selected_index {
            let visible: Vec<usize> = self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| self.is_entry_visible(e))
                .map(|(i, _)| i)
                .collect();

            if let Some(pos) = visible.iter().position(|&i| i == idx) {
                if pos > 0 {
                    self.entries[idx].selected = false;
                    let new_idx = visible[pos - 1];
                    self.entries[new_idx].selected = true;
                    self.selected_index = Some(new_idx);

                    // Adjust scroll if needed
                    if pos - 1 < self.scroll_offset {
                        self.scroll_offset = pos - 1;
                    }
                }
            }
        } else if self.visible_count > 0 {
            self.select(0);
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.visible_count == 0 {
            return;
        }

        if let Some(idx) = self.selected_index {
            let visible: Vec<usize> = self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| self.is_entry_visible(e))
                .map(|(i, _)| i)
                .collect();

            if let Some(pos) = visible.iter().position(|&i| i == idx) {
                if pos + 1 < visible.len() {
                    self.entries[idx].selected = false;
                    let new_idx = visible[pos + 1];
                    self.entries[new_idx].selected = true;
                    self.selected_index = Some(new_idx);

                    // Adjust scroll if needed
                    if pos + 1 >= self.scroll_offset + self.max_visible {
                        self.scroll_offset = pos + 2 - self.max_visible;
                    }
                }
            }
        } else if self.visible_count > 0 {
            self.select(0);
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        if self.scroll_offset + self.max_visible < self.visible_count {
            self.scroll_offset += 1;
        }
    }

    /// Can scroll up?
    pub fn can_scroll_up(&self) -> bool {
        self.scroll_offset > 0
    }

    /// Can scroll down?
    pub fn can_scroll_down(&self) -> bool {
        self.scroll_offset + self.max_visible < self.visible_count
    }

    /// Get rectangle for entry at visual index
    pub fn get_entry_rect(&self, visual_index: usize) -> Rectangle {
        let offset = visual_index as i32;

        match self.direction {
            SpellListDirection::Up => Rectangle::new(
                self.position.x,
                self.position.y - (offset + 1) * (SPELL_LIST_ICON_SIZE + SPELL_LIST_SPACING),
                SPELL_LIST_ICON_SIZE,
                SPELL_LIST_ICON_SIZE,
            ),
            SpellListDirection::Down => Rectangle::new(
                self.position.x,
                self.position.y + offset * (SPELL_LIST_ICON_SIZE + SPELL_LIST_SPACING),
                SPELL_LIST_ICON_SIZE,
                SPELL_LIST_ICON_SIZE,
            ),
            SpellListDirection::Left => Rectangle::new(
                self.position.x - (offset + 1) * (SPELL_LIST_ICON_SIZE + SPELL_LIST_SPACING),
                self.position.y,
                SPELL_LIST_ICON_SIZE,
                SPELL_LIST_ICON_SIZE,
            ),
            SpellListDirection::Right => Rectangle::new(
                self.position.x + offset * (SPELL_LIST_ICON_SIZE + SPELL_LIST_SPACING),
                self.position.y,
                SPELL_LIST_ICON_SIZE,
                SPELL_LIST_ICON_SIZE,
            ),
        }
    }

    /// Hit test at position
    pub fn hit_test(&self, x: i32, y: i32) -> Option<usize> {
        if !self.visible {
            return None;
        }

        let point = Point::new(x, y);

        for i in 0..self.max_visible.min(self.visible_count - self.scroll_offset) {
            let rect = self.get_entry_rect(i);
            if rect.contains(point) {
                return Some(i);
            }
        }

        None
    }

    /// Get total panel height
    pub fn get_panel_height(&self) -> i32 {
        let count = self.visible_count.min(self.max_visible) as i32;
        count * (SPELL_LIST_ICON_SIZE + SPELL_LIST_SPACING) - SPELL_LIST_SPACING
    }
}

impl Default for SpellList {
    fn default() -> Self {
        Self::new()
    }
}

/// Player spell data for spell list population
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerSpellData {
    /// Bitmask of memory spells (learned from books)
    pub mem_spells: u64,
    /// Bitmask of innate abilities (class skills)
    pub abl_spells: u64,
    /// Bitmask of item spells (scrolls, staff charges)
    pub item_spells: u64,
}

impl PlayerSpellData {
    /// Check if player has spell
    pub fn has_spell(&self, spell: SpellId) -> bool {
        let mask = 1u64 << (spell as u8);
        (self.mem_spells & mask) != 0
            || (self.abl_spells & mask) != 0
            || (self.item_spells & mask) != 0
    }

    /// Get spell type for player
    pub fn get_spell_type(&self, spell: SpellId) -> SpellType {
        let mask = 1u64 << (spell as u8);

        if (self.abl_spells & mask) != 0 {
            SpellType::Skill
        } else if (self.mem_spells & mask) != 0 {
            SpellType::Spell
        } else if (self.item_spells & mask) != 0 {
            // Could be scroll or staff, need to check inventory
            SpellType::Scroll
        } else {
            SpellType::Invalid
        }
    }

    /// Get all available spells as bitmask
    pub fn all_spells(&self) -> u64 {
        self.mem_spells | self.abl_spells | self.item_spells
    }
}

/// Populate spell list from player data
pub fn populate_spell_list(list: &mut SpellList, player_data: &PlayerSpellData, spell_levels: &[i32]) {
    list.clear();

    let all_spells = player_data.all_spells();

    for spell_idx in 1..=51 {
        let mask = 1u64 << spell_idx;
        if (all_spells & mask) != 0 {
            // Convert index to SpellId (this is simplified)
            let spell_id = match spell_idx {
                1 => SpellId::Firebolt,
                2 => SpellId::Healing,
                3 => SpellId::Lightning,
                4 => SpellId::Flash,
                5 => SpellId::Identify,
                6 => SpellId::FireWall,
                7 => SpellId::TownPortal,
                8 => SpellId::StoneCurse,
                9 => SpellId::Infravision,
                10 => SpellId::Phasing,
                11 => SpellId::ManaShield,
                12 => SpellId::Fireball,
                13 => SpellId::Guardian,
                14 => SpellId::ChainLightning,
                15 => SpellId::FlameWave,
                18 => SpellId::Nova,
                20 => SpellId::Inferno,
                21 => SpellId::Golem,
                23 => SpellId::Teleport,
                24 => SpellId::Apocalypse,
                29 => SpellId::Elemental,
                30 => SpellId::ChargedBolt,
                31 => SpellId::HolyBolt,
                32 => SpellId::Resurrect,
                33 => SpellId::Telekinesis,
                34 => SpellId::HealOther,
                35 => SpellId::BloodStar,
                36 => SpellId::BoneSpirit,
                40 => SpellId::LightningWall,
                41 => SpellId::Immolation,
                42 => SpellId::Warp,
                43 => SpellId::Reflect,
                44 => SpellId::Berserk,
                45 => SpellId::RingOfFire,
                46 => SpellId::Search,
                _ => continue,
            };

            let spell_type = player_data.get_spell_type(spell_id);
            let level = spell_levels.get(spell_idx as usize).copied().unwrap_or(0);
            list.add_spell(spell_id, spell_type, level);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_list_entry() {
        let mut entry = SpellListEntry::new(SpellId::Firebolt);
        assert!(!entry.visible);

        entry.set_available(SpellType::Spell, 5);
        assert!(entry.visible);
        assert_eq!(entry.level, 5);

        entry.clear();
        assert!(!entry.visible);
    }

    #[test]
    fn test_spell_list_new() {
        let list = SpellList::new();
        assert!(!list.visible);
        assert_eq!(list.visible_count, 0);
        assert_eq!(list.mode, SpellListMode::All);
    }

    #[test]
    fn test_spell_list_add() {
        let mut list = SpellList::new();
        list.add_spell(SpellId::Firebolt, SpellType::Spell, 3);
        list.add_spell(SpellId::Healing, SpellType::Skill, 1);

        assert_eq!(list.visible_count, 2);
        assert_eq!(list.entries.len(), 2);
    }

    #[test]
    fn test_spell_list_mode() {
        let mut list = SpellList::new();
        list.add_spell(SpellId::Firebolt, SpellType::Spell, 3);
        list.add_spell(SpellId::Healing, SpellType::Skill, 1);

        list.set_mode(SpellListMode::Learned);
        assert_eq!(list.visible_count, 1);

        list.set_mode(SpellListMode::Skills);
        assert_eq!(list.visible_count, 1);

        list.set_mode(SpellListMode::All);
        assert_eq!(list.visible_count, 2);
    }

    #[test]
    fn test_spell_list_selection() {
        let mut list = SpellList::new();
        list.add_spell(SpellId::Firebolt, SpellType::Spell, 3);
        list.add_spell(SpellId::Healing, SpellType::Spell, 5);
        list.add_spell(SpellId::Lightning, SpellType::Spell, 2);

        list.select(1);
        let selected = list.get_selected();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().0, SpellId::Healing);
    }

    #[test]
    fn test_spell_list_navigation() {
        let mut list = SpellList::new();
        list.add_spell(SpellId::Firebolt, SpellType::Spell, 3);
        list.add_spell(SpellId::Healing, SpellType::Spell, 5);
        list.add_spell(SpellId::Lightning, SpellType::Spell, 2);

        list.select(0);
        assert_eq!(list.get_selected().unwrap().0, SpellId::Firebolt);

        list.move_down();
        assert_eq!(list.get_selected().unwrap().0, SpellId::Healing);

        list.move_up();
        assert_eq!(list.get_selected().unwrap().0, SpellId::Firebolt);
    }

    #[test]
    fn test_player_spell_data() {
        let data = PlayerSpellData {
            mem_spells: 0b110, // Firebolt (1) and Healing (2)
            abl_spells: 0b1000, // Lightning (3)
            item_spells: 0,
        };

        assert!(data.has_spell(SpellId::Firebolt));
        assert!(!data.has_spell(SpellId::TownPortal));

        assert_eq!(data.get_spell_type(SpellId::Lightning), SpellType::Skill);
        assert_eq!(data.get_spell_type(SpellId::Firebolt), SpellType::Spell);
    }

    #[test]
    fn test_spell_list_scroll() {
        let mut list = SpellList::new();
        list.max_visible = 3;

        for i in 0..5 {
            let spell = match i {
                0 => SpellId::Firebolt,
                1 => SpellId::Healing,
                2 => SpellId::Lightning,
                3 => SpellId::Flash,
                _ => SpellId::TownPortal,
            };
            list.add_spell(spell, SpellType::Spell, 1);
        }

        assert!(list.can_scroll_down());
        assert!(!list.can_scroll_up());

        list.scroll_down();
        assert!(list.can_scroll_up());

        list.scroll_down();
        assert!(!list.can_scroll_down());
    }
}
