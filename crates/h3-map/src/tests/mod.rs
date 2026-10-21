//! Tests of the reader: the round trip of every model a version can hold, truncation, arbitrary
//! bytes, each typed error and limit, the gzip layer, and the experiment writer's maps; the
//! import in `import.rs`; random play on imported maps through `h3-game` in `play.rs` (E08 T8b2a),
//! what a viewer may learn there in `view.rs` (E08 T8b2b).

mod encode;
mod fixtures;
mod import;
mod play;
mod strategy;
mod view;

use proptest::prelude::*;

use encode::{Out, encode};
use strategy::{raw_map, version};

use crate::model::{
    Artifacts, Bits, Body, CustomHero, Header, Loss, Mission, Object, Player, PlayerDetails,
    PlayerHero, Pos, Quest, RawMap, Reward, SeerTask, Teams, Template, Terrain, Text, Tile,
    TimedEvent, TownBody, TownEvent, Version, Victory,
};
use crate::{Limits, ReadError, crc32, gunzip, parse, read};

/// A small playable map of `v`: red human, the rest unplayable, standard conditions.
fn sample(v: Version) -> RawMap {
    let mut players: [Player; 8] = Default::default();
    players[0] = Player {
        human: true,
        computer: true,
        details: Some(PlayerDetails {
            allowed_towns: 0xFF,
            heroes: if v == Version::RoE {
                Vec::new()
            } else {
                vec![PlayerHero {
                    hero_type: 7,
                    name: Text(b"Sevens".to_vec()),
                }]
            },
            ..PlayerDetails::default()
        }),
    };
    let sod = v == Version::Sod;
    RawMap {
        header: Header {
            version: v,
            any_players: true,
            size: 2,
            two_levels: false,
            name: Text(b"Sample".to_vec()),
            description: Text::default(),
            difficulty: 1,
            level_limit: 0,
        },
        players,
        victory: Victory::Standard,
        loss: Loss::Standard,
        teams: None,
        allowed_heroes: Bits(vec![0xFF; v.hero_types().div_ceil(8)]),
        placeholders: Vec::new(),
        disposed_heroes: Vec::new(),
        banned_artifacts: Bits(vec![
            0;
            if v == Version::RoE {
                0
            } else {
                v.artifacts().div_ceil(8)
            }
        ]),
        banned_spells: Bits(vec![0; if sod { 9 } else { 0 }]),
        banned_skills: Bits(vec![0; if sod { 4 } else { 0 }]),
        rumors: Vec::new(),
        custom_heroes: if sod {
            vec![CustomHero {
                hero_type: 3,
                secondary_skills: Some(vec![(1, 2)]),
                artifacts: Some(Artifacts {
                    worn: vec![None; v.artifact_slots()],
                    backpack: vec![Some(7)],
                }),
                sex: 0xFF,
                ..CustomHero::default()
            }]
        } else {
            Vec::new()
        },
        terrain: Terrain {
            size: 2,
            levels: 1,
            tiles: vec![Tile::default(); 4],
        },
        templates: Vec::new(),
        objects: Vec::new(),
        events: Vec::new(),
    }
}

/// The game's first Clover Field template (`Objects.txt`): passable everywhere, class 222.
const CLOVER_FIELD: &str = "AVXcf0.def 111111111111111111111111111111111111111111111111 \
     000000000000000000000000000000000000000000000000 111111111 011111111 222 0 0 1";

/// The land Cartographer's picture in `Objects.txt` (E08 T32a): it blocks its tile and the one
/// west of it, which is visited.
const CARTOGRAPHER: &str = "AVXmaps0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 010111111 13 1 0 0";

fn limits() -> Limits {
    Limits::default()
}

/// Expects `bytes` to fail with a [`ReadError::LimitExceeded`] on `field`.
fn assert_limit(bytes: &[u8], limits: &Limits, field: &str) {
    match parse(bytes, limits) {
        Err(ReadError::LimitExceeded { field: f, .. }) => assert_eq!(f, field),
        other => panic!("expected a limit on {field}, got {other:?}"),
    }
}

#[test]
fn samples_of_every_version_round_trip() {
    for v in [Version::RoE, Version::Ab, Version::Sod] {
        let m = sample(v);
        assert_eq!(parse(&encode(&m), &limits()), Ok(m));
    }
}

#[test]
fn zero_padding_is_accepted_and_anything_else_after_the_events_is_not() {
    let m = sample(Version::Sod);
    let mut bytes = encode(&m);
    let end = bytes.len();
    bytes.extend_from_slice(&[0; 124]);
    assert_eq!(parse(&bytes, &limits()), Ok(m));
    bytes[end + 7] = 1;
    assert_eq!(
        parse(&bytes, &limits()),
        Err(ReadError::TrailingBytes { offset: end })
    );
}

/// `sample(v)` with one template of `class` and one object of it carrying `body`.
fn with_object(v: Version, class: u32, body: Body) -> RawMap {
    let mut m = sample(v);
    m.templates = vec![Template {
        def: Text(b"obj.def".to_vec()),
        class,
        ..Template::default()
    }];
    m.objects = vec![Object {
        pos: Pos { x: 1, y: 1, z: 0 },
        template: 0,
        body,
    }];
    m
}

