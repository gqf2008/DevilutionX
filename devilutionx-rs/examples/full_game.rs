//! DevilutionX-RS 完整游戏演示
//!
//! 使用真实的Diablo资源文件，包括:
//! - CLX字体 (从devilutionx.mpq)
//! - PCX图像 (从spawn.mpq)
//! - 完整的UI系统
//! - 地牢生成系统
//! - 真实瓦片地图渲染

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;
use devilutionx_rs::engine::text_render::{GameFont, TextColor, TextRenderer};
use devilutionx_rs::engine::dungeon::{
    DungeonLevelData, TileDecoder, FRAME_WIDTH, FRAME_HEIGHT,
    DungeonType as LevelType, TilData,
};
use devilutionx_rs::game::dungeon::{DungeonMap, TileType};
use devilutionx_rs::game::types::DungeonType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use std::time::{Duration, Instant};
use std::collections::HashMap;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

/// 地图渲染常量 - 等距瓦片
const TILE_WIDTH: i32 = 64;   // Diablo mega-tile 宽度 (等距菱形)
const TILE_HEIGHT: i32 = 32;  // Diablo mega-tile 高度 (等距菱形)
const TILE_SIZE: i32 = 32;    // 用于移动和碰撞检测的网格大小
const MAP_WIDTH: usize = 40;
const MAP_HEIGHT: usize = 40;

/// 光照级别 (0=全亮, 15=全黑)
const MAX_LIGHT_LEVEL: u8 = 15;
const TORCH_LIGHT_RADIUS: i32 = 8;

/// 瓦片纹理缓存
struct TileTextureCache<'a> {
    /// 帧纹理缓存: key = (frame_index, tile_type)
    frame_textures: HashMap<(u16, u8), Texture<'a>>,
}

/// 英雄职业
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeroClass {
    Warrior,
    Rogue,
    Sorcerer,
}

impl HeroClass {
    fn name(&self) -> &'static str {
        match self {
            HeroClass::Warrior => "Warrior",
            HeroClass::Rogue => "Rogue",
            HeroClass::Sorcerer => "Sorcerer",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            HeroClass::Warrior => "A mighty fighter skilled in melee combat",
            HeroClass::Rogue => "A swift archer with deadly precision",
            HeroClass::Sorcerer => "A powerful mage wielding arcane magic",
        }
    }
}

/// 玩家状态
struct Player {
    x: i32,
    y: i32,
    class: HeroClass,
    health: i32,
    max_health: i32,
    mana: i32,
    max_mana: i32,
    level: i32,
    gold: i32,
}

impl Player {
    fn new(class: HeroClass, spawn_x: i32, spawn_y: i32) -> Self {
        let (health, mana) = match class {
            HeroClass::Warrior => (100, 10),
            HeroClass::Rogue => (70, 30),
            HeroClass::Sorcerer => (40, 80),
        };
        Self {
            x: spawn_x,
            y: spawn_y,
            class,
            health,
            max_health: health,
            mana,
            max_mana: mana,
            level: 1,
            gold: 100,
        }
    }
}

/// 游戏状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    /// 标题画面
    Title,
    /// 主菜单
    MainMenu,
    /// 选择英雄
    SelectHero,
    /// 游戏中
    InGame,
}

/// 菜单项
struct MenuItem {
    text: &'static str,
    y: i32,
}

/// 游戏资源
struct GameAssets {
    /// 标题背景
    title_bg: Vec<u8>,
    /// 菜单背景
    menu_bg: Vec<u8>,
    /// Logo动画帧
    logo_frames: Vec<Vec<u8>>,
    /// 选择指示器帧
    focus_frames: Vec<Vec<u8>>,
    /// 英雄肖像 (3个职业)
    hero_portraits: Vec<Vec<u8>>,
    /// 英雄肖像宽度
    hero_portrait_width: u32,
    /// 英雄选择背景
    selhero_bg: Vec<u8>,
    /// 调色板
    palette: [u8; 768],
}

impl GameAssets {
    fn load(spawn: &mut MpqArchive) -> Result<Self, Box<dyn std::error::Error>> {
        // 加载标题背景
        let title_pcx = spawn.read_file("ui_art\\title.pcx")?;
        let title_img = PcxImage::decode(&title_pcx).ok_or("Failed to decode title.pcx")?;
        let title_bg = title_img.to_rgba();

        // 加载主菜单背景 (spawn版使用swmmenu.pcx，不含火焰，由动画Logo覆盖)
        let menu_pcx = spawn.read_file("ui_art\\swmmenu.pcx")?;
        let menu_img = PcxImage::decode(&menu_pcx).ok_or("Failed to decode swmmenu.pcx")?;
        let menu_bg = menu_img.to_rgba();

        // 加载UI调色板 (用于CLX字体) - 从 diablo.pal 加载
        let mut palette = [0u8; 768];
        if let Ok(pal_data) = spawn.read_file("ui_art\\diablo.pal") {
            if pal_data.len() >= 768 {
                palette.copy_from_slice(&pal_data[..768]);
                println!("加载 UI 调色板: ui_art\\diablo.pal ({} bytes)", pal_data.len());
            }
        } else {
            // 如果没有 diablo.pal，使用 title.pcx 的调色板作为后备
            println!("警告: 无法加载 diablo.pal，使用 title.pcx 调色板");
            for i in 0..256 {
                let c = &title_img.palette[i];
                palette[i * 3] = c.r;
                palette[i * 3 + 1] = c.g;
                palette[i * 3 + 2] = c.b;
            }
        }

        // 加载Logo动画 (15帧, 每帧216像素高)
        let logo_pcx = spawn.read_file("ui_art\\logo.pcx")?;
        let logo_img = PcxImage::decode(&logo_pcx).ok_or("Failed to decode logo.pcx")?;

        // 调试：检查透明色的RGB值
        let transparent_color = &logo_img.palette[250];
        println!("透明色(索引250)的RGB: ({}, {}, {})", transparent_color.r, transparent_color.g, transparent_color.b);

        let logo_frames = extract_frames(&logo_img, 216, 15);

        // 检查第一帧底部区域(y=192到y=216)的透明像素比例
        let first_frame = &logo_frames[0];
        let width = 550usize;
        let bottom_start = 192;
        let bottom_end = 216;
        let mut bottom_transparent = 0;
        let mut bottom_total = 0;
        for y in bottom_start..bottom_end {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                if first_frame[idx + 3] == 0 {
                    bottom_transparent += 1;
                }
                bottom_total += 1;
            }
        }
        println!("第一帧底部区域(y=192-216): 透明像素: {}, 总像素: {}, 透明比例: {:.1}%",
            bottom_transparent, bottom_total, bottom_transparent as f64 / bottom_total as f64 * 100.0);

