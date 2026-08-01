//! Dungeon Data Formats
//!
//! This module handles loading and parsing of Diablo's dungeon data files:
//! - DUN: Dungeon layout files (predefined room/dungeon templates)
//! - MIN: Mega-tile definitions (how sub-tiles combine into tiles)
//! - TIL: Tile graphics indices
//! - SOL: Tile solidity/properties data
//! - PAL: Palette files
//!
//! Based on DevilutionX Source/levels/gendung.cpp

use bitflags::bitflags;

/// Tile dimensions
pub const TILE_WIDTH: usize = 64;
pub const TILE_HEIGHT: usize = 32;

/// Sub-tile (frame) dimensions
pub const FRAME_WIDTH: usize = 32;
pub const FRAME_HEIGHT: usize = 32;

bitflags! {
    /// Tile properties from SOL data
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TileProperties: u8 {
        const NONE = 0;
        const SOLID = 1 << 0;
        const BLOCK_LIGHT = 1 << 1;
        const BLOCK_MISSILE = 1 << 2;
        const TRANSPARENT = 1 << 3;
        const TRANSPARENT_LEFT = 1 << 4;
        const TRANSPARENT_RIGHT = 1 << 5;
        const TRAP = 1 << 7;
    }
}

/// Tile type determines rendering shape and data encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TileType {
    /// 32x32 square - stored as array of pixels
    #[default]
    Square = 0,
    /// 32x32 square with transparency - RLE encoded
    TransparentSquare = 1,
    /// Left-pointing 32x31 triangle
    LeftTriangle = 2,
    /// Right-pointing 32x31 triangle
    RightTriangle = 3,
    /// Left-pointing 32x32 trapezoid
    LeftTrapezoid = 4,
    /// Right-pointing 32x32 trapezoid
    RightTrapezoid = 5,
}

impl TileType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => TileType::Square,
            1 => TileType::TransparentSquare,
            2 => TileType::LeftTriangle,
            3 => TileType::RightTriangle,
            4 => TileType::LeftTrapezoid,
            5 => TileType::RightTrapezoid,
            _ => TileType::Square,
        }
    }
}

/// Level cel block reference (from MIN data)
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelCelBlock {
    pub data: u16,
}

impl LevelCelBlock {
    pub fn new(data: u16) -> Self {
        Self { data }
    }

    /// Check if this block has a valid frame reference
    pub fn has_value(&self) -> bool {
        self.data != 0
    }

    /// Get tile type from bits 12-14
    pub fn tile_type(&self) -> TileType {
        TileType::from_u8(((self.data & 0x7000) >> 12) as u8)
    }

    /// Get 1-based frame index from bits 0-11
    pub fn frame(&self) -> u16 {
        self.data & 0x0FFF
    }
}

/// A mega-tile (16 sub-tiles arranged in 4x4 grid)
#[derive(Debug, Clone, Default)]
pub struct MegaTile {
    /// Sub-tile references (16 blocks in 4x4 arrangement)
    /// Layout: bottom-to-top, left-to-right
    pub blocks: [LevelCelBlock; 16],
}

impl MegaTile {
    /// Parse mega-tile from MIN file data
    /// blocks_per_tile: 10 (cathedral/catacombs/caves), 12 (hell), 16 (town)
    pub fn from_bytes(data: &[u8], blocks_per_tile: usize) -> Option<Self> {
        if data.len() < blocks_per_tile * 2 {
            return None;
        }

        let mut blocks = [LevelCelBlock::default(); 16];

        // C++ logic: pieces[blocks - 2 + (block & 1) - (block & 0xE)] -> mt[block]
        // This remaps from MIN file order to rendering order
        for block in 0..blocks_per_tile.min(16) {
            // Calculate source index in MIN file
            let src_idx = blocks_per_tile - 2 + (block & 1) - (block & 0xE);
            if src_idx < blocks_per_tile {
                let raw = u16::from_le_bytes([data[src_idx * 2], data[src_idx * 2 + 1]]);
                blocks[block] = LevelCelBlock::new(raw);
            }
        }

        Some(Self { blocks })
    }
}

