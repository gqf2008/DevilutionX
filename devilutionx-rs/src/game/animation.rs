/// Sprite Animation System
/// Handles frame-based animations for characters, monsters, and effects
use std::collections::HashMap;
use super::types::{Direction, Point};

/// Animation state for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationState {
    Idle,
    Walking,
    Attacking,
    CastingSpell,
    GettingHit,
    Dying,
    Dead,
    Blocking,
}

/// A single animation frame
#[derive(Debug, Clone)]
pub struct AnimFrame {
    /// Frame index in spritesheet
    pub frame_id: u32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Offset from entity position
    pub offset_x: i32,
    pub offset_y: i32,
    /// Optional hitbox for attack frames
    pub hitbox: Option<HitBox>,
    /// Sound effect to play on this frame
    pub sound_id: Option<u32>,
}

/// Hitbox for attack collision
#[derive(Debug, Clone, Copy)]
pub struct HitBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl HitBox {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn intersects(&self, other: &HitBox) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
}

/// A complete animation sequence
#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<AnimFrame>,
    pub looping: bool,
    /// Frame where attack damage is dealt (for attack animations)
    pub attack_frame: Option<usize>,
}

impl Animation {
    pub fn new(name: &str, looping: bool) -> Self {
        Self {
            name: name.to_string(),
            frames: Vec::new(),
            looping,
            attack_frame: None,
        }
    }

    pub fn add_frame(&mut self, frame: AnimFrame) {
        self.frames.push(frame);
    }

    pub fn total_duration(&self) -> u32 {
        self.frames.iter().map(|f| f.duration_ms).sum()
    }

    /// Create a simple uniform animation
    pub fn simple(name: &str, num_frames: u32, frame_duration: u32, looping: bool) -> Self {
        let mut anim = Self::new(name, looping);
        for i in 0..num_frames {
            anim.add_frame(AnimFrame {
                frame_id: i,
                duration_ms: frame_duration,
                offset_x: 0,
                offset_y: 0,
                hitbox: None,
                sound_id: None,
            });
        }
        anim
    }
}

/// Animation controller for a single entity
#[derive(Debug, Clone)]
pub struct AnimationController {
    /// Current animation being played
    current_anim: AnimationState,
    /// Current direction
    direction: Direction,
    /// Current frame index
    current_frame: usize,
    /// Time accumulated on current frame (ms)
    frame_time: u32,
    /// Is animation finished? (for non-looping)
    pub finished: bool,
    /// Animation speed multiplier
    speed_multiplier: f32,
    /// Animations for each state and direction
    animations: HashMap<(AnimationState, Direction), Animation>,
    /// Fallback animation (no direction variant)
    fallback_anims: HashMap<AnimationState, Animation>,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            current_anim: AnimationState::Idle,
            direction: Direction::South,
            current_frame: 0,
            frame_time: 0,
            finished: false,
            speed_multiplier: 1.0,
            animations: HashMap::new(),
            fallback_anims: HashMap::new(),
        }
    }

    /// Add animation for specific state and direction
    pub fn add_animation(&mut self, state: AnimationState, direction: Direction, anim: Animation) {
        self.animations.insert((state, direction), anim);
    }

    /// Add fallback animation (used when direction variant doesn't exist)
    pub fn add_fallback(&mut self, state: AnimationState, anim: Animation) {
        self.fallback_anims.insert(state, anim);
    }

    /// Set current animation state
    pub fn set_state(&mut self, state: AnimationState) {
        if self.current_anim != state {
            self.current_anim = state;
            self.current_frame = 0;
            self.frame_time = 0;
            self.finished = false;
        }
    }

    /// Set direction
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Get current animation state
    pub fn state(&self) -> AnimationState {
        self.current_anim
    }

    /// Get current animation
    fn current_animation(&self) -> Option<&Animation> {
        self.animations.get(&(self.current_anim, self.direction))
            .or_else(|| self.fallback_anims.get(&self.current_anim))
    }

    /// Update animation, returns true if attack frame was reached
    pub fn update(&mut self, delta_ms: u32) -> bool {
        let mut attack_triggered = false;

        // First get animation info we need
        let anim_info = self.current_animation().map(|anim| {
            (
                anim.frames.len(),
                anim.frames.get(self.current_frame).map(|f| f.duration_ms).unwrap_or(100),
                anim.attack_frame,
                anim.looping,
            )
        });

        if let Some((frame_count, frame_duration, attack_frame, looping)) = anim_info {
            if frame_count == 0 || self.finished {
                return false;
            }

            let adjusted_delta = (delta_ms as f32 * self.speed_multiplier) as u32;
            self.frame_time += adjusted_delta;

            while self.frame_time >= frame_duration {
                self.frame_time -= frame_duration;

                // Check if this was attack frame
                if let Some(af) = attack_frame {
                    if self.current_frame == af {
                        attack_triggered = true;
                    }
                }

                self.current_frame += 1;

                if self.current_frame >= frame_count {
                    if looping {
                        self.current_frame = 0;
                    } else {
                        self.current_frame = frame_count - 1;
                        self.finished = true;
                        break;
                    }
                }
            }
        }

        attack_triggered
    }

    /// Get current frame ID for rendering
    pub fn current_frame_id(&self) -> u32 {
        if let Some(anim) = self.current_animation() {
            if !anim.frames.is_empty() {
                return anim.frames[self.current_frame].frame_id;
            }
        }
        0
    }

    /// Get current frame data
    pub fn current_frame_data(&self) -> Option<&AnimFrame> {
        self.current_animation()
            .and_then(|anim| anim.frames.get(self.current_frame))
    }

    /// Get animation progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if let Some(anim) = self.current_animation() {
            let total = anim.total_duration();
            if total > 0 {
                let elapsed: u32 = anim.frames.iter()
                    .take(self.current_frame)
                    .map(|f| f.duration_ms)
                    .sum::<u32>() + self.frame_time;
                return (elapsed as f32) / (total as f32);
            }
        }
        0.0
    }

    pub fn set_speed(&mut self, multiplier: f32) {
        self.speed_multiplier = multiplier.max(0.1);
    }
}

