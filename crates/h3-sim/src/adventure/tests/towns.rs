//! The env on the town map (E09 T9c, the town epic's design note §18; exit criterion 7): the
//! committed `fixtures/maps/e09-towns-36.h3m` (`h3-map`'s `tests/towns_map.rs` lays it out: eight
//! towns of every kind the import reads, red's and blue's main towns, a garrison of random stacks,
//! a visitor). A hero enters a town by a move onto its entrance, which captures an undefended one
//! (TWN-CAP-001) or opens a siege (TWN-SIE-001, -002) the battle encoding plays; since E09 T19c1
//! (encoding v9) a town's owner builds with the entry `BUILD` (its town's entrance in the tile
//! head, the building in the building head; TWN-BLD-001), and since E09 T27 (encoding v10) it
//! recruits with `RECRUIT` (the same heads; TWN-REC-001, -002), the dialog's purchases its hire
//! entries, and since E09 T38a2 (encoding v12) it hires with `TAVERN_HIRE` (the town's entrance in
//! the tile head; TWN-TAV-003). Random play of one seed (searched again at E09 T38a2, T27 and
//! T19c1: the hires, the recruitments and the builds in the random agent's draw changed every
//! random game's course, as the revisits did
//! at E08 T105b2; the golden replays `towns-s173-d21`, `-s1702-d21` and `-s3435-d21` keep the
//! games of before as their commands) runs to the day limit with the masks agreeing with the game
//! at every step and meets the town's features, its builds and its recruitments; the main property
//! `steps_agree_with_the_masks_and_the_game` aims moves at the towns' entrances ([`entrances`]).

use h3_adventure::{AdventureEvent, AdventureEventKind, Pos};
use h3_contracts::{BattleResult, BattleSide, TownId};
use h3_game::GameEvent;

use super::*;

const TOWNS: &[u8] = include_bytes!("../../../../../fixtures/maps/e09-towns-36.h3m");

/// The town map's setup at `difficulty`.
pub(super) fn town_setup(rules: &GameRuleset, difficulty: u8) -> Arc<GameSetup> {
    let m = h3_map::read(TOWNS, &h3_map::Limits::default()).unwrap();
    let mut setup = h3_map::import(&m, rules).unwrap();
    setup.difficulty = difficulty;
    Arc::new(setup)
}

/// The entrances of `state`'s towns, in `TownId` order.
pub(super) fn entrances(state: &GameState) -> Vec<Pos> {
    let map = state.adventure().map();
    map.towns()
        .iter()
        .filter_map(|&index| map.objects()[index].entrance(map))
        .collect()
}

/// What a game on the town map did with its towns, counted from the steps' events.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Played {
    /// Towns captured undefended: a `TownCaptured` in a step that ended no battle (TWN-CAP-001).
    undefended: u32,
    /// Battles opened on a town's entrance: sieges (TWN-SIE-003).
    sieges: u32,
    /// Sieges whose step merged the garrison into the defending visitor's army before the battle
    /// (`StackChanged` before `BattleStarted`; TWN-SIE-002).
    merged: u32,
    /// Sieges the attacker won: the town captured after the battle (TWN-CAP-003).
    won: u32,
    /// Sieges the attacker lost.
    lost: u32,
    /// Buildings built by their owners' commands (TWN-BLD-001: a `TownBuilt` of no town event).
    built: u32,
    /// Town hire dialogs opened (E09 T27, TWN-REC-001: a `HireOpened` into a town's garrison).
    recruited: u32,
    /// Purchases into a town's garrison (a garrison's `StackChanged` in a step with no battle).
    bought: u32,
    /// Tavern heroes hired at a town (E09 T38a2, TWN-TAV-003: a `HeroHired` of a slot).
    hired: u32,
    /// Spell books bought at a town's Mage Guild (E09 T49b, TWN-GLD-007: an accepted
    /// [`SPELL_BOOK`] action).
    books: u32,
}

