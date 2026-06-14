//! Headless Mode - Testing mode without UI
//!
//! C++ Reference: Source/headless_mode.cpp, Source/headless_mode.hpp
//!
//! Provides a flag to run without UI for unit tests.

use std::sync::atomic::{AtomicBool, Ordering};

/// Don't load UI or show Messageboxes or other user-interaction.
/// Needed for unit tests.
///
/// C++ Reference: `HeadlessMode`
static HEADLESS_MODE: AtomicBool = AtomicBool::new(false);

/// Check if headless mode is enabled
pub fn is_headless_mode() -> bool {
    HEADLESS_MODE.load(Ordering::SeqCst)
}

/// Set headless mode
pub fn set_headless_mode(value: bool) {
    HEADLESS_MODE.store(value, Ordering::SeqCst);
}

/// Enable headless mode
pub fn enable_headless_mode() {
    set_headless_mode(true);
}

/// Disable headless mode
pub fn disable_headless_mode() {
    set_headless_mode(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_headless_mode() {
        // Store original value
        let original = is_headless_mode();
        
        set_headless_mode(true);
        assert!(is_headless_mode());
        
        set_headless_mode(false);
        assert!(!is_headless_mode());
        
        // Restore original value
        set_headless_mode(original);
    }
}
