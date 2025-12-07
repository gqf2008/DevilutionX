//! 查找所有mpq中的字体文件

use devilutionx_rs::engine::mpq::MpqArchive;

fn main() {
    let mpqs = ["devilutionx.mpq", "fonts.mpq", "spawn.mpq"];

    for mpq_path in &mpqs {
        println!("\n========================================");
        println!("检查 {}:", mpq_path);
        println!("========================================");

        match MpqArchive::open(mpq_path) {
            Ok(mut archive) => {
                // 测试基础字体
                let sizes = [12, 24, 30, 42, 46, 22];
                for size in sizes {
                    let path = format!("fonts\\{}-00.clx", size);
                    if archive.has_file(&path) {
                        if let Ok(data) = archive.read_file(&path) {
                            println!("✓ {} ({} bytes)", path, data.len());
                        }
                    }
                }

                // 测试TRN颜色转换文件
                let trns = [
                    "fonts\\goldui.trn",
                    "fonts\\grayui.trn",
                    "fonts\\white.trn",
                    "fonts\\black.trn",
                ];
                for trn in &trns {
                    if archive.has_file(trn) {
                        if let Ok(data) = archive.read_file(trn) {
                            println!("✓ {} ({} bytes)", trn, data.len());
                        }
                    }
                }

                // 测试UI字体
                let ui_fonts = [
                    "ui_art\\font16.bin",
                    "ui_art\\font24.bin",
                    "ui_art\\font30.bin",
                    "ui_art\\font42.bin",
                ];
                for font in &ui_fonts {
                    if archive.has_file(font) {
                        if let Ok(data) = archive.read_file(font) {
                            println!("✓ {} ({} bytes)", font, data.len());
                        }
                    }
                }
            }
            Err(e) => {
                println!("无法打开: {:?}", e);
            }
        }
    }
}
