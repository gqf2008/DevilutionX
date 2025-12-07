//! Vision System - Exact C++ Port (M43)
//!
//! This module provides exact port of DevilutionX vision/line-of-sight functionality.
//!
//! ## C++ References
//! - Source/vision.cpp: Vision ray casting implementation
//! - Source/vision.hpp: Vision interface
//!
//! ## Key Features
//! - Ray-based visibility calculation
//! - Uses Bresenham's line algorithm approximation
//! - Four-quadrant mirroring for full circle coverage
//! - Handles diagonal tile visibility edge cases

use crate::game::types::Point;

/// Maximum vision radius supported
pub const MAX_VISION_RADIUS: usize = 15;

/// Number of vision rays per quadrant
pub const NUM_VISION_RAYS: usize = 23;

/// Points per ray (maximum)
pub const POINTS_PER_RAY: usize = 15;

/// Displacement with i8 components for ray data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplacementI8 {
    pub delta_x: i8,
    pub delta_y: i8,
}

impl DisplacementI8 {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { delta_x: x, delta_y: y }
    }
}

/// Displacement with i32 components for quadrant mirroring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplacementI32 {
    pub delta_x: i32,
    pub delta_y: i32,
}

impl DisplacementI32 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { delta_x: x, delta_y: y }
    }
}

