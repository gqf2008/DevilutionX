//! 列出 MPQ 文件中的所有文件

use devilutionx_rs::engine::mpq::MpqArchive;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mpq_path = args.get(1).map(|s| s.as_str()).unwrap_or("spawn.mpq");

    println!("Opening MPQ: {}", mpq_path);

    match MpqArchive::open(mpq_path) {
        Ok(mut archive) => {
            println!("MPQ opened successfully!");

            // 尝试一些常见的文件路径
            let test_files = [
                // 调色板
                "levels\\towndata\\town.pal",
                "levels\\l1data\\l1.pal",
                "levels\\l2data\\l2.pal",
                "levels\\l3data\\l3.pal",
                "levels\\l4data\\l4.pal",
                // Sprite 文件
                "plrgfx\\warrior\\wld\\wldas.clx",
                "plrgfx\\warrior\\wlm\\wlmas.clx",
                "plrgfx\\rogue\\rld\\rldas.clx",
                "plrgfx\\sorcerer\\sld\\sldas.clx",
                // UI
                "ui_art\\mainmenu.pcx",
                "ui_art\\title.pcx",
                "ui_art\\smlogo.pcx",
                // 关卡数据
                "levels\\towndata\\town.min",
                "levels\\towndata\\town.til",
                "levels\\towndata\\town.sol",
                "levels\\towndata\\town.cel",
                "levels\\l1data\\l1.min",
                "levels\\l1data\\l1.til",
                "levels\\l1data\\l1.sol",
                "levels\\l1data\\l1.cel",
                // 字体
                "fonts\\small\\all-chr.bin",
                "fonts\\big\\big-chr.bin",
                // 其他
                "gendata\\cut2.cel",
                "gendata\\cutstart.cel",
                "gendata\\cutl1d.cel",
                // 物品
                "items\\goldflip.cel",
                "items\\armor2.cel",
                // CLX 版本
                "ui_art\\mainmenu.clx",
                "gendata\\cut2.clx",
                "gendata\\cutstart.clx",
            ];

            println!("\nTesting file paths:");
            println!("{:-<60}", "");

            let mut found_count = 0;
            for path in &test_files {
                if archive.has_file(path) {
                    match archive.read_file(path) {
                        Ok(data) => {
                            println!("✓ {} ({} bytes)", path, data.len());
                            found_count += 1;
                        }
                        Err(e) => {
                            println!("✗ {} (found but read error: {})", path, e);
                        }
                    }
                }
            }

            println!("{:-<60}", "");
            println!("Found {} / {} test files", found_count, test_files.len());
        }
        Err(e) => {
            println!("Failed to open MPQ: {:?}", e);
        }
    }
}
