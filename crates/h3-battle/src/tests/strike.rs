//! Spells after a melee strike (E04b T7a1): the strike's spell (BAT-MAG-040), the damage spell
//! after a strike (BAT-MAG-041) and Dispel Helpful Spells (BAT-MAG-042), after the original's
//! code (`docs/formats/installation.md` §7.20).
//!
//! The test creatures are 10/10 with damage 2 and 10 HP: `blinder` casts `t:blind` after every
//! strike, `hexer` `t:curse` with 25 % on living stacks, `fly` `t:unbless` and then `t:weaken`
//! at Advanced unless active, `stormbird` deals 10 per creature as `t:bolt`, `twinblinder`
//! strikes twice and blinds, `weakener` casts `t:weaken`.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::{Ruleset, SpellId};

use super::spells::{arena, spell};
use super::support::*;
use crate::effects::TimedCast;
use crate::magic::SpellBonus;
use crate::{BattleState, Command, Event, EventKind, StackId, Status};

/// The active stack attacks `target` from `(x, y)`.
fn hit(s: &mut BattleState, r: &Ruleset, target: u8, (x, y): (i32, i32)) -> Vec<Event> {
    act(
        s,
        r,
        Command::Attack {
            target: StackId(target),
            from: at(r, x, y),
        },
    )
    .unwrap()
}

/// Gives stack `id` the timed effect of `name` at `mastery` for five rounds.
fn give(s: &mut BattleState, r: &Ruleset, (id, name): (u8, &str), mastery: u8) {
    let spell_id = spell(r, name);
    let cast = TimedCast {
        id: spell_id,
        spell: r.spell(spell_id).unwrap(),
        mastery,
        rounds: 5,
        own: SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The strike spells `events` report, as `(striker, target, spell name)`.
fn strike_spells(r: &Ruleset, events: &[Event]) -> Vec<(u8, u8, String)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::StrikeSpell {
                striker,
                target,
                spell,
            } => Some((striker.0, target.0, name(r, spell))),
            _ => None,
        })
        .collect()
}

fn name(r: &Ruleset, spell: SpellId) -> String {
    r.spell(spell).unwrap().key.as_str()[2..].into()
}

/// The strikes `events` report, as `(attacker, target)`.
fn strikes(events: &[Event]) -> Vec<(u8, u8)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker, target, ..
            } => Some((attacker.0, target.0)),
            _ => None,
        })
        .collect()
}

/// The names of the spells whose effects stack `id` has, sorted, with their rounds and values.
fn on(s: &BattleState, r: &Ruleset, id: u8) -> Vec<(String, Option<u16>, u16)> {
    let mut list: Vec<_> = s
        .effects()
        .iter()
        .filter(|e| e.stack == StackId(id))
        .map(|e| (name(r, e.spell), e.rounds, e.value))
        .collect();
    list.sort();
    list
}

/// A battle of `attacker` ×5 at (7, 5), active, against `defender` ×10 at (8, 5).
fn duel(r: &Ruleset, attacker: &'static str, defender: &'static str, seed: u64) -> BattleState {
    arena(
        r,
        [None, None],
        &[(attacker, 5, 7, 5)],
        &[(defender, 10, 8, 5)],
        seed,
    )
}

// ---------------------------------------------------------------- BAT-MAG-040

// covers: BAT-MAG-040
#[test]
fn a_strike_blinds_its_target_at_mastery_0_for_its_rounds_and_stops_its_retaliation() {
    let r = ruleset();
    let mut s = duel(&r, "blinder", "fixed", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    // The strike, then the spell; no retaliation from the blinded stack.
    assert_eq!(strikes(&events), vec![(0, 1)]);
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 1, String::from("blind"))]
    );
    // Mastery 0: Blind's 50 %, as long as the ability's rounds (one went by: the blinded stack
    // passed its turn and a round began).
    assert!(events.iter().any(|e| matches!(
        e.kind,
        EventKind::EffectApplied {
            target: StackId(1),
            rounds: Some(3),
            ..
        }
    )));
    assert_eq!(on(&s, &r, 1), vec![(String::from("blind"), Some(2), 50)]);
    assert!(s.at(StackId(1)).spells.status.contains(Status::NO_TURNS));
    assert_eq!(s.at(StackId(1)).retaliations_left, 1);
}

