//! Spells after a melee strike (E04b T7a): the strike's spell (BAT-MAG-040), the damage spell
//! after a strike (BAT-MAG-041), Dispel Helpful Spells (BAT-MAG-042), the strike's statuses
//! (BAT-MAG-043, BAT-MAG-044), Bind (BAT-MAG-045), Acid Breath (BAT-MAG-046) and the extra damage
//! (BAT-MAG-047), after the original's code (`docs/formats/installation.md` §7.20).
//!
//! The test creatures are 10/10 with damage 2 and 10 HP: `blinder` casts `t:blind` after every
//! strike, `hexer` `t:curse` with 25 % on living stacks, `fly` `t:unbless` and then `t:weaken`
//! at Advanced unless active, `stormbird` deals 10 per creature as `t:bolt`, `twinblinder`
//! strikes twice and blinds, `weakener` casts `t:weaken`; `binder` (speed 3) binds with
//! `t:tether`, `corroder` (damage 2 to 4) breathes `t:acid` and deals half a roll per creature,
//! `nibbler` (damage 1) a fifth of the time half a roll.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::{Ruleset, SpellId};
use h3_contracts::{BattleHero, BattleSide};

use super::spells::{arena, spell};
use super::support::*;
use crate::effects::TimedCast;
use crate::magic::SpellBonus;
use crate::{
    BattleState, CastTarget, Command, Event, EventKind, LegalCommands, StackId, Status,
    legal_commands,
};

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

// covers: BAT-MAG-040
#[test]
fn magic_mirror_never_sends_a_strikes_spell_away() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("blinder", 5, 7, 5), ("fixed", 5, 9, 5)],
        &[("fixed", 10, 8, 5)],
        1,
    );
    // An Expert mirror redirects every spell it may; the strike's Blind still lands.
    give(&mut s, &r, (2, "mirror"), 3);
    let events = hit(&mut s, &r, 2, (7, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 2, String::from("blind"))]
    );
    assert!(s.at(StackId(2)).spells.status.contains(Status::NO_TURNS));
}

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
    // By chance: the `squall`'s bolt follows half of its strikes, by the battle's generator.
    let bolts: Vec<bool> = (0..12)
        .map(|seed| {
            let mut s = duel(&r, "squall", "fixed", seed);
            !spell_damage(&hit(&mut s, &r, 1, (7, 5))).is_empty()
        })
        .collect();
    assert!(bolts.contains(&true) && bolts.contains(&false));
}

/// E04b T11b review (F-r4b-strike-magic-8 part a): a bolt whose damage comes to 0 does nothing,
/// as the extra damage of BAT-MAG-047: no report and nothing that damage ends (the original
/// leaves when the spell's damage on the target is not above 0, the reviewer's reading at
/// 0x4411FD). One Thunderbird against Diamond Golems: a twentieth of 10.
// covers: BAT-MAG-041
#[test]
fn a_thunderbolt_of_no_damage_does_nothing() {
    use h3_content::{AbilityDef, ContentKey};
    let one = |r: &Ruleset, striker: &'static str, target: &'static str| {
        arena(
            r,
            [None, None],
            &[(striker, 1, 7, 5)],
            &[(target, 10, 8, 5)],
            1,
        )
    };
    let r = ruleset();
    // One stormbird: 10 as `t:bolt`, of which `diamond` takes a twentieth, rounded down.
    let mut s = one(&r, "stormbird", "diamond");
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(strikes(&events), vec![(0, 1), (1, 0)]);
    assert_eq!(strike_spells(&r, &events), vec![]);
    assert_eq!(spell_damage(&events), vec![]);
    // A quarter of it (`golem`: 2) is dealt and reported.
    let mut s = one(&r, "stormbird", "golem");
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(spell_damage(&events), vec![(1, 10, 2, 0)]);
    assert_eq!(strike_spells(&r, &events).len(), 1);
    // Nor does it end what damage ends: a strike that blinds and then bolts leaves the target
    // blind (it used to wake it at once).
    let key = |name: &str| ContentKey::new("t", name).unwrap();
    let r = ruleset_from(
        "t:faithful",
        |_| {},
        |pack| {
            let blinder = pack.creatures.iter_mut().find(|c| c.key == key("blinder"));
            let bolt = AbilityDef::SpellDamageAfterStrike {
                spell: key("bolt"),
                chance_percent: 100,
                per_creature: 10,
            };
            blinder.unwrap().info.abilities.push(bolt);
        },
    );
    let mut s = one(&r, "blinder", "diamond");
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 1, String::from("blind"))]
    );
    assert_eq!(spell_damage(&events), vec![]);
    assert!(s.stacks()[1].spells.status.contains(Status::NO_TURNS));
    // With damage the same strike's bolt ends the Blind it brought.
    let mut s = one(&r, "blinder", "golem");
    hit(&mut s, &r, 1, (7, 5));
    assert!(!s.stacks()[1].spells.status.contains(Status::NO_TURNS));
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
    // The same half in the executable's stages (quirk Q-0005: stage 3, `installation.md` §7.10).
    let r = ruleset_with("t:single", |_| {});
    let (mut s, _) = struck(&r, "gazer", "fixed");
    activate(&mut s, 1);
    assert_eq!(damages(&hit(&mut s, &r, 2, (9, 5)))[0], (1, 2, 5));
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

