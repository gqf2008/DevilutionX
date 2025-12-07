//! Palette - 调色板处理
//!
//! 移植自 Source/engine/palette.h
//!
//! Diablo 使用 256 色调色板:
//! - 索引 0-127 (0x00-0x7F): 关卡特定颜色
//! - 索引 128-255 (0x80-0xFF): 全局颜色

/// 8色蓝色起始索引
pub const PAL8_BLUE: u8 = 128;
/// 8色红色起始索引
pub const PAL8_RED: u8 = 136;
/// 8色黄色起始索引
pub const PAL8_YELLOW: u8 = 144;
/// 8色橙色起始索引
pub const PAL8_ORANGE: u8 = 152;
/// 16色米色起始索引
pub const PAL16_BEIGE: u8 = 160;
/// 16色蓝色起始索引
pub const PAL16_BLUE: u8 = 176;
/// 16色黄色起始索引
pub const PAL16_YELLOW: u8 = 192;
/// 16色橙色起始索引
pub const PAL16_ORANGE: u8 = 208;
/// 16色红色起始索引
pub const PAL16_RED: u8 = 224;
/// 16色灰色起始索引
pub const PAL16_GRAY: u8 = 240;

/// RGB 颜色
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    pub const fn white() -> Self {
        Self { r: 255, g: 255, b: 255 }
    }

    /// 从 RGB 字节数组创建
    pub const fn from_bytes(bytes: [u8; 3]) -> Self {
        Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
        }
    }

    /// 转换为 RGB 字节数组
    pub const fn to_bytes(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// 转换为 RGBA u32 (0xRRGGBBAA)
    pub const fn to_rgba_u32(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | 0xFF
    }

    /// 转换为 ARGB u32 (0xAARRGGBB)
    pub const fn to_argb_u32(self) -> u32 {
        0xFF000000 | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// 256 色调色板
#[derive(Clone)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [Color::black(); 256],
        }
    }
}

impl Palette {
    /// 创建新的空调色板
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 768 字节 RGB 数据加载调色板
    pub fn from_rgb_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 768 {
            return None;
        }

        let mut palette = Self::new();
        for i in 0..256 {
            let offset = i * 3;
            palette.colors[i] = Color::new(
                data[offset],
                data[offset + 1],
                data[offset + 2],
            );
        }
        Some(palette)
    }

    /// 获取颜色
    #[inline]
    pub fn get(&self, index: u8) -> Color {
        self.colors[index as usize]
    }

    /// 设置颜色
    #[inline]
    pub fn set(&mut self, index: u8, color: Color) {
        self.colors[index as usize] = color;
    }

    /// 应用亮度调整
    ///
    /// brightness: 0 = 全黑, 256 = 正常亮度
    pub fn apply_brightness(&mut self, brightness: u32) {
        if brightness >= 256 {
            return;
        }

        for color in &mut self.colors {
            color.r = ((color.r as u32 * brightness) >> 8) as u8;
            color.g = ((color.g as u32 * brightness) >> 8) as u8;
            color.b = ((color.b as u32 * brightness) >> 8) as u8;
        }
    }

    /// 应用淡入/淡出效果
    ///
    /// fade_level: 0 = 全黑, 256 = 无效果
    pub fn apply_fade(&self, fade_level: u32) -> Palette {
        let mut result = self.clone();
        if fade_level < 256 {
            for color in &mut result.colors {
                color.r = ((color.r as u32 * fade_level) >> 8) as u8;
                color.g = ((color.g as u32 * fade_level) >> 8) as u8;
                color.b = ((color.b as u32 * fade_level) >> 8) as u8;
            }
        }
        result
    }

    /// 设置为全黑
    pub fn set_black(&mut self) {
        for color in &mut self.colors {
            *color = Color::black();
        }
    }

    /// 转换为 RGB 字节数组 (768 字节)
    pub fn to_rgb_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(768);
        for color in &self.colors {
            result.push(color.r);
            result.push(color.g);
            result.push(color.b);
        }
        result
    }

    /// 转换为 RGBA 字节数组 (1024 字节)
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1024);
        for color in &self.colors {
            result.push(color.r);
            result.push(color.g);
            result.push(color.b);
            result.push(255); // Alpha
        }
        result
    }
}

/// 光照表 - 用于不同光照级别的颜色映射
#[derive(Clone)]
pub struct LightTable {
    /// 每个光照级别的颜色映射表 (32 级别 x 256 颜色)
    pub tables: [[u8; 256]; 32],
}

impl Default for LightTable {
    fn default() -> Self {
        // 默认为恒等映射
        let mut tables = [[0u8; 256]; 32];
        for table in &mut tables {
            for i in 0..256 {
                table[i] = i as u8;
            }
        }
        Self { tables }
    }
}

impl LightTable {
    /// 创建新的光照表
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取指定光照级别的颜色映射表
    #[inline]
    pub fn get_table(&self, level: usize) -> &[u8; 256] {
        &self.tables[level.min(31)]
    }

    /// 应用光照到颜色索引
    #[inline]
    pub fn apply(&self, level: usize, color_index: u8) -> u8 {
        self.tables[level.min(31)][color_index as usize]
    }

    /// 从原始数据加载 (32 * 256 = 8192 字节)
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 32 * 256 {
            return None;
        }

        let mut tables = [[0u8; 256]; 32];
        for (level, table) in tables.iter_mut().enumerate() {
            let offset = level * 256;
            table.copy_from_slice(&data[offset..offset + 256]);
        }
        Some(Self { tables })
    }
}

