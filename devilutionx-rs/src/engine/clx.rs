//! CLX/CL2/CEL Sprite Format
//!
//! These are the sprite formats used by Diablo and DevilutionX.
//!
//! ## CLX Format (DevilutionX runtime format)
//!
//! CLX frame header (6 bytes):
//!   - bytes 0-2: header size (u16)
//!   - bytes 2-4: width (u16)
//!   - bytes 4-6: height (u16)
//!
//! ## CL2 Format (Original Diablo format)
//!
//! CL2 frame header:
//!   - bytes 0-2: header size (u16) - typically 0x0A (10 bytes)
//!   - bytes 2-4: unused
//!   - bytes 4-10: 32-pixel block offsets (3 u16s)
//!
//! ## Pixel Data Encoding (same for CLX and CL2)
//!
//! Commands are identified by the first byte:
//!   - 0x00-0x7F: Transparent - skip N pixels (N = command byte)
//!   - 0x80-0xBE: Fill - fill (0xBF - command) pixels with the next byte
//!   - 0xBF-0xFF: Copy - copy (256 - command) pixels from the stream
//!
//! Note: Original CL2 uses opposite encoding:
//!   - 0x80-0xBE: Fill with next byte, width = 0xBF - control
//!   - 0xBF-0xFF: Copy N pixels, N = -(control as i8) = 256 - control

use std::sync::Arc;

/// CLX rendering command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClxCommand {
    /// Copy N pixels from source data
    Copy(u8),
    /// Skip N transparent pixels
    Skip(u16),
    /// Fill N pixels with a single color
    Fill { count: u8, color: u8 },
}

/// A single CLX sprite frame
#[derive(Debug, Clone)]
pub struct ClxSprite {
    /// Sprite width
    pub width: u16,
    /// Sprite height
    pub height: u16,
    /// Raw pixel data (CL2 encoded)
    pub pixel_data: Vec<u8>,
}

