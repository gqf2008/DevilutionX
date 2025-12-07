//! Random - 随机数生成
//!
//! 移植自 Source/engine/random.hpp
//!
//! 包含与原版 Diablo 兼容的随机数生成器

/// Diablo 原版随机数生成器 (Borland C/C++ LCG)
///
/// 用于需要与原版游戏兼容的逻辑
#[derive(Clone, Debug)]
pub struct DiabloGenerator {
    state: u32,
}

impl DiabloGenerator {
    /// LCG 参数 (Borland C/C++)
    const MULTIPLIER: u32 = 0x015A4E35;
    const INCREMENT: u32 = 1;

    /// 创建新的生成器
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// 获取当前状态
    pub fn state(&self) -> u32 {
        self.state
    }

    /// 设置种子
    pub fn seed(&mut self, seed: u32) {
        self.state = seed;
    }

    /// 丢弃指定数量的随机值
    pub fn discard(&mut self, count: u32) {
        for _ in 0..count {
            self.next_raw();
        }
    }

    /// 生成下一个原始值
    #[inline]
    fn next_raw(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::INCREMENT);
        self.state
    }

    /// 生成一个非负整数（通常）
    ///
    /// 返回范围 [0, 2^31) 或极少数情况下返回 -2^31
    pub fn advance_rnd_seed(&mut self) -> i32 {
        let seed = self.next_raw() as i32;
        if seed == i32::MIN {
            i32::MIN
        } else {
            seed.abs()
        }
    }

    /// 生成一个小于给定限制的随机整数
    ///
    /// 如果 v <= 0，返回 0
    /// 由于原版 bug，限制在 32768-65534 范围内会有问题
    pub fn generate_rnd(&mut self, v: i32) -> i32 {
        if v <= 0 {
            return 0;
        }
        if v <= 0x7FFF {
            // 使用高位来修正 LCG 偏差
            ((self.advance_rnd_seed() >> 16) % v).abs()
        } else {
            (self.advance_rnd_seed() % v).abs()
        }
    }

    /// 生成随机布尔值
    ///
    /// frequency 为 2 时表示 50/50 概率
    pub fn flip_coin(&mut self, frequency: u32) -> bool {
        self.generate_rnd(frequency as i32) == 0
    }

    /// 生成一个非负整数
    pub fn random_int_less_than(&mut self, v: i32) -> i32 {
        self.generate_rnd(v).max(0)
    }

    /// 生成指定范围内的随机整数
    ///
    /// half_open: true 表示半开区间 [min, max)，false 表示闭区间 [min, max]
    pub fn random_int_between(&mut self, min: i32, max: i32, half_open: bool) -> i32 {
        self.random_int_less_than(max - min + if half_open { 0 } else { 1 }) + min
    }
}

/// SplitMix32 生成器
///
/// 用于快速生成种子序列
#[derive(Clone, Debug)]
pub struct SplitMix32 {
    state: u32,
}

impl SplitMix32 {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x9e3779b9);
        let mut z = self.state;
        z = (z ^ (z >> 16)).wrapping_mul(0x85ebca6b);
        z = (z ^ (z >> 13)).wrapping_mul(0xc2b2ae35);
        z ^ (z >> 16)
    }

    pub fn generate(&mut self, count: usize) -> Vec<u32> {
        (0..count).map(|_| self.next()).collect()
    }
}

/// SplitMix64 生成器
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    pub fn generate(&mut self, count: usize) -> Vec<u64> {
        (0..count).map(|_| self.next()).collect()
    }
}

/// Xoshiro128++ 生成器
///
/// 高质量随机数生成器，用于非原版兼容的逻辑
#[derive(Clone, Debug)]
pub struct Xoshiro128PlusPlus {
    state: [u32; 4],
}

impl Xoshiro128PlusPlus {
    /// 从 64 位种子创建
    pub fn new(seed: u64) -> Self {
        let mut sm = SplitMix64::new(seed);
        let seeds = sm.generate(2);

        Self {
            state: [
                (seeds[0] >> 32) as u32,
                seeds[0] as u32,
                (seeds[1] >> 32) as u32,
                seeds[1] as u32,
            ],
        }
    }

    /// 从 32 位种子创建
    pub fn from_u32(seed: u32) -> Self {
        let mut sm = SplitMix32::new(seed);
        let seeds = sm.generate(4);

        Self {
            state: [seeds[0], seeds[1], seeds[2], seeds[3]],
        }
    }

    /// 从状态数组创建
    pub fn from_state(state: [u32; 4]) -> Self {
        Self { state }
    }

    /// 获取当前状态
    pub fn state(&self) -> [u32; 4] {
        self.state
    }

    /// 生成下一个随机数
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

