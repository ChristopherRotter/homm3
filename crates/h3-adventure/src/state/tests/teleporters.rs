//! Increment 11's teleporters on the map (E08 T103a, increment 11's design note §1): since E08
//! T104 (design note §5) the build's channels and gates' pairs, and its refusal of a teleporter off
//! its surface (A-42); since E08 T105a1 (design note §7, §10) the monoliths' and the Subterranean
//! Gates' jumps (`jumps.rs`); since T105a2 (design note §7 item 5, §8) the Whirlpool's loss and
//! jump.

use std::string::String;

use h3_content::{ArtifactClass, ArtifactDef, ArtifactSlotDef, HeroEffect};
use h3_heroes::wearing;

use super::*;
use crate::ruleset::{MonolithWay, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{
    GNOLL, PEASANT, PIKEMAN, WOLF, hero_at, hero_details, key, movement_rules_with, started,
    stream, world,
};

pub(super) type Rules = (Ruleset, AdventureRuleset);

pub(super) const RED: PlayerId = PlayerId::new(0);
pub(super) const HERO: HeroId = HeroId::new(1);

/// [`movement_rules_with`] (faithful) with a two-way monolith `t:monolith`, a Subterranean Gate
/// `t:underground_gate` (the test rules hold a border gate `t:gate`) and a Whirlpool
/// `t:whirlpool`, all entered, the Whirlpool from above; since E08 T104 a one-way entrance
/// `t:entrance` and exit `t:exit` of channel 0, an exit `t:exit_1` and a two-way monolith
/// `t:two_way_1` of channel 1; since T105a2 a Whirlpool `t:whirlpool_3` of divisor 3, the
/// creatures' fight values (the Pikeman the Pegasus's 407, the Gnoll the Dendroid Guard's 690, the
/// Peasant 15, the Wolf the Azure Dragon's 56 315: batch E11's E11-10 and E11-12) and a hat
/// `t:hat` worn on the head, which spares its wearer a Whirlpool's loss.
pub(super) fn teleporter_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |core| {
            for (creature, value) in core.creatures.iter_mut().zip([407, 690, 15, 56_315]) {
                creature.info.fight_value = value;
            }
            core.artifact_slots = vec![ArtifactSlotDef {
                key: key("t:head"),
                name: "Head".into(),
                count: 1,
                backpack: true,
            }];
            core.artifacts = vec![ArtifactDef {
                key: key("t:hat"),
                name: "Hat".into(),
                class: ArtifactClass::Treasure,
                cost: vec![],
                slot: Some(key("t:head")),
                war_machine: None,
                needs_skill: None,
                effects: vec![HeroEffect::WhirlpoolProtection],
                parts: vec![],
            }];
        },
        |part| {
            let ty = |k: &str, from_above, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Enter,
                from_above,
                kind,
            };
            part.object_types.extend([
                ty(
                    "t:monolith",
                    false,
                    ObjectKindDef::Monolith {
                        channel: 0,
                        way: MonolithWay::TwoWay,
                    },
                ),
                ty("t:underground_gate", false, ObjectKindDef::SubterraneanGate),
                ty("t:whirlpool", true, ObjectKindDef::Whirlpool { divisor: 2 }),
                ty(
                    "t:whirlpool_3",
                    true,
                    ObjectKindDef::Whirlpool { divisor: 3 },
                ),
            ]);
            let monolith =
                |k: &str, channel, way| ty(k, false, ObjectKindDef::Monolith { channel, way });
            part.object_types.extend([
                monolith("t:entrance", 0, MonolithWay::Entrance),
                monolith("t:exit", 0, MonolithWay::Exit),
                monolith("t:exit_1", 1, MonolithWay::Exit),
                monolith("t:two_way_1", 1, MonolithWay::TwoWay),
            ]);
        },
    )
}

