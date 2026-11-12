//! A whole game's command numbers, the host's pause and its journal of seats (E16 M5 T8b): a
//! resend applies nothing and is answered as the first time while its seat keeps that answer,
//! else `Stale`, and a rejected command takes no number; the host pauses a running game, another
//! seat's ask is told to every player once until the game moves on, and a seat's join resumes it;
//! a game of connections reopens from its journal with its seats taken, none held, its numbers
//! kept.

use std::string::ToString;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{GameCommand as Engine, GameSetup};

use super::game_seats::{MAP, OPTIONS, end_turn, join, joined_after, send, to, token};
use super::game_session::duel;
use crate::map::game::manifest;
use crate::session::ConnId;
use crate::session::game::{GameEntry, GameRestoreError, GameSession, KEPT_ANSWERS};
use crate::wire::game::{GameCommand, GameReject};
use crate::wire::{ClientMessage, GoodbyeReason, Refusal, ServerMessage};

const A: ConnId = ConnId(1);
const C: ConnId = ConnId(3);
const D: ConnId = ConnId(4);
const E: ConnId = ConnId(5);

/// `Pause` numbered `seq`.
const fn pause(seq: u32) -> ClientMessage {
    ClientMessage::Pause { seq }
}

/// The duel's other player.
fn other(player: PlayerId) -> PlayerId {
    PlayerId::new(u8::from(player.raw() == 0))
}

/// Whether `told` starts with `GameAccepted` of `EndTurn` numbered `seq`.
fn accepted(told: &[ServerMessage], seq: u32) -> bool {
    matches!(
        told,
        [ServerMessage::GameAccepted { seq: s, command: GameCommand::EndTurn {}, .. }, ..]
            if *s == seq
    )
}

/// Whether `told` is `GameRejected` of `seq` with `code` alone.
fn rejected(told: &[ServerMessage], seq: u32, code: GameReject) -> bool {
    matches!(
        told,
        [ServerMessage::GameRejected { seq: s, code: c, .. }] if *s == seq && *c == code
    )
}

/// `Refused` with `reason`, alone.
fn refused(reason: Refusal) -> Vec<ServerMessage> {
    std::vec![ServerMessage::Refused { reason }]
}

#[test]
fn a_resend_applies_nothing_and_is_answered_as_the_first_time() {
    let (rules, setup) = duel();
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), OPTIONS, token).unwrap();
    let acting = s.state().turn_group()[0];
    let waiting = other(acting);
    send(&mut s, C, join(acting.raw(), None));
    send(&mut s, A, join(waiting.raw(), None));
    assert!(s.started());
    assert!(s.drain_entries().count() > 0);
    // A command numbered 5: accepted and journaled with its number.
    assert!(accepted(&to(&send(&mut s, C, end_turn(5)), C), 5));
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    assert_eq!(entries, [GameEntry::Numbered(acting, 5, Engine::EndTurn)]);
    let hash = s.state().state_hash();
    // Sent again, as a client resends after a reconnect: nothing applied, nothing journaled, its
    // seat alone answered as the first time, numbered on.
    let out = send(&mut s, C, end_turn(5));
    assert!(out.iter().all(|o| o.to == C));
    assert!(matches!(
        to(&out, C)[..],
        [ServerMessage::GameAccepted {
            seq: 5,
            command: GameCommand::EndTurn {},
            ..
        }]
    ));
    assert_eq!(s.state().state_hash(), hash);
    assert_eq!(s.drain_entries().count(), 0);
    // A number below the last that no accepted command took (a gap) has no answer kept: stale.
    let stale = GameReject::Stale {};
    assert!(rejected(&to(&send(&mut s, C, end_turn(3)), C), 3, stale));
    // A rejected command takes no number: 6 out of its turn is the engine's rejection, then 6 in
    // its turn is accepted.
    let out = send(&mut s, C, end_turn(6));
    assert!(matches!(
        to(&out, C)[..],
        [ServerMessage::GameRejected {
            seq: 6,
            code: GameReject::NotYourTurn { .. },
            ..
        }]
    ));
    assert_eq!(s.state().state_hash(), hash);
    assert!(accepted(&to(&send(&mut s, A, end_turn(1)), A), 1));
    assert!(accepted(&to(&send(&mut s, C, end_turn(6)), C), 6));
    // A seat keeps its last `KEPT_ANSWERS` answers: an older command's resend is stale.
    for round in 0..u32::try_from(KEPT_ANSWERS).unwrap() {
        assert!(accepted(
            &to(&send(&mut s, A, end_turn(2 + round)), A),
            2 + round
        ));
        assert!(accepted(
            &to(&send(&mut s, C, end_turn(7 + round)), C),
            7 + round
        ));
    }
    let hash = s.state().state_hash();
    assert!(rejected(&to(&send(&mut s, C, end_turn(6)), C), 6, stale));
    let out = send(&mut s, C, end_turn(7));
    assert!(matches!(
        to(&out, C)[..],
        [ServerMessage::GameAccepted { seq: 7, .. }]
    ));
    assert_eq!(s.state().state_hash(), hash);
}

