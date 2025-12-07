//! 游戏初始化模块
//!
//! C++ Reference: Source/init.cpp
//!
//! 提供游戏初始化、MPQ版本检查、资源加载验证等功能

use std::path::Path;

/// MPQ 版本字符串
pub const DEVILUTIONX_MPQ_VERSION: &str = "1\n";

/// 字体版本字符串
pub const EXTRA_FONTS_VERSION: &str = "1\n";

/// 初始化错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitError {
    /// MPQ 文件未找到
    MpqNotFound(String),
    /// MPQ 版本过期
    MpqOutOfDate,
    /// 字体未找到
    FontsNotFound,
    /// 字体版本过期
    FontsOutOfDate,
    /// 数据目录不存在
    DataDirNotFound,
    /// 窗口创建失败
    WindowCreationFailed,
    /// SDL 初始化失败
    SdlInitFailed(String),
    /// 未知错误
    Unknown(String),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::MpqNotFound(name) => write!(f, "MPQ file not found: {}", name),
            InitError::MpqOutOfDate => write!(f, "DevilutionX MPQ is out of date"),
            InitError::FontsNotFound => write!(f, "Extra fonts not found"),
            InitError::FontsOutOfDate => write!(f, "Extra fonts are out of date"),
            InitError::DataDirNotFound => write!(f, "Data directory not found"),
            InitError::WindowCreationFailed => write!(f, "Failed to create window"),
            InitError::SdlInitFailed(msg) => write!(f, "SDL initialization failed: {}", msg),
            InitError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for InitError {}

/// 游戏初始化状态
///
/// **C++ Reference**: `Source/init.cpp` - various global flags
#[derive(Debug, Clone, Default)]
pub struct InitState {
    /// 游戏是否活动
    pub is_active: bool,
    /// 主 MPQ 是否已加载
    pub mpq_loaded: bool,
    /// 字体是否已加载
    pub fonts_loaded: bool,
    /// 是否检测到 Hellfire
    pub hellfire_detected: bool,
    /// 是否检测到 Spawn (Demo)
    pub spawn_detected: bool,
    /// DevilutionX MPQ 版本
    pub devilutionx_mpq_version: Option<String>,
    /// 字体版本
    pub fonts_version: Option<String>,
    /// 数据路径
    pub data_path: Option<String>,
    /// 保存路径
    pub save_path: Option<String>,
    /// 配置路径
    pub config_path: Option<String>,
}

impl InitState {
    /// 创建新的初始化状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置为活动状态
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// 检查是否准备好运行
    pub fn is_ready(&self) -> bool {
        self.mpq_loaded && self.is_active
    }

    /// 检查是否是 Hellfire 版本
    pub fn is_hellfire(&self) -> bool {
        self.hellfire_detected
    }

    /// 检查是否是 Spawn (Demo) 版本
    pub fn is_spawn(&self) -> bool {
        self.spawn_detected
    }
}

/// 检查资产内容是否匹配预期
///
/// **C++ Reference**: `Source/init.cpp:AssetContentsEq()`
pub fn asset_contents_eq(content: &[u8], expected: &str) -> bool {
    let content_str = std::str::from_utf8(content).unwrap_or("");
    content_str == expected
}

/// 检查 DevilutionX MPQ 是否过期
///
/// **C++ Reference**: `Source/init.cpp:IsDevilutionXMpqOutOfDate()`
///
/// # Arguments
/// * `version_content` - ASSETS_VERSION 文件的内容
///
/// # Returns
/// `true` 如果 MPQ 过期或无法读取
pub fn is_devilutionx_mpq_out_of_date(version_content: Option<&[u8]>) -> bool {
    match version_content {
        Some(content) => !asset_contents_eq(content, DEVILUTIONX_MPQ_VERSION),
        None => true,
    }
}

/// 检查额外字体是否过期
///
/// **C++ Reference**: `Source/init.cpp:AreExtraFontsOutOfDate()`
///
/// # Arguments
/// * `version_content` - fonts/VERSION 文件的内容
///
/// # Returns
/// `true` 如果字体过期或无法读取
pub fn are_extra_fonts_out_of_date(version_content: Option<&[u8]>) -> bool {
    match version_content {
        Some(content) => !asset_contents_eq(content, EXTRA_FONTS_VERSION),
        None => true,
    }
}

