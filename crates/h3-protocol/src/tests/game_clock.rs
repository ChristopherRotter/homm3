//! The whole game's clock (E16 M5 T16; SRV-Q31, SRV-Q32): the host's clock passed in, a turn's
//! timer that runs out ended by the system (the time journaled, a restore keeping the time left
//! and never counting the downtime, a step cut off by a crash taken at the first reading), a
//! battle against a monster that does not stop it and is played out by the computer, the
//! level-up's hold stopping every other timer and answered at its end, the chess clocks of a
//! battle between two players' heroes (the increment, a side run out played by the computer, the
//! attacker's turn timer stopped meanwhile), a paused game and one not started counting nothing;
//! a stopping host's time journaled (T17).

use std::boxed::Box;
use std::vec::Vec;

use h3_battle::{BattleSide, BattleState, Command, LegalCommands};
use h3_content_battle::BattleRuleset;
use h3_contracts::PlayerId;
use h3_game::{Actor, GameRuleset, GameSetup, GameState, SystemCommand, legal_commands};
use h3_sim::adventure::replay::Replay;

use super::game_fold::one;
use super::game_seats::{MAP, join, send, token};
use super::game_session::duel;
use crate::codec;
use crate::map::game::{command, manifest};
use crate::session::game::{Allowance, Clock, GameEntry, GameOptions, GameOutbound, GameSession};
use crate::session::{Bot, ConnId, Outbound};
use crate::wire::game::{Duel, GameClock, Timer};
use crate::wire::{ClientMessage, ServerMessage};

const P0: PlayerId = PlayerId::new(0);
const P1: PlayerId = PlayerId::new(1);

/// A minute a turn, 20 s a level-up, 30 s and 5 s a command in a battle between players.
const CLOCK: Clock = Clock {
    turn_ms: Some(60_000),
    level_up_ms: Some(20_000),
    battle: Allowance {
        base_ms: 30_000,
        increment_ms: 5_000,
    },
};

/// Game 9 under `clock`, no checkpoints, every record unbounded.
const fn options(clock: Clock) -> GameOptions {
    GameOptions {
        game: 9,
        checkpoint_every: 0,
        record_bytes: usize::MAX,
        clock,
        commands: crate::session::game::CommandBounds::NONE,
    }
}

const fn running(left_ms: u32) -> Timer {
    Timer {
        left_ms,
        running: true,
    }
}

const fn stopped(left_ms: u32) -> Timer {
    Timer {
        left_ms,
        running: false,
    }
}

/// A turn's timer alone.
fn turn(timer: Timer) -> GameClock {
    GameClock {
        turn: Some(timer),
        ..GameClock::default()
    }
}

/// The clocks `player` was told in `frames`, in order.
fn clocks(frames: &[GameOutbound], player: PlayerId) -> Vec<GameClock> {
    let told = frames.iter().filter(|o| o.to == player);
    told.filter_map(
        |o| match codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap() {
            ServerMessage::GameClock { clock, .. } => Some(clock),
            _ => None,
        },
    )
    .collect()
}

/// The last clock `player` was told in `frames`.
fn last(frames: &[GameOutbound], player: PlayerId) -> Option<GameClock> {
    clocks(frames, player).pop()
}

/// A computer that strikes when it can, else takes a legal command that stays in the battle, by
/// a fixed stream (so that battles end).
#[derive(Debug)]
pub(super) struct Striker(pub(super) u64);

impl Bot for Striker {
    fn command(
        &mut self,
        _: &BattleState,
        _: &BattleRuleset,
        _: BattleSide,
        legal: &LegalCommands,
    ) -> Command {
        pick(&mut self.0, legal.iter().copied())
    }
}

/// A strike if `legal` has one, else a command that stays in the battle, drawn from `stream`.
pub(super) fn pick(stream: &mut u64, legal: impl Iterator<Item = Command> + Clone) -> Command {
    let strikes: Vec<Command> = (legal.clone())
        .filter(|c| matches!(c, Command::Attack { .. } | Command::Shoot { .. }))
        .collect();
    let staying: Vec<Command> = if strikes.is_empty() {
        legal
            .filter(|c| !matches!(c, Command::Retreat | Command::Surrender))
            .collect()
    } else {
        strikes
    };
    *stream = stream
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let at = usize::try_from(*stream >> 33).unwrap() % staying.len();
    staying[at]
}

