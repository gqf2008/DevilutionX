//! Diablo 瓦片查看器
//!
//! 从 spawn.mpq 加载真正的 Diablo 瓦片并显示

use devilutionx_rs::engine::dungeon::{
    DungeonType, DungeonLevelData, TileDecoder, TileType,
    FRAME_WIDTH, FRAME_HEIGHT, TILE_WIDTH, TILE_HEIGHT,
};
use devilutionx_rs::engine::mpq::MpqArchive;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::time::Duration;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Diablo 瓦片查看器 ===");

    // 初始化 SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Diablo Tile Viewer", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // 加载 MPQ
    println!("加载 spawn.mpq...");
    let mpq_path = "spawn.mpq";  // 相对路径，从项目根目录运行
    let mut mpq = MpqArchive::open(mpq_path)?;

    // 加载 Cathedral (l1) 数据
    println!("加载 Cathedral 瓦片数据...");
    let level_data = DungeonLevelData::load_from_mpq(&mut mpq, DungeonType::Cathedral)?;

    println!("  - 调色板: {} 字节", level_data.palette.colors.len());
    println!("  - SOL: {} 项", level_data.sol.properties.len());
    println!("  - MIN: {} 个 mega-tiles", level_data.min.mega_tiles.len());
    println!("  - CEL: {} 字节", level_data.level_cel.len());

    // 打印调色板前几个颜色
    println!("\n调色板前10个颜色:");
    for i in 0..10 {
        let (r, g, b) = level_data.palette.get_rgb(i);
        println!("  [{}] RGB({}, {}, {})", i, r, g, b);
    }    // 当前查看的 mega-tile 索引
    let mut current_mega_tile: usize = 0;
    let max_mega_tiles = level_data.min.mega_tiles.len();
    let mut zoom = 3;

    // 创建瓦片纹理 (32x32 for each micro-tile)
    let mut tile_textures: Vec<sdl2::render::Texture> = Vec::new();
    for _ in 0..16 {  // 最多 16 个 micro-tiles per mega-tile
        let tex = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGBA8888,
            FRAME_WIDTH as u32,
            FRAME_HEIGHT as u32,
        )?;
        tile_textures.push(tex);
    }

    let mut event_pump = sdl_context.event_pump()?;

    println!("\n按键说明:");
    println!("  左/右箭头: 前后翻页");
    println!("  上/下箭头: 跳10个");
    println!("  PageUp/PageDown: 跳100个");
    println!("  +/-: 缩放");
    println!("  ESC: 退出\n");

    'running: loop {
        // 处理事件
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::Left => {
                            if current_mega_tile > 0 {
                                current_mega_tile -= 1;
                            }
                        }
                        Keycode::Right => {
                            if current_mega_tile + 1 < max_mega_tiles {
                                current_mega_tile += 1;
                            }
                        }
                        Keycode::Up => {
                            if current_mega_tile >= 10 {
                                current_mega_tile -= 10;
                            } else {
                                current_mega_tile = 0;
                            }
                        }
                        Keycode::Down => {
                            current_mega_tile = (current_mega_tile + 10).min(max_mega_tiles - 1);
                        }
                        Keycode::PageUp => {
                            if current_mega_tile >= 100 {
                                current_mega_tile -= 100;
                            } else {
                                current_mega_tile = 0;
                            }
                        }
                        Keycode::PageDown => {
                            current_mega_tile = (current_mega_tile + 100).min(max_mega_tiles - 1);
                        }
                        Keycode::Plus | Keycode::Equals => {
                            zoom = (zoom + 1).min(8);
                        }
                        Keycode::Minus => {
                            zoom = (zoom - 1).max(1);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // 清除屏幕
        canvas.set_draw_color(sdl2::pixels::Color::RGB(32, 32, 48));
        canvas.clear();

        // 获取当前 mega-tile
        let mega_tile = &level_data.min.mega_tiles[current_mega_tile];

        // 解码并显示每个 micro-tile
        let blocks_per_tile = level_data.min.blocks_per_tile;
        let tile_size = FRAME_WIDTH as u32 * zoom;

        // Mega-tile 布局 (2 列, blocks_per_tile/2 行, 从底部开始)
        // 等距视图: 左右两列
        let base_x = (SCREEN_WIDTH - tile_size * 2) / 2;
        let base_y = 50;

        let mut valid_blocks = 0;
        let mut debug_info = String::new();

        for block_idx in 0..blocks_per_tile.min(16) {
            let block = &mega_tile.blocks[block_idx];

            if !block.has_value() {
                continue;
            }

            valid_blocks += 1;

            let frame = block.frame();
            let tile_type = block.tile_type();

            debug_info.push_str(&format!("[{}] F:{} T:{:?} ", block_idx, frame, tile_type));

            // 解码瓦片
            if let Some(pixels) = TileDecoder::decode_tile(
                &level_data.level_cel,
                frame,
                tile_type,
                &level_data.palette,
            ) {
                // 更新纹理
                tile_textures[block_idx].update(None, &pixels, FRAME_WIDTH * 4)?;

                // 计算显示位置
                // blocks 布局: [0,1] 在底部, [2,3] 在上一行, etc.
                // 列: block_idx & 1 (0=左, 1=右)
                // 行: block_idx / 2 (从下往上)
                let col = block_idx & 1;
                let row = block_idx / 2;

                let x = base_x + col as u32 * tile_size;
                let y = base_y + ((blocks_per_tile / 2 - 1 - row) as u32) * tile_size;

                // 设置混合模式以支持透明度
                tile_textures[block_idx].set_blend_mode(sdl2::render::BlendMode::Blend);

                // 绘制瓦片
                canvas.copy(
                    &tile_textures[block_idx],
                    None,
                    Some(Rect::new(x as i32, y as i32, tile_size, tile_size)),
                )?;
            }
        }

        // 每秒打印一次调试信息
        static mut LAST_PRINT: std::time::Instant = unsafe { std::mem::zeroed() };
        static mut INITIALIZED: bool = false;
        unsafe {
            if !INITIALIZED {
                LAST_PRINT = std::time::Instant::now();
                INITIALIZED = true;
            }
            if LAST_PRINT.elapsed() > Duration::from_secs(1) {
                println!("MegaTile {}: {} blocks", current_mega_tile, valid_blocks);
                if !debug_info.is_empty() {
                    println!("  {}", debug_info);
                }
                LAST_PRINT = std::time::Instant::now();
            }
        }

        // 显示信息
        let title = format!("MegaTile {}/{} | {} valid blocks | Zoom {}x",
            current_mega_tile + 1, max_mega_tiles, valid_blocks, zoom);
        canvas.window_mut().set_title(&title)?;

        // 绘制网格线
        canvas.set_draw_color(sdl2::pixels::Color::RGB(80, 80, 100));
        for row in 0..=(blocks_per_tile / 2) {
            let y = base_y + row as u32 * tile_size;
            canvas.draw_line(
                (base_x as i32, y as i32),
                ((base_x + tile_size * 2) as i32, y as i32),
            )?;
        }
        for col in 0..=2 {
            let x = base_x + col as u32 * tile_size;
            canvas.draw_line(
                (x as i32, base_y as i32),
                (x as i32, (base_y + (blocks_per_tile / 2) as u32 * tile_size) as i32),
            )?;
        }

        // 显示块信息
        let info_y = base_y + (blocks_per_tile / 2) as u32 * tile_size + 20;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(48, 48, 64));
        canvas.fill_rect(Rect::new(10, info_y as i32, SCREEN_WIDTH - 20, 150))?;

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    println!("\n查看器关闭");
    Ok(())
}
