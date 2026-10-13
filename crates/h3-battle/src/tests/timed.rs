//! Timed spells (E04b T3a): what a cast does to a stack (BAT-MAG-009), durations and their end
//! (BAT-MAG-010), the values the rules read (BAT-MAG-011), mass versions (BAT-MAG-003) and mind
//! spells (BAT-MAG-005).

use std::vec::Vec;

use h3_content::Affects;
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSide};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::{DamageRoll, slowed};
use crate::{
    ActiveEffect, BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind,
    StackId, cast_targets, castable_spells, observe,
};

/// The effect events as `(target, spell name, rounds)` (`rounds` `Some(0)` for an end).
fn timed_events(
    r: &BattleRuleset,
    events: &[Event],
) -> Vec<(u8, std::string::String, Option<u16>)> {
    let name = |id: h3_content::SpellId| {
        r.spell(id)
            .unwrap()
            .key
            .as_str()
            .trim_start_matches("t:")
            .into()
    };
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::EffectApplied {
                target,
                spell,
                rounds,
            } => Some((target.0, name(spell), rounds)),
            EventKind::EffectEnded { target, spell } => Some((target.0, name(spell), Some(0))),
            _ => None,
        })
        .collect()
}

/// A mage with `spells` and the air mastery `mastery`.
fn adept(r: &BattleRuleset, spells: &[&str], mastery: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.masteries = masteries(r, [mastery, 0, 0, 0]);
    h.magic.mana = 100;
    h
}

/// Starts a new round with `s`'s ruleset, returning its events.
fn new_round(s: &mut BattleState, r: &BattleRuleset) -> Vec<Event> {
    let mut events = Vec::new();
    crate::turn::start_round(s, r, &mut events);
    events
}

/// The entry of `spell` on stack `id`, if any.
fn entry(s: &BattleState, r: &BattleRuleset, id: u8, name: &str) -> Option<ActiveEffect> {
    s.effects()
        .iter()
        .find(|e| e.stack == StackId(id) && e.spell == spell(r, name))
        .copied()
}

// covers: BAT-MAG-009, BAT-MAG-010, BAT-MAG-011
#[test]
fn haste_lasts_power_rounds_and_ends_at_a_round_start() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["haste"], 2)), None],
        &[("fixed", 1, 3, 5)],
        &[("fast", 1, 13, 5)],
        1,
    );
    // Power 3: three rounds, the current one included; Advanced: +5.
    let mut plain = s.clone();
    let events = act(&mut s, &r, cast(&r, "haste", stack(0))).unwrap();
    assert_eq!(timed_events(&r, &events), [(0, "haste".into(), Some(3))]);
    let fixed = s.stacks()[0];
    assert_eq!((fixed.speed, fixed.speed_now()), (5, 10));
    assert_eq!(
        entry(&s, &r, 0, "haste").unwrap(),
        ActiveEffect {
            stack: StackId(0),
            spell: spell(&r, "haste"),
            rounds: Some(3),
            value: 5,
            mastery: 2,
            attack: 0,
            defense: 0,
            speed: 5,
        }
    );
    // The turn order reads the speed now: in round 2 the hasted stack (10) acts before the
    // enemy (9).
    assert_eq!(order(&mut s.clone(), &r, 3), [0, 1, 0]);
    assert_eq!(order(&mut plain, &r, 3), [0, 1, 1]);
    assert!(
        new_round(&mut s, &r)
            .iter()
            .all(|e| !matches!(e.kind, EventKind::EffectEnded { .. }))
    );
    assert_eq!(entry(&s, &r, 0, "haste").unwrap().rounds, Some(2));
    new_round(&mut s, &r);
    assert_eq!(entry(&s, &r, 0, "haste").unwrap().rounds, Some(1));
    let events = new_round(&mut s, &r);
    assert_eq!(timed_events(&r, &events), [(0, "haste".into(), Some(0))]);
    assert!(s.effects().is_empty());
    assert_eq!(s.stacks()[0].speed_now(), 5);
}

