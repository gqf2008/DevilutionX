//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Backbuffer State Management - 后备缓冲区状态管理
//!
//! 移植自 Source/engine/backbuffer_state.hpp/cpp
//!
//! 本模块管理：
//! - 多缓冲区的重绘状态 (支持双缓冲/三缓冲)
//! - 面板组件级别的脏标记
//! - 光标绘制的背景保存
//!
//! ## 模块依赖
//! - rectangle.rs: Rectangle 类型
//! - (surface 通过指针标识缓冲区，这里用 usize 代替)

use std::sync::{Mutex, OnceLock};

use super::rectangle::Rectangle;

//=============================================================================
// 常量
//=============================================================================

/// 光标背景缓冲区大小
const CURSOR_BEHIND_BUFFER_SIZE: usize = 8192;

/// 面板组件数量
const NUM_PANEL_COMPONENTS: usize = 4;

//=============================================================================
// PanelDrawComponent - 面板绘制组件
//=============================================================================

/// 面板绘制组件枚举
///
/// 用于细粒度的组件重绘控制
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PanelDrawComponent {
    /// 生命值显示
    Health = 0,
    /// 法力值显示
    Mana = 1,
    /// 控制按钮
    ControlButtons = 2,
    /// 腰带物品栏
    Belt = 3,
}

impl PanelDrawComponent {
    /// 第一个组件
    pub const FIRST: Self = Self::Health;
    /// 最后一个组件
    pub const LAST: Self = Self::Belt;

    /// 从索引转换
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Health),
            1 => Some(Self::Mana),
            2 => Some(Self::ControlButtons),
            3 => Some(Self::Belt),
            _ => None,
        }
    }

    /// 转换为索引
    pub fn to_index(self) -> usize {
        self as usize
    }

    /// 迭代所有组件
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Health, Self::Mana, Self::ControlButtons, Self::Belt].into_iter()
    }
}

//=============================================================================
// DrawnCursor - 已绘制的光标状态
//=============================================================================

/// 已绘制的光标
///
/// 保存光标绘制前的背景像素，用于擦除光标时恢复
#[derive(Clone)]
pub struct DrawnCursor {
    /// 光标矩形区域
    pub rect: Rectangle,
    /// 光标后面的像素数据
    pub behind_buffer: Vec<u8>,
}

impl Default for DrawnCursor {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawnCursor {
    /// 创建新的光标状态
    pub fn new() -> Self {
        Self {
            rect: Rectangle::default(),
            behind_buffer: vec![0u8; CURSOR_BEHIND_BUFFER_SIZE],
        }
    }

    /// 清空背景缓冲
    pub fn clear(&mut self) {
        self.rect = Rectangle::default();
        self.behind_buffer.fill(0);
    }

    /// 检查是否有保存的背景
    pub fn has_behind(&self) -> bool {
        self.rect.size.width > 0 && self.rect.size.height > 0
    }
}

//=============================================================================
// RedrawLevel - 重绘级别
//=============================================================================

/// 重绘级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedrawLevel {
    /// 无需重绘
    #[default]
    None,
    /// 仅重绘视口
    ViewportOnly,
    /// 重绘所有
    All,
}

//=============================================================================
// RedrawState - 重绘状态
//=============================================================================

/// 重绘状态
#[derive(Clone)]
struct RedrawState {
    /// 重绘级别
    redraw: RedrawLevel,
    /// 各组件的重绘标记
    redraw_components: [bool; NUM_PANEL_COMPONENTS],
}

impl Default for RedrawState {
    fn default() -> Self {
        Self {
            redraw: RedrawLevel::All,
            redraw_components: [false; NUM_PANEL_COMPONENTS],
        }
    }
}

//=============================================================================
// BackbufferState - 单个缓冲区状态
//=============================================================================

/// 单个后备缓冲区的状态
#[derive(Clone)]
struct BackbufferState {
    /// 重绘状态
    redraw_state: RedrawState,
    /// 光标状态
    cursor: DrawnCursor,
}

impl Default for BackbufferState {
    fn default() -> Self {
        Self {
            redraw_state: RedrawState::default(),
            cursor: DrawnCursor::new(),
        }
    }
}

//=============================================================================
// BackbufferPtrAndState - 缓冲区指针与状态
//=============================================================================

/// 缓冲区指针与其状态的关联
struct BackbufferPtrAndState {
    /// 缓冲区指针 (用 usize 表示，对应 C++ 的 void*)
    ptr: usize,
    /// 该缓冲区的状态
    state: BackbufferState,
}

//=============================================================================
// 全局状态管理
//=============================================================================

/// 全局状态容器
struct BackbufferStates {
    /// 所有缓冲区的状态列表
    states: Vec<BackbufferPtrAndState>,
    /// 当前活动的 surface 指针 (模拟 C++ 的 PalSurface->pixels)
    current_surface_ptr: usize,
}

