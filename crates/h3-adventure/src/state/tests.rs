//! Tests of the adventure setup and state (E08 T5a): the static map, the footprints of objects,
//! where heroes may stand, the armies, one test per setup error, and the canonical hash.

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_content::{CreatureTypeId, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::*;
use crate::setup::TileSetup;
use crate::testing::rules;

const GRASS: TerrainId = TerrainId::from_raw(0);
const SEA: TerrainId = TerrainId::from_raw(1);
const ROCK: TerrainId = TerrainId::from_raw(2);
const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

fn grass() -> TileSetup {
    TileSetup {
        terrain: GRASS,
        road: None,
        river: None,
    }
}

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

fn hero(id: u16, owner: u8, pos: Pos) -> HeroSetup {
    HeroSetup {
        id: HeroId::new(id),
        owner: PlayerId::new(owner),
        pos,
        army: vec![stack(0, IMP, 10)],
    }
}

/// Bits of a mask from `(column, row)` offsets.
fn mask(offsets: &[(u8, u8)]) -> TileMask {
    TileMask::from_bits(
        offsets
            .iter()
            .map(|&(c, r)| 1u64 << (u32::from(r) * 8 + u32::from(c)))
            .sum(),
    )
}

/// A 4 × 3 map on two levels, all grass except sea at (3, 0) and rock at (3, 1) of the surface,
/// with a road at (0, 0) and a river at (1, 0). An object anchored at (1, 1) blocks (1, 1) and
/// (0, 1) (its third blocked tile is off the map) and is visited from (0, 0); a second one on the
/// underground blocks (1, 2) and (2, 2) and is visited from (2, 2), like a town's entrance. Two
/// heroes, listed out of id order, one of them on the first object's visitable tile.
fn setup() -> AdventureSetup {
    let mut tiles = vec![grass(); 4 * 3 * 2];
    tiles[3].terrain = SEA;
    tiles[7].terrain = ROCK;
    tiles[0].road = Some(RoadId::from_raw(0));
    tiles[1].river = Some(RiverId::from_raw(0));
    AdventureSetup {
        players: 2,
        map: MapSetup {
            width: 4,
            height: 3,
            levels: 2,
            tiles,
            objects: vec![
                ObjectSetup {
                    anchor: Pos::new(1, 1, 0),
                    blocked: mask(&[(0, 0), (1, 0), (2, 0)]),
                    visitable: mask(&[(1, 1)]),
                    kind: ObjectKind::Blocker,
                },
                ObjectSetup {
                    anchor: Pos::new(2, 2, 1),
                    blocked: mask(&[(0, 0), (1, 0)]),
                    visitable: mask(&[(0, 0)]),
                    kind: ObjectKind::Blocker,
                },
            ],
        },
        heroes: vec![
            HeroSetup {
                army: vec![stack(2, GOG, 3), stack(0, IMP, 12)],
                ..hero(5, 1, Pos::new(2, 2, 0))
            },
            // On the first object's visitable tile, which a hero may stand on.
            hero(1, 0, Pos::new(0, 0, 0)),
        ],
    }
}

fn build(setup: &AdventureSetup) -> Result<AdventureState, SetupError> {
    let (core, part) = rules();
    AdventureState::new(&core, &part, setup)
}

fn error(edit: impl FnOnce(&mut AdventureSetup)) -> SetupError {
    let mut setup = setup();
    edit(&mut setup);
    build(&setup).unwrap_err()
}

// covers: ADV-OBJ-001
#[test]
fn builds_the_static_map_with_the_objects_footprints() {
    let state = build(&setup()).unwrap();
    let map = state.map();
    assert_eq!((map.width(), map.height(), map.levels()), (4, 3, 2));
    assert_eq!(map.tiles().len(), 24);
    let at = |x, y, level| *map.tile(Pos::new(x, y, level)).unwrap();
    assert_eq!(at(3, 0, 0).terrain, SEA);
    assert_eq!(at(3, 1, 0).terrain, ROCK);
    assert_eq!(at(0, 0, 0).road, Some(RoadId::from_raw(0)));
    assert_eq!(at(1, 0, 0).river, Some(RiverId::from_raw(0)));
    assert_eq!((at(1, 0, 0).road, at(0, 0, 0).river), (None, None));
    let flags = |level| -> Vec<(u16, u16, bool, bool)> {
        (0..3)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .map(|(x, y)| {
                let t = at(x, y, level);
                (x, y, t.blocked, t.visitable)
            })
            .filter(|&(_, _, b, v)| b || v)
            .collect()
    };
    // The blocked tile two columns left of (1, 1) is off the map and ignored.
    assert_eq!(
        flags(0),
        [
            (0, 0, false, true),
            (0, 1, true, false),
            (1, 1, true, false)
        ]
    );
    assert_eq!(flags(1), [(1, 2, true, false), (2, 2, true, true)]);
    assert_eq!(map.tile(Pos::new(4, 0, 0)), None);
    assert_eq!(map.tile(Pos::new(0, 3, 0)), None);
    assert_eq!(map.tile(Pos::new(0, 0, 2)), None);
    assert_eq!(map.index(Pos::new(3, 2, 1)), Some(23));
    assert_eq!(map.objects().len(), 2);
    assert_eq!(map.objects()[1].anchor, Pos::new(2, 2, 1));
    assert_eq!(map.objects()[1].kind, ObjectKind::Blocker);
    assert_eq!(map.objects()[0].blocked, mask(&[(0, 0), (1, 0), (2, 0)]));
    assert_eq!(map.objects()[0].visitable, mask(&[(1, 1)]));
}

// covers: ADV-OBJ-001
#[test]
fn an_object_may_stick_out_past_the_right_and_bottom_edges() {
    let mut setup = setup();
    setup.map.objects = vec![
        // Anchored two columns right of and one row below the surface's corner (3, 2): its
        // blocked tile three columns left and two rows up is (2, 1), its visitable tile (3, 2).
        ObjectSetup {
            anchor: Pos::new(5, 3, 0),
            blocked: mask(&[(0, 0), (3, 2), (6, 4), (0, 4)]),
            visitable: mask(&[(2, 1)]),
            kind: ObjectKind::Blocker,
        },
        // Covering no tile of the map at all.
        ObjectSetup {
            anchor: Pos::new(u16::MAX, u16::MAX, 1),
            blocked: mask(&[(0, 0)]),
            visitable: mask(&[(0, 0)]),
            kind: ObjectKind::Blocker,
        },
    ];
    let state = build(&setup).unwrap();
    let map = state.map();
    let marked: Vec<(usize, bool, bool)> = map
        .tiles()
        .iter()
        .enumerate()
        .filter(|(_, t)| t.blocked || t.visitable)
        .map(|(i, t)| (i, t.blocked, t.visitable))
        .collect();
    // (2, 1) is blocked, (3, 2) visitable; (5, 3), (-1, -1) and (5, -1) are off the map.
    assert_eq!(marked, [(6, true, false), (11, false, true)]);
    assert_eq!(map.objects().len(), 2);
}

#[test]
fn keeps_the_heroes_by_id_and_their_stacks_by_slot() {
    let state = build(&setup()).unwrap();
    assert_eq!(state.players(), 2);
    let ids: Vec<u16> = state.heroes().iter().map(|h| h.id.raw()).collect();
    assert_eq!(ids, [1, 5]);
    let five = state.hero(HeroId::new(5)).unwrap();
    assert_eq!(five.owner, PlayerId::new(1));
    assert_eq!(five.pos, Pos::new(2, 2, 0));
    assert_eq!(five.army, [stack(0, IMP, 12), stack(2, GOG, 3)]);
    assert_eq!(state.hero(HeroId::new(1)).unwrap().pos, Pos::new(0, 0, 0));
    assert_eq!(state.hero(HeroId::new(2)), None);
}

// covers: ADV-MAP-001
#[test]
fn a_hero_stands_on_a_free_land_tile_no_object_blocks() {
    let (_, part) = rules();
    let state = build(&setup()).unwrap();
    let stand = |x, y, level| state.check_stand(&part, Pos::new(x, y, level));
    assert_eq!(stand(1, 0, 0), Ok(()));
    assert_eq!(stand(0, 0, 1), Ok(()));
    assert_eq!(stand(4, 0, 0), Err(StandError::Outside));
    assert_eq!(stand(0, 0, 2), Err(StandError::Outside));
    assert_eq!(stand(3, 0, 0), Err(StandError::Terrain));
    assert_eq!(stand(3, 1, 0), Err(StandError::Terrain));
    assert_eq!(stand(0, 1, 0), Err(StandError::Blocked));
    assert_eq!(stand(1, 2, 1), Err(StandError::Blocked));
    // Blocked and visited from, like a town's entrance.
    assert_eq!(stand(2, 2, 1), Ok(()));
    assert_eq!(stand(0, 0, 0), Err(StandError::Occupied));
    assert_eq!(stand(2, 2, 0), Err(StandError::Occupied));
}

#[test]
fn each_broken_invariant_is_its_own_error() {
    let size = |width, height, levels| SetupError::MapSize {
        width,
        height,
        levels,
    };
    assert_eq!(error(|s| s.map.width = 0), size(0, 3, 2));
    assert_eq!(error(|s| s.map.width = 9), size(9, 3, 2));
    assert_eq!(error(|s| s.map.height = 0), size(4, 0, 2));
    assert_eq!(error(|s| s.map.height = 9), size(4, 9, 2));
    assert_eq!(error(|s| s.map.levels = 0), size(4, 3, 0));
    assert_eq!(error(|s| s.map.levels = 3), size(4, 3, 3));
    assert_eq!(error(|s| s.map.levels = 1), SetupError::TileCount(24));
    assert_eq!(
        error(|s| s.map.tiles[5].terrain = TerrainId::from_raw(3)),
        SetupError::UnknownTerrain(5)
    );
    assert_eq!(
        error(|s| s.map.tiles[6].road = Some(RoadId::from_raw(1))),
        SetupError::UnknownRoad(6)
    );
    assert_eq!(
        error(|s| s.map.tiles[7].river = Some(RiverId::from_raw(1))),
        SetupError::UnknownRiver(7)
    );
    assert_eq!(error(|s| s.players = 0), SetupError::Players(0));
    assert_eq!(error(|s| s.players = 3), SetupError::Players(3));
    assert_eq!(
        error(|s| s.map.objects[1].anchor = Pos::new(0, 0, 2)),
        SetupError::ObjectLevel(1)
    );
    assert_eq!(
        error(|s| s.heroes[1].id = HeroId::new(5)),
        SetupError::DuplicateHero(HeroId::new(5))
    );
    assert_eq!(
        error(|s| s.heroes[0].owner = PlayerId::new(2)),
        SetupError::UnknownOwner(HeroId::new(5))
    );
    let tile = |hero, error| SetupError::HeroTile {
        hero: HeroId::new(hero),
        error,
    };
    assert_eq!(
        error(|s| s.heroes[0].pos = Pos::new(0, 0, 2)),
        tile(5, StandError::Outside)
    );
    assert_eq!(
        error(|s| s.heroes[0].pos = Pos::new(3, 0, 0)),
        tile(5, StandError::Terrain)
    );
    assert_eq!(
        error(|s| s.heroes[0].pos = Pos::new(1, 1, 0)),
        tile(5, StandError::Blocked)
    );
    assert_eq!(
        error(|s| s.heroes[1].pos = Pos::new(2, 2, 0)),
        tile(1, StandError::Occupied)
    );
    let army = |error| SetupError::Army {
        hero: HeroId::new(5),
        error,
    };
    assert_eq!(error(|s| s.heroes[0].army.clear()), army(ArmyError::Empty));
    assert_eq!(
        error(|s| s.heroes[0].army[0].slot = 3),
        army(ArmyError::SlotOutOfRange(3))
    );
    assert_eq!(
        error(|s| s.heroes[0].army[0].slot = 0),
        army(ArmyError::DuplicateSlot(0))
    );
    assert_eq!(
        error(|s| s.heroes[0].army[1].count = 0),
        army(ArmyError::EmptyStack(0))
    );
    assert_eq!(
        error(|s| s.heroes[0].army[0].creature = CreatureTypeId::from_raw(2)),
        army(ArmyError::UnknownCreature(2))
    );
}

#[test]
fn the_errors_read_well() {
    let hero = HeroId::new(4);
    let cases = [
        (
            SetupError::MapSize {
                width: 9,
                height: 3,
                levels: 1,
            },
            "a map of 9 × 3 tiles on 1 levels is outside the bounds",
        ),
        (SetupError::TileCount(7), "7 tiles do not fill the map"),
        (
            SetupError::UnknownTerrain(2),
            "tile 2 has an unknown terrain",
        ),
        (SetupError::UnknownRoad(2), "tile 2 has an unknown road"),
        (SetupError::UnknownRiver(2), "tile 2 has an unknown river"),
        (SetupError::Players(9), "9 players are outside the bounds"),
        (
            SetupError::ObjectLevel(3),
            "object 3 is anchored on a missing level",
        ),
        (SetupError::DuplicateHero(hero), "hero 4 is placed twice"),
        (
            SetupError::UnknownOwner(hero),
            "hero 4's owner is not a player",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
    let stands = [
        (StandError::Outside, "the tile is outside the map"),
        (StandError::Terrain, "the terrain is not walked on land"),
        (StandError::Blocked, "an object blocks the tile"),
        (StandError::Occupied, "another hero stands there"),
    ];
    for (error, text) in stands {
        assert_eq!(
            SetupError::HeroTile { hero, error }.to_string(),
            std::format!("hero 4: {text}")
        );
    }
    let armies = [
        (ArmyError::Empty, "the army has no stacks"),
        (ArmyError::SlotOutOfRange(7), "slot 7 is out of range"),
        (ArmyError::DuplicateSlot(1), "slot 1 is used twice"),
        (ArmyError::EmptyStack(1), "slot 1 has no creatures"),
        (
            ArmyError::UnknownCreature(1),
            "slot 1 has an unknown creature type",
        ),
    ];
    for (error, text) in armies {
        assert_eq!(
            SetupError::Army { hero, error }.to_string(),
            std::format!("hero 4's army: {text}")
        );
    }
}

#[test]
fn a_mask_holds_the_eight_by_six_block_only() {
    let full = TileMask::from_bits(u64::MAX);
    assert_eq!(full.bits(), (1 << 48) - 1);
    assert_eq!(full.offsets().count(), 48);
    assert!(full.contains(7, 5));
    assert!(!full.contains(8, 0));
    assert!(!full.contains(0, 6));
    assert_eq!(TileMask::default().offsets().count(), 0);
    let m = mask(&[(3, 1), (0, 2), (5, 0)]);
    assert_eq!(m.offsets().collect::<Vec<_>>(), [(5, 0), (3, 1), (0, 2)]);
}

#[test]
fn the_hash_sees_content_not_sharing() {
    let base = build(&setup()).unwrap();
    let fork = base.clone();
    let rebuilt = build(&setup()).unwrap();
    assert_eq!(fork, base);
    assert_eq!(rebuilt, base);
    assert_eq!(fork.state_hash(), base.state_hash());
    assert_eq!(rebuilt.state_hash(), base.state_hash());
    assert_eq!(base.state_hash(), hash_of(&base));
    // The heroes' and stacks' order in the setup does not matter.
    let mut reordered = setup();
    reordered.heroes.reverse();
    reordered.heroes[1].army.reverse();
    assert_eq!(build(&reordered).unwrap().state_hash(), base.state_hash());

    let changes: Vec<fn(&mut AdventureSetup)> = vec![
        |s| s.map.tiles[2].terrain = ROCK,
        |s| s.map.tiles[2].road = Some(RoadId::from_raw(0)),
        |s| s.map.tiles[2].river = Some(RiverId::from_raw(0)),
        |s| s.map.objects[1].anchor = Pos::new(1, 2, 1),
        |s| s.map.objects[1].blocked = mask(&[(1, 0)]),
        |s| s.map.objects[1].blocked = mask(&[(0, 0)]),
        |s| s.map.objects[1].visitable = TileMask::default(),
        |s| s.map.objects.truncate(1),
        |s| {
            s.map.width = 3;
            s.map.height = 4;
        },
        |s| {
            s.map.levels = 1;
            s.map.tiles.truncate(12);
            s.map.objects.truncate(1);
        },
        |s| s.heroes[1].id = HeroId::new(2),
        |s| s.heroes[0].owner = PlayerId::new(0),
        |s| s.heroes[0].pos = Pos::new(2, 1, 0),
        |s| s.heroes[0].army[0].slot = 1,
        |s| s.heroes[0].army[0].creature = IMP,
        |s| s.heroes[0].army[0].count = 4,
        |s| s.heroes.truncate(1),
    ];
    for (i, change) in changes.iter().enumerate() {
        let mut other = setup();
        change(&mut other);
        let other = build(&other).unwrap();
        assert_ne!(other.state_hash(), base.state_hash(), "change {i}");
        assert_ne!(other, base, "change {i}");
    }
    let mut more = setup();
    more.players = 1;
    more.heroes[0].owner = PlayerId::new(0);
    let one = build(&more).unwrap();
    more.players = 2;
    assert_ne!(build(&more).unwrap().state_hash(), one.state_hash());
}
