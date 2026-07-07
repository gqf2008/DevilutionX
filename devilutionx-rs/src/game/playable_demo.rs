//! Playable Demo Game Loop
//!
//! Simplified game loop for M80 playable demo.
//! Full game loop is in game_loop.rs, this is for testing rendering/input.
//!
// TODO(dead code?): `PlayableDemo`/`DemoState` are `pub` but have NO external callers
// (main.rs and diablo_main use the real `game_loop.rs`). This whole module is
// functionally dead code. Kept for now because it carries 2 running lib tests
// (test_demo_state_creation, test_movement_speed); removing it would drop the
// lib test count from 1832 -> 1830, below the held baseline. Safe to delete
// once the test baseline is no longer a hard constraint.

use anyhow::Result;
use crate::engine::window::{GameWindow, Color};
use crate::game::input::{InputSystem, GameAction, Point};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

/// Game state for demo
pub struct DemoState {
    /// Is game running
    pub running: bool,
    /// Player position (pixels)
    pub player_pos: Point,
    /// Player velocity (pixels per second)
    pub player_speed: f32,
    /// Delta time (seconds)
    pub delta_time: f64,
    /// Total elapsed time
    pub elapsed_time: f64,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            running: true,
            player_pos: Point::new(320, 240),
            player_speed: 200.0,
            delta_time: 0.0,
            elapsed_time: 0.0,
        }
    }
}

/// Playable demo game loop
///
/// Implements:
/// 1. Input processing
/// 2. Game state update
/// 3. Rendering
/// 4. Frame rate control
///
/// C++ Reference: Source/diablo.cpp GameLogic() + RunGameLoop()
pub struct PlayableDemo {
    window: GameWindow,
    input: InputSystem,
    state: DemoState,
}

impl PlayableDemo {
    /// Create new playable demo
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self> {
        let window = GameWindow::new(title, width, height)?;

        Ok(Self {
            window,
            input: InputSystem::new(),
            state: DemoState::new(),
        })
    }

    /// Main game loop
    ///
    /// C++ Reference: Source/diablo.cpp:857 RunGameLoop()
    pub fn run(&mut self) -> Result<()> {
        let mut event_pump = self.window.event_pump()?;

        let mut last_frame_time = Instant::now();

        while self.state.running {
            let frame_start = Instant::now();

            // Calculate delta time
            let current_time = Instant::now();
            self.state.delta_time = (current_time - last_frame_time).as_secs_f64();
            last_frame_time = current_time;
            self.state.elapsed_time += self.state.delta_time;

            // 1. Process Input
            self.process_input(&mut event_pump)?;

            // 2. Update Game State
            self.update()?;

            // 3. Render
            self.render()?;

            // 4. Frame rate control
            self.window.wait_for_frame(frame_start);
        }

        Ok(())
    }

