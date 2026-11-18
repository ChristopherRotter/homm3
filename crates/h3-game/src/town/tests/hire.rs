//! Tests of the town's hire (E09 T38a1, design note `E09-design.md` §64, §65, §75): the owner's
//! `TownCommand::Hire` in its turn at a town with a standing Tavern, each refusal changing nothing;
//! the hero on the entrance as the visitor with its away record (TWN-TAV-003); the slot refilled
//! from the hirer's stream with one creature of the type's first stack (TWN-TAV-004); the events
//! and who sees them. On the town tests' game: red's T0 on (1, 1), blue's T1 on (3, 0), each with
//! the standard set and a Tavern; every hero type allowed.

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureEventKind, HeroOnMap, Pos, Viewers};
use h3_content::{
    ArtifactTypeId, HeroTypeId, ProfileDef, ResourceId, Ruleset, SpellId, StartingStackDef,
};
use h3_contracts::{ArmyStack, PlayerId, TownId};
use h3_heroes::{ArtifactInstance, max_mana};
use h3_towns::{
    BuildingDef, BuildingRoles, BuildingsSetup, SlotSet, TownCommand, TownCommandError, TownPack,
};

use super::command::{BLUE, RED, T0, T1, run};
use super::{rules_of, town_pack, towns_core_pack, towns_setup};
use crate::tests::key;
use crate::town::pool::HeroStatus;
use crate::town::tavern::{draw, views};
use crate::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameSetup, GameState, HiredPart,
    NoopSink, Viewer, apply, observe, project,
};

const GOLD: ResourceId = ResourceId::from_raw(0);
/// T0's entrance: its one tile.
const ENTRANCE: Pos = Pos::new(1, 1, 0);
/// The map slot of the test Tavern (the h3m's), which the standard set builds (TWN-SET-003).
const TAVERN_SLOT: u8 = 5;

/// The town tests' rules with a Tavern in `t:town`'s record and its tavern weights the knight 3
/// and the cleric 2 (as the tavern tests'), the part edited by `edit`; every hero type has a
/// starting army (a type without one 2 to 3 imps), so that a replacement's is one creature.
fn rules_with(edit: impl FnOnce(&mut TownPack)) -> GameRuleset {
    rules_in(("t:faithful", false), edit)
}

/// [`rules_with`] under `profile`, the test pack's `t:faithful` (no quirk flag on) or (E09 T38b1)
/// `t:leave` (`TWN-QUIRK-12`'s flag on: a leave's points over the cleared army); with `slow`
/// every hero type's first starting stack 2 to 3 gogs (speed 2: one gog's land points are below
/// the test part's most).
fn rules_in((profile, slow): (&str, bool), edit: impl FnOnce(&mut TownPack)) -> GameRuleset {
    let mut towns = town_pack(false);
    towns.towns[0].buildings.push(BuildingDef {
        key: "tavern".to_string(),
        name: "Tavern".into(),
        slot: Some(TAVERN_SLOT),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles: BuildingRoles {
            tavern: true,
            ..BuildingRoles::default()
        },
    });
    towns.towns[0].tavern = vec![(key("t:knight"), 3), (key("t:cleric"), 2)];
    edit(&mut towns);
    let mut core = towns_core_pack(false);
    for ty in core
        .heroes
        .iter_mut()
        .filter(|ty| slow || ty.army.is_empty())
    {
        let creature = key(if slow { "t:gog" } else { "t:imp" });
        ty.army = vec![StartingStackDef {
            creature,
            count: [2, 3],
        }];
    }
    core.profiles.push(ProfileDef {
        key: key("t:leave"),
        quirks: vec![(
            "quirk.town.leave_points_over_cleared_army".to_string(),
            true,
        )],
    });
    let core = Ruleset::compile(&[core], profile).unwrap();
    rules_of(core, &towns, false, 4)
}

fn rules() -> GameRuleset {
    rules_with(|_| {})
}

/// The game of `setup` with red's gold `gold`.
fn game(rules: &GameRuleset, setup: &GameSetup, gold: u32) -> GameState {
    let mut game = GameState::new(rules, setup, 7).unwrap();
    game.set_resource(RED, GOLD, gold);
    game
}

