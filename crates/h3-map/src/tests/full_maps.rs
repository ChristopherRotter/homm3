//! Decision 133's maps (E08 T64b; design §10 and §16): SoD, 144 × 144 with an underground (the
//! XL map's 41 472 tiles), a Pandora's Box on every tile no hero stands on, and the same map with
//! a Learning Stone on every such tile; written by [`crate::writer`] with the original's templates
//! and committed under `fixtures/maps/` for the fork benchmarks (`benches/adventure/`:
//! `fork-boxes-xl-144`, `fork-stones-xl-144`). The tests check that the committed files read to
//! the maps written here (they rewrite them when `H3_WRITE_FIXTURES` is set), and that each map
//! builds, plays a few hundred steps of random play with visits and forks: the engine's state of
//! visits and removals has no original limit (decision 133: neither A-17's 32 marks per class nor
//! A-19's 400 shared records), and a fork stays one reference count of it.

use std::path::PathBuf;

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::strategy::ValueTree;

use super::play::{FAITHFUL, Step, options, resolve};
use crate::model::Rewards;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_game::{GameState, apply};

/// The original's templates (`Objects.txt`): Pandora's Box and the Learning Stone, each one tile,
/// blocked and visited (survey `E08-rewards.md` §1; as `rewards_map.rs` carries them).
const PANDORA: &str = "AVA0128.def \
    011111111111111111111111111111111111111111111111 \
    100000000000000000000000000000000000000000000000 111111111 000000001 6 0 4 0";
const LEARNING_STONE: &str = "AVSgzbo0.def \
    011111111111111111111111111111111111111111111111 \
    100000000000000000000000000000000000000000000000 111111111 011111111 100 0 0 0";

/// The maps' side.
const SIDE: u8 = 144;
/// The heroes' tiles, on both levels: red's in the north-west corner, blue's in the south-east.
const HEROES: [(u8, u8); 2] = [(8, 8), (135, 135)];
/// Resources by `RESTYPES.TXT` index.
const GOLD: usize = 6;

/// What every free tile holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fill {
    /// A Pandora's Box that gives 100 experience and 100 gold behind its question (ADV-PIK-013):
    /// opened, it leaves the map.
    Boxes,
    /// A Learning Stone (ADV-TCH-001): 1 000 experience once per hero, its mark in the visits'
    /// block; entered, so heroes walk from stone to stone.
    Stones,
}

impl Fill {
    /// The committed file's name.
    const fn name(self) -> &'static str {
        match self {
            Self::Boxes => "e08-boxes-144.h3m",
            Self::Stones => "e08-stones-144.h3m",
        }
    }

    /// The object on `tile`.
    fn object(self, tile: (u8, u8)) -> w::MapObject {
        match self {
            Self::Boxes => {
                let mut resources = [0; 7];
                resources[GOLD] = 100;
                w::MapObject::Reward(w::RewardObject {
                    tile,
                    template: template(PANDORA),
                    body: w::RewardBody::Pandora {
                        message: None,
                        rewards: Rewards {
                            experience: 100,
                            resources,
                            ..Rewards::default()
                        },
                    },
                })
            }
            Self::Stones => w::MapObject::Plain(w::Plain {
                tile,
                template: template(LEARNING_STONE),
            }),
        }
    }
}

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

/// A hero of `hero_type` (its class's picture) on `at`, its type's own.
fn hero(at: (u8, u8), owner: u8, hero_type: u8) -> w::MapObject {
    w::MapObject::Hero(w::Hero {
        tile: at,
        owner,
        hero_type,
        class: hero_type / 8,
        name: None,
        experience: None,
        skills: None,
        army: None,
        tight_formation: false,
        artifacts: None,
        spells: None,
        primary: None,
    })
}

/// One level's objects: red's and blue's hero (of these types), then `fill`'s object on every
/// other tile, row by row.
fn level(fill: Fill, types: [u8; 2]) -> Vec<w::MapObject> {
    let heroes = HEROES
        .iter()
        .zip(types)
        .zip(0..)
        .map(|((&at, hero_type), owner)| hero(at, owner, hero_type));
    let tiles = (0..SIDE).flat_map(|y| (0..SIDE).map(move |x| (x, y)));
    let free = tiles.filter(|tile| !HEROES.contains(tile));
    heroes.chain(free.map(|tile| fill.object(tile))).collect()
}

/// The map: SoD, 144 × 144 of grass over an underground of subterranean ground; red and blue
/// with a hero on each level, every other tile `fill`'s.
fn map(fill: Fill) -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    w::Map {
        name: format!("Full of {fill:?}"),
        description: "E08 T64b: decision 133, an object on every free tile".into(),
        size: SIDE,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects: level(fill, [0, 25]),
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: Vec::new(),
            objects: level(fill, [9, 34]),
        }),
        level_limit: 0,
        banned_skills: Vec::new(),
    }
}

