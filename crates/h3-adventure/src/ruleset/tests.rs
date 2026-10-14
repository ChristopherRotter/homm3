//! Tests of the adventure ruleset part's compiler (E08 T4), on core rulesets built in code
//! (`crate::testing`).

extern crate std;

use std::format;
use std::string::ToString;
use std::vec;
use std::vec::Vec;

use core::num::{NonZeroU8, NonZeroU16, NonZeroU32};

use h3_content::{ContentPack, ProfileDef, ResourceDef, ResourceId, Ruleset, TerrainDef};
use h3_kernel::hash::hash_of;

use super::*;
use crate::testing::{key, manifest};

fn cost(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).unwrap()
}

fn terrain(k: &str) -> TerrainDef {
    TerrainDef {
        key: key(k),
        name: "Terrain".into(),
    }
}

fn resource(k: &str) -> ResourceDef {
    ResourceDef {
        key: key(k),
        name: "Resource".into(),
    }
}

/// Core packs: `t` (three terrains, the resources `t:wood` and `t:gold` and the profile), `u` (depends on `t`, one terrain, the resource `u:gem`) and `v` (no
/// dependencies, no terrains).
fn core() -> Ruleset {
    let t = ContentPack {
        terrains: vec![terrain("t:grass"), terrain("t:sea"), terrain("t:rock")],
        resources: vec![resource("t:wood"), resource("t:gold")],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(manifest("t", &[]))
    };
    let u = ContentPack {
        terrains: vec![terrain("u:swamp")],
        resources: vec![resource("u:gem")],
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
        starting_stack_chances: vec![88, 25],
        battle: None,
    }
}

