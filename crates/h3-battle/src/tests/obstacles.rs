//! Battlefield spells (E04b T5a): Force Field (BAT-MAG-027), Fire Wall (BAT-MAG-028), Teleport
//! (BAT-MAG-029), Expert Dispel's clearing (BAT-MAG-030) and a walker's path (BAT-MOV-003), after
//! the original's code (`docs/formats/installation.md` §7.15).

use std::vec::Vec;

use h3_content::{ObstacleKind, Ruleset, SpellScope};
use h3_contracts::{BattleEffect, BattleHero, BattleResult, BattleSide};

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, Hex, HexSet,
    LegalCommands, StackId, cast_targets, castable_spells, legal_commands, observe, second_targets,
};

/// A mage with `spells`, the masteries `[air, fire]` and 100 mana.
fn caster(r: &Ruleset, spells: &[&str], [air, fire]: [u8; 2]) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = air;
    h.magic.schools[1] = fire;
    h.magic.mana = 100;
    h
}

/// The hexes at `cells` as a set.
fn set(r: &Ruleset, cells: &[(i32, i32)]) -> HexSet {
    let mut out = HexSet::new();
    for &(x, y) in cells {
        out.insert(at(r, x, y));
    }
    out
}

/// Makes stack `id` active and casts `name` at the hex `(x, y)` for its side; returns the events.
fn raise(s: &mut BattleState, r: &Ruleset, id: u8, name: &str, (x, y): (i32, i32)) -> Vec<Event> {
    activate(s, id);
    act(
        s,
        r,
        Command::Cast {
            spell: spell(r, name),
            target: CastTarget::Hex(at(r, x, y)),
            second: CastTarget::None,
        },
    )
    .unwrap()
}

/// The spell obstacles' hexes, kinds and sides, in order.
fn obstacles(s: &BattleState) -> Vec<(HexSet, ObstacleKind, BattleSide)> {
    s.spell_obstacles()
        .iter()
        .map(|o| (o.hexes, o.kind, o.side))
        .collect()
}

/// The burns `events` report, as `(stack, hex, damage, kills)`.
fn burns(events: &[Event]) -> Vec<(u8, Hex, u64, u32)> {
    let mut out = Vec::new();
    let mut hex = None;
    for e in events {
        match e.kind {
            EventKind::ObstacleTriggered { hex: h, .. } => hex = Some(h),
            EventKind::SpellDamage {
                target,
                damage,
                kills,
                ..
            } => out.push((target.0, hex.take().unwrap(), damage, kills)),
            _ => {}
        }
    }
    out
}

/// The hexes `events` report each moved stack through, `(stack, from, to)`.
fn moves(events: &[Event]) -> Vec<(u8, Hex, Hex)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Moved { stack, from, to } => Some((stack.0, from, to)),
            _ => None,
        })
        .collect()
}

/// A walk of the active stack to `(x, y)`.
fn walk(r: &Ruleset, (x, y): (i32, i32)) -> Command {
    Command::Move { to: at(r, x, y) }
}

/// Plays Defend for every stack until round `round` starts; returns the events.
fn until_round(s: &mut BattleState, r: &Ruleset, round: u16) -> Vec<Event> {
    let mut events = Vec::new();
    while s.round() < round {
        events.extend(act(s, r, Command::Defend).unwrap());
    }
    events
}

