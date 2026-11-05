//! Tests of the random dwellings when the game is built (E08 T94a; increment 10's design note §5):
//! the draw and its order, the resolved object, the moved anchor at the right edge, the later
//! draws, a resolved dwelling as a placed one (owner, reveal, view, state and hash), the static
//! map's dwelling and camp lists, the name check and the refusal of a hero's and a boat's tile.

use core::num::NonZeroU16;
use std::collections::BTreeSet;
use std::format;
use std::string::ToString;

use h3_content::{ContentPack, CreatureDef, CreatureInfoDef, FactionId};
use h3_kernel::hash::hash_of;

use super::*;
use crate::ObjectView;
use crate::ruleset::{
    AdventurePack, DwellingParams, ObjectKindDef, ObjectTypeDef, RandomDwellingDef, Visit,
};
use crate::setup::{BoatSetup, Holds};
use crate::testing::{PEASANT, SEA, hero_at, key, movement_rules_with, world};

pub(super) type Rules = (Ruleset, AdventureRuleset);

const ROWS: [&str; 8] = ["gggggggggggggggg"; 8];

/// A pack's list of `(column, row)` offsets.
fn cells(offsets: &[(u8, u8)]) -> Vec<(u8, u8)> {
    offsets.to_vec()
}

/// [`movement_rules_with`] (faithful) with seven creatures of each level for the castle (`t:c1`
/// to `t:c7`), the fortress (`t:f1` to `t:f7`) and the wild (`t:w1` to `t:w7`), each of growth 2
/// costing 10 gold a level, the wild's also a wood a level above the first, and their
/// dwellings: the castle's and the wild's share `t:d1` to `t:d7` (each with both creatures of its
/// level), blocking the anchor and the cell left of it, entered at the anchor, `t:d3` also
/// blocking the cell above the left one, `t:d7` an Angel Portal (its anchor free, entered one
/// left, blocking two left and, beyond the original's, the cell above its anchor: shift 1); the
/// fortress's `t:e1` to `t:e7` entered one left of the anchor. The random dwelling `t:random`, a
/// Refugee Camp `t:camp` and a Windmill `t:windmill`; the
/// castle's, the fortress's and the wild's random dwelling tables.
pub(super) fn rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |core: &mut ContentPack| {
            for (faction, prefix) in [("t:castle", "c"), ("t:fortress", "f"), ("t:wild", "w")] {
                core.creatures.extend((1..=7).map(|level| {
                    // The wild's also cost wood, listed after the gold (the price's order is the
                    // core's: wood first), 0 at level 1.
                    let gold = (key("t:gold"), 10 * u32::from(level));
                    let cost = if prefix == "w" {
                        vec![gold, (key("t:wood"), u32::from(level) - 1)]
                    } else {
                        vec![gold]
                    };
                    CreatureDef {
                        key: key(&format!("t:{prefix}{level}")),
                        speed: 4,
                        info: CreatureInfoDef {
                            faction: Some(key(faction)),
                            level,
                            growth: 2,
                            cost,
                            ..CreatureInfoDef::default()
                        },
                    }
                }));
            }
        },
        |part: &mut AdventurePack| {
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Enter,
                from_above: false,
                kind,
            };
            for level in 1..=7 {
                let (blocked, visitable) = match level {
                    3 => (cells(&[(0, 0), (1, 0), (1, 1)]), cells(&[(0, 0)])),
                    7 => (cells(&[(1, 0), (2, 0), (0, 1)]), cells(&[(1, 0)])),
                    _ => (cells(&[(0, 0), (1, 0)]), cells(&[(0, 0)])),
                };
                part.object_types.push(ty(
                    &format!("t:d{level}"),
                    ObjectKindDef::Dwelling {
                        creatures: vec![key(&format!("t:c{level}")), key(&format!("t:w{level}"))],
                        blocked,
                        visitable,
                    },
                ));
                part.object_types.push(ty(
                    &format!("t:e{level}"),
                    ObjectKindDef::Dwelling {
                        creatures: vec![key(&format!("t:f{level}"))],
                        blocked: cells(&[(0, 0), (1, 0)]),
                        visitable: cells(&[(1, 0)]),
                    },
                ));
            }
            part.object_types.extend([
                ty("t:random", ObjectKindDef::RandomDwelling),
                ty("t:camp", ObjectKindDef::RefugeeCamp { levels: [1, 3] }),
                ty(
                    "t:windmill",
                    ObjectKindDef::Windmill {
                        resources: vec![key("t:wood"), key("t:gold")],
                        amount: [3, 6],
                    },
                ),
            ]);
            let table = |faction: &str, prefix: &str| RandomDwellingDef {
                faction: key(faction),
                dwellings: core::array::from_fn(|i| key(&format!("t:{prefix}{}", i + 1))),
            };
            part.random_dwellings = vec![
                table("t:castle", "d"),
                table("t:fortress", "e"),
                table("t:wild", "d"),
            ];
            part.params.as_mut().unwrap().dwellings = Some(DwellingParams {
                guard_level: 5,
                guard_factor: NonZeroU16::new(3).unwrap(),
                free_level: 1,
            });
        },
    )
}

