//! Isometric Tile Renderer
//!
//! Renders isometric tiles for dungeon floors and walls.
//!
//! Diablo uses diamond-shaped tiles in an isometric projection:
//! - Each tile is 64x32 pixels (diamond shape)
//! - Floor tiles use two triangles (left/right)
//! - World coordinates map to screen via isometric transform
//!
//! C++ Reference: Source/engine/render/scrollrt.cpp, Source/engine/render/dun_render.cpp

use crate::engine::{
    window::{GameWindow, Color},
    resources::Palette,
    clx::ClxSprite,
};
use anyhow::Result;

/// Tile dimensions
pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

/// Logical render resolution. The game canvas is configured with
/// `set_logical_size(640, 480)` (see `game_loop::draw_and_blit`), so all drawing
/// happens in this coordinate space regardless of the physical window size. These
/// constants are the single source of truth for "where the viewport centre is".
pub const LOGICAL_VIEWPORT_WIDTH: i32 = 640;
pub const LOGICAL_VIEWPORT_HEIGHT: i32 = 480;

/// A render viewport size in logical coordinates.
///
/// All isometric draw helpers take the viewport size as a parameter so they
/// centre correctly even when the physical window is larger than the logical
/// canvas (SDL scales the logical canvas up to fill the physical window). Using
/// `window.width()/height()` here was the original off-centre bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportSize {
    pub width: i32,
    pub height: i32,
}

impl ViewportSize {
    /// The default Diablo/DevilutionX logical viewport: 640x480.
    pub const LOGICAL: ViewportSize = ViewportSize {
        width: LOGICAL_VIEWPORT_WIDTH,
        height: LOGICAL_VIEWPORT_HEIGHT,
    };

    /// Create a viewport of the given pixel dimensions.
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Pixel coordinates of the viewport centre.
    pub fn center(&self) -> (i32, i32) {
        (self.width / 2, self.height / 2)
    }
}

impl Default for ViewportSize {
    fn default() -> Self {
        Self::LOGICAL
    }
}

/// Isometric coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsoPoint {
    pub x: i32,
    pub y: i32,
}

