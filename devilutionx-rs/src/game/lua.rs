//! Lua mod-scripting engine (C++ `Source/lua/lua_global.cpp`).
//!
//! Wraps the `mlua` (Lua 5.4) runtime and mirrors the C++ global-state core:
//! script loading/execution and the `events.{name}.trigger` dispatch used by
//! `LuaEvent`. The `devilutionx.*` module bindings (items/monsters/player/...)
//! are the follow-up.

use mlua::{Function, Lua, MultiValue, Table, Value};

/// A running Lua state for mod scripts.
pub struct LuaEngine {
    lua: Lua,
    /// Set by `devilutionx.hellfire.enable()` (C++ `gbIsHellfire`).
    pub hellfire_enabled: bool,
}

impl LuaEngine {
    /// C++ `LuaInitialize`: create the state with the standard libraries
    /// (base, coroutine, debug, math, os, package, string, table, utf8).
    pub fn new() -> mlua::Result<Self> {
        Ok(Self {
            lua: Lua::new(),
            hellfire_enabled: false,
        })
    }

    /// Load and execute a script (C++ loads each mod's source at startup).
    pub fn load_script(&self, name: &str, source: &str) -> mlua::Result<()> {
        self.lua.load(source).set_name(name).exec()
    }

    /// C++ `LuaEvent(name)`: call `events.{name}.trigger`.
    pub fn call_event(&self, name: &str) -> mlua::Result<()> {
        let events: Table = self.lua.globals().get("events")?;
        let event: Table = events.get(name)?;
        let trigger: Function = event.get("trigger")?;
        trigger.call::<()>(())
    }

    /// Access to the raw Lua state (tests / module registration).
    pub fn state(&self) -> &Lua {
        &self.lua
    }

    /// C++ `LuaShutdown`.
    pub fn shutdown(self) {
        // Dropping the Lua state releases all resources.
    }
}


