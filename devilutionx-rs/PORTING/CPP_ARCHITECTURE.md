# DevilutionX C++ 架构分析文档

> 本文档详细分析 C++ 代码架构，用于指导 Rust 移植工作。

## 目录

1. [启动流程](#1-启动流程)
2. [UI 场景系统](#2-ui-场景系统)
3. [动画系统](#3-动画系统)
4. [地图渲染系统](#4-地图渲染系统)
5. [模块依赖图](#5-模块依赖图)
6. [Rust 移植核对清单](#6-rust-移植核对清单)

---

## 1. 启动流程

### 1.1 主入口 (DiabloMain)

**文件**: `Source/diablo.cpp:2696`

```
DiabloMain(argc, argv)
│
├── 1. DiabloParseFlags(argc, argv)     // 解析命令行参数
│
├── 2. InitKeymapActions()              // 初始化键盘映射
├── 3. InitPadmapActions()              // 初始化手柄映射
│
├── 4. MpqManager 初始化
│   ├── LoadCoreArchives()              // 加载 devilutionx.mpq, fonts.mpq
│   ├── LoadOptions()                   // 加载配置文件
│   └── LoadLanguageArchive()           // 加载语言包 (zh_cn.mpq 等)
│
├── 5. ApplicationInit()                // 应用程序初始化
│   ├── init_create_window()            // 创建 SDL 窗口
│   ├── InitializeScreenReader()        // 屏幕阅读器 (无障碍)
│   ├── LanguageInitialize()            // 语言系统初始化
│   └── SetApplicationVersions()        // 设置版本信息
│
├── 6. LuaInitialize()                  // Lua 脚本引擎
│
├── 7. 游戏数据加载
│   ├── LoadGameArchives()              // 加载 spawn.mpq/diabdat.mpq
│   ├── LoadTextData()                  // 文本数据
│   ├── LoadPlayerDataFiles()           // 玩家数据
│   ├── LoadSpellData()                 // 法术数据
│   ├── LoadMissileData()               // 飞行物数据
│   ├── LoadMonsterData()               // 怪物数据
│   ├── LoadItemData()                  // 物品数据
│   ├── LoadObjectData()                // 对象数据
│   └── LoadQuestData()                 // 任务数据
│
├── 8. DiabloInit()                     // Diablo 核心初始化
│   ├── gbIsSpawn 判断                  // 是否为共享版
│   ├── gbIsHellfire 判断               // 是否为 Hellfire
│   ├── InitializeVirtualGamepad()      // 虚拟手柄
│   ├── UiInitialize()                  // ★ UI 系统初始化
│   ├── UiSelStartUpGameOption()        // 游戏模式选择 (如有多个)
│   ├── DiabloInitScreen()              // 屏幕初始化
│   ├── snd_init()                      // 音频系统
│   ├── ui_sound_init()                 // UI 音效
│   ├── InitItemGFX()                   // 物品图形
│   └── LoadSmallSelectionSpinner()     // 小型选择器
│
├── 9. DiabloSplash()                   // ★ 启动画面
│   ├── play_movie("logo.smk")          // Logo 动画
│   ├── play_movie("diablo1.smk")       // 介绍视频
│   └── UiTitleDialog()                 // ★ 标题画面
│
└── 10. mainmenu_loop()                 // ★ 主菜单循环
```

### 1.2 关键函数详解

#### UiInitialize() - UI 系统初始化

**文件**: `Source/DiabloUI/diabloui.cpp:680`

```cpp
void UiInitialize()
{
    LoadUiGFX();  // 加载所有 UI 图形资源

    if (ArtCursor) {
        SDLC_HideCursor();  // 使用自定义光标
    }
}

void LoadUiGFX()
{
    // 1. Logo 动画 (Hellfire 优先)
    ArtLogo = LoadPcxSpriteList("ui_art\\hf_logo2", 16, 0);
    if (!ArtLogo) {
        ArtLogo = LoadPcxSpriteList("ui_art\\smlogo", 15, 250);
    }

    // 2. 难度指示器
    DifficultyIndicator = LoadPcx("ui_art\\r1_gry", 0);

    // 3. Focus 选择器 (3种大小, 各8帧)
    ArtFocus[FOCUS_SMALL] = LoadPcxSpriteList("ui_art\\focus16", 8, 250);
    ArtFocus[FOCUS_MED]   = LoadPcxSpriteList("ui_art\\focus", 8, 250);
    ArtFocus[FOCUS_BIG]   = LoadPcxSpriteList("ui_art\\focus42", 8, 250);

    // 4. 鼠标光标
    ArtCursor = LoadPcx("ui_art\\cursor", 0);

    // 5. 英雄肖像
    LoadHeros();
}
```

**加载的资源列表**:

| 变量 | 文件 | 帧数 | 透明色 | 尺寸 |
|------|------|------|--------|------|
| `ArtLogo` | `ui_art/hf_logo2.pcx` 或 `ui_art/smlogo.pcx` | 16/15 | 0/250 | 550x216 |
| `ArtFocus[SMALL]` | `ui_art/focus16.pcx` | 8 | 250 | 16x16 |
| `ArtFocus[MED]` | `ui_art/focus.pcx` | 8 | 250 | 30x30 |
| `ArtFocus[BIG]` | `ui_art/focus42.pcx` | 8 | 250 | 42x42 |
| `ArtCursor` | `ui_art/cursor.pcx` | 1 | 0 | - |
| `ArtHero` | `ui_art/heros.pcx` | 按高度分割 | - | 76px 高 |

---

## 2. UI 场景系统

### 2.1 场景列表

| 场景 | 函数 | 文件 | 说明 |
|------|------|------|------|
| 标题画面 | `UiTitleDialog()` | `title.cpp` | 显示 Logo + "Press any key" |
| 主菜单 | `UiMainMenuDialog()` | `mainmenu.cpp` | 6个菜单项 |
| 角色选择 | `UiSelHeroDialog()` | `selgame.cpp` | 选择/创建角色 |
| 难度选择 | `UiSelDifficultyDialog()` | `selgame.cpp` | 普通/噩梦/地狱 |
| 多人游戏 | `UiMultiPlayerDialog()` | `multi.cpp` | 联机设置 |
| 设置 | `UiSettingsDialog()` | `settingsmenu.cpp` | 游戏设置 |
| 制作人员 | `UiCreditsDialog()` | `credits.cpp` | 滚动字幕 |

### 2.2 标题画面 (UiTitleDialog)

**文件**: `Source/DiabloUI/title.cpp`

```cpp
void UiTitleDialog()
{
    TitleLoad();  // 加载资源

    // 构建 UI 元素列表
    vecTitleScreen.clear();

    if (ArtBackgroundWidescreen.has_value()) {
        // Hellfire 风格: 宽屏背景 + 火焰动画背景
        // ...
    } else {
        // Diablo 风格
        UiAddBackground(&vecTitleScreen);  // 静态背景

        // 动画 Logo (y = 182)
        vecTitleScreen.push_back(std::make_unique<UiImageAnimatedClx>(
            *DiabloTitleLogo,
            MakeSdlRect(0, uiPosition.y + 182, 0, 0),
            UiFlags::AlignCenter));

        // 版权信息 (y = 410)
        vecTitleScreen.push_back(std::make_unique<UiArtText>(
            copyright, copyrightRect,
            UiFlags::FontSize12 | UiFlags::ColorUiSilverDark | UiFlags::AlignCenter));
    }

    // 事件循环 (7秒超时)
    Uint32 timeOut = SDL_GetTicks() + 7000;
    while (!endMenu && SDL_GetTicks() < timeOut) {
        UiRenderItems(vecTitleScreen);  // 渲染所有 UI 元素
        UiFadeIn();                      // 淡入效果
        // 事件处理...
    }

    TitleFree();
}
```

**关键坐标**:
- Logo 位置: `y = uiPosition.y + 182`
- 版权信息: `y = 410`
- 超时时间: 7000ms

### 2.3 主菜单 (UiMainMenuDialog)

**文件**: `Source/DiabloUI/mainmenu.cpp`

```cpp
void MainmenuLoad(const char *name)
{
    // 1. 创建菜单项
    vecMenuItems.push_back(std::make_unique<UiListItem>(_("Single Player"), MAINMENU_SINGLE_PLAYER));
    vecMenuItems.push_back(std::make_unique<UiListItem>(_("Multi Player"), MAINMENU_MULTIPLAYER));
    vecMenuItems.push_back(std::make_unique<UiListItem>(_("Settings"), MAINMENU_SETTINGS));
    vecMenuItems.push_back(std::make_unique<UiListItem>(_("Support"), MAINMENU_SHOW_SUPPORT));
    vecMenuItems.push_back(std::make_unique<UiListItem>(_("Show Credits"), MAINMENU_SHOW_CREDITS));
    vecMenuItems.push_back(std::make_unique<UiListItem>(_("Exit Diablo"), MAINMENU_EXIT_DIABLO));

    // 2. 加载背景
    if (!gbIsSpawn || gbIsHellfire) {
        ArtBackgroundWidescreen = LoadOptionalClx("ui_art\\mainmenuw.clx");
        LoadBackgroundArt("ui_art\\mainmenu");
    } else {
        LoadBackgroundArt("ui_art\\swmmenu");  // Spawn 版本用不同背景
    }

    // 3. 添加 UI 元素
    UiAddBackground(&vecMainMenuDialog);
    UiAddLogo(&vecMainMenuDialog);

    // 4. 菜单列表
    vecMainMenuDialog.push_back(std::make_unique<UiList>(
        vecMenuItems,
        vecMenuItems.size(),
        uiPosition.x + 64,      // x = 64
        uiPosition.y + 192,     // y = 192
        510,                    // width = 510
        43,                     // height per item = 43
        UiFlags::FontSize42 | UiFlags::ColorUiGold | UiFlags::AlignCenter,
        5                       // spacing
    ));

    // 5. 版本信息 (底部)
    vecMainMenuDialog.push_back(std::make_unique<UiArtText>(
        name, rect2,
        UiFlags::FontSize12 | UiFlags::ColorUiSilverDark));

    // 6. 初始化列表系统
    UiInitList(nullptr, UiMainMenuSelect, MainmenuEsc, vecMainMenuDialog, true);
}
```

**关键布局参数**:
```
菜单列表:
  x = uiPosition.x + 64 = 64
  y = uiPosition.y + 192 = 192
  width = 510
  item_height = 43

菜单项 Y 坐标:
  [0] Single Player: 192
  [1] Multi Player:  235 (192 + 43)
  [2] Settings:      278 (192 + 86)
  [3] Support:       321 (192 + 129)
  [4] Credits:       364 (192 + 172)
  [5] Exit:          407 (192 + 215)
```

### 2.4 DrawSelector - 选择器渲染

**文件**: `Source/DiabloUI/diabloui.cpp:815`

```cpp
void DrawSelector(const SDL_Rect &rect)
{
    // 1. 根据项目高度选择合适的 Focus 大小
    const ClxSpriteList sprites = GetListSelectorSprites(rect.h);

    // 2. 获取当前动画帧 (使用默认 fps=60)
    const ClxSprite sprite = sprites[GetAnimationFrame(sprites.numSprites())];

    // 3. 垂直居中
    const int y = rect.y + ((rect.h - static_cast<int>(sprite.height())) / 2);

    // 4. 渲染左右两侧
    const Surface &out = Surface(DiabloUiSurface());
    RenderClxSprite(out, sprite, { rect.x, y });
    RenderClxSprite(out, sprite, { rect.x + rect.w - sprite.width(), y });
}

ClxSpriteList GetListSelectorSprites(int itemHeight)
{
    int size;
    if (itemHeight >= 42) {
        size = FOCUS_BIG;       // focus42.pcx
    } else if (itemHeight >= 30) {
        size = FOCUS_MED;       // focus.pcx
    } else {
        size = FOCUS_SMALL;     // focus16.pcx
    }
    return *ArtFocus[size];
}
```

**计算示例 (主菜单, item_h=43)**:
```
Focus 大小: FOCUS_BIG (42x42), 因为 item_h >= 42
动画帧: (SDL_GetTicks() / 60) % 8
Y 坐标: item_y + (43 - 42) / 2 = item_y + 0 ≈ item_y
左侧 X: rect.x = 64
右侧 X: rect.x + rect.w - sprite.width() = 64 + 510 - 42 = 532
```

---

## 3. 动画系统

### 3.1 GetAnimationFrame

**文件**: `Source/engine/ticks.cpp`

```cpp
uint32_t GetAnimationFrame(uint32_t frames, uint32_t fps)
{
    return (SDL_GetTicks() / fps) % frames;
}
```

**参数**:
- `frames`: 动画总帧数
- `fps`: 每帧持续时间 (毫秒), **默认值 = 60**

**公式**:
```
current_frame = floor(SDL_GetTicks() / fps) % frames
```

**示例**:
```
Focus 动画 (8帧, fps=60):
  t=0ms:   frame = (0 / 60) % 8 = 0
  t=60ms:  frame = (60 / 60) % 8 = 1
  t=120ms: frame = (120 / 60) % 8 = 2
  ...
  t=420ms: frame = (420 / 60) % 8 = 7
  t=480ms: frame = (480 / 60) % 8 = 0 (循环)

完整循环时间: 8 * 60ms = 480ms
```

### 3.2 PentSpn2Spin (用于游戏内 UI)

**文件**: `Source/engine/render/text_render.cpp:922`

```cpp
uint8_t PentSpn2Spin()
{
    return GetAnimationFrame(8, 50);  // 注意: 这里是 50ms 每帧!
}
```

**注意**: 游戏内 UI (商店、任务面板等) 使用 50ms 每帧, 而 DiabloUI 使用默认的 60ms。

### 3.3 Logo 动画

**文件**: `Source/DiabloUI/title.cpp`

```cpp
void TitleLoad()
{
    // Logo 动画: 15帧, 透明色=250
    DiabloTitleLogo = LoadPcxSpriteList("ui_art\\logo", 15, 250);
}

// 渲染 (在 UiImageAnimatedClx::Render 中)
void Render(const UiImageAnimatedClx &uiImage)
{
    // 获取当前帧
    const ClxSprite sprite = uiImage.sprite(GetAnimationFrame(uiImage.numFrames()));
    // ...渲染
}
```

---

## 4. 地图渲染系统

### 4.1 关卡类型

```cpp
enum dungeon_type {
    DTYPE_TOWN      = 0,    // 崔斯特瑞姆镇
    DTYPE_CATHEDRAL = 1,    // 大教堂 (地牢 1-4层)
    DTYPE_CATACOMBS = 2,    // 地下墓穴 (5-8层)
    DTYPE_CAVES     = 3,    // 洞穴 (9-12层)
    DTYPE_HELL      = 4,    // 地狱 (13-16层)
    DTYPE_NEST      = 5,    // Hellfire: 巢穴
    DTYPE_CRYPT     = 6,    // Hellfire: 地下室
};
```

### 4.2 文件格式

| 扩展名 | 用途 | 说明 |
|--------|------|------|
| `.cel` | 瓦片图形 | RLE 编码的图块数据 |
| `.til` | 大瓦片定义 | MegaTile 结构数组 |
| `.min` | 微型瓦片映射 | 定义每个瓦片由哪些 CEL 帧组成 |
| `.sol` | 碰撞属性 | 每个瓦片的通行性/光照阻挡等 |
| `.pal` | 调色板 | 256色调色板 |

### 4.3 瓦片结构

```cpp
// 一个微型瓦片块 (32x32 像素)
struct LevelCelBlock {
    uint16_t data;

    uint16_t frame() const { return data & 0x0FFF; }      // 帧索引 (低12位)
    uint8_t type() const { return (data & 0x7000) >> 12; } // 类型 (3位)
    bool hasValue() const { return data != 0; }
};

// 一个大瓦片 (由多个微型瓦片组成)
struct MICROS {
    LevelCelBlock mt[16];  // 最多16个微型瓦片
};

// 瓦片类型
enum TileType {
    LeftTriangle  = 0,  // 左三角
    RightTriangle = 1,  // 右三角
    LeftTrapezoid = 2,  // 左梯形
    RightTrapezoid= 3,  // 右梯形
    Square        = 4,  // 正方形 (32x32 全填充)
    TransSquare   = 5,  // 透明正方形
};
```

### 4.4 瓦片布局 (等距视角)

```
一个 MegaTile (64x160 像素) 的布局:

       [0]    [1]      <- 顶部 (y offset 最大)
    [2]    [3]
       [4]    [5]
    [6]    [7]
       [8]    [9]      <- 底部 (y offset = 0)

每个块是 32x32 像素
偶数索引在左, 奇数索引在右
```

### 4.5 渲染流程

```cpp
// 1. 加载关卡图形
void LoadLvlGFX()
{
    switch (leveltype) {
    case DTYPE_CATHEDRAL:
        pDungeonCels = LoadFileInMem("levels\\l1data\\l1.cel");
        pMegaTiles = LoadFileInMem<MegaTile>("levels\\l1data\\l1.til");
        // ...
    }
}

// 2. 设置瓦片映射
void SetDungeonMicros()
{
    // 加载 .min 文件, 构建 DPieceMicros 数组
    // 每个 DPieceMicros[n] 包含该瓦片的所有微型块信息
}

// 3. 渲染地图
void DrawGame()
{
    // 遍历可见区域
    for (每个可见瓦片) {
        // 获取瓦片ID
        int tileId = dPiece[x][y];

        // 获取该瓦片的微型块定义
        MICROS &micro = DPieceMicros[tileId];

        // 渲染每个微型块
        for (int i = 0; i < 16; i++) {
            if (micro.mt[i].hasValue()) {
                RenderTileFrame(micro.mt[i].frame(), micro.mt[i].type(), screenX, screenY);
            }
        }
    }
}
```

---

## 5. 模块依赖图

```
┌──────────────────────────────────────────────────────────────────────┐
│                           main() / DiabloMain()                       │
└────────────────────────────────┬─────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          初始化层                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │ MPQ Manager │  │   Options   │  │   Window    │  │    Audio    │  │
│  │  (archives) │  │   (config)  │  │    (SDL)    │  │   (sound)   │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │
└─────────┼────────────────┼────────────────┼────────────────┼─────────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          资源层                                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │     PCX     │  │     CLX     │  │    Fonts    │  │   Palettes  │  │
│  │   (images)  │  │  (sprites)  │  │   (text)    │  │   (colors)  │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │
└─────────┼────────────────┼────────────────┼────────────────┼─────────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
┌──────────────────────────────────────────────────────────────────────┐
│                           UI 层                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                        diabloui.cpp                             │ │
│  │  - LoadUiGFX() : ArtLogo, ArtFocus[3], ArtCursor, ArtHero      │ │
│  │  - UiInitialize() / UiDestroy()                                 │ │
│  │  - DrawSelector(), UiRenderItems(), UiFadeIn()                  │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │  title.cpp   │  │ mainmenu.cpp │  │ selgame.cpp  │  ...           │
│  │ UiTitleDialog│  │UiMainMenuDlg │  │UiSelHeroDlg  │                │
│  └──────────────┘  └──────────────┘  └──────────────┘                │
└──────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          游戏层                                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   Player    │  │   Monster   │  │    Items    │  │   Dungeon   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                      Level Generation                           │ │
│  │  drlg_l1.cpp (Cathedral), drlg_l2.cpp (Catacombs), ...         │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          渲染层                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                       clx_render.cpp                            │ │
│  │  - ClxDraw() : 基础 CLX 渲染                                    │ │
│  │  - RenderClxSprite() : 顶部坐标渲染                             │ │
│  │  - ClxDrawTRN() : 带颜色映射渲染                                │ │
│  │  - ClxDrawWithLightmap() : 带光照渲染                           │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                       dun_render.cpp                            │ │
│  │  - DrawDungeon() : 地牢渲染主函数                               │ │
│  │  - RenderTileFrame() : 单个瓦片渲染                             │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 6. Rust 移植核对清单

### 6.1 启动流程

| 步骤 | C++ 函数 | Rust 实现 | 状态 |
|------|----------|-----------|------|
| 命令行解析 | `DiabloParseFlags()` | `CmdFlags::parse()` | ✅ |
| 键盘映射 | `InitKeymapActions()` | `InitKeymapActions()` | ✅ |
| 手柄映射 | `InitPadmapActions()` | `InitPadmapActions()` | ✅ |
| 核心档案 | `LoadCoreArchives()` | `load_core_archives()` | ✅ |
| 配置加载 | `LoadOptions()` | `load_options()` | ✅ |
| 语言包 | `LoadLanguageArchive()` | `load_language_archive()` | ✅ |
| 窗口创建 | `init_create_window()` | SDL2 窗口创建 | ✅ |
| 游戏档案 | `LoadGameArchives()` | `load_game_archives()` | ✅ |
| UI 初始化 | `UiInitialize()` | `load_ui_assets()` | ⚠️ 需核对 |
| 启动画面 | `DiabloSplash()` | `diablo_splash()` | ⚠️ 需核对 |
| 主菜单 | `mainmenu_loop()` | `mainmenu_loop()` | ⚠️ 需核对 |

### 6.2 UI 资源加载

| 资源 | C++ 变量 | 文件 | 帧数 | 透明色 | Rust 状态 |
|------|----------|------|------|--------|-----------|
| Logo | `ArtLogo` | `ui_art/smlogo.pcx` | 15 | 250 | ⚠️ 需核对 |
| Focus小 | `ArtFocus[0]` | `ui_art/focus16.pcx` | 8 | 250 | ✅ |
| Focus中 | `ArtFocus[1]` | `ui_art/focus.pcx` | 8 | 250 | ✅ |
| Focus大 | `ArtFocus[2]` | `ui_art/focus42.pcx` | 8 | 250 | ✅ |
| 光标 | `ArtCursor` | `ui_art/cursor.pcx` | 1 | 0 | ✅ |
| 英雄 | `ArtHero` | `ui_art/heros.pcx` | 按高度 | - | ❌ |

### 6.3 动画系统

| 功能 | C++ 实现 | Rust 实现 | 状态 |
|------|----------|-----------|------|
| 帧计算 | `GetAnimationFrame(frames, fps=60)` | `(ticks / 60) % frames` | ⚠️ **之前用的是 50!** |
| Focus选择 | `GetListSelectorSprites(h)` | 根据 item_h 选择 | ⚠️ 需核对 |
| 渲染位置 | `RenderClxSprite(out, sprite, {x, y})` | `render_ui_image()` | ⚠️ 需核对 |

### 6.4 主菜单布局

| 参数 | C++ 值 | Rust 值 | 匹配 |
|------|--------|---------|------|
| list_x | `uiPosition.x + 64` = 64 | `ui_x + 64` | ✅ |
| list_y | `uiPosition.y + 192` = 192 | `ui_y + 192` | ✅ |
| item_w | 510 | 510 | ✅ |
| item_h | 43 | 43 | ✅ |
| Focus 选择 | `h >= 42 ? BIG : h >= 30 ? MED : SMALL` | ? | ⚠️ 需核对 |

### 6.5 DrawSelector 核对

**C++ 代码**:
```cpp
void DrawSelector(const SDL_Rect &rect)
{
    // 1. 选择 Focus 大小 (item_h=43 >= 42, 所以用 FOCUS_BIG)
    const ClxSpriteList sprites = GetListSelectorSprites(rect.h);

    // 2. 获取动画帧 (fps=60)
    const ClxSprite sprite = sprites[GetAnimationFrame(sprites.numSprites())];

    // 3. Y 坐标 (垂直居中)
    const int y = rect.y + ((rect.h - sprite.height()) / 2);

    // 4. 渲染
    RenderClxSprite(out, sprite, { rect.x, y });              // 左侧
    RenderClxSprite(out, sprite, { rect.x + rect.w - sprite.width(), y }); // 右侧
}
```

**Rust 应该是**:
```rust
// 1. 选择 Focus 大小
let focus_sprites = if item_h >= 42 {
    &assets.focus_big      // 42x42
} else if item_h >= 30 {
    &assets.focus_med      // 30x30
} else {
    &assets.focus_small    // 16x16
};

// 2. 获取动画帧 (fps=60, 不是 50!)
let frame_idx = ((SDL_GetTicks() / 60) as usize) % 8;

// 3. Y 坐标
let fy = item_y + (item_h - focus.height as i32) / 2;

// 4. X 坐标
let fx_left = list_x;                                    // = 64
let fx_right = list_x + item_w - focus.width as i32;    // = 64 + 510 - 42 = 532

// 5. 渲染
render_ui_image(canvas, creator, focus, fx_left, fy, fade)?;
render_ui_image(canvas, creator, focus, fx_right, fy, fade)?;
```

---

## 7. 已知问题和修复建议

### 7.1 动画帧率问题 ✅ 已修复

**问题**: Rust 代码之前使用 `ticks / 50`，应该是 `ticks / 60`

**C++ 代码** (`Source/engine/ticks.hpp`):
```cpp
uint32_t GetAnimationFrame(uint32_t frames, uint32_t fps = 60);
```

**状态**: 已修复，当前代码使用 `(ticks / 60) % 8`

### 7.2 Focus 大小选择问题 ✅ 已修复

**问题**: 主菜单 item_h=43，应该使用 `focus_big` (42x42)，不是 `focus_med` (30x30)

**C++ 逻辑**:
```cpp
if (itemHeight >= 42) size = FOCUS_BIG;
else if (itemHeight >= 30) size = FOCUS_MED;
else size = FOCUS_SMALL;
```

**状态**: 已修复，当前代码逻辑匹配 C++

### 7.3 像素格式问题 ✅ 已修复

**C++ SDL2**: 使用 `SDL_Surface` 直接操作像素，格式为 palette indexed 或 ARGB

**Rust SDL2**: 使用 `Texture` 渲染，需要正确的像素格式
- `full_game.rs` 使用 `PixelFormatEnum::ABGR8888`
- `main.rs` 已统一使用 `ABGR8888`

### 7.4 ⚠️ 可能的问题：Focus PCX 帧内容分析

**问题描述**: 用户报告动画"看起来像静态图片"，但代码逻辑正确

**可能原因**:
1. Focus PCX 文件本身各帧差异很小，肉眼难以区分
2. 纹理每帧都重新创建，可能有性能问题但不影响正确性
3. 帧率问题：60ms/帧 = 约 16.7 FPS，可能太快

**调试步骤**:
```rust
// 在 render_main_menu 中添加调试输出
static LAST_FRAME: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(999);
let prev = LAST_FRAME.swap(frame_idx, std::sync::atomic::Ordering::Relaxed);
if prev != frame_idx {
    eprintln!("[DEBUG] Animation frame changed: {} -> {} (ticks={})", prev, frame_idx, ticks);
}
```

**验证方法**:
1. 运行程序，观察控制台输出是否每 60ms 变化一次
2. 检查 focus42.pcx 的实际内容：是否每帧真的不同？
3. 对比 C++ 版本的 Focus 动画效果

---

## 附录: 关键文件路径

```
Source/
├── diablo.cpp              # 主入口和初始化
├── menu.cpp                # 主菜单循环
├── DiabloUI/
│   ├── diabloui.cpp        # UI 核心系统
│   ├── diabloui.h          # UI 声明
│   ├── title.cpp           # 标题画面
│   ├── mainmenu.cpp        # 主菜单
│   ├── selgame.cpp         # 角色选择
│   └── ui_item.h           # UI 元素定义
├── engine/
│   ├── ticks.cpp           # 动画帧计算
│   ├── load_pcx.cpp        # PCX 加载
│   └── render/
│       ├── clx_render.cpp  # CLX 渲染
│       └── clx_render.hpp  # CLX 渲染声明
└── levels/
    ├── gendung.cpp         # 地牢生成核心
    ├── drlg_l1.cpp         # 大教堂生成
    ├── drlg_l2.cpp         # 地下墓穴生成
    ├── drlg_l3.cpp         # 洞穴生成
    └── drlg_l4.cpp         # 地狱生成
```
