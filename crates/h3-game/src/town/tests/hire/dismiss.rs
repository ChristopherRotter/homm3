//! Tests of the dismissal (E09 T38b2, design note `E09-design.md` §69; GAM-MAN-003, TWN-TAV-006):
//! a player's hero on the map leaves the game at its command, its type free with its record and
//! every artifact, one creature, its spell points full, no draw, told as the adventure's removal;
//! each refusal (`NotYourHero`, `NotOnMap`, `InTown`, `LastHero`, `DecisionOpen`, outside the
//! turn) changing nothing, and the listing giving the dismissal exactly where it passes. On the
//! hire tests' game: red's hero 0 on (0, 0), blue's hero 1 on (2, 2), red's T0 on (1, 1).

use h3_adventure::AdventureEvent;
use h3_content::ArtifactTypeId;
use h3_contracts::HeroId;
use h3_heroes::ArtifactInstance;
use h3_towns::TownHeroChange;

use super::*;
use crate::{Manage, ManageError, legal_management};

const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);

fn dismiss(hero: HeroId) -> GameCommand {
    GameCommand::Manage(Manage::Dismiss { hero })
}

fn refusal(error: ManageError) -> Result<Vec<GameEvent>, GameCommandError> {
    Err(GameCommandError::Manage(error))
}

/// The players that have explored `pos`, `owner` among them: whom a hero's removal there is told
/// (ADV-FOG-004).
fn seers_of(game: &GameState, owner: PlayerId, pos: Pos) -> Viewers {
    [RED, BLUE]
        .into_iter()
        .filter(|&p| p == owner || game.adventure().explored(p).unwrap().get(pos))
        .fold(Viewers::NONE, Viewers::with)
}

/// Whether `player`'s listing for `hero` gives its dismissal.
fn listed(rules: &GameRuleset, game: &GameState, player: PlayerId, hero: HeroId) -> bool {
    legal_management(game, rules, Actor::Player(player), hero).contains(&Manage::Dismiss { hero })
}

// covers: GAM-MAN-003, TWN-TAV-006
#[test]
fn a_dismissed_hero_leaves_the_game_its_type_free_with_every_artifact() {
    let rules = rules_in(("t:faithful", true), |_| {});
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let hero_type = game.hero(RED_HERO).unwrap().hero_type;
    let scroll = ArtifactTypeId::from_index(0)
        .map(ArtifactInstance::from)
        .unwrap();
    game.set_backpack(RED_HERO, vec![scroll; 2]);
    game.set_mana(RED_HERO, 0);
    // Listed last of its management commands, as `apply` takes it.
    let listing = legal_management(&game, &rules, Actor::Player(RED), RED_HERO);
    assert_eq!(listing.last(), Some(&Manage::Dismiss { hero: RED_HERO }));
    let before = game.clone();
    let pos = before.adventure().hero(RED_HERO).unwrap().pos;
    let events = run(&rules, &mut game, RED, dismiss(RED_HERO)).unwrap();
    // Told as its removal, to its owner and the players that see its tile, nothing more (the
    // pool is hidden); no draw.
    let removed = AdventureEvent {
        kind: AdventureEventKind::HeroRemoved {
            hero: RED_HERO,
            pos,
        },
        seen_by: seers_of(&before, RED, pos),
    };
    assert_eq!(events, [GameEvent::Adventure(removed)]);
    assert!(game.adventure().hero(RED_HERO).is_none());
    assert_eq!(game.stream(RED), before.stream(RED));
    assert_eq!(slots(&game, RED), slots(&before, RED));
    // Its type free, its record kept with every artifact and its spell points full; its away
    // record one creature of its first starting stack.
    let pool = game.pool();
    assert_eq!(pool.status(hero_type), Some(HeroStatus::Free));
    assert_eq!(pool.record(hero_type), Some(RED_HERO));
    let details = game.hero(RED_HERO).unwrap();
    assert_eq!(details.backpack, [scroll; 2]);
    assert_eq!(details.mana, max_mana(details, rules.core()));
    let ty = rules.core().hero_type(hero_type).unwrap();
    let one = ArmyStack {
        slot: 0,
        creature: ty.army[0].creature,
        count: 1,
    };
    assert_eq!(pool.away(hero_type).unwrap().army, [one]);
    // Gone from its owner's view; red keeps its turn and its town, with no hero on the map.
    let seen = observe(&game, &rules, Viewer::Player(RED));
    assert!(seen.adventure.heroes.iter().all(|h| h.id != RED_HERO));
    assert!(game.adventure().heroes().iter().all(|h| h.owner != RED));
    assert_eq!(
        run(&rules, &mut game, RED, dismiss(RED_HERO)),
        refusal(ManageError::NotYourHero(RED_HERO))
    );
}

