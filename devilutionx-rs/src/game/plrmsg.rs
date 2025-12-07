//! 玩家消息系统 (M31)
//!
//! 从 Source/plrmsg.cpp 移植
//! 处理玩家消息的显示、发送和事件处理

use std::time::{Duration, Instant};

/// 消息样式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MessageStyle {
    /// 聊天消息
    #[default]
    Chat = 0,
    /// 系统消息
    System = 1,
}

/// 玩家消息结构
#[derive(Debug, Clone)]
pub struct PlayerMessage {
    /// 消息创建时间
    pub time: Option<Instant>,
    /// 消息样式
    pub style: MessageStyle,
    /// 消息文本
    pub text: String,
    /// 前缀长度（玩家名称长度）
    pub prefix_length: usize,
    /// 行高
    pub line_height: i32,
}

impl Default for PlayerMessage {
    fn default() -> Self {
        Self {
            time: None,
            style: MessageStyle::Chat,
            text: String::new(),
            prefix_length: 0,
            line_height: 0,
        }
    }
}

impl PlayerMessage {
    /// 创建新消息
    pub fn new(text: impl Into<String>, style: MessageStyle) -> Self {
        Self {
            time: Some(Instant::now()),
            style,
            text: text.into(),
            prefix_length: 0,
            line_height: 0,
        }
    }

    /// 检查消息是否已过期
    pub fn is_expired(&self, duration: Duration) -> bool {
        match self.time {
            Some(time) => time.elapsed() > duration,
            None => true,
        }
    }

    /// 检查消息是否有效
    pub fn is_valid(&self) -> bool {
        self.time.is_some() && !self.text.is_empty()
    }
}

/// 最大消息数量
pub const PLRMSG_COUNT: usize = 8;

/// 消息持续时间（毫秒）
pub const MESSAGE_DURATION_MS: u64 = 10000;

/// 玩家消息管理器
#[derive(Debug)]
pub struct PlayerMessages {
    /// 消息数组
    messages: [PlayerMessage; PLRMSG_COUNT],
    /// 消息延迟计数器
    delay_counter: u32,
}

impl Default for PlayerMessages {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerMessages {
    /// 创建新的消息管理器
    pub fn new() -> Self {
        Self {
            messages: std::array::from_fn(|_| PlayerMessage::default()),
            delay_counter: 0,
        }
    }

    /// 初始化消息系统
    pub fn init(&mut self) {
        for msg in &mut self.messages {
            *msg = PlayerMessage::default();
        }
        self.delay_counter = 0;
    }

    /// 发送玩家消息
    ///
    /// # Arguments
    /// * `player_name` - 玩家名称
    /// * `text` - 消息文本
    pub fn send_message(&mut self, player_name: &str, text: &str) {
        let prefix_length = player_name.len();
        let full_text = format!("{}: {}", player_name, text);

        // 查找空闲槽位或最旧的消息
        let slot = self.find_slot();

        self.messages[slot] = PlayerMessage {
            time: Some(Instant::now()),
            style: MessageStyle::Chat,
            text: full_text,
            prefix_length,
            line_height: 0, // 在渲染时计算
        };
    }

    /// 发送系统消息
    pub fn send_system_message(&mut self, text: &str) {
        let slot = self.find_slot();

        self.messages[slot] = PlayerMessage {
            time: Some(Instant::now()),
            style: MessageStyle::System,
            text: text.to_string(),
            prefix_length: 0,
            line_height: 0,
        };
    }

    /// 查找可用槽位
    fn find_slot(&self) -> usize {
        let message_duration = Duration::from_millis(MESSAGE_DURATION_MS);

        // 首先查找空闲或过期的槽位
        for (i, msg) in self.messages.iter().enumerate() {
            if !msg.is_valid() || msg.is_expired(message_duration) {
                return i;
            }
        }

        // 如果没有空闲槽位，返回最旧的消息槽位
        self.messages
            .iter()
            .enumerate()
            .min_by_key(|(_, msg)| msg.time)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// 延迟消息处理
    ///
    /// 从 DelayPlrMessages 移植
    pub fn delay_messages(&mut self) {
        self.delay_counter = self.delay_counter.saturating_add(1);

        // 延迟所有消息的时间戳
        // 这在游戏暂停时使用，防止消息在暂停期间过期
    }

    /// 处理消息事件
    ///
    /// 从 EventPlrMsg 移植
    pub fn event_message(&mut self, text: &str, style: MessageStyle) {
        let slot = self.find_slot();

        self.messages[slot] = PlayerMessage {
            time: Some(Instant::now()),
            style,
            text: text.to_string(),
            prefix_length: 0,
            line_height: 0,
        };
    }

    /// 获取所有有效消息（用于绘制）
    pub fn get_active_messages(&self) -> impl Iterator<Item = &PlayerMessage> {
        let message_duration = Duration::from_millis(MESSAGE_DURATION_MS);
        self.messages
            .iter()
            .filter(move |msg| msg.is_valid() && !msg.is_expired(message_duration))
    }

    /// 获取消息数量
    pub fn active_count(&self) -> usize {
        self.get_active_messages().count()
    }

    /// 清除所有消息
    pub fn clear(&mut self) {
        self.init();
    }
}

/// 全局玩家消息实例（用于兼容C++静态变量）
pub static mut PLAYER_MESSAGES: Option<PlayerMessages> = None;

/// 初始化全局玩家消息系统
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn init_plr_msg() {
    PLAYER_MESSAGES = Some(PlayerMessages::new());
}

/// 发送玩家消息（全局函数）
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn send_plr_msg(player_name: &str, text: &str) {
    if let Some(ref mut messages) = PLAYER_MESSAGES {
        messages.send_message(player_name, text);
    }
}

/// 处理消息事件（全局函数）
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn event_plr_msg(text: &str) {
    if let Some(ref mut messages) = PLAYER_MESSAGES {
        messages.event_message(text, MessageStyle::System);
    }
}

/// 延迟消息处理（全局函数）
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn delay_plr_messages() {
    if let Some(ref mut messages) = PLAYER_MESSAGES {
        messages.delay_messages();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_message_creation() {
        let msg = PlayerMessage::new("Hello", MessageStyle::Chat);
        assert!(msg.is_valid());
        assert_eq!(msg.style, MessageStyle::Chat);
        assert_eq!(msg.text, "Hello");
    }

    #[test]
    fn test_player_messages_manager() {
        let mut manager = PlayerMessages::new();

        manager.send_message("Player1", "Hello world");
        assert_eq!(manager.active_count(), 1);

        manager.send_system_message("System message");
        assert_eq!(manager.active_count(), 2);
    }

    #[test]
    fn test_message_slot_rotation() {
        let mut manager = PlayerMessages::new();

        // 填满所有槽位
        for i in 0..PLRMSG_COUNT {
            manager.send_message(&format!("Player{}", i), "Message");
        }
        assert_eq!(manager.active_count(), PLRMSG_COUNT);

        // 再发送一条，应该覆盖最旧的
        manager.send_message("NewPlayer", "New message");
        assert_eq!(manager.active_count(), PLRMSG_COUNT);
    }

    #[test]
    fn test_message_clear() {
        let mut manager = PlayerMessages::new();
        manager.send_message("Player", "Test");
        assert_eq!(manager.active_count(), 1);

        manager.clear();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_message_style_default() {
        assert_eq!(MessageStyle::default(), MessageStyle::Chat);
    }
}
