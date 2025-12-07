//! PKWare DCL Explode/Implode Algorithm
//!
//! This is a Rust port of the PKWare Data Compression Library's
//! explode algorithm, used in MPQ archives.
//!
//! Based on libmpq's explode.c implementation.

/// PKWare compression types
const CMP_BINARY: u8 = 0;
const CMP_ASCII: u8 = 1;

/// Distance bits table
static DIST_BITS: [u8; 64] = [
    0x02, 0x04, 0x04, 0x05, 0x05, 0x05, 0x05, 0x06,
    0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x07, 0x07,
    0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
];

/// Distance code table
static DIST_CODE: [u8; 64] = [
    0x03, 0x0D, 0x05, 0x19, 0x09, 0x11, 0x01, 0x3E,
    0x1E, 0x2E, 0x0E, 0x36, 0x16, 0x26, 0x06, 0x3A,
    0x1A, 0x2A, 0x0A, 0x32, 0x12, 0x22, 0x42, 0x02,
    0x7C, 0x3C, 0x5C, 0x1C, 0x6C, 0x2C, 0x4C, 0x0C,
    0x74, 0x34, 0x54, 0x14, 0x64, 0x24, 0x44, 0x04,
    0x78, 0x38, 0x58, 0x18, 0x68, 0x28, 0x48, 0x08,
    0xF0, 0x70, 0xB0, 0x30, 0xD0, 0x50, 0x90, 0x10,
    0xE0, 0x60, 0xA0, 0x20, 0xC0, 0x40, 0x80, 0x00,
];

/// Length bits table
static CLEN_BITS: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
];

/// Length base table
static LEN_BASE: [u16; 16] = [
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007,
    0x0008, 0x000A, 0x000E, 0x0016, 0x0026, 0x0046, 0x0086, 0x0106,
];

/// Short length bits table
static SLEN_BITS: [u8; 16] = [
    0x03, 0x02, 0x03, 0x03, 0x04, 0x04, 0x04, 0x05,
    0x05, 0x05, 0x05, 0x06, 0x06, 0x06, 0x07, 0x07,
];

/// Length code table
static LEN_CODE: [u8; 16] = [
    0x05, 0x03, 0x01, 0x06, 0x0A, 0x02, 0x0C, 0x14,
    0x04, 0x18, 0x08, 0x30, 0x10, 0x20, 0x40, 0x00,
];

/// ASCII bits table
static BITS_ASC: [u8; 256] = [
    0x0B, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x08, 0x07, 0x0C, 0x0C, 0x07, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0D, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x04, 0x0A, 0x08, 0x0C, 0x0A, 0x0C, 0x0A, 0x08,
    0x07, 0x07, 0x08, 0x09, 0x07, 0x06, 0x07, 0x08,
    0x07, 0x06, 0x07, 0x07, 0x07, 0x07, 0x08, 0x07,
    0x07, 0x08, 0x08, 0x0C, 0x0B, 0x07, 0x09, 0x0B,
    0x0C, 0x06, 0x07, 0x06, 0x06, 0x05, 0x07, 0x08,
    0x08, 0x06, 0x0B, 0x09, 0x06, 0x07, 0x06, 0x06,
    0x07, 0x0B, 0x06, 0x06, 0x06, 0x07, 0x09, 0x08,
    0x09, 0x09, 0x0B, 0x08, 0x0B, 0x09, 0x0C, 0x08,
    0x0C, 0x05, 0x06, 0x06, 0x06, 0x05, 0x06, 0x06,
    0x06, 0x05, 0x0B, 0x07, 0x05, 0x06, 0x05, 0x05,
    0x06, 0x0A, 0x05, 0x05, 0x05, 0x05, 0x08, 0x07,
    0x08, 0x08, 0x0A, 0x0B, 0x0B, 0x0C, 0x0C, 0x0C,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0D, 0x0C, 0x0D, 0x0D, 0x0D, 0x0C, 0x0D, 0x0D,
    0x0D, 0x0C, 0x0D, 0x0D, 0x0D, 0x0D, 0x0C, 0x0D,
    0x0D, 0x0D, 0x0C, 0x0C, 0x0C, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
];

