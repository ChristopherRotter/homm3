//! Spell specialists (E04b T3c, BAT-MAG-020): the bonus a hero specialised in a spell adds to
//! its value, by the spell's own rule or scaled by the hero's level (`installation.md` §7.12).

use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleEffect, BattleHero};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::magic::SpellBonus;
use crate::{BattleState, Event, StackId};

/// The specialty in `name` of a hero at `level` (3 % per level, as every original one).
fn specialty(r: &Ruleset, name: &str, level: u8) -> BattleEffect {
    BattleEffect::SpellSpecialty {
        spell: spell(r, name),
        bp_per_level: 300,
        level,
    }
}

/// A mage with `spells`, Basic or better air (`mastery`), plenty of mana and `specialties`.
fn specialist(r: &Ruleset, spells: &[&str], mastery: u8, specialties: &[(&str, u8)]) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = mastery;
    h.magic.mana = 100;
    h.effects = specialties
        .iter()
        .map(|&(name, level)| specialty(r, name, level))
        .collect();
    h
}

/// The bonuses of a level-10 specialist in `name` (none: no specialty).
fn own(r: &Ruleset, name: Option<&str>) -> SpellBonus {
    name.map_or_else(SpellBonus::default, |n| SpellBonus {
        spell: spell(r, n),
        damage_bp: 0,
        per_level_bp: 300,
        level: 10,
    })
}

