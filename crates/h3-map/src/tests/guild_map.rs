//! The guild map committed under `fixtures/maps/` (E09 T50, the town epic's design note §97;
//! increment 5's own map): red's main Tower with its Mage Guild 4 standing, its Tavern and its
//! Library buildable, generating red's starting hero; red's Dungeon with its Mana Vortex, a
//! Warlock on its entrance (its garrison hero once it enters) and an Overlord beside it; red's
//! Conflux with its Grail building, the Aurora Borealis, and a guild; blue's main Necropolis with a
//! guild, generating blue's starting hero; required and banned spells in the Tower and the
//! Conflux, one spell the scenario bans; red's Knight without a spell book beside the Conflux, its
//! Wizard beside it too; blue's Knight with one Pikeman beside the Dungeon, to besiege it. Written
//! by [`crate::writer`] as the towns map (`towns_map.rs`); the golden replays `guild-*` play it
//! (`crates/h3-cli/tests/adventure_replays.rs`), each with a test here that plays its commands and
//! checks what it shows. The tests check that the committed file reads to the map written here and
//! imports with its towns, heroes and spell sets. It rewrites the file when `H3_WRITE_FIXTURES` is
//! set (a change invalidates the replays that name the map by content hash, ADV-Q2).

use std::path::PathBuf;

use super::play::FAITHFUL;
use super::towns_map::{at, entrance, town, with_army};
use crate::import::codes;
use crate::writer as w;
use crate::{Limits, RawMap, import, read};
use h3_adventure::AdventureCommand;
use h3_content::SpellId;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::{
    Actor, BuildingId, GameCommand, GameCommandError, GameEvent, GameState, HeroChange, Viewer,
    apply, observe,
};
use h3_towns::TownCommand;

/// The committed file's name.
const NAME: &str = "e09-guild-36.h3m";

/// Faction codes (`TOWNTYPE.TXT`, the writer's subclasses).
const TOWER: u32 = 2;
const NECROPOLIS: u32 = 4;
const DUNGEON: u32 = 5;
const CONFLUX: u32 = 8;

/// Creature codes: Pikemen, Troglodytes and Harpies (speed 6: the Warlock's stack acts before
/// blue's Pikeman).
const PIKEMAN: u16 = 0;
const TROGLODYTE: u16 = 70;
const HARPY: u16 = 72;

/// Building bits (`docs/formats/h3m.md`, "Town buildings").
const TAVERN: u8 = 6;
const MAGE_GUILDS: [u8; 5] = [11, 12, 13, 14, 15];
const GRAIL: u8 = 17;
/// The Dungeon's Mana Vortex (its special building 18).
const MANA_VORTEX: u8 = 18;

/// Spell codes (`SPTRAITS.TXT`'s order, the map's).
const TOWN_PORTAL: u8 = 9;
const MAGIC_ARROW: u8 = 15;
const LIGHTNING_BOLT: u8 = 17;
const SHIELD: u8 = 27;
const BLESS: u8 = 41;
const HASTE: u8 = 53;
const SLOW: u8 = 54;

/// The towns' positions by `TownId` (each entered two columns left of it): red's main Tower
/// "Athenaeum", red's Dungeon "Abyss", red's Conflux "Prism", blue's main Necropolis "Crypt".
const TOWNS: [(u8, u8); 4] = [(10, 8), (10, 20), (24, 8), (28, 28)];
const ATHENAEUM: u16 = 0;
const ABYSS: u16 = 1;
const PRISM: u16 = 2;
const CRYPT: u16 = 3;

/// The heroes placed, by hero id (the map's order): tile, owner, type (`HOTRAITS.TXT`'s order).
/// Red's Warlock Geon on the Abyss' entrance, red's Overlord Lorelei below and right of it; red's
/// Wizard Serena below and left of the Prism's entrance, red's Knight Valeska (no spell book) below
/// and right of it; blue's Knight Sorsha below and left of the Abyss' entrance.
const HEROES: [((u8, u8), u8, u8); 5] = [
    ((8, 20), 0, 92),
    ((9, 21), 0, 80),
    ((21, 9), 0, 42),
    ((23, 9), 0, 1),
    ((7, 21), 1, 5),
];
/// The heroes' ids by name, then the starting heroes the main towns generate (after the placed
/// ones, in player order: TWN-SET-009).
const GEON: u16 = 0;
const LORELEI: u16 = 1;
const SERENA: u16 = 2;
const VALESKA: u16 = 3;
const SORSHA: u16 = 4;
const RED_START: u16 = 5;
const BLUE_START: u16 = 6;

