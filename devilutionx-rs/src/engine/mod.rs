//! Engine module - handles rendering, audio, fonts, and low-level systems
//!
//! This module contains ported code from DevilutionX's C++ engine.

pub mod renderer;
pub mod audio;
pub mod assets;
pub mod font;
pub mod textures;
pub mod mpq;
pub mod archive; // Multi-MPQ ArchiveManager (load-order priority)
pub mod clx;
pub mod cel;  // CEL/CL2 decoder (M80)
pub mod cl2_sheet;  // CL2 multi-group (directional) sprite-sheet loader
pub mod explode;
pub mod pcx;
pub mod dungeon;
pub mod text_render;
pub mod window;  // SDL2 window management (M80)
pub mod resources;  // Resource manager (M80)
pub mod isometric;  // Isometric tile renderer (M80)
pub mod timing;  // Game loop timing (M80)
pub mod sprite_render;  // CLX/CEL sprite → SDL Texture bridge (R1)

// New ported modules from C++ engine
pub mod types;
pub mod surface;
pub mod clx_sprite;
pub mod clx_render;
pub mod dun_render;
pub mod palette;
pub mod animation;
pub mod random;
pub mod trn;
pub mod world_tile;
pub mod lighting;
pub mod circle;
pub mod ticks;
pub mod path;
pub mod load_clx;
pub mod demo_reader; // .dmo demo parser — Tier 0 of timedemo e2e harness

// Render submodules - ported from Source/engine/render/
pub mod primitive_render;
pub mod automap_render;
pub mod scrollrt;
pub mod light_render;
pub mod clx_render_ext;
pub mod encrypt;

// Resource loading and events
pub mod load_sprite;
pub mod events;

pub use renderer::Renderer;
pub use audio::AudioSystem;
pub use assets::AssetManager;
pub use font::{FontRenderer, PixelFont};
pub use textures::{
    TextureId, TextureRegion, TileDef, Tileset, SpriteDef,
    AssetManager as TextureManager, TextureCache, Palette as TexturePalette,
};
pub use mpq::{MpqArchive, MpqError, AssetManager as MpqAssetManager};
pub use archive::{ArchiveManager, ArchiveError};
pub use clx::{ClxSprite, ClxSpriteList, ClxSpriteSheet, DiabloPalette};
pub use pcx::PcxImage;
pub use dungeon::{
    DungeonType, DungeonLevelData, DunTemplate, MinData, SolData,
    MegaTile, LevelCelBlock, TileType, TileProperties, PaletteData,
    TileDecoder, TILE_WIDTH, TILE_HEIGHT, FRAME_WIDTH, FRAME_HEIGHT,
};
pub use text_render::{TextRenderer, GameFont, TextColor, TextRenderOptions, TextAlign};

// Re-export new types
pub use types::{Point, Size, Displacement, Rectangle, Direction};
pub use surface::{Surface, SurfaceRef, clip_rect, blit, blit_skip_color_index_zero, fill_rect};
pub use dun_render::{
    MaskType, Clip, DUN_FRAME_WIDTH, DUN_FRAME_HEIGHT,
    render_tile, render_tile_frame, render_square, render_transparent_square,
    render_left_triangle, render_right_triangle, render_left_trapezoid, render_right_trapezoid,
    draw_black_tile, get_dun_frame, get_dun_frame_offset,
};

// Re-export palette types
pub use palette::{
    Color, Palette, LightTable, TransparencyTable,
    PAL8_BLUE, PAL8_RED, PAL8_YELLOW, PAL8_ORANGE,
    PAL16_BEIGE, PAL16_BLUE, PAL16_YELLOW, PAL16_ORANGE, PAL16_RED, PAL16_GRAY,
};

// Re-export animation types
pub use animation::{AnimationInfo, AnimationDistributionFlags};

// Re-export random types
pub use random::{DiabloGenerator, Xoshiro128PlusPlus, SplitMix32, SplitMix64, generate_rnd, flip_coin, set_rnd_seed, generate_seed};

