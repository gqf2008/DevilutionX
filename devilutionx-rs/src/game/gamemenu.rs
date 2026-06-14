//! 游戏菜单系统 (M38)
//!
//! 从 Source/gamemenu.cpp 移植
//! 处理游戏内菜单（选项、保存、加载、退出等）

/// 菜单项标志
pub mod menu_flags {
    /// 启用
    pub const ENABLED: u32 = 1 << 0;
    /// 滑块
    pub const SLIDER: u32 = 1 << 1;
}

/// 音量范围
pub const VOLUME_MIN: i32 = 0;
pub const VOLUME_MAX: i32 = 100;
pub const VOLUME_STEPS: i32 = 21;

/// 菜单项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItemType {
    /// 普通按钮
    Button,
    /// 滑块
    Slider,
    /// 分隔符
    Separator,
}

/// 菜单项
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// 标志
    pub flags: u32,
    /// 显示文本
    pub text: Option<String>,
    /// 菜单项类型
    pub item_type: MenuItemType,
    /// 滑块当前值（如果是滑块）
    pub slider_value: i32,
    /// 滑块最小值
    pub slider_min: i32,
    /// 滑块最大值
    pub slider_max: i32,
    /// 滑块步数
    pub slider_steps: i32,
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            flags: menu_flags::ENABLED,
            text: None,
            item_type: MenuItemType::Button,
            slider_value: 0,
            slider_min: 0,
            slider_max: 100,
            slider_steps: 10,
        }
    }
}

impl MenuItem {
    /// 创建新菜单项
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// 创建滑块菜单项
    pub fn slider(text: impl Into<String>, min: i32, max: i32, value: i32) -> Self {
        Self {
            flags: menu_flags::ENABLED | menu_flags::SLIDER,
            text: Some(text.into()),
            item_type: MenuItemType::Slider,
            slider_value: value,
            slider_min: min,
            slider_max: max,
            slider_steps: (max - min) / 5,
        }
    }

    /// 创建分隔符
    pub fn separator() -> Self {
        Self {
            flags: 0,
            text: None,
            item_type: MenuItemType::Separator,
            ..Default::default()
        }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.flags & menu_flags::ENABLED != 0
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.flags |= menu_flags::ENABLED;
        } else {
            self.flags &= !menu_flags::ENABLED;
        }
    }

    /// 检查是否是滑块
    pub fn is_slider(&self) -> bool {
        self.flags & menu_flags::SLIDER != 0
    }

    /// 添加标志
    pub fn add_flags(&mut self, flags: u32) {
        self.flags |= flags;
    }

    /// 移除标志
    pub fn remove_flags(&mut self, flags: u32) {
        self.flags &= !flags;
    }

    /// 设置滑块值
    pub fn set_slider_value(&mut self, value: i32) {
        self.slider_value = value.clamp(self.slider_min, self.slider_max);
    }

    /// 获取滑块百分比（0.0 - 1.0）
    pub fn slider_percent(&self) -> f32 {
        if self.slider_max == self.slider_min {
            0.0
        } else {
            (self.slider_value - self.slider_min) as f32
                / (self.slider_max - self.slider_min) as f32
        }
    }
}

/// 游戏菜单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMenuState {
    /// 关闭
    #[default]
    Closed,
    /// 主菜单
    Main,
    /// 选项菜单
    Options,
}

/// 游戏菜单管理器
#[derive(Debug)]
pub struct GameMenu {
    /// 菜单状态
    pub state: GameMenuState,
    /// 是否为多人游戏
    pub is_multiplayer: bool,
    /// 主菜单项
    pub main_items: Vec<MenuItem>,
    /// 选项菜单项
    pub options_items: Vec<MenuItem>,
    /// 当前选中项
    pub selected_index: usize,
    /// 音乐音量
    pub music_volume: i32,
    /// 音效音量
    pub sound_volume: i32,
    /// 亮度
    pub brightness: i32,
    /// 游戏速度
    pub game_speed: i32,
    /// 音乐是否开启
    pub music_on: bool,
    /// 音效是否开启
    pub sound_on: bool,
}

impl Default for GameMenu {
    fn default() -> Self {
        Self::new(false)
    }
}

impl GameMenu {
    /// 创建新的游戏菜单
    pub fn new(is_multiplayer: bool) -> Self {
        let mut menu = Self {
            state: GameMenuState::Closed,
            is_multiplayer,
            main_items: Vec::new(),
            options_items: Vec::new(),
            selected_index: 0,
            music_volume: VOLUME_MAX,
            sound_volume: VOLUME_MAX,
            brightness: 50,
            game_speed: 20,
            music_on: true,
            sound_on: true,
        };
        menu.init_menus();
        menu
    }

