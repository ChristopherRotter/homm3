//! The computer's aim for the automatic turrets (E04c T7d; `docs/formats/installation.md`
//! §7.31): the score (BAT-SIE-020), the worth of a creature (BAT-SIE-021), the turns until a
//! stack can attack (BAT-SIE-022) and the desperate mode (BAT-SIE-023), on the test ruleset with
//! the turret and Fight Values ([`fighting_turret_ruleset`]). Without heroes every attack and
//! defense bonus is 0, so a creature's worth is its Fight Value; the keep of a town without
//! structures has 4 creatures, a mean damage of 12.

use std::vec::Vec;

use h3_content::FULL_BP;
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleSetup, FortLevel, Moat, Rate, Siege};
use h3_kernel::arith::{DOUBLE_BITS, X87};

use super::support::*;
use crate::aim::{advantage, context, kill_value, turns, turret_target, worth};
use crate::effects::{DamageRoll, Status};
use crate::geometry::{Field, Hex};
use crate::{BattleState, Event, EventKind, StackId, StackTraits};

/// The Fight Values of the creatures these tests use.
const FIGHTS: &[(&str, u32)] = &[
    ("archer", 140),
    ("fixed", 500),
    ("flyer", 100),
    ("magog", 300),
    ("slow", 20),
    ("sniper", 90),
    ("ballista", 650),
];

fn rules() -> BattleRuleset {
    fighting_turret_ruleset("t:faithful", FIGHTS)
}

/// A battle at a town with a Citadel (the keep, the walls), no moat.
fn siege(r: &BattleRuleset, attacker: &[Spec], defender: &[Spec]) -> BattleSetup {
    town_battle(r, Moat::Absent, attacker, defender)
}

fn town_battle(r: &BattleRuleset, moat: Moat, attacker: &[Spec], defender: &[Spec]) -> BattleSetup {
    BattleSetup {
        siege: Some(Siege {
            faction: r.faction_by_key("t:castle").unwrap(),
            fort: FortLevel::Citadel,
            moat,
            structures: 0,
            escape_tunnel: false,
            morale: 0,
            luck: 0,
        }),
        ..setup(r, attacker, defender)
    }
}

fn field(r: &BattleRuleset) -> Field {
    Field::new(r.battle().field)
}

/// The keep's stack.
fn keep(s: &BattleState) -> StackId {
    (0u8..)
        .zip(s.stacks())
        .find(|(_, st)| st.is_turret())
        .map(|(i, _)| StackId(i))
        .unwrap()
}

/// The stacks the turrets struck in `events`.
fn turret_targets(s: &BattleState, events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker, target, ..
            } if s.at(attacker).is_turret() => Some(target.0),
            _ => None,
        })
        .collect()
}

/// The battle of `setup` with its stacks whole: [`BattleState::new`] runs the first activation,
/// the keep's shot, whose damage is undone.
fn fresh(r: &BattleRuleset, setup: &BattleSetup) -> BattleState {
    let mut s = BattleState::new(r, setup, 5).unwrap();
    for st in s.stacks.as_mut_slice() {
        st.count = st.base_count;
        st.top_hp = st.max_hp;
    }
    s
}

/// The battle of `setup` started: the keep has taken its first shot.
fn started(r: &BattleRuleset, setup: &BattleSetup) -> (BattleState, Vec<Event>) {
    let mut events = Vec::new();
    let s = BattleState::start(r, setup, 5, &mut events).unwrap();
    (s, events)
}

// ---------------------------------------------------------------- BAT-SIE-020 the score

// covers: BAT-SIE-020
#[test]
fn the_keep_shoots_the_later_of_two_like_stacks() {
    // Batch 3e's AI encounter: ten Marksmen and one score alike (12 · 140 / 10 = 168 each,
    // both shoot); the later record wins the tie, where the Ballista's rule took the ten.
    let r = rules();
    let (s, events) = started(
        &r,
        &siege(
            &r,
            &[(0, "archer", 10), (1, "archer", 1)],
            &[(0, "fixed", 10)],
        ),
    );
    assert_eq!(turret_targets(&s, &events), [1]);
}

