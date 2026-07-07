//! Diablo deterministic pseudo-random number generator.
//!
//! Ports the seeded RNG used by the original Diablo / DevilutionX engine so
//! that level/item generation is bit-for-bit reproducible for the same seed
//! (required for save/replay compatibility).
//!
//! C++ Reference: `Source/engine/random.cpp` and `Source/engine/random.hpp`.
//!
//! The engine is the classic Borland C/C++ linear congruential generator
//! (`std::linear_congruential_engine<uint32_t, 0x015A4E35, 1, 0>`):
//!
//! ```text
//! GenerateRandomNumber():  sglGameSeed = sglGameSeed * 0x015A4E35 + 1  (mod 2^32)
//! ```
//!
//! `AdvanceRndSeed()` reinterprets that `uint32_t` as a signed `int32_t` and
//! applies `abs()` (the only reason the high bit is discarded most of the time;
//! `abs(INT_MIN)` would be UB so it is returned unchanged). `GenerateRnd(v)`
//! then reduces that value modulo `v`, using the high bits for small limits to
//! partially correct for LCG bias.

/// Multiplier of the Borland LCG (matches `std::linear_congruential_engine`
/// with `a = 0x015A4E35, c = 1, m = 2^32`).
const LCG_MULT: u32 = 0x015A4E35;
/// Increment of the Borland LCG.
const LCG_INC: u32 = 1;

/// Limit below which `GenerateRnd` uses the high bits to reduce LCG bias.
/// Matches the C++ constant `0x7FFF`.
const RND_HIGH_BITS_LIMIT: i32 = 0x7FFF;

/// The original `i32::MIN`, returned as-is by `AdvanceRndSeed` to avoid the
/// undefined `abs(INT_MIN)` from the C++ source.
const ADV_MIN: i32 = i32::MIN;

/// A deterministic Diablo-compatible RNG.
///
/// Holds the single global-style seed (`sglGameSeed`). This mirrors the
/// engine state in `Source/engine/random.cpp`. All methods are deterministic:
/// two `Rng` values seeded identically produce identical sequences.
#[derive(Debug, Clone, Copy)]
pub struct Rng {
    /// Current LCG state, equivalent to the C++ global `sglGameSeed`.
    sgl_game_seed: u32,
}

impl Rng {
    /// Create a new RNG with the given initial seed.
    ///
    /// Equivalent to calling [`set_seed`](Self::set_seed) after construction.
    #[inline]
    pub fn new(seed: u32) -> Self {
        Self {
            sgl_game_seed: seed,
        }
    }

    /// Create a new RNG with a default seed of `0`.
    ///
    /// The real game seeds the generator explicitly via [`set_seed`](Self::set_seed)
    /// before each generation pass, so the initial value here rarely matters.
    /// It exists mostly for `Default` / convenience.
    #[inline]
    pub fn with_default_seed() -> Self {
        Self::new(0)
    }

    /// Set the engine state to `seed`.
    ///
    /// Mirrors C++ `SetRndSeed`.
    #[inline]
    pub fn set_seed(&mut self, seed: u32) {
        self.sgl_game_seed = seed;
    }

    /// Return the current engine state without advancing it.
    ///
    /// Mirrors C++ `GetLCGEngineState`.
    #[inline]
    pub fn get_seed(&self) -> u32 {
        self.sgl_game_seed
    }

    /// Advance the engine and return the new `uint32_t` state.
    ///
    /// Mirrors C++ `GenerateRandomNumber`.
    #[inline]
    pub fn generate_random_number(&mut self) -> u32 {
        self.sgl_game_seed = self
            .sgl_game_seed
            .wrapping_mul(LCG_MULT)
            .wrapping_add(LCG_INC);
        self.sgl_game_seed
    }

    /// Advance the engine and return the result reinterpreted as a signed
    /// integer with `abs` applied.
    ///
    /// Mirrors C++ `AdvanceRndSeed`. This usually returns a non-negative value
    /// in `[0, 2^31)` but can very rarely return `i32::MIN` (when the new
    /// state is exactly `0x80000000`); that case is preserved unchanged instead
    /// of invoking undefined `abs`.
    #[inline]
    pub fn advance_rnd_seed(&mut self) -> i32 {
        let raw = self.generate_random_number();
        // Reinterpret bits as signed (matches C++ `static_cast<int32_t>`).
        let signed = raw as i32;
        if signed == ADV_MIN {
            ADV_MIN
        } else {
            signed.wrapping_abs()
        }
    }

