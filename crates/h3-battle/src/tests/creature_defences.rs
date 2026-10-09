//! The creature defences (E04b T7c2a): the Faerie Dragon's own Magic Mirror (BAT-MAG-057), the
//! Efreet Sultan's own Fire Shield (BAT-MAG-058) and the Psychic and Magic Elementals' half
//! damage (BAT-DMG-010), after the original's code (`docs/formats/installation.md` §7.21).

use std::collections::BTreeSet;
use std::vec;
use std::vec::Vec;

use h3_content::{Ruleset, SpellScope};
use h3_contracts::{BattleEffect, BattleHero, BattleSetup, BattleSide, Rate};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::{BattleState, CastTargets, Command, Event, EventKind, StackId, cast_targets};

/// Gives stack `id` the timed effect of `name` at `mastery` for five rounds.
fn give(s: &mut BattleState, r: &Ruleset, (id, name): (u8, &str), mastery: u8) {
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

/// A mage with `spells`, the air mastery `mastery` and enough mana.
fn adept(r: &Ruleset, spells: &[&str], mastery: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = mastery;
    h.magic.mana = 200;
    h
}

/// The redirections `(from, to)` in `events`.
fn redirected(events: &[Event]) -> Vec<(u8, u8)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellRedirected { from, to } => Some((from.0, to.0)),
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

/// The damage of the first strike in `events`.
fn first_strike(events: &[Event]) -> u64 {
    events
        .iter()
        .find_map(|e| match e.kind {
            EventKind::Struck { damage, .. } => Some(damage),
            _ => None,
        })
        .unwrap()
}

/// A battle of `attacker` against `defender` without heroes, the stacks at `positions`
/// (stack-id order), the first attacker stack active.
fn field_of(
    r: &Ruleset,
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
fn hit(s: &mut BattleState, r: &Ruleset, target: u8, (x, y): (i32, i32)) -> Vec<Event> {
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

// covers: BAT-MAG-057
#[test]
fn an_innate_mirror_gives_its_chance_unless_the_spell_gives_more() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["hypnotize"], 3)), None],
        &[("fixed", 5, 3, 5)],
        &[
            ("fairy", 5, 13, 5),
            ("fixed", 5, 13, 7),
            ("pixie", 1, 13, 9),
        ],
        1,
    );
    let chance = |s: &BattleState, id: u8| crate::magic::mirror_chance(&r, s.at(StackId(id)));
    assert_eq!((chance(&s, 1), chance(&s, 2), chance(&s, 3)), (20, 0, 100));
    // The creature's own mirror is not the spell: the stack's spell value stays 0, and Hypnotize
    // (which refuses a stack with the spell, 0x5A83A6) still takes the Faerie Dragons.
    assert_eq!(s.stacks()[1].spells.mirror, 0);
    let mut out = CastTargets::new();
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "hypnotize"),
        &mut out,
    );
    assert!(out.as_slice().contains(&stack(1)));
    // The spell at no mastery: 50 % beats the dragon's 20 %, and Hypnotize refuses it now.
    give(&mut s, &r, (1, "mirror"), 0);
    assert_eq!(chance(&s, 1), 50);
    let mut out = CastTargets::new();
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "hypnotize"),
        &mut out,
    );
    assert!(!out.as_slice().contains(&stack(1)));
    // A smaller spell value leaves the larger creature's own.
    give(&mut s, &r, (3, "mirror"), 0);
    assert_eq!(chance(&s, 3), 100);
}

// covers: BAT-MAG-057, BAT-MAG-022
#[test]
fn a_faerie_dragons_mirror_sends_a_heros_spell_away_at_its_chance() {
    let r = ruleset();
    let casts = 300;
    let mut sent = 0;
    for seed in 0..casts {
        let mut s = arena(
            &r,
            [Some(adept(&r, &["bolt"], 0)), None],
            &[("tank", 5, 3, 5)],
            &[("fairy", 5, 13, 5)],
            seed,
        );
        let events = act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
        match redirected(&events).as_slice() {
            [] => {}
            // Sent to the caster's only stack.
            [(1, 0)] => sent += 1,
            other => panic!("{other:?}"),
        }
    }
    // 20 % of 300: 60.
    assert!((35..=85).contains(&sent), "{sent} of {casts}");
    // A stack without a mirror of its own or the spell never sends a spell away.
    let mut s = arena(
        &r,
        [Some(adept(&r, &["bolt"], 0)), None],
        &[("tank", 5, 3, 5)],
        &[("tank", 5, 13, 5)],
        1,
    );
    let events = act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    assert!(redirected(&events).is_empty());
}

// covers: BAT-MAG-057, BAT-MAG-049
#[test]
fn a_creatures_spell_at_a_dragon_is_sent_away_untested_with_quirk_bat_quirk_9() {
    for (profile, untested) in [("t:faithful", false), ("t:mirrored", true)] {
        let r = ruleset_with(profile, |_| {});
        let mut outcomes = BTreeSet::new();
        for seed in 0..24 {
            // The pixie's own mirror (100 %) sends the bolt to the caster or to its wyrm,
            // immune to level 1.
            let mut s = arena(
                &r,
                [None, None],
                &[("boltling", 1, 3, 5), ("wyrm", 3, 3, 7)],
                &[("pixie", 3, 13, 5)],
                seed,
            );
            activate(&mut s, 0);
            let events = act(&mut s, &r, Command::CreatureCast { target: stack(2) }).unwrap();
            let to = match redirected(&events).as_slice() {
                [(2, to)] => *to,
                other => panic!("{other:?}"),
            };
            let hit = events.iter().any(|e| {
                matches!(e.kind, EventKind::SpellDamage { target, .. } if target == StackId(to))
            });
            outcomes.insert((to, hit));
        }
        // The caster is hit; the wyrm only without a test.
        assert_eq!(
            outcomes,
            BTreeSet::from([(0, true), (1, untested)]),
            "{profile}"
        );
    }
}

