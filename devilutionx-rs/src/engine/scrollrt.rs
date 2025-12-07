//! 滚动渲染工具 - 移植自 Source/engine/render/scrollrt.h
//!
//! 提供地下城渲染、视口计算和屏幕滚动功能

use super::animation::AnimationInfo;
use super::types::{Direction, Displacement, Point, Size};

/// 瓦片宽度 (像素)
pub const TILE_WIDTH: i32 = 64;

/// 瓦片高度 (像素)
pub const TILE_HEIGHT: i32 = 32;

/// 地下城帧宽度
pub const DUN_FRAME_WIDTH: i32 = 64;

/// 地下城帧高度
pub const DUN_FRAME_HEIGHT: i32 = 32;

/// 视口几何信息
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportGeometry {
    /// 视口左上角在屏幕上的位置
    pub offset: Point,
    /// 视口尺寸
    pub size: Size,
    /// 每行瓦片数
    pub tiles_per_row: i32,
    /// 行数
    pub rows: i32,
    /// 瓦片偏移 (用于居中)
    pub tile_offset: Displacement,
}

/// 计算行走动画偏移
///
/// # Arguments
/// * `anim_info` - 动画信息
/// * `dir` - 行走方向
/// * `camera_mode` - 是否为相机模式（调整相对于相机的偏移）
///
/// # Returns
/// 屏幕坐标系中的偏移量
pub fn get_offset_for_walking(
    anim_info: &AnimationInfo,
    dir: Direction,
    camera_mode: bool,
) -> Displacement {
    // 获取动画进度 (0-128)
    let progress = anim_info.get_animation_progress();

    // 根据方向获取基础偏移
    let dir_offset = dir.to_displacement();

    // 计算世界坐标偏移
    let world_offset = Displacement::new(
        dir_offset.delta_x * progress as i32 / 128,
        dir_offset.delta_y * progress as i32 / 128,
    );

    // 转换为屏幕坐标
    let screen_offset = world_offset.world_to_screen();

    if camera_mode {
        // 相机模式：反向偏移
        -screen_offset
    } else {
        screen_offset
    }
}

/// 移动网格
///
/// # Arguments
/// * `offset` - 当前偏移（会被修改）
/// * `horizontal` - 水平移动方向 (-1, 0, 1)
/// * `vertical` - 垂直移动方向 (-1, 0, 1)
pub fn shift_grid(offset: &mut Point, horizontal: i32, vertical: i32) {
    // 移动一个瓦片
    offset.x += horizontal * TILE_WIDTH;
    offset.y += vertical * TILE_HEIGHT;
}

/// 计算被主面板遮挡的行数
pub fn rows_covered_by_panel(panel_height: i32) -> i32 {
    // 每行高度为 TILE_HEIGHT / 2 (等距投影)
    (panel_height + TILE_HEIGHT / 2 - 1) / (TILE_HEIGHT / 2)
}

/// 计算瓦片偏移（用于居中）
///
/// # Arguments
/// * `viewport_width` - 视口宽度
/// * `viewport_height` - 视口高度
///
/// # Returns
/// (offset_x, offset_y) 像素偏移
pub fn calc_tile_offset(viewport_width: i32, viewport_height: i32) -> (i32, i32) {
    // 计算居中偏移
    let offset_x = (viewport_width % TILE_WIDTH) / 2;
    let offset_y = (viewport_height % TILE_HEIGHT) / 2;
    (offset_x, offset_y)
}

/// 计算视口中需要的瓦片数量
///
/// # Arguments
/// * `viewport_width` - 视口宽度
/// * `viewport_height` - 视口高度
///
/// # Returns
/// (columns, rows) 每行瓦片数和行数
pub fn tiles_in_view(viewport_width: i32, viewport_height: i32) -> (i32, i32) {
    // 计算需要的列数（每列宽度为 TILE_WIDTH/2）
    let columns = (viewport_width + TILE_WIDTH - 1) / (TILE_WIDTH / 2) + 2;

    // 计算需要的行数（每行高度为 TILE_HEIGHT）
    let rows = (viewport_height + TILE_HEIGHT - 1) / TILE_HEIGHT + 2;

    (columns, rows)
}

