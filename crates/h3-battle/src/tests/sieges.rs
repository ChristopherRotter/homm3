//! Sieges (E04c T5; `docs/formats/installation.md` §7.27, §7.28): a battle at a town
//! (BAT-SIE-001), its walls (BAT-SIE-002), its gate (BAT-SIE-003), its moat (BAT-SIE-004 to
//! BAT-SIE-006), the Tower's land mines (BAT-SIE-007) and the shot through the walls
//! (BAT-SIE-008), on the test ruleset's siege field (the original's hexes) and records
//! (`support`): `t:castle` a moat of 70, `t:inferno` one of 90 with the second row (the
//! Fortress's shape), `t:necropolis` the mines of `t:landmine` (at least 150).
//!
//! The field's rows, their wall column (the moat one to the left, the second row two):
//! 12, 12, 11, 11, 10, 11 (the gate, 96; the drawbridge 95, the outer bridge 94), 10, 11, 11, 12,
//! 12.

use std::vec;
use std::vec::Vec;

use h3_content::CreatureSelector;
use h3_content_battle::{BattleRuleset, ObstacleKind};
use h3_contracts::{
    BattleEffect, BattleHero, BattleSetup, BattleSide, FortLevel, Moat, Obstacle, Siege,
};
use h3_kernel::{Rng, StreamId};

use super::spells::arena_on;
use super::support::*;
use crate::raise::mark_corpses;
use crate::{
    BattleState, Command, CommandError, Event, EventKind, Hex, LegalCommands, NoopSink, SetupError,
    StackId, StackTraits, Viewer, apply, effects, generate_setup_obstacles, legal_commands,
    observe, observe_as,
};

/// The town of `faction` with `fort` and `moat`, nothing else.
fn town(r: &BattleRuleset, faction: &str, fort: FortLevel, moat: Moat) -> Siege {
    Siege {
        faction: r.faction_by_key(faction).unwrap(),
        fort,
        moat,
        structures: 0,
        escape_tunnel: false,
        morale: 0,
        luck: 0,
    }
}

/// A battle at `town` with the stacks placed at `(x, y)`, as `arena_on`; the first attacker
/// stack is active.
fn at_town(
    r: &BattleRuleset,
    town: Siege,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
) -> BattleState {
    arena_on(r, heroes, attacker, defender, 1, |s| s.siege = Some(town))
}

/// The wall column of each row (see the module's header).
const WALL_COLUMNS: [i32; 11] = [12, 12, 11, 11, 10, 11, 10, 11, 11, 12, 12];

/// Whether `hex` is a wall hex or inside the walls.
fn inside(r: &BattleRuleset, hex: Hex) -> bool {
    let (x, y) = field(r).coords(hex);
    x >= WALL_COLUMNS[usize::try_from(y).unwrap()]
}

/// The positions the active stack may walk or fly to.
fn destinations(s: &BattleState, r: &BattleRuleset) -> Vec<Hex> {
    let mut out = LegalCommands::new();
    legal_commands(s, r, s.side_to_act().unwrap(), &mut out);
    out.iter()
        .filter_map(|c| match *c {
            Command::Move { to } => Some(to),
            _ => None,
        })
        .collect()
}

/// The gate's events in `events`: `true` opened, `false` closed.
fn gate(events: &[Event]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::GateOpened => Some(true),
            EventKind::GateClosed => Some(false),
            _ => None,
        })
        .collect()
}

/// The moat's damage `events` report, `(stack, hex, damage, kills)`.
fn soaked(events: &[Event]) -> Vec<(u8, Hex, u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::MoatDamage {
                stack,
                hex,
                damage,
                kills,
            } => Some((stack.0, hex, damage, kills)),
            _ => None,
        })
        .collect()
}

/// Whether the town's gate is open.
fn open(s: &BattleState) -> bool {
    s.siege.unwrap().gate_open
}

/// A walk of the active stack to `(x, y)`.
fn walk(r: &BattleRuleset, x: i32, y: i32) -> Command {
    Command::Move { to: at(r, x, y) }
}

// ---------------------------------------------------------------- BAT-SIE-001

// covers: BAT-SIE-001
#[test]
fn a_town_needs_its_record_and_a_citadel_for_its_moat() {
    let r = ruleset();
    let start = |r: &BattleRuleset, siege: Siege, obstacles: Vec<Obstacle>| {
        let setup = BattleSetup {
            siege: Some(siege),
            obstacles,
            ..setup(r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        BattleState::new(r, &setup, 1)
    };
    // A ruleset without sieges, and one without the castle's record.
    let bare = ruleset_from("t:faithful", |p| p.siege = None, |_, b| b.sieges.clear());
    let castle = town(&bare, "t:castle", FortLevel::Castle, Moat::Damaging);
    assert_eq!(start(&bare, castle, vec![]), Err(SetupError::UnknownSiege));
    let other = ruleset_from(
        "t:faithful",
        |_| {},
        |_, b| b.sieges.retain(|d| d.faction.as_str() != "t:castle"),
    );
    let castle = town(&other, "t:castle", FortLevel::Castle, Moat::Damaging);
    assert_eq!(start(&other, castle, vec![]), Err(SetupError::UnknownSiege));
    // A moat needs a Citadel.
    let fort = |faction, moat| town(&r, faction, FortLevel::Fort, moat);
    let citadel = |faction, moat| town(&r, faction, FortLevel::Citadel, moat);
    assert_eq!(
        start(&r, fort("t:castle", Moat::Harmless), vec![]),
        Err(SetupError::InvalidSiege)
    );
    // No natural obstacle on the walls, the moat or the Tower's mines.
    let on = |x, y| {
        vec![Obstacle {
            hexes: vec![at(&r, x, y).0],
            anchor: None,
            kind: None,
        }]
    };
    let invalid = Err(SetupError::InvalidSiege);
    assert_eq!(
        start(&r, fort("t:castle", Moat::Absent), on(11, 5)),
        invalid
    );
    assert_eq!(
        start(&r, citadel("t:castle", Moat::Harmless), on(9, 4)),
        invalid
    );
    assert!(start(&r, fort("t:castle", Moat::Absent), on(9, 4)).is_ok());
    assert_eq!(
        start(&r, citadel("t:necropolis", Moat::Harmless), on(9, 4)),
        invalid
    );
    assert!(start(&r, citadel("t:necropolis", Moat::Harmless), on(10, 5)).is_ok());
    // The town is public and joins the hash.
    let castle = citadel("t:castle", Moat::Damaging);
    let s = start(&r, castle, vec![]).unwrap();
    let plain = battle(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)], 1);
    assert_ne!(s.state_hash(), plain.state_hash());
    for o in [
        observe(&s, &r, BattleSide::Attacker),
        observe(&s, &r, BattleSide::Defender),
        observe_as(&s, &r, Viewer::Spectator),
    ] {
        let t = o.siege.unwrap();
        assert_eq!(
            (t.faction, t.fort, t.moat_kind, t.gate_open),
            (castle.faction, FortLevel::Citadel, Moat::Damaging, false)
        );
        assert_eq!((t.walls().len(), t.moat().len()), (11, 11));
    }
    assert!(observe(&plain, &r, BattleSide::Attacker).siege.is_none());
    // Without a fort: no walls, no gate (it counts as open), no moat.
    let s = start(
        &r,
        town(&r, "t:castle", FortLevel::None, Moat::Absent),
        vec![],
    )
    .unwrap();
    let t = s.siege.unwrap();
    assert!(t.walls().is_empty() && t.moat().is_empty() && t.gate_open);
}

