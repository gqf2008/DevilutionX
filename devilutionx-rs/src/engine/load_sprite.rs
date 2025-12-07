//! 精灵资源加载器 - 移植自 Source/engine/load_cel.hpp 和 load_cl2.hpp
//!
//! 提供从文件加载 CEL/CL2/CLX 格式精灵的功能

use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;

use super::load_clx::ClxFileInfo;

/// 精灵加载错误
#[derive(Debug, Clone)]
pub enum SpriteLoadError {
    /// 文件未找到
    FileNotFound(String),
    /// IO 错误
    IoError(String),
    /// 格式错误
    InvalidFormat(String),
    /// 数据太短
    DataTooShort,
    /// 不支持的格式
    UnsupportedFormat(String),
}

impl std::fmt::Display for SpriteLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpriteLoadError::FileNotFound(path) => write!(f, "File not found: {}", path),
            SpriteLoadError::IoError(msg) => write!(f, "IO error: {}", msg),
            SpriteLoadError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            SpriteLoadError::DataTooShort => write!(f, "Data too short"),
            SpriteLoadError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
        }
    }
}

impl std::error::Error for SpriteLoadError {}

impl From<io::Error> for SpriteLoadError {
    fn from(err: io::Error) -> Self {
        SpriteLoadError::IoError(err.to_string())
    }
}

/// 资源加载器 trait
///
/// 用于抽象化文件系统访问（支持 MPQ 或普通文件）
pub trait AssetLoader {
    /// 加载资源数据
    fn load(&self, path: &str) -> Result<Vec<u8>, SpriteLoadError>;

    /// 检查资源是否存在
    fn exists(&self, path: &str) -> bool;
}

/// 文件系统资源加载器
pub struct FileSystemLoader {
    /// 基础路径
    base_path: String,
}

impl FileSystemLoader {
    /// 创建新的文件系统加载器
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

impl AssetLoader for FileSystemLoader {
    fn load(&self, path: &str) -> Result<Vec<u8>, SpriteLoadError> {
        let full_path = Path::new(&self.base_path).join(path);
        let mut file = std::fs::File::open(&full_path)
            .map_err(|_| SpriteLoadError::FileNotFound(full_path.display().to_string()))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(&self.base_path).join(path).exists()
    }
}

/// 内存资源加载器（用于测试）
pub struct MemoryLoader {
    assets: HashMap<String, Vec<u8>>,
}

impl MemoryLoader {
    /// 创建新的内存加载器
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    /// 添加资源
    pub fn add_asset(&mut self, path: impl Into<String>, data: Vec<u8>) {
        self.assets.insert(path.into(), data);
    }
}

impl Default for MemoryLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader for MemoryLoader {
    fn load(&self, path: &str) -> Result<Vec<u8>, SpriteLoadError> {
        self.assets
            .get(path)
            .cloned()
            .ok_or_else(|| SpriteLoadError::FileNotFound(path.to_string()))
    }

