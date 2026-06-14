//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! 文本渲染模块 - 与C++ text_render.cpp保持一致
//!
//! 支持从fonts.mpq和devilutionx.mpq加载CLX格式字体
//! 支持颜色转换表(TRN)和多语言字体
//!
//! C++ Reference: Source/engine/render/text_render.cpp

use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};
use crate::mpq::MpqArchive;
use std::collections::HashMap;
use std::sync::RwLock;

// =============================================================================
// Constants - 与C++保持一致
// =============================================================================

/// 零宽空格 - 用于换行机会
pub const ZWSP: char = '\u{200B}';

/// 字体像素大小 - 与C++ FontSizes对应
pub const FONT_SIZES: [i32; 6] = [12, 24, 30, 42, 46, 22];

/// 行高 - 与C++ LineHeights对应
pub const LINE_HEIGHTS: [i32; 6] = [12, 26, 38, 42, 50, 22];

/// 小字体高行高 (CJK/韩语)
pub const SMALL_FONT_TALL_LINE_HEIGHT: i32 = 16;

/// 基线偏移 - 与C++ BaseLineOffset对应
pub const BASE_LINE_OFFSET: [i32; 6] = [-3, -2, -3, -6, -7, 3];

// =============================================================================
// GameFontTables - 与C++枚举完全一致
// =============================================================================

/// 字体表索引 - 与C++ GameFontTables对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum GameFontTables {
    #[default]
    GameFont12 = 0,
    GameFont24 = 1,
    GameFont30 = 2,
    GameFont42 = 3,
    GameFont46 = 4,
    FontSizeDialog = 5,
}

impl GameFontTables {
    /// 获取字体像素大小
    pub fn pixel_size(&self) -> i32 {
        FONT_SIZES[*self as usize]
    }

    /// 获取行高
    pub fn line_height(&self) -> i32 {
        LINE_HEIGHTS[*self as usize]
    }

    /// 获取基线偏移
    pub fn baseline_offset(&self) -> i32 {
        BASE_LINE_OFFSET[*self as usize]
    }

    /// 从索引创建
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::GameFont12),
            1 => Some(Self::GameFont24),
            2 => Some(Self::GameFont30),
            3 => Some(Self::GameFont42),
            4 => Some(Self::GameFont46),
            5 => Some(Self::FontSizeDialog),
            _ => None,
        }
    }
}

// 保留旧名称作为别名
pub type GameFont = GameFontTables;

// =============================================================================
// text_color - 与C++枚举完全一致
// =============================================================================

/// 文本颜色 - 与C++ text_color枚举对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum TextColor {
    /// 金色UI文字
    ColorUiGold = 0,
    /// 银色UI文字
    ColorUiSilver = 1,
    /// 暗金色UI文字
    ColorUiGoldDark = 2,
    /// 暗银色UI文字
    ColorUiSilverDark = 3,
    /// 对话框白色 (主菜单)
    ColorDialogWhite = 4,
    /// 对话框红色
    ColorDialogRed = 5,
    /// 黄色
    ColorYellow = 6,
    /// 金色
    ColorGold = 7,
    /// 黑色
    ColorBlack = 8,
    /// 白色
    #[default]
    ColorWhite = 9,
    /// 白金色
    ColorWhitegold = 10,
    /// 红色
    ColorRed = 11,
    /// 蓝色
    ColorBlue = 12,
    /// 橙色
    ColorOrange = 13,
    /// 按钮表面
    ColorButtonface = 14,
    /// 按钮按下
    ColorButtonpushed = 15,
    /// 游戏对话框白色
    ColorInGameDialogWhite = 16,
    /// 游戏对话框黄色
    ColorInGameDialogYellow = 17,
    /// 游戏对话框红色
    ColorInGameDialogRed = 18,
}

