//! Spell Icons - Spell Icon Rendering (M64)
//!
//! Provides spell icon rendering functionality:
//! - Large and small spell icons
//! - Color translation for spell types
//! - Icon borders and selection highlighting
//!
//! ## C++ References
//! - Source/panels/spell_icons.cpp
//! - Source/panels/spell_icons.hpp

#![allow(dead_code)]
#![allow(unused_imports)]

use super::spell_book::{SpellId, SpellType};

/// Spell icon size (large icons)
pub const SPELL_ICON_LENGTH: i32 = 56;

/// Small spell icon size
pub const SMALL_SPELL_ICON_LENGTH: i32 = 37;

/// Spell icon frame indices - matches C++ SpellIcon enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SpellIcon {
    Empty = 0,
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
    DoomSerpents = 13,
    ChainLightning = 14,
    FlameWave = 15,
    BloodRitual = 16,
    Nova = 17,
    Invisibility = 18,
    Inferno = 19,
    Golem = 20,
    BloodBoil = 21,
    Teleport = 22,
    Apocalypse = 23,
    Etherealize = 24,
    ItemRepair = 25,
    StaffRecharge = 26,
    TrapDisarm = 27,
    Elemental = 28,
    ChargedBolt = 29,
    HolyBolt = 30,
    Resurrect = 31,
    Telekinesis = 32,
    HealOther = 33,
    BloodStar = 34,
    BoneSpirit = 35,
    Mana = 36,
    Jester = 37,
    LightningWall = 38,
    Immolation = 39,
    Warp = 40,
    Reflect = 41,
    Berserk = 42,
    RingOfFire = 43,
    Search = 44,
    PentaStar = 45, // Used for runes
}

/// Mapping from SpellID to SpellIcon frame
pub fn get_spell_icon_frame(spell: SpellId) -> SpellIcon {
    match spell {
        SpellId::Null => SpellIcon::Empty,
        SpellId::Firebolt => SpellIcon::Firebolt,
        SpellId::Healing => SpellIcon::Healing,
        SpellId::Lightning => SpellIcon::Lightning,
        SpellId::Flash => SpellIcon::Flash,
        SpellId::Identify => SpellIcon::Identify,
        SpellId::FireWall => SpellIcon::FireWall,
        SpellId::TownPortal => SpellIcon::TownPortal,
        SpellId::StoneCurse => SpellIcon::StoneCurse,
        SpellId::Infravision => SpellIcon::Infravision,
        SpellId::Phasing => SpellIcon::Phasing,
        SpellId::ManaShield => SpellIcon::ManaShield,
        SpellId::Fireball => SpellIcon::Fireball,
        SpellId::Guardian => SpellIcon::DoomSerpents,
        SpellId::ChainLightning => SpellIcon::ChainLightning,
        SpellId::FlameWave => SpellIcon::FlameWave,
        SpellId::DoomSerpents => SpellIcon::DoomSerpents,
        SpellId::BloodRitual => SpellIcon::BloodRitual,
        SpellId::Nova => SpellIcon::Nova,
        SpellId::Invisibility => SpellIcon::Invisibility,
        SpellId::Inferno => SpellIcon::Inferno,
        SpellId::Golem => SpellIcon::Golem,
        SpellId::Rage => SpellIcon::BloodBoil,
        SpellId::Teleport => SpellIcon::Teleport,
        SpellId::Apocalypse => SpellIcon::Apocalypse,
        SpellId::Etherealize => SpellIcon::Etherealize,
        SpellId::ItemRepair => SpellIcon::ItemRepair,
        SpellId::StaffRecharge => SpellIcon::StaffRecharge,
        SpellId::TrapDisarm => SpellIcon::TrapDisarm,
        SpellId::Elemental => SpellIcon::Elemental,
        SpellId::ChargedBolt => SpellIcon::ChargedBolt,
        SpellId::HolyBolt => SpellIcon::HolyBolt,
        SpellId::Resurrect => SpellIcon::Resurrect,
        SpellId::Telekinesis => SpellIcon::Telekinesis,
        SpellId::HealOther => SpellIcon::HealOther,
        SpellId::BloodStar => SpellIcon::BloodStar,
        SpellId::BoneSpirit => SpellIcon::BoneSpirit,
        SpellId::Mana | SpellId::Magi => SpellIcon::Mana,
        SpellId::Jester => SpellIcon::Jester,
        SpellId::LightningWall => SpellIcon::LightningWall,
        SpellId::Immolation => SpellIcon::Immolation,
        SpellId::Warp => SpellIcon::Warp,
        SpellId::Reflect => SpellIcon::Reflect,
        SpellId::Berserk => SpellIcon::Berserk,
        SpellId::RingOfFire => SpellIcon::RingOfFire,
        SpellId::Search => SpellIcon::Search,
        SpellId::RuneOfFire
        | SpellId::RuneOfLight
        | SpellId::RuneOfNova
        | SpellId::RuneOfImmolation
        | SpellId::RuneOfStone => SpellIcon::PentaStar,
        SpellId::Invalid => SpellIcon::Empty,
    }
}