// covers: BAT-MAG-040
#[test]
fn a_retaliation_casts_too_and_a_blinded_attacker_loses_its_second_strike() {
    let r = ruleset();
    let mut s = duel(&r, "twin", "blinder", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    // The twin strikes, the blinder retaliates and blinds it: no second strike.
    assert_eq!(strikes(&events), vec![(0, 1), (1, 0)]);
    assert_eq!(
        strike_spells(&r, &events),
        vec![(1, 0, String::from("blind"))]
    );
    // A twin that blinds strikes twice: its own spell does not stop it.
    let mut s = duel(&r, "twinblinder", "fixed", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strikes(&events), vec![(0, 1), (0, 1)]);
}

// covers: BAT-MAG-040
#[test]
fn shots_cast_nothing_and_a_killed_target_takes_no_spell() {
    let r = ruleset();
    // The archer shoots a blinder: no retaliation, no spell either way.
    let mut s = arena(
        &r,
        [None, None],
        &[("archer", 10, 2, 5)],
        &[("blinder", 10, 12, 5)],
        1,
    );
    let shot = Command::Shoot {
        target: StackId(1),
        at: at(&r, 12, 5),
    };
    let events = act(&mut s, &r, shot).unwrap();
    assert!(strike_spells(&r, &events).is_empty());
    // Five blinders kill a single serf: nothing to blind.
    let mut s = arena(
        &r,
        [None, None],
        &[("blinder", 5, 7, 5)],
        &[("serf", 1, 8, 5)],
        1,
    );
    let events = hit(&mut s, &r, 1, (7, 5));
    assert!(strike_spells(&r, &events).is_empty());
}

// covers: BAT-MAG-040
#[test]
fn the_chance_is_drawn_per_strike_and_living_spells_spare_other_stacks() {
    let r = ruleset();
    let curses = (0..400u64)
        .filter(|&seed| {
            let mut s = duel(&r, "hexer", "serf", seed);
            !strike_spells(&r, &hit(&mut s, &r, 1, (7, 5))).is_empty()
        })
        .count();
    // 25 % of 400 = 100 (σ ≈ 8.7).
    assert!((75..=125).contains(&curses), "{curses}");
    // `golem` is not a living creature: never cursed.
    let cursed = (0..100u64).any(|seed| {
        let mut s = duel(&r, "hexer", "golem", seed);
        !strike_spells(&r, &hit(&mut s, &r, 1, (7, 5))).is_empty()
    });
    assert!(!cursed);
}

// covers: BAT-MAG-040
#[test]
fn immunities_anti_magic_and_resistance_apply_as_to_a_heros_spell() {
    let r = ruleset();
    // A mind-immune stack, an undead one (Blind spares them) and one under Anti-Magic.
    for target in ["statue", "ghoul"] {
        let mut s = duel(&r, "blinder", target, 1);
        assert!(strike_spells(&r, &hit(&mut s, &r, 1, (7, 5))).is_empty());
    }
    // Anti-Magic at Expert keeps level 1 off.
    let mut s = duel(&r, "weakener", "fixed", 1);
    give(&mut s, &r, (1, "antimagic"), 3);
    assert!(strike_spells(&r, &hit(&mut s, &r, 1, (7, 5))).is_empty());
    // Dwarves resist 20 %: blinded about 80 % of the time.
    let blinded = (0..400u64)
        .filter(|&seed| {
            let mut s = duel(&r, "blinder", "dwarf", seed);
            !strike_spells(&r, &hit(&mut s, &r, 1, (7, 5))).is_empty()
        })
        .count();
    // 80 % of 400 = 320 (σ = 8).
    assert!((295..=345).contains(&blinded), "{blinded}");
}

// covers: BAT-MAG-040
#[test]
fn a_hypnotized_striker_blinds_a_stack_of_its_own_side() {
    let r = ruleset();
    // The attacker's blinder, hypnotized, strikes the attacker's fixed stack.
    let mut s = arena(
        &r,
        [None, None],
        &[("blinder", 5, 7, 5), ("fixed", 10, 8, 5)],
        &[("serf", 10, 14, 9)],
        1,
    );
    give(&mut s, &r, (0, "hypnotize"), 3);
    activate(&mut s, 0);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 1, String::from("blind"))]
    );
}

// ---------------------------------------------------------------- BAT-MAG-042

// covers: BAT-MAG-040, BAT-MAG-042
#[test]
fn a_fly_dispels_positive_spells_and_weakens_at_advanced_unless_weakened() {
    let r = ruleset();
    let mut s = duel(&r, "fly", "fixed", 1);
    give(&mut s, &r, (1, "skin"), 1);
    give(&mut s, &r, (1, "curse"), 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![
            (0, 1, String::from("unbless")),
            (0, 1, String::from("weaken"))
        ]
    );
    // Stone Skin ended, Curse stays; Weakness at Advanced (6) for three rounds.
    assert_eq!(
        on(&s, &r, 1),
        vec![
            (String::from("curse"), Some(5), 0),
            (String::from("weaken"), Some(3), 6)
        ]
    );
    // A second strike on it: no positive spell left, Weakness already active.
    activate(&mut s, 0);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert!(strike_spells(&r, &events).is_empty());
    // Anti-Magic is positive: Dispel Helpful Spells ends it first, then Weakness lands.
    let mut s = duel(&r, "fly", "fixed", 1);
    give(&mut s, &r, (1, "antimagic"), 3);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strike_spells(&r, &events).len(), 2);
    assert_eq!(on(&s, &r, 1), vec![(String::from("weaken"), Some(3), 6)]);
}

