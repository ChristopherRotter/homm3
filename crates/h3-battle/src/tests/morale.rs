//! Tests of morale and luck: values (BAT-MOR-001/002, BAT-LCK-001), good and bad morale
//! (BAT-MOR-003/004, wait-morale phase of BAT-TRN-002) and lucky strikes (BAT-LCK-002, quirk
//! Q-0003).
//!
//! The default test ruleset never triggers morale or luck; these tests use `ruleset_with` and a
//! die on which they (nearly) always trigger or the original 24-sided die for the chances.

use std::vec;
use std::vec::Vec;

use h3_content_battle::{BattleParamsDef, BattleRuleset};

use super::support::*;
use crate::event::StrikeFlags;
use crate::morale::{Bearer, Stat};
use crate::{BattleSetup, BattleState, Command, Event, EventKind, StackFlags, StackId};

/// The original's morale and luck chances (out of 24).
fn original(b: &mut BattleParamsDef) {
    b.good_morale_chances = vec![1, 2, 3];
    b.bad_morale_chances = vec![2, 4, 6];
    b.good_luck_chances = vec![1, 2, 3];
}

/// Good morale and luck that always trigger, and bad morale that nearly always does (254/255: a
/// certain freeze is not allowed, it could stop the battle for good).
fn always(b: &mut BattleParamsDef) {
    b.morale_dice = 255;
    b.luck_dice = 1;
    b.good_morale_chances = vec![255, 255, 255];
    b.bad_morale_chances = vec![254, 254, 254];
    b.good_luck_chances = vec![1, 1, 1];
}

/// The `(morale, luck)` of every stack.
fn values(s: &BattleState) -> Vec<(i8, i8)> {
    s.stacks().iter().map(|s| (s.morale, s.luck)).collect()
}

fn morale_of(r: &BattleRuleset, attacker: &[Spec]) -> Vec<i8> {
    let s = battle(r, attacker, &[(0, "fixed", 1)], 1);
    s.stacks()[..attacker.len()]
        .iter()
        .map(|s| s.morale)
        .collect()
}

fn count(events: &[Event], f: fn(&EventKind) -> bool) -> usize {
    events.iter().filter(|e| f(&e.kind)).count()
}

fn good(k: &EventKind) -> bool {
    matches!(k, EventKind::GoodMorale { .. })
}

fn bad(k: &EventKind) -> bool {
    matches!(k, EventKind::BadMorale { .. })
}

fn move_to(s: &mut BattleState, r: &BattleRuleset, x: i32, y: i32) -> Vec<Event> {
    let to = at(r, x, y);
    act(s, r, Command::Move { to }).unwrap()
}

fn stack(s: &mut BattleState, id: u8) -> &mut crate::Stack {
    &mut s.stacks.as_mut_slice()[usize::from(id)]
}

// ---------------------------------------------------------------- BAT-MOR-002 army composition

// covers: BAT-MOR-002
#[test]
fn one_alignment_gives_plus_one_and_every_further_one_minus_one() {
    let r = ruleset();
    // Neutral creatures form one alignment.
    assert_eq!(morale_of(&r, &[(0, "serf", 1)]), vec![1]);
    assert_eq!(
        morale_of(&r, &[(0, "pikeman_c", 1), (1, "angel", 1)])[0],
        2,
        "castle only (+1), plus the Angels' aura (+1)"
    );
    assert_eq!(
        morale_of(&r, &[(0, "pikeman_c", 1), (1, "imp", 1)]),
        vec![0, 0]
    );
    assert_eq!(
        morale_of(&r, &[(0, "pikeman_c", 1), (1, "imp", 1), (2, "serf", 1)]),
        vec![-1, -1, -1]
    );
}

