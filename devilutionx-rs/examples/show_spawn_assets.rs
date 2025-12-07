//! 展示从 spawn.mpq 加载的真实游戏资源
//! 这个演示加载并显示真实的 Diablo 资源

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 打开 spawn.mpq
    let mpq_paths = [
        "spawn.mpq",
        "../spawn.mpq",
        "../../spawn.mpq",
        "d:/Users/gxh/Documents/GitHub/DevilutionX/spawn.mpq",
    ];

    let mut archive = None;
    for path in &mpq_paths {
        if let Ok(a) = MpqArchive::open(path) {
            println!("Opened MPQ: {}", path);
            archive = Some(a);
            break;
        }
    }

    let mut archive = archive.ok_or("Could not find spawn.mpq")?;

    // 加载标题屏幕 PCX
    let title_data = archive.read_file("ui_art\\title.pcx")?;
    let title_pcx = PcxImage::decode(&title_data).ok_or("Failed to decode title.pcx")?;
    println!("Loaded title.pcx: {}x{}", title_pcx.width, title_pcx.height);

    // 加载主菜单 PCX
    let mainmenu_data = archive.read_file("ui_art\\mainmenu.pcx")?;
    let mainmenu_pcx = PcxImage::decode(&mainmenu_data).ok_or("Failed to decode mainmenu.pcx")?;
    println!("Loaded mainmenu.pcx: {}x{}", mainmenu_pcx.width, mainmenu_pcx.height);

    // 加载小 Logo PCX
    let smlogo_data = archive.read_file("ui_art\\smlogo.pcx")?;
    let smlogo_pcx = PcxImage::decode(&smlogo_data).ok_or("Failed to decode smlogo.pcx")?;
    println!("Loaded smlogo.pcx: {}x{}", smlogo_pcx.width, smlogo_pcx.height);

    // 初始化 SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("DevilutionX-RS - Real Assets Demo", 800, 600)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    // 转换 PCX 到 RGBA 并创建纹理
    let title_rgba = title_pcx.to_rgba();
    let mainmenu_rgba = mainmenu_pcx.to_rgba();
    let smlogo_rgba = smlogo_pcx.to_rgba();

    // 创建标题纹理
    let mut title_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA8888,
        title_pcx.width,
        title_pcx.height,
    )?;

    title_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..title_pcx.height as usize {
            for x in 0..title_pcx.width as usize {
                let src_idx = (y * title_pcx.width as usize + x) * 4;
                let dst_idx = y * pitch + x * 4;

                if src_idx + 3 < title_rgba.len() && dst_idx + 3 < buffer.len() {
                    // RGBA -> RGBA (SDL RGBA8888 format)
                    buffer[dst_idx] = title_rgba[src_idx];     // R
                    buffer[dst_idx + 1] = title_rgba[src_idx + 1]; // G
                    buffer[dst_idx + 2] = title_rgba[src_idx + 2]; // B
                    buffer[dst_idx + 3] = title_rgba[src_idx + 3]; // A
                }
            }
        }
    })?;

    // 创建主菜单纹理
    let mut mainmenu_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA8888,
        mainmenu_pcx.width,
        mainmenu_pcx.height,
    )?;

    mainmenu_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..mainmenu_pcx.height as usize {
            for x in 0..mainmenu_pcx.width as usize {
                let src_idx = (y * mainmenu_pcx.width as usize + x) * 4;
                let dst_idx = y * pitch + x * 4;

                if src_idx + 3 < mainmenu_rgba.len() && dst_idx + 3 < buffer.len() {
                    buffer[dst_idx] = mainmenu_rgba[src_idx];
                    buffer[dst_idx + 1] = mainmenu_rgba[src_idx + 1];
                    buffer[dst_idx + 2] = mainmenu_rgba[src_idx + 2];
                    buffer[dst_idx + 3] = mainmenu_rgba[src_idx + 3];
                }
            }
        }
    })?;

    // 创建小 logo 纹理
    let mut smlogo_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA8888,
        smlogo_pcx.width,
        smlogo_pcx.height,
    )?;

    smlogo_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..smlogo_pcx.height as usize {
            for x in 0..smlogo_pcx.width as usize {
                let src_idx = (y * smlogo_pcx.width as usize + x) * 4;
                let dst_idx = y * pitch + x * 4;

                if src_idx + 3 < smlogo_rgba.len() && dst_idx + 3 < buffer.len() {
                    buffer[dst_idx] = smlogo_rgba[src_idx];
                    buffer[dst_idx + 1] = smlogo_rgba[src_idx + 1];
                    buffer[dst_idx + 2] = smlogo_rgba[src_idx + 2];
                    buffer[dst_idx + 3] = smlogo_rgba[src_idx + 3];
                }
            }
        }
    })?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut current_view = 0; // 0: title, 1: mainmenu, 2: smlogo

    println!("\nControls:");
    println!("  1 - Show title.pcx (Main title screen)");
    println!("  2 - Show mainmenu.pcx (Menu background)");
    println!("  3 - Show smlogo.pcx (Small Diablo logo)");
    println!("  ESC - Quit");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(key), .. } => {
                    match key {
                        Keycode::Escape => break 'running,
                        Keycode::Num1 => current_view = 0,
                        Keycode::Num2 => current_view = 1,
                        Keycode::Num3 => current_view = 2,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        match current_view {
            0 => {
                // 显示 title.pcx
                let (w, h) = (title_pcx.width, title_pcx.height);
                let scale = (800.0 / w as f32).min(600.0 / h as f32);
                let dst_w = (w as f32 * scale) as u32;
                let dst_h = (h as f32 * scale) as u32;
                let dst_x = (800 - dst_w) / 2;
                let dst_y = (600 - dst_h) / 2;

                canvas.copy(
                    &title_texture,
                    None,
                    Rect::new(dst_x as i32, dst_y as i32, dst_w, dst_h),
                )?;
            }
            1 => {
                // 显示 mainmenu.pcx
                let (w, h) = (mainmenu_pcx.width, mainmenu_pcx.height);
                let scale = (800.0 / w as f32).min(600.0 / h as f32);
                let dst_w = (w as f32 * scale) as u32;
                let dst_h = (h as f32 * scale) as u32;
                let dst_x = (800 - dst_w) / 2;
                let dst_y = (600 - dst_h) / 2;

                canvas.copy(
                    &mainmenu_texture,
                    None,
                    Rect::new(dst_x as i32, dst_y as i32, dst_w, dst_h),
                )?;
            }
            2 => {
                // 显示 smlogo.pcx
                let (w, h) = (smlogo_pcx.width, smlogo_pcx.height);

                // 按比例缩放显示
                let scale = (800.0 / w as f32).min(600.0 / h as f32).min(1.0);
                let dst_w = (w as f32 * scale) as u32;
                let dst_h = (h as f32 * scale) as u32;
                let dst_x = (800 - dst_w) / 2;
                let dst_y = (600 - dst_h) / 2;

                canvas.copy(
                    &smlogo_texture,
                    None,
                    Rect::new(dst_x as i32, dst_y as i32, dst_w, dst_h),
                )?;
            }
            _ => {}
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
