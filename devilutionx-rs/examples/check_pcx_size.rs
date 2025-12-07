//! 检查PCX图像尺寸

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PCX图像尺寸检查 ===\n");

    let mut spawn = MpqArchive::open("spawn.mpq")?;

    let files = [
        "ui_art\\logo.pcx",
        "ui_art\\focus42.pcx",
        "ui_art\\focus.pcx",
        "ui_art\\focus16.pcx",
    ];

    for path in &files {
        if let Ok(data) = spawn.read_file(path) {
            if let Some(img) = PcxImage::decode(&data) {
                println!("{}", path);
                println!("  尺寸: {}x{}", img.width, img.height);

                // 猜测帧数
                let possible_heights = [16, 24, 42, 46, 50, 60, 100, 120, 150, 182, 216];
                for h in possible_heights {
                    if img.height % h == 0 {
                        let frames = img.height / h;
                        if frames > 1 && frames < 100 {
                            println!("  可能: {} 帧 x {} 高", frames, h);
                        }
                    }
                }
                println!();
            }
        }
    }

    println!("完成!");
    Ok(())
}
