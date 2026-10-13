//! Hidden spell obstacles and Quicksand (E04b T5c1): who sees a hidden obstacle (BAT-MAG-031;
//! BAT-QUIRK-1, retired) and Quicksand's pits, which stop a walker (BAT-MAG-032), after the
//! original's code (`docs/formats/installation.md` §7.16).

use std::vec::Vec;

use h3_content_battle::{BattleRuleset, ObstacleKind};
use h3_contracts::{BattleHero, BattleSetup, BattleSide};

use super::spells::{arena, mage, spell};
use super::support::*;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, Hex, HexSet,
    LegalCommands, StackId, Status, Visibility, cast_targets, legal_commands, observe, project,
};

/// A mage with `spells`, the masteries `[air, fire]` and 100 mana.
fn caster(r: &BattleRuleset, spells: &[&str], [air, fire]: [u8; 2]) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.masteries = masteries(r, [air, fire, 0, 0]);
    h.magic.mana = 100;
    h
}

/// The hexes at `cells` as a set.
fn set(r: &BattleRuleset, cells: &[(i32, i32)]) -> HexSet {
    let mut out = HexSet::new();
    for &(x, y) in cells {
        out.insert(at(r, x, y));
    }
    out
}

/// Places `side`'s Quicksand pits on `cells` as its cast would (test setup only; the cast draws
/// its hexes at random); returns the events.
fn pits(
    s: &mut BattleState,
    r: &BattleRuleset,
    side: BattleSide,
    cells: &[(i32, i32)],
) -> Vec<Event> {
    let id = spell(r, "quicksand");
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let hexes: Vec<Hex> = cells.iter().map(|&(x, y)| at(r, x, y)).collect();
    let mut events = Vec::new();
    crate::obstacles::place(
        s,
        (id, ObstacleKind::Pit, &shape),
        side,
        (&hexes, 0),
        &mut events,
    );
    events
}

/// Casts Quicksand for the side of the active stack `id`; returns the events.
fn quicksand(s: &mut BattleState, r: &BattleRuleset, id: u8) -> Vec<Event> {
    activate(s, id);
    act(
        s,
        r,
        Command::Cast {
            spell: spell(r, "quicksand"),
            target: CastTarget::None,
            second: CastTarget::None,
        },
    )
    .unwrap()
}

/// The hexes of the pits `viewer` observes.
fn seen(s: &BattleState, r: &BattleRuleset, viewer: BattleSide) -> HexSet {
    observe(s, r, viewer)
        .spell_obstacles
        .iter()
        .filter(|o| o.kind == ObstacleKind::Pit)
        .fold(HexSet::new(), |set, o| set.union(o.hexes))
}

/// The kinds of the obstacle events, with their visibility, in order.
fn obstacle_events(events: &[Event]) -> Vec<(&'static str, Visibility)> {
    events
        .iter()
        .filter_map(|e| {
            let name = match e.kind {
                EventKind::ObstaclePlaced { .. } => "placed",
                EventKind::ObstacleRemoved { .. } => "removed",
                EventKind::ObstacleRevealed { .. } => "revealed",
                EventKind::ObstacleTriggered { .. } => "triggered",
                _ => return None,
            };
            Some((name, e.visibility))
        })
        .collect()
}

/// The moves `events` report, `(stack, from, to)`.
fn moves(events: &[Event]) -> Vec<(u8, Hex, Hex)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Moved { stack, from, to } => Some((stack.0, from, to)),
            _ => None,
        })
        .collect()
}

/// Whether `events` report a melee strike.
fn struck(events: &[Event]) -> bool {
    events
        .iter()
        .any(|e| matches!(e.kind, EventKind::Struck { .. }))
}

/// The legal commands of the side to act.
fn legal(s: &BattleState, r: &BattleRuleset) -> LegalCommands {
    let mut out = LegalCommands::new();
    legal_commands(s, r, s.side_to_act().unwrap(), &mut out);
    out
}

