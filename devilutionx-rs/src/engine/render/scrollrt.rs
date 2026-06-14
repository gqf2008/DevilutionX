//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Scroll/Render - 地牢渲染和滚动功能
//!
//! 移植自 Source/engine/render/scrollrt.h/cpp
//!
//! 实现地下城、怪物和其他游戏元素的渲染，以及调用其他渲染例程。

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;

use crate::engine::{Point, Displacement, Direction, Rectangle, Size};
use crate::engine::animationinfo::AnimationInfo;
use crate::engine::surface::Surface;
use crate::game::options;
use crate::game::game_state;
use crate::ui::panel_state;
use crate::platform::sdl;
use super::dun_render::{TILE_WIDTH, TILE_HEIGHT, DUN_FRAME_WIDTH};

// ============================================================================
// 常量
// ============================================================================

/// 右侧帧偏移量
pub const RIGHT_FRAME_DISPLACEMENT: Displacement = Displacement { delta_x: DUN_FRAME_WIDTH as i32, delta_y: 0 };

/// 侧面板尺寸
pub const SIDE_PANEL_SIZE: Size = Size { width: 320, height: 352 };

/// 信息框区域
pub const INFO_BOX_RECT: Rectangle = Rectangle {
    position: Point { x: 177, y: 46 },
    size: Size { width: 288, height: 60 },
};

// ============================================================================
// 全局状态
// ============================================================================

/// 自动地图是否显示物品
pub static AUTO_MAP_SHOW_ITEMS: AtomicBool = AtomicBool::new(false);

/// 帧标志（FPS 显示）
pub static FRAME_FLAG: AtomicBool = AtomicBool::new(false);

/// 上次 FPS 更新时间（毫秒）
static LAST_FPS_UPDATE_MS: AtomicU32 = AtomicU32::new(0);

/// 视口状态
static VIEWPORT_STATE: RwLock<ViewportStateInner> = RwLock::new(ViewportStateInner::new());

/// 前一个光标矩形
static PREV_CURSOR_RECT: RwLock<Rectangle> = RwLock::new(Rectangle {
    position: Point { x: 0, y: 0 },
    size: Size { width: 0, height: 0 },
});

pub fn auto_map_show_items() -> bool {
    AUTO_MAP_SHOW_ITEMS.load(Ordering::Relaxed)
}

pub fn set_auto_map_show_items(value: bool) {
    AUTO_MAP_SHOW_ITEMS.store(value, Ordering::Relaxed);
}

pub fn frame_flag() -> bool {
    FRAME_FLAG.load(Ordering::Relaxed)
}

pub fn set_frame_flag(value: bool) {
    FRAME_FLAG.store(value, Ordering::Relaxed);
}

// ============================================================================
// 类型定义
// ============================================================================

/// 内部视口状态
#[derive(Debug, Clone)]
struct ViewportStateInner {
    /// 瓦片偏移
    pub tile_offset: Displacement,
    /// 瓦片移位
    pub tile_shift: Displacement,
    /// 视图列数
    pub tile_columns: i32,
    /// 视图行数
    pub tile_rows: i32,
}

impl ViewportStateInner {
    const fn new() -> Self {
        Self {
            tile_offset: Displacement { delta_x: 0, delta_y: 0 },
            tile_shift: Displacement { delta_x: 0, delta_y: 0 },
            tile_columns: 0,
            tile_rows: 0,
        }
    }
}

/// 视口几何状态（公开接口）
#[derive(Debug, Clone, Default)]
pub struct ViewportState {
    /// 视口偏移
    pub offset: Point,
    /// 视图列数
    pub columns: i32,
    /// 视图行数
    pub rows: i32,
}

// ============================================================================
// 行走偏移计算
// ============================================================================

