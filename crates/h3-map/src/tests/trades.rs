//! The experiment writer's trades and services (E08 T128: batch E14's maps; increment 14's design
//! note §7, §20, survey `docs/rules/surveys/E08-trades.md` §1): the seven classes without a body,
//! the nine pictures of the game's `Objects.txt` read back as written in SoD and, but for AB's
//! Freelancer's Guild (213) and snow Trading Post (221), which the writer refuses there, in RoE;
//! their templates' masks as the game's; the map's banned artifacts written beside the editor's
//! default bans, refused in RoE; and the import that types them (E08 T132b; design note §7):
//! each its type at subclass 0 with its blocked mask and its one entrance, no body, the setup
//! valid, a subclass past 0 refused, and a step onto one refused as an unbuilt visit's until T133
//! and T134 built the visits (the markets' with T133, the Altar's with T134a, the Hill Fort's with
//! T134b1, the War Machine Factory's with T134b2: since then every step onto one enters it).

use super::import::{ROE, SOD};
use super::limits;
use crate::model::{Body, RawMap};
use crate::writer as w;
use crate::{ImportError, import, read};
use h3_adventure::{AdventureCommand, AdventureCommandError, Holds, ObjectKind, ObjectSetup, Pos};
use h3_contracts::{ArmyStack, PlayerId};
use h3_game::{Actor, GameCommand, GameCommandError, GameState, HeroArmy, apply};

/// The game's `Objects.txt` lines (`H3bitmap.lod`'s): the Altar of Sacrifice (2), the Black
/// Market (7), the Hill Fort (35) on the editor's dirt and grass, the Trading Post (99) on dirt
/// and grass and on rough, the snow Trading Post (221), the War Machine Factory (106) and the
/// Freelancer's Guild (213). The first seven are entered at their anchor, the last two at the
/// tile west of it. The trades maps lay them out (E08 T135b1: `trades_map.rs`,
/// `trades_xl_map.rs`).
pub(super) const PICTURES: [&str; 9] = [
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
        banned_spells: Vec::new(),
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

// covers: ADV-OBJ-002, ADV-MKT-001, ADV-ALT-001, ADV-HFT-001, ADV-WMF-001
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

// covers: ADV-OBJ-003, ADV-MKT-001, ADV-ALT-001, ADV-HFT-001, ADV-HFT-002, ADV-WMF-001
// covers: ADV-HIR-002, GAM-DEC-008, GAM-DEC-011, GAM-DEC-012, GAM-DEC-013
#[test]
fn a_step_onto_each_trade_or_service_enters_it() {
    // A market's visit is built (E08 T133a2): the step onto its entrance ends the move there (its
    // window opens the market's decision since T133b, GAM-DEC-011); an Altar's since T134a (its
    // decision, GAM-DEC-012); a Hill Fort's since T134b1 (GAM-DEC-013); the War Machine Factory's
    // since T134b2 (its hire question, whose yes opens the hire dialog of the base pack's three
    // machines, GAM-DEC-008). Red's hero is placed on the tile below each entrance (free: the
    // pictures are four rows apart).
    let rules = &*SOD;
    let raw = read_back(&w::write(&trades_map(|_| true)).unwrap());
    let setup = import(&raw, rules).unwrap();
    let red = PlayerId::new(0);
    assert_eq!(setup.map.objects.len(), PICTURES.len());
    let (mut markets, mut altars, mut forts, mut factories) = (0, 0, 0, 0);
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
        assert!(reach.points_left(entrance).is_some(), "{index}");
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
        assert!(stepped.is_ok(), "{index}");
        assert_eq!(state.adventure().heroes()[0].pos, entrance, "{index}");
        if let Some(ObjectKind::Market(market)) = kind {
            markets += 1;
            assert_eq!(state.adventure().heroes()[0].pos, entrance, "{index}");
            // E08 T133b: its decision opens with the original packs' rates (a Trading Post's 42
            // trades of the seven resources, at five Marketplaces).
            assert_eq!(state.market(red).map(|visit| visit.hero), Some(Some(hero)));
            let seen = h3_game::observe(&state, rules, h3_game::Viewer::Player(red)).decision;
            let Some(h3_game::DecisionView::Market(view)) = seen else {
                panic!("{index}: {seen:?}")
            };
            assert_eq!(view.offers.screens, market.screens, "{index}");
            assert_eq!(view.offers.count, 5, "{index}");
            let trades = market.shows(h3_adventure::MarketScreen::Trade);
            let expected = if trades { 42 } else { 0 };
            assert_eq!(view.offers.trades.len(), expected, "{index}");
            continue;
        }
        if let Some(ObjectKind::Altar(altar)) = kind {
            altars += 1;
            // E08 T134a: its decision opens with the original packs' values (23 places), the
            // screens the hero's class's faction allows (every faction has one at least).
            assert_eq!(state.altar(red).map(|visit| visit.hero), Some(hero));
            let seen = h3_game::observe(&state, rules, h3_game::Viewer::Player(red)).decision;
            let Some(h3_game::DecisionView::Altar(view)) = seen else {
                panic!("{index}: {seen:?}")
            };
            assert_eq!((view.offers.places, altar.places), (23, 23), "{index}");
            let screens = view.offers.screens;
            assert!(screens.artifacts || screens.creatures, "{index}");
            continue;
        }
        if let Some(ObjectKind::HillFort(fort)) = kind {
            forts += 1;
            // E08 T134b1: its decision opens with the base pack's percents, a row per stack.
            assert_eq!(fort.gold, [0, 25, 50, 75, 100, 100, 100], "{index}");
            assert_eq!(state.hill_fort(red).map(|visit| visit.hero), Some(hero));
            let seen = h3_game::observe(&state, rules, h3_game::Viewer::Player(red)).decision;
            let Some(h3_game::DecisionView::HillFort(view)) = seen else {
                panic!("{index}: {seen:?}")
            };
            let army = state.adventure().heroes()[0].army.len();
            assert_eq!(view.rows.len(), army, "{index}");
            continue;
        }
        // E08 T134b2: the factory asks the hire question naming the base pack's machines; its
        // yes opens the hire dialog of them, each 1 while the hero lacks its artifact, at the
        // creatures' costs (the Ballista 2 500, the First Aid Tent 750, the Ammo Cart 1 000 gold).
        let Some(ObjectKind::WarMachineFactory(factory)) = kind else {
            panic!("{index}: {kind:?}")
        };
        factories += 1;
        let names: Vec<_> = factory
            .machines
            .iter()
            .map(|&(machine, _)| rules.core().creature_key(machine).unwrap().as_str())
            .collect();
        assert_eq!(
            names,
            ["base:ballista", "base:first_aid_tent", "base:ammo_cart"]
        );
        let answer = GameCommand::Answer(h3_game::Answer::Offer(h3_game::Consent::Accept));
        apply(
            &mut state,
            rules,
            Actor::Player(red),
            answer,
            &mut Vec::new(),
        )
        .unwrap();
        let seen = h3_game::observe(&state, rules, h3_game::Viewer::Player(red)).decision;
        let Some(h3_game::DecisionView::Hire(view)) = seen else {
            panic!("{index}: {seen:?}")
        };
        let details = &state.heroes()[usize::from(hero.raw())];
        let held = |artifact| {
            details
                .worn
                .iter()
                .flatten()
                .any(|i| i.artifact == artifact)
        };
        let expected: Vec<_> = factory
            .machines
            .iter()
            .zip([2500, 750, 1000])
            .map(|(&(machine, artifact), gold)| {
                (
                    machine,
                    u32::from(!held(artifact)),
                    vec![(gold_id(rules), gold)],
                )
            })
            .collect();
        let shown: Vec<_> = view
            .creatures
            .into_iter()
            .map(|c| (c.creature, c.count, c.cost))
            .collect();
        assert_eq!(shown, expected, "{index}");
    }
    // The Black Market, the two Trading Posts' classes and the guild's pictures; the Altar's; the
    // Hill Fort's; the factory's.
    assert!(markets >= 4, "{markets}");
    assert!(altars >= 1, "{altars}");
    assert!(forts >= 1, "{forts}");
    assert_eq!(factories, 1);
}

