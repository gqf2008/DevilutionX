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

/// Light-grid tile coordinate used by `do_lighting` / `do_vision`.
///
/// Kept as a thin local type to avoid pulling the cursor module's `Point`
/// into the engine layer (the engine must not depend on the game layer).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

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

    /// Apply `DoLighting` to a pre-allocated light grid.
    ///
    /// Port of `DoLighting(position, radius, offset)` from `lighting.cpp`.
    /// Writes the minimum of the existing grid value and the falloff value
    /// into every tile within the radius (a 31x31 diamond-bounded square).
    pub fn do_lighting(
        &self,
        grid: &mut [u8],
        grid_width: usize,
        position: Point,
        radius: u8,
    ) {
        let radius = (radius as usize).min(NUM_LIGHT_RADIUS - 1);

        // Source tile is fully lit (value 0).
        if let Some(v) = grid_get_mut(grid, grid_width, position.x, position.y) {
            *v = (*v).min(0);
        }

        // Scan the surrounding tiles (a square of side 2*15+1) and apply the
        // precomputed falloff based on the linear (rotated-diamond) distance.
        for dy in -15i32..=15 {
            for dx in -15i32..=15 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let tx = position.x + dx;
                let ty = position.y + dy;
                if tx < 0 || ty < 0 {
                    continue;
                }
                let dist = light_cone_distance(dx, dy);
                if dist >= 128 {
                    continue;
                }
                let falloff = light_falloff(radius, dist);
                if let Some(v) = grid_get_mut(grid, grid_width, tx, ty) {
                    if falloff < *v {
                        *v = falloff;
                    }
                }
            }
        }
    }

    /// Apply `DoVision` to a visibility flag grid.
    ///
    /// Port of `DoVision(position, radius, doAutomap, visible)` from
    /// `lighting.cpp`. Marks every tile inside the radius as visible/lit and
    /// explored by setting the corresponding bits in the supplied flag grid.
    pub fn do_vision(
        &self,
        flags: &mut [u8],
        grid_width: usize,
        position: Point,
        radius: u8,
        visible: bool,
    ) {
        let r = radius.max(1) as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let tx = position.x + dx;
                let ty = position.y + dy;
                if tx < 0 || ty < 0 {
                    continue;
                }
                let dist = floor_sqrt(dx * dx + dy * dy);
                if dist > r {
                    continue;
                }
                if let Some(f) = grid_get_mut(flags, grid_width, tx, ty) {
                    // Visible + Explored bits (mirrors DungeonFlag::Visible|Lit|Explored).
                    *f |= VISION_FLAG_VISIBLE | VISION_FLAG_EXPLORED;
                    if visible {
                        *f |= VISION_FLAG_LIT;
                    }
                }
            }
        }
    }

    /// Regenerate the procedural light tables (MakeLightTable port).
    ///
    /// Mirrors the generation in `MakeLightTable()` from `lighting.cpp`: 16
    /// shade tables where each successive level darkens the palette by
    /// folding entries towards index 0. Level 15 (the last table) is left
    /// entirely black, which matches the C++ behaviour
    /// (`LightTables[15] = {};`).
    pub fn make_light_table(&mut self) {
        let steps_table: [u8; 18] =
            [16, 16, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16];

        for shade in 0..NUM_LIGHTING_LEVELS {
            let table = &mut self.tables[shade];
            let mut color_index: usize = 0;
            for &steps in &steps_table {
                let shading = shade * steps as usize / 16;
                let shade_start = color_index;
                let shade_end = shade_start + steps as usize - 1;
                for step in 0..steps as usize {
                    if color_index == 0 {
                        // Black stays black.
                        table[0] = 0;
                        color_index = 1;
                        continue;
                    }
                    let mut color = shade_start + step + shading;
                    if color > shade_end || color_index == 255 {
                        color = 0;
                    }
                    table[color_index] = color as u8;
                    color_index += 1;
                }
            }
        }

        // Last table is pitch black.
        self.tables[LIGHTS_MAX as usize].fill(0);
    }
}

/// Vision flag bits written by `do_vision` (mirrors `DungeonFlag`).
pub const VISION_FLAG_VISIBLE: u8 = 1 << 0;
pub const VISION_FLAG_LIT: u8 = 1 << 1;
pub const VISION_FLAG_EXPLORED: u8 = 1 << 2;

/// Number of supported light radii (matches C++ `NumLightRadiuses`).
const NUM_LIGHT_RADIUS: usize = 16;

