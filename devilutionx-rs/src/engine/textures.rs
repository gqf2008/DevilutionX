/// Texture and Asset Loading System
/// Handles loading and caching of game textures, sprites, and tilesets
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use sdl2::pixels::PixelFormatEnum;
use anyhow::{Result, anyhow};

use crate::engine::mpq::AssetManager as MpqManager;
use crate::engine::clx::{ClxSprite, ClxSpriteList, DiabloPalette};
use crate::engine::pcx::PcxImage;

/// Unique identifier for textures
pub type TextureId = u32;

/// Texture region for sprite sheets
#[derive(Debug, Clone, Copy)]
pub struct TextureRegion {
    pub texture_id: TextureId,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl TextureRegion {
    pub fn new(texture_id: TextureId, x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { texture_id, x, y, width, height }
    }

    pub fn to_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

/// Tile definition for tileset
#[derive(Debug, Clone, Copy)]
pub struct TileDef {
    pub region: TextureRegion,
    pub passable: bool,
    pub blocks_sight: bool,
    pub animated: bool,
    pub anim_frames: u8,
    pub anim_speed: u8,
}

impl TileDef {
    pub fn new(region: TextureRegion, passable: bool) -> Self {
        Self {
            region,
            passable,
            blocks_sight: !passable,
            animated: false,
            anim_frames: 1,
            anim_speed: 0,
        }
    }

    pub fn with_animation(mut self, frames: u8, speed: u8) -> Self {
        self.animated = true;
        self.anim_frames = frames;
        self.anim_speed = speed;
        self
    }
}

/// Tileset containing multiple tiles
#[derive(Debug, Clone)]
pub struct Tileset {
    pub name: String,
    pub texture_id: TextureId,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tiles: Vec<TileDef>,
    pub columns: u32,
}

impl Tileset {
    pub fn new(name: &str, texture_id: TextureId, tile_w: u32, tile_h: u32, columns: u32) -> Self {
        Self {
            name: name.to_string(),
            texture_id,
            tile_width: tile_w,
            tile_height: tile_h,
            tiles: Vec::new(),
            columns,
        }
    }

    /// Get tile region by index
    pub fn get_tile_region(&self, tile_id: u32) -> TextureRegion {
        let col = tile_id % self.columns;
        let row = tile_id / self.columns;
        TextureRegion {
            texture_id: self.texture_id,
            x: (col * self.tile_width) as i32,
            y: (row * self.tile_height) as i32,
            width: self.tile_width,
            height: self.tile_height,
        }
    }

    /// Auto-generate simple tiles from grid
    pub fn auto_generate(&mut self, num_tiles: u32, passable_ids: &[u32]) {
        for i in 0..num_tiles {
            let region = self.get_tile_region(i);
            let passable = passable_ids.contains(&i);
            self.tiles.push(TileDef::new(region, passable));
        }
    }
}

/// Sprite definition for characters/objects
#[derive(Debug, Clone)]
pub struct SpriteDef {
    pub name: String,
    pub texture_id: TextureId,
    pub frames: Vec<TextureRegion>,
    pub frame_width: u32,
    pub frame_height: u32,
    pub pivot_x: i32,
    pub pivot_y: i32,
}

impl SpriteDef {
    pub fn new(name: &str, texture_id: TextureId, frame_w: u32, frame_h: u32) -> Self {
        Self {
            name: name.to_string(),
            texture_id,
            frames: Vec::new(),
            frame_width: frame_w,
            frame_height: frame_h,
            pivot_x: (frame_w / 2) as i32,
            pivot_y: frame_h as i32, // Bottom center
        }
    }

    /// Add frames from a horizontal strip
    pub fn add_strip(&mut self, start_x: i32, start_y: i32, num_frames: u32) {
        for i in 0..num_frames {
            self.frames.push(TextureRegion {
                texture_id: self.texture_id,
                x: start_x + (i * self.frame_width) as i32,
                y: start_y,
                width: self.frame_width,
                height: self.frame_height,
            });
        }
    }

    /// Add frames from a grid
    pub fn add_grid(&mut self, start_x: i32, start_y: i32, cols: u32, rows: u32) {
        for row in 0..rows {
            for col in 0..cols {
                self.frames.push(TextureRegion {
                    texture_id: self.texture_id,
                    x: start_x + (col * self.frame_width) as i32,
                    y: start_y + (row * self.frame_height) as i32,
                    width: self.frame_width,
                    height: self.frame_height,
                });
            }
        }
    }

    pub fn get_frame(&self, index: usize) -> Option<&TextureRegion> {
        self.frames.get(index)
    }
}

/// Loaded texture metadata
#[derive(Debug)]
pub struct TextureMeta {
    pub id: TextureId,
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Asset manager for loading and caching textures
///
/// Note: Due to SDL2's lifetime requirements, this struct holds metadata only.
/// Actual textures need to be stored in a separate HashMap with proper lifetimes.
pub struct AssetManager {
    /// Base path for assets
    base_path: PathBuf,
    /// Texture metadata
    texture_meta: HashMap<TextureId, TextureMeta>,
    /// Path to ID mapping
    path_to_id: HashMap<PathBuf, TextureId>,
    /// Next texture ID
    next_id: TextureId,
    /// Tilesets
    pub tilesets: HashMap<String, Tileset>,
    /// Sprite definitions
    pub sprites: HashMap<String, SpriteDef>,
    /// Texture names for display
    texture_names: HashMap<TextureId, String>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new("assets")
    }
}

impl AssetManager {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            texture_meta: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
            tilesets: HashMap::new(),
            sprites: HashMap::new(),
            texture_names: HashMap::new(),
        }
    }

