//! 游戏配置选项系统
//!
//! 对应 C++: Source/options.cpp/h
//!
//! 该模块提供游戏的所有可配置选项，包括：
//! - 启动选项 (游戏模式、共享版等)
//! - 图形选项 (分辨率、全屏、亮度等)
//! - 音频选项 (音量、采样率等)
//! - 游戏选项 (自动拾取、显示设置等)
//! - 控制器选项
//! - 网络选项

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use parking_lot::RwLock;

//=============================================================================
// 枚举类型
//=============================================================================

/// 启动时的游戏模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum StartUpGameMode {
    /// 询问用户
    #[default]
    Ask = 0,
    /// 地狱火
    Hellfire = 1,
    /// 暗黑
    Diablo = 2,
}

/// 启动时是否播放介绍视频
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum StartUpIntro {
    /// 关闭
    Off = 0,
    /// 仅一次
    #[default]
    Once = 1,
    /// 每次启动
    On = 2,
}

/// 启动画面类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum StartUpSplash {
    /// 无启动画面
    None = 0,
    /// 仅标题对话框
    TitleDialog = 1,
    /// Logo 和标题对话框
    #[default]
    LogoAndTitleDialog = 2,
}

/// 缩放质量
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum ScalingQuality {
    /// 最近邻像素
    #[default]
    NearestPixel = 0,
    /// 双线性过滤
    BilinearFiltering = 1,
    /// 各向异性过滤
    AnisotropicFiltering = 2,
}

/// 帧率控制方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum FrameRateControl {
    /// 无限制
    None = 0,
    /// 垂直同步
    #[default]
    VerticalSync = 1,
    /// CPU睡眠
    CPUSleep = 2,
}

/// 浮动数字显示方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum FloatingNumbers {
    /// 关闭
    #[default]
    Off = 0,
    /// 随机角度
    Random = 1,
    /// 仅垂直
    Vertical = 2,
}

//=============================================================================
// 选项结构体
//=============================================================================

/// 游戏模式选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameModeOptions {
    /// 游戏模式
    pub game_mode: StartUpGameMode,
    /// 是否共享版
    pub shareware: bool,
}

impl Default for GameModeOptions {
    fn default() -> Self {
        Self {
            game_mode: StartUpGameMode::Ask,
            shareware: false,
        }
    }
}

/// 启动选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartUpOptions {
    /// Diablo 介绍视频
    pub diablo_intro: StartUpIntro,
    /// Hellfire 介绍视频
    pub hellfire_intro: StartUpIntro,
    /// 启动画面
    pub splash: StartUpSplash,
}

impl Default for StartUpOptions {
    fn default() -> Self {
        Self {
            diablo_intro: StartUpIntro::Once,
            hellfire_intro: StartUpIntro::Once,
            splash: StartUpSplash::LogoAndTitleDialog,
        }
    }
}

/// 音频选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOptions {
    /// 音效音量 (0-100)
    pub sound_volume: i32,
    /// 音乐音量 (0-100)
    pub music_volume: i32,
    /// 行走时发出声音
    pub walking_sound: bool,
    /// 自动装备时播放装备音效
    pub auto_equip_sound: bool,
    /// 拾取物品时播放拾取音效
    pub item_pickup_sound: bool,
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 声道数 (1 或 2)
    pub channels: u8,
    /// 缓冲区大小
    pub buffer_size: u32,
}

impl Default for AudioOptions {
    fn default() -> Self {
        Self {
            sound_volume: 100,
            music_volume: 100,
            walking_sound: true,
            auto_equip_sound: false,
            item_pickup_sound: false,
            sample_rate: 22050,
            channels: 2,
            buffer_size: 2048,
        }
    }
}

/// 图形选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsOptions {
    /// 分辨率宽度
    pub width: u32,
    /// 分辨率高度
    pub height: u32,
    /// 全屏模式
    pub fullscreen: bool,
    /// 适应屏幕
    pub fit_to_screen: bool,
    /// 放大渲染
    pub upscale: bool,
    /// 缩放质量
    pub scale_quality: ScalingQuality,
    /// 整数缩放
    pub integer_scaling: bool,
    /// 帧率控制
    pub frame_rate_control: FrameRateControl,
    /// 亮度 (0-100)
    pub brightness: i32,
    /// 启动时缩放
    pub zoom: bool,
    /// 逐像素光照
    pub per_pixel_lighting: bool,
    /// 颜色循环动画
    pub color_cycling: bool,
    /// 硬件光标
    pub hardware_cursor: bool,
    /// 物品使用硬件光标
    pub hardware_cursor_for_items: bool,
    /// 硬件光标最大尺寸
    pub hardware_cursor_max_size: i32,
    /// 显示 FPS
    pub show_fps: bool,
}