/// 计算视口几何信息
pub fn calc_viewport_geometry(
    screen_width: i32,
    screen_height: i32,
    panel_height: i32,
) -> ViewportGeometry {
    let viewport_height = screen_height - panel_height;

    let (offset_x, offset_y) = calc_tile_offset(screen_width, viewport_height);
    let (columns, rows) = tiles_in_view(screen_width, viewport_height);

    ViewportGeometry {
        offset: Point::new(offset_x, offset_y),
        size: Size::new(screen_width, viewport_height),
        tiles_per_row: columns,
        rows,
        tile_offset: Displacement::new(offset_x, offset_y),
    }
}

/// 获取瓦片在屏幕上的位置
///
/// # Arguments
/// * `tile` - 地下城瓦片坐标
/// * `camera_tile` - 相机中心瓦片坐标
/// * `viewport` - 视口几何信息
///
/// # Returns
/// 屏幕坐标
pub fn get_screen_position(tile: Point, camera_tile: Point, viewport: &ViewportGeometry) -> Point {
    // 计算相对于相机的瓦片偏移
    let tile_offset = tile - camera_tile;

    // 转换为屏幕坐标 (tile_offset 已经是 Displacement 类型)
    let screen_offset = tile_offset.world_to_screen();

    // 加上视口中心偏移
    let center_x = viewport.size.width / 2;
    let center_y = viewport.size.height / 2;

    Point::new(
        center_x + screen_offset.delta_x,
        center_y + screen_offset.delta_y,
    )
}

/// 屏幕坐标转世界瓦片坐标
///
/// # Arguments
/// * `screen_pos` - 屏幕坐标
/// * `camera_tile` - 相机中心瓦片坐标
/// * `viewport` - 视口几何信息
///
/// # Returns
/// 世界瓦片坐标
pub fn screen_to_tile(screen_pos: Point, camera_tile: Point, viewport: &ViewportGeometry) -> Point {
    // 计算相对于视口中心的屏幕偏移
    let center_x = viewport.size.width / 2;
    let center_y = viewport.size.height / 2;

    let screen_offset = Displacement::new(screen_pos.x - center_x, screen_pos.y - center_y);

    // 转换为世界坐标偏移
    let world_offset = screen_offset.screen_to_world();

    // 加上相机位置
    camera_tile + world_offset
}

/// 检查瓦片是否在视口内
pub fn tile_in_viewport(
    tile: Point,
    camera_tile: Point,
    viewport: &ViewportGeometry,
    margin: i32,
) -> bool {
    let screen_pos = get_screen_position(tile, camera_tile, viewport);

    screen_pos.x >= -margin
        && screen_pos.x < viewport.size.width + margin
        && screen_pos.y >= -margin
        && screen_pos.y < viewport.size.height + margin
}

