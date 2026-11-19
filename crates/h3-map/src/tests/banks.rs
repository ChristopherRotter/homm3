//! The experiment writer's creature banks (E08 T119: batch E13's maps; increment 13's design note
//! §3, §19, survey `docs/rules/surveys/E08-banks.md` §1): the five classes without a body, class
//! 16 by subclass 0 to 6, the Crypt's three pictures and the Shipwreck's two, read back as written
//! in SoD and RoE, their templates' masks as the game's; a subclass of 16 past 6 refused. Since
//! T123b (design note §3, map import v24) the import types them, class 16 by subclass and the four
//! others at subclass 0, without a body (another subclass refused), the setup valid; since T124b2
//! a step onto a land bank's entrance visits it (its question by its kind, then its battle).

use super::import::{ROE, SOD};
use super::limits;
use crate::model::{Body, RawMap};
use crate::writer as w;
use crate::{ImportError, import, read};
use h3_adventure::{AdventureCommand, Holds, OfferKind, Pos};
use h3_contracts::{BattleSide, PlayerId};
use h3_game::{Actor, Answer, Consent, GameCommand, GameState, apply};

/// The game's `Objects.txt` lines of the land banks (`H3bitmap.lod`'s, RoE's as SoD's): class 16
/// by subclass, the Cyclops Stockpile (0), the Dwarven Treasury (1), the Griffin Conservatory (2),
/// the Imp Cache (3), the Medusa Stores (4), the Naga Bank (5) and the Dragon Fly Hive (6); the
/// Crypt (84) by its second terrain mask, sand, dirt, grass and swamp, and snow (the first
/// allows every terrain); Dragon Utopia (25).
pub(super) const LAND_BANKS: [&str; 11] = [
    "AVXbnk10.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 16 0 0 0",
    "AVXbnk20.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 16 1 0 0",
    "AVXbnk30.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 16 2 0 0",
    "AVXbnk40.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 16 3 0 0",
    "AVXbnk50.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 010111111 16 4 0 0",
    "AVXbnk60.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 16 5 0 0",
    "AVXbnk70.def 101111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 16 6 0 0",
    "AVXgyds0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000000010 84 0 0 0",
    "AVXgyne0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000010101 84 0 0 0",
    "AVXgysn0.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 000001000 84 0 0 0",
    "AVSutop0.def 100011111000001100001111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 25 0 0 0",
];

/// The game's `Objects.txt` lines of the sea banks: the Derelict Ship (24) and the Shipwreck's
/// two pictures (85), entered at the anchor's west neighbour and at the anchor.
pub(super) const WATER_BANKS: [&str; 3] = [
    "AVAdlic0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 24 0 0 0",
    "AVAwrek0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 100000000 100000000 85 0 0 0",
    "AVAwre20.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 100000000 100000000 85 0 0 0",
];

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

fn plain(tile: (u8, u8), line: &str) -> w::MapObject {
    w::MapObject::Plain(w::Plain {
        tile,
        template: template(line),
    })
}

/// The land banks' tiles: three a row from (6, 4), six columns and four rows apart.
fn land_tile(i: u8) -> (u8, u8) {
    (6 + 6 * (i % 3), 4 + 4 * (i / 3))
}

/// The sea banks' tiles.
const WATER_TILES: [(u8, u8); 3] = [(28, 6), (28, 12), (28, 18)];

