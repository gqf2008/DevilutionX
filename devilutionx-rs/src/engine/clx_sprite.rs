//! CLX 精灵格式 - 移植自 Source/engine/clx_sprite.hpp
//!
//! CLX 是 DevilutionX 运行时使用的图形格式。
//! CLX 像素编码方式与 CL2 相同，但元数据编码不同。
//!
//! CLX 帧头格式 (6 字节):
//! - 字节 0..2: uint16_t 头大小
//! - 字节 2..4: uint16_t 宽度
//! - 字节 4..6: uint16_t 高度

use std::sync::Arc;

/// 读取小端序 u16
#[inline]
fn load_le16(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}

/// 读取小端序 u32
#[inline]
fn load_le32(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// 单个 CLX 精灵
#[derive(Clone)]
pub struct ClxSprite {
    data: Arc<Vec<u8>>,
    offset: usize,
    pixel_data_size: u32,
}

impl ClxSprite {
    /// 从数据创建（offset 是精灵在数据中的偏移）
    pub fn new(data: Arc<Vec<u8>>, offset: usize, data_size: u32) -> Self {
        let header_size = load_le16(&data[offset..]) as u32;
        Self {
            data,
            offset,
            pixel_data_size: data_size - header_size,
        }
    }

    /// 获取宽度
    #[inline]
    pub fn width(&self) -> u16 {
        load_le16(&self.data[self.offset + 2..])
    }

    /// 获取高度
    #[inline]
    pub fn height(&self) -> u16 {
        load_le16(&self.data[self.offset + 4..])
    }

    /// 获取头部大小
    #[inline]
    pub fn header_size(&self) -> u16 {
        load_le16(&self.data[self.offset..])
    }

    /// 获取像素数据（CL2 帧数据）
    #[inline]
    pub fn pixel_data(&self) -> &[u8] {
        let header_size = self.header_size() as usize;
        let start = self.offset + header_size;
        let end = start + self.pixel_data_size as usize;
        &self.data[start..end]
    }

    /// 获取像素数据大小
    #[inline]
    pub fn pixel_data_size(&self) -> u32 {
        self.pixel_data_size
    }
}

impl std::fmt::Debug for ClxSprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClxSprite")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("pixel_data_size", &self.pixel_data_size)
            .finish()
    }
}

/// CLX 精灵列表（一组精灵）
#[derive(Clone)]
pub struct ClxSpriteList {
    data: Arc<Vec<u8>>,
    offset: usize,
}

impl ClxSpriteList {
    /// 从数据创建
    pub fn new(data: Arc<Vec<u8>>, offset: usize) -> Self {
        Self { data, offset }
    }

    /// 从原始数据创建（拥有数据）
    pub fn from_data(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
            offset: 0,
        }
    }

    /// 获取精灵数量
    #[inline]
    pub fn num_sprites(&self) -> u32 {
        load_le32(&self.data[self.offset..])
    }

    /// 获取精灵偏移
    #[inline]
    pub fn sprite_offset(&self, index: usize) -> u32 {
        load_le32(&self.data[self.offset + 4 + index * 4..])
    }

    /// 获取数据大小
    #[inline]
    pub fn data_size(&self) -> u32 {
        let num = self.num_sprites() as usize;
        load_le32(&self.data[self.offset + 4 + num * 4..])
    }

    /// 获取指定索引的精灵
    pub fn get(&self, index: usize) -> Option<ClxSprite> {
        if index >= self.num_sprites() as usize {
            return None;
        }
        let begin = self.sprite_offset(index);
        let end = self.sprite_offset(index + 1);
        Some(ClxSprite::new(
            self.data.clone(),
            self.offset + begin as usize,
            end - begin,
        ))
    }

    /// 迭代所有精灵
    pub fn iter(&self) -> impl Iterator<Item = ClxSprite> + '_ {
        (0..self.num_sprites() as usize).map(move |i| self.get(i).unwrap())
    }
}

impl std::fmt::Debug for ClxSpriteList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClxSpriteList")
            .field("num_sprites", &self.num_sprites())
            .field("data_size", &self.data_size())
            .finish()
    }
}

/// CLX 精灵表（精灵列表的列表）
#[derive(Clone)]
pub struct ClxSpriteSheet {
    data: Arc<Vec<u8>>,
    num_lists: u16,
}

impl ClxSpriteSheet {
    /// 从数据创建
    pub fn new(data: Arc<Vec<u8>>, num_lists: u16) -> Self {
        assert!(num_lists > 0);
        Self { data, num_lists }
    }

    /// 从原始数据创建（自动检测列表数量）
    pub fn from_data(data: Vec<u8>) -> Self {
        let num_lists = get_num_lists_from_buffer(&data);
        Self {
            data: Arc::new(data),
            num_lists,
        }
    }

    /// 获取列表数量
    #[inline]
    pub fn num_lists(&self) -> u16 {
        self.num_lists
    }

    /// 获取表偏移
    #[inline]
    pub fn sheet_offset(&self, index: usize) -> u32 {
        assert!(index < self.num_lists as usize);
        load_le32(&self.data[index * 4..])
    }

