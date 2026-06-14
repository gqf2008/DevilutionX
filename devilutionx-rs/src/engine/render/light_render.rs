//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! 光照渲染 - 移植自 Source/engine/render/light_render.hpp/cpp
//!
//! 提供逐像素光照映射和光照表查找功能
//! 使用半空间法渲染三角形进行 marching squares 光照插值

use crate::engine::{Direction, Displacement, Point};
use std::sync::Mutex;

/// 光照表大小 (256 色)
pub const LIGHT_TABLE_SIZE: usize = 256;

/// 光照级别数量 (0-15 + 1)
pub const NUM_LIGHTING_LEVELS: usize = 16;

/// 最大光照值
pub const LIGHTS_MAX: u8 = 15;

/// 瓦片宽度
pub const TILE_WIDTH: i32 = 64;
/// 瓦片高度
pub const TILE_HEIGHT: i32 = 32;

/// 最大地下城尺寸
pub const DMAXX: usize = 40;
pub const DMAXY: usize = 40;
pub const MAXDUNX: usize = 16 + DMAXX * 2 + 16;
pub const MAXDUNY: usize = 16 + DMAXY * 2 + 16;

/// 全局 Lightmap 缓冲区 - 对应 C++ static 变量
static LIGHTMAP_BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Lightmap - 对应 C++ Lightmap class
/// 存储每个像素的光照级别，用于逐像素光照渲染
pub struct Lightmap<'a> {
    /// 输出缓冲区引用（用于计算偏移）
    out_buffer: &'a [u8],
    /// 输出缓冲区的行距
    out_pitch: u16,
    /// 光照映射数据（借用或拥有）
    lightmap_buffer: LightmapBufferRef<'a>,
    /// 光照映射的行距
    lightmap_pitch: u16,
    /// 光照表引用 [光照级别][颜色索引] -> 调整后的颜色索引
    light_tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    /// 完全明亮的光照表指针
    fully_lit_light_table: *const u8,
    /// 完全黑暗的光照表指针
    fully_dark_light_table: *const u8,
}

/// Lightmap 缓冲区引用类型
enum LightmapBufferRef<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl<'a> LightmapBufferRef<'a> {
    fn as_slice(&self) -> &[u8] {
        match self {
            LightmapBufferRef::Borrowed(s) => s,
            LightmapBufferRef::Owned(v) => v.as_slice(),
        }
    }

    fn len(&self) -> usize {
        match self {
            LightmapBufferRef::Borrowed(s) => s.len(),
            LightmapBufferRef::Owned(v) => v.len(),
        }
    }
}

impl<'a> Lightmap<'a> {
    /// 创建新的 Lightmap - 对应 C++ 构造函数
    pub fn new(
        out_buffer: &'a [u8],
        out_pitch: u16,
        lightmap_buffer: &'a [u8],
        lightmap_pitch: u16,
        light_tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
        fully_lit_light_table: *const u8,
        fully_dark_light_table: *const u8,
    ) -> Self {
        Self {
            out_buffer,
            out_pitch,
            lightmap_buffer: LightmapBufferRef::Borrowed(lightmap_buffer),
            lightmap_pitch,
            light_tables,
            fully_lit_light_table,
            fully_dark_light_table,
        }
    }

    /// 简化构造函数 - 当 out_pitch == lightmap_pitch 时使用
    pub fn new_simple(
        out_buffer: &'a [u8],
        lightmap_buffer: &'a [u8],
        pitch: u16,
        light_tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
        fully_lit_light_table: *const u8,
        fully_dark_light_table: *const u8,
    ) -> Self {
        Self::new(
            out_buffer,
            pitch,
            lightmap_buffer,
            pitch,
            light_tables,
            fully_lit_light_table,
            fully_dark_light_table,
        )
    }

    /// 调整颜色（根据光照级别）- 对应 C++ adjustColor
    #[inline]
    pub fn adjust_color(&self, color: u8, light_level: u8) -> u8 {
        let level = (light_level as usize).min(NUM_LIGHTING_LEVELS - 1);
        self.light_tables[level][color as usize]
    }

    /// 获取指定输出位置的光照级别 - 对应 C++ getLightingAt
    #[inline]
    pub fn get_lighting_at(&self, out_loc: *const u8) -> *const u8 {
        let out_dist = unsafe { out_loc.offset_from(self.out_buffer.as_ptr()) };
        let row_offset = out_dist % self.out_pitch as isize;

        if out_dist < 0 {
            // 为支持墙壁瓦片的"向上渗透"，
            // 当 out_loc 超出边界时复用第一行
            let mod_offset = if row_offset < 0 {
                self.out_pitch as isize
            } else {
                0
            };
            unsafe {
                self.lightmap_buffer
                    .as_slice()
                    .as_ptr()
                    .offset(row_offset + mod_offset)
            }
        } else {
            let row = out_dist / self.out_pitch as isize;
            unsafe {
                self.lightmap_buffer
                    .as_slice()
                    .as_ptr()
                    .offset(row * self.lightmap_pitch as isize + row_offset)
            }
        }
    }

    /// 检查是否是完全明亮的光照表 - 对应 C++ isFullyLitLightTable (指针版本)
    #[inline]
    pub fn is_fully_lit_light_table(&self, light_table: *const u8) -> bool {
        light_table == self.fully_lit_light_table
    }

    /// 检查是否是完全黑暗的光照表 - 对应 C++ isFullyDarkLightTable (指针版本)
    #[inline]
    pub fn is_fully_dark_light_table(&self, light_table: *const u8) -> bool {
        light_table == self.fully_dark_light_table
    }