/// Palette color indices (from palette.h)
pub const PAL8_YELLOW: u8 = 216;
pub const PAL16_BLUE: u8 = 48;
pub const PAL16_BEIGE: u8 = 64;
pub const PAL16_YELLOW: u8 = 80;
pub const PAL16_ORANGE: u8 = 96;
pub const PAL16_RED: u8 = 112;
pub const PAL16_GRAY: u8 = 128;

/// Spell translation table for color effects
#[derive(Debug, Clone)]
pub struct SpellTranslationTable {
    /// 256-entry palette translation table
    pub table: [u8; 256],
}

impl SpellTranslationTable {
    /// Create a new identity translation table
    pub fn new() -> Self {
        let mut table = [0u8; 256];
        for i in 0..256 {
            table[i] = i as u8;
        }
        table[255] = 0; // Transparency
        Self { table }
    }

    /// Set translation for spell type
    pub fn set_spell_type(&mut self, spell_type: SpellType) {
        // Reset first 128 entries to identity for Skill type
        if spell_type == SpellType::Skill {
            for i in 0..128 {
                self.table[i] = i as u8;
            }
        }

        // Keep 128-255 as identity
        for i in 128..256 {
            self.table[i] = i as u8;
        }
        self.table[255] = 0;

        // Apply type-specific color shifts
        match spell_type {
            SpellType::Spell => {
                // Yellow -> Blue
                self.table[PAL8_YELLOW as usize] = PAL16_BLUE + 1;
                self.table[PAL8_YELLOW as usize + 1] = PAL16_BLUE + 3;
                self.table[PAL8_YELLOW as usize + 2] = PAL16_BLUE + 5;

                // Shift color palettes to blue
                for i in 0..16 {
                    self.table[(PAL16_BEIGE + i) as usize] = PAL16_BLUE + i;
                    self.table[(PAL16_YELLOW + i) as usize] = PAL16_BLUE + i;
                    self.table[(PAL16_ORANGE + i) as usize] = PAL16_BLUE + i;
                }
            }
            SpellType::Scroll => {
                // Yellow -> Beige
                self.table[PAL8_YELLOW as usize] = PAL16_BEIGE + 1;
                self.table[PAL8_YELLOW as usize + 1] = PAL16_BEIGE + 3;
                self.table[PAL8_YELLOW as usize + 2] = PAL16_BEIGE + 5;

                // Shift color palettes to beige
                for i in 0..16 {
                    self.table[(PAL16_YELLOW + i) as usize] = PAL16_BEIGE + i;
                    self.table[(PAL16_ORANGE + i) as usize] = PAL16_BEIGE + i;
                }
            }
            SpellType::Charges => {
                // Yellow -> Orange
                self.table[PAL8_YELLOW as usize] = PAL16_ORANGE + 1;
                self.table[PAL8_YELLOW as usize + 1] = PAL16_ORANGE + 3;
                self.table[PAL8_YELLOW as usize + 2] = PAL16_ORANGE + 5;

                // Shift yellow to orange
                for i in 0..16 {
                    self.table[(PAL16_YELLOW + i) as usize] = PAL16_ORANGE + i;
                }
            }
            SpellType::Invalid => {
                // Yellow -> Red
                self.table[PAL8_YELLOW as usize] = PAL16_RED + 1;
                self.table[PAL8_YELLOW as usize + 1] = PAL16_RED + 3;
                self.table[PAL8_YELLOW as usize + 2] = PAL16_RED + 5;

                // Shift warm colors to red
                for i in 0..16 {
                    self.table[(PAL16_YELLOW + i) as usize] = PAL16_RED + i;
                    self.table[(PAL16_ORANGE + i) as usize] = PAL16_RED + i;
                    self.table[(PAL16_BEIGE + i) as usize] = PAL16_RED + i;
                }
            }
            SpellType::Skill => {
                // No changes for skill - use original colors
            }
        }
    }

