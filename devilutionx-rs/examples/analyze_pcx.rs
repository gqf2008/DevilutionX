//! 详细分析 PCX 图像内容

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = MpqArchive::open("spawn.mpq")?;

    // 加载 title.pcx
    let title_data = archive.read_file("ui_art\\title.pcx")?;
    let title_pcx = PcxImage::decode(&title_data).ok_or("Failed to decode")?;

    println!("Image: {}x{}", title_pcx.width, title_pcx.height);
    println!("Total pixels: {}", title_pcx.pixels.len());

    // 统计像素索引使用情况
    let mut index_counts = [0u32; 256];
    for &pixel in &title_pcx.pixels {
        index_counts[pixel as usize] += 1;
    }

    // 找出最常用的 20 个索引
    let mut usage: Vec<(u8, u32)> = index_counts.iter().enumerate()
        .map(|(i, &c)| (i as u8, c))
        .filter(|(_, c)| *c > 0)
        .collect();
    usage.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nTop 20 most used pixel indices and their palette colors:");
    for (idx, count) in usage.iter().take(20) {
        let color = &title_pcx.palette[*idx as usize];
        let pct = (*count as f64 / title_pcx.pixels.len() as f64) * 100.0;
        println!("  Index {:3}: {:6} pixels ({:5.2}%) -> RGB({:3}, {:3}, {:3})",
                 idx, count, pct, color.r, color.g, color.b);
    }

    // 检查特定区域的像素
    println!("\n\nSampling pixels at various locations:");
    let samples = [
        (100, 100), (200, 100), (300, 100), (400, 100), (500, 100),
        (100, 200), (320, 240), (500, 300),
        (100, 400), (320, 400), (500, 400),
    ];

    for (x, y) in samples {
        let idx = (y * title_pcx.width as usize + x);
        if idx < title_pcx.pixels.len() {
            let pixel_idx = title_pcx.pixels[idx];
            let color = &title_pcx.palette[pixel_idx as usize];
            println!("  ({:3}, {:3}): index={:3} -> RGB({:3}, {:3}, {:3})",
                     x, y, pixel_idx, color.r, color.g, color.b);
        }
    }

    // 检查原始数据的调色板位置
    println!("\n\nRaw palette check for commonly used indices:");
    let palette_start = title_data.len() - 769;
    for (idx, _) in usage.iter().take(10) {
        let offset = palette_start + 1 + (*idx as usize) * 3;
        let r = title_data[offset];
        let g = title_data[offset + 1];
        let b = title_data[offset + 2];
        let parsed_color = &title_pcx.palette[*idx as usize];
        println!("  Index {:3}: raw=RGB({:3}, {:3}, {:3}), parsed=RGB({:3}, {:3}, {:3})",
                 idx, r, g, b, parsed_color.r, parsed_color.g, parsed_color.b);
    }

    Ok(())
}
