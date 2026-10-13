//! Fire Shield (BAT-MAG-018), Frenzy (BAT-MAG-019) and the Behemoths' reduction under quirk
//! Q-0005 (BAT-DMG-006), E04b T3b2.

use std::vec;
use std::vec::Vec;

use h3_content::SpellScope;
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleSetup};

use super::spells::{mage, spell};
use super::support::*;
use crate::damage::frenzied;
use crate::effects::TimedCast;
use crate::{BattleState, CastTarget, Command, Event, EventKind, StackId};

/// Gives stack `id` the timed effect of `name` cast at `mastery` for five rounds (Frenzy: by
/// its own turns).
fn give(s: &mut BattleState, r: &BattleRuleset, id: u8, name: &str, mastery: u8) {
    let id_of = spell(r, name);
    let cast = TimedCast {
        id: id_of,
        spell: r.spell(id_of).unwrap(),
        mastery,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// A battle of `attacker` against `defender` with the stacks at `positions` (stack-id order),
/// the first attacker stack active.
fn field_of(
    r: &BattleRuleset,
    attacker: &[Spec],
    defender: &[Spec],
    positions: &[(i32, i32)],
) -> BattleState {
    let mut s = battle(r, attacker, defender, 1);
    for (id, &(x, y)) in (0u8..).zip(positions) {
        place(&mut s, r, id, x, y);
    }
    activate(&mut s, 0);
    s
}

/// The active stack attacks `target` from `(x, y)`.
fn hit(s: &mut BattleState, r: &BattleRuleset, target: u8, (x, y): (i32, i32)) -> Vec<Event> {
    let from = at(r, x, y);
    act(
        s,
        r,
        Command::Attack {
            target: StackId(target),
            from,
        },
    )
    .unwrap()
}

/// `(attacker, target, damage)` of every strike in `events`.
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

/// `(stack, damage, kills)` of every reflection in `events`.
fn reflected(events: &[Event]) -> Vec<(u8, u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Reflected {
                stack,
                damage,
                kills,
            } => Some((stack.0, damage, kills)),
            _ => None,
        })
        .collect()
}

/// The reflection of 100 Fixed (stage 1: 200) striking a lone Tank (100 hit points) whose
/// shield was cast at `mastery`, in profile `profile`.
fn tank_reflects(profile: &str, mastery: u8) -> Vec<(u8, u64, u32)> {
    let r = ruleset_with(profile, |_| {});
    let mut s = field_of(
        &r,
        &[(0, "fixed", 100)],
        &[(0, "tank", 1)],
        &[(7, 5), (8, 5)],
    );
    give(&mut s, &r, 1, "fireshield", mastery);
    reflected(&hit(&mut s, &r, 1, (7, 5)))
}

// covers: BAT-MAG-018
#[test]
fn fire_shield_reflects_a_share_of_the_first_stage_capped_at_the_targets_health_with_quirk_q0010() {
    // Stage 1 is 200 (the attack is below the defense); the 150 damage kill the Tank, whose
    // 100 hit points cap the base with or without quirk Q-0010: 20 % of 100, two Fixed. Single
    // precision changes nothing here (f32(0.2) is slightly above 0.2).
    assert_eq!(tank_reflects("t:faithful", 1), [(0, 20, 2)]);
    assert_eq!(tank_reflects("t:single", 1), [(0, 20, 2)]);
    assert_eq!(tank_reflects("t:faithful", 3), [(0, 30, 3)]);
    assert_eq!(tank_reflects("t:fireshield", 1), [(0, 20, 2)]);
    // The first stage in the executable's rounding (quirk Q-0005 too).
    assert_eq!(tank_reflects("t:original", 1), [(0, 20, 2)]);
    // 10 Fixed (stage 1: 20, dealt: 15) on a shielded Tank: with quirk Q-0010 4 back (20 % of
    // the first stage), without it 3 (of the damage dealt); batch 2b R3: 60 of 300, not 30.
    let tank = |r: &BattleRuleset| {
        let mut s = field_of(r, &[(0, "fixed", 10)], &[(0, "tank", 1)], &[(7, 5), (8, 5)]);
        give(&mut s, r, 1, "fireshield", 1);
        let events = hit(&mut s, r, 1, (7, 5));
        (s, events)
    };
    assert_eq!(reflected(&tank(&ruleset()).1), [(0, 3, 0)]);
    let r = ruleset_with("t:fireshield", |_| {});
    let (s, events) = tank(&r);
    assert_eq!(strikes(&events)[0], (0, 1, 15));
    // The reflection comes before the Tank's retaliation.
    assert_eq!(reflected(&events), [(0, 4, 0)]);
    let reflection = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::Reflected { .. }));
    let retaliation = events
        .iter()
        .rposition(|e| matches!(e.kind, EventKind::Struck { .. }));
    assert!(reflection < retaliation);
    // 4 back and 1 from the retaliation.
    assert_eq!(s.stacks()[0].top_hp, 5);
    // A retaliation on a shielded stack is reflected too: the shielded attacker kills 2, the
    // 8 left strike back for 16, 3 of which come back.
    let mut s = field_of(
        &r,
        &[(0, "fixed", 10)],
        &[(0, "fixed", 10)],
        &[(7, 5), (8, 5)],
    );
    give(&mut s, &r, 0, "fireshield", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strikes(&events), [(0, 1, 20), (1, 0, 16)]);
    assert_eq!(reflected(&events), [(1, 3, 0)]);
}

