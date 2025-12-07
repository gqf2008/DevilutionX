//! 鼠标追踪系统 (M37)
//!
//! 从 Source/track.cpp 移植
//! 处理鼠标光标指向目标的追踪和玩家动作重复

use crate::game::types::Point;

/// 玩家动作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PlayerActionType {
    /// 无动作
    #[default]
    None = 0,
    /// 攻击
    Attack,
    /// 攻击怪物目标
    AttackMonsterTarget,
    /// 攻击玩家目标
    AttackPlayerTarget,
    /// 施法
    Spell,
    /// 对怪物施法
    SpellMonsterTarget,
    /// 对玩家施法
    SpellPlayerTarget,
    /// 操作物体
    OperateObject,
    /// 行走
    Walk,
}

/// 点击类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ClickType {
    /// 无点击
    #[default]
    None = 0,
    /// 左键
    Left,
    /// 右键
    Right,
}

/// 追踪系统状态
#[derive(Debug, Default)]
pub struct TrackingState {
    /// 光标下的怪物索引（-1表示无）
    pub cursor_monster: i32,
    /// 光标下的物体（占位）
    pub cursor_object: Option<usize>,
    /// 光标下的玩家（占位）
    pub cursor_player: Option<usize>,
    /// 光标位置
    pub cursor_position: Point,
    /// 最后的玩家动作
    pub last_action: PlayerActionType,
    /// 鼠标按下状态
    pub mouse_down: ClickType,
    /// 当前光标类型
    pub cursor_type: CursorType,
}

/// 光标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CursorType {
    /// 手型光标
    #[default]
    Hand = 0,
    /// 无光标
    None,
    /// 识别光标
    Identify,
    /// 修理光标
    Repair,
    /// 充能光标
    Recharge,
    /// 传送光标
    Telekinesis,
    /// 治愈他人光标
    HealOther,
    /// 物品光标
    Item,
}

impl TrackingState {
    /// 创建新的追踪状态
    pub fn new() -> Self {
        Self {
            cursor_monster: -1,
            cursor_object: None,
            cursor_player: None,
            cursor_position: Point::default(),
            last_action: PlayerActionType::None,
            mouse_down: ClickType::None,
            cursor_type: CursorType::Hand,
        }
    }

    /// 重置追踪状态
    pub fn reset(&mut self) {
        self.cursor_monster = -1;
        self.cursor_object = None;
        self.cursor_player = None;
        self.last_action = PlayerActionType::None;
    }

    /// 检查是否正在滚动（行走）
    ///
    /// 从 track_isscrolling 移植
    pub fn is_scrolling(&self) -> bool {
        self.last_action == PlayerActionType::Walk
    }
}

/// 目标追踪器
#[derive(Debug, Default)]
pub struct TargetTracker {
    /// 追踪状态
    pub state: TrackingState,
}

impl TargetTracker {
    /// 创建新的目标追踪器
    pub fn new() -> Self {
        Self {
            state: TrackingState::new(),
        }
    }

    /// 无效化目标
    ///
    /// 从 InvalidateTargets 移植
    /// 检查并清除无效的目标（死亡、隐藏、不可见等）
    pub fn invalidate_targets<F, G, H>(
        &mut self,
        is_monster_valid: F,
        is_object_valid: G,
        is_player_valid: H,
    ) where
        F: Fn(i32) -> bool,
        G: Fn(usize) -> bool,
        H: Fn(usize) -> bool,
    {
        // 检查怪物目标
        if self.state.cursor_monster != -1 && !is_monster_valid(self.state.cursor_monster) {
            self.state.cursor_monster = -1;
        }

        // 检查物体目标
        if let Some(obj_idx) = self.state.cursor_object {
            if !is_object_valid(obj_idx) {
                self.state.cursor_object = None;
            }
        }

        // 检查玩家目标
        if let Some(player_idx) = self.state.cursor_player {
            if !is_player_valid(player_idx) {
                self.state.cursor_player = None;
            }
        }
    }

    /// 设置光标位置
    pub fn set_cursor_position(&mut self, position: Point) {
        self.state.cursor_position = position;
    }

