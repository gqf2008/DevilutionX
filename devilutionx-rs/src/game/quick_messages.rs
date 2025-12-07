//! Quick Messages System
//!
//! This module implements the quick chat message system for multiplayer games.
//! Players can send predefined messages quickly using hotkeys (F1-F10).
//!
//! # C++ Source Reference
//! - Source/quick_messages.cpp
//! - Source/quick_messages.hpp
//!
//! # Features
//! - 10 predefined quick messages
//! - Configurable via settings
//! - Used in multiplayer for fast communication

/// Maximum number of quick messages
pub const MAX_QUICK_MESSAGES: usize = 10;

/// Quick message structure
///
/// Contains a configuration key and the default message text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickMessage {
    /// Configuration variable name for this quick message
    pub key: &'static str,
    /// Default message text
    pub message: &'static str,
}

impl QuickMessage {
    /// Creates a new quick message
    pub const fn new(key: &'static str, message: &'static str) -> Self {
        Self { key, message }
    }
}

/// Default quick messages
///
/// These are the default messages that can be sent using F1-F10 keys.
/// Users can customize these in the game settings.
///
/// # C++ Reference
/// ```cpp
/// std::array<QuickMessage, 10> QuickMessages = {
///     QuickMessage { "QuickMessage1", N_("I need help! Come here!") },
///     QuickMessage { "QuickMessage2", N_("Follow me.") },
///     // ...
/// };
/// ```
pub const DEFAULT_QUICK_MESSAGES: [QuickMessage; MAX_QUICK_MESSAGES] = [
    QuickMessage::new("QuickMessage1", "I need help! Come here!"),
    QuickMessage::new("QuickMessage2", "Follow me."),
    QuickMessage::new("QuickMessage3", "Here's something for you."),
    QuickMessage::new("QuickMessage4", "Now you DIE!"),
    QuickMessage::new("QuickMessage5", "Heal yourself!"),
    QuickMessage::new("QuickMessage6", "Watch out!"),
    QuickMessage::new("QuickMessage7", "Thanks."),
    QuickMessage::new("QuickMessage8", "Retreat!"),
    QuickMessage::new("QuickMessage9", "Sorry."),
    QuickMessage::new("QuickMessage10", "I'm waiting."),
];

/// Quick message manager
///
/// Manages the quick messages, allowing for customization while
/// maintaining the default messages.
#[derive(Debug, Clone)]
pub struct QuickMessageManager {
    /// Current quick messages (may be customized)
    messages: [String; MAX_QUICK_MESSAGES],
}

impl Default for QuickMessageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QuickMessageManager {
    /// Creates a new quick message manager with default messages
    pub fn new() -> Self {
        Self {
            messages: DEFAULT_QUICK_MESSAGES
                .iter()
                .map(|m| m.message.to_string())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    /// Gets a quick message by index (0-9)
    ///
    /// # Arguments
    /// * `index` - Message index (0-9)
    ///
    /// # Returns
    /// The message text, or None if index is out of range
    pub fn get(&self, index: usize) -> Option<&str> {
        self.messages.get(index).map(|s| s.as_str())
    }

    /// Gets a quick message by function key (F1-F10)
    ///
    /// # Arguments
    /// * `fkey` - Function key number (1-10)
    ///
    /// # Returns
    /// The message text, or None if key is out of range
    pub fn get_by_fkey(&self, fkey: u8) -> Option<&str> {
        if fkey >= 1 && fkey <= 10 {
            self.get((fkey - 1) as usize)
        } else {
            None
        }
    }

    /// Sets a custom message at the specified index
    ///
    /// # Arguments
    /// * `index` - Message index (0-9)
    /// * `message` - The new message text
    ///
    /// # Returns
    /// `true` if the message was set, `false` if index is out of range
    pub fn set(&mut self, index: usize, message: impl Into<String>) -> bool {
        if index < MAX_QUICK_MESSAGES {
            self.messages[index] = message.into();
            true
        } else {
            false
        }
    }

    /// Resets a message to its default value
    ///
    /// # Arguments
    /// * `index` - Message index (0-9)
    ///
    /// # Returns
    /// `true` if reset was successful, `false` if index is out of range
    pub fn reset(&mut self, index: usize) -> bool {
        if index < MAX_QUICK_MESSAGES {
            self.messages[index] = DEFAULT_QUICK_MESSAGES[index].message.to_string();
            true
        } else {
            false
        }
    }

    /// Resets all messages to their default values
    pub fn reset_all(&mut self) {
        for (i, msg) in DEFAULT_QUICK_MESSAGES.iter().enumerate() {
            self.messages[i] = msg.message.to_string();
        }
    }

    /// Gets the configuration key for a message index
    ///
    /// # Arguments
    /// * `index` - Message index (0-9)
    ///
    /// # Returns
    /// The configuration key, or None if index is out of range
    pub fn get_key(index: usize) -> Option<&'static str> {
        DEFAULT_QUICK_MESSAGES.get(index).map(|m| m.key)
    }

    /// Gets all messages as a slice
    pub fn all(&self) -> &[String; MAX_QUICK_MESSAGES] {
        &self.messages
    }

    /// Iterates over all messages with their indices
    pub fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.messages.iter().enumerate().map(|(i, s)| (i, s.as_str()))
    }
}