// covers: BAT-SIE-001, BAT-GEO-004
#[test]
fn a_town_with_a_fort_has_no_random_obstacles() {
    let r = ruleset();
    let draw = |fort| {
        let setup = BattleSetup {
            terrain: r.terrain_by_key("t:plain"),
            siege: Some(town(&r, "t:castle", fort, Moat::Absent)),
            ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        let mut rng = Rng::for_stream(9, StreamId::OBSTACLES);
        generate_setup_obstacles(&r, &setup, &mut rng)
    };
    assert!(draw(FortLevel::Fort).is_empty());
    assert!(!draw(FortLevel::None).is_empty());
}

// ---------------------------------------------------------------- BAT-SIE-002

// covers: BAT-SIE-002
#[test]
fn the_walls_block_walkers_and_landings() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let s = at_town(
        &r,
        fort,
        [None, None],
        &[("fast", 1, 8, 5), ("flyer", 1, 8, 3)],
        &[("fixed", 1, 14, 9)],
    );
    let walls = s.siege.unwrap().walls();
    // The walker reaches no hex inside while the gate is closed.
    let walker = destinations(&s, &r);
    assert!(walker.contains(&at(&r, 9, 5)));
    assert!(walker.iter().all(|&h| !inside(&r, h)));
    // The flyer flies over the walls but lands on none of them.
    let mut t = s.clone();
    activate(&mut t, 1);
    let flyer = destinations(&t, &r);
    assert!(flyer.contains(&at(&r, 12, 3)));
    assert!(flyer.iter().all(|&h| !walls.contains(h)));
    let mut t = s.clone();
    assert_eq!(
        act(&mut t, &r, walk(&r, 10, 4)),
        Err(CommandError::Unreachable)
    );
}

// ---------------------------------------------------------------- BAT-SIE-003

// covers: BAT-SIE-003
#[test]
fn the_defender_opens_the_gate_and_it_closes_behind_it() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    // The defender's walker inside on (13, 5), the attacker's stack far away.
    let s = at_town(
        &r,
        fort,
        [None, None],
        &[("fixed", 1, 2, 1)],
        &[("fast", 1, 13, 5)],
    );
    let mut s1 = s.clone();
    activate(&mut s1, 1);
    // Out through the gate: it opens on the gate's hex and closes behind the stack.
    let mut t = s1.clone();
    let events = act(&mut t, &r, walk(&r, 7, 5)).unwrap();
    assert_eq!(gate(&events), [true, false]);
    assert!(!open(&t));
    // Without a recording sink the gate opens and closes alike: the same state.
    let mut quiet = s1.clone();
    apply(
        &mut quiet,
        &r,
        BattleSide::Defender,
        walk(&r, 7, 5),
        &mut NoopSink,
    )
    .unwrap();
    assert_eq!(quiet.state_hash(), t.state_hash());
    // Onto the drawbridge: the gate stays open while the stack stands there; the attacker may
    // then pass the gate's hex.
    let mut t = s1.clone();
    assert_eq!(gate(&act(&mut t, &r, walk(&r, 10, 5)).unwrap()), [true]);
    assert!(open(&t));
    let walls = t.siege.unwrap().walls();
    assert!(
        !crate::siege::blocked_for(&t, t.stack(StackId(0)).map(crate::Stack::controller))
            .contains(at(&r, 11, 5))
    );
    assert!(
        crate::siege::blocked_for(&t, t.stack(StackId(0)).map(crate::Stack::controller))
            .contains(at(&r, 10, 4))
    );
    assert!(walls.contains(at(&r, 11, 5)));
    // Back inside, the gate closes after the walk.
    activate(&mut t, 1);
    t.stacks.as_mut_slice()[1].flags = Default::default();
    assert_eq!(gate(&act(&mut t, &r, walk(&r, 13, 5)).unwrap()), [false]);
    assert!(!open(&t));
    // A walk inside never opens it.
    let mut t = s1.clone();
    assert!(gate(&act(&mut t, &r, walk(&r, 13, 8)).unwrap()).is_empty());
    // A flight's landing on the drawbridge opens it, and the flyer holds it there.
    let mut t = at_town(
        &r,
        fort,
        [None, None],
        &[("fixed", 1, 2, 1)],
        &[("flyer", 1, 13, 3)],
    );
    activate(&mut t, 1);
    assert_eq!(gate(&act(&mut t, &r, walk(&r, 10, 5)).unwrap()), [true]);
    assert!(open(&t));
}

