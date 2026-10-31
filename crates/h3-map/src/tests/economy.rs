//! The experiment writer's increment 9 objects and timed events (E08 T78: batch E9's maps;
//! increment 9's design note §4, survey `docs/rules/surveys/E08-economy.md` §2.1, §3.4, §9, §11):
//! the seven mines, the abandoned mine (53.7 and SoD's 220.7) with its resource set, a Lighthouse
//! and a Shipyard with their owners, the three weekly objects and the map's timed events read back
//! as written, in SoD and RoE; what the writer refuses; and the map imports (E08 T83a: its
//! classes typed, the owners and sets, its events).

use h3_adventure::{Holds, Scenery};
use h3_content::ResourceId;
use h3_contracts::PlayerId;
use h3_game::TimedEventSetup;

use super::import::SOD;
use super::limits;
use crate::model::{Bits, Body, RawMap, Text, TimedEvent};
use crate::{ImportError, Unsupported, import, import_map};
use crate::read;
use crate::writer as w;

/// The game's `Objects.txt` lines of the classes written here: a grass sawmill (53.0), a grass
/// crystal cavern (53.4), a gold mine (53.6), the abandoned mine (53.7) and SoD's grass one
/// (220.7), a Lighthouse (42), a Shipyard (87), a Windmill (112), a Water Wheel (109) and a
/// Mystical Garden (55).
const LINES: [&str; 10] = [
    "AVMsawg0.def 000011110011111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000010100 53 0 0 0",
    "AVMcrys0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000000100 53 4 0 0",
    "AVMgold0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000000100 53 6 0 0",
    "AVXabnd0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000000001 53 7 0 0",
    "AVXamGr.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000000100 220 7 0 0",
    "AVXlths0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 010111111 42 0 0 0",
    "AVXshyd0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 011111111 011111111 87 0 0 0",
    "AVMwndd0.def 101111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 112 0 0 0",
    "AVMwwhl0.def 000111110001111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000010101 109 0 0 0",
    "AVTmyst0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000010101 55 0 0 0",
];

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

fn hero(tile: (u8, u8), owner: u8, hero_type: u8) -> w::MapObject {
    w::MapObject::Hero(w::Hero {
        tile,
        owner,
        hero_type,
        class: 0,
        name: None,
        experience: None,
        skills: None,
        army: Some([Some((1, 5)), None, None, None, None, None, None]),
        tight_formation: false,
        artifacts: None,
        spells: None,
        primary: None,
    })
}

fn owned(tile: (u8, u8), line: &str, body: w::OwnedBody) -> w::MapObject {
    w::MapObject::Owned(w::OwnedObject {
        tile,
        template: template(line),
        body,
    })
}

/// Batch E9-11's events: two on the file's day 0 (the game's day 1) for Red, the first taking
/// 99 999 gold, the second giving 300; one from the file's day 1 every two days taking 500, for
/// Red's computer too.
fn events() -> Vec<w::TimedEvent> {
    let gold = |amount| [0, 0, 0, 0, 0, 0, amount];
    vec![
        w::TimedEvent {
            name: "Take".into(),
            message: "The tax collector came.".into(),
            resources: gold(-99_999),
            players: 1,
            human: true,
            computer: false,
            first_day: 0,
            interval: 0,
        },
        w::TimedEvent {
            name: "Give".into(),
            message: String::new(),
            resources: gold(300),
            players: 1,
            human: true,
            computer: false,
            first_day: 0,
            interval: 0,
        },
        w::TimedEvent {
            name: "Every two days".into(),
            message: "Again.".into(),
            resources: gold(-500),
            players: 1,
            human: true,
            computer: true,
            first_day: 1,
            interval: 2,
        },
    ]
}

