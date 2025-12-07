//! Path - 寻路算法
//!
//! 移植自 Source/engine/path.cpp/h
//!
//! 使用 A* 算法实现寻路

use super::types::{Displacement, Point};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// 怪物最大路径长度
pub const MAX_PATH_LENGTH_MONSTERS: usize = 25;

/// 玩家最大路径长度
pub const MAX_PATH_LENGTH_PLAYER: usize = 100;

/// 轴对齐步进代价
pub const PATH_AXIS_ALIGNED_STEP_COST: i32 = 100;

/// 对角步进代价（略高以避免过多对角移动）
pub const PATH_DIAGONAL_STEP_COST: i32 = 101;

/// 8 方向位移数组
pub const PATH_DIRS: [Displacement; 8] = [
    Displacement::new(-1, -1), // Direction::North
    Displacement::new(-1, 1),  // Direction::West
    Displacement::new(1, -1),  // Direction::East
    Displacement::new(1, 1),   // Direction::South
    Displacement::new(-1, 0),  // Direction::NorthWest
    Displacement::new(0, -1),  // Direction::NorthEast
    Displacement::new(1, 0),   // Direction::SouthEast
    Displacement::new(0, 1),   // Direction::SouthWest
];

/// 获取路径方向编码
///
/// 返回一个数字表示从起点到相邻目标点的方向：
/// ```text
///       dx
///     -1 0 1
///     +-----
///   -1|5 1 6
/// dy 0|2 0 3
///    1|8 4 7
/// ```
pub fn get_path_direction(start: Point, dest: Point) -> i8 {
    const PATH_DIRECTIONS: [i8; 9] = [5, 1, 6, 2, 0, 3, 8, 4, 7];
    let dx = (dest.x - start.x).clamp(-1, 1) + 1;
    let dy = (dest.y - start.y).clamp(-1, 1) + 1;
    PATH_DIRECTIONS[(dy * 3 + dx) as usize]
}

/// A* 节点
#[derive(Clone, Copy, Debug)]
struct PathNode {
    position: Point,
    /// f = g + h
    f_cost: i32,
    /// 从起点到此节点的代价
    g_cost: i32,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 反转顺序以创建最小堆
        other.f_cost.cmp(&self.f_cost)
    }
}

/// 探索节点信息
#[derive(Clone, Copy, Debug)]
struct ExploredNode {
    prev: Point,
    g_cost: i32,
}

/// 检查是否为对角步进
fn is_diagonal_step(a: Point, b: Point) -> bool {
    a.x != b.x && a.y != b.y
}

/// 获取两个相邻节点间的距离
fn get_distance(start: Point, dest: Point) -> i32 {
    if is_diagonal_step(start, dest) {
        PATH_DIAGONAL_STEP_COST
    } else {
        PATH_AXIS_ALIGNED_STEP_COST
    }
}

/// 启发式函数：估计从起点到终点的代价
fn get_heuristic_cost(start: Point, dest: Point) -> i32 {
    let dx = (start.x - dest.x).abs();
    let dy = (start.y - dest.y).abs();
    let diag_steps = dx.min(dy);
    let axis_steps = (dx - dy).abs();
    diag_steps * PATH_DIAGONAL_STEP_COST + axis_steps * PATH_AXIS_ALIGNED_STEP_COST
}

/// 重建路径
fn reconstruct_path(
    explored: &HashMap<Point, ExploredNode>,
    dest: Point,
    max_path_length: usize,
) -> Vec<i8> {
    let mut path = Vec::new();
    let mut cur = dest;

    while let Some(node) = explored.get(&cur) {
        if node.g_cost == 0 {
            break; // 到达起点
        }
        if path.len() >= max_path_length {
            return Vec::new(); // 路径太长
        }
        path.push(get_path_direction(node.prev, cur));
        cur = node.prev;
    }

    path.reverse();
    path
}

