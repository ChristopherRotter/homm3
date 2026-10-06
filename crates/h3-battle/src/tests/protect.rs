//! Protective timed spells (E04b T3b1): Shield and Air Shield (BAT-MAG-012), the Protections
//! with quirk Q-0007 (BAT-MAG-013), Slayer (BAT-MAG-014) and Counterstrike (BAT-MAG-015).

use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::BattleHero;

use super::spells::{arena, cast, mage, stack};
use super::support::*;
use crate::{BattleState, Command, Event, EventKind, StackFlags, StackId};

/// A mage with `spells`, the air and fire masteries `air` and `fire` and enough mana.
fn adept(r: &Ruleset, spells: &[&str], (air, fire): (u8, u8)) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = air;
    h.magic.schools[1] = fire;
    h.magic.mana = 100;
    h
}

/// Starts a new round and gives the turn to stack `id`.
fn next_round(s: &mut BattleState, r: &Ruleset, id: u8) {
    let mut events: Vec<Event> = Vec::new();
    crate::turn::start_round(s, r, &mut events);
    activate(s, id);
}

/// The damage of the strikes and of the spells in `events`, in order.
fn damage(events: &[Event]) -> Vec<u64> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck { damage, .. } | EventKind::SpellDamage { damage, .. } => {
                Some(damage)
            }
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-012
#[test]
fn shield_and_air_shield_cut_melee_and_ranged_damage() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, Some(adept(&r, &["shield", "airshield"], (0, 0)))],
        &[("fixed", 10, 3, 5), ("archer", 10, 2, 8)],
        &[("fixed", 10, 4, 5)],
        1,
    );
    // The defender's hero (defense 2) shields its stack: 85 % of melee damage at mastery none.
    activate(&mut s, 2);
    act(&mut s, &r, cast(&r, "shield", stack(2))).unwrap();
    assert_eq!(s.stacks()[2].spells.melee_protection, 85);
    assert_eq!(s.stacks()[2].spells.ranged_protection, 100);
    // The archers (attack 6, 2–3 damage) against defense 12: 20–30 base, −15 %.
    let shot = |s: &BattleState| crate::combat::shot_damage_range(s, &r, StackId(1), StackId(2));
    assert_eq!(shot(&s), (17, 25), "Shield does not touch shots");
    next_round(&mut s, &r, 2);
    act(&mut s, &r, cast(&r, "airshield", stack(2))).unwrap();
    assert_eq!(s.stacks()[2].spells.ranged_protection, 75);
    // 20 × 0.85 × 0.75 = 12.75; 30 × 0.6375 = 19.125.
    assert_eq!(shot(&s), (12, 19));
    // A melee strike: 20 base, attack 10 against 12 (−5 %), then Shield: 16.15.
    activate(&mut s, 0);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 3, 5),
        },
    )
    .unwrap();
    assert_eq!(damage(&events)[0], 16);
}

/// The damage of `bolt` and of `smite` (air and fire) on a stack with Protection from Air (50 %)
/// and Protection from Fire (70 %) under `profile`.
fn protected_damage(profile: &str) -> (u64, u64) {
    let r = ruleset_with(profile, |_| {});
    let mut s = arena(
        &r,
        [
            Some(adept(&r, &["bolt", "smite"], (0, 0))),
            Some(adept(&r, &["airward", "fireward"], (0, 0))),
        ],
        &[("fixed", 1, 3, 5)],
        &[("tank", 10, 13, 5)],
        1,
    );
    activate(&mut s, 1);
    act(&mut s, &r, cast(&r, "airward", stack(1))).unwrap();
    next_round(&mut s, &r, 1);
    act(&mut s, &r, cast(&r, "fireward", stack(1))).unwrap();
    assert_eq!(
        s.stacks()[1].spells.school_protection[..2],
        [50, 70],
        "by school handle: air, fire"
    );
    // The attacker's hero: power 3, mastery none: 3 × 10 + 10 = 40.
    activate(&mut s, 0);
    let bolt = damage(&act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap())[0];
    next_round(&mut s, &r, 0);
    let smite = damage(&act(&mut s, &r, cast(&r, "smite", stack(1))).unwrap())[0];
    (bolt, smite)
}

