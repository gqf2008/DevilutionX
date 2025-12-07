//! TRN - 颜色变换表
//!
//! 移植自 Source/engine/trn.hpp
//!
//! TRN (Translation) 文件用于颜色映射，实现各种视觉效果：
//! - 红外视觉 (Infravision): 使怪物在黑暗中可见
//! - 石化效果 (Stone): 将角色变成灰色石头
//! - 暂停效果 (Pause): 使整个画面变暗
//! - 玩家皮肤颜色 (Class TRN): 不同职业的颜色方案

/// TRN 表 - 256 字节的颜色索引映射
pub type TrnTable = [u8; 256];

/// 颜色变换表管理器
#[derive(Clone)]
pub struct ColorTransform {
    /// 红外视觉表
    pub infravision: TrnTable,
    /// 石化效果表
    pub stone: TrnTable,
    /// 暂停效果表
    pub pause: TrnTable,
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorTransform {
    /// 创建默认的颜色变换表（恒等映射）
    pub fn new() -> Self {
        let identity = create_identity_trn();
        Self {
            infravision: identity,
            stone: identity,
            pause: identity,
        }
    }

    /// 从文件数据加载
    pub fn load(&mut self, infra_data: &[u8], stone_data: &[u8], pause_data: &[u8]) -> bool {
        if infra_data.len() < 256 || stone_data.len() < 256 || pause_data.len() < 256 {
            return false;
        }
        self.infravision.copy_from_slice(&infra_data[..256]);
        self.stone.copy_from_slice(&stone_data[..256]);
        self.pause.copy_from_slice(&pause_data[..256]);
        true
    }

    /// 应用颜色变换
    #[inline]
    pub fn apply(&self, table: &TrnTable, color: u8) -> u8 {
        table[color as usize]
    }

    /// 应用红外视觉变换
    #[inline]
    pub fn apply_infravision(&self, color: u8) -> u8 {
        self.apply(&self.infravision, color)
    }

    /// 应用石化变换
    #[inline]
    pub fn apply_stone(&self, color: u8) -> u8 {
        self.apply(&self.stone, color)
    }

    /// 应用暂停变换
    #[inline]
    pub fn apply_pause(&self, color: u8) -> u8 {
        self.apply(&self.pause, color)
    }
}

/// 创建恒等变换表（无变换）
pub fn create_identity_trn() -> TrnTable {
    let mut table = [0u8; 256];
    for (i, v) in table.iter_mut().enumerate() {
        *v = i as u8;
    }
    table
}

/// 从数据创建 TRN 表
pub fn trn_from_data(data: &[u8]) -> Option<TrnTable> {
    if data.len() < 256 {
        return None;
    }
    let mut table = [0u8; 256];
    table.copy_from_slice(&data[..256]);
    Some(table)
}

/// 创建灰度变换表
///
/// 将彩色调色板映射到灰度
pub fn create_grayscale_trn(palette: &[[u8; 3]; 256], gray_start: u8, gray_count: u8) -> TrnTable {
    let mut table = [0u8; 256];
    let gray_range = gray_count as f32;

    for (i, v) in table.iter_mut().enumerate() {
        let [r, g, b] = palette[i];
        // 使用标准亮度公式
        let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        let gray_index = ((luminance / 255.0) * gray_range) as u8;
        *v = gray_start + gray_index.min(gray_count - 1);
    }
    table
}

/// 创建单色变换表
///
/// 将所有颜色映射到指定颜色范围
pub fn create_monochrome_trn(
    palette: &[[u8; 3]; 256],
    target_start: u8,
    target_count: u8,
    hue: MonochromeHue,
) -> TrnTable {
    let mut table = [0u8; 256];
    let range = target_count as f32;

    for (i, v) in table.iter_mut().enumerate() {
        let [r, g, b] = palette[i];

        // 根据色调选择通道
        let intensity = match hue {
            MonochromeHue::Red => r as f32,
            MonochromeHue::Green => g as f32,
            MonochromeHue::Blue => b as f32,
            MonochromeHue::Gray => {
                0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
            }
        };

        let index = ((intensity / 255.0) * range) as u8;
        *v = target_start + index.min(target_count - 1);
    }
    table
}

/// 单色色调类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonochromeHue {
    Red,
    Green,
    Blue,
    Gray,
}

