# 地牢瓦片定义 (dun_tile.hpp)

## 概述

本模块定义了地牢关卡中使用的瓦片类型、属性和相关数据结构。这是渲染系统的基础。

## 常量定义

```cpp
#define TILE_WIDTH 64   // 完整瓦片宽度（像素）
#define TILE_HEIGHT 32  // 完整瓦片高度（像素）
```

## 瓦片类型 (TileType)

枚举定义了六种不同的瓦片渲染类型：

```cpp
enum class TileType : uint8_t {
    Square,              // 0: 32x32 正方形
    TransparentSquare,   // 1: 32x32 透明正方形
    LeftTriangle,        // 2: 左指向三角形
    RightTriangle,       // 3: 右指向三角形
    LeftTrapezoid,       // 4: 左梯形
    RightTrapezoid,      // 5: 右梯形
};
```

### Square（正方形）

🮆 32x32 像素的实心正方形。

**数据编码：** 按像素顺序存储（从下到上）

---

### TransparentSquare（透明正方形）

🮆 32x32 像素的透明正方形，使用 RLE 编码。

**数据编码：**
```
每个 run 以 int8_t 值开始：
- 正值 N：后跟 N 个像素
- 负值 -N：表示 N 个完全透明像素（省略）

run 不跨越行边界
```

---

### LeftTriangle（左三角形）

🭮 32x31 像素的左指向三角形。

**形状示意：**
```
        ╲
       ╲╲
      ╲╲╲
     ╲╲╲╲
    ╲╲╲╲╲
     ╲╲╲╲
      ╲╲╲
       ╲╲
        ╲
```

**数据编码：**
- 31 行变宽数据
- 最窄行（底部和顶部）为 2 像素
- 最宽行（中间）为 32 像素
- 偶数行前有 2 字节填充（在 `ReencodeDungeonCels` 中移除）

---

### RightTriangle（右三角形）

🭬 32x31 像素的右指向三角形。

**形状示意：**
```
╱
╱╱
╱╱╱
╱╱╱╱
╱╱╱╱╱
╱╱╱╱
╱╱╱
╱╱
╱
```

**数据编码：**
- 与 LeftTriangle 类似
- 偶数行后有 2 字节填充

---

### LeftTrapezoid（左梯形）

🭓 32x32 像素的左梯形：上半部分为矩形，下半部分为左三角形。

**形状示意：**
```
╲╲╲╲╲╲╲╲╲╲╲╲╲╲╲╲
╲╲╲╲╲╲╲╲╲╲╲╲╲╲╲╲
╲╲╲╲╲╲╲╲╲╲╲╲╲╲╲╲
   ╲╲╲╲╲╲╲╲╲╲╲╲
      ╲╲╲╲╲╲╲╲
         ╲╲╲╲
            ╲
```

**数据编码：**
1. 先是三角形部分（使用 LeftTriangle 编码）
2. 然后是矩形部分（像素数组）

---

### RightTrapezoid（右梯形）

🭞 32x32 像素的右梯形：上半部分为矩形，下半部分为右三角形。

---

## 数据结构

### LevelCelBlock

描述 MIN 文件中的单个图块块。

```cpp
struct LevelCelBlock {
    uint16_t data;

    // 是否有有效数据
    [[nodiscard]] bool hasValue() const { return data != 0; }

    // 获取瓦片类型（高 3 位）
    [[nodiscard]] TileType type() const {
        return static_cast<TileType>((data & 0x7000) >> 12);
    }

    // 获取帧索引（低 12 位，1-based）
    [[nodiscard]] uint16_t frame() const {
        return data & 0xFFF;
    }
};
```

**数据位布局：**
```
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│ │T│T│T│F│F│F│F│F│F│F│F│F│F│F│F│
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
 15 14-12       11-0
    类型        帧索引
```

---

### TileProperties

瓦片属性标志位。

```cpp
enum class TileProperties : uint8_t {
    None             = 0,
    Solid            = 1 << 0,  // 实心，阻挡移动
    BlockLight       = 1 << 1,  // 阻挡光线
    BlockMissile     = 1 << 2,  // 阻挡飞行物
    Transparent      = 1 << 3,  // 透明
    TransparentLeft  = 1 << 4,  // 左侧透明
    TransparentRight = 1 << 5,  // 右侧透明
    Trap             = 1 << 7,  // 陷阱
};
```

这些属性从 `.sol` 文件加载，用于碰撞检测和渲染决策。

---

### MICROS

描述一个完整的大型瓦片（Mega Tile）的所有子块。

```cpp
struct MICROS {
    LevelCelBlock mt[16];  // 最多 16 个子块
};
```

**子块布局（以 10 块瓦片为例）：**
```
      mt[8] mt[9]       ← 顶部
      mt[6] mt[7]
      mt[4] mt[5]
      mt[2] mt[3]
      mt[0] mt[1]       ← 底部（地板）
```

---

## 渲染常量

```cpp
// 渲染基元宽度（半个瓦片）
constexpr int_fast16_t DunFrameWidth = TILE_WIDTH / 2;  // 32

// 渲染基元高度（三角形除外）
constexpr int_fast16_t DunFrameHeight = TILE_HEIGHT;  // 32

// 三角形高度
constexpr int_fast16_t DunFrameTriangleHeight = 31;

// 重编码后的帧大小
constexpr size_t ReencodedTriangleFrameSize = 544 - 32;   // 512
constexpr size_t ReencodedTrapezoidFrameSize = 800 - 16;  // 784
```

---

## 辅助函数

### CalculateSpriteTileCenterX

```cpp
constexpr int CalculateSpriteTileCenterX(int width)
{
    return (width - TILE_WIDTH) / 2;
}
```

计算精灵相对于瓦片中心的 X 偏移。

---

## 等轴测坐标说明

游戏使用等轴测（菱形）坐标系统：

```
        North
          ╱╲
    West ╱  ╲ East
         ╲  ╱
          ╲╱
        South
```

一个完整瓦片（64×32 像素）由两个 32×32 的半瓦片组成：
- 左三角形（mt[0]）
- 右三角形（mt[1]）

**世界坐标到屏幕坐标转换：**
```cpp
screen_x = (tile_y - tile_x) * (TILE_WIDTH / 2);   // 32
screen_y = (tile_y + tile_x) * (TILE_HEIGHT / 2);  // 16
```

---

## 使用示例

```cpp
// 检查瓦片是否有效
LevelCelBlock block = DPieceMicros[pieceId].mt[0];
if (block.hasValue()) {
    TileType type = block.type();
    uint16_t frame = block.frame();

    // 根据类型选择渲染方法
    switch (type) {
    case TileType::LeftTriangle:
        // 渲染左三角形
        break;
    case TileType::RightTriangle:
        // 渲染右三角形
        break;
    // ...
    }
}

// 检查瓦片属性
if (HasAnyOf(SOLData[pieceId], TileProperties::Solid)) {
    // 瓦片不可通过
}
```