// covers: BAT-SIE-020, BAT-SIE-022
#[test]
fn a_stack_far_from_an_attack_scores_less() {
    let r = rules();
    let setup = siege(
        &r,
        &[(0, "fixed", 10), (1, "archer", 1)],
        &[(0, "slow", 10)],
    );
    // The walker behind the walls has no target: 12 · 500 / 10 = 600, a fifth 120; the archer
    // shoots now: 12 · 140 / 10 = 168.
    let (s, events) = started(&r, &setup);
    assert_eq!(turret_targets(&s, &events), [1]);
    // Blind, the archer counts a fifth too (33) and the walker wins.
    let mut blind = fresh(&r, &setup);
    let archer = &mut blind.stacks.as_mut_slice()[1];
    archer.spells.status = archer.spells.status.with(Status::NO_TURNS, true);
    assert_eq!(
        turret_target(&blind, &r, field(&r), keep(&blind)),
        Some(StackId(0))
    );
}

// covers: BAT-SIE-020
#[test]
fn the_score_divides_the_kills_value() {
    let r = rules();
    let mut s = fresh(&r, &siege(&r, &[(0, "archer", 3)], &[(0, "slow", 3)]));
    let v = X87::int(140);
    // 7 of 10 hit points: below the top creature's 6 left, 7 counts as dealt.
    s.stacks.as_mut_slice()[0].top_hp = 6;
    assert_eq!(kill_value(s.at(StackId(0)), 3, v), 42);
    assert_eq!(kill_value(s.at(StackId(0)), 7, v), (7 + 4) * 14);
    // A clone's creatures times the worth over 5, whatever the damage.
    let clone = &mut s.stacks.as_mut_slice()[0];
    clone.traits = clone
        .traits
        .with(StackTraits::CLONE, true)
        .with(StackTraits::SUMMONED, true);
    assert_eq!(kill_value(s.at(StackId(0)), 7, v), 3 * 140 / 5);
}

// ---------------------------------------------------------------- BAT-SIE-021 the worth

// covers: BAT-SIE-021
#[test]
fn a_creature_is_worth_its_fight_value_scaled_by_its_advantage() {
    let r = rules();
    let d = |n, den| X87::ratio(n, den, DOUBLE_BITS).unwrap();
    // 1 + 0.05 x in doubles, below 1 for a negative x.
    assert_eq!(advantage(0), X87::ONE);
    assert_eq!(advantage(2), X87::ONE.plus(X87::int(2).times(d(1, 20))));
    assert_eq!(advantage(-6), X87::ONE.minus(X87::int(6).times(d(1, 20))));
    let mut hero = hero(&r);
    [hero.attack, hero.defense] = [3, 2];
    let setup = BattleSetup {
        heroes: [Some(hero), None],
        ..siege(
            &r,
            &[(0, "archer", 10), (1, "sniper", 2), (2, "fixed", 4)],
            &[(0, "slow", 10)],
        )
    };
    let mut s = fresh(&r, &setup);
    let (f, field) = (field(&r), field(&r));
    let ctx = context(&s, &r, f, h3_contracts::BattleSide::Defender);
    assert_eq!(
        (ctx.min_attack, ctx.min_defense, ctx.desperate),
        (0, 0, false)
    );
    let worth_of =
        |s: &BattleState, id: u8, shot: bool| worth(s, (&r, field), &ctx, StackId(id), shot);
    // The attacker's stacks: attack +3, defense +2 over the least (the defender's 0).
    let base = advantage(3).times(advantage(2)).sqrt();
    assert_eq!(worth_of(&s, 0, true), base.times(X87::int(140)));
    // The sniper shoots twice.
    let twice = advantage(3).plus(advantage(3)).times(advantage(2)).sqrt();
    assert_eq!(worth_of(&s, 1, true), twice.times(X87::int(90)));
    // A walker's worth for a shot: no halving (it is no shooter).
    assert_eq!(worth_of(&s, 2, true), base.times(X87::int(500)));
    // Blessed (the maximum 3 plus 1 over the mean 2.5) and cursed (the minimum 2 less 1, over
    // 2.5), the factor before the root.
    let bless = |n: u64| {
        X87::int(n)
            .over(X87::int(5).over(X87::int(2)).unwrap())
            .unwrap()
            .times(advantage(3))
            .times(advantage(2))
            .sqrt()
            .times(X87::int(140))
    };
    s.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Maximum(1);
    assert_eq!(worth_of(&s, 0, true), bless(4));
    s.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Minimum(1);
    assert_eq!(worth_of(&s, 0, true), bless(1));
    s.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Minimum(5);
    assert_eq!(worth_of(&s, 0, true), bless(1));
    s.stacks.as_mut_slice()[0].spells.damage = DamageRoll::Random;
    // A shooter that cannot shoot (no shots left) counts half its offense.
    s.stacks.as_mut_slice()[0].shots = 0;
    let half = advantage(3).times(d(1, 2)).times(advantage(2)).sqrt();
    assert_eq!(worth_of(&s, 0, true), half.times(X87::int(140)));
    // Frenzied (the context as before): its defense 0, 10 below its base; its attack 13 with all
    // of its defense 12, 15 above its base.
    let frenzied = &mut s.stacks.as_mut_slice()[2];
    frenzied.spells.frenzy = 100;
    let expected = advantage(15).times(advantage(-10)).sqrt();
    assert_eq!(worth_of(&s, 2, true), expected.times(X87::int(500)));
}

