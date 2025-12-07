/// Map Rendering and Animation System
/// Handles isometric tile rendering, animated tiles, and map effects
use super::types::Point;
use std::collections::HashMap;

/// Animated tile state
#[derive(Debug, Clone)]
pub struct AnimatedTile {
    pub tile_x: i32,
    pub tile_y: i32,
    pub base_tile_id: u32,
    pub current_frame: u32,
    pub num_frames: u32,
    pub frame_duration_ms: u32,
    pub elapsed_ms: u32,
    pub tile_type: AnimTileType,
}

/// Types of animated tiles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimTileType {
    Water,
    Lava,
    Fire,
    Torch,
    Portal,
    Fountain,
    Steam,
    Flag,
}

impl AnimatedTile {
    pub fn new(tile_x: i32, tile_y: i32, base_tile_id: u32, tile_type: AnimTileType) -> Self {
        let (num_frames, duration) = match tile_type {
            AnimTileType::Water => (4, 200),
            AnimTileType::Lava => (4, 150),
            AnimTileType::Fire => (6, 100),
            AnimTileType::Torch => (4, 120),
            AnimTileType::Portal => (8, 80),
            AnimTileType::Fountain => (6, 130),
            AnimTileType::Steam => (4, 180),
            AnimTileType::Flag => (3, 250),
        };
        
        Self {
            tile_x,
            tile_y,
            base_tile_id,
            current_frame: 0,
            num_frames,
            frame_duration_ms: duration,
            elapsed_ms: 0,
            tile_type,
        }
    }
    
    pub fn update(&mut self, delta_ms: u32) {
        self.elapsed_ms += delta_ms;
        while self.elapsed_ms >= self.frame_duration_ms {
            self.elapsed_ms -= self.frame_duration_ms;
            self.current_frame = (self.current_frame + 1) % self.num_frames;
        }
    }
    
    pub fn current_tile_id(&self) -> u32 {
        self.base_tile_id + self.current_frame
    }
}

/// Map layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLayer {
    Floor,      // Ground tiles
    Walls,      // Wall tiles
    Objects,    // Doors, chests, etc.
    Decals,     // Blood, cracks, etc.
    Overlay,    // Effects on top
}

/// A rendered tile in the map
#[derive(Debug, Clone, Copy)]
pub struct MapTile {
    pub tile_id: u32,
    pub layer: MapLayer,
    pub flip_h: bool,
    pub flip_v: bool,
    pub tint: Option<(u8, u8, u8, u8)>,
}

impl Default for MapTile {
    fn default() -> Self {
        Self {
            tile_id: 0,
            layer: MapLayer::Floor,
            flip_h: false,
            flip_v: false,
            tint: None,
        }
    }
}

impl MapTile {
    pub fn new(tile_id: u32, layer: MapLayer) -> Self {
        Self {
            tile_id,
            layer,
            ..Default::default()
        }
    }
    
    pub fn with_tint(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.tint = Some((r, g, b, a));
        self
    }
    
    pub fn flipped_h(mut self) -> Self {
        self.flip_h = true;
        self
    }
    
    pub fn flipped_v(mut self) -> Self {
        self.flip_v = true;
        self
    }
}

/// Light source in the map
#[derive(Debug, Clone)]
pub struct LightSource {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub intensity: f32,
    pub color: (u8, u8, u8),
    pub flicker: bool,
    pub flicker_amount: f32,
    flicker_timer: f32,
}

impl LightSource {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Self {
            x,
            y,
            radius,
            intensity: 1.0,
            color: (255, 200, 150), // Warm torch light
            flicker: false,
            flicker_amount: 0.2,
            flicker_timer: 0.0,
        }
    }
    
    pub fn torch(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            radius: 5.0,
            intensity: 0.8,
            color: (255, 180, 100),
            flicker: true,
            flicker_amount: 0.3,
            flicker_timer: 0.0,
        }
    }
    
    pub fn magic(x: f32, y: f32, color: (u8, u8, u8)) -> Self {
        Self {
            x,
            y,
            radius: 4.0,
            intensity: 0.6,
            color,
            flicker: false,
            flicker_amount: 0.0,
            flicker_timer: 0.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.flicker {
            self.flicker_timer += dt * 10.0;
            // Simple sine-based flicker
            let flicker = (self.flicker_timer.sin() * 0.5 + 0.5) * self.flicker_amount;
            self.intensity = 0.8 + flicker;
        }
    }
    
    /// Calculate light intensity at a given distance
    pub fn intensity_at(&self, distance: f32) -> f32 {
        if distance >= self.radius {
            return 0.0;
        }
        let falloff = 1.0 - (distance / self.radius);
        falloff * falloff * self.intensity
    }
}

