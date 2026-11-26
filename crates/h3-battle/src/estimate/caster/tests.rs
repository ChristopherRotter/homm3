//! Tests of the computer's values of a hero's spells (E31 T2e, AI-Q36): the caster record (its
//! numbers held, the army's value and list, the word counts, no power without a book or with
//! `NoCasting`), a spell's value (the cost and the casts, the mastery on the hero's field, the
//! kinds: damage, one cast, every stack, other spells, raises, the map's spells, none or
//! several), the best value (the kinds, the level limit, the floor of 0). On the estimate tests'
//! ruleset with AI values; the numbers come from an IEEE double model of the reading (each
//! case's comment gives its sums; `E31-log.md` T2e).

use std::vec;
use std::vec::Vec;

use h3_content::CreatureTypeId;
use h3_content_battle::{BattleRuleset, SpellAiKind};
use h3_contracts::{ArmyStack, BattleEffect, BattleHero};

use super::super::tests::{army, book, caster, creature, kinds, rules, spell};
use super::*;

/// The record of `hero` with its book as its available spells and `stacks`, on `ground`.
fn record_of(
    r: &BattleRuleset,
    hero: &BattleHero,
    stacks: &[ArmyStack],
    ground: Option<BattlefieldId>,
) -> CasterRecord {
    let spells = book(hero);
    CasterRecord::new(
        r,
        EstimateHero {
            hero,
            spells: &spells,
        },
        stacks,
        ground,
    )
}

/// The record's list as `(creature, value)`.
fn list(record: &CasterRecord) -> Vec<(CreatureTypeId, i32)> {
    record
        .entries
        .iter()
        .map(|e| (e.creature, e.value))
        .collect()
}

// covers: CPU-SPL-001
#[test]
fn a_record_holds_its_numbers_and_its_army_list_highest_first() {
    let r = rules();
    let mut hero = caster(&r, 7, 30, &[("bolt", 2)]);
    hero.effects.push(BattleEffect::SpellDuration { rounds: 3 });
    // The archer's 70 000 counts 4 464 in the list (its low word), the slow's 40 000 -25 536.
    let stacks = army(
        &r,
        &[
            (0, "fixed", 10),
            (1, "tank", 5),
            (2, "slow", 40_000),
            (3, "archer", 70_000),
            (5, "fixed", 10),
        ],
    );
    let record = record_of(&r, &hero, &stacks, None);
    // 70 + 250 + 120 000 + 770 000 + 70.
    assert_eq!(record.strength, 890_390);
    assert_eq!((record.power, record.rounds, record.mana), (7, 10, 30));
    let c = |n| creature(&r, n);
    assert_eq!(
        list(&record),
        vec![
            (c("archer"), 49_104),
            (c("tank"), 250),
            (c("fixed"), 70),
            (c("fixed"), 70),
            (c("slow"), -76_608),
        ]
    );
    // The power held: 0 as 1, 120 as 99, a byte of 200 (-56) as 1.
    for (byte, held) in [(0, 1), (120, 99), (200, 1)] {
        hero.power = byte;
        assert_eq!(record_of(&r, &hero, &stacks, None).power, held, "{byte}");
    }
}

// covers: CPU-SPL-001
#[test]
fn a_hero_without_a_book_or_with_no_casting_has_no_power_and_no_list() {
    let r = rules();
    let stacks = army(&r, &[(0, "tank", 5)]);
    let mut plain = caster(&r, 7, 30, &[("bolt", 2)]);
    plain.magic.spell_book = None;
    let mut orb = caster(&r, 7, 30, &[("bolt", 2)]);
    orb.effects.push(BattleEffect::NoCasting);
    for hero in [plain, orb] {
        let spells = vec![spell(&r, "bolt")];
        let record = CasterRecord::new(
            &r,
            EstimateHero {
                hero: &hero,
                spells: &spells,
            },
            &stacks,
            None,
        );
        assert_eq!(
            (record.strength, record.power, record.rounds, record.mana),
            (0, 0, 0, 0)
        );
        assert!(record.entries.is_empty());
        assert_eq!(record.value(&r, spell(&r, "bolt")), 0);
        assert_eq!(record.best(&r, kinds(&[SpellAiKind::Damage])), 0);
    }
}

