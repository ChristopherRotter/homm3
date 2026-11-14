//! A whole game's spectators hosted (E16 M5 T22b; SRV-Q36: each player's view): a game its
//! creator opened is listed so and watched from before its start, the spectator hearing every
//! message either seat's connection gets, in the same order, numbered in its own sequence; a
//! restart ends the watch as the seats' and a spectator watches again, numbered afresh, the game
//! taking spectators as its journal says it started. The setting: closed by default, the host's
//! `SetSpectators` until the start (the lobby lists it), the server's limit, a disconnect freeing
//! a place; a watcher takes no seat and a seat watches nothing; unknown games, a battle and a
//! paused game watched by nobody; the host's pause, an abandoned game and a server that takes no
//! spectators. The void's goodbye is in `whole.rs` (T20's test), the end-to-end spectator of a
//! timed game in `clock.rs` (T24).

use h3_protocol::wire::game::{GameCommand, GameQuery, GameReject, MapGameInfo};
use h3_protocol::wire::{ClientMessage, GoodbyeReason, Phase, Refusal, Seat, ServerMessage};

use super::whole::{
    game_joined, goodbye_after, listed_until, map_listed, map_listed_as, next, with_map,
};
use super::{fast, greeted};
use crate::client::Client;
use crate::config::{Config, Limits};
use crate::journal::whole as journal;
use crate::{Running, start};

/// A whole game of the duel map created by `client`, its first player's seat taken, open to
/// spectators as `spectators` says: the game and the seat's token.
async fn created(client: &mut Client, spectators: Option<bool>) -> (u32, String) {
    let create = ClientMessage::CreateMapGame {
        seq: 0,
        offer: String::from("duel-map"),
        player: 0,
        spectators,
    };
    client.send(&create).await.unwrap();
    let Some(ServerMessage::GameCreated { game, .. }) = client.recv().await else {
        panic!("not created")
    };
    let token = game_joined(client, game, 0).await;
    (game, token)
}

/// The lobby's listing of the whole game `game`.
async fn listed(running: &Running, game: u32) -> MapGameInfo {
    let games = map_listed(running).await.1;
    games.into_iter().find(|g| g.game == game).unwrap()
}

/// Waits until the lobby lists the whole game `game` as taking spectators or not.
async fn listed_taking(running: &Running, game: u32, spectators: bool) {
    listed_until(running, |games| {
        games
            .iter()
            .any(|g| g.game == game && g.spectators == spectators)
    })
    .await;
}

/// The messages `client` gets until its seat's answer to a query numbered `seq` (sent now), the
/// answer left out: every message the game queued for it before (the answer is no player's
/// message, and the spectators do not see it).
async fn until_answer(client: &mut Client, seq: u32) -> Vec<ServerMessage> {
    let query = ClientMessage::GameQuery {
        seq,
        query: GameQuery::Moves { hero: 0 },
    };
    client.send(&query).await.unwrap();
    let mut seen = Vec::new();
    loop {
        match client.recv().await.expect("the connection closed") {
            ServerMessage::GameAnswer { seq: s, .. } if s == seq => return seen,
            message => seen.push(message),
        }
    }
}

/// The messages the spectator `client` gets until the rejection of its query numbered `seq`
/// (sent now; a spectator asks nothing: `NotSeated`), the rejection left out.
async fn until_unseated(client: &mut Client, seq: u32) -> Vec<ServerMessage> {
    let query = ClientMessage::GameQuery {
        seq,
        query: GameQuery::Moves { hero: 0 },
    };
    client.send(&query).await.unwrap();
    let unseated = ServerMessage::GameRejected {
        n: 0,
        seq,
        code: GameReject::NotSeated {},
    };
    let mut seen = Vec::new();
    loop {
        match client.recv().await.expect("the connection closed") {
            message if message == unseated => return seen,
            message => seen.push(message),
        }
    }
}

/// `client` watches the whole game `game`: its `GameWatching`, then the watch's start (every
/// player's view) up to the rejection of its query numbered `seq`.
pub(super) async fn watch(client: &mut Client, game: u32, seq: u32) -> Vec<ServerMessage> {
    client
        .send(&ClientMessage::GameWatch { game })
        .await
        .unwrap();
    let watching = client.recv().await;
    assert!(
        matches!(watching, Some(ServerMessage::GameWatching { game: g, .. }) if g == game),
        "{watching:?}"
    );
    until_unseated(client, seq).await
}

