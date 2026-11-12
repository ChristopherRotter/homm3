//! The whole game's seats and connections (E16 M5 T8a): a game hosted for connections starts when
//! every seat is held, and its connections then hear what the in-process host's players hear (a
//! golden replay beside its in-process twin, frame for frame and record for record); a seat is
//! taken by token or by account and re-taken after a disconnect with a fresh snapshot, its record
//! going on meanwhile; a join or a command out of place is refused or rejected and changes
//! nothing.

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, GameSetup, GameSetupError};

use super::game_fold::one;
use super::game_session::duel;
use crate::codec;
use crate::map::game::{command, manifest, observe};
use crate::session::game::{GameEntry, GameOptions, GameOutbound, GameSession};
use crate::session::{ConnId, Outbound};
use crate::wire::game::{GameCommand, GameReject, MapIdentity};
use crate::wire::{
    ClientMessage, Command, GoodbyeReason, Refusal, RejectCode, Seat, ServerMessage, WireU64,
};

/// A map's identity for the manifest (the session only carries it).
pub(super) const MAP: MapIdentity = MapIdentity {
    content_hash: WireU64(7),
    import_version: 1,
};

/// Game 9, no checkpoints, every record unbounded.
pub(super) const OPTIONS: GameOptions = GameOptions {
    game: 9,
    checkpoint_every: 0,
    record_bytes: usize::MAX,
};

/// The connections a test uses.
const A: ConnId = ConnId(1);
const C: ConnId = ConnId(3);
const D: ConnId = ConnId(4);
const F: ConnId = ConnId(6);

/// The token the host drew for `player`'s seat.
pub(super) fn token(player: PlayerId) -> String {
    std::format!("token-{}", player.raw())
}

/// `GameJoin` of game 9's `player` with `token`.
pub(super) fn join(player: u8, token: Option<&str>) -> ClientMessage {
    ClientMessage::GameJoin {
        game: 9,
        player,
        token: token.map(str::to_string),
    }
}

/// `GameCommand` `EndTurn` numbered `seq`.
pub(super) const fn end_turn(seq: u32) -> ClientMessage {
    ClientMessage::GameCommand {
        seq,
        command: GameCommand::EndTurn {},
    }
}

/// What `conn` sends, and the frames that come out.
pub(super) fn send(s: &mut GameSession<'_>, conn: ConnId, message: ClientMessage) -> Vec<Outbound> {
    let mut out = Vec::new();
    s.handle(conn, message, &mut out);
    out
}

/// The messages in `out` for `conn`, decoded, in order.
pub(super) fn to(out: &[Outbound], conn: ConnId) -> Vec<ServerMessage> {
    let frames = out.iter().filter(|o| o.to == conn);
    frames
        .map(|o| codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap())
        .collect()
}

/// `GameJoined` for game 9's `player` with `token`, its seat's last number none.
fn joined(s: &GameSession<'_>, player: u8, token: Option<String>) -> ServerMessage {
    joined_after(s, player, token, None)
}

/// `GameJoined` for game 9's `player` with `token`, its seat's last number `last` (E16 M5 T8b).
pub(super) fn joined_after(
    s: &GameSession<'_>,
    player: u8,
    token: Option<String>,
    last: Option<u32>,
) -> ServerMessage {
    ServerMessage::GameJoined {
        game: 9,
        player,
        token,
        manifest: s.manifest().clone(),
        last,
    }
}

/// The connection of player `index` in the golden run.
const fn conn(index: usize) -> ConnId {
    ConnId(100 + index as u64)
}

/// Each player's frames in `out` (the in-process twin's), appended to `frames`.
fn by_player(out: &[GameOutbound], frames: &mut [Vec<String>]) {
    for o in out {
        frames[o.to.index()].push(o.frame.clone());
    }
}

/// Each player's connection's frames in `out`, appended to `frames`; none closes.
fn by_conn(out: &[Outbound], frames: &mut [Vec<String>]) {
    for o in out {
        assert!(!o.close);
        let index = usize::try_from(o.to.0 - 100).unwrap();
        frames[index].push(o.frame.clone());
    }
}

