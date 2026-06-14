//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Blit 实现 - 移植自 Source/engine/render/blit_impl.hpp
//!
//! 提供多种像素拷贝(blit)操作的实现，包括：
//! - 直接拷贝 (BlitDirect)
//! - 带颜色映射的拷贝 (BlitWithMap)
//! - 带光照的拷贝 (BlitWithLightmap)
//! - 混合拷贝 (BlitBlended)
//! - 带颜色映射的混合拷贝 (BlitBlendedWithMap)
//! - 带光照的混合拷贝 (BlitBlendedWithLightmap)

use super::light_render::Lightmap;
use super::primitive_render::get_palette_transparency_lookup;

// =============================================================================
// Direct Blit Operations - 直接拷贝
// =============================================================================

/// 直接填充（不做任何颜色变换）
/// 对应 C++ BlitFillDirect
#[inline(always)]
pub fn blit_fill_direct(dst: &mut [u8], length: usize, color: u8) {
    debug_assert!(length > 0 && length <= dst.len());
    dst[..length].fill(color);
}

/// 直接拷贝像素（不做任何颜色变换）
/// 对应 C++ BlitPixelsDirect
#[inline(always)]
pub fn blit_pixels_direct(dst: &mut [u8], src: &[u8], length: usize) {
    debug_assert!(length > 0 && length <= dst.len() && length <= src.len());
    dst[..length].copy_from_slice(&src[..length]);
}

/// 直接 Blit 操作器
/// 对应 C++ struct BlitDirect
pub struct BlitDirect;

impl BlitDirect {
    /// 拷贝像素
    #[inline(always)]
    pub fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        blit_pixels_direct(dst, src, length);
    }

    /// 填充像素
    #[inline(always)]
    pub fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        blit_fill_direct(dst, length, color);
    }
}

// =============================================================================
// Blit With Color Map - 带颜色映射的拷贝
// =============================================================================

/// 带颜色映射的填充
/// 对应 C++ BlitFillWithMap
#[inline(always)]
pub fn blit_fill_with_map(dst: &mut [u8], length: usize, color: u8, color_map: &[u8; 256]) {
    debug_assert!(length > 0 && length <= dst.len());
    let mapped_color = color_map[color as usize];
    dst[..length].fill(mapped_color);
}

/// 带颜色映射的像素拷贝
/// 对应 C++ BlitPixelsWithMap
#[inline(always)]
pub fn blit_pixels_with_map(dst: &mut [u8], src: &[u8], length: usize, color_map: &[u8; 256]) {
    debug_assert!(length > 0 && length <= dst.len() && length <= src.len());
    for i in 0..length {
        dst[i] = color_map[src[i] as usize];
    }
}

/// 带颜色映射的 Blit 操作器
/// 对应 C++ struct BlitWithMap
pub struct BlitWithMap<'a> {
    pub color_map: &'a [u8; 256],
}

impl<'a> BlitWithMap<'a> {
    /// 创建新的带颜色映射 blit 操作器
    pub fn new(color_map: &'a [u8; 256]) -> Self {
        Self { color_map }
    }

    /// 拷贝像素（带颜色映射）
    #[inline(always)]
    pub fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        blit_pixels_with_map(dst, src, length, self.color_map);
    }

    /// 填充像素（带颜色映射）
    #[inline(always)]
    pub fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        blit_fill_with_map(dst, length, color, self.color_map);
    }
}

// =============================================================================
// Blit With Lightmap - 带光照的拷贝
// =============================================================================

/// 带光照的填充
/// 对应 C++ BlitFillWithLightmap
#[inline(always)]
pub fn blit_fill_with_lightmap(dst: &mut [u8], length: usize, color: u8, lightmap: &Lightmap) {
    debug_assert!(length > 0 && length <= dst.len());
    let light = lightmap.get_lighting_at(dst.as_ptr());
    for i in 0..length {
        let light_level = unsafe { *light.add(i) };
        dst[i] = lightmap.adjust_color(color, light_level);
    }
}

