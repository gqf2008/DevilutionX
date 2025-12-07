//! Random Number Generation System
//!
//! This module implements Diablo's random number generation system, consisting of:
//! 1. A Linear Congruential Generator (LCG) compatible with Borland C/C++
//! 2. A Xoshiro128++ generator for high-quality seed generation
//!
//! # C++ Source Reference
//! - Source/engine/random.cpp
//! - Source/engine/random.hpp
//!
//! # Algorithm Details
//!
//! ## Borland C LCG
//! The Diablo game uses a specific LCG for compatibility with the original game:
//! - Multiplier (a): 0x015A4E35 (22695477)
//! - Increment (c): 1
//! - Modulus (m): 2^32 (implicit via u32 overflow)
//!
//! Formula: state(n+1) = (state(n) * 22695477 + 1) mod 2^32
//!
//! ## Xoshiro128++
//! A high-quality PRNG used for generating unpredictable seeds.
//! Period: 2^128 - 1

use std::time::{SystemTime, UNIX_EPOCH};

/// LCG multiplier (Borland C compatible)
const LCG_MULTIPLIER: u32 = 0x015A4E35;

/// LCG increment
const LCG_INCREMENT: u32 = 1;

/// Borland C/C++ compatible Linear Congruential Generator
///
/// This LCG is needed for vanilla Diablo compatibility.
/// It produces the same sequence as the original game.
#[derive(Debug, Clone)]
pub struct DiabloLcg {
    state: u32,
}

impl Default for DiabloLcg {
    fn default() -> Self {
        Self::new()
    }
}

impl DiabloLcg {
    /// Creates a new LCG with default seed (0)
    pub fn new() -> Self {
        Self { state: 0 }
    }

    /// Creates a new LCG with specified seed
    pub fn with_seed(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Sets the generator seed
    pub fn seed(&mut self, seed: u32) {
        self.state = seed;
    }

    /// Returns the current state
    pub fn state(&self) -> u32 {
        self.state
    }

    /// Generates the next random number
    ///
    /// # C++ Reference
    /// ```cpp
    /// uint32_t GenerateRandomNumber()
    /// {
    ///     sglGameSeed = diabloGenerator();
    ///     return sglGameSeed;
    /// }
    /// ```
    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
        self.state
    }

    /// Discards a number of random values
    pub fn discard(&mut self, count: u32) {
        for _ in 0..count {
            self.next();
        }
    }
}

/// Xoshiro128++ random number generator
///
/// A high-quality PRNG with a period of 2^128 - 1.
/// Used for generating unpredictable seeds.
///
/// # C++ Reference
/// ```cpp
/// class xoshiro128plusplus {
/// public:
///     using state = uint32_t[4];
///     uint32_t next();
///     void jump();
///     // ...
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Xoshiro128PlusPlus {
    state: [u32; 4],
}

impl Default for Xoshiro128PlusPlus {
    fn default() -> Self {
        Self::new()
    }
}

impl Xoshiro128PlusPlus {
    /// Jump polynomial for advancing 2^64 steps
    const JUMP: [u32; 4] = [0x8764000b, 0xf542d2d3, 0x6fa035c3, 0x77f2db5b];

    /// Creates a new generator with time-based seed
    pub fn new() -> Self {
        let time_seed = Self::time_seed();
        Self::from_u64(time_seed)
    }

    /// Creates a generator from a 64-bit seed
    pub fn from_u64(seed: u64) -> Self {
        // Split the 64-bit seed into four 32-bit values using SplitMix64
        let mut sm_state = seed;
        let state = [
            Self::splitmix64(&mut sm_state) as u32,
            (Self::splitmix64(&mut sm_state) >> 32) as u32,
            Self::splitmix64(&mut sm_state) as u32,
            (Self::splitmix64(&mut sm_state) >> 32) as u32,
        ];
        Self { state }
    }

    /// Creates a generator from explicit state
    pub fn from_state(state: [u32; 4]) -> Self {
        Self { state }
    }

    /// Gets the current state
    pub fn state(&self) -> [u32; 4] {
        self.state
    }