/// The refusal of `client`'s watch of game `game`.
async fn refused_watch(client: &mut Client, game: u32) -> Refusal {
    client
        .send(&ClientMessage::GameWatch { game })
        .await
        .unwrap();
    match client.recv().await {
        Some(ServerMessage::Refused { reason }) => reason,
        other => panic!("{other:?}"),
    }
}

/// Player `player`'s messages among the spectator's `seen`, each as the player's seat got it.
pub(super) fn watched(seen: &[ServerMessage], player: u8) -> Vec<ServerMessage> {
    let of = |m: &ServerMessage| match m {
        ServerMessage::GameWatched {
            player: p, message, ..
        } if *p == player => Some((**message).clone()),
        _ => None,
    };
    seen.iter().filter_map(of).collect()
}

/// The spectator's own numbers of its messages `seen`, which are all `GameWatched`.
pub(super) fn numbers(seen: &[ServerMessage]) -> Vec<u32> {
    let n = |m: &ServerMessage| match m {
        ServerMessage::GameWatched { n, .. } => *n,
        other => panic!("not watched: {other:?}"),
    };
    seen.iter().map(n).collect()
}

/// Whether `messages` begin with a `GameSnapshot`.
fn begins_with_snapshot(messages: &[ServerMessage]) -> bool {
    matches!(messages.first(), Some(ServerMessage::GameSnapshot { .. }))
}

