//! Tests of the mage guild (E09 T47a1) on the fixture's guild: [`guild_core`]'s spells and
//! [`guild_keep`]'s record (level 1 `L1_A` 10, `L1_ZERO` 0, `L1_B` 30, the ability 50, `L1_C` 60;
//! level 2 `L2_A` 40, `L2_B` 60; level 3 `L3` 100; level 4 `L4_ZERO` 0; level 5 the bolt 100,
//! `L5_A` 50, `L5_B` 50), the part's slots 5, 4, 3, 2, 1 (six, five, four, three and two drawn).
//! Since E09 T48a what a hero learns there ([`teaches`]), with [`guild_core`]'s [`sage`]; since
//! E09 T49a1 the night's refill ([`refills`]) and the Mana Vortex ([`vortex`], [`vortex_keep`]).

use alloc::vec;
use alloc::vec::Vec;

use h3_kernel::{Rng, StreamId};

use super::*;
use crate::fixture::{
    ABILITY, AURORA, BOLT, GOLD, KEEP, L1_A, L1_B, L1_C, L1_ZERO, L2_A, L2_B, L3, L4_ZERO, L5_A,
    L5_B, LIBRARY, MAGE_GUILD_1, MAGE_GUILD_2, MAGE_GUILD_3, MAGE_GUILD_4, MAGE_GUILD_5, VORTEX,
    WILD, building, empty_part, guild_core, guild_keep, part_changed, part_with_keep, roles, sage,
    set, town, town_of, vortex_keep,
};
use crate::record::TownStatic;

/// The fixture's part with the Keep's guild.
fn guild_part() -> TownRuleset {
    part_with_keep(guild_keep)
}

/// The Keep's guild drawn with `sets` from `draws`.
fn draw_from(core: &Ruleset, part: &TownRuleset, sets: &GuildSets, draws: &mut Rng) -> GuildSlots {
    let (record, params) = part.town(KEEP).zip(part.params()).unwrap();
    draw_guild(record, params, core, sets, draws)
}

/// The Keep's guild drawn with `sets` from the towns' stream of `seed`.
fn draw(sets: &GuildSets, seed: u64) -> GuildSlots {
    let (core, part) = (guild_core(&[]), guild_part());
    draw_from(
        &core,
        &part,
        sets,
        &mut Rng::for_stream(seed, StreamId::TOWNS),
    )
}

/// `level`'s six slots.
fn slots_of(slots: &GuildSlots, level: u8) -> Vec<Option<SpellId>> {
    slots.level(level).collect()
}

/// `level`'s drawn spells, sorted, and how many slots it left empty.
fn drawn(slots: &GuildSlots, level: u8) -> (Vec<SpellId>, usize) {
    let mut spells: Vec<SpellId> = slots.level(level).flatten().collect();
    spells.sort_unstable();
    let empty = slots.level(level).filter(Option::is_none).count();
    (spells, empty)
}

// covers: TWN-GLD-002
#[test]
fn each_level_draws_its_weighted_spells_once_then_leaves_its_slots_empty() {
    let (core, part) = (guild_core(&[]), guild_part());
    for seed in 0..32 {
        let slots = draw_from(
            &core,
            &part,
            &GuildSets::default(),
            &mut Rng::for_stream(seed, StreamId::TOWNS),
        );
        // The ability (50) and the bolt (100, by name only) never; a weight of 0 never drawn.
        assert_eq!(drawn(&slots, 1), (vec![L1_A, L1_B, L1_C], 3), "seed {seed}");
        assert_eq!(drawn(&slots, 2), (vec![L2_A, L2_B], 4), "seed {seed}");
        assert_eq!(drawn(&slots, 3), (vec![L3], 5), "seed {seed}");
        assert_eq!(drawn(&slots, 4), (vec![], 6), "seed {seed}");
        // Both of level 5: its slot and the spare.
        assert_eq!(drawn(&slots, 5), (vec![L5_A, L5_B], 4), "seed {seed}");
        for level in 1..=5 {
            let filled = slots_of(&slots, level);
            assert!(filled.windows(2).all(|w| w[0].is_some() || w[1].is_none()));
        }
    }
}

