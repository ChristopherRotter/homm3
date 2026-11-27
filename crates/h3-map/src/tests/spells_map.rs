//! The spells map committed under `fixtures/maps/` (E08 T144b, increment 15's design note §17):
//! SoD, 36 × 36 of grass with a strait between two shores (columns 14 to 27; in the south, rows
//! 29 to 35, only columns 14 to 17, the east's grass coming west), an island in it (grass on
//! columns 17 to 19, rock on 20 and 21, rows 9 to 13), four empty boats, the special grounds that
//! change a cast (Lucid Pools, Magic Clouds, Magic Plains, Cursed Ground on a coast), two Prisons
//! and the Market of Time. Red's six heroes each carry a Spell Book with the five adventure
//! spells (Summon Boat, Scuttle Boat, Fly, Water Walk, Dimension Door) and the knowledge to cast
//! them, each placed for a scripted replay; blue's seven, without spells, stand below the Prisons,
//! the Market of Time and along the east shore, so that one freed prisoner makes blue's eight on
//! the map and the second Prison finds it full. Written by [`crate::writer`] with the original's
//! templates; the golden replays play it (`crates/h3-cli/tests/adventure_replays.rs`). The tests
//! check that the committed file reads to the map written here (it is rewritten when
//! `H3_WRITE_FIXTURES` is set), imports with its heroes, prisoners, grounds and boats, gives each
//! caster its ground's mastery, and refuses the casts a replay cannot record, changing nothing.

use std::path::PathBuf;

use super::garrisons_bonuses::army;
use super::import::SOD;
use super::play::{FAITHFUL, STRICT};
use super::sea_map::{boat, hero, template};
use crate::model::RawMap;
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureCommand, BoatSetup, Pos};
use h3_content::SpellId;
use h3_contracts::{HeroId, PlayerId};
use h3_game::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState, MapCastError,
    NoopSink, apply,
};

/// The committed file's name.
const NAME: &str = "e08-spells-36.h3m";

/// The original's templates (`Objects.txt`): the special grounds' 3 × 3 pictures (Lucid Pools,
/// Magic Clouds, SoD's Magic Plains and Cursed Ground), each covering its position and the two
/// columns left of it and the two rows above.
mod lines {
    pub(super) const LUCID_POOLS: &str = "AVXlp5.def \
        111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 011111111 228 0 0 1";
    pub(super) const MAGIC_CLOUDS: &str = "AVXmc5.def \
        111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 011111111 229 0 0 1";
    pub(super) const MAGIC_PLAINS: &str = "AVXmp5.def \
        111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 011111111 230 0 0 1";
    pub(super) const CURSED_GROUND: &str = "AVXcg5.def \
        111111111111111111111111111111111111111111111111 \
        000000000000000000000000000000000000000000000000 111111111 011111111 223 0 0 1";
}

/// The heroes' tiles, by hero id (the map's order): red's boatwright (0) on the Lucid Pools at
/// the west coast, the novice (1) on the coast, the flyer (2) on the Magic Clouds at the coast
/// before the island, the door's caster (3) inland, the water walker (4) on the Magic Plains at
/// the south's narrow strait, the watcher (5) on the east shore across from the flyer; blue's
/// seven: below the first Prison (6), the second (7) and the Market of Time (8), and four along
/// the east shore (9 to 12).
pub(super) const HEROES: [(u8, u8); 13] = [
    (13, 5),
    (13, 18),
    (13, 11),
    (8, 26),
    (13, 33),
    (30, 11),
    (32, 21),
    (32, 25),
    (32, 29),
    (33, 3),
    (34, 7),
    (34, 15),
    (34, 33),
];
/// The heroes' owners and types, by hero id: Knights of Castle for red, Rangers of Rampart for
/// blue.
const HERO_TYPES: [(u8, u8); 13] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 4),
    (0, 5),
    (0, 6),
    (1, 16),
    (1, 17),
    (1, 18),
    (1, 19),
    (1, 20),
    (1, 21),
    (1, 22),
];
/// The empty boats' water tiles and kinds, in the map's order: two near the boatwright, three
/// columns east and three rows north of him and five columns east (the nearer by |dx| + |dy|
/// the second, by dx² + dy² the first: `ADV-QUIRK-53`), one past his sight to the east (in his
/// casts' window), one in the far south-east.
pub(super) const BOATS: [((u8, u8), u8); 4] =
    [((16, 2), 0), ((18, 5), 1), ((22, 4), 2), ((25, 26), 0)];
