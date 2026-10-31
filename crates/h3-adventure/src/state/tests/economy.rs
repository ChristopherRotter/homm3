//! Tests of the owned and weekly objects' state (E08 T83b1; increment 9's design note §6): the
//! static map's ownable and weekly lists, an abandoned mine's draws and guards, a Shipyard's boat
//! tile, the owners and their reveal at the start, the first week's contents, their invariants and
//! the hash.

use std::collections::BTreeSet;

use h3_kernel::hash::hash_of;

use super::*;
use crate::ruleset::{AdventurePack, ObjectKindDef, ObjectTypeDef, Visit};
use crate::state::economy::fits;
use crate::testing::{PEASANT, hero_at, key, movement_rules_with, started, world};

type Rules = (Ruleset, AdventureRuleset);

const WOOD: ResourceId = ResourceId::from_raw(0);
const GOLD: ResourceId = ResourceId::from_raw(1);

/// The original's twelve offsets from a Shipyard's entrance (`shipyard_boat_offsets` 0x67FAA8).
const OFFSETS: [(i8, i8); 12] = [
    (-2, 0),
    (2, 0),
    (-2, 1),
    (2, 1),
    (-1, 1),
    (1, 1),
    (0, 1),
    (-2, -1),
    (2, -1),
    (-1, -1),
    (1, -1),
    (0, -1),
];

/// [`crate::testing::movement_rules`] (faithful) with one object type of each of increment 9's
/// kinds, entered and not from above, the Shipyard trying `offsets`, the part's flag radius
/// `radius`.
fn rules_with(offsets: &[(i8, i8)], radius: u8) -> Rules {
    movement_rules_with(
        true,
        16,
        |_| {},
        |part: &mut AdventurePack| {
            part.params.as_mut().unwrap().flag_radius = radius;
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Enter,
                from_above: false,
                kind,
            };
            part.object_types.extend([
                ty(
                    "t:sawmill",
                    ObjectKindDef::Mine {
                        resource: key("t:wood"),
                        daily: 2,
                        battle_terrain: None,
                    },
                ),
                ty(
                    "t:gold_mine",
                    ObjectKindDef::Mine {
                        resource: key("t:gold"),
                        daily: 1000,
                        battle_terrain: None,
                    },
                ),
                ty(
                    "t:abandoned",
                    ObjectKindDef::AbandonedMine {
                        guards: key("t:peasant"),
                        count: [100, 200],
                        battle_terrain: None,
                    },
                ),
                ty(
                    "t:lighthouse",
                    ObjectKindDef::Lighthouse { sea_points: 500 },
                ),
                ty(
                    "t:shipyard",
                    ObjectKindDef::Shipyard {
                        price: vec![(key("t:gold"), 1000)],
                        boat_type: 1,
                        offsets: offsets.to_vec(),
                    },
                ),
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
            ]);
        },
    )
}

fn rules() -> Rules {
    rules_with(&OFFSETS, 3)
}

