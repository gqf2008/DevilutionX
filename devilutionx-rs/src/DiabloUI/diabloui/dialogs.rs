//! Dialog Boxes - Confirmation, Progress, and Message Dialogs
//!
//! This module provides various dialog types used throughout the game:
//! - OK dialogs (single button confirmation)
//! - Yes/No dialogs (two-choice confirmation)
//! - Progress dialogs (loading/saving progress)
//! - Select dialogs (multi-option selection)
//!
//! ## C++ Alignment
//!
//! - `DiabloUI/dialogs.cpp` (120 lines)
//! - `DiabloUI/selok.cpp` (selok dialog)
//! - `DiabloUI/selyesno.cpp` (yes/no dialog)
//! - `DiabloUI/progress.cpp` (progress dialog)

use super::ui_core::{UiContext, UiEvent, UiEventResult, UiKeyCode};
use super::ui_item::{UiArtText, UiArtTextButton, UiButton, UiFlags, UiItem, UiItemBase, UiRect, UiText};

/// Dialog result codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    /// No result yet / dialog in progress
    None,
    /// User confirmed / pressed OK
    Ok,
    /// User selected Yes
    Yes,
    /// User selected No
    No,
    /// User cancelled / pressed Escape
    Cancel,
    /// User selected a specific index
    Selected(usize),
}

/// Type of dialog box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Simple OK dialog
    Ok,
    /// Yes/No confirmation dialog
    YesNo,
    /// Multi-option selection dialog
    Select,
    /// Progress bar dialog
    Progress,
    /// Error dialog (OK with error styling)
    Error,
}

/// Action button IDs for dialogs
const ACTION_OK: u32 = 1;
const ACTION_YES: u32 = 2;
const ACTION_NO: u32 = 3;
const ACTION_CANCEL: u32 = 4;

/// Base dialog structure
#[derive(Debug)]
pub struct Dialog {
    /// Dialog type
    pub dialog_type: DialogType,
    /// Title text
    pub title: String,
    /// Message/body text
    pub message: String,
    /// Current result
    pub result: DialogResult,
    /// UI items for the dialog
    pub items: Vec<UiItem>,
    /// Whether dialog is active
    pub active: bool,
}

impl Dialog {
    /// Create a new OK dialog
    pub fn ok(title: impl Into<String>, message: impl Into<String>) -> Self {
        let title = title.into();
        let message = message.into();

        let mut dialog = Self {
            dialog_type: DialogType::Ok,
            title: title.clone(),
            message: message.clone(),
            result: DialogResult::None,
            items: Vec::new(),
            active: true,
        };

        dialog.build_ok_items(&title, &message);
        dialog
    }

    /// Create a new Yes/No dialog
    pub fn yes_no(title: impl Into<String>, message: impl Into<String>) -> Self {
        let title = title.into();
        let message = message.into();

        let mut dialog = Self {
            dialog_type: DialogType::YesNo,
            title: title.clone(),
            message: message.clone(),
            result: DialogResult::None,
            items: Vec::new(),
            active: true,
        };

        dialog.build_yesno_items(&title, &message);
        dialog
    }

    /// Create a new error dialog
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        let title = title.into();
        let message = message.into();

        let mut dialog = Self {
            dialog_type: DialogType::Error,
            title: title.clone(),
            message: message.clone(),
            result: DialogResult::None,
            items: Vec::new(),
            active: true,
        };