#[test]
fn connections_hear_what_the_in_process_hosts_players_hear() {
    let (rules, setup, replay) = one("towns-s173-d21");
    let (mut twin_out, mut out) = (Vec::new(), Vec::new());
    let seed = replay.seed;
    let mut twin = GameSession::start(
        &rules,
        &setup,
        seed,
        manifest(&rules, MAP),
        OPTIONS,
        &mut twin_out,
    )
    .unwrap();
    let mut net =
        GameSession::new(&rules, &setup, seed, manifest(&rules, MAP), OPTIONS, token).unwrap();
    let players = twin.state().players();
    assert!(players >= 2);
    let (mut heard, mut told) = (vec![Vec::new(); players], vec![Vec::new(); players]);
    by_player(&twin_out, &mut told);
    // Each player's connection takes its seat in turn: `GameJoined` with its token, then its
    // snapshot; the last one held starts the game, every player that may act told its turn.
    for (index, player) in (0..=u8::MAX).take(players).enumerate() {
        assert!(!net.started());
        out.clear();
        net.handle(conn(index), join(player, None), &mut out);
        let token = Some(token(PlayerId::new(player)));
        assert_eq!(to(&out[..1], conn(index)), [joined(&net, player, token)]);
        by_conn(&out[1..], &mut heard);
    }
    assert!(net.started());
    assert_eq!(heard, told);
    // Every command as the twin's: a player's from its seat's connection, the system actor's
    // through the host, its frames routed to the seats' connections.
    let mut systems = 0;
    for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
        let (seq, wire) = (u32::try_from(seq).unwrap(), command(engine));
        out.clear();
        twin_out.clear();
        match *actor {
            Actor::Player(player) => {
                let message = ClientMessage::GameCommand {
                    seq,
                    command: wire.clone(),
                };
                net.handle(conn(player.index()), message, &mut out);
                twin.play(player, seq, wire, &mut twin_out).unwrap();
            }
            Actor::System => {
                let mut frames = Vec::new();
                net.system(wire.clone(), &mut frames).unwrap();
                net.route(frames, &mut out);
                twin.system(wire, &mut twin_out).unwrap();
                systems += 1;
            }
        }
        by_conn(&out, &mut heard);
        by_player(&twin_out, &mut told);
        assert_eq!(heard, told, "command {seq}");
    }
    assert!(systems > 0);
    assert_eq!(net.state().state_hash(), twin.state().state_hash());
    assert_eq!(net.journal(), twin.journal());
    for player in (0..=u8::MAX).take(players).map(PlayerId::new) {
        assert_eq!(net.record(player), twin.record(player));
    }
    // Its journal, the seats' entries and the numbered commands, restores the game with its seats
    // taken, none held (E16 M5 T8b).
    let entries: Vec<GameEntry> = net.drain_entries().collect();
    let numbered = entries.iter().filter(|e| matches!(e, GameEntry::Numbered(..)));
    assert_eq!(numbered.count(), replay.commands.len() - systems);
    let manifest = manifest(&rules, MAP);
    let back = GameSession::reopen(&rules, &setup, seed, manifest, OPTIONS, token, &entries);
    let back = back.unwrap();
    assert!(back.started() && !back.paused());
    assert_eq!(back.state().state_hash(), twin.state().state_hash());
    assert_eq!(back.journal(), twin.journal());
}

#[test]
fn a_seat_is_taken_by_token_or_account_and_retaken_with_a_fresh_snapshot() {
    let (rules, setup) = duel();
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), OPTIONS, token).unwrap();
    let acting = s.state().turn_group()[0];
    let waiting = PlayerId::new(u8::from(acting.raw() == 0));
    // An anonymous connection takes the waiting player's seat and gets its token; one logged in
    // as an account the acting player's, which becomes the account's: no token handed out.
    let out = send(&mut s, A, join(waiting.raw(), None));
    assert_eq!(
        to(&out, A)[0],
        joined(&s, waiting.raw(), Some(token(waiting)))
    );
    s.logged_in(C, "ann".to_string());
    let out = send(&mut s, C, join(acting.raw(), None));
    assert_eq!(to(&out, C)[..1], [joined(&s, acting.raw(), None)]);
    assert!(s.started());
    // The account's seat is its account's alone: its token does not take it.
    let out = send(&mut s, D, join(acting.raw(), Some(&token(acting))));
    let taken = ServerMessage::Refused {
        reason: Refusal::SeatTaken,
    };
    assert_eq!(to(&out, D), std::slice::from_ref(&taken));
    // The waiting player's connection is gone: the acting player's command is told to the
    // connections that hold seats, and the gone one's record goes on.
    s.disconnected(A);
    let before = s.record(waiting).unwrap().lines().count();
    let out = send(&mut s, C, end_turn(1));
    assert!(out.iter().all(|o| o.to == C));
    assert!(matches!(
        to(&out, C)[0],
        ServerMessage::GameAccepted { seq: 1, .. }
    ));
    let record = s.record(waiting).unwrap();
    assert!(record.lines().count() > before);
    let last = u32::try_from(record.lines().count() - 1).unwrap();
    // Re-taken: without its token or with another, refused; with it, the seat is back with a
    // fresh snapshot, numbered on, and its turn, as it may act now.
    assert_eq!(to(&send(&mut s, D, join(waiting.raw(), None)), D), [taken]);
    let bad = ServerMessage::Refused {
        reason: Refusal::BadToken,
    };
    assert_eq!(
        to(&send(&mut s, D, join(waiting.raw(), Some("x"))), D),
        [bad]
    );
    let out = send(&mut s, D, join(waiting.raw(), Some(&token(waiting))));
    let observation = observe(s.state(), rules, h3_game::Viewer::Player(waiting));
    let told = to(&out, D);
    assert_eq!(
        told[..2],
        [
            joined(&s, waiting.raw(), Some(token(waiting))),
            ServerMessage::GameSnapshot {
                n: last + 1,
                observation
            }
        ]
    );
    assert!(matches!(told[2..], [ServerMessage::GameTurn { n, .. }] if n == last + 2));
    // Another connection of the account re-takes its seat and replaces the one that holds it;
    // the acting player may not act now, so no turn.
    s.logged_in(F, "ann".to_string());
    let out = send(&mut s, F, join(acting.raw(), None));
    assert_eq!(
        out[0],
        Outbound {
            to: C,
            frame: codec::encode_text(&ServerMessage::Goodbye {
                reason: GoodbyeReason::Replaced
            }),
            close: true,
        }
    );
    let told = to(&out, F);
    assert_eq!(told[0], joined_after(&s, acting.raw(), None, Some(1)));
    assert!(matches!(told[1..], [ServerMessage::GameSnapshot { .. }]));
    // Leaving frees the seat and closes; its command then comes from no seat.
    let left = send(&mut s, F, ClientMessage::Leave {});
    assert!(left[0].close);
    let unseated = ServerMessage::GameRejected {
        n: 0,
        seq: 2,
        code: GameReject::NotSeated {},
    };
    assert_eq!(to(&send(&mut s, F, end_turn(2)), F), [unseated]);
}