    /// 获取指定索引的精灵列表
    pub fn get(&self, index: usize) -> Option<ClxSpriteList> {
        if index >= self.num_lists as usize {
            return None;
        }
        let offset = self.sheet_offset(index) as usize;
        Some(ClxSpriteList::new(self.data.clone(), offset))
    }

    /// 获取数据大小
    pub fn data_size(&self) -> usize {
        let last_list = self.get(self.num_lists as usize - 1).unwrap();
        let last_offset = self.sheet_offset(self.num_lists as usize - 1) as usize;
        last_offset + last_list.data_size() as usize
    }

    /// 迭代所有精灵列表
    pub fn iter(&self) -> impl Iterator<Item = ClxSpriteList> + '_ {
        (0..self.num_lists as usize).map(move |i| self.get(i).unwrap())
    }
}

impl std::fmt::Debug for ClxSpriteSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClxSpriteSheet")
            .field("num_lists", &self.num_lists)
            .finish()
    }
}

/// 从缓冲区检测是否为精灵表，返回列表数量（0 表示是单个列表）
pub fn get_num_lists_from_buffer(data: &[u8]) -> u16 {
    let maybe_num_frames = load_le32(data);

    // 如果是帧数，最后一个帧偏移应该等于文件大小
    let offset = maybe_num_frames as usize * 4 + 4;
    if offset < data.len() && load_le32(&data[offset..]) as usize != data.len() {
        return (maybe_num_frames / 4) as u16;
    }

    // 不是精灵表
    0
}

/// CLX 精灵列表或精灵表
pub enum ClxSpriteListOrSheet {
    List(ClxSpriteList),
    Sheet(ClxSpriteSheet),
}

impl ClxSpriteListOrSheet {
    /// 从数据自动检测类型
    pub fn from_data(data: Vec<u8>) -> Self {
        let num_lists = get_num_lists_from_buffer(&data);
        let arc_data = Arc::new(data);
        if num_lists == 0 {
            ClxSpriteListOrSheet::List(ClxSpriteList::new(arc_data, 0))
        } else {
            ClxSpriteListOrSheet::Sheet(ClxSpriteSheet::new(arc_data, num_lists))
        }
    }

    /// 是否为精灵表
    pub fn is_sheet(&self) -> bool {
        matches!(self, ClxSpriteListOrSheet::Sheet(_))
    }

    /// 获取列表（如果是列表）
    pub fn as_list(&self) -> Option<&ClxSpriteList> {
        match self {
            ClxSpriteListOrSheet::List(list) => Some(list),
            _ => None,
        }
    }

    /// 获取表（如果是表）
    pub fn as_sheet(&self) -> Option<&ClxSpriteSheet> {
        match self {
            ClxSpriteListOrSheet::Sheet(sheet) => Some(sheet),
            _ => None,
        }
    }
}

// ============================================================================
// CLX 解码工具函数
// ============================================================================

/// 检查控制字节是否表示不透明像素
#[inline]
pub const fn is_clx_opaque(control: u8) -> bool {
    control >= 0x80
}

/// 获取不透明像素宽度
#[inline]
pub const fn get_clx_opaque_pixels_width(control: u8) -> u8 {
    (-(control as i8)) as u8
}

/// 检查是否为填充命令
#[inline]
pub const fn is_clx_opaque_fill(control: u8) -> bool {
    control <= 0xBE
}

/// 获取填充宽度
#[inline]
pub const fn get_clx_opaque_fill_width(control: u8) -> u8 {
    0xBF - control
}

/// CLX 命令信息
pub struct ClxBlitInfo {
    pub next_offset: usize,
    pub length: u8,
}

/// 解析 CLX 命令
pub fn clx_blit_info(src: &[u8], offset: usize) -> ClxBlitInfo {
    let control = src[offset];
    if !is_clx_opaque(control) {
        // 透明运行
        ClxBlitInfo {
            next_offset: offset + 1,
            length: control,
        }
    } else if is_clx_opaque_fill(control) {
        // 填充命令
        ClxBlitInfo {
            next_offset: offset + 2,
            length: get_clx_opaque_fill_width(control),
        }
    } else {
        // 像素数据
        let width = get_clx_opaque_pixels_width(control);
        ClxBlitInfo {
            next_offset: offset + 1 + width as usize,
            length: width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clx_opaque_check() {
        assert!(!is_clx_opaque(0x7F));
        assert!(is_clx_opaque(0x80));
        assert!(is_clx_opaque(0xFF));
    }

    #[test]
    fn test_clx_fill_check() {
        assert!(is_clx_opaque_fill(0x80));
        assert!(is_clx_opaque_fill(0xBE));
        assert!(!is_clx_opaque_fill(0xBF));
        assert!(!is_clx_opaque_fill(0xFF));
    }

    #[test]
    fn test_fill_width() {
        assert_eq!(get_clx_opaque_fill_width(0xBE), 1);
        assert_eq!(get_clx_opaque_fill_width(0x80), 63);
    }
}