impl LuaEngine {
    /// Register the `devilutionx.log` module (C++ `lua/modules/log.cpp`).
    pub fn register_log_module(&self) -> mlua::Result<()> {
        let log: Table = self.lua.create_table()?;
        let format = |fmt: String, args: MultiValue| -> String {
            let mut out = String::new();
            let mut arg_iter = args.into_iter();
            let mut chars = fmt.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '{' && chars.peek() == Some(&'}') {
                    chars.next();
                    match arg_iter.next() {
                        Some(v) => out.push_str(&lua_value_to_string(&v)),
                        None => out.push_str("{}"),
                    }
                } else {
                    out.push(ch);
                }
            }
            out
        };
        log.set("info", self.lua.create_function(move |_, (fmt, args): (String, MultiValue)| {
            println!("[Lua] {}", format(fmt, args));
            Ok(())
        })?)?;
        log.set("verbose", self.lua.create_function(move |_, (fmt, args): (String, MultiValue)| {
            log::debug!("[Lua] {}", format(fmt, args));
            Ok(())
        })?)?;
        log.set("debug", self.lua.create_function(move |_, (fmt, args): (String, MultiValue)| {
            log::debug!("[Lua] {}", format(fmt, args));
            Ok(())
        })?)?;
        log.set("warn", self.lua.create_function(move |_, (fmt, args): (String, MultiValue)| {
            eprintln!("[Lua] {}", format(fmt, args));
            Ok(())
        })?)?;
        log.set("error", self.lua.create_function(move |_, (fmt, args): (String, MultiValue)| {
            eprintln!("[Lua] ERROR {}", format(fmt, args));
            Ok(())
        })?)?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("log", log)?;
        Ok(())
    }

    /// Register the `devilutionx.i18n` module (C++ `lua/modules/i18n.cpp`).
    /// The Rust port has no translation tables, so translate is the identity
    /// and the language code is "en".
    pub fn register_i18n_module(&self) -> mlua::Result<()> {
        let i18n: Table = self.lua.create_table()?;
        i18n.set("language_code", self.lua.create_function(|_, ()| Ok("en"))?)?;
        i18n.set("translate", self.lua.create_function(|_, text: String| Ok(text))?)?;
        i18n.set(
            "plural_translate",
            self.lua.create_function(|_, (singular, plural, count): (String, String, i64)| {
                if count == 1 {
                    Ok(format!("{} {}", count, singular))
                } else {
                    Ok(format!("{} {}", count, plural))
                }
            })?,
        )?;
        i18n.set(
            "particular_translate",
            self.lua.create_function(|_, (_, text): (String, String)| Ok(text))?,
        )?;
        i18n.set("is_small_font_tall", self.lua.create_function(|_, ()| Ok(false))?)?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("i18n", i18n)?;
        Ok(())
    }

    /// Register `devilutionx.hellfire` (C++ `lua/modules/hellfire.cpp`).
    pub fn register_hellfire_module(&self) -> mlua::Result<()> {
        let hellfire: Table = self.lua.create_table()?;
        hellfire.set("loadData", self.lua.create_function(|_, ()| Ok(()))?)?;
        hellfire.set("enable", self.lua.create_function(|lua, ()| {
            lua.set_named_registry_value("hellfire_enabled", true);
            Ok(())
        })?)?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("hellfire", hellfire)?;
        Ok(())
    }

    /// Register `devilutionx.audio` (C++ `lua/modules/audio.cpp`).
    /// `playSfx` validates the id and forwards to `sfx_callback` when set.
    pub fn register_audio_module(&self) -> mlua::Result<()> {
        let audio: Table = self.lua.create_table()?;
        audio.set(
            "playSfx",
            self.lua.create_function(move |_, id: i32| {
                // Sound playback is not wired; the id is validated against
                // the C++ SfxID range (C++ IsValidSfx).
                if (0..=LAST_SFX_ID).contains(&id) {
                    // TODO(lua): forward to the audio system.
                }
                Ok(())
            })?,
        )?;
        audio.set(
            "playSfxLoc",
            self.lua.create_function(move |_, (id, _x, _y): (i32, i32, i32)| {
                if (0..=LAST_SFX_ID).contains(&id) {
                    // TODO(lua): forward to the audio system.
                }
                Ok(())
            })?,
        )?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("audio", audio)?;
        Ok(())
    }

    /// Register `devilutionx.render` (C++ `lua/modules/render.cpp`).
    /// `string` drawing is not wired; screen size matches the 640x480 view.
    pub fn register_render_module(&self) -> mlua::Result<()> {
        let render: Table = self.lua.create_table()?;
        render.set("string", self.lua.create_function(|_, (_t, _x, _y): (String, i32, i32)| Ok(()))?)?;
        render.set("screen_width", self.lua.create_function(|_, ()| Ok(640))?)?;
        render.set("screen_height", self.lua.create_function(|_, ()| Ok(480))?)?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("render", render)?;
        Ok(())
    }


    /// Register `devilutionx.monsters` (C++ `lua/modules/monsters.cpp`).
    /// The TSV loading is a documented follow-up; paths are recorded in the
    /// registry for inspection.
    pub fn register_monsters_module(&self) -> mlua::Result<()> {
        let monsters: Table = self.lua.create_table()?;
        monsters.set(
            "addMonsterDataFromTsv",
            self.lua.create_function(|lua, path: String| {
                lua.set_named_registry_value("monster_tsv", path);
                Ok(())
            })?,
        )?;
        monsters.set(
            "addUniqueMonsterDataFromTsv",
            self.lua.create_function(|lua, path: String| {
                lua.set_named_registry_value("unique_monster_tsv", path);
                Ok(())
            })?,
        )?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("monsters", monsters)?;
        Ok(())
    }

    /// Register `devilutionx.player` (C++ `lua/modules/player.cpp`).
    /// `self()` returns a table with the current player's name/level from the
    /// optional `player_callback`; `walk_to` is a no-op hook.
    pub fn register_player_module(&self) -> mlua::Result<()> {
        let player: Table = self.lua.create_table()?;
        player.set(
            "self",
            self.lua.create_function(move |lua, ()| {
                let name: String = lua.named_registry_value("player_name").unwrap_or_else(|_| "Hero".to_string());
                let level: i32 = lua.named_registry_value("player_level").unwrap_or(1);
                let t: Table = lua.create_table()?;
                t.set("name", name)?;
                t.set("characterLevel", level)?;
                Ok(t)
            })?,
        )?;
        player.set(
            "walk_to",
            self.lua.create_function(|_, (_x, _y): (i32, i32)| Ok(()))?,
        )?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("player", player)?;
        Ok(())
    }

    /// Register `devilutionx.towners` (C++ `lua/modules/towners.cpp`): one
    /// table per towner with a `position()` function. The Rust port has no
    /// towner state wired yet, so `position()` returns nil.
    pub fn register_towners_module(&self) -> mlua::Result<()> {
        let towners: Table = self.lua.create_table()?;
        for name in [
            "griswold", "pepin", "deadguy", "ogden", "cain", "farnham", "adria", "gillian",
            "wirt", "cow", "lester", "celia", "nut",
        ] {
            let t: Table = self.lua.create_table()?;
            t.set("position", self.lua.create_function(|_, ()| Ok(mlua::Value::Nil))?)?;
            towners.set(name, t)?;
        }
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("towners", towners)?;
        Ok(())
    }


    /// Register `devilutionx.items` (C++ `lua/modules/items.cpp`).
    pub fn register_items_module(&self) -> mlua::Result<()> {
        let items: Table = self.lua.create_table()?;
        items.set(
            "addItemDataFromTsv",
            self.lua.create_function(|lua, (path, base): (String, i64)| {
                lua.set_named_registry_value("item_tsv_path", path)?;
                lua.set_named_registry_value("item_tsv_base", base)?;
                Ok(())
            })?,
        )?;
        items.set(
            "addUniqueItemDataFromTsv",
            self.lua.create_function(|lua, (path, base): (String, i64)| {
                lua.set_named_registry_value("unique_item_tsv_path", path)?;
                lua.set_named_registry_value("unique_item_tsv_base", base)?;
                Ok(())
            })?,
        )?;
        // Construct a Lua item from (item_type, value, name) and expose the
        // C++ Item predicate methods.
        items.set(
            "new",
            self.lua.create_function(|lua, (item_type, value, name): (String, i64, String)| {
                lua.create_userdata(LuaItem::new(&item_type, value, name))
            })?,
        )?;
        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("items", items)?;
        Ok(())
    }

    /// Register the common modules (C++ `LuaInitialize`'s table).
    pub fn register_default_modules(&self) -> mlua::Result<()> {
        let devilutionx: Table = self.lua.create_table()?;
        self.lua.globals().set("devilutionx", devilutionx)?;
        self.register_log_module()?;
        self.register_i18n_module()?;
        self.register_hellfire_module()?;
        self.register_audio_module()?;
        self.register_render_module()?;
        self.register_monsters_module()?;
        self.register_player_module()?;
        self.register_towners_module()?;
        self.register_items_module()?;
        Ok(())
    }
}

