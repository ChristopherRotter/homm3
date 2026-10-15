//! The computer's aim for the automatic turrets (E04c T7d; `docs/formats/installation.md`
//! §7.31): the score (BAT-SIE-020), the worth of a creature (BAT-SIE-021), the turns until a
//! stack can attack (BAT-SIE-022) and the desperate mode (BAT-SIE-023), on the test ruleset with
//! the turret and Fight Values ([`fighting_turret_ruleset`]). Without heroes every attack and
//! defense bonus is 0, so a creature's worth is its Fight Value; the keep of a town without
//! structures has 4 creatures, a mean damage of 12. The automatic Ballista's target (E04c T7e,
//! §7.33; BAT-WAR-008) and its hero's Artillery in its worth (BAT-SIE-021) follow.

use std::vec::Vec;

use h3_content::FULL_BP;
use h3_content_battle::BattleRuleset;
use h3_contracts::{
    BallistaControl, BattleHero, BattleSetup, BattleSide, FortLevel, Moat, Rate, Siege,
};
use h3_kernel::arith::{DOUBLE_BITS, X87};

use super::support::*;
use crate::aim::{
    advantage, avoids_stops, ballista_target, context, estimate, ftol, kill_value, shot_value,
    turns, turret_target, worth,
};
use crate::effects::{DamageRoll, Status};
use crate::geometry::{Field, Hex};
use crate::siege::damage_part;
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
    ("tent", 750),
    ("seraph", 500_000),
    ("spear", 300),
    ("fast", 100),
    ("serf", 700),
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
    let ctx = context(&s, &r, f, BattleSide::Defender);
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
    let ctx = context(&s, &r, field(&r), BattleSide::Defender);
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
    assert!(!context(&s, &r, field(&r), BattleSide::Defender).desperate);
}

// covers: BAT-SIE-020, BAT-SIE-021
#[test]
fn the_aim_s_sums_saturate_with_extreme_stacks() {
    // Two stacks of u32::MAX creatures of u32::MAX hit points beside the Ballista: their hit
    // points pass 2^64; the share's sums saturate, as the doubles do at 2^64 (`X87::MAX`), and
    // the Ballista's worth stays within its whole worth, 650.
    let r = rules();
    let mut setup = siege(
        &r,
        &[(0, "archer", 1), (1, "archer", 1)],
        &[(0, "slow", 10)],
    );
    setup.war_machines[0] = std::vec![r.creature_by_key("t:ballista").unwrap()];
    let mut s = fresh(&r, &setup);
    for st in &mut s.stacks.as_mut_slice()[..2] {
        (st.count, st.max_hp, st.top_hp) = (u32::MAX, u32::MAX, u32::MAX);
    }
    let ctx = context(&s, &r, field(&r), BattleSide::Defender);
    let ballista = worth(&s, (&r, field(&r)), &ctx, StackId(2), true);
    assert!(
        ballista > X87::ZERO && ballista <= X87::int(650),
        "{ballista:?}"
    );
    // A kill value's damage with the top creature's lost hit points saturates too: as much as
    // without them.
    let whole = kill_value(s.at(StackId(0)), i64::MAX, X87::ONE);
    s.stacks.as_mut_slice()[0].top_hp = 1;
    assert_eq!(kill_value(s.at(StackId(0)), i64::MAX, X87::ONE), whole);
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
    let at = |s: &BattleState, id: u8| turns(s, (&r, f), StackId(id));
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
    // The Catapult has no target; a human player's walker goes through the wall its first shot
    // broke, wading the moat (each moat hex ends a turn; a computer's of 100 hit points keeps out
    // of it, `a_computer_s_weak_walker_keeps_out_of_the_moat`).
    let mut setup = setup;
    setup.heroes[0] = Some(hero(&r));
    setup.human = [true, false];
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
    let living = turns(&s, (&r, f), StackId(0)).unwrap();
    assert!(living > 1, "{living}");
    let dead = &mut s.stacks.as_mut_slice()[2];
    dead.count = 0;
    dead.hex = Hex(walker.0 + 3);
    assert_eq!(turns(&s, (&r, f), StackId(0)), Some(1));
    // Bound, it reaches neither (BAT-MAG-045).
    s.binds.bound |= 1;
    assert_eq!(turns(&s, (&r, f), StackId(0)), None);
}

