//! Tests of the town as a map object (E09 T8a1, the town epic's design note §8): the static map's
//! towns and their `TownId`s, the garrisons on their objects, the towns among the ownable objects
//! with their owners and the start's reveal by the game's radii, the formation the map holds and
//! the hash, and the entrance not entered until T8a2 (`TWN-STUB-4`).

use h3_kernel::hash::hash_of;

use super::*;
use crate::ruleset::{AdventurePack, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{PEASANT, PIKEMAN, draws, hero_at, key, movement_rules_with, started, world};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);

/// [`movement_rules_with`] (faithful, maps up to 32 tiles) with a town `t:town` and a mine
/// `t:sawmill`, both entered and not from above, and the part's flag radius 1.
fn town_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |_| {},
        |part: &mut AdventurePack| {
            part.params.as_mut().unwrap().flag_radius = 1;
            let ty = |k: &str, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit: Visit::Enter,
                from_above: false,
                kind,
            };
            part.object_types.extend([
                ty("t:town", ObjectKindDef::Town),
                ty(
                    "t:sawmill",
                    ObjectKindDef::Mine {
                        resource: key("t:wood"),
                        daily: 2,
                        battle_terrain: None,
                    },
                ),
            ]);
        },
    )
}

/// An object of type `k` on `(x, y)` blocking that tile and the one left of it, entered on its
/// anchor (a town's gate), `owner`'s at the start; a town holds a loose formation.
fn object(rules: &Rules, k: &str, (x, y): (u16, u16), owner: Option<u8>) -> ObjectSetup {
    ObjectSetup {
        scenery: Scenery::Ownable,
        anchor: Pos::new(x, y, 0),
        blocked: TileMask::from_bits(0b11),
        visitable: TileMask::from_bits(1),
        object_type: rules.1.object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: if k == "t:town" {
            Holds::Town {
                formation: Formation::Loose,
            }
        } else {
            Holds::Nothing
        },
        owner: owner.map(PlayerId::new),
    }
}