fn hire(town: TownId, slot: u8) -> GameCommand {
    GameCommand::Town(TownCommand::Hire { town, slot })
}

/// The players that have explored [`ENTRANCE`], red among them: whom a hire there is told.
fn seers(game: &GameState) -> Viewers {
    [RED, BLUE]
        .into_iter()
        .filter(|&p| p == RED || game.adventure().explored(p).unwrap().get(ENTRANCE))
        .fold(Viewers::NONE, Viewers::with)
}

/// `player`'s two slots' types.
fn slots(game: &GameState, player: PlayerId) -> [Option<HeroTypeId>; 2] {
    game.pool().tavern(player).unwrap().slots
}

fn gold(game: &GameState) -> u32 {
    game.treasury(RED).unwrap()[usize::from(GOLD.raw())]
}

// covers: TWN-TAV-003
#[test]
fn a_hire_places_the_slots_hero_on_the_entrance_with_its_away_record() {
    let rules = rules();
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let [Some(hired), Some(other)] = slots(&game, RED) else {
        panic!("red's two slots are filled at the start");
    };
    let id = game.pool().record(hired).unwrap();
    let away = game.pool().away(hired).unwrap().clone();
    // The owner's turn start set its points (TWN-TAV-005).
    assert!(away.movement > 0 && !away.stale);
    let details = game.hero(id).unwrap().clone();
    let events = run(&rules, &mut game, RED, hire(T0, 0)).unwrap();
    assert_eq!(gold(&game), 2500);
    assert_eq!(
        events[..2],
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 2500,
            },
            GameEvent::HeroHired {
                player: RED,
                hero: id,
                hero_type: hired,
                slot: Some(0),
                pos: ENTRANCE,
                seen_by: seers(&game),
            },
        ]
    );
    // The hero on the entrance as the visitor, with the away record as it was; its details as
    // they were (the guild's spells are increment 5's, TWN-STUB-15).
    let placed = HeroOnMap {
        id,
        owner: RED,
        pos: ENTRANCE,
        army: away.army.clone(),
        formation: away.formation,
        tactics: away.tactics,
        movement: away.movement,
        boat: None,
        max_movement: away.max_movement,
    };
    assert_eq!(game.adventure().hero(id), Some(&placed));
    assert_eq!(game.adventure().town_visitor(T0), Some(id));
    assert_eq!(game.hero(id), Some(&details));
    // Its type taken by red's colour, the colour's bit in its mask; its away record handed over.
    let pool = game.pool();
    assert_eq!(pool.status(hired), Some(HeroStatus::Taken(0)));
    assert_eq!(pool.mask(hired).map(|mask| mask & 1), Some(1));
    assert!(pool.away(hired).is_none());
    // The other slot keeps its hero; the hired one's slot holds its replacement.
    let now = slots(&game, RED);
    assert_eq!(now[1], Some(other));
    assert!(now[0].is_some_and(|t| t != hired && t != other));
    // The owner sees the hero among its own with its details, its points told last.
    let seen = observe(&game, &rules, Viewer::Player(RED));
    let info = seen.heroes.iter().find(|h| h.id == id).unwrap();
    assert_eq!(info.details.as_deref(), Some(&details));
    let moved = AdventureEventKind::MovementChanged {
        hero: id,
        movement: away.movement,
        max_movement: away.max_movement,
    };
    assert!(
        matches!(events.last(), Some(GameEvent::Adventure(e)) if e.kind == moved),
        "{events:?}"
    );
}