// covers: BAT-MAG-010
#[test]
fn the_duration_adds_the_heros_bonus_and_is_at_least_one_round() {
    let r = ruleset();
    let rounds = |power: u8, bonus: Option<u8>| {
        let mut h = mage(&r, &["skin"]);
        h.power = power;
        h.effects = bonus
            .map(|rounds| std::vec![BattleEffect::SpellDuration { rounds }])
            .unwrap_or_default();
        let mut s = arena(
            &r,
            [Some(h), None],
            &[("fixed", 1, 3, 5)],
            &[("fixed", 1, 13, 5)],
            1,
        );
        act(&mut s, &r, cast(&r, "skin", stack(0))).unwrap();
        entry(&s, &r, 0, "skin").unwrap().rounds
    };
    assert_eq!(rounds(3, None), Some(3));
    assert_eq!(rounds(3, Some(56)), Some(59));
    assert_eq!(rounds(0, None), Some(1));
}

// covers: BAT-MAG-009, BAT-MAG-011
#[test]
fn counters_end_each_other_and_other_pairs_add_up() {
    let r = ruleset();
    let book = [
        "haste", "sloth", "lust", "weaken", "mirth", "sorrow", "bless", "curse",
    ];
    let mut s = arena(
        &r,
        [Some(adept(&r, &book, 2)), Some(adept(&r, &book, 2))],
        &[("fixed", 1, 3, 5)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    let morale = s.stacks()[0].morale;
    // Round 1: the attacker hastes its stack, the defender slows it: Haste ends (batch 2b R5,
    // like Bloodlust and Weakness staying together below).
    act(&mut s, &r, cast(&r, "haste", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].speed_now(), 10);
    activate(&mut s, 1);
    let events = act(&mut s, &r, cast(&r, "sloth", stack(0))).unwrap();
    assert_eq!(
        timed_events(&r, &events),
        [(0, "haste".into(), Some(0)), (0, "sloth".into(), Some(3))]
    );
    // Speed 5 at 50 %: 2 (rounded down).
    assert_eq!(s.stacks()[0].speed_now(), 2);
    // Round 2: Bloodlust and Weakness, round 3: Mirth and Sorrow, both pairs stay together.
    for (own, enemy) in [("lust", "weaken"), ("mirth", "sorrow"), ("bless", "curse")] {
        new_round(&mut s, &r);
        activate(&mut s, 0);
        act(&mut s, &r, cast(&r, own, stack(0))).unwrap();
        activate(&mut s, 1);
        act(&mut s, &r, cast(&r, enemy, stack(0))).unwrap();
    }
    let fixed = s.stacks()[0];
    // Bloodlust +6 with Weakness −6: the melee attack is the stack's again (10 + the hero's 3).
    assert!(entry(&s, &r, 0, "lust").is_some() && entry(&s, &r, 0, "weaken").is_some());
    assert_eq!((fixed.attack_now(false), fixed.attack_now(true)), (13, 7));
    // Mirth +2 with Sorrow −2.
    assert!(entry(&s, &r, 0, "mirth").is_some() && entry(&s, &r, 0, "sorrow").is_some());
    assert_eq!((fixed.spells.morale, fixed.morale), (0, morale));
    // Curse ended Bless: the stack is cursed.
    assert!(entry(&s, &r, 0, "bless").is_none());
    assert_eq!(fixed.spells.damage, DamageRoll::Minimum(1));
}

// covers: BAT-MAG-009
#[test]
fn a_recast_keeps_the_first_value_and_the_longer_duration() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["haste"], 0)), None],
        &[("fixed", 1, 3, 5), ("fast", 1, 3, 7)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "haste", stack(0))).unwrap();
    new_round(&mut s, &r);
    assert_eq!(entry(&s, &r, 0, "haste").unwrap().rounds, Some(2));
    // Now at Expert: the mass version renews stack 0 with its first value and hastes stack 1.
    set_masteries(&mut s, 0, &r, [3, 0, 0, 0]);
    activate(&mut s, 0);
    let events = act(&mut s, &r, cast(&r, "haste", CastTarget::None)).unwrap();
    assert_eq!(
        timed_events(&r, &events),
        [(0, "haste".into(), Some(3)), (1, "haste".into(), Some(3))]
    );
    assert_eq!(entry(&s, &r, 0, "haste").unwrap().value, 3);
    assert_eq!(entry(&s, &r, 1, "haste").unwrap().value, 5);
    assert_eq!(
        (s.stacks()[0].speed_now(), s.stacks()[1].speed_now()),
        (8, 14)
    );
}