/// A custom set of `built` built, none forbidden.
fn custom(built: &[u8]) -> w::TownBuildings {
    let mut set = [0; 6];
    for &bit in built {
        set[usize::from(bit / 8)] |= 1 << (bit % 8);
    }
    w::TownBuildings::Custom {
        built: set,
        forbidden: [0; 6],
    }
}

/// The map's nine bytes of `spells`.
fn spell_bits(spells: &[u8]) -> [u8; 9] {
    let mut bits = [0; 9];
    for &spell in spells {
        bits[usize::from(spell / 8)] |= 1 << (spell % 8);
    }
    bits
}

/// The map: SoD, 36 × 36 of grass, no underground; red in the west and north, blue in the
/// south-east; the scenario bans Haste (TWN-GLD-002: no guild draws it; TWN-QUIRK-14: the
/// faithful Aurora set holds it).
///
/// - The towns ([`TOWNS`]), every garrison empty: red's main Tower "Athenaeum" on (10, 8), entered
///   from (8, 8), with the Mage Guilds 1 to 4 and a Tavern, generating red's starting hero; it
///   requires Bless, Town Portal and Shield and bans Shield and Magic Arrow (TWN-SET-012: Shield
///   both, never in the guild, TD-25). Red's Dungeon "Abyss" on (10, 20), entered from (8, 20),
///   with the Mage Guild 1 and the Mana Vortex. Red's Conflux "Prism" on (24, 8), entered from
///   (22, 8), with the Mage Guild 1 and the Aurora Borealis (its Grail building); it bans Magic
///   Arrow and Lightning Bolt. Blue's main Necropolis "Crypt" on (28, 28) with the Mage Guild 1,
///   generating blue's starting hero.
/// - The heroes ([`HEROES`]): red's Geon (Warlock, basic Wisdom) with ten Harpies and Magic Arrow
///   and Slow in his book on the Abyss' entrance (its visitor, learning nothing at the start),
///   Lorelei (Overlord, no book) with ten Troglodytes beside it; Serena (Wizard, basic Wisdom) and
///   Valeska (Knight, no book) beside the Prism's entrance, their types' armies; blue's Sorsha
///   (Knight) with one Pikeman beside the Abyss' entrance.
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << TOWER,
    });
    players[1] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << NECROPOLIS,
    });
    let mut objects: Vec<w::MapObject> = (0..)
        .zip(HEROES)
        .map(|(id, (tile, owner, hero_type))| {
            let army: &[(u16, u16)] = match id {
                GEON => &[(HARPY, 10)],
                LORELEI => &[(TROGLODYTE, 10)],
                SORSHA => &[(PIKEMAN, 1)],
                _ => &[],
            };
            let mut hero = with_army(tile, owner, hero_type, army);
            if id == GEON {
                hero.spells = Some(spell_bits(&[MAGIC_ARROW, SLOW]));
            }
            w::MapObject::Hero(hero)
        })
        .collect();
    let generating = Some(w::MainTown {
        generates_hero: true,
    });
    let empty = Some([None; w::ARMY_SLOTS]);
    let guild = |levels: usize| MAGE_GUILDS[..levels].to_vec();
    let towns = [
        (
            "Athenaeum",
            TOWER,
            0,
            generating,
            [guild(4), vec![TAVERN]].concat(),
            w::TownSpells {
                required: vec![TOWN_PORTAL, SHIELD, BLESS],
                banned: vec![MAGIC_ARROW, SHIELD],
            },
        ),
        (
            "Abyss",
            DUNGEON,
            0,
            None,
            [guild(1), vec![MANA_VORTEX]].concat(),
            w::TownSpells::default(),
        ),
        (
            "Prism",
            CONFLUX,
            0,
            None,
            [guild(1), vec![GRAIL]].concat(),
            w::TownSpells {
                required: Vec::new(),
                banned: vec![MAGIC_ARROW, LIGHTNING_BOLT],
            },
        ),
        (
            "Crypt",
            NECROPOLIS,
            1,
            generating,
            guild(1),
            w::TownSpells::default(),
        ),
    ];
    for ((name, faction, owner, main, built, spells), tile) in towns.into_iter().zip(TOWNS) {
        objects.push(w::MapObject::AnyTown(w::AnyTown {
            main,
            name: Some(name.into()),
            garrison: empty,
            buildings: custom(&built),
            spells,
            ..town(tile, Some(faction), Some(owner))
        }));
    }
    w::Map {
        name: "Guild".into(),
        description: "E09 T50: the mage guilds' learning, the Library, the Aurora Borealis, the \
                      Mana Vortex, the night's refill, the spell book"
            .into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: Vec::new(),
        banned_spells: vec![HASTE],
        events: Vec::new(),
        teams: None,
    }
}

