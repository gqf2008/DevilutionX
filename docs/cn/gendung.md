# 地牢生成模块 (gendung.h / gendung.cpp)

## 概述

本模块负责地牢的通用生成功能，包括瓦片数据加载、透明度处理、主题房间生成等核心功能。

## 常量定义

```cpp
#define MAXTHEMES 50     // 最大主题房间数
#define MAXTILES 1379    // 最大瓦片数量

#define DMAXX 40         // 地牢地图宽度（大块瓦片）
#define DMAXY 40         // 地牢地图高度（大块瓦片）
#define MAXDUNX 112      // 地牢地图宽度（小块瓦片）
#define MAXDUNY 112      // 地牢地图高度（小块瓦片）
```

## 枚举类型

### dungeon_type - 地牢类型

```cpp
enum dungeon_type : int8_t {
    DTYPE_TOWN,       // 0: 城镇
    DTYPE_CATHEDRAL,  // 1: 大教堂 (1-4层)
    DTYPE_CATACOMBS,  // 2: 地下墓穴 (5-8层)
    DTYPE_CAVES,      // 3: 洞穴 (9-12层)
    DTYPE_HELL,       // 4: 地狱 (13-16层)
    DTYPE_NEST,       // 5: 巢穴 (Hellfire)
    DTYPE_CRYPT,      // 6: 地穴 (Hellfire)

    DTYPE_LAST = DTYPE_CRYPT,
    DTYPE_NONE = -1,  // 无效类型
};
```

### _setlevels - 特殊关卡

```cpp
enum _setlevels : int8_t {
    SL_NONE,
    SL_SKELKING,      // 骷髅王关卡
    SL_BONECHAMB,     // 骨室
    SL_MAZE,          // 迷宫
    SL_POISONWATER,   // 毒水
    SL_VILEBETRAYER,  // 邪恶背叛者
    SL_ARENA_CHURCH,  // 竞技场（教堂）
    SL_ARENA_HELL,    // 竞技场（地狱）
    SL_ARENA_CIRCLE_OF_LIFE, // 竞技场（生命之环）

    SL_FIRST_ARENA = SL_ARENA_CHURCH,
    SL_LAST = SL_ARENA_CIRCLE_OF_LIFE,
};
```

### DungeonFlag - 地牢标志

```cpp
enum class DungeonFlag : uint8_t {
    None                 = 0,
    Missile              = 1 << 0,  // 有飞行物
    Visible              = 1 << 1,  // 当前可见
    DeadPlayer           = 1 << 2,  // 有死亡玩家
    Populated            = 1 << 3,  // 已放置物体
    MissileFireWall      = 1 << 4,  // 火墙飞行物
    MissileLightningWall = 1 << 5,  // 闪电墙飞行物
    Lit                  = 1 << 6,  // 被照亮
    Explored             = 1 << 7,  // 已探索
};
```

---

## 数据结构

### MegaTile - 大型瓦片

```cpp
struct MegaTile {
    uint16_t micro1;  // 左上子块
    uint16_t micro2;  // 右上子块
    uint16_t micro3;  // 左下子块
    uint16_t micro4;  // 右下子块
};
```

### THEME_LOC - 主题房间位置

```cpp
struct THEME_LOC {
    RectangleOf<uint8_t> room;  // 房间区域
    int8_t ttval;               // 透明度值
};
```

### Miniset - 迷你模板

```cpp
struct Miniset {
    WorldTileSize size;       // 模板大小
    uint8_t search[6][6];     // 搜索模式
    uint8_t replace[6][6];    // 替换模式

    // 检查位置是否匹配模板
    bool matches(WorldTilePosition position, bool respectProtected = true) const;

    // 放置模板
    void place(WorldTilePosition position, bool protect = false) const;
};
```

---

## 全局变量

### 地图数据

| 变量 | 类型 | 说明 |
|------|------|------|
| `dungeon[DMAXX][DMAXY]` | `uint8_t` | 瓦片 ID 地图（大块） |
| `pdungeon[DMAXX][DMAXY]` | `uint8_t` | 瓦片备份 |
| `dPiece[MAXDUNX][MAXDUNY]` | `uint16_t` | 细节瓦片 ID 地图 |
| `DPieceMicros[MAXTILES]` | `MICROS` | 每个瓦片的子块数据 |
| `SOLData[MAXTILES]` | `TileProperties` | 瓦片属性表 |