#[test]
fn objects_must_name_an_existing_template() {
    let mut m = with_object(Version::Ab, 124, Body::None);
    assert_eq!(parse(&encode(&m), &limits()), Ok(m.clone()));
    m.objects[0].template = 1;
    assert!(matches!(
        parse(&encode(&m), &limits()),
        Err(ReadError::InvalidValue {
            field: "object template",
            value: 1,
            ..
        })
    ));
}

#[test]
fn unknown_quest_and_reward_kinds_are_rejected() {
    let quest = Quest {
        mission: Mission::Hero(3),
        deadline: -1,
        first_visit: Text::default(),
        next_visit: Text::default(),
        completed: Text::default(),
    };
    let m = with_object(Version::Sod, 215, Body::QuestGuard(Some(quest.clone())));
    let mut bytes = encode(&m);
    // The quest guard's body ends the object: mission code and hero type, the deadline and three
    // empty strings, then the map's event count.
    let at = bytes.len() - 4 - (2 + 4 + 12);
    assert_eq!(bytes[at..at + 2], [8, 3]);
    bytes[at] = 10;
    assert!(matches!(
        parse(&bytes, &limits()),
        Err(ReadError::InvalidValue {
            field: "quest mission",
            value: 10,
            ..
        })
    ));
    let task = SeerTask {
        quest,
        reward: Reward::Spell(7),
    };
    let m = with_object(Version::Sod, 83, Body::SeerHut(Some(task)));
    let mut bytes = encode(&m);
    // The reward code and its spell come before two unused bytes and the event count.
    let at = bytes.len() - 4 - 2 - 2;
    assert_eq!(bytes[at..at + 2], [9, 7]);
    bytes[at] = 11;
    assert!(matches!(
        parse(&bytes, &limits()),
        Err(ReadError::InvalidValue {
            field: "seer hut reward",
            value: 11,
            ..
        })
    ));
}

#[test]
fn every_mission_and_reward_round_trips() {
    use crate::model::Stack;
    let stack = Stack {
        creature: 9,
        count: 4,
    };
    let missions = [
        Mission::Level(10),
        Mission::PrimarySkills([1, 2, 3, 4]),
        Mission::KillHero(77),
        Mission::KillCreature(78),
        Mission::Artifacts(vec![5, 6]),
        Mission::Army(vec![stack]),
        Mission::Resources([1, 2, 3, 4, 5, 6, 7]),
        Mission::Hero(12),
        Mission::Player(3),
    ];
    let rewards = [
        Reward::Nothing,
        Reward::Experience(500),
        Reward::Mana(10),
        Reward::Morale(-1),
        Reward::Luck(2),
        Reward::Resource {
            resource: 6,
            amount: 1000,
        },
        Reward::PrimarySkill {
            skill: 1,
            amount: 2,
        },
        Reward::SecondarySkill { skill: 7, level: 3 },
        Reward::Artifact(40),
        Reward::Spell(15),
        Reward::Creatures(stack),
    ];
    for v in [Version::Ab, Version::Sod] {
        for (mission, reward) in missions.iter().cycle().zip(&rewards) {
            let quest = Quest {
                mission: mission.clone(),
                deadline: 30,
                first_visit: Text(b"first".to_vec()),
                next_visit: Text(b"next".to_vec()),
                completed: Text(b"done".to_vec()),
            };
            let task = SeerTask {
                quest: quest.clone(),
                reward: *reward,
            };
            for m in [
                with_object(v, 83, Body::SeerHut(Some(task))),
                with_object(v, 215, Body::QuestGuard(Some(quest))),
            ] {
                assert_eq!(parse(&encode(&m), &limits()), Ok(m));
            }
        }
    }
}

#[test]
fn object_counts_are_bounded() {
    let mut m = with_object(Version::RoE, 124, Body::None);
    m.events = vec![TimedEvent::default(); 2];
    for e in &mut m.events {
        e.human = true;
    }
    let bytes = encode(&m);
    assert_eq!(parse(&bytes, &limits()), Ok(m));
    let tight = |f: fn(&mut Limits)| {
        let mut l = limits();
        f(&mut l);
        l
    };
    assert_limit(&bytes, &tight(|l| l.max_templates = 0), "templates");
    assert_limit(&bytes, &tight(|l| l.max_objects = 0), "objects");
    assert_limit(&bytes, &tight(|l| l.max_events = 1), "timed events");
    let town = TownBody {
        events: vec![TownEvent::default(); 2],
        banned_spells: Bits(vec![0; 9]),
        alignment: u8::MAX,
        ..TownBody::default()
    };
    let mut m = with_object(Version::RoE, 98, Body::Town(Box::new(town)));
    if let Body::Town(t) = &mut m.objects[0].body {
        for e in &mut t.events {
            e.event.human = true;
        }
    }
    let bytes = encode(&m);
    assert_eq!(parse(&bytes, &limits()), Ok(m));
    assert_limit(&bytes, &tight(|l| l.max_events = 1), "town events");
}

#[test]
fn unsupported_versions_are_rejected() {
    for code in [0x0D, 0x1D, 0x20, 0x33] {
        let mut bytes = encode(&sample(Version::Sod));
        bytes[..4].copy_from_slice(&u32::to_le_bytes(code));
        assert_eq!(
            parse(&bytes, &limits()),
            Err(ReadError::UnsupportedVersion(code))
        );
    }
    assert_eq!(Version::from_code(0x1D), None);
    assert_eq!(Version::from_code(Version::Ab.code()), Some(Version::Ab));
}