/// Dungeon template (DUN file)
#[derive(Debug, Clone)]
pub struct DunTemplate {
    /// Template width in tiles
    pub width: u16,
    /// Template height in tiles
    pub height: u16,
    /// Tile data (width * height)
    pub tiles: Vec<u16>,
    /// Monster data (if present)
    pub monsters: Vec<u16>,
    /// Object data (if present)
    pub objects: Vec<u16>,
    /// Transparency data (if present)
    pub transparency: Vec<u16>,
}

impl DunTemplate {
    /// Parse DUN file
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let width = u16::from_le_bytes([data[0], data[1]]);
        let height = u16::from_le_bytes([data[2], data[3]]);
        let tile_count = (width as usize) * (height as usize);

        if data.len() < 4 + tile_count * 2 {
            return None;
        }

        // Parse tile data
        let mut tiles = Vec::with_capacity(tile_count);
        for i in 0..tile_count {
            let offset = 4 + i * 2;
            tiles.push(u16::from_le_bytes([data[offset], data[offset + 1]]));
        }

        // Optional: monsters, objects, transparency
        let mut offset = 4 + tile_count * 2;
        let mut monsters = Vec::new();
        let mut objects = Vec::new();
        let mut transparency = Vec::new();

        // Monsters section
        if data.len() >= offset + tile_count * 2 {
            for i in 0..tile_count {
                let o = offset + i * 2;
                monsters.push(u16::from_le_bytes([data[o], data[o + 1]]));
            }
            offset += tile_count * 2;
        }

        // Objects section
        if data.len() >= offset + tile_count * 2 {
            for i in 0..tile_count {
                let o = offset + i * 2;
                objects.push(u16::from_le_bytes([data[o], data[o + 1]]));
            }
            offset += tile_count * 2;
        }

        // Transparency section
        if data.len() >= offset + tile_count * 2 {
            for i in 0..tile_count {
                let o = offset + i * 2;
                transparency.push(u16::from_le_bytes([data[o], data[o + 1]]));
            }
        }

        Some(Self {
            width,
            height,
            tiles,
            monsters,
            objects,
            transparency,
        })
    }

    /// Get tile at position
    pub fn get_tile(&self, x: u16, y: u16) -> Option<u16> {
        if x < self.width && y < self.height {
            Some(self.tiles[(y as usize) * (self.width as usize) + (x as usize)])
        } else {
            None
        }
    }
}

/// SOL data (solidity/properties for each sub-tile)
#[derive(Debug, Clone)]
pub struct SolData {
    /// Properties for each sub-tile index
    pub properties: Vec<TileProperties>,
}

impl SolData {
    /// Parse SOL file
    pub fn from_bytes(data: &[u8]) -> Self {
        let properties = data.iter().map(|&b| TileProperties::from_bits_truncate(b)).collect();
        Self { properties }
    }

    /// Get properties for a sub-tile index
    pub fn get(&self, index: usize) -> TileProperties {
        self.properties.get(index).copied().unwrap_or_default()
    }

    /// Check if sub-tile is solid
    pub fn is_solid(&self, index: usize) -> bool {
        self.get(index).contains(TileProperties::SOLID)
    }

    /// Check if sub-tile blocks light
    pub fn blocks_light(&self, index: usize) -> bool {
        self.get(index).contains(TileProperties::BLOCK_LIGHT)
    }

    /// Check if sub-tile blocks missiles
    pub fn blocks_missile(&self, index: usize) -> bool {
        self.get(index).contains(TileProperties::BLOCK_MISSILE)
    }

