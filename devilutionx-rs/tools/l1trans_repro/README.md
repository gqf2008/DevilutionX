# l1trans_repro — L1 transparency pipeline cross-check

Verifies that the Rust L1 transparency pipeline
(`flood_transparency_values` + `copy_stairs_transparency` +
`fix_transparency`) matches the current C++ algorithm cell-for-cell.

`main.cpp` copies the **exact bodies** of the C++ functions
(`gendung.cpp`: `IsFloor` / `FillTransparencyValues` /
`FindTransparencyValues` / `FloodTransparencyValues` / `DRLG_CopyTrans`;
`drlg_l1.cpp`: `FixTransparency` and the `EntranceStairs` trans-copy loop)
and runs them on a dumped 40x40 logical tile grid, printing the resulting
`dTransVal` (112x112, x fastest).

## Build

From the repository root:

```sh
g++ -std=c++20 -O2 devilutionx-rs/tools/l1trans_repro/main.cpp \
    -o devilutionx-rs/tools/l1trans_repro/l1trans_repro.exe
```

## Usage

```
l1trans_repro.exe <grid.txt>
```

`grid.txt` is 40 lines of 40 space-separated tile ids (x fastest) — the
logical `dungeon[][]` output of the L1 generator. Stdout is 112 lines of 112
`dTransVal` values.

## Verified alignment (2026-08-02)

For seeds 1 / 2588 / 743271966 / 999 / 424242 the Rust pipeline output is
identical to this harness (0/12544 cells differ), including the Dirt-wall
propagation added by `FixTransparency` and the `EntranceStairs` row copy.
