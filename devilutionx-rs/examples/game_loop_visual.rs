/// 游戏循环可视化演示 - 简化版（无字体渲染）
/// 展示 GameState 的实时运行效果

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};

use devilutionx_rs::game::game_state::{GameState, GameLogicStep};
use devilutionx_rs::game::player_exact::Player;
use rand::rngs::StdRng;
use rand::SeedableRng;

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;

fn main() -> Result<(), String> {
    println!("\n🎮 ============================================");
    println!("   DevilutionX-rs 游戏循环可视化演示");
    println!("   ============================================\n");

    // 创建 Player
    let mut player = Player::new();
    player._p_level = 10;
    player._p_hit_points = 100 << 6;
    player._p_max_hp = 200 << 6;
    player._p_mana = 50 << 6;
    player._p_max_mana = 100 << 6;
    player._p_magic = 40;

    println!("📊 初始状态:");
    println!("   Player: Level {} | HP: {}/{} | Mana: {}/{}",
        player._p_level,
        player._p_hit_points >> 6,
        player._p_max_hp >> 6,
        player._p_mana >> 6,
        player._p_max_mana >> 6
    );

    // 创建游戏状态
    let mut game_state = GameState::new(player, true, 12345);
    let mut rng = StdRng::seed_from_u64(12345);

    // 初始化 SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("DevilutionX-rs - Game Loop Visualization", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl.event_pump()?;
    let mut frame_count = 0u32;
    let mut running = true;
    let mut paused = false;
    let frame_delay = Duration::from_millis(100); // 10 FPS for visibility
    let mut last_frame = Instant::now();

    println!("\n⌨️  Controls:");
    println!("   SPACE: Pause/Resume");
    println!("   ESC: Exit\n");
    println!("⏱️  Game loop started...\n");

    'game_loop: while running {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => running = false,
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    paused = !paused;
                    println!("{}", if paused { "⏸️  Paused" } else { "▶️  Resumed" });
                }
                _ => {}
            }
        }

        // Update game state
        if !paused && last_frame.elapsed() >= frame_delay {
            game_state.update(&mut rng);
            frame_count += 1;

            if frame_count % 10 == 0 {
                println!("Frame {:4} | HP: {:3}/{:3} | Mana: {:3}/{:3} | Step: {:?}",
                    frame_count,
                    game_state.player._p_hit_points >> 6,
                    game_state.player._p_max_hp >> 6,
                    game_state.player._p_mana >> 6,
                    game_state.player._p_max_mana >> 6,
                    game_state.logic_step
                );
            }

            last_frame = Instant::now();
        }

        // Render
        canvas.set_draw_color(Color::RGB(20, 20, 30));
        canvas.clear();

        // Title bar
        canvas.set_draw_color(Color::RGB(255, 200, 100));
        canvas.fill_rect(Rect::new(200, 20, 624, 50))?;

        // Frame counter box
        canvas.set_draw_color(Color::RGB(100, 100, 150));
        canvas.fill_rect(Rect::new(50, 90, 200, 40))?;

        // Current step indicator
        let step_color = match game_state.logic_step {
            GameLogicStep::ProcessPlayers => Color::RGB(100, 200, 100),
            GameLogicStep::ProcessMonsters => Color::RGB(200, 100, 100),
            GameLogicStep::ProcessObjects => Color::RGB(100, 100, 200),
            GameLogicStep::ProcessMissiles => Color::RGB(200, 200, 100),
            GameLogicStep::ProcessItems => Color::RGB(200, 100, 200),
            _ => Color::RGB(100, 100, 100),
        };
        canvas.set_draw_color(step_color);
        canvas.fill_rect(Rect::new(280, 90, 400, 40))?;

        // Player status panel
        let panel_y = 160;
        canvas.set_draw_color(Color::RGB(60, 60, 80));
        canvas.fill_rect(Rect::new(40, panel_y, 600, 350))?;

        // HP bar background
        let hp_y = panel_y + 60;
        canvas.set_draw_color(Color::RGB(40, 40, 50));
        canvas.fill_rect(Rect::new(60, hp_y, 500, 40))?;

        // HP fill
        let hp_current = game_state.player._p_hit_points >> 6;
        let hp_max = game_state.player._p_max_hp >> 6;
        if hp_max > 0 {
            let hp_width = ((hp_current as f32 / hp_max as f32) * 496.0) as u32;
            canvas.set_draw_color(Color::RGB(200, 50, 50));
            canvas.fill_rect(Rect::new(62, hp_y + 2, hp_width, 36))?;
        }

        // Mana bar background
        let mana_y = hp_y + 60;
        canvas.set_draw_color(Color::RGB(40, 40, 50));
        canvas.fill_rect(Rect::new(60, mana_y, 500, 40))?;

        // Mana fill
        let mana_current = game_state.player._p_mana >> 6;
        let mana_max = game_state.player._p_max_mana >> 6;
        if mana_max > 0 {
            let mana_width = ((mana_current as f32 / mana_max as f32) * 496.0) as u32;
            canvas.set_draw_color(Color::RGB(50, 100, 200));
            canvas.fill_rect(Rect::new(62, mana_y + 2, mana_width, 36))?;
        }

        // Info boxes
        let info_y = mana_y + 70;

        // Level box
        canvas.set_draw_color(Color::RGB(80, 80, 100));
        canvas.fill_rect(Rect::new(60, info_y, 150, 35))?;

        // Magic box
        canvas.set_draw_color(Color::RGB(80, 80, 100));
        canvas.fill_rect(Rect::new(230, info_y, 150, 35))?;

        // Regen info panel
        let regen_y = info_y + 60;
        canvas.set_draw_color(Color::RGB(50, 80, 50));
        canvas.fill_rect(Rect::new(60, regen_y, 500, 80))?;

        // Game loop step diagram (right side)
        draw_step_diagram(&mut canvas, 700, 160, &game_state.logic_step)?;

        // Status indicator
        canvas.set_draw_color(if paused {
            Color::RGB(200, 100, 100)
        } else {
            Color::RGB(100, 200, 100)
        });
        canvas.fill_rect(Rect::new(400, SCREEN_HEIGHT as i32 - 80, 224, 40))?;

        canvas.present();
        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    println!("\n🎉 Demo ended! Total frames: {}", frame_count);
    println!("   Final HP: {}/{}",
        game_state.player._p_hit_points >> 6,
        game_state.player._p_max_hp >> 6
    );
    println!("   Final Mana: {}/{}\n",
        game_state.player._p_mana >> 6,
        game_state.player._p_max_mana >> 6
    );
    Ok(())
}

