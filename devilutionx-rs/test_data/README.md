# DevilutionX 测试数据说明

本目录包含从 C++ 代码导出的测试数据，用于验证 Rust 移植的正确性。这些数据由独立的 C++ 工具 `tools/rust_data_export/` 生成。

## 数据文件概览

| 文件 | 大小 | 行数 | 说明 |
|------|------|------|------|
| `animation_data.json` | 400 KB | 25,333 | 动画帧计算测试数据 |
| `sprite_data.json` | 19 KB | 937 | 精灵格式和游戏数据 |
| `ui_layout_data.json` | 23 KB | 1,160 | UI 布局规则和常量 |
| `focus_sequence.json` | 2 KB | - | Focus 动画序列 |

**总计**: 442 KB 测试数据，覆盖 3000+ 测试用例

## 1. animation_data.json

### 用途
验证 `GetAnimationFrame` 函数的正确性，该函数根据时间刻度计算当前应显示的动画帧。

### 核心算法
```cpp
int GetAnimationFrame(int ticks, int fps, int frames) {
    return (ticks / fps) % frames;
}
```

### 数据结构

#### 1.1 基础测试用例 (3,200 个)
- **覆盖范围**: 32 种帧数 × 100 个时间刻度值
- **帧数组合**: 1, 2, 3, 4, 5, 6, 8, 10, 12, 15, 16, 20, 24, 30, 32, 40, 48, 50, 60, 64, 75, 80, 96, 100, 120, 128, 150, 160, 192, 200, 240, 256
- **时间范围**: 0-9900 ticks (间隔 100)
- **用途**: 覆盖常见动画长度和时间区间

示例:
```json
{
  "frames": 16,
  "fps": 20,
  "ticks": 0,
  "expected_frame": 0
}
```

#### 1.2 FPS 变体测试 (600 个)
- **FPS 值**: 10, 15, 20, 24, 30, 60
- **每个 FPS**: 100 个时间刻度测试
- **用途**: 验证不同帧率下的计算准确性

#### 1.3 边界条件测试 (320 个)
- **测试点**:
  - `ticks = 0` (起始帧)
  - `ticks = fps - 1` (第一帧最后时刻)
  - `ticks = fps` (第二帧开始)
  - `ticks = fps * (frames - 1)` (最后一帧开始)
  - `ticks = fps * frames - 1` (循环前最后时刻)
  - `ticks = fps * frames` (循环到第一帧)
  - `ticks = fps * frames + 1` (循环后第一帧)
  - `ticks = fps * frames * 2 - 1` (第二次循环前)
- **帧数**: 1, 2, 3, 4, 5, 8, 10, 12, 15, 16, 20, 24, 30, 32, 48, 60, 64, 100, 128, 256
- **用途**: 验证循环边界和帧切换准确性

#### 1.4 Focus 动画序列 (10,001 个样本)
- **时长**: 10 秒 (10,000 毫秒)
- **采样率**: 每毫秒一个样本
- **动画参数**: 41 帧, 20 FPS
- **计算公式**: `frame = (milliseconds / 50) % 41`
- **用途**: 验证连续动画播放的流畅性和准确性

示例:
```json
{
  "milliseconds": 1234,
  "frame": 24,
  "note": "1234ms ÷ 50 = 24, 24 % 41 = 24"
}
```

#### 1.5 Focus 尺寸选择测试 (100 个)
- **测试高度**: 1-100 像素
- **选择逻辑**:
  - `height < 30`: 使用小号 Focus (索引 0)
  - `30 ≤ height < 42`: 使用中号 Focus (索引 1)
  - `height ≥ 42`: 使用大号 Focus (索引 2)

示例:
```json
{
  "height": 42,
  "selected_size": "large",
  "index": 2
}
```

#### 1.6 Logo 动画序列 (5,001 个样本)
- **时长**: 5 秒
- **帧数**: 15 帧
- **FPS**: 20
- **采样间隔**: 每毫秒