/// 寻找从起点到终点的最短路径
///
/// # Arguments
/// * `can_step` - 检查是否可以从一个点走到相邻点
/// * `pos_ok` - 检查一个位置是否可以站立
/// * `start` - 起点
/// * `dest` - 终点
/// * `max_path_length` - 最大路径长度
///
/// # Returns
/// 路径方向数组，如果找不到路径则返回空数组
pub fn find_path<F1, F2>(
    can_step: F1,
    pos_ok: F2,
    start: Point,
    dest: Point,
    max_path_length: usize,
) -> Vec<i8>
where
    F1: Fn(Point, Point) -> bool,
    F2: Fn(Point) -> bool,
{
    let initial_h = get_heuristic_cost(start, dest);
    if initial_h > PATH_DIAGONAL_STEP_COST * max_path_length as i32 {
        return Vec::new();
    }

    let mut frontier = BinaryHeap::new();
    let mut explored: HashMap<Point, ExploredNode> = HashMap::new();

    frontier.push(PathNode {
        position: start,
        f_cost: initial_h,
        g_cost: 0,
    });
    explored.insert(start, ExploredNode { prev: start, g_cost: 0 });

    while let Some(cur) = frontier.pop() {
        if cur.position == dest {
            return reconstruct_path(&explored, cur.position, max_path_length);
        }

        let cur_g = match explored.get(&cur.position) {
            Some(node) => node.g_cost,
            None => continue,
        };

        // 跳过已经超过最大步数的节点
        if cur_g >= PATH_DIAGONAL_STEP_COST * max_path_length as i32 {
            continue;
        }

        // 检查节点是否已过期
        if cur_g + get_heuristic_cost(cur.position, dest) > cur.f_cost {
            continue;
        }

        for dir in &PATH_DIRS {
            let neighbor_pos = Point::new(
                cur.position.x + dir.delta_x,
                cur.position.y + dir.delta_y,
            );

            // 边界检查
            if neighbor_pos.x < 0 || neighbor_pos.y < 0 {
                continue;
            }

            let ok = pos_ok(neighbor_pos);
            if ok {
                if !can_step(cur.position, neighbor_pos) {
                    continue;
                }
            } else {
                // 允许目标是不可行走的位置（如怪物位置）
                if neighbor_pos != dest {
                    continue;
                }
            }

            let g = cur_g + get_distance(cur.position, neighbor_pos);
            if g >= PATH_DIAGONAL_STEP_COST * max_path_length as i32 {
                continue;
            }

            let improved = match explored.get_mut(&neighbor_pos) {
                None => {
                    explored.insert(neighbor_pos, ExploredNode {
                        prev: cur.position,
                        g_cost: g,
                    });
                    true
                }
                Some(node) if node.g_cost > g => {
                    node.prev = cur.position;
                    node.g_cost = g;
                    true
                }
                _ => false,
            };

            if improved {
                let f = g + get_heuristic_cost(neighbor_pos, dest);
                frontier.push(PathNode {
                    position: neighbor_pos,
                    f_cost: f,
                    g_cost: g,
                });
            }
        }
    }

    Vec::new()
}

/// 寻找最近的有效位置
///
/// 从起点开始，在扩展的"环"中搜索通过检查的最近位置
///
/// # Arguments
/// * `pos_ok` - 检查位置是否有效
/// * `start` - 起始位置
/// * `min_radius` - 最小搜索半径
/// * `max_radius` - 最大搜索半径
///
/// # Returns
/// 最近的有效位置，如果没有找到则返回 None
pub fn find_closest_valid_position<F>(
    pos_ok: F,
    start: Point,
    min_radius: u32,
    max_radius: u32,
) -> Option<Point>
where
    F: Fn(Point) -> bool,
{
    for radius in min_radius..=max_radius {
        // 搜索当前半径的环
        for i in -(radius as i32)..=(radius as i32) {
            for j in -(radius as i32)..=(radius as i32) {
                // 只检查当前半径上的点
                if i.abs().max(j.abs()) != radius as i32 {
                    continue;
                }

                let pos = Point::new(start.x + i, start.y + j);
                if pos.x >= 0 && pos.y >= 0 && pos_ok(pos) {
                    return Some(pos);
                }
            }
        }
    }
    None
}

