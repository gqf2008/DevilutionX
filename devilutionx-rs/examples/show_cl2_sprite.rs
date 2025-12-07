//! CL2 Sprite Viewer Example
//!
//! This example loads and displays CL2 sprite animations from a Diablo MPQ archive.
//! Usage: cargo run --example show_cl2_sprite

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find DIABDAT.MPQ - check command line arg first, then environment, then common paths
    let mpq_path = std::env::args().nth(1)
        .or_else(|| std::env::var("DIABDAT_MPQ").ok());

    let mpq_paths: Vec<String> = if let Some(path) = mpq_path {
        vec![path]
    } else {
        vec![
            "DIABDAT.MPQ".to_string(),
            "diabdat.mpq".to_string(),
            "../DIABDAT.MPQ".to_string(),
            "../../DIABDAT.MPQ".to_string(),
            "D:\\Games\\Diablo\\DIABDAT.MPQ".to_string(),
            "C:\\Games\\Diablo\\DIABDAT.MPQ".to_string(),
            "E:\\DIABDAT.MPQ".to_string(),
            "D:\\DIABDAT.MPQ".to_string(),
        ]
    };

    let mut mpq = None;
    for path in &mpq_paths {
        if let Ok(archive) = devilutionx_rs::engine::mpq::MpqArchive::open(path) {
            println!("✓ MPQ 打开成功: {}", path);
            mpq = Some(archive);
            break;
        }
    }

    let mut mpq = mpq.ok_or("找不到 DIABDAT.MPQ，请指定路径: cargo run --example show_cl2_sprite <path_to_diabdat.mpq>")?;

    // Load a palette
    let pal_data = mpq.read_file("levels\\towndata\\town.pal")
        .or_else(|_| mpq.read_file("levels\\l1data\\l1.pal"))?;

    let palette = devilutionx_rs::engine::dungeon::PaletteData::from_bytes(&pal_data)
        .ok_or("调色板解析失败")?;

    // Convert to flat palette array
    let mut pal_flat = [0u8; 768];
    pal_flat.copy_from_slice(&palette.colors);

    println!("✓ 调色板加载成功");

    // Try to load a CL2 sprite
    let cl2_files = [
        "plrgfx\\warrior\\wla\\wlaat.cl2",  // Warrior attack animation
        "plrgfx\\warrior\\wla\\wlast.cl2",  // Warrior stand animation
        "plrgfx\\rogue\\rla\\rlast.cl2",    // Rogue stand animation
        "monsters\\fallen\\fallaw.cl2",     // Fallen attack animation
        "data\\inv\\objcurs.cel",            // Object cursors (CEL)
    ];

    let mut sprite_data = None;
    let mut sprite_name = "";

    for path in &cl2_files {
        match mpq.read_file(path) {
            Ok(data) => {
                println!("✓ 精灵加载成功: {} ({} 字节)", path, data.len());
                sprite_data = Some(data);
                sprite_name = path;
                break;
            }
            Err(e) => {
                println!("  跳过 {}: {}", path, e);
            }
        }
    }

    let sprite_data = sprite_data.ok_or("找不到任何精灵文件")?;

    // Parse as CL2 sprite list
    let sprites = parse_cl2_sprite(&sprite_data)?;

    println!("✓ 解析到 {} 帧", sprites.len());

    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(&format!("CL2 Viewer: {}", sprite_name), 800, 600)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // Animation state
    let mut current_frame = 0;
    let frame_duration = Duration::from_millis(150);
    let mut last_frame_time = Instant::now();
    let mut paused = false;

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    paused = !paused;
                    println!("动画: {}", if paused { "暂停" } else { "播放" });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if current_frame > 0 {
                        current_frame -= 1;
                    } else {
                        current_frame = sprites.len() - 1;
                    }
                    println!("帧: {}/{}", current_frame + 1, sprites.len());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    current_frame = (current_frame + 1) % sprites.len();
                    println!("帧: {}/{}", current_frame + 1, sprites.len());
                }
                _ => {}
            }
        }

        // Update animation
        if !paused && last_frame_time.elapsed() >= frame_duration {
            current_frame = (current_frame + 1) % sprites.len();
            last_frame_time = Instant::now();
        }

        // Clear screen
        canvas.set_draw_color(sdl2::pixels::Color::RGB(32, 32, 48));
        canvas.clear();

        // Render current frame
        if let Some((width, height, rgba_data)) = &sprites.get(current_frame) {
            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::ABGR8888, *width as u32, *height as u32)?;

            texture.update(None, rgba_data, *width * 4)?;

            // Center the sprite
            let x = (800 - *width as i32) / 2;
            let y = (600 - *height as i32) / 2;

            canvas.copy(
                &texture,
                None,
                sdl2::rect::Rect::new(x, y, *width as u32, *height as u32),
            )?;
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