        // 加载焦点指示器
        let focus_pcx = spawn.read_file("ui_art\\focus42.pcx")?;
        let focus_img = PcxImage::decode(&focus_pcx).ok_or("Failed to decode focus42.pcx")?;
        let focus_frames = extract_frames(&focus_img, 42, 8);

        // 加载英雄肖像 (每个76像素高)
        let heros_pcx = spawn.read_file("ui_art\\heros.pcx")?;
        let heros_img = PcxImage::decode(&heros_pcx).ok_or("Failed to decode heros.pcx")?;
        let hero_portrait_width = heros_img.width as u32;
        // 根据图片高度计算肖像数量 (每个76像素)
        let num_portraits = (heros_img.height as usize) / 76;
        let hero_portraits = extract_frames(&heros_img, 76, num_portraits);

        // 加载英雄选择背景
        let selhero_pcx = spawn.read_file("ui_art\\selhero.pcx")?;
        let selhero_img = PcxImage::decode(&selhero_pcx).ok_or("Failed to decode selhero.pcx")?;
        let selhero_bg = selhero_img.to_rgba();

        Ok(Self {
            title_bg,
            menu_bg,
            logo_frames,
            focus_frames,
            hero_portraits,
            hero_portrait_width,
            selhero_bg,
            palette,
        })
    }
}

/// 从PCX图像中提取动画帧
/// Diablo UI使用调色板索引250作为透明色
fn extract_frames(img: &PcxImage, frame_height: u32, num_frames: usize) -> Vec<Vec<u8>> {
    extract_frames_with_transparency(img, frame_height, num_frames, 250)
}

/// 从PCX图像中提取动画帧，指定透明色
fn extract_frames_with_transparency(
    img: &PcxImage,
    frame_height: u32,
    num_frames: usize,
    transparent_color: u8,
) -> Vec<Vec<u8>> {
    let mut frames = Vec::with_capacity(num_frames);
    let width = img.width as usize;
    let fh = frame_height as usize;

    for i in 0..num_frames {
        let mut frame = vec![0u8; width * fh * 4];
        let start_y = i * fh;

        for y in 0..fh {
            let src_y = start_y + y;
            if src_y >= img.height as usize {
                break;
            }
            for x in 0..width {
                let src_idx = src_y * width + x;
                let dst_idx = (y * width + x) * 4;
                if src_idx < img.pixels.len() {
                    let color_idx = img.pixels[src_idx];
                    let color = &img.palette[color_idx as usize];
                    // 检查是否为透明色 (Diablo UI通常使用250)
                    let is_transparent = color_idx == transparent_color;
                    frame[dst_idx] = color.r;
                    frame[dst_idx + 1] = color.g;
                    frame[dst_idx + 2] = color.b;
                    frame[dst_idx + 3] = if is_transparent { 0 } else { 255 };
                }
            }
        }
        frames.push(frame);
    }
    frames
}

/// 创建SDL纹理
fn create_texture_rgba<'a>(
    creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<sdl2::render::Texture<'a>, Box<dyn std::error::Error>> {
    let mut texture =
        creator.create_texture_streaming(PixelFormatEnum::ABGR8888, width, height)?;
    texture.update(None, data, width as usize * 4)?;
    Ok(texture)
}