// covers: TWN-GLD-002
#[test]
fn a_slot_takes_the_first_spell_whose_running_weight_reaches_the_roll() {
    let (core, part) = (guild_core(&[]), guild_part());
    let mut seen = [false; 3];
    for seed in 0..64 {
        let mut draws = Rng::for_stream(seed, StreamId::TOWNS);
        let slots = draw_from(&core, &part, &GuildSets::default(), &mut draws);
        // Level 1's first slot: L1_A 10, L1_ZERO 0, L1_B 30, L1_C 60 in spell order (total 100).
        let roll = Rng::for_stream(seed, StreamId::TOWNS).range_inclusive(1, 100);
        let (expected, at) = match roll {
            1..=10 => (L1_A, 0),
            11..=40 => (L1_B, 1),
            _ => (L1_C, 2),
        };
        seen[at] = true;
        assert_eq!(slots.slot(1, 0), Some(expected), "seed {seed}");
    }
    assert_eq!(seen, [true; 3]);
}

// covers: TWN-GLD-002
#[test]
fn a_required_spell_is_taken_first_in_spell_order_whatever_its_weight() {
    let required = SpellSet::of(&[L3, L1_C, L1_ZERO]);
    let sets = GuildSets {
        required,
        ..GuildSets::default()
    };
    for seed in 0..16 {
        let slots = draw(&sets, seed);
        let level_1 = slots_of(&slots, 1);
        assert_eq!(level_1[..2], [Some(L1_ZERO), Some(L1_C)], "seed {seed}");
        assert_eq!(drawn(&slots, 1), (vec![L1_A, L1_ZERO, L1_B, L1_C], 2));
        assert_eq!(slots.slot(3, 0), Some(L3));
    }
    // Every slot a required spell or empty: no draw at all.
    let every = [L1_A, L1_ZERO, L1_B, L1_C, L2_A, L2_B, L3, L5_A, L5_B];
    let sets = GuildSets {
        required: SpellSet::of(&every),
        ..GuildSets::default()
    };
    let (core, part) = (guild_core(&[]), guild_part());
    let mut draws = Rng::for_stream(3, StreamId::TOWNS);
    let slots = draw_from(&core, &part, &sets, &mut draws);
    assert_eq!(draws, Rng::for_stream(3, StreamId::TOWNS));
    assert_eq!(
        slots_of(&slots, 1),
        [
            Some(L1_A),
            Some(L1_ZERO),
            Some(L1_B),
            Some(L1_C),
            None,
            None
        ]
    );
    assert_eq!(slots_of(&slots, 5)[..2], [Some(L5_A), Some(L5_B)]);
}

// covers: TWN-GLD-002
#[test]
fn td_25_a_spell_the_town_requires_and_bans_is_never_in_its_guild() {
    let sets = GuildSets {
        required: SpellSet::of(&[L1_A, L2_A]),
        banned: SpellSet::of(&[L1_A]),
        ..GuildSets::default()
    };
    for seed in 0..16 {
        let slots = draw(&sets, seed);
        assert_eq!(drawn(&slots, 1), (vec![L1_B, L1_C], 4), "seed {seed}");
        assert_eq!(slots.slot(2, 0), Some(L2_A));
    }
}

// covers: TWN-GLD-002
#[test]
fn the_scenario_s_bans_and_a_spell_granted_by_name_only_are_never_drawn() {
    let sets = GuildSets {
        required: SpellSet::of(&[BOLT, ABILITY]),
        scenario: SpellSet::of(&[L5_A, L1_C]),
        ..GuildSets::default()
    };
    for seed in 0..16 {
        let slots = draw(&sets, seed);
        assert_eq!(slots_of(&slots, 5)[..2], [Some(L5_B), None], "seed {seed}");
        assert_eq!(drawn(&slots, 1), (vec![L1_A, L1_B], 4), "seed {seed}");
    }
}

// covers: TWN-GLD-002
#[test]
fn a_level_without_a_spell_left_is_empty() {
    let sets = GuildSets {
        banned: SpellSet::of(&[L2_A, L2_B]),
        ..GuildSets::default()
    };
    assert_eq!(slots_of(&draw(&sets, 5), 2), [None; 6]);
}

