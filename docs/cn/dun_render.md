# 地牢瓦片渲染模块 (dun_render.hpp / dun_render.cpp)

## 概述

本模块实现了地牢瓦片的底层渲染功能，包括各种形状瓦片的绘制、光照处理和透明度混合。

## 遮罩类型 (MaskType)

```cpp
enum class MaskType : uint8_t {
    Solid,        // 完全不透明
    Transparent,  // 完全透明混合
    Right,        // 右上三角透明
    Left,         // 左上三角透明
};
```

### MaskType::Right

用于 `LeftTrapezoid` 和 `TransparentSquare` 瓦片。

```
上 16 行的透明度模式（🮆=不透明，🮐=混合）：
🮆🮆🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐
🮆🮆🮆🮆🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐🮐
... (每行增加 2 像素不透明区域)
🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆

下 16 行完全不透明
```

### MaskType::Left

用于 `RightTrapezoid` 和 `TransparentSquare` 瓦片，模式与 `Right` 镜像。

---

## 核心渲染函数

### RenderTileFrame

```cpp
void RenderTileFrame(
    const Surface &out,           // 输出表面
    const Lightmap &lightmap,     // 光照贴图
    const Point &position,        // 目标位置
    TileType tile,                // 瓦片类型
    const uint8_t *src,           // 源图像数据
    int_fast16_t height,          // 瓦片高度
    MaskType maskType,            // 遮罩类型
    const uint8_t *tbl            // 光照/调色表
);
```

根据瓦片类型和遮罩类型分发到具体的渲染函数。

---

### RenderTile

```cpp
void RenderTile(
    const Surface &out,
    const Lightmap &lightmap,
    const Point &position,
    const std::byte *dungeonCelData,
    LevelCelBlock levelCelBlock,
    MaskType maskType,
    const uint8_t *tbl
);
```

高层渲染接口，自动处理瓦片类型和帧数据获取。

```cpp
// 实现
const TileType tileType = levelCelBlock.type();
RenderTileFrame(out, lightmap, position, tileType,
    GetDunFrame(dungeonCelData, levelCelBlock.frame()),
    (tileType == TileType::LeftTriangle || tileType == TileType::RightTriangle)
        ? DunFrameTriangleHeight  // 31
        : DunFrameHeight,         // 32
    maskType, tbl);
```

---

### RenderTileFoliage

```cpp
void RenderTileFoliage(
    const Surface &out,
    const Lightmap &lightmap,
    const Point &position,
    const std::byte *dungeonCelData,
    LevelCelBlock levelCelBlock,
    const uint8_t *tbl
);
```

渲染地板上的植被（草、灌木等）。

植被数据紧跟在三角形帧数据之后。

---

### world_draw_black_tile

```cpp
void world_draw_black_tile(const Surface &out, int sx, int sy);
```

绘制完全黑色的菱形瓦片（64×31 像素），用于地图边界。

---

## 帧数据访问

### GetDunFrame

```cpp
const uint8_t *GetDunFrame(const std::byte *dungeonCelData, uint32_t frame);
```

获取指定帧的原始数据指针。

```cpp
// 实现
const auto *frameTable = reinterpret_cast<const uint32_t *>(dungeonCelData);
return reinterpret_cast<const uint8_t *>(
    &dungeonCelData[Swap32LE(frameTable[frame])]
);
```

### GetDunFrameFoliage

```cpp
const uint8_t *GetDunFrameFoliage(const std::byte *dungeonCelData, uint32_t frame);
```

获取植被帧数据（偏移 `ReencodedTriangleFrameSize`）。

---

## 内部渲染实现

### 光照类型

```cpp
enum class LightType : uint8_t {
    FullyDark,     // 完全黑暗 - 填充黑色
    PartiallyLit,  // 部分光照 - 使用光照表
    FullyLit,      // 完全光照 - 直接复制像素
    PerPixel,      // 每像素光照 - 使用光照贴图
};
```

### 行渲染模板