/// 简单的广度优先搜索寻路
pub fn find_path_bfs<F>(
    can_move: F,
    start: Point,
    dest: Point,
    max_steps: usize,
) -> Vec<Point>
where
    F: Fn(Point) -> bool,
{
    use std::collections::VecDeque;

    if start == dest {
        return vec![start];
    }

    let mut queue = VecDeque::new();
    let mut visited: HashMap<Point, Point> = HashMap::new();

    queue.push_back(start);
    visited.insert(start, start);

    while let Some(cur) = queue.pop_front() {
        if cur == dest {
            // 重建路径
            let mut path = vec![dest];
            let mut pos = dest;
            while pos != start {
                if let Some(&prev) = visited.get(&pos) {
                    if prev == pos {
                        break;
                    }
                    path.push(prev);
                    pos = prev;
                } else {
                    break;
                }
            }
            path.reverse();
            return path;
        }

        if visited.len() >= max_steps {
            break;
        }

        for dir in &PATH_DIRS {
            let next = Point::new(cur.x + dir.delta_x, cur.y + dir.delta_y);
            if next.x >= 0 && next.y >= 0 && can_move(next) && !visited.contains_key(&next) {
                visited.insert(next, cur);
                queue.push_back(next);
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_path_direction() {
        // 测试各方向
        assert_eq!(get_path_direction(Point::new(5, 5), Point::new(5, 4)), 1); // 上
        assert_eq!(get_path_direction(Point::new(5, 5), Point::new(5, 6)), 4); // 下
        assert_eq!(get_path_direction(Point::new(5, 5), Point::new(4, 5)), 2); // 左
        assert_eq!(get_path_direction(Point::new(5, 5), Point::new(6, 5)), 3); // 右
    }

    #[test]
    fn test_heuristic_cost() {
        let start = Point::new(0, 0);
        let dest = Point::new(3, 4);

        let cost = get_heuristic_cost(start, dest);
        // 3 对角步 + 1 直线步
        let expected = 3 * PATH_DIAGONAL_STEP_COST + 1 * PATH_AXIS_ALIGNED_STEP_COST;
        assert_eq!(cost, expected);
    }

    #[test]
    fn test_find_path_simple() {
        // 简单的 10x10 网格，全部可行走
        let can_step = |_: Point, _: Point| true;
        let pos_ok = |p: Point| p.x >= 0 && p.x < 10 && p.y >= 0 && p.y < 10;

        let path = find_path(
            can_step,
            pos_ok,
            Point::new(0, 0),
            Point::new(3, 3),
            MAX_PATH_LENGTH_PLAYER,
        );

        assert!(!path.is_empty());
        // 对角移动 3 步即可到达
        assert!(path.len() <= 5);
    }

    #[test]
    fn test_find_path_with_obstacle() {
        // 有障碍物的网格
        let obstacle = Point::new(1, 1);
        let can_step = |_from: Point, to: Point| to != obstacle;
        let pos_ok = |p: Point| p.x >= 0 && p.x < 5 && p.y >= 0 && p.y < 5 && p != obstacle;

        let path = find_path(
            can_step,
            pos_ok,
            Point::new(0, 0),
            Point::new(2, 2),
            MAX_PATH_LENGTH_PLAYER,
        );

        // 应该能找到绕过障碍的路径
        assert!(!path.is_empty());
    }

    #[test]
    fn test_find_closest_valid_position() {
        let blocked = Point::new(5, 5);
        let pos_ok = |p: Point| p != blocked;

        let result = find_closest_valid_position(pos_ok, blocked, 0, 5);
        assert!(result.is_some());
        let found = result.unwrap();
        // 应该找到相邻的位置
        assert!((found.x - blocked.x).abs() <= 1);
        assert!((found.y - blocked.y).abs() <= 1);
    }

    #[test]
    fn test_find_path_no_path() {
        // 目标被完全包围
        let can_step = |_: Point, _: Point| false;
        let pos_ok = |_: Point| true;

        let path = find_path(
            can_step,
            pos_ok,
            Point::new(0, 0),
            Point::new(5, 5),
            MAX_PATH_LENGTH_PLAYER,
        );

        assert!(path.is_empty());
    }
}