pub(super) fn type_of(rules: &Rules, k: &str) -> ObjectTypeId {
    rules.1.object_type_by_key(k).unwrap()
}

fn faction(rules: &Rules, k: &str) -> FactionId {
    rules.0.faction_by_key(k).unwrap()
}

/// The mask of these `(column, row)` offsets.
pub(super) fn mask(offsets: &[(u8, u8)]) -> TileMask {
    TileMask::from_bits(
        offsets
            .iter()
            .map(|&(c, r)| 1u64 << (u32::from(r) * 8 + u32::from(c)))
            .fold(0, |a, b| a | b),
    )
}

/// An object of type `k` on `(x, y)` with these masks, ownable, holding nothing.
pub(super) fn placed(
    rules: &Rules,
    k: &str,
    (x, y): (u16, u16),
    blocked: TileMask,
    visitable: TileMask,
) -> ObjectSetup {
    ObjectSetup {
        scenery: Scenery::Ownable,
        anchor: Pos::new(x, y, 0),
        blocked,
        visitable,
        object_type: Some(type_of(rules, k)),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// A random dwelling on `(x, y)` of these factions and levels, as the import makes one: its
/// template blocks the 2 × 2 and is entered from both bottom cells.
fn random(rules: &Rules, (x, y): (u16, u16), factions: &[&str], levels: (u8, u8)) -> ObjectSetup {
    ObjectSetup {
        holds: Holds::RandomDwelling {
            factions: factions.iter().map(|k| faction(rules, k)).collect(),
            levels,
        },
        ..placed(
            rules,
            "t:random",
            (x, y),
            mask(&[(0, 0), (1, 0), (0, 1), (1, 1)]),
            mask(&[(0, 0), (1, 0)]),
        )
    }
}

/// A dwelling of type `k` placed as a random dwelling on `(x, y)` that became it would lie: its
/// footprint, its anchor moved by its shift.
fn as_resolved(rules: &Rules, k: &str, (x, y): (u16, u16)) -> ObjectSetup {
    let Some(ObjectKind::Dwelling {
        blocked,
        visitable,
        shift,
        ..
    }) = rules.1.object_type(type_of(rules, k)).map(|ty| &ty.kind)
    else {
        panic!("{k} is no dwelling");
    };
    placed(rules, k, (x + u16::from(*shift), y), *blocked, *visitable)
}

/// A grass map of 16 × 8 with red's hero 1 on (0, 7) and blue's hero 2 on (15, 7), and these
/// objects.
pub(super) fn setup(objects: Vec<ObjectSetup>) -> AdventureSetup {
    let mut setup = world(
        &ROWS,
        1,
        vec![
            hero_at(1, 0, Pos::new(0, 7, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(15, 7, 0), &[PEASANT]),
        ],
    );
    setup.map.objects = objects;
    setup
}

pub(super) fn seeded(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::SETUP)
}

pub(super) fn build_with(
    rules: &Rules,
    setup: &AdventureSetup,
    draws: &mut Rng,
) -> Result<AdventureState, SetupError> {
    let mut state = AdventureState::new(&rules.0, &rules.1, setup, draws)?;
    state.start_game(&rules.0, &rules.1, &[], &[]);
    Ok(state)
}

pub(super) fn build(
    rules: &Rules,
    setup: &AdventureSetup,
    seed: u64,
) -> Result<AdventureState, SetupError> {
    build_with(rules, setup, &mut seeded(seed))
}

// covers: ADV-DWL-005
#[test]
fn a_random_dwelling_becomes_its_table_s_dwelling() {
    // Increment 10's design note §5: one faction and one level give exactly that table's entry,
    // its footprint, the anchor (no shift), holding nothing, drawn on no further tile, the
    // setup's owner and scenery.
    let rules = rules();
    let mut object = random(&rules, (5, 3), &["t:fortress"], (4, 4));
    object.owner = Some(PlayerId::new(1));
    object.drawn = mask(&[(2, 2)]);
    let state = build(&rules, &setup(vec![object]), 0).unwrap();
    let resolved = &state.map().objects()[0];
    let e4 = rules
        .1
        .random_dwelling(faction(&rules, "t:fortress"))
        .unwrap()[3];
    assert_eq!(e4, type_of(&rules, "t:e4"));
    assert_eq!(resolved.object_type, Some(e4));
    assert_eq!(resolved.anchor, Pos::new(5, 3, 0));
    assert_eq!(resolved.blocked, mask(&[(0, 0), (1, 0)]));
    assert_eq!(resolved.visitable, mask(&[(1, 0)]));
    assert_eq!(resolved.drawn, TileMask::default());
    assert_eq!(resolved.holds, Holds::Nothing);
    assert_eq!(resolved.scenery, Scenery::Ownable);
    assert_eq!(state.owner(0), Some(PlayerId::new(1)));
    assert!(state.check_owners(&rules.1));
    // The tiles are the dwelling's: its entrance (4, 3), the anchor blocked, the upper row free.
    let tile = |x, y| state.map().tile(Pos::new(x, y, 0)).copied().unwrap();
    assert!(tile(4, 3).blocked && tile(4, 3).visitable);
    assert!(tile(5, 3).blocked && !tile(5, 3).visitable);
    assert!(!tile(4, 2).blocked && !tile(5, 2).blocked);
    assert_eq!(state.map().visited_from(Pos::new(4, 3, 0)), &[0]);
    assert_eq!(state.map().visited_from(Pos::new(5, 3, 0)), &[] as &[usize]);
    assert_eq!(resolved.entrance(state.map()), Some(Pos::new(4, 3, 0)));
    // The Angel Portal's shift moves the anchor one right: entered on the random dwelling's
    // anchor, blocking its left cell and the cell above the moved anchor.
    let state = build(
        &rules,
        &setup(vec![random(&rules, (5, 3), &["t:castle"], (7, 7))]),
        0,
    )
    .unwrap();
    let resolved = &state.map().objects()[0];
    assert_eq!(resolved.object_type, Some(type_of(&rules, "t:d7")));
    assert_eq!(resolved.anchor, Pos::new(6, 3, 0));
    assert_eq!(resolved.entrance(state.map()), Some(Pos::new(5, 3, 0)));
    let tile = |x, y| state.map().tile(Pos::new(x, y, 0)).copied().unwrap();
    assert!(tile(5, 3).visitable && tile(4, 3).blocked && tile(6, 2).blocked);
    assert!(!tile(6, 3).blocked && !tile(6, 3).visitable);
    // The footprint it takes is the type's, a dwelling's only.
    let kind = |k| &rules.1.object_type(type_of(&rules, k)).unwrap().kind;
    assert_eq!(
        kind("t:d7").dwelling_footprint(),
        Some((mask(&[(1, 0), (2, 0), (0, 1)]), mask(&[(1, 0)]), 1))
    );
    assert_eq!(kind("t:random").dwelling_footprint(), None);
    assert_eq!(kind("t:camp").dwelling_footprint(), None);
}

// covers: ADV-DWL-005
#[test]
fn the_holds_encoding_still_tells_random_dwellings_apart() {
    // No built object holds a random dwelling's set and range since E08 T94a, but the holds'
    // encoding keeps them (tag 8, the factions, the levels).
    let rules = rules();
    let holds = |factions: &[&str], levels| Holds::RandomDwelling {
        factions: factions.iter().map(|k| faction(&rules, k)).collect(),
        levels,
    };
    let reference = hash_of(&holds(&["t:castle"], (1, 7)));
    assert_eq!(hash_of(&holds(&["t:castle"], (1, 7))), reference);
    for other in [
        holds(&["t:fortress"], (1, 7)),
        holds(&["t:castle", "t:fortress"], (1, 7)),
        holds(&["t:castle"], (2, 7)),
        holds(&["t:castle"], (1, 6)),
        Holds::Nothing,
    ] {
        assert_ne!(hash_of(&other), reference, "{other:?}");
    }
}

// covers: ADV-DWL-005
#[test]
fn a_random_dwelling_draws_its_faction_then_its_level() {
    // The faction uniform among the set (`pick`: one draw below its size), then the level
    // uniform in the range: over many seeds both factions and every level come up, each draw as
    // the setup stream's first two give.
    let rules = rules();
    let tables = [
        rules
            .1
            .random_dwelling(faction(&rules, "t:castle"))
            .unwrap(),
        rules
            .1
            .random_dwelling(faction(&rules, "t:fortress"))
            .unwrap(),
    ];
    let setup = setup(vec![random(
        &rules,
        (5, 3),
        &["t:castle", "t:fortress"],
        (1, 7),
    )]);
    let mut seen = BTreeSet::new();
    for seed in 0..64 {
        let state = build(&rules, &setup, seed).unwrap();
        let mut draws = seeded(seed);
        let side = usize::try_from(draws.below(2)).unwrap();
        let level = usize::try_from(draws.range_inclusive(1, 7)).unwrap();
        let expected = tables[side][level - 1];
        assert_eq!(
            state.map().objects()[0].object_type,
            Some(expected),
            "seed {seed}"
        );
        seen.insert((side, level));
    }
    let sides: BTreeSet<_> = seen.iter().map(|&(side, _)| side).collect();
    let levels: BTreeSet<_> = seen.iter().map(|&(_, level)| level).collect();
    assert_eq!(sides.len(), 2);
    assert_eq!(levels.len(), 7);
    // A one-level range draws too: the level's draw comes after the faction's whatever its span.
    let one = |seed| {
        let setup = self::setup(vec![random(
            &rules,
            (5, 3),
            &["t:castle", "t:fortress"],
            (2, 2),
        )]);
        build(&rules, &setup, seed).unwrap().map().objects()[0].object_type
    };
    for seed in 0..16 {
        let side = usize::try_from(seeded(seed).below(2)).unwrap();
        assert_eq!(one(seed), Some(tables[side][1]), "seed {seed}");
    }
}

// covers: ADV-DWL-005
#[test]
fn a_random_dwelling_at_the_right_edge_moves_its_anchor_off_the_map() {
    // Increment 10's design note §5: the moved anchor may lie one column past the right edge,
    // where only its free cell is; the other cells are on the map as anywhere.
    let rules = rules();
    let mut object = random(&rules, (15, 3), &["t:castle"], (7, 7));
    object.owner = Some(PlayerId::new(1));
    let state = build(&rules, &setup(vec![object]), 0).unwrap();
    let map = state.map();
    let resolved = &map.objects()[0];
    assert_eq!(resolved.anchor, Pos::new(16, 3, 0));
    assert_eq!(map.index(resolved.anchor), None);
    assert_eq!(resolved.entrance(map), Some(Pos::new(15, 3, 0)));
    let tiles: Vec<Pos> = resolved.tiles(map).collect();
    assert_eq!(tiles, [Pos::new(15, 3, 0), Pos::new(14, 3, 0)]);
    let tile = |x, y| map.tile(Pos::new(x, y, 0)).copied().unwrap();
    assert!(tile(15, 3).blocked && tile(15, 3).visitable);
    assert!(tile(14, 3).blocked && !tile(14, 3).visitable);
    assert!(!tile(15, 2).blocked && !tile(14, 2).blocked);
    assert_eq!(resolved.covers(Pos::new(16, 2, 0)), (true, false));
    assert_eq!(map.visited_from(Pos::new(15, 3, 0)), &[0]);
    // Its owner sees it from its entrance, with the moved anchor.
    let view = views(&state, &rules, PlayerId::new(1)).remove(0);
    assert_eq!(view.anchor, Pos::new(16, 3, 0));
    assert_eq!(view.object_type, type_of(&rules, "t:d7"));
    assert!(state.is_explored(PlayerId::new(1), Pos::new(12, 3, 0)));
}

/// What `viewer` sees of the typed objects, in map order.
pub(super) fn views(state: &AdventureState, rules: &Rules, viewer: PlayerId) -> Vec<ObjectView> {
    state.observe(&rules.0, &rules.1, viewer).objects
}

// covers: ADV-DWL-005
#[test]
fn the_random_dwellings_draw_before_the_static_map_s_other_draws() {
    // A random pile and a Windmill before a random dwelling in object order draw after it: as
    // the same map without the random dwelling built from the stream two draws on.
    let rules = rules();
    let others = || {
        let pile = placed(
            &rules,
            "t:any_pile",
            (2, 1),
            mask(&[(0, 0)]),
            mask(&[(0, 0)]),
        );
        let windmill = placed(
            &rules,
            "t:windmill",
            (4, 1),
            mask(&[(0, 0)]),
            mask(&[(0, 0)]),
        );
        vec![pile, windmill]
    };
    let drawn = |state: &AdventureState| {
        let pile = &state.map().objects()[0];
        (
            pile.object_type,
            pile.contents.clone(),
            state.weekly(1).cloned(),
        )
    };
    let mut with = others();
    with.push(random(&rules, (9, 3), &["t:castle", "t:fortress"], (1, 7)));
    let mut moved = false;
    for seed in 0..8 {
        let state = build(&rules, &setup(with.clone()), seed).unwrap();
        let mut later = seeded(seed);
        later.below(2);
        later.range_inclusive(1, 7);
        let alone = build_with(&rules, &setup(others()), &mut later).unwrap();
        assert_eq!(drawn(&state), drawn(&alone), "seed {seed}");
        assert!(state.weekly(1).is_some());
        // So the random dwelling moves them: the same seed draws otherwise without it.
        moved |= drawn(&state) != drawn(&build(&rules, &setup(others()), seed).unwrap());
    }
    assert!(moved);
}

// covers: ADV-DWL-005, ADV-OWN-002
#[test]
fn nothing_remembers_a_random_dwelling_was_random() {
    // A random dwelling that becomes T builds the state a placed T on the moved anchor with T's
    // footprint and the same owner builds, on a map with no other draw: equal, and equal hashes.
    let rules = rules();
    for (factions, levels, k) in [
        (&["t:fortress"][..], (5, 5), "t:e5"),
        (&["t:castle"][..], (7, 7), "t:d7"),
        (&["t:castle"][..], (3, 3), "t:d3"),
    ] {
        let mut object = random(&rules, (8, 4), factions, levels);
        object.owner = Some(PlayerId::new(0));
        let mut dwelling = as_resolved(&rules, k, (8, 4));
        dwelling.owner = Some(PlayerId::new(0));
        for seed in 0..4 {
            let state = build(&rules, &setup(vec![object.clone()]), seed).unwrap();
            let same = build(&rules, &setup(vec![dwelling.clone()]), seed).unwrap();
            assert_eq!(state, same, "{k}");
            assert_eq!(hash_of(&state), hash_of(&same), "{k}");
        }
    }
}

// covers: ADV-DWL-005
#[test]
fn the_static_map_lists_the_dwellings_and_the_camps() {
    // Increment 10's design note §5: the dwelling index (placed and resolved alike, in object
    // order) and the camp index; the dwellings are ownable, the camps not.
    let rules = rules();
    let one = |k, x| placed(&rules, k, (x, 1), mask(&[(0, 0)]), mask(&[(0, 0)]));
    let objects = vec![
        one("t:d1", 1),
        one("t:camp", 3),
        random(&rules, (6, 3), &["t:fortress"], (1, 7)),
        one("t:windmill", 8),
        one("t:camp", 10),
        one("t:e2", 12),
    ];
    let state = build(&rules, &setup(objects), 0).unwrap();
    let map = state.map();
    assert_eq!(map.dwellings(), &[0, 2, 5]);
    assert_eq!(map.camps(), &[1, 4]);
    assert_eq!(map.ownables(), &[0, 2, 5]);
    assert_eq!(map.weeklies(), &[3]);
    for (index, dwelling, camp) in [
        (0, Some(0), None),
        (1, None, Some(0)),
        (2, Some(1), None),
        (3, None, None),
        (4, None, Some(1)),
        (5, Some(2), None),
        (6, None, None),
        (99, None, None),
    ] {
        assert_eq!(map.dwelling_index(index), dwelling, "{index}");
        assert_eq!(map.camp_index(index), camp, "{index}");
    }
    // A map without either lists none.
    let state = build(&rules, &setup(vec![]), 0).unwrap();
    assert!(state.map().dwellings().is_empty() && state.map().camps().is_empty());
}

// covers: ADV-OWN-002, ADV-OWN-003
#[test]
fn a_resolved_dwelling_s_owner_sees_around_its_entrance_and_every_viewer_sees_it_resolved() {
    // Blue's random dwelling on (8, 1) becomes a fortress dwelling entered on (7, 1): blue sees
    // the disk of radius 3 around the entrance, red nothing of it; the view shows the drawn type,
    // its footprint, the anchor and the owner as a placed dwelling's.
    let rules = rules();
    let mut object = random(&rules, (8, 1), &["t:fortress"], (2, 2));
    object.owner = Some(PlayerId::new(1));
    let state = build(&rules, &setup(vec![object]), 0).unwrap();
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    let explored = |player, x, y| state.is_explored(player, Pos::new(x, y, 0));
    for x in [4, 7, 10] {
        assert!(explored(blue, x, 1) && !explored(red, x, 1), "{x}");
    }
    assert!(!explored(blue, 3, 1) && !explored(blue, 11, 1));
    let views = views(&state, &rules, blue);
    assert_eq!(views.len(), 1);
    let view = &views[0];
    assert_eq!(view.object_type, type_of(&rules, "t:e2"));
    assert_eq!(view.anchor, Pos::new(8, 1, 0));
    assert_eq!(view.blocked, mask(&[(0, 0), (1, 0)]));
    assert_eq!(view.visitable, mask(&[(1, 0)]));
    assert_eq!(view.drawn, TileMask::default());
    assert_eq!(view.owner, Some(blue));
    // As the view of a placed one.
    let mut dwelling = as_resolved(&rules, "t:e2", (8, 1));
    dwelling.owner = Some(blue);
    let same = build(&rules, &setup(vec![dwelling]), 0).unwrap();
    assert_eq!(self::views(&same, &rules, blue), views);
    // The Angel Portal's view has the moved anchor.
    let mut object = random(&rules, (8, 1), &["t:castle"], (7, 7));
    object.owner = Some(red);
    let state = build(&rules, &setup(vec![object]), 0).unwrap();
    let view = self::views(&state, &rules, red).remove(0);
    assert_eq!(view.anchor, Pos::new(9, 1, 0));
    assert_eq!(view.blocked, mask(&[(1, 0), (2, 0), (0, 1)]));
    assert_eq!(view.owner, Some(red));
    assert!(
        state.is_explored(red, Pos::new(5, 1, 0)) && !state.is_explored(red, Pos::new(4, 1, 0))
    );
}

// covers: ADV-DWL-005
#[test]
fn a_hero_may_not_stand_where_a_random_dwelling_could_lie() {
    // An agent's call generalising increment 10's design note §5: a hero on the random dwelling's
    // visitable tiles, or on a tile an outcome of its set and range covers, is refused whatever
    // the draw, before its tile is checked; elsewhere it is not.
    let rules = rules();
    let hero = HeroId::new(3);
    let with_hero = |objects: Vec<ObjectSetup>, (x, y): (u16, u16)| {
        let mut setup = setup(objects);
        setup
            .heroes
            .push(hero_at(3, 0, Pos::new(x, y, 0), &[PEASANT]));
        setup
    };
    let castle = || random(&rules, (5, 3), &["t:castle"], (1, 7));
    let refused = |object: usize| -> Result<AdventureState, SetupError> {
        Err(SetupError::RandomDwellingTile { hero, object })
    };
    for seed in 0..16 {
        // Both bottom cells, every outcome's cells: the cell above the moved anchor (the Angel
        // Portal's of this pack), the cell above the left one (`t:d3`).
        for at in [(5, 3), (4, 3), (6, 2), (4, 2)] {
            assert_eq!(
                build(&rules, &with_hero(vec![castle()], at), seed),
                refused(0),
                "{at:?}"
            );
        }
        // The moved anchor's free cell and the upper row's other cell no outcome covers.
        for at in [(6, 3), (5, 2), (3, 3)] {
            build(&rules, &with_hero(vec![castle()], at), seed).unwrap();
        }
        // Without the Angel Portal among the outcomes the cell above its anchor is free; the
        // fortress's dwellings cover only the bottom cells, which the random dwelling's entrances
        // refuse anyway.
        for (factions, levels) in [(&["t:castle"][..], (1, 6)), (&["t:fortress"][..], (1, 7))] {
            let object = random(&rules, (5, 3), factions, levels);
            build(&rules, &with_hero(vec![object.clone()], (6, 2)), seed).unwrap();
            assert_eq!(
                build(&rules, &with_hero(vec![object], (4, 3)), seed),
                refused(0)
            );
        }
    }
    // The first random dwelling that could cover the tile is named; a later object, the hero's
    // own checks before it (a duplicate, an unknown owner) come first.
    let objects = vec![
        placed(&rules, "t:d1", (1, 1), mask(&[(0, 0)]), mask(&[(0, 0)])),
        random(&rules, (6, 3), &["t:fortress"], (1, 7)),
        random(&rules, (5, 3), &["t:fortress"], (1, 7)),
    ];
    assert_eq!(
        build(&rules, &with_hero(objects.clone(), (5, 3)), 0),
        refused(1)
    );
    assert_eq!(
        build(&rules, &with_hero(objects.clone(), (4, 3)), 0),
        refused(2)
    );
    let mut twice = with_hero(objects.clone(), (5, 3));
    twice.heroes[2].id = HeroId::new(1);
    assert_eq!(
        build(&rules, &twice, 0),
        Err(SetupError::DuplicateHero(HeroId::new(1)))
    );
    let mut stranger = with_hero(objects, (5, 3));
    stranger.heroes[2].owner = PlayerId::new(5);
    assert_eq!(
        build(&rules, &stranger, 0),
        Err(SetupError::UnknownOwner(hero))
    );
    assert_eq!(
        SetupError::RandomDwellingTile { hero, object: 2 }.to_string(),
        "hero 3 stands where random dwelling 2 could lie"
    );
}

// covers: ADV-DWL-005
#[test]
fn the_name_check_sees_what_a_random_dwelling_may_become() {
    // A player names a typed object by its type and anchor: a random dwelling takes every
    // outcome's type and moved anchor, so the drawn one is checked and the setup's validity does
    // not depend on the draw; its own type and anchor are no name.
    let rules = rules();
    // Each seed's error, repeats removed: one entry when the draw does not matter.
    let shared = |objects: Vec<ObjectSetup>| {
        let mut errors: Vec<_> = (0..8)
            .map(|seed| build(&rules, &setup(objects.clone()), seed).err())
            .collect();
        errors.dedup();
        errors
    };
    let refused = vec![Some(SetupError::SharedAnchor(1))];
    let accepted = vec![None];
    let fortress = |levels| random(&rules, (5, 3), &["t:fortress"], levels);
    let e4 = as_resolved(&rules, "t:e4", (5, 3));
    assert_eq!(shared(vec![e4.clone(), fortress((4, 4))]), refused);
    assert_eq!(shared(vec![fortress((4, 4)), e4.clone()]), refused);
    // A possible outcome only: refused whatever is drawn.
    assert_eq!(shared(vec![e4.clone(), fortress((1, 7))]), refused);
    assert_eq!(shared(vec![e4, fortress((1, 3))]), accepted);
    // The Angel Portal's name has the moved anchor.
    let angel = random(&rules, (5, 3), &["t:castle"], (7, 7));
    let d7 = |x| placed(&rules, "t:d7", (x, 3), mask(&[(1, 0)]), mask(&[(1, 0)]));
    assert_eq!(shared(vec![d7(6), angel.clone()]), refused);
    assert_eq!(shared(vec![d7(5), angel]), accepted);
    // Two random dwellings on one anchor clash where their outcomes meet, not by their own type.
    assert_eq!(shared(vec![fortress((1, 3)), fortress((3, 5))]), refused);
    assert_eq!(shared(vec![fortress((1, 2)), fortress((3, 4))]), accepted);
    // Two factions sharing a dwelling give one name once: no clash with itself.
    let both = random(&rules, (5, 3), &["t:castle", "t:wild"], (1, 7));
    assert_eq!(shared(vec![both]), accepted);
}

// covers: ADV-DWL-005
#[test]
fn a_boat_may_not_lie_where_a_random_dwelling_could_lie() {
    // As a hero's tile (an agent's call, E08 T94a): a boat on a water tile an outcome of the
    // random dwelling's set and range covers is refused whatever the draw; without that outcome
    // it is not.
    let rules = rules();
    let with_boat = |object: ObjectSetup| {
        let mut setup = setup(vec![object]);
        setup.map.tiles[2 * 16 + 6].terrain = SEA;
        setup.map.boats.push(BoatSetup {
            tile: Pos::new(6, 2, 0),
            kind: 0,
        });
        setup
    };
    for seed in 0..16 {
        // The cell above the Angel Portal's moved anchor (this pack's `t:d7`).
        let castle = random(&rules, (5, 3), &["t:castle"], (1, 7));
        assert_eq!(
            build(&rules, &with_boat(castle), seed),
            Err(SetupError::Boat(0))
        );
        let lower = random(&rules, (5, 3), &["t:castle"], (1, 6));
        build(&rules, &with_boat(lower), seed).unwrap();
    }
}
