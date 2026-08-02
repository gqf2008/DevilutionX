//! 音效系统
//!
//! 移植自 Source/effects.cpp
//! 负责加载和播放游戏音效

use std::time::Instant;

/// 音量最小值
pub const VOLUME_MIN: i32 = -1600;
/// 音量最大值
pub const VOLUME_MAX: i32 = 0;

/// 音效标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SfxFlag {
    /// 流式音效
    Stream = 0x01,
    /// 杂项音效
    Misc = 0x02,
    /// UI音效
    Ui = 0x04,
    /// 武僧音效
    Monk = 0x08,
    /// 盗贼音效
    Rogue = 0x10,
    /// 战士音效
    Warrior = 0x20,
    /// 法师音效
    Sorcerer = 0x40,
}

impl SfxFlag {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Stream" => Some(Self::Stream),
            "Misc" => Some(Self::Misc),
            "Ui" => Some(Self::Ui),
            "Monk" => Some(Self::Monk),
            "Rogue" => Some(Self::Rogue),
            "Warrior" => Some(Self::Warrior),
            "Sorcerer" => Some(Self::Sorcerer),
            _ => None,
        }
    }
}

/// 音效ID枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum SfxID {
    None = -1,
    /// 行走
    Walk = 0,
    /// 射箭
    ShootBow = 1,
    /// 施法
    CastSpell = 2,
    /// 挥砍
    Swing = 3,
    /// 战士69
    Warrior69 = 10,
    /// 法师69
    Sorceror69 = 11,
    /// 盗贼69
    Rogue69 = 12,
    /// 武僧69
    Monk69 = 13,
    /// 酸液法术
    SpellAcid = 20,
    /// 操作神龛
    OperateShrine = 30,
    /// 战士14
    Warrior14 = 40,
    /// 战士15
    Warrior15 = 41,
    /// 战士16
    Warrior16 = 42,
    /// 战士2
    Warrior2 = 43,
    /// 盗贼14
    Rogue14 = 50,
    /// 法师14
    Sorceror14 = 51,
    /// 武僧14
    Monk14 = 52,
}

impl SfxID {
    /// 获取数值
    pub fn as_i16(self) -> i16 {
        self as i16
    }

    /// 从i16转换
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            -1 => Some(Self::None),
            0 => Some(Self::Walk),
            1 => Some(Self::ShootBow),
            2 => Some(Self::CastSpell),
            3 => Some(Self::Swing),
            10 => Some(Self::Warrior69),
            11 => Some(Self::Sorceror69),
            12 => Some(Self::Rogue69),
            13 => Some(Self::Monk69),
            20 => Some(Self::SpellAcid),
            30 => Some(Self::OperateShrine),
            40 => Some(Self::Warrior14),
            41 => Some(Self::Warrior15),
            42 => Some(Self::Warrior16),
            43 => Some(Self::Warrior2),
            50 => Some(Self::Rogue14),
            51 => Some(Self::Sorceror14),
            52 => Some(Self::Monk14),
            _ => None,
        }
    }
}

/// 英雄语音类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroSpeech {
    /// 受伤
    TakingDamage,
    /// 死亡
    Dying,
    /// 升级
    LevelUp,
    /// 生命值低
    LowHealth,
    /// 魔法值低
    LowMana,
}

/// 英雄职业
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroClass {
    Warrior,
    Rogue,
    Sorcerer,
    Monk,
    Bard,
    Barbarian,
}

impl HeroClass {
    /// 获取音效标志掩码
    pub fn get_sfx_mask(&self) -> u8 {
        match self {
            HeroClass::Warrior | HeroClass::Barbarian => SfxFlag::Warrior as u8,
            HeroClass::Rogue | HeroClass::Bard => SfxFlag::Rogue as u8,
            HeroClass::Sorcerer => SfxFlag::Sorcerer as u8,
            HeroClass::Monk => SfxFlag::Monk as u8,
        }
    }
}

/// 音效数据
#[derive(Debug, Clone)]
pub struct SoundEffect {
    /// 标志
    pub flags: u8,
    /// 文件路径
    pub path: String,
    /// 是否已加载
    pub loaded: bool,
    /// 是否正在播放
    pub playing: bool,
    /// 开始时间
    pub start_time: Option<Instant>,
    /// 持续时间（毫秒）
    pub duration_ms: u32,
}

