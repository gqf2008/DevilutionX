//! Resource Manager - Asset Loading and Caching
//!
//! Manages loading and caching of game assets including:
//! - CEL/CL2 sprites
//! - TIL tilesets
//! - PAL palettes
//! - Level data
//!
//! C++ Reference: Source/engine/assets.hpp, Source/engine/load_cel.cpp

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

use crate::engine::{
    clx::{ClxSprite, ClxSpriteList, ClxSpriteSheet},
    cel::CelDecoder,
};

/// Resource manager for game assets
pub struct ResourceManager {
    /// Root directory for assets
    assets_root: String,
    
    /// Cached sprite lists
    sprite_lists: HashMap<String, Arc<ClxSpriteList>>,
    
    /// Cached sprite sheets
    sprite_sheets: HashMap<String, Arc<ClxSpriteSheet>>,
    
    /// Cached palettes
    palettes: HashMap<String, Arc<Palette>>,
}

/// Color palette (256 colors × RGB)
#[derive(Debug, Clone)]
pub struct Palette {
    /// RGB data (768 bytes: 256 colors × 3 channels)
    pub data: [u8; 768],
}

impl Palette {
    /// Create new palette from RGB data
    pub fn new(data: [u8; 768]) -> Self {
        Self { data }
    }
    
    /// Load palette from file
    /// 
    /// C++ Reference: LoadFileInMem<Palette>(name)
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to load palette: {:?}", path))?;
        
        if data.len() != 768 {
            anyhow::bail!("Invalid palette size: {} (expected 768)", data.len());
        }
        
        let mut palette_data = [0u8; 768];
        palette_data.copy_from_slice(&data);
        
        Ok(Self::new(palette_data))
    }
    
    /// Get RGBA color at index
    pub fn get_rgba(&self, index: u8) -> [u8; 4] {
        let i = index as usize * 3;
        [self.data[i], self.data[i + 1], self.data[i + 2], 255]
    }
    
    /// Convert entire palette to RGBA format
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(256 * 4);
        for i in 0..256 {
            rgba.extend_from_slice(&self.get_rgba(i as u8));
        }
        rgba
    }
}

impl ResourceManager {
    /// Create new resource manager
    pub fn new(assets_root: impl Into<String>) -> Self {
        Self {
            assets_root: assets_root.into(),
            sprite_lists: HashMap::new(),
            sprite_sheets: HashMap::new(),
            palettes: HashMap::new(),
        }
    }
    
    /// Get full path to asset file
    fn get_asset_path(&self, relative_path: &str) -> std::path::PathBuf {
        Path::new(&self.assets_root).join(relative_path)
    }
    
    /// Load CEL/CL2 sprite list (single width for all frames)
    /// 
    /// C++ Reference: LoadCel(name, width)
    pub fn load_sprite_list(&mut self, name: &str, frame_width: u16) -> Result<Arc<ClxSpriteList>> {
        if let Some(cached) = self.sprite_lists.get(name) {
            return Ok(Arc::clone(cached));
        }
        
        let cel_path = self.get_asset_path(&format!("{}.cel", name));
        let clx_path = self.get_asset_path(&format!("{}.clx", name));
        
        // Try CLX first (pre-converted format)
        let sprite_list = if clx_path.exists() {
            ClxSpriteList::load_from_file(&clx_path)?
        } else if cel_path.exists() {
            // Load CEL and convert to CLX
            let cel_data = std::fs::read(&cel_path)
                .with_context(|| format!("Failed to load CEL: {:?}", cel_path))?;
            
            CelDecoder::decode_to_sprite_list(&cel_data, frame_width)?
        } else {
            anyhow::bail!("Asset not found: {} (tried .cel and .clx)", name);
        };
        
        let arc_list = Arc::new(sprite_list);
        self.sprite_lists.insert(name.to_string(), Arc::clone(&arc_list));
        Ok(arc_list)
    }
    
    /// Load CEL/CL2 sprite sheet (different width per frame)
    /// 
    /// C++ Reference: LoadCel(name, widths[])
    pub fn load_sprite_sheet(&mut self, name: &str, frame_widths: &[u16]) -> Result<Arc<ClxSpriteSheet>> {
        if let Some(cached) = self.sprite_sheets.get(name) {
            return Ok(Arc::clone(cached));
        }
        
        let cel_path = self.get_asset_path(&format!("{}.cel", name));
        let clx_path = self.get_asset_path(&format!("{}.clx", name));
        
        let sprite_sheet = if clx_path.exists() {
            ClxSpriteSheet::load_from_file(&clx_path)?
        } else if cel_path.exists() {
            let cel_data = std::fs::read(&cel_path)
                .with_context(|| format!("Failed to load CEL: {:?}", cel_path))?;
            
            CelDecoder::decode_to_sprite_sheet(&cel_data, frame_widths)?
        } else {
            anyhow::bail!("Asset not found: {} (tried .cel and .clx)", name);
        };
        
        let arc_sheet = Arc::new(sprite_sheet);
        self.sprite_sheets.insert(name.to_string(), Arc::clone(&arc_sheet));
        Ok(arc_sheet)
    }
    
    /// Load palette file
    /// 
    /// C++ Reference: LoadFileInMem<Palette>(name)
    pub fn load_palette(&mut self, name: &str) -> Result<Arc<Palette>> {
        if let Some(cached) = self.palettes.get(name) {
            return Ok(Arc::clone(cached));
        }
        
        let pal_path = self.get_asset_path(&format!("{}.pal", name));
        let palette = Palette::load_from_file(&pal_path)?;
        
        let arc_palette = Arc::new(palette);
        self.palettes.insert(name.to_string(), Arc::clone(&arc_palette));
        Ok(arc_palette)
    }
    
    /// Clear all cached resources
    pub fn clear_cache(&mut self) {
        self.sprite_lists.clear();
        self.sprite_sheets.clear();
        self.palettes.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            sprite_lists: self.sprite_lists.len(),
            sprite_sheets: self.sprite_sheets.len(),
            palettes: self.palettes.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub sprite_lists: usize,
    pub sprite_sheets: usize,
    pub palettes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_creation() {
        let data = [0u8; 768];
        let palette = Palette::new(data);
        assert_eq!(palette.data.len(), 768);
    }

    #[test]
    fn test_palette_get_rgba() {
        let mut data = [0u8; 768];
        data[0] = 255;  // R
        data[1] = 128;  // G
        data[2] = 64;   // B
        
        let palette = Palette::new(data);
        let rgba = palette.get_rgba(0);
        assert_eq!(rgba, [255, 128, 64, 255]);
    }

    #[test]
    fn test_resource_manager_creation() {
        let rm = ResourceManager::new("assets");
        assert_eq!(rm.assets_root, "assets");
        assert_eq!(rm.cache_stats().sprite_lists, 0);
    }
}
