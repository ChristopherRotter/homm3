//! Tests of the native-terrain bonus (BAT-TER-001) and obstacle generation (BAT-GEO-004).

use std::collections::BTreeSet;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_kernel::{Rng, StreamId};

use super::support::*;
use crate::geometry::Direction;
use crate::terrain::generate_obstacles;
use crate::{BattleSetup, BattleState};

fn terrain(r: &Ruleset, name: &str) -> h3_content::TerrainId {
    r.terrain_by_key(&std::format!("t:{name}")).unwrap()
}

fn generate(r: &Ruleset, name: &str, seed: u64) -> Vec<u8> {
    generate_obstacles(
        r,
        terrain(r, name),
        &mut Rng::for_stream(seed, StreamId::OBSTACLES),
    )
}

// ---------------------------------------------------------------- BAT-TER-001 native terrain

// covers: BAT-TER-001
#[test]
fn creatures_on_their_native_terrain_get_one_attack_defense_and_speed() {
    let r = ruleset();
    let stats = |t: Option<&str>| {
        let s = BattleSetup {
            terrain: t.map(|t| terrain(&r, t)),
            ..setup(&r, &[(0, "pikeman_c", 1), (1, "imp", 1)], &[(0, "serf", 1)])
        };
        let b = BattleState::new(&r, &s, 1).unwrap();
        let out: Vec<(u16, u16, u8)> = b
            .stacks()
            .iter()
            .map(|s| (s.attack, s.defense, s.speed))
            .collect();
        (out, b.terrain)
    };
    // Castle is native to the plain, Inferno to lava; neutral creatures have no native terrain.
    let (plain, at) = stats(Some("plain"));
    assert_eq!(plain, [(11, 11, 6), (10, 10, 5), (10, 10, 5)]);
    assert_eq!(at, Some(terrain(&r, "plain")));
    assert_eq!(
        stats(Some("lava")).0,
        [(10, 10, 5), (11, 11, 6), (10, 10, 5)]
    );
    assert_eq!(stats(Some("void")).0, [(10, 10, 5); 3]);
    assert_eq!(stats(None).0, [(10, 10, 5); 3]);
}

// ---------------------------------------------------------------- BAT-GEO-004 obstacles

// covers: BAT-GEO-004
#[test]
fn generated_obstacles_block_five_to_twelve_hexes_away_from_the_armies() {
    let r = ruleset();
    let f = field(&r);
    let mut seen = BTreeSet::new();
    for seed in 0..200u64 {
        let hexes = generate(&r, "plain", seed);
        assert_eq!(hexes, generate(&r, "plain", seed), "deterministic");
        // Only two-hex stones: 5..=12 hexes to block means 3..=6 stones.
        assert!(
            (6..=12).contains(&hexes.len()) && hexes.len().is_multiple_of(2),
            "{hexes:?}"
        );
        assert!(hexes.windows(2).all(|w| w[0] < w[1]), "sorted, each once");
        for &h in &hexes {
            let (x, y) = f.coords(crate::Hex(h));
            assert!((3..=13).contains(&x) && (2..=9).contains(&y), "({x}, {y})");
        }
        seen.insert(hexes.clone());
        // The setup is valid and the armies still find their places.
        let s = BattleSetup {
            obstacles: hexes,
            ..setup(&r, &[(0, "serf", 1)], &[(0, "serf", 1)])
        };
        BattleState::new(&r, &s, seed).unwrap();
    }
    assert!(seen.len() > 150, "varied layouts: {}", seen.len());
}

// covers: BAT-GEO-004
#[test]
fn absolute_obstacles_appear_with_their_chance_and_count_half() {
    let r = ruleset();
    let cliff: Vec<u8> = [(5, 4), (6, 4), (7, 4), (8, 4)]
        .iter()
        .map(|&(x, y)| at(&r, x, y).0)
        .collect();
    let mut with_cliff = 0;
    for seed in 0..400u64 {
        let hexes = generate(&r, "lava", seed);
        if cliff.iter().all(|h| hexes.contains(h)) {
            with_cliff += 1;
            // The cliff counts 2 of 5..=12, the stones fill 3..=10: 2..=5 stones.
            assert!((8..=14).contains(&hexes.len()), "{hexes:?}");
        }
    }
    // 40 % of 400 = 160 (σ ≈ 9.8); stones alone never cover all four cliff hexes in a row
    // often enough to matter.
    assert!((125..=200).contains(&with_cliff), "{with_cliff}");
}

// covers: BAT-GEO-004
#[test]
fn generation_stops_without_obstacles_or_places() {
    let r = ruleset();
    for seed in 0..20u64 {
        assert!(generate(&r, "void", seed).is_empty());
        assert!(generate(&r, "tall", seed).is_empty());
    }
}

// covers: BAT-GEO-004
#[test]
fn obstacle_offsets_follow_the_hex_rows() {
    // (1, −1) is the top-right neighbour of the anchor in odd and even rows alike.
    let r = ruleset_with("t:faithful", |_| {});
    let f = field(&r);
    let o = h3_content::ObstacleType {
        key: h3_content::ContentKey::new("t", "o").unwrap(),
        terrains: Vec::new(),
        width: 2,
        height: 1,
        blocked: std::vec![(1, -1)],
        absolute: false,
    };
    for (x, y) in [(5, 5), (5, 4)] {
        let anchor = at(&r, x, y);
        let placed = crate::terrain::placement_for_tests(f, &o, anchor);
        assert_eq!(
            placed,
            f.neighbor(anchor, Direction::TopRight)
                .map(|h| std::vec![h.0])
        );
    }
}