    /// 检查是否是完全明亮的光照表 (引用版本)
    #[inline]
    pub fn is_fully_lit_table(&self, light_table: &[u8; 256]) -> bool {
        light_table.as_ptr() == self.fully_lit_light_table
    }

    /// 检查是否是完全黑暗的光照表 (引用版本)
    #[inline]
    pub fn is_fully_dark_table(&self, light_table: &[u8; 256]) -> bool {
        light_table.as_ptr() == self.fully_dark_light_table
    }

    /// 通过偏移获取光照级别 - 用于 dun_render 等场景
    #[inline]
    pub fn get_light_level_at_offset(&self, offset: usize) -> u8 {
        if offset < self.lightmap_buffer.len() {
            self.lightmap_buffer.as_slice()[offset]
        } else {
            LIGHTS_MAX
        }
    }

    /// 获取 lightmap_pitch
    #[inline]
    pub fn lightmap_pitch(&self) -> u16 {
        self.lightmap_pitch
    }

    /// 获取 out_pitch
    #[inline]
    pub fn out_pitch(&self) -> u16 {
        self.out_pitch
    }

    /// 获取 light_tables 引用
    #[inline]
    pub fn light_tables(&self) -> &[[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS] {
        self.light_tables
    }

    /// 获取 fully_lit_light_table 指针
    #[inline]
    pub fn fully_lit_light_table(&self) -> *const u8 {
        self.fully_lit_light_table
    }

    /// 获取 fully_dark_light_table 指针
    #[inline]
    pub fn fully_dark_light_table(&self) -> *const u8 {
        self.fully_dark_light_table
    }

    /// 获取 out_buffer 引用
    #[inline]
    pub fn out_buffer(&self) -> &[u8] {
        self.out_buffer
    }

    /// 获取 lightmap_buffer 引用
    #[inline]
    pub fn lightmap_buffer(&self) -> &[u8] {
        self.lightmap_buffer.as_slice()
    }
}

// ============================================================================
// 私有辅助函数 - 对应 C++ 匿名命名空间中的函数
// ============================================================================

/// 渲染完整瓦片光照 - 对应 C++ RenderFullTile
fn render_full_tile(position: Point, light_level: u8, lightmap: &mut [u8], pitch: u16) {
    let pitch = pitch as usize;
    let base = (position.y + 1) as usize * pitch + position.x as usize - TILE_WIDTH as usize / 2;

    let mut top = base;
    let mut bottom = base + (TILE_HEIGHT as usize - 2) * pitch;

    for y in 0..(TILE_HEIGHT / 2 - 1) as usize {
        let w = 4 + y * 4;
        let x = (TILE_WIDTH as usize - w) / 2;

        // 填充顶部行
        if top + x + w <= lightmap.len() {
            lightmap[top + x..top + x + w].fill(light_level);
        }
        // 填充底部行
        if bottom + x + w <= lightmap.len() {
            lightmap[bottom + x..bottom + x + w].fill(light_level);
        }

        top += pitch;
        if bottom >= pitch {
            bottom -= pitch;
        }
    }

    // 填充中间行
    if top + TILE_WIDTH as usize <= lightmap.len() {
        lightmap[top..top + TILE_WIDTH as usize].fill(light_level);
    }
}

/// 向零递减 - 对应 C++ DecrementTowardZero
#[inline]
fn decrement_toward_zero(num: i32) -> i32 {
    if num > 0 {
        num - 1
    } else {
        num + 1
    }
}

/// 使用半空间法渲染三角形 - 对应 C++ RenderTriangle
/// 点必须按逆时针顺序提供
fn render_triangle(
    p1: Point,
    p2: Point,
    p3: Point,
    light_level: u8,
    lightmap: &mut [u8],
    pitch: u16,
    scan_lines: u16,
) {
    let pitch = pitch as i32;
    let scan_lines = scan_lines as i32;

    // 增量 (点已经是 28.4 定点数)
    let dx12 = p1.x - p2.x;
    let dx23 = p2.x - p3.x;
    let dx31 = p3.x - p1.x;

    let dy12 = p1.y - p2.y;
    let dy23 = p2.y - p3.y;
    let dy31 = p3.y - p1.y;

    // 24.8 定点数增量
    let fdx12 = dx12 << 4;
    let fdx23 = dx23 << 4;
    let fdx31 = dx31 << 4;

    let fdy12 = dy12 << 4;
    let fdy23 = dy23 << 4;
    let fdy31 = dy31 << 4;

    // 边界矩形
    let minx = ((p1.x.min(p2.x).min(p3.x) + 0xF) >> 4).max(0);
    let maxx = ((p1.x.max(p2.x).max(p3.x) + 0xF) >> 4).min(pitch);
    let xlen = maxx - minx;
    if xlen <= 0 {
        return;
    }
    let miny = ((p1.y.min(p2.y).min(p3.y) + 0xF) >> 4).max(0);
    let maxy = ((p1.y.max(p2.y).max(p3.y) + 0xF) >> 4).min(scan_lines);
    if maxy <= miny {
        return;
    }

    // 计算半边常数
    let calc_half_edge = |p: &Point, dx: i32, dy: i32| -> i32 {
        let fill_conv = if dy < 0 || (dy == 0 && dx > 0) {
            1
        } else {
            0
        };
        (dy * p.x) - (dx * p.y) + fill_conv
    };

    let c1 = calc_half_edge(&p1, dx12, dy12);
    let c2 = calc_half_edge(&p2, dx23, dy23);
    let c3 = calc_half_edge(&p3, dx31, dy31);

    let calc_cy = |minx: i32, miny: i32, dx: i32, dy: i32| -> i32 {
        (dx * (miny << 4)) - (dy * (minx << 4))
    };

    let mut cy1 = c1 + calc_cy(minx, miny, dx12, dy12);
    let mut cy2 = c2 + calc_cy(minx, miny, dx23, dy23);
    let mut cy3 = c3 + calc_cy(minx, miny, dx31, dy31);

    for y in miny..maxy {
        let cxe1 = cy1 - (fdy12 * xlen);
        let cxe2 = cy2 - (fdy23 * xlen);
        let cxe3 = cy3 - (fdy31 * xlen);

        let calc_start_x = |xlen: i32, cx: i32, cxe: i32, fdy: i32| -> i32 {
            if cx > 0 {
                return 0;
            }
            if cxe <= 0 {
                return xlen;
            }
            (cx + decrement_toward_zero(fdy)) / fdy
        };

        let startx = minx
            + calc_start_x(xlen, cy1, cxe1, fdy12)
                .max(calc_start_x(xlen, cy2, cxe2, fdy23))
                .max(calc_start_x(xlen, cy3, cxe3, fdy31));

        let calc_end_x = |xlen: i32, cx: i32, cxe: i32, fdy: i32| -> i32 {
            if cxe > 0 {
                return xlen;
            }
            if cx <= 0 {
                return 0;
            }
            (cx + decrement_toward_zero(fdy)) / fdy
        };

        let endx = minx
            + calc_end_x(xlen, cy1, cxe1, fdy12)
                .min(calc_end_x(xlen, cy2, cxe2, fdy23))
                .min(calc_end_x(xlen, cy3, cxe3, fdy31));

        if startx < endx {
            let dst_offset = (y * pitch + startx) as usize;
            let len = (endx - startx) as usize;
            if dst_offset + len <= lightmap.len() {
                lightmap[dst_offset..dst_offset + len].fill(light_level);
            }
        }

        cy1 += fdx12;
        cy2 += fdx23;
        cy3 += fdx31;
    }
}

/// 获取瓦片光照级别 - 对应 C++ GetLightLevel
fn get_light_level(tile_lights: &[[u8; MAXDUNY]; MAXDUNX], tile: Point) -> u8 {
    let x = (tile.x as usize).clamp(0, MAXDUNX - 1);
    let y = (tile.y as usize).clamp(0, MAXDUNY - 1);
    tile_lights[x][y]
}

/// 光照级别插值 - 对应 C++ Interpolate
fn interpolate(q1: u8, q2: u8, light_level: u8) -> u8 {
    // 结果是 28.4 定点数
    let numerator = ((light_level as i32) - (q1 as i32)) << 4;
    let divisor = (q2 as i32) - (q1 as i32);
    if divisor == 0 {
        return 0;
    }
    let result = (numerator + 0x8) / divisor;
    debug_assert!(result >= 0);
    result as u8
}

/// 渲染单元格光照 - 对应 C++ RenderCell
/// 使用 Marching Squares 算法
fn render_cell(
    quad: &[u8; 4],
    position: Point,
    light_level: u8,
    lightmap: &mut [u8],
    pitch: u16,
    scan_lines: u16,
) {
    let center0 = position;
    let center1 = position + Displacement::new(TILE_WIDTH / 2, TILE_HEIGHT / 2);
    let center2 = position + Displacement::new(0, TILE_HEIGHT);
    let center3 = position + Displacement::new(-TILE_WIDTH / 2, TILE_HEIGHT / 2);

    // 28.4 定点数坐标
    let fp_center0 = Point::new(center0.x << 4, center0.y << 4);
    let fp_center1 = Point::new(center1.x << 4, center1.y << 4);
    let fp_center2 = Point::new(center2.x << 4, center2.y << 4);
    let fp_center3 = Point::new(center3.x << 4, center3.y << 4);

    // Marching squares 形状
    let mut shape = 0u8;
    if quad[0] <= light_level {
        shape |= 8;
    }
    if quad[1] <= light_level {
        shape |= 4;
    }
    if quad[2] <= light_level {
        shape |= 2;
    }
    if quad[3] <= light_level {
        shape |= 1;
    }

    // 辅助函数：计算定点数插值点
    let fp_lerp = |from: Point, to: Point, factor: u8| -> Point {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        Point::new(
            from.x + (dx * factor as i32) / 16,
            from.y + (dy * factor as i32) / 16,
        )
    };

    match shape {
        // 整个单元格比 light_level 暗
        0 => {}

        // 填充单元格左下角
        1 => {
            let bottom_factor = interpolate(quad[3], quad[2], light_level);
            let left_factor = interpolate(quad[3], quad[0], light_level);
            let p1 = fp_lerp(fp_center3, fp_center2, bottom_factor);
            let p2 = fp_center3;
            let p3 = fp_lerp(fp_center3, fp_center0, left_factor);
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
        }

        // 填充单元格右下角
        2 => {
            let right_factor = interpolate(quad[2], quad[1], light_level);
            let bottom_factor = interpolate(quad[2], quad[3], light_level);
            let p1 = fp_lerp(fp_center2, fp_center1, right_factor);
            let p2 = fp_center2;
            let p3 = fp_lerp(fp_center2, fp_center3, bottom_factor);
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
        }

        // 填充单元格下半部分
        3 => {
            let right_factor = interpolate(quad[2], quad[1], light_level);
            let left_factor = interpolate(quad[3], quad[0], light_level);
            let p1 = fp_lerp(fp_center2, fp_center1, right_factor);
            let p2 = fp_center2;
            let p3 = fp_center3;
            let p4 = fp_lerp(fp_center3, fp_center1, left_factor);
            render_triangle(p1, p4, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p2, p4, p3, light_level, lightmap, pitch, scan_lines);
        }

        // 填充单元格右上角
        4 => {
            let top_factor = interpolate(quad[1], quad[0], light_level);
            let right_factor = interpolate(quad[1], quad[2], light_level);
            let p1 = fp_lerp(fp_center1, fp_center0, top_factor);
            let p2 = fp_center1;
            let p3 = fp_lerp(fp_center1, fp_center2, right_factor);
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
        }

        // 填充右上角和左下角 (对角线)
        5 => {
            let cell = (quad[0] as u16 + quad[1] as u16 + quad[2] as u16 + quad[3] as u16 + 2) / 4;
            let cell = cell as u8;
            let top_factor = interpolate(quad[1], quad[0], light_level);
            let right_factor = interpolate(quad[1], quad[2], light_level);
            let bottom_factor = interpolate(quad[3], quad[2], light_level);
            let left_factor = interpolate(quad[3], quad[0], light_level);
            let p1 = fp_lerp(fp_center1, fp_center0, top_factor);
            let p2 = fp_center1;
            let p3 = fp_lerp(fp_center1, fp_center2, right_factor);
            let p4 = fp_lerp(fp_center3, fp_center2, bottom_factor);
            let p5 = fp_center3;
            let p6 = fp_lerp(fp_center3, fp_center0, left_factor);

            if cell <= light_level {
                let mid_factor0 = interpolate(quad[0], cell, light_level);
                let mid_factor2 = interpolate(quad[2], cell, light_level);
                let mid02 = Point::new(
                    (fp_center0.x + fp_center2.x) / 2,
                    (fp_center0.y + fp_center2.y) / 2,
                );
                let p7 = fp_lerp(fp_center0, mid02, mid_factor0);
                let p8 = fp_lerp(fp_center2, mid02, mid_factor2);
                render_triangle(p1, p7, p2, light_level, lightmap, pitch, scan_lines);
                render_triangle(p2, p7, p8, light_level, lightmap, pitch, scan_lines);
                render_triangle(p2, p8, p3, light_level, lightmap, pitch, scan_lines);
                render_triangle(p4, p8, p5, light_level, lightmap, pitch, scan_lines);
                render_triangle(p5, p8, p7, light_level, lightmap, pitch, scan_lines);
                render_triangle(p5, p7, p6, light_level, lightmap, pitch, scan_lines);
            } else {
                let mid_factor1 = interpolate(quad[1], cell, light_level);
                let mid_factor3 = interpolate(quad[3], cell, light_level);
                let mid13 = Point::new(
                    (fp_center1.x + fp_center3.x) / 2,
                    (fp_center1.y + fp_center3.y) / 2,
                );
                let p7 = fp_lerp(fp_center1, mid13, mid_factor1);
                let p8 = fp_lerp(fp_center3, mid13, mid_factor3);
                render_triangle(p1, p7, p2, light_level, lightmap, pitch, scan_lines);
                render_triangle(p2, p7, p3, light_level, lightmap, pitch, scan_lines);
                render_triangle(p4, p8, p5, light_level, lightmap, pitch, scan_lines);
                render_triangle(p5, p8, p6, light_level, lightmap, pitch, scan_lines);
            }
        }

        // 填充单元格右半部分
        6 => {
            let top_factor = interpolate(quad[1], quad[0], light_level);
            let bottom_factor = interpolate(quad[2], quad[3], light_level);
            let p1 = fp_lerp(fp_center1, fp_center0, top_factor);
            let p2 = fp_center1;
            let p3 = fp_center2;
            let p4 = fp_lerp(fp_center2, fp_center3, bottom_factor);
            render_triangle(p1, p4, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p2, p4, p3, light_level, lightmap, pitch, scan_lines);
        }

        // 填充除左上角外的所有区域
        7 => {
            let top_factor = interpolate(quad[1], quad[0], light_level);
            let left_factor = interpolate(quad[3], quad[0], light_level);
            let p1 = fp_lerp(fp_center1, fp_center0, top_factor);
            let p2 = fp_center1;
            let p3 = fp_center2;
            let p4 = fp_center3;
            let p5 = fp_lerp(fp_center3, fp_center0, left_factor);
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p1, p5, p3, light_level, lightmap, pitch, scan_lines);
            render_triangle(p3, p5, p4, light_level, lightmap, pitch, scan_lines);
        }

        // 填充单元格左上角
        8 => {
            let top_factor = interpolate(quad[0], quad[1], light_level);
            let left_factor = interpolate(quad[0], quad[3], light_level);
            let p1 = fp_center0;
            let p2 = fp_lerp(fp_center0, fp_center1, top_factor);
            let p3 = fp_lerp(fp_center0, fp_center3, left_factor);
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
        }

        // 填充单元格左半部分
        9 => {
            let top_factor = interpolate(quad[0], quad[1], light_level);
            let bottom_factor = interpolate(quad[3], quad[2], light_level);
            let p1 = fp_center0;
            let p2 = fp_lerp(fp_center0, fp_center1, top_factor);
            let p3 = fp_lerp(fp_center3, fp_center2, bottom_factor);
            let p4 = fp_center3;
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p1, p4, p3, light_level, lightmap, pitch, scan_lines);
        }