```cpp
// 不透明渲染
template <LightType Light>
void RenderLineOpaque(uint8_t *dst, const uint8_t *src,
                      uint_fast8_t n, const uint8_t *tbl,
                      const Lightmap *lightmap);

// 透明混合渲染
template <LightType Light>
void RenderLineTransparent(uint8_t *dst, const uint8_t *src,
                           uint_fast8_t n, const uint8_t *tbl,
                           const Lightmap *lightmap);
```

**特化实现示例：**

```cpp
// 完全黑暗 - 填充 0
template <>
void RenderLineOpaque<LightType::FullyDark>(...) {
    BlitFillDirect(dst, n, 0);
}

// 完全光照 - 直接复制
template <>
void RenderLineOpaque<LightType::FullyLit>(...) {
    BlitPixelsDirect(dst, src, n);
}

// 部分光照 - 查表转换
template <>
void RenderLineOpaque<LightType::PartiallyLit>(...) {
    BlitPixelsWithMap(dst, src, n, tbl);
}

// 每像素光照
template <>
void RenderLineOpaque<LightType::PerPixel>(...) {
    BlitPixelsWithLightmap(dst, src, n, *lightmap);
}
```

---

## 瓦片类型渲染

### RenderSquare (正方形)

```cpp
template <LightType Light, bool Transparent>
void RenderSquare(uint8_t *dst, uint16_t dstPitch, const uint8_t *src,
                  const uint8_t *tbl, const Lightmap &lightmap, Clip clip);
```

32×32 正方形瓦片的渲染。

---

### RenderTransparentSquare (透明正方形)

```cpp
template <LightType Light, MaskType Mask>
void RenderTransparentSquare(uint8_t *dst, uint16_t dstPitch,
                             const uint8_t *src, const uint8_t *tbl,
                             const Lightmap &lightmap, Clip clip);
```

处理 RLE 编码的透明正方形。

**RLE 解码逻辑：**
```cpp
while (drawWidth > 0) {
    auto v = static_cast<int8_t>(*src++);
    if (v > 0) {
        // 绘制 v 个像素
        RenderLine<Light, Mask>(dst, src, v, tbl, lightmap, prefix);
        src += v;
    } else {
        // 跳过 -v 个透明像素
        v = -v;
    }
    dst += v;
    drawWidth -= v;
}
```

---

### RenderLeftTriangle / RenderRightTriangle

```cpp
template <LightType Light, bool Transparent>
void RenderLeftTriangle(uint8_t *dst, uint16_t dstPitch, const uint8_t *src,
                        const uint8_t *tbl, const Lightmap *lightmap, Clip clip);
```

三角形瓦片由两部分组成：
- 下半部分：从 2 像素宽增加到 32 像素宽（16 行）
- 上半部分：从 30 像素宽减少到 2 像素宽（15 行）

**下半部分渲染（展开循环）：**
```cpp
template <LightType Light, bool Transparent>
void RenderTriangleLower(uint8_t *&dst, ptrdiff_t dstLineOffset,
                         const uint8_t *&src, ...) {
    RenderLineTransparentOrOpaque<Light, Transparent>(dst - 0*dstLineOffset, src+0, 2, ...);
    RenderLineTransparentOrOpaque<Light, Transparent>(dst - 1*dstLineOffset, src+2, 4, ...);
    RenderLineTransparentOrOpaque<Light, Transparent>(dst - 2*dstLineOffset, src+6, 6, ...);
    // ... 直到
    RenderLineTransparentOrOpaque<Light, Transparent>(dst - 15*dstLineOffset, src+240, 32, ...);
    src += 272;
    dst -= 16 * dstLineOffset;
}
```

---

### RenderLeftTrapezoid / RenderRightTrapezoid

```cpp
template <LightType Light, MaskType Mask>
void RenderLeftTrapezoid(uint8_t *dst, uint16_t dstPitch, const uint8_t *src,
                         const uint8_t *tbl, const Lightmap &lightmap, Clip clip);
```

梯形由三角形下半部分 + 矩形上半部分组成。

---

## 裁剪处理

### Clip 结构