/// A Castle siege with a damaging moat, players `human`, the wall of row 4 (target 2, hex 78)
/// broken, the attacker's stack 0 at `walker` and the defender's at hex 80 (row 4, column 12),
/// just inside; the moat's hex on row 4 is 77 (column 9).
fn through_the_wall(r: &BattleRuleset, walker: Spec, human: [bool; 2], at: u8) -> BattleState {
    let mut setup = town_battle(r, Moat::Damaging, &[walker], &[(0, "slow", 1)]);
    setup.human = human;
    let mut s = fresh(r, &setup);
    damage_part(&mut s, 2, 9, &mut Vec::<Event>::new());
    s.stacks.as_mut_slice()[0].hex = Hex(at);
    s.stacks.as_mut_slice()[1].hex = Hex(4 * 17 + 12);
    s
}

// covers: BAT-SIE-022
#[test]
fn a_two_hex_walker_pays_a_moat_hex_it_covers_once() {
    // Its front three hexes before the moat (the position, its rear, at 73): the front enters
    // the moat at 2 + 3 = 5; the next step leaves its rear on that moat hex, which it covers
    // already, 6 (0x4B3701–0x4B3725); beside the defender at 7: 2 turns at speed 5. Human
    // players: no moat avoidance.
    let r = rules();
    let s = through_the_wall(&r, (0, "wide", 10), [true, true], 4 * 17 + 5);
    assert_eq!(turns(&s, (&r, field(&r)), StackId(0)), Some(2));
}

// covers: BAT-SIE-022
#[test]
fn a_computer_s_weak_walker_keeps_out_of_the_moat() {
    // A walker of 10 hit points and speed 5 three hexes before the moat: a human player's wades
    // through it (in at 2 + 3 = 5, beside the defender at 7: 2 turns); the computer's steps on
    // no stop (0x4B3400 at 0x4B375C), and the moat lies between it and every defender: none.
    let r = rules();
    let f = field(&r);
    let human = through_the_wall(&r, (0, "fixed", 1), [true, true], 4 * 17 + 6);
    assert_eq!(turns(&human, (&r, f), StackId(0)), Some(2));
    let computer = through_the_wall(&r, (0, "fixed", 1), [false, true], 4 * 17 + 6);
    assert_eq!(turns(&computer, (&r, f), StackId(0)), None);
}

