//! Rendering System - Isometric Dungeon Renderer
//!
//! This module handles all game rendering including:
//! - Dungeon tiles (floor, walls, decorations)
//! - Monsters with lighting
//! - Players with icons (mana shield, reflect)
//! - Objects (chests, doors, shrines)
//! - Missiles and effects
//! - UI overlays (health bar, automap)
//!
//! # C++ Source Reference
//! - Source/engine/render/scrollrt.cpp: Main render loop
//! - Source/engine/render/dun_render.cpp: Dungeon tile rendering
//! - Source/engine/render/clx_render.cpp: Sprite rendering
//! - Source/engine/render/light_render.cpp: Lighting effects
//!
//! # Isometric Coordinate System
//! ```text
//!        N
//!      /   \
//!    W       E
//!      \   /
//!        S
//!
//! Screen coordinates: (x increases right, y increases down)
//! World tile size: 64x32 pixels (isometric diamond)
//! ```

use anyhow::Result;

// ============================================================================
// Constants
// ============================================================================

/// Dungeon frame width in pixels
pub const DUN_FRAME_WIDTH: i32 = 64;

/// Dungeon frame height in pixels
pub const DUN_FRAME_HEIGHT: i32 = 32;

/// Tile width in isometric view
pub const TILE_WIDTH: i32 = 64;

/// Tile height in isometric view
pub const TILE_HEIGHT: i32 = 32;

/// Maximum light level (fully lit)
pub const MAX_LIGHT_LEVEL: i32 = 15;

/// Minimum light level (dark)
pub const MIN_LIGHT_LEVEL: i32 = 0;

/// Default viewport width
pub const DEFAULT_VIEWPORT_WIDTH: i32 = 640;

/// Default viewport height
pub const DEFAULT_VIEWPORT_HEIGHT: i32 = 480;

// ============================================================================
// Types
// ============================================================================

/// RGBA Color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create new color from RGBA
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create new color from RGB (alpha = 255)
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Predefined colors
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);
    pub const GOLD: Color = Color::rgb(255, 215, 0);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);

    /// Apply lighting to color (0 = dark, 15 = full bright)
    pub fn apply_light(&self, light_level: i32) -> Color {
        let factor = (light_level.clamp(0, 15) as f32) / 15.0;
        Color::rgba(
            (self.r as f32 * factor) as u8,
            (self.g as f32 * factor) as u8,
            (self.b as f32 * factor) as u8,
            self.a,
        )
    }
}

/// Rectangle for rendering
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    /// Check if point is inside rect
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }

    /// Check if rects intersect
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
}

/// Point in 2D space
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Point = Point { x: 0, y: 0 };

    /// Convert world tile to screen position
    pub fn tile_to_screen(tile_x: i32, tile_y: i32) -> Point {
        Point {
            x: (tile_x - tile_y) * (TILE_WIDTH / 2),
            y: (tile_x + tile_y) * (TILE_HEIGHT / 2),
        }
    }

    /// Convert screen position to world tile
    pub fn screen_to_tile(screen_x: i32, screen_y: i32) -> Point {
        let tx = (screen_x / (TILE_WIDTH / 2) + screen_y / (TILE_HEIGHT / 2)) / 2;
        let ty = (screen_y / (TILE_HEIGHT / 2) - screen_x / (TILE_WIDTH / 2)) / 2;
        Point { x: tx, y: ty }
    }
}

/// Displacement (offset from a point)
#[derive(Debug, Clone, Copy, Default)]
pub struct Displacement {
    pub dx: i32,
    pub dy: i32,
}

impl Displacement {
    pub const fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }

    pub const ZERO: Displacement = Displacement { dx: 0, dy: 0 };
}

/// Sprite definition for rendering
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Texture/sprite ID
    pub texture_id: u32,
    /// Source rectangle in texture
    pub src_rect: Rect,
    /// Width for rendering
    pub width: i32,
    /// Height for rendering
    pub height: i32,
    /// Offset from position
    pub offset: Displacement,
}

impl Sprite {
    pub fn new(texture_id: u32, src_rect: Rect) -> Self {
        Self {
            texture_id,
            src_rect,
            width: src_rect.width,
            height: src_rect.height,
            offset: Displacement::ZERO,
        }
    }
}