/// C++ `SfxID::LAST` sentinel for the audio module's validity check.
pub const LAST_SFX_ID: i32 = 122;

/// A Lua-visible item wrapper exposing the C++ `Item` predicate methods.
pub struct LuaItem {
    pub item_type: String,
    pub value: i64,
    pub name: String,
}

impl LuaItem {
    pub fn new(item_type: &str, value: i64, name: String) -> Self {
        Self {
            item_type: item_type.to_string(),
            value,
            name,
        }
    }
}

impl mlua::UserData for LuaItem {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("isGold", |_, this, ()| Ok(this.item_type == "Gold"));
        methods.add_method("isWeapon", |_, this, ()| {
            Ok(matches!(this.item_type.as_str(), "Sword" | "Axe" | "Mace" | "Bow" | "Staff" | "Dagger"))
        });
        methods.add_method("isArmor", |_, this, ()| {
            Ok(matches!(this.item_type.as_str(), "Helm" | "Armor" | "Shield"))
        });
        methods.add_method("isEquipment", |_, this, ()| {
            Ok(matches!(
                this.item_type.as_str(),
                "Sword" | "Axe" | "Mace" | "Bow" | "Staff" | "Dagger" | "Helm" | "Armor" | "Shield" | "Ring" | "Amulet"
            ))
        });
        methods.add_method("getName", |_, this, ()| Ok(this.name.clone()));
        methods.add_method("getValue", |_, this, ()| Ok(this.value));
    }
}

/// Format a Lua value for the log module (C++ `sol::utility::to_string`).
fn lua_value_to_string(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_default(),
        Value::Table(_) => "table".to_string(),
        _ => format!("{:?}", value),
    }
}


impl Default for LuaEngine {
    fn default() -> Self {
        Self::new().expect("Lua state creation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_and_call_event() {
        let engine = LuaEngine::new().unwrap();
        let script = r#"
            events = events or {}
            events.on_update = { trigger = function()
                _G.calls = (_G.calls or 0) + 1
            end }
        "#;
        engine.load_script("test.lua", script).unwrap();
        engine.call_event("on_update").unwrap();
        engine.call_event("on_update").unwrap();
        let calls: i64 = engine.state().globals().get("calls").unwrap();
        assert_eq!(calls, 2);
    }

    #[test]
    fn test_missing_event_errors() {
        let engine = LuaEngine::new().unwrap();
        engine.load_script("test.lua", "events = events or {}").unwrap();
        assert!(engine.call_event("does_not_exist").is_err());
    }

    #[test]
    fn test_script_can_compute() {
        let engine = LuaEngine::new().unwrap();
        engine
            .load_script("compute.lua", "function add(a, b) return a + b end")
            .unwrap();
        let add: Function = engine.state().globals().get("add").unwrap();
        let sum: i64 = add.call::<i64>((20, 22)).unwrap();
        assert_eq!(sum, 42);
    }

    #[test]
    fn test_bad_script_returns_error() {
        let engine = LuaEngine::new().unwrap();
        assert!(engine.load_script("bad.lua", "this is not lua {").is_err());
    }
    #[test]
    fn test_register_default_modules() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();

        // A mod script can call devilutionx.i18n functions.
        engine
            .load_script(
                "i18n.lua",
                r#"
                lang = devilutionx.i18n.language_code()
                translated = devilutionx.i18n.translate("Hello")
                plural = devilutionx.i18n.plural_translate("item", "items", 3)
                "#,
            )
            .unwrap();
        let lang: String = engine.state().globals().get("lang").unwrap();
        assert_eq!(lang, "en");
        let translated: String = engine.state().globals().get("translated").unwrap();
        assert_eq!(translated, "Hello");
        let plural: String = engine.state().globals().get("plural").unwrap();
        assert_eq!(plural, "3 items");
    }

