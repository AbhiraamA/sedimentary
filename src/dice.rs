//! A tiny seedable random number generator (SplitMix64).
//!
//! The game only needs dice rolls, so twenty lines beat a dependency, and a
//! fixed seed makes every test reproducible.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Dice {
    state: u64,
}

impl Dice {
    pub fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Seeded from the clock and process id. Good enough for a rock.
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5EED);
        Self::seeded(nanos ^ u64::from(std::process::id()).rotate_left(32))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform float in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// True with probability `p` (clamped to `[0, 1]`).
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p.clamp(0.0, 1.0)
    }

    /// Uniform index in `0..n`; returns 0 when `n` is 0.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// Uniform integer in `lo..=hi`.
    pub fn between(&mut self, lo: u32, hi: u32) -> u32 {
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        lo + (self.next_u64() % (u64::from(hi - lo) + 1)) as u32
    }

    pub fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_rolls() {
        let mut a = Dice::seeded(42);
        let mut b = Dice::seeded(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn ranges_hold() {
        let mut d = Dice::seeded(7);
        for _ in 0..10_000 {
            let u = d.unit();
            assert!((0.0..1.0).contains(&u));
            assert!(d.below(5) < 5);
            assert!((3..=9).contains(&d.between(3, 9)));
        }
        assert_eq!(d.below(0), 0);
        assert!(!d.chance(0.0));
        assert!(d.chance(1.0));
    }
}
