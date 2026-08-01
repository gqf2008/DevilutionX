// Standalone cross-check for the L1 transparency pipeline.
//
// Copies the EXACT bodies of the current C++ transparency functions
// (gendung.cpp: IsFloor/FillTransparencyValues/FindTransparencyValues/
// FloodTransparencyValues + DRLG_CopyTrans and drlg_l1.cpp: FixTransparency
// and the EntranceStairs trans copy loop) and runs them on a dumped 40x40
// logical tile grid, printing the resulting dTransVal (112x112) row-major.
//
// Usage: l1trans_repro.exe <grid.txt>
// grid.txt: 40 lines of 40 space-separated tile ids (x fastest).
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <stack>
#include <vector>

namespace {
constexpr int DMAXX = 40;
constexpr int DMAXY = 40;
constexpr int MAXDUNX = 112;
constexpr int MAXDUNY = 112;

uint8_t dungeon[DMAXX][DMAXY];
uint8_t dTransVal[MAXDUNX][MAXDUNY];
int TransVal = 1;

struct Point {
    int x, y;
    Point(int x_, int y_) : x(x_), y(y_) {}
    Point operator+(Point o) const { return { x + o.x, y + o.y }; }
};
struct Displacement {
    int x, y;
    Displacement(int x_, int y_) : x(x_), y(y_) {}
};

// C++ engine/displacement.hpp fromDirection: North={-1,-1}, South={1,1},
// East={1,-1}, West={-1,1}, NorthEast={0,-1}, NorthWest={-1,0},
// SouthEast={1,0}, SouthWest={0,1}.
Displacement direction_delta(int dir) {
    switch (dir) {
    case 0: return {-1, -1}; // North
    case 1: return {1, 1};   // South
    case 2: return {1, -1};  // East
    case 3: return {-1, 1};  // West
    case 4: return {0, -1};  // NorthEast
    case 5: return {-1, 0};  // NorthWest
    case 6: return {1, 0};   // SouthEast
    default: return {0, 1};  // SouthWest
    }
}

bool IsFloor(Point p, uint8_t floorID) {
    const int i = (p.x - 16) / 2;
    const int j = (p.y - 16) / 2;
    if (i < 0 || i >= DMAXX) return false;
    if (j < 0 || j >= DMAXY) return false;
    return dungeon[i][j] == floorID;
}

void FillTransparencyValues(Point floor, uint8_t floorID) {
    const int allDirections[] = { 0, 1, 2, 3, 4, 5, 6, 7 }; // North..SouthWest
    for (int dir : allDirections) {
        Displacement d = direction_delta(dir);
        Point adjacent(floor.x + d.x, floor.y + d.y);
        if (adjacent.x < 0 || adjacent.y < 0 || adjacent.x >= MAXDUNX || adjacent.y >= MAXDUNY)
            continue;
        if (!IsFloor(adjacent, floorID))
            dTransVal[adjacent.x][adjacent.y] = TransVal;
    }
    dTransVal[floor.x][floor.y] = TransVal;
}

void FindTransparencyValues(Point floor, uint8_t floorID) {
    struct Seed { int scanStart, scanEnd, y, dy; };
    std::stack<Seed, std::vector<Seed>> seedStack;
    seedStack.push({ floor.x, floor.x + 1, floor.y, 1 });

    const auto isInside = [floorID](int x, int y) {
        if (x < 0 || y < 0 || x >= MAXDUNX || y >= MAXDUNY) return false;
        if (dTransVal[x][y] != 0) return false;
        return IsFloor({ x, y }, floorID);
    };
    const auto set = [floorID](int x, int y) { FillTransparencyValues({ x, y }, floorID); };
    const Displacement left(-1, 0);
    const Displacement right(1, 0);
    const auto checkDiagonals = [&](Point p, Displacement direction) {
        Point up(p.x, p.y - 1);
        Point upOver(up.x + direction.x, up.y + direction.y);
        if (!isInside(up.x, up.y) && isInside(upOver.x, upOver.y))
            seedStack.push({ upOver.x, upOver.x + 1, upOver.y, -1 });
        Point down(p.x, p.y + 1);
        Point downOver(down.x + direction.x, down.y + direction.y);
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

void FloodTransparencyValues(uint8_t floorID) {
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

void DRLG_CopyTrans(int sx, int sy, int dx, int dy) { dTransVal[dx][dy] = dTransVal[sx][sy]; }

// drlg_l1.cpp:1031-1064
void FixTransparency() {
    int yy = 16;
    for (int j = 0; j < DMAXY; j++) {
        int xx = 16;
        for (int i = 0; i < DMAXX; i++) {
            if (dungeon[i][j] == 23 && j > 0 && dungeon[i][j - 1] == 18) {
                dTransVal[xx + 1][yy] = dTransVal[xx][yy];
                dTransVal[xx + 1][yy + 1] = dTransVal[xx][yy];
            }
            if (dungeon[i][j] == 24 && i + 1 < DMAXY && dungeon[i + 1][j] == 19) {
                dTransVal[xx][yy + 1] = dTransVal[xx][yy];
                dTransVal[xx + 1][yy + 1] = dTransVal[xx][yy];
            }
            if (dungeon[i][j] == 18) {
                dTransVal[xx + 1][yy] = dTransVal[xx][yy];
                dTransVal[xx + 1][yy + 1] = dTransVal[xx][yy];
            }
            if (dungeon[i][j] == 19) {
                dTransVal[xx][yy + 1] = dTransVal[xx][yy];
                dTransVal[xx + 1][yy + 1] = dTransVal[xx][yy];
            }
            if (dungeon[i][j] == 20) {
                dTransVal[xx + 1][yy] = dTransVal[xx][yy];
                dTransVal[xx][yy + 1] = dTransVal[xx][yy];
                dTransVal[xx + 1][yy + 1] = dTransVal[xx][yy];
            }
            xx += 2;
        }
        yy += 2;
    }
}
} // namespace

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: l1trans_repro <grid.txt>\n");
        return 1;
    }
    FILE *f = fopen(argv[1], "r");
    if (!f) { fprintf(stderr, "cannot open %s\n", argv[1]); return 1; }
    for (int j = 0; j < DMAXY; j++) {
        for (int i = 0; i < DMAXX; i++) {
            int v;
            if (fscanf(f, "%d", &v) != 1) { fprintf(stderr, "short grid\n"); return 1; }
            dungeon[i][j] = (uint8_t)v;
        }
    }
    fclose(f);

    FloodTransparencyValues(13);
    // drlg_l1.cpp:1202-1211 (EntranceStairs == 64) then FixTransparency.
    for (int j = 0; j < DMAXY; j++) {
        for (int i = 0; i < DMAXX; i++) {
            if (dungeon[i][j] == 64) {
                const int xx = 2 * i + 16;
                const int yy = 2 * j + 16;
                DRLG_CopyTrans(xx, yy + 1, xx, yy);
                DRLG_CopyTrans(xx + 1, yy + 1, xx + 1, yy);
            }
        }
    }
    FixTransparency();

    for (int y = 0; y < MAXDUNY; y++) {
        for (int x = 0; x < MAXDUNX; x++) {
            printf("%d%c", dTransVal[x][y], x == MAXDUNX - 1 ? '\n' : ' ');
        }
    }
    return 0;
}