// covers: BAT-MAG-009, BAT-MAG-010, BAT-MAG-011
#[test]
fn weakness_and_disrupting_ray_stop_at_zero_and_stone_skin_can_leave_a_negative_defense() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [
            Some(adept(&r, &["weaken", "ray"], 3)),
            Some(adept(&r, &["skin"], 0)),
        ],
        &[("fixed", 1, 3, 5)],
        &[("slow", 1, 13, 5), ("tank", 1, 13, 7)],
        1,
    );
    // Weakness at Expert takes 6, but the slow creature has 4 attack (1 + its hero's 3), the
    // tank 3.
    act(&mut s, &r, cast(&r, "weaken", CastTarget::None)).unwrap();
    assert_eq!(entry(&s, &r, 1, "weaken").unwrap().attack, -4);
    assert_eq!(entry(&s, &r, 2, "weaken").unwrap().attack, -3);
    assert_eq!(s.stacks()[1].attack_now(false), 0);
    assert_eq!(s.stacks()[2].attack_now(false), 0);
    // The defender's Stone Skin (+3) on the slow creature: defense 1 + 2 (hero) + 3.
    activate(&mut s, 1);
    act(&mut s, &r, cast(&r, "skin", stack(1))).unwrap();
    assert_eq!(s.stacks()[1].defense_now(), 6);
    // Disrupting Ray (5 at Expert, no mass version) lowers it to 1, then to 0, cumulative and
    // permanent.
    for expected in [1, 0] {
        new_round(&mut s, &r);
        activate(&mut s, 0);
        let events = act(&mut s, &r, cast(&r, "ray", stack(1))).unwrap();
        assert_eq!(timed_events(&r, &events), [(1, "ray".into(), None)]);
        assert_eq!(s.stacks()[1].defense_now(), expected);
    }
    assert_eq!(entry(&s, &r, 1, "ray").unwrap().defense, -6);
    // Stone Skin ends after its three rounds (cast in round 1): the defense is −3, and the ray
    // stays.
    new_round(&mut s, &r);
    assert!(entry(&s, &r, 1, "skin").is_none());
    assert_eq!(s.stacks()[1].defense_now(), -3);
    for _ in 0..300 {
        new_round(&mut s, &r);
    }
    assert!(entry(&s, &r, 1, "ray").is_some());
    // Weakness gives back what it took.
    assert!(entry(&s, &r, 1, "weaken").is_none());
    assert_eq!(s.stacks()[1].attack_now(false), 4);
}

// covers: BAT-MAG-003, BAT-MAG-005, BAT-MAG-009
#[test]
fn mass_versions_reach_their_side_and_spare_the_immune() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [
            Some(adept(&r, &["haste", "prayer", "mirth", "sorrow"], 3)),
            None,
        ],
        &[("fixed", 1, 3, 5), ("statue", 1, 3, 7)],
        &[("fixed", 1, 13, 5), ("statue", 1, 13, 7)],
        1,
    );
    // Below Expert a stack target; at Expert none.
    assert_eq!(
        act(&mut s, &r, cast(&r, "haste", stack(0))),
        Err(CommandError::InvalidSpellTarget)
    );
    let morale = s.stacks()[0].morale;
    let events = act(&mut s, &r, cast(&r, "mirth", CastTarget::None)).unwrap();
    // The statue is immune to mind spells; the enemy is not reached.
    assert_eq!(timed_events(&r, &events), [(0, "mirth".into(), Some(3))]);
    assert_eq!(s.stacks()[0].morale, morale + 2);
    // A mind spell cannot be aimed at a mind-immune stack.
    set_masteries(&mut s, 0, &r, [2, 0, 0, 0]);
    new_round(&mut s, &r);
    activate(&mut s, 0);
    assert_eq!(
        act(&mut s, &r, cast(&r, "sorrow", stack(3))),
        Err(CommandError::InvalidSpellTarget)
    );
    let events = act(&mut s, &r, cast(&r, "sorrow", stack(2))).unwrap();
    assert_eq!(timed_events(&r, &events), [(2, "sorrow".into(), Some(3))]);
}

