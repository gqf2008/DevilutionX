//! Hero Selection - Character Creation and Selection Screens
//!
//! This module handles the hero selection UI including:
//! - Character list display
//! - Character creation (class selection, naming)
//! - Character deletion confirmation
//! - Single player vs Multiplayer modes
//!
//! ## C++ Alignment
//!
//! - `DiabloUI/hero/selhero.cpp`
//! - `DiabloUI/diabloui.h` (hero info structures)

use super::ui_core::{UiContext, UiEvent, UiEventResult, UiKeyCode};
use super::ui_item::{UiArtText, UiEdit, UiFlags, UiItem, UiList, UiListItem, UiRect};
use super::{DefaultStats, Difficulty, HeroClass};

/// Maximum hero name length
pub const MAX_NAME_LENGTH: usize = 15;

/// Maximum number of save slots
pub const MAX_SAVE_SLOTS: usize = 99;

/// Hero selection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelHeroResult {
    /// Start new dungeon
    NewDungeon,
    /// Continue existing save
    Continue,
    /// Connect to multiplayer
    Connect,
    /// Go back to previous menu
    Previous,
}

/// Hero information structure (corresponds to _uiheroinfo)
#[derive(Debug, Clone)]
pub struct HeroInfo {
    /// Save slot number
    pub save_number: u32,
    /// Character name
    pub name: String,
    /// Character level
    pub level: u8,
    /// Hero class
    pub hero_class: HeroClass,
    /// Hero rank (for multiplayer)
    pub hero_rank: u8,
    /// Current strength
    pub strength: u16,
    /// Current magic
    pub magic: u16,
    /// Current dexterity
    pub dexterity: u16,
    /// Current vitality
    pub vitality: u16,
    /// Has saved game
    pub has_saved: bool,
    /// Is a shareware character
    pub spawned: bool,
}

impl Default for HeroInfo {
    fn default() -> Self {
        Self {
            save_number: 0,
            name: String::new(),
            level: 1,
            hero_class: HeroClass::Warrior,
            hero_rank: 0,
            strength: 0,
            magic: 0,
            dexterity: 0,
            vitality: 0,
            has_saved: false,
            spawned: false,
        }
    }
}

impl HeroInfo {
    /// Create a new hero with default stats for a class
    pub fn new_for_class(class: HeroClass, name: impl Into<String>) -> Self {
        let stats = DefaultStats::for_class(class);
        Self {
            name: name.into(),
            level: 1,
            hero_class: class,
            strength: stats.strength,
            magic: stats.magic,
            dexterity: stats.dexterity,
            vitality: stats.vitality,
            has_saved: false,
            ..Default::default()
        }
    }

    /// Get display string for hero list
    pub fn display_string(&self) -> String {
        format!(
            "{} - {} Level {}",
            self.name,
            self.hero_class.name(),
            self.level
        )
    }

    /// Check if name is valid
    pub fn is_name_valid(&self) -> bool {
        !self.name.is_empty() && self.name.len() <= MAX_NAME_LENGTH
    }
}

/// Hero selection screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelHeroState {
    /// Viewing hero list
    HeroList,
    /// Creating new hero - class selection
    ClassSelect,
    /// Creating new hero - naming
    Naming,
    /// Confirming hero deletion
    DeleteConfirm,
    /// Difficulty selection (single player only)
    DifficultySelect,
}

/// Hero selection screen
#[derive(Debug)]
pub struct HeroSelection {
    /// Current state
    state: SelHeroState,
    /// Is this for multiplayer?
    multiplayer: bool,
    /// List of available heroes
    heroes: Vec<HeroInfo>,
    /// Currently selected hero index
    selected_hero: usize,
    /// New hero being created
    new_hero: Option<HeroInfo>,
    /// Selected class for new hero
    selected_class: HeroClass,
    /// Selected difficulty (single player)
    selected_difficulty: Difficulty,
    /// Final result
    result: Option<SelHeroResult>,
    /// Selected save number
    selected_save_number: u32,
    /// UI context
    ctx: UiContext,
    /// Class list items
    class_items: Vec<UiItem>,
    /// Hero list items
    hero_items: Vec<UiItem>,
}