// covers: BAT-MAG-027
#[test]
fn a_force_field_blocks_its_hexes_until_the_second_round_start() {
    let r = ruleset();
    let heroes = [
        Some(caster(&r, &["wall"], [1, 0])),
        Some(caster(&r, &["wall"], [2, 0])),
    ];
    let mut s = arena(&r, heroes, &[("fixed", 5, 4, 5)], &[("fixed", 5, 12, 5)], 1);
    // Basic: the target and its upper right neighbour, one obstacle (the original's 0x5A0759).
    let events = raise(&mut s, &r, 0, "wall", (6, 5));
    let basic = set(&r, &[(6, 5), (6, 4)]);
    assert!(events.contains(&Event::public(EventKind::ObstaclePlaced {
        side: BattleSide::Attacker,
        spell: spell(&r, "wall"),
        hexes: basic,
    })));
    assert_eq!(
        obstacles(&s),
        [(basic, ObstacleKind::Wall, BattleSide::Attacker)]
    );
    // Advanced, for the defender: also that one's upper left, not mirrored.
    raise(&mut s, &r, 1, "wall", (9, 6));
    let advanced = set(&r, &[(9, 6), (10, 5), (9, 4)]);
    assert_eq!(obstacles(&s)[1].0, advanced);
    // Nobody walks onto or through them, nor stands there.
    activate(&mut s, 0);
    assert_eq!(
        act(&mut s, &r, walk(&r, (6, 5))),
        Err(CommandError::Unreachable)
    );
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    let blocked = basic.union(advanced);
    assert!(!legal.iter().any(
        |c| matches!(c, Command::Move { to } | Command::Attack { from: to, .. } if blocked.contains(*to))
    ));
    // Both sides see them; a fork keeps them.
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&s, &r, viewer).spell_obstacles.len(), 2);
    }
    assert_eq!(s.clone(), s);
    // They last the rest of round 1 and round 2 (2 rounds), and go at the start of round 3.
    let events = until_round(&mut s, &r, 2);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::ObstacleRemoved { .. }))
    );
    assert_eq!(s.spell_obstacles()[0].rounds, 1);
    let events = until_round(&mut s, &r, 3);
    let gone: Vec<HexSet> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::ObstacleRemoved { hexes, .. } => Some(hexes),
            _ => None,
        })
        .collect();
    assert_eq!(gone, [basic, advanced]);
    assert!(s.spell_obstacles().is_empty());
    assert!(act(&mut s, &r, walk(&r, (6, 5))).is_ok());
}

// covers: BAT-MAG-027
#[test]
fn an_obstacle_needs_free_hexes() {
    let r = ruleset();
    let hero = caster(&r, &["wall", "firewall"], [1, 1]);
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    s.obstacles.insert(at(&r, 8, 8));
    let mut out = CastTargets::new();
    cast_targets(&s, &r, BattleSide::Attacker, spell(&r, "wall"), &mut out);
    let hexes: Vec<Hex> = out
        .iter()
        .map(|t| match *t {
            CastTarget::Hex(h) => h,
            other => panic!("{other:?}"),
        })
        .collect();
    let field = field(&r);
    // Not on a stack, nor where the shape reaches one ((1, 6)'s upper right is (2, 5)), an
    // obstacle ((8, 9)'s upper right is (8, 8)), a side column or beyond the top row.
    for (x, y) in [(2, 5), (1, 6), (8, 8), (8, 9), (0, 5), (15, 6), (5, 0)] {
        let target = at(&r, x, y);
        assert!(!hexes.contains(&target), "{x}, {y}");
        assert_eq!(
            act(
                &mut s.clone(),
                &r,
                Command::Cast {
                    spell: spell(&r, "wall"),
                    target: CastTarget::Hex(target),
                    second: CastTarget::None,
                }
            ),
            Err(CommandError::InvalidSpellTarget)
        );
    }
    // (15, 6)'s upper right is in the side column; (15, 5)'s is not.
    assert!(field.is_standable(at(&r, 15, 6)) && hexes.contains(&at(&r, 15, 5)));
    // Nor on another spell obstacle (the hero may cast again for the test).
    raise(&mut s, &r, 0, "firewall", (6, 6));
    cast_targets(&s, &r, BattleSide::Attacker, spell(&r, "wall"), &mut out);
    assert!(out.is_empty());
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    cast_targets(&s, &r, BattleSide::Attacker, spell(&r, "wall"), &mut out);
    for hex in [(6, 6), (7, 5), (6, 7)] {
        assert!(
            !out.as_slice()
                .contains(&CastTarget::Hex(at(&r, hex.0, hex.1)))
        );
    }
    assert!(out.as_slice().contains(&CastTarget::Hex(at(&r, 6, 4))));
}