/// A battle on the plain (Castle's native terrain) with heroes and the stacks placed at `(x, y)`,
/// as [`arena`]; the first attacker stack is active.
fn on_plain(
    r: &BattleRuleset,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
) -> BattleState {
    let specs = |list: &[(&'static str, u32, i32, i32)]| -> Vec<Spec> {
        list.iter()
            .enumerate()
            .map(|(i, &(name, count, ..))| (u8::try_from(i).unwrap(), name, count))
            .collect()
    };
    let setup = BattleSetup {
        heroes,
        terrain: r.terrain_by_key("t:plain"),
        ..setup(r, &specs(attacker), &specs(defender))
    };
    let mut s = BattleState::new(r, &setup, 1).unwrap();
    for (i, &(_, _, x, y)) in attacker.iter().chain(defender).enumerate() {
        place(&mut s, r, u8::try_from(i).unwrap(), x, y);
    }
    activate(&mut s, 0);
    s
}

// covers: BAT-MAG-032
#[test]
fn quicksand_places_random_pits_by_mastery_for_the_whole_battle() {
    let r = ruleset();
    for (mastery, count) in [(0, 4), (1, 4), (2, 6), (3, 8)] {
        let hero = caster(&r, &["quicksand"], [mastery, 0]);
        let mut s = arena(
            &r,
            [Some(hero), None],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 12, 5)],
            u64::from(mastery) + 7,
        );
        s.natural.block(at(&r, 8, 8));
        let events = quicksand(&mut s, &r, 0);
        let list = s.spell_obstacles();
        assert_eq!(list.len(), count, "mastery {mastery}");
        let field = field(&r);
        let mut all = HexSet::new();
        for o in list {
            // One hex each, the caster's, hidden from the other side, for the whole battle.
            assert_eq!(
                (o.kind, o.side, o.hexes.len(), o.rounds, o.value, o.revealed),
                (ObstacleKind::Pit, BattleSide::Attacker, 1, 0, 0, false)
            );
            let hex = o.hexes.iter().next().unwrap();
            assert!(field.is_standable(hex) && !all.contains(hex));
            assert!(![at(&r, 2, 5), at(&r, 12, 5), at(&r, 8, 8)].contains(&hex));
            all.insert(hex);
        }
        // Each placement is told to the caster only.
        assert_eq!(
            obstacle_events(&events),
            std::vec![("placed", Visibility::Side(BattleSide::Attacker)); count]
        );
        // The pits stay through later rounds.
        let mut s2 = s.clone();
        while s2.round() < 4 {
            act(&mut s2, &r, Command::Defend).unwrap();
        }
        assert_eq!(s2.spell_obstacles(), s.spell_obstacles());
    }
}

// covers: BAT-MAG-032
#[test]
fn quicksand_avoids_obstacles_stacks_and_corpses_and_is_cast_without_a_free_hex() {
    let r = ruleset();
    let field = field(&r);
    let hero = caster(&r, &["quicksand"], [3, 0]);
    // Every standable hex is blocked but the stacks', a corpse's and three others.
    let keep = [(1, 5), (15, 5), (8, 2), (5, 5), (6, 5), (7, 5)];
    let keep: Vec<Hex> = keep.iter().map(|&(x, y)| at(&r, x, y)).collect();
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 1, 5), ("fixed", 5, 8, 2)],
        &[("fixed", 5, 15, 5)],
        3,
    );
    for h in (0u8..).take(field.hex_count()).map(Hex) {
        if field.is_standable(h) && !keep.contains(&h) {
            s.natural.block(h);
        }
    }
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    // Expert's eight pits find three hexes.
    quicksand(&mut s, &r, 0);
    assert_eq!(
        s.spell_obstacles()
            .iter()
            .fold(HexSet::new(), |set, o| set.union(o.hexes)),
        set(&r, &[(5, 5), (6, 5), (7, 5)])
    );
    // A pit is no hex for another one: with no free hex left the spell is still cast.
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    let mut out = CastTargets::new();
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "quicksand"),
        &mut out,
    );
    assert_eq!(out.as_slice(), [CastTarget::None]);
    let events = quicksand(&mut s, &r, 0);
    assert!(obstacle_events(&events).is_empty());
    assert_eq!(s.spell_obstacles().len(), 3);
}

