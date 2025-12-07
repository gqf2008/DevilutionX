//! Quest Log System - M23
//!
//! Precision port of DevilutionX quests.cpp quest log functionality
//! Handles quest UI, quest state management, and quest-related level generation

// ============================================================================
// Quest IDs
// ============================================================================

/// Quest identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum QuestId {
    /// No quest / invalid
    #[default]
    None = 0,
    /// The Magic Rock quest (Griswold)
    Rock = 1,
    /// The Mushroom quest (Adria)
    Mushroom = 2,
    /// Gharbad the Weak quest
    Garbud = 3,
    /// Zhar the Mad quest
    Zhar = 4,
    /// Ogden's Sign quest
    Sign = 5,
    /// The Butcher quest
    Butcher = 6,
    /// Poisoned Water Supply quest
    PoisonWater = 7,
    /// Skeleton King quest
    SkelKing = 8,
    /// The Chamber of Bone quest
    Schamb = 9,
    /// Ogden's Tavern Sign quest
    TavernSign = 10,
    /// Archbishop Lazarus / Betrayer quest
    Betrayer = 11,
    /// Black Mushroom quest
    BlackMushroom = 12,
    /// Anvil of Fury quest
    Anvil = 13,
    /// Warlord of Blood quest
    Warlord = 14,
    /// The Blind quest
    Blind = 15,
    /// Blood altar quest
    Blood = 16,
    /// Valor quest
    Valor = 17,
    /// Lachdanan / Veil of Steel quest
    Veil = 18,
    /// King Leoric's Banner quest
    Banner = 19,
    /// Diablo quest (final boss)
    Diablo = 20,
    /// The Defiler (Hive) quest
    Defiler = 21,
    /// Na-Krul quest
    NaKrul = 22,
    /// Grave Matters quest
    GraveMat = 23,
    /// Farmer's Orchard quest
    Farmer = 24,
    /// Little Girl / Cowsuit quest
    Girl = 25,
    /// Trader quest
    Trader = 26,
    /// Theo the Trader quest
    Theo = 27,
    /// Jersey's Jersey quest
    Jersey = 28,
}

impl QuestId {
    /// Maximum number of quests
    pub const MAX_QUESTS: usize = 29;

    /// Get quest from u8 value
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Rock,
            2 => Self::Mushroom,
            3 => Self::Garbud,
            4 => Self::Zhar,
            5 => Self::Sign,
            6 => Self::Butcher,
            7 => Self::PoisonWater,
            8 => Self::SkelKing,
            9 => Self::Schamb,
            10 => Self::TavernSign,
            11 => Self::Betrayer,
            12 => Self::BlackMushroom,
            13 => Self::Anvil,
            14 => Self::Warlord,
            15 => Self::Blind,
            16 => Self::Blood,
            17 => Self::Valor,
            18 => Self::Veil,
            19 => Self::Banner,
            20 => Self::Diablo,
            21 => Self::Defiler,
            22 => Self::NaKrul,
            23 => Self::GraveMat,
            24 => Self::Farmer,
            25 => Self::Girl,
            26 => Self::Trader,
            27 => Self::Theo,
            28 => Self::Jersey,
            _ => Self::None,
        }
    }

    /// Check if this is a Hellfire quest
    pub fn is_hellfire(&self) -> bool {
        matches!(
            self,
            Self::Defiler
                | Self::NaKrul
                | Self::GraveMat
                | Self::Farmer
                | Self::Girl
                | Self::Trader
                | Self::Theo
                | Self::Jersey
        )
    }
}

// ============================================================================
// Quest State
// ============================================================================

/// Quest active state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum QuestState {
    /// Quest not started
    #[default]
    Init = 0,
    /// Quest is active
    Active = 1,
    /// Quest completed
    Done = 2,
    /// Quest not available (wrong difficulty, etc)
    NotAvailable = 3,
    /// Hive quest done (Hellfire)
    HiveDone = 4,
}

impl QuestState {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Init,
            1 => Self::Active,
            2 => Self::Done,
            3 => Self::NotAvailable,
            4 => Self::HiveDone,
            _ => Self::Init,
        }
    }

    /// Check if quest is available (init or active)
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Init | Self::Active)
    }

    /// Check if quest is completed
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done | Self::HiveDone)
    }
}

// ============================================================================
// Quest Data
// ============================================================================

/// Quest trigger names for special maps
pub const QUEST_TRIGGER_NAMES: [&str; 5] = [
    "King Leoric's Tomb",
    "The Chamber of Bone",
    "Maze",
    "A Dark Passage",
    "Unholy Altar",
];

