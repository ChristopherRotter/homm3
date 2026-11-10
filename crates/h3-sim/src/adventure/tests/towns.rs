//! The env on the town map (E09 T9c, the town epic's design note §18; exit criterion 7): the
//! committed `fixtures/maps/e09-towns-36.h3m` (`h3-map`'s `tests/towns_map.rs` lays it out: eight
//! towns of every kind the import reads, red's and blue's main towns, a garrison of random stacks,
//! a visitor). A hero enters a town by a move onto its entrance, which captures an undefended one
//! (TWN-CAP-001) or opens a siege (TWN-SIE-001, -002) the battle encoding plays; since E09 T19c1
//! (encoding v9) a town's owner builds with the entry `BUILD` (its town's entrance in the tile
//! head, the building in the building head; TWN-BLD-001). Random play of two seeds (searched again
//! at E09 T19c1: the builds in the random agent's draw changed every random game's course, as the
//! revisits did at E08 T105b2; the golden replays `towns-s173-d21`, `-s1702-d21` and `-s3435-d21`
//! keep the games of before as their commands) runs to the day limit with the masks agreeing with
//! the game at every step and meets the town's features and its builds; the main property
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
        for event in &events {
            match *event {
                GameEvent::StackChanged { .. } => {
                    changed = true;
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

/// Two seeds of random play on the town map (E09 T19c1: a search over the seeds 1 to 299, the
/// shortest pair whose union meets the town's features, 425 steps and 66 builds; E08 T105b2's 36
/// and 330 changed course with the builds in the draw, E09 T9a's, the golden replays' three, with
/// the revisits) play through the env to their day limit:
/// the masks agree with the game before each step ([`play`]), each chosen action is legal by them
/// and accepted, a captured town is its new owner's in its observation; the union of the two
/// meets an undefended capture, a siege against a garrison alone and one against a visitor with
/// the garrison merged, a siege won (the town captured after it) and one lost, and builds by the
/// towns' owners, each town's the only one of its day ([`play`]). The games end on
/// the day after the limit with no seat to move.
#[test]
fn the_env_plays_the_town_map_to_its_end_of_days() {
    let mut all = Played::default();
    for seed in [142, 244] {
        let (env, played) = play(seed, 21);
        assert!(env.is_done(), "{seed}");
        assert_eq!((env.day(), env.to_move().count()), (22, 0), "{seed}");
        assert_eq!(
            env.fork().step(PlayerId::new(0), Action::end_turn()),
            Err(StepError::EpisodeOver)
        );
        all.undefended += played.undefended;
        all.sieges += played.sieges;
        all.merged += played.merged;
        all.won += played.won;
        all.lost += played.lost;
        all.built += played.built;
    }
    assert!(all.undefended > 0, "{all:?}");
    assert!(all.sieges > all.merged && all.merged > 0, "{all:?}");
    assert!(all.won > 0 && all.lost > 0, "{all:?}");
    assert_eq!(all.won + all.lost, all.sieges, "{all:?}");
    assert!(all.built > 0, "{all:?}");
}

/// The town map's rollouts are deterministic, and a fork taken while a siege is open (seed 244: a
/// siege on day 2; seed 330's before E09 T19c1, 3435's before E08 T105b2) carries it: played on by one
/// agent, the fork and its parent evolve identically, and another agent's play on a fork leaves
/// the parent as it was.
#[test]
fn town_map_rollouts_and_forks_are_deterministic() {
    let new = || AdventureEnv::new(FAITHFUL.clone(), town_setup(&FAITHFUL, 1), 244, 3).unwrap();
    let (mut a, mut b) = (new(), new());
    let ra = random_rollout(&mut a, 244, 0);
    assert_eq!(ra, random_rollout(&mut b, 244, 0));
    assert_eq!(a.state_hash(), b.state_hash());
    let mut parent = new();
    let mut rng = Rng::for_stream(244, StreamId::AGENT);
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
