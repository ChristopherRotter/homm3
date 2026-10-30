//! The sea map committed under `fixtures/maps/` (E08 T75a, design §15): a strait between two
//! shores with boats of the three kinds beside both coasts, the sea's four pickups, events on
//! water with and without guards, a monster on water, Favorable Winds, a hole and a guarded tile
//! on a coast, and heroes with each of the sea's sources (Navigation and its specialists, the
//! Necklace of Ocean Guidance, the Sea Captain's Hat, the Admiral's Hat) beside a boat, a flyer
//! beside a coastal Flotsam and a water walker. Written by [`crate::writer`] as the rewards map
//! (`rewards_map.rs`); the scripted golden replays play its paths
//! (`crates/h3-cli/tests/adventure_replays.rs`). The test checks that the committed file reads to
//! the map written here and imports with every sea class, and the flyer's refusal beside the
//! Flotsam (D-46), which a replay cannot record; it rewrites the file when `H3_WRITE_FIXTURES` is
//! set (a change invalidates the replays that name the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::play::FAITHFUL;
use crate::model::Rewards;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureCommand, BoatSetup, Pos};
use h3_contracts::{HeroId, PlayerId};
use h3_game::{Actor, GameCommand, GameState, NoopSink, apply};

/// The committed file's name.
const NAME: &str = "e08-sea-36.h3m";

/// The original's templates (`Objects.txt`, as `sea.rs`' and `rewards_map.rs`'): one line per
/// object picture the map places. A boat is visited (and stands) one column left of its position;
/// every other object here on its position.
mod lines {
    pub(super) const BOATS: [&str; 3] = [
        "AVXboat0.def 101111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 100000000 100000000 8 0 0 0",
        "AVXboat1.def 101111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 100000000 100000000 8 1 0 0",
        "AVXboat2.def 101111111111111111111111111111111111111111111111 \
         010000000000000000000000000000000000000000000000 100000000 100000000 8 2 0 0",
    ];
    pub(super) const FLOTSAM: &str = "AVAflot0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 100000000 100000000 29 0 0 0";
    pub(super) const SEA_CHEST: &str = "AVXccht0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 100000000 100000000 82 0 0 0";
    pub(super) const SURVIVOR: &str = "AVAsurv0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 100000000 100000000 86 0 0 0";
    pub(super) const BOTTLE: &str = "AVXbttl0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 100000000 100000000 59 0 0 0";
    pub(super) const HOLE: &str = "AVLholg0.def \
        111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 000000100 124 0 0 1";
    pub(super) const WINDS: &str = "AVXfw5.def \
        111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 100000000 100000000 225 0 0 1";
    pub(super) const EVENT: &str = "AVZevnt0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 111111111 26 0 0 0";
    pub(super) const PIKEMEN: &str = "AvWPike.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000100 54 0 2 0";
    pub(super) const GNOLLS: &str = "AVWgnll0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 54 98 2 0";
}

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

/// An object standing on its position `at`.
fn plain(line: &str, at: (u8, u8)) -> w::MapObject {
    w::MapObject::Plain(w::Plain {
        tile: at,
        template: template(line),
    })
}

/// An empty boat of `kind` on the water tile `at` (its position one column right).
fn boat(kind: usize, at: (u8, u8)) -> w::MapObject {
    plain(lines::BOATS[kind], (at.0 + 1, at.1))
}

/// A hero of `hero_type` (its class's picture) on `at`, its type's own but for what the map sets.
fn hero(at: (u8, u8), owner: u8, hero_type: u8) -> w::Hero {
    w::Hero {
        tile: at,
        owner,
        hero_type,
        class: hero_type / 8,
        name: None,
        experience: None,
        skills: None,
        army: None,
        tight_formation: false,
        artifacts: None,
        spells: None,
        primary: None,
    }
}

/// `count` creatures of `creature` in the first slot.
fn first_slot(creature: u16, count: u16) -> [Option<(u16, u16)>; w::ARMY_SLOTS] {
    let mut army = [None; w::ARMY_SLOTS];
    army[0] = Some((creature, count));
    army
}

/// A hero wearing `artifact` at `slot` (`ArtSlots.txt` order) and carrying `backpack`.
fn wearing(hero: &mut w::Hero, slot: usize, artifact: u16, backpack: Vec<u16>) {
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[slot] = Some(artifact);
    hero.artifacts = Some(w::HeroArtifacts { worn, backpack });
}

/// A savage monster that never flees: it fights whoever enters a tile it guards.
fn monster(line: &str, at: (u8, u8), count: u16) -> w::MapObject {
    w::MapObject::Monster(w::Monster {
        tile: at,
        template: template(line),
        count,
        disposition: 4,
        never_flees: true,
        fixed_count: true,
        treasure: None,
    })
}