    /// 设置光标下的怪物
    pub fn set_cursor_monster(&mut self, monster_idx: i32) {
        self.state.cursor_monster = monster_idx;
    }

    /// 设置光标下的物体
    pub fn set_cursor_object(&mut self, object_idx: Option<usize>) {
        self.state.cursor_object = object_idx;
    }

    /// 设置光标下的玩家
    pub fn set_cursor_player(&mut self, player_idx: Option<usize>) {
        self.state.cursor_player = player_idx;
    }

    /// 设置最后的玩家动作
    pub fn set_last_action(&mut self, action: PlayerActionType) {
        self.state.last_action = action;
    }

    /// 设置鼠标按下状态
    pub fn set_mouse_down(&mut self, click: ClickType) {
        self.state.mouse_down = click;
    }

    /// 获取光标下的目标类型
    pub fn get_target_type(&self) -> TargetType {
        if self.state.cursor_monster != -1 {
            TargetType::Monster
        } else if self.state.cursor_object.is_some() {
            TargetType::Object
        } else if self.state.cursor_player.is_some() {
            TargetType::Player
        } else {
            TargetType::None
        }
    }

    /// 清除所有目标
    pub fn clear_targets(&mut self) {
        self.state.cursor_monster = -1;
        self.state.cursor_object = None;
        self.state.cursor_player = None;
    }
}

/// 目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    /// 无目标
    None,
    /// 怪物
    Monster,
    /// 物体
    Object,
    /// 玩家
    Player,
    /// 地面
    Ground,
}

/// 玩家动作重复器
#[derive(Debug)]
pub struct ActionRepeater {
    /// 是否启用重复
    pub enabled: bool,
    /// 重复延迟（毫秒）
    pub repeat_delay_ms: u32,
}

impl Default for ActionRepeater {
    fn default() -> Self {
        Self {
            enabled: true,
            repeat_delay_ms: 100,
        }
    }
}

impl ActionRepeater {
    /// 创建新的动作重复器
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查是否应该重复玩家动作
    ///
    /// 从 RepeatPlayerAction 移植
    pub fn should_repeat(
        &self,
        tracking: &TrackingState,
        is_in_store: bool,
        player_has_dest: bool,
        player_invincible: bool,
        player_can_change: bool,
    ) -> bool {
        // 必须是手型光标
        if tracking.cursor_type != CursorType::Hand {
            return false;
        }

        // 必须有鼠标按下或控制器动作
        if tracking.mouse_down == ClickType::None {
            return false;
        }

        // 不在商店中
        if is_in_store {
            return false;
        }

        // 必须有上次动作
        if tracking.last_action == PlayerActionType::None {
            return false;
        }

        // 玩家没有目标动作
        if player_has_dest {
            return false;
        }

        // 玩家不是无敌的
        if player_invincible {
            return false;
        }

        // 玩家可以改变动作
        if !player_can_change {
            return false;
        }

        true
    }

    /// 获取应该重复的动作类型
    pub fn get_repeat_action(
        &self,
        tracking: &TrackingState,
        uses_ranged: bool,
    ) -> Option<RepeatAction> {
        match tracking.last_action {
            PlayerActionType::Attack => Some(RepeatAction::AttackPosition {
                ranged: uses_ranged,
            }),
            PlayerActionType::AttackMonsterTarget => {
                if tracking.cursor_monster != -1 {
                    Some(RepeatAction::AttackMonster {
                        monster_id: tracking.cursor_monster,
                        ranged: uses_ranged,
                    })
                } else {
                    None
                }
            }
            PlayerActionType::AttackPlayerTarget => {
                if let Some(player_id) = tracking.cursor_player {
                    Some(RepeatAction::AttackPlayer {
                        player_id,
                        ranged: uses_ranged,
                    })
                } else {
                    None
                }
            }
            PlayerActionType::Spell => Some(RepeatAction::CastSpell),
            PlayerActionType::SpellMonsterTarget => {
                if tracking.cursor_monster != -1 {
                    Some(RepeatAction::CastSpell)
                } else {
                    None
                }
            }
            PlayerActionType::SpellPlayerTarget => {
                if tracking.cursor_player.is_some() {
                    Some(RepeatAction::CastSpell)
                } else {
                    None
                }
            }
            PlayerActionType::OperateObject => {
                if tracking.cursor_object.is_some() {
                    Some(RepeatAction::OperateObject)
                } else {
                    None
                }
            }
            PlayerActionType::Walk => Some(RepeatAction::Walk),
            PlayerActionType::None => None,
        }
    }
}

