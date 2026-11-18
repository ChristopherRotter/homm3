//! The experiment writer's Tavern and Den of Thieves (E09 T34: batch TWN-4's maps; increment 4's
//! design note `docs/epics/E09-design.md` §78, survey `docs/rules/surveys/E09-tavern.md` §3, §6):
//! the two classes without a body, read back as written in SoD and RoE, their templates' masks as
//! the game's; and the import, which since E09 T39a (map import v26; design note §66, §74) types
//! them: the map's Tavern and the Den of Thieves, no body, no owner, the setup valid.

use super::import::{ROE, SOD};
use super::limits;
use crate::model::{Body, RawMap};
use crate::writer as w;
use crate::{import, read};
use h3_adventure::Holds;

/// The game's `Objects.txt` lines (`H3bitmap.lod`'s): the Tavern (95), entered at the tile west of
/// its anchor, and the Den of Thieves (97), entered at its anchor.
const PICTURES: [&str; 2] = [
    "AVXtvrn0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 95 0 0 0",
    "AVXdend0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 97 0 0 0",
];

/// The objects' tiles.
const TILES: [(u8, u8); 2] = [(8, 6), (16, 6)];

/// A 36 × 36 grass map, Red and Blue playing: Red's hero at (2, 30), the Tavern, the Den.
fn taverns_map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut objects = vec![super::economy::hero((2, 30), 0, 0)];
    objects.extend(PICTURES.iter().zip(TILES).map(|(line, tile)| {
        w::MapObject::Plain(w::Plain {
            tile,
            template: w::ObjectTemplate::from_objects_txt(line).unwrap(),
        })
    }));
    w::Map {
        name: "Taverns".into(),
        description: "E09 T34".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![],
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

#[test]
fn the_tavern_and_the_den_read_back_as_written() {
    let map = taverns_map();
    assert!(w::PLAIN_OBJECTS.contains(&w::TAVERN_OBJECT));
    assert!(w::PLAIN_OBJECTS.contains(&w::DEN_OF_THIEVES_OBJECT));
    for format in [w::Format::Sod, w::Format::RoE] {
        let m = read_back(&w::write_in(&map, format).unwrap());
        let placed: Vec<_> = m.objects[1..]
            .iter()
            .map(|o| {
                let t = &m.templates[usize::try_from(o.template).unwrap()];
                (t.class, t.subclass, (o.pos.x, o.pos.y))
            })
            .collect();
        assert_eq!(
            placed,
            [
                (w::TAVERN_OBJECT, 0, TILES[0]),
                (w::DEN_OF_THIEVES_OBJECT, 0, TILES[1])
            ],
            "{format:?}"
        );
        // No body: the game's maps hold none for either class.
        assert!(m.objects[1..].iter().all(|o| o.body == Body::None));
        // The templates as the game's: the anchor and its west neighbour blocked on the bottom
        // row, the Tavern entered at the west one, the Den at the anchor; every terrain, the
        // editor's menu every land terrain.
        let template_of =
            |i: usize| &m.templates[usize::try_from(m.objects[1 + i].template).unwrap()];
        assert_eq!(
            [template_of(0).visitable[5], template_of(1).visitable[5]],
            [0x40, 0x80]
        );
        for i in 0..2 {
            let t = template_of(i);
            assert_eq!(t.passable[5], 0x3f, "{i}");
            assert!(t.visitable[..5].iter().all(|&b| b == 0), "{i}");
            assert_eq!((t.terrains, t.editor_terrains), (0x1ff, 0xff), "{i}");
        }
    }
}

// covers: ADV-OBJ-001, ADV-VIS-007, ADV-VIS-008
#[test]
fn the_import_types_the_tavern_and_the_den() {
    // E09 T39a (design note §66, §74): the two classes import as the base pack's Tavern and Den
    // of Thieves, without a body or an owner, their template's blocked tiles and the one entrance
    // as the footprint; the setup validates under SoD's and RoE's rules.
    let map = taverns_map();
    for (format, rules) in [(w::Format::Sod, &*SOD), (w::Format::RoE, &*ROE)] {
        let raw = read_back(&w::write_in(&map, format).unwrap());
        let setup = import(&raw, rules).unwrap();
        setup.validate(rules).unwrap();
        assert_eq!(setup.map.objects.len(), 2);
        let types = ["base:tavern", "base:den_of_thieves"]
            .map(|k| rules.adventure().object_type_by_key(k));
        assert!(types.iter().all(Option::is_some));
        for ((object, (x, y)), ty) in setup.map.objects.iter().zip(TILES).zip(types) {
            assert_eq!(object.object_type, ty);
            assert_eq!(object.holds, Holds::Nothing);
            assert_eq!(object.owner, None);
            assert_eq!(
                (object.anchor.x, object.anchor.y),
                (u16::from(x), u16::from(y))
            );
            let entrance: Vec<_> = object.visitable.offsets().collect();
            assert_eq!(entrance.len(), 1);
            let (column, row) = entrance[0];
            assert!(object.blocked.contains(column, row));
        }
    }
}