impl Default for GraphicsOptions {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fullscreen: false,
            fit_to_screen: true,
            upscale: true,
            scale_quality: ScalingQuality::NearestPixel,
            integer_scaling: false,
            frame_rate_control: FrameRateControl::VerticalSync,
            brightness: 50,
            zoom: false,
            per_pixel_lighting: true,
            color_cycling: true,
            hardware_cursor: true,
            hardware_cursor_for_items: false,
            hardware_cursor_max_size: 128,
            show_fps: false,
        }
    }
}

/// 游戏选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayOptions {
    /// 游戏速度 (ticks per second)
    pub tick_rate: i32,
    /// 在城镇中奔跑
    pub run_in_town: bool,
    /// 捕获输入
    pub grab_input: bool,
    /// 失去焦点时暂停
    pub pause_on_focus_loss: bool,
    /// Theo 任务
    pub theo_quest: bool,
    /// 奶牛任务
    pub cow_quest: bool,
    /// 友军伤害
    pub friendly_fire: bool,
    /// 经验条
    pub experience_bar: bool,
    /// 敌人血条
    pub enemy_health_bar: bool,
    /// 自动拾取金币
    pub auto_gold_pickup: bool,
    /// 自动拾取药水
    pub auto_elixir_pickup: bool,
    /// 城镇自动拾取
    pub auto_pickup_in_town: bool,
    /// Adria 回复魔法
    pub adria_refills_mana: bool,
    /// 自动装备武器
    pub auto_equip_weapons: bool,
    /// 自动装备护甲
    pub auto_equip_armor: bool,
    /// 自动装备头盔
    pub auto_equip_helms: bool,
    /// 自动装备盾牌
    pub auto_equip_shields: bool,
    /// 自动装备首饰
    pub auto_equip_jewelry: bool,
    /// 随机任务
    pub randomize_quests: bool,
    /// 显示怪物类型
    pub show_monster_type: bool,
    /// 显示物品标签
    pub show_item_labels: bool,
    /// 自动补充腰带
    pub auto_refill_belt: bool,
    /// 禁用致残神殿
    pub disable_crippling_shrines: bool,
    /// 快速施法
    pub quick_cast: bool,
    /// 浮动数字
    pub floating_numbers: FloatingNumbers,
    /// 显示生命值
    pub show_health_values: bool,
    /// 显示魔法值
    pub show_mana_values: bool,
}

impl Default for GameplayOptions {
    fn default() -> Self {
        Self {
            tick_rate: 20,
            run_in_town: false,
            grab_input: false,
            pause_on_focus_loss: true,
            theo_quest: false,
            cow_quest: false,
            friendly_fire: true,
            experience_bar: false,
            enemy_health_bar: false,
            auto_gold_pickup: false,
            auto_elixir_pickup: false,
            auto_pickup_in_town: false,
            adria_refills_mana: false,
            auto_equip_weapons: true,
            auto_equip_armor: false,
            auto_equip_helms: false,
            auto_equip_shields: false,
            auto_equip_jewelry: false,
            randomize_quests: false,
            show_monster_type: false,
            show_item_labels: false,
            auto_refill_belt: false,
            disable_crippling_shrines: false,
            quick_cast: false,
            floating_numbers: FloatingNumbers::Off,
            show_health_values: false,
            show_mana_values: false,
        }
    }
}

/// 控制器选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerOptions {
    /// 启用控制器
    pub enabled: bool,
    /// 摇杆死区
    pub deadzone: f32,
    /// 震动反馈
    pub rumble: bool,
}

impl Default for ControllerOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            deadzone: 0.07,
            rumble: true,
        }
    }
}

/// 网络选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptions {
    /// 端口
    pub port: u16,
    /// 最大玩家数
    pub max_players: u8,
    /// 超时时间 (毫秒)
    pub timeout_ms: u32,
}

impl Default for NetworkOptions {
    fn default() -> Self {
        Self {
            port: 6112,
            max_players: 4,
            timeout_ms: 5000,
        }
    }
}

/// 语言选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageOptions {
    /// 语言代码 (ISO-15897)
    pub code: String,
}

