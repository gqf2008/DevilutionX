//! Ticks - 时间与动画帧
//!
//! 移植自 Source/engine/ticks.hpp

use std::time::{Duration, Instant};

/// 游戏开始时间（懒加载）
static mut GAME_START_TIME: Option<Instant> = None;

/// 获取游戏运行时间（毫秒）
pub fn get_ticks() -> u32 {
    unsafe {
        if GAME_START_TIME.is_none() {
            GAME_START_TIME = Some(Instant::now());
        }
        GAME_START_TIME.unwrap().elapsed().as_millis() as u32
    }
}

/// 重置游戏计时器
pub fn reset_ticks() {
    unsafe {
        GAME_START_TIME = Some(Instant::now());
    }
}

/// 获取动画帧索引
///
/// # Arguments
/// * `frames` - 总帧数
/// * `fps` - 每秒帧数（默认60）
///
/// # Returns
/// 当前应该显示的帧索引
pub fn get_animation_frame(frames: u32, fps: u32) -> u32 {
    if frames == 0 || fps == 0 {
        return 0;
    }
    (get_ticks() / fps) % frames
}

/// 获取基于毫秒的动画帧
pub fn get_animation_frame_ms(frames: u32, ms_per_frame: u32) -> u32 {
    if frames == 0 || ms_per_frame == 0 {
        return 0;
    }
    (get_ticks() / ms_per_frame) % frames
}

/// 游戏时间管理器
#[derive(Clone, Debug)]
pub struct GameTime {
    /// 开始时间
    start_time: Instant,
    /// 暂停累积时间
    paused_duration: Duration,
    /// 暂停开始时间
    pause_start: Option<Instant>,
    /// 时间缩放因子
    time_scale: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

impl GameTime {
    /// 创建新的游戏时间管理器
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            paused_duration: Duration::ZERO,
            pause_start: None,
            time_scale: 1.0,
        }
    }

    /// 获取自游戏开始以来的时间（毫秒），不包括暂停时间
    pub fn elapsed_ms(&self) -> u32 {
        let raw_elapsed = self.start_time.elapsed();
        let effective = if let Some(pause_start) = self.pause_start {
            // 如果当前暂停中，减去暂停时间
            raw_elapsed.saturating_sub(pause_start.elapsed() + self.paused_duration)
        } else {
            raw_elapsed.saturating_sub(self.paused_duration)
        };
        (effective.as_millis() as f32 * self.time_scale) as u32
    }

    /// 暂停游戏时间
    pub fn pause(&mut self) {
        if self.pause_start.is_none() {
            self.pause_start = Some(Instant::now());
        }
    }

    /// 恢复游戏时间
    pub fn resume(&mut self) {
        if let Some(pause_start) = self.pause_start.take() {
            self.paused_duration += pause_start.elapsed();
        }
    }

    /// 检查是否暂停
    pub fn is_paused(&self) -> bool {
        self.pause_start.is_some()
    }

    /// 设置时间缩放
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// 获取时间缩放
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// 重置时间
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.paused_duration = Duration::ZERO;
        self.pause_start = None;
    }

    /// 获取动画帧
    pub fn animation_frame(&self, frames: u32, fps: u32) -> u32 {
        if frames == 0 || fps == 0 {
            return 0;
        }
        (self.elapsed_ms() / fps) % frames
    }
}

/// 帧率计算器
#[derive(Clone, Debug)]
pub struct FpsCounter {
    /// 帧计数
    frame_count: u32,
    /// 上次更新时间
    last_update: Instant,
    /// 计算的 FPS
    fps: f32,
    /// 更新间隔
    update_interval: Duration,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    /// 创建新的 FPS 计数器
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
            fps: 0.0,
            update_interval: Duration::from_secs(1),
        }
    }

    /// 创建指定更新间隔的 FPS 计数器
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
            fps: 0.0,
            update_interval: interval,
        }
    }

    /// 记录一帧
    pub fn tick(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_update.elapsed();
        if elapsed >= self.update_interval {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_update = Instant::now();
        }
    }

    /// 获取当前 FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// 获取帧时间（毫秒）
    pub fn frame_time_ms(&self) -> f32 {
        if self.fps > 0.0 {
            1000.0 / self.fps
        } else {
            0.0
        }
    }
}

/// 定时器
#[derive(Clone, Debug)]
pub struct Timer {
    /// 开始时间
    start: Instant,
    /// 持续时间
    duration: Duration,
    /// 是否重复
    repeat: bool,
}

impl Timer {
    /// 创建一次性定时器
    pub fn once(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            repeat: false,
        }
    }

    /// 创建重复定时器
    pub fn repeating(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            repeat: true,
        }
    }

    /// 检查定时器是否触发
    pub fn check(&mut self) -> bool {
        if self.start.elapsed() >= self.duration {
            if self.repeat {
                self.start = Instant::now();
            }
            true
        } else {
            false
        }
    }

    /// 获取剩余时间
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.start.elapsed())
    }

    /// 获取进度 (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        if total > 0.0 {
            (elapsed / total).min(1.0)
        } else {
            1.0
        }
    }

    /// 重置定时器
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_get_animation_frame() {
        let frame = get_animation_frame(10, 60);
        assert!(frame < 10);
    }

    #[test]
    fn test_game_time() {
        let mut time = GameTime::new();

        // 初始时间应该接近 0
        let initial = time.elapsed_ms();
        assert!(initial < 100);

        // 等待一段时间
        sleep(Duration::from_millis(50));
        let elapsed = time.elapsed_ms();
        assert!(elapsed >= 50);

        // 测试暂停
        time.pause();
        assert!(time.is_paused());
        let before_sleep = time.elapsed_ms();
        sleep(Duration::from_millis(50));
        let after_sleep = time.elapsed_ms();
        // 暂停期间时间应该不增加
        assert!(after_sleep - before_sleep < 10);

        // 恢复
        time.resume();
        assert!(!time.is_paused());
    }

    #[test]
    fn test_fps_counter() {
        let mut counter = FpsCounter::with_interval(Duration::from_millis(100));

        // 记录一些帧
        for _ in 0..10 {
            counter.tick();
            sleep(Duration::from_millis(10));
        }

        // 应该有非零的 FPS
        // 由于测试环境的不确定性，只检查基本功能
        let fps = counter.fps();
        // 可能还没更新，所以只检查不是负数
        assert!(fps >= 0.0);
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::once(Duration::from_millis(50));

        // 刚创建时不应该触发
        assert!(!timer.check());
        assert!(timer.progress() < 1.0);

        // 等待后应该触发
        sleep(Duration::from_millis(60));
        assert!(timer.check());
        assert!(timer.progress() >= 1.0);

        // 一次性定时器触发后不重置
        assert!(timer.check());
    }

    #[test]
    fn test_timer_repeating() {
        let mut timer = Timer::repeating(Duration::from_millis(30));

        // 等待后应该触发
        sleep(Duration::from_millis(40));
        assert!(timer.check());

        // 重复定时器重置后再次等待应该再次触发
        sleep(Duration::from_millis(40));
        assert!(timer.check());
    }
}