/// 创建带混合模式的SDL纹理 (用于透明图像)
fn create_texture_rgba_blended<'a>(
    creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<sdl2::render::Texture<'a>, Box<dyn std::error::Error>> {
    let mut texture =
        creator.create_texture_streaming(PixelFormatEnum::ABGR8888, width, height)?;
    texture.set_blend_mode(sdl2::render::BlendMode::Blend);
    texture.update(None, data, width as usize * 4)?;
    Ok(texture)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DevilutionX-RS 完整演示 ===");

    // 初始化SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("DevilutionX-RS", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    // 打开MPQ文件
    println!("加载资源文件...");
    let mut spawn = MpqArchive::open("spawn.mpq")?;
    let mut devilutionx = MpqArchive::open("devilutionx.mpq")?;

    // 加载游戏资源
    println!("加载图像资源...");
    let assets = GameAssets::load(&mut spawn)?;

    // 创建纹理
    let title_texture = create_texture_rgba(&texture_creator, &assets.title_bg, 640, 480)?;
    let menu_texture = create_texture_rgba(&texture_creator, &assets.menu_bg, 640, 480)?;

    // 创建Logo纹理 (带透明混合) - 15帧, 550x216每帧
    let logo_textures: Vec<_> = assets
        .logo_frames
        .iter()
        .map(|f| create_texture_rgba_blended(&texture_creator, f, 550, 216))
        .collect::<Result<Vec<_>, _>>()?;

    // 创建焦点指示器纹理 (带透明混合)
    let focus_textures: Vec<_> = assets
        .focus_frames
        .iter()
        .map(|f| create_texture_rgba_blended(&texture_creator, f, 42, 42))
        .collect::<Result<Vec<_>, _>>()?;

    // 创建英雄选择背景纹理
    let selhero_texture = create_texture_rgba(&texture_creator, &assets.selhero_bg, 640, 480)?;

    // 创建英雄肖像纹理 (每个76像素高)
    let hero_portrait_textures: Vec<_> = assets
        .hero_portraits
        .iter()
        .map(|f| create_texture_rgba_blended(&texture_creator, f, assets.hero_portrait_width, 76))
        .collect::<Result<Vec<_>, _>>()?;

    // 初始化CLX字体系统
    println!("加载CLX字体...");
    let mut text_renderer = TextRenderer::new();
    text_renderer.set_palette(&assets.palette);

    // 加载基本字体 (ASCII)
    text_renderer.load_font(&mut devilutionx, GameFont::Font24, &[0x00]);
    text_renderer.load_font(&mut devilutionx, GameFont::Font30, &[0x00]);
    text_renderer.load_font(&mut devilutionx, GameFont::Font42, &[0x00]);

    // 加载颜色转换表
    let gold_loaded = text_renderer.load_color_translation(&mut devilutionx, TextColor::UiGold);
    let silver_loaded = text_renderer.load_color_translation(&mut devilutionx, TextColor::UiSilver);
    println!("TRN加载: UiGold={}, UiSilver={}", gold_loaded, silver_loaded);

    // 创建文本渲染缓冲区
    let mut text_buffer = vec![0u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4];

    // 游戏状态 - 直接跳到游戏内测试渲染
    let mut state = GameState::InGame;
    let mut selected_item = 0;
    let mut selected_hero = 0usize;  // 英雄选择索引
    let start_time = Instant::now();

    // 英雄职业列表
    let hero_classes = [HeroClass::Warrior, HeroClass::Rogue, HeroClass::Sorcerer];

    // 直接创建玩家和地牢
    let seed = 12345u64;
    let map = DungeonMap::generate(MAP_WIDTH, MAP_HEIGHT, DungeonType::Cathedral, 1, seed);
    let spawn_point = map.spawn_point;
    let mut player: Option<Player> = Some(Player::new(hero_classes[0], spawn_point.x, spawn_point.y));
    let mut dungeon: Option<DungeonMap> = Some(map);
    let mut camera_x: i32 = (spawn_point.x - spawn_point.y) * (TILE_WIDTH / 2);
    let mut camera_y: i32 = (spawn_point.x + spawn_point.y) * (TILE_HEIGHT / 2);

    // 地图拖动状态
    let mut dragging = false;
    let mut camera_offset_x: i32 = 0;  // 独立的相机偏移（用于调试）
    let mut camera_offset_y: i32 = 0;

    // 加载真正的瓦片数据
    // spawn.mpq 只包含 Cathedral 和 Town
    println!("加载Cathedral瓦片数据...");
    let level_data = DungeonLevelData::load_from_mpq(&mut spawn, LevelType::Cathedral)?;
    println!("  - 调色板: {} 字节", level_data.palette.colors.len());
    println!("  - SOL: {} 项", level_data.sol.properties.len());
    println!("  - MIN: {} 个 mega-tiles", level_data.min.mega_tiles.len());
    println!("  - CEL: {} 字节", level_data.level_cel.len());

    // 预渲染所有 mega-tile 纹理
    // Diablo mega-tile 布局 (从C++):
    // - 每个 mega-tile 包含最多 16 个 micro-tiles (32x32 的块)
    // - 块排列: blocks[0,1] 在底部左右, blocks[2,3] 在上面一行, 以此类推
    // - 总高度: 5行 * 32 = 160 像素 (或根据实际块数)
    // - 宽度: 64 像素 (左右两块各32像素)
    println!("预渲染瓦片纹理...");
    let mut mega_tile_textures: Vec<sdl2::render::Texture> = Vec::new();
    let texture_height: usize = 160; // 5 rows of 32-pixel blocks

    for (i, mega_tile) in level_data.min.mega_tiles.iter().enumerate() {
        let mut tile_rgba = vec![0u8; TILE_WIDTH as usize * texture_height * 4];

        let num_blocks = level_data.min.blocks_per_tile.min(10);
        let num_rows = (num_blocks + 1) / 2; // 向上取整的行数

        for block_idx in 0..num_blocks {
            let block = &mega_tile.blocks[block_idx];
            if !block.has_value() {
                continue;
            }

            let frame = block.frame();
            let tile_type = block.tile_type();

            if let Some(pixels) = TileDecoder::decode_tile(
                &level_data.level_cel,
                frame,
                tile_type,
                &level_data.palette,
            ) {
                // 计算块在 mega-tile 中的位置
                // C++ 布局: blocks[0,1] 在底部 (最大Y), blocks[2,3] 在上面一行
                let col = block_idx & 1;  // 0 = 左, 1 = 右
                let row = block_idx / 2;  // 从0开始的行号

                let block_x = col * FRAME_WIDTH;
                // Y坐标: 从底部往上计算
                // 纹理底部 = texture_height - FRAME_HEIGHT = 128
                // row 0 (底部): y = 128
                // row 1: y = 96
                // row 2: y = 64
                // ...
                let block_y = texture_height - (row + 1) * FRAME_HEIGHT;

                // 复制像素到 mega-tile 缓冲区
                for py in 0..FRAME_HEIGHT {
                    for px in 0..FRAME_WIDTH {
                        let src_idx = (py * FRAME_WIDTH + px) * 4;
                        let dst_x = block_x + px;
                        let dst_y = block_y + py;
                        let dst_idx = (dst_y * TILE_WIDTH as usize + dst_x) * 4;
                        if dst_idx + 3 < tile_rgba.len() && src_idx + 3 < pixels.len() {
                            tile_rgba[dst_idx] = pixels[src_idx];
                            tile_rgba[dst_idx + 1] = pixels[src_idx + 1];
                            tile_rgba[dst_idx + 2] = pixels[src_idx + 2];
                            tile_rgba[dst_idx + 3] = pixels[src_idx + 3];
                        }
                    }
                }
            }
        }

        // 创建纹理
        let mut texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::ABGR8888,
            TILE_WIDTH as u32,
            texture_height as u32,
        )?;
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        texture.update(None, &tile_rgba, TILE_WIDTH as usize * 4)?;
        mega_tile_textures.push(texture);

        if i % 100 == 0 {
            println!("  已渲染 {}/{} mega-tiles", i, level_data.min.mega_tiles.len());
        }
    }
    println!("  完成! 共 {} 个瓦片纹理", mega_tile_textures.len());

    // 菜单项 - C++中从y=192开始，每项间距43像素
    let menu_items = [
        MenuItem {
            text: "Single Player",
            y: 192,
        },
        MenuItem {
            text: "Multi Player",
            y: 235,
        },
        MenuItem {
            text: "Replay Intro",
            y: 278,
        },
        MenuItem {
            text: "Show Credits",
            y: 321,
        },
        MenuItem {
            text: "Exit Diablo",
            y: 364,
        },
    ];

    let mut event_pump = sdl_context.event_pump()?;

    println!("游戏启动成功!");
    println!("按任意键进入主菜单...");

    'running: loop {
        let frame_time = start_time.elapsed().as_millis() as u32;
        // Logo动画: 15帧, 每帧约66ms (约15fps) - 与C++的GetAnimationFrame(frames, 60)一致
        // 60表示每60ms换一帧，所以是 ticks / 60 % frames
        let logo_frame = ((frame_time / 60) % logo_textures.len() as u32) as usize;
        // Focus动画: 8帧, 同样60ms每帧
        let focus_frame = ((frame_time / 60) % focus_textures.len() as u32) as usize;

        // 事件处理
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    if state == GameState::MainMenu {
                        break 'running;
                    } else if state != GameState::Title {
                        state = GameState::MainMenu;
                    } else {
                        break 'running;
                    }
                }

                Event::KeyDown { keycode, .. } => match state {
                    GameState::Title => {
                        state = GameState::MainMenu;
                    }
                    GameState::MainMenu => match keycode {
                        Some(Keycode::Up) => {
                            if selected_item > 0 {
                                selected_item -= 1;
                            } else {
                                selected_item = menu_items.len() - 1;
                            }
                        }
                        Some(Keycode::Down) => {
                            selected_item = (selected_item + 1) % menu_items.len();
                        }
                        Some(Keycode::Return) | Some(Keycode::Space) => {
                            handle_menu_select(selected_item, &mut state);
                        }
                        _ => {}
                    },
                    GameState::SelectHero => match keycode {
                        Some(Keycode::Up) => {
                            if selected_hero > 0 {
                                selected_hero -= 1;
                            } else {
                                selected_hero = hero_classes.len() - 1;
                            }
                        }
                        Some(Keycode::Down) => {
                            selected_hero = (selected_hero + 1) % hero_classes.len();
                        }
                        Some(Keycode::Return) | Some(Keycode::Space) => {
                            // 创建玩家和地牢
                            let seed = start_time.elapsed().as_millis() as u64;
                            let map = DungeonMap::generate(MAP_WIDTH, MAP_HEIGHT, DungeonType::Cathedral, 1, seed);
                            let spawn = map.spawn_point;
                            player = Some(Player::new(hero_classes[selected_hero], spawn.x, spawn.y));
                            // 等距摄像机: 跟随玩家的等距坐标
                            camera_x = (spawn.x - spawn.y) * (TILE_WIDTH / 2);
                            camera_y = (spawn.x + spawn.y) * (TILE_HEIGHT / 2);
                            // 重置拖动偏移
                            camera_offset_x = 0;
                            camera_offset_y = 0;
                            dungeon = Some(map);
                            println!(">> 选择了 {} 职业", hero_classes[selected_hero].name());
                            println!(">> 生成地牢中...");
                            println!(">> 进入游戏! 使用鼠标右键拖动来移动地图");
                            state = GameState::InGame;
                        }
                        _ => {}
                    },
                    GameState::InGame => {
                        // 玩家移动 (等距方向映射)
                        if let (Some(ref mut p), Some(ref d)) = (&mut player, &dungeon) {
                            // 等距移动：箭头键对应屏幕方向
                            // 上 = 向左上 (-1, -1)，下 = 向右下 (+1, +1)
                            // 左 = 向左下 (-1, +1)，右 = 向右上 (+1, -1)
                            let (dx, dy) = match keycode {
                                Some(Keycode::Up) | Some(Keycode::W) => (-1, -1),    // 屏幕上 = 地图左上
                                Some(Keycode::Down) | Some(Keycode::S) => (1, 1),    // 屏幕下 = 地图右下
                                Some(Keycode::Left) | Some(Keycode::A) => (-1, 1),   // 屏幕左 = 地图左下
                                Some(Keycode::Right) | Some(Keycode::D) => (1, -1),  // 屏幕右 = 地图右上
                                _ => (0, 0),
                            };

                            let new_x = p.x + dx;
                            let new_y = p.y + dy;

                            // 检查碰撞
                            if new_x >= 0 && new_x < d.width as i32
                               && new_y >= 0 && new_y < d.height as i32
                               && d.tiles[new_y as usize][new_x as usize].is_walkable() {
                                p.x = new_x;
                                p.y = new_y;
                                // 更新等距摄像机
                                camera_x = (p.x - p.y) * (TILE_WIDTH / 2);
                                camera_y = (p.x + p.y) * (TILE_HEIGHT / 2);
                            }
                        }
                    }
                },

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    if state == GameState::Title {
                        state = GameState::MainMenu;
                    } else if state == GameState::MainMenu {
                        // 检查点击了哪个菜单项
                        for (i, item) in menu_items.iter().enumerate() {
                            let text_width =
                                text_renderer.get_text_width(item.text, GameFont::Font42, 2);
                            let left = 320 - text_width / 2;
                            let right = 320 + text_width / 2;
                            let top = item.y;
                            let bottom = item.y + 42;

                            if x >= left && x <= right && y >= top && y <= bottom {
                                selected_item = i;
                                handle_menu_select(selected_item, &mut state);
                                break;
                            }
                        }
                    }
                }

                // 右键拖动地图
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    if state == GameState::InGame {
                        dragging = true;
                        println!("开始拖动");
                    }
                }

                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    if state == GameState::InGame {
                        dragging = false;
                        println!("结束拖动, offset: ({}, {})", camera_offset_x, camera_offset_y);
                    }
                }

                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    if state == GameState::MainMenu {
                        // 高亮鼠标悬停的菜单项
                        for (i, item) in menu_items.iter().enumerate() {
                            let text_width =
                                text_renderer.get_text_width(item.text, GameFont::Font42, 2);
                            let left = 320 - text_width / 2;
                            let right = 320 + text_width / 2;
                            let top = item.y;
                            let bottom = item.y + 42;

                            if x >= left && x <= right && y >= top && y <= bottom {
                                selected_item = i;
                                break;
                            }
                        }
                    } else if state == GameState::InGame && dragging {
                        // 拖动地图
                        camera_offset_x += xrel;
                        camera_offset_y += yrel;
                        println!("拖动中: xrel={}, yrel={}, offset=({}, {})", xrel, yrel, camera_offset_x, camera_offset_y);
                    }
                }

                _ => {}
            }
        }

        // 清除文本缓冲区
        text_buffer.fill(0);

        // 渲染
        match state {
            GameState::Title => {
                // 绘制标题背景
                canvas.copy(&title_texture, None, None)?;

                // 绘制Logo (y=182与C++一致, 550x216像素)
                let logo_x = (SCREEN_WIDTH - 550) / 2;
                canvas.copy(
                    &logo_textures[logo_frame],
                    None,
                    Some(Rect::new(logo_x as i32, 182, 550, 216)),
                )?;

                // 渲染 "Press any key" 文本
                text_renderer.render_text_centered(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "Press any key to continue",
                    320,
                    410,
                    GameFont::Font24,
                    Some(TextColor::UiGold),
                    2,
                );

                // 更新文本纹理并绘制
                let mut text_texture = texture_creator.create_texture_streaming(
                    PixelFormatEnum::ABGR8888,
                    SCREEN_WIDTH,
                    SCREEN_HEIGHT,
                )?;
                text_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                text_texture.update(None, &text_buffer, SCREEN_WIDTH as usize * 4)?;
                canvas.copy(&text_texture, None, None)?;
            }

            GameState::MainMenu => {
                // 绘制菜单背景
                canvas.copy(&menu_texture, None, None)?;

                // 绘制Logo (主菜单Logo在y=0位置, 550x216像素)
                let logo_x = (SCREEN_WIDTH - 550) / 2;
                canvas.copy(
                    &logo_textures[logo_frame],
                    None,
                    Some(Rect::new(logo_x as i32, 0, 550, 216)),
                )?;

                // 清空文本缓冲区
                text_buffer.fill(0);

                // 渲染菜单文本
                for (i, item) in menu_items.iter().enumerate() {
                    let color = if i == selected_item {
                        TextColor::UiGold
                    } else {
                        TextColor::UiSilver
                    };

                    text_renderer.render_text_centered(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        item.text,
                        320,
                        item.y,
                        GameFont::Font42,
                        Some(color),
                        2,
                    );
                }

                // 调试: 每秒打印一次 text_buffer 中非零像素数量和采样颜色值
                let non_zero_pixels = text_buffer.chunks(4).filter(|c| c[3] > 0).count();
                static mut LAST_DEBUG_TIME: u32 = 0;
                unsafe {
                    if frame_time > LAST_DEBUG_TIME + 60 {
                        // 找到第一个非零像素并打印其RGBA值
                        if let Some(pixel) = text_buffer.chunks(4).find(|c| c[3] > 0) {
                            println!("[MainMenu] frame={} 非零像素: {} 采样颜色RGBA: ({}, {}, {}, {})",
                                frame_time, non_zero_pixels, pixel[0], pixel[1], pixel[2], pixel[3]);
                        }
                        LAST_DEBUG_TIME = frame_time;
                    }
                }

                // 绘制选择指示器
                let selected_y = menu_items[selected_item].y;
                let text_width = text_renderer.get_text_width(
                    menu_items[selected_item].text,
                    GameFont::Font42,
                    2,
                );
                let left_x = 320 - text_width / 2 - 50;
                let right_x = 320 + text_width / 2 + 8;

                // 更新文本纹理
                let mut text_texture = texture_creator.create_texture_streaming(
                    PixelFormatEnum::ABGR8888,
                    SCREEN_WIDTH,
                    SCREEN_HEIGHT,
                )?;
                text_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                text_texture.update(None, &text_buffer, SCREEN_WIDTH as usize * 4)?;
                canvas.copy(&text_texture, None, None)?;

                // 绘制焦点指示器
                canvas.copy(
                    &focus_textures[focus_frame],
                    None,
                    Some(Rect::new(left_x, selected_y, 42, 42)),
                )?;
                canvas.copy(
                    &focus_textures[focus_frame],
                    None,
                    Some(Rect::new(right_x, selected_y, 42, 42)),
                )?;

                // 版本信息
                text_buffer.fill(0);
                text_renderer.render_text_centered(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "DevilutionX-RS v0.1.0",
                    320,
                    455,
                    GameFont::Font24,
                    Some(TextColor::White),
                    1,
                );
                text_texture.update(None, &text_buffer, SCREEN_WIDTH as usize * 4)?;
                canvas.copy(&text_texture, None, None)?;
            }

            GameState::SelectHero => {
                // 英雄选择界面 - 使用专门的背景
                canvas.copy(&selhero_texture, None, None)?;

                // 注意：selhero背景可能本身包含装饰，Logo会挡住标题
                // C++中Logo是动画元素，会覆盖在背景上但标题在Logo之上
                // 我们先跳过Logo，让标题可见

                // 英雄肖像 - C++: x=30, y=211, 宽180, 高76 (左侧)
                if selected_hero < hero_portrait_textures.len() {
                    canvas.copy(
                        &hero_portrait_textures[selected_hero],
                        None,
                        Some(Rect::new(30, 211, assets.hero_portrait_width, 76)),
                    )?;
                }

                text_buffer.fill(0);

                // 标题 - C++: x=24起始, y=161, 宽590, 居中
                text_renderer.render_text_centered(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "New Single Player Hero",
                    320,  // 屏幕水平居中
                    161,  // C++原版位置
                    GameFont::Font30,
                    Some(TextColor::UiSilver),
                    3,
                );

                // 左侧属性显示 - C++风格布局
                // Level: x=39 (标签右对齐), x=159 (值居中), y=323
                text_renderer.render_text(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "Level:",
                    100,  // 右对齐到 ~149
                    323,
                    GameFont::Font24,
                    Some(TextColor::UiSilver),
                    1,
                );
                text_renderer.render_text(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "1",
                    170,
                    323,
                    GameFont::Font24,
                    Some(TextColor::UiSilver),
                    1,
                );

                // 属性从 y=358 开始，每行21像素
                let (str_val, mag_val, dex_val, vit_val) = match hero_classes[selected_hero] {
                    HeroClass::Warrior => (30, 10, 20, 25),
                    HeroClass::Rogue => (20, 15, 30, 20),
                    HeroClass::Sorcerer => (15, 35, 15, 20),
                };

                let stat_labels = ["Strength:", "Magic:", "Dexterity:", "Vitality:"];
                let stat_values = [str_val, mag_val, dex_val, vit_val];
                let mut stat_y = 358;

                for (label, value) in stat_labels.iter().zip(stat_values.iter()) {
                    text_renderer.render_text(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        label,
                        50,
                        stat_y,
                        GameFont::Font24,
                        Some(TextColor::UiSilver),
                        1,
                    );
                    text_renderer.render_text(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        &value.to_string(),
                        170,
                        stat_y,
                        GameFont::Font24,
                        Some(TextColor::UiSilver),
                        1,
                    );
                    stat_y += 21;
                }

                // 右侧职业选择列表 - C++: x=264, y=246附近, 宽320
                // 显示 "Select Class" 标题
                text_renderer.render_text_centered(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "Select Class",
                    424,  // 264 + 160 = 424 (列表中心)
                    211,
                    GameFont::Font30,
                    Some(TextColor::UiSilver),
                    3,
                );

                // 职业列表 - 从 y=256 开始
                let class_names = ["Warrior", "Rogue", "Sorcerer"];
                let list_y_start = 280;
                let item_height = 33;

                for (i, name) in class_names.iter().enumerate() {
                    let item_y = list_y_start + (i as i32 * item_height);
                    let color = if i == selected_hero {
                        TextColor::UiGold
                    } else {
                        TextColor::UiSilver
                    };

                    text_renderer.render_text_centered(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        name,
                        424,  // 列表中心
                        item_y,
                        GameFont::Font24,
                        Some(color),
                        2,
                    );

                    // 选中项的指示器
                    if i == selected_hero {
                        let text_width = text_renderer.get_text_width(name, GameFont::Font24, 2);
                        canvas.copy(
                            &focus_textures[focus_frame],
                            None,
                            Some(Rect::new(424 - text_width / 2 - 30, item_y - 5, 42, 42)),
                        )?;
                        canvas.copy(
                            &focus_textures[focus_frame],
                            None,
                            Some(Rect::new(424 + text_width / 2 - 12, item_y - 5, 42, 42)),
                        )?;
                    }
                }

                // OK 和 Cancel 按钮 - C++: y=429
                text_renderer.render_text_centered(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "OK",
                    349,  // 279 + 70
                    429,
                    GameFont::Font30,
                    Some(TextColor::UiGold),
                    2,
                );
                text_renderer.render_text_centered(
                    &mut text_buffer,
                    SCREEN_WIDTH as usize,
                    SCREEN_HEIGHT as usize,
                    "Cancel",
                    501,  // 429 + 72
                    429,
                    GameFont::Font30,
                    Some(TextColor::UiGold),
                    2,
                );

                let mut text_texture = texture_creator.create_texture_streaming(
                    PixelFormatEnum::ABGR8888,
                    SCREEN_WIDTH,
                    SCREEN_HEIGHT,
                )?;
                text_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                text_texture.update(None, &text_buffer, SCREEN_WIDTH as usize * 4)?;
                canvas.copy(&text_texture, None, None)?;
            }

            GameState::InGame => {
                // 游戏界面 - 渲染地牢
                canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
                canvas.clear();

                if let (Some(ref p), Some(ref d)) = (&player, &dungeon) {
                    // 渲染地图（使用真实瓦片和 TIL 数据），加上拖动偏移
                    render_dungeon(&mut canvas, d, p, camera_offset_x, camera_offset_y, &mega_tile_textures, &level_data.til)?;

                    // 清空文本缓冲区
                    text_buffer.fill(0);

                    // 显示玩家信息 (HUD)
                    let hp_text = format!("HP: {}/{}", p.health, p.max_health);
                    text_renderer.render_text(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        &hp_text,
                        10,
                        10,
                        GameFont::Font24,
                        Some(TextColor::UiGold),
                        1,
                    );

                    let mp_text = format!("MP: {}/{}", p.mana, p.max_mana);
                    text_renderer.render_text(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        &mp_text,
                        10,
                        35,
                        GameFont::Font24,
                        Some(TextColor::UiSilver),
                        1,
                    );

                    let class_text = format!("Lv.{} {}", p.level, p.class.name());
                    text_renderer.render_text(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        &class_text,
                        10,
                        60,
                        GameFont::Font24,
                        Some(TextColor::White),
                        1,
                    );

                    let gold_text = format!("Gold: {}", p.gold);
                    text_renderer.render_text(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        &gold_text,
                        10,
                        SCREEN_HEIGHT as i32 - 30,
                        GameFont::Font24,
                        Some(TextColor::UiGold),
                        1,
                    );

                    // 操作提示
                    text_renderer.render_text_centered(
                        &mut text_buffer,
                        SCREEN_WIDTH as usize,
                        SCREEN_HEIGHT as usize,
                        "WASD or Arrow Keys to move, ESC for menu",
                        320,
                        SCREEN_HEIGHT as i32 - 30,
                        GameFont::Font24,
                        Some(TextColor::UiSilver),
                        1,
                    );

                    let mut text_texture = texture_creator.create_texture_streaming(
                        PixelFormatEnum::ABGR8888,
                        SCREEN_WIDTH,
                        SCREEN_HEIGHT,
                    )?;
                    text_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                    text_texture.update(None, &text_buffer, SCREEN_WIDTH as usize * 4)?;
                    canvas.copy(&text_texture, None, None)?;
                }
            }
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    println!("游戏结束，感谢游玩!");
    Ok(())
}

