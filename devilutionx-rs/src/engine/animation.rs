//! AnimationInfo - 动画信息和逻辑
//!
//! 移植自 Source/engine/animationinfo.h
//!
//! 包含核心动画信息和相关逻辑

use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};

/// 动画分布标志
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationDistributionFlags(u8);

impl AnimationDistributionFlags {
    /// 无特殊标志
    pub const NONE: Self = Self(0);

    /// processAnimation 将在 setNewAnimation 之后调用（在同一个游戏tick中）
    pub const PROCESS_ANIMATION_PENDING: Self = Self(1 << 0);

    /// 忽略最后一帧的延迟
    pub const SKIPS_DELAY_OF_LAST_FRAME: Self = Self(1 << 1);

    /// 重复动画（例如可以在命中帧后直接重复的近战攻击）
    pub const REPEATED_ACTION: Self = Self(1 << 2);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for AnimationDistributionFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// 动画信息结构体
///
/// 包含播放动画所需的所有信息
#[derive(Clone, Default)]
pub struct AnimationInfo {
    /// 动画精灵列表
    pub sprites: Option<ClxSpriteList>,

    /// 每帧需要多少游戏tick
    pub ticks_per_frame: i8,

    /// 当前帧的tick计数器
    pub tick_counter_of_current_frame: i8,

    /// 当前动画的帧数
    pub number_of_frames: i8,

    /// 当前动画帧
    pub current_frame: i8,

    /// 动画是否被石化（不应随游戏进度推进）
    pub is_petrified: bool,

    // 私有字段
    /// 用于分布的相关帧数
    relevant_frames_for_distributing: i8,
    /// 从上一个动画跳过的帧数
    skipped_frames_from_previous_animation: i8,
    /// tick修饰符
    tick_modifier: u16,
    /// 自序列开始以来的tick数
    ticks_since_sequence_started: i16,
}

/// 分数基准值（用于动画进度计算）
pub const BASE_VALUE_FRACTION: u8 = 128;

impl AnimationInfo {
    /// 创建新的动画信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取当前精灵
    pub fn current_sprite(&self) -> Option<ClxSprite> {
        self.sprites.as_ref().map(|sprites| {
            let frame = self.get_frame_to_use_for_rendering();
            sprites.get(frame as usize).unwrap_or_else(|| sprites.get(0).unwrap())
        })
    }

    /// 检查是否是最后一帧
    #[inline]
    pub fn is_last_frame(&self) -> bool {
        self.current_frame >= self.number_of_frames - 1
    }

    /// 计算用于渲染的帧索引
    pub fn get_frame_to_use_for_rendering(&self) -> i8 {
        // 简化版本 - 直接返回当前帧
        // 完整版本需要考虑游戏进度到下一个tick的插值
        self.current_frame.clamp(0, self.number_of_frames.saturating_sub(1).max(0))
    }

    /// 获取动画进度 (0-255)
    pub fn get_animation_progress(&self) -> u8 {
        if self.number_of_frames <= 1 {
            return 0;
        }

        let total_ticks = (self.number_of_frames as i32) * (self.ticks_per_frame as i32);
        if total_ticks <= 0 {
            return 0;
        }

        let current_ticks = (self.current_frame as i32) * (self.ticks_per_frame as i32)
            + (self.tick_counter_of_current_frame as i32);

        ((current_ticks * 255) / total_ticks) as u8
    }

    /// 设置新动画
    ///
    /// # Arguments
    /// * `sprites` - 动画精灵
    /// * `number_of_frames` - 帧数
    /// * `ticks_per_frame` - 每帧tick数
    /// * `flags` - 分布标志
    /// * `num_skipped_frames` - 跳过的帧数
    /// * `distribute_frames_before_frame` - 在此帧之前分布跳过的帧
    /// * `preview_shown_game_tick_fragments` - 预览动画显示的时长
    pub fn set_new_animation(
        &mut self,
        sprites: Option<ClxSpriteList>,
        number_of_frames: i8,
        ticks_per_frame: i8,
        flags: AnimationDistributionFlags,
        num_skipped_frames: i8,
        distribute_frames_before_frame: i8,
        _preview_shown_game_tick_fragments: u8,
    ) {
        self.sprites = sprites;
        self.number_of_frames = number_of_frames;
        self.ticks_per_frame = ticks_per_frame;
        self.current_frame = 0;
        self.tick_counter_of_current_frame = 0;
        self.is_petrified = false;

        // 处理跳过帧
        self.relevant_frames_for_distributing = if distribute_frames_before_frame > 0 {
            distribute_frames_before_frame
        } else {
            number_of_frames
        };
        self.skipped_frames_from_previous_animation = num_skipped_frames;

        // 计算tick修饰符
        if self.relevant_frames_for_distributing > 0 {
            let total_ticks = (self.relevant_frames_for_distributing as u16)
                * (ticks_per_frame.max(1) as u16);
            self.tick_modifier = (BASE_VALUE_FRACTION as u16 * total_ticks)
                / (self.relevant_frames_for_distributing as u16);
        } else {
            self.tick_modifier = BASE_VALUE_FRACTION as u16;
        }

        self.ticks_since_sequence_started = 0;

        // 如果需要立即处理动画
        if flags.contains(AnimationDistributionFlags::PROCESS_ANIMATION_PENDING) {
            self.process_animation(false);
        }
    }

