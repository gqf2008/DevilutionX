#pragma once
#include <cstdint>
#include <cstring>
#include <optional>
#include "quests.h"
#define NUMLEVELS 24
namespace devilution {
extern std::optional<uint32_t> LevelSeeds[NUMLEVELS];
void DRLG_CheckQuests(WorldTilePosition setPieceRoom);
}
