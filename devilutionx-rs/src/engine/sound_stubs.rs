//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Sound Stubs - 禁用声音模式的桩函数实现
//!
//! 移植自 Source/engine/sound_stubs.cpp
//! 当 NOSOUND 模式启用时使用这些空实现

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// 音乐 ID
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicId {
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,
    Crypt = 6,
    Intro = 7,
    NumMusic = 8,
}

impl From<u8> for MusicId {
    fn from(v: u8) -> Self {
        match v {
            0 => MusicId::Town,
            1 => MusicId::Cathedral,
            2 => MusicId::Catacombs,
            3 => MusicId::Caves,
            4 => MusicId::Hell,
            5 => MusicId::Nest,
            6 => MusicId::Crypt,
            7 => MusicId::Intro,
            _ => MusicId::NumMusic,
        }
    }
}

/// 地下城类型 (用于 GetLevelMusic)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DungeonType {
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,
    Crypt = 6,
}

/// 声音是否已初始化
pub static GB_SND_INITED: AtomicBool = AtomicBool::new(false);
/// 音乐是否开启
pub static GB_MUSIC_ON: AtomicBool = AtomicBool::new(false);
/// 音效是否开启  
pub static GB_SOUND_ON: AtomicBool = AtomicBool::new(false);
/// 当前音乐轨道
pub static SGN_MUSIC_TRACK: AtomicU8 = AtomicU8::new(MusicId::NumMusic as u8);

/// 声音句柄 (桩实现)
pub struct TSnd {
    pub start_tc: u32,
}

impl TSnd {
    pub fn is_playing(&self) -> bool {
        false
    }
}

impl Drop for TSnd {
    fn drop(&mut self) {
        // 空实现
    }
}

/// 清除重复声音
pub fn clear_duplicate_sounds() {
    // 空实现
}

/// 播放声音
pub fn snd_play_snd(_snd: Option<&mut TSnd>, _volume: i32, _pan: i32) {
    // 空实现
}

/// 加载声音文件
pub fn sound_file_load(_path: &str, _stream: bool) -> Option<TSnd> {
    None
}

/// 加载声音文件并返回状态
pub fn sound_file_load_with_status(_path: &str, _stream: bool) -> Result<TSnd, String> {
    Err("NOSOUND mode".to_string())
}

/// 初始化声音系统
pub fn snd_init() {
    // 空实现
}

/// 关闭声音系统
pub fn snd_deinit() {
    // 空实现
}

/// 停止音乐
pub fn music_stop() {
    // 空实现
}

/// 开始播放音乐
pub fn music_start(_track: MusicId) {
    // 空实现
}

/// 禁用/启用音乐
pub fn sound_disable_music(_disable: bool) {
    // 空实现
}

/// 获取或设置音乐音量
pub fn sound_get_or_set_music_volume(_volume: i32) -> i32 {
    0
}

/// 获取或设置音效音量
pub fn sound_get_or_set_sound_volume(_volume: i32) -> i32 {
    0
}

/// 静音音乐
pub fn music_mute() {
    // 空实现
}

/// 取消静音音乐
pub fn music_unmute() {
    // 空实现
}

/// 获取关卡音乐
pub fn get_level_music(_dungeon_type: DungeonType) -> MusicId {
    MusicId::Town
}

// Getters/Setters for global state
pub fn is_snd_inited() -> bool {
    GB_SND_INITED.load(Ordering::Relaxed)
}

pub fn is_music_on() -> bool {
    GB_MUSIC_ON.load(Ordering::Relaxed)
}

pub fn is_sound_on() -> bool {
    GB_SOUND_ON.load(Ordering::Relaxed)
}

pub fn get_music_track() -> MusicId {
    SGN_MUSIC_TRACK.load(Ordering::Relaxed).into()
}
