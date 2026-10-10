//! Creatures that cast as their stack's action (E04b T7b1, T7b2a; BAT-MAG-048, BAT-MAG-049),
//! after the original's code (`docs/formats/installation.md` §7.21).

use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::BattleSide;

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::CastSpell::{AtResolution, Named};
use crate::state::StackFlags;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, Hex, StackId,
    Status, Visibility, creature_cast_spell, creature_cast_targets, observe, stack_spell,
};

/// The stacks the active stack of `side` may cast at now, by id.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide) -> Vec<u8> {
    let mut out = CastTargets::new();
    creature_cast_targets(s, r, side, &mut out);
    out.iter()
        .map(|t| match *t {
            CastTarget::Stack(id) => id.0,
            other => panic!("not a stack: {other:?}"),
        })
        .collect()
}

/// The command of a stack's cast at stack `id`.
fn at_stack(id: u8) -> Command {
    Command::CreatureCast { target: stack(id) }
}

/// The effects `events` apply, as `(target, spell, rounds)`.
fn applied(events: &[Event]) -> Vec<(u8, h3_content::SpellId, Option<u16>)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::EffectApplied {
                target,
                spell,
                rounds,
            } => Some((target.0, spell, rounds)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-048, BAT-MAG-009
#[test]
fn a_creatures_re_cast_keeps_the_first_value_as_a_heros_does() {
    // The creature's cast goes through the same effect routine (the original's 0x4448F0): a
    // re-cast keeps the value and takes the longer duration (Q47; the E04b T3a loose end).
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    let haste = spell(&r, "haste");
    let basic = crate::effects::TimedCast {
        id: haste,
        spell: r.spell(haste).unwrap(),
        mastery: 1,
        rounds: 1,
        own: crate::magic::SpellBonus::default(),
    };
    let speed = s.stacks()[1].speed_now();
    crate::effects::apply(&mut s, &r, basic, StackId(1), &mut Vec::<Event>::new());
    assert_eq!(s.stacks()[1].speed_now(), speed + 3);
    // The conjurer's Advanced Haste (+5, six rounds) lengthens the duration only.
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(applied(&events), vec![(1, haste, Some(6))]);
    assert_eq!(s.stacks()[1].speed_now(), speed + 3);
}

// covers: BAT-MAG-048
#[test]
fn a_caster_casts_its_spell_at_another_stack_of_its_side_and_ends_its_turn() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    let haste = spell(&r, "haste");
    assert_eq!(stack_spell(&s, &r, StackId(0)), Some(Named(haste)));
    assert_eq!(stack_spell(&s, &r, StackId(1)), None);
    assert_eq!(stack_spell(&s, &r, StackId(9)), None);
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(Named(haste))
    );
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Defender), None);
    // Not itself (the original's cursor), not the enemy (a positive spell).
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    assert!(targets(&s, &r, BattleSide::Defender).is_empty());
    assert_eq!((s.casts_left(StackId(0)), s.casts_left(StackId(1))), (2, 0));
    let speed = s.stacks()[1].speed_now();
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(
        events[0].kind,
        EventKind::StackCast {
            stack: StackId(0),
            spell: haste,
            target: stack(1),
        }
    );
    // Advanced: +5 speed (Haste's effect [3, 3, 5, 5]); power 6: six rounds; no hero needed.
    assert_eq!(applied(&events), vec![(1, haste, Some(6))]);
    assert_eq!(s.stacks()[1].speed_now(), speed + 5);
    // One cast used; the turn ends as after an action.
    assert_eq!(s.casts_left(StackId(0)), 1);
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_ne!(s.active(), Some(StackId(0)));
    assert_eq!(observe(&s, &r, BattleSide::Defender).casts_left[0], 1);
    assert!(events.iter().any(|e| e.kind
        == EventKind::TurnStarted {
            stack: s.active().unwrap()
        }));
}