// covers: BAT-MAG-013
#[test]
fn a_spell_of_two_schools_meets_one_protection() {
    // An air spell meets Protection from Air. Smite (air and fire) meets the strongest
    // protection without quirk Q-0007, with it the first by school rank (fire before air in the
    // test pack); under Q-0005 as well, 40 × f32(0.7) is just below 28.
    assert_eq!(protected_damage("t:faithful"), (20, 20));
    assert_eq!(protected_damage("t:first"), (20, 28));
    assert_eq!(protected_damage("t:original"), (20, 27));
}

// covers: BAT-MAG-014
#[test]
fn slayer_counts_against_the_king_levels_its_mastery_reaches() {
    let r = ruleset();
    // Archers (attack 6 + 3) shoot at defense 10: −2.5 % on 20–30, or with Slayer's +8 +35 %.
    let plain = (19, 29);
    let slain = (27, 40);
    for (mastery, expected) in [
        (0, [slain, plain, plain, plain]),
        (1, [slain, plain, plain, plain]),
        (2, [slain, slain, plain, plain]),
        (3, [slain, slain, slain, plain]),
    ] {
        let mut s = arena(
            &r,
            [Some(adept(&r, &["slayer"], (mastery, 0))), None],
            &[("archer", 10, 3, 5)],
            &[
                ("lord", 1, 8, 2),
                ("duke", 1, 8, 4),
                ("prince", 1, 8, 6),
                ("fixed", 1, 8, 8),
            ],
            1,
        );
        let before: Vec<_> = (1..=4)
            .map(|t| crate::combat::shot_damage_range(&s, &r, StackId(0), StackId(t)))
            .collect();
        assert_eq!(before, [plain; 4]);
        act(&mut s, &r, cast(&r, "slayer", stack(0))).unwrap();
        let after: Vec<_> = (1..=4)
            .map(|t| crate::combat::shot_damage_range(&s, &r, StackId(0), StackId(t)))
            .collect();
        assert_eq!(after, expected, "mastery {mastery}");
        assert_eq!(s.stacks()[0].spells.slayer, (8, mastery));
    }
}

// covers: BAT-MAG-015
#[test]
fn counterstrike_adds_retaliations_now_and_at_each_round_start() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["counter"], (0, 0))), None],
        &[("fixed", 1, 3, 5), ("royal", 1, 3, 7)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    // A stack that has used its retaliation gets one back at once.
    let st = &mut s.stacks.as_mut_slice()[0];
    st.retaliations_left = 0;
    st.flags.set(StackFlags::RETALIATED, true);
    act(&mut s, &r, cast(&r, "counter", stack(0))).unwrap();
    let st = s.stacks()[0];
    assert_eq!(st.retaliations_left, 1);
    assert!(!st.flags.contains(StackFlags::RETALIATED));
    // Each round start gives 1 + 1; a re-cast adds nothing (it renews the three rounds of
    // power 3), and the round in which the spell runs out still has the extra one.
    next_round(&mut s, &r, 0);
    assert_eq!(s.stacks()[0].retaliations_left, 2);
    act(&mut s, &r, cast(&r, "counter", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].retaliations_left, 2);
    next_round(&mut s, &r, 0);
    next_round(&mut s, &r, 0);
    assert!(!s.effects().is_empty());
    next_round(&mut s, &r, 0);
    assert_eq!(s.stacks()[0].retaliations_left, 2);
    assert!(
        s.effects().is_empty(),
        "the spell ran out at this round start"
    );
    next_round(&mut s, &r, 0);
    assert_eq!(s.stacks()[0].retaliations_left, 1);
    // Unlimited retaliations stay unlimited.
    next_round(&mut s, &r, 1);
    let unlimited = s.stacks()[1].retaliations_left;
    act(&mut s, &r, cast(&r, "counter", stack(1))).unwrap();
    assert_eq!(s.stacks()[1].retaliations_left, unlimited);
    next_round(&mut s, &r, 1);
    assert_eq!(s.stacks()[1].retaliations_left, unlimited);
}