/// The Prisons' entrances (each blocking the tile west of it), the Market of Time's entrance.
pub(super) const PRISONS: [(u8, u8); 2] = [(32, 20), (32, 24)];
pub(super) const MARKET_OF_TIME: (u8, u8) = (32, 28);
/// The special grounds' positions (each covering the two columns left and the two rows above):
/// the Lucid Pools under the boatwright, the Magic Clouds under the flyer, the Magic Plains under
/// the water walker, Cursed Ground on the coast south of the novice.
pub(super) const LUCID_POOLS: (u8, u8) = (13, 6);
pub(super) const MAGIC_CLOUDS: (u8, u8) = (13, 12);
pub(super) const MAGIC_PLAINS: (u8, u8) = (13, 34);
pub(super) const CURSED_GROUND: (u8, u8) = (13, 22);

/// Creature codes (`CRTRAITS.TXT` order): Pikemen, Angels.
const PIKEMAN: u16 = 0;
const ANGEL: u16 = 12;
/// Skill codes (`SSTRAITS.TXT` order): Air Magic.
const AIR_MAGIC: u8 = 15;
/// The Spell Book's code and its worn position (`ArtSlots.txt` order).
const SPELL_BOOK: u16 = 0;
const BOOK: usize = 17;
/// The five adventure spells' bits (spell codes 0, 1, 6, 7, 8): Summon Boat, Scuttle Boat, Fly,
/// Water Walk, Dimension Door.
const FIVE_SPELLS: [u8; 9] = [0b1100_0011, 0b0000_0001, 0, 0, 0, 0, 0, 0, 0];
/// The casters' primary skills: knowledge 8 (80 spell points).
const CASTER: [u8; 4] = [1, 1, 1, 8];

/// A red caster: five angels (the fastest army, so the day's points carry a long flight), the
/// book and the five spells, [`CASTER`]'s primary skills.
fn caster(at: (u8, u8), hero_type: u8) -> w::Hero {
    let mut h = hero(at, 0, hero_type);
    h.army = Some(army(&[(ANGEL, 5)]));
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[BOOK] = Some(SPELL_BOOK);
    h.artifacts = Some(w::HeroArtifacts {
        worn,
        backpack: Vec::new(),
    });
    h.spells = Some(FIVE_SPELLS);
    h.primary = Some(CASTER);
    h
}

/// The map's heroes, by hero id: red's six casters (the door's with Advanced Air Magic), blue's
/// seven with ten pikemen each.
fn heroes() -> Vec<w::MapObject> {
    HEROES
        .iter()
        .zip(HERO_TYPES)
        .enumerate()
        .map(|(id, (&tile, (owner, hero_type)))| {
            let mut h = if owner == 0 {
                caster(tile, hero_type)
            } else {
                let mut h = hero(tile, owner, hero_type);
                h.army = Some(army(&[(PIKEMAN, 10)]));
                h
            };
            if id == 3 {
                h.skills = Some(vec![(AIR_MAGIC, 2)]);
            }
            w::MapObject::Hero(h)
        })
        .collect()
}

/// The prisoners: a Death Knight with 5 000 experience and an army of his own, then a Barbarian
/// as his type has him.
fn prisons() -> Vec<w::MapObject> {
    let mut first = hero(PRISONS[0], 0xFF, 64);
    first.name = Some("Freed".into());
    first.experience = Some(5000);
    first.army = Some(army(&[(PIKEMAN, 12)]));
    let second = hero(PRISONS[1], 0xFF, 96);
    [first, second]
        .into_iter()
        .zip(PRISONS)
        .map(|(prisoner, tile)| w::MapObject::Prison(w::Prison { tile, prisoner }))
        .collect()
}

