//! Ground item labels QoL (C++ `Source/qol/itemlabels.cpp`).
//!
//! Ports the on-ground label text (gold amounts are formatted "{value} gold",
//! everything else uses the item name), the highlight-toggle predicate, and the
//! label queue with row-based overlap avoidance (C++ `DrawItemNameLabels`).

use crate::game::items::{Item, ItemType};

/// C++ `IsHighlightingLabelsEnabled()`: the highlight key flips the show/hide
/// option while held, and labels are suppressed inside stores.
pub fn is_highlighting_enabled(
    in_store: bool,
    highlight_key_pressed: bool,
    show_item_labels: bool,
) -> bool {
    !in_store && highlight_key_pressed != show_item_labels
}

/// C++ `AddItemToLabelQueue` text: gold piles render as "{value} gold" with a
/// thousands separator (C++ `FormatInteger`), all other items use `getName()`.
pub fn item_label_text(item: &Item) -> String {
    if item.item_type == ItemType::Gold {
        format!("{} gold", format_integer(item.value))
    } else {
        item.name.clone()
    }
}

/// C++ `FormatInteger` (utils/format_int): groups digits with thousands
/// separators (`,` for the default locale).
pub fn format_integer(value: i32) -> String {
    let negative = value < 0;
    let digits: Vec<char> = value.unsigned_abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*ch);
    }
    if negative {
        format!("-{out}")
    } else {
        out
    }
}

/// A positioned ground-item label (C++ `ItemLabel`).
pub struct ItemLabel {
    pub id: i32,
    /// Text width in pixels (C++ `GetLineWidth` + margins).
    pub width: i32,
    pub x: i32,
    pub y: i32,
    pub text: String,
}

/// C++ `DrawItemNameLabels` overlap avoidance (itemlabels.cpp:147+): labels
/// are drawn in Y order; within a horizontal band (one label height) labels
/// whose X interval collides with an already-placed label are skipped
/// (C++ `UsedX` + `BorderX` spacing).
pub fn layout_non_overlapping(mut labels: Vec<ItemLabel>, band_height: i32) -> Vec<ItemLabel> {
    // C++ `BorderX = 4`: minimal horizontal space between labels.
    const BORDER_X: i32 = 4;
    labels.sort_by_key(|l| l.y);
    let mut visible: Vec<ItemLabel> = Vec::new();
    let mut used_x: Vec<i32> = Vec::new();
    let mut band_top = i32::MIN;
    for label in labels {
        if label.y >= band_top + band_height {
            used_x.clear();
            band_top = label.y;
        }
        let overlaps = used_x
            .iter()
            .any(|&ux| (label.x - ux).abs() < label.width / 2 + BORDER_X);
        if overlaps {
            continue;
        }
        used_x.push(label.x);
        visible.push(label);
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::items::Item;

    fn gold_item(value: i32) -> Item {
        Item {
            item_type: ItemType::Gold,
            value,
            ..Default::default()
        }
    }

    #[test]
    fn test_gold_label_text() {
        assert_eq!(item_label_text(&gold_item(5)), "5 gold");
        assert_eq!(item_label_text(&gold_item(1234)), "1,234 gold");
        assert_eq!(item_label_text(&gold_item(1000000)), "1,000,000 gold");
    }

    #[test]
    fn test_non_gold_label_uses_name() {
        let item = Item {
            item_type: ItemType::Sword,
            name: "Short Sword".to_string(),
            ..Default::default()
        };
        assert_eq!(item_label_text(&item), "Short Sword");
    }

    #[test]
    fn test_format_integer_grouping() {
        assert_eq!(format_integer(0), "0");
        assert_eq!(format_integer(999), "999");
        assert_eq!(format_integer(1000), "1,000");
        assert_eq!(format_integer(-1234567), "-1,234,567");
    }

    #[test]
    fn test_highlight_predicate() {
        // Labels show when the highlight key is pressed and the option is off.
        assert!(is_highlighting_enabled(false, true, false));
        // ... or when the key is released and the option is on.
        assert!(is_highlighting_enabled(false, false, true));
        // Not while in a store.
        assert!(!is_highlighting_enabled(true, true, false));
        // Not when the key matches the option state.
        assert!(!is_highlighting_enabled(false, true, true));
    }


    #[test]
    fn test_layout_keeps_non_overlapping_labels() {
        // Two labels on the same row, far apart: both kept.
        let labels = vec![
            ItemLabel { id: 1, width: 60, x: 100, y: 100, text: "Short Sword".to_string() },
            ItemLabel { id: 2, width: 40, x: 300, y: 100, text: "Gold".to_string() },
        ];
        let visible = layout_non_overlapping(labels, 12);
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn test_layout_skips_overlapping_labels() {
        // Overlapping X on the same row: the second is skipped.
        let labels = vec![
            ItemLabel { id: 1, width: 60, x: 100, y: 100, text: "Short Sword".to_string() },
            ItemLabel { id: 2, width: 60, x: 110, y: 100, text: "Long Sword".to_string() },
        ];
        let visible = layout_non_overlapping(labels, 12);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, 1, "the first (leftmost) label wins");
    }

    #[test]
    fn test_layout_allows_different_rows() {
        // Different bands do not collide.
        let labels = vec![
            ItemLabel { id: 1, width: 60, x: 100, y: 100, text: "A".to_string() },
            ItemLabel { id: 2, width: 60, x: 110, y: 130, text: "B".to_string() },
        ];
        let visible = layout_non_overlapping(labels, 12);
        assert_eq!(visible.len(), 2);
    }

}
