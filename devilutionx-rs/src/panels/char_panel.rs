//! Character Panel - Player Stats Display (M64)
//!
//! Provides the character information panel displaying:
//! - Player name and class
//! - Level and experience
//! - Base and current attributes (STR, MAG, DEX, VIT)
//! - Life and mana
//! - Armor class, to-hit, damage
//! - Resistances
//! - Gold
//!
//! ## C++ References
//! - Source/panels/charpanel.cpp
//! - Source/panels/charpanel.hpp

#![allow(dead_code)]
#![allow(unused_imports)]

use super::main_panel::{Point, Size, Rectangle};

/// Character attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CharacterAttribute {
    Strength = 0,
    Magic = 1,
    Dexterity = 2,
    Vitality = 3,
}

impl CharacterAttribute {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Magic => "Magic",
            Self::Dexterity => "Dexterity",
            Self::Vitality => "Vitality",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Strength => "STR",
            Self::Magic => "MAG",
            Self::Dexterity => "DEX",
            Self::Vitality => "VIT",
        }
    }

    /// All attributes
    pub const ALL: [Self; 4] = [
        Self::Strength,
        Self::Magic,
        Self::Dexterity,
        Self::Vitality,
    ];
}

/// Hero class types - matches C++ HeroClass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeroClass {
    Warrior = 0,
    Rogue = 1,
    Sorcerer = 2,
    Monk = 3,
    Bard = 4,
    Barbarian = 5,
}

impl HeroClass {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Warrior => "Warrior",
            Self::Rogue => "Rogue",
            Self::Sorcerer => "Sorcerer",
            Self::Monk => "Monk",
            Self::Bard => "Bard",
            Self::Barbarian => "Barbarian",
        }
    }

    /// Get class-specific stat color priority
    pub fn primary_stat(&self) -> CharacterAttribute {
        match self {
            Self::Warrior | Self::Barbarian => CharacterAttribute::Strength,
            Self::Rogue | Self::Bard => CharacterAttribute::Dexterity,
            Self::Sorcerer => CharacterAttribute::Magic,
            Self::Monk => CharacterAttribute::Dexterity,
        }
    }
}

/// UI text style flags - matches C++ UiFlags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFlags(u32);

impl UiFlags {
    pub const NONE: UiFlags = UiFlags(0);
    pub const COLOR_WHITE: UiFlags = UiFlags(1 << 0);
    pub const COLOR_BLUE: UiFlags = UiFlags(1 << 1);
    pub const COLOR_RED: UiFlags = UiFlags(1 << 2);
    pub const COLOR_WHITEGOLD: UiFlags = UiFlags(1 << 3);
    pub const ALIGN_CENTER: UiFlags = UiFlags(1 << 4);
    pub const ALIGN_RIGHT: UiFlags = UiFlags(1 << 5);
    pub const KERNING_FIT_SPACING: UiFlags = UiFlags(1 << 6);

    pub fn contains(&self, other: UiFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn combine(self, other: UiFlags) -> UiFlags {
        UiFlags(self.0 | other.0)
    }
}

/// Styled text for panel display
#[derive(Debug, Clone)]
pub struct StyledText {
    /// Text style/color
    pub style: UiFlags,
    /// Text content
    pub text: String,
    /// Character spacing
    pub spacing: i32,
}

impl StyledText {
    pub fn new(style: UiFlags, text: impl Into<String>) -> Self {
        Self {
            style,
            text: text.into(),
            spacing: 1,
        }
    }

    pub fn with_spacing(mut self, spacing: i32) -> Self {
        self.spacing = spacing;
        self
    }
}

/// Character panel entry definition
#[derive(Debug, Clone)]
pub struct PanelEntry {
    /// Label text
    pub label: String,
    /// Position on panel
    pub position: Point,
    /// Maximum value length
    pub length: i32,
    /// Maximum label length (for line wrapping)
    pub label_length: i32,
}

/// Character panel dimensions (matches C++ SidePanelSize)
pub const CHAR_PANEL_WIDTH: i32 = 320;
pub const CHAR_PANEL_HEIGHT: i32 = 352;

/// Panel column positions
pub const LEFT_COLUMN_LABEL_X: i32 = 88;
pub const TOP_RIGHT_LABEL_X: i32 = 211;
pub const RIGHT_COLUMN_LABEL_X: i32 = 253;

pub const LEFT_COLUMN_LABEL_WIDTH: i32 = 76;
pub const RIGHT_COLUMN_LABEL_WIDTH: i32 = 68;

/// Attribute display state
#[derive(Debug, Clone, Copy)]
pub struct AttributeDisplay {
    /// Base value (without equipment)
    pub base: i32,
    /// Current value (with equipment)
    pub current: i32,
    /// Maximum possible value
    pub maximum: i32,
}

impl AttributeDisplay {
    pub fn new(base: i32, current: i32, maximum: i32) -> Self {
        Self { base, current, maximum }
    }

