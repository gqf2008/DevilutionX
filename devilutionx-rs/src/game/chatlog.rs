//! Chat log QoL feature (C++ `Source/qol/chatlog.cpp`).
//!
//! A rolling buffer of chat lines. The line formatting mirrors C++
//! `AddMessageToChatLog`: a `[#N] HH:MM:SS` timestamp (or just `[#N]` when no
//! timestamp is available), an optional `name (lvl X): ` player prefix, and
//! the message text. The buffer keeps every line so the UI can render a
//! scrollable window; layout/scroll constants are not ported.

/// Chat line colour (C++ `UiFlags` used by the log).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatColor {
    Red,
    Blue,
    WhiteGold,
    White,
}

/// One log line with its colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLogLine {
    pub text: String,
    pub color: ChatColor,
}

/// Rolling chat log (C++ `ChatLogLines` + `MessageCounter`).
#[derive(Debug, Clone, Default)]
pub struct ChatLog {
    lines: Vec<ChatLogLine>,
    message_counter: u32,
}

impl ChatLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of messages ever added (C++ `MessageCounter`).
    pub fn message_counter(&self) -> u32 {
        self.message_counter
    }

    pub fn lines(&self) -> &[ChatLogLine] {
        &self.lines
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// C++ `AddMessageToChatLog(message, player, flags)`.
    ///
    /// `timestamp` is `Some("HH:MM:SS")` when a wall-clock is available
    /// (injectable for deterministic tests); the system message path
    /// (player = None) uses the C++ `"{0} {1}"` layout with the timestamp in
    /// red and the message in `message_color`.
    pub fn add_message(
        &mut self,
        message: &str,
        player: Option<PlayerInfo>,
        timestamp: Option<&str>,
        message_color: ChatColor,
    ) {
        self.message_counter += 1;
        let counter = self.message_counter;
        let stamp = match timestamp {
            Some(t) => format!("[#{counter}] {t}"),
            None => format!("[#{counter}]"),
        };
        match player {
            Some(info) => {
                let name_color = if info.is_self {
                    ChatColor::WhiteGold
                } else {
                    ChatColor::Blue
                };
                let prefix = format!("{} - {} (lvl {}): ", stamp, info.name, info.level);
                let text = format!("{prefix}{message}");
                self.lines.push(ChatLogLine {
                    text,
                    color: name_color,
                });
            }
            None => {
                self.lines.push(ChatLogLine {
                    text: format!("{stamp} {message}"),
                    color: message_color,
                });
            }
        }
    }
}

/// Player identity for chat-log formatting (C++ `Player`).
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub level: u8,
    pub is_self: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_message_format() {
        let mut log = ChatLog::new();
        log.add_message("Welcome", None, Some("12:34:56"), ChatColor::White);
        assert_eq!(log.message_counter(), 1);
        assert_eq!(log.lines()[0].text, "[#1] 12:34:56 Welcome");
        assert_eq!(log.lines()[0].color, ChatColor::White);
    }

    #[test]
    fn test_no_timestamp_falls_back_to_counter_only() {
        let mut log = ChatLog::new();
        log.add_message("Hi", None, None, ChatColor::Red);
        assert_eq!(log.lines()[0].text, "[#1] Hi");
    }

    #[test]
    fn test_player_message_format_and_colors() {
        let mut log = ChatLog::new();
        log.add_message(
            "gg",
            Some(PlayerInfo {
                name: "Aria".to_string(),
                level: 12,
                is_self: false,
            }),
            Some("01:02:03"),
            ChatColor::White,
        );
        assert_eq!(log.lines()[0].text, "[#1] 01:02:03 - Aria (lvl 12): gg");
        assert_eq!(log.lines()[0].color, ChatColor::Blue);

        log.add_message(
            "me too",
            Some(PlayerInfo {
                name: "Me".to_string(),
                level: 3,
                is_self: true,
            }),
            Some("01:02:04"),
            ChatColor::White,
        );
        assert_eq!(log.lines()[1].text, "[#2] 01:02:04 - Me (lvl 3): me too");
        assert_eq!(log.lines()[1].color, ChatColor::WhiteGold);
    }

    #[test]
    fn test_counter_increments_and_clear() {
        let mut log = ChatLog::new();
        log.add_message("a", None, None, ChatColor::White);
        log.add_message("b", None, None, ChatColor::White);
        assert_eq!(log.message_counter(), 2);
        assert_eq!(log.lines().len(), 2);
        log.clear();
        assert!(log.lines().is_empty());
        // The counter keeps counting (C++ MessageCounter is never reset).
        assert_eq!(log.message_counter(), 2);
    }
}
