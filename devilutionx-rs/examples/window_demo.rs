//! Window Demo - Test SDL2 window creation and basic rendering
//!
//! Shows: Window creation, FPS counter, keyboard input

use anyhow::Result;
use devilutionx_rs::engine::window::{GameWindow, Color, DEFAULT_WIDTH, DEFAULT_HEIGHT};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== DevilutionX-RS Window Demo ===");
    println!("Press ESC to quit");
    println!("Press SPACE to change color");

    let mut window = GameWindow::new("DevilutionX-RS Window Demo", DEFAULT_WIDTH, DEFAULT_HEIGHT)?;
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut running = true;
    let mut bg_color = Color::BLACK;
    let mut cycle = 0;

    while running {
        let frame_start = Instant::now();

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false;
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    // Cycle colors
                    cycle = (cycle + 1) % 5;
                    bg_color = match cycle {
                        0 => Color::BLACK,
                        1 => Color::rgb(20, 20, 40),   // Dark blue
                        2 => Color::rgb(40, 20, 20),   // Dark red
                        3 => Color::rgb(20, 40, 20),   // Dark green
                        _ => Color::rgb(40, 40, 20),   // Dark yellow
                    };
                }
                _ => {}
            }
        }

        // Render
        window.clear(bg_color);

        // Draw test pattern - grid
        for i in 0..10 {
            let x = (i * 64) as i32;
            window.draw_line(x, 0, x, DEFAULT_HEIGHT as i32, Color::GRAY).ok();
        }

        for i in 0..8 {
            let y = (i * 60) as i32;
            window.draw_line(0, y, DEFAULT_WIDTH as i32, y, Color::GRAY).ok();
        }

        // Draw colored rectangles
        window.draw_rect(50, 50, 100, 100, Color::RED).ok();
        window.draw_rect(200, 50, 100, 100, Color::GREEN).ok();
        window.draw_rect(350, 50, 100, 100, Color::BLUE).ok();

        // Draw FPS indicator (简化版，不需要字体)
        let fps = window.fps();
        let fps_color = if fps >= 59.0 {
            Color::GREEN
        } else if fps >= 50.0 {
            Color::YELLOW
        } else {
            Color::RED
        };

        // FPS bar (60 FPS = 100 pixels wide)
        let bar_width = (fps / 60.0 * 100.0) as u32;
        window.draw_rect(10, 10, bar_width, 20, fps_color).ok();
        window.draw_rect_outline(10, 10, 100, 20, Color::WHITE).ok();

        window.present();
        window.wait_for_frame(frame_start);
    }

    println!("Window demo completed successfully!");
    Ok(())
}
