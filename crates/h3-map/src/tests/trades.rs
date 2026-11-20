//! The experiment writer's trades and services (E08 T128: batch E14's maps; increment 14's design
//! note §7, §20, survey `docs/rules/surveys/E08-trades.md` §1): the seven classes without a body,
//! the nine pictures of the game's `Objects.txt` read back as written in SoD and, but for AB's
//! Freelancer's Guild (213) and snow Trading Post (221), which the writer refuses there, in RoE;
//! their templates' masks as the game's; the map's banned artifacts written beside the editor's
//! default bans, refused in RoE; and the import that types them (E08 T132b; design note §7):
//! each its type at subclass 0 with its blocked mask and its one entrance, no body, the setup
//! valid, a subclass past 0 refused, and a step onto one refused as an unbuilt visit's until T133
//! and T134 build the visits.

use super::import::{ROE, SOD};
use super::limits;
use crate::model::{Body, RawMap};
use crate::writer as w;
use crate::{ImportError, import, read};
use h3_adventure::{AdventureCommand, AdventureCommandError, Holds, ObjectKind, Pos};
use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand, GameCommandError, GameState, apply};

/// The game's `Objects.txt` lines (`H3bitmap.lod`'s): the Altar of Sacrifice (2), the Black
/// Market (7), the Hill Fort (35) on the editor's dirt and grass, the Trading Post (99) on dirt
/// and grass and on rough, the snow Trading Post (221), the War Machine Factory (106) and the
/// Freelancer's Guild (213). The first seven are entered at their anchor, the last two at the
/// tile west of it.
const PICTURES: [&str; 9] = [
    "AvXAltar.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 2 0 0 0",
    "AVXmktb0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 7 0 0 0",
    "AVXhild0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000000001 35 0 0 0",
    "AVXhilg0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000000100 35 0 0 0",
    "AVXpost0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000000101 99 0 0 0",
    "AVXpstr0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000100000 99 0 0 0",
    "AVXpsSn.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000001000 221 0 0 0",
    "AVGsieg0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 106 0 0 0",
    "avxfgld.def 000111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 213 0 0 0",
];

/// Each picture's class.
const CLASSES: [u32; 9] = [2, 7, 35, 35, 99, 99, 221, 106, 213];

/// Each picture's object type (E08 T132b): the snow Trading Post (221) the Trading Post.
const TYPES: [&str; 9] = [
    "base:altar_of_sacrifice",
    "base:black_market",
    "base:hill_fort",
    "base:hill_fort",
    "base:trading_post",
    "base:trading_post",
    "base:trading_post",
    "base:war_machine_factory",
    "ab:freelancers_guild",
];

/// The pictures' tiles: four a row from (6, 4), six columns and four rows apart.
fn tile(i: usize) -> (u8, u8) {
    let i = u8::try_from(i).unwrap();
    (6 + 6 * (i % 4), 4 + 4 * (i / 4))
}

