//! The whole game's wire (E16 M5 T2a): the adventure map maps every field of the engine's
//! observation, losslessly (a test-only inverse gives the engine's observation back), for every
//! viewer of games played at random on the fixture maps; its parts round-trip.

use std::sync::{Arc, LazyLock};
use std::vec;
use std::vec::Vec;

use h3_adventure::{
    AdventureObservation, BoatView, GuardsView, HeroView, Known as EngineKnown, KnownObject,
    MonsterView, ObjectView, OwnHeroView, StaticTile, TileMask, TileView,
};
use h3_content::Formation as EngineFormation;
use h3_contracts::{ArmyStack as EngineStack, HeroId, PlayerId, TownId};
use h3_game::{GameRuleset, Viewer};
use h3_kernel::handle::Handle;
use h3_kernel::rng::{Rng, StreamId};
use h3_sim::adventure::AdventureEnv;
use h3_sim::adventure::rollout::choose;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::map::game::{adventure, adventure_view};
use crate::wire::WireU64;
use crate::wire::game::*;

static RULES: LazyLock<GameRuleset> = LazyLock::new(|| {
    let profile = "base:faithful";
    let (core, part) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile).unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .unwrap();
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS, profile).unwrap();
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .unwrap()
});

fn round_trips<T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug>(value: &T) {
    let json = serde_json::to_vec(value).unwrap();
    assert_eq!(&serde_json::from_slice::<T>(&json).unwrap(), value);
    let binary = postcard::to_allocvec(value).unwrap();
    assert_eq!(&postcard::from_bytes::<T>(&binary).unwrap(), value);
}

// The inverse: the wire back to the engine's types, field by field (test code, independent of
// the mapping it checks).

fn pos_back(p: Pos) -> h3_adventure::Pos {
    h3_adventure::Pos::new(p.x, p.y, p.level)
}

fn army_back(stacks: &[ArmyStack]) -> Vec<EngineStack> {
    stacks
        .iter()
        .map(|s| EngineStack {
            slot: s.slot,
            creature: Handle::from_raw(s.creature),
            count: s.count,
        })
        .collect()
}

fn tile_back(t: Tile) -> TileView {
    match t {
        Tile::Hidden {} => TileView::Hidden,
        Tile::Ground {
            terrain,
            road,
            river,
        } => TileView::Ground {
            terrain: Handle::from_raw(terrain),
            road: road.map(Handle::from_raw),
            river: river.map(Handle::from_raw),
        },
        Tile::Seen {
            terrain,
            road,
            river,
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        } => TileView::Seen(StaticTile {
            terrain: Handle::from_raw(terrain),
            road: road.map(Handle::from_raw),
            river: river.map(Handle::from_raw),
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        }),
    }
}

fn inverse(a: &Adventure) -> AdventureObservation {
    AdventureObservation {
        viewer: PlayerId::new(a.viewer),
        width: a.width,
        height: a.height,
        levels: a.levels,
        stride: usize::from(a.stride),
        explored: a.explored.iter().map(|w| w.0).collect(),
        fringe: a.fringe.iter().map(|w| w.0).collect(),
        tiles: (0..a.tiles.len())
            .map(|i| tile_back(*a.tile(i).unwrap()))
            .collect(),
        heroes: a
            .heroes
            .iter()
            .map(|h| HeroView {
                id: HeroId::new(h.id),
                owner: PlayerId::new(h.owner),
                pos: pos_back(h.pos),
                boat: h.boat,
                own: h.own.as_ref().map(|o| OwnHeroView {
                    army: army_back(&o.army),
                    movement: o.movement,
                    max_movement: o.max_movement,
                    formation: match o.formation {
                        Formation::Loose => EngineFormation::Loose,
                        Formation::Tight => EngineFormation::Tight,
                    },
                    tactics: o.tactics,
                    marks: o
                        .marks
                        .iter()
                        .map(|m| (Handle::from_raw(m.object_type), pos_back(m.anchor)))
                        .collect(),
                }),
            })
            .collect(),
        objects: a
            .objects
            .iter()
            .map(|o| ObjectView {
                object_type: Handle::from_raw(o.object_type),
                anchor: pos_back(o.anchor),
                blocked: TileMask::from_bits(o.blocked.0),
                visitable: TileMask::from_bits(o.visitable.0),
                drawn: TileMask::from_bits(o.drawn.0),
                monster: o.monster.map(|m| MonsterView {
                    creature: Handle::from_raw(m.creature),
                    count_word: m.count_word,
                }),
                owner: o.owner.map(PlayerId::new),
                production: o
                    .production
                    .map(|p| (Handle::from_raw(p.resource), p.amount)),
                guards: o.guards.as_ref().map(|g| GuardsView {
                    creature: g.creature.map(Handle::from_raw),
                    count_word: g.count_word,
                    army: g.army.as_deref().map(army_back),
                }),
                visited: o.visited,
                boat_tile: o.boat_tile.map(pos_back),
                town: o.town.map(TownId::new),
            })
            .collect(),
        bought: a.bought.iter().copied().map(Handle::from_raw).collect(),
        keys: a.keys,
        known: a
            .known
            .iter()
            .map(|k| KnownObject {
                object_type: Handle::from_raw(k.object_type),
                anchor: pos_back(k.anchor),
                known: match &k.known {
                    Known::Spell { spell } => EngineKnown::Spell(spell.map(Handle::from_raw)),
                    Known::Skill { skill } => EngineKnown::Skill(skill.map(Handle::from_raw)),
                    Known::Price { price } => EngineKnown::Price(
                        price
                            .iter()
                            .map(|p| (Handle::from_raw(p.resource), p.amount))
                            .collect(),
                    ),
                    Known::Skills { skills } => {
                        EngineKnown::Skills(skills.iter().copied().map(Handle::from_raw).collect())
                    }
                    Known::Emptied { emptied } => EngineKnown::Emptied(*emptied),
                },
            })
            .collect(),
        boats: a
            .boats
            .iter()
            .map(|b| BoatView {
                tile: pos_back(b.tile),
                kind: b.kind,
                facing: b.facing,
            })
            .collect(),
    }
}