/// The committed map, read.
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

/// A game of the committed map at `seed`.
fn game(seed: u64) -> GameState {
    let setup = import(&committed(), &FAITHFUL).unwrap();
    GameState::new(&FAITHFUL, &setup, seed).unwrap()
}

/// `player`'s `command`, applied; its events.
fn step(state: &mut GameState, player: u8, command: GameCommand) -> Vec<GameEvent> {
    let mut events = Vec::new();
    let actor = Actor::Player(PlayerId::new(player));
    let result = apply(state, &FAITHFUL, actor, command.clone(), &mut events);
    assert!(result.is_ok(), "{command:?}: {result:?}");
    events
}

/// `player`'s `command`, refused with `error`: the state and its hash untouched.
fn refused(state: &GameState, player: u8, command: GameCommand, error: &GameCommandError) {
    let mut copy = state.clone();
    let mut events = Vec::new();
    let actor = Actor::Player(PlayerId::new(player));
    let result = apply(&mut copy, &FAITHFUL, actor, command, &mut events);
    assert_eq!(result.as_ref().err(), Some(error), "{result:?}");
    assert!(events.is_empty());
    assert_eq!(copy.state_hash(), state.state_hash());
}

/// A move of `hero` onto `tile`, a neighbouring tile.
fn to(hero: u16, tile: (u8, u8)) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: vec![at(tile)],
    })
}

/// A town command.
fn town_command(command: TownCommand) -> GameCommand {
    GameCommand::Town(command)
}

/// The opening of `town`'s town screen (TWN-GLD-006).
fn open(town: u16) -> GameCommand {
    town_command(TownCommand::Open {
        town: TownId::new(town),
    })
}

/// Red's and then blue's turn ended, `days` times.
fn days(state: &mut GameState, days: usize) {
    for _ in 0..days {
        step(state, 0, GameCommand::EndTurn);
        step(state, 1, GameCommand::EndTurn);
    }
}

/// The spell of key `key`.
fn spell(key: &str) -> SpellId {
    FAITHFUL.core().spell_by_key(key).unwrap()
}

/// The building of key `key` in `town`'s faction.
fn building(state: &GameState, town: u16, key: &str) -> BuildingId {
    let record = state.towns().get(TownId::new(town)).unwrap();
    let faction = FAITHFUL.towns().town(record.faction()).unwrap();
    faction.building_by_key(key).unwrap()
}

/// The spells `hero` knows, in its book's order.
fn known(state: &GameState, hero: u16) -> Vec<SpellId> {
    state.hero(HeroId::new(hero)).unwrap().spells.clone()
}

/// `hero`'s spell points and its maximum (HER-MAN-001).
fn mana(state: &GameState, hero: u16) -> (u16, u16) {
    let details = state.hero(HeroId::new(hero)).unwrap();
    (details.mana, h3_heroes::max_mana(details, FAITHFUL.core()))
}

/// What `town` offers its heroes now, read apart from the engine's `teaches` (TWN-GLD-004):
/// nothing without a guild ever built; with the Aurora Borealis ever built its set in spell
/// order, else the guild's shown spells level after level, each level's in slot order
/// (TWN-GLD-003).
fn offered(state: &GameState, town: u16) -> Vec<SpellId> {
    let part = FAITHFUL.towns();
    let id = TownId::new(town);
    let record = state.towns().get(id).unwrap();
    let faction = part.town(record.faction()).unwrap();
    let roles = |key: fn(&h3_towns::BuildingRoles) -> bool| {
        record
            .built()
            .iter()
            .filter_map(|b| faction.building(b))
            .any(|b| key(&b.roles))
    };
    if !roles(|r| r.guild.is_some()) {
        Vec::new()
    } else if roles(|r| r.aurora) {
        let aurora = state.towns().town_static(id).unwrap().aurora;
        aurora.iter().collect()
    } else {
        (1..=5)
            .flat_map(|level| h3_towns::shown(record.guild(), record, part, level))
            .collect()
    }
}

/// Of `offered`, in order, what `hero` learns knowing `knows`: nothing without a spell book
/// worn, else the spells of a level at most its learnable level (HER-SPL-005) it does not know.
fn learnable(state: &GameState, hero: u16, offered: &[SpellId], knows: &[SpellId]) -> Vec<SpellId> {
    let core = FAITHFUL.core();
    let details = state.hero(HeroId::new(hero)).unwrap();
    if !h3_heroes::wears_spell_book(details, core) {
        return Vec::new();
    }
    let top = h3_heroes::learn_level(details, core);
    offered
        .iter()
        .copied()
        .filter(|&s| core.spell(s).unwrap().level <= top && !knows.contains(&s))
        .collect()
}

