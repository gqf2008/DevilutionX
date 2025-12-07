//! Diablo Demo - 使用真实 MPQ 资源渲染
//!
//! 这个示例展示如何从 DIABDAT.MPQ 加载并显示真正的游戏资源
//!
//! 用法: cargo run --example diablo_demo <path_to_DIABDAT.MPQ>

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::time::Duration;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Diablo Demo - 真实资源渲染 ===\n");

    // 查找 MPQ 文件
    let mpq_path = std::env::args().nth(1).unwrap_or_else(|| {
        // 尝试常见路径
        let paths = [
            "DIABDAT.MPQ",
            "diabdat.mpq",
            "../DIABDAT.MPQ",
            "D:\\Games\\Diablo\\DIABDAT.MPQ",
            "C:\\Games\\Diablo\\DIABDAT.MPQ",
            "E:\\DIABDAT.MPQ",
        ];
        for p in paths {
            if std::path::Path::new(p).exists() {
                return p.to_string();
            }
        }
        "DIABDAT.MPQ".to_string()
    });

    println!("正在打开: {}", mpq_path);
    let mut mpq = devilutionx_rs::engine::mpq::MpqArchive::open(&mpq_path)?;
    println!("✓ MPQ 打开成功!\n");

    // 加载调色板
    println!("加载资源...");
    let palette = load_palette(&mut mpq)?;
    println!("  ✓ 调色板");

    // 加载城镇背景 PCX
    let town_pcx = load_town_image(&mut mpq, &palette)?;
    println!("  ✓ 城镇背景 ({}x{})", town_pcx.0, town_pcx.1);

    // 加载一些 UI 元素
    let logo = load_logo(&mut mpq, &palette)?;
    println!("  ✓ Logo");

    // 初始化 SDL
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("Diablo Demo - 真实资源", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();

    // 创建城镇纹理
    let mut town_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::ABGR8888,
        town_pcx.0,
        town_pcx.1
    )?;
    town_texture.update(None, &town_pcx.2, town_pcx.0 as usize * 4)?;

    // 创建 logo 纹理
    let mut logo_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::ABGR8888,
        logo.0,
        logo.1
    )?;
    logo_texture.update(None, &logo.2, logo.0 as usize * 4)?;
    logo_texture.set_blend_mode(sdl2::render::BlendMode::Blend);

    let mut event_pump = sdl.event_pump()?;
    let mut scroll_x: i32 = 0;
    let mut scroll_y: i32 = 0;
    let scroll_speed = 10;

    println!("\n=== 控制 ===");
    println!("  方向键/WASD: 滚动视图");
    println!("  ESC: 退出\n");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Left), .. } |
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    scroll_x = (scroll_x - scroll_speed).max(0);
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } |
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    scroll_x = (scroll_x + scroll_speed).min(town_pcx.0 as i32 - SCREEN_WIDTH as i32);
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } |
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    scroll_y = (scroll_y - scroll_speed).max(0);
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } |
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    scroll_y = (scroll_y + scroll_speed).min(town_pcx.1 as i32 - SCREEN_HEIGHT as i32);
                }
                _ => {}
            }
        }

        // 清屏
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        // 绘制城镇背景（带滚动）
        let src_rect = Rect::new(scroll_x, scroll_y, SCREEN_WIDTH, SCREEN_HEIGHT);
        canvas.copy(&town_texture, src_rect, None)?;

        // 绘制 logo 在顶部中央
        let logo_x = (SCREEN_WIDTH as i32 - logo.0 as i32) / 2;
        let logo_rect = Rect::new(logo_x, 10, logo.0, logo.1);
        canvas.copy(&logo_texture, None, logo_rect)?;

        // 显示信息
        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    println!("退出.");
    Ok(())
}

/// 加载调色板
fn load_palette(mpq: &mut devilutionx_rs::engine::mpq::MpqArchive) -> Result<[u8; 768], Box<dyn std::error::Error>> {
    // 尝试不同的调色板
    let pal_paths = [
        "levels\\towndata\\town.pal",
        "gendata\\cut2.pal",
        "gendata\\cut3.pal",
        "levels\\l1data\\l1.pal",
    ];

    for path in pal_paths {
        if let Ok(data) = mpq.read_file(path) {
            if data.len() >= 768 {
                let mut pal = [0u8; 768];
                pal.copy_from_slice(&data[..768]);
                return Ok(pal);
            }
        }
    }

    Err("找不到调色板文件".into())
}