// covers: BAT-MAG-028
#[test]
fn fire_walls_are_one_obstacle_per_hex_and_mirrored_for_the_defender() {
    let r = ruleset();
    let heroes = [
        Some(caster(&r, &["firewall"], [0, 2])),
        Some(caster(&r, &["firewall"], [0, 1])),
    ];
    let mut s = arena(&r, heroes, &[("fixed", 5, 2, 1)], &[("fixed", 5, 14, 1)], 1);
    // Advanced, for the attacker: the target, its upper right, that one's upper left.
    raise(&mut s, &r, 0, "firewall", (6, 6));
    // Basic, for the defender: the target and its upper left.
    raise(&mut s, &r, 1, "firewall", (10, 6));
    let one = |x, y| (set(&r, &[(x, y)]), ObstacleKind::Fire);
    let placed: Vec<(HexSet, ObstacleKind)> =
        obstacles(&s).iter().map(|&(h, k, _)| (h, k)).collect();
    assert_eq!(
        placed,
        [one(6, 6), one(7, 5), one(6, 4), one(10, 6), one(10, 5)]
    );
    // The value: power 3 × 10 + 20 at Advanced, + 10 at Basic.
    let values: Vec<u64> = s.spell_obstacles().iter().map(|o| o.value).collect();
    assert_eq!(values, [50, 50, 50, 40, 40]);
}

/// A battle where the defender's hero `hero` raised a Basic Fire Wall on (6, 6) and (6, 5) (40
/// damage to each stack that enters one), then the attacker's stacks were placed: `attacker` and
/// `defender` as in [`arena`], the first attacker stack active.
fn burning_with(
    r: &Ruleset,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
) -> BattleState {
    let parked: Vec<(&'static str, u32, i32, i32)> = (0i32..)
        .zip(attacker)
        .map(|(i, &(name, count, ..))| (name, count, 3, 2 * i))
        .collect();
    let mut s = arena(r, heroes, &parked, defender, 1);
    let caster_id = u8::try_from(attacker.len()).unwrap();
    raise(&mut s, r, caster_id, "firewall", (6, 6));
    for (i, &(_, _, x, y)) in (0u8..).zip(attacker) {
        place(&mut s, r, i, x, y);
    }
    assert_eq!(
        s.spell_obstacles()
            .iter()
            .fold(HexSet::new(), |h, o| h.union(o.hexes)),
        set(r, &[(6, 6), (6, 5)])
    );
    activate(&mut s, 0);
    s
}

/// [`burning_with`] the defender's caster only.
fn burning(
    r: &Ruleset,
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
) -> BattleState {
    let heroes = [None, Some(caster(r, &["firewall"], [0, 1]))];
    burning_with(r, heroes, attacker, defender)
}

// covers: BAT-MAG-028, BAT-MOV-003
#[test]
fn fire_burns_a_walker_on_each_hex_it_enters() {
    let r = ruleset();
    // A walk along row 5 passes (6, 5): one burn of 40, four of ten creatures; the stack goes on.
    let mut s = burning(&r, &[("fixed", 10, 4, 5)], &[("fixed", 5, 14, 9)]);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 4)]);
    // Reported in two moves, split where it burned.
    assert_eq!(
        moves(&events),
        [
            (0, at(&r, 4, 5), at(&r, 6, 5)),
            (0, at(&r, 6, 5), at(&r, 8, 5))
        ]
    );
    assert_eq!(s.stacks()[0].count, 6);
    // A two-hex stack enters each hex once: its tail follows its head through (6, 5).
    let mut s = burning(&r, &[("wide", 10, 4, 5)], &[("fixed", 5, 14, 9)]);
    let events = act(&mut s, &r, walk(&r, (9, 5))).unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 1)]);
    // Stopping on a burning hex burns once; leaving a hex it started on does not burn.
    let mut s = burning(&r, &[("fixed", 10, 5, 5)], &[("fixed", 5, 14, 9)]);
    let events = act(&mut s, &r, walk(&r, (6, 5))).unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 4)]);
    activate(&mut s, 0);
    s.stacks.as_mut_slice()[0].flags = crate::state::StackFlags::default();
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert!(burns(&events).is_empty());
    // A flyer burns only where it lands.
    let mut s = burning(&r, &[("flyer", 10, 4, 5)], &[("fixed", 5, 14, 9)]);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert!(burns(&events).is_empty());
    assert_eq!(moves(&events), [(0, at(&r, 4, 5), at(&r, 8, 5))]);
    let mut s = burning(&r, &[("flyer", 10, 4, 5)], &[("fixed", 5, 14, 9)]);
    let events = act(&mut s, &r, walk(&r, (6, 5))).unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 5)]);
}

