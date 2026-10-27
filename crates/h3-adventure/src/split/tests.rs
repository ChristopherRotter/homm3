//! Tests of a wandering monster's split (ADV-MON-009, E08 T20b2): the tile's generator, the
//! table, the draws, the even stacks and the upgraded middle stack; since E08 T62b1 the split with
//! a second creature (ADV-MON-012): its stacks taken off, its order.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, FactionDef, PackManifest, ProfileDef,
};

use super::*;

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

const PIKEMAN: CreatureTypeId = CreatureTypeId::from_raw(0);
const HALBERDIER: CreatureTypeId = CreatureTypeId::from_raw(1);
const PEASANT: CreatureTypeId = CreatureTypeId::from_raw(2);
const STONE: CreatureTypeId = CreatureTypeId::from_raw(3);

/// A pikeman (AI value 100) of the castle upgrading to a halberdier (115), a neutral peasant (10)
/// that names an upgrade, a stone (0) of the castle.
fn core() -> Ruleset {
    let creature = |k: &str, faction: Option<&str>, ai_value, upgrades: &[&str]| CreatureDef {
        key: key(k),
        speed: 5,
        info: CreatureInfoDef {
            faction: faction.map(key),
            ai_value,
            upgrades: upgrades.iter().map(|u| key(u)).collect(),
            ..CreatureInfoDef::default()
        },
    };
    let pack = ContentPack {
        factions: vec![FactionDef {
            key: key("t:castle"),
            name: "Castle".into(),
            native_terrain: None,
        }],
        creatures: vec![
            creature("t:pikeman", Some("t:castle"), 100, &["t:halberdier"]),
            creature("t:halberdier", Some("t:castle"), 115, &[]),
            creature("t:peasant", None, 10, &["t:pikeman"]),
            creature("t:stone", Some("t:castle"), 0, &[]),
        ],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(PackManifest {
            id: "t".into(),
            version: "1".into(),
            depends: vec![],
        })
    };
    Ruleset::compile(&[pack], "t:faithful").unwrap()
}

/// `count` peasants in slot 0.
fn peasants(count: u32) -> Vec<ArmyStack> {
    vec![ArmyStack {
        slot: 0,
        creature: PEASANT,
        count,
    }]
}

/// The split of `monster` against `visitor` with these draws, and how many it took.
fn split_with(
    visitor: &[ArmyStack],
    monster: (CreatureTypeId, u32),
    draws: &[u32],
) -> (Vec<ArmyStack>, usize) {
    let core = core();
    let mut taken = 0;
    let army = split(&core, visitor, monster, || {
        taken += 1;
        draws[taken - 1]
    });
    (army, taken)
}

/// The stacks' counts of a split.
fn counts(army: &[ArmyStack]) -> Vec<u32> {
    army.iter().map(|s| s.count).collect()
}

// covers: ADV-MON-009
#[test]
fn the_tiles_generator_is_the_c_runtimes_seeded_by_the_tile() {
    // MSVC's rand after srand(1): 41, 18467, 6334, 26500, 19169.
    let mut one = TileRand(1);
    let drawn: Vec<u32> = (0..5).map(|_| one.rand()).collect();
    assert_eq!(drawn, [41, 18_467, 6_334, 26_500, 19_169]);
    // 248 071 x + 276 589 y + 310 111 z + 155 303: (12, 27) underground is 10 910 169, whose
    // second draw of 1 to 100 is 50 (survey section 4.1, D-11).
    assert_eq!(TileRand::new(Pos::new(12, 27, 1)), TileRand(10_910_169));
    let mut tile = TileRand::new(Pos::new(12, 27, 1));
    assert_eq!([tile.percent(), tile.percent()], [44, 50]);
    let mut origin = TileRand::new(Pos::new(0, 0, 0));
    assert_eq!([origin.percent(), origin.percent()], [73, 21]);
    // 32 bits: a far tile wraps.
    let mut far = TileRand::new(Pos::new(100, 140, 1));
    assert_eq!([far.percent(), far.percent()], [61, 43]);
}