/// A 36 × 36 grass map, columns 24 to 35 water, Red and Blue playing: Red's hero at (2, 30), the
/// land banks, then the sea banks.
fn banks_map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut objects = vec![super::economy::hero((2, 30), 0, 0)];
    objects.extend(
        LAND_BANKS
            .iter()
            .zip(0u8..)
            .map(|(line, i)| plain(land_tile(i), line)),
    );
    objects.extend(
        WATER_BANKS
            .iter()
            .zip(WATER_TILES)
            .map(|(line, tile)| plain(tile, line)),
    );
    w::Map {
        name: "Creature banks".into(),
        description: "E08 T119".into(),
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
        banned_artifacts: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

fn read_back(bytes: &[u8]) -> RawMap {
    read(&w::gzip(bytes), &limits()).unwrap()
}

/// Each object's class, subclass and tile after the hero.
fn placed(m: &RawMap) -> Vec<(u32, u32, (u8, u8))> {
    m.objects[1..]
        .iter()
        .map(|o| {
            let t = &m.templates[usize::try_from(o.template).unwrap()];
            (t.class, t.subclass, (o.pos.x, o.pos.y))
        })
        .collect()
}

fn expected() -> Vec<(u32, u32, (u8, u8))> {
    let land = [
        (16, 0),
        (16, 1),
        (16, 2),
        (16, 3),
        (16, 4),
        (16, 5),
        (16, 6),
        (84, 0),
        (84, 0),
        (84, 0),
        (25, 0),
    ];
    let mut expected: Vec<_> = land
        .iter()
        .zip(0u8..)
        .map(|(&(class, subclass), i)| (class, subclass, land_tile(i)))
        .collect();
    expected.extend(
        [(24, 0), (85, 0), (85, 0)]
            .iter()
            .zip(WATER_TILES)
            .map(|(&(class, subclass), tile)| (class, subclass, tile)),
    );
    expected
}

#[test]
fn the_banks_read_back_as_written() {
    let map = banks_map();
    for format in [w::Format::Sod, w::Format::RoE] {
        let m = read_back(&w::write_in(&map, format).unwrap());
        assert_eq!(placed(&m), expected(), "{format:?}");
        // No body: the game's maps hold none for the five classes (survey §1).
        assert!(m.objects[1..].iter().all(|o| o.body == Body::None));
    }
    assert!(w::BANK_OBJECTS.iter().all(|c| w::PLAIN_OBJECTS.contains(c)));
    let m = read_back(&w::write(&map).unwrap());
    let template_of = |i: usize| &m.templates[usize::try_from(m.objects[1 + i].template).unwrap()];
    // The templates as the game's (survey §1): one visitable tile on the bottom row, at the
    // anchor or the tile west of it, which the template also blocks; the first terrain mask every
    // terrain for the land banks, water for the ships; the second (the editor's menu, VCMI's
    // allowed terrains) every land terrain but the Medusa Stores' subterranean, each Crypt's its
    // group.
    let entrance = |i: usize| {
        let t = template_of(i);
        let row = t.visitable[5];
        assert_eq!(row.count_ones(), 1, "bank {i}");
        assert!(t.visitable[..5].iter().all(|&b| b == 0), "bank {i}");
        assert_eq!(t.passable[5] & row, 0, "bank {i} blocks its entrance");
        row
    };
    let at_anchor = 0x80;
    let west = 0x40;
    let entrances: Vec<u8> = (0..LAND_BANKS.len() + WATER_BANKS.len())
        .map(entrance)
        .collect();
    assert_eq!(
        entrances,
        [
            west, west, at_anchor, at_anchor, west, west, west, west, west, west, west, west, west,
            at_anchor
        ]
    );
    let land = LAND_BANKS.len();
    assert!((0..land).all(|i| template_of(i).terrains == 0x1ff));
    assert!((land..entrances.len()).all(|i| template_of(i).terrains == 0x100));
    let menus: Vec<u16> = (0..land).map(|i| template_of(i).editor_terrains).collect();
    assert_eq!(
        menus,
        [
            0xff, 0xff, 0xff, 0xff, 0xbf, 0xff, 0xff, 0x002, 0x015, 0x008, 0xff
        ]
    );
    // The Dragon Fly Hive leaves its anchor passable; Dragon Utopia blocks three tiles of the
    // bottom row west of its anchor, five of the row above and four of the third.
    assert_eq!(template_of(6).passable[5], 0xbf);
    let utopia = template_of(10);
    assert_eq!(
        [utopia.passable[3], utopia.passable[4], utopia.passable[5]],
        [0x0f, 0x83, 0x8f]
    );
}

#[test]
fn the_writer_refuses_a_bank_subclass_past_6() {
    // The game indexes the kinds' table by class 16's subclass without a test (survey §3).
    let mut map = banks_map();
    let w::MapObject::Plain(plain) = &mut map.objects[7] else {
        unreachable!()
    };
    assert_eq!(plain.template.subclass, 6);
    plain.template.subclass = 7;
    assert_eq!(
        w::write(&map),
        Err(w::WriteError::OutOfRange {
            what: "creature bank subclass",
            value: 7
        })
    );
    // The other classes' subclasses are their templates' (subclass 0 in the game's).
    let w::MapObject::Plain(plain) = &mut map.objects[7] else {
        unreachable!()
    };
    plain.template.subclass = 6;
    let w::MapObject::Plain(crypt) = &mut map.objects[9] else {
        unreachable!()
    };
    crypt.template.subclass = 7;
    assert!(w::write(&map).is_ok());
}

/// The bank types by object, in [`expected`]'s order.
const TYPES: [&str; 14] = [
    "base:cyclops_stockpile",
    "base:dwarven_treasury",
    "base:griffin_conservatory",
    "base:imp_cache",
    "base:medusa_stores",
    "base:naga_bank",
    "base:dragon_fly_hive",
    "base:crypt",
    "base:crypt",
    "base:crypt",
    "base:dragon_utopia",
    "base:derelict_ship",
    "base:shipwreck",
    "base:shipwreck",
];

// covers: ADV-OBJ-002, ADV-BNK-001
#[test]
fn the_import_types_the_banks() {
    // The five classes typed by the original's kinds (increment 13's design note §3): class 16 by
    // subclass, 84, 25, 24 and 85 at subclass 0; without a body (nothing held, no owner) and with
    // their template's blocked tiles and one entrance; the setup valid under SoD's and RoE's rules.
    let map = banks_map();
    for (format, rules) in [(w::Format::Sod, &*SOD), (w::Format::RoE, &*ROE)] {
        let part = rules.adventure();
        let raw = read_back(&w::write_in(&map, format).unwrap());
        let setup = import(&raw, rules).unwrap();
        setup.validate(rules).unwrap();
        assert_eq!(setup.map.objects.len(), TYPES.len());
        for ((object, (_, _, (x, y))), key) in setup.map.objects.iter().zip(expected()).zip(TYPES) {
            assert_eq!(object.object_type, part.object_type_by_key(key), "{key}");
            let ty = part.object_type(object.object_type.unwrap()).unwrap();
            assert!(
                matches!(ty.kind, h3_adventure::ObjectKind::Bank(_)),
                "{key}"
            );
            assert_eq!(object.holds, Holds::Nothing);
            assert_eq!((object.owner, object.amount), (None, None));
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
    // A subclass its class does not define: class 16's past 6 (the original indexes its kinds'
    // table by it without a test, survey §3; the writer refuses it, so the read map is edited),
    // the four others' past 0. Object 0 is the hero.
    let raw = read_back(&w::write(&map).unwrap());
    for (index, subclass) in [(7, 7), (8, 1), (11, 1), (12, 1), (13, 1)] {
        let mut edited = raw.clone();
        let template = usize::try_from(edited.objects[index].template).unwrap();
        edited.templates[template].subclass = subclass;
        assert_eq!(
            import(&edited, &SOD).err(),
            Some(ImportError::Subclass(index)),
            "{index}"
        );
    }
}

// covers: ADV-OBJ-003, ADV-BNK-001, ADV-BNK-003, ADV-BNK-004, ADV-TEL-008
#[test]
fn a_step_onto_a_land_bank_asks_by_its_kind_and_a_yes_fights_its_guards() {
    // Since T124b2 (increment 13's design note §6, §7) a bank's entrance ends a path and is
    // visited: class 16's seven kinds and Dragon Utopia mark Red at once and ask to fight, the
    // Crypt asks to search first; declined, the hero stays on the entrance and `Revisit` asks
    // again; a yes fights the guards as drawn there, unowned. Red's hero is placed on the tile
    // below each land bank's entrance (free: the banks are four rows apart).
    let rules = &*SOD;
    let raw = read_back(&w::write(&banks_map()).unwrap());
    let setup = import(&raw, rules).unwrap();
    let red = PlayerId::new(0);
    let play = |state: &mut GameState, command| {
        apply(state, rules, Actor::Player(red), command, &mut Vec::new()).unwrap();
    };
    for (index, object) in setup.map.objects[..LAND_BANKS.len()].iter().enumerate() {
        let (column, row) = object.visitable.offsets().next().unwrap();
        let entrance = Pos::new(
            object.anchor.x - u16::from(column),
            object.anchor.y - u16::from(row),
            0,
        );
        let mut setup = setup.clone();
        setup.heroes[0].pos = Pos::new(entrance.x, entrance.y + 1, 0);
        let mut state = GameState::new(rules, &setup, 1).unwrap();
        let hero = state.adventure().heroes()[0].id;
        let reach = state
            .adventure()
            .reach(rules.core(), rules.adventure(), state.heroes(), hero)
            .unwrap();
        assert!(reach.points_left(entrance).is_some(), "{index}");
        let path = vec![entrance];
        play(
            &mut state,
            GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }),
        );
        let crypt = (7..10).contains(&index);
        let asked = if crypt {
            OfferKind::Search
        } else {
            OfferKind::Fight
        };
        assert_eq!(state.decision(red).unwrap().kind, asked, "{index}");
        assert_eq!(state.adventure().knows(red, index), !crypt, "{index}");
        assert_eq!(state.adventure().heroes()[0].pos, entrance, "{index}");
        let answer = |consent| GameCommand::Answer(Answer::Offer(consent));
        play(&mut state, answer(Consent::Decline));
        assert!(state.decision(red).is_none() && state.battle(red).is_none());
        play(
            &mut state,
            GameCommand::Adventure(AdventureCommand::Revisit { hero }),
        );
        assert_eq!(state.decision(red).unwrap().kind, asked, "{index}");
        play(&mut state, answer(Consent::Accept));
        let battle = state.battle(red).unwrap();
        assert_eq!(
            (battle.tile(), battle.owners()),
            (entrance, [Some(red), None])
        );
        let guards = state.adventure().guards_left(index).to_vec();
        let defending: Vec<_> = battle
            .state()
            .stacks()
            .iter()
            .filter(|s| s.side == BattleSide::Defender)
            .map(|s| (s.creature, s.count))
            .collect();
        let drawn: Vec<_> = guards.iter().map(|s| (s.creature, s.count)).collect();
        assert_eq!(defending, drawn, "{index}");
        assert!(state.adventure().knows(red, index), "{index}");
    }
}