/// 滚动视图状态
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollState {
    /// 相机世界瓦片位置
    pub camera_tile: Point,
    /// 相机子瓦片偏移 (像素)
    pub camera_offset: Displacement,
    /// 是否正在滚动
    pub is_scrolling: bool,
    /// 滚动目标
    pub scroll_target: Option<Point>,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            camera_tile: Point::new(0, 0),
            camera_offset: Displacement::new(0, 0),
            is_scrolling: false,
            scroll_target: None,
        }
    }

    /// 设置相机位置
    pub fn set_camera(&mut self, tile: Point) {
        self.camera_tile = tile;
        self.camera_offset = Displacement::new(0, 0);
    }

    /// 平滑滚动到目标
    pub fn scroll_to(&mut self, target: Point) {
        self.scroll_target = Some(target);
        self.is_scrolling = true;
    }

    /// 更新滚动状态
    pub fn update(&mut self, delta_time: f32) {
        if !self.is_scrolling {
            return;
        }

        if let Some(target) = self.scroll_target {
            let diff = target - self.camera_tile;
            let distance = ((diff.delta_x * diff.delta_x + diff.delta_y * diff.delta_y) as f32).sqrt();

            if distance < 0.5 {
                self.camera_tile = target;
                self.camera_offset = Displacement::new(0, 0);
                self.is_scrolling = false;
                self.scroll_target = None;
            } else {
                // 平滑插值
                let speed = 10.0 * delta_time;
                let ratio = (speed / distance).min(1.0);

                self.camera_offset.delta_x += (diff.delta_x as f32 * ratio * 32.0) as i32;
                self.camera_offset.delta_y += (diff.delta_y as f32 * ratio * 16.0) as i32;

                // 如果偏移超过一个瓦片，更新瓦片位置
                while self.camera_offset.delta_x >= TILE_WIDTH / 2 {
                    self.camera_offset.delta_x -= TILE_WIDTH / 2;
                    self.camera_tile.x += 1;
                }
                while self.camera_offset.delta_x <= -TILE_WIDTH / 2 {
                    self.camera_offset.delta_x += TILE_WIDTH / 2;
                    self.camera_tile.x -= 1;
                }
                while self.camera_offset.delta_y >= TILE_HEIGHT / 2 {
                    self.camera_offset.delta_y -= TILE_HEIGHT / 2;
                    self.camera_tile.y += 1;
                }
                while self.camera_offset.delta_y <= -TILE_HEIGHT / 2 {
                    self.camera_offset.delta_y += TILE_HEIGHT / 2;
                    self.camera_tile.y -= 1;
                }
            }
        }
    }

    /// 取消滚动
    pub fn cancel_scroll(&mut self) {
        self.is_scrolling = false;
        self.scroll_target = None;
    }
}

/// 渲染顺序计算器
pub struct RenderOrderCalculator {
    viewport: ViewportGeometry,
}

impl RenderOrderCalculator {
    pub fn new(viewport: ViewportGeometry) -> Self {
        Self { viewport }
    }

    /// 生成渲染顺序的瓦片迭代器
    ///
    /// 等距投影中，需要从后往前（从左上到右下）渲染以正确处理遮挡
    pub fn iter_tiles(&self, camera_tile: Point) -> impl Iterator<Item = Point> + '_ {
        let half_cols = self.viewport.tiles_per_row / 2;
        let half_rows = self.viewport.rows / 2;

        (-half_rows..=half_rows).flat_map(move |row| {
            (-half_cols..=half_cols).map(move |col| {
                // 等距坐标转世界坐标
                Point::new(camera_tile.x + col + row, camera_tile.y - col + row)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_tile_offset() {
        let (ox, oy) = calc_tile_offset(640, 480);
        assert!(ox >= 0 && ox < TILE_WIDTH);
        assert!(oy >= 0 && oy < TILE_HEIGHT);
    }

    #[test]
    fn test_tiles_in_view() {
        let (cols, rows) = tiles_in_view(640, 480);
        assert!(cols > 0);
        assert!(rows > 0);
    }

    #[test]
    fn test_viewport_geometry() {
        let viewport = calc_viewport_geometry(640, 480, 128);
        assert_eq!(viewport.size.width, 640);
        assert_eq!(viewport.size.height, 352); // 480 - 128
        assert!(viewport.tiles_per_row > 0);
        assert!(viewport.rows > 0);
    }

    #[test]
    fn test_screen_position() {
        let viewport = calc_viewport_geometry(640, 480, 0);
        let camera = Point::new(50, 50);
        let tile = Point::new(50, 50);

        let screen_pos = get_screen_position(tile, camera, &viewport);

        // 同一位置应该在视口中心
        assert_eq!(screen_pos.x, 320);
        assert_eq!(screen_pos.y, 240);
    }

    #[test]
    fn test_scroll_state() {
        let mut state = ScrollState::new();
        state.set_camera(Point::new(50, 50));

        assert_eq!(state.camera_tile.x, 50);
        assert_eq!(state.camera_tile.y, 50);
        assert!(!state.is_scrolling);
    }

    #[test]
    fn test_shift_grid() {
        let mut offset = Point::new(0, 0);
        shift_grid(&mut offset, 1, 1);

        assert_eq!(offset.x, TILE_WIDTH);
        assert_eq!(offset.y, TILE_HEIGHT);
    }
}