/// A grass map of 32 × 16 with red's hero 1 on (0, 15) and blue's hero 2 on (31, 15), these
/// objects and garrisons.
fn town_setup(objects: Vec<ObjectSetup>, garrisons: Vec<Vec<ArmyStack>>) -> AdventureSetup {
    let mut setup = world(
        &["gggggggggggggggggggggggggggggggg"; 16],
        1,
        vec![
            hero_at(1, 0, Pos::new(0, 15, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(31, 15, 0), &[PEASANT]),
        ],
    );
    setup.map.objects = objects;
    setup.garrisons = garrisons;
    setup
}

/// A sawmill on (8, 2), red's; red's town on (16, 3); a neutral town on (24, 3).
fn objects(rules: &Rules) -> Vec<ObjectSetup> {
    vec![
        object(rules, "t:sawmill", (8, 2), Some(0)),
        object(rules, "t:town", (16, 3), Some(0)),
        object(rules, "t:town", (24, 3), None),
    ]
}

/// Red's town's garrison, out of slot order.
fn garrison() -> Vec<ArmyStack> {
    vec![stack(2, PIKEMAN, 5), stack(0, PEASANT, 3)]
}

fn build(rules: &Rules, setup: &AdventureSetup) -> Result<AdventureState, SetupError> {
    AdventureState::new(&rules.0, &rules.1, setup, &mut draws())
}

// covers: ADV-OBJ-002, ADV-OWN-002
#[test]
fn the_static_map_lists_the_towns_with_their_owners_and_garrisons() {
    let rules = town_rules();
    let setup = town_setup(objects(&rules), vec![garrison(), vec![]]);
    let state = started(&rules, &setup).unwrap();
    let map = state.map();
    // The towns in object order, a `TownId` their place; no object stores it.
    assert_eq!(map.towns(), [1, 2]);
    assert_eq!(map.town_id(1), Some(TownId::new(0)));
    assert_eq!(map.town_id(2), Some(TownId::new(1)));
    assert_eq!(map.town_id(0), None);
    assert_eq!(map.town_id(9), None);
    assert_eq!(map.town_object(TownId::new(1)), Some(2));
    assert_eq!(map.town_object(TownId::new(2)), None);
    // Among the ownable objects, owned as the setup says.
    assert_eq!(map.ownables(), [0, 1, 2]);
    assert_eq!(
        (state.owner(0), state.owner(1), state.owner(2)),
        (Some(RED), Some(RED), None)
    );
    assert!(state.check_owners(&rules.1));
    // Each garrison on its town's object by slot, an empty one written empty; no garrison for an
    // object that is no town.
    let by_slot = [stack(0, PEASANT, 3), stack(2, PIKEMAN, 5)];
    assert_eq!(state.town_garrison(1), Some(&by_slot[..]));
    assert_eq!(state.army_left(1), Some(&by_slot[..]));
    assert_eq!(state.town_garrison(2), Some(&[][..]));
    assert_eq!(state.army_left(2), Some(&[][..]));
    assert_eq!(state.town_garrison(0), None);
    // A town holds its formation; the static map's hash reads it (adventure state hash v21).
    assert_eq!(
        map.objects()[1].holds,
        Holds::Town {
            formation: Formation::Loose
        }
    );
    let mut tight = setup.clone();
    tight.map.objects[1].holds = Holds::Town {
        formation: Formation::Tight,
    };
    let tight = started(&rules, &tight).unwrap();
    assert_ne!(tight.map().content_hash(), map.content_hash());
    assert_ne!(tight.state_hash(), state.state_hash());
    // The garrison is in the state's hash.
    let mut other = setup.clone();
    other.garrisons[0].pop();
    assert_ne!(
        started(&rules, &other).unwrap().state_hash(),
        state.state_hash()
    );
    assert_ne!(hash_of(&state), 0);
}

#[test]
fn the_towns_garrisons_and_holds_are_checked() {
    let rules = town_rules();
    let refused = |edit: &dyn Fn(&mut AdventureSetup)| {
        let mut setup = town_setup(objects(&rules), vec![garrison(), vec![]]);
        edit(&mut setup);
        build(&rules, &setup).unwrap_err()
    };
    // One garrison per town.
    let error = refused(&|s| s.garrisons.truncate(1));
    assert_eq!(error, SetupError::Garrisons(1));
    assert_eq!(error.to_string(), "1 garrisons do not give one per town");
    assert_eq!(
        refused(&|s| s.garrisons.push(vec![])),
        SetupError::Garrisons(3)
    );
    // A garrison with stacks is an army's; the first bad one by `TownId`.
    let error = refused(&|s| s.garrisons[1] = vec![stack(0, PEASANT, 1), stack(0, PIKEMAN, 1)]);
    assert_eq!(
        error,
        SetupError::Garrison {
            town: TownId::new(1),
            error: ArmyError::DuplicateSlot(0),
        }
    );
    assert_eq!(error.to_string(), "town 1's garrison: slot 0 is used twice");
    assert_eq!(
        refused(&|s| {
            s.garrisons[0][0].count = 0;
            s.garrisons[1] = vec![stack(9, PEASANT, 1)];
        }),
        SetupError::Garrison {
            town: TownId::new(0),
            error: ArmyError::EmptyStack(2),
        }
    );
    // A town holds a formation, and nothing else does; an owner a player of the game.
    assert_eq!(
        refused(&|s| s.map.objects[2].holds = Holds::Nothing),
        SetupError::Holds(2)
    );
    assert_eq!(
        refused(&|s| {
            s.map.objects[0].holds = Holds::Town {
                formation: Formation::Tight,
            };
        }),
        SetupError::Holds(0)
    );
    assert_eq!(
        refused(&|s| s.map.objects[2].owner = Some(PlayerId::new(2))),
        SetupError::ObjectOwner(2)
    );
}

#[test]
fn more_towns_than_a_town_id_names_are_refused() {
    let rules = town_rules();
    let town = object(&rules, "t:town", (0, 0), None);
    let many = usize::from(u16::MAX) + 2;
    // Distinct anchors past the map, which their footprints leave alone (ADV-OBJ-001).
    let objects = (0..many)
        .map(|n| {
            let (x, y) = (n % 1024, n / 1024);
            ObjectSetup {
                anchor: Pos::new(
                    u16::try_from(x + 64).unwrap(),
                    u16::try_from(y + 64).unwrap(),
                    0,
                ),
                ..town.clone()
            }
        })
        .collect();
    let error = build(&rules, &town_setup(objects, vec![])).unwrap_err();
    assert_eq!(error, SetupError::Towns(many));
    assert_eq!(error.to_string(), "65537 towns are too many");
}

// covers: ADV-OWN-002
#[test]
fn the_start_reveals_around_a_town_with_the_games_radius() {
    // Red's sawmill with the flag radius 1, red's town with the game's 4; blue's town past the
    // game's slice with the flag radius.
    let rules = town_rules();
    let mut objects = objects(&rules);
    objects[2].owner = Some(BLUE);
    let setup = town_setup(objects, vec![vec![], vec![]]);
    let mut state = build(&rules, &setup).unwrap();
    state.start_game(&rules.0, &rules.1, &[], &[4]);
    let red = state.explored(RED).unwrap();
    let blue = state.explored(BLUE).unwrap();
    let at = |x, y| Pos::new(x, y, 0);
    assert!(red.get(at(16, 7)) && !red.get(at(16, 8)));
    assert!(red.get(at(12, 3)) && !red.get(at(11, 3)));
    assert!(red.get(at(8, 3)) && !red.get(at(8, 4)));
    assert!(blue.get(at(24, 4)) && !blue.get(at(24, 5)));
    assert!(!red.get(at(24, 4)) && !blue.get(at(16, 4)));
    // Without the game's radii every town keeps the flag radius.
    let state = started(&rules, &setup).unwrap();
    let red = state.explored(RED).unwrap();
    assert!(red.get(at(16, 4)) && !red.get(at(16, 5)));
}

// covers: ADV-OBJ-003
#[test]
fn a_towns_entrance_is_a_blockers_until_its_step_is_built() {
    // `TWN-STUB-4`: the reach and the step meet a town's entrance as a blocker's, whose entrance
    // no hero ends on or passes; a hero standing on it leaves it as from any tile, upwards too
    // (a town is not visited from above, but its visit is not built).
    let rules = town_rules();
    let entrance = Pos::new(2, 1, 0);
    let with = |object_type: Option<ObjectTypeId>, hero: Pos| {
        let mut town = object(&rules, "t:town", (2, 1), Some(0));
        if object_type.is_none() {
            town.object_type = None;
            town.holds = Holds::Nothing;
            town.owner = None;
        }
        let mut setup = town_setup(vec![town], vec![]);
        if object_type.is_some() {
            setup.garrisons = vec![vec![]];
        }
        setup.heroes[0].pos = hero;
        started(&rules, &setup).unwrap()
    };
    let town = rules.1.object_type_by_key("t:town");
    let tiles = |state: &AdventureState| {
        let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
        reach.tiles().collect::<Vec<_>>()
    };
    for hero in [Pos::new(2, 2, 0), entrance] {
        let typed = with(town, hero);
        let blocker = with(None, hero);
        assert_eq!(tiles(&typed), tiles(&blocker), "{hero:?}");
        assert!(!tiles(&typed).iter().any(|&(pos, _)| pos == entrance));
    }
    // From the entrance a hero steps upwards.
    let typed = with(town, entrance);
    assert!(
        tiles(&typed)
            .iter()
            .any(|&(pos, _)| pos == Pos::new(2, 0, 0))
    );
    // A step onto the entrance is refused as onto a blocker's.
    let walk = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![entrance],
    };
    for object_type in [town, None] {
        let mut state = with(object_type, Pos::new(2, 2, 0));
        let mut events = Vec::new();
        assert_eq!(
            state.apply(&rules.0, &rules.1, &[], RED, &walk, &mut events),
            Err(AdventureCommandError::Visit { step: 0 })
        );
    }
}