/// E04b T11b review (F-r3-control-battlefield-8): the draw of the pits and the mines avoids a
/// hex with a living stack or a corpse, not the hexes of a dead stack that left no corpse. The
/// original's draw skips a hex that holds a living stack or has a corpse on it (the reviewer's
/// reading of 0x5A03E3 to 0x5A03F3 and 0x5A05FB to 0x5A060D); a dead created stack leaves none
/// (BAT-MAG-037), nor does a sacrificed stack (BAT-MAG-036) or a corpse the Pit Lords raised
/// (BAT-MAG-052). The engine kept the hexes of every dead stack free.
// covers: BAT-MAG-032, BAT-MAG-033
#[test]
fn pits_and_mines_may_lie_where_a_dead_stack_left_no_corpse() {
    let r = ruleset();
    let field = field(&r);
    // Every standable hex is blocked but the stacks' and one other; the attacker's second stack,
    // at (8, 2), is dead: a stack of its army, a summoned one, or one whose corpse is gone (a
    // sacrifice, the Pit Lords). The hexes the Expert spell `name` then finds.
    let found = |name: &str, summoned: bool, gone: bool| -> HexSet {
        let hero = caster(&r, &["quicksand", "landmine"], [3, 3]);
        let keep = [(1, 5), (15, 5), (8, 2), (5, 5)];
        let keep: Vec<Hex> = keep.iter().map(|&(x, y)| at(&r, x, y)).collect();
        let mut s = arena(
            &r,
            [Some(hero), None],
            &[("fixed", 5, 1, 5), ("fixed", 5, 8, 2)],
            &[("fixed", 5, 15, 5)],
            3,
        );
        for h in (0u8..).take(field.hex_count()).map(Hex) {
            if field.is_standable(h) && !keep.contains(&h) {
                s.natural.block(h);
            }
        }
        let dead = &mut s.stacks.as_mut_slice()[1];
        dead.traits = dead.traits.with(crate::StackTraits::SUMMONED, summoned);
        dead.count = 0;
        dead.top_hp = 0;
        crate::raise::mark_corpses(&mut s);
        if gone {
            s.raised.corpse[1] = crate::NO_CORPSE;
        }
        let command = Command::Cast {
            spell: spell(&r, name),
            target: CastTarget::None,
            second: CastTarget::None,
        };
        act(&mut s, &r, command).unwrap();
        s.spell_obstacles()
            .iter()
            .fold(HexSet::new(), |set, o| set.union(o.hexes))
    };
    for name in ["quicksand", "landmine"] {
        // A dead stack of the army lies on its hex as a corpse: only the other hex is free.
        assert_eq!(found(name, false, false), set(&r, &[(5, 5)]), "{name}");
        // A dead summoned stack left no corpse: its hex takes a pit or a mine too.
        assert_eq!(
            found(name, true, false),
            set(&r, &[(5, 5), (8, 2)]),
            "{name}"
        );
        // Nor is a hex kept free once its corpse is gone.
        assert_eq!(
            found(name, false, true),
            set(&r, &[(5, 5), (8, 2)]),
            "{name}"
        );
    }
}

// covers: BAT-MAG-031
#[test]
fn hidden_pits_show_only_to_the_sides_that_see_them() {
    let r = ruleset();
    let hero = caster(&r, &["quicksand"], [1, 0]);
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    let events = quicksand(&mut s, &r, 0);
    let all = s
        .spell_obstacles()
        .iter()
        .fold(HexSet::new(), |set, o| set.union(o.hexes));
    assert_eq!(all.len(), 4);
    assert_eq!(seen(&s, &r, BattleSide::Attacker), all);
    assert!(seen(&s, &r, BattleSide::Defender).is_empty());
    for e in &events {
        let placed = matches!(e.kind, EventKind::ObstaclePlaced { .. });
        assert_eq!(project(e, BattleSide::Defender).is_none(), placed);
        assert!(project(e, BattleSide::Attacker).is_some());
    }
}

// covers: BAT-MAG-032
#[test]
fn a_walker_stops_in_an_unseen_pit_and_reveals_it() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    pits(&mut s, &r, BattleSide::Defender, &[(5, 5), (9, 9)]);
    // The pit does not change what the attacker may do.
    assert!(
        legal(&s, &r)
            .iter()
            .any(|c| *c == Command::Move { to: at(&r, 7, 5) })
    );
    let events = act(&mut s, &r, Command::Move { to: at(&r, 7, 5) }).unwrap();
    assert_eq!(moves(&events), [(0, at(&r, 2, 5), at(&r, 5, 5))]);
    assert_eq!(s.stack(StackId(0)).unwrap().hex, at(&r, 5, 5));
    assert_eq!(
        obstacle_events(&events),
        [
            ("revealed", Visibility::Public),
            ("triggered", Visibility::Public)
        ]
    );
    // Now both sides see that pit, not the other one.
    assert_eq!(seen(&s, &r, BattleSide::Attacker), set(&r, &[(5, 5)]));
    assert_eq!(
        seen(&s, &r, BattleSide::Defender),
        set(&r, &[(5, 5), (9, 9)])
    );
    // A stack standing in a pit leaves it freely.
    activate(&mut s, 0);
    act(&mut s, &r, Command::Move { to: at(&r, 10, 5) }).unwrap();
    assert_eq!(s.stack(StackId(0)).unwrap().hex, at(&r, 10, 5));
}

