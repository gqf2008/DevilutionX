//! In-Game Message System (DiabloMsg)
//!
//! This module implements the in-game message display system used in Diablo.
//! Messages appear in a decorated box in the center of the screen and can
//! display various game notifications including shrine effects, save status,
//! and level requirements.
//!
//! # C++ Source Reference
//! - Source/diablo_msg.cpp
//! - Source/diablo_msg.hpp
//!
//! # Features
//! - Message queue (FIFO) for sequential display
//! - Configurable display duration per message
//! - Automatic word wrapping
//! - Duplicate message prevention
//! - Decorated border rendering

use std::collections::VecDeque;

/// Default message duration in milliseconds
pub const DEFAULT_DURATION_MS: u32 = 3500;

/// Minimum line width for message box
pub const MIN_LINE_WIDTH: i32 = 418;

/// Border part dimensions
pub const BORDER_PART_WIDTH: i32 = 12;
pub const BORDER_PART_HEIGHT: i32 = 12;

/// Border thickness
pub const BORDER_THICKNESS: i32 = 3;

/// Text padding
pub const TEXT_PADDING_X: i32 = 5;

/// Message types for predefined messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiabloMessage {
    None = 0,
    GameSaved = 1,
    NoMultiplayerInDemo = 2,
    DirectSoundFailed = 3,
    NotInShareware = 4,
    NotEnoughSpace = 5,
    NoPauseInTown = 6,
    CopyToHardDisk = 7,
    MultiplayerSyncProblem = 8,
    NoPauseInMultiplayer = 9,
    Loading = 10,
    Saving = 11,
    // Shrine messages
    ShrineSomeAreWeakened = 12,
    ShrineNewStrength = 13,
    ShrineDefendSeldom = 14,
    ShrineSwordOfJustice = 15,
    ShrineSpiritVigilant = 16,
    ShrineManaRefocused = 17,
    ShrineTimeCannot = 18,
    ShrineMagicNotAlways = 19,
    ShrineWhatOnceWas = 20,
    ShrineIntensityCost = 21,
    ShrineArcanePower = 22,
    ShrineThatWhichCannot = 23,
    ShrineCrimsonAzure = 24,
    ShrineKnowledgeWisdom = 25,
    ShrineDrinkRefreshed = 26,
    ShrineWhereverYouGo = 27,
    ShrineEnergyCost = 28,
    ShrineRichesAbound = 29,
    ShrineAvariceFails = 30,
    ShrineBlessedCompanion = 31,
    ShrineHandsOfMen = 32,
    ShrineStrengthBolstered = 33,
    ShrineEssenceOfLife = 34,
    ShrineWayMadeClear = 35,
    ShrineSalvationCost = 36,
    ShrineMysteries = 37,
    ShrineLastMayBeFirst = 38,
    ShrineGenerosityRewards = 39,
    // Level requirements
    Level8Required = 40,
    Level13Required = 41,
    Level17Required = 42,
    // More shrine messages
    ShrineArcaneKnowledge = 43,
    ShrineThatWhichNotKill = 44,
    ShrineKnowledgeIsPower = 45,
    ShrineGiveAndReceive = 46,
    ShrineSomeExperience = 47,
    ShrineNoPlaceLikeHome = 48,
    ShrineSpiritualEnergy = 49,
    ShrineMoreAgile = 50,
    ShrineFeelStronger = 51,
    ShrineFeelWiser = 52,
    ShrineFeelRefreshed = 53,
    ShrineThatWhichCanBreak = 54,
}