impl BackbufferStates {
    fn new() -> Self {
        Self {
            states: Vec::new(),
            current_surface_ptr: 0,
        }
    }

    /// 获取当前缓冲区的状态，如不存在则创建
    fn get_current_state(&mut self) -> &mut BackbufferState {
        let ptr = self.current_surface_ptr;

        // 查找现有状态
        if let Some(pos) = self.states.iter().position(|s| s.ptr == ptr) {
            return &mut self.states[pos].state;
        }

        // 创建新状态
        self.states.push(BackbufferPtrAndState {
            ptr,
            state: BackbufferState::default(),
        });
        &mut self.states.last_mut().unwrap().state
    }

    /// 清空所有状态
    fn clear(&mut self) {
        self.states.clear();
    }

    /// 设置当前 surface 指针
    fn set_current_surface(&mut self, ptr: usize) {
        self.current_surface_ptr = ptr;
    }
}

/// 全局状态实例
static BACKBUFFER_STATES: OnceLock<Mutex<BackbufferStates>> = OnceLock::new();

fn states() -> &'static Mutex<BackbufferStates> {
    BACKBUFFER_STATES.get_or_init(|| Mutex::new(BackbufferStates::new()))
}

//=============================================================================
// 公共 API
//=============================================================================

/// 初始化后备缓冲区状态
///
/// 清空所有状态，通常在游戏初始化或重新创建窗口时调用
pub fn init_backbuffer_state() {
    states().lock().unwrap().clear();
}

/// 设置当前活动的 surface 指针
///
/// 在 Rust 中，传入 surface 缓冲区的地址作为标识
pub fn set_current_surface(ptr: usize) {
    states().lock().unwrap().set_current_surface(ptr);
}

/// 标记需要重绘所有内容
///
/// 对所有缓冲区设置 RedrawAll 状态
pub fn redraw_everything() {
    let mut guard = states().lock().unwrap();
    for ptr_and_state in &mut guard.states {
        ptr_and_state.state.redraw_state.redraw = RedrawLevel::All;
    }
}

/// 检查是否需要重绘所有内容
pub fn is_redraw_everything() -> bool {
    let mut guard = states().lock().unwrap();
    guard.get_current_state().redraw_state.redraw == RedrawLevel::All
}

/// 标记需要重绘视口
///
/// 仅当状态不是 RedrawAll 时才设置为 ViewportOnly
pub fn redraw_viewport() {
    let mut guard = states().lock().unwrap();
    for ptr_and_state in &mut guard.states {
        if ptr_and_state.state.redraw_state.redraw != RedrawLevel::All {
            ptr_and_state.state.redraw_state.redraw = RedrawLevel::ViewportOnly;
        }
    }
}

/// 检查是否需要重绘视口
pub fn is_redraw_viewport() -> bool {
    let mut guard = states().lock().unwrap();
    guard.get_current_state().redraw_state.redraw == RedrawLevel::ViewportOnly
}

/// 标记重绘完成
///
/// 将当前缓冲区的重绘状态设为 None
pub fn redraw_complete() {
    let mut guard = states().lock().unwrap();
    guard.get_current_state().redraw_state.redraw = RedrawLevel::None;
}

/// 标记指定组件需要重绘
pub fn redraw_component(component: PanelDrawComponent) {
    let mut guard = states().lock().unwrap();
    let index = component.to_index();
    for ptr_and_state in &mut guard.states {
        ptr_and_state.state.redraw_state.redraw_components[index] = true;
    }
}

/// 检查指定组件是否需要重绘
pub fn is_redraw_component(component: PanelDrawComponent) -> bool {
    let mut guard = states().lock().unwrap();
    let index = component.to_index();
    guard.get_current_state().redraw_state.redraw_components[index]
}

/// 标记指定组件重绘完成
pub fn redraw_component_complete(component: PanelDrawComponent) {
    let mut guard = states().lock().unwrap();
    let index = component.to_index();
    guard.get_current_state().redraw_state.redraw_components[index] = false;
}

/// 获取当前缓冲区的光标状态的可变引用
///
/// 注意：这返回一个克隆，修改后需要调用 set_drawn_cursor 写回
pub fn get_drawn_cursor() -> DrawnCursor {
    let mut guard = states().lock().unwrap();
    guard.get_current_state().cursor.clone()
}

/// 设置当前缓冲区的光标状态
pub fn set_drawn_cursor(cursor: DrawnCursor) {
    let mut guard = states().lock().unwrap();
    guard.get_current_state().cursor = cursor;
}