/// 颜色转换表TRN文件路径 - 与C++ ColorTranslations对应
pub const COLOR_TRANSLATIONS: [Option<&'static str>; 19] = [
    Some("fonts\\goldui.trn"),        // ColorUiGold
    Some("fonts\\grayui.trn"),        // ColorUiSilver
    Some("fonts\\golduis.trn"),       // ColorUiGoldDark
    Some("fonts\\grayuis.trn"),       // ColorUiSilverDark
    None,                             // ColorDialogWhite
    None,                             // ColorDialogRed
    Some("fonts\\yellow.trn"),        // ColorYellow
    None,                             // ColorGold (无TRN)
    Some("fonts\\black.trn"),         // ColorBlack
    Some("fonts\\white.trn"),         // ColorWhite
    Some("fonts\\whitegold.trn"),     // ColorWhitegold
    Some("fonts\\red.trn"),           // ColorRed
    Some("fonts\\blue.trn"),          // ColorBlue
    Some("fonts\\orange.trn"),        // ColorOrange
    Some("fonts\\buttonface.trn"),    // ColorButtonface
    Some("fonts\\buttonpushed.trn"),  // ColorButtonpushed
    Some("fonts\\gamedialogwhite.trn"),  // ColorInGameDialogWhite
    Some("fonts\\gamedialogyellow.trn"), // ColorInGameDialogYellow
    Some("fonts\\gamedialogred.trn"),    // ColorInGameDialogRed
];

impl TextColor {
    /// 获取TRN文件路径
    pub fn trn_path(&self) -> Option<&'static str> {
        COLOR_TRANSLATIONS[*self as usize]
    }

    /// 从索引创建
    pub fn from_index(index: u8) -> Option<Self> {
        if index <= 18 {
            // Safety: index is checked to be in valid range
            Some(unsafe { std::mem::transmute(index) })
        } else {
            None
        }
    }
}

// =============================================================================
// UiFlags - 与C++ UiFlags保持一致
// =============================================================================

bitflags::bitflags! {
    /// UI渲染标志 - 与C++ UiFlags完全对应
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct UiFlags: u32 {
        const NONE = 0;

        // 字体大小
        const FONT_SIZE_12 = 1 << 0;
        const FONT_SIZE_24 = 1 << 1;
        const FONT_SIZE_30 = 1 << 2;
        const FONT_SIZE_42 = 1 << 3;
        const FONT_SIZE_46 = 1 << 4;
        const FONT_SIZE_DIALOG = 1 << 5;

        // 颜色
        const COLOR_UI_GOLD = 1 << 6;
        const COLOR_UI_SILVER = 1 << 7;
        const COLOR_UI_GOLD_DARK = 1 << 8;
        const COLOR_UI_SILVER_DARK = 1 << 9;
        const COLOR_DIALOG_WHITE = 1 << 10;
        const COLOR_DIALOG_YELLOW = 1 << 11;
        const COLOR_DIALOG_RED = 1 << 12;
        const COLOR_YELLOW = 1 << 13;
        const COLOR_GOLD = 1 << 14;
        const COLOR_BLACK = 1 << 15;
        const COLOR_WHITE = 1 << 16;
        const COLOR_WHITEGOLD = 1 << 17;
        const COLOR_RED = 1 << 18;
        const COLOR_BLUE = 1 << 19;
        const COLOR_ORANGE = 1 << 20;
        const COLOR_BUTTONFACE = 1 << 21;
        const COLOR_BUTTONPUSHED = 1 << 22;

        // 对齐
        const ALIGN_CENTER = 1 << 23;
        const ALIGN_RIGHT = 1 << 24;
        const VERTICAL_CENTER = 1 << 25;

        // 其他
        const KERNING_FIT_SPACING = 1 << 26;
        const ELEMENT_DISABLED = 1 << 27;
        const ELEMENT_HIDDEN = 1 << 28;
        const PENTA_CURSOR = 1 << 29;
        const OUTLINED = 1 << 30;
        const NEEDS_NEXT_ELEMENT = 1 << 31;
    }
}

