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
    StackId, Viewer, apply, effects, generate_setup_obstacles, legal_commands, observe, observe_as,
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
fn a_town_with_a_fort_sets_up_no_ballista_for_its_defender() {
    let r = ruleset();
    let machine = |k: &str| r.creature_by_key(&std::format!("t:{k}")).unwrap();
    // The war machines each side's battle sets up, by creature.
    let machines = |fort, defender: Vec<_>| {
        let setup = BattleSetup {
            siege: Some(town(&r, "t:castle", fort, Moat::Absent)),
            war_machines: [vec![machine("ballista")], defender],
            heroes: [Some(hero(&r)), Some(hero(&r))],
            ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        BattleState::new(&r, &setup, 1).map(|s| {
            BattleSide::BOTH.map(|side| {
                s.stacks()
                    .iter()
                    .filter(|st| st.side == side && st.has(crate::StackTraits::WAR_MACHINE))
                    .map(|st| st.creature)
                    .collect::<Vec<_>>()
            })
        })
    };
    let listed = vec![machine("ballista"), machine("tent")];
    // At a town without a fort the defender keeps its Ballista; with a fort the battle leaves it
    // out and keeps the rest; the attacker's Ballista stays beside its Catapult.
    assert_eq!(
        machines(FortLevel::None, listed.clone()).unwrap(),
        [vec![machine("ballista")], listed.clone()]
    );
    for fort in [FortLevel::Fort, FortLevel::Citadel, FortLevel::Castle] {
        let [attacker, defender] = machines(fort, listed.clone()).unwrap();
        assert_eq!(attacker, [machine("ballista"), machine("catapult")]);
        assert_eq!(defender, [machine("tent")]);
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
