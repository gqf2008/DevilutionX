//! Scrolling Dialog Text System (MiniText)
//!
//! This module implements the scrolling quest dialog text system used in Diablo.
//! When NPCs give quest-related speeches, the text scrolls vertically in a
//! decorated window while the voice audio plays.
//!
//! # C++ Source Reference
//! - Source/minitext.cpp
//!
//! # Features
//! - Automatic text wrapping to fit window width
//! - Smooth vertical scrolling synchronized with audio
//! - Decorated text box with border graphics

use crate::game::types::{Point, Rectangle};

/// Pixels for a line of text and the empty space under it
pub const LINE_HEIGHT: i32 = 38;

/// Default text wrap width in pixels
pub const DEFAULT_WRAP_WIDTH: i32 = 543;

/// Number of visible lines in the text window
pub const VISIBLE_LINES: i32 = 8;

/// Vertical offset for text start position
pub const TEXT_START_OFFSET: i32 = 260;

/// Additional lines to show after audio ends
pub const TRAILING_LINES: i32 = 5;

/// Simple size structure
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

/// Speech ID enumeration (simplified from C++ _speech_id)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i16)]
pub enum SpeechId {
    #[default]
    None = -1,
    // Quest speeches
    King1 = 0,
    King2 = 1,
    King3 = 2,
    // ... (many more in full implementation)
    Banner1 = 11,
    Vile1 = 23,
    Poison1 = 37,
    Bone1 = 47,
    Butch1 = 55,
    Blind1 = 65,
    Blood1 = 93,
    Warlord1 = 101,
    // Book speeches
    Book11 = 239,
    Book12 = 240,
    Book13 = 241,
    // Intro
    Intro = 248,
}

/// Sound effect ID (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum SfxId {
    #[default]
    None = 0,
    Warrior1 = 1,
    Warrior10 = 10,
    Warrior11 = 11,
    Warrior12 = 12,
    Warrior54 = 54,
    Warrior55 = 55,
    Warrior56 = 56,
}

/// Hero speech types for class-specific audio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeroSpeech {
    ChamberOfBoneLore,
    ValorLore,
    HallsOfTheBlindLore,
    WarlordOfBloodLore,
    InSpirituSanctum,
    PraedictumOtium,
    EfficioObitusUtInimicus,
}

/// Speech data entry
#[derive(Debug, Clone, Default)]
pub struct Speech {
    /// The text string to display
    pub text: String,
    /// Whether to show scrolling text
    pub scroll_text: bool,
    /// Associated sound effect
    pub sfx_id: SfxId,
}

/// Scrolling quest text state
///
/// Manages the state of the scrolling dialog text display,
/// including text content, scroll position, and timing.
#[derive(Debug, Clone)]
pub struct QuestTextState {
    /// Whether quest text is currently being displayed
    pub active: bool,
    /// Scrolling speed in milliseconds per pixel
    pub speed_ms_per_px: u32,
    /// Timestamp when scrolling started (milliseconds since startup)
    pub start_time: u32,
    /// Text lines to display (after word wrapping)
    pub lines: Vec<String>,
    /// Text box graphics loaded flag
    pub graphics_loaded: bool,
}

impl Default for QuestTextState {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestTextState {
    /// Creates a new quest text state
    pub fn new() -> Self {
        Self {
            active: false,
            speed_ms_per_px: 0,
            start_time: 0,
            lines: Vec::new(),
            graphics_loaded: false,
        }
    }

    /// Resets the state
    pub fn reset(&mut self) {
        self.active = false;
        self.speed_ms_per_px = 0;
        self.start_time = 0;
        self.lines.clear();
    }

    /// Loads and word-wraps text for display
    ///
    /// # Arguments
    /// * `text` - The text to display
    /// * `max_width` - Maximum width in pixels for word wrapping
    ///
    /// # C++ Reference
    /// ```cpp
    /// void LoadText(std::string_view text)
    /// {
    ///     TextLines.clear();
    ///     const std::string paragraphs = WordWrapString(text, 543, GameFont30);
    ///     // Split by newlines...
    /// }
    /// ```
    pub fn load_text(&mut self, text: &str, max_width: i32) {
        self.lines.clear();

        // Simple word wrapping implementation
        // In real implementation, this would use font metrics
        let wrapped = word_wrap_string(text, max_width);

        for line in wrapped.lines() {
            self.lines.push(line.to_string());
        }
    }

