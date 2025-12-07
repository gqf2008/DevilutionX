//! Crawl Algorithm - Exact port from C++ Source/crawl.cpp
//!
//! 爬行遍历算法：从中心向外遍历周围的瓦片位置
//! 用于搜索周围物品、怪物、空闲位置等

use super::types::Displacement;

/// 从中心向外爬行遍历指定半径范围
///
/// 遍历顺序: 从半径0开始，逐圈向外扩展
/// 每圈的遍历顺序: 上/下 → 上下两行的内部点 → 对角点 → 左右 → 左右两列的内部点
///
/// # Arguments
/// * `radius` - 遍历的最大半径
/// * `callback` - 每个位置的回调函数，返回false时停止遍历
///
/// # Returns
/// 如果遍历完成返回true，被回调中断返回false
///
/// # Example
/// ```ignore
/// use devilutionx::game::crawl::do_crawl;
/// use devilutionx::game::types::Displacement;
///
/// let mut found_positions = Vec::new();
/// do_crawl(2, |d| {
///     found_positions.push((d.delta_x, d.delta_y));
///     true // 继续遍历
/// });
/// ```
pub fn do_crawl<F>(radius: u32, mut callback: F) -> bool
where
    F: FnMut(Displacement) -> bool,
{
    do_crawl_range(0, radius, &mut callback)
}

/// 从中心向外爬行遍历指定半径范围（指定最小和最大半径）
///
/// # Arguments
/// * `min_radius` - 开始遍历的最小半径
/// * `max_radius` - 遍历的最大半径
/// * `callback` - 每个位置的回调函数，返回false时停止遍历
///
/// # Returns
/// 如果遍历完成返回true，被回调中断返回false
pub fn do_crawl_range<F>(min_radius: u32, max_radius: u32, mut callback: F) -> bool
where
    F: FnMut(Displacement) -> bool,
{
    for r in min_radius as i32..=max_radius as i32 {
        // 先检查正上方 (0, r)
        if !callback(Displacement::new(0, r)) {
            return false;
        }

        if r == 0 {
            continue;
        }

        // 检查正下方 (0, -r)
        if !callback(Displacement::new(0, -r)) {
            return false;
        }

        // 上下两行的内部点 (-x, r), (x, r), (-x, -r), (x, -r) for x in 1..r
        for x in 1..r {
            if !callback(Displacement::new(-x, r)) {
                return false;
            }
            if !callback(Displacement::new(x, r)) {
                return false;
            }
            if !callback(Displacement::new(-x, -r)) {
                return false;
            }
            if !callback(Displacement::new(x, -r)) {
                return false;
            }
        }

        // 对角点 (r > 1时)
        if r > 1 {
            let d = r - 1;
            if !callback(Displacement::new(-d, d)) {
                return false;
            }
            if !callback(Displacement::new(d, d)) {
                return false;
            }
            if !callback(Displacement::new(-d, -d)) {
                return false;
            }
            if !callback(Displacement::new(d, -d)) {
                return false;
            }
        }

        // 左右两侧 (-r, 0), (r, 0)
        if !callback(Displacement::new(-r, 0)) {
            return false;
        }
        if !callback(Displacement::new(r, 0)) {
            return false;
        }

        // 左右两列的内部点 (-r, y), (r, y), (-r, -y), (r, -y) for y in 1..r
        for y in 1..r {
            if !callback(Displacement::new(-r, y)) {
                return false;
            }
            if !callback(Displacement::new(r, y)) {
                return false;
            }
            if !callback(Displacement::new(-r, -y)) {
                return false;
            }
            if !callback(Displacement::new(r, -y)) {
                return false;
            }
        }
    }

    true
}

/// 计算指定半径范围内的总位置数
///
/// # Arguments
/// * `radius` - 最大半径
///
/// # Returns
/// 总位置数
pub fn count_positions_in_radius(radius: u32) -> u32 {
    let mut count = 0u32;
    do_crawl(radius, |_| {
        count += 1;
        true
    });
    count
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_do_crawl_radius_0() {
        let mut positions = Vec::new();
        let result = do_crawl(0, |d| {
            positions.push((d.delta_x, d.delta_y));
            true
        });

        assert!(result);
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], (0, 0));
    }

    #[test]
    fn test_do_crawl_radius_1() {
        let mut positions = Vec::new();
        let result = do_crawl(1, |d| {
            positions.push((d.delta_x, d.delta_y));
            true
        });

        assert!(result);
        // radius 0: (0,0) = 1
        // radius 1: (0,1), (0,-1), (-1,0), (1,0) = 4
        // total = 5
        assert_eq!(positions.len(), 5);
        assert!(positions.contains(&(0, 0)));
        assert!(positions.contains(&(0, 1)));
        assert!(positions.contains(&(0, -1)));
        assert!(positions.contains(&(-1, 0)));
        assert!(positions.contains(&(1, 0)));
    }

    #[test]
    fn test_do_crawl_radius_2() {
        let mut positions = Vec::new();
        let result = do_crawl(2, |d| {
            positions.push((d.delta_x, d.delta_y));
            true
        });

        assert!(result);
        // radius 0: 1
        // radius 1: 4
        // radius 2: (0,2), (0,-2), (-1,2), (1,2), (-1,-2), (1,-2),
        //           (-1,1), (1,1), (-1,-1), (1,-1),
        //           (-2,0), (2,0), (-2,1), (2,1), (-2,-1), (2,-1) = 16
        // total = 21
        assert_eq!(positions.len(), 21);
    }

    #[test]
    fn test_do_crawl_early_termination() {
        let mut count = 0;
        let result = do_crawl(5, |_| {
            count += 1;
            count < 10 // 在第10个位置停止
        });

        assert!(!result); // 未完成
        assert_eq!(count, 10);
    }

    #[test]
    fn test_do_crawl_range() {
        let mut positions = Vec::new();
        let result = do_crawl_range(2, 2, |d| {
            positions.push((d.delta_x, d.delta_y));
            true
        });

        assert!(result);
        // 只遍历半径2的一圈，不包括内部
        assert_eq!(positions.len(), 16);
    }

    #[test]
    fn test_count_positions() {
        assert_eq!(count_positions_in_radius(0), 1);
        assert_eq!(count_positions_in_radius(1), 5);
        assert_eq!(count_positions_in_radius(2), 21);
        // 公式: 1 + 4*1 + 4*2 + 4*(2-1)*2 = 1 + 4 + 8 + 8 = 21
        // 通用: sum(r=0..n) of (r==0 ? 1 : 4*r + 4*(r-1))
    }

    #[test]
    fn test_do_crawl_first_position_is_center() {
        let mut first = None;
        do_crawl(3, |d| {
            if first.is_none() {
                first = Some((d.delta_x, d.delta_y));
            }
            true
        });

        // 第一个位置应该是中心偏移 (0, 0)
        assert_eq!(first, Some((0, 0)));
    }

    #[test]
    fn test_crawl_symmetry() {
        let mut positions = Vec::new();
        do_crawl(3, |d| {
            positions.push((d.delta_x, d.delta_y));
            true
        });

        // 检查对称性：如果(x,y)存在，则(-x,y), (x,-y), (-x,-y)也应该存在
        for &(x, y) in &positions {
            if x != 0 {
                assert!(positions.contains(&(-x, y)), "Missing (-{}, {})", x, y);
            }
            if y != 0 {
                assert!(positions.contains(&(x, -y)), "Missing ({}, -{})", x, y);
            }
            if x != 0 && y != 0 {
                assert!(positions.contains(&(-x, -y)), "Missing (-{}, -{})", x, y);
            }
        }
    }
}