// covers: BAT-SIE-003, BAT-TAC-002
#[test]
fn a_defender_s_placement_through_the_gate_opens_it_with_every_sink() {
    // E17 M5 T4c2b: a placement in the tactics phase is the original's walk (0x445950): out
    // through the closed gate it opens on the gate's hex and closes behind the stack.
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let mut tactician = hero(&r);
    tactician.tactics = 3;
    let mut s = at_town(
        &r,
        fort,
        [None, Some(tactician)],
        &[("fixed", 1, 2, 1)],
        &[("fast", 1, 13, 5)],
    );
    assert!(s.tactics().is_some_and(|t| t.side == BattleSide::Defender));
    activate(&mut s, 1);
    let mut t = s.clone();
    let events = act(&mut t, &r, walk(&r, 9, 4)).unwrap();
    assert_eq!(gate(&events), [true, false]);
    assert!(!open(&t));
    let mut quiet = s.clone();
    apply(
        &mut quiet,
        &r,
        BattleSide::Defender,
        walk(&r, 9, 4),
        &mut NoopSink,
    )
    .unwrap();
    assert_eq!(quiet.state_hash(), t.state_hash());
}

// covers: BAT-SIE-003
#[test]
fn a_stack_or_a_corpse_on_the_bridges_keeps_the_gate_shut_for_the_defender() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    // An attacker's stack on the outer bridge (9, 5); the defender's walker inside.
    let mut s = at_town(
        &r,
        fort,
        [None, None],
        &[("fixed", 1, 9, 5), ("fixed", 1, 2, 1)],
        &[("fast", 1, 13, 5)],
    );
    activate(&mut s, 2);
    let shut = |s: &BattleState| {
        let d = destinations(s, &r);
        !d.is_empty() && d.iter().all(|&h| inside(&r, h))
    };
    assert!(shut(&s));
    // Its corpse keeps it shut too; without either the gate lets the defender out.
    let stack = &mut s.stacks.as_mut_slice()[0];
    stack.count = 0;
    stack.top_hp = 0;
    effects::remove_dead(&mut s, &r);
    mark_corpses(&mut s);
    assert!(shut(&s));
    let mut t = s.clone();
    place(&mut t, &r, 0, 2, 9);
    assert!(!shut(&t));
    // Nor does the gate let the attacker through: a stack that is no defender's waits outside.
    let mut t = s.clone();
    activate(&mut t, 1);
    place(&mut t, &r, 0, 2, 9);
    place(&mut t, &r, 1, 9, 5);
    assert!(destinations(&t, &r).iter().all(|&h| !inside(&r, h)));
    // The defender's own stack on the outer bridge bars itself: neither bridge nor gate for it.
    let mut t = s.clone();
    place(&mut t, &r, 0, 2, 9);
    place(&mut t, &r, 2, 9, 5);
    let d = destinations(&t, &r);
    assert!(d.contains(&at(&r, 8, 5)));
    assert!(!d.contains(&at(&r, 10, 5)) && !d.contains(&at(&r, 11, 5)));
}

// covers: BAT-SIE-003, BAT-MAG-037
#[test]
fn a_created_stack_killed_on_the_drawbridge_lets_the_gate_close_before_good_morale() {
    // Good morale that always triggers (BAT-MOR-003).
    let r = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.good_morale_chances = vec![255, 255, 255];
    });
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    // The defender's stack on the drawbridge (10, 5) holds the gate open; the attacker's archers
    // shoot it dead and act again.
    let shoot = |created: bool| {
        let mut s = at_town(
            &r,
            fort,
            [None, None],
            &[("archer", 10, 8, 5)],
            &[("slow", 1, 10, 5), ("fixed", 1, 14, 9)],
        );
        let stack = &mut s.stacks.as_mut_slice()[1];
        stack.traits = stack.traits.with(StackTraits::SUMMONED, created);
        s.stacks.as_mut_slice()[0].morale = 1;
        let mut siege = s.siege.unwrap();
        siege.gate_open = true;
        s.siege = Some(siege);
        let target = StackId(1);
        let events = act(
            &mut s,
            &r,
            Command::Shoot {
                target,
                at: at(&r, 10, 5),
            },
        )
        .unwrap();
        assert!(!s.stacks()[1].is_alive());
        assert!(
            events
                .iter()
                .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
        );
        assert_eq!(s.active(), Some(StackId(0)));
        (s, gate(&events))
    };
    // A created stack leaves no corpse (BAT-MAG-037): the gate closes at the shot's end, so the
    // archers' second turn finds the drawbridge and the gate barred.
    let (s, events) = shoot(true);
    assert_eq!(events, [false]);
    assert!(!open(&s));
    let d = destinations(&s, &r);
    assert!(!d.contains(&at(&r, 10, 5)) && !d.contains(&at(&r, 11, 5)));
    // A corpse holds it open.
    let (s, events) = shoot(false);
    assert!(events.is_empty());
    assert!(open(&s));
    assert!(destinations(&s, &r).contains(&at(&r, 11, 5)));
}

// covers: BAT-SIE-003, BAT-SIE-004
#[test]
fn the_fortress_s_outer_bridge_holds_the_gate_open() {
    let r = ruleset();
    for (faction, held) in [("t:castle", false), ("t:inferno", true)] {
        let moat = town(&r, faction, FortLevel::Citadel, Moat::Harmless);
        let mut s = at_town(
            &r,
            moat,
            [None, None],
            &[("fixed", 1, 2, 1)],
            &[("fast", 1, 13, 5)],
        );
        activate(&mut s, 1);
        // The outer bridge (9, 5): the inferno's second row holds the gate, the castle's not.
        let events = act(&mut s, &r, walk(&r, 9, 5)).unwrap();
        let expected: &[bool] = if held { &[true] } else { &[true, false] };
        assert_eq!(gate(&events), expected, "{faction}");
        assert_eq!(open(&s), held);
        // The bridges are no moat while the gate is open.
        let moat = s.siege.unwrap().moat();
        assert_eq!(moat.contains(at(&r, 10, 5)), !held);
        assert!(!moat.contains(at(&r, 9, 5)));
    }
}