        dialog.build_error_items(&title, &message);
        dialog
    }

    fn build_ok_items(&mut self, title: &str, message: &str) {
        // Background would be added by renderer

        // Title
        self.items.push(UiItem::ArtText(UiArtText::new(
            title,
            UiRect::new(140, 197, 360, 37),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));

        // Message
        self.items.push(UiItem::Text(UiText::new(
            message,
            UiRect::new(140, 256, 360, 40),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_12,
        )));

        // OK Button
        self.items.push(UiItem::ArtTextButton(UiArtTextButton::new(
            "OK",
            ACTION_OK,
            UiRect::new(230, 390, 180, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));
    }

    fn build_yesno_items(&mut self, title: &str, message: &str) {
        // Title
        self.items.push(UiItem::ArtText(UiArtText::new(
            title,
            UiRect::new(140, 197, 360, 37),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));

        // Message
        self.items.push(UiItem::Text(UiText::new(
            message,
            UiRect::new(140, 256, 360, 40),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_12,
        )));

        // Yes Button
        self.items.push(UiItem::ArtTextButton(UiArtTextButton::new(
            "Yes",
            ACTION_YES,
            UiRect::new(140, 390, 140, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));

        // No Button
        self.items.push(UiItem::ArtTextButton(UiArtTextButton::new(
            "No",
            ACTION_NO,
            UiRect::new(360, 390, 140, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));
    }

    fn build_error_items(&mut self, title: &str, message: &str) {
        // Title with red color
        self.items.push(UiItem::ArtText(UiArtText::new(
            title,
            UiRect::new(140, 197, 360, 37),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_RED,
        )));

        // Message
        self.items.push(UiItem::Text(UiText::new(
            message,
            UiRect::new(140, 256, 360, 60),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_12,
        )));

        // OK Button
        self.items.push(UiItem::ArtTextButton(UiArtTextButton::new(
            "OK",
            ACTION_OK,
            UiRect::new(230, 390, 180, 35),
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30,
        )));
    }

    /// Handle dialog event
    pub fn handle_event(&mut self, event: &UiEvent) -> DialogResult {
        match event {
            UiEvent::KeyDown { keycode, .. } => match keycode {
                UiKeyCode::Return | UiKeyCode::Space => {
                    self.result = match self.dialog_type {
                        DialogType::Ok | DialogType::Error => DialogResult::Ok,
                        DialogType::YesNo => DialogResult::Yes,
                        _ => DialogResult::None,
                    };
                    self.active = false;
                    self.result
                }
                UiKeyCode::Escape => {
                    self.result = match self.dialog_type {
                        DialogType::YesNo => DialogResult::No,
                        _ => DialogResult::Cancel,
                    };
                    self.active = false;
                    self.result
                }
                _ => DialogResult::None,
            },
            UiEvent::MouseDown { x, y, .. } => {
                for item in &self.items {
                    if let UiItem::ArtTextButton(btn) = item {
                        if item.rect().contains_point(*x, *y) {
                            self.result = match btn.action_id {
                                ACTION_OK => DialogResult::Ok,
                                ACTION_YES => DialogResult::Yes,
                                ACTION_NO => DialogResult::No,
                                ACTION_CANCEL => DialogResult::Cancel,
                                _ => DialogResult::None,
                            };
                            if self.result != DialogResult::None {
                                self.active = false;
                            }
                            return self.result;
                        }
                    }
                    if let UiItem::Button(btn) = item {
                        if item.rect().contains_point(*x, *y) {
                            self.result = match btn.action_id {
                                ACTION_OK => DialogResult::Ok,
                                ACTION_YES => DialogResult::Yes,
                                ACTION_NO => DialogResult::No,
                                ACTION_CANCEL => DialogResult::Cancel,
                                _ => DialogResult::None,
                            };
                            if self.result != DialogResult::None {
                                self.active = false;
                            }
                            return self.result;
                        }
                    }
                }
                DialogResult::None
            }
            _ => DialogResult::None,
        }
    }

    /// Check if dialog is still active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get dialog result
    pub fn get_result(&self) -> DialogResult {
        self.result
    }
}

/// Progress dialog for loading/saving operations
#[derive(Debug)]
pub struct ProgressDialog {
    /// Title text
    pub title: String,
    /// Current progress (0-100)
    pub progress: i32,
    /// Progress bar rect
    pub bar_rect: UiRect,
    /// Whether operation is complete
    pub complete: bool,
    /// Whether operation was cancelled
    pub cancelled: bool,
}

impl ProgressDialog {
    /// Create a new progress dialog
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            progress: 0,
            bar_rect: UiRect::new(160, 240, 320, 24),
            complete: false,
            cancelled: false,
        }
    }

    /// Update progress (0-100)
    pub fn set_progress(&mut self, progress: i32) {
        self.progress = progress.clamp(0, 100);
        if self.progress >= 100 {
            self.complete = true;
        }
    }

    /// Increment progress
    pub fn increment(&mut self, amount: i32) {
        self.set_progress(self.progress + amount);
    }

    /// Get the filled portion of the progress bar
    pub fn filled_rect(&self) -> UiRect {
        let width = (self.bar_rect.width * self.progress) / 100;
        UiRect::new(
            self.bar_rect.x,
            self.bar_rect.y,
            width,
            self.bar_rect.height,
        )
    }

    /// Handle event (allow cancellation with Escape)
    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        if let UiEvent::KeyDown { keycode, .. } = event {
            if *keycode == UiKeyCode::Escape {
                self.cancelled = true;
                return true;
            }
        }
        false
    }

    /// Check if dialog is done (complete or cancelled)
    pub fn is_done(&self) -> bool {
        self.complete || self.cancelled
    }
}

/// Select dialog with multiple options
#[derive(Debug)]
pub struct SelectDialog {
    /// Title text
    pub title: String,
    /// Option labels
    pub options: Vec<String>,
    /// Currently selected option
    pub selected_index: usize,
    /// Dialog result
    pub result: DialogResult,
    /// UI context for navigation
    ctx: UiContext,
}

impl SelectDialog {
    /// Create a new select dialog
    pub fn new(title: impl Into<String>, options: Vec<String>) -> Self {
        let max_index = options.len().saturating_sub(1);
        let mut ctx = UiContext::new();
        ctx.init_list(Vec::new(), max_index, options.len().min(8), false, 0);

        Self {
            title: title.into(),
            options,
            selected_index: 0,
            result: DialogResult::None,
            ctx,
        }
    }

    /// Handle event
    pub fn handle_event(&mut self, event: &UiEvent) -> DialogResult {
        match self.ctx.handle_event(event) {
            UiEventResult::Selected(index) => {
                self.result = DialogResult::Selected(index);
                self.result
            }
            UiEventResult::Escape => {
                self.result = DialogResult::Cancel;
                self.result
            }
            UiEventResult::Handled => {
                // Update selected index from context
                if let Some(state) = &self.ctx.list_state {
                    self.selected_index = state.selected_index;
                }
                DialogResult::None
            }
            _ => DialogResult::None,
        }
    }

    /// Get current selection
    pub fn selection(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|s| s.as_str())
    }

    /// Check if dialog is done
    pub fn is_done(&self) -> bool {
        self.result != DialogResult::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_dialog() {
        let mut dialog = Dialog::ok("Test Title", "Test message");
        assert!(dialog.is_active());
        assert_eq!(dialog.get_result(), DialogResult::None);

        // Press Enter
        let event = UiEvent::KeyDown {
            keycode: UiKeyCode::Return,
            character: None,
        };
        let result = dialog.handle_event(&event);
        assert_eq!(result, DialogResult::Ok);
        assert!(!dialog.is_active());
    }

    #[test]
    fn test_yesno_dialog() {
        let mut dialog = Dialog::yes_no("Confirm", "Are you sure?");
        assert!(dialog.is_active());

        // Press Escape (No)
        let event = UiEvent::KeyDown {
            keycode: UiKeyCode::Escape,
            character: None,
        };
        let result = dialog.handle_event(&event);
        assert_eq!(result, DialogResult::No);
    }

    #[test]
    fn test_progress_dialog() {
        let mut progress = ProgressDialog::new("Loading...");
        assert_eq!(progress.progress, 0);
        assert!(!progress.complete);

        progress.set_progress(50);
        assert_eq!(progress.progress, 50);
        assert!(!progress.complete);

        let filled = progress.filled_rect();
        assert_eq!(filled.width, 160); // 50% of 320

        progress.set_progress(100);
        assert!(progress.complete);
        assert!(progress.is_done());
    }

    #[test]
    fn test_select_dialog() {
        let options = vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ];
        let mut dialog = SelectDialog::new("Select Option", options);

        assert_eq!(dialog.selected_index, 0);
        assert_eq!(dialog.selection(), Some("Option 1"));

        // Navigate down
        let event = UiEvent::KeyDown {
            keycode: UiKeyCode::Down,
            character: None,
        };
        dialog.handle_event(&event);
        assert_eq!(dialog.selected_index, 1);
    }

    #[test]
    fn test_error_dialog() {
        let dialog = Dialog::error("Error!", "Something went wrong");
        assert_eq!(dialog.dialog_type, DialogType::Error);
        assert!(dialog.items.iter().any(|item| {
            if let UiItem::ArtText(text) = item {
                text.flags().contains(UiFlags::COLOR_RED)
            } else {
                false
            }
        }));
    }
}
