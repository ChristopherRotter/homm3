//! The trades map committed under `fixtures/maps/` (E08 T135b1, increment 14's design note §18):
//! SoD, 36 × 36 of grass, dirt on columns 0 to 11, rough on columns 24 to 35 of rows 0 to 8, snow
//! on columns 24 to 35 of rows 21 to 35. Every class of increment 14 twice, seven in the north and
//! seven in the south: two Trading Posts of class 99 (the dirt and grass picture on dirt, the rough
//! one on rough) and two of class 221 on snow, two Freelancer's Guilds, two Altars of Sacrifice,
//! two Black Markets, two Hill Forts (the dirt picture on dirt, the grass one on grass) and two War
//! Machine Factories. The map bans batch E14-3's artifacts (every treasure, minor and major below
//! 127 but seventeen), so the Black Markets draw their fourteen from those seventeen. Red's heroes
//! each below an entrance for a scripted replay: one at a Trading Post, one at a guild, a good one
//! with 900 experience and artifacts at an Altar, one at a Black Market beside a hero who can
//! receive no artifact, one at a Hill Fort with five stacks of levels 1 to 6, one at a factory;
//! blue's evil and neutral heroes below the other Altar, one below a snow Trading Post and one in
//! the south's middle. Written by [`crate::writer`] with the original's templates; the golden
//! replays play it (`crates/h3-cli/tests/adventure_replays.rs`). The tests check that the committed
//! file reads to the map written here (it is rewritten when `H3_WRITE_FIXTURES` is set), imports
//! with each object typed at its anchor, builds with each Black Market's seven drawn outside the
//! bans and distinct, and refuses the full hero's purchase changing nothing.

use std::path::PathBuf;

use super::garrisons_bonuses::army;
use super::import::SOD;
use super::sea_map::{hero, plain};
use super::trades::PICTURES;
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureCommand, Pos};
use h3_contracts::PlayerId;
use h3_game::{
    Actor, Answer, GameCommand, GameCommandError, GameState, MarketAnswer, MarketError, apply,
};

/// The committed file's name.
const NAME: &str = "e08-trades-36.h3m";

/// [`PICTURES`]' lines by object.
const ALTAR: usize = 0;
const BLACK_MARKET: usize = 1;
const HILL_FORT_DIRT: usize = 2;
const HILL_FORT_GRASS: usize = 3;
const POST: usize = 4;
const POST_ROUGH: usize = 5;
const POST_SNOW: usize = 6;
const FACTORY: usize = 7;
const GUILD: usize = 8;

/// The heroes' tiles, by hero id (the map's order), each below an entrance: red's Orrin (0) at
/// the north's Trading Post, Valeska (1) at its guild, Edric (2) at its Altar, Sylvia (3) at its
/// Black Market and Christian (4) east of her, Sorsha (5) at its Hill Fort, Tyris (6) at its
/// factory; blue's Straker (7) at the south's Altar and Yog (8) east of him, Gurnisson (9) at the
/// first snow Trading Post, Adela (10) in the south's middle.
pub(super) const HEROES: [(u8, u8); 11] = [
    (4, 5),
    (10, 5),
    (16, 5),
    (22, 5),
    (23, 5),
    (4, 11),
    (16, 11),
    (16, 25),
    (17, 25),
    (30, 25),
    (19, 28),
];
/// The heroes' owners and types, by hero id: Knights of Castle for red (good), a Death Knight of
/// Necropolis (evil), two Barbarians of Stronghold (neutral) and a Cleric of Castle for blue.
const HERO_TYPES: [(u8, u8); 11] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 6),
    (0, 5),
    (0, 7),
    (1, 64),
    (1, 96),
    (1, 97),
    (1, 9),
];

/// The objects in the map's order after the heroes: the anchor and the line of [`PICTURES`]. The
/// guild and the factory are entered west of the anchor, the others on it.
pub(super) const OBJECTS: [((u8, u8), usize); 14] = [
    // The north, row 4: the Trading Post on dirt, the guild, the Altar, the Black Market, the
    // rough Trading Post; row 10: the Hill Fort on dirt, the factory.
    ((4, 4), POST),
    ((11, 4), GUILD),
    ((16, 4), ALTAR),
    ((22, 4), BLACK_MARKET),
    ((30, 4), POST_ROUGH),
    ((4, 10), HILL_FORT_DIRT),
    ((17, 10), FACTORY),
    // The south, row 24: the Altar, the factory, a snow Trading Post; row 30: the guild, the Hill
    // Fort on grass, the Black Market, a snow Trading Post.
    ((16, 24), ALTAR),
    ((23, 24), FACTORY),
    ((30, 24), POST_SNOW),
    ((11, 30), GUILD),
    ((16, 30), HILL_FORT_GRASS),
    ((22, 30), BLACK_MARKET),
    ((30, 30), POST_SNOW),
];

