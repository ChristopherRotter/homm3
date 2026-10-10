//! The Phoenix's rebirth (E04b T7c2b; BAT-MAG-059, quirk BAT-QUIRK-17), after the original's
//! code (`docs/formats/installation.md` §7.21, "The Phoenix's rebirth").

use std::collections::BTreeSet;
use std::vec;
use std::vec::Vec;

use h3_content::{ObstacleKind, Ruleset};
use h3_contracts::{BattleHero, BattleResult, BattleSide};

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::damage::apply_damage;
use crate::effects;
use crate::raise::{mark_corpses, rebirths};
use crate::state::StackFlags;
use crate::{
    BattleState, CastTarget, Command, Event, EventKind, Hex, NO_CORPSE, StackId, StackTraits,
};

/// The rebirths in `events` as `(stack, creatures)`.
fn reborn(events: &[Event]) -> Vec<(u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Reborn { stack, creatures } => Some((stack.0, creatures)),
            _ => None,
        })
        .collect()
}

/// The active stack attacks stack `target` from where it stands.
fn strike(s: &mut BattleState, r: &Ruleset, target: u8) -> Vec<Event> {
    let from = s.stacks()[usize::from(s.active().unwrap().0)].hex;
    let target = StackId(target);
    act(s, r, Command::Attack { target, from }).unwrap()
}

/// Kills stack `id` outright, as a strike would, without the rebirth check.
fn kill(s: &mut BattleState, r: &Ruleset, id: u8) {
    let total = s.stacks()[usize::from(id)].total_hp();
    apply_damage(s.at_mut(StackId(id)), total);
    effects::damaged(s, r, StackId(id), &mut Vec::<Event>::new());
}

/// A mage with `spells`, every school at `mastery`, power 3 and 100 mana.
fn caster(r: &Ruleset, spells: &[&str], mastery: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.masteries = masteries(r, [mastery; 4]);
    h.magic.mana = 100;
    h
}

// covers: BAT-MAG-059, BAT-END-001
#[test]
fn a_phoenix_killed_by_a_strike_comes_back_with_a_fifth_having_acted() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "fixed", 60)], &[(0, "phoenix", 10)], 1);
    place(&mut s, &r, 0, 4, 5);
    place(&mut s, &r, 1, 5, 5);
    activate(&mut s, 0);
    // 120 damage kill all ten: ten div 5 come back, none left over to draw for.
    let events = strike(&mut s, &r, 1);
    assert_eq!(reborn(&events), vec![(1, 2)]);
    // The rebirth comes before the end's check: the side's only stack is back.
    assert_eq!(s.result(), None);
    let phoenix = s.stacks()[1];
    assert_eq!(
        (phoenix.count, phoenix.top_hp, phoenix.hex),
        (2, 10, at(&r, 5, 5))
    );
    assert_eq!((s.casts_left(StackId(1)), s.corpse(StackId(1))), (0, 0));
    // It has acted this round: a new round starts, where it acts first.
    assert_eq!((s.round(), s.active()), (2, Some(StackId(1))));
    // Its one cast is used: killed again, it stays dead.
    advance_to(&mut s, &r, 0);
    let events = strike(&mut s, &r, 1);
    assert!(reborn(&events).is_empty());
    assert_eq!(
        s.result(),
        Some(BattleResult::Victory(BattleSide::Attacker))
    );
    // The outcome counts the creatures that came back as the stack's own (BAT-END-003).
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.stacks[1][0].end, 0);
}