// covers: BAT-SIE-022
#[test]
fn the_computer_s_walker_keeps_out_of_its_stops_by_its_hit_points() {
    // 0x4B3400's prologue (0x4B3420–0x4B34B2): at a town, a stack whose controller's player is
    // no human keeps out of its stops with at most 4 times the town's moat damage in hit points
    // (the table's, t:castle's 70, harmless or not), or 40 times when it is the defender's or
    // the gate is not closed. `fixed` has 10 hit points.
    let r = rules();
    let at = |fort, moat, human, (attackers, defenders): (u32, u32)| {
        let siege = Siege {
            faction: r.faction_by_key("t:castle").unwrap(),
            fort,
            moat,
            structures: 0,
            escape_tunnel: false,
            morale: 0,
            luck: 0,
        };
        let setup = BattleSetup {
            siege: Some(siege),
            human,
            ..setup(&r, &[(0, "fixed", attackers)], &[(0, "fixed", defenders)])
        };
        fresh(&r, &setup)
    };
    let avoids = |s: &BattleState, id: u8| avoids_stops(s, &r, s.at(StackId(id)));
    let citadel = |counts| at(FortLevel::Citadel, Moat::Damaging, [false; 2], counts);
    // The attacker before the closed gate: 4 times; the defender: 40 times.
    let s = citadel((28, 280));
    assert!(avoids(&s, 0) && avoids(&s, 1));
    let s = citadel((29, 281));
    assert!(!avoids(&s, 0) && !avoids(&s, 1));
    // The gate open: 40 times for the attacker too.
    let mut open = citadel((29, 1));
    let mut town = open.siege.unwrap();
    town.gate_open = true;
    open.siege = Some(town);
    assert!(avoids(&open, 0));
    open.stacks.as_mut_slice()[0].count = 281;
    assert!(!avoids(&open, 0));
    // The table's damage under a harmless moat; a town without a fort keeps the original's
    // closed gate (3, from the setup).
    assert!(avoids(
        &at(FortLevel::Citadel, Moat::Harmless, [false; 2], (28, 1)),
        0
    ));
    let fortless = |n| at(FortLevel::None, Moat::Absent, [false; 2], (n, 1));
    assert!(avoids(&fortless(28), 0) && !avoids(&fortless(29), 0));
    // A clone has 1 hit point.
    let mut s = citadel((29, 1));
    let clone = &mut s.stacks.as_mut_slice()[0];
    clone.traits = clone.traits.with(StackTraits::CLONE, true);
    assert!(avoids(&s, 0));
    // A human player's stacks wade; a hypnotized stack goes by its controller's player.
    let s = at(FortLevel::Citadel, Moat::Damaging, [true, false], (28, 280));
    assert!(!avoids(&s, 0) && avoids(&s, 1));
    let mut s = at(FortLevel::Citadel, Moat::Damaging, [false, true], (28, 1));
    let st = &mut s.stacks.as_mut_slice()[0];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    assert!(!avoids(&s, 0));
    // Away from a town, never.
    let s = fresh(&r, &setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)]));
    assert!(!avoids(&s, 0));
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
    assert!(context(&s, &r, field(&r), BattleSide::Defender).desperate);
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
    assert!(!context(&s, &r, field(&r), BattleSide::Defender).desperate);
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
        .filter(|(_, st)| st.side == BattleSide::Attacker)
        .last()
        .unwrap()
        .0;
    assert_eq!(s.at(StackId(last)).slot, crate::state::CATAPULT_SLOT);
    assert_eq!(
        turret_target(&s, &r, field(&r), keep(&s)),
        Some(StackId(last))
    );
}

// ---------------------------------------------------------------- BAT-WAR-008 the Ballista's aim

/// A hero of `attack` (and no defense) whose army brings a Ballista (no Artillery: it shoots on
/// its own).
fn gunner(r: &BattleRuleset, attack: u8) -> BattleHero {
    let mut h = hero(r);
    [h.attack, h.defense] = [attack, 0];
    h
}

/// A battle away from towns: the attacker's army and its Ballista under a hero of `attack`, the
/// defender's army and war machines; ids: the attacker's stacks, its Ballista, the defender's.
fn ballista_battle(
    r: &BattleRuleset,
    attack: u8,
    (attacker, defender): (&[Spec], &[Spec]),
    machines: &[&str],
) -> BattleSetup {
    let mut s = setup(r, attacker, defender);
    s.heroes[0] = Some(gunner(r, attack));
    s.war_machines = [
        std::vec![r.creature_by_key("t:ballista").unwrap()],
        machines
            .iter()
            .map(|k| r.creature_by_key(&std::format!("t:{k}")).unwrap())
            .collect(),
    ];
    s
}

/// The attacker's Ballista's id in [`ballista_battle`].
fn gun(s: &BattleState) -> StackId {
    (0u8..)
        .zip(s.stacks())
        .find(|(_, st)| st.side == BattleSide::Attacker && st.has(StackTraits::WAR_MACHINE))
        .map(|(i, _)| StackId(i))
        .unwrap()
}

/// The target of the attacker's Ballista.
fn aimed(r: &BattleRuleset, s: &BattleState) -> Option<u8> {
    ballista_target(s, r, field(r), gun(s)).map(|id| id.0)
}