    /// Starts the scrolling animation
    ///
    /// # Arguments
    /// * `speed` - Scroll speed in ms/pixel
    /// * `current_time` - Current timestamp in milliseconds
    pub fn start(&mut self, speed: u32, current_time: u32) {
        self.active = true;
        self.speed_ms_per_px = speed;
        self.start_time = current_time;
    }

    /// Stops the scrolling animation
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Calculates the current scroll position
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp in milliseconds
    ///
    /// # Returns
    /// The Y offset in pixels (positive = scrolled up)
    ///
    /// # C++ Reference
    /// ```cpp
    /// int CalculateTextPosition()
    /// {
    ///     const uint32_t currTime = GetMillisecondsSinceStartup();
    ///     const int y = (currTime - ScrollStart) / qtextSpd - 260;
    ///     // ...
    /// }
    /// ```
    pub fn calculate_position(&self, current_time: u32) -> i32 {
        if self.speed_ms_per_px == 0 {
            return 0;
        }

        let elapsed = current_time.saturating_sub(self.start_time);
        let y = (elapsed / self.speed_ms_per_px) as i32 - TEXT_START_OFFSET;
        y
    }

    /// Checks if the text has finished scrolling
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp in milliseconds
    ///
    /// # Returns
    /// `true` if all text has scrolled past
    pub fn is_finished(&self, current_time: u32) -> bool {
        let y = self.calculate_position(current_time);
        let text_height = LINE_HEIGHT * self.lines.len() as i32;
        y >= text_height
    }

    /// Updates state and returns whether still active
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp in milliseconds
    pub fn update(&mut self, current_time: u32) -> bool {
        if self.active && self.is_finished(current_time) {
            self.active = false;
        }
        self.active
    }

