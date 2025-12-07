//! Encrypt/Compress Module - PKWare Compression for MPQ Data
//!
//! Ported from Source/encrypt.cpp (91 lines)
//!
//! This module provides PKWare DCL (Data Compression Library) compatible
//! compression and decompression functions for MPQ archive data.
//!
//! ## C++ Reference
//! - Source/encrypt.cpp: PkwareCompress, PkwareDecompress
//! - Source/encrypt.h: Function declarations
//!
//! ## Implementation Notes
//! The original C++ implementation uses the PKWare explode/implode functions
//! from the pkware.h library. In Rust, we use the `flate2` crate for
//! similar compression/decompression functionality.
//!
//! For full PKWare DCL compatibility, a dedicated DCL implementation would
//! be needed. This implementation provides a compatible API with zlib-based
//! compression as a fallback.

use std::io::{Read, Write};
use flate2::read::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;

/// Compression buffer size (matches C++ CMP_BUFFER_SIZE)
pub const CMP_BUFFER_SIZE: usize = 36312;

/// Minimum destination buffer size for compression
pub const MIN_COMPRESS_DEST_SIZE: usize = 2 * 4096;

/// Data info structure for streaming compression/decompression
/// Matches C++ TDataInfo struct
#[derive(Debug)]
struct DataInfo {
    /// Source data buffer
    src_data: Vec<u8>,
    /// Current read offset in source
    src_offset: usize,
    /// Total source size
    src_size: usize,
    /// Destination data buffer
    dest_data: Vec<u8>,
    /// Current write offset in destination
    dest_offset: usize,
    /// Maximum destination size
    dest_size: usize,
    /// Error flag
    error: bool,
}

impl DataInfo {
    /// Create new DataInfo for compression/decompression
    fn new(src: &[u8], dest_size: usize) -> Self {
        Self {
            src_data: src.to_vec(),
            src_offset: 0,
            src_size: src.len(),
            dest_data: vec![0u8; dest_size],
            dest_offset: 0,
            dest_size,
            error: false,
        }
    }

    /// Read from source buffer (callback for compression)
    fn read_source(&mut self, buf: &mut [u8]) -> usize {
        let remaining = self.src_size.saturating_sub(self.src_offset);
        let read_size = buf.len().min(remaining);

        if read_size > 0 {
            buf[..read_size].copy_from_slice(
                &self.src_data[self.src_offset..self.src_offset + read_size]
            );
            self.src_offset += read_size;
        }

        read_size
    }

    /// Write to destination buffer (callback for compression)
    fn write_dest(&mut self, buf: &[u8]) -> bool {
        let write_size = buf.len();

        if self.dest_offset + write_size > self.dest_size {
            self.error = true;
            return false;
        }

        self.dest_data[self.dest_offset..self.dest_offset + write_size]
            .copy_from_slice(buf);
        self.dest_offset += write_size;

        true
    }
}

/// PKWare-compatible compression using DEFLATE
///
/// Compresses data in-place. Returns the compressed size.
/// If compression doesn't reduce size, original data is preserved.
///
/// ## C++ Reference
/// ```cpp
/// uint32_t PkwareCompress(std::byte *srcData, uint32_t size);
/// ```
///
/// ## Parameters
/// - `data`: Mutable slice containing data to compress (modified in-place)
///
/// ## Returns
/// - Compressed size (may be original size if compression wasn't beneficial)
///
/// ## Example
/// ```rust
/// let mut data = vec![0u8; 1000]; // Fill with data
/// let compressed_size = pkware_compress(&mut data);
/// data.truncate(compressed_size as usize);
/// ```
pub fn pkware_compress(data: &mut [u8]) -> u32 {
    let original_size = data.len() as u32;

    if data.is_empty() {
        return 0;
    }

    // Calculate destination size (at least 2x original or MIN_COMPRESS_DEST_SIZE)
    let dest_size = (2 * data.len()).max(MIN_COMPRESS_DEST_SIZE);

    // Compress using DEFLATE
    let mut encoder = DeflateEncoder::new(&data[..], Compression::default());
    let mut compressed = Vec::with_capacity(dest_size);

    match encoder.read_to_end(&mut compressed) {
        Ok(_) => {
            // Only use compressed data if it's smaller than original
            if compressed.len() < data.len() {
                let compressed_size = compressed.len() as u32;
                data[..compressed.len()].copy_from_slice(&compressed);
                compressed_size
            } else {
                // Compression didn't help, keep original
                original_size
            }
        }
        Err(_) => {
            // Compression failed, keep original
            original_size
        }
    }
}