// covers: BAT-MAG-009, BAT-MAG-040
#[test]
fn disease_takes_no_more_attack_and_defense_than_there_is() {
    let r = ruleset();
    // Peasants have 1 attack and 1 defense: Disease takes 1 of each and gives back 1.
    let (mut s, events) = struck(&r, "rotter", "peasant");
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 2, String::from("plague"))]
    );
    let peasants = |s: &BattleState| {
        let p = s.at(StackId(2));
        (p.attack_now(false), p.defense_now())
    };
    assert_eq!(peasants(&s), (0, 0));
    for _ in 0..3 {
        new_round(&mut s, &r);
    }
    assert_eq!(peasants(&s), (1, 1));
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

// covers: BAT-MAG-044, BAT-MAG-010, BAT-MAG-035
#[test]
fn a_stack_that_dies_aged_comes_back_with_its_full_hit_points() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("ancient", 5, 7, 5), ("seraph", 20, 9, 5)],
        &[("serf", 10, 8, 5), ("fixed", 1, 14, 2)],
        1,
    );
    activate(&mut s, 0);
    hit(&mut s, &r, 2, (7, 5));
    assert_eq!(s.at(StackId(2)).max_hp, 5);
    activate(&mut s, 1);
    hit(&mut s, &r, 2, (9, 5));
    // Its death ended Age (the original's 0x444120 ends every spell): 10 per creature again,
    // so 20 hit points raise two serfs, not four.
    let serfs = s.at(StackId(2));
    assert_eq!((serfs.is_alive(), serfs.max_hp), (false, 10));
    crate::raise::raise(&mut s, &r, (StackId(2), 20), false, &mut Vec::new());
    assert_eq!(s.at(StackId(2)).count, 2);
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

// covers: BAT-MAG-044
#[test]
fn poison_is_reported_when_age_ends_at_the_same_round_start() {
    let r = ruleset();
    let mut s = duel(&r, "fixed", "serf", 1);
    // Poison for three rounds (10 × 0.9 + 0.95 → 9) and Age in its last round (4.5 + 0.95 → 5).
    for (name, rounds) in [("venom", 3), ("wither", 1)] {
        let id = spell(&r, name);
        let cast = TimedCast {
            id,
            spell: r.spell(id).unwrap(),
            mastery: 0,
            rounds,
            own: SpellBonus::default(),
        };
        crate::effects::apply(&mut s, &r, cast, StackId(1), &mut Vec::<Event>::new());
    }
    assert_eq!(s.at(StackId(1)).max_hp, 5);
    // The round start ends Age (9 again), then Poison takes its tenth (8): the fall is
    // reported, as the original compares with the hit points after Age's end (review r4b F-8b;
    // compared with the aged 5, the 8 looked like a gain and nothing was reported).
    let events = new_round(&mut s, &r);
    assert_eq!(poisoned(&events), vec![(1, 8)]);
    assert_eq!(s.at(StackId(1)).max_hp, 8);
    assert_eq!(on(&s, &r, 1), vec![(String::from("venom"), Some(2), 0)]);
}