#[test]
fn a_join_or_a_command_out_of_place_changes_nothing() {
    let (rules, setup) = duel();
    // A setup that starts no game hosts none.
    let none = GameSetup {
        difficulty: 200,
        ..setup.clone()
    };
    let hosted = GameSession::new(rules, &none, 3, manifest(rules, MAP), OPTIONS, token);
    assert_eq!(hosted.map(|_| ()), Err(GameSetupError::Difficulty(200)));
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), OPTIONS, token).unwrap();
    let hash = s.state().state_hash();
    let refused = |reason| vec![ServerMessage::Refused { reason }];
    let elsewhere = ClientMessage::GameJoin {
        game: 8,
        player: 0,
        token: None,
    };
    assert_eq!(
        to(&send(&mut s, A, elsewhere), A),
        refused(Refusal::NoSuchGame)
    );
    assert_eq!(
        to(&send(&mut s, A, join(2, None)), A),
        refused(Refusal::NoSuchSeat)
    );
    let unseated = |seq| {
        vec![ServerMessage::GameRejected {
            n: 0,
            seq,
            code: GameReject::NotSeated {},
        }]
    };
    assert_eq!(to(&send(&mut s, A, end_turn(1)), A), unseated(1));
    // A seat taken before the start: one seat a connection, and no command yet; its first take
    // an entry (E16 M5 T8b).
    assert_eq!(to(&send(&mut s, A, join(0, None)), A).len(), 2);
    let taken: Vec<GameEntry> = s.drain_entries().collect();
    assert_eq!(taken, [GameEntry::Taken(PlayerId::new(0))]);
    assert_eq!(
        to(&send(&mut s, A, join(1, None)), A),
        refused(Refusal::AlreadyJoined)
    );
    assert!(!s.started());
    let not_started = ServerMessage::GameRejected {
        n: 2,
        seq: 2,
        code: GameReject::NotStarted {},
    };
    assert_eq!(to(&send(&mut s, A, end_turn(2)), A), [not_started]);
    let mut frames = Vec::new();
    let system = s.system(GameCommand::EndTurn {}, &mut frames);
    assert_eq!(
        (system, frames),
        (Err(GameReject::NotStarted {}), Vec::new())
    );
    // A battle's join and command; the connection's messages, a battle's others.
    let battle = ClientMessage::Join {
        game: 9,
        seat: Seat::Attacker,
        token: None,
    };
    assert_eq!(
        to(&send(&mut s, A, battle), A),
        refused(Refusal::NoSuchGame)
    );
    let defend = ClientMessage::Command {
        seq: 3,
        command: Command::Defend {},
    };
    let rejected = ServerMessage::Rejected {
        n: 0,
        seq: 3,
        code: RejectCode::NotSeated,
    };
    assert_eq!(to(&send(&mut s, A, defend), A), [rejected]);
    for message in [
        ClientMessage::Ping { nonce: 1 },
        ClientMessage::FetchRecord { seq: 4 },
    ] {
        assert!(send(&mut s, A, message).is_empty());
    }
    assert_eq!(s.state().state_hash(), hash);
    assert_eq!(s.drain_entries().count(), 0);
    assert_eq!(s.record(PlayerId::new(0)).unwrap().lines().count(), 3);
    // In-process, the host holds every seat: none to take, no connection's command.
    let mut out = Vec::new();
    let manifest = manifest(rules, MAP);
    let mut host = GameSession::start(rules, &setup, 3, manifest, OPTIONS, &mut out).unwrap();
    assert!(host.started());
    let hash = host.state().state_hash();
    assert_eq!(
        to(&send(&mut host, A, join(0, None)), A),
        refused(Refusal::SeatTaken)
    );
    assert_eq!(to(&send(&mut host, A, end_turn(5)), A), unseated(5));
    host.disconnected(A);
    assert_eq!(host.state().state_hash(), hash);
}
