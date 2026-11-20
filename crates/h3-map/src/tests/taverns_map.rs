//! The taverns map committed under `fixtures/maps/` (E09 T40b, the town epic's design note §78;
//! increment 4's own map, decision 159): two players with Taverns (red's Castle with the
//! Brotherhood of the Sword), a map Tavern, a Den of Thieves, a wandering monster to retreat from,
//! heroes of both players side by side to attack, retreat and surrender, and few hero types the
//! taverns may offer (the map's allowed heroes), so that a freed type comes back. Written by
//! [`crate::writer`] with the original's templates, its allowed heroes then set in the raw model
//! (the writer writes the default set) and encoded; the golden replays `taverns-*` play it
//! (`crates/h3-cli/tests/adventure_replays.rs`), each with a test here that plays its commands
//! and checks what it shows. The tests check that the committed file reads to the map built here
//! and imports with its towns, objects and allowed types. It rewrites the file when
//! `H3_WRITE_FIXTURES` is set (a change invalidates the replays that name the map by content
//! hash, ADV-Q2).

use std::path::PathBuf;

use super::encode::encode;
use super::play::FAITHFUL;
use super::towns_map::{at, entrance, town, with_army};
use crate::model::{Bits, RawMap};
use crate::writer as w;
use crate::{Limits, import, read};
use h3_adventure::{AdventureCommand, ObjectKind, Pos};
use h3_battle::Command;
use h3_content::HeroTypeId;
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_game::{
    Actor, Answer, GameCommand, GameEvent, GameState, HeroStatus, Manage, TavernAnswer, Viewer,
    apply, observe,
};
use h3_towns::TownCommand;

/// The committed file's name.
const NAME: &str = "e09-taverns-36.h3m";

/// Faction codes (`TOWNTYPE.TXT`, the writer's subclasses).
const CASTLE: u32 = 0;
const RAMPART: u32 = 1;

/// Creature codes: Royal Griffins (speed 9: they act first in every battle here), Dwarves and
/// Dendroid Guards (speed 3).
const ROYAL_GRIFFIN: u16 = 5;
const DWARF: u16 = 16;
const DENDROID_GUARD: u16 = 22;

/// Building bits (`docs/formats/h3m.md`, "Town buildings").
const FORT: u8 = 3;
const TAVERN: u8 = 6;
const BROTHERHOOD: u8 = 19;

/// The Charm of Mana's code (`ARTRAITS.TXT`), in Valeska's backpack (T-14: a dismissed hero's
/// artifacts come back with it).
const CHARM_OF_MANA: u16 = 73;

/// The towns' positions by `TownId` (each entered two columns left of it): red's main Castle
/// "Bastion" and blue's main Rampart "Glade".
const TOWNS: [(u8, u8); 2] = [(8, 8), (28, 8)];

/// The heroes placed, by hero id (the map's order): tile, owner, type (`HOTRAITS.TXT`'s order).
/// Red's Orrin beside the golems; blue's Mephala between red's Sorsha (east) and Cuthbert (west);
/// red's Valeska below the map Tavern's visitable tile; blue's Ufretin east of red's Edric; blue's
/// Ryland below the Den of Thieves.
const HEROES: [((u8, u8), u8, u8); 8] = [
    ((4, 20), 0, 0),
    ((20, 20), 1, 16),
    ((11, 15), 0, 1),
    ((21, 20), 0, 5),
    ((19, 20), 0, 10),
    ((15, 30), 1, 17),
    ((14, 30), 0, 2),
    ((26, 21), 1, 19),
];
/// The heroes' ids by name.
const ORRIN: u16 = 0;
const MEPHALA: u16 = 1;
const VALESKA: u16 = 2;
const SORSHA: u16 = 3;
const CUTHBERT: u16 = 4;
const UFRETIN: u16 = 5;
const EDRIC: u16 = 6;
const RYLAND: u16 = 7;

