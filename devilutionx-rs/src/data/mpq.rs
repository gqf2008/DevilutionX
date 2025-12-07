/// MPQ archive format reader
/// MPQ (Mo'PaQ) is Blizzard's archive format used in Diablo, StarCraft, etc.

use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// MPQ Archive header magic
const MPQ_MAGIC: u32 = 0x1A51504D; // "MPQ\x1A"

/// MPQ Hash table entry
#[derive(Debug, Clone)]
struct HashEntry {
    hash_a: u32,
    hash_b: u32,
    _locale: u16,
    _platform: u16,
    block_index: u32,
}

/// MPQ Block table entry
#[derive(Debug, Clone)]
struct BlockEntry {
    offset: u32,
    packed_size: u32,
    unpacked_size: u32,
    flags: u32,
}

/// MPQ Archive reader
pub struct MpqArchive {
    file: File,
    header_offset: u64,
    hash_table: Vec<HashEntry>,
    block_table: Vec<BlockEntry>,
    _sector_size: u32,
}

impl MpqArchive {
    /// Open an MPQ archive from a file path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())?;

        // Search for MPQ header (can be offset in some files)
        let header_offset = Self::find_header(&mut file)?;
        file.seek(SeekFrom::Start(header_offset))?;

        // Read header
        let magic = file.read_u32::<LittleEndian>()?;
        if magic != MPQ_MAGIC {
            return Err(anyhow!("Invalid MPQ magic: {:08X}", magic));
        }

        let header_size = file.read_u32::<LittleEndian>()?;
        let _archive_size = file.read_u32::<LittleEndian>()?;
        let format_version = file.read_u16::<LittleEndian>()?;
        let sector_size_shift = file.read_u16::<LittleEndian>()?;
        let hash_table_offset = file.read_u32::<LittleEndian>()?;
        let block_table_offset = file.read_u32::<LittleEndian>()?;
        let hash_table_entries = file.read_u32::<LittleEndian>()?;
        let block_table_entries = file.read_u32::<LittleEndian>()?;

        let sector_size = 512u32 << sector_size_shift;

        log::debug!("MPQ Header: version={}, header_size={}", format_version, header_size);
        log::debug!("Hash table: {} entries at offset {}", hash_table_entries, hash_table_offset);
        log::debug!("Block table: {} entries at offset {}", block_table_entries, block_table_offset);

        // Read hash table
        let hash_table = Self::read_hash_table(
            &mut file,
            header_offset + hash_table_offset as u64,
            hash_table_entries as usize,
        )?;

        // Read block table
        let block_table = Self::read_block_table(
            &mut file,
            header_offset + block_table_offset as u64,
            block_table_entries as usize,
        )?;