pub(super) fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// An object of type `k` (none: an untyped blocker) anchored on `anchor`, blocking and visited
/// from the tiles of `visitable`.
fn object(rules: &Rules, k: Option<&str>, anchor: Pos, visitable: u64) -> ObjectSetup {
    ObjectSetup {
        scenery: Scenery::default(),
        anchor,
        blocked: TileMask::from_bits(visitable),
        visitable: TileMask::from_bits(visitable),
        object_type: k.map(|k| rules.1.object_type_by_key(k).unwrap()),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// The static map `setup` builds to under `rules`.
fn built(rules: &Rules, setup: &AdventureSetup) -> Arc<StaticMap> {
    Arc::clone(&started(rules, setup).unwrap().map)
}

pub(super) fn at(x: u16, y: u16, level: u8) -> Pos {
    Pos::new(x, y, level)
}

// covers: ADV-TEL-002
#[test]
fn the_channels_list_each_ways_tiles_in_the_walks_order() {
    let rules = teleporter_rules();
    let mut setup = world(&["gggggg"; 8], 2, vec![]);
    let o = |k, anchor, visitable| object(&rules, k, anchor, visitable);
    setup.map.objects.extend([
        // Channel 0's exits in map order: the walk's order is level, row, column, so (5, 0) comes
        // before (4, 3) on the surface, and the underground's after both (`Pos`' order, column
        // first, would take the underground's (1, 0) first).
        o(Some("t:exit"), at(4, 3, 0), 1),
        o(Some("t:exit"), at(1, 0, 1), 1),
        o(Some("t:exit"), at(5, 0, 0), 1),
        // An entrance is in no list.
        o(Some("t:entrance"), at(0, 0, 0), 1),
        // Channel 0's two-way monoliths: (4, 1) before (1, 2).
        o(Some("t:monolith"), at(1, 2, 0), 1),
        o(Some("t:monolith"), at(4, 1, 0), 1),
        // Channel 1's: a one-way and a two-way channel of one number are distinct.
        o(Some("t:two_way_1"), at(3, 3, 1), 1),
        o(Some("t:exit_1"), at(2, 1, 1), 1),
        // An exit under a blocker visited from its tile, placed after it: the tile is the
        // blocker's (the original's tile class, the last object placed), in no list.
        o(Some("t:exit"), at(3, 1, 0), 1),
        o(None, at(3, 1, 0), 1),
        // An exit visited from a tile off the map alone (one column left of the first): none.
        o(Some("t:exit"), at(0, 2, 0), 2),
    ]);
    let map = built(&rules, &setup);
    let channels = map.channels();
    assert!(!channels.is_empty());
    assert_eq!(channels.exits(0), [at(5, 0, 0), at(4, 3, 0), at(1, 0, 1)]);
    assert_eq!(channels.two_way(0), [at(4, 1, 0), at(1, 2, 0)]);
    assert_eq!(channels.exits(1), [at(2, 1, 1)]);
    assert_eq!(channels.two_way(1), [at(3, 3, 1)]);
    assert!(channels.exits(2).is_empty() && channels.two_way(2).is_empty());
    let one_way: Vec<_> = channels.one_way_channels().collect();
    assert_eq!(
        one_way,
        [
            (0, &[at(5, 0, 0), at(4, 3, 0), at(1, 0, 1)][..]),
            (1, &[at(2, 1, 1)][..])
        ]
    );
    let two_way: Vec<_> = channels.two_way_channels().collect();
    assert_eq!(
        two_way,
        [
            (0, &[at(4, 1, 0), at(1, 2, 0)][..]),
            (1, &[at(3, 3, 1)][..])
        ]
    );
    assert!(channels.whirlpools().is_empty() && channels.gates().is_empty());
    // Built without a draw: the setup's stream stands where a map without them leaves it.
    let mut draws = draws();
    AdventureState::new(&rules.0, &rules.1, &setup, &mut draws).unwrap();
    let mut bare = crate::testing::draws();
    AdventureState::new(
        &rules.0,
        &rules.1,
        &world(&["gggggg"; 8], 2, vec![]),
        &mut bare,
    )
    .unwrap();
    assert_eq!(draws, bare);
}

// covers: ADV-TEL-002
#[test]
fn the_whirlpools_list_every_tile_with_its_whirlpool() {
    let rules = teleporter_rules();
    let mut setup = world(&["~~~~~~~~~~", "~~~~~~~~~~", "gggggggggg"], 1, vec![]);
    // Two Whirlpools of 3 × 2 tiles side by side, the second first in map order: the walk takes
    // row 0 of both, then row 1.
    let six = 0x0707;
    setup.map.objects.extend([
        object(&rules, Some("t:whirlpool"), at(8, 1, 0), six),
        object(&rules, Some("t:whirlpool"), at(3, 1, 0), six),
    ]);
    let map = built(&rules, &setup);
    let tiles: Vec<(Pos, usize)> = map
        .channels()
        .whirlpools()
        .iter()
        .map(|w| (w.tile, w.object))
        .collect();
    let row = |y, from: u16, object| (from..from + 3).map(move |x| (at(x, y, 0), object));
    let expected: Vec<_> = row(0, 1, 1)
        .chain(row(0, 6, 0))
        .chain(row(1, 1, 1))
        .chain(row(1, 6, 0))
        .collect();
    assert_eq!(tiles, expected);
}

// covers: ADV-TEL-002
#[test]
fn the_gates_pair_greedily_in_the_walks_order_by_the_truncated_distance() {
    let rules = teleporter_rules();
    let mut setup = world(&["gggggggggggggggg"; 8], 2, vec![]);
    let gate = |anchor| object(&rules, Some("t:underground_gate"), anchor, 1);
    // In map order, which the walk reorders: on the surface A (0, 0), B (4, 0), E (14, 3);
    // underground C (3, 0), D (10, 0), F (12, 2), G (12, 3).
    let [a, b, e] = [at(0, 0, 0), at(4, 0, 0), at(14, 3, 0)];
    let [c, d, f, g] = [at(3, 0, 1), at(10, 0, 1), at(12, 2, 1), at(12, 3, 1)];
    setup.map.objects.extend([g, e, d, a, c, f, b].map(gate));
    let map = built(&rules, &setup);
    let channels = map.channels();
    let pairs: Vec<(Pos, Option<Pos>)> =
        channels.gates().iter().map(|g| (g.tile, g.pair)).collect();
    assert_eq!(
        pairs,
        [
            // A's nearest is C (3); B's would be C too (1), but A took it: D (6), the greedy
            // order the original pairs in (survey §1: 18 shipped gates not with their nearest).
            (a, Some(c)),
            (b, Some(d)),
            // E is ⌊√5⌋ = 2 from F and ⌊√4⌋ = 2 from G: a tie, so F, the first in the walk's
            // order, though G is nearer (VCMI's and a squared distance's pick).
            (e, Some(f)),
            (c, Some(a)),
            (d, Some(b)),
            (f, Some(e)),
            // G is left over: no gate after it on another level.
            (g, None),
        ]
    );
    assert_eq!(channels.gate_pair(b), Some(d));
    assert_eq!(channels.gate_pair(f), Some(e));
    assert_eq!(channels.gate_pair(g), None);
    assert_eq!(channels.gate_pair(at(1, 0, 0)), None);
    // Gates of one level never pair.
    let mut setup = world(&["gggggggggggggggg"; 8], 2, vec![]);
    setup.map.objects.extend([a, b, e].map(gate));
    let map = built(&rules, &setup);
    assert!(map.channels().gates().iter().all(|g| g.pair.is_none()));
}

// covers: ADV-TEL-001, ADV-TEL-002
#[test]
fn a_teleporter_off_its_surface_is_refused_when_the_game_is_built() {
    let rules = teleporter_rules();
    // Grass, sea and rock.
    let rows = ["gg~~##"];
    let o = |k, x, visitable| object(&rules, Some(k), at(x, 0, 0), visitable);
    // On its surface (a tile off the map ignored: the second mask's left tile).
    let fine = [o("t:monolith", 0, 3), o("t:whirlpool", 2, 1)];
    let mut setup = world(&rows, 1, vec![]);
    setup.map.objects.extend(fine.clone());
    assert!(started(&rules, &setup).is_ok());
    for (k, x) in [
        ("t:monolith", 2),
        ("t:entrance", 4),
        ("t:underground_gate", 3),
        ("t:whirlpool", 1),
        ("t:whirlpool", 5),
    ] {
        let mut setup = world(&rows, 1, vec![]);
        setup.map.objects.extend(fine.clone());
        // Visited from its anchor and the tile left of it, one of them off its surface.
        setup.map.objects.push(o(k, x, 3));
        assert_eq!(
            started(&rules, &setup).err(),
            Some(SetupError::TeleporterSurface(2)),
            "{k} on {x}"
        );
    }
    assert_eq!(
        SetupError::TeleporterSurface(2).to_string(),
        "teleporter 2 has a visitable tile off its surface"
    );
    // Without teleporters the channels are empty.
    let map = built(&rules, &world(&rows, 1, vec![]));
    assert!(map.channels().is_empty());
    assert_eq!(*map.channels(), Channels::default());
}

// The jumps (E08 T105a1; ADV-TEL-003 to -005, -007; design note §7, §10).

pub(super) const BLUE: PlayerId = PlayerId::new(1);

/// The width of the jumps' maps: wide enough that an exit's surroundings lie outside the start's
/// reveal (the scouting radius 5) and the fringe.
const WIDE: usize = 30;

/// The started state of a `levels`-level map of [`WIDE`] × 3 grass tiles per level (`rows`
/// edits a row of the surface: a monster `M`), with Red's hero [`HERO`] (Pikemen) on (0, 1),
/// `others` besides and `objects`, each of a type by key on its anchor, visited from it.
pub(super) fn jump_state(
    rules: &Rules,
    (levels, monster): (u8, Option<Pos>),
    others: &[(u16, u8, Pos)],
    objects: &[(&str, Pos)],
) -> AdventureState {
    let mut rows = vec!["g".repeat(WIDE); 3 * usize::from(levels)];
    if let Some(at) = monster {
        let row = &mut rows[usize::from(at.level) * 3 + usize::from(at.y)];
        row.replace_range(usize::from(at.x)..=usize::from(at.x), "M");
    }
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut heroes = vec![hero_at(1, 0, p(0, 1), &[PIKEMAN])];
    heroes.extend(
        others
            .iter()
            .map(|&(id, owner, pos)| hero_at(id, owner, pos, &[PIKEMAN])),
    );
    let mut setup = world(&rows, levels, heroes);
    for &(k, anchor) in objects {
        setup.map.objects.push(object(rules, Some(k), anchor, 1));
    }
    started(rules, &setup).unwrap()
}

/// Red's hero [`HERO`] walks `path` with `draws` as its stream: the outcome and the events, or
/// the error (a rejected move changes neither the state nor the stream and emits nothing).
pub(super) fn walk(
    state: &mut AdventureState,
    rules: &Rules,
    path: &[Pos],
    draws: &mut Rng,
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let (before, stream) = (state.clone(), draws.clone());
    let heroes = [Arc::new(hero_details(&[]))];
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let mut events = Vec::new();
    let result = state.apply(
        &rules.0,
        &rules.1,
        &heroes,
        (RED, draws),
        &command,
        &mut events,
    );
    if result.is_err() {
        assert_eq!((&*state, &*draws), (&before, &stream));
        assert!(events.is_empty());
    }
    result.map(|outcome| (outcome, events))
}

/// The walk onto the entrance on (2, 1).
pub(super) const ONTO: [Pos; 2] = [Pos::new(1, 1, 0), Pos::new(2, 1, 0)];

/// The jumps among `events`: each one's entrance and exit.
pub(super) fn jumps(events: &[AdventureEvent]) -> Vec<(Option<Pos>, Option<Pos>)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            AdventureEventKind::HeroTeleported { from, to, .. } => Some((from, to)),
            _ => None,
        })
        .collect()
}

