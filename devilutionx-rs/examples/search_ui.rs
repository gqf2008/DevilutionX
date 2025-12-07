//! 搜索 MPQ 中的 UI 相关文件

use devilutionx_rs::engine::mpq::MpqArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = MpqArchive::open("spawn.mpq")?;

    // 测试更多 UI 文件
    let ui_files = [
        // 字体
        "ui_art\\font16.bin",
        "ui_art\\font16s.bin",
        "ui_art\\font24.bin",
        "ui_art\\font30.bin",
        "ui_art\\font42.bin",
        "ui_art\\font16g.pcx",
        "ui_art\\font16s.pcx",
        "ui_art\\font24g.pcx",
        "ui_art\\font24s.pcx",
        "ui_art\\font30g.pcx",
        "ui_art\\font30s.pcx",
        "ui_art\\font42g.pcx",
        "ui_art\\font42y.pcx",
        // 菜单按钮和文字
        "ui_art\\but_sml.pcx",
        "ui_art\\but_lrg.pcx",
        "ui_art\\butbroad.pcx",
        "ui_art\\selhero.pcx",
        "ui_art\\selgame.pcx",
        "ui_art\\selconn.pcx",
        "ui_art\\credits.pcx",
        // 对话框
        "ui_art\\lpopup.pcx",
        "ui_art\\spopup.pcx",
        // 菜单项
        "ui_art\\menu.pcx",
        "ui_art\\menulogo.pcx",
        // 光标
        "ui_art\\cursor.pcx",
        // 其他 UI
        "ui_art\\heros.pcx",
        "ui_art\\herow.pcx",
        "ui_art\\heror.pcx",
        "ui_art\\focus16.pcx",
        "ui_art\\focus42.pcx",
        "ui_art\\focus.pcx",
        // 滚动条
        "ui_art\\sb_arrow.pcx",
        "ui_art\\sb_bg.pcx",
        "ui_art\\sb_thumb.pcx",
        // 进度条
        "ui_art\\prog_bg.pcx",
        "ui_art\\prog_fil.pcx",
        // 小图标
        "ui_art\\logo.pcx",
        "ui_art\\diabsmal.pcx",
    ];

    println!("Searching UI files in spawn.mpq:\n");

    let mut found = Vec::new();
    for path in &ui_files {
        if archive.has_file(path) {
            match archive.read_file(path) {
                Ok(data) => {
                    println!("✓ {} ({} bytes)", path, data.len());
                    found.push(*path);
                }
                Err(e) => {
                    println!("✗ {} (error: {})", path, e);
                }
            }
        }
    }

    println!("\n========================================");
    println!("Found {} UI files", found.len());

    Ok(())
}