// covers: BAT-MAG-048
#[test]
fn a_cast_needs_a_caster_with_casts_and_a_target_of_its_spell() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[
            ("conjurer", 5, 2, 3),
            ("fixed", 3, 2, 7),
            ("fixed", 3, 2, 9),
        ],
        &[("fixed", 3, 13, 5)],
        1,
    );
    let before = s.clone();
    for (target, error) in [
        (stack(0), CommandError::InvalidSpellTarget),
        (stack(3), CommandError::InvalidSpellTarget),
        (stack(40), CommandError::InvalidSpellTarget),
        (CastTarget::None, CommandError::InvalidSpellTarget),
        (CastTarget::Hex(Hex(30)), CommandError::InvalidSpellTarget),
    ] {
        let command = Command::CreatureCast { target };
        assert_eq!(act(&mut s, &r, command), Err(error), "{target:?}");
        assert_eq!(
            crate::apply(
                &mut s,
                &r,
                BattleSide::Defender,
                command,
                &mut Vec::<Event>::new()
            ),
            Err(CommandError::NotYourTurn)
        );
    }
    assert_eq!(s, before);
    // A dead stack is no target.
    s.stacks.as_mut_slice()[2].count = 0;
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    assert_eq!(
        act(&mut s, &r, at_stack(2)),
        Err(CommandError::InvalidSpellTarget)
    );
    // Without casts left it may not cast.
    s.casts.left[0] = 0;
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(act(&mut s, &r, at_stack(1)), Err(CommandError::CannotCast));
    // Nor may a stack that is no caster.
    activate(&mut s, 1);
    assert_eq!(act(&mut s, &r, at_stack(0)), Err(CommandError::CannotCast));
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
}

// covers: BAT-MAG-048
#[test]
fn a_hypnotized_caster_casts_for_its_controller() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 3, 2, 3), ("fixed", 3, 2, 7)],
        &[("conjurer", 5, 13, 5), ("fixed", 3, 13, 7)],
        1,
    );
    let st = &mut s.stacks.as_mut_slice()[2];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    activate(&mut s, 2);
    // The attacker commands it, and its spell reaches the attacker's stacks (the original's
    // side of the caster, +0x288).
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    assert!(targets(&s, &r, BattleSide::Defender).is_empty());
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![0, 1]);
    let events = act(&mut s, &r, at_stack(0)).unwrap();
    assert_eq!(applied(&events), vec![(0, spell(&r, "haste"), Some(6))]);
}

// covers: BAT-MAG-048
#[test]
fn the_stack_a_creature_casts_at_never_rolls_for_resistance() {
    let r = ruleset();
    let weaken = spell(&r, "weaken");
    for seed in 0..40 {
        let mut s = arena(
            &r,
            [None, None],
            &[("warlock", 5, 2, 3)],
            &[("dwarf", 5, 13, 5)],
            seed,
        );
        // Basic: −3 attack; power 4: four rounds; the dwarves' 20 % never helps them.
        let attack = s.stacks()[1].attack_now(false);
        let events = act(&mut s, &r, at_stack(1)).unwrap();
        assert_eq!(applied(&events), vec![(1, weaken, Some(4))], "seed {seed}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e.kind, EventKind::SpellResisted { .. }))
        );
        assert_eq!(s.stacks()[1].attack_now(false), attack - 3);
    }
}

// covers: BAT-MAG-048, BAT-MOR-003
#[test]
fn good_morale_may_follow_a_cast() {
    // Good morale that always triggers (BAT-MOR-003), as after a move, an attack or a shot (the
    // original's 0x464920).
    let r = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.good_morale_chances = vec![255, 255, 255];
    });
    let mut s = arena(
        &r,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].morale = 1;
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert_eq!(s.active(), Some(StackId(0)));
    // Its second cast in a row, its last.
    act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(s.casts_left(StackId(0)), 0);
    assert_ne!(s.active(), Some(StackId(0)));
}

// covers: BAT-MAG-048, BAT-MAG-038
#[test]
fn a_created_stack_has_its_creatures_casts() {
    let r = ruleset();
    let mut hero = mage(&r, &["copy"]);
    hero.magic.masteries = masteries(&r, [1, 0, 0, 0]);
    hero.magic.mana = 100;
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("conjurer", 5, 5, 5), ("fixed", 3, 2, 9)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    act(&mut s, &r, at_stack(1)).unwrap();
    activate(&mut s, 1);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "copy"),
            target: stack(0),
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert!(events.iter().any(|e| matches!(
        e.kind,
        EventKind::StackCreated {
            stack: StackId(3),
            ..
        }
    )));
    assert_eq!((s.casts_left(StackId(0)), s.casts_left(StackId(3))), (1, 2));
}