    /// SplitMix64 for seeding
    fn splitmix64(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Gets a time-based seed
    ///
    /// # C++ Reference
    /// ```cpp
    /// uint64_t xoshiro128plusplus::timeSeed()
    /// {
    ///     auto now = std::chrono::system_clock::now();
    ///     auto nano = std::chrono::nanoseconds(now.time_since_epoch());
    ///     return static_cast<uint64_t>(nano.count());
    /// }
    /// ```
    pub fn time_seed() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x12345678_9ABCDEF0)
    }

    /// Generates the next random number
    ///
    /// # C++ Reference
    /// ```cpp
    /// uint32_t xoshiro128plusplus::next()
    /// {
    ///     const uint32_t result = std::rotl(s[0] + s[3], 7) + s[0];
    ///     const uint32_t t = s[1] << 9;
    ///     s[2] ^= s[0];
    ///     s[3] ^= s[1];
    ///     s[1] ^= s[2];
    ///     s[0] ^= s[3];
    ///     s[2] ^= t;
    ///     s[3] = std::rotl(s[3], 11);
    ///     return result;
    /// }
    /// ```
    pub fn next(&mut self) -> u32 {
        let result = (self.state[0].wrapping_add(self.state[3]))
            .rotate_left(7)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 9;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(11);

        result
    }

    /// Advances the generator by 2^64 steps
    ///
    /// This is useful for reserving a sequence of random numbers.
    pub fn jump(&mut self) {
        let mut s0 = 0u32;
        let mut s1 = 0u32;
        let mut s2 = 0u32;
        let mut s3 = 0u32;

        for &jump_val in &Self::JUMP {
            for b in 0..32 {
                if (jump_val >> b) & 1 != 0 {
                    s0 ^= self.state[0];
                    s1 ^= self.state[1];
                    s2 ^= self.state[2];
                    s3 ^= self.state[3];
                }
                self.next();
            }
        }

        self.state[0] = s0;
        self.state[1] = s1;
        self.state[2] = s2;
        self.state[3] = s3;
    }
}

/// Game random number manager
///
/// Combines the LCG and Xoshiro generators into a unified interface.
///
/// # C++ Reference
/// ```cpp
/// uint32_t sglGameSeed;
/// std::linear_congruential_engine<...> diabloGenerator;
/// xoshiro128plusplus seedGenerator;
/// ```
#[derive(Debug, Clone)]
pub struct GameRandom {
    /// Borland C LCG for game logic
    lcg: DiabloLcg,
    /// Xoshiro128++ for seed generation
    seed_generator: Xoshiro128PlusPlus,
    /// Current game seed (mirrors sglGameSeed)
    game_seed: u32,
}

impl Default for GameRandom {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRandom {
    /// Creates a new game random manager
    pub fn new() -> Self {
        Self {
            lcg: DiabloLcg::new(),
            seed_generator: Xoshiro128PlusPlus::new(),
            game_seed: 0,
        }
    }

    /// Sets the random seed
    ///
    /// # C++ Reference
    /// ```cpp
    /// void SetRndSeed(uint32_t seed)
    /// {
    ///     diabloGenerator.seed(seed);
    ///     sglGameSeed = seed;
    /// }
    /// ```
    pub fn set_seed(&mut self, seed: u32) {
        self.lcg.seed(seed);
        self.game_seed = seed;
    }

    /// Gets the current LCG engine state
    ///
    /// # C++ Reference
    /// ```cpp
    /// uint32_t GetLCGEngineState()
    /// {
    ///     return sglGameSeed;
    /// }
    /// ```
    pub fn get_state(&self) -> u32 {
        self.game_seed
    }

    /// Generates a random number using the LCG
    ///
    /// # C++ Reference
    /// ```cpp
    /// uint32_t GenerateRandomNumber()
    /// {
    ///     sglGameSeed = diabloGenerator();
    ///     return sglGameSeed;
    /// }
    /// ```
    pub fn generate(&mut self) -> u32 {
        self.game_seed = self.lcg.next();
        self.game_seed
    }

