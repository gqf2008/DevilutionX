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

// =============================================================================
// C++ 忠实绘制管线 — Source/engine/render/scrollrt.cpp
// -----------------------------------------------------------------------------
// 以下函数严格移植 C++ 的 DrawView → CalcFirstTilePosition + DrawGame →
// DrawFloor / DrawTileContent → DrawCell 链路，使用 dun_render 的逐瓦片渲染。
// 渲染目标是 8-bit 调色板索引 Surface（与 C++ PalSurface 一致）；本增量
// 光照为全亮（light_table = None → render_line_opaque 直拷），实体/光照/
// 透明/HUD 仍由调用方在 SDL canvas 上叠加（下一增量入面）。
// =============================================================================

use super::dun_render::{
    self, DUN_FRAME_WIDTH as HALF_TILE_WIDTH, TileType, MaskType,
};
use super::dungeon::{DungeonLevelData, MegaTile, TileProperties};
use super::lighting::{LIGHT_TABLE_SIZE, NUM_LIGHTING_LEVELS, LIGHTS_MAX};

/// 地下城网格尺寸（C++ MAXDUNX/MAXDUNY）。
const MAXDUN: i32 = 112;

/// 渲染光照上下文：逐 tile 光照级别网格（C++ `dLight`）+ 光照表（C++
/// `LightTables`）。`table_for` 由 tile 的光照级别选出对应的颜色重映射表。
/// 城镇传入全 0 网格 → 恒等表（全亮）；地牢传入含玩家光晕的网格。
pub struct Lighting<'a> {
    /// dLight 网格（MAXDUN×MAXDUN 扁平），值 0(全亮)..15(全黑)。
    dlight: &'a [u8],
    /// 光照表（C++ LightTables，由 LightManager::make_light_table 生成）。
    tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    /// Per-tile transparency value (C++ `dTransVal`), flat MAXDUN×MAXDUN.
    /// Empty = no transparency data (all opaque).
    trans_val: &'a [i8],
    /// C++ `TransList` — which TransVal regions are currently see-through.
    /// Filled per frame by `DoVision`; empty = nothing transparent.
    trans_list: &'a [bool],
}

impl<'a> Lighting<'a> {
    pub fn new(
        dlight: &'a [u8],
        tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    ) -> Self {
        Self {
            dlight,
            tables,
            trans_val: &[],
            trans_list: &[],
        }
    }

    /// Construct a lighting context that also carries C++ `dTransVal` /
    /// `TransList` for per-tile transparency rendering.
    pub fn with_transparency(
        dlight: &'a [u8],
        tables: &'a [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
        trans_val: &'a [i8],
        trans_list: &'a [bool],
    ) -> Self {
        Self {
            dlight,
            tables,
            trans_val,
            trans_list,
        }
    }

    /// C++ `TransList[dTransVal[x][y]]` (scrollrt.cpp:540). False when no
    /// transparency data is present or the value is out of range.
    fn transparency_for(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAXDUN || y >= MAXDUN {
            return false;
        }
        let Some(&v) = self
            .trans_val
            .get(y as usize * MAXDUN as usize + x as usize)
        else {
            return false;
        };
        if v <= 0 {
            return false;
        }
        self.trans_list.get(v as usize).copied().unwrap_or(false)
    }

    /// 查 tile 的光照表（越界按全亮处理）。
    fn table_for(&self, x: i32, y: i32) -> &'a [u8; LIGHT_TABLE_SIZE] {
        let level = if x >= 0 && y >= 0 && x < MAXDUN && y < MAXDUN {
            self.dlight
                .get(y as usize * MAXDUN as usize + x as usize)
                .copied()
                .unwrap_or(0) as usize
        } else {
            0
        };
        &self.tables[level.min(LIGHTS_MAX as usize)]
    }
}

/// C++ `RightFrameDisplacement` = { DunFrameWidth, 0 }（半个瓦片宽，32px）。
const RIGHT_FRAME_DISPLACEMENT: Displacement = Displacement { delta_x: HALF_TILE_WIDTH, delta_y: 0 };

/// `dPiece` 网格视图（C++ `dPiece[x][y]`）。城镇与地牢布局都暴露
/// `get(x, y) -> u16`，本 trait 让 draw_view 统一接收两者。
pub trait DPieceGrid {
    fn d_piece(&self, x: i32, y: i32) -> u16;