// covers: BAT-SIE-003, BAT-MOV-003
#[test]
fn a_walk_over_the_outer_bridge_opens_the_held_gate_with_every_sink() {
    let r = ruleset();
    // Without a moat the walk goes straight (`obstacles::go`); a damaging moat makes every walk
    // check its positions (`travel_through`).
    for (fort, moat) in [
        (FortLevel::Fort, Moat::Absent),
        (FortLevel::Citadel, Moat::Damaging),
    ] {
        let walled = town(&r, "t:castle", fort, moat);
        // The defender's walker outside on (9, 6), a defender's stack moved onto the closed
        // gate's hex (11, 5), as a Clone may be placed there.
        let mut s = at_town(
            &r,
            walled,
            [None, None],
            &[("fixed", 1, 2, 1)],
            &[("fast", 1, 9, 6), ("fixed", 1, 14, 9)],
        );
        place(&mut s, &r, 2, 11, 5);
        assert!(!open(&s));
        activate(&mut s, 1);
        // On to (8, 4) over the outer bridge (9, 5): the step there opens the gate, and the
        // stack on the gate's hex holds it open after the walk.
        let mut t = s.clone();
        let events = act(&mut t, &r, walk(&r, 8, 4)).unwrap();
        let path: Vec<Hex> = events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Moved { to, .. } => Some(to),
                _ => None,
            })
            .collect();
        assert_eq!(path, [at(&r, 9, 5), at(&r, 8, 4)], "{moat:?}");
        assert_eq!(gate(&events), [true], "{moat:?}");
        assert!(open(&t));
        assert!(soaked(&events).is_empty());
        // A sink that records nothing walks it alike: the same state.
        let mut quiet = s.clone();
        apply(
            &mut quiet,
            &r,
            BattleSide::Defender,
            walk(&r, 8, 4),
            &mut NoopSink,
        )
        .unwrap();
        assert!(open(&quiet), "{moat:?}");
        assert_eq!(quiet.state_hash(), t.state_hash(), "{moat:?}");
    }
}

// covers: BAT-MOV-003, BAT-SIE-003
/// Batch 3c M2 and G4 (the original without HD, played by an agent): of equally short paths a
/// walker takes its pathfinder's (`installation.md` §7.40), where BAT-MOV-003's choice from the
/// end (until E23 T1c) took the others. M2: the attacker's Magic Elementals (speed 9) walk from 86
/// (1, 5) into the moat's 111 (9, 6) of a Citadel along row 5 to 94 (9, 5) and step down, not
/// along row 6. G4: the defender's Minotaur Kings walk from the gate's 96 (11, 5) of a Fort out to
/// 76 (8, 4) over 95 and 94, not 77 (9, 4).
#[test]
fn batch_3c_m2_and_g4_walks_take_the_pathfinders_path() {
    let r = ruleset();
    let moved = |events: &[Event]| -> Vec<(i32, i32)> {
        events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Moved { to, .. } => Some(field(&r).coords(to)),
                _ => None,
            })
            .collect()
    };
    let citadel = town(&r, "t:castle", FortLevel::Citadel, Moat::Damaging);
    let mut s = at_town(
        &r,
        citadel,
        [None, None],
        &[("fast", 10, 1, 5)],
        &[("fixed", 1, 14, 9)],
    );
    let events = act(&mut s, &r, walk(&r, 9, 6)).unwrap();
    let mut row_5: Vec<(i32, i32)> = (2..10).map(|x| (x, 5)).collect();
    row_5.push((9, 6));
    assert_eq!(moved(&events), row_5);
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let mut s = at_town(
        &r,
        fort,
        [None, None],
        &[("fixed", 1, 2, 1)],
        &[("fixed", 1, 14, 9)],
    );
    place(&mut s, &r, 1, 11, 5);
    activate(&mut s, 1);
    let events = act(&mut s, &r, walk(&r, 8, 4)).unwrap();
    assert_eq!(moved(&events), [(10, 5), (9, 5), (8, 4)]);
}

// ---------------------------------------------------------------- BAT-SIE-004

// covers: BAT-SIE-004
#[test]
fn the_moat_stops_walkers_but_not_flyers_or_the_defender_on_the_bridges() {
    let r = ruleset();
    let inferno = town(&r, "t:inferno", FortLevel::Citadel, Moat::Harmless);
    let s = at_town(
        &r,
        inferno,
        [None, None],
        &[("fast", 1, 5, 4), ("flyer", 1, 5, 2)],
        &[("fast", 1, 13, 5)],
    );
    // The walker reaches the second row (8, 4) but not the inner moat (9, 4) behind it.
    let walker = destinations(&s, &r);
    assert!(walker.contains(&at(&r, 8, 4)) && !walker.contains(&at(&r, 9, 4)));
    let mut t = s.clone();
    act(&mut t, &r, walk(&r, 8, 4)).unwrap();
    assert_eq!(t.stacks()[0].hex, at(&r, 8, 4));
    // From the moat it goes on (its own hex is no stop), into the next moat hex.
    activate(&mut t, 0);
    t.stacks.as_mut_slice()[0].flags = Default::default();
    let walker = destinations(&t, &r);
    assert!(walker.contains(&at(&r, 9, 4)) && walker.contains(&at(&r, 5, 4)));
    // The flyer lands there at once.
    let mut t = s.clone();
    activate(&mut t, 1);
    assert!(destinations(&t, &r).contains(&at(&r, 9, 4)));
    // The defender crosses both bridges, but stops in the moat beside them.
    let mut t = s.clone();
    activate(&mut t, 2);
    let defender = destinations(&t, &r);
    assert!(defender.contains(&at(&r, 7, 5)) && defender.contains(&at(&r, 9, 6)));
    let mut t = s.clone();
    activate(&mut t, 2);
    act(&mut t, &r, walk(&r, 9, 6)).unwrap();
    assert_eq!(t.stacks()[2].hex, at(&r, 9, 6));
}

