//! Tests of the town as a map object (E09 T8a1, the town epic's design note §8): the static map's
//! towns and their `TownId`s, the garrisons on their objects, the towns among the ownable objects
//! with their owners and the start's reveal by the game's radii, the formation the map holds and
//! the hash; since T8a2 the step onto a town (TWN-VIS-001), its visitor (TWN-VIS-002) and the
//! capture with its reveal (TWN-CAP-001; the town epic's design note §9, §11); since T8b2b the
//! siege against a visitor of another team and the garrison's merge into its army (TWN-SIE-001,
//! TWN-SIE-002; design note §10).

use h3_content::{CreatureDef, CreatureInfoDef};
use h3_content::{FactionId, SkillId};
use h3_kernel::hash::hash_of;

use super::*;
use crate::army::{ArmyRef, ArmySlot};
use crate::ruleset::{AdventurePack, ObjectKindDef, ObjectTypeDef, Visit};
use crate::testing::{
    FLYING, MAX_STACK, PEASANT, PIKEMAN, WATER_WALKING, draws, hero_at, hero_details, hero_skills,
    key, movement_rules_with, started, stream, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);

/// The creatures [`town_rules`] adds after [`movement_rules_with`]'s four, with their AI values:
/// batch TWN-1's row 3 (`experiments/specs/E09-batch-1.toml`, `CRTRAITS.TXT`'s values).
const MERGED: [(&str, u32); 9] = [
    ("t:swordsman", 445),
    ("t:centaur", 100),
    ("t:archer", 126),
    ("t:griffin", 351),
    ("t:pikeman80", 80),
    ("t:monk", 485),
    ("t:cavalier", 1946),
    ("t:halberdier", 115),
    ("t:marksman", 184),
];
const SWORDSMAN: CreatureTypeId = CreatureTypeId::from_raw(4);
const CENTAUR: CreatureTypeId = CreatureTypeId::from_raw(5);
const ARCHER: CreatureTypeId = CreatureTypeId::from_raw(6);
const GRIFFIN: CreatureTypeId = CreatureTypeId::from_raw(7);
/// A pikeman worth 80 (the test rulesets' [`PIKEMAN`] is worth nothing).
const PIKE: CreatureTypeId = CreatureTypeId::from_raw(8);
const MONK: CreatureTypeId = CreatureTypeId::from_raw(9);
const CAVALIER: CreatureTypeId = CreatureTypeId::from_raw(10);
const HALBERDIER: CreatureTypeId = CreatureTypeId::from_raw(11);
const MARKSMAN: CreatureTypeId = CreatureTypeId::from_raw(12);

/// A town's boat offsets in the base pack (TWN-SET-011): one column left, then right, two rows
/// below the entrance.
const BOAT_OFFSETS: [(i8, i8); 2] = [(-1, 2), (1, 2)];

