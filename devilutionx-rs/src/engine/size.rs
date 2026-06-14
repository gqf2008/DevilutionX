//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Size - 2D 尺寸类型
//!
//! 移植自 Source/engine/size.hpp

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    #[inline]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// 正方形 (单参数构造)
    /// C++ API: `explicit constexpr SizeOf(SizeT size)`
    #[inline]
    pub const fn uniform(size: i32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

// C++ operator+=
impl Add<i32> for Size {
    type Output = Size;
    fn add(self, factor: i32) -> Self::Output {
        Size::new(self.width + factor, self.height + factor)
    }
}

impl AddAssign<i32> for Size {
    fn add_assign(&mut self, factor: i32) {
        self.width += factor;
        self.height += factor;
    }
}

// C++ operator-=
impl Sub<i32> for Size {
    type Output = Size;
    fn sub(self, factor: i32) -> Self::Output {
        Size::new(self.width - factor, self.height - factor)
    }
}

impl SubAssign<i32> for Size {
    fn sub_assign(&mut self, factor: i32) {
        self.width -= factor;
        self.height -= factor;
    }
}

// C++ operator*=
impl Mul<i32> for Size {
    type Output = Size;
    fn mul(self, factor: i32) -> Self::Output {
        Size::new(self.width * factor, self.height * factor)
    }
}

impl MulAssign<i32> for Size {
    fn mul_assign(&mut self, factor: i32) {
        self.width *= factor;
        self.height *= factor;
    }
}

impl Mul<f32> for Size {
    type Output = Size;
    fn mul(self, factor: f32) -> Self::Output {
        Size::new(
            (self.width as f32 * factor) as i32,
            (self.height as f32 * factor) as i32,
        )
    }
}

impl MulAssign<f32> for Size {
    fn mul_assign(&mut self, factor: f32) {
        self.width = (self.width as f32 * factor) as i32;
        self.height = (self.height as f32 * factor) as i32;
    }
}

// C++ operator/=
impl Div<i32> for Size {
    type Output = Size;
    fn div(self, factor: i32) -> Self::Output {
        Size::new(self.width / factor, self.height / factor)
    }
}

impl DivAssign<i32> for Size {
    fn div_assign(&mut self, factor: i32) {
        self.width /= factor;
        self.height /= factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_ops() {
        let mut s = Size::new(10, 20);
        s += 5;
        assert_eq!(s, Size::new(15, 25));
        s -= 5;
        assert_eq!(s, Size::new(10, 20));
        s *= 2;
        assert_eq!(s, Size::new(20, 40));
        s /= 2;
        assert_eq!(s, Size::new(10, 20));
    }
}
