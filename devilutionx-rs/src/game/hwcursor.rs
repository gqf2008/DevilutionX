//! Hardware Cursor - SDL2/SDL3 hardware cursor support
//!
//! C++ Reference: Source/hwcursor.cpp, Source/hwcursor.hpp
//!
//! Provides hardware cursor support for SDL2/SDL3.

use std::sync::Mutex;

/// Cursor type
///
/// C++ Reference: `CursorType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CursorType {
    #[default]
    Unknown = 0,
    UserInterface = 1,
    Game = 2,
}

/// Cursor information
///
/// C++ Reference: `CursorInfo`
#[derive(Debug, Clone, Default)]
pub struct CursorInfo {
    /// Type of cursor
    cursor_type: CursorType,
    /// ID for Game cursor type
    id: i32,
    /// Whether hardware cursor is enabled
    enabled: bool,
    /// Whether cursor needs reinitialization
    needs_reinitialization: bool,
}

impl CursorInfo {
    /// Create a new cursor info
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a user interface cursor
    pub fn user_interface_cursor() -> Self {
        Self {
            cursor_type: CursorType::UserInterface,
            id: 0,
            enabled: false,
            needs_reinitialization: false,
        }
    }
    
    /// Create a game cursor with a sprite ID
    pub fn game_cursor(game_sprite_id: i32) -> Self {
        Self {
            cursor_type: CursorType::Game,
            id: game_sprite_id,
            enabled: false,
            needs_reinitialization: false,
        }
    }
    
    /// Create an unknown cursor
    pub fn unknown_cursor() -> Self {
        Self {
            cursor_type: CursorType::Unknown,
            id: 0,
            enabled: false,
            needs_reinitialization: false,
        }
    }
    
    /// Get cursor type
    pub fn cursor_type(&self) -> CursorType {
        self.cursor_type
    }
    
    /// Get cursor ID (for game cursors)
    pub fn id(&self) -> i32 {
        self.id
    }
    
    /// Check if hardware cursor is enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set enabled state
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }
    
    /// Check if cursor needs reinitialization
    pub fn needs_reinitialization(&self) -> bool {
        self.needs_reinitialization
    }
    
    /// Set needs reinitialization flag
    pub fn set_needs_reinitialization(&mut self, value: bool) {
        self.needs_reinitialization = value;
    }
}

impl PartialEq for CursorInfo {
    fn eq(&self, other: &Self) -> bool {
        self.cursor_type == other.cursor_type
            && (self.cursor_type != CursorType::Game || self.id == other.id)
    }
}

/// Current cursor info
static CURRENT_CURSOR_INFO: Mutex<CursorInfo> = Mutex::new(CursorInfo {
    cursor_type: CursorType::Unknown,
    id: 0,
    enabled: false,
    needs_reinitialization: false,
});

/// Get the current cursor info
pub fn get_current_cursor_info() -> CursorInfo {
    CURRENT_CURSOR_INFO.lock().unwrap().clone()
}

/// Set the current cursor info
pub fn set_current_cursor_info(info: CursorInfo) {
    *CURRENT_CURSOR_INFO.lock().unwrap() = info;
}

/// Check if hardware cursor is enabled in settings
///
/// Note: This always returns false until SDL2 integration is complete
pub fn is_hardware_cursor_enabled() -> bool {
    // TODO: Check options and SDL version
    // return *GetOptions().Graphics.hardwareCursor && HardwareCursorSupported();
    false
}

/// Check if current cursor is a hardware cursor
pub fn is_hardware_cursor() -> bool {
    CURRENT_CURSOR_INFO.lock().unwrap().enabled()
}

/// Set the hardware cursor
///
/// # Arguments
///
/// * `cursor_info` - The cursor info to set
pub fn set_hardware_cursor(mut cursor_info: CursorInfo) {
    cursor_info.set_needs_reinitialization(false);
    
    match cursor_info.cursor_type() {
        CursorType::Game => {
            // TODO: SetHardwareCursorFromSprite
            cursor_info.set_enabled(false);
        }
        CursorType::UserInterface => {
            // TODO: SetHardwareCursorFromClxSprite with ArtCursor
            cursor_info.set_enabled(false);
        }
        CursorType::Unknown => {
            cursor_info.set_enabled(false);
        }
    }
    
    if !cursor_info.enabled() {
        set_hardware_cursor_visible(false);
    }
    
    *CURRENT_CURSOR_INFO.lock().unwrap() = cursor_info;
}