/// The palette holds each view once, in the order the tiles first use them.
fn check_palette(a: &Adventure) {
    let mut next = 0u32;
    for &i in &a.tiles {
        assert!(i <= next, "index {i} before {next}");
        if i == next {
            next += 1;
        }
    }
    assert_eq!(next as usize, a.palette.len());
    let distinct: std::collections::BTreeSet<_> = a.palette.iter().collect();
    assert_eq!(distinct.len(), a.palette.len());
}

fn stacks(n: u8) -> Vec<EngineStack> {
    (0..n)
        .map(|slot| EngineStack {
            slot,
            creature: Handle::from_raw(10 + u16::from(slot)),
            count: 5 * u32::from(slot) + 1,
        })
        .collect()
}

/// An observation with every variant and every optional part both present and absent.
fn every_part() -> AdventureObservation {
    let p = h3_adventure::Pos::new;
    let seen = |blocked| {
        TileView::Seen(StaticTile {
            terrain: Handle::from_raw(2),
            road: Some(Handle::from_raw(1)),
            river: Some(Handle::from_raw(3)),
            blocked,
            visitable: !blocked,
            no_native: true,
            winds: false,
            hole: true,
        })
    };
    let ground = TileView::Ground {
        terrain: Handle::from_raw(7),
        road: Some(Handle::from_raw(0)),
        river: None,
    };
    let object = |town: bool| ObjectView {
        object_type: Handle::from_raw(40),
        anchor: p(3, 1, 0),
        blocked: TileMask::from_bits(0b0110 << 40),
        visitable: TileMask::from_bits(1 << 47),
        drawn: TileMask::from_bits(0xffff_ffff_ffff),
        monster: town.then_some(MonsterView {
            creature: Handle::from_raw(12),
            count_word: 4,
        }),
        owner: town.then_some(PlayerId::new(1)),
        production: town.then_some((Handle::from_raw(6), 1000)),
        guards: town.then(|| GuardsView {
            creature: None,
            count_word: 2,
            army: Some(stacks(2)),
        }),
        visited: town,
        boat_tile: town.then_some(p(4, 2, 0)),
        town: town.then_some(TownId::new(3)),
    };
    AdventureObservation {
        viewer: PlayerId::new(1),
        width: 3,
        height: 2,
        levels: 1,
        stride: 1,
        explored: vec![0b11_0100],
        fringe: vec![0b00_1010],
        tiles: vec![
            TileView::Hidden,
            ground,
            seen(true),
            seen(true),
            seen(false),
            TileView::Hidden,
        ],
        heroes: vec![
            HeroView {
                id: HeroId::new(4),
                owner: PlayerId::new(1),
                pos: p(2, 1, 0),
                boat: None,
                own: Some(OwnHeroView {
                    army: stacks(3),
                    movement: 1500,
                    max_movement: 2000,
                    formation: EngineFormation::Tight,
                    tactics: false,
                    marks: vec![(Handle::from_raw(9), p(1, 1, 0))],
                }),
            },
            HeroView {
                id: HeroId::new(5),
                owner: PlayerId::new(1),
                pos: p(0, 1, 0),
                boat: Some(2),
                own: Some(OwnHeroView {
                    army: Vec::new(),
                    movement: 0,
                    max_movement: 1300,
                    formation: EngineFormation::Loose,
                    tactics: true,
                    marks: Vec::new(),
                }),
            },
            HeroView {
                id: HeroId::new(9),
                owner: PlayerId::new(0),
                pos: p(1, 0, 0),
                boat: None,
                own: None,
            },
        ],
        objects: vec![
            object(true),
            object(false),
            ObjectView {
                guards: Some(GuardsView {
                    creature: Some(Handle::from_raw(8)),
                    count_word: 0,
                    army: None,
                }),
                ..object(false)
            },
        ],
        bought: vec![Handle::from_raw(30), Handle::from_raw(31)],
        keys: 0b1000_0001,
        known: [
            EngineKnown::Spell(Some(Handle::from_raw(17))),
            EngineKnown::Spell(None),
            EngineKnown::Skill(Some(Handle::from_raw(5))),
            EngineKnown::Skill(None),
            EngineKnown::Price(vec![(Handle::from_raw(6), 2000), (Handle::from_raw(2), 5)]),
            EngineKnown::Skills(vec![Handle::from_raw(1), Handle::from_raw(11)]),
            EngineKnown::Emptied(true),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, known)| KnownObject {
            object_type: Handle::from_raw(50),
            anchor: p(u16::try_from(i).unwrap(), 0, 0),
            known,
        })
        .collect(),
        boats: vec![BoatView {
            tile: p(1, 1, 0),
            kind: 1,
            facing: 6,
        }],
    }
}

