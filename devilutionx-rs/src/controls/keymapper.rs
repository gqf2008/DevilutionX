//! Keymapper / Padmapper model (C++ `Source/options.h` KeymapperOptions /
//! PadmapperOptions + `Source/diablo.cpp` InitKeymapActions/InitPadmapActions).
//!
//! Ports the *model* behind the settings-menu KeyInput / PadInput screens: the
//! action tables (name, description, default binding) and the key-name map
//! used to render the currently bound key. Runtime dispatch (keymapper.cpp /
//! padmapper.cpp) and SDL event capture are a follow-up.
//!
//! Note on keycodes: upstream `Source/` was synced to SDL3, so the C++ stores
//! SDL3 keycodes. The Rust input layer still uses SDL2-style keycodes
//! (`controls::game_controls::KeyCode` / `ui::menu`), so this module keeps
//! default bindings *symbolically* via the display names (`key_id_to_name`),
//! which are identical in both SDL generations. Bridging to a concrete
//! keycode happens when the input layer moves to SDL3.

use std::collections::HashMap;

use super::controller::{ControllerButton, ControllerButtonCombo};

/// C++ `KeymapperMouseButtonMask` (options.h:701): mouse-button bindings are
/// stored with this high bit set.
pub const KEYMAPPER_MOUSE_BUTTON_MASK: u32 = 1 << 31;

/// C++ mouse-scroll bindings (options.h:702-705).
pub const MOUSE_SCROLL_UP: u32 = 65536 | KEYMAPPER_MOUSE_BUTTON_MASK;
pub const MOUSE_SCROLL_DOWN: u32 = 65537 | KEYMAPPER_MOUSE_BUTTON_MASK;
pub const MOUSE_SCROLL_LEFT: u32 = 65538 | KEYMAPPER_MOUSE_BUTTON_MASK;
pub const MOUSE_SCROLL_RIGHT: u32 = 65539 | KEYMAPPER_MOUSE_BUTTON_MASK;

/// A keymapper action (C++ `KeymapperOptions::Action`).
#[derive(Debug, Clone)]
pub struct KeyAction {
    /// INI key (e.g. `"QuickSave"`; `BeltItem{}` etc. are formatted with index).
    pub key: String,
    /// C++ `GetName()` (display name, translated at runtime in C++).
    pub name: String,
    /// C++ `GetDescription()`.
    pub description: String,
    /// Default binding as a key-name (`""` = unbound / `SDLK_UNKNOWN`).
    pub default_key_name: &'static str,
    /// Current binding as a key-name.
    pub bound_key_name: String,
    /// C++ `enable` gate (kept as a descriptive label; callbacks are follow-up).
    pub enabled_note: &'static str,
}

impl KeyAction {
    /// C++ `GetValueDescription()`: the name of the currently bound key.
    pub fn value_description(&self) -> &str {
        &self.bound_key_name
    }

    /// C++ `SetValue(int)`; here expressed through the key-name map so the
    /// model stays keycode-agnostic. Returns false for unknown keys.
    pub fn set_value(&mut self, key_name: &str, names: &HashMap<u32, String>) -> bool {
        if !key_name.is_empty() && !names.values().any(|n| n == key_name) {
            return false;
        }
        self.bound_key_name = key_name.to_string();
        true
    }
}

/// A padmapper action (C++ `PadmapperOptions::Action`).
#[derive(Debug, Clone)]
pub struct PadAction {
    /// INI key (e.g. `"PrimaryAction"`; `BeltItem{}` etc. are formatted).
    pub key: String,
    /// C++ `GetName()` (display name).
    pub name: String,
    /// C++ `GetDescription()`.
    pub description: String,
    /// Default binding (C++ `defaultInput`).
    pub default_input: ControllerButtonCombo,
    /// Current binding (C++ `boundInput`).
    pub bound_input: ControllerButtonCombo,
    /// C++ `enable` gate (kept as a descriptive label; callbacks are follow-up).
    pub enabled_note: &'static str,
}