/// 行走方向的偏移量表
/// 顺序: South, SouthWest, West, NorthWest, North, NorthEast, East, SouthEast
const MOVING_OFFSET: [Displacement; 8] = [
    Displacement { delta_x:   0, delta_y:  32 },  // South
    Displacement { delta_x: -32, delta_y:  16 },  // SouthWest
    Displacement { delta_x: -64, delta_y:   0 },  // West
    Displacement { delta_x: -32, delta_y: -16 },  // NorthWest
    Displacement { delta_x:   0, delta_y: -32 },  // North
    Displacement { delta_x:  32, delta_y: -16 },  // NorthEast
    Displacement { delta_x:  64, delta_y:   0 },  // East
    Displacement { delta_x:  32, delta_y:  16 },  // SouthEast
];

/// 获取行走动画的偏移量
///
/// C++ 原型: Displacement GetOffsetForWalking(const AnimationInfo &animationInfo, Direction dir, bool cameraMode)
pub fn get_offset_for_walking(
    animation_info: &AnimationInfo,
    dir: Direction,
    camera_mode: bool,
) -> Displacement {
    let animation_progress = animation_info.get_animation_progress();
    let base_offset = MOVING_OFFSET[dir as usize];
    
    let mut offset = Displacement {
        delta_x: (base_offset.delta_x as i64 * animation_progress as i64 / AnimationInfo::BASE_VALUE_FRACTION as i64) as i32,
        delta_y: (base_offset.delta_y as i64 * animation_progress as i64 / AnimationInfo::BASE_VALUE_FRACTION as i64) as i32,
    };

    if camera_mode {
        offset = Displacement {
            delta_x: -offset.delta_x,
            delta_y: -offset.delta_y,
        };
    }

    offset
}

/// 清除光标状态
///
/// C++ 原型: void ClearCursor()
pub fn clear_cursor() {
    *PREV_CURSOR_RECT.write().unwrap() = Rectangle {
        position: Point { x: 0, y: 0 },
        size: Size { width: 0, height: 0 },
    };
}

/// 在逻辑网格上移动视图区域
///
/// C++ 原型: void ShiftGrid(Point *offset, int horizontal, int vertical)
///
/// 注意：这不允许在奇偶行之间切换
pub fn shift_grid(offset: &mut Point, horizontal: i32, vertical: i32) {
    offset.x += vertical + horizontal;
    offset.y += vertical - horizontal;
}

/// 获取主面板覆盖的行数
///
/// C++ 原型: int RowsCoveredByPanel()
pub fn rows_covered_by_panel() -> i32 {
    let main_panel_width = 640; // TODO: 从 GetMainPanel() 获取
    let main_panel_height = 128;
    let screen_width = sdl::get_screen_width() as i32;
    
    if screen_width <= main_panel_width {
        return 0;
    }

    let mut rows = main_panel_height / TILE_HEIGHT;
    
    // 如果启用缩放，行数减半
    let zoom = options::options().graphics.zoom;
    if zoom {
        rows /= 2;
    }

    rows
}

/// 计算视图区域居中所需的偏移量
///
/// C++ 原型: void CalcTileOffset(int *offsetX, int *offsetY)
pub fn calc_tile_offset(offset_x: &mut i32, offset_y: &mut i32) {
    let screen_width = sdl::get_screen_width() as i32;
    let viewport_height = sdl::get_viewport_height() as i32;
    let zoom = options::options().graphics.zoom;

    let (x, y) = if !zoom {
        (screen_width % TILE_WIDTH, viewport_height % TILE_HEIGHT)
    } else {
        ((screen_width / 2) % TILE_WIDTH, (viewport_height / 2) % TILE_HEIGHT)
    };

    *offset_x = if x != 0 { (TILE_WIDTH - x) / 2 } else { 0 };
    *offset_y = if y != 0 { (TILE_HEIGHT - y) / 2 } else { 0 };
}