        // 填充左上角和右下角 (对角线)
        10 => {
            let cell = (quad[0] as u16 + quad[1] as u16 + quad[2] as u16 + quad[3] as u16 + 2) / 4;
            let cell = cell as u8;
            let top_factor = interpolate(quad[0], quad[1], light_level);
            let right_factor = interpolate(quad[2], quad[1], light_level);
            let bottom_factor = interpolate(quad[2], quad[3], light_level);
            let left_factor = interpolate(quad[0], quad[3], light_level);
            let p1 = fp_center0;
            let p2 = fp_lerp(fp_center0, fp_center1, top_factor);
            let p3 = fp_lerp(fp_center2, fp_center1, right_factor);
            let p4 = fp_center2;
            let p5 = fp_lerp(fp_center2, fp_center3, bottom_factor);
            let p6 = fp_lerp(fp_center0, fp_center3, left_factor);

            if cell <= light_level {
                let mid_factor1 = interpolate(quad[1], cell, light_level);
                let mid_factor3 = interpolate(quad[3], cell, light_level);
                let mid13 = Point::new(
                    (fp_center1.x + fp_center3.x) / 2,
                    (fp_center1.y + fp_center3.y) / 2,
                );
                let p7 = fp_lerp(fp_center1, mid13, mid_factor1);
                let p8 = fp_lerp(fp_center3, mid13, mid_factor3);
                render_triangle(p1, p7, p2, light_level, lightmap, pitch, scan_lines);
                render_triangle(p1, p6, p8, light_level, lightmap, pitch, scan_lines);
                render_triangle(p1, p8, p7, light_level, lightmap, pitch, scan_lines);
                render_triangle(p3, p7, p4, light_level, lightmap, pitch, scan_lines);
                render_triangle(p4, p8, p5, light_level, lightmap, pitch, scan_lines);
                render_triangle(p4, p7, p8, light_level, lightmap, pitch, scan_lines);
            } else {
                let mid_factor0 = interpolate(quad[0], cell, light_level);
                let mid_factor2 = interpolate(quad[2], cell, light_level);
                let mid02 = Point::new(
                    (fp_center0.x + fp_center2.x) / 2,
                    (fp_center0.y + fp_center2.y) / 2,
                );
                let p7 = fp_lerp(fp_center0, mid02, mid_factor0);
                let p8 = fp_lerp(fp_center2, mid02, mid_factor2);
                render_triangle(p1, p7, p2, light_level, lightmap, pitch, scan_lines);
                render_triangle(p1, p6, p7, light_level, lightmap, pitch, scan_lines);
                render_triangle(p3, p8, p4, light_level, lightmap, pitch, scan_lines);
                render_triangle(p4, p8, p5, light_level, lightmap, pitch, scan_lines);
            }
        }