/// The value of a shot of the attacker's Ballista at `target`, out of desperate mode.
fn value(r: &BattleRuleset, s: &BattleState, target: u8) -> i32 {
    let ctx = context(s, r, field(r), BattleSide::Attacker);
    shot_value(s, (r, field(r)), &ctx, (gun(s), StackId(target)), false)
}

// covers: BAT-WAR-008
#[test]
fn the_ballista_estimates_its_shot_without_draws() {
    let r = rules();
    let mut s = fresh(
        &r,
        &ballista_battle(
            &r,
            20,
            (&[(0, "sniper", 5), (1, "lich", 5)], &[(0, "fixed", 10)]),
            &[],
        ),
    );
    let (snipers, liches, b, t) = (StackId(0), StackId(1), StackId(2), StackId(3));
    let shot = |s: &BattleState, from: StackId, base: u64| {
        crate::combat::shot_damage(s, &r, (from, t), base)
    };
    // The Ballista's record holds its creature's 2..3, not the 42..63 its hero's attack makes of
    // them in a strike (0x4431AC): the mean 2.5, truncated.
    assert_eq!(estimate(&s, &r, (b, t)), shot(&s, b, 2));
    // Blessed, the maximum and the bonus (its own 3, not 63).
    s.stacks.as_mut_slice()[2].spells.damage = DamageRoll::Maximum(3);
    assert_eq!(estimate(&s, &r, (b, t)), shot(&s, b, 6));
    s.stacks.as_mut_slice()[2].spells.damage = DamageRoll::Random;
    // Five liches of damage 11..13: 5 × 12; blessed 5 × (13 + 2); cursed 5 × (11 - 4), at least
    // 1 each.
    assert_eq!(estimate(&s, &r, (liches, t)), shot(&s, liches, 60));
    s.stacks.as_mut_slice()[1].spells.damage = DamageRoll::Maximum(2);
    assert_eq!(estimate(&s, &r, (liches, t)), shot(&s, liches, 75));
    s.stacks.as_mut_slice()[1].spells.damage = DamageRoll::Minimum(4);
    assert_eq!(estimate(&s, &r, (liches, t)), shot(&s, liches, 35));
    s.stacks.as_mut_slice()[1].spells.damage = DamageRoll::Minimum(50);
    assert_eq!(estimate(&s, &r, (liches, t)), shot(&s, liches, 5));
    s.stacks.as_mut_slice()[1].spells.damage = DamageRoll::Random;
    // Five snipers of damage 2..3 that shoot twice: 5 × 2.5 truncated, doubled.
    assert_eq!(estimate(&s, &r, (snipers, t)), 2 * shot(&s, snipers, 12));
    // Their count from their hit points, rounded up: a hurt top creature still counts.
    s.stacks.as_mut_slice()[0].top_hp = 1;
    assert_eq!(estimate(&s, &r, (snipers, t)), 2 * shot(&s, snipers, 12));
    // At most the target's hit points: one fixed creature (10) or a clone (1).
    s.stacks.as_mut_slice()[3].count = 1;
    assert!(shot(&s, liches, 60) > 10);
    assert_eq!(estimate(&s, &r, (liches, t)), 10);
    let clone = &mut s.stacks.as_mut_slice()[3];
    clone.traits = clone
        .traits
        .with(StackTraits::CLONE, true)
        .with(StackTraits::SUMMONED, true);
    assert_eq!(estimate(&s, &r, (liches, t)), 1);
}