/// 检测 Hellfire 安装
///
/// **C++ Reference**: `Source/init.cpp:DiscoverLocalOreHellfire()`
///
/// # Arguments
/// * `data_path` - 数据目录路径
///
/// # Returns
/// `true` 如果检测到 Hellfire
pub fn discover_hellfire<P: AsRef<Path>>(data_path: P) -> bool {
    let hellfire_mpq = data_path.as_ref().join("hellfire.mpq");
    let hfmonk_mpq = data_path.as_ref().join("hfmonk.mpq");
    let hfmusic_mpq = data_path.as_ref().join("hfmusic.mpq");
    let hfvoice_mpq = data_path.as_ref().join("hfvoice.mpq");

    // 检查任一 Hellfire MPQ 文件是否存在
    hellfire_mpq.exists() || hfmonk_mpq.exists() || hfmusic_mpq.exists() || hfvoice_mpq.exists()
}

/// 检测 Spawn (Demo) 安装
///
/// # Arguments
/// * `data_path` - 数据目录路径
///
/// # Returns
/// `true` 如果检测到 Spawn
pub fn discover_spawn<P: AsRef<Path>>(data_path: P) -> bool {
    let spawn_mpq = data_path.as_ref().join("spawn.mpq");
    spawn_mpq.exists()
}

/// 检测 Diablo 完整版安装
///
/// # Arguments
/// * `data_path` - 数据目录路径
///
/// # Returns
/// `true` 如果检测到完整版
pub fn discover_diablo<P: AsRef<Path>>(data_path: P) -> bool {
    let diabdat_mpq = data_path.as_ref().join("diabdat.mpq");
    let diablo_mpq = data_path.as_ref().join("DIABDAT.MPQ");
    diabdat_mpq.exists() || diablo_mpq.exists()
}

/// MPQ 档案类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpqType {
    /// DevilutionX 主资源
    DevilutionX,
    /// Diablo 完整版
    Diabdat,
    /// Spawn (Demo)
    Spawn,
    /// Hellfire 主资源
    Hellfire,
    /// Hellfire Monk
    HfMonk,
    /// Hellfire 音乐
    HfMusic,
    /// Hellfire 语音
    HfVoice,
    /// 字体
    Fonts,
    /// 翻译
    Translation,
}

impl MpqType {
    /// 获取 MPQ 文件名
    pub fn filename(&self) -> &'static str {
        match self {
            MpqType::DevilutionX => "devilutionx.mpq",
            MpqType::Diabdat => "diabdat.mpq",
            MpqType::Spawn => "spawn.mpq",
            MpqType::Hellfire => "hellfire.mpq",
            MpqType::HfMonk => "hfmonk.mpq",
            MpqType::HfMusic => "hfmusic.mpq",
            MpqType::HfVoice => "hfvoice.mpq",
            MpqType::Fonts => "fonts.mpq",
            MpqType::Translation => "lang.mpq",
        }
    }

    /// 是否必须存在
    pub fn is_required(&self) -> bool {
        matches!(self, MpqType::DevilutionX | MpqType::Diabdat | MpqType::Spawn)
    }
}

/// 初始化配置
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// 数据目录
    pub data_path: String,
    /// 保存目录
    pub save_path: String,
    /// 配置目录
    pub config_path: String,
    /// 是否启用 Hellfire
    pub hellfire: bool,
    /// 是否是 Spawn 版本
    pub spawn: bool,
    /// 窗口标题
    pub window_title: String,
    /// 窗口宽度
    pub window_width: u32,
    /// 窗口高度
    pub window_height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 是否启用垂直同步
    pub vsync: bool,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            data_path: String::new(),
            save_path: String::new(),
            config_path: String::new(),
            hellfire: false,
            spawn: false,
            window_title: "DevilutionX".to_string(),
            window_width: 640,
            window_height: 480,
            fullscreen: false,
            vsync: true,
        }
    }
}

