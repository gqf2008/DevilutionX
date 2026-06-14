//! ⚠️ 警告：本文件已完成移植，禁止再次移植！
//! ⚠️ WARNING: This file has been fully ported. DO NOT port again!
//!
//! UI 面板状态管理
//!
//! 对应 C++: Source/control.h, Source/inv.h, Source/quests.h 中的 UI 状态变量
//!
//! 该模块提供对各种 UI 面板打开/关闭状态的访问。

use std::sync::RwLock;

// ============================================================================
// 面板状态标志
// ============================================================================

/// UI 面板状态
#[derive(Debug, Clone, Default)]
pub struct PanelState {
    /// 物品栏打开
    /// C++ 等价: `invflag`
    pub inv_flag: bool,
    
    /// 角色面板打开
    /// C++ 等价: `CharFlag`
    pub char_flag: bool,
    
    /// 任务日志打开
    /// C++ 等价: `QuestLogIsOpen`
    pub quest_log_is_open: bool,
    
    /// 法术书打开
    /// C++ 等价: `SpellbookFlag`
    pub spellbook_flag: bool,
    
    /// 对方打开
    /// C++ 等价: `talkflag`
    pub talk_flag: bool,
    
    /// 商店打开
    /// C++ 等价: `stextflag`
    pub store_flag: bool,
    
    /// 队伍侧边栏打开
    /// C++ 等价: `PartySidePanelOpen`
    pub party_side_panel_open: bool,
    
    /// 自动地图打开
    /// C++ 等价: `AutomapActive`
    pub automap_active: bool,
    
    /// 帮助打开
    /// C++ 等价: `HelpFlag`
    pub help_flag: bool,
    
    /// 聊天输入打开
    /// C++ 等价: `ChatLogFlag`
    pub chat_log_flag: bool,
    
    /// 游戏菜单打开
    /// C++ 等价: `gmenu_is_active()`
    pub game_menu_active: bool,
    
    /// 存档对话框打开
    pub save_dialog_open: bool,
    
    /// 主面板可见
    pub main_panel_visible: bool,
}

/// 全局面板状态
static PANEL_STATE: RwLock<PanelState> = RwLock::new(PanelState {
    inv_flag: false,
    char_flag: false,
    quest_log_is_open: false,
    spellbook_flag: false,
    talk_flag: false,
    store_flag: false,
    party_side_panel_open: false,
    automap_active: false,
    help_flag: false,
    chat_log_flag: false,
    game_menu_active: false,
    save_dialog_open: false,
    main_panel_visible: true,
});

// ============================================================================
// 访问函数
// ============================================================================

/// 获取物品栏是否打开
/// 
/// C++ 等价: `invflag`
pub fn is_inv_open() -> bool {
    PANEL_STATE.read().unwrap().inv_flag
}

/// 设置物品栏状态
pub fn set_inv_open(open: bool) {
    PANEL_STATE.write().unwrap().inv_flag = open;
}

/// 获取角色面板是否打开
/// 
/// C++ 等价: `CharFlag`
pub fn is_char_panel_open() -> bool {
    PANEL_STATE.read().unwrap().char_flag
}

/// 设置角色面板状态
pub fn set_char_panel_open(open: bool) {
    PANEL_STATE.write().unwrap().char_flag = open;
}

/// 获取任务日志是否打开
/// 
/// C++ 等价: `QuestLogIsOpen`
pub fn is_quest_log_open() -> bool {
    PANEL_STATE.read().unwrap().quest_log_is_open
}

/// 设置任务日志状态
pub fn set_quest_log_open(open: bool) {
    PANEL_STATE.write().unwrap().quest_log_is_open = open;
}

/// 获取法术书是否打开
/// 
/// C++ 等价: `SpellbookFlag`
pub fn is_spellbook_open() -> bool {
    PANEL_STATE.read().unwrap().spellbook_flag
}

/// 设置法术书状态
pub fn set_spellbook_open(open: bool) {
    PANEL_STATE.write().unwrap().spellbook_flag = open;
}

/// 获取商店是否打开
/// 
/// C++ 等价: `stextflag`
pub fn is_store_open() -> bool {
    PANEL_STATE.read().unwrap().store_flag
}

/// 设置商店状态
pub fn set_store_open(open: bool) {
    PANEL_STATE.write().unwrap().store_flag = open;
}

/// 检查是否有任何左侧面板打开
/// 
/// C++ 等价: 检查 CharFlag || QuestLogIsOpen
pub fn is_left_panel_open() -> bool {
    let state = PANEL_STATE.read().unwrap();
    state.char_flag || state.quest_log_is_open
}

/// 检查是否有任何右侧面板打开
/// 
/// C++ 等价: 检查 invflag || SpellbookFlag
pub fn is_right_panel_open() -> bool {
    let state = PANEL_STATE.read().unwrap();
    state.inv_flag || state.spellbook_flag
}

/// 检查是否有任何面板打开（阻止游戏输入）
pub fn is_any_panel_open() -> bool {
    let state = PANEL_STATE.read().unwrap();
    state.inv_flag
        || state.char_flag
        || state.quest_log_is_open
        || state.spellbook_flag
        || state.talk_flag
        || state.store_flag
        || state.help_flag
}

/// 获取自动地图是否活动
/// 
/// C++ 等价: `AutomapActive`
pub fn is_automap_active() -> bool {
    PANEL_STATE.read().unwrap().automap_active
}

/// 设置自动地图状态
pub fn set_automap_active(active: bool) {
    PANEL_STATE.write().unwrap().automap_active = active;
}

/// 获取游戏菜单是否活动
/// 
/// C++ 等价: `gmenu_is_active()`
pub fn is_game_menu_active() -> bool {
    PANEL_STATE.read().unwrap().game_menu_active
}

/// 设置游戏菜单状态
pub fn set_game_menu_active(active: bool) {
    PANEL_STATE.write().unwrap().game_menu_active = active;
}

/// 获取整个面板状态的快照
pub fn get_panel_state() -> PanelState {
    PANEL_STATE.read().unwrap().clone()
}

/// 用闭包访问面板状态
pub fn with_panel_state<F, R>(f: F) -> R
where
    F: FnOnce(&PanelState) -> R,
{
    f(&PANEL_STATE.read().unwrap())
}

/// 用闭包修改面板状态
pub fn with_panel_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut PanelState) -> R,
{
    f(&mut PANEL_STATE.write().unwrap())
}

/// 关闭所有面板
pub fn close_all_panels() {
    let mut state = PANEL_STATE.write().unwrap();
    state.inv_flag = false;
    state.char_flag = false;
    state.quest_log_is_open = false;
    state.spellbook_flag = false;
    state.help_flag = false;
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_flags() {
        set_inv_open(true);
        assert!(is_inv_open());
        assert!(is_right_panel_open());
        
        set_char_panel_open(true);
        assert!(is_char_panel_open());
        assert!(is_left_panel_open());
        
        assert!(is_any_panel_open());
        
        close_all_panels();
        assert!(!is_inv_open());
        assert!(!is_char_panel_open());
        assert!(!is_any_panel_open());
    }
}
