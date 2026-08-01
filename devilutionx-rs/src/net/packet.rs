//! C++-compatible wire packets.
//!
//! Port of `Source/dvlnet/packet.h` / `packet.cpp` (DevilutionX multiplayer).
//! A packet is `[type u8][src u8][dest u8]` followed by a per-type body.
//! Scalars are little-endian (1/2/4 bytes); buffer fields (`message`/`info`)
//! are the trailing payload with no length prefix.

/// Packet type ids (C++ `enum packet_type` in dvlnet/packet.h).
pub mod packet_type {
    pub const PT_MESSAGE: u8 = 0x01;
    pub const PT_TURN: u8 = 0x02;
    pub const PT_JOIN_REQUEST: u8 = 0x11;
    pub const PT_JOIN_ACCEPT: u8 = 0x12;
    pub const PT_CONNECT: u8 = 0x13;
    pub const PT_DISCONNECT: u8 = 0x14;
    pub const PT_INFO_REQUEST: u8 = 0x21;
    pub const PT_INFO_REPLY: u8 = 0x22;
    pub const PT_ECHO_REQUEST: u8 = 0x31;
    pub const PT_ECHO_REPLY: u8 = 0x32;
}

/// C++ `PLR_MASTER` / `PLR_BROADCAST` (dvlnet/packet.h).
pub const PLR_MASTER: u8 = 0xFE;
pub const PLR_BROADCAST: u8 = 0xFF;

/// C++ `leaveinfo_t` values (dvlnet/leaveinfo.hpp).
pub mod leave_info {
    pub const LEAVE_EXIT: u32 = 3;
    pub const LEAVE_ENDING: u32 = 0x4000_0004;
    pub const LEAVE_DROP: u32 = 0x4000_0006;
}

/// A decoded/encodable packet, mirroring C++ `net::packet` fields.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Packet {
    pub packet_type: u8,
    pub source: u8,
    pub destination: u8,
    /// PT_MESSAGE payload.
    pub message: Vec<u8>,
    /// PT_TURN.
    pub sequence: u8,
    pub value: i32,
    /// PT_JOIN_REQUEST / PT_JOIN_ACCEPT.
    pub cookie: u32,
    /// PT_JOIN_ACCEPT / PT_CONNECT / PT_DISCONNECT.
    pub new_player: u8,
    /// PT_DISCONNECT leave reason.
    pub leave_info: u32,
    /// PT_ECHO_REQUEST / PT_ECHO_REPLY.
    pub time: u32,
    /// PT_JOIN_REQUEST / PT_JOIN_ACCEPT / PT_CONNECT / PT_INFO_REPLY payload.
    pub info: Vec<u8>,
}

impl Packet {
    pub fn new(packet_type: u8, source: u8, destination: u8) -> Self {
        Self {
            packet_type,
            source,
            destination,
            ..Self::default()
        }
    }