impl SoundEffect {
    /// 创建新的音效
    pub fn new(flags: u8, path: String) -> Self {
        Self {
            flags,
            path,
            loaded: false,
            playing: false,
            start_time: None,
            duration_ms: 0,
        }
    }

    /// 是否是流式音效
    pub fn is_stream(&self) -> bool {
        (self.flags & SfxFlag::Stream as u8) != 0
    }

    /// 是否是杂项音效
    pub fn is_misc(&self) -> bool {
        (self.flags & SfxFlag::Misc as u8) != 0
    }

    /// 是否是UI音效
    pub fn is_ui(&self) -> bool {
        (self.flags & SfxFlag::Ui as u8) != 0
    }

    /// 检查是否仍在播放
    pub fn update_playing_state(&mut self) {
        if let Some(start) = self.start_time {
            if start.elapsed().as_millis() as u32 >= self.duration_ms {
                self.playing = false;
                self.start_time = None;
            }
        }
    }
}

/// 音效管理器
#[derive(Debug)]
pub struct EffectsManager {
    /// 音效列表
    effects: Vec<SoundEffect>,
    /// 当前流式音效索引
    current_stream: Option<usize>,
    /// 延迟播放
    pub sfx_delay: i32,
    /// 延迟播放的音效ID
    pub sfx_delay_id: SfxID,
    /// 音效是否初始化
    pub sound_initialized: bool,
    /// 音效是否启用
    pub sound_enabled: bool,
    /// 是否是多人游戏
    pub is_multiplayer: bool,
    /// 是否是spawn版本
    pub is_spawn: bool,
}

