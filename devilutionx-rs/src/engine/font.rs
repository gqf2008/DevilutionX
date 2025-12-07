//! Font rendering system using fontdue
//!
//! Provides text rendering capabilities for the game UI.
//! Supports TTF/OTF fonts with CJK character support.

use anyhow::{Context, Result};
use fontdue::{Font, FontSettings};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::HashMap;
use std::path::Path;

/// Cached glyph data
struct GlyphCache {
    bitmap: Vec<u8>,
    width: usize,
    height: usize,
    advance_x: f32,
    offset_y: i32,
}

/// Font renderer with glyph caching
pub struct FontRenderer {
    font: Font,
    size: f32,
    cache: HashMap<char, GlyphCache>,
}

impl FontRenderer {
    /// Create a new font renderer from a TTF/OTF file
    pub fn from_file<P: AsRef<Path>>(path: P, size: f32) -> Result<Self> {
        let font_data = std::fs::read(path.as_ref())
            .with_context(|| format!("Failed to read font file: {:?}", path.as_ref()))?;
        Self::from_bytes(&font_data, size)
    }

    /// Create a font renderer from font bytes
    pub fn from_bytes(data: &[u8], size: f32) -> Result<Self> {
        let font = Font::from_bytes(data, FontSettings::default())
            .map_err(|e| anyhow::anyhow!("Failed to load font: {}", e))?;

        Ok(Self {
            font,
            size,
            cache: HashMap::new(),
        })
    }

    /// Try to load a system font
    pub fn from_system_font(size: f32) -> Result<Self> {
        // Try common system font paths
        #[cfg(target_os = "windows")]
        let font_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",      // 微软雅黑 (Chinese)
            "C:\\Windows\\Fonts\\simhei.ttf",    // 黑体
            "C:\\Windows\\Fonts\\simsun.ttc",    // 宋体
            "C:\\Windows\\Fonts\\arial.ttf",     // Arial
            "C:\\Windows\\Fonts\\segoeui.ttf",   // Segoe UI
        ];

        #[cfg(target_os = "macos")]
        let font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
        ];

        #[cfg(target_os = "linux")]
        let font_paths = [
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        ];

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let font_paths: [&str; 0] = [];

        for path in font_paths {
            if let Ok(font) = Self::from_file(path, size) {
                return Ok(font);
            }
        }

        Err(anyhow::anyhow!("No system font found"))
    }

    /// Rasterize a character and cache it
    fn rasterize(&mut self, c: char) -> &GlyphCache {
        if !self.cache.contains_key(&c) {
            let (metrics, bitmap) = self.font.rasterize(c, self.size);
            let glyph = GlyphCache {
                bitmap,
                width: metrics.width,
                height: metrics.height,
                advance_x: metrics.advance_width,
                offset_y: metrics.ymin,
            };
            self.cache.insert(c, glyph);
        }
        self.cache.get(&c).unwrap()
    }

    /// Render text to canvas at given position
    pub fn render_text(
        &mut self,
        canvas: &mut Canvas<Window>,
        text: &str,
        x: i32,
        y: i32,
        color: Color,
    ) {
        let mut cursor_x = x;

        for c in text.chars() {
            if c == ' ' {
                cursor_x += (self.size * 0.3) as i32;
                continue;
            }

            let glyph = self.rasterize(c);
            let glyph_y = y - glyph.height as i32 - glyph.offset_y;

            // Render glyph pixel by pixel
            for py in 0..glyph.height {
                for px in 0..glyph.width {
                    let alpha = glyph.bitmap[py * glyph.width + px];
                    if alpha > 0 {
                        // Blend with alpha
                        let blended = Color::RGBA(
                            color.r,
                            color.g,
                            color.b,
                            ((alpha as u16 * color.a as u16) / 255) as u8,
                        );
                        canvas.set_draw_color(blended);
                        let _ = canvas.fill_rect(Rect::new(
                            cursor_x + px as i32,
                            glyph_y + py as i32,
                            1,
                            1,
                        ));
                    }
                }
            }

            cursor_x += glyph.advance_x as i32;
        }
    }

    /// Render text centered horizontally
    pub fn render_text_centered(
        &mut self,
        canvas: &mut Canvas<Window>,
        text: &str,
        center_x: i32,
        y: i32,
        color: Color,
    ) {
        let width = self.text_width(text);
        self.render_text(canvas, text, center_x - width / 2, y, color);
    }

    /// Calculate text width in pixels
    pub fn text_width(&mut self, text: &str) -> i32 {
        let mut width = 0;
        for c in text.chars() {
            if c == ' ' {
                width += (self.size * 0.3) as i32;
            } else {
                let glyph = self.rasterize(c);
                width += glyph.advance_x as i32;
            }
        }
        width
    }

    /// Get font line height
    pub fn line_height(&self) -> i32 {
        (self.size * 1.2) as i32
    }

    /// Get font size
    pub fn size(&self) -> f32 {
        self.size
    }

    /// Change font size (clears cache)
    pub fn set_size(&mut self, size: f32) {
        if (self.size - size).abs() > 0.1 {
            self.size = size;
            self.cache.clear();
        }
    }
}

