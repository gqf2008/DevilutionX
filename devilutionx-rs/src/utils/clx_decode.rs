//! CLX Decode - CLX format decoding utilities
//!
//! Ported from Source/utils/clx_decode.hpp

/// Check if control byte indicates opaque pixels
///
/// # C++ Reference
/// ```cpp
/// constexpr bool IsClxOpaque(uint8_t control)
/// {
///     constexpr uint8_t ClxOpaqueMin = 0x80;
///     return control >= ClxOpaqueMin;
/// }
/// ```
#[inline]
pub const fn is_clx_opaque(control: u8) -> bool {
    const CLX_OPAQUE_MIN: u8 = 0x80;
    control >= CLX_OPAQUE_MIN
}

/// Get opaque pixels width from control byte
///
/// # C++ Reference
/// ```cpp
/// constexpr uint8_t GetClxOpaquePixelsWidth(uint8_t control)
/// {
///     return -static_cast<int8_t>(control);
/// }
/// ```
#[inline]
pub const fn get_clx_opaque_pixels_width(control: u8) -> u8 {
    (-(control as i8)) as u8
}

/// Check if control byte indicates fill command
///
/// # C++ Reference
/// ```cpp
/// constexpr bool IsClxOpaqueFill(uint8_t control)
/// {
///     constexpr uint8_t ClxFillMax = 0xBE;
///     return control <= ClxFillMax;
/// }
/// ```
#[inline]
pub const fn is_clx_opaque_fill(control: u8) -> bool {
    const CLX_FILL_MAX: u8 = 0xBE;
    control <= CLX_FILL_MAX
}

/// Get fill width from control byte
///
/// # C++ Reference
/// ```cpp
/// constexpr uint8_t GetClxOpaqueFillWidth(uint8_t control)
/// {
///     constexpr uint8_t ClxFillEnd = 0xBF;
///     return ClxFillEnd - control;
/// }
/// ```
#[inline]
pub const fn get_clx_opaque_fill_width(control: u8) -> u8 {
    const CLX_FILL_END: u8 = 0xBF;
    CLX_FILL_END - control
}

/// Skip size information for line overrun handling
///
/// # C++ Reference
/// ```cpp
/// struct SkipSize {
///     int_fast16_t wholeLines;
///     int_fast16_t xOffset;
/// };
/// ```
pub struct SkipSize {
    pub whole_lines: i16,
    pub x_offset: i16,
}

/// Returns the number of lines and the x-offset by which the rendering has overrun
/// the current line (when a CLX command overruns the current line).
///
/// Requires: remaining_width <= 0.
///
/// # C++ Reference
/// ```cpp
/// SkipSize GetSkipSize(int_fast16_t remainingWidth, int_fast16_t srcWidth)
/// ```
#[inline]
pub fn get_skip_size(remaining_width: i16, src_width: i16) -> SkipSize {
    // Remaining width of 0 (= no overrun) is a common case.
    if remaining_width == 0 {
        return SkipSize {
            whole_lines: 1,
            x_offset: 0,
        };
    }

    let overrun = (-remaining_width) as u16;
    let overrun_lines = overrun / src_width as u16 + 1;
    let x_offset = overrun % src_width as u16;

    SkipSize {
        whole_lines: overrun_lines as i16,
        x_offset: x_offset as i16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_clx_opaque() {
        assert!(!is_clx_opaque(0x7F));
        assert!(is_clx_opaque(0x80));
        assert!(is_clx_opaque(0xFF));
    }

    #[test]
    fn test_is_clx_opaque_fill() {
        assert!(is_clx_opaque_fill(0x80));
        assert!(is_clx_opaque_fill(0xBE));
        assert!(!is_clx_opaque_fill(0xBF));
        assert!(!is_clx_opaque_fill(0xFF));
    }

    #[test]
    fn test_get_clx_opaque_fill_width() {
        assert_eq!(get_clx_opaque_fill_width(0xBE), 1);
        assert_eq!(get_clx_opaque_fill_width(0x80), 63);
    }

    #[test]
    fn test_get_skip_size_no_overrun() {
        let skip = get_skip_size(0, 64);
        assert_eq!(skip.whole_lines, 1);
        assert_eq!(skip.x_offset, 0);
    }

    #[test]
    fn test_get_skip_size_with_overrun() {
        let skip = get_skip_size(-10, 64);
        assert_eq!(skip.whole_lines, 1);
        assert_eq!(skip.x_offset, 10);
    }
}