    /// Register a texture (without loading it yet)
    pub fn register_texture(&mut self, name: &str, path: &str) -> TextureId {
        let full_path = self.base_path.join(path);

        // Check if already registered
        if let Some(&id) = self.path_to_id.get(&full_path) {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.texture_meta.insert(id, TextureMeta {
            id,
            path: full_path.clone(),
            width: 0,
            height: 0,
        });
        self.path_to_id.insert(full_path, id);
        self.texture_names.insert(id, name.to_string());

        id
    }

    /// Get texture metadata
    pub fn get_meta(&self, id: TextureId) -> Option<&TextureMeta> {
        self.texture_meta.get(&id)
    }

    /// Get texture ID by path
    pub fn get_id(&self, path: &str) -> Option<TextureId> {
        let full_path = self.base_path.join(path);
        self.path_to_id.get(&full_path).copied()
    }

    /// Get texture name
    pub fn get_name(&self, id: TextureId) -> Option<&str> {
        self.texture_names.get(&id).map(|s| s.as_str())
    }

    /// Register a tileset
    pub fn register_tileset(&mut self, name: &str, texture_path: &str, tile_w: u32, tile_h: u32, cols: u32) {
        let tex_id = self.register_texture(&format!("tileset_{}", name), texture_path);
        let tileset = Tileset::new(name, tex_id, tile_w, tile_h, cols);
        self.tilesets.insert(name.to_string(), tileset);
    }

    /// Get tileset
    pub fn get_tileset(&self, name: &str) -> Option<&Tileset> {
        self.tilesets.get(name)
    }

    /// Get mutable tileset
    pub fn get_tileset_mut(&mut self, name: &str) -> Option<&mut Tileset> {
        self.tilesets.get_mut(name)
    }

    /// Register a sprite
    pub fn register_sprite(&mut self, name: &str, texture_path: &str, frame_w: u32, frame_h: u32) -> &mut SpriteDef {
        let tex_id = self.register_texture(&format!("sprite_{}", name), texture_path);
        let sprite = SpriteDef::new(name, tex_id, frame_w, frame_h);
        self.sprites.insert(name.to_string(), sprite);
        self.sprites.get_mut(name).unwrap()
    }

    /// Get sprite definition
    pub fn get_sprite(&self, name: &str) -> Option<&SpriteDef> {
        self.sprites.get(name)
    }

    /// Create default Diablo-style assets
    pub fn create_diablo_defaults(&mut self) {
        // Register tileset (would load from real files in production)
        self.register_tileset("dungeon", "tiles/dungeon.png", 64, 32, 16);
        self.register_tileset("town", "tiles/town.png", 64, 32, 16);
        self.register_tileset("caves", "tiles/caves.png", 64, 32, 16);
        self.register_tileset("hell", "tiles/hell.png", 64, 32, 16);

        // Register player sprites
        let warrior = self.register_sprite("warrior", "plrgfx/warrior.png", 96, 96);
        warrior.add_grid(0, 0, 8, 8); // 8 directions, 8 frames per direction

        let rogue = self.register_sprite("rogue", "plrgfx/rogue.png", 96, 96);
        rogue.add_grid(0, 0, 8, 8);

        let sorcerer = self.register_sprite("sorcerer", "plrgfx/sorcerer.png", 96, 96);
        sorcerer.add_grid(0, 0, 8, 8);

        // Register monster sprites
        let skeleton = self.register_sprite("skeleton", "monsters/skeleton.png", 64, 64);
        skeleton.add_grid(0, 0, 8, 4);

        let zombie = self.register_sprite("zombie", "monsters/zombie.png", 64, 64);
        zombie.add_grid(0, 0, 8, 4);

        let fallen = self.register_sprite("fallen", "monsters/fallen.png", 48, 48);
        fallen.add_grid(0, 0, 6, 4);

        // Register item sprites
        let items = self.register_sprite("items", "objcurs.png", 28, 28);
        items.add_grid(0, 0, 16, 16);

        // Register spell effects
        let fireball = self.register_sprite("fireball", "missiles/fireball.png", 32, 32);
        fireball.add_grid(0, 0, 16, 1);

        let lightning = self.register_sprite("lightning", "missiles/lightning.png", 32, 64);
        lightning.add_strip(0, 0, 8);

        // Register UI elements
        self.register_texture("inv_panel", "ui/inv.png");
        self.register_texture("spell_icons", "ui/spells.png");
        self.register_texture("belt_bg", "ui/belt.png");
        self.register_texture("health_orb", "ui/healthorb.png");
        self.register_texture("mana_orb", "ui/manaorb.png");
    }
}

/// Helper struct for runtime texture storage with SDL2
/// This is kept separate from AssetManager due to lifetime requirements
pub struct TextureCache<'a> {
    textures: HashMap<TextureId, Texture<'a>>,
}

