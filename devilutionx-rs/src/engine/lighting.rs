//! Lighting - 光照系统
//!
//! 移植自 Source/engine/lighting_defs.hpp 和 render/light_render.hpp
//!
//! Diablo 使用 16 级光照 (0-15)，每级对应一个颜色映射表

/// 最大光源数量
pub const MAX_LIGHTS: usize = 32;

/// 最大视野光源数量
pub const MAX_VISION: usize = 4;

/// 无光源标记
pub const NO_LIGHT: i32 = -1;

/// 最大光照级别 (15 = 最暗)
pub const LIGHTS_MAX: u8 = 15;

/// 光照表大小 (等于调色板大小)
pub const LIGHT_TABLE_SIZE: usize = 256;

/// 支持的光照级别数量 (0-15 共 16 级)
pub const NUM_LIGHTING_LEVELS: usize = LIGHTS_MAX as usize + 1;

/// 单个光照级别的颜色映射表
pub type LightingTable = [u8; LIGHT_TABLE_SIZE];

/// 所有光照级别的映射表
pub type AllLightingTables = [LightingTable; NUM_LIGHTING_LEVELS];

/// 光源
#[derive(Clone, Copy, Debug)]
pub struct Light {
    /// 光源位置 (地块坐标)
    pub x: i32,
    pub y: i32,
    /// 光照半径
    pub radius: i32,
    /// 光源 ID (用于查找)
    pub id: i32,
    /// 光源是否活动
    pub active: bool,
    /// 光源是否属于玩家
    pub player_controlled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            radius: 0,
            id: NO_LIGHT,
            active: false,
            player_controlled: false,
        }
    }
}

impl Light {
    /// 创建新光源
    pub fn new(x: i32, y: i32, radius: i32) -> Self {
        Self {
            x,
            y,
            radius,
            id: NO_LIGHT,
            active: true,
            player_controlled: false,
        }
    }

    /// 计算到指定点的光照级别
    ///
    /// 返回 0 (最亮) 到 15 (最暗)
    pub fn calculate_light_level(&self, target_x: i32, target_y: i32) -> u8 {
        if !self.active || self.radius <= 0 {
            return LIGHTS_MAX;
        }

        let dx = (target_x - self.x).abs();
        let dy = (target_y - self.y).abs();
        let distance = ((dx * dx + dy * dy) as f32).sqrt() as i32;

        if distance >= self.radius {
            return LIGHTS_MAX;
        }

        // 线性衰减
        ((distance * LIGHTS_MAX as i32) / self.radius).clamp(0, LIGHTS_MAX as i32) as u8
    }
}

/// 光照管理器
#[derive(Clone)]
pub struct LightManager {
    /// 光照表
    pub tables: AllLightingTables,
    /// 活动光源
    pub lights: [Light; MAX_LIGHTS],
    /// 活动光源数量
    pub num_lights: usize,
    /// 玩家视野光源
    pub vision_lights: [Light; MAX_VISION],
    /// 全局光照级别 (地下城黑暗度)
    pub ambient_light: u8,
}