/// Fog of war state for a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    Unexplored, // Never seen
    Explored,   // Seen before, now hidden
    Visible,    // Currently visible
}

/// Map renderer with animation and lighting
pub struct MapRenderer {
    /// Map dimensions
    pub width: usize,
    pub height: usize,
    /// Tile size for isometric rendering
    pub tile_width: i32,
    pub tile_height: i32,
    /// Floor layer tiles
    floor: Vec<MapTile>,
    /// Wall layer tiles
    walls: Vec<MapTile>,
    /// Object layer tiles
    objects: Vec<MapTile>,
    /// Animated tiles
    pub animated_tiles: Vec<AnimatedTile>,
    /// Light sources
    pub lights: Vec<LightSource>,
    /// Fog of war
    fog: Vec<FogState>,
    /// Light map (calculated each frame)
    lightmap: Vec<f32>,
    /// Ambient light level (0.0 - 1.0)
    pub ambient_light: f32,
}

impl MapRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            tile_width: 64,
            tile_height: 32,
            floor: vec![MapTile::default(); size],
            walls: vec![MapTile::default(); size],
            objects: vec![MapTile::default(); size],
            animated_tiles: Vec::new(),
            lights: Vec::new(),
            fog: vec![FogState::Unexplored; size],
            lightmap: vec![0.0; size],
            ambient_light: 0.3,
        }
    }
    
    /// Set tile at position
    pub fn set_tile(&mut self, x: usize, y: usize, tile: MapTile) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            match tile.layer {
                MapLayer::Floor => self.floor[idx] = tile,
                MapLayer::Walls => self.walls[idx] = tile,
                MapLayer::Objects | MapLayer::Decals | MapLayer::Overlay => {
                    self.objects[idx] = tile;
                }
            }
        }
    }
    
    /// Get tile at position
    pub fn get_tile(&self, x: usize, y: usize, layer: MapLayer) -> Option<&MapTile> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            Some(match layer {
                MapLayer::Floor => &self.floor[idx],
                MapLayer::Walls => &self.walls[idx],
                MapLayer::Objects | MapLayer::Decals | MapLayer::Overlay => &self.objects[idx],
            })
        } else {
            None
        }
    }
    
    /// Add animated tile
    pub fn add_animated_tile(&mut self, tile: AnimatedTile) {
        self.animated_tiles.push(tile);
    }
    
    /// Add light source
    pub fn add_light(&mut self, light: LightSource) {
        self.lights.push(light);
    }
    
    /// Update all animations and lighting
    pub fn update(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;
        
        // Update animated tiles
        for tile in &mut self.animated_tiles {
            tile.update(delta_ms);
        }
        
        // Update lights (flicker effect)
        for light in &mut self.lights {
            light.update(dt);
        }
        
        // Recalculate lightmap
        self.calculate_lightmap();
    }
    
    /// Calculate light values for all tiles
    fn calculate_lightmap(&mut self) {
        // Start with ambient light
        for light_val in &mut self.lightmap {
            *light_val = self.ambient_light;
        }
        
        // Add light from each source
        for light in &self.lights {
            let center_x = light.x as i32;
            let center_y = light.y as i32;
            let radius = light.radius.ceil() as i32;
            
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let tx = center_x + dx;
                    let ty = center_y + dy;
                    
                    if tx >= 0 && tx < self.width as i32 && ty >= 0 && ty < self.height as i32 {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        let intensity = light.intensity_at(distance);
                        
                        if intensity > 0.0 {
                            let idx = (ty as usize) * self.width + (tx as usize);
                            self.lightmap[idx] = (self.lightmap[idx] + intensity).min(1.0);
                        }
                    }
                }
            }
        }
    }
    
    /// Get light value at tile
    pub fn get_light(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.lightmap[y * self.width + x]
        } else {
            0.0
        }
    }
    
    /// Update fog of war from player vision
    pub fn update_fog(&mut self, player_x: i32, player_y: i32, vision_radius: i32) {
        // Mark tiles in range as visible
        for dy in -vision_radius..=vision_radius {
            for dx in -vision_radius..=vision_radius {
                let tx = player_x + dx;
                let ty = player_y + dy;
                
                if tx >= 0 && tx < self.width as i32 && ty >= 0 && ty < self.height as i32 {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= vision_radius * vision_radius {
                        // Check line of sight
                        if self.has_los(player_x, player_y, tx, ty) {
                            let idx = (ty as usize) * self.width + (tx as usize);
                            self.fog[idx] = FogState::Visible;
                        }
                    }
                }
            }
        }
    }
    
    /// Mark tiles outside vision as explored (not visible)
    pub fn fade_fog(&mut self, player_x: i32, player_y: i32, vision_radius: i32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as i32 - player_x;
                let dy = y as i32 - player_y;
                let dist_sq = dx * dx + dy * dy;
                
                let idx = y * self.width + x;
                if self.fog[idx] == FogState::Visible {
                    if dist_sq > vision_radius * vision_radius || 
                       !self.has_los(player_x, player_y, x as i32, y as i32) {
                        self.fog[idx] = FogState::Explored;
                    }
                }
            }
        }
    }
    
    /// Get fog state at tile
    pub fn get_fog(&self, x: usize, y: usize) -> FogState {
        if x < self.width && y < self.height {
            self.fog[y * self.width + x]
        } else {
            FogState::Unexplored
        }
    }
    
    /// Simple line of sight check
    fn has_los(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        // Bresenham line algorithm
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        
        let mut x = x1;
        let mut y = y1;
        
        while x != x2 || y != y2 {
            // Check if wall blocks sight
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let idx = (y as usize) * self.width + (x as usize);
                if self.walls[idx].tile_id > 0 && (x != x1 || y != y1) {
                    return false;
                }
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
        
        true
    }
    
    /// Convert world position to isometric screen coordinates
    pub fn world_to_screen(&self, world_x: i32, world_y: i32) -> (i32, i32) {
        let iso_x = (world_x - world_y) * (self.tile_width / 2);
        let iso_y = (world_x + world_y) * (self.tile_height / 2);
        (iso_x, iso_y)
    }
    
    /// Convert screen coordinates to world position
    pub fn screen_to_world(&self, screen_x: i32, screen_y: i32) -> (i32, i32) {
        let half_w = self.tile_width / 2;
        let half_h = self.tile_height / 2;
        
        let world_x = (screen_x / half_w + screen_y / half_h) / 2;
        let world_y = (screen_y / half_h - screen_x / half_w) / 2;
        (world_x, world_y)
    }
    
    /// Get tiles to render in order (back to front for isometric)
    pub fn get_render_order(&self, camera_x: i32, camera_y: i32, screen_w: i32, screen_h: i32) -> Vec<RenderTile> {
        let mut tiles = Vec::new();
        
        // Calculate visible tile range
        let (start_wx, start_wy) = self.screen_to_world(camera_x - self.tile_width, camera_y - self.tile_height);
        let (end_wx, end_wy) = self.screen_to_world(
            camera_x + screen_w + self.tile_width,
            camera_y + screen_h + self.tile_height,
        );
        
        // Expand range a bit for safety
        let min_x = (start_wx - 2).max(0) as usize;
        let min_y = (start_wy - 2).max(0) as usize;
        let max_x = ((end_wx + 2) as usize).min(self.width);
        let max_y = ((end_wy + 2) as usize).min(self.height);
        
        // Render in isometric order (back to front)
        for y in min_y..max_y {
            for x in min_x..max_x {
                let fog = self.get_fog(x, y);
                if fog == FogState::Unexplored {
                    continue;
                }
                
                let light = self.get_light(x, y);
                let (screen_x, screen_y) = self.world_to_screen(x as i32, y as i32);
                
                // Floor tile
                let floor = &self.floor[y * self.width + x];
                if floor.tile_id > 0 {
                    tiles.push(RenderTile {
                        screen_x: screen_x - camera_x,
                        screen_y: screen_y - camera_y,
                        tile: *floor,
                        light,
                        fog,
                        world_x: x as i32,
                        world_y: y as i32,
                    });
                }
                
                // Wall tile
                let wall = &self.walls[y * self.width + x];
                if wall.tile_id > 0 {
                    tiles.push(RenderTile {
                        screen_x: screen_x - camera_x,
                        screen_y: screen_y - camera_y - self.tile_height, // Walls render above floor
                        tile: *wall,
                        light,
                        fog,
                        world_x: x as i32,
                        world_y: y as i32,
                    });
                }
            }
        }
        
        tiles
    }
}

