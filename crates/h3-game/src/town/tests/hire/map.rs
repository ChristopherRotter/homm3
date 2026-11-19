//! Tests of the adventure map's Tavern and the Den of Thieves (E09 T39a, design note
//! `E09-design.md` §66, §74, §75; GAM-DEC-010, ADV-VIS-007, -008): a step toward the Tavern opens
//! its decision for the visitor's owner (`TavernOpened`, `DecisionView::Tavern`), answered by a
//! hire of a tavern hero placed on the Tavern's visitable tile, which closes it, or by `Close`;
//! each refusal typed, changing nothing and leaving it open; the listing and the default answer; a
//! hired hero on the tile hides it; the Den entered, its visit showing the visitor's owner the
//! Thieves' Guild at everything (E09 T39b2, design note §74, §75; `ThievesGuildSeen`, its parts).
//! On the hire tests' game: red's hero 0 on (0, 0), blue's hero 1 on (2, 2), red's T0 on (1, 1),
//! blue's T1 on (3, 0); the Tavern's one tile (0, 2), visited from (1, 2), the Den's (2, 0).

use h3_adventure::{
    AdventureCommand, Holds, ObjectKindDef, ObjectSetup, ObjectTypeDef, Scenery, TileMask, Visit,
};
use h3_contracts::HeroId;

use super::*;
use crate::town::thieves::thieves_guild_at;
use crate::{
    Answer, DEN_LEVEL, DecisionView, GuildPart, TavernAnswer, TavernError, TavernVisit,
    ThievesGuildView, default_answer, legal_commands, thieves_guild,
};

const RED_HERO: HeroId = HeroId::new(0);
/// The Tavern's one tile, its anchor and its visitable tile.
const TAVERN: Pos = Pos::new(0, 2, 0);
/// The tile beside it the visitor stands on, east of it (the Tavern is not visited from above).
const BESIDE: Pos = Pos::new(1, 2, 0);
/// The Den of Thieves' one tile.
const DEN: Pos = Pos::new(2, 0, 0);

/// The two adventure types the hire tests' rules add: the map's Tavern `t:tavern`, visited from a
/// neighbour, and the Den of Thieves `t:den`, entered; neither from above.
pub(super) fn inn_types() -> [ObjectTypeDef; 2] {
    let ty = |k: &str, visit, kind| ObjectTypeDef {
        key: key(k),
        name: "Inn".into(),
        visit,
        from_above: false,
        kind_mark: false,
        kind,
    };
    [
        ty("t:tavern", Visit::Neighbour, ObjectKindDef::Tavern),
        ty("t:den", Visit::Enter, ObjectKindDef::DenOfThieves),
    ]
}

/// An object of the type `k` on `at`, its one tile blocked and visitable.
fn inn(rules: &GameRuleset, k: &str, at: Pos) -> ObjectSetup {
    ObjectSetup {
        anchor: at,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        scenery: Scenery::Other,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// The hire tests' setup with the Tavern and the Den.
pub(super) fn inn_setup(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.map.objects.push(inn(rules, "t:tavern", TAVERN));
    setup.map.objects.push(inn(rules, "t:den", DEN));
    setup
}

fn step(path: &[Pos]) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: path.to_vec(),
    })
}

fn answer(answer: TavernAnswer) -> GameCommand {
    GameCommand::Answer(Answer::Tavern(answer))
}

/// The game of `setup` with red's gold `gold`, red's hero 0 having stepped toward the Tavern.
pub(super) fn visited(
    rules: &GameRuleset,
    setup: &GameSetup,
    gold: u32,
) -> (GameState, Vec<GameEvent>) {
    let mut game = game(rules, setup, gold);
    let path = [Pos::new(0, 1, 0), BESIDE, TAVERN];
    let events = run(rules, &mut game, RED, step(&path)).unwrap();
    (game, events)
}

/// The visit the decision holds.
fn visit(rules: &GameRuleset) -> TavernVisit {
    TavernVisit {
        hero: RED_HERO,
        object_type: rules.adventure().object_type_by_key("t:tavern").unwrap(),
        anchor: TAVERN,
        entrance: TAVERN,
    }
}