    /// Translate a palette index
    pub fn translate(&self, index: u8) -> u8 {
        self.table[index as usize]
    }
}

impl Default for SpellTranslationTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Spell icon state for rendering
#[derive(Debug, Clone)]
pub struct SpellIconState {
    /// Current spell
    pub spell: SpellId,
    /// Spell type (determines color)
    pub spell_type: SpellType,
    /// Is icon selected?
    pub selected: bool,
    /// Is icon hovered?
    pub hovered: bool,
    /// Translation table for color
    pub translation: SpellTranslationTable,
}

impl SpellIconState {
    pub fn new(spell: SpellId, spell_type: SpellType) -> Self {
        let mut state = Self {
            spell,
            spell_type,
            selected: false,
            hovered: false,
            translation: SpellTranslationTable::new(),
        };
        state.translation.set_spell_type(spell_type);
        state
    }

    /// Update spell type (also updates translation)
    pub fn set_spell_type(&mut self, spell_type: SpellType) {
        self.spell_type = spell_type;
        self.translation.set_spell_type(spell_type);
    }

    /// Get icon frame for this spell
    pub fn icon_frame(&self) -> u8 {
        get_spell_icon_frame(self.spell) as u8
    }

    /// Get border color for selection
    pub fn border_color(&self) -> u8 {
        if self.selected {
            self.translation.translate(PAL8_YELLOW + 2)
        } else {
            0
        }
    }
}

/// Spell icon manager - handles loading and drawing spell icons
#[derive(Debug, Clone)]
pub struct SpellIconManager {
    /// Current translation table
    pub current_translation: SpellTranslationTable,
    /// Are large icons loaded?
    pub large_icons_loaded: bool,
    /// Are small icons loaded?
    pub small_icons_loaded: bool,
}

impl SpellIconManager {
    pub fn new() -> Self {
        Self {
            current_translation: SpellTranslationTable::new(),
            large_icons_loaded: false,
            small_icons_loaded: false,
        }
    }

    /// Set translation for spell type
    pub fn set_spell_trans(&mut self, spell_type: SpellType) {
        self.current_translation.set_spell_type(spell_type);
    }

    /// Mark large icons as loaded
    pub fn load_large_icons(&mut self) {
        self.large_icons_loaded = true;
        self.set_spell_trans(SpellType::Skill);
    }

    /// Mark large icons as freed
    pub fn free_large_icons(&mut self) {
        self.large_icons_loaded = false;
    }

    /// Mark small icons as loaded
    pub fn load_small_icons(&mut self) {
        self.small_icons_loaded = true;
    }

    /// Mark small icons as freed
    pub fn free_small_icons(&mut self) {
        self.small_icons_loaded = false;
    }
}

impl Default for SpellIconManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick spell slots (F-keys)
#[derive(Debug, Clone, Copy)]
pub struct QuickSpellSlot {
    /// Spell in this slot
    pub spell: SpellId,
    /// Spell type
    pub spell_type: SpellType,
    /// Hotkey (F5-F8)
    pub hotkey: u8,
}

impl QuickSpellSlot {
    pub fn new(hotkey: u8) -> Self {
        Self {
            spell: SpellId::Invalid,
            spell_type: SpellType::Invalid,
            hotkey,
        }
    }

    pub fn set(&mut self, spell: SpellId, spell_type: SpellType) {
        self.spell = spell;
        self.spell_type = spell_type;
    }