#[test]
fn every_part_of_the_adventure_map_maps_back_and_round_trips() {
    let o = every_part();
    let wire = adventure_view(&o);
    assert_eq!(inverse(&wire), o);
    assert_eq!(wire.tiles, [0, 1, 2, 2, 3, 0]);
    assert_eq!(wire.palette.len(), 4);
    check_palette(&wire);
    assert_eq!(wire.tile(6), None);
    round_trips(&wire);
}

fn text<T: Serialize>(value: &T) -> std::string::String {
    serde_json::to_string(value).unwrap()
}

#[test]
fn the_adventure_maps_json_is_serdes_default_form() {
    assert_eq!(text(&Tile::Hidden {}), r#"{"Hidden":{}}"#);
    assert_eq!(
        text(&Tile::Ground {
            terrain: 7,
            road: Some(0),
            river: None
        }),
        r#"{"Ground":{"terrain":7,"road":0,"river":null}}"#
    );
    assert_eq!(text(&Formation::Tight), r#""Tight""#);
    assert_eq!(
        text(&Known::Spell { spell: None }),
        r#"{"Spell":{"spell":null}}"#
    );
    // A plane's word and a mask are 64-bit values: decimal strings.
    let wire = adventure_view(&every_part());
    assert_eq!(text(&wire.explored), r#"["52"]"#);
    assert_eq!(text(&wire.objects[0].visitable), r#""140737488355328""#);
    assert_eq!(text(&wire.objects[0].anchor), r#"{"x":3,"y":1,"level":0}"#);
    // Unknown fields are refused.
    let extra = r#"{"x":3,"y":1,"level":0,"z":1}"#;
    assert!(serde_json::from_str::<Pos>(extra).is_err());
    assert_eq!(WireU64(52), wire.explored[0]);
}

/// Plays `steps` random steps of `map` and checks every viewer's map after each.
fn check_played(map: &[u8], seed: u64, steps: usize) -> (usize, usize) {
    let rules = &*RULES;
    let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
    let setup = Arc::new(h3_map::import(&m, rules).unwrap());
    let mut env = AdventureEnv::new(rules.clone(), setup, seed, 28).unwrap();
    let mut rng = Rng::for_stream(seed, StreamId::AGENT);
    let (mut largest, mut checked) = (0, 0);
    for step in 0..steps {
        for player in (0..=u8::MAX).take(env.state().players()) {
            let viewer = PlayerId::new(player);
            let o = env
                .state()
                .adventure()
                .observe(rules.core(), rules.adventure(), viewer);
            let wire = adventure(env.state(), rules, Viewer::Player(viewer));
            assert_eq!(wire, adventure_view(&o));
            assert_eq!(inverse(&wire), o, "step {step}, player {player}");
            check_palette(&wire);
            if step % 50 == 0 {
                round_trips(&wire);
            }
            largest = largest.max(wire.palette.len());
            checked += 1;
        }
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        env.step(seat, action).unwrap();
    }
    (largest, checked)
}

#[test]
fn a_played_games_maps_map_back_for_every_viewer() {
    let maps: [(&[u8], u64); 5] = [
        (
            include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m"),
            1,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e09-towns-36.h3m"),
            2,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-rewards-36.h3m"),
            3,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-economy-36.h3m"),
            4,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-sea-36.h3m"),
            5,
        ),
    ];
    for (map, seed) in maps {
        let (largest, checked) = check_played(map, seed, 400);
        assert!(checked >= 300, "{checked}");
        // A map's distinct tile views are few: 4 to 64 on these maps of 2 592 tiles.
        assert!((2..200).contains(&largest), "{largest}");
    }
}