// covers: BAT-MAG-059
#[test]
fn each_creature_left_over_comes_back_with_a_one_in_divisor_chance() {
    let r = ruleset();
    // Seven phoenixes: one for sure and two draws at 1 in 5; two firebirds: two draws at 1 in 3.
    let mut phoenixes = BTreeSet::new();
    let mut firebirds = BTreeSet::new();
    for seed in 0..200 {
        let mut s = battle(
            &r,
            &[(0, "phoenix", 7), (1, "firebird", 2)],
            &[(0, "fixed", 1)],
            seed,
        );
        kill(&mut s, &r, 0);
        kill(&mut s, &r, 1);
        let mut events = Vec::new();
        rebirths(&mut s, &r, &mut events);
        let back = |id: u8| s.stacks()[usize::from(id)].count;
        phoenixes.insert(back(0));
        firebirds.insert(back(1));
        assert_eq!(
            reborn(&events).len(),
            [back(0), back(1)].iter().filter(|&&n| n > 0).count()
        );
        // Every rebirth uses a cast, whether a creature comes back or not.
        assert_eq!((s.casts_left(StackId(0)), s.casts_left(StackId(1))), (0, 1));
        // A firebird that drew none has not acted; the next check takes its second cast.
        if back(1) == 0 {
            assert!(!s.stacks()[1].flags.contains(StackFlags::ACTED));
            rebirths(&mut s, &r, &mut events);
            assert_eq!(s.casts_left(StackId(1)), 0);
        }
    }
    assert_eq!(phoenixes, BTreeSet::from([1, 2, 3]));
    assert_eq!(firebirds, BTreeSet::from([0, 1, 2]));
}

// covers: BAT-MAG-059, BAT-MOR-003
#[test]
fn a_phoenix_killed_by_the_retaliation_keeps_its_good_morale_turn() {
    let r = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.good_morale_chances = vec![255, 255, 255];
    });
    // Neutral phoenixes alone: +1 morale; the retaliation's 120 kill them all.
    let mut s = battle(&r, &[(0, "phoenix", 5)], &[(0, "fixed", 60)], 1);
    place(&mut s, &r, 0, 4, 5);
    place(&mut s, &r, 1, 5, 5);
    activate(&mut s, 0);
    let events = strike(&mut s, &r, 1);
    assert_eq!(reborn(&events), vec![(0, 1)]);
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    // Good morale clears the acted mark of the rebirth, as the original's roll clears its done
    // bit: the reborn phoenix waits and acts again in the wait phase.
    assert_eq!(s.active(), Some(StackId(0)));
    act(&mut s, &r, Command::Wait).unwrap();
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!((s.round(), s.active()), (1, Some(StackId(0))));
}

// covers: BAT-MAG-059
#[test]
fn a_damage_spell_a_shot_and_a_mine_bring_the_rebirth_too() {
    let r = ruleset();
    // A hero's Expert bolt: 3 × 10 + 30 = 60 damage on five phoenixes.
    let mut s = arena(
        &r,
        [Some(caster(&r, &["bolt"], 3)), None],
        &[("fixed", 1, 2, 5)],
        &[("phoenix", 5, 14, 5)],
        1,
    );
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "bolt"),
            target: stack(1),
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert_eq!(reborn(&events), vec![(1, 1)]);
    assert_eq!(s.result(), None);
    // Forty archers' shot does at least 40 × 2 × 0.9 = 72 on five phoenixes.
    let mut s = battle(&r, &[(0, "archer", 40)], &[(0, "phoenix", 5)], 3);
    place(&mut s, &r, 0, 2, 5);
    place(&mut s, &r, 1, 8, 5);
    activate(&mut s, 0);
    let hex = s.stacks()[1].hex;
    let events = act(
        &mut s,
        &r,
        Command::Shoot {
            target: StackId(1),
            at: hex,
        },
    )
    .unwrap();
    assert_eq!(reborn(&events), vec![(1, 1)]);
    // A flying firebird lands on an enemy mine of 40: three of 10 hit points die, one comes back
    // (three div 3), and the battle goes on.
    let mut s = battle(&r, &[(0, "firebird", 3)], &[(0, "fixed", 1)], 1);
    place(&mut s, &r, 0, 2, 5);
    place(&mut s, &r, 1, 14, 5);
    activate(&mut s, 0);
    let mine = spell(&r, "landmine");
    let shape = r.spell(mine).unwrap().magic.obstacle.clone().unwrap();
    let hexes: Vec<Hex> = vec![at(&r, 6, 5)];
    crate::obstacles::place(
        &mut s,
        (mine, ObstacleKind::Mine, &shape),
        BattleSide::Defender,
        (&hexes, 40),
        &mut Vec::new(),
    );
    let events = act(&mut s, &r, Command::Move { to: at(&r, 6, 5) }).unwrap();
    assert_eq!(reborn(&events), vec![(0, 1)]);
    assert_eq!((s.result(), s.stacks()[0].hex), (None, at(&r, 6, 5)));
}

