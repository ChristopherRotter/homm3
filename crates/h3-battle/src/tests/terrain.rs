//! Tests of the native-terrain bonus (BAT-TER-001) and obstacle generation (BAT-GEO-004).

use std::collections::BTreeSet;
use std::string::String;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_kernel::{Rng, StreamId};

use super::support::*;
use crate::geometry::Direction;
use crate::terrain::generate_obstacles;
use crate::{BattleSetup, BattleState, MAX_REMOVABLE_HEXES};

fn terrain(r: &BattleRuleset, name: &str) -> h3_content::TerrainId {
    r.terrain_by_key(&std::format!("t:{name}")).unwrap()
}

fn pieces(r: &BattleRuleset, name: &str, seed: u64) -> Vec<h3_contracts::Obstacle> {
    generate_obstacles(
        r,
        terrain(r, name),
        &mut Rng::for_stream(seed, StreamId::OBSTACLES),
    )
}

/// The generated obstacles' hexes, in hex order.
fn generate(r: &BattleRuleset, name: &str, seed: u64) -> Vec<u8> {
    let mut hexes: Vec<u8> = pieces(r, name, seed)
        .into_iter()
        .flat_map(|o| o.hexes)
        .collect();
    hexes.sort_unstable();
    hexes
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
    // A terrain handle of another ruleset is rejected.
    let unknown = BattleSetup {
        terrain: Some(h3_content::TerrainId::from_raw(99)),
        ..setup(&r, &[(0, "serf", 1)], &[(0, "serf", 1)])
    };
    assert_eq!(
        BattleState::new(&r, &unknown, 1),
        Err(crate::SetupError::UnknownTerrain)
    );
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
        // Each stone keeps its anchor, its bottom-left hex (Q55).
        let ps = pieces(&r, "plain", seed);
        assert!(
            ps.iter()
                .all(|o| o.hexes.len() == 2 && o.anchor == Some(o.hexes[0])),
            "{ps:?}"
        );
        // The setup is valid and the armies still find their places.
        let s = BattleSetup {
            obstacles: ps,
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
            // An absolute obstacle comes first and has no anchor: no spell removes it.
            let first = &pieces(&r, "lava", seed)[0];
            assert_eq!((&first.hexes, first.anchor), (&cliff, None));
            // Each piece names its type (E17 M1, CLT-Q3): the cliff, then stones.
            let kinds: Vec<_> = pieces(&r, "lava", seed)
                .iter()
                .map(|p| p.kind.as_ref().map(|k| String::from(k.as_str())))
                .collect();
            assert_eq!(kinds[0].as_deref(), Some("t:cliff"));
            assert!(kinds[1..].iter().all(|k| k.as_deref() == Some("t:stone")));
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

/// E04b T11b review (F-robust-5): the battle accepts every battlefield the generator makes. A
/// usual obstacle becomes a removable piece, which holds at most [`MAX_REMOVABLE_HEXES`] hexes
/// (BAT-GEO-003): a type that blocks so many is placed and accepted, one that blocks more does
/// not compile (it used to compile, and the battle refused the generated battlefield).
// covers: BAT-GEO-004, BAT-GEO-003
#[test]
fn the_battle_accepts_every_generated_battlefield() {
    use h3_content::ContentKey;
    use h3_content_battle::{BattleCompileError, ObstacleDef};
    // A row of `hexes` hexes from its anchor, on the terrain without other obstacles.
    let with_ridge = |hexes: u8| {
        try_ruleset_from(
            "t:faithful",
            |_| {},
            |_, battle| {
                battle.obstacles.push(ObstacleDef {
                    key: ContentKey::new("t", "ridge").unwrap(),
                    terrains: std::vec![ContentKey::new("t", "void").unwrap()],
                    battlefields: Vec::new(),
                    width: hexes,
                    height: 1,
                    blocked: (0..hexes.cast_signed()).map(|dx| (dx, 0)).collect(),
                    absolute: false,
                });
            },
        )
    };
    let most = u8::try_from(MAX_REMOVABLE_HEXES).unwrap();
    let r = with_ridge(most).unwrap();
    for seed in 0..20u64 {
        let obstacles = pieces(&r, "void", seed);
        assert!(!obstacles.is_empty());
        assert!(
            obstacles
                .iter()
                .all(|o| o.hexes.len() == MAX_REMOVABLE_HEXES)
        );
        let setup = BattleSetup {
            terrain: Some(terrain(&r, "void")),
            obstacles,
            ..setup(&r, &[(0, "fixed", 10)], &[(0, "fixed", 10)])
        };
        assert!(BattleState::new(&r, &setup, seed).is_ok(), "{seed}");
    }
    assert_eq!(
        with_ridge(most + 1).err(),
        Some(BattleCompileError::InvalidObstacle {
            key: String::from("t:ridge"),
            reason: "usual obstacles block at most 8 hexes",
        })
    );
}

// covers: BAT-GEO-004
#[test]
fn obstacle_offsets_follow_the_hex_rows() {
    // (1, −1) is the top-right neighbour of the anchor in odd and even rows alike.
    let r = ruleset_with("t:faithful", |_| {});
    let f = field(&r);
    let o = h3_content_battle::ObstacleType {
        key: h3_content::ContentKey::new("t", "o").unwrap(),
        terrains: Vec::new(),
        battlefields: Vec::new(),
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
