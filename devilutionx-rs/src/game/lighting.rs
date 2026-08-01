// Lighting System Module
//
// C++ References:
// - Source/lighting.cpp (595 lines)
// - Source/lighting.h
// - Source/engine/lighting_defs.hpp
//
// This module implements the light and vision system for the game,
// handling dynamic lighting, player vision, and light color cycling.

use crate::engine::dungeon::{SolData, TileProperties};
use crate::game::types::Point;

//
// Constants (from lighting_defs.hpp)
//

/// Maximum number of light sources
pub const MAX_LIGHTS: usize = 32;

/// Maximum number of vision sources (players)
pub const MAX_VISION: usize = 4;

/// Invalid light index
pub const NO_LIGHT: i32 = -1;

/// Maximum light level
pub const LIGHTS_MAX: u8 = 15;

/// Light table size (palette size)
pub const LIGHT_TABLE_SIZE: usize = 256;

/// Number of supported light levels
pub const NUM_LIGHTING_LEVELS: usize = (LIGHTS_MAX as usize) + 1;

/// Maximum crawl radius for light calculations
pub const MAX_CRAWL_RADIUS: i32 = 18;

/// Number of supported light radiuses (0-15)
pub const NUM_LIGHT_RADIUSES: usize = 16;

/// Maximum light falloff distance
pub const MAX_FALLOFF_DISTANCE: usize = 128;

/// C++ `VisionRays[23][15]` — exact port from `Source/vision.cpp`.
/// 23 rays of up to 15 points in one quadrant (0°-90°), mirrored to the other
/// three quadrants at cast time. Zero points are padding and ignored.
pub const VISION_RAYS: [[(i8, i8); 15]; 23] = [
    [(1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0), (10, 0), (11, 0), (12, 0), (13, 0), (14, 0), (15, 0)],
    [(1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 1), (9, 1), (10, 1), (11, 1), (12, 1), (13, 1), (14, 1), (15, 1)],
    [(1, 0), (2, 0), (3, 0), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1), (9, 1), (10, 1), (11, 1), (12, 2), (13, 2), (14, 2), (15, 2)],
    [(1, 0), (2, 0), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 2), (9, 2), (10, 2), (11, 2), (12, 2), (13, 3), (14, 3), (15, 3)],
    [(1, 0), (2, 1), (3, 1), (4, 1), (5, 1), (6, 2), (7, 2), (8, 2), (9, 3), (10, 3), (11, 3), (12, 3), (13, 4), (14, 4), (0, 0)],
    [(1, 0), (2, 1), (3, 1), (4, 1), (5, 2), (6, 2), (7, 3), (8, 3), (9, 3), (10, 4), (11, 4), (12, 4), (13, 5), (14, 5), (0, 0)],
    [(1, 0), (2, 1), (3, 1), (4, 2), (5, 2), (6, 3), (7, 3), (8, 3), (9, 4), (10, 4), (11, 5), (12, 5), (13, 6), (14, 6), (0, 0)],
    [(1, 1), (2, 1), (3, 2), (4, 2), (5, 3), (6, 3), (7, 4), (8, 4), (9, 5), (10, 5), (11, 6), (12, 6), (13, 7), (0, 0), (0, 0)],
    [(1, 1), (2, 1), (3, 2), (4, 2), (5, 3), (6, 4), (7, 4), (8, 5), (9, 6), (10, 6), (11, 7), (12, 7), (12, 8), (13, 8), (0, 0)],
    [(1, 1), (2, 2), (3, 2), (4, 3), (5, 4), (6, 5), (7, 5), (8, 6), (9, 7), (10, 7), (10, 8), (11, 8), (12, 9), (0, 0), (0, 0)],
    [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 5), (7, 6), (8, 7), (9, 8), (10, 9), (11, 9), (11, 10), (0, 0), (0, 0), (0, 0)],
    [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7), (8, 8), (9, 9), (10, 10), (11, 11), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (5, 6), (6, 7), (7, 8), (8, 9), (9, 10), (9, 11), (10, 11), (0, 0), (0, 0), (0, 0)],
    [(1, 1), (2, 2), (2, 3), (3, 4), (4, 5), (5, 6), (5, 7), (6, 8), (7, 9), (7, 10), (8, 10), (8, 11), (9, 12), (0, 0), (0, 0)],
    [(1, 1), (1, 2), (2, 3), (2, 4), (3, 5), (4, 6), (4, 7), (5, 8), (6, 9), (6, 10), (7, 11), (7, 12), (8, 12), (8, 13), (0, 0)],
    [(1, 1), (1, 2), (2, 3), (2, 4), (3, 5), (3, 6), (4, 7), (4, 8), (5, 9), (5, 10), (6, 11), (6, 12), (7, 13), (0, 0), (0, 0)],
    [(0, 1), (1, 2), (1, 3), (2, 4), (2, 5), (3, 6), (3, 7), (3, 8), (4, 9), (4, 10), (5, 11), (5, 12), (6, 13), (6, 14), (0, 0)],
    [(0, 1), (1, 2), (1, 3), (1, 4), (2, 5), (2, 6), (3, 7), (3, 8), (3, 9), (4, 10), (4, 11), (4, 12), (5, 13), (5, 14), (0, 0)],
    [(0, 1), (1, 2), (1, 3), (1, 4), (1, 5), (2, 6), (2, 7), (2, 8), (3, 9), (3, 10), (3, 11), (3, 12), (4, 13), (4, 14), (0, 0)],
    [(0, 1), (0, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (2, 8), (2, 9), (2, 10), (2, 11), (2, 12), (3, 13), (3, 14), (3, 15)],
    [(0, 1), (0, 2), (0, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8), (1, 9), (1, 10), (1, 11), (2, 12), (2, 13), (2, 14), (2, 15)],
    [(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (1, 8), (1, 9), (1, 10), (1, 11), (1, 12), (1, 13), (1, 14), (1, 15)],
    [(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8), (0, 9), (0, 10), (0, 11), (0, 12), (0, 13), (0, 14), (0, 15)],
];

/// Ray-length adjustment so all rays lie on an accurate circle (C++ `RayLenAdj`).
const RAY_LEN_ADJ: [u8; 23] = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 4, 3, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0];

/// Dungeon level types for lighting behavior
///
/// **C++ Reference**: `Source/engine/lighting_defs.hpp`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DungeonLevelType {
    #[default]
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,    // Hellfire
    Crypt = 6,   // Hellfire
}

impl DungeonLevelType {
    /// Check if this is a Hellfire level type
    pub fn is_hellfire(&self) -> bool {
        matches!(self, DungeonLevelType::Nest | DungeonLevelType::Crypt)
    }
}

//
// Types
//

/// Light position with offset and old position tracking
///
/// **C++ Reference**: `Source/lighting.h` - `LightPosition`
#[derive(Debug, Clone, Copy, Default)]
pub struct LightPosition {
    /// Current tile position
    pub tile: Point,
    /// Pixel offset from tile (for smooth lighting)
    pub offset: (i8, i8),
    /// Previous tile position (for change detection)
    pub old: Point,
}

impl LightPosition {
    /// Create a new light position
    pub fn new(tile: Point) -> Self {
        Self {
            tile,
            offset: (0, 0),
            old: tile,
        }
    }