/// ASCII code table (partial - first 128 entries)
static CODE_ASC: [u16; 256] = [
    0x0490, 0x0FE0, 0x07E0, 0x0BE0, 0x03E0, 0x0DE0, 0x05E0, 0x09E0,
    0x01E0, 0x00B8, 0x0062, 0x0EE0, 0x06E0, 0x0022, 0x0AE0, 0x02E0,
    0x0CE0, 0x04E0, 0x08E0, 0x00E0, 0x0F60, 0x0760, 0x0B60, 0x0360,
    0x0D60, 0x0560, 0x1240, 0x0960, 0x0160, 0x0E60, 0x0660, 0x0A60,
    0x000F, 0x0250, 0x0038, 0x0260, 0x0050, 0x0C60, 0x0390, 0x00D8,
    0x0042, 0x0002, 0x0058, 0x01B0, 0x007C, 0x0029, 0x003C, 0x0098,
    0x005C, 0x0009, 0x001C, 0x006C, 0x002C, 0x004C, 0x0018, 0x000C,
    0x0074, 0x00E8, 0x0068, 0x0460, 0x0090, 0x0034, 0x00B0, 0x0710,
    0x0860, 0x0031, 0x0054, 0x0011, 0x0021, 0x0017, 0x0014, 0x00A8,
    0x0028, 0x0001, 0x0310, 0x0130, 0x003E, 0x0064, 0x001E, 0x002E,
    0x0024, 0x0510, 0x000E, 0x0036, 0x0016, 0x0044, 0x0030, 0x00C8,
    0x01D0, 0x00D0, 0x0110, 0x0048, 0x0610, 0x0150, 0x0060, 0x0088,
    0x0FA0, 0x0007, 0x0026, 0x0006, 0x003A, 0x001B, 0x001A, 0x002A,
    0x000A, 0x000B, 0x0210, 0x0004, 0x0013, 0x0032, 0x0003, 0x001D,
    0x0012, 0x0190, 0x000D, 0x0015, 0x0005, 0x0019, 0x0008, 0x0078,
    0x00F0, 0x0070, 0x0290, 0x0410, 0x0010, 0x07A0, 0x0BA0, 0x03A0,
    0x0240, 0x1C40, 0x0C40, 0x1440, 0x0440, 0x1840, 0x0840, 0x1040,
    0x0040, 0x1F80, 0x0F80, 0x1780, 0x0780, 0x1B80, 0x0B80, 0x1380,
    0x0380, 0x1D80, 0x0D80, 0x1580, 0x0580, 0x1980, 0x0980, 0x1180,
    0x0180, 0x1E80, 0x0E80, 0x1680, 0x0680, 0x1A80, 0x0A80, 0x1280,
    0x0280, 0x1C80, 0x0C80, 0x1480, 0x0480, 0x1880, 0x0880, 0x1080,
    0x0080, 0x1F00, 0x0F00, 0x1700, 0x0700, 0x1B00, 0x0B00, 0x1300,
    0x0DA0, 0x05A0, 0x09A0, 0x01A0, 0x0EA0, 0x06A0, 0x0AA0, 0x02A0,
    0x0CA0, 0x04A0, 0x08A0, 0x00A0, 0x0F20, 0x0720, 0x0B20, 0x0320,
    0x0D20, 0x0520, 0x0920, 0x0120, 0x0E20, 0x0620, 0x0A20, 0x0220,
    0x0C20, 0x0420, 0x0820, 0x0020, 0x0FC0, 0x07C0, 0x0BC0, 0x03C0,
    0x0DC0, 0x05C0, 0x09C0, 0x01C0, 0x0EC0, 0x06C0, 0x0AC0, 0x02C0,
    0x0CC0, 0x04C0, 0x08C0, 0x00C0, 0x0F40, 0x0740, 0x0B40, 0x0340,
    0x0300, 0x0D40, 0x1D00, 0x0D00, 0x1500, 0x0540, 0x0500, 0x1900,
    0x0900, 0x0940, 0x1100, 0x0100, 0x1E00, 0x0E00, 0x0140, 0x1600,
    0x0600, 0x1A00, 0x0E40, 0x0640, 0x0A40, 0x0A00, 0x1200, 0x0200,
    0x1C00, 0x0C00, 0x1400, 0x0400, 0x1800, 0x0800, 0x1000, 0x0000,
];

/// PKWare explode state
struct ExplodeState<'a> {
    /// Input data
    input: &'a [u8],
    /// Input position
    in_pos: usize,
    /// Bit buffer
    bit_buf: u32,
    /// Extra bits in buffer
    extra_bits: u32,
    /// Compression type (binary or ASCII)
    cmp_type: u8,
    /// Dictionary size bits
    dsize_bits: u8,
    /// Dictionary size mask
    dsize_mask: u32,
    /// Output buffer
    output: Vec<u8>,
    /// Output position in circular buffer
    out_pos: usize,
    /// Decoded tables
    pos1: [u8; 256],
    pos2: [u8; 256],
    bits_asc: [u8; 256],
    offs_2c34: [u8; 256],
    offs_2d34: [u8; 256],
    offs_2e34: [u8; 128],
    offs_2eb4: [u8; 256],
}