/// Simple pixel font for fallback (no external dependencies)
pub struct PixelFont {
    scale: i32,
}

impl PixelFont {
    pub fn new(scale: i32) -> Self {
        Self { scale }
    }

    /// Get 5x7 bitmap for ASCII character
    fn get_char_bitmap(c: char) -> Option<[u8; 5]> {
        match c.to_ascii_uppercase() {
            ' ' => Some([0x00, 0x00, 0x00, 0x00, 0x00]),
            '!' => Some([0x00, 0x00, 0x5F, 0x00, 0x00]),
            '"' => Some([0x00, 0x07, 0x00, 0x07, 0x00]),
            '#' => Some([0x14, 0x7F, 0x14, 0x7F, 0x14]),
            '$' => Some([0x24, 0x2A, 0x7F, 0x2A, 0x12]),
            '%' => Some([0x23, 0x13, 0x08, 0x64, 0x62]),
            '&' => Some([0x36, 0x49, 0x55, 0x22, 0x50]),
            '\'' => Some([0x00, 0x05, 0x03, 0x00, 0x00]),
            '(' => Some([0x00, 0x1C, 0x22, 0x41, 0x00]),
            ')' => Some([0x00, 0x41, 0x22, 0x1C, 0x00]),
            '*' => Some([0x14, 0x08, 0x3E, 0x08, 0x14]),
            '+' => Some([0x08, 0x08, 0x3E, 0x08, 0x08]),
            ',' => Some([0x00, 0x50, 0x30, 0x00, 0x00]),
            '-' => Some([0x08, 0x08, 0x08, 0x08, 0x08]),
            '.' => Some([0x00, 0x60, 0x60, 0x00, 0x00]),
            '/' => Some([0x20, 0x10, 0x08, 0x04, 0x02]),
            '0' => Some([0x3E, 0x51, 0x49, 0x45, 0x3E]),
            '1' => Some([0x00, 0x42, 0x7F, 0x40, 0x00]),
            '2' => Some([0x42, 0x61, 0x51, 0x49, 0x46]),
            '3' => Some([0x21, 0x41, 0x45, 0x4B, 0x31]),
            '4' => Some([0x18, 0x14, 0x12, 0x7F, 0x10]),
            '5' => Some([0x27, 0x45, 0x45, 0x45, 0x39]),
            '6' => Some([0x3C, 0x4A, 0x49, 0x49, 0x30]),
            '7' => Some([0x01, 0x71, 0x09, 0x05, 0x03]),
            '8' => Some([0x36, 0x49, 0x49, 0x49, 0x36]),
            '9' => Some([0x06, 0x49, 0x49, 0x29, 0x1E]),
            ':' => Some([0x00, 0x36, 0x36, 0x00, 0x00]),
            ';' => Some([0x00, 0x56, 0x36, 0x00, 0x00]),
            '<' => Some([0x08, 0x14, 0x22, 0x41, 0x00]),
            '=' => Some([0x14, 0x14, 0x14, 0x14, 0x14]),
            '>' => Some([0x00, 0x41, 0x22, 0x14, 0x08]),
            '?' => Some([0x02, 0x01, 0x51, 0x09, 0x06]),
            '@' => Some([0x32, 0x49, 0x79, 0x41, 0x3E]),
            'A' => Some([0x7E, 0x11, 0x11, 0x11, 0x7E]),
            'B' => Some([0x7F, 0x49, 0x49, 0x49, 0x36]),
            'C' => Some([0x3E, 0x41, 0x41, 0x41, 0x22]),
            'D' => Some([0x7F, 0x41, 0x41, 0x22, 0x1C]),
            'E' => Some([0x7F, 0x49, 0x49, 0x49, 0x41]),
            'F' => Some([0x7F, 0x09, 0x09, 0x09, 0x01]),
            'G' => Some([0x3E, 0x41, 0x49, 0x49, 0x7A]),
            'H' => Some([0x7F, 0x08, 0x08, 0x08, 0x7F]),
            'I' => Some([0x00, 0x41, 0x7F, 0x41, 0x00]),
            'J' => Some([0x20, 0x40, 0x41, 0x3F, 0x01]),
            'K' => Some([0x7F, 0x08, 0x14, 0x22, 0x41]),
            'L' => Some([0x7F, 0x40, 0x40, 0x40, 0x40]),
            'M' => Some([0x7F, 0x02, 0x0C, 0x02, 0x7F]),
            'N' => Some([0x7F, 0x04, 0x08, 0x10, 0x7F]),
            'O' => Some([0x3E, 0x41, 0x41, 0x41, 0x3E]),
            'P' => Some([0x7F, 0x09, 0x09, 0x09, 0x06]),
            'Q' => Some([0x3E, 0x41, 0x51, 0x21, 0x5E]),
            'R' => Some([0x7F, 0x09, 0x19, 0x29, 0x46]),
            'S' => Some([0x46, 0x49, 0x49, 0x49, 0x31]),
            'T' => Some([0x01, 0x01, 0x7F, 0x01, 0x01]),
            'U' => Some([0x3F, 0x40, 0x40, 0x40, 0x3F]),
            'V' => Some([0x1F, 0x20, 0x40, 0x20, 0x1F]),
            'W' => Some([0x3F, 0x40, 0x38, 0x40, 0x3F]),
            'X' => Some([0x63, 0x14, 0x08, 0x14, 0x63]),
            'Y' => Some([0x07, 0x08, 0x70, 0x08, 0x07]),
            'Z' => Some([0x61, 0x51, 0x49, 0x45, 0x43]),
            '[' => Some([0x00, 0x7F, 0x41, 0x41, 0x00]),
            '\\' => Some([0x02, 0x04, 0x08, 0x10, 0x20]),
            ']' => Some([0x00, 0x41, 0x41, 0x7F, 0x00]),
            '^' => Some([0x04, 0x02, 0x01, 0x02, 0x04]),
            '_' => Some([0x40, 0x40, 0x40, 0x40, 0x40]),
            '`' => Some([0x00, 0x01, 0x02, 0x04, 0x00]),
            _ => None,
        }
    }