/// The duel map's game under [`CLOCK`], in-process, its clock read first at 1 s.
fn started<'r>(rules: &'r GameRuleset, setup: &GameSetup) -> (GameSession<'r>, Vec<GameOutbound>) {
    let mut out = Vec::new();
    let mut s = GameSession::start(
        rules,
        setup,
        1,
        manifest(rules, MAP),
        options(CLOCK),
        &mut out,
    )
    .unwrap();
    assert_eq!(s.deadline(), None, "before the clock's first reading");
    let start = out.clone();
    out.clear();
    // The first reading counts nothing.
    s.elapse(1_000, &mut out);
    assert!(out.is_empty());
    (s, start)
}

#[test]
fn a_turn_that_runs_out_is_ended_by_the_system_and_its_time_journaled() {
    let (rules, setup) = duel();
    let (mut s, start) = started(rules, &setup);
    // The start: the acting player's timer after its snapshot; none for the other.
    assert!(s.state().is_acting(P0) && !s.state().is_acting(P1));
    assert_eq!(clocks(&start, P0), [turn(running(60_000))]);
    assert_eq!(clocks(&start, P1), []);
    // Counting down is not told; it is journaled every 15 s at the latest.
    assert_eq!(s.deadline(), Some(16_000));
    let mut out = Vec::new();
    s.elapse(16_000, &mut out);
    assert!(out.is_empty());
    assert_eq!(
        s.drain_entries().collect::<Vec<_>>(),
        [GameEntry::Elapsed(15_000)]
    );
    assert_eq!(s.deadline(), Some(31_000));
    // A clock that goes back is ignored.
    s.elapse(500, &mut out);
    assert!(out.is_empty());
    assert_eq!(s.deadline(), Some(31_000));
    // At 61 s the turn ran out: the system ends it, and player 1's turn timer starts then.
    s.elapse(71_000, &mut out);
    let end = SystemCommand::EndTurn { player: P0 };
    assert_eq!(
        s.drain_entries().collect::<Vec<_>>(),
        [
            GameEntry::Elapsed(45_000),
            GameEntry::Command(Actor::System, h3_game::GameCommand::System(end)),
        ]
    );
    assert!(!s.state().is_acting(P0) && s.state().is_acting(P1));
    assert_eq!(clocks(&out, P0), [GameClock::default()]);
    // Told as it started at 61 s; at 71 s it still ends then, so nothing more is told.
    assert_eq!(clocks(&out, P1), [turn(running(60_000))]);
    assert_eq!(s.deadline(), Some(76_000));
    let mut seen = Vec::new();
    s.elapse(76_000, &mut seen);
    assert!(seen.is_empty());
    assert_eq!(
        s.drain_entries().collect::<Vec<_>>(),
        [GameEntry::Elapsed(15_000)]
    );
}