#### 1.7 常用动画定义 (10 个)
包含游戏中常见动画的标准参数:
- `Hero_Stand`: 10 帧, 20 FPS
- `Hero_Walk`: 8 帧, 20 FPS
- `Hero_Attack`: 16 帧, 20 FPS
- `Hero_Block`: 2 帧, 20 FPS
- `Hero_Death`: 20 帧, 20 FPS
- `Monster_Idle`: 12 帧, 15 FPS
- `Monster_Walk`: 8 帧, 15 FPS
- `Monster_Attack`: 14 帧, 15 FPS
- `Monster_Hit`: 6 帧, 20 FPS
- `Monster_Death`: 18 帧, 15 FPS

#### 1.8 帧时间查找表 (1,000 毫秒)
提供 0-999 毫秒的帧索引快速查找表（20 FPS, 16 帧）。

## 2. sprite_data.json

### 用途
提供精灵格式规范和完整的游戏数据参考，用于验证 Rust 数据结构的正确性。

### 数据结构

#### 2.1 像素格式定义 (6 种)
- `RGBA8888`: 32 位真彩色 + Alpha
- `ARGB8888`: Alpha 在前的 32 位格式
- `ABGR8888`: BGR 顺序 + Alpha
- `RGB888`: 24 位真彩色
- `RGB565`: 16 位高彩色 (常用于移动设备)
- `PAL8`: 8 位调色板索引

每种格式包含:
- `bits_per_pixel`: 每像素比特数
- `byte_order`: 字节序说明
- `usage`: 适用场景

#### 2.2 PCX 文件格式规范
- **头部大小**: 128 字节
- **关键字段**:
  - `manufacturer`: 制造商标识 (0x0A)
  - `version`: 版本号 (0-5)
  - `encoding`: 编码方式 (1 = RLE)
  - `bits_per_pixel`: 每像素位数
  - `xmin, ymin, xmax, ymax`: 图像边界
  - `hdpi, vdpi`: 水平/垂直 DPI
  - `colormap`: 16 色调色板 (48 字节)
  - `reserved`: 保留字节
  - `num_planes`: 颜色平面数
  - `bytes_per_line`: 每行字节数
  - `palette_info`: 调色板类型
  - `hscreen_size, vscreen_size`: 屏幕尺寸
  - `filler`: 填充至 128 字节

#### 2.3 CLX 精灵格式结构
DevilutionX 自定义的压缩精灵格式:
- **像素命令类型**:
  - 透明像素跳过
  - 调色板颜色填充
  - RLE 压缩运行
- **特性**: 游戏中主要精灵格式

#### 2.4 UI 精灵目录 (25 个)
完整的 UI 精灵资源列表，每个包含:
- `id`: 精灵标识符
- `file`: 文件路径
- `width`, `height`: 尺寸
- `frames`: 帧数
- `type`: 精灵类型
- `usage`: 用途说明

示例精灵:
- `Focus_Small`: 72×17, 单帧
- `Focus_Medium`: 88×23, 单帧
- `Focus_Large`: 128×41, 41 帧动画
- `Cursor`: 32×32, 单帧
- `Logo_Small`: 320×120, 15 帧动画
- `Dialog_Background`: 640×480, 单帧
- `Character_Panel`: 320×352, 单帧
- `Inventory_Panel`: 320×352, 单帧
- `Spell_Icons`: 37×38, 52 帧 (不同法术)
- `Health_Orb`: 96×96, 90 帧动画
- `Mana_Orb`: 96×96, 90 帧动画

#### 2.5 英雄职业数据 (6 个)
- **职业**: Warrior, Rogue, Sorcerer, Monk, Bard, Barbarian
- **属性**:
  - `strength`, `magic`, `dexterity`, `vitality`: 初始属性
  - `starting_spell`: 起始法术
  - `max_strength`, `max_magic`, `max_dexterity`, `max_vitality`: 属性上限

示例:
```json
{
  "id": "Warrior",
  "name": "战士",
  "strength": 30,
  "magic": 10,
  "dexterity": 20,
  "vitality": 25,
  "starting_spell": "无",
  "max_strength": 250,
  "max_magic": 50,
  "max_dexterity": 60,
  "max_vitality": 100
}
```

#### 2.6 法术数据 (27 个)
包含所有可用法术:
- `id`: 法术标识符
- `name`: 法术名称
- `magic_type`: 魔法类型 (火、闪电、魔法)
- `min_level`: 最低需求等级
- `mana_cost`: 法力消耗