    /// 跳跃函数 - 相当于 2^64 次 next() 调用
    pub fn jump(&mut self) {
        const JUMP: [u32; 4] = [0x8764000b, 0xf542d2d3, 0x6fa035c3, 0x77f2db5b];

        let mut s0 = 0u32;
        let mut s1 = 0u32;
        let mut s2 = 0u32;
        let mut s3 = 0u32;

        for entry in JUMP {
            for b in 0..32 {
                if entry & (1 << b) != 0 {
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

    /// 生成一个范围内的随机数 [0, max)
    pub fn random_int_less_than(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        self.next() % max
    }

    /// 生成一个范围内的随机数 [min, max]
    pub fn random_int_between(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next() % ((max - min + 1) as u32)) as i32
    }

    /// 生成随机布尔值
    pub fn flip_coin(&mut self) -> bool {
        self.next() & 1 == 0
    }

    /// 生成指定概率的布尔值 (1/frequency 的概率为 true)
    pub fn flip_coin_weighted(&mut self, frequency: u32) -> bool {
        if frequency == 0 {
            return false;
        }
        self.random_int_less_than(frequency) == 0
    }
}

/// 全局 Diablo 随机数生成器（用于原版兼容）
use std::sync::Mutex;

static GLOBAL_DIABLO_RNG: Mutex<DiabloGenerator> = Mutex::new(DiabloGenerator { state: 0 });
static GLOBAL_SEED_RNG: Mutex<Option<Xoshiro128PlusPlus>> = Mutex::new(None);

/// 设置全局随机种子
pub fn set_rnd_seed(seed: u32) {
    if let Ok(mut rng) = GLOBAL_DIABLO_RNG.lock() {
        rng.seed(seed);
    }
}

/// 获取当前 LCG 引擎状态
pub fn get_lcg_engine_state() -> u32 {
    GLOBAL_DIABLO_RNG.lock().map(|rng| rng.state()).unwrap_or(0)
}

/// 丢弃随机值
pub fn discard_random_values(count: u32) {
    if let Ok(mut rng) = GLOBAL_DIABLO_RNG.lock() {
        rng.discard(count);
    }
}

/// 生成随机数
pub fn generate_random_number() -> u32 {
    GLOBAL_DIABLO_RNG
        .lock()
        .map(|mut rng| rng.next_raw())
        .unwrap_or(0)
}

/// 推进随机种子
pub fn advance_rnd_seed() -> i32 {
    GLOBAL_DIABLO_RNG
        .lock()
        .map(|mut rng| rng.advance_rnd_seed())
        .unwrap_or(0)
}

/// 生成小于 v 的随机数
pub fn generate_rnd(v: i32) -> i32 {
    GLOBAL_DIABLO_RNG
        .lock()
        .map(|mut rng| rng.generate_rnd(v))
        .unwrap_or(0)
}

/// 抛硬币
pub fn flip_coin(frequency: u32) -> bool {
    GLOBAL_DIABLO_RNG
        .lock()
        .map(|mut rng| rng.flip_coin(frequency))
        .unwrap_or(false)
}

/// 生成非负整数
pub fn random_int_less_than(v: i32) -> i32 {
    generate_rnd(v).max(0)
}

/// 生成范围内的随机数
pub fn random_int_between(min: i32, max: i32, half_open: bool) -> i32 {
    random_int_less_than(max - min + if half_open { 0 } else { 1 }) + min
}

/// 生成种子
pub fn generate_seed() -> u32 {
    let mut guard = GLOBAL_SEED_RNG.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Xoshiro128PlusPlus::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(12345),
        ));
    }
    guard.as_mut().unwrap().next()
}

/// 保留种子序列
pub fn reserve_seed_sequence() -> Xoshiro128PlusPlus {
    let mut guard = GLOBAL_SEED_RNG.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Xoshiro128PlusPlus::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(12345),
        ));
    }
    let mut result = guard.as_ref().unwrap().clone();
    guard.as_mut().unwrap().jump();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diablo_generator() {
        let mut rng = DiabloGenerator::new(12345);

        // 测试连续调用产生不同值
        let v1 = rng.generate_rnd(100);
        let v2 = rng.generate_rnd(100);
        let v3 = rng.generate_rnd(100);

        // 值应该在有效范围内
        assert!(v1 >= 0 && v1 < 100);
        assert!(v2 >= 0 && v2 < 100);
        assert!(v3 >= 0 && v3 < 100);
    }

    #[test]
    fn test_diablo_generator_reproducibility() {
        let mut rng1 = DiabloGenerator::new(42);
        let mut rng2 = DiabloGenerator::new(42);

        // 相同种子应产生相同序列
        for _ in 0..100 {
            assert_eq!(rng1.generate_rnd(1000), rng2.generate_rnd(1000));
        }
    }

    #[test]
    fn test_xoshiro128_plusplus() {
        let mut rng = Xoshiro128PlusPlus::new(12345);

        let v1 = rng.next();
        let v2 = rng.next();

        // 应产生不同值
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_split_mix() {
        let mut sm32 = SplitMix32::new(42);
        let mut sm64 = SplitMix64::new(42);

        // 应该产生非零值
        assert_ne!(sm32.next(), 0);
        assert_ne!(sm64.next(), 0);
    }

    #[test]
    fn test_flip_coin() {
        let mut rng = DiabloGenerator::new(12345);
        let mut true_count = 0;
        let mut false_count = 0;

        for _ in 0..1000 {
            if rng.flip_coin(2) {
                true_count += 1;
            } else {
                false_count += 1;
            }
        }

        // 应该大致各占一半
        assert!(true_count > 400 && true_count < 600);
        assert!(false_count > 400 && false_count < 600);
    }
}
