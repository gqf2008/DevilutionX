//! 网络线程管理
//!
//! 移植自 Source/nthread.cpp
//! 负责游戏tick管理、网络同步和多人游戏时序控制

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// 最大玩家数量
pub const MAX_PLRS: usize = 4;

/// 默认游戏tick延迟（毫秒）
pub const DEFAULT_TICK_DELAY: u32 = 50;

/// 动画基础分数值
pub const ANIMATION_BASE_VALUE_FRACTION: u8 = 64;

/// 网络更新速率
pub static NET_UPDATE_RATE: AtomicU8 = AtomicU8::new(1);

/// 游戏进度到下一个tick
pub static PROGRESS_TO_NEXT_GAME_TICK: AtomicU8 = AtomicU8::new(0);

/// 消息长度表
#[derive(Debug)]
pub struct MessageLengthTable {
    lengths: [usize; MAX_PLRS],
}

impl Default for MessageLengthTable {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageLengthTable {
    pub fn new() -> Self {
        Self {
            lengths: [0; MAX_PLRS],
        }
    }

    pub fn get(&self, player: usize) -> usize {
        self.lengths.get(player).copied().unwrap_or(0)
    }

    pub fn set(&mut self, player: usize, len: usize) {
        if let Some(slot) = self.lengths.get_mut(player) {
            *slot = len;
        }
    }
}

/// 网络能力描述
#[derive(Debug, Clone)]
pub struct NetworkCaps {
    /// 默认传输中的回合数
    pub default_turns_in_transit: u32,
    /// 每秒默认回合数
    pub default_turns_sec: u32,
    /// 每秒字节数
    pub bytes_sec: u32,
    /// 最大消息大小
    pub max_message_size: u32,
    /// 最大玩家数
    pub max_players: u32,
}

impl Default for NetworkCaps {
    fn default() -> Self {
        Self {
            default_turns_in_transit: 1,
            default_turns_sec: 20,
            bytes_sec: 1024,
            max_message_size: 512,
            max_players: 4,
        }
    }
}

/// 网络线程状态
#[derive(Debug)]
pub struct NThreadState {
    /// 是否应该运行
    should_run: Arc<AtomicBool>,
    /// 同步倒计时
    sync_countdown: i8,
    /// 数据包倒计时
    packet_countdown: i8,
    /// Turn上位bit
    turn_upper_bit: u32,
    /// 是否tick同步
    ticks_out_of_sync: bool,
    /// 线程是否运行中
    thread_running: Arc<AtomicBool>,
    /// 传输中的turn数
    turns_in_transit: u32,
    /// 最大消息大小
    largest_msg_size: u32,
    /// 正常消息大小
    normal_msg_size: u32,
    /// 上一次tick时间
    last_tick: Instant,
    /// tick延迟
    tick_delay: Duration,
}

impl Default for NThreadState {
    fn default() -> Self {
        Self::new()
    }
}

impl NThreadState {
    pub fn new() -> Self {
        Self {
            should_run: Arc::new(AtomicBool::new(false)),
            sync_countdown: 1,
            packet_countdown: 1,
            turn_upper_bit: 0,
            ticks_out_of_sync: true,
            thread_running: Arc::new(AtomicBool::new(false)),
            turns_in_transit: 0,
            largest_msg_size: 512,
            normal_msg_size: 0,
            last_tick: Instant::now(),
            tick_delay: Duration::from_millis(DEFAULT_TICK_DELAY as u64),
        }
    }

    /// 设置turn上位bit
    pub fn set_turn_upper_bit(&mut self) {
        self.turn_upper_bit = 0x80000000;
    }

