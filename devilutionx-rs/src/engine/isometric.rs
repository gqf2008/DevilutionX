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
    
    /// Convert world tile coordinates to screen coordinates
    /// 
    /// C++ Reference: WorldToScreen() in world_tile.hpp
    pub fn to_screen(&self, camera_offset: IsoPoint) -> (i32, i32) {
        let screen_x = (self.x - self.y) * (TILE_WIDTH / 2) - camera_offset.x;
        let screen_y = (self.x + self.y) * (TILE_HEIGHT / 2) - camera_offset.y;
        (screen_x, screen_y)
    }
    
    /// Convert screen coordinates to world tile coordinates
    pub fn from_screen(screen_x: i32, screen_y: i32, camera_offset: IsoPoint) -> Self {
        let wx = screen_x + camera_offset.x;
        let wy = screen_y + camera_offset.y;
        
        let tile_x = (wx / (TILE_WIDTH / 2) + wy / (TILE_HEIGHT / 2)) / 2;
        let tile_y = (wy / (TILE_HEIGHT / 2) - wx / (TILE_WIDTH / 2)) / 2;
        
        Self::new(tile_x, tile_y)
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
}

impl IsometricRenderer {
    /// Create new isometric renderer
    pub fn new() -> Self {
        Self {
            camera: IsoPoint::new(0, 0),
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
    
    /// Draw floor grid (debug visualization)
    /// 
    /// C++ Reference: DrawFloor() in scrollrt.cpp
    pub fn draw_floor_grid(
        &self,
        window: &mut GameWindow,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let screen_center_x = window.width() as i32 / 2;
        let screen_center_y = window.height() as i32 / 2;
        
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
        let screen_center_x = window.width() as i32 / 2;
        let screen_center_y = window.height() as i32 / 2;
        
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
        let screen_center_x = window.width() as i32 / 2;
        let screen_center_y = window.height() as i32 / 2;
        
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
    
    /// Get visible tile range for current camera and screen size
    pub fn get_visible_tiles(&self, screen_width: u32, screen_height: u32) -> (i32, i32, i32, i32) {
        let half_width = screen_width as i32 / 2;
        let half_height = screen_height as i32 / 2;
        
        // Calculate corners
        let top_left = IsoPoint::from_screen(-half_width, -half_height, self.camera);
        let top_right = IsoPoint::from_screen(half_width, -half_height, self.camera);
        let bottom_left = IsoPoint::from_screen(-half_width, half_height, self.camera);
        let bottom_right = IsoPoint::from_screen(half_width, half_height, self.camera);
        
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
}