/// The core's gold.
fn gold_id(rules: &h3_game::GameRuleset) -> h3_content::ResourceId {
    let resources = rules.core().resources();
    let at = resources.iter().position(|r| r.key.as_str() == "base:gold");
    h3_content::ResourceId::from_index(at.unwrap()).unwrap()
}

/// The entrance of `object`, its one visitable tile.
fn entrance_of(object: &ObjectSetup) -> Pos {
    let (column, row) = object.visitable.offsets().next().unwrap();
    Pos::new(
        object.anchor.x - u16::from(column),
        object.anchor.y - u16::from(row),
        0,
    )
}

/// Red's first hero's one step onto `to`: the command's result.
fn step(
    state: &mut GameState,
    rules: &h3_game::GameRuleset,
    to: Pos,
) -> Result<(), GameCommandError> {
    let hero = state.adventure().heroes()[0].id;
    let path = vec![to];
    apply(
        state,
        rules,
        Actor::Player(PlayerId::new(0)),
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }),
        &mut Vec::new(),
    )
}

// covers: ADV-HFT-001, ADV-HFT-002
#[test]
fn a_hill_fort_upgrades_the_elementals_only_with_the_ab_pack() {
    // Decision 378 (the enabled packs decide): red's hero with the four elementals steps into a
    // Hill Fort. Under RoE's rules alone (the base pack) none of its rows has an upgrade; under
    // SoD's (the ab pack's `upgrades` override) each its own: the Storm, Magma, Energy and Ice
    // Elementals.
    let elementals = ["air", "earth", "fire", "water"];
    let upgrades = ["storm", "magma", "energy", "ice"];
    for (format, rules, keep) in [
        (w::Format::RoE, &*ROE, in_roe as fn(usize) -> bool),
        (w::Format::Sod, &*SOD, |_| true),
    ] {
        let ab = matches!(format, w::Format::Sod);
        let creature = |k: String| rules.core().creature_by_key(&k).unwrap();
        let raw = read_back(&w::write_in(&trades_map(keep), format).unwrap());
        let mut setup = import(&raw, rules).unwrap();
        let fort = setup.map.objects.iter().find(|object| {
            let ty = object
                .object_type
                .and_then(|ty| rules.adventure().object_type(ty));
            matches!(ty.map(|ty| &ty.kind), Some(ObjectKind::HillFort(_)))
        });
        let entrance = entrance_of(fort.unwrap());
        setup.heroes[0].pos = Pos::new(entrance.x, entrance.y + 1, 0);
        let army = elementals.iter().zip(0..);
        let army = army.map(|(e, slot)| ArmyStack {
            slot,
            creature: creature(format!("base:{e}_elemental")),
            count: 1,
        });
        setup.heroes[0].army = HeroArmy::Custom(army.collect());
        let mut state = GameState::new(rules, &setup, 1).unwrap();
        step(&mut state, rules, entrance).unwrap();
        let red = h3_game::Viewer::Player(PlayerId::new(0));
        let seen = h3_game::observe(&state, rules, red).decision;
        let Some(h3_game::DecisionView::HillFort(view)) = seen else {
            panic!("{format:?}: {seen:?}")
        };
        let shown: Vec<_> = view.rows.iter().map(|row| row.row.upgrade).collect();
        let expected: Vec<_> = upgrades
            .iter()
            .map(|u| ab.then(|| creature(format!("ab:{u}_elemental"))))
            .collect();
        assert_eq!(shown, expected, "{format:?}");
    }
}