/// Sprite sheet definition
#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub name: String,
    pub texture_id: u32,
    pub frame_width: u32,
    pub frame_height: u32,
    pub columns: u32,
    pub rows: u32,
    /// Pivot point offset (from top-left)
    pub pivot_x: i32,
    pub pivot_y: i32,
}

impl SpriteSheet {
    pub fn new(name: &str, texture_id: u32, frame_w: u32, frame_h: u32, cols: u32, rows: u32) -> Self {
        Self {
            name: name.to_string(),
            texture_id,
            frame_width: frame_w,
            frame_height: frame_h,
            columns: cols,
            rows,
            pivot_x: (frame_w / 2) as i32,
            pivot_y: frame_h as i32,  // Bottom center for characters
        }
    }

    /// Get source rectangle for a frame
    pub fn frame_rect(&self, frame_id: u32) -> (i32, i32, u32, u32) {
        let col = frame_id % self.columns;
        let row = frame_id / self.columns;
        (
            (col * self.frame_width) as i32,
            (row * self.frame_height) as i32,
            self.frame_width,
            self.frame_height,
        )
    }
}

/// Pre-built animation templates for Diablo-style characters
pub struct DiabloAnimations;

impl DiabloAnimations {
    /// Create warrior-style animation set
    pub fn warrior() -> AnimationController {
        let mut ctrl = AnimationController::new();

        // Each direction has its own animation row
        for (dir_idx, dir) in [
            Direction::South,
            Direction::SouthWest,
            Direction::West,
            Direction::NorthWest,
            Direction::North,
            Direction::NorthEast,
            Direction::East,
            Direction::SouthEast,
        ].iter().enumerate() {
            let base = dir_idx as u32 * 16; // 16 frames per direction

            // Idle animation (4 frames)
            let mut idle = Animation::simple("idle", 4, 200, true);
            for (i, frame) in idle.frames.iter_mut().enumerate() {
                frame.frame_id = base + i as u32;
            }
            ctrl.add_animation(AnimationState::Idle, *dir, idle);

            // Walk animation (8 frames)
            let mut walk = Animation::simple("walk", 8, 100, true);
            for (i, frame) in walk.frames.iter_mut().enumerate() {
                frame.frame_id = base + 4 + i as u32;
            }
            ctrl.add_animation(AnimationState::Walking, *dir, walk);

            // Attack animation (frames vary)
            let mut attack = Animation::simple("attack", 6, 80, false);
            attack.attack_frame = Some(3); // Damage on frame 3
            for (i, frame) in attack.frames.iter_mut().enumerate() {
                frame.frame_id = base + 12 + i as u32; // Offset into attack frames
            }
            ctrl.add_animation(AnimationState::Attacking, *dir, attack);
        }

        // Death animation (no direction, use single set)
        let mut death = Animation::simple("death", 10, 120, false);
        for (i, frame) in death.frames.iter_mut().enumerate() {
            frame.frame_id = 128 + i as u32; // Death frames at end of sheet
        }
        ctrl.add_fallback(AnimationState::Dying, death);

        ctrl
    }