impl PadAction {
    /// C++ `GetValueDescription()`: `"MOD + BUTTON"` / `"BUTTON"` / `""`.
    pub fn value_description(&self, short: bool) -> String {
        combo_description(self.bound_input, short)
    }

    /// C++ `SetValue(ControllerButtonCombo)`.
    pub fn set_value(&mut self, combo: ControllerButtonCombo) -> bool {
        self.bound_input = combo;
        true
    }
}

/// C++ `buttonToButtonName` (options.cpp Padmapper ctor): the display names
/// for controller buttons. `short` applies C++ `Shorten()` (first 3 UTF-8
/// code points).
pub fn button_name(button: ControllerButton, short: bool) -> String {
    use ControllerButton::*;
    let name = match button {
        None | AxisLeftStickUp | AxisLeftStickDown | AxisLeftStickLeft | AxisLeftStickRight
        | AxisRightStickUp | AxisRightStickDown | AxisRightStickLeft | AxisRightStickRight => "",
        AxisTriggerLeft | LeftTrigger => "LT",
        AxisTriggerRight | RightTrigger => "RT",
        A => "A",
        B => "B",
        X => "X",
        Y => "Y",
        LeftShoulder => "LB",
        RightShoulder => "RB",
        LeftStick => "LS",
        RightStick => "RS",
        Start => "Start",
        Back => "Select",
        Guide => "Guide",
        DPadUp => "Up",
        DPadDown => "Down",
        DPadLeft => "Left",
        DPadRight => "Right",
    };
    if short {
        name.chars().take(3).collect()
    } else {
        name.to_string()
    }
}

/// Format a combo like C++ `PadmapperOptions::Action::UpdateValueDescription()`
/// (options.cpp:1452-1469): `"modifier+button"` when a modifier is set,
/// `"button"` otherwise, `""` when unbound (`ControllerButton_NONE`).
pub fn combo_description(combo: ControllerButtonCombo, short: bool) -> String {
    if combo.button == ControllerButton::None {
        return String::new();
    }
    if combo.modifier == ControllerButton::None {
        return button_name(combo.button, short);
    }
    format!(
        "{}+{}",
        button_name(combo.modifier, short),
        button_name(combo.button, short)
    )
}

