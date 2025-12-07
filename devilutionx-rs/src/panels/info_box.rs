//! Info Box - Contextual Information Display (M64)
//!
//! Provides the info box panel for contextual information:
//! - Item descriptions and stats
//! - Monster information
//! - Spell details
//! - Object/shrine information
//!
//! ## C++ References
//! - Source/panels/info_box.cpp
//! - Source/panels/info_box.hpp

#![allow(dead_code)]
#![allow(unused_imports)]

use super::main_panel::{Point, Size, Rectangle};
use super::char_panel::UiFlags;

/// Info box dimensions
pub const INFO_BOX_WIDTH: i32 = 320;
pub const INFO_BOX_HEIGHT: i32 = 128;
pub const INFO_BOX_LINE_HEIGHT: i32 = 12;

/// Maximum lines in info box
pub const MAX_INFO_LINES: usize = 8;

/// Info box content type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoBoxType {
    /// No content
    None,
    /// Item information
    Item,
    /// Monster information
    Monster,
    /// Spell information
    Spell,
    /// Object/shrine information
    Object,
    /// General text message
    Text,
    /// Help text
    Help,
}

/// A line of text in the info box
#[derive(Debug, Clone)]
pub struct InfoLine {
    /// Text content
    pub text: String,
    /// Text style
    pub style: UiFlags,
    /// Vertical offset from default position
    pub y_offset: i32,
}

impl InfoLine {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: UiFlags::COLOR_WHITE,
            y_offset: 0,
        }
    }

    pub fn with_style(mut self, style: UiFlags) -> Self {
        self.style = style;
        self
    }

    pub fn with_offset(mut self, offset: i32) -> Self {
        self.y_offset = offset;
        self
    }
}

impl Default for InfoLine {
    fn default() -> Self {
        Self::new("")
    }
}

/// Item quality colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemQuality {
    /// Normal item (white)
    Normal,
    /// Magic item (blue)
    Magic,
    /// Unique item (gold)
    Unique,
    /// Quest item (orange/special)
    Quest,
    /// Broken/cursed item (red)
    Broken,
}

impl ItemQuality {
    pub fn color(&self) -> UiFlags {
        match self {
            Self::Normal => UiFlags::COLOR_WHITE,
            Self::Magic => UiFlags::COLOR_BLUE,
            Self::Unique => UiFlags::COLOR_WHITEGOLD,
            Self::Quest => UiFlags::COLOR_WHITEGOLD,
            Self::Broken => UiFlags::COLOR_RED,
        }
    }
}

/// Item display info for info box
#[derive(Debug, Clone)]
pub struct ItemInfo {
    /// Item name
    pub name: String,
    /// Item quality (determines color)
    pub quality: ItemQuality,
    /// Item type (e.g., "Sword", "Helm")
    pub item_type: String,
    /// Damage or armor value
    pub stat_line: Option<String>,
    /// Requirements (str, mag, dex)
    pub requirements: Option<String>,
    /// Durability
    pub durability: Option<String>,
    /// Special properties
    pub properties: Vec<String>,
    /// Gold value
    pub value: Option<u32>,
    /// Is identified?
    pub identified: bool,
}

impl ItemInfo {
    pub fn new(name: &str, quality: ItemQuality) -> Self {
        Self {
            name: name.to_string(),
            quality,
            item_type: String::new(),
            stat_line: None,
            requirements: None,
            durability: None,
            properties: Vec::new(),
            value: None,
            identified: true,
        }
    }

    /// Convert to info lines
    pub fn to_lines(&self) -> Vec<InfoLine> {
        let mut lines = Vec::new();

        // Name
        lines.push(InfoLine::new(&self.name).with_style(self.quality.color()));

        // Type
        if !self.item_type.is_empty() {
            lines.push(InfoLine::new(&self.item_type));
        }

        // Stats
        if let Some(ref stat) = self.stat_line {
            lines.push(InfoLine::new(stat));
        }

        if self.identified {
            // Properties
            for prop in &self.properties {
                lines.push(InfoLine::new(prop).with_style(UiFlags::COLOR_BLUE));
            }

            // Requirements
            if let Some(ref req) = self.requirements {
                lines.push(InfoLine::new(req));
            }

            // Durability
            if let Some(ref dur) = self.durability {
                lines.push(InfoLine::new(dur));
            }
        } else {
            lines.push(InfoLine::new("Not identified"));
        }

        lines
    }
}

