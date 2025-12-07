//! CEL/CL2 Format Decoder
//!
//! Converts legacy CEL/CL2 sprite formats to modern CLX format.
//!
//! CEL Format:
//! - Header: List of frame offsets (4 bytes each)
//! - Frames: Compressed pixel data
//!
//! CL2 Format (multi-group):
//! - Header: List of group offsets
//! - Each group: CEL-formatted frames
//!
//! C++ Reference: Source/utils/cel_to_clx.cpp

use anyhow::{Context, Result};
use crate::engine::clx::{ClxSprite, ClxSpriteList, ClxSpriteSheet};

/// CEL/CL2 decoder
pub struct CelDecoder;

impl CelDecoder {
    /// Check if byte represents transparent pixels
    /// 
    /// C++ Reference: IsCelTransparent(control)
    #[inline]
    const fn is_transparent(control: u8) -> bool {
        control >= 0x80
    }
    
    /// Get width of transparent run
    /// 
    /// C++ Reference: GetCelTransparentWidth(control)
    #[inline]
    const fn get_transparent_width(control: u8) -> u8 {
        (-(control as i8)) as u8
    }
    
    /// Decode CEL data to sprite list (all frames same width)
    /// 
    /// C++ Reference: CelToClx(data, size, width)
    pub fn decode_to_sprite_list(cel_data: &[u8], frame_width: u16) -> Result<ClxSpriteList> {
        if cel_data.len() < 8 {
            anyhow::bail!("CEL data too small: {} bytes", cel_data.len());
        }
        
        let maybe_num_frames = u32::from_le_bytes([
            cel_data[0], cel_data[1], cel_data[2], cel_data[3]
        ]);
        
        // Check if this is a multi-group file
        let last_offset_pos = (maybe_num_frames * 4 + 4) as usize;
        if last_offset_pos >= cel_data.len() {
            anyhow::bail!("Invalid CEL header");
        }
        
        let last_offset = u32::from_le_bytes([
            cel_data[last_offset_pos],
            cel_data[last_offset_pos + 1],
            cel_data[last_offset_pos + 2],
            cel_data[last_offset_pos + 3],
        ]);
        
        // Single group file: last offset equals file size
        if last_offset != cel_data.len() as u32 {
            // Multi-group file - only decode first group for now
            let num_groups = maybe_num_frames / 4;
            let first_group_offset = u32::from_le_bytes([
                cel_data[0], cel_data[1], cel_data[2], cel_data[3]
            ]) as usize;
            
            return Self::decode_group(&cel_data[first_group_offset..], frame_width);
        }
        
        Self::decode_group(cel_data, frame_width)
    }
    
    /// Decode single group to sprite list
    fn decode_group(data: &[u8], frame_width: u16) -> Result<ClxSpriteList> {
        if data.len() < 8 {
            anyhow::bail!("CEL group too small");
        }
        
        let num_frames = u32::from_le_bytes([
            data[0], data[1], data[2], data[3]
        ]) as usize;
        
        if num_frames == 0 {
            anyhow::bail!("CEL has zero frames");
        }
        
        let mut sprites = Vec::with_capacity(num_frames);
        
        for frame_idx in 0..num_frames {
            let frame_offset = u32::from_le_bytes([
                data[4 * (frame_idx + 1)],
                data[4 * (frame_idx + 1) + 1],
                data[4 * (frame_idx + 1) + 2],
                data[4 * (frame_idx + 1) + 3],
            ]) as usize;
            
            let frame_end = if frame_idx + 1 < num_frames {
                u32::from_le_bytes([
                    data[4 * (frame_idx + 2)],
                    data[4 * (frame_idx + 2) + 1],
                    data[4 * (frame_idx + 2) + 2],
                    data[4 * (frame_idx + 2) + 3],
                ]) as usize
            } else {
                data.len()
            };
            
            if frame_offset >= data.len() || frame_end > data.len() {
                anyhow::bail!("Invalid frame offset");
            }
            
            let frame_data = &data[frame_offset..frame_end];
            let sprite = Self::decode_frame(frame_data, frame_width)
                .with_context(|| format!("Failed to decode frame {}", frame_idx))?;
            
            sprites.push(sprite);
        }
        
        Ok(ClxSpriteList::new(sprites))
    }
    
