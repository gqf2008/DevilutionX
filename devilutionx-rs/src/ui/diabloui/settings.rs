//! Settings menu model (C++ `Source/DiabloUI/settingsmenu.cpp` +
//! `Source/options.h`).
//!
//! The Rust port uses a custom UI framework, so this module ports the
//! *model* behind the C++ settings menu: the option categories and entries
//! (`OptionCategoryBase` / `OptionEntryBase`) with their display names,
//! current-value descriptions and value-cycling behaviour, wired to the live
//! engine options (`utils::options`). A screen can render this model directly;
//! the C++ `settingsmenu.cpp` state machine (Categories -> Settings ->
//! ListOption -> KeyInput/PadInput) is a follow-up.

use crate::utils::options as opt;

/// C++ `OptionEntryType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsEntryType {
    Boolean,
    List,
}

/// A single settings entry (C++ `OptionEntryBase`).
pub struct SettingsEntry {
    /// C++ `GetName()` (display name, e.g. "Auto Gold Pickup").
    pub name: &'static str,
    /// C++ `GetDescription()`.
    pub description: &'static str,
    pub kind: SettingsEntryType,
    value: Box<dyn Fn() -> String>,
    /// C++ `ChangeOptionValue`: advance/cycle the value. Returns `true` when
    /// the UI should refresh the value description.
    cycle: Box<dyn Fn() -> bool>,
    /// C++ `OptionEntryListBase` value list (list-type entries only).
    list_values: Option<&'static [i32]>,
    list_get: Option<fn(&opt::Options) -> i32>,
    list_set: Option<fn(&mut opt::Options, i32)>,
}

impl SettingsEntry {
    /// C++ `GetValueDescription()`.
    pub fn value_description(&self) -> String {
        (self.value)()
    }

    /// C++ `ChangeOptionValue(pEntry, ...)` for boolean / short-list entries.
    pub fn change(&self) -> bool {
        (self.cycle)()
    }

    /// C++ `OptionEntryListBase::GetListSize()`.
    pub fn list_size(&self) -> usize {
        self.list_values.map_or(0, |v| v.len())
    }

    /// C++ `GetListValue(listIndex)`.
    pub fn list_value(&self, idx: usize) -> Option<String> {
        self.list_values.and_then(|v| v.get(idx)).map(|x| x.to_string())
    }

    /// C++ `OptionEntryListBase::GetActiveListIndex()`.
    pub fn active_index(&self) -> usize {
        let Some(values) = self.list_values else { return 0 };
        let Some(get) = self.list_get else { return 0 };
        values.iter().position(|&v| get(&opt::options()) == v).unwrap_or(0)
    }

    /// C++ `ChangeOptionValue(pEntry, listIndex)`: write the chosen value.
    pub fn set_active_index(&self, idx: usize) -> bool {
        let (Some(values), Some(set)) = (self.list_values, self.list_set) else {
            return false;
        };
        let Some(&value) = values.get(idx) else { return false };
        let mut o = opt::options_mut();
        set(&mut o, value);
        true
    }
}

/// A settings category (C++ `OptionCategoryBase`).
pub struct SettingsCategory {
    /// C++ INI-section key (e.g. `"Game"` for the Gameplay category).
    pub key: &'static str,
    /// C++ `GetName()` (e.g. "Gameplay").
    pub name: &'static str,
    /// C++ `GetDescription()`.
    pub description: &'static str,
    pub entries: Vec<SettingsEntry>,
}

/// Build a boolean entry bound to a live `utils::options` field.
fn boolean(
    name: &'static str,
    description: &'static str,
    get: fn(&opt::Options) -> bool,
    set: fn(&mut opt::Options, bool),
) -> SettingsEntry {
    SettingsEntry {
        name,
        description,
        kind: SettingsEntryType::Boolean,
        value: Box::new(move || if get(&opt::options()) { "On".to_string() } else { "Off".to_string() }),
        cycle: Box::new(move || {
            let mut o = opt::options_mut();
            let next = !get(&o);
            set(&mut o, next);
            true
        }),
        list_values: None,
        list_get: None,
        list_set: None,
    }
}

