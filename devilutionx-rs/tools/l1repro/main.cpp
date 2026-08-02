// Standalone C++ L1 (Cathedral) generator repro.
// Includes Source/levels/drlg_l1.cpp directly so the anonymous-namespace
// GenerateLevel is reachable, mirrors the gendung.cpp helpers it needs, and
// dumps the authoritative C++ 40x40 dungeon[][] grid for any seed.
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <memory>
#include <optional>
#include <string>
#include <vector>

#include "engine/random.hpp"
#include "levels/drlg_l1.h"
#include "multi.h"
#include "levels/gendung.h"
#include "quests.h"

#define NUMLEVELS 24

namespace devilution {

// ---- globals (mirror gendung.cpp) ----
Bitset2d<DMAXX, DMAXY> DungeonMask;
uint8_t dungeon[DMAXX][DMAXY];
uint8_t pdungeon[DMAXX][DMAXY];
Bitset2d<DMAXX, DMAXY> Protected;
Bitset2d<DMAXX, DMAXY> Chamber;
struct Player {
	bool pOriginalCathedral = false;
};
Player *MyPlayer = nullptr;
struct CornerStoneStruct {
	Point position;
	bool activated = false;
	bool isAvailable() const { return false; }
};
CornerStoneStruct CornerStone;
void AddL1Objs(int, int, int, int) {}
void AddCryptObjects(int, int, int, int) {}
void DRLG_CheckQuests(WorldTilePosition) {}
bool UseMultiplayerQuests() { return false; }
int UberRow = 0;
bool g_dumpStages = false;
int UberCol = 0;
const Miniset L5STAIRSUP {
	{ 4, 4 },
	{
	    { 22, 22, 22, 22 },
	    { 2, 2, 2, 2 },
	    { 13, 13, 13, 13 },
	    { 13, 13, 13, 13 },
	},
	{
	    { 0, 66, 23, 0 },
	    { 63, 64, 65, 0 },
	    { 0, 67, 68, 0 },
	    { 0, 0, 0, 0 },
	}
};
void DRLG_MRectTrans(RectangleOf<unsigned char, unsigned char>) {}
bool PlaceCryptStairs(lvl_entry) { return true; }
void SetCryptRoom() {}
void CryptSubstitution() {}
void FixCryptDirtTiles() {}
void DRLG_LPass3(int) {}
void InitCryptPieces() {}
void PlaceCryptLights() {}
void SetCryptSetPieceRoom() {}
template <typename T> std::unique_ptr<T[]> LoadFileInMem(const char *) { return nullptr; }
std::expected<void, std::string> LoadDungeonBase(const char *, Point, int, int) { return std::unexpected(std::string("stub")); }

WorldTileRectangle SetPieceRoom;
WorldTileRectangle SetPiece;
OptionalOwnedClxSpriteList pSpecialCels;
std::unique_ptr<MegaTile[]> pMegaTiles;
std::unique_ptr<std::byte[]> pDungeonCels;
TileProperties SOLData[MAXTILES];
WorldTilePosition dminPosition;
WorldTilePosition dmaxPosition;
dungeon_type leveltype = DTYPE_TOWN;
uint8_t currlevel = 0;
bool setlevel = false;
_setlevels setlvlnum = SL_NONE;
dungeon_type setlvltype = DTYPE_TOWN;
Point ViewPosition;
uint_fast8_t MicroTileLen = 0;
int8_t TransVal = 0;
std::array<bool, 256> TransList {};
uint16_t dPiece[MAXDUNX][MAXDUNY] {};
MICROS DPieceMicros[MAXTILES] {};
int8_t dTransVal[MAXDUNX][MAXDUNY] {};
uint8_t dLight[MAXDUNX][MAXDUNY] {};
uint8_t dPreLight[MAXDUNX][MAXDUNY] {};
DungeonFlag dFlags[MAXDUNX][MAXDUNY] {};
int8_t dSpecial[MAXDUNX][MAXDUNY] {};
THEME_LOC themeLoc[MAXTHEMES] {};
int themeCount = 0;

std::optional<uint32_t> LevelSeeds[NUMLEVELS];
Quest Quests[MAXQUESTS];
const QuestData QuestsData[MAXQUESTS] {};

// ---- helpers copied from gendung.cpp ----

uint16_t Swap16LE(uint16_t v) { return v; }

WorldTileSize GetDunSize(const uint16_t *dunData)
{
	return WorldTileSize(static_cast<WorldTileCoord>(Swap16LE(dunData[0])), static_cast<WorldTileCoord>(Swap16LE(dunData[1])));
}

void PlaceDunTiles(const uint16_t *dunData, Point position, int floorId)
{
	const WorldTileSize size = GetDunSize(dunData);
	const uint16_t *tileLayer = &dunData[2];
	const uint16_t *mapLayer = &tileLayer[size.width * size.height];
	for (int j = 0; j < size.height; j++) {
		for (int i = 0; i < size.width; i++) {
			const uint16_t tile = Swap16LE(tileLayer[j * size.width + i]);
			dungeon[position.x + i][position.y + j] = static_cast<uint8_t>(tile == 0 ? floorId : tile);
		}
	}
}

void DRLG_InitTrans()
{
	Protected.reset();
	Chamber.reset();
	memset(dTransVal, 0, sizeof(dTransVal));
	TransVal = 0;
	TransList = {};
	MicroTileLen = 0;
}

void DRLG_CopyTrans(int sx, int sy, int dx, int dy)
{
	dTransVal[dx][dy] = dTransVal[sx][sy];
	TransList[dTransVal[sx][sy]] = true;
	if (dTransVal[sx][sy] > TransVal)
		TransVal = dTransVal[sx][sy];
}

void FloodTransparencyValues(uint8_t floorID)
{
	for (int j = 0; j < DMAXY; j++) {
		for (int i = 0; i < DMAXX; i++) {
			if (dungeon[i][j] == floorID)
				dTransVal[i][j] = 1;
		}
	}
}

std::optional<Point> PlaceMiniSet(const Miniset &miniset, int tries, bool drlg1Quirk)
{
	const int sw = miniset.size.width;
	const int sh = miniset.size.height;
	Point position { GenerateRnd(DMAXX - sw), GenerateRnd(DMAXY - sh) };
	for (int i = 0; i < tries; i++, position.x++) {
		if (position.x == DMAXX - sw) {
			position.x = 0;
			position.y++;
			if (position.y == DMAXY - sh) {
				position.y = 0;
			}
		}
		if (drlg1Quirk) {
			bool valid = true;
			if (position.x <= 12) {
				position.x++;
				valid = false;
			}
			if (position.y <= 12) {
				position.y++;
				valid = false;
			}
			if (!valid)
				continue;
		}
		if (g_dumpStages) printf("DEBUG pm miniset sw=%d sh=%d search0={%d,%d,%d,%d} search1={%d,%d,%d,%d}\n", sw, sh, miniset.search[0][0], miniset.search[0][1], miniset.search[0][2], miniset.search[0][3], miniset.search[1][0], miniset.search[1][1], miniset.search[1][2], miniset.search[1][3]);
		if (i < 12)
			if (g_dumpStages) printf("DEBUG pm i=%d x=%d y=%d tile=%d\n", i, position.x, position.y, dungeon[position.x][position.y]);
		if (SetPieceRoom.contains(position))
			continue;
		if (!miniset.matches(position))
			continue;
		if (g_dumpStages) printf("DEBUG pm SUCCESS x=%d y=%d\n", position.x, position.y);
		miniset.place(position);
		return position;
	}
	return {};
}

// ---- Quest stubs ----
bool Quest::IsAvailable() const
{
	if (currlevel != _qlevel)
		return false;
	if (_qactive == QUEST_NOTAVAIL)
		return false;
	return true;
}

// ---- pull in the real drlg_l1.cpp (same TU -> anonymous namespace reachable) ----
} // namespace devilution