fn handle_menu_select(item: usize, state: &mut GameState) {
    match item {
        0 => {
            // Single Player
            println!(">> Single Player");
            *state = GameState::SelectHero;
        }
        1 => {
            // Multi Player
            println!(">> Multi Player (Coming Soon)");
        }
        2 => {
            // Replay Intro
            println!(">> Replay Intro");
            *state = GameState::Title;
        }
        3 => {
            // Show Credits
            println!(">> Show Credits (Coming Soon)");
        }
        4 => {
            // Exit
            println!(">> Exit");
            std::process::exit(0);
        }
        _ => {}
    }
}

/// 计算光照强度 (基于玩家位置)
fn calculate_light(tile_x: i32, tile_y: i32, player_x: i32, player_y: i32) -> u8 {
    let dx = (tile_x - player_x).abs();
    let dy = (tile_y - player_y).abs();
    let distance = ((dx * dx + dy * dy) as f32).sqrt() as i32;

    if distance <= TORCH_LIGHT_RADIUS {
        // 在火把范围内，光照逐渐减弱
        let light = (distance * MAX_LIGHT_LEVEL as i32 / TORCH_LIGHT_RADIUS) as u8;
        light.min(MAX_LIGHT_LEVEL)
    } else {
        // 超出范围，使用环境光
        MAX_LIGHT_LEVEL - 3  // 环境光，不完全黑暗
    }
}

