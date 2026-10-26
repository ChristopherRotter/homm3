//! Tests of the import (E08 T5a, T6a) with the original packs' rules: the static layer, the
//! players and placed heroes, each typed error and unsupported case, the code tables against the
//! packs, and parse, import and validation together on arbitrary bytes.

use std::sync::{Arc, LazyLock};

use h3_adventure::{Guards, Pos as TilePos, Scenery, TileMask};
use h3_content::{Formation, HeroEffect};
use h3_contracts::{ArmyStack, HeroSex, PlayerId};
use h3_game::{GameRuleset, HeroArmy, HeroDetailsSetup, PlayerSetup};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};
use proptest::prelude::*;

use super::encode::encode;
use super::sample;
use super::strategy::{raw_map, version};
use crate::import::codes;
use crate::model::{
    Artifacts, Bits, Body, CustomHero, HeroBody, MainTown, Message, MonsterBody, MonsterTreasure,
    Object, Player, Pos, RawMap, Stack, Template, Text, Tile, Version,
};
use crate::{ImportError, Limits, Unsupported, import, import_map, parse};

/// The rules of RoE to SoD, or of RoE alone.
fn rules(packs: usize) -> GameRuleset {
    let (core, part) = compile(&ORIGINAL_PACKS[..packs], "base:faithful").unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ][..packs],
        "base:faithful",
    )
    .unwrap();
    GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle)).unwrap()
}

pub(super) static SOD: LazyLock<GameRuleset> = LazyLock::new(|| rules(3));

/// A template mask in the file's encoding marking these `(column, row)` tiles of the anchor:
/// bit `j` of byte `i` is the tile `7 − j` columns left of and `5 − i` rows above it.
fn file_mask(tiles: &[(u8, u8)]) -> [u8; 6] {
    let mut mask = [0u8; 6];
    for &(column, row) in tiles {
        let bit = 47 - (8 * usize::from(row) + usize::from(column));
        mask[bit / 8] |= 1 << (bit % 8);
    }
    mask
}

/// A template of `class` blocking `blocked` and visited from `visitable`.
pub(super) fn template(class: u32, blocked: &[(u8, u8)], visitable: &[(u8, u8)]) -> Template {
    Template {
        def: Text(b"obj.def".to_vec()),
        passable: file_mask(blocked).map(|b| !b),
        visitable: file_mask(visitable),
        class,
        ..Template::default()
    }
}

pub(super) fn object(x: u8, y: u8, template: u32, body: Body) -> Object {
    Object {
        pos: Pos { x, y, z: 0 },
        template,
        body,
    }
}

/// A 3 × 3 SoD map: red plays, the tiles are dirt with grass and roads and rivers at (1, 0);
/// a blocker anchored at (2, 2) blocks it and the tile left of it and is visited from (2, 1);
/// a hero object anchored at (2, 0), so red's Orrin (type 0) stands on (1, 0), with
/// stone golems in slot 2 and obsidian gargoyles in slot 5; its template is the editor's
/// (visited one column left).
fn map() -> RawMap {
    let mut m = sample(Version::Sod);
    m.header.size = 3;
    m.terrain.size = 3;
    m.terrain.tiles = vec![Tile::default(); 9];
    m.terrain.tiles[1] = Tile {
        terrain: 2,
        road: 3,
        river: 4,
        ..Tile::default()
    };
    m.templates = vec![
        template(124, &[(0, 0), (1, 0)], &[(0, 1)]),
        template(34, &[(1, 0)], &[(1, 0)]),
    ];
    let mut army = vec![None; 7];
    army[2] = Some(Stack {
        creature: 32,
        count: 9,
    });
    army[5] = Some(Stack {
        creature: 31,
        count: 4,
    });
    m.objects = vec![
        object(2, 2, 0, Body::None),
        object(
            2,
            0,
            1,
            Body::Hero(Box::new(HeroBody {
                owner: 0,
                hero_type: 0,
                army: Some(army),
                sex: 0xFF,
                ..HeroBody::default()
            })),
        ),
    ];
    m
}

fn import_err(edit: impl FnOnce(&mut RawMap)) -> ImportError {
    let mut m = map();
    edit(&mut m);
    import(&m, &SOD).unwrap_err()
}

// covers: ADV-OBJ-001
#[test]
fn imports_the_static_layer_players_and_a_placed_hero() {
    let core = SOD.core();
    let part = SOD.adventure();
    let setup = import(&map(), &SOD).unwrap();
    assert_eq!(
        setup.players,
        [PlayerSetup {
            colour: 0,
            human: true
        }]
    );
    let s = &setup.map;
    assert_eq!((s.width, s.height, s.levels, s.tiles.len()), (3, 3, 1, 9));
    let dirt = core.terrain_by_key("base:dirt");
    assert!(
        s.tiles
            .iter()
            .enumerate()
            .all(|(i, t)| i == 1
                || (Some(t.terrain) == dirt && t.road.is_none() && t.river.is_none()))
    );
    assert_eq!(Some(s.tiles[1].terrain), core.terrain_by_key("base:grass"));
    assert_eq!(s.tiles[1].road, part.road_by_key("base:cobblestone_road"));
    assert_eq!(s.tiles[1].river, part.river_by_key("base:lava_river"));
    // The hero is not a static object.
    assert_eq!(s.objects.len(), 1);
    let blocker = s.objects[0].clone();
    assert_eq!(blocker.anchor, TilePos::new(2, 2, 0));
    assert_eq!(blocker.blocked, TileMask::from_bits(0b11));
    assert_eq!(blocker.visitable, TileMask::from_bits(1 << 8));
    assert_eq!((blocker.object_type, blocker.amount), (None, None));
    assert_eq!(blocker.scenery, Scenery::Other);
    assert_eq!(import_map(&map(), &SOD).unwrap(), setup.map);

    let [hero] = setup.heroes.as_slice() else {
        panic!("one hero expected");
    };
    assert_eq!(Some(hero.hero_type), core.hero_type_by_key("base:orrin"));
    assert_eq!(hero.owner, PlayerId::new(0));
    assert_eq!(hero.pos, TilePos::new(1, 0, 0));
    let creature = |key| core.creature_by_key(key).unwrap();
    assert_eq!(
        hero.army,
        HeroArmy::Custom(vec![
            ArmyStack {
                slot: 2,
                creature: creature("base:stone_golem"),
                count: 9,
            },
            ArmyStack {
                slot: 5,
                creature: creature("base:obsidian_gargoyle"),
                count: 4,
            },
        ])
    );
    assert_eq!(setup.validate(&SOD), Ok(()));
}