// covers: GAM-MAN-003
#[test]
fn a_dismissal_is_refused_for_a_hero_not_on_the_map_in_a_town_or_the_last() {
    let rules = rules_with(|_| {});
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    let tavern_hero = game.pool().record(slots(&game, RED)[0].unwrap()).unwrap();
    // Another player's hero, a tavern hero, a hero out of the game: not the player's.
    for hero in [BLUE_HERO, tavern_hero, HeroId::new(99)] {
        assert_eq!(
            run(&rules, &mut game, RED, dismiss(hero)),
            refusal(ManageError::NotYourHero(hero))
        );
        assert!(!listed(&rules, &game, RED, hero));
    }
    // Outside its turn.
    assert_eq!(
        run(&rules, &mut game, BLUE, dismiss(BLUE_HERO)),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    assert!(!listed(&rules, &game, BLUE, BLUE_HERO));
    // The hired hero on T0's entrance, its visitor (TD-22); in T0's garrison, off the map. Red's
    // other hero passes beside either.
    run(&rules, &mut game, RED, hire(T0, 0)).unwrap();
    assert_eq!(
        run(&rules, &mut game, RED, dismiss(tavern_hero)),
        refusal(ManageError::InTown)
    );
    assert!(!listed(&rules, &game, RED, tavern_hero));
    assert!(listed(&rules, &game, RED, RED_HERO));
    let enter = GameCommand::Town(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    });
    run(&rules, &mut game, RED, enter).unwrap();
    assert_eq!(
        run(&rules, &mut game, RED, dismiss(tavern_hero)),
        refusal(ManageError::NotOnMap)
    );
    assert!(!listed(&rules, &game, RED, tavern_hero));
    // Red's last hero on the map passes while red owns T0; blue's last does not without a town.
    assert!(listed(&rules, &game, RED, RED_HERO));
    let mut setup = towns_setup(&rules);
    setup.map.objects[1].owner = None;
    let mut townless = self::game(&rules, &setup, 5000);
    run(&rules, &mut townless, RED, GameCommand::EndTurn).unwrap();
    assert_eq!(
        run(&rules, &mut townless, BLUE, dismiss(BLUE_HERO)),
        refusal(ManageError::LastHero)
    );
    assert!(!listed(&rules, &townless, BLUE, BLUE_HERO));
    run(&rules, &mut game, RED, GameCommand::EndTurn).unwrap();
    assert!(listed(&rules, &game, BLUE, BLUE_HERO));
}

// covers: GAM-MAN-003, GAM-DEC-001, GAM-DEC-003
#[test]
fn a_dismissal_waits_for_an_open_meeting() {
    let rules = rules_with(|_| {});
    let mut game = game(&rules, &towns_setup(&rules), 5000);
    // Red's hero steps onto its hired hero on T0's entrance: a meeting holds both.
    let hired = game.pool().record(slots(&game, RED)[0].unwrap()).unwrap();
    run(&rules, &mut game, RED, hire(T0, 0)).unwrap();
    let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: [ENTRANCE].to_vec(),
    });
    run(&rules, &mut game, RED, step).unwrap();
    assert!(game.meeting(RED).is_some());
    for hero in [RED_HERO, hired] {
        assert_eq!(
            run(&rules, &mut game, RED, dismiss(hero)),
            Err(GameCommandError::DecisionOpen)
        );
        assert!(!listed(&rules, &game, RED, hero));
        assert!(!legal_management(&game, &rules, Actor::Player(RED), hero).is_empty());
    }
}