// covers: ADV-MON-009
#[test]
fn the_number_of_stacks_follows_the_table() {
    // 100 peasants (a strength of 1 000) against peasants worth 10 each; a neutral creature
    // takes no second draw.
    let stacks = |value: u32| {
        let (army, taken) = split_with(&peasants(value / 10), (PEASANT, 100), &[50]);
        assert_eq!(taken, 1);
        army.len()
    };
    let table = [
        (490, 7),
        (500, 6),
        (660, 6),
        // 0.67 as a double: 670 / 1 000 is the same double.
        (670, 5),
        (990, 5),
        (1_000, 4),
        (1_490, 4),
        (1_500, 3),
        (1_990, 3),
        (2_000, 2),
        (100_000, 2),
    ];
    for (value, expected) in table {
        assert_eq!(stacks(value), expected, "{value}");
    }
}

// covers: ADV-MON-009
#[test]
fn the_first_draw_moves_the_number_by_one_and_the_count_bounds_it() {
    let one = |draw| split_with(&peasants(100), (PEASANT, 100), &[draw]).0.len();
    // Base 4 (a ratio of 1).
    assert_eq!(one(1), 3);
    assert_eq!(one(20), 3);
    assert_eq!(one(21), 4);
    assert_eq!(one(79), 4);
    assert_eq!(one(80), 5);
    assert_eq!(one(100), 5);
    // At seven, no more.
    assert_eq!(split_with(&peasants(1), (PEASANT, 100), &[90]).0.len(), 7);
    // At most the count, at least one.
    assert_eq!(split_with(&peasants(1), (PEASANT, 3), &[50]).0.len(), 3);
    assert_eq!(split_with(&peasants(1_000), (PEASANT, 3), &[10]).0.len(), 1);
}

// covers: ADV-MON-009
#[test]
fn the_count_divides_evenly_the_first_stacks_one_more() {
    // 23 peasants (230) against 230: base 4, one more: 5 stacks.
    let (army, _) = split_with(&peasants(23), (PEASANT, 23), &[90]);
    assert_eq!(counts(&army), [5, 5, 5, 4, 4]);
    let slots: Vec<u8> = army.iter().map(|s| s.slot).collect();
    assert_eq!(slots, [0, 1, 2, 3, 4]);
    assert!(army.iter().all(|s| s.creature == PEASANT));
}

// covers: ADV-MON-009
#[test]
fn a_town_creatures_middle_stack_is_upgraded_on_the_second_draw() {
    // 10 pikemen (1 000) against peasants worth 1 000: 4 stacks, the third (index 2) upgraded.
    let creatures = |draws: &[u32]| {
        let (army, taken) = split_with(&peasants(100), (PIKEMAN, 10), draws);
        let creatures: Vec<CreatureTypeId> = army.iter().map(|s| s.creature).collect();
        (creatures, counts(&army), taken)
    };
    assert_eq!(
        creatures(&[50, 50]),
        (
            vec![PIKEMAN, PIKEMAN, HALBERDIER, PIKEMAN],
            vec![3, 3, 2, 2],
            2
        )
    );
    assert_eq!(
        creatures(&[50, 51]),
        (vec![PIKEMAN; 4], vec![3, 3, 2, 2], 2)
    );
    // Three stacks: the middle one.
    assert_eq!(creatures(&[1, 1]).0, [PIKEMAN, HALBERDIER, PIKEMAN]);
    // One stack: no second draw.
    let (army, taken) = split_with(&peasants(100), (PIKEMAN, 1), &[50]);
    assert_eq!((army.len(), taken), (1, 1));
    // An upgrade (no upgrade of its own) takes no second draw.
    let (_, taken) = split_with(&peasants(100), (HALBERDIER, 10), &[50]);
    assert_eq!(taken, 1);
}

// covers: ADV-MON-009
#[test]
fn a_monster_of_no_strength_splits_as_the_fpus_infinity_or_nan() {
    // A strength of 0 against an army worth something: 2 stacks; worth nothing too: 7.
    assert_eq!(split_with(&peasants(5), (STONE, 10), &[50]).0.len(), 2);
    assert_eq!(split_with(&[], (STONE, 10), &[50]).0.len(), 7);
}