fn ground(line: &str, tile: (u8, u8)) -> w::MapObject {
    w::MapObject::Ground(w::Ground {
        tile,
        template: template(line),
    })
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
    let mut objects = heroes();
    objects.extend(BOATS.map(|(at, kind)| boat(usize::from(kind), at)));
    objects.extend(prisons());
    objects.push(w::MapObject::Plain(w::Plain {
        tile: MARKET_OF_TIME,
        template: w::ObjectTemplate::market_of_time(),
    }));
    objects.extend([
        ground(lines::LUCID_POOLS, LUCID_POOLS),
        ground(lines::MAGIC_CLOUDS, MAGIC_CLOUDS),
        ground(lines::MAGIC_PLAINS, MAGIC_PLAINS),
        ground(lines::CURSED_GROUND, CURSED_GROUND),
    ]);
    let patch = |terrain, from, to| w::Patch { terrain, from, to };
    w::Map {
        name: "Spells".into(),
        description: "E08 T144b: the Prison, the Market of Time and the adventure spells".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![
            patch(w::Terrain::Water, (14, 0), (27, 35)),
            patch(w::Terrain::Grass, (18, 29), (27, 35)),
            patch(w::Terrain::Grass, (17, 9), (19, 13)),
            patch(w::Terrain::Rock, (20, 9), (21, 13)),
        ],
        roads: Vec::new(),
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

/// The committed file, checked to read to the map written here (rewritten first when
/// `H3_WRITE_FIXTURES` is set).
pub(super) fn committed() -> RawMap {
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

pub(super) fn at((x, y): (u8, u8)) -> Pos {
    Pos::new(x.into(), y.into(), 0)
}

// covers: ADV-PRI-001, ADV-OBJ-008
#[test]
fn the_spells_map_is_committed_as_written_and_imports() {
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
    assert_eq!(
        setup.map.boats,
        BOATS.map(|(tile, kind)| BoatSetup {
            tile: at(tile),
            kind
        })
    );
    assert_eq!(setup.prisoners.len(), 2);
    let part = rules.adventure();
    let keys: Vec<&str> = (setup.map.objects.iter())
        .map(|o| {
            part.object_type(o.object_type.unwrap())
                .unwrap()
                .key
                .as_str()
        })
        .collect();
    assert_eq!(
        keys,
        [
            "base:prison",
            "base:prison",
            "base:market_of_time",
            "sod:lucid_pools",
            "sod:magic_clouds",
            "sod:magic_plains",
            "sod:cursed_ground",
        ]
    );
}

/// The core spells' indices: Summon Boat, Scuttle Boat, Visions, Fly, Water Walk, Dimension Door.
const SUMMON_BOAT: u16 = 0;
const SCUTTLE_BOAT: u16 = 1;
const VISIONS: u16 = 2;
const FLY: u16 = 6;
const WATER_WALK: u16 = 7;
const DOOR: u16 = 8;

/// A game on the committed map under `rules`.
fn game(rules: &GameRuleset) -> GameState {
    GameState::new(rules, &import(&committed(), rules).unwrap(), 1).unwrap()
}

/// Red's cast of `spell` by `hero` at `target`: its events, or the refusal.
fn cast(
    state: &mut GameState,
    rules: &GameRuleset,
    hero: u16,
    spell: u16,
    target: Option<(u8, u8)>,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let command = GameCommand::Cast {
        hero: HeroId::new(hero),
        spell: SpellId::from_raw(spell),
        target: target.map(at),
    };
    let mut events = Vec::new();
    apply(state, rules, RED, command, &mut events).map(|()| events)
}

/// Red's cast refused with `error`, the state's hash unchanged.
fn refused(
    state: &mut GameState,
    rules: &GameRuleset,
    (hero, spell, target): (u16, u16, Option<(u8, u8)>),
    error: MapCastError,
) {
    let before = state.state_hash();
    assert_eq!(
        cast(state, rules, hero, spell, target),
        Err(GameCommandError::Cast(error)),
        "hero {hero}, spell {spell}, {target:?}"
    );
    assert_eq!(state.state_hash(), before, "hero {hero}, spell {spell}");
}

/// Red's move of `hero` to `to` by the reach's path.
fn walk(state: &mut GameState, rules: &GameRuleset, hero: u16, to: (u8, u8)) {
    let reach = h3_game::legal_moves(state, rules, RED, HeroId::new(hero)).unwrap();
    let path = reach.path(at(to)).unwrap();
    let command = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path,
    });
    apply(state, rules, RED, command, &mut NoopSink).unwrap();
}

const RED: Actor = Actor::Player(PlayerId::new(0));

// covers: HER-SPL-003, HER-SPL-008, GAM-SPL-001
#[test]
fn the_spells_map_s_grounds_give_their_casters_their_masteries() {
    // The Lucid Pools raise water, the Magic Clouds air, the Magic Plains every school to Expert;
    // the novice off every ground casts at None, the door's caster at its Advanced Air Magic. The
    // cost is the table's at the mastery (no army's ±2).
    let rules = &*FAITHFUL;
    let mut state = game(rules);
    for (hero, spell, target, mastery, cost) in [
        (0, SUMMON_BOAT, None, 3, 7),
        (2, FLY, None, 3, 15),
        (4, WATER_WALK, None, 3, 8),
        (1, SUMMON_BOAT, None, 0, 8),
        (3, DOOR, Some((4, 23)), 2, 20),
    ] {
        let events = cast(&mut state, rules, hero, spell, target).unwrap();
        let cast = events.iter().find_map(|e| match e {
            GameEvent::SpellCast { mastery, cost, .. } => Some((*mastery, *cost)),
            _ => None,
        });
        assert_eq!(cast, Some((mastery, cost)), "hero {hero}");
    }
}

// covers: ADV-SPL-001, ADV-SPL-003, ADV-SPL-004, ADV-SPL-005, ADV-SPL-006, ADV-SPL-007, GAM-SPL-001
#[test]
fn the_spells_map_refuses_the_casts_a_replay_cannot_hold_changing_nothing() {
    use MapCastError as M;
    use h3_adventure::CastError as C;
    for rules in [&*FAITHFUL, &*STRICT] {
        let strict = std::ptr::eq(rules, &*STRICT);
        let mut state = game(rules);
        // The game's tests: blue's hero, an unknown spell, a target where none is named or none
        // where one is.
        refused(
            &mut state,
            rules,
            (6, FLY, None),
            M::NotYourHero(HeroId::new(6)),
        );
        let visions = SpellId::from_raw(VISIONS);
        refused(&mut state, rules, (0, VISIONS, None), M::Unknown(visions));
        refused(
            &mut state,
            rules,
            (0, SUMMON_BOAT, Some((14, 4))),
            M::Target,
        );
        refused(&mut state, rules, (3, DOOR, None), M::Target);
        // The spells' own: Summon Boat inland; Scuttle Boat on water without a boat; the door out
        // of its window and onto a hero's tile, onto the explored water from land only with
        // `ADV-QUIRK-55` off (on: a charged failure, `spells-door-s1`).
        refused(
            &mut state,
            rules,
            (3, SUMMON_BOAT, None),
            M::Adventure(C::NoWater),
        );
        refused(
            &mut state,
            rules,
            (0, SCUTTLE_BOAT, Some((14, 5))),
            M::Adventure(C::NoBoat),
        );
        refused(
            &mut state,
            rules,
            (3, DOOR, Some((30, 26))),
            M::Adventure(C::OutOfWindow),
        );
        refused(
            &mut state,
            rules,
            (3, DOOR, Some((13, 18))),
            M::Adventure(C::NotFree),
        );
        if strict {
            refused(
                &mut state,
                rules,
                (3, DOOR, Some((14, 28))),
                M::Adventure(C::WrongKind),
            );
        }
        // Advanced: three doors a day, the fourth refused.
        for target in [(4, 23), (6, 23), (8, 23)] {
            cast(&mut state, rules, 3, DOOR, Some(target)).unwrap();
        }
        refused(
            &mut state,
            rules,
            (3, DOOR, Some((4, 23))),
            M::Adventure(C::NoCastsLeft),
        );
        // Fly and Water Walk again the same day: refused while set (`ADV-QUIRK-54` on) or when the
        // level would not rise (off).
        let again = if strict { C::NotHigher } else { C::AlreadyOn };
        cast(&mut state, rules, 2, FLY, None).unwrap();
        refused(&mut state, rules, (2, FLY, None), M::Adventure(again));
        cast(&mut state, rules, 4, WATER_WALK, None).unwrap();
        refused(
            &mut state,
            rules,
            (4, WATER_WALK, None),
            M::Adventure(again),
        );
        // In a boat: no summon, no flight.
        cast(&mut state, rules, 0, SUMMON_BOAT, None).unwrap();
        walk(&mut state, rules, 0, (14, 4));
        refused(
            &mut state,
            rules,
            (0, SUMMON_BOAT, None),
            M::Adventure(C::InBoat),
        );
        refused(&mut state, rules, (0, FLY, None), M::Adventure(C::InBoat));
        // Cursed Ground: nothing above level 1 (Summon Boat's 1 is `spells-chance-s2`'s).
        walk(&mut state, rules, 1, (13, 21));
        refused(
            &mut state,
            rules,
            (1, SCUTTLE_BOAT, Some((14, 21))),
            M::Ground,
        );
        refused(&mut state, rules, (1, FLY, None), M::Ground);
        // The spell points: ten summons at None spend the novice's 80; the eleventh is refused.
        for _ in 0..10 {
            cast(&mut state, rules, 1, SUMMON_BOAT, None).unwrap();
        }
        assert_eq!(state.hero(HeroId::new(1)).unwrap().mana, 0);
        refused(&mut state, rules, (1, SUMMON_BOAT, None), M::SpellPoints);
    }
}