/// The reflection of 100 Fixed (stage 1: 200, dealt: 150) striking a lone Sultan (100 hit
/// points, its own shield 25 %) with the spell cast on it at `mastery` (none: not).
fn sultan_reflects(r: &Ruleset, mastery: Option<u8>) -> Vec<(u8, u64, u32)> {
    let mut s = field_of(
        r,
        &[(0, "fixed", 100)],
        &[(0, "sultan", 1)],
        &[(7, 5), (8, 5)],
    );
    if let Some(m) = mastery {
        give(&mut s, r, (1, "fireshield"), m);
    }
    reflected(&hit(&mut s, r, 1, (7, 5)))
}

// covers: BAT-MAG-058, BAT-MAG-018
#[test]
fn an_efreet_sultans_shield_reflects_unless_the_spell_reflects_more() {
    let r = ruleset();
    // 25 % of the 100 hit points: 25, two Fixed and 5 hit points.
    assert_eq!(sultan_reflects(&r, None), [(0, 25, 2)]);
    // The spell at Basic (20 %) reflects less: the Sultan's own 25 %.
    assert_eq!(sultan_reflects(&r, Some(1)), [(0, 25, 2)]);
    // At Expert (30 %) the spell's.
    assert_eq!(sultan_reflects(&r, Some(3)), [(0, 30, 3)]);
    // With quirk Q-0010 the first stage, capped at the Sultan's hit points: the same here; in the
    // original's single precision too.
    for profile in ["t:fireshield", "t:original"] {
        let r = ruleset_with(profile, |_| {});
        assert_eq!(sultan_reflects(&r, None), [(0, 25, 2)], "{profile}");
    }
    // Shots and fire-immune strikers are not burnt.
    let mut s = field_of(
        &r,
        &[(0, "archer", 10)],
        &[(0, "sultan", 1)],
        &[(3, 5), (12, 5)],
    );
    let shot = Command::Shoot {
        target: StackId(1),
        at: s.stacks()[1].hex,
    };
    let events = act(&mut s, &r, shot).unwrap();
    assert!(first_strike(&events) > 0);
    assert!(reflected(&events).is_empty());
    let mut s = field_of(
        &r,
        &[(0, "salamander", 10)],
        &[(0, "sultan", 1)],
        &[(7, 5), (8, 5)],
    );
    let events = hit(&mut s, &r, 1, (7, 5));
    assert!(first_strike(&events) > 0);
    assert!(reflected(&events).is_empty());
    // A stack with a mirror of its own reflects nothing.
    let mut s = field_of(
        &r,
        &[(0, "fixed", 10)],
        &[(0, "pixie", 1)],
        &[(7, 5), (8, 5)],
    );
    assert!(reflected(&hit(&mut s, &r, 1, (7, 5))).is_empty());
}

// covers: BAT-MAG-058, BAT-MAG-004
#[test]
fn the_sultans_shield_is_the_shields_spell_damage_with_its_heros_bonus() {
    let r = ruleset();
    // The Sultan's hero adds 50 % to its spells (its attack and defense are 0).
    let mut h = hero(&r);
    (h.attack, h.defense) = (0, 0);
    h.effects = vec![BattleEffect::SpellDamage {
        scope: SpellScope::All,
        rate: Rate::bp(5000),
    }];
    let setup = BattleSetup {
        heroes: [None, Some(h)],
        ..setup(&r, &[(0, "fixed", 10)], &[(0, "sultan", 1)])
    };
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    place(&mut s, &r, 0, 7, 5);
    place(&mut s, &r, 1, 8, 5);
    activate(&mut s, 0);
    // 10 Fixed deal 15: 25 % is 3, × 1.5 = 4.
    assert_eq!(reflected(&hit(&mut s, &r, 1, (7, 5))), [(0, 4, 0)]);
}

/// The damage of 10 `striker` (attack 10, damage 2) on a lone `target`, in `profile`.
fn strike(profile: &str, striker: &'static str, target: &'static str) -> u64 {
    let r = ruleset_with(profile, |_| {});
    let mut s = field_of(
        &r,
        &[(0, striker, 10)],
        &[(0, target, 1)],
        &[(7, 5), (8, 5)],
    );
    first_strike(&hit(&mut s, &r, 1, (7, 5)))
}

// covers: BAT-DMG-010
#[test]
fn the_elementals_deal_half_damage_to_their_targets() {
    for profile in ["t:faithful", "t:original"] {
        // 20 against defense 10; half against a mind-immune statue, or a psychic.
        assert_eq!(strike(profile, "psychic", "fixed"), 20, "{profile}");
        assert_eq!(strike(profile, "psychic", "statue"), 10, "{profile}");
        assert_eq!(strike(profile, "psychic", "psychic"), 10, "{profile}");
        // Half against its own kind and the Tank, not against others.
        assert_eq!(strike(profile, "magical", "magical"), 10, "{profile}");
        assert_eq!(strike(profile, "magical", "statue"), 20, "{profile}");
        // With the defense factor in the same stage: 20 × 0.75 × 0.5 = 7.5 against the Tank
        // (defense 20), 15 for the others.
        assert_eq!(strike(profile, "magical", "tank"), 7, "{profile}");
        assert_eq!(strike(profile, "fixed", "tank"), 15, "{profile}");
        // The striker's penalty only: others strike them in full.
        assert_eq!(strike(profile, "fixed", "psychic"), 20, "{profile}");
    }
}
