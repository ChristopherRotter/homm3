//! Stacks that lose actions (E04b T4b): Forgetfulness (BAT-MAG-023) and Blind (BAT-MAG-024),
//! after the original's code (`docs/formats/installation.md` §7.14).

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleSide};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::state::StackFlags;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, LegalCommands,
    StackId, Status, cast_targets, legal_commands,
};

/// Gives stack `id` the timed effect of `name` at `mastery` for five rounds.
fn give(s: &mut BattleState, r: &BattleRuleset, (id, name): (u8, &str), mastery: u8) {
    let spell_id = spell(r, name);
    let cast = TimedCast {
        id: spell_id,
        spell: r.spell(spell_id).unwrap(),
        mastery,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The strikes `events` report, as `(attacker, target, damage)`.
fn strikes(events: &[Event]) -> Vec<(u8, u8, u64)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker,
                target,
                damage,
                ..
            } => Some((attacker.0, target.0, damage)),
            _ => None,
        })
        .collect()
}

/// Whether `events` report that `name`'s effect on stack `id` ended.
fn ended(r: &BattleRuleset, events: &[Event], id: u8, name: &str) -> bool {
    events.iter().any(|e| {
        e.kind
            == EventKind::EffectEnded {
                target: StackId(id),
                spell: spell(r, name),
            }
    })
}

/// A mage with `spells`, the air mastery `mastery` and enough mana.
fn adept(r: &BattleRuleset, spells: &[&str], mastery: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.masteries = masteries(r, [mastery, 0, 0, 0]);
    h.magic.mana = 200;
    h
}

/// The damage of one shot (`melee` false) or melee attack of `count` blessed archers (which
/// deal 3 each) on `fixed`, with Forgetfulness cast at `forget` (`None`: none), and of the
/// retaliation it draws.
fn archers(r: &BattleRuleset, count: u32, forget: Option<u8>, melee: bool) -> Vec<(u8, u8, u64)> {
    let x = if melee { 3 } else { 9 };
    let mut s = arena(
        r,
        [None, None],
        &[("archer", count, 2, 5)],
        &[("fixed", 50, x, 5)],
        7,
    );
    give(&mut s, r, (0, "bless"), 1);
    if let Some(mastery) = forget {
        give(&mut s, r, (0, "forget"), mastery);
    }
    let command = if melee {
        Command::Attack {
            target: StackId(1),
            from: at(r, 2, 5),
        }
    } else {
        Command::Shoot {
            target: StackId(1),
            at: at(r, x, 5),
        }
    };
    if melee {
        // Next to an enemy the archers cannot shoot: they strike in melee.
        assert_eq!(
            act(
                &mut s,
                r,
                Command::Shoot {
                    target: StackId(1),
                    at: at(r, x, 5),
                }
            ),
            Err(CommandError::CannotShoot)
        );
    }
    strikes(&act(&mut s, r, command).unwrap())
}

// covers: BAT-MAG-023
#[test]
fn forgetfulness_halves_the_creatures_counted_on_every_strike() {
    let r = ruleset();
    // A shot: 10 forgetful archers deal what 5 deal, 7 what 3 deal, and 1 what 1 deals (half
    // of the creatures rounded down, at least one), at any mastery.
    for (count, half) in [(10, 5), (7, 3), (1, 1)] {
        for mastery in [0, 1] {
            let forgetful = archers(&r, count, Some(mastery), false);
            assert_eq!(forgetful, archers(&r, half, None, false), "{count}");
        }
    }
    assert_ne!(
        archers(&r, 10, Some(1), false),
        archers(&r, 10, None, false)
    );
    // Melee, at Advanced too (survey S-09: the original's rule in every profile), with the
    // shooter's melee penalty: the archers' strike is halved; `fixed` retaliates in full.
    for mastery in [1, 2] {
        let forgetful = archers(&r, 10, Some(mastery), true);
        let half = archers(&r, 5, None, true);
        assert_eq!(forgetful[0], half[0]);
        assert_eq!(forgetful.len(), 2);
    }
    // A forgetful archer's retaliation is halved as well.
    let retaliation = |forget: bool| {
        let mut s = arena(
            &r,
            [None, None],
            &[("fixed", 5, 2, 5)],
            &[("archer", 10, 3, 5)],
            7,
        );
        give(&mut s, &r, (1, "bless"), 1);
        if forget {
            give(&mut s, &r, (1, "forget"), 1);
        }
        let attack = Command::Attack {
            target: StackId(1),
            from: at(&r, 2, 5),
        };
        strikes(&act(&mut s, &r, attack).unwrap())[1]
    };
    assert!(retaliation(true).2 < retaliation(false).2);
}

