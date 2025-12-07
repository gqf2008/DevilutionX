//! MPQ Archive Reader
//!
//! Port of the MPQ (MoPaQ) archive format reader from DevilutionX.
//! MPQ is the archive format used by Blizzard games including Diablo.
//!
//! Reference: https://github.com/savagesteel/d1-file-formats/blob/master/PC-Mac/MPQ.md

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::{Path, PathBuf};

/// MPQ file header signature
const MPQ_SIGNATURE: u32 = 0x1A51504D; // "MPQ\x1A"

/// MPQ file header size for Diablo
const MPQ_HEADER_SIZE: u32 = 32;

/// Block entry flags
const BLOCK_FLAG_EXISTS: u32 = 0x80000000;
const BLOCK_FLAG_COMPRESSED_PKZIP: u32 = 0x00000100;
const BLOCK_FLAG_COMPRESSED_MULTI: u32 = 0x00000200;
const BLOCK_FLAG_SINGLE_UNIT: u32 = 0x01000000;
const BLOCK_FLAG_KEY: u32 = 0x00020000;
const BLOCK_FLAG_KEY_ADJUSTED: u32 = 0x00010000;

/// Special block values
const BLOCK_NULL: u32 = 0xFFFFFFFF;
const BLOCK_DELETED: u32 = 0xFFFFFFFE;

/// Hash types for encryption
const HASH_TYPE_TABLE_OFFSET: u32 = 0;
const HASH_TYPE_NAME_A: u32 = 1;
const HASH_TYPE_NAME_B: u32 = 2;
const HASH_TYPE_FILE_KEY: u32 = 3;

/// MPQ File Header
#[derive(Debug, Clone)]
pub struct MpqHeader {
    /// Signature (always MPQ_SIGNATURE for Diablo MPQs)
    pub signature: u32,
    /// Header size (always 32 for Diablo MPQs)
    pub header_size: u32,
    /// Archive size
    pub archive_size: u32,
    /// Format version (0 for Diablo)
    pub format_version: u16,
    /// Block size factor (block_size = 512 << block_size_factor)
    pub block_size_factor: u16,
    /// Offset to hash table
    pub hash_table_offset: u32,
    /// Offset to block table
    pub block_table_offset: u32,
    /// Number of hash entries
    pub hash_table_size: u32,
    /// Number of block entries
    pub block_table_size: u32,
}

impl MpqHeader {
    /// Read MPQ header from file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MpqError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;

        let signature = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if signature != MPQ_SIGNATURE {
            return Err(MpqError::InvalidSignature);
        }

        Ok(Self {
            signature,
            header_size: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            archive_size: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            format_version: u16::from_le_bytes([buf[12], buf[13]]),
            block_size_factor: u16::from_le_bytes([buf[14], buf[15]]),
            hash_table_offset: u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]),
            block_table_offset: u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]),
            hash_table_size: u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]),
            block_table_size: u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]),
        })
    }

    /// Get block size from factor
    pub fn block_size(&self) -> u32 {
        512 << self.block_size_factor
    }
}

/// MPQ Hash Table Entry
#[derive(Debug, Clone, Copy)]
pub struct MpqHashEntry {
    /// Hash A (for collision resolution)
    pub hash_a: u32,
    /// Hash B (for collision resolution)
    pub hash_b: u32,
    /// Locale (always 0 in Diablo)
    pub locale: u16,
    /// Platform (always 0 in Diablo)
    pub platform: u16,
    /// Block index or special value
    pub block_index: u32,
}

impl MpqHashEntry {
    /// Read hash entry from buffer
    pub fn from_bytes(buf: &[u8; 16]) -> Self {
        Self {
            hash_a: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
            hash_b: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            locale: u16::from_le_bytes([buf[8], buf[9]]),
            platform: u16::from_le_bytes([buf[10], buf[11]]),
            block_index: u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
        }
    }

    /// Check if entry is empty
    pub fn is_empty(&self) -> bool {
        self.block_index == BLOCK_NULL
    }

    /// Check if entry is deleted
    pub fn is_deleted(&self) -> bool {
        self.block_index == BLOCK_DELETED
    }
}

/// MPQ Block Table Entry
#[derive(Debug, Clone, Copy)]
pub struct MpqBlockEntry {
    /// Offset of file data
    pub offset: u32,
    /// Compressed file size
    pub packed_size: u32,
    /// Uncompressed file size
    pub unpacked_size: u32,
    /// Flags
    pub flags: u32,
}

