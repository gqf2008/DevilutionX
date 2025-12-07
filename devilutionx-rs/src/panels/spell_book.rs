//! Spell Book - Spell Learning and Casting Interface (M64)
//!
//! Provides the spell book panel for:
//! - Viewing learned spells
//! - Spell level and mana cost display
//! - Spell page navigation
//! - Spell selection for casting
//!
//! ## C++ References
//! - Source/panels/spell_book.cpp
//! - Source/panels/spell_book.hpp

#![allow(dead_code)]
#![allow(unused_imports)]

use super::main_panel::{Point, Size, Rectangle};
use super::char_panel::UiFlags;

/// Spell ID - matches C++ SpellID enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum SpellId {
    Null = 0,
    Firebolt = 1,
    Healing = 2,
    Lightning = 3,
    Flash = 4,
    Identify = 5,
    FireWall = 6,
    TownPortal = 7,
    StoneCurse = 8,
    Infravision = 9,
    Phasing = 10,
    ManaShield = 11,
    Fireball = 12,
    Guardian = 13,
    ChainLightning = 14,
    FlameWave = 15,
    DoomSerpents = 16,
    BloodRitual = 17,
    Nova = 18,
    Invisibility = 19,
    Inferno = 20,
    Golem = 21,
    Rage = 22,
    Teleport = 23,
    Apocalypse = 24,
    Etherealize = 25,
    ItemRepair = 26,
    StaffRecharge = 27,
    TrapDisarm = 28,
    Elemental = 29,
    ChargedBolt = 30,
    HolyBolt = 31,
    Resurrect = 32,
    Telekinesis = 33,
    HealOther = 34,
    BloodStar = 35,
    BoneSpirit = 36,
    Mana = 37,
    Magi = 38,
    Jester = 39,
    // Hellfire spells
    LightningWall = 40,
    Immolation = 41,
    Warp = 42,
    Reflect = 43,
    Berserk = 44,
    RingOfFire = 45,
    Search = 46,
    // Runes
    RuneOfFire = 47,
    RuneOfLight = 48,
    RuneOfNova = 49,
    RuneOfImmolation = 50,
    RuneOfStone = 51,
    Invalid = -1,
}

impl SpellId {
    /// Get spell name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Null => "",
            Self::Firebolt => "Firebolt",
            Self::Healing => "Healing",
            Self::Lightning => "Lightning",
            Self::Flash => "Flash",
            Self::Identify => "Identify",
            Self::FireWall => "Fire Wall",
            Self::TownPortal => "Town Portal",
            Self::StoneCurse => "Stone Curse",
            Self::Infravision => "Infravision",
            Self::Phasing => "Phasing",
            Self::ManaShield => "Mana Shield",
            Self::Fireball => "Fireball",
            Self::Guardian => "Guardian",
            Self::ChainLightning => "Chain Lightning",
            Self::FlameWave => "Flame Wave",
            Self::DoomSerpents => "Doom Serpents",
            Self::BloodRitual => "Blood Ritual",
            Self::Nova => "Nova",
            Self::Invisibility => "Invisibility",
            Self::Inferno => "Inferno",
            Self::Golem => "Golem",
            Self::Rage => "Rage",
            Self::Teleport => "Teleport",
            Self::Apocalypse => "Apocalypse",
            Self::Etherealize => "Etherealize",
            Self::ItemRepair => "Item Repair",
            Self::StaffRecharge => "Staff Recharge",
            Self::TrapDisarm => "Trap Disarm",
            Self::Elemental => "Elemental",
            Self::ChargedBolt => "Charged Bolt",
            Self::HolyBolt => "Holy Bolt",
            Self::Resurrect => "Resurrect",
            Self::Telekinesis => "Telekinesis",
            Self::HealOther => "Heal Other",
            Self::BloodStar => "Blood Star",
            Self::BoneSpirit => "Bone Spirit",
            Self::Mana => "Mana",
            Self::Magi => "Magi",
            Self::Jester => "Jester",
            Self::LightningWall => "Lightning Wall",
            Self::Immolation => "Immolation",
            Self::Warp => "Warp",
            Self::Reflect => "Reflect",
            Self::Berserk => "Berserk",
            Self::RingOfFire => "Ring of Fire",
            Self::Search => "Search",
            Self::RuneOfFire => "Rune of Fire",
            Self::RuneOfLight => "Rune of Light",
            Self::RuneOfNova => "Rune of Nova",
            Self::RuneOfImmolation => "Rune of Immolation",
            Self::RuneOfStone => "Rune of Stone",
            Self::Invalid => "Invalid",
        }
    }

    /// Check if spell is valid
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Null | Self::Invalid)
    }

    /// Check if this is a Hellfire spell
    pub fn is_hellfire(&self) -> bool {
        matches!(
            self,
            Self::LightningWall
                | Self::Immolation
                | Self::Warp
                | Self::Reflect
                | Self::Berserk
                | Self::RingOfFire
                | Self::Search
                | Self::RuneOfFire
                | Self::RuneOfLight
                | Self::RuneOfNova
                | Self::RuneOfImmolation
                | Self::RuneOfStone
        )
    }

    /// Check if this is a rune skill
    pub fn is_rune(&self) -> bool {
        matches!(
            self,
            Self::RuneOfFire
                | Self::RuneOfLight
                | Self::RuneOfNova
                | Self::RuneOfImmolation
                | Self::RuneOfStone
        )
    }

    /// Check if spell can be used in town
    pub fn is_allowed_in_town(&self) -> bool {
        matches!(
            self,
            Self::Healing
                | Self::Identify
                | Self::TownPortal
                | Self::Infravision
                | Self::Phasing
                | Self::ManaShield
                | Self::ItemRepair
                | Self::StaffRecharge
                | Self::Telekinesis
                | Self::HealOther
                | Self::Search
                | Self::Warp
        )
    }
}