    /// Create a light position with offset
    pub fn with_offset(tile: Point, offset: (i8, i8)) -> Self {
        Self {
            tile,
            offset,
            old: tile,
        }
    }
}

/// A light source (also used for vision)
///
/// **C++ Reference**: `Source/lighting.h` - `Light`
#[derive(Debug, Clone, Copy)]
pub struct Light {
    /// Light position
    pub position: LightPosition,
    /// Current light radius
    pub radius: u8,
    /// Previous radius (for change detection)
    pub old_radius: u8,
    /// Is this light source invalid/to be removed?
    pub is_invalid: bool,
    /// Has this light source changed since last update?
    pub has_changed: bool,
}

impl Light {
    /// Create a new light source
    pub fn new(position: Point, radius: u8) -> Self {
        Self {
            position: LightPosition::new(position),
            radius,
            old_radius: radius,
            is_invalid: false,
            has_changed: false,
        }
    }

    /// Mark light as changed
    pub fn mark_changed(&mut self) {
        self.has_changed = true;
        self.position.old = self.position.tile;
        self.old_radius = self.radius;
    }

    /// Mark light as invalid (to be removed)
    pub fn mark_invalid(&mut self) {
        self.is_invalid = true;
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: LightPosition::default(),
            radius: 0,
            old_radius: 0,
            is_invalid: true,
            has_changed: false,
        }
    }
}

/// Map exploration type for vision
///
/// **C++ Reference**: `Source/automap.h` - `MapExplorationType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapExplorationType {
    /// Don't explore map
    #[default]
    None = 0,
    /// Explore for self (main player)
    ExploreSelf = 1,
    /// Explore for others (friendly multiplayer)
    ExploreOthers = 2,
}

