// codec.rs - Save game encryption/decryption codec
// Ported from Source/codec.cpp (171 lines) + Source/sha.cpp (X-SHA-1).
//
// Diablo's "SHA1" is NOT standard SHA-1: it uses arithmetic (sign-extending)
// right shifts in the circular-shift step (Source/sha.cpp note). The codec
// XORs 64-byte blocks with the running X-SHA1 digest of the plaintext and
// appends an 8-byte signature { checksum, error, lastChunkSize, 0, 0 }.

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

/// Diablo X-SHA1 context (Source/sha.h): 5-word state + 16-word block buffer.
#[derive(Debug, Clone)]
pub struct SHA1Context {
    state: [u32; SHA1_HASH_SIZE],
    buffer: [u32; BLOCK_SIZE],
}

impl SHA1Context {
    pub fn new() -> Self {
        Self {
            state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            buffer: [0; BLOCK_SIZE],
        }
    }

    /// Diablo-"SHA1" circular left shift: when the high bit is set, the shift
    /// sign-extends (arithmetic right shift), filling the high bits with 1s.
    fn circular_shift(word: u32, bits: u32) -> u32 {
        if word & (1 << 31) != 0 {
            (0xFFFFFFFFu32 << bits) | (word >> (32 - bits))
        } else {
            (word << bits) | (word >> (32 - bits))
        }
    }