### 状态数据

| 变量 | 类型 | 说明 |
|------|------|------|
| `dLight[MAXDUNX][MAXDUNY]` | `uint8_t` | 实时光照值 |
| `dPreLight[MAXDUNX][MAXDUNY]` | `uint8_t` | 预计算静态光照 |
| `dFlags[MAXDUNX][MAXDUNY]` | `DungeonFlag` | 瓦片标志 |
| `dTransVal[MAXDUNX][MAXDUNY]` | `int8_t` | 透明度区域 |
| `TransList[256]` | `bool` | 透明度是否激活 |

### 实体数据

| 变量 | 类型 | 说明 |
|------|------|------|
| `dPlayer[MAXDUNX][MAXDUNY]` | `int8_t` | 玩家位置 |
| `dMonster[MAXDUNX][MAXDUNY]` | `int16_t` | 怪物/NPC 位置 |
| `dCorpse[MAXDUNX][MAXDUNY]` | `int8_t` | 尸体数据 |
| `dObject[MAXDUNX][MAXDUNY]` | `int8_t` | 物体位置 |
| `dSpecial[MAXDUNX][MAXDUNY]` | `int8_t` | 特殊瓦片帧 |

### 图形数据

| 变量 | 类型 | 说明 |
|------|------|------|
| `pMegaTiles` | `unique_ptr<MegaTile[]>` | 大型瓦片定义 |
| `pDungeonCels` | `unique_ptr<std::byte[]>` | 地牢图形数据 |
| `pSpecialCels` | `OptionalOwnedClxSpriteList` | 特殊瓦片图形 |

---

## 核心函数

### CreateDungeon

```cpp
void CreateDungeon(uint32_t rseed, lvl_entry entry);
```

创建地牢主函数。

**流程：**
1. 初始化全局数据（`InitGlobals`）
2. 根据 `leveltype` 调用对应的生成函数
3. 设置保护区域

**关卡类型映射：**
```cpp
switch (leveltype) {
    case DTYPE_TOWN:
        CreateTown(entry);
        break;
    case DTYPE_CATHEDRAL:
    case DTYPE_CRYPT:
        CreateL5Dungeon(rseed, entry);  // 使用L5生成器
        break;
    case DTYPE_CATACOMBS:
        CreateL2Dungeon(rseed, entry);
        break;
    case DTYPE_CAVES:
    case DTYPE_NEST:
        CreateL3Dungeon(rseed, entry);
        break;
    case DTYPE_HELL:
        CreateL4Dungeon(rseed, entry);
        break;
}
```

---

### LoadLevelSOLData

```cpp
tl::expected<void, std::string> LoadLevelSOLData();
```

加载当前关卡的瓦片属性（`.sol` 文件）。

**文件路径：**
- Town: `levels/towndata/town.sol`
- Cathedral: `levels/l1data/l1.sol`
- Catacombs: `levels/l2data/l2.sol`
- Caves: `levels/l3data/l3.sol`
- Hell: `levels/l4data/l4.sol`
- Nest: `nlevels/l6data/l6.sol`
- Crypt: `nlevels/l5data/l5.sol`

**注意：** 函数还会修正原版中标记错误的瓦片属性。

---

### SetDungeonMicros

```cpp
void SetDungeonMicros(
    std::unique_ptr<std::byte[]> &dungeonCels,
    uint_fast8_t &microTileLen
);
```

设置地牢的微块数据。

**微块数量：**
- Town: 16 块
- Hell: 12 块（实际使用 16 块的存储）
- 其他: 10 块

**流程：**
1. 加载 `.min` 文件（瓦片定义）
2. 解析每个瓦片的子块信息
3. 重编码 CEL 数据（`ReencodeDungeonCels`）
4. 计算块地址调整值

---

### DRLG_LPass3

```cpp
void DRLG_LPass3(int lv);
```

第三阶段处理：将 `dungeon` 数组转换为 `dPiece` 数组。

