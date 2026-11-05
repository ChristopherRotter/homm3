//! Tests of the town as a map object (E09 T8a1, the town epic's design note §8): the static map's
//! towns and their `TownId`s, the garrisons on their objects, the towns among the ownable objects
//! with their owners and the start's reveal by the game's radii, the formation the map holds and
//! the hash; since T8a2 the step onto a town (TWN-VIS-001), its visitor (TWN-VIS-002) and the
//! capture with its reveal (TWN-CAP-001; the town epic's design note §9, §11).

use h3_content::SkillId;
use h3_kernel::hash::hash_of;

use super::*;
use crate::ruleset::{AdventurePack, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{
    FLYING, PEASANT, PIKEMAN, WATER_WALKING, draws, hero_at, hero_details, hero_skills, key,
    movement_rules_with, started, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);

/// [`movement_rules_with`] (faithful, maps up to 32 tiles) with a town `t:town` and a mine
/// `t:sawmill`, both entered and not from above, and the part's flag radius 1; the skills of
/// [`hero_skills`] (flying and water walking among them).
fn town_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |core| core.skills = hero_skills(),
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
    state.start_game(&rules.0, &rules.1, &[], &[TownReveal::Radius(4)]);
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

/// The entrance of [`at_town`]'s town, whose object also blocks the tile left of it.
const ENTRANCE: Pos = Pos::new(4, 4, 0);

/// [`town_setup`] with one town on (4, 4) of `owner` holding `garrison`, red's hero 1 below its
/// entrance on (4, 5), blue's hero 2 on `blue` and, if `red` names a tile, red's hero 3 there.
fn at_town(
    rules: &Rules,
    owner: Option<u8>,
    garrison: Vec<ArmyStack>,
    (blue, red): (Pos, Option<Pos>),
) -> AdventureState {
    let town = object(rules, "t:town", (4, 4), owner);
    let mut setup = town_setup(vec![town], vec![garrison]);
    setup.heroes[0].pos = Pos::new(4, 5, 0);
    setup.heroes[1].pos = blue;
    setup
        .heroes
        .extend(red.map(|pos| hero_at(3, 0, pos, &[PEASANT])));
    started(rules, &setup).unwrap()
}

/// Blue's hero 2 where it starts in [`town_setup`], far from the town.
const FAR: Pos = Pos::new(31, 15, 0);

/// The store of heroes' details: hero 1 with `skills`, heroes 0, 2 and 3 without.
fn store(skills: &[(SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![
        Arc::new(hero_details(&[])),
        Arc::new(hero_details(skills)),
        Arc::new(hero_details(&[])),
        Arc::new(hero_details(&[])),
    ]
}

/// Red's hero 1 moved along `path` with the details of `heroes`, its events.
fn walk(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let walk = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let mut events = Vec::new();
    let outcome = state.apply(&rules.0, &rules.1, heroes, RED, &walk, &mut events)?;
    Ok((outcome, events))
}

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

// covers: TWN-VIS-001, TWN-VIS-002
#[test]
fn a_town_of_the_heros_team_is_entered_and_ends_the_walk() {
    let rules = town_rules();
    // Red's town, defended or not: entered; the walk ends on its entrance.
    for garrison in [vec![], garrison()] {
        let mut state = at_town(&rules, Some(0), garrison, (FAR, None));
        let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
        assert_eq!(reach.path(ENTRANCE), Some(vec![ENTRANCE]));
        // The tile above is reached around it, not through it.
        assert!(
            reach
                .path(p(4, 3))
                .is_some_and(|path| !path.contains(&ENTRANCE))
        );
        assert_eq!(
            walk(&rules, &mut state.clone(), &[], &[ENTRANCE, p(5, 3)]).unwrap_err(),
            AdventureCommandError::PastVisit { step: 1 }
        );
        assert_eq!(state.town_visitor(TownId::new(0)), None);
        let (outcome, events) = walk(&rules, &mut state, &[], &[ENTRANCE]).unwrap();
        // No service, no decision, no event beyond the step's.
        assert_eq!(outcome, AdventureOutcome::default());
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].kind,
            AdventureEventKind::HeroStepped { hero: HERO, .. }
        ));
        assert_eq!(state.hero(HERO).unwrap().pos, ENTRANCE);
        assert_eq!(state.town_visitor(TownId::new(0)), Some(HERO));
        assert_eq!(state.map().town_at(ENTRANCE), Some(TownId::new(0)));
        assert_eq!(state.map().town_at(p(3, 4)), None);
        // On the entrance the hero leaves it downwards only (ADV-OBJ-004: a town is not visited
        // from above).
        let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
        for (pos, _) in reach.tiles() {
            assert!(reach.path(pos).unwrap()[0].y >= ENTRANCE.y, "{pos:?}");
        }
        assert_eq!(
            walk(&rules, &mut state.clone(), &[], &[p(4, 3)]).unwrap_err(),
            AdventureCommandError::FromAbove { step: 0 }
        );
    }
    // No visitor for a town the map does not have.
    let state = at_town(&rules, Some(0), vec![], (ENTRANCE, None));
    assert_eq!(state.town_visitor(TownId::new(0)), Some(HeroId::new(2)));
    assert_eq!(state.town_visitor(TownId::new(1)), None);
}