// covers: BAT-WAR-008
#[test]
fn the_ballista_scores_kills_over_turns_but_not_on_easy_for_the_computer() {
    // The Ballista's estimate is 2 hit points for either stack (its 2..3, its attack 30 over
    // their defense, halved beyond ten hexes). The spearmen (Fight Value 300 per 10 hit points)
    // score 60 but stand four turns away (15); the archer (140 per 10) shoots now (28). On Easy
    // a computer side does not divide: the spearmen.
    let r = rules();
    let base = ballista_battle(
        &r,
        20,
        (&[(0, "fixed", 1)], &[(0, "spear", 10), (1, "archer", 1)]),
        &[],
    );
    let at = |difficulty: u8, human: [bool; 2]| {
        let s = fresh(
            &r,
            &BattleSetup {
                difficulty,
                human,
                ..base.clone()
            },
        );
        (aimed(&r, &s), value(&r, &s, 2), value(&r, &s, 3))
    };
    assert_eq!(at(1, [false, false]), (Some(3), 15, 28));
    assert_eq!(at(0, [true, false]), (Some(3), 15, 28));
    assert_eq!(at(0, [false, true]), (Some(2), 60, 28));
    // The value is the estimate's kills times the worth over the turns.
    let s = fresh(
        &r,
        &BattleSetup {
            difficulty: 2,
            ..base
        },
    );
    let dealt = |t: u8| i64::try_from(estimate(&s, &r, (gun(&s), StackId(t)))).unwrap();
    assert_eq!((dealt(2), dealt(3)), (2, 2));
    assert_eq!(turns(&s, (&r, field(&r)), StackId(2)), Some(4));
    assert_eq!(
        value(&r, &s, 3),
        kill_value(s.at(StackId(3)), dealt(3), X87::int(140))
    );
    assert_eq!(turns(&s, (&r, field(&r)), StackId(3)), Some(1));
}

// covers: BAT-WAR-008
#[test]
fn the_ballista_prefers_a_stack_that_keeps_its_turns() {
    // The fast creatures (100 per 3 hit points: 66, two turns away: 33) are worth more than the
    // archer (28), also blind (a tenth: 6); the archer keeps its turns and is shot first. Both
    // blind, the higher value.
    let r = rules();
    let mut s = fresh(
        &r,
        &BattleSetup {
            difficulty: 1,
            ..ballista_battle(
                &r,
                20,
                (&[(0, "fixed", 1)], &[(0, "fast", 10), (1, "archer", 1)]),
                &[],
            )
        },
    );
    assert_eq!((value(&r, &s, 2), value(&r, &s, 3)), (33, 28));
    assert_eq!(aimed(&r, &s), Some(2));
    let blind = |s: &mut BattleState, id: usize| {
        let st = &mut s.stacks.as_mut_slice()[id];
        st.spells.status = st.spells.status.with(Status::NO_TURNS, true);
    };
    blind(&mut s, 2);
    assert_eq!(value(&r, &s, 2), 66 / 10);
    assert_eq!(aimed(&r, &s), Some(3));
    blind(&mut s, 3);
    assert_eq!(value(&r, &s, 3), 28 / 10);
    assert_eq!(aimed(&r, &s), Some(2));
    // An earlier stack that keeps its turns before a later blind one of a higher value: the slow
    // creatures (20 per 5 hit points: 8, more than five turns away: a fifth, 1) before the blind
    // fast ones (6).
    let mut s = fresh(
        &r,
        &BattleSetup {
            difficulty: 1,
            ..ballista_battle(
                &r,
                20,
                (&[(0, "fixed", 1)], &[(0, "slow", 10), (1, "fast", 10)]),
                &[],
            )
        },
    );
    blind(&mut s, 3);
    assert!(turns(&s, (&r, field(&r)), StackId(2)).is_some_and(|t| t > 5));
    assert_eq!((value(&r, &s, 2), value(&r, &s, 3)), (8 / 5, 66 / 10));
    assert_eq!(aimed(&r, &s), Some(2));
}

// covers: BAT-WAR-008
#[test]
fn the_ballista_shoots_the_later_of_two_like_stacks() {
    // VCMI's rule took the lower id.
    let r = rules();
    let s = fresh(
        &r,
        &ballista_battle(
            &r,
            0,
            (&[(0, "fixed", 1)], &[(0, "archer", 1), (1, "archer", 1)]),
            &[],
        ),
    );
    assert_eq!(value(&r, &s, 2), value(&r, &s, 3));
    assert_eq!(aimed(&r, &s), Some(3));
}