#[test]
fn the_map_size_is_bounded() {
    let mut m = sample(Version::Ab);
    m.header.size = 0;
    assert!(matches!(
        parse(&encode(&m), &limits()),
        Err(ReadError::InvalidValue {
            field: "map size",
            value: 0,
            offset: 5
        })
    ));
    m.header.size = 145;
    assert_limit(&encode(&m), &limits(), "map size");
    let small = Limits {
        max_size: 1,
        ..limits()
    };
    assert_limit(&encode(&sample(Version::RoE)), &small, "map size");
}

#[test]
fn strings_and_counts_are_bounded() {
    let mut m = sample(Version::Sod);
    m.header.name = Text(vec![b'a'; 10]);
    let tight = Limits {
        max_string: 9,
        ..limits()
    };
    assert_limit(&encode(&m), &tight, "map name");

    let mut m = sample(Version::Sod);
    m.rumors = vec![crate::model::Rumor::default(); 2];
    let tight = Limits {
        max_rumors: 1,
        ..limits()
    };
    assert_limit(&encode(&m), &tight, "rumors");

    let m = sample(Version::Sod);
    let tight = Limits {
        max_player_heroes: 0,
        ..limits()
    };
    assert_limit(&encode(&m), &tight, "player heroes");

    let mut m = sample(Version::Ab);
    m.placeholders = vec![1, 2];
    let tight = Limits {
        max_placeholders: 1,
        ..limits()
    };
    assert_limit(&encode(&m), &tight, "placeholders");

    let m = sample(Version::Sod);
    let tight = Limits {
        max_secondary_skills: 0,
        ..limits()
    };
    assert_limit(&encode(&m), &tight, "secondary skills");
    let tight = Limits {
        max_backpack: 0,
        ..limits()
    };
    assert_limit(&encode(&m), &tight, "backpack");
}

#[test]
fn a_count_beyond_the_bytes_left_is_truncation_not_allocation() {
    // A rumor count of 2^31 with a generous limit: the bytes left cannot hold it.
    let m = sample(Version::RoE);
    let mut bytes = encode(&m);
    let generous = Limits {
        max_rumors: u32::MAX,
        ..limits()
    };
    // The rumor count precedes four tiles and the template, object and event counts.
    let at = bytes.len() - 3 * 4 - 4 * 7 - 4;
    bytes[at..at + 4].copy_from_slice(&(1u32 << 31).to_le_bytes());
    assert_eq!(
        parse(&bytes, &generous),
        Err(ReadError::Truncated {
            field: "rumors",
            offset: at
        })
    );
}

#[test]
fn unknown_condition_kinds_are_rejected() {
    let m = sample(Version::RoE);
    let bytes = encode(&m);
    // The victory byte follows the header (10 + 10 name + 4 description + 1 difficulty) and
    // the players (red 8 bytes, seven unplayable ones 8 each); then loss and team count.
    let at = 25 + 8 + 7 * 8;
    assert_eq!(bytes[at..at + 3], [0xFF, 0xFF, 0]);
    let mut bad = bytes.clone();
    bad[at] = 11;
    assert!(matches!(
        parse(&bad, &limits()),
        Err(ReadError::InvalidValue {
            field: "victory condition",
            value: 11,
            ..
        })
    ));
    let mut bad = bytes;
    bad[at + 1] = 3;
    assert!(matches!(
        parse(&bad, &limits()),
        Err(ReadError::InvalidValue {
            field: "loss condition",
            value: 3,
            ..
        })
    ));
}

#[test]
fn the_raw_model_answers_its_questions() {
    let bits = Bits(vec![0b0000_0101, 0x80]);
    assert!(bits.get(0) && !bits.get(1) && bits.get(2) && bits.get(15));
    assert!(!bits.get(16) && !bits.get(1000));
    assert_eq!(bits.bytes(), [5, 0x80]);
    let mut m = sample(Version::Sod);
    m.header.two_levels = true;
    assert_eq!(m.header.levels(), 2);
    m.terrain.tiles[3].terrain = 9;
    assert_eq!(
        m.terrain.tile(Pos { x: 1, y: 1, z: 0 }).map(|t| t.terrain),
        Some(9)
    );
    assert_eq!(m.terrain.tile(Pos { x: 2, y: 0, z: 0 }), None);
    assert_eq!(m.terrain.tile(Pos { x: 0, y: 2, z: 0 }), None);
    assert_eq!(m.terrain.tile(Pos { x: 0, y: 0, z: 1 }), None);
    assert_eq!(format!("{:?}", Text(b"Orrin".to_vec())), "\"Orrin\"");
    assert_eq!(Text(b"x".to_vec()).as_bytes(), b"x");
    assert_eq!(
        [Version::RoE, Version::Ab, Version::Sod].map(Version::artifacts),
        [127, 129, 144]
    );
    let teams = Teams {
        count: 2,
        of_player: [0, 1, 0, 1, 0, 1, 0, 1],
    };
    let mut m = sample(Version::Ab);
    m.teams = Some(teams);
    assert_eq!(
        parse(&encode(&m), &limits()).map(|m| m.teams),
        Ok(Some(teams))
    );
}