/// Vision ray displacement data
///
/// XY points of vision rays are cast to trace the visibility of the
/// surrounding environment. The table represents N rays of M points in
/// one quadrant (0°-90°) of a circle, so rays for other quadrants will
/// be created by mirroring. Zero points at the end will be trimmed and
/// ignored.
///
/// A similar table can be recreated using Bresenham's line drawing algorithm:
/// https://en.wikipedia.org/wiki/Bresenham's_line_algorithm
pub static VISION_RAYS: [[DisplacementI8; POINTS_PER_RAY]; NUM_VISION_RAYS] = [
    // Ray 0: Due East (0°)
    [d(1,0), d(2,0), d(3,0), d(4,0), d(5,0), d(6,0), d(7,0), d(8,0), d(9,0), d(10,0), d(11,0), d(12,0), d(13,0), d(14,0), d(15,0)],
    // Ray 1
    [d(1,0), d(2,0), d(3,0), d(4,0), d(5,0), d(6,0), d(7,0), d(8,1), d(9,1), d(10,1), d(11,1), d(12,1), d(13,1), d(14,1), d(15,1)],
    // Ray 2
    [d(1,0), d(2,0), d(3,0), d(4,1), d(5,1), d(6,1), d(7,1), d(8,1), d(9,1), d(10,1), d(11,1), d(12,2), d(13,2), d(14,2), d(15,2)],
    // Ray 3
    [d(1,0), d(2,0), d(3,1), d(4,1), d(5,1), d(6,1), d(7,1), d(8,2), d(9,2), d(10,2), d(11,2), d(12,2), d(13,3), d(14,3), d(15,3)],
    // Ray 4
    [d(1,0), d(2,1), d(3,1), d(4,1), d(5,1), d(6,2), d(7,2), d(8,2), d(9,3), d(10,3), d(11,3), d(12,3), d(13,4), d(14,4), d(0,0)],
    // Ray 5
    [d(1,0), d(2,1), d(3,1), d(4,1), d(5,2), d(6,2), d(7,3), d(8,3), d(9,3), d(10,4), d(11,4), d(12,4), d(13,5), d(14,5), d(0,0)],
    // Ray 6
    [d(1,0), d(2,1), d(3,1), d(4,2), d(5,2), d(6,3), d(7,3), d(8,3), d(9,4), d(10,4), d(11,5), d(12,5), d(13,6), d(14,6), d(0,0)],
    // Ray 7 (~30°)
    [d(1,1), d(2,1), d(3,2), d(4,2), d(5,3), d(6,3), d(7,4), d(8,4), d(9,5), d(10,5), d(11,6), d(12,6), d(13,7), d(0,0), d(0,0)],
    // Ray 8
    [d(1,1), d(2,1), d(3,2), d(4,2), d(5,3), d(6,4), d(7,4), d(8,5), d(9,6), d(10,6), d(11,7), d(12,7), d(12,8), d(13,8), d(0,0)],
    // Ray 9
    [d(1,1), d(2,2), d(3,2), d(4,3), d(5,4), d(6,5), d(7,5), d(8,6), d(9,7), d(10,7), d(10,8), d(11,8), d(12,9), d(0,0), d(0,0)],
    // Ray 10
    [d(1,1), d(2,2), d(3,3), d(4,4), d(5,5), d(6,5), d(7,6), d(8,7), d(9,8), d(10,9), d(11,9), d(11,10), d(0,0), d(0,0), d(0,0)],
    // Ray 11: 45° diagonal
    [d(1,1), d(2,2), d(3,3), d(4,4), d(5,5), d(6,6), d(7,7), d(8,8), d(9,9), d(10,10), d(11,11), d(0,0), d(0,0), d(0,0), d(0,0)],
    // Ray 12
    [d(1,1), d(2,2), d(3,3), d(4,4), d(5,5), d(5,6), d(6,7), d(7,8), d(8,9), d(9,10), d(9,11), d(10,11), d(0,0), d(0,0), d(0,0)],
    // Ray 13
    [d(1,1), d(2,2), d(2,3), d(3,4), d(4,5), d(5,6), d(5,7), d(6,8), d(7,9), d(7,10), d(8,10), d(8,11), d(9,12), d(0,0), d(0,0)],
    // Ray 14
    [d(1,1), d(1,2), d(2,3), d(2,4), d(3,5), d(4,6), d(4,7), d(5,8), d(6,9), d(6,10), d(7,11), d(7,12), d(8,12), d(8,13), d(0,0)],
    // Ray 15 (~60°)
    [d(1,1), d(1,2), d(2,3), d(2,4), d(3,5), d(3,6), d(4,7), d(4,8), d(5,9), d(5,10), d(6,11), d(6,12), d(7,13), d(0,0), d(0,0)],
    // Ray 16
    [d(0,1), d(1,2), d(1,3), d(2,4), d(2,5), d(3,6), d(3,7), d(3,8), d(4,9), d(4,10), d(5,11), d(5,12), d(6,13), d(6,14), d(0,0)],
    // Ray 17
    [d(0,1), d(1,2), d(1,3), d(1,4), d(2,5), d(2,6), d(3,7), d(3,8), d(3,9), d(4,10), d(4,11), d(4,12), d(5,13), d(5,14), d(0,0)],
    // Ray 18
    [d(0,1), d(1,2), d(1,3), d(1,4), d(1,5), d(2,6), d(2,7), d(2,8), d(3,9), d(3,10), d(3,11), d(3,12), d(4,13), d(4,14), d(0,0)],
    // Ray 19
    [d(0,1), d(0,2), d(1,3), d(1,4), d(1,5), d(1,6), d(1,7), d(2,8), d(2,9), d(2,10), d(2,11), d(2,12), d(3,13), d(3,14), d(3,15)],
    // Ray 20
    [d(0,1), d(0,2), d(0,3), d(1,4), d(1,5), d(1,6), d(1,7), d(1,8), d(1,9), d(1,10), d(1,11), d(2,12), d(2,13), d(2,14), d(2,15)],
    // Ray 21
    [d(0,1), d(0,2), d(0,3), d(0,4), d(0,5), d(0,6), d(0,7), d(1,8), d(1,9), d(1,10), d(1,11), d(1,12), d(1,13), d(1,14), d(1,15)],
    // Ray 22: Due South (90°)
    [d(0,1), d(0,2), d(0,3), d(0,4), d(0,5), d(0,6), d(0,7), d(0,8), d(0,9), d(0,10), d(0,11), d(0,12), d(0,13), d(0,14), d(0,15)],
];

/// Ray length adjustments for accurate circle
/// Ensures all rays lie on an accurate circle by trimming corner rays
pub static RAY_LEN_ADJ: [u8; NUM_VISION_RAYS] = [
    0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 4, 3, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0
];

/// Four quadrant directions for ray mirroring
pub static QUADRANTS: [DisplacementI32; 4] = [
    DisplacementI32 { delta_x: 1, delta_y: 1 },    // Quadrant 1: +X, +Y
    DisplacementI32 { delta_x: -1, delta_y: 1 },   // Quadrant 2: -X, +Y
    DisplacementI32 { delta_x: 1, delta_y: -1 },   // Quadrant 3: +X, -Y
    DisplacementI32 { delta_x: -1, delta_y: -1 },  // Quadrant 4: -X, -Y
];