impl<'a> ExplodeState<'a> {
    fn new(input: &'a [u8]) -> Option<Self> {
        if input.len() < 4 {
            return None;
        }

        let cmp_type = input[0];
        let dsize_bits = input[1];

        // Validate dictionary size (4-6)
        if dsize_bits < 4 || dsize_bits > 6 {
            return None;
        }

        // Validate compression type
        if cmp_type != CMP_BINARY && cmp_type != CMP_ASCII {
            return None;
        }

        let mut state = Self {
            input,
            in_pos: 3,
            bit_buf: input[2] as u32,
            extra_bits: 0,
            cmp_type,
            dsize_bits,
            dsize_mask: 0xFFFF >> (16 - dsize_bits),
            output: Vec::with_capacity(0x2000),
            out_pos: 0x1000,
            pos1: [0; 256],
            pos2: [0; 256],
            bits_asc: BITS_ASC,
            offs_2c34: [0; 256],
            offs_2d34: [0; 256],
            offs_2e34: [0; 128],
            offs_2eb4: [0; 256],
        };

        // Initialize output buffer
        state.output.resize(0x2000, 0);

        // Generate decode tables
        state.generate_tables_decode(16, &SLEN_BITS, &LEN_CODE, true);
        state.generate_tables_decode(64, &DIST_BITS, &DIST_CODE, false);

        // Generate ASCII tables if needed
        if cmp_type == CMP_ASCII {
            state.generate_tables_ascii();
        }

        Some(state)
    }

    /// Generate decode tables
    fn generate_tables_decode(&mut self, count: usize, bits: &[u8], code: &[u8], is_pos2: bool) {
        for i in (0..count).rev() {
            let mut idx1 = code[i] as usize;
            let idx2 = 1usize << bits[i];

            while idx1 < 256 {
                if is_pos2 {
                    self.pos2[idx1] = i as u8;
                } else {
                    self.pos1[idx1] = i as u8;
                }
                idx1 += idx2;
            }
        }
    }

    /// Generate ASCII decode tables
    fn generate_tables_ascii(&mut self) {
        for count in (0..=255usize).rev() {
            let bits_tmp = self.bits_asc[count];
            let code_asc = CODE_ASC[count];

            if bits_tmp <= 8 {
                let add = 1usize << bits_tmp;
                let mut acc = code_asc as usize;
                while acc < 256 {
                    self.offs_2c34[acc] = count as u8;
                    acc += add;
                }
            } else {
                let acc = (code_asc & 0xFF) as usize;
                if acc != 0 {
                    self.offs_2c34[acc] = 0xFF;
                    if (code_asc & 0x3F) != 0 {
                        let mut bits_tmp = bits_tmp - 4;
                        self.bits_asc[count] = bits_tmp;
                        let add = 1usize << bits_tmp;
                        let mut acc = (code_asc >> 4) as usize;
                        while acc < 256 {
                            self.offs_2d34[acc] = count as u8;
                            acc += add;
                        }
                    } else {
                        let mut bits_tmp = bits_tmp - 6;
                        self.bits_asc[count] = bits_tmp;
                        let add = 1usize << bits_tmp;
                        let mut acc = (code_asc >> 6) as usize;
                        while acc < 128 {
                            self.offs_2e34[acc] = count as u8;
                            acc += add;
                        }
                    }
                } else {
                    let bits_tmp = bits_tmp - 8;
                    self.bits_asc[count] = bits_tmp;
                    let add = 1usize << bits_tmp;
                    let mut acc = (code_asc >> 8) as usize;
                    while acc < 256 {
                        self.offs_2eb4[acc] = count as u8;
                        acc += add;
                    }
                }
            }
        }
    }

    /// Skip bits in the bit buffer
    fn skip_bits(&mut self, bits: u32) -> bool {
        if bits <= self.extra_bits {
            self.extra_bits -= bits;
            self.bit_buf >>= bits;
            return true;
        }

        self.bit_buf >>= self.extra_bits;

        if self.in_pos >= self.input.len() {
            return false;
        }

        self.bit_buf |= (self.input[self.in_pos] as u32) << 8;
        self.in_pos += 1;
        self.bit_buf >>= bits - self.extra_bits;
        self.extra_bits = self.extra_bits + 8 - bits;

        true
    }

