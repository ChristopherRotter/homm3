//! A whole game's spectators (E16 M5 T22a; SRV-Q36): a spectator is shown every player's view,
//! each player's frames wrapped as `GameWatched` in the spectator's own numbering, while what the
//! players hear stays what they hear unwatched (a golden replay beside its in-process twin); a
//! late watcher starts from each player's view as a re-taken seat would; the creator's setting,
//! its host's change until the start (told to every other player, and named in each seat's
//! `GameJoined`, since T24: SRV-Q38), its refusals, and the setting the game started with
//! journaled.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::Actor;

use super::game_fold::one;
use super::game_seats::{MAP, OPTIONS, end_turn, join, send, to, token};
use super::game_session::duel;
use crate::codec;
use crate::map::game::{command, manifest, observe};
use crate::session::game::{GameEntry, GameOptions, GameOutbound, GameSession, Spectators};
use crate::session::{ConnId, Outbound};
use crate::wire::game::GameReject;
use crate::wire::{ClientMessage, GoodbyeReason, Refusal, ServerMessage};

/// [`OPTIONS`] open to two spectators.
const WATCHED: GameOptions = GameOptions {
    spectators: Spectators {
        limit: 2,
        open: true,
    },
    ..OPTIONS
};

/// The spectators' connections.
const S: ConnId = ConnId(50);
const T: ConnId = ConnId(51);
const U: ConnId = ConnId(52);

/// The connection of player `index`.
const fn conn(index: usize) -> ConnId {
    ConnId(100 + index as u64)
}

/// `GameWatch` of game 9.
const WATCH: ClientMessage = ClientMessage::GameWatch { game: 9 };

/// `Refused` for `reason`.
const fn refused(reason: Refusal) -> ServerMessage {
    ServerMessage::Refused { reason }
}

/// A spectator's frame unwrapped: its number, the player and the player's frame, re-encoded (the
/// splice is the codec's text: the frame re-encodes to itself).
fn unwrap(frame: &str) -> (u32, u8, String) {
    let decoded = codec::decode_server(frame.as_bytes(), usize::MAX).unwrap();
    assert_eq!(
        codec::encode_text(&decoded),
        frame,
        "the splice is serde's form"
    );
    let ServerMessage::GameWatched { n, player, message } = decoded else {
        panic!("not a watched frame: {frame}");
    };
    (n, player, codec::encode_text(&message))
}

#[test]
fn a_spectator_is_shown_every_players_view_and_the_players_hear_what_they_hear_unwatched() {
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
        GameSession::new(&rules, &setup, seed, manifest(&rules, MAP), WATCHED, token).unwrap();
    let players = twin.state().players();
    // A spectator watches the waiting game: every player's snapshot, numbered as its last (none
    // yet), no turn before the start.
    out.clear();
    net.handle(S, WATCH, &mut out);
    let watching = ServerMessage::GameWatching {
        game: 9,
        manifest: net.manifest().clone(),
    };
    assert_eq!(to(&out[..1], S), [watching]);
    let mut numbers = 0;
    for (o, player) in out[1..].iter().zip(0..=u8::MAX) {
        let (n, of, frame) = unwrap(&o.frame);
        numbers += 1;
        assert_eq!((n, of), (numbers, player));
        let observation = observe(
            net.state(),
            &rules,
            h3_game::Viewer::Player(PlayerId::new(of)),
        );
        let snapshot = ServerMessage::GameSnapshot { n: 0, observation };
        assert_eq!(frame, codec::encode_text(&snapshot));
    }
    assert_eq!(out.len(), 1 + players);
    assert_eq!(net.spectators(), 1);
    let (mut heard, mut told) = (vec![Vec::new(); players], vec![Vec::new(); players]);
    let mut watched = vec![Vec::new(); players];
    let mut hear = |out: &[Outbound], heard: &mut [Vec<String>], watched: &mut [Vec<String>]| {
        for o in out {
            if o.to == S {
                let (n, player, frame) = unwrap(&o.frame);
                numbers += 1;
                assert_eq!(n, numbers, "numbered in the spectator's own sequence");
                watched[usize::from(player)].push(frame);
            } else {
                heard[usize::try_from(o.to.0 - 100).unwrap()].push(o.frame.clone());
            }
        }
    };
    for o in &twin_out {
        told[o.to.index()].push(o.frame.clone());
    }
    for (index, player) in (0..=u8::MAX).take(players).enumerate() {
        out.clear();
        net.handle(conn(index), join(player, None), &mut out);
        // `GameJoined` with the token is the seat's alone.
        assert!(matches!(
            to(&out[..1], conn(index))[..],
            [ServerMessage::GameJoined { .. }]
        ));
        hear(&out[1..], &mut heard, &mut watched);
    }
    assert!(net.started());
    // The setting it started with is journaled right before the start.
    let entries: Vec<GameEntry> = net.drain_entries().collect();
    let started = entries
        .iter()
        .position(|e| *e == GameEntry::Started)
        .unwrap();
    assert_eq!(entries[started - 1], GameEntry::Watchable);
    let mut journal = entries;
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
            }
        }
        hear(&out, &mut heard, &mut watched);
        for o in &twin_out {
            told[o.to.index()].push(o.frame.clone());
        }
        assert_eq!(heard, told, "command {seq}");
    }
    // The spectator heard every frame of every player's, in order, beside its watch's start.
    assert_eq!(watched, heard);
    assert_eq!(net.state().state_hash(), twin.state().state_hash());
    journal.extend(net.drain_entries());
    // Restored, the game takes spectators as it started; nobody watches.
    let back = GameSession::reopen(
        &rules,
        &setup,
        seed,
        manifest(&rules, MAP),
        OPTIONS,
        token,
        &journal,
    )
    .unwrap();
    assert!(back.watchable());
    assert_eq!(back.spectators(), 0);
}