#[test]
fn players_are_the_colours_that_play_in_order() {
    let mut m = map();
    m.players[0].human = false;
    m.players[3] = m.players[0].clone();
    m.players[6].human = true;
    let setup = import(&m, &SOD).unwrap();
    let colours: Vec<u8> = setup.players.iter().map(|p| p.colour).collect();
    assert_eq!(colours, [0, 3, 6]);
    // Owned by blue (colour 3), the second player.
    if let Body::Hero(h) = &mut m.objects[1].body {
        h.owner = 3;
    }
    assert_eq!(import(&m, &SOD).unwrap().heroes[0].owner, PlayerId::new(1));
}

#[test]
fn a_hero_without_stacks_of_its_own_takes_its_starting_army() {
    for army in [None, Some(vec![None; 7])] {
        let mut m = map();
        if let Body::Hero(h) = &mut m.objects[1].body {
            h.army = army;
        }
        let setup = import(&m, &SOD).unwrap();
        assert_eq!(setup.heroes[0].army, HeroArmy::Starting);
        assert_eq!(setup.validate(&SOD), Ok(()));
    }
}

#[test]
fn what_the_increment_cannot_represent_is_unsupported() {
    let unsupported = |edit: fn(&mut RawMap)| match import_err(edit) {
        ImportError::Unsupported(what) => what,
        other => panic!("expected an unsupported case, got {other:?}"),
    };
    assert_eq!(
        unsupported(|m| {
            m.players[0].details.as_mut().unwrap().main_town = Some(MainTown {
                generate_hero: true,
                ..MainTown::default()
            });
        }),
        Unsupported::HeroAtMainTown(0)
    );
    assert_eq!(
        unsupported(|m| m.templates[1].class = 70),
        Unsupported::RandomHero(1)
    );
    assert_eq!(
        unsupported(|m| m.templates[1].class = 214),
        Unsupported::HeroPlaceholder(1)
    );
    for code in [u16::MAX - 1, u16::MAX - 14] {
        let mut m = map();
        if let Body::Hero(h) = &mut m.objects[1].body {
            h.army.as_mut().unwrap()[0] = Some(Stack {
                creature: code,
                count: 1,
            });
        }
        assert_eq!(
            import(&m, &SOD),
            Err(ImportError::Unsupported(Unsupported::RandomStack(1)))
        );
    }
    // RoE codes are one byte.
    let mut m = map();
    m.header.version = Version::RoE;
    if let Body::Hero(h) = &mut m.objects[1].body {
        h.army.as_mut().unwrap()[0] = Some(Stack {
            creature: 0xF1,
            count: 1,
        });
    }
    assert_eq!(
        import(&m, &SOD),
        Err(ImportError::Unsupported(Unsupported::RandomStack(1)))
    );
    // A town (of any kind) blocking the hero's tile at (1, 0).
    for class in [98, 77] {
        let mut m = map();
        m.templates.push(template(class, &[(0, 0)], &[]));
        m.objects.push(object(1, 0, 2, Body::None));
        assert_eq!(
            import(&m, &SOD),
            Err(ImportError::Unsupported(Unsupported::HeroInTown(1)))
        );
    }
}

/// [`map`] with its blocker (object 0) of this class and subclass and with this body.
fn typed_map(class: u32, subclass: u32, body: Body) -> RawMap {
    let mut m = map();
    m.templates[0].class = class;
    m.templates[0].subclass = subclass;
    m.objects[0].body = body;
    m
}

fn pile_body(amount: u32, message: Option<Message>) -> Body {
    Body::Resource { message, amount }
}

// covers: ADV-OBJ-002
#[test]
fn the_typed_classes_import_as_their_object_types() {
    let colours = [
        "light_blue",
        "green",
        "red",
        "dark_blue",
        "brown",
        "purple",
        "white",
        "black",
    ];
    let resources = [
        "wood", "mercury", "ore", "sulfur", "crystal", "gems", "gold",
    ];
    let mut cases: Vec<(u32, u32, String)> = vec![
        (76, 0, "base:random_resource".into()),
        (12, 0, "base:campfire".into()),
        (58, 0, "base:redwood_observatory".into()),
        (60, 0, "base:pillar_of_fire".into()),
        (13, 0, "base:cartographer_water".into()),
        (13, 1, "base:cartographer_land".into()),
        (13, 2, "base:cartographer_subterranean".into()),
    ];
    for (subclass, resource) in (0..).zip(resources) {
        cases.push((79, subclass, format!("base:pile_{resource}")));
    }
    for (subclass, colour) in (0..).zip(colours) {
        cases.push((10, subclass, format!("base:keymaster_{colour}")));
        cases.push((9, subclass, format!("base:border_guard_{colour}")));
        cases.push((212, subclass, format!("ab:border_gate_{colour}")));
    }
    // E08 T20a2a: the monsters (class 54's subclass is the creature) and the special grounds.
    cases.push((54, 0, "base:monster".into()));
    cases.push((71, 0, "base:random_monster".into()));
    for (class, level) in [72, 73, 74, 75, 162, 163, 164].into_iter().zip(1..) {
        cases.push((class, 0, format!("base:random_monster_{level}")));
    }
    for (class, key) in [
        (21, "base:cursed_ground"),
        (46, "base:magic_plains"),
        (222, "sod:clover_field"),
        (223, "sod:cursed_ground"),
        (224, "sod:evil_fog"),
        (226, "sod:fiery_fields"),
        (227, "sod:holy_ground"),
        (228, "sod:lucid_pools"),
        (229, "sod:magic_clouds"),
        (230, "sod:magic_plains"),
        (231, "sod:rocklands"),
    ] {
        cases.push((class, 0, key.into()));
    }
    let part = SOD.adventure();
    for (class, subclass, key) in &cases {
        let body = match class {
            76 | 79 => pile_body(0, None),
            54 | 71..=75 | 162..=164 => Body::Monster(MonsterBody::default()),
            _ => Body::None,
        };
        let setup = import(&typed_map(*class, *subclass, body), &SOD).unwrap();
        let object = setup.map.objects[0].clone();
        assert_eq!(object.object_type, part.object_type_by_key(key), "{key}");
        assert!(object.object_type.is_some(), "{key}");
        assert_eq!(object.amount, None);
        // The footprint is the template's, as for a blocker.
        assert_eq!(object.blocked, TileMask::from_bits(0b11));
    }
    // Every object type of the original packs has its class.
    assert_eq!(cases.len(), part.object_types().len());
    // A typed class ignores the subclass where it has one type.
    let fire = import(&typed_map(12, 7, Body::None), &SOD).unwrap();
    assert_eq!(
        fire.map.objects[0].object_type,
        part.object_type_by_key("base:campfire")
    );
    // A class without a type stays a blocker.
    let other = import(&typed_map(55, 0, Body::None), &SOD).unwrap();
    assert_eq!(other.map.objects[0].object_type, None);
}