/// That `outcome` is Red's hero's battle against `defender` at a jump's `exit`, with the jump
/// held after it and nothing else (GAM-BAT-004, GAM-DEC-007 amended; E08 T105b1, increment 11's
/// design note §12): fought at the exit, the defender's tile.
fn fought_at(outcome: &AdventureOutcome, exit: Pos, defender: HeroId) -> &BattleStart {
    let battle = outcome.battle.as_ref().unwrap();
    assert_eq!(battle.attacker.hero, HERO);
    assert!(matches!(&battle.defender, Defender::Hero(c) if c.hero == defender));
    assert_eq!((battle.tile, battle.attacker_ground.as_ref()), (exit, None));
    let alone = AdventureOutcome {
        battle: Some(battle.clone()),
        jump_after: Some((HERO, exit)),
        ..AdventureOutcome::default()
    };
    assert_eq!(outcome, &alone);
    battle
}

/// Red's hero's tile, points and day's maximum.
pub(super) fn red(state: &AdventureState) -> (Pos, u32, u32) {
    let hero = state.hero(HERO).unwrap();
    (hero.pos, hero.movement, hero.max_movement)
}

// covers: ADV-TEL-004, ADV-TEL-007, ADV-TEL-003, ADV-OBJ-003
#[test]
fn a_one_way_entrance_jumps_to_its_one_exit_without_a_draw_keeping_the_points() {
    let rules = teleporter_rules();
    let (entrance, exit) = (p(2, 1), p(20, 1));
    let mut state = jump_state(
        &rules,
        (1, None),
        &[],
        &[("t:entrance", entrance), ("t:exit", exit)],
    );
    // The entrance is a destination with the points left after the step (D-58): no path goes
    // through it, a step past it is refused.
    let heroes = [Arc::new(hero_details(&[]))];
    let reach = state.reach(&rules.0, &rules.1, &heroes, HERO).unwrap();
    assert_eq!(reach.path(entrance), Some(ONTO.to_vec()));
    let left = reach.points_left(entrance).unwrap();
    let past = [ONTO[0], ONTO[1], p(3, 1)];
    assert_eq!(
        walk(&mut state, &rules, &past, &mut stream()).err(),
        Some(AdventureCommandError::PastVisit { step: 2 })
    );
    assert!(!state.is_explored(RED, exit));
    let (_, max) = (red(&state).1, red(&state).2);
    let mut draws = stream();
    let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
    // On the exit with the step's points and the day's maximum, the stream untouched (one
    // candidate, no draw), the move over: nothing opened.
    assert_eq!(red(&state), (exit, left, max));
    assert_eq!(draws, stream());
    assert_eq!(outcome, AdventureOutcome::default());
    // The two steps, then the jump, seen by Red alone (Blue's hero is nowhere: none on this map).
    let red_only = Viewers::NONE.with(RED);
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[2],
        AdventureEvent {
            kind: AdventureEventKind::HeroTeleported {
                hero: HERO,
                from: Some(entrance),
                to: Some(exit),
                seen_from: red_only,
                seen_to: red_only,
            },
            seen_by: red_only,
        }
    );
    // The exit's scouting radius explored, no farther.
    for (x, explored) in [(14, false), (15, true), (25, true), (26, false)] {
        assert_eq!(state.is_explored(RED, p(x, 1)), explored, "{x}");
    }
    assert!(state.is_explored(RED, p(20, 2)) && state.is_explored(RED, p(20, 0)));
}