// covers: BAT-MAG-028
#[test]
fn a_stack_that_dies_in_the_fire_stops_there_and_does_not_attack() {
    let r = ruleset();
    // One creature of 10 hit points: it dies on (6, 5) and never reaches the Wisps.
    let mut s = burning(
        &r,
        &[("fixed", 1, 4, 5), ("fixed", 5, 2, 9)],
        &[("wisp", 5, 9, 5)],
    );
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 8, 5),
        },
    )
    .unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 1)]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Struck { .. }))
    );
    assert_eq!(s.stacks()[0].hex, at(&r, 6, 5));
    assert!(!s.stacks()[0].is_alive());
    assert_eq!(s.stacks()[2].count, 5);
    // The last stack of a side dying in the fire ends the battle.
    let mut s = burning(&r, &[("fixed", 1, 4, 5)], &[("wisp", 5, 9, 5)]);
    let events = act(&mut s, &r, walk(&r, (8, 5))).unwrap();
    assert!(events.contains(&Event::public(EventKind::BattleEnded {
        result: BattleResult::Victory(BattleSide::Defender)
    })));
    assert_eq!(
        s.result(),
        Some(BattleResult::Victory(BattleSide::Defender))
    );
    assert_eq!(s.active(), None);
}

// covers: BAT-MAG-028
#[test]
fn a_stack_burns_again_when_it_acts_on_the_fire_without_moving() {
    let r = ruleset();
    let standing = || {
        burning(
            &r,
            &[("fixed", 20, 6, 5), ("archer", 20, 6, 6)],
            &[("fixed", 20, 7, 5)],
        )
    };
    // Defend, wait: after the action (the original's 0x4798F3, 0x479AD6).
    for (command, report) in [
        (Command::Defend, EventKind::Defended { stack: StackId(0) }),
        (Command::Wait, EventKind::Waited { stack: StackId(0) }),
    ] {
        let mut s = standing();
        let events = act(&mut s, &r, command).unwrap();
        assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 4)]);
        let k = |kind| events.iter().position(|e| e.kind == kind).unwrap();
        assert!(
            k(report)
                < events
                    .iter()
                    .position(|e| matches!(e.kind, EventKind::ObstacleTriggered { .. }))
                    .unwrap()
        );
    }
    // A melee attack from where it stands: the strike first, then the burn.
    let mut s = standing();
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 6, 5),
        },
    )
    .unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 4)]);
    let struck = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::Struck { .. }))
        .unwrap();
    assert!(struck < events.len() - 1);
    // A shot (the archer on (6, 6)).
    let mut s = standing();
    activate(&mut s, 1);
    s.stacks.as_mut_slice()[2].hex = at(&r, 12, 5);
    let events = act(
        &mut s,
        &r,
        Command::Shoot {
            target: StackId(2),
            at: at(&r, 12, 5),
        },
    )
    .unwrap();
    assert_eq!(burns(&events), [(1, at(&r, 6, 6), 40, 4)]);
    // A stack that walked in and attacks burns only on the way.
    let mut s = burning(&r, &[("fixed", 20, 4, 5)], &[("fixed", 20, 7, 5)]);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 6, 5),
        },
    )
    .unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 4)]);
}

