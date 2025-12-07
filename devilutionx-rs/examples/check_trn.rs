//! 检查TRN文件原始内容

use devilutionx_rs::engine::mpq::MpqArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TRN文件检查 ===\n");

    let mut devilutionx = MpqArchive::open("devilutionx.mpq")?;

    let trn_files = [
        "fonts\\goldui.trn",
        "fonts\\white.trn",
        "fonts\\grayui.trn",
    ];

    for path in &trn_files {
        println!("\n文件: {}", path);
        if devilutionx.has_file(path) {
            match devilutionx.read_file(path) {
                Ok(data) => {
                    println!("  大小: {} 字节", data.len());

                    // 打印前16字节
                    print!("  前16字节: ");
                    for b in data.iter().take(16) {
                        print!("{:02X} ", b);
                    }
                    println!();

                    // 检查恒等映射
                    let mut identity_count = 0;
                    let mut changes = Vec::new();
                    for i in 0..256.min(data.len()) {
                        if data[i] as usize == i {
                            identity_count += 1;
                        } else {
                            changes.push((i, data[i]));
                        }
                    }
                    println!("  恒等映射数: {}", identity_count);
                    println!("  非恒等映射数: {}", changes.len());

                    // 打印所有非恒等映射
                    println!("  所有非恒等映射:");
                    for (src, dst) in &changes {
                        println!("    {} -> {}", src, dst);
                    }
                }
                Err(e) => {
                    println!("  读取失败: {}", e);
                }
            }
        } else {
            println!("  文件不存在!");
        }
    }

    println!("\n检查完成!");
    Ok(())
}