/// Plays the town map with the env's random agent (as `random_rollout` and `h3 adventure run
/// --seed N --agent-seed N`) to its day limit `days`, checking the masks against the game before
/// each step (the map's where the towns matter, below), the observation of a town's new owner
/// after its capture and, after a build, that its town offers no other build that day
/// (TWN-BLD-004: its entrance out of `BUILD`'s tile mask): what the game did with its towns.
fn play(seed: u64, days: u32) -> (AdventureEnv, Played) {
    let mut env =
        AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), seed, days).unwrap();
    let mut rng = Rng::for_stream(seed, StreamId::AGENT);
    let mut played = Played::default();
    // The town of the siege in progress.
    let mut siege: Option<TownId> = None;
    let seats = [
        Seat::Player(PlayerId::new(0)),
        Seat::Player(PlayerId::new(1)),
        Seat::Neutral,
    ];
    let gates = entrances(env.state());
    // The last step captured a town, merged a garrison or ended a siege.
    let mut touched = false;
    loop {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        let aimed = match action {
            Action::Adventure(a) => {
                a.entry == BUILD
                    || a.entry == RECRUIT
                    || a.entry >= TAVERN_HIRE
                    || is_move(a.entry)
                        && tile_pos(env.state(), a.tile).is_some_and(|p| gates.contains(&p))
            }
            Action::Battle(_) => false,
        };
        // Every seat's battle masks at every step. The map's (each hero's reach over the map)
        // cost the most in a debug build: the acting seat's before a move onto a town's entrance
        // or a build, after a step that changed a town and at every eighth step, every seat's at
        // every fortieth (the main property checks them at every step on every map).
        let steps = env.steps();
        for other in seats {
            let map = other == seat && (aimed || touched || steps.is_multiple_of(8));
            if map || steps.is_multiple_of(40) || env.phase(other) == Phase::Battle {
                check_masks(&env, other).unwrap();
            } else {
                check_battle_masks(&env, other).unwrap();
            }
        }
        assert!(legal_by_masks(&env, seat, action), "{seat:?} {action:?}");
        let mut events = Vec::new();
        let result = env.step_with(seat, action, &mut events).unwrap();
        played.books += u32::from(matches!(action, Action::Adventure(a) if a.entry == SPELL_BOOK));
        // Only the day limit ends an episode: the step that starts the day after it.
        assert_eq!(
            (result.done, result.truncated),
            (env.is_done(), env.is_done())
        );
        let map = env.state().adventure().map();
        let mut ended = false;
        let mut changed = false;
        touched = false;
        let battle = events
            .iter()
            .any(|e| matches!(e, GameEvent::BattleStarted { .. }));
        for event in &events {
            match *event {
                GameEvent::StackChanged { at, .. } => {
                    changed = true;
                    touched = true;
                    let garrison = matches!(at.army, h3_game::ArmyRef::Garrison(_));
                    played.bought += u32::from(garrison && !battle);
                }
                GameEvent::HireOpened {
                    target: h3_game::HireTarget::Garrison(_),
                    ..
                } => played.recruited += 1,
                GameEvent::HeroHired { slot: Some(_), .. } => {
                    played.hired += 1;
                    touched = true;
                }
                GameEvent::BattleStarted { tile, .. } => {
                    if let Some(town) = map.town_at(tile) {
                        played.sieges += 1;
                        played.merged += u32::from(changed);
                        siege = Some(town);
                    }
                }
                GameEvent::BattleEnded { result, .. } => {
                    ended = true;
                    if siege.take().is_some() {
                        touched = true;
                        // The attacker is side 0: it won when the defender fell, fled or yielded.
                        let won = matches!(
                            result,
                            BattleResult::Victory(BattleSide::Attacker)
                                | BattleResult::Retreat(BattleSide::Defender)
                                | BattleResult::Surrender(BattleSide::Defender)
                        );
                        played.won += u32::from(won);
                        played.lost += u32::from(!won);
                    }
                }
                GameEvent::TownBuilt {
                    player,
                    town,
                    event: None,
                    ..
                } => {
                    played.built += 1;
                    touched = true;
                    // One build a town a day (TWN-BLD-004): its entrance leaves `BUILD`'s tiles.
                    let mut tiles = Vec::new();
                    env.tile_mask(player, BUILD, &mut tiles);
                    let gate = town_tile(env.state(), town).unwrap() as usize;
                    assert!(!tiles[gate], "{town:?} built twice a day");
                }
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::TownCaptured { town, owner, .. },
                    ..
                }) => {
                    played.undefended += u32::from(!ended);
                    touched = true;
                    // The new owner sees its town as its own at once.
                    let view = env.observation(owner);
                    let seen = view.towns.iter().find(|t| t.town == town).unwrap();
                    assert_eq!(seen.owner, Some(owner));
                }
                _ => {}
            }
        }
    }
    (env, played)
}