    /// Per-tile C++ `dTransVal` for transparency. `None` = no transparency
    /// data (all opaque), matching the default layouts.
    fn trans_val(&self, _x: i32, _y: i32) -> Option<i8> {
        None
    }
}

#[inline]
fn in_dungeon_bounds(p: Point) -> bool {
    p.x >= 0 && p.x < MAXDUN && p.y >= 0 && p.y < MAXDUN
}

/// 由 level-piece id（`dPiece` 值，1 基）解析其 mega-tile（C++
/// `DPieceMicros[levelPieceId]`）。按 `mega_tiles[piece - 1]` 索引，与活渲染器
/// 既有的映射一致。
fn mega_for_piece<'a>(level: &'a DungeonLevelData, piece_id: u16) -> Option<&'a MegaTile> {
    level.min.mega_tiles.get(piece_id.saturating_sub(1) as usize)
}

/// C++ `CalcViewportGeometry`（scrollrt.cpp:1573）的纯函数移植（zoom=false）。
/// 返回起始 tile/offset 与行列数。
#[derive(Clone, Copy, Debug)]
pub struct TileViewport {
    /// 屏幕偏移（C++ `tileOffset`）。
    pub tile_offset: Displacement,
    /// 起始 tile 相对玩家 tile 的位移（C++ `tileShift`）。
    pub tile_shift: Displacement,
    pub tile_rows: i32,
    pub tile_columns: i32,
}

pub fn viewport_geometry(screen_w: i32, screen_h: i32, viewport_h: i32) -> TileViewport {
    // viewport_h = screen_h - panel_height（C++ gnViewportHeight）；同时它也就是
    // 玩家所在可视区的高度（pixelsToPanel），用于把玩家定位到可视区垂直中心。
    let pixels_to_panel = viewport_h;
    let player_position = Point::new(screen_w / 2, pixels_to_panel / 2);

    let tiles_to_top = (player_position.y + TILE_HEIGHT - 1) / TILE_HEIGHT;
    let tiles_to_left = (player_position.x + TILE_WIDTH - 1) / TILE_WIDTH;

    // 渲染起始 tile 中心相对视口原点的屏幕位置。
    let mut start_position = player_position
        - Displacement::new(tiles_to_left * TILE_WIDTH, tiles_to_top * TILE_HEIGHT);

    // 起始 tile 在 tile 空间相对玩家 tile 的位移。
    let mut tile_shift = Displacement::new(0, 0);
    tile_shift = tile_shift + Direction::North.to_displacement() * tiles_to_top;
    tile_shift = tile_shift + Direction::West.to_displacement() * tiles_to_left;

    // 渲染循环期望从"列数较少"的行开始。
    if tiles_to_left * TILE_WIDTH >= player_position.x {
        start_position = start_position + Displacement::new(TILE_WIDTH / 2, -TILE_HEIGHT / 2);
        tile_shift = tile_shift + Direction::NorthEast.to_displacement();
    } else if tiles_to_top * TILE_HEIGHT < player_position.y {
        start_position = start_position + Displacement::new(0, -TILE_HEIGHT);
        tile_shift = tile_shift + Direction::North.to_displacement();
    }

    let tile_offset = Displacement::new(
        start_position.x - TILE_WIDTH / 2,
        start_position.y + TILE_HEIGHT / 2 - 1,
    );

    let render_start = start_position - Displacement::new(TILE_WIDTH / 2, TILE_HEIGHT / 2);
    let tile_rows = (viewport_h - render_start.y + TILE_HEIGHT / 2 - 1) / (TILE_HEIGHT / 2);
    let tile_columns = (screen_w - render_start.x + TILE_WIDTH - 1) / TILE_WIDTH;

    TileViewport { tile_offset, tile_shift, tile_rows, tile_columns }
}

