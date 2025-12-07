//! 列出 fonts.mpq 文件中的所有文件

use devilutionx_rs::engine::mpq::MpqArchive;

fn main() {
    let mpq_path = "fonts.mpq";

    println!("Opening MPQ: {}", mpq_path);

    match MpqArchive::open(mpq_path) {
        Ok(mut archive) => {
            println!("MPQ opened successfully!");

            // 尝试各种可能的字体文件路径
            let test_files = [
                // CLX字体文件 - 按照C++代码中的路径格式 fonts\{size}-{row}.clx
                "fonts\\12-00.clx",
                "fonts\\24-00.clx",
                "fonts\\30-00.clx",
                "fonts\\42-00.clx",
                "fonts\\46-00.clx",
                "fonts\\22-00.clx",
                // 颜色转换表
                "fonts\\goldui.trn",
                "fonts\\grayui.trn",
                "fonts\\golduis.trn",
                "fonts\\grayuis.trn",
                "fonts\\yellow.trn",
                "fonts\\black.trn",
                "fonts\\white.trn",
                "fonts\\whitegold.trn",
                "fonts\\red.trn",
                "fonts\\blue.trn",
                "fonts\\orange.trn",
                "fonts\\buttonface.trn",
                "fonts\\buttonpushed.trn",
                "fonts\\gamedialogwhite.trn",
                "fonts\\gamedialogyellow.trn",
                "fonts\\gamedialogred.trn",
                // 旧版字体 (可能是PCX格式)
                "fonts\\12-00",
                "fonts\\24-00",
                "fonts\\30-00",
                "fonts\\42-00",
                // UI字体bin文件
                "ui_art\\font16.bin",
                "ui_art\\font16s.bin",
                "ui_art\\font24.bin",
                "ui_art\\font30.bin",
                "ui_art\\font42.bin",
                // 中文字体路径
                "fonts\\zh_CN\\12-00.clx",
                "fonts\\zh_CN\\24-00.clx",
                "fonts\\zh_CN\\30-00.clx",
                "fonts\\zh_CN\\42-00.clx",
                "fonts\\zh_TW\\12-00.clx",
                "fonts\\zh_TW\\24-00.clx",
                // CJK字体 (Unicode行 0x30-0x9f)
                "fonts\\12-30.clx",
                "fonts\\12-4e.clx", // 常用汉字起始
                "fonts\\24-4e.clx",
                "fonts\\30-4e.clx",
                "fonts\\42-4e.clx",
                // 韩文字体 (Unicode行 0xac-0xd7)
                "fonts\\12-ac.clx",
                "fonts\\24-ac.clx",
                // 其他可能的格式
                "fonts\\smaltext.cel",
                "fonts\\bigtext.cel",
                "fonts\\medtext.cel",
            ];

            println!("\nTesting font file paths:");
            println!("{:-<60}", "");

            let mut found_count = 0;
            for path in &test_files {
                if archive.has_file(path) {
                    match archive.read_file(path) {
                        Ok(data) => {
                            println!("✓ {} ({} bytes)", path, data.len());
                            found_count += 1;

                            // 打印前32字节作为参考
                            if data.len() >= 32 {
                                print!("  Header: ");
                                for b in &data[..32] {
                                    print!("{:02x} ", b);
                                }
                                println!();
                            }
                        }
                        Err(e) => {
                            println!("✗ {} (found but read error: {})", path, e);
                        }
                    }
                }
            }

            println!("{:-<60}", "");
            println!("Found {} / {} test files", found_count, test_files.len());

            // 暴力搜索 - 尝试更多可能的路径组合
            println!("\n尝试暴力搜索更多字体路径...");
            let sizes = [12, 16, 22, 24, 30, 42, 46];
            let rows: Vec<u8> = (0..=255).collect();
            let extensions = [".clx", "", ".pcx", ".cel"];

            let mut additional_found = 0;
            for size in &sizes {
                for row in &rows {
                    for ext in &extensions {
                        let path = format!("fonts\\{}-{:02x}{}", size, row, ext);
                        if archive.has_file(&path) {
                            if let Ok(data) = archive.read_file(&path) {
                                if additional_found < 20 { // 只显示前20个
                                    println!("✓ {} ({} bytes)", path, data.len());
                                }
                                additional_found += 1;
                            }
                        }
                    }
                }
            }
            println!("暴力搜索共找到 {} 个字体文件", additional_found);
        }
        Err(e) => {
            println!("Failed to open MPQ: {:?}", e);
        }
    }
}
