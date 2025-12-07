//! 游戏菜单模块
//!
//! C++ Reference: Source/gmenu.cpp
//!
//! 提供游戏内菜单系统，包括选项菜单、滑块控件等

/// 滑块物品宽度
pub const SLIDER_ITEM_WIDTH: i32 = 490;

/// 菜单顶部位置
pub const GMENU_TOP: i32 = 117;

/// 菜单项高度
pub const GMENU_ITEM_HEIGHT: i32 = 45;

/// 滑块步数
pub const SLIDER_STEPS: u32 = 10;

/// 菜单项标志
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItemFlags(pub u32);

impl MenuItemFlags {
    /// 启用状态
    pub const ENABLED: MenuItemFlags = MenuItemFlags(1);
    /// 滑块类型
    pub const SLIDER: MenuItemFlags = MenuItemFlags(2);
    /// 多选类型
    pub const SELECT: MenuItemFlags = MenuItemFlags(3);
    /// 是否可选中
    pub const SELECTABLE: MenuItemFlags = MenuItemFlags(4);
    /// 是否高亮
    pub const HIGHLIGHTED: MenuItemFlags = MenuItemFlags(8);
    /// 是否禁用
    pub const DISABLED: MenuItemFlags = MenuItemFlags(16);

    /// 检查是否设置了标志
    pub fn contains(&self, other: MenuItemFlags) -> bool {
        (self.0 & other.0) != 0
    }

    /// 设置标志
    pub fn set(&mut self, other: MenuItemFlags) {
        self.0 |= other.0;
    }

    /// 清除标志
    pub fn clear(&mut self, other: MenuItemFlags) {
        self.0 &= !other.0;
    }

    /// 切换标志
    pub fn toggle(&mut self, other: MenuItemFlags) {
        self.0 ^= other.0;
    }
}

impl Default for MenuItemFlags {
    fn default() -> Self {
        Self::ENABLED
    }
}

/// 菜单项回调类型
pub type MenuCallback = fn();

/// 菜单项
///
/// **C++ Reference**: `Source/gmenu.cpp:TMenuItem`
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// 项目标志
    pub flags: MenuItemFlags,
    /// 显示文本
    pub text: String,
    /// 回调函数 (可选)
    pub callback: Option<MenuCallback>,
    /// 当前值 (用于滑块)
    pub value: i32,
    /// 最大值 (用于滑块)
    pub max_value: i32,
}

impl MenuItem {
    /// 创建新菜单项
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            flags: MenuItemFlags::ENABLED,
            text: text.into(),
            callback: None,
            value: 0,
            max_value: 0,
        }
    }

    /// 创建滑块菜单项
    pub fn slider(text: impl Into<String>, value: i32, max_value: i32) -> Self {
        Self {
            flags: MenuItemFlags(MenuItemFlags::ENABLED.0 | MenuItemFlags::SLIDER.0),
            text: text.into(),
            callback: None,
            value,
            max_value,
        }
    }

    /// 创建带回调的菜单项
    pub fn with_callback(text: impl Into<String>, callback: MenuCallback) -> Self {
        Self {
            flags: MenuItemFlags::ENABLED,
            text: text.into(),
            callback: Some(callback),
            value: 0,
            max_value: 0,
        }
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.flags.contains(MenuItemFlags::ENABLED) && !self.flags.contains(MenuItemFlags::DISABLED)
    }

    /// 是否是滑块
    pub fn is_slider(&self) -> bool {
        self.flags.contains(MenuItemFlags::SLIDER)
    }

    /// 是否高亮
    pub fn is_highlighted(&self) -> bool {
        self.flags.contains(MenuItemFlags::HIGHLIGHTED)
    }

    /// 设置高亮状态
    pub fn set_highlighted(&mut self, highlighted: bool) {
        if highlighted {
            self.flags.set(MenuItemFlags::HIGHLIGHTED);
        } else {
            self.flags.clear(MenuItemFlags::HIGHLIGHTED);
        }
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.flags.set(MenuItemFlags::ENABLED);
            self.flags.clear(MenuItemFlags::DISABLED);
        } else {
            self.flags.clear(MenuItemFlags::ENABLED);
            self.flags.set(MenuItemFlags::DISABLED);
        }
    }

    /// 调用回调
    pub fn invoke(&self) {
        if let Some(callback) = self.callback {
            callback();
        }
    }
}