/// Render layer for draw ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    /// Floor tiles (lowest)
    Floor = 0,
    /// Floor decorations (blood, carpets)
    FloorDecor = 1,
    /// Dead bodies, items on ground
    Ground = 2,
    /// Objects (chests, shrines)
    Objects = 3,
    /// Monsters
    Monsters = 4,
    /// Players
    Players = 5,
    /// Missiles
    Missiles = 6,
    /// Effects (particles, damage numbers)
    Effects = 7,
    /// Walls (rendered after actors for proper occlusion)
    Walls = 8,
    /// UI overlays (highest)
    UI = 9,
}

/// Render command for batching
#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub layer: RenderLayer,
    pub sprite: Sprite,
    pub position: Point,
    pub light_level: i32,
    pub tint: Option<Color>,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl RenderCommand {
    pub fn new(layer: RenderLayer, sprite: Sprite, position: Point) -> Self {
        Self {
            layer,
            sprite,
            position,
            light_level: MAX_LIGHT_LEVEL,
            tint: None,
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn with_light(mut self, light_level: i32) -> Self {
        self.light_level = light_level;
        self
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = Some(tint);
        self
    }

    pub fn with_flip(mut self, flip_x: bool, flip_y: bool) -> Self {
        self.flip_x = flip_x;
        self.flip_y = flip_y;
        self
    }
}

// ============================================================================
// Viewport
// ============================================================================

/// Camera/Viewport for scrolling
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Camera position in world space (center)
    pub position: Point,
    /// Viewport size in pixels
    pub width: i32,
    pub height: i32,
    /// Zoom level (1.0 = normal)
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            position: Point::ZERO,
            width: DEFAULT_VIEWPORT_WIDTH,
            height: DEFAULT_VIEWPORT_HEIGHT,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// Create viewport with size
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            position: Point::ZERO,
            width,
            height,
            zoom: 1.0,
        }
    }

    /// Center viewport on world position
    pub fn center_on(&mut self, world_x: i32, world_y: i32) {
        self.position = Point::new(world_x, world_y);
    }

    /// Convert world position to screen position
    pub fn world_to_screen(&self, world_x: i32, world_y: i32) -> Point {
        let screen = Point::tile_to_screen(world_x, world_y);
        Point {
            x: screen.x - self.position.x + self.width / 2,
            y: screen.y - self.position.y + self.height / 2,
        }
    }

    /// Convert screen position to world tile
    pub fn screen_to_world(&self, screen_x: i32, screen_y: i32) -> Point {
        let world_screen_x = screen_x - self.width / 2 + self.position.x;
        let world_screen_y = screen_y - self.height / 2 + self.position.y;
        Point::screen_to_tile(world_screen_x, world_screen_y)
    }

    /// Get visible tile range
    pub fn get_visible_tiles(&self) -> (Point, Point) {
        let margin = 2; // Extra tiles for smooth scrolling
        let tiles_x = (self.width / TILE_WIDTH) + margin * 2;
        let tiles_y = (self.height / TILE_HEIGHT) + margin * 2;

        let center = Point::screen_to_tile(self.position.x, self.position.y);
        let min = Point::new(center.x - tiles_x / 2, center.y - tiles_y / 2);
        let max = Point::new(center.x + tiles_x / 2, center.y + tiles_y / 2);

        (min, max)
    }

    /// Check if world position is visible
    pub fn is_visible(&self, world_x: i32, world_y: i32) -> bool {
        let screen = self.world_to_screen(world_x, world_y);
        screen.x >= -TILE_WIDTH && screen.x <= self.width + TILE_WIDTH &&
        screen.y >= -TILE_HEIGHT && screen.y <= self.height + TILE_HEIGHT
    }
}

// ============================================================================
// Render Stats
// ============================================================================

/// Rendering statistics for profiling
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// Number of tiles rendered
    pub tiles_rendered: u32,
    /// Number of sprites rendered
    pub sprites_rendered: u32,
    /// Number of draw calls
    pub draw_calls: u32,
    /// Time spent rendering (microseconds)
    pub render_time_us: u64,
    /// Frames per second
    pub fps: f32,
    /// Frame count for FPS calculation
    frame_count: u32,
    /// Last FPS update time
    last_fps_time: u64,
}