// covers: ADV-TEL-003, ADV-TEL-004
#[test]
fn a_monoliths_exit_is_drawn_among_its_candidates_from_the_visitors_stream() {
    let rules = teleporter_rules();
    // Channel 0's exits in the walk's order: (12, 0) free, (16, 0) under Blue's hero 2 (a
    // candidate), (20, 0) free, (27, 2) under Red's own hero 3 (no candidate).
    let exits = [p(12, 0), p(16, 0), p(20, 0), p(27, 2)];
    let candidates = [exits[0], exits[1], exits[2]];
    let mut objects = vec![("t:entrance", p(2, 1))];
    objects.extend(exits.map(|e| ("t:exit", e)));
    let others = [(2, 1, exits[1]), (3, 0, exits[3])];
    let mut seen = [false; 3];
    for seed in 0..24 {
        let mut state = jump_state(&rules, (1, None), &others, &objects);
        let mut draws = Rng::for_stream(seed, StreamId::player(0));
        let mut expected = draws.clone();
        let place = usize::try_from(expected.below(3)).unwrap();
        seen[place] = true;
        let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
        // One draw among the three candidates, never Red's own hero's exit.
        assert_eq!(draws, expected, "seed {seed}");
        let drawn = candidates[place];
        if drawn == exits[1] {
            // Blue's hero on the exit: the tiles within 1 of it explored by Red, the hero still
            // on the entrance, the battle at the exit with the jump held after it (E08 T105b1).
            assert_eq!(red(&state).0, p(2, 1));
            assert!(jumps(&events).is_empty());
            for (x, y, explored) in [(15, 0, true), (17, 1, true), (14, 0, false), (16, 2, false)] {
                assert_eq!(state.is_explored(RED, p(x, y)), explored, "{x}, {y}");
            }
            let battle = fought_at(&outcome, drawn, HeroId::new(2));
            assert_eq!(battle.battlefield, None);
        } else {
            assert_eq!(outcome, AdventureOutcome::default());
            assert_eq!(red(&state).0, drawn);
            assert_eq!(jumps(&events), [(Some(p(2, 1)), Some(drawn))]);
        }
    }
    assert_eq!(seen, [true; 3]);
}

// covers: ADV-TEL-003, ADV-TEL-004
#[test]
fn a_monolith_without_a_candidate_does_nothing() {
    let rules = teleporter_rules();
    // A one-way entrance whose one exit holds Red's own hero, and a two-way monolith alone in its
    // channel: no candidate, no draw, no event but the steps.
    let cases = [
        vec![("t:entrance", p(2, 1)), ("t:exit", p(20, 1))],
        vec![("t:monolith", p(2, 1))],
    ];
    for objects in cases {
        let mut state = jump_state(&rules, (1, None), &[(3, 0, p(20, 1))], &objects);
        let mut draws = stream();
        let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
        assert_eq!(red(&state).0, p(2, 1));
        assert_eq!((outcome, events.len()), (AdventureOutcome::default(), 2));
        assert_eq!(draws, stream());
    }
}

