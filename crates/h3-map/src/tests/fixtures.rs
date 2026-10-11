//! The generated maps committed under `fixtures/maps/` (E08 T9a): `h3-sim`'s adventure
//! environment plays them in its tests, and T9b's golden replays and benchmarks will. Each is laid
//! out by T8b2a's [`lay_out`] from a plan built here with a fixed generator (not proptest's, so
//! that the plan depends on this file alone), written as the game stores maps (gzip). The test
//! checks that the committed file reads to the same map and imports; it rewrites the file when
//! `H3_WRITE_FIXTURES` is set (a change of a committed map invalidates the replays that name it by
//! content hash, ADV-Q2).

use std::path::PathBuf;

use super::gzip_with;
use super::play::{FAITHFUL, Plan, lay_out};
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

/// `e08-duel-36.h3m`: decision 11's random-play map. SoD, 36 × 36 with an underground, red and
/// blue with two heroes each (one on each level, red's in the west, blue's in the east; their
/// types' starting armies), terrain in patches of 6 × 6 tiles (all eight land terrains, some
/// water and rock), a road across each level, rivers on some patches, 72 blockers.
fn duel() -> Plan {
    const SIDE: u8 = 36;
    let mut mix = Mix(36);
    let patches: Vec<(u8, u8)> = (0..2 * 6 * 6)
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
    let mut tiles = Vec::new();
    for z in 0..2u8 {
        for y in 0..SIDE {
            for x in 0..SIDE {
                let patch = (usize::from(z) * 6 + usize::from(y / 6)) * 6 + usize::from(x / 6);
                let (terrain, river) = patches[patch];
                // A road across the level: row 17 on the surface, column 17 underground.
                let road = if (z == 0 && y == 17) || (z == 1 && x == 17) {
                    1 + z
                } else {
                    0
                };
                let terrain = if road == 0 { terrain } else { 2 };
                tiles.push((terrain, road, river));
            }
        }
    }
    let blockers = (0..72)
        .map(|_| {
            let anchor = (mix.below(SIDE), mix.below(SIDE), mix.below(2));
            (anchor, usize::from(mix.below(3)))
        })
        .collect();
    Plan {
        version: Version::Sod,
        side: SIDE,
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

/// A committed generated map: its file name and plan.
type Fixture = (&'static str, fn() -> Plan);

/// The committed generated maps.
const FIXTURES: [Fixture; 1] = [("e08-duel-36.h3m", duel)];

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
        assert_eq!(setup.heroes.len(), 4, "{name}");
    }
}