/// Build a list entry (C++ `OptionEntryListBase`). Lists with 2 values cycle
/// directly on Enter (C++ settingsmenu.cpp:292-296); larger lists open the
/// ListOption submenu.
fn list(
    name: &'static str,
    description: &'static str,
    values: &'static [i32],
    get: fn(&opt::Options) -> i32,
    set: fn(&mut opt::Options, i32),
) -> SettingsEntry {
    SettingsEntry {
        name,
        description,
        kind: SettingsEntryType::List,
        value: Box::new(move || {
            let current = get(&opt::options());
            values
                .iter()
                .find(|&&v| v == current)
                .map_or_else(|| current.to_string(), |v| v.to_string())
        }),
        cycle: Box::new(move || {
            if values.len() > 2 {
                return false; // handled by the ListOption submenu
            }
            let mut o = opt::options_mut();
            let current = get(&o);
            let next = values
                .iter()
                .position(|&v| v == current)
                .map(|i| values[(i + 1) % values.len()])
                .unwrap_or(values[0]);
            set(&mut o, next);
            true
        }),
        list_values: Some(values),
        list_get: Some(get),
        list_set: Some(set),
    }
}

/// Ported option categories in C++ `GetCategories()` relative order.
///
/// C++ order (options.h:893-911): Language, Mods, GameMode, StartUp,
/// Graphics, Audio, Diablo, Hellfire, Gameplay, Controller, Network, Chat,
/// Keymapper, Padmapper. Only the categories with a live Rust `Options`
/// mapping are ported; their relative order matches C++.
pub fn settings_categories() -> Vec<SettingsCategory> {
    vec![
        SettingsCategory {
            key: "Language",
            name: "Language",
            description: "Language Settings",
            entries: vec![list(
                "Language",
                "Select the language used by the game.",
                &[0, 1],
                |o| if o.language.code.starts_with("zh") { 1 } else { 0 },
                |o, v| o.language.code = if v == 1 { "zh_CN".to_string() } else { "en".to_string() },
            )],
        },
        SettingsCategory {
            key: "Graphics",
            name: "Graphics",
            description: "Graphics Settings",
            entries: vec![
                boolean("Fullscreen", "Run the game in fullscreen mode.", |o| o.graphics.fullscreen, |o, v| o.graphics.fullscreen = v),
                boolean("Fit to Screen", "Scale the image to fit the screen.", |o| o.graphics.fit_to_screen, |o, v| o.graphics.fit_to_screen = v),
                boolean("Integer Scaling", "Use integer scaling ratios.", |o| o.graphics.integer_scaling, |o, v| o.graphics.integer_scaling = v),
                boolean("Per-Pixel Lighting", "Render lighting per pixel.", |o| o.graphics.per_pixel_lighting, |o, v| o.graphics.per_pixel_lighting = v),
                boolean("Color Cycling", "Animate the palette color cycling.", |o| o.graphics.color_cycling, |o, v| o.graphics.color_cycling = v),
                boolean("Hardware Cursor", "Use a hardware cursor.", |o| o.graphics.hardware_cursor, |o, v| o.graphics.hardware_cursor = v),
                boolean("Show FPS", "Display the current framerate.", |o| o.graphics.show_fps, |o, v| o.graphics.show_fps = v),
            ],
        },
        SettingsCategory {
            key: "Audio",
            name: "Audio",
            description: "Audio Settings",
            entries: vec![
                list("Sample Rate", "Output sample rate (Hz).", &[22050, 44100, 48000], |o| o.audio.sample_rate as i32, |o, v| o.audio.sample_rate = v as u32),
                list("Channels", "Number of output channels.", &[1, 2], |o| o.audio.channels as i32, |o, v| o.audio.channels = v as u8),
                list("Buffer Size", "Buffer size (number of frames per channel).", &[1024, 2048, 5120], |o| o.audio.buffer_size as i32, |o, v| o.audio.buffer_size = v as u32),
                boolean("Walking Sound", "Player emits sound when walking.", |o| o.audio.walking_sound, |o, v| o.audio.walking_sound = v),
                boolean("Auto Equip Sound", "Automatically equipping items on pickup emits the equipment sound.", |o| o.audio.auto_equip_sound, |o, v| o.audio.auto_equip_sound = v),
                boolean("Item Pickup Sound", "Picking up items emits the items pickup sound.", |o| o.audio.item_pickup_sound, |o, v| o.audio.item_pickup_sound = v),
            ],
        },
        SettingsCategory {
            key: "Game",
            name: "Gameplay",
            description: "Gameplay Settings",
            entries: vec![
                boolean("Run in Town", "Enable jogging/fast walking in town for Diablo and Hellfire. This option was introduced in the expansion.", |o| o.gameplay.run_in_town, |o, v| o.gameplay.run_in_town = v),
                boolean("Grab Input", "When enabled mouse is locked to the game window.", |o| o.gameplay.grab_input, |o, v| o.gameplay.grab_input = v),
                boolean("Pause Game When Window Loses Focus", "When enabled, the game will pause when focus is lost.", |o| o.gameplay.pause_on_focus_loss, |o, v| o.gameplay.pause_on_focus_loss = v),
                boolean("Friendly Fire", "Allow arrow/spell damage between players in multiplayer even when the friendly mode is on.", |o| o.gameplay.friendly_fire, |o, v| o.gameplay.friendly_fire = v),
                boolean("Experience Bar", "Experience Bar is added to the UI at the bottom of the screen.", |o| o.gameplay.experience_bar, |o, v| o.gameplay.experience_bar = v),
                boolean("Enemy Health Bar", "Enemy Health Bar is displayed at the top of the screen.", |o| o.gameplay.enemy_health_bar, |o, v| o.gameplay.enemy_health_bar = v),
                boolean("Auto Gold Pickup", "Gold is automatically collected when in close proximity to the player.", |o| o.gameplay.auto_gold_pickup, |o, v| o.gameplay.auto_gold_pickup = v),
                boolean("Auto Elixir Pickup", "Elixirs are automatically collected when in close proximity to the player.", |o| o.gameplay.auto_elixir_pickup, |o, v| o.gameplay.auto_elixir_pickup = v),
                boolean("Auto Oil Pickup", "Oils are automatically collected when in close proximity to the player.", |o| o.gameplay.auto_oil_pickup, |o, v| o.gameplay.auto_oil_pickup = v),
                boolean("Auto Pickup in Town", "Automatically pickup items in town.", |o| o.gameplay.auto_pickup_in_town, |o, v| o.gameplay.auto_pickup_in_town = v),
                boolean("Auto Equip Weapons", "Weapons will be automatically equipped on pickup or purchase if enabled.", |o| o.gameplay.auto_equip_weapons, |o, v| o.gameplay.auto_equip_weapons = v),
                boolean("Auto Equip Armor", "Armor will be automatically equipped on pickup or purchase if enabled.", |o| o.gameplay.auto_equip_armor, |o, v| o.gameplay.auto_equip_armor = v),
                boolean("Auto Equip Helms", "Helms will be automatically equipped on pickup or purchase if enabled.", |o| o.gameplay.auto_equip_helms, |o, v| o.gameplay.auto_equip_helms = v),
                boolean("Auto Equip Shields", "Shields will be automatically equipped on pickup or purchase if enabled.", |o| o.gameplay.auto_equip_shields, |o, v| o.gameplay.auto_equip_shields = v),
                boolean("Auto Equip Jewelry", "Jewelry will be automatically equipped on pickup or purchase if enabled.", |o| o.gameplay.auto_equip_jewelry, |o, v| o.gameplay.auto_equip_jewelry = v),
                boolean("Randomize Quests", "Randomly selecting available quests for new games.", |o| o.gameplay.randomize_quests, |o, v| o.gameplay.randomize_quests = v),
                boolean("Show Monster Type", "Hovering over a monster will display the type of monster in the description box in the UI.", |o| o.gameplay.show_monster_type, |o, v| o.gameplay.show_monster_type = v),
                boolean("Show Item Labels", "Show labels for items on the ground when enabled.", |o| o.gameplay.show_item_labels, |o, v| o.gameplay.show_item_labels = v),
                boolean("Auto Refill Belt", "Refill belt from inventory when belt item is consumed.", |o| o.gameplay.auto_refill_belt, |o, v| o.gameplay.auto_refill_belt = v),
                boolean("Disable Crippling Shrines", "When enabled Cauldrons, Fascinating Shrines, Goat Shrines, Ornate Shrines, Sacred Shrines and Murphy's Shrines are not able to be clicked on and labeled as disabled.", |o| o.gameplay.disable_crippling_shrines, |o, v| o.gameplay.disable_crippling_shrines = v),
                boolean("Quick Cast", "Spell hotkeys instantly cast the spell, rather than switching the readied spell.", |o| o.gameplay.quick_cast, |o, v| o.gameplay.quick_cast = v),
                boolean("Show health values", "Displays current / max health value on health globe.", |o| o.gameplay.show_health_values, |o, v| o.gameplay.show_health_values = v),
                boolean("Show mana values", "Displays current / max mana value on mana globe.", |o| o.gameplay.show_mana_values, |o, v| o.gameplay.show_mana_values = v),
            ],
        },
        SettingsCategory {
            key: "Controller",
            name: "Controller",
            description: "Controller Settings",
            entries: vec![
                boolean("Controller Enabled", "Enable controller input.", |o| o.controller.enabled, |o, v| o.controller.enabled = v),
                boolean("Rumble", "Enable controller rumble.", |o| o.controller.rumble, |o, v| o.controller.rumble = v),
            ],
        },
    ]
}

