//! 事件处理系统 - 移植自 Source/engine/events.hpp
//!
//! 提供游戏事件处理的抽象层

use std::collections::VecDeque;

/// 键盘修饰键状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModState {
    /// Shift 键按下
    pub shift: bool,
    /// Ctrl 键按下
    pub ctrl: bool,
    /// Alt 键按下
    pub alt: bool,
    /// Caps Lock 启用
    pub caps_lock: bool,
    /// Num Lock 启用
    pub num_lock: bool,
}

impl ModState {
    /// 空修饰状态
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        caps_lock: false,
        num_lock: false,
    };

    /// 从位掩码创建
    pub fn from_bits(bits: u16) -> Self {
        Self {
            shift: bits & 0x0001 != 0,   // KMOD_SHIFT
            ctrl: bits & 0x0040 != 0,    // KMOD_CTRL
            alt: bits & 0x0100 != 0,     // KMOD_ALT
            caps_lock: bits & 0x2000 != 0, // KMOD_CAPS
            num_lock: bits & 0x1000 != 0,  // KMOD_NUM
        }
    }

    /// 转换为位掩码
    pub fn to_bits(&self) -> u16 {
        let mut bits = 0u16;
        if self.shift { bits |= 0x0001; }
        if self.ctrl { bits |= 0x0040; }
        if self.alt { bits |= 0x0100; }
        if self.caps_lock { bits |= 0x2000; }
        if self.num_lock { bits |= 0x1000; }
        bits
    }
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

impl MouseButton {
    /// 从 SDL 按钮索引转换
    pub fn from_sdl(button: u8) -> Option<Self> {
        match button {
            1 => Some(MouseButton::Left),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Right),
            4 => Some(MouseButton::X1),
            5 => Some(MouseButton::X2),
            _ => None,
        }
    }

    /// 转换为 SDL 按钮索引
    pub fn to_sdl(&self) -> u8 {
        match self {
            MouseButton::Left => 1,
            MouseButton::Middle => 2,
            MouseButton::Right => 3,
            MouseButton::X1 => 4,
            MouseButton::X2 => 5,
        }
    }
}

/// 虚拟键码
///
/// 常用键码定义，基于 SDL 键码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyCode {
    Unknown = 0,

    // 字母键
    A = b'a' as u32,
    B = b'b' as u32,
    C = b'c' as u32,
    D = b'd' as u32,
    E = b'e' as u32,
    F = b'f' as u32,
    G = b'g' as u32,
    H = b'h' as u32,
    I = b'i' as u32,
    J = b'j' as u32,
    K = b'k' as u32,
    L = b'l' as u32,
    M = b'm' as u32,
    N = b'n' as u32,
    O = b'o' as u32,
    P = b'p' as u32,
    Q = b'q' as u32,
    R = b'r' as u32,
    S = b's' as u32,
    T = b't' as u32,
    U = b'u' as u32,
    V = b'v' as u32,
    W = b'w' as u32,
    X = b'x' as u32,
    Y = b'y' as u32,
    Z = b'z' as u32,

    // 数字键
    Num0 = b'0' as u32,
    Num1 = b'1' as u32,
    Num2 = b'2' as u32,
    Num3 = b'3' as u32,
    Num4 = b'4' as u32,
    Num5 = b'5' as u32,
    Num6 = b'6' as u32,
    Num7 = b'7' as u32,
    Num8 = b'8' as u32,
    Num9 = b'9' as u32,

    // 功能键
    F1 = 0x4000_003A,
    F2 = 0x4000_003B,
    F3 = 0x4000_003C,
    F4 = 0x4000_003D,
    F5 = 0x4000_003E,
    F6 = 0x4000_003F,
    F7 = 0x4000_0040,
    F8 = 0x4000_0041,
    F9 = 0x4000_0042,
    F10 = 0x4000_0043,
    F11 = 0x4000_0044,
    F12 = 0x4000_0045,

    // 特殊键
    Return = b'\r' as u32,
    Escape = 0x1B,
    Backspace = 0x08,
    Tab = b'\t' as u32,
    Space = b' ' as u32,

    // 方向键
    Right = 0x4000_004F,
    Left = 0x4000_0050,
    Down = 0x4000_0051,
    Up = 0x4000_0052,

    // 编辑键
    Insert = 0x4000_0049,
    Delete = 0x7F,
    Home = 0x4000_004A,
    End = 0x4000_004D,
    PageUp = 0x4000_004B,
    PageDown = 0x4000_004E,

    // 小键盘
    Kp0 = 0x4000_0062,
    Kp1 = 0x4000_0059,
    Kp2 = 0x4000_005A,
    Kp3 = 0x4000_005B,
    Kp4 = 0x4000_005C,
    Kp5 = 0x4000_005D,
    Kp6 = 0x4000_005E,
    Kp7 = 0x4000_005F,
    Kp8 = 0x4000_0060,
    Kp9 = 0x4000_0061,
    KpEnter = 0x4000_0058,
    KpPlus = 0x4000_0057,
    KpMinus = 0x4000_0056,
    KpMultiply = 0x4000_0055,
    KpDivide = 0x4000_0054,

    // 修饰键
    LShift = 0x4000_00E1,
    RShift = 0x4000_00E5,
    LCtrl = 0x4000_00E0,
    RCtrl = 0x4000_00E4,
    LAlt = 0x4000_00E2,
    RAlt = 0x4000_00E6,
}