/// Random play on the town map (E09 T49b: a search over the seeds 1 to 3 000 after the spell
/// books' purchases joined the agent's draw, encoding v19: seed 277 the shortest of the four
/// meeting every feature, 657 steps, 30 builds, 29 dialogs, 9 hires and a book; E09 T38b1's seed
/// 2541, the shortest of ten after a hero that leaves the game came back, TWN-TAV-006, its
/// returns drawing from its owner's stream and its freed type drawn again, 543 steps, 28 builds,
/// 58 dialogs and 13 hires; E09 T38a2's seed 2369, the shortest of ten over the seeds
/// 1 to 2 400 after the hires joined the agent's draw (encoding v12), E09 T27's seed 70, E09 T19c1's 142 and 244, E08 T105b2's 36 and 330 and E09 T9a's, the golden
/// replays' three, changed course with the hires, the recruitments, the builds and the revisits)
/// plays through the env to its day limit: the masks agree with the game before each
/// step ([`play`]), each chosen action is legal by them and accepted, a captured town is its new
/// owner's in its observation; it meets an undefended capture, a siege against a garrison alone
/// and one against a visitor with the garrison merged, a siege won (the town captured after it)
/// and one lost, builds by the towns' owners, each town's the only one of its day, town hire
/// dialogs with purchases into a garrison (TWN-REC-001; [`play`]) and tavern heroes hired
/// (TWN-TAV-003, E09 T38a2) and spell books bought at a town's Mage Guild (TWN-GLD-007, E09
/// T49b). The game ends on the day after
/// the limit with no seat to move.
#[test]
fn the_env_plays_the_town_map_to_its_end_of_days() {
    let (env, all) = play(277, 21);
    assert!(env.is_done());
    assert_eq!((env.day(), env.to_move().count()), (22, 0));
    assert_eq!(
        env.fork().step(PlayerId::new(0), Action::end_turn()),
        Err(StepError::EpisodeOver)
    );
    assert!(all.undefended > 0, "{all:?}");
    assert!(all.sieges > all.merged && all.merged > 0, "{all:?}");
    assert!(all.won > 0 && all.lost > 0, "{all:?}");
    assert_eq!(all.won + all.lost, all.sieges, "{all:?}");
    assert!(all.built > 0, "{all:?}");
    assert!(all.recruited > 0 && all.bought > 0, "{all:?}");
    assert!(all.hired > 0, "{all:?}");
    assert!(all.books > 0, "{all:?}");
}

/// The town map's rollouts are deterministic, and a fork taken while a siege is open (seed 6: a
/// siege by day 3; seed 70's before E09 T38a2, 46's before E09 T28a, 244's before E09 T27, 330's before E09 T19c1,
/// 3435's before E08 T105b2) carries it: played on by one agent, the fork and its parent evolve
/// identically, and another agent's play on a fork leaves the parent as it was.
#[test]
fn town_map_rollouts_and_forks_are_deterministic() {
    let new = || AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), 6, 3).unwrap();
    let (mut a, mut b) = (new(), new());
    let ra = random_rollout(&mut a, 6, 0);
    assert_eq!(ra, random_rollout(&mut b, 6, 0));
    assert_eq!(a.state_hash(), b.state_hash());
    let mut parent = new();
    let mut rng = Rng::for_stream(6, StreamId::AGENT);
    let besieged = |env: &AdventureEnv| {
        let map = env.state().adventure().map();
        (0..env.state().players()).any(|p| {
            let player = PlayerId::new(u8::try_from(p).unwrap());
            env.state()
                .battle(player)
                .is_some_and(|battle| map.town_at(battle.tile()).is_some())
        })
    };
    while !besieged(&parent) {
        let seat = parent
            .to_move()
            .next()
            .expect("a siege before the day limit");
        let action = choose(&parent, seat, &mut rng);
        parent.step(seat, action).unwrap();
    }
    let hash = parent.state_hash();
    let mut wild = parent.fork();
    random_rollout(&mut wild, 1, 40);
    assert_eq!(parent.state_hash(), hash);
    let mut fork = parent.fork();
    let played = random_rollout(&mut parent, 7, 0);
    assert_eq!(played, random_rollout(&mut fork, 7, 0));
    assert_eq!(parent.state(), fork.state());
    assert_eq!(parent.state_hash(), fork.state_hash());
}