// Re-export trn types (ColorTransform also defined in palette, use trn module directly)
pub use trn::{TrnTable, TrnContext, EffectType, create_identity_trn, trn_from_data, combine_trn};

// Re-export world tile types
pub use world_tile::{
    WorldTilePosition, WorldTileDisplacement, WorldTileSize, WorldTileRectangle,
    ActorPosition, Displacement16, Displacement8,
};

// Re-export lighting types
pub use lighting::{
    Light, LightManager, Lightmap, LightingTable, AllLightingTables,
    MAX_LIGHTS, LIGHTS_MAX, NUM_LIGHTING_LEVELS, NO_LIGHT,
    generate_light_tables, build_tile_lightmap,
};

// Re-export circle types
pub use circle::Circle;

// Re-export ticks types
pub use ticks::{get_ticks, get_animation_frame, GameTime, FpsCounter, Timer};

// Re-export path types
pub use path::{
    find_path, find_closest_valid_position, get_path_direction,
    MAX_PATH_LENGTH_MONSTERS, MAX_PATH_LENGTH_PLAYER, PATH_DIRS,
    PATH_AXIS_ALIGNED_STEP_COST, PATH_DIAGONAL_STEP_COST,
};

// Re-export load_clx types
pub use load_clx::{
    parse_clx_data, load_clx_from_file, load_clx_from_memory,
    ClxLoadError, ClxFileInfo, ClxHeader, ClxFrameInfo,
};

// Re-export primitive_render types
pub use primitive_render::{
    TransparencyLookup,
    fill_rect as primitive_fill_rect, draw_horizontal_line, draw_vertical_line,
    unsafe_draw_horizontal_line, unsafe_draw_vertical_line,
    draw_half_transparent_horizontal_line, draw_half_transparent_vertical_line,
    draw_half_transparent_rect, draw_half_transparent_rect_colored,
    set_half_transparent_pixel, unsafe_draw_border_2px, draw_rect_outline, draw_line,
};

// Re-export automap_render types
pub use automap_render::{
    AutomapType, AutomapRenderer,
    draw_map_line_ns, draw_map_line_we, draw_map_line_ne, draw_map_line_se,
    draw_map_line_nw, draw_map_line_sw, draw_map_free_line,
};

// Re-export scrollrt types
pub use scrollrt::{
    TILE_WIDTH as SCROLL_TILE_WIDTH, TILE_HEIGHT as SCROLL_TILE_HEIGHT,
    ViewportGeometry, ScrollState, RenderOrderCalculator,
    get_offset_for_walking, shift_grid, rows_covered_by_panel,
    calc_tile_offset, tiles_in_view, calc_viewport_geometry,
    get_screen_position, screen_to_tile, tile_in_viewport,
};

// Re-export light_render types
pub use light_render::{
    LIGHT_TABLE_SIZE, MAXDUNX, MAXDUNY,
    LightRenderMap, TileLightInfo, TileLightGrid,
    build_lightmap_for_viewport, bleed_lightmap_up,
};

// Re-export clx_render_ext types
pub use clx_render_ext::{
    clx_draw_blended, clx_draw_with_lightmap, clx_draw_blended_with_lightmap,
    is_point_within_clx, clx_measure_solid_horizontal_bounds, clx_apply_trans,
};

// Re-export load_sprite types
pub use load_sprite::{
    SpriteLoadError, AssetLoader, FileSystemLoader, MemoryLoader,
    SpriteWidths, SpriteCache,
    load_cel, load_optional_cel, load_cl2, load_clx as load_clx_file,
    CEL_EXT, CL2_EXT, CLX_EXT,
};

// Re-export events types
pub use events::{
    ModState, MouseButton, KeyCode, GameEvent, EventWithMod, EventManager,
    is_printable_key, key_to_char,
};

// Re-export sprite_render types (CLX/CEL → SDL Texture bridge)
pub use sprite_render::{
    SpriteRenderError,
    clx_sprite_to_rgba, clx_sprite_to_texture, rgba_to_texture,
    solid_color_texture, palette_to_rgb768,
};