impl RenderStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_frame(&mut self) {
        self.tiles_rendered = 0;
        self.sprites_rendered = 0;
        self.draw_calls = 0;
    }

    pub fn update_fps(&mut self, current_time_ms: u64) {
        self.frame_count += 1;
        let elapsed = current_time_ms - self.last_fps_time;
        if elapsed >= 1000 {
            self.fps = (self.frame_count as f32 * 1000.0) / elapsed as f32;
            self.frame_count = 0;
            self.last_fps_time = current_time_ms;
        }
    }
}

// ============================================================================
// Lighting System
// ============================================================================

/// Light source
#[derive(Debug, Clone)]
pub struct LightSource {
    /// Position in world tiles
    pub position: Point,
    /// Light radius in tiles
    pub radius: i32,
    /// Light color
    pub color: Color,
    /// Light intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Is this light flickering?
    pub flicker: bool,
}

impl LightSource {
    pub fn new(x: i32, y: i32, radius: i32) -> Self {
        Self {
            position: Point::new(x, y),
            radius,
            color: Color::WHITE,
            intensity: 1.0,
            flicker: false,
        }
    }

    pub fn torch(x: i32, y: i32) -> Self {
        Self {
            position: Point::new(x, y),
            radius: 8,
            color: Color::rgb(255, 200, 100), // Warm light
            intensity: 0.9,
            flicker: true,
        }
    }

    /// Calculate light level at a position
    pub fn get_light_at(&self, x: i32, y: i32) -> i32 {
        let dx = (x - self.position.x).abs();
        let dy = (y - self.position.y).abs();
        let distance = ((dx * dx + dy * dy) as f32).sqrt();

        if distance >= self.radius as f32 {
            return 0;
        }

        let falloff = 1.0 - (distance / self.radius as f32);
        (falloff * self.intensity * MAX_LIGHT_LEVEL as f32) as i32
    }
}

/// Light map for a dungeon level
#[derive(Debug, Clone)]
pub struct LightMap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<i32>,
    pub sources: Vec<LightSource>,
}

impl LightMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0; width * height],
            sources: Vec::new(),
        }
    }

    /// Get light level at position
    pub fn get(&self, x: i32, y: i32) -> i32 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0;
        }
        self.data[y as usize * self.width + x as usize]
    }

    /// Set light level at position
    pub fn set(&mut self, x: i32, y: i32, level: i32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.data[y as usize * self.width + x as usize] = level.clamp(0, MAX_LIGHT_LEVEL);
        }
    }

    /// Add a light source
    pub fn add_light(&mut self, light: LightSource) {
        self.sources.push(light);
    }

    /// Recalculate all lighting
    pub fn recalculate(&mut self) {
        // Clear to darkness
        self.data.fill(0);

        // Collect light contributions first to avoid borrow issues
        let mut light_contributions: Vec<(i32, i32, i32)> = Vec::new();

        for source in &self.sources {
            for dy in -source.radius..=source.radius {
                for dx in -source.radius..=source.radius {
                    let x = source.position.x + dx;
                    let y = source.position.y + dy;
                    let light = source.get_light_at(x, y);
                    if light > 0 {
                        light_contributions.push((x, y, light));
                    }
                }
            }
        }

        // Apply all light contributions
        for (x, y, light) in light_contributions {
            let current = self.get(x, y);
            self.set(x, y, current.max(light));
        }
    }

    /// Check if tile is lit
    pub fn is_lit(&self, x: i32, y: i32) -> bool {
        self.get(x, y) > 0
    }
}

// ============================================================================
// Main Renderer
// ============================================================================

/// Main rendering system
///
/// **C++ Reference**: scrollrt.cpp DrawGame()
pub struct Renderer {
    /// Viewport for camera control
    pub viewport: Viewport,
    /// Render command queue
    commands: Vec<RenderCommand>,
    /// Rendering statistics
    pub stats: RenderStats,
    /// Light map
    pub light_map: Option<LightMap>,
    /// Show automap overlay
    pub show_automap: bool,
    /// Show monster health bars
    pub show_health_bars: bool,
    /// Show item labels
    pub show_item_labels: bool,
    /// Infravision active (see in dark)
    pub infravision: bool,
    /// Debug rendering
    pub debug_render: bool,
}