/// The hero types the map allows besides the placed ones (its allowed heroes): two of each of
/// the players' factions' classes (Castle: the clerics Rion and Adela; Rampart: the ranger Jenova
/// and the druid Coronius) and two alchemists (Piquedram, Thane); none has Tactics, Artillery or
/// First Aid (no tactics phase, no war machine).
const FREE: [u8; 6] = [8, 9, 18, 24, 32, 33];

/// The adventure map's Tavern (class 95; `taverns.rs`' `PICTURES`): its anchor, visited from a
/// neighbour on the tile west of it.
const MAP_TAVERN: (u8, u8) = (12, 14);
const MAP_TAVERN_TILE: (u8, u8) = (11, 14);
/// The Den of Thieves (class 97), entered at its anchor.
const DEN: (u8, u8) = (26, 20);
/// Forty Stone Golems (savage, never fleeing: their battle starts at Orrin's step), east of Orrin.
const GOLEMS: (u8, u8) = (5, 20);

/// The game's `Objects.txt` lines (`H3bitmap.lod`'s): the Tavern, the Den, the Stone Golems.
const TAVERN_LINE: &str = "AVXtvrn0.def 001111111111111111111111111111111111111111111111 \
     010000000000000000000000000000000000000000000000 111111111 011111111 95 0 0 0";
const DEN_LINE: &str = "AVXdend0.def 001111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 011111111 97 0 0 0";
const GOLEMS_LINE: &str = "AVWgolm0.def 011111111111111111111111111111111111111111111111 \
     100000000000000000000000000000000000000000000000 111111111 000000001 54 32 2 0";

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

/// The map: SoD, 36 × 36 of grass, no underground; red (Castle) in the west, blue (Rampart) in
/// the east.
///
/// - The towns ([`TOWNS`]): red's main Castle "Bastion" on (8, 8), entered from (6, 8): a Fort,
///   a Tavern and the Brotherhood of the Sword; blue's main Rampart "Glade" on (28, 8), entered
///   from (26, 8): a Fort and a Tavern. Neither generates a hero.
/// - The heroes ([`HEROES`]): red's with five Royal Griffins each (Valeska with the Charm of Mana
///   in her backpack), blue's Mephala with twenty Dendroid Guards, Ufretin and Ryland with ten
///   Dwarves each.
/// - The map Tavern on (12, 14), visited from (11, 14)'s neighbours ([`MAP_TAVERN`]); the Den of
///   Thieves on (26, 20) ([`DEN`]); forty savage Stone Golems on (5, 20) that never flee
///   ([`GOLEMS`]).
fn map() -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << CASTLE,
    });
    players[1] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << RAMPART,
    });
    let mut objects: Vec<w::MapObject> = HEROES
        .iter()
        .map(|&(tile, owner, hero_type)| {
            let army = match owner {
                0 => [(ROYAL_GRIFFIN, 5)],
                _ if hero_type == 16 => [(DENDROID_GUARD, 20)],
                _ => [(DWARF, 10)],
            };
            let mut hero = with_army(tile, owner, hero_type, &army);
            if u16::from(hero_type) == u16::from(HEROES[usize::from(VALESKA)].2) {
                hero.artifacts = Some(w::HeroArtifacts {
                    worn: [None; w::ARTIFACT_SLOTS],
                    backpack: vec![CHARM_OF_MANA],
                });
            }
            w::MapObject::Hero(hero)
        })
        .collect();
    let [bastion, glade] = TOWNS;
    let main = Some(w::MainTown {
        generates_hero: false,
    });
    objects.push(w::MapObject::AnyTown(w::AnyTown {
        main,
        name: Some("Bastion".into()),
        buildings: custom(&[FORT, TAVERN, BROTHERHOOD]),
        ..town(bastion, Some(CASTLE), Some(0))
    }));
    objects.push(w::MapObject::AnyTown(w::AnyTown {
        main,
        name: Some("Glade".into()),
        buildings: custom(&[FORT, TAVERN]),
        ..town(glade, Some(RAMPART), Some(1))
    }));
    for (line, tile) in [(TAVERN_LINE, MAP_TAVERN), (DEN_LINE, DEN)] {
        objects.push(w::MapObject::Plain(w::Plain {
            tile,
            template: w::ObjectTemplate::from_objects_txt(line).unwrap(),
        }));
    }
    objects.push(w::MapObject::Monster(w::Monster {
        tile: GOLEMS,
        template: w::ObjectTemplate::from_objects_txt(GOLEMS_LINE).unwrap(),
        count: 40,
        disposition: 4,
        never_flees: true,
        fixed_count: true,
        treasure: None,
    }));
    w::Map {
        name: "Taverns".into(),
        description: "E09 T40b: the taverns' fills, hires, leaves and returns, the dismissal, \
                      the Den of Thieves"
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
        banned_spells: Vec::new(),
        events: Vec::new(),
        teams: None,
    }
}