// covers: BAT-SIE-004
#[test]
fn a_walker_goes_round_a_moat_hex_and_attacks_from_none_it_enters() {
    let r = ruleset();
    let castle = town(&r, "t:castle", FortLevel::Citadel, Moat::Harmless);
    // From the moat (9, 6) to the moat (10, 8): round (10, 7) over (9, 7) and (9, 8), no stop
    // before the end (batch 3c M, the single row of the original's Inferno).
    let mut s = at_town(
        &r,
        castle,
        [None, None],
        &[("fast", 1, 9, 6)],
        &[("fixed", 1, 14, 9)],
    );
    assert!(destinations(&s, &r).contains(&at(&r, 10, 8)));
    let events = act(&mut s, &r, walk(&r, 10, 8)).unwrap();
    let steps: Vec<Hex> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Moved { to, .. } => Some(to),
            _ => None,
        })
        .collect();
    assert_eq!(steps, [at(&r, 9, 7), at(&r, 9, 8), at(&r, 10, 8)]);
    // An attack ordered with a walk that ends in the moat is no command (as from a seen pit,
    // BAT-MAG-032): the walker on (9, 3) may strike the stack on (9, 5) from (8, 4), not from the
    // moat (9, 4).
    let mut s = at_town(
        &r,
        castle,
        [None, None],
        &[("fast", 1, 9, 3)],
        &[("fixed", 1, 9, 5)],
    );
    let attack = |x, y| Command::Attack {
        target: StackId(1),
        from: at(&r, x, y),
    };
    let mut out = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut out);
    assert!(out.as_slice().contains(&attack(8, 4)));
    assert!(!out.as_slice().contains(&attack(9, 4)));
    assert_eq!(
        act(&mut s, &r, attack(9, 4)),
        Err(CommandError::Unreachable)
    );
}

// ---------------------------------------------------------------- BAT-SIE-005

// covers: BAT-SIE-005
#[test]
fn a_damaging_moat_hurts_every_stack_that_enters_it() {
    let r = ruleset();
    let castle = |moat| town(&r, "t:castle", FortLevel::Citadel, moat);
    // The attacker's walker (10 × 10 hit points) into the moat: 70 damage, 7 killed.
    let mut s = at_town(
        &r,
        castle(Moat::Damaging),
        [None, None],
        &[("fixed", 10, 7, 4)],
        &[("fast", 5, 13, 5)],
    );
    let events = act(&mut s, &r, walk(&r, 9, 4)).unwrap();
    assert_eq!(soaked(&events), [(0, at(&r, 9, 4), 70, 7)]);
    assert_eq!(s.stacks()[0].count, 3);
    // It takes the damage again when it acts where it stands.
    activate(&mut s, 0);
    s.stacks.as_mut_slice()[0].flags = Default::default();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(soaked(&events), [(0, at(&r, 9, 4), 70, 3)]);
    // A harmless moat does nothing.
    let mut s = at_town(
        &r,
        castle(Moat::Harmless),
        [None, None],
        &[("fixed", 10, 7, 4)],
        &[("fast", 5, 13, 5)],
    );
    assert!(soaked(&act(&mut s, &r, walk(&r, 9, 4)).unwrap()).is_empty());
    // The defender's stack out through the gate into the moat (the drawbridge is down).
    let mut s = at_town(
        &r,
        castle(Moat::Damaging),
        [None, None],
        &[("fixed", 10, 2, 1)],
        &[("fast", 30, 13, 5)],
    );
    activate(&mut s, 1);
    let events = act(&mut s, &r, walk(&r, 9, 4)).unwrap();
    assert_eq!(soaked(&events), [(1, at(&r, 9, 4), 70, 23)]);
    assert_eq!(gate(&events), [true, false]);
}

// covers: BAT-SIE-005
#[test]
fn a_two_hex_stack_is_hurt_once_and_the_moat_may_end_the_battle() {
    let r = ruleset();
    let inferno = town(&r, "t:inferno", FortLevel::Citadel, Moat::Damaging);
    // A dragon lands with both hexes in the inferno's two rows: 90 damage once (reported on the
    // front hex, the first it enters).
    let mut s = at_town(
        &r,
        inferno,
        [None, None],
        &[("dragon", 10, 5, 2)],
        &[("fast", 5, 13, 5)],
    );
    let events = act(&mut s, &r, walk(&r, 8, 4)).unwrap();
    assert_eq!(soaked(&events), [(0, at(&r, 9, 4), 90, 1)]);
    // The moat kills the attacker's last stack: the defender wins.
    let mut s = at_town(
        &r,
        inferno,
        [None, None],
        &[("fixed", 5, 6, 4)],
        &[("fast", 5, 13, 5)],
    );
    let events = act(&mut s, &r, walk(&r, 8, 4)).unwrap();
    assert_eq!(soaked(&events), [(0, at(&r, 8, 4), 90, 5)]);
    assert_eq!(
        s.result(),
        Some(h3_contracts::BattleResult::Victory(BattleSide::Defender))
    );
    // A flyer landing in the moat, and one that strikes and returns into it.
    let mut s = at_town(
        &r,
        inferno,
        [None, None],
        &[("flyer", 10, 5, 2), ("harpy", 10, 9, 4)],
        &[("fixed", 5, 13, 4)],
    );
    let events = act(&mut s, &r, walk(&r, 9, 3)).unwrap();
    assert_eq!(soaked(&events), [(0, at(&r, 9, 3), 90, 10)]);
    activate(&mut s, 1);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 12, 4),
        },
    )
    .unwrap();
    assert_eq!(soaked(&events), [(1, at(&r, 9, 4), 90, 9)]);
}

// covers: BAT-SIE-005
#[test]
fn a_stack_in_a_damaging_moat_is_hurt_again_whatever_it_does_where_it_stands() {
    let r = ruleset();
    // The attacker's walker in the moat (9, 4) beside the defender's stack on (8, 4), its archers
    // in the moat (10, 8); 10 creatures of 10 hit points each: 70 damage, 7 killed.
    let s = at_town(
        &r,
        town(&r, "t:castle", FortLevel::Citadel, Moat::Damaging),
        [None, None],
        &[("fixed", 10, 9, 4), ("archer", 10, 10, 8)],
        &[("slow", 1, 8, 4), ("fixed", 1, 14, 1)],
    );
    let hurt = |id: u8, command: Command| {
        let mut t = s.clone();
        activate(&mut t, id);
        soaked(&act(&mut t, &r, command).unwrap())
    };
    let walker = |x, y| vec![(0, at(&r, x, y), 70, 7)];
    // A wait, an attack from where it stands, a shot: the moat's damage each time.
    assert_eq!(hurt(0, Command::Wait), walker(9, 4));
    let attack = Command::Attack {
        target: StackId(2),
        from: at(&r, 9, 4),
    };
    assert_eq!(hurt(0, attack), walker(9, 4));
    let shot = Command::Shoot {
        target: StackId(2),
        at: at(&r, 8, 4),
    };
    assert_eq!(hurt(1, shot), [(1, at(&r, 10, 8), 70, 7)]);
    // A walk from the moat into the next moat hex: once, on that hex (its own hex is no entry).
    assert_eq!(hurt(0, walk(&r, 10, 3)), walker(10, 3));
}

