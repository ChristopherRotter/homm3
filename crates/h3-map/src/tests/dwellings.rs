//! The experiment writer's increment 10 objects and the map's teams (E08 T89a: batch E10's maps;
//! increment 10's design note §1, §4, §16, survey `docs/rules/surveys/E08-dwellings.md` §1, §3.1,
//! §7): dwellings of one creature (17) and of several (20) with their owners, the Refugee Camp
//! (78), the random dwellings (216 to 218) not tied to a town with their faction sets and levels,
//! and the teams, read back as written in SoD and RoE; what the writer refuses; and what the
//! import makes of them until T93 types them.

use super::economy::{hero, read_back};
use super::import::SOD;
use crate::import;
use crate::model::{Bits, Body, RandomDwelling};
use crate::writer as w;

/// The game's `Objects.txt` lines of the classes written here: a Guardhouse (17.56), the Dragon
/// Cliffs (17.24), the Golem Factory (20.1), the Elemental Conflux (20.0), a Refugee Camp (78),
/// a random dwelling (216), one of level 3 (217.2) and one of Inferno's (218.3).
const LINES: [&str; 8] = [
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
fn placed(m: &crate::RawMap) -> Vec<(u32, u32, (u8, u8))> {
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

#[test]
fn the_dwellings_map_imports_as_the_import_reads_it_today() {
    // Until T93 types the classes (increment 10's design note §4), the import places each as an
    // untyped object of its template, unowned.
    let setup = import(&read_back(&w::write(&dwellings_map()).unwrap()), &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    let placed: Vec<_> = setup
        .map
        .objects
        .iter()
        .map(|o| (o.object_type, o.owner, (o.anchor.x, o.anchor.y)))
        .collect();
    let tiles = [
        (6, 4),
        (12, 4),
        (18, 4),
        (24, 4),
        (6, 10),
        (12, 10),
        (18, 10),
        (24, 10),
    ];
    assert_eq!(placed, tiles.map(|tile| (None, None, tile)));
}
