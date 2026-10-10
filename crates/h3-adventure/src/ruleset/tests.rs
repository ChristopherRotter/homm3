//! Tests of the adventure ruleset part's compiler (E08 T4), on core rulesets built in code
//! (`crate::testing`).

extern crate std;

use std::format;
use std::string::ToString;
use std::vec;
use std::vec::Vec;

use core::num::{NonZeroU8, NonZeroU16};

use h3_content::{ContentPack, ProfileDef, Ruleset, TerrainDef};
use h3_kernel::hash::hash_of;

use super::*;
use crate::testing::{battle, key, manifest};

fn cost(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).unwrap()
}

fn terrain(k: &str) -> TerrainDef {
    TerrainDef {
        key: key(k),
        name: "Terrain".into(),
    }
}

/// Core packs: `t` (three terrains, the battle parameters and the profile), `u` (depends on `t`,
/// one terrain) and `v` (no dependencies, no terrains).
fn core() -> Ruleset {
    let t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        battle: Some(battle()),
        ..ContentPack::new(manifest("t", &[]))
    };
    let u = ContentPack {
        terrains: vec![terrain("u:swamp")],
        ..ContentPack::new(manifest("u", &["t"]))
    };
    let v = ContentPack::new(manifest("v", &[]));
    Ruleset::compile(&[t, u, v], "t:faithful").unwrap()
}

fn params() -> AdventureParamsDef {
    AdventureParamsDef {
        native_cost: cost(100),
        land_points: vec![1300, 1360, 1430, 1500],
        scouting_radius: 5,
        max_map_size: NonZeroU16::new(144).unwrap(),
        max_levels: NonZeroU8::new(2).unwrap(),
        max_players: NonZeroU8::new(8).unwrap(),
        army_slots: NonZeroU8::new(7).unwrap(),
    }
}

fn movement(k: &str, passage: Passage) -> TerrainMovementDef {
    TerrainMovementDef {
        terrain: key(k),
        passage,
    }
}

fn road(k: &str, c: u16) -> RoadDef {
    RoadDef {
        key: key(k),
        name: "Road".into(),
        cost: cost(c),
    }
}

fn river(k: &str) -> RiverDef {
    RiverDef {
        key: key(k),
        name: "River".into(),
    }
}

/// Adventure packs for [`core`]: `t` gives its terrains, a road, a river and the parameters;
/// `u` its terrain and a second road; `v` is empty.
fn packs() -> Vec<AdventurePack> {
    let t = AdventurePack {
        terrains: vec![
            movement("t:grass", Passage::Land(cost(100))),
            movement("t:sea", Passage::Water(cost(100))),
            movement("t:rock", Passage::Impassable),
        ],
        roads: vec![road("t:dirt_road", 75)],
        rivers: vec![river("t:clear_river")],
        params: Some(params()),
        ..AdventurePack::new(manifest("t", &[]))
    };
    let u = AdventurePack {
        terrains: vec![movement("u:swamp", Passage::Land(cost(175)))],
        roads: vec![road("u:cobblestone_road", 50)],
        ..AdventurePack::new(manifest("u", &["t"]))
    };
    let v = AdventurePack::new(manifest("v", &[]));
    vec![v, u, t]
}

fn compile(packs: &[AdventurePack]) -> Result<AdventureRuleset, AdventureCompileError> {
    AdventureRuleset::compile(&core(), packs)
}

fn compile_err(packs: &[AdventurePack]) -> AdventureCompileError {
    compile(packs).unwrap_err()
}

fn edit(f: impl FnOnce(&mut Vec<AdventurePack>)) -> Vec<AdventurePack> {
    let mut packs = packs();
    f(&mut packs);
    packs
}

/// The pack with this id in `packs`.
fn pack<'a>(packs: &'a mut [AdventurePack], id: &str) -> &'a mut AdventurePack {
    packs.iter_mut().find(|p| p.manifest.id == id).unwrap()
}