// covers: BAT-SIE-021
#[test]
fn a_war_machine_is_worth_its_share_of_its_army() {
    // The attacker's Ballista: 650 times the army's other hit points (10) over its own and theirs
    // (250 + 10): 25.
    let r = rules();
    let mut setup = siege(&r, &[(0, "archer", 1)], &[(0, "slow", 10)]);
    setup.war_machines[0] = std::vec![r.creature_by_key("t:ballista").unwrap()];
    let mut s = fresh(&r, &setup);
    let ctx = context(&s, &r, field(&r), h3_contracts::BattleSide::Defender);
    assert_eq!(
        worth(&s, (&r, field(&r)), &ctx, StackId(1), true),
        X87::int(25)
    );
    // A clone is created: its worth is shared too, its own hit points counted 1 and the army's
    // others none (the Ballista is a war machine): 140 · 0 / 1.
    let clone = &mut s.stacks.as_mut_slice()[0];
    clone.traits = clone
        .traits
        .with(StackTraits::CLONE, true)
        .with(StackTraits::SUMMONED, true);
    assert_eq!(
        worth(&s, (&r, field(&r)), &ctx, StackId(0), true),
        X87::ZERO
    );
    // The clone's strength: its creature times its worth over 5, here nothing.
    assert!(!context(&s, &r, field(&r), h3_contracts::BattleSide::Defender).desperate);
}

// ---------------------------------------------------------------- BAT-SIE-022 the turns

// covers: BAT-SIE-022
#[test]
fn the_turns_until_a_stack_can_attack() {
    let r = rules();
    let setup = town_battle(
        &r,
        Moat::Damaging,
        &[(0, "fixed", 10), (1, "flyer", 10), (2, "archer", 1)],
        &[(0, "slow", 10)],
    );
    let s = fresh(&r, &setup);
    let f = field(&r);
    let at = |s: &BattleState, id: u8| turns(s, f, StackId(id));
    // Behind the walls the walker reaches no defender; the archer shoots now; the flyer needs
    // its hex distance to a place next to the defender, 7 a turn.
    assert_eq!(at(&s, 0), None);
    assert_eq!(at(&s, 2), Some(1));
    let flyer = s.at(StackId(1));
    let defender = s.at(StackId(3));
    let reach = f
        .attack_positions(false, flyer.side, defender.footprint(f))
        .into_iter()
        .flatten()
        .map(|p| f.distance(flyer.hex, p))
        .min()
        .unwrap();
    assert_eq!(at(&s, 1), Some(reach.div_ceil(7)));
    // No target for a hypnotized or a berserk stack.
    for status in [Status::HYPNOTIZED, Status::BERSERK] {
        let mut t = s.clone();
        let flyer = &mut t.stacks.as_mut_slice()[1];
        flyer.spells.status = flyer.spells.status.with(status, true);
        assert_eq!(at(&t, 1), None);
    }
    // A defender in front of the moat (row 5): the walker reaches a place beside it in 7 steps,
    // 2 turns at speed 5; the places in the moat cost a turn more.
    let mut out = s.clone();
    out.stacks.as_mut_slice()[3].hex = Hex(5 * 17 + 9);
    assert_eq!(at(&out, 0), Some(2));
    // The Catapult has no target; the walker goes through the wall its first shot broke, wading
    // the moat (each moat hex ends a turn).
    let mut setup = setup;
    setup.heroes[0] = Some(hero(&r));
    let shot = BattleState::new(&r, &setup, 5).unwrap();
    let catapult = (0u8..)
        .zip(shot.stacks())
        .find(|(_, st)| st.slot == crate::state::CATAPULT_SLOT)
        .unwrap()
        .0;
    assert_eq!(at(&shot, catapult), None);
    assert!(at(&shot, 0).is_some_and(|t| t >= 2), "{:?}", at(&shot, 0));
}

