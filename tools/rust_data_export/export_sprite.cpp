/**
 * @file export_sprite.cpp
 * @brief 导出精灵/图像和游戏数据供 Rust 验证使用 - 完整版
 */

#include "export_types.h"
#include <iostream>
#include <cstring>

namespace devilution::rust_export {

void ExportSpriteData(const std::string& outputPath, const std::string& spriteName)
{
    JsonWriter writer(outputPath);

    writer.StartObject();
    writer.WriteKeyValue("type", "sprite_and_game_data");
    writer.WriteKeyValue("version", "2.0");
    writer.WriteKeyValue("description", "DevilutionX Sprite and Game Constants - Comprehensive");

    // ========== 1. 像素格式定义 ==========
    writer.WriteKey("pixel_formats");
    writer.StartArray();

    struct PixelFormat {
        const char* name;
        int bpp;
        const char* layout;
        const char* usage;
    };
    PixelFormat formats[] = {
        {"RGBA8888", 32, "R8 G8 B8 A8", "Standard 32-bit with alpha"},
        {"ARGB8888", 32, "A8 R8 G8 B8", "Alternative 32-bit layout"},
        {"ABGR8888", 32, "A8 B8 G8 R8", "SDL default on little-endian"},
        {"RGB888", 24, "R8 G8 B8", "24-bit no alpha"},
        {"RGB565", 16, "R5 G6 B5", "16-bit color"},
        {"PAL8", 8, "Index8", "8-bit palettized"},
    };

    for (const auto& f : formats) {
        writer.StartObject();
        writer.WriteKeyValue("name", f.name);
        writer.WriteKeyValue("bits_per_pixel", f.bpp);
        writer.WriteKeyValue("layout", f.layout);
        writer.WriteKeyValue("usage", f.usage);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 2. PCX 文件格式 ==========
    writer.WriteKey("pcx_format");
    writer.StartObject();
    writer.WriteKeyValue("header_size", 128);
    writer.WriteKeyValue("magic", 10);
    writer.WriteKeyValue("version", 5);
    writer.WriteKeyValue("encoding", 1);
    writer.WriteKeyValue("palette_offset", -769);
    writer.WriteKeyValue("palette_marker", 12);

    writer.WriteKey("header_fields");
    writer.StartArray();
    struct HeaderField {
        const char* name;
        int offset;
        int size;
        const char* desc;
    };
    HeaderField fields[] = {
        {"magic", 0, 1, "Always 10 (0x0A)"},
        {"version", 1, 1, "PCX version (5 = 3.0)"},
        {"encoding", 2, 1, "1 = RLE"},
        {"bpp", 3, 1, "Bits per pixel per plane"},
        {"xmin", 4, 2, "Image bounds left"},
        {"ymin", 6, 2, "Image bounds top"},
        {"xmax", 8, 2, "Image bounds right"},
        {"ymax", 10, 2, "Image bounds bottom"},
        {"hdpi", 12, 2, "Horizontal DPI"},
        {"vdpi", 14, 2, "Vertical DPI"},
        {"colormap", 16, 48, "16-color palette"},
        {"reserved", 64, 1, "Reserved byte"},
        {"nplanes", 65, 1, "Number of color planes"},
        {"bytes_per_line", 66, 2, "Bytes per scanline"},
        {"palette_info", 68, 2, "Palette type"},
    };
    for (const auto& hf : fields) {
        writer.StartObject();
        writer.WriteKeyValue("name", hf.name);
        writer.WriteKeyValue("offset", hf.offset);
        writer.WriteKeyValue("size", hf.size);
        writer.WriteKeyValue("description", hf.desc);
        writer.EndObject();
    }
    writer.EndArray();
    writer.EndObject();

    // ========== 3. CLX 精灵格式 ==========
    writer.WriteKey("clx_format");
    writer.StartObject();
    writer.WriteKeyValue("description", "DevilutionX native sprite format");

    writer.WriteKey("structure");
    writer.StartArray();
    writer.WriteString("uint32_t num_groups");
    writer.WriteString("uint32_t frame_offsets[num_frames+1]");
    writer.WriteString("for each frame: ClxFrame data");
    writer.EndArray();

    writer.WriteKey("pixel_commands");
    writer.StartArray();
    struct PixelCmd {
        const char* name;
        int value_range_start;
        int value_range_end;
        const char* meaning;
    };
    PixelCmd cmds[] = {
        {"transparent", 0x00, 0x7F, "Skip N transparent pixels"},
        {"fill", 0x80, 0xBE, "Fill N pixels with next byte color"},
        {"pixels", 0xBF, 0xFF, "Copy N raw pixel bytes"},
    };
    for (const auto& cmd : cmds) {
        writer.StartObject();
        writer.WriteKeyValue("name", cmd.name);
        writer.WriteKeyValue("byte_range_start", cmd.value_range_start);
        writer.WriteKeyValue("byte_range_end", cmd.value_range_end);
        writer.WriteKeyValue("meaning", cmd.meaning);
        writer.EndObject();
    }
    writer.EndArray();
    writer.EndObject();

    // ========== 4. UI 精灵完整清单 ==========
    writer.WriteKey("ui_sprites");
    writer.StartArray();

    struct UISpriteInfo {
        const char* name;
        const char* file;
        int frames;
        int width;
        int height;
        const char* usage;
    };
    UISpriteInfo uiSprites[] = {
        {"focus_small", "ui_art/focus16.pcx", 8, 16, 16, "Small menu focus indicator"},
        {"focus_med", "ui_art/focus.pcx", 8, 30, 30, "Medium menu focus indicator"},
        {"focus_big", "ui_art/focus42.pcx", 8, 42, 42, "Large menu focus indicator"},
        {"logo", "ui_art/logo.pcx", 15, 137, 137, "Animated main menu logo"},
        {"smlogo", "ui_art/smlogo.pcx", 1, 64, 64, "Small logo"},
        {"cursor", "ui_art/cursor.pcx", 1, 33, 32, "Mouse cursor"},
        {"penta_big", "fonts/bigpentspn.clx", 8, 48, 48, "Large pentagon cursor"},
        {"penta_med", "fonts/medpentspn.clx", 8, 32, 32, "Medium pentagon cursor"},
        {"penta_small", "fonts/smlpentspn.clx", 8, 16, 16, "Small pentagon cursor"},
        {"mainmenu_bg", "ui_art/mainmenu.pcx", 1, 640, 480, "Main menu background"},
        {"selhero_bg", "ui_art/selhero.pcx", 1, 640, 480, "Select hero background"},
        {"selgame_bg", "ui_art/selgame.pcx", 1, 640, 480, "Select game background"},
        {"button_small", "ui_art/but_sml.pcx", 2, 110, 28, "Small button (normal/pressed)"},
        {"button_xsmall", "ui_art/but_xsm.pcx", 2, 72, 20, "Extra small button"},
        {"scrollbar_bg", "ui_art/scrollbar.pcx", 1, 18, 200, "Scrollbar background"},
        {"scrollbar_arrow", "ui_art/sb_arrow.pcx", 2, 18, 18, "Scrollbar arrows (up/down)"},
        {"scrollbar_thumb", "ui_art/sb_thumb.pcx", 1, 14, 20, "Scrollbar thumb"},
        {"progress_bg", "ui_art/prog_bg.pcx", 1, 227, 29, "Progress bar background"},
        {"progress_fill", "ui_art/prog_fil.pcx", 1, 227, 29, "Progress bar fill"},
        {"hero_portraits", "ui_art/heros.pcx", 6, 72, 96, "Hero class portraits"},
        {"panel_left", "ui_art/lpanel.pcx", 1, 320, 352, "Left panel background"},
        {"panel_right", "ui_art/rpanel.pcx", 1, 320, 352, "Right panel background"},
        {"spellbook", "ui_art/spellbk.pcx", 1, 320, 352, "Spellbook panel"},
        {"spellicons", "ui_art/spelli2.pcx", 52, 56, 56, "Spell icons"},
    };

    for (const auto& s : uiSprites) {
        writer.StartObject();
        writer.WriteKeyValue("name", s.name);
        writer.WriteKeyValue("file", s.file);
        writer.WriteKeyValue("frames", s.frames);
        writer.WriteKeyValue("width", s.width);
        writer.WriteKeyValue("height", s.height);
        writer.WriteKeyValue("usage", s.usage);
        writer.WriteKeyValue("transparent_index", 250);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 5. 英雄职业数据 ==========
    writer.WriteKey("hero_classes");
    writer.StartArray();

    struct HeroClass {
        const char* name;
        int str, mag, dex, vit;
        int max_str, max_mag, max_dex, max_vit;
        int base_life, base_mana;
    };
    HeroClass heroes[] = {
        {"Warrior", 30, 10, 20, 25, 250, 50, 60, 100, 70, 10},
        {"Rogue", 20, 15, 30, 20, 55, 70, 250, 80, 45, 22},
        {"Sorcerer", 15, 35, 15, 20, 45, 250, 85, 80, 30, 70},
        {"Monk", 25, 15, 25, 20, 150, 80, 150, 80, 60, 22},
        {"Bard", 20, 20, 25, 20, 120, 120, 120, 100, 55, 35},
        {"Barbarian", 40, 0, 20, 25, 255, 0, 55, 150, 70, 0},
    };

    for (const auto& h : heroes) {
        writer.StartObject();
        writer.WriteKeyValue("name", h.name);
        writer.WriteKeyValue("base_str", h.str);
        writer.WriteKeyValue("base_mag", h.mag);
        writer.WriteKeyValue("base_dex", h.dex);
        writer.WriteKeyValue("base_vit", h.vit);
        writer.WriteKeyValue("max_str", h.max_str);
        writer.WriteKeyValue("max_mag", h.max_mag);
        writer.WriteKeyValue("max_dex", h.max_dex);
        writer.WriteKeyValue("max_vit", h.max_vit);
        writer.WriteKeyValue("base_life", h.base_life);
        writer.WriteKeyValue("base_mana", h.base_mana);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 6. 法术数据 ==========
    writer.WriteKey("spells");
    writer.StartArray();

    struct SpellInfo {
        const char* name;
        int id;
        const char* type;
        int min_level;
        int base_mana;
    };
    SpellInfo spells[] = {
        {"Firebolt", 1, "Fire", 1, 6},
        {"Healing", 2, "Magic", 1, 5},
        {"Lightning", 3, "Lightning", 4, 10},
        {"Flash", 4, "Lightning", 5, 16},
        {"Identify", 5, "Magic", 1, 13},
        {"Fire Wall", 6, "Fire", 3, 28},
        {"Town Portal", 7, "Magic", 3, 35},
        {"Stone Curse", 8, "Magic", 6, 60},
        {"Infravision", 9, "Magic", 1, 40},
        {"Phasing", 10, "Magic", 7, 12},
        {"Mana Shield", 11, "Magic", 6, 33},
        {"Fireball", 12, "Fire", 8, 16},
        {"Guardian", 13, "Fire", 9, 50},
        {"Chain Lightning", 14, "Lightning", 8, 30},
        {"Flame Wave", 15, "Fire", 9, 35},
        {"Nova", 17, "Magic", 14, 60},
        {"Inferno", 20, "Fire", 3, 11},
        {"Golem", 21, "Magic", 8, 100},
        {"Teleport", 23, "Magic", 14, 35},
        {"Apocalypse", 24, "Fire", 19, 150},
        {"Elemental", 29, "Fire", 8, 35},
        {"Charged Bolt", 30, "Lightning", 1, 6},
        {"Holy Bolt", 31, "Magic", 1, 7},
        {"Resurrect", 32, "Magic", 1, 20},
        {"Telekinesis", 33, "Magic", 2, 15},
        {"Blood Star", 35, "Magic", 14, 25},
        {"Bone Spirit", 36, "Magic", 9, 24},
    };

    for (const auto& s : spells) {
        writer.StartObject();
        writer.WriteKeyValue("name", s.name);
        writer.WriteKeyValue("id", s.id);
        writer.WriteKeyValue("magic_type", s.type);
        writer.WriteKeyValue("min_level", s.min_level);
        writer.WriteKeyValue("base_mana", s.base_mana);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 7. 地牢层级数据 ==========
    writer.WriteKey("dungeon_levels");
    writer.StartArray();

    struct DungeonLevel {
        const char* name;
        int level;
        int min_mlvl;
        int max_mlvl;
        const char* tileset;
    };
    DungeonLevel levels[] = {
        {"Cathedral 1", 1, 1, 2, "l1"},
        {"Cathedral 2", 2, 2, 4, "l1"},
        {"Cathedral 3", 3, 3, 5, "l1"},
        {"Cathedral 4", 4, 4, 6, "l1"},
        {"Catacombs 1", 5, 5, 8, "l2"},
        {"Catacombs 2", 6, 6, 9, "l2"},
        {"Catacombs 3", 7, 7, 10, "l2"},
        {"Catacombs 4", 8, 8, 11, "l2"},
        {"Caves 1", 9, 9, 12, "l3"},
        {"Caves 2", 10, 10, 13, "l3"},
        {"Caves 3", 11, 11, 14, "l3"},
        {"Caves 4", 12, 12, 15, "l3"},
        {"Hell 1", 13, 13, 16, "l4"},
        {"Hell 2", 14, 14, 17, "l4"},
        {"Hell 3", 15, 15, 18, "l4"},
        {"Hell 4", 16, 16, 30, "l4"},
    };

    for (const auto& l : levels) {
        writer.StartObject();
        writer.WriteKeyValue("name", l.name);
        writer.WriteKeyValue("level", l.level);
        writer.WriteKeyValue("min_monster_level", l.min_mlvl);
        writer.WriteKeyValue("max_monster_level", l.max_mlvl);
        writer.WriteKeyValue("tileset", l.tileset);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 8. 物品类型数据 ==========
    writer.WriteKey("item_types");
    writer.StartArray();

    struct ItemType {
        const char* name;
        int class_id;
        const char* equip_loc;
    };
    ItemType itemTypes[] = {
        {"Gold", 4, "none"},
        {"Sword", 1, "one_hand"},
        {"Axe", 1, "one_hand"},
        {"Bow", 1, "two_hand"},
        {"Staff", 1, "two_hand"},
        {"Mace", 1, "one_hand"},
        {"Shield", 2, "one_hand"},
        {"Helm", 2, "head"},
        {"Light Armor", 2, "torso"},
        {"Medium Armor", 2, "torso"},
        {"Heavy Armor", 2, "torso"},
        {"Ring", 3, "ring"},
        {"Amulet", 3, "amulet"},
        {"Potion", 3, "belt"},
        {"Scroll", 3, "none"},
        {"Book", 3, "none"},
        {"Quest", 5, "none"},
    };

    for (const auto& t : itemTypes) {
        writer.StartObject();
        writer.WriteKeyValue("name", t.name);
        writer.WriteKeyValue("class_id", t.class_id);
        writer.WriteKeyValue("equip_location", t.equip_loc);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 9. 游戏常量 ==========
    writer.WriteKey("game_constants");
    writer.StartObject();

    writer.WriteKeyValue("TILE_WIDTH", 64);
    writer.WriteKeyValue("TILE_HEIGHT", 32);
    writer.WriteKeyValue("MAX_DUNGEON_SIZE", 112);
    writer.WriteKeyValue("MAX_PLRS", 4);
    writer.WriteKeyValue("MAX_MONSTERS", 200);
    writer.WriteKeyValue("MAX_ITEMS", 127);
    writer.WriteKeyValue("MAX_OBJECTS", 127);
    writer.WriteKeyValue("MAX_MISSILES", 125);
    writer.WriteKeyValue("MAXQUESTS", 24);
    writer.WriteKeyValue("MAX_SPELLS", 52);
    writer.WriteKeyValue("MAXBELTITEMS", 8);
    writer.WriteKeyValue("MAXLIGHTS", 32);
    writer.WriteKeyValue("MAXVISION", 32);
    writer.WriteKeyValue("INVENTORY_WIDTH", 10);
    writer.WriteKeyValue("INVENTORY_HEIGHT", 4);
    writer.WriteKeyValue("GOLD_SMALL_LIMIT", 1000);
    writer.WriteKeyValue("GOLD_MEDIUM_LIMIT", 2500);
    writer.WriteKeyValue("GOLD_MAX_LIMIT", 5000);
    writer.WriteKeyValue("MAX_PLAYER_LEVEL", 50);
    writer.WriteKeyValue("NIGHTMARE_LEVEL_BONUS", 16);
    writer.WriteKeyValue("HELL_LEVEL_BONUS", 32);

    writer.EndObject();

    // ========== 10. MPQ 档案结构 ==========
    writer.WriteKey("mpq_archives");
    writer.StartArray();

    struct MPQArchive {
        const char* name;
        const char* description;
        bool required;
    };
    MPQArchive archives[] = {
        {"diabdat.mpq", "Main Diablo game data", true},
        {"hellfire.mpq", "Hellfire expansion data", false},
        {"hfmonk.mpq", "Hellfire monk class", false},
        {"hfmusic.mpq", "Hellfire music", false},
        {"hfvoice.mpq", "Hellfire voice", false},
        {"spawn.mpq", "Shareware content", false},
        {"devilutionx.mpq", "DevilutionX assets", true},
        {"fonts.mpq", "Font assets", true},
    };

    for (const auto& a : archives) {
        writer.StartObject();
        writer.WriteKeyValue("name", a.name);
        writer.WriteKeyValue("description", a.description);
        writer.WriteKeyValue("required", a.required);
        writer.EndObject();
    }
    writer.EndArray();

    writer.EndObject();

    std::cout << "Sprite and game data exported (comprehensive): " << outputPath << std::endl;
}

} // namespace devilution::rust_export