/// Whether `messages` hold a `GameTurn`.
fn has_turn(messages: &[ServerMessage]) -> bool {
    let turn = |m: &ServerMessage| matches!(m, ServerMessage::GameTurn { .. });
    messages.iter().any(turn)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_spectator_sees_every_seats_messages_and_watches_again_after_a_restart() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let mut a = greeted(&running).await;
    let (game, t0) = created(&mut a, Some(true)).await;
    listed_taking(&running, game, true).await;
    // Watched before the second seat is taken: each player's view, numbered from 1 in the
    // spectator's own sequence.
    let mut s = greeted(&running).await;
    let start_ = watch(&mut s, game, 1).await;
    let k = u32::try_from(start_.len()).unwrap();
    assert_eq!(numbers(&start_), (1..=k).collect::<Vec<_>>());
    assert!(begins_with_snapshot(&watched(&start_, 0)));
    assert!(begins_with_snapshot(&watched(&start_, 1)));
    assert!(!has_turn(&start_), "the game waits");
    // The second seat's join starts the game: every message either seat's connection gets is the
    // spectator's too, in the same order, numbered on.
    let mut b = greeted(&running).await;
    let join = ClientMessage::GameJoin {
        game,
        player: 1,
        token: None,
    };
    b.send(&join).await.unwrap();
    let joined = b.recv().await;
    assert!(
        matches!(joined, Some(ServerMessage::GameJoined { player: 1, .. })),
        "{joined:?}"
    );
    let of_b = until_answer(&mut b, 1).await;
    let of_a = until_answer(&mut a, 1).await;
    let seen = until_unseated(&mut s, 2).await;
    assert_eq!(watched(&seen, 0), of_a);
    assert_eq!(watched(&seen, 1), of_b);
    assert!(begins_with_snapshot(&of_b) && has_turn(&of_a));
    let m = u32::try_from(seen.len()).unwrap();
    assert_eq!(numbers(&seen), (k + 1..=k + m).collect::<Vec<_>>());
    // A command's messages too.
    let end_turn = ClientMessage::GameCommand {
        seq: 2,
        command: GameCommand::EndTurn {},
    };
    a.send(&end_turn).await.unwrap();
    let of_a = until_answer(&mut a, 3).await;
    let of_b = until_answer(&mut b, 3).await;
    let seen = until_unseated(&mut s, 3).await;
    assert!(
        (of_a.iter()).any(|m| matches!(m, ServerMessage::GameAccepted { seq: 2, .. })),
        "{of_a:?}"
    );
    assert_eq!(watched(&seen, 0), of_a);
    assert_eq!(watched(&seen, 1), of_b);
    assert_eq!(numbers(&seen)[0], k + m + 1);
    // The stop ends the watch as the seats' play; the game started open, as its journal says.
    running.shutdown().await;
    for client in [&mut a, &mut b, &mut s] {
        assert_eq!(goodbye_after(client).await, GoodbyeReason::Shutdown);
    }
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    let entries = journal::read(&path).unwrap().entries;
    let at = (entries.iter())
        .position(|e| *e == h3_protocol::session::game::GameEntry::Started)
        .unwrap();
    assert_eq!(
        entries[at - 1],
        h3_protocol::session::game::GameEntry::Watchable
    );
    // Restored, it is listed as taking spectators, and one watches again, numbered afresh: the
    // acting player's view with its turn.
    let again = start(&config).await.unwrap();
    listed_until(&again, |games| {
        (games.iter()).any(|g| g.game == game && g.phase == Phase::Running && g.spectators)
    })
    .await;
    let mut s = greeted(&again).await;
    let start_ = watch(&mut s, game, 1).await;
    let k = u32::try_from(start_.len()).unwrap();
    assert_eq!(numbers(&start_), (1..=k).collect::<Vec<_>>());
    let (view_a, view_b) = (watched(&start_, 0), watched(&start_, 1));
    assert!(begins_with_snapshot(&view_a) && begins_with_snapshot(&view_b));
    assert!(
        !has_turn(&view_a) && has_turn(&view_b),
        "the second player's turn"
    );
    // A seat re-taken: its messages are the spectator's too.
    let mut a = greeted(&again).await;
    let rejoin = ClientMessage::GameJoin {
        game,
        player: 0,
        token: Some(t0),
    };
    a.send(&rejoin).await.unwrap();
    let joined = a.recv().await;
    assert!(
        matches!(joined, Some(ServerMessage::GameJoined { player: 0, .. })),
        "{joined:?}"
    );
    let of_a = until_answer(&mut a, 4).await;
    let seen = until_unseated(&mut s, 2).await;
    assert!(begins_with_snapshot(&of_a));
    assert_eq!(watched(&seen, 0), of_a);
    assert_eq!(numbers(&seen)[0], k + 1);
    again.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_whole_games_host_opens_and_closes_it_to_spectators_within_the_servers_limit() {
    let config = Config {
        limits: Limits {
            spectators_per_game: 1,
            ..fast()
        },
        ..with_map()
    };
    let running = start(&config).await.unwrap();
    // Closed by default: listed so, a watch refused (it leaves the game).
    let mut a = greeted(&running).await;
    let (game, _) = created(&mut a, None).await;
    assert!(!listed(&running, game).await.spectators);
    let mut s = greeted(&running).await;
    assert_eq!(refused_watch(&mut s, game).await, Refusal::NoSpectators);
    // Its host opens it: listed so, one spectator watches, a second is refused (the limit).
    let set = |seq, open| ClientMessage::SetSpectators { seq, open };
    let told = |seq, open| {
        move |m| {
            matches!(m, ServerMessage::SpectatorsSet { seq: s, open: o, .. } if s == seq && o == open)
                .then_some(())
        }
    };
    a.send(&set(1, true)).await.unwrap();
    next(&mut a, told(1, true)).await;
    listed_taking(&running, game, true).await;
    watch(&mut s, game, 1).await;
    let mut t = greeted(&running).await;
    assert_eq!(refused_watch(&mut t, game).await, Refusal::NoSpectators);
    // Closing it ends the watch, and it is listed closed.
    a.send(&set(2, false)).await.unwrap();
    assert_eq!(goodbye_after(&mut s).await, GoodbyeReason::SpectatorsClosed);
    listed_taking(&running, game, false).await;
    // Opened again: a spectator's disconnect frees its place.
    a.send(&set(3, true)).await.unwrap();
    next(&mut a, told(3, true)).await;
    watch(&mut t, game, 1).await;
    t.close().await;
    let mut u = greeted(&running).await;
    let mut tries = 0;
    loop {
        u.send(&ClientMessage::GameWatch { game }).await.unwrap();
        match u.recv().await {
            Some(ServerMessage::GameWatching { .. }) => break,
            Some(ServerMessage::Refused {
                reason: Refusal::NoSpectators,
            }) if tries < 250 => {
                tries += 1;
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            other => panic!("{other:?}"),
        }
    }
    until_unseated(&mut u, 1).await;
    // The second seat starts it, listed as taking spectators still. A spectator takes no seat, a
    // seat watches nothing; only the host sets it, and only before the start.
    let mut b = greeted(&running).await;
    let join = |player, token| ClientMessage::GameJoin {
        game,
        player,
        token,
    };
    b.send(&join(1, None)).await.unwrap();
    let t1 = game_joined(&mut b, game, 1).await;
    listed_until(&running, |games| {
        (games.iter()).any(|g| g.game == game && g.phase == Phase::Running && g.spectators)
    })
    .await;
    u.send(&join(1, Some(t1))).await.unwrap();
    let already = |m| {
        matches!(
            m,
            ServerMessage::Refused {
                reason: Refusal::AlreadyJoined
            }
        )
        .then_some(())
    };
    next(&mut u, already).await;
    a.send(&ClientMessage::GameWatch { game }).await.unwrap();
    next(&mut a, already).await;
    let rejected = |seq, code: GameReject| {
        move |m| match m {
            ServerMessage::GameRejected {
                seq: s, code: c, ..
            } if s == seq => {
                assert_eq!(c, code);
                Some(())
            }
            _ => None,
        }
    };
    b.send(&set(1, false)).await.unwrap();
    next(&mut b, rejected(1, GameReject::NotHost {})).await;
    a.send(&set(4, false)).await.unwrap();
    next(&mut a, rejected(4, GameReject::AlreadyStarted {})).await;
    // No other game is watched: an unknown one, a battle.
    let mut v = greeted(&running).await;
    assert_eq!(refused_watch(&mut v, 999).await, Refusal::NoSuchGame);
    let mut c = greeted(&running).await;
    c.create("pvp", Seat::Attacker).await.unwrap();
    let Some(ServerMessage::GameCreated { game: battle, .. }) = c.recv().await else {
        panic!("no battle")
    };
    assert_eq!(refused_watch(&mut v, battle).await, Refusal::NoSuchGame);
    // The host's pause ends the watch as the seats' play; a paused game is listed as taking none,
    // and nobody watches it.
    a.send(&ClientMessage::Pause { seq: 5 }).await.unwrap();
    for client in [&mut a, &mut b, &mut u] {
        assert_eq!(goodbye_after(client).await, GoodbyeReason::Paused);
    }
    map_listed_as(&running, game, Some(Phase::Paused)).await;
    assert!(!listed(&running, game).await.spectators);
    assert_eq!(refused_watch(&mut v, game).await, Refusal::NoSpectators);
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_abandoned_whole_game_ends_its_watch_and_a_server_without_spectators_lists_none() {
    // A server that takes no spectators lists a game its creator opened as taking none.
    let config = Config {
        limits: Limits {
            spectators_per_game: 0,
            ..fast()
        },
        ..with_map()
    };
    let running = start(&config).await.unwrap();
    let mut a = greeted(&running).await;
    let (game, _) = created(&mut a, Some(true)).await;
    assert!(!listed(&running, game).await.spectators);
    let mut s = greeted(&running).await;
    assert_eq!(refused_watch(&mut s, game).await, Refusal::NoSpectators);
    running.shutdown().await;
    // Its creator gone before the start, the game is abandoned: its spectator hears it.
    let running = start(&with_map()).await.unwrap();
    let mut a = greeted(&running).await;
    let (game, _) = created(&mut a, Some(true)).await;
    let mut s = greeted(&running).await;
    watch(&mut s, game, 1).await;
    a.send(&ClientMessage::Leave {}).await.unwrap();
    assert_eq!(goodbye_after(&mut a).await, GoodbyeReason::Left);
    assert_eq!(goodbye_after(&mut s).await, GoodbyeReason::Abandoned);
    map_listed_as(&running, game, None).await;
    running.shutdown().await;
}