// covers: BAT-MAG-018
#[test]
fn shots_and_fire_immune_strikers_are_not_burnt() {
    let r = ruleset();
    let mut s = field_of(
        &r,
        &[(0, "archer", 10)],
        &[(0, "fixed", 10)],
        &[(3, 5), (12, 5)],
    );
    give(&mut s, &r, 1, "fireshield", 3);
    let shot = Command::Shoot {
        target: StackId(1),
        at: s.stacks()[1].hex,
    };
    let events = act(&mut s, &r, shot).unwrap();
    assert!(!strikes(&events).is_empty());
    assert!(reflected(&events).is_empty());
    let mut s = field_of(
        &r,
        &[(0, "salamander", 10)],
        &[(0, "tank", 1)],
        &[(7, 5), (8, 5)],
    );
    give(&mut s, &r, 1, "fireshield", 3);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert!(!strikes(&events).is_empty());
    assert!(reflected(&events).is_empty());
}

// covers: BAT-MAG-018
#[test]
fn a_breath_burns_on_its_target_only_a_multi_headed_striker_on_every_stack() {
    let r = ruleset();
    // The drake (position 6,5) attacks 2; its target and the stack behind it (1) are shielded:
    // only 2 reflects.
    let mut s = field_of(
        &r,
        &[(0, "drake", 10), (1, "serf", 10)],
        &[(0, "fixed", 10)],
        &[(6, 5), (9, 5), (8, 5)],
    );
    give(&mut s, &r, 1, "fireshield", 1);
    give(&mut s, &r, 2, "fireshield", 1);
    let events = hit(&mut s, &r, 2, (6, 5));
    assert_eq!(reflected(&events), [(0, 4, 0)]);
    // A hydra striking three shielded stacks takes back 4 from each, at once.
    let mut s = field_of(
        &r,
        &[(0, "hydra", 10), (1, "serf", 10)],
        &[
            (0, "fixed", 10),
            (1, "fixed", 10),
            (2, "fixed", 10),
            (3, "fixed", 10),
        ],
        &[(6, 5), (7, 6), (8, 5), (5, 5), (6, 4), (10, 5)],
    );
    for id in 2..6 {
        give(&mut s, &r, id, "fireshield", 1);
    }
    let events = hit(&mut s, &r, 2, (6, 5));
    assert_eq!(strikes(&events).len(), 3);
    assert_eq!(reflected(&events), [(0, 12, 1)]);
}