// covers: TWN-VIS-001
#[test]
fn an_undefended_town_of_another_player_or_none_is_entered_for_its_capture() {
    let rules = town_rules();
    for owner in [Some(1), None] {
        let mut state = at_town(&rules, owner, vec![], (FAR, None));
        let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
        assert_eq!(reach.path(ENTRANCE), Some(vec![ENTRANCE]));
        let (outcome, events) = walk(&rules, &mut state, &[], &[ENTRANCE]).unwrap();
        // The capture is the game's: the step changes no owner.
        assert_eq!(
            outcome,
            AdventureOutcome {
                capture: Some(TownId::new(0)),
                ..AdventureOutcome::default()
            }
        );
        assert_eq!(events.len(), 1);
        assert_eq!(state.hero(HERO).unwrap().pos, ENTRANCE);
        assert_eq!(state.owner(0), owner.map(PlayerId::new));
        assert_eq!(state.town_visitor(TownId::new(0)), Some(HERO));
    }
}

// covers: TWN-VIS-001
#[test]
fn a_defended_town_of_another_player_or_none_is_refused_until_the_siege() {
    // `TWN-STUB-7`: no step ends on or passes its entrance; a flight passes over it.
    let rules = town_rules();
    let (flyer, walker) = (store(&[(FLYING, 3)]), store(&[(WATER_WALKING, 3)]));
    for owner in [Some(1), None] {
        let mut state = at_town(&rules, owner, garrison(), (FAR, None));
        let before = state.clone();
        for heroes in [&[][..], &walker] {
            let reach = state.reach(&rules.0, &rules.1, heroes, HERO).unwrap();
            assert_eq!(reach.points_left(ENTRANCE), None);
            assert_eq!(
                walk(&rules, &mut state, heroes, &[ENTRANCE]).unwrap_err(),
                AdventureCommandError::Visit { step: 0 }
            );
            assert_eq!(state, before);
        }
        let reach = state.reach(&rules.0, &rules.1, &flyer, HERO).unwrap();
        assert_eq!(reach.points_left(ENTRANCE), None);
        assert!(reach.points_left(p(4, 3)).is_some());
        walk(&rules, &mut state, &flyer, &[ENTRANCE, p(5, 4)]).unwrap();
        assert_eq!(state.hero(HERO).unwrap().pos, p(5, 4));
    }
}

// covers: TWN-VIS-002
#[test]
fn another_players_visitor_is_refused_until_the_siege_and_an_allied_one_met() {
    // `TWN-STUB-7`: blue's hero on the entrance of any town defends it against red, whatever the
    // town's owner; a flight passes over it, never stops there.
    let rules = town_rules();
    let (flyer, walker) = (store(&[(FLYING, 3)]), store(&[(WATER_WALKING, 3)]));
    for owner in [Some(0), Some(1), None] {
        let mut state = at_town(&rules, owner, vec![], (ENTRANCE, None));
        assert_eq!(state.town_visitor(TownId::new(0)), Some(HeroId::new(2)));
        let before = state.clone();
        for heroes in [&[][..], &walker] {
            let reach = state.reach(&rules.0, &rules.1, heroes, HERO).unwrap();
            assert_eq!(reach.points_left(ENTRANCE), None);
            assert_eq!(
                walk(&rules, &mut state, heroes, &[ENTRANCE]).unwrap_err(),
                AdventureCommandError::Visit { step: 0 }
            );
            assert_eq!(state, before);
        }
        let reach = state.reach(&rules.0, &rules.1, &flyer, HERO).unwrap();
        assert_eq!(reach.points_left(ENTRANCE), None);
        assert!(reach.points_left(p(4, 3)).is_some());
        assert_eq!(
            walk(&rules, &mut state, &flyer, &[ENTRANCE]).unwrap_err(),
            AdventureCommandError::NoStop { step: 0 }
        );
        walk(&rules, &mut state, &flyer, &[ENTRANCE, p(5, 4)]).unwrap();
        assert_eq!(state.hero(HERO).unwrap().pos, p(5, 4));
    }
    // Blue's hero off a town's entrance is met in a battle as before (GAM-BAT-004).
    let mut state = at_town(&rules, Some(1), vec![], (p(5, 5), None));
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert!(reach.points_left(p(5, 5)).is_some());
    let (outcome, _) = walk(&rules, &mut state, &[], &[p(5, 5)]).unwrap();
    assert!(outcome.battle.is_some());
    // Red's own hero on the entrance, of red's town or blue's: a meeting (GAM-MET-001).
    for owner in [Some(0), Some(1)] {
        let mut state = at_town(&rules, owner, garrison(), (FAR, Some(ENTRANCE)));
        assert_eq!(state.town_visitor(TownId::new(0)), Some(HeroId::new(3)));
        let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
        assert_eq!(reach.path(ENTRANCE), Some(vec![ENTRANCE]));
        let (outcome, _) = walk(&rules, &mut state, &[], &[ENTRANCE]).unwrap();
        assert_eq!(
            outcome.meeting,
            Some(Meeting {
                visitor: HERO,
                visited: HeroId::new(3),
            })
        );
    }
}

