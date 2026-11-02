//! Tests of the day's start's steps (E08 T85a; increment 9's design note §10, §11): the week's
//! step (the weekly objects filled again, the monsters' growth in sixteenths to the cap, batch
//! E9's E9-8 to E9-10) and a creature month's step (its monsters on free land tiles, as placed
//! ones; the draws' order).

use std::boxed::Box;
use std::vec;
use std::vec::Vec;

use core::num::{NonZeroU8, NonZeroU16};

use h3_content::ContentPack;
use h3_kernel::{Rng, StreamId};

use super::economy::{GRASS_ROWS, object, setup};
use super::*;
use crate::command::AdventureEvent;
use crate::ruleset::{AdventurePack, MonthParamsDef, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::{EventSetup, Holds, RewardRecord};
use crate::testing::{
    MONSTER, PEASANT, PIKEMAN, WOLF, hero_at, key, movement_rules, movement_rules_with, started,
    world,
};
use crate::{MonsterView, ObjectView};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
const WOOD: ResourceId = ResourceId::from_raw(0);
const GOLD: ResourceId = ResourceId::from_raw(1);

/// The faithful movement rules with the three weekly kinds of the economy tests' (a Windmill of
/// wood or gold, 3 to 6; a Water Wheel of 500 gold, then 1 000; a Garden of 500 gold or 5 wood),
/// an event's type, and a month of wolves (2 to 5 on the map, so 4 to 10 a monster) with one
/// chance in `one_in` a tile and an aggression of 1 to 10.
fn rules(one_in: u16) -> Rules {
    movement_rules_with(
        true,
        16,
        |core: &mut ContentPack| core.creatures[3].info.adventure_count = [2, 5],
        |part: &mut AdventurePack| {
            part.params.as_mut().unwrap().months = Some(MonthParamsDef {
                odds: [5, 4, 1],
                creatures: vec![key("t:wolf")],
                monster: key("t:monster"),
                spawn_one_in: NonZeroU16::new(one_in).unwrap(),
                spawn_factor: NonZeroU8::new(2).unwrap(),
                spawn_aggression: [1, 10],
            });
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Enter,
                from_above: false,
                kind,
            };
            part.object_types.extend([
                ty(
                    "t:windmill",
                    ObjectKindDef::Windmill {
                        resources: vec![key("t:wood"), key("t:gold")],
                        amount: [3, 6],
                    },
                ),
                ty(
                    "t:water_wheel",
                    ObjectKindDef::WaterWheel {
                        resource: key("t:gold"),
                        first: 500,
                        later: 1000,
                    },
                ),
                ty(
                    "t:garden",
                    ObjectKindDef::Garden {
                        gifts: vec![(key("t:gold"), 500), (key("t:wood"), 5)],
                    },
                ),
                ty("t:event", ObjectKindDef::Event),
            ]);
        },
    )
}

/// The game's stream the steps draw from in these tests.
fn stream() -> Rng {
    Rng::for_stream(7, StreamId::SETUP)
}

/// The monsters' value in sixteenths after a week's growth, capped: the formula as batch E9's
/// E9-9 and E9-10 read it, apart from the code under test.
fn grown(count: u32, fraction: u8) -> (u32, u8) {
    let value = count * 16 + u32::from(fraction);
    let value = (value + value / 10).min(4000 * 16);
    (value / 16, u8::try_from(value % 16).unwrap())
}