// covers: BAT-MOR-002
#[test]
fn undead_in_the_army_cost_the_living_one_morale() {
    let r = ruleset();
    // Neutral + necropolis: two alignments (0), undead (−1); the skeletons have no morale.
    assert_eq!(
        morale_of(&r, &[(0, "serf", 1), (1, "skeleton", 1)]),
        vec![-1, 0]
    );
    // Four alignments (−2) and undead (−1).
    assert_eq!(
        morale_of(
            &r,
            &[
                (0, "pikeman_c", 1),
                (1, "imp", 1),
                (2, "serf", 1),
                (3, "skeleton", 1)
            ]
        ),
        vec![-3, -3, -3, 0]
    );
    let s = battle(
        &r,
        &[(0, "serf", 1), (1, "skeleton", 1)],
        &[(0, "fixed", 1)],
        1,
    );
    let attacker = Bearer::Side(crate::BattleSide::Attacker);
    let army: Vec<i32> = s
        .modifiers
        .iter()
        .filter(|m| m.bearer == attacker)
        .map(|m| m.amount)
        .collect();
    assert_eq!(army, vec![0, -1]);
}

// ---------------------------------------------------------------- BAT-MOR-001 morale value

// covers: BAT-MOR-001
#[test]
fn equal_auras_do_not_stack_and_reach_the_other_side() {
    let r = ruleset();
    // Angels and Archangels: one +1. The enemy's Bone Dragons: −1 on this side.
    let s = battle(
        &r,
        &[(0, "angel", 1), (1, "archangel", 1), (2, "pikeman_c", 1)],
        &[(0, "bone_dragon", 1), (1, "skeleton", 1)],
        1,
    );
    assert_eq!(values(&s), vec![(1, 0), (1, 0), (1, 0), (0, 0), (0, 0)]);
    let morale_auras = s
        .modifiers
        .iter()
        .filter(|m| m.stat == Stat::Morale && m.amount != 0)
        .count();
    // Attacker: castle (+1), Angels (+1, once), Bone Dragons (−1); defender: necropolis (+1),
    // undead (−1).
    assert_eq!(morale_auras, 5);
}

// covers: BAT-MOR-001
#[test]
fn auras_persist_after_their_bearers_die() {
    let r = ruleset_with("t:faithful", always);
    let mut s = battle(
        &r,
        &[(0, "angel", 1), (1, "pikeman_c", 1)],
        &[(0, "fixed", 50)],
        1,
    );
    place(&mut s, &r, 0, 7, 5);
    place(&mut s, &r, 2, 8, 5);
    activate(&mut s, 0);
    // The Angel dies in the retaliation; the pikemen keep +2.
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 7, 5),
        },
    )
    .unwrap();
    assert!(!s.stacks()[0].is_alive(), "{events:?}");
    assert_eq!(s.stacks()[1].morale, 2);
}

// covers: BAT-MOR-001
#[test]
fn minimums_raise_and_ranges_clamp_morale() {
    let r = ruleset();
    // Three alignments (−1), but Minotaurs keep at least +1.
    assert_eq!(
        morale_of(
            &r,
            &[(0, "minotaur", 1), (1, "pikeman_c", 1), (2, "imp", 1)]
        ),
        vec![1, -1, -1]
    );
    // With one level of good and bad morale, +2 and −3 clamp to ±1.
    let r = ruleset_with("t:faithful", |b| {
        b.good_morale_chances = vec![0];
        b.bad_morale_chances = vec![0];
    });
    assert_eq!(
        morale_of(&r, &[(0, "pikeman_c", 1), (1, "angel", 1)]),
        vec![1, 1]
    );
    assert_eq!(
        morale_of(
            &r,
            &[
                (0, "pikeman_c", 1),
                (1, "imp", 1),
                (2, "serf", 1),
                (3, "skeleton", 1)
            ]
        ),
        vec![-1, -1, -1, 0]
    );
}

// ---------------------------------------------------------------- BAT-LCK-001 luck value