#[test]
fn a_late_spectator_starts_from_every_players_view_as_a_retaken_seat_would() {
    let (rules, setup) = duel();
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), WATCHED, token).unwrap();
    let acting = s.state().turn_group()[0];
    let waiting = PlayerId::new(u8::from(acting.raw() == 0));
    send(&mut s, conn(0), join(acting.raw(), None));
    send(&mut s, conn(1), join(waiting.raw(), None));
    let out = send(&mut s, conn(0), end_turn(1));
    let last = |s: &GameSession<'_>, player: PlayerId| {
        let record = s.record(player).unwrap();
        u32::try_from(record.lines().count() - 1).unwrap()
    };
    // The waiting player acts now: its turn's commands are its last message.
    let ServerMessage::GameTurn {
        legal, castable, ..
    } = to(&out, conn(1)).pop().unwrap()
    else {
        panic!("the other player's turn");
    };
    let out = send(&mut s, S, WATCH);
    let started = u32::try_from(out.len() - 1).unwrap();
    let mut frames = out[1..].iter().map(|o| unwrap(&o.frame));
    let mut expect = Vec::new();
    for player in [PlayerId::new(0), PlayerId::new(1)] {
        let n = last(&s, player);
        let observation = observe(s.state(), rules, h3_game::Viewer::Player(player));
        expect.push((player.raw(), ServerMessage::GameSnapshot { n, observation }));
        if player == waiting {
            let (legal, castable) = (legal.clone(), castable.clone());
            expect.push((player.raw(), ServerMessage::GameTurn { n, legal, castable }));
        }
    }
    for (number, (player, message)) in (1..).zip(expect) {
        assert_eq!(
            frames.next(),
            Some((number, player, codec::encode_text(&message)))
        );
    }
    assert_eq!(frames.next(), None);
    // Its watch goes on with the players' messages, numbered on.
    let out = send(&mut s, conn(1), end_turn(1));
    let watched: Vec<_> = out.iter().filter(|o| o.to == S).collect();
    let numbers: Vec<u32> = watched.iter().map(|o| unwrap(&o.frame).0).collect();
    assert!(!numbers.is_empty());
    assert_eq!(
        numbers,
        (started + 1..).take(numbers.len()).collect::<Vec<_>>()
    );
    let of_player: Vec<String> = (out.iter())
        .filter(|o| o.to == conn(0) || o.to == conn(1))
        .map(|o| o.frame.clone())
        .collect();
    let unwrapped: Vec<String> = watched.iter().map(|o| unwrap(&o.frame).2).collect();
    assert_eq!(unwrapped, of_player);
}