/// The map's allowed heroes: the placed types and [`FREE`] (SoD's 20 bytes).
fn allowed() -> Bits {
    let mut bits = vec![0u8; 20];
    for code in HEROES.iter().map(|&(_, _, t)| t).chain(FREE) {
        bits[usize::from(code / 8)] |= 1 << (code % 8);
    }
    Bits(bits)
}

/// The map as built here: the writer's, read back, with [`allowed`] as its allowed heroes.
fn built() -> RawMap {
    let bytes = w::gzip(&w::write(&map()).unwrap());
    let mut raw = read(&bytes, &Limits::default()).unwrap();
    raw.allowed_heroes = allowed();
    raw
}

/// The committed map, read.
fn committed() -> RawMap {
    let raw = built();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/maps")
        .join(NAME);
    if std::env::var_os("H3_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, w::gzip(&encode(&raw))).unwrap();
    }
    let file = std::fs::read(&path).unwrap();
    let committed = read(&file, &Limits::default()).unwrap();
    assert!(
        committed == raw,
        "{NAME} is not the map built here: H3_WRITE_FIXTURES=1 rewrites it"
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

/// A move of `hero` onto `to`, a neighbouring tile.
fn to(hero: u16, tile: (u8, u8)) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: vec![at(tile)],
    })
}

/// The battle command `command` (of the side to act).
fn battle(command: Command) -> GameCommand {
    GameCommand::Battle(command)
}

/// `player`'s and then the other's turn ended, `days` times.
fn days(state: &mut GameState, first: u8, days: usize) {
    for _ in 0..days {
        step(state, first, GameCommand::EndTurn);
        step(state, 1 - first, GameCommand::EndTurn);
    }
}

/// `player`'s two tavern heroes' types and returning marks.
fn tavern(state: &GameState, player: u8) -> ([Option<HeroTypeId>; 2], [bool; 2]) {
    let part = state.player_tavern(PlayerId::new(player)).unwrap();
    (part.slots, part.returning)
}

/// The type of the hero `hero`.
fn type_of(state: &GameState, hero: u16) -> HeroTypeId {
    state.hero(HeroId::new(hero)).unwrap().hero_type
}

/// The hero type of code `code`.
fn code(code: u8) -> HeroTypeId {
    HeroTypeId::from_index(usize::from(code)).unwrap()
}

/// The types the map allows that no hero is placed as ([`FREE`]).
fn free() -> Vec<HeroTypeId> {
    FREE.iter().map(|&c| code(c)).collect()
}

