# L3 (Caves) C++ reproduction harness

Ground-truth tool for the DevilutionX-RS <-> C++ L3 alignment workflow
(sibling of `tools/l2repro/`; shares its stub headers via `-I ../l2repro/stubs`).

Builds the current-HEAD C++ Caves generator standalone with g++:

    g++ -std=c++20 -O2 -I stubs -I ../l2repro/stubs -I ../../Source -I ../../../3rdParty/tl \
        main.cpp drlg_l3.cpp ../../../Source/engine/random.cpp \
        -o l3repro.exe

Run (from the repository root, so `test/fixtures/...` resolves):

    l3repro.exe <seed>

Output: the final 40x40 dungeon tile grid (current HEAD semantics).

Notes:
- `drlg_l3.cpp` is a copy of `Source/levels/drlg_l3.cpp` plus RNG/checksum
  checkpoints and `DVL_DUMP_AFTER_FENCE` / `DVL_DUMP_AFTER_THEMES` debug
  dumps (early-return modes for stage-by-stage comparison).
- `main.cpp` mirrors the gendung.cpp helpers (PlaceMiniSet,
  DRLG_PlaceThemeRooms, CreateThemeRoom with the CAVES/NEST branch,
  GetSizeForThemeRoom, IsNearThemeRoom, PlaceDunTiles) and stubs the heavy
  game dependencies (lighting/monster/objdat/objects).
- Verified: seed 262005438 == `test/fixtures/diablo/9-262005438.dun`
  1600/1600, and == the Rust port 1600/1600. The 9-262005438 fixture is
  current-valid (unlike the 2022 L1/L2-quest fixtures).