/// The objects' types by key, in the map's order.
const TYPES: [&str; 14] = [
    "base:trading_post",
    "ab:freelancers_guild",
    "base:altar_of_sacrifice",
    "base:black_market",
    "base:trading_post",
    "base:hill_fort",
    "base:war_machine_factory",
    "base:altar_of_sacrifice",
    "base:war_machine_factory",
    "base:trading_post",
    "ab:freelancers_guild",
    "base:hill_fort",
    "base:black_market",
    "base:trading_post",
];

/// The artifacts the bans leave in the Black Markets' pool (batch E14-3's, by code in
/// `ARTRAITS.TXT` order): seven treasures (Centaur's Axe, the Shield of the Dwarven Lords, the
/// Helm of the Alabaster Unicorn, the Breastplate of Petrified Wood, the Spyglass, the Bird of
/// Perception, the Stoic Watchman), seven minors (the Blackshard of the Dead Knight, the Greater
/// Gnoll's Flail, the Shield of the Yawning Dead, the Buckler of the Gnoll King, the Crown of the
/// Supreme Magi, the Rib Cage, the Red Dragon Flame Tongue) and three majors (the Sword of
/// Hellfire, the Shield of the Damned, the Hellstorm Helmet).
const TREASURES: [u16; 7] = [7, 13, 19, 25, 53, 63, 64];
const MINORS: [u16; 7] = [8, 9, 14, 15, 22, 26, 38];
const MAJORS: [u16; 3] = [11, 17, 23];
/// The first code past the original's artifacts that a Black Market may draw: the editor bans
/// 127 to 143 by default, so the bans name every treasure, minor and major below it.
const DRAWN_BELOW: u16 = 127;

/// Creature codes (`CRTRAITS.TXT` order): Pikemen, Archers, Griffins, Swordsmen, Cavaliers;
/// Skeletons, Walking Dead, Wights.
const PIKEMAN: u16 = 0;
const ARCHER: u16 = 2;
const GRIFFIN: u16 = 4;
const SWORDSMAN: u16 = 6;
const CAVALIER: u16 = 10;
const SKELETON: u16 = 56;
const WALKING_DEAD: u16 = 58;
const WIGHT: u16 = 60;

/// Artifact codes (`ARTRAITS.TXT` order): the Spell Book; the treasures Skull Helmet and Badge
/// of Courage, the minor Helm of Chaos (all banned); the relics Titan's Gladius, Sentinel's
/// Shield, Thunder Helmet, Titan's Cuirass, the four tomes, the Orb of Vulnerability; the banned
/// Clover of Fortune.
const SPELL_BOOK: u16 = 0;
const SKULL_HELMET: u16 = 20;
const BADGE_OF_COURAGE: u16 = 49;
const HELM_OF_CHAOS: u16 = 21;
const TITANS_GLADIUS: u16 = 12;
const SENTINELS_SHIELD: u16 = 18;
const THUNDER_HELMET: u16 = 24;
const TITANS_CUIRASS: u16 = 30;
const TOMES: [u16; 4] = [86, 87, 88, 89];
const ORB_OF_VULNERABILITY: u16 = 93;
const CLOVER_OF_FORTUNE: u16 = 46;
/// Worn positions (`ArtSlots.txt` order): the head, the right and left hands, the torso, misc 1
/// to 4, the spell book, misc 5.
const HEAD: usize = 0;
const RIGHT_HAND: usize = 3;
const LEFT_HAND: usize = 4;
const TORSO: usize = 5;
const MISC: [usize; 5] = [9, 10, 11, 12, 18];
const BOOK: usize = 17;
/// A full backpack (`ADV-QUIRK-22`: 64 refuse an artifact under `faithful`).
const FULL_BACKPACK: usize = 64;

/// The full hero's artifacts: every position any of the seventeen takes (the hands, the head,
/// the torso, the five misc) worn with a relic, and 64 Clovers of Fortune in the backpack.
fn full() -> w::HeroArtifacts {
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[RIGHT_HAND] = Some(TITANS_GLADIUS);
    worn[LEFT_HAND] = Some(SENTINELS_SHIELD);
    worn[HEAD] = Some(THUNDER_HELMET);
    worn[TORSO] = Some(TITANS_CUIRASS);
    for (&position, artifact) in MISC
        .iter()
        .zip(TOMES.into_iter().chain([ORB_OF_VULNERABILITY]))
    {
        worn[position] = Some(artifact);
    }
    w::HeroArtifacts {
        worn,
        backpack: vec![CLOVER_OF_FORTUNE; FULL_BACKPACK],
    }
}