/// E04b T11b review (F-r1-3): a spell without a target is cast only while some stack in its
/// reach would be affected with a chance above zero. The original's book asks its target test
/// of every hex before anything is paid and refuses the spell when no stack passes (the
/// reviewer's reading of 0x59ED5A to 0x59EE03 with the factor of 0x5A39C0). The engine asked
/// only for a stack in reach: the cast took the mana and the round's cast, and every stack
/// resisted.
// covers: BAT-MAG-003, BAT-MAG-008
#[test]
fn a_spell_without_a_target_needs_a_stack_it_would_affect() {
    let r = ruleset();
    // The attacker's hero has the mass Slow (Expert) and the battle-wide storm; the defender's
    // hero keeps `resistance` of every hostile spell off its two stacks. Ids: 0 the attacker's
    // `own` stack; 1 and 2 the defender's.
    let start = |own: &'static str, resistance: u32, seed: u64| {
        let mut warded = hero(&r);
        warded.effects = std::vec![BattleEffect::MagicResistance {
            rate: Rate::bp(resistance)
        }];
        arena(
            &r,
            [Some(adept(&r, &["sloth", "storm"], 3)), Some(warded)],
            &[(own, 1, 3, 5)],
            &[("fixed", 1, 13, 5), ("fixed", 1, 13, 7)],
            seed,
        )
    };
    let castable = |s: &BattleState| -> Vec<h3_content::SpellId> {
        castable_spells(s, &r, BattleSide::Attacker)
            .iter()
            .collect()
    };
    let spent = |s: &BattleState| {
        let h = s.hero(BattleSide::Attacker).unwrap();
        (h.mana, h.casts)
    };
    let (sloth, storm) = (spell(&r, "sloth"), spell(&r, "storm"));
    // The enemy's stacks always resist and the caster's wyrm is immune to level-1 spells: no
    // stack would be affected. Neither spell is castable or has a target, a cast is refused and
    // nothing is spent.
    let mut s = start("wyrm", 10_000, 1);
    assert_eq!(castable(&s), []);
    let mut targets = CastTargets::new();
    for name in ["sloth", "storm"] {
        cast_targets(&s, &r, BattleSide::Attacker, spell(&r, name), &mut targets);
        assert!(targets.is_empty(), "{name}");
        let before = s.clone();
        assert_eq!(
            act(&mut s, &r, cast(&r, name, CastTarget::None)),
            Err(CommandError::InvalidSpellTarget),
            "{name}"
        );
        assert_eq!(s, before, "{name}");
    }
    assert_eq!(spent(&s), (100, 0));
    // One stack that accepts the storm, the caster's own, makes it castable: that stack is
    // struck and the enemy's resist. The mass Slow reaches the enemy only and stays refused.
    let mut s = start("fixed", 10_000, 1);
    assert_eq!(castable(&s), [storm]);
    let events = act(&mut s, &r, cast(&r, "storm", CastTarget::None)).unwrap();
    let reached: Vec<(u8, bool)> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage { target, .. } => Some((target.0, true)),
            EventKind::SpellResisted { target } => Some((target.0, false)),
            _ => None,
        })
        .collect();
    assert_eq!(reached, [(0, true), (1, false), (2, false)]);
    assert_eq!(spent(&s), (95, 1));
    // A stack that can only resist by a roll counts: with 99 % resistance each enemy stack
    // keeps a chance of 1 %. Both spells are castable; the mass Slow is paid for, and each of
    // the two stacks rolls: nearly always it resists, now and then it is slowed.
    let (mut slowed, mut resisted) = (0, 0);
    for seed in 1..=300 {
        let mut s = start("wyrm", 9_900, seed);
        assert_eq!(castable(&s), [storm, sloth]);
        cast_targets(&s, &r, BattleSide::Attacker, sloth, &mut targets);
        assert_eq!(targets.as_slice(), [CastTarget::None]);
        let events = act(&mut s, &r, cast(&r, "sloth", CastTarget::None)).unwrap();
        assert_eq!(spent(&s), (96, 1));
        slowed += timed_events(&r, &events).len();
        resisted += events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::SpellResisted { .. }))
            .count();
    }
    assert_eq!(slowed + resisted, 600);
    assert!((1..=20).contains(&slowed), "{slowed}");
}