// covers: BAT-LCK-001
#[test]
fn luck_comes_from_auras_and_minimums_and_is_never_negative() {
    let r = ruleset();
    let s = battle(
        &r,
        &[(0, "serf", 1), (1, "halfling", 1), (2, "lucky", 1)],
        &[(0, "devil", 1)],
        1,
    );
    // Devils: −1 for the enemy, which counts as 0; Halflings keep +1, the lucky ones +3.
    let luck: Vec<i8> = s.stacks().iter().map(|s| s.luck).collect();
    assert_eq!(luck, vec![0, 1, 3, 0]);
}

// ---------------------------------------------------------------- BAT-LCK-002 lucky strikes

/// The strikes `(attacker, target, damage, flags)` in `events`.
fn strikes(events: &[Event]) -> Vec<(u8, u8, u64, StrikeFlags)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker,
                target,
                damage,
                flags,
                ..
            } => Some((attacker.0, target.0, damage, flags)),
            _ => None,
        })
        .collect()
}

fn attack_from_7_5(s: &mut BattleState, r: &BattleRuleset, target: u8) -> Vec<Event> {
    let from = at(r, 7, 5);
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

// covers: BAT-LCK-002
#[test]
fn lucky_strikes_add_the_base_damage_again_on_the_target_only() {
    let l = StrikeFlags::LUCKY;
    let r = ruleset_with("t:faithful", always);
    // 10 Halflings (luck 1): 20 → 40. The retaliation of the fixed stack has no luck.
    let mut s = battle(&r, &[(0, "halfling", 10)], &[(0, "fixed", 10)], 1);
    place(&mut s, &r, 0, 7, 5);
    place(&mut s, &r, 1, 8, 5);
    let events = attack_from_7_5(&mut s, &r, 1);
    assert_eq!(
        &strikes(&events)[..2],
        &[(0, 1, 40, l), (1, 0, 12, StrikeFlags::RETALIATION)]
    );
    // A breath attack: luck only on the attacked stack (Q18).
    let mut s = battle(
        &r,
        &[(0, "drake", 10), (1, "serf", 10)],
        &[(0, "fixed", 10)],
        1,
    );
    // The drake's position (6, 5), its front hex (7, 5).
    place(&mut s, &r, 0, 6, 5);
    place(&mut s, &r, 1, 9, 5);
    place(&mut s, &r, 2, 8, 5);
    stack(&mut s, 0).luck = 1;
    let from = at(&r, 6, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from,
        },
    )
    .unwrap();
    assert_eq!(
        &strikes(&events)[..2],
        &[(0, 2, 40, l), (0, 1, 20, StrikeFlags::SECONDARY)]
    );
    // Luck and a death blow add up: 20 · (1 + 1 + 1) = 60.
    let mut s = battle(&r, &[(0, "doom", 10)], &[(0, "fixed", 10)], 1);
    place(&mut s, &r, 0, 7, 5);
    place(&mut s, &r, 1, 8, 5);
    stack(&mut s, 0).luck = 1;
    let events = attack_from_7_5(&mut s, &r, 1);
    assert_eq!(
        strikes(&events)[0],
        (0, 1, 60, l.with(StrikeFlags::DEATH_BLOW))
    );
}

// covers: BAT-LCK-002
#[test]
fn without_a_hero_there_is_no_luck_under_quirk_q0003() {
    let lucky_first = |profile: &str, led: [bool; 2]| {
        let r = ruleset_with(profile, always);
        let setup = BattleSetup {
            heroes: led.map(|led| led.then(|| hero(&r))),
            ..setup(&r, &[(0, "halfling", 10)], &[(0, "fixed", 10)])
        };
        let mut s = BattleState::new(&r, &setup, 1).unwrap();
        place(&mut s, &r, 0, 7, 5);
        place(&mut s, &r, 1, 8, 5);
        strikes(&attack_from_7_5(&mut s, &r, 1))[0]
            .3
            .contains(StrikeFlags::LUCKY)
    };
    assert!(lucky_first("t:faithful", [false, false]));
    assert!(!lucky_first("t:quirky", [false, true]));
    assert!(lucky_first("t:quirky", [true, false]));
}