    /// Decode single CEL frame to CLX sprite
    /// 
    /// C++ Reference: CelToClx frame conversion loop
    fn decode_frame(frame_data: &[u8], frame_width: u16) -> Result<ClxSprite> {
        if frame_data.is_empty() {
            anyhow::bail!("Empty frame data");
        }
        
        let mut src = frame_data;
        
        // Skip CEL frame header if present (10 bytes)
        const CEL_FRAME_HEADER_SIZE: usize = 10;
        if src.len() >= 2 {
            let maybe_header_size = u16::from_le_bytes([src[0], src[1]]) as usize;
            if maybe_header_size == CEL_FRAME_HEADER_SIZE && src.len() >= CEL_FRAME_HEADER_SIZE {
                src = &src[CEL_FRAME_HEADER_SIZE..];
            }
        }
        
        // Decode pixel data
        let mut clx_data = Vec::new();
        let mut frame_height = 0u16;
        let mut transparent_run_width = 0u16;
        
        let mut pos = 0;
        while pos < src.len() {
            // Process one scanline
            let mut remaining_width = frame_width;
            
            while remaining_width > 0 {
                if pos >= src.len() {
                    break;
                }
                
                let val = src[pos];
                pos += 1;
                
                if Self::is_transparent(val) {
                    // Transparent run
                    let width = Self::get_transparent_width(val);
                    transparent_run_width += width as u16;
                    remaining_width = remaining_width.saturating_sub(width as u16);
                } else {
                    // Solid pixels
                    // Flush pending transparent run
                    if transparent_run_width > 0 {
                        Self::append_transparent_run(transparent_run_width, &mut clx_data);
                        transparent_run_width = 0;
                    }
                    
                    // Copy pixel data
                    let count = val;
                    if pos + count as usize > src.len() {
                        anyhow::bail!("CEL frame data truncated");
                    }
                    
                    Self::append_pixels_run(&src[pos..pos + count as usize], &mut clx_data);
                    pos += count as usize;
                    remaining_width = remaining_width.saturating_sub(count as u16);
                }
            }
            
            frame_height += 1;
            
            // Check if we've consumed all data
            if pos >= src.len() {
                break;
            }
        }
        
        // Flush final transparent run
        if transparent_run_width > 0 {
            Self::append_transparent_run(transparent_run_width, &mut clx_data);
        }
        
        Ok(ClxSprite {
            width: frame_width,
            height: frame_height,
            pixel_data: clx_data,
        })
    }
    
    /// Append transparent run to CLX data
    /// 
    /// CLX encoding: 0x00-0x7F = skip N pixels
    fn append_transparent_run(width: u16, data: &mut Vec<u8>) {
        let mut remaining = width;
        while remaining > 0 {
            let chunk = remaining.min(0x7F);
            data.push(chunk as u8);
            remaining -= chunk;
        }
    }
    
    /// Append pixel run to CLX data
    /// 
    /// CLX encoding: 0xBF-0xFF = copy (256 - control) pixels
    fn append_pixels_run(pixels: &[u8], data: &mut Vec<u8>) {
        let mut pos = 0;
        while pos < pixels.len() {
            let chunk_size = (pixels.len() - pos).min(65) as u8;
            let control = 256u16 - chunk_size as u16;
            data.push(control as u8);
            data.extend_from_slice(&pixels[pos..pos + chunk_size as usize]);
            pos += chunk_size as usize;
        }
    }
    
    /// Decode CEL data to sprite sheet (different width per frame)
    /// 
    /// C++ Reference: CelToClx(data, size, widths[])
    pub fn decode_to_sprite_sheet(cel_data: &[u8], frame_widths: &[u16]) -> Result<ClxSpriteSheet> {
        if cel_data.len() < 8 {
            anyhow::bail!("CEL data too small");
        }
        
        let num_frames = u32::from_le_bytes([
            cel_data[0], cel_data[1], cel_data[2], cel_data[3]
        ]) as usize;
        
        if num_frames != frame_widths.len() {
            anyhow::bail!("Frame count mismatch: {} vs {}", num_frames, frame_widths.len());
        }
        
        let mut sprites = Vec::with_capacity(num_frames);
        
        for frame_idx in 0..num_frames {
            let frame_offset = u32::from_le_bytes([
                cel_data[4 * (frame_idx + 1)],
                cel_data[4 * (frame_idx + 1) + 1],
                cel_data[4 * (frame_idx + 1) + 2],
                cel_data[4 * (frame_idx + 1) + 3],
            ]) as usize;
            
            let frame_end = if frame_idx + 1 < num_frames {
                u32::from_le_bytes([
                    cel_data[4 * (frame_idx + 2)],
                    cel_data[4 * (frame_idx + 2) + 1],
                    cel_data[4 * (frame_idx + 2) + 2],
                    cel_data[4 * (frame_idx + 2) + 3],
                ]) as usize
            } else {
                cel_data.len()
            };
            
            let frame_data = &cel_data[frame_offset..frame_end];
            let sprite = Self::decode_frame(frame_data, frame_widths[frame_idx])
                .with_context(|| format!("Failed to decode frame {}", frame_idx))?;
            
            sprites.push(sprite);
        }
        
        Ok(ClxSpriteSheet::new(sprites))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_transparent() {
        assert!(!CelDecoder::is_transparent(0x00));
        assert!(!CelDecoder::is_transparent(0x7F));
        assert!(CelDecoder::is_transparent(0x80));
        assert!(CelDecoder::is_transparent(0xFF));
    }

    #[test]
    fn test_get_transparent_width() {
        assert_eq!(CelDecoder::get_transparent_width(0x80), 128);
        assert_eq!(CelDecoder::get_transparent_width(0xFF), 1);
        assert_eq!(CelDecoder::get_transparent_width(0xFE), 2);
    }

    #[test]
    fn test_append_transparent_run() {
        let mut data = Vec::new();
        CelDecoder::append_transparent_run(50, &mut data);
        assert_eq!(data, vec![50]);
        
        data.clear();
        CelDecoder::append_transparent_run(200, &mut data);
        assert_eq!(data, vec![0x7F, 0x7F, 46]);
    }

    #[test]
    fn test_append_pixels_run() {
        let mut data = Vec::new();
        let pixels = vec![1, 2, 3, 4, 5];
        CelDecoder::append_pixels_run(&pixels, &mut data);
        assert_eq!(data[0], 251); // 256 - 5 = 251
        assert_eq!(&data[1..], &[1, 2, 3, 4, 5]);
    }
}
