//! Rendering subsystem - ported from Source/engine/render/
//!
//! This module contains all rendering-related functionality from DevilutionX C++ engine.

// Core rendering modules - matching C++ structure
pub mod scrollrt;        // Source/engine/render/scrollrt.cpp
pub mod dun_render;      // Source/engine/render/dun_render.cpp
pub mod light_render;    // Source/engine/render/light_render.cpp
pub mod automap_render;  // Source/engine/render/automap_render.cpp
pub mod primitive_render; // Source/engine/render/primitive_render.cpp
pub mod clx_render;      // Source/engine/render/clx_render.cpp
pub mod text_render;     // Source/engine/render/text_render.cpp
pub mod blit_impl;       // Source/engine/render/blit_impl.hpp

// Re-export commonly used types and functions

// From scrollrt
pub use scrollrt::{
    ViewportState, get_direction,
    get_offset_for_walking, clear_cursor, shift_grid,
    rows_covered_by_panel, calc_tile_offset, tiles_in_view,
    calc_viewport_geometry, get_screen_position, clear_screen_buffer,
    enable_frame_count, scrollrt_draw_game_screen, draw_and_blit,
    auto_map_show_items, set_auto_map_show_items, frame_flag, set_frame_flag,
    in_dungeon_bounds, is_left_panel_open, is_right_panel_open, can_panels_cover_view,
    clx_draw_light, clx_draw_light_blended,
    RIGHT_FRAME_DISPLACEMENT, SIDE_PANEL_SIZE, INFO_BOX_RECT,
};

#[cfg(debug_assertions)]
pub use scrollrt::scroll_view;

// From dun_render
pub use dun_render::{
    TILE_WIDTH, TILE_HEIGHT, DUN_FRAME_WIDTH, DUN_FRAME_HEIGHT, DUN_FRAME_TRIANGLE_HEIGHT,
    REENCODED_TRIANGLE_FRAME_SIZE,
    TileType, LevelCelBlock, MaskType, Clip,
    render_tile_frame, render_tile, render_tile_foliage,
    render_square, render_transparent_square,
    render_left_triangle, render_right_triangle,
    get_dun_frame, get_dun_frame_offset, get_dun_frame_foliage,
    world_draw_black_tile, draw_black_tile,
};

// From light_render
pub use light_render::{
    LIGHT_TABLE_SIZE, LIGHTS_MAX, NUM_LIGHTING_LEVELS, MAXDUNX, MAXDUNY,
    Lightmap, OwnedLightmap, BleedUpLightmap, BleedUpOwnedLightmap,
    TileLightInfo, TileLightGrid,
    lightmap_build, lightmap_bleed_up,
};

// From automap_render
pub use automap_render::{
    AutomapType, get_automap_type, set_automap_type,
    get_minimap_rect, set_minimap_rect, set_map_pixel,
    draw_map_line_ns, draw_map_line_we, draw_map_line_ne, draw_map_line_se,
    draw_map_line_nw, draw_map_line_sw,
    draw_map_line_steep_ne, draw_map_line_steep_se,
    draw_map_line_steep_nw, draw_map_line_steep_sw,
    draw_map_free_line,
};

// From primitive_render
pub use primitive_render::{
    TransparencyLookup,
    get_palette_transparency_lookup, set_palette_transparency_lookup,
    fill_rect as primitive_fill_rect, draw_horizontal_line, draw_vertical_line,
    unsafe_draw_horizontal_line, unsafe_draw_vertical_line,
    draw_half_transparent_horizontal_line, draw_half_transparent_vertical_line,
    draw_half_transparent_horizontal_line_with_lookup, draw_half_transparent_vertical_line_with_lookup,
    draw_half_transparent_rect, draw_half_transparent_rect_colored,
    draw_half_transparent_rect_with_lookup, draw_half_transparent_rect_colored_with_lookup,
    set_half_transparent_pixel, set_half_transparent_pixel_with_lookup,
    unsafe_draw_border_2px, draw_rect_outline, draw_line,
};

// From clx_render
pub use clx_render::{
    clx_draw, clx_draw_trn, clx_draw_outline,
    clx_draw_outline_skip_color_zero, clx_apply_trans_list, clx_apply_trans_sheet,
    clx_draw_blended_trn, clx_draw_blended, clx_draw_with_lightmap,
    clx_draw_blended_with_lightmap, is_point_within_clx,
    clx_measure_solid_horizontal_bounds, clear_clx_draw_cache,
};

// From text_render
pub use text_render::{
    TextRenderer, GameFont, TextColor, TextRenderOptions, TextAlign,
};

// From blit_impl
pub use blit_impl::{
    // Direct blit
    blit_fill_direct, blit_pixels_direct, BlitDirect,
    // Blit with color map
    blit_fill_with_map, blit_pixels_with_map, BlitWithMap,
    // Blit with lightmap
    blit_fill_with_lightmap, blit_pixels_with_lightmap, BlitWithLightmap,
    // Blended blit
    blit_fill_blended, blit_pixels_blended, BlitBlended,
    // Blended blit with color map
    blit_pixels_blended_with_map, BlitBlendedWithMap,
    // Blended blit with lightmap
    blit_fill_blended_with_lightmap, blit_pixels_blended_with_lightmap, BlitBlendedWithLightmap,
    // Trait
    Blit,
};