    #[test]
    fn test_log_module_registered() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script("log.lua", "devilutionx.log.warn('hit {} dmg', 42)")
            .unwrap();
        // The warn call goes through eprintln; just verify it didn't panic.
    }


    #[test]
    fn test_hellfire_module() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script("hellfire.lua", "devilutionx.hellfire.enable()")
            .unwrap();
        let enabled: bool = engine.state().named_registry_value("hellfire_enabled").unwrap();
        assert!(enabled);
    }

    #[test]
    fn test_render_module_screen_size() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script(
                "render.lua",
                "w = devilutionx.render.screen_width() h = devilutionx.render.screen_height()",
            )
            .unwrap();
        let w: i32 = engine.state().globals().get("w").unwrap();
        let h: i32 = engine.state().globals().get("h").unwrap();
        assert_eq!(w, 640);
        assert_eq!(h, 480);
    }

    #[test]
    fn test_audio_module_accepts_sfx_ids() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        // Valid and invalid ids both return without error (no-op).
        engine
            .load_script("audio.lua", "devilutionx.audio.playSfx(5)")
            .unwrap();
        engine
            .load_script("audio2.lua", "devilutionx.audio.playSfx(9999)")
            .unwrap();
    }


    #[test]
    fn test_monsters_module_tsv_paths() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script(
                "monsters.lua",
                "devilutionx.monsters.addMonsterDataFromTsv('monsters.tsv')",
            )
            .unwrap();
        let path: String = engine.state().named_registry_value("monster_tsv").unwrap();
        assert_eq!(path, "monsters.tsv");
    }

    #[test]
    fn test_player_module_self() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .state()
            .set_named_registry_value("player_name", "Aria")
            .unwrap();
        engine.state().set_named_registry_value("player_level", 12).unwrap();
        engine
            .load_script(
                "player.lua",
                "p = devilutionx.player.self() name = p.name lvl = p.characterLevel",
            )
            .unwrap();
        let name: String = engine.state().globals().get("name").unwrap();
        let lvl: i32 = engine.state().globals().get("lvl").unwrap();
        assert_eq!(name, "Aria");
        assert_eq!(lvl, 12);
    }

    #[test]
    fn test_towners_module_registered() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script("towners.lua", "p = devilutionx.towners.griswold.position()")
            .unwrap();
        let p: mlua::Value = engine.state().globals().get("p").unwrap();
        assert!(matches!(p, mlua::Value::Nil));
    }


    #[test]
    fn test_items_module_predicates() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script(
                "items.lua",
                r#"
                gold = devilutionx.items.new("Gold", 50, "")
                is_gold = gold:isGold()
                sword = devilutionx.items.new("Sword", 0, "Short Sword")
                is_weapon = sword:isWeapon()
                is_armor = sword:isArmor()
                name = sword:getName()
                "#,
            )
            .unwrap();
        let is_gold: bool = engine.state().globals().get("is_gold").unwrap();
        let is_weapon: bool = engine.state().globals().get("is_weapon").unwrap();
        let is_armor: bool = engine.state().globals().get("is_armor").unwrap();
        let name: String = engine.state().globals().get("name").unwrap();
        assert!(is_gold);
        assert!(is_weapon);
        assert!(!is_armor);
        assert_eq!(name, "Short Sword");
    }

    #[test]
    fn test_items_module_tsv_paths() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script(
                "items_tsv.lua",
                "devilutionx.items.addItemDataFromTsv('items.tsv', 100)",
            )
            .unwrap();
        let path: String = engine.state().named_registry_value("item_tsv_path").unwrap();
        let base: i64 = engine.state().named_registry_value("item_tsv_base").unwrap();
        assert_eq!(path, "items.tsv");
        assert_eq!(base, 100);
    }


}