// covers: BAT-MAG-023
#[test]
fn forgetfulness_from_advanced_keeps_shooters_from_shooting() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("archer", 10, 2, 5), ("archer", 10, 2, 7)],
        &[("fixed", 5, 6, 5)],
        1,
    );
    give(&mut s, &r, (0, "forget"), 2);
    // A Basic cast first keeps its mastery: the stack still shoots after an Advanced one.
    give(&mut s, &r, (1, "forget"), 1);
    give(&mut s, &r, (1, "forget"), 3);
    assert!(s.stacks()[0].spells.status.contains(Status::NO_SHOOTING));
    assert!(!s.stacks()[1].spells.status.contains(Status::NO_SHOOTING));
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Attacker, &mut legal);
    assert!(legal.iter().all(|c| !matches!(c, Command::Shoot { .. })));
    assert!(legal.iter().any(|c| matches!(
        c,
        Command::Attack {
            target: StackId(2),
            ..
        }
    )));
    let shoot = Command::Shoot {
        target: StackId(2),
        at: at(&r, 6, 5),
    };
    assert_eq!(act(&mut s, &r, shoot), Err(CommandError::CannotShoot));
    activate(&mut s, 1);
    assert!(act(&mut s, &r, shoot).is_ok());
}

// covers: BAT-MAG-023
#[test]
fn forgetfulness_reaches_enemy_shooters_and_is_mass_from_advanced_with_its_quirk() {
    let targets = |r: &BattleRuleset, mastery: u8| {
        let s = arena(
            r,
            [Some(adept(r, &["forget"], mastery)), None],
            &[("fixed", 5, 2, 5)],
            &[("archer", 10, 9, 5), ("fixed", 5, 9, 7), ("lich", 3, 9, 9)],
            1,
        );
        let mut out = CastTargets::new();
        cast_targets(&s, r, BattleSide::Attacker, spell(r, "forget"), &mut out);
        out.as_slice().to_vec()
    };
    // Only shooters, whatever their side's loyalties to the living (the lich is undead).
    let fixed = ruleset();
    assert_eq!(targets(&fixed, 1), [stack(1), stack(3)]);
    // Without quirk Q-0011 the Advanced cast still takes a target; the mass version is Expert's.
    assert_eq!(targets(&fixed, 2), [stack(1), stack(3)]);
    assert_eq!(targets(&fixed, 3), [CastTarget::None]);
    // The original's: mass from Advanced, reaching every enemy shooter.
    let early = ruleset_with("t:early", |_| {});
    assert_eq!(targets(&early, 2), [CastTarget::None]);
    let mut s = arena(
        &early,
        [Some(adept(&early, &["forget"], 2)), None],
        &[("archer", 10, 2, 5)],
        &[("archer", 10, 9, 5), ("fixed", 5, 9, 7), ("lich", 3, 9, 9)],
        1,
    );
    act(&mut s, &early, cast(&early, "forget", CastTarget::None)).unwrap();
    let forgetful: Vec<bool> = s
        .stacks()
        .iter()
        .map(|st| st.spells.status.contains(Status::NO_SHOOTING))
        .collect();
    assert_eq!(forgetful, [false, true, false, true]);
}

