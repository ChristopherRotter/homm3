//! The whole game's clock hosted (E16 M5 T17; SRV-Q31, SRV-Q32): the offer's clock in the lobby
//! and in the game's journal; the game's thread waking at the session's deadline, so that a turn
//! runs out at its end with nobody sending anything, and a stop journals the time up to it, so
//! that a restart keeps the time left; a battle against a monster played out by the server's
//! computer at the turn's end; a battle between players' heroes whose defender's chess clock runs
//! out, its side the computer's, the attacker's turn timer stopped meanwhile; a level-up answered
//! by its default at its limit, the other player told the game waits, then no more.

use std::fs;
use std::time::Duration;

use h3_contracts::PlayerId;
use h3_game::{Actor, SystemCommand};
use h3_protocol::session::game::{Allowance, Clock, GameEntry};
use h3_protocol::wire::game::{GameClock, GameCommand, GameEvent, OfferClock, Timer};
use h3_protocol::wire::{ClientMessage, Command, ServerMessage};
use tokio::time::Instant;

use super::whole::{MONSTER_REPLAY, duel_map, map_listed, next, rejoined, started_on};
use super::{fast, root};
use crate::client::Client;
use crate::config::{self, Config};
use crate::journal::whole as journal;
use crate::start;

/// A turn's duration in the tests that wait for its end.
const TURN_MS: u64 = 1_500;

/// The map offer `id` of the map file at `path`, pinned by its hash, a golden replay's seed and
/// difficulty (`replay`) and a computer's seed in place of the OS's, timed by `clock`.
fn timed(
    id: &str,
    replay: &str,
    clock: Clock,
) -> (config::MapOffer, h3_sim::adventure::replay::Replay) {
    let golden = fs::read_to_string(root().join(replay)).unwrap();
    let golden = h3_files::adventure::parse_replay(&golden).unwrap();
    let path = golden.map.path.clone();
    let (_, map) = h3_files::adventure::read_map(&root(), &path).unwrap();
    let offer = config::MapOffer {
        id: String::from(id),
        file: root().join(&path),
        hash: format!("{:#018x}", map.content_hash),
        difficulty: golden.difficulty,
        seeds: Some((golden.seed, 7)),
        test_clock: Some(clock),
        ..duel_map()
    };
    (offer, golden)
}

/// A clock of `turn_ms` a turn, `level_up_ms` a level-up and `battle` between players.
const fn clock(turn_ms: u64, level_up_ms: u64, battle: Allowance) -> Clock {
    Clock {
        turn_ms: Some(turn_ms),
        level_up_ms: Some(level_up_ms),
        battle,
    }
}

/// The test servers' configuration offering `maps`.
fn with(maps: Vec<config::MapOffer>) -> Config {
    Config {
        maps,
        ..super::config(fast())
    }
}

/// The next clock `client` is told, the messages before it skipped.
async fn told(client: &mut Client) -> GameClock {
    next(client, |m| match m {
        ServerMessage::GameClock { clock, .. } => Some(clock),
        _ => None,
    })
    .await
}

/// Reads `client`'s clocks until it is told `clock` (a timer run out may be told at 0 first).
async fn told_until(client: &mut Client, clock: GameClock) {
    while told(client).await != clock {}
}

/// The replay's commands `seqs`, each sent by its player's client (`clients`, by player) and
/// accepted.
async fn play(
    clients: [&mut Client; 2],
    replay: &h3_sim::adventure::replay::Replay,
    seqs: std::ops::RangeInclusive<usize>,
) {
    for seq in seqs {
        let (Actor::Player(player), command) = &replay.commands[seq] else {
            panic!("command {seq} is the system actor's");
        };
        let seq = u32::try_from(seq).unwrap();
        let message = ClientMessage::GameCommand {
            seq,
            command: h3_protocol::map::game::command(command),
        };
        let client = &mut *clients[player.index()];
        client.send(&message).await.unwrap();
        next(client, |m| {
            matches!(m, ServerMessage::GameAccepted { seq: s, .. } if s == seq).then_some(())
        })
        .await;
    }
}