/// Monster display info
#[derive(Debug, Clone)]
pub struct MonsterInfo {
    /// Monster name
    pub name: String,
    /// Monster level (optional)
    pub level: Option<i32>,
    /// Monster type (e.g., "Undead", "Animal")
    pub monster_type: Option<String>,
    /// Hit points (if visible)
    pub hit_points: Option<(i32, i32)>, // (current, max)
    /// Is unique monster?
    pub is_unique: bool,
    /// Is friendly?
    pub is_friendly: bool,
    /// Immunities
    pub immunities: Vec<String>,
    /// Resistances
    pub resistances: Vec<String>,
}

impl MonsterInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: None,
            monster_type: None,
            hit_points: None,
            is_unique: false,
            is_friendly: false,
            immunities: Vec::new(),
            resistances: Vec::new(),
        }
    }

    /// Convert to info lines
    pub fn to_lines(&self) -> Vec<InfoLine> {
        let mut lines = Vec::new();

        // Name (gold for unique, white otherwise)
        let name_style = if self.is_unique {
            UiFlags::COLOR_WHITEGOLD
        } else if self.is_friendly {
            UiFlags::COLOR_BLUE
        } else {
            UiFlags::COLOR_WHITE
        };
        lines.push(InfoLine::new(&self.name).with_style(name_style));

        // Level and type
        let mut type_line = String::new();
        if let Some(lvl) = self.level {
            type_line = format!("Level: {}", lvl);
        }
        if let Some(ref mt) = self.monster_type {
            if !type_line.is_empty() {
                type_line.push_str(" - ");
            }
            type_line.push_str(mt);
        }
        if !type_line.is_empty() {
            lines.push(InfoLine::new(&type_line));
        }

        // Hit points
        if let Some((cur, max)) = self.hit_points {
            let hp_style = if cur < max / 3 {
                UiFlags::COLOR_RED
            } else if cur < max * 2 / 3 {
                UiFlags::COLOR_WHITE
            } else {
                UiFlags::COLOR_BLUE
            };
            lines.push(InfoLine::new(&format!("Hit Points: {}/{}", cur, max)).with_style(hp_style));
        }

        // Immunities
        if !self.immunities.is_empty() {
            let imm_text = format!("Immune: {}", self.immunities.join(", "));
            lines.push(InfoLine::new(&imm_text).with_style(UiFlags::COLOR_RED));
        }

        // Resistances
        if !self.resistances.is_empty() {
            let res_text = format!("Resist: {}", self.resistances.join(", "));
            lines.push(InfoLine::new(&res_text));
        }

        lines
    }
}

/// Info box panel
#[derive(Debug, Clone)]
pub struct InfoBox {
    /// Panel position
    pub position: Point,
    /// Panel size
    pub size: Size,
    /// Is panel visible?
    pub visible: bool,
    /// Content type
    pub content_type: InfoBoxType,
    /// Lines to display
    pub lines: Vec<InfoLine>,
    /// Title (optional)
    pub title: Option<String>,
    /// Show border?
    pub show_border: bool,
    /// Background alpha (0-255)
    pub background_alpha: u8,
}

impl InfoBox {
    /// Create a new info box
    pub fn new() -> Self {
        Self {
            position: Point::new(0, 0),
            size: Size::new(INFO_BOX_WIDTH, INFO_BOX_HEIGHT),
            visible: false,
            content_type: InfoBoxType::None,
            lines: Vec::with_capacity(MAX_INFO_LINES),
            title: None,
            show_border: true,
            background_alpha: 192,
        }
    }

