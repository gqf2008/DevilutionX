//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Load File - 文件加载工具
//!
//! 移植自 Source/engine/load_file.hpp
//!
//! 通过资产系统加载文件到内存

use crate::engine::assets::{open_asset, find_asset, AssetError};

/// 加载文件到内存（带状态）
///
/// C++ API: `tl::expected<std::unique_ptr<T[]>, std::string> LoadFileInMemWithStatus(const char *path, std::size_t *numRead)`
pub fn load_file_in_mem_with_status(path: &str) -> Result<Vec<u8>, String> {
    let asset_ref = find_asset(path)
        .ok_or_else(|| format!("File not found: {}", path))?;
    
    let mut handle = open_asset(asset_ref)
        .map_err(|e| format!("Failed to open {}: {:?}", path, e))?;
    
    let data = handle.read_all();
    if data.is_empty() {
        return Err(format!("Failed to read file: {}", path));
    }
    
    Ok(data)
}

/// 加载文件到内存（失败时 panic）
///
/// C++ API: `std::unique_ptr<T[]> LoadFileInMem(const char *path, std::size_t *numRead)`
pub fn load_file_in_mem(path: &str) -> Vec<u8> {
    load_file_in_mem_with_status(path)
        .unwrap_or_else(|e| panic!("{}", e))
}

/// 可选加载文件到内存
///
/// C++ API: `bool LoadOptionalFileInMem(const char *path, T *data, std::size_t count)`
pub fn load_optional_file_in_mem(path: &str) -> Option<Vec<u8>> {
    load_file_in_mem_with_status(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_file_in_mem_with_status("nonexistent_file.dat");
        assert!(result.is_err());
    }
}