#[test]
fn a_restore_keeps_the_time_left_and_never_counts_the_downtime() {
    let (rules, setup) = duel();
    let (mut s, _) = started(rules, &setup);
    let mut out = Vec::new();
    s.elapse(61_000, &mut out);
    let ended: Vec<GameEntry> = s.drain_entries().collect();
    // Player 1's turn: 20 s counted, journaled; 5 s more, not yet.
    s.elapse(81_000, &mut out);
    s.elapse(86_000, &mut out);
    let mut entries = ended.clone();
    entries.extend(s.drain_entries());
    assert_eq!(entries.last(), Some(&GameEntry::Elapsed(20_000)));
    // A host that stops journals the 5 s (E16 M5 T17); nothing more when nothing counted since.
    let mut stopped = entries.clone();
    s.flush_time();
    s.flush_time();
    stopped.extend(s.drain_entries());
    assert_eq!(stopped[entries.len()..], [GameEntry::Elapsed(5_000)]);
    let mut told = Vec::new();
    let restore = |entries: &[GameEntry], told: &mut Vec<GameOutbound>| {
        let manifest = manifest(rules, MAP);
        GameSession::restore(rules, &setup, 1, manifest, options(CLOCK), entries, told).unwrap()
    };
    let mut r = restore(&entries, &mut told);
    assert_eq!(r.state().state_hash(), s.state().state_hash());
    assert_eq!(clocks(&told, P1), [turn(running(40_000))]);
    let mut at_stop = Vec::new();
    restore(&stopped, &mut at_stop);
    assert_eq!(clocks(&at_stop, P1), [turn(running(35_000))]);
    // The downtime never counts: the first reading counts nothing.
    let mut again = Vec::new();
    r.elapse(5_000_000, &mut again);
    r.elapse(5_030_000, &mut again);
    assert!(again.is_empty());
    assert_eq!(
        r.drain_entries().collect::<Vec<_>>(),
        [GameEntry::Elapsed(30_000)]
    );
    assert_eq!(r.deadline(), Some(5_040_000));
    // A journal a crash cut between a turn's end and the system's step: the step at the first
    // reading.
    let cut = &ended[..1];
    assert_eq!(cut, [GameEntry::Elapsed(60_000)]);
    let mut c = restore(cut, &mut told);
    assert!(c.state().is_acting(P0));
    assert_eq!(c.deadline(), None);
    c.elapse(9, &mut again);
    assert_eq!(c.drain_entries().collect::<Vec<_>>(), ended[1..]);
    assert!(c.state().is_acting(P1));
}

/// A golden replay played in-process under [`CLOCK`] up to and including command `last`, the
/// clock read first before it; the frames of the last command.
fn replayed<'r>(
    rules: &'r GameRuleset,
    setup: &GameSetup,
    replay: &Replay,
    last: usize,
) -> (GameSession<'r>, Vec<GameOutbound>) {
    let mut out = Vec::new();
    let manifest = manifest(rules, MAP);
    let mut s = GameSession::start(
        rules,
        setup,
        replay.seed,
        manifest,
        options(CLOCK),
        &mut out,
    )
    .unwrap();
    s.elapse(0, &mut out);
    for (seq, (actor, engine)) in replay.commands[..=last].iter().enumerate() {
        out.clear();
        let seq = u32::try_from(seq).unwrap();
        match *actor {
            Actor::Player(player) => s.play(player, seq, command(engine), &mut out),
            Actor::System => s.system(command(engine), &mut out),
        }
        .unwrap();
    }
    s.drain_entries().for_each(drop);
    (s, out)
}

#[test]
fn a_level_up_holds_every_timer_and_is_answered_at_its_end() {
    // Player 0's hero levels up after its battle at command 1, in its own turn.
    let (rules, setup, replay) = one("rewards-stone-guard-s1");
    let (mut s, out) = replayed(&rules, &setup, &replay, 1);
    assert_eq!(s.state().waiting_for(), Some(P0));
    let held = GameClock {
        turn: Some(stopped(60_000)),
        level_up: Some(running(20_000)),
        ..GameClock::default()
    };
    assert_eq!(last(&out, P0), Some(held));
    let waits = GameClock {
        waits: true,
        ..GameClock::default()
    };
    assert_eq!(last(&out, P1), Some(waits));
    // At 20 s the system answers it by its default; the turn's timer runs on from there.
    let mut out = Vec::new();
    s.elapse(25_000, &mut out);
    let answer = SystemCommand::Answer { player: P0 };
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    assert_eq!(
        entries[..2],
        [
            GameEntry::Elapsed(20_000),
            GameEntry::Command(Actor::System, h3_game::GameCommand::System(answer)),
        ]
    );
    assert_eq!(s.state().waiting_for(), None);
    assert_eq!(last(&out, P1), Some(GameClock::default()));
    assert_eq!(last(&out, P0), Some(turn(running(60_000))));
    assert_eq!(s.deadline(), Some(35_000));
}

