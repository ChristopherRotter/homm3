//! The experiment writer's increment 10 objects and the map's teams (E08 T89a: batch E10's maps;
//! increment 10's design note §1, §4, §16, survey `docs/rules/surveys/E08-dwellings.md` §1, §3.1,
//! §7): dwellings of one creature (17) and of several (20) with their owners, the Refugee Camp
//! (78), the random dwellings (216 to 218) not tied to a town with their faction sets and levels,
//! and the teams, read back as written in SoD and RoE; what the writer refuses; and, since E08
//! T93b, what the import makes of them (increment 10's design note §4) and what it refuses.

use h3_adventure::{Holds, Scenery};
use h3_contracts::{PlayerId, TownId};

use super::economy::{hero, read_back};
use super::import::{self as raw_import, ROE, SOD, object};
use crate::import::codes;
use crate::model::{Bits, Body, RandomDwelling, RawMap, TownBody};
use crate::writer as w;
use crate::{ImportError, import};

/// The game's `Objects.txt` lines of the classes written here: a Guardhouse (17.56), the Dragon
/// Cliffs (17.24), the Golem Factory (20.1), the Elemental Conflux (20.0), a Refugee Camp (78),
/// a random dwelling (216), one of level 3 (217.2) and one of Inferno's (218.3).
pub(super) const LINES: [&str; 8] = [
    "AVGpike0.def 001111111011111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 56 0 0",
    "AVGgdrg0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 17 24 0 0",
    "AVGgolm0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 20 1 0 0",
    "AVGelem0.def 000111110001111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 20 0 0 0",
    "AVGrefg0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 78 0 0 0",
    "avrcgen0.def 001111110011111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 011111111 216 0 0 0",
    "avrcgen3.def 001111110011111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 011111111 217 2 0 0",
    "avrcgn03.def 001111110011111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 011111111 218 3 0 0",
];

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

fn owned(tile: (u8, u8), line: &str, body: w::OwnedBody) -> w::MapObject {
    w::MapObject::Owned(w::OwnedObject {
        tile,
        template: template(line),
        body,
    })
}

/// A random dwelling's body.
fn random(owner: Option<u8>, factions: Option<u16>, levels: Option<(u8, u8)>) -> w::OwnedBody {
    w::OwnedBody::RandomDwelling {
        owner,
        factions,
        levels,
    }
}

/// Batch E10's teams: Red and Blue allied (row E10-6), the colours nobody plays on a second team.
const TEAMS: w::Teams = w::Teams {
    count: 2,
    of_player: [0, 0, 1, 1, 1, 1, 1, 1],
};

/// A 36 × 36 grass map, Red and Blue allied, each with a hero: a Red Guardhouse, the neutral
/// Dragon Cliffs, a Blue Golem Factory, a neutral Elemental Conflux, a Refugee Camp, a neutral
/// random dwelling of Castle's level 7 (E10-7's Angel Portal), a neutral one of level 3 of the
/// Tower's (E10-7's Golem Factory) and a Red one of Inferno's levels 1 to 3.
fn dwellings_map() -> w::Map {
    use w::OwnedBody::Owner;
    let [pike, dragon, golem, conflux, _, angel, level_3, inferno] = LINES;
    let camp = template(LINES[4]);
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1,
    });
    players[1] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << 6,
    });
    w::Map {
        name: "Dwellings".into(),
        description: "E08 T89a".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![],
        roads: vec![],
        players,
        objects: vec![
            hero((2, 2), 0, 0),
            hero((2, 30), 1, 1),
            owned((6, 4), pike, Owner(Some(0))),
            owned((12, 4), dragon, Owner(None)),
            owned((18, 4), golem, Owner(Some(1))),
            owned((24, 4), conflux, Owner(None)),
            w::MapObject::Plain(w::Plain {
                tile: (6, 10),
                template: camp,
            }),
            owned((12, 10), angel, random(None, Some(1), Some((6, 6)))),
            owned((18, 10), level_3, random(None, Some(1 << 2), None)),
            owned((24, 10), inferno, random(Some(0), None, Some((0, 2)))),
        ],
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: Vec::new(),
        teams: Some(TEAMS),
    }
}

