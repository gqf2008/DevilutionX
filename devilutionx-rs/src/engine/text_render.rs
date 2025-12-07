//! 文本渲染模块 - 与C++版本保持一致
//!
//! 支持从fonts.mpq和devilutionx.mpq加载CLX格式字体
//! 支持颜色转换表(TRN)和多语言字体

use super::clx::{ClxSprite, ClxSpriteList};
use super::mpq::MpqArchive;
use std::collections::HashMap;

/// 字体大小枚举 - 与C++的GameFontTables对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameFont {
    /// 12像素小字体
    Small = 12,
    /// 22像素字体
    Font22 = 22,
    /// 24像素字体
    Font24 = 24,
    /// 30像素字体
    Font30 = 30,
    /// 42像素字体
    Font42 = 42,
    /// 46像素大字体
    Large = 46,
}

impl GameFont {
    /// 获取字体像素大小
    pub fn size(&self) -> u32 {
        *self as u32
    }

    /// 获取行高
    pub fn line_height(&self) -> i32 {
        match self {
            GameFont::Small => 12,
            GameFont::Font22 => 22,
            GameFont::Font24 => 26,
            GameFont::Font30 => 38,
            GameFont::Font42 => 42,
            GameFont::Large => 50,
        }
    }

    /// 获取基线偏移
    pub fn baseline_offset(&self) -> i32 {
        match self {
            GameFont::Small => -3,
            GameFont::Font22 => 3,
            GameFont::Font24 => -2,
            GameFont::Font30 => -3,
            GameFont::Font42 => -6,
            GameFont::Large => -7,
        }
    }
}

/// 文本颜色 - 与C++的text_color对应
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextColor {
    /// 金色UI文字
    UiGold,
    /// 银色UI文字
    UiSilver,
    /// 暗金色UI文字
    UiGoldDark,
    /// 暗银色UI文字
    UiSilverDark,
    /// 对话框白色
    DialogWhite,
    /// 对话框红色
    DialogRed,
    /// 黄色
    Yellow,
    /// 黑色
    Black,
    /// 白色
    White,
    /// 白金色
    WhiteGold,
    /// 红色
    Red,
    /// 蓝色
    Blue,
    /// 橙色
    Orange,
    /// 按钮表面
    ButtonFace,
    /// 按钮按下
    ButtonPushed,
    /// 游戏对话框白色
    GameDialogWhite,
    /// 游戏对话框黄色
    GameDialogYellow,
    /// 游戏对话框红色
    GameDialogRed,
}

impl TextColor {
    /// 获取TRN文件路径
    pub fn trn_path(&self) -> Option<&'static str> {
        match self {
            TextColor::UiGold => Some("fonts\\goldui.trn"),
            TextColor::UiSilver => Some("fonts\\grayui.trn"),
            TextColor::UiGoldDark => Some("fonts\\golduis.trn"),
            TextColor::UiSilverDark => Some("fonts\\grayuis.trn"),
            TextColor::DialogWhite => None,
            TextColor::DialogRed => None,
            TextColor::Yellow => Some("fonts\\yellow.trn"),
            TextColor::Black => Some("fonts\\black.trn"),
            TextColor::White => Some("fonts\\white.trn"),
            TextColor::WhiteGold => Some("fonts\\whitegold.trn"),
            TextColor::Red => Some("fonts\\red.trn"),
            TextColor::Blue => Some("fonts\\blue.trn"),
            TextColor::Orange => Some("fonts\\orange.trn"),
            TextColor::ButtonFace => Some("fonts\\buttonface.trn"),
            TextColor::ButtonPushed => Some("fonts\\buttonpushed.trn"),
            TextColor::GameDialogWhite => Some("fonts\\gamedialogwhite.trn"),
            TextColor::GameDialogYellow => Some("fonts\\gamedialogyellow.trn"),
            TextColor::GameDialogRed => Some("fonts\\gamedialogred.trn"),
        }
    }
}