/// Spell type - how the spell is sourced
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellType {
    /// Invalid/no spell
    Invalid = 0,
    /// Natural skill (class ability)
    Skill = 1,
    /// Learned from book
    Spell = 2,
    /// Cast from scroll
    Scroll = 3,
    /// Staff charges
    Charges = 4,
}

impl SpellType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Invalid => "Invalid",
            Self::Skill => "Skill",
            Self::Spell => "Spell",
            Self::Scroll => "Scroll",
            Self::Charges => "Staff",
        }
    }
}

/// Number of spell book pages
pub const SPELL_BOOK_PAGES: usize = 6;

/// Number of entries per spell book page
pub const SPELL_BOOK_PAGE_ENTRIES: usize = 7;

/// Spell book page layout - matches C++ SpellPages array
pub const SPELL_PAGES: [[SpellId; SPELL_BOOK_PAGE_ENTRIES]; SPELL_BOOK_PAGES] = [
    // Page 0
    [
        SpellId::Null,
        SpellId::Firebolt,
        SpellId::ChargedBolt,
        SpellId::HolyBolt,
        SpellId::Healing,
        SpellId::HealOther,
        SpellId::Inferno,
    ],
    // Page 1
    [
        SpellId::Resurrect,
        SpellId::FireWall,
        SpellId::Telekinesis,
        SpellId::Lightning,
        SpellId::TownPortal,
        SpellId::Flash,
        SpellId::StoneCurse,
    ],
    // Page 2
    [
        SpellId::Phasing,
        SpellId::ManaShield,
        SpellId::Elemental,
        SpellId::Fireball,
        SpellId::FlameWave,
        SpellId::ChainLightning,
        SpellId::Guardian,
    ],
    // Page 3
    [
        SpellId::Nova,
        SpellId::Golem,
        SpellId::Teleport,
        SpellId::Apocalypse,
        SpellId::BoneSpirit,
        SpellId::BloodStar,
        SpellId::Etherealize,
    ],
    // Page 4 (Hellfire)
    [
        SpellId::LightningWall,
        SpellId::Immolation,
        SpellId::Warp,
        SpellId::Reflect,
        SpellId::Berserk,
        SpellId::RingOfFire,
        SpellId::Search,
    ],
    // Page 5 (empty in original)
    [
        SpellId::Invalid,
        SpellId::Invalid,
        SpellId::Invalid,
        SpellId::Invalid,
        SpellId::Invalid,
        SpellId::Invalid,
        SpellId::Invalid,
    ],
];

/// Spell book dimensions
pub const SPELL_BOOK_WIDTH: i32 = 320;
pub const SPELL_BOOK_HEIGHT: i32 = 352;

/// Spell book description area
pub const SPELL_BOOK_DESCRIPTION_WIDTH: i32 = 250;
pub const SPELL_BOOK_DESCRIPTION_HEIGHT: i32 = 43;

/// Spell icon size
pub const SPELL_ICON_SIZE: i32 = 37;

