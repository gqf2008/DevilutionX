//! TCP transport (C++ `Source/dvlnet/tcp_client.cpp` / `tcp_server.cpp`).
//!
//! Frames every message with `net::frame_queue::FrameQueue` (a 4-byte
//! little-endian size word), exactly as the C++ transport does. The session
//! handshake (JOIN_REQUEST / cookie / player assignment) lives in
//! `net::base_protocol` and is layered on top of this transport.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::net::frame_queue::FrameQueue;

/// A connected TCP endpoint with frame-queue reassembly.
#[derive(Debug)]
pub struct TcpConnection {
    stream: TcpStream,
    read_queue: FrameQueue,
}

impl TcpConnection {
    /// Connect to a host:port and enable no-delay (C++ sets TCP_NODELAY).
    pub fn connect(host: &str, port: u16) -> std::io::Result<Self> {
        let stream = TcpStream::connect((host, port))?;
        stream.set_nodelay(true).ok();
        Ok(Self {
            stream,
            read_queue: FrameQueue::new(),
        })
    }

    /// Write an already-framed byte sequence (C++ asio::async_write of
    /// `frame_queue::MakeFrame` output).
    pub fn send_frame(&mut self, frame: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(frame)?;
        self.stream.flush()
    }

    /// Block until the next frame is available and return its payload
    /// (C++ `HandleReceive` -> `recv_queue.ReadPacket`).
    pub fn receive_frame(&mut self) -> std::io::Result<Vec<u8>> {
        loop {
            if self.read_queue.packet_ready().unwrap_or(false) {
                return self
                    .read_queue
                    .read_packet()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e));
            }
            let mut buf = [0u8; 4096];
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "connection closed by peer",
                ));
            }
            self.read_queue.write(&buf[..n]);
        }
    }
}

/// A listening TCP server (C++ `tcp_server`).
#[derive(Debug)]
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    /// Bind to 127.0.0.1:port (port 0 picks an ephemeral port for tests).
    pub fn bind(port: u16) -> std::io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(("127.0.0.1", port))?,
        })
    }

    /// The bound port (useful when binding port 0).
    pub fn local_port(&self) -> u16 {
        self.listener.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    /// Accept the next connection (C++ `handle_accept`).
    pub fn accept(&self) -> std::io::Result<TcpConnection> {
        let (stream, _) = self.listener.accept()?;
        stream.set_nodelay(true).ok();
        Ok(TcpConnection {
            stream,
            read_queue: FrameQueue::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_frame_exchange_echo() {
        let server = TcpServer::bind(0).expect("bind ephemeral port");
        let port = server.local_port();
        assert!(port > 0);

        let server_thread = std::thread::spawn(move || {
            let mut conn = server.accept().expect("accept");
            let payload = conn.receive_frame().expect("server receives frame");
            // Echo the payload back framed.
            conn.send_frame(&FrameQueue::make_frame(&payload, 0).expect("frame")).expect("send");
            payload
        });

        let mut client = TcpConnection::connect("127.0.0.1", port).expect("connect");
        let payload = b"hello tcp transport";
        client
            .send_frame(&FrameQueue::make_frame(payload, 0).expect("frame"))
            .expect("client send");
        let echo = client.receive_frame().expect("client receives echo");
        assert_eq!(echo, payload);
        assert_eq!(server_thread.join().expect("server thread"), payload);
    }

    #[test]
    fn test_tcp_multiple_frames_round_trip() {
        let server = TcpServer::bind(0).unwrap();
        let port = server.local_port();
        let server_thread = std::thread::spawn(move || {
            let mut conn = server.accept().unwrap();
            let mut got = Vec::new();
            got.push(conn.receive_frame().unwrap());
            got.push(conn.receive_frame().unwrap());
            got
        });
        let mut client = TcpConnection::connect("127.0.0.1", port).unwrap();
        for p in [b"one".as_slice(), b"two".as_slice()] {
            client.send_frame(&FrameQueue::make_frame(p, 0).unwrap()).unwrap();
        }
        let got = server_thread.join().unwrap();
        assert_eq!(got, vec![b"one".to_vec(), b"two".to_vec()]);
    }
}
