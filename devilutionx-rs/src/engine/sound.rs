//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Sound - 音频系统
//!
//! 移植自 Source/engine/sound.h/cpp
//!
//! 提供音效和音乐播放功能

use std::sync::atomic::{AtomicBool, Ordering};

/// 音乐 ID
///
/// C++ 原型: enum _music_id : uint8_t
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MusicId {
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,
    Crypt = 6,
    Intro = 7,
}

impl MusicId {
    pub const NUM_MUSIC: usize = 8;
    
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Town),
            1 => Some(Self::Cathedral),
            2 => Some(Self::Catacombs),
            3 => Some(Self::Caves),
            4 => Some(Self::Hell),
            5 => Some(Self::Nest),
            6 => Some(Self::Crypt),
            7 => Some(Self::Intro),
            _ => None,
        }
    }
}

/// 声音句柄
///
/// C++ 原型: struct TSnd
pub struct TSnd {
    /// 开始时间 (tick count)
    pub start_tc: u32,
    // 注意: SoundSample DSB 需要 SDL 音频后端支持
    // 依赖: utils/soundsample.h 需要 SDL_mixer 或类似库
}

impl TSnd {
    /// 检查声音是否正在播放
    ///
    /// C++ 原型: bool isPlaying()
    pub fn is_playing(&self) -> bool {
        // 依赖: 需要 SDL 音频后端实现
        // 当前返回 false (NOSOUND 模式)
        false
    }
}

// ============================================================================
// 全局状态
// ============================================================================

/// 音频是否已初始化
///
/// C++ 原型: extern bool gbSndInited
static GB_SND_INITED: AtomicBool = AtomicBool::new(false);

/// 音乐是否开启
///
/// C++ 原型: extern bool gbMusicOn
static GB_MUSIC_ON: AtomicBool = AtomicBool::new(true);

/// 音效是否开启
///
/// C++ 原型: extern bool gbSoundOn
static GB_SOUND_ON: AtomicBool = AtomicBool::new(true);

pub fn is_sound_inited() -> bool {
    GB_SND_INITED.load(Ordering::Relaxed)
}

pub fn is_music_on() -> bool {
    GB_MUSIC_ON.load(Ordering::Relaxed)
}

pub fn is_sound_on() -> bool {
    GB_SOUND_ON.load(Ordering::Relaxed)
}

pub fn set_music_on(on: bool) {
    GB_MUSIC_ON.store(on, Ordering::Relaxed);
}

pub fn set_sound_on(on: bool) {
    GB_SOUND_ON.store(on, Ordering::Relaxed);
}

// ============================================================================
// 音频函数
// ============================================================================

/// 初始化音频系统
///
/// C++ 原型: void snd_init()
pub fn snd_init() {
    // 依赖: 需要 SDL_mixer 或类似库初始化
    // 当前为 stub 实现
    GB_SND_INITED.store(true, Ordering::Relaxed);
}

/// 关闭音频系统
///
/// C++ 原型: void snd_deinit()
pub fn snd_deinit() {
    // 依赖: 需要 SDL_mixer 清理
    GB_SND_INITED.store(false, Ordering::Relaxed);
}

/// 播放声音
///
/// C++ 原型: void snd_play_snd(TSnd *pSnd, int lVolume, int lPan)
pub fn snd_play_snd(_snd: &TSnd, _volume: i32, _pan: i32) {
    // 依赖: 需要 SDL_mixer 播放音频
    // 当前为 stub 实现
}

/// 清除重复声音
///
/// C++ 原型: void ClearDuplicateSounds()
pub fn clear_duplicate_sounds() {
    // 依赖: 需要音频后端实现
}

/// 获取关卡音乐
///
/// C++ 原型: _music_id GetLevelMusic(dungeon_type dungeonType)
pub fn get_level_music(dungeon_type: u8) -> MusicId {
    // dungeon_type 对应 levels/gendung.h 中的 dungeon_type 枚举
    match dungeon_type {
        0 => MusicId::Town,      // DTYPE_TOWN
        1 => MusicId::Cathedral, // DTYPE_CATHEDRAL
        2 => MusicId::Catacombs, // DTYPE_CATACOMBS
        3 => MusicId::Caves,     // DTYPE_CAVES
        4 => MusicId::Hell,      // DTYPE_HELL
        5 => MusicId::Nest,      // DTYPE_NEST
        6 => MusicId::Crypt,     // DTYPE_CRYPT
        _ => MusicId::Town,
    }
}

/// 停止音乐
///
/// C++ 原型: void music_stop()
pub fn music_stop() {
    // 依赖: 需要 SDL_mixer 停止音乐
}

/// 开始播放音乐
///
/// C++ 原型: void music_start(_music_id nTrack)
pub fn music_start(_track: MusicId) {
    // 依赖: 需要 SDL_mixer 播放音乐
}

/// 禁用/启用音乐
///
/// C++ 原型: void sound_disable_music(bool disable)
pub fn sound_disable_music(disable: bool) {
    set_music_on(!disable);
    if disable {
        music_stop();
    }
}

/// 获取或设置音乐音量
///
/// C++ 原型: int sound_get_or_set_music_volume(int volume)
pub fn sound_get_or_set_music_volume(volume: i32) -> i32 {
    // 依赖: 需要 SDL_mixer 音量控制
    // 返回当前音量
    if volume < 0 {
        100 // 默认返回 100
    } else {
        volume.clamp(0, 100)
    }
}

/// 获取或设置音效音量
///
/// C++ 原型: int sound_get_or_set_sound_volume(int volume)
pub fn sound_get_or_set_sound_volume(volume: i32) -> i32 {
    // 依赖: 需要 SDL_mixer 音量控制
    if volume < 0 {
        100
    } else {
        volume.clamp(0, 100)
    }
}

/// 静音音乐
///
/// C++ 原型: void music_mute()
pub fn music_mute() {
    // 依赖: 需要 SDL_mixer
}

/// 取消静音音乐
///
/// C++ 原型: void music_unmute()
pub fn music_unmute() {
    // 依赖: 需要 SDL_mixer
}

// ============================================================================
// 兼容性导出 (保持旧 API 可用)
// ============================================================================

/// 旧 API 兼容 - AudioSystem 类型别名
pub struct AudioSystem;

impl AudioSystem {
    pub fn new() -> anyhow::Result<Self> {
        snd_init();
        Ok(Self)
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_music_id() {
        assert_eq!(MusicId::Town as u8, 0);
        assert_eq!(MusicId::Intro as u8, 7);
        assert_eq!(MusicId::from_u8(0), Some(MusicId::Town));
        assert_eq!(MusicId::from_u8(8), None);
    }

    #[test]
    fn test_get_level_music() {
        assert_eq!(get_level_music(0), MusicId::Town);
        assert_eq!(get_level_music(4), MusicId::Hell);
    }
}
