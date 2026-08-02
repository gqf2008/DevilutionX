# l1repro — Standalone C++ L1 (Cathedral) generator repro

Compiles the **current HEAD** `Source/levels/drlg_l1.cpp` (plus the Borland LCG
from `Source/engine/random.cpp`) against stub headers so we can dump the
authoritative C++ 40x40 `dungeon[][]` grid for any Cathedral seed and compare
it cell-for-cell with the Rust port (`devilutionx-rs/src/levels/drlg_l1.rs`).

`drlg_l1.cpp` is `#include`d directly into `main.cpp` (same translation unit)
so the anonymous-namespace `GenerateLevel` is reachable.

## Build

From the repository root:

```sh
g++ -std=c++23 -O2 -I devilutionx-rs/tools/l1repro/stubs \
    -I devilutionx-rs/tools/l3repro/stubs -I devilutionx-rs/tools/l2repro/stubs \
    -I Source -I 3rdParty/tl \
    devilutionx-rs/tools/l1repro/main.cpp Source/engine/random.cpp \
    -o devilutionx-rs/tools/l1repro/l1repro.exe
```

## Usage

```
l1repro.exe <seed>
```

Prints the 40x40 grid one row per line (`x` varying fastest), the same
orientation as the `.dun` fixture tiles. All quests are NOTAVAIL (no set
piece), matching the quest-free fixture export.

Example:

```sh
l1repro.exe 743271966 > grid.txt
```

## Verified alignment (2026-08-02)

With `pOriginalCathedral = true` (the classic non-Hellfire default,
interfac.cpp:327), the real `L5STAIRSUP` miniset (crypt.cpp:19-33) and the
incremental `PlaceMiniSet` scan (gendung.cpp:646-683), the repro matches both
fixtures cell-for-cell: `1-743271966.dun` and `1-2588.dun` = **0/1600 diffs**.
The Rust `CathedralGenerator` (default `original_cathedral = true`) produces the
same grids, so the L1 generator is fully aligned with current C++ and the 2022
fixtures. Earlier "stale fixture" reports were artifacts of an incorrect
`PlaceMiniSet` stub in this tool.

## Debug stages

Set `L1_DUMP_STAGES=1` to print the grid after `area_ok` (rooms/chambers/walls),
`fixdirt` and `fillfloor` stages for step-by-step comparison with the Rust
`CathedralGenerator::debug_dump_stages`.
