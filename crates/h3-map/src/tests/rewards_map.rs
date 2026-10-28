//! The rewards map committed under `fixtures/maps/` (E08 T64a, design §16): every object class
//! of increment 7 on one map, written by [`crate::writer`] (not laid out from a plan as the
//! other fixtures, `fixtures.rs`), and beside red's three heroes the order-dependent paths the
//! scripted golden replays play (`crates/h3-cli/tests/adventure_replays.rs`). The test checks
//! that the committed file reads to the map written here and imports with every class; it
//! rewrites the file when `H3_WRITE_FIXTURES` is set (a change invalidates the replays that name
//! the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::play::FAITHFUL;
use crate::model::{Rewards, Stack};
use crate::writer as w;
use crate::{Limits, import, read};

/// The committed file's name.
const NAME: &str = "e08-rewards-36.h3m";

/// The original's templates (`Objects.txt`; their masks give the objects their shapes and
/// entrances, survey `E08-rewards.md` §1): one line per object picture the map places.
mod lines {
    pub(super) const ARENA: &str = "AVSarna0.def \
        000111110001111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 4 0 0 0";
    pub(super) const AXE: &str = "AVA0007.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 5 7 4 0";
    pub(super) const BLACKSHARD: &str = "AVA0008.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 5 8 4 0";
    pub(super) const PANDORA: &str = "AVA0128.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 6 0 4 0";
    pub(super) const CORPSE: &str = "AVXskds0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000100010 22 0 0 0";
    pub(super) const MARLETTO: &str = "AvSMarl.def \
        001111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 23 0 0 0";
    pub(super) const EVENT: &str = "AVZevnt0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 111111111 26 0 0 0";
    pub(super) const GARDEN: &str = "AVSgrdn0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 001010101 32 0 0 0";
    pub(super) const LEAN_TO: &str = "AVMlean0.def \
        001111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000001000 39 0 0 0";
    pub(super) const LIBRARY: &str = "AVSlibr0.def \
        000011110011111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 41 0 0 0";
    pub(super) const SCHOOL_OF_MAGIC: &str = "AVSschm0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 47 0 0 0";
    pub(super) const MERCENARY_CAMP: &str = "AVSmerc0.def \
        000111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 51 0 0 0";
    pub(super) const GNOLLS: &str = "AVWgnll0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 54 98 2 0";
    pub(super) const STAR_AXIS: &str = "AVSaxis0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 61 0 0 0";
    pub(super) const PYRAMID: &str = "AVXprmd0.def \
        001111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000010 63 0 0 0";
    pub(super) const RANDOM_ARTIFACTS: [&str; 5] = [
        "AVArand.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 000000001 65 0 4 0",
        "AVArnd1.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 000000001 66 0 4 0",
        "AVArnd2.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 000000001 67 0 4 0",
        "AVArnd3.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 000000001 68 0 4 0",
        "AVArnd4.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 000000001 69 0 4 0",
    ];
    pub(super) const SCHOLAR: &str = "AVXschl0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 81 0 0 0";
    pub(super) const SHRINES: [&str; 3] = [
        "AVXl1sh0.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 011111111 88 0 0 0",
        "AVXl2sh0.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 011111111 89 0 0 0",
        "AVXl3sh0.def \
            011111111111111111111111111111111111111111111111 \
            100000000000000000000000000000000000000000000000 111111111 011111111 90 0 0 0",
    ];
    pub(super) const SCROLL: &str = "AVA0001.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 93 0 4 0";
    pub(super) const LEARNING_STONE: &str = "AVSgzbo0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 100 0 0 0";
    pub(super) const CHEST: &str = "AVTchst0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 101 0 5 0";
    pub(super) const TREE_OF_KNOWLEDGE: &str = "AVXtrek0.def \
        101111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 000000101 102 0 0 0";
    pub(super) const UNIVERSITY: &str = "AVSuniv0.def \
        000011110000111110001111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 104 0 0 0";
    pub(super) const WAGON: &str = "AVTwagn0.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000100000 105 0 0 0";
    pub(super) const SCHOOL_OF_WAR: &str = "AVSwar20.def \
        100111110000111110011111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 107 0 0 0";
    pub(super) const TOMB: &str = "AVXtomb0.def \
        001111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 011111111 108 0 0 0";
    pub(super) const WITCH_HUT: &str = "AVSwtch0.def \
        101111111111111111111111111111111111111111111111 \
        010000000000000000000000000000000000000000000000 111111111 011111111 113 0 0 0";
}

fn template(line: &str) -> w::ObjectTemplate {
    w::ObjectTemplate::from_objects_txt(line).unwrap()
}