```cpp
struct Clip {
    int_fast16_t top;      // 顶部裁剪行数
    int_fast16_t bottom;   // 底部裁剪行数
    int_fast16_t left;     // 左侧裁剪像素
    int_fast16_t right;    // 右侧裁剪像素
    int_fast16_t width;    // 实际绘制宽度
    int_fast16_t height;   // 实际绘制高度
};
```

### CalculateClip

```cpp
Clip CalculateClip(int_fast16_t x, int_fast16_t y,
                   int_fast16_t w, int_fast16_t h,
                   const Surface &out);
```

计算瓦片与屏幕边界的裁剪参数。

### DiamondClipY

```cpp
struct DiamondClipY {
    int_fast16_t lowerBottom;  // 下三角底部裁剪
    int_fast16_t lowerTop;     // 下三角顶部裁剪
    int_fast16_t upperBottom;  // 上三角底部裁剪
    int_fast16_t upperTop;     // 上三角顶部裁剪
};
```

用于三角形/梯形的垂直裁剪。

---

## 遮罩处理

### RenderLine 模板

```cpp
template <LightType Light, MaskType Mask>
void RenderLine(uint8_t *dst, const uint8_t *src, uint_fast8_t n,
                const uint8_t *tbl, const Lightmap &lightmap, int8_t prefix);
```

根据遮罩类型和前缀宽度决定如何渲染行：

```cpp
if constexpr (Mask == MaskType::Solid || Mask == MaskType::Transparent) {
    // 整行使用同一模式
    RenderLineTransparentOrOpaque<Light, Mask == Transparent>(dst, src, n, ...);
} else if (prefix >= n) {
    // 前缀超出行宽 - 整行不透明或透明
    if constexpr (Mask == MaskType::Right) {
        RenderLineOpaque<Light>(dst, src, n, ...);
    } else {
        RenderLineTransparent<Light>(dst, src, n, ...);
    }
} else if (prefix <= 0) {
    // 无前缀 - 整行相反模式
    // ...
} else {
    // 混合模式 - 分段渲染
    RenderLineTransparentAndOpaque<Light, Mask == Right>(
        dst, src, prefix, n, tbl, lightmap);
}
```

### 前缀增量

```cpp
// 左遮罩：透明区域从右向左增长
template <>
constexpr int8_t PrefixIncrement<MaskType::Left> = 2;

// 右遮罩：不透明区域从左向右收缩
template <>
constexpr int8_t PrefixIncrement<MaskType::Right> = -2;
```

---

## 调试功能

### 渲染统计

```cpp
#ifdef DUN_RENDER_STATS
ankerl::unordered_dense::map<DunRenderType, size_t, DunRenderTypeHash> DunRenderStats;

std::string_view TileTypeToString(TileType tileType);
std::string_view MaskTypeToString(MaskType maskType);
#endif
```

启用后可以统计每种瓦片/遮罩组合的渲染次数。

### 调试着色

```cpp
#ifdef DEBUG_RENDER_COLOR
int DBGCOLOR = 0;
int GetTileDebugColor(TileType tile);  // 返回不同瓦片类型的调试颜色
#endif
```

---

## 性能优化

1. **模板特化**：针对不同光照类型生成优化代码
2. **循环展开**：三角形渲染使用手动展开的循环
3. **编译器提示**：使用 `DVL_ALWAYS_INLINE` 和 `DVL_ATTRIBUTE_HOT`
4. **分支预测**：使用 `DVL_ASSUME` 提示编译器
5. **裁剪优化**：未裁剪瓦片使用专门的快速路径

---

## 常量

```cpp
constexpr int_fast16_t Width = DunFrameWidth;           // 32
constexpr int_fast16_t Height = DunFrameHeight;         // 32
constexpr int_fast16_t LowerHeight = Height / 2;        // 16
constexpr int_fast16_t TriangleUpperHeight = Height/2-1; // 15
constexpr int_fast16_t TrapezoidUpperHeight = Height/2;  // 16
constexpr int_fast16_t TriangleHeight = 31;
constexpr int_fast16_t XStep = 2;  // 三角形每行宽度变化
```