/// A 36 × 36 grass map, Red and Blue playing: Red's hero at (2, 30), then the pictures whose
/// index `keep` takes, at their tiles.
fn trades_map(keep: impl Fn(usize) -> bool) -> w::Map {
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
        PICTURES
            .iter()
            .enumerate()
            .filter(|&(i, _)| keep(i))
            .map(|(i, line)| {
                w::MapObject::Plain(w::Plain {
                    tile: tile(i),
                    template: w::ObjectTemplate::from_objects_txt(line).unwrap(),
                })
            }),
    );
    w::Map {
        name: "Trades".into(),
        description: "E08 T128".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![],
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

/// The pictures an RoE map may hold: all but AB's two.
fn in_roe(i: usize) -> bool {
    !w::AB_TRADE_OBJECTS.contains(&CLASSES[i])
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

fn expected(keep: impl Fn(usize) -> bool) -> Vec<(u32, u32, (u8, u8))> {
    (0..PICTURES.len())
        .filter(|&i| keep(i))
        .map(|i| (CLASSES[i], 0, tile(i)))
        .collect()
}

#[test]
fn the_trades_read_back_as_written() {
    assert!(
        w::TRADE_OBJECTS
            .iter()
            .all(|c| w::PLAIN_OBJECTS.contains(c))
    );
    assert!(
        w::AB_TRADE_OBJECTS
            .iter()
            .all(|c| w::TRADE_OBJECTS.contains(c))
    );
    assert!(CLASSES.iter().all(|c| w::TRADE_OBJECTS.contains(c)));
    for (format, keep) in [
        (w::Format::Sod, (|_| true) as fn(usize) -> bool),
        (w::Format::RoE, in_roe),
    ] {
        let m = read_back(&w::write_in(&trades_map(keep), format).unwrap());
        assert_eq!(placed(&m), expected(keep), "{format:?}");
        // No body: the game's maps hold none for the seven classes (survey §1).
        assert!(m.objects[1..].iter().all(|o| o.body == Body::None));
    }
    // The templates as the game's (survey §1): one visitable tile on the bottom row, which the
    // template also blocks, at the anchor for the first seven, west of it for the factory and
    // the guild; the anchor and the tile west of it blocked, the guild also the second west;
    // every terrain; the editor's menu every land terrain, but the Hill Fort's dirt or grass and
    // the Trading Post's dirt and grass, rough or snow.
    let m = read_back(&w::write(&trades_map(|_| true)).unwrap());
    let template_of = |i: usize| &m.templates[usize::try_from(m.objects[1 + i].template).unwrap()];
    let (at_anchor, west) = (0x80, 0x40);
    for i in 0..PICTURES.len() {
        let t = template_of(i);
        assert!(t.visitable[..5].iter().all(|&b| b == 0), "{i}");
        assert!(t.passable[..5].iter().all(|&b| b == 0xff), "{i}");
        assert_eq!(t.passable[5] & t.visitable[5], 0, "{i} blocks its entrance");
        assert_eq!(t.terrains, 0x1ff, "{i}");
    }
    let entrances: Vec<u8> = (0..PICTURES.len())
        .map(|i| template_of(i).visitable[5])
        .collect();
    assert_eq!(
        entrances,
        [
            at_anchor, at_anchor, at_anchor, at_anchor, at_anchor, at_anchor, at_anchor, west, west
        ]
    );
    let blocked: Vec<u8> = (0..PICTURES.len())
        .map(|i| template_of(i).passable[5])
        .collect();
    assert_eq!(
        blocked,
        [0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x1f]
    );
    let menus: Vec<u16> = (0..PICTURES.len())
        .map(|i| template_of(i).editor_terrains)
        .collect();
    assert_eq!(
        menus,
        [0xff, 0xff, 0x001, 0x004, 0x005, 0x020, 0x008, 0xff, 0xff]
    );
}

#[test]
fn the_writer_refuses_ab_s_trades_in_roe() {
    // AB added the Freelancer's Guild and the snow Trading Post (survey §1: no RoE map holds
    // one): the first such object refuses the RoE map, in the order written.
    let refused = |keep: fn(usize) -> bool| w::write_in(&trades_map(keep), w::Format::RoE);
    let refusal = |value| {
        Err(w::WriteError::OutOfRange {
            what: "an AB trade in an RoE map",
            value,
        })
    };
    assert_eq!(refused(|_| true), refusal(221));
    assert_eq!(refused(|i| CLASSES[i] != 221), refusal(213));
    assert!(refused(in_roe).is_ok());
}

// covers: GAM-SET-003
#[test]
fn the_map_s_banned_artifacts_are_written_beside_the_default_ones() {
    // E14-3's ban (survey §7.1): a set bit per code below the format's 144 artifacts, beside
    // the editor's default 127 to 143; none in RoE, which holds no banned artifacts.
    let mut map = trades_map(in_roe);
    map.banned_artifacts = vec![0, 7, 126, 143];
    let m = read_back(&w::write(&map).unwrap());
    let banned: Vec<usize> = (0..144).filter(|&a| m.banned_artifacts.get(a)).collect();
    assert_eq!(
        banned,
        [0, 7].into_iter().chain(126..144).collect::<Vec<_>>()
    );
    let plain = read_back(&w::write(&trades_map(in_roe)).unwrap());
    let banned: Vec<usize> = (0..144)
        .filter(|&a| plain.banned_artifacts.get(a))
        .collect();
    assert_eq!(banned, (127..144).collect::<Vec<_>>());
    // The import names each banned artifact a code names (GAM-SET-003): the map's three
    // before the default ones (code 143 is one of the three unused records).
    let setup = import(&m, &SOD).unwrap();
    let defaults = import(&plain, &SOD).unwrap().banned_artifacts;
    let core = SOD.core();
    let artifact = |key| core.artifact_by_key(key).unwrap();
    assert_eq!(
        setup.banned_artifacts[..3],
        [
            artifact("base:spell_book"),
            artifact("base:centaurs_axe"),
            artifact("base:orb_of_inhibition")
        ]
    );
    assert_eq!(setup.banned_artifacts[3..], defaults[..]);
    map.banned_artifacts = vec![0, 144];
    assert_eq!(
        w::write(&map),
        Err(w::WriteError::OutOfRange {
            what: "banned artifact",
            value: 144
        })
    );
    map.banned_artifacts = vec![7];
    assert_eq!(
        w::write_in(&map, w::Format::RoE),
        Err(w::WriteError::OutOfRange {
            what: "banned artifact in an RoE map",
            value: 7
        })
    );
}

// covers: ADV-OBJ-002, ADV-MKT-001, ADV-ALT-001, ADV-WMF-001
#[test]
fn the_import_types_the_trades() {
    // The seven classes at subclass 0 (increment 14's design note §7): 2 the Altar of Sacrifice,
    // 7 the Black Market, 35 the Hill Fort, 99 and 221 the Trading Post, 106 the War Machine
    // Factory and 213 AB's Freelancer's Guild, each of its kind; without a body (nothing held, no
    // owner) and with their template's blocked tiles and one entrance; the setup valid under
    // SoD's and RoE's rules (an RoE map holds neither 213 nor 221).
    for (format, rules, keep) in [
        (w::Format::Sod, &*SOD, (|_| true) as fn(usize) -> bool),
        (w::Format::RoE, &*ROE, in_roe),
    ] {
        let part = rules.adventure();
        let raw = read_back(&w::write_in(&trades_map(keep), format).unwrap());
        let setup = import(&raw, rules).unwrap();
        setup.validate(rules).unwrap();
        let kept: Vec<usize> = (0..PICTURES.len()).filter(|&i| keep(i)).collect();
        assert_eq!(setup.map.objects.len(), kept.len(), "{format:?}");
        for (object, i) in setup.map.objects.iter().zip(kept) {
            let key = TYPES[i];
            assert_eq!(object.object_type, part.object_type_by_key(key), "{key}");
            let kind = &part.object_type(object.object_type.unwrap()).unwrap().kind;
            let of_its_kind = match CLASSES[i] {
                2 => matches!(kind, ObjectKind::Altar(_)),
                35 => matches!(kind, ObjectKind::HillFort(_)),
                106 => matches!(kind, ObjectKind::WarMachineFactory(_)),
                _ => matches!(kind, ObjectKind::Market(_)),
            };
            assert!(of_its_kind, "{key}");
            assert_eq!(object.holds, Holds::Nothing);
            assert_eq!((object.owner, object.amount), (None, None));
            let (x, y) = tile(i);
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
    // A subclass past 0 is one the class does not define (the writer writes 0 alone, so the read
    // map is edited). Object 0 is the hero.
    let raw = read_back(&w::write(&trades_map(|_| true)).unwrap());
    for index in 1..=PICTURES.len() {
        let mut edited = raw.clone();
        let template = usize::try_from(edited.objects[index].template).unwrap();
        edited.templates[template].subclass = 1;
        assert_eq!(
            import(&edited, &SOD).err(),
            Some(ImportError::Subclass(index)),
            "{index}"
        );
    }
}

// covers: ADV-OBJ-003, ADV-MKT-001
#[test]
fn a_step_onto_a_market_enters_it_and_onto_another_trade_is_refused_until_t134() {
    // A market's visit is built (E08 T133a2): the step onto its entrance ends the move there (its
    // window opens no decision until T133b, ADV-STUB-18). Until T134 builds the Altar's, the Hill
    // Fort's and the factory's (increment 14's design note §10), their entrance is refused as any
    // unbuilt visit's (`effect` none): Red's hero, placed on the tile below each entrance (free:
    // the pictures are four rows apart), reaches no point on it, the step onto it is `Visit` and
    // the state stays as it was.
    let rules = &*SOD;
    let raw = read_back(&w::write(&trades_map(|_| true)).unwrap());
    let setup = import(&raw, rules).unwrap();
    let red = PlayerId::new(0);
    assert_eq!(setup.map.objects.len(), PICTURES.len());
    let mut markets = 0;
    for (index, object) in setup.map.objects.iter().enumerate() {
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
        let before = state.state_hash();
        let path = vec![entrance];
        let stepped = apply(
            &mut state,
            rules,
            Actor::Player(red),
            GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }),
            &mut Vec::new(),
        );
        let kind = object
            .object_type
            .and_then(|ty| rules.adventure().object_type(ty))
            .map(|ty| &ty.kind);
        if let Some(ObjectKind::Market(_)) = kind {
            markets += 1;
            assert!(reach.points_left(entrance).is_some(), "{index}");
            assert!(stepped.is_ok(), "{index}");
            assert_eq!(state.adventure().heroes()[0].pos, entrance, "{index}");
            continue;
        }
        assert_eq!(reach.points_left(entrance), None, "{index}");
        assert!(
            matches!(
                stepped,
                Err(GameCommandError::Adventure(AdventureCommandError::Visit {
                    step: 0
                }))
            ),
            "{index}"
        );
        assert_eq!(state.state_hash(), before, "{index}");
    }
    // The Black Market, the two Trading Posts' classes and the guild's pictures.
    assert!(markets >= 4, "{markets}");
}
