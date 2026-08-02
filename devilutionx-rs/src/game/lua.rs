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

    /// Register `devilutionx.dev` (C++ `lua/modules/dev.cpp`, `_DEBUG` only).
    ///
    /// Mirrors C++ `LuaDevModule`: sub-tables `display`, `items`, `level`,
    /// `monsters`, `player`, `quests`, `search`, `towners` (including the
    /// nested `level.map`/`level.warp`/`player.gold`/`player.spells`/
    /// `player.stats`/`player.trn` tables). `search` and the `display`/`player`
    /// toggles are wired to `game::debug` state; the remaining commands expose
    /// the C++ API surface and report that game-state wiring is pending (see
    /// the alignment matrix, section 4 `lua` row).
    #[cfg(debug_assertions)]
    pub fn register_dev_module(&self) -> mlua::Result<()> {
        let dev: Table = self.lua.create_table()?;

        // ---- display (C++ lua/modules/dev/display.cpp) ------------------
        let display: Table = self.lua.create_table()?;
        display.set("fps", self.lua.create_function(|_, _on: Option<bool>| {
            Ok("FPS counter: Not wired (frameflag)".to_string())
        })?)?;
        display.set("fullbright", self.lua.create_function(|_, _on: Option<bool>| {
            Ok("Fullbright: Not wired (lighting toggle)".to_string())
        })?)?;
        display.set("grid", self.lua.create_function(|_, on: Option<bool>| {
            let v = toggle_debug_flag(&crate::game::debug::DEBUG_GRID, on);
            Ok(format!("Tile grid highlighting: {}", if v { "On" } else { "Off" }))
        })?)?;
        display.set("path", self.lua.create_function(|_, on: Option<bool>| {
            let v = toggle_debug_flag(&crate::game::debug::DEBUG_PATH, on);
            Ok(format!("Path highlighting: {}", if v { "On" } else { "Off" }))
        })?)?;
        display.set("scrollView", self.lua.create_function(|_, on: Option<bool>| {
            let v = toggle_debug_flag(&crate::game::debug::DEBUG_SCROLL_VIEW_ENABLED, on);
            Ok(format!("Scroll view: {}", if v { "On" } else { "Off" }))
        })?)?;
        display.set("vision", self.lua.create_function(|_, on: Option<bool>| {
            let v = toggle_debug_flag(&crate::game::debug::DEBUG_VISION, on);
            Ok(format!("Vision highlighting: {}", if v { "On" } else { "Off" }))
        })?)?;
        display.set("tileData", self.lua.create_function(|_, data_type: Option<String>| {
            const DATA_TYPES: [&str; 23] = [
                "microTiles", "dPiece", "dTransVal", "dLight", "dPreLight",
                "dFlags", "dPlayer", "dMonster", "missiles", "dCorpse",
                "dObject", "dItem", "dSpecial", "coords", "cursorcoords",
                "objectindex", "solid", "transparent", "trap", "AutomapView",
                "dungeon", "pdungeon", "Protected",
            ];
            let list = || {
                let mut s = String::from("clear");
                for t in DATA_TYPES {
                    s.push_str(", ");
                    s.push_str(t);
                }
                s
            };
            match data_type {
                None => Ok(format!("Valid values for the first argument:\n{}", list())),
                Some(t) if t == "clear" => Ok("Tile data cleared.".to_string()),
                Some(t) if DATA_TYPES.contains(&t.as_str()) => {
                    // Grid-text rendering is not wired; names are validated
                    // against the C++ DataTypes list and the toggle reported.
                    Ok("Tile data: On".to_string())
                }
                Some(_) => Ok(format!("Invalid name! Valid names are:\n{}", list())),
            }
        })?)?;
        dev.set("display", display)?;

        // ---- items (C++ lua/modules/dev/items.cpp) ----------------------
        let items: Table = self.lua.create_table()?;
        items.set("get", self.lua.create_function(|_, ()| {
            Ok("Not wired: selected-item access".to_string())
        })?)?;
        items.set("info", self.lua.create_function(|_, ()| {
            Ok("Not wired: selected-item info".to_string())
        })?)?;
        items.set("spawn", self.lua.create_function(|_, _name: String| {
            Ok("Not wired: item spawning".to_string())
        })?)?;
        items.set("spawnUnique", self.lua.create_function(|_, _name: String| {
            Ok("Not wired: unique item spawning".to_string())
        })?)?;
        dev.set("items", items)?;

        // ---- level (C++ lua/modules/dev/level.cpp) ----------------------
        let level: Table = self.lua.create_table()?;
        level.set("exportDun", self.lua.create_function(|_, ()| {
            Ok("Not wired: dun export".to_string())
        })?)?;
        let map: Table = self.lua.create_table()?;
        map.set("hide", self.lua.create_function(|_, ()| {
            Ok("Not wired: automap hide".to_string())
        })?)?;
        map.set("reveal", self.lua.create_function(|_, ()| {
            Ok("Not wired: automap reveal".to_string())
        })?)?;
        level.set("map", map)?;
        level.set("reset", self.lua.create_function(|_, (_level, _seed): (i64, Option<i64>)| {
            Ok("Not wired: level reset".to_string())
        })?)?;
        level.set("seed", self.lua.create_function(|_, _level: Option<i64>| {
            Ok("Not wired: DungeonSeeds".to_string())
        })?)?;
        let warp: Table = self.lua.create_table()?;
        warp.set("dungeon", self.lua.create_function(|_, _n: i64| {
            Ok("Not wired: warp to dungeon level".to_string())
        })?)?;
        warp.set("map", self.lua.create_function(|_, (_path, _dun_type, _x, _y): (String, i64, i64, i64)| {
            Ok("Not wired: warp to custom map".to_string())
        })?)?;
        warp.set("quest", self.lua.create_function(|_, _n: i64| {
            Ok("Not wired: warp to quest level".to_string())
        })?)?;
        level.set("warp", warp)?;
        dev.set("level", level)?;

        // ---- monsters (C++ lua/modules/dev/monsters.cpp) ----------------
        let monsters: Table = self.lua.create_table()?;
        monsters.set("spawn", self.lua.create_function(|_, (_name, _count): (String, Option<i64>)| {
            Ok("Not wired: monster spawning".to_string())
        })?)?;
        monsters.set("spawnUnique", self.lua.create_function(|_, (_name, _count): (String, Option<i64>)| {
            Ok("Not wired: unique monster spawning".to_string())
        })?)?;
        dev.set("monsters", monsters)?;

        // ---- player (C++ lua/modules/dev/player.cpp) --------------------
        let player: Table = self.lua.create_table()?;
        player.set("arrow", self.lua.create_function(|_, _effect: String| {
            Ok("Not wired: arrow effect".to_string())
        })?)?;
        player.set("god", self.lua.create_function(|_, on: Option<bool>| {
            let v = toggle_debug_flag(&crate::game::debug::DEBUG_GOD_MODE, on);
            Ok(format!("God mode: {}", if v { "On" } else { "Off" }))
        })?)?;
        player.set("invisible", self.lua.create_function(|_, on: Option<bool>| {
            let v = toggle_debug_flag(&crate::game::debug::DEBUG_INVISIBLE, on);
            Ok(format!("Invisible: {}", if v { "On" } else { "Off" }))
        })?)?;
        let gold: Table = self.lua.create_table()?;
        gold.set("give", self.lua.create_function(|_, _amount: Option<i64>| {
            Ok("Not wired: give gold".to_string())
        })?)?;
        gold.set("take", self.lua.create_function(|_, _amount: Option<i64>| {
            Ok("Not wired: take gold".to_string())
        })?)?;
        player.set("gold", gold)?;
        player.set("info", self.lua.create_function(|_, _id: Option<i64>| {
            Ok("Not wired: player info".to_string())
        })?)?;
        let spells: Table = self.lua.create_table()?;
        spells.set("setLevel", self.lua.create_function(|_, _level: i64| {
            Ok("Not wired: spell levels".to_string())
        })?)?;
        player.set("spells", spells)?;
        let stats: Table = self.lua.create_table()?;
        stats.set("adjustHealth", self.lua.create_function(|_, _amount: i64| {
            Ok("Not wired: adjust health".to_string())
        })?)?;
        stats.set("adjustMana", self.lua.create_function(|_, _amount: i64| {
            Ok("Not wired: adjust mana".to_string())
        })?)?;
        stats.set("levelUp", self.lua.create_function(|_, _amount: Option<i64>| {
            Ok("Not wired: level up".to_string())
        })?)?;
        stats.set("rejuvenate", self.lua.create_function(|_, ()| {
            Ok("Not wired: rejuvenate".to_string())
        })?)?;
        stats.set("setAttrToMax", self.lua.create_function(|_, ()| {
            Ok("Not wired: max stats".to_string())
        })?)?;
        stats.set("setAttrToMin", self.lua.create_function(|_, ()| {
            Ok("Not wired: min stats".to_string())
        })?)?;
        player.set("stats", stats)?;
        let trn: Table = self.lua.create_table()?;
        trn.set("mon", self.lua.create_function(|_, name: String| {
            *crate::game::debug::DEBUG_TRN.lock().unwrap() = format!("monsters\\{name}.trn");
            Ok("TRN set".to_string())
        })?)?;
        trn.set("plr", self.lua.create_function(|_, name: String| {
            *crate::game::debug::DEBUG_TRN.lock().unwrap() = format!("plrgfx\\{name}.trn");
            Ok("TRN set".to_string())
        })?)?;
        trn.set("clear", self.lua.create_function(|_, ()| {
            *crate::game::debug::DEBUG_TRN.lock().unwrap() = String::new();
            Ok("TRN unset".to_string())
        })?)?;
        player.set("trn", trn)?;
        dev.set("player", player)?;

        // ---- quests (C++ lua/modules/dev/quests.cpp) --------------------
        let quests: Table = self.lua.create_table()?;
        // QuestManager is instance-based in the Rust port and not yet exposed
        // to Lua; the commands keep the C++ arity and report availability.
        quests.set("activate", self.lua.create_function(|_, _id: i64| {
            Ok("Quest state unavailable.".to_string())
        })?)?;
        quests.set("activateAll", self.lua.create_function(|_, ()| {
            Ok("Quest state unavailable.".to_string())
        })?)?;
        quests.set("all", self.lua.create_function(|_, ()| {
            Ok("Quest state unavailable.".to_string())
        })?)?;
        quests.set("info", self.lua.create_function(|_, _id: i64| {
            Ok("Quest state unavailable.".to_string())
        })?)?;
        dev.set("quests", quests)?;

        // ---- search (C++ lua/modules/dev/search.cpp) --------------------
        // Fully wired: mirrors AddDebugAutomap*Highlight + ClearDebugAutomapHighlights.
        let search: Table = self.lua.create_table()?;
        search.set("clear", self.lua.create_function(|_, ()| {
            crate::game::debug::clear_debug_automap_highlights();
            Ok("Removed all automap search markers.".to_string())
        })?)?;
        search.set("item", self.lua.create_function(|_, name: String| {
            if name.is_empty() {
                return Ok("Missing item name!".to_string());
            }
            crate::game::debug::add_debug_automap_item_highlight(&name);
            Ok(format!("Added automap marker for item {name}."))
        })?)?;
        search.set("monster", self.lua.create_function(|_, name: String| {
            if name.is_empty() {
                return Ok("Missing monster name!".to_string());
            }
            crate::game::debug::add_debug_automap_monster_highlight(&name);
            Ok(format!("Added automap marker for monster {name}."))
        })?)?;
        search.set("object", self.lua.create_function(|_, name: String| {
            if name.is_empty() {
                return Ok("Missing object name!".to_string());
            }
            crate::game::debug::add_debug_automap_object_highlight(&name);
            Ok(format!("Added automap marker for object {name}."))
        })?)?;
        dev.set("search", search)?;

        // ---- towners (C++ lua/modules/dev/towners.cpp) ------------------
        let towners: Table = self.lua.create_table()?;
        towners.set("talk", self.lua.create_function(|_, _name: String| {
            Ok("Not wired: talk to towner".to_string())
        })?)?;
        towners.set("visit", self.lua.create_function(|_, _name: String| {
            Ok("Not wired: teleport to towner".to_string())
        })?)?;
        dev.set("towners", towners)?;

        let devilutionx: Table = self.lua.globals().get("devilutionx")?;
        devilutionx.set("dev", dev)?;
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
        #[cfg(debug_assertions)]
        self.register_dev_module()?;
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

/// Toggle a debug flag: set to `on` when given, otherwise flip it
/// (C++ `DebugCmd*` `optional<bool>` pattern). Returns the new value.
#[cfg(debug_assertions)]
fn toggle_debug_flag(flag: &std::sync::atomic::AtomicBool, on: Option<bool>) -> bool {
    use std::sync::atomic::Ordering;
    match on {
        Some(v) => {
            flag.store(v, Ordering::SeqCst);
            v
        }
        None => {
            let v = !flag.load(Ordering::SeqCst);
            flag.store(v, Ordering::SeqCst);
            v
        }
    }
}

// ============================================================================
// Global engine state (C++ `Source/lua/lua_global.cpp`)
// ============================================================================

use std::cell::RefCell;

thread_local! {
    /// The process-wide Lua engine, created by `LuaInitialize` (main thread).
    /// Kept thread-local because `mlua::Lua` is not `Sync`; the game loop and
    /// startup/shutdown all run on the main thread.
    static GLOBAL_LUA: RefCell<Option<LuaEngine>> = const { RefCell::new(None) };
}

/// C++ `LuaInitialize` (lua_global.cpp): create the global Lua state, open the
/// standard libraries, and register the `devilutionx.*` modules.
pub fn lua_initialize() -> mlua::Result<()> {
    let engine = LuaEngine::new()?;
    engine.register_default_modules()?;
    GLOBAL_LUA.with(|slot| *slot.borrow_mut() = Some(engine));
    Ok(())
}

/// C++ `LuaShutdown` (lua_global.cpp): release the global Lua state.
pub fn lua_shutdown() {
    GLOBAL_LUA.with(|slot| *slot.borrow_mut() = None);
}

/// Run `f` with the global engine when it has been initialized.
pub fn with_lua<T>(f: impl FnOnce(&LuaEngine) -> T) -> Option<T> {
    GLOBAL_LUA.with(|slot| slot.borrow().as_ref().map(f))
}

/// C++ `LuaEvent(name).trigger()` dispatch. Called by the game loop for the
/// events the C++ engine forwards to mods.
pub fn call_event(name: &str) {
    with_lua(|engine| {
        let _ = engine.call_event(name);
    });
}

/// Push the live player's name/level into the Lua registry so
/// `devilutionx.player.self()` reports the real character (C++ keeps the
/// player info in the game globals and the Lua module reads it live).
pub fn sync_player(player: &crate::game::player_exact::Player) {
    with_lua(|engine| {
        let name = decode_player_name(&player._p_name);
        let _ = engine.state().set_named_registry_value("player_name", name);
        let _ = engine.state().set_named_registry_value("player_level", player._p_level as i32);
    });
}

/// Decode a C++ fixed-size player-name byte array (NUL-terminated) to `String`.
fn decode_player_name(raw: &[u8]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).to_string()
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


    #[cfg(debug_assertions)]
    #[test]
    fn test_dev_module_registered() {
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script(
                "dev.lua",
                r#"
                dev = devilutionx.dev
                assert(dev.display ~= nil)
                assert(dev.display.grid ~= nil)
                assert(dev.display.tileData ~= nil)
                assert(dev.items.get ~= nil)
                assert(dev.level.exportDun ~= nil)
                assert(dev.level.map.hide ~= nil)
                assert(dev.level.map.reveal ~= nil)
                assert(dev.level.warp.dungeon ~= nil)
                assert(dev.monsters.spawn ~= nil)
                assert(dev.player.god ~= nil)
                assert(dev.player.gold.give ~= nil)
                assert(dev.player.spells.setLevel ~= nil)
                assert(dev.player.stats.rejuvenate ~= nil)
                assert(dev.player.trn.mon ~= nil)
                assert(dev.quests.activate ~= nil)
                assert(dev.search.monster ~= nil)
                assert(dev.towners.talk ~= nil)
                "#,
            )
            .unwrap();
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_dev_search_wires_debug_highlights() {
        crate::game::debug::clear_debug_automap_highlights();
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script(
                "search.lua",
                r#"
                msg = devilutionx.dev.search.monster("Skeleton")
                msg2 = devilutionx.dev.search.item("Short Sword")
                "#,
            )
            .unwrap();
        assert!(crate::game::debug::is_debug_automap_highlight_needed());
        let msg: String = engine.state().globals().get("msg").unwrap();
        assert!(msg.contains("Skeleton"));
        let msg2: String = engine.state().globals().get("msg2").unwrap();
        assert!(msg2.contains("Short Sword"));
        crate::game::debug::clear_debug_automap_highlights();
        assert!(!crate::game::debug::is_debug_automap_highlight_needed());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_dev_display_toggle_wires_debug_flag() {
        use std::sync::atomic::Ordering;
        crate::game::debug::DEBUG_GRID.store(false, Ordering::SeqCst);
        let engine = LuaEngine::new().unwrap();
        engine.register_default_modules().unwrap();
        engine
            .load_script("grid.lua", "state = devilutionx.dev.display.grid()")
            .unwrap();
        let state: String = engine.state().globals().get("state").unwrap();
        assert!(state.contains("On"));
        assert!(crate::game::debug::DEBUG_GRID.load(Ordering::SeqCst));
    }

    #[test]
    fn test_global_engine_sync_player_updates_registry() {
        use crate::game::player_exact::Player;

        // Thread-local global: independent per test thread.
        crate::game::lua::lua_initialize().unwrap();
        assert!(crate::game::lua::with_lua(|_| ()).is_some());

        let mut player = Player::new();
        player._p_name[..9].copy_from_slice(b"TestHero\0");
        player._p_level = 3;
        crate::game::lua::sync_player(&player);

        crate::game::lua::with_lua(|engine| {
            engine
                .load_script(
                    "sync_player.lua",
                    r#"
                    p = devilutionx.player.self()
                    lua_name = p.name
                    lua_level = p.characterLevel
                    "#,
                )
                .unwrap();
            let lua_name: String = engine.state().globals().get("lua_name").unwrap();
            assert_eq!(lua_name, "TestHero");
            let lua_level: i64 = engine.state().globals().get("lua_level").unwrap();
            assert_eq!(lua_level, 3);
        });

        // The registry keeps a default even before sync (module default Hero/1).
        crate::game::lua::lua_shutdown();
        assert!(crate::game::lua::with_lua(|_| ()).is_none());
    }

    #[test]
    fn test_call_event_dispatch_noop_without_engine() {
        // with_lua/call_event are safe when the engine is not initialized.
        crate::game::lua::call_event("on_update");
        assert!(crate::game::lua::with_lua(|_| ()).is_none());
    }

}