/// A 36 × 36 grass map whose columns 30 to 35 are water, Red and Blue each with a hero: a Red
/// sawmill, a Blue crystal cavern, a neutral gold mine, the two abandoned mines (wood and gold;
/// crystal), a Red Lighthouse and a neutral Shipyard on the coast, the three weekly objects, and
/// three timed events.
fn economy_map() -> w::Map {
    use w::OwnedBody::{Owner, Resources};
    let [
        sawmill,
        crystal,
        gold,
        abandoned,
        abandoned_sod,
        lighthouse,
        shipyard,
        ..,
    ] = LINES;
    let [.., windmill, wheel, garden] = LINES.map(template);
    let plain = |tile, template| w::MapObject::Plain(w::Plain { tile, template });
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
        name: "Economy".into(),
        description: "E08 T78".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (30, 0),
            to: (35, 35),
        }],
        roads: vec![],
        players,
        objects: vec![
            hero((2, 2), 0, 0),
            hero((2, 30), 1, 1),
            owned((6, 4), sawmill, Owner(Some(0))),
            owned((12, 4), crystal, Owner(Some(1))),
            owned((18, 4), gold, Owner(None)),
            owned((6, 10), abandoned, Resources(0b100_0001)),
            owned((12, 10), abandoned_sod, Resources(0b1_0000)),
            owned((29, 10), lighthouse, Owner(Some(0))),
            owned((28, 16), shipyard, Owner(None)),
            plain((6, 20), windmill),
            plain((12, 20), wheel),
            plain((18, 20), garden),
        ],
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: events(),
    }
}

fn read_back(bytes: &[u8]) -> RawMap {
    read(&w::gzip(bytes), &limits()).unwrap()
}

/// The model's event for the writer's.
fn model(event: &w::TimedEvent) -> TimedEvent {
    TimedEvent {
        name: Text(event.name.as_bytes().to_vec()),
        message: Text(event.message.as_bytes().to_vec()),
        resources: event.resources,
        players: event.players,
        human: event.human,
        computer: event.computer,
        first_day: event.first_day,
        interval: event.interval,
    }
}

#[test]
fn increment_9_s_objects_and_events_read_back_as_written() {
    let m = read_back(&w::write(&economy_map()).unwrap());
    let placed: Vec<(u32, u32, (u8, u8))> = m
        .objects
        .iter()
        .map(|o| {
            let t = &m.templates[usize::try_from(o.template).unwrap()];
            (t.class, t.subclass, (o.pos.x, o.pos.y))
        })
        .collect();
    assert_eq!(
        placed[2..],
        [
            (53, 0, (6, 4)),
            (53, 4, (12, 4)),
            (53, 6, (18, 4)),
            (53, 7, (6, 10)),
            (220, 7, (12, 10)),
            (42, 0, (29, 10)),
            (87, 0, (28, 16)),
            (112, 0, (6, 20)),
            (109, 0, (12, 20)),
            (55, 0, (18, 20)),
        ]
    );
    let bodies: Vec<&Body> = m.objects.iter().map(|o| &o.body).collect();
    assert_eq!(*bodies[2], Body::Mine { owner: 0 });
    assert_eq!(*bodies[3], Body::Mine { owner: 1 });
    assert_eq!(*bodies[4], Body::Mine { owner: 0xff });
    assert_eq!(*bodies[5], Body::AbandonedMine(Bits(vec![0x41, 0, 0, 0])));
    assert_eq!(*bodies[6], Body::AbandonedMine(Bits(vec![0x10, 0, 0, 0])));
    assert_eq!(*bodies[7], Body::Owned { owner: 0 });
    assert_eq!(*bodies[8], Body::Owned { owner: 0xff });
    assert!(bodies[9..].iter().all(|b| **b == Body::None));
    assert_eq!(m.events, events().iter().map(model).collect::<Vec<_>>());
}

#[test]
fn an_roe_map_s_events_hold_no_human_flag() {
    // RoE without SoD's abandoned mine: the events without the human byte read back as every
    // human's (the reader's layout by version), the abandoned mine's set as SoD's.
    let mut map = economy_map();
    map.objects.remove(6);
    let roe = w::write_in(&map, w::Format::RoE).unwrap();
    let m = read_back(&roe);
    assert_eq!(m.events, events().iter().map(model).collect::<Vec<_>>());
    assert_eq!(
        m.objects[5].body,
        Body::AbandonedMine(Bits(vec![0x41, 0, 0, 0]))
    );
}

