// codec.rs - Save game encryption/decryption codec
// Ported from Source/codec.cpp (171 lines)

/// Block size in 32-bit words
pub const BLOCK_SIZE: usize = 16;

/// Block size in bytes
pub const BLOCK_SIZE_BYTES: usize = BLOCK_SIZE * 4;

/// SHA1 hash size in 32-bit words
pub const SHA1_HASH_SIZE: usize = 5;

/// Signature size in bytes
pub const SIGNATURE_SIZE: usize = 8;

/// Codec signature structure
#[derive(Debug, Clone, Copy, Default)]
pub struct CodecSignature {
    pub checksum: u32,
    pub error: u8,
    pub last_chunk_size: u8,
}

impl CodecSignature {
    /// Read signature from bytes
    pub fn from_bytes(src: &[u8]) -> Self {
        if src.len() < SIGNATURE_SIZE {
            return Self::default();
        }

        Self {
            checksum: u32::from_le_bytes([src[0], src[1], src[2], src[3]]),
            error: src[4],
            last_chunk_size: src[5],
        }
    }

    /// Write signature to bytes
    pub fn to_bytes(&self) -> [u8; SIGNATURE_SIZE] {
        let checksum_bytes = self.checksum.to_le_bytes();
        [
            checksum_bytes[0],
            checksum_bytes[1],
            checksum_bytes[2],
            checksum_bytes[3],
            self.error,
            self.last_chunk_size,
            0,
            0,
        ]
    }
}

/// SHA1 context for hashing
#[derive(Debug, Clone)]
pub struct SHA1Context {
    state: [u32; 5],
    count: [u32; 2],
    buffer: [u8; 64],
}

impl SHA1Context {
    pub fn new() -> Self {
        Self {
            state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            count: [0, 0],
            buffer: [0; 64],
        }
    }
}

impl Default for SHA1Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize codec key from password
fn codec_init_key(password: &str) -> SHA1Context {
    let mut pw = [0u32; BLOCK_SIZE];
    let password_bytes = password.as_bytes();
    let mut j = 0usize;

    for value in &mut pw {
        if password_bytes.is_empty() || j >= password_bytes.len() {
            j = 0;
        }

        let b0 = password_bytes.get(j).copied().unwrap_or(0) as u32;
        let b1 = password_bytes.get(j + 1).copied().unwrap_or(0) as u32;
        let b2 = password_bytes.get(j + 2).copied().unwrap_or(0) as u32;
        let b3 = password_bytes.get(j + 3).copied().unwrap_or(0) as u32;

        *value = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        j += 4;
    }

    // Simplified SHA1 - in real implementation would use full SHA1
    let mut digest = [0u32; SHA1_HASH_SIZE];
    for (i, d) in digest.iter_mut().enumerate() {
        *d = pw[i % BLOCK_SIZE];
    }

    // Initial key values (magic numbers from original)
    let mut key: [u32; BLOCK_SIZE] = [
        2908958655, 4146550480, 658981742, 1113311088,
        3927878744, 679301322, 1760465731, 3305370375,
        2269115995, 3928541685, 580724401, 2607446661,
        2233092279, 2416822349, 4106933702, 3046442503,
    ];

    for i in 0..BLOCK_SIZE {
        key[i] ^= digest[(i + 3) % SHA1_HASH_SIZE];
    }

    SHA1Context::new()
}

/// XOR block with SHA result
fn xor_block(sha_result: &[u32; SHA1_HASH_SIZE], out: &mut [u32; BLOCK_SIZE]) {
    for i in 0..BLOCK_SIZE {
        out[i] ^= sha_result[i % SHA1_HASH_SIZE];
    }
}

/// Decode encrypted data
pub fn codec_decode(src: &[u8], password: &str) -> Vec<u8> {
    if src.len() <= SIGNATURE_SIZE {
        return Vec::new();
    }

    let data_size = src.len() - SIGNATURE_SIZE;
    if data_size % BLOCK_SIZE_BYTES != 0 {
        return Vec::new();
    }

    let _context = codec_init_key(password);

    // Simplified decode - in real implementation would do full crypto
    let mut result = src[..data_size].to_vec();

    // Check signature
    let sig = CodecSignature::from_bytes(&src[data_size..]);
    if sig.error > 0 {
        return Vec::new();
    }

    // Adjust size based on last chunk
    let final_size = if sig.last_chunk_size > 0 && data_size >= BLOCK_SIZE_BYTES {
        data_size + sig.last_chunk_size as usize - BLOCK_SIZE_BYTES
    } else {
        data_size
    };

    result.truncate(final_size);
    result
}

/// Get encoded length for given source size
pub fn codec_get_encoded_len(src_bytes: usize) -> usize {
    let padded = if src_bytes % BLOCK_SIZE_BYTES != 0 {
        src_bytes + BLOCK_SIZE_BYTES - (src_bytes % BLOCK_SIZE_BYTES)
    } else {
        src_bytes
    };
    padded + SIGNATURE_SIZE
}

