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

/// Logical (palette-space) resolution the C++-faithful renderer targets.
///
/// Matches C++'s PalSurface dimensions and the canvas logical size set in
/// `main.rs` (`set_logical_size(640, 480)`). The faithful `render::scrollrt`
/// pipeline draws 8-bit palette indices into a `BACKBUFFER_WIDTH ×
/// BACKBUFFER_HEIGHT` buffer that is then palette-converted and uploaded.
pub const BACKBUFFER_WIDTH: usize = 640;
pub const BACKBUFFER_HEIGHT: usize = 480;

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
    /// 8-bit palette-index backbuffer (C++ `PalSurface` equivalent). The
    /// faithful `render::scrollrt` pipeline writes palette indices here; the
    /// whole frame is then palette-converted and uploaded once per frame.
    backbuffer: Vec<u8>,
    /// RGBA scratch buffer reused across frames for palette conversion/upload.
    backbuffer_rgba: Vec<u8>,
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

        let mut window = video_subsystem
            .window(title, width, height)
            .position_centered()
            .resizable()
            .build()
            .context("Failed to create window")?;

        // CRITICAL: show the window BEFORE creating the canvas/renderer. SDL2's
        // WindowBuilder::build() returns a hidden window; if we let into_canvas
        // consume it without showing first, the resulting renderer presents to a
        // backbuffer that never reaches the screen (verified: window stays
        // visible=False, client 0x0, flat default colour on screen).
        window.show();

        // Detect headless/no-GPU video drivers (e.g. SDL_VIDEODRIVER=dummy
        // used in CI / smoke tests). Such drivers have no GPU to accelerate
        // against, so requesting SDL_RENDERER_ACCELERATED would either fail or,
        // in the dummy driver's case, segfault inside SDL_CreateRenderer. For
        // those drivers we build a plain software canvas, which is always
        // available. On a real desktop driver we keep the accelerated + vsync
        // path for the best performance/tearing behaviour.
        let driver = video_subsystem.current_video_driver();
        let headless = matches!(driver, "dummy" | "DUMMY" | "offscreen" | "OFFSCREEN");

        let canvas = if headless {
            window
                .into_canvas()
                .software()
                .build()
                .context("Failed to create software canvas")?
        } else {
            window
                .into_canvas()
                .accelerated()
                .present_vsync()
                .build()
                .context("Failed to create canvas")?
        };

        Ok(Self {
            _sdl_context: sdl_context,
            _video_subsystem: video_subsystem,
            canvas,
            width,
            height,
            frame_start: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            backbuffer: vec![0u8; BACKBUFFER_WIDTH * BACKBUFFER_HEIGHT],
            backbuffer_rgba: vec![0u8; BACKBUFFER_WIDTH * BACKBUFFER_HEIGHT * 4],
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

    /// Borrow the 8-bit palette-index backbuffer as a `(width, height, &mut [u8])`.
    ///
    /// The faithful `render::scrollrt` pipeline writes into this buffer at
    /// `BACKBUFFER_WIDTH × BACKBUFFER_HEIGHT` resolution; call
    /// [`present_backbuffer`](Self::present_backbuffer) to upload it.
    pub fn backbuffer_mut(&mut self) -> (usize, usize, &mut [u8]) {
        (BACKBUFFER_WIDTH, BACKBUFFER_HEIGHT, &mut self.backbuffer)
    }

    /// Fill the palette-index backbuffer with a single index (0 = black in
    /// Diablo's palette). C++ `ClearScreenBuffer()` equivalent.
    pub fn clear_backbuffer(&mut self) {
        self.backbuffer.fill(0);
    }

    /// Palette-convert the backbuffer and upload it to the canvas in a single
    /// blit, then draw it full-canvas.
    ///
    /// Replaces the per-tile RGBA-texture path: the whole frame is one texture
    /// upload. The palette is supplied per-frame (the current level's palette).
    pub fn present_backbuffer(&mut self, palette: &crate::engine::palette::Palette) -> Result<()> {
        palette_indices_to_rgba(&self.backbuffer, palette, &mut self.backbuffer_rgba);

        let creator = self.canvas.texture_creator();
        let mut tex = creator
            .create_texture_streaming(
                PixelFormatEnum::RGBA8888,
                BACKBUFFER_WIDTH as u32,
                BACKBUFFER_HEIGHT as u32,
            )
            .context("Failed to create backbuffer texture")?;
        tex.update(None, &self.backbuffer_rgba, BACKBUFFER_WIDTH * 4)
            .context("Failed to update backbuffer texture")?;
        self.canvas
            .copy(&tex, None, None)
            .map_err(|e| anyhow::anyhow!("Backbuffer copy failed: {}", e))?;
        Ok(())
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

/// Expand `BACKBUFFER_WIDTH × BACKBUFFER_HEIGHT` palette indices into an RGBA
/// (RGBA8888 layout) byte buffer, using `palette` for colour lookup. Factored
/// out so it can be unit-tested without an SDL context.
fn palette_indices_to_rgba(
    indices: &[u8],
    palette: &crate::engine::palette::Palette,
    out: &mut [u8],
) {
    debug_assert_eq!(indices.len() * 4, out.len());
    for (i, &idx) in indices.iter().enumerate() {
        let c = palette.get(idx);
        let o = i * 4;
        out[o] = c.r;
        out[o + 1] = c.g;
        out[o + 2] = c.b;
        out[o + 3] = 255;
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

    #[test]
    fn test_palette_indices_to_rgba() {
        use crate::engine::palette::Palette;
        // Palette with three distinguishable entries.
        let mut pal = Palette::new();
        pal.set(0, crate::engine::palette::Color::new(10, 20, 30));
        pal.set(1, crate::engine::palette::Color::new(40, 50, 60));
        pal.set(2, crate::engine::palette::Color::new(70, 80, 90));

        let indices = [0u8, 1, 2, 1];
        let mut out = vec![0u8; indices.len() * 4];
        palette_indices_to_rgba(&indices, &pal, &mut out);
        assert_eq!(out, [10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 40, 50, 60, 255]);
    }
}