// covers: ADV-MOV-001, ADV-MOV-003, ADV-MOV-004, ADV-MOV-007, ADV-FOG-001
#[test]
fn compiles_terrains_roads_rivers_and_parameters() {
    let core = core();
    let part = AdventureRuleset::compile(&core, &packs()).unwrap();
    assert_eq!(part.core_hash(), core.content_hash());
    let passage = |k: &str| part.passage(core.terrain_by_key(k).unwrap()).unwrap();
    assert_eq!(passage("t:grass"), Passage::Land(cost(100)));
    assert_eq!(passage("t:sea"), Passage::Water(cost(100)));
    assert_eq!(passage("t:rock"), Passage::Impassable);
    assert_eq!(passage("u:swamp"), Passage::Land(cost(175)));
    let keys: Vec<&str> = part.terrains().iter().map(|t| t.key.as_str()).collect();
    assert_eq!(keys, ["t:grass", "t:sea", "t:rock", "u:swamp"]);
    assert_eq!(part.passage(TerrainId::from_raw(4)), None);

    // Roads in load order: `t` before `u`.
    let dirt = part.road_by_key("t:dirt_road").unwrap();
    let cobble = part.road_by_key("u:cobblestone_road").unwrap();
    assert_eq!((dirt.raw(), cobble.raw()), (0, 1));
    assert_eq!(part.road(cobble).unwrap().cost, cost(50));
    assert_eq!(part.roads().len(), 2);
    assert_eq!(part.road(RoadId::from_raw(2)), None);
    assert_eq!(part.road_by_key("t:gravel_road"), None);
    let clear = part.river_by_key("t:clear_river").unwrap();
    assert_eq!(part.river(clear).unwrap().name, "River");
    assert_eq!(part.rivers().len(), 1);
    assert_eq!(part.river(RiverId::from_raw(1)), None);
    assert_eq!(part.river_by_key("t:icy_river"), None);

    let params = part.params();
    assert_eq!(params.native_cost(), cost(100));
    assert_eq!(params.scouting_radius(), 5);
    assert_eq!(
        (
            params.max_map_size(),
            params.max_levels(),
            params.max_players(),
            params.army_slots()
        ),
        (144, 2, 8, 7)
    );
    assert_eq!(params.land_points_table(), [1300, 1360, 1430, 1500]);
    assert_eq!(params.land_points(0), 1300);
    assert_eq!(params.land_points(3), 1500);
    // Faster armies take the last entry.
    assert_eq!(params.land_points(4), 1500);
    assert_eq!(params.land_points(u8::MAX), 1500);
}

#[test]
fn the_hash_is_stable_and_sees_every_part() {
    let base = compile(&packs()).unwrap();
    assert_eq!(
        base.content_hash(),
        compile(&packs()).unwrap().content_hash()
    );
    assert_eq!(base.content_hash(), hash_of(&base));
    let changes: Vec<Vec<AdventurePack>> = vec![
        edit(|p| pack(p, "t").terrains[1].passage = Passage::Land(cost(100))),
        edit(|p| pack(p, "t").terrains[1].passage = Passage::Water(cost(101))),
        edit(|p| pack(p, "t").terrains[2].passage = Passage::Land(cost(100))),
        edit(|p| pack(p, "t").roads[0].cost = cost(76)),
        edit(|p| pack(p, "t").roads[0].name = "Other".into()),
        edit(|p| pack(p, "t").rivers[0].name = "Other".into()),
        edit(|p| pack(p, "t").rivers.clear()),
        edit(|p| pack(p, "t").params.as_mut().unwrap().native_cost = cost(99)),
        edit(|p| pack(p, "t").params.as_mut().unwrap().land_points.push(1560)),
        edit(|p| pack(p, "t").params.as_mut().unwrap().scouting_radius = 6),
        edit(|p| pack(p, "t").params.as_mut().unwrap().max_map_size = cost(108)),
        edit(|p| pack(p, "t").params.as_mut().unwrap().max_levels = NonZeroU8::MIN),
        edit(|p| pack(p, "t").params.as_mut().unwrap().max_players = NonZeroU8::MIN),
        edit(|p| pack(p, "t").params.as_mut().unwrap().army_slots = NonZeroU8::MIN),
    ];
    for (i, packs) in changes.iter().enumerate() {
        let other = compile(packs).unwrap();
        assert_ne!(other.content_hash(), base.content_hash(), "change {i}");
    }
}

#[test]
fn the_packs_must_match_the_core() {
    assert_eq!(
        compile_err(&edit(|p| p.retain(|p| p.manifest.id != "u"))),
        AdventureCompileError::MissingPack("u".into())
    );
    assert_eq!(
        compile_err(&edit(|p| p.push(AdventurePack::new(manifest("w", &[]))))),
        AdventureCompileError::UnknownPack("w@1".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "v").manifest.version = "2".into())),
        AdventureCompileError::UnknownPack("v@2".into())
    );
    assert_eq!(
        compile_err(&edit(|p| p.push(AdventurePack::new(manifest("v", &[]))))),
        AdventureCompileError::DuplicatePack("v".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u").manifest.depends.push("v".into()))),
        AdventureCompileError::UnknownDependency {
            pack: "u".into(),
            dependency: "v".into()
        }
    );
}

#[test]
fn keys_stay_in_their_pack_and_are_unique() {
    let outside = |key: &str| AdventureCompileError::KeyOutsidePack {
        key: key.into(),
        pack: "u".into(),
    };
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u")
            .roads
            .push(road("t:gravel_road", 65)))),
        outside("t:gravel_road")
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u").rivers.push(river("t:icy_river")))),
        outside("t:icy_river")
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u")
            .roads
            .push(road("u:cobblestone_road", 50)))),
        AdventureCompileError::DuplicateKey("u:cobblestone_road".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "t").rivers.push(river("t:clear_river")))),
        AdventureCompileError::DuplicateKey("t:clear_river".into())
    );
}