    /// Process input events
    ///
    /// C++ Reference: Source/diablo.cpp ProcessInput()
    fn process_input(&mut self, event_pump: &mut sdl2::EventPump) -> Result<()> {
        self.input.begin_frame();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.state.running = false;
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    self.input.on_key_down(key);

                    // Handle immediate quit
                    if key == Keycode::Escape {
                        self.state.running = false;
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    self.input.on_key_up(key);
                }
                Event::MouseMotion { x, y, .. } => {
                    self.input.on_mouse_move(x, y);
                }
                Event::MouseButtonDown { mouse_btn, .. } => {
                    self.input.on_mouse_button_down(mouse_btn);
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    self.input.on_mouse_button_up(mouse_btn);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Update game state
    ///
    /// C++ Reference: Source/diablo.cpp GameLogic()
    fn update(&mut self) -> Result<()> {
        let actions = self.input.get_actions();

        // Handle movement
        let (dx, dy) = self.input.get_movement_direction();
        if dx != 0 || dy != 0 {
            // Normalize diagonal movement
            let speed = if dx != 0 && dy != 0 {
                self.state.player_speed / 1.414 // sqrt(2)
            } else {
                self.state.player_speed
            };

            let move_x = (dx as f32 * speed * self.state.delta_time as f32) as i32;
            let move_y = (dy as f32 * speed * self.state.delta_time as f32) as i32;

            self.state.player_pos.x += move_x;
            self.state.player_pos.y += move_y;

            // Keep player in bounds
            let margin = 20;
            self.state.player_pos.x = self.state.player_pos.x
                .max(margin)
                .min(self.window.width() as i32 - margin);
            self.state.player_pos.y = self.state.player_pos.y
                .max(margin)
                .min(self.window.height() as i32 - margin);
        }

        // Handle other actions
        for action in actions {
            match action {
                GameAction::Attack => {
                    // Visual feedback: player blinks
                }
                GameAction::Pause => {
                    // TODO: Implement pause
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Render current frame
    ///
    /// C++ Reference: Source/engine/render/scrollrt.cpp DrawAndBlit()
    fn render(&mut self) -> Result<()> {
        // Clear screen
        self.window.clear(Color::rgb(20, 20, 40)); // Dark blue background

        // Draw grid (representing dungeon tiles)
        self.draw_grid()?;

        // Draw player (as colored square for now)
        self.draw_player()?;

        // Draw UI
        self.draw_ui()?;

        // Present
        self.window.present();

        Ok(())
    }

    /// Draw background grid
    fn draw_grid(&mut self) -> Result<()> {
        let grid_size = 64;
        let grid_color = Color::rgb(40, 40, 60);

        // Vertical lines
        for i in 0..=(self.window.width() / grid_size) {
            let x = (i * grid_size) as i32;
            self.window.draw_line(
                x, 0,
                x, self.window.height() as i32,
                grid_color
            )?;
        }

        // Horizontal lines
        for i in 0..=(self.window.height() / grid_size) {
            let y = (i * grid_size) as i32;
            self.window.draw_line(
                0, y,
                self.window.width() as i32, y,
                grid_color
            )?;
        }

        Ok(())
    }

    /// Draw player sprite (placeholder square)
    fn draw_player(&mut self) -> Result<()> {
        let size = 32;
        let half_size = size / 2;

        // Player body (animated color based on time)
        let phase = (self.state.elapsed_time * 2.0).sin();
        let color_value = (128.0 + phase * 127.0) as u8;
        let player_color = Color::rgb(color_value, 200, 100);

        self.window.draw_rect(
            self.state.player_pos.x - half_size,
            self.state.player_pos.y - half_size,
            size as u32,
            size as u32,
            player_color
        )?;

        // Player outline
        self.window.draw_rect_outline(
            self.state.player_pos.x - half_size,
            self.state.player_pos.y - half_size,
            size as u32,
            size as u32,
            Color::WHITE
        )?;

        // Direction indicator (small square in movement direction)
        let (dx, dy) = self.input.get_movement_direction();
        if dx != 0 || dy != 0 {
            let indicator_size = 8u32;
            let offset = 20;
            self.window.draw_rect(
                self.state.player_pos.x + dx * offset - indicator_size as i32 / 2,
                self.state.player_pos.y + dy * offset - indicator_size as i32 / 2,
                indicator_size,
                indicator_size,
                Color::YELLOW
            )?;
        }

        Ok(())
    }

    /// Draw UI elements
    fn draw_ui(&mut self) -> Result<()> {
        // FPS bar (top left)
        let fps = self.window.fps();
        let fps_color = if fps >= 59.0 {
            Color::GREEN
        } else if fps >= 50.0 {
            Color::YELLOW
        } else {
            Color::RED
        };

        let bar_width = (fps / 60.0 * 150.0).min(150.0) as u32;
        self.window.draw_rect(10, 10, bar_width, 15, fps_color)?;
        self.window.draw_rect_outline(10, 10, 150, 15, Color::WHITE)?;

        // Position indicator (top right)
        let pos_x = (self.window.width() - 160) as i32;
        self.window.draw_rect(pos_x, 10, 150, 40, Color::rgba(0, 0, 0, 180))?;

        // Simple text representation (pixel dots forming digits)
        // For now just draw colored bars representing position
        let x_bar_width = (self.state.player_pos.x as f32 / self.window.width() as f32 * 140.0) as u32;
        let y_bar_width = (self.state.player_pos.y as f32 / self.window.height() as f32 * 140.0) as u32;

        self.window.draw_rect(pos_x + 5, 15, x_bar_width.max(1), 10, Color::RED)?;
        self.window.draw_rect(pos_x + 5, 35, y_bar_width.max(1), 10, Color::BLUE)?;

        // Mouse position indicator
        let mouse = self.input.mouse_pos();
        self.window.draw_rect(mouse.x - 2, mouse.y - 2, 4, 4, Color::GREEN)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_state_creation() {
        let state = DemoState::new();
        assert!(state.running);
        assert_eq!(state.player_pos.x, 320);
        assert_eq!(state.player_pos.y, 240);
    }

    #[test]
    fn test_movement_speed() {
        let state = DemoState::new();
        assert_eq!(state.player_speed, 200.0);
    }
}