// ---------------------------------------------------------------- BAT-MAG-045 to 047

/// Whether `events` report the end of `t:tether` (Bind) on stack `id`.
fn unbound(r: &Ruleset, events: &[Event], id: u8) -> bool {
    events.contains(&Event::public(EventKind::EffectEnded {
        target: StackId(id),
        spell: spell(r, "tether"),
    }))
}

/// Two binders at (7, 5) and (9, 5), `fixed` ×10 between them at (8, 5); the first has struck
/// it, `fixed` ×5 waits at (3, 3).
fn bound(r: &Ruleset, heroes: [Option<BattleHero>; 2]) -> BattleState {
    let mut s = arena(
        r,
        heroes,
        &[("binder", 5, 7, 5), ("binder", 1, 9, 5), ("fixed", 5, 3, 3)],
        &[("fixed", 10, 8, 5)],
        1,
    );
    let events = hit(&mut s, r, 3, (7, 5));
    assert_eq!(
        strike_spells(r, &events),
        vec![(0, 3, String::from("tether"))]
    );
    s
}

// covers: BAT-MAG-045
#[test]
fn bind_holds_the_struck_stack_for_255_rounds_where_it_attacks_only_next_to_it() {
    let r = ruleset();
    let mut s = bound(&r, [None, None]);
    // No test, fixed rounds; the struck stack retaliated and is bound.
    assert_eq!(on(&s, &r, 3), vec![(String::from("tether"), Some(255), 0)]);
    assert!(crate::binds::is_bound(&s, StackId(3)));
    // Its commands: no move, attacks from where it stands only (both binders next to it).
    activate(&mut s, 3);
    let mut legal = LegalCommands::new();
    legal_commands(&s, &r, BattleSide::Defender, &mut legal);
    assert!(!legal.iter().any(|c| matches!(c, Command::Move { .. })));
    let attacks: Vec<_> = legal
        .iter()
        .filter_map(|c| match *c {
            Command::Attack { target, from } => Some((target.0, from)),
            _ => None,
        })
        .collect();
    let here = at(&r, 8, 5);
    assert_eq!(attacks, vec![(0, here), (1, here)]);
    let walk = Command::Move { to: at(&r, 8, 6) };
    assert_eq!(act(&mut s, &r, walk), Err(crate::CommandError::Unreachable));
    // The same binder strikes again: nothing (it already binds the stack).
    activate(&mut s, 0);
    assert!(strike_spells(&r, &hit(&mut s, &r, 3, (7, 5))).is_empty());
    // The rounds pass; a stack left without binders is free (rounds counted down meanwhile).
    new_round(&mut s, &r);
    assert_eq!(on(&s, &r, 3), vec![(String::from("tether"), Some(254), 0)]);
}