/// [`movement_rules_with`] (faithful, maps up to 32 tiles) with a town `t:town` (the original's
/// boat offsets, [`BOAT_OFFSETS`]) and a mine `t:sawmill`, both entered and not from above, and
/// the part's flag radius 1; the skills of [`hero_skills`] (flying and water walking among them)
/// and the creatures of [`MERGED`].
fn town_rules() -> Rules {
    movement_rules_with(
        true,
        32,
        |core| {
            core.skills = hero_skills();
            core.creatures
                .extend(MERGED.iter().map(|&(k, ai_value)| CreatureDef {
                    key: key(k),
                    speed: 5,
                    info: CreatureInfoDef {
                        ai_value,
                        ..CreatureInfoDef::default()
                    },
                }));
        },
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
                ty(
                    "t:town",
                    ObjectKindDef::Town {
                        boat_offsets: BOAT_OFFSETS.to_vec(),
                    },
                ),
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
    // E09 T11: a faction per town, which only a random dwelling tied to a town reads.
    setup.town_factions = vec![FactionId::from_raw(0); garrisons.len()];
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

// covers: TWN-SET-011, ADV-SHY-001
#[test]
fn a_town_fixes_its_spot_when_the_game_is_built() {
    let rules = town_rules();
    // Red's town's entrance (16, 3), the neutral one's (24, 3): their offsets reach row 5.
    let built = |water: &[usize], more: Vec<ObjectSetup>| {
        let row5: alloc::string::String = (0..32)
            .map(|x| if water.contains(&x) { '~' } else { 'g' })
            .collect();
        let mut rows = ["gggggggggggggggggggggggggggggggg"; 16];
        rows[5] = &row5;
        let mut setup = town_setup(objects(&rules), vec![garrison(), vec![]]);
        setup.map.tiles = world(&rows, 1, Vec::new()).map.tiles;
        setup.map.objects.extend(more);
        started(&rules, &setup).unwrap()
    };
    let spots = |state: &AdventureState| {
        let map = state.map();
        [0, 1, 2].map(|town| map.town_boat_spot(TownId::new(town)))
    };
    // The first offset on free water: red's (15, 5) left, the neutral one's (25, 5) right.
    let state = built(&[15, 25], vec![]);
    assert_eq!(spots(&state), [Some(p(15, 5)), Some(p(25, 5)), None]);
    assert_eq!(
        state.map().objects()[1].resolved,
        Resolved::BoatTile(p(15, 5))
    );
    // Land under both offsets: none, the object resolving nothing.
    let land = built(&[], vec![]);
    assert_eq!(spots(&land), [None; 3]);
    assert_eq!(land.map().objects()[1].resolved, Resolved::Nothing);
    // Water an object blocks, then water an object is visited from, is passed over.
    let water = &[15, 16, 17, 25];
    let blocker = ObjectSetup {
        object_type: None,
        visitable: TileMask::default(),
        blocked: TileMask::from_bits(1),
        ..object(&rules, "t:sawmill", (15, 5), None)
    };
    let visited = ObjectSetup {
        object_type: None,
        blocked: TileMask::default(),
        ..object(&rules, "t:sawmill", (17, 5), None)
    };
    let blocked = built(water, vec![blocker.clone()]);
    assert_eq!(spots(&blocked)[0], Some(p(17, 5)));
    assert_eq!(spots(&built(water, vec![blocker, visited]))[0], None);
    // Every town gets one, owned or neutral; the owner's view keeps it out of the town's object
    // (the Shipyard's boat tile, ADV-OWN-003, stays the Shipyard's).
    let seen = state.observe(&rules.0, &rules.1, RED);
    let town = seen.objects.iter().find(|o| o.town == Some(TownId::new(0)));
    assert_eq!(
        town.map(|o| (o.owner, o.boat_tile)),
        Some((Some(RED), None))
    );
    // The static map's hash sees the spot.
    assert_ne!(state.map().content_hash(), land.map().content_hash());
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
    // E09 T11: one faction per town, whether a random dwelling reads it or not.
    let error = refused(&|s| s.town_factions.truncate(1));
    assert_eq!(error, SetupError::TownFactions(1));
    assert_eq!(
        error.to_string(),
        "1 town factions do not give one per town"
    );
    assert_eq!(
        refused(&|s| s.town_factions.push(FactionId::from_raw(1))),
        SetupError::TownFactions(3)
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
    let outcome = state.apply(
        &rules.0,
        &rules.1,
        heroes,
        (RED, &mut stream()),
        &walk,
        &mut events,
    )?;
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

// covers: TWN-VIS-001, TWN-SIE-001
#[test]
fn a_town_its_garrison_defends_is_besieged_from_the_tile_before() {
    // Blue's or a neutral town with a garrison: a walker, a water walker and a flyer each reach
    // its entrance as a destination from the tile below, as a monster's; the step is paid with
    // its drop and the hero stays; the siege against the garrison at the entrance, the town's
    // battle, its owner's side or the neutral seat's (survey §9.1's last row).
    let rules = town_rules();
    let (flyer, walker) = (store(&[(FLYING, 3)]), store(&[(WATER_WALKING, 3)]));
    for owner in [Some(1), None] {
        for heroes in [&[][..], &walker, &flyer] {
            let mut state = at_town(&rules, owner, garrison(), (FAR, None));
            let army = state.town_garrison(0).unwrap().to_vec();
            let reach = state.reach(&rules.0, &rules.1, heroes, HERO).unwrap();
            assert_eq!(reach.path(ENTRANCE), Some(vec![ENTRANCE]));
            let left = reach.points_left(ENTRANCE).unwrap();
            // A walk's visit ends the move there; a flight passes over it (ADV-MOV-009, as over a
            // monster's tile) and reaches the tile above.
            let past = walk(&rules, &mut state.clone(), heroes, &[ENTRANCE, p(5, 4)]);
            if heroes == &flyer[..] {
                assert!(past.unwrap().0.battle.is_none());
                assert!(reach.points_left(p(4, 3)).is_some());
                // Its visit is made from a stop node only: not from over the town's blocked tile.
                assert_eq!(
                    walk(&rules, &mut state.clone(), heroes, &[p(3, 4), ENTRANCE]).unwrap_err(),
                    AdventureCommandError::NoStop { step: 1 }
                );
            } else {
                assert_eq!(
                    past.unwrap_err(),
                    AdventureCommandError::PastVisit { step: 1 }
                );
            }
            let (outcome, events) = walk(&rules, &mut state, heroes, &[ENTRANCE]).unwrap();
            let battle = outcome.battle.clone().unwrap();
            assert_eq!(
                outcome,
                AdventureOutcome {
                    battle: Some(battle.clone()),
                    ..AdventureOutcome::default()
                }
            );
            let defender = match owner {
                Some(owner) => Defender::Garrison {
                    owner: PlayerId::new(owner),
                    army: army.clone(),
                },
                None => Defender::Guards { army: army.clone() },
            };
            assert_eq!(battle.defender, defender);
            assert_eq!(
                (battle.attacker.hero, battle.tile, battle.town),
                (HERO, ENTRANCE, Some(TownId::new(0)))
            );
            // The hero has not moved: no step's event; nothing else changed yet.
            assert!(events.is_empty(), "{events:?}");
            let hero = state.hero(HERO).unwrap();
            assert_eq!((hero.pos, hero.movement), (p(4, 5), left));
            assert_eq!(state.owner(0), owner.map(PlayerId::new));
            assert_eq!(state.town_garrison(0), Some(&army[..]));
            assert_eq!(state.town_visitor(TownId::new(0)), None);
        }
    }
    // Red's own town with a garrison is entered, not besieged (TWN-VIS-001's first case).
    let mut state = at_town(&rules, Some(0), garrison(), (FAR, None));
    let (outcome, _) = walk(&rules, &mut state, &[], &[ENTRANCE]).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    // The garrison's formation is the map's (the town epic's design note §8).
    assert_eq!(
        state.map().town_formation(TownId::new(0)),
        Some(Formation::Loose)
    );
    assert_eq!(state.map().town_formation(TownId::new(1)), None);
    assert_eq!(crate::town::holds_formation(&Holds::Nothing), None);
}

// covers: TWN-VIS-002, TWN-SIE-001
#[test]
fn another_teams_visitor_defends_its_town_besieged_from_the_tile_before() {
    // Blue's hero on the entrance of any town, red's own included, with a garrison or none,
    // defends it against red (survey §7.1, §7.2 step 3): a walker, a water walker and a flyer each
    // reach the entrance as a destination from the tile below, as a monster's; the step onto the
    // visitor is the siege against it from there, never a field battle: the step paid with its
    // drop, the hero stays, the visitor the defending hero at the entrance in the town's battle.
    // The garrison is merged into its army by the game as the battle starts (TWN-SIE-002).
    let rules = town_rules();
    let (flyer, walker) = (store(&[(FLYING, 3)]), store(&[(WATER_WALKING, 3)]));
    for owner in [Some(0), Some(1), None] {
        for army in [vec![], garrison()] {
            for heroes in [&[][..], &walker, &flyer] {
                let mut state = at_town(&rules, owner, army.clone(), (ENTRANCE, None));
                let visitor = state.hero(HeroId::new(2)).unwrap().combatant();
                let kept = state.town_garrison(0).unwrap().to_vec();
                let reach = state.reach(&rules.0, &rules.1, heroes, HERO).unwrap();
                assert_eq!(reach.path(ENTRANCE), Some(vec![ENTRANCE]));
                let left = reach.points_left(ENTRANCE).unwrap();
                let past = walk(&rules, &mut state.clone(), heroes, &[ENTRANCE, p(5, 4)]);
                if heroes == &flyer[..] {
                    // A flight passes over the visitor (ADV-MOV-009) and reaches the tile above;
                    // its siege is made from a stop node only.
                    assert!(past.unwrap().0.battle.is_none());
                    assert!(reach.points_left(p(4, 3)).is_some());
                    assert_eq!(
                        walk(&rules, &mut state.clone(), heroes, &[p(3, 4), ENTRANCE]).unwrap_err(),
                        AdventureCommandError::NoStop { step: 1 }
                    );
                } else {
                    assert_eq!(
                        past.unwrap_err(),
                        AdventureCommandError::PastVisit { step: 1 }
                    );
                }
                let (outcome, events) = walk(&rules, &mut state, heroes, &[ENTRANCE]).unwrap();
                let battle = outcome.battle.clone().unwrap();
                assert_eq!(
                    outcome,
                    AdventureOutcome {
                        battle: Some(battle.clone()),
                        ..AdventureOutcome::default()
                    }
                );
                assert_eq!(battle.defender, Defender::Hero(visitor.clone()));
                assert_eq!(
                    (battle.attacker.hero, battle.tile, battle.town),
                    (HERO, ENTRANCE, Some(TownId::new(0)))
                );
                assert!(events.is_empty(), "{events:?}");
                let hero = state.hero(HERO).unwrap();
                assert_eq!((hero.pos, hero.movement), (p(4, 5), left));
                assert_eq!(state.town_garrison(0), Some(&kept[..]));
                assert_eq!(state.town_visitor(TownId::new(0)), Some(HeroId::new(2)));
            }
        }
    }
    // Blue's hero off a town's entrance is met in a battle as before (GAM-BAT-004).
    let mut state = at_town(&rules, Some(1), vec![], (p(5, 5), None));
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert!(reach.points_left(p(5, 5)).is_some());
    let (outcome, _) = walk(&rules, &mut state, &[], &[p(5, 5)]).unwrap();
    assert_eq!(outcome.battle.unwrap().town, None);
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
    // object, the last in map order (ADV-OBJ-003). The town: its siege against the hero (E09
    // T8b2b); the sawmill: a field battle with the hero; the step and the reachable tiles agree.
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
        let left = reach.points_left(ENTRANCE).unwrap();
        let (outcome, _) = walk(&rules, &mut state, &[], &[ENTRANCE]).unwrap();
        let town = siege.then_some(TownId::new(0));
        assert_eq!(outcome.battle.unwrap().town, town);
        assert_eq!(state.hero(HERO).unwrap().movement, left);
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
    setup.town_factions = vec![FactionId::from_raw(0)];
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

/// Batch TWN-1's row 3 (`experiments/specs/E09-batch-1.toml`): the visitor's seven stacks, worth
/// 1 780, 400, 630, 1 755, 400, 1 455 and 800.
fn row_3_army() -> Vec<ArmyStack> {
    vec![
        stack(0, SWORDSMAN, 4),
        stack(1, CENTAUR, 4),
        stack(2, ARCHER, 5),
        stack(3, GRIFFIN, 5),
        stack(4, PIKE, 5),
        stack(5, MONK, 3),
        stack(6, PIKE, 10),
    ]
}

/// Batch TWN-1's row 3: the garrison's three stacks, worth 1 946, 1 840 and 1 840.
fn row_3_garrison() -> Vec<ArmyStack> {
    vec![
        stack(0, CAVALIER, 1),
        stack(1, HALBERDIER, 16),
        stack(2, MARKSMAN, 10),
    ]
}

/// Batch TWN-1's row 3 after the merge, as both runs read it (survey §15).
fn row_3_merged() -> Vec<ArmyStack> {
    vec![
        stack(0, SWORDSMAN, 4),
        stack(1, HALBERDIER, 16),
        stack(2, MARKSMAN, 10),
        stack(3, GRIFFIN, 5),
        stack(4, CAVALIER, 1),
        stack(5, MONK, 3),
        stack(6, PIKE, 15),
    ]
}

/// The army slots and a stack's most creatures of [`town_rules`].
const SLOTS: (u8, u32) = (7, MAX_STACK);

// covers: TWN-SIE-002
#[test]
fn the_garrison_merges_by_the_greatest_gain_until_none_gains() {
    // Batch TWN-1's row 3 (TD-7, confirmed): (1) the weakest is the last of the two worth 400
    // (slot 4's pikemen), the cavalier gains the most and changes places with it; (2) the first of
    // the two equal gains (the halberdiers before the marksmen) changes places with slot 1's
    // centaurs; (3) the marksmen with slot 2's archers; (4) the swapped-out pikemen join slot 6's
    // (their creature's stack: room, the gain their whole value); (5) nothing gains.
    let rules = town_rules();
    let (mut army, mut garrison) = (row_3_army(), row_3_garrison());
    crate::town::merge(&rules.0, SLOTS, &mut army, &mut garrison);
    assert_eq!(army, row_3_merged());
    assert_eq!(garrison, vec![stack(1, CENTAUR, 4), stack(2, ARCHER, 5)]);
}

// covers: TWN-SIE-002
#[test]
fn a_garrison_stack_joins_where_the_army_has_room_and_a_worthless_one_never() {
    let rules = town_rules();
    let merge = |mut army: Vec<ArmyStack>, mut garrison: Vec<ArmyStack>| {
        crate::town::merge(&rules.0, SLOTS, &mut army, &mut garrison);
        (army, garrison)
    };
    // Room, a stack of its creature or a free slot, makes the gain its whole value: the pikemen
    // (160) join slot 2's, then the centaur (100, less than the weakest's 240) the first free
    // slot. The peasants and a creature the ruleset does not have are worth nothing: they stay.
    let unknown = CreatureTypeId::from_raw(99);
    let (army, garrison) = merge(
        vec![stack(0, CAVALIER, 1), stack(2, PIKE, 1)],
        vec![
            stack(3, PIKE, 2),
            stack(5, CENTAUR, 1),
            stack(6, PEASANT, 50),
            stack(1, unknown, 9),
        ],
    );
    assert_eq!(
        army,
        vec![
            stack(0, CAVALIER, 1),
            stack(1, CENTAUR, 1),
            stack(2, PIKE, 3)
        ]
    );
    assert_eq!(garrison, vec![stack(6, PEASANT, 50), stack(1, unknown, 9)]);
    // A full army whose pikemen's stack cannot take two more (ADV-ARM-007): no room, so the
    // pikemen gain their value less the weakest's (160 − 100) and change places with the last of
    // the equal centaurs, which then joins the first centaurs' stack (room: its whole value).
    let mut full = vec![stack(0, PIKE, MAX_STACK - 1)];
    full.extend((1..7).map(|slot| stack(slot, CENTAUR, 1)));
    let (army, garrison) = merge(full, vec![stack(0, PIKE, 2)]);
    let mut expected = vec![stack(0, PIKE, MAX_STACK - 1), stack(1, CENTAUR, 2)];
    expected.extend((2..6).map(|slot| stack(slot, CENTAUR, 1)));
    expected.push(stack(6, PIKE, 2));
    assert_eq!((army, garrison), (expected, vec![]));
    // Equal gains, one with room (the pikemen, 320) and one without (the centaurs, 400 − 80):
    // the first slot's, whichever kind; the centaurs change places with the weakest pikeman, and
    // then nothing gains (the army has no pikemen left).
    let mut full = vec![stack(0, PIKE, 1)];
    full.extend((1..7).map(|slot| stack(slot, SWORDSMAN, 1)));
    let (army, garrison) = merge(full, vec![stack(0, CENTAUR, 4), stack(1, PIKE, 4)]);
    assert_eq!(army[0], stack(0, CENTAUR, 4));
    assert_eq!(garrison, vec![stack(0, PIKE, 1), stack(1, PIKE, 4)]);
    // A stack worth no more than the weakest, without room, never changes places.
    let (army, garrison) = merge(
        (0..7).map(|slot| stack(slot, CENTAUR, 2)).collect(),
        vec![stack(0, PIKE, 2), stack(1, ARCHER, 1)],
    );
    assert_eq!(army.len(), 7);
    assert_eq!(garrison, vec![stack(0, PIKE, 2), stack(1, ARCHER, 1)]);
    // An army without a stack (never a hero's) takes nothing.
    let (army, garrison) = merge(vec![], row_3_garrison());
    assert_eq!((army, garrison), (vec![], row_3_garrison()));
}

// covers: TWN-SIE-002
#[test]
fn the_state_merges_a_towns_garrison_into_its_visitor_and_names_the_changed_slots() {
    let rules = town_rules();
    let blue = HeroId::new(2);
    // Blue's town with row 3's garrison, blue's hero on its entrance with row 3's army.
    let mut state = at_town(&rules, Some(1), row_3_garrison(), (ENTRANCE, None));
    state.hero_mut(blue).unwrap().army = row_3_army();
    let slots = state.merge_garrison(&rules.0, &rules.1, TownId::new(0));
    let hero = |slot| ArmySlot {
        army: ArmyRef::Hero(blue),
        slot,
    };
    let town = |slot| ArmySlot {
        army: ArmyRef::Garrison(TownId::new(0)),
        slot,
    };
    // The visitor's slots, then the garrison's, each in slot order.
    assert_eq!(
        slots,
        vec![
            hero(1),
            hero(2),
            hero(4),
            hero(6),
            town(0),
            town(1),
            town(2)
        ]
    );
    assert_eq!(state.hero(blue).unwrap().army, row_3_merged());
    let left = [stack(1, CENTAUR, 4), stack(2, ARCHER, 5)];
    assert_eq!(state.town_garrison(0), Some(&left[..]));
    // Again: nothing gains, nothing changes, the garrison's block as it was.
    let before = state.clone();
    assert_eq!(
        state.merge_garrison(&rules.0, &rules.1, TownId::new(0)),
        vec![]
    );
    assert_eq!(state, before);
    // A town without a visitor, or one the map does not have: nothing.
    let mut alone = at_town(&rules, Some(1), row_3_garrison(), (FAR, None));
    let before = alone.clone();
    for id in [0, 1] {
        assert_eq!(
            alone.merge_garrison(&rules.0, &rules.1, TownId::new(id)),
            vec![]
        );
    }
    assert_eq!(alone, before);
}