    /// Create skeleton monster animation set
    pub fn skeleton() -> AnimationController {
        let mut ctrl = AnimationController::new();

        // Simplified - 4 directions
        for (dir_idx, dir) in [
            Direction::South,
            Direction::West,
            Direction::North,
            Direction::East,
        ].iter().enumerate() {
            let base = dir_idx as u32 * 12;

            // Idle
            ctrl.add_animation(AnimationState::Idle, *dir,
                Animation::simple("idle", 4, 250, true));

            // Walk (6 frames)
            let mut walk = Animation::simple("walk", 6, 120, true);
            for (i, frame) in walk.frames.iter_mut().enumerate() {
                frame.frame_id = base + i as u32;
            }
            ctrl.add_animation(AnimationState::Walking, *dir, walk);

            // Attack (4 frames)
            let mut attack = Animation::simple("attack", 4, 100, false);
            attack.attack_frame = Some(2);
            for (i, frame) in attack.frames.iter_mut().enumerate() {
                frame.frame_id = base + 6 + i as u32;
            }
            ctrl.add_animation(AnimationState::Attacking, *dir, attack);
        }

        // Death
        ctrl.add_fallback(AnimationState::Dying, Animation::simple("death", 6, 150, false));

        ctrl
    }

    /// Create generic monster animations based on type
    pub fn monster(monster_type: &str) -> AnimationController {
        match monster_type {
            "zombie" => Self::zombie(),
            "skeleton" => Self::skeleton(),
            _ => Self::skeleton(), // Default
        }
    }

    /// Zombie animation set
    pub fn zombie() -> AnimationController {
        let mut ctrl = AnimationController::new();

        // Zombies are slow
        ctrl.set_speed(0.7);

        for (dir_idx, dir) in [
            Direction::South,
            Direction::West,
            Direction::North,
            Direction::East,
        ].iter().enumerate() {
            let base = dir_idx as u32 * 10;

            // Slow idle
            ctrl.add_animation(AnimationState::Idle, *dir,
                Animation::simple("idle", 4, 300, true));

            // Slow walk
            let mut walk = Animation::simple("walk", 8, 150, true);
            for (i, frame) in walk.frames.iter_mut().enumerate() {
                frame.frame_id = base + i as u32;
            }
            ctrl.add_animation(AnimationState::Walking, *dir, walk);

            // Slow attack
            let mut attack = Animation::simple("attack", 6, 120, false);
            attack.attack_frame = Some(4);
            ctrl.add_animation(AnimationState::Attacking, *dir, attack);
        }

        ctrl.add_fallback(AnimationState::Dying, Animation::simple("death", 8, 180, false));

        ctrl
    }
}

/// Particle effect type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleType {
    Blood,
    Fire,
    Ice,
    Lightning,
    Magic,
    Spark,
    Smoke,
    Dust,
    Gold,
    Heal,
}

/// Single particle
#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub particle_type: ParticleType,
    pub color: (u8, u8, u8, u8),
    pub gravity: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32, particle_type: ParticleType) -> Self {
        let (color, lifetime, size, gravity) = match particle_type {
            ParticleType::Blood => ((200, 0, 0, 255), 0.5, 3.0, 200.0),
            ParticleType::Fire => ((255, 150, 50, 255), 0.8, 4.0, -50.0),
            ParticleType::Ice => ((100, 200, 255, 255), 0.6, 3.0, 50.0),
            ParticleType::Lightning => ((255, 255, 100, 255), 0.2, 2.0, 0.0),
            ParticleType::Magic => ((150, 100, 255, 255), 1.0, 4.0, -30.0),
            ParticleType::Spark => ((255, 255, 200, 255), 0.3, 2.0, 100.0),
            ParticleType::Smoke => ((100, 100, 100, 150), 1.5, 6.0, -20.0),
            ParticleType::Dust => ((150, 130, 100, 200), 0.8, 3.0, 50.0),
            ParticleType::Gold => ((255, 215, 0, 255), 0.6, 3.0, 150.0),
            ParticleType::Heal => ((100, 255, 100, 255), 1.2, 5.0, -40.0),
        };

        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            lifetime,
            max_lifetime: lifetime,
            size,
            particle_type,
            color,
            gravity,
        }
    }

    pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.vx = vx;
        self.vy = vy;
        self
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        if self.lifetime <= 0.0 {
            return false;
        }

        self.vy += self.gravity * dt;
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Fade out
        let alpha = (self.lifetime / self.max_lifetime * 255.0) as u8;
        self.color.3 = alpha;

        // Shrink some particles
        if matches!(self.particle_type, ParticleType::Fire | ParticleType::Smoke | ParticleType::Heal) {
            self.size *= 0.98;
        }

        true
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

