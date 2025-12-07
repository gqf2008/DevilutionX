//! Test MPQ file loading
//!
//! Run with: cargo run --example test_mpq

use devilutionx_rs::engine::mpq::MpqArchive;

fn main() {
    println!("=== DevilutionX-RS MPQ 测试 ===\n");

    // 尝试打开 spawn.mpq
    let mpq_path = ".\\spawn.mpq";
    println!("📦 打开 MPQ 文件: {}", mpq_path);

    match MpqArchive::open(mpq_path) {
        Ok(mut archive) => {
            println!("✓ MPQ 打开成功!");
            println!("  - Block 大小: {} 字节", archive.block_size());

            // 获取块信息
            let info = archive.get_block_info();
            let valid_blocks: Vec<_> = info.iter()
                .filter(|(_, _, packed, _)| *packed > 0)
                .collect();
            println!("  - 有效块数: {}", valid_blocks.len());

            // 测试读取多种文件类型
            println!("\n📄 测试文件读取:\n");

            let test_files = [
                ("ui_art\\title.pcx", "PCX 图像"),
                ("ui_art\\logo.pcx", "PCX 图像"),
                ("data\\inv\\objcurs.cel", "CEL 精灵"),
                ("gendata\\cutportl.cel", "CEL 精灵"),
                ("levels\\towndata\\town.cel", "城镇地图"),
                ("plrgfx\\warrior\\wlb\\wlbas.cl2", "CL2 动画"),
                ("sfx\\items\\gold.wav", "WAV 音频"),
            ];

            for (filename, desc) in &test_files {
                print!("  {} [{}] ... ", filename, desc);
                match archive.read_file(filename) {
                    Ok(data) => {
                        println!("✓ 成功! ({} 字节)", data.len());
                        // 显示文件头
                        if !data.is_empty() {
                            let preview: Vec<String> = data.iter()
                                .take(8)
                                .map(|b| format!("{:02X}", b))
                                .collect();
                            println!("    文件头: {}", preview.join(" "));
                        }
                    }
                    Err(e) => {
                        println!("✗ 失败: {}", e);
                    }
                }
            }

            // 统计成功读取的文件数量
            println!("\n📊 测试总结:\n");
            let mut success = 0;
            let mut failed = 0;

            // 测试所有可能存在的文件
            let more_files = [
                "ui_art\\smlogo.pcx",
                "ui_art\\focus.pcx",
                "ui_art\\but_sml.cel",
                "data\\char.cel",
                "gendata\\cutgate.cel",
                "gendata\\cutl1d.cel",
                "levels\\l1data\\l1.cel",
                "levels\\l2data\\l2.cel",
                "levels\\l3data\\l3.cel",
                "levels\\l4data\\l4.cel",
            ];

            for filename in &more_files {
                match archive.read_file(filename) {
                    Ok(data) => {
                        success += 1;
                        println!("  ✓ {} ({} 字节)", filename, data.len());
                    }
                    Err(_) => {
                        failed += 1;
                    }
                }
            }

            println!("\n  成功: {} / {}", success, success + failed);
        }
        Err(e) => {
            println!("✗ 打开失败: {}", e);
        }
    }

    println!("\n=== 测试完成 ===");
}