// covers: TWN-SET-005, TWN-TAV-001, TWN-TAV-002, ADV-VIS-007, ADV-VIS-008
#[test]
fn the_taverns_map_is_committed_as_written() {
    let committed = committed();
    let setup = import(&committed, &FAITHFUL).unwrap();
    setup.validate(&FAITHFUL).unwrap();
    let state = GameState::new(&FAITHFUL, &setup, 1).unwrap();
    let adventure = state.adventure();
    // The objects typed: the map Tavern and the Den, the golems.
    let kinds: Vec<_> = adventure
        .map()
        .objects()
        .iter()
        .filter_map(|o| {
            let ty = FAITHFUL.adventure().object_type(o.object_type?)?;
            match ty.kind {
                ObjectKind::Tavern | ObjectKind::DenOfThieves | ObjectKind::Monster { .. } => {
                    Some((ty.kind.clone(), o.anchor))
                }
                _ => None,
            }
        })
        .collect();
    assert_eq!(kinds.len(), 3, "{kinds:?}");
    assert!(matches!(kinds[0], (ObjectKind::Tavern, a) if a == at(MAP_TAVERN)));
    assert!(matches!(kinds[1], (ObjectKind::DenOfThieves, a) if a == at(DEN)));
    // The heroes placed, each its type taken by its owner.
    for (id, &(tile, owner, hero_type)) in HEROES.iter().enumerate() {
        let hero = adventure
            .hero(HeroId::new(u16::try_from(id).unwrap()))
            .unwrap();
        assert_eq!((hero.pos, hero.owner.raw()), (at(tile), owner));
        let status = state.hero_status(code(hero_type));
        assert_eq!(status, Some(HeroStatus::Taken(owner)));
    }
    // Every other type the map does not allow is out; the free ones fill the four slots.
    let types = FAITHFUL.core().hero_types().len();
    let placed: Vec<HeroTypeId> = HEROES.iter().map(|&(_, _, t)| code(t)).collect();
    for index in 0..types {
        let ty = HeroTypeId::from_index(index).unwrap();
        let status = state.hero_status(ty).unwrap();
        if !placed.contains(&ty) && !free().contains(&ty) {
            assert_eq!(status, HeroStatus::Out, "{ty:?}");
        }
    }
    let offered: Vec<HeroTypeId> = (0..2).flat_map(|p| tavern(&state, p).0).flatten().collect();
    assert_eq!(offered.len(), 4, "{offered:?}");
    assert!(offered.iter().all(|t| free().contains(t)));
    for ty in &offered {
        assert_eq!(state.hero_status(*ty), Some(HeroStatus::Offered));
    }
    // Each player's Tavern: red's Bastion with the Brotherhood.
    let part = FAITHFUL.towns();
    for (id, keys) in [
        (0, &["tavern", "brotherhood_of_the_sword"][..]),
        (1, &["tavern"]),
    ] {
        let record = state.towns().get(TownId::new(id)).unwrap();
        let faction = part.town(record.faction()).unwrap();
        for key in keys {
            let building = faction.building_by_key(key).unwrap();
            assert!(record.built().contains(building), "{id}: {key}");
        }
    }
}

/// The heroes `events` tell `player` its slots now hold (`TavernOffered`), by slot.
fn offered(events: &[GameEvent], player: u8) -> Vec<(u8, Option<HeroId>)> {
    events
        .iter()
        .filter_map(|e| match e {
            GameEvent::TavernOffered {
                player: p,
                slot,
                hero,
            } if p.raw() == player => Some((*slot, hero.as_ref().map(|h| h.id))),
            _ => None,
        })
        .collect()
}

/// `player`'s tavern hero in `slot` as its owner sees it: its record and army.
fn seen(state: &GameState, player: u8, slot: usize) -> Option<(HeroId, Vec<ArmyStack>)> {
    let view = observe(state, &FAITHFUL, Viewer::Player(PlayerId::new(player)));
    let hero = view.tavern.get(slot).cloned().flatten()?;
    Some((hero.hero.id, hero.army))
}

/// Five Royal Griffins in the first slot: every red hero's army as placed.
fn griffins() -> Vec<ArmyStack> {
    let creature = FAITHFUL
        .core()
        .creature_by_key("base:royal_griffin")
        .unwrap();
    vec![ArmyStack {
        slot: 0,
        creature,
        count: 5,
    }]
}