// covers: ADV-TEL-003, ADV-TEL-004
#[test]
fn a_two_way_monolith_leads_to_another_of_its_channel_never_itself() {
    let rules = teleporter_rules();
    let members = [p(2, 1), p(12, 1), p(22, 1)];
    let objects = members.map(|m| ("t:monolith", m));
    let mut seen = [false; 2];
    for seed in 0..16 {
        let mut state = jump_state(&rules, (1, None), &[], &objects);
        let mut draws = Rng::for_stream(seed, StreamId::player(0));
        let mut expected = draws.clone();
        let place = usize::try_from(expected.below(2)).unwrap();
        seen[place] = true;
        let (_, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
        assert_eq!(draws, expected);
        assert_eq!(red(&state).0, members[1 + place]);
        assert_eq!(
            jumps(&events),
            [(Some(members[0]), Some(members[1 + place]))]
        );
    }
    assert_eq!(seen, [true; 2]);
}

// covers: ADV-TEL-004, ADV-TEL-005
#[test]
fn a_one_way_exit_and_a_gate_without_a_pair_lead_nowhere() {
    let rules = teleporter_rules();
    for k in ["t:exit", "t:underground_gate"] {
        let mut state = jump_state(&rules, (2, None), &[], &[(k, p(2, 1))]);
        let mut draws = stream();
        let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
        assert_eq!(red(&state).0, p(2, 1), "{k}");
        assert_eq!(outcome, AdventureOutcome::default());
        assert_eq!(draws, stream());
        assert_eq!(
            events.last(),
            Some(&AdventureEvent {
                kind: AdventureEventKind::LeadsNowhere {
                    hero: HERO,
                    object_type: rules.1.object_type_by_key(k).unwrap(),
                    anchor: p(2, 1),
                },
                seen_by: Viewers::NONE.with(RED),
            }),
            "{k}"
        );
        assert_eq!(events.len(), 3);
    }
}

// covers: ADV-TEL-005, ADV-TEL-007
#[test]
fn a_gate_leads_to_its_pair_without_a_draw() {
    let rules = teleporter_rules();
    let (gate, pair) = (p(2, 1), at(20, 1, 1));
    let gates = [("t:underground_gate", gate), ("t:underground_gate", pair)];
    let mut state = jump_state(&rules, (2, None), &[], &gates);
    let mut draws = stream();
    let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
    assert_eq!(red(&state).0, pair);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(draws, stream());
    assert_eq!(jumps(&events), [(Some(gate), Some(pair))]);
    assert!(state.is_explored(RED, at(25, 1, 1)) && !state.is_explored(RED, at(14, 1, 1)));
    // A hero on the pair, of the team or another: the tiles within 1 of the pair explored by Red,
    // the mover on the gate (Red's own hero there explored its own surroundings at the start);
    // one of the team is met (GAM-MET-001, the two heroes no neighbours), an enemy fought at the
    // pair with the jump held after it (E08 T105b1).
    for owner in [0, 1] {
        let mut state = jump_state(&rules, (2, None), &[(2, owner, pair)], &gates);
        let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut draws).unwrap();
        assert_eq!(red(&state).0, gate);
        assert!(jumps(&events).is_empty());
        assert!(state.is_explored(RED, at(21, 2, 1)));
        assert_eq!(state.is_explored(RED, at(22, 1, 1)), owner == 0);
        if owner == 0 {
            let meeting = Meeting {
                visitor: HERO,
                visited: HeroId::new(2),
            };
            let met = AdventureOutcome {
                meeting: Some(meeting),
                ..AdventureOutcome::default()
            };
            assert_eq!(outcome, met);
        } else {
            fought_at(&outcome, pair, HeroId::new(2));
        }
    }
    assert_eq!(draws, stream());
}

// covers: ADV-TEL-007
#[test]
fn the_arrival_meets_no_guard_and_visits_nothing() {
    let rules = teleporter_rules();
    // The exit is a one-way exit (a visit would lead nowhere) beside an aggressive monster.
    let monster = p(21, 1);
    let objects = [("t:entrance", p(2, 1)), ("t:exit", p(20, 1))];
    let mut state = jump_state(&rules, (1, Some(monster)), &[], &objects);
    let (outcome, events) = walk(&mut state, &rules, &ONTO, &mut stream()).unwrap();
    assert_eq!(red(&state).0, p(20, 1));
    assert_eq!(outcome, AdventureOutcome::default());
    assert!(matches!(
        events.last().map(|e| e.kind),
        Some(AdventureEventKind::HeroTeleported { .. })
    ));
}

// covers: GAM-DEC-007, ADV-TEL-007
#[test]
fn the_held_jump_arrives_on_a_free_exit_alone() {
    let rules = teleporter_rules();
    let store = [Arc::new(hero_details(&[]))];
    let context = (&rules.0, &rules.1, &store[..]);
    let (entrance, exit) = (p(2, 1), p(20, 1));
    let objects = [("t:entrance", entrance), ("t:exit", exit)];
    // Red's hero on the entrance, as after its battle at the exit; Blue's hero 2 on (5, 0).
    let on_entrance = || {
        let mut state = jump_state(&rules, (1, None), &[(2, 1, p(5, 0))], &objects);
        let at = state.heroes.partition_point(|h| h.id < HERO);
        state.heroes[at].pos = entrance;
        state
    };
    let untouched = |mut state: AdventureState, player| {
        let (before, mut events) = (state.clone(), Vec::new());
        state.arrive(context, (player, HERO), exit, &mut events);
        assert_eq!((state, events), (before, Vec::new()));
    };
    // Nothing for another player's hero, a mover the battle took off the map, or a hero on the
    // exit.
    untouched(on_entrance(), PlayerId::new(1));
    let mut gone = on_entrance();
    gone.end_battle(&rules.0, &[(HERO, None)], &mut Vec::new());
    untouched(gone, RED);
    let mut held = on_entrance();
    let at = held.heroes.partition_point(|h| h.id < HeroId::new(2));
    held.heroes[at].pos = exit;
    untouched(held, RED);
    // A free exit: the arrival from the entrance, with its event and the points kept.
    let mut state = on_entrance();
    let (_, points, max) = red(&state);
    let mut events = Vec::new();
    state.arrive(context, (RED, HERO), exit, &mut events);
    assert_eq!(red(&state), (exit, points, max));
    assert_eq!(jumps(&events), [(Some(entrance), Some(exit))]);
}