// covers: BAT-MAG-009, BAT-MAG-005
#[test]
fn war_machines_get_prayers_attack_and_defense_but_no_speed() {
    let r = ruleset();
    let mut h = adept(&r, &["prayer", "haste"], 0);
    h.war_machines.ballista = None;
    let mut setup = crate::BattleSetup {
        heroes: [Some(h), None],
        ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    setup.war_machines[0] = std::vec![r.creature_by_key("t:ballista").unwrap()];
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    let ballista = (0u8..)
        .zip(s.stacks())
        .find(|(_, st)| st.has(crate::StackTraits::WAR_MACHINE))
        .unwrap()
        .0;
    activate(&mut s, 0);
    // Haste excludes war machines.
    assert_eq!(
        act(&mut s, &r, cast(&r, "haste", stack(ballista))),
        Err(CommandError::InvalidSpellTarget)
    );
    act(&mut s, &r, cast(&r, "prayer", stack(ballista))).unwrap();
    let b = s.stacks()[usize::from(ballista)];
    let e = entry(&s, &r, ballista, "prayer").unwrap();
    assert_eq!((e.attack, e.defense, e.speed), (2, 2, 0));
    assert_eq!(
        (
            b.attack_now(true) - i32::from(b.attack),
            b.defense_now() - i32::from(b.defense)
        ),
        (2, 2)
    );
    assert_eq!(b.speed_now(), b.speed);
}

// covers: BAT-MAG-010
#[test]
fn a_stack_that_dies_loses_its_effects() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(mage(&r, &["skin"])), None],
        &[("fixed", 1, 3, 5), ("fast", 1, 3, 7)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "skin", stack(1))).unwrap();
    assert_eq!(s.effects().len(), 1);
    // Its death (set directly here) takes the effect away with the next command; a round that
    // starts in between leaves the dead stack's effects alone.
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    new_round(&mut s, &r);
    assert_eq!(entry(&s, &r, 1, "skin").unwrap().rounds, Some(3));
    activate(&mut s, 0);
    act(&mut s, &r, Command::Defend).unwrap();
    assert!(s.effects().is_empty());
}

// covers: BAT-MAG-011
#[test]
fn bloodlust_and_precision_count_for_their_kind_of_attack_and_bless_fixes_the_damage() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["lust", "aim", "bless", "curse"], 0)), None],
        &[("archer", 10, 3, 5)],
        &[("fixed", 10, 8, 5)],
        1,
    );
    // The archers' attack is 6 + the hero's 3.
    act(&mut s, &r, cast(&r, "lust", stack(0))).unwrap();
    let archer = s.stacks()[0];
    assert_eq!((archer.attack_now(false), archer.attack_now(true)), (12, 9));
    new_round(&mut s, &r);
    activate(&mut s, 0);
    act(&mut s, &r, cast(&r, "aim", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].attack_now(true), 12);
    // Bless at mastery none: every creature deals its maximum (3); 10 archers with attack 12
    // against defense 10: 30 × 1.1 = 33 for the whole range.
    new_round(&mut s, &r);
    activate(&mut s, 0);
    act(&mut s, &r, cast(&r, "bless", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].spells.damage, DamageRoll::Maximum(0));
    assert_eq!(
        crate::combat::shot_damage_range(&s, &r, StackId(0), StackId(1)),
        (33, 33)
    );
    let at = s.stacks()[1].hex;
    let events = act(
        &mut s,
        &r,
        Command::Shoot {
            target: StackId(1),
            at,
        },
    )
    .unwrap();
    let damage: Vec<u64> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck { damage, .. } => Some(damage),
            _ => None,
        })
        .collect();
    assert_eq!(damage, [33]);
    assert_eq!(DamageRoll::Minimum(1).fixed(2, 3), Some(1));
    assert_eq!(DamageRoll::Minimum(5).fixed(2, 3), Some(1));
    assert_eq!(DamageRoll::Maximum(1).fixed(2, 3), Some(4));
    assert_eq!(DamageRoll::Random.fixed(2, 3), None);
}

