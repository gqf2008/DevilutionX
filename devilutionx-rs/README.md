# DevilutionX-RS

Rust port of [DevilutionX](https://github.com/diasurgical/devilutionX) - Diablo 1 engine reimplementation.

## 🎮 About

This is a work-in-progress Rust reimplementation of DevilutionX, aiming to bring the classic Diablo 1 experience to modern platforms with the benefits of Rust's memory safety and modern tooling.

## 🚀 Building

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- SDL2 libraries (bundled automatically via Cargo features)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

## 📁 Project Structure

```
devilutionx-rs/
├── src/
│   ├── main.rs          # Entry point
│   ├── engine/          # Rendering, audio, assets
│   ├── game/            # Game logic (player, monsters, items)
│   ├── ui/              # Menus and HUD
│   ├── data/            # MPQ and asset loading
│   └── net/             # Multiplayer (optional)
└── Cargo.toml
```

## 🎯 Features

- [x] SDL2 initialization
- [x] Basic window and rendering
- [x] Engine layer (100% complete)
- [x] Spell data system
- [x] Monster data system
- [x] Item data system (70%)
- [ ] MPQ file loading
- [ ] Asset management
- [ ] Player movement
- [ ] Combat system
- [ ] Multiplayer

## 📊 Progress

**Current Status**: 45,142 lines / 125,500 lines = **36.0%**

### Recent Milestones (2025)
- ✅ **M17: Lighting System** (933 lines, 28 tests) - Light/Vision system
- ✅ **M16: GameLoop** (819 lines, 21 tests) - Main game loop
- ✅ **M15: Missiles Extension** (1,269 lines, 28 tests) - Missile/Projectile system
- ✅ **M14: Dialogue System** (1,523 lines, 26 tests) - Quest/NPC dialogue
- ✅ **M13: Store/Shop System** (900 lines, 24 tests) - Trading system
- ✅ **M12: NPC/Towner System** (800 lines, 22 tests) - Town NPCs
- ✅ **M10: Inventory System** (1,200 lines, 30 tests) - Item management

See [PORTING/](PORTING/) directory for detailed porting documentation and progress tracking.

## 📋 Roadmap

### Phase 1: Foundation (Current)
- [x] Project structure
- [x] SDL2 integration
- [ ] Basic rendering loop
- [ ] Input handling

### Phase 2: Assets
- [ ] MPQ archive reader
- [ ] Image loading (CEL/PCX)
- [ ] Audio loading
- [ ] Font rendering

### Phase 3: Game Core
- [ ] World/level system
- [ ] Player character
- [ ] Monster AI
- [ ] Combat mechanics
- [ ] Item system

### Phase 4: Network
- [ ] Protocol implementation
- [ ] Client-server architecture
- [ ] State synchronization

## 📝 License

This project follows the original DevilutionX licensing.

## 🙏 Credits

- Original [DevilutionX](https://github.com/diasurgical/devilutionX) team
- [Diablo](https://en.wikipedia.org/wiki/Diablo_(video_game)) by Blizzard North

---

**Note**: This project requires the original Diablo 1 game files (DIABDAT.MPQ) to run.