/// Activates stack `id` as the battle does for its side's command (a random caster draws its
/// spell, BAT-MAG-049), returning the events.
fn drawing(s: &mut BattleState, r: &Ruleset, id: u8) -> Vec<Event> {
    activate(s, id);
    let mut events = Vec::new();
    crate::casters::activated(s, r, StackId(id), &mut events);
    events
}

/// The hexes the active stack of `side` may cast at now, by index.
fn hexes(s: &BattleState, r: &Ruleset, side: BattleSide) -> Vec<u8> {
    let mut out = CastTargets::new();
    creature_cast_targets(s, r, side, &mut out);
    out.iter()
        .map(|t| match *t {
            CastTarget::Hex(h) => h.0,
            other => panic!("not a hex: {other:?}"),
        })
        .collect()
}

/// The spell damage `events` report, as `(target, value, damage)`.
fn damage(events: &[Event]) -> Vec<(u8, u64, u64)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage {
                target,
                value,
                damage,
                ..
            } => Some((target.0, value, damage)),
            _ => None,
        })
        .collect()
}

/// Puts the timed spell `name` at `mastery` on stack `id` (five rounds).
fn give(s: &mut BattleState, r: &Ruleset, (id, name): (u8, &str), mastery: u8) {
    let spell_id = spell(r, name);
    let cast = crate::effects::TimedCast {
        id: spell_id,
        spell: r.spell(spell_id).unwrap(),
        mastery,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

// covers: BAT-MAG-049
#[test]
fn a_random_caster_draws_its_spell_by_weight_and_only_its_controller_is_told() {
    let r = ruleset();
    // `dragonet`: bolt 22, nova 21, ring 10, chain 5 (of 58), in this order.
    let order = [
        (22, spell(&r, "bolt")),
        (43, spell(&r, "nova")),
        (53, spell(&r, "ring")),
        (58, spell(&r, "chain")),
    ];
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..60 {
        let mut s = arena(
            &r,
            [None, None],
            &[("dragonet", 2, 3, 5)],
            &[("fixed", 3, 13, 5)],
            seed,
        );
        let x = s.rng.clone().below(58);
        let expected = order.iter().find(|&&(below, _)| x < below).unwrap().1;
        let events = drawing(&mut s, &r, 0);
        assert_eq!(
            events,
            vec![Event {
                visibility: Visibility::Side(BattleSide::Attacker),
                kind: EventKind::SpellDrawn {
                    stack: StackId(0),
                    spell: expected,
                },
            }]
        );
        assert_eq!(crate::project(&events[0], BattleSide::Defender), None);
        assert_eq!(stack_spell(&s, &r, StackId(0)), Some(Named(expected)));
        assert_eq!(
            creature_cast_spell(&s, &r, BattleSide::Attacker),
            Some(Named(expected))
        );
        assert_eq!(observe(&s, &r, BattleSide::Attacker).drawn, Some(expected));
        assert_eq!(observe(&s, &r, BattleSide::Defender).drawn, None);
        // The draw is in the state hash (v34): another spell drawn hashes differently.
        let mut other = s.clone();
        let another = order.iter().map(|o| o.1).find(|&o| o != expected).unwrap();
        other.casts.drawn = Some((StackId(0), another));
        assert_ne!(other.state_hash(), s.state_hash());
        seen.insert(expected);
    }
    assert_eq!(seen.len(), 4);
}

// covers: BAT-MAG-049
#[test]
fn a_random_caster_without_casts_draws_nothing_and_another_activation_forgets_its_draw() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("boltling", 1, 3, 5), ("fixed", 3, 3, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    s.casts.left[0] = 0;
    assert!(drawing(&mut s, &r, 0).is_empty());
    assert_eq!(stack_spell(&s, &r, StackId(0)), None);
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
    s.casts.left[0] = 2;
    assert_eq!(drawing(&mut s, &r, 0).len(), 1);
    // Only while it is the active stack.
    activate(&mut s, 1);
    assert_eq!(stack_spell(&s, &r, StackId(0)), None);
    activate(&mut s, 0);
    assert_eq!(
        stack_spell(&s, &r, StackId(0)),
        Some(Named(spell(&r, "bolt")))
    );
    // Another stack's activation forgets it.
    assert!(drawing(&mut s, &r, 1).is_empty());
    activate(&mut s, 0);
    assert_eq!(stack_spell(&s, &r, StackId(0)), None);
    assert_eq!(observe(&s, &r, BattleSide::Attacker).drawn, None);
    assert_eq!(act(&mut s, &r, at_stack(2)), Err(CommandError::CannotCast));
}

// covers: BAT-MAG-049
#[test]
fn a_drawn_spell_is_aimed_at_any_stack_but_the_caster_that_it_can_affect() {
    let r = ruleset();
    for (caster, expected) in [
        // A negative spell: only enemies; the wyrm is immune to level 1.
        ("boltling", vec![2, 4]),
        // A neutral one (Chain Lightning): own stacks too, never the caster itself.
        ("chainling", vec![1, 2, 4]),
        // A positive one: only its own side's other stacks.
        ("hasteling", vec![1]),
    ] {
        let mut s = arena(
            &r,
            [None, None],
            &[(caster, 1, 3, 5), ("fixed", 3, 3, 7)],
            &[
                ("fixed", 3, 13, 5),
                ("wyrm", 3, 13, 7),
                ("dwarf", 3, 13, 9),
                ("fixed", 3, 13, 3),
            ],
            1,
        );
        s.stacks.as_mut_slice()[5].count = 0;
        drawing(&mut s, &r, 0);
        assert_eq!(targets(&s, &r, BattleSide::Attacker), expected, "{caster}");
        let before = s.clone();
        for target in [
            stack(0),
            stack(3),
            stack(5),
            stack(40),
            CastTarget::None,
            CastTarget::Hex(Hex(30)),
        ] {
            assert_eq!(
                act(&mut s, &r, Command::CreatureCast { target }),
                Err(CommandError::InvalidSpellTarget),
                "{caster} {target:?}"
            );
        }
        assert_eq!(s, before);
    }
    // A positive timed spell lasts its power: 5 per creature, two creatures.
    let mut s = arena(
        &r,
        [None, None],
        &[("hasteling", 2, 3, 5), ("fixed", 3, 3, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    drawing(&mut s, &r, 0);
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(applied(&events), vec![(1, spell(&r, "haste"), Some(10))]);
}

// covers: BAT-MAG-049
#[test]
fn a_drawn_area_spell_is_aimed_at_a_hex_of_a_stack_and_may_hit_the_caster() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("novaling", 2, 3, 5), ("ogre", 1, 5, 5)],
        &[("fixed", 3, 13, 5), ("ember", 3, 13, 7)],
        1,
    );
    drawing(&mut s, &r, 0);
    // The two-hex ogre's tail and head, the enemy's hex; not the caster's, not the fire-immune
    // ember's, never an empty hex.
    let hex = |x: i32, y: i32| at(&r, x, y).0;
    assert_eq!(
        hexes(&s, &r, BattleSide::Attacker),
        vec![hex(4, 5), hex(5, 5), hex(13, 5)]
    );
    for target in [
        stack(1),
        CastTarget::Hex(at(&r, 3, 5)),
        CastTarget::Hex(at(&r, 8, 5)),
    ] {
        assert_eq!(
            act(&mut s, &r, Command::CreatureCast { target }),
            Err(CommandError::InvalidSpellTarget)
        );
    }
    // Aimed at the ogre's tail beside the caster: the area reaches both; power 5 × 2 creatures:
    // 10 × 10 + 20 at Advanced.
    let events = act(
        &mut s,
        &r,
        Command::CreatureCast {
            target: CastTarget::Hex(at(&r, 4, 5)),
        },
    )
    .unwrap();
    assert_eq!(
        events[0].kind,
        EventKind::StackCast {
            stack: StackId(0),
            spell: spell(&r, "nova"),
            target: CastTarget::Hex(at(&r, 4, 5)),
        }
    );
    assert_eq!(damage(&events), vec![(0, 120, 120), (1, 120, 120)]);
    assert_eq!((s.stacks()[0].count, s.stacks()[1].count), (1, 0));
    assert_eq!(s.casts_left(StackId(0)), 1);
}

// covers: BAT-MAG-049, BAT-MAG-004
#[test]
fn a_random_casters_power_is_its_creatures_times_its_power_per_creature() {
    // Survey S-18 is not reproducible in SoD: a Faerie Dragon's Magic Arrow is at least 70 before a
    // golem's reduction, which leaves a Diamond Golem 3 and a Gold Golem 10.
    let r = ruleset();
    for (count, golem, expected) in [
        (1, "diamond", (70, 3)),
        (1, "goldgolem", (70, 10)),
        (3, "diamond", (170, 8)),
    ] {
        let mut s = arena(
            &r,
            [None, None],
            &[("boltling", count, 3, 5)],
            &[(golem, 3, 13, 5)],
            1,
        );
        drawing(&mut s, &r, 0);
        let events = act(&mut s, &r, at_stack(1)).unwrap();
        assert_eq!(
            damage(&events),
            vec![(1, expected.0, expected.1)],
            "{count} {golem}"
        );
    }
}

// covers: BAT-MAG-049
#[test]
fn a_hypnotized_random_caster_draws_for_its_controller() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 3, 2, 3)],
        &[("boltling", 1, 13, 5), ("fixed", 3, 13, 7)],
        1,
    );
    let st = &mut s.stacks.as_mut_slice()[1];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    let events = drawing(&mut s, &r, 1);
    assert_eq!(events[0].visibility, Visibility::Side(BattleSide::Attacker));
    assert!(observe(&s, &r, BattleSide::Attacker).drawn.is_some());
    assert_eq!(observe(&s, &r, BattleSide::Defender).drawn, None);
    // Its negative spell reaches the stacks of its owner's side, not itself.
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![2]);
}