/// Spell book button widths
pub const SPELL_BOOK_BUTTON_WIDTH_DIABLO: u16 = 76;
pub const SPELL_BOOK_BUTTON_WIDTH_HELLFIRE: u16 = 61;

/// Spell entry in spell book
#[derive(Debug, Clone)]
pub struct SpellBookEntry {
    /// Spell ID
    pub spell_id: SpellId,
    /// How the spell is available (skill, spell, scroll, staff)
    pub spell_type: SpellType,
    /// Spell level (0 = not learned)
    pub level: i32,
    /// Mana cost
    pub mana_cost: i32,
    /// Is spell selected?
    pub selected: bool,
}

impl SpellBookEntry {
    pub fn new(spell_id: SpellId) -> Self {
        Self {
            spell_id,
            spell_type: SpellType::Invalid,
            level: 0,
            mana_cost: 0,
            selected: false,
        }
    }

    /// Set as learned spell
    pub fn set_learned(&mut self, level: i32, mana_cost: i32) {
        self.spell_type = SpellType::Spell;
        self.level = level;
        self.mana_cost = mana_cost;
    }

    /// Set as skill
    pub fn set_skill(&mut self) {
        self.spell_type = SpellType::Skill;
        self.level = 1;
        self.mana_cost = 0;
    }

    /// Set as staff charges
    pub fn set_charges(&mut self, charges: i32) {
        self.spell_type = SpellType::Charges;
        self.level = charges;
        self.mana_cost = 0;
    }

    /// Is spell usable?
    pub fn is_usable(&self) -> bool {
        self.spell_type != SpellType::Invalid && self.level > 0
    }

    /// Get display style for spell type
    pub fn type_style(&self) -> UiFlags {
        match self.spell_type {
            SpellType::Skill => UiFlags::COLOR_WHITEGOLD,
            SpellType::Spell => UiFlags::COLOR_WHITE,
            SpellType::Scroll => UiFlags::COLOR_WHITE,
            SpellType::Charges => UiFlags::COLOR_BLUE,
            SpellType::Invalid => UiFlags::COLOR_RED,
        }
    }

    /// Format spell info text
    pub fn format_info(&self) -> String {
        match self.spell_type {
            SpellType::Skill => "Skill".to_string(),
            SpellType::Charges => format!("Staff ({} charges)", self.level),
            SpellType::Spell | SpellType::Scroll => {
                format!("Level {} - Mana: {}", self.level, self.mana_cost)
            }
            SpellType::Invalid => "Unusable".to_string(),
        }
    }
}

/// Spell book panel
#[derive(Debug, Clone)]
pub struct SpellBook {
    /// Panel position
    pub position: Point,
    /// Panel size
    pub size: Size,
    /// Is panel visible?
    pub visible: bool,
    /// Current page (0-5)
    pub current_page: usize,
    /// Spell entries for current page
    pub entries: [SpellBookEntry; SPELL_BOOK_PAGE_ENTRIES],
    /// Is Hellfire mode?
    pub is_hellfire: bool,
    /// Currently selected spell
    pub selected_spell: Option<SpellId>,
    /// Spell icon rectangles (for hit testing)
    pub icon_rects: [Rectangle; SPELL_BOOK_PAGE_ENTRIES],
    /// Page button rectangles
    pub page_buttons: [Rectangle; SPELL_BOOK_PAGES],
}

impl SpellBook {
    /// Create a new spell book
    pub fn new() -> Self {
        let mut book = Self {
            position: Point::new(0, 0),
            size: Size::new(SPELL_BOOK_WIDTH, SPELL_BOOK_HEIGHT),
            visible: false,
            current_page: 0,
            entries: std::array::from_fn(|i| {
                SpellBookEntry::new(SPELL_PAGES[0][i])
            }),
            is_hellfire: false,
            selected_spell: None,
            icon_rects: std::array::from_fn(|i| {
                Rectangle::new(11, 18 + (i as i32 * SPELL_BOOK_DESCRIPTION_HEIGHT), SPELL_ICON_SIZE, SPELL_ICON_SIZE)
            }),
            page_buttons: std::array::from_fn(|i| {
                Rectangle::new(7 + (i as i32 * 76), 348, 76, 18)
            }),
        };
        book.refresh_entries();
        book
    }

    /// Set Hellfire mode
    pub fn set_hellfire(&mut self, hellfire: bool) {
        self.is_hellfire = hellfire;
    }

