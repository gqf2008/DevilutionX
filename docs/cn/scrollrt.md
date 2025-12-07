# 滚动渲染模块 (scrollrt.cpp / scrollrt.h)

## 概述

本模块是游戏的核心渲染系统，负责绘制地牢、怪物、玩家、物品、飞行物等所有游戏元素。

## 核心常量

```cpp
constexpr auto RightFrameDisplacement = Displacement { DunFrameWidth, 0 };
// 右半瓦片的偏移量 (32, 0)
```

## 渲染流程

### 主渲染循环

```
DrawAndBlit()
    ├── UndrawCursor()           // 移除光标
    ├── UpdateProgressToNextGameTick()
    ├── DrawView()               // 主视图渲染
    │   ├── CalcFirstTilePosition()
    │   ├── DrawGame()
    │   │   ├── UpdateMissilesRendererData()
    │   │   ├── Lightmap::build()
    │   │   ├── DrawFloor()      // 第一遍：绘制地板
    │   │   └── DrawTileContent()  // 第二遍：绘制墙壁/实体
    │   └── DrawAutomap()        // 小地图
    ├── DrawMainPanel()          // 控制面板
    ├── DrawCursor()             // 绘制光标
    └── RenderPresent()          // 提交到屏幕
```

---

## 核心函数

### DrawView

```cpp
void DrawView(const Surface &out, Point startPosition);
```

主视图渲染入口点。

**流程：**
1. 调整起始瓦片位置
2. 调用 `DrawGame` 执行实际渲染
3. 绘制小地图（如果开启）
4. 绘制UI元素（物品名称、怪物血条等）

---

### DrawGame

```cpp
void DrawGame(const Surface &fullOut, Point position, Displacement offset);
```

执行游戏场景的实际渲染。

**重要参数：**
- `position`: 起始瓦片的世界坐标
- `offset`: 屏幕偏移量

**两遍渲染策略：**
```cpp
// 第一遍：仅绘制地板瓦片
DrawFloor(out, lightmap, position, Point {} + offset, rows, columns);

// 第二遍：绘制墙壁、物体和实体
DrawTileContent(out, lightmap, position, Point {} + offset, rows, columns);
```

**缩放处理：**
```cpp
if (*GetOptions().Graphics.zoom) {
    Zoom(fullOut.subregionY(0, gnViewportHeight));
}
```

---

### DrawFloor

```cpp
void DrawFloor(const Surface &out, const Lightmap &lightmap,
               Point tilePosition, Point targetBufferPosition,
               int rows, int columns);
```

渲染所有地板瓦片。

**遍历模式：**
采用锯齿形遍历，模拟等轴测视角：

```
行 0:  ▪ ▪ ▪ ▪ ▪ (columns 个瓦片)
行 1:   ▪ ▪ ▪ ▪ ▪ ▪ (columns+1 个瓦片)
行 2:  ▪ ▪ ▪ ▪ ▪ (columns 个瓦片)
行 3:   ▪ ▪ ▪ ▪ ▪ ▪ (columns+1 个瓦片)
...
```

**关键逻辑：**
```cpp
for (int i = 0; i < rows; i++) {
    for (int j = 0; j < columns; j++) {
        if (!InDungeonBounds(tilePosition)) {
            world_draw_black_tile(out, targetBufferPosition.x, targetBufferPosition.y);
        } else if (IsFloor(tilePosition)) {
            DrawFloorTile(out, lightmap, tilePosition, targetBufferPosition);
        }
        tilePosition += Direction::East;
        targetBufferPosition.x += TILE_WIDTH;
    }
    // 行切换逻辑
    if ((i & 1) != 0) {
        tilePosition.x++;
        columns--;
        targetBufferPosition.x += TILE_WIDTH / 2;
    } else {
        tilePosition.y++;
        columns++;
        targetBufferPosition.x -= TILE_WIDTH / 2;
    }
    targetBufferPosition.y += TILE_HEIGHT / 2;
}
```