impl Default for MenuItem {
    fn default() -> Self {
        Self::new("")
    }
}

/// 游戏菜单
///
/// **C++ Reference**: `Source/gmenu.cpp` - menu state variables
#[derive(Debug)]
pub struct GMenu {
    /// 菜单项列表
    items: Vec<MenuItem>,
    /// 当前选中索引
    selected_index: usize,
    /// 是否显示
    visible: bool,
    /// 菜单宽度
    width: i32,
    /// 菜单位置 X
    pos_x: i32,
    /// 菜单位置 Y
    pos_y: i32,
    /// 当前拖动的滑块索引
    dragging_slider: Option<usize>,
}

impl GMenu {
    /// 创建新菜单
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            visible: false,
            width: SLIDER_ITEM_WIDTH,
            pos_x: 0,
            pos_y: GMENU_TOP,
            dragging_slider: None,
        }
    }

    /// 添加菜单项
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// 清空菜单
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = 0;
        self.dragging_slider = None;
    }

    /// 获取菜单项数量
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// 获取当前选中的项
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    /// 获取当前选中的项 (可变)
    pub fn selected_item_mut(&mut self) -> Option<&mut MenuItem> {
        self.items.get_mut(self.selected_index)
    }

    /// 获取选中索引
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// 设置选中索引
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.items.len() {
            // 清除旧的高亮
            if let Some(old_item) = self.items.get_mut(self.selected_index) {
                old_item.set_highlighted(false);
            }

            self.selected_index = index;

            // 设置新的高亮
            if let Some(new_item) = self.items.get_mut(self.selected_index) {
                new_item.set_highlighted(true);
            }
        }
    }

    /// 向上移动选择
    ///
    /// **C++ Reference**: `Source/gmenu.cpp:gmenu_up_down()`
    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let start = self.selected_index;
        loop {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };

            // 找到可选的项或回到起点
            if self.items[self.selected_index].is_enabled() || self.selected_index == start {
                break;
            }
        }

        self.update_highlight();
    }

    /// 向下移动选择
    ///
    /// **C++ Reference**: `Source/gmenu.cpp:gmenu_up_down()`
    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let start = self.selected_index;
        loop {
            self.selected_index = (self.selected_index + 1) % self.items.len();

            // 找到可选的项或回到起点
            if self.items[self.selected_index].is_enabled() || self.selected_index == start {
                break;
            }
        }

        self.update_highlight();
    }

    /// 更新高亮状态
    fn update_highlight(&mut self) {
        for (i, item) in self.items.iter_mut().enumerate() {
            item.set_highlighted(i == self.selected_index);
        }
    }

    /// 选择当前项 (按下回车)
    ///
    /// **C++ Reference**: `Source/gmenu.cpp:gmenu_presskey()`
    pub fn select_current(&mut self) {
        if let Some(item) = self.items.get(self.selected_index) {
            if item.is_enabled() && !item.is_slider() {
                item.invoke();
            }
        }
    }

    /// 调整滑块值
    ///
    /// **C++ Reference**: `Source/gmenu.cpp:gmenu_left_right()`
    ///
    /// # Arguments
    /// * `delta` - 变化量 (负数向左，正数向右)
    pub fn adjust_slider(&mut self, delta: i32) {
        if let Some(item) = self.items.get_mut(self.selected_index) {
            if item.is_slider() {
                let new_value = (item.value + delta).clamp(0, item.max_value);
                item.value = new_value;
            }
        }
    }

    /// 左移滑块
    pub fn slider_left(&mut self) {
        self.adjust_slider(-1);
    }

    /// 右移滑块
    pub fn slider_right(&mut self) {
        self.adjust_slider(1);
    }

    /// 显示菜单
    pub fn show(&mut self) {
        self.visible = true;
        self.update_highlight();
    }

    /// 隐藏菜单
    pub fn hide(&mut self) {
        self.visible = false;
        self.dragging_slider = None;
    }

    /// 切换显示状态
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    /// 是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 获取菜单宽度
    pub fn width(&self) -> i32 {
        self.width
    }

    /// 设置菜单宽度
    pub fn set_width(&mut self, width: i32) {
        self.width = width;
    }

    /// 获取菜单位置
    pub fn position(&self) -> (i32, i32) {
        (self.pos_x, self.pos_y)
    }

    /// 设置菜单位置
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.pos_x = x;
        self.pos_y = y;
    }

    /// 计算菜单项的 Y 坐标
    ///
    /// **C++ Reference**: `Source/gmenu.cpp` - drawing calculations
    pub fn item_y(&self, index: usize) -> i32 {
        self.pos_y + (index as i32) * GMENU_ITEM_HEIGHT
    }

    /// 处理鼠标点击
    ///
    /// **C++ Reference**: `Source/gmenu.cpp:gmenu_on_mouse_move()`
    ///
    /// # Arguments
    /// * `x` - 鼠标 X 坐标
    /// * `y` - 鼠标 Y 坐标
    ///
    /// # Returns
    /// 是否命中菜单
    pub fn handle_click(&mut self, _x: i32, y: i32) -> bool {
        if !self.visible || self.items.is_empty() {
            return false;
        }

        let relative_y = y - self.pos_y;
        if relative_y < 0 {
            return false;
        }

        let index = (relative_y / GMENU_ITEM_HEIGHT) as usize;
        if index < self.items.len() {
            self.set_selected_index(index);

            // 如果是滑块，开始拖动
            if self.items[index].is_slider() {
                self.dragging_slider = Some(index);
            } else {
                self.select_current();
            }
            return true;
        }

        false
    }

    /// 处理鼠标释放
    pub fn handle_mouse_up(&mut self) {
        self.dragging_slider = None;
    }

    /// 获取所有菜单项
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// 获取所有菜单项 (可变)
    pub fn items_mut(&mut self) -> &mut [MenuItem] {
        &mut self.items
    }

    /// 按索引获取菜单项
    pub fn item(&self, index: usize) -> Option<&MenuItem> {
        self.items.get(index)
    }

    /// 按索引获取菜单项 (可变)
    pub fn item_mut(&mut self, index: usize) -> Option<&mut MenuItem> {
        self.items.get_mut(index)
    }
}