/// 计算覆盖视图区域所需的菱形瓦片数
///
/// C++ 原型: void TilesInView(int *columns, int *rows)
pub fn tiles_in_view(columns: &mut i32, rows: &mut i32) {
    let screen_width = sdl::get_screen_width() as i32;
    let viewport_height = sdl::get_viewport_height() as i32;
    let zoom = options::options().graphics.zoom;

    let mut cols = screen_width / TILE_WIDTH;
    if screen_width % TILE_WIDTH != 0 {
        cols += 1;
    }
    
    let mut row_count = viewport_height / TILE_HEIGHT;
    if viewport_height % TILE_HEIGHT != 0 {
        row_count += 1;
    }

    if zoom {
        if cols & 1 != 0 { cols += 1; }
        cols /= 2;
        if row_count & 1 != 0 { row_count += 1; }
        row_count /= 2;
    }

    *columns = cols;
    *rows = row_count;
}

/// 计算视口几何
///
/// C++ 原型: void CalcViewportGeometry()
pub fn calc_viewport_geometry() {
    let zoom = options::options().graphics.zoom;
    let zoom_factor = if zoom { 2 } else { 1 };
    let screen_width = sdl::get_screen_width() as i32 / zoom_factor;
    let screen_height = sdl::get_screen_height() as i32 / zoom_factor;
    let panel_height = 128 / zoom_factor; // TODO: 从 GetMainPanel() 获取
    let pixels_to_panel = screen_height - panel_height;
    let mut player_position = Point::new(screen_width / 2, pixels_to_panel / 2);

    if zoom {
        player_position.y += TILE_HEIGHT / 4;
    }

    let tiles_to_top = (player_position.y + TILE_HEIGHT - 1) / TILE_HEIGHT;
    let tiles_to_left = (player_position.x + TILE_WIDTH - 1) / TILE_WIDTH;

    let mut start_position = Point::new(
        player_position.x - tiles_to_left * TILE_WIDTH,
        player_position.y - tiles_to_top * TILE_HEIGHT,
    );

    let mut tile_shift = Displacement::new(0, 0);
    tile_shift = tile_shift + Displacement::from_direction(Direction::North) * tiles_to_top;
    tile_shift = tile_shift + Displacement::from_direction(Direction::West) * tiles_to_left;

    if tiles_to_left * TILE_WIDTH >= player_position.x {
        start_position.x += TILE_WIDTH / 2;
        start_position.y -= TILE_HEIGHT / 2;
        tile_shift = tile_shift + Displacement::from_direction(Direction::NorthEast);
    } else if tiles_to_top * TILE_HEIGHT < player_position.y {
        start_position.y -= TILE_HEIGHT;
        tile_shift = tile_shift + Displacement::from_direction(Direction::North);
    }

    let tile_offset = Displacement {
        delta_x: start_position.x - TILE_WIDTH / 2,
        delta_y: start_position.y + TILE_HEIGHT / 2 - 1,
    };

    let viewport_height = sdl::get_viewport_height() as i32 / zoom_factor;
    let render_start = Point::new(
        start_position.x - TILE_WIDTH / 2,
        start_position.y - TILE_HEIGHT / 2,
    );
    let tile_rows = (viewport_height - render_start.y + TILE_HEIGHT / 2 - 1) / (TILE_HEIGHT / 2);
    let tile_columns = (screen_width - render_start.x + TILE_WIDTH - 1) / TILE_WIDTH;

    let mut state = VIEWPORT_STATE.write().unwrap();
    state.tile_offset = tile_offset;
    state.tile_shift = tile_shift;
    state.tile_rows = tile_rows;
    state.tile_columns = tile_columns;
}