/// C++ `KeymapperOptions` ctor key map: keycode -> display name (options.cpp:
/// 1098-1169). Keycodes are the SDL2-style values used elsewhere in the Rust
/// input layer; the *names* are identical under SDL3.
pub fn key_id_to_name() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for c in b'A'..=b'Z' {
        map.insert(c as u32, String::from_utf8(vec![c]).unwrap());
    }
    for c in b'0'..=b'9' {
        map.insert(c as u32, String::from_utf8(vec![c]).unwrap());
    }
    // F1-F12 (SDL2 values; names match C++ keyIDToKeyName F1..F24).
    for i in 0..12 {
        map.insert(282 + i, format!("F{}", i + 1));
    }
    // Keypad (SDL2 values).
    map.insert(256, "KEYPADNUM 0".to_string());
    for i in 0..9 {
        map.insert(257 + i, format!("KEYPADNUM {}", i + 1));
    }
    map.insert(308, "LALT".to_string());
    map.insert(307, "RALT".to_string());
    map.insert(32, "SPACE".to_string());
    map.insert(305, "RCONTROL".to_string());
    map.insert(306, "LCONTROL".to_string());
    map.insert(316, "PRINT".to_string());
    map.insert(19, "PAUSE".to_string());
    map.insert(9, "TAB".to_string());
    map.insert(2 | KEYMAPPER_MOUSE_BUTTON_MASK, "MMOUSE".to_string());
    map.insert(4 | KEYMAPPER_MOUSE_BUTTON_MASK, "X1MOUSE".to_string());
    map.insert(5 | KEYMAPPER_MOUSE_BUTTON_MASK, "X2MOUSE".to_string());
    map.insert(MOUSE_SCROLL_UP, "SCROLLUPMOUSE".to_string());
    map.insert(MOUSE_SCROLL_DOWN, "SCROLLDOWNMOUSE".to_string());
    map.insert(MOUSE_SCROLL_LEFT, "SCROLLLEFTMOUSE".to_string());
    map.insert(MOUSE_SCROLL_RIGHT, "SCROLLRIGHTMOUSE".to_string());
    map.insert(96, "`".to_string());
    map.insert(91, "[".to_string());
    map.insert(93, "]".to_string());
    map.insert(92, "\\".to_string());
    map.insert(59, ";".to_string());
    map.insert(39, "'".to_string());
    map.insert(44, ",".to_string());
    map.insert(46, ".".to_string());
    map.insert(47, "/".to_string());
    map.insert(8, "BACKSPACE".to_string());
    map.insert(301, "CAPSLOCK".to_string());
    map.insert(302, "SCROLLLOCK".to_string());
    map.insert(277, "INSERT".to_string());
    map.insert(127, "DELETE".to_string());
    map.insert(278, "HOME".to_string());
    map.insert(279, "END".to_string());
    map.insert(267, "KEYPAD /".to_string());
    map.insert(268, "KEYPAD *".to_string());
    map.insert(271, "KEYPAD ENTER".to_string());
    map.insert(266, "KEYPAD DECIMAL".to_string());
    map
}

/// Translate an SDL keycode (SDL2-style, as used by the Rust input layer) to
/// the C++ keymapper key-name, or `None` when the key is not mappable.
/// Mirrors the settingsmenu KeyInput handler: letters are normalised to
/// upper-case (settingsmenu.cpp:487-489), everything else must be a known
/// special key / keypad key from the C++ map.
pub fn keycode_to_key_name(
    keycode: u32,
    names: &std::collections::HashMap<u32, String>,
) -> Option<String> {
    if (b'A' as u32..=b'Z' as u32).contains(&keycode) || (b'0' as u32..=b'9' as u32).contains(&keycode) {
        return Some((keycode as u8 as char).to_string());
    }
    names.get(&keycode).cloned()
}

/// Key-name lookup used by the settings UI (C++ `keyIDToKeyName`).
pub fn key_name_for(key: u32, names: &HashMap<u32, String>) -> &str {
    names.get(&key).map(String::as_str).unwrap_or("")
}

fn action(key: &str, name: &str, description: &str, default_key_name: &'static str) -> KeyAction {
    KeyAction {
        key: key.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        default_key_name,
        bound_key_name: default_key_name.to_string(),
        enabled_note: "",
    }
}

fn pad_action(
    key: &str,
    name: &str,
    description: &str,
    default_input: ControllerButtonCombo,
) -> PadAction {
    PadAction {
        key: key.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        default_input,
        bound_input: default_input,
        enabled_note: "",
    }
}

