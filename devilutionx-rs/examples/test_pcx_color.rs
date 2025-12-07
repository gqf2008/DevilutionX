//! 简单测试 PCX 颜色解码

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = MpqArchive::open("spawn.mpq")?;

    // 加载 title.pcx
    let title_data = archive.read_file("ui_art\\title.pcx")?;
    let title_pcx = PcxImage::decode(&title_data).ok_or("Failed to decode")?;

    println!("Image: {}x{}", title_pcx.width, title_pcx.height);
    println!("Palette has {} colors", title_pcx.palette.len());

    // 打印前几个调色板颜色
    println!("\nFirst 10 palette colors:");
    for (i, color) in title_pcx.palette.iter().take(10).enumerate() {
        println!("  Color {}: R={}, G={}, B={}", i, color.r, color.g, color.b);
    }

    // 检查图像中某个像素
    if let Some(color) = title_pcx.get_pixel(320, 240) {
        println!("\nCenter pixel (320,240): R={}, G={}, B={}", color.r, color.g, color.b);
    }

    // 检查 to_rgba 输出
    let rgba = title_pcx.to_rgba();
    let center_idx = (240 * title_pcx.width as usize + 320) * 4;
    println!("\nto_rgba at center:");
    println!("  byte[0] (R?) = {}", rgba[center_idx]);
    println!("  byte[1] (G?) = {}", rgba[center_idx + 1]);
    println!("  byte[2] (B?) = {}", rgba[center_idx + 2]);
    println!("  byte[3] (A?) = {}", rgba[center_idx + 3]);

    // 统计一下图像中使用最多的颜色索引
    let mut color_counts = [0u32; 256];
    for &pixel in &title_pcx.pixels {
        color_counts[pixel as usize] += 1;
    }

    let mut most_used: Vec<(usize, u32)> = color_counts.iter().enumerate()
        .map(|(i, &c)| (i, c))
        .filter(|(_, c)| *c > 0)
        .collect();
    most_used.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nMost used colors:");
    for (idx, count) in most_used.iter().take(5) {
        let c = &title_pcx.palette[*idx];
        println!("  Index {}: {} pixels, RGB({}, {}, {})", idx, count, c.r, c.g, c.b);
    }

    Ok(())
}