#[test]
fn the_creators_setting_decides_and_a_spectator_neither_plays_nor_sits() {
    let (rules, setup) = duel();
    // Closed by default: nobody watches, on a server that would take two; in-process nobody does
    // either.
    let closed = GameOptions {
        spectators: Spectators {
            limit: 2,
            open: false,
        },
        ..OPTIONS
    };
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), closed, token).unwrap();
    assert!(!s.watchable() && !s.takes_spectators());
    assert_eq!(
        to(&send(&mut s, S, WATCH), S),
        [refused(Refusal::NoSpectators)]
    );
    let mut started = Vec::new();
    let open = GameOptions {
        spectators: Spectators {
            limit: 2,
            open: true,
        },
        ..OPTIONS
    };
    let mut host = GameSession::start(rules, &setup, 3, manifest(rules, MAP), open, &mut started);
    let host = host.as_mut().unwrap();
    assert_eq!(
        to(&send(host, S, WATCH), S),
        [refused(Refusal::NoSpectators)]
    );
    // Another game's, refused; the host (the first seat taken) opens it until the start, another
    // seat may not.
    let other = ClientMessage::GameWatch { game: 8 };
    assert_eq!(
        to(&send(&mut s, S, other), S),
        [refused(Refusal::NoSuchGame)]
    );
    // A seat's `GameJoined` names the setting (SRV-Q38 (b)).
    let joined_as = |out: &[Outbound], conn, open| {
        let told = to(out, conn);
        assert!(
            matches!(told[0], ServerMessage::GameJoined { spectators, .. } if spectators == open),
            "{told:?}"
        );
    };
    joined_as(&send(&mut s, conn(0), join(0, None)), conn(0), false);
    let set = |seq, open| ClientMessage::SetSpectators { seq, open };
    // The host's change is told to every other player, in its own sequence (SRV-Q38 (b)): here
    // the second player's record, whose seat nobody holds yet.
    let told_other = |s: &GameSession<'_>, open| {
        let record = s.record(PlayerId::new(1)).unwrap();
        let n = u32::try_from(record.lines().count() - 1).unwrap();
        let told = ServerMessage::GameSpectators { n, open };
        assert_eq!(
            record.lines().last(),
            Some(codec::encode_text(&told).as_str())
        );
    };
    assert_eq!(
        to(&send(&mut s, S, set(1, true)), S),
        [ServerMessage::GameRejected {
            n: 0,
            seq: 1,
            code: GameReject::NotSeated {}
        }]
    );
    let out = send(&mut s, conn(0), set(2, true));
    assert!(matches!(
        to(&out, conn(0))[..],
        [ServerMessage::SpectatorsSet {
            seq: 2,
            open: true,
            ..
        }]
    ));
    assert!(s.watchable() && s.takes_spectators());
    told_other(&s, true);
    // Two may watch, a third is refused; a spectator watches once and takes no seat; a seat's
    // connection does not watch.
    assert!(matches!(
        to(&send(&mut s, S, WATCH), S)[0],
        ServerMessage::GameWatching { .. }
    ));
    let joined = refused(Refusal::AlreadyJoined);
    assert_eq!(
        to(&send(&mut s, S, WATCH), S),
        std::slice::from_ref(&joined)
    );
    assert_eq!(
        to(&send(&mut s, S, join(1, None)), S),
        std::slice::from_ref(&joined)
    );
    assert_eq!(to(&send(&mut s, conn(0), WATCH), conn(0)), [joined]);
    send(&mut s, T, WATCH);
    assert_eq!(
        to(&send(&mut s, U, WATCH), U),
        [refused(Refusal::NoSpectators)]
    );
    assert_eq!(s.spectators(), 2);
    // A spectator's command, query, pause and record are no seat's.
    let unseated = |seq| ServerMessage::GameRejected {
        n: 0,
        seq,
        code: GameReject::NotSeated {},
    };
    assert_eq!(to(&send(&mut s, S, end_turn(3)), S), [unseated(3)]);
    let record = ClientMessage::FetchRecord { seq: 4 };
    assert_eq!(to(&send(&mut s, S, record), S), [unseated(4)]);
    assert_eq!(
        to(&send(&mut s, S, ClientMessage::Pause { seq: 5 }), S),
        [unseated(5)]
    );
    // Leaving ends a watch; closing the game ends the others'.
    let left = send(&mut s, T, ClientMessage::Leave {});
    assert!(left[0].close);
    assert_eq!(s.spectators(), 1);
    let out = send(&mut s, conn(0), set(6, false));
    let closed = ServerMessage::Goodbye {
        reason: GoodbyeReason::SpectatorsClosed,
    };
    assert_eq!(to(&out, S), [closed]);
    assert!(out.iter().any(|o| o.to == S && o.close));
    assert_eq!((s.spectators(), s.watchable()), (0, false));
    told_other(&s, false);
    // Another seat may not change it; once the game started, nobody may. Re-opened, the second
    // seat's join names it.
    send(&mut s, conn(0), set(7, true));
    joined_as(&send(&mut s, conn(1), join(1, None)), conn(1), true);
    assert!(s.started());
    let rejected = |n, seq, code| ServerMessage::GameRejected { n, seq, code };
    let told = to(&send(&mut s, conn(1), set(8, false)), conn(1));
    assert!(matches!(
        told[..],
        [ServerMessage::GameRejected {
            seq: 8,
            code: GameReject::NotHost {},
            ..
        }]
    ));
    let n = |s: &GameSession<'_>, player| {
        u32::try_from(s.record(PlayerId::new(player)).unwrap().lines().count() - 1).unwrap()
    };
    let next = n(&s, 0) + 1;
    assert_eq!(
        to(&send(&mut s, conn(0), set(9, false)), conn(0)),
        [rejected(next, 9, GameReject::AlreadyStarted {})]
    );
    // The setting it started with is journaled; a spectator's watch ends at a disconnect and
    // at the host's pause, as the seats'.
    send(&mut s, S, WATCH);
    s.disconnected(S);
    assert_eq!(s.spectators(), 0);
    send(&mut s, S, WATCH);
    let out = send(&mut s, conn(0), ClientMessage::Pause { seq: 10 });
    let paused = ServerMessage::Goodbye {
        reason: GoodbyeReason::Paused,
    };
    assert_eq!(to(&out, S), [paused]);
    assert_eq!(s.spectators(), 0);
    // A paused game is listed as taking none (E16 M5 T22b).
    assert!(s.watchable() && !s.takes_spectators());
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    let at = entries
        .iter()
        .position(|e| *e == GameEntry::Started)
        .unwrap();
    assert_eq!(entries[at - 1], GameEntry::Watchable);
    // A game that started closed restores closed, whatever the host's options say now.
    let mut closed =
        GameSession::new(rules, &setup, 3, manifest(rules, MAP), OPTIONS, token).unwrap();
    send(&mut closed, conn(0), join(0, None));
    joined_as(&send(&mut closed, conn(1), join(1, None)), conn(1), false);
    let entries: Vec<GameEntry> = closed.drain_entries().collect();
    assert!(!entries.contains(&GameEntry::Watchable));
    let back = GameSession::reopen(
        rules,
        &setup,
        3,
        manifest(rules, MAP),
        open,
        token,
        &entries,
    );
    assert!(!back.unwrap().watchable());
    // Open on a host that takes no spectators, it is listed as taking none (E16 M5 T22b).
    let none = GameOptions {
        spectators: Spectators {
            limit: 0,
            open: true,
        },
        ..OPTIONS
    };
    let s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), none, token).unwrap();
    assert!(s.watchable() && !s.takes_spectators());
}