/// An object's anchor (the bottom-right tile of its picture) from its entrance: `entrance`
/// columns right of and rows below the entrance as the template's visitable tile lies left of
/// and above the anchor.
fn anchor(line: &str, entrance: (u8, u8)) -> (u8, u8) {
    let visit = line.split_whitespace().nth(2).unwrap();
    let at = visit.find('1').unwrap();
    let (column, row) = (u8::try_from(at % 8).unwrap(), u8::try_from(at / 8).unwrap());
    (entrance.0 + column, entrance.1 + row)
}

fn plain(line: &str, entrance: (u8, u8)) -> w::MapObject {
    w::MapObject::Plain(w::Plain {
        tile: anchor(line, entrance),
        template: template(line),
    })
}

fn reward(line: &str, entrance: (u8, u8), body: w::RewardBody) -> w::MapObject {
    w::MapObject::Reward(w::RewardObject {
        tile: anchor(line, entrance),
        template: template(line),
        body,
    })
}

/// An object the writer gives the editor's body (E24 M1 T3a): the Arena, the Schools, a Scholar
/// (a bonus drawn at a new game), …
fn any(line: &str, entrance: (u8, u8)) -> w::AnyObject {
    w::AnyObject {
        level: 0,
        tile: anchor(line, entrance),
        template: template(line),
        owner: None,
    }
}

/// A hero of `hero_type` (its class's picture) standing on `at`, its type's own but for what the
/// map sets after.
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

/// A savage stack of gnolls that never flees: it fights whoever enters a tile it guards.
fn guard(at: (u8, u8), count: u16) -> w::MapObject {
    w::MapObject::Monster(w::Monster {
        tile: at,
        template: template(lines::GNOLLS),
        count,
        disposition: 4,
        never_flees: true,
        fixed_count: true,
        treasure: None,
    })
}

/// Guards by slot: `count` creatures of `creature` in the first slot.
fn first_slot(creature: u16, count: u16) -> [Option<(u16, u16)>; w::ARMY_SLOTS] {
    let mut army = [None; w::ARMY_SLOTS];
    army[0] = Some((creature, count));
    army
}

/// The warrior's (red's first hero's) full army: seven kinds, none a reward's creature
/// (archangels, pikemen, archers, griffins, swordsmen, monks, cavaliers).
const WARRIOR_ARMY: [(u16, u16); 7] = [(13, 20), (0, 10), (2, 10), (4, 5), (6, 5), (8, 4), (10, 3)];
/// The mage's custom experience: 50 below level 2 (1 000), so that its guards' battle (ten
/// pikemen, 100 hit points) brings a level-up.
const MAGE_EXPERIENCE: u32 = 950;
/// Wisdom's skill code (HER-SPL-005).
const WISDOM: u8 = 7;
/// Bless (level 1) and Prayer (level 4): the guarded box's spells (HER-SPL-005: a level-4 spell
/// needs advanced Wisdom).
const BOX_SPELLS: [u8; 2] = [41, 48];
/// Resources by `RESTYPES.TXT` index.
const CRYSTAL: usize = 4;
const GOLD: usize = 6;