#[test]
fn the_writer_refuses_what_the_game_cannot_load() {
    use w::OwnedBody::{Owner, Resources};
    let [sawmill, _, _, abandoned, _, lighthouse, ..] = LINES;
    let refused = |object: w::MapObject, what: &str| {
        let mut map = economy_map();
        map.objects.push(object);
        match w::write(&map) {
            Err(w::WriteError::OutOfRange { what: got, .. }) => assert_eq!(got, what),
            other => panic!("{what}: {other:?}"),
        }
    };
    // An owner nobody plays; an owner on a class without one; a resource set on a mine.
    refused(
        owned((20, 30), lighthouse, Owner(Some(2))),
        "object owner (no playable player)",
    );
    refused(
        owned((20, 30), abandoned, Owner(None)),
        "owned object class",
    );
    let pikeman = "AvWPike.def 011111111111111111111111111111111111111111111111 \
                   100000000000000000000000000000000000000000000000 111111111 000000100 \
                   54 0 2 0";
    refused(owned((20, 30), pikeman, Owner(None)), "owned object class");
    refused(
        owned((20, 30), sawmill, Resources(1)),
        "abandoned mine class",
    );
    // An abandoned mine without a resource, or with bit 7 (no resource of the game's).
    for set in [0, 0x80, 0x81] {
        refused(
            owned((20, 30), abandoned, Resources(set)),
            "abandoned mine resources",
        );
    }
    // RoE: no class 220, every event a human's.
    let mut map = economy_map();
    let roe = |map: &w::Map| match w::write_in(map, w::Format::RoE) {
        Err(w::WriteError::OutOfRange { what, .. }) => what,
        other => panic!("{other:?}"),
    };
    assert_eq!(roe(&map), "SoD's abandoned mine in an RoE map");
    map.objects.remove(6);
    map.events[1].human = false;
    assert_eq!(roe(&map), "timed event without human players in an RoE map");
    // SoD keeps an event for the computer alone.
    w::write(&map).unwrap();
}