**转换规则：**
- 每个 `dungeon[i][j]` 大块变为 4 个 `dPiece` 小块
- 使用 `pMegaTiles[tileId]` 获取子块 ID

```cpp
// 大块到小块的映射
dPiece[xx + 0][yy + 0] = mega.micro1;
dPiece[xx + 1][yy + 0] = mega.micro2;
dPiece[xx + 0][yy + 1] = mega.micro3;
dPiece[xx + 1][yy + 1] = mega.micro4;
```

---

### 透明度处理

#### DRLG_InitTrans

```cpp
void DRLG_InitTrans();
```

初始化透明度系统。清空 `dTransVal` 并设置 `TransVal = 1`。

#### DRLG_RectTrans

```cpp
void DRLG_RectTrans(WorldTileRectangle area);
```

为矩形区域设置透明度值，然后递增 `TransVal`。

#### DRLG_MRectTrans

```cpp
void DRLG_MRectTrans(WorldTileRectangle area);
```

为大块矩形区域设置透明度（自动转换为小块坐标）。

#### FloodTransparencyValues

```cpp
void FloodTransparencyValues(uint8_t floorID);
```

使用泛洪填充算法为地板区域分配透明度值。

---

### 主题房间

#### DRLG_PlaceThemeRooms

```cpp
void DRLG_PlaceThemeRooms(
    int minSize, int maxSize,
    int floor, int freq,
    bool rndSize
);
```

放置主题房间。

**参数：**
- `minSize`/`maxSize`: 房间大小范围
- `floor`: 地板瓦片 ID
- `freq`: 放置频率（百分比）
- `rndSize`: 是否随机大小

#### IsNearThemeRoom

```cpp
bool IsNearThemeRoom(WorldTilePosition position);
```

检查位置是否靠近主题房间（2 格范围内）。

---

### 辅助函数

#### GetLevelType

```cpp
dungeon_type GetLevelType(int level);
```

根据层数返回地牢类型。

| 层数 | 类型 |
|------|------|
| 0 | DTYPE_TOWN |
| 1-4 | DTYPE_CATHEDRAL |
| 5-8 | DTYPE_CATACOMBS |
| 9-12 | DTYPE_CAVES |
| 13-16 | DTYPE_HELL |
| 17-20 | DTYPE_NEST |
| 21-24 | DTYPE_CRYPT |

#### InDungeonBounds

```cpp
constexpr bool InDungeonBounds(Point position);
```

检查坐标是否在地图边界内。

#### TileHasAny

```cpp
bool TileHasAny(Point coords, TileProperties property);
```

检查瓦片是否具有指定属性。

```cpp
// 使用示例
if (TileHasAny(position, TileProperties::Solid)) {
    // 瓦片不可通过
}
```

---

### PlaceMiniSet

```cpp
std::optional<Point> PlaceMiniSet(
    const Miniset &miniset,
    int tries = 199,
    bool drlg1Quirk = false
);
```

在地图上放置迷你模板。

**算法：**
1. 随机选择起始位置
2. 扫描尝试次数个位置
3. 检查是否与 `SetPieceRoom` 冲突
4. 检查模板是否匹配
5. 放置模板

---

### PlaceDunTiles

```cpp
void PlaceDunTiles(const uint16_t *dunData, Point position, int floorId = 0);
```

从 `.dun` 文件数据放置瓦片。

---

## 地图文件格式

### .dun 文件结构

```
┌────────────────────┐
│ width (uint16)     │
│ height (uint16)    │
├────────────────────┤
│ 瓦片层数据          │
│ (width × height)   │
├────────────────────┤
│ 透明度层数据        │
│ (width×2 × height×2) │
└────────────────────┘
```

### GetDunSize

```cpp
WorldTileSize GetDunSize(const uint16_t *dunData);
```

获取 `.dun` 文件的大小。

---

## 坐标系统

游戏使用两套坐标：

1. **大块坐标** (`dungeon` 数组): 40×40
2. **小块坐标** (`dPiece` 数组): 112×112

转换关系：
```cpp
// 大块 → 小块
small_x = big_x * 2 + 16;
small_y = big_y * 2 + 16;

// WorldTilePosition 提供 megaToWorld() 方法
```

边界 16 格是填充区域，实际游戏区域是 80×80。
