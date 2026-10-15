//! The wall shots (E04c T6; `docs/formats/installation.md` §7.29): the attacker's Catapult
//! (BAT-SIE-009), the wall-shot targets (BAT-SIE-010), Ballistics and the automatic Catapult
//! (BAT-SIE-011), a target's destruction (BAT-SIE-012), the Cyclops (BAT-SIE-013), the shot
//! (BAT-SIE-014) and Earthquake (BAT-SIE-015), on the test ruleset's siege field (the original's
//! targets: 0 the upper tower, 1 the upper wall on hex 29, 2 the mid-upper wall on 78, 3 the gate
//! on 96, 4 the mid-lower wall on 130, 5 the lower wall on 182, 6 the lower tower, 7 the keep)
//! and Ballistics table.

use std::vec;
use std::vec::Vec;

use h3_content_battle::{BattleRuleset, WallKind};
use h3_contracts::{BattleHero, BattleSetup, BattleSide, CatapultControl, FortLevel, Moat, Siege};
use h3_kernel::{Rng, StreamId};

use super::spells::spell;
use super::support::*;
use crate::siege::{CATAPULT_SLOT, damage_part, earthquake, nearest, shoot_walls};
use crate::{
    BattleState, CastTarget, Command, CommandError, Event, EventKind, LegalCommands, SetupError,
    StackId, StackTraits, apply, castable_spells, legal_commands,
};

/// The town of `faction` with `fort`, no moat.
fn town(r: &BattleRuleset, fort: FortLevel) -> Siege {
    Siege {
        faction: r.faction_by_key("t:castle").unwrap(),
        fort,
        moat: Moat::Absent,
        structures: 0,
        escape_tunnel: false,
        morale: 0,
        luck: 0,
    }
}

/// A battle at a town of `fort` with `heroes`, the armies' stacks placed at `(x, y)` (the
/// Catapult, if any, where the battle sets it up), started with its events; `edit` changes the
/// setup first.
fn siege(
    r: &BattleRuleset,
    (fort, heroes): (FortLevel, [Option<BattleHero>; 2]),
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
    edit: impl FnOnce(&mut BattleSetup),
) -> (BattleState, Vec<Event>) {
    let specs = |list: &[(&'static str, u32, i32, i32)]| -> Vec<Spec> {
        list.iter()
            .enumerate()
            .map(|(i, &(name, count, ..))| (u8::try_from(i).unwrap(), name, count))
            .collect()
    };
    let mut setup = BattleSetup {
        heroes,
        siege: Some(town(r, fort)),
        ..setup(r, &specs(attacker), &specs(defender))
    };
    edit(&mut setup);
    let mut events = Vec::new();
    let mut s = BattleState::start(r, &setup, 1, &mut events).unwrap();
    let army: Vec<u8> = (0u8..)
        .zip(s.stacks())
        .filter(|(_, st)| st.slot != CATAPULT_SLOT)
        .map(|(i, _)| i)
        .collect();
    for (&id, &(_, _, x, y)) in army.iter().zip(attacker.iter().chain(defender)) {
        place(&mut s, r, id, x, y);
    }
    (s, events)
}

/// A hero commanding its Catapult at Ballistics `level`.
fn ballistician(r: &BattleRuleset, level: u8) -> BattleHero {
    let mut h = hero(r);
    h.war_machines.catapult = Some(CatapultControl { level });
    h
}

/// The Catapult's stack id, if the battle has one.
fn catapult(s: &BattleState) -> Option<u8> {
    (0u8..)
        .zip(s.stacks())
        .find(|(_, st)| st.slot == CATAPULT_SLOT)
        .map(|(i, _)| i)
}

/// The wall shots `events` report, `(stack, aimed, part, damage)`.
fn shots(events: &[Event]) -> Vec<(u8, u8, u8, u8)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::WallShot {
                stack,
                aimed,
                part,
                damage,
            } => Some((stack.0, aimed, part, damage)),
            _ => None,
        })
        .collect()
}

/// The parts `events` report destroyed.
fn destroyed(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::WallDestroyed { part } => Some(part),
            _ => None,
        })
        .collect()
}

/// The legal commands of the side to act.
fn legal(s: &BattleState, r: &BattleRuleset) -> Vec<Command> {
    let mut out = LegalCommands::new();
    legal_commands(s, r, s.side_to_act().unwrap(), &mut out);
    out.iter().copied().collect()
}

/// The wall shots among `commands`, by part.
fn wall_shots(commands: &[Command]) -> Vec<u8> {
    commands
        .iter()
        .filter_map(|c| match *c {
            Command::ShootWall { part } => Some(part),
            _ => None,
        })
        .collect()
}