// covers: BAT-MAG-059, BAT-MAG-010
#[test]
fn a_reborn_phoenix_comes_back_without_the_effects_it_died_with() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "phoenix", 10)], 1);
    // A hasted, berserk phoenix dies; the command's end has not removed its effects yet.
    for name in ["haste", "berserk"] {
        let id = spell(&r, name);
        let cast = effects::TimedCast {
            id,
            spell: r.spell(id).unwrap(),
            mastery: 1,
            rounds: 5,
            own: crate::magic::SpellBonus::default(),
        };
        effects::apply(&mut s, &r, cast, StackId(1), &mut Vec::<Event>::new());
    }
    let speed = s.stacks()[1].speed;
    assert!(s.stacks()[1].speed_now() > speed);
    kill(&mut s, &r, 1);
    assert_eq!(s.effects().len(), 2);
    let mut events = Vec::new();
    rebirths(&mut s, &r, &mut events);
    assert_eq!(reborn(&events), vec![(1, 2)]);
    let phoenix = s.stacks()[1];
    assert_eq!(phoenix.spells.status, effects::Status::default());
    assert_eq!(phoenix.speed_now(), speed);
    assert!(s.effects().is_empty());
}

// covers: BAT-MAG-059, BAT-MAG-004
#[test]
fn an_active_phoenix_its_heros_spell_killed_comes_back_having_acted() {
    let r = ruleset();
    // Its own hero's Expert storm (60 on every stack) kills the five active phoenixes: one comes
    // back and keeps the turn (it is alive), but has acted: after a wait it does not act again.
    let mut s = arena(
        &r,
        [Some(caster(&r, &["storm"], 3)), None],
        &[("phoenix", 5, 2, 5)],
        &[("fixed", 60, 14, 5)],
        1,
    );
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "storm"),
            target: CastTarget::None,
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert_eq!(reborn(&events), vec![(0, 1)]);
    assert_eq!(s.active(), Some(StackId(0)));
    act(&mut s, &r, Command::Wait).unwrap();
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
}

// covers: BAT-MAG-059
#[test]
fn clones_and_sacrificed_phoenixes_are_never_reborn() {
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "phoenix", 10), (1, "phoenix", 10)],
        &[(0, "fixed", 1)],
        1,
    );
    // A clone (stack 1 made one) dies for good.
    s.stacks.as_mut_slice()[1].traits = s.stacks()[1].traits.with(StackTraits::CLONE, true);
    kill(&mut s, &r, 1);
    // A sacrificed stack left no corpse (BAT-MAG-036): with quirk BAT-QUIRK-17 fixed, it is
    // not reborn by any later check.
    kill(&mut s, &r, 0);
    s.raised.corpse[0] = NO_CORPSE;
    let mut events = Vec::new();
    rebirths(&mut s, &r, &mut events);
    assert!(reborn(&events).is_empty());
    assert_eq!((s.casts_left(StackId(0)), s.casts_left(StackId(1))), (1, 1));
    // Nor is a stack that died before this action and lies as a corpse.
    s.raised.corpse[0] = 0;
    mark_corpses(&mut s);
    rebirths(&mut s, &r, &mut events);
    assert!(reborn(&events).is_empty());
    assert_eq!(s.casts_left(StackId(0)), 1);
}

// covers: BAT-MAG-059, BAT-MAG-036
#[test]
fn a_sacrificed_phoenix_stays_dead_after_the_next_strike() {
    let r = ruleset();
    let mut hero = mage(&r, &["offering"]);
    hero.magic.masteries = masteries(&r, [0, 1, 0, 0]);
    hero.magic.mana = 100;
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 4, 5), ("serf", 2, 2, 3), ("phoenix", 10, 2, 7)],
        &[("fixed", 60, 5, 5)],
        1,
    );
    kill(&mut s, &r, 1);
    mark_corpses(&mut s);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "offering"),
            target: stack(1),
            second: stack(2),
        },
    )
    .unwrap();
    assert!(events.iter().any(|e| e.kind
        == EventKind::Sacrificed {
            victim: StackId(2),
            creatures: 10
        }));
    // The active stack strikes: the check after the strike leaves the phoenixes dead.
    let events = strike(&mut s, &r, 3);
    assert!(reborn(&events).is_empty());
    assert_eq!((s.stacks()[2].count, s.casts_left(StackId(2))), (0, 1));
}
