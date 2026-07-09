//! CL2 (Compressed CEL, multi-group) sprite-sheet loader.
//!
//! Diablo's player/monster sprites ship as `.cl2` files. A CL2 file is either:
//!
//! * **Single-group**: `[u32 num_frames][u32 offsets.. num_frames+1][frame data]`
//! * **Multi-group** (directional, e.g. 8 walk directions):
//!   `[u32 group_offset_0..N][group 0][group 1]...`
//!   where each group is itself a single-group CL2 (frame offsets are relative
//!   to the start of the group).
//!
//! Each frame begins with a 10-byte header:
//!   `[u16 header_size (=10)]` followed by 8 bytes of 32-pixel block offsets
//!   used for random access in the original engine. After the header comes the
//!   RLE pixel data, which uses the *same* control-byte encoding as CLX:
//!     - `0x00..=0x7F`: skip (transparent) N pixels
//!     - `0x80..=0xBE`: fill `(0xBF - control)` pixels with the next byte
//!     - `0xBF..=0xFF`: copy `(256 - control)` literal pixels
//!   Frames are encoded **bottom-to-top**.
//!
//! This loader turns a CL2 file into [`Cl2SpriteSheet`] / [`Cl2SpriteList`],
//! each holding [`ClxSprite`] values whose `pixel_data` is the post-header RLE
//! (so the existing `ClxSprite::decode_rgba` renders them correctly) and whose
//! `height` is computed by replaying the line wraps during decode.
//!
//! C++ Reference: `Source/utils/cl2_to_clx.cpp` (`Cl2ToClx`).

use crate::engine::clx::ClxSprite;

/// A list of CL2 frames decoded to CLX-style sprites (one direction).
#[derive(Debug, Clone)]
pub struct Cl2SpriteList {
    pub sprites: Vec<ClxSprite>,
}

impl Cl2SpriteList {
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&ClxSprite> {
        self.sprites.get(index)
    }
}

/// A directional CL2 sheet: one [`Cl2SpriteList`] per group/direction.
#[derive(Debug, Clone)]
pub struct Cl2SpriteSheet {
    pub lists: Vec<Cl2SpriteList>,
}

impl Cl2SpriteSheet {
    /// Number of groups/directions.
    pub fn num_lists(&self) -> usize {
        self.lists.len()
    }

    pub fn get(&self, direction: usize) -> Option<&Cl2SpriteList> {
        self.lists.get(direction)
    }

    /// First frame of the given direction, if any. Convenience for the common
    /// "show the standing frame of direction N" case.
    pub fn first_frame(&self, direction: usize) -> Option<&ClxSprite> {
        self.get(direction).and_then(|l| l.get(0))
    }
}

/// Parse a CL2 file into a directional sprite sheet.
///
/// `frame_width` is the per-frame pixel width (CL2 frames do not store it).
/// For Diablo player sprites this comes from the per-class `sprites.tsv`
/// (`walk`/`stand`/...); the Warrior walk width is 96.
///
/// Returns a sheet with one list per CL2 group. A single-group file yields a
/// sheet with exactly one list.
pub fn parse_cl2_sheet(data: &[u8], frame_width: u16) -> Option<Cl2SpriteSheet> {
    if data.len() < 8 {
        return None;
    }
    let maybe_first = le_u32(data, 0)? as usize;

    // Determine single-group vs multi-group: in a single-group file, the offset
    // stored at `maybe_first * 4 + 4` equals the file size (it's the end offset
    // of the last frame). Otherwise `maybe_first` is the byte offset of group 0
    // and the number of groups is `maybe_first / 4`.
    let group_table_end = maybe_first.saturating_mul(4).saturating_add(4);
    let is_single = if group_table_end + 4 <= data.len() {
        le_u32(data, group_table_end)? as usize == data.len()
    } else {
        true
    };

    if is_single {
        let list = parse_cl2_group(data, frame_width)?;
        return Some(Cl2SpriteSheet { lists: vec![list] });
    }

    let num_groups = maybe_first / 4;
    if num_groups == 0 || maybe_first + num_groups * 4 > data.len() {
        return None;
    }

    // Read group offsets: the leading `num_groups` u32s. Each points at the
    // group's header (`[num_frames][frame offsets...]`). CL2 multi-group files
    // lay out *all* group headers first, then the frame data, and each group's
    // frame offsets are relative to its own header — so a group's frames may
    // physically extend past the next group's header. We therefore pass each
    // group the slice from its header to the *end of the file* and let it read
    // frames by offset.
    let mut group_offsets: Vec<usize> = Vec::with_capacity(num_groups);
    for i in 0..num_groups {
        group_offsets.push(le_u32(data, i * 4)? as usize);
    }

    let mut lists = Vec::with_capacity(num_groups);
    for i in 0..num_groups {
        let start = group_offsets[i];
        if start >= data.len() {
            continue;
        }
        if let Some(list) = parse_cl2_group(&data[start..], frame_width) {
            lists.push(list);
        }
    }
    if lists.is_empty() {
        return None;
    }
    Some(Cl2SpriteSheet { lists })
}

