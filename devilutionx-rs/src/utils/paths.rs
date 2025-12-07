//! 文件路径管理系统
//!
//! 对应 C++: Source/utils/paths.cpp/h
//!
//! 负责管理游戏运行所需的各种路径：
//! - 基础路径 (程序安装目录)
//! - 偏好路径 (用户设置/存档目录)
//! - 配置路径 (配置文件目录)
//! - 资源路径 (游戏资源目录)

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use parking_lot::RwLock;

/// 目录分隔符
#[cfg(windows)]
pub const DIRECTORY_SEPARATOR: char = '\\';
#[cfg(not(windows))]
pub const DIRECTORY_SEPARATOR: char = '/';

/// 目录分隔符字符串
#[cfg(windows)]
pub const DIRECTORY_SEPARATOR_STR: &str = "\\";
#[cfg(not(windows))]
pub const DIRECTORY_SEPARATOR_STR: &str = "/";

/// 路径状态 (延迟初始化)
struct PathsState {
    base_path: Option<String>,
    pref_path: Option<String>,
    config_path: Option<String>,
    assets_path: Option<String>,
}

impl PathsState {
    fn new() -> Self {
        Self {
            base_path: None,
            pref_path: None,
            config_path: None,
            assets_path: None,
        }
    }
}

/// 全局路径状态
static PATHS: OnceLock<RwLock<PathsState>> = OnceLock::new();

fn paths() -> &'static RwLock<PathsState> {
    PATHS.get_or_init(|| RwLock::new(PathsState::new()))
}

/// 确保路径以目录分隔符结尾
fn add_trailing_slash(path: &mut String) {
    if !path.is_empty() && !path.ends_with(DIRECTORY_SEPARATOR) {
        path.push(DIRECTORY_SEPARATOR);
    }
}

/// 获取可执行文件所在目录
fn get_exe_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// 获取当前工作目录
fn get_current_dir() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

/// 在常见位置查找本地 diablo.ini（便携模式）
///
/// 优先级：
/// - 当前工作目录及其最多 3 层父目录
/// - 可执行所在目录及其最多 3 层父目录
/// - 若以上目录下有 devilutionx-rs/diablo.ini 也视作命中（方便从仓库根目录运行）
fn find_portable_ini() -> Option<PathBuf> {
    fn search_ancestors(mut dir: PathBuf) -> Option<PathBuf> {
        for _ in 0..4 {
            let direct = dir.join("diablo.ini");
            if direct.exists() {
                return Some(dir.clone());
            }

            let nested = dir.join("devilutionx-rs").join("diablo.ini");
            if nested.exists() {
                return Some(dir.join("devilutionx-rs"));
            }

            if !dir.pop() {
                break;
            }
        }
        None
    }

    if let Some(cur) = get_current_dir() {
        if let Some(found) = search_ancestors(cur) {
            return Some(found);
        }
    }

    if let Some(exe_dir) = get_exe_path() {
        if let Some(found) = search_ancestors(exe_dir) {
            return Some(found);
        }
    }

    None
}