    /// C++ `TileHasAny(Point, TileProperties)` (Source/levels/gendung.h:296):
    /// `HasAnyOf(SOLData[dPiece[coords.x][coords.y]], property)`.
    ///
    /// SOL data is indexed by level-piece id (the `dPiece` grid value).
    #[inline]
    pub fn tile_has_any(&self, level_piece_id: u16, property: TileProperties) -> bool {
        self.get(level_piece_id as usize).intersects(property)
    }

    /// C++ `IsFloor(Point)` (Source/engine/render/scrollrt.cpp:101):
    /// `!TileHasAny(tilePosition, TileProperties::Solid | TileProperties::BlockMissile)`
    #[inline]
    pub fn is_floor(&self, level_piece_id: u16) -> bool {
        !self.tile_has_any(level_piece_id, TileProperties::SOLID | TileProperties::BLOCK_MISSILE)
    }

    /// C++ `IsWall(Point)` (Source/engine/render/scrollrt.cpp:106):
    /// `!IsFloor(tilePosition) || dSpecial[tilePosition.x][tilePosition.y] != 0`
    ///
    /// `dSpecial` is not ported yet (town sector decorations); treated as 0,
    /// so this reduces to `!IsFloor`.
    #[inline]
    pub fn is_wall(&self, level_piece_id: u16) -> bool {
        !self.is_floor(level_piece_id)
    }

    /// C++ `IsTileNotSolid(Point)` (Source/levels/tile_properties.cpp:11):
    /// `!TileHasAny(position, TileProperties::Solid)` (out-of-bounds → false,
    /// bounds are checked by the caller).
    #[inline]
    pub fn is_tile_not_solid(&self, level_piece_id: u16) -> bool {
        !self.tile_has_any(level_piece_id, TileProperties::SOLID)
    }
}

/// MIN data (mega-tile definitions)
#[derive(Debug, Clone)]
pub struct MinData {
    /// All mega-tiles
    pub mega_tiles: Vec<MegaTile>,
    /// Blocks per tile (varies by dungeon type)
    pub blocks_per_tile: usize,
}

impl MinData {
    /// Parse MIN file
    /// blocks_per_tile: 10 (cathedral/catacombs/caves), 12 (hell), 16 (town)
    pub fn from_bytes(data: &[u8], blocks_per_tile: usize) -> Self {
        let bytes_per_tile = blocks_per_tile * 2;
        let tile_count = data.len() / bytes_per_tile;

        let mut mega_tiles = Vec::with_capacity(tile_count);
        for i in 0..tile_count {
            let offset = i * bytes_per_tile;
            if let Some(tile) = MegaTile::from_bytes(&data[offset..], blocks_per_tile) {
                mega_tiles.push(tile);
            }
        }

        Self {
            mega_tiles,
            blocks_per_tile,
        }
    }

    /// Get mega-tile by index
    pub fn get(&self, index: usize) -> Option<&MegaTile> {
        self.mega_tiles.get(index)
    }
}

/// Palette data (256 RGB colors)
#[derive(Debug, Clone)]
pub struct PaletteData {
    /// RGB colors (256 * 3 = 768 bytes)
    pub colors: [u8; 768],
}

impl PaletteData {
    /// Create empty (black) palette
    pub fn new() -> Self {
        Self { colors: [0; 768] }
    }

    /// Parse PAL file
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 768 {
            return None;
        }

        let mut colors = [0u8; 768];
        colors.copy_from_slice(&data[..768]);

        Some(Self { colors })
    }

    /// Get RGB color for index
    pub fn get_rgb(&self, index: u8) -> (u8, u8, u8) {
        let i = (index as usize) * 3;
        (self.colors[i], self.colors[i + 1], self.colors[i + 2])
    }

    /// Get RGBA color for index (index 0 is transparent)
    pub fn get_rgba(&self, index: u8) -> (u8, u8, u8, u8) {
        if index == 0 {
            (0, 0, 0, 0)
        } else {
            let (r, g, b) = self.get_rgb(index);
            (r, g, b, 255)
        }
    }

    /// Convert to flat RGBA array (for texture upload)
    pub fn to_rgba_array(&self) -> [u8; 1024] {
        let mut result = [0u8; 1024];
        for i in 0..256 {
            let (r, g, b, a) = self.get_rgba(i as u8);
            result[i * 4] = r;
            result[i * 4 + 1] = g;
            result[i * 4 + 2] = b;
            result[i * 4 + 3] = a;
        }
        result
    }
}