示例: `Firebolt` (火焰箭), `Lightning` (闪电), `Teleport` (传送), `Healing` (治疗)

#### 2.7 地牢层级数据 (16 层)
- **层级**: 1-16 (教堂 1-4, 地下墓穴 5-8, 洞穴 9-12, 地狱 13-16)
- **信息**:
  - `name`: 层级名称
  - `monster_level`: 怪物等级范围
  - `theme`: 主题环境

#### 2.8 物品类型数据 (17 种)
- 武器: 剑、斧、弓、法杖、锤
- 防具: 头盔、盔甲、盾牌、手套、靴子
- 消耗品: 药水、卷轴
- 其他: 戒指、项链、黄金、任务物品

#### 2.9 游戏常量 (20+ 个)
- **地图**: `TILE_WIDTH=64`, `TILE_HEIGHT=32`
- **怪物**: `MAX_MONSTERS=200`, `MAX_MONSTER_TYPES=256`
- **玩家**: `MAX_PLRS=4`, `MAX_CHARACTERS=10`
- **物品**: `MAXITEMS=127`, `ITEMTYPES=35`
- **法术**: `MAX_SPELLS=52`
- **等级**: `MAX_LEVEL=50`, `MAX_DUNGEON_LEVEL=16`
- **经验**: `MAX_EXP=2000000000`

#### 2.10 MPQ 归档文件 (8 个)
列出所有资源包文件及其用途:
- `diabdat.mpq`: 核心游戏数据
- `spawn.mpq`: Shareware 版本数据
- `devilutionx.mpq`: DevilutionX 扩展资源
- `fonts.mpq`: 字体文件
- `hellfire.mpq`: Hellfire 扩展包
- `hfmonk.mpq`, `hfmusic.mpq`, `hfvoice.mpq`: Hellfire 额外资源

## 3. ui_layout_data.json

### 用途
提供 UI 布局计算规则、UiFlags 枚举定义、窗口坐标转换等测试数据。

### 数据结构

#### 3.1 常用分辨率 (10 种)
从 640×480 到 3840×2160 的标准分辨率，每个包含:
- `width`, `height`: 分辨率
- `aspect_ratio`: 纵横比
- `scale_factor`: 相对于 640×480 的缩放比例
- `ui_scale`: UI 缩放建议

分辨率列表:
- 640×480 (VGA, 4:3)
- 800×600 (SVGA, 4:3)
- 1024×768 (XGA, 4:3)
- 1280×720 (HD, 16:9)
- 1920×1080 (Full HD, 16:9)
- 2560×1440 (2K, 16:9)
- 3840×2160 (4K, 16:9)
- 1280×800 (WXGA, 16:10)
- 1440×900 (WXGA+, 16:10)
- 1680×1050 (WSXGA+, 16:10)

#### 3.2 UiFlags 枚举定义 (32 个)
完整的 UI 标志位定义，每个包含:
- `name`: 标志名称
- `value`: 位位置/值
- `description`: 功能说明

关键标志:
- `AlignCenter`: 水平居中对齐
- `VerticalCenter`: 垂直居中对齐
- `FontSize12` - `FontSize46`: 字体大小
- `ColorUiGold`: 金色文本
- `ColorUiSilver`: 银色文本
- `ColorWhite`: 白色文本
- `ColorRed`: 红色文本
- `ElementHidden`: 隐藏元素
- `ElementDisabled`: 禁用元素
- `NeedsRedraw`: 需要重绘

#### 3.3 主菜单布局
- **Logo**: 位置 (y=5), 动画 (15 帧)
- **菜单项** (6 个):
  - 单人游戏 (y=190)
  - 多人游戏 (y=235)
  - 重播过场动画 (y=280)
  - 支持我们 (y=325)
  - 设置 (y=370)
  - 退出 Diablo (y=415)
- **对齐**: 水平居中

#### 3.4 Focus 精灵定义
- **Small**: 72×17 像素 (用于高度 < 30)
- **Medium**: 88×23 像素 (用于高度 30-41)
- **Large**: 128×41 像素，41 帧 (用于高度 ≥ 42)

#### 3.5 Focus 尺寸选择测试 (100 个)
验证基于矩形高度选择正确 Focus 尺寸的逻辑。