// covers: BAT-LCK-002
#[test]
fn lucky_strikes_happen_with_their_chance() {
    let r = ruleset_with("t:faithful", original);
    let lucky = (0..400u64)
        .filter(|&seed| {
            let mut s = battle(&r, &[(0, "lucky", 10)], &[(0, "fixed", 10)], seed);
            place(&mut s, &r, 0, 7, 5);
            place(&mut s, &r, 1, 8, 5);
            strikes(&attack_from_7_5(&mut s, &r, 1))[0]
                .3
                .contains(StrikeFlags::LUCKY)
        })
        .count();
    // Luck +3: 3/24 = 12.5 % of 400 = 50 (σ ≈ 6.6).
    assert!((30..=70).contains(&lucky), "{lucky}");
}

// ---------------------------------------------------------------- BAT-MOR-003 good morale

// covers: BAT-MOR-003
#[test]
fn good_morale_gives_one_more_turn_per_round_after_acting() {
    let r = ruleset_with("t:faithful", always);
    // Castle (+1) against two alignments (0).
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 5), (1, "pikeman_c", 5)],
        &[(0, "imp", 1), (1, "serf", 1)],
        1,
    );
    let events = move_to(&mut s, &r, 3, 2);
    assert_eq!(count(&events, good), 1);
    assert_eq!(
        events.last().unwrap().kind,
        EventKind::TurnStarted { stack: StackId(0) }
    );
    assert_eq!(s.active(), Some(StackId(0)));
    assert!(s.stacks()[0].flags.contains(StackFlags::HAD_MORALE));
    // The second turn brings no further one.
    let events = move_to(&mut s, &r, 4, 2);
    assert_eq!(count(&events, good), 0);
    assert_ne!(s.active(), Some(StackId(0)));
    // Defending gets none; a new round allows it again.
    advance_to(&mut s, &r, 1);
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(count(&events, good), 0);
    advance_to(&mut s, &r, 0);
    assert!(!s.stacks()[0].flags.contains(StackFlags::HAD_MORALE));
    assert_eq!(count(&move_to(&mut s, &r, 5, 2), good), 1);
}

// covers: BAT-MOR-003, BAT-TRN-002
#[test]
fn waiting_during_a_morale_turn_acts_before_the_other_waiters() {
    let r = ruleset_with("t:faithful", always);
    // 0 = castle pikemen (speed 5, morale +1); 1 = imp (5), 2 = slow (2): two alignments.
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 5)],
        &[(0, "imp", 1), (1, "slow", 1)],
        1,
    );
    move_to(&mut s, &r, 3, 5);
    assert_eq!(s.active(), Some(StackId(0)));
    act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(s.active(), Some(StackId(1)));
    act(&mut s, &r, Command::Wait).unwrap();
    act(&mut s, &r, Command::Wait).unwrap();
    // Wait phases: the morale waiter first, then the others slowest first (defending brings no
    // good morale).
    assert_eq!(order(&mut s, &r, 3), vec![0, 2, 1]);
    assert_eq!(s.round(), 2);
}

// covers: BAT-MOR-003
#[test]
fn acting_in_the_wait_phase_can_still_bring_good_morale() {
    let r = ruleset_with("t:faithful", always);
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 5)],
        &[(0, "imp", 1), (1, "serf", 1)],
        1,
    );
    act(&mut s, &r, Command::Wait).unwrap();
    advance_to(&mut s, &r, 0);
    assert!(s.stacks()[0].flags.contains(StackFlags::WAITED));
    assert_eq!(count(&move_to(&mut s, &r, 3, 5), good), 1);
    // The extra turn cannot wait again.
    assert_eq!(
        act(&mut s, &r, Command::Wait),
        Err(crate::CommandError::AlreadyWaited)
    );
}