impl KeyCode {
    /// 从 u32 值创建
    pub fn from_u32(value: u32) -> Self {
        // 安全转换常用键
        match value {
            x if x == KeyCode::A as u32 => KeyCode::A,
            x if x == KeyCode::B as u32 => KeyCode::B,
            x if x == KeyCode::C as u32 => KeyCode::C,
            x if x == KeyCode::D as u32 => KeyCode::D,
            x if x == KeyCode::E as u32 => KeyCode::E,
            x if x == KeyCode::F as u32 => KeyCode::F,
            x if x == KeyCode::G as u32 => KeyCode::G,
            x if x == KeyCode::H as u32 => KeyCode::H,
            x if x == KeyCode::I as u32 => KeyCode::I,
            x if x == KeyCode::J as u32 => KeyCode::J,
            x if x == KeyCode::K as u32 => KeyCode::K,
            x if x == KeyCode::L as u32 => KeyCode::L,
            x if x == KeyCode::M as u32 => KeyCode::M,
            x if x == KeyCode::N as u32 => KeyCode::N,
            x if x == KeyCode::O as u32 => KeyCode::O,
            x if x == KeyCode::P as u32 => KeyCode::P,
            x if x == KeyCode::Q as u32 => KeyCode::Q,
            x if x == KeyCode::R as u32 => KeyCode::R,
            x if x == KeyCode::S as u32 => KeyCode::S,
            x if x == KeyCode::T as u32 => KeyCode::T,
            x if x == KeyCode::U as u32 => KeyCode::U,
            x if x == KeyCode::V as u32 => KeyCode::V,
            x if x == KeyCode::W as u32 => KeyCode::W,
            x if x == KeyCode::X as u32 => KeyCode::X,
            x if x == KeyCode::Y as u32 => KeyCode::Y,
            x if x == KeyCode::Z as u32 => KeyCode::Z,
            x if x == KeyCode::Escape as u32 => KeyCode::Escape,
            x if x == KeyCode::Return as u32 => KeyCode::Return,
            x if x == KeyCode::Space as u32 => KeyCode::Space,
            x if x == KeyCode::Up as u32 => KeyCode::Up,
            x if x == KeyCode::Down as u32 => KeyCode::Down,
            x if x == KeyCode::Left as u32 => KeyCode::Left,
            x if x == KeyCode::Right as u32 => KeyCode::Right,
            _ => KeyCode::Unknown,
        }
    }
}

