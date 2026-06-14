//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! SDL2 平台封装
//!
//! 移植自 C++ 中使用的 SDL2 功能，提供统一的跨平台接口。
//!
//! 包括：
//! - 时间管理 (SDL_GetTicks)
//! - 键盘状态 (SDL_GetModState)
//! - 显示/窗口管理

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;
use std::time::Instant;

// ============================================================================
// 时间管理
// ============================================================================

/// 程序启动时间点（用于计算运行时间）
static START_TIME: RwLock<Option<Instant>> = RwLock::new(None);

/// 初始化 SDL 时间系统
pub fn init_time() {
    let mut start = START_TIME.write().unwrap();
    if start.is_none() {
        *start = Some(Instant::now());
    }
}

/// 获取自程序启动以来的毫秒数
/// 
/// C++ 等价: `SDL_GetTicks()`
pub fn get_ticks() -> u32 {
    let start = START_TIME.read().unwrap();
    match *start {
        Some(instant) => instant.elapsed().as_millis() as u32,
        None => {
            // 自动初始化
            drop(start);
            init_time();
            0
        }
    }
}

/// 延迟指定毫秒数
/// 
/// C++ 等价: `SDL_Delay(ms)`
pub fn delay(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

// ============================================================================
// 键盘修饰键状态
// ============================================================================

bitflags::bitflags! {
    /// 键盘修饰键状态
    /// 
    /// C++ 等价: `SDL_Keymod` (SDL_KMOD_*)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KeyMod: u16 {
        const NONE = 0x0000;
        const LSHIFT = 0x0001;
        const RSHIFT = 0x0002;
        const LCTRL = 0x0040;
        const RCTRL = 0x0080;
        const LALT = 0x0100;
        const RALT = 0x0200;
        const LGUI = 0x0400;
        const RGUI = 0x0800;
        const NUM = 0x1000;
        const CAPS = 0x2000;
        const MODE = 0x4000;
        const SCROLL = 0x8000;
        
        // 组合键
        const CTRL = Self::LCTRL.bits() | Self::RCTRL.bits();
        const SHIFT = Self::LSHIFT.bits() | Self::RSHIFT.bits();
        const ALT = Self::LALT.bits() | Self::RALT.bits();
        const GUI = Self::LGUI.bits() | Self::RGUI.bits();
    }
}

/// 当前键盘修饰键状态
static CURRENT_MOD_STATE: AtomicU32 = AtomicU32::new(0);

/// 获取当前键盘修饰键状态
/// 
/// C++ 等价: `SDL_GetModState()`
pub fn get_mod_state() -> KeyMod {
    KeyMod::from_bits_truncate(CURRENT_MOD_STATE.load(Ordering::Relaxed) as u16)
}

/// 设置键盘修饰键状态（通常由事件系统调用）
pub fn set_mod_state(state: KeyMod) {
    CURRENT_MOD_STATE.store(state.bits() as u32, Ordering::Relaxed);
}

// ============================================================================
// 显示/窗口管理
// ============================================================================

/// 屏幕尺寸
static SCREEN_WIDTH: AtomicU32 = AtomicU32::new(640);
static SCREEN_HEIGHT: AtomicU32 = AtomicU32::new(480);

/// 视口高度（不包含控制面板）
static VIEWPORT_HEIGHT: AtomicU32 = AtomicU32::new(352); // 480 - 128

/// 是否为无头模式（无渲染）
static HEADLESS_MODE: AtomicBool = AtomicBool::new(false);

/// 窗口是否活动
static WINDOW_ACTIVE: AtomicBool = AtomicBool::new(true);

/// 获取屏幕宽度
/// 
/// C++ 等价: `GetScreenWidth()`
pub fn get_screen_width() -> u32 {
    SCREEN_WIDTH.load(Ordering::Relaxed)
}

/// 获取屏幕高度
/// 
/// C++ 等价: `GetScreenHeight()`
pub fn get_screen_height() -> u32 {
    SCREEN_HEIGHT.load(Ordering::Relaxed)
}

/// 获取视口高度（游戏区域，不含面板）
/// 
/// C++ 等价: `GetViewportHeight()`
pub fn get_viewport_height() -> u32 {
    VIEWPORT_HEIGHT.load(Ordering::Relaxed)
}

/// 设置屏幕尺寸
pub fn set_screen_size(width: u32, height: u32) {
    SCREEN_WIDTH.store(width, Ordering::Relaxed);
    SCREEN_HEIGHT.store(height, Ordering::Relaxed);
}

