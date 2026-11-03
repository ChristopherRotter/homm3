//! The economy map committed under `fixtures/maps/` (E08 T86a, increment 9's design note §15):
//! two players; the seven mines, two guarded by a monster beside their entrance and the others
//! not, blue's Crystal Cavern owned at load; an abandoned mine; two Lighthouses on the coast of a
//! sea; a Shipyard with its boat tile; two of each weekly object; growing and not growing
//! monsters; timed events once, repeating and taking; heroes with Estates, Mysticism, resource
//! specialists, the income and mana artifacts and Crystal Dragons. Written by [`crate::writer`]
//! as the sea map (`sea_map.rs`); the golden replays play it (random play over 60 days and the
//! scripted ones, `crates/h3-cli/tests/adventure_replays.rs`). The test checks that the committed
//! file reads to the map written here and imports with every class, its owners, its set, its
//! events and its Shipyard's boat tile; it rewrites the file when `H3_WRITE_FIXTURES` is set (a
//! change invalidates the replays that name the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::play::FAITHFUL;
use super::sea_map::{hero, plain, template};
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{Holds, Pos};
use h3_contracts::PlayerId;
use h3_game::GameState;

/// The committed file's name.
const NAME: &str = "e08-economy-36.h3m";

/// The original's templates (`Objects.txt`): one line per object picture the map places, the
/// grass pictures where a class has several. A mine, the Shipyard, a Windmill and a Mystical
/// Garden are visited one column left of their position, a Lighthouse and a Water Wheel on it.
pub(super) mod lines {
    pub(crate) const MINES: [&str; 7] = [
        "AVMsawg0.def 000011110011111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000010100 53 0 0 0",
        "AVMalch0.def 000111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000000111 53 1 0 0",
        "AVMore0.def 000111110001111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000000100 53 2 0 1",
        "AVMsulf0.def 000111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000000111 53 3 0 0",
        "AVMcrys0.def 000111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000000100 53 4 0 0",
        "AVMgems0.def 000111110001111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000010100 53 5 0 1",
        "AVMgold0.def 000111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 111111111 000000100 53 6 0 0",
    ];
    pub(crate) const ABANDONED: &str = "AVXabnd0.def \
        000111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 000000001 53 7 0 0";
    pub(crate) const LIGHTHOUSE: &str = "AVXlths0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 010111111 42 0 0 0";
    pub(crate) const SHIPYARD: &str = "AVXshyd0.def \
        000111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 011111111 011111111 87 0 0 0";
    pub(crate) const WINDMILL: &str = "AVMwndd0.def \
        101111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 112 0 0 0";
    pub(crate) const WATER_WHEEL: &str = "AVMwwhl0.def \
        000111110001111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000010101 109 0 0 0";
    pub(crate) const GARDEN: &str = "AVTmyst0.def \
        000111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 000010101 55 0 0 0";
    pub(crate) const PIKEMEN: &str = "AvWPike.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000100 54 0 2 0";
    pub(crate) const GNOLLS: &str = "AVWgnll0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 54 98 2 0";
}

/// Artifact codes (`ARTRAITS.TXT` order): the Charm and the Mystic Orb of Mana, the Endless Sack
/// and the Endless Purse of Gold.
const CHARM_OF_MANA: u16 = 73;
const MYSTIC_ORB: u16 = 75;
const ENDLESS_SACK: u16 = 115;
const ENDLESS_PURSE: u16 = 117;
/// The first misc slot (`ArtSlots.txt` order).
const MISC: usize = 9;
/// Creature codes: Pikemen, Archangels, Crystal Dragons.
const PIKEMAN: u16 = 0;
const ARCHANGEL: u16 = 13;
const CRYSTAL_DRAGON: u16 = 133;
/// Resource bits of an abandoned mine's set (bit 0 wood … bit 6 gold).
const CRYSTAL_OR_GOLD: u8 = 0b101_0000;

/// The heroes' tiles, by hero id (the map's order): red's Lord Haart, Rissa and Orrin, then
/// blue's Sephinroth and Caitlin.
pub(super) const HEROES: [(u8, u8); 5] = [(3, 3), (3, 9), (26, 23), (3, 26), (12, 32)];
/// The mines' positions by subclass (sawmill … gold mine); blue owns the Crystal Cavern (4).
pub(super) const MINES: [(u8, u8); 7] = [
    (9, 16),
    (15, 5),
    (15, 11),
    (15, 27),
    (9, 22),
    (21, 9),
    (21, 22),
];
/// The abandoned mine (crystal or gold).
pub(super) const ABANDONED: (u8, u8) = (21, 16);
/// The Lighthouses, on the sea's coast.
pub(super) const LIGHTHOUSES: [(u8, u8); 2] = [(29, 4), (29, 10)];
/// The Shipyard and its boat tile on the sea (the second of its offsets, two columns right of
/// its entrance: ADV-SHY-001).
pub(super) const SHIPYARD: (u8, u8) = (29, 20);
pub(super) const BOAT_TILE: (u8, u8) = (30, 20);
/// The weekly objects: two Windmills, two Water Wheels, two Mystical Gardens.
pub(super) const WINDMILLS: [(u8, u8); 2] = [(9, 6), (21, 28)];
pub(super) const WATER_WHEELS: [(u8, u8); 2] = [(15, 15), (25, 32)];
pub(super) const GARDENS: [(u8, u8); 2] = [(25, 13), (15, 32)];

