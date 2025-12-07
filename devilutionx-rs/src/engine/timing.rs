// Timing module - Equivalent to nthread timing functions from C++
// Controls the two-speed game loop: fast path (60 FPS) vs logic path (2 Hz)

use std::time::{Duration, Instant};

/// Game timing controller
/// Manages the two-speed loop architecture:
/// - Fast path: Every frame (~60 Hz) for input/rendering
/// - Logic path: Every 500ms (~2 Hz) for game state updates
pub struct GameTiming {
    /// Last time game logic was executed
    last_logic_time: Instant,
    
    /// Interval between logic updates (500ms)
    logic_interval: Duration,
    
    /// Frame counter for diagnostics
    frame_count: u64,
    
    /// Logic tick counter
    logic_tick_count: u64,
}

impl GameTiming {
    /// Create new timing controller with 500ms logic interval
    pub fn new() -> Self {
        Self {
            last_logic_time: Instant::now(),
            logic_interval: Duration::from_millis(500),
            frame_count: 0,
            logic_tick_count: 0,
        }
    }
    
    /// Create timing controller with custom interval (for testing)
    pub fn with_interval(interval_ms: u64) -> Self {
        Self {
            last_logic_time: Instant::now(),
            logic_interval: Duration::from_millis(interval_ms),
            frame_count: 0,
            logic_tick_count: 0,
        }
    }
    
    /// Check if 500ms has passed since last logic update
    /// 
    /// Equivalent to C++: nthread_has_500ms_passed(&drawGame)
    /// 
    /// # Arguments
    /// * `draw_flag` - Set to true if rendering should occur this frame
    /// 
    /// # Returns
    /// * `true` - 500ms elapsed, should run game logic
    /// * `false` - Not yet time, use fast path
    pub fn has_500ms_passed(&mut self, draw_flag: &mut bool) -> bool {
        self.frame_count += 1;
        
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_logic_time);
        
        if elapsed >= self.logic_interval {
            // Time for logic update
            self.last_logic_time = now;
            self.logic_tick_count += 1;
            *draw_flag = true;
            true
        } else {
            // Fast path - may or may not draw
            // For now, always allow drawing (can optimize later)
            *draw_flag = true;
            false
        }
    }
    
    /// Reset timing (called on game start or level transition)
    pub fn reset(&mut self) {
        self.last_logic_time = Instant::now();
        self.frame_count = 0;
        self.logic_tick_count = 0;
    }
    
    /// Get frame count (diagnostic)
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Get logic tick count (diagnostic)
    pub fn logic_tick_count(&self) -> u64 {
        self.logic_tick_count
    }
    
    /// Get current FPS (approximate)
    pub fn current_fps(&self) -> f64 {
        if self.logic_tick_count == 0 {
            return 0.0;
        }
        
        let elapsed = Instant::now().duration_since(self.last_logic_time);
        let total_time = elapsed.as_secs_f64() + (self.logic_tick_count as f64 * 0.5);
        
        if total_time > 0.0 {
            self.frame_count as f64 / total_time
        } else {
            0.0
        }
    }
}

impl Default for GameTiming {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_timing_basic() {
        let mut timing = GameTiming::with_interval(100); // 100ms for faster testing
        let mut draw_flag = false;
        
        // First call should not trigger logic (just started)
        assert!(!timing.has_500ms_passed(&mut draw_flag));
        assert!(draw_flag); // Should still allow drawing
        
        // Wait and check again
        thread::sleep(Duration::from_millis(110));
        assert!(timing.has_500ms_passed(&mut draw_flag));
        assert_eq!(timing.logic_tick_count(), 1);
    }
    
    #[test]
    fn test_timing_multiple_ticks() {
        let mut timing = GameTiming::with_interval(50);
        let mut draw_flag = false;
        
        for i in 0..10 {
            timing.has_500ms_passed(&mut draw_flag);
            thread::sleep(Duration::from_millis(60));
        }
        
        // Should have triggered logic multiple times
        assert!(timing.logic_tick_count() >= 8);
        assert!(timing.frame_count() == 10);
    }
    
    #[test]
    fn test_timing_reset() {
        let mut timing = GameTiming::with_interval(100);
        let mut draw_flag = false;
        
        timing.has_500ms_passed(&mut draw_flag);
        timing.has_500ms_passed(&mut draw_flag);
        
        assert_eq!(timing.frame_count(), 2);
        
        timing.reset();
        assert_eq!(timing.frame_count(), 0);
        assert_eq!(timing.logic_tick_count(), 0);
    }
}