/// The objects' classes, subclasses and positions, the heroes left out.
fn placed(m: &RawMap) -> Vec<(u32, u32, (u8, u8))> {
    m.objects[2..]
        .iter()
        .map(|o| {
            let t = &m.templates[usize::try_from(o.template).unwrap()];
            (t.class, t.subclass, (o.pos.x, o.pos.y))
        })
        .collect()
}

#[test]
fn increment_10_s_objects_and_the_teams_read_back_as_written() {
    let m = read_back(&w::write(&dwellings_map()).unwrap());
    assert_eq!(
        placed(&m),
        [
            (17, 56, (6, 4)),
            (17, 24, (12, 4)),
            (20, 1, (18, 4)),
            (20, 0, (24, 4)),
            (78, 0, (6, 10)),
            (216, 0, (12, 10)),
            (217, 2, (18, 10)),
            (218, 3, (24, 10)),
        ]
    );
    let bodies: Vec<&Body> = m.objects[2..].iter().map(|o| &o.body).collect();
    assert_eq!(*bodies[0], Body::Owned { owner: 0 });
    assert_eq!(*bodies[1], Body::Owned { owner: 0xff });
    assert_eq!(*bodies[2], Body::Owned { owner: 1 });
    assert_eq!(*bodies[3], Body::Owned { owner: 0xff });
    assert_eq!(*bodies[4], Body::None);
    // No town (identifier 0): the set follows, SoD's two bytes.
    assert_eq!(
        *bodies[5],
        Body::RandomDwelling(RandomDwelling {
            owner: 0xff,
            faction: Some((0, Some(Bits(vec![1, 0])))),
            levels: Some((6, 6)),
        })
    );
    assert_eq!(
        *bodies[6],
        Body::RandomDwelling(RandomDwelling {
            owner: 0xff,
            faction: Some((0, Some(Bits(vec![4, 0])))),
            levels: None,
        })
    );
    assert_eq!(
        *bodies[7],
        Body::RandomDwelling(RandomDwelling {
            owner: 0,
            faction: None,
            levels: Some((0, 2)),
        })
    );
    assert_eq!(m.teams, Some(TEAMS));
    // The Conflux's bit is the second byte's first; no teams is a count of 0.
    let mut map = dwellings_map();
    map.objects[7] = owned((12, 10), LINES[5], random(None, Some(0x1ff), Some((0, 6))));
    map.teams = None;
    let m = read_back(&w::write(&map).unwrap());
    assert_eq!(
        m.objects[7].body,
        Body::RandomDwelling(RandomDwelling {
            owner: 0xff,
            faction: Some((0, Some(Bits(vec![0xff, 1])))),
            levels: Some((0, 6)),
        })
    );
    assert_eq!(m.teams, None);
}

#[test]
fn an_roe_map_holds_the_dwellings_the_camp_and_the_teams() {
    // RoE has no random dwelling (the census, survey §1); its dwellings, camps and teams read
    // back as SoD's.
    let mut map = dwellings_map();
    map.objects.truncate(7);
    let m = read_back(&w::write_in(&map, w::Format::RoE).unwrap());
    assert_eq!(
        placed(&m),
        [
            (17, 56, (6, 4)),
            (17, 24, (12, 4)),
            (20, 1, (18, 4)),
            (20, 0, (24, 4)),
            (78, 0, (6, 10)),
        ]
    );
    assert_eq!(m.objects[4].body, Body::Owned { owner: 1 });
    assert_eq!(m.teams, Some(TEAMS));
}

