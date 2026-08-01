//! Floating damage numbers QoL (C++ `Source/qol/floatingnumbers.cpp`).
//!
//! Ports the display-data core: fixed-point damage formatting, per-damage
//! font size, and per-type colour. The SDL rendering / merge-animation queue
//! is a follow-up.

use crate::game::combat_system::DamageType;

/// C++ `GameFont` sizes used by the floating numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Small = 12,
    Medium = 24,
    Large = 30,
}

/// C++ `UiFlags` colour choices used by the floating numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingColor {
    Gold,
    UiSilver,
    Blue,
    Orange,
    Yellow,
}

/// A single floating damage/heal number (C++ `FloatingNumber`).
#[derive(Debug, Clone)]
pub struct FloatingNumber {
    pub text: String,
    pub font_size: FontSize,
    pub color: FloatingColor,
    pub damage_type: DamageType,
    /// Fixed-point damage value (C++ `value`, shifted right 6 for display).
    pub value: i32,
}

impl FloatingNumber {
    /// C++ `UpdateFloatingData`: format the value and pick font size + colour.
    pub fn new(damage_type: DamageType, value: i32) -> Self {
        let text = format_damage(value);
        let font_size = font_size_for_damage(value);
        let color = damage_color(damage_type);
        Self {
            text,
            font_size,
            color,
            damage_type,
            value,
        }
    }
}

/// C++ `UpdateFloatingData` text: fixed-point value below 1 is shown with two
/// decimals (`value / 64.0`), otherwise the integer part (`value >> 6`).
pub fn format_damage(value: i32) -> String {
    if value > 0 && value < 64 {
        format!("{:.2}", value as f64 / 64.0)
    } else {
        (value >> 6).to_string()
    }
}

/// C++ `GetFontSizeByDamage`: value (after >> 6) >= 300 -> 30, >= 100 -> 24,
/// else 12.
pub fn font_size_for_damage(value: i32) -> FontSize {
    let v = value >> 6;
    if v >= 300 {
        FontSize::Large
    } else if v >= 100 {
        FontSize::Medium
    } else {
        FontSize::Small
    }
}

/// C++ `UpdateFloatingData` colour switch. The Rust `DamageType` lacks the
/// C++ `Acid` variant; it maps to the same yellow the C++ uses.
pub fn damage_color(damage_type: DamageType) -> FloatingColor {
    match damage_type {
        DamageType::Physical => FloatingColor::Gold,
        DamageType::Fire => FloatingColor::UiSilver,
        DamageType::Lightning => FloatingColor::Blue,
        DamageType::Magic => FloatingColor::Orange,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_damage_fixed_point() {
        // Sub-1 fixed-point values render as two decimals.
        assert_eq!(format_damage(32), "0.50");
        assert_eq!(format_damage(1), "0.02");
        // Values at/above 1 render as integers (fixed point >> 6).
        assert_eq!(format_damage(64), "1");
        assert_eq!(format_damage(128), "2");
        assert_eq!(format_damage(0), "0");
    }

    #[test]
    fn test_font_size_thresholds() {
        assert_eq!(font_size_for_damage(64), FontSize::Small); // 1
        assert_eq!(font_size_for_damage(6400), FontSize::Medium); // 100
        assert_eq!(font_size_for_damage(12800), FontSize::Medium); // 200
        assert_eq!(font_size_for_damage(19200), FontSize::Large); // 300
    }

    #[test]
    fn test_colors_match_cpp() {
        assert_eq!(damage_color(DamageType::Physical), FloatingColor::Gold);
        assert_eq!(damage_color(DamageType::Fire), FloatingColor::UiSilver);
        assert_eq!(damage_color(DamageType::Lightning), FloatingColor::Blue);
        assert_eq!(damage_color(DamageType::Magic), FloatingColor::Orange);
    }

    #[test]
    fn test_floating_number_struct() {
        let n = FloatingNumber::new(DamageType::Physical, 96); // 1.5 -> int 1
        assert_eq!(n.text, "1");
        assert_eq!(n.font_size, FontSize::Small);
        assert_eq!(n.color, FloatingColor::Gold);
        let n2 = FloatingNumber::new(DamageType::Lightning, 6400);
        assert_eq!(n2.text, "100");
        assert_eq!(n2.font_size, FontSize::Medium);
        assert_eq!(n2.color, FloatingColor::Blue);
    }
}