impl Default for PaletteData {
    fn default() -> Self {
        Self::new()
    }
}

/// TIL file entry - defines a 2x2 tile composed of 4 mega-tile pieces
/// Each entry in TIL file is 8 bytes (4 x uint16)
#[derive(Debug, Clone, Copy, Default)]
pub struct TilEntry {
    /// Top-left piece index (into MIN data)
    pub micro1: u16,
    /// Top-right piece index
    pub micro2: u16,
    /// Bottom-left piece index
    pub micro3: u16,
    /// Bottom-right piece index
    pub micro4: u16,
}

impl TilEntry {
    /// Parse from 8 bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        Some(Self {
            micro1: u16::from_le_bytes([data[0], data[1]]),
            micro2: u16::from_le_bytes([data[2], data[3]]),
            micro3: u16::from_le_bytes([data[4], data[5]]),
            micro4: u16::from_le_bytes([data[6], data[7]]),
        })
    }
}

/// TIL data - maps tile indices to 2x2 mega-tile pieces
#[derive(Debug, Clone)]
pub struct TilData {
    /// All tile entries
    pub tiles: Vec<TilEntry>,
}

impl TilData {
    /// Parse TIL file
    pub fn from_bytes(data: &[u8]) -> Self {
        let entry_count = data.len() / 8;
        let mut tiles = Vec::with_capacity(entry_count);

        for i in 0..entry_count {
            let offset = i * 8;
            if let Some(entry) = TilEntry::from_bytes(&data[offset..]) {
                tiles.push(entry);
            }
        }

        Self { tiles }
    }

    /// Get tile entry by index (1-based from dungeon array, converted to 0-based)
    pub fn get(&self, index: usize) -> Option<&TilEntry> {
        self.tiles.get(index)
    }

    /// Number of tile entries
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

/// Dungeon type (determines which data files to load)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DungeonType {
    #[default]
    Town,
    Cathedral,
    Catacombs,
    Caves,
    Hell,
    Nest,   // Hellfire
    Crypt,  // Hellfire
}

impl DungeonType {
    /// Get data file prefix
    pub fn data_prefix(&self) -> &'static str {
        match self {
            DungeonType::Town => "levels\\towndata\\town",
            DungeonType::Cathedral => "levels\\l1data\\l1",
            DungeonType::Catacombs => "levels\\l2data\\l2",
            DungeonType::Caves => "levels\\l3data\\l3",
            DungeonType::Hell => "levels\\l4data\\l4",
            DungeonType::Nest => "nlevels\\l6data\\l6",
            DungeonType::Crypt => "nlevels\\l5data\\l5",
        }
    }

    /// Get blocks per tile for MIN data
    pub fn blocks_per_tile(&self) -> usize {
        match self {
            DungeonType::Town => 16,
            DungeonType::Hell => 12,
            _ => 10,
        }
    }

    /// Get micro tile length for rendering
    pub fn micro_tile_len(&self) -> usize {
        match self {
            DungeonType::Town => 16,
            DungeonType::Hell => 12,
            _ => 10,
        }
    }
}

/// Complete dungeon level data
#[derive(Debug, Clone)]
pub struct DungeonLevelData {
    /// Dungeon type
    pub dungeon_type: DungeonType,
    /// Palette for this level
    pub palette: PaletteData,
    /// SOL data (tile properties)
    pub sol: SolData,
    /// MIN data (mega-tile definitions)
    pub min: MinData,
    /// TIL data (2x2 tile definitions)
    pub til: TilData,
    /// Level CEL data (tile graphics) - raw bytes
    pub level_cel: Vec<u8>,
}