impl MpqBlockEntry {
    /// Read block entry from buffer
    pub fn from_bytes(buf: &[u8; 16]) -> Self {
        Self {
            offset: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
            packed_size: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            unpacked_size: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            flags: u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
        }
    }

    /// Check if file exists
    pub fn exists(&self) -> bool {
        (self.flags & BLOCK_FLAG_EXISTS) != 0
    }

    /// Check if file is compressed with PKZip
    pub fn is_pkzip_compressed(&self) -> bool {
        (self.flags & BLOCK_FLAG_COMPRESSED_PKZIP) != 0
    }

    /// Check if file uses multi-compression
    pub fn is_multi_compressed(&self) -> bool {
        (self.flags & BLOCK_FLAG_COMPRESSED_MULTI) != 0
    }

    /// Check if file is encrypted
    pub fn is_encrypted(&self) -> bool {
        (self.flags & BLOCK_FLAG_KEY) != 0
    }

    /// Check if file is stored as a single unit
    pub fn is_single_unit(&self) -> bool {
        (self.flags & BLOCK_FLAG_SINGLE_UNIT) != 0
    }
}

/// MPQ Error types
#[derive(Debug)]
pub enum MpqError {
    Io(std::io::Error),
    InvalidSignature,
    FileNotFound(String),
    InvalidHash,
    DecompressionError(String),
    EncryptedFile,
}

impl From<std::io::Error> for MpqError {
    fn from(e: std::io::Error) -> Self {
        MpqError::Io(e)
    }
}

impl std::fmt::Display for MpqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpqError::Io(e) => write!(f, "IO error: {}", e),
            MpqError::InvalidSignature => write!(f, "Invalid MPQ signature"),
            MpqError::FileNotFound(name) => write!(f, "File not found: {}", name),
            MpqError::InvalidHash => write!(f, "Invalid hash"),
            MpqError::DecompressionError(msg) => write!(f, "Decompression error: {}", msg),
            MpqError::EncryptedFile => write!(f, "Encrypted files not supported"),
        }
    }
}

impl std::error::Error for MpqError {}

/// Encryption table for MPQ hashing
static CRYPT_TABLE: std::sync::LazyLock<[u32; 0x500]> = std::sync::LazyLock::new(|| {
    let mut table = [0u32; 0x500];
    let mut seed: u32 = 0x00100001;

    for i in 0..0x100 {
        let mut index = i;
        for _ in 0..5 {
            seed = (seed * 125 + 3) % 0x2AAAAB;
            let temp1 = (seed & 0xFFFF) << 16;
            seed = (seed * 125 + 3) % 0x2AAAAB;
            let temp2 = seed & 0xFFFF;
            table[index] = temp1 | temp2;
            index += 0x100;
        }
    }

    table
});

/// Calculate hash for filename
pub fn hash_string(name: &str, hash_type: u32) -> u32 {
    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;

    for ch in name.bytes() {
        // Convert to uppercase and handle backslash
        let ch = if ch == b'/' { b'\\' } else { ch.to_ascii_uppercase() };
        let table_index = (hash_type << 8) + ch as u32;
        seed1 = CRYPT_TABLE[table_index as usize] ^ (seed1.wrapping_add(seed2));
        seed2 = (ch as u32)
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }

    seed1
}

/// Calculate file hash (3 components)
pub fn calculate_file_hash(filename: &str) -> (u32, u32, u32) {
    (
        hash_string(filename, HASH_TYPE_TABLE_OFFSET),
        hash_string(filename, HASH_TYPE_NAME_A),
        hash_string(filename, HASH_TYPE_NAME_B),
    )
}

/// Decrypt data block
pub fn decrypt_block(data: &mut [u32], key: u32) {
    let mut seed1 = key;
    let mut seed2: u32 = 0xEEEEEEEE;

    for value in data.iter_mut() {
        let table_index = 0x400 + (seed1 & 0xFF);
        seed2 = seed2.wrapping_add(CRYPT_TABLE[table_index as usize]);
        let ch = *value ^ (seed1.wrapping_add(seed2));
        seed1 = ((!seed1 << 21).wrapping_add(0x11111111)) | (seed1 >> 11);
        seed2 = ch
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
        *value = ch;
    }
}