    /// Render a single character
    fn render_char(&self, canvas: &mut Canvas<Window>, c: char, x: i32, y: i32, color: Color) {
        if let Some(bitmap) = Self::get_char_bitmap(c) {
            canvas.set_draw_color(color);
            for (col, &byte) in bitmap.iter().enumerate() {
                for row in 0..7 {
                    if (byte >> row) & 1 == 1 {
                        let px = x + (col as i32) * self.scale;
                        let py = y + (row as i32) * self.scale;
                        let _ = canvas.fill_rect(Rect::new(px, py, self.scale as u32, self.scale as u32));
                    }
                }
            }
        }
    }

    /// Render text
    pub fn render_text(&self, canvas: &mut Canvas<Window>, text: &str, x: i32, y: i32, color: Color) {
        let char_width = 6 * self.scale;
        for (i, c) in text.chars().enumerate() {
            self.render_char(canvas, c, x + (i as i32) * char_width, y, color);
        }
    }

    /// Render centered text
    pub fn render_text_centered(&self, canvas: &mut Canvas<Window>, text: &str, center_x: i32, y: i32, color: Color) {
        let width = self.text_width(text);
        self.render_text(canvas, text, center_x - width / 2, y, color);
    }

    /// Calculate text width
    pub fn text_width(&self, text: &str) -> i32 {
        text.len() as i32 * 6 * self.scale
    }

    /// Get line height
    pub fn line_height(&self) -> i32 {
        7 * self.scale + 2
    }
}