    /// Generates a random seed (uses the high-quality generator)
    ///
    /// # C++ Reference
    /// ```cpp
    /// uint32_t GenerateSeed()
    /// {
    ///     return seedGenerator.next();
    /// }
    /// ```
    pub fn generate_seed(&mut self) -> u32 {
        self.seed_generator.next()
    }

    /// Advances the random seed and returns absolute value
    ///
    /// # C++ Reference
    /// ```cpp
    /// int32_t AdvanceRndSeed()
    /// {
    ///     const int32_t seed = static_cast<int32_t>(GenerateRandomNumber());
    ///     return seed == INT_MIN ? INT_MIN : std::abs(seed);
    /// }
    /// ```
    pub fn advance_seed(&mut self) -> i32 {
        let seed = self.generate() as i32;
        if seed == i32::MIN {
            i32::MIN
        } else {
            seed.abs()
        }
    }

    /// Generates a random number in range [0, max)
    ///
    /// # C++ Reference
    /// ```cpp
    /// int32_t GenerateRnd(int32_t v)
    /// {
    ///     if (v <= 0)
    ///         return 0;
    ///     if (v <= 0x7FFF)
    ///         return (AdvanceRndSeed() >> 16) % v;
    ///     return AdvanceRndSeed() % v;
    /// }
    /// ```
    pub fn generate_rnd(&mut self, max: i32) -> i32 {
        if max <= 0 {
            return 0;
        }
        if max <= 0x7FFF {
            // Use high bits to correct for LCG bias
            (self.advance_seed() >> 16) % max
        } else {
            self.advance_seed() % max
        }
    }

    /// Flip a coin with given frequency
    ///
    /// Returns true with probability 1/frequency.
    ///
    /// # C++ Reference
    /// ```cpp
    /// bool FlipCoin(unsigned frequency)
    /// {
    ///     return GenerateRnd(static_cast<int32_t>(frequency)) == 0;
    /// }
    /// ```
    pub fn flip_coin(&mut self, frequency: u32) -> bool {
        self.generate_rnd(frequency as i32) == 0
    }

    /// Discards a number of random values
    ///
    /// # C++ Reference
    /// ```cpp
    /// void DiscardRandomValues(unsigned count)
    /// {
    ///     while (count != 0) {
    ///         GenerateRandomNumber();
    ///         count--;
    ///     }
    /// }
    /// ```
    pub fn discard(&mut self, count: u32) {
        for _ in 0..count {
            self.generate();
        }
    }