/// 从UiFlags获取字体大小 - 与C++ GetFontSizeFromUiFlags对应
pub fn get_font_size_from_ui_flags(flags: UiFlags) -> GameFontTables {
    if flags.contains(UiFlags::FONT_SIZE_24) {
        GameFontTables::GameFont24
    } else if flags.contains(UiFlags::FONT_SIZE_30) {
        GameFontTables::GameFont30
    } else if flags.contains(UiFlags::FONT_SIZE_42) {
        GameFontTables::GameFont42
    } else if flags.contains(UiFlags::FONT_SIZE_46) {
        GameFontTables::GameFont46
    } else if flags.contains(UiFlags::FONT_SIZE_DIALOG) {
        GameFontTables::FontSizeDialog
    } else {
        GameFontTables::GameFont12
    }
}

/// 从UiFlags获取颜色 - 与C++ GetColorFromFlags对应
pub fn get_color_from_flags(flags: UiFlags, in_game: bool) -> TextColor {
    if flags.contains(UiFlags::COLOR_WHITE) {
        TextColor::ColorWhite
    } else if flags.contains(UiFlags::COLOR_BLUE) {
        TextColor::ColorBlue
    } else if flags.contains(UiFlags::COLOR_ORANGE) {
        TextColor::ColorOrange
    } else if flags.contains(UiFlags::COLOR_RED) {
        TextColor::ColorRed
    } else if flags.contains(UiFlags::COLOR_BLACK) {
        TextColor::ColorBlack
    } else if flags.contains(UiFlags::COLOR_GOLD) {
        TextColor::ColorGold
    } else if flags.contains(UiFlags::COLOR_UI_GOLD) {
        TextColor::ColorUiGold
    } else if flags.contains(UiFlags::COLOR_UI_SILVER) {
        TextColor::ColorUiSilver
    } else if flags.contains(UiFlags::COLOR_UI_GOLD_DARK) {
        TextColor::ColorUiGoldDark
    } else if flags.contains(UiFlags::COLOR_UI_SILVER_DARK) {
        TextColor::ColorUiSilverDark
    } else if flags.contains(UiFlags::COLOR_DIALOG_WHITE) {
        if in_game {
            TextColor::ColorInGameDialogWhite
        } else {
            TextColor::ColorDialogWhite
        }
    } else if flags.contains(UiFlags::COLOR_DIALOG_YELLOW) {
        TextColor::ColorInGameDialogYellow
    } else if flags.contains(UiFlags::COLOR_DIALOG_RED) {
        TextColor::ColorInGameDialogRed
    } else if flags.contains(UiFlags::COLOR_YELLOW) {
        TextColor::ColorYellow
    } else if flags.contains(UiFlags::COLOR_BUTTONFACE) {
        TextColor::ColorButtonface
    } else if flags.contains(UiFlags::COLOR_BUTTONPUSHED) {
        TextColor::ColorButtonpushed
    } else {
        TextColor::ColorWhitegold
    }
}

// =============================================================================
// ColorTranslation - TRN颜色转换表
// =============================================================================

/// 颜色转换表 (Translation Table) - 256字节索引映射
#[derive(Debug, Clone)]
pub struct ColorTranslation {
    /// 256字节的颜色索引映射表
    pub data: [u8; 256],
}

impl ColorTranslation {
    /// 从原始数据加载
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 256 {
            return None;
        }
        let mut result = [0u8; 256];
        result.copy_from_slice(&data[..256]);
        Some(Self { data: result })
    }

    /// 转换颜色索引
    #[inline]
    pub fn translate(&self, color: u8) -> u8 {
        self.data[color as usize]
    }
}

// =============================================================================
// Global Font Cache - 与C++全局Fonts缓存对应
// =============================================================================

use std::sync::LazyLock;