#[test]
fn errors_describe_themselves() {
    let errors = [
        (
            ReadError::FileTooLarge { size: 9, max: 8 },
            "map file of 9 bytes exceeds the limit of 8",
        ),
        (ReadError::NotGzip, "not a gzip-compressed map file"),
        (
            ReadError::MapTooLarge { max: 8 },
            "map inflates to more than 8 bytes",
        ),
        (
            ReadError::ChecksumMismatch,
            "gzip checksum or length mismatch",
        ),
        (
            ReadError::UnsupportedVersion(0x33),
            "unsupported map format version 0x33",
        ),
        (
            ReadError::Truncated {
                field: "terrain",
                offset: 4,
            },
            "map ends inside terrain (offset 4)",
        ),
        (
            ReadError::LimitExceeded {
                field: "rumors",
                value: 9,
                max: 8,
                offset: 4,
            },
            "rumors = 9 exceeds the limit 8 (offset 4)",
        ),
        (
            ReadError::TrailingBytes { offset: 4 },
            "unexpected bytes after the timed events (offset 4)",
        ),
        (
            ReadError::InvalidValue {
                field: "loss condition",
                value: 3,
                offset: 4,
            },
            "invalid loss condition 3 (offset 4)",
        ),
    ];
    for (e, text) in errors {
        assert_eq!(e.to_string(), text);
    }
}

// --- gzip ---

/// A gzip member around `data` with the given flags and optional fields (`extra` is FEXTRA's
/// payload; FNAME, FCOMMENT and FHCRC get fixed contents).
fn gzip_with(data: &[u8], flags: u8, extra: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, flags, 0, 0, 0, 0, 0, 3];
    if flags & 4 != 0 {
        out.extend_from_slice(&u16::try_from(extra.len()).unwrap().to_le_bytes());
        out.extend_from_slice(extra);
    }
    if flags & 8 != 0 {
        out.extend_from_slice(b"name.h3m\0");
    }
    if flags & 16 != 0 {
        out.extend_from_slice(b"comment\0");
    }
    if flags & 2 != 0 {
        out.extend_from_slice(&[0x12, 0x34]);
    }
    out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec(data, 6));
    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
    out
}

#[test]
fn crc32_matches_the_check_value() {
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(crc32(b""), 0);
}

#[test]
fn gzip_headers_with_optional_fields_inflate() {
    let data = b"some map bytes".repeat(20);
    for flags in [0, 2, 4, 8, 16, 2 | 4 | 8 | 16] {
        assert_eq!(
            gunzip(&gzip_with(&data, flags, b"xy"), 1 << 20),
            Ok(data.clone())
        );
    }
    assert_eq!(
        gunzip(&crate::writer::gzip(&data), 1 << 20),
        Ok(data.clone())
    );
}

#[test]
fn broken_gzip_is_rejected() {
    let data = b"some map bytes".repeat(20);
    let good = gzip_with(&data, 8, b"");
    assert_eq!(gunzip(&good[..9], 1 << 20), Err(ReadError::NotGzip));
    assert_eq!(gunzip(&good[..12], 1 << 20), Err(ReadError::NotGzip));
    for (at, value) in [(0, 0), (1, 0), (2, 7), (3, 0x20)] {
        let mut bad = good.clone();
        bad[at] = value;
        assert_eq!(gunzip(&bad, 1 << 20), Err(ReadError::NotGzip), "byte {at}");
    }
    // FEXTRA that runs past the end; FNAME without its terminating zero.
    assert_eq!(
        gunzip(&[0x1f, 0x8b, 8, 4, 0, 0, 0, 0, 0, 3, 9], 99),
        Err(ReadError::NotGzip)
    );
    assert_eq!(
        gunzip(&[0x1f, 0x8b, 8, 8, 0, 0, 0, 0, 0, 3, b'a'], 99),
        Err(ReadError::NotGzip)
    );
    // Corrupt deflate data.
    let mut bad = good.clone();
    let deflate_at = 10 + 9;
    bad[deflate_at] = 0xFF;
    bad[deflate_at + 1] = 0xFF;
    assert_eq!(gunzip(&bad, 1 << 20), Err(ReadError::NotGzip));
    // Checksum and length.
    let n = good.len();
    let mut bad = good.clone();
    bad[n - 8] ^= 1;
    assert_eq!(gunzip(&bad, 1 << 20), Err(ReadError::ChecksumMismatch));
    let mut bad = good.clone();
    bad[n - 1] ^= 1;
    assert_eq!(gunzip(&bad, 1 << 20), Err(ReadError::ChecksumMismatch));
    // Size limit.
    assert_eq!(
        gunzip(&good, data.len() - 1),
        Err(ReadError::MapTooLarge {
            max: data.len() - 1
        })
    );
}

#[test]
fn read_bounds_the_file_and_inflates_it() {
    let m = sample(Version::Sod);
    let file = crate::writer::gzip(&encode(&m));
    assert_eq!(read(&file, &limits()), Ok(m));
    let tight = Limits {
        max_file_bytes: file.len() - 1,
        ..limits()
    };
    assert_eq!(
        read(&file, &tight),
        Err(ReadError::FileTooLarge {
            size: file.len(),
            max: file.len() - 1
        })
    );
    assert_eq!(
        read(b"plain bytes, no gzip", &limits()),
        Err(ReadError::NotGzip)
    );
}