// covers: BAT-MAG-024
#[test]
fn a_blind_stack_passes_its_turns_until_damage_ends_it() {
    let r = ruleset();
    // fast (9), the blind flyer (7), the archers (4).
    let mut s = arena(
        &r,
        [None, None],
        &[("fast", 5, 2, 5), ("archer", 10, 2, 7)],
        &[("flyer", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (2, "blind"), 1);
    let mut events = act(&mut s, &r, Command::Defend).unwrap();
    // The flyer's turn starts and passes; the archers act next.
    assert!(events.contains(&Event::public(EventKind::TurnStarted { stack: StackId(2) })));
    assert!(events.contains(&Event::public(EventKind::TurnPassed { stack: StackId(2) })));
    assert_eq!(s.active(), Some(StackId(1)));
    assert!(s.stacks()[2].flags.contains(StackFlags::STARTED));
    assert!(!s.stacks()[2].flags.contains(StackFlags::ACTED));
    // A shot ends the blindness without a retaliation; the flyer then acts in this round.
    let shoot = Command::Shoot {
        target: StackId(2),
        at: at(&r, 12, 5),
    };
    events = act(&mut s, &r, shoot).unwrap();
    assert!(ended(&r, &events, 2, "blind"));
    assert_eq!(strikes(&events).len(), 1);
    assert!(!s.stacks()[2].spells.status.contains(Status::NO_TURNS));
    assert_eq!(s.active(), Some(StackId(2)));
    // Its turn already started: its turn-start rules do not run again.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Regenerated { .. }))
    );
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    // Every round it stays blind its turn passes; Blind runs out with its rounds.
    let mut s = arena(
        &r,
        [None, None],
        &[("fast", 5, 2, 5)],
        &[("flyer", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (1, "blind"), 1);
    let turns = order(&mut s, &r, 7);
    assert_eq!(turns, [0, 0, 0, 0, 0, 0, 1]);
}

// covers: BAT-MAG-024
#[test]
fn a_blind_stack_retaliates_at_its_share_and_not_at_all_at_expert() {
    for profile in ["t:faithful", "t:single"] {
        let r = ruleset_with(profile, |_| {});
        let retaliation = |mastery: Option<u8>| {
            let mut s = arena(
                &r,
                [None, None],
                &[("fixed", 10, 2, 5)],
                &[("serf", 10, 3, 5)],
                1,
            );
            if let Some(m) = mastery {
                give(&mut s, &r, (1, "blind"), m);
            }
            let attack = Command::Attack {
                target: StackId(1),
                from: at(&r, 2, 5),
            };
            let events = act(&mut s, &r, attack).unwrap();
            if mastery.is_some() {
                assert!(ended(&r, &events, 1, "blind"));
            }
            (strikes(&events), s.stacks()[1].retaliations_left)
        };
        // 10 strike for 20 and kill 2; 8 strike back for 16.
        assert_eq!(retaliation(None).0, [(0, 1, 20), (1, 0, 16)], "{profile}");
        assert_eq!(retaliation(Some(1)).0, [(0, 1, 20), (1, 0, 8)], "{profile}");
        assert_eq!(retaliation(Some(2)).0, [(0, 1, 20), (1, 0, 4)], "{profile}");
        // At Expert none, and the retaliation is kept.
        assert_eq!(retaliation(Some(3)), (vec![(0, 1, 20)], 1), "{profile}");
    }
}

// covers: BAT-MAG-024
#[test]
fn blind_ends_with_spell_damage_spares_the_undead_and_targets_enemies() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["blind", "bolt"], 1)), None],
        &[("fixed", 5, 2, 5)],
        &[("serf", 10, 9, 5), ("vampire", 5, 9, 7)],
        1,
    );
    let mut out = CastTargets::new();
    cast_targets(&s, &r, BattleSide::Attacker, spell(&r, "blind"), &mut out);
    assert_eq!(out.as_slice(), [stack(1)]);
    act(&mut s, &r, cast(&r, "blind", stack(1))).unwrap();
    assert!(s.stacks()[1].spells.status.contains(Status::NO_TURNS));
    // Next round the hero's damage spell ends it.
    crate::turn::start_round(&mut s, &r, &mut Vec::<Event>::new());
    activate(&mut s, 0);
    let events = act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    assert!(ended(&r, &events, 1, "blind"));
    assert!(!s.stacks()[1].spells.status.contains(Status::NO_TURNS));
}
