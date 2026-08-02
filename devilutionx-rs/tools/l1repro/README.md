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

## Finding (2026-08-02)

The repo fixtures `1-2588.dun` / `1-743271966.dun` are **stale** relative to
current HEAD C++: the repro matches them only 453/1600 and 1234/1600, while the
Rust port matches `1-743271966.dun` 100% (it was tuned to the 2022 fixture
baseline, like the L4 fixtures noted in `l4repro/README.md`). The repro is the
authoritative current-C++ baseline; the Rust port still needs to be re-verified
against it once the fixture staleness is resolved.
