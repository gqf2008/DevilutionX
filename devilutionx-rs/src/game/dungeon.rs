//! Dungeon Generation System - Based on DevilutionX drlg_l*.cpp
//!
//! Procedural dungeon generation using BSP trees and cellular automata

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use super::types::{DungeonType, Point};

/// Tile types for dungeon generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Floor,
    Door,
    Stairs,
    StairsDown,
    StairsUp,
    Pillar,
    Altar,
    Chest,
    Barrel,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        matches!(self, TileType::Floor | TileType::Door | TileType::Stairs |
                      TileType::StairsDown | TileType::StairsUp)
    }

    pub fn is_transparent(&self) -> bool {
        !matches!(self, TileType::Wall | TileType::Pillar)
    }
}

/// Room structure for BSP generation
#[derive(Debug, Clone)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub connected: bool,
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height, connected: false }
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
}

/// Generated dungeon map
pub struct DungeonMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileType>>,
    pub rooms: Vec<Room>,
    pub spawn_point: Point,
    pub exit_point: Point,
    pub dungeon_type: DungeonType,
    pub level: u8,
}

impl DungeonMap {
    /// Create a new dungeon with given parameters
    pub fn generate(width: usize, height: usize, dungeon_type: DungeonType, level: u8, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut map = Self {
            width,
            height,
            tiles: vec![vec![TileType::Wall; width]; height],
            rooms: Vec::new(),
            spawn_point: Point::new(0, 0),
            exit_point: Point::new(0, 0),
            dungeon_type,
            level,
        };

        match dungeon_type {
            DungeonType::Town => map.generate_town(&mut rng),
            DungeonType::Cathedral => map.generate_cathedral(&mut rng),
            DungeonType::Catacombs => map.generate_catacombs(&mut rng),
            DungeonType::Caves => map.generate_caves(&mut rng),
            DungeonType::Hell => map.generate_hell(&mut rng),
            _ => map.generate_cathedral(&mut rng),
        }

        map
    }

