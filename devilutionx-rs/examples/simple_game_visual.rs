/// 超级简单的游戏循环可视化 - 确保窗口显示
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};

use devilutionx_rs::game::game_state::{GameState, GameLogicStep};
use devilutionx_rs::game::player_exact::Player;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() -> Result<(), String> {
    println!("\n🎮 DevilutionX-rs 游戏循环演示");
    println!("================================\n");

    // 创建 Player
    let mut player = Player::new();
    player._p_level = 10;
    player._p_hit_points = 100 << 6;
    player._p_max_hp = 200 << 6;
    player._p_mana = 50 << 6;
    player._p_max_mana = 100 << 6;
    player._p_magic = 40;

    println!("初始 HP: {}/{}", player._p_hit_points >> 6, player._p_max_hp >> 6);
    println!("初始 Mana: {}/{}\n", player._p_mana >> 6, player._p_max_mana >> 6);

    let mut game_state = GameState::new(player, true);
    let mut rng = StdRng::seed_from_u64(12345);

    // 初始化 SDL2
    println!("正在初始化 SDL2...");
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    println!("正在创建窗口...");
    let window = video
        .window("游戏循环可视化 - 按 ESC 退出", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| format!("窗口创建失败: {}", e))?;

    println!("✓ 窗口创建成功！");

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| format!("Canvas 创建失败: {}", e))?;

    println!("\n窗口已打开！请查看屏幕...");
    println!("\n操作:");
    println!("  空格键 = 暂停/继续");
    println!("  ESC = 退出\n");
    println!("开始运行...\n");

    let mut event_pump = sdl.event_pump()?;
    let mut frame = 0u32;
    let mut paused = false;
    let mut last_update = Instant::now();

    loop {
        // 事件处理
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("\n窗口关闭，退出程序");
                    return Ok(());
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    println!("\n按下 ESC，退出程序");
                    return Ok(());
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    paused = !paused;
                    println!("{}", if paused { "⏸️  已暂停" } else { "▶️  继续" });
                }
                _ => {}
            }
        }

        // 每 100ms 更新一次游戏状态
        if !paused && last_update.elapsed() >= Duration::from_millis(100) {
            game_state.update(&mut rng);
            frame += 1;

            if frame % 10 == 0 {
                let hp = game_state.player._p_hit_points >> 6;
                let hp_max = game_state.player._p_max_hp >> 6;
                let mana = game_state.player._p_mana >> 6;
                let mana_max = game_state.player._p_max_mana >> 6;

                println!("帧 {:3} | HP {:3}/{:3} | Mana {:3}/{:3} | {:?}",
                    frame, hp, hp_max, mana, mana_max, game_state.logic_step);
            }

            last_update = Instant::now();
        }

        // 清屏
        canvas.set_draw_color(Color::RGB(30, 30, 40));
        canvas.clear();

        // 绘制标题栏（橙色）
        canvas.set_draw_color(Color::RGB(255, 180, 80));
        canvas.fill_rect(Rect::new(100, 20, 600, 50))?;

        // 绘制帧数指示器
        canvas.set_draw_color(Color::RGB(120, 120, 180));
        canvas.fill_rect(Rect::new(50, 100, 150, 40))?;

        // 当前游戏步骤颜色
        let step_color = match game_state.logic_step {
            GameLogicStep::ProcessPlayers => Color::RGB(120, 220, 120),
            GameLogicStep::ProcessMonsters => Color::RGB(220, 120, 120),
            GameLogicStep::ProcessObjects => Color::RGB(120, 120, 220),
            GameLogicStep::ProcessMissiles => Color::RGB(220, 220, 120),
            _ => Color::RGB(120, 120, 120),
        };
        canvas.set_draw_color(step_color);
        canvas.fill_rect(Rect::new(250, 100, 300, 40))?;

        // HP 条背景
        canvas.set_draw_color(Color::RGB(50, 50, 60));
        canvas.fill_rect(Rect::new(100, 200, 600, 50))?;

        // HP 条填充（红色）
        let hp = game_state.player._p_hit_points >> 6;
        let hp_max = game_state.player._p_max_hp >> 6;
        if hp_max > 0 {
            let hp_width = ((hp as f32 / hp_max as f32) * 596.0) as i32;
            canvas.set_draw_color(Color::RGB(220, 60, 60));
            canvas.fill_rect(Rect::new(102, 202, hp_width as u32, 46))?;
        }

        // Mana 条背景
        canvas.set_draw_color(Color::RGB(50, 50, 60));
        canvas.fill_rect(Rect::new(100, 280, 600, 50))?;

        // Mana 条填充（蓝色）
        let mana = game_state.player._p_mana >> 6;
        let mana_max = game_state.player._p_max_mana >> 6;
        if mana_max > 0 {
            let mana_width = ((mana as f32 / mana_max as f32) * 596.0) as i32;
            canvas.set_draw_color(Color::RGB(60, 120, 220));
            canvas.fill_rect(Rect::new(102, 282, mana_width as u32, 46))?;
        }

        // 游戏循环步骤图（右侧）
        let steps = [
            ("Players", GameLogicStep::ProcessPlayers, Color::RGB(120, 220, 120)),
            ("Monsters", GameLogicStep::ProcessMonsters, Color::RGB(220, 120, 120)),
            ("Objects", GameLogicStep::ProcessObjects, Color::RGB(120, 120, 220)),
            ("Missiles", GameLogicStep::ProcessMissiles, Color::RGB(220, 220, 120)),
        ];

        for (i, (_name, step, color)) in steps.iter().enumerate() {
            let y = 180 + i as i32 * 70;
            let is_current = &game_state.logic_step == step;

            canvas.set_draw_color(if is_current { *color } else { Color::RGB(60, 60, 70) });
            canvas.fill_rect(Rect::new(650, y, 100, 50))?;

            canvas.set_draw_color(if is_current {
                Color::RGB(255, 255, 255)
            } else {
                Color::RGB(100, 100, 110)
            });
            canvas.draw_rect(Rect::new(650, y, 100, 50))?;

            // 绘制箭头
            if i < 3 {
                canvas.set_draw_color(Color::RGB(150, 150, 150));
                canvas.draw_line((700, y + 50), (700, y + 60))?;
                canvas.draw_line((700, y + 60), (695, y + 55))?;
                canvas.draw_line((700, y + 60), (705, y + 55))?;
            }
        }

        // 状态指示器
        canvas.set_draw_color(if paused {
            Color::RGB(220, 100, 100)
        } else {
            Color::RGB(100, 220, 100)
        });
        canvas.fill_rect(Rect::new(300, 520, 200, 50))?;

        canvas.present();

        // 控制帧率 ~60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }
}
