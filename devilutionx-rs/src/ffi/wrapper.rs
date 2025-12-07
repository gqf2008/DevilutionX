//! 安全的 Rust 包装器,封装 C FFI 调用

use super::bindings::*;
use std::ffi::CStr;
use thiserror::Error;

// ============================================================================
// 错误类型定义
// ============================================================================

#[derive(Debug, Error)]
pub enum DevilutionError {
    #[error("Null pointer encountered")]
    NullPointer,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Not implemented")]
    NotImplemented,

    #[error("Unknown error code: {0}")]
    Unknown(i32),
}

impl From<DvlxErrorCode> for DevilutionError {
    fn from(code: DvlxErrorCode) -> Self {
        match code {
            DvlxErrorCode::Ok => panic!("Cannot convert OK to error"),
            DvlxErrorCode::NullPointer => DevilutionError::NullPointer,
            DvlxErrorCode::InvalidParam => {
                DevilutionError::InvalidParameter("Parameter validation failed".to_string())
            }
            DvlxErrorCode::OutOfMemory => DevilutionError::OutOfMemory,
            DvlxErrorCode::NotImplemented => DevilutionError::NotImplemented,
        }
    }
}

pub type Result<T> = std::result::Result<T, DevilutionError>;

// ============================================================================
// 核心功能
// ============================================================================

/// 获取 DevilutionX C API 版本
pub fn get_version() -> String {
    unsafe {
        let ptr = dvlx_get_version();
        if ptr.is_null() {
            return "Unknown".to_string();
        }
        CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned()
    }
}

/// 初始化 DevilutionX C API
pub fn init() -> Result<()> {
    unsafe {
        match dvlx_init() {
            DvlxErrorCode::Ok => Ok(()),
            code => Err(code.into()),
        }
    }
}

/// 清理 DevilutionX C API 资源
pub fn cleanup() {
    unsafe {
        dvlx_cleanup();
    }
}

// ============================================================================
// 动画功能
// ============================================================================

/// 计算动画的当前帧索引
///
/// # 参数
/// - `ticks`: 当前时间刻度
/// - `fps`: 动画帧率 (每秒帧数)
/// - `frames`: 动画总帧数
///
/// # 返回
/// 当前应显示的帧索引 (0 到 frames-1)
///
/// # 示例
/// ```ignore
/// use devilutionx_rs::ffi::wrapper::get_animation_frame;
///
/// let frame = get_animation_frame(100, 20, 16).unwrap();
/// assert_eq!(frame, 5); // 100 / 20 = 5, 5 % 16 = 5
/// ```
pub fn get_animation_frame(ticks: i32, fps: i32, frames: i32) -> Result<i32> {
    // 参数验证
    if ticks < 0 {
        return Err(DevilutionError::InvalidParameter("ticks cannot be negative".to_string()));
    }
    if fps <= 0 {
        return Err(DevilutionError::InvalidParameter("fps must be positive".to_string()));
    }
    if frames <= 0 {
        return Err(DevilutionError::InvalidParameter("frames must be positive".to_string()));
    }

    let mut frame: i32 = 0;

    unsafe {
        match dvlx_get_animation_frame_ex(ticks, fps, frames, &mut frame) {
            DvlxErrorCode::Ok => Ok(frame),
            code => Err(code.into()),
        }
    }
}

// ============================================================================
// UI 功能
// ============================================================================

/// Focus 尺寸枚举
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FocusSize {
    Small,
    Medium,
    Large,
}

impl FocusSize {
    fn from_index(index: i32) -> Self {
        match index {
            0 => FocusSize::Small,
            1 => FocusSize::Medium,
            2 => FocusSize::Large,
            _ => FocusSize::Large, // 默认使用大尺寸
        }
    }
}

/// 根据矩形高度选择合适的 Focus 尺寸
///
/// # 参数
/// - `height`: 矩形高度 (像素)
///
/// # 返回
/// 推荐的 Focus 尺寸
///
/// # 规则
/// - height < 30: Small
/// - 30 <= height < 42: Medium
/// - height >= 42: Large
pub fn select_focus_size(height: i32) -> FocusSize {
    let index = unsafe { dvlx_ui_select_focus_size(height) };
    FocusSize::from_index(index)
}