impl Default for LightManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LightManager {
    /// 创建新的光照管理器
    pub fn new() -> Self {
        // 默认光照表 (恒等映射)
        let mut tables = [[0u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS];
        for table in &mut tables {
            for i in 0..LIGHT_TABLE_SIZE {
                table[i] = i as u8;
            }
        }

        Self {
            tables,
            lights: [Light::default(); MAX_LIGHTS],
            num_lights: 0,
            vision_lights: [Light::default(); MAX_VISION],
            ambient_light: 0,
        }
    }

    /// 从数据加载光照表
    pub fn load_tables(&mut self, data: &[u8]) -> bool {
        if data.len() < LIGHT_TABLE_SIZE * NUM_LIGHTING_LEVELS {
            return false;
        }

        for level in 0..NUM_LIGHTING_LEVELS {
            let offset = level * LIGHT_TABLE_SIZE;
            self.tables[level].copy_from_slice(&data[offset..offset + LIGHT_TABLE_SIZE]);
        }
        true
    }

    /// 获取指定光照级别的映射表
    #[inline]
    pub fn get_table(&self, level: u8) -> &LightingTable {
        &self.tables[(level as usize).min(LIGHTS_MAX as usize)]
    }

    /// 应用光照到颜色索引
    #[inline]
    pub fn apply_light(&self, level: u8, color: u8) -> u8 {
        self.tables[(level as usize).min(LIGHTS_MAX as usize)][color as usize]
    }

    /// 添加光源
    pub fn add_light(&mut self, x: i32, y: i32, radius: i32) -> Option<usize> {
        if self.num_lights >= MAX_LIGHTS {
            return None;
        }

        let index = self.num_lights;
        self.lights[index] = Light::new(x, y, radius);
        self.lights[index].id = index as i32;
        self.num_lights += 1;
        Some(index)
    }

    /// 移除光源
    pub fn remove_light(&mut self, id: i32) {
        if id < 0 || id as usize >= self.num_lights {
            return;
        }

        let index = id as usize;
        self.lights[index].active = false;
    }

    /// 移动光源
    pub fn move_light(&mut self, id: i32, x: i32, y: i32) {
        if id < 0 || id as usize >= self.num_lights {
            return;
        }

        let index = id as usize;
        self.lights[index].x = x;
        self.lights[index].y = y;
    }

    /// 改变光源半径
    pub fn change_light_radius(&mut self, id: i32, radius: i32) {
        if id < 0 || id as usize >= self.num_lights {
            return;
        }

        let index = id as usize;
        self.lights[index].radius = radius;
    }

    /// 计算指定位置的组合光照级别
    pub fn calculate_combined_light(&self, x: i32, y: i32) -> u8 {
        let mut min_level = self.ambient_light;

        // 检查所有活动光源
        for i in 0..self.num_lights {
            let light = &self.lights[i];
            if light.active {
                let level = light.calculate_light_level(x, y);
                min_level = min_level.min(level);
            }
        }

        // 检查视野光源
        for light in &self.vision_lights {
            if light.active {
                let level = light.calculate_light_level(x, y);
                min_level = min_level.min(level);
            }
        }

        min_level
    }

    /// 清除所有光源
    pub fn clear(&mut self) {
        for light in &mut self.lights {
            light.active = false;
        }
        self.num_lights = 0;
        for light in &mut self.vision_lights {
            light.active = false;
        }
    }
}

/// 光照映射图
///
/// 存储每个像素的光照级别，用于每像素光照
#[derive(Clone)]
pub struct Lightmap {
    /// 光照级别数据
    pub data: Vec<u8>,
    /// 宽度
    pub width: usize,
    /// 高度
    pub height: usize,
    /// 行距
    pub pitch: usize,
}

impl Lightmap {
    /// 创建新的光照映射图
    pub fn new(width: usize, height: usize) -> Self {
        let pitch = width;
        Self {
            data: vec![0; pitch * height],
            width,
            height,
            pitch,
        }
    }

    /// 获取指定位置的光照级别
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x >= self.width || y >= self.height {
            return LIGHTS_MAX;
        }
        self.data[y * self.pitch + x]
    }

    /// 设置指定位置的光照级别
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, level: u8) {
        if x < self.width && y < self.height {
            self.data[y * self.pitch + x] = level;
        }
    }

    /// 用指定值填充整个映射图
    pub fn fill(&mut self, level: u8) {
        self.data.fill(level);
    }

    /// 应用光照到颜色
    #[inline]
    pub fn adjust_color(&self, tables: &AllLightingTables, x: usize, y: usize, color: u8) -> u8 {
        let level = self.get(x, y) as usize;
        tables[level.min(LIGHTS_MAX as usize)][color as usize]
    }
}