/// Particle emitter for effects
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    pub x: f32,
    pub y: f32,
    pub particles: Vec<Particle>,
    pub particle_type: ParticleType,
    pub emission_rate: f32,
    pub emission_timer: f32,
    pub spread: f32,
    pub speed: f32,
    pub active: bool,
    pub one_shot: bool,
    pub duration: f32,
}

impl ParticleEmitter {
    pub fn new(x: f32, y: f32, particle_type: ParticleType) -> Self {
        Self {
            x,
            y,
            particles: Vec::new(),
            particle_type,
            emission_rate: 20.0,
            emission_timer: 0.0,
            spread: std::f32::consts::PI * 2.0,
            speed: 100.0,
            active: true,
            one_shot: false,
            duration: f32::MAX,
        }
    }

    /// Create burst effect (one-shot)
    pub fn burst(x: f32, y: f32, particle_type: ParticleType, count: u32) -> Self {
        let mut emitter = Self::new(x, y, particle_type);
        emitter.one_shot = true;
        emitter.active = false;

        // Create all particles at once
        let mut rng_seed = (x as u32).wrapping_mul(73856093) ^ (y as u32).wrapping_mul(19349663);

        for _ in 0..count {
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let angle = (rng_seed as f32 / u32::MAX as f32) * std::f32::consts::PI * 2.0;

            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let speed = emitter.speed * (0.5 + (rng_seed as f32 / u32::MAX as f32) * 0.5);

            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            emitter.particles.push(
                Particle::new(x, y, particle_type).with_velocity(vx, vy)
            );
        }

        emitter
    }

    pub fn update(&mut self, dt: f32) {
        self.duration -= dt;
        if self.duration <= 0.0 {
            self.active = false;
        }

        // Update existing particles
        self.particles.retain_mut(|p| p.update(dt));

        // Emit new particles
        if self.active && !self.one_shot {
            self.emission_timer += dt;
            let emit_interval = 1.0 / self.emission_rate;

            while self.emission_timer >= emit_interval {
                self.emission_timer -= emit_interval;
                self.emit_particle();
            }
        }
    }

    fn emit_particle(&mut self) {
        // Simple pseudo-random
        let seed = (self.particles.len() as u32).wrapping_mul(1103515245).wrapping_add(12345);
        let angle = (seed as f32 / u32::MAX as f32) * self.spread - self.spread / 2.0;

        let vx = angle.cos() * self.speed;
        let vy = angle.sin() * self.speed - self.speed * 0.5;

        self.particles.push(
            Particle::new(self.x, self.y, self.particle_type).with_velocity(vx, vy)
        );
    }

    pub fn is_finished(&self) -> bool {
        !self.active && self.particles.is_empty()
    }
}

/// Effect manager
pub struct EffectManager {
    pub emitters: Vec<ParticleEmitter>,
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            emitters: Vec::new(),
        }
    }

    pub fn spawn_blood(&mut self, x: f32, y: f32) {
        self.emitters.push(ParticleEmitter::burst(x, y, ParticleType::Blood, 10));
    }

    pub fn spawn_hit_sparks(&mut self, x: f32, y: f32) {
        self.emitters.push(ParticleEmitter::burst(x, y, ParticleType::Spark, 8));
    }

    pub fn spawn_magic(&mut self, x: f32, y: f32) {
        self.emitters.push(ParticleEmitter::burst(x, y, ParticleType::Magic, 15));
    }

    pub fn spawn_heal(&mut self, x: f32, y: f32) {
        let mut emitter = ParticleEmitter::new(x, y, ParticleType::Heal);
        emitter.duration = 0.5;
        emitter.emission_rate = 30.0;
        emitter.speed = 50.0;
        self.emitters.push(emitter);
    }

    pub fn spawn_gold_pickup(&mut self, x: f32, y: f32) {
        self.emitters.push(ParticleEmitter::burst(x, y, ParticleType::Gold, 5));
    }

    pub fn spawn_fire(&mut self, x: f32, y: f32, duration: f32) {
        let mut emitter = ParticleEmitter::new(x, y, ParticleType::Fire);
        emitter.duration = duration;
        emitter.emission_rate = 40.0;
        self.emitters.push(emitter);
    }

    pub fn update(&mut self, dt: f32) {
        for emitter in &mut self.emitters {
            emitter.update(dt);
        }
        self.emitters.retain(|e| !e.is_finished());
    }

    /// Get all particles for rendering
    pub fn all_particles(&self) -> impl Iterator<Item = &Particle> {
        self.emitters.iter().flat_map(|e| e.particles.iter())
    }
}
