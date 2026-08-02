//! Floating damage numbers QoL (C++ `Source/qol/floatingnumbers.cpp`).
//!
//! Ports the display-data core (fixed-point formatting, font size, colour)
//! plus the C++ `FloatingQueue` state machine (merge by id, 2500 ms lifetime).

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

/// C++ `FloatingQueue` entry: display data + position + lifetime.
pub struct ActiveFloatingNumber {
    pub number: FloatingNumber,
    /// C++ `startPos` — the tile the number originates from.
    pub start_pos: crate::game::types::Point,
    /// Tick at which the number expires (C++ `SDL_GetTicks() + 2500`).
    pub expire_tick: u64,
    /// Tick of the last merge (C++ `lastMerge`, 100 ms window).
    pub last_merge: u64,
    /// C++ `id` — merge key (0 = never merge).
    pub id: i32,
}

/// C++ `FloatingQueue` — a FIFO of active floating numbers.
pub struct FloatingNumbers {
    queue: Vec<ActiveFloatingNumber>,
}

/// C++ 2500 ms lifetime, expressed in game ticks (~16 ms each).
pub const FLOATING_LIFETIME_TICKS: u64 = 150;

/// C++ 100 ms merge window, expressed in game ticks.
pub const FLOATING_MERGE_TICKS: u64 = 6;

impl FloatingNumbers {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// C++ `ClearFloatingNumbers`.
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ActiveFloatingNumber> {
        self.queue.iter()
    }

    /// C++ `AddFloatingNumber` (floatingnumbers.cpp:58-82): merge a number
    /// with the same non-zero id inside the 100 ms window, otherwise append
    /// with the 2500 ms lifetime.
    pub fn add(
        &mut self,
        now_tick: u64,
        start_pos: crate::game::types::Point,
        number: FloatingNumber,
        id: i32,
    ) {
        if id != 0 {
            for entry in &mut self.queue {
                if entry.id == id && now_tick.saturating_sub(entry.last_merge) <= FLOATING_MERGE_TICKS {
                    entry.number = number;
                    entry.last_merge = now_tick;
                    entry.start_pos = start_pos;
                    return;
                }
            }
        }
        self.queue.push(ActiveFloatingNumber {
            number,
            start_pos,
            expire_tick: now_tick + FLOATING_LIFETIME_TICKS,
            last_merge: now_tick,
            id,
        });
    }

    /// C++ `ClearExpiredNumbers` (floatingnumbers.cpp:36-45): drop numbers
    /// whose lifetime has passed.
    pub fn clear_expired(&mut self, now_tick: u64) {
        self.queue.retain(|n| n.expire_tick > now_tick);
    }

    /// C++ `DrawFloatingNumbers` age factor: 0 at spawn, 1 at expiry.
    pub fn age(&self, entry: &ActiveFloatingNumber, now_tick: u64) -> f32 {
        let remaining = entry.expire_tick.saturating_sub(now_tick) as f32;
        1.0 - (remaining / FLOATING_LIFETIME_TICKS as f32).min(1.0)
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


    #[test]
    fn test_floating_numbers_merge_and_expire() {
        use crate::game::types::Point;
        let mut queue = FloatingNumbers::new();
        let p = Point::new(10, 20);
        let n1 = FloatingNumber::new(DamageType::Physical, 128);

        // Add at tick 100 with id 7.
        queue.add(100, p, n1.clone(), 7);
        assert_eq!(queue.len(), 1);

        // Same id within the merge window replaces the entry (still 1).
        let n2 = FloatingNumber::new(DamageType::Fire, 192);
        queue.add(102, Point::new(11, 20), n2.clone(), 7);
        assert_eq!(queue.len(), 1, "same-id merge keeps one entry");
        let head = queue.iter().next().unwrap();
        assert_eq!(head.number.value, 192, "merged text/value is updated");

        // Different id appends.
        queue.add(102, p, n1, 8);
        assert_eq!(queue.len(), 2);

        // Not expired yet.
        queue.clear_expired(100 + FLOATING_LIFETIME_TICKS - 1);
        assert_eq!(queue.len(), 2);
        // Expired (the second entry was added at tick 102, so expires later).
        queue.clear_expired(100 + FLOATING_LIFETIME_TICKS + 3);
        assert_eq!(queue.len(), 0, "expired numbers are removed");
    }

    #[test]
    fn test_floating_numbers_age_curve() {
        use crate::game::types::Point;
        let mut queue = FloatingNumbers::new();
        queue.add(100, Point::new(5, 5), FloatingNumber::new(DamageType::Physical, 64), 1);
        let entry = queue.iter().next().unwrap();
        assert_eq!(queue.age(entry, 100), 0.0, "age 0 at spawn");
        assert!((queue.age(entry, 100 + FLOATING_LIFETIME_TICKS / 2) - 0.5).abs() < 0.001);
        assert!((queue.age(entry, 100 + FLOATING_LIFETIME_TICKS) - 1.0).abs() < 0.001);
    }

}