// covers: TWN-TAV-004
#[test]
fn the_slot_is_refilled_from_the_hirers_stream_with_one_creature_of_the_first_stack() {
    let rules = rules();
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let [Some(hired), Some(other)] = slots(&game, RED) else {
        panic!("red's two slots are filled at the start");
    };
    // The replacement's draw: the pool after the hire, the other slot's class avoided, any
    // faction, from red's stream as the hire's checks left it.
    let mut expected = game.clone();
    let colour = expected.colour(RED).unwrap();
    let faction = expected.pool().tavern(RED).unwrap().faction;
    Arc::make_mut(expected.taverns().pool).hire((RED, 0), hired, colour);
    let avoid = rules.core().hero_type(other).map(|ty| ty.class);
    let mut stream = game.stream(RED);
    let drawn = draw(
        expected.pool(),
        (rules.core(), rules.towns()),
        (colour, faction),
        (avoid, false),
        &mut stream,
    );
    let records = game.heroes().len();
    run(&rules, &mut game, RED, hire(T0, 0)).unwrap();
    let new = slots(&game, RED)[0];
    assert_eq!(new, drawn);
    let new = new.unwrap();
    let pool = game.pool();
    assert_eq!(pool.status(new), Some(HeroStatus::Offered));
    // A type never drawn before gets its record now, from red's stream (not the taverns').
    let id = pool.record(new).unwrap();
    assert_eq!(usize::from(id.raw()), records);
    assert_ne!(game.stream(RED), stream);
    // One creature of its type's first starting stack; its spell points full; since E09 T38b1 a
    // new record's points the land maximum of that one creature (design note §67 point 5), stale
    // for red's next turn start.
    let first = rules.core().hero_type(new).unwrap().army[0].creature;
    let away = pool.away(new).unwrap();
    let one = ArmyStack {
        slot: 0,
        creature: first,
        count: 1,
    };
    assert_eq!(away.army, [one]);
    let details = game.hero(id).unwrap();
    assert_eq!(details.mana, max_mana(details, rules.core()));
    let (core, part) = (rules.core(), rules.adventure());
    let points = h3_adventure::day_points(core, part, Some(details), &[one], 0);
    assert!(points > 0);
    assert_eq!(
        (away.movement, away.max_movement, away.stale, away.full),
        (points, points, true, true)
    );
    // The hired hero holds the entrance now: the other slot waits for it to leave.
    game.set_resource(RED, GOLD, 5000);
    assert_eq!(
        refused(&rules, &mut game, RED, hire(T0, 1)),
        GameCommandError::Town(TownCommandError::EntranceHeld)
    );
}

// covers: TWN-TAV-003, TWN-TAV-004
#[test]
fn the_owner_is_told_the_hire_its_parts_and_the_slots_new_view() {
    let rules = rules();
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let id = game.pool().record(slots(&game, RED)[0].unwrap()).unwrap();
    // A backpack past the view's five places and a learned spell for the parts to tell.
    let scroll = ArtifactTypeId::from_index(0)
        .map(ArtifactInstance::from)
        .unwrap();
    game.set_backpack(id, vec![scroll; 7]);
    let spell = SpellId::from_raw(0);
    game.set_spells(id, vec![spell]);
    let before = observe(&game, &rules, Viewer::Player(RED)).tavern;
    let events = run(&rules, &mut game, RED, hire(T0, 0)).unwrap();
    let details = game.hero(id).unwrap();
    let parts: Vec<HiredPart> = events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::HiredPartSeen { player, hero, part } => {
                assert_eq!((player, hero), (RED, id));
                Some(part)
            }
            _ => None,
        })
        .collect();
    assert_eq!(parts, HiredPart::of(details).collect::<Vec<_>>());
    assert_eq!(
        parts[0],
        HiredPart::Details {
            offered: details.offered,
            positions: u8::try_from(details.worn.len()).unwrap(),
        }
    );
    assert_eq!(
        parts[1..],
        [
            HiredPart::Carried(scroll),
            HiredPart::Carried(scroll),
            HiredPart::Spell(spell),
        ]
    );
    // Then the slot's new view, as the week tells a slot (after the parts).
    let offered = events
        .iter()
        .position(|e| matches!(e, GameEvent::TavernOffered { slot: 0, .. }))
        .unwrap();
    let hired = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroHired { .. }))
        .unwrap();
    assert!(hired < offered);
    let new = views(&game, RED);
    assert_ne!(new[0], before[0]);
    assert_eq!(new[1], before[1]);
    assert_eq!(
        events[offered],
        GameEvent::TavernOffered {
            player: RED,
            slot: 0,
            hero: new[0].as_ref().map(|view| view.hero),
        }
    );
    // The towns' views do not move (the visitor is the adventure's hero view).
    let towns: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::TownChanged { .. }))
        .collect();
    assert!(towns.is_empty(), "{towns:?}");
    // Blue sees the hire (it sees the tile) without the slot, and nothing else of red's.
    let blue: Vec<GameEvent> = events
        .iter()
        .filter_map(|e| project(e, Viewer::Player(BLUE)))
        .collect();
    assert!(seers(&game).contains(BLUE));
    assert_eq!(
        blue,
        [GameEvent::HeroHired {
            player: RED,
            hero: id,
            hero_type: details.hero_type,
            slot: None,
            pos: ENTRANCE,
            seen_by: Viewers::NONE.with(BLUE),
        }]
    );
    let info = observe(&game, &rules, Viewer::Player(BLUE));
    let info = info.heroes.iter().find(|h| h.id == id).unwrap();
    assert_eq!(
        (info.hero_type, info.details.is_none()),
        (details.hero_type, true)
    );
}

