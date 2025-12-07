//! UI Item Types - Base UI Component Classes
//!
//! This module defines the base types for UI items used throughout
//! the DiabloUI system. These correspond to the C++ UiItemBase hierarchy.
//!
//! ## C++ Alignment
//!
//! - `ui_item.h` (460 lines)
//! - `ui_flags.hpp` (flags and enums)

use std::fmt;

/// UI rendering flags controlling appearance and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiFlags(u32);

impl UiFlags {
    /// No special flags
    pub const NONE: Self = Self(0);

    // Font size flags (bits 0-2)
    /// Small font (12pt equivalent)
    pub const FONT_SIZE_12: Self = Self(1 << 0);
    /// Medium font (24pt equivalent)
    pub const FONT_SIZE_24: Self = Self(1 << 1);
    /// Large font (30pt equivalent)
    pub const FONT_SIZE_30: Self = Self(1 << 2);
    /// Extra large font (42pt equivalent)
    pub const FONT_SIZE_42: Self = Self(1 << 3);
    /// Huge font (46pt equivalent)
    pub const FONT_SIZE_46: Self = Self(1 << 4);

    // Alignment flags (bits 5-6)
    /// Center text horizontally
    pub const ALIGN_CENTER: Self = Self(1 << 5);
    /// Align text to right
    pub const ALIGN_RIGHT: Self = Self(1 << 6);

    // Color flags (bits 7-14)
    /// Silver/gray text color
    pub const COLOR_SILVER: Self = Self(1 << 7);
    /// Gold/yellow text color
    pub const COLOR_GOLD: Self = Self(1 << 8);
    /// Red text color
    pub const COLOR_RED: Self = Self(1 << 9);
    /// Blue text color
    pub const COLOR_BLUE: Self = Self(1 << 10);
    /// Black text color
    pub const COLOR_BLACK: Self = Self(1 << 11);
    /// White text color
    pub const COLOR_WHITE: Self = Self(1 << 12);
    /// Orange text color
    pub const COLOR_ORANGE: Self = Self(1 << 13);
    /// Yellow text color
    pub const COLOR_YELLOW: Self = Self(1 << 14);

    // Behavior flags (bits 15-22)
    /// Element is hidden
    pub const ELEMENT_HIDDEN: Self = Self(1 << 15);
    /// Element is disabled (not interactive)
    pub const ELEMENT_DISABLED: Self = Self(1 << 16);
    /// Element has pending state
    pub const PENDING: Self = Self(1 << 17);
    /// Text is outlined
    pub const OUTLINED: Self = Self(1 << 18);
    /// Text with shadow
    pub const SHADOWED: Self = Self(1 << 19);
    /// Vertical centering
    pub const VCENTER: Self = Self(1 << 20);
    /// Item needs next element for proper rendering
    pub const NEEDS_NEXT_ELEMENT: Self = Self(1 << 21);
    /// Kerning disabled
    pub const KERNING_DISABLED: Self = Self(1 << 22);

    /// Check if flags contain the specified flag(s)
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if flags have any of the specified flags
    pub fn has_any_of(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Combine flags with OR
    pub fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Remove flags
    pub fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Get raw value
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for UiFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for UiFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for UiFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for UiFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for UiFlags {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// UI item type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UiType {
    /// Static or dynamic text
    Text = 0,
    /// Art-rendered text (special font)
    ArtText = 1,
    /// Art text that acts as a button
    ArtTextButton = 2,
    /// CLX sprite image
    ImageClx = 3,
    /// Animated CLX sprite
    ImageAnimatedClx = 4,
    /// Clickable button
    Button = 5,
    /// Selectable list
    List = 6,
    /// Scrollbar control
    Scrollbar = 7,
    /// Text input/edit field
    Edit = 8,
}

/// Rectangle for UI layout
#[derive(Debug, Clone, Copy, Default)]
pub struct UiRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl UiRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if point is inside rectangle
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    /// Get the center X coordinate
    pub fn center_x(&self) -> i32 {
        self.x + self.width / 2
    }

    /// Get the center Y coordinate
    pub fn center_y(&self) -> i32 {
        self.y + self.height / 2
    }
}

/// Base trait for all UI items
pub trait UiItemBase: fmt::Debug {
    /// Get the UI type
    fn ui_type(&self) -> UiType;