// --- the experiment writer's maps (h3-formats, until the writer moves here) ---

/// E04c T3: land next to water (any of the eight neighbours) carries the coastal flag 0x40, as in
/// every one of the game's own maps; water and other land do not. The map's edge has no water.
#[test]
fn the_experiment_writer_marks_the_coast() {
    use crate::writer as w;
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1,
    });
    let map = w::Map {
        name: "Coast".to_owned(),
        description: String::new(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![
            w::Patch {
                terrain: w::Terrain::Water,
                from: (0, 0),
                to: (1, 0),
            },
            w::Patch {
                terrain: w::Terrain::Water,
                from: (20, 20),
                to: (20, 21),
            },
        ],
        roads: vec![],
        players,
        objects: vec![],
        underground: None,
        level_limit: 0,
    };
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    for (index, t) in m.terrain.tiles.iter().enumerate() {
        let (x, y) = (index % 36, index / 36);
        let water = matches!((x, y), (0..=1, 0) | (20, 20..=21));
        let near = (x <= 2 && y <= 1) || ((19..=21).contains(&x) && (19..=22).contains(&y));
        let coastal = if near && !water { 0x40 } else { 0 };
        assert_eq!(
            (t.terrain == 8, t.flags),
            (water, coastal),
            "tile ({x}, {y})"
        );
    }
}

/// E04c T5b: a town reads back with its body, and a hero on its entrance, two columns west of
/// the town's position, stands in it (the import's own test of a town's blocked tiles).
#[test]
fn towns_from_the_experiment_writer_read_back() {
    use crate::writer as w;
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for (colour, player) in players.iter_mut().take(2).enumerate() {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1 << (6 * colour),
        });
    }
    let town = w::Town {
        tile: (12, 9),
        template: w::ObjectTemplate::from_objects_txt(
            "AVCstrx0.def 000001110000011110001111111111111111111111111111 \
             001000000000000000000000000000000000000000000000 011111111 011111111 98 6 1 0",
        )
        .unwrap(),
        owner: Some(1),
        garrison: Some([Some((86, 30)), None, None, None, None, None, Some((87, 4))]),
        tight_formation: false,
        buildings: w::TownBuildings::Custom {
            built: [0x58, 0, 0x04, 0, 0, 0],
            forbidden: [0, 0, 0, 0, 0, 0x01],
        },
    };
    let hero = |tile, owner, hero_type| {
        w::MapObject::Hero(w::Hero {
            tile,
            owner,
            hero_type,
            class: 0,
            name: None,
            experience: None,
            skills: None,
            army: None,
            tight_formation: false,
            artifacts: None,
            spells: None,
            primary: None,
        })
    };
    let map = w::Map {
        name: "Siege".to_owned(),
        description: String::new(),
        size: 36,
        terrain: w::Terrain::Rough,
        patches: vec![],
        roads: vec![],
        players,
        objects: vec![
            hero((10, 10), 0, 0),
            w::MapObject::Town(town),
            hero((10, 9), 1, 1),
        ],
        underground: None,
        level_limit: 0,
    };
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    let template = &m.templates[m.objects[1].template as usize];
    assert_eq!((template.class, template.subclass), (98, 6));
    assert_eq!(m.objects[1].pos, Pos { x: 12, y: 9, z: 0 });
    let Body::Town(t) = &m.objects[1].body else {
        panic!("not a town: {:?}", m.objects[1].body)
    };
    assert_eq!((t.identifier, t.owner, t.name.as_ref()), (Some(2), 1, None));
    let garrison = t.garrison.as_ref().unwrap();
    let stack = |creature, count| Some(crate::model::Stack { creature, count });
    assert_eq!((garrison[0], garrison[6]), (stack(86, 30), stack(87, 4)));
    assert!(garrison[1..6].iter().all(Option::is_none));
    assert_eq!(
        t.buildings,
        crate::model::Buildings::Custom {
            built: [0x58, 0, 0x04, 0, 0, 0],
            forbidden: [0, 0, 0, 0, 0, 0x01],
        }
    );
    assert_eq!((t.formation, t.alignment, t.events.len()), (0, 0xff, 0));
    // The visiting hero on the entrance stands in the town, which the import cannot represent
    // yet (towns: E09).
    assert!(matches!(
        crate::import(&m, &import::SOD),
        Err(crate::ImportError::Unsupported(
            crate::Unsupported::HeroInTown(2)
        ))
    ));
}