    /// Generate town (open area)
    fn generate_town(&mut self, rng: &mut StdRng) {
        // Town is mostly open
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                self.tiles[y][x] = TileType::Floor;
            }
        }

        // Add some buildings/structures
        self.add_building(5, 5, 8, 6, rng);
        self.add_building(20, 5, 10, 8, rng);
        self.add_building(5, 20, 6, 6, rng);
        self.add_building(35, 15, 8, 8, rng);

        self.spawn_point = Point::new(self.width as i32 / 2, self.height as i32 / 2);
        self.exit_point = Point::new(self.width as i32 - 5, self.height as i32 - 5);
    }

    /// Add a building to town
    fn add_building(&mut self, x: usize, y: usize, w: usize, h: usize, rng: &mut StdRng) {
        if x + w >= self.width || y + h >= self.height {
            return;
        }

        // Walls
        for by in y..y + h {
            for bx in x..x + w {
                if by == y || by == y + h - 1 || bx == x || bx == x + w - 1 {
                    self.tiles[by][bx] = TileType::Wall;
                }
            }
        }

        // Door
        let door_side = rng.gen_range(0..4);
        match door_side {
            0 => self.tiles[y][x + w / 2] = TileType::Door,
            1 => self.tiles[y + h - 1][x + w / 2] = TileType::Door,
            2 => self.tiles[y + h / 2][x] = TileType::Door,
            _ => self.tiles[y + h / 2][x + w - 1] = TileType::Door,
        }
    }

    /// Generate Cathedral style dungeon (L1 - rectangular rooms)
    fn generate_cathedral(&mut self, rng: &mut StdRng) {
        let num_rooms = rng.gen_range(8..15);
        let mut attempts = 0;

        while self.rooms.len() < num_rooms && attempts < 1000 {
            let room_width = rng.gen_range(5..12);
            let room_height = rng.gen_range(5..12);
            let x = rng.gen_range(1..self.width as i32 - room_width - 1);
            let y = rng.gen_range(1..self.height as i32 - room_height - 1);

            let new_room = Room::new(x, y, room_width, room_height);

            // Check for overlaps
            let mut overlaps = false;
            for room in &self.rooms {
                if new_room.intersects(room) {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps {
                self.carve_room(&new_room);

                // Connect to previous room
                if !self.rooms.is_empty() {
                    let prev_center = self.rooms.last().unwrap().center();
                    let new_center = new_room.center();
                    self.carve_corridor(prev_center, new_center, rng);
                }

                self.rooms.push(new_room);
            }

            attempts += 1;
        }

        // Set spawn and exit
        if !self.rooms.is_empty() {
            self.spawn_point = self.rooms[0].center();
            self.exit_point = self.rooms.last().unwrap().center();

            // Add stairs
            let exit = self.exit_point;
            self.tiles[exit.y as usize][exit.x as usize] = TileType::StairsDown;
        }

        // Add decorations
        self.add_cathedral_decorations(rng);
    }

    /// Generate Catacombs style dungeon (L2 - maze-like)
    fn generate_catacombs(&mut self, rng: &mut StdRng) {
        // Use recursive backtracking maze algorithm
        self.generate_maze(rng);

        // Widen some corridors
        self.widen_passages(rng, 3);

        // Add rooms
        for _ in 0..5 {
            let x = rng.gen_range(3..self.width as i32 - 8);
            let y = rng.gen_range(3..self.height as i32 - 8);
            let room = Room::new(x, y, rng.gen_range(4..8), rng.gen_range(4..8));
            self.carve_room(&room);
            self.rooms.push(room);
        }

        // Set spawn and exit
        self.spawn_point = Point::new(2, 2);
        self.find_valid_spawn();
        self.exit_point = Point::new(self.width as i32 - 3, self.height as i32 - 3);
        self.find_valid_exit();

        let exit = self.exit_point;
        if self.tiles[exit.y as usize][exit.x as usize] == TileType::Floor {
            self.tiles[exit.y as usize][exit.x as usize] = TileType::StairsDown;
        }
    }

    /// Generate Caves style dungeon (L3 - cellular automata)
    fn generate_caves(&mut self, rng: &mut StdRng) {
        // Initial random fill
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if rng.gen_ratio(45, 100) {
                    self.tiles[y][x] = TileType::Floor;
                }
            }
        }

        // Cellular automata smoothing
        for _ in 0..5 {
            self.cellular_automata_step();
        }

        // Ensure connectivity
        self.ensure_connectivity(rng);

        // Set spawn and exit
        self.find_valid_spawn();
        self.exit_point = Point::new(self.width as i32 - 5, self.height as i32 - 5);
        self.find_valid_exit();

        let exit = self.exit_point;
        if self.tiles[exit.y as usize][exit.x as usize] == TileType::Floor {
            self.tiles[exit.y as usize][exit.x as usize] = TileType::StairsDown;
        }
    }

    /// Generate Hell style dungeon (L4 - mixed)
    fn generate_hell(&mut self, rng: &mut StdRng) {
        // Hell combines elements
        self.generate_cathedral(rng);

        // Add lava pools (represented as walls for now)
        for _ in 0..rng.gen_range(3..8) {
            let cx = rng.gen_range(5..self.width - 5) as i32;
            let cy = rng.gen_range(5..self.height - 5) as i32;
            let radius = rng.gen_range(2..5);

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let nx = (cx + dx) as usize;
                        let ny = (cy + dy) as usize;
                        if nx > 0 && nx < self.width - 1 && ny > 0 && ny < self.height - 1 {
                            // Don't overwrite important tiles
                            if self.tiles[ny][nx] == TileType::Floor {
                                self.tiles[ny][nx] = TileType::Wall; // Lava = impassable
                            }
                        }
                    }
                }
            }
        }
    }

    /// Carve a room into the map
    fn carve_room(&mut self, room: &Room) {
        for y in room.y..room.y + room.height {
            for x in room.x..room.x + room.width {
                if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
            }
        }
    }

    /// Carve a corridor between two points
    fn carve_corridor(&mut self, from: Point, to: Point, rng: &mut StdRng) {
        let mut x = from.x;
        let mut y = from.y;

        // Randomly choose horizontal or vertical first
        if rng.gen_bool(0.5) {
            // Horizontal then vertical
            while x != to.x {
                if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
                x += if to.x > x { 1 } else { -1 };
            }
            while y != to.y {
                if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
                y += if to.y > y { 1 } else { -1 };
            }
        } else {
            // Vertical then horizontal
            while y != to.y {
                if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
                y += if to.y > y { 1 } else { -1 };
            }
            while x != to.x {
                if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
                x += if to.x > x { 1 } else { -1 };
            }
        }
    }

    /// Generate maze using recursive backtracking
    fn generate_maze(&mut self, rng: &mut StdRng) {
        let mut visited = vec![vec![false; self.width]; self.height];
        let mut stack = Vec::new();

        // Start from (1, 1)
        let start_x = 1;
        let start_y = 1;
        visited[start_y][start_x] = true;
        self.tiles[start_y][start_x] = TileType::Floor;
        stack.push((start_x, start_y));

        let directions = [(0, -2), (0, 2), (-2, 0), (2, 0)];

        while let Some(&(cx, cy)) = stack.last() {
            let mut neighbors = Vec::new();

            for &(dx, dy) in &directions {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;

                if nx > 0 && nx < self.width as i32 - 1 &&
                   ny > 0 && ny < self.height as i32 - 1 &&
                   !visited[ny as usize][nx as usize] {
                    neighbors.push((nx as usize, ny as usize, dx, dy));
                }
            }

            if neighbors.is_empty() {
                stack.pop();
            } else {
                let (nx, ny, dx, dy) = neighbors[rng.gen_range(0..neighbors.len())];

                // Carve passage
                let mid_x = (cx as i32 + dx / 2) as usize;
                let mid_y = (cy as i32 + dy / 2) as usize;

                self.tiles[mid_y][mid_x] = TileType::Floor;
                self.tiles[ny][nx] = TileType::Floor;
                visited[ny][nx] = true;
                stack.push((nx, ny));
            }
        }
    }

    /// Widen maze passages
    fn widen_passages(&mut self, rng: &mut StdRng, chance: u32) {
        let old_tiles = self.tiles.clone();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if old_tiles[y][x] == TileType::Floor {
                    // Randomly widen
                    if rng.gen_ratio(1, chance) {
                        for dy in -1i32..=1 {
                            for dx in -1i32..=1 {
                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                if nx > 0 && nx < self.width - 1 && ny > 0 && ny < self.height - 1 {
                                    self.tiles[ny][nx] = TileType::Floor;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Cellular automata step for cave generation
    fn cellular_automata_step(&mut self) {
        let old_tiles = self.tiles.clone();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let mut walls = 0;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if old_tiles[ny][nx] == TileType::Wall {
                            walls += 1;
                        }
                    }
                }

                // 4-5 rule
                if walls >= 5 {
                    self.tiles[y][x] = TileType::Wall;
                } else {
                    self.tiles[y][x] = TileType::Floor;
                }
            }
        }
    }

    /// Ensure cave is connected
    fn ensure_connectivity(&mut self, rng: &mut StdRng) {
        // Simple flood fill to find disconnected regions and connect them
        let mut region_id = vec![vec![0u32; self.width]; self.height];
        let mut current_region = 0u32;
        let mut region_points: Vec<Vec<(usize, usize)>> = Vec::new();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.tiles[y][x] == TileType::Floor && region_id[y][x] == 0 {
                    current_region += 1;
                    let mut points = Vec::new();
                    self.flood_fill(x, y, current_region, &mut region_id, &mut points);
                    region_points.push(points);
                }
            }
        }

        // Connect all regions to the largest one
        if region_points.len() > 1 {
            // Find largest region
            let largest_idx = region_points.iter()
                .enumerate()
                .max_by_key(|(_, p)| p.len())
                .map(|(i, _)| i)
                .unwrap_or(0);

            let largest_point = region_points[largest_idx][0];

            for (i, points) in region_points.iter().enumerate() {
                if i != largest_idx && !points.is_empty() {
                    let point = points[rng.gen_range(0..points.len())];
                    self.carve_corridor(
                        Point::new(point.0 as i32, point.1 as i32),
                        Point::new(largest_point.0 as i32, largest_point.1 as i32),
                        rng
                    );
                }
            }
        }
    }

    /// Flood fill for connectivity check
    fn flood_fill(&self, x: usize, y: usize, region: u32, region_id: &mut Vec<Vec<u32>>, points: &mut Vec<(usize, usize)>) {
        let mut stack = vec![(x, y)];

        while let Some((cx, cy)) = stack.pop() {
            if cx == 0 || cy == 0 || cx >= self.width - 1 || cy >= self.height - 1 {
                continue;
            }
            if region_id[cy][cx] != 0 || self.tiles[cy][cx] != TileType::Floor {
                continue;
            }

            region_id[cy][cx] = region;
            points.push((cx, cy));

            stack.push((cx + 1, cy));
            stack.push((cx - 1, cy));
            stack.push((cx, cy + 1));
            stack.push((cx, cy - 1));
        }
    }

    /// Find a valid spawn point
    fn find_valid_spawn(&mut self) {
        for y in 2..self.height - 2 {
            for x in 2..self.width - 2 {
                if self.tiles[y][x] == TileType::Floor {
                    self.spawn_point = Point::new(x as i32, y as i32);
                    return;
                }
            }
        }
    }

    /// Find a valid exit point
    fn find_valid_exit(&mut self) {
        for y in (2..self.height - 2).rev() {
            for x in (2..self.width - 2).rev() {
                if self.tiles[y][x] == TileType::Floor {
                    self.exit_point = Point::new(x as i32, y as i32);
                    return;
                }
            }
        }
    }

    /// Add decorations to cathedral
    fn add_cathedral_decorations(&mut self, rng: &mut StdRng) {
        for room in &self.rooms {
            // Add pillars in larger rooms
            if room.width >= 8 && room.height >= 8 {
                let px1 = room.x + 2;
                let px2 = room.x + room.width - 3;
                let py1 = room.y + 2;
                let py2 = room.y + room.height - 3;

                if rng.gen_bool(0.5) {
                    self.tiles[py1 as usize][px1 as usize] = TileType::Pillar;
                    self.tiles[py1 as usize][px2 as usize] = TileType::Pillar;
                    self.tiles[py2 as usize][px1 as usize] = TileType::Pillar;
                    self.tiles[py2 as usize][px2 as usize] = TileType::Pillar;
                }
            }

            // Random barrels/chests
            for _ in 0..rng.gen_range(0..3) {
                let bx = room.x + rng.gen_range(1..room.width - 1);
                let by = room.y + rng.gen_range(1..room.height - 1);
                if self.tiles[by as usize][bx as usize] == TileType::Floor {
                    self.tiles[by as usize][bx as usize] = if rng.gen_bool(0.7) {
                        TileType::Barrel
                    } else {
                        TileType::Chest
                    };
                }
            }
        }
    }

    /// Get tile at position
    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileType> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(self.tiles[y as usize][x as usize])
        } else {
            None
        }
    }

    /// Check if position is walkable
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).map(|t| t.is_walkable()).unwrap_or(false)
    }

    /// Get all walkable positions
    pub fn get_walkable_positions(&self) -> Vec<Point> {
        let mut positions = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.tiles[y][x].is_walkable() {
                    positions.push(Point::new(x as i32, y as i32));
                }
            }
        }
        positions
    }

    /// Get random walkable position
    pub fn random_walkable_position(&self, rng: &mut impl Rng) -> Option<Point> {
        let positions = self.get_walkable_positions();
        if positions.is_empty() {
            None
        } else {
            Some(positions[rng.gen_range(0..positions.len())])
        }
    }
}
