# L2 (Catacombs) C++ reproduction harness

Ground-truth tool for the DevilutionX-RS <-> C++ L2 alignment workflow.

Builds the current-HEAD C++ Catacombs generator standalone with g++:

    g++ -std=c++20 -O2 -I stubs -I ../../Source -I ../../../3rdParty/tl \
        main.cpp drlg_l2_debug.cpp ../../../Source/engine/random.cpp \
        -o l2repro.exe

Run (from the repository root, so `test/fixtures/...` resolves):

    l2repro.exe <seed> <quest> [pd|th]

- `<quest>`: 1 = Q_BLOOD INIT (14x20 quest room + blood1.dun set piece),
  0 = Q_BLOOD NOTAVAIL.
- Default output: the final 40x40 dungeon tile grid (current HEAD semantics).
- `pd`: dump the predungeon (room/hall ASCII layout) instead.
- `th`: dump theme room locations.

Notes:
- `drlg_l2_debug.cpp` is `Source/levels/drlg_l2.cpp` plus debug hooks
  (RNG/checksum checkpoints, `DVL_DUMP_LOCKOUT`) and exported debug accessors.
- `main.cpp` mirrors the gendung.cpp helpers used by drlg_l2.cpp
  (PlaceMiniSet, DRLG_PlaceThemeRooms, CreateThemeRoom, GetSizeForThemeRoom,
  IsNearThemeRoom, PlaceDunTiles) and stubs the heavy game dependencies.
- `Swap16LE` is the identity on little-endian hosts (matching `SDL_SwapLE16`).
- Verified: seed 1677631846 (quest=0) == `test/fixtures/diablo/5-1677631846.dun`
  1600/1600; seed 68685319 (quest=1) == gold minus the 49 blood1 cells that
  the stale 2022 fixture lacks.
