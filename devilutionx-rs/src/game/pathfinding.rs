/// A* Pathfinding Algorithm for monster navigation
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use super::types::Point;

/// Node for A* algorithm
#[derive(Clone, Eq, PartialEq)]
struct PathNode {
    pos: Point,
    g_cost: i32,  // Cost from start
    f_cost: i32,  // g_cost + heuristic
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* Pathfinder
pub struct Pathfinder {
    /// Maximum path length to search
    max_search_depth: usize,
    /// Cache of recently computed paths
    path_cache: HashMap<(Point, Point), Vec<Point>>,
    /// Cache timeout (number of uses before invalidation)
    cache_hits: HashMap<(Point, Point), u32>,
}

impl Default for Pathfinder {
    fn default() -> Self {
        Self::new()
    }
}

impl Pathfinder {
    pub fn new() -> Self {
        Self {
            max_search_depth: 50,
            path_cache: HashMap::new(),
            cache_hits: HashMap::new(),
        }
    }

    /// Clear path cache
    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
        self.cache_hits.clear();
    }

    /// Manhattan distance heuristic
    fn heuristic(a: Point, b: Point) -> i32 {
        (a.x - b.x).abs() + (a.y - b.y).abs()
    }

    /// Diagonal distance (allows 8-directional movement)
    fn diagonal_distance(a: Point, b: Point) -> i32 {
        let dx = (a.x - b.x).abs();
        let dy = (a.y - b.y).abs();
        // D * (dx + dy) + (D2 - 2*D) * min(dx, dy)
        // With D=10, D2=14 for diagonal
        10 * (dx + dy) + (-6) * dx.min(dy)
    }

    /// Find path from start to goal using A*
    /// Returns list of points to follow, or empty vec if no path found
    pub fn find_path<F>(
        &mut self,
        start: Point,
        goal: Point,
        is_walkable: F,
    ) -> Vec<Point>
    where
        F: Fn(i32, i32) -> bool,
    {
        // Check if we're already at goal
        if start == goal {
            return vec![];
        }

        // Check cache first
        let cache_key = (start, goal);
        if let Some(cached) = self.path_cache.get(&cache_key) {
            if let Some(hits) = self.cache_hits.get_mut(&cache_key) {
                *hits += 1;
                if *hits < 10 {
                    return cached.clone();
                }
            }
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<Point, Point> = HashMap::new();
        let mut g_score: HashMap<Point, i32> = HashMap::new();
        let mut closed_set: HashSet<Point> = HashSet::new();

        g_score.insert(start, 0);
        open_set.push(PathNode {
            pos: start,
            g_cost: 0,
            f_cost: Self::diagonal_distance(start, goal),
        });

        // 8 directions for movement
        let directions = [
            (0, -1),  // N
            (1, -1),  // NE
            (1, 0),   // E
            (1, 1),   // SE
            (0, 1),   // S
            (-1, 1),  // SW
            (-1, 0),  // W
            (-1, -1), // NW
        ];

        let mut iterations = 0;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > self.max_search_depth * 20 {
                // Exceeded search limit
                break;
            }

            if current.pos == goal {
                // Reconstruct path
                let path = self.reconstruct_path(&came_from, goal);

                // Cache the result
                self.path_cache.insert(cache_key, path.clone());
                self.cache_hits.insert(cache_key, 0);

                return path;
            }

            if closed_set.contains(&current.pos) {
                continue;
            }
            closed_set.insert(current.pos);

            // Check all neighbors
            for (dx, dy) in &directions {
                let nx = current.pos.x + dx;
                let ny = current.pos.y + dy;
                let neighbor = Point::new(nx, ny);

                if closed_set.contains(&neighbor) {
                    continue;
                }

                // Check if walkable
                if !is_walkable(nx, ny) {
                    continue;
                }

                // Diagonal movement cost is higher
                let move_cost = if *dx != 0 && *dy != 0 {
                    14 // sqrt(2) * 10 ≈ 14
                } else {
                    10
                };

                // For diagonal movement, check if we can cut corner
                if *dx != 0 && *dy != 0 {
                    let corner1_ok = is_walkable(current.pos.x + dx, current.pos.y);
                    let corner2_ok = is_walkable(current.pos.x, current.pos.y + dy);
                    if !corner1_ok || !corner2_ok {
                        continue; // Can't cut corners
                    }
                }

                let tentative_g = g_score.get(&current.pos).unwrap_or(&i32::MAX)
                    .saturating_add(move_cost);

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                    came_from.insert(neighbor, current.pos);
                    g_score.insert(neighbor, tentative_g);

                    let f_cost = tentative_g + Self::diagonal_distance(neighbor, goal);

                    open_set.push(PathNode {
                        pos: neighbor,
                        g_cost: tentative_g,
                        f_cost,
                    });
                }
            }
        }

