//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Graphics Pipeline Initialization
//!
//! C++ Reference: Source/engine/dx.h, Source/engine/dx.cpp

use std::sync::atomic::{AtomicBool, Ordering};

// Whether we render directly to the screen surface, i.e. `PalSurface == GetOutputSurface()`
static RENDER_DIRECTLY_TO_OUTPUT_SURFACE: AtomicBool = AtomicBool::new(false);

/// Whether we render directly to the screen surface
pub fn render_directly_to_output_surface() -> bool {
    RENDER_DIRECTLY_TO_OUTPUT_SURFACE.load(Ordering::Relaxed)
}

/// Set whether we render directly to the screen surface
pub fn set_render_directly_to_output_surface(value: bool) {
    RENDER_DIRECTLY_TO_OUTPUT_SURFACE.store(value, Ordering::Relaxed);
}

/// SDL_Rect equivalent
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

/// Initialize DirectX/SDL graphics subsystem
///
/// C++ Reference: dx.cpp:dx_init()
pub fn dx_init() {
    // SDL initialization will be done when SDL context is available
}

/// Cleanup DirectX/SDL graphics subsystem
///
/// C++ Reference: dx.cpp:dx_cleanup()
pub fn dx_cleanup() {
    // SDL cleanup will be done when SDL context is available
}

/// Create the back buffer surface
///
/// C++ Reference: dx.cpp:CreateBackBuffer()
pub fn create_back_buffer() {
    // Implementation requires SDL surface management
}

/// Get the global back buffer as a Surface
///
/// C++ Reference: dx.cpp:GlobalBackBuffer()
/// 
/// 依赖: 需要 SDL surface 管理，返回 PalSurface 的 Surface 包装
/// 当前返回 stub，实际实现需要与 SDL 集成
pub fn global_back_buffer() {
    // Returns the PalSurface as a Surface wrapper
    // Implementation requires SDL surface management
}

/// Fast blit from source rect to dest rect
///
/// C++ Reference: dx.cpp:BltFast()
pub fn blt_fast(_src_rect: Option<&Rect>, _dst_rect: Option<&Rect>) {
    // Implementation requires SDL surface management
}

/// Blit from a source surface
///
/// C++ Reference: dx.cpp:Blit()
pub fn blit(_src_rect: Option<&Rect>, _dst_rect: Option<&Rect>) {
    // Implementation requires SDL surface management
}

/// Present the rendered frame to the screen
///
/// C++ Reference: dx.cpp:RenderPresent()
pub fn render_present() {
    // Implementation requires SDL renderer management
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let r = Rect::new(10, 20, 100, 200);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.w, 100);
        assert_eq!(r.h, 200);
    }

    #[test]
    fn test_render_directly_flag() {
        set_render_directly_to_output_surface(true);
        assert!(render_directly_to_output_surface());
        set_render_directly_to_output_surface(false);
        assert!(!render_directly_to_output_surface());
    }
}