#[test]
fn maps_from_the_experiment_writer_read_back() {
    use crate::writer as w;
    let hero = w::Hero {
        tile: (3, 4),
        owner: 0,
        hero_type: 0,
        class: 0,
        name: None,
        experience: Some(999),
        skills: Some(vec![(6, 2)]),
        army: Some([Some((1, 25)), None, None, None, None, None, None]),
        tight_formation: false,
        artifacts: None,
        spells: None,
        primary: Some([3, 2, 1, 1]),
    };
    let monster = w::Monster {
        tile: (10, 10),
        template: w::ObjectTemplate {
            def: "AVWpike.def".to_owned(),
            passable: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
            visitable: [0, 0, 0, 0, 0, 0x80],
            terrains: 0x1FF,
            menu: 0x1FF,
            class: 54,
            subclass: 12,
            group: 2,
            priority: 0,
        },
        count: 30,
        disposition: 4,
        never_flees: true,
        fixed_count: false,
    };
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 0x1FF,
    });
    players[1] = Some(w::Player {
        human: false,
        computer: true,
        towns: 1,
    });
    let map = w::Map {
        name: "Writer".to_owned(),
        description: "Read back".to_owned(),
        size: 36,
        terrain: w::Terrain::Grass,
        // A band of swamp, full height (the movement experiments, E08 T10a), and sand over its
        // top left tile.
        patches: vec![
            w::Patch {
                terrain: w::Terrain::Swamp,
                from: (10, 0),
                to: (12, 35),
            },
            w::Patch {
                terrain: w::Terrain::Sand,
                from: (10, 0),
                to: (10, 0),
            },
        ],
        // A dirt road ending on the swamp, as the movement experiments lay them.
        roads: vec![w::Road {
            kind: w::RoadKind::Dirt,
            row: 7,
            from: 8,
            to: 10,
        }],
        players,
        objects: vec![
            w::MapObject::Hero(hero),
            w::MapObject::Monster(monster),
            // A hero with everything its hero type's (E08 T10a: the starting armies).
            w::MapObject::Hero(w::Hero {
                tile: (20, 5),
                owner: 0,
                hero_type: 1,
                class: 0,
                name: None,
                experience: None,
                skills: None,
                army: None,
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            }),
            // A random mercury pile (E08 T16b: the piles of batch 2).
            w::MapObject::Pile(w::Pile {
                tile: (15, 20),
                template: w::ObjectTemplate::from_objects_txt(
                    "AVTmerc0.def 011111111111111111111111111111111111111111111111 \
                     100000000000000000000000000000000000000000000000 111111111 000000001 79 1 5 0",
                )
                .unwrap(),
                amount: 0,
                guards: None,
            }),
            // A Clover Field (E04c T3: the special grounds of batch 3), without a body.
            w::MapObject::Ground(w::Ground {
                tile: (30, 30),
                template: w::ObjectTemplate::from_objects_txt(CLOVER_FIELD).unwrap(),
            }),
            // A land Cartographer (E08 T32a: batch E4b's E4-10), without a body.
            w::MapObject::Plain(w::Plain {
                tile: (25, 25),
                template: w::ObjectTemplate::from_objects_txt(CARTOGRAPHER).unwrap(),
            }),
        ],
        underground: None,
        // The scenario's level limit (E08 T32a: batch E4a's E4-6).
        level_limit: 5,
    };
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    assert_eq!(m.header.version, Version::Sod);
    assert_eq!((m.header.size, m.header.two_levels), (36, false));
    assert_eq!(m.header.name.as_bytes(), b"Writer");
    assert_eq!(m.header.description.as_bytes(), b"Read back");
    assert_eq!(m.header.level_limit, 5);
    let red = m.players[0].details.as_ref().unwrap();
    assert_eq!(red.allowed_towns, 0x1FF);
    assert_eq!(red.main_hero.as_ref().map(|h| h.hero_type), Some(0));
    assert_eq!(
        red.heroes,
        [0, 1].map(|hero_type| PlayerHero {
            hero_type,
            name: Text::default()
        })
    );
    let blue = m.players[1].details.as_ref().unwrap();
    assert!(!m.players[1].human && m.players[1].computer && blue.heroes.is_empty());
    assert!(m.players[2..].iter().all(|p| p.details.is_none()));
    assert_eq!(
        (m.victory, m.loss, m.teams),
        (Victory::Standard, Loss::Standard, None)
    );
    assert_eq!(m.terrain.tiles.len(), 36 * 36);
    for (index, t) in m.terrain.tiles.iter().enumerate() {
        let (x, y) = (index % 36, index / 36);
        let (terrain, first) = match (x, y) {
            (10, 0) => (1, 0),
            (10..=12, _) => (4, 49),
            _ => (2, 49),
        };
        assert_eq!(t.terrain, terrain, "terrain of ({x}, {y})");
        assert!(
            (first..first + 8).contains(&t.view),
            "picture of ({x}, {y})"
        );
        let road = match (x, y) {
            (8, 7) => (1, 15, 0),
            (9, 7) => (1, 12, 0),
            (10, 7) => (1, 15, 0x10),
            _ => (0, 0, 0),
        };
        assert_eq!((t.road, t.road_dir, t.flags), road, "road of ({x}, {y})");
        assert_eq!((t.river, t.river_dir), (0, 0));
    }
    // The heroes (written one column right of their tiles, docs/formats/h3m.md), the monster, the
    // pile, the ground and the Cartographer.
    assert_eq!(m.objects.len(), 6);
    let hero_template = &m.templates[m.objects[0].template as usize];
    assert_eq!((hero_template.class, hero_template.subclass), (34, 0));
    assert_eq!(m.objects[0].pos, Pos { x: 4, y: 4, z: 0 });
    let Body::Hero(h) = &m.objects[0].body else {
        panic!("not a hero: {:?}", m.objects[0].body)
    };
    assert_eq!((h.owner, h.hero_type, h.experience), (0, 0, Some(999)));
    assert_eq!(h.secondary_skills, Some(vec![(6, 2)]));
    assert_eq!(h.primary_skills, Some([3, 2, 1, 1]));
    let army = h.army.as_ref().unwrap();
    assert_eq!(
        army[0],
        Some(crate::model::Stack {
            creature: 1,
            count: 25
        })
    );
    assert!(army[1..].iter().all(Option::is_none));
    let monster_template = &m.templates[m.objects[1].template as usize];
    assert_eq!(
        (monster_template.class, monster_template.subclass),
        (54, 12)
    );
    let Body::Monster(mon) = &m.objects[1].body else {
        panic!("not a monster: {:?}", m.objects[1].body)
    };
    assert_eq!((mon.count, mon.disposition, mon.never_flees), (30, 4, true));
    let Body::Hero(h) = &m.objects[2].body else {
        panic!("not a hero: {:?}", m.objects[2].body)
    };
    assert_eq!(m.objects[2].pos, Pos { x: 21, y: 5, z: 0 });
    assert_eq!((h.owner, h.hero_type, h.name.as_ref()), (0, 1, None));
    assert_eq!((h.experience, h.secondary_skills.as_ref()), (None, None));
    assert_eq!((h.army.as_ref(), h.artifacts.as_ref()), (None, None));
    assert_eq!((h.spells.as_ref(), h.primary_skills), (None, None));
    let pile_template = &m.templates[m.objects[3].template as usize];
    assert_eq!((pile_template.class, pile_template.subclass), (79, 1));
    assert_eq!(m.objects[3].pos, Pos { x: 15, y: 20, z: 0 });
    assert_eq!(
        m.objects[3].body,
        Body::Resource {
            message: None,
            amount: 0
        }
    );
    let ground_template = &m.templates[m.objects[4].template as usize];
    assert_eq!((ground_template.class, ground_template.subclass), (222, 0));
    assert_eq!(m.objects[4].pos, Pos { x: 30, y: 30, z: 0 });
    assert_eq!(m.objects[4].body, Body::None);
    let plain_template = &m.templates[m.objects[5].template as usize];
    assert_eq!((plain_template.class, plain_template.subclass), (13, 1));
    assert_eq!(m.objects[5].pos, Pos { x: 25, y: 25, z: 0 });
    assert_eq!(m.objects[5].body, Body::None);
    assert!(m.events.is_empty());
    // The import types the pile as random mercury: no fixed amount, drawn when the game starts.
    // Heroes leave the objects for the setup's heroes: the monster, the pile, the ground, then the
    // Cartographer; the setup keeps the level limit.
    let setup = crate::import(&m, &import::SOD).unwrap();
    assert_eq!(setup.map.objects.len(), 4);
    assert_eq!(
        setup.map.objects[3].object_type,
        import::SOD
            .adventure()
            .object_type_by_key("base:cartographer_land")
    );
    assert_eq!(setup.level_limit, 5);
    let pile = &setup.map.objects[1];
    assert_eq!(
        pile.object_type,
        import::SOD
            .adventure()
            .object_type_by_key("base:pile_mercury")
    );
    assert!(pile.object_type.is_some());
    assert_eq!(pile.amount, None);
}