// covers: BAT-WAR-008
#[test]
fn a_war_machine_is_a_target_as_any_stack() {
    // The defender's tanks have no Fight Value: its Ballista (its share of the army: 650 · 1000 /
    // 1250, shooting now) is shot before them, where VCMI's rule preferred any creature; its
    // First Aid Tent counts a fifth.
    let r = rules();
    let mut s = fresh(
        &r,
        &BattleSetup {
            difficulty: 1,
            ..ballista_battle(
                &r,
                20,
                (&[(0, "fixed", 1)], &[(0, "tank", 10)]),
                &["ballista"],
            )
        },
    );
    assert_eq!(value(&r, &s, 2), 0);
    assert!(value(&r, &s, 3) > 0);
    assert_eq!(aimed(&r, &s), Some(3));
    let mut with_tent =
        ballista_battle(&r, 20, (&[(0, "fixed", 1)], &[(0, "tank", 10)]), &["tent"]);
    with_tent.difficulty = 1;
    s = fresh(&r, &with_tent);
    let ctx = context(&s, &r, field(&r), BattleSide::Attacker);
    let dealt = i64::try_from(estimate(&s, &r, (gun(&s), StackId(3)))).unwrap();
    let whole = kill_value(
        s.at(StackId(3)),
        dealt,
        worth(&s, (&r, field(&r)), &ctx, StackId(3), true),
    );
    assert!(whole > 0);
    assert_eq!(value(&r, &s, 3), whole / 5);
}

// covers: BAT-WAR-008, BAT-SIE-023
#[test]
fn a_desperate_ballista_counts_every_creature_1000() {
    // A weak attacker (one slow creature) against ten Magogs (300, 13 hit points) and ten archers
    // (140, 10), both shooting now: the Magogs by their worth, the archers in desperate mode
    // (above Easy or human), where each creature killed counts 1000.
    let r = rules();
    let base = ballista_battle(
        &r,
        20,
        (&[(0, "slow", 1)], &[(0, "magog", 10), (1, "archer", 10)]),
        &[],
    );
    let at = |difficulty: u8, human: [bool; 2]| {
        let s = fresh(
            &r,
            &BattleSetup {
                difficulty,
                human,
                ..base.clone()
            },
        );
        aimed(&r, &s)
    };
    assert_eq!(at(0, [false, false]), Some(2));
    assert_eq!(at(1, [false, false]), Some(3));
    assert_eq!(at(0, [true, false]), Some(3));
    // A clone counts its creatures times its real worth over 5, desperate or not: 140 shared
    // with the side's other 130 hit points (its own counted 1), 10 · 138.9 / 5.
    let mut s = fresh(
        &r,
        &BattleSetup {
            difficulty: 1,
            ..base
        },
    );
    let clone = &mut s.stacks.as_mut_slice()[3];
    clone.traits = clone
        .traits
        .with(StackTraits::CLONE, true)
        .with(StackTraits::SUMMONED, true);
    let ctx = context(&s, &r, field(&r), BattleSide::Attacker);
    assert!(ctx.desperate);
    let desperate = shot_value(&s, (&r, field(&r)), &ctx, (gun(&s), StackId(3)), true);
    let real = worth(&s, (&r, field(&r)), &ctx, StackId(3), true);
    assert_eq!(desperate, kill_value(s.at(StackId(3)), 0, real));
    assert_eq!(desperate, 277);
}

// covers: BAT-SIE-020, BAT-WAR-008
#[test]
fn the_runtime_s_ftol_keeps_the_low_32_bits() {
    // 0x6179D4 truncates into 64 bits (`fistp qword`) and its callers keep `eax`.
    assert_eq!(ftol(X87::int(5)), 5);
    assert_eq!(ftol(X87::int(5).over(X87::int(2)).unwrap()), 2);
    assert_eq!(ftol(X87::int(1 << 31)), i32::MIN);
    assert_eq!(ftol(X87::int((1 << 32) + 7)), 7);
    // From 2^63 on the integer indefinite, 2^63: its low half is 0.
    assert_eq!(ftol(X87::int(1 << 63)), 0);
    assert_eq!(ftol(X87::int(u64::MAX)), 0);
}

