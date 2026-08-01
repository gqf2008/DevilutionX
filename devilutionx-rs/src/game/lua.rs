//! Lua mod-scripting engine (C++ `Source/lua/lua_global.cpp`).
//!
//! Wraps the `mlua` (Lua 5.4) runtime and mirrors the C++ global-state core:
//! script loading/execution and the `events.{name}.trigger` dispatch used by
//! `LuaEvent`. The `devilutionx.*` module bindings (items/monsters/player/...)
//! are the follow-up.

use mlua::{Function, Lua, Table};

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
}
