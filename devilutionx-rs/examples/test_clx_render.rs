//! 测试CLX字体渲染
//! 从 devilutionx.mpq 加载CLX字体并渲染文字

use devilutionx_rs::engine::clx::ClxSpriteList;
use devilutionx_rs::engine::mpq::MpqArchive;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::time::Duration;

/// CLX字体渲染器
struct ClxFont {
    /// 各Unicode行的字形 (row -> ClxSpriteList)
    glyphs: HashMap<u16, ClxSpriteList>,
    /// 字体大小
    size: u32,
}

impl ClxFont {
    /// 从MPQ加载字体
    fn load(archive: &mut MpqArchive, size: u32, rows: &[u16]) -> Option<Self> {
        let mut glyphs = HashMap::new();

        for &row in rows {
            let path = format!("fonts\\{}-{:02x}.clx", size, row);
            if archive.has_file(&path) {
                if let Ok(data) = archive.read_file(&path) {
                    if let Some(list) = ClxSpriteList::from_bytes(data) {
                        println!("已加载字体: {} ({} glyphs)", path, list.len());
                        glyphs.insert(row, list);
                    }
                }
            }
        }

        if glyphs.is_empty() {
            None
        } else {
            Some(Self { glyphs, size })
        }
    }

    /// 获取字符的字形
    fn get_glyph(&self, ch: char) -> Option<&devilutionx_rs::engine::clx::ClxSprite> {
        let code = ch as u32;
        let row = (code >> 8) as u16;
        let idx = (code & 0xFF) as usize;

        self.glyphs.get(&row).and_then(|list| list.get(idx))
    }

    /// 计算文本宽度
    fn text_width(&self, text: &str) -> u32 {
        let mut width = 0u32;
        for ch in text.chars() {
            if let Some(glyph) = self.get_glyph(ch) {
                width += glyph.width as u32;
            } else {
                width += self.size / 2; // 默认宽度
            }
        }
        width
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("CLX Font Test", 800, 600)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // 打开devilutionx.mpq
    println!("打开 devilutionx.mpq...");
    let mut archive = MpqArchive::open("devilutionx.mpq")?;

    // 加载字体 (基本ASCII + 常用汉字)
    println!("加载CLX字体...");
    let rows = vec![0x00, 0x4e, 0x4f, 0x50]; // ASCII + 部分CJK
    let font = ClxFont::load(&mut archive, 24, &rows)
        .ok_or("无法加载字体")?;

    // 加载颜色转换表 (TRN)
    let trn_data = if let Ok(data) = archive.read_file("fonts\\white.trn") {
        println!("已加载颜色转换表");
        Some(data)
    } else {
        None
    };

    // 创建一个调色板 (白色字体)
    let mut palette = [0u8; 768];
    for i in 0..256 {
        // 如果有TRN，使用它来转换颜色
        let color_idx = if let Some(ref trn) = trn_data {
            trn[i] as usize
        } else {
            i
        };

        // 简单的灰度调色板
        palette[i * 3] = color_idx as u8;
        palette[i * 3 + 1] = color_idx as u8;
        palette[i * 3 + 2] = color_idx as u8;
    }

    // 创建一个大纹理来渲染所有文字
    let mut text_surface = vec![0u8; 800 * 600 * 4];

    // 渲染测试文字
    let test_texts = [
        ("Hello World!", 50, 50),
        ("ABCDEFGHIJKLM", 50, 100),
        ("NOPQRSTUVWXYZ", 50, 150),
        ("0123456789", 50, 200),
        ("!@#$%^&*()", 50, 250),
    ];

    for (text, x, y) in &test_texts {
        let mut cur_x = *x as i32;
        for ch in text.chars() {
            if let Some(glyph) = font.get_glyph(ch) {
                // 渲染字形到表面
                glyph.render_to_rgba(
                    &mut text_surface,
                    800,
                    cur_x,
                    *y as i32,
                    &palette,
                );
                cur_x += glyph.width as i32;
            } else {
                cur_x += 12; // 空格宽度
            }
        }
    }

    // 创建SDL纹理
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::ABGR8888,
        800,
        600,
    )?;

    texture.update(None, &text_surface, 800 * 4)?;

    // 主循环
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(32, 32, 64));
        canvas.clear();

        canvas.copy(&texture, None, Some(Rect::new(0, 0, 800, 600)))?;

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