// covers: GAM-DEC-007, ADV-TEL-006, ADV-TEL-007, ADV-SEA-006
#[test]
fn the_held_jump_lands_in_the_boat_once_the_defender_left_with_its_own() {
    let rules = teleporter_rules();
    let store = [Arc::new(hero_details(&[]))];
    let context = (&rules.0, &rules.1, &store[..]);
    // The entrance's Whirlpool and another of one tile, (20, 1), where Blue's hero 2 stands in
    // boat 2: the one candidate, so no draw (A-37 fixed by the battle, decision 265).
    let exit = p(20, 1);
    let whirlpools = [("t:whirlpool", p(3, 1), 0x0303), ("t:whirlpool", exit, 1)];
    let mut state = whirlpool_state(&rules, vec![stack(0, PIKEMAN, 4)], &whirlpools);
    let blue = HeroId::new(2);
    state.put_aboard(blue, 2);
    let at = state.heroes.partition_point(|h| h.id < blue);
    state.heroes[at].pos = exit;
    let (outcome, events) = whirl(&mut state, &rules, &store, &mut stream());
    assert!(jumps(&events).is_empty());
    fought_at(&outcome, exit, blue);
    // A tile with an empty boat (boat 1 on (12, 1)) takes no arrival.
    let before = state.clone();
    let mut events = Vec::new();
    state.arrive(context, (RED, HERO), p(12, 1), &mut events);
    assert_eq!((&state, events.len()), (&before, 0));
    // Won: Blue's hero leaves the map with its boat, and the held jump lands Red's in its own,
    // whose record keeps the tile it was boarded on.
    state.end_battle(&rules.0, &[(blue, None)], &mut Vec::new());
    state.arrive(context, (RED, HERO), exit, &mut events);
    assert_eq!(red(&state).0, exit);
    assert_eq!(state.hero(HERO).unwrap().boat, Some(0));
    assert_eq!(state.boat(0).unwrap().tile, p(1, 1));
    assert_eq!(jumps(&events), [(Some(INTO[0]), Some(exit))]);
    assert!(state.check_boats(&rules.1));
}

// covers: ADV-MON-006, ADV-TEL-004
#[test]
fn a_guarded_entrance_meets_its_guard_first_then_the_jump_follows() {
    let rules = teleporter_rules();
    // The monster on (3, 2) guards the entrance on (2, 1) (its row below and its own row: a
    // monolith is not visited from above).
    let objects = [("t:entrance", p(2, 1)), ("t:exit", p(20, 1))];
    let mut state = jump_state(&rules, (1, Some(p(3, 2))), &[], &objects);
    let (outcome, _) = walk(&mut state, &rules, &ONTO, &mut stream()).unwrap();
    assert_eq!(outcome.visit_after, Some(HERO));
    assert_eq!(red(&state).0, p(2, 1));
    // Once the guard's visit is over, the held visit is the jump, from the visitor's stream.
    let heroes = [Arc::new(hero_details(&[]))];
    let mut events = Vec::new();
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    let held = state.visit_held(rules3, (RED, HERO), &mut stream(), &mut events);
    assert_eq!(held, AdventureOutcome::default());
    assert_eq!(red(&state).0, p(20, 1));
    assert_eq!(jumps(&events), [(Some(p(2, 1)), Some(p(20, 1)))]);
}

// covers: ADV-FOG-004, ADV-TEL-007
#[test]
fn a_viewer_is_told_the_ends_of_a_jump_it_sees_never_a_hidden_one() {
    let rules = teleporter_rules();
    let (entrance, exit) = (p(2, 1), p(20, 1));
    let objects = [("t:entrance", entrance), ("t:exit", exit)];
    // Blue's hero near the entrance sees it there only, near the exit only there; far from both
    // (beyond the fringe of either), not at all.
    for (blue, from, to) in [
        (p(4, 0), Some(entrance), None),
        (p(23, 0), None, Some(exit)),
        (p(11, 2), None, None),
    ] {
        let mut state = jump_state(&rules, (1, None), &[(2, 1, blue)], &objects);
        let (_, events) = walk(&mut state, &rules, &ONTO, &mut stream()).unwrap();
        let jump = events.last().unwrap();
        assert_eq!(
            jumps(core::slice::from_ref(jump)),
            [(Some(entrance), Some(exit))]
        );
        let told = jump.project(BLUE);
        let expected = (from.is_some() || to.is_some()).then(|| {
            let alone = Viewers::NONE.with(BLUE);
            AdventureEvent {
                kind: AdventureEventKind::HeroTeleported {
                    hero: HERO,
                    from,
                    to,
                    seen_from: if from.is_some() { alone } else { Viewers::NONE },
                    seen_to: if to.is_some() { alone } else { Viewers::NONE },
                },
                seen_by: alone,
            }
        });
        assert_eq!(told, expected, "{blue:?}");
    }
}

/// A stack of `count` of `creature` in `slot`.
pub(super) const fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The step from Red's boat on (1, 1) onto the Whirlpool's tile (2, 1).
pub(super) const INTO: [Pos; 1] = [Pos::new(2, 1, 0)];

