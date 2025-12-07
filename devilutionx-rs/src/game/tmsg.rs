//! 定时消息系统 (M36)
//!
//! 从 Source/tmsg.cpp 移植
//! 处理带延迟的网络消息传输

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 定时消息结构
#[derive(Debug, Clone)]
pub struct TimedMessage {
    /// 消息发送时间
    pub send_time: Instant,
    /// 消息数据
    pub body: Vec<u8>,
}

impl TimedMessage {
    /// 创建新的定时消息
    pub fn new(delay: Duration, data: &[u8]) -> Self {
        Self {
            send_time: Instant::now() + delay,
            body: data.to_vec(),
        }
    }

    /// 检查消息是否可以发送
    pub fn is_ready(&self) -> bool {
        Instant::now() >= self.send_time
    }

    /// 获取消息长度
    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// 检查消息是否为空
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }
}

/// 定时消息管理器
#[derive(Debug, Default)]
pub struct TimedMessageQueue {
    /// 消息队列
    messages: VecDeque<TimedMessage>,
    /// 默认延迟（tick数）
    pub tick_delay: u32,
}

impl TimedMessageQueue {
    /// 创建新的消息队列
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            tick_delay: 50, // 默认50ms每tick
        }
    }

    /// 设置tick延迟
    pub fn set_tick_delay(&mut self, delay: u32) {
        self.tick_delay = delay;
    }

    /// 添加定时消息
    ///
    /// 从 tmsg_add 移植
    pub fn add(&mut self, data: &[u8]) {
        let delay = Duration::from_millis((self.tick_delay * 10) as u64);
        self.messages.push_back(TimedMessage::new(delay, data));
    }

    /// 添加带自定义延迟的消息
    pub fn add_with_delay(&mut self, data: &[u8], delay: Duration) {
        self.messages.push_back(TimedMessage::new(delay, data));
    }

    /// 获取就绪的消息
    ///
    /// 从 tmsg_get 移植
    pub fn get(&mut self) -> Option<Vec<u8>> {
        if self.messages.is_empty() {
            return None;
        }

        // 检查队首消息是否就绪
        if let Some(front) = self.messages.front() {
            if front.is_ready() {
                return self.messages.pop_front().map(|m| m.body);
            }
        }

        None
    }

    /// 获取就绪消息并返回长度
    ///
    /// 兼容C++接口
    pub fn get_with_len(&mut self) -> (Option<Vec<u8>>, u8) {
        if let Some(data) = self.get() {
            let len = data.len().min(255) as u8;
            (Some(data), len)
        } else {
            (None, 0)
        }
    }

    /// 开始消息队列（清空确认为空）
    ///
    /// 从 tmsg_start 移植
    pub fn start(&mut self) {
        debug_assert!(
            self.messages.is_empty(),
            "Message queue should be empty on start"
        );
    }

    /// 清理消息队列
    ///
    /// 从 tmsg_cleanup 移植
    pub fn cleanup(&mut self) {
        self.messages.clear();
    }

    /// 获取队列中的消息数量
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// 获取就绪消息数量
    pub fn ready_count(&self) -> usize {
        self.messages.iter().filter(|m| m.is_ready()).count()
    }

    /// 清除所有就绪消息并返回它们
    pub fn drain_ready(&mut self) -> Vec<Vec<u8>> {
        let mut ready = Vec::new();
        while let Some(data) = self.get() {
            ready.push(data);
        }
        ready
    }
}

/// 全局定时消息队列
pub static mut TIMED_MSG_QUEUE: Option<TimedMessageQueue> = None;

/// 初始化定时消息系统
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn tmsg_start() {
    TIMED_MSG_QUEUE = Some(TimedMessageQueue::new());
    if let Some(ref mut queue) = TIMED_MSG_QUEUE {
        queue.start();
    }
}

/// 添加定时消息
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn tmsg_add(data: &[u8]) {
    if let Some(ref mut queue) = TIMED_MSG_QUEUE {
        queue.add(data);
    }
}

/// 获取就绪消息
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn tmsg_get() -> (Option<Vec<u8>>, u8) {
    if let Some(ref mut queue) = TIMED_MSG_QUEUE {
        queue.get_with_len()
    } else {
        (None, 0)
    }
}

/// 清理定时消息系统
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn tmsg_cleanup() {
    if let Some(ref mut queue) = TIMED_MSG_QUEUE {
        queue.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_timed_message_creation() {
        let data = vec![1, 2, 3, 4];
        let msg = TimedMessage::new(Duration::from_millis(100), &data);
        assert_eq!(msg.len(), 4);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_message_not_ready_immediately() {
        let data = vec![1, 2, 3];
        let msg = TimedMessage::new(Duration::from_secs(10), &data);
        assert!(!msg.is_ready());
    }

    #[test]
    fn test_message_ready_after_delay() {
        let data = vec![1, 2, 3];
        let msg = TimedMessage::new(Duration::from_millis(10), &data);
        sleep(Duration::from_millis(20));
        assert!(msg.is_ready());
    }

    #[test]
    fn test_queue_creation() {
        let queue = TimedMessageQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_add_get() {
        let mut queue = TimedMessageQueue::new();
        queue.set_tick_delay(1); // 很短的延迟

        let data = vec![1, 2, 3, 4, 5];
        queue.add(&data);

        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        // 等待消息就绪
        sleep(Duration::from_millis(20));

        let result = queue.get();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_cleanup() {
        let mut queue = TimedMessageQueue::new();
        queue.add(&[1, 2, 3]);
        queue.add(&[4, 5, 6]);

        assert_eq!(queue.len(), 2);

        queue.cleanup();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_get_with_len() {
        let mut queue = TimedMessageQueue::new();
        queue.set_tick_delay(1);
        queue.add(&[1, 2, 3, 4]);

        sleep(Duration::from_millis(20));

        let (data, len) = queue.get_with_len();
        assert!(data.is_some());
        assert_eq!(len, 4);
    }

    #[test]
    fn test_empty_queue_get() {
        let mut queue = TimedMessageQueue::new();
        let (data, len) = queue.get_with_len();
        assert!(data.is_none());
        assert_eq!(len, 0);
    }

    #[test]
    fn test_drain_ready() {
        let mut queue = TimedMessageQueue::new();
        queue.set_tick_delay(1);
        queue.add(&[1, 2]);
        queue.add(&[3, 4]);
        queue.add(&[5, 6]);

        sleep(Duration::from_millis(20));

        let ready = queue.drain_ready();
        assert_eq!(ready.len(), 3);
        assert!(queue.is_empty());
    }
}
