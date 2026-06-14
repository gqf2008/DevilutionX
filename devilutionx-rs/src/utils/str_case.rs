//! ASCII 字符串大小写转换工具
//!
//! 对应 C++: Source/utils/str_case.cpp
//!
//! ## 依赖关系 (Dependencies)
//!
//! C++ 依赖:
//! - str_case.hpp (自身头文件)
//!
//! Rust 依赖:
//! - 无外部依赖，纯标准库实现
//!
//! ## 移植说明 (Porting Notes)
//!
//! C++ 版本提供两个函数:
//! 1. `AsciiStrToLower(std::string &str)` - 原地修改版本
//! 2. `AsciiStrToLower(std::string_view str)` - 返回新字符串版本
//!
//! Rust 版本提供:
//! 1. `ascii_str_to_lower_in_place(&mut String)` - 原地修改版本
//! 2. `ascii_str_to_lower(&str) -> String` - 返回新字符串版本

/// 原地将 ASCII 字符串转换为小写
///
/// 只处理 ASCII 大写字母 A-Z，其他字符保持不变
///
/// # 参数
///
/// * `s` - 要转换的字符串（可变引用）
///
/// # 示例
///
/// ```
/// # use devilutionx::utils::str_case::ascii_str_to_lower_in_place;
/// let mut s = String::from("HELLO World 123");
/// ascii_str_to_lower_in_place(&mut s);
/// assert_eq!(s, "hello world 123");
/// ```
///
/// # C++ 对应函数
///
/// ```cpp
/// void AsciiStrToLower(std::string &str)
/// {
///     for (char &c : str) {
///         if (c >= 'A' && c <= 'Z')
///             c += ('a' - 'A');
///     }
/// }
/// ```
#[inline]
pub fn ascii_str_to_lower_in_place(s: &mut String) {
    // SAFETY: 我们只修改 ASCII 字符，这些字符在 UTF-8 中是单字节的，
    // 所以直接操作字节是安全的
    // 注意：std::string 中的 char 是单字节，对应 Rust 的 u8
    // 我们需要使用 unsafe 来匹配 C++ 的逐字节修改行为
    unsafe {
        let bytes = s.as_bytes_mut();
        for byte in bytes.iter_mut() {
            if *byte >= b'A' && *byte <= b'Z' {
                *byte += b'a' - b'A';
            }
        }
    }
}

/// 将 ASCII 字符串转换为小写，返回新字符串
///
/// 只处理 ASCII 大写字母 A-Z，其他字符保持不变
///
/// # 参数
///
/// * `s` - 要转换的字符串切片
///
/// # 返回值
///
/// 转换后的新字符串
///
/// # 示例
///
/// ```
/// # use devilutionx::utils::str_case::ascii_str_to_lower;
/// let s = "HELLO World 123";
/// let lower = ascii_str_to_lower(s);
/// assert_eq!(lower, "hello world 123");
/// ```
///
/// # C++ 对应函数
///
/// ```cpp
/// [[nodiscard]] inline std::string AsciiStrToLower(std::string_view str)
/// {
///     std::string copy { str.data(), str.size() };
///     AsciiStrToLower(copy);
///     return copy;
/// }
/// ```
#[inline]
pub fn ascii_str_to_lower(s: &str) -> String {
    let mut copy = String::from(s);
    ascii_str_to_lower_in_place(&mut copy);
    copy
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试原地转换 - 全大写
    #[test]
    fn test_ascii_str_to_lower_in_place_all_upper() {
        let mut s = String::from("HELLO");
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "hello");
    }

    /// 测试原地转换 - 混合大小写
    #[test]
    fn test_ascii_str_to_lower_in_place_mixed() {
        let mut s = String::from("HeLLo WoRLd");
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "hello world");
    }

    /// 测试原地转换 - 已经是小写
    #[test]
    fn test_ascii_str_to_lower_in_place_already_lower() {
        let mut s = String::from("hello world");
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "hello world");
    }

    /// 测试原地转换 - 带数字和特殊字符
    #[test]
    fn test_ascii_str_to_lower_in_place_with_numbers() {
        let mut s = String::from("ABC123DEF!@#");
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "abc123def!@#");
    }

    /// 测试原地转换 - 空字符串
    #[test]
    fn test_ascii_str_to_lower_in_place_empty() {
        let mut s = String::new();
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "");
    }

    /// 测试返回新字符串版本
    #[test]
    fn test_ascii_str_to_lower() {
        let s = "HELLO World 123";
        let result = ascii_str_to_lower(s);
        assert_eq!(result, "hello world 123");
        // 原始字符串应该保持不变
        assert_eq!(s, "HELLO World 123");
    }

    /// 测试边界字符 - 'A' 和 'Z'
    #[test]
    fn test_ascii_str_to_lower_boundary_chars() {
        let mut s = String::from("@AZ[");
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "@az["); // @ 是 'A'-1, [ 是 'Z'+1
    }

    /// 测试非 ASCII 字符（应保持不变）
    #[test]
    fn test_ascii_str_to_lower_non_ascii() {
        // UTF-8 字符串包含非 ASCII 字符
        // 注意：我们的实现只处理 ASCII，非 ASCII 字符保持不变
        let mut s = String::from("ABCéDEF");
        ascii_str_to_lower_in_place(&mut s);
        assert_eq!(s, "abcédef");
    }
}
