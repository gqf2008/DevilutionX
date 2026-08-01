//! Ground item labels QoL (C++ `Source/qol/itemlabels.cpp`).
//!
//! Ports the label-building core: the on-ground label text (gold amounts are
//! formatted "{value} gold", everything else uses the item name) and the
//! highlight-toggle predicate. The label queue, layout/collision handling and
//! rendering are follow-ups.

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
}