impl ClxSprite {
    /// Parse a CLX sprite from raw data
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }

        let header_size = u16::from_le_bytes([data[0], data[1]]) as usize;
        let width = u16::from_le_bytes([data[2], data[3]]);
        let height = u16::from_le_bytes([data[4], data[5]]);

        if header_size > data.len() {
            return None;
        }

        let pixel_data = data[header_size..].to_vec();

        Some(Self {
            width,
            height,
            pixel_data,
        })
    }

    /// Parse a CL2 sprite from raw data (requires known width/height)
    pub fn from_cl2_bytes(data: &[u8], width: u16, height: u16) -> Option<Self> {
        if data.len() < 10 {
            return None;
        }
        let header_size = u16::from_le_bytes([data[0], data[1]]) as usize;
        if header_size > data.len() {
            return None;
        }
        Some(Self {
            width,
            height,
            pixel_data: data[header_size..].to_vec(),
        })
    }

    /// Decode sprite to RGBA pixels using a palette
    pub fn decode_rgba(&self, palette: &[u8; 768]) -> Vec<u8> {
        let mut pixels = vec![0u8; self.width as usize * self.height as usize * 4];
        self.render_to_rgba(&mut pixels, self.width as usize, 0, 0, palette);
        pixels
    }

    /// Render CL2 sprite to a target buffer
    pub fn render_cl2_to_rgba(
        &self,
        target: &mut [u8],
        target_width: usize,
        x: i32,
        y: i32,
        palette: &[u8; 768],
    ) {
        let mut src_idx = 0;
        let mut dst_x = x;
        let mut dst_y = y + self.height as i32 - 1;
        let sprite_width = self.width as i32;

        while src_idx < self.pixel_data.len() && dst_y >= y {
            let control = self.pixel_data[src_idx];
            src_idx += 1;

            if control < 0x80 {
                // Transparent: skip N pixels
                dst_x += control as i32;
            } else if control <= 0xBE {
                // Fill: fill N pixels with next byte
                // N = 0xBF - control
                let width = (0xBF - control) as usize;
                if src_idx >= self.pixel_data.len() { break; }
                let color_idx = self.pixel_data[src_idx] as usize;
                src_idx += 1;

                for _ in 0..width {
                    if dst_x >= 0 && dst_x < target_width as i32 && dst_y >= 0 {
                        let target_idx = ((dst_y as usize * target_width) + dst_x as usize) * 4;
                        if target_idx + 3 < target.len() {
                            target[target_idx] = palette[color_idx * 3];
                            target[target_idx + 1] = palette[color_idx * 3 + 1];
                            target[target_idx + 2] = palette[color_idx * 3 + 2];
                            target[target_idx + 3] = 255;
                        }
                    }
                    dst_x += 1;
                }
            } else {
                // Copy: copy N pixels
                // N = 256 - control
                let width = (256 - (control as u16)) as usize;
                for _ in 0..width {
                    if src_idx >= self.pixel_data.len() { break; }
                    let color_idx = self.pixel_data[src_idx] as usize;
                    src_idx += 1;

                    if dst_x >= 0 && dst_x < target_width as i32 && dst_y >= 0 {
                        let target_idx = ((dst_y as usize * target_width) + dst_x as usize) * 4;
                        if target_idx + 3 < target.len() {
                            target[target_idx] = palette[color_idx * 3];
                            target[target_idx + 1] = palette[color_idx * 3 + 1];
                            target[target_idx + 2] = palette[color_idx * 3 + 2];
                            target[target_idx + 3] = 255;
                        }
                    }
                    dst_x += 1;
                }
            }

            while dst_x >= x + sprite_width {
                dst_x -= sprite_width;
                dst_y -= 1;
            }
        }
    }

    /// Render sprite to a target buffer at given position
    /// Uses CL2 encoding: renders bottom-to-top
    pub fn render_to_rgba(
        &self,
        target: &mut [u8],
        target_width: usize,
        x: i32,
        y: i32,
        palette: &[u8; 768],
    ) {
        let mut src_idx = 0;
        let mut dst_x = x;
        let mut dst_y = y + self.height as i32 - 1; // CL2 renders bottom-to-top
        let sprite_width = self.width as i32;

        while src_idx < self.pixel_data.len() && dst_y >= y {
            let control = self.pixel_data[src_idx];
            src_idx += 1;

            if !is_clx_opaque(control) {
                // Transparent: skip N pixels (N = control byte, 0x00-0x7F)
                dst_x += control as i32;
            } else if is_clx_opaque_fill(control) {
                // Fill: fill N pixels with the next byte (N = 0xBF - control)
                let width = get_clx_opaque_fill_width(control) as usize;
                if src_idx >= self.pixel_data.len() {
                    break;
                }
                let color_idx = self.pixel_data[src_idx] as usize;
                src_idx += 1;

                for _ in 0..width {
                    if dst_x >= 0 && dst_x < target_width as i32 && dst_y >= 0 {
                        let target_idx = ((dst_y as usize * target_width) + dst_x as usize) * 4;
                        if target_idx + 3 < target.len() {
                            target[target_idx] = palette[color_idx * 3];
                            target[target_idx + 1] = palette[color_idx * 3 + 1];
                            target[target_idx + 2] = palette[color_idx * 3 + 2];
                            target[target_idx + 3] = 255;
                        }
                    }
                    dst_x += 1;
                }
            } else {
                // Copy: copy N pixels from stream (N = 256 - control = -control as i8)
                let width = get_clx_opaque_pixels_width(control) as usize;
                for _ in 0..width {
                    if src_idx >= self.pixel_data.len() {
                        break;
                    }
                    let color_idx = self.pixel_data[src_idx] as usize;
                    src_idx += 1;

                    if dst_x >= 0 && dst_x < target_width as i32 && dst_y >= 0 {
                        let target_idx = ((dst_y as usize * target_width) + dst_x as usize) * 4;
                        if target_idx + 3 < target.len() {
                            target[target_idx] = palette[color_idx * 3];
                            target[target_idx + 1] = palette[color_idx * 3 + 1];
                            target[target_idx + 2] = palette[color_idx * 3 + 2];
                            target[target_idx + 3] = 255;
                        }
                    }
                    dst_x += 1;
                }
            }

            // Handle line wrap - when we've gone past the sprite width
            while dst_x >= x + sprite_width {
                dst_x -= sprite_width;
                dst_y -= 1;
            }
        }
    }
}

/// Check if control byte indicates opaque pixels (>= 0x80)
#[inline]
fn is_clx_opaque(control: u8) -> bool {
    control >= 0x80
}

/// Check if control byte indicates fill operation (0x80..=0xBE)
#[inline]
fn is_clx_opaque_fill(control: u8) -> bool {
    control <= 0xBE
}