// covers: BAT-MAG-011
#[test]
fn slow_rounds_down_and_leaves_at_least_one() {
    assert_eq!(slowed(5, 75, false), 3);
    assert_eq!(slowed(5, 50, true), 2);
    assert_eq!(slowed(1, 50, false), 1);
    assert_eq!(slowed(9, 100, true), 9);
    // A share without an exact single-precision value (0.7 → 0.699999988) under Q-0005.
    assert_eq!((slowed(10, 70, false), slowed(10, 70, true)), (7, 6));
}

// covers: BAT-MAG-011
#[test]
fn morale_and_luck_spells_join_the_sum_before_the_minimums() {
    let r = ruleset();
    let mut h = adept(&r, &["fortune", "mirth"], 3);
    h.effects = std::vec![
        BattleEffect::Morale {
            amount: 4,
            affects: Affects::Own,
        },
        BattleEffect::Luck {
            amount: -2,
            affects: Affects::Own,
        },
    ];
    let mut defender = adept(&r, &["sorrow", "misfortune"], 0);
    defender.effects = std::vec![];
    let mut s = arena(
        &r,
        [Some(h), Some(defender)],
        &[("fixed", 1, 3, 5), ("halfling", 1, 3, 7)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    // Luck −2 + Fortune 2 at Expert: 0; the halfling keeps its minimum 1.
    act(&mut s, &r, cast(&r, "fortune", CastTarget::None)).unwrap();
    assert_eq!((s.stacks()[0].luck, s.stacks()[1].luck), (0, 1));
    // Morale 4 with Sorrow's −1: 3. (This pinned 2, a clamp to 3 before the spell that the
    // original does not make: review r2a F-1.)
    activate(&mut s, 2);
    act(&mut s, &r, cast(&r, "sorrow", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].morale, 3);
    // The other side's Misfortune on its own enemy brings the halfling's luck to its minimum.
    new_round(&mut s, &r);
    activate(&mut s, 2);
    act(&mut s, &r, cast(&r, "misfortune", stack(1))).unwrap();
    assert_eq!(s.stacks()[1].luck, 1);
}

// covers: BAT-MAG-011, BAT-MOR-001, BAT-MOR-003, BAT-LCK-001, BAT-LCK-002
#[test]
fn morale_and_luck_beyond_the_range_absorb_the_spells_and_are_clamped_once() {
    // Good morale and luck come at +3 only, and then always: the rolls show the value they read.
    let r = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.luck_dice = 1;
        b.good_morale_chances = std::vec![0, 0, 255];
        b.good_luck_chances = std::vec![0, 0, 1];
    });
    // The attacker's hero changes its stacks' morale by `morale` and gives them luck +5; its
    // one alignment adds 1 to the morale.
    let start = |morale: i8| {
        let mut h = adept(&r, &["mirth"], 3);
        h.effects = std::vec![
            BattleEffect::Morale {
                amount: morale,
                affects: Affects::Own,
            },
            BattleEffect::Luck {
                amount: 5,
                affects: Affects::Own,
            },
        ];
        let mut foe = adept(&r, &["sorrow", "misfortune"], 3);
        foe.effects = std::vec![];
        let (ours, theirs) = ([("fixed", 5, 3, 5)], [("fixed", 5, 4, 5)]);
        arena(&r, [Some(h), Some(foe)], &ours, &theirs, 1)
    };
    // Base morale 4 + 1 = 5 with Expert Sorrow's −2: 3. The original clamps the sum once,
    // where it rolls (review r2a F-1); clamped before the spell it came to 3 − 2 = 1.
    let mut s = start(4);
    activate(&mut s, 1);
    act(&mut s, &r, cast(&r, "sorrow", CastTarget::None)).unwrap();
    assert_eq!(s.stacks()[0].morale, 3);
    // Base luck 5 with Expert Misfortune's −2: 3, not 1.
    new_round(&mut s, &r);
    activate(&mut s, 1);
    act(&mut s, &r, cast(&r, "misfortune", CastTarget::None)).unwrap();
    assert_eq!(s.stacks()[0].luck, 3);
    // The observation shows the clamped values, and the rolls read them: a lucky strike, then
    // another turn.
    let seen = observe(&s, &r, BattleSide::Defender).stacks.as_slice()[0];
    assert_eq!((seen.morale, seen.luck), (3, 3));
    activate(&mut s, 0);
    let from = s.stacks()[0].hex;
    let target = StackId(1);
    let events = act(&mut s, &r, Command::Attack { target, from }).unwrap();
    let lucky = |e: &Event| {
        matches!(e.kind, EventKind::Struck { attacker: StackId(0), flags, .. }
            if flags.contains(crate::event::StrikeFlags::LUCKY))
    };
    assert!(events.iter().any(lucky));
    let again = EventKind::GoodMorale { stack: StackId(0) };
    assert!(events.iter().any(|e| e.kind == again));
    // Base morale −6 + 1 = −5 with Expert Mirth's +2: −3, not −3 + 2 = −1.
    let mut s = start(-6);
    act(&mut s, &r, cast(&r, "mirth", CastTarget::None)).unwrap();
    assert_eq!(s.stacks()[0].morale, -3);
}