/// The rollout draws a town's builds (E09 T19c1, encoding v9; from E09 T18b2b they were left out
/// of the draw, decision 56): on red's first turn of the town map `legal_commands` lists red's
/// builds, and of 64 agent streams' first choices some are builds, each legal by the masks and
/// decoding to a listed build.
#[test]
fn the_rollout_draws_the_builds() {
    let env = AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), 173, 21).unwrap();
    let red = Seat::Player(PlayerId::new(0));
    let listed = legal_commands(env.state(), env.rules(), red.actor());
    assert!(listed.iter().any(|c| matches!(c, GameCommand::Town(_))));
    let mut drawn = 0;
    for stream in 0..64 {
        let action = choose(&env, red, &mut Rng::for_stream(stream, StreamId::AGENT));
        if matches!(action, Action::Adventure(a) if a.entry == BUILD) {
            drawn += 1;
            assert!(legal_by_masks(&env, red, action), "{action:?}");
            let command = action::decode(env.state(), env.rules(), red, action).unwrap();
            assert!(listed.contains(&command), "{command:?}");
        }
    }
    assert!(drawn > 0);
}

/// A build's action names its town by the town's entrance and its building by its index in the
/// town's record (E09 T19c1, encoding v9): red's first listed build on the town map encodes so and
/// decodes back; a building past the head, a tile that is no town's entrance or off the map, and a
/// town the map does not have name nothing; after `BUILD` and a tile with no town the building
/// mask is empty, after `EndTurn` 0 alone at tile 0 and nothing at another tile; a building with
/// another entry decodes to nothing.
#[test]
fn a_build_action_names_its_town_by_the_entrance() {
    let env = AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), 173, 21).unwrap();
    let (state, rules) = (env.state(), env.rules());
    let red = PlayerId::new(0);
    let build = legal_commands(state, rules, Actor::Player(red))
        .into_iter()
        .find(|c| matches!(c, GameCommand::Town(_)))
        .unwrap();
    let GameCommand::Town(TownCommand::Build { town, building }) = build else {
        unreachable!()
    };
    let gate = town_tile(state, town).unwrap();
    assert_eq!(
        tile_pos(state, gate),
        entrances(state).get(town.index()).copied()
    );
    let action = Action::build(gate, building.raw());
    assert_eq!(action::encode(state, rules, red, &build), Some(action));
    assert_eq!(action::decode(state, rules, red, action), Some(build));
    let count = u32::try_from(env.tile_count()).unwrap();
    let elsewhere = (0..count).find(|&t| town_of(state, t).is_none()).unwrap();
    for none in [
        Action::build(gate, u8::try_from(BUILDING_COUNT).unwrap()),
        Action::build(elsewhere, building.raw()),
        Action::build(count, building.raw()),
        Action::Adventure(MapAction {
            entry: END_TURN,
            tile: 0,
            building: 3,
        }),
    ] {
        assert_eq!(action::decode(state, rules, red, none), None, "{none:?}");
    }
    let unknown = GameCommand::Town(TownCommand::Build {
        town: TownId::new(99),
        building,
    });
    assert_eq!(action::encode(state, rules, red, &unknown), None);
    assert_eq!(town_tile(state, TownId::new(99)), None);
    let mut mask = [true; BUILDING_COUNT];
    env.building_mask(red, BUILD, elsewhere, &mut mask);
    assert_eq!(mask, [false; BUILDING_COUNT]);
    env.building_mask(red, BUILD, gate, &mut mask);
    assert!(mask[building.index()]);
    env.building_mask(red, END_TURN, 0, &mut mask);
    assert_eq!((mask[0], mask.iter().filter(|&&b| b).count()), (true, 1));
    env.building_mask(red, END_TURN, 1, &mut mask);
    assert_eq!(mask, [false; BUILDING_COUNT]);
}