impl IsoPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert world tile coordinates to screen-space (camera-relative) pixel
    /// coordinates. The result is *relative to the viewport centre* — add the
    /// viewport centre (e.g. `(320, 240)` for a 640x480 logical canvas) to get
    /// absolute draw coordinates.
    ///
    /// C++ Reference: WorldToScreen() in world_tile.hpp
    pub fn to_screen(&self, camera_offset: IsoPoint) -> (i32, i32) {
        let screen_x = (self.x - self.y) * (TILE_WIDTH / 2) - camera_offset.x;
        let screen_y = (self.x + self.y) * (TILE_HEIGHT / 2) - camera_offset.y;
        (screen_x, screen_y)
    }

    /// Convert world tile coordinates to absolute draw coordinates within a
    /// viewport of `viewport` logical pixels. This is the centre-correct
    /// variant of [`IsoPoint::to_screen`].
    ///
    /// `camera_offset` is the same pre-computed pixel offset (already in screen
    /// space) used by [`IsoPoint::to_screen`].
    pub fn to_screen_centered(
        &self,
        camera_offset: IsoPoint,
        viewport: ViewportSize,
    ) -> (i32, i32) {
        let (sx, sy) = self.to_screen(camera_offset);
        let (cx, cy) = viewport.center();
        (cx + sx, cy + sy)
    }

    /// Convert screen coordinates to world tile coordinates
    pub fn from_screen(screen_x: i32, screen_y: i32, camera_offset: IsoPoint) -> Self {
        let wx = screen_x + camera_offset.x;
        let wy = screen_y + camera_offset.y;

        let tile_x = (wx / (TILE_WIDTH / 2) + wy / (TILE_HEIGHT / 2)) / 2;
        let tile_y = (wy / (TILE_HEIGHT / 2) - wx / (TILE_WIDTH / 2)) / 2;

        Self::new(tile_x, tile_y)
    }

    /// Convert viewport-relative pixel coordinates (i.e. absolute draw coords on
    /// the logical canvas) to world tile coordinates, accounting for the
    /// viewport centre. This is the inverse of [`IsoPoint::to_screen_centered`].
    pub fn from_screen_centered(
        screen_x: i32,
        screen_y: i32,
        camera_offset: IsoPoint,
        viewport: ViewportSize,
    ) -> Self {
        let (cx, cy) = viewport.center();
        Self::from_screen(screen_x - cx, screen_y - cy, camera_offset)
    }

    /// High-level, intuitive helper: map a world tile to absolute draw
    /// coordinates, given the **camera's tile coordinates** (not a pre-computed
    /// pixel offset) and the logical viewport.
    ///
    /// This is what a renderer typically wants: "where on the 640x480 canvas do
    /// I draw tile `(wx,wy)` when the camera sits on tile `(cam_x,cam_y)`?"
    /// The tile under the camera always lands exactly on the viewport centre.
    pub fn tile_to_screen_pixel(
        &self,
        cam_x: i32,
        cam_y: i32,
        viewport: ViewportSize,
    ) -> (i32, i32) {
        // Pixel offset of the camera tile in screen space.
        let cam_offset = IsoPoint::new(
            (cam_x - cam_y) * (TILE_WIDTH / 2),
            (cam_x + cam_y) * (TILE_HEIGHT / 2),
        );
        let (sx, sy) = self.to_screen(cam_offset);
        let (cx, cy) = viewport.center();
        (cx + sx, cy + sy)
    }

    /// Inverse of [`IsoPoint::tile_to_screen_pixel`]: given an absolute draw
    /// coordinate on the logical canvas and the camera tile, return the world
    /// tile under that pixel.
    pub fn pixel_to_tile(
        screen_x: i32,
        screen_y: i32,
        cam_x: i32,
        cam_y: i32,
        viewport: ViewportSize,
    ) -> Self {
        let cam_offset = IsoPoint::new(
            (cam_x - cam_y) * (TILE_WIDTH / 2),
            (cam_x + cam_y) * (TILE_HEIGHT / 2),
        );
        Self::from_screen_centered(screen_x, screen_y, cam_offset, viewport)
    }
}

/// Tile type for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    /// Empty/transparent
    Empty,
    /// Floor tile (both triangles)
    Floor,
    /// Left wall
    WallLeft,
    /// Right wall
    WallRight,
    /// Full wall
    WallFull,
}

/// Isometric tile renderer
pub struct IsometricRenderer {
    /// Camera position in world coordinates
    camera: IsoPoint,
    /// Logical viewport size used to centre tiles. Defaults to the Diablo
    /// logical canvas (640x480). Setting this to the physical window size was
    /// the original off-centre bug; we always work in logical pixels here.
    viewport: ViewportSize,
}

impl IsometricRenderer {
    /// Create new isometric renderer with the default logical viewport.
    pub fn new() -> Self {
        Self {
            camera: IsoPoint::new(0, 0),
            viewport: ViewportSize::LOGICAL,
        }
    }

    /// Set camera position
    pub fn set_camera(&mut self, x: i32, y: i32) {
        self.camera = IsoPoint::new(x, y);
    }

    /// Get camera position
    pub fn camera(&self) -> IsoPoint {
        self.camera
    }

    /// Set the logical viewport size. This should be the logical canvas size
    /// (e.g. 640x480), **not** the physical window size. All draw helpers
    /// centre on `viewport.center()`.
    pub fn set_viewport(&mut self, viewport: ViewportSize) {
        self.viewport = viewport;
    }

    /// Get the current logical viewport.
    pub fn viewport(&self) -> ViewportSize {
        self.viewport
    }

