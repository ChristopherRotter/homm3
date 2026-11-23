//! Tests of a spell book's purchase at a town's Mage Guild (E09 T49b, design note `E09-design.md`
//! §91; TWN-GLD-007): `TownCommand::BuySpellBook` pays the part's price, gives the town's visitor,
//! else its garrison hero, the ruleset's spell book, then teaches the visitor, then the garrison
//! hero (TWN-GLD-005); each refusal changes nothing; the legal commands list it where `apply`
//! accepts it, after the town's opening and before its hero moves. On [`super::learn`]'s game:
//! red's T0 on (1, 1) and blue's T1 on (3, 0), each built by its setup's slots only; red holds
//! 10 000 gold; the part's book 500 gold.

use alloc::vec;

use h3_adventure::Pos;
use h3_content::{HeroEffect, ResourceId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{TownCommand, TownCommandError, TownHeroChange};

use super::command::{BLUE, RED, T0, T1, run};
use super::learn::{
    GUILD_SLOT, RED_HERO, TAVERN_SLOT, VORTEX_SLOT, game, level_1, rules, rules_edited, setup,
    taught, town_command,
};
use crate::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameSetup, GameState,
    HeroArtifactsSetup,
};

/// The core's gold (the town tests' first resource).
const GOLD: ResourceId = ResourceId::from_raw(0);

fn book(town: TownId) -> GameCommand {
    town_command(TownCommand::BuySpellBook { town })
}

fn heroes(change: TownHeroChange) -> GameCommand {
    town_command(TownCommand::Heroes { town: T0, change })
}

/// [`setup`] with red's hero (0) wearing nothing: no spell book.
fn bookless(rules: &GameRuleset, slots: &[u8]) -> GameSetup {
    let mut setup = setup(rules, slots);
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn: h3_heroes::wearing(rules.core(), &[]),
        backpack: vec![],
    });
    setup
}

fn wears_book(rules: &GameRuleset, game: &GameState, hero: HeroId) -> bool {
    h3_heroes::wears_spell_book(game.hero(hero).unwrap(), rules.core())
}

fn gold(game: &GameState, player: PlayerId) -> u32 {
    game.treasury(player).unwrap()[GOLD.index()]
}

/// Whether `player`'s legal commands list `town`'s purchase.
fn listed(rules: &GameRuleset, game: &GameState, player: PlayerId, town: TownId) -> bool {
    crate::legal_commands(game, rules, Actor::Player(player)).contains(&book(town))
}

/// The purchase refused with `error`, the state's hash unchanged.
fn refused(
    rules: &GameRuleset,
    game: &mut GameState,
    (player, town): (PlayerId, TownId),
    error: GameCommandError,
) {
    let before = game.state_hash();
    assert_eq!(run(rules, game, player, book(town)), Err(error));
    assert_eq!(game.state_hash(), before);
}

// covers: TWN-GLD-007, TWN-GLD-005
#[test]
fn a_purchase_gives_the_visitor_the_book_for_the_price_then_teaches_it_at_once() {
    let rules = rules();
    let mut game = game(&rules, &bookless(&rules, &[GUILD_SLOT]));
    let [slot, _] = level_1(&game, T0);
    let spell_book = h3_heroes::spell_book(rules.core()).unwrap();
    assert!(!wears_book(&rules, &game, RED_HERO));
    let events = run(&rules, &mut game, RED, book(T0)).unwrap();
    // The price paid, the book given to red's visitor, then its learning.
    assert_eq!(
        events,
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 500,
            },
            GameEvent::HeroChanged {
                player: RED,
                hero: RED_HERO,
                change: HeroChange::AddArtifact(spell_book.into()),
            },
            GameEvent::HeroChanged {
                player: RED,
                hero: RED_HERO,
                change: HeroChange::LearnSpell(slot),
            },
        ]
    );
    assert!(wears_book(&rules, &game, RED_HERO));
    assert_eq!(gold(&game, RED), 9_500);
    // Again: the hero has one.
    let has = GameCommandError::Town(TownCommandError::HasSpellBook);
    refused(&rules, &mut game, (RED, T0), has);
}

// covers: TWN-GLD-007, TWN-GLD-005
#[test]
fn the_visitor_is_offered_the_book_before_the_garrison_hero() {
    let rules = rules();
    let mut game = game(&rules, &bookless(&rules, &[GUILD_SLOT, TAVERN_SLOT]));
    let [slot, _] = level_1(&game, T0);
    // Red's bookless hero alone in the garrison: it is offered the book.
    run(&rules, &mut game, RED, heroes(TownHeroChange::Enter)).unwrap();
    assert!(game.adventure().town_visitor(T0).is_none());
    assert!(listed(&rules, &game, RED, T0));
    let mut alone = game.clone();
    let events = run(&rules, &mut alone, RED, book(T0)).unwrap();
    assert!(wears_book(&rules, &alone, RED_HERO));
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
    // A hired visitor with its type's book stands before it: the book is the visitor's to
    // have, so none is sold while it has one, even with the garrison hero without.
    run(
        &rules,
        &mut game,
        RED,
        town_command(TownCommand::Hire { town: T0, slot: 0 }),
    )
    .unwrap();
    let hired = game.adventure().town_visitor(T0).unwrap();
    assert!(wears_book(&rules, &game, hired));
    assert!(!listed(&rules, &game, RED, T0));
    let has = GameCommandError::Town(TownCommandError::HasSpellBook);
    refused(&rules, &mut game, (RED, T0), has);
    // Swapped: the bookless hero is the visitor, the book its; then both learn, the visitor
    // first (the hired hero knows the slot already from its hire).
    run(&rules, &mut game, RED, heroes(TownHeroChange::Swap)).unwrap();
    let events = run(&rules, &mut game, RED, book(T0)).unwrap();
    assert!(wears_book(&rules, &game, RED_HERO));
    let added = events.iter().filter(|e| {
        matches!(e, GameEvent::HeroChanged { hero, change: HeroChange::AddArtifact(_), .. }
            if *hero == RED_HERO)
    });
    assert_eq!(added.count(), 1, "{events:?}");
    assert_eq!(taught(&events), [(RED, RED_HERO, slot)]);
}