// covers: GAM-DEC-010, ADV-VIS-007
#[test]
fn a_step_toward_the_map_tavern_opens_its_decision_for_the_visitors_owner() {
    let rules = rules();
    let setup = inn_setup(&rules);
    let (game, events) = visited(&rules, &setup, 5000);
    // The hero stays beside it; the decision told to its owner alone, last.
    assert_eq!(game.adventure().hero(RED_HERO).map(|h| h.pos), Some(BESIDE));
    let visit = visit(&rules);
    let opened = GameEvent::TavernOpened {
        player: RED,
        hero: RED_HERO,
        object_type: visit.object_type,
        anchor: TAVERN,
    };
    assert!(events.contains(&opened), "{events:?}");
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    assert_eq!(game.tavern(RED), Some(&visit));
    assert_eq!(game.tavern(BLUE), None);
    assert_eq!(
        observe(&game, &rules, Viewer::Player(RED)).decision,
        Some(DecisionView::Tavern {
            hero: RED_HERO,
            object_type: visit.object_type,
            anchor: TAVERN,
        })
    );
    assert_eq!(observe(&game, &rules, Viewer::Player(BLUE)).decision, None);
    // Its answers alone: the two hires that pass, then `Close`, the default.
    assert_eq!(
        legal_commands(&game, &rules, Actor::Player(RED)),
        [
            answer(TavernAnswer::Hire { slot: 0 }),
            answer(TavernAnswer::Hire { slot: 1 }),
            answer(TavernAnswer::Close),
        ]
    );
    assert_eq!(
        default_answer(&game, &rules, RED),
        Some(Answer::Tavern(TavernAnswer::Close))
    );
    let mut ended = game.clone();
    assert_eq!(
        run(&rules, &mut ended, RED, GameCommand::EndTurn),
        Err(GameCommandError::DecisionOpen)
    );
    // Another kind's answer is refused; the decision is in the hash.
    let mut wrong = game.clone();
    assert_eq!(
        run(&rules, &mut wrong, RED, GameCommand::Answer(Answer::Close)),
        Err(GameCommandError::WrongAnswer)
    );
    let mut closed = game.clone();
    let events = run(&rules, &mut closed, RED, answer(TavernAnswer::Close)).unwrap();
    assert_eq!(events, [GameEvent::DecisionClosed { player: RED }]);
    assert_eq!(closed.tavern(RED), None);
    assert_ne!(closed.state_hash(), game.state_hash());
    // Closed, nothing else changed: the pool, the treasury, the map's heroes.
    assert_eq!(closed.pool(), game.pool());
    assert_eq!(gold(&closed), 5000);
    assert_eq!(closed.adventure().heroes(), game.adventure().heroes());
}

// covers: GAM-DEC-010, TWN-TAV-004
#[test]
fn a_map_taverns_hire_places_the_hero_on_its_tile_and_closes_it() {
    let rules = rules();
    let setup = inn_setup(&rules);
    let (mut game, _) = visited(&rules, &setup, 5000);
    let [_, Some(hired)] = slots(&game, RED) else {
        panic!("red's two slots are filled at the start");
    };
    let id = game.pool().record(hired).unwrap();
    let away = game.pool().away(hired).unwrap().clone();
    let details = game.hero(id).unwrap().clone();
    let events = run(
        &rules,
        &mut game,
        RED,
        answer(TavernAnswer::Hire { slot: 1 }),
    )
    .unwrap();
    assert_eq!(gold(&game), 2500);
    let seen_by = [RED, BLUE]
        .into_iter()
        .filter(|&p| p == RED || game.adventure().explored(p).unwrap().get(TAVERN))
        .fold(Viewers::NONE, Viewers::with);
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
                slot: Some(1),
                pos: TAVERN,
                seen_by,
            },
        ]
    );
    // Closed after the hire's events (one hire a visit).
    let closed = events
        .iter()
        .position(|e| *e == GameEvent::DecisionClosed { player: RED })
        .unwrap();
    assert!(closed > 1);
    assert_eq!(game.tavern(RED), None);
    assert!(!game.has_decision(RED));
    // The hero on the Tavern's tile with its away record and its details as they were (no guild
    // spells); its type taken, its slot refilled.
    let on_map = game.adventure().hero(id).unwrap();
    assert_eq!(
        (on_map.pos, &on_map.army, on_map.movement, on_map.boat),
        (TAVERN, &away.army, away.movement, None)
    );
    assert_eq!(game.hero(id), Some(&details));
    assert_eq!(game.pool().status(hired), Some(HeroStatus::Taken(0)));
    assert!(slots(&game, RED)[1].is_some_and(|t| t != hired));
    // A hero on a neighbour-visited object's tile revisits nothing (the original skips case 95
    // under its visitor).
    assert!(
        !legal_commands(&game, &rules, Actor::Player(RED)).contains(&GameCommand::Adventure(
            AdventureCommand::Revisit { hero: id }
        ))
    );
    // A second step toward the Tavern now meets the hired hero (a meeting), not the Tavern.
    let mut again = game.clone();
    let events = run(&rules, &mut again, RED, step(&[TAVERN])).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::MeetingOpened { .. })),
        "{events:?}"
    );
    assert_eq!(again.tavern(RED), None);
}

