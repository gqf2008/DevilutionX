//! 简单字体渲染测试

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;
use devilutionx_rs::engine::text_render::{GameFont, TextColor, TextRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 字体渲染测试 ===\n");

    // 打开MPQ文件
    let mut devilutionx = MpqArchive::open("devilutionx.mpq")?;
    let mut spawn = MpqArchive::open("spawn.mpq")?;

    // 加载调色板
    let title_pcx = spawn.read_file("ui_art\\title.pcx")?;
    let title_img = PcxImage::decode(&title_pcx).ok_or("Failed to decode title.pcx")?;
    let mut palette = [0u8; 768];
    for i in 0..256 {
        let c = &title_img.palette[i];
        palette[i * 3] = c.r;
        palette[i * 3 + 1] = c.g;
        palette[i * 3 + 2] = c.b;
    }

    // 创建渲染器
    let mut text_renderer = TextRenderer::new();
    text_renderer.set_palette(&palette);

    // 加载字体
    text_renderer.load_font(&mut devilutionx, GameFont::Font24, &[0x00]);

    // 加载颜色转换表
    let gold_loaded = text_renderer.load_color_translation(&mut devilutionx, TextColor::UiGold);
    let white_loaded = text_renderer.load_color_translation(&mut devilutionx, TextColor::White);
    println!("UiGold TRN loaded: {}", gold_loaded);
    println!("White TRN loaded: {}", white_loaded);

    // 创建小缓冲区渲染单个字符
    let width = 50;
    let height = 50;
    let mut buffer = vec![0u8; width * height * 4];

    // 渲染 'A' 字符（无颜色转换）
    println!("\n渲染 'A' 无颜色转换:");
    text_renderer.render_text(&mut buffer, width, height, "A", 10, 10, GameFont::Font24, None, 0);
    print_non_black_pixels(&buffer, width, height, &palette, 5);

    // 清除并重新渲染（金色）
    buffer.fill(0);
    println!("\n渲染 'A' 金色:");
    text_renderer.render_text(&mut buffer, width, height, "A", 10, 10, GameFont::Font24, Some(TextColor::UiGold), 0);
    print_non_black_pixels(&buffer, width, height, &palette, 5);

    // 清除并重新渲染（白色）
    buffer.fill(0);
    println!("\n渲染 'A' 白色:");
    text_renderer.render_text(&mut buffer, width, height, "A", 10, 10, GameFont::Font24, Some(TextColor::White), 0);
    print_non_black_pixels(&buffer, width, height, &palette, 5);

    println!("\n测试完成!");
    Ok(())
}

fn print_non_black_pixels(buffer: &[u8], width: usize, height: usize, _palette: &[u8; 768], max_count: usize) {
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let r = buffer[idx];
            let g = buffer[idx + 1];
            let b = buffer[idx + 2];
            let a = buffer[idx + 3];

            if a > 0 && (r > 10 || g > 10 || b > 10) && count < max_count {
                println!("  像素 ({:2},{:2}): RGBA({:3},{:3},{:3},{:3})", x, y, r, g, b, a);
                count += 1;
            }
        }
    }
    println!("  (显示前{}个非黑像素)", max_count);
}