// covers: TWN-GLD-002
#[test]
fn a_seed_draws_alike_and_seeds_differ() {
    let sets = GuildSets::default();
    assert_eq!(draw(&sets, 9), draw(&sets, 9));
    let mut levels_1: Vec<Vec<Option<SpellId>>> =
        (0..8).map(|seed| slots_of(&draw(&sets, seed), 1)).collect();
    levels_1.sort_unstable();
    levels_1.dedup();
    assert!(levels_1.len() > 1);
}

#[test]
fn a_slot_outside_the_guild_is_empty() {
    let slots = GuildSlots::EMPTY.with(0, 0, L1_A);
    assert_eq!(slots.slot(1, 0), Some(L1_A));
    assert_eq!(
        [slots.slot(0, 0), slots.slot(6, 0), slots.slot(1, 6)],
        [None; 3]
    );
    assert_eq!(slots_of(&slots, 0), [None; 6]);
    // A slot past the slots is not written.
    assert_eq!(GuildSlots::EMPTY.with(5, 0, L1_A), GuildSlots::EMPTY);
    assert_eq!(GuildSlots::EMPTY.with(0, 6, L1_A), GuildSlots::EMPTY);
}

/// Slots with each level full: level `l`'s slot `s` the spell of handle `10 l + s`.
fn full() -> GuildSlots {
    (0..5).fold(GuildSlots::EMPTY, |slots, index| {
        (0..MAX_GUILD_SLOTS).fold(slots, |slots, slot| {
            let raw = u16::try_from(10 * (index + 1) + slot).unwrap();
            slots.with(index, slot, SpellId::from_raw(raw))
        })
    })
}

/// How many spells `town`'s guild shows of `slots` at levels 1 to 5.
fn counts(slots: &GuildSlots, town: &Town, part: &TownRuleset) -> [usize; 5] {
    [1, 2, 3, 4, 5].map(|level| shown(slots, town, part, level).count())
}

// covers: TWN-GLD-003
#[test]
fn the_guild_level_is_the_highest_standing_mage_guild() {
    let part = guild_part();
    assert_eq!(guild_level(&town(&[]), &part), 0);
    assert_eq!(guild_level(&town(&[MAGE_GUILD_1]), &part), 1);
    let third = town(&[MAGE_GUILD_1, MAGE_GUILD_2, MAGE_GUILD_3]);
    assert_eq!(guild_level(&third, &part), 3);
    assert_eq!(guild_level(&town(&[MAGE_GUILD_5, LIBRARY]), &part), 5);
    // Without the faction's record: none.
    assert_eq!(guild_level(&town_of(WILD, &[]), &part), 0);
    assert_eq!(guild_level(&third, &empty_part()), 0);
}

// covers: TWN-GLD-003
#[test]
fn each_built_level_shows_the_part_s_slots_and_the_library_one_more() {
    let (part, slots) = (guild_part(), full());
    assert_eq!(counts(&slots, &town(&[]), &part), [0; 5]);
    let third = [MAGE_GUILD_1, MAGE_GUILD_2, MAGE_GUILD_3];
    assert_eq!(counts(&slots, &town(&third), &part), [5, 4, 3, 0, 0]);
    let fifth = [MAGE_GUILD_4, MAGE_GUILD_5];
    assert_eq!(counts(&slots, &town(&fifth), &part), [5, 4, 3, 2, 1]);
    // The Library built later: each built level's spare at once.
    let library = [MAGE_GUILD_3, LIBRARY];
    assert_eq!(counts(&slots, &town(&library), &part), [6, 5, 4, 0, 0]);
    let all = [MAGE_GUILD_5, LIBRARY];
    assert_eq!(counts(&slots, &town(&all), &part), [6, 5, 4, 3, 2]);
    // The slots in order, the level's first.
    let shown_2: Vec<SpellId> = shown(&slots, &town(&third), &part, 2).collect();
    assert_eq!(shown_2, [20, 21, 22, 23].map(SpellId::from_raw));
    // No level 0 or 6; nothing without the faction's record or the part's parameters.
    assert_eq!(shown(&slots, &town(&all), &part, 0).count(), 0);
    assert_eq!(shown(&slots, &town(&all), &part, 6).count(), 0);
    assert_eq!(
        counts(&slots, &town_of(WILD, &[MAGE_GUILD_5]), &part),
        [0; 5]
    );
    assert_eq!(counts(&slots, &town(&all), &empty_part()), [0; 5]);
}

