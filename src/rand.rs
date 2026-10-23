//! Deterministic pseudo-random number generation.
//!
//! This is not cryptographic.  It is used for wallpaper grain, particle
//! effects and randomised playlists; the kernel seeds it from the TSC and RTC.

/// A SplitMix64 generator, which is small, fast and has good statistical
/// properties for a 64-bit state.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Create a generator seeded from a hardware counter and an address.
    pub fn from_entropy(counter: u64, address: usize) -> Self {
        let seed = counter ^ (address as u64).rotate_left(17) ^ 0x9e37_79b9_7f4a_7c15;
        Self::new(seed)
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform value in `0..bound`.  Returns zero if `bound` is zero.
    #[inline]
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as u32
        }
    }

    /// Uniform value in `low..high`.
    #[inline]
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        if high <= low {
            low
        } else {
            low + self.below((high - low) as u32) as i32
        }
    }

    /// Uniform float in `0.0..1.0`.
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// A value in `-scale..=scale`.
    #[inline]
    pub fn signed(&mut self, scale: f32) -> f32 {
        (self.next_f32() * 2.0 - 1.0) * scale
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.below((i + 1) as u32) as usize;
            slice.swap(i, j);
        }
    }
}

/// A tiny xorshift generator for hot paths that need only a few bits.
#[derive(Clone, Debug)]
pub struct XorShift32 {
    state: u32,
}

impl XorShift32 {
    pub const fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x1234_5678 } else { seed },
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

/// A random seed derived from the CPU timestamp counter.
pub fn tsc_seed() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: reading the TSC is unprivileged and has no side effects.
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        0x5eed_1234_5678_9abc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::new(123);
        let mut b = Rng::new(123);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn ranges() {
        let mut rng = Rng::new(7);
        for _ in 0..1_000 {
            let value = rng.range(-5, 5);
            assert!((-5..5).contains(&value));
            let f = rng.next_f32();
            assert!((0.0..1.0).contains(&f));
        }
    }
}