/// The split of 40 pikemen with peasants in `fixed` stacks against `visitor` with these draws, as
/// (slot, creature, count), and how many draws it took.
fn split_second(
    visitor: &[ArmyStack],
    first: u32,
    fixed: &[u32],
    draws: &[u32],
) -> (Vec<(u8, CreatureTypeId, u32)>, usize) {
    let core = core();
    let mut taken = 0;
    let army = split_with_second(&core, visitor, (PIKEMAN, first), (PEASANT, fixed), || {
        taken += 1;
        draws[taken - 1]
    });
    let army = army
        .into_iter()
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    (army, taken)
}

// covers: ADV-MON-012
#[test]
fn a_second_creatures_stacks_come_off_the_first_and_the_slots_follow_the_originals_order() {
    // 40 pikemen (4 000) and 20 peasants in 2 stacks (200): a strength of 4 200. The visitor's
    // peasants set the ratio: base 7, 6, 5, 4, 3 less the 2 stacks gives 5 to 1 stacks of
    // pikemen, ordered by 0x63DF94's rows 35, 28, 21, 14, 7 (survey E08-rewards.md section 7.5).
    let (g, d) = (PIKEMAN, PEASANT);
    let rows = [
        (
            200,
            vec![
                (0, g, 8),
                (1, g, 8),
                (2, d, 10),
                (3, g, 8),
                (4, d, 10),
                (5, g, 8),
                (6, g, 8),
            ],
        ),
        (
            250,
            vec![
                (0, g, 10),
                (1, d, 10),
                (2, g, 10),
                (3, d, 10),
                (4, g, 10),
                (6, g, 10),
            ],
        ),
        (
            400,
            vec![(0, d, 10), (1, g, 14), (2, d, 10), (3, g, 13), (5, g, 13)],
        ),
        (500, vec![(0, d, 10), (1, d, 10), (2, g, 20), (4, g, 20)]),
        (700, vec![(0, d, 10), (1, d, 10), (3, g, 40)]),
        // Base 2 less 2: at least one stack.
        (1_000, vec![(0, d, 10), (1, d, 10), (3, g, 40)]),
    ];
    for (peasants_count, expected) in rows {
        let (army, taken) = split_second(&peasants(peasants_count), 40, &[10, 10], &[50]);
        assert_eq!(army, expected, "{peasants_count}");
        // One draw: no upgrade of the town's pikemen beside a second creature.
        assert_eq!(taken, 1);
    }
}

// covers: ADV-MON-012
#[test]
fn the_draw_and_the_count_bound_the_first_creatures_stacks() {
    let pikemen =
        |army: &[(u8, CreatureTypeId, u32)]| army.iter().filter(|&&(_, c, _)| c == PIKEMAN).count();
    // Base 6 (250 peasants), one more on 80: 5 stacks; one fewer on 20: 3.
    assert_eq!(
        pikemen(&split_second(&peasants(250), 40, &[10, 10], &[90]).0),
        5
    );
    assert_eq!(
        pikemen(&split_second(&peasants(250), 40, &[10, 10], &[20]).0),
        3
    );
    // At most the first creature's count.
    let (army, _) = split_second(&peasants(1), 2, &[10, 10], &[50]);
    assert_eq!(pikemen(&army), 2);
    // One second stack: up to 6 of the first, row 42.
    let (army, _) = split_second(&peasants(1), 60, &[5], &[50]);
    let slots: Vec<(u8, CreatureTypeId)> = army.iter().map(|&(s, c, _)| (s, c)).collect();
    let g = PIKEMAN;
    assert_eq!(
        slots,
        [(0, g), (1, g), (2, g), (3, PEASANT), (4, g), (5, g), (6, g)]
    );
    assert!(
        army.iter()
            .filter(|&&(_, c, _)| c == g)
            .all(|&(_, _, n)| n == 10)
    );
}
