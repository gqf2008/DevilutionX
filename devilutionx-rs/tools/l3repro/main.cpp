// Standalone C++ L3 (Caves) generator repro.
// Compiles drlg_l3.cpp + random.cpp against stub headers and mirrors the
// gendung.cpp helpers used by drlg_l3.cpp (PlaceMiniSet, DRLG_PlaceThemeRooms,
// CreateThemeRoom CAVES branch, GetSizeForThemeRoom, IsNearThemeRoom,
// PlaceDunTiles) so we can dump the current-HEAD C++ cave grid for a seed and
// compare it with the Rust port / .dun fixtures.
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <memory>
#include <optional>
#include <string>
#include <vector>

#include "diablo.h"
#include "engine/load_file.hpp"
#include "engine/random.hpp"
#include "levels/drlg_l3.h"
#include "levels/gendung.h"
#include "quests.h"
#include "utils/is_of.hpp"

namespace devilution {

// ---- globals (mirror gendung.cpp) ----
Bitset2d<DMAXX, DMAXY> DungeonMask;
uint8_t dungeon[DMAXX][DMAXY];
uint8_t pdungeon[DMAXX][DMAXY];
Bitset2d<DMAXX, DMAXY> Protected;
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

uint16_t Swap16LE(uint16_t v)
{
	// Little-endian host: SDL_SwapLE16 is the identity.
	return v;
}

WorldTileSize GetDunSize(const uint16_t *dunData)
{
	return WorldTileSize(static_cast<WorldTileCoord>(Swap16LE(dunData[0])), static_cast<WorldTileCoord>(Swap16LE(dunData[1])));
}

void PlaceDunTiles(const uint16_t *dunData, Point position, int floorId)
{
	const WorldTileSize size = GetDunSize(dunData);
	const uint16_t *tileLayer = &dunData[2];
	for (WorldTileCoord j = 0; j < size.height; j++) {
		for (WorldTileCoord i = 0; i < size.width; i++) {
			auto tileId = static_cast<uint8_t>(Swap16LE(tileLayer[j * size.width + i]));
			if (tileId != 0) {
				dungeon[position.x + i][position.y + j] = tileId;
				Protected.set(position.x + i, position.y + j);
			} else if (floorId != 0) {
				dungeon[position.x + i][position.y + j] = floorId;
			}
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
			if (!valid) {
				continue;
			}
		}
		if (SetPieceRoom.contains(position))
			continue;
		if (!miniset.matches(position))
			continue;
		miniset.place(position);
		return position;
	}
	return {};
}

bool IsNearThemeRoom(WorldTilePosition testPosition)
{
	for (int i = 0; i < themeCount; i++) {
		if (WorldTileRectangle(themeLoc[i].room.position - WorldTileDisplacement { 2 }, themeLoc[i].room.size + 5).contains(testPosition))
			return true;
	}
	return false;
}

std::optional<WorldTileSize> GetSizeForThemeRoom(uint8_t floor, WorldTilePosition origin, WorldTileCoord minSize, WorldTileCoord maxSize)
{
	if (origin.x + maxSize > DMAXX && origin.y + maxSize > DMAXY) {
		return {};
	}
	if (IsNearThemeRoom(origin)) {
		return {};
	}
	const WorldTileCoord maxWidth = std::min<WorldTileCoord>(maxSize, DMAXX - origin.x);
	const WorldTileCoord maxHeight = std::min<WorldTileCoord>(maxSize, DMAXY - origin.y);
	WorldTileSize room { maxWidth, maxHeight };
	for (WorldTileCoord i = 0; i < maxSize; i++) {
		WorldTileCoord width = i < room.height ? i : 0;
		if (i < maxHeight) {
			while (width < room.width) {
				if (dungeon[origin.x + width][origin.y + i] != floor)
					break;
				width++;
			}
		}
		WorldTileCoord height = i < room.width ? i : 0;
		if (i < maxWidth) {
			while (height < room.height) {
				if (dungeon[origin.x + i][origin.y + height] != floor)
					break;
				height++;
			}
		}
		if (width < minSize || height < minSize) {
			if (i < minSize)
				return {};
			break;
		}
		room = { std::min(room.width, width), std::min(room.height, height) };
	}
	return room - 2;
}

void CreateThemeRoom(int themeIndex)
{
	const int lx = themeLoc[themeIndex].room.position.x;
	const int ly = themeLoc[themeIndex].room.position.y;
	const int hx = lx + themeLoc[themeIndex].room.size.width;
	const int hy = ly + themeLoc[themeIndex].room.size.height;
	for (int yy = ly; yy < hy; yy++) {
		for (int xx = lx; xx < hx; xx++) {
			if (leveltype == DTYPE_CATACOMBS) {
				if (yy == ly || yy == hy - 1) {
					dungeon[xx][yy] = 2;
				} else if (xx == lx || xx == hx - 1) {
					dungeon[xx][yy] = 1;
				} else {
					dungeon[xx][yy] = 3;
				}
			}
			if (IsAnyOf(leveltype, DTYPE_CAVES, DTYPE_NEST)) {
				if (yy == ly || yy == hy - 1) {
					dungeon[xx][yy] = 134;
				} else if (xx == lx || xx == hx - 1) {
					dungeon[xx][yy] = 137;
				} else {
					dungeon[xx][yy] = 7;
				}
			}
		}
	}
	if (leveltype == DTYPE_CATACOMBS) {
		dungeon[lx][ly] = 8;
		dungeon[hx - 1][ly] = 7;
		dungeon[lx][hy - 1] = 9;
		dungeon[hx - 1][hy - 1] = 6;
	}
	if (IsAnyOf(leveltype, DTYPE_CAVES, DTYPE_NEST)) {
		dungeon[lx][ly] = 150;
		dungeon[hx - 1][ly] = 151;
		dungeon[lx][hy - 1] = 152;
		dungeon[hx - 1][hy - 1] = 138;
	}
	if (leveltype == DTYPE_CATACOMBS) {
		if (FlipCoin())
			dungeon[hx - 1][(ly + hy) / 2] = 4;
		else
			dungeon[(lx + hx) / 2][hy - 1] = 5;
	}
	if (IsAnyOf(leveltype, DTYPE_CAVES, DTYPE_NEST)) {
		if (FlipCoin())
			dungeon[hx - 1][(ly + hy) / 2] = 147;
		else
			dungeon[(lx + hx) / 2][hy - 1] = 146;
	}
}

void DRLG_MRectTrans(WorldTileRectangle area) {}
void DRLG_RectTrans(WorldTileRectangle area) {}
void DRLG_InitTrans() {}
void FloodTransparencyValues(uint8_t floorID) {}
void DRLG_CheckQuests(WorldTilePosition) {}
void DRLG_LPass3(int) {}
void Make_SetPC(WorldTileRectangle) {}
void DoLighting(Point, int, Point) {}
void AddL3Objs(int, int, int, int) {}
void InitLevels() {}
void LoadDungeonBase(const char *, Point, int, int) {}

void DRLG_PlaceThemeRooms(int minSize, int maxSize, int floor, int freq, bool rndSize)
{
	themeCount = 0;
	memset(themeLoc, 0, sizeof(*themeLoc));
	for (WorldTileCoord j = 0; j < DMAXY; j++) {
		for (WorldTileCoord i = 0; i < DMAXX; i++) {
			if (dungeon[i][j] == floor && FlipCoin(freq)) {
				std::optional<WorldTileSize> themeSize = GetSizeForThemeRoom(floor, { i, j }, minSize, maxSize);
				if (!themeSize)
					continue;
				if (rndSize) {
					const int min = minSize - 2;
					const int max = maxSize - 2;
					themeSize->width = min + GenerateRnd(GenerateRnd(themeSize->width - min + 1));
					if (themeSize->width < min || themeSize->width > max)
						themeSize->width = min;
					themeSize->height = min + GenerateRnd(GenerateRnd(themeSize->height - min + 1));
					if (themeSize->height < min || themeSize->height > max)
						themeSize->height = min;
				}
				THEME_LOC &theme = themeLoc[themeCount];
				theme.room = { WorldTilePosition { i, j } + Direction::South, *themeSize };
				if (IsAnyOf(leveltype, DTYPE_CAVES, DTYPE_NEST)) {
					DRLG_RectTrans({ (theme.room.position + Direction::South).megaToWorld(), theme.room.size * 2 - 5 });
				} else {
					DRLG_MRectTrans({ theme.room.position, theme.room.size - 1 });
				}
				theme.ttval = TransVal - 1;
				CreateThemeRoom(themeCount);
				themeCount++;
			}
		}
	}
}

// ---- LoadFileInMem<uint16_t> stub: raw bytes into u16 array ----
template <>
std::unique_ptr<uint16_t[]> LoadFileInMem<uint16_t>(const char *path)
{
	std::string resolved = path;
	const std::string prefix = "levels\\\\l3data\\\\";
	if (resolved.rfind(prefix, 0) == 0)
		resolved = "test/fixtures/levels/l3data/" + resolved.substr(prefix.size());
	std::ifstream f(resolved, std::ios::binary);
	if (!f)
		return nullptr;
	f.seekg(0, std::ios::end);
	std::streamsize len = f.tellg();
	f.seekg(0, std::ios::beg);
	std::vector<uint8_t> bytes((size_t)len);
	f.read(reinterpret_cast<char *>(bytes.data()), len);
	std::unique_ptr<uint16_t[]> out(new uint16_t[(bytes.size() + 1) / 2]);
	std::memcpy(out.get(), bytes.data(), bytes.size());
	return out;
}

// ---- Quest::IsAvailable ----
bool Quest::IsAvailable() const
{
	if (currlevel != _qlevel)
		return false;
	if (_qactive == QUEST_NOTAVAIL)
		return false;
	return true;
}

} // namespace devilution

int main(int argc, char **argv)
{
	using namespace devilution;
	uint32_t seed = 262005438;
	if (argc > 1)
		seed = (uint32_t)std::strtoul(argv[1], nullptr, 10);

	for (auto &o : LevelSeeds)
		o = std::nullopt;
	currlevel = 9;
	leveltype = DTYPE_CAVES;
	for (auto &q : Quests) {
		q._qactive = QUEST_NOTAVAIL;
		q._qlevel = 0;
		q._qidx = Q_ANVIL;
	}
	SetPieceRoom = WorldTileRectangle(WorldTilePosition { 0, 0 }, WorldTileSize { 0, 0 });
	SetPiece = WorldTileRectangle(WorldTilePosition { 0, 0 }, WorldTileSize { 0, 0 });

	CreateL3Dungeon(seed, ENTRY_MAIN);

	for (int y = 0; y < DMAXY; y++) {
		for (int x = 0; x < DMAXX; x++) {
			printf("%d%c", dungeon[x][y], x == DMAXX - 1 ? '\n' : ' ');
		}
	}
	return 0;
}