impl<'a> TextureCache<'a> {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// Load texture from file
    pub fn load_texture(
        &mut self,
        id: TextureId,
        path: &Path,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<()> {
        // Try to load using SDL2_image if available, otherwise create placeholder
        match self.try_load_image(path, creator) {
            Ok(tex) => {
                self.textures.insert(id, tex);
                Ok(())
            }
            Err(e) => {
                // Create placeholder texture
                eprintln!("Warning: Could not load texture {:?}: {}", path, e);
                self.create_placeholder(id, creator)?;
                Ok(())
            }
        }
    }

    fn try_load_image(
        &self,
        path: &Path,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Texture<'a>> {
        // This would use sdl2::image in production
        // For now, return an error to trigger placeholder creation
        Err(anyhow!("SDL2_image not configured, path: {:?}", path))
    }

    /// Create a colored placeholder texture
    pub fn create_placeholder(
        &mut self,
        id: TextureId,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<()> {
        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 64)
            .map_err(|e| anyhow!("Failed to create texture: {}", e))?;

        // Create checkerboard pattern
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..64 {
                for x in 0..64 {
                    let offset = y * pitch + x * 3;
                    let checker = ((x / 8) + (y / 8)) % 2 == 0;
                    if checker {
                        buffer[offset] = 255;     // R
                        buffer[offset + 1] = 0;   // G
                        buffer[offset + 2] = 255; // B (magenta)
                    } else {
                        buffer[offset] = 0;       // R
                        buffer[offset + 1] = 0;   // G
                        buffer[offset + 2] = 0;   // B (black)
                    }
                }
            }
        }).map_err(|e| anyhow!("Failed to lock texture: {}", e))?;

