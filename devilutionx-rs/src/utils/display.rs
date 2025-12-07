//! 显示/窗口管理系统
//!
//! 对应 C++: Source/utils/display.cpp/h
//!
//! 该模块负责：
//! - 窗口创建和管理
//! - 显示模式设置
//! - 分辨率和缩放
//! - 坐标转换 (逻辑 <-> 输出)

use std::sync::OnceLock;
use parking_lot::RwLock;

//=============================================================================
// 类型定义
//=============================================================================

/// 二维尺寸
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self { width: 0, height: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// 二维点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// 矩形区域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn x(&self) -> i32 {
        self.position.x
    }

    pub fn y(&self) -> i32 {
        self.position.y
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn right(&self) -> i32 {
        self.position.x + self.size.width as i32
    }

    pub fn bottom(&self) -> i32 {
        self.position.y + self.size.height as i32
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.right()
            && point.y >= self.position.y
            && point.y < self.bottom()
    }
}

//=============================================================================
// 显示状态
//=============================================================================

/// 显示状态
pub struct DisplayState {
    /// 屏幕宽度
    screen_width: u16,
    /// 屏幕高度
    screen_height: u16,
    /// 视口高度
    viewport_height: u16,
    /// 强制分辨率
    force_resolution: Size,
    /// UI 矩形区域
    ui_rectangle: Rectangle,
    /// 是否全屏
    fullscreen: bool,
    /// DPI 缩放因子
    dpi_scale: f32,
    /// 刷新延迟 (纳秒)
    refresh_delay: i32,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            screen_width: 640,
            screen_height: 480,
            viewport_height: 480,
            force_resolution: Size::zero(),
            ui_rectangle: Rectangle::new(0, 0, 640, 480),
            fullscreen: false,
            dpi_scale: 1.0,
            refresh_delay: 16_666_667, // ~60 FPS
        }
    }
}

/// 全局显示状态
static DISPLAY: OnceLock<RwLock<DisplayState>> = OnceLock::new();

fn display() -> &'static RwLock<DisplayState> {
    DISPLAY.get_or_init(|| RwLock::new(DisplayState::default()))
}

//=============================================================================
// 公共 API
//=============================================================================

/// 获取屏幕宽度
pub fn get_screen_width() -> u16 {
    display().read().screen_width
}

/// 获取屏幕高度
pub fn get_screen_height() -> u16 {
    display().read().screen_height
}

/// 获取视口高度
pub fn get_viewport_height() -> u16 {
    display().read().viewport_height
}

/// 获取屏幕尺寸
pub fn get_screen_size() -> Size {
    let state = display().read();
    Size::new(state.screen_width as u32, state.screen_height as u32)
}

/// 获取 UI 矩形区域
pub fn get_ui_rectangle() -> Rectangle {
    display().read().ui_rectangle
}

/// 是否全屏
pub fn is_fullscreen() -> bool {
    display().read().fullscreen
}

/// 获取 DPI 缩放因子
pub fn get_dpi_scaling_factor() -> f32 {
    display().read().dpi_scale
}

/// 获取刷新延迟 (纳秒)
pub fn get_refresh_delay() -> i32 {
    display().read().refresh_delay
}

/// 设置屏幕尺寸
pub fn set_screen_size(width: u16, height: u16) {
    let mut state = display().write();
    state.screen_width = width;
    state.screen_height = height;
}

/// 设置视口高度
pub fn set_viewport_height(height: u16) {
    display().write().viewport_height = height;
}

/// 设置全屏状态
pub fn set_fullscreen(fullscreen: bool) {
    display().write().fullscreen = fullscreen;
}

/// 设置强制分辨率
pub fn set_force_resolution(size: Size) {
    display().write().force_resolution = size;
}

/// 获取强制分辨率
pub fn get_force_resolution() -> Size {
    display().read().force_resolution
}

/// 设置 UI 矩形
pub fn set_ui_rectangle(rect: Rectangle) {
    display().write().ui_rectangle = rect;
}

/// 设置 DPI 缩放因子
pub fn set_dpi_scaling_factor(scale: f32) {
    display().write().dpi_scale = scale;
}

/// 设置刷新延迟
pub fn set_refresh_delay(delay: i32) {
    display().write().refresh_delay = delay;
}

//=============================================================================
// 坐标转换
//=============================================================================