/// The map's heroes, by hero id: Edric (2) with 900 experience (a treasure's 1 000 gives level
/// 2), the Helm of Chaos and a Spell Book worn and a Skull Helmet in his backpack; Christian (4)
/// with no room for an artifact ([`full`]); Sorsha (5) with twenty Pikemen, ten Archers, five
/// Griffins, four Swordsmen and two Cavaliers; Straker (7) with twenty Skeletons, ten Walking Dead
/// and five Wights; Yog (8) with a Badge of Courage in his backpack; the others their types' own.
fn heroes() -> Vec<w::MapObject> {
    HEROES
        .iter()
        .zip(HERO_TYPES)
        .enumerate()
        .map(|(id, (&tile, (owner, hero_type)))| {
            let mut h = hero(tile, owner, hero_type);
            match id {
                2 => {
                    h.experience = Some(900);
                    let mut worn = [None; w::ARTIFACT_SLOTS];
                    worn[HEAD] = Some(HELM_OF_CHAOS);
                    worn[BOOK] = Some(SPELL_BOOK);
                    h.artifacts = Some(w::HeroArtifacts {
                        worn,
                        backpack: vec![SKULL_HELMET],
                    });
                }
                4 => h.artifacts = Some(full()),
                5 => {
                    h.army = Some(army(&[
                        (PIKEMAN, 20),
                        (ARCHER, 10),
                        (GRIFFIN, 5),
                        (SWORDSMAN, 4),
                        (CAVALIER, 2),
                    ]));
                }
                7 => h.army = Some(army(&[(SKELETON, 20), (WALKING_DEAD, 10), (WIGHT, 5)])),
                8 => {
                    h.artifacts = Some(w::HeroArtifacts {
                        worn: [None; w::ARTIFACT_SLOTS],
                        backpack: vec![BADGE_OF_COURAGE],
                    });
                }
                _ => {}
            }
            w::MapObject::Hero(h)
        })
        .collect()
}

/// Batch E14-3's bans: every treasure, minor and major code below [`DRAWN_BELOW`] but the
/// seventeen (the editor's default bans 127 to 143 the writer's own).
pub(super) fn banned(rules: &h3_game::GameRuleset) -> Vec<u16> {
    let kept: Vec<u16> = [&TREASURES[..], &MINORS, &MAJORS].concat();
    let core = rules.core();
    (0..DRAWN_BELOW)
        .filter(|code| !kept.contains(code))
        .filter(|&code| {
            let id = h3_content::ArtifactTypeId::from_index(usize::from(code)).unwrap();
            let class = core.artifact(id).unwrap().class;
            matches!(
                class,
                h3_content::ArtifactClass::Treasure
                    | h3_content::ArtifactClass::Minor
                    | h3_content::ArtifactClass::Major
            )
        })
        .collect()
}

