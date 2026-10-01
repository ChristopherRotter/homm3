//! Deterministic pseudo-random numbers with named streams (ADR-0003 §7).
//!
//! * Generator: **xoshiro256\*\*** (Blackman & Vigna, public-domain reference
//!   <https://prng.di.unimi.it/xoshiro256starstar.c>): 256-bit state, fast, excellent quality,
//!   trivially cloneable (the state is four `u64`).
//! * Seeding: **SplitMix64** (<https://prng.di.unimi.it/splitmix64.c>) expands a 64-bit seed into
//!   the 256-bit state, as recommended by the xoshiro authors.
//! * Streams: a named stream of a master seed is seeded from `master ^ splitmix64(stream id)`, so
//!   consumers (battle, agents, ...) never share or perturb each other's sequences.
//! * Bounded integers: Lemire's multiply-and-reject method on 32-bit draws (unbiased).
//!
//! Replays record [`RNG_ALGORITHM`] and [`RNG_VERSION`]; changing anything here that alters the
//! produced sequence requires bumping the version.

use crate::hash::{CanonicalHash, StateHasher};

/// Algorithm identifier recorded in replay headers.
pub const RNG_ALGORITHM: &str = "xoshiro256starstar+splitmix64";
/// Version of the generator, seeding, stream derivation and bounded-integer methods.
pub const RNG_VERSION: u32 = 1;

const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// SplitMix64, used for seeding and stream derivation.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// A SplitMix64 generator with the given state.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next output.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// Identifier of a named random stream. Stream ids are a closed, documented list so that two
/// consumers never use the same stream by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StreamId(pub u64);

impl StreamId {
    /// Battle mechanics (damage rolls, later morale/luck, ...).
    pub const BATTLE: Self = Self(1);
    /// Agents, bots and tie-breaking outside game rules (never feeds game state directly).
    pub const AGENT: Self = Self(2);
    /// Random battlefield obstacles, generated before a battle starts (BAT-GEO-004).
    pub const OBSTACLES: Self = Self(3);
}

/// The engine's random number generator (xoshiro256\*\*). All randomness in game logic goes
/// through an explicit `&mut Rng`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    /// A generator seeded from a 64-bit seed via SplitMix64.
    // rule: KER-RNG-001
    pub fn from_seed(seed: u64) -> Self {
        let mut sm = SplitMix64::new(seed);
        // Four consecutive SplitMix64 outputs are never all zero (its output function is a
        // bijection of distinct counters), so the xoshiro state is always valid.
        Self {
            s: [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()],
        }
    }

    /// The generator for a named stream of a master seed.
    // rule: KER-RNG-001
    pub fn for_stream(master_seed: u64, stream: StreamId) -> Self {
        let mix = SplitMix64::new(stream.0).next_u64();
        Self::from_seed(master_seed ^ mix)
    }

    /// Next 64 random bits.
    // rule: KER-RNG-001
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Next 32 random bits (the high half of [`Rng::next_u64`], the better-quality bits).
    pub fn next_u32(&mut self) -> u32 {
        // Truncation is the point: keep the high 32 bits.
        #[allow(clippy::cast_possible_truncation)]
        let v = (self.next_u64() >> 32) as u32;
        v
    }

    /// Uniform integer in `0..n` (Lemire's method, unbiased). Returns 0 when `n == 0`.
    // rule: KER-RNG-002
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let mut m = u64::from(self.next_u32()) * u64::from(n);
        #[allow(clippy::cast_possible_truncation)]
        let mut low = m as u32;
        if low < n {
            let threshold = n.wrapping_neg() % n;
            while low < threshold {
                m = u64::from(self.next_u32()) * u64::from(n);
                #[allow(clippy::cast_possible_truncation)]
                let l = m as u32;
                low = l;
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        let v = (m >> 32) as u32;
        v
    }

    /// Uniform integer in `lo..=hi`. If `hi < lo` the bounds are swapped.
    // rule: KER-RNG-002
    pub fn range_inclusive(&mut self, lo: u32, hi: u32) -> u32 {
        let (lo, hi) = if hi < lo { (hi, lo) } else { (lo, hi) };
        match (hi - lo).checked_add(1) {
            Some(span) => lo + self.below(span),
            None => self.next_u32(),
        }
    }

    /// The raw 256-bit state (for canonical hashing and diagnostics).
    pub const fn state(&self) -> [u64; 4] {
        self.s
    }
}