/// 字体缓存 - fontId -> (baseFont, overrideFont)
/// fontId = (size << 16) | unicodeRow
static FONTS: LazyLock<RwLock<HashMap<u32, FontStack>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// 颜色转换表缓存
static COLOR_TRANSLATIONS_DATA: LazyLock<RwLock<[Option<[u8; 256]>; 19]>> = 
    LazyLock::new(|| RwLock::new([None; 19]));

/// 字体栈 - 支持基础字体和覆盖字体
#[derive(Clone)]
pub struct FontStack {
    /// 基础字体
    pub base_font: Option<ClxSpriteList>,
    /// 覆盖字体 (语言特定)
    pub override_font: Option<ClxSpriteList>,
}

impl FontStack {
    /// 检查是否有值
    pub fn has_value(&self) -> bool {
        self.base_font.is_some() || self.override_font.is_some()
    }

    /// 获取字形
    pub fn glyph(&self, index: usize) -> Option<ClxSprite> {
        // 优先使用覆盖字体
        if let Some(ref font) = self.override_font {
            if let Some(sprite) = font.get(index) {
                if sprite.width() != 0 {
                    return Some(sprite);
                }
            }
        }
        // 回退到基础字体
        self.base_font.as_ref().and_then(|f| f.get(index))
    }
}

/// 获取Unicode行号
#[inline]
pub fn get_unicode_row(codepoint: char) -> u16 {
    (codepoint as u32 >> 8) as u16
}

/// 获取字体ID
#[inline]
pub fn get_font_id(size: GameFontTables, row: u16) -> u32 {
    ((size as u32) << 16) | (row as u32)
}

/// 检查是否是CJK字符行
pub fn is_cjk(row: u16) -> bool {
    row >= 0x30 && row <= 0x9f
}

/// 检查是否是韩语字符行
pub fn is_hangul(row: u16) -> bool {
    row >= 0xac && row <= 0xd7
}

/// 检查是否是小字体高行 (CJK或韩语)
pub fn is_small_font_tall_row(row: u16) -> bool {
    is_cjk(row) || is_hangul(row)
}

/// 是否使用高行高的小字体
/// TODO: 从语言设置获取
pub fn is_small_font_tall() -> bool {
    false // 默认不使用高行高
}

/// 卸载所有字体 - 与C++ UnloadFonts对应
pub fn unload_fonts() {
    if let Ok(mut fonts) = FONTS.write() {
        fonts.clear();
    }
}

// =============================================================================
// GetLineWidth - 计算行宽度
// =============================================================================

/// 计算文本行宽度 - 与C++ GetLineWidth对应
/// 
/// # Arguments
/// * `text` - 要计算的文本
/// * `size` - 字体大小
/// * `spacing` - 字符间距
/// * `characters_in_line` - 输出行中的字符数
pub fn get_line_width(
    text: &str,
    size: GameFontTables,
    spacing: i32,
    characters_in_line: Option<&mut i32>,
) -> i32 {
    let mut line_width = 0i32;
    let mut codepoints = 0i32;

    for ch in text.chars() {
        if ch == ZWSP {
            continue;
        }
        if ch == '\n' {
            break;
        }

        // 尝试获取字形宽度
        let frame = (ch as u32 & 0xFF) as usize;
        let row = get_unicode_row(ch);
        let font_id = get_font_id(size, row);
        
        let width = if let Ok(fonts) = FONTS.read() {
            fonts.get(&font_id)
                .and_then(|f| f.glyph(frame))
                .map(|g| g.width() as i32)
                .unwrap_or(0)
        } else {
            0
        };
        
        if width > 0 {
            line_width += width + spacing;
            codepoints += 1;
        }
    }

    if let Some(count) = characters_in_line {
        *count = codepoints;
    }

    if line_width > 0 {
        line_width - spacing
    } else {
        0
    }
}

// =============================================================================
// GetLineHeight - 计算行高度
// =============================================================================

