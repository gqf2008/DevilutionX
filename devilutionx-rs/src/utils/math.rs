//! 数学工具函数
//!
//! 对应 C++: Source/utils/math.h
//!
//! ## 依赖关系 (Dependencies)
//!
//! C++ 依赖: 无
//! Rust 依赖: 无
//!
//! ## 禁止变更 - 已完成移植

use std::ops::{Add, Div, Mul, Sub};

/// 计算值的符号
///
/// # 参数
///
/// * `t` - 要计算符号的值
///
/// # 返回值
///
/// * `-1` 如果 `t < 0`
/// * `1` 如果 `t > 0`
/// * `0` 如果 `t == 0`
///
/// # C++ 对应函数
///
/// ```cpp
/// template <typename T>
/// int Sign(T t)
/// {
///     return (t > T(0)) - (t < T(0));
/// }
/// ```
#[inline]
pub fn sign<T>(t: T) -> i32
where
    T: Default + PartialOrd,
{
    let zero = T::default();
    (t > zero) as i32 - (t < zero) as i32
}

/// 线性插值
///
/// 从 `a` 向 `b` 进行线性插值，使用混合值 `t`
///
/// # 参数
///
/// * `a` - 低插值值（当 `t == 0` 时返回）
/// * `b` - 高插值值（当 `t == 1` 时返回）
/// * `t` - 插值器，通常在 `[0..1]` 范围内，超出范围的值将进行外推
///
/// # 返回值
///
/// `a + (b - a) * t`
///
/// # C++ 对应函数
///
/// ```cpp
/// template <typename V, typename T>
/// V Lerp(V a, V b, T t)
/// {
///     return a + (b - a) * t;
/// }
/// ```
#[inline]
pub fn lerp<V, T>(a: V, b: V, t: T) -> V
where
    V: Copy + Add<Output = V> + Sub<Output = V> + Mul<T, Output = V>,
{
    a + (b - a) * t
}

/// 逆线性插值
///
/// 给定两个关键值 `a` 和 `b`，以及一个自由值 `v`，
/// 确定混合因子 `t` 使得 `v = lerp(a, b, t)`
///
/// # 参数
///
/// * `a` - 低关键值（如果 `v == a` 则返回 0）
/// * `b` - 高关键值（如果 `v == b` 则返回 1）
/// * `v` - 混合因子，通常在 `[a..b]` 范围内以获得 `[0..1]` 的返回值
///
/// # 返回值
///
/// 值 `t` 使得 `v = lerp(a, b, t)`；如果 `b == a` 则返回 0
///
/// # C++ 对应函数
///
/// ```cpp
/// template <typename T>
/// T InvLerp(T a, T b, T v)
/// {
///     if (b == a)
///         return T(0);
///     return (v - a) / (b - a);
/// }
/// ```
#[inline]
pub fn inv_lerp<T>(a: T, b: T, v: T) -> T
where
    T: Copy + Default + PartialEq + Sub<Output = T> + Div<Output = T>,
{
    if b == a {
        T::default()
    } else {
        (v - a) / (b - a)
    }
}

/// 重映射值
///
/// 将值 `v` 从范围 `[in_min, in_max]` 重映射到 `[out_min, out_max]`
///
/// # 参数
///
/// * `in_min` - 输入范围的第一个边界
/// * `in_max` - 输入范围的第二个边界
/// * `out_min` - 输出范围的第一个边界
/// * `out_max` - 输出范围的第二个边界
/// * `v` - 要重映射的值
///
/// # 返回值
///
/// 转换后的值，使得 `inv_lerp(in_min, in_max, v) == inv_lerp(out_min, out_max, result)`
///
/// # C++ 对应函数
///
/// ```cpp
/// template <typename T>
/// T Remap(T inMin, T inMax, T outMin, T outMax, T v)
/// {
///     auto t = InvLerp(inMin, inMax, v);
///     return Lerp(outMin, outMax, t);
/// }
/// ```
#[inline]
pub fn remap<T>(in_min: T, in_max: T, out_min: T, out_max: T, v: T) -> T
where
    T: Copy + Default + PartialEq + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    let t = inv_lerp(in_min, in_max, v);
    lerp(out_min, out_max, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_positive() {
        assert_eq!(sign(5), 1);
        assert_eq!(sign(100i32), 1);
        assert_eq!(sign(0.5f64), 1);
    }

    #[test]
    fn test_sign_negative() {
        assert_eq!(sign(-5), -1);
        assert_eq!(sign(-100i32), -1);
        assert_eq!(sign(-0.5f64), -1);
    }

    #[test]
    fn test_sign_zero() {
        assert_eq!(sign(0), 0);
        assert_eq!(sign(0i32), 0);
        assert_eq!(sign(0.0f64), 0);
    }

    #[test]
    fn test_lerp_basic() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_lerp_extrapolate() {
        assert_eq!(lerp(0.0, 10.0, 2.0), 20.0);
        assert_eq!(lerp(0.0, 10.0, -1.0), -10.0);
    }

    #[test]
    fn test_inv_lerp_basic() {
        assert_eq!(inv_lerp(0.0, 10.0, 5.0), 0.5);
        assert_eq!(inv_lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(inv_lerp(0.0, 10.0, 10.0), 1.0);
    }

    #[test]
    fn test_inv_lerp_same_bounds() {
        assert_eq!(inv_lerp(5.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn test_remap() {
        // 从 [0, 100] 映射到 [0, 1]
        assert_eq!(remap(0.0, 100.0, 0.0, 1.0, 50.0), 0.5);
        // 从 [0, 10] 映射到 [100, 200]
        assert_eq!(remap(0.0, 10.0, 100.0, 200.0, 5.0), 150.0);
    }
}