/// An event for every colour, removed once it fired.
fn event(
    at: (u8, u8),
    guards: Option<[Option<(u16, u16)>; w::ARMY_SLOTS]>,
    rewards: Rewards,
) -> w::MapObject {
    w::MapObject::Reward(w::RewardObject {
        tile: at,
        template: template(lines::EVENT),
        body: w::RewardBody::Event(w::Event {
            message: guards.map(|guards| w::Message {
                text: String::new(),
                guards: Some(guards),
            }),
            rewards,
            players: 0xff,
            computer: true,
            removed_after_visit: true,
        }),
    })
}

/// An Ocean Bottle holding `text`.
fn bottle(at: (u8, u8), text: &str) -> w::MapObject {
    w::MapObject::Reward(w::RewardObject {
        tile: at,
        template: template(lines::BOTTLE),
        body: w::RewardBody::Sign(text.into()),
    })
}

/// Skill codes (`SSTRAITS.TXT` order).
const NAVIGATION: u8 = 5;
/// Artifact codes (`ARTRAITS.TXT` order): Centaur's Axe (the full backpack's), the Necklace of
/// Ocean Guidance, Angel Wings, Boots of Levitation, the Sea Captain's Hat, the Admiral's Hat.
const AXE: u16 = 7;
const NECKLACE: u16 = 71;
const WINGS: u16 = 72;
const LEVITATION: u16 = 90;
const CAPTAINS_HAT: u16 = 123;
const ADMIRALS_HAT: u16 = 136;
/// Artifact slots (`ArtSlots.txt` order).
const HEAD: usize = 0;
const SHOULDERS: usize = 1;
const NECK: usize = 2;
const FEET: usize = 8;
/// A full backpack (`ADV-QUIRK-22`: 64 refuse an artifact under `faithful`).
const FULL_BACKPACK: usize = 64;
/// Creature codes: Pikemen, Angels, Archangels.
const PIKEMAN: u16 = 0;
const ANGEL: u16 = 12;
const ARCHANGEL: u16 = 13;

/// The heroes' tiles, by hero id (the map's order): red's Sylvia, Orrin, Mephala and Tyris, then
/// blue's Voy, Fiona and Astral.
pub(super) const HEROES: [(u8, u8); 7] = [
    (9, 4),
    (9, 10),
    (10, 16),
    (9, 27),
    (27, 22),
    (27, 28),
    (10, 31),
];
/// The empty boats' water tiles and kinds, in the map's order.
pub(super) const BOATS: [((u8, u8), u8); 6] = [
    ((12, 4), 0),
    ((12, 10), 1),
    ((23, 6), 1),
    ((23, 22), 2),
    ((23, 28), 0),
    ((12, 31), 2),
];
/// The Flotsam beside Mephala's coast.
const COASTAL_FLOTSAM: (u8, u8) = (12, 16);

/// The map: SoD, 36 × 36, grass on the west (columns 0 to 11) and the east (24 to 35) and the sea
/// between (12 to 23), no underground; red on the west, blue on the east (but Astral).
///
/// - Red's Sylvia (hero 0, a Navigation specialist with basic Navigation) on (9, 4), the Admiral's
///   Hat on her head (free boarding) and twenty archangels; her boat on (12, 4). On the sea before
///   her: ten pikemen on (16, 4) (they guard the water around them), an event on (14, 7) (500
///   gold) and one on (17, 7) whose five pikemen fight first (five crystals).
/// - Red's Orrin (hero 1), his type's own, on (9, 10); his boat on (12, 10), a straight crossing
///   east to the free coast at (24, 10).
/// - Red's Mephala (hero 2) on (10, 16) with Angel Wings: a Flotsam on (12, 16) beside her coast
///   (no flight takes or passes it, D-46).
/// - Red's Tyris (hero 3) on (9, 27) with Boots of Levitation and five angels.
/// - Blue's Voy (hero 4, a Navigation specialist with basic Navigation) on (27, 22) with the
///   Necklace of Ocean Guidance and a full backpack (64 Centaur's Axes); his boat on (23, 22), a
///   Sea Chest on (21, 21) and the Shipwreck Survivor on (21, 23) before it.
/// - Blue's Fiona (hero 5) on (27, 28) with expert Navigation and the Sea Captain's Hat; her boat
///   on (23, 28).
/// - Blue's Astral (hero 6) on (10, 31), on red's shore, with one pikeman; his boat on (12, 31).
/// - The rest: an empty boat on (23, 6); Favorable Winds on (18, 12) to (20, 14); a Flotsam on
///   (18, 25), a Sea Chest on (16, 13), a bottle with a message on (19, 20) and an empty one on
///   (15, 28); a hole on the west coast at (11, 20); two gnolls on (25, 16) guarding the east
///   coast's (24, 16).
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let [sylvia, orrin, mephala, tyris, voy, fiona, astral] = HEROES;
    let mut sylvia = hero(sylvia, 0, 3);
    wearing(&mut sylvia, HEAD, ADMIRALS_HAT, Vec::new());
    sylvia.army = Some(first_slot(ARCHANGEL, 20));
    let orrin = hero(orrin, 0, 0);
    let mut mephala = hero(mephala, 0, 16);
    wearing(&mut mephala, SHOULDERS, WINGS, Vec::new());
    let mut tyris = hero(tyris, 0, 7);
    wearing(&mut tyris, FEET, LEVITATION, Vec::new());
    tyris.army = Some(first_slot(ANGEL, 5));
    let mut voy = hero(voy, 1, 122);
    wearing(&mut voy, NECK, NECKLACE, vec![AXE; FULL_BACKPACK]);
    let mut fiona = hero(fiona, 1, 48);
    wearing(&mut fiona, HEAD, CAPTAINS_HAT, Vec::new());
    fiona.skills = Some(vec![(NAVIGATION, 3)]);
    let mut astral = hero(astral, 1, 40);
    astral.army = Some(first_slot(PIKEMAN, 1));
    let mut gold = Rewards::default();
    gold.resources[6] = 500;
    let mut crystals = Rewards::default();
    crystals.resources[4] = 5;
    let mut objects: Vec<w::MapObject> = [sylvia, orrin, mephala, tyris, voy, fiona, astral]
        .into_iter()
        .map(w::MapObject::Hero)
        .collect();
    objects.extend(BOATS.map(|(at, kind)| boat(usize::from(kind), at)));
    objects.extend([
        w::MapObject::Ground(w::Ground {
            tile: (20, 14),
            template: template(lines::WINDS),
        }),
        monster(lines::PIKEMEN, (16, 4), 10),
        event((14, 7), None, gold),
        event((17, 7), Some(first_slot(PIKEMAN, 5)), crystals),
        plain(lines::FLOTSAM, COASTAL_FLOTSAM),
        plain(lines::SEA_CHEST, (21, 21)),
        plain(lines::SURVIVOR, (21, 23)),
        plain(lines::FLOTSAM, (18, 25)),
        plain(lines::SEA_CHEST, (16, 13)),
        bottle((19, 20), "Fair winds"),
        bottle((15, 28), ""),
        plain(lines::HOLE, (11, 20)),
        monster(lines::GNOLLS, (25, 16), 2),
    ]);
    w::Map {
        name: "Sea".into(),
        description: "E08 T75a: boats and the sea".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![w::Patch {
            terrain: w::Terrain::Water,
            from: (12, 0),
            to: (23, 35),
        }],
        roads: Vec::new(),
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
    }
}