/// One creature of the first starting stack of `hero_type`: a retreated hero's army back in its
/// owner's tavern (TWN-TAV-006).
fn lone(hero_type: HeroTypeId) -> Vec<ArmyStack> {
    let first = &FAITHFUL.core().hero_type(hero_type).unwrap().army[0];
    vec![ArmyStack {
        slot: 0,
        creature: first.creature,
        count: 1,
    }]
}

/// Every artifact `hero` holds, worn or in its backpack, in artifact order.
fn artifacts(state: &GameState, hero: u16) -> Vec<u32> {
    let record = state.hero(HeroId::new(hero)).unwrap();
    let mut all: Vec<u32> = record
        .worn
        .iter()
        .flatten()
        .chain(&record.backpack)
        .map(|a| u32::from(a.artifact.raw()))
        .collect();
    all.sort_unstable();
    all
}

/// Whether `hero` stands on the map at `tile`.
fn stands(state: &GameState, hero: u16, tile: (u8, u8)) -> bool {
    state.adventure().hero(HeroId::new(hero)).map(|h| h.pos) == Some(at(tile))
}

/// The hero `events` tell `player` was hired, and where.
fn hired(events: &[GameEvent], player: u8) -> Option<(HeroId, Option<u8>, Pos)> {
    events.iter().find_map(|e| match *e {
        GameEvent::HeroHired {
            player: p,
            hero,
            slot,
            pos,
            ..
        } if p.raw() == player => Some((hero, slot, pos)),
        _ => None,
    })
}

/// Red's and blue's gold.
fn gold(state: &GameState) -> [u32; 2] {
    [0, 1].map(|p| state.treasury(PlayerId::new(p)).unwrap()[6])
}

// covers: TWN-TAV-001, TWN-TAV-002, GAM-CAL-002
#[test]
fn four_weeks_of_fills() {
    // The golden replay `taverns-weeks-s1`: 28 days of turn ends. At the start and at each week's
    // start after it (days 8, 15, 22 and 29) both players' slots are full of the free types,
    // none twice, slot 0 of the player's faction's classes (red's a cleric of the two free, blue's
    // the ranger or the druid), each told to its owner; every other free type stays free; the
    // weeks draw anew.
    let mut state = game(1);
    let mut weeks = vec![[tavern(&state, 0).0, tavern(&state, 1).0]];
    for day in 1..=28 {
        step(&mut state, 0, GameCommand::EndTurn);
        let events = step(&mut state, 1, GameCommand::EndTurn);
        if day % 7 == 0 {
            for player in 0..2 {
                let told: Vec<u8> = offered(&events, player).iter().map(|&(s, _)| s).collect();
                assert_eq!(told, [0, 1], "day {}", day + 1);
            }
            weeks.push([tavern(&state, 0).0, tavern(&state, 1).0]);
        }
    }
    assert_eq!(weeks.len(), 5);
    for week in &weeks {
        let types: Vec<HeroTypeId> = week.iter().flatten().map(|t| t.unwrap()).collect();
        assert_eq!(types.len(), 4);
        for (i, ty) in types.iter().enumerate() {
            assert!(free().contains(ty) && !types[..i].contains(ty), "{week:?}");
        }
        assert!([code(8), code(9)].contains(&types[0]), "{week:?}");
        assert!([code(18), code(24)].contains(&types[2]), "{week:?}");
    }
    for ty in free() {
        let offered = weeks[4].iter().flatten().any(|t| *t == Some(ty));
        let status = if offered {
            HeroStatus::Offered
        } else {
            HeroStatus::Free
        };
        assert_eq!(state.hero_status(ty), Some(status));
    }
    assert!(weeks.windows(2).any(|w| w[0] != w[1]));
}