/// 带光照的像素拷贝
/// 对应 C++ BlitPixelsWithLightmap
#[inline(always)]
pub fn blit_pixels_with_lightmap(dst: &mut [u8], src: &[u8], length: usize, lightmap: &Lightmap) {
    debug_assert!(length > 0 && length <= dst.len() && length <= src.len());
    let light = lightmap.get_lighting_at(dst.as_ptr());
    for i in 0..length {
        let src_color = src[i];
        let light_level = unsafe { *light.add(i) };
        dst[i] = lightmap.adjust_color(src_color, light_level);
    }
}

/// 带光照的 Blit 操作器
/// 对应 C++ struct BlitWithLightmap
pub struct BlitWithLightmap<'a> {
    pub lightmap: &'a Lightmap<'a>,
}

impl<'a> BlitWithLightmap<'a> {
    /// 创建新的带光照 blit 操作器
    pub fn new(lightmap: &'a Lightmap<'a>) -> Self {
        Self { lightmap }
    }

    /// 拷贝像素（带光照）
    #[inline(always)]
    pub fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        blit_pixels_with_lightmap(dst, src, length, self.lightmap);
    }

    /// 填充像素（带光照）
    #[inline(always)]
    pub fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        blit_fill_with_lightmap(dst, length, color, self.lightmap);
    }
}

// =============================================================================
// Blended Blit Operations - 混合拷贝
// =============================================================================

/// 混合填充（使用调色板透明度查找表）
/// 对应 C++ BlitFillBlended
#[inline(always)]
pub fn blit_fill_blended(dst: &mut [u8], length: usize, color: u8) {
    debug_assert!(length > 0 && length <= dst.len());
    let lookup = get_palette_transparency_lookup();
    let tbl = lookup.lookup_row(color);
    for i in 0..length {
        dst[i] = tbl[dst[i] as usize];
    }
}

/// 混合像素拷贝（使用调色板透明度查找表）
/// 对应 C++ BlitPixelsBlended
#[inline(always)]
pub fn blit_pixels_blended(dst: &mut [u8], src: &[u8], length: usize) {
    debug_assert!(length > 0 && length <= dst.len() && length <= src.len());
    let lookup = get_palette_transparency_lookup();
    for i in 0..length {
        let src_color = src[i];
        let dst_color = dst[i];
        dst[i] = lookup.blend(src_color, dst_color);
    }
}

/// 混合 Blit 操作器
/// 对应 C++ struct BlitBlended
pub struct BlitBlended;

impl BlitBlended {
    /// 拷贝像素（混合）
    #[inline(always)]
    pub fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        blit_pixels_blended(dst, src, length);
    }

    /// 填充像素（混合）
    #[inline(always)]
    pub fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        blit_fill_blended(dst, length, color);
    }
}

// =============================================================================
// Blended Blit With Color Map - 带颜色映射的混合拷贝
// =============================================================================

/// 带颜色映射的混合像素拷贝
/// 对应 C++ BlitPixelsBlendedWithMap
#[inline(always)]
pub fn blit_pixels_blended_with_map(
    dst: &mut [u8],
    src: &[u8],
    length: usize,
    color_map: &[u8; 256],
) {
    debug_assert!(length > 0 && length <= dst.len() && length <= src.len());
    let lookup = get_palette_transparency_lookup();
    for i in 0..length {
        let src_color = src[i];
        let dst_color = dst[i];
        let mapped_src = color_map[src_color as usize];
        dst[i] = lookup.blend(dst_color, mapped_src);
    }
}

/// 带颜色映射的混合 Blit 操作器
/// 对应 C++ struct BlitBlendedWithMap
pub struct BlitBlendedWithMap<'a> {
    pub color_map: &'a [u8; 256],
}

impl<'a> BlitBlendedWithMap<'a> {
    /// 创建新的带颜色映射混合 blit 操作器
    pub fn new(color_map: &'a [u8; 256]) -> Self {
        Self { color_map }
    }

    /// 拷贝像素（带颜色映射的混合）
    #[inline(always)]
    pub fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        blit_pixels_blended_with_map(dst, src, length, self.color_map);
    }

    /// 填充像素（带颜色映射的混合）
    #[inline(always)]
    pub fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        let mapped_color = self.color_map[color as usize];
        blit_fill_blended(dst, length, mapped_color);
    }
}