// covers: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003
#[test]
fn the_week_fills_every_weekly_object_again_and_keeps_the_marks() {
    let rules = rules(200);
    let objects = vec![
        object(&rules, 2, 7, "t:windmill"),
        object(&rules, 4, 7, "t:water_wheel"),
        object(&rules, 6, 7, "t:water_wheel"),
        object(&rules, 8, 7, "t:garden"),
    ];
    let mut s = started(&rules, &setup(&GRASS_ROWS, objects)).unwrap();
    // The first week as visits leave it: the Windmill, the first Wheel and the Garden emptied
    // with their marks, the second Wheel never visited (its first week's 500).
    let emptied = |place: usize, marks| Weekly {
        amount: 0,
        marks,
        ..*s.weekly(s.map().weeklies()[place]).unwrap()
    };
    let visited = [emptied(0, 0b01), emptied(1, 0b11), emptied(3, 0b10)];
    for (place, weekly) in [0, 1, 3].into_iter().zip(visited) {
        s.set_weekly(place, weekly);
    }
    assert_eq!(s.weekly(2).unwrap().amount, 500);
    // Two weeks, the second drawing on from where the first left the stream.
    let mut draws = stream();
    for _ in 0..2 {
        // The draws, apart from the code: the Windmill's resource then amount, the Garden's gift,
        // in the weekly objects' order; the Wheels draw nothing.
        let mut replay = draws.clone();
        let windmill = (
            [WOOD, GOLD][usize::try_from(replay.below(2)).unwrap()],
            replay.range_inclusive(3, 6),
        );
        let garden = [(GOLD, 500), (WOOD, 5)][usize::try_from(replay.below(2)).unwrap()];
        let mut events = Vec::new();
        s.start_week(&rules.0, &rules.1, &mut draws, &mut events);
        assert!(events.is_empty());
        let full = |(resource, amount), marks| Weekly {
            resource,
            amount,
            marks,
        };
        assert_eq!(s.weekly(0), Some(&full(windmill, 0b01)));
        assert_eq!(s.weekly(1), Some(&full((GOLD, 1000), 0b11)));
        assert_eq!(s.weekly(2), Some(&full((GOLD, 1000), 0)));
        assert_eq!(s.weekly(3), Some(&full(garden, 0b10)));
        assert!(s.check_weekly(&rules.1));
        assert_eq!(draws.clone().next_u64(), replay.next_u64());
    }
}

