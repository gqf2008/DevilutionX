//! 光照渲染 - 移植自 Source/engine/render/light_render.hpp
//!
//! 提供逐像素光照映射和光照表查找功能

use super::types::Point;

/// 光照表大小 (256 色)
pub const LIGHT_TABLE_SIZE: usize = 256;

/// 光照级别数量 (0-15)
pub const NUM_LIGHTING_LEVELS: usize = 16;

/// 最大地下城尺寸
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;

/// 光照映射
///
/// 存储每个像素的光照级别，用于逐像素光照渲染
pub struct LightRenderMap {
    /// 输出缓冲区的基地址（用于计算偏移）
    out_buffer_base: usize,
    /// 输出缓冲区的行距
    out_pitch: u16,
    /// 光照映射数据
    lightmap_buffer: Vec<u8>,
    /// 光照映射的行距
    lightmap_pitch: u16,
    /// 光照表 [光照级别][颜色索引] -> 调整后的颜色索引
    light_tables: [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    /// 完全明亮的光照表索引
    fully_lit_table_index: usize,
    /// 完全黑暗的光照表索引
    fully_dark_table_index: usize,
}

impl LightRenderMap {
    /// 创建新的光照映射
    pub fn new(out_pitch: u16, lightmap_pitch: u16, width: usize, height: usize) -> Self {
        Self {
            out_buffer_base: 0,
            out_pitch,
            lightmap_buffer: vec![0; width * height],
            lightmap_pitch,
            light_tables: [[0; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
            fully_lit_table_index: 0,
            fully_dark_table_index: NUM_LIGHTING_LEVELS - 1,
        }
    }

    /// 设置光照表
    pub fn set_light_tables(&mut self, tables: &[[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS]) {
        self.light_tables = *tables;
    }

    /// 设置单个光照表
    pub fn set_light_table(&mut self, level: usize, table: &[u8; LIGHT_TABLE_SIZE]) {
        if level < NUM_LIGHTING_LEVELS {
            self.light_tables[level] = *table;
        }
    }

    /// 调整颜色（根据光照级别）
    #[inline]
    pub fn adjust_color(&self, color: u8, light_level: u8) -> u8 {
        let level = (light_level as usize).min(NUM_LIGHTING_LEVELS - 1);
        self.light_tables[level][color as usize]
    }

    /// 获取指定像素位置的光照级别
    #[inline]
    pub fn get_light_level_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 {
            return NUM_LIGHTING_LEVELS as u8 - 1; // 完全黑暗
        }

        let idx = (y as usize) * (self.lightmap_pitch as usize) + (x as usize);
        if idx < self.lightmap_buffer.len() {
            self.lightmap_buffer[idx]
        } else {
            NUM_LIGHTING_LEVELS as u8 - 1
        }
    }

    /// 设置指定像素位置的光照级别
    pub fn set_light_level_at(&mut self, x: i32, y: i32, level: u8) {
        if x < 0 || y < 0 {
            return;
        }

        let idx = (y as usize) * (self.lightmap_pitch as usize) + (x as usize);
        if idx < self.lightmap_buffer.len() {
            self.lightmap_buffer[idx] = level.min(NUM_LIGHTING_LEVELS as u8 - 1);
        }
    }

    /// 填充区域的光照级别
    pub fn fill_region(&mut self, x: i32, y: i32, width: i32, height: i32, level: u8) {
        let level = level.min(NUM_LIGHTING_LEVELS as u8 - 1);

        for dy in 0..height {
            for dx in 0..width {
                self.set_light_level_at(x + dx, y + dy, level);
            }
        }
    }

    /// 检查是否是完全明亮的光照表
    #[inline]
    pub fn is_fully_lit(&self, light_level: u8) -> bool {
        light_level as usize == self.fully_lit_table_index
    }

    /// 检查是否是完全黑暗的光照表
    #[inline]
    pub fn is_fully_dark(&self, light_level: u8) -> bool {
        light_level as usize == self.fully_dark_table_index
    }

    /// 获取光照表
    pub fn get_light_table(&self, level: usize) -> Option<&[u8; LIGHT_TABLE_SIZE]> {
        self.light_tables.get(level)
    }

    /// 清空光照映射（设置为完全黑暗）
    pub fn clear(&mut self) {
        let dark_level = self.fully_dark_table_index as u8;
        self.lightmap_buffer.fill(dark_level);
    }

    /// 清空为完全明亮
    pub fn clear_lit(&mut self) {
        let lit_level = self.fully_lit_table_index as u8;
        self.lightmap_buffer.fill(lit_level);
    }
}

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
    data: [[TileLightInfo; MAXDUNY]; MAXDUNX],
}

impl TileLightGrid {
    pub fn new() -> Self {
        Self {
            data: [[TileLightInfo::default(); MAXDUNY]; MAXDUNX],
        }
    }

    /// 获取瓦片光照信息
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&TileLightInfo> {
        if x < MAXDUNX && y < MAXDUNY {
            Some(&self.data[x][y])
        } else {
            None
        }
    }