// covers: ADV-OBJ-002, ADV-PIK-004
#[test]
fn a_piles_fixed_amount_message_and_guards_carry_over() {
    let part = SOD.adventure();
    let amount = |class, amount, message| {
        let setup = import(&typed_map(class, 6, pile_body(amount, message)), &SOD).unwrap();
        (
            setup.map.objects[0].object_type,
            setup.map.objects[0].amount,
        )
    };
    let gold = part.object_type_by_key("base:pile_gold");
    let random = part.object_type_by_key("base:random_resource");
    // Gold counts in hundreds: the amount stays the file's, the type multiplies it.
    assert_eq!(amount(79, 10, None), (gold, Some(10)));
    assert_eq!(amount(79, 0, None), (gold, None));
    assert_eq!(amount(76, 3, None), (random, Some(3)));
    // A message without guards is presentation only.
    let note = Message {
        text: Text(b"Take it".to_vec()),
        guards: None,
    };
    assert_eq!(amount(79, 2, Some(note.clone())), (gold, Some(2)));
    // E08 T21a: a guards record's stacks are the pile's guards, by slot, the message's text a
    // question; an empty record is a plain pile, as the original's stack count reads it.
    let pikemen = SOD.core().creature_by_key("base:pikeman").unwrap();
    let archers = SOD.core().creature_by_key("base:archer").unwrap();
    let stack = |creature, count| Some(Stack { creature, count });
    let guarded = |class, text: &[u8], army: Vec<Option<Stack>>| {
        let message = Message {
            text: Text(text.to_vec()),
            guards: Some(army),
        };
        let setup = import(&typed_map(class, 6, pile_body(4, Some(message))), &SOD).unwrap();
        assert!(setup.validate(&SOD).is_ok());
        let object = setup.map.objects[0].clone();
        assert_eq!(object.amount, Some(4));
        object.guards
    };
    for class in [76, 79] {
        assert_eq!(guarded(class, b"", vec![None; 7]), None);
        assert_eq!(guarded(class, b"Fight?", vec![]), None);
        let mut army = vec![None; 7];
        army[2] = stack(0, 5);
        army[4] = stack(2, 3);
        let both = vec![
            ArmyStack {
                slot: 2,
                creature: pikemen,
                count: 5,
            },
            ArmyStack {
                slot: 4,
                creature: archers,
                count: 3,
            },
        ];
        assert_eq!(
            guarded(class, b"Fight?", army.clone()),
            Some(Guards {
                army: both.clone(),
                question: true
            })
        );
        assert_eq!(
            guarded(class, b"", army),
            Some(Guards {
                army: both,
                question: false
            })
        );
    }
    // A random stack among the guards makes the map unsupported; its static layer keeps the pile
    // unguarded.
    let message = Message {
        guards: Some(vec![stack(0, 5), stack(u16::MAX - 1, 1)]),
        ..note.clone()
    };
    let m = typed_map(79, 6, pile_body(4, Some(message)));
    assert_eq!(
        import(&m, &SOD),
        Err(ImportError::Unsupported(Unsupported::RandomStack(0)))
    );
    let layer = import_map(&m, &SOD).unwrap();
    assert_eq!(
        (
            layer.objects[0].object_type,
            layer.objects[0].amount,
            &layer.objects[0].guards
        ),
        (gold, Some(4), &None)
    );
    // An unknown creature among them is the map's error.
    let message = Message {
        guards: Some(vec![stack(500, 5)]),
        ..note
    };
    assert!(matches!(
        import_map(&typed_map(79, 6, pile_body(4, Some(message))), &SOD),
        Err(ImportError::UnknownCode {
            what: "creature",
            code: 500
        })
    ));
    // A fixed amount the setup rejects passes the import: validation refuses it.
    let m = typed_map(79, 0, pile_body(u32::MAX, None));
    let setup = import(&m, &SOD).unwrap();
    assert_eq!(setup.map.objects[0].amount, Some(u32::MAX));
    assert!(setup.validate(&SOD).is_err());
}