/// What `town` would teach `hero` now ([`offered`], [`learnable`]).
fn would_learn(state: &GameState, town: u16, hero: u16) -> Vec<SpellId> {
    learnable(state, hero, &offered(state, town), &known(state, hero))
}

/// What `town`'s guild shows at each level 1 to 5 (TWN-GLD-003).
fn shows(state: &GameState, town: u16) -> Vec<Vec<SpellId>> {
    let record = state.towns().get(TownId::new(town)).unwrap();
    (1..=5)
        .map(|level| h3_towns::shown(record.guild(), record, FAITHFUL.towns(), level).collect())
        .collect()
}

/// The spells `events` tell `player` its `hero` learned (`HeroChanged` with `LearnSpell`), in
/// order.
fn learned(events: &[GameEvent], hero: u16) -> Vec<SpellId> {
    events
        .iter()
        .filter_map(|e| match e {
            GameEvent::HeroChanged {
                hero: h,
                change: HeroChange::LearnSpell(spell),
                ..
            } if h.raw() == hero => Some(*spell),
            _ => None,
        })
        .collect()
}

/// Whether `town`'s Mana Vortex has its week's flag set, in the record and in its owner's view
/// (TWN-VEW-004).
fn vortex(state: &GameState, town: u16) -> bool {
    let id = TownId::new(town);
    let flag = state.towns().get(id).unwrap().vortex();
    let view = observe(state, &FAITHFUL, Viewer::Player(PlayerId::new(0)));
    let seen = view.towns.iter().find(|t| t.town == id).unwrap();
    assert_eq!(seen.buildings.as_ref().map(|b| b.vortex), Some(flag));
    flag
}

// covers: TWN-SET-012, TWN-GLD-002, TWN-GLD-003, TWN-GLD-004
#[test]
fn the_guild_map_is_committed_as_written() {
    let committed = committed();
    let setup = import(&committed, &FAITHFUL).unwrap();
    setup.validate(&FAITHFUL).unwrap();
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let core = FAITHFUL.core();
    let code = |c: u8| codes::SPELLS[usize::from(c)];
    let id = |c: u8| core.spell_by_key(code(c)).unwrap();
    // The towns' spell sets as the map gives them (TWN-SET-012), in spell order.
    let sets: Vec<_> = setup
        .towns
        .iter()
        .map(|t| (t.required_spells.clone(), t.banned_spells.clone()))
        .collect();
    assert_eq!(
        sets,
        [
            (
                vec![id(TOWN_PORTAL), id(SHIELD), id(BLESS)],
                vec![id(MAGIC_ARROW), id(SHIELD)]
            ),
            (Vec::new(), Vec::new()),
            (Vec::new(), vec![id(MAGIC_ARROW), id(LIGHTNING_BOLT)]),
            (Vec::new(), Vec::new()),
        ]
    );
    // The heroes placed, then the two starting heroes on their main towns' entrances.
    for (hero, &(tile, owner, _)) in HEROES.iter().enumerate() {
        let on_map = state
            .adventure()
            .hero(HeroId::new(u16::try_from(hero).unwrap()))
            .unwrap();
        assert_eq!((on_map.pos, on_map.owner.raw()), (at(tile), owner));
    }
    for (hero, town, owner) in [(RED_START, 0, 0), (BLUE_START, 3, 1)] {
        let on_map = state.adventure().hero(HeroId::new(hero)).unwrap();
        assert_eq!(
            (on_map.pos, on_map.owner.raw()),
            (entrance(TOWNS[town]), owner)
        );
    }
    // The guilds drawn (TWN-GLD-002): the Athenaeum shows its required Bless first at level 1
    // and Town Portal at level 4, never Shield (required and banned, TD-25) nor Magic Arrow; no
    // guild holds Haste, which the scenario bans; the guilds show up to their levels (TWN-GLD-003).
    let athenaeum = shows(&state, ATHENAEUM);
    assert_eq!(athenaeum[0].first(), Some(&spell("base:bless")));
    assert_eq!(athenaeum[3].first(), Some(&spell("base:town_portal")));
    assert!(athenaeum[..4].iter().all(|level| !level.is_empty()));
    assert!(athenaeum[4].is_empty());
    for town in [ATHENAEUM, ABYSS, PRISM, CRYPT] {
        let record = state.towns().get(TownId::new(town)).unwrap();
        let held: Vec<SpellId> = (1..=5)
            .flat_map(|level| record.guild().level(level).flatten().collect::<Vec<_>>())
            .collect();
        assert!(!held.contains(&spell("base:haste")), "{town}");
        if town == ATHENAEUM {
            for banned in ["base:shield", "base:magic_arrow"] {
                assert!(!held.contains(&spell(banned)), "{banned}");
            }
        }
        let levels = shows(&state, town);
        let top = if town == ATHENAEUM { 4 } else { 1 };
        for (level, shown) in (1..).zip(&levels) {
            assert_eq!(!shown.is_empty(), level <= top, "{town}'s level {level}");
        }
    }
    // The Prism's Aurora set (TWN-GLD-004): every hero spell a guild may hold but its own bans;
    // Haste in it (TWN-QUIRK-14, faithful).
    let aurora = state
        .towns()
        .town_static(TownId::new(PRISM))
        .unwrap()
        .aurora;
    assert!(aurora.contains(spell("base:haste")));
    assert!(aurora.contains(spell("base:implosion")));
    for banned in ["base:magic_arrow", "base:lightning_bolt"] {
        assert!(!aurora.contains(spell(banned)), "{banned}");
    }
    // Each Vortex flag set at the build (TWN-GLD-009): the Abyss' alone has a Mana Vortex.
    assert!(vortex(&state, ABYSS));
}