    /// Gets the lines that should be visible at current scroll position
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// Iterator of (line_index, y_position, line_text)
    pub fn visible_lines(&self, current_time: u32) -> Vec<(usize, i32, &str)> {
        let y = self.calculate_position(current_time);
        let skip_lines = (y / LINE_HEIGHT).max(0) as usize;
        let sy = -(y % LINE_HEIGHT);

        let mut result = Vec::new();
        for i in 0..VISIBLE_LINES as usize {
            let line_number = skip_lines + i;
            if line_number >= self.lines.len() {
                continue;
            }
            let line = &self.lines[line_number];
            if !line.is_empty() {
                result.push((line_number, sy + (i as i32) * LINE_HEIGHT, line.as_str()));
            }
        }
        result
    }
}

/// Calculates scroll speed to match audio duration
///
/// # Arguments
/// * `num_lines` - Number of text lines
/// * `audio_duration_ms` - Duration of audio in milliseconds
///
/// # Returns
/// Scroll speed in ms/pixel
///
/// # C++ Reference
/// ```cpp
/// uint32_t CalculateTextSpeed(SfxID nSFX)
/// {
///     uint32_t sfxFrames = GetSFXLength(nSFX);
///     uint32_t textHeight = LineHeight * numLines;
///     textHeight += LineHeight * 5; // adjust for trailing
///     return sfxFrames / textHeight;
/// }
/// ```
pub fn calculate_text_speed(num_lines: u32, audio_duration_ms: u32) -> u32 {
    if num_lines == 0 || audio_duration_ms == 0 {
        return 1; // Prevent division by zero
    }

    let text_height = LINE_HEIGHT as u32 * num_lines;
    // Add trailing lines adjustment
    let adjusted_height = text_height + (LINE_HEIGHT as u32 * TRAILING_LINES as u32);

    if adjusted_height == 0 {
        return 1;
    }

    audio_duration_ms / adjusted_height
}

/// Simple word wrapping implementation
///
/// Wraps text to fit within the specified pixel width.
/// This is a simplified version - real implementation would use font metrics.
///
/// # Arguments
/// * `text` - The text to wrap
/// * `max_width` - Maximum width in pixels
///
/// # Returns
/// Word-wrapped string with newlines
fn word_wrap_string(text: &str, max_width: i32) -> String {
    // Approximate characters per line based on average character width
    // For GameFont30, assume ~12 pixels per character
    let chars_per_line = (max_width / 12).max(1) as usize;

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

/// Text box dimensions
#[derive(Debug, Clone, Copy)]
pub struct TextBoxLayout {
    /// Position of the text box
    pub position: Point,
    /// Size of the text box
    pub size: Size,
    /// Text area within the box
    pub text_area: Rectangle,
    /// Text start X offset
    pub text_x: i32,
}

impl TextBoxLayout {
    /// Creates default text box layout
    ///
    /// # Arguments
    /// * `ui_rect` - The UI rectangle position
    pub fn new(ui_position: Point) -> Self {
        Self {
            position: Point {
                x: ui_position.x + 24,
                y: ui_position.y + 327,
            },
            size: Size {
                width: 591,
                height: 297,
            },
            text_area: Rectangle {
                position: Point {
                    x: ui_position.x + 27,
                    y: ui_position.y + 28,
                },
                width: 585,
                height: 297,
            },
            text_x: ui_position.x + 48,
        }
    }
}

/// Quest text manager for handling speech display
#[derive(Debug, Clone, Default)]
pub struct QuestTextManager {
    /// Current quest text state
    pub state: QuestTextState,
    /// Layout information
    pub layout: Option<TextBoxLayout>,
}

impl QuestTextManager {
    /// Creates a new quest text manager
    pub fn new() -> Self {
        Self {
            state: QuestTextState::new(),
            layout: None,
        }
    }

    /// Initializes the quest text system
    ///
    /// # Arguments
    /// * `ui_position` - The UI rectangle position
    pub fn init(&mut self, ui_position: Point) {
        self.layout = Some(TextBoxLayout::new(ui_position));
        self.state.graphics_loaded = true;
    }

    /// Frees quest text resources
    pub fn free(&mut self) {
        self.state.graphics_loaded = false;
        self.layout = None;
    }

    /// Starts displaying a quest message
    ///
    /// # Arguments
    /// * `text` - The text to display
    /// * `audio_duration_ms` - Duration of audio in milliseconds
    /// * `current_time` - Current timestamp
    pub fn start_message(&mut self, text: &str, audio_duration_ms: u32, current_time: u32) {
        self.state.load_text(text, DEFAULT_WRAP_WIDTH);
        let speed = calculate_text_speed(self.state.lines.len() as u32, audio_duration_ms);
        self.state.start(speed, current_time);
    }

    /// Updates the quest text state
    ///
    /// # Arguments
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// `true` if quest text is still active
    pub fn update(&mut self, current_time: u32) -> bool {
        self.state.update(current_time)
    }

    /// Checks if quest text is currently active
    pub fn is_active(&self) -> bool {
        self.state.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = QuestTextState::new();
        assert!(!state.active);
        assert_eq!(state.speed_ms_per_px, 0);
        assert_eq!(state.start_time, 0);
        assert!(state.lines.is_empty());
    }

    #[test]
    fn test_load_text_single_line() {
        let mut state = QuestTextState::new();
        state.load_text("Hello world", 1000);
        assert_eq!(state.lines.len(), 1);
        assert_eq!(state.lines[0], "Hello world");
    }

    #[test]
    fn test_load_text_wrap() {
        let mut state = QuestTextState::new();
        // With max_width of 120 pixels (~10 chars), this should wrap
        let long_text = "This is a longer text that should be wrapped across multiple lines";
        state.load_text(long_text, 120);
        assert!(state.lines.len() > 1);
    }

    #[test]
    fn test_start_stop() {
        let mut state = QuestTextState::new();
        state.load_text("Test", 500);

        state.start(10, 1000);
        assert!(state.active);
        assert_eq!(state.speed_ms_per_px, 10);
        assert_eq!(state.start_time, 1000);

        state.stop();
        assert!(!state.active);
    }

    #[test]
    fn test_calculate_position() {
        let mut state = QuestTextState::new();
        state.load_text("Test", 500);
        state.start(10, 1000); // 10 ms per pixel

        // At start time + 2600ms = 260px scrolled, minus 260 offset = 0
        let pos = state.calculate_position(3600);
        assert_eq!(pos, 0);

        // At start time + 3600ms = 360px scrolled, minus 260 offset = 100
        let pos = state.calculate_position(4600);
        assert_eq!(pos, 100);
    }

    #[test]
    fn test_is_finished() {
        let mut state = QuestTextState::new();
        state.load_text("Line 1\nLine 2", 500);
        state.start(1, 0); // 1 ms per pixel (fast)

        // With 2 lines, text height = 2 * 38 = 76
        // Need to scroll 260 + 76 = 336 pixels
        // At 1ms/px, need 336ms
        assert!(!state.is_finished(100));
        assert!(state.is_finished(400)); // Well past finish
    }

    #[test]
    fn test_calculate_text_speed() {
        // 10 lines, 3000ms audio
        let num_lines = 10;
        let audio_ms = 3000;

        let speed = calculate_text_speed(num_lines, audio_ms);

        // text_height = 38 * 10 = 380
        // adjusted = 380 + 38 * 5 = 570
        // speed = 3000 / 570 = 5
        assert_eq!(speed, 5);
    }

    #[test]
    fn test_calculate_text_speed_edge_cases() {
        // Zero lines
        assert_eq!(calculate_text_speed(0, 3000), 1);

        // Zero audio
        assert_eq!(calculate_text_speed(10, 0), 1);
    }

    #[test]
    fn test_word_wrap() {
        let text = "This is a test of the word wrap function";
        let wrapped = word_wrap_string(text, 120); // ~10 chars per line

        // Should have multiple lines
        let lines: Vec<&str> = wrapped.lines().collect();
        assert!(lines.len() > 1);

        // No line should be too long (approximate check)
        for line in lines {
            assert!(line.len() <= 15); // Some tolerance
        }
    }

    #[test]
    fn test_visible_lines() {
        let mut state = QuestTextState::new();
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10";
        state.lines = text.lines().map(String::from).collect();
        state.start(1, 0);

        // At time where y = 0 (fully at top)
        let visible = state.visible_lines(TEXT_START_OFFSET as u32);
        assert!(!visible.is_empty());
        assert!(visible.len() <= VISIBLE_LINES as usize);
    }

    #[test]
    fn test_reset() {
        let mut state = QuestTextState::new();
        state.load_text("Test", 500);
        state.start(10, 1000);

        state.reset();

        assert!(!state.active);
        assert_eq!(state.speed_ms_per_px, 0);
        assert_eq!(state.start_time, 0);
        assert!(state.lines.is_empty());
    }

    #[test]
    fn test_text_box_layout() {
        let ui_pos = Point { x: 0, y: 0 };
        let layout = TextBoxLayout::new(ui_pos);

        assert_eq!(layout.position.x, 24);
        assert_eq!(layout.position.y, 327);
        assert_eq!(layout.size.width, 591);
        assert_eq!(layout.text_x, 48);
    }

    #[test]
    fn test_quest_text_manager() {
        let mut manager = QuestTextManager::new();

        // Initialize
        manager.init(Point { x: 0, y: 0 });
        assert!(manager.state.graphics_loaded);
        assert!(manager.layout.is_some());

        // Start message
        manager.start_message("Test message", 3000, 0);
        assert!(manager.is_active());

        // Update past finish
        assert!(!manager.update(10000));
        assert!(!manager.is_active());
    }

    #[test]
    fn test_update_auto_stop() {
        let mut state = QuestTextState::new();
        state.load_text("Short", 500);
        state.start(1, 0); // Very fast

        // Should auto-stop when finished
        let still_active = state.update(10000);
        assert!(!still_active);
        assert!(!state.active);
    }
}
