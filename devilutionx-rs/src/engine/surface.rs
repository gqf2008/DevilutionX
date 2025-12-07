//! Surface - 8-bit rendering surface
//!
//! Ported from Source/engine/surface.hpp

use crate::engine::types::{Point, Rectangle, Size};

/// 8-bit surface for rendering.
///
/// A Surface wraps a raw pixel buffer with a defined region,
/// allowing for subregion operations and pixel manipulation.
pub struct Surface<'a> {
    /// Pointer to the underlying pixel data
    pub pixels: &'a mut [u8],
    /// Pitch (bytes per row) of the underlying buffer
    pub pitch: u32,
    /// The active region within the surface
    pub region: Rectangle,
}

impl<'a> Surface<'a> {
    /// Create a new surface from a raw pixel buffer.
    pub fn new(pixels: &'a mut [u8], pitch: u32, width: i32, height: i32) -> Self {
        Self {
            pixels,
            pitch,
            region: Rectangle::new(Point::new(0, 0), Size::new(width, height)),
        }
    }

    /// Create a surface with a specific region.
    pub fn with_region(pixels: &'a mut [u8], pitch: u32, region: Rectangle) -> Self {
        Self {
            pixels,
            pitch,
            region,
        }
    }

    /// Get the width of the surface region.
    #[inline]
    pub fn w(&self) -> i32 {
        self.region.size.width
    }

    /// Get the height of the surface region.
    #[inline]
    pub fn h(&self) -> i32 {
        self.region.size.height
    }

    /// Get the pitch (bytes per row) of the underlying buffer.
    #[inline]
    pub fn pitch(&self) -> u32 {
        self.pitch
    }

    /// Get a reference to a pixel at the given coordinates.
    #[inline]
    pub fn at(&self, x: i32, y: i32) -> Option<&u8> {
        let offset = self.pixel_offset(x, y)?;
        self.pixels.get(offset)
    }

    /// Get a mutable reference to a pixel at the given coordinates.
    #[inline]
    pub fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut u8> {
        let offset = self.pixel_offset(x, y)?;
        self.pixels.get_mut(offset)
    }

    /// Calculate the byte offset for a given position.
    #[inline]
    fn pixel_offset(&self, x: i32, y: i32) -> Option<usize> {
        let abs_x = self.region.position.x + x;
        let abs_y = self.region.position.y + y;
        
        if abs_x < 0 || abs_y < 0 {
            return None;
        }
        
        let offset = (abs_x as usize) + (self.pitch as usize) * (abs_y as usize);
        if offset < self.pixels.len() {
            Some(offset)
        } else {
            None
        }
    }

    /// Get a pixel value at a point.
    #[inline]
    pub fn get_pixel(&self, point: Point) -> Option<u8> {
        self.at(point.x, point.y).copied()
    }

    /// Set a pixel value at a point if it is within bounds.
    #[inline]
    pub fn set_pixel(&mut self, point: Point, color: u8) {
        if self.in_bounds(point) {
            if let Some(pixel) = self.at_mut(point.x, point.y) {
                *pixel = color;
            }
        }
    }

    /// Check if a point is within the surface bounds.
    #[inline]
    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0
            && point.y >= 0
            && point.x < self.region.size.width
            && point.y < self.region.size.height
    }

    /// Get a row slice of the surface.
    #[inline]
    pub fn row(&self, y: i32) -> Option<&[u8]> {
        if y < 0 || y >= self.region.size.height {
            return None;
        }
        
        let abs_y = self.region.position.y + y;
        if abs_y < 0 {
            return None;
        }
        
        let start = (self.region.position.x as usize) + (self.pitch as usize) * (abs_y as usize);
        let end = start + (self.region.size.width as usize);
        
        if end <= self.pixels.len() {
            Some(&self.pixels[start..end])
        } else {
            None
        }
    }

    /// Get a mutable row slice of the surface.
    #[inline]
    pub fn row_mut(&mut self, y: i32) -> Option<&mut [u8]> {
        if y < 0 || y >= self.region.size.height {
            return None;
        }
        
        let abs_y = self.region.position.y + y;
        if abs_y < 0 {
            return None;
        }
        
        let start = (self.region.position.x as usize) + (self.pitch as usize) * (abs_y as usize);
        let end = start + (self.region.size.width as usize);
        
        if end <= self.pixels.len() {
            Some(&mut self.pixels[start..end])
        } else {
            None
        }
    }
}

/// An immutable view into a surface.
pub struct SurfaceRef<'a> {
    /// Pointer to the underlying pixel data
    pub pixels: &'a [u8],
    /// Pitch (bytes per row) of the underlying buffer
    pub pitch: u32,
    /// The active region within the surface
    pub region: Rectangle,
}

impl<'a> SurfaceRef<'a> {
    /// Create a new immutable surface reference.
    pub fn new(pixels: &'a [u8], pitch: u32, width: i32, height: i32) -> Self {
        Self {
            pixels,
            pitch,
            region: Rectangle::new(Point::new(0, 0), Size::new(width, height)),
        }
    }