/// `player`'s `command` refused with `error`, the state unchanged ([`run`]).
fn refused(
    rules: &GameRuleset,
    game: &mut GameState,
    player: PlayerId,
    command: GameCommand,
) -> GameCommandError {
    let before = game.clone();
    let error = run(rules, game, player, command).unwrap_err();
    assert_eq!(game.pool(), before.pool());
    assert_eq!(game.adventure(), before.adventure());
    error
}

// covers: TWN-TAV-003
#[test]
fn each_refusal_changes_nothing() {
    let town = GameCommandError::Town;
    let rules = rules();
    let setup = towns_setup(&rules);
    let mut game = game(&rules, &setup, 5000);
    let refusals = [
        (hire(TownId::new(9), 0), town(TownCommandError::UnknownTown)),
        (hire(T1, 0), town(TownCommandError::NotOwner)),
        (hire(T0, 2), town(TownCommandError::UnknownSlot)),
    ];
    for (command, error) in refusals {
        assert_eq!(refused(&rules, &mut game, RED, command), error);
    }
    // Not in another player's turn (GAM-DEC-001).
    assert_eq!(
        refused(&rules, &mut game, BLUE, hire(T1, 0)),
        GameCommandError::NotYourTurn(BLUE)
    );
    // A town without a standing Tavern.
    let mut bare = setup.clone();
    bare.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    let mut bare = self::game(&rules, &bare, 5000);
    assert_eq!(
        refused(&rules, &mut bare, RED, hire(T0, 0)),
        town(TownCommandError::NoTavern)
    );
    // An empty slot: one type allowed fills slot 0 alone.
    let mut one = setup.clone();
    one.allowed_heroes.truncate(1);
    let mut one = self::game(&rules, &one, 5000);
    assert_eq!(slots(&one, RED)[1], None);
    assert_eq!(
        refused(&rules, &mut one, RED, hire(T0, 1)),
        town(TownCommandError::EmptySlot)
    );
    // The most heroes on the map: red's one hero fills a most of 1 (the room before the entrance).
    let most = rules_with(|towns| towns.params.as_mut().unwrap().most_heroes = 1);
    let mut full = self::game(&most, &towns_setup(&most), 5000);
    let mut visited = towns_setup(&most);
    visited.heroes[0].pos = ENTRANCE;
    let mut visited = self::game(&most, &visited, 5000);
    for game in [&mut full, &mut visited] {
        assert_eq!(
            refused(&most, game, RED, hire(T0, 0)),
            town(TownCommandError::TooManyHeroes)
        );
    }
    // A hero on the entrance.
    let mut visited = setup.clone();
    visited.heroes[0].pos = ENTRANCE;
    let mut visited = self::game(&rules, &visited, 5000);
    assert_eq!(
        refused(&rules, &mut visited, RED, hire(T0, 0)),
        town(TownCommandError::EntranceHeld)
    );
    // The treasury below the price, after every town check.
    let mut poor = self::game(&rules, &setup, 2499);
    assert_eq!(
        refused(&rules, &mut poor, RED, hire(T0, 0)),
        GameCommandError::CannotPay
    );
    let error = TownCommandError::NoTavern.to_string();
    assert!(!error.is_empty());
}

