//! 整数和定点数解析工具
//!
//! 对应 C++: Source/utils/parse_int.cpp, Source/utils/parse_int.hpp
//!
//! ## 依赖关系 (Dependencies)
//!
//! C++ 依赖:
//! - `<charconv>` (std::from_chars)
//! - `<expected.hpp>` (tl::expected)
//!
//! Rust 依赖:
//! - 标准库 str::parse
//!
//! ## 禁止变更 - 已完成移植

/// 整数解析错误类型
///
/// 对应 C++: `enum class ParseIntError`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseIntError {
    /// 解析错误（无效输入）
    ParseError = 1,
    /// 超出范围
    OutOfRange,
}

/// 整数解析结果类型
///
/// 对应 C++: `template <typename IntT> using ParseIntResult = tl::expected<IntT, ParseIntError>;`
pub type ParseIntResult<T> = Result<T, ParseIntError>;

/// 解析整数，支持范围限制
///
/// # 参数
///
/// * `s` - 要解析的字符串
/// * `min` - 最小值限制
/// * `max` - 最大值限制
///
/// # 返回值
///
/// 返回 `(Result<T, ParseIntError>, &str)`，其中第二个元素是解析停止位置之后的剩余字符串
///
/// # C++ 对应函数
///
/// ```cpp
/// template <typename IntT>
/// ParseIntResult<IntT> ParseInt(
///     std::string_view str, IntT min = std::numeric_limits<IntT>::min(),
///     IntT max = std::numeric_limits<IntT>::max(), const char **endOfParse = nullptr)
/// ```
pub fn parse_int<T>(s: &str, min: T, max: T) -> (ParseIntResult<T>, &str)
where
    T: std::str::FromStr + std::cmp::PartialOrd + Copy,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    // 找到数字结束位置
    let mut end_idx = 0;
    let bytes = s.as_bytes();
    
    // 处理可选的负号
    if !bytes.is_empty() && bytes[0] == b'-' {
        end_idx = 1;
    }
    
    // 读取数字字符
    while end_idx < bytes.len() && bytes[end_idx] >= b'0' && bytes[end_idx] <= b'9' {
        end_idx += 1;
    }
    
    if end_idx == 0 || (end_idx == 1 && bytes[0] == b'-') {
        // 没有数字
        return (Err(ParseIntError::ParseError), s);
    }
    
    let num_str = &s[..end_idx];
    let rest = &s[end_idx..];
    
    match num_str.parse::<T>() {
        Ok(value) => {
            if value < min || value > max {
                (Err(ParseIntError::OutOfRange), rest)
            } else {
                (Ok(value), rest)
            }
        }
        Err(_) => {
            // 解析失败，可能是溢出
            (Err(ParseIntError::OutOfRange), rest)
        }
    }
}

/// 解析整数（使用默认范围）
pub fn parse_int_default<T>(s: &str) -> (ParseIntResult<T>, &str)
where
    T: std::str::FromStr + std::cmp::PartialOrd + Copy + Bounded,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    parse_int(s, T::min_value(), T::max_value())
}

/// 提供类型边界的 trait
pub trait Bounded {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! impl_bounded {
    ($($t:ty),*) => {
        $(
            impl Bounded for $t {
                fn min_value() -> Self { <$t>::MIN }
                fn max_value() -> Self { <$t>::MAX }
            }
        )*
    };
}

impl_bounded!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

/// 解析小数部分为 6 位定点数
///
/// 将十进制小数字符串解析为 [0, 64] 范围的值，表示 [0.0, 1.0] 的 2.6 定点数
///
/// # 参数
///
/// * `s` - 十进制数字字符串（可以后跟非数字字符）
///
/// # 返回值
///
/// 返回 `(u8, &str)`，其中：
/// - 第一个元素是 [0, 64] 范围的值
/// - 第二个元素是解析停止位置之后的剩余字符串
///
/// # C++ 对应函数
///
/// ```cpp
/// uint8_t ParseFixed6Fraction(std::string_view str, const char **endOfParse = nullptr)
/// {
///     unsigned numDigits = 0;
///     uint32_t decimalFraction = 0;
///     // Read at most 7 digits...
///     // to ensure rounding to nearest we normalise all values to 7 decimal places
///     // we add half the step between representable values to use integer truncation
///     return (decimalFraction + 78125) / 156250;
/// }
/// ```
pub fn parse_fixed6_fraction(s: &str) -> (u8, &str) {
    let mut num_digits: u32 = 0;
    let mut decimal_fraction: u32 = 0;
    let bytes = s.as_bytes();
    let mut idx = 0;

    // Read at most 7 digits, at that threshold we're able to determine an exact rounding for 6 bit fixed point numbers
    while idx < bytes.len() && num_digits < 7 {
        let c = bytes[idx];
        if c < b'0' || c > b'9' {
            break;
        }
        decimal_fraction = decimal_fraction * 10 + (c - b'0') as u32;
        num_digits += 1;
        idx += 1;
    }

    // To mimic the behaviour of std::from_chars consume all remaining digits
    while idx < bytes.len() && bytes[idx] >= b'0' && bytes[idx] <= b'9' {
        idx += 1;
    }

    // To ensure rounding to nearest we normalise all values to 7 decimal places
    while num_digits < 7 {
        decimal_fraction *= 10;
        num_digits += 1;
    }

    // We add half the step between representable values to use integer truncation as a substitute for rounding to nearest
    let result = ((decimal_fraction + 78125) / 156250) as u8;
    (result, &s[idx..])
}

/// 解析定点数（整数部分 + 小数部分）
///
/// # 参数
///
/// * `s` - 要解析的字符串，格式如 "1.5" 或 "-2.25" 或 ".5"
///
/// # 返回值
///
/// 返回 `(ParseIntResult<T>, &str)`
///
/// # C++ 对应函数
///
/// ```cpp
/// template <typename IntT>
/// ParseIntResult<IntT> ParseFixed6(std::string_view str, const char **endOfParse = nullptr)
/// ```
pub fn parse_fixed6<T>(s: &str) -> (ParseIntResult<T>, &str)
where
    T: std::str::FromStr
        + std::cmp::PartialOrd
        + Copy
        + Bounded
        + From<i8>
        + std::ops::Shl<i32, Output = T>
        + std::ops::Shr<i32, Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    if s.is_empty() {
        return (Err(ParseIntError::ParseError), s);
    }