/// Light manager handling all light sources and vision
///
/// **C++ Reference**: Global state in `Source/lighting.cpp`
pub struct LightManager {
    /// Vision list (one per player)
    pub vision_list: [Light; MAX_VISION],
    /// Vision active flags
    pub vision_active: [bool; MAX_VISION],
    /// Light sources
    pub lights: [Light; MAX_LIGHTS],
    /// Active light indices
    pub active_lights: [u8; MAX_LIGHTS],
    /// Number of active lights
    pub active_light_count: usize,
    /// Light lookup tables (for rendering)
    pub light_tables: [[u8; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
    /// Light falloff tables (radius -> distance -> light level)
    pub light_falloffs: [[u8; MAX_FALLOFF_DISTANCE]; NUM_LIGHT_RADIUSES],
    /// Light cone interpolation tables (offsetX -> offsetY -> x -> y -> distance)
    pub light_cone_interpolations: [[[[u8; 16]; 16]; 8]; 8],
    /// Fully lit light table pointer (None if special case)
    pub fully_lit_table: Option<usize>,
    /// Fully dark light table pointer (None if special case)
    pub fully_dark_table: Option<usize>,
    /// Infravision color table
    pub infravision_table: [u8; 256],
    /// Stone (petrification) color table
    pub stone_table: [u8; 256],
    /// Pause overlay color table
    pub pause_table: [u8; 256],
    /// Whether lighting needs update
    pub update_lighting: bool,
    /// Whether vision needs update
    pub update_vision: bool,
    /// Pre-light buffer (for level loading)
    pub pre_light: [[u8; 112]; 112],
    /// Current light buffer
    pub light_buffer: [[u8; 112]; 112],
    /// Tiles revealed by vision rays this frame (C++ dFlags Visible bit).
    pub visible: [[bool; 112]; 112],
    /// Current level type (affects light behavior)
    pub level_type: DungeonLevelType,
    /// Debug: disable lighting
    #[cfg(debug_assertions)]
    pub disable_lighting: bool,
}

/// C++ `TileAllowsLight` (lighting.cpp:93): a micro-tile lets vision/light
/// rays continue unless its SOL data sets `TileProperties::BlockLight`
/// (solid walls). `piece` is the dPiece value at the micro-tile position.
pub fn tile_allows_light(piece: u16, sol: &SolData) -> bool {
    !sol.get(piece as usize).contains(TileProperties::BLOCK_LIGHT)
}

impl LightManager {
    /// Create a new light manager
    pub fn new() -> Self {
        Self {
            vision_list: [Light::default(); MAX_VISION],
            vision_active: [false; MAX_VISION],
            lights: [Light::default(); MAX_LIGHTS],
            active_lights: [0; MAX_LIGHTS],
            active_light_count: 0,
            light_tables: [[0; LIGHT_TABLE_SIZE]; NUM_LIGHTING_LEVELS],
            light_falloffs: [[LIGHTS_MAX; MAX_FALLOFF_DISTANCE]; NUM_LIGHT_RADIUSES],
            light_cone_interpolations: [[[[0; 16]; 16]; 8]; 8],
            fully_lit_table: Some(0),
            fully_dark_table: Some(LIGHTS_MAX as usize),
            infravision_table: [0; 256],
            stone_table: [0; 256],
            pause_table: [0; 256],
            update_lighting: false,
            update_vision: false,
            pre_light: [[0; 112]; 112],
            light_buffer: [[LIGHTS_MAX; 112]; 112],
            visible: [[false; 112]; 112],
            level_type: DungeonLevelType::default(),
            #[cfg(debug_assertions)]
            disable_lighting: false,
        }
    }

    /// Initialize lighting system
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `InitLighting()`
    pub fn init(&mut self) {
        self.active_light_count = 0;
        self.update_lighting = false;
        self.update_vision = false;

        // Initialize all lights as invalid
        for light in &mut self.lights {
            light.is_invalid = true;
        }

        // Initialize vision as inactive
        for i in 0..MAX_VISION {
            self.vision_active[i] = false;
            self.vision_list[i].is_invalid = true;
        }

        // Initialize light buffer to max darkness
        for row in &mut self.light_buffer {
            row.fill(LIGHTS_MAX);
        }
    }

    /// Generate light tables for palette-based rendering
    ///
    /// **C++ Reference**: `Source/lighting.cpp:230-295` - `MakeLightTable()`
    ///
    /// This generates 16 gradually darker translation tables for doing lighting.
    /// The palette is divided into ranges of colors, and each shade level
    /// shifts colors within their range towards black.
    pub fn make_light_table(&mut self, level_type: DungeonLevelType) {
        self.level_type = level_type;

        // Palette color ranges: { 16, 16, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16 }
        const COLOR_STEPS: [u8; 18] = [16, 16, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16];
        const BLACK: u8 = 0;
        const WHITE: u8 = 255;

        // Generate 16 gradually darker translation tables
        for shade in 0..NUM_LIGHTING_LEVELS {
            let mut color_index: u8 = 0;

            for &steps in &COLOR_STEPS {
                let shading = (shade as u8 * steps) / 16;
                let shade_start = color_index;
                let shade_end = shade_start.saturating_add(steps - 1);

                for _ in 0..steps {
                    if color_index == BLACK {
                        self.light_tables[shade][color_index as usize] = BLACK;
                        color_index = color_index.wrapping_add(1);
                        continue;
                    }

                    let mut color = shade_start as i32 + (color_index - shade_start) as i32 + shading as i32;
                    if color > shade_end as i32 || color == WHITE as i32 {
                        color = BLACK as i32;
                    }

                    self.light_tables[shade][color_index as usize] = color as u8;
                    color_index = color_index.wrapping_add(1);
                }
            }
        }

        // Make last shade pitch black
        self.light_tables[15].fill(0);

        // Set fully lit/dark table pointers
        self.fully_lit_table = Some(0);
        self.fully_dark_table = Some(LIGHTS_MAX as usize);

        // Apply level-specific modifications
        match level_type {
            DungeonLevelType::Hell => {
                // Blood wall lighting
                let shades = NUM_LIGHTING_LEVELS - 1;
                for i in 0..shades {
                    const RANGE: i32 = 16;
                    for j in 0..RANGE {
                        let mut color = ((RANGE - 1) << 4) / shades as i32 * (shades as i32 - i as i32) / RANGE * (j + 1);
                        color = 1 + (color >> 4);
                        let idx1 = (j + 1) as usize;
                        let idx2 = (31 - j) as usize;
                        self.light_tables[i][idx1] = color as u8;
                        self.light_tables[i][idx2] = color as u8;
                    }
                }
                // Disable fully lit optimization for Hell (ceiling animation uses color map)
                self.fully_lit_table = None;
            }
            DungeonLevelType::Nest | DungeonLevelType::Crypt => {
                // Make the lava fully bright
                for table in &mut self.light_tables {
                    for i in 0..16u8 {
                        table[i as usize] = i;
                    }
                }
                // Special handling for last shade
                self.light_tables[15][0] = 0;
                for i in 1..16 {
                    self.light_tables[15][i] = 1;
                }
                // Disable fully dark optimization (Hellfire tiles are never completely black)
                self.fully_dark_table = None;
            }
            _ => {}
        }

        // Generate light falloffs and cone interpolations
        self.generate_light_falloffs();
        self.generate_light_cone_interpolations();
    }

    /// Generate light falloff tables
    ///
    /// **C++ Reference**: `Source/lighting.cpp:290-315`
    ///
    /// Creates lookup tables for light intensity based on distance from light source.
    fn generate_light_falloffs(&mut self) {
        const MAX_DARKNESS: f32 = 15.0;
        const MAX_BRIGHTNESS: f32 = 0.0;

        for radius in 0..NUM_LIGHT_RADIUSES {
            let max_distance = ((radius + 1) * 8) as f32;

            for distance in 0..MAX_FALLOFF_DISTANCE {
                if distance as f32 > max_distance {
                    self.light_falloffs[radius][distance] = 15;
                } else {
                    let factor = distance as f32 / max_distance;
                    let scaled = if self.level_type.is_hellfire() {
                        // Quadratic falloff with over-exposure for Hellfire
                        let brightness = radius as f32 * 1.25;
                        let mut val = factor * factor * brightness + (MAX_DARKNESS - brightness);
                        val = val.max(MAX_BRIGHTNESS);
                        val
                    } else {
                        // Linear falloff for standard levels
                        factor * MAX_DARKNESS
                    };
                    // Round up
                    self.light_falloffs[radius][distance] = (scaled + 0.5) as u8;
                }
            }
        }
    }

    /// Generate light cone interpolation tables
    ///
    /// **C++ Reference**: `Source/lighting.cpp:317-328`
    ///
    /// Creates lookup tables for sub-tile lighting interpolation.
    /// The 8x8 offset grid allows smooth lighting as lights move between tiles.
    fn generate_light_cone_interpolations(&mut self) {
        for offset_y in 0..8 {
            for offset_x in 0..8 {
                for y in 0..16 {
                    for x in 0..16 {
                        let a = 8 * x as i32 - offset_x as i32;
                        let b = 8 * y as i32 - offset_y as i32;
                        let distance = ((a * a + b * b) as f32).sqrt() as u8;
                        self.light_cone_interpolations[offset_x][offset_y][x][y] = distance;
                    }
                }
            }
        }
    }

    /// Get light table for a given light level
    ///
    /// Returns the color translation table for the specified light level.
    /// Level 0 = fully lit, Level 15 = fully dark.
    pub fn get_light_table(&self, level: u8) -> &[u8; LIGHT_TABLE_SIZE] {
        let level = (level as usize).min(LIGHTS_MAX as usize);
        &self.light_tables[level]
    }

    /// Apply lighting to a color index
    ///
    /// Translates a palette color index through the light table for the
    /// specified light level. Used by the renderer for palette-based lighting.
    pub fn apply_lighting(&self, color_index: u8, light_level: u8) -> u8 {
        let level = (light_level as usize).min(LIGHTS_MAX as usize);
        self.light_tables[level][color_index as usize]
    }

    /// Check if fully lit table optimization is available
    pub fn has_fully_lit_optimization(&self) -> bool {
        self.fully_lit_table.is_some()
    }

    /// Check if fully dark table optimization is available
    pub fn has_fully_dark_optimization(&self) -> bool {
        self.fully_dark_table.is_some()
    }

    /// Add a light source
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `AddLight()`
    ///
    /// # Returns
    /// Light index, or NO_LIGHT if no slots available
    pub fn add_light(&mut self, position: Point, radius: u8) -> i32 {
        if self.active_light_count >= MAX_LIGHTS {
            return NO_LIGHT;
        }

        // Find an available slot
        let mut slot = None;
        for i in 0..MAX_LIGHTS {
            if self.lights[i].is_invalid {
                slot = Some(i);
                break;
            }
        }

        let slot = match slot {
            Some(s) => s,
            None => return NO_LIGHT,
        };

        // Initialize the light
        self.lights[slot] = Light::new(position, radius);
        self.lights[slot].is_invalid = false;

        // Add to active list
        self.active_lights[self.active_light_count] = slot as u8;
        self.active_light_count += 1;

        self.update_lighting = true;

        slot as i32
    }

    /// Remove a light source (mark as invalid)
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `AddUnLight()`
    pub fn remove_light(&mut self, index: i32) {
        if index < 0 || index >= MAX_LIGHTS as i32 {
            return;
        }

        self.lights[index as usize].mark_invalid();
        self.update_lighting = true;
    }

    /// Change light radius
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ChangeLightRadius()`
    pub fn change_light_radius(&mut self, index: i32, radius: u8) {
        if index < 0 || index >= MAX_LIGHTS as i32 {
            return;
        }

        let light = &mut self.lights[index as usize];
        if light.is_invalid {
            return;
        }

        light.mark_changed();
        light.radius = radius;
        self.update_lighting = true;
    }

    /// Change light position
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ChangeLightXY()`
    pub fn change_light_position(&mut self, index: i32, position: Point) {
        if index < 0 || index >= MAX_LIGHTS as i32 {
            return;
        }

        let light = &mut self.lights[index as usize];
        if light.is_invalid {
            return;
        }

        light.mark_changed();
        light.position.tile = position;
        self.update_lighting = true;
    }

    /// Change light offset (for smooth movement)
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ChangeLightOffset()`
    pub fn change_light_offset(&mut self, index: i32, offset: (i8, i8)) {
        if index < 0 || index >= MAX_LIGHTS as i32 {
            return;
        }

        let light = &mut self.lights[index as usize];
        if light.is_invalid {
            return;
        }

        light.position.offset = offset;
        self.update_lighting = true;
    }

    /// Change light position and radius
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ChangeLight()`
    pub fn change_light(&mut self, index: i32, position: Point, radius: u8) {
        if index < 0 || index >= MAX_LIGHTS as i32 {
            return;
        }

        let light = &mut self.lights[index as usize];
        if light.is_invalid {
            return;
        }

        light.mark_changed();
        light.position.tile = position;
        light.radius = radius;
        self.update_lighting = true;
    }

    /// Process all light sources
    ///
    /// **C++ Reference**: `Source/lighting.cpp:475` - `ProcessLightList()`
    ///
    /// # C++ Implementation
    /// ```cpp
    /// void ProcessLightList() {
    ///     if (!UpdateLighting) return;
    ///
    ///     // First pass: remove old lighting
    ///     for (int i = 0; i < ActiveLightCount; i++) {
    ///         Light &light = Lights[ActiveLights[i]];
    ///         if (light.isInvalid) {
    ///             DoUnLight(light.position.tile, light.radius);
    ///         }
    ///         if (light.hasChanged) {
    ///             DoUnLight(light.position.old, light.oldRadius);
    ///             light.hasChanged = false;
    ///         }
    ///     }
    ///
    ///     // Second pass: apply new lighting
    ///     for (int i = 0; i < ActiveLightCount; i++) {
    ///         const Light &light = Lights[ActiveLights[i]];
    ///         if (light.isInvalid) {
    ///             // Remove from active list
    ///             ActiveLightCount--;
    ///             swap(ActiveLights[i], ActiveLights[ActiveLightCount]);
    ///             i--;
    ///             continue;
    ///         }
    ///         DoLighting(light.position.tile, light.radius, light.position.offset);
    ///     }
    ///
    ///     UpdateLighting = false;
    /// }
    /// ```
    pub fn process_light_list(&mut self) {
        #[cfg(debug_assertions)]
        if self.disable_lighting {
            return;
        }

        if !self.update_lighting {
            return;
        }

        // First pass: collect information about lights that need unlit
        let mut unlight_ops: Vec<(Point, u8)> = Vec::new();
        let mut changed_indices: Vec<usize> = Vec::new();

        for i in 0..self.active_light_count {
            let light_idx = self.active_lights[i] as usize;
            let light = &self.lights[light_idx];

            if light.is_invalid {
                unlight_ops.push((light.position.tile, light.radius));
            }

            if light.has_changed {
                unlight_ops.push((light.position.old, light.old_radius));
                changed_indices.push(light_idx);
            }
        }

        // Apply unlight operations
        for (pos, radius) in unlight_ops {
            self.do_unlight(pos, radius);
        }

        // Clear changed flags
        for idx in changed_indices {
            self.lights[idx].has_changed = false;
        }

        // Second pass: collect lighting info, then apply
        let mut light_ops: Vec<(Point, u8, (i8, i8))> = Vec::new();
        let mut invalid_indices: Vec<usize> = Vec::new();

        let mut i = 0;
        while i < self.active_light_count {
            let light_idx = self.active_lights[i] as usize;
            let light = &self.lights[light_idx];

            if light.is_invalid {
                // Mark for removal from active list
                invalid_indices.push(i);
                i += 1;
                continue;
            }

            // Collect lighting operation
            light_ops.push((light.position.tile, light.radius, light.position.offset));
            i += 1;
        }

        // Remove invalid lights from active list (in reverse to preserve indices)
        for &idx in invalid_indices.iter().rev() {
            self.active_light_count -= 1;
            self.active_lights.swap(idx, self.active_light_count);
        }

        // Apply lighting operations
        for (pos, radius, offset) in light_ops {
            self.do_lighting(pos, radius, offset);
        }

        self.update_lighting = false;
    }

    /// Remove lighting from an area
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `DoUnLight()`
    fn do_unlight(&mut self, position: Point, radius: u8) {
        let radius = radius as i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = position.x + dx;
                let y = position.y + dy;

                // Bounds check
                if x < 0 || x >= 112 || y < 0 || y >= 112 {
                    continue;
                }

                // Reset to max darkness
                self.light_buffer[y as usize][x as usize] = LIGHTS_MAX;
            }
        }
    }

