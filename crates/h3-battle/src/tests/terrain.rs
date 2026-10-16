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

/// A usual obstacle type: `(name, width, height, blocked hexes)`.
type UsualType<'a> = (&'a str, u8, u8, &'a [(i8, i8)]);

/// The test ruleset with the usual obstacle types `types` on the void terrain, in this order.
fn void_with(types: &[UsualType<'_>]) -> BattleRuleset {
    use h3_content::ContentKey;
    ruleset_from(
        "t:faithful",
        |_| {},
        |_, battle| {
            for &(name, width, height, blocked) in types {
                battle.obstacles.push(h3_content_battle::ObstacleDef {
                    key: ContentKey::new("t", name).unwrap(),
                    terrains: std::vec![ContentKey::new("t", "void").unwrap()],
                    battlefields: Vec::new(),
                    width,
                    height,
                    blocked: blocked.to_vec(),
                    absolute: false,
                });
            }
        },
    )
}

// covers: BAT-GEO-004
#[test]
fn generated_obstacles_block_five_to_twelve_hexes_away_from_the_armies() {
    // Eight two-hex stone types: each is drawn at most once, so they never run out here.
    let stone: &[(i8, i8)] = &[(0, 0), (1, 0)];
    let r = void_with(&["s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8"].map(|n| (n, 2, 1, stone)));
    let f = field(&r);
    let mut seen = BTreeSet::new();
    let mut in_row_one = false;
    for seed in 0..200u64 {
        let hexes = generate(&r, "void", seed);
        assert_eq!(hexes, generate(&r, "void", seed), "deterministic");
        // Only two-hex stones: 5..=12 hexes to block means 3..=6 stones.
        assert!(
            (6..=12).contains(&hexes.len()) && hexes.len().is_multiple_of(2),
            "{hexes:?}"
        );
        assert!(hexes.windows(2).all(|w| w[0] < w[1]), "sorted, each once");
        for &h in &hexes {
            let (x, y) = f.coords(crate::Hex(h));
            assert!((3..=13).contains(&x) && (1..=9).contains(&y), "({x}, {y})");
            in_row_one |= y == 1;
        }
        seen.insert(hexes.clone());
        // Each stone keeps its anchor, its bottom-left hex (Q55), and is of another type.
        let ps = pieces(&r, "void", seed);
        assert!(
            ps.iter()
                .all(|o| o.hexes.len() == 2 && o.anchor == Some(o.hexes[0])),
            "{ps:?}"
        );
        let kinds: BTreeSet<_> = ps
            .iter()
            .map(|o| o.kind.as_ref().map(|k| String::from(k.as_str())))
            .collect();
        assert_eq!(kinds.len(), ps.len(), "{ps:?}");
        // The setup is valid and the armies still find their places.
        let s = BattleSetup {
            obstacles: ps,
            ..setup(&r, &[(0, "serf", 1)], &[(0, "serf", 1)])
        };
        BattleState::new(&r, &s, seed).unwrap();
    }
    assert!(seen.len() > 150, "varied layouts: {}", seen.len());
    // A height-1 stone may lie in row 1 (the original's 0x46608D).
    assert!(in_row_one);
}

// covers: BAT-GEO-004
#[test]
fn an_anchor_may_lie_in_the_row_of_its_obstacles_height() {
    // The original's 0x466010 refuses an anchor only in a row above its type's height
    // (0x46608D: the row below the height); VCMI refused that row too.
    let r = ruleset_with("t:faithful", |_| {});
    let f = field(&r);
    for height in [1u8, 2] {
        let o = h3_content_battle::ObstacleType {
            key: h3_content::ContentKey::new("t", "o").unwrap(),
            terrains: Vec::new(),
            battlefields: Vec::new(),
            width: 1,
            height,
            blocked: std::vec![(0, 0)],
            absolute: false,
        };
        let row = i32::from(height);
        let anchor = at(&r, 5, row);
        assert_eq!(
            crate::terrain::placement_for_tests(f, &o, anchor),
            Some(std::vec![anchor.0])
        );
        assert_eq!(
            crate::terrain::placement_for_tests(f, &o, at(&r, 5, row - 1)),
            None
        );
    }
}

/// The original draws each usual type at most once in a battle (its sampler over the types is
/// set up once, 0x4667F8) and goes on with the next type after one without a place (0x466858):
/// one-hex types `a`, `b` and `c` and a `spire` too tall for the field.
// covers: BAT-GEO-004
#[test]
fn usual_types_are_drawn_once_each_and_one_without_a_place_is_passed_over() {
    let one: &[(i8, i8)] = &[(0, 0)];
    let r = void_with(&[
        ("a", 1, 1, one),
        ("spire", 1, 10, one),
        ("b", 1, 1, one),
        ("c", 1, 1, one),
    ]);
    let f = field(&r);
    let void = terrain(&r, "void");
    let types: Vec<_> = r
        .obstacles()
        .iter()
        .filter(|o| o.terrains.contains(&void))
        .cloned()
        .collect();
    let below =
        |rng: &mut Rng, n: usize| usize::try_from(rng.below(u32::try_from(n).unwrap())).unwrap();
    for seed in 0..50u64 {
        let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
        let mut model = rng.clone();
        let got = generate_obstacles(&r, void, &mut rng);
        // The draws in the original's order: 5..=12 hexes to block, the 40 % roll (no absolute
        // type here), then while hexes remain a type among those not drawn yet, in the
        // ruleset's order, and, if it has a place, an anchor among its places.
        let to_block = model.range_inclusive(5, 12);
        model.range_inclusive(1, 100);
        let mut left = types.clone();
        let (mut blocked, mut expected) = (BTreeSet::new(), Vec::new());
        while u32::try_from(blocked.len()).unwrap() < to_block && !left.is_empty() {
            let o = left.remove(below(&mut model, left.len()));
            let places: Vec<u8> = (18..=168u8)
                .filter(|&h| {
                    !blocked.contains(&h)
                        && crate::terrain::placement_for_tests(f, &o, crate::Hex(h)).is_some()
                })
                .collect();
            if places.is_empty() {
                continue;
            }
            let anchor = places[below(&mut model, places.len())];
            blocked.insert(anchor);
            expected.push(h3_contracts::Obstacle {
                hexes: std::vec![anchor],
                anchor: Some(anchor),
                kind: Some(o.key.clone()),
            });
        }
        assert_eq!(got, expected, "{seed}");
        assert_eq!(rng, model, "the same draws: {seed}");
        // 5..=12 hexes to block outlast the three types that have a place.
        assert_eq!(got.len(), 3);
    }
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
            // The cliff counts 2 of 5..=12; 3..=10 remain, and the lava's one usual type, the
            // stone, comes at most once (the original's 0x4667F8).
            assert_eq!(kinds, [Some("t:cliff".into()), Some("t:stone".into())]);
            assert_eq!(hexes.len(), 6, "{hexes:?}");
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
