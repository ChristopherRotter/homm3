//! The env on the town map (E09 T9c, the town epic's design note §18; exit criterion 7): the
//! committed `fixtures/maps/e09-towns-36.h3m` (`h3-map`'s `tests/towns_map.rs` lays it out: eight
//! towns of every kind the import reads, red's and blue's main towns, a garrison of random stacks,
//! a visitor). A hero enters a town by a move onto its entrance, which captures an undefended one
//! (TWN-CAP-001) or opens a siege (TWN-SIE-001, -002) the battle encoding plays; since E09 T19c1
//! (encoding v9) a town's owner builds with the entry `BUILD` (its town's entrance in the tile
//! head, the building in the building head; TWN-BLD-001), and since E09 T27 (encoding v10) it
//! recruits with `RECRUIT` (the same heads; TWN-REC-001, -002), the dialog's purchases its hire
//! entries. Random play of one seed (searched again at E09 T27, and at T19c1: the recruitments and
//! the builds in the random agent's draw changed every random game's course, as the revisits did
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

/// Random play on the town map (E09 T27: a search over the seeds 1 to 399 after the recruitments
/// joined the agent's draw, seed 70 alone meeting the town's features in 439 steps, 27 builds and
/// 83 dialogs; E09 T19c1's 142 and 244, the shortest pair then, E08 T105b2's 36 and 330 and E09
/// T9a's, the golden replays' three, changed course with the recruitments, the builds and the
/// revisits) plays through the env to its day limit: the masks agree with the game before each
/// step ([`play`]), each chosen action is legal by them and accepted, a captured town is its new
/// owner's in its observation; it meets an undefended capture, a siege against a garrison alone
/// and one against a visitor with the garrison merged, a siege won (the town captured after it)
/// and one lost, builds by the towns' owners, each town's the only one of its day, and town hire
/// dialogs with purchases into a garrison (TWN-REC-001; [`play`]). The game ends on the day after
/// the limit with no seat to move.
#[test]
fn the_env_plays_the_town_map_to_its_end_of_days() {
    let (env, all) = play(70, 21);
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
}

/// The town map's rollouts are deterministic, and a fork taken while a siege is open (seed 46: a
/// siege on day 1; seed 244's before E09 T27, 330's before E09 T19c1, 3435's before E08 T105b2)
/// carries it: played on by one
/// agent, the fork and its parent evolve identically, and another agent's play on a fork leaves
/// the parent as it was.
#[test]
fn town_map_rollouts_and_forks_are_deterministic() {
    let new = || AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), 46, 3).unwrap();
    let (mut a, mut b) = (new(), new());
    let ra = random_rollout(&mut a, 46, 0);
    assert_eq!(ra, random_rollout(&mut b, 46, 0));
    assert_eq!(a.state_hash(), b.state_hash());
    let mut parent = new();
    let mut rng = Rng::for_stream(46, StreamId::AGENT);
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

/// The town whose entrance is the tile at `index`, if one's is.
fn town_of(state: &GameState, index: u32) -> Option<TownId> {
    state.adventure().map().town_at(tile_pos(state, index)?)
}