// covers: GAM-DEC-010
#[test]
fn each_refusal_of_a_map_taverns_hire_changes_nothing_and_leaves_it_open() {
    let refused = |error| Err(GameCommandError::Tavern(error));
    let rules = rules();
    let setup = inn_setup(&rules);
    let (mut game, _) = visited(&rules, &setup, 5000);
    assert_eq!(
        run(
            &rules,
            &mut game,
            RED,
            answer(TavernAnswer::Hire { slot: 2 })
        ),
        refused(TavernError::UnknownSlot)
    );
    assert!(game.tavern(RED).is_some());
    // An empty slot: two types allowed fill slot 0 alone here (red's random starting hero takes
    // the other).
    let mut one = setup.clone();
    one.allowed_heroes.truncate(2);
    let (mut one, _) = visited(&rules, &one, 5000);
    assert_eq!(slots(&one, RED)[1], None);
    assert_eq!(
        run(
            &rules,
            &mut one,
            RED,
            answer(TavernAnswer::Hire { slot: 1 })
        ),
        refused(TavernError::EmptySlot)
    );
    assert_eq!(
        legal_commands(&one, &rules, Actor::Player(RED)),
        [
            answer(TavernAnswer::Hire { slot: 0 }),
            answer(TavernAnswer::Close)
        ]
    );
    // The most heroes on the map: red's one fills a most of 1.
    let most = rules_with(|towns| towns.params.as_mut().unwrap().most_heroes = 1);
    let (mut full, _) = visited(&most, &inn_setup(&most), 5000);
    assert_eq!(
        run(
            &most,
            &mut full,
            RED,
            answer(TavernAnswer::Hire { slot: 0 })
        ),
        refused(TavernError::TooManyHeroes)
    );
    assert_eq!(
        legal_commands(&full, &most, Actor::Player(RED)),
        [answer(TavernAnswer::Close)]
    );
    // A hero on the Tavern's tile (VCMI's refusal; the original never reaches it): placed there
    // while the decision is open.
    let mut held = game.clone();
    let blue = held.adventure().hero(HeroId::new(1)).unwrap().clone();
    let (adventure, _) = held.adventure_with_heroes();
    adventure.place_hired(
        rules.core(),
        HeroOnMap {
            id: HeroId::new(9),
            pos: TAVERN,
            ..blue
        },
    );
    assert_eq!(
        run(
            &rules,
            &mut held,
            RED,
            answer(TavernAnswer::Hire { slot: 0 })
        ),
        refused(TavernError::Occupied)
    );
    assert_eq!(
        legal_commands(&held, &rules, Actor::Player(RED)),
        [answer(TavernAnswer::Close)]
    );
    // The treasury below the price, after the map's checks.
    let (mut poor, _) = visited(&rules, &setup, 2499);
    assert_eq!(
        run(
            &rules,
            &mut poor,
            RED,
            answer(TavernAnswer::Hire { slot: 0 })
        ),
        Err(GameCommandError::CannotPay)
    );
    assert_eq!(
        legal_commands(&poor, &rules, Actor::Player(RED)),
        [answer(TavernAnswer::Close)]
    );
    for game in [&game, &one, &held, &poor] {
        assert!(game.tavern(RED).is_some());
    }
    let texts = [
        TavernError::UnknownSlot,
        TavernError::EmptySlot,
        TavernError::TooManyHeroes,
        TavernError::Occupied,
    ]
    .map(|error| GameCommandError::Tavern(error).to_string());
    assert!(texts.iter().all(|text| !text.is_empty()));
}

/// The Thieves' Guild a command's events told: its head's player and level, then the view its
/// parts make (a place of a row other than the one before begins that row), as a client folds
/// them; none without a head.
fn told_guild(events: &[GameEvent]) -> Option<(PlayerId, ThievesGuildView)> {
    let mut told: Option<(PlayerId, ThievesGuildView)> = None;
    for event in events {
        match *event {
            GameEvent::ThievesGuildSeen { player, level } => {
                assert!(told.is_none(), "one guild a visit");
                let view = ThievesGuildView {
                    level,
                    rows: Vec::new(),
                    players: Vec::new(),
                };
                told = Some((player, view));
            }
            GameEvent::GuildPartSeen { player, part } => {
                let (seer, view) = told.as_mut().expect("its head first");
                assert_eq!(player, *seer);
                match part {
                    GuildPart::Place { row, of, place } => {
                        if view.rows.last().is_none_or(|r| r.row != row) {
                            view.rows.push(crate::GuildRanking {
                                row,
                                places: Vec::new(),
                            });
                        }
                        let ranking = view.rows.last_mut().unwrap();
                        assert_eq!(usize::from(of.raw()), ranking.places.len());
                        ranking.places.push(place);
                    }
                    GuildPart::Player { of, shown } => {
                        assert_eq!(usize::from(of.raw()), view.players.len());
                        view.players.push(shown);
                    }
                }
            }
            _ => {}
        }
    }
    told
}