        // No path found - try to get as close as possible
        self.find_closest_reachable(start, goal, is_walkable, &closed_set)
    }

    /// Reconstruct path from came_from map
    fn reconstruct_path(&self, came_from: &HashMap<Point, Point>, goal: Point) -> Vec<Point> {
        let mut path = vec![goal];
        let mut current = goal;

        while let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        // Remove starting position
        if !path.is_empty() {
            path.remove(0);
        }
        path
    }

    /// When no path to goal, find closest reachable point
    fn find_closest_reachable<F>(
        &self,
        _start: Point,
        goal: Point,
        _is_walkable: F,
        explored: &HashSet<Point>,
    ) -> Vec<Point>
    where
        F: Fn(i32, i32) -> bool,
    {
        // Find the explored node closest to goal
        let mut best_point = None;
        let mut best_dist = i32::MAX;

        for point in explored {
            let dist = Self::heuristic(*point, goal);
            if dist < best_dist {
                best_dist = dist;
                best_point = Some(*point);
            }
        }

        if let Some(point) = best_point {
            if point != goal {
                // Return single step toward best point
                return vec![point];
            }
        }

        vec![]
    }

    /// Simple line-of-sight check using Bresenham
    pub fn has_line_of_sight<F>(start: Point, end: Point, is_transparent: F) -> bool
    where
        F: Fn(i32, i32) -> bool,
    {
        let mut x = start.x;
        let mut y = start.y;
        let dx = (end.x - start.x).abs();
        let dy = (end.y - start.y).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };
        let mut err = dx - dy;

        while x != end.x || y != end.y {
            if !is_transparent(x, y) && (x != start.x || y != start.y) {
                return false;
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
}

/// Smooth path by removing unnecessary waypoints
pub fn smooth_path<F>(path: &[Point], is_walkable: F) -> Vec<Point>
where
    F: Fn(i32, i32) -> bool,
{
    if path.len() <= 2 {
        return path.to_vec();
    }

    let mut result = vec![path[0]];
    let mut i = 0;

    while i < path.len() - 1 {
        let mut furthest = i + 1;

        // Find furthest point we can see directly
        for j in (i + 2)..path.len() {
            if Pathfinder::has_line_of_sight(path[i], path[j], &is_walkable) {
                furthest = j;
            } else {
                break;
            }
        }

        result.push(path[furthest]);
        i = furthest;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pathfinding() {
        let mut pf = Pathfinder::new();

        // Simple open area
        let path = pf.find_path(
            Point::new(0, 0),
            Point::new(5, 5),
            |_, _| true,
        );

        assert!(!path.is_empty());
        assert_eq!(path.last(), Some(&Point::new(5, 5)));
    }

    #[test]
    fn test_obstacle_avoidance() {
        let mut pf = Pathfinder::new();

        // Wall at x=2
        let path = pf.find_path(
            Point::new(0, 2),
            Point::new(4, 2),
            |x, _| x != 2,
        );

        // Should find path around wall
        assert!(!path.is_empty());
    }
}