/// The growth tests' map (E9-9, E9-10): on row 0 monsters of 43 at x 1, 46 at 2, 81 at 3, 15 at
/// 4, 43 not growing at 7, 4 095 at 8 and 46 at 13 (indices 0 to 6); red's hero 1 on (0, 2) sees
/// x 0 to 5 (6 on its fringe), blue's hero 2 on (15, 2) x 10 to 15 (9 on its fringe), so nobody
/// sees the monster at 8.
fn growing(rules: &Rules) -> AdventureState {
    let mut setup = world(
        &["gMMMMggMMggggMgg", "gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![
            hero_at(1, 0, Pos::new(0, 2, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(15, 2, 0), &[PEASANT]),
        ],
    );
    for (object, count) in setup
        .map
        .objects
        .iter_mut()
        .zip([43, 46, 81, 15, 43, 4095, 46])
    {
        object.monster.as_mut().unwrap().count = count;
    }
    setup.map.objects[4].monster.as_mut().unwrap().no_growth = true;
    started(rules, &setup).unwrap()
}

/// The monster at `index` as it stands: its count and its fraction in sixteenths.
fn value(s: &AdventureState, index: usize) -> (u32, u8) {
    (s.monster_count(index).unwrap(), s.fraction(index))
}

// covers: ADV-MON-013
#[test]
fn the_growth_follows_the_sixteenths_to_the_cap() {
    let rules = rules(200);
    let mut s = growing(&rules);
    let changed = |x, word, player| AdventureEvent {
        kind: AdventureEventKind::MonsterChanged {
            object_type: MONSTER,
            anchor: Pos::new(x, 0, 0),
            count_word: word,
        },
        seen_by: Viewers::NONE.with(player),
    };
    let fork = s.clone();
    let before = s.state_hash();
    let mut events = Vec::new();
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut events);
    // E9-9: 43, 81 and 15 become 47 + 4/16, 89 + 1/16 and 16 + 8/16; the one that does not grow
    // stays 43; E9-10: 4 095 falls to the cap, 4 000. 46 becomes 50 + 9/16, its word from the
    // fourth (20 to 49) to the fifth (50 to 99): told to the players that see it, each to its
    // own; 4 000's word stays (1 000 and more), and nobody sees it anyway.
    let week1 = [
        (47, 4),
        (50, 9),
        (89, 1),
        (16, 8),
        (43, 0),
        (4000, 0),
        (50, 9),
    ];
    assert_eq!((0..7).map(|i| value(&s, i)).collect::<Vec<_>>(), week1);
    for (index, &(count, fraction)) in [43, 46, 81, 15].iter().zip(&week1) {
        assert_eq!(grown(*index, 0), (count, fraction));
    }
    assert_eq!(events, [changed(2, 4, RED), changed(13, 4, BLUE)]);
    assert!(s.army_left(4).is_none());
    assert!(s.check_added(&rules.1));
    // The fork before the week keeps its own: nothing it shares moved.
    assert_eq!(fork.state_hash(), before);
    assert_eq!(value(&fork, 0), (43, 0));
    assert_ne!(s.state_hash(), before);
    // A battle leaves 20 of the 89 + 1/16, which keep the fraction (D-49: the original writes
    // the count's bits only); a battle takes the 16 + 8/16 off the map, its fraction with it.
    let rules3 = (&rules.0, &rules.1, &[][..]);
    let survivors = [ArmyStack {
        slot: 0,
        creature: PIKEMAN,
        count: 20,
    }];
    s.end_monster_battle(
        rules3,
        (HERO, Pos::new(3, 0, 0)),
        (&survivors, false),
        &mut Vec::new(),
    );
    s.end_monster_battle(
        rules3,
        (HERO, Pos::new(4, 0, 0)),
        (&[], true),
        &mut Vec::new(),
    );
    assert_eq!(value(&s, 2), (20, 1));
    assert!(s.is_removed(3));
    assert_eq!(s.fraction(3), 0);
    let mut events = Vec::new();
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut events);
    // E9-9's second week: 51 + 15/16 (its word now the fifth, told to red) and 18 + 2/16 (that
    // one off the map now); 50 + 9/16 becomes 55 + 9/16 and 20 + 1/16 22 + 1/16 within their
    // words.
    assert_eq!(grown(47, 4), (51, 15));
    assert_eq!(grown(16, 8), (18, 2));
    assert_eq!(grown(89, 1), (97, 15));
    let week2 = [(51, 15), (55, 9), (22, 1), (4000, 0), (55, 9)];
    let left: Vec<(u32, u8)> = [0, 1, 2, 5, 6].iter().map(|&i| value(&s, i)).collect();
    assert_eq!(left, week2);
    assert_eq!(value(&s, 4), (43, 0));
    assert_eq!(s.fraction(3), 0);
    assert_eq!(events, [changed(1, 4, RED)]);
    assert!(s.check_added(&rules.1));
}

// covers: ADV-WEK-001, ADV-MON-013
#[test]
fn a_week_without_weekly_objects_or_growth_parameters_changes_nothing() {
    // No weekly object: the week's first part draws nothing.
    let rules = rules(200);
    let mut s = growing(&rules);
    let mut draws = stream();
    s.start_week(&rules.0, &rules.1, &mut draws, &mut Vec::new());
    assert_eq!(draws.next_u64(), stream().next_u64());
    // A part without the monsters' parameters (and so without months): nothing grows, nothing
    // is added.
    let rules = movement_rules_with(
        true,
        16,
        |_| {},
        |part: &mut AdventurePack| part.params.as_mut().unwrap().monsters = None,
    );
    let setup = world(
        &["gggg"],
        1,
        vec![hero_at(1, 0, Pos::new(0, 0, 0), &[PEASANT])],
    );
    let mut s = started(&rules, &setup).unwrap();
    let before = s.state_hash();
    let mut events = Vec::new();
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut events);
    s.start_month(&rules.0, &rules.1, WOLF, &mut stream(), &mut events);
    assert_eq!(s.state_hash(), before);
    assert!(events.is_empty());
}