/// C++ `InitKeymapActions()` (diablo.cpp:1846-2158): the keymapper action
/// table. Dynamic-index entries (BeltItem / QuickSpell / QuickMessage) are
/// expanded 1..=N like the C++ `AddAction(..., index)` loop.
pub fn default_key_actions() -> Vec<KeyAction> {
    use ControllerButton as _;
    let mut actions = Vec::new();
    for i in 1..=8 {
        actions.push(action(
            &format!("BeltItem{i}"),
            &format!("Belt item {i}"),
            "Use Belt item.",
            "",
        ));
    }
    for i in 1..=8 {
        // C++: QuickSpell 1-4 default to F5-F8, 5-8 unbound.
        let default = if i <= 4 { Box::leak(format!("F{}", i + 4).into_boxed_str()) } else { "" };
        actions.push(action(
            &format!("QuickSpell{i}"),
            &format!("Quick spell {i}"),
            "Hotkey for skill or spell.",
            default,
        ));
    }
    actions.push(action(
        "QuickSpellPrevious",
        "Previous quick spell",
        "Selects the previous quick spell (cycles).",
        "SCROLLUPMOUSE",
    ));
    actions.push(action(
        "QuickSpellNext",
        "Next quick spell",
        "Selects the next quick spell (cycles).",
        "SCROLLDOWNMOUSE",
    ));
    actions.push(action("UseHealthPotion", "Use health potion", "Use health potions from belt.", ""));
    actions.push(action("UseManaPotion", "Use mana potion", "Use mana potions from belt.", ""));
    actions.push(action("DisplaySpells", "Speedbook", "Open Speedbook.", "S"));
    actions.push(action("QuickSave", "Quick save", "Saves the game.", "F2"));
    actions.push(action("QuickLoad", "Quick load", "Loads the game.", "F3"));
    actions.push(action("QuitGame", "Quit game", "Closes the game.", ""));
    actions.push(action("StopHero", "Stop hero", "Stops walking and cancel pending actions.", ""));
    actions.push(action("ItemHighlighting", "Item highlighting", "Show/hide items on ground.", "LALT"));
    actions.push(action("ToggleItemHighlighting", "Toggle item highlighting", "Permanent show/hide items on ground.", "RCONTROL"));
    actions.push(action("ToggleAutomap", "Toggle automap", "Toggles if automap is displayed.", "TAB"));
    actions.push(action("CycleAutomapType", "Cycle map type", "Opaque -> Transparent -> Minimap -> None", "M"));
    actions.push(action("Inventory", "Inventory", "Open Inventory screen.", "I"));
    actions.push(action("Character", "Character", "Open Character screen.", "C"));
    actions.push(action("Party", "Party", "Open side Party panel.", "Y"));
    actions.push(action("QuestLog", "Quest log", "Open Quest log.", "Q"));
    actions.push(action("SpellBook", "Spellbook", "Open Spellbook.", "B"));
    for i in 1..=8 {
        // C++: QuickMessage 1-4 default to F9-F12, 5-8 unbound.
        let default = if i <= 4 { Box::leak(format!("F{}", i + 8).into_boxed_str()) } else { "" };
        actions.push(action(
            &format!("QuickMessage{i}"),
            &format!("Quick Message {i}"),
            "Use Quick Message in chat.",
            default,
        ));
    }
    actions.push(action("HideInfoScreens", "Hide Info Screens", "Hide all info screens.", "SPACE"));
    actions.push(action("Zoom", "Zoom", "Zoom Game Screen.", "Z"));
    actions.push(action("PauseGame", "Pause Game", "Pauses the game.", "P"));
    actions.push(action("PauseGameAlternate", "Pause Game (Alternate)", "Pauses the game.", "PAUSE"));
    actions.push(action("DecreaseBrightness", "Decrease Brightness", "Reduce screen brightness.", "F"));
    actions.push(action("IncreaseBrightness", "Increase Brightness", "Increase screen brightness.", "G"));
    actions.push(action("Help", "Help", "Open Help Screen.", "F1"));
    actions.push(action("Screenshot", "Screenshot", "Takes a screenshot.", "PRINT"));
    actions.push(action("GameInfo", "Game info", "Displays game infos.", "V"));
    actions.push(action("ChatLog", "Chat Log", "Displays chat log.", "L"));
    actions.push(action("SortInv", "Sort Inventory", "Sorts the inventory.", "R"));
    actions
}