// covers: TWN-VIS-002
#[test]
fn the_visitors_town_is_the_tiles_object_the_step_meets() {
    // A town and a sawmill both entered on (4, 4), blue's hero there: the step meets the tile's
    // object, the last in map order (ADV-OBJ-003). The town: its siege, refused by the step and
    // left out of the reachable tiles alike (`TWN-STUB-7`); the sawmill: a battle with the hero,
    // which both offer alike.
    let rules = town_rules();
    let town = object(&rules, "t:town", (4, 4), None);
    let mine = object(&rules, "t:sawmill", (4, 4), None);
    for (objects, siege) in [
        (vec![town.clone(), mine.clone()], false),
        (vec![mine, town], true),
    ] {
        let mut setup = town_setup(objects, vec![vec![]]);
        setup.heroes[0].pos = p(4, 5);
        setup.heroes[1].pos = ENTRANCE;
        let mut state = started(&rules, &setup).unwrap();
        let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
        assert_eq!(reach.points_left(ENTRANCE).is_some(), !siege);
        let result = walk(&rules, &mut state, &[], &[ENTRANCE]);
        if siege {
            assert_eq!(
                result.unwrap_err(),
                AdventureCommandError::Visit { step: 0 }
            );
        } else {
            assert!(result.unwrap().0.battle.is_some());
        }
    }
}

// covers: TWN-CAP-001
#[test]
fn a_capture_writes_the_owner_empties_the_garrison_and_reveals_around_the_entrance() {
    let rules = town_rules();
    let setup = town_setup(objects(&rules), vec![garrison(), vec![]]);
    let mut state = started(&rules, &setup).unwrap();
    let town = rules.1.object_type_by_key("t:town").unwrap();
    // Red's town for blue, by a radius of 2: red sees it (its own), blue as its new owner.
    let before = state.explored(BLUE).unwrap().count();
    let mut events = Vec::new();
    let told = state.capture_town(
        &rules.0,
        TownId::new(0),
        BLUE,
        TownReveal::Radius(2),
        &mut events,
    );
    assert_eq!(told, Some(Viewers::NONE.with(RED).with(BLUE)));
    assert_eq!((state.owner(1), state.owner(2)), (Some(BLUE), None));
    assert_eq!(state.town_garrison(1), Some(&[][..]));
    assert!(state.check_owners(&rules.1));
    let blue = state.explored(BLUE).unwrap();
    let at = |x, y| Pos::new(x, y, 0);
    assert!(blue.get(at(16, 5)) && !blue.get(at(16, 6)));
    assert!(blue.get(at(18, 3)) && !blue.get(at(19, 3)));
    assert_eq!(
        events,
        [
            AdventureEvent {
                kind: AdventureEventKind::TownCaptured {
                    town: TownId::new(0),
                    anchor: at(16, 3),
                    owner: BLUE,
                },
                seen_by: Viewers::NONE.with(RED).with(BLUE),
            },
            AdventureEvent {
                kind: AdventureEventKind::TilesRevealed {
                    object_type: town,
                    count: blue.count() - before,
                },
                seen_by: Viewers::NONE.with(BLUE),
            },
        ]
    );
    assert!(blue.count() > before);
    // The neutral town for red: nobody else has explored it.
    let mut events = Vec::new();
    let told = state.capture_town(
        &rules.0,
        TownId::new(1),
        RED,
        TownReveal::Radius(1),
        &mut events,
    );
    assert_eq!(state.owner(2), Some(RED));
    assert_eq!(told, Some(Viewers::NONE.with(RED)));
    assert_eq!(events[0].seen_by, Viewers::NONE.with(RED));
    assert!(state.explored(RED).unwrap().get(at(24, 4)));
    // A `TownId` the map does not have changes nothing.
    let before = state.clone();
    let mut events = Vec::new();
    let told = state.capture_town(&rules.0, TownId::new(2), BLUE, TownReveal::All, &mut events);
    assert_eq!(told, None);
    assert_eq!(state, before);
    assert!(events.is_empty());
}