/// A savage monster that never flees, growing each week unless `fixed`.
fn monster(line: &str, at: (u8, u8), count: u16, fixed: bool) -> w::MapObject {
    w::MapObject::Monster(w::Monster {
        tile: at,
        template: template(line),
        count,
        disposition: 4,
        never_flees: true,
        fixed_count: fixed,
        treasure: None,
    })
}

pub(super) fn owned(line: &str, at: (u8, u8), body: w::OwnedBody) -> w::MapObject {
    w::MapObject::Owned(w::OwnedObject {
        tile: at,
        template: template(line),
        body,
    })
}

/// A hero's worn artifact on the first misc slot.
fn wearing(hero: &mut w::Hero, artifact: u16) {
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[MISC] = Some(artifact);
    hero.artifacts = Some(w::HeroArtifacts {
        worn,
        backpack: Vec::new(),
    });
}

/// The timed events: on day 1 red's human player gains 500 gold (once); from day 2 every seven
/// days blue gains two crystals (repeating); from day 3 every fourteen days both lose 1 000 gold
/// and five wood (taking, floored at 0).
fn events() -> Vec<w::TimedEvent> {
    vec![
        w::TimedEvent {
            name: "Once".into(),
            message: "A gift.".into(),
            resources: [0, 0, 0, 0, 0, 0, 500],
            players: 0b01,
            human: true,
            computer: false,
            first_day: 0,
            interval: 0,
        },
        w::TimedEvent {
            name: "Weekly".into(),
            message: "Crystals.".into(),
            resources: [0, 0, 0, 0, 2, 0, 0],
            players: 0b10,
            human: true,
            computer: true,
            first_day: 1,
            interval: 7,
        },
        w::TimedEvent {
            name: "Tax".into(),
            message: "Taxes.".into(),
            resources: [-5, 0, 0, 0, 0, 0, -1_000],
            players: 0b11,
            human: true,
            computer: true,
            first_day: 2,
            interval: 14,
        },
    ]
}