// covers: BAT-SIE-022
#[test]
fn a_dead_defender_s_place_counts_within_one_turn() {
    // Away from a town: the walker (speed 5) is three turns from the living defender, but the
    // place of a dead one lies within its move (the original's pathfinder marks, 0x422CC5).
    let r = rules();
    let mut s = fresh(
        &r,
        &setup(&r, &[(0, "fixed", 10)], &[(0, "slow", 10), (1, "slow", 1)]),
    );
    let f = field(&r);
    let walker = s.at(StackId(0)).hex;
    let living = turns(&s, f, StackId(0)).unwrap();
    assert!(living > 1, "{living}");
    let dead = &mut s.stacks.as_mut_slice()[2];
    dead.count = 0;
    dead.hex = Hex(walker.0 + 3);
    assert_eq!(turns(&s, f, StackId(0)), Some(1));
    // Bound, it reaches neither (BAT-MAG-045).
    s.binds.bound |= 1;
    assert_eq!(turns(&s, f, StackId(0)), None);
}

// ---------------------------------------------------------------- BAT-SIE-023 desperate mode

// covers: BAT-SIE-023
#[test]
fn a_weak_defender_turns_desperate_above_easy_or_when_human() {
    // The Magogs score 12 · 300 / 13 = 276, the archers 12 · 140 / 10 = 168: the Magogs. In
    // desperate mode each creature is worth 1000: 923 and 1200, the archers.
    let r = rules();
    let base = siege(
        &r,
        &[(0, "magog", 10), (1, "archer", 10)],
        &[(0, "slow", 1)],
    );
    let target = |difficulty: u8, human: [bool; 2]| {
        let setup = BattleSetup {
            difficulty,
            human,
            ..base.clone()
        };
        let (s, events) = started(&r, &setup);
        turret_targets(&s, &events)
    };
    assert_eq!(target(0, [false, false]), [0]);
    assert_eq!(target(0, [true, false]), [0]);
    assert_eq!(target(1, [false, false]), [1]);
    assert_eq!(target(0, [false, true]), [1]);
    // The strengths: the defender's 20 doubled is below the attacker's 3000 + 1400.
    let s = BattleState::new(
        &r,
        &BattleSetup {
            difficulty: 4,
            ..base.clone()
        },
        5,
    )
    .unwrap();
    assert!(context(&s, &r, field(&r), h3_contracts::BattleSide::Defender).desperate);
    // A strong defender is never desperate.
    let strong = siege(&r, &[(0, "archer", 1)], &[(0, "fixed", 10)]);
    let s = BattleState::new(
        &r,
        &BattleSetup {
            difficulty: 4,
            ..strong
        },
        5,
    )
    .unwrap();
    assert!(!context(&s, &r, field(&r), h3_contracts::BattleSide::Defender).desperate);
}

// covers: BAT-SIE-020, BAT-SIE-023
#[test]
fn without_a_score_above_0_the_last_stack_is_shot() {
    // The attacker's hero's Armorer takes all of the keep's damage: every score is 0, in
    // desperate mode and again with the real worth, and the last record (the Catapult) wins.
    let r = rules();
    let mut plated: BattleHero = hero(&r);
    plated.damage.reduction = Rate::bp(FULL_BP.into());
    let setup = BattleSetup {
        difficulty: 2,
        heroes: [Some(plated), None],
        ..siege(
            &r,
            &[(0, "magog", 10), (1, "archer", 10)],
            &[(0, "slow", 1)],
        )
    };
    let s = fresh(&r, &setup);
    let last = (0u8..)
        .zip(s.stacks())
        .filter(|(_, st)| st.side == h3_contracts::BattleSide::Attacker)
        .last()
        .unwrap()
        .0;
    assert_eq!(s.at(StackId(last)).slot, crate::state::CATAPULT_SLOT);
    assert_eq!(
        turret_target(&s, &r, field(&r), keep(&s)),
        Some(StackId(last))
    );
}