// ---------------------------------------------------------------- BAT-MAG-041

/// The damage events of `events` as `(target, value, damage, kills)`.
fn spell_damage(events: &[Event]) -> Vec<(u8, u64, u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage {
                target,
                value,
                damage,
                kills,
                ..
            } => Some((target.0, value, damage, kills)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-041
#[test]
fn a_thunderbolt_deals_its_damage_per_creature_as_the_spell_without_a_hero() {
    let r = ruleset();
    // Five stormbirds: 10 each, 50 as `t:bolt`; the strike took 10 of the 100 hit points.
    let mut s = duel(&r, "stormbird", "fixed", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(spell_damage(&events), vec![(1, 50, 50, 5)]);
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 1, String::from("bolt"))]
    );
    // The target's factors: double against `ember`, a quarter against `golem`.
    let mut s = duel(&r, "stormbird", "ember", 1);
    assert_eq!(
        spell_damage(&hit(&mut s, &r, 1, (7, 5))),
        vec![(1, 50, 100, 9)]
    );
    let mut s = duel(&r, "stormbird", "golem", 1);
    assert_eq!(
        spell_damage(&hit(&mut s, &r, 1, (7, 5))),
        vec![(1, 50, 12, 1)]
    );
    // `wyrm` is immune to the spell's level.
    let mut s = duel(&r, "stormbird", "wyrm", 1);
    assert!(spell_damage(&hit(&mut s, &r, 1, (7, 5))).is_empty());
}

// ---------------------------------------------------------------- BAT-MAG-043, BAT-MAG-044

/// Starts a new round.
fn new_round(s: &mut BattleState, r: &Ruleset) -> Vec<Event> {
    let mut events = Vec::new();
    crate::turn::start_round(s, r, &mut events);
    events
}