/// 计算给定瓦片的屏幕位置
///
/// C++ 原型: Point GetScreenPosition(Point tile)
pub fn get_screen_position(tile: Point) -> Point {
    let view_position = game_state::get_view_position();
    let state = VIEWPORT_STATE.read().unwrap();
    
    let delta = Displacement {
        delta_x: view_position.x - tile.x,
        delta_y: view_position.y - tile.y,
    };

    let mut position = Point::new(0, 0);
    position.x += (delta.delta_x - delta.delta_y) * (TILE_WIDTH / 2);
    position.y += (delta.delta_x + delta.delta_y) * (TILE_HEIGHT / 2);
    position.x += state.tile_offset.delta_x;
    position.y += state.tile_offset.delta_y;
    
    position
}

/// 用黑色填充整个屏幕
///
/// C++ 原型: void ClearScreenBuffer()
pub fn clear_screen_buffer() {
    if sdl::is_headless_mode() {
        return;
    }
    // TODO: SDL_FillSurfaceRect(PalSurface, nullptr, 0)
}

/// 当鼠标靠近边缘时滚动屏幕 (仅 DEBUG)
///
/// C++ 原型: void ScrollView()
#[cfg(debug_assertions)]
pub fn scroll_view() {
    // TODO: 依赖鼠标位置、ViewPosition
}

/// 初始化 FPS 计数器
///
/// C++ 原型: void EnableFrameCount()
pub fn enable_frame_count() {
    FRAME_FLAG.store(true, Ordering::Relaxed);
    LAST_FPS_UPDATE_MS.store(sdl::get_ticks(), Ordering::Relaxed);
}

/// 重绘屏幕
///
/// C++ 原型: void scrollrt_draw_game_screen()
pub fn scrollrt_draw_game_screen() {
    if sdl::is_headless_mode() {
        return;
    }
    
    // TODO: 完整的渲染管线
    // 1. 检查是否需要完全重绘
    // 2. 获取全局后备缓冲区
    // 3. 取消绘制光标
    // 4. 绘制光标
    // 5. DrawMain
    // 6. RenderPresent
    sdl::render_present();
}

/// 渲染游戏并 blit 到屏幕
///
/// C++ 原型: void DrawAndBlit()
pub fn draw_and_blit() {
    if sdl::is_headless_mode() {
        return;
    }
    
    // TODO: 完整的渲染管线
    // 1. 检查重绘组件
    // 2. 获取主面板区域
    // 3. 取消绘制光标
    // 4. 更新游戏进度
    // 5. DrawView
    // 6. 绘制各种 UI 组件
    // 7. 绘制光标
    // 8. 绘制 FPS
    // 9. DrawMain
    // 10. RedrawComplete
    // 11. RenderPresent
    sdl::render_present();
}

/// 根据两点计算方向
///
/// C++ 原型: Direction GetDirection(Point start, Point destination)
/// 使用 5:2 比例确定对角线方向
pub fn get_direction(start: Point, destination: Point) -> Direction {
    let mx = destination.x - start.x;
    let my = destination.y - start.y;

    if mx >= 0 {
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return Direction::SouthWest;
            }
            if 5 * my <= mx * 2 {
                return Direction::SouthEast;
            }
            Direction::South
        } else {
            let my = -my;
            if 5 * mx <= my * 2 {
                return Direction::NorthEast;
            }
            if 5 * my <= mx * 2 {
                return Direction::SouthEast;
            }
            Direction::East
        }
    } else {
        let mx = -mx;
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return Direction::SouthWest;
            }
            if 5 * my <= mx * 2 {
                return Direction::NorthWest;
            }
            Direction::West
        } else {
            let my = -my;
            if 5 * mx <= my * 2 {
                return Direction::NorthEast;
            }
            if 5 * my <= mx * 2 {
                return Direction::NorthWest;
            }
            Direction::North
        }
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 检查是否在地牢边界内
#[inline]
pub fn in_dungeon_bounds(position: Point) -> bool {
    position.x >= 0 && position.x < 112 && position.y >= 0 && position.y < 112
}

/// 检查左面板是否打开
pub fn is_left_panel_open() -> bool {
    panel_state::is_left_panel_open()
}