/// 创建变暗变换表
///
/// 将颜色映射到更暗的版本
pub fn create_darken_trn(light_table: &[u8], darkness_level: usize) -> TrnTable {
    let mut table = [0u8; 256];
    let offset = darkness_level * 256;

    for (i, v) in table.iter_mut().enumerate() {
        if offset + i < light_table.len() {
            *v = light_table[offset + i];
        } else {
            *v = 0; // 完全黑暗
        }
    }
    table
}

/// 组合两个变换表
pub fn combine_trn(first: &TrnTable, second: &TrnTable) -> TrnTable {
    let mut result = [0u8; 256];
    for (i, v) in result.iter_mut().enumerate() {
        *v = second[first[i] as usize];
    }
    result
}

/// 玩家职业颜色方案
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerClass {
    Warrior,
    Rogue,
    Sorcerer,
    Monk,
    Bard,
    Barbarian,
}

impl PlayerClass {
    /// 获取职业 TRN 文件名
    pub fn trn_filename(&self) -> &'static str {
        match self {
            PlayerClass::Warrior => "plrgfx\\warrior.trn",
            PlayerClass::Rogue => "plrgfx\\rogue.trn",
            PlayerClass::Sorcerer => "plrgfx\\sorceror.trn",
            PlayerClass::Monk => "plrgfx\\monk.trn",
            PlayerClass::Bard => "plrgfx\\bard.trn",
            PlayerClass::Barbarian => "plrgfx\\barbarian.trn",
        }
    }
}

/// 特效类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectType {
    /// 正常（无效果）
    None,
    /// 红外视觉
    Infravision,
    /// 石化
    Stone,
    /// 暂停
    Pause,
    /// 自定义 TRN
    Custom,
}

/// 渲染时使用的颜色变换上下文
pub struct TrnContext<'a> {
    /// 当前活动的变换表
    pub active_trn: Option<&'a TrnTable>,
    /// 效果类型
    pub effect: EffectType,
}

impl<'a> TrnContext<'a> {
    /// 无变换
    pub fn none() -> Self {
        Self {
            active_trn: None,
            effect: EffectType::None,
        }
    }

    /// 使用指定变换
    pub fn with_trn(trn: &'a TrnTable, effect: EffectType) -> Self {
        Self {
            active_trn: Some(trn),
            effect,
        }
    }

    /// 应用变换
    #[inline]
    pub fn transform(&self, color: u8) -> u8 {
        match self.active_trn {
            Some(trn) => trn[color as usize],
            None => color,
        }
    }

    /// 是否有活动变换
    pub fn is_active(&self) -> bool {
        self.active_trn.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_trn() {
        let trn = create_identity_trn();
        for i in 0..256 {
            assert_eq!(trn[i], i as u8);
        }
    }

    #[test]
    fn test_trn_from_data() {
        let data: Vec<u8> = (0..256).map(|i| 255 - i as u8).collect();
        let trn = trn_from_data(&data).unwrap();

        for i in 0..256 {
            assert_eq!(trn[i], 255 - i as u8);
        }
    }

    #[test]
    fn test_trn_from_data_too_short() {
        let data = vec![0u8; 100];
        assert!(trn_from_data(&data).is_none());
    }

    #[test]
    fn test_combine_trn() {
        // 创建两个简单变换
        let mut first = create_identity_trn();
        let mut second = create_identity_trn();

        // first: 将 0 映射到 1
        first[0] = 1;
        // second: 将 1 映射到 2
        second[1] = 2;

        let combined = combine_trn(&first, &second);

        // 0 -> first -> 1 -> second -> 2
        assert_eq!(combined[0], 2);
    }

    #[test]
    fn test_color_transform() {
        let transform = ColorTransform::new();

        // 默认应该是恒等映射
        for i in 0..256 {
            assert_eq!(transform.apply_infravision(i as u8), i as u8);
            assert_eq!(transform.apply_stone(i as u8), i as u8);
            assert_eq!(transform.apply_pause(i as u8), i as u8);
        }
    }

    #[test]
    fn test_trn_context() {
        let ctx = TrnContext::none();
        assert!(!ctx.is_active());
        assert_eq!(ctx.transform(128), 128);

        let mut trn = create_identity_trn();
        trn[128] = 64;

        let ctx = TrnContext::with_trn(&trn, EffectType::Custom);
        assert!(ctx.is_active());
        assert_eq!(ctx.transform(128), 64);
    }
}
