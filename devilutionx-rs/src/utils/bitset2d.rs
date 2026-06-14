//! 2D 位集合
//!
//! 对应 C++: Source/utils/bitset2d.hpp
//!
//! ## 依赖关系 (Dependencies)
//!
//! C++ 依赖: `<bitset>`, `<cstddef>`
//! Rust 依赖: 无
//!
//! ## 禁止变更 - 已完成移植

/// 2D 位集合
///
/// `std::bitset` 的 2D 变体，使用动态分配的 Vec<u64>
///
/// # C++ 对应类
///
/// ```cpp
/// template <size_t Width, size_t Height>
/// class Bitset2d {
///     std::bitset<Width * Height> data_;
/// };
/// ```
pub struct Bitset2d {
    width: usize,
    height: usize,
    data: Vec<u64>,
}

impl Bitset2d {
    /// 创建新的空位集合（所有位为 0）
    pub fn new(width: usize, height: usize) -> Self {
        let total_bits = width * height;
        let num_words = (total_bits + 63) / 64;
        Self {
            width,
            height,
            data: vec![0u64; num_words],
        }
    }

    /// 计算 (x, y) 对应的位索引
    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// 测试指定位置的位是否被设置
    ///
    /// # C++ 对应函数
    ///
    /// ```cpp
    /// bool test(size_t x, size_t y) const
    /// {
    ///     return data_.test(index(x, y));
    /// }
    /// ```
    #[inline]
    pub fn test(&self, x: usize, y: usize) -> bool {
        let idx = self.index(x, y);
        let word_idx = idx / 64;
        let bit_idx = idx % 64;
        (self.data[word_idx] & (1u64 << bit_idx)) != 0
    }

    /// 设置指定位置的位
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标
    /// * `y` - Y 坐标
    /// * `value` - 要设置的值（默认为 true）
    ///
    /// # C++ 对应函数
    ///
    /// ```cpp
    /// void set(size_t x, size_t y, bool value = true)
    /// {
    ///     data_.set(index(x, y), value);
    /// }
    /// ```
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let idx = self.index(x, y);
        let word_idx = idx / 64;
        let bit_idx = idx % 64;
        if value {
            self.data[word_idx] |= 1u64 << bit_idx;
        } else {
            self.data[word_idx] &= !(1u64 << bit_idx);
        }
    }

    /// 重置指定位置的位（设为 0）
    ///
    /// # C++ 对应函数
    ///
    /// ```cpp
    /// void reset(size_t x, size_t y)
    /// {
    ///     data_.reset(index(x, y));
    /// }
    /// ```
    #[inline]
    pub fn reset_at(&mut self, x: usize, y: usize) {
        self.set(x, y, false);
    }

    /// 重置所有位（全部设为 0）
    ///
    /// # C++ 对应函数
    ///
    /// ```cpp
    /// void reset()
    /// {
    ///     data_.reset();
    /// }
    /// ```
    #[inline]
    pub fn reset(&mut self) {
        for word in &mut self.data {
            *word = 0;
        }
    }

    /// 计算被设置的位的数量
    ///
    /// # C++ 对应函数
    ///
    /// ```cpp
    /// [[nodiscard]] size_t count() const
    /// {
    ///     return data_.count();
    /// }
    /// ```
    #[inline]
    pub fn count(&self) -> usize {
        self.data.iter().map(|w| w.count_ones() as usize).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let bs = Bitset2d::new(10, 10);
        assert_eq!(bs.count(), 0);
    }

    #[test]
    fn test_set_and_test() {
        let mut bs = Bitset2d::new(10, 10);
        assert!(!bs.test(5, 5));
        bs.set(5, 5, true);
        assert!(bs.test(5, 5));
    }

    #[test]
    fn test_reset_at() {
        let mut bs = Bitset2d::new(10, 10);
        bs.set(3, 4, true);
        assert!(bs.test(3, 4));
        bs.reset_at(3, 4);
        assert!(!bs.test(3, 4));
    }

    #[test]
    fn test_reset_all() {
        let mut bs = Bitset2d::new(10, 10);
        bs.set(1, 1, true);
        bs.set(2, 2, true);
        bs.set(9, 9, true);
        assert_eq!(bs.count(), 3);
        bs.reset();
        assert_eq!(bs.count(), 0);
    }

    #[test]
    fn test_count() {
        let mut bs = Bitset2d::new(8, 8);
        bs.set(0, 0, true);
        bs.set(7, 7, true);
        bs.set(3, 5, true);
        assert_eq!(bs.count(), 3);
    }

    #[test]
    fn test_large_bitset() {
        let mut bs = Bitset2d::new(112, 112);
        bs.set(0, 0, true);
        bs.set(111, 111, true);
        bs.set(50, 60, true);
        assert_eq!(bs.count(), 3);
        assert!(bs.test(111, 111));
    }
}