    /// Get style for base stat display
    pub fn base_style(&self) -> UiFlags {
        if self.base >= self.maximum {
            UiFlags::COLOR_WHITEGOLD
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Get style for current stat display
    pub fn current_style(&self) -> UiFlags {
        if self.current > self.base {
            UiFlags::COLOR_BLUE
        } else if self.current < self.base {
            UiFlags::COLOR_RED
        } else {
            UiFlags::COLOR_WHITE
        }
    }
}

/// Resistance display
#[derive(Debug, Clone, Copy)]
pub struct ResistanceDisplay {
    /// Resistance value (-100 to 100)
    pub value: i8,
}

impl ResistanceDisplay {
    /// Maximum resistance value
    pub const MAX_RESISTANCE: i8 = 75;

    pub fn new(value: i8) -> Self {
        Self { value }
    }

    /// Get style for resistance display
    pub fn style(&self) -> UiFlags {
        if self.value >= Self::MAX_RESISTANCE {
            UiFlags::COLOR_WHITEGOLD
        } else if self.value > 0 {
            UiFlags::COLOR_BLUE
        } else if self.value < 0 {
            UiFlags::COLOR_RED
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Format resistance text
    pub fn format(&self) -> String {
        format!("{}%", self.value)
    }
}

/// Damage range display
#[derive(Debug, Clone, Copy)]
pub struct DamageDisplay {
    /// Minimum damage
    pub min: i32,
    /// Maximum damage
    pub max: i32,
    /// Bonus damage percentage
    pub bonus: i32,
}

impl DamageDisplay {
    pub fn new(min: i32, max: i32, bonus: i32) -> Self {
        Self { min, max, bonus }
    }

    /// Get style based on bonus
    pub fn style(&self) -> UiFlags {
        if self.bonus > 0 {
            UiFlags::COLOR_BLUE
        } else if self.bonus < 0 {
            UiFlags::COLOR_RED
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Format damage text
    pub fn format(&self) -> String {
        format!("{}-{}", self.min, self.max)
    }
}

/// Character panel state
#[derive(Debug, Clone)]
pub struct CharacterPanel {
    /// Panel position
    pub position: Point,
    /// Panel size
    pub size: Size,
    /// Is panel visible?
    pub visible: bool,
    /// Player name
    pub player_name: String,
    /// Player class
    pub player_class: HeroClass,
    /// Character level
    pub level: i32,
    /// Experience points
    pub experience: u32,
    /// Next level experience threshold
    pub next_level_exp: u32,
    /// Attributes (Str, Mag, Dex, Vit)
    pub attributes: [AttributeDisplay; 4],
    /// Points available to distribute
    pub stat_points: i32,
    /// Gold held
    pub gold: u32,
    /// Armor class
    pub armor_class: i32,
    /// Armor class bonus
    pub armor_bonus: i32,
    /// To-hit chance
    pub to_hit: i32,
    /// To-hit bonus
    pub to_hit_bonus: i32,
    /// Damage range
    pub damage: DamageDisplay,
    /// Current life
    pub life_current: i32,
    /// Maximum life
    pub life_max: i32,
    /// Base max life
    pub life_base_max: i32,
    /// Current mana
    pub mana_current: i32,
    /// Maximum mana
    pub mana_max: i32,
    /// Base max mana
    pub mana_base_max: i32,
    /// Magic resistance
    pub resist_magic: ResistanceDisplay,
    /// Fire resistance
    pub resist_fire: ResistanceDisplay,
    /// Lightning resistance
    pub resist_lightning: ResistanceDisplay,
    /// Is inspecting another player?
    pub inspecting_other: bool,
    /// Attribute button positions for stat allocation
    pub attribute_buttons: [Rectangle; 4],
}

impl CharacterPanel {
    /// Create a new character panel
    pub fn new() -> Self {
        Self {
            position: Point::new(0, 0),
            size: Size::new(CHAR_PANEL_WIDTH, CHAR_PANEL_HEIGHT),
            visible: false,
            player_name: String::new(),
            player_class: HeroClass::Warrior,
            level: 1,
            experience: 0,
            next_level_exp: 2000,
            attributes: [
                AttributeDisplay::new(10, 10, 250),
                AttributeDisplay::new(10, 10, 250),
                AttributeDisplay::new(10, 10, 250),
                AttributeDisplay::new(10, 10, 250),
            ],
            stat_points: 0,
            gold: 0,
            armor_class: 0,
            armor_bonus: 0,
            to_hit: 50,
            to_hit_bonus: 0,
            damage: DamageDisplay::new(1, 4, 0),
            life_current: 45,
            life_max: 45,
            life_base_max: 45,
            mana_current: 10,
            mana_max: 10,
            mana_base_max: 10,
            resist_magic: ResistanceDisplay::new(0),
            resist_fire: ResistanceDisplay::new(0),
            resist_lightning: ResistanceDisplay::new(0),
            inspecting_other: false,
            attribute_buttons: [
                Rectangle::new(137, 138, 41, 22),
                Rectangle::new(137, 166, 41, 22),
                Rectangle::new(137, 194, 41, 22),
                Rectangle::new(137, 222, 41, 22),
            ],
        }
    }

    /// Show the panel
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the panel
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set player info
    pub fn set_player_info(&mut self, name: &str, class: HeroClass, level: i32) {
        self.player_name = name.to_string();
        self.player_class = class;
        self.level = level;
    }

    /// Set experience
    pub fn set_experience(&mut self, current: u32, next_level: u32) {
        self.experience = current;
        self.next_level_exp = next_level;
    }

    /// Set attribute values
    pub fn set_attribute(&mut self, attr: CharacterAttribute, base: i32, current: i32, max: i32) {
        let idx = attr as usize;
        self.attributes[idx] = AttributeDisplay::new(base, current, max);
    }

    /// Set all resistances
    pub fn set_resistances(&mut self, magic: i8, fire: i8, lightning: i8) {
        self.resist_magic = ResistanceDisplay::new(magic);
        self.resist_fire = ResistanceDisplay::new(fire);
        self.resist_lightning = ResistanceDisplay::new(lightning);
    }

    /// Set life values
    pub fn set_life(&mut self, current: i32, max: i32, base_max: i32) {
        self.life_current = current;
        self.life_max = max;
        self.life_base_max = base_max;
    }

    /// Set mana values
    pub fn set_mana(&mut self, current: i32, max: i32, base_max: i32) {
        self.mana_current = current;
        self.mana_max = max;
        self.mana_base_max = base_max;
    }

    /// Get life style (color based on current vs max)
    pub fn life_style(&self) -> UiFlags {
        if self.life_current < self.life_max {
            UiFlags::COLOR_RED
        } else if self.life_max > self.life_base_max {
            UiFlags::COLOR_BLUE
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Get mana style (color based on current vs max)
    pub fn mana_style(&self) -> UiFlags {
        if self.mana_current < self.mana_max {
            UiFlags::COLOR_RED
        } else if self.mana_max > self.mana_base_max {
            UiFlags::COLOR_BLUE
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Get armor class style
    pub fn armor_style(&self) -> UiFlags {
        if self.armor_bonus > 0 {
            UiFlags::COLOR_BLUE
        } else if self.armor_bonus < 0 {
            UiFlags::COLOR_RED
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Get to-hit style
    pub fn to_hit_style(&self) -> UiFlags {
        if self.to_hit_bonus > 0 {
            UiFlags::COLOR_BLUE
        } else if self.to_hit_bonus < 0 {
            UiFlags::COLOR_RED
        } else {
            UiFlags::COLOR_WHITE
        }
    }

    /// Check if attribute button is hit
    pub fn hit_test_attribute_button(&self, x: i32, y: i32) -> Option<CharacterAttribute> {
        if !self.visible || self.stat_points == 0 || self.inspecting_other {
            return None;
        }

        let point = Point::new(x - self.position.x, y - self.position.y);

        for (i, rect) in self.attribute_buttons.iter().enumerate() {
            if rect.contains(point) {
                return CharacterAttribute::ALL.get(i).copied();
            }
        }

        None
    }

    /// Can allocate stat points?
    pub fn can_allocate_stat(&self, attr: CharacterAttribute) -> bool {
        if self.stat_points == 0 || self.inspecting_other {
            return false;
        }

        let idx = attr as usize;
        self.attributes[idx].base < self.attributes[idx].maximum
    }

    /// Get formatted experience text
    pub fn format_experience(&self) -> String {
        format_number(self.experience)
    }

    /// Get formatted next level text
    pub fn format_next_level(&self) -> String {
        if self.level >= 50 {
            "None".to_string()
        } else {
            format_number(self.next_level_exp)
        }
    }

    /// Get formatted gold text
    pub fn format_gold(&self) -> String {
        format_number(self.gold)
    }
}

impl Default for CharacterPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Format large numbers with commas
fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

/// Character panel button (for stat allocation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharPanelButton {
    /// Increase strength
    IncreaseStr,
    /// Increase magic
    IncreaseMag,
    /// Increase dexterity
    IncreaseDex,
    /// Increase vitality
    IncreaseVit,
}

impl CharPanelButton {
    pub fn attribute(&self) -> CharacterAttribute {
        match self {
            Self::IncreaseStr => CharacterAttribute::Strength,
            Self::IncreaseMag => CharacterAttribute::Magic,
            Self::IncreaseDex => CharacterAttribute::Dexterity,
            Self::IncreaseVit => CharacterAttribute::Vitality,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_attribute() {
        assert_eq!(CharacterAttribute::Strength.label(), "Strength");
        assert_eq!(CharacterAttribute::Magic.short_label(), "MAG");
    }

    #[test]
    fn test_hero_class() {
        assert_eq!(HeroClass::Warrior.name(), "Warrior");
        assert_eq!(HeroClass::Warrior.primary_stat(), CharacterAttribute::Strength);
        assert_eq!(HeroClass::Sorcerer.primary_stat(), CharacterAttribute::Magic);
    }

    #[test]
    fn test_attribute_display_styles() {
        let attr = AttributeDisplay::new(50, 50, 250);
        assert_eq!(attr.base_style(), UiFlags::COLOR_WHITE);
        assert_eq!(attr.current_style(), UiFlags::COLOR_WHITE);

        let boosted = AttributeDisplay::new(50, 75, 250);
        assert_eq!(boosted.current_style(), UiFlags::COLOR_BLUE);

        let debuffed = AttributeDisplay::new(50, 25, 250);
        assert_eq!(debuffed.current_style(), UiFlags::COLOR_RED);

        let maxed = AttributeDisplay::new(250, 250, 250);
        assert_eq!(maxed.base_style(), UiFlags::COLOR_WHITEGOLD);
    }

    #[test]
    fn test_resistance_display() {
        let neutral = ResistanceDisplay::new(0);
        assert_eq!(neutral.style(), UiFlags::COLOR_WHITE);
        assert_eq!(neutral.format(), "0%");

        let positive = ResistanceDisplay::new(50);
        assert_eq!(positive.style(), UiFlags::COLOR_BLUE);
        assert_eq!(positive.format(), "50%");

        let max = ResistanceDisplay::new(75);
        assert_eq!(max.style(), UiFlags::COLOR_WHITEGOLD);

        let negative = ResistanceDisplay::new(-25);
        assert_eq!(negative.style(), UiFlags::COLOR_RED);
        assert_eq!(negative.format(), "-25%");
    }

    #[test]
    fn test_damage_display() {
        let damage = DamageDisplay::new(10, 25, 0);
        assert_eq!(damage.style(), UiFlags::COLOR_WHITE);
        assert_eq!(damage.format(), "10-25");

        let boosted = DamageDisplay::new(15, 30, 50);
        assert_eq!(boosted.style(), UiFlags::COLOR_BLUE);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
    }

    #[test]
    fn test_character_panel_new() {
        let panel = CharacterPanel::new();
        assert!(!panel.visible);
        assert_eq!(panel.level, 1);
    }

    #[test]
    fn test_character_panel_toggle() {
        let mut panel = CharacterPanel::new();
        assert!(!panel.visible);

        panel.toggle();
        assert!(panel.visible);

        panel.toggle();
        assert!(!panel.visible);
    }

    #[test]
    fn test_character_panel_set_info() {
        let mut panel = CharacterPanel::new();
        panel.set_player_info("TestPlayer", HeroClass::Sorcerer, 25);

        assert_eq!(panel.player_name, "TestPlayer");
        assert_eq!(panel.player_class, HeroClass::Sorcerer);
        assert_eq!(panel.level, 25);
    }

    #[test]
    fn test_character_panel_life_style() {
        let mut panel = CharacterPanel::new();

        panel.set_life(45, 45, 45);
        assert_eq!(panel.life_style(), UiFlags::COLOR_WHITE);

        panel.set_life(30, 45, 45);
        assert_eq!(panel.life_style(), UiFlags::COLOR_RED);

        panel.set_life(60, 60, 45);
        assert_eq!(panel.life_style(), UiFlags::COLOR_BLUE);
    }

    #[test]
    fn test_ui_flags() {
        let flags = UiFlags::COLOR_WHITE.combine(UiFlags::ALIGN_CENTER);
        assert!(flags.contains(UiFlags::COLOR_WHITE));
        assert!(flags.contains(UiFlags::ALIGN_CENTER));
        assert!(!flags.contains(UiFlags::COLOR_RED));
    }
}