/// A hire's action names its town by the town's entrance and the slot by its entry (E09 T38a2,
/// encoding v12): each listed hire of red's on the town map's first turn encodes so, decodes back
/// and is legal by the masks; a slot past the two encodes to nothing, and a tile that is no town's
/// entrance, a tile off the map or a building other than 0 decode to nothing.
#[test]
fn a_hire_action_names_its_town_by_the_entrance_and_the_slot_by_its_entry() {
    let env = AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), 173, 21).unwrap();
    let (state, rules) = (env.state(), env.rules());
    let red = PlayerId::new(0);
    let hires: Vec<GameCommand> = legal_commands(state, rules, Actor::Player(red))
        .into_iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::Hire { .. })))
        .collect();
    assert!(!hires.is_empty());
    for hire in &hires {
        let GameCommand::Town(TownCommand::Hire { town, slot }) = *hire else {
            unreachable!()
        };
        let action = Action::tavern_hire(town_tile(state, town).unwrap(), usize::from(slot));
        assert_eq!(action::encode(state, rules, red, hire), Some(action));
        assert_eq!(
            action::decode(state, rules, red, action),
            Some(hire.clone())
        );
        assert!(
            legal_by_masks(&env, Seat::Player(red), action),
            "{action:?}"
        );
    }
    let GameCommand::Town(TownCommand::Hire { town, .. }) = hires[0] else {
        unreachable!()
    };
    let gate = town_tile(state, town).unwrap();
    let past = GameCommand::Town(TownCommand::Hire { town, slot: 2 });
    assert_eq!(action::encode(state, rules, red, &past), None);
    let count = u32::try_from(env.tile_count()).unwrap();
    let elsewhere = (0..count).find(|&t| town_of(state, t).is_none()).unwrap();
    for none in [
        Action::tavern_hire(elsewhere, 0),
        Action::tavern_hire(count, 1),
        Action::Adventure(MapAction {
            entry: TAVERN_HIRE,
            tile: gate,
            building: 1,
        }),
    ] {
        assert_eq!(action::decode(state, rules, red, none), None, "{none:?}");
    }
}

/// The town whose entrance is the tile at `index`, if one's is.
fn town_of(state: &GameState, index: u32) -> Option<TownId> {
    state.adventure().map().town_at(tile_pos(state, index)?)
}

/// The town map as a replay names it.
fn towns_map() -> replay::MapRef {
    replay::MapRef {
        path: "fixtures/maps/e09-towns-36.h3m".to_owned(),
        content_hash: replay::map_content_hash(TOWNS),
        import_version: h3_map::MAP_IMPORT_VERSION,
    }
}

/// The players' choices over the import (replay format v20, E27 M4 T3a; CLT-Q156 (a)): a game set
/// up with red's hero picked and blue's faction drawn (its one faction, the map's fixed choice)
/// and its human flag turned records them and verifies against the map's plain import; without
/// them the import's game is another. A choice of a colour no player has, or of a colour twice,
/// is refused before the game is built, and a pick the offer lacks by the game.
#[test]
fn a_replay_keeps_the_players_choices_over_the_import() {
    use h3_game::FactionChoice;
    use replay::ReplayError as E;
    let import = town_setup(&FAITHFUL, 1);
    let (red, seed) = (PlayerId::new(0), 7);
    let drawn = GameState::new(&FAITHFUL, &import, seed)
        .unwrap()
        .state_hash();
    let mut setup = (*import).clone();
    setup.players[1].start.choice = Some(FactionChoice::Random);
    setup.players[1].human = !setup.players[1].human;
    let offer = setup.offer(&FAITHFUL, red);
    // A hero of red's offer whose game is not the import's draw.
    let pick = offer
        .iter()
        .copied()
        .find(|&hero| {
            setup.players[0].start.pick = Some(hero);
            GameState::new(&FAITHFUL, &setup, seed).is_ok_and(|s| s.state_hash() != drawn)
        })
        .unwrap();
    setup.players[0].start.pick = Some(pick);
    let commands = vec![(Actor::Player(red), GameCommand::EndTurn)];
    let recorded =
        replay::Replay::record_commands(&FAITHFUL, (&setup, towns_map()), seed, commands, 1)
            .unwrap();
    assert_eq!(recorded.players, replay::PlayerChoices::of(&setup));
    assert_eq!(recorded.players.len(), import.players.len());
    assert_eq!(recorded.setup(&import), Ok(setup.clone()));
    let verify = |r: &replay::Replay| r.verify(&FAITHFUL, &towns_map(), &import);
    assert_eq!(verify(&recorded), Ok(()));
    let mut none = recorded.clone();
    none.players.clear();
    assert_eq!(none.setup(&import), Ok((*import).clone()));
    assert!(matches!(verify(&none), Err(E::HashMismatch { .. })));
    let mut stranger = recorded.clone();
    stranger.players[1].colour = 9;
    assert_eq!(verify(&stranger), Err(E::Incompatible("players")));
    let mut twice = recorded.clone();
    twice.players[1].colour = twice.players[0].colour;
    assert_eq!(twice.setup(&import), Err(E::Incompatible("players")));
    let unoffered = (0..u16::MAX)
        .map(h3_content::HeroTypeId::from_raw)
        .find(|hero| !offer.contains(hero))
        .unwrap();
    let mut wrong = recorded.clone();
    wrong.players[0].hero = Some(unoffered);
    assert_eq!(
        verify(&wrong),
        Err(E::Setup(h3_game::GameSetupError::Pick(red)))
    );
}