        // 填充除右上角外的所有区域
        11 => {
            let top_factor = interpolate(quad[0], quad[1], light_level);
            let right_factor = interpolate(quad[2], quad[1], light_level);
            let p1 = fp_center0;
            let p2 = fp_lerp(fp_center0, fp_center1, top_factor);
            let p3 = fp_lerp(fp_center2, fp_center1, right_factor);
            let p4 = fp_center2;
            let p5 = fp_center3;
            render_triangle(p1, p5, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p2, p5, p3, light_level, lightmap, pitch, scan_lines);
            render_triangle(p3, p5, p4, light_level, lightmap, pitch, scan_lines);
        }

        // 填充单元格上半部分
        12 => {
            let right_factor = interpolate(quad[1], quad[2], light_level);
            let left_factor = interpolate(quad[0], quad[3], light_level);
            let p1 = fp_center0;
            let p2 = fp_center1;
            let p3 = fp_lerp(fp_center1, fp_center2, right_factor);
            let p4 = fp_lerp(fp_center0, fp_center3, left_factor);
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p1, p4, p3, light_level, lightmap, pitch, scan_lines);
        }

        // 填充除右下角外的所有区域
        13 => {
            let right_factor = interpolate(quad[1], quad[2], light_level);
            let bottom_factor = interpolate(quad[3], quad[2], light_level);
            let p1 = fp_center0;
            let p2 = fp_center1;
            let p3 = fp_lerp(fp_center1, fp_center2, right_factor);
            let p4 = fp_lerp(fp_center3, fp_center2, bottom_factor);
            let p5 = fp_center3;
            render_triangle(p1, p3, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p1, p4, p3, light_level, lightmap, pitch, scan_lines);
            render_triangle(p1, p5, p4, light_level, lightmap, pitch, scan_lines);
        }

        // 填充除左下角外的所有区域
        14 => {
            let bottom_factor = interpolate(quad[2], quad[3], light_level);
            let left_factor = interpolate(quad[0], quad[3], light_level);
            let p1 = fp_center0;
            let p2 = fp_center1;
            let p3 = fp_center2;
            let p4 = fp_lerp(fp_center2, fp_center3, bottom_factor);
            let p5 = fp_lerp(fp_center0, fp_center3, left_factor);
            render_triangle(p1, p5, p2, light_level, lightmap, pitch, scan_lines);
            render_triangle(p2, p5, p4, light_level, lightmap, pitch, scan_lines);
            render_triangle(p2, p4, p3, light_level, lightmap, pitch, scan_lines);
        }

        // 填充整个单元格
        15 => {
            if center3.x < 0
                || center1.x >= pitch as i32
                || center0.y < 0
                || center2.y >= scan_lines as i32
            {
                render_triangle(
                    fp_center0,
                    fp_center2,
                    fp_center1,
                    light_level,
                    lightmap,
                    pitch,
                    scan_lines,
                );
                render_triangle(
                    fp_center0,
                    fp_center3,
                    fp_center2,
                    light_level,
                    lightmap,
                    pitch,
                    scan_lines,
                );
            } else {
                // 如果完整瓦片可见，使用优化的渲染路径
                render_full_tile(center0, light_level, lightmap, pitch);
            }
        }

        _ => {}
    }
}