// covers: BAT-MAG-018, BAT-MAG-004, BAT-MAG-013
#[test]
fn the_reflection_is_the_shields_spell_damage() {
    let r = ruleset_with("t:fireshield", |_| {});
    // The struck stack's hero adds 50 % to its spells (its attack and defense are 0).
    let mut h = hero(&r);
    (h.attack, h.defense) = (0, 0);
    h.effects = vec![BattleEffect::SpellDamage {
        scope: SpellScope::All,
        rate: Rate::bp(5000),
    }];
    let burnt = |striker: &'static str, ward: bool| {
        let setup = BattleSetup {
            heroes: [None, Some(h.clone())],
            ..setup(&r, &[(0, striker, 10)], &[(0, "tank", 1)])
        };
        let mut s = BattleState::new(&r, &setup, 1).unwrap();
        place(&mut s, &r, 0, 7, 5);
        place(&mut s, &r, 1, 8, 5);
        activate(&mut s, 0);
        give(&mut s, &r, 1, "fireshield", 1);
        if ward {
            give(&mut s, &r, 0, "fireward", 1);
        }
        reflected(&hit(&mut s, &r, 1, (7, 5)))
    };
    // 4 × 1.5 = 6; a golem keeps 25 % of a spell's damage (1); Protection from Fire 70 % (4).
    assert_eq!(burnt("fixed", false), [(0, 6, 0)]);
    assert_eq!(burnt("golem", false), [(0, 1, 0)]);
    assert_eq!(burnt("fixed", true), [(0, 4, 0)]);
}

// covers: BAT-MAG-018, BAT-MAG-004
#[test]
fn a_fire_shield_reflects_a_huge_strike_without_overflow() {
    use h3_content::{CreatureSelector, CreatureStat};
    use h3_contracts::{BattleSide, StatOp};
    // The robustness review's test for F-robust-3 without any spell damage bonus: the value
    // itself reaches 2^48 when a Fire Shield reflects the damage of a huge strike on a huge
    // stack (`magic::reflection` passes `first × v %` to `spell_damage`): `u32::MAX` creatures
    // whose heroes raise their damage and their hit points with sound `StackStat` effects.
    // With quirk Q-0005 that overflowed `u128` (a panic).
    let r = ruleset_with("t:single", |_| {});
    // `sultan`: an innate Fire Shield of 25 %, 100 hit points.
    let mut start = setup(&r, &[(0, "flyer", u32::MAX)], &[(0, "sultan", u32::MAX)]);
    let stat = |stat, amount| BattleEffect::StackStat {
        creatures: CreatureSelector::All,
        stat,
        op: StatOp::Add,
        amount,
    };
    let mut striker = hero(&r);
    striker.attack = 255;
    striker.damage.melee = Rate::bp(10_000);
    striker.effects = vec![
        stat(CreatureStat::Speed, 30),
        stat(CreatureStat::MinDamage, i16::MAX),
        stat(CreatureStat::MinDamage, i16::MAX),
        stat(CreatureStat::MaxDamage, i16::MAX),
        stat(CreatureStat::MaxDamage, i16::MAX),
    ];
    let mut shielded = hero(&r);
    shielded.effects = vec![stat(CreatureStat::HitPoints, i16::MAX); 40];
    start.heroes = [Some(striker), Some(shielded)];
    let mut state = BattleState::new(&r, &start, 1).unwrap();
    assert_eq!(state.side_to_act(), Some(BattleSide::Attacker));
    let mut legal = crate::LegalCommands::new();
    crate::legal_commands(&state, &r, BattleSide::Attacker, &mut legal);
    let attack = legal
        .iter()
        .copied()
        .find(|c| matches!(c, Command::Attack { .. }))
        .unwrap();
    // The listed attack is applied without a panic, and the shield burns the striker.
    let events = act(&mut state, &r, attack).unwrap();
    assert_eq!(reflected(&events).len(), 1);
}

/// The damage of 10 Fixed, frenzied at `mastery` (none: not), on a lone `target`, in
/// `profile`.
fn frenzied_strike(profile: &str, mastery: Option<u8>, target: &'static str) -> u64 {
    let r = ruleset_with(profile, |_| {});
    let mut s = field_of(
        &r,
        &[(0, "fixed", 10)],
        &[(0, target, 1)],
        &[(7, 5), (8, 5)],
    );
    if let Some(m) = mastery {
        give(&mut s, &r, 0, "frenzy", m);
    }
    strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2
}