// covers: CPU-SPL-002
#[test]
fn a_spell_costs_its_mana_and_a_spell_of_no_kind_or_several_is_worth_one() {
    let r = rules();
    let stacks = army(&r, &[(0, "tank", 100)]);
    // The bolt costs 4 at Basic: worth nothing with 3.
    let poor = record_of(&r, &caster(&r, 5, 3, &[("bolt", 1)]), &stacks, None);
    assert_eq!(poor.value(&r, spell(&r, "bolt")), 0);
    let hero = caster(&r, 5, 20, &[("ward", 0), ("smite", 1), ("quake", 0)]);
    let record = record_of(&r, &hero, &stacks, None);
    // `ward` has no AI kind, `smite` two.
    assert_eq!(record.value(&r, spell(&r, "ward")), 1);
    assert_eq!(record.value(&r, spell(&r, "smite")), 1);
    // `quake` is free: 1 000 casts, (0.001 sqrt(1 000) + 0.009) x 5 000 = 203.11, even with no
    // mana.
    let broke = record_of(&r, &caster(&r, 5, 0, &[("quake", 0)]), &stacks, None);
    assert_eq!(broke.value(&r, spell(&r, "quake")), 203);
    assert_eq!(record.value(&r, spell(&r, "quake")), 203);
    // A spell the ruleset lacks.
    assert_eq!(record.value(&r, SpellId::from_index(999).unwrap()), 0);
}

// covers: CPU-SPL-003
#[test]
fn a_damage_spell_follows_the_curve_bounded_by_its_casts() {
    let r = rules();
    let tanks = |n| army(&r, &[(0, "tank", n)]);
    let bolt = spell(&r, "bolt");
    // Power 5 at Basic: b = 6, X = 60 / 100 = 0.6, row 3: Y = 1.3632; five casts Z = 1.794:
    // 100 x Y = 136.32; one cast (4 mana) Z = 0.498: 100 x Z = 49.8.
    let rich = caster(&r, 5, 20, &[("bolt", 1), ("chain", 1)]);
    assert_eq!(record_of(&r, &rich, &tanks(2), None).value(&r, bolt), 136);
    let poor = caster(&r, 5, 4, &[("bolt", 1)]);
    assert_eq!(record_of(&r, &poor, &tanks(2), None).value(&r, bolt), 49);
    // The chain is valued as one cast whatever the mana.
    let chain = spell(&r, "chain");
    assert_eq!(record_of(&r, &rich, &tanks(2), None).value(&r, chain), 49);
    // X = 60 / 10 000 = 0.006 reaches no row: Y = 4.53 X = 0.02718 above Z = 2.99 X = 0.01794:
    // 10 000 x Z = 179.4.
    assert_eq!(record_of(&r, &rich, &tanks(200), None).value(&r, bolt), 179);
    // No strength: the original's NaN, 0.
    assert_eq!(record_of(&r, &rich, &[], None).value(&r, bolt), 0);
}

// covers: CPU-SPL-004
#[test]
fn a_spell_on_every_stack_is_worth_its_damage_on_the_spared_part_less_the_hurt_part() {
    let r = rules();
    let hero = caster(&r, 5, 20, &[("storm", 1)]);
    let storm = spell(&r, "storm");
    let value =
        |spec: &[(u8, &str, u32)]| record_of(&r, &hero, &army(&r, spec), None).value(&r, storm);
    // The wyrm is immune (level 1): 500 spared of 570; against 500 X = 0.12, row 7, Y = 0.43932
    // above Z = 0.3588: 179, less the 70 hurt.
    assert_eq!(value(&[(0, "wyrm", 50), (1, "fixed", 10)]), 109);
    // The dwarf lets 0.8 (f32) through: trunc(200 x 0.19999998) = 39 spared; against 39 X =
    // 1.538, row 2, Y = 2.3605: 92, less the 161 hurt: at least 0.
    assert_eq!(value(&[(0, "dwarf", 10)]), 0);
    // Nothing spared: 0.
    assert_eq!(value(&[(0, "fixed", 10)]), 0);
}