/// The `SetSpellPoints` `events` tell for `hero`, in order.
fn points_set(events: &[GameEvent], hero: u16) -> Vec<u16> {
    events
        .iter()
        .filter_map(|e| match e {
            GameEvent::HeroChanged {
                hero: h,
                change: HeroChange::SetSpellPoints(points),
                ..
            } if h.raw() == hero => Some(*points),
            _ => None,
        })
        .collect()
}

/// The spells of `hero`'s type (HOTRAITS' book).
fn own_spells(state: &GameState, hero: u16) -> Vec<SpellId> {
    let core = FAITHFUL.core();
    let ty = state.hero(HeroId::new(hero)).unwrap().hero_type;
    core.hero_type(ty).unwrap().spells.clone()
}

// covers: TWN-GLD-005, TWN-SET-009, HER-MAN-002
#[test]
fn the_starting_heroes_learn_at_the_start() {
    // The golden replay `guild-start-s1`. At the game's build each main town's starting hero, its
    // visitor, has learned what the town's guild shows at the levels it can learn (TD-10, in
    // player order: red's at the Athenaeum, the required Bless among them; blue's at the Crypt);
    // Geon, placed on the Abyss' entrance, learned nothing (his book as the map gives it). The
    // turns' ends and day 2's start teach nothing and every hero keeps its maximum spell points.
    let mut state = game(1);
    for (hero, town) in [(RED_START, ATHENAEUM), (BLUE_START, CRYPT)] {
        let own = own_spells(&state, hero);
        let taught = learnable(&state, hero, &offered(&state, town), &own);
        assert!(!taught.is_empty(), "{hero}");
        assert_eq!(known(&state, hero), [own, taught].concat(), "{hero}");
        assert!(would_learn(&state, town, hero).is_empty());
    }
    assert!(known(&state, RED_START).contains(&spell("base:bless")));
    let arrow_and_slow = vec![spell("base:magic_arrow"), spell("base:slow")];
    assert_eq!(known(&state, GEON), arrow_and_slow);
    assert!(!would_learn(&state, ABYSS, GEON).is_empty());
    let heroes = [
        GEON, LORELEI, SERENA, VALESKA, SORSHA, RED_START, BLUE_START,
    ];
    let before: Vec<_> = heroes.iter().map(|&h| known(&state, h)).collect();
    days(&mut state, 1);
    assert_eq!(state.day(), 2);
    let after: Vec<_> = heroes.iter().map(|&h| known(&state, h)).collect();
    assert_eq!(after, before);
    for hero in heroes {
        let (points, max) = mana(&state, hero);
        assert_eq!(points, max, "{hero}");
    }
}