// ---------------------------------------------------------------- BAT-SIE-006

// covers: BAT-SIE-006
#[test]
fn the_moat_lowers_the_defense_below_zero_but_not_a_frenzied_stack_s() {
    let r = ruleset();
    // `fixed` (attack 10, damage 2, 10 creatures: 20) strikes the stack on (9, 4).
    let strike = |faction: &str, moat, victim: &'static str, frenzy: u16| {
        let mut s = at_town(
            &r,
            town(&r, faction, FortLevel::Citadel, moat),
            [None, None],
            &[("fixed", 10, 8, 4)],
            &[(victim, 50, 9, 4)],
        );
        s.stacks.as_mut_slice()[1].spells.frenzy = frenzy;
        let events = act(
            &mut s,
            &r,
            Command::Attack {
                target: StackId(1),
                from: at(&r, 8, 4),
            },
        )
        .unwrap();
        events
            .iter()
            .find_map(|e| match e.kind {
                EventKind::Struck {
                    target: StackId(1),
                    damage,
                    ..
                } => Some(damage),
                _ => None,
            })
            .unwrap()
    };
    // Defense 10 - 3 = 7: +15 %, a harmless moat too.
    assert_eq!(strike("t:castle", Moat::Harmless, "fixed", 0), 23);
    assert_eq!(strike("t:castle", Moat::Absent, "fixed", 0), 20);
    // Defense 1 - 3 = -2: +60 % (D-10, not floored).
    assert_eq!(strike("t:castle", Moat::Damaging, "slow", 0), 32);
    // A frenzied stack's defense is 0, the moat's -3 not added.
    assert_eq!(strike("t:castle", Moat::Harmless, "fixed", 50), 30);
    // The Tower's mines are no moat.
    assert_eq!(strike("t:necropolis", Moat::Harmless, "fixed", 0), 20);
}

/// The damage stack `striker` deals stack `target` attacking it from where it stands.
fn struck(s: &mut BattleState, r: &BattleRuleset, striker: u8, target: u8) -> u64 {
    activate(s, striker);
    let from = s.stacks()[usize::from(striker)].hex;
    let target = StackId(target);
    let events = act(s, r, Command::Attack { target, from }).unwrap();
    events
        .iter()
        .find_map(|e| match e.kind {
            EventKind::Struck {
                target: t, damage, ..
            } if t == target => Some(damage),
            _ => None,
        })
        .unwrap()
}

// covers: BAT-SIE-006, BAT-MAG-019
#[test]
fn the_moat_s_penalty_reads_either_hex_the_closed_drawbridge_and_a_frenzied_striker() {
    let r = ruleset();
    let castle = |moat| town(&r, "t:castle", FortLevel::Citadel, moat);
    // The attacker's two-hex stack (defense 6) on (8, 4) with its front hex in the moat (9, 4),
    // struck by 10 `fixed` (damage 20) from (9, 5): defense 3, +35 %; without a moat +20 %.
    let wide = |moat| {
        let mut s = at_town(
            &r,
            castle(moat),
            [None, None],
            &[("wide", 10, 8, 4)],
            &[("fixed", 10, 9, 5)],
        );
        struck(&mut s, &r, 1, 0)
    };
    assert_eq!(wide(Moat::Harmless), 27);
    assert_eq!(wide(Moat::Absent), 24);
    // The drawbridge's hex is a moat hex only while the gate is closed: the defender's stack on
    // (10, 5) struck from (9, 5).
    let bridge = |open: bool| {
        let mut s = at_town(
            &r,
            castle(Moat::Harmless),
            [None, None],
            &[("fixed", 10, 9, 5)],
            &[("fixed", 50, 10, 5)],
        );
        let mut siege = s.siege.unwrap();
        siege.gate_open = open;
        s.siege = Some(siege);
        struck(&mut s, &r, 0, 1)
    };
    assert_eq!(bridge(false), 23);
    assert_eq!(bridge(true), 20);
    // A frenzied striker in the moat adds its defense less the moat's 3 (the original's 0x442410
    // asks 0x442590 for the striker's own defense): 10 + (10 - 3) × 100 % = 17 against defense
    // 10, +35 %; outside the moat 10 + 10, +50 %.
    let frenzied = |moat| {
        let mut s = at_town(
            &r,
            castle(moat),
            [None, None],
            &[("fixed", 10, 9, 4)],
            &[("fixed", 50, 8, 4)],
        );
        s.stacks.as_mut_slice()[0].spells.frenzy = 100;
        struck(&mut s, &r, 0, 1)
    };
    assert_eq!(frenzied(Moat::Harmless), 27);
    assert_eq!(frenzied(Moat::Absent), 30);
    // Not floored: a striker of defense 1 in the moat adds (1 - 3) × 100 %: attack 4 - 2 = 2
    // against `fixed`'s defense 10 (`slow`: damage 1, 50 creatures: 50; -8 × 2.5 % = -20 %).
    let mut s = at_town(
        &r,
        castle(Moat::Harmless),
        [None, None],
        &[("slow", 50, 9, 4)],
        &[("fixed", 50, 8, 4)],
    );
    let slow = &mut s.stacks.as_mut_slice()[0];
    slow.attack = 4;
    slow.spells.frenzy = 100;
    assert_eq!(struck(&mut s, &r, 0, 1), 40);
}

// ---------------------------------------------------------------- BAT-SIE-007

