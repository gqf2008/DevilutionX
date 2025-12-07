//! SDL2 Window Management
//!
//! Provides window creation, rendering, and event handling via SDL2.
//!
//! C++ Reference: Source/engine/dx.cpp, Source/engine/sdl2_to_1_2_backports.cpp

use anyhow::{Result, Context};
use sdl2::{
    Sdl, VideoSubsystem, EventPump,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    pixels::PixelFormatEnum,
    rect::Rect as SdlRect,
    event::Event,
    keyboard::Keycode,
};
use std::time::{Duration, Instant};

/// Default window width
pub const DEFAULT_WIDTH: u32 = 640;

/// Default window height
pub const DEFAULT_HEIGHT: u32 = 480;

/// Target frames per second
pub const TARGET_FPS: u32 = 60;

/// Frame time in microseconds (1/60 second)
pub const FRAME_TIME_US: u64 = 16_666;

/// RGBA Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create new RGBA color
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create new RGB color (alpha = 255)
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Common colors
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);
    pub const GRAY: Color = Color::rgb(128, 128, 128);
}

impl From<Color> for sdl2::pixels::Color {
    fn from(c: Color) -> Self {
        sdl2::pixels::Color::RGBA(c.r, c.g, c.b, c.a)
    }
}

/// Main game window with SDL2 rendering
pub struct GameWindow {
    _sdl_context: Sdl,
    _video_subsystem: VideoSubsystem,
    canvas: Canvas<Window>,
    width: u32,
    height: u32,
    frame_start: Instant,
    frame_count: u64,
    fps: f64,
}

impl GameWindow {
    /// Create new game window
    ///
    /// C++ Reference: dx.cpp:ReinitializeRenderer()
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self> {
        let sdl_context = sdl2::init()
            .map_err(|e| anyhow::anyhow!("SDL2 init failed: {}", e))?;

        let video_subsystem = sdl_context.video()
            .map_err(|e| anyhow::anyhow!("Video subsystem failed: {}", e))?;

        let window = video_subsystem
            .window(title, width, height)
            .position_centered()
            .resizable()
            .build()
            .context("Failed to create window")?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .context("Failed to create canvas")?;

        Ok(Self {
            _sdl_context: sdl_context,
            _video_subsystem: video_subsystem,
            canvas,
            width,
            height,
            frame_start: Instant::now(),
            frame_count: 0,
            fps: 0.0,
        })
    }

    /// Show or hide the system cursor (useful when rendering a custom cursor)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        let _ = self._sdl_context.mouse().show_cursor(visible);
    }

    /// Get window width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get window height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get current FPS
    pub fn fps(&self) -> f64 {
        self.fps
    }


    /// Get mutable reference to canvas
    pub fn canvas_mut(&mut self) -> &mut Canvas<Window> {
        &mut self.canvas
    }

    /// Create event pump for input handling
    pub fn event_pump(&self) -> Result<EventPump> {
        self._sdl_context.event_pump()
            .map_err(|e| anyhow::anyhow!("Failed to create event pump: {}", e))
    }

    /// Clear window with color
    pub fn clear(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    /// Present rendered frame to screen
    ///
    /// C++ Reference: dx.cpp:RenderPresent()
    pub fn present(&mut self) {
        self.canvas.present();

        // Update FPS counter
        self.frame_count += 1;
        let elapsed = self.frame_start.elapsed();
        if elapsed.as_secs() >= 1 {
            self.fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.frame_start = Instant::now();
        }
    }

    /// Draw single pixel
    pub fn draw_pixel(&mut self, x: i32, y: i32, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color);
        self.canvas.draw_point((x, y))
            .map_err(|e| anyhow::anyhow!("Draw pixel failed: {}", e))
    }

    /// Draw filled rectangle
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(SdlRect::new(x, y, w, h))
            .map_err(|e| anyhow::anyhow!("Draw rect failed: {}", e))
    }

    /// Draw rectangle outline
    pub fn draw_rect_outline(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color);
        self.canvas.draw_rect(SdlRect::new(x, y, w, h))
            .map_err(|e| anyhow::anyhow!("Draw rect outline failed: {}", e))
    }

    /// Draw line
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) -> Result<()> {
        self.canvas.set_draw_color(color);
        self.canvas.draw_line((x1, y1), (x2, y2))
            .map_err(|e| anyhow::anyhow!("Draw line failed: {}", e))
    }


    /// Wait to maintain target frame rate
    pub fn wait_for_frame(&self, frame_start: Instant) {
        let frame_time = Duration::from_micros(FRAME_TIME_US);
        let elapsed = frame_start.elapsed();
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let c = Color::rgb(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::RED, Color::rgb(255, 0, 0));
    }
}