// =============================================================================
// Blended Blit With Lightmap - 带光照的混合拷贝
// =============================================================================

/// 带光照的混合填充
/// 对应 C++ BlitFillBlendedWithLightmap
#[inline(always)]
pub fn blit_fill_blended_with_lightmap(
    dst: &mut [u8],
    length: usize,
    color: u8,
    lightmap: &Lightmap,
) {
    debug_assert!(length > 0 && length <= dst.len());
    let lookup = get_palette_transparency_lookup();
    let light = lightmap.get_lighting_at(dst.as_ptr());
    for i in 0..length {
        let light_level = unsafe { *light.add(i) };
        let src_color = lightmap.adjust_color(color, light_level);
        let dst_color = dst[i];
        dst[i] = lookup.blend(src_color, dst_color);
    }
}

/// 带光照的混合像素拷贝
/// 对应 C++ BlitPixelsBlendedWithLightmap
///
/// 对于长度小于 1024 的情况，使用栈上临时缓冲区提高缓存命中率
/// 对于更长的情况，直接逐像素处理
#[inline(always)]
pub fn blit_pixels_blended_with_lightmap(
    dst: &mut [u8],
    src: &[u8],
    length: usize,
    lightmap: &Lightmap,
) {
    debug_assert!(length > 0 && length <= dst.len() && length <= src.len());
    let lookup = get_palette_transparency_lookup();
    let light = lightmap.get_lighting_at(dst.as_ptr());

    // 对于短序列，使用栈上临时缓冲区（更好的缓存命中）
    if length < 1024 {
        let mut lit_src = [0u8; 1024];
        // 第一趟：应用光照到源像素
        for i in 0..length {
            let src_color = src[i];
            let light_level = unsafe { *light.add(i) };
            lit_src[i] = lightmap.adjust_color(src_color, light_level);
        }
        // 第二趟：与目标混合
        for i in 0..length {
            let src_color = lit_src[i];
            let dst_color = dst[i];
            dst[i] = lookup.blend(dst_color, src_color);
        }
        return;
    }

    // 对于长序列，直接逐像素处理
    for i in 0..length {
        let src_color = src[i];
        let dst_color = dst[i];
        let light_level = unsafe { *light.add(i) };
        let lit_color = lightmap.adjust_color(src_color, light_level);
        dst[i] = lookup.blend(dst_color, lit_color);
    }
}

/// 带光照的混合 Blit 操作器
/// 对应 C++ struct BlitBlendedWithLightmap
pub struct BlitBlendedWithLightmap<'a> {
    pub lightmap: &'a Lightmap<'a>,
}

impl<'a> BlitBlendedWithLightmap<'a> {
    /// 创建新的带光照混合 blit 操作器
    pub fn new(lightmap: &'a Lightmap<'a>) -> Self {
        Self { lightmap }
    }

    /// 拷贝像素（带光照的混合）
    #[inline(always)]
    pub fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        blit_pixels_blended_with_lightmap(dst, src, length, self.lightmap);
    }

    /// 填充像素（带光照的混合）
    #[inline(always)]
    pub fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        blit_fill_blended_with_lightmap(dst, length, color, self.lightmap);
    }
}

// =============================================================================
// Blit Trait - 统一的 Blit 接口
// =============================================================================

/// Blit 操作 trait
/// 提供统一的 blit 接口，用于泛型编程
pub trait Blit {
    /// 拷贝像素
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize);
    /// 填充像素
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8);
}

impl Blit for BlitDirect {
    #[inline(always)]
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        self.blit_pixels(dst, src, length);
    }

    #[inline(always)]
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        self.blit_fill(dst, length, color);
    }
}

impl<'a> Blit for BlitWithMap<'a> {
    #[inline(always)]
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        self.blit_pixels(dst, src, length);
    }

    #[inline(always)]
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        self.blit_fill(dst, length, color);
    }
}

impl<'a> Blit for BlitWithLightmap<'a> {
    #[inline(always)]
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        self.blit_pixels(dst, src, length);
    }

    #[inline(always)]
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        self.blit_fill(dst, length, color);
    }
}