/// Quest structure
#[derive(Debug, Clone, Default)]
pub struct Quest {
    /// Quest ID
    pub id: QuestId,
    /// Quest active state
    pub state: QuestState,
    /// Quest dungeon level
    pub level: i32,
    /// Quest dungeon type
    pub level_type: i32,
    /// Quest log has been shown
    pub log_shown: bool,
    /// Quest message displayed
    pub msg_displayed: bool,
    /// Quest position
    pub position: Point,
    /// Quest variable 1 (state tracking)
    pub var1: i32,
    /// Quest variable 2 (state tracking)
    pub var2: i32,
    /// Quest special level number
    pub special_level: i32,
    /// Quest name string
    pub name: String,
    /// Quest book order
    pub book_order: i32,
}

impl Quest {
    pub fn new(id: QuestId) -> Self {
        Self {
            id,
            state: QuestState::Init,
            ..Default::default()
        }
    }

    /// Check if quest is available
    pub fn is_available(&self) -> bool {
        self.state.is_available()
    }

    /// Check if quest is done
    pub fn is_done(&self) -> bool {
        self.state.is_done()
    }

    /// Activate quest
    pub fn activate(&mut self) {
        if self.state == QuestState::Init {
            self.state = QuestState::Active;
        }
    }

    /// Complete quest
    pub fn complete(&mut self) {
        self.state = QuestState::Done;
    }
}

// ============================================================================
// Point Type
// ============================================================================

/// 2D Point
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

// ============================================================================
// Quest Log
// ============================================================================

/// Quest log UI state
#[derive(Debug, Default)]
pub struct QuestLog {
    /// Quest log is open
    pub is_open: bool,
    /// Encountered quests (quest IDs)
    pub encountered: Vec<QuestId>,
    /// Index of first finished quest in list
    pub first_finished: usize,
    /// Currently selected quest
    pub selected: i32,
    /// List Y offset
    pub list_y_offset: i32,
    /// Line spacing
    pub line_spacing: i32,
    /// Finished quest separator offset
    pub finished_offset: i32,
}

/// Quest log constants
pub const QUEST_LOG_LINE_HEIGHT: i32 = 12;
pub const QUEST_LOG_MAX_SPACING: i32 = QUEST_LOG_LINE_HEIGHT * 2;
pub const QUEST_LOG_INNER_WIDTH: i32 = 280;
pub const QUEST_LOG_INNER_HEIGHT: i32 = 300;

impl QuestLog {
    pub fn new() -> Self {
        Self {
            is_open: false,
            encountered: Vec::with_capacity(QuestId::MAX_QUESTS),
            first_finished: 0,
            selected: -1,
            list_y_offset: 0,
            line_spacing: QUEST_LOG_LINE_HEIGHT,
            finished_offset: 0,
        }
    }

    /// Open quest log and populate list
    pub fn open(&mut self, quests: &[Quest]) {
        self.encountered.clear();
        self.first_finished = 0;

        // Add active quests first
        for quest in quests {
            if quest.state == QuestState::Active && quest.log_shown {
                self.encountered.push(quest.id);
            }
        }
        self.first_finished = self.encountered.len();

        // Add finished quests
        for quest in quests {
            if quest.state == QuestState::Done || quest.state == QuestState::HiveDone {
                self.encountered.push(quest.id);
            }
        }

        // Sort by book order (would need quest data)
        // self.encountered[..self.first_finished].sort_by(|a, b| ...);
        // self.encountered[self.first_finished..].sort_by(|a, b| ...);

        self.calculate_layout();
        self.selected = if self.first_finished == 0 { -1 } else { 0 };
        self.is_open = true;
    }

    /// Close quest log
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Toggle quest log
    pub fn toggle(&mut self, quests: &[Quest]) {
        if self.is_open {
            self.close();
        } else {
            self.open(quests);
        }
    }

