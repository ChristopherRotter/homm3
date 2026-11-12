//! The experiment writer's increment 12 objects (E08 T109a: batch E12's maps; increment 12's
//! design note §18, survey `docs/rules/surveys/E08-garrisons-bonuses.md` §1, §2): the garrisons
//! (33 and AB's 219, subclass 1 the anti-magic garrison) with their owners, armies and removable
//! flags, and the fifteen bonus visitables without a body, read back as written in SoD and RoE,
//! their templates' masks as the game's; what the writer refuses; and since E08 T113b the import's
//! types of the seventeen classes (ADV-GAR-001, ADV-BON-001), a garrison's body.

use super::import::{ROE, SOD};
use super::limits;
use crate::import::codes;
use crate::model::{Body, RawMap, Stack};
use crate::writer as w;
use crate::{ImportError, import, read};
use h3_adventure::{Holds, SetupError};
use h3_content::CreatureTypeId;
use h3_contracts::{ArmyStack, PlayerId};
use h3_game::GameSetupError;

/// The game's `Objects.txt` lines of the garrisons: horizontal (33.0), horizontal anti-magic
/// (33.1), vertical (219.0) and vertical anti-magic (219.1).
const GARRISONS: [&str; 4] = [
    "AVCgar10.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 33 0 0 0",
    "AVCgar20.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 33 1 0 0",
    "AVCvgr.def 011111110111111101111111111111111111111111111111 \
     000000001000000000000000000000000000000000000000 111111111 011111111 219 0 0 0",
    "avcvgarm.def 011111110111111101111111111111111111111111111111 \
     000000001000000000000000000000000000000000000000 111111111 011111111 219 1 0 0",
];

/// The game's `Objects.txt` lines of the twelve land bonus visitables (one picture each, the
/// Swan Pond's and the Magic Well's first): the Swan Pond (14), the Faerie Ring (28), the
/// Fountain of Fortune (30), the Fountain of Youth (31), the Idol of Fortune (38), the Magic
/// Spring (48), the Magic Well (49), the Oasis (56), the Rally Flag (64), the Stables (94), the
/// Temple (96) and the Watering Hole (110).
const LAND_BONUSES: [&str; 12] = [
    "AVSclvd0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000000001 14 0 0 0",
    "AVSring0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000010101 28 0 0 0",
    "AVSfntn0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000010101 30 0 0 0",
    "AVXfyth0.def 001111110011111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000010101 31 0 0 0",
    "AVSidol0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 38 0 0 0",
    "AVXmags0.def 001111111111111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 000100000 48 0 0 0",
    "AVXwelg0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000010101 49 0 0 0",
    "AVXosis0.def 001111110011111111111111111111111111111111111111 \
     110000000000000000000000000000000000000000000000 111111111 000000010 56 0 0 0",
    "AVXrlly0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 64 0 0 0",
    "AVXstbl0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000100101 94 0 0 0",
    "AVStmpl0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 96 0 0 0",
    "AVXwtrh0.def 000011111111111111111111111111111111111111111111 \
     111100000000000000000000000000000000000000000000 111111111 000100000 110 0 0 0",
];

/// The game's `Objects.txt` lines of the three water bonus visitables: the Buoy (11), the
/// Mermaids (52) and the Sirens (92).
const WATER_BONUSES: [&str; 3] = [
    "AVSbuoy0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 100000000 100000000 11 0 0 0",
    "AVXmerm0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 52 0 0 0",
    "AVXsirn0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 92 0 0 0",
];

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

/// A stack of `count` of the creature `creature` in slot order.
fn army(stacks: &[(u16, u16)]) -> [Option<(u16, u16)>; w::ARMY_SLOTS] {
    let mut army = [None; w::ARMY_SLOTS];
    for (slot, &stack) in army.iter_mut().zip(stacks) {
        *slot = Some(stack);
    }
    army
}