/// 游戏事件类型
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// 退出事件
    Quit,

    /// 键盘按下
    KeyDown {
        key: KeyCode,
        scancode: u32,
        repeat: bool,
    },

    /// 键盘释放
    KeyUp {
        key: KeyCode,
        scancode: u32,
    },

    /// 鼠标移动
    MouseMotion {
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
    },

    /// 鼠标按钮按下
    MouseButtonDown {
        button: MouseButton,
        x: i32,
        y: i32,
    },

    /// 鼠标按钮释放
    MouseButtonUp {
        button: MouseButton,
        x: i32,
        y: i32,
    },

    /// 鼠标滚轮
    MouseWheel {
        x: i32,
        y: i32,
    },

    /// 文本输入
    TextInput {
        text: String,
    },

    /// 窗口获得焦点
    WindowFocusGained,

    /// 窗口失去焦点
    WindowFocusLost,

    /// 窗口大小改变
    WindowResized {
        width: i32,
        height: i32,
    },

    /// 游戏手柄按钮按下
    GamepadButtonDown {
        button: u8,
        gamepad_id: u32,
    },

    /// 游戏手柄按钮释放
    GamepadButtonUp {
        button: u8,
        gamepad_id: u32,
    },

    /// 游戏手柄轴移动
    GamepadAxisMotion {
        axis: u8,
        value: i16,
        gamepad_id: u32,
    },

    /// 自定义事件
    Custom {
        event_type: u32,
        data1: i64,
        data2: i64,
    },
}

/// 带修饰状态的事件
#[derive(Debug, Clone)]
pub struct EventWithMod {
    pub event: GameEvent,
    pub mod_state: ModState,
}

/// 事件处理器类型
pub type EventHandler = fn(&GameEvent, ModState);

/// 事件管理器
pub struct EventManager {
    /// 当前事件处理器
    current_handler: Option<EventHandler>,
    /// 事件队列
    event_queue: VecDeque<EventWithMod>,
    /// 当前鼠标位置
    mouse_position: (i32, i32),
}