/// 重复动作类型
#[derive(Debug, Clone, Copy)]
pub enum RepeatAction {
    /// 攻击位置
    AttackPosition { ranged: bool },
    /// 攻击怪物
    AttackMonster { monster_id: i32, ranged: bool },
    /// 攻击玩家
    AttackPlayer { player_id: usize, ranged: bool },
    /// 施法
    CastSpell,
    /// 操作物体
    OperateObject,
    /// 行走
    Walk,
}

/// 全局目标追踪器
pub static mut TARGET_TRACKER: Option<TargetTracker> = None;

/// 初始化追踪系统
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn init_tracking() {
    TARGET_TRACKER = Some(TargetTracker::new());
}

/// 检查是否正在滚动
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn track_isscrolling() -> bool {
    if let Some(ref tracker) = TARGET_TRACKER {
        tracker.state.is_scrolling()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_state_creation() {
        let state = TrackingState::new();
        assert_eq!(state.cursor_monster, -1);
        assert!(state.cursor_object.is_none());
        assert!(state.cursor_player.is_none());
        assert_eq!(state.last_action, PlayerActionType::None);
    }

    #[test]
    fn test_is_scrolling() {
        let mut state = TrackingState::new();
        assert!(!state.is_scrolling());

        state.last_action = PlayerActionType::Walk;
        assert!(state.is_scrolling());
    }

    #[test]
    fn test_target_tracker_creation() {
        let tracker = TargetTracker::new();
        assert_eq!(tracker.get_target_type(), TargetType::None);
    }

    #[test]
    fn test_target_type_detection() {
        let mut tracker = TargetTracker::new();

        tracker.set_cursor_monster(5);
        assert_eq!(tracker.get_target_type(), TargetType::Monster);

        tracker.clear_targets();
        tracker.set_cursor_object(Some(3));
        assert_eq!(tracker.get_target_type(), TargetType::Object);

        tracker.clear_targets();
        tracker.set_cursor_player(Some(1));
        assert_eq!(tracker.get_target_type(), TargetType::Player);
    }

    #[test]
    fn test_invalidate_targets() {
        let mut tracker = TargetTracker::new();
        tracker.set_cursor_monster(5);
        tracker.set_cursor_object(Some(3));
        tracker.set_cursor_player(Some(1));

        // 所有目标都无效
        tracker.invalidate_targets(|_| false, |_| false, |_| false);

        assert_eq!(tracker.state.cursor_monster, -1);
        assert!(tracker.state.cursor_object.is_none());
        assert!(tracker.state.cursor_player.is_none());
    }

    #[test]
    fn test_action_repeater() {
        let repeater = ActionRepeater::new();
        let mut state = TrackingState::new();

        // 默认状态不应该重复
        assert!(!repeater.should_repeat(&state, false, false, false, true));

        // 设置正确的状态
        state.cursor_type = CursorType::Hand;
        state.mouse_down = ClickType::Left;
        state.last_action = PlayerActionType::Attack;

        assert!(repeater.should_repeat(&state, false, false, false, true));
    }

    #[test]
    fn test_get_repeat_action() {
        let repeater = ActionRepeater::new();
        let mut state = TrackingState::new();

        state.last_action = PlayerActionType::Attack;
        let action = repeater.get_repeat_action(&state, false);
        assert!(matches!(
            action,
            Some(RepeatAction::AttackPosition { ranged: false })
        ));

        state.last_action = PlayerActionType::Walk;
        let action = repeater.get_repeat_action(&state, false);
        assert!(matches!(action, Some(RepeatAction::Walk)));
    }

    #[test]
    fn test_cursor_type_default() {
        assert_eq!(CursorType::default(), CursorType::Hand);
    }

    #[test]
    fn test_player_action_type_default() {
        assert_eq!(PlayerActionType::default(), PlayerActionType::None);
    }
}
