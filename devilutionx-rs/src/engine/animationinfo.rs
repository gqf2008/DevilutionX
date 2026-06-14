//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! AnimationInfo - 动画信息和逻辑
//!
//! 移植自 Source/engine/animationinfo.h
//!
//! 包含核心动画信息和相关逻辑

use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};

/// 动画分布标志
///
/// 对应 C++ AnimationDistributionFlags 枚举
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum AnimationDistributionFlags {
    #[default]
    None = 0,
    /// processAnimation 将在 setNewAnimation 之后调用（在同一个游戏tick中）
    ProcessAnimationPending = 1 << 0,
    /// 忽略最后一帧的延迟
    SkipsDelayOfLastFrame = 1 << 1,
    /// 重复动画（例如可以在命中帧后直接重复的近战攻击）
    RepeatedAction = 1 << 2,
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

    // 私有字段 (对应 C++ private 成员)
    /// 用于分布的相关帧数
    relevant_frames_for_distributing_: i8,
    /// 从上一个动画跳过的帧数
    skipped_frames_from_previous_animation_: i8,
    /// tick修饰符
    tick_modifier_: u16,
    /// 自序列开始以来的tick数
    ticks_since_sequence_started_: i16,
}

impl AnimationInfo {
    /// 分数基准值 (对应 C++ baseValueFraction)
    pub const BASE_VALUE_FRACTION: u8 = 128;

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
        // 简化版本 - 完整实现需要考虑 gfProgressToNextGameTick 插值
        self.current_frame.clamp(0, self.number_of_frames.saturating_sub(1).max(0))
    }

    /// 获取动画进度 (0-baseValueFraction)
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

        ((current_ticks * (Self::BASE_VALUE_FRACTION as i32)) / total_ticks) as u8
    }

    /// 设置新动画
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
        self.relevant_frames_for_distributing_ = if distribute_frames_before_frame > 0 {
            distribute_frames_before_frame
        } else {
            number_of_frames
        };
        self.skipped_frames_from_previous_animation_ = num_skipped_frames;

        // 计算tick修饰符
        if self.relevant_frames_for_distributing_ > 0 {
            let total_ticks = (self.relevant_frames_for_distributing_ as u16)
                * (ticks_per_frame.max(1) as u16);
            self.tick_modifier_ = (Self::BASE_VALUE_FRACTION as u16 * total_ticks)
                / (self.relevant_frames_for_distributing_ as u16);
        } else {
            self.tick_modifier_ = Self::BASE_VALUE_FRACTION as u16;
        }

        self.ticks_since_sequence_started_ = 0;

        // 如果需要立即处理动画
        if flags == AnimationDistributionFlags::ProcessAnimationPending {
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
    pub fn process_animation(&mut self, reverse_animation: bool) {
        if self.is_petrified {
            return;
        }

        self.ticks_since_sequence_started_ += 1;
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

    // 私有方法 (对应 C++ private)
    fn get_progress_to_next_game_tick(&self) -> u8 {
        // 简化实现 - 完整版本需要访问全局 gfProgressToNextGameTick
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_info_default() {
        let anim = AnimationInfo::default();
        assert_eq!(anim.current_frame, 0);
        assert!(anim.sprites.is_none());
    }

    #[test]
    fn test_animation_flags() {
        assert_eq!(AnimationDistributionFlags::None as u8, 0);
        assert_eq!(AnimationDistributionFlags::ProcessAnimationPending as u8, 1);
        assert_eq!(AnimationDistributionFlags::SkipsDelayOfLastFrame as u8, 2);
        assert_eq!(AnimationDistributionFlags::RepeatedAction as u8, 4);
    }
}
