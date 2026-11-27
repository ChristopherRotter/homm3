//! Tests of Summon Boat and Scuttle Boat (E08 T143a; increment 15's design note §10.1, §10.2,
//! §11.3; ADV-SPL-003, ADV-SPL-004): the refusals, the chance drawn from the caster's player's
//! stream, Summon Boat's choice under `ADV-QUIRK-53` and its made boat up to `boats_most`, Scuttle
//! Boat's window and hidden targets under `ADV-QUIRK-52` with the reveal, and the events each
//! viewer is told. Batch E15's rows as sources: E15-4 (a nobody's boat summoned), E15-9a (the
//! hero's last boat from the underground), E15-9b (the player's own boat over another player's
//! nearer one), E15-10 (a hidden empty boat a target and sunk, nothing revealed).

use alloc::sync::Arc;
use core::num::NonZeroU16;

use h3_content::{SpellCategory, SpellDef, SpellId, SpellSchoolDef};
use h3_heroes::HeroDetails;

use super::*;
use crate::command::{AdventureEvent, AdventureEventKind, Viewers};
use crate::ruleset::{AdventureSpell, SpellParams};
use crate::testing::{PEASANT, hero_at, hero_details, key, movement_rules_with, started, world};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
const BLUE_HERO: HeroId = HeroId::new(2);
/// The core's two test spells, in order: Summon Boat's and Scuttle Boat's chance (50, 50, 75,
/// 100) as the original's.
const SUMMON: SpellId = SpellId::from_raw(0);
const SCUTTLE: SpellId = SpellId::from_raw(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// The movement rules (`faithful` with both flags) with the two spells and `[params.spells]`, at
/// most `boats_most` boats in use for a made one.
fn rules(faithful: bool, boats_most: u16) -> Rules {
    movement_rules_with(
        faithful,
        16,
        |core| {
            core.spell_schools = vec![SpellSchoolDef {
                key: key("t:water"),
                name: "Water".into(),
                protection_rank: 0,
            }];
            core.spells = ["t:summon", "t:scuttle"]
                .into_iter()
                .map(|k| SpellDef {
                    key: key(k),
                    name: "Spell".into(),
                    category: SpellCategory::Adventure,
                    level: 1,
                    schools: vec![key("t:water")],
                    cost: [8, 7, 7, 7],
                    power: 0,
                    effect: [50, 50, 75, 100],
                    granted_by_name_only: false,
                })
                .collect();
        },
        |part| {
            part.params.as_mut().unwrap().spells = Some(SpellParams {
                door_points: [300, 300, 300, 200],
                target_range: [9, 8],
                boats_most: NonZeroU16::new(boats_most).unwrap(),
            });
        },
    )
}

fn store() -> Vec<Arc<HeroDetails>> {
    (0..3).map(|_| Arc::new(hero_details(&[]))).collect()
}

/// The started state of `rows` (`levels` levels), red's hero 1 on `red` and blue's hero 2 on
/// `blue`.
fn boat_world(rules: &Rules, (rows, levels): (&[&str], u8), red: Pos, blue: Pos) -> AdventureState {
    let heroes = vec![
        hero_at(1, 0, red, &[PEASANT]),
        hero_at(2, 1, blue, &[PEASANT]),
    ];
    started(rules, &world(rows, levels, heroes)).unwrap()
}

const fn summon(mastery: u8) -> Cast {
    Cast {
        spell: SUMMON,
        effect: AdventureSpell::SummonBoat,
        mastery,
        target: None,
    }
}

const fn scuttle(mastery: u8, target: Pos) -> Cast {
    Cast {
        spell: SCUTTLE,
        effect: AdventureSpell::ScuttleBoat,
        mastery,
        target: Some(target),
    }
}

/// A stream whose first draw of 1 to 100 is above 50 (`fails`) or not: a chance of 50 fails or
/// passes on it.
fn draws(fails: bool) -> Rng {
    (0..)
        .map(Rng::from_seed)
        .find(|rng| (rng.clone().range_inclusive(1, 100) > 50) == fails)
        .unwrap()
}

/// What red's hero 1's cast told and did.
type Made = (Result<(), CastFailure>, Vec<AdventureEvent>);

/// Red's hero 1's cast `cast`, checked, then made with `draws` as the game makes it.
fn cast(
    state: &mut AdventureState,
    rules: &Rules,
    cast: Cast,
    draws: &mut Rng,
) -> Result<Made, CastError> {
    let store = store();
    let rules = (&rules.0, &rules.1);
    state.check_cast(rules, &store, (RED, HERO), cast)?;
    let mut events = Vec::new();
    let made = state.apply_cast(
        (rules.0, rules.1, &store),
        (RED, HERO),
        cast,
        draws,
        &mut events,
    );
    Ok((made, events))
}

fn explored(state: &AdventureState, player: PlayerId, pos: Pos) -> bool {
    state.explored(player).unwrap().get(pos)
}

/// The summoned boat's event as emitted.
fn summoned(from: Pos, to: Pos, (seen_from, seen_to): (Viewers, Viewers)) -> AdventureEvent {
    AdventureEvent {
        kind: AdventureEventKind::BoatSummoned {
            boat: 0,
            from: Some(from),
            to: Some(to),
            facing: 2,
            seen_from,
            seen_to,
        },
        seen_by: seen_from.union(seen_to),
    }
}

// covers: ADV-SPL-003, ADV-SPL-010
#[test]
fn summon_boat_brings_the_hero_s_last_boat_from_any_level_in_both_settings() {
    // E15-9a: the hero's last boat (its captain, nobody aboard) comes first, from the underground,
    // over a nobody's boat on the hero's level, whatever the distance; its facing is kept and its
    // record keeps its owner. Told by the tiles each viewer sees: blue sees the boat leave the
    // underground, red sees it come beside its hero.
    let rows = [
        "g~~~~~~~~~",
        "~~~~~~~~~~",
        "~~b~~~~~~~",
        "gggggggggg",
        "gggggggggg",
        "gggggggg~b",
        "gggggggggg",
        "gggggggggg",
    ];
    for faithful in [true, false] {
        let rules = rules(faithful, 64);
        let mut state = boat_world(&rules, (&rows, 2), p(0, 0), Pos::new(8, 3, 1));
        state.mark_boat(1, (Some(RED), Some(HERO)));
        let from = Pos::new(9, 1, 1);
        let (red, blue) = (Viewers::NONE.with(RED), Viewers::NONE.with(BLUE));
        let made = cast(&mut state, &rules, summon(3), &mut draws(true)).unwrap();
        assert_eq!(made, (Ok(()), vec![summoned(from, p(1, 0), (blue, red))]));
        let boat = state.boat(1).unwrap();
        assert_eq!(
            (boat.tile, boat.facing, boat.owner, boat.captain),
            (p(1, 0), 2, Some(RED), Some(HERO))
        );
        assert!(state.is_empty_boat(p(1, 0)) && !state.is_empty_boat(from));
        assert_eq!(state.boat(0).unwrap().tile, p(2, 2));
        assert!(state.check_boats(&rules.1));
        // Each viewer is told the end it sees.
        let told = |viewer| match made.1[0].project(viewer).unwrap().kind {
            AdventureEventKind::BoatSummoned { from, to, .. } => Some((from, to)),
            _ => None,
        };
        assert_eq!(told(RED), Some((None, Some(p(1, 0)))));
        assert_eq!(told(BLUE), Some((Some(from), None)));
    }
}

// covers: ADV-SPL-003
#[test]
fn summon_boat_brings_the_nearest_own_or_nobody_s_boat_by_steps_or_by_distance() {
    // E15-9b: never another player's boat (blue's on (1, 4), nearest by distance). Under
    // `ADV-QUIRK-53` (faithful) by |dx| + |dy|, levels ignored: the underground's (2, 2) at 4
    // steps before (5, 0) at 5 and red's (3, 3) at 6; off by dx² + dy² on the hero's level: red's
    // (3, 3) at 18 before (5, 0) at 25.
    let rows = [
        "g~~~~b~~~~~~",
        "~~~~~~~~~~~~",
        "~~~~~~~~~~~~",
        "~~~b~~~~~~~~",
        "~b~~~~~~~~~~",
        "gggggggggggg",
        "gggggggggggg",
        "gggggggggggg",
        "g~b~gggggggg",
        "gggggggggggg",
        "gggggggggggg",
        "gggggggggggg",
    ];
    let red = Viewers::NONE.with(RED);
    for (faithful, chosen, from, seen_from) in [
        (true, 3, Pos::new(2, 2, 1), Viewers::NONE),
        (false, 1, p(3, 3), red),
    ] {
        let rules = rules(faithful, 64);
        let mut state = boat_world(&rules, (&rows, 2), p(0, 0), Pos::new(11, 5, 1));
        state.mark_boat(1, (Some(RED), None));
        state.mark_boat(2, (Some(BLUE), Some(BLUE_HERO)));
        let made = cast(&mut state, &rules, summon(3), &mut draws(true)).unwrap();
        assert_eq!(
            made,
            (Ok(()), vec![summoned(from, p(1, 0), (seen_from, red))])
        );
        assert_eq!(state.boat(chosen).unwrap().tile, p(1, 0));
        assert_eq!(state.boat(2).unwrap().tile, p(1, 4));
    }
}

// covers: ADV-SPL-003
#[test]
fn a_tie_goes_to_the_later_boat_by_steps_and_to_the_lower_by_distance() {
    // Two nobody's boats 2 steps and 2 × 2 away: `summon_boat_choice`'s `jg` keeps the later
    // (faithful); the fixed side the lower index.
    let rows = ["g~b~", "~~~~", "b~~~", "gggg"];
    for (faithful, chosen) in [(true, 1), (false, 0)] {
        let rules = rules(faithful, 64);
        let mut state = boat_world(&rules, (&rows, 1), p(0, 0), p(3, 3));
        cast(&mut state, &rules, summon(0), &mut draws(false))
            .unwrap()
            .0
            .unwrap();
        assert_eq!(state.boat(chosen).unwrap().tile, p(1, 0));
        assert_eq!(
            state.boat(1 - chosen).unwrap().tile,
            [p(2, 0), p(0, 2)][1 - chosen]
        );
    }
}

// covers: ADV-SPL-003, ADV-SPL-010
#[test]
fn summon_boat_makes_a_boat_at_advanced_or_expert_up_to_the_most_in_use() {
    // With only another player's boat on the map: at None and Basic no boat (GENRLTXT 336,
    // charged); at Advanced a boat of type 0 of red's, facing 2, at the next index; with as many
    // boats in use as `boats_most` none at Expert either. A map without boats gets its first.
    let rows = ["g~~~", "~~~~", "~~~b", "gggg"];
    let rules64 = rules(true, 64);
    let mut state = boat_world(&rules64, (&rows, 1), p(0, 0), p(3, 3));
    state.mark_boat(0, (Some(BLUE), Some(BLUE_HERO)));
    let before = state.clone();
    for mastery in [0, 1] {
        let made = cast(&mut state, &rules64, summon(mastery), &mut draws(false)).unwrap();
        assert_eq!(made, (Err(CastFailure::NoBoat), vec![]));
        assert_eq!(state, before);
    }
    let made = cast(&mut state, &rules64, summon(2), &mut draws(false)).unwrap();
    let created = AdventureEvent {
        kind: AdventureEventKind::BoatCreated {
            tile: p(1, 0),
            boat_type: 0,
        },
        seen_by: Viewers::NONE.with(RED).with(BLUE),
    };
    assert_eq!(made, (Ok(()), vec![created]));
    let boat = *state.boat(1).unwrap();
    assert_eq!(
        (boat.tile, boat.kind, boat.facing, boat.owner, boat.captain),
        (p(1, 0), 0, 2, Some(RED), None)
    );
    assert!(state.check_boats(&rules64.1));
    // One boat in use, at most one.
    let rules1 = rules(true, 1);
    let mut full = before.clone();
    let made = cast(&mut full, &rules1, summon(3), &mut draws(false)).unwrap();
    assert_eq!(made, (Err(CastFailure::NoBoat), vec![]));
    assert_eq!(full, before);
    // No boat on the map.
    let mut bare = boat_world(
        &rules64,
        (&["g~~~", "~~~~", "~~~~", "gggg"], 1),
        p(0, 0),
        p(3, 3),
    );
    cast(&mut bare, &rules64, summon(3), &mut draws(true))
        .unwrap()
        .0
        .unwrap();
    assert_eq!(bare.boats().count(), 1);
    assert!(bare.is_empty_boat(p(1, 0)) && bare.check_boats(&rules64.1));
}

// covers: ADV-SPL-003
#[test]
fn summon_boat_draws_its_chance_first_and_takes_the_first_free_water_clockwise() {
    // The draw above the chance fails, charged (GENRLTXT 337), before any boat; at Expert's 100
    // it never fails. The tile: the first of the eight from up, clockwise, on water with no boat:
    // (1, 0) holds the boat, so (2, 0), up and to the right.
    let rows = ["gb~g", "gg~g", "gggg"];
    let rules = rules(true, 64);
    let mut state = boat_world(&rules, (&rows, 1), p(1, 1), p(3, 2));
    let before = state.clone();
    let made = cast(&mut state, &rules, summon(0), &mut draws(true)).unwrap();
    assert_eq!(made, (Err(CastFailure::Chance), vec![]));
    assert_eq!(state, before);
    let made = cast(&mut state, &rules, summon(3), &mut draws(true)).unwrap();
    let seen = Viewers::NONE.with(RED).with(BLUE);
    assert_eq!(
        made,
        (Ok(()), vec![summoned(p(1, 0), p(2, 0), (seen, seen))])
    );
}

// covers: ADV-SPL-003
#[test]
fn summon_boat_is_refused_in_a_boat_and_without_free_water() {
    // GENRLTXT 334 on water (in a boat), 335 without a free water neighbour: an empty boat or a
    // hero in a boat takes one; both free, drawing nothing.
    let rules = rules(true, 64);
    let rows = ["gbgg", "ggb~", "gggg"];
    let mut state = boat_world(&rules, (&rows, 1), p(1, 1), p(3, 2));
    let mut stream = draws(true);
    let unused = stream.clone();
    assert_eq!(
        cast(&mut state, &rules, summon(3), &mut stream),
        Err(CastError::NoWater)
    );
    state.put_aboard(BLUE_HERO, 1);
    assert_eq!(
        cast(&mut state, &rules, summon(3), &mut stream),
        Err(CastError::NoWater)
    );
    // At the map's corners, the neighbours off it are no water (the steps above the top and left
    // of the edge, and those past the right and bottom).
    let land = ["ggg", "ggg", "ggg"];
    for (red, blue) in [(p(0, 0), p(2, 2)), (p(2, 2), p(0, 0))] {
        let mut corner = boat_world(&rules, (&land, 1), red, blue);
        assert_eq!(
            cast(&mut corner, &rules, summon(3), &mut stream),
            Err(CastError::NoWater)
        );
    }
    let mut afloat = boat_world(&rules, (&["g~~", "~b~", "ggg"], 1), p(0, 0), p(2, 2));
    afloat.put_aboard(HERO, 0);
    assert_eq!(
        cast(&mut afloat, &rules, summon(3), &mut stream),
        Err(CastError::InBoat)
    );
    assert_eq!(stream, unused);
}

/// Scuttle Boat's map: red's hero on (0, 0) sees to dx² + dy² ≤ 30; empty boats on (4, 0)
/// (explored) and (8, 0) (hidden), boat 2 on (2, 1) with blue's hero aboard (it started on
/// (0, 4), seeing neither empty boat); (9, 0) hidden water.
fn scuttle_world(rules: &Rules) -> AdventureState {
    let rows = [
        "g~~~b~~~b~~~",
        "~~b~~~~~~~~~",
        "~~~~~~~~~~~~",
        "~~~~~~~~~~~~",
        "gggggggggggg",
    ];
    let mut state = boat_world(rules, (&rows, 1), p(0, 0), p(0, 4));
    state.put_aboard(BLUE_HERO, 2);
    state
}

// covers: ADV-SPL-004
#[test]
fn scuttle_boat_is_refused_outside_its_window_and_without_an_empty_boat() {
    // The window 9 × 8 on the hero's level, on the map; an explored tile without an empty boat,
    // a boat with a hero aboard (the hero's tile); under `ADV-QUIRK-52` a hidden tile without one
    // too (the original's no cursor), off a target. Free, drawing nothing.
    for faithful in [true, false] {
        let rules = rules(faithful, 64);
        let mut state = scuttle_world(&rules);
        let mut stream = draws(false);
        let unused = stream.clone();
        let refused = |state: &mut AdventureState, stream: &mut Rng, target| {
            cast(state, &rules, scuttle(3, target), stream).err()
        };
        for target in [p(10, 0), Pos::new(4, 0, 1), p(4, 9)] {
            assert_eq!(
                refused(&mut state, &mut stream, target),
                Some(CastError::OutOfWindow)
            );
        }
        for target in [p(1, 0), p(2, 1)] {
            assert_eq!(
                refused(&mut state, &mut stream, target),
                Some(CastError::NoBoat)
            );
        }
        let hidden = refused(&mut state, &mut stream, p(9, 0));
        assert_eq!(hidden, faithful.then_some(CastError::NoBoat));
        if faithful {
            assert_eq!(stream, unused);
        }
    }
}

// covers: ADV-SPL-004
#[test]
fn scuttle_boat_sinks_a_seen_empty_boat_by_its_chance() {
    // Above the chance (GENRLTXT 338): charged, the boat stays; at Expert sunk, its record out of
    // use, told to those that see the tile.
    let rules = rules(false, 64);
    let mut state = scuttle_world(&rules);
    let before = state.clone();
    let made = cast(&mut state, &rules, scuttle(0, p(4, 0)), &mut draws(true)).unwrap();
    assert_eq!(made, (Err(CastFailure::Chance), vec![]));
    assert_eq!(state, before);
    let made = cast(&mut state, &rules, scuttle(3, p(4, 0)), &mut draws(true)).unwrap();
    let sunk = AdventureEvent {
        kind: AdventureEventKind::BoatScuttled {
            tile: p(4, 0),
            boat: 0,
        },
        seen_by: Viewers::NONE.with(RED),
    };
    assert_eq!(made, (Ok(()), vec![sunk]));
    let boat = state.boat(0).unwrap();
    assert!(boat.gone && !boat.aboard && !state.is_empty_boat(p(4, 0)));
    assert!(state.check_boats(&rules.1));
    let observed = state.observe(&rules.0, &rules.1, RED);
    assert!(observed.boats.iter().all(|b| b.tile != p(4, 0)));
    // A sunk boat is no target.
    assert_eq!(
        cast(&mut state, &rules, scuttle(3, p(4, 0)), &mut draws(true)),
        Err(CastError::NoBoat)
    );
}

// covers: ADV-SPL-004
#[test]
fn a_hidden_target_reads_the_truth_under_adv_quirk_52_or_is_revealed_without_it() {
    // E15-10 (faithful): the hidden empty boat on (8, 0) a target, sunk, nothing revealed, told
    // to nobody (nobody saw it). Off (strict-fixed): every hidden tile of the window a target;
    // the tile alone revealed in every outcome; one without an empty boat fails, charged
    // (`Target`), one with a boat takes the draw.
    let rules_on = rules(true, 64);
    let mut state = scuttle_world(&rules_on);
    let made = cast(&mut state, &rules_on, scuttle(3, p(8, 0)), &mut draws(true)).unwrap();
    let unseen = AdventureEvent {
        kind: AdventureEventKind::BoatScuttled {
            tile: p(8, 0),
            boat: 0,
        },
        seen_by: Viewers::NONE,
    };
    assert_eq!(made, (Ok(()), vec![unseen]));
    assert!(state.boat(1).unwrap().gone && !explored(&state, RED, p(8, 0)));

    let rules_off = rules(false, 64);
    let mut state = scuttle_world(&rules_off);
    let made = cast(
        &mut state,
        &rules_off,
        scuttle(3, p(9, 0)),
        &mut draws(true),
    )
    .unwrap();
    assert_eq!(made, (Err(CastFailure::Target), vec![]));
    assert!(explored(&state, RED, p(9, 0)) && !explored(&state, RED, p(8, 0)));
    let made = cast(
        &mut state,
        &rules_off,
        scuttle(0, p(8, 0)),
        &mut draws(true),
    )
    .unwrap();
    assert_eq!(made, (Err(CastFailure::Chance), vec![]));
    assert!(explored(&state, RED, p(8, 0)) && state.is_empty_boat(p(8, 0)));
    // Now seen, it is sunk as any boat the player sees.
    let made = cast(
        &mut state,
        &rules_off,
        scuttle(3, p(8, 0)),
        &mut draws(true),
    )
    .unwrap();
    assert_eq!(
        made.1[0].seen_by,
        Viewers::NONE.with(RED),
        "the revealed tile's viewers"
    );
    assert!(state.boat(1).unwrap().gone);
    // A hidden boat sunk at once is told to nobody: the caster sees the tile empty in the reveal.
    let mut state = scuttle_world(&rules_off);
    let made = cast(
        &mut state,
        &rules_off,
        scuttle(3, p(8, 0)),
        &mut draws(true),
    )
    .unwrap();
    assert_eq!(made, (Ok(()), vec![unseen]));
    assert!(explored(&state, RED, p(8, 0)) && state.check_boats(&rules_off.1));
}

#[test]
fn the_boat_spells_errors_explain_themselves() {
    let cases = [
        (CastError::NoWater, "no free water lies around the hero"),
        (
            CastError::OutOfWindow,
            "the target lies outside the spell's window",
        ),
        (CastError::NoBoat, "no empty boat lies on the target"),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}
