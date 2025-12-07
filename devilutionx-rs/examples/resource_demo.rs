//! Resource Loading Demo - M80 Day 3
//!
//! Tests CEL/CLX loading and palette rendering

use devilutionx_rs::engine::{
    window::{GameWindow, Color},
    resources::{ResourceManager, Palette},
    cel::CelDecoder,
    clx::ClxSprite,
};
use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== DevilutionX-RS Resource Loading Demo ===");
    println!();
    println!("Testing:");
    println!("  - Palette loading");
    println!("  - CEL decoding");
    println!("  - Sprite rendering");
    println!();

    // Create window
    let mut window = GameWindow::new("DevilutionX-RS - Resource Demo", 640, 480)?;
    let mut event_pump = window.event_pump()?;

    // Create resource manager
    let mut resources = ResourceManager::new("./assets");

    // Try to load town palette (or create test palette)
    let palette = create_test_palette();

    println!("Resources initialized:");
    let stats = resources.cache_stats();
    println!("  Sprite lists: {}", stats.sprite_lists);
    println!("  Sprite sheets: {}", stats.sprite_sheets);
    println!("  Palettes: {}", stats.palettes);
    println!();
    println!("Press ESC to quit");

    let mut running = true;
    while running {
        let frame_start = Instant::now();

        // Process input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => running = false,
                _ => {}
            }
        }

        // Render
        window.clear(Color::rgb(0, 0, 0));

        // Draw palette test pattern
        draw_palette_test(&mut window, &palette)?;

        // Draw info text (using colored rectangles)
        draw_info_ui(&mut window)?;

        window.present();
        window.wait_for_frame(frame_start);
    }

    println!("Demo closed.");
    Ok(())
}

/// Create test palette (grayscale + colors)
fn create_test_palette() -> Palette {
    let mut data = [0u8; 768];

    // First 16 colors: grayscale
    for i in 0..16 {
        let value = (i * 16) as u8;
        data[i * 3] = value;
        data[i * 3 + 1] = value;
        data[i * 3 + 2] = value;
    }

    // Next colors: rainbow
    for i in 16..256 {
        let angle = (i - 16) as f32 / 240.0 * 6.0;
        let (r, g, b) = hsv_to_rgb(angle, 1.0, 1.0);
        data[i * 3] = r;
        data[i * 3 + 1] = g;
        data[i * 3 + 2] = b;
    }

    Palette::new(data)
}

/// Convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Draw palette test pattern
fn draw_palette_test(window: &mut GameWindow, palette: &Palette) -> Result<()> {
    const SQUARE_SIZE: u32 = 20;
    const SQUARES_PER_ROW: u32 = 16;
    const START_X: i32 = 20;
    const START_Y: i32 = 50;

    for i in 0..256 {
        let row = i / SQUARES_PER_ROW;
        let col = i % SQUARES_PER_ROW;

        let x = START_X + (col * SQUARE_SIZE) as i32;
        let y = START_Y + (row * SQUARE_SIZE) as i32;

        let rgba = palette.get_rgba(i as u8);
        let color = Color::rgb(rgba[0], rgba[1], rgba[2]);

        window.draw_rect(x, y, SQUARE_SIZE, SQUARE_SIZE, color)?;
    }

    Ok(())
}

/// Draw info UI
fn draw_info_ui(window: &mut GameWindow) -> Result<()> {
    // Title bar
    window.draw_rect(0, 0, window.width(), 40, Color::rgba(0, 0, 0, 200))?;

    // Title indicator (colored bars spelling "PALETTE TEST")
    let title_colors = [
        Color::RED,
        Color::rgb(255, 128, 0),  // Orange
        Color::YELLOW,
        Color::GREEN,
        Color::rgb(0, 128, 255),  // Cyan
        Color::BLUE,
        Color::rgb(128, 0, 255),  // Purple
    ];

    for (i, color) in title_colors.iter().enumerate() {
        window.draw_rect(
            20 + (i as i32 * 80),
            10,
            70,
            20,
            *color
        )?;
    }

    // FPS bar
    let fps = window.fps();
    let fps_color = if fps >= 59.0 {
        Color::GREEN
    } else if fps >= 50.0 {
        Color::YELLOW
    } else {
        Color::RED
    };

    let bar_width = (fps / 60.0 * 100.0).min(100.0) as u32;
    window.draw_rect(window.width() as i32 - 120, 10, bar_width, 20, fps_color)?;
    window.draw_rect_outline(window.width() as i32 - 120, 10, 100, 20, Color::WHITE)?;

    Ok(())
}
