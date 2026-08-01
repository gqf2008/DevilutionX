# Repository Guidelines

DevilutionX is a C++20 port of Diablo/Hellfire, with a work-in-progress Rust reimplementation under `devilutionx-rs/`. These guidelines help contributors navigate the codebase, build, test, and submit changes.

## Project Structure & Module Organization

- `Source/` — main C++ source, organized into domain modules such as `engine/`, `levels/`, `items/`, `monsters/`, `quests/`, `DiabloUI/`, `platform/`, and `utils/`. Each module pairs `.cpp` implementations with `.h`/`.hpp` headers.
- `test/` — GoogleTest unit tests: one `*_test.cpp` per module, plus `fixtures/` with test assets.
- `docs/` — contribution and build guides (`CONTRIBUTING.md`, `building.md`, `CHANGELOG.md`).
- `assets/`, `Translations/`, `Packaging/`, `3rdParty/`, `CMake/`, `tools/` — game data, translations, packaging scripts, vendored dependencies, CMake helpers, and developer tools.

## Build, Test, and Development Commands

CMake is the build system; GoogleTest powers the test suite.

```bash
cmake -S . -B build                     # configure (BUILD_TESTING is ON by default)
cmake --build build -j                  # compile
cd build && ctest --output-on-failure   # run all tests
```

For coverage, configure with `-DENABLE_CODECOVERAGE=ON`; results are reported to Codecov. See `docs/building.md` for per-platform instructions.

## Coding Style & Naming Conventions

- Indent with tabs (tab width 4); format with `clang-format` using `Source/.clang-format` (WebKit-based). CI enforces this via the `clang-format-check` and `clang-tidy-check` workflows.
- Files use snake_case (`automap.cpp`, `player.h`); types and functions use PascalCase (`DrawMissile`); enums and macros use `UPPER_SNAKE_CASE`.
- Wrap code in `namespace devilution` and start each source file with a Doxygen `@file` block.
- Keep code compatible with the project's practical C++17 feature set even though CMake sets standard 20.
- Run `run-clang-tidy -p build 'Source.*'` locally for static analysis.

## Testing Guidelines

- Frameworks: GoogleTest/Google Mock; benchmarks use Google Benchmark (`*_benchmark.cpp`).
- Name test files `*_test.cpp` under `test/`; give tests descriptive names covering a single behavior.
- Keep tests deterministic and fast; tests needing real game assets should skip gracefully when the asset is absent.
- Run `cd build && ctest --output-on-failure` before opening a pull request.

## Commit & Pull Request Guidelines

- Use Conventional Commits with an imperative summary: `fix:`, `feat:`, `docs:`, `chore:`, `refactor:`, `build:` — e.g. `fix: handle dMissile[x][y] == 0 in DrawMissile`.
- Keep one logical change per commit; add a short body explaining the "why" when not obvious.
- PR descriptions should summarize the change, link the issue (`Fixes #123`), and include screenshots for UI or rendering changes. All applicable CI jobs (Linux, Windows, macOS, Android, consoles) must pass.