// covers: TWN-GLD-003
#[test]
fn an_empty_slot_ends_its_level() {
    let part = guild_part();
    let slots = GuildSlots::EMPTY
        .with(0, 0, L1_A)
        .with(0, 2, L1_B)
        .with(1, 0, L2_A);
    let built = town(&[MAGE_GUILD_2, LIBRARY]);
    assert_eq!(shown(&slots, &built, &part, 1).collect::<Vec<_>>(), [L1_A]);
    assert_eq!(counts(&slots, &built, &part), [1, 1, 0, 0, 0]);
    // A drawn guild shows its draws: the Aurora Borealis changes nothing of it.
    let drawn = draw(&GuildSets::default(), 1);
    let fourth = town(&[MAGE_GUILD_4, AURORA]);
    assert_eq!(counts(&drawn, &fourth, &part), [3, 2, 1, 0, 0]);
}

// covers: TWN-GLD-004
#[test]
fn twn_quirk_14_the_aurora_set_holds_the_scenario_s_bans_only_with_the_flag_on() {
    let sets = GuildSets {
        required: SpellSet::of(&[L4_ZERO]),
        banned: SpellSet::of(&[L2_A]),
        scenario: SpellSet::of(&[L3]),
    };
    let every = [
        L1_A, L1_ZERO, L1_B, L1_C, L2_A, L2_B, L3, L4_ZERO, L5_A, L5_B,
    ];
    let without = |out: &[SpellId]| {
        let spells: Vec<SpellId> = every.into_iter().filter(|s| !out.contains(s)).collect();
        SpellSet::of(&spells)
    };
    // `strict-fixed` (the flag off): both bans respected.
    assert_eq!(aurora_set(&guild_core(&[]), &sets), without(&[L2_A, L3]));
    // `faithful` (on): the scenario's ban taught, the town's not; never the bolt nor the ability.
    let faithful = guild_core(&["quirk.town.aurora_ignores_map_bans"]);
    assert_eq!(aurora_set(&faithful, &sets), without(&[L2_A]));
    assert_eq!(aurora_set(&faithful, &GuildSets::default()), without(&[]));
}

#[test]
fn a_spell_set_holds_spells_by_handle() {
    let set = SpellSet::of(&[L3, L1_A]);
    assert_eq!(set.iter().collect::<Vec<_>>(), [L1_A, L3]);
    assert_eq!(set.bits(), 1 | 1 << 7);
    assert!(set.contains(L3) && !set.contains(L2_A));
    assert_eq!(set.union(SpellSet::of(&[L2_A])).iter().count(), 3);
    let top = SpellId::from_raw(127);
    assert_eq!(SpellSet::EMPTY.with(top).iter().collect::<Vec<_>>(), [top]);
    // A handle past the content's spells is never held.
    let past = SpellId::from_raw(200);
    assert_eq!(set.with(past), set);
    assert!(!set.with(past).contains(past));
    assert_eq!(SpellSet::EMPTY.iter().count(), 0);
}