// covers: TWN-TAV-003, TWN-TAV-004, GAM-DEC-010, ADV-VIS-007
#[test]
fn hires_at_the_town_and_the_map_tavern() {
    // The golden replay `taverns-hires-s1`. Day 1, red: the Bastion's hire of slot 0 onto its
    // entrance, the slot refilled from the two types left free; Valeska's step toward the map
    // Tavern's tile opens the tavern decision and leaves her beside it; the hire of slot 1 places
    // its hero on the Tavern's tile, pays the price, refills the slot from the last free type and
    // closes the decision. Blue: the Glade's hire of slot 0 empties it (no type is free). Day 2,
    // red: the hero hired at the Tavern walks off its tile; Valeska's second visit opens the
    // decision again (a hire listed), closed.
    let mut state = game(1);
    let (bastion, glade) = (TownId::new(0), TownId::new(1));
    let first = tavern(&state, 0).0;
    let hire = |town, slot| GameCommand::Town(TownCommand::Hire { town, slot });
    let events = step(&mut state, 0, hire(bastion, 0));
    let (hero, slot, pos) = hired(&events, 0).unwrap();
    assert_eq!((slot, pos), (Some(0), entrance(TOWNS[0])));
    assert_eq!(state.hero(hero).unwrap().hero_type, first[0].unwrap());
    assert!(matches!(offered(&events, 0)[..], [(0, Some(_))]));
    let events = step(&mut state, 0, to(VALESKA, MAP_TAVERN_TILE));
    assert!(stands(&state, VALESKA, HEROES[usize::from(VALESKA)].0));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TavernOpened { .. }))
    );
    let visit = *state.tavern(PlayerId::new(0)).unwrap();
    assert_eq!(
        (visit.anchor, visit.entrance),
        (at(MAP_TAVERN), at(MAP_TAVERN_TILE))
    );
    let answer = GameCommand::Answer(Answer::Tavern(TavernAnswer::Hire { slot: 1 }));
    let events = step(&mut state, 0, answer);
    let (at_tavern, slot, pos) = hired(&events, 0).unwrap();
    assert_eq!(
        (at_tavern, slot, pos),
        (HeroId::new(9), Some(1), at(MAP_TAVERN_TILE))
    );
    assert_eq!(state.hero(at_tavern).unwrap().hero_type, first[1].unwrap());
    assert!(matches!(offered(&events, 0)[..], [(1, Some(_))]));
    assert!(state.tavern(PlayerId::new(0)).is_none());
    assert_eq!(gold(&state), [15_000, 20_000]);
    step(&mut state, 0, GameCommand::EndTurn);
    let events = step(&mut state, 1, hire(glade, 0));
    assert_eq!(offered(&events, 1), [(0, None)]);
    assert_eq!(tavern(&state, 1).0[0], None);
    step(&mut state, 1, GameCommand::EndTurn);
    step(&mut state, 0, to(at_tavern.raw(), (10, 14)));
    assert!(stands(&state, at_tavern.raw(), (10, 14)));
    step(&mut state, 0, to(VALESKA, MAP_TAVERN_TILE));
    assert!(state.tavern(PlayerId::new(0)).is_some());
    let legal = h3_game::legal_commands(&state, &FAITHFUL, Actor::Player(PlayerId::new(0)));
    let close = GameCommand::Answer(Answer::Tavern(TavernAnswer::Close));
    assert_eq!(legal.last(), Some(&close));
    assert!(legal.len() > 1, "{legal:?}");
    step(&mut state, 0, close);
    assert!(state.tavern(PlayerId::new(0)).is_none());
    step(&mut state, 0, GameCommand::EndTurn);
}