#[test]
fn a_watched_frame_is_the_codecs_text_of_its_message() {
    let inner = ServerMessage::GamePauseAsked { n: 4, player: 1 };
    let frames = vec![GameOutbound {
        to: PlayerId::new(1),
        frame: codec::encode_text(&inner),
    }];
    let (rules, setup) = duel();
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), WATCHED, token).unwrap();
    send(&mut s, S, WATCH);
    let mut out = Vec::new();
    s.route(frames, &mut out);
    let typed = ServerMessage::GameWatched {
        n: 3,
        player: 1,
        message: std::boxed::Box::new(inner),
    };
    assert_eq!(
        out,
        [Outbound {
            to: S,
            frame: codec::encode_text(&typed),
            close: false,
        }]
    );
}

#[test]
fn a_timed_games_spectator_starts_with_every_players_clock() {
    let (rules, setup) = duel();
    let clock = crate::session::game::Clock {
        turn_ms: Some(60_000),
        ..crate::session::game::Clock::UNLIMITED
    };
    let timed = GameOptions { clock, ..WATCHED };
    let mut s = GameSession::new(rules, &setup, 3, manifest(rules, MAP), timed, token).unwrap();
    s.elapse(1_000, &mut Vec::new());
    send(&mut s, conn(0), join(0, None));
    send(&mut s, conn(1), join(1, None));
    s.elapse(2_000, &mut Vec::new());
    let out = send(&mut s, S, WATCH);
    // Each player's snapshot, then its clock (the acting player's timer runs), the acting one's
    // turn last.
    let kinds: Vec<(u8, &str)> = (out[1..].iter())
        .map(|o| {
            let (_, player, frame) = unwrap(&o.frame);
            let kind = match codec::decode_server(frame.as_bytes(), usize::MAX).unwrap() {
                ServerMessage::GameSnapshot { .. } => "snapshot",
                ServerMessage::GameClock { .. } => "clock",
                ServerMessage::GameTurn { .. } => "turn",
                other => panic!("{other:?}"),
            };
            (player, kind)
        })
        .collect();
    let acting = s.state().turn_group()[0].raw();
    assert!(kinds.contains(&(acting, "clock")), "{kinds:?}");
    assert!(kinds.contains(&(acting, "turn")), "{kinds:?}");
}