impl Renderer {
    /// Create new renderer
    pub fn new() -> Result<Self> {
        Ok(Self {
            viewport: Viewport::default(),
            commands: Vec::with_capacity(1000),
            stats: RenderStats::new(),
            light_map: None,
            show_automap: false,
            show_health_bars: true,
            show_item_labels: true,
            infravision: false,
            debug_render: false,
        })
    }

    /// Set viewport size
    pub fn set_viewport_size(&mut self, width: i32, height: i32) {
        self.viewport.width = width;
        self.viewport.height = height;
    }

    /// Set light map for current level
    pub fn set_light_map(&mut self, light_map: LightMap) {
        self.light_map = Some(light_map);
    }

    /// Clear render queue
    pub fn clear(&mut self) {
        self.commands.clear();
        self.stats.reset_frame();
    }

    /// Queue a render command
    pub fn queue(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    /// Queue sprite at world position
    pub fn queue_sprite(&mut self, layer: RenderLayer, sprite: Sprite, world_x: i32, world_y: i32) {
        let screen_pos = self.viewport.world_to_screen(world_x, world_y);
        let light_level = self.get_light_level(world_x, world_y);

        let command = RenderCommand::new(layer, sprite, screen_pos)
            .with_light(light_level);
        self.queue(command);
    }

    /// Get light level at world position
    fn get_light_level(&self, x: i32, y: i32) -> i32 {
        if self.infravision {
            return MAX_LIGHT_LEVEL;
        }
        match &self.light_map {
            Some(map) => map.get(x, y),
            None => MAX_LIGHT_LEVEL, // Default fully lit
        }
    }

    /// Sort render commands by layer and y-position
    pub fn sort_commands(&mut self) {
        self.commands.sort_by(|a, b| {
            // First by layer
            match a.layer.cmp(&b.layer) {
                std::cmp::Ordering::Equal => {
                    // Then by Y position for proper depth sorting
                    a.position.y.cmp(&b.position.y)
                }
                other => other,
            }
        });
    }

    /// Present frame (sort and return commands for actual rendering)
    pub fn present(&mut self) -> &[RenderCommand] {
        self.sort_commands();
        self.stats.draw_calls = self.commands.len() as u32;
        &self.commands
    }

    // ========================================================================
    // High-Level Render Functions
    // ========================================================================

    /// Draw dungeon floor tile
    ///
    /// **C++ Reference**: DrawFloor() in scrollrt.cpp
    pub fn draw_floor(&mut self, tile_x: i32, tile_y: i32, sprite: Sprite) {
        self.queue_sprite(RenderLayer::Floor, sprite, tile_x, tile_y);
        self.stats.tiles_rendered += 1;
    }

    /// Draw dungeon wall
    ///
    /// **C++ Reference**: DrawWall() in scrollrt.cpp
    pub fn draw_wall(&mut self, tile_x: i32, tile_y: i32, sprite: Sprite) {
        self.queue_sprite(RenderLayer::Walls, sprite, tile_x, tile_y);
        self.stats.tiles_rendered += 1;
    }

    /// Draw object (chest, shrine, door, etc.)
    ///
    /// **C++ Reference**: DrawObject() in scrollrt.cpp
    pub fn draw_object(&mut self, world_x: i32, world_y: i32, sprite: Sprite) {
        self.queue_sprite(RenderLayer::Objects, sprite, world_x, world_y);
        self.stats.sprites_rendered += 1;
    }

    /// Draw monster
    ///
    /// **C++ Reference**: DrawMonster() in scrollrt.cpp
    pub fn draw_monster(&mut self, world_x: i32, world_y: i32, sprite: Sprite, is_unique: bool) {
        let screen_pos = self.viewport.world_to_screen(world_x, world_y);
        let light_level = self.get_light_level(world_x, world_y);

        let mut command = RenderCommand::new(RenderLayer::Monsters, sprite, screen_pos)
            .with_light(light_level);

        // Unique monsters get special tint
        if is_unique {
            command = command.with_tint(Color::rgb(255, 200, 200));
        }

        self.queue(command);
        self.stats.sprites_rendered += 1;
    }

    /// Draw player
    ///
    /// **C++ Reference**: DrawPlayer() in scrollrt.cpp
    pub fn draw_player(&mut self, world_x: i32, world_y: i32, sprite: Sprite, is_local: bool) {
        let screen_pos = self.viewport.world_to_screen(world_x, world_y);
        let light_level = if is_local {
            MAX_LIGHT_LEVEL // Local player always fully lit
        } else {
            self.get_light_level(world_x, world_y)
        };

        let command = RenderCommand::new(RenderLayer::Players, sprite, screen_pos)
            .with_light(light_level);
        self.queue(command);
        self.stats.sprites_rendered += 1;
    }

    /// Draw player with mana shield icon
    ///
    /// **C++ Reference**: DrawPlayerIcons() in scrollrt.cpp
    pub fn draw_player_with_icons(
        &mut self,
        world_x: i32,
        world_y: i32,
        player_sprite: Sprite,
        mana_shield: Option<Sprite>,
        reflect: Option<Sprite>,
        is_local: bool,
    ) {
        // Draw base player
        self.draw_player(world_x, world_y, player_sprite, is_local);

        let screen_pos = self.viewport.world_to_screen(world_x, world_y);

        // Draw mana shield icon
        if let Some(shield_sprite) = mana_shield {
            let icon_pos = Point::new(screen_pos.x, screen_pos.y - 32);
            let command = RenderCommand::new(RenderLayer::Effects, shield_sprite, icon_pos);
            self.queue(command);
        }

        // Draw reflect icon
        if let Some(reflect_sprite) = reflect {
            let icon_pos = Point::new(screen_pos.x, screen_pos.y - 16);
            let command = RenderCommand::new(RenderLayer::Effects, reflect_sprite, icon_pos);
            self.queue(command);
        }
    }

    /// Draw missile
    ///
    /// **C++ Reference**: DrawMissile() in scrollrt.cpp
    pub fn draw_missile(&mut self, world_x: i32, world_y: i32, sprite: Sprite, offset: Displacement) {
        let screen_pos = self.viewport.world_to_screen(world_x, world_y);
        let adjusted_pos = Point::new(screen_pos.x + offset.dx, screen_pos.y + offset.dy);

        let command = RenderCommand::new(RenderLayer::Missiles, sprite, adjusted_pos)
            .with_light(MAX_LIGHT_LEVEL); // Missiles are self-lit
        self.queue(command);
        self.stats.sprites_rendered += 1;
    }

    /// Draw ground item
    pub fn draw_ground_item(&mut self, world_x: i32, world_y: i32, sprite: Sprite) {
        self.queue_sprite(RenderLayer::Ground, sprite, world_x, world_y);
        self.stats.sprites_rendered += 1;
    }

    /// Draw particle effect
    pub fn draw_particle(&mut self, screen_x: i32, screen_y: i32, sprite: Sprite, alpha: u8) {
        let pos = Point::new(screen_x, screen_y);
        let command = RenderCommand::new(RenderLayer::Effects, sprite, pos)
            .with_tint(Color::rgba(255, 255, 255, alpha));
        self.queue(command);
    }

    /// Draw UI element (screen-space)
    pub fn draw_ui(&mut self, screen_x: i32, screen_y: i32, sprite: Sprite) {
        let pos = Point::new(screen_x, screen_y);
        let command = RenderCommand::new(RenderLayer::UI, sprite, pos);
        self.queue(command);
    }

    // ========================================================================
    // Tile Visibility
    // ========================================================================

    /// Check if tile is visible and lit
    ///
    /// **C++ Reference**: IsTileLit() in scrollrt.cpp
    pub fn is_tile_visible(&self, tile_x: i32, tile_y: i32) -> bool {
        if !self.viewport.is_visible(tile_x, tile_y) {
            return false;
        }
        if self.infravision {
            return true;
        }
        match &self.light_map {
            Some(map) => map.is_lit(tile_x, tile_y),
            None => true,
        }
    }

    // ========================================================================
    // Debug Rendering
    // ========================================================================

    /// Draw debug grid overlay
    pub fn draw_debug_grid(&mut self, min_tile: Point, max_tile: Point) {
        if !self.debug_render {
            return;
        }

        // Would draw tile grid lines
        for ty in min_tile.y..=max_tile.y {
            for tx in min_tile.x..=max_tile.x {
                // Mark tile corners
                let _screen = self.viewport.world_to_screen(tx, ty);
                // Debug drawing would happen here
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new().expect("Failed to create renderer")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);

        let c2 = Color::rgba(100, 100, 100, 128);
        assert_eq!(c2.a, 128);
    }

    #[test]
    fn test_color_lighting() {
        let white = Color::WHITE;

        let half_lit = white.apply_light(7);
        assert!(half_lit.r < 255);
        assert!(half_lit.r > 100);

        let dark = white.apply_light(0);
        assert_eq!(dark.r, 0);
        assert_eq!(dark.g, 0);
        assert_eq!(dark.b, 0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 100);

        assert!(rect.contains(50, 50));
        assert!(rect.contains(10, 10));
        assert!(!rect.contains(5, 5));
        assert!(!rect.contains(110, 110));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0, 0, 100, 100);
        let r2 = Rect::new(50, 50, 100, 100);
        let r3 = Rect::new(200, 200, 50, 50);

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_tile_to_screen() {
        let screen = Point::tile_to_screen(0, 0);
        assert_eq!(screen.x, 0);
        assert_eq!(screen.y, 0);

        let screen2 = Point::tile_to_screen(1, 0);
        assert_eq!(screen2.x, TILE_WIDTH / 2);
        assert_eq!(screen2.y, TILE_HEIGHT / 2);

        let screen3 = Point::tile_to_screen(0, 1);
        assert_eq!(screen3.x, -TILE_WIDTH / 2);
        assert_eq!(screen3.y, TILE_HEIGHT / 2);
    }

    #[test]
    fn test_viewport_world_to_screen() {
        let mut viewport = Viewport::new(640, 480);
        viewport.center_on(0, 0);

        let screen = viewport.world_to_screen(0, 0);
        assert_eq!(screen.x, 320); // Center of viewport
        assert_eq!(screen.y, 240);
    }

    #[test]
    fn test_viewport_visible_tiles() {
        let viewport = Viewport::new(640, 480);
        let (min, max) = viewport.get_visible_tiles();

        assert!(min.x < max.x);
        assert!(min.y < max.y);
    }

    #[test]
    fn test_light_source() {
        let light = LightSource::new(5, 5, 10);

        let at_source = light.get_light_at(5, 5);
        assert_eq!(at_source, MAX_LIGHT_LEVEL);

        let at_edge = light.get_light_at(15, 5);
        assert_eq!(at_edge, 0);

        let mid = light.get_light_at(8, 5);
        assert!(mid > 0 && mid < MAX_LIGHT_LEVEL);
    }

    #[test]
    fn test_light_map() {
        let mut light_map = LightMap::new(20, 20);
        light_map.add_light(LightSource::new(10, 10, 5));
        light_map.recalculate();

        assert!(light_map.is_lit(10, 10));
        assert!(!light_map.is_lit(0, 0));
    }

    #[test]
    fn test_renderer_queue() {
        let mut renderer = Renderer::new().unwrap();

        let sprite = Sprite::new(1, Rect::new(0, 0, 64, 64));
        renderer.draw_floor(5, 5, sprite);

        assert_eq!(renderer.stats.tiles_rendered, 1);
    }

    #[test]
    fn test_render_layer_ordering() {
        assert!(RenderLayer::Floor < RenderLayer::Ground);
        assert!(RenderLayer::Ground < RenderLayer::Objects);
        assert!(RenderLayer::Objects < RenderLayer::Monsters);
        assert!(RenderLayer::Monsters < RenderLayer::Players);
        assert!(RenderLayer::Players < RenderLayer::Missiles);
        assert!(RenderLayer::Missiles < RenderLayer::UI);
    }

    #[test]
    fn test_renderer_sort() {
        let mut renderer = Renderer::new().unwrap();

        // Add commands in wrong order
        let sprite = Sprite::new(1, Rect::new(0, 0, 64, 64));
        renderer.queue(RenderCommand::new(
            RenderLayer::UI,
            sprite.clone(),
            Point::new(0, 0),
        ));
        renderer.queue(RenderCommand::new(
            RenderLayer::Floor,
            sprite.clone(),
            Point::new(0, 0),
        ));
        renderer.queue(RenderCommand::new(
            RenderLayer::Monsters,
            sprite,
            Point::new(0, 0),
        ));

        let commands = renderer.present();

        assert_eq!(commands[0].layer, RenderLayer::Floor);
        assert_eq!(commands[1].layer, RenderLayer::Monsters);
        assert_eq!(commands[2].layer, RenderLayer::UI);
    }
}
