//! Tests of the town screen's opening (E09 T49a2, design note `E09-design.md` §90; TWN-GLD-006):
//! `TownCommand::Open` gives the Mana Vortex to the garrison hero, then the visitor, then teaches
//! the visitor, then the garrison hero (with or without a visitor, decision 178); each refusal
//! changes nothing; an opening that changes nothing is accepted and changes nothing; the legal
//! commands list it only where it would change something, after the town's hires and before its
//! hero moves. On [`super::learn`]'s game: red's T0 on (1, 1) and blue's T1 on (3, 0), each built by
//! its setup's slots only; the part's Vortex factor 2.

use alloc::vec::Vec;

use h3_adventure::Pos;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{TownCommand, TownCommandError, TownHeroChange};

use super::command::{BLUE, RED, T0, T1, run};
use super::learn::{
    GUILD_SLOT, RED_HERO, TAVERN_SLOT, VORTEX_SLOT, game, level_1, rules, setup, taught,
    town_command,
};
use crate::{Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState};

fn open(town: TownId) -> GameCommand {
    town_command(TownCommand::Open { town })
}

fn enter() -> GameCommand {
    town_command(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    })
}

/// Twice `hero`'s maximum spell points (the part's Vortex factor 2; HER-MAN-001).
fn twice(rules: &GameRuleset, game: &GameState, hero: HeroId) -> u16 {
    2 * h3_heroes::max_mana(game.hero(hero).unwrap(), rules.core())
}

/// The spell points `events` set, as (owner, hero, points) in order.
fn given(events: &[GameEvent]) -> Vec<(PlayerId, HeroId, u16)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::HeroChanged {
                player,
                hero,
                change: HeroChange::SetSpellPoints(points),
            } => Some((player, hero, points)),
            _ => None,
        })
        .collect()
}

/// Whether `player`'s legal commands list `town`'s opening.
fn listed(rules: &GameRuleset, game: &GameState, player: PlayerId, town: TownId) -> bool {
    crate::legal_commands(game, rules, Actor::Player(player)).contains(&open(town))
}

/// The opening with nothing to give: accepted, no event, the state's hash unchanged.
fn changes_nothing(rules: &GameRuleset, game: &mut GameState, town: TownId) {
    let before = game.state_hash();
    assert_eq!(run(rules, game, RED, open(town)).unwrap(), []);
    assert_eq!(game.state_hash(), before);
}

// covers: TWN-GLD-006, TWN-GLD-009, TWN-GLD-005, TWN-VEW-004
#[test]
fn an_opening_gives_the_vortex_to_the_garrison_hero_first_then_teaches_the_visitor_then_it() {
    let rules = rules();
    let mut game = game(
        &rules,
        &setup(&rules, &[GUILD_SLOT, TAVERN_SLOT, VORTEX_SLOT]),
    );
    let [slot, _] = level_1(&game, T0);
    // Red's hero into the garrison, a hired visitor (the hire takes the week's Vortex and
    // teaches both); then the week's flag set again, both drained and knowing no spell.
    run(&rules, &mut game, RED, enter()).unwrap();
    run(
        &rules,
        &mut game,
        RED,
        town_command(TownCommand::Hire { town: T0, slot: 0 }),
    )
    .unwrap();
    let hired = game.adventure().town_visitor(T0).unwrap();
    game.towns_mut().start_week();
    for hero in [RED_HERO, hired] {
        game.set_mana(hero, 0);
        game.set_spells(hero, Vec::new());
    }
    let garrison = twice(&rules, &game, RED_HERO);
    assert!(listed(&rules, &game, RED, T0));
    let events = run(&rules, &mut game, RED, open(T0)).unwrap();
    // The Vortex to the garrison hero alone, then the cleared flag told to red, then the
    // visitor's learning, then the garrison hero's.
    assert_eq!(given(&events), [(RED, RED_HERO, garrison)]);
    assert_eq!(taught(&events), [(RED, hired, slot), (RED, RED_HERO, slot)]);
    let set = events
        .iter()
        .position(|e| matches!(e, GameEvent::HeroChanged { .. }))
        .unwrap();
    let told = events
        .iter()
        .position(|e| matches!(e, GameEvent::TownChanged { .. }))
        .unwrap();
    assert!(set < told, "{events:?}");
    assert!(
        matches!(
            events[told],
            GameEvent::TownChanged { player: RED, town: T0, buildings: Some(b), .. } if !b.vortex
        ),
        "{events:?}"
    );
    // The town's view told whole (its parts after its `TownChanged`), then the learning last.
    let learns = |e: &GameEvent| {
        matches!(
            e,
            GameEvent::HeroChanged {
                change: HeroChange::LearnSpell(_),
                ..
            }
        )
    };
    let learned = events.iter().position(learns).unwrap();
    assert!(told < learned, "{events:?}");
    assert!(events[learned..].iter().all(learns), "{events:?}");
    assert_eq!(game.hero(RED_HERO).unwrap().mana, garrison);
    assert_eq!(game.hero(hired).unwrap().mana, 0);
    assert!(!game.towns().get(T0).unwrap().vortex());
    // Again: nothing left to give, nothing listed, the opening still accepted.
    assert!(!listed(&rules, &game, RED, T0));
    changes_nothing(&rules, &mut game, T0);
}