// covers: BAT-MAG-045
#[test]
fn bind_ends_when_its_last_binder_moves_or_dies() {
    let r = ruleset();
    let mut s = bound(&r, [None, None]);
    // The second binder joins (the struck stack retaliated already this round).
    activate(&mut s, 1);
    let events = hit(&mut s, &r, 3, (9, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![(1, 3, String::from("tether"))]
    );
    // The first one walks away: the second still binds it.
    activate(&mut s, 0);
    let events = act(&mut s, &r, Command::Move { to: at(&r, 6, 5) }).unwrap();
    assert!(!unbound(&r, &events, 3));
    assert!(crate::binds::is_bound(&s, StackId(3)));
    // The bound stack kills the second binder: free.
    activate(&mut s, 3);
    let events = hit(&mut s, &r, 1, (8, 5));
    assert!(!s.at(StackId(1)).is_alive());
    assert!(unbound(&r, &events, 3));
    assert!(!crate::binds::is_bound(&s, StackId(3)));
    assert!(on(&s, &r, 3).is_empty());
    // Now it moves.
    activate(&mut s, 3);
    act(&mut s, &r, Command::Move { to: at(&r, 8, 6) }).unwrap();
}

// covers: BAT-MAG-045
#[test]
fn teleport_ends_the_bind_of_a_stack_teleported_and_frees_the_stacks_of_a_binder() {
    let r = ruleset();
    let porter = || {
        let mut h = super::spells::mage(&r, &["teleport"]);
        h.magic.mana = 100;
        Some(h)
    };
    let s = bound(&r, [porter(), porter()]);
    let teleport = |id: u8, x, y| Command::Cast {
        spell: spell(&r, "teleport"),
        target: super::spells::stack(id),
        second: CastTarget::Hex(at(&r, x, y)),
    };
    // The bound stack's hero teleports it.
    let mut t = s.clone();
    activate(&mut t, 3);
    let events = act(&mut t, &r, teleport(3, 12, 5)).unwrap();
    assert!(unbound(&r, &events, 3));
    assert!(!crate::binds::is_bound(&t, StackId(3)));
    // The binder's hero teleports the binder.
    let mut t = s.clone();
    activate(&mut t, 0);
    let events = act(&mut t, &r, teleport(0, 2, 2)).unwrap();
    assert!(unbound(&r, &events, 3));
    // Any other cast keeps it.
    let mut t = s;
    activate(&mut t, 0);
    let events = act(&mut t, &r, teleport(2, 2, 2)).unwrap();
    assert!(!unbound(&r, &events, 3));
}

// covers: BAT-MAG-045
#[test]
fn dispel_ends_bind_cure_does_not_and_the_binder_may_bind_again() {
    let r = ruleset();
    let cleric = || {
        let mut h = super::spells::mage(&r, &["cure", "dispel"]);
        h.magic.mana = 100;
        Some(h)
    };
    let mut s = bound(&r, [None, cleric()]);
    activate(&mut s, 3);
    let cure = super::spells::cast(&r, "cure", super::spells::stack(3));
    act(&mut s, &r, cure).unwrap();
    assert!(crate::binds::is_bound(&s, StackId(3)));
    new_round(&mut s, &r);
    activate(&mut s, 3);
    let dispel = super::spells::cast(&r, "dispel", super::spells::stack(3));
    let events = act(&mut s, &r, dispel).unwrap();
    assert!(unbound(&r, &events, 3));
    activate(&mut s, 0);
    let events = hit(&mut s, &r, 3, (7, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 3, String::from("tether"))]
    );
}

// covers: BAT-MAG-025, BAT-MAG-045
#[test]
fn a_bound_berserk_stack_attacks_only_next_to_it() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 4, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    give(&mut s, &r, (0, "tether"), 0);
    let mut events = Vec::new();
    crate::turn::berserk_now(&mut s, &r, &mut events);
    assert!(events.contains(&Event::public(EventKind::TurnPassed { stack: StackId(0) })));
    assert_eq!(s.at(StackId(0)).hex, at(&r, 2, 5));
    // Next to a stack, it attacks it.
    place(&mut s, &r, 1, 3, 5);
    activate(&mut s, 0);
    let mut events = Vec::new();
    crate::turn::berserk_now(&mut s, &r, &mut events);
    assert_eq!(strikes(&events), vec![(0, 1), (1, 0)]);
}