/// The map: SoD, 36 × 36 of grass, one level; red's three heroes and blue's two, and every
/// object class of increment 7 (survey `E08-rewards.md` §1; `codes.rs`' 31 classes).
///
/// - Red's warrior, Orrin, stands on (6, 4) with a full army (`WARRIOR_ARMY`): east of it
///   Pandora's Box on (9, 4) gives ten wood elves and 1 000 experience (no room: the army dialog,
///   then the experience); south of it an event on (6, 8) that the gnolls on (7, 9) guard (the
///   guard's battle, then the event), and the Learning Stone on (3, 8) that the gnolls on (2, 9)
///   guard, entered from (4, 8) (the stone's level-up, then the guard's battle).
/// - Red's mage, Adela, stands on (28, 4) with basic Wisdom alone, a Spell Book and no spell,
///   twenty archangels and `MAGE_EXPERIENCE`: Pandora's Box on (31, 4), guarded by ten pikemen,
///   gives Bless and Prayer (the battle's level-up offers advanced Wisdom first, then the spells).
/// - Red's walker, Mephala, stands on (5, 28) with its type's army: an event for red on (8, 28)
///   stops a move east, and the University's entrance is (13, 26).
/// - Blue's two heroes stand on (32, 30) and (32, 18), their types' own.
/// - The rest in rows: the big gifts (Library, School of War, Arena, Mercenary Camp) entered on
///   row 11, the two-tile ones (Marletto Tower, Pyramid, Lean-To, Warrior's Tomb) on row 16, the
///   one-tile gifts and the Witch Hut on row 21, Shrines, a School of Magic and a Wagon on row
///   13, chests, an artifact and three random ones on row 18, the other two random artifacts,
///   a scroll, a Scholar and a Corpse on row 23, a chest, a guarded artifact, a box of every
///   reward part and a Learning Stone on row 7; gnolls with an artifact on (20, 27), an event for
///   blue with guards on (24, 27), an event for everyone that stays on (16, 29).
#[allow(clippy::too_many_lines)]
fn map() -> (w::Map, w::Extras) {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    for player in &mut players[..2] {
        *player = Some(w::Player {
            human: true,
            computer: true,
            towns: 1,
        });
    }
    let mut warrior = hero((6, 4), 0, 0);
    let mut army = [None; w::ARMY_SLOTS];
    for (slot, stack) in army.iter_mut().zip(WARRIOR_ARMY) {
        *slot = Some(stack);
    }
    warrior.army = Some(army);
    warrior.experience = Some(0);
    let mut mage = hero((28, 4), 0, 9);
    mage.experience = Some(MAGE_EXPERIENCE);
    mage.skills = Some(vec![(WISDOM, 1)]);
    mage.army = Some(first_slot(13, 20));
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[17] = Some(0);
    mage.artifacts = Some(w::HeroArtifacts {
        worn,
        backpack: Vec::new(),
    });
    mage.spells = Some([0; 9]);
    let walker = hero((5, 28), 0, 16);
    let event = |players: u8, removed: bool, message: Option<w::Message>, rewards: Rewards| {
        w::RewardBody::Event(w::Event {
            message,
            rewards,
            players,
            computer: true,
            removed_after_visit: removed,
        })
    };
    let mut resources = [0; 7];
    resources[GOLD] = 500;
    let gold = Rewards {
        resources,
        ..Rewards::default()
    };
    let mut resources = [0; 7];
    resources[CRYSTAL] = 5;
    let crystals = Rewards {
        resources,
        ..Rewards::default()
    };
    let objects = vec![
        w::MapObject::Hero(warrior),
        w::MapObject::Hero(mage),
        w::MapObject::Hero(walker),
        w::MapObject::Hero(hero((32, 30), 1, 25)),
        w::MapObject::Hero(hero((32, 18), 1, 34)),
        // The warrior's paths.
        reward(
            lines::PANDORA,
            (9, 4),
            w::RewardBody::Pandora {
                message: None,
                rewards: Rewards {
                    experience: 1000,
                    creatures: vec![Stack {
                        creature: 18,
                        count: 10,
                    }],
                    ..Rewards::default()
                },
            },
        ),
        reward(lines::EVENT, (6, 8), event(1, true, None, crystals)),
        guard((7, 9), 2),
        plain(lines::LEARNING_STONE, (3, 8)),
        guard((2, 9), 2),
        // The mage's.
        reward(
            lines::PANDORA,
            (31, 4),
            w::RewardBody::Pandora {
                message: Some(w::Message {
                    text: String::new(),
                    guards: Some(first_slot(0, 10)),
                }),
                rewards: Rewards {
                    spells: BOX_SPELLS.to_vec(),
                    ..Rewards::default()
                },
            },
        ),
        // The walker's.
        reward(lines::EVENT, (8, 28), event(1, true, None, gold)),
        plain(lines::UNIVERSITY, (13, 26)),
        // Row 7.
        plain(lines::CHEST, (12, 7)),
        reward(
            lines::BLACKSHARD,
            (16, 7),
            w::RewardBody::Artifact(Some(w::Message {
                text: "Fight for it?".into(),
                guards: Some(first_slot(98, 8)),
            })),
        ),
        reward(
            lines::PANDORA,
            (20, 7),
            w::RewardBody::Pandora {
                message: None,
                rewards: Rewards {
                    experience: 500,
                    mana: 10,
                    morale: 1,
                    luck: -1,
                    resources: [5, 0, 0, 0, 0, 0, 1000],
                    primary: [1, 0, 0, 1],
                    skills: vec![(1, 1)],
                    artifacts: vec![10],
                    spells: vec![15],
                    creatures: vec![Stack {
                        creature: 14,
                        count: 5,
                    }],
                },
            },
        ),
        plain(lines::LEARNING_STONE, (24, 7)),
        // Row 16: the two-tile Pyramid, Warrior's Tomb (Marletto Tower and the Lean-To below,
        // as `any`).
        plain(lines::PYRAMID, (17, 16)),
        plain(lines::TOMB, (27, 16)),
        // Row 13.
        reward(lines::SHRINES[0], (14, 13), w::RewardBody::Shrine(None)),
        reward(lines::SHRINES[1], (19, 13), w::RewardBody::Shrine(None)),
        reward(lines::SHRINES[2], (24, 13), w::RewardBody::Shrine(None)),
        plain(lines::WAGON, (29, 13)),
        // Row 18.
        plain(lines::CHEST, (10, 18)),
        reward(lines::AXE, (14, 18), w::RewardBody::Artifact(None)),
        reward(
            lines::RANDOM_ARTIFACTS[0],
            (19, 18),
            w::RewardBody::Artifact(None),
        ),
        reward(
            lines::RANDOM_ARTIFACTS[1],
            (24, 18),
            w::RewardBody::Artifact(None),
        ),
        reward(
            lines::RANDOM_ARTIFACTS[2],
            (29, 18),
            w::RewardBody::Artifact(None),
        ),
        // Row 21.
        reward(lines::WITCH_HUT, (11, 21), w::RewardBody::WitchHut(None)),
        // Row 23.
        reward(
            lines::RANDOM_ARTIFACTS[3],
            (17, 23),
            w::RewardBody::Artifact(None),
        ),
        reward(
            lines::RANDOM_ARTIFACTS[4],
            (20, 23),
            w::RewardBody::Artifact(None),
        ),
        reward(
            lines::SCROLL,
            (23, 23),
            w::RewardBody::Scroll {
                message: None,
                spell: 17,
            },
        ),
        plain(lines::CORPSE, (29, 23)),
        // The rest of the south.
        w::MapObject::Monster(w::Monster {
            tile: (20, 27),
            template: template(lines::GNOLLS),
            count: 6,
            disposition: 2,
            never_flees: false,
            fixed_count: true,
            treasure: Some(w::Treasure {
                text: String::new(),
                resources: [0, 0, 0, 0, 0, 0, 500],
                artifact: Some(9),
            }),
        }),
        reward(
            lines::EVENT,
            (24, 27),
            event(
                2,
                true,
                Some(w::Message {
                    text: String::new(),
                    guards: Some(first_slot(0, 5)),
                }),
                Rewards {
                    artifacts: vec![13],
                    ..Rewards::default()
                },
            ),
        ),
        reward(
            lines::EVENT,
            (16, 29),
            event(
                0xff,
                false,
                None,
                Rewards {
                    morale: 1,
                    luck: 1,
                    ..Rewards::default()
                },
            ),
        ),
    ];
    let extras = w::Extras {
        tiles: Vec::new(),
        objects: vec![
            // Row 11: the big gifts.
            any(lines::LIBRARY, (11, 11)),
            any(lines::SCHOOL_OF_WAR, (16, 11)),
            any(lines::ARENA, (21, 11)),
            any(lines::MERCENARY_CAMP, (26, 11)),
            // Row 16.
            any(lines::MARLETTO, (12, 16)),
            any(lines::LEAN_TO, (22, 16)),
            // Row 13.
            any(lines::SCHOOL_OF_MAGIC, (10, 13)),
            // Row 21.
            any(lines::TREE_OF_KNOWLEDGE, (16, 21)),
            any(lines::STAR_AXIS, (22, 21)),
            any(lines::GARDEN, (27, 21)),
            // Row 23.
            any(lines::SCHOLAR, (26, 23)),
        ],
    };
    let map = w::Map {
        name: "Rewards".into(),
        description: "E08 T64a: every reward class of increment 7".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
    };
    (map, extras)
}