/// Parse a single CL2 group (frame-offset table + frame data) into a sprite list.
///
/// `data` points at the group's own header: `[u32 num_frames][u32 offsets..][frames]`.
/// Frame offsets are relative to the start of this group slice.
fn parse_cl2_group(data: &[u8], frame_width: u16) -> Option<Cl2SpriteList> {
    if data.len() < 8 {
        return None;
    }
    let num_frames = le_u32(data, 0)? as usize;
    let table_end = 4 + (num_frames + 1) * 4;
    if data.len() < table_end {
        return None;
    }

    let mut sprites = Vec::with_capacity(num_frames);
    for i in 0..num_frames {
        let begin = le_u32(data, 4 + i * 4)? as usize;
        let end = le_u32(data, 4 + (i + 1) * 4)? as usize;
        if begin >= data.len() || end > data.len() || begin >= end {
            continue;
        }
        let frame = &data[begin..end];
        if let Some(sprite) = parse_cl2_frame(frame, frame_width) {
            sprites.push(sprite);
        }
    }
    if sprites.is_empty() {
        return None;
    }
    Some(Cl2SpriteList { sprites })
}

/// Parse a single CL2 frame into a [`ClxSprite`].
///
/// The frame header's first `u16` is the header size (typically 10). We strip
/// it and compute the height by replaying the RLE line wraps, since the CL2
/// header does not store an explicit height.
fn parse_cl2_frame(frame: &[u8], frame_width: u16) -> Option<ClxSprite> {
    if frame.len() < 2 {
        return None;
    }
    let header_size = le_u16(frame, 0)? as usize;
    if header_size > frame.len() {
        return None;
    }
    let pixel_data = frame[header_size..].to_vec();

    let height = compute_frame_height(&pixel_data, frame_width);

    Some(ClxSprite {
        width: frame_width,
        height,
        pixel_data,
    })
}

/// Compute a CL2/CLX frame's height by replaying the RLE stream and counting
/// how many times the x cursor wraps past `frame_width`.
///
/// This mirrors the `GetSkipSize` accounting in C++ `Cl2ToClx`. RLE control
/// bytes have the same meaning as CLX (see module docs).
fn compute_frame_height(pixel_data: &[u8], frame_width: u16) -> u16 {
    let fw = frame_width as i32;
    if fw <= 0 {
        return 0;
    }
    let mut x: i32 = 0;
    let mut height: u16 = 0;
    let mut i = 0;
    while i < pixel_data.len() {
        let control = pixel_data[i];
        i += 1;
        let advance: i32 = if control < 0x80 {
            // Transparent skip.
            control as i32
        } else if control <= 0xBE {
            // Fill: width = 0xBF - control, consumes one colour byte.
            let w = (0xBF - control) as i32;
            i += 1; // skip colour byte
            w
        } else {
            // Copy: width = 256 - control, consumes that many colour bytes.
            let w = (256 - control as i32) as i32;
            i += w as usize;
            w
        };
        x += advance;
        // Wrap: each time we fill a full row, move up a line.
        while x >= fw {
            x -= fw;
            height = height.saturating_add(1);
        }
    }
    height
}

#[inline]
fn le_u32(data: &[u8], pos: usize) -> Option<u32> {
    if pos + 4 > data.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        data[pos],
        data[pos + 1],
        data[pos + 2],
        data[pos + 3],
    ]))
}