    /// Decode a literal value
    fn decode_literal(&mut self) -> Option<u32> {
        // Check if the current bit is set
        if (self.bit_buf & 1) != 0 {
            // Skip current bit
            if !self.skip_bits(1) {
                return None;
            }

            // Get position in buffers
            let value = self.pos2[(self.bit_buf & 0xFF) as usize] as usize;

            // Skip bits
            if !self.skip_bits(SLEN_BITS[value] as u32) {
                return None;
            }

            // Check if we need more bits
            let bits = CLEN_BITS[value];
            if bits != 0 {
                let val2 = self.bit_buf & ((1 << bits) - 1);

                if !self.skip_bits(bits as u32) {
                    if value as u32 + val2 != 0x10E {
                        return None;
                    }
                }

                return Some(LEN_BASE[value] as u32 + val2 + 0x100);
            }

            return Some(value as u32 + 0x100);
        }

        // Skip one bit
        if !self.skip_bits(1) {
            return None;
        }

        // Binary compression
        if self.cmp_type == CMP_BINARY {
            let value = self.bit_buf & 0xFF;
            if !self.skip_bits(8) {
                return None;
            }
            return Some(value);
        }

        // ASCII compression
        let value;
        if (self.bit_buf & 0xFF) != 0 {
            value = self.offs_2c34[(self.bit_buf & 0xFF) as usize];

            if value == 0xFF {
                if (self.bit_buf & 0x3F) != 0 {
                    if !self.skip_bits(4) {
                        return None;
                    }
                    let v = self.offs_2d34[(self.bit_buf & 0xFF) as usize];
                    if !self.skip_bits(self.bits_asc[v as usize] as u32) {
                        return None;
                    }
                    return Some(v as u32);
                } else {
                    if !self.skip_bits(6) {
                        return None;
                    }
                    let v = self.offs_2e34[(self.bit_buf & 0x7F) as usize];
                    if !self.skip_bits(self.bits_asc[v as usize] as u32) {
                        return None;
                    }
                    return Some(v as u32);
                }
            }
        } else {
            if !self.skip_bits(8) {
                return None;
            }
            value = self.offs_2eb4[(self.bit_buf & 0xFF) as usize];
        }

        if !self.skip_bits(self.bits_asc[value as usize] as u32) {
            return None;
        }

        Some(value as u32)
    }

    /// Decode distance to move back
    fn decode_distance(&mut self, length: u32) -> Option<u32> {
        let pos = self.pos1[(self.bit_buf & 0xFF) as usize] as usize;
        let skip = DIST_BITS[pos];

        if !self.skip_bits(skip as u32) {
            return None;
        }

        if length == 2 {
            let pos = ((pos as u32) << 2) | (self.bit_buf & 0x03);
            if !self.skip_bits(2) {
                return None;
            }
            Some(pos + 1)
        } else {
            let pos = ((pos as u32) << self.dsize_bits) | (self.bit_buf & self.dsize_mask);
            if !self.skip_bits(self.dsize_bits as u32) {
                return None;
            }
            Some(pos + 1)
        }
    }

    /// Main decompression function
    fn expand(&mut self) -> Option<Vec<u8>> {
        let mut result = Vec::new();

        loop {
            let one_byte = self.decode_literal()?;

            if one_byte >= 0x305 {
                break;
            }

            if one_byte >= 0x100 {
                // Copy from previous output
                let copy_length = (one_byte - 0xFE) as usize;
                let move_back = self.decode_distance(copy_length as u32)? as usize;

                if move_back > self.out_pos {
                    return None;
                }

                let source_start = self.out_pos - move_back;

                for i in 0..copy_length {
                    let byte = self.output[source_start + i];
                    self.output[self.out_pos] = byte;
                    self.out_pos += 1;
                }
            } else {
                // Single byte
                self.output[self.out_pos] = one_byte as u8;
                self.out_pos += 1;
            }

            // Flush buffer if needed
            if self.out_pos >= 0x2000 {
                result.extend_from_slice(&self.output[0x1000..0x2000]);

                // Copy remaining to beginning
                self.output.copy_within(0x1000..self.out_pos, 0);
                self.out_pos -= 0x1000;
            }
        }

        // Flush remaining
        if self.out_pos > 0x1000 {
            result.extend_from_slice(&self.output[0x1000..self.out_pos]);
        }

        Some(result)
    }
}

/// Decompress PKWare DCL imploded data
///
/// # Arguments
/// * `input` - Compressed data (including 3-byte header)
/// * `unpacked_size` - Expected uncompressed size
///
/// # Returns
/// Decompressed data or None on error
pub fn explode(input: &[u8], unpacked_size: usize) -> Option<Vec<u8>> {
    let mut state = ExplodeState::new(input)?;
    let result = state.expand()?;

    // Validate size
    if result.len() != unpacked_size {
        // Allow some tolerance for padding
        if result.len() < unpacked_size {
            return None;
        }
    }

    Some(result.into_iter().take(unpacked_size).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explode_empty() {
        assert!(explode(&[], 0).is_none());
    }

    #[test]
    fn test_explode_invalid_header() {
        // Invalid dictionary size
        assert!(explode(&[0, 7, 0, 0], 10).is_none());
        // Invalid compression type
        assert!(explode(&[2, 4, 0, 0], 10).is_none());
    }
}