/// C++ `DrawFloorTile`（scrollrt.cpp:652）。mt[0] 左三角 + mt[1] 右三角。
fn draw_floor_tile(
    out: &mut super::surface::Surface,
    level: &DungeonLevelData,
    piece_id: u16,
    tile_position: Point,
    target_buffer_position: Point,
    lighting: &Lighting,
) {
    let Some(mega) = mega_for_piece(level, piece_id) else {
        return;
    };
    let tbl = lighting.table_for(tile_position.x, tile_position.y);

    let block = mega.blocks[0];
    if block.has_value() {
        if let Some(src) = dun_render::get_dun_frame(&level.level_cel, block.frame() as u32) {
            dun_render::render_tile_frame(
                out,
                target_buffer_position,
                TileType::LeftTriangle,
                src,
                Some(tbl),
                MaskType::Solid,
            );
        }
    }
    let block = mega.blocks[1];
    if block.has_value() {
        let pos = target_buffer_position + RIGHT_FRAME_DISPLACEMENT;
        if let Some(src) = dun_render::get_dun_frame(&level.level_cel, block.frame() as u32) {
            dun_render::render_tile_frame(
                out,
                pos,
                TileType::RightTriangle,
                src,
                Some(tbl),
                MaskType::Solid,
            );
        }
    }
}