#[test]
fn the_host_pauses_another_seat_asks_and_a_join_resumes() {
    let (rules, setup) = duel();
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), OPTIONS, token).unwrap();
    let acting = s.state().turn_group()[0];
    let waiting = other(acting);
    // A takes the waiting player's seat first: the host's. Before the start no seat pauses, and a
    // connection without a seat never.
    send(&mut s, A, join(waiting.raw(), None));
    let not_started = GameReject::NotStarted {};
    assert!(rejected(&to(&send(&mut s, A, pause(1)), A), 1, not_started));
    assert!(matches!(
        to(&send(&mut s, D, pause(2)), D)[..],
        [ServerMessage::GameRejected {
            n: 0,
            seq: 2,
            code: GameReject::NotSeated {},
        }]
    ));
    send(&mut s, C, join(acting.raw(), None));
    assert!(s.started());
    // Another seat's ask is told to every player, once until the game moves on.
    let out = send(&mut s, C, pause(3));
    for conn in [A, C] {
        assert!(matches!(
            to(&out, conn)[..],
            [ServerMessage::GamePauseAsked { player, .. }] if player == acting.raw()
        ));
    }
    let again = GameReject::AlreadyAsked {};
    assert!(rejected(&to(&send(&mut s, C, pause(4)), C), 4, again));
    assert!(accepted(&to(&send(&mut s, C, end_turn(5)), C), 5));
    let out = send(&mut s, C, pause(6));
    assert!(matches!(
        to(&out, A)[..],
        [ServerMessage::GamePauseAsked { .. }]
    ));
    // The host's pause: an entry, every holder told `Goodbye { Paused }` and closed; the game
    // then applies no command, the system actor's neither.
    assert!(s.drain_entries().count() > 0);
    let out = send(&mut s, A, pause(7));
    let paused = ServerMessage::Goodbye {
        reason: GoodbyeReason::Paused,
    };
    assert_eq!(to(&out, A), std::slice::from_ref(&paused));
    assert_eq!(to(&out, C), [paused]);
    assert!(out.iter().all(|o| o.close));
    assert!(s.paused());
    assert_eq!(s.drain_entries().collect::<Vec<_>>(), [GameEntry::Paused]);
    let unseated = GameReject::NotSeated {};
    assert!(rejected(&to(&send(&mut s, C, end_turn(8)), C), 8, unseated));
    let mut frames = Vec::new();
    let system = s.system(GameCommand::EndTurn {}, &mut frames);
    assert_eq!((system, frames), (Err(not_started), Vec::new()));
    // A seat's join resumes it, an entry: the seat back with its last number.
    let out = send(&mut s, C, join(acting.raw(), Some(&token(acting))));
    assert!(!s.paused());
    assert_eq!(s.drain_entries().collect::<Vec<_>>(), [GameEntry::Resumed]);
    let back = joined_after(&s, acting.raw(), Some(token(acting)), Some(5));
    assert_eq!(to(&out, C)[0], back);
    // The game runs again: the host's seat re-taken, its first command accepted.
    send(&mut s, A, join(waiting.raw(), Some(&token(waiting))));
    assert!(accepted(&to(&send(&mut s, A, end_turn(1)), A), 1));
}