#[test]
fn a_level_up_without_a_limit_waits() {
    let (rules, setup, replay) = one("rewards-stone-guard-s1");
    let mut out = Vec::new();
    let clock = Clock {
        level_up_ms: None,
        ..CLOCK
    };
    let manifest = manifest(&rules, MAP);
    let mut s = GameSession::start(
        &rules,
        &setup,
        replay.seed,
        manifest,
        options(clock),
        &mut out,
    )
    .unwrap();
    s.elapse(0, &mut out);
    for (seq, (actor, engine)) in replay.commands[..=1].iter().enumerate() {
        out.clear();
        let seq = u32::try_from(seq).unwrap();
        match *actor {
            Actor::Player(player) => s.play(player, seq, command(engine), &mut out),
            Actor::System => s.system(command(engine), &mut out),
        }
        .unwrap();
    }
    assert_eq!(last(&out, P0), Some(turn(stopped(60_000))));
    assert_eq!(s.deadline(), None, "nothing runs");
    s.drain_entries().for_each(drop);
    s.elapse(1_000_000, &mut out);
    assert_eq!(s.drain_entries().count(), 0);
    assert_eq!(s.state().waiting_for(), Some(P0));
}

/// A game of the replay `name` hosted for connections under [`CLOCK`], every seat taken, the
/// computer a [`Striker`] and the clock read first at 0.
fn hosted<'r>(rules: &'r GameRuleset, setup: &GameSetup, replay: &Replay) -> GameSession<'r> {
    let manifest = manifest(rules, MAP);
    let mut s =
        GameSession::new(rules, setup, replay.seed, manifest, options(CLOCK), token).unwrap();
    for player in (0..=u8::MAX).take(s.state().players()) {
        send(&mut s, conn(player), join(player, None));
    }
    assert!(s.started());
    let mut out = Vec::new();
    s.computer(Box::new(Striker(7)), &mut out);
    let mut frames = Vec::new();
    s.elapse(0, &mut frames);
    s.drain_entries().for_each(drop);
    s
}

/// Player `player`'s connection.
const fn conn(player: u8) -> ConnId {
    ConnId(300 + player as u64)
}

/// The replay's command `seq`, a player's, sent by its seat.
fn seat(s: &mut GameSession<'_>, replay: &Replay, seq: usize) -> Vec<Outbound> {
    let (Actor::Player(player), engine) = &replay.commands[seq] else {
        panic!("command {seq} is the system actor's");
    };
    let message = ClientMessage::GameCommand {
        seq: u32::try_from(seq).unwrap(),
        command: command(engine),
    };
    send(s, conn(player.raw()), message)
}

#[test]
fn a_battle_against_a_monster_runs_on_and_is_played_out_at_the_turns_end() {
    // A battle against a wandering monster opens at command 15.
    let (rules, setup, replay) = one("duel-s3-easy-d7");
    let mut s = hosted(&rules, &setup, &replay);
    for seq in 0..=15 {
        seat(&mut s, &replay, seq);
    }
    s.drain_entries().for_each(drop);
    let Actor::Player(owner) = replay.commands[15].0 else {
        panic!("a player's command");
    };
    assert!(s.state().battle(owner).is_some() && s.state().is_acting(owner));
    // The turn's timer runs in the battle; at its end the computer plays the owner's side out,
    // the system answers what follows and ends the turn.
    let mut out = Vec::new();
    assert_eq!(s.deadline(), Some(15_000));
    s.elapse(61_000, &mut out);
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    assert_eq!(entries[0], GameEntry::Elapsed(60_000));
    let system: Vec<&h3_game::GameCommand> = entries[1..]
        .iter()
        .filter_map(|e| match e {
            GameEntry::Command(Actor::System, c) => Some(c),
            GameEntry::Elapsed(_) => None,
            other => panic!("not the system's: {other:?}"),
        })
        .collect();
    let owners = system.iter().filter(|c| {
        matches!(c, h3_game::GameCommand::System(SystemCommand::Battle { player, .. }) if *player == owner)
    });
    assert!(
        owners.count() > 0,
        "the owner's side played by the computer"
    );
    assert_eq!(
        system.last(),
        Some(&&h3_game::GameCommand::System(SystemCommand::EndTurn {
            player: owner
        }))
    );
    assert!(s.state().battle(owner).is_none() && !s.state().is_acting(owner));
}

