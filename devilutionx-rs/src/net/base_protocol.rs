//! Session layer bridging game data and the wire transport.
//!
//! Port of C++ `Source/dvlnet/base_protocol.h`: game commands (raw bytes) are
//! framed into `PT_MESSAGE` wire packets (`net::packet`) and exchanged through
//! a transport (`net::transport::Loopback` for single-player). Mirrors
//! `base_protocol::SendTo` / `recv_decrypted` for the message path; the full
//! join/handshake state machine remains a follow-up.

use crate::net::packet::{packet_type, Packet};
use crate::net::transport::{Loopback, PLR_SINGLE};

/// C++ `base_protocol<P>` over the loopback transport.
#[derive(Debug)]
pub struct BaseProtocol {
    transport: Loopback,
    /// Local player id (C++ `my_player_id`).
    pub player_id: u8,
    /// C++ `isGameHost_`.
    pub is_game_host: bool,
    /// C++ `game_init_info` payload.
    game_info: Vec<u8>,
}

impl Default for BaseProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseProtocol {
    pub fn new() -> Self {
        Self {
            transport: Loopback::new(),
            player_id: PLR_SINGLE,
            is_game_host: false,
            game_info: Vec::new(),
        }
    }

    /// C++ `base_protocol::create(addrstr)` — host a game. The loopback
    /// transport is always "online", so hosting succeeds immediately with the
    /// local player as player 0.
    pub fn create_game(&mut self, _addrstr: &str) -> i32 {
        self.is_game_host = true;
        self.player_id = self.transport.create();
        self.player_id as i32
    }

    /// C++ `base_protocol::join(addrstr)` — the loopback transport ABORTs on
    /// join (dvlnet/loopback.cpp); mirror that as an error return.
    pub fn join_game(&mut self, _addrstr: &str) -> i32 {
        -1
    }

    /// C++ `IsGameHost()`.
    pub fn is_game_host(&self) -> bool {
        self.is_game_host
    }

    /// C++ `SNetLeaveGame` — the loopback has no peers, so it always succeeds.
    pub fn leave_game(&mut self, _reason: u32) -> bool {
        true
    }

    /// C++ `SNetDropPlayer` — the loopback has no other players.
    pub fn drop_player(&mut self, _player_id: u8, _reason: u32) -> bool {
        true
    }

    /// C++ `setup_gameinfo(info)`.
    pub fn set_game_info(&mut self, info: Vec<u8>) {
        self.game_info = info;
    }

    /// C++ `game_init_info` (mirrors the stored payload).
    pub fn game_info(&self) -> &[u8] {
        &self.game_info
    }

    /// C++ `make_default_gamename()` (via the transport).
    pub fn make_default_gamename(&self) -> String {
        self.transport.make_default_gamename()
    }

    /// C++ `base_protocol::create()` — enters loopback mode as the single
    /// player and returns the local player id.
    pub fn create(&mut self) -> u8 {
        self.player_id = self.transport.create();
        self.player_id
    }

    /// C++ `base_protocol::SendTo(player, pkt)` for the message path: frame
    /// `data` as a `PT_MESSAGE` wire packet from the local player and hand it
    /// to the transport.
    pub fn send_message(&mut self, dest: u8, data: &[u8]) -> bool {
        let wire = Packet {
            packet_type: packet_type::PT_MESSAGE,
            source: self.player_id,
            destination: dest,
            message: data.to_vec(),
            ..Packet::default()
        };
        self.transport.send_message(dest, &wire.encode())
    }

    /// C++ `recv_decrypted` / `SNetReceiveMessage`: pop the next wire packet,
    /// decode it, and return `(sender, payload)`.
    pub fn receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        let (_, bytes) = self.transport.receive_message()?;
        let pkt = Packet::decode(&bytes).ok()?;
        Some((pkt.source, pkt.message))
    }

    /// C++ `SNetReceiveTurns`: empty turn per player on loopback.
    pub fn receive_turns(&self) -> Vec<Vec<u8>> {
        self.transport.receive_turns()
    }

    /// C++ `SNetSendTurn`: no-op on loopback.
    pub fn send_turn(&mut self) {}

    /// C++ `SNetGetProviderCaps` (via the transport).
    pub fn provider_caps(&self) -> crate::net::transport::ProviderCaps {
        self.transport.provider_caps()
    }

    /// Whether loopback mode is active (single-player pseudo-multiplayer).
    pub fn is_loopback(&self) -> bool {
        self.transport.is_loopback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet::{packet_type, PLR_BROADCAST};

    #[test]
    fn test_create_returns_single_player() {
        let mut session = BaseProtocol::new();
        assert_eq!(session.create(), PLR_SINGLE);
        assert!(session.is_loopback());
    }

    #[test]
    fn test_send_receive_message_round_trip_via_wire_packets() {
        let mut session = BaseProtocol::new();
        session.create();

        // The transport stores the C++ PT_MESSAGE wire bytes: 3-byte header +
        // trailing payload (no length prefix).
        let payload = b"\x01\x02\x03cmd".as_slice();
        // Loopback only delivers messages addressed to the single player.
        assert!(session.send_message(PLR_SINGLE, payload));
        let (sender, data) = session.receive_message().expect("queued message");
        assert_eq!(sender, session.player_id);
        assert_eq!(data, payload);

        // Drain is exhausted.
        assert!(session.receive_message().is_none());
    }

    #[test]
    fn test_message_payload_survives_framing() {
        // The PT_MESSAGE wire layout itself is byte-exact tested in
        // net::packet; here we confirm non-trivial payloads survive the
        // frame + transport + unframe round-trip intact.
        let mut session = BaseProtocol::new();
        session.create();
        let payload = vec![0u8, 0xFE, 0xFF, 7, 8, 9, 10, 11];
        assert!(session.send_message(PLR_SINGLE, &payload));
        let (sender, data) = session.receive_message().expect("queued");
        assert_eq!(sender, PLR_SINGLE);
        assert_eq!(data, payload);
    }

    #[test]
    fn test_create_game_hosts_immediately() {
        let mut session = BaseProtocol::new();
        assert!(!session.is_game_host());
        let pid = session.create_game("loopback");
        assert_eq!(pid, PLR_SINGLE as i32);
        assert!(session.is_game_host());
        assert!(session.is_loopback());
        // Game info round-trips.
        session.set_game_info(vec![1, 2, 3, 4]);
        assert_eq!(session.game_info(), &[1, 2, 3, 4]);
        assert_eq!(session.make_default_gamename(), "loopback");
    }

    #[test]
    fn test_join_game_fails_on_loopback() {
        let mut session = BaseProtocol::new();
        assert_eq!(session.join_game("somewhere"), -1, "loopback cannot join");
        assert!(!session.is_game_host());
    }

    #[test]
    fn test_leave_and_drop_succeed_on_loopback() {
        let mut session = BaseProtocol::new();
        session.create_game("loopback");
        assert!(session.leave_game(crate::net::packet::leave_info::LEAVE_EXIT));
        assert!(session.drop_player(1, crate::net::packet::leave_info::LEAVE_DROP));
    }
}