    pub fn clear(&mut self) {
        self.spell = SpellId::Invalid;
        self.spell_type = SpellType::Invalid;
    }

    pub fn is_set(&self) -> bool {
        self.spell.is_valid()
    }
}

/// Quick spell bar (4 slots for F5-F8)
#[derive(Debug, Clone)]
pub struct QuickSpellBar {
    pub slots: [QuickSpellSlot; 4],
}

impl QuickSpellBar {
    pub fn new() -> Self {
        Self {
            slots: [
                QuickSpellSlot::new(5), // F5
                QuickSpellSlot::new(6), // F6
                QuickSpellSlot::new(7), // F7
                QuickSpellSlot::new(8), // F8
            ],
        }
    }

    /// Set spell for slot
    pub fn set_slot(&mut self, slot: usize, spell: SpellId, spell_type: SpellType) {
        if slot < 4 {
            self.slots[slot].set(spell, spell_type);
        }
    }

    /// Get spell from slot
    pub fn get_slot(&self, slot: usize) -> Option<(SpellId, SpellType)> {
        if slot < 4 && self.slots[slot].is_set() {
            Some((self.slots[slot].spell, self.slots[slot].spell_type))
        } else {
            None
        }
    }

    /// Clear a slot
    pub fn clear_slot(&mut self, slot: usize) {
        if slot < 4 {
            self.slots[slot].clear();
        }
    }

    /// Find slot containing spell
    pub fn find_spell(&self, spell: SpellId) -> Option<usize> {
        self.slots.iter().position(|s| s.spell == spell)
    }
}

impl Default for QuickSpellBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_icon_mapping() {
        assert_eq!(get_spell_icon_frame(SpellId::Firebolt), SpellIcon::Firebolt);
        assert_eq!(get_spell_icon_frame(SpellId::Null), SpellIcon::Empty);
        assert_eq!(get_spell_icon_frame(SpellId::RuneOfFire), SpellIcon::PentaStar);
    }

    #[test]
    fn test_translation_table_identity() {
        let table = SpellTranslationTable::new();
        for i in 0..255 {
            assert_eq!(table.translate(i as u8), i as u8);
        }
        assert_eq!(table.translate(255), 0);
    }

    #[test]
    fn test_translation_table_spell_type() {
        let mut table = SpellTranslationTable::new();

        table.set_spell_type(SpellType::Spell);
        // Yellow should shift to blue
        assert_eq!(table.translate(PAL8_YELLOW), PAL16_BLUE + 1);

        table.set_spell_type(SpellType::Scroll);
        // Yellow should shift to beige
        assert_eq!(table.translate(PAL8_YELLOW), PAL16_BEIGE + 1);

        table.set_spell_type(SpellType::Charges);
        // Yellow should shift to orange
        assert_eq!(table.translate(PAL8_YELLOW), PAL16_ORANGE + 1);
    }

    #[test]
    fn test_spell_icon_state() {
        let state = SpellIconState::new(SpellId::Firebolt, SpellType::Spell);
        assert_eq!(state.icon_frame(), SpellIcon::Firebolt as u8);
        assert_eq!(state.spell_type, SpellType::Spell);
    }

    #[test]
    fn test_quick_spell_slot() {
        let mut slot = QuickSpellSlot::new(5);
        assert!(!slot.is_set());

        slot.set(SpellId::Firebolt, SpellType::Spell);
        assert!(slot.is_set());
        assert_eq!(slot.spell, SpellId::Firebolt);

        slot.clear();
        assert!(!slot.is_set());
    }

    #[test]
    fn test_quick_spell_bar() {
        let mut bar = QuickSpellBar::new();

        bar.set_slot(0, SpellId::Firebolt, SpellType::Spell);
        bar.set_slot(2, SpellId::Healing, SpellType::Skill);

        assert_eq!(bar.get_slot(0), Some((SpellId::Firebolt, SpellType::Spell)));
        assert_eq!(bar.get_slot(1), None);
        assert_eq!(bar.get_slot(2), Some((SpellId::Healing, SpellType::Skill)));

        assert_eq!(bar.find_spell(SpellId::Healing), Some(2));
        assert_eq!(bar.find_spell(SpellId::Lightning), None);
    }
}