// covers: BAT-MAG-032, BAT-MOV-003
#[test]
fn a_walker_goes_around_a_seen_pit_and_may_stop_in_it() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    pits(&mut s, &r, BattleSide::Attacker, &[(5, 5)]);
    let moves_to = |s: &BattleState, (x, y)| {
        legal(s, &r)
            .iter()
            .any(|c| *c == Command::Move { to: at(&r, x, y) })
    };
    // Around the pit (6, 5) takes 5 steps and (7, 5) 6, beyond the speed; the pit itself 3.
    assert!(moves_to(&s, (5, 5)) && moves_to(&s, (6, 5)) && !moves_to(&s, (7, 5)));
    let mut around = s.clone();
    let events = act(&mut around, &r, Command::Move { to: at(&r, 6, 5) }).unwrap();
    assert!(obstacle_events(&events).is_empty());
    assert_eq!(around.stack(StackId(0)).unwrap().hex, at(&r, 6, 5));
    // Its own pit stops it without revealing it: the other side learns nothing.
    let events = act(&mut s, &r, Command::Move { to: at(&r, 5, 5) }).unwrap();
    assert_eq!(
        obstacle_events(&events),
        [("triggered", Visibility::Side(BattleSide::Attacker))]
    );
    assert!(seen(&s, &r, BattleSide::Defender).is_empty());
    // Flyers ignore pits: one flies over and lands in an unseen one.
    let mut f = arena(
        &r,
        [None, None],
        &[("flyer", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    pits(&mut f, &r, BattleSide::Defender, &[(4, 5), (6, 5)]);
    let events = act(&mut f, &r, Command::Move { to: at(&r, 6, 5) }).unwrap();
    assert!(obstacle_events(&events).is_empty());
    assert_eq!(f.stack(StackId(0)).unwrap().hex, at(&r, 6, 5));
    assert!(seen(&f, &r, BattleSide::Attacker).is_empty());
}

// covers: BAT-MAG-032
#[test]
fn a_two_hex_walker_stops_when_its_rear_hex_enters_a_pit() {
    let r = ruleset();
    // The attacker's position, its rear hex, is left of its front hex: walking left, the rear
    // hex enters (4, 5) first.
    let mut s = arena(
        &r,
        [None, None],
        &[("wide", 5, 5, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    pits(&mut s, &r, BattleSide::Defender, &[(4, 5)]);
    let events = act(&mut s, &r, Command::Move { to: at(&r, 2, 5) }).unwrap();
    assert_eq!(moves(&events), [(0, at(&r, 5, 5), at(&r, 4, 5))]);
    assert_eq!(
        obstacle_events(&events),
        [
            ("revealed", Visibility::Public),
            ("triggered", Visibility::Public)
        ]
    );
}

// covers: BAT-MAG-032
#[test]
fn an_attack_whose_walk_stops_in_a_seen_pit_is_neither_listed_nor_accepted() {
    // E04b T11b review (r3 F-4; the user, BAT-Q30): the original offers no attack from a position
    // with a hex on a pit the stack's side sees (its cursor's attack positions and its
    // pathfinder's attack marking leave them out, `installation.md` §7.16), so such an attack is
    // legal in no profile. Under the flag of BAT-QUIRK-2 (retired) the engine listed and
    // accepted it: the stack walked into the pit and did not strike.
    for profile in ["t:faithful", "t:quirky"] {
        let r = ruleset_with(profile, |_| {});
        // The attack of stack 0 at stack 1 from `from`, a position within its reach where its
        // walk stops: not listed, refused, and the state untouched.
        let refused = |s: &mut BattleState, from: Hex| {
            let reach =
                crate::movement::reachable(s, field(&r), StackId(0), s.stack(StackId(0)).unwrap());
            assert!(reach.can_move_to(from) && reach.stops_at(from), "{profile}");
            let attack = Command::Attack {
                target: StackId(1),
                from,
            };
            assert!(!legal(s, &r).iter().any(|c| *c == attack), "{profile}");
            // Other positions next to the target stay legal, and so does the move into the pit.
            assert!(legal(s, &r).iter().any(|c| matches!(
                *c,
                Command::Attack { target: StackId(1), from: other } if other != from
            )));
            assert!(
                legal(s, &r)
                    .iter()
                    .any(|c| *c == Command::Move { to: from })
            );
            let before = s.clone();
            assert_eq!(
                act(s, &r, attack),
                Err(CommandError::Unreachable),
                "{profile}"
            );
            assert_eq!(*s, before, "{profile}");
        };
        let mut s = arena(
            &r,
            [None, None],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 6, 5)],
            1,
        );
        pits(&mut s, &r, BattleSide::Attacker, &[(5, 5)]);
        refused(&mut s, at(&r, 5, 5));
        // A two-hex walker: the position whose rear hex lies on the pit, its front hex next to
        // the target.
        let mut s = arena(
            &r,
            [None, None],
            &[("wide", 5, 4, 4)],
            &[("fixed", 5, 8, 5)],
            1,
        );
        pits(&mut s, &r, BattleSide::Attacker, &[(6, 5)]);
        refused(&mut s, at(&r, 6, 5));
        // The hexes a stack starts on never stop it: standing in the pit, it attacks from there.
        let mut s = arena(
            &r,
            [None, None],
            &[("fixed", 5, 5, 5)],
            &[("fixed", 5, 6, 5)],
            1,
        );
        pits(&mut s, &r, BattleSide::Attacker, &[(5, 5)]);
        let attack = Command::Attack {
            target: StackId(1),
            from: at(&r, 5, 5),
        };
        assert!(legal(&s, &r).iter().any(|c| *c == attack), "{profile}");
        assert!(struck(&act(&mut s, &r, attack).unwrap()), "{profile}");
        // No pit stops a flyer: it lands on the pit and strikes.
        let mut s = arena(
            &r,
            [None, None],
            &[("flyer", 5, 2, 5)],
            &[("fixed", 5, 6, 5)],
            1,
        );
        pits(&mut s, &r, BattleSide::Attacker, &[(5, 5)]);
        assert!(legal(&s, &r).iter().any(|c| *c == attack), "{profile}");
        let events = act(&mut s, &r, attack).unwrap();
        assert!(
            struck(&events) && obstacle_events(&events).is_empty(),
            "{profile}"
        );
        assert_eq!(s.stack(StackId(0)).unwrap().hex, at(&r, 5, 5));
    }
}

// covers: BAT-MAG-032
#[test]
fn a_move_and_attack_into_an_unseen_pit_stops_without_the_attack() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 6, 5)],
        1,
    );
    pits(&mut s, &r, BattleSide::Defender, &[(5, 5)]);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 5, 5),
        },
    )
    .unwrap();
    assert!(!struck(&events));
    assert_eq!(s.stack(StackId(0)).unwrap().hex, at(&r, 5, 5));
    assert_eq!(seen(&s, &r, BattleSide::Attacker), set(&r, &[(5, 5)]));
}