    /// 获取瓦片光照信息（可变）
    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut TileLightInfo> {
        if x < MAXDUNX && y < MAXDUNY {
            Some(&mut self.data[x][y])
        } else {
            None
        }
    }

    /// 设置瓦片光照级别
    pub fn set_light_level(&mut self, x: usize, y: usize, level: u8) {
        if let Some(info) = self.get_mut(x, y) {
            info.light_level = level.min(NUM_LIGHTING_LEVELS as u8 - 1);
        }
    }

    /// 设置瓦片可见性
    pub fn set_visible(&mut self, x: usize, y: usize, visible: bool) {
        if let Some(info) = self.get_mut(x, y) {
            info.visible = visible;
            if visible {
                info.explored = true;
            }
        }
    }

    /// 清空所有可见性
    pub fn clear_visibility(&mut self) {
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.data[x][y].visible = false;
            }
        }
    }

    /// 重置所有数据
    pub fn reset(&mut self) {
        for x in 0..MAXDUNX {
            for y in 0..MAXDUNY {
                self.data[x][y] = TileLightInfo::default();
            }
        }
    }
}

impl Default for TileLightGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// 构建视口的逐像素光照映射
///
/// # Arguments
/// * `per_pixel_lighting` - 是否启用逐像素光照
/// * `tile_position` - 起始瓦片位置
/// * `target_position` - 目标缓冲区位置
/// * `viewport_width` - 视口宽度
/// * `viewport_height` - 视口高度
/// * `rows` - 行数
/// * `columns` - 列数
/// * `tile_lights` - 瓦片光照网格
/// * `micro_tile_len` - 微瓦片长度
pub fn build_lightmap_for_viewport(
    per_pixel_lighting: bool,
    tile_position: Point,
    target_position: Point,
    viewport_width: i32,
    viewport_height: i32,
    rows: i32,
    columns: i32,
    tile_lights: &TileLightGrid,
    micro_tile_len: u8,
) -> LightRenderMap {
    let mut lightmap = LightRenderMap::new(
        viewport_width as u16,
        viewport_width as u16,
        viewport_width as usize,
        viewport_height as usize,
    );

    if !per_pixel_lighting {
        // 非逐像素模式：使用瓦片级光照
        lightmap.clear_lit();
        return lightmap;
    }

    // 逐像素光照计算
    // 这里简化实现，实际需要根据瓦片位置插值

    let tile_size = micro_tile_len as i32;

    for row in 0..rows {
        for col in 0..columns {
            let tile_x = (tile_position.x + col) as usize;
            let tile_y = (tile_position.y + row) as usize;

            let light_level = if let Some(info) = tile_lights.get(tile_x, tile_y) {
                if info.visible {
                    info.light_level
                } else {
                    NUM_LIGHTING_LEVELS as u8 - 1
                }
            } else {
                NUM_LIGHTING_LEVELS as u8 - 1
            };

            // 填充此瓦片对应的像素区域
            let px = target_position.x + col * tile_size;
            let py = target_position.y + row * tile_size;
            lightmap.fill_region(px, py, tile_size, tile_size, light_level);
        }
    }

    lightmap
}

/// 光照渐变向上延伸
///
/// 用于墙壁瓦片：使用下方瓦片的光照向上延伸
pub fn bleed_lightmap_up(
    source: &LightRenderMap,
    target_position: Point,
    height: i32,
) -> Vec<u8> {
    let width = source.lightmap_pitch as usize;
    let mut buffer = vec![0u8; width * height as usize];

    // 复制第一行到所有行（简化实现）
    if height > 0 {
        let src_level = source.get_light_level_at(target_position.x, target_position.y);
        buffer.fill(src_level);
    }

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_render_map_creation() {
        let map = LightRenderMap::new(640, 640, 640, 480);
        assert_eq!(map.out_pitch, 640);
        assert_eq!(map.lightmap_pitch, 640);
    }

    #[test]
    fn test_light_render_map_adjust_color() {
        let mut map = LightRenderMap::new(100, 100, 100, 100);

        // 设置一个简单的光照表
        let mut table = [0u8; LIGHT_TABLE_SIZE];
        for i in 0..256 {
            table[i] = (i / 2) as u8; // 简单的暗化
        }
        map.set_light_table(5, &table);

        let adjusted = map.adjust_color(100, 5);
        assert_eq!(adjusted, 50);
    }

    #[test]
    fn test_tile_light_grid() {
        let mut grid = TileLightGrid::new();

        grid.set_light_level(50, 50, 5);
        grid.set_visible(50, 50, true);

        let info = grid.get(50, 50).unwrap();
        assert_eq!(info.light_level, 5);
        assert!(info.visible);
        assert!(info.explored);
    }

    #[test]
    fn test_tile_light_grid_clear_visibility() {
        let mut grid = TileLightGrid::new();

        grid.set_visible(50, 50, true);
        assert!(grid.get(50, 50).unwrap().visible);

        grid.clear_visibility();
        assert!(!grid.get(50, 50).unwrap().visible);
        assert!(grid.get(50, 50).unwrap().explored); // 探索状态保留
    }
}