fn garrison(
    tile: (u8, u8),
    line: &str,
    owner: Option<u8>,
    stacks: &[(u16, u16)],
    removable: bool,
) -> w::MapObject {
    w::MapObject::Owned(w::OwnedObject {
        tile,
        template: template(line),
        body: w::OwnedBody::Garrison {
            owner,
            army: army(stacks),
            removable,
        },
    })
}

/// The garrisons' tiles: 33.0 Red's with Pikemen and Archers at (6, 4), 33.1 neutral and empty
/// at (12, 4), 219.0 Blue's with Griffins at (6, 10), not removable, and 219.1 neutral with seven
/// stacks at (12, 10), not removable.
fn garrisons() -> Vec<w::MapObject> {
    let [horizontal, anti_magic, vertical, vertical_anti_magic] = GARRISONS;
    let seven = [(0, 1), (2, 2), (4, 3), (6, 4), (8, 5), (10, 6), (144, 200)];
    vec![
        garrison((6, 4), horizontal, Some(0), &[(0, 10), (2, 5)], true),
        garrison((12, 4), anti_magic, None, &[], true),
        garrison((6, 10), vertical, Some(1), &[(4, 3)], false),
        garrison((12, 10), vertical_anti_magic, None, &seven, false),
    ]
}

/// The land bonus visitables in rows of four from (4, 16), six columns and four rows apart; the
/// water ones at (28, 6), (28, 12) and (28, 18).
fn bonuses() -> Vec<w::MapObject> {
    let plain = |tile, line: &str| {
        w::MapObject::Plain(w::Plain {
            tile,
            template: template(line),
        })
    };
    let land = LAND_BONUSES.iter().zip(0u8..).map(|(line, i)| {
        let tile = (4 + 6 * (i % 4), 16 + 4 * (i / 4));
        plain(tile, line)
    });
    let water = WATER_BONUSES
        .iter()
        .zip([6u8, 12, 18])
        .map(|(line, y)| plain((28, y), line));
    land.chain(water).collect()
}

/// A 36 × 36 grass map, columns 24 to 35 water, Red and Blue playing: Red's hero at (2, 2), the
/// garrisons, then the bonus visitables.
fn increment_12_map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut objects = vec![super::economy::hero((2, 2), 0, 0)];
    objects.extend(garrisons());
    objects.extend(bonuses());
    w::Map {
        name: "Garrisons and bonuses".into(),
        description: "E08 T109a".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (24, 0),
            to: (35, 35),
        }],
        roads: vec![],
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

fn read_back(bytes: &[u8]) -> RawMap {
    read(&w::gzip(bytes), &limits()).unwrap()
}

/// The model's army for the writer's stacks.
fn model_army(stacks: &[(u16, u16)]) -> Vec<Option<Stack>> {
    army(stacks)
        .iter()
        .map(|s| s.map(|(creature, count)| Stack { creature, count }))
        .collect()
}