#[test]
fn a_battle_between_players_runs_their_chess_clocks() {
    // Player 0's hero attacks player 1's at command 0.
    let (rules, setup, replay) = one("towns-visitor-s1");
    let (mut s, out) = replayed(&rules, &setup, &replay, 0);
    let battle = s.state().battle(P0).unwrap();
    assert_eq!(battle.owners(), [Some(P0), Some(P1)]);
    let both = |attacker, defender| Some(Duel { attacker, defender });
    let to_act = battle.state().side_to_act().unwrap();
    let side = |side| {
        if to_act == side {
            running(30_000)
        } else {
            stopped(30_000)
        }
    };
    let opened = both(side(BattleSide::Attacker), side(BattleSide::Defender));
    // The attacker's turn timer stops; the defender has none.
    let attacker = GameClock {
        turn: Some(stopped(60_000)),
        battle: opened,
        ..GameClock::default()
    };
    assert_eq!(last(&out, P0), Some(attacker));
    let defender = GameClock {
        battle: opened,
        ..GameClock::default()
    };
    assert_eq!(last(&out, P1), Some(defender));
    // The attacker plays; the defender's clock runs out and the computer (Defend in-process)
    // plays its side for the rest of the battle.
    let (mut at, mut seq, mut stream, mut increments) = (0, 100, 1, 0);
    let (mut out, mut run_out) = (Vec::new(), false);
    while let Some(battle) = s.state().battle(P0) {
        let side = battle.state().side_to_act().unwrap();
        out.clear();
        if side == BattleSide::Attacker {
            let legal = legal_commands(s.state(), &rules, Actor::Player(P0));
            let battle = legal.iter().filter_map(|c| match c {
                h3_game::GameCommand::Battle(c) => Some(*c),
                _ => None,
            });
            let choice = pick(&mut stream, battle);
            let before = left(&s, BattleSide::Attacker);
            let wire = command(&h3_game::GameCommand::Battle(choice));
            s.play(P0, seq, wire, &mut out).unwrap();
            seq += 1;
            // A side whose clock ran out is told no battle commands: the computer plays it at
            // once (E16 M5 T19, the 2b reviewer's).
            assert!(!run_out || battle_turns(&out, P1) == 0);
            if s.state().battle(P0).is_some() {
                assert_eq!(
                    left(&s, BattleSide::Attacker),
                    before + 5_000,
                    "the increment"
                );
                increments += 1;
            }
        } else {
            // Exactly to its end: the attacker's clock counts nothing meanwhile.
            at += u64::from(left(&s, BattleSide::Defender));
            s.elapse(at, &mut out);
            assert!(
                s.state()
                    .battle(P0)
                    .is_none_or(|b| b.state().side_to_act() != Some(BattleSide::Defender))
            );
            if let Some(clock) = last(&out, P0).and_then(|c| c.battle) {
                assert_eq!(clock.defender, stopped(0));
            }
            assert_eq!(battle_turns(&out, P1), 0);
            run_out = true;
        }
    }
    assert!(increments > 0 && run_out);
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    let defends = entries.iter().filter(|e| {
        matches!(e, GameEntry::Command(Actor::System, h3_game::GameCommand::System(SystemCommand::Battle { player, .. })) if *player == P1)
    });
    assert!(
        defends.count() > 0,
        "the defender's side played by the computer"
    );
    // The attacker's turn resumes with what it had left: nothing counted in the battle.
    assert_eq!(turn_left(s.state(), &s), 60_000);
}

/// How many of `player`'s turns in `frames` list battle commands.
fn battle_turns(frames: &[GameOutbound], player: PlayerId) -> usize {
    let told = frames.iter().filter(|o| o.to == player);
    told.filter(|o| {
        matches!(codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap(),
            ServerMessage::GameTurn { legal, .. } if legal.iter().any(|c| matches!(c, crate::wire::game::GameCommand::Battle { .. })))
    })
    .count()
}