/// Get fill width from control byte (0xBF - control)
#[inline]
fn get_clx_opaque_fill_width(control: u8) -> u8 {
    0xBF - control
}

/// Get copy width from control byte (256 - control = -(control as i8))
#[inline]
fn get_clx_opaque_pixels_width(control: u8) -> u8 {
    (-(control as i8)) as u8
}

/// A list of CLX sprites (animation frames)
#[derive(Debug, Clone)]
pub struct ClxSpriteList {
    /// All sprites in the list
    pub sprites: Vec<ClxSprite>,
    /// Raw data (owned)
    data: Arc<Vec<u8>>,
}

impl ClxSpriteList {
    /// Create new sprite list from sprites
    pub fn new(sprites: Vec<ClxSprite>) -> Self {
        Self {
            sprites,
            data: Arc::new(Vec::new()),
        }
    }

    /// Load from file
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to load CLX: {:?}", path))?;
        Self::from_bytes(data)
            .ok_or_else(|| anyhow::anyhow!("Invalid CLX format"))
    }

    /// Parse a CLX sprite list from raw data
    pub fn from_bytes(data: Vec<u8>) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let num_sprites = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if data.len() < 4 + (num_sprites + 1) * 4 {
            return None;
        }

        let mut sprites = Vec::with_capacity(num_sprites);

        for i in 0..num_sprites {
            let offset_idx = 4 + i * 4;
            let begin = u32::from_le_bytes([
                data[offset_idx],
                data[offset_idx + 1],
                data[offset_idx + 2],
                data[offset_idx + 3],
            ]) as usize;

            let end = u32::from_le_bytes([
                data[offset_idx + 4],
                data[offset_idx + 5],
                data[offset_idx + 6],
                data[offset_idx + 7],
            ]) as usize;

            if begin >= data.len() || end > data.len() || begin >= end {
                continue;
            }

            if let Some(sprite) = ClxSprite::from_bytes(&data[begin..end]) {
                sprites.push(sprite);
            }
        }

        Some(Self {
            sprites,
            data: Arc::new(data),
        })
    }

    /// Get number of sprites
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// Get sprite by index
    pub fn get(&self, index: usize) -> Option<&ClxSprite> {
        self.sprites.get(index)
    }
}

impl std::ops::Index<usize> for ClxSpriteList {
    type Output = ClxSprite;

    fn index(&self, index: usize) -> &Self::Output {
        &self.sprites[index]
    }
}

/// A sprite sheet (multiple sprite lists, one per direction)
#[derive(Debug, Clone)]
pub struct ClxSpriteSheet {
    /// Sprite lists (typically 8 directions)
    pub lists: Vec<ClxSpriteList>,
    /// Raw data
    data: Arc<Vec<u8>>,
}

impl ClxSpriteSheet {
    /// Create new sprite sheet
    pub fn new(sprites: Vec<ClxSprite>) -> Self {
        // Treat all sprites as single list for now
        let list = ClxSpriteList::new(sprites);
        Self {
            lists: vec![list],
            data: Arc::new(Vec::new()),
        }
    }

    /// Load from file
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to load CLX sheet: {:?}", path))?;
        // Assume 8 directions for now
        Self::from_bytes(data, 8)
            .ok_or_else(|| anyhow::anyhow!("Invalid CLX sheet format"))
    }

    /// Parse a sprite sheet from raw data
    /// The number of lists must be provided (e.g., 8 for directional sprites)
    pub fn from_bytes(data: Vec<u8>, num_lists: usize) -> Option<Self> {
        if data.len() < num_lists * 4 {
            return None;
        }

        let shared_data = Arc::new(data);
        let mut lists = Vec::with_capacity(num_lists);

        for i in 0..num_lists {
            let offset = u32::from_le_bytes([
                shared_data[i * 4],
                shared_data[i * 4 + 1],
                shared_data[i * 4 + 2],
                shared_data[i * 4 + 3],
            ]) as usize;

            if offset >= shared_data.len() {
                continue;
            }

            // Get the data slice for this list
            let end = if i + 1 < num_lists {
                u32::from_le_bytes([
                    shared_data[(i + 1) * 4],
                    shared_data[(i + 1) * 4 + 1],
                    shared_data[(i + 1) * 4 + 2],
                    shared_data[(i + 1) * 4 + 3],
                ]) as usize
            } else {
                shared_data.len()
            };

            let list_data = shared_data[offset..end].to_vec();
            if let Some(list) = ClxSpriteList::from_bytes(list_data) {
                lists.push(list);
            }
        }

        Some(Self {
            lists,
            data: shared_data,
        })
    }

    /// Get number of lists (directions)
    pub fn num_lists(&self) -> usize {
        self.lists.len()
    }

    /// Get sprite list by index
    pub fn get(&self, index: usize) -> Option<&ClxSpriteList> {
        self.lists.get(index)
    }
}