/// The started state of a sea of [`WIDE`] × 2 tiles above a row of grass, with `whirlpools` (each
/// of a type by key, its anchor and its tiles' mask; the first's tiles (2, 0) to (3, 1)) and Red's
/// hero [`HERO`] with `army` aboard boat 0 on (1, 1), 100 points to the step; empty boats 1 to 3
/// on (12, 1) to (14, 1); Blue's hero 2 and Red's hero 3, each with a Pikeman, on the grass at
/// (13, 2) and (12, 2).
pub(super) fn whirlpool_state(
    rules: &Rules,
    army: Vec<ArmyStack>,
    whirlpools: &[(&str, Pos, u64)],
) -> AdventureState {
    let mut boats = "~".repeat(WIDE);
    boats.replace_range(1..2, "b");
    boats.replace_range(12..15, "bbb");
    let rows = ["~".repeat(WIDE), boats, "g".repeat(WIDE)];
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let heroes = vec![
        HeroSetup {
            army,
            ..hero_at(1, 0, p(0, 2), &[])
        },
        hero_at(2, 1, p(13, 2), &[PIKEMAN]),
        hero_at(3, 0, p(12, 2), &[PIKEMAN]),
    ];
    let mut setup = world(&rows, 1, heroes);
    for &(k, anchor, tiles) in whirlpools {
        setup
            .map
            .objects
            .push(object(rules, Some(k), anchor, tiles));
    }
    let mut state = started(rules, &setup).unwrap();
    state.put_aboard(HERO, 0);
    state
}

/// Red's hero [`HERO`] steps into the Whirlpool with `store` and `draws`: the outcome and the
/// events (the boats keep their invariants).
pub(super) fn whirl(
    state: &mut AdventureState,
    rules: &Rules,
    store: &[Arc<HeroDetails>],
    draws: &mut Rng,
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: INTO.to_vec(),
    };
    let mut events = Vec::new();
    let outcome = state
        .apply(
            &rules.0,
            &rules.1,
            store,
            (RED, draws),
            &command,
            &mut events,
        )
        .unwrap();
    assert!(state.check_boats(&rules.1));
    (outcome, events)
}

/// The Whirlpools' losses among `events`, as `(hero, slot, creature, count)`.
pub(super) fn losses(events: &[AdventureEvent]) -> Vec<(HeroId, u8, CreatureTypeId, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            AdventureEventKind::WhirlpoolLoss {
                hero,
                slot,
                creature,
                count,
            } => {
                assert_eq!(e.seen_by, Viewers::NONE.with(RED));
                Some((hero, slot, creature, count))
            }
            _ => None,
        })
        .collect()
}

// covers: ADV-TEL-006, ADV-OBJ-003
#[test]
fn a_whirlpool_takes_the_weakest_stack_by_fight_value_times_count_first_slot_on_a_tie() {
    let rules = teleporter_rules();
    let alone = |k| [(k, p(3, 1), 0x0303)];
    // Each army in the Whirlpool alone (no exit: the hero stays, the loss taken, no draw), with
    // its army after and its loss (slot, creature, count lost).
    let cases = [
        // E11-10: 3 Pegasi (1 221) below 3 Dendroids (2 070): 3 → 1.
        (
            "t:whirlpool",
            vec![stack(0, PIKEMAN, 3), stack(1, GNOLL, 3), stack(2, WOLF, 2)],
            vec![stack(0, PIKEMAN, 1), stack(1, GNOLL, 3), stack(2, WOLF, 2)],
            (0, PIKEMAN, 2),
        ),
        // E11-11: a tie between slots 1 and 2, the first taken.
        (
            "t:whirlpool",
            vec![
                stack(0, WOLF, 1),
                stack(1, PEASANT, 5),
                stack(2, PEASANT, 5),
            ],
            vec![
                stack(0, WOLF, 1),
                stack(1, PEASANT, 2),
                stack(2, PEASANT, 5),
            ],
            (1, PEASANT, 3),
        ),
        // E11-12: a lone creature kept, told with 0 lost; a lone stack of 3 keeps 1.
        (
            "t:whirlpool",
            vec![stack(0, PIKEMAN, 1)],
            vec![stack(0, PIKEMAN, 1)],
            (0, PIKEMAN, 0),
        ),
        (
            "t:whirlpool",
            vec![stack(3, PIKEMAN, 3)],
            vec![stack(3, PIKEMAN, 1)],
            (3, PIKEMAN, 2),
        ),
        // A-38 not reproduced (ADV-QUIRK-40): a lone 1 776 Azure Dragons (100 015 440, past the
        // original's 99 999 999) halved, and of two such stacks the lesser, none corrupted.
        (
            "t:whirlpool",
            vec![stack(0, WOLF, 1776)],
            vec![stack(0, WOLF, 888)],
            (0, WOLF, 888),
        ),
        (
            "t:whirlpool",
            vec![stack(0, WOLF, 1800), stack(1, WOLF, 1776)],
            vec![stack(0, WOLF, 1800), stack(1, WOLF, 888)],
            (1, WOLF, 888),
        ),
        // Beside another stack a lone creature is lost, its slot emptied.
        (
            "t:whirlpool",
            vec![stack(0, WOLF, 2), stack(4, PEASANT, 1)],
            vec![stack(0, WOLF, 2)],
            (4, PEASANT, 1),
        ),
        // The divisor: 7 keep 7 / 3 = 2; a lone stack of 2 keeps 1 (2 / 3 is 0).
        (
            "t:whirlpool_3",
            vec![stack(0, PIKEMAN, 7), stack(1, WOLF, 1)],
            vec![stack(0, PIKEMAN, 2), stack(1, WOLF, 1)],
            (0, PIKEMAN, 5),
        ),
        (
            "t:whirlpool_3",
            vec![stack(0, PIKEMAN, 2)],
            vec![stack(0, PIKEMAN, 1)],
            (0, PIKEMAN, 1),
        ),
    ];
    let store = [Arc::new(hero_details(&[]))];
    for (k, army, after, (slot, creature, count)) in cases {
        let mut state = whirlpool_state(&rules, army, &alone(k));
        // The Whirlpool's tile is a destination of the boat's reach, never a tile on the way.
        let reach = state.reach(&rules.0, &rules.1, &store, HERO).unwrap();
        assert_eq!(reach.path(INTO[0]), Some(INTO.to_vec()));
        let past = [INTO[0], p(3, 1)];
        assert_eq!(
            walk(&mut state, &rules, &past, &mut stream()).err(),
            Some(AdventureCommandError::PastVisit { step: 1 })
        );
        let mut draws = stream();
        let (outcome, events) = whirl(&mut state, &rules, &store, &mut draws);
        assert_eq!(outcome, AdventureOutcome::default());
        let hero = state.hero(HERO).unwrap();
        assert_eq!((hero.pos, hero.boat), (INTO[0], Some(0)));
        assert_eq!(hero.army, after, "{k}");
        // The step, then the loss to Red alone; no jump, no draw.
        assert_eq!(events.len(), 2);
        assert_eq!(losses(&events), [(HERO, slot, creature, count)]);
        assert_eq!(draws, stream());
    }
}

