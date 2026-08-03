// objrepro: L1 object-placement repro. Extends l1repro with the real
// l1.til/l1.sol (extracted to E:/tmp) to build dPiece and verify the C++
// InitRndLocBigObj acceptance for specific candidates.
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
#include <stack>
#include <string>

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
	TransVal = 1;
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

bool IsFloor(Point p, uint8_t floorID)
{
	const int i = (p.x - 16) / 2;
	const int j = (p.y - 16) / 2;
	if (i < 0 || i >= DMAXX)
		return false;
	if (j < 0 || j >= DMAXY)
		return false;
	return dungeon[i][j] == floorID;
}

void FillTransparencyValues(Point floor, uint8_t floorID)
{
	const Direction allDirections[] = {
		Direction::North, Direction::South, Direction::East, Direction::West,
		Direction::NorthEast, Direction::NorthWest, Direction::SouthEast, Direction::SouthWest,
	};
	for (const Direction dir : allDirections) {
		const Point adjacent = floor + dir;
		if (!IsFloor(adjacent, floorID))
			dTransVal[adjacent.x][adjacent.y] = TransVal;
	}
	dTransVal[floor.x][floor.y] = TransVal;
}

void FindTransparencyValues(Point floor, uint8_t floorID)
{
	struct Seed {
		int scanStart;
		int scanEnd;
		int y;
		int dy;
	};
	std::stack<Seed, std::vector<Seed>> seedStack;
	seedStack.push({ floor.x, floor.x + 1, floor.y, 1 });

	const auto isInside = [floorID](int x, int y) {
		if (dTransVal[x][y] != 0)
			return false;
		return IsFloor({ x, y }, floorID);
	};

	const auto set = [floorID](int x, int y) {
		FillTransparencyValues({ x, y }, floorID);
	};

	const Displacement left = { -1, 0 };
	const Displacement right = { 1, 0 };
	const auto checkDiagonals = [&](Point p, Displacement direction) {
		const Point up = p + Displacement { 0, -1 };
		const Point upOver = up + direction;
		if (!isInside(up.x, up.y) && isInside(upOver.x, upOver.y))
			seedStack.push({ upOver.x, upOver.x + 1, upOver.y, -1 });
		const Point down = p + Displacement { 0, 1 };
		const Point downOver = down + direction;
		if (!isInside(down.x, down.y) && isInside(downOver.x, downOver.y))
			seedStack.push(Seed { downOver.x, downOver.x + 1, downOver.y, 1 });
	};

	while (!seedStack.empty()) {
		const auto [scanStart, scanEnd, y, dy] = seedStack.top();
		seedStack.pop();

		int scanLeft = scanStart;
		if (isInside(scanLeft, y)) {
			while (isInside(scanLeft - 1, y)) {
				set(scanLeft - 1, y);
				scanLeft--;
			}
			checkDiagonals({ scanLeft, y }, left);
		}
		if (scanLeft < scanStart)
			seedStack.push(Seed { scanLeft, scanStart - 1, y - dy, -dy });

		int scanRight = scanStart;
		while (scanRight < scanEnd) {
			while (isInside(scanRight, y)) {
				set(scanRight, y);
				scanRight++;
			}
			seedStack.push(Seed { scanLeft, scanRight - 1, y + dy, dy });
			if (scanRight - 1 > scanEnd)
				seedStack.push(Seed { scanEnd + 1, scanRight - 1, y - dy, -dy });
			if (scanLeft < scanRight)
				checkDiagonals({ scanRight - 1, y }, right);

			while (scanRight < scanEnd && !isInside(scanRight, y))
				scanRight++;
			scanLeft = scanRight;
			if (scanLeft < scanEnd)
				checkDiagonals({ scanLeft, y }, left);
		}
	}
}