/// Message string lookup table
pub const MSG_STRINGS: &[&str] = &[
    "",
    "Game saved",
    "No multiplayer functions in demo",
    "Direct Sound Creation Failed",
    "Not available in shareware version",
    "Not enough space to save",
    "No Pause in town",
    "Copying to a hard disk is recommended",
    "Multiplayer sync problem",
    "No pause in multiplayer",
    "Loading...",
    "Saving...",
    // Shrine messages
    "Some are weakened as one grows strong",
    "New strength is forged through destruction",
    "Those who defend seldom attack",
    "The sword of justice is swift and sharp",
    "While the spirit is vigilant the body thrives",
    "The powers of mana refocused renews",
    "Time cannot diminish the power of steel",
    "Magic is not always what it seems to be",
    "What once was opened now is closed",
    "Intensity comes at the cost of wisdom",
    "Arcane power brings destruction",
    "That which cannot be held cannot be harmed",
    "Crimson and Azure become as the sun",
    "Knowledge and wisdom at the cost of self",
    "Drink and be refreshed",
    "Wherever you go, there you are",
    "Energy comes at the cost of wisdom",
    "Riches abound when least expected",
    "Where avarice fails, patience gains reward",
    "Blessed by a benevolent companion!",
    "The hands of men may be guided by fate",
    "Strength is bolstered by heavenly faith",
    "The essence of life flows from within",
    "The way is made clear when viewed from above",
    "Salvation comes at the cost of wisdom",
    "Mysteries are revealed in the light of reason",
    "Those who are last may yet be first",
    "Generosity brings its own rewards",
    // Level requirements
    "You must be at least level 8 to use this.",
    "You must be at least level 13 to use this.",
    "You must be at least level 17 to use this.",
    // More shrine messages
    "Arcane knowledge gained!",
    "That which does not kill you...",
    "Knowledge is power.",
    "Give and you shall receive.",
    "Some experience is gained by touch.",
    "There's no place like home.",
    "Spiritual energy is restored.",
    "You feel more agile.",
    "You feel stronger.",
    "You feel wiser.",
    "You feel refreshed.",
    "That which can break will.",
];

/// A single message entry in the queue
#[derive(Debug, Clone)]
pub struct MessageEntry {
    /// The message text
    pub text: String,
    /// Duration to display in milliseconds
    pub duration_ms: u32,
}

impl MessageEntry {
    /// Creates a new message entry
    pub fn new(text: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            text: text.into(),
            duration_ms,
        }
    }
}

/// Message display state
#[derive(Debug, Clone)]
pub struct MessageDisplayState {
    /// Word-wrapped text lines for display
    pub lines: Vec<String>,
    /// Calculated outer height
    pub outer_height: i32,
    /// Calculated line width
    pub line_width: i32,
    /// Line height for the current font
    pub line_height: i32,
}

impl Default for MessageDisplayState {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            outer_height: 0,
            line_width: MIN_LINE_WIDTH,
            line_height: 0,
        }
    }
}

/// In-game message manager
///
/// Handles the queue of messages to display and manages
/// their timing and rendering state.
#[derive(Debug, Clone, Default)]
pub struct DiabloMsgManager {
    /// Queue of messages to display
    messages: VecDeque<MessageEntry>,
    /// Timestamp when current message started (milliseconds)
    start_time: u32,
    /// Current display state
    display: MessageDisplayState,
}