impl HeroSelection {
    /// Create hero selection for single player
    pub fn single_player(heroes: Vec<HeroInfo>) -> Self {
        Self::new(heroes, false)
    }

    /// Create hero selection for multiplayer
    pub fn multiplayer(heroes: Vec<HeroInfo>) -> Self {
        Self::new(heroes, true)
    }

    fn new(heroes: Vec<HeroInfo>, multiplayer: bool) -> Self {
        let mut selection = Self {
            state: SelHeroState::HeroList,
            multiplayer,
            heroes,
            selected_hero: 0,
            new_hero: None,
            selected_class: HeroClass::Warrior,
            selected_difficulty: Difficulty::Normal,
            result: None,
            selected_save_number: 0,
            ctx: UiContext::new(),
            class_items: Vec::new(),
            hero_items: Vec::new(),
        };
        selection.build_hero_list();
        selection
    }

    fn build_hero_list(&mut self) {
        self.hero_items.clear();

        // Title
        self.hero_items.push(UiItem::ArtText(UiArtText::new(
            if self.multiplayer {
                "Multi Player Characters"
            } else {
                "Single Player Characters"
            },
            UiRect::new(24, 161, 590, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));

        // Build hero list items
        let mut list_items = Vec::new();

        // "New Hero" option
        list_items.push(UiListItem::new("New Hero", -1, UiFlags::NONE));

        // Existing heroes
        for (i, hero) in self.heroes.iter().enumerate() {
            list_items.push(UiListItem::new(hero.display_string(), i as i32, UiFlags::NONE));
        }

        let max_index = list_items.len().saturating_sub(1);
        let list = UiList::new(list_items, UiRect::new(64, 211, 510, 240), UiFlags::NONE, 8);

        self.hero_items.push(UiItem::List(list));

        // Update context
        self.ctx.init_list(Vec::new(), max_index, 8, false, self.selected_hero);
    }

    fn build_class_select(&mut self) {
        self.class_items.clear();

        // Title
        self.class_items.push(UiItem::ArtText(UiArtText::new(
            "Choose Class",
            UiRect::new(24, 161, 590, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));

        // Class list
        let mut list_items = Vec::new();
        for i in 0..HeroClass::COUNT {
            if let Some(class) = HeroClass::from_index(i) {
                // Bard and Barbarian are Hellfire only
                let flags = if i >= 4 {
                    // TODO: Check if Hellfire is enabled
                    UiFlags::COLOR_SILVER | UiFlags::ELEMENT_DISABLED
                } else {
                    UiFlags::NONE
                };
                list_items.push(UiListItem::new(class.name(), i as i32, flags));
            }
        }

        let list = UiList::new(list_items, UiRect::new(64, 211, 510, 180), UiFlags::NONE, 6);
        self.class_items.push(UiItem::List(list));

        self.ctx.init_list(Vec::new(), HeroClass::COUNT - 1, 6, false, 0);
    }

    fn build_name_input(&mut self) {
        // This would normally show an edit field
        // For now, we'll use a simple state
    }

    /// Handle input event
    pub fn handle_event(&mut self, event: &UiEvent) -> Option<SelHeroResult> {
        if self.result.is_some() {
            return self.result;
        }

        match self.state {
            SelHeroState::HeroList => self.handle_hero_list_event(event),
            SelHeroState::ClassSelect => self.handle_class_select_event(event),
            SelHeroState::Naming => self.handle_naming_event(event),
            SelHeroState::DeleteConfirm => self.handle_delete_confirm_event(event),
            SelHeroState::DifficultySelect => self.handle_difficulty_event(event),
        }

        self.result
    }

    fn handle_hero_list_event(&mut self, event: &UiEvent) {
        match self.ctx.handle_event(event) {
            UiEventResult::Selected(index) => {
                if index == 0 {
                    // "New Hero" selected
                    self.state = SelHeroState::ClassSelect;
                    self.build_class_select();
                } else {
                    // Existing hero selected
                    self.selected_hero = index - 1;
                    if let Some(hero) = self.heroes.get(self.selected_hero) {
                        self.selected_save_number = hero.save_number;
                        if self.multiplayer {
                            self.result = Some(SelHeroResult::Connect);
                        } else {
                            // Go to difficulty selection for existing heroes
                            self.state = SelHeroState::DifficultySelect;
                        }
                    }
                }
            }
            UiEventResult::Escape => {
                self.result = Some(SelHeroResult::Previous);
            }
            _ => {}
        }

        // Handle delete key for hero deletion
        if let UiEvent::KeyDown { keycode, .. } = event {
            if *keycode == UiKeyCode::Delete {
                if let Some(state) = &self.ctx.list_state {
                    if state.selected_index > 0 {
                        // Not "New Hero", can delete
                        self.selected_hero = state.selected_index - 1;
                        self.state = SelHeroState::DeleteConfirm;
                    }
                }
            }
        }
    }

    fn handle_class_select_event(&mut self, event: &UiEvent) {
        match self.ctx.handle_event(event) {
            UiEventResult::Selected(index) => {
                if let Some(class) = HeroClass::from_index(index) {
                    self.selected_class = class;
                    self.new_hero = Some(HeroInfo::new_for_class(class, ""));
                    self.state = SelHeroState::Naming;
                    self.build_name_input();
                }
            }
            UiEventResult::Escape => {
                self.state = SelHeroState::HeroList;
                self.build_hero_list();
            }
            _ => {}
        }
    }

    fn handle_naming_event(&mut self, event: &UiEvent) {
        match event {
            UiEvent::KeyDown { keycode, character } => {
                // Calculate save slot outside of the borrow
                let next_save_slot = self.find_next_save_slot();

                if let Some(hero) = &mut self.new_hero {
                    match keycode {
                        UiKeyCode::Return => {
                            if hero.is_name_valid() {
                                // Find next available save slot
                                hero.save_number = next_save_slot;
                                self.heroes.push(hero.clone());
                                self.selected_hero = self.heroes.len() - 1;
                                self.selected_save_number = hero.save_number;

                                if self.multiplayer {
                                    self.result = Some(SelHeroResult::Connect);
                                } else {
                                    self.state = SelHeroState::DifficultySelect;
                                }
                            }
                        }
                        UiKeyCode::Escape => {
                            self.new_hero = None;
                            self.state = SelHeroState::ClassSelect;
                            self.build_class_select();
                        }
                        UiKeyCode::Backspace => {
                            hero.name.pop();
                        }
                        _ => {
                            if let Some(c) = character {
                                if hero.name.len() < MAX_NAME_LENGTH && c.is_alphanumeric() {
                                    hero.name.push(*c);
                                }
                            }
                        }
                    }
                }
            }
            UiEvent::TextInput { text } => {
                if let Some(hero) = &mut self.new_hero {
                    for c in text.chars() {
                        if hero.name.len() < MAX_NAME_LENGTH && (c.is_alphanumeric() || c == ' ') {
                            hero.name.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_delete_confirm_event(&mut self, event: &UiEvent) {
        if let UiEvent::KeyDown { keycode, .. } = event {
            match keycode {
                UiKeyCode::Return => {
                    // Confirm deletion
                    if self.selected_hero < self.heroes.len() {
                        self.heroes.remove(self.selected_hero);
                        if self.selected_hero > 0 {
                            self.selected_hero -= 1;
                        }
                    }
                    self.state = SelHeroState::HeroList;
                    self.build_hero_list();
                }
                UiKeyCode::Escape => {
                    self.state = SelHeroState::HeroList;
                    self.build_hero_list();
                }
                _ => {}
            }
        }
    }

    fn handle_difficulty_event(&mut self, event: &UiEvent) {
        match self.ctx.handle_event(event) {
            UiEventResult::Selected(index) => {
                if let Some(diff) = Difficulty::from_index(index) {
                    self.selected_difficulty = diff;
                    if let Some(hero) = self.heroes.get(self.selected_hero) {
                        if hero.has_saved {
                            self.result = Some(SelHeroResult::Continue);
                        } else {
                            self.result = Some(SelHeroResult::NewDungeon);
                        }
                    } else {
                        self.result = Some(SelHeroResult::NewDungeon);
                    }
                }
            }
            UiEventResult::Escape => {
                self.state = SelHeroState::HeroList;
                self.build_hero_list();
            }
            _ => {}
        }
    }

    fn find_next_save_slot(&self) -> u32 {
        let mut used: Vec<u32> = self.heroes.iter().map(|h| h.save_number).collect();
        used.sort();

        for i in 0..MAX_SAVE_SLOTS as u32 {
            if !used.contains(&i) {
                return i;
            }
        }
        0 // Fallback
    }

    /// Get the selected hero
    pub fn selected_hero(&self) -> Option<&HeroInfo> {
        self.heroes.get(self.selected_hero)
    }

    /// Get the selected save number
    pub fn selected_save_number(&self) -> u32 {
        self.selected_save_number
    }

    /// Get the selected difficulty
    pub fn selected_difficulty(&self) -> Difficulty {
        self.selected_difficulty
    }

    /// Get all heroes
    pub fn heroes(&self) -> &[HeroInfo] {
        &self.heroes
    }

    /// Check if result is ready
    pub fn is_done(&self) -> bool {
        self.result.is_some()
    }

    /// Get current state (for rendering)
    pub fn current_state(&self) -> &str {
        match self.state {
            SelHeroState::HeroList => "hero_list",
            SelHeroState::ClassSelect => "class_select",
            SelHeroState::Naming => "naming",
            SelHeroState::DeleteConfirm => "delete_confirm",
            SelHeroState::DifficultySelect => "difficulty_select",
        }
    }

    /// Get new hero being created (for display)
    pub fn new_hero(&self) -> Option<&HeroInfo> {
        self.new_hero.as_ref()
    }
}

/// Validate a player name
pub fn is_valid_player_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_NAME_LENGTH {
        return false;
    }

    // Check for valid characters
    for c in name.chars() {
        if !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_' {
            return false;
        }
    }

    // Check for at least one alphanumeric character
    name.chars().any(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_info() {
        let hero = HeroInfo::new_for_class(HeroClass::Warrior, "Test");
        assert_eq!(hero.name, "Test");
        assert_eq!(hero.hero_class, HeroClass::Warrior);
        assert_eq!(hero.level, 1);
        assert_eq!(hero.strength, 30); // Warrior base strength
    }

    #[test]
    fn test_hero_display_string() {
        let hero = HeroInfo {
            name: "TestHero".to_string(),
            hero_class: HeroClass::Sorcerer,
            level: 15,
            ..Default::default()
        };
        assert_eq!(hero.display_string(), "TestHero - Sorcerer Level 15");
    }

    #[test]
    fn test_valid_player_name() {
        assert!(is_valid_player_name("ValidName"));
        assert!(is_valid_player_name("Name123"));
        assert!(is_valid_player_name("Name With Space"));
        assert!(!is_valid_player_name("")); // Empty
        assert!(!is_valid_player_name("ThisNameIsTooLongForTheGame")); // Too long
        assert!(!is_valid_player_name("Name@#$")); // Invalid chars
    }

    #[test]
    fn test_hero_selection_single_player() {
        let heroes = vec![
            HeroInfo::new_for_class(HeroClass::Warrior, "Hero1"),
            HeroInfo::new_for_class(HeroClass::Rogue, "Hero2"),
        ];

        let selection = HeroSelection::single_player(heroes);
        assert!(!selection.multiplayer);
        assert_eq!(selection.heroes.len(), 2);
    }

    #[test]
    fn test_hero_selection_multiplayer() {
        let selection = HeroSelection::multiplayer(Vec::new());
        assert!(selection.multiplayer);
        assert!(selection.heroes.is_empty());
    }

    #[test]
    fn test_find_save_slot() {
        let heroes = vec![
            HeroInfo {
                save_number: 0,
                ..Default::default()
            },
            HeroInfo {
                save_number: 1,
                ..Default::default()
            },
            HeroInfo {
                save_number: 3,
                ..Default::default()
            },
        ];

        let selection = HeroSelection::single_player(heroes);
        assert_eq!(selection.find_next_save_slot(), 2); // Slot 2 is available
    }
}
