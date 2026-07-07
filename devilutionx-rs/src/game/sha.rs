//! Diablo X-SHA-1 Hash Algorithm Implementation
//!
//! This is a Rust port of Diablo's "SHA1" implementation, which differs from
//! standard SHA-1 in that it uses arithmetic right shifts (sign bit extension).
//!
//! **WARNING**: This is an intentionally flawed implementation for compatibility
//! with the original Diablo save game format. Do NOT use for actual cryptographic
//! purposes.
//!
//! # C++ Source Reference
//! - Source/sha.cpp
//! - Source/sha.h
//!
//! # Algorithm Differences from Standard SHA-1
//!
//! The "SHA1CircularShift" function in Diablo treats the input word as a signed
//! value and uses arithmetic right shifts. When the high bit is set, the upper
//! bits after rotation are filled with 1s instead of 0s.

/// Block size in 32-bit words (16 words = 64 bytes)
pub const BLOCK_SIZE: usize = 16;

/// SHA1 hash size in 32-bit words (5 words = 160 bits = 20 bytes)
pub const SHA1_HASH_SIZE: usize = 5;

/// SHA1 round constants
const K: [u32; 4] = [
    0x5A827999, // Rounds  0-19
    0x6ED9EBA1, // Rounds 20-39
    0x8F1BBCDC, // Rounds 40-59
    0xCA62C1D6, // Rounds 60-79
];

/// Initial hash values (same as standard SHA-1)
const H_INIT: [u32; SHA1_HASH_SIZE] = [
    0x67452301, // H0
    0xEFCDAB89, // H1
    0x98BADCFE, // H2
    0x10325476, // H3
    0xC3D2E1F0, // H4
];

/// Diablo X-SHA-1 context structure
///
/// Maintains the hash state and input buffer for incremental hashing.
#[derive(Debug, Clone)]
pub struct Sha1Context {
    /// Current hash state (5 x 32-bit words)
    pub state: [u32; SHA1_HASH_SIZE],
    /// Input buffer (16 x 32-bit words = 64 bytes)
    pub buffer: [u32; BLOCK_SIZE],
}

impl Default for Sha1Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha1Context {
    /// Creates a new SHA1 context with initial state
    pub fn new() -> Self {
        Self {
            state: H_INIT,
            buffer: [0; BLOCK_SIZE],
        }
    }

    /// Resets the context to initial state
    pub fn reset(&mut self) {
        self.state = H_INIT;
        self.buffer = [0; BLOCK_SIZE];
    }
}

/// Diablo-"SHA1" circular left shift, portable version.
///
/// Unlike standard SHA-1 which uses logical shifts, Diablo's implementation
/// treats the word as signed and uses arithmetic right shifts. This means
/// when the high bit is set, the vacated bits after right shift are filled
/// with 1s instead of 0s.
///
/// # Arguments
/// * `word` - The 32-bit word to rotate
/// * `bits` - Number of bits to rotate left (0-31)
///
/// # Returns
/// The rotated word with Diablo's sign-extending behavior
///
/// # C++ Reference
/// ```cpp
/// uint32_t SHA1CircularShift(uint32_t word, size_t bits)
/// {
///     if ((word & (1 << 31)) != 0)
///         return (0xFFFFFFFF << bits) | (word >> (32 - bits));
///     return (word << bits) | (word >> (32 - bits));
/// }
/// ```
fn sha1_circular_shift(word: u32, bits: u32) -> u32 {
    // Check if the sign bit (bit 31) is set
    if (word & (1 << 31)) != 0 {
        // Sign bit is set: fill high bits with 1s after rotation
        // This simulates arithmetic right shift behavior
        //
        // C++ uses unsigned shifts which are well-defined for bits < 32.
        // In practice SHA1ProcessMessageBlock only ever calls this with
        // bits == 5 or bits == 30, so `32 - bits` never reaches 0 and the
        // shift amounts stay in range. We use wrapping ops defensively so
        // the function is total (shifts by the full width are well-defined
        // as 0 in Rust via wrapping_{shl,shr}).
        (0xFFFFFFFF_u32).wrapping_shl(bits) | word.wrapping_shr(32 - bits)
    } else {
        // Sign bit is clear: standard circular left shift
        word.wrapping_shl(bits) | word.wrapping_shr(32 - bits)
    }
}