// covers: BAT-ATK-004, BAT-MAG-045, BAT-MAG-024
#[test]
fn a_harpy_bound_or_blinded_by_the_retaliation_stays_where_it_struck() {
    let r = ruleset();
    let duel = |defender| {
        let mut s = arena(
            &r,
            [None, None],
            &[("harpy", 5, 3, 5)],
            &[(defender, 10, 8, 5), ("fixed", 1, 14, 2)],
            1,
        );
        activate(&mut s, 0);
        hit(&mut s, &r, 1, (7, 5));
        s
    };
    assert_eq!(duel("fixed").at(StackId(0)).hex, at(&r, 3, 5));
    assert_eq!(duel("blinder").at(StackId(0)).hex, at(&r, 7, 5));
    // Survey S-14 fixed: a bound harpy stays next to its binder, which frees it by walking away.
    let mut s = duel("binder");
    assert_eq!(s.at(StackId(0)).hex, at(&r, 7, 5));
    assert!(crate::binds::is_bound(&s, StackId(0)));
    activate(&mut s, 1);
    let events = act(&mut s, &r, Command::Move { to: at(&r, 10, 5) }).unwrap();
    assert!(unbound(&r, &events, 0));
    activate(&mut s, 0);
    act(&mut s, &r, Command::Move { to: at(&r, 3, 5) }).unwrap();
}

// covers: BAT-MAG-045
#[test]
fn a_bound_shooter_still_shoots() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("archer", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "tether"), 0);
    assert!(crate::binds::is_bound(&s, StackId(0)));
    activate(&mut s, 0);
    let shoot = Command::Shoot {
        target: StackId(1),
        at: s.at(StackId(1)).hex,
    };
    act(&mut s, &r, shoot).unwrap();
}

// covers: BAT-MAG-045, BAT-MAG-038
#[test]
fn a_binding_clone_that_vanishes_with_its_original_frees_the_stack_it_binds() {
    let r = ruleset();
    let mut copier = super::spells::mage(&r, &["copy"]);
    copier.magic.masteries = masteries(&r, [1, 0, 0, 0]);
    copier.magic.mana = 100;
    let mut s = arena(
        &r,
        [Some(copier), None],
        &[("binder", 5, 2, 2)],
        &[("fixed", 10, 8, 5), ("fixed", 1, 14, 2)],
        1,
    );
    activate(&mut s, 0);
    act(
        &mut s,
        &r,
        super::spells::cast(&r, "copy", super::spells::stack(0)),
    )
    .unwrap();
    let clone = StackId(3);
    assert!(s.at(clone).is_created());
    // The clone binds the Defender's stack (as its strike would).
    assert!(crate::binds::join(&mut s, clone, StackId(1)));
    give(&mut s, &r, (1, "tether"), 0);
    assert!(crate::binds::is_bound(&s, StackId(1)));
    // Its original dies and takes it along: the bound stack is free.
    let mut events = Vec::new();
    crate::damage::apply_damage(s.at_mut(StackId(0)), 1000);
    crate::effects::damaged(&mut s, &r, StackId(0), &mut events);
    assert!(!s.at(clone).is_alive());
    assert!(unbound(&r, &events, 1));
}