// covers: ADV-OBJ-002, ADV-OWN-002, ADV-MIN-003, GAM-CAL-004
#[test]
fn the_economy_map_imports() {
    // E08 T83a (increment 9's design note §4): every class typed, the owners as players, the
    // abandoned mines' sets, the timed events in the map's order.
    let setup = import(&read_back(&w::write(&economy_map()).unwrap()), &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    assert_eq!(setup.heroes.len(), 2);
    let part = SOD.adventure();
    let types: Vec<_> = setup.map.objects.iter().map(|o| o.object_type).collect();
    let keys = [
        "sawmill",
        "crystal_cavern",
        "gold_mine",
        "abandoned_mine",
        "abandoned_mine",
        "lighthouse",
        "shipyard",
        "windmill",
        "water_wheel",
        "mystical_garden",
    ]
    .map(|k| part.object_type_by_key(&format!("base:{k}")));
    assert_eq!(types, keys);
    let owners: Vec<_> = setup.map.objects.iter().map(|o| o.owner).collect();
    let (red, blue) = (Some(PlayerId::new(0)), Some(PlayerId::new(1)));
    assert_eq!(
        owners,
        [red, blue, None, None, None, red, None, None, None, None]
    );
    let resource = |k: &str| {
        let index = SOD
            .core()
            .resources()
            .iter()
            .position(|r| r.key.as_str() == format!("base:{k}"))
            .unwrap();
        ResourceId::from_index(index).unwrap()
    };
    let holds: Vec<_> = setup.map.objects.iter().map(|o| o.holds.clone()).collect();
    let mut expected = vec![Holds::Nothing; 10];
    expected[3] = Holds::Resources(vec![resource("wood"), resource("gold")]);
    expected[4] = Holds::Resources(vec![resource("crystal")]);
    assert_eq!(holds, expected);
    // The abandoned mines are ownable scenery as the mines (GAM-BAT-011: 220 is 53 too).
    assert!(
        setup.map.objects[..7]
            .iter()
            .all(|o| o.scenery == Scenery::Ownable)
    );
    let gold = resource("gold");
    let event = |amount, computer, first_day, interval, message| TimedEventSetup {
        players: vec![PlayerId::new(0)],
        human: true,
        computer,
        first_day,
        interval,
        resources: vec![(gold, amount)],
        message,
    };
    assert_eq!(
        setup.events,
        [
            event(-99_999, false, 1, 0, 0),
            event(300, false, 1, 0, 1),
            event(-500, true, 2, 2, 2),
        ]
    );
}

// covers: ADV-OWN-002, ADV-MIN-003, GAM-CAL-004
#[test]
fn the_import_refuses_an_owner_nobody_plays_and_an_empty_mine() {
    // The writer refuses both (the editor's rules): the read map is changed instead. Errors name
    // the map's object index (the heroes 0 and 1 included).
    let raw = read_back(&w::write(&economy_map()).unwrap());
    let with = |index: usize, body: Body| {
        let mut map = raw.clone();
        map.objects[index].body = body;
        import(&map, &SOD)
    };
    // Purple (5) does not play, nor does a colour past the eight.
    assert_eq!(
        with(2, Body::Mine { owner: 5 }).unwrap_err(),
        ImportError::ObjectOwner(2)
    );
    assert_eq!(
        with(7, Body::Owned { owner: 8 }).unwrap_err(),
        ImportError::ObjectOwner(7)
    );
    assert_eq!(
        with(5, Body::AbandonedMine(Bits(vec![0, 0x41, 0, 0]))).unwrap_err(),
        ImportError::MineResources(5)
    );
    assert_eq!(
        with(6, Body::AbandonedMine(Bits(vec![0x81, 0, 0, 0]))).unwrap_err(),
        ImportError::MineResources(6)
    );
    assert_eq!(
        with(6, Body::AbandonedMine(Bits(vec![]))).unwrap_err(),
        ImportError::MineResources(6)
    );
    // The other bytes are dropped, as the original reads the set.
    assert!(with(6, Body::AbandonedMine(Bits(vec![0x10, 0xFF, 0xFF, 0xFF]))).is_ok());
    // An event for a colour nobody plays is kept for none; one without resources gives none.
    let mut map = raw.clone();
    map.events[0].players = 1 << 4 | 1 << 1;
    map.events[1].resources = [0; 7];
    let setup = import(&map, &SOD).unwrap();
    assert_eq!(setup.events[0].players, [PlayerId::new(1)]);
    assert!(setup.events[1].resources.is_empty());
    setup.validate(&SOD).unwrap();
    map.events[0].players = 1 << 4;
    assert!(import(&map, &SOD).unwrap().events[0].players.is_empty());
    // The file's last day saturates.
    map.events[2].first_day = u16::MAX;
    assert_eq!(import(&map, &SOD).unwrap().events[2].first_day, u16::MAX);
}

// covers: ADV-OWN-002
#[test]
fn an_ownable_object_entered_from_water_is_unsupported() {
    // The Lighthouse moved one column east: its entrance (its anchor) on the water from column
    // 30; the static layer keeps it typed.
    let mut map = economy_map();
    let w::MapObject::Owned(lighthouse) = &mut map.objects[7] else {
        panic!("not owned");
    };
    lighthouse.tile = (30, 10);
    let raw = read_back(&w::write(&map).unwrap());
    assert_eq!(
        import(&raw, &SOD).unwrap_err(),
        ImportError::Unsupported(Unsupported::OwnedOnWater(7))
    );
    let layer = import_map(&raw, &SOD).unwrap();
    assert_eq!(
        layer.objects[5].object_type,
        SOD.adventure().object_type_by_key("base:lighthouse")
    );
}
