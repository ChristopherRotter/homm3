//! Tests of the towns' income in the day's advance (E09 T8c1, the town epic's design note §16):
//! each town a player owns pays it from day 2 its hall's gold, a Grail building's and its silo's
//! resources (TWN-INC-001, TWN-INC-002), beside its mines and heroes, one `ResourceGained` per
//! resource; a neutral town pays nobody; a town captured today pays its new owner the next day,
//! its old owner no more.

use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, Holds, ObjectSetup, Pos, TileMask};
use h3_content::ResourceId;
use h3_contracts::{HeroId, PlayerId};
use h3_towns::{BuildingsSetup, SlotSet, TownFaction};

use super::{CAPITOL_SLOT, GRAIL_SLOT, SILO_SLOT, faction, towns_rules, towns_setup};
use crate::{
    Actor, GameCommand, GameEvent, GameRuleset, GameSetup, GameState, Viewer, apply, observe,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(0);

/// The custom set of `slots` (and the Village Hall, always).
fn custom(slots: &[u8]) -> BuildingsSetup {
    BuildingsSetup::Custom {
        built: slots
            .iter()
            .fold(SlotSet::EMPTY, |set, &slot| set.with(slot)),
        forbidden: SlotSet::EMPTY,
    }
}

/// `player`'s command applied; its events.
fn play(
    rules: &GameRuleset,
    game: &mut GameState,
    player: PlayerId,
    command: GameCommand,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    apply(game, rules, Actor::Player(player), command, &mut events).unwrap();
    events
}

/// Red's and blue's turns ended: the next day's events.
fn end_day(rules: &GameRuleset, game: &mut GameState) -> Vec<GameEvent> {
    let mut events = play(rules, game, RED, GameCommand::EndTurn);
    events.extend(play(rules, game, BLUE, GameCommand::EndTurn));
    events
}

/// The gains of `events`, by player, resource and amount.
fn gains(events: &[GameEvent]) -> Vec<(PlayerId, ResourceId, u32)> {
    events
        .iter()
        .filter_map(|event| match *event {
            GameEvent::ResourceGained {
                player,
                resource,
                amount,
            } => Some((player, resource, amount)),
            _ => None,
        })
        .collect()
}

/// Each player's gold.
fn gold(game: &GameState) -> [u32; 2] {
    [RED, BLUE].map(|player| game.treasury(player).unwrap()[GOLD.index()])
}

// covers: TWN-INC-001, TWN-INC-002
#[test]
fn each_town_pays_its_owner_its_hall_its_grail_and_its_silo_from_day_2() {
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    // Red's town: the Capitol (hall 4), the Grail building and the silo (its part's `t:gold` 1);
    // blue's: the standard set, its Village Hall's.
    setup.towns[0].buildings = custom(&[CAPITOL_SLOT, GRAIL_SLOT, SILO_SLOT]);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let start = gold(&game);
    let events = end_day(&rules, &mut game);
    // The heroes and the map earn nothing else: the towns' income alone, one gain per player
    // and resource, in player order, after the day's start.
    assert_eq!(gains(&events), [(RED, GOLD, 9001), (BLUE, GOLD, 500)]);
    let day = events
        .iter()
        .position(|e| matches!(e, GameEvent::DayStarted { day: 2 }))
        .unwrap();
    assert!(
        events[..day]
            .iter()
            .all(|e| !matches!(e, GameEvent::ResourceGained { .. }))
    );
    assert_eq!(gold(&game), [start[0] + 9001, start[1] + 500]);
    // And every day after.
    let events = end_day(&rules, &mut game);
    assert_eq!(gains(&events), [(RED, GOLD, 9001), (BLUE, GOLD, 500)]);
}

// covers: TWN-INC-001
#[test]
fn a_towns_income_joins_its_owners_mines_in_one_gain_per_resource() {
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    // Red's gold mine (1 000 a day) on (2, 3).
    setup.map.objects.push(ObjectSetup {
        scenery: Default::default(),
        anchor: Pos::new(2, 3, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:mine"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: Some(RED),
    });
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    assert_eq!(
        gains(&end_day(&rules, &mut game)),
        [(RED, GOLD, 1500), (BLUE, GOLD, 500)]
    );
}

// covers: TWN-INC-001
#[test]
fn several_capitols_at_the_start_each_pay_theirs() {
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    setup.map.objects[1].owner = Some(RED);
    setup.towns[1] = super::town(TownFaction::Fixed(faction(&rules, "t:town")));
    for town in &mut setup.towns {
        town.buildings = custom(&[CAPITOL_SLOT]);
    }
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    // The limit comes with a capture only (TWN-CAP-002): both stay Capitols and pay 4 000.
    assert_eq!(gains(&end_day(&rules, &mut game)), [(RED, GOLD, 8000)]);
}

/// [`towns_setup`] with town 1 neutral, of `t:town`, holding an empty garrison (undefended), and
/// red's hero beside its entrance.
fn neutral_town(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.heroes[0].pos = Pos::new(3, 1, 0);
    setup.map.objects[1].owner = None;
    setup.towns[1] = super::town(TownFaction::Fixed(faction(rules, "t:town")));
    setup.towns[1].garrison = Some(Vec::new());
    setup
}

// covers: TWN-INC-001
#[test]
fn a_neutral_town_pays_nobody_and_a_captured_one_its_new_owner_from_the_next_day() {
    let rules = towns_rules();
    let mut game = GameState::new(&rules, &neutral_town(&rules), 7).unwrap();
    assert_eq!(gains(&end_day(&rules, &mut game)), [(RED, GOLD, 500)]);
    // Red captures it on day 2: no gain at the capture, its gold from day 3.
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: [Pos::new(3, 0, 0)].to_vec(),
    });
    let events = play(&rules, &mut game, RED, step);
    assert_eq!(game.adventure().owner(1), Some(RED));
    assert!(gains(&events).is_empty());
    assert_eq!(gains(&end_day(&rules, &mut game)), [(RED, GOLD, 1000)]);
}

// covers: TWN-INC-001, TWN-VEW-003
#[test]
fn a_captured_town_pays_and_shows_its_new_owner_only() {
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    setup.heroes[0].pos = Pos::new(3, 1, 0);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    // Red takes blue's undefended town on day 1: blue gets nothing the next day.
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: [Pos::new(3, 0, 0)].to_vec(),
    });
    play(&rules, &mut game, RED, step);
    assert_eq!(game.adventure().owner(1), Some(RED));
    assert_eq!(gains(&end_day(&rules, &mut game)), [(RED, GOLD, 1000)]);
    // Its view changes hands: the owner's facts to red, the public ones to blue.
    let view = |player| observe(&game, &rules, Viewer::Player(player)).towns[1].clone();
    let (red, blue) = (view(RED), view(BLUE));
    assert_eq!((red.owner, blue.owner), (Some(RED), Some(RED)));
    assert_eq!(red.garrison.army, Some(Vec::new()));
    assert!(red.owner_facts.is_some());
    assert_eq!((blue.garrison.army, blue.owner_facts), (None, None));
}