impl Default for GMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// 预定义菜单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    /// 主菜单
    Main,
    /// 选项菜单
    Options,
    /// 音频选项
    Audio,
    /// 图形选项
    Graphics,
    /// 游戏选项
    Gameplay,
    /// 确认对话框
    Confirm,
}

/// 创建选项菜单
///
/// **C++ Reference**: `Source/gmenu.cpp:GmOptItems`
pub fn create_options_menu() -> GMenu {
    let mut menu = GMenu::new();

    menu.add_item(MenuItem::slider("音乐音量", 5, SLIDER_STEPS as i32));
    menu.add_item(MenuItem::slider("音效音量", 5, SLIDER_STEPS as i32));
    menu.add_item(MenuItem::slider("伽马值", 5, SLIDER_STEPS as i32));
    menu.add_item(MenuItem::new("返回"));

    menu
}

/// 计算滑块位置
///
/// **C++ Reference**: `Source/gmenu.cpp:gmenu_slider_get()`
///
/// # Arguments
/// * `item` - 菜单项
/// * `slider_width` - 滑块总宽度
///
/// # Returns
/// 滑块当前位置 (像素)
pub fn slider_get_position(item: &MenuItem, slider_width: i32) -> i32 {
    if item.max_value == 0 {
        return 0;
    }
    (item.value * slider_width) / item.max_value
}

