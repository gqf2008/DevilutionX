//! Storm network API over the loopback session (C++ `Source/storm/storm_net.hpp`).
//!
//! The game layer (multi.cpp) talks to the network through the storm SNet*
//! interface. This module binds those calls to `net::base_protocol::BaseProtocol`
//! (wire framing + loopback transport) for the single-player path.

use crate::net::base_protocol::BaseProtocol;
use crate::net::transport::ProviderCaps;

/// C++ `conn_type::SELCONN_LOOPBACK`.
pub const SELCONN_LOOPBACK: u8 = 2;

/// C++ `PS_CONNECTED` player-state bit (Source/storm/storm_net.hpp).
pub const PS_CONNECTED: u32 = 0x10000;

/// C++ `event_type` (Source/storm/storm_net.hpp).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    PlayerCreateGame = 0,
    PlayerLeaveGame = 1,
    PlayerMessage = 2,
}

/// Storm network provider state (single-player loopback path).
#[derive(Debug)]
pub struct StormNet {
    session: BaseProtocol,
    game_name: String,
    /// C++ `provider` global — SELCONN_LOOPBACK for the loopback transport.
    pub provider: u8,
}

impl Default for StormNet {
    fn default() -> Self {
        Self::new()
    }
}

impl StormNet {
    pub fn new() -> Self {
        Self {
            session: BaseProtocol::new(),
            game_name: String::new(),
            provider: SELCONN_LOOPBACK,
        }
    }

    /// C++ `SNetCreateGame(...)`: host a game. Loopback always succeeds and
    /// assigns the local player as player 0.
    pub fn create_game(&mut self, game_name: &str, _password: &str, _template: &[u8]) -> Option<u8> {
        self.game_name = game_name.to_string();
        let pid = self.session.create_game(game_name);
        Some(pid as u8)
    }

    /// C++ `SNetJoinGame(...)`: the loopback transport cannot join
    /// (loopback::join ABORTs); returns `None`.
    pub fn join_game(&mut self, game_name: &str, _password: &str) -> Option<u8> {
        if self.session.join_game(game_name) < 0 {
            None
        } else {
            Some(self.session.player_id)
        }
    }

    /// C++ `SNetSendMessage(dest, data, size)`.
    pub fn send_message(&mut self, dest: u8, data: &[u8]) -> bool {
        self.session.send_message(dest, data)
    }

    /// C++ `SNetReceiveMessage(sender, data, size)`.
    pub fn receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        self.session.receive_message()
    }

    /// C++ `SNetSendTurn` — no-op on loopback.
    pub fn send_turn(&mut self) -> bool {
        true
    }

    /// C++ `SNetReceiveTurns` — empty turn per player.
    pub fn receive_turns(&self) -> Vec<Vec<u8>> {
        self.session.receive_turns()
    }

    /// C++ `SNetLeaveGame`.
    pub fn leave_game(&mut self, reason: u32) -> bool {
        self.session.leave_game(reason)
    }

    /// C++ `SNetDropPlayer`.
    pub fn drop_player(&mut self, player: u8, reason: u32) -> bool {
        self.session.drop_player(player, reason)
    }

    /// C++ `SNetGetTurnsInTransit`.
    pub fn turns_in_transit(&self) -> u32 {
        0
    }

    /// C++ `SNetGetProviderCaps`.
    pub fn provider_caps(&self) -> ProviderCaps {
        self.session.provider_caps()
    }

    /// C++ `IsGameHost()`.
    pub fn is_game_host(&self) -> bool {
        self.session.is_game_host()
    }

    /// The active game name (C++ `gamename`).
    pub fn game_name(&self) -> &str {
        &self.game_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_game_assigns_single_player() {
        let mut net = StormNet::new();
        let pid = net.create_game("mygame", "", b"template").expect("loopback host");
        assert_eq!(pid, 0);
        assert!(net.is_game_host());
        assert_eq!(net.game_name(), "mygame");
        assert_eq!(net.provider, SELCONN_LOOPBACK);
    }

    #[test]
    fn test_join_game_fails_on_loopback() {
        let mut net = StormNet::new();
        assert!(net.join_game("somegame", "").is_none());
    }

    #[test]
    fn test_message_round_trip_through_storm() {
        let mut net = StormNet::new();
        net.create_game("mygame", "", b"");
        assert!(net.send_message(0, b"\x01cmd"));
        let (sender, data) = net.receive_message().expect("queued");
        assert_eq!(sender, 0);
        assert_eq!(data, b"\x01cmd");
    }

    #[test]
    fn test_turns_and_caps_via_storm() {
        let net = StormNet::new();
        assert_eq!(net.receive_turns().len(), 4);
        assert_eq!(net.turns_in_transit(), 0);
        let caps = net.provider_caps();
        assert_eq!(caps.max_players, 4);
        assert_eq!(caps.max_message_size, 512);
    }
}