/// Integer square root helper (matches `std::sqrt` truncation behaviour).
///
/// Implemented as a free function rather than a trait method because the
/// standard library's inherent `i32::isqrt` (stabilised in Rust 1.84) takes
/// precedence over trait methods at call sites and panics on negatives.
fn floor_sqrt(value: i32) -> i32 {
    if value <= 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}

/// Index a `[width * h]` grid with signed coordinates, returning a mutable ref.
fn grid_get_mut(grid: &mut [u8], width: usize, x: i32, y: i32) -> Option<&mut u8> {
    if x < 0 || y < 0 {
        return None;
    }
    let idx = y as usize * width + x as usize;
    grid.get_mut(idx)
}

/// Linear cone distance for the light falloff lookup.
///
/// Port of the `LightConeInterpolations` table build in `MakeLightTable`:
/// `sqrt((8*x - offsetX)^2 + (8*y - offsetY)^2)` with no sub-tile offset.
fn light_cone_distance(dx: i32, dy: i32) -> usize {
    let a = 8 * dx;
    let b = 8 * dy;
    ((a * a + b * b) as f32).sqrt() as usize
}

/// Light falloff value for a given radius and distance.
///
/// Port of the linear-falloff branch of `MakeLightTable`:
/// `factor = distance / maxDistance; scaled = factor * 15 + 0.5`.
fn light_falloff(radius: usize, distance: usize) -> u8 {
    let max_distance = (radius + 1) * 8;
    if distance > max_distance {
        return LIGHTS_MAX;
    }
    let factor = distance as f32 / max_distance as f32;
    let scaled = factor * (LIGHTS_MAX as f32) + 0.5;
    scaled.clamp(0.0, LIGHTS_MAX as f32) as u8
}

/// Player light offset used to centre the player's vision/light on their tile.
///
/// Port of the conceptual `CalcPlrLightOffset` helper: returns the tile-space
/// displacement that should be subtracted from the player's position before
/// running `DoLighting`/`DoVision`, so the light cone stays aligned with the
/// rendered sprite when the player is between tiles.
///
/// The original C++ folds this offset into `DoLighting` via the
/// `offset.deltaX/deltaY < 0` adjustments; for the headless Rust port we expose
/// it as a standalone signed displacement.
pub fn calc_plr_light_offset(walking: bool, facing_x: i32, facing_y: i32) -> (i32, i32) {
    if !walking {
        return (0, 0);
    }
    // Half-tile bias in the direction the player is facing.
    (facing_x.signum(), facing_y.signum())
}

/// Update a single player's vision + light sources for this frame.
///
/// Conceptual port of the per-player light bookkeeping scattered across
/// `ProcessLightList`/`ProcessVisionList` in `lighting.cpp`. Returns the
/// index of the player's vision light if one was activated.
pub fn light_player(
    manager: &mut LightManager,
    vision_slot: usize,
    player_x: i32,
    player_y: i32,
    light_radius: u8,
) -> Option<usize> {
    if vision_slot >= MAX_VISION {
        return None;
    }
    manager.vision_lights[vision_slot] = Light::new(player_x, player_y, light_radius as i32);
    manager.vision_lights[vision_slot].player_controlled = true;
    manager.vision_lights[vision_slot].id = vision_slot as i32;
    Some(vision_slot)
}