// covers: TWN-TAV-006, TWN-TAV-005, GAM-BAT-003
#[test]
fn retreats_in_the_owners_turn_and_anothers() {
    // The golden replay `taverns-retreats-s1`. Day 1, red: Orrin attacks the golems and retreats
    // at his first turn; he comes back into red's tavern at once (slot 1 drawn, TD-19) with one
    // creature of his type's first starting stack, marked as returning, the mark cleared at red's turn's end. Day 6, blue: Mephala
    // attacks Sorsha, who retreats in blue's turn (slot 0, marked; the mark cleared at day 7's
    // start). Day 7, blue: Mephala attacks Cuthbert, who retreats (slot 0 drawn again, Sorsha's
    // type freed) and stays offered through day 8's release (TD-18), slot 1 refilled.
    let mut state = game(1);
    step(&mut state, 0, to(ORRIN, GOLEMS));
    assert!(state.battle(PlayerId::new(0)).is_some());
    let events = step(&mut state, 0, battle(Command::Retreat));
    assert!(state.adventure().hero(HeroId::new(ORRIN)).is_none());
    assert_eq!(offered(&events, 0), [(1, Some(HeroId::new(ORRIN)))]);
    assert_eq!(tavern(&state, 0).1, [false, true]);
    let orrin = lone(type_of(&state, ORRIN));
    assert_eq!(seen(&state, 0, 1), Some((HeroId::new(ORRIN), orrin)));
    step(&mut state, 0, GameCommand::EndTurn);
    assert_eq!(tavern(&state, 0).1, [false, false]);
    step(&mut state, 1, GameCommand::EndTurn);
    days(&mut state, 0, 4);
    step(&mut state, 0, GameCommand::EndTurn);
    let mephala = HEROES[usize::from(MEPHALA)].0;
    step(&mut state, 1, to(MEPHALA, HEROES[usize::from(SORSHA)].0));
    let events = step(&mut state, 0, battle(Command::Retreat));
    assert_eq!(offered(&events, 0), [(0, Some(HeroId::new(SORSHA)))]);
    assert_eq!(tavern(&state, 0).1, [true, false]);
    assert!(stands(&state, MEPHALA, mephala));
    step(&mut state, 1, GameCommand::EndTurn);
    step(&mut state, 0, GameCommand::EndTurn);
    assert_eq!(tavern(&state, 0).1, [false, false]);
    step(&mut state, 1, to(MEPHALA, HEROES[usize::from(CUTHBERT)].0));
    let events = step(&mut state, 0, battle(Command::Retreat));
    assert_eq!(offered(&events, 0), [(0, Some(HeroId::new(CUTHBERT)))]);
    assert_eq!(
        state.hero_status(type_of(&state, SORSHA)),
        Some(HeroStatus::Free)
    );
    let events = step(&mut state, 1, GameCommand::EndTurn);
    assert_eq!(offered(&events, 0), [(1, Some(HeroId::new(ORRIN)))]);
    assert_eq!(tavern(&state, 0).0[0], Some(type_of(&state, CUTHBERT)));
    let cuthbert = lone(type_of(&state, CUTHBERT));
    assert_eq!(seen(&state, 0, 0), Some((HeroId::new(CUTHBERT), cuthbert)));
}