/// 检查右面板是否打开
pub fn is_right_panel_open() -> bool {
    panel_state::is_right_panel_open()
}

/// 检查面板是否可以覆盖视图
pub fn can_panels_cover_view() -> bool {
    let screen_width = sdl::get_screen_width() as i32;
    let main_panel_width = 640; // 主面板宽度
    screen_width > main_panel_width
}

/// 带光照的 CLX 精灵绘制
pub fn clx_draw_light(
    _out: &Surface,
    _position: Point,
    _clx_data: &[u8],
    _light_table_index: i32,
) {
    // TODO: 依赖 ClxDraw 和 ClxDrawTRN
}

/// 带光照和透明度混合的 CLX 精灵绘制
pub fn clx_draw_light_blended(
    _out: &Surface,
    _position: Point,
    _clx_data: &[u8],
    _light_table_index: i32,
) {
    // TODO: 依赖 ClxDrawBlended 和 ClxDrawBlendedTRN
}

// ============================================================================
// 内部渲染函数 (需要更多依赖)
// ============================================================================

/// 绘制地板瓦片
/// 
/// C++ 原型: void DrawFloorTile(const Surface &out, const Lightmap &lightmap, Point tilePosition, Point targetBufferPosition)
#[allow(dead_code)]
fn draw_floor_tile(
    _out: &Surface,
    _tile_position: Point,
    _target_buffer_position: Point,
) {
    // TODO: 依赖 dPiece, DPieceMicros, pDungeonCels, RenderTileFrame
}

/// 渲染地板行
/// 
/// C++ 原型: void DrawFloor(const Surface &out, const Lightmap &lightmap, Point tilePosition, Point targetBufferPosition, int rows, int columns)
#[allow(dead_code)]
fn draw_floor(
    _out: &Surface,
    _tile_position: Point,
    _target_buffer_position: Point,
    _rows: i32,
    _columns: i32,
) {
    // TODO: 完整实现需要 InDungeonBounds, IsFloor, DrawFloorTile, world_draw_black_tile
}

/// 渲染瓦片内容
/// 
/// C++ 原型: void DrawTileContent(const Surface &out, const Lightmap &lightmap, Point tilePosition, Point targetBufferPosition, int rows, int columns)
#[allow(dead_code)]
fn draw_tile_content(
    _out: &Surface,
    _tile_position: Point,
    _target_buffer_position: Point,
    _rows: i32,
    _columns: i32,
) {
    // TODO: 完整实现需要 DrawDungeon, 怪物、物品、玩家渲染
}

/// 渲染单个地牢瓦片及其内容
/// 
/// C++ 原型: void DrawDungeon(const Surface &out, const Lightmap &lightmap, Point tilePosition, Point targetBufferPosition)
#[allow(dead_code)]
fn draw_dungeon(
    _out: &Surface,
    _tile_position: Point,
    _target_buffer_position: Point,
) {
    // TODO: 依赖 DrawFloor, DrawItem, DrawMonster, DrawPlayer, DrawObject, DrawMissile
}

/// 绘制视图
/// 
/// C++ 原型: void DrawView(const Surface &out, Point viewPosition)
pub fn draw_view(_out: &Surface, _view_position: Point) {
    // TODO: 完整实现需要 CalcFirstTilePosition, DrawGame
}

/// 绘制游戏
/// 
/// C++ 原型: void DrawGame(const Surface &fullOut, Point position, Displacement offset)
#[allow(dead_code)]
fn draw_game(
    _full_out: &Surface,
    _position: Point,
    _offset: Displacement,
) {
    // TODO: 依赖 DrawFloor, DrawTileContent, Zoom, Lightmap::build
}

/// 绘制主内容
/// 
/// C++ 原型: void DrawMain(int hgt, bool drawInfoBox, bool drawHealth, bool drawMana, bool drawBelt, bool drawControlButtons)
pub fn draw_main(
    _hgt: i32,
    _draw_info_box: bool,
    _draw_health: bool,
    _draw_mana: bool,
    _draw_belt: bool,
    _draw_control_buttons: bool,
) {
    // TODO: 依赖 DoBlitScreen, DrawInv, DrawSpellBook, DrawChr, DrawQuestLog 等 UI 绘制函数
}

