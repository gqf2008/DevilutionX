//! DevilutionX-RS 真实资源游戏启动器
//! 与 C++ 版本保持一致的界面

use devilutionx_rs::engine::mpq::MpqArchive;
use devilutionx_rs::engine::pcx::PcxImage;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::time::Duration;

/// 从 PCX 创建 SDL2 纹理
fn create_texture_from_pcx<'a>(
    pcx: &PcxImage,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>, String> {
    let rgba = pcx.to_rgba();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, pcx.width, pcx.height)
        .map_err(|e| e.to_string())?;

    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..pcx.height as usize {
            for x in 0..pcx.width as usize {
                let src_idx = (y * pcx.width as usize + x) * 4;
                let dst_idx = y * pitch + x * 4;
                if src_idx + 3 < rgba.len() && dst_idx + 3 < buffer.len() {
                    buffer[dst_idx] = rgba[src_idx];
                    buffer[dst_idx + 1] = rgba[src_idx + 1];
                    buffer[dst_idx + 2] = rgba[src_idx + 2];
                    buffer[dst_idx + 3] = 255;
                }
            }
        }
    })?;

    Ok(texture)
}

/// 位图字体
struct BitmapFont<'a> {
    texture: Texture<'a>,
    char_width: u32,
    char_height: u32,
    chars_per_row: u32,
}

impl<'a> BitmapFont<'a> {
    fn load(
        archive: &mut MpqArchive,
        pcx_path: &str,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let data = archive.read_file(pcx_path)?;
        let pcx = PcxImage::decode(&data).ok_or("Failed to decode font PCX")?;

        // 假设字体是 16 列排列
        let chars_per_row = 16u32;
        let char_width = pcx.width / chars_per_row;
        let char_height = pcx.height / 16; // 16 行

        let texture = create_texture_from_pcx(&pcx, texture_creator)?;

        Ok(Self {
            texture,
            char_width,
            char_height,
            chars_per_row,
        })
    }

    fn draw_text(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        text: &str,
        x: i32,
        y: i32,
    ) -> Result<(), String> {
        let mut current_x = x;

        for ch in text.chars() {
            let ascii = ch as u32;
            if ascii >= 32 && ascii < 128 {
                let char_index = ascii - 32;
                let src_x = (char_index % self.chars_per_row) * self.char_width;
                let src_y = (char_index / self.chars_per_row) * self.char_height;

                let src_rect = Rect::new(
                    src_x as i32,
                    src_y as i32,
                    self.char_width,
                    self.char_height,
                );
                let dst_rect = Rect::new(
                    current_x,
                    y,
                    self.char_width,
                    self.char_height,
                );

                canvas.copy(&self.texture, src_rect, dst_rect)?;
            }
            current_x += self.char_width as i32;
        }

        Ok(())
    }

    fn text_width(&self, text: &str) -> u32 {
        text.len() as u32 * self.char_width
    }

    fn draw_text_centered(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        text: &str,
        center_x: i32,
        y: i32,
    ) -> Result<(), String> {
        let width = self.text_width(text) as i32;
        self.draw_text(canvas, text, center_x - width / 2, y)
    }
}