    /// Apply lighting to an area
    ///
    /// **C++ Reference**: `Source/lighting.cpp:127-189` - `DoLighting()`
    ///
    /// Uses light falloff tables and cone interpolations for accurate light rendering.
    /// The algorithm processes 4 quadrants around the light source.
    fn do_lighting(&mut self, mut position: Point, radius: u8, mut offset: (i8, i8)) {
        let radius_idx = (radius as usize).min(NUM_LIGHT_RADIUSES - 1);

        // Handle negative offsets by adjusting position
        if offset.0 < 0 {
            offset.0 += 8;
            position.x -= 1;
        }
        if offset.1 < 0 {
            offset.1 += 8;
            position.y -= 1;
        }

        let offset_x = offset.0 as usize;
        let offset_y = offset.1 as usize;

        // Calculate bounds
        let min_x = if position.x - 15 < 0 { (position.x + 1) as usize } else { 15 };
        let max_x = if position.x + 15 > 112 { (112 - position.x) as usize } else { 15 };
        let min_y = if position.y - 15 < 0 { (position.y + 1) as usize } else { 15 };
        let max_y = if position.y + 15 > 112 { (112 - position.y) as usize } else { 15 };

        // Set center light level
        if self.level_type.is_hellfire() {
            // Allow for dim lights in crypt and nest
            let center_x = position.x as usize;
            let center_y = position.y as usize;
            if center_x < 112 && center_y < 112 {
                let current = self.light_buffer[center_y][center_x];
                let falloff_center = self.light_falloffs[radius_idx][0];
                if current > falloff_center {
                    self.light_buffer[center_y][center_x] = falloff_center;
                }
            }
        } else {
            let center_x = position.x as usize;
            let center_y = position.y as usize;
            if center_x < 112 && center_y < 112 {
                self.light_buffer[center_y][center_x] = 0;
            }
        }

        // Process 4 quadrants
        for quadrant in 0..4 {
            let y_bound = if quadrant > 0 && quadrant < 3 { max_y } else { min_y };
            let x_bound = if quadrant < 2 { max_x } else { min_x };

            for y in 0..y_bound {
                for x in 1..x_bound {
                    // Get interpolated distance from cone table
                    let lookup_x = (x + if quadrant >= 2 { 0 } else { 0 }).min(15);
                    let lookup_y = (y + if quadrant == 1 || quadrant == 2 { 0 } else { 0 }).min(15);
                    let linear_distance = self.light_cone_interpolations[offset_x.min(7)][offset_y.min(7)][lookup_x][lookup_y];

                    if linear_distance >= 128 {
                        continue;
                    }

                    // Calculate target position with rotation
                    let (dx, dy) = match quadrant {
                        0 => (x as i32, y as i32),
                        1 => (y as i32, -(x as i32)),
                        2 => (-(x as i32), -(y as i32)),
                        3 => (-(y as i32), x as i32),
                        _ => unreachable!(),
                    };

                    let target_x = position.x + dx;
                    let target_y = position.y + dy;

                    // Bounds check
                    if target_x < 0 || target_x >= 112 || target_y < 0 || target_y >= 112 {
                        continue;
                    }

                    // Get light level from falloff table
                    let light_level = self.light_falloffs[radius_idx][linear_distance as usize];

                    // Apply minimum (brighter wins)
                    let tx = target_x as usize;
                    let ty = target_y as usize;
                    let current = self.light_buffer[ty][tx];
                    if light_level < current {
                        self.light_buffer[ty][tx] = light_level;
                    }
                }
            }
        }
    }