    let bytes = s.as_bytes();
    let is_negative = bytes[0] == b'-';

    // 计算整数部分的范围限制（右移 6 位）
    let min_integer_value = T::min_value() >> 6;
    let max_integer_value = T::max_value() >> 6;

    let mut current_str = s;
    let mut have_digits = false;
    let mut integer_result: ParseIntResult<T> = Err(ParseIntError::ParseError);

    // 尝试解析整数部分
    let (int_res, rest) = parse_int(current_str, min_integer_value, max_integer_value);
    
    if int_res.is_ok() || int_res == Err(ParseIntError::OutOfRange) {
        have_digits = true;
        integer_result = int_res;
        current_str = rest;
    } else if is_negative {
        // 跳过负号
        current_str = &current_str[1..];
    }

    // 解析小数部分
    let mut fraction_part: u8 = 0;
    if !current_str.is_empty() && current_str.as_bytes()[0] == b'.' {
        // 跳过小数点
        current_str = &current_str[1..];
        
        let start_ptr = current_str;
        let (frac, rest) = parse_fixed6_fraction(current_str);
        fraction_part = frac;
        
        if !std::ptr::eq(start_ptr.as_ptr(), rest.as_ptr()) {
            have_digits = true;
        }
        current_str = rest;
    }

    if !have_digits {
        // 没有读到任何数字，如 "-.abc"
        return (Err(ParseIntError::ParseError), s);
    }

    if integer_result == Err(ParseIntError::OutOfRange) {
        return (Err(ParseIntError::OutOfRange), current_str);
    }

    // integer_result 可能是 ParseError（字符串如 ".123" 或 "-.1"）
    let integer_part = integer_result.unwrap_or(T::from(0i8));

    // 检查溢出：rounding 可能给我们 64 作为小数部分
    if fraction_part >= 64 {
        if integer_part >= max_integer_value || integer_part <= min_integer_value {
            return (Err(ParseIntError::OutOfRange), current_str);
        }
    }

    let mut fixed_value = integer_part << 6;
    let frac_t = T::from(fraction_part as i8);
    
    if is_negative {
        fixed_value = fixed_value - frac_t;
    } else {
        fixed_value = fixed_value + frac_t;
    }

    (Ok(fixed_value), current_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixed6_fraction_basic() {
        // 0.5 -> 32 (half of 64)
        let (result, _) = parse_fixed6_fraction("5");
        assert_eq!(result, 32);
    }

    #[test]
    fn test_parse_fixed6_fraction_zero() {
        let (result, _) = parse_fixed6_fraction("0");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_parse_fixed6_fraction_one() {
        // 0.9999999 should round to 64
        let (result, _) = parse_fixed6_fraction("9999999");
        assert_eq!(result, 64);
    }

    #[test]
    fn test_parse_fixed6_fraction_quarter() {
        // 0.25 -> 16
        let (result, _) = parse_fixed6_fraction("25");
        assert_eq!(result, 16);
    }

    #[test]
    fn test_parse_int_basic() {
        let (result, rest) = parse_int::<i32>("123abc", i32::MIN, i32::MAX);
        assert_eq!(result, Ok(123));
        assert_eq!(rest, "abc");
    }

    #[test]
    fn test_parse_int_negative() {
        let (result, rest) = parse_int::<i32>("-456xyz", i32::MIN, i32::MAX);
        assert_eq!(result, Ok(-456));
        assert_eq!(rest, "xyz");
    }

    #[test]
    fn test_parse_int_out_of_range() {
        let (result, _) = parse_int::<i32>("100", 0, 50);
        assert_eq!(result, Err(ParseIntError::OutOfRange));
    }

    #[test]
    fn test_parse_int_parse_error() {
        let (result, rest) = parse_int::<i32>("abc", i32::MIN, i32::MAX);
        assert_eq!(result, Err(ParseIntError::ParseError));
        assert_eq!(rest, "abc");
    }
}