/// Gives stack `id` the timed effect of `name` cast at `mastery` by a caster with the bonuses
/// `own`; returns the effect's value.
fn give(
    s: &mut BattleState,
    r: &Ruleset,
    (id, name): (u8, &str),
    mastery: u8,
    own: SpellBonus,
) -> u16 {
    let id_of = spell(r, name);
    let cast = TimedCast {
        id: id_of,
        spell: r.spell(id_of).unwrap(),
        mastery,
        rounds: 5,
        own,
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
    s.effects()
        .iter()
        .find(|e| e.stack == StackId(id) && e.spell == id_of)
        .unwrap()
        .value
}

/// Own stacks of creature levels 1, 3, 5 and 7, and one enemy.
fn levels(r: &Ruleset) -> BattleState {
    arena(
        r,
        [None, None],
        &[
            ("fixed", 1, 3, 1),
            ("tier3", 1, 3, 3),
            ("tier5", 1, 3, 5),
            ("titan", 1, 3, 7),
            ("spear", 1, 3, 9),
        ],
        &[("fixed", 1, 13, 5)],
        1,
    )
}

// covers: BAT-MAG-020
#[test]
fn a_cast_adds_the_specialists_bonus_by_the_targets_level() {
    let r = ruleset();
    // Through a cast: Stone Skin at no mastery (3) by a level-10 specialist on a level-1 stack.
    let mut s = arena(
        &r,
        [Some(specialist(&r, &["skin"], 0, &[("skin", 10)])), None],
        &[("fixed", 1, 3, 5)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "skin", stack(0))).unwrap();
    // 10, the hero's 2 and 3 + 3.
    assert_eq!(s.stacks()[0].defense_now(), 18);
    // A specialist in another spell adds nothing.
    let mut s = arena(
        &r,
        [Some(specialist(&r, &["skin"], 0, &[("haste", 10)])), None],
        &[("fixed", 1, 3, 5)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "skin", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].defense_now(), 15);
    // The creature-level table: 3, 3, 2, 2, 1, 1, 0 for levels 1 to 7, whatever the hero's level.
    let skin = own(&r, Some("skin"));
    let mut s = levels(&r);
    let values: Vec<u16> = (0..4)
        .map(|id| give(&mut s, &r, (id, "skin"), 0, skin))
        .collect();
    assert_eq!(values, [6, 5, 4, 3]);
    assert_eq!(s.stacks()[2].defense_now(), 14);
    // Haste at Advanced (5) on level 3, Prayer (2) on level 1, Bloodlust on level 5, Precision
    // on level 7.
    let mut s = levels(&r);
    assert_eq!(give(&mut s, &r, (1, "haste"), 2, own(&r, Some("haste"))), 7);
    assert_eq!(s.stacks()[1].speed_now(), 12);
    assert_eq!(
        give(&mut s, &r, (0, "prayer"), 0, own(&r, Some("prayer"))),
        5
    );
    let fixed = s.stacks()[0];
    assert_eq!(
        (
            fixed.attack_now(false),
            fixed.defense_now(),
            fixed.speed_now()
        ),
        (15, 15, 10)
    );
    assert_eq!(give(&mut s, &r, (2, "lust"), 0, own(&r, Some("lust"))), 4);
    assert_eq!(give(&mut s, &r, (3, "aim"), 0, own(&r, Some("aim"))), 3);
    // Without the specialty in the spell nothing changes; a spell without a rule of its own
    // (Mirth) gets nothing at the cast either.
    assert_eq!(give(&mut s, &r, (2, "skin"), 0, own(&r, None)), 3);
    assert_eq!(give(&mut s, &r, (0, "mirth"), 0, own(&r, Some("mirth"))), 1);
}

// covers: BAT-MAG-020, BAT-MAG-009
#[test]
fn weakness_ray_and_fortune_follow_their_own_rules() {
    let r = ruleset();
    let mut s = levels(&r);
    // Weakness: 3 + 3 before the cut to the attack: the Spearmen (attack 4) keep none.
    assert_eq!(
        give(&mut s, &r, (4, "weaken"), 0, own(&r, Some("weaken"))),
        6
    );
    assert_eq!(s.stacks()[4].attack_now(false), 0);
    // Disrupting Ray: 2 more at every cast, the defense at least 0 (10 → 5 → 0).
    let ray = own(&r, Some("ray"));
    assert_eq!(give(&mut s, &r, (0, "ray"), 0, ray), 5);
    assert_eq!(s.stacks()[0].defense_now(), 5);
    give(&mut s, &r, (0, "ray"), 0, ray);
    assert_eq!(s.stacks()[0].defense_now(), 0);
    // Fortune: luck 3 at every mastery.
    let fortune = own(&r, Some("fortune"));
    assert_eq!(give(&mut s, &r, (1, "fortune"), 0, fortune), 3);
    assert_eq!(give(&mut s, &r, (2, "fortune"), 3, fortune), 3);
    assert_eq!(s.stacks()[2].spells.luck, 3);
    assert_eq!(give(&mut s, &r, (3, "fortune"), 3, own(&r, None)), 2);
}

/// The damage range of 10 Archers shooting at `target` after their hero cast `name` on them.
fn shot_after(r: &Ruleset, hero: BattleHero, name: &str, target: &'static str) -> (u64, u64) {
    let mut s = arena(
        r,
        [Some(hero), None],
        &[("archer", 10, 3, 5)],
        &[(target, 1, 8, 2)],
        1,
    );
    act(&mut s, r, cast(r, name, stack(0))).unwrap();
    crate::combat::shot_damage_range(&s, r, StackId(0), StackId(1))
}

// covers: BAT-MAG-020, BAT-MAG-014
#[test]
fn a_slayer_specialist_adds_attack_by_the_strikers_level_at_the_strike() {
    let r = ruleset();
    // Archers (6 + 3) with Slayer's +8 against a king (defense 10): +35 % on 20–30; a
    // specialist adds 4 for a level-1 striker: +55 %.
    let plain = mage(&r, &["slayer"]);
    assert_eq!(shot_after(&r, plain, "slayer", "lord"), (27, 40));
    let coronius = specialist(&r, &["slayer"], 0, &[("slayer", 10)]);
    assert_eq!(shot_after(&r, coronius.clone(), "slayer", "lord"), (31, 46));
    // Not against a creature Slayer does not count against.
    assert_eq!(
        shot_after(&r, coronius.clone(), "slayer", "fixed"),
        (19, 29)
    );
    // The value stays Slayer's: the bonus is the hero's at the strike.
    let mut s = arena(
        &r,
        [Some(coronius), None],
        &[("archer", 10, 3, 5)],
        &[("lord", 1, 8, 2)],
        1,
    );
    act(&mut s, &r, cast(&r, "slayer", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].spells.slayer, (8, 0));
}

// covers: BAT-MAG-020, BAT-DMG-002
#[test]
fn a_bless_specialist_adds_a_share_of_the_base_rounded_up_at_the_first_stage() {
    // Blessed Archers deal 3 each (30); 9 against 10 is −2.5 %. A level-10 specialist adds
    // ⌈30 × 3 % × 10⌉ = 9 to the first stage: 39 × 0.975 = 38; level 5 adds 5 (4.5 rounded
    // up), level 1 adds 1 (0.9). The same in both roundings.
    for profile in ["t:faithful", "t:single"] {
        let r = ruleset_with(profile, |_| {});
        let blessed = |level: Option<u8>| {
            let specialties: Vec<(&str, u8)> = level.map(|l| ("bless", l)).into_iter().collect();
            shot_after(
                &r,
                specialist(&r, &["bless"], 0, &specialties),
                "bless",
                "fixed",
            )
        };
        assert_eq!(blessed(None), (29, 29));
        assert_eq!(blessed(Some(10)), (38, 38));
        assert_eq!(blessed(Some(5)), (34, 34));
        assert_eq!(blessed(Some(1)), (30, 30));
        assert_eq!(blessed(Some(0)), (29, 29));
    }
    // A hero with other specialties too: only Bless's counts.
    let r = ruleset();
    let hero = specialist(&r, &["bless", "aim"], 0, &[("aim", 10), ("bless", 10)]);
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("archer", 10, 3, 5)],
        &[("fixed", 1, 8, 2)],
        1,
    );
    act(&mut s, &r, cast(&r, "aim", stack(0))).unwrap();
    give(&mut s, &r, (0, "bless"), 0, own(&r, None));
    // Precision: 3 + 3 (attack 15 against 10: +25 %), Bless's 9 more: 30 × 1.25 + 9 = 46.
    assert_eq!(
        crate::combat::shot_damage_range(&s, &r, StackId(0), StackId(1)),
        (46, 46)
    );
}

// covers: BAT-MAG-020, BAT-MAG-017
#[test]
fn a_cure_specialist_heals_more_by_its_level_over_the_creatures() {
    let r = ruleset();
    // Cure: power 3 × 5 + 10 = 25; a level-10 specialist adds ⌈25 × 3 % × 10⌉ = 8 on a level-1
    // stack and ⌈25 × 3 % × 1⌉ = 1 on a level-7 one.
    let cured = |specialties: &[(&str, u8)], creature: &'static str| {
        let hero = specialist(&r, &["cure"], 0, specialties);
        let mut s = arena(
            &r,
            [Some(hero), None],
            &[(creature, 1, 3, 5)],
            &[("fixed", 1, 13, 5)],
            1,
        );
        s.stacks.as_mut_slice()[0].top_hp = 50;
        act(&mut s, &r, cast(&r, "cure", stack(0))).unwrap();
        s.stacks()[0].top_hp
    };
    assert_eq!(cured(&[], "tank"), 75);
    assert_eq!(cured(&[("cure", 10)], "tank"), 83);
    assert_eq!(cured(&[("cure", 10)], "titan"), 76);
}
