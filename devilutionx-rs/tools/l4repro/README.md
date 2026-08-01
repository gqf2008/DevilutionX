# l4repro — Standalone C++ L4 (Hell) generator repro

Compiles the **current HEAD** `Source/levels/drlg_l4.cpp` (plus the Borland LCG
from `Source/engine/random.cpp`) against stub headers so we can dump the
authoritative C++ 40x40 `dungeon[][]` grid for any Hell seed and compare it
cell-for-cell with the Rust port (`devilutionx-rs/src/levels/drlg_l4.rs`).

## Build

From the repository root:

```sh
g++ -std=c++20 -O2 -I devilutionx-rs/tools/l4repro/stubs \
    -I devilutionx-rs/tools/l3repro/stubs -I devilutionx-rs/tools/l2repro/stubs \
    -I Source -I 3rdParty/tl \
    devilutionx-rs/tools/l4repro/main.cpp devilutionx-rs/tools/l4repro/drlg_l4.cpp \
    Source/engine/random.cpp -o devilutionx-rs/tools/l4repro/l4repro.exe
```

## Usage

```
l4repro.exe <seed> <level> <warlord> <diablo> <betrayer> <multiplayer>
```

Prints one row per line (`x` varying fastest) — the same orientation as the
`.dun` fixture tiles. `warlord=1` enables the Q_WARLORD quest room on level 13;
`diablo=1` / `betrayer=1` / `multiplayer=1` set the corresponding level-15
gate / Betrayer set-piece states.

Example:

```sh
l4repro.exe 717625719 14 0 0 0 0 > grid.txt
```

## Stubs

`main.cpp` re-implements the `gendung.cpp` helpers used by `drlg_l4.cpp`
(`PlaceMiniSet`, `DRLG_PlaceThemeRooms`, `GetSizeForThemeRoom`,
`IsNearThemeRoom`, `CreateThemeRoom`, `PlaceDunTiles`, transparency no-ops,
...) as faithful copies of the current C++. `LoadFileInMem<uint16_t>` resolves
`levels\\l3data\\` / `levels\\l4data\\` to `test/fixtures/levels/...`
so quest set pieces (warlord.dun, diab*.dun) load from the repo fixtures.

> Note: `CreateThemeRoom` here must include the `DTYPE_HELL` frame branch
> (`2/1/6` walls + `9/16/15/12` corners) exactly like `gendung.cpp` — an
> earlier copy omitted it, which silently corrupted every layout that placed a
> theme room.

## Verified alignment (2026-08-02)

Rust vs repro: 0/1600 for all current fixtures — 13-428074402,
13-594689775 (warlord=1), 14-717625719, 14-815743776, 15-1256511996,
15-1583642716, 16-741281013. The repo fixtures `13-594689775.dun` and
`15-1256511996.dun` are stale (2022 export / old warlord.dun asset) and do not
match current HEAD C++; the repro is the authoritative current-C++ baseline.