/// Increment 7's object types (`codes.rs`' 31 classes), each on the map at least once.
const CLASSES: [&str; 31] = [
    "base:arena",
    "base:artifact",
    "base:pandoras_box",
    "base:corpse",
    "base:marletto_tower",
    "base:event",
    "base:garden_of_revelation",
    "base:lean_to",
    "base:library_of_enlightenment",
    "base:school_of_magic",
    "base:mercenary_camp",
    "base:star_axis",
    "base:pyramid",
    "base:random_artifact",
    "base:random_treasure_artifact",
    "base:random_minor_artifact",
    "base:random_major_artifact",
    "base:random_relic_artifact",
    "base:scholar",
    "base:shrine_of_magic_incantation",
    "base:shrine_of_magic_gesture",
    "base:shrine_of_magic_thought",
    "base:spell_scroll",
    "base:learning_stone",
    "base:treasure_chest",
    "base:tree_of_knowledge",
    "base:university",
    "base:wagon",
    "base:school_of_war",
    "base:warriors_tomb",
    "base:witch_hut",
];

#[test]
fn the_rewards_map_is_committed_as_written() {
    let (map, extras) = map();
    let bytes = w::write_with(&map, &extras).unwrap();
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
    let custom: Vec<_> = setup.heroes.iter().map(|h| h.details.experience).collect();
    assert_eq!(
        custom,
        [Some(0), Some(MAGE_EXPERIENCE), None, None, None],
        "red's three heroes, then blue's two"
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
}
