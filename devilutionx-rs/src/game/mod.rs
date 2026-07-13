//! Game logic module - based on DevilutionX game systems
//! Precision port of DevilutionX C++ source code

pub mod types;
pub mod core;
pub mod player;
pub mod monster;
pub mod monster_exact; // Adapter: re-exports + MonsterManager (consolidation shim)
pub mod monster_object_interaction; // Monster ⟷ door/blocking interaction
pub mod monster_sprites; // Monster CL2 sprite loading + caching for dungeon rendering
pub mod items;
pub mod spells;
pub mod world;
pub mod level;
pub mod dungeon;
pub mod combat;
pub mod pathfinding;
pub mod animation;
pub mod map_render;
pub mod save;
pub mod loadsave; // Full C++-aligned save/load: SaveLevel/LoadLevel/SaveGame/LoadGame binary I/O
pub mod quest;
pub mod quests;        // Quest system (M9 Day 88+)
pub mod inventory;     // Inventory System (M10 Day 93+)
pub mod input;         // Input system (M80)
pub mod playable_demo; // Playable demo (M80)

// C++ aligned data modules (Source/*.cpp)
pub mod spelldat;      // ✅ Source/spelldat.cpp
pub mod objdat;        // ✅ Source/objdat.cpp

// Exact data ports from C++ headers
pub mod data;          // Static game data tables (NEW!)
pub mod player_dat;
pub mod item_dat;
pub mod monstdat;      // ✅ Source/monstdat.cpp/h
pub mod combat_system;
pub mod player_new;    // Exact Player struct port
pub mod item_new;      // Exact Item struct port
pub mod level_new;     // Exact Level/Dungeon system port
pub mod item_affix;    // Exact item affix/prefix/suffix system port
pub mod quest_new;     // Exact quest system port
pub mod objects_new;   // Exact object system port
pub mod objects;       // Object system (Day 10-11)
pub mod player_exact;  // Exact Player system port (Day 19-25)
pub mod shrine_effects; // Shrine effects with Player integration (Day 26-30)
pub mod combat_integration; // Monster ⟷ Player combat (Day 36)
pub mod game_state; // Unified GameState manager (Day 38-39)
pub mod dungeon_level; // L1 Cathedral generation + dPiece layout (levels bridge)
pub mod missiles; // Missile/Projectile system (Day 41)
pub mod items_processing; // Items Processing system (Day 42-43)
pub mod player_movement; // Player Movement system (Day 44-45)
pub mod towner; // NPC/Towner system (M12 Day 89-91)
pub mod store; // Store/Shop system (M13 Day 92+)
pub mod dialogue; // Dialogue system (M14 Day 96+)
pub mod game_loop; // Complete C++ aligned game loop (M80)
pub mod hud; // In-game HUD panel (life/mana spheres, XP bar, belt) (M-HUD)
pub mod network;   // Network stubs (M80)
pub mod lighting; // Lighting system (M17)
pub mod automap; // Automap system (M20)
pub mod control; // Control panel system (M21)
pub mod cursor; // Cursor system (M22)
pub mod quest_log; // Quest log UI system (M23)
pub mod multi; // Multiplayer system (M24)
pub mod pfile; // Save file system (M25)
pub mod interfac; // Interface/load screen system (M26)
pub mod pack; // Data packing system (M27)
pub mod codec; // Encryption codec system (M28)
pub mod appfat; // Fatal error handling (M29)
pub mod capture; // Screenshot capture (M30)
pub mod plrmsg; // Player message system (M31)
pub mod help; // Help system (M32)
pub mod doom; // Map of Stars quest (M33)
pub mod dead; // Corpse system (M34)
pub mod portal; // Town portal system (M35)
pub mod tmsg; // Timed message system (M36)
pub mod track; // Mouse tracking system (M37)
pub mod gamemenu; // Game menu system (M38)
pub mod sync; // Multiplayer sync system (M39)
pub mod nthread; // Network thread system (M40)
pub mod effects; // Sound effects system (M41)
pub mod movie; // Video playback system (M42)
pub mod spells_cast; // Spell casting system (M43)
pub mod vision; // Vision/line-of-sight system (M43)
pub mod crawl; // Crawl traversal algorithm (M44)
pub mod inv; // Inventory system (player backpack/equipment/belt)
pub mod inv_exact; // Exact inventory system (M44)
pub mod sha; // X-SHA-1 hash algorithm (M45)
pub mod minitext; // Scrolling dialog text system (M46)
pub mod diablo_msg; // In-game message system (M47)
pub mod restrict; // File system restriction testing (M48)
pub mod quick_messages; // Quick chat messages (M48)
pub mod game_mode_state; // Game mode state (M48)
pub mod random; // Random number generation (M49)
pub mod diablo_main; // Diablo main program core (M50)
pub mod scrollrt; // Scroll/render types and isometric projection (M51)
pub mod init; // Game initialization system (M53)
pub mod gmenu; // Game menu system (M53)
pub mod msg; // Network message system (M54)
pub mod menu; // Main menu system (M56)

pub use types::*;
pub use core::{Game, GameMode, GameConfig, Camera, InputState, isometric};
pub use player::{Player, PlayerClass, PlayerMode, PlayerAction};
pub use monster::{Monster, MonsterType, MonsterSpawner, MonsterAIState};
pub use items::{Item, ItemType, ItemQuality, GroundItem, ItemGenerator, Inventory};
pub use spells::{Spell, SpellType, Spellbook, SpellProjectile};
pub use world::World;
pub use level::Level;
pub use dungeon::{DungeonMap, TileType, Room};
pub use combat::{Combat, AttackResult, DamageType, DamageNumber};
pub use pathfinding::{Pathfinder, smooth_path};
pub use animation::{
    AnimationState, Animation, AnimFrame, AnimationController,
    SpriteSheet, DiabloAnimations, HitBox,
    Particle, ParticleType, ParticleEmitter, EffectManager,
};
pub use map_render::{
    MapRenderer, MapTile, MapLayer, AnimatedTile, AnimTileType,
    LightSource, FogState, RenderTile,
    WeatherSystem, WeatherType, WeatherParticle,
};
pub use save::{SaveManager, GameSave, SaveHeader, SavedPlayer, SavedInventory};
pub use quest::{QuestId, QuestState, QuestProgress, QuestData, QuestManager, QuestReward};
pub mod save_decode;