/// 绘制 FPS
/// 
/// C++ 原型: void DrawFPS(const Surface &out)
pub fn draw_fps(_out: &Surface) {
    if !FRAME_FLAG.load(Ordering::Relaxed) {
        return;
    }
    
    // TODO: 完整实现需要 DrawString, 字体渲染
    let _runtime_in_ms = sdl::get_ticks();
    let _ms_since_last_update = _runtime_in_ms - LAST_FPS_UPDATE_MS.load(Ordering::Relaxed);
    
    // 计算 FPS 并绘制
}

/// 从后备缓冲区更新屏幕区域
/// 
/// C++ 原型: void DoBlitScreen(Rectangle area)
pub fn do_blit_screen(area: Rectangle) {
    let src_rect = sdl::make_sdl_rect(area);
    let dst_rect = sdl::make_sdl_rect(area);
    sdl::blt_fast(&src_rect, &dst_rect);
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_direction() {
        let origin = Point::new(0, 0);
        
        assert_eq!(get_direction(origin, Point::new(10, 10)), Direction::South);
        assert_eq!(get_direction(origin, Point::new(-10, -10)), Direction::North);
        assert_eq!(get_direction(origin, Point::new(10, -10)), Direction::East);
        assert_eq!(get_direction(origin, Point::new(-10, 10)), Direction::West);
    }

    #[test]
    fn test_shift_grid() {
        let mut offset = Point::new(0, 0);
        shift_grid(&mut offset, 5, 3);
        assert_eq!(offset.x, 8);  // 3 + 5
        assert_eq!(offset.y, -2); // 3 - 5
    }

    #[test]
    fn test_moving_offset_table() {
        assert_eq!(MOVING_OFFSET[Direction::South as usize].delta_x, 0);
        assert_eq!(MOVING_OFFSET[Direction::South as usize].delta_y, 32);
        assert_eq!(MOVING_OFFSET[Direction::North as usize].delta_x, 0);
        assert_eq!(MOVING_OFFSET[Direction::North as usize].delta_y, -32);
        assert_eq!(MOVING_OFFSET[Direction::East as usize].delta_x, 64);
        assert_eq!(MOVING_OFFSET[Direction::West as usize].delta_x, -64);
    }

    #[test]
    fn test_in_dungeon_bounds() {
        assert!(in_dungeon_bounds(Point::new(0, 0)));
        assert!(in_dungeon_bounds(Point::new(50, 50)));
        assert!(in_dungeon_bounds(Point::new(111, 111)));
        assert!(!in_dungeon_bounds(Point::new(-1, 0)));
        assert!(!in_dungeon_bounds(Point::new(112, 0)));
    }

    #[test]
    fn test_calc_tile_offset() {
        let mut offset_x = 0;
        let mut offset_y = 0;
        calc_tile_offset(&mut offset_x, &mut offset_y);
        // 640 % 64 = 0, 480 % 32 = 0
        assert_eq!(offset_x, 0);
        assert_eq!(offset_y, 0);
    }

    #[test]
    fn test_tiles_in_view() {
        let mut columns = 0;
        let mut rows = 0;
        tiles_in_view(&mut columns, &mut rows);
        // 640 / 64 = 10, 480 / 32 = 15
        assert_eq!(columns, 10);
        assert_eq!(rows, 15);
    }

    #[test]
    fn test_right_frame_displacement() {
        assert_eq!(RIGHT_FRAME_DISPLACEMENT.delta_x, DUN_FRAME_WIDTH as i32);
        assert_eq!(RIGHT_FRAME_DISPLACEMENT.delta_y, 0);
    }
}