    /// Save pre-lighting buffer
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `SavePreLighting()`
    pub fn save_pre_lighting(&mut self) {
        self.pre_light = self.light_buffer;
    }

    /// Activate vision for a player
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ActivateVision()`
    pub fn activate_vision(&mut self, position: Point, radius: i32, id: usize) {
        if id >= MAX_VISION {
            return;
        }

        let vision = &mut self.vision_list[id];
        vision.position.tile = position;
        vision.radius = radius as u8;
        vision.is_invalid = false;
        vision.has_changed = false;
        self.vision_active[id] = true;
        self.update_vision = true;
    }

    /// Deactivate vision for a player
    pub fn deactivate_vision(&mut self, id: usize) {
        if id >= MAX_VISION {
            return;
        }

        self.vision_active[id] = false;
        self.update_vision = true;
    }

    /// Change vision radius
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ChangeVisionRadius()`
    pub fn change_vision_radius(&mut self, id: usize, radius: i32) {
        if id >= MAX_VISION {
            return;
        }

        let vision = &mut self.vision_list[id];
        vision.mark_changed();
        vision.radius = radius as u8;
        self.update_vision = true;
    }

    /// Change vision position
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `ChangeVisionXY()`
    pub fn change_vision_position(&mut self, id: usize, position: Point) {
        if id >= MAX_VISION {
            return;
        }

        let vision = &mut self.vision_list[id];
        vision.mark_changed();
        vision.position.tile = position;
        self.update_vision = true;
    }

    /// Process vision list
    ///
    /// **C++ Reference**: `Source/lighting.cpp:546` - `ProcessVisionList()`
    ///
    /// # C++ Implementation
    /// ```cpp
    /// void ProcessVisionList() {
    ///     if (!UpdateVision) return;
    ///
    ///     TransList = {};
    ///
    ///     // First pass: remove old vision
    ///     for (const Player &player : Players) {
    ///         const size_t id = player.getId();
    ///         if (!VisionActive[id]) continue;
    ///
    ///         Light &vision = VisionList[id];
    ///         if (!player.plractive || !player.isOnActiveLevel()) {
    ///             DoUnVision(vision.position.tile, vision.radius);
    ///             VisionActive[id] = false;
    ///             continue;
    ///         }
    ///
    ///         if (vision.hasChanged) {
    ///             DoUnVision(vision.position.old, vision.oldRadius);
    ///             vision.hasChanged = false;
    ///         }
    ///     }
    ///
    ///     // Second pass: apply new vision
    ///     for (const Player &player : Players) {
    ///         const size_t id = player.getId();
    ///         if (!VisionActive[id]) continue;
    ///
    ///         const Light &vision = VisionList[id];
    ///         DoVision(vision.position.tile, vision.radius, ...);
    ///     }
    ///
    ///     UpdateVision = false;
    /// }
    /// ```
    pub fn process_vision_list(&mut self, player_active: &[bool; MAX_VISION]) {
        if !self.update_vision {
            return;
        }

        // First pass: collect unvision operations
        let mut unvision_ops: Vec<(Point, u8)> = Vec::new();
        let mut deactivate_ids: Vec<usize> = Vec::new();
        let mut clear_changed_ids: Vec<usize> = Vec::new();

        for id in 0..MAX_VISION {
            if !self.vision_active[id] {
                continue;
            }

            let vision = &self.vision_list[id];

            // Check if player is inactive
            if !player_active[id] {
                unvision_ops.push((vision.position.tile, vision.radius));
                deactivate_ids.push(id);
                continue;
            }

            // Handle changed vision
            if vision.has_changed {
                unvision_ops.push((vision.position.old, vision.old_radius));
                clear_changed_ids.push(id);
            }
        }

        // Apply unvision operations
        for (pos, radius) in unvision_ops {
            self.do_unvision(pos, radius);
        }

        // Deactivate vision for inactive players
        for id in deactivate_ids {
            self.vision_active[id] = false;
        }

        // Clear changed flags
        for id in clear_changed_ids {
            self.vision_list[id].has_changed = false;
        }

        // Second pass: collect vision operations
        let mut vision_ops: Vec<(Point, u8, MapExplorationType, bool)> = Vec::new();

        for id in 0..MAX_VISION {
            if !self.vision_active[id] {
                continue;
            }

            let vision = &self.vision_list[id];
            vision_ops.push((vision.position.tile, vision.radius, MapExplorationType::ExploreSelf, id == 0));
        }

        // Apply vision operations
        for (pos, radius, exploration, is_main) in vision_ops {
            self.do_vision(pos, radius, exploration, is_main);
        }

        self.update_vision = false;
    }

    /// Remove vision from an area
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `DoUnVision()`
    fn do_unvision(&mut self, _position: Point, _radius: u8) {
        // Vision removal - typically handled by automap system.
        // Follow-up: clear the `visible` grid within `_position ± _radius+2`
        // (C++ DoUnVision clears the Visible|Lit flag bits).
    }

    /// Apply vision to an area
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `DoVision()`
    fn do_vision(&mut self, position: Point, radius: u8, _exploration: MapExplorationType, _is_main_player: bool) {
        // Faithful port of C++ DoVision(): cast the 23 VisionRays in all four
        // quadrants and mark every tile the rays reach as visible.
        //
        // Wall blocking (C++ `TileAllowsLight`, `TileProperties::BlockLight`)
        // needs the level's tile-property grid, which this module does not
        // own; the live path passes `|_| true` until the level layer is wired
        // in. `cast_vision_rays` is the exact C++ algorithm and is tested
        // against wall-blocking semantics directly.
        let rays = Self::cast_vision_rays(
            position,
            radius,
            |p| p.x >= 0 && p.x < 112 && p.y >= 0 && p.y < 112,
            |_| true,
        );
        for tile in rays {
            self.visible[tile.x as usize][tile.y as usize] = true;
        }
    }

    /// Cast C++ `DoVision` rays and return every tile they reach.
    ///
    /// **C++ Reference**: `Source/vision.cpp:51` - `DoVision(position, radius,
    /// markVisibleFn, markTransparentFn, passesLightFn, inBoundsFn)`.
    ///
    /// Ports the exact algorithm: 23 rays per quadrant (`VisionRays`), length
    /// `radius - RayLenAdj[j]`, mirrored over the four quadrants. A ray stops
    /// at the first tile that does not pass light (`tile_allows_light`), and
    /// rays crossing diagonally additionally require one of the two diagonally
    /// adjacent tiles to pass light (the corner case documented in vision.cpp).
    pub fn cast_vision_rays(
        position: Point,
        radius: u8,
        in_bounds: impl Fn(Point) -> bool,
        tile_allows_light: impl Fn(Point) -> bool,
    ) -> Vec<Point> {
        let mut visible = Vec::new();
        visible.push(position);

        const QUADRANTS: [(i8, i8); 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)];