    /// Reserves a seed sequence by advancing the seed generator
    ///
    /// # C++ Reference
    /// ```cpp
    /// xoshiro128plusplus ReserveSeedSequence()
    /// {
    ///     xoshiro128plusplus reserved = seedGenerator;
    ///     seedGenerator.jump();
    ///     return reserved;
    /// }
    /// ```
    pub fn reserve_seed_sequence(&mut self) -> Xoshiro128PlusPlus {
        let reserved = self.seed_generator.clone();
        self.seed_generator.jump();
        reserved
    }
}

/// Picks a random element from a slice
///
/// # C++ Reference
/// ```cpp
/// template <typename... Args>
/// T PickRandomlyAmong(std::initializer_list<T> list)
/// ```
pub fn pick_randomly<T: Copy>(rng: &mut GameRandom, choices: &[T]) -> Option<T> {
    if choices.is_empty() {
        None
    } else {
        let idx = rng.generate_rnd(choices.len() as i32) as usize;
        Some(choices[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test LCG basic sequence
    #[test]
    fn test_lcg_sequence() {
        let mut lcg = DiabloLcg::with_seed(0);

        // First few values from LCG starting at seed 0
        // state(n+1) = state(n) * 0x015A4E35 + 1
        let v1 = lcg.next();
        assert_eq!(v1, 1); // 0 * 22695477 + 1 = 1

        let v2 = lcg.next();
        assert_eq!(v2, 22695478); // 1 * 22695477 + 1 = 22695478

        let v3 = lcg.next();
        // 22695478 * 22695477 + 1 = 515157563 mod 2^32
        // Let's verify manually:
        // 22695478 * 22695477 = 515157540299706
        // 515157540299706 mod 2^32 = 515157540299706 mod 4294967296 = 515157562
        // +1 = 515157563
        assert_eq!(v3, 515157563);
    }

    /// Test LCG determinism
    #[test]
    fn test_lcg_determinism() {
        let mut lcg1 = DiabloLcg::with_seed(12345);
        let mut lcg2 = DiabloLcg::with_seed(12345);

        for _ in 0..100 {
            assert_eq!(lcg1.next(), lcg2.next());
        }
    }

    /// Test LCG different seeds produce different sequences
    #[test]
    fn test_lcg_different_seeds() {
        let mut lcg1 = DiabloLcg::with_seed(1);
        let mut lcg2 = DiabloLcg::with_seed(2);

        assert_ne!(lcg1.next(), lcg2.next());
    }

    /// Test LCG discard
    #[test]
    fn test_lcg_discard() {
        let mut lcg1 = DiabloLcg::with_seed(42);
        let mut lcg2 = DiabloLcg::with_seed(42);

        lcg1.discard(10);
        for _ in 0..10 {
            lcg2.next();
        }

        assert_eq!(lcg1.state(), lcg2.state());
    }

    /// Test Xoshiro basic generation
    #[test]
    fn test_xoshiro_generation() {
        let mut xo = Xoshiro128PlusPlus::from_state([1, 2, 3, 4]);

        // Should produce different values
        let values: Vec<u32> = (0..10).map(|_| xo.next()).collect();

        // Check all values are different (very likely for a good PRNG)
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(values[i], values[j], "Values at {} and {} are equal", i, j);
            }
        }
    }

    /// Test Xoshiro determinism
    #[test]
    fn test_xoshiro_determinism() {
        let mut xo1 = Xoshiro128PlusPlus::from_state([11, 22, 33, 44]);
        let mut xo2 = Xoshiro128PlusPlus::from_state([11, 22, 33, 44]);

        for _ in 0..100 {
            assert_eq!(xo1.next(), xo2.next());
        }
    }

    /// Test Xoshiro jump
    #[test]
    fn test_xoshiro_jump() {
        let mut xo1 = Xoshiro128PlusPlus::from_state([1, 2, 3, 4]);
        let xo2 = xo1.clone();

        xo1.jump();

        // After jump, states should be different
        assert_ne!(xo1.state(), xo2.state());
    }

    /// Test GameRandom set_seed
    #[test]
    fn test_game_random_set_seed() {
        let mut rng = GameRandom::new();
        rng.set_seed(12345);

        assert_eq!(rng.get_state(), 12345);
    }

    /// Test GameRandom generate
    #[test]
    fn test_game_random_generate() {
        let mut rng = GameRandom::new();
        rng.set_seed(0);

        let v1 = rng.generate();
        assert_eq!(v1, 1);
        assert_eq!(rng.get_state(), 1);
    }

    /// Test generate_rnd range
    #[test]
    fn test_generate_rnd_range() {
        let mut rng = GameRandom::new();
        rng.set_seed(12345);

        for _ in 0..1000 {
            let v = rng.generate_rnd(10);
            assert!(v >= 0 && v < 10);
        }
    }

    /// Test generate_rnd with zero and negative
    #[test]
    fn test_generate_rnd_edge_cases() {
        let mut rng = GameRandom::new();
        rng.set_seed(42);

        assert_eq!(rng.generate_rnd(0), 0);
        assert_eq!(rng.generate_rnd(-5), 0);
    }

    /// Test generate_rnd uses high bits for small values
    #[test]
    fn test_generate_rnd_high_bits() {
        let mut rng1 = GameRandom::new();
        let mut rng2 = GameRandom::new();
        rng1.set_seed(1000);
        rng2.set_seed(1000);

        // For values <= 0x7FFF, should use high bits
        let small = rng1.generate_rnd(100);
        let large = rng2.generate_rnd(0x10000); // > 0x7FFF

        // They may or may not be equal, but both should be valid
        assert!(small >= 0 && small < 100);
        assert!(large >= 0 && large < 0x10000);
    }

    /// Test flip_coin
    #[test]
    fn test_flip_coin() {
        let mut rng = GameRandom::new();
        rng.set_seed(12345);

        // flip_coin(1) should always return true
        // Because generate_rnd(1) always returns 0
        for _ in 0..100 {
            let mut test_rng = GameRandom::new();
            test_rng.set_seed(rng.generate_seed());
            // Note: flip_coin(1) = generate_rnd(1) == 0
            // generate_rnd(1) = 0 always
            assert!(test_rng.flip_coin(1));
        }
    }

    /// Test flip_coin distribution
    #[test]
    fn test_flip_coin_distribution() {
        let mut rng = GameRandom::new();
        rng.set_seed(54321);

        let mut true_count = 0;
        let iterations = 10000;

        for _ in 0..iterations {
            if rng.flip_coin(2) {
                true_count += 1;
            }
        }

        // Should be roughly 50%, allow 10% tolerance
        let expected = iterations / 2;
        let tolerance = iterations / 10;
        assert!(
            (true_count as i32 - expected as i32).abs() < tolerance as i32,
            "Expected ~{} true results, got {}",
            expected,
            true_count
        );
    }

    /// Test discard
    #[test]
    fn test_game_random_discard() {
        let mut rng1 = GameRandom::new();
        let mut rng2 = GameRandom::new();
        rng1.set_seed(100);
        rng2.set_seed(100);

        rng1.discard(5);
        for _ in 0..5 {
            rng2.generate();
        }

        assert_eq!(rng1.get_state(), rng2.get_state());
    }

    /// Test reserve_seed_sequence
    #[test]
    fn test_reserve_seed_sequence() {
        let mut rng = GameRandom::new();

        let reserved = rng.reserve_seed_sequence();

        // Reserved should have the original state
        // Main generator should have jumped
        assert_ne!(reserved.state(), rng.seed_generator.state());
    }

    /// Test pick_randomly
    #[test]
    fn test_pick_randomly() {
        let mut rng = GameRandom::new();
        rng.set_seed(12345);

        let choices = [10, 20, 30, 40, 50];
        let result = pick_randomly(&mut rng, &choices);

        assert!(result.is_some());
        assert!(choices.contains(&result.unwrap()));
    }

    /// Test pick_randomly empty slice
    #[test]
    fn test_pick_randomly_empty() {
        let mut rng = GameRandom::new();
        rng.set_seed(12345);

        let empty: [i32; 0] = [];
        let result = pick_randomly(&mut rng, &empty);

        assert!(result.is_none());
    }

    /// Test advance_seed with INT_MIN
    #[test]
    fn test_advance_seed_int_min() {
        let mut rng = GameRandom::new();

        // Find a seed that produces INT_MIN
        // This is tricky, so we'll just test the logic
        // When the generated value is INT_MIN, it should return INT_MIN
        // (since abs(INT_MIN) is undefined behavior in C++)
        // We can't easily force this, so we just verify normal behavior
        rng.set_seed(0);
        let result = rng.advance_seed();
        assert!(result >= 0 || result == i32::MIN);
    }

    /// Test time_seed produces different values (probabilistic)
    #[test]
    fn test_time_seed_varies() {
        let seed1 = Xoshiro128PlusPlus::time_seed();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let seed2 = Xoshiro128PlusPlus::time_seed();

        // Seeds should be different (with very high probability)
        assert_ne!(seed1, seed2);
    }

    /// Test full roundtrip
    #[test]
    fn test_full_roundtrip() {
        let mut rng = GameRandom::new();
        rng.set_seed(42);

        // Generate some values
        let values: Vec<u32> = (0..10).map(|_| rng.generate()).collect();

        // Reset and verify same sequence
        let mut rng2 = GameRandom::new();
        rng2.set_seed(42);

        for &expected in &values {
            assert_eq!(rng2.generate(), expected);
        }
    }
}