/// E08 T22a1: the writer's underground level and guarded piles, read back and imported (batch
/// E3's monster below and guarded piles; ADV-PIK-004).
#[test]
fn the_writer_s_underground_and_guarded_piles_read_back() {
    use crate::writer as w;
    let hero = |tile, hero_type| {
        w::MapObject::Hero(w::Hero {
            tile,
            owner: 0,
            hero_type,
            class: 0,
            name: None,
            experience: None,
            skills: None,
            army: Some([Some((0, 5)), None, None, None, None, None, None]),
            tight_formation: false,
            artifacts: None,
            spells: None,
            primary: None,
        })
    };
    let wood = w::ObjectTemplate::from_objects_txt(
        "AVTwood0.def 011111111111111111111111111111111111111111111111 \
         100000000000000000000000000000000000000000000000 111111111 000000001 79 0 5 0",
    )
    .unwrap();
    let mut guards = [None; w::ARMY_SLOTS];
    guards[1] = Some((0, 12));
    guards[4] = Some((1, 3));
    let pile = |tile, question: &str| {
        w::MapObject::Pile(w::Pile {
            tile,
            template: wood.clone(),
            amount: 7,
            guards: Some(w::Guards {
                army: guards,
                question: question.into(),
            }),
        })
    };
    let monster = w::MapObject::Monster(w::Monster {
        tile: (8, 8),
        template: w::ObjectTemplate::from_objects_txt(
            "AvWPike.def 011111111111111111111111111111111111111111111111 \
             100000000000000000000000000000000000000000000000 111111111 000000100 54 0 2 0",
        )
        .unwrap(),
        count: 20,
        disposition: 1,
        never_flees: false,
        fixed_count: true,
    });
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1,
    });
    let map = w::Map {
        name: "Below".to_owned(),
        description: "Two levels".to_owned(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![],
        roads: vec![],
        players,
        objects: vec![
            hero((3, 4), 0),
            pile((10, 10), "Fight?"),
            pile((12, 10), ""),
        ],
        underground: Some(w::Underground {
            terrain: w::Terrain::Subterranean,
            patches: vec![w::Patch {
                terrain: w::Terrain::Rock,
                from: (0, 0),
                to: (35, 1),
            }],
            objects: vec![hero((5, 5), 1), monster],
        }),
        level_limit: 0,
    };
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    assert!(m.header.two_levels);
    assert_eq!((m.terrain.levels, m.terrain.tiles.len()), (2, 2 * 36 * 36));
    let level = |pos: Pos| {
        m.terrain.tiles[(usize::from(pos.z) * 36 + usize::from(pos.y)) * 36 + usize::from(pos.x)]
            .terrain
    };
    assert_eq!(level(Pos { x: 9, y: 1, z: 1 }), 9);
    assert_eq!(level(Pos { x: 9, y: 2, z: 1 }), 6);
    assert_eq!(level(Pos { x: 9, y: 1, z: 0 }), 2);
    let red = m.players[0].details.as_ref().unwrap();
    assert_eq!(
        red.heroes.iter().map(|h| h.hero_type).collect::<Vec<_>>(),
        [0, 1]
    );
    let positions: Vec<Pos> = m.objects.iter().map(|o| o.pos).collect();
    assert_eq!(
        positions,
        [(4, 4, 0), (10, 10, 0), (12, 10, 0), (6, 5, 1), (8, 8, 1)].map(|(x, y, z)| Pos {
            x,
            y,
            z
        })
    );
    let Body::Resource {
        message: Some(message),
        amount: 7,
    } = &m.objects[1].body
    else {
        panic!("not a guarded pile: {:?}", m.objects[1].body)
    };
    assert_eq!(message.text.as_bytes(), b"Fight?");
    let army = message.guards.as_ref().unwrap();
    assert_eq!(
        army.iter()
            .map(|s| s.map(|s| (s.creature, s.count)))
            .collect::<Vec<_>>(),
        [None, Some((0, 12)), None, None, Some((1, 3)), None, None]
    );
    let Body::Monster(mon) = &m.objects[4].body else {
        panic!("not a monster: {:?}", m.objects[4].body)
    };
    assert_eq!(
        (mon.count, mon.disposition, mon.never_flees),
        (20, 1, false)
    );
    // The import: both piles guarded, the first asks; the second hero and the monster below.
    let setup = crate::import(&m, &import::SOD).unwrap();
    let guards: Vec<_> = setup.map.objects[..2]
        .iter()
        .map(|o| o.guards.as_ref().map(|g| (g.army.len(), g.question)))
        .collect();
    assert_eq!(guards, [Some((2, true)), Some((2, false))]);
    let below = setup.map.objects[2]
        .monster
        .as_ref()
        .map(|_| setup.map.objects[2].anchor.level);
    assert_eq!(below, Some(1));
    assert_eq!(
        setup.heroes.iter().map(|h| h.pos.level).collect::<Vec<_>>(),
        [0, 1]
    );
}