    /// C++ `SHA1ProcessMessageBlock`.
    fn process_message_block(&mut self) {
        let mut w = [0u32; 80];
        w[..BLOCK_SIZE].copy_from_slice(&self.buffer);
        for i in 16..80 {
            w[i] = w[i - 16] ^ w[i - 14] ^ w[i - 8] ^ w[i - 3];
        }

        let (mut a, mut b, mut c, mut d, mut e) = (
            self.state[0],
            self.state[1],
            self.state[2],
            self.state[3],
            self.state[4],
        );

        for i in 0..20 {
            let temp = Self::circular_shift(a, 5)
                .wrapping_add((b & c) | ((!b) & d))
                .wrapping_add(e)
                .wrapping_add(w[i])
                .wrapping_add(0x5A827999);
            e = d;
            d = c;
            c = Self::circular_shift(b, 30);
            b = a;
            a = temp;
        }
        for i in 20..40 {
            let temp = Self::circular_shift(a, 5)
                .wrapping_add(b ^ c ^ d)
                .wrapping_add(e)
                .wrapping_add(w[i])
                .wrapping_add(0x6ED9EBA1);
            e = d;
            d = c;
            c = Self::circular_shift(b, 30);
            b = a;
            a = temp;
        }
        for i in 40..60 {
            let temp = Self::circular_shift(a, 5)
                .wrapping_add((b & c) | (b & d) | (c & d))
                .wrapping_add(e)
                .wrapping_add(w[i])
                .wrapping_add(0x8F1BBCDC);
            e = d;
            d = c;
            c = Self::circular_shift(b, 30);
            b = a;
            a = temp;
        }
        for i in 60..80 {
            let temp = Self::circular_shift(a, 5)
                .wrapping_add(b ^ c ^ d)
                .wrapping_add(e)
                .wrapping_add(w[i])
                .wrapping_add(0xCA62C1D6);
            e = d;
            d = c;
            c = Self::circular_shift(b, 30);
            b = a;
            a = temp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }

    /// C++ `SHA1Calculate`: load one 16-word block and process it.
    fn calculate(&mut self, data: &[u32; BLOCK_SIZE]) {
        self.buffer.copy_from_slice(data);
        self.process_message_block();
    }

    /// C++ `SHA1Result`: copy the current message digest.
    fn result(&self) -> [u32; SHA1_HASH_SIZE] {
        self.state
    }
}

impl Default for SHA1Context {
    fn default() -> Self {
        Self::new()
    }
}

/// C++ `CodecInitKey`: expand the password into 16 LE words (repeating),
/// X-SHA1 them, XOR the magic key table with the digest, then X-SHA1 the key.
fn codec_init_key(password: &str) -> SHA1Context {
    let bytes = password.as_bytes();
    let mut pw = [0u32; BLOCK_SIZE];
    let mut j = 0usize;
    for value in &mut pw {
        // C++: `if (pszPassword[j] == '\0') j = 0;` — wrap at the NUL/end.
        if j >= bytes.len() || bytes[j] == 0 {
            j = 0;
        }
        let b0 = bytes.get(j).copied().unwrap_or(0) as u32;
        let b1 = bytes.get(j + 1).copied().unwrap_or(0) as u32;
        let b2 = bytes.get(j + 2).copied().unwrap_or(0) as u32;
        let b3 = bytes.get(j + 3).copied().unwrap_or(0) as u32;
        *value = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        j += 4;
    }

    let mut context = SHA1Context::new();
    context.calculate(&pw);
    let digest = context.result();

    // Magic key table (Source/codec.cpp CodecInitKey).
    let mut key: [u32; BLOCK_SIZE] = [
        2908958655, 4146550480, 658981742, 1113311088,
        3927878744, 679301322, 1760465731, 3305370375,
        2269115995, 3928541685, 580724401, 2607446661,
        2233092279, 2416822349, 4106933702, 3046442503,
    ];
    for i in 0..BLOCK_SIZE {
        key[i] ^= digest[(i + 3) % SHA1_HASH_SIZE];
    }
    // C++ CodecInitKey uses a *fresh* context for the key X-SHA1 (the pw
    // digest was read from a scoped context), so do not reuse `context`.
    let mut context = SHA1Context::new();
    context.calculate(&key);
    context
}

/// C++ `ByteSwapBlock` (`Swap32LE` per word): identity on little-endian hosts.
fn byte_swap_block(buf: &mut [u32; BLOCK_SIZE]) {
    for w in buf.iter_mut() {
        *w = w.to_le();
    }
}

/// XOR block with the X-SHA1 digest (repeating every 5 words).
fn xor_block(sha_result: &[u32; SHA1_HASH_SIZE], out: &mut [u32; BLOCK_SIZE]) {
    for i in 0..BLOCK_SIZE {
        out[i] ^= sha_result[i % SHA1_HASH_SIZE];
    }
}

/// Decode codec-encrypted data (C++ `codec_decode`).
pub fn codec_decode(src: &[u8], password: &str) -> Vec<u8> {
    if src.len() <= SIGNATURE_SIZE {
        return Vec::new();
    }

    let mut size = src.len() - SIGNATURE_SIZE;
    if size % BLOCK_SIZE_BYTES != 0 {
        return Vec::new();
    }

    let mut context = codec_init_key(password);
    let mut result = vec![0u8; size];

    for offset in (0..size).step_by(BLOCK_SIZE_BYTES) {
        let mut buf = [0u32; BLOCK_SIZE];
        for (k, chunk) in buf.iter_mut().enumerate() {
            let b = offset + k * 4;
            *chunk = u32::from_le_bytes([src[b], src[b + 1], src[b + 2], src[b + 3]]);
        }
        byte_swap_block(&mut buf);
        let dst = context.result();
        xor_block(&dst, &mut buf);
        context.calculate(&buf);
        byte_swap_block(&mut buf);
        for (k, chunk) in buf.iter().enumerate() {
            let b = offset + k * 4;
            result[b..b + 4].copy_from_slice(&chunk.to_le_bytes());
        }
    }

    let sig = CodecSignature::from_bytes(&src[src.len() - SIGNATURE_SIZE..]);
    if sig.error > 0 {
        return Vec::new();
    }
    let dst = context.result();
    if sig.checksum != dst[0] {
        return Vec::new();
    }

    size = size
        .wrapping_add(sig.last_chunk_size as usize)
        .wrapping_sub(BLOCK_SIZE_BYTES);
    result.truncate(size);
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

/// Encode data with password (C++ `codec_encode`).
pub fn codec_encode(src: &[u8], password: &str, encoded_len: usize) -> Vec<u8> {
    let expected_len = codec_get_encoded_len(src.len());
    if encoded_len != expected_len {
        return Vec::new();
    }

    let mut context = codec_init_key(password);
    let mut result = Vec::with_capacity(encoded_len);

    let mut last_chunk = 0usize;
    let mut size = src.len();
    let mut offset = 0usize;
    while size != 0 {
        let chunk = size.min(BLOCK_SIZE_BYTES);
        let mut buf = [0u32; BLOCK_SIZE];
        for (k, w) in buf.iter_mut().enumerate() {
            let b = offset + k * 4;
            let mut bytes = [0u8; 4];
            if b < src.len() {
                let take = (src.len() - b).min(4);
                bytes[..take].copy_from_slice(&src[b..b + take]);
            }
            *w = u32::from_le_bytes(bytes);
        }
        byte_swap_block(&mut buf);
        let dst = context.result();
        context.calculate(&buf);
        xor_block(&dst, &mut buf);
        byte_swap_block(&mut buf);
        for w in buf.iter() {
            result.extend_from_slice(&w.to_le_bytes());
        }
        offset += BLOCK_SIZE_BYTES;
        last_chunk = chunk;
        size -= chunk;
    }

    let dst = context.result();
    let sig = CodecSignature {
        checksum: dst[0],
        error: 0,
        last_chunk_size: last_chunk as u8,
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

    /// CodecInitKey cross-check (C++ harness for password "adslhfb1"):
    /// pw = alternating 6c736461/31626668, digest = c4129b81 27b2309f
    /// 908e0030 fe910bcb cc23a01a, final context = 6c4e23ef b121e35a
    /// 855349f8 0820d579 a71de5b3.
    #[test]
    fn test_codec_init_key_matches_cpp() {
        let ctx = codec_init_key("adslhfb1");
        assert_eq!(
            ctx.result(),
            [0x6c4e23ef, 0xb121e35a, 0x855349f8, 0x0820d579, 0xa71de5b3],
            "codec_init_key(adslhfb1) context must match C++"
        );
    }

    /// X-SHA1 primitive check against the C++ sha.cpp (Source/sha.cpp):
    /// SHA1Calculate of the block [0..15] yields `0af5b05b 7509ce92 5e38ee65
    /// 1821225c a4804d11` (verified with the C++ harness). If this fails the
    /// arithmetic-right-shift circular shift or block schedule diverged.
    #[test]
    fn test_x_sha1_matches_cpp() {
        let mut ctx = SHA1Context::new();
        let data: [u32; BLOCK_SIZE] = std::array::from_fn(|i| i as u32);
        ctx.calculate(&data);
        let digest = ctx.result();
        assert_eq!(
            digest,
            [0x0af5b05b, 0x7509ce92, 0x5e38ee65, 0x1821225c, 0xa4804d11],
            "X-SHA1 of [0..15] must match C++ sha.cpp"
        );
    }

    /// Round-trip: encode then decode must recover the plaintext exactly.
    #[test]
    fn test_codec_roundtrip_recovers_plaintext() {
        for password in ["xrgyrkj1", "adslhfb1", "szqnlsk1", "lshbkfg1"] {
            for len in [0usize, 1, 63, 64, 65, 100, 1024, 230840] {
                let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
                let encoded_len = codec_get_encoded_len(len);
                let encoded = codec_encode(&data, password, encoded_len);
                assert_eq!(encoded.len(), encoded_len);
                let decoded = codec_decode(&encoded, password);
                assert_eq!(decoded, data, "roundtrip for password {} len {}", password, len);
            }
        }
    }
}
