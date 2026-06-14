//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! CEL 文件加载 - 移植自 Source/engine/load_cel.hpp/cpp
//!
//! CEL 是原版 Diablo 的精灵格式。在 DevilutionX 中，
//! 使用解包的 MPQ 时直接加载预转换的 CLX 文件。
//!
//! 注意：Rust 版本假设使用 UNPACKED_MPQS，即 .cel 文件已预先转换为 .clx

// MAX_MPQ_PATH_SIZE from game/pfile.rs
const MAX_MPQ_PATH_SIZE: usize = 260;
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteSheet, ClxSpriteListOrSheet};
use crate::engine::load_clx::{
    load_clx_list_or_sheet_with_status, load_clx_list_or_sheet, LoadClxError,
};

/// CEL 文件扩展名（解包 MPQ 时为 .clx）
pub const DEVILUTIONX_CEL_EXT: &str = ".clx";

/// 加载 CEL 精灵列表或表（带状态）
///
/// C++ API: `tl::expected<OwnedClxSpriteListOrSheet, std::string> LoadCelListOrSheetWithStatus(...)`
pub fn load_cel_list_or_sheet_with_status(
    name: &str,
    _width_or_widths: WidthOrWidths,
) -> Result<ClxSpriteListOrSheet, String> {
    let mut path = String::with_capacity(MAX_MPQ_PATH_SIZE);
    path.push_str(name);
    path.push_str(DEVILUTIONX_CEL_EXT);
    
    // UNPACKED_MPQS: 直接加载 CLX
    load_clx_list_or_sheet_with_status(&path)
}

/// 加载 CEL 精灵列表或表
///
/// C++ API: `OwnedClxSpriteListOrSheet LoadCelListOrSheet(...)`
pub fn load_cel_list_or_sheet(
    name: &str,
    width_or_widths: WidthOrWidths,
) -> ClxSpriteListOrSheet {
    load_cel_list_or_sheet_with_status(name, width_or_widths)
        .expect("Failed to load CEL file")
}

/// 加载 CEL 精灵列表
///
/// C++ API: `OwnedClxSpriteList LoadCel(const char *pszName, uint16_t width)`
pub fn load_cel(name: &str, width: u16) -> ClxSpriteList {
    load_cel_list_or_sheet(name, WidthOrWidths::Single(width)).list()
}

/// 加载 CEL 精灵列表（带宽度数组）
///
/// C++ API: `OwnedClxSpriteList LoadCel(const char *pszName, const uint16_t *widths)`
pub fn load_cel_with_widths(name: &str, widths: &[u16]) -> ClxSpriteList {
    load_cel_list_or_sheet(name, WidthOrWidths::Multiple(widths)).list()
}

/// 加载可选 CEL 精灵列表
///
/// C++ API: `OptionalOwnedClxSpriteList LoadOptionalCel(const char *pszName, uint16_t width)`
pub fn load_optional_cel(name: &str, width: u16) -> Option<ClxSpriteList> {
    load_cel_list_or_sheet_with_status(name, WidthOrWidths::Single(width))
        .ok()
        .map(|los| los.list())
}

/// 加载可选 CEL 精灵列表（带宽度数组）
///
/// C++ API: `OptionalOwnedClxSpriteList LoadOptionalCel(const char *pszName, const uint16_t *widths)`
pub fn load_optional_cel_with_widths(name: &str, widths: &[u16]) -> Option<ClxSpriteList> {
    load_cel_list_or_sheet_with_status(name, WidthOrWidths::Multiple(widths))
        .ok()
        .map(|los| los.list())
}

/// 加载 CEL 精灵列表（带状态）
///
/// C++ API: `tl::expected<OwnedClxSpriteList, std::string> LoadCelWithStatus(...)`
pub fn load_cel_with_status(name: &str, width: u16) -> Result<ClxSpriteList, String> {
    load_cel_list_or_sheet_with_status(name, WidthOrWidths::Single(width))
        .map(|los| los.list())
}

/// 加载 CEL 精灵列表（带状态和宽度数组）
pub fn load_cel_with_status_and_widths(name: &str, widths: &[u16]) -> Result<ClxSpriteList, String> {
    load_cel_list_or_sheet_with_status(name, WidthOrWidths::Multiple(widths))
        .map(|los| los.list())
}

/// 加载 CEL 精灵表
///
/// C++ API: `OwnedClxSpriteSheet LoadCelSheet(const char *pszName, uint16_t width)`
pub fn load_cel_sheet(name: &str, width: u16) -> ClxSpriteSheet {
    load_cel_list_or_sheet(name, WidthOrWidths::Single(width)).sheet()
}

/// 宽度规格（对应 C++ PointerOrValue<uint16_t>）
#[derive(Clone, Copy)]
pub enum WidthOrWidths<'a> {
    /// 单一宽度
    Single(u16),
    /// 宽度数组指针
    Multiple(&'a [u16]),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cel_extension() {
        assert_eq!(DEVILUTIONX_CEL_EXT, ".clx");
    }
}