// covers: TWN-TAV-006, GAM-BAT-003, BAT-FLE-002
#[test]
fn surrenders_in_the_owners_turn_and_anothers() {
    // The golden replay `taverns-surrenders-s1`. Day 1, red: Edric attacks Ufretin and
    // surrenders at his first turn, paying 600 gold to blue; he comes back into red's tavern at
    // once (slot 1) with his army, marked. Blue: Mephala attacks Sorsha, who surrenders in blue's
    // turn, 600 more (slot 0, marked after red's turn ended). Day 2, red: the Bastion hires Sorsha
    // back: she stands on its entrance with her army as it left.
    let mut state = game(1);
    step(&mut state, 0, to(EDRIC, HEROES[usize::from(UFRETIN)].0));
    let events = step(&mut state, 0, battle(Command::Surrender));
    assert_eq!(offered(&events, 0), [(1, Some(HeroId::new(EDRIC)))]);
    assert_eq!(gold(&state), [19_400, 20_600]);
    assert_eq!(tavern(&state, 0).1, [false, true]);
    assert_eq!(seen(&state, 0, 1), Some((HeroId::new(EDRIC), griffins())));
    step(&mut state, 0, GameCommand::EndTurn);
    step(&mut state, 1, to(MEPHALA, HEROES[usize::from(SORSHA)].0));
    let events = step(&mut state, 0, battle(Command::Surrender));
    assert_eq!(offered(&events, 0), [(0, Some(HeroId::new(SORSHA)))]);
    assert_eq!(gold(&state), [18_800, 21_200]);
    assert_eq!(tavern(&state, 0).1, [true, false]);
    step(&mut state, 1, GameCommand::EndTurn);
    let hire = GameCommand::Town(TownCommand::Hire {
        town: TownId::new(0),
        slot: 0,
    });
    let events = step(&mut state, 0, hire);
    let (hero, _, pos) = hired(&events, 0).unwrap();
    assert_eq!((hero, pos), (HeroId::new(SORSHA), entrance(TOWNS[0])));
    let army = &state.adventure().hero(hero).unwrap().army;
    assert_eq!(army, &griffins());
}

// covers: GAM-MAN-003, TWN-TAV-002
#[test]
fn a_dismissed_type_is_drawn_again() {
    // The golden replay `taverns-dismiss-s1`. Day 1, red dismisses Valeska (the Charm of Mana in
    // her backpack): her type is free, her record kept. Week 2's fill on day 8 draws her into
    // red's slot 0 (few free types), and red hires her at the Bastion: she comes back with the
    // Charm (T-14; the fill equips her backpack, TWN-TAV-002).
    let mut state = game(1);
    let valeska = type_of(&state, VALESKA);
    let held = artifacts(&state, VALESKA);
    assert_eq!(held, [u32::from(CHARM_OF_MANA)]);
    let dismiss = Manage::Dismiss {
        hero: HeroId::new(VALESKA),
    };
    step(&mut state, 0, GameCommand::Manage(dismiss));
    assert!(state.adventure().hero(HeroId::new(VALESKA)).is_none());
    assert_eq!(state.hero_status(valeska), Some(HeroStatus::Free));
    assert_eq!(state.hero_record(valeska), Some(HeroId::new(VALESKA)));
    days(&mut state, 0, 6);
    step(&mut state, 0, GameCommand::EndTurn);
    let events = step(&mut state, 1, GameCommand::EndTurn);
    assert!(offered(&events, 0).contains(&(0, Some(HeroId::new(VALESKA)))));
    assert_eq!(tavern(&state, 0).0[0], Some(valeska));
    let hire = GameCommand::Town(TownCommand::Hire {
        town: TownId::new(0),
        slot: 0,
    });
    let events = step(&mut state, 0, hire);
    assert_eq!(
        hired(&events, 0).map(|(h, _, _)| h),
        Some(HeroId::new(VALESKA))
    );
    assert_eq!(artifacts(&state, VALESKA), held);
}

// covers: TWN-TAV-009, ADV-VIS-008
#[test]
fn a_den_shows_the_guild() {
    // The golden replay `taverns-den-s1`. Day 1, red ends its turn; blue's Ryland enters the Den
    // of Thieves: blue is told the Thieves' Guild at everything (`ThievesGuildSeen`, then each
    // part), Ryland on the Den's tile.
    let mut state = game(1);
    step(&mut state, 0, GameCommand::EndTurn);
    let events = step(&mut state, 1, to(RYLAND, DEN));
    assert!(stands(&state, RYLAND, DEN));
    let levels: Vec<u8> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ThievesGuildSeen { player, level } if player.raw() == 1 => Some(level),
            _ => None,
        })
        .collect();
    assert_eq!(levels, [5], "{events:?}");
    let parts = events
        .iter()
        .filter(|e| matches!(e, GameEvent::GuildPartSeen { player, .. } if player.raw() == 1))
        .count();
    assert!(parts > 0);
}