impl Blit for BlitBlended {
    #[inline(always)]
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        self.blit_pixels(dst, src, length);
    }

    #[inline(always)]
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        self.blit_fill(dst, length, color);
    }
}

impl<'a> Blit for BlitBlendedWithMap<'a> {
    #[inline(always)]
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        self.blit_pixels(dst, src, length);
    }

    #[inline(always)]
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        self.blit_fill(dst, length, color);
    }
}

impl<'a> Blit for BlitBlendedWithLightmap<'a> {
    #[inline(always)]
    fn blit_pixels(&self, dst: &mut [u8], src: &[u8], length: usize) {
        self.blit_pixels(dst, src, length);
    }

    #[inline(always)]
    fn blit_fill(&self, dst: &mut [u8], length: usize, color: u8) {
        self.blit_fill(dst, length, color);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blit_fill_direct() {
        let mut dst = [0u8; 16];
        blit_fill_direct(&mut dst, 8, 42);
        assert_eq!(&dst[..8], &[42u8; 8]);
        assert_eq!(&dst[8..], &[0u8; 8]);
    }

    #[test]
    fn test_blit_pixels_direct() {
        let mut dst = [0u8; 16];
        let src = [1u8, 2, 3, 4, 5, 6, 7, 8];
        blit_pixels_direct(&mut dst, &src, 8);
        assert_eq!(&dst[..8], &src);
        assert_eq!(&dst[8..], &[0u8; 8]);
    }

    #[test]
    fn test_blit_fill_with_map() {
        let mut color_map = [0u8; 256];
        for i in 0..256 {
            color_map[i] = (255 - i) as u8;
        }
        let mut dst = [0u8; 8];
        blit_fill_with_map(&mut dst, 8, 10, &color_map);
        assert_eq!(dst, [245u8; 8]); // 255 - 10 = 245
    }

    #[test]
    fn test_blit_pixels_with_map() {
        let mut color_map = [0u8; 256];
        for i in 0..256 {
            color_map[i] = (i / 2) as u8;
        }
        let mut dst = [0u8; 4];
        let src = [0u8, 100, 200, 255];
        blit_pixels_with_map(&mut dst, &src, 4, &color_map);
        assert_eq!(dst, [0, 50, 100, 127]);
    }

    #[test]
    fn test_blit_direct_struct() {
        let blit = BlitDirect;
        let mut dst = [0u8; 8];
        let src = [1u8, 2, 3, 4];
        blit.blit_pixels(&mut dst, &src, 4);
        assert_eq!(&dst[..4], &src);

        blit.blit_fill(&mut dst[4..], 4, 99);
        assert_eq!(&dst[4..], &[99u8; 4]);
    }

    #[test]
    fn test_blit_with_map_struct() {
        let mut color_map = [0u8; 256];
        for i in 0..256 {
            color_map[i] = ((i + 1) % 256) as u8;
        }
        let blit = BlitWithMap::new(&color_map);
        
        let mut dst = [0u8; 4];
        let src = [0u8, 127, 254, 255];
        blit.blit_pixels(&mut dst, &src, 4);
        assert_eq!(dst, [1, 128, 255, 0]); // (x+1) % 256
    }

    #[test]
    fn test_blit_blended_struct() {
        // 注意：这个测试依赖全局的 paletteTransparencyLookup
        // 默认情况下为空，所以混合结果会是 0
        let blit = BlitBlended;
        let mut dst = [10u8; 4];
        let src = [20u8, 30, 40, 50];
        blit.blit_pixels(&mut dst, &src, 4);
        // 由于默认 lookup 是空的，结果都是 0
        assert_eq!(dst, [0u8; 4]);
    }

    #[test]
    fn test_blit_trait() {
        fn generic_blit<B: Blit>(blit: &B, dst: &mut [u8], src: &[u8]) {
            blit.blit_pixels(dst, src, src.len());
        }

        let direct = BlitDirect;
        let mut dst = [0u8; 4];
        let src = [1u8, 2, 3, 4];
        generic_blit(&direct, &mut dst, &src);
        assert_eq!(dst, src);
    }
}