    /// Get the bounding rectangle
    fn rect(&self) -> &UiRect;

    /// Get mutable bounding rectangle
    fn rect_mut(&mut self) -> &mut UiRect;

    /// Get UI flags
    fn flags(&self) -> UiFlags;

    /// Set UI flags
    fn set_flags(&mut self, flags: UiFlags);

    /// Check if item is hidden
    fn is_hidden(&self) -> bool {
        self.flags().contains(UiFlags::ELEMENT_HIDDEN)
    }

    /// Check if item is disabled
    fn is_disabled(&self) -> bool {
        self.flags().contains(UiFlags::ELEMENT_DISABLED)
    }

    /// Check if item is interactive (not hidden and not disabled)
    fn is_interactive(&self) -> bool {
        !self.flags().has_any_of(UiFlags::ELEMENT_HIDDEN | UiFlags::ELEMENT_DISABLED)
    }

    /// Hide the item
    fn hide(&mut self) {
        let flags = self.flags() | UiFlags::ELEMENT_HIDDEN;
        self.set_flags(flags);
    }

    /// Show the item
    fn show(&mut self) {
        let flags = self.flags().remove(UiFlags::ELEMENT_HIDDEN);
        self.set_flags(flags);
    }

    /// Enable the item
    fn enable(&mut self) {
        let flags = self.flags().remove(UiFlags::ELEMENT_DISABLED);
        self.set_flags(flags);
    }

    /// Disable the item
    fn disable(&mut self) {
        let flags = self.flags() | UiFlags::ELEMENT_DISABLED;
        self.set_flags(flags);
    }
}

/// Static text item
#[derive(Debug)]
pub struct UiText {
    rect: UiRect,
    flags: UiFlags,
    pub text: String,
    pub spacing: i32,
    pub line_height: i32,
}

impl UiText {
    pub fn new(text: impl Into<String>, rect: UiRect, flags: UiFlags) -> Self {
        Self {
            rect,
            flags,
            text: text.into(),
            spacing: 1,
            line_height: -1,
        }
    }