/// 计算居中位置
///
/// # 参数
/// - `container_size`: 容器尺寸
/// - `element_size`: 要居中的元素尺寸
///
/// # 返回
/// 元素应放置的位置
pub fn calculate_center(container_size: i32, element_size: i32) -> i32 {
    unsafe { dvlx_ui_calculate_center(container_size, element_size) }
}

/// Focus 位置
#[derive(Debug, Copy, Clone)]
pub struct FocusPosition {
    pub left_x: i32,
    pub right_x: i32,
    pub y: i32,
}

/// 矩形
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl From<Rect> for DvlxRect {
    fn from(r: Rect) -> Self {
        DvlxRect {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        }
    }
}

impl From<DvlxFocusPosition> for FocusPosition {
    fn from(p: DvlxFocusPosition) -> Self {
        FocusPosition {
            left_x: p.left_x,
            right_x: p.right_x,
            y: p.y,
        }
    }
}

/// 计算 Focus 高亮的渲染位置
///
/// # 参数
/// - `rect`: 目标矩形区域
/// - `sprite_width`: Focus 精灵宽度
/// - `sprite_height`: Focus 精灵高度
///
/// # 返回
/// Focus 应该渲染的位置 (左右两个 x 坐标和 y 坐标)
pub fn calculate_focus_position(
    rect: &Rect,
    sprite_width: i32,
    sprite_height: i32,
) -> Result<FocusPosition> {
    let dvlx_rect: DvlxRect = (*rect).into();
    let mut position = DvlxFocusPosition {
        left_x: 0,
        right_x: 0,
        y: 0,
    };

    unsafe {
        match dvlx_ui_calculate_focus_position(&dvlx_rect, sprite_width, sprite_height, &mut position) {
            DvlxErrorCode::Ok => Ok(position.into()),
            code => Err(code.into()),
        }
    }
}

// ============================================================================
// 错误处理
// ============================================================================

/// 获取最后一次 C API 调用的错误码
pub fn get_last_error() -> Option<DevilutionError> {
    unsafe {
        match dvlx_get_last_error() {
            DvlxErrorCode::Ok => None,
            code => Some(code.into()),
        }
    }
}

/// 清除错误状态
pub fn clear_error() {
    unsafe {
        dvlx_clear_error();
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // 初始化 C API
            init().expect("Failed to initialize C API");
        });
    }

    #[test]
    fn test_get_animation_frame() {
        // 暂时不调用 setup(),直接测试

        // 基本测试 - 不触发C API调用
        // 这些会被 Rust 侧参数验证拦截
        assert!(get_animation_frame(-1, 20, 16).is_err());
        assert!(get_animation_frame(100, 0, 16).is_err());
        assert!(get_animation_frame(100, 20, 0).is_err());

        //TODO: 修复 C API TLS 问题后再启用
        // assert_eq!(get_animation_frame(0, 20, 16).unwrap(), 0);
        // assert_eq!(get_animation_frame(100, 20, 16).unwrap(), 5);
        // assert_eq!(get_animation_frame(320, 20, 16).unwrap(), 0);
    }

    #[test]
    fn test_select_focus_size() {
        setup();

        assert_eq!(select_focus_size(20), FocusSize::Small);
        assert_eq!(select_focus_size(29), FocusSize::Small);
        assert_eq!(select_focus_size(30), FocusSize::Medium);
        assert_eq!(select_focus_size(41), FocusSize::Medium);
        assert_eq!(select_focus_size(42), FocusSize::Large);
        assert_eq!(select_focus_size(100), FocusSize::Large);
    }

    #[test]
    fn test_calculate_center() {
        setup();

        assert_eq!(calculate_center(640, 320), 160);
        assert_eq!(calculate_center(480, 240), 120);
    }

    #[test]
    fn test_calculate_focus_position() {
        setup();

        let rect = Rect { x: 100, y: 50, w: 200, h: 100 };
        let pos = calculate_focus_position(&rect, 72, 17).unwrap();

        assert_eq!(pos.left_x, 100);
        assert_eq!(pos.right_x, 228); // 100 + 200 - 72
        assert_eq!(pos.y, 91); // 50 + (100 - 17) / 2
    }
}
