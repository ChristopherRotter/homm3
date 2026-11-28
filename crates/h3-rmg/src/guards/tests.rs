//! The guards on small tables (RMG-CON-018, RMG-CON-021): the guard value's tables, the
//! towns allowed, the fitting creatures, the draw's walk from the top (RMG-QUIRK-10's both
//! halves), the count's spread, the serial and the faults. The corpus's against the phase oracle
//! are `h3-pack-rmg`'s.

use alloc::vec;
use alloc::vec::Vec;

use h3_kernel::rng::CrtRand;

use super::*;
use crate::generate::StepState;
use crate::objects::tests::{line, settings};
use crate::part::{ObjectTemplate, Strength, ZoneKind};

#[derive(Debug, Default)]
struct Log {
    draws: Vec<(u32, u32)>,
}

impl Trace for Log {
    fn draw(&mut self, site: u32, value: u32) {
        self.draws.push((site, value));
    }

    fn step_end(&mut self, _step: u8, _state: StepState<'_>) {}
}

/// A zone allowing the neutral monsters and the towns of `towns` (bits Castle to Conflux).
fn zone(neutral: bool, towns: u16) -> Zone {
    Zone {
        id: 0,
        kind: ZoneKind::Treasure,
        base_size: 10,
        humans: [0, 8],
        players: [0, 8],
        ownership: -1,
        player_towns: [0; 4],
        neutral_towns: [0; 4],
        same_type: false,
        town_types: 0,
        mines: [0; 7],
        mine_density: [0; 7],
        terrain_match: false,
        terrains: 0,
        strength: Strength::Average,
        monster_match: false,
        neutral_monsters: neutral,
        monster_towns: towns,
        treasure: [[0; 3]; 3],
    }
}

/// A creature of `faction` and `level`, AI value `ai`, the counts `counts`.
fn creature(faction: i32, level: i32, ai: i32, counts: [i32; 2]) -> GuardCreature {
    GuardCreature {
        faction,
        level,
        ai_value: ai,
        counts,
    }
}

/// The table of `len` ids, each [`GuardCreature::UNUSED`] but `set`'s.
fn table(len: usize, set: &[(usize, GuardCreature)]) -> Vec<GuardCreature> {
    let mut t = vec![GuardCreature::UNUSED; len];
    for &(id, c) in set {
        t[id] = c;
    }
    t
}

/// A monster line of subclass `subclass`, its anchor its entrance.
fn monster(subclass: u32) -> ObjectTemplate {
    ObjectTemplate {
        actions: 1,
        ..line(54, subclass)
    }
}

/// The objects of the monster lines of `subclasses`, in that order, in `format`.
fn monsters(subclasses: &[u32], format: MapFormat) -> Objects {
    let lines: Vec<ObjectTemplate> = subclasses.iter().map(|&s| monster(s)).collect();
    Objects::new(&lines, &settings(), format, 1)
}

/// A guard monster of `value` from `creatures` and `objects` in `format`, with `walk`, for
/// `zone` and `town_type`, drawn from `seed`: the monster and the draws.
#[allow(clippy::too_many_arguments)]
fn guard(
    creatures: &[GuardCreature],
    objects: &mut Objects,
    format: MapFormat,
    walk: bool,
    value: i32,
    zone: &Zone,
    town_type: i32,
    seed: u32,
) -> (Option<NewMonster>, Vec<(u32, u32)>) {
    let input = GuardInput {
        creatures,
        format,
        strength: 3,
        roe_guard_walk: walk,
    };
    let mut log = Log::default();
    let m = create(
        &mut Stream::new(seed, &mut log),
        &input,
        objects,
        value,
        zone,
        town_type,
    );
    (m, log.draws)
}

/// The prototype of `subclass` among `objects`' monsters (the last of it).
fn prototype(objects: &Objects, subclass: u32) -> usize {
    objects.with_subclass(54, subclass).last().unwrap()
}

// covers: RMG-CON-018
/// The guard value by the monster strength's tables: the low part above the low threshold, the
/// high part above the high one, quarters truncating, none under 2000; strength 0 guards
/// nothing; a strength past the tables takes their ends.
#[test]
fn the_guard_value_takes_the_strengths_thresholds_and_factors() {
    assert_eq!(guard_value(1_500, 3), 0);
    assert_eq!(guard_value(3_000, 3), 2_000);
    assert_eq!(guard_value(10_000, 3), 9_000 + 2_500);
    assert_eq!(guard_value(10_001, 2), (8_501 * 3) / 4 + (2_501 * 3) / 4);
    assert_eq!(guard_value(60_000, 0), 0);
    assert_eq!(guard_value(6_000, 5), 9_000 + 1_500);
    assert_eq!(guard_value(6_000, 9), guard_value(6_000, 5));
    assert_eq!(guard_value(60_000, -1), 0);
    assert_eq!(guard_value(-5, 5), 0);
}

// covers: RMG-CON-021
/// The creatures fitting a value: AI × ((low + high) / 2) up to AI × 100, level 0 or more, an
/// allowed town; the draw takes the r-th with a prototype from 144 down, the last prototype of
/// its subclass; its count (value + AI / 2) / AI spread by two draws of n / 4 + 1; the monster
/// takes the serial and raises its prototype's use count.
#[test]
fn a_guard_monster_is_drawn_among_the_fitting_creatures_from_the_top() {
    let creatures = table(
        GUARD_CREATURES,
        &[
            (0, creature(0, 0, 100, [10, 30])),
            (5, creature(-1, 1, 500, [4, 8])),
            // Too strong for 4000 (500 × 10), too weak (4000 > 30 × 100), of a town not allowed.
            (6, creature(-1, 2, 500, [8, 12])),
            (7, creature(-1, 0, 30, [1, 1])),
            (8, creature(3, 0, 100, [10, 30])),
            (144, creature(-1, 0, 100, [10, 30])),
        ],
    );
    let all = zone(true, 0x1FF & !(1 << 3));
    let mut objects = monsters(&[0, 5, 5, 6, 7, 8, 144], MapFormat::SoD);
    let picks: Vec<(usize, usize)> = (0..40)
        .map(|seed| {
            let first = CrtRand::new(seed).rand();
            let (m, draws) = guard(
                &creatures,
                &mut objects,
                MapFormat::SoD,
                true,
                4_000,
                &all,
                -1,
                seed,
            );
            let m = m.unwrap();
            assert_eq!(draws[0], (SITE_GUARD_CREATURE, first));
            // 144, 5 and 0 fit: from the top.
            let (sub, n) = [(144, 40), (5, 8), (0, 40)][first as usize % 3];
            assert_eq!(m.prototype, prototype(&objects, sub));
            let s = n / 4 + 1;
            assert_eq!(
                draws[1..],
                [
                    (SITE_GUARD_COUNT_PLUS, draws[1].1),
                    (SITE_GUARD_COUNT_MINUS, draws[2].1)
                ]
            );
            assert_eq!(m.count, n + draws[1].1 as i32 % s - draws[2].1 as i32 % s);
            (sub as usize, m.serial as usize)
        })
        .collect();
    // Each fitting creature drawn; the serials run from 1.
    for sub in [0, 5, 144] {
        assert!(picks.iter().any(|&(s, _)| s == sub));
    }
    assert_eq!(
        picks.iter().map(|&(_, s)| s).collect::<Vec<_>>(),
        (1..=40).collect::<Vec<_>>()
    );
    assert_eq!(objects.serial(), 41);
    let used: i32 = [0, 5, 144]
        .iter()
        .map(|&s| objects.prototypes()[prototype(&objects, s)].used)
        .sum();
    assert_eq!(used, 40);
    // A count of at most 3 is not spread.
    let small = table(1, &[(0, creature(-1, 0, 1_000, [1, 3]))]);
    let (m, draws) = guard(
        &small,
        &mut objects,
        MapFormat::SoD,
        true,
        2_500,
        &all,
        -1,
        1,
    );
    assert_eq!(m.unwrap().count, 3);
    assert_eq!(draws.len(), 1);
}

// covers: RMG-CON-021
/// The towns allowed: a zone whose monsters match its town takes that town type alone (a type
/// past Conflux allows none), else its neutral byte and its monster towns; nothing fitting draws
/// nothing; value 0 fits the unused ids' zeros, which their level -1 refuses.
#[test]
fn the_towns_allowed_follow_the_zones_monster_settings() {
    let creatures = table(
        GUARD_CREATURES,
        &[
            (0, creature(0, 0, 100, [10, 30])),
            (1, creature(-1, 0, 100, [10, 30])),
            (2, creature(9, 0, 100, [10, 30])),
            (3, creature(-2, 0, 100, [10, 30])),
        ],
    );
    let mut objects = monsters(&[0, 1, 2, 3], MapFormat::SoD);
    let matching = Zone {
        monster_match: true,
        ..zone(true, 0)
    };
    let (castle, neutral) = (prototype(&objects, 0), prototype(&objects, 1));
    for (zone, town_type, fits) in [
        (&matching, 0, Some(castle)),
        (&matching, 9, None),
        (&matching, -1, Some(neutral)),
        (&zone(false, 1), -1, Some(castle)),
        (&zone(false, 0), 2, None),
    ] {
        let (m, draws) = guard(
            &creatures,
            &mut objects,
            MapFormat::SoD,
            true,
            3_000,
            zone,
            town_type,
            7,
        );
        assert_eq!(m.map(|m| m.prototype), fits);
        assert_eq!(draws.is_empty(), fits.is_none());
    }
    let (m, draws) = guard(
        &creatures,
        &mut objects,
        MapFormat::SoD,
        true,
        0,
        &zone(true, 0x1FF),
        -1,
        7,
    );
    assert_eq!((m, draws.len()), (None, 0));
}

// covers: RMG-CON-021
/// RoE filters and counts the creatures 0 to 116 and drops the prototypes from 118, but walks
/// from 144 (RMG-QUIRK-10 as the original): creature 117's prototype is drawn uncounted. Without
/// the walk it starts at 116.
#[test]
fn roe_walks_from_144_and_draws_creature_117_uncounted() {
    let creatures = table(
        GUARD_CREATURES,
        &[
            (0, creature(-1, 0, 100, [10, 30])),
            (117, creature(-1, 5, 50, [1, 1])),
            (118, creature(-1, 0, 100, [10, 30])),
        ],
    );
    let all = zone(true, 0x1FF);
    for (walk, sub, n) in [(true, 117, 60), (false, 0, 30)] {
        let mut objects = monsters(&[0, 117, 118], MapFormat::RoE);
        let (m, draws) = guard(
            &creatures,
            &mut objects,
            MapFormat::RoE,
            walk,
            3_000,
            &all,
            -1,
            3,
        );
        let m = m.unwrap();
        assert_eq!(m.prototype, prototype(&objects, sub));
        assert_eq!(draws.len(), 3);
        let s = n / 4 + 1;
        assert_eq!(m.count, n + draws[1].1 as i32 % s - draws[2].1 as i32 % s);
    }
}

// covers: RMG-CON-021
/// The original's faults (RMG-QUIRK-3, -4), each recorded with no monster: a monster prototype
/// past 144 (no draw), a counted creature without a prototype walked past, creature 117 walked
/// to in RoE without an AI value (the count's division).
#[test]
fn a_guard_monsters_faults_are_recorded() {
    let creatures = table(GUARD_CREATURES, &[(7, creature(-1, 0, 100, [10, 30]))]);
    let all = zone(true, 0x1FF);
    let mut objects = monsters(&[7, 145], MapFormat::SoD);
    let (m, draws) = guard(
        &creatures,
        &mut objects,
        MapFormat::SoD,
        true,
        3_000,
        &all,
        -1,
        1,
    );
    assert_eq!((m, draws.len()), (None, 0));
    assert_eq!(objects.take_fault(), Some(ObjectFault::Creature));
    let mut objects = monsters(&[6], MapFormat::SoD);
    let (m, draws) = guard(
        &creatures,
        &mut objects,
        MapFormat::SoD,
        true,
        3_000,
        &all,
        -1,
        1,
    );
    assert_eq!((m, draws.len()), (None, 1));
    assert_eq!(objects.take_fault(), Some(ObjectFault::Creature));
    // A table of 117 ids: creature 117, walked to, has none.
    let short = table(117, &[(7, creature(-1, 0, 100, [10, 30]))]);
    let mut objects = monsters(&[7, 117], MapFormat::RoE);
    let (m, draws) = guard(
        &short,
        &mut objects,
        MapFormat::RoE,
        true,
        3_000,
        &all,
        -1,
        1,
    );
    assert_eq!((m, draws.len()), (None, 1));
    assert_eq!(objects.take_fault(), Some(ObjectFault::Creature));
    assert_eq!(objects.serial(), 1);
}