impl EventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        Self {
            current_handler: None,
            event_queue: VecDeque::new(),
            mouse_position: (0, 0),
        }
    }

    /// 设置事件处理器
    ///
    /// 返回之前的处理器
    pub fn set_event_handler(&mut self, handler: Option<EventHandler>) -> Option<EventHandler> {
        let previous = self.current_handler;
        self.current_handler = handler;
        previous
    }

    /// 推送事件
    pub fn push_event(&mut self, event: GameEvent, mod_state: ModState) {
        // 更新鼠标位置
        match &event {
            GameEvent::MouseMotion { x, y, .. } => {
                self.mouse_position = (*x, *y);
            }
            GameEvent::MouseButtonDown { x, y, .. } |
            GameEvent::MouseButtonUp { x, y, .. } => {
                self.mouse_position = (*x, *y);
            }
            _ => {}
        }

        self.event_queue.push_back(EventWithMod { event, mod_state });
    }

    /// 获取下一个事件
    pub fn poll_event(&mut self) -> Option<EventWithMod> {
        self.event_queue.pop_front()
    }

    /// 处理下一个事件
    pub fn handle_next_event(&mut self) -> bool {
        if let Some(event_with_mod) = self.poll_event() {
            if let Some(handler) = self.current_handler {
                handler(&event_with_mod.event, event_with_mod.mod_state);
            }
            true
        } else {
            false
        }
    }

    /// 处理所有待处理事件
    pub fn process_all_events(&mut self) {
        while self.handle_next_event() {}
    }

    /// 获取当前鼠标位置
    pub fn mouse_position(&self) -> (i32, i32) {
        self.mouse_position
    }

    /// 清空事件队列
    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    /// 队列中的事件数量
    pub fn pending_events(&self) -> usize {
        self.event_queue.len()
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 检测是否为可打印字符键
pub fn is_printable_key(key: KeyCode) -> bool {
    matches!(key,
        KeyCode::A | KeyCode::B | KeyCode::C | KeyCode::D | KeyCode::E |
        KeyCode::F | KeyCode::G | KeyCode::H | KeyCode::I | KeyCode::J |
        KeyCode::K | KeyCode::L | KeyCode::M | KeyCode::N | KeyCode::O |
        KeyCode::P | KeyCode::Q | KeyCode::R | KeyCode::S | KeyCode::T |
        KeyCode::U | KeyCode::V | KeyCode::W | KeyCode::X | KeyCode::Y | KeyCode::Z |
        KeyCode::Num0 | KeyCode::Num1 | KeyCode::Num2 | KeyCode::Num3 | KeyCode::Num4 |
        KeyCode::Num5 | KeyCode::Num6 | KeyCode::Num7 | KeyCode::Num8 | KeyCode::Num9 |
        KeyCode::Space
    )
}

/// 获取按键对应的字符
pub fn key_to_char(key: KeyCode, shift: bool) -> Option<char> {
    let c = match key {
        KeyCode::A => 'a',
        KeyCode::B => 'b',
        KeyCode::C => 'c',
        KeyCode::D => 'd',
        KeyCode::E => 'e',
        KeyCode::F => 'f',
        KeyCode::G => 'g',
        KeyCode::H => 'h',
        KeyCode::I => 'i',
        KeyCode::J => 'j',
        KeyCode::K => 'k',
        KeyCode::L => 'l',
        KeyCode::M => 'm',
        KeyCode::N => 'n',
        KeyCode::O => 'o',
        KeyCode::P => 'p',
        KeyCode::Q => 'q',
        KeyCode::R => 'r',
        KeyCode::S => 's',
        KeyCode::T => 't',
        KeyCode::U => 'u',
        KeyCode::V => 'v',
        KeyCode::W => 'w',
        KeyCode::X => 'x',
        KeyCode::Y => 'y',
        KeyCode::Z => 'z',
        KeyCode::Num0 => '0',
        KeyCode::Num1 => '1',
        KeyCode::Num2 => '2',
        KeyCode::Num3 => '3',
        KeyCode::Num4 => '4',
        KeyCode::Num5 => '5',
        KeyCode::Num6 => '6',
        KeyCode::Num7 => '7',
        KeyCode::Num8 => '8',
        KeyCode::Num9 => '9',
        KeyCode::Space => ' ',
        _ => return None,
    };

    if shift && c.is_ascii_lowercase() {
        Some(c.to_ascii_uppercase())
    } else {
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_state() {
        let state = ModState::NONE;
        assert!(!state.shift);
        assert!(!state.ctrl);
        assert!(!state.alt);

        let bits = 0x0041; // SHIFT + CTRL
        let state = ModState::from_bits(bits);
        assert!(state.shift);
        assert!(state.ctrl);
        assert!(!state.alt);
    }

    #[test]
    fn test_mouse_button() {
        assert_eq!(MouseButton::from_sdl(1), Some(MouseButton::Left));
        assert_eq!(MouseButton::from_sdl(3), Some(MouseButton::Right));
        assert_eq!(MouseButton::from_sdl(99), None);

        assert_eq!(MouseButton::Left.to_sdl(), 1);
        assert_eq!(MouseButton::Right.to_sdl(), 3);
    }

    #[test]
    fn test_event_manager() {
        let mut manager = EventManager::new();
        assert_eq!(manager.pending_events(), 0);

        manager.push_event(
            GameEvent::MouseMotion { x: 100, y: 200, dx: 5, dy: 10 },
            ModState::NONE,
        );
        assert_eq!(manager.pending_events(), 1);
        assert_eq!(manager.mouse_position(), (100, 200));

        let event = manager.poll_event().unwrap();
        assert!(matches!(event.event, GameEvent::MouseMotion { x: 100, y: 200, .. }));
        assert_eq!(manager.pending_events(), 0);
    }

    #[test]
    fn test_key_to_char() {
        assert_eq!(key_to_char(KeyCode::A, false), Some('a'));
        assert_eq!(key_to_char(KeyCode::A, true), Some('A'));
        assert_eq!(key_to_char(KeyCode::Space, false), Some(' '));
        assert_eq!(key_to_char(KeyCode::Escape, false), None);
    }

    #[test]
    fn test_is_printable_key() {
        assert!(is_printable_key(KeyCode::A));
        assert!(is_printable_key(KeyCode::Num5));
        assert!(is_printable_key(KeyCode::Space));
        assert!(!is_printable_key(KeyCode::Escape));
        assert!(!is_printable_key(KeyCode::F1));
    }
}