/// `client`'s messages until `until` takes one (with it), or the connection's end.
async fn read_until(
    mut client: Client,
    until: impl Fn(&ServerMessage) -> bool,
) -> (Client, Vec<ServerMessage>) {
    let mut seen = Vec::new();
    while let Some(message) = client.recv().await {
        let last = until(&message);
        seen.push(message);
        if last {
            break;
        }
    }
    (client, seen)
}

/// Whether `message` tells a battle's end.
fn battle_ended(message: &ServerMessage) -> bool {
    matches!(message, ServerMessage::GameEvents { events, .. }
        if events.iter().any(|e| matches!(e, GameEvent::BattleEnded { .. })))
}

/// Whether `entry` is the system's command for `player`'s battle side.
fn system_battle(entry: &GameEntry, player: PlayerId) -> bool {
    matches!(entry, GameEntry::Command(Actor::System, h3_game::GameCommand::System(SystemCommand::Battle { player: p, .. })) if *p == player)
}

/// The system's command `command` as journaled.
fn system(command: SystemCommand) -> GameEntry {
    GameEntry::Command(Actor::System, h3_game::GameCommand::System(command))
}

/// Game `game`'s journal entries in the data directory `data`.
fn entries(data: &std::path::Path, game: u32) -> Vec<GameEntry> {
    let path = journal::dir(data).join(format!("{game}.journal"));
    journal::read(&path).unwrap().entries
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_turn_runs_out_at_its_end_and_a_restart_keeps_the_time_left() {
    let offer = config::MapOffer {
        test_clock: Some(clock(TURN_MS, 60_000, Allowance::DEFAULT)),
        ..duel_map()
    };
    let config = with(vec![offer]);
    let data = &config.server.data;
    let running = start(&config).await.unwrap();
    // The lobby shows how the offer's games are timed.
    let (maps, _) = map_listed(&running).await;
    let shown = OfferClock {
        turn_ms: Some(1_500),
        level_up_ms: Some(60_000),
        battle_ms: 180_000,
        increment_ms: 10_000,
    };
    assert_eq!(maps[0].clock, shown);
    // Nobody acts: the first player's turn runs out at its end, the system ends it, and the
    // second player's turn timer starts.
    let begun = Instant::now();
    let (a, mut b, game, [_, t1]) = started_on(&running, "duel-map").await;
    let started = told(&mut b).await;
    assert!(begun.elapsed() >= Duration::from_millis(TURN_MS));
    let Some(Timer {
        left_ms,
        running: true,
    }) = started.turn
    else {
        panic!("{started:?}")
    };
    assert!(u64::from(left_ms) <= TURN_MS);
    // A stop journals the time counted up to it.
    tokio::time::sleep(Duration::from_millis(TURN_MS / 3)).await;
    a.close().await;
    running.shutdown().await;
    let read = entries(data, game);
    let end = |player| {
        system(SystemCommand::EndTurn {
            player: PlayerId::new(player),
        })
    };
    let ended = read.iter().position(|e| *e == end(0)).unwrap();
    assert_eq!(read[ended - 1], GameEntry::Elapsed(TURN_MS));
    let [GameEntry::Elapsed(before)] = read[ended + 1..] else {
        panic!("{:?}", &read[ended + 1..])
    };
    assert!(before > 0 && before < TURN_MS, "{before}");
    // The restart: the second player's seat re-taken, its turn timer with the time it had left
    // (its clock ran since the restore, its seat empty); it runs out at its end.
    let restarted = Instant::now();
    let running = start(&config).await.unwrap();
    let mut b = super::greeted(&running).await;
    rejoined(&mut b, game, (1, &t1)).await;
    let back = told(&mut b).await;
    let Some(Timer {
        left_ms,
        running: true,
    }) = back.turn
    else {
        panic!("{back:?}")
    };
    assert!(u64::from(left_ms) <= TURN_MS - before, "{left_ms}");
    // Its turn ended.
    told_until(&mut b, GameClock::default()).await;
    assert!(restarted.elapsed() >= Duration::from_millis(TURN_MS - before));
    b.close().await;
    running.shutdown().await;
    // Its turn counted the duration exactly, across the restart.
    let read = entries(data, game);
    let ended = read.iter().position(|e| *e == end(0)).unwrap();
    let again = ended + 1 + read[ended + 1..].iter().position(|e| *e == end(1)).unwrap();
    let counted: u64 = (read[ended + 1..again].iter())
        .map(|e| match e {
            GameEntry::Elapsed(ms) => *ms,
            other => panic!("not the clock's: {other:?}"),
        })
        .sum();
    assert_eq!(counted, TURN_MS);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_battle_against_a_monster_is_played_out_by_the_computer_at_the_turns_end() {
    // Three seconds a turn: the replay's opening plays well within each.
    let (offer, golden) = timed(
        "duel-map",
        MONSTER_REPLAY,
        clock(2 * TURN_MS, 60_000, Allowance::DEFAULT),
    );
    let config = with(vec![offer]);
    let running = start(&config).await.unwrap();
    let (mut a, mut b, game, _) = started_on(&running, "duel-map").await;
    // The opening up to the command that opens the second player's battle against a wandering
    // monster; the fighter then sends nothing.
    let opens = 16;
    play([&mut a, &mut b], &golden, 0..=opens).await;
    let reading = tokio::spawn(read_until(a, |_| false));
    let (mut b, seen) = read_until(b, battle_ended).await;
    assert!(battle_ended(seen.last().unwrap()));
    // Its turn ended.
    told_until(&mut b, GameClock::default()).await;
    b.close().await;
    running.shutdown().await;
    let (_, _) = reading.await.unwrap();
    // After the fighter's last command, the time to the turn's end, then the computer's commands
    // for its side, and the turn's end: the system's.
    let read = entries(&config.server.data, game);
    let last = (read.iter())
        .rposition(|e| matches!(e, GameEntry::Numbered(..)))
        .unwrap();
    assert!(
        matches!(&read[last], GameEntry::Numbered(p, n, _) if (p.raw(), *n) == (1, 16)),
        "{:?}",
        read[last]
    );
    let fighter = PlayerId::new(1);
    let played = (read[last..].iter())
        .position(|e| system_battle(e, fighter))
        .unwrap();
    assert!(
        (read[last..last + played].iter()).any(|e| matches!(e, GameEntry::Elapsed(_))),
        "{:?}",
        &read[last..]
    );
    let end = system(SystemCommand::EndTurn { player: fighter });
    let ended = read.iter().position(|e| *e == end).unwrap();
    assert!(ended > last + played);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_side_whose_chess_clock_runs_out_is_the_computers_the_attackers_turn_timer_stopped() {
    // A minute a turn; a second each side in a battle between players, half a second more a
    // command.
    let allowance = Allowance {
        base_ms: 1_000,
        increment_ms: 500,
    };
    let (offer, golden) = timed(
        "towns-map",
        "fixtures/replays/adventure/towns-visitor-s1.toml",
        clock(60_000, 60_000, allowance),
    );
    let config = with(vec![offer]);
    let running = start(&config).await.unwrap();
    let (maps, _) = map_listed(&running).await;
    assert_eq!(
        (maps[0].clock.battle_ms, maps[0].clock.increment_ms),
        (1_000, 500)
    );
    let (mut a, mut b, game, _) = started_on(&running, "towns-map").await;
    // The first player's hero attacks the second's; the defender acts first and sends nothing.
    play([&mut a, &mut b], &golden, 0..=0).await;
    let opened = told(&mut a).await;
    let attacker = |clock: GameClock| clock.battle.map(|duel| duel.attacker);
    assert!(
        matches!(opened.turn, Some(Timer { running: false, .. })),
        "{opened:?}"
    );
    assert_eq!(
        attacker(opened),
        Some(Timer {
            left_ms: 1_000,
            running: false
        })
    );
    let defending = tokio::spawn(read_until(b, battle_ended));
    // The attacker plays its side at once until the battle ends.
    let begun = Instant::now();
    let mut rng = h3_kernel::Rng::from_seed(5);
    let mut seq = 0;
    loop {
        match a.recv().await.expect("the connection closed") {
            ServerMessage::GameTurn { legal, .. } => {
                let battle: Vec<Command> = (legal.iter())
                    .filter_map(|c| match c {
                        GameCommand::Battle { command } => Some(*command),
                        _ => None,
                    })
                    .collect();
                let Some(command) = crate::bench::choose(&battle, &mut rng) else {
                    continue;
                };
                seq += 1;
                let command = GameCommand::Battle { command };
                a.send(&ClientMessage::GameCommand { seq, command })
                    .await
                    .unwrap();
            }
            message if battle_ended(&message) => break,
            _ => {}
        }
    }
    let fought = begun.elapsed();
    assert!(fought >= Duration::from_millis(allowance.base_ms));
    // The attacker's turn timer runs again with what it had left: nothing counted in the battle.
    let after = told(&mut a).await;
    assert_eq!(after.battle, None);
    let Some(Timer {
        left_ms,
        running: true,
    }) = after.turn
    else {
        panic!("{after:?}")
    };
    let used = Duration::from_millis(60_000 - u64::from(left_ms));
    assert!(
        used < fought,
        "{used:?} of the turn used, {fought:?} in the battle"
    );
    // The defender was told its clock ran out; its side was the computer's from then on.
    let (b, seen) = defending.await.unwrap();
    let defender = (seen.iter())
        .filter_map(|m| match m {
            ServerMessage::GameClock { clock, .. } => clock.battle.map(|duel| duel.defender),
            _ => None,
        })
        .next_back();
    assert_eq!(
        defender,
        Some(Timer {
            left_ms: 0,
            running: false
        })
    );
    for client in [a, b] {
        client.close().await;
    }
    running.shutdown().await;
    let read = entries(&config.server.data, game);
    let theirs = read.iter().filter(|e| system_battle(e, PlayerId::new(1)));
    assert!(theirs.count() > 0, "{read:?}");
    assert!(!read.iter().any(|e| system_battle(e, PlayerId::new(0))));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_level_up_is_answered_by_its_default_at_its_limit() {
    let level_up_ms = 800;
    let (offer, golden) = timed(
        "rewards-map",
        "fixtures/replays/adventure/rewards-stone-guard-s1.toml",
        clock(60_000, level_up_ms, Allowance::DEFAULT),
    );
    let config = with(vec![offer]);
    let running = start(&config).await.unwrap();
    let (mut a, mut b, game, _) = started_on(&running, "rewards-map").await;
    // The first player's hero wins its battle and levels up: its level-up holds the game.
    play([&mut a, &mut b], &golden, 0..=0).await;
    let begun = Instant::now();
    play([&mut a, &mut b], &golden, 1..=1).await;
    let held = told(&mut a).await;
    assert_eq!(
        held.level_up,
        Some(Timer {
            left_ms: u32::try_from(level_up_ms).unwrap(),
            running: true,
        })
    );
    assert!(matches!(held.turn, Some(Timer { running: false, .. })));
    let waits = GameClock {
        waits: true,
        ..GameClock::default()
    };
    assert_eq!(told(&mut b).await, waits);
    // At its limit the system answers it by its default; the game goes on.
    assert_eq!(told(&mut b).await, GameClock::default());
    assert!(begun.elapsed() >= Duration::from_millis(level_up_ms));
    let after = told(&mut a).await;
    assert_eq!(
        (after.level_up, after.turn.map(|t| t.running)),
        (None, Some(true))
    );
    for client in [a, b] {
        client.close().await;
    }
    running.shutdown().await;
    let read = entries(&config.server.data, game);
    let answer = system(SystemCommand::Answer {
        player: PlayerId::new(0),
    });
    let answered = read.iter().position(|e| *e == answer).unwrap();
    assert_eq!(read[answered - 1], GameEntry::Elapsed(level_up_ms));
}