/// 颜色转换表 (Translation Table)
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
    fn get_glyph(&self, codepoint: char) -> Option<&ClxSprite> {
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

/// CLX字体渲染器 - 与C++版本保持一致
pub struct TextRenderer {
    /// 各字号的字形缓存
    fonts: HashMap<GameFont, FontGlyphs>,
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
    pub fn load_font(&mut self, archive: &mut MpqArchive, font: GameFont, rows: &[u16]) -> bool {
        let mut glyphs = FontGlyphs::new();
        let size = font.size();

        for &row in rows {
            let path = format!("fonts\\{}-{:02x}.clx", size, row);
            if archive.has_file(&path) {
                if let Ok(data) = archive.read_file(&path) {
                    if let Some(list) = ClxSpriteList::from_bytes(data) {
                        glyphs.add_row(row, list);
                    }
                }
            }
        }

        if glyphs.rows.is_empty() {
            return false;
        }

        self.fonts.insert(font, glyphs);
        true
    }

    /// 尝试动态加载Unicode行 (从多个MPQ源)
    pub fn load_font_row(
        &mut self,
        archives: &mut [&mut MpqArchive],
        font: GameFont,
        row: u16,
    ) -> bool {
        // 检查是否已加载
        if let Some(glyphs) = self.fonts.get(&font) {
            if glyphs.has_row(row) {
                return true;
            }
        }

        let size = font.size();
        let path = format!("fonts\\{}-{:02x}.clx", size, row);

        // 尝试从各MPQ加载
        for archive in archives.iter_mut() {
            if archive.has_file(&path) {
                if let Ok(data) = archive.read_file(&path) {
                    if let Some(list) = ClxSpriteList::from_bytes(data) {
                        let glyphs = self.fonts.entry(font).or_insert_with(FontGlyphs::new);
                        glyphs.add_row(row, list);
                        return true;
                    }
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
    fn get_glyph(&self, font: GameFont, ch: char) -> Option<&ClxSprite> {
        self.fonts.get(&font).and_then(|g| g.get_glyph(ch))
    }

    /// 计算文本宽度
    pub fn get_text_width(&self, text: &str, font: GameFont, spacing: i32) -> i32 {
        let mut width = 0i32;
        let mut char_count = 0;

        for ch in text.chars() {
            if ch == '\n' {
                break;
            }
            if let Some(glyph) = self.get_glyph(font, ch) {
                width += glyph.width as i32;
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
        font: GameFont,
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
                    glyph,
                    cur_x,
                    cur_y,
                    trn,
                );
                cur_x += glyph.width as i32 + spacing;
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
        font: GameFont,
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
        let mut dst_y = y + glyph.height as i32 - 1; // CL2渲染从下到上
        let sprite_width = glyph.width as i32;

        while src_idx < glyph.pixel_data.len() && dst_y >= y {
            let control = glyph.pixel_data[src_idx];
            src_idx += 1;

            if control < 0x80 {
                // 透明: 跳过N个像素
                dst_x += control as i32;
            } else if control <= 0xBE {
                // 填充: 用下一个字节填充N个像素
                let width = (0xBF - control) as i32;
                if src_idx >= glyph.pixel_data.len() {
                    break;
                }
                let mut color_idx = glyph.pixel_data[src_idx];
                src_idx += 1;

                // 应用颜色转换
                if let Some(t) = trn {
                    color_idx = t.translate(color_idx);
                }

                for _ in 0..width {
                    self.put_pixel(target, target_width, target_height, dst_x, dst_y, color_idx);
                    dst_x += 1;
                }
            } else {
                // 复制: 从流中复制N个像素
                let width = (256 - control as i32) as i32;
                for _ in 0..width {
                    if src_idx >= glyph.pixel_data.len() {
                        break;
                    }
                    let mut color_idx = glyph.pixel_data[src_idx];
                    src_idx += 1;

                    // 应用颜色转换
                    if let Some(t) = trn {
                        color_idx = t.translate(color_idx);
                    }

                    self.put_pixel(target, target_width, target_height, dst_x, dst_y, color_idx);
                    dst_x += 1;
                }
            }

            // 换行处理
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

        // 跳过透明色 (索引0)
        if color_idx == 0 {
            return;
        }

        let idx = (y as usize * target_width + x as usize) * 4;
        if idx + 3 < target.len() {
            let ci = color_idx as usize * 3;
            target[idx] = self.palette[ci];         // R
            target[idx + 1] = self.palette[ci + 1]; // G
            target[idx + 2] = self.palette[ci + 2]; // B
            target[idx + 3] = 255;                  // A
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// 文本渲染选项
#[derive(Debug, Clone)]
pub struct TextRenderOptions {
    pub font: GameFont,
    pub color: Option<TextColor>,
    pub align: TextAlign,
    pub spacing: i32,
    pub line_height: Option<i32>,
    pub outlined: bool,
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            font: GameFont::Font24,
            color: Some(TextColor::White),
            align: TextAlign::Left,
            spacing: 1,
            line_height: None,
            outlined: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_font_sizes() {
        assert_eq!(GameFont::Small.size(), 12);
        assert_eq!(GameFont::Font42.size(), 42);
        assert_eq!(GameFont::Large.size(), 46);
    }

    #[test]
    fn test_color_translation() {
        let mut data = [0u8; 256];
        for i in 0..256 {
            data[i] = (255 - i) as u8; // 反转
        }
        let trn = ColorTranslation::from_bytes(&data).unwrap();
        assert_eq!(trn.translate(0), 255);
        assert_eq!(trn.translate(255), 0);
    }
}