/// 计算文本行高度 - 与C++ GetLineHeight对应
pub fn get_line_height(text: &str, font_index: GameFontTables) -> i32 {
    if font_index == GameFontTables::GameFont12 && is_small_font_tall() {
        // 检查是否包含高行字符
        for ch in text.chars() {
            if ch == ZWSP {
                continue;
            }
            let row = get_unicode_row(ch);
            if is_small_font_tall_row(row) {
                return SMALL_FONT_TALL_LINE_HEIGHT;
            }
        }
    }
    LINE_HEIGHTS[font_index as usize]
}

// =============================================================================
// IsBreakableWhitespace - 可换行空白字符检查
// =============================================================================

/// 检查字符是否是可换行的空白字符 - 与C++ IsBreakableWhitespace对应
pub fn is_breakable_whitespace(c: char) -> bool {
    matches!(c, ' ' | '　' | '\u{200B}')
}

/// 检查是否是全角标点
fn is_full_width_punct(c: char) -> bool {
    matches!(c, '，' | '、' | '。' | '？' | '！')
}

/// 检查是否允许在此处换行
fn is_break_allowed(codepoint: char, next_codepoint: char) -> bool {
    is_full_width_punct(codepoint) && !is_full_width_punct(next_codepoint)
}

// =============================================================================
// WordWrapString - 文本自动换行
// =============================================================================

/// 自动换行文本 - 与C++ WordWrapString对应
/// 
/// # Arguments
/// * `text` - 源文本
/// * `width` - 输出区域宽度(像素)
/// * `size` - 字体大小
/// * `spacing` - 字符间距
pub fn word_wrap_string(
    text: &str,
    width: u32,
    size: GameFontTables,
    spacing: i32,
) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut output = String::with_capacity(text.len());
    let mut line_width = 0u32;
    let mut last_breakable_pos: Option<usize> = None;
    let mut last_breakable_output_pos: Option<usize> = None;

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        let next_ch = chars.get(i + 1).copied().unwrap_or('\0');

        if ch == '\n' {
            output.push(ch);
            line_width = 0;
            last_breakable_pos = None;
            last_breakable_output_pos = None;
            i += 1;
            continue;
        }

        // 计算字形宽度
        let glyph_width = if ch != ZWSP {
            let frame = (ch as u32 & 0xFF) as usize;
            let row = get_unicode_row(ch);
            let font_id = get_font_id(size, row);
            
            if let Ok(fonts) = FONTS.read() {
                fonts.get(&font_id)
                    .and_then(|f| f.glyph(frame))
                    .map(|g| g.width() as u32)
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // 记录可换行位置
        if is_breakable_whitespace(ch) {
            last_breakable_pos = Some(i);
            last_breakable_output_pos = Some(output.len());
        }

        // 检查是否需要换行
        let char_width = glyph_width + spacing as u32;
        if line_width + glyph_width > width && line_width > 0 {
            // 需要换行
            if let Some(break_output_pos) = last_breakable_output_pos {
                // 在上一个可换行位置换行
                output.truncate(break_output_pos);
                output.push('\n');
                // 重新从换行位置之后开始
                i = last_breakable_pos.unwrap() + 1;
                line_width = 0;
                last_breakable_pos = None;
                last_breakable_output_pos = None;
                continue;
            } else if is_break_allowed(ch, next_ch) {
                // 允许在当前位置换行
                output.push(ch);
                output.push('\n');
                i += 1;
                line_width = 0;
                continue;
            } else {
                // 强制换行
                output.push('\n');
                line_width = 0;
            }
        }

        output.push(ch);
        line_width += char_width;
        
        if is_break_allowed(ch, next_ch) {
            last_breakable_pos = Some(i + 1);
            last_breakable_output_pos = Some(output.len());
        }

        i += 1;
    }

    output
}

// =============================================================================
// PentSpn2Spin - 旋转动画帧
// =============================================================================