/// Increment 8's object types on the map (the boats are the map's boats, no objects).
const CLASSES: [&str; 8] = [
    "base:flotsam",
    "base:sea_chest",
    "base:shipwreck_survivor",
    "base:ocean_bottle",
    "base:hole",
    "sod:favorable_winds",
    "base:event",
    "base:monster",
];

fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

// covers: ADV-SEA-001, ADV-PIK-018
#[test]
fn the_sea_map_is_committed_as_written() {
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
    let heroes: Vec<(Pos, usize)> = setup
        .heroes
        .iter()
        .map(|h| (h.pos, h.owner.index()))
        .collect();
    assert_eq!(
        heroes,
        HEROES
            .iter()
            .zip([0, 0, 0, 0, 1, 1, 1])
            .map(|(&tile, owner)| (at(tile), owner))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        setup.map.boats,
        BOATS.map(|(tile, kind)| BoatSetup {
            tile: at(tile),
            kind
        })
    );
    let adventure = FAITHFUL.adventure();
    for key in CLASSES {
        let kind = adventure.object_type_by_key(key).unwrap();
        assert!(
            setup
                .map
                .objects
                .iter()
                .any(|o| o.object_type == Some(kind)),
            "{key}"
        );
    }
    // The flyer beside the coastal Flotsam (ADV-PIK-018; D-46, batch E8c): a flight neither ends
    // on the Flotsam's water tile nor passes it, while one over the plain water beside it lands.
    let mut state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let red = Actor::Player(PlayerId::new(0));
    let mephala = HeroId::new(2);
    let flight = |path: &[(u8, u8)]| {
        GameCommand::Adventure(AdventureCommand::MoveHero {
            hero: mephala,
            path: path.iter().copied().map(at).collect(),
        })
    };
    let (x, y) = COASTAL_FLOTSAM;
    for path in [
        vec![(x - 1, y), (x, y)],
        vec![(x - 1, y), (x, y), (x - 1, y + 1)],
        vec![(x - 1, y - 1), (x, y), (x - 1, y + 1)],
    ] {
        let before = state.state_hash();
        assert!(
            apply(&mut state, &FAITHFUL, red, flight(&path), &mut NoopSink).is_err(),
            "{path:?}"
        );
        assert_eq!(state.state_hash(), before, "{path:?}");
    }
    let reach = h3_game::legal_moves(&state, &FAITHFUL, red, mephala).unwrap();
    assert!(reach.path(at(COASTAL_FLOTSAM)).is_none());
    assert!(
        reach
            .tiles()
            .filter_map(|(pos, _)| reach.path(pos))
            .all(|path| !path.contains(&at(COASTAL_FLOTSAM)))
    );
    let over = flight(&[(x - 1, y - 1), (x, y - 1), (x - 1, y - 2)]);
    apply(&mut state, &FAITHFUL, red, over, &mut NoopSink).unwrap();
}