/// 加载城镇背景图
fn load_town_image(
    mpq: &mut devilutionx_rs::engine::mpq::MpqArchive,
    palette: &[u8; 768]
) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    // 尝试加载城镇背景或其他大图
    let pcx_paths = [
        "gendata\\cuttt.cel",      // 城镇过场
        "gendata\\cut2.cel",       // 过场动画
        "ui_art\\mainmenu.pcx",    // 主菜单背景
        "ui_art\\title.pcx",       // 标题画面
    ];

    // 先尝试 PCX
    for path in ["ui_art\\mainmenu.pcx", "ctrlpan\\panel8.pcx", "data\\diabsmal.cel"] {
        if let Ok(data) = mpq.read_file(path) {
            if let Some(pcx) = devilutionx_rs::engine::pcx::PcxImage::decode(&data) {
                let rgba = pcx.to_rgba();
                return Ok((pcx.width as u32, pcx.height as u32, rgba));
            }
        }
    }

    // 尝试创建一个城镇地图的简单渲染
    // 使用地牢瓷砖数据
    create_dungeon_preview(mpq, palette)
}

/// 创建地牢预览图
fn create_dungeon_preview(
    mpq: &mut devilutionx_rs::engine::mpq::MpqArchive,
    palette: &[u8; 768]
) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    // 尝试加载 L1（大教堂）的瓷砖数据
    let cel_path = "levels\\l1data\\l1.cel";
    let min_path = "levels\\l1data\\l1.min";

    let cel_data = mpq.read_file(cel_path)?;
    let min_data = mpq.read_file(min_path)?;

    // 创建一个简单的瓷砖预览
    let preview_width = 640u32;
    let preview_height = 480u32;
    let mut rgba = vec![0u8; (preview_width * preview_height * 4) as usize];

    // 解析 MIN 数据获取瓷砖信息
    let blocks_per_tile = 10; // L1 uses 10 blocks per tile
    let tile_count = min_data.len() / (blocks_per_tile * 2);

    println!("    地牢瓷砖: {} 个", tile_count);
    println!("    CEL 数据: {} 字节", cel_data.len());

    // 渲染一些瓷砖作为预览
    let tiles_per_row = 16;
    let tile_size = 32;

    for tile_idx in 0..tile_count.min(256) {
        let tx = (tile_idx % tiles_per_row) as i32 * tile_size;
        let ty = (tile_idx / tiles_per_row) as i32 * tile_size;

        if tx + tile_size > preview_width as i32 || ty + tile_size > preview_height as i32 {
            continue;
        }

        // 读取瓷砖颜色（简化 - 只用第一个块的第一个像素）
        let offset = tile_idx * blocks_per_tile * 2;
        if offset + 2 <= min_data.len() {
            let block_ref = u16::from_le_bytes([min_data[offset], min_data[offset + 1]]);
            let frame_idx = (block_ref & 0x0FFF) as usize;

            // 获取颜色（简化）
            let color_idx = (frame_idx % 256) as u8;
            let r = palette[color_idx as usize * 3];
            let g = palette[color_idx as usize * 3 + 1];
            let b = palette[color_idx as usize * 3 + 2];

            // 填充瓷砖
            for py in 0..tile_size as i32 {
                for px in 0..tile_size as i32 {
                    let x = tx + px;
                    let y = ty + py;
                    if x >= 0 && x < preview_width as i32 && y >= 0 && y < preview_height as i32 {
                        let idx = ((y as u32 * preview_width + x as u32) * 4) as usize;
                        rgba[idx] = r;
                        rgba[idx + 1] = g;
                        rgba[idx + 2] = b;
                        rgba[idx + 3] = 255;
                    }
                }
            }
        }
    }

    Ok((preview_width, preview_height, rgba))
}

/// 加载 Diablo Logo
fn load_logo(
    mpq: &mut devilutionx_rs::engine::mpq::MpqArchive,
    palette: &[u8; 768]
) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    // 尝试加载 logo PCX
    let paths = [
        "ui_art\\smlogo.pcx",
        "ui_art\\logo.pcx",
        "data\\diabsmal.cel",
    ];

    for path in paths {
        if let Ok(data) = mpq.read_file(path) {
            // 尝试作为 PCX 解析
            if let Some(pcx) = devilutionx_rs::engine::pcx::PcxImage::decode(&data) {
                let rgba = pcx.to_rgba();
                return Ok((pcx.width as u32, pcx.height as u32, rgba));
            }
        }
    }

    // 如果找不到，创建一个占位符
    let w = 296u32;
    let h = 100u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];

    // 简单的 "DIABLO" 文字占位符
    // 这里只是填充一个简单的矩形
    for y in 20..80 {
        for x in 20..276 {
            let idx = ((y * w + x) * 4) as usize;
            rgba[idx] = 139; // 金色
            rgba[idx + 1] = 69;
            rgba[idx + 2] = 19;
            rgba[idx + 3] = 200;
        }
    }

    Ok((w, h, rgba))
}
