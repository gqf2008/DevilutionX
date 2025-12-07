//! 帮助系统 (M32)
//!
//! 从 Source/help.cpp 移植
//! 处理游戏内帮助文本的显示和滚动

/// 帮助系统状态标志
pub static mut HELP_FLAG: bool = false;

/// 帮助文本跳过的行数（用于滚动）
static mut SKIP_LINES: usize = 0;

/// 帮助窗口可见行数
pub const HELP_VISIBLE_LINES: usize = 16;

/// 帮助文本内容
///
/// 从 C++ HelpText 数组移植
pub const HELP_TEXT: &[&str] = &[
    // 基础控制
    "控制说明:",
    "",
    "移动:",
    "  左键点击地面移动角色",
    "  按住Shift键可原地攻击",
    "",
    "攻击:",
    "  左键点击怪物进行攻击",
    "  右键使用当前技能/法术",
    "",
    "物品:",
    "  左键点击物品拾取",
    "  拖拽物品到装备栏装备",
    "  右键点击物品使用",
    "",
    // 快捷键
    "快捷键:",
    "",
    "  Tab - 显示/隐藏地图",
    "  I - 打开物品栏",
    "  C - 打开角色面板",
    "  S - 打开技能树",
    "  Q - 打开任务日志",
    "  B - 打开法术书",
    "",
    "  F1 - 显示帮助",
    "  F5-F8 - 快速保存/读取",
    "  Esc - 打开菜单",
    "",
    // 战斗提示
    "战斗提示:",
    "",
    "  注意生命值和法力值",
    "  使用药水恢复状态",
    "  升级提高角色属性",
    "  装备更好的武器和防具",
    "",
    // 多人游戏
    "多人游戏:",
    "",
    "  Enter - 打开聊天框",
    "  发送消息与其他玩家交流",
    "",
];

/// 帮助文本行结构
#[derive(Debug, Clone)]
pub struct HelpTextLine {
    /// 文本内容
    pub text: String,
    /// 是否为标题行
    pub is_header: bool,
}

impl HelpTextLine {
    /// 创建普通文本行
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_header: false,
        }
    }

    /// 创建标题行
    pub fn header(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_header: true,
        }
    }
}

/// 帮助系统管理器
#[derive(Debug)]
pub struct HelpSystem {
    /// 是否显示帮助
    pub visible: bool,
    /// 当前滚动位置
    pub scroll_position: usize,
    /// 帮助文本行
    pub lines: Vec<HelpTextLine>,
    /// 总行数
    pub total_lines: usize,
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpSystem {
    /// 创建新的帮助系统
    pub fn new() -> Self {
        let lines: Vec<HelpTextLine> = HELP_TEXT
            .iter()
            .map(|&text| {
                if text.ends_with(':') && !text.starts_with(' ') {
                    HelpTextLine::header(text)
                } else {
                    HelpTextLine::new(text)
                }
            })
            .collect();

        let total_lines = lines.len();

        Self {
            visible: false,
            scroll_position: 0,
            lines,
            total_lines,
        }
    }

    /// 初始化帮助系统
    ///
    /// 从 InitHelp 移植
    pub fn init(&mut self) {
        self.visible = false;
        self.scroll_position = 0;
    }

    /// 显示帮助
    ///
    /// 从 DisplayHelp 移植
    pub fn display(&mut self) {
        self.visible = true;
        self.scroll_position = 0;
    }

    /// 隐藏帮助
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// 切换帮助显示
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.display();
        }
    }

    /// 向上滚动
    ///
    /// 从 HelpScrollUp 移植
    pub fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }

    /// 向下滚动
    ///
    /// 从 HelpScrollDown 移植
    pub fn scroll_down(&mut self) {
        let max_scroll = self.max_scroll_position();
        if self.scroll_position < max_scroll {
            self.scroll_position += 1;
        }
    }

    /// 向上翻页
    pub fn page_up(&mut self) {
        self.scroll_position = self.scroll_position.saturating_sub(HELP_VISIBLE_LINES);
    }

    /// 向下翻页
    pub fn page_down(&mut self) {
        let max_scroll = self.max_scroll_position();
        self.scroll_position = (self.scroll_position + HELP_VISIBLE_LINES).min(max_scroll);
    }

    /// 滚动到顶部
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.max_scroll_position();
    }

    /// 获取最大滚动位置
    pub fn max_scroll_position(&self) -> usize {
        self.total_lines.saturating_sub(HELP_VISIBLE_LINES)
    }

    /// 获取当前可见行
    pub fn visible_lines(&self) -> impl Iterator<Item = &HelpTextLine> {
        self.lines
            .iter()
            .skip(self.scroll_position)
            .take(HELP_VISIBLE_LINES)
    }

    /// 获取滚动条位置（0.0 - 1.0）
    pub fn scroll_bar_position(&self) -> f32 {
        let max_scroll = self.max_scroll_position();
        if max_scroll == 0 {
            0.0
        } else {
            self.scroll_position as f32 / max_scroll as f32
        }
    }

    /// 检查是否可以向上滚动
    pub fn can_scroll_up(&self) -> bool {
        self.scroll_position > 0
    }

    /// 检查是否可以向下滚动
    pub fn can_scroll_down(&self) -> bool {
        self.scroll_position < self.max_scroll_position()
    }

    /// 添加自定义帮助文本
    pub fn add_line(&mut self, line: HelpTextLine) {
        self.lines.push(line);
        self.total_lines = self.lines.len();
    }

    /// 清除所有文本
    pub fn clear(&mut self) {
        self.lines.clear();
        self.total_lines = 0;
        self.scroll_position = 0;
    }

    /// 重新加载默认帮助文本
    pub fn reload_default(&mut self) {
        self.lines = HELP_TEXT
            .iter()
            .map(|&text| {
                if text.ends_with(':') && !text.starts_with(' ') {
                    HelpTextLine::header(text)
                } else {
                    HelpTextLine::new(text)
                }
            })
            .collect();
        self.total_lines = self.lines.len();
        self.scroll_position = 0;
    }
}