/// TRN 颜色变换表
///
/// TRN 文件用于改变精灵的颜色，例如:
/// - 不同玩家职业的颜色
/// - 石化效果
/// - 红外视觉效果
#[derive(Clone)]
pub struct ColorTransform {
    /// 256 字节的颜色映射表
    pub table: [u8; 256],
}

impl Default for ColorTransform {
    fn default() -> Self {
        // 默认为恒等映射
        let mut table = [0u8; 256];
        for i in 0..256 {
            table[i] = i as u8;
        }
        Self { table }
    }
}

impl ColorTransform {
    /// 创建恒等变换
    pub fn identity() -> Self {
        Self::default()
    }

    /// 从 256 字节数据加载
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 256 {
            return None;
        }

        let mut table = [0u8; 256];
        table.copy_from_slice(&data[..256]);
        Some(Self { table })
    }

    /// 应用变换到颜色索引
    #[inline]
    pub fn apply(&self, color_index: u8) -> u8 {
        self.table[color_index as usize]
    }

    /// 组合两个变换 (先应用 self，再应用 other)
    pub fn compose(&self, other: &ColorTransform) -> ColorTransform {
        let mut result = [0u8; 256];
        for i in 0..256 {
            result[i] = other.table[self.table[i] as usize];
        }
        ColorTransform { table: result }
    }

    /// 转换为字节数组
    pub fn to_bytes(&self) -> &[u8; 256] {
        &self.table
    }
}

/// 透明度混合查找表
///
/// 用于快速计算两个颜色索引的混合结果
#[derive(Clone)]
pub struct TransparencyTable {
    /// 256x256 查找表: table[背景色][前景色] = 混合结果
    pub table: [[u8; 256]; 256],
}

impl Default for TransparencyTable {
    fn default() -> Self {
        // 默认为简单覆盖 (无混合)
        let mut table = [[0u8; 256]; 256];
        for bg in 0..256 {
            for fg in 0..256 {
                // 默认前景色覆盖背景色
                table[bg][fg] = fg as u8;
            }
        }
        Self { table }
    }
}

impl TransparencyTable {
    /// 创建新的透明度表
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 65536 字节数据加载
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 256 * 256 {
            return None;
        }

        let mut table = [[0u8; 256]; 256];
        for bg in 0..256 {
            let offset = bg * 256;
            table[bg].copy_from_slice(&data[offset..offset + 256]);
        }
        Some(Self { table })
    }

    /// 混合两个颜色
    #[inline]
    pub fn blend(&self, background: u8, foreground: u8) -> u8 {
        self.table[background as usize][foreground as usize]
    }

    /// 使用调色板生成透明度表
    pub fn generate_from_palette(palette: &Palette, alpha: u8) -> Self {
        let mut table = [[0u8; 256]; 256];
        let alpha_f = alpha as f32 / 255.0;
        let inv_alpha = 1.0 - alpha_f;

        for bg_idx in 0..256 {
            let bg = palette.colors[bg_idx];
            for fg_idx in 0..256 {
                let fg = palette.colors[fg_idx];

                // 混合颜色
                let r = (bg.r as f32 * inv_alpha + fg.r as f32 * alpha_f) as u8;
                let g = (bg.g as f32 * inv_alpha + fg.g as f32 * alpha_f) as u8;
                let b = (bg.b as f32 * inv_alpha + fg.b as f32 * alpha_f) as u8;

                // 找到最接近的调色板颜色
                table[bg_idx][fg_idx] = find_nearest_color(palette, r, g, b);
            }
        }

        Self { table }
    }
}

/// 在调色板中找到最接近的颜色
fn find_nearest_color(palette: &Palette, r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;

    for (i, color) in palette.colors.iter().enumerate() {
        let dr = (color.r as i32 - r as i32).abs() as u32;
        let dg = (color.g as i32 - g as i32).abs() as u32;
        let db = (color.b as i32 - b as i32).abs() as u32;

        // 使用简单的距离计算 (可以改用感知加权)
        let distance = dr * dr + dg * dg + db * db;

        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let c = Color::new(255, 128, 64);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
    }

    #[test]
    fn test_palette_from_bytes() {
        let mut data = vec![0u8; 768];
        // 设置第一个颜色为红色
        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        // 设置第二个颜色为绿色
        data[3] = 0;
        data[4] = 255;
        data[5] = 0;

        let palette = Palette::from_rgb_bytes(&data).unwrap();
        assert_eq!(palette.get(0), Color::new(255, 0, 0));
        assert_eq!(palette.get(1), Color::new(0, 255, 0));
    }

    #[test]
    fn test_color_transform() {
        let mut data = [0u8; 256];
        // 创建一个简单的交换变换: 0->1, 1->0
        data[0] = 1;
        data[1] = 0;
        for i in 2..256 {
            data[i] = i as u8;
        }

        let trn = ColorTransform::from_bytes(&data).unwrap();
        assert_eq!(trn.apply(0), 1);
        assert_eq!(trn.apply(1), 0);
        assert_eq!(trn.apply(2), 2);
    }

    #[test]
    fn test_light_table() {
        let light_table = LightTable::new();
        // 默认应为恒等映射
        assert_eq!(light_table.apply(0, 100), 100);
        assert_eq!(light_table.apply(15, 200), 200);
    }
}