impl DungeonLevelData {
    /// Create empty level data
    pub fn new(dungeon_type: DungeonType) -> Self {
        Self {
            dungeon_type,
            palette: PaletteData::default(),
            sol: SolData { properties: Vec::new() },
            min: MinData { mega_tiles: Vec::new(), blocks_per_tile: dungeon_type.blocks_per_tile() },
            til: TilData { tiles: Vec::new() },
            level_cel: Vec::new(),
        }
    }

    /// Load level data from MPQ
    pub fn load_from_mpq(mpq: &mut crate::engine::mpq::MpqArchive, dungeon_type: DungeonType) -> Result<Self, crate::engine::mpq::MpqError> {
        let prefix = dungeon_type.data_prefix();
        let blocks_per_tile = dungeon_type.blocks_per_tile();

        // Load palette
        let pal_path = format!("{}.pal", prefix);
        let pal_data = mpq.read_file(&pal_path)?;
        let palette = PaletteData::from_bytes(&pal_data)
            .ok_or_else(|| crate::engine::mpq::MpqError::DecompressionError("Invalid palette".to_string()))?;

        // Load SOL
        let sol_path = format!("{}.sol", prefix);
        let sol_data = mpq.read_file(&sol_path)?;
        let sol = SolData::from_bytes(&sol_data);

        // Load MIN
        let min_path = format!("{}.min", prefix);
        let min_data = mpq.read_file(&min_path)?;
        let min = MinData::from_bytes(&min_data, blocks_per_tile);

        // Load TIL
        let til_path = format!("{}.til", prefix);
        let til_data = mpq.read_file(&til_path)?;
        let til = TilData::from_bytes(&til_data);
        println!("加载 TIL: {} ({} 个 tile 定义)", til_path, til.len());

        // Load level CEL
        let cel_path = format!("{}.cel", prefix);
        let level_cel = mpq.read_file(&cel_path)?;

        Ok(Self {
            dungeon_type,
            palette,
            sol,
            min,
            til,
            level_cel,
        })
    }

    /// Load level data from an `AssetManager` (the multi-MPQ wrapper used by
    /// the game's main context). Equivalent to `load_from_mpq` but operates on
    /// the wrapper, which transparently resolves files across all loaded MPQs.
    ///
    /// This is the entry point the game loop uses (main.rs holds an
    /// `AssetManager`, not a raw `MpqArchive`).
    pub fn load_from_asset_manager(
        assets: &mut crate::engine::mpq::AssetManager,
        dungeon_type: DungeonType,
    ) -> Result<Self, crate::engine::mpq::MpqError> {
        let prefix = dungeon_type.data_prefix();
        let blocks_per_tile = dungeon_type.blocks_per_tile();

        let pal_data = assets.read_file(&format!("{}.pal", prefix))?;
        let palette = PaletteData::from_bytes(&pal_data)
            .ok_or_else(|| crate::engine::mpq::MpqError::DecompressionError("Invalid palette".to_string()))?;

        let sol = SolData::from_bytes(&assets.read_file(&format!("{}.sol", prefix))?);
        let min = MinData::from_bytes(
            &assets.read_file(&format!("{}.min", prefix))?,
            blocks_per_tile,
        );
        let til = TilData::from_bytes(&assets.read_file(&format!("{}.til", prefix))?);
        let level_cel = assets.read_file(&format!("{}.cel", prefix))?;

        Ok(Self {
            dungeon_type,
            palette,
            sol,
            min,
            til,
            level_cel,
        })
    }
}

/// CEL tile decoder
/// Decodes a single 32x32 tile frame to RGBA pixels
pub struct TileDecoder;