    /// Show the spell book
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the spell book
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set current page
    pub fn set_page(&mut self, page: usize) {
        if page < SPELL_BOOK_PAGES {
            self.current_page = page;
            self.refresh_entries();
        }
    }

    /// Go to next page
    pub fn next_page(&mut self) {
        let max_page = if self.is_hellfire { 5 } else { 4 };
        if self.current_page < max_page {
            self.current_page += 1;
            self.refresh_entries();
        }
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.refresh_entries();
        }
    }

    /// Refresh spell entries for current page
    fn refresh_entries(&mut self) {
        for (i, entry) in self.entries.iter_mut().enumerate() {
            entry.spell_id = SPELL_PAGES[self.current_page][i];
        }
    }

    /// Get spell at position on current page
    pub fn get_spell(&self, entry: usize) -> Option<SpellId> {
        if entry < SPELL_BOOK_PAGE_ENTRIES {
            let spell = self.entries[entry].spell_id;
            if spell.is_valid() {
                return Some(spell);
            }
        }
        None
    }

    /// Select a spell
    pub fn select_spell(&mut self, spell_id: SpellId) {
        self.selected_spell = Some(spell_id);
        for entry in &mut self.entries {
            entry.selected = entry.spell_id == spell_id;
        }
    }

    /// Deselect all spells
    pub fn deselect_all(&mut self) {
        self.selected_spell = None;
        for entry in &mut self.entries {
            entry.selected = false;
        }
    }

    /// Hit test spell icon
    pub fn hit_test_icon(&self, x: i32, y: i32) -> Option<usize> {
        if !self.visible {
            return None;
        }

        let local_x = x - self.position.x;
        let local_y = y - self.position.y;
        let point = Point::new(local_x, local_y);

        for (i, rect) in self.icon_rects.iter().enumerate() {
            if rect.contains(point) && self.entries[i].spell_id.is_valid() {
                return Some(i);
            }
        }

        None
    }

    /// Hit test page button
    pub fn hit_test_page_button(&self, x: i32, y: i32) -> Option<usize> {
        if !self.visible {
            return None;
        }

        let local_x = x - self.position.x;
        let local_y = y - self.position.y;
        let point = Point::new(local_x, local_y);

        for (i, rect) in self.page_buttons.iter().enumerate() {
            if rect.contains(point) {
                let max_page = if self.is_hellfire { 5 } else { 4 };
                if i <= max_page {
                    return Some(i);
                }
            }
        }

        None
    }

    /// Get button width based on game mode
    pub fn button_width(&self) -> u16 {
        if self.is_hellfire {
            SPELL_BOOK_BUTTON_WIDTH_HELLFIRE
        } else {
            SPELL_BOOK_BUTTON_WIDTH_DIABLO
        }
    }

    /// Update spell entry with player data
    pub fn update_entry(&mut self, entry_idx: usize, spell_type: SpellType, level: i32, mana_cost: i32) {
        if entry_idx < SPELL_BOOK_PAGE_ENTRIES {
            self.entries[entry_idx].spell_type = spell_type;
            self.entries[entry_idx].level = level;
            self.entries[entry_idx].mana_cost = mana_cost;
        }
    }
}

impl Default for SpellBook {
    fn default() -> Self {
        Self::new()
    }
}

/// Get spell damage amount (min, max)
/// Returns (-1, -1) if spell has no damage
pub fn get_spell_damage(spell: SpellId, level: i32) -> (i32, i32) {
    if level <= 0 {
        return (-1, -1);
    }

    match spell {
        SpellId::Firebolt => {
            let min = (level + 1) / 2 + 1;
            let max = (level + 1) / 2 + 10;
            (min, max)
        }
        SpellId::ChargedBolt => {
            let min = 1;
            let max = level + 1;
            (min, max)
        }
        SpellId::HolyBolt => {
            let min = level + 1;
            let max = level + 10;
            (min, max)
        }
        SpellId::Lightning => {
            let min = 2;
            let max = level + 6;
            (min, max)
        }
        SpellId::Flash => {
            let base = 10 + (level - 1) * 5;
            (base / 2, base)
        }
        SpellId::FireWall => {
            let base = (8 + level) * 2;
            (base, base * 2)
        }
        SpellId::Fireball => {
            let base = 10 + level * 2;
            (base, base + 10)
        }
        SpellId::ChainLightning => {
            let min = 4;
            let max = level * 2 + 8;
            (min, max)
        }
        SpellId::FlameWave => {
            let base = 8 + level * 2;
            (base, base * 2)
        }
        SpellId::Nova => {
            let base = level * 5 + 10;
            (base / 2, base)
        }
        SpellId::Apocalypse => {
            let base = level * 6;
            (base, base * 6)
        }
        SpellId::Inferno => {
            let base = level * 3;
            (base, base + 3)
        }
        SpellId::BloodStar => {
            let base = level + 5;
            (base, base * 2)
        }
        SpellId::BoneSpirit => {
            // Bone Spirit does 1/3 of target HP
            (-1, -1)
        }
        SpellId::Healing | SpellId::HealOther => {
            let base = level * 2 + 2;
            (base, base + level * 2)
        }
        _ => (-1, -1),
    }
}