/// Processes a single 64-byte message block
///
/// This is the core SHA-1 compression function with Diablo's modifications.
///
/// # Arguments
/// * `context` - Mutable reference to the SHA1 context
///
/// # C++ Reference
/// ```cpp
/// void SHA1ProcessMessageBlock(SHA1Context *context)
/// ```
fn sha1_process_message_block(context: &mut Sha1Context) {
    // Message schedule array (80 words)
    let mut w: [u32; 80] = [0; 80];

    // Copy buffer to first 16 words
    w[..BLOCK_SIZE].copy_from_slice(&context.buffer);

    // Extend the 16 words into 80 words
    // Note: Unlike standard SHA-1, Diablo doesn't use rotation here
    for i in 16..80 {
        w[i] = w[i - 16] ^ w[i - 14] ^ w[i - 8] ^ w[i - 3];
    }

    // Initialize working variables
    let mut a = context.state[0];
    let mut b = context.state[1];
    let mut c = context.state[2];
    let mut d = context.state[3];
    let mut e = context.state[4];

    // Round 1 (0-19): f = (b & c) | ((~b) & d)
    for i in 0..20 {
        let f = (b & c) | ((!b) & d);
        let temp = sha1_circular_shift(a, 5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(w[i])
            .wrapping_add(K[0]);
        e = d;
        d = c;
        c = sha1_circular_shift(b, 30);
        b = a;
        a = temp;
    }

    // Round 2 (20-39): f = b ^ c ^ d
    for i in 20..40 {
        let f = b ^ c ^ d;
        let temp = sha1_circular_shift(a, 5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(w[i])
            .wrapping_add(K[1]);
        e = d;
        d = c;
        c = sha1_circular_shift(b, 30);
        b = a;
        a = temp;
    }

    // Round 3 (40-59): f = (b & c) | (b & d) | (c & d)
    for i in 40..60 {
        let f = (b & c) | (b & d) | (c & d);
        let temp = sha1_circular_shift(a, 5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(w[i])
            .wrapping_add(K[2]);
        e = d;
        d = c;
        c = sha1_circular_shift(b, 30);
        b = a;
        a = temp;
    }

    // Round 4 (60-79): f = b ^ c ^ d
    for i in 60..80 {
        let f = b ^ c ^ d;
        let temp = sha1_circular_shift(a, 5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(w[i])
            .wrapping_add(K[3]);
        e = d;
        d = c;
        c = sha1_circular_shift(b, 30);
        b = a;
        a = temp;
    }

    // Add working variables to state
    context.state[0] = context.state[0].wrapping_add(a);
    context.state[1] = context.state[1].wrapping_add(b);
    context.state[2] = context.state[2].wrapping_add(c);
    context.state[3] = context.state[3].wrapping_add(d);
    context.state[4] = context.state[4].wrapping_add(e);
}

/// Returns the computed hash digest
///
/// # Arguments
/// * `context` - Reference to the SHA1 context
///
/// # Returns
/// A copy of the 5-word (160-bit) hash state
///
/// # C++ Reference
/// ```cpp
/// void SHA1Result(SHA1Context &context, uint32_t messageDigest[SHA1HashSize])
/// {
///     memcpy(messageDigest, context.state, sizeof(context.state));
/// }
/// ```
pub fn sha1_result(context: &Sha1Context) -> [u32; SHA1_HASH_SIZE] {
    context.state
}

/// Copies data into buffer and processes one block
///
/// # Arguments
/// * `context` - Mutable reference to the SHA1 context
/// * `data` - The 16-word (64-byte) data block to process
///
/// # C++ Reference
/// ```cpp
/// void SHA1Calculate(SHA1Context &context, const uint32_t data[BlockSize])
/// {
///     memcpy(&context.buffer[0], data, BlockSize * sizeof(uint32_t));
///     SHA1ProcessMessageBlock(&context);
/// }
/// ```
pub fn sha1_calculate(context: &mut Sha1Context, data: &[u32; BLOCK_SIZE]) {
    context.buffer.copy_from_slice(data);
    sha1_process_message_block(context);
}

/// Convenience function to hash a single block and return the digest
///
/// # Arguments
/// * `data` - The 16-word (64-byte) data block to hash
///
/// # Returns
/// The 5-word (160-bit) hash digest
pub fn sha1_hash_block(data: &[u32; BLOCK_SIZE]) -> [u32; SHA1_HASH_SIZE] {
    let mut context = Sha1Context::new();
    sha1_calculate(&mut context, data);
    sha1_result(&context)
}

/// Hashes multiple blocks sequentially
///
/// # Arguments
/// * `blocks` - Slice of 16-word blocks to hash
///
/// # Returns
/// The 5-word (160-bit) hash digest after processing all blocks
pub fn sha1_hash_blocks(blocks: &[[u32; BLOCK_SIZE]]) -> [u32; SHA1_HASH_SIZE] {
    let mut context = Sha1Context::new();
    for block in blocks {
        sha1_calculate(&mut context, block);
    }
    sha1_result(&context)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test circular shift with positive numbers (high bit clear)
    #[test]
    fn test_circular_shift_positive() {
        // When high bit is clear, behaves like standard rotate left
        assert_eq!(sha1_circular_shift(0x00000001, 1), 0x00000002);
        assert_eq!(sha1_circular_shift(0x00000001, 4), 0x00000010);
        assert_eq!(sha1_circular_shift(0x12345678, 8), 0x34567812);

        // Edge case: shift by 0
        assert_eq!(sha1_circular_shift(0x12345678, 0), 0x12345678);
    }

    /// Test circular shift with negative numbers (high bit set)
    /// This is where Diablo's implementation differs from standard SHA-1
    #[test]
    fn test_circular_shift_negative() {
        // When high bit is set, the Diablo implementation fills high bits with 1s
        // For 0x80000000 shifted left by 5:
        //   (0xFFFFFFFF << 5) = 0xFFFFFFE0
        //   (0x80000000 >> 27) = 0x00000010
        //   OR = 0xFFFFFFF0
        let result = sha1_circular_shift(0x80000000, 5);
        assert_eq!(result, 0xFFFFFFF0);

        // Another test: 0xFFFFFFFF (all bits set)
        // Shifted left by 1: (0xFFFFFFFF << 1) | (0xFFFFFFFF >> 31) = 0xFFFFFFFF
        let result2 = sha1_circular_shift(0xFFFFFFFF, 1);
        assert_eq!(result2, 0xFFFFFFFF);
    }

    /// Test initial state matches expected values
    #[test]
    fn test_initial_state() {
        let context = Sha1Context::new();
        assert_eq!(context.state[0], 0x67452301);
        assert_eq!(context.state[1], 0xEFCDAB89);
        assert_eq!(context.state[2], 0x98BADCFE);
        assert_eq!(context.state[3], 0x10325476);
        assert_eq!(context.state[4], 0xC3D2E1F0);
    }

    /// Test hashing an all-zero block
    #[test]
    fn test_zero_block() {
        let data = [0u32; BLOCK_SIZE];
        let hash = sha1_hash_block(&data);

        // The hash should be deterministic
        let hash2 = sha1_hash_block(&data);
        assert_eq!(hash, hash2);

        // Hash should be different from initial state
        assert_ne!(hash, H_INIT);
    }

    /// Test hashing an all-ones block
    #[test]
    fn test_ones_block() {
        let data = [0xFFFFFFFF_u32; BLOCK_SIZE];
        let hash = sha1_hash_block(&data);

        // Hash should be deterministic
        let hash2 = sha1_hash_block(&data);
        assert_eq!(hash, hash2);

        // Different from zero block
        let zero_hash = sha1_hash_block(&[0u32; BLOCK_SIZE]);
        assert_ne!(hash, zero_hash);
    }

    /// Test that reset restores initial state
    #[test]
    fn test_reset() {
        let mut context = Sha1Context::new();
        let data = [0x12345678_u32; BLOCK_SIZE];
        sha1_calculate(&mut context, &data);

        // State should have changed
        assert_ne!(context.state, H_INIT);

        // Reset should restore initial state
        context.reset();
        assert_eq!(context.state, H_INIT);
        assert_eq!(context.buffer, [0; BLOCK_SIZE]);
    }

    /// Test multiple block processing
    #[test]
    fn test_multiple_blocks() {
        let block1 = [0x11111111_u32; BLOCK_SIZE];
        let block2 = [0x22222222_u32; BLOCK_SIZE];

        // Hash two blocks sequentially
        let mut context = Sha1Context::new();
        sha1_calculate(&mut context, &block1);
        let intermediate = sha1_result(&context);
        sha1_calculate(&mut context, &block2);
        let final_hash = sha1_result(&context);

        // Final hash should be different from intermediate
        assert_ne!(intermediate, final_hash);

        // Same result using convenience function
        let blocks = [block1, block2];
        let hash2 = sha1_hash_blocks(&blocks);
        assert_eq!(final_hash, hash2);
    }

    /// Test sha1_result returns a copy of state
    #[test]
    fn test_sha1_result() {
        let mut context = Sha1Context::new();
        let data = [0xDEADBEEF_u32; BLOCK_SIZE];
        sha1_calculate(&mut context, &data);

        let result = sha1_result(&context);
        assert_eq!(result, context.state);
    }

    /// Test sha1_calculate copies data to buffer
    #[test]
    fn test_sha1_calculate_buffer() {
        let mut context = Sha1Context::new();
        let data: [u32; BLOCK_SIZE] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        ];
        sha1_calculate(&mut context, &data);

        // Buffer should contain the data
        assert_eq!(context.buffer, data);
    }

    /// Test determinism - same input always produces same output
    #[test]
    fn test_determinism() {
        let data: [u32; BLOCK_SIZE] = [
            0xCAFEBABE, 0xDEADC0DE, 0xBAADF00D, 0xFEEDFACE,
            0x12345678, 0x9ABCDEF0, 0x11223344, 0x55667788,
            0x99AABBCC, 0xDDEEFF00, 0x01020304, 0x05060708,
            0x090A0B0C, 0x0D0E0F10, 0x11121314, 0x15161718,
        ];

        let hash1 = sha1_hash_block(&data);
        let hash2 = sha1_hash_block(&data);
        let hash3 = sha1_hash_block(&data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    /// Test that the Diablo circular shift produces different results than standard
    #[test]
    fn test_diablo_vs_standard_shift() {
        // Standard rotate left
        fn standard_rotate(word: u32, bits: u32) -> u32 {
            word.rotate_left(bits)
        }

        // For positive numbers (high bit clear), should be same
        let positive = 0x12345678_u32;
        assert_eq!(sha1_circular_shift(positive, 5), standard_rotate(positive, 5));

        // For negative numbers (high bit set), should be different
        let negative = 0x80000001_u32;
        let diablo_result = sha1_circular_shift(negative, 5);
        let standard_result = standard_rotate(negative, 5);
        assert_ne!(diablo_result, standard_result);
    }

    /// Test edge cases for circular shift
    #[test]
    fn test_circular_shift_edge_cases() {
        // Shift by 30 (used in SHA-1 for variable c)
        let word = 0x12345678_u32;
        let result = sha1_circular_shift(word, 30);
        // Should rotate 30 bits left = rotate 2 bits right
        assert_eq!(result, (word << 30) | (word >> 2));

        // Negative number shifted by 30
        let neg_word = 0x80000000_u32;
        let neg_result = sha1_circular_shift(neg_word, 30);
        // High bits filled with 1s
        assert_eq!(neg_result, (0xFFFFFFFF_u32 << 30) | (neg_word >> 2));
    }
}
