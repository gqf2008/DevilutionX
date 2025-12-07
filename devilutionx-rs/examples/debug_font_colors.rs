//! 调试字体颜色问题
//! 分析CLX字体数据、TRN转换表和调色板

use devilutionx_rs::engine::clx::ClxSpriteList;
use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 字体颜色调试工具 ===\n");

    // 打开MPQ文件
    let mut devilutionx = MpqArchive::open("devilutionx.mpq")?;
    let mut spawn = MpqArchive::open("spawn.mpq")?;

    // 1. 加载调色板
    println!("1. 分析调色板");
    let title_pcx = spawn.read_file("ui_art\\title.pcx")?;
    let title_img = PcxImage::decode(&title_pcx).ok_or("Failed to decode title.pcx")?;

    println!("  title.pcx 调色板 (前32色):");
    for i in 0..32 {
        let c = &title_img.palette[i];
        print!("  {:3}: RGB({:3},{:3},{:3})", i, c.r, c.g, c.b);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }

    println!("\n  调色板 (索引192-255 - 通常是文字颜色区域):");
    for i in 192..256 {
        let c = &title_img.palette[i];
        if i % 8 == 0 {
            print!("  {:3}: ", i);
        }
        // 判断颜色类型
        let color_type = if c.r > 200 && c.g > 200 && c.b > 200 {
            "W" // 白色
        } else if c.r > c.g && c.r > c.b {
            "R" // 红色
        } else if c.g > c.r && c.g > c.b {
            "G" // 绿色
        } else if c.b > c.r && c.b > c.g {
            "B" // 蓝色
        } else if c.r > 150 && c.g > 100 && c.b < 100 {
            "O" // 橙/金色
        } else if c.r < 50 && c.g < 50 && c.b < 50 {
            "K" // 黑色
        } else {
            "?" // 其他
        };
        print!("{} ", color_type);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }

    // 2. 加载TRN文件
    println!("\n2. 分析TRN颜色转换表");

    let trn_files = [
        "fonts\\goldui.trn",
        "fonts\\white.trn",
        "fonts\\yellow.trn",
    ];

    for trn_path in &trn_files {
        if let Ok(trn_data) = devilutionx.read_file(trn_path) {
            println!("\n  {}:", trn_path);
            println!("  文件大小: {} 字节", trn_data.len());

            // 统计非恒等映射
            let mut changes = Vec::new();
            for i in 0..256.min(trn_data.len()) {
                if trn_data[i] as usize != i {
                    changes.push((i, trn_data[i]));
                }
            }

            println!("  非恒等映射数量: {}", changes.len());
            println!("  示例映射 (源 -> 目标):");
            for (src, dst) in changes.iter() {
                let src_color = &title_img.palette[*src];
                let dst_color = &title_img.palette[*dst as usize];
                println!(
                    "    {:3} -> {:3}  (RGB({:3},{:3},{:3}) -> RGB({:3},{:3},{:3}))",
                    src, dst, src_color.r, src_color.g, src_color.b, dst_color.r, dst_color.g, dst_color.b
                );
            }
        }
    }

    // 3. 分析CLX字体数据
    println!("\n3. 分析CLX字体数据");

    if let Ok(font_data) = devilutionx.read_file("fonts\\24-00.clx") {
        println!("  fonts\\24-00.clx:");
        println!("  文件大小: {} 字节", font_data.len());

        if let Some(font_list) = ClxSpriteList::from_bytes(font_data) {
            println!("  精灵数量: {}", font_list.len());

            // 分析几个字符的像素值
            let test_chars = ['A', 'a', '0', '!'];
            for ch in test_chars {
                let idx = ch as usize;
                if let Some(sprite) = font_list.get(idx) {
                    // 收集像素颜色索引
                    let mut color_counts: HashMap<u8, usize> = HashMap::new();
                    let mut src_idx = 0;
                    let data = &sprite.pixel_data;

                    while src_idx < data.len() {
                        let control = data[src_idx];
                        src_idx += 1;

                        if control < 0x80 {
                            // 透明
                        } else if control <= 0xBE {
                            // 填充
                            let width = (0xBF - control) as usize;
                            if src_idx < data.len() {
                                let color = data[src_idx];
                                *color_counts.entry(color).or_insert(0) += width;
                                src_idx += 1;
                            }
                        } else {
                            // 复制
                            let width = (256 - control as usize) as usize;
                            for _ in 0..width {
                                if src_idx < data.len() {
                                    let color = data[src_idx];
                                    *color_counts.entry(color).or_insert(0) += 1;
                                    src_idx += 1;
                                }
                            }
                        }
                    }

                    println!(
                        "\n  字符 '{}' (索引 {}): {}x{} 像素",
                        ch, idx, sprite.width, sprite.height
                    );
                    println!("    使用的颜色索引:");
                    let mut colors: Vec<_> = color_counts.iter().collect();
                    colors.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
                    for (color_idx, count) in colors.iter().take(10) {
                        let c = &title_img.palette[**color_idx as usize];
                        println!(
                            "      索引 {:3}: {:5} 像素, RGB({:3},{:3},{:3})",
                            color_idx, count, c.r, c.g, c.b
                        );
                    }
                }
            }
        }
    }

    // 4. 检查字体是否使用特殊调色板
    println!("\n4. 检查字体专用调色板");

    // 尝试加载一些可能的字体调色板
    let palette_files = [
        "fonts\\palette.dat",
        "fonts\\font.pal",
        "gendata\\cut.pal",
        "ui_art\\font.pal",
    ];

    for pal_path in &palette_files {
        if devilutionx.has_file(pal_path) {
            println!("  找到: {}", pal_path);
        } else if spawn.has_file(pal_path) {
            println!("  在spawn.mpq中找到: {}", pal_path);
        }
    }

    println!("\n调试完成!");
    Ok(())
}