// covers: BAT-SIE-007
#[test]
fn the_tower_lays_hidden_mines_on_its_moat_by_the_game_s_version_and_its_hero() {
    let r = ruleset();
    // A defending hero of power `power` and fire mastery `fire`.
    let lord = |power: u8, fire: u8| {
        let mut h = hero(&r);
        h.power = power;
        h.magic.masteries = masteries(&r, [0, fire, 0, 0]);
        h
    };
    let values = |moat, fort, hero: Option<BattleHero>| {
        let s = at_town(
            &r,
            town(&r, "t:necropolis", fort, moat),
            [None, hero],
            &[("fixed", 1, 2, 1)],
            &[("fixed", 1, 14, 9)],
        );
        s.spell_obstacles()
            .iter()
            .map(|o| o.value)
            .collect::<Vec<_>>()
    };
    // Under SoD rules at least 150, the hero's when larger (power × 10 + 25, 25, 50, 100).
    let citadel = FortLevel::Citadel;
    assert_eq!(values(Moat::Damaging, citadel, None), [150; 10]);
    assert_eq!(values(Moat::Damaging, citadel, Some(lord(1, 0))), [150; 10]);
    assert_eq!(
        values(Moat::Damaging, citadel, Some(lord(20, 3))),
        [300; 10]
    );
    // Before: the hero's, or the spell's at power 3 without one.
    assert_eq!(values(Moat::Harmless, citadel, None), [55; 10]);
    assert_eq!(values(Moat::Harmless, citadel, Some(lord(1, 2))), [60; 10]);
    // The hero's mastery on the battlefield: Magic Plains make its Fire Magic Expert (BAT-TER-005),
    // 10 + 100 instead of 10 + 25.
    assert_eq!(values(Moat::Harmless, citadel, Some(lord(1, 0))), [35; 10]);
    let plains = arena_on(
        &r,
        [None, Some(lord(1, 0))],
        &[("fixed", 1, 2, 1)],
        &[("fixed", 1, 14, 9)],
        1,
        |s| {
            s.siege = Some(town(&r, "t:necropolis", citadel, Moat::Harmless));
            s.battlefield = h3_content::ContentKey::new("t", "plains").ok();
        },
    );
    let mines: Vec<u64> = plains.spell_obstacles().iter().map(|o| o.value).collect();
    assert_eq!(mines, [110; 10]);
    // No moat, no mines.
    assert!(values(Moat::Absent, FortLevel::Castle, None).is_empty());
    // The defender's, hidden from the attacker, on the moat but the drawbridge, for the battle.
    let s = at_town(
        &r,
        town(&r, "t:necropolis", citadel, Moat::Damaging),
        [None, None],
        &[("fixed", 20, 7, 4)],
        &[("fixed", 1, 14, 9)],
    );
    let mut hexes = Vec::new();
    for o in s.spell_obstacles() {
        assert_eq!(
            (o.kind, o.side, o.rounds, o.revealed),
            (ObstacleKind::Mine, BattleSide::Defender, 0, false)
        );
        hexes.extend(o.hexes.iter());
    }
    let moat: Vec<Hex> = (0..11)
        .map(|y| at(&r, WALL_COLUMNS[y] - 1, i32::try_from(y).unwrap()))
        .filter(|&h| h != at(&r, 10, 5))
        .collect();
    assert_eq!(hexes, moat);
    assert!(
        observe(&s, &r, BattleSide::Attacker)
            .spell_obstacles
            .is_empty()
    );
    assert_eq!(
        observe(&s, &r, BattleSide::Defender).spell_obstacles.len(),
        10
    );
    // No moat: the walker is not stopped, and a mine goes off under it.
    let mut t = s.clone();
    let events = act(&mut t, &r, walk(&r, 9, 4)).unwrap();
    assert!(soaked(&events).is_empty());
    assert_eq!(t.stacks()[0].count, 5);
}

// ---------------------------------------------------------------- BAT-SIE-008

/// The damage of a shot of stack `shooter` at the stack on `(x, y)` (stack ids: the attacker's
/// shooter 0, the defender's `fixed` 1), with damage 10 per creature.
fn shot(s: &BattleState, r: &BattleRuleset, shooter: u8, target: u8) -> u64 {
    let mut t = s.clone();
    activate(&mut t, shooter);
    let st = &mut t.stacks.as_mut_slice()[usize::from(shooter)];
    st.damage_min = 10;
    st.damage_max = 10;
    let at = t.stacks()[usize::from(target)].footprint(field(r)).front;
    let events = act(
        &mut t,
        r,
        Command::Shoot {
            target: StackId(target),
            at,
        },
    )
    .unwrap();
    events
        .iter()
        .find_map(|e| match e.kind {
            EventKind::Struck { damage, .. } => Some(damage),
            _ => None,
        })
        .unwrap()
}

// covers: BAT-SIE-008
#[test]
fn the_walls_halve_shots_from_outside_at_targets_inside() {
    // archer (attack 6) at fixed (defense 10), 10 creatures of 10 damage: 90, halved 45.
    for profile in ["t:faithful", "t:single"] {
        let r = ruleset_with(profile, |_| {});
        let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
        let damage = |shooter: (&'static str, i32, i32), target: (i32, i32)| {
            let s = at_town(
                &r,
                fort,
                [None, None],
                &[(shooter.0, 10, shooter.1, shooter.2)],
                &[("fixed", 10, target.0, target.1)],
            );
            shot(&s, &r, 0, 1)
        };
        // Through the closed gate, a wall on row 4, rising and falling lines.
        assert_eq!(damage(("archer", 5, 5), (13, 5)), 45, "{profile}");
        assert_eq!(damage(("archer", 5, 4), (13, 4)), 45);
        assert_eq!(damage(("archer", 6, 2), (13, 7)), 45);
        assert_eq!(damage(("archer", 6, 8), (13, 3)), 45);
        // At a target outside, or from inside: none.
        assert_eq!(damage(("archer", 5, 5), (9, 5)), 90);
        assert_eq!(damage(("archer", 12, 5), (14, 7)), 90);
        // A shooter the walls do not halve.
        assert_eq!(damage(("mage_s", 5, 5), (13, 5)), 90);
        // With the distance's penalty: a quarter.
        assert_eq!(damage(("archer", 2, 5), (14, 5)), 22);
    }
}

