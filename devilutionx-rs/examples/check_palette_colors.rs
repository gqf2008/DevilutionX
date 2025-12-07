//! 检查特定调色板索引的颜色

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 检查调色板颜色 ===\n");

    let mut spawn = MpqArchive::open("spawn.mpq")?;
    let title_pcx = spawn.read_file("ui_art\\title.pcx")?;
    let title_img = PcxImage::decode(&title_pcx).ok_or("Failed to decode title.pcx")?;

    // white.trn 映射的目标索引
    let white_targets = [242, 243, 244, 246, 249, 251, 252, 254];
    println!("white.trn 目标颜色:");
    for idx in white_targets {
        let c = &title_img.palette[idx];
        println!("  索引 {}: RGB({:3},{:3},{:3})", idx, c.r, c.g, c.b);
    }

    // goldui.trn 映射的目标索引 (176-191)
    println!("\ngoldui.trn 目标颜色 (176-191):");
    for idx in 176..=191 {
        let c = &title_img.palette[idx];
        println!("  索引 {}: RGB({:3},{:3},{:3})", idx, c.r, c.g, c.b);
    }

    // grayui.trn 映射的目标索引 (224-238)
    println!("\ngrayui.trn 目标颜色 (224-238):");
    for idx in 224..=238 {
        let c = &title_img.palette[idx];
        println!("  索引 {}: RGB({:3},{:3},{:3})", idx, c.r, c.g, c.b);
    }

    // 检查索引240-255是什么颜色
    println!("\n调色板 240-255 (通常是系统色):");
    for idx in 240..=255 {
        let c = &title_img.palette[idx];
        println!("  索引 {}: RGB({:3},{:3},{:3})", idx, c.r, c.g, c.b);
    }

    println!("\n完成!");
    Ok(())
}