/// C++ `DrawFloor`（scrollrt.cpp:927）。菱形网格 zigzag 迭代。
fn draw_floor(
    out: &mut super::surface::Surface,
    level: &DungeonLevelData,
    grid: &dyn DPieceGrid,
    mut tile_position: Point,
    mut target_buffer_position: Point,
    rows: i32,
    mut columns: i32,
    lighting: &Lighting,
) {
    for i in 0..rows {
        for _ in 0..columns {
            if !in_dungeon_bounds(tile_position) {
                dun_render::draw_black_tile(out, target_buffer_position.x, target_buffer_position.y);
            } else {
                let piece_id = grid.d_piece(tile_position.x, tile_position.y);
                if level.sol.is_floor(piece_id) {
                    draw_floor_tile(out, level, piece_id, tile_position, target_buffer_position, lighting);
                }
            }
            tile_position += Direction::East;
            target_buffer_position.x += TILE_WIDTH;
        }
        // 回到行首。
        tile_position = tile_position + Direction::West.to_displacement() * columns;
        target_buffer_position.x -= columns * TILE_WIDTH;

        // 跳到下一行（zigzag）。
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

/// C++ `DrawCell`（scrollrt.cpp:521）。渲染单个 mega-tile 的全部 micro：mt[0]/mt[1]
/// 地板对（仅对非地板 piece 或 foliage），再 mt[2..MicroTileLen] 墙壁逐行上移。
///
/// 本增量 `transparency` 强制为 false（全 `MaskType::Solid`，C++:540 需要
/// `dTransVal`/`TransList` 未移植）；光照全亮（`light_table = None`）。foliage
/// 渲染（C++ `RenderTileFoliage`）在活 dun_render 中未移植，地板透明顶暂不绘制。
fn draw_cell(
    out: &mut super::surface::Surface,
    level: &DungeonLevelData,
    piece_id: u16,
    tile_position: Point,
    target_buffer_position: Point,
    lighting: &Lighting,
) {
    let Some(mega) = mega_for_piece(level, piece_id) else {
        return;
    };
    let micro_tile_len = level.dungeon_type.blocks_per_tile();
    let is_floor = level.sol.is_floor(piece_id);
    let tbl = lighting.table_for(tile_position.x, tile_position.y);
    let mut tbp = target_buffer_position;

    // C++ scrollrt.cpp:540 — the tile is see-through only when the SOL data
    // marks it Transparent AND the per-frame TransList enables its region.
    let transparency = level.sol.tile_has_any(piece_id, TileProperties::TRANSPARENT)
        && lighting.transparency_for(tile_position.x, tile_position.y);

    // C++ getFirstTileMaskLeft/Right (scrollrt.cpp:547-578).
    let first_mask_left = |tile_type: TileType| -> MaskType {
        if !transparency {
            return MaskType::Solid;
        }
        match tile_type {
            TileType::LeftTrapezoid | TileType::TransparentSquare => {
                if level.sol.tile_has_any(piece_id, TileProperties::TRANSPARENT_LEFT) {
                    MaskType::Left
                } else {
                    MaskType::Solid
                }
            }
            TileType::LeftTriangle => MaskType::Solid,
            _ => MaskType::Transparent,
        }
    };
    let first_mask_right = |tile_type: TileType| -> MaskType {
        if !transparency {
            return MaskType::Solid;
        }
        match tile_type {
            TileType::RightTrapezoid | TileType::TransparentSquare => {
                if level.sol.tile_has_any(piece_id, TileProperties::TRANSPARENT_RIGHT) {
                    MaskType::Right
                } else {
                    MaskType::Solid
                }
            }
            TileType::RightTriangle => MaskType::Solid,
            _ => MaskType::Transparent,
        }
    };

    // mt[0] — 左地板/叶半（C++:588-599）。
    let block = mega.blocks[0];
    if block.has_value() {
        let tile_type = block.tile_type();
        if !is_floor || tile_type == TileType::TransparentSquare {
            if !(is_floor && tile_type == TileType::TransparentSquare) {
                dun_render::render_tile(out, tbp, &level.level_cel, block, first_mask_left(tile_type), Some(tbl));
            }
            // foliage 分支跳过（活 dun_render 无 render_tile_foliage）。
        }
    }
    // mt[1] — 右地板/叶半（C++:600-611）。
    let block = mega.blocks[1];
    if block.has_value() {
        let tile_type = block.tile_type();
        if !is_floor || tile_type == TileType::TransparentSquare {
            if !(is_floor && tile_type == TileType::TransparentSquare) {
                dun_render::render_tile(
                    out,
                    tbp + RIGHT_FRAME_DISPLACEMENT,
                    &level.level_cel,
                    block,
                    first_mask_right(tile_type),
                    Some(tbl),
                );
            }
        }
    }
    tbp.y -= TILE_HEIGHT;

    // 墙壁：mt[2..MicroTileLen] 成对，每行上移 TILE_HEIGHT。C++ 用
    // `transparency ? MaskType::Transparent : MaskType::Solid`。
    let wall_mask = if transparency { MaskType::Transparent } else { MaskType::Solid };
    let mut i = 2;
    while i < micro_tile_len {
        let block = mega.blocks[i];
        if block.has_value() {
            dun_render::render_tile(out, tbp, &level.level_cel, block, wall_mask, Some(tbl));
        }
        let block = mega.blocks[i + 1];
        if block.has_value() {
            dun_render::render_tile(
                out,
                tbp + RIGHT_FRAME_DISPLACEMENT,
                &level.level_cel,
                block,
                wall_mask,
                Some(tbl),
            );
        }
        tbp.y -= TILE_HEIGHT;
        i += 2;
    }
}

/// C++ `DrawTileContent`（scrollrt.cpp:966）。zigzag 迭代，每 tile 调
/// `draw_cell`，含 wall-behind 前置绘制（C++:983-998），防止精灵穿墙。
fn draw_tile_content(
    out: &mut super::surface::Surface,
    level: &DungeonLevelData,
    grid: &dyn DPieceGrid,
    mut tile_position: Point,
    mut target_buffer_position: Point,
    rows: i32,
    mut columns: i32,
    lighting: &Lighting,
) {
    let micro_tile_len = level.dungeon_type.blocks_per_tile() as i32;
    let mut rows = rows + micro_tile_len;
    let screen_width = out.w();
    let mut skip = false;

    let mut i = 0;
    while i < rows {
        let mut skip_next = false;
        for _ in 0..columns {
            if in_dungeon_bounds(tile_position) {
                // wall-behind 前置绘制（C++:983-994）。
                if tile_position.x + 1 < MAXDUN
                    && tile_position.y - 1 >= 0
                    && target_buffer_position.x + TILE_WIDTH <= screen_width
                {
                    let piece = grid.d_piece(tile_position.x, tile_position.y);
                    if level.sol.is_wall(piece) {
                        let east_is_wall =
                            level.sol.is_wall(grid.d_piece(tile_position.x + 1, tile_position.y));
                        let west_is_wall = if tile_position.x > 0 {
                            level.sol.is_wall(grid.d_piece(tile_position.x - 1, tile_position.y))
                        } else {
                            false
                        };
                        if east_is_wall || west_is_wall {
                            let ne_not_solid = level
                                .sol
                                .is_tile_not_solid(grid.d_piece(tile_position.x + 1, tile_position.y - 1));
                            let n_not_solid = level
                                .sol
                                .is_tile_not_solid(grid.d_piece(tile_position.x, tile_position.y - 1));
                            if ne_not_solid && n_not_solid {
                                // 先绘制东侧 tile（C++:990）。
                                let east_pos = Point::new(
                                    target_buffer_position.x + TILE_WIDTH,
                                    target_buffer_position.y,
                                );
                                let east_piece = grid.d_piece(tile_position.x + 1, tile_position.y);
                                draw_cell(
                                    out,
                                    level,
                                    east_piece,
                                    Point::new(tile_position.x + 1, tile_position.y),
                                    east_pos,
                                    lighting,
                                );
                                skip_next = true;
                            }
                        }
                    }
                }
                if !skip {
                    let piece = grid.d_piece(tile_position.x, tile_position.y);
                    draw_cell(out, level, piece, tile_position, target_buffer_position, lighting);
                }
                skip = skip_next;
            }
            tile_position += Direction::East;
            target_buffer_position.x += TILE_WIDTH;
        }
        // 回到行首 + 跳到下一行（与 draw_floor 相同的 zigzag）。
        tile_position = tile_position + Direction::West.to_displacement() * columns;
        target_buffer_position.x -= columns * TILE_WIDTH;

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
        i += 1;
    }
}

/// C++ `CalcFirstTilePosition`（scrollrt.cpp:1081）。本增量非行走、无面板覆盖：
/// offset = tileOffset，position += tileShift。
fn calc_first_tile_position(position: &mut Point, offset: &mut Displacement, geom: TileViewport) {
    *offset = geom.tile_offset;
    *position = *position + geom.tile_shift;
}

/// C++ `DrawGame`（scrollrt.cpp:1132）。先 DrawFloor 再 DrawTileContent。
/// 渲染到整个后备缓冲（HUD 由调用方叠加，故不做 viewport subregionY）。
fn draw_game(
    out: &mut super::surface::Surface,
    level: &DungeonLevelData,
    grid: &dyn DPieceGrid,
    mut position: Point,
    geom: TileViewport,
    lighting: &Lighting,
) {
    let mut offset = Displacement::new(0, 0);
    calc_first_tile_position(&mut position, &mut offset, geom);
    let target_start = Point::new(0, 0) + offset;
    draw_floor(out, level, grid, position, target_start, geom.tile_rows, geom.tile_columns, lighting);
    draw_tile_content(out, level, grid, position, target_start, geom.tile_rows, geom.tile_columns, lighting);
}

/// C++ `DrawView`（scrollrt.cpp:1215）。忠实管线入口：由屏幕尺寸算视口几何，
/// 再 DrawGame。实体/automap/标签/血条（C++:1223-1296）本增量仍由调用方在
/// SDL canvas 上叠加。
pub fn draw_view(
    out: &mut super::surface::Surface,
    level: &DungeonLevelData,
    grid: &dyn DPieceGrid,
    start_position: Point,
    screen_w: i32,
    screen_h: i32,
    viewport_h: i32,
    lighting: &Lighting,
) {
    let geom = viewport_geometry(screen_w, screen_h, viewport_h);
    draw_game(out, level, grid, start_position, geom, lighting);
}

/// C++ `GetScreenPosition`（scrollrt.cpp:1620）。返回 tile 在后备缓冲中的屏幕
/// 锚点位置（tile 底部）。实体 overlay 用本函数与地板管线共享同一套投影，
/// 保证地板/实体天然对齐（不再各自手算 rel_x/rel_y）。
pub fn tile_screen_position(
    tile: Point,
    camera: Point,
    screen_w: i32,
    screen_h: i32,
    viewport_h: i32,
) -> Point {
    let geom = viewport_geometry(screen_w, screen_h, viewport_h);
    let mut first_tile = camera;
    let mut offset = Displacement::new(0, 0);
    calc_first_tile_position(&mut first_tile, &mut offset, geom);
    // delta = firstTile - tile（C++ 用反序 delta 喂给 worldToScreen）。
    let delta = first_tile - tile;
    let mut position = Point::new(0, 0);
    position += delta.world_to_screen();
    position += offset;
    position
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全亮光照测试夹具：恒等光照表 + 全 0 dLight 网格。
    fn test_lighting() -> ([[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS], Vec<u8>) {
        let mut tables = [[0u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS];
        for tbl in tables.iter_mut() {
            for i in 0..LIGHT_TABLE_SIZE {
                tbl[i] = i as u8;
            }
        }
        let dlight = vec![0u8; MAXDUN as usize * MAXDUN as usize];
        (tables, dlight)
    }

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

    /// 640×480 / 面板 128 / 无 zoom 的忠实视口几何（手算自 C++ CalcViewportGeometry）。
    #[test]
    fn test_viewport_geometry_golden() {
        let g = viewport_geometry(640, 480, 352);
        assert_eq!(g.tile_rows, 25);
        assert_eq!(g.tile_columns, 10);
        // tileShift = North*6 + West*5 + NorthEast = (-6,-6)+(-5,5)+(0,-1) = (-11,-2)
        assert_eq!(g.tile_shift, Displacement::new(-11, -2));
        // tileOffset = (startPos.x-32, startPos.y+15) where startPos=(32,-32) → (0,-17)
        assert_eq!(g.tile_offset, Displacement::new(0, -17));
    }

    /// tile_screen_position 与地板管线共享投影：相机 tile 锚点位置确定，
    /// 且 East 相邻 tile 在屏幕上 +64x（与 zigzag 列步进一致，无镜像）。
    #[test]
    fn test_tile_screen_position() {
        let cam = Point::new(75, 68);
        // viewport_h=336 (480-144 Rust 面板)：相机锚点 = (288,183)。
        let cam_pos = tile_screen_position(cam, cam, 640, 480, 336);
        assert_eq!(cam_pos, Point::new(288, 183));
        // East 相邻 tile (dx=1,dy=-1) → 屏幕 +64x、同 y。
        let east = tile_screen_position(Point::new(76, 67), cam, 640, 480, 336);
        assert_eq!(east, Point::new(cam_pos.x + 64, cam_pos.y));
        // SouthEast 相邻 tile (dx=1,dy=0) → 屏幕 +32x、+16y（iso 向下，SDL y 向下增）。
        let se = tile_screen_position(Point::new(76, 68), cam, 640, 480, 336);
        assert_eq!(se, Point::new(cam_pos.x + 32, cam_pos.y + 16));
    }

    /// 端到端渲染正确性：合成一个 packed 左三角帧（全填色 150），经
    /// `draw_floor_tile` 渲染到 Surface，断言三角形像素落在 C++ 预期位置
    /// （底边在 position.y、向右对齐、向上增长到满宽）。验证 dun_render 的
    /// 帧解码与屏幕朝向正确——这是光照/实体入面之前的地基层验证。
    #[test]
    fn test_render_floor_triangle_pixels() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MegaTile, LevelCelBlock};
        use crate::engine::surface::Surface;

        // level_cel: [num_frames=1][frame1_offset=8] + 512 字节三角帧（全色 150）。
        // 重编码左三角帧 = 512 字节（下三角 2+4+...+32=272 + 上三角 30+...+2=240）。
        let mut level_cel = Vec::new();
        level_cel.extend_from_slice(&1u32.to_le_bytes());
        level_cel.extend_from_slice(&8u32.to_le_bytes());
        level_cel.extend_from_slice(&[150u8; 512]);

        let mut level = DungeonLevelData::new(DungeonType::Cathedral);
        level.level_cel = level_cel;
        // mega-tile：blocks[0] = frame 1（draw_floor_tile 强制按左三角渲染）。
        let mut blocks = [LevelCelBlock::default(); 16];
        blocks[0] = LevelCelBlock::new(0x0001); // frame 1
        level.min.mega_tiles = vec![MegaTile { blocks }];

        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        // 在 target (100, 200) 渲染单个地板 tile（仅 mt[0] 左三角，mt[1] 空）。
        let (tables, dlight) = test_lighting();
        let lighting = Lighting::new(&dlight, &tables);
        draw_floor_tile(&mut surface, &level, 1, Point::new(0, 0), Point::new(100, 200), &lighting);

        // 左三角 row 0（底边 y=200）：2px 右对齐于 x=130,131。
        assert_eq!(surface.at(130, 200).copied(), Some(150), "bottom-right pixel");
        assert_eq!(surface.at(100, 200).copied(), Some(0), "bottom-left outside triangle");
        // 左三角 row 15（顶部 y=185）：满宽 32px，含最左 x=100。
        assert_eq!(surface.at(100, 185).copied(), Some(150), "top-row leftmost pixel");
        // 三角形外（顶部行的左侧之外）应为 0。
        assert_eq!(surface.at(99, 185).copied(), Some(0), "just left of triangle");
    }

    /// 墙壁渲染正确性：合成方块帧放入 mega-tile 的 blocks[2]（第一层墙），
    /// 经 `draw_cell` 渲染到 Surface，断言方块像素落在地板上方一行
    /// （target.y - TILE_HEIGHT），且方块行 0 在底部（C++ 朝向）。
    #[test]
    fn test_render_wall_square_pixels() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MegaTile, LevelCelBlock};
        use crate::engine::surface::Surface;

        // 方块帧 = 1024 字节（32x32）。底行（前 32 字节）= 色 100，其余 = 色 50。
        let mut frame = vec![50u8; 1024];
        for b in frame.iter_mut().take(32) {
            *b = 100;
        }
        let mut level_cel = Vec::new();
        level_cel.extend_from_slice(&1u32.to_le_bytes());
        level_cel.extend_from_slice(&8u32.to_le_bytes());
        level_cel.extend_from_slice(&frame);

        let mut level = DungeonLevelData::new(DungeonType::Cathedral);
        level.level_cel = level_cel;
        // blocks[2] = 方块 frame 1（tile_type Square=0 → data = frame 1 = 0x0001）。
        let mut blocks = [LevelCelBlock::default(); 16];
        blocks[2] = LevelCelBlock::new(0x0001);
        level.min.mega_tiles = vec![MegaTile { blocks }];

        struct G;
        impl DPieceGrid for G {
            fn d_piece(&self, _x: i32, _y: i32) -> u16 { 1 }
        }
        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        // draw_cell 在 target (100,200)：blocks[2] 墙渲染在 y=200-32=168。
        let (tables, dlight) = test_lighting();
        let lighting = Lighting::new(&dlight, &tables);
        draw_cell(&mut surface, &level, 1, Point::new(0, 0), Point::new(100, 200), &lighting);

        // 方块底行（frame row 0）在 y=168，色 100。
        assert_eq!(surface.at(100, 168).copied(), Some(100), "square bottom-left");
        assert_eq!(surface.at(131, 168).copied(), Some(100), "square bottom-right");
        // 上一行（frame row 1）色 50。
        assert_eq!(surface.at(100, 167).copied(), Some(50), "square row 1");
        // 地板行（y=200 附近）无 mt[0]/mt[1]（空 block）→ 应为 0。
        assert_eq!(surface.at(100, 200).copied(), Some(0), "floor row empty");
    }

    /// 光照生效：同一 tile 在全暗 dLight（表 15=全黑）下渲染为黑，在全亮
    /// dLight（表 0=恒等）下渲染出原色——证明 dLight→查表→render_tile_frame
    /// 的光照路径端到端工作。
    #[test]
    fn test_render_applies_light_table() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType, MegaTile, LevelCelBlock};
        use crate::engine::surface::Surface;
        use crate::engine::lighting::LightManager;

        // 左三角帧全填色 150。
        let mut level_cel = Vec::new();
        level_cel.extend_from_slice(&1u32.to_le_bytes());
        level_cel.extend_from_slice(&8u32.to_le_bytes());
        level_cel.extend_from_slice(&[150u8; 512]);
        let mut level = DungeonLevelData::new(DungeonType::Cathedral);
        level.level_cel = level_cel;
        let mut blocks = [LevelCelBlock::default(); 16];
        blocks[0] = LevelCelBlock::new(0x0001);
        level.min.mega_tiles = vec![MegaTile { blocks }];

        let mut lm = LightManager::new();
        lm.make_light_table();

        // 全暗（level 15 → 黑表）：渲染全黑。
        let dlight_dark = vec![15u8; MAXDUN as usize * MAXDUN as usize];
        let lighting_dark = Lighting::new(&dlight_dark, &lm.tables);
        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        draw_floor_tile(&mut surface, &level, 1, Point::new(50, 50), Point::new(100, 200), &lighting_dark);
        assert_eq!(surface.at(130, 200).copied(), Some(0), "dark light → black pixel");

        // 全亮（level 0 → 恒等表）：渲染出色 150。
        let dlight_lit = vec![0u8; MAXDUN as usize * MAXDUN as usize];
        let lighting_lit = Lighting::new(&dlight_lit, &lm.tables);
        let mut buf2 = vec![0u8; 640 * 480];
        let mut surface2 = Surface::new(&mut buf2, 640, 640, 480);
        draw_floor_tile(&mut surface2, &level, 1, Point::new(50, 50), Point::new(100, 200), &lighting_lit);
        assert_eq!(surface2.at(130, 200).copied(), Some(150), "lit → color 150");
    }

    /// `DPieceGrid` 记录每次查询的 `(x,y)`，返回 piece 0，使渲染器的
    /// `is_floor`/`mega_for_piece` 路径空转，从而观察迭代序列本身。
    struct RecordingGrid {
        visits: std::cell::RefCell<Vec<(i32, i32)>>,
    }
    impl DPieceGrid for RecordingGrid {
        fn d_piece(&self, x: i32, y: i32) -> u16 {
            self.visits.borrow_mut().push((x, y));
            0
        }
    }

    /// draw_floor 的 zigzag 访问序列黄金值（起始 tile (50,50)，3 行 4 列）。
    #[test]
    fn test_draw_floor_zigzag_sequence() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType};
        use crate::engine::surface::Surface;

        let grid = RecordingGrid { visits: std::cell::RefCell::new(Vec::new()) };
        let level = DungeonLevelData::new(DungeonType::Town);
        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        let (tables, dlight) = test_lighting();
        let lighting = Lighting::new(&dlight, &tables);
        draw_floor(
            &mut surface,
            &level,
            &grid,
            Point::new(50, 50),
            Point::new(0, 0),
            3,
            4,
            &lighting,
        );
        let visits = grid.visits.borrow();
        // Row 0 (columns=4). 内层每访问后 += East，故行末 tile 已前移一格。
        assert_eq!(&visits[..4], &[(50, 50), (51, 49), (52, 48), (53, 47)]);
        // Row 1 (columns grew to 5): 回行首 + West*4 后 (50,50)，偶行 y+=1 → (50,51)。
        assert_eq!(
            &visits[4..9],
            &[(50, 51), (51, 50), (52, 49), (53, 48), (54, 47)]
        );
        // Row 2 (columns shrank to 4): 行首 (51,51)。
        assert_eq!(&visits[9..13], &[(51, 51), (52, 50), (53, 49), (54, 48)]);
    }

    /// draw_view 管线 smoke：空 level + 记录 grid 不 panic，且访问真实视口量级
    /// 的 tile（640×480 ≈ 10 列 × 25 行）。
    #[test]
    fn test_draw_view_pipeline_smoke() {
        use crate::engine::dungeon::{DungeonLevelData, DungeonType};
        use crate::engine::surface::Surface;

        let grid = RecordingGrid { visits: std::cell::RefCell::new(Vec::new()) };
        let level = DungeonLevelData::new(DungeonType::Town);
        let mut buf = vec![0u8; 640 * 480];
        let mut surface = Surface::new(&mut buf, 640, 640, 480);
        let (tables, dlight) = test_lighting();
        let lighting = Lighting::new(&dlight, &tables);
        draw_view(&mut surface, &level, &grid, Point::new(75, 68), 640, 480, 352, &lighting);
        let n = grid.visits.borrow().len();
        assert!(n > 100, "draw_view should visit a full viewport, got {n} queries");
    }
    #[test]
    fn test_transparency_for_follows_translist_semantics() {
        // C++ scrollrt.cpp:540 — `TransList[dTransVal[x][y]]`, guarded by a
        // non-zero dTransVal and bounds.
        let (tables, dlight) = test_lighting();

        // No transparency data -> always opaque.
        let plain = Lighting::new(&dlight, &tables);
        assert!(!plain.transparency_for(56, 56));

        let mut trans_val = vec![0i8; MAXDUN as usize * MAXDUN as usize];
        trans_val[56 * MAXDUN as usize + 56] = 3;
        let mut trans_list = vec![false; 16];

        // Region not enabled -> opaque.
        let off = Lighting::with_transparency(&dlight, &tables, &trans_val, &trans_list);
        assert!(!off.transparency_for(56, 56));

        // Region enabled -> see-through.
        trans_list[3] = true;
        let on = Lighting::with_transparency(&dlight, &tables, &trans_val, &trans_list);
        assert!(on.transparency_for(56, 56));

        // Zero dTransVal tiles stay opaque even when the list is on.
        assert!(!on.transparency_for(57, 56));
        // Out-of-bounds is always opaque.
        assert!(!on.transparency_for(-1, 56));
        assert!(!on.transparency_for(200, 56));
    }


}
