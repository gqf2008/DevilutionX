//! 报告 spawn.mpq 中关键 UI 资产的 PCX 尺寸，判断哪些是多帧动画。
use devilutionx_rs::engine::mpq::MpqArchive;

// 最小 PCX 头解析:只读 width/height
fn pcx_size(data: &[u8]) -> Option<(u16, u16)> {
    if data.len() < 12 { return None; }
    // PCX: manufacturer=0x0a, version, encoding, bpp@2
    // window: xmin(u16@4), ymin(u16@6), xmax(u16@8), ymax(u16@10)
    let xmin = u16::from_le_bytes([data[4], data[5]]);
    let ymin = u16::from_le_bytes([data[6], data[7]]);
    let xmax = u16::from_le_bytes([data[8], data[9]]);
    let ymax = u16::from_le_bytes([data[10], data[11]]);
    Some(((xmax - xmin + 1), (ymax - ymin + 1)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mpq = MpqArchive::open("spawn.mpq")?;
    let targets = [
        r"ui_art\swmmenu.pcx", r"ui_art\smlogo.pcx", r"ui_art\title.pcx",
        r"ui_art\focus.pcx", r"ui_art\focus42.pcx", r"ui_art\focus16.pcx",
        r"ui_art\selhero.pcx", r"ui_art\welcome.pcx", r"ui_art\cursor.pcx",
        r"ui_art\spopup.pcx", r"ui_art\spwnport.pcx",
    ];
    for path in &targets {
        match mpq.read_file(path) {
            Ok(data) => {
                if let Some((w, h)) = pcx_size(&data) {
                    let ratio = if w > 0 { h as f32 / w as f32 } else { 0.0 };
                    let multi = if ratio > 1.5 { " <-- 高度异常(可能多帧动画)" } else { "" };
                    println!("{:30} {}x{} (h/w={:.2}, {} bytes){}", path, w, h, ratio, data.len(), multi);
                } else {
                    println!("{:30} 无法解析 PCX 头", path);
                }
            }
            Err(e) => println!("{:30} 读取失败: {}", path, e),
        }
    }
    Ok(())
}