// covers: BAT-SIE-008
#[test]
fn the_line_s_integer_axis_is_the_x87_s() {
    use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};

    use crate::siege::{LineAxis, seventeenth};
    // The original's axis step by step in the FPU's arithmetic (as `line_blocked` computed it
    // until E04c T10a): each position the 53-bit sum stored as an `f32`, kept at 0 below it.
    // Every start, difference and longer difference of the field (17 columns, 11 rows), ends
    // outside it included.
    for long in 1..=16u32 {
        let reach = i32::try_from(long).unwrap();
        for d in -reach..=reach {
            let unit = X87::ratio(d.unsigned_abs(), long, DOUBLE_BITS)
                .unwrap()
                .round(SINGLE_BITS);
            let by = seventeenth(unit);
            for from in 0..=16i32 {
                let mut x87 = X87::int(u64::from(from.unsigned_abs()));
                let mut axis = LineAxis::new(from, d, long);
                for step in 0..17 * long {
                    x87 = if d < 0 { x87.minus(by) } else { x87.plus(by) }.round(SINGLE_BITS);
                    let column = axis.advance();
                    assert_eq!(
                        (axis.position(), i64::from(column)),
                        (x87.to_fixed(), i64::try_from(x87.trunc()).unwrap()),
                        "from {from}, difference {d} of {long}, step {step}"
                    );
                }
            }
        }
    }
    // A line from a hex to itself takes no step and stays where it starts.
    assert_eq!(LineAxis::new(7, 0, 0).column(), 7);
}

// covers: BAT-SIE-008
#[test]
fn the_open_gate_the_walls_absence_and_a_bow_let_shots_through() {
    let r = ruleset();
    let s = at_town(
        &r,
        town(&r, "t:castle", FortLevel::Fort, Moat::Absent),
        [None, None],
        &[("archer", 10, 5, 5)],
        &[("fixed", 10, 13, 5)],
    );
    assert_eq!(shot(&s, &r, 0, 1), 45);
    // The line through the open gate is free (D-8).
    let mut t = s.clone();
    let mut town_state = t.siege.unwrap();
    town_state.gate_open = true;
    t.siege = Some(town_state);
    assert_eq!(shot(&t, &r, 0, 1), 90);
    // A town without a fort has no walls.
    let none = at_town(
        &r,
        town(&r, "t:castle", FortLevel::None, Moat::Absent),
        [None, None],
        &[("archer", 10, 5, 5)],
        &[("fixed", 10, 13, 5)],
    );
    assert_eq!(shot(&none, &r, 0, 1), 90);
    // The hero's bow (NoRangePenalty): neither the distance's penalty nor the walls' (the hero's
    // attack 3 raises the archer's to 9: 97). The hero brings the Catapult (stack 1, BAT-SIE-009):
    // the defender's stack is 2.
    let mut archer = hero(&r);
    archer.effects.push(BattleEffect::NoRangePenalty {
        creatures: CreatureSelector::All,
    });
    let bow = at_town(
        &r,
        town(&r, "t:castle", FortLevel::Fort, Moat::Absent),
        [Some(archer), None],
        &[("archer", 10, 2, 5)],
        &[("fixed", 10, 14, 5)],
    );
    assert_eq!(shot(&bow, &r, 0, 2), 97);
    // The defender's shooter inside, turned to its target outside, is not halved.
    let back = at_town(
        &r,
        town(&r, "t:castle", FortLevel::Fort, Moat::Absent),
        [None, None],
        &[("fixed", 10, 5, 5)],
        &[("archer", 10, 13, 5)],
    );
    // archer (attack 6) at fixed (defense 10): 90.
    assert_eq!(shot(&back, &r, 1, 0), 90);
}

// ---------------------------------------------------------------- BAT-SIE-029

// covers: BAT-SIE-029
#[test]
fn with_bat_quirk_23_a_town_with_a_fort_sets_up_no_ballista_for_its_defender() {
    // With the quirk (profile `t:fort_ballista`) as the original; fixed (`t:faithful` has no
    // quirks) the defender's Ballista is set up at every town.
    town_ballistae(&ruleset_with("t:fort_ballista", |_| {}), true);
    town_ballistae(&ruleset(), false);
}

/// The war machines the battles at a town of each fort level set up for `r`, whose profile has
/// BAT-QUIRK-23 if `quirk`.
fn town_ballistae(r: &BattleRuleset, quirk: bool) {
    let machine = |k: &str| r.creature_by_key(&std::format!("t:{k}")).unwrap();
    // The war machines each side's battle sets up, by creature.
    let machines = |fort, defender: Vec<_>| {
        let setup = BattleSetup {
            siege: Some(town(r, "t:castle", fort, Moat::Absent)),
            war_machines: [vec![machine("ballista")], defender],
            heroes: [Some(hero(r)), Some(hero(r))],
            ..setup(r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        BattleState::new(r, &setup, 1).map(|s| {
            BattleSide::BOTH.map(|side| {
                s.stacks()
                    .iter()
                    .filter(|st| st.side == side && st.has(StackTraits::WAR_MACHINE))
                    .map(|st| st.creature)
                    .collect::<Vec<_>>()
            })
        })
    };
    let listed = vec![machine("ballista"), machine("tent")];
    // At a town without a fort the defender keeps its Ballista; with a fort the quirk leaves it
    // out and keeps the rest, the fix sets it up; the attacker's Ballista stays beside its
    // Catapult.
    assert_eq!(
        machines(FortLevel::None, listed.clone()).unwrap(),
        [vec![machine("ballista")], listed.clone()]
    );
    for fort in [FortLevel::Fort, FortLevel::Citadel, FortLevel::Castle] {
        let [attacker, defender] = machines(fort, listed.clone()).unwrap();
        // The Catapult's record comes first (D-23).
        assert_eq!(attacker, [machine("catapult"), machine("ballista")]);
        let kept = if quirk {
            vec![machine("tent")]
        } else {
            listed.clone()
        };
        assert_eq!(defender, kept);
    }
    // A Ballista listed twice is still refused.
    assert_eq!(
        machines(
            FortLevel::Fort,
            vec![machine("ballista"), machine("ballista")]
        ),
        Err(SetupError::InvalidWarMachine {
            side: BattleSide::Defender
        })
    );
}