        self.textures.insert(id, texture);
        Ok(())
    }

    /// Get texture reference
    pub fn get(&self, id: TextureId) -> Option<&Texture<'a>> {
        self.textures.get(&id)
    }

    /// Check if texture is loaded
    pub fn is_loaded(&self, id: TextureId) -> bool {
        self.textures.contains_key(&id)
    }

    /// Load CLX sprite from MPQ and create a texture atlas
    pub fn load_clx(
        &mut self,
        id: TextureId,
        filename: &str,
        mpq: &mut MpqManager,
        palette: &DiabloPalette,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<(u32, u32, Vec<TextureRegion>)> {
        // Read file from MPQ
        let data = mpq.read_file(filename)
            .map_err(|e| anyhow!("Failed to read {}: {}", filename, e))?;

        // Parse CLX
        let sprite_list = ClxSpriteList::from_bytes(data)
            .ok_or_else(|| anyhow!("Failed to parse CLX: {}", filename))?;

        if sprite_list.is_empty() {
            return Err(anyhow!("Empty sprite list: {}", filename));
        }

        // Calculate atlas size
        // For simplicity, we'll arrange them in a grid
        let count = sprite_list.len();
        let first_frame = &sprite_list[0];
        let frame_width = first_frame.width as u32;
        let frame_height = first_frame.height as u32;

        let cols = (count as f64).sqrt().ceil() as u32;
        let rows = (count as u32 + cols - 1) / cols;

        let atlas_width = cols * frame_width;
        let atlas_height = rows * frame_height;

        // Create texture
        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, atlas_width, atlas_height)
            .map_err(|e| anyhow!("Failed to create texture: {}", e))?;

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        // Render frames to texture
        let mut regions = Vec::new();

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            // Clear buffer
            for b in buffer.iter_mut() { *b = 0; }

            for (i, sprite) in sprite_list.sprites.iter().enumerate() {
                let col = i as u32 % cols;
                let row = i as u32 / cols;
                let x = col * frame_width;
                let y = row * frame_height;

                regions.push(TextureRegion {
                    texture_id: id,
                    x: x as i32,
                    y: y as i32,
                    width: frame_width,
                    height: frame_height,
                });

                // Render sprite to buffer
                // We need to render into the locked buffer which has a specific pitch
                // ClxSprite::render_to_rgba expects a packed RGBA buffer, but here we have a pitched buffer
                // So we'll render to a temp buffer then copy

                let mut temp_buf = vec![0u8; (frame_width * frame_height * 4) as usize];
                sprite.render_to_rgba(&mut temp_buf, frame_width as usize, 0, 0, &palette.colors);

                // Copy to texture buffer
                for fy in 0..frame_height {
                    for fx in 0..frame_width {
                        let src_idx = ((fy * frame_width + fx) * 4) as usize;
                        let dst_idx = ((y + fy) as usize * pitch) + ((x + fx) as usize * 4);

                        if dst_idx + 3 < buffer.len() {
                            buffer[dst_idx] = temp_buf[src_idx];         // R
                            buffer[dst_idx + 1] = temp_buf[src_idx + 1]; // G
                            buffer[dst_idx + 2] = temp_buf[src_idx + 2]; // B
                            buffer[dst_idx + 3] = temp_buf[src_idx + 3]; // A
                        }
                    }
                }
            }
        }).map_err(|e| anyhow!("Failed to lock texture: {}", e))?;

        self.textures.insert(id, texture);

        Ok((frame_width, frame_height, regions))
    }

    /// Load CL2 sprite from MPQ (single frame or simple list)
    pub fn load_cl2(
        &mut self,
        id: TextureId,
        filename: &str,
        width: u16,
        height: u16,
        mpq: &mut MpqManager,
        palette: &DiabloPalette,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<(u32, u32, Vec<TextureRegion>)> {
        // Read file from MPQ
        let data = mpq.read_file(filename)
            .map_err(|e| anyhow!("Failed to read {}: {}", filename, e))?;

        // Parse CL2 Header
        // CL2 files start with a header that describes the frames.
        // Usually:
        // u32 num_frames (or offset to groups)
        // u32 offsets[num_frames + 1]

        if data.len() < 4 {
            return Err(anyhow!("CL2 file too small: {}", filename));
        }

        let header_val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let num_frames = if header_val < 256 {
            // Likely a frame count
            header_val
        } else {
            // Likely an offset to groups, or just a single frame file?
            // For now, assume it's a frame count if small, otherwise we might need more complex logic.
            // But standard CL2 files usually start with frame count.
            // If it's a large number, it might be the offset to the first group (if groups are used).
            // Let's assume standard CL2 for now.
            // If header_val is large, it might be the file size check for "maybeNumFrames" logic in C++.
            // But let's try to be robust.

            // If the first u32 is large, it might be that the file is just raw CL2 data (unlikely for standard assets)
            // or it uses the group system.
            // For the warrior sprite, it likely has multiple frames.

            // Let's try to detect if it's a frame count.
            // Check if data[header_val * 4 + 4] == file_size
            let check_offset = (header_val as usize) * 4 + 4;
            if check_offset < data.len() {
                let check_val = u32::from_le_bytes([
                    data[check_offset], data[check_offset+1],
                    data[check_offset+2], data[check_offset+3]
                ]);
                if check_val as usize == data.len() {
                    header_val
                } else {
                    // Fallback: treat as 1 frame?
                    1
                }
            } else {
                1
            }
        };

        let mut frames = Vec::new();

        if num_frames > 1 && num_frames < 1024 {
            // Read offsets
            let mut offsets = Vec::new();
            for i in 0..num_frames {
                let offset_idx = 4 + i as usize * 4;
                if offset_idx + 4 > data.len() { break; }
                let offset = u32::from_le_bytes([
                    data[offset_idx], data[offset_idx+1],
                    data[offset_idx+2], data[offset_idx+3]
                ]);
                offsets.push(offset);
            }

            for offset in offsets {
                let offset = offset as usize;
                if offset >= data.len() { continue; }

                // Parse frame at offset
                // Frame starts with header size (u16)
                // Then header data
                // Then pixel data
                if offset + 2 > data.len() { continue; }
                // let frame_header_size = u16::from_le_bytes([data[offset], data[offset+1]]) as usize;

                // Pixel data starts at offset + frame_header_size
                // But wait, ClxSprite::from_cl2_bytes expects the slice to start with the header size
                // because it does: let header_size = ... data[0]...
                // So we pass the slice starting at `offset`.

                if let Some(sprite) = ClxSprite::from_cl2_bytes(&data[offset..], width, height) {
                    frames.push(sprite);
                }
            }
        } else {
            // Single frame or raw data
            if let Some(sprite) = ClxSprite::from_cl2_bytes(&data, width, height) {
                frames.push(sprite);
            }
        }

        if frames.is_empty() {
             return Err(anyhow!("No valid frames found in CL2: {}", filename));
        }

        let width_u32 = width as u32;
        let height_u32 = height as u32;

        // Create texture atlas
        // Arrange in a grid
        let count = frames.len() as u32;
        let cols = (count as f64).sqrt().ceil() as u32;
        let rows = (count + cols - 1) / cols;

        let atlas_width = cols * width_u32;
        let atlas_height = rows * height_u32;

        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, atlas_width, atlas_height)
            .map_err(|e| anyhow!("Failed to create texture: {}", e))?;

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        let mut regions = Vec::new();

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            // Clear buffer
            for b in buffer.iter_mut() { *b = 0; }

            for (i, sprite) in frames.iter().enumerate() {
                let col = i as u32 % cols;
                let row = i as u32 / cols;
                let x = col * width_u32;
                let y = row * height_u32;

                regions.push(TextureRegion {
                    texture_id: id,
                    x: x as i32,
                    y: y as i32,
                    width: width_u32,
                    height: height_u32,
                });

                // Render sprite to buffer
                let mut temp_buf = vec![0u8; (width_u32 * height_u32 * 4) as usize];
                sprite.render_cl2_to_rgba(&mut temp_buf, width_u32 as usize, 0, 0, &palette.colors);

                // Copy to texture buffer
                for fy in 0..height_u32 {
                    for fx in 0..width_u32 {
                        let src_idx = ((fy * width_u32 + fx) * 4) as usize;
                        let dst_idx = ((y + fy) as usize * pitch) + ((x + fx) as usize * 4);

                        if dst_idx + 3 < buffer.len() {
                            buffer[dst_idx] = temp_buf[src_idx];         // R
                            buffer[dst_idx + 1] = temp_buf[src_idx + 1]; // G
                            buffer[dst_idx + 2] = temp_buf[src_idx + 2]; // B
                            buffer[dst_idx + 3] = temp_buf[src_idx + 3]; // A
                        }
                    }
                }
            }
        }).map_err(|e| anyhow!("Failed to lock texture: {}", e))?;

        self.textures.insert(id, texture);

        Ok((width_u32, height_u32, regions))
    }
    /// Load PCX image from MPQ
    pub fn load_pcx(
        &mut self,
        id: TextureId,
        filename: &str,
        mpq: &mut MpqManager,
        palette: &mut DiabloPalette, // Can update palette if PCX has one
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<(u32, u32)> {
        // Read file from MPQ
        let data = mpq.read_file(filename)
            .map_err(|e| anyhow!("Failed to read {}: {}", filename, e))?;

        // Parse PCX
        let pcx = PcxImage::decode(&data)
            .ok_or_else(|| anyhow!("Failed to parse PCX {}", filename))?;

        // Update global palette from PCX palette
        // PCX palette is Vec<Color>, DiabloPalette is [u8; 768]
        if pcx.palette.len() >= 256 {
            for (i, color) in pcx.palette.iter().enumerate().take(256) {
                palette.colors[i * 3] = color.r;
                palette.colors[i * 3 + 1] = color.g;
                palette.colors[i * 3 + 2] = color.b;
            }
        }

        // Create texture
        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, pcx.width, pcx.height)
            .map_err(|e| anyhow!("Failed to create texture: {}", e))?;

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..pcx.height {
                for x in 0..pcx.width {
                    let src_idx = (y as usize * pcx.width as usize) + x as usize;
                    if src_idx >= pcx.pixels.len() { break; }

                    let color_idx = pcx.pixels[src_idx] as usize;
                    // Use the PCX's own palette for rendering this texture to ensure it looks right
                    // even if we updated the global palette
                    let color = &pcx.palette[color_idx];

                    let dst_idx = (y as usize * pitch) + (x as usize * 4);
                    if dst_idx + 3 < buffer.len() {
                        buffer[dst_idx] = color.r;
                        buffer[dst_idx + 1] = color.g;
                        buffer[dst_idx + 2] = color.b;
                        buffer[dst_idx + 3] = 255; // Opaque
                    }
                }
            }
        }).map_err(|e| anyhow!("Failed to lock texture: {}", e))?;

        self.textures.insert(id, texture);

        Ok((pcx.width, pcx.height))
    }
}