    /// Draw floor grid (debug visualization)
    ///
    /// C++ Reference: DrawFloor() in scrollrt.cpp
    pub fn draw_floor_grid(
        &self,
        window: &mut GameWindow,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let (screen_center_x, screen_center_y) = self.viewport.center();

        for ty in 0..height {
            for tx in 0..width {
                let tile_pos = IsoPoint::new(tx, ty);
                let (sx, sy) = tile_pos.to_screen(self.camera);

                let screen_x = screen_center_x + sx;
                let screen_y = screen_center_y + sy;

                // Draw diamond outline
                self.draw_tile_outline(window, screen_x, screen_y, Color::rgb(80, 80, 100))?;
            }
        }

        Ok(())
    }

    /// Draw tile outline (diamond shape)
    fn draw_tile_outline(
        &self,
        window: &mut GameWindow,
        center_x: i32,
        center_y: i32,
        color: Color,
    ) -> Result<()> {
        let half_width = TILE_WIDTH / 2;
        let half_height = TILE_HEIGHT / 2;

        // Top vertex
        let top_x = center_x;
        let top_y = center_y - half_height;

        // Right vertex
        let right_x = center_x + half_width;
        let right_y = center_y;

        // Bottom vertex
        let bottom_x = center_x;
        let bottom_y = center_y + half_height;

        // Left vertex
        let left_x = center_x - half_width;
        let left_y = center_y;

        // Draw diamond
        window.draw_line(top_x, top_y, right_x, right_y, color)?;
        window.draw_line(right_x, right_y, bottom_x, bottom_y, color)?;
        window.draw_line(bottom_x, bottom_y, left_x, left_y, color)?;
        window.draw_line(left_x, left_y, top_x, top_y, color)?;

        Ok(())
    }

    /// Draw filled floor tile
    pub fn draw_floor_tile(
        &self,
        window: &mut GameWindow,
        tile_x: i32,
        tile_y: i32,
        color: Color,
    ) -> Result<()> {
        let (screen_center_x, screen_center_y) = self.viewport.center();

        let tile_pos = IsoPoint::new(tile_x, tile_y);
        let (sx, sy) = tile_pos.to_screen(self.camera);

        let center_x = screen_center_x + sx;
        let center_y = screen_center_y + sy;

        // Fill diamond with horizontal lines
        let half_width = TILE_WIDTH / 2;
        let half_height = TILE_HEIGHT / 2;

        for dy in -half_height..=half_height {
            let width_at_y = half_width - (dy.abs() * half_width / half_height);
            let y = center_y + dy;
            let x_start = center_x - width_at_y;
            let x_end = center_x + width_at_y;

            if width_at_y > 0 {
                window.draw_line(x_start, y, x_end, y, color)?;
            }
        }

        Ok(())
    }

    /// Draw checkered floor pattern
    pub fn draw_checkered_floor(
        &self,
        window: &mut GameWindow,
        width: i32,
        height: i32,
    ) -> Result<()> {
        for ty in 0..height {
            for tx in 0..width {
                let is_dark = (tx + ty) % 2 == 0;
                let color = if is_dark {
                    Color::rgb(40, 40, 60)
                } else {
                    Color::rgb(60, 60, 80)
                };

                self.draw_floor_tile(window, tx, ty, color)?;
            }
        }

        Ok(())
    }