#[test]
fn the_writer_refuses_what_the_game_cannot_load() {
    use w::OwnedBody::Owner;
    let [pike, _, golem, _, _, angel, level_3, inferno] = LINES;
    let refused_in = |map: w::Map, what: &str| match w::write(&map) {
        Err(w::WriteError::OutOfRange { what: got, .. }) => assert_eq!(got, what),
        other => panic!("{what}: {other:?}"),
    };
    let refused = |object: w::MapObject, what: &str| {
        let mut map = dwellings_map();
        map.objects.push(object);
        refused_in(map, what);
    };
    // Classes 18 and 19 (the original stops at them, A-36) and the subclasses past the types.
    for (from, to) in [(" 17 56 ", " 18 0 "), (" 17 56 ", " 17 80 ")] {
        let line = pike.replace(from, to);
        refused(owned((20, 30), &line, Owner(None)), "owned object class");
    }
    let line = golem.replace(" 20 1 ", " 20 2 ");
    refused(owned((20, 30), &line, Owner(None)), "owned object class");
    refused(
        owned((20, 30), pike, Owner(Some(2))),
        "object owner (no playable player)",
    );
    refused(
        owned((20, 30), inferno, random(Some(3), None, Some((0, 0)))),
        "object owner (no playable player)",
    );
    // A random dwelling's body on another class, a level past 7 or a faction past Conflux.
    let all = Some(1);
    refused(
        owned((20, 30), pike, random(None, all, Some((0, 6)))),
        "random dwelling class",
    );
    let line = level_3.replace(" 217 2 ", " 217 7 ");
    refused(
        owned((20, 30), &line, random(None, all, None)),
        "random dwelling class",
    );
    let line = inferno.replace(" 218 3 ", " 218 9 ");
    refused(
        owned((20, 30), &line, random(None, None, Some((0, 6)))),
        "random dwelling class",
    );
    // The set where the class has one (216, 217), at least one faction of the nine (A-34).
    for (line, factions, levels) in [(angel, None, Some((0, 6))), (inferno, all, Some((0, 6)))] {
        refused(
            owned((20, 30), line, random(None, factions, levels)),
            "random dwelling factions (216 and 217's)",
        );
    }
    for set in [0, 0x200] {
        refused(
            owned((20, 30), angel, random(None, Some(set), Some((0, 6)))),
            "random dwelling factions",
        );
    }
    // The levels where the class has them (216, 218), 1 to 7, the lowest not above (A-36).
    for (line, factions, levels) in [(angel, all, None), (level_3, all, Some((0, 6)))] {
        refused(
            owned((20, 30), line, random(None, factions, levels)),
            "random dwelling levels (216 and 218's)",
        );
    }
    for levels in [(3, 2), (0, 7)] {
        refused(
            owned((20, 30), inferno, random(None, None, Some(levels))),
            "random dwelling levels",
        );
    }
    // Teams: one to eight, each colour's below the count.
    for count in [0, 9] {
        let mut map = dwellings_map();
        map.teams = Some(w::Teams {
            count,
            of_player: [0; 8],
        });
        refused_in(map, "team count");
    }
    let mut map = dwellings_map();
    map.teams = Some(w::Teams {
        count: 2,
        of_player: [0, 1, 2, 0, 0, 0, 0, 0],
    });
    refused_in(map, "team (past the count)");
    // RoE: no random dwelling.
    match w::write_in(&dwellings_map(), w::Format::RoE) {
        Err(w::WriteError::OutOfRange { what, .. }) => {
            assert_eq!(what, "random dwelling in an RoE map");
        }
        other => panic!("{other:?}"),
    }
}

/// The dwellings map read back (SoD).
fn raw() -> RawMap {
    read_back(&w::write(&dwellings_map()).unwrap())
}

/// The map with the object at `index` of this class and subclass (its own template).
fn retyped(raw: &RawMap, index: usize, class: u32, subclass: u32) -> RawMap {
    let mut map = raw.clone();
    let t = usize::try_from(map.objects[index].template).unwrap();
    map.templates[t].class = class;
    map.templates[t].subclass = subclass;
    map
}

/// The map with the object at `index` holding this random dwelling's body.
fn with_body(
    raw: &RawMap,
    index: usize,
    owner: u32,
    faction: Option<(u32, Option<Bits>)>,
    levels: Option<(u8, u8)>,
) -> RawMap {
    let mut map = raw.clone();
    map.objects[index].body = Body::RandomDwelling(RandomDwelling {
        owner,
        faction,
        levels,
    });
    map
}

/// A random dwelling's holds in SoD's rules: the factions by their codes.
fn holds(codes: &[usize], levels: (u8, u8)) -> Holds {
    let factions = codes
        .iter()
        .map(|&code| SOD.core().faction_by_key(codes::FACTIONS[code]).unwrap())
        .collect();
    Holds::RandomDwelling { factions, levels }
}

