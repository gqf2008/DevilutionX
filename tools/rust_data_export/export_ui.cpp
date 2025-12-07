/**
 * @file export_ui.cpp
 * @brief 导出 UI 布局数据供 Rust 验证使用 - 完整版
 */

#include "export_types.h"
#include <iostream>

namespace devilution::rust_export {

void ExportUILayoutData(const std::string& outputPath)
{
    JsonWriter writer(outputPath);

    writer.StartObject();
    writer.WriteKeyValue("type", "ui_layout_data");
    writer.WriteKeyValue("description", "DevilutionX UI Layout - Comprehensive Test Data");
    writer.WriteKeyValue("version", "2.0");

    // ========== 1. 基础分辨率和缩放 ==========
    writer.WriteKey("resolutions");
    writer.StartObject();

    writer.WriteKey("logical");
    writer.StartObject();
    writer.WriteKeyValue("width", 640);
    writer.WriteKeyValue("height", 480);
    writer.WriteKeyValue("description", "Base logical resolution for all UI coordinates");
    writer.EndObject();

    writer.WriteKey("common_window_sizes");
    writer.StartArray();
    int sizes[][2] = {{640, 480}, {800, 600}, {1024, 768}, {1280, 720},
                      {1280, 960}, {1366, 768}, {1600, 900}, {1920, 1080},
                      {2560, 1440}, {3840, 2160}};
    for (auto& sz : sizes) {
        writer.StartObject();
        writer.WriteKeyValue("width", sz[0]);
        writer.WriteKeyValue("height", sz[1]);
        writer.WriteKeyValue("scale_x", static_cast<double>(sz[0]) / 640.0);
        writer.WriteKeyValue("scale_y", static_cast<double>(sz[1]) / 480.0);
        writer.EndObject();
    }
    writer.EndArray();
    writer.EndObject();

    // ========== 2. UiFlags 枚举完整定义 ==========
    writer.WriteKey("ui_flags");
    writer.StartArray();

    struct FlagDef {
        const char* name;
        uint32_t value;
        const char* description;
    };
    FlagDef flags[] = {
        {"None", 0, "No flags"},
        {"FontSize12", 1 << 0, "12pt font"},
        {"FontSize24", 1 << 1, "24pt font"},
        {"FontSize30", 1 << 2, "30pt font"},
        {"FontSize42", 1 << 3, "42pt font"},
        {"FontSize46", 1 << 4, "46pt font"},
        {"FontSizeDialog", 1 << 5, "Dialog font size"},
        {"ColorUiGold", 1 << 6, "Gold UI color"},
        {"ColorUiSilver", 1 << 7, "Silver UI color"},
        {"ColorUiGoldDark", 1 << 8, "Dark gold UI color"},
        {"ColorUiSilverDark", 1 << 9, "Dark silver UI color"},
        {"ColorDialogWhite", 1 << 10, "White dialog color"},
        {"ColorDialogYellow", 1 << 11, "Yellow dialog color"},
        {"ColorDialogRed", 1 << 12, "Red dialog color"},
        {"ColorYellow", 1 << 13, "Yellow color"},
        {"ColorGold", 1 << 14, "Gold color"},
        {"ColorBlack", 1 << 15, "Black color"},
        {"ColorWhite", 1 << 16, "White color"},
        {"ColorWhitegold", 1 << 17, "White-gold color"},
        {"ColorRed", 1 << 18, "Red color"},
        {"ColorBlue", 1 << 19, "Blue color"},
        {"ColorOrange", 1 << 20, "Orange color"},
        {"ColorButtonface", 1 << 21, "Button face color"},
        {"ColorButtonpushed", 1 << 22, "Button pushed color"},
        {"AlignCenter", 1 << 23, "Center alignment"},
        {"AlignRight", 1 << 24, "Right alignment"},
        {"VerticalCenter", 1 << 25, "Vertical center"},
        {"KerningFitSpacing", 1 << 26, "Fit kerning spacing"},
        {"ElementDisabled", 1 << 27, "Element is disabled"},
        {"ElementHidden", 1 << 28, "Element is hidden"},
        {"PentaCursor", 1 << 29, "Pentagon cursor"},
        {"Outlined", 1 << 30, "Outlined text"},
        {"NeedsNextElement", 1U << 31U, "Needs next element visible"},
    };

    for (const auto& f : flags) {
        writer.StartObject();
        writer.WriteKeyValue("name", f.name);
        writer.WriteKeyValue("value", static_cast<uint64_t>(f.value));
        writer.WriteKeyValue("hex", f.name); // Will be formatted separately
        writer.WriteKeyValue("description", f.description);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 3. 主菜单完整布局 ==========
    writer.WriteKey("main_menu_layout");
    writer.StartObject();

    // Logo
    writer.WriteKey("logo");
    writer.StartObject();
    writer.WriteKeyValue("x", 0);
    writer.WriteKeyValue("y", 5);
    writer.WriteKeyValue("width", 0);
    writer.WriteKeyValue("height", 0);
    writer.WriteKeyValue("flags", "AlignCenter");
    writer.WriteKeyValue("animated", true);
    writer.WriteKeyValue("frames", 15);
    writer.EndObject();

    // 菜单项
    writer.WriteKey("menu_items");
    writer.StartArray();

    struct MenuItem {
        const char* text;
        int y;
        int h;
    };
    MenuItem items[] = {
        {"Single Player", 190, 43},
        {"Multi Player", 235, 43},
        {"Replay Intro", 280, 43},
        {"Support", 325, 43},
        {"Show Credits", 370, 43},
        {"Exit Diablo", 415, 43},
    };

    for (const auto& item : items) {
        writer.StartObject();
        writer.WriteKeyValue("text", item.text);
        writer.WriteKeyValue("rect_x", 64);
        writer.WriteKeyValue("rect_y", item.y);
        writer.WriteKeyValue("rect_w", 510);
        writer.WriteKeyValue("rect_h", item.h);
        writer.WriteKeyValue("flags", "AlignCenter | FontSize42 | ColorUiGold");
        writer.EndObject();
    }
    writer.EndArray();

    // 版本文本
    writer.WriteKey("version_text");
    writer.StartObject();
    writer.WriteKeyValue("rect_x", 17);
    writer.WriteKeyValue("rect_y", 444);
    writer.WriteKeyValue("rect_w", 605);
    writer.WriteKeyValue("rect_h", 21);
    writer.WriteKeyValue("flags", "ColorUiSilver | AlignCenter");
    writer.EndObject();

    writer.EndObject();

    // ========== 4. Focus 选择器完整数据 ==========
    writer.WriteKey("focus_selector");
    writer.StartObject();

    writer.WriteKey("sprites");
    writer.StartArray();

    struct FocusSprite {
        const char* name;
        const char* file;
        int width;
        int height;
        int frames;
        int threshold;
    };
    FocusSprite focusSprites[] = {
        {"FOCUS_SMALL", "ui_art/focus16.pcx", 16, 16, 8, 0},
        {"FOCUS_MED", "ui_art/focus.pcx", 30, 30, 8, 30},
        {"FOCUS_BIG", "ui_art/focus42.pcx", 42, 42, 8, 42},
    };

    for (const auto& fs : focusSprites) {
        writer.StartObject();
        writer.WriteKeyValue("name", fs.name);
        writer.WriteKeyValue("file", fs.file);
        writer.WriteKeyValue("width", fs.width);
        writer.WriteKeyValue("height", fs.height);
        writer.WriteKeyValue("frames", fs.frames);
        writer.WriteKeyValue("height_threshold", fs.threshold);
        writer.WriteKeyValue("transparent_color", 250);
        writer.EndObject();
    }
    writer.EndArray();

    writer.WriteKey("size_selection_tests");
    writer.StartArray();
    for (int h = 1; h <= 100; h++) {
        int idx = (h >= 42) ? 2 : (h >= 30) ? 1 : 0;
        writer.StartObject();
        writer.WriteKeyValue("item_height", h);
        writer.WriteKeyValue("sprite_index", idx);
        writer.EndObject();
    }
    writer.EndArray();

    writer.WriteKey("render_position_formula");
    writer.StartObject();
    writer.WriteKeyValue("left_x", "rect.x");
    writer.WriteKeyValue("right_x", "rect.x + rect.w - sprite.width()");
    writer.WriteKeyValue("y", "rect.y + (rect.h - sprite.height()) / 2");
    writer.EndObject();

    writer.EndObject();

    // ========== 5. 对话框布局 ==========
    writer.WriteKey("dialogs");
    writer.StartArray();

    struct DialogDef {
        const char* name;
        int width;
        int height;
        const char* usage;
    };
    DialogDef dialogs[] = {
        {"small", 250, 150, "Simple confirmations"},
        {"medium", 350, 200, "Standard dialogs"},
        {"large", 450, 280, "Complex dialogs with lists"},
        {"progress", 300, 100, "Progress bars"},
        {"input", 400, 150, "Text input dialogs"},
        {"character_select", 580, 400, "Character selection screen"},
        {"save_load", 520, 350, "Save/Load game screens"},
    };

    for (const auto& d : dialogs) {
        writer.StartObject();
        writer.WriteKeyValue("name", d.name);
        writer.WriteKeyValue("width", d.width);
        writer.WriteKeyValue("height", d.height);
        writer.WriteKeyValue("center_x", (640 - d.width) / 2);
        writer.WriteKeyValue("center_y", (480 - d.height) / 2);
        writer.WriteKeyValue("usage", d.usage);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 6. 字体尺寸映射 ==========
    writer.WriteKey("font_sizes");
    writer.StartArray();

    struct FontSize {
        const char* name;
        int size;
        int line_height;
        const char* usage;
    };
    FontSize fonts[] = {
        {"FontSize12", 12, 14, "Small text, tooltips"},
        {"FontSize24", 24, 28, "Normal text"},
        {"FontSize30", 30, 35, "Medium headers"},
        {"FontSize42", 42, 48, "Menu items"},
        {"FontSize46", 46, 52, "Large headers"},
        {"FontSizeDialog", 22, 26, "Dialog text"},
    };

    for (const auto& f : fonts) {
        writer.StartObject();
        writer.WriteKeyValue("flag_name", f.name);
        writer.WriteKeyValue("pixel_size", f.size);
        writer.WriteKeyValue("line_height", f.line_height);
        writer.WriteKeyValue("usage", f.usage);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 7. 颜色定义 ==========
    writer.WriteKey("colors");
    writer.StartArray();

    struct ColorDef {
        const char* name;
        uint8_t r, g, b;
        const char* usage;
    };
    ColorDef colors[] = {
        {"UiGold", 255, 215, 0, "Primary UI text"},
        {"UiSilver", 192, 192, 192, "Secondary UI text"},
        {"UiGoldDark", 180, 150, 0, "Disabled gold text"},
        {"UiSilverDark", 128, 128, 128, "Disabled silver text"},
        {"DialogWhite", 255, 255, 255, "Dialog text"},
        {"DialogYellow", 255, 255, 0, "Highlighted dialog text"},
        {"DialogRed", 255, 0, 0, "Warning/error text"},
        {"Black", 0, 0, 0, "Background, shadows"},
        {"ButtonFace", 128, 128, 128, "Button normal state"},
        {"ButtonPushed", 96, 96, 96, "Button pressed state"},
    };

    for (const auto& c : colors) {
        writer.StartObject();
        writer.WriteKeyValue("name", c.name);
        writer.WriteKeyValue("r", c.r);
        writer.WriteKeyValue("g", c.g);
        writer.WriteKeyValue("b", c.b);
        writer.WriteKeyValue("hex_rgb", c.name); // placeholder
        writer.WriteKeyValue("usage", c.usage);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 8. UI 资源文件清单 ==========
    writer.WriteKey("ui_resources");
    writer.StartArray();

    const char* resources[] = {
        "ui_art/mainmenu.pcx",
        "ui_art/selgame.pcx",
        "ui_art/selhero.pcx",
        "ui_art/focus.pcx",
        "ui_art/focus16.pcx",
        "ui_art/focus42.pcx",
        "ui_art/logo.pcx",
        "ui_art/smlogo.pcx",
        "ui_art/cursor.pcx",
        "ui_art/heros.pcx",
        "ui_art/but_sml.pcx",
        "ui_art/but_xsm.pcx",
        "ui_art/scrollbar.pcx",
        "ui_art/sb_arrow.pcx",
        "ui_art/sb_thumb.pcx",
        "ui_art/prog_bg.pcx",
        "ui_art/prog_fil.pcx",
        "ui_art/lpanel.pcx",
        "ui_art/rpanel.pcx",
        "ui_art/menu.pcx",
        "ui_art/spellbk.pcx",
        "ui_art/spelli2.pcx",
        "ui_art/panel8.pcx",
        "ui_art/panel8bu.pcx",
        "fonts/bigpentspn.clx",
        "fonts/medpentspn.clx",
        "fonts/smlpentspn.clx",
    };

    for (const char* res : resources) {
        writer.WriteString(res);
    }
    writer.EndArray();

    // ========== 9. 坐标变换测试用例 ==========
    writer.WriteKey("coordinate_tests");
    writer.StartArray();

    // 测试 AlignCenter 计算
    for (int w = 100; w <= 600; w += 50) {
        for (int containerW = 640; containerW >= w; containerW -= 100) {
            writer.StartObject();
            writer.WriteKeyValue("element_width", w);
            writer.WriteKeyValue("container_width", containerW);
            writer.WriteKeyValue("centered_x", (containerW - w) / 2);
            writer.EndObject();
        }
    }
    writer.EndArray();

    writer.EndObject();

    std::cout << "UI layout data exported (comprehensive): " << outputPath << std::endl;
}

} // namespace devilution::rust_export