/// Attach a follow-light to the cursor position (spell targeting cursor etc.).
///
/// There is no direct C++ equivalent (the original game uses a dedicated
/// cursor light in `player.cpp`); this helper exposes the same idea for the
/// headless port: register a small radius light at the cursor tile.
pub fn light_cursor(manager: &mut LightManager, cursor_x: i32, cursor_y: i32) -> Option<usize> {
    manager.add_light(cursor_x, cursor_y, 3)
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

    #[test]
    fn test_do_lighting_centres_brightness() {
        let manager = LightManager::new();
        let w = 32usize;
        let mut grid = vec![LIGHTS_MAX; w * w];
        let pos = Point::new(16, 16);

        manager.do_lighting(&mut grid, w, pos, 8);

        // Source tile becomes fully lit.
        assert_eq!(grid[16 * w + 16], 0);
        // A tile well inside the radius is brighter than the ambient floor.
        assert!(grid[16 * w + 18] < LIGHTS_MAX);
        // A tile outside the radius keeps the ambient value.
        assert_eq!(grid[1 * w + 1], LIGHTS_MAX);
    }

    #[test]
    fn test_do_lighting_never_increases_brightness() {
        let manager = LightManager::new();
        let w = 16usize;
        // Pre-light some tiles brighter than the falloff could ever reach.
        let mut grid = vec![3u8; w * w];
        manager.do_lighting(&mut grid, w, Point::new(8, 8), 4);

        // Every touched tile must be <= its previous value (min semantics).
        for v in grid.iter() {
            assert!(*v <= 3);
        }
    }

    #[test]
    fn test_do_vision_marks_visible_and_explored() {
        let manager = LightManager::new();
        let w = 16usize;
        let mut flags = vec![0u8; w * w];

        manager.do_vision(&mut flags, w, Point::new(8, 8), 3, true);

        // Centre must have all three bits set.
        assert_eq!(
            flags[8 * w + 8] & (VISION_FLAG_VISIBLE | VISION_FLAG_LIT | VISION_FLAG_EXPLORED),
            VISION_FLAG_VISIBLE | VISION_FLAG_LIT | VISION_FLAG_EXPLORED
        );
        // A far tile is untouched.
        assert_eq!(flags[0], 0);
    }

    #[test]
    fn test_do_vision_hidden_when_visible_false() {
        let manager = LightManager::new();
        let w = 16usize;
        let mut flags = vec![0u8; w * w];

        manager.do_vision(&mut flags, w, Point::new(8, 8), 3, false);

        assert_eq!(flags[8 * w + 8] & VISION_FLAG_LIT, 0);
        assert_ne!(flags[8 * w + 8] & VISION_FLAG_VISIBLE, 0);
    }

    #[test]
    fn test_make_light_table_black_last_level() {
        let mut manager = LightManager::new();
        manager.make_light_table();

        // Last level must be all-black (matches `LightTables[15] = {};`).
        for &v in &manager.tables[LIGHTS_MAX as usize] {
            assert_eq!(v, 0);
        }
        // Level 0 keeps index 0 as black.
        assert_eq!(manager.tables[0][0], 0);
    }

    #[test]
    fn test_light_falloff_monotonic() {
        // Falloff must never decrease as distance grows.
        let mut prev = 0u8;
        for dist in 0..128 {
            let v = light_falloff(4, dist);
            assert!(v >= prev, "non-monotonic at dist={dist}: {v} < {prev}");
            prev = v;
        }
        assert_eq!(light_falloff(4, 127), LIGHTS_MAX);
    }

    #[test]
    fn test_light_cone_distance_origin() {
        assert_eq!(light_cone_distance(0, 0), 0);
        assert!(light_cone_distance(1, 0) > 0);
    }

    #[test]
    fn test_calc_plr_light_offset() {
        // Idle player -> no offset.
        assert_eq!(calc_plr_light_offset(false, 1, 0), (0, 0));
        // Walking east biases by sign of facing.
        assert_eq!(calc_plr_light_offset(true, 1, 0), (1, 0));
        assert_eq!(calc_plr_light_offset(true, -1, -1), (-1, -1));
    }

    #[test]
    fn test_light_player_activates_vision_slot() {
        let mut manager = LightManager::new();
        let slot = light_player(&mut manager, 0, 50, 50, 8).unwrap();
        assert_eq!(slot, 0);
        assert!(manager.vision_lights[0].active);
        assert!(manager.vision_lights[0].player_controlled);
        assert_eq!(manager.vision_lights[0].x, 50);
    }

    #[test]
    fn test_light_player_rejects_bad_slot() {
        let mut manager = LightManager::new();
        assert!(light_player(&mut manager, MAX_VISION, 0, 0, 8).is_none());
    }

    #[test]
    fn test_light_cursor_adds_light() {
        let mut manager = LightManager::new();
        let id = light_cursor(&mut manager, 10, 12).unwrap();
        assert_eq!(manager.lights[id].x, 10);
        assert_eq!(manager.lights[id].y, 12);
        assert!(manager.lights[id].active);
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(floor_sqrt(0), 0);
        assert_eq!(floor_sqrt(1), 1);
        assert_eq!(floor_sqrt(4), 2);
        assert_eq!(floor_sqrt(9), 3);
        assert_eq!(floor_sqrt(15), 3);
        assert_eq!(floor_sqrt(16), 4);
        assert_eq!(floor_sqrt(-5), 0);
    }
}