    /// Set position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
    }

    /// Center above a point
    pub fn center_above(&mut self, x: i32, y: i32, margin: i32) {
        self.position.x = x - self.size.width / 2;
        self.position.y = y - self.size.height - margin;
    }

    /// Show the info box
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the info box
    pub fn hide(&mut self) {
        self.visible = false;
        self.clear();
    }

    /// Clear content
    pub fn clear(&mut self) {
        self.lines.clear();
        self.title = None;
        self.content_type = InfoBoxType::None;
    }

    /// Set item info
    pub fn set_item(&mut self, item: &ItemInfo) {
        self.clear();
        self.content_type = InfoBoxType::Item;
        self.title = Some(item.name.clone());
        self.lines = item.to_lines();
        self.visible = true;
    }

    /// Set monster info
    pub fn set_monster(&mut self, monster: &MonsterInfo) {
        self.clear();
        self.content_type = InfoBoxType::Monster;
        self.title = Some(monster.name.clone());
        self.lines = monster.to_lines();
        self.visible = true;
    }

    /// Set text content
    pub fn set_text(&mut self, lines: Vec<InfoLine>) {
        self.clear();
        self.content_type = InfoBoxType::Text;
        self.lines = lines;
        self.visible = true;
    }

    /// Set single line text
    pub fn set_single_line(&mut self, text: &str, style: UiFlags) {
        self.clear();
        self.content_type = InfoBoxType::Text;
        self.lines.push(InfoLine::new(text).with_style(style));
        self.visible = true;
    }

    /// Add a line
    pub fn add_line(&mut self, line: InfoLine) {
        if self.lines.len() < MAX_INFO_LINES {
            self.lines.push(line);
        }
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Calculate required height based on content
    pub fn calculate_height(&self) -> i32 {
        let line_count = self.lines.len() as i32;
        let title_height = if self.title.is_some() {
            INFO_BOX_LINE_HEIGHT + 4
        } else {
            0
        };
        let border_padding = if self.show_border { 8 } else { 0 };

        title_height + (line_count * INFO_BOX_LINE_HEIGHT) + border_padding
    }

    /// Resize to fit content
    pub fn resize_to_fit(&mut self) {
        self.size.height = self.calculate_height().max(32);
    }

    /// Get rectangle for the info box
    pub fn get_rect(&self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y,
            self.size.width,
            self.size.height,
        )
    }

    /// Check if point is inside info box
    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.visible && self.get_rect().contains(Point::new(x, y))
    }
}

impl Default for InfoBox {
    fn default() -> Self {
        Self::new()
    }
}

/// Format item stat line (damage or armor)
pub fn format_item_stat(damage_min: i32, damage_max: i32) -> String {
    if damage_min == damage_max {
        format!("Damage: {}", damage_min)
    } else {
        format!("Damage: {}-{}", damage_min, damage_max)
    }
}

/// Format armor stat
pub fn format_armor_stat(armor: i32) -> String {
    format!("Armor: {}", armor)
}

/// Format durability
pub fn format_durability(current: i32, max: i32) -> String {
    format!("Durability: {}/{}", current, max)
}

/// Format requirements
pub fn format_requirements(str_req: i32, mag_req: i32, dex_req: i32) -> Option<String> {
    let mut parts = Vec::new();

    if str_req > 0 {
        parts.push(format!("{} Str", str_req));
    }
    if mag_req > 0 {
        parts.push(format!("{} Mag", mag_req));
    }
    if dex_req > 0 {
        parts.push(format!("{} Dex", dex_req));
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("Required: {}", parts.join(", ")))
    }
}

/// Format gold value
pub fn format_gold_value(value: u32) -> String {
    format!("{} gold", value)
}

/// Tooltip style info box (appears on hover)
#[derive(Debug, Clone)]
pub struct Tooltip {
    /// Info box for display
    pub info_box: InfoBox,
    /// Delay before showing (in ticks)
    pub show_delay: u32,
    /// Current delay counter
    pub delay_counter: u32,
    /// Is showing (after delay)?
    pub showing: bool,
}

impl Tooltip {
    pub fn new() -> Self {
        Self {
            info_box: InfoBox::new(),
            show_delay: 30, // About 1 second at 30fps
            delay_counter: 0,
            showing: false,
        }
    }

    /// Reset the tooltip
    pub fn reset(&mut self) {
        self.info_box.hide();
        self.delay_counter = 0;
        self.showing = false;
    }

    /// Start showing tooltip (after delay)
    pub fn start(&mut self, x: i32, y: i32) {
        if !self.showing {
            self.info_box.set_position(x, y);
            self.delay_counter = 0;
        }
    }