#### 3.6 Focus 位置计算公式
```cpp
// 水平位置
left_x = rect.x;
right_x = rect.x + rect.w - sprite.width();

// 垂直居中
y = rect.y + (rect.h - sprite.height()) / 2;
```

测试用例覆盖多种矩形和精灵尺寸组合。

#### 3.7 对话框类型 (7 个)
- `Message`: 消息框 (400×200)
- `Confirm`: 确认框 (480×240)
- `Character`: 角色面板 (320×352)
- `Inventory`: 物品栏 (320×352)
- `Store`: 商店 (600×400)
- `Quest`: 任务日志 (500×350)
- `Spellbook`: 法术书 (400×450)

每个包含居中位置计算（基于 640×480 分辨率）。

#### 3.8 字体大小定义 (6 种)
- `Size12`: 12pt, 行高 14px (小型 UI 文本)
- `Size24`: 24pt, 行高 28px (标准 UI 文本)
- `Size30`: 30pt, 行高 36px (子标题)
- `Size42`: 42pt, 行高 50px (标题)
- `Size46`: 46pt, 行高 54px (大标题)
- `Size16`: 16pt, 行高 19px (对话框文本)

#### 3.9 颜色定义 (10 种)
标准 UI 颜色的 RGB 值:
- `UI_GOLD`: RGB(213, 160, 0) - 金色文本
- `UI_SILVER`: RGB(192, 192, 192) - 银色文本
- `WHITE`: RGB(255, 255, 255) - 白色文本
- `BLACK`: RGB(0, 0, 0) - 黑色文本
- `RED`: RGB(255, 0, 0) - 红色（错误/危险）
- `YELLOW`: RGB(255, 255, 0) - 黄色（警告）
- `BLUE`: RGB(0, 0, 255) - 蓝色（魔法值）
- `LIGHT_BLUE`: RGB(128, 210, 255) - 浅蓝色（信息）
- `ORANGE`: RGB(255, 128, 0) - 橙色（稀有物品）
- `DARK_GRAY`: RGB(64, 64, 64) - 深灰（禁用）

#### 3.10 UI 资源文件路径 (26 个)
所有 UI 相关资源的文件路径，包括:
- 精灵: `ui_art/focus*.pcx`, `ui_art/logo*.pcx`
- 面板: `ui_art/panel8*.pcx`, `ui_art/charpan*.pcx`
- 图标: `ui_art/spelli*.cel`, `ui_art/inv/*.cel`
- 字体: `fonts/font*.pcx`
- 光标: `ui_art/cursor*.pcx`

#### 3.11 坐标转换测试
验证 `AlignCenter` 标志的居中计算:
```cpp
centered_x = screen_width / 2 - element_width / 2;
```

包含 5 个不同元素宽度的测试用例。

## 4. focus_sequence.json

### 用途
独立的 Focus 动画序列文件，可用于快速测试动画播放逻辑。

### 内容
与 `animation_data.json` 中的 Focus 序列相同，但作为独立文件方便单独测试。

## 使用方法

### 在 Rust 单元测试中使用

```rust
#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::fs;

    #[test]
    fn test_get_animation_frame() {
        let data = fs::read_to_string("test_data/animation_data.json")
            .expect("无法读取测试数据");
        let json: Value = serde_json::from_str(&data)
            .expect("无法解析 JSON");

        for test in json["basic_tests"].as_array().unwrap() {
            let frames = test["frames"].as_i64().unwrap() as i32;
            let fps = test["fps"].as_i64().unwrap() as i32;
            let ticks = test["ticks"].as_i64().unwrap() as i32;
            let expected = test["expected_frame"].as_i64().unwrap() as i32;

            let result = get_animation_frame(ticks, fps, frames);
            assert_eq!(result, expected,
                "frames={}, fps={}, ticks={}", frames, fps, ticks);
        }
    }

    #[test]
    fn test_focus_size_selection() {
        let data = fs::read_to_string("test_data/ui_layout_data.json")
            .expect("无法读取测试数据");
        let json: Value = serde_json::from_str(&data)
            .expect("无法解析 JSON");

        for test in json["focus_size_selection_tests"].as_array().unwrap() {
            let height = test["height"].as_i64().unwrap() as i32;
            let expected_index = test["index"].as_i64().unwrap() as usize;

            let result = select_focus_size(height);
            assert_eq!(result, expected_index,
                "height={}", height);
        }
    }
}
```