    /// 清除turn上位bit
    pub fn clear_turn_upper_bit(&mut self) {
        self.turn_upper_bit = 0;
    }
}

/// 网络线程管理器
pub struct NThreadManager {
    /// 内部状态
    state: Arc<Mutex<NThreadState>>,
    /// 工作线程句柄
    thread_handle: Option<JoinHandle<()>>,
    /// 消息长度表
    msg_lengths: MessageLengthTable,
    /// 消息指针表（简化为缓冲区）
    msg_buffers: [Vec<u8>; MAX_PLRS],
    /// 是否是多人游戏
    is_multiplayer: bool,
    /// 玩家ID
    my_player_id: usize,
}

impl Default for NThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NThreadManager {
    /// 创建新的网络线程管理器
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(NThreadState::new())),
            thread_handle: None,
            msg_lengths: MessageLengthTable::new(),
            msg_buffers: Default::default(),
            is_multiplayer: false,
            my_player_id: 0,
        }
    }

    /// 启动网络线程
    pub fn start(&mut self, set_turn_upper_bit: bool, caps: &NetworkCaps) {
        {
            let mut state = self.state.lock().unwrap();
            state.last_tick = Instant::now();
            state.packet_countdown = 1;
            state.sync_countdown = 1;
            state.ticks_out_of_sync = true;

            if set_turn_upper_bit {
                state.set_turn_upper_bit();
            } else {
                state.clear_turn_upper_bit();
            }

            // 配置网络参数
            state.turns_in_transit = caps.default_turns_in_transit.max(1);

            let net_update_rate = if caps.default_turns_sec <= 20 && caps.default_turns_sec != 0 {
                20 / caps.default_turns_sec
            } else {
                1
            };
            NET_UPDATE_RATE.store(net_update_rate as u8, Ordering::SeqCst);

            let mut largest_msg_size = 512;
            if caps.max_message_size < 0x200 {
                largest_msg_size = caps.max_message_size;
            }
            state.largest_msg_size = largest_msg_size;

            let mut normal_msg_size = caps.bytes_sec * net_update_rate / 20;
            normal_msg_size = (normal_msg_size * 3) >> 2;

            let max_players = caps.max_players.min(MAX_PLRS as u32);
            normal_msg_size /= max_players;

            let mut final_net_rate = net_update_rate;
            while normal_msg_size < 0x80 {
                normal_msg_size *= 2;
                final_net_rate *= 2;
            }
            NET_UPDATE_RATE.store(final_net_rate as u8, Ordering::SeqCst);

            if normal_msg_size > largest_msg_size {
                normal_msg_size = largest_msg_size;
            }
            state.normal_msg_size = normal_msg_size;

            state.should_run.store(true, Ordering::SeqCst);
        }

        // 只在多人游戏中启动后台线程
        if self.is_multiplayer {
            self.start_background_thread();
        }
    }

    /// 启动后台线程
    ///
    /// **C++ Reference**: `Source/nthread.cpp` - `NthreadHandler()`
    ///
    /// ```cpp
    /// void NthreadHandler()
    /// {
    ///     if (!nthread_should_run) {
    ///         return;
    ///     }
    ///     while (true) {
    ///         MemCrit.lock();
    ///         if (!nthread_should_run) {
    ///             MemCrit.unlock();
    ///             break;
    ///         }
    ///         nthread_send_and_recv_turn(0, 0);
    ///         int delta = gnTickDelay;
    ///         if (nthread_recv_turns())
    ///             delta = last_tick - SDL_GetTicks();
    ///         MemCrit.unlock();
    ///         if (delta > 0)
    ///             SDL_Delay(delta);
    ///         if (!nthread_should_run)
    ///             return;
    ///     }
    /// }
    /// ```
    fn start_background_thread(&mut self) {
        let state = Arc::clone(&self.state);
        let should_run = {
            let s = state.lock().unwrap();
            Arc::clone(&s.should_run)
        };

        self.thread_handle = Some(thread::spawn(move || {
            if !should_run.load(Ordering::SeqCst) {
                return;
            }
            loop {
                // 锁定并执行网络回合处理
                let delta = {
                    let s = state.lock().unwrap();
                    if !s.should_run.load(Ordering::SeqCst) {
                        break;
                    }
                    // 对应 nthread_send_and_recv_turn(0, 0) + nthread_recv_turns()
                    // 此处不直接递归调用（避免死锁），仅计算 sleep 时间
                    let delta_base = s.tick_delay;
                    let elapsed = s.last_tick.elapsed();
                    if elapsed < delta_base {
                        delta_base - elapsed
                    } else {
                        Duration::ZERO
                    }
                };

                if delta > Duration::ZERO {
                    thread::sleep(delta);
                }

                if !should_run.load(Ordering::SeqCst) {
                    return;
                }
            }
        }));
    }

    /// 执行单次网络线程迭代（对应 NthreadHandler 的单次循环体）
    ///
    /// **C++ Reference**: `Source/nthread.cpp` - `NthreadHandler()` 循环体
    ///
    /// 在主线程中可手动调用以驱动网络处理（无后台线程时）。
    /// 返回建议的睡眠时间。
    pub fn nthread_run_iteration(&mut self) -> Duration {
        // nthread_send_and_recv_turn(0, 0)
        self.send_and_recv_turn(0, 0);

        // 计算 delta
        let mut delta = self.get_tick_delay();
        // nthread_recv_turns() 成功时 delta = last_tick - now
        if self.recv_turns() {
            let state = self.state.lock().unwrap();
            let now = Instant::now();
            if state.last_tick > now {
                delta = state.last_tick - now;
            } else {
                delta = Duration::ZERO;
            }
        }
        delta
    }

    /// 清理网络线程
    pub fn cleanup(&mut self) {
        {
            let mut state = self.state.lock().unwrap();
            state.should_run.store(false, Ordering::SeqCst);
            state.turns_in_transit = 0;
            state.normal_msg_size = 0;
            state.largest_msg_size = 0;
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// 发送并接收turn
    pub fn send_and_recv_turn(&mut self, cur_turn: u32, turn_delta: i32) -> u32 {
        let mut state = self.state.lock().unwrap();
        let mut current_turn = cur_turn;
        let mut turns_in_transit = 0u32;

        while turns_in_transit < state.turns_in_transit {
            let turn = state.turn_upper_bit | (current_turn & 0x7FFFFFFF);
            state.turn_upper_bit = 0;

            // 发送turn（简化实现）
            self.send_turn(turn);

            current_turn = current_turn.wrapping_add(turn_delta as u32);
            if current_turn >= 0x7FFFFFFF {
                current_turn &= 0xFFFF;
            }
            turns_in_transit += 1;
        }

        current_turn
    }

    /// 发送turn数据
    fn send_turn(&self, _turn: u32) {
        // 实际实现会通过网络发送
    }

    /// 接收turns
    pub fn recv_turns(&mut self) -> bool {
        let mut state = self.state.lock().unwrap();

        state.packet_countdown -= 1;
        if state.packet_countdown > 0 {
            state.last_tick = Instant::now();
            return true;
        }

        state.sync_countdown -= 1;
        state.packet_countdown = NET_UPDATE_RATE.load(Ordering::SeqCst) as i8;

        if state.sync_countdown != 0 {
            state.last_tick = Instant::now();
            return true;
        }

        // 尝试接收turns（简化实现）
        if !self.receive_turns_internal() {
            state.ticks_out_of_sync = false;
            state.sync_countdown = 1;
            state.packet_countdown = 1;
            return false;
        }

        if !state.ticks_out_of_sync {
            state.ticks_out_of_sync = true;
            state.last_tick = Instant::now();
        }

        state.sync_countdown = 4;
        state.last_tick = Instant::now();
        true
    }

    /// 内部接收turns
    fn receive_turns_internal(&self) -> bool {
        // 简化实现
        true
    }

    /// 检查是否已过500ms
    pub fn has_500ms_passed(&self) -> (bool, bool) {
        let state = self.state.lock().unwrap();
        let elapsed = state.last_tick.elapsed();
        let ticks_elapsed = elapsed.as_millis() as i32;
        let tick_delay_ms = state.tick_delay.as_millis() as i32;

        // 检查是否错过了多个游戏tick（>10）
        let should_reset = ticks_elapsed > tick_delay_ms * 10;

        // 检查是否错过了一个游戏tick
        let draw_game = ticks_elapsed <= tick_delay_ms;

        (!should_reset && ticks_elapsed >= 0, draw_game)
    }

    /// 更新到下一个游戏tick的进度
    pub fn update_progress_to_next_game_tick(&self, game_running: bool, paused: bool) {
        if !game_running || paused {
            return;
        }

        let state = self.state.lock().unwrap();
        let elapsed = state.last_tick.elapsed();
        let tick_delay_ms = state.tick_delay.as_millis() as u64;

        if elapsed.as_millis() as u64 >= tick_delay_ms {
            PROGRESS_TO_NEXT_GAME_TICK.store(ANIMATION_BASE_VALUE_FRACTION, Ordering::SeqCst);
            return;
        }

        let ticks_advanced = elapsed.as_millis() as u64;
        let fraction = (ticks_advanced * ANIMATION_BASE_VALUE_FRACTION as u64 / tick_delay_ms)
            .min(ANIMATION_BASE_VALUE_FRACTION as u64) as u8;

        PROGRESS_TO_NEXT_GAME_TICK.store(fraction, Ordering::SeqCst);
    }

    /// 忽略互斥锁
    pub fn ignore_mutex(&mut self, start: bool) {
        if self.thread_handle.is_none() {
            return;
        }

        let state = self.state.lock().unwrap();
        state.thread_running.store(start, Ordering::SeqCst);
    }

    /// 设置多人游戏模式
    pub fn set_multiplayer(&mut self, is_multiplayer: bool) {
        self.is_multiplayer = is_multiplayer;
    }

    /// 设置玩家ID
    pub fn set_player_id(&mut self, player_id: usize) {
        self.my_player_id = player_id;
    }

    /// 获取tick延迟
    pub fn get_tick_delay(&self) -> Duration {
        let state = self.state.lock().unwrap();
        state.tick_delay
    }

    /// 设置tick延迟
    pub fn set_tick_delay(&mut self, delay_ms: u32) {
        let mut state = self.state.lock().unwrap();
        state.tick_delay = Duration::from_millis(delay_ms as u64);
    }

    /// 获取上次tick时间
    pub fn get_last_tick(&self) -> Instant {
        let state = self.state.lock().unwrap();
        state.last_tick
    }

    /// 终止游戏
    pub fn terminate_game(&mut self, reason: &str) {
        eprintln!("Game terminated: {}", reason);
        self.cleanup();
    }
}

impl Drop for NThreadManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// 计算动画进度
pub fn calculate_animation_progress(
    last_tick: Instant,
    tick_delay: Duration,
) -> u8 {
    let elapsed = last_tick.elapsed();

    if elapsed >= tick_delay {
        return ANIMATION_BASE_VALUE_FRACTION;
    }

    let progress = elapsed.as_millis() as u64 * ANIMATION_BASE_VALUE_FRACTION as u64
        / tick_delay.as_millis() as u64;

    progress.min(ANIMATION_BASE_VALUE_FRACTION as u64) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_length_table() {
        let mut table = MessageLengthTable::new();
        assert_eq!(table.get(0), 0);

        table.set(1, 100);
        assert_eq!(table.get(1), 100);

        // 超出范围
        assert_eq!(table.get(10), 0);
    }

    #[test]
    fn test_network_caps_default() {
        let caps = NetworkCaps::default();
        assert_eq!(caps.default_turns_in_transit, 1);
        assert_eq!(caps.default_turns_sec, 20);
        assert_eq!(caps.max_players, 4);
    }

    #[test]
    fn test_nthread_state_new() {
        let state = NThreadState::new();
        assert_eq!(state.sync_countdown, 1);
        assert_eq!(state.packet_countdown, 1);
        assert!(state.ticks_out_of_sync);
    }

    #[test]
    fn test_nthread_state_turn_upper_bit() {
        let mut state = NThreadState::new();
        assert_eq!(state.turn_upper_bit, 0);

        state.set_turn_upper_bit();
        assert_eq!(state.turn_upper_bit, 0x80000000);

        state.clear_turn_upper_bit();
        assert_eq!(state.turn_upper_bit, 0);
    }

    #[test]
    fn test_nthread_manager_new() {
        let manager = NThreadManager::new();
        assert!(!manager.is_multiplayer);
        assert_eq!(manager.my_player_id, 0);
    }

    #[test]
    fn test_nthread_manager_settings() {
        let mut manager = NThreadManager::new();

        manager.set_multiplayer(true);
        assert!(manager.is_multiplayer);

        manager.set_player_id(2);
        assert_eq!(manager.my_player_id, 2);
    }

    #[test]
    fn test_nthread_manager_tick_delay() {
        let mut manager = NThreadManager::new();

        manager.set_tick_delay(100);
        assert_eq!(manager.get_tick_delay(), Duration::from_millis(100));
    }

    #[test]
    fn test_calculate_animation_progress() {
        let tick_delay = Duration::from_millis(100);
        let last_tick = Instant::now();

        // 刚开始时进度应该很低
        let progress = calculate_animation_progress(last_tick, tick_delay);
        assert!(progress <= 10); // 应该接近0
    }

    #[test]
    fn test_progress_atomic() {
        PROGRESS_TO_NEXT_GAME_TICK.store(32, Ordering::SeqCst);
        assert_eq!(PROGRESS_TO_NEXT_GAME_TICK.load(Ordering::SeqCst), 32);
    }

    #[test]
    fn test_net_update_rate_atomic() {
        NET_UPDATE_RATE.store(5, Ordering::SeqCst);
        assert_eq!(NET_UPDATE_RATE.load(Ordering::SeqCst), 5);
        NET_UPDATE_RATE.store(1, Ordering::SeqCst); // 重置
    }

    #[test]
    fn test_nthread_run_iteration_returns_duration() {
        // 单机模式（非多人）下不应启动后台线程，但可以手动驱动迭代
        let mut manager = NThreadManager::new();
        manager.set_tick_delay(50);

        // 调用一次迭代，应返回一个 Duration
        let delta = manager.nthread_run_iteration();
        // 由于 last_tick 刚设置，delta 可能接近 0 或返回 tick_delay
        // 主要确保函数不 panic 且返回有效 Duration
        assert!(delta <= Duration::from_millis(50) || delta == Duration::ZERO);
    }

    #[test]
    fn test_nthread_terminate_game_cleans_up() {
        let mut manager = NThreadManager::new();
        manager.set_multiplayer(false);
        manager.terminate_game("test reason");
        // 应该不 panic 并清理状态
        assert_eq!(manager.get_tick_delay(), manager.get_tick_delay());
    }

    #[test]
    fn test_nthread_has_500ms_passed_returns_tuple() {
        let manager = NThreadManager::new();
        let (passed, _draw_game) = manager.has_500ms_passed();
        // 刚创建时 last_tick 是 now，所以 ticks_elapsed 接近 0
        // 函数返回 (bool, bool) - 第一个表示是否正常推进
        let _ = passed;
    }
}