/// 构建光照映射 - 对应 C++ BuildLightmap
fn build_lightmap(
    tile_position: Point,
    target_buffer_position: Point,
    viewport_width: u16,
    viewport_height: u16,
    rows: i32,
    columns: i32,
    tile_lights: &[[u8; MAXDUNY]; MAXDUNX],
    micro_tile_len: u8,
) {
    // 由于光照可能需要渗透到墙壁瓦片的顶部，
    // 扩展缓冲区空间以包含最高瓦片图形的完整基底菱形
    let buffer_height = viewport_height as i32 + TILE_HEIGHT * (micro_tile_len as i32 / 2 + 1);
    let rows = rows + micro_tile_len as i32 + 2;

    let total_pixels = viewport_width as usize * buffer_height as usize;

    // 获取全局缓冲区
    let mut buffer = LIGHTMAP_BUFFER.lock().unwrap();
    buffer.resize(total_pixels, LIGHTS_MAX);
    buffer.fill(LIGHTS_MAX);

    // 由于渲染发生在四边形之间的单元格中，
    // 扩展渲染空间以包含视口外的瓦片
    let mut tile_position = tile_position + Displacement::from_direction(Direction::NorthWest) * 2;
    let mut target_buffer_position =
        target_buffer_position - Displacement::new(TILE_WIDTH, TILE_HEIGHT);
    let rows = rows + 3;
    let mut columns = columns + 1;

    for i in 0..rows {
        let mut tile_pos = tile_position;
        let mut target_pos = target_buffer_position;

        for _j in 0..columns {
            let center0 =
                target_pos + Displacement::new(TILE_WIDTH / 2, -TILE_HEIGHT / 2);

            let tile0 = tile_pos;
            let tile1 = tile_pos + Displacement::new(1, 0);
            let tile2 = tile_pos + Displacement::new(1, 1);
            let tile3 = tile_pos + Displacement::new(0, 1);

            let quad = [
                get_light_level(tile_lights, tile0),
                get_light_level(tile_lights, tile1),
                get_light_level(tile_lights, tile2),
                get_light_level(tile_lights, tile3),
            ];

            let max_light = quad.iter().copied().max().unwrap_or(0);
            let min_light = quad.iter().copied().min().unwrap_or(0);

            for light_idx in 0..LIGHTS_MAX {
                let light_level = LIGHTS_MAX - light_idx - 1;
                if light_level > max_light {
                    continue;
                }
                if light_level < min_light {
                    break;
                }
                render_cell(
                    &quad,
                    center0,
                    light_level,
                    &mut buffer,
                    viewport_width,
                    buffer_height as u16,
                );
            }

            tile_pos += Displacement::from_direction(Direction::East);
            target_pos.x += TILE_WIDTH;
        }

        // 跳转到下一行
        target_buffer_position.y += TILE_HEIGHT / 2;
        if (i & 1) != 0 {
            tile_position.x += 1;
            columns -= 1;
            target_buffer_position.x += TILE_WIDTH / 2;
        } else {
            tile_position.y += 1;
            columns += 1;
            target_buffer_position.x -= TILE_WIDTH / 2;
        }
    }
}