// covers: BAT-MOR-003
#[test]
fn good_morale_happens_with_its_chance() {
    let r = ruleset_with("t:faithful", original);
    let lucky = (0..400u64)
        .filter(|&seed| {
            let mut s = battle(&r, &[(0, "pikeman_c", 5)], &[(0, "fixed", 1)], seed);
            stack(&mut s, 0).morale = 3;
            count(&move_to(&mut s, &r, 3, 5), good) == 1
        })
        .count();
    // Morale +3: 3/24 = 12.5 % of 400 = 50 (σ ≈ 6.6).
    assert!((30..=70).contains(&lucky), "{lucky}");
}

// ---------------------------------------------------------------- BAT-MOR-004 bad morale

// covers: BAT-MOR-004
#[test]
fn bad_morale_freezes_a_stack_at_its_activation() {
    let r = ruleset_with("t:faithful", always);
    // Attacker: three alignments (−1). Its first stack is the battle's first: the original's
    // battle start picks it without the freezing checks (0x462A20).
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 1), (1, "imp", 1), (2, "serf", 1)],
        &[(0, "fixed", 1)],
        1,
    );
    assert_eq!(s.active(), Some(StackId(0)));
    let mut start = Vec::new();
    let setup = setup(
        &r,
        &[(0, "pikeman_c", 1), (1, "imp", 1), (2, "serf", 1)],
        &[(0, "fixed", 1)],
    );
    let same = BattleState::start(&r, &setup, 1, &mut start).unwrap();
    assert_eq!(same, s);
    let kinds: Vec<EventKind> = start.iter().map(|e| e.kind).collect();
    assert_eq!(kinds, vec![EventKind::TurnStarted { stack: StackId(0) }]);
    // Then the defender (the side that did not act last); after its turn the other two freeze,
    // each with an event, and a new round starts.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!((count(&events, bad), s.active()), (0, Some(StackId(3))));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(count(&events, bad), 2);
    assert_eq!((s.round(), s.active()), (2, Some(StackId(3))));
    // From round 2 the first stack freezes too.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(count(&events, bad), 3);
    assert_eq!(s.round(), 3);
}

// covers: BAT-MOR-003, BAT-MAG-024
#[test]
fn a_stack_its_action_blinded_passes_its_good_morale_turn_until_the_blindness_ends() {
    let r = ruleset_with("t:faithful", always);
    // Castle pikemen (+1) strike a blinder, whose retaliation blinds them; the defender's two
    // alignments give it no morale.
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 5)],
        &[(0, "blinder", 5), (1, "imp", 5)],
        1,
    );
    place(&mut s, &r, 0, 4, 5);
    place(&mut s, &r, 1, 5, 5);
    place(&mut s, &r, 2, 3, 5);
    activate(&mut s, 0);
    let from = at(&r, 4, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from,
        },
    )
    .unwrap();
    // Good morale, but no turn: the original's roll clears the done bit and its pick passes a
    // blind stack over (0x465222).
    assert_eq!(count(&events, good), 1);
    assert!(
        !events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(0) })
    );
    let flags = s.stacks()[0].flags;
    assert!(!flags.contains(StackFlags::ACTED) && flags.contains(StackFlags::HAD_MORALE));
    assert_eq!(s.active(), Some(StackId(1)));
    // Struck, the pikemen see again and act in the same round.
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(2)));
    let from = at(&r, 3, 5);
    act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(0),
            from,
        },
    )
    .unwrap();
    assert_eq!((s.round(), s.active()), (1, Some(StackId(0))));
}

// covers: BAT-MOR-004
#[test]
fn stacks_activated_from_waiting_do_not_freeze() {
    let r = ruleset_with("t:faithful", always);
    // Two alignments: morale 0, so the attacker can wait.
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 1), (1, "imp", 1)],
        &[(0, "fixed", 1)],
        1,
    );
    act(&mut s, &r, Command::Wait).unwrap();
    stack(&mut s, 0).morale = -3;
    advance_to(&mut s, &r, 0);
    assert!(s.stacks()[0].flags.contains(StackFlags::WAITED));
}