impl DiabloMsgManager {
    /// Creates a new message manager
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            start_time: 0,
            display: MessageDisplayState::default(),
        }
    }

    /// Initializes display state for the current message
    fn init_display(&mut self, current_time: u32, font_line_height: i32) {
        self.display.lines.clear();

        if self.messages.is_empty() {
            return;
        }

        let text = &self.messages.front().unwrap().text;
        self.display.line_width = MIN_LINE_WIDTH;

        // Simple word wrapping
        let wrapped = word_wrap_simple(text, MIN_LINE_WIDTH);
        for line in wrapped.lines() {
            // Track max line width (simplified - real impl uses font metrics)
            let line_width = line.len() as i32 * 8; // Approximate
            self.display.line_width = self.display.line_width.max(line_width);
            self.display.lines.push(line.to_string());
        }

        self.start_time = current_time;
        self.display.line_height = font_line_height.max(16);
        self.display.outer_height =
            (self.display.lines.len() as i32 * self.display.line_height) + 42;
    }

    /// Adds a message by ID
    ///
    /// # Arguments
    /// * `msg` - The message ID
    /// * `duration_ms` - Display duration in milliseconds
    /// * `current_time` - Current timestamp
    /// * `font_line_height` - Line height for the font
    pub fn init_message(
        &mut self,
        msg: DiabloMessage,
        duration_ms: u32,
        current_time: u32,
        font_line_height: i32,
    ) {
        let msg_idx = msg as usize;
        if msg_idx < MSG_STRINGS.len() {
            self.init_message_text(MSG_STRINGS[msg_idx], duration_ms, current_time, font_line_height);
        }
    }

    /// Adds a message by text
    ///
    /// # Arguments
    /// * `msg` - The message text
    /// * `duration_ms` - Display duration in milliseconds
    /// * `current_time` - Current timestamp
    /// * `font_line_height` - Line height for the font
    ///
    /// # C++ Reference
    /// ```cpp
    /// void InitDiabloMsg(std::string_view msg, uint32_t duration = 3500)
    /// {
    ///     if (c_find_if(DiabloMessages, ...) != DiabloMessages.end())
    ///         return; // Duplicate prevention
    ///     DiabloMessages.push_back({ std::string(msg), duration });
    ///     if (DiabloMessages.size() == 1) {
    ///         InitDiabloMsg();
    ///     }
    /// }
    /// ```
    pub fn init_message_text(
        &mut self,
        msg: &str,
        duration_ms: u32,
        current_time: u32,
        font_line_height: i32,
    ) {
        // Prevent duplicate messages
        if self.messages.iter().any(|entry| entry.text == msg) {
            return;
        }

        self.messages.push_back(MessageEntry::new(msg, duration_ms));

        // Initialize display if this is the first message
        if self.messages.len() == 1 {
            self.init_display(current_time, font_line_height);
        }
    }

    /// Checks if there are messages to display
    pub fn is_available(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Cancels the current message and moves to the next
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp
    /// * `font_line_height` - Line height for the font
    pub fn cancel_current(&mut self, current_time: u32, font_line_height: i32) {
        if !self.messages.is_empty() {
            self.messages.pop_front();
            self.init_display(current_time, font_line_height);
        }
    }

    /// Clears all messages
    pub fn clear_all(&mut self) {
        self.messages.clear();
        self.display = MessageDisplayState::default();
    }

    /// Updates the message system
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp
    /// * `font_line_height` - Line height for the font
    ///
    /// # Returns
    /// `true` if a message is currently being displayed
    pub fn update(&mut self, current_time: u32, font_line_height: i32) -> bool {
        if self.messages.is_empty() {
            return false;
        }

        let elapsed = current_time.saturating_sub(self.start_time);
        let duration = self.messages.front().unwrap().duration_ms;

        if elapsed >= duration {
            self.messages.pop_front();
            self.init_display(current_time, font_line_height);
        }

        !self.messages.is_empty()
    }

    /// Gets the current message text lines for rendering
    pub fn get_display_lines(&self) -> &[String] {
        &self.display.lines
    }

    /// Gets the current display state
    pub fn get_display_state(&self) -> &MessageDisplayState {
        &self.display
    }

    /// Gets the number of queued messages
    pub fn queue_len(&self) -> usize {
        self.messages.len()
    }

    /// Gets remaining time for current message in milliseconds
    pub fn remaining_time(&self, current_time: u32) -> u32 {
        if self.messages.is_empty() {
            return 0;
        }

        let elapsed = current_time.saturating_sub(self.start_time);
        let duration = self.messages.front().unwrap().duration_ms;
        duration.saturating_sub(elapsed)
    }
}

/// Simple word wrapping implementation
///
/// # Arguments
/// * `text` - The text to wrap
/// * `max_width` - Maximum width in pixels
///
/// # Returns
/// Word-wrapped string with newlines
fn word_wrap_simple(text: &str, max_width: i32) -> String {
    // Approximate characters per line (assuming ~8 pixels per character)
    let chars_per_line = (max_width / 8).max(1) as usize;

    let mut result = String::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= chars_per_line {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&current_line);
    }

    result
}