impl Default for LanguageOptions {
    fn default() -> Self {
        Self {
            code: "en".to_string(),
        }
    }
}

//=============================================================================
// 主配置结构
//=============================================================================

/// 游戏选项配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Options {
    pub game_mode: GameModeOptions,
    pub startup: StartUpOptions,
    pub audio: AudioOptions,
    pub graphics: GraphicsOptions,
    pub gameplay: GameplayOptions,
    pub controller: ControllerOptions,
    pub network: NetworkOptions,
    pub language: LanguageOptions,
}

impl Options {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 INI 文件加载
    pub fn load_from_file(path: &Path) -> Result<Self, OptionsError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| OptionsError::IoError(e.to_string()))?;

        // 简单的 INI 解析
        let mut options = Self::default();
        options.parse_ini(&content)?;
        Ok(options)
    }

    /// 保存到 INI 文件
    pub fn save_to_file(&self, path: &Path) -> Result<(), OptionsError> {
        let content = self.to_ini();
        std::fs::write(path, content)
            .map_err(|e| OptionsError::IoError(e.to_string()))?;
        Ok(())
    }

    /// 解析 INI 格式
    fn parse_ini(&mut self, content: &str) -> Result<(), OptionsError> {
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // 检测 section
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_lowercase();
                continue;
            }

            // 解析键值对
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                self.set_value(&current_section, &key, value);
            }
        }

        Ok(())
    }

    /// 设置配置值
    fn set_value(&mut self, section: &str, key: &str, value: &str) {
        match section {
            "audio" => match key {
                "sound volume" | "soundvolume" => {
                    if let Ok(v) = value.parse() {
                        self.audio.sound_volume = v;
                    }
                }
                "music volume" | "musicvolume" => {
                    if let Ok(v) = value.parse() {
                        self.audio.music_volume = v;
                    }
                }
                "walking sound" | "walkingsound" => {
                    self.audio.walking_sound = parse_bool(value);
                }
                "sample rate" | "samplerate" => {
                    if let Ok(v) = value.parse() {
                        self.audio.sample_rate = v;
                    }
                }
                "channels" => {
                    if let Ok(v) = value.parse() {
                        self.audio.channels = v;
                    }
                }
                _ => {}
            }
            "graphics" => match key {
                "width" => {
                    if let Ok(v) = value.parse() {
                        self.graphics.width = v;
                    }
                }
                "height" => {
                    if let Ok(v) = value.parse() {
                        self.graphics.height = v;
                    }
                }
                "fullscreen" => {
                    self.graphics.fullscreen = parse_bool(value);
                }
                "fit to screen" | "fittoscreen" => {
                    self.graphics.fit_to_screen = parse_bool(value);
                }
                "upscale" => {
                    self.graphics.upscale = parse_bool(value);
                }
                "integer scaling" | "integerscaling" => {
                    self.graphics.integer_scaling = parse_bool(value);
                }
                "brightness" | "gamma" => {
                    if let Ok(v) = value.parse() {
                        self.graphics.brightness = v;
                    }
                }
                "show fps" | "showfps" => {
                    self.graphics.show_fps = parse_bool(value);
                }
                _ => {}
            }
            "gameplay" => match key {
                "speed" | "tickrate" | "tick rate" => {
                    if let Ok(v) = value.parse() {
                        self.gameplay.tick_rate = v;
                    }
                }
                "run in town" | "runintown" => {
                    self.gameplay.run_in_town = parse_bool(value);
                }
                "grab input" | "grabinput" => {
                    self.gameplay.grab_input = parse_bool(value);
                }
                "friendly fire" | "friendlyfire" => {
                    self.gameplay.friendly_fire = parse_bool(value);
                }
                "experience bar" | "experiencebar" => {
                    self.gameplay.experience_bar = parse_bool(value);
                }
                "enemy health bar" | "enemyhealthbar" => {
                    self.gameplay.enemy_health_bar = parse_bool(value);
                }
                "auto gold pickup" | "autogoldpickup" => {
                    self.gameplay.auto_gold_pickup = parse_bool(value);
                }
                "auto equip weapons" | "autoequipweapons" => {
                    self.gameplay.auto_equip_weapons = parse_bool(value);
                }
                "show item labels" | "showitemlabels" => {
                    self.gameplay.show_item_labels = parse_bool(value);
                }
                "quick cast" | "quickcast" => {
                    self.gameplay.quick_cast = parse_bool(value);
                }
                _ => {}
            }
            "network" => match key {
                "port" => {
                    if let Ok(v) = value.parse() {
                        self.network.port = v;
                    }
                }
                "max players" | "maxplayers" => {
                    if let Ok(v) = value.parse() {
                        self.network.max_players = v;
                    }
                }
                _ => {}
            }
            "language" => match key {
                "code" => {
                    self.language.code = value.to_string();
                }
                _ => {}
            }
            _ => {}
        }
    }

    /// 转换为 INI 格式
    fn to_ini(&self) -> String {
        let mut s = String::new();

        // Audio
        s.push_str("[Audio]\n");
        s.push_str(&format!("Sound Volume={}\n", self.audio.sound_volume));
        s.push_str(&format!("Music Volume={}\n", self.audio.music_volume));
        s.push_str(&format!("Walking Sound={}\n", bool_to_str(self.audio.walking_sound)));
        s.push_str(&format!("Sample Rate={}\n", self.audio.sample_rate));
        s.push_str(&format!("Channels={}\n", self.audio.channels));
        s.push('\n');

        // Graphics
        s.push_str("[Graphics]\n");
        s.push_str(&format!("Width={}\n", self.graphics.width));
        s.push_str(&format!("Height={}\n", self.graphics.height));
        s.push_str(&format!("Fullscreen={}\n", bool_to_str(self.graphics.fullscreen)));
        s.push_str(&format!("Fit To Screen={}\n", bool_to_str(self.graphics.fit_to_screen)));
        s.push_str(&format!("Upscale={}\n", bool_to_str(self.graphics.upscale)));
        s.push_str(&format!("Integer Scaling={}\n", bool_to_str(self.graphics.integer_scaling)));
        s.push_str(&format!("Brightness={}\n", self.graphics.brightness));
        s.push_str(&format!("Show FPS={}\n", bool_to_str(self.graphics.show_fps)));
        s.push('\n');

        // Gameplay
        s.push_str("[Gameplay]\n");
        s.push_str(&format!("Speed={}\n", self.gameplay.tick_rate));
        s.push_str(&format!("Run In Town={}\n", bool_to_str(self.gameplay.run_in_town)));
        s.push_str(&format!("Grab Input={}\n", bool_to_str(self.gameplay.grab_input)));
        s.push_str(&format!("Friendly Fire={}\n", bool_to_str(self.gameplay.friendly_fire)));
        s.push_str(&format!("Experience Bar={}\n", bool_to_str(self.gameplay.experience_bar)));
        s.push_str(&format!("Enemy Health Bar={}\n", bool_to_str(self.gameplay.enemy_health_bar)));
        s.push_str(&format!("Auto Gold Pickup={}\n", bool_to_str(self.gameplay.auto_gold_pickup)));
        s.push_str(&format!("Auto Equip Weapons={}\n", bool_to_str(self.gameplay.auto_equip_weapons)));
        s.push_str(&format!("Show Item Labels={}\n", bool_to_str(self.gameplay.show_item_labels)));
        s.push_str(&format!("Quick Cast={}\n", bool_to_str(self.gameplay.quick_cast)));
        s.push('\n');

        // Network
        s.push_str("[Network]\n");
        s.push_str(&format!("Port={}\n", self.network.port));
        s.push_str(&format!("Max Players={}\n", self.network.max_players));
        s.push('\n');

        // Language
        s.push_str("[Language]\n");
        s.push_str(&format!("Code={}\n", self.language.code));

        s
    }
}