// covers: BAT-WAR-008, BAT-SIE-023
#[test]
fn a_desperate_ballista_without_a_value_above_0_chooses_again() {
    // Two stacks of 100 000 hit points a creature: no estimate kills a thousandth of one, so in
    // desperate mode every value is 0 (the later stack would be shot); chosen again with the real
    // worth, the seraphs' (500 000 each) count.
    let r = rules();
    let mut s = fresh(
        &r,
        &BattleSetup {
            difficulty: 1,
            ..ballista_battle(
                &r,
                20,
                (&[(0, "slow", 1)], &[(0, "seraph", 1), (1, "tank", 1)]),
                &[],
            )
        },
    );
    for st in &mut s.stacks.as_mut_slice()[2..4] {
        st.max_hp = 100_000;
        st.top_hp = 100_000;
    }
    assert!(context(&s, &r, field(&r), BattleSide::Attacker).desperate);
    assert_eq!(aimed(&r, &s), Some(2));
    // Blind, the seraphs lose to the tanks, which keep their turns, whatever their value.
    s.stacks.as_mut_slice()[2].spells.status =
        s.stacks()[2].spells.status.with(Status::NO_TURNS, true);
    assert_eq!(aimed(&r, &s), Some(3));
}

// covers: BAT-WAR-008, BAT-MAG-026
#[test]
fn the_ballista_shoots_the_other_sides_stacks_whoever_controls_them() {
    // Its own side's stack hypnotized by the defender is no target; the defender's stack it
    // hypnotized is one (the original walks the other side's records), but without a threat
    // record it counts a fifth: its like is shot first.
    let r = rules();
    let mut s = fresh(
        &r,
        &BattleSetup {
            difficulty: 1,
            ..ballista_battle(
                &r,
                0,
                (&[(0, "archer", 1)], &[(0, "archer", 1), (1, "archer", 1)]),
                &[],
            )
        },
    );
    let hypnotize = |s: &mut BattleState, id: usize| {
        let st = &mut s.stacks.as_mut_slice()[id];
        st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    };
    hypnotize(&mut s, 0);
    assert_eq!(aimed(&r, &s), Some(3));
    hypnotize(&mut s, 3);
    assert_eq!(value(&r, &s, 3), value(&r, &s, 2) / 5);
    assert_eq!(aimed(&r, &s), Some(2));
    // Only the hypnotized stack left: it is shot.
    s.stacks.as_mut_slice()[2].count = 0;
    assert_eq!(aimed(&r, &s), Some(3));
    // Nothing left to shoot.
    s.stacks.as_mut_slice()[3].count = 0;
    assert_eq!(aimed(&r, &s), None);
}