/// 构建地块光照映射图
pub fn build_tile_lightmap(
    manager: &LightManager,
    tile_x: i32,
    tile_y: i32,
    tile_width: usize,
    tile_height: usize,
) -> Lightmap {
    let mut map = Lightmap::new(tile_width, tile_height);

    for py in 0..tile_height {
        for px in 0..tile_width {
            // 简单的地块内光照 (可以用更精细的方法)
            let level = manager.calculate_combined_light(tile_x, tile_y);
            map.set(px, py, level);
        }
    }

    map
}

/// 生成光照衰减表
///
/// 从调色板生成 16 级光照表
pub fn generate_light_tables(palette: &[[u8; 3]; 256]) -> AllLightingTables {
    let mut tables = [[0u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS];

    for level in 0..NUM_LIGHTING_LEVELS {
        let brightness = 256 - (level * 256 / NUM_LIGHTING_LEVELS);

        for i in 0..LIGHT_TABLE_SIZE {
            if brightness >= 256 {
                // 全亮
                tables[level][i] = i as u8;
            } else {
                // 需要找到暗化后最接近的颜色
                let [r, g, b] = palette[i];
                let dark_r = ((r as usize * brightness) >> 8) as u8;
                let dark_g = ((g as usize * brightness) >> 8) as u8;
                let dark_b = ((b as usize * brightness) >> 8) as u8;

                // 找到最接近的颜色
                tables[level][i] = find_nearest_color(palette, dark_r, dark_g, dark_b);
            }
        }
    }

    tables
}

/// 在调色板中找到最接近的颜色
fn find_nearest_color(palette: &[[u8; 3]; 256], r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;

    for (i, &[pr, pg, pb]) in palette.iter().enumerate() {
        let dr = (pr as i32 - r as i32).abs() as u32;
        let dg = (pg as i32 - g as i32).abs() as u32;
        let db = (pb as i32 - b as i32).abs() as u32;

        let distance = dr * dr + dg * dg + db * db;

        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_creation() {
        let light = Light::new(50, 50, 10);
        assert!(light.active);
        assert_eq!(light.x, 50);
        assert_eq!(light.y, 50);
        assert_eq!(light.radius, 10);
    }

    #[test]
    fn test_light_level_calculation() {
        let light = Light::new(50, 50, 10);

        // 中心点应该是最亮的
        let center_level = light.calculate_light_level(50, 50);
        assert_eq!(center_level, 0);

        // 边缘应该是最暗的
        let edge_level = light.calculate_light_level(60, 50);
        assert_eq!(edge_level, LIGHTS_MAX);

        // 中间距离
        let mid_level = light.calculate_light_level(55, 50);
        assert!(mid_level > 0 && mid_level < LIGHTS_MAX);
    }

    #[test]
    fn test_light_manager() {
        let mut manager = LightManager::new();

        let id = manager.add_light(50, 50, 10).unwrap();
        assert_eq!(id, 0);
        assert_eq!(manager.num_lights, 1);

        // 计算光照
        let level_center = manager.calculate_combined_light(50, 50);
        assert_eq!(level_center, 0);

        let level_far = manager.calculate_combined_light(100, 100);
        assert_eq!(level_far, manager.ambient_light);
    }

    #[test]
    fn test_lightmap() {
        let mut map = Lightmap::new(64, 32);
        assert_eq!(map.width, 64);
        assert_eq!(map.height, 32);

        map.set(10, 10, 5);
        assert_eq!(map.get(10, 10), 5);

        map.fill(8);
        assert_eq!(map.get(0, 0), 8);
        assert_eq!(map.get(63, 31), 8);
    }

    #[test]
    fn test_light_apply() {
        let manager = LightManager::new();

        // 默认表应该是恒等映射
        for i in 0..256 {
            assert_eq!(manager.apply_light(0, i as u8), i as u8);
        }
    }
}