/// C++ `InitPadmapActions()` (diablo.cpp:2166-2478): the padmapper action
/// table.
pub fn default_pad_actions() -> Vec<PadAction> {
    use ControllerButton::*;
    let mut actions = Vec::new();
    for i in 1..=8 {
        actions.push(pad_action(
            &format!("BeltItem{i}"),
            &format!("Belt item {i}"),
            "Use Belt item.",
            ControllerButtonCombo::new(None),
        ));
    }
    for i in 1..=8 {
        actions.push(pad_action(
            &format!("QuickSpell{i}"),
            &format!("Quick spell {i}"),
            "Hotkey for skill or spell.",
            ControllerButtonCombo::new(None),
        ));
    }
    actions.push(pad_action("PrimaryAction", "Primary action", "Attack monsters, talk to towners, lift and place inventory items.", ControllerButtonCombo::new(B)));
    actions.push(pad_action("SecondaryAction", "Secondary action", "Open chests, interact with doors, pick up items.", ControllerButtonCombo::new(Y)));
    actions.push(pad_action("SpellAction", "Spell action", "Cast the active spell.", ControllerButtonCombo::new(X)));
    actions.push(pad_action("CancelAction", "Cancel action", "Close menus.", ControllerButtonCombo::new(A)));
    actions.push(pad_action("MoveUp", "Move up", "Moves the player character up.", ControllerButtonCombo::new(DPadUp)));
    actions.push(pad_action("MoveDown", "Move down", "Moves the player character down.", ControllerButtonCombo::new(DPadDown)));
    actions.push(pad_action("MoveLeft", "Move left", "Moves the player character left.", ControllerButtonCombo::new(DPadLeft)));
    actions.push(pad_action("MoveRight", "Move right", "Moves the player character right.", ControllerButtonCombo::new(DPadRight)));
    actions.push(pad_action("StandGround", "Stand ground", "Hold to prevent the player from moving.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("ToggleStandGround", "Toggle stand ground", "Toggle whether the player moves.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("UseHealthPotion", "Use health potion", "Use health potions from belt.", ControllerButtonCombo::new(LeftShoulder)));
    actions.push(pad_action("UseManaPotion", "Use mana potion", "Use mana potions from belt.", ControllerButtonCombo::new(RightShoulder)));
    actions.push(pad_action("Character", "Character", "Open Character screen.", ControllerButtonCombo::new(AxisTriggerLeft)));
    actions.push(pad_action("Inventory", "Inventory", "Open Inventory screen.", ControllerButtonCombo::new(AxisTriggerRight)));
    actions.push(pad_action("QuestLog", "Quest log", "Open Quest log.", ControllerButtonCombo::with_modifier(AxisTriggerLeft, Back)));
    actions.push(pad_action("SpellBook", "Spellbook", "Open Spellbook.", ControllerButtonCombo::with_modifier(AxisTriggerRight, Back)));
    actions.push(pad_action("DisplaySpells", "Speedbook", "Open Speedbook.", ControllerButtonCombo::new(A)));
    actions.push(pad_action("ToggleAutomap", "Toggle automap", "Toggles if automap is displayed.", ControllerButtonCombo::new(LeftStick)));
    actions.push(pad_action("AutomapMoveUp", "Automap Move Up", "Moves the automap up when active.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("AutomapMoveDown", "Automap Move Down", "Moves the automap down when active.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("AutomapMoveLeft", "Automap Move Left", "Moves the automap left when active.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("AutomapMoveRight", "Automap Move Right", "Moves the automap right when active.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("MouseUp", "Move mouse up", "Simulates upward mouse movement.", ControllerButtonCombo::with_modifier(DPadUp, Back)));
    actions.push(pad_action("MouseDown", "Move mouse down", "Simulates downward mouse movement.", ControllerButtonCombo::with_modifier(DPadDown, Back)));
    actions.push(pad_action("MouseLeft", "Move mouse left", "Simulates leftward mouse movement.", ControllerButtonCombo::with_modifier(DPadLeft, Back)));
    actions.push(pad_action("MouseRight", "Move mouse right", "Simulates rightward mouse movement.", ControllerButtonCombo::with_modifier(DPadRight, Back)));
    actions.push(pad_action("LeftMouseClick1", "Left mouse click", "Simulates the left mouse button.", ControllerButtonCombo::new(RightStick)));
    actions.push(pad_action("LeftMouseClick2", "Left mouse click", "Simulates the left mouse button.", ControllerButtonCombo::with_modifier(LeftShoulder, Back)));
    actions.push(pad_action("RightMouseClick1", "Right mouse click", "Simulates the right mouse button.", ControllerButtonCombo::with_modifier(RightStick, Back)));
    actions.push(pad_action("RightMouseClick2", "Right mouse click", "Simulates the right mouse button.", ControllerButtonCombo::with_modifier(RightShoulder, Back)));
    actions.push(pad_action("PadHotspellMenu", "Gamepad hotspell menu", "Hold to set or use spell hotkeys.", ControllerButtonCombo::new(Back)));
    actions.push(pad_action("PadMenuNavigator", "Gamepad menu navigator", "Hold to access gamepad menu navigation.", ControllerButtonCombo::new(Start)));
    actions.push(pad_action("ToggleGameMenu1", "Toggle game menu", "Opens the game menu.", ControllerButtonCombo::with_modifier(Start, Back)));
    actions.push(pad_action("ToggleGameMenu2", "Toggle game menu", "Opens the game menu.", ControllerButtonCombo::with_modifier(Back, Start)));
    actions.push(pad_action("QuickSave", "Quick save", "Saves the game.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("QuickLoad", "Quick load", "Loads the game.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("ItemHighlighting", "Item highlighting", "Show/hide items on ground.", ControllerButtonCombo::new(None)));
    actions.push(pad_action("ToggleItemHighlighting", "Toggle item highlighting", "Permanent show/hide items on ground.", ControllerButtonCombo::new(None)));
    actions
}

/// C++ `KeymapperOptions::findAction(key)` — first action bound to the key.
pub fn find_key_action<'a>(actions: &'a [KeyAction], key_name: &str) -> Option<&'a KeyAction> {
    actions.iter().find(|a| a.bound_key_name == key_name)
}

/// C++ `PadmapperOptions::findAction(button, isModifierPressed)` — first
/// action whose bound combo contains the button (modifier-aware resolution is
/// a runtime follow-up; the model returns the first exact-button match).
pub fn find_pad_action<'a>(actions: &'a [PadAction], button: ControllerButton) -> Option<&'a PadAction> {
    actions.iter().find(|a| a.bound_input.button == button)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_action_table_matches_cpp_defaults() {
        let actions = default_key_actions();
        // C++ InitKeymapActions: 8 belt + 8 quick spells + 33 fixed actions
        // (through SortInv), excluding the _DEBUG-only Console/DebugToggle.
        assert_eq!(actions.len(), 8 + 8 + 37, "keymapper action count");
        let by_key: HashMap<&str, &KeyAction> = actions.iter().map(|a| (a.key.as_str(), a)).collect();
        // Spot-check the C++ default bindings.
        assert_eq!(by_key["BeltItem1"].bound_key_name, "");
        assert_eq!(by_key["QuickSpell1"].bound_key_name, "F5");
        assert_eq!(by_key["QuickSpell4"].bound_key_name, "F8");
        assert_eq!(by_key["QuickSpell5"].bound_key_name, "");
        assert_eq!(by_key["QuickSpellPrevious"].bound_key_name, "SCROLLUPMOUSE");
        assert_eq!(by_key["QuickSpellNext"].bound_key_name, "SCROLLDOWNMOUSE");
        assert_eq!(by_key["DisplaySpells"].bound_key_name, "S");
        assert_eq!(by_key["QuickSave"].bound_key_name, "F2");
        assert_eq!(by_key["QuickLoad"].bound_key_name, "F3");
        assert_eq!(by_key["ItemHighlighting"].bound_key_name, "LALT");
        assert_eq!(by_key["ToggleItemHighlighting"].bound_key_name, "RCONTROL");
        assert_eq!(by_key["ToggleAutomap"].bound_key_name, "TAB");
        assert_eq!(by_key["CycleAutomapType"].bound_key_name, "M");
        assert_eq!(by_key["Inventory"].bound_key_name, "I");
        assert_eq!(by_key["Character"].bound_key_name, "C");
        assert_eq!(by_key["Party"].bound_key_name, "Y");
        assert_eq!(by_key["QuestLog"].bound_key_name, "Q");
        assert_eq!(by_key["SpellBook"].bound_key_name, "B");
        assert_eq!(by_key["QuickMessage1"].bound_key_name, "F9");
        assert_eq!(by_key["QuickMessage4"].bound_key_name, "F12");
        assert_eq!(by_key["QuickMessage5"].bound_key_name, "");
        assert_eq!(by_key["HideInfoScreens"].bound_key_name, "SPACE");
        assert_eq!(by_key["Zoom"].bound_key_name, "Z");
        assert_eq!(by_key["PauseGame"].bound_key_name, "P");
        assert_eq!(by_key["PauseGameAlternate"].bound_key_name, "PAUSE");
        assert_eq!(by_key["DecreaseBrightness"].bound_key_name, "F");
        assert_eq!(by_key["IncreaseBrightness"].bound_key_name, "G");
        assert_eq!(by_key["Help"].bound_key_name, "F1");
        assert_eq!(by_key["Screenshot"].bound_key_name, "PRINT");
        assert_eq!(by_key["GameInfo"].bound_key_name, "V");
        assert_eq!(by_key["ChatLog"].bound_key_name, "L");
        assert_eq!(by_key["SortInv"].bound_key_name, "R");
    }

    #[test]
    fn pad_action_table_matches_cpp_defaults() {
        let actions = default_pad_actions();
        // C++ InitPadmapActions: 8 belt + 8 quick spells + fixed actions.
        assert_eq!(actions.len(), 8 + 8 + 38, "padmapper action count");
        let by_key: HashMap<&str, &PadAction> = actions.iter().map(|a| (a.key.as_str(), a)).collect();
        assert_eq!(by_key["PrimaryAction"].bound_input.button, ControllerButton::B);
        assert_eq!(by_key["SecondaryAction"].bound_input.button, ControllerButton::Y);
        assert_eq!(by_key["SpellAction"].bound_input.button, ControllerButton::X);
        assert_eq!(by_key["CancelAction"].bound_input.button, ControllerButton::A);
        assert_eq!(by_key["MoveUp"].bound_input.button, ControllerButton::DPadUp);
        assert_eq!(by_key["UseHealthPotion"].bound_input.button, ControllerButton::LeftShoulder);
        assert_eq!(by_key["UseManaPotion"].bound_input.button, ControllerButton::RightShoulder);
        assert_eq!(by_key["Character"].bound_input.button, ControllerButton::AxisTriggerLeft);
        assert_eq!(by_key["Inventory"].bound_input.button, ControllerButton::AxisTriggerRight);
        assert_eq!(by_key["QuestLog"].bound_input.button, ControllerButton::AxisTriggerLeft);
        assert_eq!(by_key["QuestLog"].bound_input.modifier, ControllerButton::Back);
        assert_eq!(by_key["SpellBook"].bound_input.button, ControllerButton::AxisTriggerRight);
        assert_eq!(by_key["SpellBook"].bound_input.modifier, ControllerButton::Back);
        assert_eq!(by_key["DisplaySpells"].bound_input.button, ControllerButton::A);
        assert_eq!(by_key["ToggleAutomap"].bound_input.button, ControllerButton::LeftStick);
        assert_eq!(by_key["MouseUp"].bound_input.button, ControllerButton::DPadUp);
        assert_eq!(by_key["MouseUp"].bound_input.modifier, ControllerButton::Back);
        assert_eq!(by_key["LeftMouseClick1"].bound_input.button, ControllerButton::RightStick);
        assert_eq!(by_key["LeftMouseClick2"].bound_input.modifier, ControllerButton::Back);
        assert_eq!(by_key["RightMouseClick1"].bound_input.button, ControllerButton::RightStick);
        assert_eq!(by_key["RightMouseClick2"].bound_input.modifier, ControllerButton::Back);
        assert_eq!(by_key["PadHotspellMenu"].bound_input.button, ControllerButton::Back);
        assert_eq!(by_key["PadMenuNavigator"].bound_input.button, ControllerButton::Start);
        // C++ {modifier, button}: ToggleGameMenu1 = {BACK, START}, 2 = {START, BACK}.
        assert_eq!(by_key["ToggleGameMenu1"].bound_input.button, ControllerButton::Start);
        assert_eq!(by_key["ToggleGameMenu1"].bound_input.modifier, ControllerButton::Back);
        assert_eq!(by_key["ToggleGameMenu2"].bound_input.button, ControllerButton::Back);
        assert_eq!(by_key["ToggleGameMenu2"].bound_input.modifier, ControllerButton::Start);
        assert_eq!(by_key["BeltItem1"].bound_input.button, ControllerButton::None);
        assert_eq!(by_key["QuickSpell1"].bound_input.button, ControllerButton::None);
    }

    #[test]
    fn key_name_map_covers_cpp_entries() {
        let names = key_id_to_name();
        for expected in [
            "A", "Z", "0", "9", "F1", "F12", "KEYPADNUM 0", "KEYPADNUM 9", "LALT", "RALT",
            "SPACE", "RCONTROL", "LCONTROL", "PRINT", "PAUSE", "TAB", "MMOUSE", "X1MOUSE",
            "X2MOUSE", "SCROLLUPMOUSE", "SCROLLDOWNMOUSE", "SCROLLLEFTMOUSE", "SCROLLRIGHTMOUSE",
            "`", "[", "]", "\\", ";", "'", ",", ".", "/", "BACKSPACE", "CAPSLOCK",
            "SCROLLLOCK", "INSERT", "DELETE", "HOME", "END", "KEYPAD /", "KEYPAD *",
            "KEYPAD ENTER", "KEYPAD DECIMAL",
        ] {
            assert!(names.values().any(|n| n == expected), "missing key name {expected:?}");
        }
    }

    #[test]
    fn combo_description_formats_like_cpp() {
        use ControllerButton::*;
        assert_eq!(combo_description(ControllerButtonCombo::new(None), false), "");
        assert_eq!(combo_description(ControllerButtonCombo::new(B), false), "B");
        assert_eq!(
            combo_description(ControllerButtonCombo::with_modifier(AxisTriggerLeft, Back), false),
            "Select+LT"
        );
        assert_eq!(
            combo_description(ControllerButtonCombo::with_modifier(Start, Back), false),
            "Select+Start"
        );
        // Short form truncates each name to 3 code points (C++ Shorten()).
        assert_eq!(
            combo_description(ControllerButtonCombo::with_modifier(Start, Back), true),
            "Sel+Sta"
        );
        assert_eq!(
            combo_description(ControllerButtonCombo::with_modifier(DPadUp, Back), false),
            "Select+Up"
        );
    }

    #[test]
    fn find_actions_by_binding() {
        let keys = default_key_actions();
        assert!(find_key_action(&keys, "I").is_some());
        assert!(find_key_action(&keys, "NOPE").is_none());
        let pads = default_pad_actions();
        assert_eq!(find_pad_action(&pads, ControllerButton::B).map(|a| a.key.as_str()), Some("PrimaryAction"));
        assert_eq!(find_pad_action(&pads, ControllerButton::Y).map(|a| a.key.as_str()), Some("SecondaryAction"));
    }
}