/// Tile prepared for rendering
#[derive(Debug, Clone)]
pub struct RenderTile {
    pub screen_x: i32,
    pub screen_y: i32,
    pub tile: MapTile,
    pub light: f32,
    pub fog: FogState,
    pub world_x: i32,
    pub world_y: i32,
}

impl RenderTile {
    /// Get tint color based on lighting and fog
    pub fn get_tint(&self) -> (u8, u8, u8, u8) {
        let base_tint = self.tile.tint.unwrap_or((255, 255, 255, 255));
        
        // Apply lighting
        let light_factor = match self.fog {
            FogState::Visible => self.light,
            FogState::Explored => self.light * 0.4,  // Darker when not visible
            FogState::Unexplored => 0.0,
        };
        
        let r = (base_tint.0 as f32 * light_factor) as u8;
        let g = (base_tint.1 as f32 * light_factor) as u8;
        let b = (base_tint.2 as f32 * light_factor) as u8;
        
        (r, g, b, base_tint.3)
    }
}

/// Weather effect type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherType {
    None,
    Rain,
    Snow,
    Fog,
    Sandstorm,
}

/// Weather particle
#[derive(Debug, Clone)]
pub struct WeatherParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub alpha: f32,
}

/// Weather system
pub struct WeatherSystem {
    pub weather: WeatherType,
    pub particles: Vec<WeatherParticle>,
    pub intensity: f32,
    pub wind_x: f32,
    pub wind_y: f32,
    screen_width: f32,
    screen_height: f32,
    spawn_timer: f32,
}