/// Check if hardware cursor is visible
pub fn is_hardware_cursor_visible() -> bool {
    // TODO: SDL_CursorVisible()
    false
}

/// Set hardware cursor visibility
pub fn set_hardware_cursor_visible(visible: bool) {
    if is_hardware_cursor_visible() == visible {
        return;
    }
    
    if visible {
        let info = CURRENT_CURSOR_INFO.lock().unwrap();
        if info.needs_reinitialization() {
            drop(info);
            do_reinitialize_hardware_cursor();
        }
    }
    
    // TODO: SDLC_ShowCursor() / SDLC_HideCursor()
}

/// Reinitialize the hardware cursor
pub fn do_reinitialize_hardware_cursor() {
    let info = get_current_cursor_info();
    set_hardware_cursor(info);
}

/// Request hardware cursor reinitialization
pub fn reinitialize_hardware_cursor() {
    if is_hardware_cursor_visible() {
        do_reinitialize_hardware_cursor();
    } else {
        CURRENT_CURSOR_INFO.lock().unwrap().set_needs_reinitialization(true);
    }
}

/// Hotpoint position for cursor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HotpointPosition {
    #[default]
    TopLeft,
    Center,
}

/// Hardware cursor manager
///
/// Manages SDL2/SDL3 hardware cursor lifecycle
#[derive(Debug, Default)]
pub struct HardwareCursorManager {
    /// Current cursor info
    current: CursorInfo,
    /// Whether the cursor is visible
    visible: bool,
}

impl HardwareCursorManager {
    /// Create a new hardware cursor manager
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get current cursor info
    pub fn current(&self) -> &CursorInfo {
        &self.current
    }
    
    /// Check if cursor is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Set cursor visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    /// Set the cursor
    pub fn set_cursor(&mut self, info: CursorInfo) {
        self.current = info;
    }
    
    /// Check if hardware cursor is enabled
    pub fn is_enabled(&self) -> bool {
        self.current.enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cursor_info_creation() {
        let ui = CursorInfo::user_interface_cursor();
        assert_eq!(ui.cursor_type(), CursorType::UserInterface);
        assert!(!ui.enabled());
        
        let game = CursorInfo::game_cursor(5);
        assert_eq!(game.cursor_type(), CursorType::Game);
        assert_eq!(game.id(), 5);
        
        let unknown = CursorInfo::unknown_cursor();
        assert_eq!(unknown.cursor_type(), CursorType::Unknown);
    }
    
    #[test]
    fn test_cursor_info_equality() {
        let game1 = CursorInfo::game_cursor(5);
        let game2 = CursorInfo::game_cursor(5);
        let game3 = CursorInfo::game_cursor(10);
        
        assert_eq!(game1, game2);
        assert_ne!(game1, game3);
        
        let ui1 = CursorInfo::user_interface_cursor();
        let ui2 = CursorInfo::user_interface_cursor();
        assert_eq!(ui1, ui2);
        
        assert_ne!(game1, ui1);
    }
    
    #[test]
    fn test_cursor_enabled_state() {
        let mut cursor = CursorInfo::game_cursor(1);
        assert!(!cursor.enabled());
        
        cursor.set_enabled(true);
        assert!(cursor.enabled());
        
        cursor.set_enabled(false);
        assert!(!cursor.enabled());
    }
    
    #[test]
    fn test_cursor_reinitialization() {
        let mut cursor = CursorInfo::game_cursor(1);
        assert!(!cursor.needs_reinitialization());
        
        cursor.set_needs_reinitialization(true);
        assert!(cursor.needs_reinitialization());
        
        cursor.set_needs_reinitialization(false);
        assert!(!cursor.needs_reinitialization());
    }
}