/// Sets the parts' hit points of the town of `s`.
fn set_parts(s: &mut BattleState, parts: [u8; 8]) {
    let mut t = s.siege.unwrap();
    t.parts = parts;
    s.siege = Some(t);
}

// ---------------------------------------------------------------- BAT-SIE-009

// covers: BAT-SIE-009
#[test]
fn a_led_attacker_brings_the_catapult_to_a_fort() {
    let r = ruleset();
    let catapult_type = r.creature_by_key("t:catapult").unwrap();
    let led = [Some(hero(&r)), None];
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, led.clone()),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    // After the army (stack 0), before the defender's (stack 2): on hex 119 and 120.
    let id = catapult(&s).unwrap();
    assert_eq!(id, 1);
    let c = &s.stacks()[1];
    assert_eq!(
        (c.creature, c.side, c.count, c.hex),
        (catapult_type, BattleSide::Attacker, 1, crate::Hex(119))
    );
    assert!(c.footprint(field(&r)).covers(crate::Hex(120)));
    assert!(c.has(StackTraits::WAR_MACHINE) && !c.has(StackTraits::SHOOTER));
    // None without the attacker's hero, without a fort, for the defender's hero, outside towns.
    let none = |fort, heroes: [Option<BattleHero>; 2]| {
        catapult(
            &siege(
                &r,
                (fort, heroes),
                &[("fixed", 5, 3, 5)],
                &[("fixed", 5, 13, 5)],
                |_| {},
            )
            .0,
        )
    };
    assert_eq!(none(FortLevel::Fort, [None, None]), None);
    assert_eq!(none(FortLevel::None, led.clone()), None);
    assert_eq!(none(FortLevel::Castle, [None, Some(hero(&r))]), None);
    let open = BattleState::new(
        &r,
        &BattleSetup {
            heroes: led.clone(),
            ..setup(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)])
        },
        1,
    )
    .unwrap();
    assert_eq!(catapult(&open), None);
    // A setup never lists it.
    let listed = BattleSetup {
        heroes: led,
        siege: Some(town(&r, FortLevel::Fort)),
        war_machines: [vec![catapult_type], vec![]],
        ..setup(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)])
    };
    assert_eq!(
        BattleState::new(&r, &listed, 1).unwrap_err(),
        SetupError::InvalidWarMachine {
            side: BattleSide::Attacker
        }
    );
    // Its place taken: no room; a ruleset without a Catapult creature sets none up.
    let taken = BattleSetup {
        obstacles: blocked(vec![120]),
        ..listed.clone()
    };
    let taken = BattleSetup {
        war_machines: [vec![], vec![]],
        ..taken
    };
    assert_eq!(
        BattleState::new(&r, &taken, 1).unwrap_err(),
        SetupError::NoRoom {
            side: BattleSide::Attacker,
            slot: CATAPULT_SLOT
        }
    );
    let without = ruleset_from(
        "t:faithful",
        |_| {},
        |core, battle| {
            core.creatures.retain(|c| c.key.name() != "catapult");
            battle.creatures.retain(|c| c.key.name() != "catapult");
        },
    );
    let s = BattleState::new(
        &without,
        &BattleSetup {
            war_machines: [vec![], vec![]],
            ..BattleSetup {
                heroes: [Some(hero(&without)), None],
                siege: Some(town(&without, FortLevel::Fort)),
                ..setup(&without, &[(0, "fixed", 5)], &[(0, "fixed", 5)])
            }
        },
        1,
    )
    .unwrap();
    assert_eq!(catapult(&s), None);
}

// covers: BAT-SIE-009, BAT-SIE-011
#[test]
fn the_catapult_shoots_first_each_round_and_never_at_stacks() {
    let r = ruleset();
    // The fast defender would act first; the Catapult (speed 0) comes before it every round.
    let (mut s, events) = siege(
        &r,
        (FortLevel::Fort, [Some(hero(&r)), None]),
        &[("fixed", 5, 3, 5)],
        &[("fast", 5, 13, 5)],
        |_| {},
    );
    let first = |events: &[Event]| {
        events.iter().find_map(|e| match e.kind {
            EventKind::TurnStarted { stack } => Some(stack.0),
            _ => None,
        })
    };
    assert_eq!(first(&events), Some(1));
    assert_eq!(shots(&events).len(), 1);
    // Rounds 2 and 3 start with it too.
    let mut later = Vec::new();
    while s.round < 3 || s.active() != Some(StackId(2)) {
        let side = s.side_to_act().unwrap();
        apply(&mut s, &r, side, Command::Defend, &mut later).unwrap();
    }
    let starts: Vec<usize> = (0..later.len())
        .filter(|&i| matches!(later[i].kind, EventKind::RoundStarted { .. }))
        .collect();
    assert_eq!(starts.len(), 2);
    for i in starts {
        assert_eq!(first(&later[i..]), Some(1));
    }
    // Commanded, it awaits a command with no shot or move at a stack, no Wait and no Defend.
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, [Some(ballistician(&r, 1)), None]),
        &[("fixed", 5, 3, 5)],
        &[("fast", 5, 13, 5)],
        |_| {},
    );
    assert_eq!(s.active(), Some(StackId(1)));
    let commands = legal(&s, &r);
    assert!(
        commands
            .iter()
            .all(|c| matches!(c, Command::ShootWall { .. } | Command::Retreat))
    );
    assert_eq!(wall_shots(&commands), [1, 2, 3, 4, 5]);
}

