//! Mega-tile 浏览器 - 查看所有预渲染的瓦片纹理
//!
//! 用方向键浏览不同的 mega-tile，找到正确的地板/墙壁索引

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use devilutionx_rs::engine::{
    DungeonLevelData, TileDecoder, FRAME_WIDTH, FRAME_HEIGHT,
};
use devilutionx_rs::engine::dungeon::DungeonType as LevelType;
use devilutionx_rs::engine::mpq::MpqArchive;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const TILE_WIDTH: i32 = 64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 SDL
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Mega-Tile Browser", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // 加载 MPQ
    println!("加载 spawn.mpq...");
    let mpq_paths = vec![
        "spawn.mpq",
        "../spawn.mpq",
        "d:/Users/gxh/Documents/GitHub/DevilutionX/devilutionx-rs/spawn.mpq",
    ];

    let mut spawn = None;
    for path in &mpq_paths {
        println!("  尝试路径: {}", path);
        if let Ok(archive) = MpqArchive::open(path) {
            println!("  成功打开: {}", path);
            spawn = Some(archive);
            break;
        }
    }
    let mut spawn = spawn.ok_or("找不到 spawn.mpq!")?;

    // 加载 Cathedral 瓦片数据
    println!("加载 Cathedral 瓦片数据...");
    let level_data = DungeonLevelData::load_from_mpq(&mut spawn, LevelType::Cathedral)?;
    println!("  - MIN: {} 个 mega-tiles", level_data.min.mega_tiles.len());

    // 预渲染所有 mega-tile 纹理
    println!("预渲染瓦片纹理...");
    let mut mega_tile_textures: Vec<sdl2::render::Texture> = Vec::new();
    let texture_height: usize = 160;

    for (i, mega_tile) in level_data.min.mega_tiles.iter().enumerate() {
        let mut tile_rgba = vec![0u8; TILE_WIDTH as usize * texture_height * 4];

        let num_blocks = level_data.min.blocks_per_tile.min(10);

        for block_idx in 0..num_blocks {
            let block = &mega_tile.blocks[block_idx];
            if !block.has_value() {
                continue;
            }

            let frame = block.frame();
            let tile_type = block.tile_type();

            if let Some(pixels) = TileDecoder::decode_tile(
                &level_data.level_cel,
                frame,
                tile_type,
                &level_data.palette,
            ) {
                let col = block_idx & 1;
                let row = block_idx / 2;
                let block_x = col * FRAME_WIDTH;
                let block_y = texture_height - (row + 1) * FRAME_HEIGHT;

                for py in 0..FRAME_HEIGHT {
                    for px in 0..FRAME_WIDTH {
                        let src_idx = (py * FRAME_WIDTH + px) * 4;
                        let dst_x = block_x + px;
                        let dst_y = block_y + py;
                        let dst_idx = (dst_y * TILE_WIDTH as usize + dst_x) * 4;
                        if dst_idx + 3 < tile_rgba.len() && src_idx + 3 < pixels.len() {
                            tile_rgba[dst_idx] = pixels[src_idx];
                            tile_rgba[dst_idx + 1] = pixels[src_idx + 1];
                            tile_rgba[dst_idx + 2] = pixels[src_idx + 2];
                            tile_rgba[dst_idx + 3] = pixels[src_idx + 3];
                        }
                    }
                }
            }
        }

        let mut texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::ABGR8888,
            TILE_WIDTH as u32,
            texture_height as u32,
        )?;
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        texture.update(None, &tile_rgba, TILE_WIDTH as usize * 4)?;
        mega_tile_textures.push(texture);

        if i % 100 == 0 {
            println!("  已渲染 {}/{} mega-tiles", i, level_data.min.mega_tiles.len());
        }
    }
    println!("  完成! 共 {} 个瓦片纹理", mega_tile_textures.len());

    let mut event_pump = sdl_context.event_pump()?;
    let mut current_idx: usize = 0;
    let tiles_per_row = 10;
    let tiles_per_page = tiles_per_row * 4;

    println!("\n=== 操作说明 ===");
    println!("方向键 上/下: 翻页");
    println!("方向键 左/右: 切换单个瓦片");
    println!("数字键 0-9: 快速跳转到第 N*100 个瓦片");
    println!("ESC: 退出");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Left => {
                            if current_idx > 0 {
                                current_idx -= 1;
                            }
                        }
                        Keycode::Right => {
                            if current_idx + 1 < mega_tile_textures.len() {
                                current_idx += 1;
                            }
                        }
                        Keycode::Up => {
                            if current_idx >= tiles_per_page {
                                current_idx -= tiles_per_page;
                            } else {
                                current_idx = 0;
                            }
                        }
                        Keycode::Down => {
                            current_idx = (current_idx + tiles_per_page).min(mega_tile_textures.len() - 1);
                        }
                        Keycode::Num0 => current_idx = 0,
                        Keycode::Num1 => current_idx = 100.min(mega_tile_textures.len() - 1),
                        Keycode::Num2 => current_idx = 200.min(mega_tile_textures.len() - 1),
                        Keycode::Num3 => current_idx = 300.min(mega_tile_textures.len() - 1),
                        Keycode::Num4 => current_idx = 400.min(mega_tile_textures.len() - 1),
                        Keycode::Num5 => current_idx = 500.min(mega_tile_textures.len() - 1),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // 清屏
        canvas.set_draw_color(sdl2::pixels::Color::RGB(40, 40, 40));
        canvas.clear();

        // 显示当前页的瓦片
        let page_start = (current_idx / tiles_per_page) * tiles_per_page;

        for i in 0..tiles_per_page {
            let idx = page_start + i;
            if idx >= mega_tile_textures.len() {
                break;
            }

            let col = (i % tiles_per_row) as i32;
            let row = (i / tiles_per_row) as i32;

            let x = 20 + col * 75;
            let y = 50 + row * 170;

            // 高亮当前选中的
            if idx == current_idx {
                canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 0));
                canvas.draw_rect(Rect::new(x - 2, y - 2, 68, 164))?;
            }

            // 绘制瓦片
            let dst = Rect::new(x, y, 64, 160);
            canvas.copy(&mega_tile_textures[idx], None, dst)?;

            // 绘制索引号
            // (SDL2 没有内置文字渲染，用颜色方块代替)
        }

        // 显示标题信息 (用颜色条表示当前索引)
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        // 画一个进度条表示当前位置
        let progress = current_idx as f32 / mega_tile_textures.len() as f32;
        let bar_width = (SCREEN_WIDTH as f32 * progress) as u32;
        canvas.fill_rect(Rect::new(0, 0, bar_width, 5))?;

        // 在窗口标题显示当前索引
        canvas.window_mut().set_title(&format!(
            "Mega-Tile Browser - 索引: {} / {} (页 {})",
            current_idx,
            mega_tile_textures.len(),
            current_idx / tiles_per_page
        ))?;

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}