// covers: CPU-SPL-005
#[test]
fn another_spell_counts_its_rounds_on_the_best_entries() {
    let r = rules();
    let stacks = army(&r, &[(0, "fixed", 10), (1, "tank", 5)]);
    let curse = spell(&r, "curse");
    let value = |power, mana, mastery, ground| {
        let hero = caster(&r, power, mana, &[("curse", mastery)]);
        record_of(&r, &hero, &stacks, ground).value(&r, curse)
    };
    // One target at Basic (v 2): power 2 and 5 casts make 10 rounds, one entry for 7: 2 x 250 x
    // 7 / 700 = 5; power 6, 30 rounds, at most 4 entries (both): (3 500 + 980) / 700 = 6; one
    // cast, 2 rounds below 7: the first entry for 2: 1 000 / 700 = 1.
    assert_eq!(value(2, 20, 1, None), 5);
    assert_eq!(value(6, 20, 1, None), 6);
    assert_eq!(value(2, 4, 1, None), 1);
    // At Expert (v 4) the curse takes every stack, its rounds held to 7: 4 x 320 x 7 / 700 =
    // 12; on the plains (air at Expert) the same at Basic.
    assert_eq!(value(2, 20, 3, None), 12);
    let plains = r.battlefield_by_key("t:plains");
    assert_eq!(value(2, 20, 1, plains), 12);
}

// covers: CPU-SPL-005
#[test]
fn a_positive_spell_passes_the_entries_it_cannot_affect_and_a_value_may_go_below_zero() {
    let r = rules();
    let stacks = army(&r, &[(0, "fixed", 10), (1, "wyrm", 50)]);
    let both = caster(&r, 2, 20, &[("bless", 3), ("curse", 3)]);
    let record = record_of(&r, &both, &stacks, None);
    // The wyrm (500) is immune to the blessing: 4 x 70 x 7 / 700 = 2; the curse counts both:
    // 4 x 570 x 7 / 700 = 22.
    assert_eq!(record.value(&r, spell(&r, "bless")), 2);
    assert_eq!(record.value(&r, spell(&r, "curse")), 22);
    // The slow's 40 000 count -25 536: 2 x -76 608 x 7 / 700 = -1 532 (toward 0), which the best
    // value does not take.
    let slow = army(&r, &[(0, "slow", 40_000)]);
    let record = record_of(&r, &caster(&r, 2, 20, &[("curse", 1)]), &slow, None);
    assert_eq!(record.value(&r, spell(&r, "curse")), -1_532);
    assert_eq!(record.best(&r, kinds(&[SpellAiKind::Effect])), 0);
}

// covers: CPU-SPL-002
#[test]
fn a_raise_is_worth_its_ratio_held_twice_times_the_strength() {
    let r = rules();
    let rise = spell(&r, "rise");
    let value = |mana, tanks| {
        let hero = caster(&r, 5, mana, &[("rise", 2)]);
        record_of(&r, &hero, &army(&r, &[(0, "tank", tanks)]), None).value(&r, rise)
    };
    // Advanced (v 60): 10 x 60 x 7 / 100 = 42, held to 0.9; five casts x 2.99 = 2.691: 269.1;
    // twenty x 4.53 = 4.077, held to 3.9: 390.
    assert_eq!(value(20, 2), 269);
    assert_eq!(value(80, 2), 390);
    // Against 100 000: 0.042 x 2.99 = 0.12558, times 100 000 12 558.000000000002.
    assert_eq!(value(20, 2000), 12_558);
    // No strength: 0.
    assert_eq!(value(20, 0), 0);
}

// covers: CPU-SPL-006
#[test]
fn the_best_value_takes_the_kinds_asked_for_and_the_level_limit() {
    let r = rules();
    let stacks = army(&r, &[(0, "tank", 2)]);
    let mut hero = caster(&r, 5, 20, &[("bolt", 1), ("chain", 1), ("quake", 0)]);
    let record = record_of(&r, &hero, &stacks, None);
    use SpellAiKind::{Damage, DamageOnce, Special};
    assert_eq!(record.best(&r, kinds(&[Damage])), 136);
    assert_eq!(record.best(&r, kinds(&[DamageOnce])), 49);
    assert_eq!(record.best(&r, kinds(&[Damage, DamageOnce])), 136);
    // The quake (level 3): (0.001 sqrt(1 000) + 0.009) x 100 = 4.06; not under a limit of 2.
    assert_eq!(record.best(&r, kinds(&[Special])), 4);
    hero.effects
        .push(BattleEffect::SpellLevelLimit { max_level: 2 });
    let limited = record_of(&r, &hero, &stacks, None);
    assert_eq!(limited.best(&r, kinds(&[Special])), 0);
    assert_eq!(limited.best(&r, kinds(&[Damage])), 136);
}