        for (qx, qy) in QUADRANTS {
            for (j, ray) in VISION_RAYS.iter().enumerate() {
                let ray_len = radius as i32 - RAY_LEN_ADJ[j] as i32;
                for k in 0..ray_len {
                    let (rx, ry) = ray[k as usize];
                    let ray_point = Point::new(
                        position.x + (rx as i32) * (qx as i32),
                        position.y + (ry as i32) * (qy as i32),
                    );
                    if !in_bounds(ray_point) {
                        break;
                    }
                    // Diagonal rays must also clear one of the two diagonally
                    // adjacent tiles (C++ adjacent1/adjacent2 check).
                    if rx > 0 && ry > 0 {
                        let adjacent1 = Point::new(
                            ray_point.x - (qx as i32),
                            ray_point.y,
                        );
                        let adjacent2 = Point::new(
                            ray_point.x,
                            ray_point.y - (qy as i32),
                        );
                        if !(tile_allows_light(adjacent1) || tile_allows_light(adjacent2)) {
                            break;
                        }
                    }
                    visible.push(ray_point);
                    if !tile_allows_light(ray_point) {
                        break;
                    }
                }
            }
        }
        visible
    }

    /// Perform light color cycling (Hell level effect)
    ///
    /// **C++ Reference**: `Source/lighting.cpp` - `lighting_color_cycling()`
    ///
    /// # C++ Implementation
    /// ```cpp
    /// void lighting_color_cycling() {
    ///     for (auto &lightTable : LightTables) {
    ///         // shift elements between indexes 1-31 to left
    ///         std::rotate(lightTable.begin() + 1, lightTable.begin() + 2, lightTable.begin() + 32);
    ///     }
    /// }
    /// ```
    pub fn lighting_color_cycling(&mut self) {
        for table in &mut self.light_tables {
            // Rotate elements at indices 1-31 to the left
            // This creates the flickering fire effect in Hell
            table[1..32].rotate_left(1);
        }
    }

    /// Get light level at a position
    pub fn get_light_level(&self, x: i32, y: i32) -> u8 {
        if x < 0 || x >= 112 || y < 0 || y >= 112 {
            return LIGHTS_MAX;
        }
        self.light_buffer[y as usize][x as usize]
    }

    /// Get number of active lights
    pub fn active_count(&self) -> usize {
        self.active_light_count
    }

    /// Check if lighting update is needed
    pub fn needs_update(&self) -> bool {
        self.update_lighting
    }

    /// Check if vision update is needed
    pub fn needs_vision_update(&self) -> bool {
        self.update_vision
    }

    /// Toggle lighting (debug)
    #[cfg(debug_assertions)]
    pub fn toggle_lighting(&mut self) {
        self.disable_lighting = !self.disable_lighting;
    }
}

impl Default for LightManager {
    fn default() -> Self {
        Self::new()
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;

    /// C++ `TileAllowsLight` (lighting.cpp:93): a micro-tile with SOL
    /// `BlockLight` blocks vision/light rays; floor tiles pass.
    #[test]
    fn test_tile_allows_light_matches_cpp() {
        use crate::engine::dungeon::TileProperties;
        let sol = SolData {
            properties: vec![
                TileProperties::empty(),               // 0: floor
                TileProperties::BLOCK_LIGHT,           // 1: solid wall
                TileProperties::SOLID,                 // 2: solid but not BlockLight
                TileProperties::BLOCK_LIGHT | TileProperties::TRANSPARENT, // 3: see-through wall still blocks light
            ],
        };
        assert!(tile_allows_light(0, &sol), "floor allows light");
        assert!(!tile_allows_light(1, &sol), "BlockLight wall blocks");
        assert!(tile_allows_light(2, &sol), "SOLID alone does not block light");
        assert!(!tile_allows_light(3, &sol), "BlockLight bit wins");
        // Out-of-range SOL index defaults to empty (no BlockLight).
        assert!(tile_allows_light(999, &sol));
    }

    /// Vision rays stop at the first BlockLight tile (the C++ `passesLightFn`
    /// break in vision.cpp:113) — tiles beyond a wall are not reachable.
    #[test]
    fn test_cast_vision_rays_stop_behind_block_light_wall() {
        use crate::engine::dungeon::TileProperties;
        let sol = SolData {
            properties: vec![
                TileProperties::empty(),
                TileProperties::BLOCK_LIGHT,
            ],
        };
        // d_piece: piece 0 = floor everywhere except x == 24 which is a wall.
        let origin = Point::new(20, 20);
        let allows = |p: Point| -> bool {
            if p.x < 0 || p.y < 0 || p.x >= 112 || p.y >= 112 {
                return false;
            }
            let piece = if p.x == 24 && p.y == 20 { 1u16 } else { 0u16 };
            tile_allows_light(piece, &sol)
        };
        let visible = LightManager::cast_vision_rays(origin, 9, |p| p.x >= 0 && p.x < 112 && p.y >= 0 && p.y < 112, allows);
        assert!(visible.contains(&Point::new(23, 20)), "tile before wall visible");
        assert!(!visible.contains(&Point::new(25, 20)), "tile beyond wall occluded");
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_LIGHTS, 32);
        assert_eq!(MAX_VISION, 4);
        assert_eq!(NO_LIGHT, -1);
        assert_eq!(LIGHTS_MAX, 15);
        assert_eq!(LIGHT_TABLE_SIZE, 256);
        assert_eq!(NUM_LIGHTING_LEVELS, 16);
        assert_eq!(NUM_LIGHT_RADIUSES, 16);
        assert_eq!(MAX_FALLOFF_DISTANCE, 128);
    }

    #[test]
    fn test_dungeon_level_type() {
        assert_eq!(DungeonLevelType::default(), DungeonLevelType::Town);
        assert!(!DungeonLevelType::Town.is_hellfire());
        assert!(!DungeonLevelType::Cathedral.is_hellfire());
        assert!(!DungeonLevelType::Hell.is_hellfire());
        assert!(DungeonLevelType::Nest.is_hellfire());
        assert!(DungeonLevelType::Crypt.is_hellfire());
    }