/// What the attacker's view shows left on its side's chess clock.
fn left(s: &GameSession<'_>, side: BattleSide) -> u32 {
    let clock = view(s, P0).battle.unwrap();
    match side {
        BattleSide::Attacker => clock.attacker.left_ms,
        BattleSide::Defender => clock.defender.left_ms,
    }
}

/// Player 0's turn's time left.
fn turn_left(state: &GameState, s: &GameSession<'_>) -> u32 {
    assert!(state.is_acting(P0));
    view(s, P0).turn.unwrap().left_ms
}

/// `player`'s clock as a fresh snapshot would tell it: its record's last `GameClock`.
fn view(s: &GameSession<'_>, player: PlayerId) -> GameClock {
    let record = s.record(player).unwrap();
    let clocks = record.lines().skip(1).filter_map(|line| {
        match codec::decode_server(line.as_bytes(), usize::MAX).unwrap() {
            ServerMessage::GameClock { clock, .. } => Some(clock),
            _ => None,
        }
    });
    clocks.last().unwrap()
}

#[test]
fn a_paused_game_and_one_not_started_count_nothing() {
    let (rules, setup) = duel();
    let manifest = manifest(rules, MAP);
    let mut s = GameSession::new(rules, &setup, 1, manifest, options(CLOCK), token).unwrap();
    let mut out = Vec::new();
    s.elapse(0, &mut out);
    send(&mut s, conn(0), join(0, None));
    s.elapse(100_000, &mut out);
    assert_eq!(s.deadline(), None, "not started");
    let begun = send(&mut s, conn(1), join(1, None));
    assert!(s.started());
    // The start tells each player its clock as it runs now (E16 M5 T19, the 2b reviewer's): the
    // first player's turn timer running, though its join's snapshot told it stopped; the second
    // has nothing to be told.
    let told = |to: ConnId| {
        let told = begun.iter().filter(|o| o.to == to);
        told.filter_map(
            |o| match codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap() {
                ServerMessage::GameClock { clock, .. } => Some(clock),
                _ => None,
            },
        )
        .collect::<Vec<_>>()
    };
    assert_eq!(told(conn(0)), [turn(running(60_000))]);
    assert_eq!(told(conn(1)), []);
    s.drain_entries().for_each(drop);
    s.elapse(110_000, &mut out);
    // The host pauses: the time counted is journaled first.
    send(&mut s, conn(0), ClientMessage::Pause { seq: 1 });
    assert_eq!(
        s.drain_entries().collect::<Vec<_>>(),
        [GameEntry::Elapsed(10_000), GameEntry::Paused]
    );
    assert_eq!(s.deadline(), None, "paused");
    s.elapse(1_000_000, &mut out);
    assert_eq!(s.drain_entries().count(), 0);
    // Resumed, the turn goes on with what it had left.
    let joined = send(&mut s, conn(0), join(0, Some(&token(P0))));
    assert_eq!(s.deadline(), Some(1_015_000));
    let clock = joined.iter().find_map(|o| {
        match codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap() {
            ServerMessage::GameClock { clock, .. } => Some(clock),
            _ => None,
        }
    });
    assert_eq!(clock, Some(turn(running(50_000))));
}

#[test]
fn a_turn_that_runs_out_answers_its_open_decisions_first() {
    // Player 0's hero visits a dwelling at command 0: its hire dialog is open.
    let (rules, setup, replay) = one("dwellings-dialog-s1");
    let (mut s, _) = replayed(&rules, &setup, &replay, 0);
    assert!(s.state().has_decision(P0) && s.state().battle(P0).is_none());
    let mut out = Vec::new();
    s.elapse(60_000, &mut out);
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    let system = |command| GameEntry::Command(Actor::System, h3_game::GameCommand::System(command));
    assert_eq!(entries[0], GameEntry::Elapsed(60_000));
    assert_eq!(entries[1], system(SystemCommand::Answer { player: P0 }));
    assert_eq!(
        entries.last(),
        Some(&system(SystemCommand::EndTurn { player: P0 }))
    );
    assert!(!s.state().has_decision(P0) && !s.state().is_acting(P0));
}