/// Gets the message string for a DiabloMessage ID
pub fn get_message_string(msg: DiabloMessage) -> &'static str {
    let idx = msg as usize;
    if idx < MSG_STRINGS.len() {
        MSG_STRINGS[idx]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let manager = DiabloMsgManager::new();
        assert!(!manager.is_available());
        assert_eq!(manager.queue_len(), 0);
    }

    #[test]
    fn test_add_message_by_text() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("Test message", 3500, 0, 20);

        assert!(manager.is_available());
        assert_eq!(manager.queue_len(), 1);
    }

    #[test]
    fn test_add_message_by_id() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message(DiabloMessage::GameSaved, 3500, 0, 20);

        assert!(manager.is_available());
        let lines = manager.get_display_lines();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("saved"));
    }

    #[test]
    fn test_duplicate_prevention() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("Same message", 3500, 0, 20);
        manager.init_message_text("Same message", 3500, 0, 20); // Duplicate
        manager.init_message_text("Different message", 3500, 0, 20);

        assert_eq!(manager.queue_len(), 2);
    }

    #[test]
    fn test_message_queue_fifo() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("First", 100, 0, 20);
        manager.init_message_text("Second", 100, 0, 20);
        manager.init_message_text("Third", 100, 0, 20);

        // First message should be showing
        assert!(manager.get_display_lines().join(" ").contains("First"));

        // Cancel first, should show second
        manager.cancel_current(0, 20);
        assert!(manager.get_display_lines().join(" ").contains("Second"));

        // Cancel second, should show third
        manager.cancel_current(0, 20);
        assert!(manager.get_display_lines().join(" ").contains("Third"));
    }

    #[test]
    fn test_cancel_message() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("Message", 3500, 0, 20);

        assert!(manager.is_available());
        manager.cancel_current(0, 20);
        assert!(!manager.is_available());
    }

    #[test]
    fn test_clear_all() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("First", 3500, 0, 20);
        manager.init_message_text("Second", 3500, 0, 20);
        manager.init_message_text("Third", 3500, 0, 20);

        manager.clear_all();
        assert!(!manager.is_available());
        assert_eq!(manager.queue_len(), 0);
    }

    #[test]
    fn test_message_duration_timeout() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("First", 100, 0, 20); // 100ms duration
        manager.init_message_text("Second", 100, 0, 20);

        // At time 0, first message is showing
        assert!(manager.is_available());

        // At time 150, first message should have expired
        manager.update(150, 20);
        assert!(manager.is_available()); // Second message now

        // At time 300, second message should have expired
        manager.update(300, 20);
        assert!(!manager.is_available());
    }

    #[test]
    fn test_remaining_time() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("Test", 1000, 0, 20);

        assert_eq!(manager.remaining_time(0), 1000);
        assert_eq!(manager.remaining_time(500), 500);
        assert_eq!(manager.remaining_time(1000), 0);
        assert_eq!(manager.remaining_time(1500), 0); // Saturates at 0
    }

    #[test]
    fn test_get_message_string() {
        assert_eq!(get_message_string(DiabloMessage::GameSaved), "Game saved");
        assert_eq!(get_message_string(DiabloMessage::Loading), "Loading...");
        assert_eq!(
            get_message_string(DiabloMessage::ShrineSomeAreWeakened),
            "Some are weakened as one grows strong"
        );
    }

    #[test]
    fn test_shrine_messages() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message(DiabloMessage::ShrineSwordOfJustice, 3500, 0, 20);

        let lines = manager.get_display_lines();
        let text = lines.join(" ");
        assert!(text.contains("sword") || text.contains("justice"));
    }

    #[test]
    fn test_word_wrap() {
        let long_text = "This is a very long message that should be wrapped across multiple lines";
        let wrapped = word_wrap_simple(long_text, 100); // ~12 chars per line

        let lines: Vec<&str> = wrapped.lines().collect();
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_display_state() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("Test", 3500, 0, 20);

        let state = manager.get_display_state();
        assert!(state.outer_height > 0);
        assert!(state.line_width >= MIN_LINE_WIDTH);
        assert_eq!(state.line_height, 20);
    }

    #[test]
    fn test_empty_message() {
        let mut manager = DiabloMsgManager::new();
        manager.init_message_text("", 3500, 0, 20);

        // Empty messages should still be added (matching C++ behavior)
        assert!(manager.is_available());
    }

    #[test]
    fn test_message_entry() {
        let entry = MessageEntry::new("Hello", 5000);
        assert_eq!(entry.text, "Hello");
        assert_eq!(entry.duration_ms, 5000);
    }
}