/// 渲染地牢地图 - 使用正确的等轴测坐标
///
/// C++ 坐标系统:
/// - screen_x = (tile_y - tile_x) * 32
/// - screen_y = (tile_y + tile_x) * 16
/// - 这个坐标是 64x32 菱形地板的**左顶点**位置
///
/// mega-tile 渲染:
/// - 每个 mega-tile 有多个 32x32 的 blocks
/// - mt[0] 在 (x, y)，mt[1] 在 (x+32, y)
/// - mt[2] 在 (x, y-32)，mt[3] 在 (x+32, y-32)
/// - 以此类推向上
fn render_dungeon(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    dungeon: &DungeonMap,
    player: &Player,
    camera_offset_x: i32,  // 相机额外偏移（用于拖动）
    camera_offset_y: i32,
    mega_tile_textures: &[sdl2::render::Texture],
    til_data: &TilData,
) -> Result<(), Box<dyn std::error::Error>> {
    let screen_width = SCREEN_WIDTH as i32;
    let screen_height = SCREEN_HEIGHT as i32;

    // 相机跟随玩家，玩家在屏幕中央
    // 玩家的 dPiece 坐标（每个地图 tile 对应 2x2 dPiece）
    let player_px = player.x * 2;
    let player_py = player.y * 2;

    // 玩家位置的屏幕坐标（如果原点在(0,0)的话）
    let player_screen_x = (player_py - player_px) * 32;
    let player_screen_y = (player_py + player_px) * 16;

    // origin: 让玩家位置对应到屏幕中央，加上拖动偏移
    let origin_x = screen_width / 2 - player_screen_x + camera_offset_x;
    let origin_y = screen_height / 2 - player_screen_y + camera_offset_y;

    // 将 dPiece 坐标转为屏幕坐标
    // 返回的是 64x32 菱形的**左顶点**位置
    let dpiece_to_screen = |px: i32, py: i32| -> (i32, i32) {
        let screen_x = origin_x + (py - px) * 32;
        let screen_y = origin_y + (py + px) * 16;
        (screen_x, screen_y)
    };

    // 绘制地图
    // TIL 文件将 dungeon[][] 的值映射到 2x2 的 dPiece
    // dungeon[x][y] -> 4个 mega-tile IDs (micro1, micro2, micro3, micro4)
    //   dPiece[x*2][y*2]     = micro1 (左上)
    //   dPiece[x*2+1][y*2]   = micro2 (右上)
    //   dPiece[x*2][y*2+1]   = micro3 (左下)
    //   dPiece[x*2+1][y*2+1] = micro4 (右下)

    let map_width = dungeon.width as i32;
    let map_height = dungeon.height as i32;

    // 调试：打印 TIL 信息
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        println!("=== 渲染调试 ===");
        println!("地图大小: {}x{}", map_width, map_height);
        println!("TIL 条目数: {}", til_data.len());
        println!("纹理数量: {}", mega_tile_textures.len());
        // 打印前几个 TIL 条目
        for i in 0..5.min(til_data.len()) {
            if let Some(entry) = til_data.get(i) {
                println!("TIL[{}]: micro1={}, micro2={}, micro3={}, micro4={}",
                    i, entry.micro1, entry.micro2, entry.micro3, entry.micro4);
            }
        }
        println!("玩家位置: ({}, {})", player.x, player.y);
        println!("玩家dPiece: ({}, {})", player_px, player_py);
        println!("origin: ({}, {})", origin_x, origin_y);
    });

    let mut rendered_count = 0;

    // 先绘制所有地板（从后往前，Y轴降序）
    for ty in (0..map_height).rev() {
        for tx in (0..map_width).rev() {
            let tile = match dungeon.get_tile(tx, ty) {
                Some(t) => t,
                None => continue,
            };

            // Cathedral TIL 索引映射 (基于 C++ drlg_l1.cpp 的 Tile 枚举)
            // 注意: C++ 中 dungeon[][] 存储 tile ID (1-based)，TIL 索引 = tile ID - 1
            // 但我们这里直接使用 0-based 索引
            let til_idx = match tile {
                TileType::Floor => {
                    // Floor = 13 in C++, 所以 TIL 索引 = 12
                    12
                },
                TileType::Wall => {
                    // 检查墙壁方向 - 看周围哪些是地板
                    let has_floor_north = ty > 0 && dungeon.get_tile(tx, ty - 1).map(|t| t == TileType::Floor).unwrap_or(false);
                    let has_floor_south = dungeon.get_tile(tx, ty + 1).map(|t| t == TileType::Floor).unwrap_or(false);
                    let has_floor_west = tx > 0 && dungeon.get_tile(tx - 1, ty).map(|t| t == TileType::Floor).unwrap_or(false);
                    let has_floor_east = dungeon.get_tile(tx + 1, ty).map(|t| t == TileType::Floor).unwrap_or(false);

                    // Cathedral 墙壁 TIL 索引 (C++ tile ID - 1)
                    // VWall=1 -> 0, HWall=2 -> 1, Corner=3 -> 2, etc.
                    if has_floor_south && has_floor_east {
                        2   // Corner (西北角) = 3-1
                    } else if has_floor_south && has_floor_west {
                        15  // VCorner = 16-1
                    } else if has_floor_north && has_floor_east {
                        16  // HCorner = 17-1
                    } else if has_floor_north && has_floor_west {
                        2   // Corner
                    } else if has_floor_south {
                        1   // HWall (水平墙，面向南) = 2-1
                    } else if has_floor_north {
                        1   // HWall
                    } else if has_floor_east {
                        0   // VWall (垂直墙，面向东) = 1-1
                    } else if has_floor_west {
                        0   // VWall
                    } else {
                        2   // Corner (默认)
                    }
                },
                TileType::Door => 24,       // VDoor = 25-1
                TileType::Pillar => 14,     // Pillar = 15-1
                TileType::Stairs | TileType::StairsDown => 63,  // EntranceStairs = 64-1
                TileType::StairsUp => 63,
                _ => 12,  // 默认用地板
            };

            // 确保索引有效
            let til_idx = til_idx.min(til_data.len().saturating_sub(1));

            // 从 TIL 获取 4 个 mega-tile IDs
            if let Some(til_entry) = til_data.get(til_idx) {
                // 这个 dungeon tile 对应 2x2 的 dPiece
                let base_px = tx * 2;
                let base_py = ty * 2;

                // 渲染 4 个 dPiece（2x2 组合）
                let pieces = [
                    (til_entry.micro1, base_px, base_py),         // 左上
                    (til_entry.micro2, base_px + 1, base_py),     // 右上
                    (til_entry.micro3, base_px, base_py + 1),     // 左下
                    (til_entry.micro4, base_px + 1, base_py + 1), // 右下
                ];

                for (piece_id, px, py) in pieces {
                    if piece_id == 0 {
                        continue;
                    }
                    let idx = (piece_id - 1) as usize;
                    if idx >= mega_tile_textures.len() {
                        continue;
                    }

                    // 获取这个 dPiece 的屏幕位置（左顶点）
                    let (left_x, left_y) = dpiece_to_screen(px, py);

                    // 检查是否在屏幕内
                    if left_x < -200 || left_x > screen_width + 100
                        || left_y < -200 || left_y > screen_height + 100 {
                        continue;
                    }

                    // 渲染 mega-tile 纹理
                    let dst = Rect::new(left_x, left_y - 128, 64, 160);
                    canvas.copy(&mega_tile_textures[idx], None, dst)?;
                    rendered_count += 1;
                }
            }
        }
    }

    // 每隔一段时间打印渲染计数
    static FRAME_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let frame = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if frame % 300 == 0 {
        println!("帧 {}: 渲染了 {} 个瓦片", frame, rendered_count);
    }

    // 可选：绘制网格线用于调试
    let show_grid = false;
    if show_grid {
        canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 0, 100));
        for ty in 0..map_height * 2 {
            for tx in 0..map_width * 2 {
                let (left_x, left_y) = dpiece_to_screen(tx, ty);
                // 菱形的 4 个顶点
                let top = sdl2::rect::Point::new(left_x + 32, left_y);
                let right = sdl2::rect::Point::new(left_x + 64, left_y + 16);
                let bottom = sdl2::rect::Point::new(left_x + 32, left_y + 32);
                let left = sdl2::rect::Point::new(left_x, left_y + 16);

                canvas.draw_line(left, top).ok();
                canvas.draw_line(top, right).ok();
                canvas.draw_line(right, bottom).ok();
                canvas.draw_line(bottom, left).ok();
            }
        }
    }

    // 绘制玩家位置标记
    let (player_x, player_y) = dpiece_to_screen(player.x * 2, player.y * 2);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
    canvas.fill_rect(Rect::new(player_x + 28, player_y + 12, 8, 8))?;

    Ok(())
}