/// A two-level grass map of 8 × 6 tiles a level with a town on (4, 4) of the surface, red's at
/// the start, red's hero 1 on (0, 0) and blue's hero 2 on (7, 5) of the surface.
fn two_levels(rules: &Rules) -> AdventureSetup {
    let mut setup = world(
        &["gggggggg"; 12],
        2,
        vec![
            hero_at(1, 0, Pos::new(0, 0, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(7, 5, 0), &[PEASANT]),
        ],
    );
    setup.map.objects = vec![object(rules, "t:town", (4, 4), Some(0))];
    setup.garrisons = vec![vec![]];
    setup
}

// covers: TWN-CAP-001, ADV-OWN-002
#[test]
fn a_reveal_of_all_explores_every_tile_of_every_level() {
    let rules = town_rules();
    let setup = two_levels(&rules);
    let every = 8 * 6 * 2;
    // At the start, for the town's owner alone.
    let mut state = AdventureState::new(&rules.0, &rules.1, &setup, &mut draws()).unwrap();
    state.start_game(&rules.0, &rules.1, &[], &[TownReveal::All]);
    assert_eq!(state.explored(RED).unwrap().count(), every);
    assert!(state.explored(BLUE).unwrap().count() < every);
    assert!(!state.explored(BLUE).unwrap().get(Pos::new(0, 0, 1)));
    // At a capture, for its new owner.
    let mut events = Vec::new();
    state.capture_town(&rules.0, TownId::new(0), BLUE, TownReveal::All, &mut events);
    assert_eq!(state.explored(BLUE).unwrap().count(), every);
    assert_eq!(events.len(), 2);
}

// covers: TWN-CAP-003
#[test]
fn a_sieges_end_keeps_the_garrisons_survivors_or_captures_the_town_after_the_win() {
    // E09 T8b1: blue's town, its garrison fighting without a hero, red's hero below its entrance.
    let rules = town_rules();
    let state = at_town(&rules, Some(1), garrison(), (FAR, None));
    let heroes = store(&[]);
    let context = (&rules.0, &rules.1, &heroes[..]);
    let end = |survivors: &[ArmyStack], won| {
        let mut ended = state.clone();
        let outcome =
            ended.end_unowned_battle(context, (HERO, ENTRANCE), (survivors, won), &mut Vec::new());
        (ended, outcome)
    };
    // Lost: the survivors stay as the garrison, by slot; nothing follows.
    let survivors = [stack(2, PIKEMAN, 1)];
    let (lost, outcome) = end(&survivors, false);
    assert_eq!(lost.town_garrison(0), Some(&survivors[..]));
    assert_eq!(
        (outcome, lost.owner(0)),
        (AdventureOutcome::default(), Some(BLUE))
    );
    // Won: the garrison beaten, the capture after the winner's levels, the owner unchanged until
    // the game applies it; the hero stays below the entrance.
    let (mut won, outcome) = end(&[], true);
    assert_eq!(won.town_garrison(0), Some(&[][..]));
    assert_eq!(outcome.reward_after, Some((HERO, ENTRANCE)));
    let given = won.give_won(context, (HERO, ENTRANCE), &mut Vec::new());
    assert_eq!(given.capture, Some(TownId::new(0)));
    assert_eq!(
        (won.owner(0), won.hero(HERO).unwrap().pos),
        (Some(BLUE), p(4, 5))
    );
    // A town of the hero's team by then, or a hero no longer on the map: nothing.
    won.capture_town(
        &rules.0,
        TownId::new(0),
        RED,
        TownReveal::Radius(0),
        &mut Vec::new(),
    );
    let given = won.give_won(context, (HERO, ENTRANCE), &mut Vec::new());
    assert_eq!(given, AdventureOutcome::default());
    let (mut other, _) = end(&[], true);
    let gone = other.give_won(context, (HeroId::new(9), ENTRANCE), &mut Vec::new());
    assert_eq!(gone, AdventureOutcome::default());
}