// covers: BAT-MAG-031
#[test]
fn natives_at_the_start_reveal_the_other_sides_pits() {
    // The original's timing in every profile (BAT-QUIRK-1 retired, BAT-Q34; 0x46402C, 0x5A0442).
    let r = ruleset();
    let hero = caster(&r, &["quicksand"], [1, 0]);
    // Castle's Pikemen fight on their native plain for the defender.
    let mut s = on_plain(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 2, 5)],
        &[("pikeman_c", 5, 12, 5), ("fixed", 5, 12, 7)],
    );
    let events = quicksand(&mut s, &r, 0);
    assert_eq!(
        obstacle_events(&events),
        std::vec![("placed", Visibility::Public); 4]
    );
    assert!(s.spell_obstacles().iter().all(|o| o.revealed));
    // The natives die: what was placed stays revealed, and so is the next cast.
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    quicksand(&mut s, &r, 0);
    assert_eq!(s.spell_obstacles().len(), 8);
    assert!(s.spell_obstacles().iter().all(|o| o.revealed));
    assert_eq!(seen(&s, &r, BattleSide::Defender).len(), 8);
    // The attacker had no natives: the defender's pits stay hidden from it.
    let mut d = s.clone();
    pits(&mut d, &r, BattleSide::Defender, &[(3, 9)]);
    assert!(!seen(&d, &r, BattleSide::Attacker).contains(at(&r, 3, 9)));
    assert!(!d.spell_obstacles()[8].revealed);
}