/// Gets the default message for a given index
///
/// # Arguments
/// * `index` - Message index (0-9)
///
/// # Returns
/// The default message text, or None if index is out of range
pub fn get_default_message(index: usize) -> Option<&'static str> {
    DEFAULT_QUICK_MESSAGES.get(index).map(|m| m.message)
}

/// Gets the configuration key for a given index
///
/// # Arguments
/// * `index` - Message index (0-9)
///
/// # Returns
/// The configuration key, or None if index is out of range
pub fn get_config_key(index: usize) -> Option<&'static str> {
    DEFAULT_QUICK_MESSAGES.get(index).map(|m| m.key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_messages() {
        assert_eq!(DEFAULT_QUICK_MESSAGES.len(), MAX_QUICK_MESSAGES);
        assert_eq!(DEFAULT_QUICK_MESSAGES[0].message, "I need help! Come here!");
        assert_eq!(DEFAULT_QUICK_MESSAGES[9].message, "I'm waiting.");
    }

    #[test]
    fn test_config_keys() {
        assert_eq!(DEFAULT_QUICK_MESSAGES[0].key, "QuickMessage1");
        assert_eq!(DEFAULT_QUICK_MESSAGES[9].key, "QuickMessage10");
    }

    #[test]
    fn test_manager_new() {
        let manager = QuickMessageManager::new();
        assert_eq!(manager.get(0), Some("I need help! Come here!"));
        assert_eq!(manager.get(9), Some("I'm waiting."));
    }

    #[test]
    fn test_manager_get_by_fkey() {
        let manager = QuickMessageManager::new();
        assert_eq!(manager.get_by_fkey(1), Some("I need help! Come here!"));
        assert_eq!(manager.get_by_fkey(10), Some("I'm waiting."));
        assert_eq!(manager.get_by_fkey(0), None);
        assert_eq!(manager.get_by_fkey(11), None);
    }

    #[test]
    fn test_manager_set() {
        let mut manager = QuickMessageManager::new();
        assert!(manager.set(0, "Custom message"));
        assert_eq!(manager.get(0), Some("Custom message"));

        // Out of range
        assert!(!manager.set(10, "Invalid"));
    }

    #[test]
    fn test_manager_reset() {
        let mut manager = QuickMessageManager::new();
        manager.set(0, "Custom message");
        assert_eq!(manager.get(0), Some("Custom message"));

        manager.reset(0);
        assert_eq!(manager.get(0), Some("I need help! Come here!"));
    }

    #[test]
    fn test_manager_reset_all() {
        let mut manager = QuickMessageManager::new();
        manager.set(0, "Custom 1");
        manager.set(5, "Custom 2");

        manager.reset_all();
        assert_eq!(manager.get(0), Some("I need help! Come here!"));
        assert_eq!(manager.get(5), Some("Watch out!"));
    }

    #[test]
    fn test_get_key() {
        assert_eq!(QuickMessageManager::get_key(0), Some("QuickMessage1"));
        assert_eq!(QuickMessageManager::get_key(9), Some("QuickMessage10"));
        assert_eq!(QuickMessageManager::get_key(10), None);
    }

    #[test]
    fn test_iter() {
        let manager = QuickMessageManager::new();
        let items: Vec<_> = manager.iter().collect();
        assert_eq!(items.len(), 10);
        assert_eq!(items[0], (0, "I need help! Come here!"));
    }

    #[test]
    fn test_get_default_message() {
        assert_eq!(get_default_message(0), Some("I need help! Come here!"));
        assert_eq!(get_default_message(10), None);
    }

    #[test]
    fn test_get_config_key() {
        assert_eq!(get_config_key(0), Some("QuickMessage1"));
        assert_eq!(get_config_key(10), None);
    }

    #[test]
    fn test_quick_message_struct() {
        let msg = QuickMessage::new("TestKey", "Test Message");
        assert_eq!(msg.key, "TestKey");
        assert_eq!(msg.message, "Test Message");
    }
}