/// The month tests' map, two levels of 8 × 3: on level 0 sea at (1, 0), rock at (2, 0), a
/// blocked tile at (3, 0), a visited one at (4, 0), a monster of five pikemen at (5, 0) (index
/// 2), a pile at (7, 0) taken off the map (index 3), an event at (3, 1) (index 4), red's hero 1 at
/// (0, 1); on level 1 sea at (4, 0) and (5, 0), rock at (1, 1) and (2, 1), blue's hero 2 at
/// (7, 2).
fn month_map(rules: &Rules) -> AdventureState {
    let rows = [
        "g~#BVMgP", "gggggggg", "gggggggg", //
        "gggg~~gg", "g##ggggg", "gggggggg",
    ];
    let mut setup = world(
        &rows,
        2,
        vec![
            hero_at(1, 0, Pos::new(0, 1, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(7, 2, 1), &[PEASANT]),
        ],
    );
    setup.map.objects.push(ObjectSetup {
        anchor: Pos::new(3, 1, 0),
        holds: Holds::Event(Box::new(EventSetup {
            record: RewardRecord {
                resources: vec![(GOLD, 100)],
                ..RewardRecord::default()
            },
            colours: 0b11,
            computer: false,
            remove: true,
        })),
        ..object(rules, 3, 1, "t:event")
    });
    let mut s = started(rules, &setup).unwrap();
    s.remove_object(3);
    s
}

/// The tiles [`month_map`] leaves free, in level, row and column order: land, no object still on
/// the map blocking or visited from it, no event, no hero.
fn free_tiles(rows_taken: &[(u8, u16, &[u16])]) -> Vec<Pos> {
    let mut free = Vec::new();
    for level in 0..2 {
        for y in 0..3 {
            for x in 0..8 {
                let taken = rows_taken
                    .iter()
                    .any(|&(l, row, xs)| l == level && row == y && xs.contains(&x));
                if !taken {
                    free.push(Pos::new(x, y, level));
                }
            }
        }
    }
    free
}

/// [`month_map`]'s free tiles: the pile's freed tile among them, the event's not, the tiles
/// beside the placed monster too (nothing tests a guard zone).
fn month_free() -> Vec<Pos> {
    free_tiles(&[
        (0, 0, &[1, 2, 3, 4, 5]),
        (0, 1, &[0, 3]),
        (1, 0, &[4, 5]),
        (1, 1, &[1, 2]),
        (1, 2, &[7]),
    ])
}

// covers: ADV-MON-014, ADV-FOG-004
#[test]
fn a_creature_month_adds_its_monsters_on_free_land_tiles_as_placed_ones() {
    // One chance in one: every free tile gets a monster.
    let rules = rules(1);
    let mut s = month_map(&rules);
    let placed = s.map().objects()[2].clone();
    let statics = s.map().objects().len();
    let mut events = Vec::new();
    s.start_month(&rules.0, &rules.1, WOLF, &mut stream(), &mut events);
    let added = s.added_objects().to_vec();
    let free = month_free();
    assert_eq!(
        added.iter().map(|o| o.anchor).collect::<Vec<_>>(),
        free,
        "every free land tile, in level, row and column order"
    );
    assert_eq!(events.len(), added.len());
    let views: Vec<Vec<ObjectView>> = [RED, BLUE]
        .iter()
        .map(|&p| s.observe(&rules.0, &rules.1, p).objects)
        .collect();
    for (n, (object, event)) in added.iter().zip(&events).enumerate() {
        let monster = object.monster.clone().unwrap();
        // A wolf, twice 2 to 5, aggression 1 to 10, growing, fleeing as any, no treasure: the
        // placed monster's template and type otherwise.
        assert_eq!(monster.creature, WOLF);
        assert!(monster.count % 2 == 0 && (4..=10).contains(&monster.count));
        assert!((1..=10).contains(&monster.aggression));
        let expected = MapObject {
            anchor: object.anchor,
            monster: Some(Monster {
                creature: WOLF,
                count: monster.count,
                aggression: monster.aggression,
                ..placed.monster.clone().unwrap()
            }),
            ..placed.clone()
        };
        assert_eq!(object, &expected);
        assert_eq!(s.monster_count(statics + n), Some(monster.count));
        // Told to the players that see it, as the observation shows it, with its word.
        let word = rules
            .1
            .params()
            .monsters()
            .unwrap()
            .count_word(monster.count);
        assert_eq!(
            event.kind,
            AdventureEventKind::MonsterAdded {
                object_type: MONSTER,
                anchor: object.anchor,
                creature: WOLF,
                count_word: word,
            }
        );
        for (player, view) in [RED, BLUE].iter().zip(&views) {
            let seen = view.iter().any(|v| {
                v.anchor == object.anchor
                    && v.monster
                        == Some(MonsterView {
                            creature: WOLF,
                            count_word: word,
                        })
            });
            assert_eq!(
                event.seen_by.contains(*player),
                seen,
                "{player:?} {object:?}"
            );
        }
    }
    // Red sees the one beside its hero, blue (on level 1) does not.
    assert_eq!(events[1].seen_by, Viewers::NONE.with(RED));
    assert!(s.check_added(&rules.1));
    // They guard as placed ones: the tile beside the first is guarded by it.
    assert!(s.guard_of(&rules.1, Pos::new(1, 2, 0)).is_some());
    // And grow as the placed ones, after them.
    let mut events = Vec::new();
    s.start_week(&rules.0, &rules.1, &mut stream(), &mut events);
    for (n, object) in added.iter().enumerate() {
        let count = object.monster.as_ref().unwrap().count;
        assert_eq!(value(&s, statics + n), grown(count, 0));
    }
    assert_eq!(value(&s, 2), grown(5, 0));
}

// covers: ADV-MON-014
#[test]
fn the_month_draws_the_chance_then_the_count_then_the_aggression() {
    let rules = rules(3);
    let mut s = month_map(&rules);
    let mut draws = stream();
    // The draws, apart from the code: a free tile's chance, then a monster's count and
    // aggression, tile by tile.
    let mut replay = draws.clone();
    let mut expected = Vec::new();
    for pos in month_free() {
        if replay.range_inclusive(1, 3) == 1 {
            let count = 2 * replay.range_inclusive(2, 5);
            let aggression = 1 + i8::try_from(replay.range_inclusive(0, 9)).unwrap();
            expected.push((pos, count, aggression));
        }
    }
    assert!(!expected.is_empty() && expected.len() < month_free().len());
    s.start_month(&rules.0, &rules.1, WOLF, &mut draws, &mut Vec::new());
    let added: Vec<(Pos, u32, i8)> = s
        .added_objects()
        .iter()
        .map(|o| {
            let monster = o.monster.as_ref().unwrap();
            (o.anchor, monster.count, monster.aggression)
        })
        .collect();
    assert_eq!(added, expected);
    assert_eq!(draws.next_u64(), replay.next_u64());
    // A second month walks the tiles the first left free: its monsters' tiles are taken.
    let first = added.len();
    s.start_month(&rules.0, &rules.1, WOLF, &mut draws, &mut Vec::new());
    let anchors: Vec<Pos> = s.added_objects().iter().map(|o| o.anchor).collect();
    let mut seen = anchors.clone();
    seen.sort_unstable_by_key(|p| (p.level, p.y, p.x));
    seen.dedup();
    assert_eq!(seen.len(), anchors.len());
    assert!(anchors.len() >= first);
    assert!(s.check_added(&rules.1));
}

// covers: ADV-MON-014
#[test]
fn a_part_without_months_adds_nothing() {
    let rules = movement_rules(true);
    let mut s = started(
        &rules,
        &world(
            &["gggg"],
            1,
            vec![hero_at(1, 0, Pos::new(0, 0, 0), &[PEASANT])],
        ),
    )
    .unwrap();
    let before = s.state_hash();
    let mut draws = stream();
    s.start_month(&rules.0, &rules.1, WOLF, &mut draws, &mut Vec::new());
    assert_eq!(s.state_hash(), before);
    assert_eq!(draws.next_u64(), stream().next_u64());
}