// covers: BAT-MAG-009
#[test]
fn effects_are_public_and_hashed() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(mage(&r, &["skin"])), None],
        &[("fixed", 1, 3, 5)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    let before = s.state_hash();
    act(&mut s, &r, cast(&r, "skin", stack(0))).unwrap();
    assert_ne!(s.state_hash(), before);
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&s, &r, viewer).effects, s.effects());
    }
    // A fork keeps the reserve and the entries.
    let fork = s.clone();
    assert_eq!(fork, s);
    assert!(fork.effects.as_slice().len() == 1);
    // Castable lists the timed spell while the hero has a cast left.
    let fresh = arena(
        &r,
        [Some(mage(&r, &["skin"])), None],
        &[("fixed", 1, 3, 5)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    assert!(castable_spells(&fresh, &r, BattleSide::Attacker).contains(spell(&r, "skin")));
}

// covers: BAT-MAG-011
#[test]
fn the_hash_sees_every_spell_value() {
    use h3_kernel::hash::hash_of;
    let base = crate::SpellValues::default();
    let variants = [
        crate::SpellValues { attack: -1, ..base },
        crate::SpellValues { defense: 1, ..base },
        crate::SpellValues { speed: 1, ..base },
        crate::SpellValues {
            melee_attack: 1,
            ..base
        },
        crate::SpellValues {
            ranged_attack: 1,
            ..base
        },
        crate::SpellValues {
            damage: DamageRoll::Maximum(0),
            ..base
        },
        crate::SpellValues {
            damage: DamageRoll::Minimum(0),
            ..base
        },
        crate::SpellValues {
            speed_percent: 50,
            ..base
        },
        crate::SpellValues { morale: 1, ..base },
        crate::SpellValues { luck: -1, ..base },
    ];
    let digests: Vec<u64> = variants.iter().map(hash_of).collect();
    for (i, d) in digests.iter().enumerate() {
        assert_ne!(*d, hash_of(&base));
        assert!(!digests[..i].contains(d));
    }
}
