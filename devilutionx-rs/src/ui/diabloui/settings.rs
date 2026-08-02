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
}

impl SettingsEntry {
    /// C++ `GetValueDescription()`.
    pub fn value_description(&self) -> String {
        (self.value)()
    }

    /// C++ `ChangeOptionValue(pEntry, ...)` for boolean/list entries.
    pub fn change(&self) -> bool {
        (self.cycle)()
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
    }
}

/// Build a list entry showing the current value; cycling is a follow-up.
fn list_value(
    name: &'static str,
    description: &'static str,
    get: fn(&opt::Options) -> String,
) -> SettingsEntry {
    SettingsEntry {
        name,
        description,
        kind: SettingsEntryType::List,
        value: Box::new(move || get(&opt::options())),
        cycle: Box::new(|| false),
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
            entries: vec![list_value(
                "Language",
                "Select the language used by the game.",
                |o| o.language.code.clone(),
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
                list_value("Sound Volume", "Sound effect volume (0-100).", |o| format!("{}", o.audio.sound_volume)),
                list_value("Music Volume", "Music volume (0-100).", |o| format!("{}", o.audio.music_volume)),
                boolean("Walking Sound", "Play a sound while walking.", |o| o.audio.walking_sound, |o, v| o.audio.walking_sound = v),
                boolean("Auto Equip Sound", "Play a sound when auto-equipping.", |o| o.audio.auto_equip_sound, |o, v| o.audio.auto_equip_sound = v),
                boolean("Item Pickup Sound", "Play a sound when picking up items.", |o| o.audio.item_pickup_sound, |o, v| o.audio.item_pickup_sound = v),
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

    #[test]
    fn test_audio_entries_reflect_live_values() {
        {
            let mut o = opt::options_mut();
            o.audio.sound_volume = 73;
            o.audio.music_volume = 41;
        }
        let categories = settings_categories();
        let audio = categories.iter().find(|c| c.name == "Audio").unwrap();
        let sound = audio.entries.iter().find(|e| e.name == "Sound Volume").unwrap();
        assert_eq!(sound.kind, SettingsEntryType::List);
        assert_eq!(sound.value_description(), "73");
        let music = audio.entries.iter().find(|e| e.name == "Music Volume").unwrap();
        assert_eq!(music.value_description(), "41");
    }
}