/// Two difficulties: 0 with gold and wood for a human, gold for a computer; 1 with gold only.
fn starting() -> Vec<StartingResourcesDef> {
    vec![
        StartingResourcesDef {
            human: vec![(key("t:gold"), 3000), (key("t:wood"), 5)],
            computer: vec![(key("t:gold"), 500)],
        },
        StartingResourcesDef {
            human: vec![(key("t:gold"), 2000)],
            computer: vec![],
        },
    ]
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

fn object(k: &str, visit: Visit, from_above: bool, kind: ObjectKindDef) -> ObjectTypeDef {
    ObjectTypeDef {
        key: key(k),
        name: "Object".into(),
        visit,
        from_above,
        kind,
    }
}

fn n(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

fn pile(k: &str, resource: &str, amount: [u32; 2], multiplier: u32) -> ObjectTypeDef {
    let kind = ObjectKindDef::Pickup(PickupDef::Resource {
        resource: key(resource),
        amount,
        multiplier: n(multiplier),
    });
    object(k, Visit::Neighbour, true, kind)
}

/// `t`'s object types: two piles, a random pile naming a later type of its own pack first, a
/// paired pickup, a terrain reveal, an area reveal and a key tent, guard and gate.
fn objects() -> Vec<ObjectTypeDef> {
    let keys = |list: &[&str]| list.iter().map(|k| key(k)).collect::<Vec<_>>();
    vec![
        object(
            "t:any_pile",
            Visit::Neighbour,
            true,
            ObjectKindDef::Pickup(PickupDef::OneOf(keys(&["t:gold_pile", "t:wood_pile"]))),
        ),
        pile("t:wood_pile", "t:wood", [5, 10], 1),
        pile("t:gold_pile", "t:gold", [5, 10], 100),
        object(
            "t:fire",
            Visit::Neighbour,
            true,
            ObjectKindDef::Pickup(PickupDef::Paired {
                amount: [4, 6],
                resources: keys(&["t:wood"]),
                paired: key("t:gold"),
                multiplier: n(100),
            }),
        ),
        object(
            "t:mapper",
            Visit::Enter,
            false,
            ObjectKindDef::RevealTerrain {
                terrains: keys(&["t:sea", "t:grass"]),
                price: vec![(key("t:gold"), 1000), (key("t:wood"), 2)],
            },
        ),
        object(
            "t:tower",
            Visit::Enter,
            false,
            ObjectKindDef::RevealArea { radius: 20 },
        ),
        object(
            "t:tent",
            Visit::Enter,
            false,
            ObjectKindDef::KeyTent { key: 0 },
        ),
        object(
            "t:guard",
            Visit::Neighbour,
            true,
            ObjectKindDef::KeyGuard { key: 1 },
        ),
        object(
            "t:gate",
            Visit::Neighbour,
            true,
            ObjectKindDef::KeyGate { key: 7 },
        ),
    ]
}

/// Adventure packs for [`core`]: `t` gives its terrains, a road, a river, the parameters and
/// [`objects`]; `u` its terrain, a second road and a gem pile; `v` is empty.
fn packs() -> Vec<AdventurePack> {
    let t = AdventurePack {
        terrains: vec![
            movement("t:grass", Passage::Land(cost(100))),
            movement("t:sea", Passage::Water(cost(100))),
            movement("t:rock", Passage::Impassable),
        ],
        roads: vec![road("t:dirt_road", 75)],
        rivers: vec![river("t:clear_river")],
        object_types: objects(),
        params: Some(params()),
        starting_resources: starting(),
        ..AdventurePack::new(manifest("t", &[]))
    };
    let u = AdventurePack {
        terrains: vec![movement("u:swamp", Passage::Land(cost(175)))],
        roads: vec![road("u:cobblestone_road", 50)],
        object_types: vec![pile("u:gem_pile", "u:gem", [3, 6], 1)],
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
        edit(|p| pack(p, "t").object_types[1].name = "Other".into()),
        edit(|p| pack(p, "t").object_types[1].visit = Visit::Enter),
        edit(|p| pack(p, "t").object_types[1].from_above = false),
        edit(|p| pack(p, "t").object_types.swap(1, 2)),
        edit(|p| pack(p, "u").object_types.clear()),
        edit(|p| pack(p, "t").object_types[2] = pile("t:gold_pile", "t:gold", [5, 10], 10)),
        edit(|p| pack(p, "t").object_types[2] = pile("t:gold_pile", "t:gold", [5, 9], 100)),
        edit(|p| pack(p, "t").object_types[2] = pile("t:gold_pile", "t:gold", [4, 10], 100)),
        edit(|p| pack(p, "t").object_types[2] = pile("t:gold_pile", "t:wood", [5, 10], 100)),
        edit(|p| {
            set_kind(
                p,
                0,
                ObjectKindDef::Pickup(PickupDef::OneOf(vec![key("t:gold_pile")])),
            )
        }),
        edit(|p| set_kind(p, 3, paired([4, 5], &["t:wood"], "t:gold", 100))),
        edit(|p| set_kind(p, 3, paired([3, 6], &["t:wood"], "t:gold", 100))),
        edit(|p| set_kind(p, 3, paired([4, 6], &["t:gold"], "t:gold", 100))),
        edit(|p| set_kind(p, 3, paired([4, 6], &["t:wood"], "t:wood", 100))),
        edit(|p| set_kind(p, 3, paired([4, 6], &["t:wood"], "t:gold", 10))),
        edit(|p| set_kind(p, 4, reveal(&["t:sea"], &[("t:gold", 1000), ("t:wood", 2)]))),
        edit(|p| set_kind(p, 4, reveal(&["t:sea", "t:grass"], &[("t:gold", 1000)]))),
        edit(|p| {
            set_kind(
                p,
                4,
                reveal(&["t:sea", "t:grass"], &[("t:gold", 999), ("t:wood", 2)]),
            )
        }),
        edit(|p| set_kind(p, 5, ObjectKindDef::RevealArea { radius: 19 })),
        edit(|p| set_kind(p, 6, ObjectKindDef::KeyTent { key: 1 })),
        edit(|p| set_kind(p, 7, ObjectKindDef::KeyGuard { key: 2 })),
        edit(|p| set_kind(p, 8, ObjectKindDef::KeyGate { key: 6 })),
        // A guard (visited from a neighbour) becomes a tent of its key: only the kind differs.
        edit(|p| set_kind(p, 7, ObjectKindDef::KeyTent { key: 1 })),
        edit(|p| set_kind(p, 7, ObjectKindDef::KeyGate { key: 1 })),
        edit(|p| set_kind(p, 8, ObjectKindDef::KeyTent { key: 7 })),
        edit(|p| set_kind(p, 5, ObjectKindDef::KeyTent { key: 4 })),
        edit(|p| pack(p, "t").starting_resources[0].human[0].1 = 2999),
        edit(|p| pack(p, "t").starting_resources[0].computer[0].1 = 501),
        edit(|p| pack(p, "t").starting_resources.swap(0, 1)),
        edit(|p| pack(p, "t").starting_resources.truncate(1)),
    ];
    for (i, packs) in changes.iter().enumerate() {
        let other = compile(packs).unwrap();
        assert_ne!(other.content_hash(), base.content_hash(), "change {i}");
    }
}

fn set_kind(packs: &mut [AdventurePack], index: usize, kind: ObjectKindDef) {
    pack(packs, "t").object_types[index].kind = kind;
}

fn paired(amount: [u32; 2], resources: &[&str], paired: &str, multiplier: u32) -> ObjectKindDef {
    ObjectKindDef::Pickup(PickupDef::Paired {
        amount,
        resources: resources.iter().map(|k| key(k)).collect(),
        paired: key(paired),
        multiplier: n(multiplier),
    })
}

fn reveal(terrains: &[&str], price: &[(&str, u32)]) -> ObjectKindDef {
    ObjectKindDef::RevealTerrain {
        terrains: terrains.iter().map(|k| key(k)).collect(),
        price: price.iter().map(|&(k, a)| (key(k), a)).collect(),
    }
}

// covers: ADV-OBJ-002
#[test]
fn compiles_object_types() {
    let core = core();
    let part = AdventureRuleset::compile(&core, &packs()).unwrap();
    let id = |k: &str| part.object_type_by_key(k).unwrap();
    let ty = |k: &str| part.object_type(id(k)).unwrap();
    let keys: Vec<&str> = part.object_types().iter().map(|t| t.key.as_str()).collect();
    // `t` before `u` (the core's load order), each pack's types in its order.
    assert_eq!(
        keys,
        [
            "t:any_pile",
            "t:wood_pile",
            "t:gold_pile",
            "t:fire",
            "t:mapper",
            "t:tower",
            "t:tent",
            "t:guard",
            "t:gate",
            "u:gem_pile"
        ]
    );
    assert_eq!(id("u:gem_pile").raw(), 9);
    assert_eq!(part.object_type(ObjectTypeId::from_raw(10)), None);
    assert_eq!(part.object_type_by_key("t:none"), None);
    let resource = |k: &str| {
        let index = core.resources().iter().position(|r| r.key.as_str() == k);
        ResourceId::from_index(index.unwrap()).unwrap()
    };
    let terrain = |k: &str| core.terrain_by_key(k).unwrap();
    assert_eq!(
        ty("t:any_pile").kind,
        ObjectKind::Pickup(Pickup::OneOf(vec![id("t:gold_pile"), id("t:wood_pile")]))
    );
    assert_eq!(
        ty("t:gold_pile").kind,
        ObjectKind::Pickup(Pickup::Resource {
            resource: resource("t:gold"),
            amount: [5, 10],
            multiplier: n(100),
        })
    );
    assert_eq!(
        (ty("t:gold_pile").visit, ty("t:gold_pile").from_above),
        (Visit::Neighbour, true)
    );
    assert_eq!(
        ty("u:gem_pile").kind,
        ObjectKind::Pickup(Pickup::Resource {
            resource: resource("u:gem"),
            amount: [3, 6],
            multiplier: n(1),
        })
    );
    assert_eq!(
        ty("t:fire").kind,
        ObjectKind::Pickup(Pickup::Paired {
            amount: [4, 6],
            resources: vec![resource("t:wood")],
            paired: resource("t:gold"),
            multiplier: n(100),
        })
    );
    // Terrains and prices in the core's order.
    assert_eq!(
        ty("t:mapper").kind,
        ObjectKind::RevealTerrain {
            terrains: vec![terrain("t:grass"), terrain("t:sea")],
            price: vec![(resource("t:wood"), 2), (resource("t:gold"), 1000)],
        }
    );
    assert_eq!(
        (ty("t:mapper").visit, ty("t:mapper").from_above),
        (Visit::Enter, false)
    );
    assert_eq!(ty("t:tower").kind, ObjectKind::RevealArea { radius: 20 });
    assert_eq!(ty("t:tent").kind, ObjectKind::KeyTent { key: 0 });
    assert_eq!(ty("t:guard").kind, ObjectKind::KeyGuard { key: 1 });
    assert_eq!(ty("t:gate").kind, ObjectKind::KeyGate { key: 7 });
    let takes: Vec<bool> = part
        .object_types()
        .iter()
        .map(ObjectType::takes_amount)
        .collect();
    assert_eq!(
        takes,
        [
            true, true, true, false, false, false, false, false, false, true
        ]
    );
}

// covers: ADV-OBJ-002
#[test]
fn object_types_reference_known_content() {
    let outside = |key: &str, pack: &str| AdventureCompileError::ReferenceOutsideDependencies {
        key: key.into(),
        pack: pack.into(),
    };
    assert_eq!(
        compile_err(&edit(|p| pack(p, "t").object_types.push(pile(
            "t:p",
            "t:iron",
            [1, 1],
            1
        )))),
        AdventureCompileError::UnknownResource("t:iron".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "v").object_types.push(pile(
            "v:p",
            "t:wood",
            [1, 1],
            1
        )))),
        outside("t:wood", "v")
    );
    assert_eq!(
        compile_err(&edit(|p| set_kind(
            p,
            0,
            ObjectKindDef::Pickup(PickupDef::OneOf(vec![key("t:none")]))
        ))),
        AdventureCompileError::UnknownObjectType("t:none".into())
    );
    // A type of a later pack is out of reach even where its key exists.
    assert_eq!(
        compile_err(&edit(|p| set_kind(
            p,
            0,
            ObjectKindDef::Pickup(PickupDef::OneOf(vec![key("u:gem_pile")]))
        ))),
        outside("u:gem_pile", "t")
    );
    assert_eq!(
        compile_err(&edit(|p| set_kind(
            p,
            3,
            paired([4, 6], &["t:iron"], "t:gold", 1)
        ))),
        AdventureCompileError::UnknownResource("t:iron".into())
    );
    assert_eq!(
        compile_err(&edit(|p| set_kind(
            p,
            3,
            paired([4, 6], &["t:wood"], "u:gem", 1)
        ))),
        outside("u:gem", "t")
    );
    assert_eq!(
        compile_err(&edit(|p| set_kind(p, 4, reveal(&["t:lava"], &[])))),
        AdventureCompileError::UnknownTerrain("t:lava".into())
    );
    assert_eq!(
        compile_err(&edit(|p| set_kind(p, 4, reveal(&["u:swamp"], &[])))),
        outside("u:swamp", "t")
    );
    assert_eq!(
        compile_err(&edit(|p| set_kind(
            p,
            4,
            reveal(&["t:sea"], &[("t:iron", 1)])
        ))),
        AdventureCompileError::UnknownResource("t:iron".into())
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u").object_types.push(pile(
            "t:p",
            "t:wood",
            [1, 1],
            1
        )))),
        AdventureCompileError::KeyOutsidePack {
            key: "t:p".into(),
            pack: "u".into()
        }
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "t").object_types.push(pile(
            "t:fire",
            "t:wood",
            [1, 1],
            1
        )))),
        AdventureCompileError::DuplicateKey("t:fire".into())
    );
}