pub(super) fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut all = heroes();
    all.extend(OBJECTS.iter().map(|&(at, line)| plain(PICTURES[line], at)));
    let patch = |terrain, from, to| w::Patch { terrain, from, to };
    w::Map {
        name: "Trades".into(),
        description: "E08 T135b1: the trades and services".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![
            patch(w::Terrain::Dirt, (0, 0), (11, 35)),
            patch(w::Terrain::Rough, (24, 0), (35, 8)),
            patch(w::Terrain::Snow, (24, 21), (35, 35)),
        ],
        roads: Vec::new(),
        players,
        objects: all,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: banned(&SOD),
        banned_spells: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

/// The committed file, checked to read to the map written here (rewritten first when
/// `H3_WRITE_FIXTURES` is set).
fn committed() -> RawMap {
    let bytes = w::write(&map()).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/maps")
        .join(NAME);
    if std::env::var_os("H3_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, w::gzip(&bytes)).unwrap();
    }
    let file = std::fs::read(&path).unwrap();
    let written = read(&w::gzip(&bytes), &Limits::default()).unwrap();
    let committed = read(&file, &Limits::default()).unwrap();
    assert!(
        committed == written,
        "{NAME} is not the map written here: H3_WRITE_FIXTURES=1 rewrites it"
    );
    committed
}

fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

/// The Black Markets' indices among the map's objects.
fn black_markets() -> impl Iterator<Item = usize> {
    (0..OBJECTS.len()).filter(|&i| OBJECTS[i].1 == BLACK_MARKET)
}

// covers: ADV-MKT-001, ADV-MKT-006, GAM-SET-003
#[test]
fn the_trades_map_is_committed_as_written_and_imports_the_fourteen() {
    let rules = &*SOD;
    let setup = import(&committed(), rules).unwrap();
    let heroes: Vec<(Pos, usize)> = setup
        .heroes
        .iter()
        .map(|h| (h.pos, h.owner.index()))
        .collect();
    let expected: Vec<(Pos, usize)> = HEROES
        .iter()
        .zip(HERO_TYPES)
        .map(|(&tile, (owner, _))| (at(tile), usize::from(owner)))
        .collect();
    assert_eq!(heroes, expected);
    let part = rules.adventure();
    let placed: Vec<(&str, Pos)> = (setup.map.objects.iter())
        .map(|o| {
            let key = part
                .object_type(o.object_type.unwrap())
                .unwrap()
                .key
                .as_str();
            (key, o.anchor)
        })
        .collect();
    let expected: Vec<(&str, Pos)> = TYPES
        .into_iter()
        .zip(OBJECTS.iter().map(|&(tile, _)| at(tile)))
        .collect();
    assert_eq!(placed, expected);
    // Each object one entrance on its anchor's row: on the anchor, the guild's and the factory's
    // west of it (each hero stands on the tile below one).
    for (index, object) in setup.map.objects.iter().enumerate() {
        let (column, row) = object.visitable.offsets().next().unwrap();
        assert_eq!(
            object.visitable.offsets().count(),
            1,
            "{index}: one entrance"
        );
        let west = u8::from(matches!(OBJECTS[index].1, GUILD | FACTORY));
        assert_eq!((column, row), (west, 0), "{index}");
    }
    // Each Black Market draws three treasures, three minors and one major outside the bans, the
    // two markets' fourteen distinct (ADV-MKT-006; batch E14-3), at every seed.
    let code = |id: h3_content::ArtifactTypeId| id.raw();
    for seed in 1..=16 {
        let game = GameState::new(rules, &setup, seed).unwrap();
        let mut all = Vec::new();
        for index in black_markets() {
            let stock: Vec<u16> = game.adventure().map().objects()[index]
                .resolved
                .stock()
                .iter()
                .map(|&id| code(id))
                .collect();
            assert_eq!(stock.len(), 7, "{seed}: {index}");
            let of = |class: &[u16]| stock.iter().filter(|a| class.contains(a)).count();
            assert_eq!(
                (of(&TREASURES), of(&MINORS), of(&MAJORS)),
                (3, 3, 1),
                "{seed}: {index}: {stock:?}"
            );
            all.extend(stock);
        }
        let mut distinct = all.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), all.len(), "{seed}: {all:?}");
    }
}

// covers: ADV-MKT-007
#[test]
fn the_full_hero_s_purchase_on_the_trades_map_is_refused_changing_nothing() {
    // Christian (hero 4) onto the north's Black Market: its window opens; each purchase is
    // refused before paying (A-46 fixed, ADV-QUIRK-48: `MarketError::NoRoom`), the state's hash
    // unchanged. A replay holds no refused command (the scripted `trades-black-market-s1` holds
    // his visit and its `Leave`), so the refusal is this test's.
    let rules = &*SOD;
    let setup = import(&committed(), rules).unwrap();
    let mut game = GameState::new(rules, &setup, 1).unwrap();
    let red = PlayerId::new(0);
    let christian = game.adventure().heroes()[4].id;
    let (anchor, _) = OBJECTS[black_markets().next().unwrap()];
    let path = vec![at(anchor)];
    let command = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: christian,
        path,
    });
    apply(
        &mut game,
        rules,
        Actor::Player(red),
        command,
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(game.market(red).map(|visit| visit.hero), Some(christian));
    let gold = rules
        .core()
        .resources()
        .iter()
        .position(|r| r.key.as_str() == "base:gold")
        .and_then(h3_content::ResourceId::from_index)
        .unwrap();
    let before = game.state_hash();
    for entry in 0..7 {
        let answer = GameCommand::Answer(Answer::Market(MarketAnswer::Buy { entry, pay: gold }));
        let refused = apply(
            &mut game,
            rules,
            Actor::Player(red),
            answer,
            &mut Vec::new(),
        );
        assert_eq!(
            refused,
            Err(GameCommandError::Market(MarketError::NoRoom)),
            "{entry}"
        );
        assert_eq!(game.state_hash(), before, "{entry}");
    }
}