/// E04b T11b review (F-robust-1, F-sec-1 part b), kept for the computer's aim: the turns a
/// candidate needs are counted from the pits both sides see. The shot is public, so a Quicksand
/// pit hidden from one side (BAT-MAG-031) must not change the target.
// covers: BAT-WAR-008, BAT-MAG-031
#[test]
fn the_automatic_ballista_does_not_show_hidden_pits() {
    use h3_content_battle::ObstacleKind;
    let r = rules();
    // Ids: 0 serf, 1 fixed (attacker; both of speed 5); 2 seraph (so strong a defender that it is
    // not desperate), 3 tank, 4 ballista (defender), whose hero aims it.
    let mut setup = setup(
        &r,
        &[(0, "serf", 10), (1, "fixed", 10)],
        &[(0, "seraph", 1), (1, "tank", 10)],
    );
    setup.heroes[1] = Some(gunner(&r, 20));
    setup.war_machines[1] = std::vec![r.creature_by_key("t:ballista").unwrap()];
    setup.difficulty = 1;
    // A corridor along row 5 from the first attacker stack to the first defender stack.
    setup.obstacles = blocked(
        (4..=8)
            .flat_map(|x| [at(&r, x, 4).0, at(&r, x, 6).0])
            .collect(),
    );
    let mut clear = fresh(&r, &setup);
    // The attacker's stacks are five steps (their speed) from an attack position on a defender
    // stack each: the first only through the corridor.
    place(&mut clear, &r, 0, 3, 5);
    place(&mut clear, &r, 1, 3, 9);
    place(&mut clear, &r, 2, 9, 5);
    place(&mut clear, &r, 3, 9, 9);
    let quicksand = r.spell_by_key("t:quicksand").unwrap();
    let shape = r.spell(quicksand).unwrap().magic.obstacle.clone().unwrap();
    let with_pit = |side: BattleSide| {
        let mut s = clear.clone();
        crate::obstacles::place(
            &mut s,
            (quicksand, ObstacleKind::Pit, &shape),
            side,
            (&[at(&r, 6, 5)], 0),
            &mut Vec::new(),
        );
        assert!(crate::obstacles::seen_by(&s, side.opponent()).is_empty());
        s
    };
    let shot = |s: &BattleState| ballista_target(s, &r, field(&r), StackId(4)).map(|id| id.0);
    // Both one turn away: the serfs (the estimate's 2 hit points at 700 per 10: 140) before the
    // fixed (500: 100).
    assert_eq!(turns(&clear, (&r, field(&r)), StackId(0)), Some(1));
    assert_eq!(shot(&clear), Some(0));
    // A hidden pit changes nothing, whichever side's.
    assert_eq!(shot(&with_pit(BattleSide::Attacker)), Some(0));
    assert_eq!(shot(&with_pit(BattleSide::Defender)), Some(0));
    // A pit both sides see stops the serfs: two turns, half their value (70), the fixed first.
    for side in [BattleSide::Attacker, BattleSide::Defender] {
        let mut s = with_pit(side);
        s.spell_obstacles.reveal_for_tests(at(&r, 6, 5));
        assert_eq!(turns(&s, (&r, field(&r)), StackId(0)), Some(2));
        assert_eq!(shot(&s), Some(1), "{side:?}");
    }
}

// ---------------------------------------------------------------- BAT-SIE-021 Artillery

// covers: BAT-SIE-021
#[test]
fn the_artillery_level_grows_a_ballistas_worth() {
    // The attacker's Ballista (650; its share of the army: 10 of 260 hit points) under a hero of
    // no attack: its offense 1 times 1, 1.5, 3, 4 by the hero's Artillery level (0x63B7F0, doubled
    // above Basic), before the root.
    let r = rules();
    let worth_at = |level: Option<u8>| {
        let mut setup = ballista_battle(&r, 0, (&[(0, "archer", 1)], &[(0, "slow", 10)]), &[]);
        let h = setup.heroes[0].as_mut().unwrap();
        h.war_machines.ballista = level.map(|level| BallistaControl {
            level,
            shots: 1,
            double_damage_chance_bp: 0,
        });
        let s = fresh(&r, &setup);
        let ctx = context(&s, &r, field(&r), BattleSide::Defender);
        worth(&s, (&r, field(&r)), &ctx, gun(&s), true)
    };
    let expected = |factor: X87| {
        X87::int(10)
            .times(factor.sqrt().times(X87::int(650)))
            .over(X87::int(260))
            .unwrap()
    };
    let d = |n, den| X87::ratio(n, den, DOUBLE_BITS).unwrap();
    assert_eq!(worth_at(None), expected(X87::ONE));
    assert_eq!(worth_at(Some(1)), expected(d(3, 2)));
    assert_eq!(worth_at(Some(2)), expected(X87::int(2).times(d(3, 2))));
    assert_eq!(worth_at(Some(3)), expected(X87::int(2).times(X87::int(2))));
}