/// An object of type `k` on `(x, y)`, blocking and visited there.
fn object(rules: &Rules, x: u16, y: u16, k: &str) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(x, y, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(rules.1.object_type_by_key(k).unwrap()),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// An abandoned mine on `(x, 0)` that may draw `set`.
fn abandoned(rules: &Rules, x: u16, set: Vec<ResourceId>) -> ObjectSetup {
    ObjectSetup {
        holds: Holds::Resources(set),
        ..object(rules, x, 0, "t:abandoned")
    }
}

/// An object of type `k` on `(x, y)` owned by `owner`.
fn owned(rules: &Rules, x: u16, y: u16, k: &str, owner: u8) -> ObjectSetup {
    ObjectSetup {
        owner: Some(PlayerId::new(owner)),
        ..object(rules, x, y, k)
    }
}

/// A grass map of `rows` with red's hero 1 on the bottom left tile and blue's hero 2 on the bottom
/// right one, and these objects after [`world`]'s.
fn setup(rows: &[&str], objects: Vec<ObjectSetup>) -> AdventureSetup {
    let (width, height) = (rows[0].len(), rows.len());
    let corner = |x: usize| {
        Pos::new(
            u16::try_from(x).unwrap(),
            u16::try_from(height - 1).unwrap(),
            0,
        )
    };
    let mut setup = world(
        rows,
        1,
        vec![
            hero_at(1, 0, corner(0), &[PEASANT]),
            hero_at(2, 1, corner(width - 1), &[PEASANT]),
        ],
    );
    setup.map.objects.extend(objects);
    setup
}

const GRASS_ROWS: [&str; 8] = ["gggggggggggggggg"; 8];

// covers: ADV-MIN-003
#[test]
fn an_abandoned_mine_draws_its_resource_and_its_guards() {
    let rules = rules();
    let objects = vec![
        object(&rules, 1, 0, "t:sawmill"),
        abandoned(&rules, 3, vec![WOOD, GOLD]),
        abandoned(&rules, 5, vec![GOLD]),
        object(&rules, 7, 0, "t:windmill"),
    ];
    let setup = setup(&GRASS_ROWS, objects);
    let state = started(&rules, &setup).unwrap();
    let map = state.map();
    assert_eq!(map.ownables(), &[0, 1, 2]);
    assert_eq!(map.ownable_index(2), Some(2));
    assert_eq!(map.ownable_index(3), None);
    let (resource, guards) = map.objects()[1].resolved.mine().unwrap();
    assert!([WOOD, GOLD].contains(&resource), "{resource:?}");
    assert!((100..=200).contains(&guards), "{guards}");
    assert_eq!(map.objects()[2].resolved.mine().map(|(r, _)| r), Some(GOLD));
    // A mine of the seven draws nothing and has no guards; nor has an object that is no mine.
    assert_eq!(map.objects()[0].resolved, Resolved::Nothing);
    assert_eq!(map.objects()[0].resolved.mine(), None);
    assert_eq!(
        state.mine_guards(&rules.1, 1),
        vec![ArmyStack {
            slot: 0,
            creature: PEASANT,
            count: guards,
        }]
    );
    for index in [0, 3, 99] {
        assert!(state.mine_guards(&rules.1, index).is_empty(), "{index}");
    }
    // Every resource of the set comes, with equal chances; the count within its bounds.
    let mut resources = BTreeSet::new();
    let mut counts = BTreeSet::new();
    for seed in 0..64 {
        let drawn = AdventureState::new(&rules.0, &rules.1, &setup, &mut Rng::from_seed(seed)).unwrap();
        let (resource, guards) = drawn.map().objects()[1].resolved.mine().unwrap();
        assert!((100..=200).contains(&guards));
        resources.insert(resource);
        counts.insert(guards);
    }
    assert_eq!(resources.len(), 2);
    assert!(counts.len() > 30, "{}", counts.len());
    // An army written on the mine stands for its guards; written empty, no guards come back.
    let mut written = state.clone();
    let army = vec![ArmyStack {
        slot: 2,
        creature: PEASANT,
        count: 7,
    }];
    written.set_army_left(1, army.clone());
    assert_eq!(written.mine_guards(&rules.1, 1), army);
    assert!(written.check_owners(&rules.1));
    assert_ne!(written.state_hash(), state.state_hash());
    written.set_army_left(1, vec![]);
    assert!(written.mine_guards(&rules.1, 1).is_empty());
    assert!(written.check_owners(&rules.1));
    // The guards' invariants: slots in order within the army's, each one to a stack's most.
    let stack = |slot, count| ArmyStack {
        slot,
        creature: PEASANT,
        count,
    };
    let slots = rules.1.params().army_slots();
    let most = rules.1.params().max_stack_count();
    for army in [
        vec![stack(0, 0)],
        vec![stack(slots, 1)],
        vec![stack(1, 1), stack(0, 1)],
        vec![stack(0, most + 1)],
    ] {
        let mut wrong = state.clone();
        wrong.set_army_left(0, army.clone());
        assert!(!wrong.check_owners(&rules.1), "{army:?}");
    }
}

// covers: ADV-SHY-001
#[test]
fn a_shipyard_fixes_its_boat_tile_when_the_game_is_built() {
    let rules = rules();
    // The entrance at (3, 2): the offsets' first water tile is (5, 1), (2, 0)'s row above.
    let rows = ["gg~~~~~~", "gg~~~~~~", "gggggggg", "gggggggg"];
    let shipyard = || object(&rules, 3, 2, "t:shipyard");
    let tile_of = |objects: Vec<ObjectSetup>, rows: &[&str]| {
        let state = started(&rules, &setup(rows, objects)).unwrap();
        let shipyard = state.map().ownables()[0];
        state.map().objects()[shipyard].resolved.boat_tile()
    };
    assert_eq!(tile_of(vec![shipyard()], &rows), Some(Pos::new(5, 1, 0)));
    // An empty boat lies on no object's tile: it does not move the boat tile (the original takes
    // a boat's entrance too).
    let boat = ["gg~~~~~~", "gg~~~b~~", "gggggggg", "gggggggg"];
    assert_eq!(tile_of(vec![shipyard()], &boat), Some(Pos::new(5, 1, 0)));
    // A tile an object blocks, then one an object is visited from, is passed over.
    let blocker = ObjectSetup {
        object_type: None,
        visitable: TileMask::default(),
        ..object(&rules, 5, 1, "t:shipyard")
    };
    let visited = ObjectSetup {
        object_type: None,
        blocked: TileMask::default(),
        ..object(&rules, 2, 1, "t:shipyard")
    };
    assert_eq!(
        tile_of(vec![shipyard(), blocker.clone()], &rows),
        Some(Pos::new(2, 1, 0))
    );
    assert_eq!(
        tile_of(vec![shipyard(), blocker, visited], &rows),
        Some(Pos::new(4, 1, 0))
    );
    // No free water among the offsets (Hoard's Shipyard): none, and it never builds; the offsets
    // past the map's edges are skipped.
    let land = ["gggggggg", "gggggggg", "gggggggg", "gggggggg"];
    assert_eq!(tile_of(vec![shipyard()], &land), None);
    let corner = object(&rules, 0, 0, "t:shipyard");
    assert_eq!(tile_of(vec![corner], &land), None);
    // The first free tile in the kind's order: reversed, the last of the original's twelve.
    let reversed: Vec<(i8, i8)> = OFFSETS.iter().rev().copied().collect();
    let other = rules_with(&reversed, 3);
    let state = started(
        &other,
        &setup(&rows, vec![object(&other, 3, 2, "t:shipyard")]),
    )
    .unwrap();
    assert_eq!(
        state.map().objects()[0].resolved,
        Resolved::BoatTile(Pos::new(3, 1, 0))
    );
    // The static map's hash sees the tile.
    let first = started(&rules, &setup(&rows, vec![shipyard()])).unwrap();
    assert_ne!(first.map().content_hash(), state.map().content_hash());
}

// covers: ADV-OWN-002
#[test]
fn the_owners_come_from_the_setup_and_reveal_around_their_objects() {
    let rules = rules();
    let objects = vec![
        owned(&rules, 8, 1, "t:sawmill", 1),
        object(&rules, 3, 1, "t:lighthouse"),
        owned(&rules, 12, 1, "t:gold_mine", 0),
        object(&rules, 8, 4, "t:windmill"),
    ];
    let state = started(&rules, &setup(&GRASS_ROWS, objects.clone())).unwrap();
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    assert_eq!(state.owner(0), Some(blue));
    assert_eq!(state.owner(1), None);
    assert_eq!(state.owner(2), Some(red));
    assert_eq!(state.owner(3), None);
    assert_eq!(state.owner(99), None);
    assert!(state.check_owners(&rules.1));
    // Blue sees the disk of radius 3 around its sawmill's entrance, (8, 1); red does not, nor
    // does anybody the neutral Lighthouse's surroundings.
    let explored =
        |state: &AdventureState, player, x, y| state.is_explored(player, Pos::new(x, y, 0));
    for (x, y) in [(8, 1), (5, 1), (11, 1), (7, 4), (9, 4), (8, 0)] {
        assert!(explored(&state, blue, x, y), "({x}, {y})");
        assert!(!explored(&state, red, x, y) || x == 11, "({x}, {y})");
    }
    for (x, y) in [(4, 1), (6, 4), (10, 4), (8, 5)] {
        assert!(!explored(&state, blue, x, y), "({x}, {y})");
    }
    assert!(explored(&state, red, 12, 1) && explored(&state, red, 15, 1));
    assert!(!explored(&state, red, 3, 1) && !explored(&state, blue, 3, 1));
    // The part's radius.
    let near = rules_with(&OFFSETS, 1);
    let state = started(
        &near,
        &setup(&GRASS_ROWS, vec![owned(&near, 8, 1, "t:sawmill", 1)]),
    )
    .unwrap();
    assert!(explored(&state, blue, 9, 1) && !explored(&state, blue, 10, 1));
}

// covers: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003
#[test]
fn the_weekly_objects_start_with_their_first_week() {
    let rules = rules();
    let objects = vec![
        object(&rules, 1, 0, "t:windmill"),
        object(&rules, 3, 0, "t:sawmill"),
        object(&rules, 5, 0, "t:water_wheel"),
        object(&rules, 7, 0, "t:garden"),
    ];
    let setup = setup(&GRASS_ROWS, objects);
    let state = started(&rules, &setup).unwrap();
    assert_eq!(state.map().weeklies(), &[0, 2, 3]);
    assert_eq!(state.map().weekly_index(3), Some(2));
    assert_eq!(state.map().weekly_index(1), None);
    let windmill = *state.weekly(0).unwrap();
    assert!([WOOD, GOLD].contains(&windmill.resource) && (3..=6).contains(&windmill.amount));
    assert_eq!(
        state.weekly(2),
        Some(&Weekly {
            resource: GOLD,
            amount: 500,
            marks: 0
        })
    );
    let garden = *state.weekly(3).unwrap();
    assert!([(GOLD, 500), (WOOD, 5)].contains(&(garden.resource, garden.amount)));
    assert_eq!((windmill.marks, garden.marks), (0, 0));
    assert_eq!(state.weekly(1), None);
    assert!(state.check_weekly(&rules.1));
    // Every resource, every amount and every gift come.
    let (mut mills, mut gardens) = (BTreeSet::new(), BTreeSet::new());
    for seed in 0..64 {
        let drawn = AdventureState::new(&rules.0, &rules.1, &setup, &mut Rng::from_seed(seed)).unwrap();
        let mill = drawn.weekly(0).unwrap();
        mills.insert((mill.resource, mill.amount));
        let garden = drawn.weekly(3).unwrap();
        gardens.insert((garden.resource, garden.amount));
    }
    assert_eq!(mills.len(), 8, "{mills:?}");
    assert_eq!(gardens.len(), 2);
    // A map without weekly objects has none.
    let plain = started(&rules, &self::setup(&GRASS_ROWS, vec![])).unwrap();
    assert_eq!(plain.weekly(0), None);
    assert!(plain.check_weekly(&rules.1));
    // The invariants: the marks of the game's players only, the contents their kind's.
    let mut marked = state.clone();
    marked.set_weekly(
        1,
        Weekly {
            marks: 0b11,
            ..*state.weekly(2).unwrap()
        },
    );
    assert!(marked.check_weekly(&rules.1));
    assert_ne!(marked.state_hash(), state.state_hash());
    marked.set_weekly(
        1,
        Weekly {
            marks: 0b100,
            ..*state.weekly(2).unwrap()
        },
    );
    assert!(!marked.check_weekly(&rules.1));
    let mut emptied = state.clone();
    emptied.set_weekly(
        0,
        Weekly {
            amount: 0,
            ..windmill
        },
    );
    assert!(emptied.check_weekly(&rules.1));
    emptied.set_weekly(
        0,
        Weekly {
            amount: 7,
            ..windmill
        },
    );
    assert!(!emptied.check_weekly(&rules.1));
}

// covers: ADV-WEK-001, ADV-WEK-002, ADV-WEK-003
#[test]
fn a_weekly_object_s_contents_fit_its_kind() {
    let rules = rules();
    let kind = |k: &str| {
        let id = rules.1.object_type_by_key(k).unwrap();
        Some(&rules.1.object_type(id).unwrap().kind)
    };
    let weekly = |resource, amount| Weekly {
        resource,
        amount,
        marks: 0,
    };
    for (k, resource, amount, fit) in [
        ("t:windmill", WOOD, 3, true),
        ("t:windmill", GOLD, 6, true),
        ("t:windmill", GOLD, 0, true),
        ("t:windmill", GOLD, 2, false),
        ("t:windmill", WOOD, 7, false),
        ("t:windmill", ResourceId::from_raw(2), 4, false),
        ("t:water_wheel", GOLD, 500, true),
        ("t:water_wheel", GOLD, 1000, true),
        ("t:water_wheel", GOLD, 0, true),
        ("t:water_wheel", GOLD, 700, false),
        ("t:water_wheel", WOOD, 500, false),
        ("t:garden", GOLD, 500, true),
        ("t:garden", WOOD, 5, true),
        ("t:garden", WOOD, 0, true),
        ("t:garden", GOLD, 5, false),
        ("t:garden", WOOD, 500, false),
        ("t:sawmill", WOOD, 2, false),
    ] {
        assert_eq!(
            fits(kind(k), &weekly(resource, amount)),
            fit,
            "{k} {resource:?} {amount}"
        );
    }
    assert!(!fits(None, &weekly(GOLD, 500)));
}

#[test]
fn the_hash_sees_the_owners_and_the_weekly_contents() {
    let rules = rules();
    let hash = |owner: Option<u8>| {
        let mine = ObjectSetup {
            owner: owner.map(PlayerId::new),
            ..object(&rules, 8, 1, "t:sawmill")
        };
        hash_of(&started(&rules, &setup(&GRASS_ROWS, vec![mine])).unwrap())
    };
    let hashes: BTreeSet<u64> = [None, Some(0), Some(1)].into_iter().map(hash).collect();
    assert_eq!(hashes.len(), 3);
    // A fork shares the blocks and hashes the same.
    let state = started(
        &rules,
        &setup(&GRASS_ROWS, vec![object(&rules, 1, 0, "t:garden")]),
    )
    .unwrap();
    let fork = state.clone();
    assert_eq!(fork.state_hash(), state.state_hash());
    assert_eq!(fork, state);
}
