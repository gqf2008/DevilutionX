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
        }
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
}