    /// 更改动画数据（不重置位置）
    pub fn change_animation_data(
        &mut self,
        sprites: Option<ClxSpriteList>,
        number_of_frames: i8,
        ticks_per_frame: i8,
    ) {
        self.sprites = sprites;
        self.number_of_frames = number_of_frames;
        self.ticks_per_frame = ticks_per_frame;

        // 确保当前帧在有效范围内
        if self.current_frame >= number_of_frames {
            self.current_frame = number_of_frames.saturating_sub(1).max(0);
        }
    }

    /// 处理动画（推进帧）
    ///
    /// # Arguments
    /// * `reverse_animation` - 是否反向播放
    pub fn process_animation(&mut self, reverse_animation: bool) {
        if self.is_petrified {
            return;
        }

        self.ticks_since_sequence_started += 1;
        self.tick_counter_of_current_frame += 1;

        if self.tick_counter_of_current_frame >= self.ticks_per_frame {
            self.tick_counter_of_current_frame = 0;

            if reverse_animation {
                self.current_frame -= 1;
                if self.current_frame < 0 {
                    self.current_frame = self.number_of_frames.saturating_sub(1).max(0);
                }
            } else {
                self.current_frame += 1;
                if self.current_frame >= self.number_of_frames {
                    self.current_frame = 0;
                }
            }
        }
    }

    /// 重置动画到第一帧
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.tick_counter_of_current_frame = 0;
        self.ticks_since_sequence_started = 0;
    }

    /// 跳转到指定帧
    pub fn set_frame(&mut self, frame: i8) {
        self.current_frame = frame.clamp(0, self.number_of_frames.saturating_sub(1).max(0));
        self.tick_counter_of_current_frame = 0;
    }

    /// 检查动画是否有效
    pub fn is_valid(&self) -> bool {
        self.sprites.is_some() && self.number_of_frames > 0
    }

    /// 获取总时长（以tick为单位）
    pub fn total_duration(&self) -> i32 {
        (self.number_of_frames as i32) * (self.ticks_per_frame as i32)
    }
}

/// 简单动画播放器
///
/// 用于 UI 动画等简单情况
#[derive(Clone, Default)]
pub struct SimpleAnimation {
    /// 当前帧
    pub frame: u32,
    /// 总帧数
    pub frame_count: u32,
    /// 每帧持续时间（毫秒）
    pub frame_duration_ms: u32,
    /// 累计时间
    pub elapsed_ms: u32,
    /// 是否循环
    pub looping: bool,
    /// 是否完成
    pub finished: bool,
}

impl SimpleAnimation {
    /// 创建新的简单动画
    pub fn new(frame_count: u32, frame_duration_ms: u32, looping: bool) -> Self {
        Self {
            frame: 0,
            frame_count,
            frame_duration_ms,
            elapsed_ms: 0,
            looping,
            finished: false,
        }
    }

    /// 更新动画
    ///
    /// 返回是否切换了帧
    pub fn update(&mut self, delta_ms: u32) -> bool {
        if self.finished || self.frame_count == 0 {
            return false;
        }

        self.elapsed_ms += delta_ms;
        let mut frame_changed = false;

        while self.elapsed_ms >= self.frame_duration_ms {
            self.elapsed_ms -= self.frame_duration_ms;
            self.frame += 1;
            frame_changed = true;

            if self.frame >= self.frame_count {
                if self.looping {
                    self.frame = 0;
                } else {
                    self.frame = self.frame_count - 1;
                    self.finished = true;
                    break;
                }
            }
        }

        frame_changed
    }

    /// 重置动画
    pub fn reset(&mut self) {
        self.frame = 0;
        self.elapsed_ms = 0;
        self.finished = false;
    }

    /// 获取动画进度 (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.frame_count == 0 {
            return 0.0;
        }

        let frame_progress = self.elapsed_ms as f32 / self.frame_duration_ms as f32;
        (self.frame as f32 + frame_progress) / self.frame_count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_info_creation() {
        let anim = AnimationInfo::new();
        assert_eq!(anim.current_frame, 0);
        assert!(!anim.is_valid());
    }

    #[test]
    fn test_simple_animation() {
        let mut anim = SimpleAnimation::new(4, 100, false);

        assert_eq!(anim.frame, 0);
        assert!(!anim.finished);

        // 更新 50ms - 不应该切换帧
        assert!(!anim.update(50));
        assert_eq!(anim.frame, 0);

        // 更新 50ms - 应该切换到帧 1
        assert!(anim.update(50));
        assert_eq!(anim.frame, 1);

        // 更新 300ms - 应该切换到帧 3（最后一帧）并完成
        assert!(anim.update(300));
        assert_eq!(anim.frame, 3);
        assert!(anim.finished);
    }

    #[test]
    fn test_simple_animation_looping() {
        let mut anim = SimpleAnimation::new(3, 100, true);

        // 更新 400ms - 应该循环回来
        anim.update(400);
        assert_eq!(anim.frame, 1); // 4 帧后回到帧 1
        assert!(!anim.finished);
    }

    #[test]
    fn test_animation_flags() {
        let flags = AnimationDistributionFlags::PROCESS_ANIMATION_PENDING
            | AnimationDistributionFlags::SKIPS_DELAY_OF_LAST_FRAME;

        assert!(flags.contains(AnimationDistributionFlags::PROCESS_ANIMATION_PENDING));
        assert!(flags.contains(AnimationDistributionFlags::SKIPS_DELAY_OF_LAST_FRAME));
        assert!(!flags.contains(AnimationDistributionFlags::REPEATED_ACTION));
    }
}