#[test]
fn increment_12_s_objects_read_back_as_written() {
    let m = read_back(&w::write(&increment_12_map()).unwrap());
    let template_of = |i: usize| &m.templates[usize::try_from(m.objects[i].template).unwrap()];
    let placed: Vec<(u32, u32, (u8, u8))> = (1..m.objects.len())
        .map(|i| {
            let (t, o) = (template_of(i), &m.objects[i]);
            (t.class, t.subclass, (o.pos.x, o.pos.y))
        })
        .collect();
    let mut expected = vec![
        (33, 0, (6, 4)),
        (33, 1, (12, 4)),
        (219, 0, (6, 10)),
        (219, 1, (12, 10)),
    ];
    let land_classes = [14, 28, 30, 31, 38, 48, 49, 56, 64, 94, 96, 110];
    expected.extend(
        land_classes
            .iter()
            .zip(0u8..)
            .map(|(&class, i)| (class, 0, (4 + 6 * (i % 4), 16 + 4 * (i / 4)))),
    );
    expected.extend([(11, 0, (28, 6)), (52, 0, (28, 12)), (92, 0, (28, 18))]);
    assert_eq!(placed, expected);
    let mut classes: Vec<u32> = placed[4..].iter().map(|p| p.0).collect();
    classes.sort_unstable();
    assert_eq!(classes, w::BONUS_OBJECTS);
    assert!(
        w::BONUS_OBJECTS
            .iter()
            .all(|c| w::PLAIN_OBJECTS.contains(c))
    );
    // The bodies: each garrison's owner, army and flag; no bonus visitable has one.
    let seven = [(0, 1), (2, 2), (4, 3), (6, 4), (8, 5), (10, 6), (144, 200)];
    let bodies = [
        (0, model_army(&[(0, 10), (2, 5)]), true),
        (0xff, model_army(&[]), true),
        (1, model_army(&[(4, 3)]), false),
        (0xff, model_army(&seven), false),
    ];
    for (i, (owner, army, removable)) in bodies.into_iter().enumerate() {
        let want = Body::Garrison {
            owner,
            army,
            removable,
        };
        assert_eq!(m.objects[1 + i].body, want, "garrison {i}");
    }
    assert!(m.objects[5..].iter().all(|o| o.body == Body::None));
    // The templates as the game's (survey §1): a horizontal garrison blocks its tile and the two
    // west of it, the middle one entered; a vertical one its tile and the two above, the middle
    // one entered, both on any terrain; the Spring entered on two tiles, the Watering Hole on
    // four; the water ones on water only.
    let masks = |i: usize| {
        let t = template_of(i);
        (
            [t.passable[3], t.passable[4], t.passable[5]],
            [t.visitable[4], t.visitable[5]],
            t.terrains,
        )
    };
    assert_eq!(masks(1), ([0xff, 0xff, 0x1f], [0x00, 0x40], 0x1ff));
    assert_eq!(masks(2), masks(1));
    assert_eq!(masks(3), ([0x7f, 0x7f, 0x7f], [0x80, 0x00], 0x1ff));
    assert_eq!(masks(4), masks(3));
    assert_eq!(masks(10), ([0xff, 0xff, 0x3f], [0x00, 0xc0], 0x1ff));
    assert_eq!(masks(16), ([0xff, 0xff, 0x0f], [0x00, 0xf0], 0x1ff));
    assert_eq!(masks(17), ([0xff, 0xff, 0x7f], [0x00, 0x80], 0x100));
    assert_eq!(masks(19), ([0xff, 0xff, 0x1f], [0x00, 0x40], 0x100));
}

#[test]
fn an_roe_map_s_garrisons_hold_no_removable_byte() {
    // RoE without AB's vertical garrisons: the stacks' one-byte codes and no removable byte read
    // back as written (the reader's layout by version: every RoE garrison removable).
    let mut map = increment_12_map();
    map.objects.drain(3..5);
    let m = read_back(&w::write_in(&map, w::Format::RoE).unwrap());
    assert_eq!(
        m.objects[1].body,
        Body::Garrison {
            owner: 0,
            army: model_army(&[(0, 10), (2, 5)]),
            removable: true
        }
    );
    assert_eq!(
        m.objects[2].body,
        Body::Garrison {
            owner: 0xff,
            army: model_army(&[]),
            removable: true
        }
    );
    assert!(m.objects[3..].iter().all(|o| o.body == Body::None));
}