use crate::engine::ticks::get_animation_frame;

/// 获取PentSpn2动画帧 - 与C++ PentSpn2Spin对应
pub fn pent_spn2_spin() -> u8 {
    get_animation_frame(8, 50) as u8
}

// =============================================================================
// TextRenderOptions - 文本渲染选项
// =============================================================================

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// 高亮范围
#[derive(Debug, Clone, Copy, Default)]
pub struct HighlightRange {
    pub begin: i32,
    pub end: i32,
}

/// 文本渲染选项 - 与C++ TextRenderOptions对应
#[derive(Debug, Clone)]
pub struct TextRenderOptions {
    /// UI标志
    pub flags: UiFlags,
    /// 字符间距
    pub spacing: i32,
    /// 行高 (-1表示使用默认)
    pub line_height: i32,
    /// 光标位置 (-1表示不显示)
    pub cursor_position: i32,
    /// 高亮范围
    pub highlight_range: HighlightRange,
    /// 高亮颜色
    pub highlight_color: u8,
    /// 渲染的光标位置输出
    pub rendered_cursor_position: Option<(i32, i32)>,
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            flags: UiFlags::NONE,
            spacing: 1,
            line_height: -1,
            cursor_position: -1,
            highlight_range: HighlightRange::default(),
            highlight_color: 0, // PAL8_RED + 6
            rendered_cursor_position: None,
        }
    }
}

// =============================================================================
// 保留旧的TextRenderer实现以保持兼容性
// =============================================================================

/// 字体字形缓存 - 按Unicode行缓存
struct FontGlyphs {
    /// Unicode行 -> ClxSpriteList
    rows: HashMap<u16, ClxSpriteList>,
}

impl FontGlyphs {
    fn new() -> Self {
        Self {
            rows: HashMap::new(),
        }
    }

    /// 获取字形
    fn get_glyph(&self, codepoint: char) -> Option<ClxSprite> {
        let code = codepoint as u32;
        let row = (code >> 8) as u16;
        let idx = (code & 0xFF) as usize;
        self.rows.get(&row).and_then(|list| list.get(idx))
    }

    /// 检查是否有该行
    fn has_row(&self, row: u16) -> bool {
        self.rows.contains_key(&row)
    }

    /// 添加一行字形
    fn add_row(&mut self, row: u16, list: ClxSpriteList) {
        self.rows.insert(row, list);
    }
}

/// CLX字体渲染器 - 实例版本
pub struct TextRenderer {
    /// 各字号的字形缓存
    fonts: HashMap<GameFontTables, FontGlyphs>,
    /// 颜色转换表缓存
    color_translations: HashMap<TextColor, ColorTranslation>,
    /// 默认调色板 (用于渲染)
    palette: [u8; 768],
}

impl TextRenderer {
    /// 创建新的文本渲染器
    pub fn new() -> Self {
        // 创建默认的灰度调色板
        let mut palette = [0u8; 768];
        for i in 0..256 {
            palette[i * 3] = i as u8;
            palette[i * 3 + 1] = i as u8;
            palette[i * 3 + 2] = i as u8;
        }

        Self {
            fonts: HashMap::new(),
            color_translations: HashMap::new(),
            palette,
        }
    }

    /// 设置调色板
    pub fn set_palette(&mut self, palette: &[u8; 768]) {
        self.palette.copy_from_slice(palette);
    }

    /// 从MPQ加载字体
    pub fn load_font(&mut self, archive: &mut MpqArchive, font: GameFontTables, rows: &[u16]) -> bool {
        let mut glyphs = FontGlyphs::new();
        let size = font.pixel_size();

        for &row in rows {
            let path = format!("fonts\\{}-{:02x}.clx", size, row);
            if archive.has_file(&path) {
                if let Ok(data) = archive.read_file(&path) {
                    let list = ClxSpriteList::from_data(data);
                    glyphs.add_row(row, list);
                }
            }
        }

        if glyphs.rows.is_empty() {
            return false;
        }

        self.fonts.insert(font, glyphs);
        true
    }