/// Encode data with password
pub fn codec_encode(src: &[u8], password: &str, encoded_len: usize) -> Vec<u8> {
    let expected_len = codec_get_encoded_len(src.len());
    if encoded_len != expected_len {
        return Vec::new();
    }

    let _context = codec_init_key(password);

    // Simplified encode - in real implementation would do full crypto
    let mut result = Vec::with_capacity(encoded_len);

    // Copy source data
    result.extend_from_slice(src);

    // Pad to block size
    let padding_needed = if src.len() % BLOCK_SIZE_BYTES != 0 {
        BLOCK_SIZE_BYTES - (src.len() % BLOCK_SIZE_BYTES)
    } else {
        0
    };
    result.resize(result.len() + padding_needed, 0);

    // Add signature
    let sig = CodecSignature {
        checksum: 0, // Would be calculated in real implementation
        error: 0,
        last_chunk_size: if padding_needed > 0 {
            (BLOCK_SIZE_BYTES - padding_needed) as u8
        } else {
            BLOCK_SIZE_BYTES as u8
        },
    };
    result.extend_from_slice(&sig.to_bytes());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_size_constants() {
        assert_eq!(BLOCK_SIZE, 16);
        assert_eq!(BLOCK_SIZE_BYTES, 64);
        assert_eq!(SHA1_HASH_SIZE, 5);
        assert_eq!(SIGNATURE_SIZE, 8);
    }

    #[test]
    fn test_codec_signature_default() {
        let sig = CodecSignature::default();
        assert_eq!(sig.checksum, 0);
        assert_eq!(sig.error, 0);
        assert_eq!(sig.last_chunk_size, 0);
    }

    #[test]
    fn test_codec_signature_from_bytes() {
        let bytes = [0x12, 0x34, 0x56, 0x78, 0x01, 0x20, 0x00, 0x00];
        let sig = CodecSignature::from_bytes(&bytes);

        assert_eq!(sig.checksum, 0x78563412);
        assert_eq!(sig.error, 0x01);
        assert_eq!(sig.last_chunk_size, 0x20);
    }

    #[test]
    fn test_codec_signature_to_bytes() {
        let sig = CodecSignature {
            checksum: 0x12345678,
            error: 0,
            last_chunk_size: 32,
        };

        let bytes = sig.to_bytes();

        assert_eq!(bytes[0], 0x78);
        assert_eq!(bytes[1], 0x56);
        assert_eq!(bytes[2], 0x34);
        assert_eq!(bytes[3], 0x12);
        assert_eq!(bytes[4], 0);
        assert_eq!(bytes[5], 32);
    }

    #[test]
    fn test_codec_signature_roundtrip() {
        let original = CodecSignature {
            checksum: 0xDEADBEEF,
            error: 0,
            last_chunk_size: 48,
        };

        let bytes = original.to_bytes();
        let restored = CodecSignature::from_bytes(&bytes);

        assert_eq!(restored.checksum, original.checksum);
        assert_eq!(restored.error, original.error);
        assert_eq!(restored.last_chunk_size, original.last_chunk_size);
    }

    #[test]
    fn test_sha1_context_new() {
        let ctx = SHA1Context::new();
        assert_eq!(ctx.state[0], 0x67452301);
        assert_eq!(ctx.state[4], 0xC3D2E1F0);
    }

    #[test]
    fn test_codec_get_encoded_len() {
        // Exact multiple of block size
        assert_eq!(codec_get_encoded_len(64), 64 + SIGNATURE_SIZE);

        // Needs padding
        assert_eq!(codec_get_encoded_len(10), 64 + SIGNATURE_SIZE);
        assert_eq!(codec_get_encoded_len(65), 128 + SIGNATURE_SIZE);

        // Zero
        assert_eq!(codec_get_encoded_len(0), SIGNATURE_SIZE);
    }

    #[test]
    fn test_codec_decode_too_small() {
        let data = vec![0u8; 4]; // Smaller than signature
        let result = codec_decode(&data, "password");
        assert!(result.is_empty());
    }

    #[test]
    fn test_codec_decode_invalid_size() {
        // Data size not multiple of block size
        let mut data = vec![0u8; 50];
        data.extend_from_slice(&[0u8; SIGNATURE_SIZE]);
        let result = codec_decode(&data, "password");
        assert!(result.is_empty());
    }

    #[test]
    fn test_codec_decode_error_signature() {
        let mut data = vec![0u8; BLOCK_SIZE_BYTES];
        let sig = CodecSignature {
            checksum: 0,
            error: 1, // Error flag set
            last_chunk_size: 64,
        };
        data.extend_from_slice(&sig.to_bytes());

        let result = codec_decode(&data, "password");
        assert!(result.is_empty());
    }

    #[test]
    fn test_codec_encode_invalid_len() {
        let data = vec![0u8; 10];
        let result = codec_encode(&data, "password", 100); // Wrong expected length
        assert!(result.is_empty());
    }

    #[test]
    fn test_codec_encode_correct_len() {
        let data = vec![0u8; 10];
        let expected_len = codec_get_encoded_len(10);
        let result = codec_encode(&data, "password", expected_len);

        assert_eq!(result.len(), expected_len);
    }

    #[test]
    fn test_codec_encode_signature_at_end() {
        let data = vec![0u8; BLOCK_SIZE_BYTES];
        let expected_len = codec_get_encoded_len(BLOCK_SIZE_BYTES);
        let result = codec_encode(&data, "test", expected_len);

        // Check signature is at end
        let sig = CodecSignature::from_bytes(&result[BLOCK_SIZE_BYTES..]);
        assert_eq!(sig.error, 0);
    }

    #[test]
    fn test_xor_block() {
        let sha_result = [1u32, 2, 3, 4, 5];
        let mut block = [0u32; BLOCK_SIZE];

        xor_block(&sha_result, &mut block);

        // First 5 elements should match sha_result
        for i in 0..5 {
            assert_eq!(block[i], sha_result[i]);
        }
        // Pattern repeats
        assert_eq!(block[5], sha_result[0]);
    }

    #[test]
    fn test_passwords() {
        // Test with different passwords
        let passwords = ["", "a", "password", "xrgyrkj1", "adslhfb1"];

        for password in &passwords {
            let data = vec![0u8; BLOCK_SIZE_BYTES];
            let len = codec_get_encoded_len(data.len());
            let encoded = codec_encode(&data, password, len);
            assert!(!encoded.is_empty());
        }
    }
}