// covers: TWN-GLD-005, TWN-GLD-004
#[test]
fn a_visit_learns_the_aurora_set() {
    // The golden replay `guild-visit-s1`. Day 1, red: Serena (a Wizard, basic Wisdom: levels 1 to
    // 3) steps onto the Prism's entrance; the visit's end teaches her the Conflux's Aurora set at
    // those levels in spell order (the Aurora Borealis ever built, the guild's level
    // notwithstanding), each told to red: Haste, which the scenario bans, among them (TWN-QUIRK-14,
    // faithful), never the Prism's banned Magic Arrow or Lightning Bolt.
    let mut state = game(1);
    let before = known(&state, SERENA);
    let expected = would_learn(&state, PRISM, SERENA);
    assert!(expected.len() > shows(&state, PRISM)[..3].iter().flatten().count());
    let events = step(&mut state, 0, to(SERENA, (22, 8)));
    assert_eq!(
        state.adventure().hero(HeroId::new(SERENA)).unwrap().pos,
        entrance(TOWNS[2])
    );
    assert_eq!(learned(&events, SERENA), expected);
    assert_eq!(known(&state, SERENA), [before, expected].concat());
    let knows = known(&state, SERENA);
    assert!(knows.contains(&spell("base:haste")));
    for banned in ["base:magic_arrow", "base:lightning_bolt"] {
        assert!(!knows.contains(&spell(banned)), "{banned}");
    }
    assert!(
        knows
            .iter()
            .all(|&s| FAITHFUL.core().spell(s).unwrap().level <= 3)
    );
    step(&mut state, 0, GameCommand::EndTurn);
}

// covers: TWN-GLD-003, TWN-GLD-005, TWN-GLD-009, TWN-BLD-005
#[test]
fn a_library_and_a_guild_level_built_with_visitors() {
    // The golden replay `guild-builds-s1`. Day 1, red: the Athenaeum builds its Library with red's
    // starting hero visiting: every level 1 to 4 shows one more slot at once (the spare, TWN-GLD-003)
    // and the hero learns the new ones it can learn, after the `TownBuilt`. The Abyss builds its
    // Mage Guild 2 with Geon visiting: he learns levels 1 and 2 (no trigger taught him level 1
    // before), then its Mana Vortex gives him twice his maximum spell points (after the build's
    // learning, TWN-GLD-009) and its week's flag clears. Both builds paid; the turns end.
    let mut state = game(1);
    let counts = |state: &GameState| -> Vec<usize> {
        shows(state, ATHENAEUM).iter().map(Vec::len).collect()
    };
    let without = counts(&state);
    let before = known(&state, RED_START);
    let library = building(&state, ATHENAEUM, "library");
    let build = |town: u16, building: BuildingId| {
        town_command(TownCommand::Build {
            town: TownId::new(town),
            building,
        })
    };
    let events = step(&mut state, 0, build(ATHENAEUM, library));
    let with: Vec<usize> = counts(&state);
    assert_eq!(&with[4..], &without[4..]);
    for level in 0..4 {
        assert_eq!(with[level], without[level] + 1, "level {}", level + 1);
    }
    let spare: Vec<SpellId> = shows(&state, ATHENAEUM)
        .iter()
        .take(4)
        .filter_map(|level| level.last().copied())
        .collect();
    let taught = learnable(&state, RED_START, &spare, &before);
    assert!(!taught.is_empty());
    assert_eq!(learned(&events, RED_START), taught);
    assert_eq!(known(&state, RED_START), [before, taught].concat());
    let built = events
        .iter()
        .position(|e| matches!(e, GameEvent::TownBuilt { .. }))
        .unwrap();
    let first = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroChanged { .. }))
        .unwrap();
    assert!(built < first);
    let gold = state.treasury(PlayerId::new(0)).unwrap()[6];
    assert_eq!(gold, 18_500);
    // The Abyss' second level.
    let knew = known(&state, GEON);
    let (points, max) = mana(&state, GEON);
    assert_eq!(points, max);
    assert!(vortex(&state, ABYSS));
    let level_2 = building(&state, ABYSS, "mage_guild_level_2");
    let events = step(&mut state, 0, build(ABYSS, level_2));
    let taught = learnable(&state, GEON, &offered(&state, ABYSS), &knew);
    let levels: Vec<u8> = taught
        .iter()
        .map(|&s| FAITHFUL.core().spell(s).unwrap().level)
        .collect();
    assert!(levels.contains(&1) && levels.contains(&2), "{levels:?}");
    assert_eq!(learned(&events, GEON), taught);
    assert_eq!(points_set(&events, GEON), [2 * max]);
    let last_learned = events
        .iter()
        .rposition(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::LearnSpell(_),
                    ..
                }
            )
        })
        .unwrap();
    let set = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::SetSpellPoints(_),
                    ..
                }
            )
        })
        .unwrap();
    assert!(last_learned < set);
    assert_eq!(mana(&state, GEON), (2 * max, max));
    assert!(!vortex(&state, ABYSS));
    assert_eq!(state.treasury(PlayerId::new(0)).unwrap()[6], 17_500);
    days(&mut state, 1);
}