// covers: ADV-DWL-001, ADV-DWL-005, ADV-RFG-001, ADV-OWN-002, GAM-BAT-011
#[test]
fn the_dwellings_map_imports_typed() {
    // E08 T93b (increment 10's design note §4): the dwellings by class and subclass, the camp and
    // the random dwellings with their owners, faction sets and level ranges.
    let setup = import(&raw(), &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    let part = SOD.adventure();
    let types: Vec<_> = setup.map.objects.iter().map(|o| o.object_type).collect();
    let keys = [
        "guardhouse",
        "dragon_cliffs",
        "golem_factory",
        "elemental_conflux",
        "refugee_camp",
        "random_dwelling",
        "random_dwelling",
        "random_dwelling",
    ]
    .map(|k| part.object_type_by_key(&format!("base:{k}")));
    assert!(keys.iter().all(Option::is_some));
    assert_eq!(types, keys);
    let owners: Vec<_> = setup.map.objects.iter().map(|o| o.owner).collect();
    let (red, blue) = (Some(PlayerId::new(0)), Some(PlayerId::new(1)));
    assert_eq!(owners, [red, None, blue, None, None, None, None, red]);
    let held: Vec<_> = setup.map.objects.iter().map(|o| o.holds.clone()).collect();
    let mut expected = vec![Holds::Nothing; 8];
    expected[5] = holds(&[0], (7, 7));
    expected[6] = holds(&[2], (3, 3));
    expected[7] = holds(&[3], (1, 3));
    assert_eq!(held, expected);
    // Every one but the camp is ownable scenery (GAM-BAT-011: 216 to 218 are 17 or 20 then).
    let scenery: Vec<_> = setup.map.objects.iter().map(|o| o.scenery).collect();
    let mut expected = [Scenery::Ownable; 8];
    expected[4] = Scenery::Other;
    assert_eq!(scenery, expected);
    // The table's last subclasses, a level 7 of 217 and Conflux by 218, and a set of several
    // factions in the core's order with the full level range.
    assert!(import(&retyped(&raw(), 2, 17, 79), &SOD).is_ok());
    assert!(import(&retyped(&raw(), 4, 20, 1), &SOD).is_ok());
    let placed =
        |map: &RawMap, index: usize| import(map, &SOD).unwrap().map.objects[index].holds.clone();
    assert_eq!(placed(&retyped(&raw(), 8, 217, 6), 6), holds(&[2], (7, 7)));
    assert_eq!(placed(&retyped(&raw(), 9, 218, 8), 7), holds(&[8], (1, 3)));
    let several = with_body(
        &raw(),
        7,
        0xFF,
        Some((0, Some(Bits(vec![0x85, 1])))),
        Some((0, 6)),
    );
    assert_eq!(placed(&several, 5), holds(&[0, 2, 7, 8], (1, 7)));
    // E08 T97a's audit: the owners of the random classes the map leaves neutral or owned, a blue
    // 216 and a neutral 218.
    let owner = |map: &RawMap, index: usize| import(map, &SOD).unwrap().map.objects[index].owner;
    let castle = Some((0, Some(Bits(vec![1, 0]))));
    assert_eq!(
        owner(&with_body(&raw(), 7, 1, castle, Some((6, 6))), 5),
        blue
    );
    assert_eq!(
        owner(&with_body(&raw(), 9, 0xFF, None, Some((0, 2))), 7),
        None
    );
}

// covers: ADV-DWL-005
#[test]
fn a_random_dwelling_tied_to_a_town_holds_the_last_town_with_its_identifier() {
    // E09 T11 (map import v20; ADV-STUB-13 before): an identifier a town or a random town of the
    // map has types the object as a random dwelling holding that town's `TownId` (its place among
    // the towns, 77 and 98, in object order) and its levels, owned as any random dwelling; the last
    // town with the identifier, as the original searches from the end (`random_dwellings_resolve`
    // 0x502B60). An RoE town has no identifier, and an object of a town's class without a town's
    // body names none: both take their places. Its levels and owner are checked as any random
    // dwelling's (E08 T97b1; A-36, ADV-OWN-002).
    let mut map = with_body(&raw(), 7, 0, Some((5, None)), Some((2, 4)));
    let town = |identifier| {
        Body::Town(Box::new(TownBody {
            identifier,
            ..TownBody::default()
        }))
    };
    let first = u32::try_from(map.templates.len()).unwrap();
    map.templates.extend(
        [98, 77, 77, 77, 98].map(|class| raw_import::template(class, &[(0, 0)], &[(0, 0)])),
    );
    map.objects.extend([
        object(30, 20, first, town(Some(5))),
        object(30, 24, first + 1, town(None)),
        object(30, 28, first + 2, Body::None),
        object(33, 20, first + 3, town(Some(5))),
        object(33, 24, first + 4, town(Some(6))),
    ]);
    let tied_to = |map: &RawMap, identifier| {
        let mut map = map.clone();
        map.objects[7].body = with_body(&raw(), 7, 0, Some((identifier, None)), Some((2, 4)))
            .objects[7]
            .body
            .clone();
        let setup = import(&map, &SOD).unwrap();
        setup.map.objects[5].clone()
    };
    let dwelling = tied_to(&map, 5);
    let random = SOD.adventure().object_type_by_key("base:random_dwelling");
    assert_eq!(
        (dwelling.object_type, dwelling.owner, &dwelling.holds),
        (
            random,
            Some(PlayerId::new(0)),
            &Holds::LinkedDwelling {
                town: TownId::new(3),
                levels: (3, 5)
            }
        )
    );
    assert_eq!(dwelling.scenery, Scenery::Ownable);
    assert_eq!(
        tied_to(&map, 6).holds,
        Holds::LinkedDwelling {
            town: TownId::new(4),
            levels: (3, 5)
        }
    );
    // Its levels past the seventh or the lowest above the highest, then an owner nobody plays
    // (colour 5): refused, as an untied one's.
    let tied = |map: &RawMap, owner, levels| {
        let mut map = map.clone();
        map.objects[7].body = with_body(&raw(), 7, owner, Some((5, None)), levels).objects[7]
            .body
            .clone();
        import(&map, &SOD)
    };
    for levels in [(9, 9), (4, 2)] {
        assert_eq!(
            tied(&map, 5, Some(levels)).unwrap_err(),
            ImportError::DwellingLevels(7)
        );
    }
    assert_eq!(
        tied(&map, 5, Some((2, 4))).unwrap_err(),
        ImportError::ObjectOwner(7)
    );
    // Class 217's level is its subclass (2: level 3).
    let setup = tied(&retyped(&map, 7, 217, 2), 0xFF, None).unwrap();
    assert_eq!(
        setup.map.objects[5].holds,
        Holds::LinkedDwelling {
            town: TownId::new(3),
            levels: (3, 3)
        }
    );
    // The other identifiers name no town.
    let mut other = map.clone();
    other.objects[7].body = with_body(&raw(), 7, 0xFF, Some((7, None)), Some((0, 0))).objects[7]
        .body
        .clone();
    assert_eq!(
        import(&other, &SOD).unwrap_err(),
        ImportError::DwellingTown(7)
    );
}

// covers: ADV-DWL-001, ADV-DWL-005, ADV-OWN-002
#[test]
fn the_import_refuses_what_the_original_cannot_load() {
    // E08 T93b (increment 10's design note §4): input bounds as A-31 (ADV-QUIRK-36 to -38); errors
    // name the map's object index (the heroes 0 and 1 included).
    let raw = raw();
    let refused = |map: &RawMap| import(map, &SOD).unwrap_err();
    // Classes 18 and 19 (A-36), subclasses past the tables and past 217's and 218's codes.
    assert_eq!(
        refused(&retyped(&raw, 2, 18, 0)),
        ImportError::DwellingClass(2)
    );
    assert_eq!(
        refused(&retyped(&raw, 3, 19, 0)),
        ImportError::DwellingClass(3)
    );
    assert_eq!(refused(&retyped(&raw, 2, 17, 80)), ImportError::Subclass(2));
    assert_eq!(refused(&retyped(&raw, 4, 20, 2)), ImportError::Subclass(4));
    assert_eq!(refused(&retyped(&raw, 8, 217, 7)), ImportError::Subclass(8));
    assert_eq!(refused(&retyped(&raw, 9, 218, 9)), ImportError::Subclass(9));
    // A town no town of the map has (A-35).
    let body = |faction, levels| with_body(&raw, 7, 0xFF, faction, levels);
    assert_eq!(
        refused(&body(Some((5, None)), Some((0, 6)))),
        ImportError::DwellingTown(7)
    );
    // A faction code past 8, an empty set (A-34), a faction the rules lack (RoE's: Conflux).
    assert_eq!(
        refused(&body(Some((0, Some(Bits(vec![1, 2])))), Some((0, 6)))),
        ImportError::UnknownCode {
            what: "faction",
            code: 9
        }
    );
    for set in [Some(Bits(vec![0, 0])), Some(Bits(vec![])), None] {
        assert_eq!(
            refused(&body(Some((0, set)), Some((0, 6)))),
            ImportError::DwellingFactions(7)
        );
    }
    assert_eq!(
        import(&body(Some((0, Some(Bits(vec![1, 1])))), Some((0, 6))), &ROE).unwrap_err(),
        ImportError::MissingContent("ab:conflux")
    );
    // A level past 6 or the lowest above the highest (A-36).
    for levels in [(7, 7), (0, 7), (3, 2)] {
        assert_eq!(
            refused(&body(Some((0, Some(Bits(vec![1, 0])))), Some(levels))),
            ImportError::DwellingLevels(7)
        );
    }
    // An owner nobody plays: a dwelling's and a random dwelling's.
    assert_eq!(
        refused(&with_body(&raw, 9, 5, None, Some((0, 2)))),
        ImportError::ObjectOwner(9)
    );
    let mut map = raw.clone();
    map.objects[2].body = Body::Owned { owner: 5 };
    assert_eq!(refused(&map), ImportError::ObjectOwner(2));
    // The order within one object: subclass, town, faction set, levels, owner.
    let both = |faction, levels| with_body(&raw, 8, 5, faction, levels);
    let mut map = both(Some((5, None)), None);
    let t = usize::try_from(map.objects[8].template).unwrap();
    map.templates[t].subclass = 7;
    assert_eq!(refused(&map), ImportError::Subclass(8));
    assert_eq!(
        refused(&both(Some((5, None)), Some((3, 2)))),
        ImportError::DwellingTown(8)
    );
    assert_eq!(
        refused(&both(Some((0, None)), Some((3, 2)))),
        ImportError::DwellingFactions(8)
    );
    assert_eq!(
        refused(&both(Some((0, Some(Bits(vec![1, 0])))), Some((3, 2)))),
        ImportError::DwellingLevels(8)
    );
    assert_eq!(
        refused(&both(Some((0, Some(Bits(vec![1, 0])))), Some((2, 3)))),
        ImportError::ObjectOwner(8)
    );
}

// covers: ADV-OWN-002, ADV-DWL-004
#[test]
fn a_dwelling_entered_from_water_is_imported() {
    // E08 T97b1 (survey `E08-dwellings.md` §17: a hero in a boat visits a dwelling as on land):
    // the Pikeman's dwelling (class 17) and Inferno's random dwelling (218), each with its
    // entrance on water, import as on land, typed and owned; the other ownable kinds stay
    // unsupported there (`an_ownable_object_entered_from_water_is_unsupported`).
    let mut map = raw();
    let before = import(&map, &SOD).unwrap();
    for index in [0, 7] {
        let dwelling = &before.map.objects[index];
        let (column, row) = dwelling.visitable.offsets().next().unwrap();
        let x = usize::from(dwelling.anchor.x) - usize::from(column);
        let y = usize::from(dwelling.anchor.y) - usize::from(row);
        map.terrain.tiles[y * 36 + x].terrain = 8;
    }
    let after = import(&map, &SOD).unwrap();
    assert_ne!(after.map.tiles, before.map.tiles);
    for index in [0, 7] {
        let (a, b) = (&after.map.objects[index], &before.map.objects[index]);
        assert_eq!(
            (a.object_type, a.owner, &a.holds),
            (b.object_type, b.owner, &b.holds)
        );
        assert!(a.object_type.is_some());
    }
}