impl WeatherSystem {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            weather: WeatherType::None,
            particles: Vec::new(),
            intensity: 1.0,
            wind_x: 0.0,
            wind_y: 0.0,
            screen_width,
            screen_height,
            spawn_timer: 0.0,
        }
    }
    
    pub fn set_weather(&mut self, weather: WeatherType) {
        if self.weather != weather {
            self.weather = weather;
            self.particles.clear();
            
            // Set default wind for weather type
            match weather {
                WeatherType::Rain => {
                    self.wind_x = 20.0;
                    self.wind_y = 200.0;
                }
                WeatherType::Snow => {
                    self.wind_x = 10.0;
                    self.wind_y = 30.0;
                }
                WeatherType::Sandstorm => {
                    self.wind_x = 150.0;
                    self.wind_y = 20.0;
                }
                _ => {
                    self.wind_x = 0.0;
                    self.wind_y = 0.0;
                }
            }
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.weather == WeatherType::None {
            return;
        }
        
        // Update existing particles
        self.particles.retain_mut(|p| {
            p.x += (p.vx + self.wind_x) * dt;
            p.y += (p.vy + self.wind_y) * dt;
            
            // Keep if on screen
            p.x >= -50.0 && p.x < self.screen_width + 50.0 &&
            p.y >= -50.0 && p.y < self.screen_height + 50.0
        });
        
        // Spawn new particles
        self.spawn_timer += dt;
        let spawn_rate = match self.weather {
            WeatherType::Rain => 0.01 / self.intensity,
            WeatherType::Snow => 0.05 / self.intensity,
            WeatherType::Sandstorm => 0.02 / self.intensity,
            _ => 1.0,
        };
        
        while self.spawn_timer >= spawn_rate {
            self.spawn_timer -= spawn_rate;
            self.spawn_particle();
        }
    }
    
    fn spawn_particle(&mut self) {
        // Simple pseudo-random
        let seed = self.particles.len() as u32;
        let rand_x = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % 1000) as f32 / 1000.0;
        let rand_v = ((seed.wrapping_mul(987654321).wrapping_add(54321)) % 1000) as f32 / 1000.0;
        
        let particle = match self.weather {
            WeatherType::Rain => WeatherParticle {
                x: rand_x * (self.screen_width + 100.0) - 50.0,
                y: -10.0,
                vx: 0.0,
                vy: 150.0 + rand_v * 100.0,
                size: 2.0 + rand_v,
                alpha: 0.5 + rand_v * 0.3,
            },
            WeatherType::Snow => WeatherParticle {
                x: rand_x * (self.screen_width + 100.0) - 50.0,
                y: -10.0,
                vx: (rand_v - 0.5) * 30.0,
                vy: 20.0 + rand_v * 20.0,
                size: 2.0 + rand_v * 3.0,
                alpha: 0.7 + rand_v * 0.3,
            },
            WeatherType::Sandstorm => WeatherParticle {
                x: -10.0,
                y: rand_x * self.screen_height,
                vx: 50.0 + rand_v * 100.0,
                vy: (rand_v - 0.5) * 30.0,
                size: 1.0 + rand_v * 2.0,
                alpha: 0.3 + rand_v * 0.4,
            },
            _ => return,
        };
        
        self.particles.push(particle);
    }
    
    /// Get particle color for weather type
    pub fn get_color(&self) -> (u8, u8, u8) {
        match self.weather {
            WeatherType::Rain => (150, 180, 255),
            WeatherType::Snow => (255, 255, 255),
            WeatherType::Sandstorm => (210, 180, 140),
            WeatherType::Fog => (200, 200, 200),
            WeatherType::None => (0, 0, 0),
        }
    }
}