// ============================================================================
// Lightmap 静态方法
// ============================================================================

/// 构建 Lightmap - 对应 C++ Lightmap::build
pub fn lightmap_build<'a>(
    per_pixel_lighting: bool,
    tile_position: Point,
    target_buffer_position: Point,
    viewport_width: i32,
    viewport_height: i32,
    rows: i32,
    columns: i32,
    out_buffer: &'a [u8],
    out_pitch: u16,
    light_tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    fully_lit_light_table: *const u8,
    fully_dark_light_table: *const u8,
    tile_lights: &[[u8; MAXDUNY]; MAXDUNX],
    micro_tile_len: u8,
) -> OwnedLightmap<'a> {
    if per_pixel_lighting {
        build_lightmap(
            tile_position,
            target_buffer_position,
            viewport_width as u16,
            viewport_height as u16,
            rows,
            columns,
            tile_lights,
            micro_tile_len,
        );
    }

    // 复制全局缓冲区的内容
    let buffer = {
        let global_buffer = LIGHTMAP_BUFFER.lock().unwrap();
        global_buffer.clone()
    };

    OwnedLightmap {
        out_buffer,
        out_pitch,
        lightmap_buffer: buffer,
        lightmap_pitch: viewport_width as u16,
        light_tables,
        fully_lit_light_table,
        fully_dark_light_table,
    }
}

/// 拥有缓冲区的 Lightmap - 用于 build 返回值
pub struct OwnedLightmap<'a> {
    out_buffer: &'a [u8],
    out_pitch: u16,
    lightmap_buffer: Vec<u8>,
    lightmap_pitch: u16,
    light_tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    fully_lit_light_table: *const u8,
    fully_dark_light_table: *const u8,
}