// covers: BAT-MAG-028
#[test]
fn fire_damage_follows_immunities_reductions_and_the_casters_bonuses() {
    let r = ruleset();
    let count = |name, bonus: Option<u16>| {
        let heroes = [None, Some(caster(&r, &["firewall"], [0, 1]))];
        let mut heroes = heroes;
        if let (Some(h), Some(bp)) = (heroes[1].as_mut(), bonus) {
            h.effects.push(BattleEffect::SpellDamage {
                scope: SpellScope::All,
                rate: Rate::bp(bp.into()),
            });
        }
        let mut s = burning_with(&r, heroes, &[(name, 20, 6, 5)], &[("fixed", 5, 14, 9)]);
        let events = act(&mut s, &r, Command::Defend).unwrap();
        burns(&events).first().map(|b| b.2)
    };
    // The fire-immune Ember is spared; the Golem's spell damage reduction applies; the Dwarves'
    // magic resistance does not (no resistance roll, `installation.md` §7.15).
    assert_eq!(count("ember", None), None);
    assert_eq!(count("golem", None), Some(10));
    assert_eq!(count("dwarf", None), Some(40));
    // The bonuses of the hero who cast it: + 50 %.
    assert_eq!(count("fixed", Some(5000)), Some(60));
}

// covers: BAT-MAG-028
#[test]
fn a_two_hex_stack_burns_on_each_burning_hex_it_stands_on() {
    let r = ruleset();
    // The defender's fire on (6, 6) and (6, 5), the attacker's on (7, 5) and (7, 4): a two-hex
    // attacker with its head on (7, 5) stands on (7, 5) and (6, 5), and burns twice, its head's
    // hex first (40 each from either hero).
    let heroes = [
        Some(caster(&r, &["firewall"], [0, 1])),
        Some(caster(&r, &["firewall"], [0, 1])),
    ];
    let stand = |count| {
        let mut s = burning_with(
            &r,
            heroes.clone(),
            &[("wide", count, 3, 9)],
            &[("fixed", 5, 14, 9)],
        );
        raise(&mut s, &r, 0, "firewall", (7, 5));
        place(&mut s, &r, 0, 7, 5);
        activate(&mut s, 0);
        act(&mut s, &r, Command::Defend).unwrap()
    };
    assert_eq!(
        burns(&stand(4)),
        [(0, at(&r, 7, 5), 40, 1), (0, at(&r, 6, 5), 40, 2)]
    );
    // One creature of 25 hit points dies on its head's hex: the tail's does not burn it.
    let events = stand(1);
    assert_eq!(burns(&events), [(0, at(&r, 7, 5), 40, 1)]);
    assert!(events.contains(&Event::public(EventKind::BattleEnded {
        result: BattleResult::Victory(BattleSide::Defender)
    })));
}

// covers: BAT-MAG-028
#[test]
fn berserk_stacks_burn_like_commanded_ones() {
    let r = ruleset();
    let heroes = [
        Some(caster(&r, &["berserk"], [3, 0])),
        Some(caster(&r, &["firewall"], [0, 1])),
    ];
    // A berserk walker passes (6, 5) on its way to the Wisps; a berserk stack on the fire
    // attacks from where it stands and burns after its attack.
    for (start, burnt) in [((4, 5), 40), ((6, 5), 40)] {
        let mut s = burning_with(
            &r,
            heroes.clone(),
            &[("fixed", 20, start.0, start.1)],
            &[("wisp", 20, 7, 5)],
        );
        let mut events = act(
            &mut s,
            &r,
            Command::Cast {
                spell: spell(&r, "berserk"),
                target: CastTarget::Hex(at(&r, start.0, start.1)),
                second: CastTarget::None,
            },
        )
        .unwrap();
        events.retain(|e| !matches!(e.kind, EventKind::SpellCast { .. }));
        let b = burns(&events);
        assert_eq!(
            b.first().map(|b| (b.0, b.1, b.2)),
            Some((0, at(&r, 6, 5), burnt))
        );
    }
    // A berserk walker that dies on its way does not attack.
    let mut s = burning_with(
        &r,
        heroes.clone(),
        &[("fixed", 1, 4, 5), ("fixed", 5, 1, 1)],
        &[("wisp", 20, 8, 5)],
    );
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "berserk"),
            target: CastTarget::Hex(at(&r, 4, 5)),
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert_eq!(burns(&events), [(0, at(&r, 6, 5), 40, 1)]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Struck { .. }))
    );
}