    /// Generate a random integer strictly less than `max_value`.
    ///
    /// Mirrors C++ `GenerateRnd(int32_t v)`:
    /// - `max_value <= 0` returns `0` (and does *not* advance the engine).
    /// - `max_value <= 0x7FFF` uses the high 15 bits of `AdvanceRndSeed` to
    ///   reduce LCG bias: `(advance >> 16) % max_value`.
    /// - otherwise uses the full value: `advance % max_value`.
    ///
    /// The shift is arithmetic (operates on the signed `AdvanceRndSeed`
    /// result), and `%` preserves the sign of the dividend, so this can in
    /// principle return a small negative value when `AdvanceRndSeed` happens to
    /// be `i32::MIN` — matching the documented C++ behaviour.
    #[inline]
    pub fn generate(&mut self, max_value: i32) -> i32 {
        if max_value <= 0 {
            return 0;
        }
        let advance = self.advance_rnd_seed();
        if max_value <= RND_HIGH_BITS_LIMIT {
            // Arithmetic shift right on a signed value (matches C++ `>> 16`).
            (advance >> 16) % max_value
        } else {
            advance % max_value
        }
    }

    /// Return `true` with probability `percent / 100`.
    ///
    /// Matches the percentile checks used throughout `drlg_l2.cpp` of the form
    /// `GenerateRnd(100) < percent` (e.g. corridor steering in `ConnectHall`
    /// and the `fMinusFlag` / `fPlusFlag` flags). With a deterministic engine
    /// the outcome is reproducible for a given seed.
    #[inline]
    pub fn random_chance(&mut self, percent: u32) -> bool {
        // `GenerateRnd(100)` is in [0, 99] (ignoring the rare i32::MIN glitch);
        // comparing `< percent` gives the desired `percent%` chance.
        (self.generate(100)) < percent as i32
    }

    /// Return a random non-negative index in `[0, max_exclusive)`.
    ///
    /// Convenience wrapper clamping the (rare) negative glitch of
    /// [`generate`](Self::generate) to `0`, mirroring C++ `RandomIntLessThan`.
    /// `max_exclusive <= 0` yields `0` without advancing the engine.
    #[inline]
    pub fn random_less_than(&mut self, max_exclusive: i32) -> i32 {
        std::cmp::max(0, self.generate(max_exclusive))
    }
}