// covers: BAT-MAG-049, BAT-MOR-003
#[test]
fn a_random_caster_draws_again_after_good_morale_and_after_waiting() {
    let always = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.good_morale_chances = vec![255, 255, 255];
    });
    let mut s = arena(
        &always,
        [None, None],
        &[("boltling", 1, 3, 5)],
        &[("fixed", 30, 13, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].morale = 1;
    drawing(&mut s, &always, 0);
    let events = act(&mut s, &always, at_stack(1)).unwrap();
    let drawn = |events: &[Event]| {
        events.iter().any(|e| {
            matches!(
                e.kind,
                EventKind::SpellDrawn {
                    stack: StackId(0),
                    ..
                }
            )
        })
    };
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert!(drawn(&events));
    // A waited stack draws again when it comes back in the round.
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("boltling", 1, 3, 5)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    drawing(&mut s, &r, 0);
    act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(s.active(), Some(StackId(1)));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(0)));
    assert!(drawn(&events));
    // The defender's command does not tell the defender.
    let told: Vec<Event> = events
        .iter()
        .filter_map(|e| crate::project(e, BattleSide::Defender))
        .collect();
    assert!(!drawn(&told));
}

// covers: BAT-MAG-049, BAT-MAG-022
#[test]
fn a_mirrored_random_cast_is_untested_with_quirk_bat_quirk_9() {
    for (profile, untested) in [("t:faithful", false), ("t:mirrored", true)] {
        let r = ruleset_with(profile, |_| {});
        // Where the mirror sends the bolt: to the caster itself or to its own immune wyrm, or
        // its own dwarves (20 % resistance).
        let mut outcomes = std::collections::BTreeSet::new();
        for (own, seed) in [("wyrm", 0..30), ("dwarf", 30..90)]
            .into_iter()
            .flat_map(|(own, seeds)| seeds.map(move |seed| (own, seed)))
        {
            let mut s = arena(
                &r,
                [None, None],
                &[("boltling", 1, 3, 5), (own, 3, 3, 7)],
                &[("fixed", 3, 13, 5)],
                seed,
            );
            give(&mut s, &r, (2, "mirror"), 3);
            drawing(&mut s, &r, 0);
            let events = act(&mut s, &r, at_stack(2)).unwrap();
            let to = events
                .iter()
                .find_map(|e| match e.kind {
                    EventKind::SpellRedirected { from, to } => {
                        assert_eq!(from, StackId(2));
                        Some(to.0)
                    }
                    _ => None,
                })
                .unwrap();
            let resisted = events.iter().any(|e| {
                e.kind
                    == EventKind::SpellResisted {
                        target: StackId(to),
                    }
            });
            let hit = !damage(&events).is_empty();
            assert_ne!(resisted, hit, "{profile} {own} {seed}");
            match (to, own) {
                // The caster is not immune and has no resistance.
                (0, _) => assert!(hit),
                // Immune: hit only without a test.
                (_, "wyrm") => assert_eq!(hit, untested, "{profile} {seed}"),
                // Resistance: never resisted without a test.
                _ => assert!(hit || !untested, "{profile} {seed}"),
            }
            outcomes.insert((to, own, hit));
        }
        // Both stacks were chosen; without the quirk the dwarves sometimes resisted.
        assert!(outcomes.contains(&(0, "wyrm", true)));
        assert!(outcomes.contains(&(1, "wyrm", untested)));
        assert!(outcomes.contains(&(1, "dwarf", true)));
        assert_eq!(outcomes.contains(&(1, "dwarf", false)), !untested);
    }
}