#[test]
fn a_game_of_connections_reopens_with_its_seats_taken_and_its_numbers_kept() {
    let (rules, setup) = duel();
    let host = || GameSession::new(rules, &setup, 3, manifest(rules, MAP), OPTIONS, token);
    let mut s = host().unwrap();
    let acting = s.state().turn_group()[0];
    let waiting = other(acting);
    // The account's seat first (the host's), the anonymous one second; a command, a pause, a
    // resume, a command, the host back, the other seat gone, a pause (told to the host alone).
    s.logged_in(A, "ann".to_string());
    send(&mut s, A, join(acting.raw(), None));
    send(&mut s, C, join(waiting.raw(), None));
    assert!(accepted(&to(&send(&mut s, A, end_turn(4)), A), 4));
    send(&mut s, A, pause(5));
    send(&mut s, C, join(waiting.raw(), Some(&token(waiting))));
    assert!(accepted(&to(&send(&mut s, C, end_turn(1)), C), 1));
    send(&mut s, A, join(acting.raw(), None));
    s.disconnected(C);
    let out = send(&mut s, A, pause(6));
    assert!(out.len() == 1 && out[0].to == A && out[0].close);
    assert!(s.paused());
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    assert_eq!(
        entries,
        [
            GameEntry::TakenBy(acting, "ann".to_string()),
            GameEntry::Taken(waiting),
            GameEntry::Started,
            GameEntry::Numbered(acting, 4, Engine::EndTurn),
            GameEntry::Paused,
            GameEntry::Resumed,
            GameEntry::Numbered(waiting, 1, Engine::EndTurn),
            GameEntry::Paused,
        ]
    );
    // Reopened: the same game, paused, no entry lacking; each player's record begins again with
    // its snapshot, told to nobody.
    let reopen = |entries: &[GameEntry]| {
        GameSession::reopen(
            rules,
            &setup,
            3,
            manifest(rules, MAP),
            OPTIONS,
            token,
            entries,
        )
    };
    let mut back = reopen(&entries).unwrap();
    assert!(back.started() && back.paused());
    assert_eq!(back.state().state_hash(), s.state().state_hash());
    assert_eq!(back.journal(), s.journal());
    assert_eq!(back.drain_entries().count(), 0);
    assert!(back.record(acting).unwrap().lines().count() >= 2);
    // No seat held: the account's is re-taken by its account alone, the anonymous one with its
    // token; the first join resumes the game.
    let taken = refused(Refusal::SeatTaken);
    let by_token = join(acting.raw(), Some(&token(acting)));
    assert_eq!(to(&send(&mut back, D, by_token), D), taken);
    back.logged_in(D, "ann".to_string());
    let out = send(&mut back, D, join(acting.raw(), None));
    assert_eq!(
        to(&out, D)[0],
        joined_after(&back, acting.raw(), None, Some(4))
    );
    assert!(!back.paused());
    let resumed: Vec<GameEntry> = back.drain_entries().collect();
    assert_eq!(resumed, [GameEntry::Resumed]);
    assert_eq!(to(&send(&mut back, E, join(waiting.raw(), None)), E), taken);
    let out = send(&mut back, E, join(waiting.raw(), Some(&token(waiting))));
    let rejoined = joined_after(&back, waiting.raw(), Some(token(waiting)), Some(1));
    assert_eq!(to(&out, E)[0], rejoined);
    // The numbers survived the restart: a resend applies nothing, answered as the first time.
    let hash = back.state().state_hash();
    let out = send(&mut back, E, end_turn(1));
    assert!(matches!(
        to(&out, E)[..],
        [ServerMessage::GameAccepted { seq: 1, .. }]
    ));
    assert_eq!(back.state().state_hash(), hash);
    // A game that never started reopens unstarted, its seat taken: it starts when both are held.
    let mut early = host().unwrap();
    send(&mut early, A, join(0, None));
    let entries: Vec<GameEntry> = early.drain_entries().collect();
    let mut back = reopen(&entries).unwrap();
    assert!(!back.started() && !back.paused());
    assert_eq!(to(&send(&mut back, C, join(0, None)), C), taken);
    send(&mut back, C, join(1, None));
    assert!(!back.started());
    send(&mut back, A, join(0, Some(&token(PlayerId::new(0)))));
    assert!(back.started());
    // In-process, the host holds every seat: a journal of seats is not its game.
    let mut out = Vec::new();
    let restored = GameSession::restore(
        rules,
        &setup,
        3,
        manifest(rules, MAP),
        OPTIONS,
        &entries,
        &mut out,
    );
    assert!(matches!(restored, Err(GameRestoreError::Diverged { .. })));
    // A seat's numbered command re-runs in-process as its player's: the host vouches for it.
    let numbered = [GameEntry::Numbered(acting, 4, Engine::EndTurn)];
    let restored = GameSession::restore(
        rules,
        &setup,
        3,
        manifest(rules, MAP),
        OPTIONS,
        &numbered,
        &mut out,
    )
    .unwrap();
    let mut twin = GameSession::start(rules, &setup, 3, manifest(rules, MAP), OPTIONS, &mut out);
    let twin = twin.as_mut().unwrap();
    twin.play(acting, 4, GameCommand::EndTurn {}, &mut out)
        .unwrap();
    assert_eq!(restored.state().state_hash(), twin.state().state_hash());
    // A setup that starts no game reopens none.
    let none = GameSetup {
        difficulty: 200,
        ..setup.clone()
    };
    let hosted = GameSession::reopen(
        rules,
        &none,
        3,
        manifest(rules, MAP),
        OPTIONS,
        token,
        &entries,
    );
    assert!(matches!(hosted, Err(GameRestoreError::Setup(_))));
    // A journal the game rejects reopens none: the waiting player's turn ended out of its turn.
    let out_of_turn = [GameEntry::Numbered(waiting, 1, Engine::EndTurn)];
    let hosted = reopen(&out_of_turn);
    assert!(matches!(
        hosted,
        Err(GameRestoreError::Rejected { at: 0, .. })
    ));
}