    pub fn with_spacing(mut self, spacing: i32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_line_height(mut self, height: i32) -> Self {
        self.line_height = height;
        self
    }
}

impl UiItemBase for UiText {
    fn ui_type(&self) -> UiType {
        UiType::Text
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// Art-rendered text (using game's special font)
#[derive(Debug)]
pub struct UiArtText {
    rect: UiRect,
    flags: UiFlags,
    pub text: String,
    pub spacing: i32,
    pub line_height: i32,
}

impl UiArtText {
    pub fn new(text: impl Into<String>, rect: UiRect, flags: UiFlags) -> Self {
        Self {
            rect,
            flags,
            text: text.into(),
            spacing: 1,
            line_height: -1,
        }
    }

    pub fn with_spacing(mut self, spacing: i32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl UiItemBase for UiArtText {
    fn ui_type(&self) -> UiType {
        UiType::ArtText
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// Art text button (clickable art text)
#[derive(Debug)]
pub struct UiArtTextButton {
    rect: UiRect,
    flags: UiFlags,
    pub text: String,
    pub action_id: u32,
}

impl UiArtTextButton {
    pub fn new(text: impl Into<String>, action_id: u32, rect: UiRect, flags: UiFlags) -> Self {
        Self {
            rect,
            flags,
            text: text.into(),
            action_id,
        }
    }
}

impl UiItemBase for UiArtTextButton {
    fn ui_type(&self) -> UiType {
        UiType::ArtTextButton
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// CLX sprite image
#[derive(Debug)]
pub struct UiImageClx {
    rect: UiRect,
    flags: UiFlags,
    /// Sprite frame index
    pub frame: u16,
    /// Sprite sheet identifier
    pub sprite_sheet_id: u32,
}

impl UiImageClx {
    pub fn new(sprite_sheet_id: u32, frame: u16, rect: UiRect, flags: UiFlags) -> Self {
        Self {
            rect,
            flags,
            frame,
            sprite_sheet_id,
        }
    }

    pub fn is_centered(&self) -> bool {
        self.flags.contains(UiFlags::ALIGN_CENTER)
    }
}

impl UiItemBase for UiImageClx {
    fn ui_type(&self) -> UiType {
        UiType::ImageClx
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// Standard clickable button
#[derive(Debug)]
pub struct UiButton {
    rect: UiRect,
    flags: UiFlags,
    pub text: String,
    pub action_id: u32,
    pub pressed: bool,
}

impl UiButton {
    pub fn new(text: impl Into<String>, action_id: u32, rect: UiRect, flags: UiFlags) -> Self {
        Self {
            rect,
            flags,
            text: text.into(),
            action_id,
            pressed: false,
        }
    }
}

impl UiItemBase for UiButton {
    fn ui_type(&self) -> UiType {
        UiType::Button
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// List item for UiList
#[derive(Debug, Clone)]
pub struct UiListItem {
    pub text: String,
    pub value: i32,
    pub flags: UiFlags,
}

impl UiListItem {
    pub fn new(text: impl Into<String>, value: i32, flags: UiFlags) -> Self {
        Self {
            text: text.into(),
            value,
            flags,
        }
    }

    pub fn is_selectable(&self) -> bool {
        !self
            .flags
            .has_any_of(UiFlags::ELEMENT_HIDDEN | UiFlags::ELEMENT_DISABLED)
    }
}

/// Selectable list control
#[derive(Debug)]
pub struct UiList {
    rect: UiRect,
    flags: UiFlags,
    pub items: Vec<UiListItem>,
    pub selected_index: usize,
    pub viewport_size: usize,
    pub scroll_offset: usize,
    pub item_height: i32,
}

impl UiList {
    pub fn new(
        items: Vec<UiListItem>,
        rect: UiRect,
        flags: UiFlags,
        viewport_size: usize,
    ) -> Self {
        Self {
            rect,
            flags,
            items,
            selected_index: 0,
            viewport_size,
            scroll_offset: 0,
            item_height: 26,
        }
    }

    /// Get item at index
    pub fn get_item(&self, index: usize) -> Option<&UiListItem> {
        self.items.get(index)
    }

    /// Get mutable item at index
    pub fn get_item_mut(&mut self, index: usize) -> Option<&mut UiListItem> {
        self.items.get_mut(index)
    }

    /// Get currently selected item
    pub fn selected_item(&self) -> Option<&UiListItem> {
        self.items.get(self.selected_index)
    }

    /// Get maximum selectable index
    pub fn max_index(&self) -> usize {
        self.items.len().saturating_sub(1)
    }

    /// Check if scrolling is needed
    pub fn needs_scrollbar(&self) -> bool {
        self.items.len() > self.viewport_size
    }

    /// Adjust scroll offset to show selected item
    pub fn adjust_scroll(&mut self) {
        if self.selected_index >= self.scroll_offset + self.viewport_size {
            self.scroll_offset = self.selected_index.saturating_sub(self.viewport_size - 1);
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self, wrap: bool) -> bool {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll();
            true
        } else if wrap && !self.items.is_empty() {
            self.selected_index = self.max_index();
            self.adjust_scroll();
            true
        } else {
            false
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, wrap: bool) -> bool {
        if self.selected_index < self.max_index() {
            self.selected_index += 1;
            self.adjust_scroll();
            true
        } else if wrap && !self.items.is_empty() {
            self.selected_index = 0;
            self.adjust_scroll();
            true
        } else {
            false
        }
    }

    /// Get visible items based on scroll offset
    pub fn visible_items(&self) -> impl Iterator<Item = (usize, &UiListItem)> {
        self.items
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
    }
}

impl UiItemBase for UiList {
    fn ui_type(&self) -> UiType {
        UiType::List
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// Scrollbar control
#[derive(Debug)]
pub struct UiScrollbar {
    rect: UiRect,
    flags: UiFlags,
    pub thumb_position: f32,
    pub thumb_size: f32,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub dragging: bool,
}

impl UiScrollbar {
    pub fn new(rect: UiRect, flags: UiFlags) -> Self {
        Self {
            rect,
            flags,
            thumb_position: 0.0,
            thumb_size: 0.25,
            up_pressed: false,
            down_pressed: false,
            dragging: false,
        }
    }

    /// Update thumb position based on list state
    pub fn update_from_list(&mut self, list: &UiList) {
        if list.items.is_empty() || list.viewport_size >= list.items.len() {
            self.thumb_position = 0.0;
            self.thumb_size = 1.0;
            return;
        }

        let total_items = list.items.len() as f32;
        let viewport = list.viewport_size as f32;

        self.thumb_size = (viewport / total_items).min(1.0).max(0.1);
        self.thumb_position = list.scroll_offset as f32 / (total_items - viewport);
    }

    /// Get thumb rectangle
    pub fn thumb_rect(&self) -> UiRect {
        let track_height = self.rect.height - 48; // Space for arrows
        let thumb_height = (track_height as f32 * self.thumb_size) as i32;
        let thumb_y = self.rect.y + 24 + ((track_height - thumb_height) as f32 * self.thumb_position) as i32;

        UiRect::new(self.rect.x, thumb_y, self.rect.width, thumb_height)
    }
}

impl UiItemBase for UiScrollbar {
    fn ui_type(&self) -> UiType {
        UiType::Scrollbar
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// Text input/edit field
#[derive(Debug)]
pub struct UiEdit {
    rect: UiRect,
    flags: UiFlags,
    pub value: String,
    pub max_length: usize,
    pub cursor_pos: usize,
    pub hint: String,
    pub allow_empty: bool,
}

impl UiEdit {
    pub fn new(
        hint: impl Into<String>,
        initial_value: impl Into<String>,
        max_length: usize,
        rect: UiRect,
        flags: UiFlags,
    ) -> Self {
        let value = initial_value.into();
        let cursor = value.len();
        Self {
            rect,
            flags,
            value,
            max_length,
            cursor_pos: cursor,
            hint: hint.into(),
            allow_empty: false,
        }
    }

    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        if self.value.len() < self.max_length {
            self.value.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
        }
    }

    /// Delete character before cursor (backspace)
    pub fn delete_before(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.value.remove(self.cursor_pos);
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_at(&mut self) {
        if self.cursor_pos < self.value.len() {
            self.value.remove(self.cursor_pos);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        self.cursor_pos = self.cursor_pos.min(self.value.len()).saturating_add(1);
        self.cursor_pos = self.cursor_pos.min(self.value.len());
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.value.len();
    }

    /// Check if current value is valid
    pub fn is_valid(&self) -> bool {
        self.allow_empty || !self.value.is_empty()
    }
}

impl UiItemBase for UiEdit {
    fn ui_type(&self) -> UiType {
        UiType::Edit
    }
    fn rect(&self) -> &UiRect {
        &self.rect
    }
    fn rect_mut(&mut self) -> &mut UiRect {
        &mut self.rect
    }
    fn flags(&self) -> UiFlags {
        self.flags
    }
    fn set_flags(&mut self, flags: UiFlags) {
        self.flags = flags;
    }
}

/// Generic UI item enum for storing different item types
#[derive(Debug)]
pub enum UiItem {
    Text(UiText),
    ArtText(UiArtText),
    ArtTextButton(UiArtTextButton),
    ImageClx(UiImageClx),
    Button(UiButton),
    List(UiList),
    Scrollbar(UiScrollbar),
    Edit(UiEdit),
}

impl UiItem {
    /// Get UI type
    pub fn ui_type(&self) -> UiType {
        match self {
            Self::Text(_) => UiType::Text,
            Self::ArtText(_) => UiType::ArtText,
            Self::ArtTextButton(_) => UiType::ArtTextButton,
            Self::ImageClx(_) => UiType::ImageClx,
            Self::Button(_) => UiType::Button,
            Self::List(_) => UiType::List,
            Self::Scrollbar(_) => UiType::Scrollbar,
            Self::Edit(_) => UiType::Edit,
        }
    }

    /// Get bounding rectangle
    pub fn rect(&self) -> &UiRect {
        match self {
            Self::Text(item) => item.rect(),
            Self::ArtText(item) => item.rect(),
            Self::ArtTextButton(item) => item.rect(),
            Self::ImageClx(item) => item.rect(),
            Self::Button(item) => item.rect(),
            Self::List(item) => item.rect(),
            Self::Scrollbar(item) => item.rect(),
            Self::Edit(item) => item.rect(),
        }
    }

    /// Get flags
    pub fn flags(&self) -> UiFlags {
        match self {
            Self::Text(item) => item.flags(),
            Self::ArtText(item) => item.flags(),
            Self::ArtTextButton(item) => item.flags(),
            Self::ImageClx(item) => item.flags(),
            Self::Button(item) => item.flags(),
            Self::List(item) => item.flags(),
            Self::Scrollbar(item) => item.flags(),
            Self::Edit(item) => item.flags(),
        }
    }

    /// Check if hidden
    pub fn is_hidden(&self) -> bool {
        self.flags().contains(UiFlags::ELEMENT_HIDDEN)
    }

    /// Check if interactive
    pub fn is_interactive(&self) -> bool {
        !self
            .flags()
            .has_any_of(UiFlags::ELEMENT_HIDDEN | UiFlags::ELEMENT_DISABLED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_flags() {
        let flags = UiFlags::ALIGN_CENTER | UiFlags::COLOR_GOLD;
        assert!(flags.contains(UiFlags::ALIGN_CENTER));
        assert!(flags.contains(UiFlags::COLOR_GOLD));
        assert!(!flags.contains(UiFlags::COLOR_RED));
        assert!(flags.has_any_of(UiFlags::ALIGN_CENTER | UiFlags::COLOR_RED));
    }

    #[test]
    fn test_ui_rect() {
        let rect = UiRect::new(10, 20, 100, 50);
        assert!(rect.contains_point(10, 20));
        assert!(rect.contains_point(109, 69));
        assert!(!rect.contains_point(110, 70));
        assert_eq!(rect.center_x(), 60);
        assert_eq!(rect.center_y(), 45);
    }

    #[test]
    fn test_ui_text() {
        let text = UiText::new("Hello", UiRect::new(0, 0, 100, 20), UiFlags::ALIGN_CENTER);
        assert_eq!(text.text, "Hello");
        assert_eq!(text.ui_type(), UiType::Text);
        assert!(!text.is_hidden());
    }

    #[test]
    fn test_ui_list() {
        let items = vec![
            UiListItem::new("Item 1", 1, UiFlags::NONE),
            UiListItem::new("Item 2", 2, UiFlags::NONE),
            UiListItem::new("Item 3", 3, UiFlags::NONE),
        ];
        let mut list = UiList::new(items, UiRect::new(0, 0, 200, 100), UiFlags::NONE, 2);

        assert_eq!(list.max_index(), 2);
        assert!(list.needs_scrollbar());
        assert_eq!(list.selected_index, 0);

        assert!(list.select_next(false));
        assert_eq!(list.selected_index, 1);

        assert!(list.select_next(false));
        assert_eq!(list.selected_index, 2);

        assert!(!list.select_next(false)); // Can't go further without wrap
        assert!(list.select_next(true)); // With wrap, goes to 0
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_ui_edit() {
        let mut edit = UiEdit::new("Enter name", "", 16, UiRect::new(0, 0, 200, 30), UiFlags::NONE);

        edit.insert_char('H');
        edit.insert_char('e');
        edit.insert_char('l');
        edit.insert_char('l');
        edit.insert_char('o');
        assert_eq!(edit.value, "Hello");
        assert_eq!(edit.cursor_pos, 5);

        edit.delete_before();
        assert_eq!(edit.value, "Hell");

        edit.cursor_left();
        edit.cursor_left();
        edit.insert_char('X');
        assert_eq!(edit.value, "HeXll");
    }

    #[test]
    fn test_ui_scrollbar() {
        let items = vec![
            UiListItem::new("Item 1", 1, UiFlags::NONE),
            UiListItem::new("Item 2", 2, UiFlags::NONE),
            UiListItem::new("Item 3", 3, UiFlags::NONE),
            UiListItem::new("Item 4", 4, UiFlags::NONE),
        ];
        let list = UiList::new(items, UiRect::new(0, 0, 200, 100), UiFlags::NONE, 2);

        let mut scrollbar = UiScrollbar::new(UiRect::new(200, 0, 20, 100), UiFlags::NONE);
        scrollbar.update_from_list(&list);

        assert_eq!(scrollbar.thumb_size, 0.5); // 2 visible out of 4
        assert_eq!(scrollbar.thumb_position, 0.0); // At top
    }
}