// covers: TWN-GLD-003
#[test]
fn the_shown_slots_hold_each_level_s_shown_spells_in_order_and_nothing_else() {
    let (part, slots) = (guild_part(), full());
    let level = |shown: &GuildSlots, level| slots_of(shown, level);
    // Three levels, no Library: each level's part's count, in slot order; the rest empty.
    let third = town(&[MAGE_GUILD_1, MAGE_GUILD_2, MAGE_GUILD_3]);
    let held = shown_slots(&slots, &third, &part);
    for at in 1..=5 {
        let spells: Vec<Option<SpellId>> = shown(&slots, &third, &part, at).map(Some).collect();
        let empty = MAX_GUILD_SLOTS - spells.len();
        assert_eq!(
            level(&held, at),
            spells
                .into_iter()
                .chain(vec![None; empty])
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(held.slot(2, 3), Some(SpellId::from_raw(23)));
    assert_eq!(held.slot(2, 4), None);
    // With the Library each built level's spare too; nothing without a guild or a record.
    let library = shown_slots(&slots, &town(&[MAGE_GUILD_5, LIBRARY]), &part);
    assert_eq!(library.level(1).flatten().count(), 6);
    assert_eq!(library.level(5).flatten().count(), 2);
    assert_eq!(shown_slots(&slots, &town(&[]), &part), GuildSlots::EMPTY);
    let wild = town_of(WILD, &[MAGE_GUILD_5]);
    assert_eq!(shown_slots(&slots, &wild, &part), GuildSlots::EMPTY);
}

// covers: TWN-GLD-004
#[test]
fn a_town_has_the_aurora_once_its_building_was_built() {
    let part = guild_part();
    assert!(!has_aurora(&town(&[]), &part));
    assert!(!has_aurora(&town(&[MAGE_GUILD_5, LIBRARY]), &part));
    assert!(has_aurora(&town(&[AURORA]), &part));
    // Without the faction's record: none.
    assert!(!has_aurora(&town_of(WILD, &[AURORA]), &part));
    assert!(!has_aurora(&town(&[AURORA]), &empty_part()));
}

/// Slots of every level's own order (E09 T48a): level 1 `L1_C`, `L1_A`; level 2 `L2_B`, `L2_A`;
/// level 3 `L3`; level 4 none; level 5 `L5_A`, then `L5_B` (the spare: shown with the Library).
fn ordered() -> GuildSlots {
    GuildSlots::EMPTY
        .with(0, 0, L1_C)
        .with(0, 1, L1_A)
        .with(1, 0, L2_B)
        .with(1, 1, L2_A)
        .with(2, 0, L3)
        .with(4, 0, L5_A)
        .with(4, 1, L5_B)
}

/// What [`sage`] (`book`, Wisdom `wisdom`, `known` learned) learns at the Keep built `built` with
/// [`ordered`] slots and the Aurora set `aurora`, in `core`.
fn learned(
    core: &Ruleset,
    (built, aurora): (&[usize], SpellSet),
    (book, wisdom, known): (bool, u8, &[SpellId]),
) -> Vec<SpellId> {
    let town = town(built).with_guild(ordered());
    let statics = TownStatic {
        aurora,
        ..TownStatic::default()
    };
    teaches(
        &town,
        &statics,
        &guild_part(),
        core,
        &sage(core, book, wisdom, known),
    )
}

// covers: TWN-GLD-004
#[test]
fn a_hero_with_a_book_learns_what_the_guild_shows_up_to_its_wisdom_level_then_slot() {
    let core = guild_core(&[]);
    let all = [MAGE_GUILD_5, LIBRARY];
    let five = (&all[..], SpellSet::EMPTY);
    // Wisdom's bound: none 2, Basic 3, Expert 5 (Advanced 4: the guild has no level-4 spell).
    let two = [L1_C, L1_A, L2_B, L2_A];
    assert_eq!(learned(&core, five, (true, 0, &[])), two);
    assert_eq!(
        learned(&core, five, (true, 1, &[])),
        [&two[..], &[L3]].concat()
    );
    assert_eq!(
        learned(&core, five, (true, 2, &[])),
        [&two[..], &[L3]].concat()
    );
    let every = [&two[..], &[L3, L5_A, L5_B]].concat();
    assert_eq!(learned(&core, five, (true, 3, &[])), every);
    // Without the Library level 5 shows its one slot; the guild level bounds what is taught.
    let fifth = [MAGE_GUILD_5];
    let shown_5 = [&two[..], &[L3, L5_A]].concat();
    assert_eq!(
        learned(&core, (&fifth, SpellSet::EMPTY), (true, 3, &[])),
        shown_5
    );
    let second = [MAGE_GUILD_1, MAGE_GUILD_2];
    assert_eq!(
        learned(&core, (&second, SpellSet::EMPTY), (true, 3, &[])),
        two
    );
    // A known spell is skipped; a hero without a book learns nothing.
    assert_eq!(learned(&core, five, (true, 0, &[L1_A, L2_B])), [L1_C, L2_A]);
    assert_eq!(learned(&core, five, (false, 3, &[])), []);
}

// covers: TWN-GLD-004
#[test]
fn nothing_is_taught_without_a_guild_ever_built_or_the_faction_s_record() {
    let core = guild_core(&[]);
    let aurora = aurora_set(&core, &GuildSets::default());
    let hero = (true, 3, &[][..]);
    assert_eq!(learned(&core, (&[], aurora), hero), []);
    // The Aurora Borealis alone teaches nothing either.
    assert_eq!(learned(&core, (&[LIBRARY, AURORA], aurora), hero), []);
    let wild = town_of(WILD, &[MAGE_GUILD_5]).with_guild(ordered());
    let statics = TownStatic::default();
    let details = sage(&core, true, 3, &[]);
    assert_eq!(teaches(&wild, &statics, &guild_part(), &core, &details), []);
    let keep = town(&[MAGE_GUILD_5]).with_guild(ordered());
    assert_eq!(teaches(&keep, &statics, &empty_part(), &core, &details), []);
}

// covers: TWN-GLD-004
#[test]
fn the_aurora_teaches_its_set_in_spell_order_at_any_guild_level_up_to_wisdom() {
    let sets = GuildSets {
        required: SpellSet::EMPTY,
        banned: SpellSet::of(&[L2_A]),
        scenario: SpellSet::of(&[L3]),
    };
    let built = [MAGE_GUILD_1, AURORA];
    // `strict-fixed` (the flag off): neither ban taught; every other guild spell, a weight of 0
    // too, never the ability nor the bolt; level 5 at a level-1 guild with Expert Wisdom.
    let strict = guild_core(&[]);
    let set = (&built[..], aurora_set(&strict, &sets));
    let low = [L1_A, L1_ZERO, L1_B, L1_C, L2_B];
    assert_eq!(learned(&strict, set, (true, 0, &[])), low);
    let high = [&low[..], &[L4_ZERO, L5_A, L5_B]].concat();
    assert_eq!(learned(&strict, set, (true, 3, &[])), high);
    let known = [L1_A, L1_ZERO, L1_C, L2_B, L4_ZERO, L5_A, L5_B];
    assert_eq!(learned(&strict, set, (true, 3, &[L1_B])), known);
    assert_eq!(learned(&strict, set, (false, 3, &[])), []);
    // `faithful` (on, T-18): the scenario's ban taught, the town's never.
    let faithful = guild_core(&["quirk.town.aurora_ignores_map_bans"]);
    let set = (&built[..], aurora_set(&faithful, &sets));
    let all = [L1_A, L1_ZERO, L1_B, L1_C, L2_B, L3, L4_ZERO, L5_A, L5_B];
    assert_eq!(learned(&faithful, set, (true, 3, &[])), all);
    assert!(!all.contains(&ABILITY) && !all.contains(&BOLT));
}

/// [`sage`] without a book in `core` with `mana` spell points, and its maximum (20: Knowledge 2).
fn sage_at(core: &Ruleset, mana: u16) -> (HeroDetails, u16) {
    let mut hero = sage(core, false, 0, &[]);
    hero.mana = mana;
    let max = h3_heroes::max_mana(&hero, core);
    (hero, max)
}

// covers: TWN-GLD-008
#[test]
fn a_night_refills_in_a_town_with_a_guild_ever_built() {
    let part = guild_part();
    // Any guild level, even one superseded since.
    for built in [
        &[MAGE_GUILD_1][..],
        &[MAGE_GUILD_1, MAGE_GUILD_3],
        &[MAGE_GUILD_5],
    ] {
        assert!(refills(&town(built), &part));
    }
    // No guild ever built (the Library alone), no faction's record: none.
    assert!(!refills(&town(&[LIBRARY]), &part));
    assert!(!refills(&town_of(WILD, &[MAGE_GUILD_1]), &part));
    assert!(!refills(&town(&[MAGE_GUILD_1]), &empty_part()));
}

// covers: TWN-GLD-009
#[test]
fn a_standing_mana_vortex_with_the_week_s_flag_gives_the_factor_times_the_maximum() {
    let (core, part) = (guild_core(&[]), part_with_keep(vortex_keep));
    let keep = town(&[MAGE_GUILD_1, VORTEX]);
    let (empty, max) = sage_at(&core, 0);
    assert_eq!(max, 20);
    assert!(keep.vortex());
    assert_eq!(vortex(&keep, &part, &core, &empty), Some(2 * max));
    // Below twice the maximum it gives; at or above it nothing (the flag stays for the next).
    let points = |mana| vortex(&keep, &part, &core, &sage_at(&core, mana).0);
    assert_eq!(points(39), Some(40));
    assert_eq!((points(40), points(55)), (None, None));
    // The part's factor; the flag cleared, the Vortex not standing, never built: nothing.
    let five = part_changed(|p| p.vortex_factor = 5, vortex_keep);
    assert_eq!(vortex(&keep, &five, &core, &empty), Some(5 * max));
    assert_eq!(vortex(&keep.with_vortex(false), &part, &core, &empty), None);
    let superseded = part_with_keep(|keep| {
        vortex_keep(keep);
        keep.buildings[LIBRARY].supersedes = set(&[VORTEX]);
    });
    let both = town(&[MAGE_GUILD_1, VORTEX, LIBRARY]);
    assert_eq!(vortex(&both, &superseded, &core, &empty), None);
    assert_eq!(vortex(&town(&[MAGE_GUILD_1]), &part, &core, &empty), None);
    // No faction's record, no parameters: nothing.
    let wild = town_of(WILD, &[MAGE_GUILD_1, VORTEX]);
    assert_eq!(vortex(&wild, &part, &core, &empty), None);
    assert_eq!(vortex(&keep, &empty_part(), &core, &empty), None);
}

/// Whether an opening of `town` gives one of `heroes` anything, by [`vortex`] and [`teaches`]
/// asked hero by hero, checked against [`opening_gives`] (E09 T49a2); since E09 T49b also against
/// [`guild_offers`] with the first hero the visitor and the second the garrison hero, its book
/// against [`check_spell_book`] and the book hero's spell book.
fn gives(
    (town, statics): (&Town, &TownStatic),
    part: &TownRuleset,
    core: &Ruleset,
    heroes: &[HeroDetails],
) -> bool {
    let expected = heroes.iter().any(|hero| {
        vortex(town, part, core, hero).is_some()
            || !teaches(town, statics, part, core, hero).is_empty()
    });
    assert_eq!(
        opening_gives(town, statics, part, core, heroes),
        expected,
        "{:?} {:?}",
        town.built(),
        heroes
            .iter()
            .map(|h| (h.mana, &h.spells))
            .collect::<Vec<_>>()
    );
    let (visitor, garrison) = (heroes.first(), heroes.get(1));
    let book = visitor
        .or(garrison)
        .and_then(|_| check_spell_book(town, part).ok());
    let offers = guild_offers(town, statics, part, core, (visitor, garrison));
    assert_eq!(
        offers,
        GuildOffers {
            opening: expected,
            book
        },
        "{:?}",
        town.built()
    );
    // Either hero alone as the garrison hero.
    let alone = guild_offers(town, statics, part, core, (None, visitor));
    assert_eq!(alone.book, visitor.and(book), "{:?}", town.built());
    expected
}

// covers: TWN-GLD-006, TWN-GLD-007
#[test]
fn an_opening_gives_exactly_where_the_vortex_or_the_guild_would() {
    let core = guild_core(&[]);
    let aurora = aurora_set(&core, &GuildSets::default());
    let superseded = part_with_keep(|keep| {
        vortex_keep(keep);
        keep.buildings[LIBRARY].supersedes = set(&[VORTEX]);
    });
    let parts = [
        guild_part(),
        part_with_keep(vortex_keep),
        superseded,
        empty_part(),
    ];
    let builts: [&[usize]; 9] = [
        &[],
        &[MAGE_GUILD_1],
        &[MAGE_GUILD_1, MAGE_GUILD_3],
        &[MAGE_GUILD_5, LIBRARY],
        &[MAGE_GUILD_1, AURORA],
        &[LIBRARY, AURORA],
        &[MAGE_GUILD_1, VORTEX],
        &[MAGE_GUILD_1, VORTEX, LIBRARY],
        &[VORTEX],
    ];
    let shown = [L1_C, L1_A, L2_B, L2_A, L3, L5_A, L5_B];
    let mut heroes = Vec::new();
    for (book, wisdom) in [(false, 0), (true, 0), (true, 3)] {
        for known in [&[][..], &[L1_C, L1_A], &shown] {
            for mana in [0, 40] {
                let mut hero = sage(&core, book, wisdom, known);
                hero.mana = mana;
                heroes.push(hero);
            }
        }
    }
    let mut outcomes = [0; 2];
    for part in &parts {
        for built in builts {
            for flag in [true, false] {
                let keep = town(built).with_guild(ordered()).with_vortex(flag);
                let statics = TownStatic {
                    aurora,
                    ..TownStatic::default()
                };
                for (at, hero) in heroes.iter().enumerate() {
                    let one = gives((&keep, &statics), part, &core, core::slice::from_ref(hero));
                    outcomes[usize::from(one)] += 1;
                    // Two heroes: either's gift counts.
                    let other = &heroes[(at + 5) % heroes.len()];
                    gives(
                        (&keep, &statics),
                        part,
                        &core,
                        &[hero.clone(), other.clone()],
                    );
                }
            }
        }
    }
    assert!(outcomes[0] > 0 && outcomes[1] > 0, "{outcomes:?}");
    // No hero, no faction's record: nothing.
    let keep = town(&[MAGE_GUILD_5, VORTEX]).with_guild(ordered());
    let statics = TownStatic::default();
    assert!(!opening_gives(&keep, &statics, &parts[1], &core, &[]));
    let none = GuildOffers {
        opening: false,
        book: None,
    };
    let offers = guild_offers(&keep, &statics, &parts[1], &core, (None, None));
    assert_eq!(offers, none);
    let wild = town_of(WILD, &[MAGE_GUILD_5, VORTEX]).with_guild(ordered());
    assert!(!gives((&wild, &statics), &parts[1], &core, &heroes[..1]));
}

// covers: TWN-GLD-007
#[test]
fn a_spell_book_is_sold_where_a_mage_guild_stands_for_the_part_s_price() {
    let part = guild_part();
    let price: &[(ResourceId, u32)] = &[(GOLD, 500)];
    fn sells<'a>(
        built: &[usize],
        part: &'a TownRuleset,
    ) -> Result<&'a [(ResourceId, u32)], TownCommandError> {
        check_spell_book(&town(built), part)
    }
    assert_eq!(sells(&[MAGE_GUILD_1], &part), Ok(price));
    // Any level: the highest, standing over the lower ones.
    let third = [MAGE_GUILD_1, MAGE_GUILD_2, MAGE_GUILD_3];
    assert_eq!(sells(&third, &part), Ok(price));
    assert_eq!(sells(&[MAGE_GUILD_5, LIBRARY, AURORA], &part), Ok(price));
    // No guild: nothing else of the guild's sells one.
    for built in [&[][..], &[LIBRARY], &[AURORA]] {
        assert_eq!(sells(built, &part), Err(TownCommandError::NoGuild));
    }
    // A guild ever built but superseded by a building that is none stands no more.
    let ruin = part_with_keep(|keep| {
        guild_keep(keep);
        keep.buildings
            .push(building("ruin", &[MAGE_GUILD_1], roles(|_| {})));
    });
    assert_eq!(
        sells(&[MAGE_GUILD_1, 31], &ruin),
        Err(TownCommandError::NoGuild)
    );
    assert_eq!(sells(&[MAGE_GUILD_1], &ruin), Ok(price));
    // The part's price, whatever it is.
    let dear = part_changed(|p| p.spell_book_price = vec![(GOLD, 700)], guild_keep);
    let dearer: &[(ResourceId, u32)] = &[(GOLD, 700)];
    assert_eq!(sells(&[MAGE_GUILD_2], &dear), Ok(dearer));
    // Without the faction's record or the part's parameters: none.
    let wild = town_of(WILD, &[MAGE_GUILD_1]);
    assert_eq!(
        check_spell_book(&wild, &part),
        Err(TownCommandError::NoGuild)
    );
    assert_eq!(
        sells(&[MAGE_GUILD_1], &empty_part()),
        Err(TownCommandError::NoGuild)
    );
}