    /// 初始化菜单项
    fn init_menus(&mut self) {
        // 单人游戏主菜单
        if !self.is_multiplayer {
            self.main_items = vec![
                MenuItem::new("选项"),
                MenuItem::new("保存游戏"),
                MenuItem::new("加载游戏"),
                MenuItem::new("返回主菜单"),
                MenuItem::new("退出游戏"),
            ];
        } else {
            // 多人游戏主菜单
            self.main_items = vec![
                MenuItem::new("选项"),
                MenuItem::new("返回主菜单"),
                MenuItem::new("退出游戏"),
            ];
        }

        // 选项菜单
        self.options_items = vec![
            MenuItem::slider("音乐", VOLUME_MIN, VOLUME_MAX, self.music_volume),
            MenuItem::slider("音效", VOLUME_MIN, VOLUME_MAX, self.sound_volume),
            MenuItem::slider("亮度", 0, 100, self.brightness),
            MenuItem::slider("速度", 20, 50, self.game_speed),
            MenuItem::new("返回上一菜单"),
        ];

        // 多人游戏禁用速度滑块
        if self.is_multiplayer {
            self.options_items[3].remove_flags(menu_flags::ENABLED | menu_flags::SLIDER);
        }
    }

    /// 打开游戏菜单
    ///
    /// 从 gamemenu_on 移植
    pub fn open(&mut self) {
        self.state = GameMenuState::Main;
        self.selected_index = 0;
        self.update_main_menu();
    }

    /// 关闭游戏菜单
    ///
    /// 从 gamemenu_off 移植
    pub fn close(&mut self) {
        self.state = GameMenuState::Closed;
    }

    /// 切换菜单
    ///
    /// 从 gamemenu_handle_previous 移植
    pub fn toggle(&mut self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }

    /// 检查菜单是否打开
    pub fn is_open(&self) -> bool {
        self.state != GameMenuState::Closed
    }

    /// 更新主菜单状态
    fn update_main_menu(&mut self) {
        if !self.is_multiplayer && self.main_items.len() > 2 {
            // 更新"加载游戏"按钮状态（需要有效存档）
            // 这里简化处理，实际需要检查存档文件
        }
    }

    /// 打开选项菜单
    ///
    /// 从 GamemenuOptions 移植
    pub fn open_options(&mut self) {
        self.state = GameMenuState::Options;
        self.selected_index = 0;
        self.update_options_menu();
    }

    /// 更新选项菜单
    fn update_options_menu(&mut self) {
        // 更新音乐滑块
        if self.music_on {
            self.options_items[0].add_flags(menu_flags::ENABLED | menu_flags::SLIDER);
            self.options_items[0].text = Some("音乐".to_string());
        } else {
            self.options_items[0].remove_flags(menu_flags::SLIDER);
            self.options_items[0].text = Some("音乐（已禁用）".to_string());
        }

        // 更新音效滑块
        if self.sound_on {
            self.options_items[1].add_flags(menu_flags::ENABLED | menu_flags::SLIDER);
            self.options_items[1].text = Some("音效".to_string());
        } else {
            self.options_items[1].remove_flags(menu_flags::SLIDER);
            self.options_items[1].text = Some("音效（已禁用）".to_string());
        }

        // 更新滑块值
        self.options_items[0].set_slider_value(self.music_volume);
        self.options_items[1].set_slider_value(self.sound_volume);
        self.options_items[2].set_slider_value(self.brightness);
        self.options_items[3].set_slider_value(self.game_speed);
    }

    /// 返回上一菜单
    pub fn go_back(&mut self) {
        match self.state {
            GameMenuState::Options => {
                self.state = GameMenuState::Main;
                self.selected_index = 0;
            }
            GameMenuState::Main => {
                self.close();
            }
            GameMenuState::Closed => {}
        }
    }

    /// 选择上一项
    pub fn select_prev(&mut self) {
        let items_len = self.current_items().len();
        if items_len == 0 {
            return;
        }

        loop {
            if self.selected_index == 0 {
                self.selected_index = items_len - 1;
            } else {
                self.selected_index -= 1;
            }

            // 跳过分隔符和禁用项
            let item = &self.current_items()[self.selected_index];
            if item.item_type != MenuItemType::Separator && item.is_enabled() {
                break;
            }
        }
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        let items_len = self.current_items().len();
        if items_len == 0 {
            return;
        }

        loop {
            self.selected_index = (self.selected_index + 1) % items_len;

            // 跳过分隔符和禁用项
            let item = &self.current_items()[self.selected_index];
            if item.item_type != MenuItemType::Separator && item.is_enabled() {
                break;
            }
        }
    }

    /// 获取当前菜单项
    pub fn current_items(&self) -> &[MenuItem] {
        match self.state {
            GameMenuState::Main => &self.main_items,
            GameMenuState::Options => &self.options_items,
            GameMenuState::Closed => &[],
        }
    }