/// 旧的复杂版本（备份）
fn _get_mega_tile_index_complex(tile: &TileType, tile_x: i32, tile_y: i32) -> usize {
    match tile {
        TileType::Floor => {
            // 地板变化 - 使用不同的地板瓦片
            let variation = ((tile_x * 7 + tile_y * 13) % 8) as usize;
            match variation {
                0 => 2,
                1 => 3,
                2 => 4,
                3 => 6,
                4 => 7,
                5 => 11,
                6 => 12,
                _ => 2,
            }
        },
        TileType::Wall => {
            // 墙壁 - 根据位置选择不同的墙壁瓦片
            let hash = ((tile_x * 3 + tile_y * 5) % 4) as usize;
            match hash {
                0 => 1,
                1 => 8,
                2 => 9,
                _ => 14,
            }
        },
        TileType::Door => 10,
        TileType::Stairs | TileType::StairsDown => 15,
        TileType::StairsUp => 16,
        TileType::Pillar => 5,
        TileType::Altar => 20,
        TileType::Chest => 25,
        TileType::Barrel => 30,
    }
}

/// 绘制玩家
fn draw_player(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    player: &Player,
    screen_center_x: i32,
    screen_center_y: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let player_screen_x = screen_center_x;
    let player_screen_y = screen_center_y - 32; // 稍微向上偏移

    // 玩家颜色根据职业
    let player_color = match player.class {
        HeroClass::Warrior => sdl2::pixels::Color::RGB(255, 80, 80),   // 红色
        HeroClass::Rogue => sdl2::pixels::Color::RGB(80, 255, 80),     // 绿色
        HeroClass::Sorcerer => sdl2::pixels::Color::RGB(80, 80, 255),  // 蓝色
    };

    // 绘制玩家为菱形
    canvas.set_draw_color(player_color);
    let ps = 10; // 玩家大小

    // 填充菱形
    for i in 0..ps {
        let w = i * 2;
        canvas.draw_line(
            sdl2::rect::Point::new(player_screen_x - w, player_screen_y - ps + i),
            sdl2::rect::Point::new(player_screen_x + w, player_screen_y - ps + i),
        )?;
    }
    for i in 0..ps {
        let w = (ps - i - 1) * 2;
        canvas.draw_line(
            sdl2::rect::Point::new(player_screen_x - w, player_screen_y + i),
            sdl2::rect::Point::new(player_screen_x + w, player_screen_y + i),
        )?;
    }

    // 玩家边框
    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
    let player_points = [
        sdl2::rect::Point::new(player_screen_x, player_screen_y - ps),
        sdl2::rect::Point::new(player_screen_x + ps * 2, player_screen_y),
        sdl2::rect::Point::new(player_screen_x, player_screen_y + ps),
        sdl2::rect::Point::new(player_screen_x - ps * 2, player_screen_y),
        sdl2::rect::Point::new(player_screen_x, player_screen_y - ps),
    ];
    canvas.draw_lines(&player_points[..])?;

    Ok(())
}