/// Helper function to create DisplacementI8
const fn d(x: i8, y: i8) -> DisplacementI8 {
    DisplacementI8 { delta_x: x, delta_y: y }
}

/// Perform vision calculation from a position
///
/// Exact port of DoVision from vision.cpp
///
/// # Arguments
/// * `position` - Center position for vision calculation
/// * `radius` - Vision radius (0-15)
/// * `mark_visible` - Callback to mark a tile as visible
/// * `mark_transparent` - Callback to mark a tile as transparent (light passes through)
/// * `passes_light` - Callback to check if light passes through a tile
/// * `in_bounds` - Callback to check if position is within dungeon bounds
pub fn do_vision<F1, F2, F3, F4>(
    position: Point,
    radius: u8,
    mut mark_visible: F1,
    mut mark_transparent: F2,
    passes_light: F3,
    in_bounds: F4,
) where
    F1: FnMut(Point),
    F2: FnMut(Point),
    F3: Fn(Point) -> bool,
    F4: Fn(Point) -> bool,
{
    // Mark center position as visible
    mark_visible(position);

    // Loop over all four quadrants
    for quadrant in &QUADRANTS {
        // Cast each ray in the quadrant
        for j in 0..NUM_VISION_RAYS {
            // Adjust ray length for accurate circle
            let ray_len = (radius as usize).saturating_sub(RAY_LEN_ADJ[j] as usize);

            // Trace along the ray
            for k in 0..ray_len {
                let rel_ray_point = VISION_RAYS[j][k];

                // Skip zero points (end of ray data)
                if rel_ray_point.delta_x == 0 && rel_ray_point.delta_y == 0 {
                    break;
                }

                // Calculate the next point on the ray in this quadrant
                let ray_point = Point {
                    x: position.x + (rel_ray_point.delta_x as i32) * quadrant.delta_x,
                    y: position.y + (rel_ray_point.delta_y as i32) * quadrant.delta_y,
                };

                // Check bounds
                if !in_bounds(ray_point) {
                    break;
                }

                // Handle diagonal visibility edge case
                //
                // We've cast an approximated ray on an integer 2D grid, so we need
                // to check if a ray can pass through diagonally adjacent tiles.
                //
                // For example, consider this case:
                //
                //        #?
                //       ↗ #
                //     x
                //
                // The ray is cast from the observer 'x', and reaches the '?', but
                // diagonally adjacent tiles '#' do not pass the light, so the '?'
                // should not be visible for the 2D observer.
                //
                // The trick is to perform two additional visibility checks for the
                // diagonally adjacent tiles, but only for rays that are not parallel
                // to the X or Y coordinate lines.
                //
                if rel_ray_point.delta_x > 0 && rel_ray_point.delta_y > 0 {
                    let adjacent1 = Point {
                        x: ray_point.x - quadrant.delta_x,
                        y: ray_point.y,
                    };
                    let adjacent2 = Point {
                        x: ray_point.x,
                        y: ray_point.y - quadrant.delta_y,
                    };

                    // If diagonally adjacent tiles do not pass light, stop this ray
                    let passes = passes_light(adjacent1) || passes_light(adjacent2);
                    if !passes {
                        break;
                    }
                }

                // Mark this tile as visible
                mark_visible(ray_point);

                // If this tile doesn't pass light, stop the ray
                if !passes_light(ray_point) {
                    break;
                }

                // Mark as transparent (for lighting calculations)
                mark_transparent(ray_point);
            }
        }
    }
}

/// Simple visibility check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityResult {
    /// Tile is visible
    Visible,
    /// Tile is not visible (blocked)
    Hidden,
    /// Tile is out of range
    OutOfRange,
}