#[test]
fn the_writer_refuses_a_garrison_the_game_cannot_load() {
    let [horizontal, anti_magic, vertical, _] = GARRISONS;
    let refused = |object: w::MapObject, what: &str| {
        let mut map = increment_12_map();
        map.objects.push(object);
        match w::write(&map) {
            Err(w::WriteError::OutOfRange { what: got, .. }) => assert_eq!(got, what),
            other => panic!("{what}: {other:?}"),
        }
    };
    // A garrison's body on another class or past subclass 1; an owner nobody plays; a stack the
    // format cannot name or one of no creature.
    let pikeman = "AvWPike.def 011111111111111111111111111111111111111111111111 \
                   100000000000000000000000000000000000000000000000 111111111 000000100 \
                   54 0 2 0";
    refused(
        garrison((20, 30), pikeman, None, &[], true),
        "garrison class",
    );
    let third = anti_magic.replace(" 33 1 0 0", " 33 2 0 0");
    refused(
        garrison((20, 30), &third, None, &[], true),
        "garrison class",
    );
    refused(
        garrison((20, 30), horizontal, Some(2), &[], true),
        "object owner (no playable player)",
    );
    refused(
        garrison((20, 30), horizontal, None, &[(145, 1)], true),
        "creature",
    );
    refused(
        garrison((20, 30), horizontal, None, &[(0, 0)], true),
        "stack size",
    );
    // A garrison's body is the garrisons' only: an owner alone on class 33 is refused.
    refused(
        w::MapObject::Owned(w::OwnedObject {
            tile: (20, 30),
            template: template(horizontal),
            body: w::OwnedBody::Owner(None),
        }),
        "owned object class",
    );
    // RoE: no class 219, every garrison removable, RoE's creatures only.
    let roe = |map: &w::Map| match w::write_in(map, w::Format::RoE) {
        Err(w::WriteError::OutOfRange { what, .. }) => what,
        other => panic!("{other:?}"),
    };
    let mut map = increment_12_map();
    assert_eq!(roe(&map), "AB's vertical garrison in an RoE map");
    map.objects.drain(3..5);
    map.objects
        .push(garrison((20, 30), horizontal, None, &[], false));
    assert_eq!(roe(&map), "a garrison not removable in an RoE map");
    map.objects.pop();
    map.objects
        .push(garrison((20, 30), horizontal, None, &[(118, 1)], true));
    assert_eq!(roe(&map), "creature");
    // SoD takes a vertical garrison that is not removable.
    map.objects.pop();
    map.objects
        .push(garrison((20, 30), vertical, None, &[], false));
    w::write(&map).unwrap();
}

/// The creature type of an original creature code in SoD's rules.
fn creature(code: u16) -> CreatureTypeId {
    let i = codes::CREATURES
        .binary_search_by_key(&code, |&(id, _)| id)
        .unwrap();
    SOD.core().creature_by_key(codes::CREATURES[i].1).unwrap()
}

/// The troops of `stacks` (code, count) in slots from 0.
fn troops(stacks: &[(u16, u32)]) -> Vec<ArmyStack> {
    stacks
        .iter()
        .zip(0u8..)
        .map(|(&(code, count), slot)| ArmyStack {
            slot,
            creature: creature(code),
            count,
        })
        .collect()
}

