//! 测试CLX字体文件的解析

use devilutionx_rs::engine::clx::ClxSpriteList;
use devilutionx_rs::engine::mpq::MpqArchive;

fn main() {
    let mpq_path = "fonts.mpq";

    println!("Opening fonts.mpq...");

    match MpqArchive::open(mpq_path) {
        Ok(mut archive) => {
            println!("MPQ opened successfully!");

            // 加载一个字体CLX文件 - fonts\12-00.clx 是基本ASCII字体
            let font_path = "fonts\\12-00.clx";

            if archive.has_file(font_path) {
                println!("\n找到字体文件: {}", font_path);

                match archive.read_file(font_path) {
                    Ok(data) => {
                        println!("文件大小: {} 字节", data.len());

                        // 打印前64字节头部信息
                        println!("\n文件头部 (前64字节):");
                        for (i, chunk) in data[..64.min(data.len())].chunks(16).enumerate() {
                            print!("{:04x}: ", i * 16);
                            for b in chunk {
                                print!("{:02x} ", b);
                            }
                            println!();
                        }

                        // 解析CLX sprite list
                        if let Some(sprite_list) = ClxSpriteList::from_bytes(data.clone()) {
                            println!("\nCLX SpriteList 解析成功!");
                            println!("Sprite数量: {}", sprite_list.len());

                            // 打印几个字形的信息
                            println!("\n字形示例 (ASCII字符):");
                            let chars_to_show = [' ', '!', 'A', 'a', '0'];
                            for ch in chars_to_show {
                                let idx = ch as usize;
                                if idx < sprite_list.len() {
                                    let sprite = &sprite_list[idx];
                                    println!("  '{}' (0x{:02x}): {}x{} pixels, {} bytes data",
                                        ch, idx, sprite.width, sprite.height, sprite.pixel_data.len());
                                }
                            }
                        } else {
                            println!("CLX解析失败!");
                        }
                    }
                    Err(e) => println!("读取失败: {:?}", e),
                }
            }

            // 测试加载更大字号的字体
            println!("\n\n测试不同字号的字体:");
            let sizes = [12, 24, 30, 42, 46];
            for size in sizes {
                let path = format!("fonts\\{}-00.clx", size);
                if archive.has_file(&path) {
                    if let Ok(data) = archive.read_file(&path) {
                        if let Some(list) = ClxSpriteList::from_bytes(data) {
                            let a_sprite = list.get('A' as usize);
                            if let Some(sprite) = a_sprite {
                                println!("  {}px 字体: 'A' = {}x{} pixels",
                                    size, sprite.width, sprite.height);
                            }
                        }
                    }
                } else {
                    println!("  {}px 字体: 未找到", size);
                }
            }

            // 测试中文字体
            println!("\n\n测试中文字体 (Unicode 0x4e00 区域 - 常用汉字):");
            let path = "fonts\\12-4e.clx";
            if archive.has_file(path) {
                if let Ok(data) = archive.read_file(path) {
                    println!("  文件大小: {} 字节", data.len());
                    if let Some(list) = ClxSpriteList::from_bytes(data) {
                        println!("  Sprite数量: {}", list.len());
                        // Unicode 0x4E00 = '一', 所以索引0对应'一'
                        // Unicode 0x4E2D = '中', 索引 0x2D = 45
                        if let Some(sprite) = list.get(0) {
                            println!("  '一' (0x4E00, idx=0): {}x{} pixels",
                                sprite.width, sprite.height);
                        }
                        if let Some(sprite) = list.get(0x2D) {
                            println!("  '中' (0x4E2D, idx=0x2D): {}x{} pixels",
                                sprite.width, sprite.height);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to open MPQ: {:?}", e);
        }
    }
}