// covers: BAT-MOR-004, BAT-MAG-024
#[test]
fn a_stack_that_sees_again_in_the_wait_phase_does_not_freeze() {
    let r = ruleset_with("t:faithful", always);
    // Three alignments: the attacker's morale is −1. Its pikemen are blinded by the blinder's
    // strike before their turn and passed over; the defender's imps wait and, in the wait phase,
    // strike them: they see again and act in that phase without the bad-morale check (the
    // original's pick skips it for the whole phase, +0x13DE4), although they never waited. Their
    // own Wait flag decided it until E04b T11a.
    let mut s = battle(
        &r,
        &[(0, "pikeman_c", 5), (1, "imp", 1), (2, "serf", 1)],
        &[(0, "blinder", 5), (1, "imp", 5)],
        1,
    );
    place(&mut s, &r, 0, 4, 5);
    place(&mut s, &r, 3, 5, 5);
    place(&mut s, &r, 4, 3, 5);
    assert_eq!(s.stacks()[0].morale, -1);
    activate(&mut s, 3);
    let from = at(&r, 5, 5);
    act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(0),
            from,
        },
    )
    .unwrap();
    // The other attackers freeze or defend; the blind pikemen are passed over.
    advance_to(&mut s, &r, 4);
    assert!(!s.stacks()[0].flags.contains(StackFlags::ACTED));
    act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(s.active(), Some(StackId(4)));
    assert!(s.wait_phase);
    let from = at(&r, 3, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(0),
            from,
        },
    )
    .unwrap();
    assert_eq!(count(&events, bad), 0);
    assert_eq!((s.round(), s.active()), (1, Some(StackId(0))));
    assert!(!s.stacks()[0].flags.contains(StackFlags::WAITED));
}

/// BAT-Q33 (batches 4a LO and 4b LC): with BAT-QUIRK-27 a frozen stack of a side whose player is
/// not human is let off when a second draw of 1 to 4 gives 1 (0x464BDB): a neutral side's or a
/// computer player's; a human player's never, nor any side without the quirk.
// covers: BAT-MOR-004
#[test]
fn with_bat_quirk_27_a_side_without_a_human_player_escapes_one_freeze_in_four() {
    let frozen = |profile: &str, human: bool| {
        let r = ruleset_with(profile, always);
        (0..400u64)
            .filter(|&seed| {
                let mut setup = setup(&r, &[(0, "pikeman_c", 1), (1, "imp", 1)], &[(0, "slow", 1)]);
                setup.human = [true, human];
                let mut s = BattleState::new(&r, &setup, seed).unwrap();
                stack(&mut s, 2).morale = -3;
                act(&mut s, &r, Command::Defend).unwrap();
                let events = act(&mut s, &r, Command::Defend).unwrap();
                count(&events, bad) == 1
            })
            .count()
    };
    // 254 of 255 freeze, three in four of them stay frozen: about 299 of 400 (σ ≈ 8.7).
    let let_off = frozen("t:let_off", false);
    assert!((260..=340).contains(&let_off), "{let_off}");
    assert!(frozen("t:let_off", true) >= 395);
    assert!(frozen("t:faithful", false) >= 395);
}

// covers: BAT-MOR-004
#[test]
fn bad_morale_happens_with_its_chance() {
    let r = ruleset_with("t:faithful", original);
    let frozen = (0..400u64)
        .filter(|&seed| {
            // Two alignments (0) for the attacker; the defender's morale is set to −3 before its
            // first activation.
            let mut s = battle(
                &r,
                &[(0, "pikeman_c", 1), (1, "imp", 1)],
                &[(0, "slow", 1)],
                seed,
            );
            stack(&mut s, 2).morale = -3;
            act(&mut s, &r, Command::Defend).unwrap();
            let events = act(&mut s, &r, Command::Defend).unwrap();
            count(&events, bad) == 1
        })
        .count();
    // Morale −3: 6/24 = 25 % of 400 = 100 (σ ≈ 8.7).
    assert!((70..=130).contains(&frozen), "{frozen}");
}