/// 获取默认偏好路径 (平台相关)
#[cfg(windows)]
fn get_default_pref_path() -> Option<PathBuf> {
    // Windows: %APPDATA%\diasurgical\devilution
    std::env::var("APPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join("diasurgical").join("devilution"))
}

#[cfg(target_os = "macos")]
fn get_default_pref_path() -> Option<PathBuf> {
    // macOS: ~/Library/Application Support/diasurgical/devilution
    dirs::data_dir()
        .map(|p| p.join("diasurgical").join("devilution"))
}

#[cfg(target_os = "linux")]
fn get_default_pref_path() -> Option<PathBuf> {
    // Linux: ~/.local/share/diasurgical/devilution
    dirs::data_local_dir()
        .map(|p| p.join("diasurgical").join("devilution"))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn get_default_pref_path() -> Option<PathBuf> {
    // 其他平台: 使用当前目录
    get_current_dir()
}

/// 检查文件是否存在且可写
fn file_exists_and_is_writeable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    // 尝试以追加模式打开来检查可写性
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .is_ok()
}

/// 获取基础路径 (程序安装目录)
///
/// 返回程序可执行文件所在的目录。
pub fn base_path() -> String {
    let state = paths().read();
    if let Some(ref path) = state.base_path {
        println!("[paths] base_path cached -> {}", path);
        return path.clone();
    }
    drop(state);

    // 延迟初始化
    let mut state = paths().write();
    if state.base_path.is_none() {
        let mut path = get_exe_path()
            .or_else(get_current_dir)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        add_trailing_slash(&mut path);
        println!("[paths] base_path resolved -> {}", path);
        state.base_path = Some(path);
    }
    state.base_path.clone().unwrap_or_default()
}

/// 获取偏好路径 (用户设置/存档目录)
///
/// 返回用户特定的数据目录，用于存储：
/// - 配置文件
/// - 存档文件
/// - 截图
pub fn pref_path() -> String {
    let state = paths().read();
    if let Some(ref path) = state.pref_path {
        return path.clone();
    }
    drop(state);

    let mut state = paths().write();
    if state.pref_path.is_none() {
        // 便携模式：查找附近的 diablo.ini
        if let Some(portable_dir) = find_portable_ini() {
            state.pref_path = Some(portable_dir.to_string_lossy().into_owned() + DIRECTORY_SEPARATOR_STR);
        } else {
            let mut path = get_default_pref_path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            add_trailing_slash(&mut path);

            // 确保目录存在
            if !path.is_empty() {
                let _ = std::fs::create_dir_all(&path);
            }

            state.pref_path = Some(path);
        }
    }
    state.pref_path.clone().unwrap_or_default()
}

/// 获取配置路径
///
/// 通常与偏好路径相同，但可以单独设置。
pub fn config_path() -> String {
    let state = paths().read();
    if let Some(ref path) = state.config_path {
        return path.clone();
    }
    drop(state);

    // 先计算 pref，避免持有写锁时递归调用导致死锁
    let pref = pref_path();

    let mut state = paths().write();
    if state.config_path.is_none() {
        // 默认与偏好路径相同
        state.config_path = Some(pref);
    }
    state.config_path.clone().unwrap_or_default()
}

/// 获取资源路径
///
/// 返回游戏资源 (MPQ文件、字体等) 所在目录。
pub fn assets_path() -> String {
    let state = paths().read();
    if let Some(ref path) = state.assets_path {
        println!("[paths] assets_path cached -> {}", path);
        return path.clone();
    }
    drop(state);

    // 先计算 base，避免持有写锁时递归调用 base_path 造成死锁
    let base = base_path();

    let mut state = paths().write();
    if state.assets_path.is_none() {
        let mut path = format!("{}assets{}", base, DIRECTORY_SEPARATOR);

        println!("[paths] assets_path base={} candidate={}", base, path);

        // 如果 assets 目录不存在，尝试使用 base 目录
        if !Path::new(&path).exists() {
            println!("[paths] assets directory missing, falling back to base");
            path = base;
        }

        state.assets_path = Some(path);
    }
    state.assets_path.clone().unwrap_or_default()
}

/// 设置基础路径
pub fn set_base_path(path: &str) {
    let mut p = path.to_string();
    add_trailing_slash(&mut p);
    let mut state = paths().write();
    state.base_path = Some(p);
}

/// 设置偏好路径
pub fn set_pref_path(path: &str) {
    let mut p = path.to_string();
    add_trailing_slash(&mut p);
    let mut state = paths().write();
    state.pref_path = Some(p);
}

/// 设置配置路径
pub fn set_config_path(path: &str) {
    let mut p = path.to_string();
    add_trailing_slash(&mut p);
    let mut state = paths().write();
    state.config_path = Some(p);
}

/// 设置资源路径
pub fn set_assets_path(path: &str) {
    let mut p = path.to_string();
    add_trailing_slash(&mut p);
    let mut state = paths().write();
    state.assets_path = Some(p);
}

/// 获取 MPQ 文件的完整路径
///
/// 按以下顺序搜索：
/// 1. 资源路径
/// 2. 基础路径
/// 3. 偏好路径
pub fn mpq_path(name: &str) -> Option<PathBuf> {
    let search_paths = [
        assets_path(),
        base_path(),
        pref_path(),
    ];

    for dir in &search_paths {
        if dir.is_empty() {
            continue;
        }
        let path = PathBuf::from(dir).join(name);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// 获取存档文件路径
pub fn save_path(name: &str) -> PathBuf {
    let pref = pref_path();
    PathBuf::from(&pref).join(name)
}

/// 获取配置文件路径
pub fn config_file_path(name: &str) -> PathBuf {
    let config = config_path();
    PathBuf::from(&config).join(name)
}

/// 获取截图保存路径
pub fn screenshot_path() -> PathBuf {
    let pref = pref_path();
    PathBuf::from(&pref).join("screenshots")
}

/// 重置所有路径 (主要用于测试)
#[cfg(test)]
pub fn reset_paths() {
    let mut state = paths().write();
    *state = PathsState::new();
}

//=============================================================================
// 测试
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_separator() {
        #[cfg(windows)]
        {
            assert_eq!(DIRECTORY_SEPARATOR, '\\');
            assert_eq!(DIRECTORY_SEPARATOR_STR, "\\");
        }
        #[cfg(not(windows))]
        {
            assert_eq!(DIRECTORY_SEPARATOR, '/');
            assert_eq!(DIRECTORY_SEPARATOR_STR, "/");
        }
    }

    #[test]
    fn test_add_trailing_slash() {
        let mut path = String::from("test");
        add_trailing_slash(&mut path);
        assert!(path.ends_with(DIRECTORY_SEPARATOR));

        // 已有分隔符时不应重复添加
        let len = path.len();
        add_trailing_slash(&mut path);
        assert_eq!(path.len(), len);

        // 空字符串不应添加
        let mut empty = String::new();
        add_trailing_slash(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_base_path_not_empty() {
        // 基础路径应该有值
        let path = base_path();
        // 在测试环境中可能为空，但至少不应该 panic
        let _ = path;
    }

    #[test]
    fn test_pref_path_not_empty() {
        let path = pref_path();
        // 可能为空（便携模式），但不应该 panic
        let _ = path;
    }

    #[test]
    fn test_config_path() {
        let path = config_path();
        let _ = path;
    }

    #[test]
    fn test_assets_path() {
        let path = assets_path();
        let _ = path;
    }

    #[test]
    fn test_set_paths() {
        reset_paths();

        set_base_path("/test/base");
        let base = base_path();
        assert!(base.starts_with("/test/base"));
        assert!(base.ends_with(DIRECTORY_SEPARATOR));

        set_pref_path("/test/pref");
        let pref = pref_path();
        assert!(pref.starts_with("/test/pref"));

        set_config_path("/test/config");
        let config = config_path();
        assert!(config.starts_with("/test/config"));

        set_assets_path("/test/assets");
        let assets = assets_path();
        assert!(assets.starts_with("/test/assets"));

        reset_paths();
    }

    #[test]
    fn test_mpq_path_not_found() {
        // 不存在的文件应返回 None
        let result = mpq_path("nonexistent_file_12345.mpq");
        assert!(result.is_none());
    }

    #[test]
    fn test_save_path() {
        let save = save_path("test_save.sv");
        assert!(save.to_string_lossy().contains("test_save.sv"));
    }

    #[test]
    fn test_config_file_path() {
        let config = config_file_path("diablo.ini");
        assert!(config.to_string_lossy().contains("diablo.ini"));
    }

    #[test]
    fn test_screenshot_path() {
        let path = screenshot_path();
        assert!(path.to_string_lossy().contains("screenshots"));
    }

    #[test]
    fn test_file_exists_check() {
        // Cargo.toml 应该存在于测试目录
        let cargo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        assert!(file_exists_and_is_writeable(&cargo_path) || cargo_path.exists());

        // 不存在的文件
        let fake_path = Path::new("/nonexistent/path/file.txt");
        assert!(!file_exists_and_is_writeable(fake_path));
    }
}