    /// Update tooltip (call each frame while hovering)
    pub fn update(&mut self) {
        if !self.showing {
            self.delay_counter += 1;
            if self.delay_counter >= self.show_delay {
                self.showing = true;
                self.info_box.show();
            }
        }
    }

    /// Set tooltip content
    pub fn set_content(&mut self, lines: Vec<InfoLine>) {
        self.info_box.set_text(lines);
        self.info_box.resize_to_fit();
    }
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_line() {
        let line = InfoLine::new("Test text")
            .with_style(UiFlags::COLOR_BLUE)
            .with_offset(5);

        assert_eq!(line.text, "Test text");
        assert_eq!(line.style, UiFlags::COLOR_BLUE);
        assert_eq!(line.y_offset, 5);
    }

    #[test]
    fn test_item_quality_color() {
        assert_eq!(ItemQuality::Normal.color(), UiFlags::COLOR_WHITE);
        assert_eq!(ItemQuality::Magic.color(), UiFlags::COLOR_BLUE);
        assert_eq!(ItemQuality::Unique.color(), UiFlags::COLOR_WHITEGOLD);
        assert_eq!(ItemQuality::Broken.color(), UiFlags::COLOR_RED);
    }

    #[test]
    fn test_item_info_to_lines() {
        let mut item = ItemInfo::new("Long Sword of the Stars", ItemQuality::Unique);
        item.item_type = "Sword".to_string();
        item.stat_line = Some("Damage: 8-16".to_string());
        item.properties.push("+50 Life".to_string());
        item.identified = true;

        let lines = item.to_lines();
        assert!(lines.len() >= 3);
        assert_eq!(lines[0].text, "Long Sword of the Stars");
    }

    #[test]
    fn test_monster_info_to_lines() {
        let mut monster = MonsterInfo::new("The Butcher");
        monster.is_unique = true;
        monster.level = Some(10);
        monster.hit_points = Some((100, 100));
        monster.immunities.push("Magic".to_string());

        let lines = monster.to_lines();
        assert!(lines.len() >= 3);
        assert_eq!(lines[0].style, UiFlags::COLOR_WHITEGOLD);
    }

    #[test]
    fn test_info_box_new() {
        let info_box = InfoBox::new();
        assert!(!info_box.visible);
        assert_eq!(info_box.content_type, InfoBoxType::None);
    }

    #[test]
    fn test_info_box_set_item() {
        let mut info_box = InfoBox::new();
        let item = ItemInfo::new("Test Item", ItemQuality::Magic);

        info_box.set_item(&item);
        assert!(info_box.visible);
        assert_eq!(info_box.content_type, InfoBoxType::Item);
    }

    #[test]
    fn test_info_box_calculate_height() {
        let mut info_box = InfoBox::new();
        info_box.add_line(InfoLine::new("Line 1"));
        info_box.add_line(InfoLine::new("Line 2"));
        info_box.add_line(InfoLine::new("Line 3"));

        let height = info_box.calculate_height();
        assert!(height > 0);
        assert_eq!(height, 3 * INFO_BOX_LINE_HEIGHT + 8);
    }

    #[test]
    fn test_format_functions() {
        assert_eq!(format_item_stat(5, 10), "Damage: 5-10");
        assert_eq!(format_item_stat(10, 10), "Damage: 10");
        assert_eq!(format_armor_stat(25), "Armor: 25");
        assert_eq!(format_durability(40, 60), "Durability: 40/60");
        assert_eq!(format_gold_value(1000), "1000 gold");
    }

    #[test]
    fn test_format_requirements() {
        assert_eq!(format_requirements(25, 0, 0), Some("Required: 25 Str".to_string()));
        assert_eq!(format_requirements(20, 15, 0), Some("Required: 20 Str, 15 Mag".to_string()));
        assert_eq!(format_requirements(0, 0, 0), None);
    }

    #[test]
    fn test_tooltip() {
        let mut tooltip = Tooltip::new();
        tooltip.show_delay = 3;

        tooltip.start(100, 100);
        assert!(!tooltip.showing);

        tooltip.update();
        tooltip.update();
        assert!(!tooltip.showing);

        tooltip.update();
        assert!(tooltip.showing);

        tooltip.reset();
        assert!(!tooltip.showing);
    }
}
