//! TCP frame queue (C++ `Source/dvlnet/frame_queue.h` / `frame_queue.cpp`).
//!
//! Frames packets for the TCP transport: each frame is a 4-byte little-endian
//! `u32` (payload size | flags << 16) followed by the payload. The queue
//! reconstructs frames from an arbitrary byte stream.

/// C++ `frame_size_mask` / `max_frame_size`.
pub const FRAME_SIZE_MASK: u32 = 0xFFFF;
pub const MAX_FRAME_SIZE: u32 = 0xFFFF;

/// Reassembles length-prefixed frames from a byte stream (C++ `frame_queue`).
#[derive(Debug, Default)]
pub struct FrameQueue {
    buffer: Vec<u8>,
    /// The pending frame's size-and-flags word.
    next_size: u32,
}

impl FrameQueue {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            next_size: 0,
        }
    }

    /// C++ `MakeFrame(packetbuf, flags)`: `[size | flags<<16 (u32 LE)][packet]`.
    pub fn make_frame(packet: &[u8], flags: u16) -> Result<Vec<u8>, String> {
        if packet.len() > MAX_FRAME_SIZE as usize {
            return Err("Buffer exceeds maximum frame size".to_string());
        }
        let size = packet.len() as u32 | ((flags as u32) << 16);
        let mut out = Vec::with_capacity(4 + packet.len());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(packet);
        Ok(out)
    }

    /// C++ `Write(buf)`: append raw stream bytes.
    pub fn write(&mut self, buf: &[u8]) {
        self.buffer.extend_from_slice(buf);
    }

    /// C++ `PacketReady()`: `true` when a complete frame is buffered. Reads
    /// and consumes the 4-byte size prefix on first call.
    pub fn packet_ready(&mut self) -> Result<bool, String> {
        if self.next_size == 0 {
            if self.buffer.len() < 4 {
                return Ok(false);
            }
            let word = u32::from_le_bytes(self.buffer[0..4].try_into().unwrap());
            self.buffer.drain(..4);
            self.next_size = word;
            if self.next_size == 0 {
                return Err("Incorrect frame size".to_string());
            }
        }
        Ok(self.buffer.len() >= (self.next_size & FRAME_SIZE_MASK) as usize)
    }

    /// C++ `ReadPacketFlags()`: the frame's flags word (upper 16 bits).
    pub fn read_packet_flags(&self) -> u16 {
        (self.next_size >> 16) as u16
    }

    /// C++ `ReadPacket()`: consume and return the next frame payload.
    pub fn read_packet(&mut self) -> Result<Vec<u8>, String> {
        let packet_size = (self.next_size & FRAME_SIZE_MASK) as usize;
        if self.next_size == 0 || self.buffer.len() < packet_size {
            return Err("Incorrect frame size".to_string());
        }
        let payload: Vec<u8> = self.buffer.drain(..packet_size).collect();
        self.next_size = 0;
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_frame_layout() {
        let frame = FrameQueue::make_frame(b"hi", 0).unwrap();
        // size=2 LE + payload.
        assert_eq!(frame, vec![2, 0, 0, 0, b'h', b'i']);
    }

    #[test]
    fn test_make_frame_with_flags() {
        let frame = FrameQueue::make_frame(b"x", 0x1234).unwrap();
        // size(1) | flags(0x1234) << 16 = 0x12340001 LE.
        assert_eq!(&frame[0..4], &[0x01, 0x00, 0x34, 0x12]);
    }

    #[test]
    fn test_make_frame_rejects_oversize() {
        let big = vec![0u8; 0x10000];
        assert!(FrameQueue::make_frame(&big, 0).is_err());
    }

    #[test]
    fn test_frame_round_trip_in_one_write() {
        let mut q = FrameQueue::new();
        q.write(&FrameQueue::make_frame(b"packet", 0).unwrap());
        assert!(q.packet_ready().unwrap());
        assert_eq!(q.read_packet_flags(), 0);
        assert_eq!(q.read_packet().unwrap(), b"packet");
    }

    #[test]
    fn test_partial_stream_becomes_ready() {
        let mut q = FrameQueue::new();
        let frame = FrameQueue::make_frame(b"hello world", 7).unwrap();
        // Feed the 4-byte prefix + half the payload.
        q.write(&frame[..4 + 5]);
        assert!(!q.packet_ready().unwrap(), "payload incomplete");
        assert_eq!(q.read_packet_flags(), 7);
        // Feed the rest.
        q.write(&frame[4 + 5..]);
        assert!(q.packet_ready().unwrap());
        assert_eq!(q.read_packet().unwrap(), b"hello world");
    }

    #[test]
    fn test_multiple_frames_in_one_stream() {
        let mut q = FrameQueue::new();
        let f1 = FrameQueue::make_frame(b"one", 1).unwrap();
        let f2 = FrameQueue::make_frame(b"two", 2).unwrap();
        let mut stream = f1.clone();
        stream.extend_from_slice(&f2);
        q.write(&stream);
        assert!(q.packet_ready().unwrap());
        assert_eq!(q.read_packet_flags(), 1);
        assert_eq!(q.read_packet().unwrap(), b"one");
        assert!(q.packet_ready().unwrap());
        assert_eq!(q.read_packet_flags(), 2);
        assert_eq!(q.read_packet().unwrap(), b"two");
        assert!(!q.packet_ready().unwrap(), "stream drained");
    }

    #[test]
    fn test_zero_size_prefix_rejected() {
        let mut q = FrameQueue::new();
        q.write(&[0, 0, 0, 0]);
        assert!(q.packet_ready().is_err());
    }
}