/// 从输出坐标转换为逻辑坐标
///
/// 用于处理鼠标输入等，将屏幕坐标转换为游戏逻辑坐标。
pub fn output_to_logical(x: i32, y: i32) -> (i32, i32) {
    let state = display().read();
    let scale = state.dpi_scale;

    if scale == 1.0 {
        return (x, y);
    }

    let logical_x = (x as f32 / scale) as i32;
    let logical_y = (y as f32 / scale) as i32;

    (logical_x, logical_y)
}

/// 从逻辑坐标转换为输出坐标
///
/// 用于渲染时将游戏坐标转换为实际屏幕坐标。
pub fn logical_to_output(x: i32, y: i32) -> (i32, i32) {
    let state = display().read();
    let scale = state.dpi_scale;

    if scale == 1.0 {
        return (x, y);
    }

    let output_x = (x as f32 * scale) as i32;
    let output_y = (y as f32 * scale) as i32;

    (output_x, output_y)
}

/// 缩放输出矩形
pub fn scale_output_rect(rect: &Rectangle) -> Rectangle {
    let state = display().read();
    let scale = state.dpi_scale;

    if scale == 1.0 {
        return *rect;
    }

    Rectangle::new(
        (rect.position.x as f32 * scale) as i32,
        (rect.position.y as f32 * scale) as i32,
        (rect.size.width as f32 * scale) as u32,
        (rect.size.height as f32 * scale) as u32,
    )
}

//=============================================================================
// 显示模式计算
//=============================================================================

/// 计算居中偏移
pub fn calculate_centered_offset(window_size: Size, output_size: Size) -> Point {
    Point::new(
        (window_size.width as i32 - output_size.width as i32) / 2,
        (window_size.height as i32 - output_size.height as i32) / 2,
    )
}

/// 计算整数缩放
///
/// 返回 (缩放因子, 输出尺寸)
pub fn calculate_integer_scale(window_size: Size, base_size: Size) -> (u32, Size) {
    if base_size.is_empty() {
        return (1, window_size);
    }

    let scale_x = window_size.width / base_size.width;
    let scale_y = window_size.height / base_size.height;
    let scale = scale_x.min(scale_y).max(1);

    let output = Size::new(base_size.width * scale, base_size.height * scale);
    (scale, output)
}

/// 查找最接近的显示模式
pub fn find_nearest_resolution(preferred: Size, available: &[Size]) -> Option<Size> {
    if available.is_empty() {
        return None;
    }

    let target_area = preferred.area() as i64;

    available
        .iter()
        .min_by_key(|size| {
            let area = size.area() as i64;
            (area - target_area).abs()
        })
        .copied()
}

/// 获取常见分辨率列表
pub fn get_common_resolutions() -> Vec<Size> {
    vec![
        Size::new(640, 480),
        Size::new(800, 600),
        Size::new(1024, 768),
        Size::new(1280, 720),
        Size::new(1280, 800),
        Size::new(1366, 768),
        Size::new(1440, 900),
        Size::new(1600, 900),
        Size::new(1680, 1050),
        Size::new(1920, 1080),
        Size::new(1920, 1200),
        Size::new(2560, 1440),
        Size::new(2560, 1600),
        Size::new(3840, 2160),
    ]
}

/// 计算保持宽高比的缩放尺寸
pub fn calculate_scaled_size(original: Size, target: Size, maintain_aspect: bool) -> Size {
    if !maintain_aspect {
        return target;
    }

    if original.is_empty() {
        return target;
    }

    let scale_x = target.width as f64 / original.width as f64;
    let scale_y = target.height as f64 / original.height as f64;
    let scale = scale_x.min(scale_y);

    Size::new(
        (original.width as f64 * scale) as u32,
        (original.height as f64 * scale) as u32,
    )
}