    /// 尝试动态加载Unicode行
    pub fn load_font_row(
        &mut self,
        archives: &mut [&mut MpqArchive],
        font: GameFontTables,
        row: u16,
    ) -> bool {
        if let Some(glyphs) = self.fonts.get(&font) {
            if glyphs.has_row(row) {
                return true;
            }
        }

        let size = font.pixel_size();
        let path = format!("fonts\\{}-{:02x}.clx", size, row);

        for archive in archives.iter_mut() {
            if archive.has_file(&path) {
                if let Ok(data) = archive.read_file(&path) {
                    let list = ClxSpriteList::from_data(data);
                    let glyphs = self.fonts.entry(font).or_insert_with(FontGlyphs::new);
                    glyphs.add_row(row, list);
                    return true;
                }
            }
        }

        false
    }

    /// 加载颜色转换表
    pub fn load_color_translation(
        &mut self,
        archive: &mut MpqArchive,
        color: TextColor,
    ) -> bool {
        if let Some(path) = color.trn_path() {
            if let Ok(data) = archive.read_file(path) {
                if let Some(trn) = ColorTranslation::from_bytes(&data) {
                    self.color_translations.insert(color, trn);
                    return true;
                }
            }
        }
        false
    }

    /// 获取字形
    fn get_glyph(&self, font: GameFontTables, ch: char) -> Option<ClxSprite> {
        self.fonts.get(&font).and_then(|g| g.get_glyph(ch))
    }

    /// 计算文本宽度
    pub fn get_text_width(&self, text: &str, font: GameFontTables, spacing: i32) -> i32 {
        let mut width = 0i32;
        let mut char_count = 0;

        for ch in text.chars() {
            if ch == '\n' {
                break;
            }
            if let Some(glyph) = self.get_glyph(font, ch) {
                width += glyph.width() as i32;
                char_count += 1;
            }
        }

        if char_count > 1 {
            width += spacing * (char_count - 1);
        }

        width
    }

    /// 渲染文本到RGBA缓冲区
    pub fn render_text(
        &self,
        target: &mut [u8],
        target_width: usize,
        target_height: usize,
        text: &str,
        x: i32,
        y: i32,
        font: GameFontTables,
        color: Option<TextColor>,
        spacing: i32,
    ) {
        let trn = color.and_then(|c| self.color_translations.get(&c));
        let mut cur_x = x;
        let mut cur_y = y + font.baseline_offset();

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = x;
                cur_y += font.line_height();
                continue;
            }