    /// Draw sprite at tile position
    ///
    /// C++ Reference: RenderTile() in scrollrt.cpp
    pub fn draw_sprite_at_tile(
        &self,
        window: &mut GameWindow,
        sprite: &ClxSprite,
        tile_x: i32,
        tile_y: i32,
        palette: &Palette,
    ) -> Result<()> {
        let (screen_center_x, screen_center_y) = self.viewport.center();

        let tile_pos = IsoPoint::new(tile_x, tile_y);
        let (sx, sy) = tile_pos.to_screen(self.camera);

        let screen_x = screen_center_x + sx - sprite.width as i32 / 2;
        let screen_y = screen_center_y + sy - sprite.height as i32;

        // Decode and render sprite
        let rgba = sprite.decode_rgba(&palette.data);

        // Simple pixel-by-pixel rendering (slow but correct for demo)
        for y in 0..sprite.height {
            for x in 0..sprite.width {
                let pixel_idx = (y as usize * sprite.width as usize + x as usize) * 4;
                if pixel_idx + 3 < rgba.len() {
                    let alpha = rgba[pixel_idx + 3];
                    if alpha > 0 {
                        let color = Color::rgba(
                            rgba[pixel_idx],
                            rgba[pixel_idx + 1],
                            rgba[pixel_idx + 2],
                            alpha,
                        );
                        window.draw_pixel(
                            screen_x + x as i32,
                            screen_y + y as i32,
                            color,
                        ).ok();
                    }
                }
            }
        }

        Ok(())
    }

    /// Get visible tile range for the current camera and viewport.
    ///
    /// Takes the viewport size in logical pixels (use the renderer's configured
    /// [`ViewportSize`], or pass `ViewportSize::LOGICAL` for the default
    /// 640x480 canvas). Returns `(min_x, min_y, max_x, max_y)` in tile space.
    ///
    /// `self.camera` is interpreted as a **world tile coordinate** (the intuitive
    /// model used by [`IsoPoint::tile_to_screen_pixel`]).
    pub fn get_visible_tiles(&self, viewport: ViewportSize) -> (i32, i32, i32, i32) {
        let cam_x = self.camera.x;
        let cam_y = self.camera.y;
        let half_width = viewport.width / 2;
        let half_height = viewport.height / 2;

        // Sample the four viewport corners (in logical pixels) back into tile
        // space using the intuitive pixel→tile helper, which accounts for both
        // the viewport centre and the camera tile.
        let top_left = IsoPoint::pixel_to_tile(0, 0, cam_x, cam_y, viewport);
        let top_right = IsoPoint::pixel_to_tile(viewport.width, 0, cam_x, cam_y, viewport);
        let bottom_left = IsoPoint::pixel_to_tile(0, viewport.height, cam_x, cam_y, viewport);
        let bottom_right =
            IsoPoint::pixel_to_tile(viewport.width, viewport.height, cam_x, cam_y, viewport);

        let min_x = top_left.x.min(top_right.x).min(bottom_left.x).min(bottom_right.x) - 2;
        let max_x = top_left.x.max(top_right.x).max(bottom_left.x).max(bottom_right.x) + 2;
        let min_y = top_left.y.min(top_right.y).min(bottom_left.y).min(bottom_right.y) - 2;
        let max_y = top_left.y.max(top_right.y).max(bottom_left.y).max(bottom_right.y) + 2;

        (min_x.max(0), min_y.max(0), max_x, max_y)
    }
}