/// 菜单选项
const MENU_ITEMS: &[&str] = &[
    "Single Player",
    "Multi Player",
    "Replay Intro",
    "Show Credits",
    "Exit Diablo",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DevilutionX-RS");
    println!("==============");

    // 打开 spawn.mpq
    let mpq_paths = [
        "spawn.mpq",
        "../spawn.mpq",
        "d:/Users/gxh/Documents/GitHub/DevilutionX/spawn.mpq",
    ];

    let mut archive = None;
    for path in &mpq_paths {
        if let Ok(a) = MpqArchive::open(path) {
            println!("Loaded: {}", path);
            archive = Some(a);
            break;
        }
    }
    let mut archive = archive.ok_or("spawn.mpq not found!")?;

    // 加载资源
    println!("Loading assets...");

    // 标题画面
    let title_data = archive.read_file("ui_art\\title.pcx")?;
    let title_pcx = PcxImage::decode(&title_data).ok_or("Failed to decode title.pcx")?;

    // 主菜单背景
    let menu_data = archive.read_file("ui_art\\menu.pcx")?;
    let menu_pcx = PcxImage::decode(&menu_data).ok_or("Failed to decode menu.pcx")?;

    // Logo 动画
    let logo_data = archive.read_file("ui_art\\logo.pcx")?;
    let logo_pcx = PcxImage::decode(&logo_data).ok_or("Failed to decode logo.pcx")?;

    // 焦点/选择指示器
    let focus_data = archive.read_file("ui_art\\focus42.pcx")?;
    let focus_pcx = PcxImage::decode(&focus_data).ok_or("Failed to decode focus42.pcx")?;

    println!("  title.pcx: {}x{}", title_pcx.width, title_pcx.height);
    println!("  menu.pcx: {}x{}", menu_pcx.width, menu_pcx.height);
    println!("  logo.pcx: {}x{}", logo_pcx.width, logo_pcx.height);
    println!("  focus42.pcx: {}x{}", focus_pcx.width, focus_pcx.height);

    // 初始化 SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("DevilutionX-RS", 640, 480)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    // 创建纹理
    let title_texture = create_texture_from_pcx(&title_pcx, &texture_creator)?;
    let menu_texture = create_texture_from_pcx(&menu_pcx, &texture_creator)?;
    let focus_texture = create_texture_from_pcx(&focus_pcx, &texture_creator)?;

    // Logo 动画帧 - 每帧 120 高
    let logo_frame_height = 120;
    let logo_frames = logo_pcx.height as usize / logo_frame_height;
    let logo_rgba = logo_pcx.to_rgba();

    let mut logo_textures = Vec::new();
    for frame in 0..logo_frames {
        let mut frame_texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::ABGR8888, logo_pcx.width, logo_frame_height as u32)
            .map_err(|e| e.to_string())?;

        let frame_start_y = frame * logo_frame_height;
        frame_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..logo_frame_height {
                for x in 0..logo_pcx.width as usize {
                    let src_idx = ((frame_start_y + y) * logo_pcx.width as usize + x) * 4;
                    let dst_idx = y * pitch + x * 4;
                    if src_idx + 3 < logo_rgba.len() && dst_idx + 3 < buffer.len() {
                        buffer[dst_idx] = logo_rgba[src_idx];
                        buffer[dst_idx + 1] = logo_rgba[src_idx + 1];
                        buffer[dst_idx + 2] = logo_rgba[src_idx + 2];
                        buffer[dst_idx + 3] = 255;
                    }
                }
            }
        })?;
        logo_textures.push(frame_texture);
    }
    println!("  Logo: {} frames", logo_frames);

    // 加载字体
    let font42 = BitmapFont::load(&mut archive, "ui_art\\font42g.pcx", &texture_creator)?;
    let font30 = BitmapFont::load(&mut archive, "ui_art\\font30g.pcx", &texture_creator)?;
    let font24 = BitmapFont::load(&mut archive, "ui_art\\font24g.pcx", &texture_creator)?;
    println!("  Fonts loaded");

    // 焦点动画帧
    let focus_frame_width = focus_pcx.width / 8; // 8 帧

    // 游戏状态
    #[derive(PartialEq, Clone, Copy)]
    enum GameState {
        TitleScreen,
        MainMenu,
    }

    let mut state = GameState::TitleScreen;
    let mut selected = 0usize;
    let mut logo_frame = 0usize;
    let mut focus_frame = 0usize;
    let mut anim_timer = 0.0f32;

    let mut event_pump = sdl_context.event_pump()?;
    let start_time = std::time::Instant::now();

    println!("\nGame ready! Press any key...");

    'running: loop {
        let elapsed = start_time.elapsed().as_secs_f32();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(key), .. } => match state {
                    GameState::TitleScreen => {
                        state = GameState::MainMenu;
                    }
                    GameState::MainMenu => match key {
                        Keycode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            } else {
                                selected = MENU_ITEMS.len() - 1;
                            }
                        }
                        Keycode::Down => {
                            selected = (selected + 1) % MENU_ITEMS.len();
                        }
                        Keycode::Return | Keycode::Space => {
                            match selected {
                                0 => println!(">> Single Player"),
                                1 => println!(">> Multi Player (not implemented)"),
                                2 => {
                                    state = GameState::TitleScreen;
                                }
                                3 => println!(">> Credits: DevilutionX-RS"),
                                4 => break 'running,
                                _ => {}
                            }
                        }
                        Keycode::Escape => break 'running,
                        _ => {}
                    },
                },
                Event::MouseButtonDown { x, y, mouse_btn: sdl2::mouse::MouseButton::Left, .. } => {
                    if state == GameState::TitleScreen {
                        state = GameState::MainMenu;
                    } else {
                        // 检测菜单点击
                        let menu_y_start = 192;
                        let item_height = 45;
                        for (i, _) in MENU_ITEMS.iter().enumerate() {
                            let item_y = menu_y_start + i as i32 * item_height;
                            if y >= item_y && y < item_y + item_height && x >= 200 && x <= 440 {
                                selected = i;
                                // 执行选择
                                match selected {
                                    4 => break 'running,
                                    2 => state = GameState::TitleScreen,
                                    _ => println!(">> {}", MENU_ITEMS[selected]),
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // 动画更新
        anim_timer += 1.0 / 60.0;
        if anim_timer >= 0.1 {
            anim_timer = 0.0;
            if !logo_textures.is_empty() {
                logo_frame = (logo_frame + 1) % logo_textures.len();
            }
            focus_frame = (focus_frame + 1) % 8;
        }

        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        match state {
            GameState::TitleScreen => {
                // 标题画面
                canvas.copy(&title_texture, None, None)?;

                // 闪烁文字 "Press any key to continue"
                if (elapsed * 2.0) as u32 % 2 == 0 {
                    font24.draw_text_centered(&mut canvas, "Press any key to continue", 320, 410)?;
                }
            }
            GameState::MainMenu => {
                // 主菜单背景
                canvas.copy(&menu_texture, None, None)?;

                // Logo 动画 (居中显示在顶部)
                if !logo_textures.is_empty() {
                    let logo_x = (640 - logo_pcx.width as i32) / 2;
                    canvas.copy(
                        &logo_textures[logo_frame],
                        None,
                        Rect::new(logo_x, 0, logo_pcx.width, logo_frame_height as u32),
                    )?;
                }

                // 菜单项
                let menu_y_start = 192;
                let item_height = 45;

                for (i, &item) in MENU_ITEMS.iter().enumerate() {
                    let y = menu_y_start + i as i32 * item_height;

                    // 绘制文字
                    font42.draw_text_centered(&mut canvas, item, 320, y)?;

                    // 选中指示器
                    if i == selected {
                        let text_width = font42.text_width(item) as i32;
                        let focus_x = 320 - text_width / 2 - focus_frame_width as i32 - 10;
                        let focus_src = Rect::new(
                            (focus_frame as u32 * focus_frame_width) as i32,
                            0,
                            focus_frame_width,
                            focus_pcx.height,
                        );
                        canvas.copy(
                            &focus_texture,
                            focus_src,
                            Rect::new(focus_x, y - 5, focus_frame_width, focus_pcx.height),
                        )?;

                        // 右侧指示器
                        let focus_x_right = 320 + text_width / 2 + 10;
                        canvas.copy(
                            &focus_texture,
                            focus_src,
                            Rect::new(focus_x_right, y - 5, focus_frame_width, focus_pcx.height),
                        )?;
                    }
                }

                // 版本信息
                font24.draw_text_centered(&mut canvas, "DevilutionX-RS v0.1.0", 320, 450)?;
            }
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    println!("Game ended.");
    Ok(())
}