/// 解析布尔值
fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

/// 布尔值转字符串
fn bool_to_str(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

//=============================================================================
// 错误类型
//=============================================================================

/// 配置错误
#[derive(Debug)]
pub enum OptionsError {
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for OptionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionsError::IoError(e) => write!(f, "IO error: {}", e),
            OptionsError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for OptionsError {}

//=============================================================================
// 全局选项单例
//=============================================================================

/// 全局选项实例
static OPTIONS: OnceLock<RwLock<Options>> = OnceLock::new();

/// 获取全局选项 (只读)
pub fn options() -> parking_lot::RwLockReadGuard<'static, Options> {
    OPTIONS.get_or_init(|| RwLock::new(Options::default())).read()
}

/// 获取全局选项 (可写)
pub fn options_mut() -> parking_lot::RwLockWriteGuard<'static, Options> {
    OPTIONS.get_or_init(|| RwLock::new(Options::default())).write()
}

/// 加载全局选项
pub fn load_options(path: &Path) -> Result<(), OptionsError> {
    let opts = Options::load_from_file(path)?;
    let mut global = options_mut();
    *global = opts;
    Ok(())
}

/// 保存全局选项
pub fn save_options(path: &Path) -> Result<(), OptionsError> {
    let opts = options();
    opts.save_to_file(path)
}

