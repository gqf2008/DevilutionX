//! DevilutionX C FFI 绑定模块
//!
//! 提供 C++ 核心库的 Rust 接口

pub mod bindings;
pub mod wrapper;

// 重新导出常用类型
pub use wrapper::{
    DevilutionError, Result,
    get_version, init, cleanup,
    get_animation_frame,
    select_focus_size, FocusSize,
    calculate_center,
    calculate_focus_position, FocusPosition, Rect,
    get_last_error, clear_error,
};