// covers: ADV-MON-001
#[test]
fn monsters_import_with_their_data() {
    let body = |count, treasure| {
        Body::Monster(MonsterBody {
            identifier: Some(7),
            count,
            disposition: 3,
            treasure,
            never_flees: true,
            no_growth: false,
        })
    };
    let rules = SOD.core();
    let resource = |key: &str| {
        let index = rules.resources().iter().position(|r| r.key.as_str() == key);
        h3_content::ResourceId::from_index(index.unwrap()).unwrap()
    };
    // Class 54's subclass is the creature; the treasure keeps its resources above 0, in order.
    let treasure = MonsterTreasure {
        text: Text::default(),
        resources: [5, 0, -3, 0, 0, 0, 2500],
        artifact: None,
    };
    let setup = import(&typed_map(54, 13, body(20, Some(treasure.clone()))), &SOD).unwrap();
    let monster = setup.map.objects[0].monster.clone().unwrap();
    assert_eq!(monster.creature, rules.creature_by_key("base:archangel"));
    assert_eq!(
        (
            monster.count,
            monster.disposition,
            monster.never_flees,
            monster.no_growth
        ),
        (20, 3, true, false)
    );
    assert_eq!(
        monster.treasure,
        [(resource("base:wood"), 5), (resource("base:gold"), 2500)]
    );
    setup.validate(&SOD).unwrap();
    // A random monster names none; a count of 0 is drawn when the game is built.
    let setup = import(&typed_map(164, 0, body(0, None)), &SOD).unwrap();
    let monster = setup.map.objects[0].monster.clone().unwrap();
    assert_eq!((monster.creature, monster.count), (None, 0));
    assert!(monster.treasure.is_empty());
    setup.validate(&SOD).unwrap();
    // A count above the original's 12 bits passes the import; validation refuses it.
    let setup = import(&typed_map(54, 0, body(4096, None)), &SOD).unwrap();
    assert!(setup.validate(&SOD).is_err());
    // An unknown creature code.
    assert_eq!(
        import(&typed_map(54, 122, body(1, None)), &SOD),
        Err(ImportError::UnknownCode {
            what: "creature",
            code: 122
        })
    );
    // A treasure with an artifact: unsupported, its static layer keeps the monster without it.
    let m = typed_map(
        54,
        0,
        body(
            9,
            Some(MonsterTreasure {
                artifact: Some(7),
                ..treasure
            }),
        ),
    );
    assert_eq!(
        import(&m, &SOD),
        Err(ImportError::Unsupported(Unsupported::MonsterArtifact(0)))
    );
    let layer = import_map(&m, &SOD).unwrap();
    assert_eq!(layer.objects[0].monster.as_ref().unwrap().count, 9);
    assert!(layer.objects[0].object_type.is_some());
}

// covers: ADV-GRD-001
#[test]
fn a_special_ground_lies_on_its_pictures_tiles() {
    let drawn = |class: u32, picture: &str| {
        let mut m = typed_map(class, 0, Body::None);
        m.templates[0].def = Text(picture.as_bytes().to_vec());
        import(&m, &SOD).unwrap().map.objects[0].drawn
    };
    // 6 × 4, 3 × 3 and 2 × 5 tiles to the left of and above the anchor, by the picture's name
    // in any case; a picture the table lacks lies on its anchor alone (`default.msk`).
    let rect = |width: u8, height: u8| {
        let row = (1u64 << width) - 1;
        TileMask::from_bits((0..height).fold(0, |b, r| b | row << (8 * u32::from(r))))
    };
    assert_eq!(drawn(21, "AVXcrsd0.def"), rect(6, 4));
    assert_eq!(drawn(230, "avxmp5.DEF"), rect(3, 3));
    assert_eq!(drawn(223, "AVXcg6"), rect(2, 5));
    assert_eq!(drawn(227, "AVXhg1.def"), rect(8, 6));
    assert_eq!(drawn(46, "unknown.def"), rect(1, 1));
    // Other objects lie on no more than their masks.
    assert_eq!(drawn(58, "AVXcrsd0.def"), TileMask::default());
    // Favorable Winds changes no battle: a blocker without tiles.
    let winds = import(&typed_map(225, 0, Body::None), &SOD).unwrap();
    assert_eq!(winds.map.objects[0].object_type, None);
}

// covers: ADV-OBJ-001
#[test]
fn a_town_elsewhere_is_a_blocker() {
    let in_town = |x: u8, y: u8, z: u8, blocked: &[(u8, u8)]| {
        let mut m = map();
        m.header.two_levels = true;
        m.terrain.levels = 2;
        m.terrain.tiles = vec![Tile::default(); 18];
        m.templates.push(template(98, blocked, &[]));
        m.objects.push(object(x, y, 2, Body::None));
        m.objects[2].pos.z = z;
        match import(&m, &SOD) {
            Ok(setup) => {
                assert_eq!(setup.map.objects.len(), 2);
                false
            }
            Err(e) => {
                assert_eq!(e, ImportError::Unsupported(Unsupported::HeroInTown(1)));
                true
            }
        }
    };
    // The hero stands on (1, 0) of the surface.
    assert!(in_town(2, 0, 0, &[(1, 0)]));
    assert!(!in_town(2, 0, 0, &[(0, 0)]));
    assert!(!in_town(1, 0, 1, &[(0, 0)]));
    // Anchored left of the hero, or reaching only the row below it.
    assert!(!in_town(0, 0, 0, &[(0, 0)]));
    assert!(!in_town(1, 1, 0, &[(0, 0)]));
    assert!(!in_town(2, 0, 0, &[]));
    // Too far right or down for a mask's column or row to reach back.
    assert!(!in_town(255, 0, 0, &[(7, 0)]));
    assert!(!in_town(1, 255, 0, &[(0, 5)]));
}

#[test]
fn each_import_error_is_typed() {
    let mut m = map();
    m.terrain.size = usize::from(u16::MAX) + 1;
    assert_eq!(
        import_map(&m, &SOD),
        Err(ImportError::MapSize(usize::from(u16::MAX) + 1, 1))
    );
    m.terrain.size = 3;
    m.terrain.levels = 256;
    assert_eq!(import_map(&m, &SOD), Err(ImportError::MapSize(3, 256)));
    let code = |what, code| ImportError::UnknownCode { what, code };
    assert_eq!(
        import_err(|m| m.terrain.tiles[4].terrain = 10),
        code("terrain", 10)
    );
    assert_eq!(import_err(|m| m.terrain.tiles[4].road = 4), code("road", 4));
    assert_eq!(
        import_err(|m| m.terrain.tiles[4].river = 5),
        code("river", 5)
    );
    let set_army = |m: &mut RawMap, creature| {
        if let Body::Hero(h) = &mut m.objects[1].body {
            h.army.as_mut().unwrap()[2] = Some(Stack { creature, count: 1 });
        }
    };
    assert_eq!(import_err(|m| set_army(m, 122)), code("creature", 122));
    assert_eq!(
        import_err(|m| set_army(m, 0xFFF0)),
        code("creature", 0xFFF0)
    );
    assert_eq!(
        import_err(|m| {
            if let Body::Hero(h) = &mut m.objects[1].body {
                h.hero_type = 156;
            }
        }),
        code("hero type", 156)
    );
    assert_eq!(
        import_err(|m| m.objects[0].template = 2),
        ImportError::Template(0)
    );
    for (class, subclass) in [(79, 7), (13, 3), (10, 8), (9, 8), (212, u32::MAX)] {
        let m = typed_map(class, subclass, Body::None);
        assert_eq!(import(&m, &SOD), Err(ImportError::Subclass(0)));
        assert_eq!(import_map(&m, &SOD), Err(ImportError::Subclass(0)));
    }
    for owner in [1, 0xFF] {
        let mut m = map();
        if let Body::Hero(h) = &mut m.objects[1].body {
            h.owner = owner;
        }
        assert_eq!(import(&m, &SOD), Err(ImportError::HeroOwner(1)));
    }
    let tile = |visitable: &[(u8, u8)], x| {
        let mut m = map();
        m.templates[1].visitable = file_mask(visitable);
        m.objects[1].pos.x = x;
        import(&m, &SOD)
    };
    assert_eq!(tile(&[], 2), Err(ImportError::HeroTile(1)));
    assert_eq!(tile(&[(1, 0), (0, 0)], 2), Err(ImportError::HeroTile(1)));
    assert_eq!(tile(&[(1, 0)], 0), Err(ImportError::HeroTile(1)));
    assert_eq!(tile(&[(0, 1)], 0), Err(ImportError::HeroTile(1)));
    assert_eq!(
        tile(&[(0, 0)], 0).unwrap().heroes[0].pos,
        TilePos::new(0, 0, 0)
    );
}