#[test]
fn handles_run_out_after_u16_max_entries() {
    let roads = edit(|p| {
        let v = pack(p, "v");
        v.roads = (0..=usize::from(u16::MAX) + 1)
            .map(|i| road(&format!("v:r{i}"), 1))
            .collect();
    });
    assert_eq!(
        compile_err(&roads),
        AdventureCompileError::TooManyEntries("road")
    );
    let rivers = edit(|p| {
        let v = pack(p, "v");
        v.rivers = (0..=usize::from(u16::MAX) + 1)
            .map(|i| river(&format!("v:r{i}")))
            .collect();
    });
    assert_eq!(
        compile_err(&rivers),
        AdventureCompileError::TooManyEntries("river")
    );
}

// covers: ADV-MOV-007
#[test]
fn every_core_terrain_has_movement_data_once() {
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u").terrains.clear())),
        AdventureCompileError::MissingTerrain("u:swamp".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u")
            .terrains
            .push(movement("t:rock", Passage::Impassable)))),
        AdventureCompileError::DuplicateTerrain("t:rock".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u")
            .terrains
            .push(movement("t:lava", Passage::Impassable)))),
        AdventureCompileError::UnknownTerrain("t:lava".into())
    );
    // `v` does not depend on `t`.
    assert_eq!(
        compile_err(&edit(|p| pack(p, "v")
            .terrains
            .push(movement("t:grass", Passage::Impassable)))),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "t:grass".into(),
            pack: "v".into()
        }
    );
}

#[test]
fn one_pack_provides_valid_parameters() {
    assert_eq!(
        compile_err(&edit(|p| pack(p, "t").params = None)),
        AdventureCompileError::MissingParams
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u").params = Some(params()))),
        AdventureCompileError::DuplicateParams("u".into())
    );
    let invalid = |points: Vec<u32>| {
        compile_err(&edit(|p| {
            pack(p, "t").params.as_mut().unwrap().land_points = points
        }))
    };
    assert_eq!(
        invalid(vec![]),
        AdventureCompileError::InvalidParams("land_points is empty")
    );
    assert_eq!(
        invalid(vec![1500; MAX_LAND_POINTS + 1]),
        AdventureCompileError::InvalidParams("land_points has more entries than speeds")
    );
    assert_eq!(
        invalid(vec![1500, 0]),
        AdventureCompileError::InvalidParams("a land_points entry is 0")
    );
    // A full table compiles.
    let full = edit(|p| {
        pack(p, "t").params.as_mut().unwrap().land_points = vec![1500; MAX_LAND_POINTS];
    });
    assert_eq!(compile(&full).unwrap().params().land_points(u8::MAX), 1500);
}

#[test]
fn errors_explain_themselves() {
    let cases: Vec<(AdventureCompileError, &str)> = vec![
        (
            AdventureCompileError::MissingPack("u".into()),
            "pack `u` has no adventure part",
        ),
        (
            AdventureCompileError::UnknownPack("w@1".into()),
            "adventure part of `w@1`, which the core lacks",
        ),
        (
            AdventureCompileError::DuplicatePack("v".into()),
            "pack `v` has two adventure parts",
        ),
        (
            AdventureCompileError::UnknownDependency {
                pack: "u".into(),
                dependency: "v".into(),
            },
            "pack `u` depends on `v`, which is not loaded",
        ),
        (
            AdventureCompileError::KeyOutsidePack {
                key: "t:x".into(),
                pack: "u".into(),
            },
            "pack `u` defines `t:x` outside its namespace",
        ),
        (
            AdventureCompileError::DuplicateKey("t:x".into()),
            "duplicate key `t:x`",
        ),
        (
            AdventureCompileError::ReferenceOutsideDependencies {
                key: "t:x".into(),
                pack: "v".into(),
            },
            "pack `v` references `t:x` outside its dependencies",
        ),
        (
            AdventureCompileError::UnknownTerrain("t:x".into()),
            "unknown terrain `t:x`",
        ),
        (
            AdventureCompileError::DuplicateTerrain("t:x".into()),
            "terrain `t:x` has movement data twice",
        ),
        (
            AdventureCompileError::MissingTerrain("t:x".into()),
            "terrain `t:x` has no movement data",
        ),
        (
            AdventureCompileError::MissingParams,
            "no pack provides the adventure parameters",
        ),
        (
            AdventureCompileError::DuplicateParams("u".into()),
            "pack `u` provides the adventure parameters a second time",
        ),
        (
            AdventureCompileError::InvalidParams("land_points is empty"),
            "adventure parameters: land_points is empty",
        ),
        (
            AdventureCompileError::TooManyEntries("road"),
            "too many road entries",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
}
