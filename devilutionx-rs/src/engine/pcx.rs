//! PCX Image Parser
//!
//! PCX is the image format used by Diablo for UI elements and textures.
//! This is a simplified parser for 8-bit paletted PCX images.

use sdl2::pixels::Color;

/// PCX file header (128 bytes)
#[derive(Debug, Clone)]
pub struct PcxHeader {
    /// Manufacturer (always 0x0A for PCX)
    pub manufacturer: u8,
    /// Version
    pub version: u8,
    /// Encoding (1 = RLE)
    pub encoding: u8,
    /// Bits per pixel per plane
    pub bits_per_pixel: u8,
    /// Image dimensions
    pub x_min: u16,
    pub y_min: u16,
    pub x_max: u16,
    pub y_max: u16,
    /// DPI
    pub h_dpi: u16,
    pub v_dpi: u16,
    /// Number of color planes
    pub n_planes: u8,
    /// Bytes per line (per plane)
    pub bytes_per_line: u16,
    /// Palette type (1 = color/BW, 2 = grayscale)
    pub palette_type: u16,
}

impl PcxHeader {
    /// Parse PCX header from data
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 128 {
            return None;
        }

        // Check manufacturer
        if data[0] != 0x0A {
            return None;
        }

        Some(Self {
            manufacturer: data[0],
            version: data[1],
            encoding: data[2],
            bits_per_pixel: data[3],
            x_min: u16::from_le_bytes([data[4], data[5]]),
            y_min: u16::from_le_bytes([data[6], data[7]]),
            x_max: u16::from_le_bytes([data[8], data[9]]),
            y_max: u16::from_le_bytes([data[10], data[11]]),
            h_dpi: u16::from_le_bytes([data[12], data[13]]),
            v_dpi: u16::from_le_bytes([data[14], data[15]]),
            n_planes: data[65],
            bytes_per_line: u16::from_le_bytes([data[66], data[67]]),
            palette_type: u16::from_le_bytes([data[68], data[69]]),
        })
    }

    /// Get image width
    pub fn width(&self) -> u32 {
        (self.x_max - self.x_min + 1) as u32
    }

    /// Get image height
    pub fn height(&self) -> u32 {
        (self.y_max - self.y_min + 1) as u32
    }
}

/// Decoded PCX image
#[derive(Debug, Clone)]
pub struct PcxImage {
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Pixel data (indices into palette)
    pub pixels: Vec<u8>,
    /// Color palette (256 colors)
    pub palette: Vec<Color>,
}

impl PcxImage {
    /// Decode a PCX image from raw data
    pub fn decode(data: &[u8]) -> Option<Self> {
        let header = PcxHeader::parse(data)?;

        // Only support 8-bit PCX with 1 plane
        if header.bits_per_pixel != 8 || header.n_planes != 1 {
            // Try to handle other formats
            if header.bits_per_pixel * header.n_planes != 8 {
                return None;
            }
        }

        let width = header.width();
        let height = header.height();
        let bytes_per_line = header.bytes_per_line as usize;

        // Decode RLE pixel data
        let mut pixels = Vec::with_capacity((width * height) as usize);
        let mut pos = 128; // Skip header

        for _y in 0..height {
            let mut line_bytes = 0;
            while line_bytes < bytes_per_line {
                if pos >= data.len() {
                    return None;
                }

                let byte = data[pos];
                pos += 1;

                if byte >= 0xC0 {
                    // RLE run
                    let count = (byte & 0x3F) as usize;
                    if pos >= data.len() {
                        return None;
                    }
                    let value = data[pos];
                    pos += 1;

                    for _ in 0..count {
                        if line_bytes < width as usize {
                            pixels.push(value);
                        }
                        line_bytes += 1;
                    }
                } else {
                    // Single byte
                    if line_bytes < width as usize {
                        pixels.push(byte);
                    }
                    line_bytes += 1;
                }
            }
        }

        // Read palette (last 769 bytes: 0x0C marker + 256 * 3 bytes)
        let palette = if data.len() >= 769 {
            let palette_start = data.len() - 769;
            if data[palette_start] == 0x0C {
                let mut palette = Vec::with_capacity(256);
                for i in 0..256 {
                    let offset = palette_start + 1 + i * 3;
                    palette.push(Color::RGB(
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                    ));
                }
                palette
            } else {
                // Use default grayscale palette
                (0..256).map(|i| Color::RGB(i as u8, i as u8, i as u8)).collect()
            }
        } else {
            // Use default grayscale palette
            (0..256).map(|i| Color::RGB(i as u8, i as u8, i as u8)).collect()
        };

        Some(Self {
            width,
            height,
            pixels,
            palette,
        })
    }

    /// Get RGBA pixel data
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for &pixel in &self.pixels {
            let color = &self.palette[pixel as usize];
            rgba.push(color.r);
            rgba.push(color.g);
            rgba.push(color.b);
            rgba.push(255); // Alpha
        }
        rgba
    }

    /// Get pixel color at position
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.pixels.len() {
                return Some(self.palette[self.pixels[index] as usize]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcx_header_parse() {
        // Minimal test - just check that invalid data returns None
        assert!(PcxHeader::parse(&[]).is_none());
        assert!(PcxHeader::parse(&[0x00; 128]).is_none()); // Wrong manufacturer
    }
}