// covers: ADV-TEL-006, HER-ADV-009
#[test]
fn a_whirlpool_spares_a_hero_wearing_the_hat() {
    let rules = teleporter_rules();
    let hat = rules.0.artifact_by_key("t:hat").unwrap();
    let hatted = HeroDetails {
        worn: wearing(&rules.0, &[hat.into()]),
        ..hero_details(&[])
    };
    let army = vec![stack(0, PIKEMAN, 10)];
    let bare = Arc::new(hero_details(&[]));
    // With the hat nothing is lost and nothing told (E11-13); without it the loss is taken.
    for (details, after, lost) in [(hatted, 10, vec![]), (hero_details(&[]), 5, vec![5])] {
        let store = [Arc::clone(&bare), Arc::new(details)];
        let mut state = whirlpool_state(&rules, army.clone(), &[("t:whirlpool", p(3, 1), 0x0303)]);
        let (_, events) = whirl(&mut state, &rules, &store, &mut stream());
        assert_eq!(state.hero(HERO).unwrap().army, [stack(0, PIKEMAN, after)]);
        let told: Vec<u32> = losses(&events).iter().map(|loss| loss.3).collect();
        assert_eq!(told, lost);
    }
}

// covers: ADV-TEL-006, ADV-TEL-003, ADV-TEL-007
#[test]
fn a_whirlpools_exit_is_drawn_among_the_other_whirlpools_tiles_the_hero_jumping_in_its_boat() {
    let rules = teleporter_rules();
    // The Whirlpools' list in the walk's order: W1's (2, 0), (3, 0), W2's (19, 0), (20, 0), then
    // W1's (2, 1) (the entrance) and (3, 1), W2's (19, 1), (20, 1) and W3's (27, 1). W1's own
    // tiles are excluded, (19, 0) holds an empty boat (a hero landed from it) and (19, 1) Red's
    // own hero 3 in its boat: the candidates are (20, 0) free, (20, 1) under Blue's hero 2 in its
    // boat and (27, 1) free.
    let whirlpools = [
        ("t:whirlpool", p(3, 1), 0x0303),
        ("t:whirlpool", p(20, 1), 0x0303),
        ("t:whirlpool", p(27, 1), 1),
    ];
    let candidates = [p(20, 0), p(20, 1), p(27, 1)];
    let store = [Arc::new(hero_details(&[]))];
    let mut seen = [false; 3];
    for seed in 0..24 {
        let mut state = whirlpool_state(&rules, vec![stack(0, PIKEMAN, 4)], &whirlpools);
        for (hero, boat, tile) in [(HeroId::new(3), 1, p(19, 1)), (HeroId::new(2), 2, p(20, 1))] {
            state.put_aboard(hero, boat);
            let at = state.heroes.partition_point(|h| h.id < hero);
            state.heroes[at].pos = tile;
        }
        let boats = state.boats.as_mut().unwrap();
        let mut landed = *boats.records.get(3).unwrap();
        boats.empty.set(landed.tile, false);
        landed.tile = p(19, 0);
        boats.empty.set(landed.tile, true);
        boats.records.set(3, Some(landed));
        assert!(state.check_boats(&rules.1));
        let (points, max) = (red(&state).1 - 100, red(&state).2);
        let mut draws = Rng::for_stream(seed, StreamId::player(0));
        let mut expected = draws.clone();
        let place = usize::try_from(expected.below(3)).unwrap();
        seen[place] = true;
        let (outcome, events) = whirl(&mut state, &rules, &store, &mut draws);
        // One draw for the Whirlpool and its tile among the three candidates.
        assert_eq!(draws, expected, "seed {seed}");
        // The loss first, whatever follows.
        assert_eq!(losses(&events), [(HERO, 0, PIKEMAN, 2)]);
        assert!(matches!(
            events[1].kind,
            AdventureEventKind::WhirlpoolLoss { .. }
        ));
        let drawn = candidates[place];
        if drawn == p(20, 1) {
            // Blue's hero on the exit (A-37 fixed by the battle, decision 265, `ADV-QUIRK-39`):
            // the tiles within 1 of it explored by Red, the hero still on the entrance in its
            // boat, the battle at the exit on the two boats with the jump held after it.
            assert_eq!(red(&state), (INTO[0], points, max));
            assert!(jumps(&events).is_empty());
            assert!(state.is_explored(RED, p(21, 0)) && !state.is_explored(RED, p(22, 1)));
            let battle = fought_at(&outcome, drawn, HeroId::new(2));
            assert_eq!(battle.battlefield, Some(key("t:boats")));
        } else {
            assert_eq!(outcome, AdventureOutcome::default());
            // On the exit in its boat, the points and the maximum kept; the boat's record keeps
            // the tile it was boarded on (ADR-0016 point 1).
            assert_eq!(red(&state), (drawn, points, max));
            assert_eq!(state.hero(HERO).unwrap().boat, Some(0));
            assert_eq!(state.boat(0).unwrap().tile, p(1, 1));
            assert_eq!(jumps(&events), [(Some(INTO[0]), Some(drawn))]);
        }
    }
    assert_eq!(seen, [true; 3]);
}