// covers: BAT-MAG-019
#[test]
fn frenzy_adds_the_defense_to_the_attack_and_leaves_none_against_strikes() {
    // Attack 10 and defense 10 against the Tank's 20: 15 unfrenzied, attack 20 at Basic (20),
    // 25 at Advanced (+25 %: 25), 30 at Expert (+50 %: 30).
    assert_eq!(frenzied_strike("t:faithful", None, "tank"), 15);
    assert_eq!(frenzied_strike("t:faithful", Some(1), "tank"), 20);
    assert_eq!(frenzied_strike("t:faithful", Some(2), "tank"), 25);
    assert_eq!(frenzied_strike("t:faithful", Some(3), "tank"), 30);
    // A frenzied Tank has no defense: 10 against 0, +50 %.
    let r = ruleset();
    let mut s = field_of(
        &r,
        &[(0, "fixed", 10)],
        &[(0, "tank", 1)],
        &[(7, 5), (8, 5)],
    );
    give(&mut s, &r, 1, "frenzy", 1);
    assert_eq!(strikes(&hit(&mut s, &r, 1, (7, 5)))[0].2, 30);
    // The share is rounded toward 0 with the attack, single precision with quirk Q-0005.
    assert_eq!(frenzied(10, -3, 150, false), 5);
    assert_eq!(frenzied(10, -3, 150, true), 5);
    assert_eq!(frenzied(0, 10, 70, false), 7);
    assert_eq!(frenzied(0, 10, 70, true), 6);
}

// covers: BAT-MAG-019, BAT-DMG-006
#[test]
fn frenzy_against_a_behemoth_counts_its_reduction_with_quirk_q0009() {
    // Against a Behemoth (defense 10): the frenzied attack is 10 + 10 = 20 (+50 %: 30); with
    // Q-0009 the Behemoth's reduction lowers the defense added: 10 + 6 = 16 (+30 %: 26).
    assert_eq!(frenzied_strike("t:faithful", Some(1), "behemoth"), 30);
    assert_eq!(frenzied_strike("t:frenzy", Some(1), "behemoth"), 26);
    // With quirk Q-0005 too the reduction is the executable's (10 − 4.00000006 = 5.99 → 5;
    // review r2b F-1): 10 + 5 = 15 (+25 %: 25).
    assert_eq!(frenzied_strike("t:frenzy_single", Some(1), "behemoth"), 25);
    // Other abilities change nothing: 20 against 10.
    assert_eq!(frenzied_strike("t:frenzy", Some(1), "pikeman"), 30);
}

// covers: BAT-MAG-019
#[test]
fn a_frenzied_stack_that_waits_is_counted_once_a_round() {
    // E04b T11b review (r2b F-6): Frenzy is counted at the stack's first turn start of a round,
    // not at its turn after waiting, through real commands. A hero without attack and defense
    // casts it (no mastery: 100 %) on its Nagas, whose strikes are never retaliated: ten of them
    // (attack 10, defense 10, damage 2) deal a Titan (defense 20) 15, frenzied 20. Ids: 0 the
    // Nagas beside the Titan, 1 Nagas far away, 2 the Titan (the fastest).
    let r = ruleset();
    let start = || {
        let mut hero = mage(&r, &["frenzy"]);
        (hero.attack, hero.defense) = (0, 0);
        let setup = BattleSetup {
            heroes: [Some(hero), None],
            ..setup(&r, &[(0, "naga", 10), (1, "naga", 10)], &[(0, "titan", 1)])
        };
        let mut s = BattleState::new(&r, &setup, 1).unwrap();
        for (id, (x, y)) in [(0, (7, 5)), (1, (3, 8)), (2, (8, 5))] {
            place(&mut s, &r, id, x, y);
        }
        s
    };
    // Stack `id`, whose turn it is, does `command`.
    let turn = |s: &mut BattleState, id: u8, command: Command| {
        assert_eq!(s.active(), Some(StackId(id)));
        act(s, &r, command).unwrap()
    };
    let frenzy = Command::Cast {
        spell: spell(&r, "frenzy"),
        target: CastTarget::Stack(StackId(0)),
        second: CastTarget::None,
    };
    let strike = Command::Attack {
        target: StackId(2),
        from: at(&r, 7, 5),
    };
    // The turns Frenzy has left on stack 0.
    let left = |s: &BattleState| {
        s.effects()
            .iter()
            .find(|e| e.stack == StackId(0))
            .map(|e| e.rounds)
    };
    // Cast on the active stack: one turn. The Nagas wait and strike in the wait phase, still
    // frenzied: the turn after waiting takes nothing.
    let mut s = start();
    turn(&mut s, 2, Command::Wait);
    turn(&mut s, 0, frenzy);
    assert_eq!(left(&s), Some(Some(1)));
    turn(&mut s, 0, Command::Wait);
    turn(&mut s, 1, Command::Defend);
    assert_eq!(strikes(&turn(&mut s, 0, strike)), [(0, 2, 20)]);
    assert_eq!(left(&s), Some(Some(1)));
    turn(&mut s, 2, Command::Defend);
    // Round 2: their first turn start ends it.
    turn(&mut s, 2, Command::Wait);
    assert_eq!((s.round(), left(&s)), (2, None));
    assert_eq!(strikes(&turn(&mut s, 0, strike)), [(0, 2, 15)]);
    // Cast on a stack that has waited (in the other Nagas' turn): two turns. It strikes frenzied
    // in the wait phase, which takes nothing, ...
    let mut s = start();
    turn(&mut s, 2, Command::Wait);
    turn(&mut s, 0, Command::Wait);
    turn(&mut s, 1, frenzy);
    assert_eq!(left(&s), Some(Some(2)));
    turn(&mut s, 1, Command::Defend);
    assert_eq!(strikes(&turn(&mut s, 0, strike)), [(0, 2, 20)]);
    assert_eq!(left(&s), Some(Some(2)));
    turn(&mut s, 2, Command::Defend);
    // ... and again in its next turn, whose start takes one; ...
    turn(&mut s, 2, Command::Wait);
    assert_eq!((s.round(), left(&s)), (2, Some(Some(1))));
    assert_eq!(strikes(&turn(&mut s, 0, strike)), [(0, 2, 20)]);
    turn(&mut s, 1, Command::Defend);
    turn(&mut s, 2, Command::Defend);
    // ... the turn start after that ends it.
    turn(&mut s, 2, Command::Wait);
    assert_eq!((s.round(), left(&s)), (3, None));
    assert_eq!(strikes(&turn(&mut s, 0, strike)), [(0, 2, 15)]);
}