---

### DrawFloorTile

```cpp
void DrawFloorTile(const Surface &out, const Lightmap &lightmap,
                   Point tilePosition, Point targetBufferPosition);
```

渲染单个地板瓦片。

**实现：**
```cpp
// 获取瓦片 ID
const uint16_t levelPieceId = dPiece[tilePosition.x][tilePosition.y];

// 绘制左三角形 (mt[0])
const LevelCelBlock leftBlock { DPieceMicros[levelPieceId].mt[0] };
if (leftBlock.hasValue()) {
    RenderTileFrame(out, lightmap, targetBufferPosition,
                    TileType::LeftTriangle, ...);
}

// 绘制右三角形 (mt[1])，偏移 32 像素
const LevelCelBlock rightBlock { DPieceMicros[levelPieceId].mt[1] };
if (rightBlock.hasValue()) {
    RenderTileFrame(out, lightmap,
                    targetBufferPosition + RightFrameDisplacement,
                    TileType::RightTriangle, ...);
}
```

---

### DrawTileContent

```cpp
void DrawTileContent(const Surface &out, const Lightmap &lightmap,
                     Point tilePosition, Point targetBufferPosition,
                     int rows, int columns);
```

渲染瓦片内容（墙壁、实体）。

**特殊处理：**
- 额外渲染 `MicroTileLen` 行以处理高于地板的墙壁
- 处理墙后物体的特殊渲染顺序

---

### DrawCell

```cpp
void DrawCell(const Surface &out, const Lightmap lightmap,
              Point tilePosition, Point targetBufferPosition,
              int lightTableIndex);
```

渲染单个瓦片单元的完整内容。

**渲染顺序：**
1. 墙壁基元 (mt[0], mt[1] 如果不是地板)
2. 上层墙壁块 (mt[2] 到 mt[n])
3. 植被（如果 mt[0]/mt[1] 是 TransparentSquare）

**透明度处理：**
```cpp
bool transparency = TileHasAny(tilePosition, TileProperties::Transparent)
                    && TransList[dTransVal[tilePosition.x][tilePosition.y]];

// 根据透明度选择遮罩类型
const auto getFirstTileMaskLeft = [=](TileType tile) -> MaskType {
    if (transparency) {
        switch (tile) {
        case TileType::LeftTrapezoid:
        case TileType::TransparentSquare:
            return TileHasAny(tilePosition, TileProperties::TransparentLeft)
                ? MaskType::Left : MaskType::Solid;
        // ...
        }
    }
    return MaskType::Solid;
};
```

---

### DrawDungeon

```cpp
void DrawDungeon(const Surface &out, const Lightmap &lightmap,
                 Point tilePosition, Point targetBufferPosition);
```

绘制地牢瓦片的完整内容，包括所有实体。

**绘制顺序：**
1. `DrawCell` - 瓦片本身
2. 前置飞行物 (`MissilePreFlag`)
3. 尸体
4. 前置物体
5. 地面物品
6. 死亡玩家
7. 玩家
8. 怪物
9. 后置飞行物
10. 后置物体
11. 后置物品
12. 特殊瓦片（拱门/树叶）

---

## 实体渲染

### DrawPlayer

```cpp
void DrawPlayer(const Surface &out, const Player &player,
                Point tilePosition, Point targetBufferPosition,
                int lightTableIndex);
```

渲染玩家精灵。

**特殊处理：**
- 玩家高亮轮廓
- 红外视觉效果
- 光照应用
- 护盾/反射图标

---

### DrawMonster

```cpp
void DrawMonster(const Surface &out, Point tilePosition,
                 Point targetBufferPosition, const Monster &monster,
                 int lightTableIndex);
```

渲染怪物精灵。

**变色处理：**
```cpp
if (monster.isUnique())
    trn = monster.uniqueMonsterTRN.get();
if (monster.mode == MonsterMode::Petrified)
    trn = GetStoneTRN();
```

