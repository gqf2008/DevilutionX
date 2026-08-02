//! Monster health bar QoL (C++ `Source/qol/monhealthbar.cpp`).
//!
//! Ports the bar-geometry computation: the fill width from current/max hit
//! points, including the lifesteal wrap-around (HP above max draws a blue
//! bar for each full extra bar). The CLX sprite rendering is a follow-up.

/// C++ `monster.hitPoints` / `monster.maxHitPoints` are 64x fixed point.
/// `barWidth` is the full fill width (C++ `(*health)[0].width()`).

/// Result of computing the health-bar fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarMetrics {
    /// The fill width to draw (C++ `barProgress`).
    pub progress: i32,
    /// Number of full extra bars for lifestealing monsters (blue bars).
    pub multiplier: i32,
    /// The current bar's life value (wrapped for lifesteal).
    pub current_life: i32,
}

/// C++ geometry inside `DrawMonsterHealthBar`:
///
/// ```cpp
/// int multiplier = 0;
/// int currLife = monster.hitPoints;
/// if (monster.hitPoints > monster.maxHitPoints) {
///     multiplier = monster.hitPoints / monster.maxHitPoints;
///     currLife = monster.hitPoints - monster.maxHitPoints * multiplier;
///     if (currLife == 0 && multiplier > 0) {
///         multiplier--;
///         currLife = monster.maxHitPoints;
///     }
/// }
/// const int barProgress = (barWidth * currLife) / monster.maxHitPoints;
/// ```
pub fn bar_metrics(bar_width: i32, hit_points: i32, max_hit_points: i32) -> BarMetrics {
    let mut multiplier = 0i32;
    let mut current_life = hit_points;
    if hit_points > max_hit_points {
        multiplier = hit_points / max_hit_points;
        current_life = hit_points - max_hit_points * multiplier;
        if current_life == 0 && multiplier > 0 {
            multiplier -= 1;
            current_life = max_hit_points;
        }
    }
    let progress = if max_hit_points > 0 {
        (bar_width * current_life) / max_hit_points
    } else {
        0
    };
    BarMetrics {
        progress,
        multiplier,
        current_life,
    }
}

/// C++ `pcursmonst`: the monster currently under the mouse cursor.
///
/// The demo cursor is projected to a world tile (same space as `Monster::x/y`);
/// returns the first alive monster on that tile, mirroring the C++ hover lookup.
pub fn find_hovered_monster<'a>(
    monsters: impl IntoIterator<Item = &'a crate::game::monster::Monster>,
    tile: (i32, i32),
) -> Option<&'a crate::game::monster::Monster> {
    monsters
        .into_iter()
        .find(|m| m.x == tile.0 && m.y == tile.1 && m.is_alive())
}

/// C++ `DrawMonsterHealthBar` bar geometry constants (the CLX sprite width is
/// replaced by a fixed pixel width for the rect-based renderer).
pub const HEALTH_BAR_WIDTH: i32 = 94;
pub const HEALTH_BAR_HEIGHT: i32 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_monster_bar() {
        // 50% health of a 64x fixed-point monster.
        let m = bar_metrics(100, 32, 64);
        assert_eq!(m.progress, 50);
        assert_eq!(m.multiplier, 0);
        assert_eq!(m.current_life, 32);
    }

    #[test]
    fn test_full_health_fills_bar() {
        let m = bar_metrics(100, 64, 64);
        assert_eq!(m.progress, 100);
        assert_eq!(m.multiplier, 0);
    }

    #[test]
    fn test_lifesteal_wrap_shows_extra_blue_bar() {
        // 1.5x max HP -> one full extra bar + a half bar.
        let m = bar_metrics(100, 96, 64);
        assert_eq!(m.multiplier, 1);
        assert_eq!(m.current_life, 32);
        assert_eq!(m.progress, 50);
    }

    #[test]
    fn test_lifesteal_exact_multiple_borrows_one_bar() {
        // Exactly 2x max HP: multiplier becomes 1 and the bar is full.
        let m = bar_metrics(100, 128, 64);
        assert_eq!(m.multiplier, 1);
        assert_eq!(m.current_life, 64);
        assert_eq!(m.progress, 100);
    }

    #[test]
    fn test_dead_monster_has_zero_bar() {
        let m = bar_metrics(100, 0, 64);
        assert_eq!(m.progress, 0);
        assert_eq!(m.multiplier, 0);
    }


    #[test]
    fn test_find_hovered_monster_by_tile() {
        use crate::game::monster::{Monster, MonsterAIState, MonsterType};
        let mut zombie = Monster::new(1, MonsterType::Zombie, 20, 10, 1);
        zombie.ai_state = MonsterAIState::Idle;
        let mut dead = Monster::new(2, MonsterType::Zombie, 21, 10, 1);
        dead.ai_state = MonsterAIState::Dead;
        let others = vec![zombie, dead];

        // Monster on the hovered tile.
        assert!(find_hovered_monster(others.iter(), (20, 10)).is_some());
        // Dead monster is skipped.
        assert!(find_hovered_monster(others.iter(), (21, 10)).is_none());
        // Empty tile.
        assert!(find_hovered_monster(others.iter(), (30, 30)).is_none());
    }

}