// covers: TWN-GLD-005, TWN-TAV-003, TWN-TAV-001
#[test]
fn a_hire_at_the_tower_learns() {
    // The golden replay `guild-hire-s1`. Day 1, red: its starting hero steps off the Athenaeum's
    // entrance; the Athenaeum hires the hero of slot 0 (of the Tower's classes, TWN-TAV-001: a
    // spell book), who stands on the entrance and learns what the guild shows at the levels it can
    // learn, after the hire's events (TWN-GLD-005), each told to red.
    let mut state = game(1);
    step(&mut state, 0, to(RED_START, (8, 9)));
    let slot = state.player_tavern(PlayerId::new(0)).unwrap().slots[0].unwrap();
    let hero = state.hero_record(slot).unwrap().raw();
    let expected = would_learn(&state, ATHENAEUM, hero);
    let before = known(&state, hero);
    assert!(!expected.is_empty());
    let hire = town_command(TownCommand::Hire {
        town: TownId::new(ATHENAEUM),
        slot: 0,
    });
    let events = step(&mut state, 0, hire);
    let on_map = state.adventure().hero(HeroId::new(hero)).unwrap();
    assert_eq!(on_map.pos, entrance(TOWNS[0]));
    assert_eq!(learned(&events, hero), expected);
    let hired = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroHired { .. }))
        .unwrap();
    let first = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::LearnSpell(_),
                    ..
                }
            )
        })
        .unwrap();
    assert!(hired < first);
    assert_eq!(known(&state, hero), [before, expected].concat());
    assert!(known(&state, hero).contains(&spell("base:bless")));
    step(&mut state, 0, GameCommand::EndTurn);
}

// covers: TWN-GLD-007, TWN-GLD-005, TWN-GLD-004
#[test]
fn a_spell_book_bought_then_learning() {
    // The golden replay `guild-book-s1`. Day 1, red: the Prism has no hero (a book refused,
    // NoHero); Valeska (a Knight, no book) steps onto its entrance and learns nothing at the
    // visit's end; red buys her a spell book for 500 gold: the book worn, then the Aurora set's
    // levels 1 and 2 learned (no Wisdom), each told to red. A second book is refused
    // (HasSpellBook).
    let mut state = game(1);
    let book = town_command(TownCommand::BuySpellBook {
        town: TownId::new(PRISM),
    });
    let no_hero = GameCommandError::Town(h3_towns::TownCommandError::NoHero);
    refused(&state, 0, book.clone(), &no_hero);
    let events = step(&mut state, 0, to(VALESKA, (22, 8)));
    assert!(learned(&events, VALESKA).is_empty());
    assert!(known(&state, VALESKA).is_empty());
    let core = FAITHFUL.core();
    assert!(!h3_heroes::wears_spell_book(
        state.hero(HeroId::new(VALESKA)).unwrap(),
        core
    ));
    let events = step(&mut state, 0, book.clone());
    assert!(h3_heroes::wears_spell_book(
        state.hero(HeroId::new(VALESKA)).unwrap(),
        core
    ));
    assert_eq!(state.treasury(PlayerId::new(0)).unwrap()[6], 19_500);
    let added = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::AddArtifact(_),
                    ..
                }
            )
        })
        .unwrap();
    let taught = learned(&events, VALESKA);
    let aurora = state
        .towns()
        .town_static(TownId::new(PRISM))
        .unwrap()
        .aurora;
    let expected: Vec<SpellId> = aurora
        .iter()
        .filter(|&s| core.spell(s).unwrap().level <= 2)
        .collect();
    assert_eq!(taught, expected);
    assert!(taught.contains(&spell("base:haste")));
    let first = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::LearnSpell(_),
                    ..
                }
            )
        })
        .unwrap();
    assert!(added < first);
    assert_eq!(known(&state, VALESKA), expected);
    let has_book = GameCommandError::Town(h3_towns::TownCommandError::HasSpellBook);
    refused(&state, 0, book, &has_book);
    step(&mut state, 0, GameCommand::EndTurn);
}