            if let Some(glyph) = self.get_glyph(font, ch) {
                self.render_glyph(
                    target,
                    target_width,
                    target_height,
                    &glyph,
                    cur_x,
                    cur_y,
                    trn,
                );
                cur_x += glyph.width() as i32 + spacing;
            }
        }
    }

    /// 渲染居中文本
    pub fn render_text_centered(
        &self,
        target: &mut [u8],
        target_width: usize,
        target_height: usize,
        text: &str,
        center_x: i32,
        y: i32,
        font: GameFontTables,
        color: Option<TextColor>,
        spacing: i32,
    ) {
        let width = self.get_text_width(text, font, spacing);
        let x = center_x - width / 2;
        self.render_text(target, target_width, target_height, text, x, y, font, color, spacing);
    }

    /// 渲染单个字形
    fn render_glyph(
        &self,
        target: &mut [u8],
        target_width: usize,
        target_height: usize,
        glyph: &ClxSprite,
        x: i32,
        y: i32,
        trn: Option<&ColorTranslation>,
    ) {
        let mut src_idx = 0;
        let mut dst_x = x;
        let mut dst_y = y + glyph.height() as i32 - 1;
        let sprite_width = glyph.width() as i32;
        let pixel_data = glyph.pixel_data();

        while src_idx < pixel_data.len() && dst_y >= y {
            let control = pixel_data[src_idx];
            src_idx += 1;

            if control < 0x80 {
                dst_x += control as i32;
            } else if control <= 0xBE {
                let width = (0xBF - control) as i32;
                if src_idx >= pixel_data.len() {
                    break;
                }
                let mut color_idx = pixel_data[src_idx];
                src_idx += 1;

                if let Some(t) = trn {
                    color_idx = t.translate(color_idx);
                }

                for _ in 0..width {
                    self.put_pixel(target, target_width, target_height, dst_x, dst_y, color_idx);
                    dst_x += 1;
                }
            } else {
                let width = (256 - control as i32) as i32;
                for _ in 0..width {
                    if src_idx >= pixel_data.len() {
                        break;
                    }
                    let mut color_idx = pixel_data[src_idx];
                    src_idx += 1;

                    if let Some(t) = trn {
                        color_idx = t.translate(color_idx);
                    }

                    self.put_pixel(target, target_width, target_height, dst_x, dst_y, color_idx);
                    dst_x += 1;
                }
            }

            while dst_x >= x + sprite_width {
                dst_x -= sprite_width;
                dst_y -= 1;
            }
        }
    }

    /// 绘制单个像素
    #[inline]
    fn put_pixel(
        &self,
        target: &mut [u8],
        target_width: usize,
        target_height: usize,
        x: i32,
        y: i32,
        color_idx: u8,
    ) {
        if x < 0 || y < 0 || x >= target_width as i32 || y >= target_height as i32 {
            return;
        }

        if color_idx == 0 {
            return;
        }

        let idx = (y as usize * target_width + x as usize) * 4;
        if idx + 3 < target.len() {
            let ci = color_idx as usize * 3;
            target[idx] = self.palette[ci];
            target[idx + 1] = self.palette[ci + 1];
            target[idx + 2] = self.palette[ci + 2];
            target[idx + 3] = 255;
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_font_tables() {
        assert_eq!(GameFontTables::GameFont12.pixel_size(), 12);
        assert_eq!(GameFontTables::GameFont24.pixel_size(), 24);
        assert_eq!(GameFontTables::GameFont42.pixel_size(), 42);
        assert_eq!(GameFontTables::GameFont46.pixel_size(), 46);
        assert_eq!(GameFontTables::FontSizeDialog.pixel_size(), 22);
    }

    #[test]
    fn test_line_heights() {
        assert_eq!(GameFontTables::GameFont12.line_height(), 12);
        assert_eq!(GameFontTables::GameFont24.line_height(), 26);
        assert_eq!(GameFontTables::GameFont46.line_height(), 50);
    }

    #[test]
    fn test_color_translation() {
        let mut data = [0u8; 256];
        for i in 0..256 {
            data[i] = (255 - i) as u8;
        }
        let trn = ColorTranslation::from_bytes(&data).unwrap();
        assert_eq!(trn.translate(0), 255);
        assert_eq!(trn.translate(255), 0);
    }

    #[test]
    fn test_ui_flags() {
        let flags = UiFlags::FONT_SIZE_24 | UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
        assert_eq!(get_font_size_from_ui_flags(flags), GameFontTables::GameFont24);
        assert_eq!(get_color_from_flags(flags, false), TextColor::ColorWhite);
    }

    #[test]
    fn test_unicode_row() {
        assert_eq!(get_unicode_row('A'), 0x00);
        assert_eq!(get_unicode_row('中'), 0x4E);
        assert!(is_cjk(0x4E));
        assert!(!is_hangul(0x4E));
    }

    #[test]
    fn test_breakable_whitespace() {
        assert!(is_breakable_whitespace(' '));
        assert!(is_breakable_whitespace('　'));
        assert!(is_breakable_whitespace(ZWSP));
        assert!(!is_breakable_whitespace('a'));
    }
}