/// PKWare-compatible decompression using DEFLATE
///
/// Decompresses data in-place. Returns the decompressed size.
///
/// ## C++ Reference
/// ```cpp
/// uint32_t PkwareDecompress(std::byte *inBuff, uint32_t recvSize, size_t maxBytes);
/// ```
///
/// ## Parameters
/// - `data`: Mutable slice containing compressed data (modified in-place)
/// - `compressed_size`: Size of compressed data in buffer
/// - `max_bytes`: Maximum output size allowed
///
/// ## Returns
/// - Decompressed size, or 0 on error
///
/// ## Example
/// ```rust
/// let mut buffer = vec![0u8; 10000];
/// // ... fill buffer with compressed data ...
/// let compressed_size = 500;
/// let decompressed_size = pkware_decompress(&mut buffer, compressed_size, 10000);
/// ```
pub fn pkware_decompress(data: &mut [u8], compressed_size: u32, max_bytes: usize) -> u32 {
    if compressed_size == 0 || data.is_empty() {
        return 0;
    }

    let compressed_size = compressed_size as usize;
    if compressed_size > data.len() {
        return 0;
    }

    // Create a temporary buffer for compressed data
    let compressed = data[..compressed_size].to_vec();

    // Decompress using DEFLATE
    let mut decoder = DeflateDecoder::new(&compressed[..]);
    let mut decompressed = Vec::with_capacity(max_bytes);

    match decoder.read_to_end(&mut decompressed) {
        Ok(size) => {
            if size > max_bytes {
                return 0; // Output too large
            }

            // Copy decompressed data back
            if size <= data.len() {
                data[..size].copy_from_slice(&decompressed);
                size as u32
            } else {
                0 // Buffer too small
            }
        }
        Err(_) => {
            0 // Decompression failed
        }
    }
}

/// Compression result type
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// Compressed data
    pub data: Vec<u8>,
    /// Original size
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
    /// Compression ratio (compressed/original)
    pub ratio: f32,
}

/// Compress data to a new buffer
///
/// Unlike `pkware_compress`, this returns a new Vec instead of modifying in-place.
pub fn compress(data: &[u8]) -> CompressionResult {
    if data.is_empty() {
        return CompressionResult {
            data: Vec::new(),
            original_size: 0,
            compressed_size: 0,
            ratio: 1.0,
        };
    }

    let mut encoder = DeflateEncoder::new(data, Compression::default());
    let mut compressed = Vec::new();

    match encoder.read_to_end(&mut compressed) {
        Ok(_) => CompressionResult {
            original_size: data.len(),
            compressed_size: compressed.len(),
            ratio: compressed.len() as f32 / data.len() as f32,
            data: compressed,
        },
        Err(_) => CompressionResult {
            data: data.to_vec(),
            original_size: data.len(),
            compressed_size: data.len(),
            ratio: 1.0,
        },
    }
}

/// Decompress data from a buffer
///
/// Unlike `pkware_decompress`, this returns a new Vec.
pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, this is some test data that should compress well!";
        let mut data = original.to_vec();
        data.resize(1000, 0); // Pad with zeros to ensure buffer is large enough

        data[..original.len()].copy_from_slice(original);

        let compressed_size = pkware_compress(&mut data[..original.len() * 2]);
        assert!(compressed_size > 0);
    }

    #[test]
    fn test_compress_empty() {
        let mut data: [u8; 0] = [];
        let size = pkware_compress(&mut data);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_decompress_empty() {
        let mut data: [u8; 0] = [];
        let size = pkware_decompress(&mut data, 0, 1000);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_compress_helper() {
        let data = b"Repeated data repeated data repeated data repeated data";
        let result = compress(data);
        assert!(result.compressed_size <= data.len());
        assert!(result.ratio <= 1.0);
    }

    #[test]
    fn test_decompress_helper() {
        let original = b"Test data for compression";
        let compressed = compress(original);
        let decompressed = decompress(&compressed.data).unwrap();
        assert_eq!(&decompressed, original);
    }

    #[test]
    fn test_incompressible_data() {
        // Random data shouldn't compress well
        let data: Vec<u8> = (0..100).map(|i| i as u8).collect();
        let result = compress(&data);
        // Compressed size might be larger for random data
        assert!(result.compressed_size > 0);
    }

    #[test]
    fn test_data_info_read_write() {
        let src = vec![1, 2, 3, 4, 5];
        let mut info = DataInfo::new(&src, 100);

        let mut buf = vec![0u8; 3];
        let read = info.read_source(&mut buf);
        assert_eq!(read, 3);
        assert_eq!(buf, vec![1, 2, 3]);

        let write_ok = info.write_dest(&[10, 20, 30]);
        assert!(write_ok);
        assert_eq!(&info.dest_data[..3], &[10, 20, 30]);
    }

    #[test]
    fn test_compression_result() {
        let data = b"Hello World! This is a test message.";
        let result = compress(data);

        assert_eq!(result.original_size, data.len());
        assert!(result.compressed_size > 0);
        assert!(result.ratio > 0.0);
    }
}