#[test]
fn content_beyond_the_rules_is_missing() {
    let roe = rules(1);
    let mut m = map();
    // A Conflux creature and an AB hero type with RoE's rules.
    if let Body::Hero(h) = &mut m.objects[1].body {
        h.army.as_mut().unwrap()[2] = Some(Stack {
            creature: 118,
            count: 1,
        });
    }
    assert_eq!(
        import(&m, &roe),
        Err(ImportError::MissingContent("ab:pixie"))
    );
    if let Body::Hero(h) = &mut m.objects[1].body {
        h.hero_type = 128;
    }
    assert!(matches!(
        import(&m, &roe),
        Err(ImportError::MissingContent(key)) if key.starts_with("ab:")
    ));
    // Border gates came with AB.
    assert_eq!(
        import(&typed_map(212, 3, Body::None), &roe),
        Err(ImportError::MissingContent("ab:border_gate_dark_blue"))
    );
}

#[test]
fn the_errors_and_reasons_read_well() {
    let cases = [
        (
            ImportError::Unsupported(Unsupported::RandomHero(4)),
            "not supported yet: a random hero",
        ),
        (
            ImportError::MapSize(300, 2),
            "a map of side 300 on 2 levels is too large",
        ),
        (
            ImportError::UnknownCode {
                what: "road",
                code: 9,
            },
            "unknown road code 9",
        ),
        (
            ImportError::MissingContent("ab:pixie"),
            "the ruleset lacks `ab:pixie`",
        ),
        (ImportError::Template(3), "object 3 has no template"),
        (
            ImportError::Subclass(3),
            "object 3 has a subclass its class does not define",
        ),
        (
            ImportError::HeroOwner(3),
            "hero object 3 belongs to a colour nobody plays",
        ),
        (
            ImportError::HeroTile(3),
            "hero object 3 does not name one tile to stand on",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
    }
    let reasons = [
        (
            Unsupported::HeroAtMainTown(0),
            "a hero generated at a main town",
        ),
        (Unsupported::RandomHero(0), "a random hero"),
        (
            Unsupported::HeroPlaceholder(0),
            "a campaign hero placeholder",
        ),
        (
            Unsupported::RandomStack(0),
            "a random stack in a hero's army or a pile's guards",
        ),
        (Unsupported::HeroInTown(0), "a hero standing in a town"),
        (
            Unsupported::MonsterArtifact(0),
            "a monster with an artifact in its treasure",
        ),
    ];
    for (what, text) in reasons {
        assert_eq!(what.reason(), text);
    }
}

/// The ids the packs record beside each creature and hero type (`# original id N`).
fn pack_ids(kind: &str) -> Vec<(u16, String)> {
    let mut ids = Vec::new();
    for (pack, text) in [
        ("base", h3_pack::BASE_CORE),
        ("ab", h3_pack::AB_CORE),
        ("sod", h3_pack::SOD_CORE),
    ] {
        let mut lines = text.lines();
        while let Some(line) = lines.next() {
            if line != format!("[[{kind}]]") {
                continue;
            }
            let entry = lines.next().unwrap();
            let (id, original) = entry
                .strip_prefix("id = \"")
                .and_then(|rest| rest.split_once("\"  # original id "))
                .unwrap();
            let number = original
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap();
            ids.push((number.parse().unwrap(), format!("{pack}:{id}")));
        }
    }
    ids.sort();
    ids
}

#[test]
fn the_code_tables_follow_the_packs() {
    let creatures: Vec<(u16, String)> = codes::CREATURES
        .iter()
        .map(|&(id, key)| (id, key.to_owned()))
        .collect();
    assert_eq!(creatures, pack_ids("creature"));
    let heroes: Vec<(u16, String)> = (0..)
        .zip(codes::HERO_TYPES)
        .map(|(id, key)| (id, key.to_owned()))
        .collect();
    assert_eq!(heroes, pack_ids("hero"));
    for (table, kind) in [
        (&codes::ARTIFACTS[..], "artifact"),
        (&codes::SPELLS[..], "spell"),
    ] {
        let ids: Vec<(u16, String)> = (0..)
            .zip(table)
            .map(|(id, k)| (id, (*k).to_owned()))
            .collect();
        assert_eq!(ids, pack_ids(kind), "{kind}");
    }
    let skills: Vec<&str> = SOD.core().skills().iter().map(|s| s.key.as_str()).collect();
    assert_eq!(skills, codes::SKILLS);
    let core = SOD.core();
    let part = SOD.adventure();
    assert!(
        codes::TERRAINS
            .iter()
            .all(|k| core.terrain_by_key(k).is_some())
    );
    assert!(codes::ROADS.iter().all(|k| part.road_by_key(k).is_some()));
    assert!(codes::RIVERS.iter().all(|k| part.river_by_key(k).is_some()));
    // The terrains in the .h3m code order are the core's first ten.
    let order: Vec<&str> = core.terrains().iter().map(|t| t.key.as_str()).collect();
    assert_eq!(order, codes::TERRAINS);
}

proptest! {
    /// Parse, import and validation together never panic on arbitrary bytes (ADR-0015 point 4;
    /// `cargo-fuzz` later).
    #[test]
    fn arbitrary_bytes_never_panic_parse_import_or_validation(
        v in version(),
        rest in prop::collection::vec(any::<u8>(), 0..1500),
    ) {
        let mut bytes = v.code().to_le_bytes().to_vec();
        bytes.extend_from_slice(&rest);
        if let Ok(m) = parse(&bytes, &Limits::default())
            && let Ok(setup) = import(&m, &SOD)
        {
            let _ = setup.validate(&SOD);
        }
    }

    /// Every map the reader's model can hold, and every damaged encoding of one, goes through
    /// parse, import and validation without a panic.
    #[test]
    fn maps_and_damaged_maps_never_panic_the_import(
        m in raw_map(),
        edits in prop::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 0..8),
    ) {
        let mut bytes = encode(&m);
        for (at, value) in edits {
            let i = at.index(bytes.len());
            bytes[i] = value;
        }
        for m in [Ok(m), parse(&bytes, &Limits::default())].into_iter().flatten() {
            if let Ok(setup) = import(&m, &SOD) {
                let _ = setup.validate(&SOD);
            }
        }
    }
}

