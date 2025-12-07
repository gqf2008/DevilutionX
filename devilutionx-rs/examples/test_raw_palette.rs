//! 测试原始 PCX 调色板数据

use devilutionx_rs::engine::mpq::MpqArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = MpqArchive::open("spawn.mpq")?;

    // 加载 title.pcx 原始数据
    let data = archive.read_file("ui_art\\title.pcx")?;
    println!("PCX file size: {} bytes", data.len());

    // PCX 调色板在文件末尾 769 字节处
    // 格式: 1 byte marker (0x0C) + 256 * 3 bytes (RGB)
    let palette_start = data.len() - 769;
    let marker = data[palette_start];
    println!("Palette marker: 0x{:02X} (should be 0x0C)", marker);

    if marker == 0x0C {
        println!("\nFirst 10 palette entries from raw data:");
        for i in 0..10 {
            let offset = palette_start + 1 + i * 3;
            let r = data[offset];
            let g = data[offset + 1];
            let b = data[offset + 2];
            println!("  Color {}: R={}, G={}, B={}", i, r, g, b);
        }

        // 检查中间和末尾的颜色
        println!("\nMiddle palette entries (128-137):");
        for i in 128..138 {
            let offset = palette_start + 1 + i * 3;
            let r = data[offset];
            let g = data[offset + 1];
            let b = data[offset + 2];
            println!("  Color {}: R={}, G={}, B={}", i, r, g, b);
        }

        // 计算非零颜色数量
        let mut non_black = 0;
        let mut has_green = 0;
        let mut has_blue = 0;
        for i in 0..256 {
            let offset = palette_start + 1 + i * 3;
            let r = data[offset];
            let g = data[offset + 1];
            let b = data[offset + 2];
            if r > 0 || g > 0 || b > 0 {
                non_black += 1;
            }
            if g > 0 {
                has_green += 1;
            }
            if b > 0 {
                has_blue += 1;
            }
        }
        println!("\nPalette stats:");
        println!("  Non-black colors: {}", non_black);
        println!("  Colors with green > 0: {}", has_green);
        println!("  Colors with blue > 0: {}", has_blue);

        // 随机打印一些有颜色的条目
        println!("\nSample of colored entries:");
        let mut count = 0;
        for i in 0..256 {
            let offset = palette_start + 1 + i * 3;
            let r = data[offset];
            let g = data[offset + 1];
            let b = data[offset + 2];
            if g > 10 || b > 10 {
                println!("  Color {}: R={}, G={}, B={}", i, r, g, b);
                count += 1;
                if count >= 10 {
                    break;
                }
            }
        }
    } else {
        println!("ERROR: Invalid palette marker!");
    }

    Ok(())
}