impl Default for Rng {
    #[inline]
    fn default() -> Self {
        Self::with_default_seed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C++ `GenerateRandomNumber()` sequence produced from `SetRndSeed(1)`
    /// (verified against the reference implementation).
    const REF_GEN_NUM_SEED1: [u32; 10] = [
        22695478,
        2156045615,
        2867233980,
        71484141,
        2911408402,
        2613937339,
        1153135800,
        420428313,
        1503962414,
        4187371143,
    ];

    /// C++ `AdvanceRndSeed()` sequence produced from `SetRndSeed(1)`
    /// (note `abs` applied to the signed reinterpretation).
    const REF_ADV_SEED1: [i32; 10] = [
        22695478,
        2138921681,
        1427733316,
        71484141,
        1383558894,
        1681029957,
        1153135800,
        420428313,
        1503962414,
        107596153,
    ];

    /// C++ `GenerateRnd(100)` sequence produced from `SetRndSeed(1)`.
    const REF_GEN_100_SEED1: [i32; 10] = [
        46, 37, 85, 90, 11, 50, 95, 15, 48, 41,
    ];

    #[test]
    fn test_generate_random_number_matches_cpp_seed1() {
        let mut rng = Rng::new(1);
        for &expected in &REF_GEN_NUM_SEED1 {
            assert_eq!(rng.generate_random_number(), expected);
        }
    }

    #[test]
    fn test_advance_rnd_seed_matches_cpp_seed1() {
        let mut rng = Rng::new(1);
        for &expected in &REF_ADV_SEED1 {
            assert_eq!(rng.advance_rnd_seed(), expected);
        }
    }

    #[test]
    fn test_generate_100_matches_cpp_seed1() {
        let mut rng = Rng::new(1);
        for &expected in &REF_GEN_100_SEED1 {
            assert_eq!(rng.generate(100), expected);
        }
    }

    #[test]
    fn test_determinism_same_seed_same_sequence() {
        // Two independent RNGs with the same seed must produce identical
        // sequences regardless of how many values are drawn.
        let mut a = Rng::new(0x12345678);
        let mut b = Rng::new(0x12345678);
        for _ in 0..50 {
            assert_eq!(a.generate(100), b.generate(100));
        }

        // Different seeds (with overwhelming probability) diverge immediately.
        let mut c = Rng::new(1);
        let mut d = Rng::new(2);
        let diverged = (0..20).any(|_| c.generate(1000) != d.generate(1000));
        assert!(diverged, "different seeds should not stay in lockstep");
    }

    #[test]
    fn test_set_and_get_seed() {
        let mut rng = Rng::new(0);
        rng.set_seed(42);
        assert_eq!(rng.get_seed(), 42);
        // set_seed fully resets the stream.
        let mut r1 = Rng::new(7);
        let mut r2 = Rng::new(0);
        r2.set_seed(7);
        for _ in 0..10 {
            assert_eq!(r1.generate_random_number(), r2.generate_random_number());
        }
    }

    #[test]
    fn test_generate_zero_and_negative_return_zero_without_advancing() {
        let mut rng = Rng::new(1);
        // Neither call should consume engine state.
        assert_eq!(rng.generate(0), 0);
        assert_eq!(rng.generate(-1), 0);
        assert_eq!(rng.generate(-1000), 0);
        // Engine still at the initial seed.
        assert_eq!(rng.get_seed(), 1);
        // The next real draw still matches the reference sequence.
        assert_eq!(rng.generate_random_number(), REF_GEN_NUM_SEED1[0]);
    }

    #[test]
    fn test_generate_range_bounds() {
        let mut rng = Rng::new(1);
        // All draws must land in [0, 100) (the documented glitch would need the
        // engine to land on exactly i32::MIN, which never occurs for seed 1
        // within this window — assert the practical contract).
        for _ in 0..1000 {
            let v = rng.generate(100);
            assert!(
                (0..100).contains(&v),
                "GenerateRnd(100) returned {v}, out of [0,100)"
            );
        }

        // Small limit stays in range too.
        let mut rng = Rng::new(99);
        for _ in 0..500 {
            let v = rng.generate(7);
            assert!((0..7).contains(&v), "GenerateRnd(7) returned {v}");
        }
    }

    #[test]
    fn test_random_chance_extremes() {
        // percent == 0 must never succeed.
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            assert!(!rng.random_chance(0));
        }

        // percent == 100 must always succeed.
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            assert!(rng.random_chance(100));
        }

        // percent > 100 also always succeeds (matches `GenerateRnd(100) < p`).
        let mut rng = Rng::new(1);
        for _ in 0..100 {
            assert!(rng.random_chance(150));
        }
    }

    #[test]
    fn test_random_chance_50_is_about_half() {
        // Statistical sanity check (deterministic, but the LCG spreads ~half).
        let mut rng = Rng::new(1);
        let mut hits = 0u32;
        let n = 10_000u32;
        for _ in 0..n {
            if rng.random_chance(50) {
                hits += 1;
            }
        }
        // Allow a wide band; this is only guarding against gross regressions.
        let ratio = hits as f64 / n as f64;
        assert!(
            (0.45..0.55).contains(&ratio),
            "random_chance(50) hit ratio {ratio:.3} is far from 0.5"
        );
    }

    #[test]
    fn test_random_less_than_is_non_negative() {
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            let v = rng.random_less_than(38);
            assert!((0..38).contains(&v), "random_less_than(38) returned {v}");
        }
    }

    #[test]
    fn test_large_limit_uses_full_modulo() {
        // Limits above 0x7FFF bypass the high-bit shift and use the full
        // AdvanceRndSeed value modulo `v` instead of `(advance >> 16) % v`.
        // From seed 1, AdvanceRndSeed() == 22695478, so the two paths are:
        let mut rng = Rng::new(1);
        let small_path = rng.generate(0x7FFF); // (22695478 >> 16) % 0x7FFF
        rng.set_seed(1);
        let big_path = rng.generate(100_000); // 22695478 % 100000
        assert_eq!(small_path, (22695478i32 >> 16) % 0x7FFF);
        assert_eq!(big_path, 22695478 % 100_000);
        assert!(small_path >= 0 && small_path < 0x7FFF);
        assert!(big_path >= 0 && big_path < 100_000);
    }
}