/// 全局帮助系统实例
pub static mut HELP_SYSTEM: Option<HelpSystem> = None;

/// 初始化帮助系统
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn init_help() {
    HELP_FLAG = false;
    SKIP_LINES = 0;
    HELP_SYSTEM = Some(HelpSystem::new());
}

/// 显示帮助
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn display_help() {
    HELP_FLAG = true;
    SKIP_LINES = 0;
    if let Some(ref mut help) = HELP_SYSTEM {
        help.display();
    }
}

/// 向上滚动帮助
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn help_scroll_up() {
    if SKIP_LINES > 0 {
        SKIP_LINES -= 1;
    }
    if let Some(ref mut help) = HELP_SYSTEM {
        help.scroll_up();
    }
}

/// 向下滚动帮助
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn help_scroll_down() {
    if let Some(ref help) = HELP_SYSTEM {
        if SKIP_LINES < help.max_scroll_position() {
            SKIP_LINES += 1;
        }
    }
    if let Some(ref mut help) = HELP_SYSTEM {
        help.scroll_down();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help = HelpSystem::new();
        assert!(!help.visible);
        assert_eq!(help.scroll_position, 0);
        assert!(!help.lines.is_empty());
    }

    #[test]
    fn test_help_display_toggle() {
        let mut help = HelpSystem::new();

        assert!(!help.visible);
        help.display();
        assert!(help.visible);
        help.hide();
        assert!(!help.visible);
        help.toggle();
        assert!(help.visible);
    }

    #[test]
    fn test_help_scrolling() {
        let mut help = HelpSystem::new();
        help.display();

        // 初始位置
        assert_eq!(help.scroll_position, 0);
        assert!(!help.can_scroll_up());

        // 向下滚动
        help.scroll_down();
        assert_eq!(help.scroll_position, 1);
        assert!(help.can_scroll_up());

        // 向上滚动
        help.scroll_up();
        assert_eq!(help.scroll_position, 0);
    }

    #[test]
    fn test_help_page_navigation() {
        let mut help = HelpSystem::new();
        help.display();

        // 翻页
        help.page_down();
        assert!(help.scroll_position > 0);

        help.page_up();
        assert_eq!(help.scroll_position, 0);
    }

    #[test]
    fn test_scroll_bar_position() {
        let mut help = HelpSystem::new();

        // 顶部
        assert_eq!(help.scroll_bar_position(), 0.0);

        // 底部
        help.scroll_to_bottom();
        assert!((help.scroll_bar_position() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_visible_lines() {
        let help = HelpSystem::new();
        let visible: Vec<_> = help.visible_lines().collect();
        assert!(visible.len() <= HELP_VISIBLE_LINES);
    }

    #[test]
    fn test_help_text_line() {
        let normal = HelpTextLine::new("Normal text");
        assert!(!normal.is_header);

        let header = HelpTextLine::header("Header:");
        assert!(header.is_header);
    }

    #[test]
    fn test_custom_help_text() {
        let mut help = HelpSystem::new();
        let original_count = help.total_lines;

        help.add_line(HelpTextLine::new("Custom line"));
        assert_eq!(help.total_lines, original_count + 1);

        help.clear();
        assert_eq!(help.total_lines, 0);

        help.reload_default();
        assert_eq!(help.total_lines, original_count);
    }
}
