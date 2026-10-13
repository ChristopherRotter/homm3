//! Tests of the import (E08 T5a, T6a) with the original packs' rules: the static layer, the
//! players and placed heroes, each typed error and unsupported case, the code tables against the
//! packs, and parse, import and validation together on arbitrary bytes.

use std::sync::{Arc, LazyLock};

use h3_adventure::{Pos as TilePos, TileMask};
use h3_contracts::{ArmyStack, PlayerId};
use h3_game::{GameRuleset, HeroArmy, PlayerSetup};
use h3_pack_adventure::{ORIGINAL_PACKS, compile};
use proptest::prelude::*;

use super::encode::encode;
use super::sample;
use super::strategy::{raw_map, version};
use crate::import::codes;
use crate::model::{
    Body, HeroBody, MainTown, Message, Object, Pos, RawMap, Stack, Template, Text, Tile, Version,
};
use crate::{ImportError, Limits, Unsupported, import, import_map, parse};

/// The rules of RoE to SoD, or of RoE alone.
fn rules(packs: usize) -> GameRuleset {
    let (core, part) = compile(&ORIGINAL_PACKS[..packs], "base:faithful").unwrap();
    GameRuleset::new(Arc::new(core), Arc::new(part)).unwrap()
}

static SOD: LazyLock<GameRuleset> = LazyLock::new(|| rules(3));

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
    assert_eq!(setup.players, [PlayerSetup { colour: 0 }]);
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
    let blocker = s.objects[0];
    assert_eq!(blocker.anchor, TilePos::new(2, 2, 0));
    assert_eq!(blocker.blocked, TileMask::from_bits(0b11));
    assert_eq!(blocker.visitable, TileMask::from_bits(1 << 8));
    assert_eq!((blocker.object_type, blocker.amount), (None, None));
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
    let part = SOD.adventure();
    for (class, subclass, key) in &cases {
        let body = if matches!(class, 76 | 79) {
            pile_body(0, None)
        } else {
            Body::None
        };
        let setup = import(&typed_map(*class, *subclass, body), &SOD).unwrap();
        let object = setup.map.objects[0];
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

// covers: ADV-OBJ-002
#[test]
fn a_piles_fixed_amount_and_message_carry_over_and_guards_are_unsupported() {
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
    // A guards record, even an empty one, makes the pile unsupported; its static layer keeps it
    // as a blocker.
    for guards in [
        vec![None; 7],
        vec![Some(Stack {
            creature: 0,
            count: 5,
        })],
    ] {
        for class in [76, 79] {
            let message = Message {
                guards: Some(guards.clone()),
                ..note.clone()
            };
            let m = typed_map(class, 6, pile_body(4, Some(message)));
            assert_eq!(
                import(&m, &SOD),
                Err(ImportError::Unsupported(Unsupported::GuardedPile(0)))
            );
            let layer = import_map(&m, &SOD).unwrap();
            assert_eq!(
                (layer.objects[0].object_type, layer.objects[0].amount),
                (None, None)
            );
        }
    }
    // A fixed amount the setup rejects passes the import: validation refuses it.
    let m = typed_map(79, 0, pile_body(u32::MAX, None));
    let setup = import(&m, &SOD).unwrap();
    assert_eq!(setup.map.objects[0].amount, Some(u32::MAX));
    assert!(setup.validate(&SOD).is_err());
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
            "a random stack in a hero's army",
        ),
        (Unsupported::HeroInTown(0), "a hero standing in a town"),
        (Unsupported::GuardedPile(0), "a guarded resource pile"),
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