    fn exists(&self, path: &str) -> bool {
        self.assets.contains_key(path)
    }
}

/// 精灵宽度规格
#[derive(Clone)]
pub enum SpriteWidths {
    /// 单一宽度（所有帧相同）
    Single(u16),
    /// 每帧独立宽度
    PerFrame(Vec<u16>),
}

impl From<u16> for SpriteWidths {
    fn from(width: u16) -> Self {
        SpriteWidths::Single(width)
    }
}

impl From<&[u16]> for SpriteWidths {
    fn from(widths: &[u16]) -> Self {
        SpriteWidths::PerFrame(widths.to_vec())
    }
}

/// CEL 文件扩展名
pub const CEL_EXT: &str = ".cel";

/// CL2 文件扩展名
pub const CL2_EXT: &str = ".cl2";

/// CLX 文件扩展名
pub const CLX_EXT: &str = ".clx";

/// 加载 CEL 精灵列表
///
/// CEL 是原版 Diablo 的精灵格式
pub fn load_cel<L: AssetLoader>(
    loader: &L,
    path: &str,
    widths: impl Into<SpriteWidths>,
) -> Result<ClxFileInfo, SpriteLoadError> {
    let data = loader.load(path)?;
    parse_cel_data(&data, widths.into())
}

/// 加载可选的 CEL 精灵列表
///
/// 如果文件不存在返回 None
pub fn load_optional_cel<L: AssetLoader>(
    loader: &L,
    path: &str,
    widths: impl Into<SpriteWidths>,
) -> Option<ClxFileInfo> {
    load_cel(loader, path, widths).ok()
}

/// 加载 CL2 精灵列表
///
/// CL2 是原版 Diablo 的动画精灵格式
pub fn load_cl2<L: AssetLoader>(
    loader: &L,
    path: &str,
    widths: impl Into<SpriteWidths>,
) -> Result<ClxFileInfo, SpriteLoadError> {
    let data = loader.load(path)?;
    parse_cl2_data(&data, widths.into())
}

/// 加载 CLX 精灵列表
///
/// CLX 是 DevilutionX 的运行时格式
pub fn load_clx<L: AssetLoader>(
    loader: &L,
    path: &str,
) -> Result<ClxFileInfo, SpriteLoadError> {
    let data = loader.load(path)?;
    super::load_clx::parse_clx_data(data)
        .map_err(|e| SpriteLoadError::InvalidFormat(e.to_string()))
}

/// 解析 CEL 数据
///
/// CEL 格式：
/// - 4 字节: 帧数
/// - (帧数+1) * 4 字节: 帧偏移表
/// - 像素数据
fn parse_cel_data(_data: &[u8], _widths: SpriteWidths) -> Result<ClxFileInfo, SpriteLoadError> {
    // CEL -> CLX 转换是一个复杂的过程
    // 原始 C++ 代码在 utils/cel_to_clx.cpp
    // 这里提供一个占位实现
    Err(SpriteLoadError::UnsupportedFormat(
        "CEL to CLX conversion not yet implemented".to_string()
    ))
}

/// 解析 CL2 数据
///
/// CL2 格式类似 CEL，但有额外的方向/组头
fn parse_cl2_data(_data: &[u8], _widths: SpriteWidths) -> Result<ClxFileInfo, SpriteLoadError> {
    // CL2 -> CLX 转换
    // 原始 C++ 代码在 utils/cl2_to_clx.cpp
    // 这里提供一个占位实现
    Err(SpriteLoadError::UnsupportedFormat(
        "CL2 to CLX conversion not yet implemented".to_string()
    ))
}

/// 精灵缓存
///
/// 缓存已加载的精灵以避免重复加载
pub struct SpriteCache {
    cache: HashMap<String, ClxFileInfo>,
}

impl SpriteCache {
    /// 创建新的精灵缓存
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// 获取或加载精灵
    pub fn get_or_load<L: AssetLoader>(
        &mut self,
        loader: &L,
        path: &str,
    ) -> Result<&ClxFileInfo, SpriteLoadError> {
        if !self.cache.contains_key(path) {
            let sprites = load_clx(loader, path)?;
            self.cache.insert(path.to_string(), sprites);
        }
        Ok(self.cache.get(path).unwrap())
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 移除特定资源
    pub fn remove(&mut self, path: &str) -> Option<ClxFileInfo> {
        self.cache.remove(path)
    }

    /// 缓存大小
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for SpriteCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_loader() {
        let mut loader = MemoryLoader::new();
        loader.add_asset("test.clx", vec![0x01, 0x02, 0x03]);

        assert!(loader.exists("test.clx"));
        assert!(!loader.exists("missing.clx"));

        let data = loader.load("test.clx").unwrap();
        assert_eq!(data, vec![0x01, 0x02, 0x03]);

        let err = loader.load("missing.clx").unwrap_err();
        assert!(matches!(err, SpriteLoadError::FileNotFound(_)));
    }

    #[test]
    fn test_sprite_widths() {
        let single: SpriteWidths = 96u16.into();
        assert!(matches!(single, SpriteWidths::Single(96)));

        let widths = [32u16, 64, 96];
        let per_frame: SpriteWidths = widths.as_slice().into();
        assert!(matches!(per_frame, SpriteWidths::PerFrame(_)));
    }

    #[test]
    fn test_sprite_cache() {
        let cache = SpriteCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_sprite_load_error_display() {
        let err = SpriteLoadError::FileNotFound("test.clx".to_string());
        assert!(err.to_string().contains("test.clx"));

        let err = SpriteLoadError::DataTooShort;
        assert!(err.to_string().contains("too short"));
    }
}