/// 初始化游戏
///
/// **C++ Reference**: `Source/init.cpp:init_archives()`
///
/// # Arguments
/// * `config` - 初始化配置
///
/// # Returns
/// 初始化状态或错误
pub fn initialize(config: &InitConfig) -> Result<InitState, InitError> {
    let mut state = InitState::new();

    // 检查数据目录
    let data_path = Path::new(&config.data_path);
    if !data_path.exists() {
        return Err(InitError::DataDirNotFound);
    }

    state.data_path = Some(config.data_path.clone());
    state.save_path = Some(config.save_path.clone());
    state.config_path = Some(config.config_path.clone());

    // 检测游戏版本
    state.hellfire_detected = discover_hellfire(data_path);
    state.spawn_detected = discover_spawn(data_path);

    // TODO: 实际加载 MPQ 档案
    // 这里只是模拟成功
    state.mpq_loaded = true;
    state.fonts_loaded = true;
    state.is_active = true;

    Ok(state)
}

/// 清理资源
///
/// **C++ Reference**: `Source/init.cpp` - cleanup functions
pub fn cleanup(state: &mut InitState) {
    state.is_active = false;
    state.mpq_loaded = false;
    state.fonts_loaded = false;
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_state_new() {
        let state = InitState::new();
        assert!(!state.is_active);
        assert!(!state.mpq_loaded);
        assert!(!state.fonts_loaded);
        assert!(!state.hellfire_detected);
        assert!(!state.spawn_detected);
    }

    #[test]
    fn test_init_state_set_active() {
        let mut state = InitState::new();
        state.set_active(true);
        assert!(state.is_active);

        state.set_active(false);
        assert!(!state.is_active);
    }

    #[test]
    fn test_init_state_is_ready() {
        let mut state = InitState::new();
        assert!(!state.is_ready());

        state.mpq_loaded = true;
        assert!(!state.is_ready());

        state.is_active = true;
        assert!(state.is_ready());
    }

    #[test]
    fn test_asset_contents_eq() {
        assert!(asset_contents_eq(b"1\n", "1\n"));
        assert!(!asset_contents_eq(b"2\n", "1\n"));
        assert!(!asset_contents_eq(b"", "1\n"));
    }

    #[test]
    fn test_is_devilutionx_mpq_out_of_date() {
        // None 应该返回 true (过期)
        assert!(is_devilutionx_mpq_out_of_date(None));

        // 正确版本应该返回 false (不过期)
        assert!(!is_devilutionx_mpq_out_of_date(Some(b"1\n")));

        // 错误版本应该返回 true (过期)
        assert!(is_devilutionx_mpq_out_of_date(Some(b"0\n")));
        assert!(is_devilutionx_mpq_out_of_date(Some(b"2\n")));
    }

    #[test]
    fn test_are_extra_fonts_out_of_date() {
        assert!(are_extra_fonts_out_of_date(None));
        assert!(!are_extra_fonts_out_of_date(Some(b"1\n")));
        assert!(are_extra_fonts_out_of_date(Some(b"0\n")));
    }

    #[test]
    fn test_mpq_type_filename() {
        assert_eq!(MpqType::DevilutionX.filename(), "devilutionx.mpq");
        assert_eq!(MpqType::Diabdat.filename(), "diabdat.mpq");
        assert_eq!(MpqType::Spawn.filename(), "spawn.mpq");
        assert_eq!(MpqType::Hellfire.filename(), "hellfire.mpq");
    }

    #[test]
    fn test_mpq_type_is_required() {
        assert!(MpqType::DevilutionX.is_required());
        assert!(MpqType::Diabdat.is_required());
        assert!(MpqType::Spawn.is_required());
        assert!(!MpqType::Hellfire.is_required());
        assert!(!MpqType::Fonts.is_required());
    }

    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert_eq!(config.window_title, "DevilutionX");
        assert_eq!(config.window_width, 640);
        assert_eq!(config.window_height, 480);
        assert!(!config.fullscreen);
        assert!(config.vsync);
    }

    #[test]
    fn test_init_error_display() {
        let err = InitError::MpqNotFound("test.mpq".to_string());
        assert!(err.to_string().contains("test.mpq"));

        let err = InitError::MpqOutOfDate;
        assert!(err.to_string().contains("out of date"));
    }

    #[test]
    fn test_cleanup() {
        let mut state = InitState::new();
        state.is_active = true;
        state.mpq_loaded = true;
        state.fonts_loaded = true;

        cleanup(&mut state);

        assert!(!state.is_active);
        assert!(!state.mpq_loaded);
        assert!(!state.fonts_loaded);
    }
}