#[test]
fn steps_and_targets_convert() {
    use h3_content::HexStep;

    use crate::Direction;
    let directions: Vec<Direction> = HexStep::ALL
        .iter()
        .map(|&s| Direction::from_step(s))
        .collect();
    assert_eq!(directions, Direction::ALL);
    assert_eq!(CastTarget::Hex(Hex(5)).hex(), Some(Hex(5)));
    assert_eq!(CastTarget::None.hex(), None);
    assert_eq!(stack(1).hex(), None);
}

// covers: BAT-MAG-029
#[test]
fn teleport_moves_an_own_stack_to_a_free_position() {
    let r = ruleset();
    let heroes = [
        Some(caster(&r, &["teleport", "wall", "bolt"], [1, 0])),
        Some(caster(&r, &["firewall"], [0, 1])),
    ];
    let mut s = arena(
        &r,
        heroes,
        &[("fixed", 10, 2, 5), ("wide", 5, 3, 7)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    raise(&mut s, &r, 2, "firewall", (6, 6));
    activate(&mut s, 0);
    let teleport = spell(&r, "teleport");
    let mut out = CastTargets::new();
    // Own stacks only.
    cast_targets(&s, &r, BattleSide::Attacker, teleport, &mut out);
    assert_eq!(out.as_slice(), [stack(0), stack(1)]);
    // Every free position but its own: not occupied, blocked, nor off the field.
    second_targets(&s, &r, BattleSide::Attacker, (teleport, stack(0)), &mut out);
    let field = field(&r);
    let free = (0..187u8)
        .map(Hex)
        .filter(|&h| {
            field.is_standable(h)
                && ![at(&r, 2, 5), at(&r, 3, 7), at(&r, 2, 7), at(&r, 12, 5)].contains(&h)
        })
        .count();
    assert_eq!(out.len(), free);
    for h in [(2, 5), (3, 7), (2, 7), (12, 5), (0, 3)] {
        assert!(!out.as_slice().contains(&CastTarget::Hex(at(&r, h.0, h.1))));
    }
    // The two-hex stack needs room for its tail.
    second_targets(&s, &r, BattleSide::Attacker, (teleport, stack(1)), &mut out);
    assert!(!out.as_slice().contains(&CastTarget::Hex(at(&r, 1, 3))));
    assert!(out.as_slice().contains(&CastTarget::Hex(at(&r, 2, 3))));
    // Onto the fire: no burn (0x5A1B4D enters no hex).
    let go = |to: CastTarget| Command::Cast {
        spell: teleport,
        target: stack(0),
        second: to,
    };
    let mut t = s.clone();
    let events = act(&mut t, &r, go(CastTarget::Hex(at(&r, 6, 5)))).unwrap();
    assert!(events.contains(&Event::public(EventKind::Teleported {
        stack: StackId(0),
        from: at(&r, 2, 5),
        to: at(&r, 6, 5),
    })));
    assert!(burns(&events).is_empty());
    assert_eq!(t.stacks()[0].hex, at(&r, 6, 5));
    assert_eq!(t.active(), Some(StackId(0)));
    // Without a destination, at an occupied one, at an enemy, or a second target for another
    // spell: refused.
    for bad in [
        go(CastTarget::None),
        go(CastTarget::Hex(at(&r, 12, 5))),
        go(CastTarget::Stack(StackId(1))),
        Command::Cast {
            spell: teleport,
            target: stack(2),
            second: CastTarget::Hex(at(&r, 6, 5)),
        },
        Command::Cast {
            spell: spell(&r, "bolt"),
            target: stack(2),
            second: CastTarget::Hex(at(&r, 6, 5)),
        },
    ] {
        assert_eq!(
            act(&mut s.clone(), &r, bad),
            Err(CommandError::InvalidSpellTarget),
            "{bad:?}"
        );
    }
    // Other spells take only none as a second target; nothing for a target that is none of
    // the spell's, for a spell not in the book, or when it is not the side's turn.
    second_targets(
        &s,
        &r,
        BattleSide::Attacker,
        (spell(&r, "bolt"), stack(2)),
        &mut out,
    );
    assert_eq!(out.as_slice(), [CastTarget::None]);
    second_targets(&s, &r, BattleSide::Attacker, (teleport, stack(2)), &mut out);
    assert!(out.is_empty());
    second_targets(
        &s,
        &r,
        BattleSide::Attacker,
        (spell(&r, "haste"), stack(0)),
        &mut out,
    );
    assert!(out.is_empty());
    second_targets(&s, &r, BattleSide::Defender, (teleport, stack(0)), &mut out);
    assert!(out.is_empty());
    assert!(castable_spells(&s, &r, BattleSide::Attacker).contains(teleport));
}

// covers: BAT-MAG-030
#[test]
fn expert_dispel_removes_every_spell_obstacle() {
    let r = ruleset();
    for (mastery, cleared) in [(2, false), (3, true)] {
        let heroes = [
            Some(caster(&r, &["wall", "dispel", "bless"], [mastery, 0])),
            Some(caster(&r, &["firewall"], [0, 1])),
        ];
        let mut s = arena(&r, heroes, &[("fixed", 5, 2, 5)], &[("fixed", 5, 12, 5)], 1);
        raise(&mut s, &r, 0, "wall", (9, 8));
        raise(&mut s, &r, 1, "firewall", (6, 6));
        // A new round lets the attacker's hero cast again; the obstacles stay.
        until_round(&mut s, &r, 2);
        activate(&mut s, 0);
        let bless = spell(&r, "bless");
        let timed = TimedCast {
            id: bless,
            spell: r.spell(bless).unwrap(),
            mastery: 1,
            rounds: 5,
            own: crate::magic::SpellBonus::default(),
        };
        crate::effects::apply(&mut s, &r, timed, StackId(0), &mut Vec::<Event>::new());
        let target = if mastery == 3 {
            CastTarget::None
        } else {
            stack(0)
        };
        let events = act(
            &mut s,
            &r,
            Command::Cast {
                spell: spell(&r, "dispel"),
                target,
                second: CastTarget::None,
            },
        )
        .unwrap();
        let removed = events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::ObstacleRemoved { .. }))
            .count();
        assert_eq!(removed, if cleared { 3 } else { 0 });
        assert_eq!(s.spell_obstacles().is_empty(), cleared);
    }
}

