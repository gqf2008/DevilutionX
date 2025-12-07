// SDL测试 - 确认SDL2是否正常工作
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

fn main() -> Result<(), String> {
    println!("Starting SDL2 test...");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    println!("SDL2 initialized successfully");

    let window = video_subsystem
        .window("SDL2 Test Window", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    println!("Window created successfully");

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    println!("Canvas created successfully");
    println!("Window should be visible now!");
    println!("Press ESC to quit");

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    println!("Quit event received");
                    break 'running;
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    println!("Key pressed: {:?}", key);
                }
                _ => {}
            }
        }

        // Clear screen with red color
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();

        // Draw a white rectangle
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rect(sdl2::rect::Rect::new(100, 100, 600, 400))?;

        // Present
        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }

    println!("Test complete!");
    Ok(())
}