// covers: BAT-MAG-019
#[test]
fn frenzy_lasts_through_the_stacks_next_turn() {
    let r = ruleset();
    let mut s = field_of(
        &r,
        &[(0, "fixed", 10), (1, "fixed", 10)],
        &[(0, "fixed", 10)],
        &[(3, 5), (3, 7), (13, 5)],
    );
    // On the active stack 1 turn, on another 2, whatever the power; the round start leaves
    // them, it counts only the others (Haste) down.
    give(&mut s, &r, 0, "frenzy", 1);
    give(&mut s, &r, 1, "frenzy", 1);
    give(&mut s, &r, 1, "haste", 1);
    give(&mut s, &r, 1, "ray", 1);
    let rounds = |s: &BattleState, id: u8, name: &str| {
        s.effects()
            .iter()
            .find(|e| e.stack == StackId(id) && e.spell == spell(&r, name))
            .map(|e| e.rounds)
    };
    assert_eq!(rounds(&s, 0, "frenzy"), Some(Some(1)));
    assert_eq!(rounds(&s, 1, "frenzy"), Some(Some(2)));
    let mut events = Vec::new();
    crate::turn::start_round(&mut s, &r, &mut events);
    assert_eq!(rounds(&s, 1, "frenzy"), Some(Some(2)));
    assert_eq!(rounds(&s, 1, "haste"), Some(Some(4)));
    // The stack's own turn counts it down; the last ends it.
    crate::effects::own_turn(&mut s, &r, StackId(1), &mut events);
    assert_eq!(rounds(&s, 1, "frenzy"), Some(Some(1)));
    assert_eq!(rounds(&s, 1, "haste"), Some(Some(4)));
    crate::effects::own_turn(&mut s, &r, StackId(1), &mut events);
    assert_eq!(rounds(&s, 1, "frenzy"), None);
    assert_eq!(s.stacks()[1].spells.frenzy, 0);
    assert!(events.iter().any(|e| matches!(
        e.kind,
        EventKind::EffectEnded {
            target: StackId(1),
            ..
        }
    )));
    // In play: the active stack defends, the enemy too; at its next turn its Frenzy ends.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(s.stacks()[0].spells.frenzy > 0, "{events:?}");
    while s.active() != Some(StackId(0)) {
        act(&mut s, &r, Command::Defend).unwrap();
    }
    assert_eq!(rounds(&s, 0, "frenzy"), None);
    assert_eq!(s.stacks()[0].spells.frenzy, 0);
}