    /// 获取当前选中项
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.current_items().get(self.selected_index)
    }

    /// 调整滑块值
    pub fn adjust_slider(&mut self, delta: i32) {
        let items = match self.state {
            GameMenuState::Options => &mut self.options_items,
            _ => return,
        };

        if let Some(item) = items.get_mut(self.selected_index) {
            if item.is_slider() {
                let step = (item.slider_max - item.slider_min) / item.slider_steps.max(1);
                item.set_slider_value(item.slider_value + delta * step);

                // 更新对应的设置
                match self.selected_index {
                    0 => {
                        self.music_volume = item.slider_value;
                        self.music_on = self.music_volume > VOLUME_MIN;
                    }
                    1 => {
                        self.sound_volume = item.slider_value;
                        self.sound_on = self.sound_volume > VOLUME_MIN;
                    }
                    2 => self.brightness = item.slider_value,
                    3 => self.game_speed = item.slider_value,
                    _ => {}
                }
            }
        }
    }

    /// 切换音乐
    pub fn toggle_music(&mut self) {
        self.music_on = !self.music_on;
        if self.music_on {
            self.music_volume = VOLUME_MAX;
        } else {
            self.music_volume = VOLUME_MIN;
        }
        self.update_options_menu();
    }

    /// 切换音效
    pub fn toggle_sound(&mut self) {
        self.sound_on = !self.sound_on;
        if self.sound_on {
            self.sound_volume = VOLUME_MAX;
        } else {
            self.sound_volume = VOLUME_MIN;
        }
        self.update_options_menu();
    }
}

/// 菜单动作结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// 无动作
    None,
    /// 打开选项
    OpenOptions,
    /// 保存游戏
    SaveGame,
    /// 加载游戏
    LoadGame,
    /// 返回主菜单
    ExitToMainMenu,
    /// 退出游戏
    QuitGame,
    /// 返回上一菜单
    GoBack,
}

/// 全局游戏菜单状态
pub static mut IS_GAME_MENU_OPEN: bool = false;

/// 全局游戏菜单实例
pub static mut GAME_MENU: Option<GameMenu> = None;

/// 打开游戏菜单
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn gamemenu_on() {
    IS_GAME_MENU_OPEN = true;
    if let Some(ref mut menu) = GAME_MENU {
        menu.open();
    }
}

/// 关闭游戏菜单
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn gamemenu_off() {
    IS_GAME_MENU_OPEN = false;
    if let Some(ref mut menu) = GAME_MENU {
        menu.close();
    }
}

/// 切换游戏菜单
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn gamemenu_handle_previous() {
    if let Some(ref mut menu) = GAME_MENU {
        menu.toggle();
        IS_GAME_MENU_OPEN = menu.is_open();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("Test");
        assert!(item.is_enabled());
        assert!(!item.is_slider());
        assert_eq!(item.text, Some("Test".to_string()));
    }

    #[test]
    fn test_slider_creation() {
        let slider = MenuItem::slider("Volume", 0, 100, 50);
        assert!(slider.is_enabled());
        assert!(slider.is_slider());
        assert_eq!(slider.slider_value, 50);
        assert_eq!(slider.slider_percent(), 0.5);
    }

    #[test]
    fn test_game_menu_creation() {
        let menu = GameMenu::new(false);
        assert!(!menu.is_open());
        assert!(!menu.main_items.is_empty());
    }

    #[test]
    fn test_menu_open_close() {
        let mut menu = GameMenu::new(false);

        menu.open();
        assert!(menu.is_open());
        assert_eq!(menu.state, GameMenuState::Main);

        menu.close();
        assert!(!menu.is_open());
        assert_eq!(menu.state, GameMenuState::Closed);
    }

    #[test]
    fn test_menu_toggle() {
        let mut menu = GameMenu::new(false);

        menu.toggle();
        assert!(menu.is_open());

        menu.toggle();
        assert!(!menu.is_open());
    }

    #[test]
    fn test_options_menu() {
        let mut menu = GameMenu::new(false);
        menu.open();
        menu.open_options();

        assert_eq!(menu.state, GameMenuState::Options);
        assert!(!menu.options_items.is_empty());
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = GameMenu::new(false);
        menu.open();

        let initial = menu.selected_index;
        menu.select_next();
        assert_ne!(menu.selected_index, initial);

        menu.select_prev();
        assert_eq!(menu.selected_index, initial);
    }

    #[test]
    fn test_slider_adjustment() {
        let mut menu = GameMenu::new(false);
        menu.open();
        menu.open_options();

        menu.selected_index = 0; // highlight the music slider
        let initial_volume = menu.music_volume;
        // Increase; if already at max, decrease — a real slider must respond.
        menu.adjust_slider(1);
        if menu.music_volume == initial_volume {
            menu.adjust_slider(-1);
        }
        assert_ne!(menu.music_volume, initial_volume, "music slider should respond to adjustment");
    }

    #[test]
    fn test_go_back() {
        let mut menu = GameMenu::new(false);
        menu.open();
        menu.open_options();

        assert_eq!(menu.state, GameMenuState::Options);

        menu.go_back();
        assert_eq!(menu.state, GameMenuState::Main);

        menu.go_back();
        assert_eq!(menu.state, GameMenuState::Closed);
    }

    #[test]
    fn test_multiplayer_menu() {
        let menu = GameMenu::new(true);

        // 多人游戏菜单项较少
        assert!(menu.main_items.len() < 5);

        // 速度滑块应该被禁用
        assert!(!menu.options_items[3].is_enabled());
    }
}