// covers: ADV-VIS-008, TWN-TAV-007
#[test]
fn a_den_of_thieves_shows_its_visitors_owner_the_guild_at_everything() {
    // E09 T39b2 (design note §74, §75): a step onto the Den ends the move there and tells red
    // alone the guild at everything, whatever its Taverns (one: its own query's level is 1),
    // after the step; blue is told nothing of it.
    let rules = rules();
    let mut game = game(&rules, &inn_setup(&rules), 5000);
    assert_eq!(thieves_guild(&game, &rules, RED).map(|g| g.level), Some(1));
    let events = run(&rules, &mut game, RED, step(&[Pos::new(1, 0, 0), DEN])).unwrap();
    assert_eq!(game.adventure().hero(RED_HERO).map(|h| h.pos), Some(DEN));
    assert!(!game.has_decision(RED));
    assert!(!events.iter().any(|e| matches!(
        e,
        GameEvent::TavernOpened { .. } | GameEvent::OfferOpened { .. }
    )));
    let (seer, view) = told_guild(&events).unwrap();
    assert_eq!(seer, RED);
    let at_everything = thieves_guild_at(&game, &rules, DEN_LEVEL);
    assert_eq!(view, at_everything);
    assert_eq!(
        (view.level, view.rows.len(), view.players.len()),
        (DEN_LEVEL, 9, 2)
    );
    // Told after the step, together, in the parts' order.
    let head = events
        .iter()
        .position(|e| matches!(e, GameEvent::ThievesGuildSeen { .. }))
        .unwrap();
    let stepped = events.iter().rposition(|e| {
        matches!(e, GameEvent::Adventure(event) if matches!(
            event.kind,
            AdventureEventKind::HeroStepped { .. }
        ))
    });
    assert!(stepped.is_some_and(|s| s < head));
    let parts: Vec<GuildPart> = (events[head + 1..].iter())
        .map_while(|e| match *e {
            GameEvent::GuildPartSeen { part, .. } => Some(part),
            _ => None,
        })
        .collect();
    assert_eq!(parts, at_everything.parts().collect::<Vec<_>>());
    assert_eq!(parts.len(), 9 * 2 + 2);
    let guild = |e: &GameEvent| {
        matches!(
            e,
            GameEvent::ThievesGuildSeen { .. } | GameEvent::GuildPartSeen { .. }
        )
    };
    let blue: Vec<_> = (events.iter())
        .filter_map(|e| project(e, Viewer::Player(BLUE)))
        .collect();
    assert!(!blue.iter().any(guild));
    let red: Vec<_> = (events.iter())
        .filter_map(|e| project(e, Viewer::Player(RED)))
        .collect();
    assert_eq!(red.iter().filter(|e| guild(e)).count(), 1 + parts.len());
}

// covers: ADV-VIS-008
#[test]
fn every_visit_of_the_den_is_alike_and_a_revisit_is_one() {
    // A `Revisit` (listed for the hero on the Den) and a second step onto it the same day each
    // tell the guild at everything again, as it stands then.
    let rules = rules();
    let mut game = game(&rules, &inn_setup(&rules), 5000);
    let first = run(&rules, &mut game, RED, step(&[Pos::new(1, 0, 0), DEN])).unwrap();
    let revisit = GameCommand::Adventure(AdventureCommand::Revisit { hero: RED_HERO });
    assert!(legal_commands(&game, &rules, Actor::Player(RED)).contains(&revisit));
    let again = run(&rules, &mut game, RED, revisit).unwrap();
    assert_eq!(game.adventure().hero(RED_HERO).map(|h| h.pos), Some(DEN));
    assert_eq!(told_guild(&again), told_guild(&first));
    let back = run(&rules, &mut game, RED, step(&[Pos::new(1, 0, 0), DEN])).unwrap();
    assert_eq!(game.adventure().hero(RED_HERO).map(|h| h.pos), Some(DEN));
    let (seer, view) = told_guild(&back).unwrap();
    assert_eq!(
        (seer, view),
        (RED, thieves_guild_at(&game, &rules, DEN_LEVEL))
    );
}

// covers: ADV-VIS-008
#[test]
fn a_den_visit_without_a_recording_sink_changes_the_same() {
    // The guild is not stored: a sink that keeps nothing is told nothing and the state is the
    // same as a recorded visit's.
    let rules = rules();
    let mut recorded = game(&rules, &inn_setup(&rules), 5000);
    let mut quiet = recorded.clone();
    let command = step(&[Pos::new(1, 0, 0), DEN]);
    run(&rules, &mut recorded, RED, command.clone()).unwrap();
    apply(
        &mut quiet,
        &rules,
        Actor::Player(RED),
        command,
        &mut NoopSink,
    )
    .unwrap();
    assert_eq!(quiet.state_hash(), recorded.state_hash());
}
