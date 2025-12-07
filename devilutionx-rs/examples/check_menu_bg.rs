//! 检查主菜单背景文件

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 检查主菜单背景文件 ===\n");

    let mut spawn = MpqArchive::open("spawn.mpq")?;

    let files = [
        "ui_art\\mainmenu.pcx",
        "ui_art\\swmmenu.pcx",
        "ui_art\\menu.pcx",
        "ui_art\\title.pcx",
    ];

    for path in &files {
        print!("{}: ", path);
        if spawn.has_file(path) {
            if let Ok(data) = spawn.read_file(path) {
                if let Some(img) = PcxImage::decode(&data) {
                    println!("{}x{}", img.width, img.height);
                } else {
                    println!("解码失败");
                }
            }
        } else {
            println!("不存在");
        }
    }

    println!("\n完成!");
    Ok(())
}