/// Two-level settings navigation (C++ `ShownMenuType::Categories` /
/// `ShownMenuType::Settings`). The deeper ListOption / KeyInput / PadInput
/// levels are follow-ups.
pub struct SettingsMenu {
    categories: Vec<SettingsCategory>,
    active_category: Option<usize>,
    /// C++ `ShownMenuType::ListOption`: the entry index whose value list is
    /// currently shown.
    list_entry: Option<usize>,
    selected: usize,
}

impl SettingsMenu {
    pub fn new() -> Self {
        Self {
            categories: settings_categories(),
            active_category: None,
            list_entry: None,
            selected: 0,
        }
    }

    /// Number of rows in the current level: categories, a category's entries,
    /// or a list entry's values (C++ `ShownMenuType`).
    pub fn item_count(&self) -> usize {
        let Some(ci) = self.active_category else {
            return self.categories.len();
        };
        if let Some(ei) = self.list_entry {
            return self.categories[ci].entries[ei].list_size();
        }
        self.categories[ci].entries.len()
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn set_selection(&mut self, idx: usize) {
        if idx < self.item_count() {
            self.selected = idx;
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let count = self.item_count();
        if count == 0 {
            return;
        }
        self.selected = (self.selected as i32 + delta).rem_euclid(count as i32) as usize;
    }

    pub fn in_categories(&self) -> bool {
        self.active_category.is_none()
    }

    /// True when the value-list submenu (C++ `ListOption`) is shown.
    pub fn in_list_option(&self) -> bool {
        self.list_entry.is_some()
    }

    /// Screen title (C++ shows the category / option name in the sub-levels).
    pub fn title(&self) -> String {
        let Some(ci) = self.active_category else {
            return "Settings".to_string();
        };
        if let Some(ei) = self.list_entry {
            return self.categories[ci].entries[ei].name.to_string();
        }
        self.categories[ci].name.to_string()
    }

    /// Row label: category name, "Entry: current value", or a list value.
    pub fn row_label(&self, idx: usize) -> String {
        let Some(ci) = self.active_category else {
            return self.categories[idx].name.to_string();
        };
        if let Some(ei) = self.list_entry {
            return self.categories[ci].entries[ei]
                .list_value(idx)
                .unwrap_or_default();
        }
        let entry = &self.categories[ci].entries[idx];
        format!("{}: {}", entry.name, entry.value_description())
    }

    /// Enter: drill into a category / value list, or apply the selected value
    /// (C++ `ItemSelected`). Returns `true` when a value changed.
    pub fn activate(&mut self) -> bool {
        let Some(ci) = self.active_category else {
            if self.selected < self.categories.len() {
                self.active_category = Some(self.selected);
                self.selected = 0;
            }
            return false;
        };
        if let Some(ei) = self.list_entry {
            // ListOption level: pick the value, then return to the settings level.
            let changed = self.categories[ci].entries[ei].set_active_index(self.selected);
            self.list_entry = None;
            self.selected = ei; // keep the entry highlighted (C++ returns to Settings)
            return changed;
        }
        if self.selected >= self.categories[ci].entries.len() {
            return false;
        }
        let entry = &self.categories[ci].entries[self.selected];
        if entry.kind == SettingsEntryType::List && entry.list_size() > 2 {
            // Open the ListOption submenu (C++ settingsmenu.cpp:285-290).
            self.list_entry = Some(self.selected);
            self.selected = entry.active_index();
            false
        } else {
            // Booleans and 2-value lists change immediately (C++:292-296).
            entry.change()
        }
    }

    /// Esc: back one level; at the top level, `true` exits the settings
    /// screen (C++ `GoBackOneMenuLevel` -> `backToMain`).
    pub fn escape(&mut self) -> bool {
        if self.list_entry.take().is_some() {
            self.selected = 0;
            return false;
        }
        if self.active_category.take().is_some() {
            self.selected = 0;
            return false;
        }
        true
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_match_cpp_order_for_ported_subset() {
        // C++ GetCategories (options.h:893-911) relative order, restricted to
        // the categories with a live Rust Options mapping.
        let categories = settings_categories();
        let names: Vec<&str> = categories.iter().map(|c| c.name).collect();
        assert_eq!(names, vec!["Language", "Graphics", "Audio", "Gameplay", "Controller"]);
        // C++ keys: Gameplay category uses INI section "Game".
        let gameplay = categories.iter().find(|c| c.name == "Gameplay").unwrap();
        assert_eq!(gameplay.key, "Game");
        assert_eq!(gameplay.description, "Gameplay Settings");
    }

    #[test]
    fn test_gameplay_entries_match_cpp_names() {
        let categories = settings_categories();
        let gameplay = categories.iter().find(|c| c.name == "Gameplay").unwrap();
        let names: Vec<&str> = gameplay.entries.iter().map(|e| e.name).collect();
        for expected in [
            "Run in Town",
            "Grab Input",
            "Pause Game When Window Loses Focus",
            "Friendly Fire",
            "Experience Bar",
            "Enemy Health Bar",
            "Auto Gold Pickup",
            "Auto Elixir Pickup",
            "Auto Oil Pickup",
            "Auto Pickup in Town",
            "Auto Equip Weapons",
            "Show Item Labels",
            "Quick Cast",
            "Show health values",
            "Show mana values",
        ] {
            assert!(names.contains(&expected), "missing C++ entry {expected:?}");
        }
    }

    #[test]
    fn test_boolean_cycle_flips_live_option() {
        // Reset the live option, cycle it on, verify the model + the engine
        // option agree, then cycle back off.
        {
            let mut o = opt::options_mut();
            o.gameplay.run_in_town = false;
        }
        let categories = settings_categories();
        let entry = categories
            .iter()
            .find(|c| c.name == "Gameplay")
            .unwrap()
            .entries
            .iter()
            .find(|e| e.name == "Run in Town")
            .unwrap();
        assert_eq!(entry.kind, SettingsEntryType::Boolean);
        assert_eq!(entry.value_description(), "Off");

        assert!(entry.change());
        assert!(opt::options().gameplay.run_in_town, "cycle() flipped the live option");
        assert_eq!(entry.value_description(), "On");

        assert!(entry.change());
        assert!(!opt::options().gameplay.run_in_town);
        assert_eq!(entry.value_description(), "Off");
    }

    #[test]    #[test]
    fn test_settings_menu_navigation() {
        let mut menu = SettingsMenu::new();
        assert!(menu.in_categories());
        assert_eq!(menu.item_count(), 5);
        assert_eq!(menu.title(), "Settings");
        assert_eq!(menu.row_label(0), "Language");

        // Drill into Gameplay (index 3 in the ported subset order).
        menu.set_selection(3);
        menu.activate();
        assert!(!menu.in_categories());
        assert_eq!(menu.title(), "Gameplay");
        assert!(menu.item_count() > 5);
        assert!(menu.row_label(0).starts_with("Run in Town:"));

        // Cycle the selected boolean entry via activate().
        {
            let mut o = opt::options_mut();
            o.gameplay.run_in_town = false;
        }
        menu.set_selection(0);
        assert!(menu.activate(), "boolean entry change reports a refresh");
        assert!(opt::options().gameplay.run_in_town);

        // Esc returns to the category list; Esc again exits.
        assert!(!menu.escape());
        assert!(menu.in_categories());
        assert!(menu.escape());
    }



    #[test]
    fn test_settings_menu_list_option_level() {
        // Set the live value first (the options global is shared across tests).
        {
            let mut o = opt::options_mut();
            o.audio.sample_rate = 48000;
        }
        let mut menu = SettingsMenu::new();
        // Audio list entries reflect the live value (checked in the same
        // test to avoid racing the shared options global).
        let categories = settings_categories();
        let audio = categories.iter().find(|c| c.name == "Audio").unwrap();
        let sample = audio.entries.iter().find(|e| e.name == "Sample Rate").unwrap();
        assert_eq!(sample.kind, SettingsEntryType::List);
        assert_eq!(sample.list_size(), 3);
        assert_eq!(sample.value_description(), "48000");
        assert!(!sample.change(), "3-value lists open the submenu, not cycle");
        let channels = audio.entries.iter().find(|e| e.name == "Channels").unwrap();
        {
            let mut o = opt::options_mut();
            o.audio.channels = 1;
        }
        assert!(channels.change(), "2-value lists cycle directly (C++)");
        assert_eq!(opt::options().audio.channels, 2);
        // Drill into Audio (index 2) -> Sample Rate (index 0, a 3-value list).
        menu.set_selection(2);
        menu.activate();
        menu.set_selection(0);
        assert!(menu.in_categories() == false && menu.in_list_option() == false);
        // Entering the Sample Rate entry opens the ListOption submenu.
        menu.activate();
        assert!(menu.in_list_option(), "large list opens the ListOption level");
        assert_eq!(menu.title(), "Sample Rate");
        assert_eq!(menu.item_count(), 3);
        // Active value is highlighted first.
        assert_eq!(menu.selected_index(), 2, "selection starts on the active value");
        // Esc returns to the settings level without changing anything.
        assert!(!menu.escape());
        assert!(!menu.in_list_option());
        assert_eq!(opt::options().audio.sample_rate, 48000);
        // Re-enter and pick 22050.
        menu.set_selection(0);
        menu.activate();
        assert!(menu.in_list_option());
        menu.set_selection(0);
        assert!(menu.activate(), "picking a list value reports a change");
        assert_eq!(opt::options().audio.sample_rate, 22050);
        assert_eq!(menu.selected_index(), 0, "back on the entry, still highlighted");
        assert!(!menu.escape(), "settings level goes back to categories");
        assert!(menu.in_categories());
        assert!(menu.escape(), "category level exits the settings screen");
    }

}