// covers: TWN-GLD-009, TWN-GLD-006, TWN-GLD-008, TWN-ROW-002
#[test]
fn the_vortex_over_two_weeks() {
    // The golden replay `guild-vortex-s1`. Day 1, red: Geon enters the Abyss' garrison (no
    // learning); red opens the Abyss: its Mana Vortex gives the garrison hero twice his maximum
    // spell points and clears its week's flag, then he learns the guild's spells; Lorelei steps
    // onto the entrance: the flag cleared, the visit's end gives her nothing (no book: no
    // learning). Every night to day 8 keeps Geon's points above his maximum (the night's refill
    // never lowers, TWN-GLD-008). Day 8: the week's first day sets the flag again; red opens the
    // Abyss: Geon at twice his maximum is passed over, Lorelei takes the Vortex; a second opening
    // changes nothing (no event, the state's hash unchanged).
    let mut state = game(1);
    let enter = town_command(TownCommand::Heroes {
        town: TownId::new(ABYSS),
        change: h3_towns::TownHeroChange::Enter,
    });
    let events = step(&mut state, 0, enter);
    assert!(learned(&events, GEON).is_empty());
    assert!(
        state
            .adventure()
            .town_heroes()
            .iter()
            .any(|(t, h)| (t.raw(), h.id.raw()) == (ABYSS, GEON))
    );
    assert!(vortex(&state, ABYSS));
    let taught = would_learn(&state, ABYSS, GEON);
    assert!(!taught.is_empty());
    let (_, geon) = mana(&state, GEON);
    let events = step(&mut state, 0, open(ABYSS));
    assert_eq!(points_set(&events, GEON), [2 * geon]);
    assert_eq!(learned(&events, GEON), taught);
    assert!(!vortex(&state, ABYSS));
    let (_, lorelei) = mana(&state, LORELEI);
    let events = step(&mut state, 0, to(LORELEI, (8, 20)));
    assert!(points_set(&events, LORELEI).is_empty());
    assert!(learned(&events, LORELEI).is_empty());
    assert_eq!(mana(&state, LORELEI), (lorelei, lorelei));
    for day in 2..=8 {
        days(&mut state, 1);
        assert_eq!(state.day(), day);
        assert_eq!(mana(&state, GEON), (2 * geon, geon), "day {day}");
        assert_eq!(mana(&state, LORELEI), (lorelei, lorelei), "day {day}");
        assert_eq!(vortex(&state, ABYSS), day == 8, "day {day}");
    }
    let events = step(&mut state, 0, open(ABYSS));
    assert!(points_set(&events, GEON).is_empty());
    assert_eq!(points_set(&events, LORELEI), [2 * lorelei]);
    assert!(!vortex(&state, ABYSS));
    let hash = state.state_hash();
    let events = step(&mut state, 0, open(ABYSS));
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(state.state_hash(), hash);
    step(&mut state, 0, GameCommand::EndTurn);
}

// covers: TWN-GLD-005, TWN-GLD-008, TWN-SIE-004, BAT-MAG-001
#[test]
fn a_siege_won_by_the_defender_teaches() {
    // The golden replay `guild-siege-s1`. Day 1: red ends its turn; blue's Sorsha attacks the
    // Abyss, Geon its visitor; Geon's Harpies act first and he casts Magic Arrow at Sorsha's
    // Pikeman, which ends the battle, Sorsha defeated. The siege's end teaches Geon the Abyss'
    // spells (the defending visitor that won, TWN-GLD-005), the Vortex not given (its flag kept);
    // he keeps the points the cast left him (TWN-SIE-004). The night refills them to his maximum
    // (TWN-GLD-008), more than the day's regain alone.
    let mut state = game(1);
    step(&mut state, 0, GameCommand::EndTurn);
    let taught = would_learn(&state, ABYSS, GEON);
    assert!(!taught.is_empty());
    let (_, max) = mana(&state, GEON);
    step(&mut state, 1, to(SORSHA, (8, 20)));
    let battle = state.battle(PlayerId::new(0)).unwrap();
    assert_eq!(battle.owner_to_act(), Some(PlayerId::new(0)));
    let pikeman = battle
        .state()
        .stacks()
        .iter()
        .position(|s| s.side == h3_battle::BattleSide::Attacker)
        .unwrap();
    let cast = GameCommand::Battle(h3_battle::Command::Cast {
        spell: spell("base:magic_arrow"),
        target: h3_battle::CastTarget::Stack(h3_battle::StackId(u8::try_from(pikeman).unwrap())),
        second: h3_battle::CastTarget::None,
    });
    let events = step(&mut state, 0, cast);
    assert!(state.battle(PlayerId::new(0)).is_none());
    assert!(state.adventure().hero(HeroId::new(SORSHA)).is_none());
    assert_eq!(learned(&events, GEON), taught);
    assert!(vortex(&state, ABYSS));
    let (left, _) = mana(&state, GEON);
    assert!(left < max, "{left} of {max}");
    let regained = h3_heroes::day_mana(state.hero(HeroId::new(GEON)).unwrap(), FAITHFUL.core());
    assert!(regained < max);
    step(&mut state, 1, GameCommand::EndTurn);
    assert_eq!(state.day(), 2);
    assert_eq!(mana(&state, GEON), (max, max));
}