/// 使用回调修改光标状态 (避免克隆开销)
pub fn with_drawn_cursor<F, R>(f: F) -> R
where
    F: FnOnce(&mut DrawnCursor) -> R,
{
    let mut guard = states().lock().unwrap();
    let cursor = &mut guard.get_current_state().cursor;
    f(cursor)
}

//=============================================================================
// 测试
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::point::Point;
    use crate::engine::size::Size;

    fn reset_for_test() {
        init_backbuffer_state();
        set_current_surface(0x1000); // 模拟一个 surface 地址
    }

    #[test]
    fn test_panel_draw_component() {
        assert_eq!(PanelDrawComponent::Health.to_index(), 0);
        assert_eq!(PanelDrawComponent::Mana.to_index(), 1);
        assert_eq!(PanelDrawComponent::ControlButtons.to_index(), 2);
        assert_eq!(PanelDrawComponent::Belt.to_index(), 3);

        assert_eq!(PanelDrawComponent::from_index(0), Some(PanelDrawComponent::Health));
        assert_eq!(PanelDrawComponent::from_index(3), Some(PanelDrawComponent::Belt));
        assert_eq!(PanelDrawComponent::from_index(4), None);
    }

    #[test]
    fn test_panel_draw_component_iter() {
        let components: Vec<_> = PanelDrawComponent::iter().collect();
        assert_eq!(components.len(), 4);
        assert_eq!(components[0], PanelDrawComponent::Health);
        assert_eq!(components[3], PanelDrawComponent::Belt);
    }

    #[test]
    fn test_drawn_cursor() {
        let cursor = DrawnCursor::new();
        assert!(!cursor.has_behind());
        assert_eq!(cursor.behind_buffer.len(), CURSOR_BEHIND_BUFFER_SIZE);
    }

    #[test]
    fn test_drawn_cursor_clear() {
        let mut cursor = DrawnCursor::new();
        cursor.behind_buffer[0] = 0xFF;
        cursor.clear();
        assert_eq!(cursor.behind_buffer[0], 0);
    }

    #[test]
    fn test_init_backbuffer_state() {
        reset_for_test();
        // 初始状态应该是 RedrawAll
        assert!(is_redraw_everything());
    }

    #[test]
    fn test_redraw_complete() {
        reset_for_test();
        assert!(is_redraw_everything());

        redraw_complete();
        assert!(!is_redraw_everything());
        assert!(!is_redraw_viewport());
    }

    #[test]
    fn test_redraw_viewport() {
        reset_for_test();
        redraw_complete();

        redraw_viewport();
        assert!(is_redraw_viewport());
        assert!(!is_redraw_everything());
    }

    #[test]
    fn test_redraw_everything_overrides_viewport() {
        reset_for_test();
        redraw_complete();
        redraw_viewport();

        // RedrawAll 不应被 viewport 覆盖
        redraw_everything();
        assert!(is_redraw_everything());

        // 此时调用 redraw_viewport 不应改变状态
        redraw_viewport();
        assert!(is_redraw_everything());
    }

    #[test]
    fn test_component_redraw() {
        reset_for_test();

        // 初始状态组件不需要重绘
        assert!(!is_redraw_component(PanelDrawComponent::Health));

        // 标记组件需要重绘
        redraw_component(PanelDrawComponent::Health);
        assert!(is_redraw_component(PanelDrawComponent::Health));
        assert!(!is_redraw_component(PanelDrawComponent::Mana));

        // 标记完成
        redraw_component_complete(PanelDrawComponent::Health);
        assert!(!is_redraw_component(PanelDrawComponent::Health));
    }

    #[test]
    fn test_with_drawn_cursor() {
        reset_for_test();

        with_drawn_cursor(|cursor| {
            cursor.rect = Rectangle::new(Point::new(10, 20), Size::new(32, 32));
            cursor.behind_buffer[0] = 0xAB;
        });

        let cursor = get_drawn_cursor();
        assert_eq!(cursor.rect.position.x, 10);
        assert_eq!(cursor.rect.position.y, 20);
        assert_eq!(cursor.behind_buffer[0], 0xAB);
    }

    #[test]
    fn test_multiple_surfaces() {
        init_backbuffer_state();

        // Surface 1
        set_current_surface(0x1000);
        assert!(is_redraw_everything()); // 新 surface 默认 RedrawAll
        redraw_complete();
        assert!(!is_redraw_everything());

        // Surface 2 - 应该有独立状态
        set_current_surface(0x2000);
        assert!(is_redraw_everything()); // 新 surface 默认 RedrawAll

        // 切回 Surface 1 - 应该保持之前的状态
        set_current_surface(0x1000);
        assert!(!is_redraw_everything());
    }

    #[test]
    fn test_redraw_level() {
        assert_eq!(RedrawLevel::default(), RedrawLevel::None);
        assert_ne!(RedrawLevel::All, RedrawLevel::ViewportOnly);
    }
}
