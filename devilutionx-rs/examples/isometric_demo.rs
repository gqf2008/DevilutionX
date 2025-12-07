//! Isometric Map Demo - M80 Day 3
//!
//! Interactive demo showing isometric tile rendering

use devilutionx_rs::engine::{
    window::{GameWindow, Color},
    resources::{ResourceManager, Palette},
    isometric::{IsometricRenderer, IsoPoint},
};
use devilutionx_rs::game::input::{InputSystem, GameAction};
use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== DevilutionX-RS Isometric Map Demo ===");
    println!();
    println!("Controls:");
    println!("  WASD - Move camera");
    println!("  Arrow Keys - Move camera");
    println!("  1 - Grid view");
    println!("  2 - Checkered floor");
    println!("  ESC - Quit");
    println!();

    // Create window
    let mut window = GameWindow::new("DevilutionX-RS - Isometric Map", 800, 600)?;
    let mut event_pump = window.event_pump()?;

    // Create systems
    let mut input = InputSystem::new();
    let mut iso_renderer = IsometricRenderer::new();
    let _resources = ResourceManager::new("./assets");

    // Game state
    let mut camera_x = 5;
    let mut camera_y = 5;
    let mut view_mode = ViewMode::Checkered;
    let map_width = 20;
    let map_height = 20;

    // Create test palette
    let palette = create_test_palette();

    println!("Map size: {}x{}", map_width, map_height);
    println!("Starting camera: ({}, {})", camera_x, camera_y);
    println!();
    println!("Demo running...");

    let mut running = true;
    let mut last_frame_time = Instant::now();

    while running {
        let frame_start = Instant::now();
        let delta_time = (frame_start - last_frame_time).as_secs_f64();
        last_frame_time = frame_start;

        // Input processing
        input.begin_frame();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown { keycode: Some(key), .. } => {
                    input.on_key_down(key);

                    match key {
                        Keycode::Escape => running = false,
                        Keycode::Num1 => view_mode = ViewMode::Grid,
                        Keycode::Num2 => view_mode = ViewMode::Checkered,
                        _ => {}
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    input.on_key_up(key);
                }
                _ => {}
            }
        }

        // Update camera movement
        let (dx, dy) = input.get_movement_direction();
        if dx != 0 || dy != 0 {
            let camera_speed = 5.0; // tiles per second
            let move_amount = (camera_speed * delta_time) as i32;

            if move_amount > 0 {
                camera_x += dx * move_amount;
                camera_y += dy * move_amount;

                // Clamp camera to map bounds
                camera_x = camera_x.max(0).min(map_width - 1);
                camera_y = camera_y.max(0).min(map_height - 1);

                // Update renderer camera (convert to pixel offset)
                iso_renderer.set_camera(
                    camera_x * 32,
                    camera_y * 16,
                );
            }
        }

        // Render
        window.clear(Color::rgb(0, 0, 0));

        // Draw isometric map
        match view_mode {
            ViewMode::Grid => {
                iso_renderer.draw_floor_grid(&mut window, map_width, map_height)?;
            }
            ViewMode::Checkered => {
                iso_renderer.draw_checkered_floor(&mut window, map_width, map_height)?;
            }
        }

        // Draw player marker at camera center
        draw_player_marker(&mut window, &palette)?;

        // Draw UI
        draw_ui(&mut window, camera_x, camera_y, &view_mode)?;

        window.present();
        window.wait_for_frame(frame_start);
    }

    println!("Demo closed.");
    Ok(())
}

/// View modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Grid,
    Checkered,
}

impl ViewMode {
    fn name(&self) -> &'static str {
        match self {
            ViewMode::Grid => "Grid",
            ViewMode::Checkered => "Checkered",
        }
    }
}

/// Create test palette
fn create_test_palette() -> Palette {
    let mut data = [0u8; 768];

    // Grayscale
    for i in 0..256 {
        let value = i as u8;
        data[i * 3] = value;
        data[i * 3 + 1] = value;
        data[i * 3 + 2] = value;
    }

    Palette::new(data)
}

/// Draw player marker
fn draw_player_marker(window: &mut GameWindow, _palette: &Palette) -> Result<()> {
    let center_x = window.width() as i32 / 2;
    let center_y = window.height() as i32 / 2;

    // Draw cross
    window.draw_line(center_x - 10, center_y, center_x + 10, center_y, Color::RED)?;
    window.draw_line(center_x, center_y - 10, center_x, center_y + 10, Color::RED)?;

    // Draw circle outline
    for angle in 0..360 {
        let rad = angle as f32 * std::f32::consts::PI / 180.0;
        let x = center_x + (rad.cos() * 15.0) as i32;
        let y = center_y + (rad.sin() * 15.0) as i32;
        window.draw_pixel(x, y, Color::YELLOW).ok();
    }

    Ok(())
}

/// Draw UI
fn draw_ui(
    window: &mut GameWindow,
    camera_x: i32,
    camera_y: i32,
    view_mode: &ViewMode,
) -> Result<()> {
    // Title bar
    window.draw_rect(0, 0, window.width(), 30, Color::rgba(0, 0, 0, 200))?;

    // Mode indicator (colored bar)
    let mode_color = match view_mode {
        ViewMode::Grid => Color::BLUE,
        ViewMode::Checkered => Color::GREEN,
    };
    window.draw_rect(10, 5, 150, 20, mode_color)?;

    // Camera position (as colored bars)
    let x_width = (camera_x.min(20) * 5) as u32;
    let y_width = (camera_y.min(20) * 5) as u32;

    window.draw_rect(window.width() as i32 - 220, 5, 100, 10, Color::rgba(40, 40, 40, 200))?;
    window.draw_rect(window.width() as i32 - 220, 5, x_width, 10, Color::RED)?;

    window.draw_rect(window.width() as i32 - 220, 17, 100, 10, Color::rgba(40, 40, 40, 200))?;
    window.draw_rect(window.width() as i32 - 220, 17, y_width, 10, Color::BLUE)?;

    // FPS
    let fps = window.fps();
    let fps_color = if fps >= 59.0 {
        Color::GREEN
    } else if fps >= 50.0 {
        Color::YELLOW
    } else {
        Color::RED
    };

    let bar_width = (fps / 60.0 * 100.0).min(100.0) as u32;
    window.draw_rect(window.width() as i32 - 120, 5, bar_width, 20, fps_color)?;
    window.draw_rect_outline(window.width() as i32 - 120, 5, 100, 20, Color::WHITE)?;

    Ok(())
}