        Ok(Self {
            file,
            header_offset,
            hash_table,
            block_table,
            _sector_size: sector_size,
        })
    }

    /// Find the MPQ header offset (search for magic number)
    fn find_header(file: &mut File) -> Result<u64> {
        let mut buf = [0u8; 4];
        let file_size = file.seek(SeekFrom::End(0))?;

        // Search at 512-byte boundaries (common for MPQ)
        for offset in (0..file_size.min(0x200000)).step_by(512) {
            file.seek(SeekFrom::Start(offset))?;
            if file.read_exact(&mut buf).is_ok() {
                let magic = u32::from_le_bytes(buf);
                if magic == MPQ_MAGIC {
                    return Ok(offset);
                }
            }
        }

        Err(anyhow!("MPQ header not found"))
    }

    /// Read and decrypt the hash table
    fn read_hash_table(file: &mut File, offset: u64, count: usize) -> Result<Vec<HashEntry>> {
        file.seek(SeekFrom::Start(offset))?;

        // Read raw data
        let mut data = vec![0u8; count * 16];
        file.read_exact(&mut data)?;

        // Decrypt hash table
        Self::decrypt_table(&mut data, Self::hash_string("(hash table)", 0x300));

        // Parse entries
        let mut entries = Vec::with_capacity(count);
        let mut cursor = std::io::Cursor::new(&data);

        for _ in 0..count {
            entries.push(HashEntry {
                hash_a: cursor.read_u32::<LittleEndian>()?,
                hash_b: cursor.read_u32::<LittleEndian>()?,
                _locale: cursor.read_u16::<LittleEndian>()?,
                _platform: cursor.read_u16::<LittleEndian>()?,
                block_index: cursor.read_u32::<LittleEndian>()?,
            });
        }

        Ok(entries)
    }

    /// Read and decrypt the block table
    fn read_block_table(file: &mut File, offset: u64, count: usize) -> Result<Vec<BlockEntry>> {
        file.seek(SeekFrom::Start(offset))?;

        // Read raw data
        let mut data = vec![0u8; count * 16];
        file.read_exact(&mut data)?;

        // Decrypt block table
        Self::decrypt_table(&mut data, Self::hash_string("(block table)", 0x300));

        // Parse entries
        let mut entries = Vec::with_capacity(count);
        let mut cursor = std::io::Cursor::new(&data);

        for _ in 0..count {
            entries.push(BlockEntry {
                offset: cursor.read_u32::<LittleEndian>()?,
                packed_size: cursor.read_u32::<LittleEndian>()?,
                unpacked_size: cursor.read_u32::<LittleEndian>()?,
                flags: cursor.read_u32::<LittleEndian>()?,
            });
        }

        Ok(entries)
    }

    /// MPQ encryption table (generated once)
    fn get_crypt_table() -> &'static [u32; 0x500] {
        static CRYPT_TABLE: std::sync::OnceLock<[u32; 0x500]> = std::sync::OnceLock::new();
        CRYPT_TABLE.get_or_init(|| {
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
        })
    }

    /// Hash a string using MPQ's hash algorithm
    fn hash_string(s: &str, hash_type: u32) -> u32 {
        let crypt_table = Self::get_crypt_table();
        let mut seed1: u32 = 0x7FED7FED;
        let mut seed2: u32 = 0xEEEEEEEE;

        for c in s.bytes() {
            let ch = (c as char).to_ascii_uppercase() as u32;
            let ch = if ch == b'/' as u32 { b'\\' as u32 } else { ch };
            seed1 = crypt_table[(hash_type + ch) as usize] ^ seed1.wrapping_add(seed2);
            seed2 = ch.wrapping_add(seed1).wrapping_add(seed2).wrapping_add(seed2 << 5).wrapping_add(3);
        }

        seed1
    }

    /// Decrypt a data block
    fn decrypt_table(data: &mut [u8], key: u32) {
        let crypt_table = Self::get_crypt_table();
        let mut seed1 = key;
        let mut seed2: u32 = 0xEEEEEEEE;

        for chunk in data.chunks_exact_mut(4) {
            seed2 = seed2.wrapping_add(crypt_table[(0x400 + (seed1 & 0xFF)) as usize]);
            let mut value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            value ^= seed1.wrapping_add(seed2);
            seed1 = ((!seed1 << 21).wrapping_add(0x11111111)) | (seed1 >> 11);
            seed2 = value.wrapping_add(seed2).wrapping_add(seed2 << 5).wrapping_add(3);
            chunk.copy_from_slice(&value.to_le_bytes());
        }
    }

    /// Find a file in the archive by name
    fn find_file(&self, filename: &str) -> Option<&BlockEntry> {
        let hash_a = Self::hash_string(filename, 0x100);
        let hash_b = Self::hash_string(filename, 0x200);
        let hash_index = Self::hash_string(filename, 0x000) as usize % self.hash_table.len();

        let mut index = hash_index;
        loop {
            let entry = &self.hash_table[index];

            if entry.block_index == 0xFFFFFFFF {
                return None; // Empty entry, file not found
            }

            if entry.hash_a == hash_a && entry.hash_b == hash_b {
                if entry.block_index < self.block_table.len() as u32 {
                    return Some(&self.block_table[entry.block_index as usize]);
                }
            }

            index = (index + 1) % self.hash_table.len();
            if index == hash_index {
                return None; // Wrapped around, file not found
            }
        }
    }

    /// Check if a file exists in the archive
    pub fn has_file(&self, filename: &str) -> bool {
        self.find_file(filename).is_some()
    }

    /// Read a file from the archive
    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>> {
        let block = self.find_file(filename)
            .ok_or_else(|| anyhow!("File not found: {}", filename))?
            .clone();

        let file_offset = self.header_offset + block.offset as u64;
        self.file.seek(SeekFrom::Start(file_offset))?;

        let mut data = vec![0u8; block.packed_size as usize];
        self.file.read_exact(&mut data)?;

        // Check if file is compressed
        const MPQ_FILE_COMPRESS: u32 = 0x00000200;
        const MPQ_FILE_IMPLODE: u32 = 0x00000100;

        if block.flags & (MPQ_FILE_COMPRESS | MPQ_FILE_IMPLODE) != 0 && block.packed_size != block.unpacked_size {
            // File is compressed - for now just return raw data
            // TODO: Implement proper decompression
            log::warn!("Compressed files not fully supported yet");
            Ok(data)
        } else {
            Ok(data)
        }
    }

    /// List all files (if listfile exists)
    pub fn list_files(&mut self) -> Result<Vec<String>> {
        // Try to read internal listfile
        if let Ok(listfile_data) = self.read_file("(listfile)") {
            let content = String::from_utf8_lossy(&listfile_data);
            Ok(content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        } else {
            Ok(Vec::new())
        }
    }
}