//=============================================================================
// 测试
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_new() {
        let size = Size::new(800, 600);
        assert_eq!(size.width, 800);
        assert_eq!(size.height, 600);
    }

    #[test]
    fn test_size_zero() {
        let size = Size::zero();
        assert_eq!(size.width, 0);
        assert_eq!(size.height, 0);
        assert!(size.is_empty());
    }

    #[test]
    fn test_size_area() {
        let size = Size::new(100, 200);
        assert_eq!(size.area(), 20000);
    }

    #[test]
    fn test_point_new() {
        let point = Point::new(10, 20);
        assert_eq!(point.x, 10);
        assert_eq!(point.y, 20);
    }

    #[test]
    fn test_rectangle_new() {
        let rect = Rectangle::new(10, 20, 100, 200);
        assert_eq!(rect.x(), 10);
        assert_eq!(rect.y(), 20);
        assert_eq!(rect.width(), 100);
        assert_eq!(rect.height(), 200);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 220);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(0, 0, 100, 100);

        assert!(rect.contains(Point::new(0, 0)));
        assert!(rect.contains(Point::new(50, 50)));
        assert!(rect.contains(Point::new(99, 99)));

        assert!(!rect.contains(Point::new(-1, 0)));
        assert!(!rect.contains(Point::new(0, -1)));
        assert!(!rect.contains(Point::new(100, 0)));
        assert!(!rect.contains(Point::new(0, 100)));
    }

    #[test]
    fn test_calculate_centered_offset() {
        let window = Size::new(1920, 1080);
        let output = Size::new(640, 480);
        let offset = calculate_centered_offset(window, output);

        assert_eq!(offset.x, 640);  // (1920 - 640) / 2
        assert_eq!(offset.y, 300);  // (1080 - 480) / 2
    }

    #[test]
    fn test_calculate_integer_scale() {
        let window = Size::new(1920, 1080);
        let base = Size::new(640, 480);
        let (scale, output) = calculate_integer_scale(window, base);

        assert_eq!(scale, 2);  // min(1920/640, 1080/480) = min(3, 2.25) = 2
        assert_eq!(output.width, 1280);
        assert_eq!(output.height, 960);
    }

    #[test]
    fn test_calculate_integer_scale_exact() {
        let window = Size::new(1280, 960);
        let base = Size::new(640, 480);
        let (scale, output) = calculate_integer_scale(window, base);

        assert_eq!(scale, 2);
        assert_eq!(output.width, 1280);
        assert_eq!(output.height, 960);
    }

    #[test]
    fn test_find_nearest_resolution() {
        let resolutions = get_common_resolutions();

        // 精确匹配
        let nearest = find_nearest_resolution(Size::new(1920, 1080), &resolutions);
        assert_eq!(nearest, Some(Size::new(1920, 1080)));

        // 接近匹配
        let nearest = find_nearest_resolution(Size::new(1900, 1060), &resolutions);
        assert_eq!(nearest, Some(Size::new(1920, 1080)));
    }

    #[test]
    fn test_find_nearest_resolution_empty() {
        let resolutions: Vec<Size> = vec![];
        let nearest = find_nearest_resolution(Size::new(1920, 1080), &resolutions);
        assert!(nearest.is_none());
    }

    #[test]
    fn test_calculate_scaled_size() {
        let original = Size::new(640, 480);
        let target = Size::new(1920, 1080);

        let scaled = calculate_scaled_size(original, target, true);

        // 宽高比 4:3 -> 在 1920x1080 中应该是 1440x1080
        assert_eq!(scaled.width, 1440);
        assert_eq!(scaled.height, 1080);
    }

    #[test]
    fn test_calculate_scaled_size_no_aspect() {
        let original = Size::new(640, 480);
        let target = Size::new(1920, 1080);

        let scaled = calculate_scaled_size(original, target, false);

        assert_eq!(scaled.width, 1920);
        assert_eq!(scaled.height, 1080);
    }

    #[test]
    fn test_output_to_logical_no_scale() {
        // 默认缩放为 1.0
        let (x, y) = output_to_logical(100, 200);
        assert_eq!(x, 100);
        assert_eq!(y, 200);
    }

    #[test]
    fn test_logical_to_output_no_scale() {
        let (x, y) = logical_to_output(100, 200);
        assert_eq!(x, 100);
        assert_eq!(y, 200);
    }

    #[test]
    fn test_display_state_default() {
        let state = DisplayState::default();
        assert_eq!(state.screen_width, 640);
        assert_eq!(state.screen_height, 480);
        assert_eq!(state.viewport_height, 480);
        assert!(!state.fullscreen);
        assert_eq!(state.dpi_scale, 1.0);
    }

    #[test]
    fn test_get_common_resolutions() {
        let resolutions = get_common_resolutions();
        assert!(!resolutions.is_empty());
        assert!(resolutions.contains(&Size::new(640, 480)));
        assert!(resolutions.contains(&Size::new(1920, 1080)));
    }
}
