//! 手动定义的 C FFI 绑定
//! 
//! 这些绑定对应 tools/c_api/devilutionx_c_api.h 中定义的 C API

use std::os::raw::{c_char, c_int};

// ============================================================================
// 错误码定义
// ============================================================================

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DvlxErrorCode {
    Ok = 0,
    NullPointer = 1,
    InvalidParam = 2,
    OutOfMemory = 3,
    NotImplemented = 4,
}

// ============================================================================
// 数据结构定义
// ============================================================================

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DvlxRect {
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DvlxPoint {
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DvlxFocusPosition {
    pub left_x: c_int,
    pub right_x: c_int,
    pub y: c_int,
}

// ============================================================================
// 核心函数声明
// ============================================================================

extern "C" {
    /// 获取 API 版本号
    pub fn dvlx_get_version() -> *const c_char;
    
    /// 初始化库
    pub fn dvlx_init() -> DvlxErrorCode;
    
    /// 清理库资源
    pub fn dvlx_cleanup();
}

// ============================================================================
// 动画函数声明
// ============================================================================

extern "C" {
    /// 计算动画帧索引
    /// 
    /// # 参数
    /// - ticks: 当前时间刻度
    /// - fps: 帧率 (每秒帧数)
    /// - frames: 总帧数
    /// 
    /// # 返回
    /// 成功返回帧索引 (0 到 frames-1), 失败返回 -1
    pub fn dvlx_get_animation_frame(ticks: c_int, fps: c_int, frames: c_int) -> c_int;
    
    /// 计算动画帧索引 (带错误码)
    /// 
    /// # 参数
    /// - ticks: 当前时间刻度
    /// - fps: 帧率
    /// - frames: 总帧数
    /// - out_frame: 输出帧索引的指针
    /// 
    /// # 返回
    /// 错误码
    pub fn dvlx_get_animation_frame_ex(
        ticks: c_int,
        fps: c_int,
        frames: c_int,
        out_frame: *mut c_int,
    ) -> DvlxErrorCode;
}

// ============================================================================
// UI 函数声明
// ============================================================================

extern "C" {
    /// 根据高度选择 Focus 尺寸
    /// 
    /// # 参数
    /// - height: 矩形高度
    /// 
    /// # 返回
    /// Focus 尺寸索引: 0=Small, 1=Medium, 2=Large
    pub fn dvlx_ui_select_focus_size(height: c_int) -> c_int;
    
    /// 计算居中位置
    /// 
    /// # 参数
    /// - container_size: 容器尺寸
    /// - element_size: 元素尺寸
    /// 
    /// # 返回
    /// 居中位置
    pub fn dvlx_ui_calculate_center(container_size: c_int, element_size: c_int) -> c_int;
    
    /// 计算 Focus 高亮位置
    /// 
    /// # 参数
    /// - rect: 目标矩形区域
    /// - sprite_width: Focus 精灵宽度
    /// - sprite_height: Focus 精灵高度
    /// - out_position: 输出位置的指针
    /// 
    /// # 返回
    /// 错误码
    pub fn dvlx_ui_calculate_focus_position(
        rect: *const DvlxRect,
        sprite_width: c_int,
        sprite_height: c_int,
        out_position: *mut DvlxFocusPosition,
    ) -> DvlxErrorCode;
}

// ============================================================================
// 错误处理函数声明
// ============================================================================

extern "C" {
    /// 获取最后一次错误码
    pub fn dvlx_get_last_error() -> DvlxErrorCode;
    
    /// 获取错误消息
    /// 
    /// # 参数
    /// - error_code: 错误码
    /// 
    /// # 返回
    /// 错误消息字符串 (C 风格, 不可变)
    pub fn dvlx_get_error_message(error_code: DvlxErrorCode) -> *const c_char;
    
    /// 清除错误状态
    pub fn dvlx_clear_error();
}