#[inline]
fn le_u16(data: &[u8], pos: usize) -> Option<u16> {
    if pos + 2 > data.len() {
        return None;
    }
    Some(u16::from_le_bytes([data[pos], data[pos + 1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a tiny single-group CL2 by hand: 1 frame, width 2, height 1.
    /// Frame data: header_size=10 (10 zero bytes) + 1 opaque pixel (control 0xFF
    /// = copy 1, then colour byte) — but to make the height compute to 1 we
    /// need the x cursor to wrap. With width 2, two copied pixels wrap once.
    fn build_single_group_cl2() -> Vec<u8> {
        // Frame body: 10-byte header + RLE. RLE: copy 2 pixels (control 0xFE) + 2 colour bytes.
        let mut frame = Vec::new();
        frame.extend_from_slice(&10u16.to_le_bytes()); // header_size
        frame.extend_from_slice(&[0u8; 8]); // rest of header
        frame.push(0xFE); // copy 2 pixels (256 - 0xFE = 2)
        frame.push(5); // colour 5
        frame.push(6); // colour 6
        // Group: num_frames=1, offsets [header(8), header+frame_len].
        let frame_off = 4 + 2 * 4; // = 12
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes()); // num_frames
        data.extend_from_slice(&(frame_off as u32).to_le_bytes()); // frame 0 offset
        data.extend_from_slice(&((frame_off + frame.len()) as u32).to_le_bytes()); // end offset
        data.extend_from_slice(&frame);
        data
    }

    #[test]
    fn test_parse_single_group_cl2() {
        let data = build_single_group_cl2();
        let sheet = parse_cl2_sheet(&data, 2).expect("single-group parse");
        assert_eq!(sheet.num_lists(), 1);
        let list = sheet.get(0).expect("list 0");
        assert_eq!(list.len(), 1);
        let sprite = list.get(0).expect("frame 0");
        assert_eq!(sprite.width, 2);
        assert_eq!(sprite.height, 1, "2 pixels at width 2 => 1 row");
    }

    #[test]
    fn test_compute_frame_height_wraps() {
        // RLE: copy 4 pixels (control 0xFC = 256-4) + 4 colour bytes, width 2 => 2 rows.
        let pixels = vec![0xFC, 1, 2, 3, 4];
        assert_eq!(compute_frame_height(&pixels, 2), 2);
        // width 4 => 1 row.
        assert_eq!(compute_frame_height(&pixels, 4), 1);
    }

    #[test]
    fn test_parse_too_small_returns_none() {
        assert!(parse_cl2_sheet(&[1, 2, 3], 96).is_none());
    }

    /// Integration: decode the real Warrior town-walk CL2 from spawn.mpq and
    /// verify we get 8 directions with non-trivial frames. Skipped if the MPQ
    /// isn't present (e.g. CI without assets).
    #[test]
    fn parse_real_warrior_walk_cl2() {
        use std::path::PathBuf;
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let candidates = [
            manifest.join("spawn.mpq"),
            manifest.parent().unwrap().join("devilutionx-rs").join("spawn.mpq"),
        ];
        let mpq_path = match candidates.iter().find(|p| p.exists()) {
            Some(p) => p.clone(),
            None => {
                eprintln!("[cl2_sheet] spawn.mpq not present; skipping real-asset test");
                return;
            }
        };
        let mut archive = crate::engine::mpq::MpqArchive::open(&mpq_path).expect("open spawn.mpq");
        let data = archive
            .read_file("plrgfx\\warrior\\wln\\wlnwl.cl2")
            .expect("read wlnwl.cl2");

        let sheet = parse_cl2_sheet(&data, 96).expect("parse warrior walk sheet");
        // 8 directional groups.
        assert_eq!(sheet.num_lists(), 8, "expected 8 walk directions");
        // Each direction has 8 walk frames.
        let dir0 = sheet.get(0).expect("direction 0");
        assert!(dir0.len() >= 1, "direction 0 has frames");
        let frame0 = dir0.get(0).expect("direction 0 frame 0");
        assert_eq!(frame0.width, 96);
        assert!(frame0.height > 0, "frame0 height computed");
        assert!(!frame0.pixel_data.is_empty(), "frame0 has pixel data");

        // Sanity: decode to RGBA with the real town palette and confirm the
        // sprite has both opaque pixels and non-trivial (non-near-black) colour.
        let pal_bytes = archive.read_file("levels\\towndata\\town.pal").expect("read town.pal");
        let mut palette = [0u8; 768];
        palette.copy_from_slice(&pal_bytes[..768]);
        let rgba = frame0.decode_rgba(&palette);
        assert_eq!(rgba.len(), 96 * frame0.height as usize * 4);
        let opaque = rgba.chunks_exact(4).filter(|c| c[3] > 0).count();
        assert!(opaque > 0, "expected some opaque pixels, got {}", opaque);
        // At least one opaque pixel must be noticeably brighter than black
        // (proves the palette lookup is happening on real indices).
        let bright = rgba
            .chunks_exact(4)
            .filter(|c| c[3] > 0 && (c[0] as u32 + c[1] as u32 + c[2] as u32) > 60)
            .count();
        assert!(bright > 0, "expected some brightly-coloured pixels");
    }
}