/// Check if a specific tile is visible from a position
///
/// Simplified version for quick point-to-point visibility checks.
pub fn is_tile_visible<F1, F2>(
    from: Point,
    to: Point,
    radius: u8,
    passes_light: F1,
    in_bounds: F2,
) -> VisibilityResult
where
    F1: Fn(Point) -> bool,
    F2: Fn(Point) -> bool,
{
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();

    // Quick distance check
    let dist_sq = dx * dx + dy * dy;
    let radius_sq = (radius as i32) * (radius as i32);
    if dist_sq > radius_sq {
        return VisibilityResult::OutOfRange;
    }

    // Use Bresenham's line algorithm for direct line-of-sight check
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = from.x;
    let mut y = from.y;

    while x != to.x || y != to.y {
        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }

        let current = Point { x, y };

        // Check bounds
        if !in_bounds(current) {
            return VisibilityResult::Hidden;
        }

        // Check if we've reached the destination
        if x == to.x && y == to.y {
            break;
        }

        // Check if light passes through
        if !passes_light(current) {
            return VisibilityResult::Hidden;
        }
    }

    VisibilityResult::Visible
}

/// Vision system state for a level
#[derive(Debug, Clone)]
pub struct VisionState {
    /// Visible tiles (true = visible)
    pub visible: Vec<Vec<bool>>,
    /// Explored tiles (true = explored, shows on automap)
    pub explored: Vec<Vec<bool>>,
    /// Width of the dungeon
    pub width: usize,
    /// Height of the dungeon
    pub height: usize,
}