/// Whether `name` is useful on stack `target` for a helpful caster of the attacker (BAT-MAG-050).
fn is_useful(s: &BattleState, r: &Ruleset, name: &str, target: u8) -> bool {
    let id = spell(r, name);
    crate::casters::useful(
        s,
        r,
        BattleSide::Attacker,
        (id, r.spell(id).unwrap()),
        StackId(target),
    )
}

// covers: BAT-MAG-050
#[test]
fn a_helpful_caster_casts_a_spell_drawn_among_those_useful_for_its_target() {
    let r = ruleset();
    // For an own melee stack against an enemy melee stack, without heroes, wounds or kings:
    // Shield, Bless, Bloodlust and Prayer, in the jinn's order.
    let useful = ["shield", "bless", "lust", "prayer"].map(|n| spell(&r, n));
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..30 {
        let mut s = arena(
            &r,
            [None, None],
            &[("jinn", 2, 3, 5), ("fixed", 3, 3, 7)],
            &[("fixed", 3, 13, 5)],
            seed,
        );
        // Nothing is drawn at the activation; the cast names no spell.
        assert!(drawing(&mut s, &r, 0).is_empty());
        assert_eq!(stack_spell(&s, &r, StackId(0)), Some(AtResolution));
        assert_eq!(
            creature_cast_spell(&s, &r, BattleSide::Attacker),
            Some(AtResolution)
        );
        assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
        let mut counts = [0; crate::STACK_CAPACITY];
        counts[1] = 4;
        assert_eq!(
            observe(&s, &r, BattleSide::Attacker).cast_candidates,
            counts
        );
        assert_eq!(
            observe(&s, &r, BattleSide::Defender).cast_candidates,
            [0; crate::STACK_CAPACITY]
        );
        let expected = useful[s.rng.clone().below(4) as usize];
        let events = act(&mut s, &r, at_stack(1)).unwrap();
        assert_eq!(
            events[0].kind,
            EventKind::StackCast {
                stack: StackId(0),
                spell: expected,
                target: stack(1),
            }
        );
        // Advanced, power 6: six rounds.
        assert_eq!(applied(&events), vec![(1, expected, Some(6))]);
        assert_eq!(s.casts_left(StackId(0)), 2);
        seen.insert(expected);
    }
    assert_eq!(seen.len(), 4);
}