// covers: ADV-MKT-008
#[test]
fn another_player_sees_each_trade_by_its_type_and_nothing_of_the_visit() {
    // Blue's hero beside red's below each entrance: once red stepped in, blue's observation shows
    // the object by its type at its anchor (the same to every viewer) and no decision of its own;
    // red's window (and the factory's question) are red's alone.
    let rules = &*SOD;
    let raw = read_back(&w::write(&trades_map(|_| true)).unwrap());
    let setup = import(&raw, rules).unwrap();
    let blue = PlayerId::new(1);
    for (index, object) in setup.map.objects.iter().enumerate() {
        let entrance = entrance_of(object);
        let mut setup = setup.clone();
        setup.heroes[0].pos = Pos::new(entrance.x, entrance.y + 1, 0);
        let mut theirs = setup.heroes[0].clone();
        theirs.owner = blue;
        theirs.hero_type = None;
        theirs.pos = Pos::new(entrance.x + 1, entrance.y + 1, 0);
        setup.heroes.push(theirs);
        let mut state = GameState::new(rules, &setup, 1).unwrap();
        step(&mut state, rules, entrance).unwrap();
        let red = h3_game::Viewer::Player(PlayerId::new(0));
        assert!(
            h3_game::observe(&state, rules, red).decision.is_some(),
            "{index}"
        );
        let seen = h3_game::observe(&state, rules, h3_game::Viewer::Player(blue));
        let shown = seen
            .adventure
            .objects
            .iter()
            .find(|o| o.anchor == object.anchor);
        assert_eq!(shown.map(|o| o.object_type), object.object_type, "{index}");
        assert_eq!(seen.decision, None, "{index}");
        let open = (state.market(blue), state.altar(blue), state.hill_fort(blue));
        assert_eq!(open, (None, None, None), "{index}");
    }
}

// covers: ADV-OBJ-004
#[test]
fn no_trade_or_service_is_entered_from_above() {
    // None of the seven classes is visited from above: red's hero on a free tile of the three
    // above an entrance does not step down onto it (`FromAbove`), the state unchanged.
    let rules = &*SOD;
    let raw = read_back(&w::write(&trades_map(|_| true)).unwrap());
    let setup = import(&raw, rules).unwrap();
    let blocked: Vec<Pos> = setup
        .map
        .objects
        .iter()
        .flat_map(|object| {
            object.blocked.offsets().map(|(column, row)| {
                let (x, y) = (object.anchor.x, object.anchor.y);
                Pos::new(x - u16::from(column), y - u16::from(row), 0)
            })
        })
        .collect();
    let mut tried = 0;
    for (index, object) in setup.map.objects.iter().enumerate() {
        let ty = rules.adventure().object_type(object.object_type.unwrap());
        assert!(!ty.unwrap().from_above, "{index}");
        let entrance = entrance_of(object);
        for x in entrance.x - 1..=entrance.x + 1 {
            let from = Pos::new(x, entrance.y - 1, 0);
            if blocked.contains(&from) {
                continue;
            }
            tried += 1;
            let mut setup = setup.clone();
            setup.heroes[0].pos = from;
            let mut state = GameState::new(rules, &setup, 1).unwrap();
            let before = state.clone();
            let refused = AdventureCommandError::FromAbove { step: 0 };
            assert_eq!(
                step(&mut state, rules, entrance),
                Err(GameCommandError::Adventure(refused)),
                "{index}: {from:?}"
            );
            assert_eq!(state, before, "{index}: {from:?}");
        }
    }
    // The three tiles above each of the nine entrances are free.
    assert_eq!(tried, 3 * PICTURES.len());
}