impl<'a> Default for TextureCache<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Color palette for retro rendering
#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: Vec<(u8, u8, u8)>,
}

impl Default for Palette {
    fn default() -> Self {
        Self::diablo()
    }
}

impl Palette {
    /// Create Diablo-style palette
    pub fn diablo() -> Self {
        let mut colors = Vec::with_capacity(256);

        // Black and grays
        colors.push((0, 0, 0));
        for i in 1..32 {
            let v = i * 8;
            colors.push((v, v, v));
        }

        // Reds (for blood/fire)
        for i in 0..32 {
            colors.push((128 + i * 4, i * 2, 0));
        }

        // Oranges/yellows (for gold/fire)
        for i in 0..32 {
            colors.push((255, 100 + i * 4, i * 2));
        }

        // Greens (for poison)
        for i in 0..32 {
            colors.push((i * 2, 64 + i * 4, i * 2));
        }

        // Blues (for mana/magic)
        for i in 0..32 {
            colors.push((i * 2, i * 2, 128 + i * 4));
        }

        // Browns (for earth/wood)
        for i in 0..32 {
            colors.push((80 + i * 2, 40 + i, 20 + i / 2));
        }

        // Fill rest with grays
        while colors.len() < 256 {
            let idx = colors.len();
            let v = (idx % 32 * 8) as u8;
            colors.push((v, v, v));
        }

        Self { colors }
    }

    /// Get color at index
    pub fn get(&self, index: u8) -> (u8, u8, u8) {
        self.colors.get(index as usize).copied().unwrap_or((255, 0, 255))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_manager() {
        let mut manager = AssetManager::new("assets");
        let id = manager.register_texture("test", "test.png");
        assert_eq!(manager.get_name(id), Some("test"));
    }

    #[test]
    fn test_texture_region() {
        let region = TextureRegion::new(1, 0, 0, 32, 32);
        let rect = region.to_rect();
        assert_eq!(rect.width(), 32);
        assert_eq!(rect.height(), 32);
    }

    #[test]
    fn test_tileset() {
        let mut tileset = Tileset::new("test", 1, 32, 32, 16);
        tileset.auto_generate(32, &[0, 1, 2, 3]);

        assert_eq!(tileset.tiles.len(), 32);
        assert!(tileset.tiles[0].passable);
        assert!(!tileset.tiles[10].passable);
    }
}
