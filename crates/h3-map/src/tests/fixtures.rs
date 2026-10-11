//! The generated maps committed under `fixtures/maps/` (E08 T9a): `h3-sim`'s adventure
//! environment plays them in its tests, T9b's golden replays and benchmarks too. Each is laid
//! out by T8b2a's [`lay_out`] from a plan built here with a fixed generator (not proptest's, so
//! that the plan depends on this file alone), written as the game stores maps (gzip). The test
//! checks that the committed file reads to the same map and imports; it rewrites the file when
//! `H3_WRITE_FIXTURES` is set (a change of a committed map invalidates the replays that name it by
//! content hash, ADV-Q2).

use std::path::PathBuf;

use super::gzip_with;
use super::play::{FAITHFUL, Plan, Spot, lay_out};
use crate::model::Version;
use crate::{Limits, import, read};

/// SplitMix64: a fixed generator for the plans.
struct Mix(u64);

impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value below `n`.
    fn below(&mut self, n: u8) -> u8 {
        u8::try_from(self.next() % u64::from(n)).unwrap()
    }
}

/// A tile's terrain, road and river codes.
type Codes = (u8, u8, u8);

/// Terrain in patches of 6 × 6 tiles (all eight land terrains, some water and rock), a road
/// across each level (the middle row on the surface, the middle column underground), rivers on
/// some patches, and `count` blockers, for a two-level map of `side` tiles, all drawn from `mix`
/// in this order (`e08-duel-36.h3m`'s draws: changing the order changes the committed maps).
fn patchwork(side: u8, mix: &mut Mix, count: usize) -> (Vec<Codes>, Vec<(Spot, usize)>) {
    assert_eq!(side % 6, 0, "whole patches only");
    let per_row = usize::from(side / 6);
    let patches: Vec<(u8, u8)> = (0..2 * per_row * per_row)
        .map(|_| {
            let terrain = match mix.below(12) {
                10 => 8,
                11 => 9,
                land => land % 8,
            };
            let river = if mix.below(4) == 0 {
                1 + mix.below(4)
            } else {
                0
            };
            (terrain, river)
        })
        .collect();
    let middle = side / 2 - 1;
    let mut tiles = Vec::new();
    for z in 0..2u8 {
        for y in 0..side {
            for x in 0..side {
                let patch =
                    (usize::from(z) * per_row + usize::from(y / 6)) * per_row + usize::from(x / 6);
                let (terrain, river) = patches[patch];
                let road = if (z == 0 && y == middle) || (z == 1 && x == middle) {
                    1 + z
                } else {
                    0
                };
                let terrain = if road == 0 { terrain } else { 2 };
                tiles.push((terrain, road, river));
            }
        }
    }
    let blockers = (0..count)
        .map(|_| {
            let anchor = (mix.below(side), mix.below(side), mix.below(2));
            (anchor, usize::from(mix.below(3)))
        })
        .collect();
    (tiles, blockers)
}

/// `e08-duel-36.h3m`: decision 11's random-play map. SoD, 36 × 36 with an underground, red and
/// blue with two heroes each (one on each level, red's in the west, blue's in the east; their
/// types' starting armies), [`patchwork`] terrain, 72 blockers.
fn duel() -> Plan {
    let (tiles, blockers) = patchwork(36, &mut Mix(36), 72);
    Plan {
        version: Version::Sod,
        side: 36,
        two_levels: true,
        tiles,
        players: vec![(0, true), (1, true)],
        // Owners go round the players: red, blue, red, blue.
        heroes: vec![
            ((4, 5, 0), None),
            ((31, 30, 0), None),
            ((5, 30, 1), None),
            ((30, 5, 1), None),
        ],
        hero_types: 3,
        blockers,
    }
}

/// `e08-xl-144.h3m`: decision 11's pathfinding and fork map (E08 T9b). SoD, 144 × 144 with an
/// underground (the largest map, 41 472 tiles), red and blue with four heroes each (two on each
/// level, red's in the west, blue's in the east), [`patchwork`] terrain, 1 152 blockers (the
/// duel's density).
fn xl() -> Plan {
    let (tiles, blockers) = patchwork(144, &mut Mix(144), 72 * 16);
    Plan {
        version: Version::Sod,
        side: 144,
        two_levels: true,
        tiles,
        players: vec![(0, true), (1, true)],
        // Owners go round the players: red, blue, red, blue, …
        heroes: vec![
            ((10, 10, 0), None),
            ((133, 133, 0), None),
            ((10, 130, 0), None),
            ((133, 13, 0), None),
            ((12, 72, 1), None),
            ((131, 72, 1), None),
            ((70, 20, 1), None),
            ((73, 123, 1), None),
        ],
        hero_types: 5,
        blockers,
    }
}

/// A committed generated map: its file name and plan.
type Fixture = (&'static str, fn() -> Plan);

/// The committed generated maps.
const FIXTURES: [Fixture; 2] = [("e08-duel-36.h3m", duel), ("e08-xl-144.h3m", xl)];

#[test]
fn generated_maps_are_committed_as_generated() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/maps");
    for (name, plan) in FIXTURES {
        let m = lay_out(&plan());
        let path = dir.join(name);
        if std::env::var_os("H3_WRITE_FIXTURES").is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&path, gzip_with(&super::encode::encode(&m), 0, &[])).unwrap();
        }
        let file = std::fs::read(&path).unwrap();
        let read = read(&file, &Limits::default()).unwrap();
        assert!(
            read == m,
            "{name} is not its plan's map: H3_WRITE_FIXTURES=1 rewrites it"
        );
        let setup = import(&read, &FAITHFUL).unwrap();
        assert_eq!(setup.heroes.len(), plan().heroes.len(), "{name}");
    }
}