    /// Serialize to the C++ wire format (dvlnet/packet.h `process_data`).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![self.packet_type, self.source, self.destination];
        match self.packet_type {
            packet_type::PT_MESSAGE => out.extend_from_slice(&self.message),
            packet_type::PT_TURN => {
                out.push(self.sequence);
                out.extend_from_slice(&self.value.to_le_bytes());
            }
            packet_type::PT_JOIN_REQUEST => {
                out.extend_from_slice(&self.cookie.to_le_bytes());
                out.extend_from_slice(&self.info);
            }
            packet_type::PT_JOIN_ACCEPT => {
                out.extend_from_slice(&self.cookie.to_le_bytes());
                out.push(self.new_player);
                out.extend_from_slice(&self.info);
            }
            packet_type::PT_CONNECT => {
                out.push(self.new_player);
                out.extend_from_slice(&self.info);
            }
            packet_type::PT_DISCONNECT => {
                out.push(self.new_player);
                out.extend_from_slice(&self.leave_info.to_le_bytes());
            }
            packet_type::PT_INFO_REPLY => out.extend_from_slice(&self.info),
            packet_type::PT_INFO_REQUEST => {}
            packet_type::PT_ECHO_REQUEST | packet_type::PT_ECHO_REPLY => {
                out.extend_from_slice(&self.time.to_le_bytes());
            }
            _ => {}
        }
        out
    }

    /// Deserialize from the C++ wire format.
    pub fn decode(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 3 {
            return Err(format!("packet too short: {} bytes", buf.len()));
        }
        let packet_type = buf[0];
        let source = buf[1];
        let destination = buf[2];
        let body = &buf[3..];
        let mut p = Packet::new(packet_type, source, destination);
        match packet_type {
            packet_type::PT_MESSAGE => p.message = body.to_vec(),
            packet_type::PT_TURN => {
                if body.len() < 5 {
                    return Err("PT_TURN body too short".into());
                }
                p.sequence = body[0];
                p.value = i32::from_le_bytes(body[1..5].try_into().unwrap());
            }
            packet_type::PT_JOIN_REQUEST => {
                if body.len() < 4 {
                    return Err("PT_JOIN_REQUEST body too short".into());
                }
                p.cookie = u32::from_le_bytes(body[0..4].try_into().unwrap());
                p.info = body[4..].to_vec();
            }
            packet_type::PT_JOIN_ACCEPT => {
                if body.len() < 5 {
                    return Err("PT_JOIN_ACCEPT body too short".into());
                }
                p.cookie = u32::from_le_bytes(body[0..4].try_into().unwrap());
                p.new_player = body[4];
                p.info = body[5..].to_vec();
            }
            packet_type::PT_CONNECT => {
                if body.is_empty() {
                    return Err("PT_CONNECT body too short".into());
                }
                p.new_player = body[0];
                p.info = body[1..].to_vec();
            }
            packet_type::PT_DISCONNECT => {
                if body.len() < 5 {
                    return Err("PT_DISCONNECT body too short".into());
                }
                p.new_player = body[0];
                p.leave_info = u32::from_le_bytes(body[1..5].try_into().unwrap());
            }
            packet_type::PT_INFO_REPLY => p.info = body.to_vec(),
            packet_type::PT_INFO_REQUEST => {}
            packet_type::PT_ECHO_REQUEST | packet_type::PT_ECHO_REPLY => {
                if body.len() < 4 {
                    return Err("echo body too short".into());
                }
                p.time = u32::from_le_bytes(body[0..4].try_into().unwrap());
            }
            other => return Err(format!("unknown packet type 0x{:02X}", other)),
        }
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_packet_bytes_match_cpp_layout() {
        // C++ PT_TURN: [type][src][dest][seq u8][value i32 LE].
        let p = Packet {
            packet_type: packet_type::PT_TURN,
            source: 0x01,
            destination: 0x02,
            sequence: 5,
            value: -7,
            ..Packet::default()
        };
        assert_eq!(p.encode(), vec![0x02, 0x01, 0x02, 5, 0xF9, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_message_packet_round_trip() {
        let p = Packet {
            packet_type: packet_type::PT_MESSAGE,
            source: PLR_BROADCAST,
            destination: 0,
            message: b"hello".to_vec(),
            ..Packet::default()
        };
        let bytes = p.encode();
        assert_eq!(bytes[0], packet_type::PT_MESSAGE);
        let decoded = Packet::decode(&bytes).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn test_join_request_round_trip() {
        let p = Packet {
            packet_type: packet_type::PT_JOIN_REQUEST,
            source: 3,
            destination: PLR_MASTER,
            cookie: 0xDEADBEEF,
            info: vec![1, 2, 3],
            ..Packet::default()
        };
        let decoded = Packet::decode(&p.encode()).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn test_disconnect_round_trip() {
        let p = Packet {
            packet_type: packet_type::PT_DISCONNECT,
            source: 2,
            destination: PLR_BROADCAST,
            new_player: 4,
            leave_info: leave_info::LEAVE_EXIT,
            ..Packet::default()
        };
        let decoded = Packet::decode(&p.encode()).unwrap();
        assert_eq!(decoded, p);
        assert_eq!(decoded.leave_info, 3);
    }

    #[test]
    fn test_all_types_round_trip() {
        let packets = [
            Packet { packet_type: packet_type::PT_INFO_REQUEST, source: 1, destination: 2, ..Packet::default() },
            Packet { packet_type: packet_type::PT_INFO_REPLY, source: 1, destination: 2, info: vec![9, 9], ..Packet::default() },
            Packet { packet_type: packet_type::PT_JOIN_ACCEPT, source: 1, destination: 2, cookie: 7, new_player: 3, info: vec![0], ..Packet::default() },
            Packet { packet_type: packet_type::PT_CONNECT, source: 1, destination: 2, new_player: 3, info: vec![], ..Packet::default() },
            Packet { packet_type: packet_type::PT_ECHO_REQUEST, source: 1, destination: 2, time: 1234, ..Packet::default() },
            Packet { packet_type: packet_type::PT_ECHO_REPLY, source: 1, destination: 2, time: 5678, ..Packet::default() },
        ];
        for p in packets {
            assert_eq!(Packet::decode(&p.encode()).unwrap(), p, "type 0x{:02X}", p.packet_type);
        }
    }

    #[test]
    fn test_decode_rejects_bad_input() {
        assert!(Packet::decode(&[]).is_err());
        assert!(Packet::decode(&[0x01, 0x02]).is_err());
        assert!(Packet::decode(&[0x99, 0, 0]).is_err());
        assert!(Packet::decode(&[packet_type::PT_TURN, 0, 0, 1]).is_err());
    }
}