/// Compression type flags for multi-compression
const COMPRESSION_HUFFMAN: u8 = 0x01;
const COMPRESSION_ZLIB: u8 = 0x02;
const COMPRESSION_PKZIP: u8 = 0x08;
const COMPRESSION_BZIP2: u8 = 0x10;
const COMPRESSION_WAVE_MONO: u8 = 0x40;
const COMPRESSION_WAVE_STEREO: u8 = 0x80;

/// MPQ Archive
pub struct MpqArchive {
    /// File path
    path: PathBuf,
    /// File handle
    file: BufReader<File>,
    /// Header
    header: MpqHeader,
    /// Hash table
    hash_table: Vec<MpqHashEntry>,
    /// Block table
    block_table: Vec<MpqBlockEntry>,
}

impl MpqArchive {
    /// Open an MPQ archive
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MpqError> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mut file = BufReader::new(file);

        // Read header
        let header = MpqHeader::read(&mut file)?;

        // Read and decrypt hash table
        file.seek(SeekFrom::Start(header.hash_table_offset as u64))?;
        let hash_table_bytes = header.hash_table_size as usize * 16;
        let mut hash_data = vec![0u8; hash_table_bytes];
        file.read_exact(&mut hash_data)?;

        // Decrypt hash table
        let mut hash_u32: Vec<u32> = hash_data
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        decrypt_block(&mut hash_u32, hash_string("(hash table)", HASH_TYPE_FILE_KEY));

        // Parse hash entries
        let hash_table: Vec<MpqHashEntry> = hash_u32
            .chunks_exact(4)
            .map(|chunk| {
                let bytes: [u8; 16] = [
                    chunk[0].to_le_bytes()[0], chunk[0].to_le_bytes()[1], chunk[0].to_le_bytes()[2], chunk[0].to_le_bytes()[3],
                    chunk[1].to_le_bytes()[0], chunk[1].to_le_bytes()[1], chunk[1].to_le_bytes()[2], chunk[1].to_le_bytes()[3],
                    chunk[2].to_le_bytes()[0], chunk[2].to_le_bytes()[1], chunk[2].to_le_bytes()[2], chunk[2].to_le_bytes()[3],
                    chunk[3].to_le_bytes()[0], chunk[3].to_le_bytes()[1], chunk[3].to_le_bytes()[2], chunk[3].to_le_bytes()[3],
                ];
                MpqHashEntry::from_bytes(&bytes)
            })
            .collect();

        // Read and decrypt block table
        file.seek(SeekFrom::Start(header.block_table_offset as u64))?;
        let block_table_bytes = header.block_table_size as usize * 16;
        let mut block_data = vec![0u8; block_table_bytes];
        file.read_exact(&mut block_data)?;

        // Decrypt block table
        let mut block_u32: Vec<u32> = block_data
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        decrypt_block(&mut block_u32, hash_string("(block table)", HASH_TYPE_FILE_KEY));

        // Parse block entries
        let block_table: Vec<MpqBlockEntry> = block_u32
            .chunks_exact(4)
            .map(|chunk| {
                let bytes: [u8; 16] = [
                    chunk[0].to_le_bytes()[0], chunk[0].to_le_bytes()[1], chunk[0].to_le_bytes()[2], chunk[0].to_le_bytes()[3],
                    chunk[1].to_le_bytes()[0], chunk[1].to_le_bytes()[1], chunk[1].to_le_bytes()[2], chunk[1].to_le_bytes()[3],
                    chunk[2].to_le_bytes()[0], chunk[2].to_le_bytes()[1], chunk[2].to_le_bytes()[2], chunk[2].to_le_bytes()[3],
                    chunk[3].to_le_bytes()[0], chunk[3].to_le_bytes()[1], chunk[3].to_le_bytes()[2], chunk[3].to_le_bytes()[3],
                ];
                MpqBlockEntry::from_bytes(&bytes)
            })
            .collect();