impl TileDecoder {
    /// Decode a tile frame to RGBA buffer
    /// Returns 32x32 RGBA pixels (4096 bytes)
    pub fn decode_tile(
        cel_data: &[u8],
        frame_index: u16,
        tile_type: TileType,
        palette: &PaletteData,
    ) -> Option<Vec<u8>> {
        // Frame index is 1-based in LevelCelBlock
        // C++ accesses frameTable[frame] directly where:
        // frameTable[0] = num_frames (or first offset)
        // frameTable[1] = frame 1's offset
        // frameTable[frame] = frame N's data offset
        if frame_index == 0 {
            return None;
        }

        // Get frame offset from header
        let frame_offsets = Self::get_frame_offsets(cel_data)?;
        let frame_idx = frame_index as usize; // Use directly, not -1

        if frame_idx >= frame_offsets.len().saturating_sub(1) {
            return None;
        }

        let frame_start = frame_offsets[frame_idx] as usize;
        let frame_end = frame_offsets[frame_idx + 1] as usize;

        if frame_start >= cel_data.len() || frame_end > cel_data.len() {
            return None;
        }

        let frame_data = &cel_data[frame_start..frame_end];

        // Create 32x32 RGBA buffer (transparent by default)
        let mut pixels = vec![0u8; FRAME_WIDTH * FRAME_HEIGHT * 4];

        match tile_type {
            TileType::Square => Self::decode_square(frame_data, &mut pixels, palette),
            TileType::TransparentSquare => Self::decode_transparent_square(frame_data, &mut pixels, palette),
            TileType::LeftTriangle => Self::decode_left_triangle(frame_data, &mut pixels, palette),
            TileType::RightTriangle => Self::decode_right_triangle(frame_data, &mut pixels, palette),
            TileType::LeftTrapezoid => Self::decode_left_trapezoid(frame_data, &mut pixels, palette),
            TileType::RightTrapezoid => Self::decode_right_trapezoid(frame_data, &mut pixels, palette),
        }

        Some(pixels)
    }

    /// Get frame offsets from CEL header
    /// CEL format:
    /// - Offset 0: frame count (u32)
    /// - Offset 4: frame 0 offset (u32) - but first frame is at index 1 in LevelCelBlock
    /// - Offset 8: frame 1 offset (u32)
    /// - ...
    /// - Offset 4 + frames*4: end offset (for calculating last frame size)
    ///
    /// Note: LevelCelBlock.frame() returns 1-based index, but the frame table
    /// is accessed directly with frameTable[frame] in C++, which means:
    /// - frame=1 -> frameTable[1] -> actual frame 0's data
    fn get_frame_offsets(cel_data: &[u8]) -> Option<Vec<u32>> {
        if cel_data.len() < 4 {
            return None;
        }

        let num_frames = u32::from_le_bytes([cel_data[0], cel_data[1], cel_data[2], cel_data[3]]) as usize;

        // Frame table starts at offset 0, contains num_frames + 1 entries
        // We need at least (num_frames + 1) * 4 bytes for the frame table
        if cel_data.len() < (num_frames + 1) * 4 {
            return None;
        }

        let mut offsets = Vec::with_capacity(num_frames + 1);
        // Read frame table entries directly (no +4 offset)
        for i in 0..=num_frames {
            let offset = i * 4;
            let value = u32::from_le_bytes([
                cel_data[offset],
                cel_data[offset + 1],
                cel_data[offset + 2],
                cel_data[offset + 3],
            ]);
            offsets.push(value);
        }

        Some(offsets)
    }

    /// Set pixel in buffer (bottom-to-top, left-to-right)
    #[inline]
    fn set_pixel(pixels: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
        if x < FRAME_WIDTH && y < FRAME_HEIGHT {
            // Flip Y: Diablo stores bottom-to-top, we want top-to-bottom
            let flipped_y = FRAME_HEIGHT - 1 - y;
            let idx = (flipped_y * FRAME_WIDTH + x) * 4;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = a;
        }
    }

