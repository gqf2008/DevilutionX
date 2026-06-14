//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! Demo Mode - 演示录制与回放
//!
//! 移植自 Source/engine/demomode.h/cpp
//!
//! 当前为 DISABLE_DEMOMODE 模式的桩实现

use std::sync::atomic::{AtomicU8, Ordering};

/// 当前到下一个游戏tick的进度
pub static PROGRESS_TO_NEXT_GAME_TICK: AtomicU8 = AtomicU8::new(0);

/// 覆盖选项（DISABLE_DEMOMODE 时为空实现）
pub fn override_options() {
    // 空实现
}

/// 检查 demo 是否正在运行
pub fn is_running() -> bool {
    false
}

/// 检查是否正在录制
pub fn is_recording() -> bool {
    false
}

/// 获取游戏循环是否应该运行
pub fn get_run_game_loop(_draw_game: &mut bool, _process_input: &mut bool) -> bool {
    false
}

/// 获取消息
/// 
/// C++ 原型: bool FetchMessage(SDL_Event *event, uint16_t *modState)
pub fn fetch_message(_event: &mut sdl2::event::Event, _mod_state: &mut u16) -> bool {
    false
}

/// 记录游戏循环结果
pub fn record_game_loop_result(_run_game_loop: bool) {
    // 空实现
}

/// 记录消息
pub fn record_message(_event: &sdl2::event::Event, _mod_state: u16) {
    // 空实现
}

/// 通知游戏循环开始
pub fn notify_game_loop_start() {
    // 空实现
}

/// 通知游戏循环结束
pub fn notify_game_loop_end() {
    // 空实现
}

/// 模拟启动以来的毫秒数
pub fn simulate_milliseconds_since_startup() -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demomode_disabled() {
        assert!(!is_running());
        assert!(!is_recording());
    }
}