/// The test map's hero body.
// covers: GAM-SET-002
#[test]
fn the_scenarios_level_limit_and_banned_skills_carry_over() {
    // E08 T29: the header's level limit (AB and SoD) and SoD's set of banned skills, each bit
    // the skill of its code (the original's ids).
    let core = SOD.core();
    let mut m = map();
    m.header.level_limit = 12;
    // Navigation (5), Eagle Eye (11) and First Aid (27).
    m.banned_skills = Bits(vec![1 << 5, 1 << 3, 0, 1 << 3]);
    let setup = import(&m, &SOD).unwrap();
    assert_eq!(setup.level_limit, 12);
    let skill = |k| core.skill_by_key(k).unwrap();
    assert_eq!(
        setup.banned_skills,
        [
            skill("base:navigation"),
            skill("base:eagle_eye"),
            skill("base:first_aid")
        ]
    );
    assert_eq!(setup.validate(&SOD), Ok(()));
    // The test map bans none and has no limit.
    let plain = import(&map(), &SOD).unwrap();
    assert_eq!((plain.level_limit, plain.banned_skills.len()), (0, 0));
    // A set bit past the 28 skills is an unknown code.
    assert_eq!(
        import_err(|m| m.banned_skills = Bits(vec![0, 0, 0, 1 << 4])),
        ImportError::UnknownCode {
            what: "secondary skill",
            code: 28
        }
    );
}

fn body(m: &mut RawMap) -> &mut HeroBody {
    match &mut m.objects[1].body {
        Body::Hero(h) => h,
        _ => unreachable!("the test map's second object is its hero"),
    }
}

/// The imported customisations of the test map's hero after `edit`.
fn customised(edit: impl FnOnce(&mut RawMap)) -> HeroDetailsSetup {
    let mut m = map();
    edit(&mut m);
    import(&m, &SOD).unwrap().heroes[0].details.clone()
}

#[test]
fn a_placed_heros_customisations_and_formation_carry_over() {
    let core = SOD.core();
    let artifact = |key: &str| core.artifact_by_key(key).unwrap();
    let none = customised(|_| {});
    assert_eq!(none, HeroDetailsSetup::default());
    let mut m = map();
    {
        let h = body(&mut m);
        h.experience = Some(999);
        h.primary_skills = Some([120, 0, 0, 3]);
        h.secondary_skills = Some(vec![(1, 2), (12, 3)]);
        // By position (E08 T49a): the axe in the right hand (3), the Ballista on the first
        // machine's (13), the book (17) and the Clover on SoD's fifth misc (18).
        let mut worn = vec![None; 19];
        worn[3] = Some(7);
        worn[13] = Some(4);
        worn[17] = Some(0);
        worn[18] = Some(46);
        h.artifacts = Some(Artifacts {
            worn,
            backpack: vec![Some(8), None],
        });
        // SoD's spells: bits 0 and 9.
        let mut spells = vec![0u8; 9];
        spells[0] = 1;
        spells[1] = 2;
        h.spells = Some(Bits(spells));
        h.formation = 1;
    }
    let setup = import(&m, &SOD).unwrap();
    let hero = &setup.heroes[0];
    assert_eq!(hero.formation, Formation::Tight);
    let d = &hero.details;
    assert_eq!(d.experience, Some(999));
    // Clamped as the original's load (0–99; Spell Power and Knowledge at least 1).
    assert_eq!(d.primary, Some([99, 0, 1, 3]));
    assert_eq!(
        d.skills,
        Some(vec![
            (core.skill_by_key("base:archery").unwrap(), 2),
            (core.skill_by_key("base:necromancy").unwrap(), 3),
        ])
    );
    let artifacts = d.artifacts.as_ref().unwrap();
    // The content's positions: the fifth misc (13) before the machines (14 to 17) and the book
    // (18).
    assert_eq!(artifacts.worn.len(), 19);
    let worn: Vec<_> = (0..)
        .zip(&artifacts.worn)
        .filter_map(|(at, a)| a.map(|a| (at, a.artifact, a.spell)))
        .collect();
    assert_eq!(
        worn,
        [
            (3, artifact("base:centaurs_axe"), None),
            (13, artifact("base:clover_of_fortune"), None),
            (14, artifact("base:ballista"), None),
            (18, artifact("base:spell_book"), None),
        ]
    );
    let backpack: Vec<_> = artifacts.backpack.iter().map(|a| a.artifact).collect();
    assert_eq!(backpack, [artifact("base:blackshard_of_the_dead_knight")]);
    assert_eq!(
        d.spells,
        Some(vec![
            core.spell_by_key("base:summon_boat").unwrap(),
            core.spell_by_key("base:town_portal").unwrap(),
        ])
    );
    assert_eq!(setup.validate(&SOD), Ok(()));
}