// covers: BAT-MAG-031
#[test]
fn without_natives_at_the_start_the_other_sides_pits_stay_hidden() {
    let r = ruleset();
    let hero = caster(&r, &["quicksand", "dispel"], [3, 0]);
    // No defender stack fights on its native terrain.
    let mut s = on_plain(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5), ("fixed", 5, 12, 7)],
    );
    let pit = at(&r, 10, 5);
    pits(&mut s, &r, BattleSide::Attacker, &[(10, 5)]);
    let events = quicksand(&mut s, &r, 0);
    // Told to the caster's side only, and not seen by the other.
    assert_eq!(
        obstacle_events(&events),
        std::vec![("placed", Visibility::Side(BattleSide::Attacker)); 8]
    );
    assert!(s.spell_obstacles().iter().all(|o| !o.revealed));
    assert!(seen(&s, &r, BattleSide::Defender).is_empty());
    // Its walkers do not stop there.
    let reach = |s: &BattleState| {
        crate::movement::reachable(s, field(&r), StackId(2), s.stack(StackId(2)).unwrap())
    };
    assert!(!reach(&s).stops_at(pit));
    // Expert Dispel removes them, told to the attacker only.
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(&mut s, 0);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "dispel"),
            target: CastTarget::None,
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert!(s.spell_obstacles().is_empty());
    assert_eq!(
        obstacle_events(&events),
        std::vec![("removed", Visibility::Side(BattleSide::Attacker)); 9]
    );
}

// covers: BAT-MAG-031, BAT-MAG-027
#[test]
fn a_wall_or_fire_may_go_over_a_pit_its_caster_does_not_see() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [
            Some(caster(&r, &["firewall"], [0, 1])),
            Some(caster(&r, &["firewall", "wall"], [1, 1])),
        ],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    pits(&mut s, &r, BattleSide::Attacker, &[(8, 5)]);
    let targets = |s: &BattleState, side, name| {
        let mut out = CastTargets::new();
        cast_targets(s, &r, side, spell(&r, name), &mut out);
        out.as_slice().contains(&CastTarget::Hex(at(&r, 8, 5)))
    };
    // The original refuses the hex, which gives the pit away (Q-0002): here only its owner sees
    // it there.
    assert!(!targets(&s, BattleSide::Attacker, "firewall"));
    activate(&mut s, 1);
    assert!(targets(&s, BattleSide::Defender, "firewall"));
    assert!(targets(&s, BattleSide::Defender, "wall"));
    act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "firewall"),
            target: CastTarget::Hex(at(&r, 8, 5)),
            second: CastTarget::None,
        },
    )
    .unwrap();
    let kinds: Vec<ObstacleKind> = s
        .spell_obstacles()
        .iter()
        .filter(|o| o.hexes.contains(at(&r, 8, 5)))
        .map(|o| o.kind)
        .collect();
    assert_eq!(kinds, [ObstacleKind::Pit, ObstacleKind::Fire]);
    // The defender sees the fire, still not the pit.
    assert!(seen(&s, &r, BattleSide::Defender).is_empty());
}

// covers: BAT-MAG-031, BAT-MAG-032
#[test]
fn a_hypnotized_walker_uses_its_controllers_view() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 9)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    // The defender's own pit on the hypnotized stack's way, unseen by its controller.
    pits(&mut s, &r, BattleSide::Defender, &[(9, 5)]);
    let st = &mut s.stacks.as_mut_slice()[1];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    activate(&mut s, 1);
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    let events = act(&mut s, &r, Command::Move { to: at(&r, 8, 5) }).unwrap();
    assert_eq!(s.stack(StackId(1)).unwrap().hex, at(&r, 9, 5));
    assert_eq!(
        obstacle_events(&events),
        [
            ("revealed", Visibility::Public),
            ("triggered", Visibility::Public)
        ]
    );
}