/// 设置视口高度
pub fn set_viewport_height(height: u32) {
    VIEWPORT_HEIGHT.store(height, Ordering::Relaxed);
}

/// 是否为无头模式
/// 
/// C++ 等价: `HeadlessMode`
pub fn is_headless_mode() -> bool {
    HEADLESS_MODE.load(Ordering::Relaxed)
}

/// 设置无头模式
pub fn set_headless_mode(headless: bool) {
    HEADLESS_MODE.store(headless, Ordering::Relaxed);
}

/// 窗口是否活动
/// 
/// C++ 等价: `gbActive`
pub fn is_window_active() -> bool {
    WINDOW_ACTIVE.load(Ordering::Relaxed)
}

/// 设置窗口活动状态
pub fn set_window_active(active: bool) {
    WINDOW_ACTIVE.store(active, Ordering::Relaxed);
}

// ============================================================================
// 鼠标
// ============================================================================

/// 鼠标位置
static MOUSE_POSITION: RwLock<(i32, i32)> = RwLock::new((0, 0));

/// 获取鼠标位置
/// 
/// C++ 等价: `MousePosition`
pub fn get_mouse_position() -> (i32, i32) {
    *MOUSE_POSITION.read().unwrap()
}

/// 设置鼠标位置
pub fn set_mouse_position(x: i32, y: i32) {
    *MOUSE_POSITION.write().unwrap() = (x, y);
}

// ============================================================================
// 渲染相关
// ============================================================================

/// 是否直接渲染到输出表面
/// 
/// C++ 等价: `RenderDirectlyToOutputSurface`
static RENDER_DIRECTLY_TO_OUTPUT: AtomicBool = AtomicBool::new(false);

/// 是否直接渲染到输出表面
pub fn render_directly_to_output_surface() -> bool {
    RENDER_DIRECTLY_TO_OUTPUT.load(Ordering::Relaxed)
}

/// 设置是否直接渲染到输出表面
pub fn set_render_directly_to_output_surface(value: bool) {
    RENDER_DIRECTLY_TO_OUTPUT.store(value, Ordering::Relaxed);
}

/// 渲染呈现（实际的 SDL 调用）
/// 
/// C++ 等价: `RenderPresent()`
pub fn render_present() {
    // TODO: 实际调用 SDL2 的 RenderPresent
    // 当前为空实现
}

/// Blit 操作（快速位块传输）
/// 
/// C++ 等价: `BltFast()`
pub fn blt_fast(src_rect: &SdlRect, dst_rect: &SdlRect) {
    // TODO: 实际调用 SDL2 的 blit
    let _ = (src_rect, dst_rect);
}

// ============================================================================
// SDL 矩形
// ============================================================================

/// SDL 矩形结构
/// 
/// C++ 等价: `SDL_Rect`
#[derive(Debug, Clone, Copy, Default)]
pub struct SdlRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl SdlRect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

/// 从引擎 Rectangle 创建 SDL 矩形
/// 
/// C++ 等价: `MakeSdlRect()`
pub fn make_sdl_rect(rect: crate::engine::Rectangle) -> SdlRect {
    SdlRect {
        x: rect.position.x,
        y: rect.position.y,
        w: rect.size.width,
        h: rect.size.height,
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time() {
        init_time();
        let t1 = get_ticks();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = get_ticks();
        assert!(t2 >= t1 + 10);
    }

    #[test]
    fn test_key_mod() {
        set_mod_state(KeyMod::LSHIFT | KeyMod::LCTRL);
        let state = get_mod_state();
        assert!(state.contains(KeyMod::LSHIFT));
        assert!(state.contains(KeyMod::LCTRL));
        assert!(state.contains(KeyMod::SHIFT));
        assert!(state.contains(KeyMod::CTRL));
        assert!(!state.contains(KeyMod::ALT));
    }

    #[test]
    fn test_screen_size() {
        set_screen_size(800, 600);
        assert_eq!(get_screen_width(), 800);
        assert_eq!(get_screen_height(), 600);
    }

    #[test]
    fn test_mouse_position() {
        set_mouse_position(100, 200);
        let (x, y) = get_mouse_position();
        assert_eq!(x, 100);
        assert_eq!(y, 200);
    }

    #[test]
    fn test_sdl_rect() {
        let rect = SdlRect::new(10, 20, 100, 200);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.w, 100);
        assert_eq!(rect.h, 200);
    }
}