// ---------------------------------------------------------------- BAT-SIE-010

// covers: BAT-SIE-010
#[test]
fn the_fort_level_sets_up_the_targets_and_their_hit_points() {
    let r = ruleset();
    let parts = |fort| {
        let (s, _) = siege(
            &r,
            (fort, [None, None]),
            &[("fixed", 5, 3, 5)],
            &[("fixed", 5, 13, 5)],
            |_| {},
        );
        let t = s.siege.unwrap();
        (
            (0..8).map(|p| t.stands(p)).collect::<Vec<_>>(),
            t.parts,
            t.targets,
        )
    };
    // A Fort: the walls and the gate; the turrets' parts keep their hit points (Earthquake's).
    let (standing, hp, n) = parts(FortLevel::Fort);
    assert_eq!(n, 8);
    assert_eq!(
        standing,
        [false, true, true, true, true, true, false, false]
    );
    assert_eq!(hp, [2; 8]);
    // A Citadel adds the keep; a Castle the towers and a hit point to each wall.
    assert_eq!(
        parts(FortLevel::Citadel).0,
        [false, true, true, true, true, true, false, true]
    );
    let (standing, hp, _) = parts(FortLevel::Castle);
    assert_eq!(standing, [true; 8]);
    assert_eq!(hp, [2, 3, 3, 2, 3, 3, 2, 2]);
    // Without a fort no target.
    let (standing, _, n) = parts(FortLevel::None);
    assert_eq!((n, standing), (0, vec![false; 8]));
    let t = siege(
        &r,
        (FortLevel::Castle, [None, None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    )
    .0
    .siege
    .unwrap();
    assert_eq!(
        (0..8).map(|p| t.kind(p).unwrap()).collect::<Vec<_>>(),
        [
            WallKind::Tower,
            WallKind::Wall,
            WallKind::Wall,
            WallKind::Gate,
            WallKind::Wall,
            WallKind::Wall,
            WallKind::Tower,
            WallKind::Keep
        ]
    );
    assert_eq!(t.kind(8), None);
}

// ---------------------------------------------------------------- BAT-SIE-011

// covers: BAT-SIE-011
#[test]
fn without_ballistics_the_catapult_aims_at_the_weakest_wall() {
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Castle, [Some(hero(&r)), None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    let aims = |parts: [u8; 8]| {
        let mut out = Vec::new();
        for seed in 0..40 {
            let mut t = s.clone();
            set_parts(&mut t, parts);
            t.rng = Rng::for_stream(seed, StreamId::BATTLE);
            let town = t.siege.unwrap();
            out.push(crate::siege::automatic_aim(
                &mut t,
                (town, r.siege_field().unwrap()),
            ));
        }
        out.sort_unstable();
        out.dedup();
        out
    };
    // The walls 1, 2, 4, 5: all of 3 hit points, a tie drawn among them.
    assert_eq!(
        aims([2, 3, 3, 2, 3, 3, 2, 2]),
        [Some(1), Some(2), Some(4), Some(5)]
    );
    // The weakest wall, ties drawn.
    assert_eq!(aims([2, 3, 1, 2, 3, 1, 2, 2]), [Some(2), Some(5)]);
    assert_eq!(aims([2, 0, 3, 2, 2, 3, 2, 2]), [Some(4)]);
    // No wall: the gate, the keep, the upper tower, the lower tower.
    assert_eq!(aims([2, 0, 0, 1, 0, 0, 2, 2]), [Some(3)]);
    assert_eq!(aims([2, 0, 0, 0, 0, 0, 2, 2]), [Some(7)]);
    assert_eq!(aims([2, 0, 0, 0, 0, 0, 2, 0]), [Some(0)]);
    assert_eq!(aims([0, 0, 0, 0, 0, 0, 2, 0]), [Some(6)]);
    assert_eq!(aims([0; 8]), [None]);
    // The automatic Catapult shoots with the record of no Ballistics: one shot.
    let (_, events) = siege(
        &r,
        (FortLevel::Fort, [Some(hero(&r)), None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    let shot = shots(&events);
    assert_eq!(shot.len(), 1);
    assert!([1, 2, 4, 5].contains(&shot[0].1));
}

/// The stacks whose turns `events` report started, in order.
fn turns(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::TurnStarted { stack } => Some(stack.0),
            _ => None,
        })
        .collect()
}

/// The first `n` turns of the battle `s` started with `events`: the side to act defends, a
/// commanded Catapult shoots its first standing target.
fn first_turns(
    s: &mut BattleState,
    r: &BattleRuleset,
    mut events: Vec<Event>,
    n: usize,
) -> Vec<u8> {
    while turns(&events).len() < n {
        let command = wall_shots(&legal(s, r))
            .first()
            .map_or(Command::Defend, |&part| Command::ShootWall { part });
        events.extend(act(s, r, command).unwrap());
    }
    turns(&events)[..n].to_vec()
}

// covers: BAT-SIE-011, BAT-TRN-001
#[test]
fn after_the_catapult_s_turn_the_attacker_wins_a_tie() {
    let r = ruleset();
    // Equal speeds: after the Catapult (stack 1) the attacker's stack acts first, every round,
    // whether the Catapult shoots on its own or on its hero's command (the original's pick,
    // 0x4650BB, records the defender as the side activated last).
    for catapult in [hero(&r), ballistician(&r, 1)] {
        let (mut s, events) = siege(
            &r,
            (FortLevel::Fort, [Some(catapult), None]),
            &[("fixed", 5, 3, 5)],
            &[("fixed", 5, 13, 5)],
            |_| {},
        );
        assert_eq!(first_turns(&mut s, &r, events, 7), [1, 0, 2, 1, 0, 2, 1]);
    }
}

// covers: BAT-SIE-011, BAT-SIE-016
#[test]
fn the_turrets_then_the_catapult_open_each_round() {
    let r = turret_ruleset("t:faithful");
    // A Castle: the keep (3), the upper (4) and the lower tower (5), then the Catapult (1), then
    // the attacker's stack, which wins the tie after the Catapult (BAT-TRN-001).
    let (mut s, events) = siege(
        &r,
        (FortLevel::Castle, [Some(hero(&r)), None]),
        &[("fixed", 200, 3, 5)],
        &[("fixed", 200, 13, 5)],
        |_| {},
    );
    assert_eq!(
        first_turns(&mut s, &r, events, 11),
        [3, 4, 5, 1, 0, 2, 3, 4, 5, 1, 0]
    );
}

// covers: BAT-SIE-011
#[test]
fn ballistics_commands_the_catapult() {
    let r = ruleset();
    for level in 1..=3 {
        let (mut s, events) = siege(
            &r,
            (FortLevel::Citadel, [Some(ballistician(&r, level)), None]),
            &[("fixed", 5, 3, 5)],
            &[("fixed", 5, 13, 5)],
            |_| {},
        );
        // It waits for the command: the walls, the gate and the keep.
        assert!(shots(&events).is_empty());
        assert_eq!(s.active(), Some(StackId(1)));
        assert_eq!(wall_shots(&legal(&s, &r)), [1, 2, 3, 4, 5, 7]);
        // Basic one shot, Advanced and Expert two.
        let fired = act(&mut s, &r, Command::ShootWall { part: 7 }).unwrap();
        assert_eq!(shots(&fired).len(), if level == 1 { 1 } else { 2 });
        assert_ne!(s.active(), Some(StackId(1)));
    }
    // No Wait and no Defend (the original offers neither, BAT-Q50); a target not set up,
    // destroyed or no target.
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, [Some(ballistician(&r, 2)), None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    for part in [0, 6, 7, 8] {
        let mut t = s.clone();
        assert_eq!(
            act(&mut t, &r, Command::ShootWall { part }),
            Err(CommandError::NoSuchPart)
        );
    }
    let mut t = s.clone();
    set_parts(&mut t, [2, 0, 2, 2, 2, 2, 2, 2]);
    assert_eq!(
        act(&mut t, &r, Command::ShootWall { part: 1 }),
        Err(CommandError::NoSuchPart)
    );
    for command in [Command::Wait, Command::Defend] {
        let mut t = s.clone();
        assert_eq!(act(&mut t, &r, command), Err(CommandError::MustShoot));
    }
    // With nothing standing a commanded Catapult's turn passes: at once when its hero's
    // Earthquake brings down the last standing target (it may not Wait or Defend), and at its
    // turn's start.
    let quake = spell(&r, "quake");
    let mut caster = ballistician(&r, 2);
    caster.magic.spell_book = Some(vec![quake]);
    caster.magic.mana = 40;
    caster.magic.masteries = vec![(quake, 3)];
    let (mut s, _) = siege(
        &r,
        (FortLevel::Fort, [Some(caster), None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    assert_eq!(s.active(), Some(StackId(1)));
    set_parts(&mut s, [0, 1, 0, 0, 0, 0, 0, 0]);
    let quaked = Command::Cast {
        spell: quake,
        target: CastTarget::None,
        second: CastTarget::None,
    };
    let mut events = act(&mut s, &r, quaked).unwrap();
    assert_eq!(destroyed(&events), [1]);
    assert_eq!(s.active(), Some(StackId(0)));
    while s.round < 2 {
        events.extend(act(&mut s, &r, Command::Defend).unwrap());
    }
    assert_ne!(s.active(), Some(StackId(1)));
    assert!(shots(&events).is_empty());
}

// ---------------------------------------------------------------- BAT-SIE-012

// covers: BAT-SIE-012
#[test]
fn a_destroyed_wall_opens_its_hex() {
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("fast", 5, 5, 4)],
        &[("fixed", 5, 14, 9)],
        |_| {},
    );
    let reach = |s: &BattleState| {
        let mut out = LegalCommands::new();
        legal_commands(s, &r, BattleSide::Attacker, &mut out);
        out.iter()
            .any(|c| *c == Command::Move { to: crate::Hex(78) })
    };
    assert!(!reach(&s));
    assert!(s.siege.unwrap().walls().contains(crate::Hex(78)));
    let mut t = s.clone();
    let mut events = Vec::new();
    damage_part(&mut t, 2, 2, &mut events);
    assert_eq!(destroyed(&events), [2]);
    assert!(reach(&t));
    assert!(!t.siege.unwrap().walls().contains(crate::Hex(78)));
    // Damage of 1 leaves it standing; damage to a destroyed part destroys nothing more.
    let mut t = s.clone();
    let mut events = Vec::new();
    damage_part(&mut t, 2, 1, &mut events);
    assert!(events.is_empty() && !reach(&t) && t.siege.unwrap().parts[2] == 1);
    damage_part(&mut t, 2, 2, &mut events);
    damage_part(&mut t, 2, 2, &mut events);
    assert_eq!(
        (destroyed(&events), t.siege.unwrap().parts[2]),
        (vec![2], 0)
    );
}

// covers: BAT-SIE-012
#[test]
fn a_destroyed_gate_stands_open_for_both_sides() {
    let r = ruleset();
    let fort = Siege {
        moat: Moat::Damaging,
        ..town(&r, FortLevel::Citadel)
    };
    let (s, _) = siege(
        &r,
        (FortLevel::Citadel, [None, None]),
        &[("fast", 5, 8, 5)],
        &[("fixed", 5, 14, 9)],
        |setup| setup.siege = Some(fort),
    );
    let gate = crate::Hex(96);
    let mut t = s.clone();
    let mut events = Vec::new();
    damage_part(&mut t, 3, 2, &mut events);
    assert_eq!(destroyed(&events), [3]);
    let town = t.siege.unwrap();
    assert!(town.gate_open && town.gate_destroyed());
    // The drawbridge's hex is no moat hex, the gate's no wall hex.
    assert!(!town.moat().contains(crate::Hex(95)) && !town.walls().contains(gate));
    // The attacker walks through, and the gate never closes again.
    let mut events = Vec::new();
    apply(
        &mut t,
        &r,
        BattleSide::Attacker,
        Command::Move { to: gate },
        &mut events,
    )
    .unwrap();
    assert_eq!(t.stacks()[0].hex, gate);
    assert!(
        events
            .iter()
            .all(|e| !matches!(e.kind, EventKind::GateClosed | EventKind::MoatDamage { .. }))
    );
    let mut u = t.clone();
    place(&mut u, &r, 0, 3, 5);
    crate::siege::close_gate(&mut u, &mut events);
    assert!(u.siege.unwrap().gate_open);
}

// ---------------------------------------------------------------- BAT-SIE-013

// covers: BAT-SIE-013
#[test]
fn an_attacker_s_cyclops_shoots_the_walls_without_shots() {
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("cyclops", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    let mut s = s;
    activate(&mut s, 0);
    assert_eq!(wall_shots(&legal(&s, &r)), [1, 2, 3, 4, 5]);
    let shots_before = s.stacks()[0].shots;
    let mut t = s.clone();
    let events = act(&mut t, &r, Command::ShootWall { part: 3 }).unwrap();
    assert_eq!(shots(&events).len(), 1);
    assert_eq!(t.stacks()[0].shots, shots_before);
    // Next to an enemy too.
    let mut t = s.clone();
    place(&mut t, &r, 1, 4, 5);
    assert_eq!(wall_shots(&legal(&t, &r)), [1, 2, 3, 4, 5]);
    // The Cyclops King shoots twice.
    let (mut k, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("cyclops_king", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    activate(&mut k, 0);
    assert_eq!(
        shots(&act(&mut k, &r, Command::ShootWall { part: 1 }).unwrap()).len(),
        2
    );
    // Not for the defender, not outside a town with a fort, not for other stacks (one with
    // other abilities either).
    let (mut o, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("sniper", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    activate(&mut o, 0);
    assert!(wall_shots(&legal(&o, &r)).is_empty());
    let (mut d, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("fixed", 5, 3, 5)],
        &[("cyclops", 5, 13, 5)],
        |_| {},
    );
    activate(&mut d, 1);
    assert!(wall_shots(&legal(&d, &r)).is_empty());
    assert_eq!(
        act(&mut d, &r, Command::ShootWall { part: 1 }),
        Err(CommandError::CannotShootWalls)
    );
    let (mut n, _) = siege(
        &r,
        (FortLevel::None, [None, None]),
        &[("cyclops", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    activate(&mut n, 0);
    assert_eq!(
        act(&mut n, &r, Command::ShootWall { part: 1 }),
        Err(CommandError::CannotShootWalls)
    );
    let (mut f, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    activate(&mut f, 0);
    assert_eq!(
        act(&mut f, &r, Command::ShootWall { part: 1 }),
        Err(CommandError::CannotShootWalls)
    );
    // Nothing standing.
    set_parts(&mut s, [2, 0, 0, 0, 0, 0, 2, 2]);
    assert_eq!(
        act(&mut s, &r, Command::ShootWall { part: 1 }),
        Err(CommandError::CannotShootWalls)
    );
}

// covers: BAT-SIE-013, BAT-SIE-011
#[test]
fn a_living_stack_in_the_gate_spares_it_from_the_cyclops() {
    let r = ruleset();
    // The defender's stack stands in the open gate: the original's cursor sends a click on the
    // gate's hex to the stack (0x4766AA to 0x4766B9), so the Cyclops has no shot at the gate.
    let in_gate = |s: &mut BattleState, id: usize| {
        let mut town = s.siege.unwrap();
        town.gate_open = true;
        s.siege = Some(town);
        s.stacks.as_mut_slice()[id].hex = crate::Hex(96);
    };
    let (mut s, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("cyclops", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    in_gate(&mut s, 1);
    activate(&mut s, 0);
    assert_eq!(wall_shots(&legal(&s, &r)), [1, 2, 4, 5]);
    assert_eq!(
        act(&mut s.clone(), &r, Command::ShootWall { part: 3 }),
        Err(CommandError::NoSuchPart)
    );
    // A dead stack there does not count.
    let mut t = s.clone();
    t.stacks.as_mut_slice()[1].count = 0;
    assert_eq!(wall_shots(&legal(&t, &r)), [1, 2, 3, 4, 5]);
    // The Catapult shoots it whoever stands there.
    let (mut c, _) = siege(
        &r,
        (FortLevel::Fort, [Some(ballistician(&r, 1)), None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    in_gate(&mut c, 2);
    assert_eq!(c.active(), Some(StackId(1)));
    assert_eq!(wall_shots(&legal(&c, &r)), [1, 2, 3, 4, 5]);
    assert_eq!(
        shots(&act(&mut c, &r, Command::ShootWall { part: 3 }).unwrap()).len(),
        1
    );
}

// ---------------------------------------------------------------- BAT-SIE-014

// covers: BAT-SIE-014
#[test]
fn a_missed_shot_lands_on_the_nearest_other_wall() {
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Castle, [None, None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    let field = r.siege_field().unwrap();
    let landed = |parts: [u8; 8], aimed: u8| {
        let mut out = Vec::new();
        for seed in 0..60 {
            let mut t = s.clone();
            set_parts(&mut t, parts);
            t.rng = Rng::for_stream(seed, StreamId::BATTLE);
            out.push(nearest(&mut t, &field.missed, aimed));
        }
        out.sort_unstable();
        out.dedup();
        out
    };
    let all = [2, 3, 3, 2, 3, 3, 2, 2];
    // The towers and the keep land on the wall beside them, a wall on its neighbour, the gate on
    // either wall beside it (a tie: half and half).
    assert_eq!(landed(all, 0), [Some(1)]);
    assert_eq!(landed(all, 7), [Some(5)]);
    assert_eq!(landed(all, 6), [Some(5)]);
    assert_eq!(landed(all, 1), [Some(2)]);
    assert_eq!(landed(all, 4), [Some(5)]);
    assert_eq!(landed(all, 3), [Some(2), Some(4)]);
    // Only standing walls; none: the aimed target itself.
    assert_eq!(landed([2, 0, 0, 2, 3, 3, 2, 2], 0), [Some(4)]);
    assert_eq!(landed([2, 3, 0, 0, 0, 0, 2, 2], 1), [None]);
    // The tie's direction: a roll of at most 50 takes the later entry of the list, above 50 keeps
    // the earlier (0x445F62 to 0x445F7A); both rolls occur among the seeds.
    let mut seen = [false; 2];
    for seed in 0..20 {
        let mut t = s.clone();
        t.rng = Rng::for_stream(seed, StreamId::BATTLE);
        let low = t.rng.clone().range_inclusive(1, 100) <= 50;
        seen[usize::from(low)] = true;
        let pick = |list: [u8; 2]| if low { list[1] } else { list[0] };
        assert_eq!(nearest(&mut t.clone(), &[2, 4], 3), Some(pick([2, 4])));
        assert_eq!(nearest(&mut t, &[4, 2], 3), Some(pick([4, 2])));
    }
    assert_eq!(seen, [true, true]);
}

// covers: BAT-SIE-014
#[test]
fn shots_roll_their_damage_and_hit_by_the_ballistics_record() {
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Castle, [None, None]),
        &[("cyclops", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    // Without Ballistics at the gate: 25 % hits, damage 0/1/2 at 10/60/30 %.
    let (mut hits, mut damage) = (0u32, [0u32; 3]);
    for seed in 0..2000 {
        let mut t = s.clone();
        t.rng = Rng::for_stream(seed, StreamId::BATTLE);
        let mut events = Vec::new();
        shoot_walls(
            &mut t,
            r.siege_field().unwrap(),
            (StackId(0), 3, 0),
            &mut events,
        );
        let shot = shots(&events);
        assert_eq!(shot.len(), 1);
        let (_, aimed, part, d) = shot[0];
        assert_eq!(aimed, 3);
        assert!(part == 3 || part == 2 || part == 4);
        hits += u32::from(part == 3);
        damage[usize::from(d)] += 1;
        assert_eq!(
            t.siege.unwrap().parts[usize::from(part)],
            s.siege.unwrap().parts[usize::from(part)] - d
        );
    }
    assert!((400..600).contains(&hits), "{hits}");
    assert!(
        (140..260).contains(&damage[0]) && (1080..1320).contains(&damage[1]),
        "{damage:?}"
    );
    // Expert: always 2 damage, two shots; once the aimed wall falls the second moves to the
    // nearest standing target of all (a neighbour, the gate among them).
    let mut seen = Vec::new();
    for seed in 0..200 {
        let mut t = s.clone();
        set_parts(&mut t, [2, 2, 2, 2, 2, 2, 2, 2]);
        t.rng = Rng::for_stream(seed, StreamId::BATTLE);
        let mut events = Vec::new();
        shoot_walls(
            &mut t,
            r.siege_field().unwrap(),
            (StackId(0), 2, 3),
            &mut events,
        );
        let shot = shots(&events);
        assert_eq!(shot.len(), 2);
        assert!(shot.iter().all(|&(_, _, _, d)| d == 2));
        if shot[0].2 == 2 {
            seen.push(shot[1].1);
        }
    }
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen, [1, 3]);
    // With nothing standing after the first shot, the second is not fired.
    let mut t = s.clone();
    set_parts(&mut t, [0, 0, 1, 0, 0, 0, 0, 0]);
    let mut events = Vec::new();
    shoot_walls(
        &mut t,
        r.siege_field().unwrap(),
        (StackId(0), 2, 3),
        &mut events,
    );
    assert_eq!(shots(&events).len(), 1);
}

// ---------------------------------------------------------------- BAT-SIE-015

// covers: BAT-SIE-015
#[test]
fn earthquake_draws_as_the_original() {
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    let quake = |parts: [u8; 8], hits: u32, seed: u64| {
        let mut t = s.clone();
        set_parts(&mut t, parts);
        t.rng = Rng::for_stream(seed, StreamId::BATTLE);
        let mut events = Vec::new();
        let town = t.siege.unwrap();
        earthquake(&mut t, town, hits, &mut events);
        let hit: Vec<(u8, u8)> = events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::WallQuaked { part, damage } => Some((part, damage)),
                _ => None,
            })
            .collect();
        (hit, t.siege.unwrap().parts)
    };
    // Two parts of 1 and two hits: the first part drawn first is drawn again (the hits drawn are
    // not subtracted), the second never alone (D-14).
    let mut outcomes = Vec::new();
    for seed in 0..60 {
        outcomes.push(quake([1, 1, 0, 0, 0, 0, 0, 0], 2, seed).0);
    }
    outcomes.sort_unstable();
    outcomes.dedup();
    assert_eq!(outcomes, [vec![(0, 1), (1, 1)], vec![(0, 2)]]);
    // At a Fort the absent turrets are drawn too and soak hits; nothing else happens to them.
    let mut soaked = false;
    for seed in 0..60 {
        let (hit, parts) = quake([2; 8], 4, seed);
        assert_eq!(hit.iter().map(|&(_, d)| u32::from(d)).sum::<u32>(), 4);
        soaked |= hit.iter().any(|&(p, _)| [0, 6, 7].contains(&p));
        assert!(parts.iter().all(|&p| p <= 2));
    }
    assert!(soaked);
    // Nothing standing: no hit.
    assert!(quake([0; 8], 4, 1).0.is_empty());
}

// covers: BAT-SIE-015
#[test]
fn earthquake_is_cast_by_either_side_at_a_town_with_a_fort() {
    let r = ruleset();
    let quake = spell(&r, "quake");
    let mut caster = hero(&r);
    caster.magic.spell_book = Some(vec![quake]);
    caster.magic.mana = 40;
    caster.magic.masteries = vec![(quake, 3)];
    let cast = Command::Cast {
        spell: quake,
        target: CastTarget::None,
        second: CastTarget::None,
    };
    for side in BattleSide::BOTH {
        let heroes = match side {
            BattleSide::Attacker => [Some(caster.clone()), None],
            BattleSide::Defender => [None, Some(caster.clone())],
        };
        let (mut s, _) = siege(
            &r,
            (FortLevel::Fort, heroes),
            &[("fixed", 5, 3, 5)],
            &[("fixed", 5, 13, 5)],
            |_| {},
        );
        let id = if side == BattleSide::Attacker { 0 } else { 1 };
        activate(&mut s, id);
        assert!(castable_spells(&s, &r, side).contains(quake));
        let mut events = Vec::new();
        apply(&mut s, &r, side, cast, &mut events).unwrap();
        // Expert: four hits of 1.
        let hits: u32 = events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::WallQuaked { damage, .. } => Some(u32::from(damage)),
                _ => None,
            })
            .sum();
        assert_eq!(hits, 4);
    }
    // Not without a fort, not outside a town.
    let heroes = [Some(caster.clone()), None];
    let (mut s, _) = siege(
        &r,
        (FortLevel::None, heroes.clone()),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5)],
        |_| {},
    );
    activate(&mut s, 0);
    assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(quake));
    assert!(act(&mut s, &r, cast).is_err());
}

// covers: BAT-SIE-015
#[test]
fn an_absent_turret_quaked_down_harms_no_stack() {
    // BAT-QUIRK-22 (S-08, exploit-fix): the original marks the stack an absent turret's record
    // names dead, the defender's first; the engine harms none.
    let r = ruleset();
    let (s, _) = siege(
        &r,
        (FortLevel::Fort, [None, None]),
        &[("fixed", 5, 3, 5)],
        &[("fixed", 5, 13, 5), ("slow", 5, 13, 7)],
        |_| {},
    );
    for t in [0u8, 6, 7] {
        let mut u = s.clone();
        let mut events = Vec::new();
        damage_part(&mut u, t, 2, &mut events);
        assert!(destroyed(&events).is_empty());
        assert_eq!(u.siege.unwrap().parts[usize::from(t)], 0);
        assert_eq!(u.stacks(), s.stacks());
    }
    let mut u = s.clone();
    set_parts(&mut u, [2, 0, 0, 0, 0, 0, 2, 2]);
    let mut events = Vec::new();
    let town = u.siege.unwrap();
    earthquake(&mut u, town, 8, &mut events);
    // The code's draw can spare the keep, the last in the order (D-14); the turrets fall or not,
    // no stack is touched.
    assert!(u.siege.unwrap().parts.iter().sum::<u8>() < 6);
    assert_eq!(u.stacks(), s.stacks());
    advance_to(&mut u, &r, 1);
}