/// Format spell power/damage text
pub fn format_spell_power(spell: SpellId, level: i32) -> String {
    if level == 0 {
        return "Unusable".to_string();
    }

    if spell == SpellId::BoneSpirit {
        return "Dmg: 1/3 target hp".to_string();
    }

    let (min, max) = get_spell_damage(spell, level);
    if min == -1 {
        return String::new();
    }

    if matches!(spell, SpellId::Healing | SpellId::HealOther) {
        format!("Heals: {} - {}", min, max)
    } else {
        format!("Damage: {} - {}", min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_id_names() {
        assert_eq!(SpellId::Firebolt.name(), "Firebolt");
        assert_eq!(SpellId::TownPortal.name(), "Town Portal");
        assert_eq!(SpellId::Invalid.name(), "Invalid");
    }

    #[test]
    fn test_spell_id_valid() {
        assert!(SpellId::Firebolt.is_valid());
        assert!(!SpellId::Null.is_valid());
        assert!(!SpellId::Invalid.is_valid());
    }

    #[test]
    fn test_spell_id_hellfire() {
        assert!(!SpellId::Firebolt.is_hellfire());
        assert!(SpellId::LightningWall.is_hellfire());
        assert!(SpellId::RuneOfFire.is_hellfire());
    }

    #[test]
    fn test_spell_id_town() {
        assert!(SpellId::Healing.is_allowed_in_town());
        assert!(SpellId::TownPortal.is_allowed_in_town());
        assert!(!SpellId::Firebolt.is_allowed_in_town());
        assert!(!SpellId::Lightning.is_allowed_in_town());
    }

    #[test]
    fn test_spell_book_entry() {
        let mut entry = SpellBookEntry::new(SpellId::Firebolt);
        assert!(!entry.is_usable());

        entry.set_learned(5, 8);
        assert!(entry.is_usable());
        assert_eq!(entry.level, 5);
        assert_eq!(entry.mana_cost, 8);
    }

    #[test]
    fn test_spell_book_pages() {
        let book = SpellBook::new();
        assert_eq!(book.current_page, 0);
        assert_eq!(book.entries[1].spell_id, SpellId::Firebolt);
    }

    #[test]
    fn test_spell_book_navigation() {
        let mut book = SpellBook::new();

        book.next_page();
        assert_eq!(book.current_page, 1);
        assert_eq!(book.entries[0].spell_id, SpellId::Resurrect);

        book.prev_page();
        assert_eq!(book.current_page, 0);

        book.prev_page(); // Should stay at 0
        assert_eq!(book.current_page, 0);
    }

    #[test]
    fn test_spell_book_hellfire() {
        let mut book = SpellBook::new();
        book.set_hellfire(true);

        book.set_page(4);
        assert_eq!(book.entries[0].spell_id, SpellId::LightningWall);
    }

    #[test]
    fn test_spell_damage() {
        let (min, max) = get_spell_damage(SpellId::Firebolt, 1);
        assert!(min > 0);
        assert!(max > min);

        let (min, max) = get_spell_damage(SpellId::Firebolt, 0);
        assert_eq!(min, -1);
        assert_eq!(max, -1);
    }

    #[test]
    fn test_format_spell_power() {
        let text = format_spell_power(SpellId::Firebolt, 5);
        assert!(text.starts_with("Damage:"));

        let text = format_spell_power(SpellId::Healing, 3);
        assert!(text.starts_with("Heals:"));

        let text = format_spell_power(SpellId::BoneSpirit, 1);
        assert_eq!(text, "Dmg: 1/3 target hp");

        let text = format_spell_power(SpellId::Firebolt, 0);
        assert_eq!(text, "Unusable");
    }
}