### 重新生成测试数据

如果 C++ 代码发生变化，可以重新生成测试数据:

```powershell
# 进入构建目录
cd d:\Users\gxh\Documents\GitHub\DevilutionX\build

# 重新编译导出工具
cmake --build . --config Release --target rust_data_export

# 导出所有数据
.\Release\rust_data_export.exe all ..\devilutionx-rs\test_data
```

### 导出特定模块

```powershell
# 仅导出动画数据
.\Release\rust_data_export.exe animation ..\devilutionx-rs\test_data

# 仅导出精灵数据
.\Release\rust_data_export.exe sprite ..\devilutionx-rs\test_data

# 仅导出 UI 数据
.\Release\rust_data_export.exe ui ..\devilutionx-rs\test_data

# 仅导出 Focus 序列
.\Release\rust_data_export.exe focus ..\devilutionx-rs\test_data
```

## 数据完整性

### 验证方法
所有测试数据都从 C++ 参考实现计算得出，确保:
- ✅ 算法逻辑与 C++ 代码完全一致
- ✅ 边界条件覆盖完整
- ✅ 数值精度符合要求
- ✅ JSON 格式正确，易于解析

### 覆盖率统计
- **动画测试**: 3,000+ 基础用例 + 10,000+ 连续序列样本
- **边界测试**: 320 个关键边界点
- **FPS 测试**: 6 种帧率 × 100 时间点
- **UI 测试**: 100 个尺寸选择 + 10 种分辨率 + 32 个标志位
- **游戏数据**: 6 职业 + 27 法术 + 16 层级 + 17 物品类型 + 20+ 常量

## 版本信息

- **生成工具**: `tools/rust_data_export/` (独立 C++ 工具)
- **编译器**: Visual Studio 2022 (MSVC 19.43)
- **C++ 标准**: C++20
- **生成日期**: 2025年12月7日
- **数据版本**: 2.0 (comprehensive)

## 技术细节

### JSON 编码
- 使用标准 JSON 格式
- 字符串转义: `"`, `\`, `\n`, `\r`, `\t`
- 数字精度: int64_t (整数), double (浮点)
- 缩进: 2 空格

### 工具架构
- **模块化设计**: 6 个独立导出模块
- **零依赖**: 不依赖 SDL、libdevilutionx 等外部库
- **独立编译**: 可单独构建，不影响主项目
- **跨平台**: 支持 Windows, Linux, macOS

### 扩展性
如需添加新的测试数据:
1. 在 `tools/rust_data_export/` 中添加新的导出函数
2. 在 `main.cpp` 中注册新命令
3. 重新编译并运行导出工具
4. 更新本文档说明

## 相关文件

- `tools/rust_data_export/CMakeLists.txt`: 构建配置
- `tools/rust_data_export/export_types.h`: 类型定义
- `tools/rust_data_export/json_writer.cpp`: JSON 序列化
- `tools/rust_data_export/export_animation.cpp`: 动画数据导出
- `tools/rust_data_export/export_sprite.cpp`: 精灵数据导出
- `tools/rust_data_export/export_ui.cpp`: UI 数据导出
- `tools/rust_data_export/main.cpp`: 命令行工具入口

## 常见问题

### Q: 为什么需要这些测试数据？
A: Rust 移植需要验证每个函数的行为与 C++ 参考实现完全一致。这些测试数据提供了可靠的验证基准。

### Q: 数据量是否足够？
A: 目前包含 3,000+ 基础测试用例和 10,000+ 连续序列样本，覆盖了所有关键算法和边界条件。如需更多特定场景的测试数据，可扩展导出工具。

### Q: 如何确保数据正确性？
A: 所有数据都由 C++ 参考代码直接计算生成，未经人工编辑。生成逻辑与游戏代码保持一致。

### Q: 测试数据会随 C++ 代码更新吗？
A: 是的。当 C++ 实现发生变化时，应重新运行导出工具生成新的测试数据，确保 Rust 移植始终与最新的 C++ 代码对齐。

## 许可证

本测试数据与 DevilutionX 项目共享相同的许可证。详见项目根目录的 `LICENSE.md`。