impl CanonicalHash for Rng {
    fn hash_into(&self, h: &mut StateHasher) {
        for w in self.s {
            h.write_u64(w);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use proptest::prelude::{prop_assert, prop_assert_eq, proptest};
    use std::vec::Vec;

    // covers: KER-RNG-001
    #[test]
    fn splitmix64_reference_vectors() {
        // Outputs of the reference C implementation (splitmix64.c) for seeds 0 and 1234567.
        let mut a = SplitMix64::new(0);
        assert_eq!(
            [a.next_u64(), a.next_u64(), a.next_u64()],
            [
                0xe220_a839_7b1d_cdaf,
                0x6e78_9e6a_a1b9_65f4,
                0x06c4_5d18_8009_454f
            ]
        );
        let mut b = SplitMix64::new(1_234_567);
        assert_eq!(
            [b.next_u64(), b.next_u64(), b.next_u64()],
            [
                0x599e_d017_fb08_fc85,
                0x2c73_f084_5854_0fa5,
                0x883e_bce5_a3f2_7c77
            ]
        );
    }

    // covers: KER-RNG-001
    #[test]
    fn xoshiro256starstar_reference_vectors() {
        // Reference C implementation (xoshiro256starstar.c) from state {1, 2, 3, 4}.
        let mut r = Rng { s: [1, 2, 3, 4] };
        let got: Vec<u64> = (0..4).map(|_| r.next_u64()).collect();
        assert_eq!(got, [11520, 0, 1_509_978_240, 1_215_971_899_390_074_240]);
        // Seeded through SplitMix64 (reference C: seed 42 expanded into four words).
        let mut s = Rng::from_seed(42);
        let got: Vec<u64> = (0..4).map(|_| s.next_u64()).collect();
        assert_eq!(
            got,
            [
                0x1578_0b2e_0c2e_c716,
                0x6104_d986_6d11_3a7e,
                0xae17_5332_39e4_99a1,
                0xecb8_ad47_03b3_60a1
            ]
        );
    }

    // covers: KER-RNG-001
    #[test]
    fn streams_are_distinct_and_reproducible() {
        let mut battle = Rng::for_stream(7, StreamId::BATTLE);
        let mut agent = Rng::for_stream(7, StreamId::AGENT);
        let mut battle2 = Rng::for_stream(7, StreamId::BATTLE);
        let a: Vec<u64> = (0..8).map(|_| battle.next_u64()).collect();
        let b: Vec<u64> = (0..8).map(|_| agent.next_u64()).collect();
        let c: Vec<u64> = (0..8).map(|_| battle2.next_u64()).collect();
        assert_ne!(a, b);
        assert_eq!(a, c);
        assert_ne!(
            Rng::for_stream(8, StreamId::BATTLE),
            Rng::for_stream(7, StreamId::BATTLE)
        );
        // Golden value: pins stream derivation across platforms and versions.
        assert_eq!(
            Rng::for_stream(7, StreamId::BATTLE).next_u64(),
            GOLDEN_STREAM_7_BATTLE
        );
        assert_eq!(RNG_ALGORITHM, "xoshiro256starstar+splitmix64");
        assert_eq!(RNG_VERSION, 1);
    }

    // Computed independently (Python transcription of the reference algorithms) for
    // master seed 7, stream BATTLE (id 1).
    const GOLDEN_STREAM_7_BATTLE: u64 = 0x16cd_47ca_fa16_17c5;

    // covers: KER-RNG-002
    #[test]
    fn below_edge_cases_and_rejection() {
        let mut r = Rng::from_seed(1);
        assert_eq!(r.below(0), 0);
        assert_eq!(r.below(1), 0);
        // A draw of exactly 0 has low == 0 < threshold for any n that does not divide 2^32:
        // exercise the rejection loop with a crafted state that yields a 0 first.
        let mut crafted = Rng { s: [0, 0, 1, 0] };
        let first = crafted.clone().next_u32();
        assert_eq!(first, 0, "crafted state must produce a zero draw");
        let v = crafted.below(3);
        assert!(v < 3);
        assert_eq!(r.range_inclusive(5, 5), 5);
        assert!((2..=9).contains(&r.range_inclusive(9, 2)));
        let _full = r.range_inclusive(0, u32::MAX);
        assert_eq!(r.state().len(), 4);
    }

    // covers: KER-RNG-002
    #[test]
    fn below_is_roughly_uniform() {
        let mut r = Rng::from_seed(99);
        let mut counts = [0u32; 6];
        for _ in 0..60_000 {
            counts[r.below(6) as usize] += 1;
        }
        for c in counts {
            assert!((9_400..=10_600).contains(&c), "{counts:?}");
        }
    }

    #[test]
    fn hash_covers_whole_state() {
        let r = Rng { s: [1, 2, 3, 4] };
        let mut h = StateHasher::new();
        r.hash_into(&mut h);
        let mut g = StateHasher::new();
        for w in [1u64, 2, 3, 4] {
            g.write_u64(w);
        }
        assert_eq!(h.finish(), g.finish());
    }

    proptest! {
        // covers: KER-RNG-002
        #[test]
        fn bounded_draws_stay_in_range(seed: u64, n in 1u32.., lo: u32, hi: u32) {
            let mut r = Rng::from_seed(seed);
            prop_assert!(r.below(n) < n);
            let v = r.range_inclusive(lo, hi);
            prop_assert!(v >= lo.min(hi) && v <= lo.max(hi));
        }

        #[test]
        fn clones_evolve_identically(seed: u64) {
            let mut a = Rng::from_seed(seed);
            let mut b = a.clone();
            for _ in 0..16 {
                prop_assert_eq!(a.next_u64(), b.next_u64());
            }
        }
    }
}
