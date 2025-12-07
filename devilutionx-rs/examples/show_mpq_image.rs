//! Display images from MPQ files
//!
//! Run with: cargo run --example show_mpq_image

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DevilutionX-RS MPQ 图像查看器 ===\n");

    // 打开 MPQ
    let mut archive = MpqArchive::open(".\\spawn.mpq")?;
    println!("✓ MPQ 打开成功!");

    // 加载 title.pcx
    let title_data = archive.read_file("ui_art\\title.pcx")?;
    println!("✓ 加载 ui_art\\title.pcx ({} 字节)", title_data.len());

    // 解码 PCX
    let title_image = PcxImage::decode(&title_data)
        .ok_or("Failed to decode PCX")?;
    println!("✓ PCX 解码成功: {}x{}", title_image.width, title_image.height);

    // 加载 logo.pcx
    let logo_data = archive.read_file("ui_art\\logo.pcx")?;
    let logo_image = PcxImage::decode(&logo_data)
        .ok_or("Failed to decode logo PCX")?;
    println!("✓ 加载 logo: {}x{}", logo_image.width, logo_image.height);

    // 初始化 SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // 创建窗口
    let window = video_subsystem
        .window("DevilutionX-RS - MPQ Image Viewer", 640, 480)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // 创建 title 纹理 (使用 ABGR8888 因为 SDL2 在小端系统上是这个顺序)
    let mut title_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::ABGR8888,
        title_image.width,
        title_image.height,
    )?;

    // 上传像素数据
    let rgba = title_image.to_rgba();
    title_texture.update(None, &rgba, (title_image.width * 4) as usize)?;

    // 创建 logo 纹理
    let mut logo_texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::ABGR8888,
        logo_image.width,
        logo_image.height,
    )?;
    let logo_rgba = logo_image.to_rgba();
    logo_texture.update(None, &logo_rgba, (logo_image.width * 4) as usize)?;

    println!("\n按 1 显示 title.pcx");
    println!("按 2 显示 logo.pcx");
    println!("按 ESC 或关闭窗口退出\n");

    let mut event_pump = sdl_context.event_pump()?;
    let mut show_logo = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    show_logo = false;
                    println!("显示: title.pcx");
                }
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    show_logo = true;
                    println!("显示: logo.pcx");
                }
                _ => {}
            }
        }

        // 清屏
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        // 显示图像
        if show_logo {
            // 居中显示 logo
            let scale = 1.0f32;
            let w = (logo_image.width as f32 * scale) as u32;
            let h = (logo_image.height as f32 * scale) as u32;
            let x = (640 - w as i32) / 2;
            let y = (480 - h as i32) / 2;
            canvas.copy(&logo_texture, None, Some(Rect::new(x, y, w, h)))?;
        } else {
            // 缩放显示 title (适应窗口)
            let scale_x = 640.0 / title_image.width as f32;
            let scale_y = 480.0 / title_image.height as f32;
            let scale = scale_x.min(scale_y);
            let w = (title_image.width as f32 * scale) as u32;
            let h = (title_image.height as f32 * scale) as u32;
            let x = (640 - w as i32) / 2;
            let y = (480 - h as i32) / 2;
            canvas.copy(&title_texture, None, Some(Rect::new(x, y, w, h)))?;
        }

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    println!("退出图像查看器");
    Ok(())
}