    /// Calculate layout based on quest count
    fn calculate_layout(&mut self) {
        let count = self.encountered.len();
        let two_blocks = self.first_finished > 0 && self.first_finished < count;

        self.finished_offset = if two_blocks {
            QUEST_LOG_LINE_HEIGHT / 2
        } else {
            0
        };

        if count > 0 {
            let min_height = count as i32 * QUEST_LOG_LINE_HEIGHT + self.finished_offset;
            let space = QUEST_LOG_INNER_HEIGHT;
            let additional_space = space - min_height;

            let add_line_spacing = (additional_space / count as i32)
                .min(QUEST_LOG_MAX_SPACING - QUEST_LOG_LINE_HEIGHT);
            self.line_spacing = QUEST_LOG_LINE_HEIGHT + add_line_spacing;

            if two_blocks {
                let sep_space = (additional_space - add_line_spacing * count as i32)
                    .min(QUEST_LOG_LINE_HEIGHT);
                self.finished_offset = sep_space.max(4);
            }

            let overall_height = count as i32 * self.line_spacing + self.finished_offset;
            self.list_y_offset = (space - overall_height) / 2;
        } else {
            self.list_y_offset = 0;
            self.line_spacing = QUEST_LOG_LINE_HEIGHT;
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.first_finished == 0 {
            self.selected = -1;
        } else {
            self.selected -= 1;
            if self.selected < 0 {
                self.selected = self.first_finished as i32 - 1;
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.first_finished == 0 {
            self.selected = -1;
        } else {
            self.selected += 1;
            if self.selected >= self.first_finished as i32 {
                self.selected = 0;
            }
        }
    }

    /// Get selected quest ID
    pub fn get_selected(&self) -> Option<QuestId> {
        if self.selected >= 0 && (self.selected as usize) < self.encountered.len() {
            Some(self.encountered[self.selected as usize])
        } else {
            None
        }
    }

    /// Check if index is a finished quest
    pub fn is_finished(&self, index: usize) -> bool {
        index >= self.first_finished
    }

    /// Get quest count
    pub fn count(&self) -> usize {
        self.encountered.len()
    }

    /// Get active quest count
    pub fn active_count(&self) -> usize {
        self.first_finished
    }

    /// Get finished quest count
    pub fn finished_count(&self) -> usize {
        self.encountered.len() - self.first_finished
    }
}

// ============================================================================
// Quest Manager
// ============================================================================

/// Quest manager for the game
pub struct QuestManager {
    /// All quests
    pub quests: [Quest; QuestId::MAX_QUESTS],
    /// Quest log UI
    pub log: QuestLog,
    /// Return level position
    pub return_position: Point,
    /// Return level type
    pub return_level_type: i32,
    /// Return level number
    pub return_level: i32,
    /// Water done counter (for poisoned water quest)
    pub water_done: i32,
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestManager {
    pub fn new() -> Self {
        let mut quests: [Quest; QuestId::MAX_QUESTS] = Default::default();
        for (i, quest) in quests.iter_mut().enumerate() {
            quest.id = QuestId::from_u8(i as u8);
        }

        Self {
            quests,
            log: QuestLog::new(),
            return_position: Point::default(),
            return_level_type: 0,
            return_level: 0,
            water_done: 0,
        }
    }

    /// Get quest by ID
    pub fn get(&self, id: QuestId) -> &Quest {
        &self.quests[id as usize]
    }

    /// Get mutable quest by ID
    pub fn get_mut(&mut self, id: QuestId) -> &mut Quest {
        &mut self.quests[id as usize]
    }

    /// Initialize quests for a new game
    pub fn init_quests(&mut self, is_hellfire: bool, difficulty: i32) {
        for quest in &mut self.quests {
            quest.state = QuestState::Init;
            quest.log_shown = false;
            quest.msg_displayed = false;
            quest.var1 = 0;
            quest.var2 = 0;
        }

        // Disable Hellfire quests if not Hellfire
        if !is_hellfire {
            for quest in &mut self.quests {
                if quest.id.is_hellfire() {
                    quest.state = QuestState::NotAvailable;
                }
            }
        }

        // Some quests are only available on certain difficulties
        // This would be configured based on quest data
        let _ = difficulty;
    }

    /// Check if quest is available
    pub fn is_quest_available(&self, id: QuestId) -> bool {
        self.quests[id as usize].is_available()
    }

    /// Activate a quest
    pub fn activate_quest(&mut self, id: QuestId) {
        self.quests[id as usize].activate();
    }

    /// Complete a quest
    pub fn complete_quest(&mut self, id: QuestId) {
        self.quests[id as usize].complete();
    }

    /// Get map return level
    pub fn get_map_return_level(&self, set_level_num: i32) -> i32 {
        match set_level_num {
            1 => self.get(QuestId::SkelKing).level, // SL_SKELKING
            2 => self.get(QuestId::Schamb).level,   // SL_BONECHAMB
            3 => self.get(QuestId::PoisonWater).level, // SL_POISONWATER
            5 => self.get(QuestId::Betrayer).level, // SL_VILEBETRAYER
            _ => 0,
        }
    }

    /// Toggle quest log
    pub fn toggle_quest_log(&mut self) {
        self.log.toggle(&self.quests);
    }

    /// Open quest log
    pub fn open_quest_log(&mut self) {
        self.log.open(&self.quests);
    }

    /// Close quest log
    pub fn close_quest_log(&mut self) {
        self.log.close();
    }

    /// Check quest log status
    pub fn is_quest_log_open(&self) -> bool {
        self.log.is_open
    }

    /// Update water done counter
    pub fn update_water_palette(&mut self) {
        if self.water_done > 0 {
            self.water_done -= 1;
        }
    }
}

// ============================================================================
// Quest State Variables (Gharbad, Zhar, etc.)
// ============================================================================

/// Gharbad quest states
pub mod gharbad_state {
    pub const INIT: i32 = 0;
    pub const ITEM_SPAWNED: i32 = 1;
    pub const ANGRY: i32 = 2;
    pub const ATTACKING: i32 = 3;
}

/// Zhar quest states
pub mod zhar_state {
    pub const INIT: i32 = 0;
    pub const ITEM_SPAWNED: i32 = 1;
    pub const ANGRY: i32 = 2;
    pub const ATTACKING: i32 = 3;
}

/// Warlord quest states
pub mod warlord_state {
    pub const INIT: i32 = 0;
    pub const ATTACKING: i32 = 1;
}

/// Veil quest states
pub mod veil_state {
    pub const INIT: i32 = 0;
    pub const EARLY_RETURN: i32 = 1;
    pub const ITEM_SPAWNED: i32 = 2;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_id() {
        assert_eq!(QuestId::from_u8(6), QuestId::Butcher);
        assert_eq!(QuestId::from_u8(8), QuestId::SkelKing);
        assert_eq!(QuestId::from_u8(20), QuestId::Diablo);
        assert!(QuestId::Defiler.is_hellfire());
        assert!(!QuestId::Butcher.is_hellfire());
    }

    #[test]
    fn test_quest_state() {
        assert!(QuestState::Init.is_available());
        assert!(QuestState::Active.is_available());
        assert!(!QuestState::Done.is_available());
        assert!(QuestState::Done.is_done());
        assert!(QuestState::HiveDone.is_done());
    }

    #[test]
    fn test_quest() {
        let mut quest = Quest::new(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Init);
        assert!(quest.is_available());

        quest.activate();
        assert_eq!(quest.state, QuestState::Active);

        quest.complete();
        assert_eq!(quest.state, QuestState::Done);
        assert!(!quest.is_available());
    }

    #[test]
    fn test_quest_log() {
        let mut log = QuestLog::new();
        assert!(!log.is_open);
        assert_eq!(log.count(), 0);

        // Create some test quests
        let mut quests = vec![
            Quest::new(QuestId::Butcher),
            Quest::new(QuestId::SkelKing),
        ];
        quests[0].state = QuestState::Active;
        quests[0].log_shown = true;
        quests[1].state = QuestState::Done;

        log.open(&quests);
        assert!(log.is_open);
        assert_eq!(log.active_count(), 1);
        assert_eq!(log.finished_count(), 1);
    }

    #[test]
    fn test_quest_log_navigation() {
        let mut log = QuestLog::new();

        // Create active quests
        let mut quests = vec![
            Quest::new(QuestId::Butcher),
            Quest::new(QuestId::SkelKing),
            Quest::new(QuestId::PoisonWater),
        ];
        for q in &mut quests {
            q.state = QuestState::Active;
            q.log_shown = true;
        }

        log.open(&quests);
        assert_eq!(log.selected, 0);

        log.move_down();
        assert_eq!(log.selected, 1);

        log.move_down();
        assert_eq!(log.selected, 2);

        log.move_down();
        assert_eq!(log.selected, 0); // Wrap around

        log.move_up();
        assert_eq!(log.selected, 2);
    }

    #[test]
    fn test_quest_manager() {
        let mut manager = QuestManager::new();

        assert!(manager.is_quest_available(QuestId::Butcher));
        assert_eq!(manager.get(QuestId::Butcher).state, QuestState::Init);

        manager.activate_quest(QuestId::Butcher);
        assert_eq!(manager.get(QuestId::Butcher).state, QuestState::Active);

        manager.complete_quest(QuestId::Butcher);
        assert_eq!(manager.get(QuestId::Butcher).state, QuestState::Done);
    }

    #[test]
    fn test_quest_manager_hellfire() {
        let mut manager = QuestManager::new();

        // Without Hellfire
        manager.init_quests(false, 0);
        assert!(!manager.is_quest_available(QuestId::Defiler));
        assert!(!manager.is_quest_available(QuestId::NaKrul));

        // With Hellfire
        manager.init_quests(true, 0);
        assert!(manager.is_quest_available(QuestId::Defiler));
    }

    #[test]
    fn test_quest_log_toggle() {
        let mut manager = QuestManager::new();

        assert!(!manager.is_quest_log_open());
        manager.toggle_quest_log();
        assert!(manager.is_quest_log_open());
        manager.toggle_quest_log();
        assert!(!manager.is_quest_log_open());
    }
}