// covers: ADV-OBJ-002, ADV-GAR-001, ADV-BON-001
#[test]
fn the_import_types_increment_12_s_classes() {
    // E08 T113b (increment 12's design note §4): each of the seventeen classes imports as its
    // type, a garrison with its owner, troops and removable flag; the setup validates.
    let raw = read_back(&w::write(&increment_12_map()).unwrap());
    let setup = import(&raw, &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    let part = SOD.adventure();
    let keys: Vec<&str> = setup
        .map
        .objects
        .iter()
        .map(|o| part.object_type(o.object_type.unwrap()).unwrap().key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "base:garrison",
            "base:anti_magic_garrison",
            "ab:garrison_vertical",
            "ab:anti_magic_garrison_vertical",
            "base:swan_pond",
            "base:faerie_ring",
            "base:fountain_of_fortune",
            "base:fountain_of_youth",
            "base:idol_of_fortune",
            "base:magic_spring",
            "base:magic_well",
            "base:oasis",
            "base:rally_flag",
            "base:stables",
            "base:temple",
            "base:watering_hole",
            "base:buoy",
            "base:mermaids",
            "base:sirens",
        ]
    );
    let red = Some(PlayerId::new(0));
    let blue = Some(PlayerId::new(1));
    let seven = [(0, 1), (2, 2), (4, 3), (6, 4), (8, 5), (10, 6), (144, 200)];
    let garrisons: Vec<_> = setup.map.objects[..4]
        .iter()
        .map(|o| (o.owner, o.holds.clone()))
        .collect();
    let held = |stacks: &[(u16, u32)], removable| Holds::Garrison {
        army: troops(stacks),
        removable,
    };
    assert_eq!(
        garrisons,
        [
            (red, held(&[(0, 10), (2, 5)], true)),
            (None, held(&[], true)),
            (blue, held(&[(4, 3)], false)),
            (None, held(&seven, false)),
        ]
    );
    // The fifteen hold nothing and have no owner.
    assert!(
        setup.map.objects[4..]
            .iter()
            .all(|o| o.holds == Holds::Nothing && o.owner.is_none())
    );
    // A count of 0 is an empty slot, a random stack's too; the count's 16 bits are read whole
    // (the setup's army rules then bound it).
    let mut edited = raw.clone();
    let Body::Garrison { army, .. } = &mut edited.objects[1].body else {
        unreachable!()
    };
    army[0] = Some(Stack {
        creature: 0,
        count: 0,
    });
    army[2] = Some(Stack {
        creature: 0xFFFE,
        count: 0,
    });
    army[3] = Some(Stack {
        creature: 6,
        count: u16::MAX,
    });
    let setup = import(&edited, &SOD).unwrap();
    assert_eq!(
        setup.map.objects[0].holds,
        Holds::Garrison {
            army: vec![
                ArmyStack {
                    slot: 1,
                    creature: creature(2),
                    count: 5,
                },
                ArmyStack {
                    slot: 3,
                    creature: creature(6),
                    count: u32::from(u16::MAX),
                },
            ],
            removable: true,
        }
    );
    // A random stack is refused for good (decision 299; RoE's codes are a byte's).
    let Body::Garrison { army, .. } = &mut edited.objects[1].body else {
        unreachable!()
    };
    army[2] = Some(Stack {
        creature: 0xFFFE,
        count: 1,
    });
    assert_eq!(import(&edited, &SOD).err(), Some(ImportError::RandomStack(1)));
    // An owner nobody plays is refused (ADV-OWN-002), the garrison's subclass past 1 and a bonus
    // visitable's past 0 are none their classes define.
    let mut edited = raw.clone();
    let Body::Garrison { owner, .. } = &mut edited.objects[1].body else {
        unreachable!()
    };
    *owner = 5;
    assert_eq!(import(&edited, &SOD).err(), Some(ImportError::ObjectOwner(1)));
    for (index, subclass) in [(1, 2), (4, 2), (5, 1), (19, 1)] {
        let mut edited = raw.clone();
        let template = usize::try_from(edited.objects[index].template).unwrap();
        edited.templates[template].subclass = subclass;
        assert_eq!(
            import(&edited, &SOD).err(),
            Some(ImportError::Subclass(index)),
            "{index}"
        );
    }
    // A war machine in a garrison is an army no garrison holds (the setup's check).
    let mut edited = raw.clone();
    let Body::Garrison { army, .. } = &mut edited.objects[1].body else {
        unreachable!()
    };
    army[0] = Some(Stack {
        creature: 146,
        count: 1,
    });
    assert_eq!(
        import(&edited, &SOD).unwrap().validate(&SOD),
        Err(GameSetupError::Adventure(SetupError::Holds(0)))
    );
    // An RoE map's horizontal garrisons (removable, the byte absent) and bonus visitables import
    // under RoE's rules alike.
    let mut map = increment_12_map();
    map.objects.drain(3..5);
    let raw = read_back(&w::write_in(&map, w::Format::RoE).unwrap());
    let setup = import(&raw, &ROE).unwrap();
    setup.validate(&ROE).unwrap();
    assert!(setup.map.objects.iter().all(|o| o.object_type.is_some()));
    assert_eq!(
        setup.map.objects[1].holds,
        Holds::Garrison {
            army: Vec::new(),
            removable: true
        }
    );
}