impl std::ops::Index<usize> for ClxSpriteSheet {
    type Output = ClxSpriteList;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lists[index]
    }
}

/// CEL format parser (original Diablo format, converted to CLX at load)
pub struct CelSprite;

impl CelSprite {
    /// Convert CEL data to CLX format
    /// CEL format is similar but has different header structure
    pub fn cel_to_clx(cel_data: &[u8], width: u16) -> Option<ClxSpriteList> {
        if cel_data.len() < 4 {
            return None;
        }

        let num_frames = u32::from_le_bytes([cel_data[0], cel_data[1], cel_data[2], cel_data[3]]) as usize;

        if cel_data.len() < 4 + (num_frames + 1) * 4 {
            return None;
        }

        let mut sprites = Vec::with_capacity(num_frames);

        for i in 0..num_frames {
            let offset_idx = 4 + i * 4;
            let begin = u32::from_le_bytes([
                cel_data[offset_idx],
                cel_data[offset_idx + 1],
                cel_data[offset_idx + 2],
                cel_data[offset_idx + 3],
            ]) as usize;

            let end = u32::from_le_bytes([
                cel_data[offset_idx + 4],
                cel_data[offset_idx + 5],
                cel_data[offset_idx + 6],
                cel_data[offset_idx + 7],
            ]) as usize;

            if begin >= cel_data.len() || end > cel_data.len() || begin >= end {
                continue;
            }

            // CEL frames don't have width/height in header, we need to provide them
            let frame_data = &cel_data[begin..end];

            // Calculate height from pixel count (assuming no compression for estimation)
            // This is a simplification - real CEL parsing is more complex
            let height = (frame_data.len() / width as usize).max(1) as u16;

            sprites.push(ClxSprite {
                width,
                height,
                pixel_data: frame_data.to_vec(),
            });
        }

        Some(ClxSpriteList {
            sprites,
            data: Arc::new(cel_data.to_vec()),
        })
    }
}

/// Palette for Diablo sprites
#[derive(Debug, Clone)]
pub struct DiabloPalette {
    /// RGB values (256 colors * 3 bytes)
    pub colors: [u8; 768],
}

impl DiabloPalette {
    /// Create a default grayscale palette
    pub fn grayscale() -> Self {
        let mut colors = [0u8; 768];
        for i in 0..256 {
            colors[i * 3] = i as u8;
            colors[i * 3 + 1] = i as u8;
            colors[i * 3 + 2] = i as u8;
        }
        Self { colors }
    }

    /// Load palette from PAL file data
    pub fn from_pal_data(data: &[u8]) -> Option<Self> {
        if data.len() < 768 {
            return None;
        }

        let mut colors = [0u8; 768];
        colors.copy_from_slice(&data[..768]);
        Some(Self { colors })
    }

    /// Get RGB color for palette index
    pub fn get_rgb(&self, index: u8) -> (u8, u8, u8) {
        let idx = index as usize * 3;
        (self.colors[idx], self.colors[idx + 1], self.colors[idx + 2])
    }

    /// Get RGBA color for palette index (transparent index 0)
    pub fn get_rgba(&self, index: u8) -> (u8, u8, u8, u8) {
        if index == 0 {
            return (0, 0, 0, 0);
        }
        let (r, g, b) = self.get_rgb(index);
        (r, g, b, 255)
    }
}

impl Default for DiabloPalette {
    fn default() -> Self {
        Self::grayscale()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale_palette() {
        let palette = DiabloPalette::grayscale();
        assert_eq!(palette.get_rgb(0), (0, 0, 0));
        assert_eq!(palette.get_rgb(128), (128, 128, 128));
        assert_eq!(palette.get_rgb(255), (255, 255, 255));
    }

    #[test]
    fn test_clx_sprite_empty() {
        let data = vec![0u8; 5]; // Too short
        assert!(ClxSprite::from_bytes(&data).is_none());
    }
}