// covers: ADV-OBJ-002
#[test]
fn object_type_parameters_are_checked() {
    let invalid = |index: usize, kind: ObjectKindDef| {
        let error = compile_err(&edit(|p| set_kind(p, index, kind)));
        let AdventureCompileError::InvalidObjectType { key, reason } = error else {
            panic!("{error:?}");
        };
        let at = packs()[2].object_types[index].key.as_str().to_string();
        assert_eq!(key, at);
        reason
    };
    let resource = |amount, multiplier| {
        ObjectKindDef::Pickup(PickupDef::Resource {
            resource: key("t:wood"),
            amount,
            multiplier: n(multiplier),
        })
    };
    let bounds = "amount is not 1 <= low <= high <= MAX_PICKUP_AMOUNT";
    for amount in [[0, 1], [3, 2], [1, MAX_PICKUP_AMOUNT + 1]] {
        assert_eq!(invalid(1, resource(amount, 1)), bounds);
        assert_eq!(invalid(3, paired(amount, &["t:wood"], "t:gold", 1)), bounds);
    }
    let gain = "MAX_PICKUP_AMOUNT times the multiplier is above MAX_GAIN";
    assert_eq!(invalid(1, resource([1, 1], 4097)), gain);
    assert_eq!(invalid(1, resource([1, 1], u32::MAX)), gain);
    assert_eq!(
        invalid(3, paired([1, 1], &["t:wood"], "t:gold", 4097)),
        gain
    );
    // The largest multiplier: MAX_PICKUP_AMOUNT × 4096 = 2 147 479 552.
    compile(&edit(|p| {
        set_kind(p, 1, resource([1, MAX_PICKUP_AMOUNT], 4096))
    }))
    .unwrap();
    let one_of = |list: &[&str]| {
        ObjectKindDef::Pickup(PickupDef::OneOf(list.iter().map(|k| key(k)).collect()))
    };
    assert_eq!(invalid(0, one_of(&[])), "one_of is empty");
    assert_eq!(
        invalid(0, one_of(&["t:wood_pile", "t:wood_pile"])),
        "one_of names a type twice"
    );
    for not_a_pile in ["t:any_pile", "t:fire", "t:gate"] {
        assert_eq!(
            invalid(0, one_of(&["t:wood_pile", not_a_pile])),
            "one_of names a type that is not a resource pickup"
        );
    }
    assert_eq!(
        invalid(3, paired([4, 6], &[], "t:gold", 1)),
        "resources is empty"
    );
    assert_eq!(
        invalid(3, paired([4, 6], &["t:wood", "t:wood"], "t:gold", 1)),
        "resources names a resource twice"
    );
    assert_eq!(invalid(4, reveal(&[], &[])), "terrains is empty");
    assert_eq!(
        invalid(4, reveal(&["t:sea", "t:sea"], &[])),
        "terrains names a terrain twice"
    );
    assert_eq!(
        invalid(4, reveal(&["t:sea"], &[("t:gold", 1), ("t:gold", 2)])),
        "price names a resource twice"
    );
    for amount in [0, MAX_GAIN + 1] {
        assert_eq!(
            invalid(4, reveal(&["t:sea"], &[("t:gold", amount)])),
            "a price is 0 or above MAX_GAIN"
        );
    }
    // A free reveal and the largest price are fine.
    compile(&edit(|p| set_kind(p, 4, reveal(&["t:sea"], &[])))).unwrap();
    compile(&edit(|p| {
        set_kind(p, 4, reveal(&["t:sea"], &[("t:gold", MAX_GAIN)]))
    }))
    .unwrap();
    assert_eq!(
        invalid(5, ObjectKindDef::RevealArea { radius: 0 }),
        "radius is 0"
    );
    let key_reason = "key is not below KEY_COLOURS";
    assert_eq!(
        invalid(6, ObjectKindDef::KeyTent { key: KEY_COLOURS }),
        key_reason
    );
    assert_eq!(
        invalid(7, ObjectKindDef::KeyGuard { key: KEY_COLOURS }),
        key_reason
    );
    assert_eq!(
        invalid(8, ObjectKindDef::KeyGate { key: u8::MAX }),
        key_reason
    );
    // A gate is crossed, so it is visited from above (E08 T15b); a guard need not be.
    let error = compile_err(&edit(|p| pack(p, "t").object_types[8].from_above = false));
    assert_eq!(
        error,
        AdventureCompileError::InvalidObjectType {
            key: "t:gate".into(),
            reason: "a key gate is not visited from above",
        }
    );
    compile(&edit(|p| pack(p, "t").object_types[7].from_above = false)).unwrap();
    // Guards and gates block the players without their key, so they are never entered (the
    // review of E08 T15).
    for (index, key) in [(7, "t:guard"), (8, "t:gate")] {
        let error = compile_err(&edit(|p| {
            pack(p, "t").object_types[index].visit = Visit::Enter
        }));
        assert_eq!(
            error,
            AdventureCompileError::InvalidObjectType {
                key: key.into(),
                reason: "a key guard or gate is not visited from a neighbour",
            }
        );
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
    let objects = edit(|p| {
        let v = pack(p, "v");
        v.object_types = (0..=usize::from(u16::MAX) + 1)
            .map(|i| {
                object(
                    &format!("v:o{i}"),
                    Visit::Enter,
                    false,
                    ObjectKindDef::RevealArea { radius: 1 },
                )
            })
            .collect();
    });
    assert_eq!(
        compile_err(&objects),
        AdventureCompileError::TooManyEntries("object_type")
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
    let chances = |chances: Vec<u8>| {
        AdventureRuleset::compile(
            &core(),
            &edit(|p| pack(p, "t").params.as_mut().unwrap().starting_stack_chances = chances),
        )
        .map(|part| part.params().starting_stack_chances().to_vec())
    };
    assert_eq!(
        chances(vec![88, 101]),
        Err(AdventureCompileError::InvalidParams(
            "a starting_stack_chances entry is above 100"
        ))
    );
    let players = |players: u8| {
        AdventureRuleset::compile(
            &core(),
            &edit(|p| {
                pack(p, "t").params.as_mut().unwrap().max_players = NonZeroU8::new(players).unwrap()
            }),
        )
        .map(|part| part.params().max_players())
    };
    assert_eq!(
        players(MAX_PLAYERS + 1),
        Err(AdventureCompileError::InvalidParams(
            "max_players is above MAX_PLAYERS"
        ))
    );
    assert_eq!(players(MAX_PLAYERS), Ok(MAX_PLAYERS));
    // Any chance up to 100 compiles, and no chances at all: only the first stack comes.
    assert_eq!(chances(vec![0, 100]), Ok(vec![0, 100]));
    assert_eq!(chances(vec![]), Ok(vec![]));
    // A full table compiles.
    let full = edit(|p| {
        pack(p, "t").params.as_mut().unwrap().land_points = vec![1500; MAX_LAND_POINTS];
    });
    assert_eq!(compile(&full).unwrap().params().land_points(u8::MAX), 1500);
}

// covers: GAM-RES-002
#[test]
fn one_pack_provides_the_starting_resources() {
    let core = core();
    let part = compile(&packs()).unwrap();
    let index = |k: &str| {
        core.resources()
            .iter()
            .position(|r| r.key == key(k))
            .unwrap()
    };
    let (gold, wood) = (index("t:gold"), index("t:wood"));
    let rows = part.starting_resources();
    assert_eq!(rows.len(), 2);
    // Every core resource has an amount, in the core's order; one left out starts at 0.
    assert_eq!(rows[0].human().len(), core.resources().len());
    assert_eq!((rows[0].human()[gold], rows[0].human()[wood]), (3000, 5));
    assert_eq!(
        (rows[0].computer()[gold], rows[0].computer()[wood]),
        (500, 0)
    );
    assert_eq!((rows[1].human()[gold], rows[1].human()[wood]), (2000, 0));
    assert!(rows[1].computer().iter().all(|&a| a == 0));
    assert_eq!(
        compile_err(&edit(|p| pack(p, "t").starting_resources.clear())),
        AdventureCompileError::MissingDifficulties
    );
    assert_eq!(
        compile_err(&edit(|p| pack(p, "u").starting_resources = starting())),
        AdventureCompileError::DuplicateDifficulties("u".into())
    );
    let row = |f: fn(&mut StartingResourcesDef)| {
        compile_err(&edit(|p| f(&mut pack(p, "t").starting_resources[1])))
    };
    assert_eq!(
        row(|r| r.computer.push((key("t:gold"), MAX_GAIN + 1))),
        AdventureCompileError::InvalidDifficulty {
            row: 1,
            reason: "an amount is above 2^31 - 1"
        }
    );
    assert_eq!(
        row(|r| r.human.push((key("t:gold"), 1))),
        AdventureCompileError::InvalidDifficulty {
            row: 1,
            reason: "a resource is named twice"
        }
    );
    assert_eq!(
        row(|r| r.human.push((key("t:ore"), 1))),
        AdventureCompileError::UnknownResource("t:ore".into())
    );
    // `v` does not depend on `u`.
    assert_eq!(
        compile_err(&edit(|p| {
            pack(p, "t").starting_resources.clear();
            pack(p, "v").starting_resources = vec![StartingResourcesDef {
                human: vec![(key("u:gem"), 1)],
                computer: vec![],
            }];
        })),
        AdventureCompileError::ReferenceOutsideDependencies {
            key: "u:gem".into(),
            pack: "v".into()
        }
    );
    // The largest amount and the most rows compile.
    let most = edit(|p| {
        pack(p, "t").starting_resources = (0..MAX_DIFFICULTIES)
            .map(|_| StartingResourcesDef {
                human: vec![(key("t:gold"), MAX_GAIN)],
                computer: vec![],
            })
            .collect();
    });
    let part = compile(&most).unwrap();
    assert_eq!(part.starting_resources().len(), MAX_DIFFICULTIES);
    assert_eq!(part.starting_resources()[255].human()[gold], MAX_GAIN);
    let too_many = edit(|p| {
        pack(p, "t")
            .starting_resources
            .resize(MAX_DIFFICULTIES + 1, starting()[1].clone());
    });
    assert_eq!(
        compile_err(&too_many),
        AdventureCompileError::TooManyEntries("difficulty")
    );
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
            AdventureCompileError::UnknownResource("t:x".into()),
            "unknown resource `t:x`",
        ),
        (
            AdventureCompileError::UnknownObjectType("t:x".into()),
            "unknown object type `t:x`",
        ),
        (
            AdventureCompileError::InvalidObjectType {
                key: "t:x".into(),
                reason: "radius is 0",
            },
            "object type `t:x`: radius is 0",
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
            AdventureCompileError::MissingDifficulties,
            "no pack provides the starting resources by difficulty",
        ),
        (
            AdventureCompileError::DuplicateDifficulties("u".into()),
            "pack `u` provides the starting resources by difficulty a second time",
        ),
        (
            AdventureCompileError::InvalidDifficulty {
                row: 2,
                reason: "a resource is named twice",
            },
            "difficulty 2: a resource is named twice",
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

// covers: GAM-BAT-001
#[test]
fn the_battle_terrains_name_terrains_of_the_closure() {
    let battle = |underground: &str, coast: &str, field: &str| {
        edit(|p| {
            pack(p, "t").params.as_mut().unwrap().battle = Some(BattleTerrainsDef {
                underground: key(underground),
                coast: key(coast),
                coast_battlefield: key(field),
            });
        })
    };
    let part = compile(&battle("t:rock", "t:grass", "t:shore")).unwrap();
    let core = core();
    let terrain = |k| core.terrain_by_key(k).unwrap();
    assert_eq!(
        part.params().battle_terrains(),
        Some(&BattleTerrains {
            underground: terrain("t:rock"),
            coast: terrain("t:grass"),
            coast_battlefield: key("t:shore"),
        })
    );
    assert_eq!(compile(&packs()).unwrap().params().battle_terrains(), None);
    // The battle terrains are part of the part's hash.
    assert_ne!(
        part.content_hash(),
        compile(&packs()).unwrap().content_hash()
    );
    assert_ne!(
        part.content_hash(),
        compile(&battle("t:rock", "t:grass", "t:boats"))
            .unwrap()
            .content_hash()
    );
    assert_eq!(
        compile_err(&battle("t:lava", "t:grass", "t:shore")),
        AdventureCompileError::UnknownTerrain("t:lava".into())
    );
    assert_eq!(
        compile_err(&battle("t:rock", "t:lava", "t:shore")),
        AdventureCompileError::UnknownTerrain("t:lava".into())
    );
    // `t` does not depend on `u`.
    let outside = |k: &str| AdventureCompileError::ReferenceOutsideDependencies {
        key: k.into(),
        pack: "t".into(),
    };
    assert_eq!(
        compile_err(&battle("u:swamp", "t:grass", "t:shore")),
        outside("u:swamp")
    );
    assert_eq!(
        compile_err(&battle("t:rock", "t:grass", "u:shore")),
        outside("u:shore")
    );
}