// covers: BAT-MAG-050
#[test]
fn a_helpful_caster_needs_another_own_stack_with_a_useful_spell() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("jinn", 2, 3, 5), ("fixed", 3, 3, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    s.stacks.as_mut_slice()[1].count = 0;
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(
        observe(&s, &r, BattleSide::Attacker).cast_candidates,
        [0; crate::STACK_CAPACITY]
    );
    let before = s.clone();
    for target in [
        stack(0),
        stack(1),
        stack(2),
        stack(40),
        CastTarget::None,
        CastTarget::Hex(at(&r, 3, 7)),
    ] {
        assert_eq!(
            act(&mut s, &r, Command::CreatureCast { target }),
            Err(CommandError::InvalidSpellTarget),
            "{target:?}"
        );
    }
    assert_eq!(s, before);
}

// covers: BAT-MAG-050
#[test]
fn a_spell_is_useful_as_the_originals_test_says() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("jinn", 2, 3, 5), ("fixed", 3, 3, 7), ("sniper", 3, 3, 9)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    // Always, unless active, for an own stack it can affect.
    assert!(is_useful(&s, &r, "bless", 1));
    assert!(!is_useful(&s, &r, "bless", 3));
    let mut warded = s.clone();
    give(&mut warded, &r, (1, "antimagic"), 3);
    assert!(!is_useful(&warded, &r, "bless", 1));
    give(&mut warded, &r, (2, "bless"), 0);
    assert!(!is_useful(&warded, &r, "bless", 2));
    // An enemy that cannot shoot: Shield, not Air Shield.
    assert!(is_useful(&s, &r, "shield", 1));
    assert!(!is_useful(&s, &r, "airshield", 1));
    // Bloodlust for a stack that cannot shoot, Precision for one that can.
    let sniper = 2;
    assert!(is_useful(&s, &r, "lust", 1) && !is_useful(&s, &r, "aim", 1));
    assert!(is_useful(&s, &r, "aim", sniper) && !is_useful(&s, &r, "lust", sniper));
    // A blocked shooter fights in melee.
    let mut blocked = s.clone();
    place(&mut blocked, &r, 3, 4, 9);
    assert!(is_useful(&blocked, &r, "lust", sniper) && !is_useful(&blocked, &r, "aim", sniper));
    // Prayer before the stack acted; Cure for a wounded top creature.
    assert!(is_useful(&s, &r, "prayer", 1));
    assert!(!is_useful(&s, &r, "cure", 1));
    let st = &mut s.stacks.as_mut_slice()[1];
    st.flags.set(StackFlags::ACTED, true);
    st.top_hp -= 1;
    assert!(!is_useful(&s, &r, "prayer", 1));
    assert!(is_useful(&s, &r, "cure", 1));
    // No king among the enemies.
    assert!(!is_useful(&s, &r, "slayer", 1));
}