// covers: TWN-TAV-004
#[test]
fn no_type_free_leaves_the_slot_empty() {
    let rules = rules();
    let mut setup = towns_setup(&rules);
    // Red's random starting hero and the two slots take three types: none is left free.
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let taken: Vec<HeroTypeId> = game
        .adventure()
        .heroes()
        .iter()
        .map(|h| game.hero(h.id).unwrap().hero_type)
        .chain(slots(&game, RED).into_iter().flatten())
        .chain(slots(&game, BLUE).into_iter().flatten())
        .collect();
    setup.allowed_heroes.retain(|t| taken.contains(t));
    game = self::game(&rules, &setup, 5000);
    let [Some(_), Some(other)] = slots(&game, RED) else {
        panic!("red's two slots are filled at the start");
    };
    let free = (0..rules.core().hero_types().len())
        .filter_map(HeroTypeId::from_index)
        .any(|t| game.pool().is_free(t));
    assert!(!free, "every allowed type is taken or offered");
    let mut quiet = game.clone();
    run(&rules, &mut game, RED, hire(T0, 0)).unwrap();
    assert_eq!(slots(&game, RED), [None, Some(other)]);
    // Applied with a sink that keeps nothing, the same state (nothing is read for the events).
    apply(
        &mut quiet,
        &rules,
        Actor::Player(RED),
        hire(T0, 0),
        &mut NoopSink,
    )
    .unwrap();
    assert_eq!(quiet.state_hash(), game.state_hash());
}

/// `player`'s listed hires (E09 T38a2) in their order.
fn listed_hires(rules: &GameRuleset, game: &GameState, player: PlayerId) -> Vec<GameCommand> {
    crate::legal_commands(game, rules, Actor::Player(player))
        .into_iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::Hire { .. })))
        .collect()
}

// covers: TWN-TAV-003
#[test]
fn the_legal_commands_list_each_hire_that_passes_after_the_towns_recruitments() {
    let rules = rules();
    let setup = towns_setup(&rules);
    let game = game(&rules, &setup, 5000);
    let legal = crate::legal_commands(&game, &rules, Actor::Player(RED));
    let hires = listed_hires(&rules, &game, RED);
    assert_eq!(hires, vec![hire(T0, 0), hire(T0, 1)]);
    // After the town's recruitments, before its hero moves (design §64, §76).
    let at = |wanted: fn(&TownCommand) -> bool| {
        legal
            .iter()
            .rposition(|c| matches!(c, GameCommand::Town(t) if wanted(t)))
    };
    let first_hire = legal.iter().position(|c| *c == hire(T0, 0)).unwrap();
    let recruit = at(|t| matches!(t, TownCommand::Recruit { .. }));
    assert!(recruit.is_none_or(|r| r < first_hire), "{legal:?}");
    let moves = legal
        .iter()
        .position(|c| matches!(c, GameCommand::Town(TownCommand::Heroes { .. })));
    assert!(moves.is_none_or(|m| m > first_hire + 1), "{legal:?}");
    // Each listed hire is one `apply` accepts.
    for command in hires {
        let mut probe = game.clone();
        run(&rules, &mut probe, RED, command).unwrap();
    }
    // None in another player's turn, none for its town.
    assert!(listed_hires(&rules, &game, BLUE).is_empty());
    // None where a check refuses: no Tavern, below the price, no room, the entrance held; an
    // empty slot's alone.
    let mut bare = setup.clone();
    bare.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    let mut visited = setup.clone();
    visited.heroes[0].pos = ENTRANCE;
    let most = rules_with(|towns| towns.params.as_mut().unwrap().most_heroes = 1);
    let unlisted = [
        (self::game(&rules, &bare, 5000), &rules),
        (self::game(&rules, &setup, 2499), &rules),
        (self::game(&rules, &visited, 5000), &rules),
        (self::game(&most, &towns_setup(&most), 5000), &most),
    ];
    for (game, rules) in &unlisted {
        assert!(listed_hires(rules, game, RED).is_empty());
    }
    let mut one = game.clone();
    let pool = Arc::make_mut(one.taverns().pool);
    pool.tavern_mut(RED).unwrap().slots[0] = None;
    assert_eq!(listed_hires(&rules, &one, RED), vec![hire(T0, 1)]);
}

mod dismiss;
mod leave;