// covers: TWN-GLD-006, TWN-GLD-005
#[test]
fn an_opening_teaches_a_garrison_hero_alone_and_a_town_without_a_guild_nothing() {
    let rules = rules();
    let mut game = game(&rules, &setup(&rules, &[GUILD_SLOT]));
    let [slot, _] = level_1(&game, T0);
    // Into the garrison: no visitor stands, nothing taught by the move (TWN-GLD-005).
    assert_eq!(taught(&run(&rules, &mut game, RED, enter()).unwrap()), []);
    assert!(game.adventure().town_visitor(T0).is_none());
    // The opening teaches the garrison hero with no visitor there (decision 178).
    let events = run(&rules, &mut game, RED, open(T0)).unwrap();
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
    assert_eq!(events.len(), 1, "{events:?}");
    // Without a guild nothing is taught, nothing listed: the opening is accepted all the same.
    let mut bare = self::game(&rules, &setup(&rules, &[TAVERN_SLOT]));
    assert!(!listed(&rules, &bare, RED, T0));
    changes_nothing(&rules, &mut bare, T0);
}

// covers: TWN-GLD-006
#[test]
fn each_refusal_of_an_opening_changes_nothing() {
    let rules = rules();
    let mut game = game(&rules, &setup(&rules, &[GUILD_SLOT]));
    let town = |error| Err(GameCommandError::Town(error));
    // A town the store lacks; another player's town (TD-11's rule).
    assert_eq!(
        run(&rules, &mut game, RED, open(TownId::new(9))),
        town(TownCommandError::UnknownTown)
    );
    assert_eq!(
        run(&rules, &mut game, RED, open(T1)),
        town(TownCommandError::NotOwner)
    );
    // Another player's turn (`check_ready`).
    assert!(matches!(
        run(&rules, &mut game, BLUE, open(T1)),
        Err(GameCommandError::NotYourTurn(_))
    ));
    // Red's own opening still teaches its visitor afterwards.
    let [slot, _] = level_1(&game, T0);
    let events = run(&rules, &mut game, RED, open(T0)).unwrap();
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
}

// covers: TWN-GLD-006, TWN-GLD-009
#[test]
fn the_legal_commands_list_an_opening_only_where_it_would_change_something() {
    let rules = rules();
    let mut setup = setup(&rules, &[GUILD_SLOT, VORTEX_SLOT]);
    let mut game = game(&rules, &setup);
    // A placed visitor that has learned nothing: listed after the town's builds and
    // recruitments, before its hero moves.
    let legal = crate::legal_commands(&game, &rules, Actor::Player(RED));
    let at = legal.iter().position(|c| *c == open(T0)).unwrap();
    let town = |c: &GameCommand, t: fn(&TownCommand) -> bool| matches!(c, GameCommand::Town(command) if t(command));
    assert!(
        legal[..at]
            .iter()
            .all(|c| !town(c, |t| matches!(t, TownCommand::Heroes { .. })))
    );
    assert!(legal[at + 1..].iter().all(|c| !town(c, |t| matches!(
        t,
        TownCommand::Build { .. } | TownCommand::Recruit { .. } | TownCommand::Hire { .. }
    ))));
    assert!(
        legal[at + 1..]
            .iter()
            .any(|c| town(c, |t| matches!(t, TownCommand::Heroes { .. })))
    );
    // Nothing in blue's town, which no hero stands in, nor in another player's turn.
    assert!(!listed(&rules, &game, RED, T1));
    assert!(!listed(&rules, &game, BLUE, T1));
    // Once opened: the spells learned, the Vortex taken, nothing listed.
    run(&rules, &mut game, RED, open(T0)).unwrap();
    assert!(!listed(&rules, &game, RED, T0));
    // The week's flag again: listed while the hero is below twice its maximum, not at it.
    game.towns_mut().start_week();
    game.set_mana(RED_HERO, 0);
    assert!(listed(&rules, &game, RED, T0));
    let full = twice(&rules, &game, RED_HERO);
    game.set_mana(RED_HERO, full);
    assert!(!listed(&rules, &game, RED, T0));
    game.set_mana(RED_HERO, full - 1);
    assert!(listed(&rules, &game, RED, T0));
    // Each listed opening is one `apply` accepts; a hero beside the town lists none.
    run(&rules, &mut game, RED, open(T0)).unwrap();
    setup.heroes[0].pos = Pos::new(0, 1, 0);
    let away = self::game(&rules, &setup);
    assert!(!listed(&rules, &away, RED, T0));
}