---

### DrawMissile

```cpp
void DrawMissile(const Surface &out, WorldTilePosition tilePosition,
                 Point targetBufferPosition, bool pre, int lightTableIndex);
```

渲染飞行物精灵。

**飞行物位置更新：**
```cpp
void UpdateMissileRendererData(Missile &m) {
    // 计算插值位置
    int progress = ProgressToNextGameTick;
    UpdateMissilePositionForRendering(m, progress);

    // 碰撞检测：如果新位置可能无效，回退到旧位置
    if (CouldMissileCollide(m.position.tileForRendering, ...)) {
        // 二分查找最后有效位置
        while (m.position.tile != m.position.tileForRendering) {
            progress -= 1;
            UpdateMissilePositionForRendering(m, progress);
        }
    }
}
```

---

## 视口计算

### CalcViewportGeometry

```cpp
void CalcViewportGeometry();
```

计算视口参数。

**输出变量：**
- `tileOffset`: 起始瓦片的屏幕偏移
- `tileShift`: 相对于玩家的瓦片偏移
- `tileColumns`: 每行瓦片数
- `tileRows`: 总行数

### CalcFirstTilePosition

```cpp
void CalcFirstTilePosition(Point &position, Displacement &offset);
```

计算第一个渲染瓦片的位置。

**考虑因素：**
- 玩家行走偏移
- 面板覆盖
- 缩放模式

---

### GetOffsetForWalking

```cpp
Displacement GetOffsetForWalking(const AnimationInfo &animationInfo,
                                  Direction dir, bool cameraMode = false);
```

获取行走动画的像素偏移。

**方向偏移表：**
```cpp
//                    South,      SouthWest,  West,       NorthWest,
//                    North,      NorthEast,  East,       SouthEast
const Displacement MovingOffset[8] = {
    {   0,  32 }, { -32,  16 }, { -64,   0 }, { -32, -16 },
    {   0, -32 }, {  32, -16 }, {  64,   0 }, {  32,  16 }
};
```

---

## 屏幕坐标转换

### GetScreenPosition

```cpp
Point GetScreenPosition(Point tile);
```

将世界瓦片坐标转换为屏幕坐标。

---

## 缩放处理

### Zoom

```cpp
void Zoom(const Surface &out);
```

将左上角区域放大 2 倍。

---

## 光标处理

### DrawCursor / UndrawCursor

光标使用双缓冲技术：
1. `UndrawCursor`: 恢复光标下的背景
2. `DrawCursor`: 保存背景并绘制光标

---

## 性能优化

### 调试统计

```cpp
#ifdef DUN_RENDER_STATS
ankerl::unordered_dense::map<DunRenderType, size_t> DunRenderStats;
#endif
```

启用后会统计每种瓦片类型的渲染次数。

### 光照优化

使用 `Lightmap::build()` 预计算每像素光照，支持：
- 完全黑暗
- 部分光照
- 完全光照
- 每像素光照

---

## 辅助函数

### IsFloor / IsWall

```cpp
[[nodiscard]] bool IsFloor(Point tilePosition) {
    return !TileHasAny(tilePosition, TileProperties::Solid | TileProperties::BlockMissile);
}

[[nodiscard]] bool IsWall(Point tilePosition) {
    return !IsFloor(tilePosition) || dSpecial[tilePosition.x][tilePosition.y] != 0;
}
```

### world_draw_black_tile

```cpp
void world_draw_black_tile(const Surface &out, int sx, int sy);
```

绘制边界外的黑色菱形瓦片。

---

## FPS 显示

```cpp
void DrawFPS(const Surface &out);
```

显示当前帧率（每秒更新一次）。

---

## 配置选项

- `GetOptions().Graphics.zoom`: 2 倍缩放模式
- `GetOptions().Graphics.perPixelLighting`: 每像素光照
- `GetOptions().Graphics.showFPS`: 显示 FPS