/// The test map's hero wearing these (map entry, artifact code) pairs.
fn wearing(m: &mut RawMap, entries: &[(usize, u16)]) {
    let mut worn = vec![None; 19];
    for &(at, code) in entries {
        worn[at] = Some(code);
    }
    body(m).artifacts = Some(Artifacts {
        worn,
        backpack: vec![],
    });
}

// covers: HER-ART-001, HER-ART-007
#[test]
fn a_placed_heros_artifacts_keep_the_maps_positions() {
    // Every entry to its position: the head to the feet, misc 1 to 4, the machines and the book
    // in the map's order; SoD's fifth misc (the map's 18) before the machines (the content's 13).
    let map_codes: [u16; 19] = [
        19, 42, 33, 7, 13, 25, 37, 45, 32, 46, 47, 48, 49, 4, 5, 6, 3, 0, 50,
    ];
    let entries: Vec<(usize, u16)> = (0..).zip(map_codes).collect();
    let artifacts = customised(|m| wearing(m, &entries)).artifacts.unwrap();
    let core = SOD.core();
    let keys: Vec<Option<String>> = artifacts
        .worn
        .iter()
        .map(|a| a.map(|a| core.artifact(a.artifact).unwrap().key.to_string()))
        .collect();
    let content: [u16; 19] = [
        19, 42, 33, 7, 13, 25, 37, 45, 32, 46, 47, 48, 49, 50, 4, 5, 6, 3, 0,
    ];
    let expected: Vec<Option<String>> = content
        .iter()
        .map(|&code| Some(codes::ARTIFACTS[usize::from(code)].to_string()))
        .collect();
    assert_eq!(keys, expected);
    let mut m = map();
    wearing(&mut m, &entries);
    assert_eq!(import(&m, &SOD).unwrap().validate(&SOD), Ok(()));
    // An entry its position does not take is left out, in the map's order, as the original's
    // loader: a misc artifact on the left hand (the Tutorial's second hero's Shackles of War, 125),
    // Angelic Alliance (129) on the right hand after a helm on the head it locks; RoE's rules have
    // no fifth misc position (the original refuses it before SoD).
    let left = |entries: &[(usize, u16)], rules: &GameRuleset| {
        let mut m = map();
        wearing(&mut m, entries);
        let setup = import(&m, rules).unwrap();
        assert_eq!(setup.validate(rules), Ok(()));
        let artifacts = setup.heroes[0].details.artifacts.clone().unwrap();
        (0..)
            .zip(artifacts.worn)
            .filter_map(|(at, a)| a.map(|a| (at, a.artifact)))
            .collect::<Vec<(usize, _)>>()
    };
    assert_eq!(
        left(&[(4, 125), (9, 125)], &SOD),
        [(9, artifact(core, 125))]
    );
    assert_eq!(left(&[(0, 19), (3, 129)], &SOD), [(0, artifact(core, 19))]);
    assert_eq!(left(&[(3, 129), (5, 25)], &SOD), [(3, artifact(core, 129))]);
    // Titan's Thunder (135) on the right hand equips a Spell Book (0) on the empty book position
    // (E08 T49b, HER-ART-007): the map's own book entry, later in its order, finds it taken; a
    // hero wearing the Thunder without a book entry gets one all the same.
    let book = artifact(core, 0);
    let thunder = artifact(core, 135);
    assert_eq!(left(&[(3, 135)], &SOD), [(3, thunder), (18, book)]);
    assert_eq!(left(&[(3, 135), (17, 0)], &SOD), [(3, thunder), (18, book)]);
    let roe = rules(1);
    assert_eq!(left(&[(18, 46)], &roe), []);
    assert_eq!(left(&[(17, 0)], &roe), [(17, artifact(roe.core(), 0))]);
    assert_eq!(
        roe.core()
            .artifact_slots()
            .iter()
            .map(|s| usize::from(s.count))
            .sum::<usize>(),
        18
    );
}

/// An artifact of `core` by its map code.
fn artifact(core: &h3_content::Ruleset, code: u16) -> h3_content::ArtifactTypeId {
    core.artifact_by_key(codes::ARTIFACTS[usize::from(code)])
        .unwrap()
}

// covers: GAM-BAT-010, GAM-BAT-011
#[test]
fn a_maps_player_kinds_and_object_classes_carry_over() {
    // A computer-only blue beside red, whom a human may play.
    let mut m = map();
    m.players[1] = Player {
        human: false,
        computer: true,
        details: m.players[0].details.clone(),
    };
    let setup = import(&m, &SOD).unwrap();
    let player = |colour, human| PlayerSetup { colour, human };
    assert_eq!(setup.players, [player(0, true), player(1, false)]);
    // Each class's scenery (`codes::scenery`, 0x40FCA0's and 0x41ADB0's tables).
    let kinds = [
        (114, Scenery::Tree),
        (115, Scenery::Tree),
        (135, Scenery::Tree),
        (137, Scenery::Tree),
        (155, Scenery::Tree),
        (159, Scenery::Tree),
        (160, Scenery::Tree),
        (123, Scenery::Mountain),
        (133, Scenery::Mountain),
        (134, Scenery::Mountain),
        (136, Scenery::Mountain),
        (157, Scenery::Mountain),
        (158, Scenery::Mountain),
        (119, Scenery::DeadVegetation),
        (17, Scenery::Ownable),
        (20, Scenery::Ownable),
        (33, Scenery::Ownable),
        (42, Scenery::Ownable),
        (53, Scenery::Ownable),
        (77, Scenery::Ownable),
        (87, Scenery::Ownable),
        (116, Scenery::Other),
        (124, Scenery::Other),
        (161, Scenery::Other),
        (216, Scenery::Other),
    ];
    for (class, scenery) in kinds {
        let mut m = map();
        m.templates[0].class = class;
        let objects = import_map(&m, &SOD).unwrap().objects;
        assert_eq!(objects[0].scenery, scenery, "class {class}");
    }
}