    /// Get the width of the surface region.
    #[inline]
    pub fn w(&self) -> i32 {
        self.region.size.width
    }

    /// Get the height of the surface region.
    #[inline]
    pub fn h(&self) -> i32 {
        self.region.size.height
    }

    /// Get the pitch.
    #[inline]
    pub fn pitch(&self) -> u32 {
        self.pitch
    }

    /// Get a reference to a pixel at the given coordinates.
    #[inline]
    pub fn at(&self, x: i32, y: i32) -> Option<&u8> {
        let abs_x = self.region.position.x + x;
        let abs_y = self.region.position.y + y;
        
        if abs_x < 0 || abs_y < 0 {
            return None;
        }
        
        let offset = (abs_x as usize) + (self.pitch as usize) * (abs_y as usize);
        self.pixels.get(offset)
    }

    /// Get a pixel value at a point.
    #[inline]
    pub fn get_pixel(&self, point: Point) -> Option<u8> {
        self.at(point.x, point.y).copied()
    }

    /// Check if a point is within the surface bounds.
    #[inline]
    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0
            && point.y >= 0
            && point.x < self.region.size.width
            && point.y < self.region.size.height
    }

    /// Get a row slice of the surface.
    #[inline]
    pub fn row(&self, y: i32) -> Option<&[u8]> {
        if y < 0 || y >= self.region.size.height {
            return None;
        }
        
        let abs_y = self.region.position.y + y;
        if abs_y < 0 {
            return None;
        }
        
        let start = (self.region.position.x as usize) + (self.pitch as usize) * (abs_y as usize);
        let end = start + (self.region.size.width as usize);
        
        if end <= self.pixels.len() {
            Some(&self.pixels[start..end])
        } else {
            None
        }
    }
}

/// Clip a source rect and target position to the output bounds.
pub fn clip_rect(
    out_region: &Rectangle,
    src_rect: &mut Rectangle,
    target_position: &mut Point,
) {
    if target_position.x < 0 {
        src_rect.position.x -= target_position.x;
        src_rect.size.width += target_position.x;
        target_position.x = 0;
    }
    if target_position.y < 0 {
        src_rect.position.y -= target_position.y;
        src_rect.size.height += target_position.y;
        target_position.y = 0;
    }
    if target_position.x + src_rect.size.width > out_region.size.width {
        src_rect.size.width = out_region.size.width - target_position.x;
    }
    if target_position.y + src_rect.size.height > out_region.size.height {
        src_rect.size.height = out_region.size.height - target_position.y;
    }
}

/// Blit source to destination, copying all pixels.
pub fn blit(
    dst: &mut Surface,
    src: &SurfaceRef,
    src_rect: Rectangle,
    mut target_position: Point,
) {
    let mut src_rect = src_rect;
    clip_rect(&dst.region, &mut src_rect, &mut target_position);

    if src_rect.size.width <= 0 || src_rect.size.height <= 0 {
        return;
    }

    for y in 0..src_rect.size.height {
        let src_y = src_rect.position.y + y;
        let dst_y = target_position.y + y;

        if let (Some(src_row), Some(dst_row)) = (src.row(src_y), dst.row_mut(dst_y)) {
            let src_start = src_rect.position.x as usize;
            let src_end = src_start + src_rect.size.width as usize;
            let dst_start = target_position.x as usize;
            let dst_end = dst_start + src_rect.size.width as usize;

            if src_end <= src_row.len() && dst_end <= dst_row.len() {
                dst_row[dst_start..dst_end].copy_from_slice(&src_row[src_start..src_end]);
            }
        }
    }
}

/// Blit source to destination, skipping pixels with color index 0 (transparent).
pub fn blit_skip_color_index_zero(
    dst: &mut Surface,
    src: &SurfaceRef,
    src_rect: Rectangle,
    mut target_position: Point,
) {
    let mut src_rect = src_rect;
    clip_rect(&dst.region, &mut src_rect, &mut target_position);

    if src_rect.size.width <= 0 || src_rect.size.height <= 0 {
        return;
    }

    for y in 0..src_rect.size.height {
        let src_y = src_rect.position.y + y;
        let dst_y = target_position.y + y;

        if let (Some(src_row), Some(dst_row)) = (src.row(src_y), dst.row_mut(dst_y)) {
            for x in 0..src_rect.size.width {
                let src_x = (src_rect.position.x + x) as usize;
                let dst_x = (target_position.x + x) as usize;

                if src_x < src_row.len() && dst_x < dst_row.len() {
                    let pixel = src_row[src_x];
                    if pixel != 0 {
                        dst_row[dst_x] = pixel;
                    }
                }
            }
        }
    }
}

/// Fill a rectangle with a solid color.
pub fn fill_rect(dst: &mut Surface, rect: Rectangle, color: u8) {
    for y in rect.position.y..(rect.position.y + rect.size.height) {
        if let Some(row) = dst.row_mut(y) {
            let start = rect.position.x.max(0) as usize;
            let end = (rect.position.x + rect.size.width).min(row.len() as i32) as usize;
            if start < end && end <= row.len() {
                row[start..end].fill(color);
            }
        }
    }
}