// --- properties ---

proptest! {
    /// Every model a version can hold reads back from its encoding.
    #[test]
    fn every_model_round_trips(m in raw_map()) {
        prop_assert_eq!(parse(&encode(&m), &limits()), Ok(m));
    }

    /// Every strict prefix of a map fails as truncated, never as anything else.
    #[test]
    fn every_prefix_is_truncated(m in raw_map(), cut in any::<prop::sample::Index>()) {
        let bytes = encode(&m);
        let n = cut.index(bytes.len());
        let result = parse(&bytes[..n], &limits());
        prop_assert!(matches!(result, Err(ReadError::Truncated { .. })), "{:?}", result);
    }

    /// Arbitrary bytes after a supported version never panic the reader.
    #[test]
    fn arbitrary_bytes_never_panic(v in version(), rest in prop::collection::vec(any::<u8>(), 0..1500)) {
        let mut bytes = v.code().to_le_bytes().to_vec();
        bytes.extend_from_slice(&rest);
        let _ = parse(&bytes, &limits());
    }

    /// A valid map with some bytes overwritten never panics the reader.
    #[test]
    fn mutated_maps_never_panic(
        m in raw_map(),
        edits in prop::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 1..8),
    ) {
        let mut bytes = encode(&m);
        for (at, value) in edits {
            let i = at.index(bytes.len());
            bytes[i] = value;
        }
        let _ = parse(&bytes, &limits());
    }

    /// The gzip layer: a compressed map reads back; arbitrary or damaged files never panic.
    #[test]
    fn gzip_files_read_back_or_fail_cleanly(
        m in raw_map(),
        noise in prop::collection::vec(any::<u8>(), 0..300),
        edit in (any::<prop::sample::Index>(), any::<u8>()),
    ) {
        let mut file = crate::writer::gzip(&encode(&m));
        prop_assert_eq!(read(&file, &limits()), Ok(m));
        let _ = read(&noise, &limits());
        let i = edit.0.index(file.len());
        file[i] = edit.1;
        let _ = read(&file, &limits());
    }
}

/// The encoder's helpers are only used by the tests above; this keeps `Out` honest about the
/// byte layout of an artifact slot in each version.
#[test]
fn artifact_slots_have_the_version_width() {
    let mut o = Out::default();
    o.artifact(Version::RoE, None);
    o.artifact(Version::Ab, None);
    o.artifact(Version::Sod, Some(3));
    assert_eq!(o.0, [0xFF, 0xFF, 0xFF, 3, 0]);
}