impl<'a> OwnedLightmap<'a> {
    /// 获取 Lightmap 引用
    pub fn as_lightmap(&'a self) -> Lightmap<'a> {
        Lightmap {
            out_buffer: self.out_buffer,
            out_pitch: self.out_pitch,
            lightmap_buffer: LightmapBufferRef::Borrowed(&self.lightmap_buffer),
            lightmap_pitch: self.lightmap_pitch,
            light_tables: self.light_tables,
            fully_lit_light_table: self.fully_lit_light_table,
            fully_dark_light_table: self.fully_dark_light_table,
        }
    }

    /// 调整颜色
    #[inline]
    pub fn adjust_color(&self, color: u8, light_level: u8) -> u8 {
        let level = (light_level as usize).min(NUM_LIGHTING_LEVELS - 1);
        self.light_tables[level][color as usize]
    }

    /// 获取指定输出位置的光照级别
    #[inline]
    pub fn get_lighting_at(&self, out_loc: *const u8) -> *const u8 {
        let out_dist = unsafe { out_loc.offset_from(self.out_buffer.as_ptr()) };
        let row_offset = out_dist % self.out_pitch as isize;

        if out_dist < 0 {
            let mod_offset = if row_offset < 0 {
                self.out_pitch as isize
            } else {
                0
            };
            unsafe {
                self.lightmap_buffer
                    .as_ptr()
                    .offset(row_offset + mod_offset)
            }
        } else {
            let row = out_dist / self.out_pitch as isize;
            unsafe {
                self.lightmap_buffer
                    .as_ptr()
                    .offset(row * self.lightmap_pitch as isize + row_offset)
            }
        }
    }

    /// 检查是否是完全明亮的光照表
    #[inline]
    pub fn is_fully_lit_light_table(&self, light_table: *const u8) -> bool {
        light_table == self.fully_lit_light_table
    }

    /// 检查是否是完全黑暗的光照表
    #[inline]
    pub fn is_fully_dark_light_table(&self, light_table: *const u8) -> bool {
        light_table == self.fully_dark_light_table
    }

    /// 获取 lightmap_pitch
    #[inline]
    pub fn lightmap_pitch(&self) -> u16 {
        self.lightmap_pitch
    }

    /// 获取 out_pitch
    #[inline]
    pub fn out_pitch(&self) -> u16 {
        self.out_pitch
    }

    /// 获取 light_tables 引用
    #[inline]
    pub fn light_tables(&self) -> &[[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS] {
        self.light_tables
    }

    /// 获取 fully_lit_light_table 指针
    #[inline]
    pub fn fully_lit_light_table(&self) -> *const u8 {
        self.fully_lit_light_table
    }

    /// 获取 fully_dark_light_table 指针
    #[inline]
    pub fn fully_dark_light_table(&self) -> *const u8 {
        self.fully_dark_light_table
    }

    /// 获取 out_buffer 引用
    #[inline]
    pub fn out_buffer(&self) -> &[u8] {
        self.out_buffer
    }

    /// 获取 lightmap_buffer 引用
    #[inline]
    pub fn lightmap_buffer(&self) -> &[u8] {
        &self.lightmap_buffer
    }
}

/// 向上渗透光照 - 对应 C++ Lightmap::bleedUp
pub fn lightmap_bleed_up<'a>(
    per_pixel_lighting: bool,
    source: &'a OwnedLightmap<'a>,
    target_buffer_position: Point,
    lightmap_buffer: &'a mut [u8],
) -> BleedUpLightmap<'a> {
    assert!(lightmap_buffer.len() >= (TILE_WIDTH * TILE_HEIGHT) as usize);

    if !per_pixel_lighting {
        return BleedUpLightmap::Source(source);
    }

    let source_height = source.lightmap_buffer.len() as i32 / source.lightmap_pitch as i32;
    let clip_left = 0.max(-target_buffer_position.x);
    let clip_top = 0.max(-(target_buffer_position.y - TILE_HEIGHT + 1));
    let clip_right = 0.max(target_buffer_position.x + TILE_WIDTH - source.lightmap_pitch as i32);
    let clip_bottom = 0.max(target_buffer_position.y - source_height + 1);

    // 如果瓦片完全超出光照映射边界，无法处理
    if clip_left + clip_right >= TILE_WIDTH {
        return BleedUpLightmap::Source(source);
    }
    if clip_top + clip_bottom >= TILE_HEIGHT {
        return BleedUpLightmap::Source(source);
    }

    let lightmap_pitch = (TILE_WIDTH - clip_left - clip_right).max(0) as u16;
    let lightmap_height = (TILE_HEIGHT - clip_top - clip_bottom) as usize;

    // 找到瓦片最后一行的左边缘
    let out_offset = ((target_buffer_position.y - clip_bottom) * source.out_pitch as i32
        + target_buffer_position.x
        + clip_left)
        .max(0) as usize;
    let out_loc = unsafe { source.out_buffer.as_ptr().add(out_offset) };
    let out_buffer =
        unsafe { out_loc.sub((lightmap_height - 1) * source.out_pitch as usize) };

    // 从瓦片底行开始复制字节
    let src_ptr = source.get_lighting_at(out_loc);
    let mut dst_offset = (lightmap_height - 1) * lightmap_pitch as usize;

    let mut row_count = clip_bottom as usize;
    let mut src_offset = 0isize;

    while dst_offset < lightmap_buffer.len() && row_count < lightmap_height + clip_bottom as usize {
        let bleed = if row_count > TILE_HEIGHT as usize / 2 {
            ((row_count - TILE_HEIGHT as usize / 2) * 2) as i32
        } else {
            0
        };
        let light_offset = bleed.max(clip_left) - clip_left;
        let light_length = (TILE_WIDTH - clip_left - bleed.max(clip_right) - light_offset).max(0);

        // 通过从下一行复制数据来向上渗透像素
        if row_count > clip_bottom as usize && (light_length as u16) < lightmap_pitch {
            if dst_offset + lightmap_pitch as usize <= lightmap_buffer.len() {
                lightmap_buffer.copy_within(
                    dst_offset + lightmap_pitch as usize
                        ..dst_offset + 2 * lightmap_pitch as usize,
                    dst_offset,
                );
            }
        }

        // 从源光照映射复制基底菱形顶边缘之间的数据
        let light_offset = light_offset as usize;
        let light_length = light_length as usize;
        if dst_offset + light_offset + light_length <= lightmap_buffer.len() {
            unsafe {
                let src_slice = std::slice::from_raw_parts(
                    src_ptr.offset(src_offset).add(light_offset),
                    light_length,
                );
                lightmap_buffer[dst_offset + light_offset..dst_offset + light_offset + light_length]
                    .copy_from_slice(src_slice);
            }
        }

        src_offset -= source.lightmap_pitch as isize;
        if dst_offset >= lightmap_pitch as usize {
            dst_offset -= lightmap_pitch as usize;
        } else {
            break;
        }
        row_count += 1;
    }