// covers: GAM-BAT-012
#[test]
fn a_placed_heros_sex_is_the_bodys_else_the_map_wide_entrys() {
    let sex = |own: u8, entry: Option<u8>| {
        customised(|m| {
            body(m).sex = own;
            if let Some(code) = entry {
                m.custom_heroes = vec![CustomHero {
                    hero_type: 0,
                    sex: code,
                    ..CustomHero::default()
                }];
            }
        })
        .sex
    };
    assert_eq!(sex(0xFF, None), None);
    assert_eq!(sex(1, None), Some(HeroSex::Female));
    assert_eq!(sex(0, Some(1)), Some(HeroSex::Male));
    assert_eq!(sex(0xFF, Some(1)), Some(HeroSex::Female));
    assert_eq!(sex(0xFF, Some(0)), Some(HeroSex::Male));
    // Any other code is the type's own.
    assert_eq!(sex(2, Some(0xFF)), None);
}

#[test]
fn the_body_wins_over_the_hero_types_map_wide_entry() {
    let core = SOD.core();
    let entry = CustomHero {
        hero_type: 0,
        experience: Some(500),
        secondary_skills: Some(vec![(0, 1)]),
        spells: Some(Bits(vec![1, 0, 0, 0, 0, 0, 0, 0, 0])),
        primary_skills: Some([1, 2, 3, 4]),
        sex: 0xFF,
        ..CustomHero::default()
    };
    let d = customised(|m| m.custom_heroes = vec![entry.clone()]);
    assert_eq!(d.experience, Some(500));
    assert_eq!(d.primary, Some([1, 2, 3, 4]));
    assert_eq!(
        d.skills,
        Some(vec![(core.skill_by_key("base:pathfinding").unwrap(), 1)])
    );
    assert_eq!(d.spells.map(|s| s.len()), Some(1));
    let d = customised(|m| {
        m.custom_heroes = vec![entry.clone()];
        let h = body(m);
        h.experience = Some(0);
        h.primary_skills = Some([5, 5, 5, 5]);
        h.spells = Some(Bits(vec![0; 9]));
    });
    assert_eq!(d.experience, Some(0), "SoD's flag makes 0 an experience");
    assert_eq!(d.primary, Some([5, 5, 5, 5]));
    assert_eq!(d.spells, Some(vec![]));
}

#[test]
fn before_sod_experience_0_and_the_spell_byte_mean_the_types() {
    let roe = |experience| {
        customised(|m| {
            m.header.version = Version::Ab;
            body(m).experience = Some(experience);
        })
    };
    assert_eq!(roe(0).experience, None);
    assert_eq!(roe(40).experience, Some(40));
    let ab = |spell| {
        customised(|m| {
            m.header.version = Version::Ab;
            body(m).spell = Some(spell);
        })
        .spells
    };
    assert_eq!(ab(0xFE), None);
    assert_eq!(ab(0xFF), Some(vec![]));
    // Another spell beside the type's own (Orrin has none); a hero type with a spell keeps it.
    let core = SOD.core();
    assert_eq!(
        ab(9),
        Some(vec![core.spell_by_key("base:town_portal").unwrap()])
    );
    let (caster, own) = (0u8..)
        .zip(codes::HERO_TYPES)
        .find_map(|(code, key)| {
            let spells = &core.hero_type(core.hero_type_by_key(key)?)?.spells;
            (!spells.is_empty()).then(|| (code, spells.clone()))
        })
        .unwrap();
    let with = |spell| {
        customised(|m| {
            m.header.version = Version::Ab;
            let h = body(m);
            h.hero_type = caster;
            h.spell = Some(spell);
        })
        .spells
    };
    let mut expected = own.clone();
    expected.push(core.spell_by_key("base:town_portal").unwrap());
    assert_eq!(with(9), Some(expected));
    let code = codes::SPELLS
        .iter()
        .position(|k| core.spell_by_key(k) == Some(own[0]))
        .unwrap();
    assert_eq!(
        with(u8::try_from(code).unwrap()),
        Some(own),
        "the type's own spell once"
    );
}

#[test]
fn a_heros_experience_passes_as_it_is_and_unknown_codes_are_refused() {
    // E08 T29: a custom experience of a level above 1, the body's or the map-wide entry's,
    // imports as it is; the game gives its levels at load (GAM-SET-002).
    let experience = |edit: fn(&mut RawMap)| customised(edit).experience;
    assert_eq!(experience(|m| body(m).experience = Some(1000)), Some(1000));
    assert_eq!(
        experience(|m| {
            m.custom_heroes = vec![CustomHero {
                hero_type: 0,
                experience: Some(5_000_000),
                ..CustomHero::default()
            }];
        }),
        Some(5_000_000)
    );
    assert_eq!(
        import_err(|m| body(m).secondary_skills = Some(vec![(28, 1)])),
        ImportError::UnknownCode {
            what: "secondary skill",
            code: 28
        }
    );
    assert_eq!(
        import_err(|m| body(m).spells = Some(Bits(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x08]))),
        ImportError::UnknownCode {
            what: "spell",
            code: 83
        }
    );
    assert_eq!(
        import_err(|m| {
            body(m).artifacts = Some(Artifacts {
                worn: vec![Some(141)],
                backpack: vec![],
            });
        }),
        ImportError::UnknownCode {
            what: "artifact",
            code: 141
        }
    );
    // An AB artifact with RoE's rules.
    let mut m = map();
    body(&mut m).artifacts = Some(Artifacts {
        worn: vec![],
        backpack: vec![Some(128)],
    });
    assert_eq!(
        import(&m, &rules(1)),
        Err(ImportError::MissingContent("ab:armageddons_blade"))
    );
}

// covers: HER-ADV-002
/// The original packs' Diplomacy gives a join bonus of its level (E08 T20a1, the harvester's
/// gap-fill) beside its surrender discount: what the wandering monsters' answer will read.
#[test]
fn diplomacy_gives_a_join_bonus_of_its_level() {
    let core = SOD.core();
    let diplomacy = core
        .skill(core.skill_by_key("base:diplomacy").unwrap())
        .unwrap();
    for level in 1..=3u8 {
        let effects = diplomacy.effects(level);
        assert!(
            effects
                .iter()
                .any(|e| matches!(*e, HeroEffect::JoinBonus { level: l } if l == level)),
            "{effects:?}"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, HeroEffect::SurrenderDiscount { .. }))
        );
    }
}