//=============================================================================
// 测试
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = Options::default();
        assert_eq!(opts.graphics.width, 640);
        assert_eq!(opts.graphics.height, 480);
        assert!(!opts.graphics.fullscreen);
        assert_eq!(opts.audio.sound_volume, 100);
        assert_eq!(opts.network.port, 6112);
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("1"));
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("on"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("off"));
    }

    #[test]
    fn test_bool_to_str() {
        assert_eq!(bool_to_str(true), "1");
        assert_eq!(bool_to_str(false), "0");
    }

    #[test]
    fn test_ini_roundtrip() {
        let mut opts = Options::default();
        opts.graphics.width = 1920;
        opts.graphics.height = 1080;
        opts.graphics.fullscreen = true;
        opts.audio.sound_volume = 75;
        opts.gameplay.run_in_town = true;

        let ini = opts.to_ini();

        let mut parsed = Options::default();
        parsed.parse_ini(&ini).unwrap();

        assert_eq!(parsed.graphics.width, 1920);
        assert_eq!(parsed.graphics.height, 1080);
        assert!(parsed.graphics.fullscreen);
        assert_eq!(parsed.audio.sound_volume, 75);
        assert!(parsed.gameplay.run_in_town);
    }

    #[test]
    fn test_parse_ini_basic() {
        let ini = r#"
[Graphics]
Width=800
Height=600
Fullscreen=true

[Audio]
Sound Volume=50
Music Volume=80
"#;

        let mut opts = Options::default();
        opts.parse_ini(ini).unwrap();

        assert_eq!(opts.graphics.width, 800);
        assert_eq!(opts.graphics.height, 600);
        assert!(opts.graphics.fullscreen);
        assert_eq!(opts.audio.sound_volume, 50);
        assert_eq!(opts.audio.music_volume, 80);
    }

    #[test]
    fn test_parse_ini_with_comments() {
        let ini = r#"
# This is a comment
; This is also a comment
[Graphics]
Width=1024
# Another comment
Height=768
"#;

        let mut opts = Options::default();
        opts.parse_ini(ini).unwrap();

        assert_eq!(opts.graphics.width, 1024);
        assert_eq!(opts.graphics.height, 768);
    }

    #[test]
    fn test_startup_game_mode() {
        assert_eq!(StartUpGameMode::Ask as u8, 0);
        assert_eq!(StartUpGameMode::Hellfire as u8, 1);
        assert_eq!(StartUpGameMode::Diablo as u8, 2);
    }

    #[test]
    fn test_scaling_quality() {
        assert_eq!(ScalingQuality::NearestPixel as u8, 0);
        assert_eq!(ScalingQuality::BilinearFiltering as u8, 1);
        assert_eq!(ScalingQuality::AnisotropicFiltering as u8, 2);
    }

    #[test]
    fn test_frame_rate_control() {
        assert_eq!(FrameRateControl::None as u8, 0);
        assert_eq!(FrameRateControl::VerticalSync as u8, 1);
        assert_eq!(FrameRateControl::CPUSleep as u8, 2);
    }

    #[test]
    fn test_controller_options_default() {
        let opts = ControllerOptions::default();
        assert!(opts.enabled);
        assert_eq!(opts.deadzone, 0.07);
        assert!(opts.rumble);
    }

    #[test]
    fn test_network_options_default() {
        let opts = NetworkOptions::default();
        assert_eq!(opts.port, 6112);
        assert_eq!(opts.max_players, 4);
        assert_eq!(opts.timeout_ms, 5000);
    }

    #[test]
    fn test_to_ini_format() {
        let opts = Options::default();
        let ini = opts.to_ini();

        assert!(ini.contains("[Audio]"));
        assert!(ini.contains("[Graphics]"));
        assert!(ini.contains("[Gameplay]"));
        assert!(ini.contains("[Network]"));
        assert!(ini.contains("[Language]"));
        assert!(ini.contains("Width=640"));
        assert!(ini.contains("Height=480"));
    }
}