/// The committed file of `fill`'s map, checked to read to the map written here (rewritten first
/// when `H3_WRITE_FIXTURES` is set).
fn committed(fill: Fill) -> crate::model::RawMap {
    let bytes = w::write(&map(fill)).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/maps")
        .join(fill.name());
    if std::env::var_os("H3_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, w::gzip(&bytes)).unwrap();
    }
    let file = std::fs::read(&path).unwrap();
    let written = read(&w::gzip(&bytes), &Limits::default()).unwrap();
    let committed = read(&file, &Limits::default()).unwrap();
    assert!(
        committed == written,
        "{} is not the map written here: H3_WRITE_FIXTURES=1 rewrites it",
        fill.name()
    );
    committed
}

#[test]
fn the_full_maps_are_committed_as_written() {
    for fill in [Fill::Boxes, Fill::Stones] {
        let m = committed(fill);
        let setup = import(&m, &FAITHFUL).unwrap();
        // Every tile but the four heroes' holds one object.
        let tiles = 2 * usize::from(SIDE) * usize::from(SIDE);
        assert_eq!(setup.map.objects.len(), tiles - 4, "{fill:?}");
        assert_eq!(setup.heroes.len(), 4, "{fill:?}");
    }
}

/// Decision 133's test (design §16): each full map builds, plays 300 steps of random play that
/// visit (its boxes opened and gone, its stones' marks in the visits' block), and forks: a fork
/// playing on leaves its parent as it was, and evolves as the parent does under the same steps.
// covers: ADV-PIK-013, ADV-TCH-001, ADV-VIS-006
#[test]
fn a_map_full_of_boxes_or_stones_builds_plays_and_forks() {
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let steps = vec(
        prop_oneof![
            3 => any::<u16>().prop_map(Step::Visit),
            3 => any::<u16>().prop_map(Step::Legal),
            1 => Just(Step::EndTurn),
        ],
        360,
    );
    for fill in [Fill::Boxes, Fill::Stones] {
        let rules = &*FAITHFUL;
        let m = committed(fill);
        let setup = import(&m, rules).unwrap();
        let mut state = GameState::new(rules, &setup, 1).unwrap();
        let steps = steps.new_tree(&mut runner).unwrap().current();
        let (played, rest) = steps.split_at(300);
        let mut visits = 0;
        for step in played {
            let legal = options(&state, rules);
            let (actor, command) = resolve(&state, rules, &legal, step);
            let mut events = Vec::new();
            if apply(&mut state, rules, actor, command, &mut events).is_ok() {
                visits += usize::from(events.iter().any(|e| {
                    matches!(
                        e,
                        h3_game::GameEvent::OfferOpened { .. }
                            | h3_game::GameEvent::Adventure(h3_adventure::AdventureEvent {
                                kind: h3_adventure::AdventureEventKind::ObjectRemoved { .. },
                                ..
                            })
                            | h3_game::GameEvent::HeroChanged { .. }
                    )
                }));
            }
        }
        let adventure = state.adventure();
        let objects = adventure.map().objects().len();
        match fill {
            Fill::Boxes => {
                let opened = (0..objects).filter(|&i| adventure.is_removed(i)).count();
                assert!(opened >= 8, "{opened} boxes opened");
            }
            Fill::Stones => {
                let block = adventure.visits().expect("the visits' block");
                let heroes = adventure.heroes().iter().map(|h| h.id);
                let marks = heroes
                    .flat_map(|hero| (0..objects).filter(move |&i| block.is_marked(hero, i)))
                    .count();
                assert!(marks >= 8, "{marks} stones marked");
            }
        }
        assert!(visits >= 16, "{fill:?}: {visits} visits");
        // A fork: playing on leaves the parent as it was; under the same steps the two evolve
        // alike.
        let parent = state.clone();
        let mut fork = state.clone();
        for step in rest {
            for game in [&mut state, &mut fork] {
                let legal = options(game, rules);
                let (actor, command) = resolve(game, rules, &legal, step);
                let _ = apply(game, rules, actor, command, &mut Vec::new());
            }
            assert_eq!(state, fork, "{fill:?}");
        }
        assert_eq!(state.state_hash(), fork.state_hash());
        assert_ne!(
            parent.state_hash(),
            state.state_hash(),
            "{fill:?}: nothing played"
        );
        let replayed = {
            let mut again = parent.clone();
            for step in rest {
                let legal = options(&again, rules);
                let (actor, command) = resolve(&again, rules, &legal, step);
                let _ = apply(&mut again, rules, actor, command, &mut Vec::new());
            }
            again
        };
        assert_eq!(replayed, state, "{fill:?}: the parent changed");
    }
}