void FloodTransparencyValues(uint8_t floorID)
{
	int yy = 16;
	for (int j = 0; j < DMAXY; j++) {
		int xx = 16;
		for (int i = 0; i < DMAXX; i++) {
			if (dungeon[i][j] == floorID && dTransVal[xx][yy] == 0) {
				FindTransparencyValues({ xx, yy }, floorID);
				TransVal++;
			}
			xx += 2;
		}
		yy += 2;
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
		if (getenv("DUMP_TRANSVAL") != nullptr) {
			printf("TRANSVAL %d\n", TransVal);
			for (int y = 0; y < MAXDUNY; y++) { for (int x = 0; x < MAXDUNX; x++) printf("%d%c", dTransVal[x][y], x == MAXDUNX - 1 ? '\n' : ' '); }
		}
		if (PlaceStairs(ENTRY_MAIN))
			break;
	}
	FixTransparency();
	if (getenv("DUMP_TRANSVAL") != nullptr) {
		printf("FIXEDTRANSVAL %d\n", TransVal);
		for (int y = 0; y < MAXDUNY; y++) { for (int x = 0; x < MAXDUNX; x++) printf("%d%c", dTransVal[x][y], x == MAXDUNX - 1 ? '\n' : ' '); }
	}
	FixDirtTiles();
	if (dumpStages) { printf("\nSTAGE fixdirt\n"); for (int y=0;y<DMAXY;y++){for(int x=0;x<DMAXX;x++)printf("%d%c",dungeon[x][y],x==DMAXX-1?'\n':' ');} }
	FixCornerTiles();
	Substitution();
	ApplyShadowsPatterns();
	const int numt = GenerateRnd(5) + 5;
	for (int i = 0; i < numt; i++)
		PlaceMiniSet(LAMPS, DMAXX * DMAXY, true);
	FillFloor();

	// ---- objrepro: build dPiece from the final grid + real l1.til ----
	uint16_t til[206][4];
	{
		FILE *f = fopen("E:/tmp/l1.til", "rb");
		if (!f) { printf("cannot open l1.til\n"); return 1; }
		size_t n = fread(til, 8, 206, f);
		fclose(f);
		if (n != 206) { printf("til read %zu\n", n); return 1; }
	}
	uint8_t sol[453];
	{
		FILE *f = fopen("E:/tmp/l1.sol", "rb");
		if (!f) { printf("cannot open l1.sol\n"); return 1; }
		size_t n = fread(sol, 1, 453, f);
		fclose(f);
		if (n != 453) { printf("sol read %zu\n", n); return 1; }
	}
	uint16_t dp[MAXDUNX][MAXDUNY] {};
	for (int j = 0; j < DMAXY; j++) {
		for (int i = 0; i < DMAXX; i++) {
			int tile = dungeon[i][j];
			if (tile <= 0 || tile > 206) continue;
			const uint16_t *m = til[tile - 1];
			int xx = 16 + 2 * i;
			int yy = 16 + 2 * j;
			dp[xx][yy] = m[0];
			dp[xx + 1][yy] = m[1];
			dp[xx][yy + 1] = m[2];
			dp[xx + 1][yy + 1] = m[3];
		}
	}
	auto inBounds = [](int x, int y) { return x >= 0 && x < MAXDUNX && y >= 0 && y < MAXDUNY; };
	auto rndLocOk = [&](int x, int y, int dObjX, int dObjY, int dObjMask) -> const char* {
		if (dObjMask) return nullptr; // caller handles
		return nullptr;
	};
	// Direct RndLocOk replica:
	auto rndOk = [&](int x, int y, const int *dObj, int playerX, int playerY) -> const char* {
		if (!inBounds(x, y)) return "OOB";
		if (dObj[y * MAXDUNX + x] != 0) return "dObject";
		if (x == playerX && y == playerY) return "dPlayer";
		int pn = dp[x][y];
		if ((sol[pn] & 1) != 0) return "Solid";
		if (pn > 125 && pn < 143) return "range126-142";
		return nullptr;
	};
	auto areaOk = [&](int x0, int y0, int w, int h, const int *dObj, int playerX, int playerY) -> const char* {
		for (int dy = 0; dy < h; dy++) {
			for (int dx = 0; dx < w; dx++) {
				const char *why = rndOk(x0 + dx, y0 + dy, dObj, playerX, playerY);
				if (why) return why;
			}
		}
		return nullptr;
	};
	// Verify dPiece at the reference door/light positions
	{
		struct { int x, y, expect; } checks[] = {
			{42,33,43},{41,36,45},{27,66,45},{49,80,45},{69,80,45},{30,81,43},{41,84,45},{25,86,45},
			{56,46,269},{48,48,269},{54,50,269},{82,50,269},{76,58,269},{46,72,269},{54,80,269}
		};
		for (auto &c : checks) {
			printf("[DoorCheck] (%d,%d) dPiece=%d expect=%d %s\n", c.x, c.y, dp[c.x][c.y], c.expect, dp[c.x][c.y] == c.expect ? "OK" : "MISMATCH");
		}
	}
	// Test candidate (55,69) with sarc0 at (48,71) and player at (77,46)
	int dObj[MAXDUNX * MAXDUNY] {};
	dObj[71 * MAXDUNX + 48] = 1; // sarc 0
	{
		const char *why = areaOk(54, 67, 3, 4, dObj, 77, 46);
		printf("IsAreaOk(55,69) with sarc0: %s\n", why ? why : "ACCEPT");
		// print each point
		for (int dy = 0; dy < 4; dy++) {
			for (int dx = 0; dx < 3; dx++) {
				int x = 54 + dx, y = 67 + dy;
				const char *w = rndOk(x, y, dObj, 77, 46);
				printf("  (%d,%d) dPiece=%d sol=%02x -> %s\n", x, y, dp[x][y], sol[dp[x][y]], w ? w : "ok");
			}
		}
	}
	return 0;
}