        Ok(Self {
            path,
            file,
            header,
            hash_table,
            block_table,
        })
    }

    /// Get the path of this archive
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get block size
    pub fn block_size(&self) -> u32 {
        self.header.block_size()
    }

    /// Find file by name
    pub fn find_file(&self, filename: &str) -> Option<usize> {
        let (hash_index, hash_a, hash_b) = calculate_file_hash(filename);
        let table_size = self.hash_table.len() as u32;

        if table_size == 0 {
            return None;
        }

        let mut index = hash_index % table_size;
        let start_index = index;

        loop {
            let entry = &self.hash_table[index as usize];

            if entry.is_empty() {
                return None;
            }

            if !entry.is_deleted() && entry.hash_a == hash_a && entry.hash_b == hash_b {
                return Some(entry.block_index as usize);
            }

            index = (index + 1) % table_size;
            if index == start_index {
                return None;
            }
        }
    }

    /// Check if file exists
    pub fn has_file(&self, filename: &str) -> bool {
        self.find_file(filename).is_some()
    }

    /// Get file size
    pub fn file_size(&self, filename: &str) -> Option<u32> {
        self.find_file(filename)
            .and_then(|idx| self.block_table.get(idx))
            .map(|block| block.unpacked_size)
    }

    /// Get block table info for debugging
    /// Returns: (index, is_encrypted, packed_size, unpacked_size)
    pub fn get_block_info(&self) -> Vec<(usize, bool, u32, u32)> {
        self.block_table
            .iter()
            .enumerate()
            .map(|(i, block)| (i, block.is_encrypted(), block.packed_size, block.unpacked_size))
            .collect()
    }

    /// Get block flags for a specific block index
    pub fn get_block_flags(&self, index: usize) -> u32 {
        self.block_table.get(index).map(|b| b.flags).unwrap_or(0)
    }

    /// Get block offset for a specific block index
    pub fn get_block_offset(&self, index: usize) -> u32 {
        self.block_table.get(index).map(|b| b.offset).unwrap_or(0)
    }

    /// Calculate file key for encrypted files
    fn calculate_file_key(&self, filename: &str, block: &MpqBlockEntry) -> u32 {
        // Get base filename (without path)
        let base_name = filename.rsplit(['\\', '/']).next().unwrap_or(filename);
        let mut key = hash_string(base_name, HASH_TYPE_FILE_KEY);

        // If KEY_ADJUSTED flag is set, adjust the key
        if (block.flags & BLOCK_FLAG_KEY_ADJUSTED) != 0 {
            key = (key.wrapping_add(block.offset)) ^ block.unpacked_size;
        }

        key
    }

    /// Read file contents (supports encrypted files)
    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>, MpqError> {
        let block_index = self.find_file(filename)
            .ok_or_else(|| MpqError::FileNotFound(filename.to_string()))?;

        let block = self.block_table[block_index];

        if !block.exists() {
            return Err(MpqError::FileNotFound(filename.to_string()));
        }

        // Seek to file data
        self.file.seek(SeekFrom::Start(block.offset as u64))?;

        // Read compressed data
        let mut data = vec![0u8; block.packed_size as usize];
        self.file.read_exact(&mut data)?;

        // If encrypted, decrypt the data
        let is_encrypted = block.is_encrypted();
        let file_key = if is_encrypted {
            Some(self.calculate_file_key(filename, &block))
        } else {
            None
        };

        // If not compressed and not encrypted, return as-is
        if block.packed_size == block.unpacked_size && !is_encrypted {
            return Ok(data);
        }

        // Check if file is stored as single unit
        let is_single_unit = block.is_single_unit();
        let is_pkzip = block.is_pkzip_compressed();
        let is_multi = block.is_multi_compressed();

        if is_encrypted && is_single_unit {
            // Decrypt single unit
            let mut data_u32: Vec<u32> = data
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            decrypt_block(&mut data_u32, file_key.unwrap());
            data = data_u32.iter().flat_map(|v| v.to_le_bytes()).collect();
        }

        // If not compressed, return decrypted data
        if block.packed_size == block.unpacked_size {
            return Ok(data);
        }

        if is_single_unit {
            // Single unit - decompress all at once
            if is_pkzip {
                return self.decompress_pkzip(&data, block.unpacked_size as usize);
            } else if is_multi {
                return self.decompress_multi(&data, block.unpacked_size as usize);
            }
        }

        // Sector-based decompression
        self.read_file_sectored_encrypted(&data, &block, is_pkzip, is_multi, file_key)
    }

    /// Read file with sector-based compression (supports encryption)
    fn read_file_sectored_encrypted(
        &self,
        compressed: &[u8],
        block: &MpqBlockEntry,
        is_pkzip: bool,
        is_multi: bool,
        file_key: Option<u32>,
    ) -> Result<Vec<u8>, MpqError> {
        let block_size = self.header.block_size() as usize;
        let unpacked_size = block.unpacked_size as usize;

        // Calculate number of sectors
        let num_sectors = (unpacked_size + block_size - 1) / block_size;

        // Sector offset table is at the beginning of the data
        // It has num_sectors + 1 entries (last one is end offset)
        let offset_table_size = (num_sectors + 1) * 4;

        if compressed.len() < offset_table_size {
            return Err(MpqError::DecompressionError(
                "Compressed data too small for sector offset table".to_string()
            ));
        }

        // Read and potentially decrypt sector offsets
        let mut sector_offset_data: Vec<u32> = compressed[..offset_table_size]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();

        if let Some(key) = file_key {
            decrypt_block(&mut sector_offset_data, key.wrapping_sub(1));
        }

        let sector_offsets: Vec<usize> = sector_offset_data.iter().map(|&v| v as usize).collect();

        // Validate first offset should be the offset table size
        if sector_offsets[0] != offset_table_size {
            // It might be that the file is not actually sectored
            // Try to decompress as a whole
            if is_pkzip {
                return self.decompress_pkzip(compressed, unpacked_size);
            } else if is_multi {
                return self.decompress_multi(compressed, unpacked_size);
            }
            return Err(MpqError::DecompressionError(
                format!("Invalid sector offset table: first offset {} != table size {}",
                    sector_offsets[0], offset_table_size)
            ));
        }

        let mut output = Vec::with_capacity(unpacked_size);

        // Decompress each sector
        for i in 0..num_sectors {
            let sector_start = sector_offsets[i];
            let sector_end = sector_offsets[i + 1];

            if sector_end > compressed.len() || sector_start > sector_end {
                return Err(MpqError::DecompressionError(
                    format!("Invalid sector bounds: {} - {} (data size: {})",
                        sector_start, sector_end, compressed.len())
                ));
            }

            let mut sector_data = compressed[sector_start..sector_end].to_vec();
            let sector_unpacked_size = if i == num_sectors - 1 {
                // Last sector may be smaller
                unpacked_size - (i * block_size)
            } else {
                block_size
            };

            // Decrypt sector if encrypted
            if let Some(key) = file_key {
                let sector_key = key.wrapping_add(i as u32);
                let mut sector_u32: Vec<u32> = sector_data
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                decrypt_block(&mut sector_u32, sector_key);
                sector_data = sector_u32.iter().flat_map(|v| v.to_le_bytes()).collect();
                // Handle partial last chunk
                let remainder = sector_data.len() % 4;
                if remainder != 0 {
                    let orig_len = compressed[sector_start..sector_end].len();
                    sector_data.truncate(orig_len);
                }
            }

            // Check if sector is compressed
            let sector_packed_size = sector_end - sector_start;
            if sector_packed_size == sector_unpacked_size {
                // Sector is not compressed
                output.extend_from_slice(&sector_data);
            } else if is_pkzip {
                // PKWare DCL compression
                let decompressed = super::explode::explode(&sector_data, sector_unpacked_size)
                    .ok_or_else(|| MpqError::DecompressionError(
                        format!("PKWare explode failed for sector {}", i)
                    ))?;
                output.extend_from_slice(&decompressed);
            } else if is_multi {
                // Multi-compression
                let decompressed = self.decompress_sector_multi(&sector_data, sector_unpacked_size)?;
                output.extend_from_slice(&decompressed);
            } else {
                // Unknown compression, just copy
                output.extend_from_slice(&sector_data);
            }
        }

        Ok(output)
    }

    /// Read file with sector-based compression (legacy, non-encrypted)
    fn read_file_sectored(
        &self,
        compressed: &[u8],
        block: &MpqBlockEntry,
        is_pkzip: bool,
        is_multi: bool,
    ) -> Result<Vec<u8>, MpqError> {
        self.read_file_sectored_encrypted(compressed, block, is_pkzip, is_multi, None)
    }

    /// Decompress a single sector with multi-compression
    fn decompress_sector_multi(&self, data: &[u8], unpacked_size: usize) -> Result<Vec<u8>, MpqError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let compression_type = data[0];
        let compressed_data = &data[1..];

        // Handle multiple compression types in sequence
        let mut current_data = compressed_data.to_vec();
        let current_size = unpacked_size;

        // Apply decompression in order: Huffman, Zlib, PKWare, BZip2, Wave
        // (the order matters for multi-compression)

        if compression_type & COMPRESSION_HUFFMAN != 0 {
            return Err(MpqError::DecompressionError(
                "Huffman compression not supported".to_string()
            ));
        }

        if compression_type & COMPRESSION_ZLIB != 0 {
            use std::io::Read;
            let mut decoder = flate2::read::ZlibDecoder::new(&current_data[..]);
            let mut output = Vec::with_capacity(current_size);
            decoder.read_to_end(&mut output)
                .map_err(|e| MpqError::DecompressionError(e.to_string()))?;
            current_data = output;
        }

        if compression_type & COMPRESSION_PKZIP != 0 {
            current_data = super::explode::explode(&current_data, current_size)
                .ok_or_else(|| MpqError::DecompressionError(
                    "PKWare DCL explode failed".to_string()
                ))?;
        }

        if compression_type & COMPRESSION_BZIP2 != 0 {
            use std::io::Read;
            let mut decoder = bzip2::read::BzDecoder::new(&current_data[..]);
            let mut output = Vec::with_capacity(current_size);
            decoder.read_to_end(&mut output)
                .map_err(|e| MpqError::DecompressionError(e.to_string()))?;
            current_data = output;
        }

        // Wave compression (mono/stereo) not implemented yet
        if compression_type & (COMPRESSION_WAVE_MONO | COMPRESSION_WAVE_STEREO) != 0 {
            return Err(MpqError::DecompressionError(
                "Wave compression not supported".to_string()
            ));
        }

        Ok(current_data)
    }

    /// Decompress PKZip compressed data (PKWare DCL Implode)
    fn decompress_pkzip(&self, data: &[u8], unpacked_size: usize) -> Result<Vec<u8>, MpqError> {
        // MPQ uses PKWare DCL "implode" algorithm, not standard deflate
        super::explode::explode(data, unpacked_size)
            .ok_or_else(|| MpqError::DecompressionError("PKWare DCL explode failed".to_string()))
    }

    /// Decompress multi-compressed data
    fn decompress_multi(&self, data: &[u8], unpacked_size: usize) -> Result<Vec<u8>, MpqError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // First byte indicates compression type
        let compression_type = data[0];
        let compressed_data = &data[1..];

        match compression_type {
            0x01 => {
                // Huffman (not implemented)
                Err(MpqError::DecompressionError("Huffman compression not supported".to_string()))
            }
            0x02 => {
                // Zlib
                use std::io::Read;
                let mut decoder = flate2::read::ZlibDecoder::new(compressed_data);
                let mut output = Vec::with_capacity(unpacked_size);
                decoder.read_to_end(&mut output)
                    .map_err(|e| MpqError::DecompressionError(e.to_string()))?;
                Ok(output)
            }
            0x08 => {
                // PKWare DCL Implode
                super::explode::explode(compressed_data, unpacked_size)
                    .ok_or_else(|| MpqError::DecompressionError("PKWare DCL explode failed".to_string()))
            }
            0x10 => {
                // BZip2
                use std::io::Read;
                let mut decoder = bzip2::read::BzDecoder::new(compressed_data);
                let mut output = Vec::with_capacity(unpacked_size);
                decoder.read_to_end(&mut output)
                    .map_err(|e| MpqError::DecompressionError(e.to_string()))?;
                Ok(output)
            }
            _ => {
                Err(MpqError::DecompressionError(format!("Unknown compression type: 0x{:02X}", compression_type)))
            }
        }
    }

    /// List all files (requires listfile)
    pub fn list_files(&mut self) -> Result<Vec<String>, MpqError> {
        let listfile = self.read_file("(listfile)")?;
        let content = String::from_utf8_lossy(&listfile);

        Ok(content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
}

/// Asset Manager - manages multiple MPQ archives
pub struct AssetManager {
    /// MPQ archives in priority order (higher priority first)
    archives: Vec<(i32, MpqArchive)>,
    /// Search paths for MPQ files
    search_paths: Vec<PathBuf>,
}

impl AssetManager {
    /// Create a new asset manager
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
            search_paths: Vec::new(),
        }
    }

    /// Add a search path
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Load an MPQ archive with given priority
    pub fn load_mpq<P: AsRef<Path>>(&mut self, path: P, priority: i32) -> Result<(), MpqError> {
        let archive = MpqArchive::open(path)?;
        self.archives.push((priority, archive));
        // Sort by priority (descending)
        self.archives.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(())
    }

    /// Load MPQ from search paths
    pub fn load_mpq_from_search(&mut self, name: &str, priority: i32) -> Result<bool, MpqError> {
        for search_path in &self.search_paths {
            let mpq_path = search_path.join(format!("{}.mpq", name));
            if mpq_path.exists() {
                self.load_mpq(&mpq_path, priority)?;
                println!("Loaded MPQ: {:?}", mpq_path);
                return Ok(true);
            }

            // Try uppercase (for DIABDAT.MPQ)
            let mpq_path_upper = search_path.join(format!("{}.MPQ", name.to_uppercase()));
            if mpq_path_upper.exists() {
                self.load_mpq(&mpq_path_upper, priority)?;
                println!("Loaded MPQ: {:?}", mpq_path_upper);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Find file across all archives
    pub fn find_file(&self, filename: &str) -> Option<usize> {
        for (i, (_, archive)) in self.archives.iter().enumerate() {
            if archive.has_file(filename) {
                return Some(i);
            }
        }
        None
    }

    /// Check if file exists
    pub fn has_file(&self, filename: &str) -> bool {
        self.find_file(filename).is_some()
    }

    /// Read file from any archive
    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>, MpqError> {
        for (_, archive) in &mut self.archives {
            if archive.has_file(filename) {
                return archive.read_file(filename);
            }
        }
        Err(MpqError::FileNotFound(filename.to_string()))
    }

    /// Get file size
    pub fn file_size(&self, filename: &str) -> Option<u32> {
        for (_, archive) in &self.archives {
            if let Some(size) = archive.file_size(filename) {
                return Some(size);
            }
        }
        None
    }

    /// Load core archives (devilutionx.mpq, fonts.mpq)
    pub fn load_core_archives(&mut self) -> Result<(), MpqError> {
        // devilutionx.mpq has priority 9000
        let _ = self.load_mpq_from_search("devilutionx", 9000);

        // fonts.mpq has priority 9200
        let _ = self.load_mpq_from_search("fonts", 9200);

        Ok(())
    }

    /// Load game archives (diabdat.mpq or spawn.mpq)
    pub fn load_game_archives(&mut self) -> Result<bool, MpqError> {
        // Try DIABDAT.MPQ first (uppercase on CD)
        if self.load_mpq_from_search("DIABDAT", 1000)? {
            return Ok(true);
        }

        // Try lowercase diabdat.mpq
        if self.load_mpq_from_search("diabdat", 1000)? {
            return Ok(true);
        }

        // Fall back to spawn.mpq (shareware)
        if self.load_mpq_from_search("spawn", 1000)? {
            return Ok(true);
        }

        Ok(false)
    }

    /// Load Hellfire archives
    pub fn load_hellfire_archives(&mut self) -> Result<bool, MpqError> {
        if self.load_mpq_from_search("hellfire", 8000)? {
            let _ = self.load_mpq_from_search("hfmonk", 8100);
            let _ = self.load_mpq_from_search("hfmusic", 8200);
            let _ = self.load_mpq_from_search("hfvoice", 8500);
            let _ = self.load_mpq_from_search("hfbard", 8110);
            let _ = self.load_mpq_from_search("hfbarb", 8120);
            return Ok(true);
        }
        Ok(false)
    }
    /// List all files (requires listfile)
    pub fn list_files(&mut self) -> Result<Vec<String>, MpqError> {
        let mut all_files = std::collections::HashSet::new();

        for (_, archive) in &mut self.archives {
            if let Ok(files) = archive.list_files() {
                for f in files {
                    all_files.insert(f);
                }
            }
        }

        Ok(all_files.into_iter().collect())
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        // Known hash values from original game
        let hash = hash_string("(hash table)", HASH_TYPE_FILE_KEY);
        assert_eq!(hash, 0xC3AF3770);

        let hash = hash_string("(block table)", HASH_TYPE_FILE_KEY);
        assert_eq!(hash, 0xEC83B3A3);
    }

    #[test]
    fn test_calculate_file_hash() {
        let (offset, a, b) = calculate_file_hash("(listfile)");
        // These are known values
        assert!(offset > 0);
        assert!(a > 0);
        assert!(b > 0);
    }
}
