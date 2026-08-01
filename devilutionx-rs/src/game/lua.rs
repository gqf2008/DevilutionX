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
}

impl LuaEngine {
    /// C++ `LuaInitialize`: create the state with the standard libraries
    /// (base, coroutine, debug, math, os, package, string, table, utf8).
    pub fn new() -> mlua::Result<Self> {
        Ok(Self { lua: Lua::new() })
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

    /// Register the common modules (C++ `LuaInitialize`'s table).
    pub fn register_default_modules(&self) -> mlua::Result<()> {
        let devilutionx: Table = self.lua.create_table()?;
        self.lua.globals().set("devilutionx", devilutionx)?;
        self.register_log_module()?;
        self.register_i18n_module()?;
        Ok(())
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


}