impl Default for EffectsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectsManager {
    /// 创建新的音效管理器
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            current_stream: None,
            sfx_delay: 0,
            sfx_delay_id: SfxID::None,
            sound_initialized: false,
            sound_enabled: true,
            is_multiplayer: false,
            is_spawn: false,
        }
    }

    /// 加载音效数据
    pub fn load_effects_data(&mut self, data: &[(u8, &str)]) {
        self.effects.clear();
        self.effects.reserve(data.len());

        for (flags, path) in data {
            self.effects.push(SoundEffect::new(*flags, path.to_string()));
        }
    }

    /// 初始化音效（基于玩家职业）
    pub fn init_for_class(&mut self, hero_class: HeroClass) {
        if !self.sound_initialized {
            return;
        }

        let mut mask = SfxFlag::Misc as u8;

        if self.is_multiplayer {
            mask |= SfxFlag::Warrior as u8 | SfxFlag::Monk as u8;
            if !self.is_spawn {
                mask |= SfxFlag::Rogue as u8 | SfxFlag::Sorcerer as u8;
            }
        } else {
            mask |= hero_class.get_sfx_mask();
        }

        self.load_effects_with_mask(mask);
    }

    /// 初始化UI音效
    pub fn init_ui_sounds(&mut self) {
        self.load_effects_with_mask(SfxFlag::Ui as u8);
    }

    /// 根据掩码加载音效
    fn load_effects_with_mask(&mut self, mask: u8) {
        if !self.sound_initialized {
            return;
        }

        for effect in &mut self.effects {
            if effect.flags == 0 || effect.loaded {
                continue;
            }

            // 跳过流式音效（按需加载）
            if effect.is_stream() {
                continue;
            }

            if (effect.flags & mask) == 0 {
                continue;
            }

            // 标记为已加载（实际加载由音频后端处理）
            effect.loaded = true;
        }
    }

    /// 检查音效是否正在播放
    pub fn is_playing(&self, sfx_id: SfxID) -> bool {
        if !self.sound_initialized {
            return false;
        }

        let index = sfx_id.as_i16();
        if index < 0 || index as usize >= self.effects.len() {
            return false;
        }

        let effect = &self.effects[index as usize];
        if effect.playing {
            return true;
        }

        if effect.is_stream() {
            return self.current_stream == Some(index as usize);
        }

        false
    }

    /// 停止流式音效
    pub fn stop_stream(&mut self) {
        if let Some(index) = self.current_stream.take() {
            if let Some(effect) = self.effects.get_mut(index) {
                effect.playing = false;
                effect.start_time = None;
            }
        }
    }

    /// 播放音效
    pub fn play_sfx(&mut self, sfx_id: SfxID) {
        let sfx_id = self.randomize_sfx(sfx_id);

        if !self.sound_initialized {
            return;
        }

        self.play_sfx_internal(sfx_id, false, (0, 0));
    }

    /// 在指定位置播放音效
    pub fn play_sfx_at(&mut self, sfx_id: SfxID, position: (i32, i32), randomize: bool) {
        let sfx_id = if randomize {
            self.randomize_sfx(sfx_id)
        } else {
            sfx_id
        };

        if !self.sound_initialized {
            return;
        }

        // 某些音效需要重置计时
        if matches!(sfx_id, SfxID::Walk | SfxID::ShootBow | SfxID::CastSpell | SfxID::Swing) {
            if let Some(effect) = self.effects.get_mut(sfx_id.as_i16() as usize) {
                effect.start_time = None;
            }
        }

        self.play_sfx_internal(sfx_id, true, position);
    }

    /// 内部播放音效
    fn play_sfx_internal(&mut self, sfx_id: SfxID, localized: bool, position: (i32, i32)) {
        let index = sfx_id.as_i16();
        if index < 0 || index as usize >= self.effects.len() {
            return;
        }

        if !self.sound_enabled {
            return;
        }

        let is_stream = self.effects[index as usize].is_stream();
        let is_misc = self.effects[index as usize].is_misc();
        let is_playing = self.effects[index as usize].playing;

        // 非流式、非杂项音效如果正在播放则跳过
        if !is_stream && !is_misc && is_playing {
            return;
        }

        // 计算音量和声像
        let (volume, _pan) = if localized {
            self.calculate_sound_position(position)
        } else {
            (0, 0)
        };

        // 检查音量是否足够
        if localized && volume < VOLUME_MIN {
            return;
        }

        if is_stream {
            // 流式播放 - 先停止当前流
            self.stop_stream();
        }

        let effect = &mut self.effects[index as usize];
        effect.playing = true;
        effect.start_time = Some(Instant::now());

        if is_stream {
            self.current_stream = Some(index as usize);
        }
    }

    /// 随机化某些音效
    fn randomize_sfx(&self, sfx_id: SfxID) -> SfxID {

        match sfx_id {
            // 2个变种
            SfxID::Warrior69 | SfxID::Sorceror69 | SfxID::Rogue69 |
            SfxID::Monk69 | SfxID::Swing | SfxID::SpellAcid | SfxID::OperateShrine => {
                let offset = crate::engine::random::gameplay_rnd(0, 1) as i16;
                SfxID::from_i16(sfx_id.as_i16() + offset).unwrap_or(sfx_id)
            }
            // 3个变种
            SfxID::Warrior14 | SfxID::Warrior15 | SfxID::Warrior16 |
            SfxID::Warrior2 | SfxID::Rogue14 | SfxID::Sorceror14 | SfxID::Monk14 => {
                let offset = crate::engine::random::gameplay_rnd(0, 2) as i16;
                SfxID::from_i16(sfx_id.as_i16() + offset).unwrap_or(sfx_id)
            }
            _ => sfx_id,
        }
    }

    /// 计算音效位置（音量和声像）
    fn calculate_sound_position(&self, _position: (i32, i32)) -> (i32, i32) {
        // 简化实现，实际需要基于玩家位置计算
        (0, 0)
    }

    /// 停止所有音效
    pub fn stop_all(&mut self) {
        if !self.sound_initialized {
            return;
        }

        for effect in &mut self.effects {
            effect.playing = false;
            effect.start_time = None;
        }

        self.current_stream = None;
    }

    /// 更新音效状态
    pub fn update(&mut self) {
        if !self.sound_initialized {
            return;
        }

        // 更新流式音效状态
        if let Some(index) = self.current_stream {
            if let Some(effect) = self.effects.get_mut(index) {
                effect.update_playing_state();
                if !effect.playing {
                    self.current_stream = None;
                }
            }
        }
    }

    /// 清理音效
    pub fn cleanup(&mut self) {
        self.stop_all();
        for effect in &mut self.effects {
            effect.loaded = false;
        }
    }

    /// 直接播放音效（无位置）
    pub fn play_effect(&mut self, sfx_id: SfxID) {
        if !self.sound_initialized || !self.sound_enabled {
            return;
        }

        let index = sfx_id.as_i16();
        if index < 0 || index as usize >= self.effects.len() {
            return;
        }

        let effect = &mut self.effects[index as usize];
        if !effect.playing {
            effect.playing = true;
            effect.start_time = Some(Instant::now());
        }
    }

    /// 获取音效长度
    pub fn get_sfx_length(&self, sfx_id: SfxID) -> u32 {
        let index = sfx_id.as_i16();
        if index < 0 || index as usize >= self.effects.len() {
            return 0;
        }

        self.effects[index as usize].duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sfx_flag_from_str() {
        assert_eq!(SfxFlag::from_str("Stream"), Some(SfxFlag::Stream));
        assert_eq!(SfxFlag::from_str("Misc"), Some(SfxFlag::Misc));
        assert_eq!(SfxFlag::from_str("Unknown"), None);
    }

    #[test]
    fn test_sfx_id_conversion() {
        assert_eq!(SfxID::Walk.as_i16(), 0);
        assert_eq!(SfxID::from_i16(0), Some(SfxID::Walk));
        assert_eq!(SfxID::from_i16(-1), Some(SfxID::None));
        assert_eq!(SfxID::from_i16(999), None);
    }

    #[test]
    fn test_hero_class_sfx_mask() {
        assert_eq!(HeroClass::Warrior.get_sfx_mask(), SfxFlag::Warrior as u8);
        assert_eq!(HeroClass::Rogue.get_sfx_mask(), SfxFlag::Rogue as u8);
        assert_eq!(HeroClass::Sorcerer.get_sfx_mask(), SfxFlag::Sorcerer as u8);
        assert_eq!(HeroClass::Monk.get_sfx_mask(), SfxFlag::Monk as u8);
    }

    #[test]
    fn test_sound_effect_new() {
        let effect = SoundEffect::new(0x03, "test.wav".to_string());
        assert_eq!(effect.flags, 0x03);
        assert_eq!(effect.path, "test.wav");
        assert!(!effect.loaded);
        assert!(!effect.playing);
    }

    #[test]
    fn test_sound_effect_flags() {
        let stream = SoundEffect::new(SfxFlag::Stream as u8, "s.wav".to_string());
        assert!(stream.is_stream());
        assert!(!stream.is_misc());

        let misc = SoundEffect::new(SfxFlag::Misc as u8, "m.wav".to_string());
        assert!(!misc.is_stream());
        assert!(misc.is_misc());
    }

    #[test]
    fn test_effects_manager_new() {
        let manager = EffectsManager::new();
        assert!(!manager.sound_initialized);
        assert!(manager.sound_enabled);
        assert!(manager.effects.is_empty());
    }

    #[test]
    fn test_effects_manager_load_data() {
        let mut manager = EffectsManager::new();
        manager.load_effects_data(&[
            (0x02, "sound1.wav"),
            (0x04, "sound2.wav"),
        ]);
        assert_eq!(manager.effects.len(), 2);
    }

    #[test]
    fn test_effects_manager_stop_stream() {
        let mut manager = EffectsManager::new();
        manager.load_effects_data(&[(SfxFlag::Stream as u8, "stream.wav")]);
        manager.sound_initialized = true;
        manager.current_stream = Some(0);
        manager.effects[0].playing = true;

        manager.stop_stream();
        assert!(manager.current_stream.is_none());
        assert!(!manager.effects[0].playing);
    }

    #[test]
    fn test_effects_manager_is_playing() {
        let mut manager = EffectsManager::new();
        manager.load_effects_data(&[(0x02, "test.wav")]);
        manager.sound_initialized = true;

        assert!(!manager.is_playing(SfxID::Walk));

        manager.effects[0].playing = true;
        assert!(manager.is_playing(SfxID::Walk));
    }

    #[test]
    fn test_effects_manager_stop_all() {
        let mut manager = EffectsManager::new();
        manager.load_effects_data(&[
            (0x02, "sound1.wav"),
            (0x02, "sound2.wav"),
        ]);
        manager.sound_initialized = true;
        manager.effects[0].playing = true;
        manager.effects[1].playing = true;
        manager.current_stream = Some(0);

        manager.stop_all();
        assert!(!manager.effects[0].playing);
        assert!(!manager.effects[1].playing);
        assert!(manager.current_stream.is_none());
    }

    #[test]
    fn test_effects_manager_cleanup() {
        let mut manager = EffectsManager::new();
        manager.load_effects_data(&[(0x02, "test.wav")]);
        manager.sound_initialized = true;
        manager.effects[0].loaded = true;
        manager.effects[0].playing = true;

        manager.cleanup();
        assert!(!manager.effects[0].loaded);
        assert!(!manager.effects[0].playing);
    }
}
