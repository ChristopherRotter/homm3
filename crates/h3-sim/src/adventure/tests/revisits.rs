//! The env's revisit (E08 T105b2, action encoding v8; ADV-TEL-008, ADR-0017 point 8): on the duel
//! with a Shipyard of each player's ([`shipyard_setup`]), red's first hero entered on its own, the
//! entry `REVISIT_BASE + s` of its slot is legal with tile 0 alone, decodes to its `Revisit` and
//! back, and applies without a step; a tile, an empty slot and the neutral seat decode to nothing.

use h3_adventure::{AdventureEvent, AdventureEventKind};
use h3_game::GameEvent;

use super::decisions::legal_entries;
use super::shipyards::{shipyard_setup, shipyards};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

// covers: ADV-TEL-008
#[test]
fn a_revisit_is_its_hero_slots_entry_with_tile_0_alone() {
    let rules = &*FAITHFUL;
    let setup = shipyard_setup(rules, 1);
    let mut env = AdventureEnv::new(rules.clone(), Arc::clone(&setup), 1, 0).unwrap();
    let yard = shipyards(&setup)[0];
    let hero = hero_in_slot(env.state(), RED, 0).unwrap();
    assert!(!legal_entries(&env, RED).contains(&REVISIT_BASE));
    // Red's first hero enters its Shipyard; each offer declined (the monster guarding the tile
    // asks first, then the Shipyard's visit, held after it, offers its boat).
    let tile = tile_index(env.state(), yard).unwrap();
    env.step(RED, Action::move_to(0, tile)).unwrap();
    while env.state().has_decision(RED) {
        env.step(RED, Action::decline()).unwrap();
    }
    assert_eq!(env.state().adventure().hero(hero).unwrap().pos, yard);
    let legal = legal_entries(&env, RED);
    assert!(legal.contains(&REVISIT_BASE), "{legal:?}");
    assert!(!legal.contains(&(REVISIT_BASE + 1)), "{legal:?}");
    let mut tiles = Vec::new();
    env.tile_mask(RED, REVISIT_BASE, &mut tiles);
    let masked: Vec<usize> = (0..tiles.len()).filter(|&t| tiles[t]).collect();
    assert_eq!(masked, [0]);
    env.tile_mask(RED, REVISIT_BASE + 1, &mut tiles);
    assert!(!tiles.contains(&true));
    let revisit = GameCommand::Adventure(AdventureCommand::Revisit { hero });
    let (state, game) = (env.state(), env.rules());
    assert_eq!(
        action::decode(state, game, RED, Action::revisit(0)),
        Some(revisit.clone())
    );
    assert_eq!(
        action::encode(state, game, RED, &revisit),
        Some(Action::revisit(0))
    );
    // Another tile, an empty slot (red has two heroes) and the neutral seat name no command; no
    // other seat names red's hero.
    for (seat, action) in [
        (Seat::Player(RED), Action::map(REVISIT_BASE, 1)),
        (Seat::Player(RED), Action::revisit(HERO_SLOTS - 1)),
        (Seat::Neutral, Action::revisit(0)),
    ] {
        assert_eq!(action::decode(state, game, seat, action), None, "{seat:?}");
    }
    for seat in [Seat::Player(BLUE), Seat::Neutral] {
        assert_eq!(
            action::encode(state, game, seat, &revisit),
            None,
            "{seat:?}"
        );
    }
    // Blue's own revisit is refused out of its turn; red's visits the Shipyard again, no step.
    assert_eq!(
        env.step(BLUE, Action::revisit(0)),
        Err(StepError::Rejected(GameCommandError::NotYourTurn(BLUE)))
    );
    let mut events = Vec::new();
    env.step_with(RED, Action::revisit(0), &mut events).unwrap();
    assert!(!events.is_empty());
    assert!(!events.iter().any(|e| matches!(
        e,
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroStepped { .. },
            ..
        })
    )));
    assert_eq!(env.state().adventure().hero(hero).unwrap().pos, yard);
}