/// Parse CL2 data into a vector of (width, height, rgba_data) frames
fn parse_cl2_sprite(data: &[u8]) -> Result<Vec<(usize, usize, Vec<u8>)>, Box<dyn std::error::Error>> {
    if data.len() < 4 {
        return Err("数据太短".into());
    }

    let num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    // Sanity check
    if num_frames == 0 || num_frames > 1000 {
        return Err(format!("无效帧数: {}", num_frames).into());
    }

    if data.len() < 4 + (num_frames + 1) * 4 {
        return Err("帧偏移表不完整".into());
    }

    let mut frames = Vec::with_capacity(num_frames);

    // Create a grayscale palette as fallback
    let mut palette = [0u8; 768];
    for i in 0..256 {
        palette[i * 3] = i as u8;
        palette[i * 3 + 1] = i as u8;
        palette[i * 3 + 2] = i as u8;
    }

    for i in 0..num_frames {
        let offset_idx = 4 + i * 4;
        let begin = u32::from_le_bytes([
            data[offset_idx],
            data[offset_idx + 1],
            data[offset_idx + 2],
            data[offset_idx + 3],
        ]) as usize;

        let end = u32::from_le_bytes([
            data[offset_idx + 4],
            data[offset_idx + 5],
            data[offset_idx + 6],
            data[offset_idx + 7],
        ]) as usize;

        if begin >= data.len() || end > data.len() || begin >= end {
            println!("  跳过帧 {}: 无效偏移 {}-{}", i, begin, end);
            continue;
        }

        let frame_data = &data[begin..end];

        // Parse CL2 frame header
        if frame_data.len() < 10 {
            continue;
        }

        let header_size = u16::from_le_bytes([frame_data[0], frame_data[1]]) as usize;

        // CL2 frames have 10-byte headers with block offsets
        // We need to estimate width and height from the data
        // Typical Diablo sprite sizes: 96x96, 128x128, etc.
        let width = 96;  // Default estimate
        let height = estimate_height(frame_data, header_size, width);

        let pixel_data = if header_size < frame_data.len() {
            &frame_data[header_size..]
        } else {
            &frame_data[10..]
        };

        // Decode CL2 to RGBA
        let rgba = decode_cl2_frame(pixel_data, width, height, &palette);

        frames.push((width, height, rgba));
    }

    if frames.is_empty() {
        Err("无法解析任何帧".into())
    } else {
        Ok(frames)
    }
}

fn estimate_height(frame_data: &[u8], header_size: usize, width: usize) -> usize {
    // Count total pixels by simulating decode
    let pixel_data = if header_size < frame_data.len() {
        &frame_data[header_size..]
    } else if frame_data.len() > 10 {
        &frame_data[10..]
    } else {
        return 96;
    };

    let mut total_pixels = 0;
    let mut src_idx = 0;

    while src_idx < pixel_data.len() {
        let control = pixel_data[src_idx];
        src_idx += 1;

        if control < 0x80 {
            // Transparent: skip N pixels
            total_pixels += control as usize;
        } else if control <= 0xBE {
            // Fill: fill (0xBF - control) pixels
            let w = (0xBF - control) as usize;
            total_pixels += w;
            src_idx += 1; // skip color byte
        } else {
            // Copy: copy (256 - control) pixels
            let w = (256 - control as usize);
            total_pixels += w;
            src_idx += w;
        }

        if src_idx >= pixel_data.len() {
            break;
        }
    }

    // Estimate height
    let height = (total_pixels + width - 1) / width;
    height.max(1).min(256)
}

fn decode_cl2_frame(pixel_data: &[u8], width: usize, height: usize, palette: &[u8; 768]) -> Vec<u8> {
    let mut rgba = vec![0u8; width * height * 4];

    let mut src_idx = 0;
    let mut x = 0;
    let mut y = height.saturating_sub(1);  // Start from bottom

    while src_idx < pixel_data.len() {
        let control = pixel_data[src_idx];
        src_idx += 1;

        if control < 0x80 {
            // Transparent: skip N pixels
            let skip = control as usize;
            for _ in 0..skip {
                // Leave as transparent (already 0)
                x += 1;
                if x >= width {
                    x = 0;
                    if y == 0 { return rgba; }
                    y -= 1;
                }
            }
        } else if control <= 0xBE {
            // Fill: fill (0xBF - control) pixels with next byte
            let w = (0xBF - control) as usize;
            if src_idx >= pixel_data.len() { break; }
            let color_idx = pixel_data[src_idx] as usize;
            src_idx += 1;

            for _ in 0..w {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    if idx + 3 < rgba.len() {
                        rgba[idx] = palette[color_idx * 3];
                        rgba[idx + 1] = palette[color_idx * 3 + 1];
                        rgba[idx + 2] = palette[color_idx * 3 + 2];
                        rgba[idx + 3] = 255;
                    }
                }
                x += 1;
                if x >= width {
                    x = 0;
                    if y == 0 { return rgba; }
                    y -= 1;
                }
            }
        } else {
            // Copy: copy (256 - control) pixels from stream
            let w = (256 - control as usize);
            for _ in 0..w {
                if src_idx >= pixel_data.len() { break; }
                let color_idx = pixel_data[src_idx] as usize;
                src_idx += 1;

                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    if idx + 3 < rgba.len() {
                        rgba[idx] = palette[color_idx * 3];
                        rgba[idx + 1] = palette[color_idx * 3 + 1];
                        rgba[idx + 2] = palette[color_idx * 3 + 2];
                        rgba[idx + 3] = 255;
                    }
                }
                x += 1;
                if x >= width {
                    x = 0;
                    if y == 0 { return rgba; }
                    y -= 1;
                }
            }
        }
    }

    rgba
}