/// The extra damage `events` report, as `(striker, target, damage, kills)`.
fn extra(events: &[Event]) -> Vec<(u8, u8, u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::StrikeDamage {
                striker,
                target,
                damage,
                kills,
            } => Some((striker.0, target.0, damage, kills)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-046
#[test]
fn acid_breath_lowers_the_defense_by_three_for_the_battle_down_to_0() {
    let r = ruleset();
    let mut s = duel(&r, "corroder", "seraph", 1);
    let mut defenses = Vec::new();
    for _ in 0..8 {
        activate(&mut s, 0);
        let events = hit(&mut s, &r, 1, (7, 5));
        let acid = strike_spells(&r, &events).len();
        defenses.push((acid, s.at(StackId(1)).defense_now()));
    }
    // 20, 17, …, 2, then 0; at 0 the breath does nothing.
    let lowered = [17, 14, 11, 8, 5, 2, 0].map(|d| (1, d));
    assert_eq!(defenses, [&lowered[..], &[(0, 0)]].concat());
    assert_eq!(on(&s, &r, 1), vec![(String::from("acid"), None, 3)]);
    // For the battle: the rounds pass.
    for _ in 0..4 {
        new_round(&mut s, &r);
    }
    assert_eq!(s.at(StackId(1)).defense_now(), 0);
    // Without a test: also on a stack immune to its level.
    let mut s = duel(&r, "corroder", "gilded", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    assert_eq!(
        strike_spells(&r, &events),
        vec![(0, 1, String::from("acid"))]
    );
}

// covers: BAT-MAG-046, BAT-MAG-010, BAT-MAG-035
#[test]
fn acid_breath_stays_on_a_stack_that_dies_and_is_raised() {
    let r = ruleset();
    let mut s = duel(&r, "corroder", "serf", 1);
    hit(&mut s, &r, 1, (7, 5));
    assert_eq!(s.at(StackId(1)).defense_now(), 7);
    // The serfs die: the end of the command ends a dead stack's timed effects, and the breath
    // is none (review r4b F-1: the original's death gives no defense back, nor does a raise).
    let total = s.at(StackId(1)).total_hp();
    crate::damage::apply_damage(s.at_mut(StackId(1)), total);
    crate::effects::damaged(&mut s, &r, StackId(1), &mut Vec::<Event>::new());
    crate::effects::remove_dead(&mut s, &r);
    crate::raise::mark_corpses(&mut s);
    assert_eq!(on(&s, &r, 1), vec![(String::from("acid"), None, 3)]);
    assert_eq!(s.at(StackId(1)).defense_now(), 7);
    let raise = (StackId(1), 20);
    crate::raise::raise(&mut s, &r, raise, false, &mut Vec::<Event>::new());
    let serfs = s.at(StackId(1));
    assert_eq!((serfs.count, serfs.defense_now()), (2, 7));
    assert_eq!(on(&s, &r, 1), vec![(String::from("acid"), None, 3)]);
}

// covers: BAT-MAG-047
#[test]
fn the_extra_damage_is_a_share_of_one_roll_per_creature_after_the_breath() {
    let r = ruleset();
    let mut s = duel(&r, "corroder", "seraph", 1);
    let events = hit(&mut s, &r, 1, (7, 5));
    // The strike, the breath, then 50 % of one roll of 2 to 4 times five creatures.
    let rows = extra(&events);
    assert_eq!(rows.len(), 1);
    let (striker, target, damage, kills) = rows[0];
    assert_eq!((striker, target, kills), (0, 1, 0));
    assert!([5, 7, 10].contains(&damage), "{damage}");
    let order: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                EventKind::StrikeSpell { .. } | EventKind::StrikeDamage { .. }
            )
        })
        .collect();
    assert!(matches!(order[0].kind, EventKind::StrikeSpell { .. }));
    let struck = damages(&events)[0].2;
    let seraphs = s.at(StackId(1));
    assert_eq!(u64::from(seraphs.max_hp - seraphs.top_hp), struck + damage);
    // A draw per strike; nothing when the share rounds to 0 (one nibbler: half of 1).
    let mut drawn = 0;
    for seed in 0..10 {
        let mut s = arena(
            &r,
            [None, None],
            &[("nibbler", 1, 7, 5), ("nibbler", 4, 9, 5)],
            &[("seraph", 10, 8, 5)],
            seed,
        );
        assert!(extra(&hit(&mut s, &r, 2, (7, 5))).is_empty());
        activate(&mut s, 1);
        let rows = extra(&hit(&mut s, &r, 2, (9, 5)));
        assert!(rows.iter().all(|&(_, _, damage, _)| damage == 2));
        drawn += rows.len();
    }
    assert!((1..10).contains(&drawn), "{drawn}");
}
