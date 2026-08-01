//! Network transports (C++ `Source/dvlnet/`).
//!
//! `Loopback` is a port of `dvlnet/loopback.cpp`: an in-memory message queue
//! used for single-player pseudo-multiplayer. The transport is byte-agnostic —
//! the base-protocol layer (not yet ported) is responsible for framing game
//! data into the wire packets defined in `net::packet`.

use std::collections::VecDeque;

/// C++ `plr_single` in dvlnet/loopback.cpp.
pub const PLR_SINGLE: u8 = 0;

/// C++ `MAX_PLRS` (Source/multi.h).
pub const MAX_PLRS: usize = 4;

/// Provider capabilities, mirroring `loopback::SNetGetProviderCaps` in
/// dvlnet/loopback.cpp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderCaps {
    pub max_message_size: usize,
    pub max_players: usize,
    pub bytes_per_sec: usize,
    pub default_turns_sec: usize,
    pub default_turns_in_transit: usize,
}

impl Default for ProviderCaps {
    fn default() -> Self {
        Self {
            max_message_size: 512,
            max_players: MAX_PLRS,
            bytes_per_sec: 1_000_000,
            default_turns_sec: 10,
            default_turns_in_transit: 1,
        }
    }
}

/// In-memory loopback transport (C++ `net::loopback`).
#[derive(Debug, Default)]
pub struct Loopback {
    message_queue: VecDeque<Vec<u8>>,
    /// C++ `IsLoopback` global, set by `create()`.
    pub is_loopback: bool,
}

impl Loopback {
    pub fn new() -> Self {
        Self {
            message_queue: VecDeque::new(),
            is_loopback: false,
        }
    }

    /// C++ `loopback::create()`: switches the engine into loopback mode and
    /// returns the single-player id.
    pub fn create(&mut self) -> u8 {
        self.is_loopback = true;
        PLR_SINGLE
    }

    /// C++ `SNetSendMessage(dest, data, size)`: only messages addressed to the
    /// single player are queued; everything else is silently accepted.
    pub fn send_message(&mut self, dest: u8, data: &[u8]) -> bool {
        if dest == PLR_SINGLE {
            self.message_queue.push_back(data.to_vec());
        }
        true
    }

    /// C++ `SNetReceiveMessage(sender, data, size)`: pops the oldest queued
    /// message. Returns `(sender, payload)` or `None` when empty.
    pub fn receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        let payload = self.message_queue.pop_front()?;
        Some((PLR_SINGLE, payload))
    }

    /// C++ `SNetReceiveTurns`: loopback has no turn data — all players get an
    /// empty turn.
    pub fn receive_turns(&self) -> Vec<Vec<u8>> {
        vec![Vec::new(); MAX_PLRS]
    }

    /// C++ `SNetSendTurn`: no-op.
    pub fn send_turn(&mut self) {}

    /// C++ `SNetGetProviderCaps`.
    pub fn provider_caps(&self) -> ProviderCaps {
        ProviderCaps::default()
    }

    /// C++ `loopback::make_default_gamename()`.
    pub fn make_default_gamename(&self) -> String {
        "loopback".to_string()
    }

    /// Number of messages currently queued (diagnostic).
    pub fn queued(&self) -> usize {
        self.message_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_enters_loopback_and_returns_single() {
        let mut lb = Loopback::new();
        assert!(!lb.is_loopback);
        assert_eq!(lb.create(), PLR_SINGLE);
        assert!(lb.is_loopback);
    }

    #[test]
    fn test_message_queue_fifo() {
        let mut lb = Loopback::new();
        lb.create();
        // Messages to other players are accepted but not delivered.
        assert!(lb.send_message(1, b"drop".as_slice()));
        assert_eq!(lb.queued(), 0);
        // The single player's messages are queued FIFO.
        lb.send_message(PLR_SINGLE, b"first");
        lb.send_message(PLR_SINGLE, b"second");
        assert_eq!(lb.queued(), 2);
        assert_eq!(lb.receive_message(), Some((PLR_SINGLE, b"first".to_vec())));
        assert_eq!(lb.receive_message(), Some((PLR_SINGLE, b"second".to_vec())));
        assert_eq!(lb.receive_message(), None);
    }

    #[test]
    fn test_turns_and_caps_match_cpp() {
        let mut lb = Loopback::new();
        lb.create();
        // C++ loopback returns empty turns for every player and no-ops send_turn.
        assert_eq!(lb.receive_turns().len(), MAX_PLRS);
        assert!(lb.receive_turns().iter().all(|t| t.is_empty()));
        lb.send_turn();
        let caps = lb.provider_caps();
        assert_eq!(caps.max_message_size, 512);
        assert_eq!(caps.max_players, MAX_PLRS);
        assert_eq!(caps.default_turns_in_transit, 1);
    }
}