impl Default for IsometricRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_point_creation() {
        let p = IsoPoint::new(5, 10);
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 10);
    }

    #[test]
    fn test_to_screen_origin() {
        let p = IsoPoint::new(0, 0);
        let camera = IsoPoint::new(0, 0);
        let (sx, sy) = p.to_screen(camera);
        assert_eq!(sx, 0);
        assert_eq!(sy, 0);
    }

    #[test]
    fn test_to_screen_offset() {
        let p = IsoPoint::new(1, 0);
        let camera = IsoPoint::new(0, 0);
        let (sx, sy) = p.to_screen(camera);
        assert_eq!(sx, TILE_WIDTH / 2);
        assert_eq!(sy, TILE_HEIGHT / 2);
    }

    #[test]
    fn test_round_trip() {
        let original = IsoPoint::new(5, 7);
        let camera = IsoPoint::new(0, 0);
        let (sx, sy) = original.to_screen(camera);
        let recovered = IsoPoint::from_screen(sx, sy, camera);

        // Allow small rounding error
        assert!((recovered.x - original.x).abs() <= 1);
        assert!((recovered.y - original.y).abs() <= 1);
    }

    #[test]
    fn test_viewport_default_is_logical() {
        // The default viewport must be the logical 640x480 canvas, not the
        // physical window size. This is the core of the off-centre fix.
        assert_eq!(ViewportSize::default(), ViewportSize::LOGICAL);
        assert_eq!(ViewportSize::LOGICAL.width, 640);
        assert_eq!(ViewportSize::LOGICAL.height, 480);
    }

    #[test]
    fn test_viewport_center() {
        // The logical viewport centre is (320, 240) — where the player marker
        // is drawn. Using the physical window centre (e.g. 960,540 for 1920x1080)
        // would push everything down-right.
        let (cx, cy) = ViewportSize::LOGICAL.center();
        assert_eq!(cx, 320);
        assert_eq!(cy, 240);
    }

    #[test]
    fn test_tile_to_screen_pixel_camera_tile_at_centre() {
        // The tile under the camera must map to the viewport centre (320,240).
        let tile = IsoPoint::new(75, 68);
        let (sx, sy) = tile.tile_to_screen_pixel(75, 68, ViewportSize::LOGICAL);
        assert_eq!(sx, 320);
        assert_eq!(sy, 240);
    }

    #[test]
    fn test_tile_to_screen_pixel_offset_from_camera() {
        // Tile one step +x from the camera moves +32 px right and +16 px down.
        let tile = IsoPoint::new(51, 50);
        let (sx, sy) = tile.tile_to_screen_pixel(50, 50, ViewportSize::LOGICAL);
        assert_eq!(sx, 320 + 32);
        assert_eq!(sy, 240 + 16);
    }

    #[test]
    fn test_centered_round_trip_inverse() {
        // tile_to_screen_pixel and pixel_to_tile must be inverses (within
        // rounding) so that "draw at tile, then pick under cursor" round-trips.
        let original = IsoPoint::new(43, 33);
        let (sx, sy) = original.tile_to_screen_pixel(40, 30, ViewportSize::LOGICAL);
        let recovered = IsoPoint::pixel_to_tile(sx, sy, 40, 30, ViewportSize::LOGICAL);
        assert!((recovered.x - original.x).abs() <= 1, "x: {} vs {}", recovered.x, original.x);
        assert!((recovered.y - original.y).abs() <= 1, "y: {} vs {}", recovered.y, original.y);
    }

    #[test]
    fn test_renderer_uses_logical_viewport_not_physical() {
        // Regression: the renderer must centre on the logical viewport, not the
        // window size. The camera tile must map to the logical centre (320,240).
        let mut r = IsometricRenderer::new();
        r.set_camera(50, 50);
        let tile = IsoPoint::new(50, 50);
        let (sx, sy) = tile.tile_to_screen_pixel(50, 50, r.viewport());
        assert_eq!((sx, sy), (320, 240));
    }

    #[test]
    fn test_renderer_viewport_get_visible_tiles_uses_logical() {
        // get_visible_tiles now takes a ViewportSize (logical), not a physical
        // u32,u32 pair. With the 640x480 logical viewport the visible range
        // around camera (50,50) must be a modest neighbourhood that includes
        // the camera tile.
        let mut r = IsometricRenderer::new();
        r.set_camera(50, 50);
        let (min_x, min_y, max_x, max_y) = r.get_visible_tiles(ViewportSize::LOGICAL);
        // Camera tile must be inside the visible range.
        assert!(min_x <= 50 && 50 <= max_x, "x range {}..{} must include 50", min_x, max_x);
        assert!(min_y <= 50 && 50 <= max_y, "y range {}..{} must include 50", min_y, max_y);
        // And the range must be bounded by roughly the 640x480 / tile size.
        // (A 1920-wide physical window would yield a much wider range, which is
        // exactly the bug this test guards against.)
        assert!(max_x - min_x < 40, "x span {} too wide for logical viewport", max_x - min_x);
        assert!(max_y - min_y < 40, "y span {} too tall for logical viewport", max_y - min_y);
    }
}