fn draw_step_diagram(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    x: i32,
    y: i32,
    current_step: &GameLogicStep,
) -> Result<(), String> {
    let steps = [
        GameLogicStep::ProcessPlayers,
        GameLogicStep::ProcessMonsters,
        GameLogicStep::ProcessObjects,
        GameLogicStep::ProcessMissiles,
    ];

    let colors = [
        Color::RGB(100, 200, 100),
        Color::RGB(200, 100, 100),
        Color::RGB(100, 100, 200),
        Color::RGB(200, 200, 100),
    ];

    let box_width = 120;
    let box_height = 50;
    let spacing = 20;

    for (i, step) in steps.iter().enumerate() {
        let box_y = y + (box_height + spacing) * i as i32;
        let is_current = current_step == step;

        // Draw box
        let color = if is_current {
            Color::RGB(colors[i].r, colors[i].g, colors[i].b)
        } else {
            Color::RGB(60, 60, 80)
        };
        canvas.set_draw_color(color);
        canvas.fill_rect(Rect::new(x, box_y, box_width as u32, box_height as u32))?;

        // Border
        canvas.set_draw_color(if is_current {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(100, 100, 120)
        });
        canvas.draw_rect(Rect::new(x, box_y, box_width as u32, box_height as u32))?;

        // Arrow
        if i < steps.len() - 1 {
            canvas.set_draw_color(Color::RGB(150, 150, 150));
            let arrow_y = box_y + box_height as i32 + spacing / 2;
            canvas.draw_line((x + box_width / 2, box_y + box_height as i32),
                            (x + box_width / 2, arrow_y))?;
            // Arrow head
            canvas.draw_line((x + box_width / 2, arrow_y),
                            (x + box_width / 2 - 5, arrow_y - 5))?;
            canvas.draw_line((x + box_width / 2, arrow_y),
                            (x + box_width / 2 + 5, arrow_y - 5))?;
        }
    }

    Ok(())
}