/// 从位置设置滑块值
///
/// **C++ Reference**: `Source/gmenu.cpp:gmenu_slider_set()`
///
/// # Arguments
/// * `item` - 菜单项
/// * `position` - 像素位置
/// * `slider_width` - 滑块总宽度
pub fn slider_set_from_position(item: &mut MenuItem, position: i32, slider_width: i32) {
    if slider_width == 0 {
        return;
    }
    let new_value = (position * item.max_value) / slider_width;
    item.value = new_value.clamp(0, item.max_value);
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_flags_contains() {
        let flags = MenuItemFlags(MenuItemFlags::ENABLED.0 | MenuItemFlags::SLIDER.0);
        assert!(flags.contains(MenuItemFlags::ENABLED));
        assert!(flags.contains(MenuItemFlags::SLIDER));
        assert!(!flags.contains(MenuItemFlags::HIGHLIGHTED));
    }

    #[test]
    fn test_menu_item_flags_set_clear() {
        let mut flags = MenuItemFlags::ENABLED;

        flags.set(MenuItemFlags::HIGHLIGHTED);
        assert!(flags.contains(MenuItemFlags::HIGHLIGHTED));

        flags.clear(MenuItemFlags::HIGHLIGHTED);
        assert!(!flags.contains(MenuItemFlags::HIGHLIGHTED));
    }

    #[test]
    fn test_menu_item_new() {
        let item = MenuItem::new("Test");
        assert_eq!(item.text, "Test");
        assert!(item.is_enabled());
        assert!(!item.is_slider());
        assert!(!item.is_highlighted());
    }

    #[test]
    fn test_menu_item_slider() {
        let item = MenuItem::slider("Volume", 5, 10);
        assert_eq!(item.text, "Volume");
        assert!(item.is_slider());
        assert_eq!(item.value, 5);
        assert_eq!(item.max_value, 10);
    }

    #[test]
    fn test_menu_item_set_enabled() {
        let mut item = MenuItem::new("Test");
        assert!(item.is_enabled());

        item.set_enabled(false);
        assert!(!item.is_enabled());

        item.set_enabled(true);
        assert!(item.is_enabled());
    }

    #[test]
    fn test_gmenu_new() {
        let menu = GMenu::new();
        assert_eq!(menu.item_count(), 0);
        assert!(!menu.is_visible());
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn test_gmenu_add_item() {
        let mut menu = GMenu::new();
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));

        assert_eq!(menu.item_count(), 2);
        assert_eq!(menu.item(0).unwrap().text, "Item 1");
        assert_eq!(menu.item(1).unwrap().text, "Item 2");
    }

    #[test]
    fn test_gmenu_clear() {
        let mut menu = GMenu::new();
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));

        menu.clear();
        assert_eq!(menu.item_count(), 0);
    }

    #[test]
    fn test_gmenu_move_up_down() {
        let mut menu = GMenu::new();
        menu.add_item(MenuItem::new("Item 0"));
        menu.add_item(MenuItem::new("Item 1"));
        menu.add_item(MenuItem::new("Item 2"));

        assert_eq!(menu.selected_index(), 0);

        menu.move_down();
        assert_eq!(menu.selected_index(), 1);

        menu.move_down();
        assert_eq!(menu.selected_index(), 2);

        menu.move_down(); // 应该循环回 0
        assert_eq!(menu.selected_index(), 0);

        menu.move_up(); // 应该循环到 2
        assert_eq!(menu.selected_index(), 2);
    }

    #[test]
    fn test_gmenu_skip_disabled() {
        let mut menu = GMenu::new();
        menu.add_item(MenuItem::new("Item 0"));

        let mut disabled = MenuItem::new("Item 1");
        disabled.set_enabled(false);
        menu.add_item(disabled);

        menu.add_item(MenuItem::new("Item 2"));

        assert_eq!(menu.selected_index(), 0);

        menu.move_down(); // 应该跳过禁用项到 2
        assert_eq!(menu.selected_index(), 2);
    }

    #[test]
    fn test_gmenu_adjust_slider() {
        let mut menu = GMenu::new();
        menu.add_item(MenuItem::slider("Volume", 5, 10));

        menu.adjust_slider(1);
        assert_eq!(menu.selected_item().unwrap().value, 6);

        menu.adjust_slider(-2);
        assert_eq!(menu.selected_item().unwrap().value, 4);

        // 测试上限
        menu.adjust_slider(100);
        assert_eq!(menu.selected_item().unwrap().value, 10);

        // 测试下限
        menu.adjust_slider(-100);
        assert_eq!(menu.selected_item().unwrap().value, 0);
    }

    #[test]
    fn test_gmenu_visibility() {
        let mut menu = GMenu::new();
        assert!(!menu.is_visible());

        menu.show();
        assert!(menu.is_visible());

        menu.hide();
        assert!(!menu.is_visible());

        menu.toggle();
        assert!(menu.is_visible());

        menu.toggle();
        assert!(!menu.is_visible());
    }

    #[test]
    fn test_gmenu_position() {
        let mut menu = GMenu::new();
        assert_eq!(menu.position(), (0, GMENU_TOP));

        menu.set_position(100, 200);
        assert_eq!(menu.position(), (100, 200));
    }

    #[test]
    fn test_gmenu_item_y() {
        let menu = GMenu::new();
        assert_eq!(menu.item_y(0), GMENU_TOP);
        assert_eq!(menu.item_y(1), GMENU_TOP + GMENU_ITEM_HEIGHT);
        assert_eq!(menu.item_y(2), GMENU_TOP + 2 * GMENU_ITEM_HEIGHT);
    }

    #[test]
    fn test_create_options_menu() {
        let menu = create_options_menu();
        assert_eq!(menu.item_count(), 4);

        // 检查滑块项
        assert!(menu.item(0).unwrap().is_slider());
        assert!(menu.item(1).unwrap().is_slider());
        assert!(menu.item(2).unwrap().is_slider());

        // 检查返回按钮
        assert!(!menu.item(3).unwrap().is_slider());
    }

    #[test]
    fn test_slider_position_calculation() {
        let item = MenuItem::slider("Test", 5, 10);

        let pos = slider_get_position(&item, 100);
        assert_eq!(pos, 50); // 5/10 * 100 = 50

        let item2 = MenuItem::slider("Test", 0, 10);
        let pos2 = slider_get_position(&item2, 100);
        assert_eq!(pos2, 0);

        let item3 = MenuItem::slider("Test", 10, 10);
        let pos3 = slider_get_position(&item3, 100);
        assert_eq!(pos3, 100);
    }

    #[test]
    fn test_slider_set_from_position() {
        let mut item = MenuItem::slider("Test", 0, 10);

        slider_set_from_position(&mut item, 50, 100);
        assert_eq!(item.value, 5);

        slider_set_from_position(&mut item, 0, 100);
        assert_eq!(item.value, 0);

        slider_set_from_position(&mut item, 100, 100);
        assert_eq!(item.value, 10);

        // 测试越界
        slider_set_from_position(&mut item, 150, 100);
        assert_eq!(item.value, 10); // 被限制到 max
    }

    #[test]
    fn test_constants() {
        assert_eq!(SLIDER_ITEM_WIDTH, 490);
        assert_eq!(GMENU_TOP, 117);
        assert_eq!(GMENU_ITEM_HEIGHT, 45);
        assert_eq!(SLIDER_STEPS, 10);
    }
}