    #[test]
    fn test_make_light_table_basic() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Cathedral);

        // Black (0) should always stay black
        assert_eq!(manager.light_tables[0][0], 0);
        assert_eq!(manager.light_tables[8][0], 0);
        assert_eq!(manager.light_tables[15][0], 0);

        // Last shade should be all black
        for i in 0..256 {
            assert_eq!(manager.light_tables[15][i], 0, "Index {} should be 0", i);
        }

        // Fully lit optimization should be enabled
        assert!(manager.has_fully_lit_optimization());
        assert!(manager.has_fully_dark_optimization());
    }

    #[test]
    fn test_make_light_table_hell() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Hell);

        // Fully lit optimization should be disabled for Hell
        assert!(!manager.has_fully_lit_optimization());
        assert!(manager.has_fully_dark_optimization());

        // Blood wall colors should be modified
        // Colors 1-16 and 16-31 should have special values
        assert!(manager.light_tables[0][1] > 0);
    }

    #[test]
    fn test_make_light_table_hellfire() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Nest);

        // Fully dark optimization should be disabled for Hellfire
        assert!(manager.has_fully_lit_optimization());
        assert!(!manager.has_fully_dark_optimization());

        // First 16 colors should be identity mapped (lava brightness)
        for i in 0..16 {
            assert_eq!(manager.light_tables[0][i], i as u8);
        }
    }

    #[test]
    fn test_light_falloffs_generation() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Cathedral);

        // Center should always be brightest (0)
        assert_eq!(manager.light_falloffs[0][0], 0);
        assert_eq!(manager.light_falloffs[15][0], 0);

        // Distance beyond max should be darkest (15)
        assert_eq!(manager.light_falloffs[0][127], 15);
        assert_eq!(manager.light_falloffs[0][100], 15);

        // Larger radius should have brighter at same distance
        let small_radius_falloff = manager.light_falloffs[2][16];
        let large_radius_falloff = manager.light_falloffs[10][16];
        assert!(large_radius_falloff <= small_radius_falloff);
    }

    #[test]
    fn test_light_cone_interpolations() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Cathedral);

        // Center of interpolation should be 0 or small
        assert!(manager.light_cone_interpolations[0][0][0][0] < 10);

        // Corners should have larger values
        let corner = manager.light_cone_interpolations[0][0][15][15];
        let center = manager.light_cone_interpolations[0][0][0][0];
        assert!(corner > center);
    }

    #[test]
    fn test_apply_lighting() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Cathedral);

        // At full light (0), colors should stay relatively unchanged (except at shade boundaries)
        let result_full = manager.apply_lighting(100, 0);
        assert!(result_full > 0); // Not black

        // At full dark (15), everything should be black
        let result_dark = manager.apply_lighting(100, 15);
        assert_eq!(result_dark, 0);
    }

    #[test]
    fn test_get_light_table() {
        let mut manager = LightManager::new();
        manager.make_light_table(DungeonLevelType::Cathedral);

        let table_0 = manager.get_light_table(0);
        let table_15 = manager.get_light_table(15);

        // Tables should be different
        assert_ne!(table_0[100], table_15[100]);

        // Out of bounds should clamp
        let table_clamped = manager.get_light_table(200);
        assert_eq!(table_clamped[0], table_15[0]); // Clamped to max
    }

    #[test]
    fn test_light_position() {
        let pos = LightPosition::new(Point::new(10, 20));
        assert_eq!(pos.tile, Point::new(10, 20));
        assert_eq!(pos.offset, (0, 0));
        assert_eq!(pos.old, Point::new(10, 20));

        let pos_offset = LightPosition::with_offset(Point::new(5, 5), (3, -2));
        assert_eq!(pos_offset.tile, Point::new(5, 5));
        assert_eq!(pos_offset.offset, (3, -2));
    }

    #[test]
    fn test_light_creation() {
        let light = Light::new(Point::new(15, 25), 8);
        assert_eq!(light.position.tile, Point::new(15, 25));
        assert_eq!(light.radius, 8);
        assert_eq!(light.old_radius, 8);
        assert!(!light.is_invalid);
        assert!(!light.has_changed);
    }

    #[test]
    fn test_light_mark_changed() {
        let mut light = Light::new(Point::new(10, 10), 5);
        light.position.tile = Point::new(12, 12);
        light.radius = 7;

        light.mark_changed();

        assert!(light.has_changed);
        assert_eq!(light.position.old, Point::new(12, 12));
        assert_eq!(light.old_radius, 7);
    }

    #[test]
    fn test_light_mark_invalid() {
        let mut light = Light::new(Point::new(10, 10), 5);
        assert!(!light.is_invalid);

        light.mark_invalid();
        assert!(light.is_invalid);
    }

    #[test]
    fn test_light_manager_creation() {
        let manager = LightManager::new();
        assert_eq!(manager.active_light_count, 0);
        assert!(!manager.update_lighting);
        assert!(!manager.update_vision);
    }

    #[test]
    fn test_light_manager_init() {
        let mut manager = LightManager::new();
        manager.active_light_count = 5;
        manager.update_lighting = true;

        manager.init();

        assert_eq!(manager.active_light_count, 0);
        assert!(!manager.update_lighting);
        assert!(!manager.update_vision);
    }

    #[test]
    fn test_add_light() {
        let mut manager = LightManager::new();
        manager.init();

        let idx = manager.add_light(Point::new(50, 50), 10);

        assert!(idx >= 0);
        assert_eq!(manager.active_light_count, 1);
        assert!(manager.update_lighting);
        assert!(!manager.lights[idx as usize].is_invalid);
    }

    #[test]
    fn test_add_multiple_lights() {
        let mut manager = LightManager::new();
        manager.init();

        let idx1 = manager.add_light(Point::new(10, 10), 5);
        let idx2 = manager.add_light(Point::new(20, 20), 8);
        let idx3 = manager.add_light(Point::new(30, 30), 12);

        assert!(idx1 >= 0);
        assert!(idx2 >= 0);
        assert!(idx3 >= 0);
        assert_eq!(manager.active_light_count, 3);
    }

    #[test]
    fn test_add_light_full() {
        let mut manager = LightManager::new();
        manager.init();

        // Fill all slots
        for i in 0..MAX_LIGHTS {
            let idx = manager.add_light(Point::new(i as i32, i as i32), 5);
            assert!(idx >= 0);
        }

        // Should fail
        let idx = manager.add_light(Point::new(100, 100), 5);
        assert_eq!(idx, NO_LIGHT);
    }

    #[test]
    fn test_remove_light() {
        let mut manager = LightManager::new();
        manager.init();

        let idx = manager.add_light(Point::new(50, 50), 10);
        assert!(!manager.lights[idx as usize].is_invalid);

        manager.remove_light(idx);
        assert!(manager.lights[idx as usize].is_invalid);
    }

    #[test]
    fn test_change_light_radius() {
        let mut manager = LightManager::new();
        manager.init();

        let idx = manager.add_light(Point::new(50, 50), 10);
        manager.change_light_radius(idx, 15);

        assert_eq!(manager.lights[idx as usize].radius, 15);
        assert!(manager.lights[idx as usize].has_changed);
    }

    #[test]
    fn test_change_light_position() {
        let mut manager = LightManager::new();
        manager.init();

        let idx = manager.add_light(Point::new(50, 50), 10);
        manager.change_light_position(idx, Point::new(60, 70));

        assert_eq!(manager.lights[idx as usize].position.tile, Point::new(60, 70));
        assert!(manager.lights[idx as usize].has_changed);
    }

    #[test]
    fn test_process_light_list_empty() {
        let mut manager = LightManager::new();
        manager.init();
        manager.update_lighting = true;

        manager.process_light_list();

        assert!(!manager.update_lighting);
    }

    #[test]
    fn test_process_light_list_with_lights() {
        let mut manager = LightManager::new();
        manager.init();

        manager.add_light(Point::new(50, 50), 5);
        manager.add_light(Point::new(60, 60), 8);

        manager.process_light_list();

        assert!(!manager.update_lighting);
        // Check that light was applied
        let level = manager.get_light_level(50, 50);
        assert!(level < LIGHTS_MAX, "Center should be lit");
    }

    #[test]
    fn test_process_light_list_removes_invalid() {
        let mut manager = LightManager::new();
        manager.init();

        let idx1 = manager.add_light(Point::new(50, 50), 5);
        let _idx2 = manager.add_light(Point::new(60, 60), 8);

        assert_eq!(manager.active_light_count, 2);

        manager.remove_light(idx1);
        manager.process_light_list();

        assert_eq!(manager.active_light_count, 1);
    }

    #[test]
    fn test_activate_vision() {
        let mut manager = LightManager::new();
        manager.init();

        manager.activate_vision(Point::new(50, 50), 10, 0);

        assert!(manager.vision_active[0]);
        assert_eq!(manager.vision_list[0].position.tile, Point::new(50, 50));
        assert_eq!(manager.vision_list[0].radius, 10);
        assert!(manager.update_vision);
    }

    #[test]
    fn test_deactivate_vision() {
        let mut manager = LightManager::new();
        manager.init();

        manager.activate_vision(Point::new(50, 50), 10, 0);
        assert!(manager.vision_active[0]);

        manager.deactivate_vision(0);
        assert!(!manager.vision_active[0]);
    }

    #[test]
    fn test_change_vision_radius() {
        let mut manager = LightManager::new();
        manager.init();

        manager.activate_vision(Point::new(50, 50), 10, 0);
        manager.change_vision_radius(0, 15);

        assert_eq!(manager.vision_list[0].radius, 15);
        assert!(manager.vision_list[0].has_changed);
    }

    #[test]
    fn test_change_vision_position() {
        let mut manager = LightManager::new();
        manager.init();

        manager.activate_vision(Point::new(50, 50), 10, 0);
        manager.change_vision_position(0, Point::new(60, 70));

        assert_eq!(manager.vision_list[0].position.tile, Point::new(60, 70));
        assert!(manager.vision_list[0].has_changed);
    }

    #[test]
    fn test_process_vision_list() {
        let mut manager = LightManager::new();
        manager.init();

        manager.activate_vision(Point::new(50, 50), 10, 0);
        let player_active = [true, false, false, false];

        manager.process_vision_list(&player_active);

        assert!(!manager.update_vision);
    }

    #[test]
    fn test_process_vision_deactivates_inactive_player() {
        let mut manager = LightManager::new();
        manager.init();

        manager.activate_vision(Point::new(50, 50), 10, 0);
        let player_active = [false, false, false, false]; // Player 0 inactive

        manager.process_vision_list(&player_active);

        assert!(!manager.vision_active[0]);
    }

    #[test]
    fn test_lighting_color_cycling() {
        let mut manager = LightManager::new();
        manager.init();

        // Set up some values
        for i in 0..32 {
            manager.light_tables[0][i] = i as u8;
        }

        let original_1 = manager.light_tables[0][1];
        let original_2 = manager.light_tables[0][2];

        manager.lighting_color_cycling();

        // After rotation, index 1 should have what was at index 2
        assert_eq!(manager.light_tables[0][1], original_2);
        // And index 31 should have what was at index 1
        assert_eq!(manager.light_tables[0][31], original_1);
    }

    #[test]
    fn test_get_light_level() {
        let mut manager = LightManager::new();
        manager.init();

        // Default should be max darkness
        assert_eq!(manager.get_light_level(50, 50), LIGHTS_MAX);

        // Out of bounds should return max
        assert_eq!(manager.get_light_level(-1, 50), LIGHTS_MAX);
        assert_eq!(manager.get_light_level(50, 200), LIGHTS_MAX);
    }

    #[test]
    fn test_save_pre_lighting() {
        let mut manager = LightManager::new();
        manager.init();

        // Modify light buffer
        manager.light_buffer[10][10] = 5;

        manager.save_pre_lighting();

        assert_eq!(manager.pre_light[10][10], 5);
    }

    #[test]
    fn test_map_exploration_type() {
        assert_eq!(MapExplorationType::default(), MapExplorationType::None);
        assert_eq!(MapExplorationType::ExploreSelf as i32, 1);
        assert_eq!(MapExplorationType::ExploreOthers as i32, 2);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_toggle_lighting() {
        let mut manager = LightManager::new();
        manager.init();

        assert!(!manager.disable_lighting);

        manager.toggle_lighting();
        assert!(manager.disable_lighting);

        manager.toggle_lighting();
        assert!(!manager.disable_lighting);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_disable_lighting_skips_processing() {
        let mut manager = LightManager::new();
        manager.init();
        manager.disable_lighting = true;

        manager.add_light(Point::new(50, 50), 10);
        manager.process_light_list();

        // update_lighting should still be true because processing was skipped
        assert!(manager.update_lighting);
    }
    // -----------------------------------------------------------------------
    // Vision raycasting (port of Source/vision.cpp DoVision)
    // -----------------------------------------------------------------------

    #[test]
    fn test_vision_rays_table_shape() {
        assert_eq!(VISION_RAYS.len(), 23);
        for row in VISION_RAYS.iter() {
            assert_eq!(row.len(), 15);
        }
        assert_eq!(RAY_LEN_ADJ.len(), 23);
    }

    #[test]
    fn test_cast_vision_rays_open_floor_marks_origin_and_axis() {
        let visible = LightManager::cast_vision_rays(
            Point::new(50, 50),
            8,
            |p| p.x >= 0 && p.x < 112 && p.y >= 0 && p.y < 112,
            |_| true,
        );
        assert!(visible.contains(&Point::new(50, 50)));
        // East axis ray (row 0 of VisionRays, quadrant (1,1)).
        assert!(visible.contains(&Point::new(58, 50)));
        // South axis ray (rel (0,1), quadrant (1,1)).
        assert!(visible.contains(&Point::new(50, 58)));
        // Distance is never exceeded: (50, 62) is beyond radius 8.
        assert!(!visible.contains(&Point::new(50, 62)));
    }

    #[test]
    fn test_cast_vision_rays_wall_blocks_ray() {
        let visible = LightManager::cast_vision_rays(
            Point::new(50, 50),
            8,
            |p| p.x >= 0 && p.x < 112 && p.y >= 0 && p.y < 112,
            |p| !(p.x == 53 && p.y == 50), // wall directly east
        );
        assert!(visible.contains(&Point::new(52, 50)), "tile before wall visible");
        assert!(visible.contains(&Point::new(53, 50)), "wall tile itself visible");
        assert!(!visible.contains(&Point::new(54, 50)), "tile beyond wall hidden");
        assert!(!visible.contains(&Point::new(58, 50)), "far east tile hidden");
    }

    #[test]
    fn test_cast_vision_rays_diagonal_corner_blocked() {
        // C++ vision.cpp corner case: 'x' at origin, walls '#' at (50,51) and
        // (51,50) must hide the diagonal tile '?' at (51,51).
        let observer = Point::new(50, 50);
        let wall = |p: Point| (p.x == 50 && p.y == 51) || (p.x == 51 && p.y == 50);
        let visible = LightManager::cast_vision_rays(observer, 8, |p| p.x >= 0 && p.x < 112 && p.y >= 0 && p.y < 112, |p| !wall(p));
        assert!(!visible.contains(&Point::new(51, 51)), "diagonal tile hidden by corner walls");

        // With one adjacent tile open, the diagonal passes (C++ `||`).
        let wall2 = |p: Point| (p.x == 51 && p.y == 50);
        let visible2 = LightManager::cast_vision_rays(observer, 8, |p| p.x >= 0 && p.x < 112 && p.y >= 0 && p.y < 112, |p| !wall2(p));
        assert!(visible2.contains(&Point::new(51, 51)), "diagonal tile visible with one open adjacent");
    }

    #[test]
    fn test_do_vision_marks_visible_grid() {
        let mut manager = LightManager::new();
        manager.init();
        manager.update_vision = true;
        manager.vision_active[0] = true;
        manager.vision_list[0] = Light::new(Point::new(50, 50), 8);

        manager.process_vision_list(&[true, false, false, false]);

        assert!(manager.visible[50][50], "origin visible");
        assert!(manager.visible[50][58], "south tile visible");
        assert!(!manager.visible[50][62], "beyond radius not visible");
    }


}