impl VisionState {
    /// Create new vision state for a dungeon
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            visible: vec![vec![false; height]; width],
            explored: vec![vec![false; height]; width],
            width,
            height,
        }
    }

    /// Clear all visibility (but keep explored)
    pub fn clear_visible(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.visible[x][y] = false;
            }
        }
    }

    /// Reset all state
    pub fn reset(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.visible[x][y] = false;
                self.explored[x][y] = false;
            }
        }
    }

    /// Check if position is in bounds
    pub fn in_bounds(&self, p: Point) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height
    }

    /// Mark tile as visible
    pub fn mark_visible(&mut self, p: Point) {
        if self.in_bounds(p) {
            self.visible[p.x as usize][p.y as usize] = true;
            self.explored[p.x as usize][p.y as usize] = true;
        }
    }

    /// Check if tile is visible
    pub fn is_visible(&self, p: Point) -> bool {
        if self.in_bounds(p) {
            self.visible[p.x as usize][p.y as usize]
        } else {
            false
        }
    }

    /// Check if tile is explored
    pub fn is_explored(&self, p: Point) -> bool {
        if self.in_bounds(p) {
            self.explored[p.x as usize][p.y as usize]
        } else {
            false
        }
    }

    /// Update vision from a position
    pub fn update_vision<F>(&mut self, position: Point, radius: u8, passes_light: F)
    where
        F: Fn(Point) -> bool,
    {
        // Clear current visibility
        self.clear_visible();

        // Calculate new visibility
        let width = self.width;
        let height = self.height;

        do_vision(
            position,
            radius,
            |p| {
                if p.x >= 0 && p.y >= 0 && (p.x as usize) < width && (p.y as usize) < height {
                    self.visible[p.x as usize][p.y as usize] = true;
                    self.explored[p.x as usize][p.y as usize] = true;
                }
            },
            |_p| { /* Mark transparent - not used for basic vision */ },
            passes_light,
            |p| p.x >= 0 && p.y >= 0 && (p.x as usize) < width && (p.y as usize) < height,
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_rays_table_size() {
        assert_eq!(VISION_RAYS.len(), NUM_VISION_RAYS);
        assert_eq!(VISION_RAYS[0].len(), POINTS_PER_RAY);
        assert_eq!(RAY_LEN_ADJ.len(), NUM_VISION_RAYS);
    }

    #[test]
    fn test_ray_0_is_horizontal() {
        // Ray 0 should be purely horizontal (due east)
        for point in &VISION_RAYS[0] {
            assert_eq!(point.delta_y, 0);
        }
    }

    #[test]
    fn test_ray_22_is_vertical() {
        // Ray 22 should be purely vertical (due south)
        for point in &VISION_RAYS[22] {
            assert_eq!(point.delta_x, 0);
        }
    }

    #[test]
    fn test_ray_11_is_diagonal() {
        // Ray 11 should be 45-degree diagonal
        for i in 0..11 {
            let point = VISION_RAYS[11][i];
            if point.delta_x == 0 && point.delta_y == 0 {
                break;
            }
            assert_eq!(point.delta_x, point.delta_y);
        }
    }

    #[test]
    fn test_quadrants() {
        assert_eq!(QUADRANTS.len(), 4);

        // Each quadrant should have unique sign combination
        assert_eq!(QUADRANTS[0].delta_x, 1);
        assert_eq!(QUADRANTS[0].delta_y, 1);
        assert_eq!(QUADRANTS[1].delta_x, -1);
        assert_eq!(QUADRANTS[1].delta_y, 1);
        assert_eq!(QUADRANTS[2].delta_x, 1);
        assert_eq!(QUADRANTS[2].delta_y, -1);
        assert_eq!(QUADRANTS[3].delta_x, -1);
        assert_eq!(QUADRANTS[3].delta_y, -1);
    }

    #[test]
    fn test_do_vision_center_always_visible() {
        let center = Point { x: 10, y: 10 };
        let mut center_marked = false;

        do_vision(
            center,
            5,
            |p| {
                if p.x == center.x && p.y == center.y {
                    center_marked = true;
                }
            },
            |_| {},
            |_| true,
            |p| p.x >= 0 && p.y >= 0 && p.x < 20 && p.y < 20,
        );

        assert!(center_marked, "Center should always be marked visible");
    }

    #[test]
    fn test_do_vision_blocked_by_wall() {
        let center = Point { x: 10, y: 10 };
        let wall = Point { x: 12, y: 10 };
        let beyond_wall = Point { x: 14, y: 10 };

        let mut beyond_visible = false;

        do_vision(
            center,
            5,
            |p| {
                if p.x == beyond_wall.x && p.y == beyond_wall.y {
                    beyond_visible = true;
                }
            },
            |_| {},
            |p| !(p.x == wall.x && p.y == wall.y),  // Wall blocks light
            |p| p.x >= 0 && p.y >= 0 && p.x < 20 && p.y < 20,
        );

        assert!(!beyond_visible, "Tile beyond wall should not be visible");
    }

    #[test]
    fn test_vision_state_new() {
        let state = VisionState::new(50, 50);
        assert_eq!(state.width, 50);
        assert_eq!(state.height, 50);
        assert!(!state.is_visible(Point { x: 10, y: 10 }));
        assert!(!state.is_explored(Point { x: 10, y: 10 }));
    }

    #[test]
    fn test_vision_state_mark_visible() {
        let mut state = VisionState::new(50, 50);
        let p = Point { x: 10, y: 10 };

        state.mark_visible(p);

        assert!(state.is_visible(p));
        assert!(state.is_explored(p));
    }

    #[test]
    fn test_vision_state_clear_visible() {
        let mut state = VisionState::new(50, 50);
        let p = Point { x: 10, y: 10 };

        state.mark_visible(p);
        state.clear_visible();

        assert!(!state.is_visible(p));
        assert!(state.is_explored(p), "Explored should persist after clear_visible");
    }

    #[test]
    fn test_is_tile_visible_direct_line() {
        let from = Point { x: 0, y: 0 };
        let to = Point { x: 5, y: 0 };

        let result = is_tile_visible(
            from, to, 10,
            |_| true,  // All tiles pass light
            |p| p.x >= 0 && p.y >= 0 && p.x < 20 && p.y < 20,
        );

        assert_eq!(result, VisibilityResult::Visible);
    }

    #[test]
    fn test_is_tile_visible_out_of_range() {
        let from = Point { x: 0, y: 0 };
        let to = Point { x: 20, y: 0 };  // Beyond radius 10

        let result = is_tile_visible(
            from, to, 10,
            |_| true,
            |p| p.x >= 0 && p.y >= 0 && p.x < 30 && p.y < 30,
        );

        assert_eq!(result, VisibilityResult::OutOfRange);
    }

    #[test]
    fn test_is_tile_visible_blocked() {
        let from = Point { x: 0, y: 0 };
        let wall = Point { x: 3, y: 0 };
        let to = Point { x: 5, y: 0 };

        let result = is_tile_visible(
            from, to, 10,
            |p| !(p.x == wall.x && p.y == wall.y),  // Wall at (3,0)
            |p| p.x >= 0 && p.y >= 0 && p.x < 20 && p.y < 20,
        );

        assert_eq!(result, VisibilityResult::Hidden);
    }
}