    /// Decode Square tile (32x32 raw pixels, stored bottom-to-top)
    fn decode_square(data: &[u8], pixels: &mut [u8], palette: &PaletteData) {
        for row in 0..FRAME_HEIGHT {
            for col in 0..FRAME_WIDTH {
                let src_idx = row * FRAME_WIDTH + col;
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, col, row, r, g, b, a);
                }
            }
        }
    }

    /// Decode TransparentSquare tile (RLE encoded)
    /// Format: signed byte - positive = pixel count, negative = transparent count
    fn decode_transparent_square(data: &[u8], pixels: &mut [u8], palette: &PaletteData) {
        let mut src_idx = 0;
        let mut row = 0;
        let mut col = 0;

        while src_idx < data.len() && row < FRAME_HEIGHT {
            let cmd = data[src_idx] as i8;
            src_idx += 1;

            if cmd > 0 {
                // Opaque pixels
                let count = cmd as usize;
                for _ in 0..count {
                    if src_idx < data.len() && col < FRAME_WIDTH {
                        let color_idx = data[src_idx];
                        let (r, g, b, a) = palette.get_rgba(color_idx);
                        Self::set_pixel(pixels, col, row, r, g, b, a);
                        src_idx += 1;
                        col += 1;
                    }
                }
            } else {
                // Transparent pixels (skip)
                let count = (-cmd) as usize;
                col += count;
            }

            // Move to next row when reaching end of line
            if col >= FRAME_WIDTH {
                col = 0;
                row += 1;
            }
        }
    }

    /// Decode LeftTriangle tile
    /// Lower half: expanding from right (width increases: 2,4,6,...,32)
    /// Upper half: contracting from left (width decreases: 30,28,...,2)
    fn decode_left_triangle(data: &[u8], pixels: &mut [u8], palette: &PaletteData) {
        let mut src_idx = 0;

        // Lower half: 16 rows, starting from right edge
        // Row 0: 2 pixels at x=30-31
        // Row 1: 4 pixels at x=28-31
        // ...
        // Row 15: 32 pixels at x=0-31
        for row in 0..16 {
            let width = (row + 1) * 2;
            let start_x = FRAME_WIDTH - width;
            for col in 0..width {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, start_x + col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }

        // Upper half: 15 rows, starting from left edge
        // Row 16: 30 pixels at x=2-31
        // Row 17: 28 pixels at x=4-31
        // ...
        // Row 30: 2 pixels at x=30-31
        for i in 0..15 {
            let row = 16 + i;
            let width = 30 - i * 2;
            let start_x = (i + 1) * 2;
            for col in 0..width {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, start_x + col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }
    }

    /// Decode RightTriangle tile
    /// Lower half: expanding from left (width increases: 2,4,6,...,32)
    /// Upper half: contracting from right (width decreases: 30,28,...,2)
    fn decode_right_triangle(data: &[u8], pixels: &mut [u8], palette: &PaletteData) {
        let mut src_idx = 0;

        // Lower half: 16 rows, starting from left edge
        // Row 0: 2 pixels at x=0-1
        // Row 1: 4 pixels at x=0-3
        // ...
        // Row 15: 32 pixels at x=0-31
        for row in 0..16 {
            let width = (row + 1) * 2;
            for col in 0..width {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }

        // Upper half: 15 rows, ending at right edge
        // Row 16: 30 pixels at x=0-29
        // Row 17: 28 pixels at x=0-27
        // ...
        // Row 30: 2 pixels at x=0-1
        for i in 0..15 {
            let row = 16 + i;
            let width = 30 - i * 2;
            for col in 0..width {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }
    }

    /// Decode LeftTrapezoid tile
    /// Lower half: like left triangle (expanding from right)
    /// Upper half: full 32-pixel rows
    fn decode_left_trapezoid(data: &[u8], pixels: &mut [u8], palette: &PaletteData) {
        let mut src_idx = 0;

        // Lower half: 16 rows expanding from right
        for row in 0..16 {
            let width = (row + 1) * 2;
            let start_x = FRAME_WIDTH - width;
            for col in 0..width {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, start_x + col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }

        // Upper half: 16 rows full width
        for row in 16..32 {
            for col in 0..FRAME_WIDTH {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }
    }

    /// Decode RightTrapezoid tile
    /// Lower half: like right triangle (expanding from left)
    /// Upper half: full 32-pixel rows
    fn decode_right_trapezoid(data: &[u8], pixels: &mut [u8], palette: &PaletteData) {
        let mut src_idx = 0;

        // Lower half: 16 rows expanding from left
        for row in 0..16 {
            let width = (row + 1) * 2;
            for col in 0..width {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }

        // Upper half: 16 rows full width
        for row in 16..32 {
            for col in 0..FRAME_WIDTH {
                if src_idx < data.len() {
                    let color_idx = data[src_idx];
                    let (r, g, b, a) = palette.get_rgba(color_idx);
                    Self::set_pixel(pixels, col, row, r, g, b, a);
                    src_idx += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_type() {
        assert_eq!(TileType::from_u8(0), TileType::Square);
        assert_eq!(TileType::from_u8(1), TileType::TransparentSquare);
        assert_eq!(TileType::from_u8(2), TileType::LeftTriangle);
    }

    #[test]
    fn test_level_cel_block() {
        let block = LevelCelBlock::new(0x1234);
        assert!(block.has_value());
        assert_eq!(block.frame(), 0x234);
        assert_eq!(block.tile_type() as u8, 1);
    }

    #[test]
    fn test_dun_template() {
        // Minimal valid DUN: 2x2 with 4 tiles
        let data = [
            2, 0,   // width = 2
            2, 0,   // height = 2
            1, 0,   // tile 0
            2, 0,   // tile 1
            3, 0,   // tile 2
            4, 0,   // tile 3
        ];
        let dun = DunTemplate::from_bytes(&data).unwrap();
        assert_eq!(dun.width, 2);
        assert_eq!(dun.height, 2);
        assert_eq!(dun.get_tile(0, 0), Some(1));
        assert_eq!(dun.get_tile(1, 1), Some(4));
    }

    #[test]
    fn test_sol_data() {
        let data = [0x01, 0x02, 0x03]; // SOLID, BLOCK_LIGHT, SOLID | BLOCK_LIGHT
        let sol = SolData::from_bytes(&data);
        assert!(sol.is_solid(0));
        assert!(!sol.is_solid(1));
        assert!(sol.blocks_light(1));
    }

    /// C++ semantics (scrollrt.cpp:101/106, tile_properties.cpp:11):
    /// piece 0 = no props (floor), 1 = SOLID (wall), 2 = BLOCK_LIGHT (floor),
    /// 3 = SOLID|BLOCK_LIGHT (wall), 4 = BLOCK_MISSILE (not floor), out-of-range = NONE (floor).
    #[test]
    fn test_tile_property_queries() {
        let data = [0x00, 0x01, 0x02, 0x03, 0x04];
        let sol = SolData::from_bytes(&data);
        // IsFloor = !TileHasAny(Solid | BlockMissile)
        assert!(sol.is_floor(0));
        assert!(!sol.is_floor(1));
        assert!(sol.is_floor(2));
        assert!(!sol.is_floor(3));
        assert!(!sol.is_floor(4)); // block-missile tile is not floor
        // IsWall = !IsFloor (dSpecial unported, treated 0)
        assert!(!sol.is_wall(0));
        assert!(sol.is_wall(1));
        assert!(sol.is_wall(4));
        // IsTileNotSolid = !TileHasAny(Solid)
        assert!(sol.is_tile_not_solid(0));
        assert!(!sol.is_tile_not_solid(1));
        assert!(sol.is_tile_not_solid(2));
        // Missing SOL entry → NONE → floor/not-solid (mirrors get() default)
        assert!(sol.is_floor(99));
        assert!(sol.is_tile_not_solid(99));
    }
}
