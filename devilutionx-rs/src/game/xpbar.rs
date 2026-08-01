//! Experience bar QoL (C++ `Source/qol/xpbar.cpp`).
//!
//! Ports the bar-fill computation from `DrawXPBar`: the full-bar width and
//! the fade of the end pixel for the current XP within the level. Max-level
//! characters draw a solid golden bar (handled by the caller). The CLX
//! sprite rendering is a follow-up.

/// C++ `BarWidth` in xpbar.cpp.
pub const XP_BAR_WIDTH: u32 = 307;

/// C++ `SilverGradient` length (`gradient.size() - 1` used in the fade).
pub const XP_GRADIENT_STEPS: u32 = 12;

/// Result of computing the XP bar fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XpBarMetrics {
    /// Full-brightness bar width (C++ `fullBar`).
    pub full_bar: u32,
    /// Fade index for the end pixel (C++ `fade`, 0..gradient-1).
    pub fade: u32,
}

/// C++ `DrawXPBar` geometry:
///
/// ```cpp
/// prevXpDelta1 = player._pExperience - prevXp;
/// prevXpDelta   = nextXp - prevXp;
/// fullBar       = BarWidth * prevXpDelta1 / prevXpDelta;
/// onePx         = prevXpDelta / BarWidth + 1;
/// lastFullPx    = fullBar * prevXpDelta / BarWidth;
/// fade          = (prevXpDelta1 - lastFullPx) * (Gradient-1) / onePx;
/// ```
///
/// Returns `None` when the player has not yet reached `prev_xp` (C++ early
/// return). `max_level` should be reported by the caller before calling.
pub fn xp_bar_metrics(
    bar_width: u32,
    current_xp: u64,
    prev_xp: u64,
    next_xp: u64,
) -> Option<XpBarMetrics> {
    if current_xp < prev_xp || next_xp <= prev_xp {
        return None;
    }
    let delta1 = current_xp - prev_xp;
    let delta = next_xp - prev_xp;
    let full_bar = (bar_width as u64 * delta1) / delta;
    let one_px = delta / bar_width as u64 + 1;
    let last_full_px = full_bar * delta / bar_width as u64;
    let fade = (delta1 - last_full_px) * (XP_GRADIENT_STEPS as u64 - 1) / one_px;
    Some(XpBarMetrics {
        full_bar: full_bar as u32,
        fade: fade as u32,
    })
}

/// C++ `player.isMaxCharacterLevel()` — the caller checks this before drawing
/// the solid golden bar; exposed here for the constant (max Diablo level).
pub const MAX_CHARACTER_LEVEL: u8 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_level_bar() {
        // prev=1000, next=2000, current=1500 -> half the 307px bar.
        let m = xp_bar_metrics(XP_BAR_WIDTH, 1500, 1000, 2000).unwrap();
        assert_eq!(m.full_bar, 153);
        assert!(m.fade <= XP_GRADIENT_STEPS - 1);
    }

    #[test]
    fn test_full_bar_at_next_level() {
        let m = xp_bar_metrics(XP_BAR_WIDTH, 2000, 1000, 2000).unwrap();
        assert_eq!(m.full_bar, XP_BAR_WIDTH);
    }

    #[test]
    fn test_below_prev_threshold_returns_none() {
        assert!(xp_bar_metrics(XP_BAR_WIDTH, 500, 1000, 2000).is_none());
    }

    #[test]
    fn test_max_level_no_delta_returns_none() {
        assert!(xp_bar_metrics(XP_BAR_WIDTH, 5000, 5000, 5000).is_none());
    }

    #[test]
    fn test_zero_current() {
        let m = xp_bar_metrics(XP_BAR_WIDTH, 1000, 1000, 2000).unwrap();
        assert_eq!(m.full_bar, 0);
    }
}