/// The strikes `events` report, as `(attacker, target, damage)`.
fn damages(events: &[Event]) -> Vec<(u8, u8, u64)> {
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

/// `attacker` ×5 at (7, 5) and `fixed` ×5 at (9, 5) against `defender` ×10 at (8, 5); the first
/// attacker strikes it.
fn struck(
    r: &Ruleset,
    attacker: &'static str,
    defender: &'static str,
) -> (BattleState, Vec<Event>) {
    let mut s = arena(
        r,
        [None, None],
        &[(attacker, 5, 7, 5), ("fixed", 5, 9, 5)],
        &[(defender, 10, 8, 5)],
        1,
    );
    let events = hit(&mut s, r, 2, (7, 5));
    (s, events)
}

// covers: BAT-MAG-043
#[test]
fn stone_gaze_stops_turns_halves_the_next_strike_and_ends_with_it() {
    let r = ruleset();
    let (mut s, events) = struck(&r, "gazer", "fixed");
    assert_eq!(damages(&events), vec![(0, 2, 10)]);
    let status = s.at(StackId(2)).spells.status;
    assert!(status.contains(Status::NO_TURNS) && status.contains(Status::PETRIFIED));
    // Five more strike it for half (5 of 10); the damage ends the petrification, so it
    // retaliates as usual (the original's 0x444090 ends it before the retaliation test).
    activate(&mut s, 1);
    let events = hit(&mut s, &r, 2, (9, 5));
    assert_eq!(damages(&events), vec![(1, 2, 5), (2, 1, 18)]);
    assert!(on(&s, &r, 2).is_empty());
}

// covers: BAT-MAG-043
#[test]
fn a_paralysed_stack_loses_its_turns_and_retaliates_with_a_quarter() {
    let r = ruleset();
    let (mut s, events) = struck(&r, "stinger", "fixed");
    assert_eq!(damages(&events), vec![(0, 2, 10)]);
    assert!(s.at(StackId(2)).spells.status.contains(Status::NO_TURNS));
    // Struck for 10 (one dies), the 8 left retaliate with 25 % of 16; the paralysis ends.
    activate(&mut s, 1);
    let events = hit(&mut s, &r, 2, (9, 5));
    assert_eq!(damages(&events), vec![(1, 2, 10), (2, 1, 4)]);
    assert!(on(&s, &r, 2).is_empty());
    // Blind's 50 % and Paralyze's 25 %: the smaller share.
    let (mut s, _) = struck(&r, "stinger", "fixed");
    give(&mut s, &r, (2, "blind"), 1);
    activate(&mut s, 1);
    assert_eq!(damages(&hit(&mut s, &r, 2, (9, 5)))[1], (2, 1, 4));
}

// covers: BAT-MAG-043
#[test]
fn a_petrified_or_paralysed_attacker_makes_no_second_strike() {
    let r = ruleset();
    for retaliator in ["gazer", "stinger"] {
        let mut s = duel(&r, "twin", retaliator, 1);
        let events = hit(&mut s, &r, 1, (7, 5));
        assert_eq!(strikes(&events), vec![(0, 1), (1, 0)], "{retaliator}");
    }
}

// covers: BAT-MAG-009, BAT-MAG-040
#[test]
fn disease_takes_two_attack_and_defense_from_living_stacks_for_its_rounds() {
    let r = ruleset();
    let (mut s, _) = struck(&r, "rotter", "serf");
    assert_eq!(
        (
            s.at(StackId(2)).attack_now(false),
            s.at(StackId(2)).defense_now()
        ),
        (8, 8)
    );
    for _ in 0..3 {
        new_round(&mut s, &r);
    }
    assert_eq!(
        (
            s.at(StackId(2)).attack_now(false),
            s.at(StackId(2)).defense_now()
        ),
        (10, 10)
    );
    // Not on a stack of a creature that is not living.
    let (_, events) = struck(&r, "rotter", "golem");
    assert!(strike_spells(&r, &events).is_empty());
}

// covers: BAT-MAG-044
#[test]
fn age_halves_the_hit_points_per_creature_while_it_lasts() {
    let r = ruleset();
    // Three ancients deal 6 to ten serfs of 10 hit points: the top one keeps 4.
    let mut s = arena(
        &r,
        [None, None],
        &[("ancient", 3, 7, 5)],
        &[("serf", 10, 8, 5)],
        1,
    );
    hit(&mut s, &r, 1, (7, 5));
    // Aged: 10 × ½ + 0.95 = 5.95 → 5 per creature; the top one keeps at least 1 (it lost 6).
    let serfs = s.at(StackId(1));
    assert_eq!((serfs.max_hp, serfs.top_hp, serfs.count), (5, 1, 10));
    for _ in 0..3 {
        new_round(&mut s, &r);
    }
    // Age ended: 10 per creature again, the top one having lost 4 of the 5.
    let serfs = s.at(StackId(1));
    assert_eq!((serfs.max_hp, serfs.top_hp), (10, 6));
}

/// The `Poisoned` events of `events`, as `(stack, hit points)`.
fn poisoned(events: &[Event]) -> Vec<(u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Poisoned { stack, hit_points } => Some((stack.0, hit_points)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-044
#[test]
fn poison_takes_a_tenth_at_the_strike_and_each_round_and_stays_until_a_cure() {
    let r = ruleset();
    let mut cleric = super::spells::mage(&r, &["cure"]);
    cleric.magic.mana = 50;
    let mut s = arena(
        &r,
        [None, Some(cleric)],
        &[("wyvern", 5, 7, 5)],
        &[("serf", 10, 8, 5)],
        1,
    );
    hit(&mut s, &r, 1, (7, 5));
    // 10 × 0.9 + 0.95 → 9, then 8 and 7 at the next two round starts; it ends at the third.
    assert_eq!(s.at(StackId(1)).max_hp, 9);
    let rounds: Vec<_> = (0..3).map(|_| poisoned(&new_round(&mut s, &r))).collect();
    assert_eq!(rounds, vec![vec![(1, 8)], vec![(1, 7)], vec![]]);
    assert!(on(&s, &r, 1).is_empty());
    assert_eq!(s.at(StackId(1)).max_hp, 7);
    // A second Poison while one lasts takes nothing more; the factor stops at half.
    activate(&mut s, 0);
    hit(&mut s, &r, 1, (7, 5));
    activate(&mut s, 0);
    hit(&mut s, &r, 1, (7, 5));
    assert_eq!(s.at(StackId(1)).max_hp, 6);
    for _ in 0..4 {
        new_round(&mut s, &r);
    }
    assert_eq!(s.at(StackId(1)).max_hp, 5);
    // A Cure gives the hit points back.
    activate(&mut s, 1);
    let cure = super::spells::cast(&r, "cure", super::spells::stack(1));
    act(&mut s, &r, cure).unwrap();
    assert_eq!(s.at(StackId(1)).max_hp, 10);
    // A clone starts from the start value, without the original's poison.
    let (mut s, _) = struck(&r, "wyvern", "serf");
    crate::created::clone(&mut s, &r, (StackId(2), 3), &mut Vec::<Event>::new());
    assert_eq!(s.at(StackId(3)).max_hp, 10);
}
