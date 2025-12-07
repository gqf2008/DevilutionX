//! 死亡画面/星图任务系统 (M33)
//!
//! 从 Source/doom.cpp 移植
//! 处理星图任务（Map of the Stars）的显示

/// 星图显示标志
pub static mut DOOM_FLAG: bool = false;

/// 星图精灵资源路径
pub const DOOM_SPRITE_PATH: &str = "items/map/mapztown";

/// 星图精灵宽度
pub const DOOM_SPRITE_WIDTH: i32 = 640;

/// 星图精灵高度
pub const DOOM_SPRITE_HEIGHT: i32 = 352;

/// 星图精灵数据（占位）
#[derive(Debug, Clone)]
pub struct DoomSprite {
    /// 精灵宽度
    pub width: i32,
    /// 精灵高度
    pub height: i32,
    /// 是否已加载
    pub loaded: bool,
}

impl Default for DoomSprite {
    fn default() -> Self {
        Self {
            width: DOOM_SPRITE_WIDTH,
            height: DOOM_SPRITE_HEIGHT,
            loaded: false,
        }
    }
}

impl DoomSprite {
    /// 创建新的星图精灵
    pub fn new() -> Self {
        Self::default()
    }

    /// 加载精灵
    pub fn load(&mut self) {
        // 实际实现需要加载CEL文件
        // DoomSprite = LoadCel("items\\map\\mapztown", 640);
        self.loaded = true;
    }

    /// 卸载精灵
    pub fn unload(&mut self) {
        self.loaded = false;
    }
}

/// 星图系统管理器
#[derive(Debug, Default)]
pub struct DoomSystem {
    /// 星图精灵
    pub sprite: DoomSprite,
    /// 是否显示星图
    pub visible: bool,
}

impl DoomSystem {
    /// 创建新的星图系统
    pub fn new() -> Self {
        Self {
            sprite: DoomSprite::new(),
            visible: false,
        }
    }

    /// 初始化星图
    ///
    /// 从 doom_init 移植
    pub fn init(&mut self) {
        self.sprite.load();
        self.visible = true;
    }

    /// 关闭星图
    ///
    /// 从 doom_close 移植
    pub fn close(&mut self) {
        self.visible = false;
        self.sprite.unload();
    }

    /// 检查是否应该绘制
    pub fn should_draw(&self) -> bool {
        self.visible && self.sprite.loaded
    }

    /// 获取绘制位置
    ///
    /// 返回相对于UI矩形的偏移量
    pub fn get_draw_position(&self) -> (i32, i32) {
        // 从 C++: GetUIRectangle().position + Displacement { 0, 352 }
        (0, DOOM_SPRITE_HEIGHT)
    }

    /// 绘制星图（占位实现）
    ///
    /// 实际渲染需要与渲染系统集成
    pub fn draw(&self) -> bool {
        if !self.should_draw() {
            return false;
        }
        
        // 实际实现:
        // ClxDraw(out, GetUIRectangle().position + Displacement { 0, 352 }, (*DoomSprite)[0]);
        true
    }

    /// 切换星图显示
    pub fn toggle(&mut self) {
        if self.visible {
            self.close();
        } else {
            self.init();
        }
    }
}

/// 全局星图系统实例
pub static mut DOOM_SYSTEM: Option<DoomSystem> = None;

/// 初始化星图
///
/// 从 doom_init 移植
///
/// # Safety
/// 必须在单线程环境中调用
pub unsafe fn doom_init() {
    DOOM_FLAG = true;
    if let Some(ref mut doom) = DOOM_SYSTEM {
        doom.init();
    } else {
        let mut system = DoomSystem::new();
        system.init();
        DOOM_SYSTEM = Some(system);
    }
}

/// 关闭星图
///
/// 从 doom_close 移植
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn doom_close() {
    DOOM_FLAG = false;
    if let Some(ref mut doom) = DOOM_SYSTEM {
        doom.close();
    }
}

/// 绘制星图
///
/// 从 doom_draw 移植
///
/// # Safety
/// 必须在初始化后调用
pub unsafe fn doom_draw() -> bool {
    if !DOOM_FLAG {
        return false;
    }
    
    if let Some(ref doom) = DOOM_SYSTEM {
        doom.draw()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doom_sprite_creation() {
        let sprite = DoomSprite::new();
        assert!(!sprite.loaded);
        assert_eq!(sprite.width, DOOM_SPRITE_WIDTH);
        assert_eq!(sprite.height, DOOM_SPRITE_HEIGHT);
    }

    #[test]
    fn test_doom_sprite_load() {
        let mut sprite = DoomSprite::new();
        sprite.load();
        assert!(sprite.loaded);
        
        sprite.unload();
        assert!(!sprite.loaded);
    }

    #[test]
    fn test_doom_system_creation() {
        let system = DoomSystem::new();
        assert!(!system.visible);
        assert!(!system.sprite.loaded);
    }

    #[test]
    fn test_doom_system_init_close() {
        let mut system = DoomSystem::new();
        
        system.init();
        assert!(system.visible);
        assert!(system.sprite.loaded);
        
        system.close();
        assert!(!system.visible);
        assert!(!system.sprite.loaded);
    }

    #[test]
    fn test_doom_system_should_draw() {
        let mut system = DoomSystem::new();
        
        // 未初始化
        assert!(!system.should_draw());
        
        // 初始化后
        system.init();
        assert!(system.should_draw());
        
        // 关闭后
        system.close();
        assert!(!system.should_draw());
    }

    #[test]
    fn test_doom_system_toggle() {
        let mut system = DoomSystem::new();
        
        system.toggle();
        assert!(system.visible);
        
        system.toggle();
        assert!(!system.visible);
    }

    #[test]
    fn test_doom_draw_position() {
        let system = DoomSystem::new();
        let (x, y) = system.get_draw_position();
        assert_eq!(x, 0);
        assert_eq!(y, DOOM_SPRITE_HEIGHT);
    }
}
