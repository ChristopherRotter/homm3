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
    /// The game's own randomness, such as the day advance (ADR-0017 point 4).
    pub const GAME: Self = Self(4);
    /// Resolving a game setup's random placeholders when the state is built (ADR-0017 point 4).
    pub const SETUP: Self = Self(5);
    /// First id of the per-player streams; ids below it are the named streams above.
    const PLAYER_BASE: u64 = 0x100;
    /// First id of the per-seat streams, after the players'.
    const SEAT_BASE: u64 = 0x200;
    /// First id of the per-environment streams, above every other id.
    const ENV_BASE: u64 = 1 << 32;

    /// The stream of the player at `index` of a game's player list: randomness caused by that
    /// player's commands draws only from it (ADR-0017 point 4). Distinct from every named stream
    /// and from every other player's.
    pub const fn player(index: u8) -> Self {
        Self(Self::PLAYER_BASE + index as u64)
    }

    /// The stream of the battle seat at `index` (0 the attacker, 1 the defender) of an episode's
    /// seed: a policy played inside the engine for that side draws only from it (ADR-0025
    /// point 5). Distinct from every named, player and environment stream.
    pub const fn seat(index: u8) -> Self {
        Self(Self::SEAT_BASE + index as u64)
    }

    /// The stream of the environment at `index` of a batch's seed: that environment's episode
    /// seeds are its successive draws (ADR-0025 point 4), so an episode depends on the batch's
    /// seed, the environment's index and the episode's number only. Distinct from every other
    /// stream.
    pub const fn env(index: u32) -> Self {
        Self(Self::ENV_BASE + index as u64)
    }
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

    /// An approximately binomial count in constant time: the number of successes among `trials`
    /// independent trials of `percent` % each (at most 100), drawn from the normal approximation
    /// with mean n·p and standard deviation √(n·p·(1 − p)). The normal variate is the sum of
    /// twelve uniform draws minus six (variance 1, within ±6); the count is rounded to the nearest
    /// integer and clamped to `0..=trials`. Integer arithmetic only (ADR-0003). Meant for large
    /// `trials` (tens of thousands and more), where it is within about 0.6 % of the exact law in
    /// total variation; small counts should roll each trial.
    // rule: KER-RNG-003
    pub fn binomial_approx(&mut self, trials: u32, percent: u8) -> u32 {
        let n = u128::from(trials);
        let c = u128::from(percent.min(100));
        // n·c·(100 − c) < 2^44, so the shifted values stay below 2^108.
        // σ · 2^32 = √(n·c·(100 − c)) · 2^32 / 100.
        let sigma = ((n * c * (100 - c)) << 64).isqrt() / 100;
        // The mean n·c / 100 in 64.64 fixed point.
        let mean = ((n * c) << 64) / 100;
        // Z · 2^32 for Z = Σ U_i − 6, U_i uniform in [0, 1).
        let sum: i128 = (0..12).map(|_| i128::from(self.next_u32())).sum();
        let z = sum - (6 << 32);
        // σ < 2^47 and |Z| · 2^32 < 2^35: the product stays below 2^82.
        let x = i128::try_from(mean).unwrap_or(i128::MAX) + i128::try_from(sigma).unwrap_or(0) * z;
        let k = (x + (1 << 63)) >> 64;
        u32::try_from(k.clamp(0, i128::from(trials))).unwrap_or(trials)
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

    // covers: KER-RNG-003
    #[test]
    fn approximate_binomial_draws_have_the_binomial_mean_and_spread() {
        let mut rng = Rng::from_seed(7);
        // Certain and impossible trials, and the clamp to 0..=n.
        assert_eq!(rng.binomial_approx(100_000, 100), 100_000);
        assert_eq!(rng.binomial_approx(100_000, 0), 0);
        assert_eq!(rng.binomial_approx(0, 50), 0);
        assert_eq!(rng.binomial_approx(u32::MAX, 200), u32::MAX);
        for _ in 0..100 {
            assert!(rng.binomial_approx(1, 50) <= 1);
        }
        // 100 000 trials at 10 %: mean 10 000, standard deviation ≈ 94.9.
        let draws: Vec<u32> = (0..4000)
            .map(|_| rng.binomial_approx(100_000, 10))
            .collect();
        let sum: i64 = draws.iter().map(|&d| i64::from(d)).sum();
        assert!((sum - 4000 * 10_000).abs() < 4000 * 5, "{sum}");
        // The variance n·p·(1 − p) = 9000, within ±750.
        let squares: i64 = draws.iter().map(|&d| (i64::from(d) - 10_000).pow(2)).sum();
        assert!((8250 * 4000..=9750 * 4000).contains(&squares), "{squares}");
        assert!(draws.iter().all(|&d| (9_430..=10_570).contains(&d)));
    }

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

    // covers: KER-RNG-001
    #[test]
    fn player_streams_are_distinct_from_named_streams_and_each_other() {
        let named = [
            StreamId::BATTLE,
            StreamId::AGENT,
            StreamId::OBSTACLES,
            StreamId::GAME,
            StreamId::SETUP,
        ];
        let players: Vec<StreamId> = (0..=u8::MAX).map(StreamId::player).collect();
        assert!(players.iter().all(|p| !named.contains(p)));
        assert!(players.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(StreamId::player(0), StreamId(0x100));
        assert_ne!(
            Rng::for_stream(7, StreamId::player(0)),
            Rng::for_stream(7, StreamId::player(1))
        );
    }

    // covers: KER-RNG-001
    #[test]
    fn seat_and_env_streams_are_distinct_from_every_other_stream() {
        let named = [1, 2, 3, 4, 5].map(StreamId);
        let players = (0..=u8::MAX).map(StreamId::player);
        let seats: Vec<StreamId> = (0..=u8::MAX).map(StreamId::seat).collect();
        let envs = [0, 1, 2, u32::MAX - 1, u32::MAX].map(StreamId::env);
        let mut all: Vec<StreamId> = named.into_iter().chain(players).collect();
        assert!(seats.iter().all(|s| !all.contains(s)));
        all.extend(&seats);
        assert!(envs.iter().all(|e| !all.contains(e)));
        assert!(seats.windows(2).all(|w| w[0] < w[1]));
        assert!(envs.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(StreamId::seat(0), StreamId(0x200));
        assert_eq!(StreamId::env(0), StreamId(1 << 32));
        assert_ne!(
            Rng::for_stream(7, StreamId::env(0)).next_u64(),
            Rng::for_stream(7, StreamId::env(1)).next_u64()
        );
        assert_ne!(
            Rng::for_stream(7, StreamId::seat(0)).next_u64(),
            Rng::for_stream(7, StreamId::seat(1)).next_u64()
        );
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