/// The map: SoD, 36 × 36, grass with the sea on the east (columns 30 to 35), no underground; red
/// in the north, blue in the south.
///
/// - Red's Lord Haart (hero 0, an Estates specialist with basic Estates) on (3, 3) with the
///   Endless Sack of Gold, two Crystal Dragons, ten archangels and twenty pikemen.
/// - Red's Rissa (hero 1, a mercury specialist with basic Mysticism and a Spell Book) on (3, 9)
///   with the Charm of Mana, her type's army.
/// - Red's Orrin (hero 2) on (26, 23), below the Shipyard.
/// - Blue's Sephinroth (hero 3, a crystal specialist) on (3, 26) with the Mystic Orb of Mana and
///   Caitlin (hero 4, a gold specialist) on (12, 32) with the Endless Purse of Gold, their types'
///   armies.
/// - The mines ([`MINES`]): the sawmill on (9, 16) between both sides, neutral and unguarded;
///   the Alchemist's Lab on (15, 5), its entrance guarded by twelve gnolls on (14, 7); the Ore Pit
///   on (15, 11); the Sulfur Dune on (15, 27), its entrance guarded by eight pikemen on (14, 29)
///   that never grow; blue's Crystal Cavern on (9, 22); the Gem Pond on (21, 9); the gold mine on
///   (21, 22). The abandoned mine on (21, 16), crystal or gold.
/// - The Lighthouses on (29, 4) and (29, 10), the Shipyard on (29, 20) (its boat tile (30, 20)).
/// - The weekly objects: Windmills on (9, 6) and (21, 28), Water Wheels on (15, 15) and (25, 32),
///   Mystical Gardens on (25, 13) and (15, 32).
/// - Wandering monsters: ten gnolls on (12, 19) and six on (5, 14) (growing), eight pikemen on
///   (25, 17) (not growing).
/// - The timed events ([`events`]).
fn map() -> w::Map {
    use w::OwnedBody::{Owner, Resources};
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let [haart, rissa, orrin, sephinroth, caitlin] = HEROES;
    let mut haart = hero(haart, 0, 4);
    wearing(&mut haart, ENDLESS_SACK);
    let mut army = [None; w::ARMY_SLOTS];
    army[0] = Some((CRYSTAL_DRAGON, 2));
    army[1] = Some((ARCHANGEL, 10));
    army[2] = Some((PIKEMAN, 20));
    haart.army = Some(army);
    let mut rissa = hero(rissa, 0, 38);
    wearing(&mut rissa, CHARM_OF_MANA);
    let orrin = hero(orrin, 0, 0);
    let mut sephinroth = hero(sephinroth, 1, 94);
    wearing(&mut sephinroth, MYSTIC_ORB);
    let mut caitlin = hero(caitlin, 1, 15);
    wearing(&mut caitlin, ENDLESS_PURSE);
    let mut objects: Vec<w::MapObject> = [haart, rissa, orrin, sephinroth, caitlin]
        .into_iter()
        .map(w::MapObject::Hero)
        .collect();
    for (subclass, (line, at)) in lines::MINES.into_iter().zip(MINES).enumerate() {
        let owner = (subclass == 4).then_some(1);
        objects.push(owned(line, at, Owner(owner)));
    }
    objects.push(owned(
        lines::ABANDONED,
        ABANDONED,
        Resources(CRYSTAL_OR_GOLD),
    ));
    for at in LIGHTHOUSES {
        objects.push(owned(lines::LIGHTHOUSE, at, Owner(None)));
    }
    objects.push(owned(lines::SHIPYARD, SHIPYARD, Owner(None)));
    for (line, tiles) in [
        (lines::WINDMILL, WINDMILLS),
        (lines::WATER_WHEEL, WATER_WHEELS),
        (lines::GARDEN, GARDENS),
    ] {
        objects.extend(tiles.map(|at| plain(line, at)));
    }
    objects.extend([
        monster(lines::GNOLLS, (14, 7), 12, false),
        monster(lines::PIKEMEN, (14, 29), 8, true),
        monster(lines::GNOLLS, (12, 19), 10, false),
        monster(lines::GNOLLS, (5, 14), 6, false),
        monster(lines::PIKEMEN, (25, 17), 8, true),
    ]);
    w::Map {
        name: "Economy".into(),
        description: "E08 T86a: mines, the week and owned objects".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (30, 0),
            to: (35, 35),
        }],
        roads: Vec::new(),
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: events(),
        teams: None,
    }
}

/// Increment 9's object types on the map, in its order after the heroes.
const CLASSES: [&str; 18] = [
    "base:sawmill",
    "base:alchemists_lab",
    "base:ore_pit",
    "base:sulfur_dune",
    "base:crystal_cavern",
    "base:gem_pond",
    "base:gold_mine",
    "base:abandoned_mine",
    "base:lighthouse",
    "base:lighthouse",
    "base:shipyard",
    "base:windmill",
    "base:windmill",
    "base:water_wheel",
    "base:water_wheel",
    "base:mystical_garden",
    "base:mystical_garden",
    "base:monster",
];

fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

// covers: ADV-OWN-002, ADV-SHY-001
#[test]
fn the_economy_map_is_committed_as_written() {
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
    let setup = import(&committed, &FAITHFUL).unwrap();
    setup.validate(&FAITHFUL).unwrap();
    let heroes: Vec<(Pos, usize)> = setup
        .heroes
        .iter()
        .map(|h| (h.pos, h.owner.index()))
        .collect();
    assert_eq!(
        heroes,
        HEROES
            .iter()
            .zip([0, 0, 0, 1, 1])
            .map(|(&tile, owner)| (at(tile), owner))
            .collect::<Vec<_>>()
    );
    let adventure = FAITHFUL.adventure();
    let types: Vec<_> = setup.map.objects.iter().map(|o| o.object_type).collect();
    let mut expected: Vec<_> = CLASSES
        .iter()
        .map(|key| Some(adventure.object_type_by_key(key).unwrap()))
        .collect();
    expected.extend([expected[17]; 4]);
    assert_eq!(types, expected);
    // Blue's Crystal Cavern the one owner at load; the abandoned mine's set.
    let owners: Vec<_> = setup.map.objects.iter().map(|o| o.owner).collect();
    let mut expected = vec![None; owners.len()];
    expected[4] = Some(PlayerId::new(1));
    assert_eq!(owners, expected);
    assert!(matches!(&setup.map.objects[7].holds, Holds::Resources(set) if set.len() == 2));
    assert_eq!(setup.events.len(), 3);
    // The Shipyard's boat tile on the sea, the abandoned mine's draw within its set.
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let objects = state.adventure().map().objects();
    assert_eq!(objects[10].resolved.boat_tile(), Some(at(BOAT_TILE)));
    assert!(objects[7].resolved.mine().is_some());
}