// covers: BAT-MAG-050
#[test]
fn the_enemy_decides_the_shields_protections_and_slayer() {
    let r = ruleset();
    let enemy = |name: &'static str, heroes: [Option<h3_contracts::BattleHero>; 2]| {
        arena(
            &r,
            heroes,
            &[("jinn", 2, 3, 5), ("fixed", 3, 3, 7)],
            &[(name, 3, 13, 5)],
            1,
        )
    };
    // A shooter that can shoot: Air Shield, not Shield; neither once it cannot fight (blind,
    // petrified or paralysed) or is dead, or is a war machine that heals or supplies.
    let mut s = enemy("sniper", [None, None]);
    assert!(is_useful(&s, &r, "airshield", 1) && !is_useful(&s, &r, "shield", 1));
    for change in 0..3 {
        let mut t = s.clone();
        let st = &mut t.stacks.as_mut_slice()[2];
        match change {
            0 => st.spells.status = st.spells.status.with(Status::NO_TURNS, true),
            1 => st.count = 0,
            _ => st.traits = st.traits.with(crate::state::StackTraits::FIRST_AID, true),
        }
        assert!(!is_useful(&t, &r, "airshield", 1) && !is_useful(&t, &r, "shield", 1));
    }
    let st = &mut s.stacks.as_mut_slice()[2];
    st.traits = st.traits.with(crate::state::StackTraits::AMMO_SUPPLY, true);
    assert!(!is_useful(&s, &r, "airshield", 1));
    // The protections need an enemy hero with a spell book.
    for (heroes, expected) in [
        ([None, None], false),
        ([None, Some(hero(&r))], false),
        ([None, Some(mage(&r, &["bolt"]))], true),
    ] {
        let s = enemy("fixed", heroes);
        assert_eq!(is_useful(&s, &r, "airward", 1), expected);
        assert_eq!(
            s.hero(BattleSide::Defender).is_some_and(|h| h.spell_book),
            expected
        );
    }
    // Slayer: an enemy stack of the first king level, even a dead one; not the second alone.
    let mut s = enemy("lord", [None, None]);
    assert!(is_useful(&s, &r, "slayer", 1));
    s.stacks.as_mut_slice()[2].count = 0;
    assert!(is_useful(&s, &r, "slayer", 1));
    assert!(!is_useful(&enemy("duke", [None, None]), &r, "slayer", 1));
}
