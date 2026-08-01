//! TCP session handshake over the frame transport.
//!
//! Mirrors the essential join flow of C++ `base_protocol<P>`: the client sends
//! a `PT_JOIN_REQUEST` with a cookie + game info, the host replies with a
//! `PT_JOIN_ACCEPT` carrying the same cookie and the assigned player id. The
//! full peer fan-out (`PT_CONNECT`), game-list discovery, and send queues are
//! follow-ups layered on this core.

use std::io;

use crate::net::frame_queue::FrameQueue;
use crate::net::packet::{packet_type, Packet, PLR_BROADCAST, PLR_MASTER};
use crate::net::tcp::{TcpConnection, TcpServer};

/// A joined TCP session (host or client) with an assigned player id.
#[derive(Debug)]
pub struct TcpSession {
    conn: TcpConnection,
    /// Assigned local player id (host = 0, first joiner = 1, ...).
    pub player_id: u8,
}

impl TcpSession {
    /// Host side: accept one connection and complete the join handshake.
    /// Returns `(session, cookie)` so the host can track the joiner.
    pub fn host_accept(server: &TcpServer, game_info: &[u8]) -> io::Result<(Self, u32)> {
        let mut conn = server.accept()?;
        let frame = conn.receive_frame()?;
        let req = Packet::decode(&frame)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if req.packet_type != packet_type::PT_JOIN_REQUEST {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected PT_JOIN_REQUEST, got 0x{:02X}", req.packet_type),
            ));
        }
        let cookie = req.cookie;
        let new_player = 1; // first joiner (host is player 0)
        let reply = Packet {
            packet_type: packet_type::PT_JOIN_ACCEPT,
            source: 0,
            destination: req.source,
            cookie,
            new_player,
            info: game_info.to_vec(),
            ..Packet::default()
        };
        conn.send_frame(&FrameQueue::make_frame(&reply.encode(), 0)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)?;
        Ok((Self { conn, player_id: 0 }, cookie))
    }

    /// Client side: connect to the host, send `PT_JOIN_REQUEST` with the given
    /// cookie, and verify the `PT_JOIN_ACCEPT` reply.
    pub fn client_join(host: &str, port: u16, cookie: u32, game_info: &[u8]) -> io::Result<Self> {
        let mut conn = TcpConnection::connect(host, port)?;
        let req = Packet {
            packet_type: packet_type::PT_JOIN_REQUEST,
            source: PLR_BROADCAST,
            destination: PLR_MASTER,
            cookie,
            info: game_info.to_vec(),
            ..Packet::default()
        };
        conn.send_frame(&FrameQueue::make_frame(&req.encode(), 0)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)?;
        let frame = conn.receive_frame()?;
        let reply = Packet::decode(&frame)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if reply.packet_type != packet_type::PT_JOIN_ACCEPT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected PT_JOIN_ACCEPT, got 0x{:02X}", reply.packet_type),
            ));
        }
        if reply.cookie != cookie {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "cookie mismatch in PT_JOIN_ACCEPT",
            ));
        }
        Ok(Self {
            conn,
            player_id: reply.new_player,
        })
    }

    /// Send a framed `PT_MESSAGE` wire packet to the peer.
    pub fn send_message(&mut self, dest: u8, data: &[u8]) -> io::Result<()> {
        let pkt = Packet {
            packet_type: packet_type::PT_MESSAGE,
            source: self.player_id,
            destination: dest,
            message: data.to_vec(),
            ..Packet::default()
        };
        self.conn.send_frame(&FrameQueue::make_frame(&pkt.encode(), 0)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    /// Receive and decode the next wire packet from the peer.
    pub fn receive_message(&mut self) -> io::Result<(u8, Vec<u8>)> {
        let frame = self.conn.receive_frame()?;
        let pkt = Packet::decode(&frame)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if pkt.packet_type != packet_type::PT_MESSAGE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected PT_MESSAGE, got 0x{:02X}", pkt.packet_type),
            ));
        }
        Ok((pkt.source, pkt.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::transport::PLR_SINGLE;

    #[test]
    fn test_tcp_join_handshake_and_message_exchange() {
        let server = TcpServer::bind(0).unwrap();
        let port = server.local_port();
        let game_info = vec![1, 2, 3];

        let server_info = game_info.clone();
        let server_thread = std::thread::spawn(move || {
            let (mut host, cookie) = TcpSession::host_accept(&server, &server_info).unwrap();
            assert_eq!(cookie, 0xCAFE_BABE);
            // Host receives the client's message.
            let (sender, msg) = host.receive_message().unwrap();
            assert_eq!(sender, 1);
            assert_eq!(msg, b"ping");
            host.send_message(1, b"pong").unwrap();
        });

        let mut client =
            TcpSession::client_join("127.0.0.1", port, 0xCAFE_BABE, &game_info).unwrap();
        assert_eq!(client.player_id, 1);
        client.send_message(PLR_SINGLE, b"ping").unwrap();
        let (_, reply) = client.receive_message().unwrap();
        assert_eq!(reply, b"pong");

        server_thread.join().unwrap();
    }

    #[test]
    fn test_tcp_handshake_rejects_bad_cookie() {
        let server = TcpServer::bind(0).unwrap();
        let port = server.local_port();
        let server_thread = std::thread::spawn(move || {
            // A buggy host replies with a different cookie than the request.
            let mut conn = server.accept().unwrap();
            let frame = conn.receive_frame().unwrap();
            let req = Packet::decode(&frame).unwrap();
            let reply = Packet {
                packet_type: packet_type::PT_JOIN_ACCEPT,
                source: 0,
                destination: req.source,
                cookie: req.cookie.wrapping_add(1),
                new_player: 1,
                info: vec![],
                ..Packet::default()
            };
            conn.send_frame(&FrameQueue::make_frame(&reply.encode(), 0).unwrap())
                .unwrap();
        });
        // The client must reject the mismatched cookie.
        let err = TcpSession::client_join("127.0.0.1", port, 43, &[]).unwrap_err();
        assert!(err.to_string().contains("cookie mismatch"));
        server_thread.join().unwrap();
    }
}