// covers: BAT-MOV-003
#[test]
fn a_walkers_path_is_a_shortest_one_chosen_from_its_end() {
    let r = ruleset();
    let s = battle_with_obstacles(
        &r,
        &[(0, "fixed", 1)],
        &[(0, "fixed", 1)],
        &[(6, 4), (6, 5), (6, 6)],
    );
    let field = field(&r);
    let mut s = s;
    place(&mut s, &r, 0, 4, 5);
    let me = s.stacks()[0];
    // Around the wall of obstacles from (4, 5) to (8, 5).
    let (path, len) = crate::movement::path(&s, field, StackId(0), &me, at(&r, 8, 5));
    let cells: Vec<(i32, i32)> = path[..len].iter().map(|&h| field.coords(h)).collect();
    assert_eq!(cells.len(), 6);
    assert_eq!(cells.last(), Some(&(8, 5)));
    for pair in cells.windows(2) {
        let (a, b) = (at(&r, pair[0].0, pair[0].1), at(&r, pair[1].0, pair[1].1));
        assert_eq!(field.distance(a, b), 1);
    }
    assert!(!cells.contains(&(6, 4)) && !cells.contains(&(6, 5)) && !cells.contains(&(6, 6)));
    // The start, or a position it cannot reach: no path.
    let enclosed = at(&r, 6, 5);
    assert_eq!(
        crate::movement::path(&s, field, StackId(0), &me, enclosed).1,
        0
    );
    assert_eq!(
        crate::movement::path(&s, field, StackId(0), &me, me.hex).1,
        0
    );
}