#include "levels/drlg_l1.cpp"

int main(int argc, char **argv)
{
	using namespace devilution;
	uint32_t seed = 2588;
	if (argc > 1)
		seed = (uint32_t)std::strtoul(argv[1], nullptr, 10);

	for (auto &o : LevelSeeds)
		o = std::nullopt;
	currlevel = 1;
	leveltype = DTYPE_CATHEDRAL;
	for (auto &q : Quests) {
		q._qactive = QUEST_NOTAVAIL;
		q._qlevel = 0;
		q._qidx = Q_BUTCHER;
	}
	SetPieceRoom = WorldTileRectangle(WorldTilePosition { 0, 0 }, WorldTileSize { 0, 0 });
	SetPiece = WorldTileRectangle(WorldTilePosition { 0, 0 }, WorldTileSize { 0, 0 });

	Player dummy;
	dummy.pOriginalCathedral = true;
	MyPlayer = &dummy;

	SetRndSeed(seed);
	// Instrumented step-by-step run (mirrors GenerateLevel, dumping each stage).
	const char *dumpStages = std::getenv("L1_DUMP_STAGES");
	g_dumpStages = dumpStages != nullptr;
	SetRndSeed(seed);
	while (true) {
		DRLG_InitTrans();
		do {
			LevelSeeds[currlevel] = GetLCGEngineState();
			FirstRoom();
		} while (FindArea() < 533);
		InitDungeonFlags();
		MakeDmt();
		FillChambers();
		FixTilesPatterns();
		AddWall();
		if (dumpStages) { printf("\nSTAGE area_ok\n"); for (int y=0;y<DMAXY;y++){for(int x=0;x<DMAXX;x++)printf("%d%c",dungeon[x][y],x==DMAXX-1?'\n':' ');} }
		if (g_dumpStages) printf("RNGSTATE %u\n", GetLCGEngineState());
		FloodTransparencyValues(13);
		if (PlaceStairs(ENTRY_MAIN))
			break;
	}
	FixTransparency();
	FixDirtTiles();
	if (dumpStages) { printf("\nSTAGE fixdirt\n"); for (int y=0;y<DMAXY;y++){for(int x=0;x<DMAXX;x++)printf("%d%c",dungeon[x][y],x==DMAXX-1?'\n':' ');} }
	FixCornerTiles();
	Substitution();
	ApplyShadowsPatterns();
	const int numt = GenerateRnd(5) + 5;
	for (int i = 0; i < numt; i++)
		PlaceMiniSet(LAMPS, DMAXX * DMAXY, true);
	FillFloor();
	if (dumpStages) {
		printf("\nSTAGE fillfloor\n");
		for (int y = 0; y < DMAXY; y++) { for (int x = 0; x < DMAXX; x++) printf("%d%c", dungeon[x][y], x == DMAXX - 1 ? '\n' : ' '); }
	} else {
		for (int y = 0; y < DMAXY; y++) { for (int x = 0; x < DMAXX; x++) printf("%d%c", dungeon[x][y], x == DMAXX - 1 ? '\n' : ' '); }
	}
	return 0;
}