// covers: TWN-GLD-007
#[test]
fn each_refusal_of_a_purchase_changes_nothing() {
    let rules = rules();
    let town = GameCommandError::Town;
    let mut game = game(&rules, &bookless(&rules, &[GUILD_SLOT]));
    // A town the store lacks; another player's town (TD-11's rule); another player's turn.
    let unknown = town(TownCommandError::UnknownTown);
    refused(&rules, &mut game, (RED, TownId::new(9)), unknown);
    refused(
        &rules,
        &mut game,
        (RED, T1),
        town(TownCommandError::NotOwner),
    );
    let before = game.state_hash();
    assert!(matches!(
        run(&rules, &mut game, BLUE, book(T1)),
        Err(GameCommandError::NotYourTurn(_))
    ));
    assert_eq!(game.state_hash(), before);
    // Below the price: the original's GENRLTXT 214.
    game.set_resource(RED, GOLD, 499);
    refused(&rules, &mut game, (RED, T0), GameCommandError::CannotPay);
    // No guild; no hero in the town; a hero wearing its type's book.
    let mut bare = self::game(&rules, &bookless(&rules, &[TAVERN_SLOT]));
    refused(
        &rules,
        &mut bare,
        (RED, T0),
        town(TownCommandError::NoGuild),
    );
    let mut away = bookless(&rules, &[GUILD_SLOT]);
    away.heroes[0].pos = Pos::new(0, 1, 0);
    let mut away = self::game(&rules, &away);
    refused(&rules, &mut away, (RED, T0), town(TownCommandError::NoHero));
    let mut booked = self::game(&rules, &setup(&rules, &[GUILD_SLOT]));
    let has = town(TownCommandError::HasSpellBook);
    refused(&rules, &mut booked, (RED, T0), has);
    // A ruleset without a spell book sells none.
    let plain = rules_edited(|core| {
        for artifact in &mut core.artifacts {
            artifact.effects.retain(|e| *e != HeroEffect::SpellBook);
        }
    });
    let mut none = self::game(&plain, &setup(&plain, &[GUILD_SLOT]));
    assert!(!listed(&plain, &none, RED, T0));
    refused(
        &plain,
        &mut none,
        (RED, T0),
        town(TownCommandError::NoGuild),
    );
    // Red's own purchase still goes through afterwards.
    game.set_resource(RED, GOLD, 500);
    run(&rules, &mut game, RED, book(T0)).unwrap();
    assert_eq!(gold(&game, RED), 0);
}

// covers: TWN-GLD-007
#[test]
fn the_legal_commands_list_a_purchase_where_it_is_accepted_after_the_opening() {
    let rules = rules();
    let mut game = game(&rules, &bookless(&rules, &[GUILD_SLOT, VORTEX_SLOT]));
    game.set_mana(RED_HERO, 0);
    // The Vortex makes the opening listed: the purchase comes right after it, before the
    // town's hero moves.
    let legal = crate::legal_commands(&game, &rules, Actor::Player(RED));
    let open = legal
        .iter()
        .position(|c| *c == town_command(TownCommand::Open { town: T0 }))
        .unwrap();
    assert_eq!(legal.get(open + 1), Some(&book(T0)));
    let moves = |c: &GameCommand| matches!(c, GameCommand::Town(TownCommand::Heroes { .. }));
    assert!(legal[..open].iter().all(|c| !moves(c)));
    assert!(legal[open + 2..].iter().any(moves));
    // Nothing in blue's town, nor in its turn; nothing below the price.
    assert!(!listed(&rules, &game, RED, T1));
    assert!(!listed(&rules, &game, BLUE, T1));
    let mut poor = game.clone();
    poor.set_resource(RED, GOLD, 499);
    assert!(!listed(&rules, &poor, RED, T0));
    // Every listed purchase is one `apply` accepts; then the hero has a book: none listed.
    run(&rules, &mut game, RED, book(T0)).unwrap();
    assert!(!listed(&rules, &game, RED, T0));
    // No guild, no hero, a hero with its book: none listed.
    let bare = self::game(&rules, &bookless(&rules, &[VORTEX_SLOT]));
    assert!(!listed(&rules, &bare, RED, T0));
    let mut away = bookless(&rules, &[GUILD_SLOT]);
    away.heroes[0].pos = Pos::new(0, 1, 0);
    assert!(!listed(&rules, &self::game(&rules, &away), RED, T0));
    let booked = self::game(&rules, &setup(&rules, &[GUILD_SLOT]));
    assert!(!listed(&rules, &booked, RED, T0));
}