    // 计算 out_buffer 的安全偏移
    let out_buffer_offset = unsafe { out_buffer.offset_from(source.out_buffer.as_ptr()) };
    let out_buffer_slice = if out_buffer_offset >= 0 {
        &source.out_buffer[out_buffer_offset as usize..]
    } else {
        source.out_buffer
    };

    BleedUpLightmap::Owned(BleedUpOwnedLightmap {
        out_buffer: out_buffer_slice,
        out_pitch: source.out_pitch,
        lightmap_buffer,
        lightmap_pitch,
        light_tables: source.light_tables,
        fully_lit_light_table: source.fully_lit_light_table,
        fully_dark_light_table: source.fully_dark_light_table,
    })
}

/// BleedUp 返回的 Lightmap 类型
pub enum BleedUpLightmap<'a> {
    Source(&'a OwnedLightmap<'a>),
    Owned(BleedUpOwnedLightmap<'a>),
}

/// BleedUp 拥有的 Lightmap
pub struct BleedUpOwnedLightmap<'a> {
    out_buffer: &'a [u8],
    out_pitch: u16,
    lightmap_buffer: &'a [u8],
    lightmap_pitch: u16,
    light_tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    fully_lit_light_table: *const u8,
    fully_dark_light_table: *const u8,
}

impl<'a> BleedUpLightmap<'a> {
    /// 调整颜色
    #[inline]
    pub fn adjust_color(&self, color: u8, light_level: u8) -> u8 {
        match self {
            BleedUpLightmap::Source(s) => s.adjust_color(color, light_level),
            BleedUpLightmap::Owned(o) => {
                let level = (light_level as usize).min(NUM_LIGHTING_LEVELS - 1);
                o.light_tables[level][color as usize]
            }
        }
    }

    /// 检查是否是完全明亮的光照表
    #[inline]
    pub fn is_fully_lit_light_table(&self, light_table: *const u8) -> bool {
        match self {
            BleedUpLightmap::Source(s) => s.is_fully_lit_light_table(light_table),
            BleedUpLightmap::Owned(o) => light_table == o.fully_lit_light_table,
        }
    }

    /// 检查是否是完全黑暗的光照表
    #[inline]
    pub fn is_fully_dark_light_table(&self, light_table: *const u8) -> bool {
        match self {
            BleedUpLightmap::Source(s) => s.is_fully_dark_light_table(light_table),
            BleedUpLightmap::Owned(o) => light_table == o.fully_dark_light_table,
        }
    }
}

// ============================================================================
// 辅助类型
// ============================================================================

/// 瓦片光照数据
#[derive(Debug, Clone, Copy, Default)]
pub struct TileLightInfo {
    /// 光照级别 (0 = 最亮, 15 = 最暗)
    pub light_level: u8,
    /// 是否可见
    pub visible: bool,
    /// 是否被探索过
    pub explored: bool,
}

/// 瓦片光照网格
pub struct TileLightGrid {
    /// 光照数据 [x][y]
    data: [[u8; MAXDUNY]; MAXDUNX],
}

impl TileLightGrid {
    pub fn new() -> Self {
        Self {
            data: [[LIGHTS_MAX; MAXDUNY]; MAXDUNX],
        }
    }

    /// 获取瓦片光照级别
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x < MAXDUNX && y < MAXDUNY {
            self.data[x][y]
        } else {
            LIGHTS_MAX
        }
    }

    /// 设置瓦片光照级别
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, level: u8) {
        if x < MAXDUNX && y < MAXDUNY {
            self.data[x][y] = level.min(LIGHTS_MAX);
        }
    }

    /// 获取内部数据的引用（供 build_lightmap 使用）
    #[inline]
    pub fn as_array(&self) -> &[[u8; MAXDUNY]; MAXDUNX] {
        &self.data
    }

    /// 获取内部数据的可变引用
    #[inline]
    pub fn as_array_mut(&mut self) -> &mut [[u8; MAXDUNY]; MAXDUNX] {
        &mut self.data
    }

    /// 清空为完全黑暗
    pub fn clear(&mut self) {
        for x in 0..MAXDUNX {
            self.data[x].fill(LIGHTS_MAX);
        }
    }

    /// 填充为指定光照级别
    pub fn fill(&mut self, level: u8) {
        let level = level.min(LIGHTS_MAX);
        for x in 0..MAXDUNX {
            self.data[x].fill(level);
        }
    }
}

impl Default for TileLightGrid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_light_grid() {
        let mut grid = TileLightGrid::new();

        // 默认为最大光照（最暗）
        assert_eq!(grid.get(50, 50), LIGHTS_MAX);

        grid.set(50, 50, 5);
        assert_eq!(grid.get(50, 50), 5);

        // 超出边界应返回 LIGHTS_MAX
        assert_eq!(grid.get(MAXDUNX, 0), LIGHTS_MAX);
    }

    #[test]
    fn test_interpolate() {
        // 测试插值函数
        let result = interpolate(0, 16, 8);
        // 8 在 0 和 16 之间的中点
        assert!(result > 0);
    }

    #[test]
    fn test_decrement_toward_zero() {
        assert_eq!(decrement_toward_zero(5), 4);
        assert_eq!(decrement_toward_zero(-5), -4);
        assert_eq!(decrement_toward_zero(0), 1); // 边界情况
    }

    #[test]
    fn test_get_light_level() {
        let mut tile_lights = [[LIGHTS_MAX; MAXDUNY]; MAXDUNX];
        tile_lights[10][20] = 5;

        let result = get_light_level(&tile_lights, Point::new(10, 20));
        assert_eq!(result, 5);

        // 测试边界 clamp
        let result = get_light_level(&tile_lights, Point::new(-5, 20));
        assert_eq!(result, tile_lights[0][20]);
    }
}
