//! The computer of a game of connections (E16 M5 T10): it plays the unowned battle sides whenever
//! one is to act in the running game, as the in-process host plays the system actor's commands
//! (a golden replay's system commands handed to a scripted computer: frame for frame and record
//! for record); a choice the engine rejects is replaced by the first legal command; a game resumed
//! by a join, or restored from a journal a crash cut, gets the computer's answers then, and a
//! restore re-runs the journal without asking it.

use std::collections::VecDeque;
use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_battle::{BattleSide, BattleState, LegalCommands};
use h3_content_battle::BattleRuleset;
use h3_contracts::PlayerId;
use h3_game::{Actor, GameRuleset, GameSetup};
use h3_sim::adventure::replay::Replay;

use super::game_fold::one;
use super::game_seats::{OPTIONS, join, send, to, token};
use crate::map::game::{command, manifest};
use crate::session::game::{GameEntry, GameOutbound, GameSession};
use crate::session::{Bot, ConnId, Outbound};
use crate::wire::game::MapIdentity;
use crate::wire::{ClientMessage, GoodbyeReason, ServerMessage, WireU64};

/// A map's identity for the manifest (the session only carries it).
const MAP: MapIdentity = MapIdentity {
    content_hash: WireU64(7),
    import_version: 1,
};

/// A golden replay with a battle against a wandering monster (its system commands from the
/// 17th command on), on the duel map.
const REPLAY: &str = "duel-s3-easy-d7";

/// A computer that gives the replay's system commands in order: asked when the replay's host was
/// not, it runs out.
#[derive(Debug)]
struct Scripted(VecDeque<h3_battle::Command>);

impl Scripted {
    /// The system actor's commands of `replay`, in order, from command `from` on.
    fn of(replay: &Replay, from: usize) -> Self {
        let systems = replay.commands[from..]
            .iter()
            .filter_map(|(actor, c)| match c {
                h3_game::GameCommand::Battle(c) if *actor == Actor::System => Some(*c),
                _ => None,
            });
        Self(systems.collect())
    }
}

impl Bot for Scripted {
    fn command(
        &mut self,
        _: &BattleState,
        _: &BattleRuleset,
        _: BattleSide,
        _: &LegalCommands,
    ) -> h3_battle::Command {
        self.0
            .pop_front()
            .expect("asked beyond the replay's system commands")
    }
}

/// A computer whose every choice the engine rejects (an unowned side has no hero to retreat),
/// counting how often it is asked.
#[derive(Debug, Default)]
struct Retreats(usize);

impl Bot for Retreats {
    fn command(
        &mut self,
        _: &BattleState,
        _: &BattleRuleset,
        _: BattleSide,
        _: &LegalCommands,
    ) -> h3_battle::Command {
        self.0 += 1;
        h3_battle::Command::Retreat
    }
}

/// A computer that must not be asked.
#[derive(Debug)]
struct Never;

impl Bot for Never {
    fn command(
        &mut self,
        _: &BattleState,
        _: &BattleRuleset,
        _: BattleSide,
        _: &LegalCommands,
    ) -> h3_battle::Command {
        panic!("a restore asks no computer")
    }
}

/// The connection of player `index`.
const fn conn(index: usize) -> ConnId {
    ConnId(200 + index as u64)
}

/// A game of `REPLAY` hosted for connections, every seat taken in turn by its connection (player
/// 0's first: the host's), so the game runs.
fn seated<'r>(rules: &'r GameRuleset, setup: &GameSetup, seed: u64) -> GameSession<'r> {
    let mut s = GameSession::new(rules, setup, seed, manifest(rules, MAP), OPTIONS, token).unwrap();
    for (index, player) in (0..=u8::MAX).take(s.state().players()).enumerate() {
        send(&mut s, conn(index), join(player, None));
    }
    assert!(s.started());
    s
}

/// The replay's command `seq`, a player's, sent by its seat's connection.
fn seat_command(s: &mut GameSession<'_>, replay: &Replay, seq: usize) -> Vec<Outbound> {
    let (Actor::Player(player), engine) = &replay.commands[seq] else {
        panic!("command {seq} is the system actor's");
    };
    let seq32 = u32::try_from(seq).unwrap();
    let message = ClientMessage::GameCommand {
        seq: seq32,
        command: command(engine),
    };
    send(s, conn(player.index()), message)
}

/// Where the replay's first system command is.
fn first_system(replay: &Replay) -> usize {
    let at = replay
        .commands
        .iter()
        .position(|(a, _)| *a == Actor::System);
    at.unwrap()
}

/// The replay's run of system commands from `at`, as journal entries.
fn systems_from(replay: &Replay, at: usize) -> Vec<GameEntry> {
    let run = replay.commands[at..].iter();
    run.take_while(|(actor, _)| *actor == Actor::System)
        .map(|(actor, c)| GameEntry::Command(*actor, c.clone()))
        .collect()
}

#[test]
fn the_computer_plays_the_unowned_sides_as_the_in_process_host_would() {
    let (rules, setup, replay) = one(REPLAY);
    let mut twin_out = Vec::new();
    let manifest = || manifest(&rules, MAP);
    let start = GameSession::start(
        &rules,
        &setup,
        replay.seed,
        manifest(),
        OPTIONS,
        &mut twin_out,
    );
    let mut twin = start.unwrap();
    let mut net = seated(&rules, &setup, replay.seed);
    let mut out = Vec::new();
    net.computer(std::boxed::Box::new(Scripted::of(&replay, 0)), &mut out);
    assert!(out.is_empty() && net.journal().commands.is_empty());
    let players = twin.state().players();
    let (mut heard, mut told) = (vec![Vec::new(); players], vec![Vec::new(); players]);
    let mut systems = 0;
    for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
        twin_out.clear();
        match *actor {
            // The connection's command, the computer's answers inside its call.
            Actor::Player(player) => {
                for o in seat_command(&mut net, &replay, seq) {
                    let index = usize::try_from(o.to.0 - 200).unwrap();
                    heard[index].push(o.frame);
                }
                let seq = u32::try_from(seq).unwrap();
                twin.play(player, seq, command(engine), &mut twin_out)
                    .unwrap();
            }
            Actor::System => {
                twin.system(command(engine), &mut twin_out).unwrap();
                systems += 1;
            }
        }
        for GameOutbound { to, frame } in twin_out.drain(..) {
            told[to.index()].push(frame);
        }
        let next = replay.commands.get(seq + 1);
        if next.is_none_or(|(actor, _)| *actor != Actor::System) {
            assert_eq!(heard, told, "after command {seq}");
        }
    }
    assert!(systems > 0);
    assert_eq!(net.state().state_hash(), twin.state().state_hash());
    assert_eq!(net.journal(), twin.journal());
    for player in (0..=u8::MAX).take(players).map(PlayerId::new) {
        assert_eq!(net.record(player), twin.record(player));
    }
    // Each of its commands journaled as the system actor's: a restore re-runs them and asks no
    // computer, which has nothing to play then either.
    let entries: Vec<GameEntry> = net.drain_entries().collect();
    let theirs = entries.iter().filter(|e| {
        matches!(
            e,
            GameEntry::Command(Actor::System, h3_game::GameCommand::Battle(_))
        )
    });
    assert_eq!(theirs.count(), systems);
    let reopened = GameSession::reopen(
        &rules,
        &setup,
        replay.seed,
        manifest(),
        OPTIONS,
        token,
        &entries,
    );
    let mut back = reopened.unwrap();
    back.computer(std::boxed::Box::new(Never), &mut out);
    assert!(out.is_empty() && back.drain_entries().next().is_none());
    assert_eq!(back.journal(), twin.journal());
}

#[test]
fn a_rejected_choice_is_replaced_by_the_first_legal_command() {
    let (rules, setup, replay) = one(REPLAY);
    let mut s = seated(&rules, &setup, replay.seed);
    let mut out = Vec::new();
    let first = first_system(&replay);
    for seq in 0..first - 1 {
        seat_command(&mut s, &replay, seq);
    }
    // Without a computer the unowned side waits; the host's legal commands are its.
    let mut opens = seated(&rules, &setup, replay.seed);
    for seq in 0..first {
        seat_command(&mut opens, &replay, seq);
    }
    let legal = opens.system_legal();
    assert!(!legal.is_empty());
    s.computer(std::boxed::Box::new(Retreats::default()), &mut out);
    assert!(out.is_empty());
    s.drain_entries().for_each(drop);
    seat_command(&mut s, &replay, first - 1);
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    let Some(GameEntry::Command(Actor::System, played)) = entries.get(1) else {
        panic!("the computer did not play: {entries:?}");
    };
    assert_eq!(command(played), legal[0]);
    assert!(s.state().unowned_battle().is_none());
}

#[test]
fn a_resumed_or_restored_game_gets_the_computers_answers() {
    let (rules, setup, replay) = one(REPLAY);
    let first = first_system(&replay);
    let answers = systems_from(&replay, first);
    // A game with no computer yet: the battle's unowned side is to act after the player's command.
    let mut s = seated(&rules, &setup, replay.seed);
    for seq in 0..first {
        seat_command(&mut s, &replay, seq);
    }
    assert!(s.state().unowned_battle().is_some());
    let cut: Vec<GameEntry> = s.drain_entries().collect();
    // Restored from a journal a crash cut there: handed over, the computer answers at once, its
    // frames into the records (no seat is held), its commands the entries the journal lacks.
    let reopened = GameSession::reopen(
        &rules,
        &setup,
        replay.seed,
        manifest(&rules, MAP),
        OPTIONS,
        token,
        &cut,
    );
    let mut back = reopened.unwrap();
    let mut out = Vec::new();
    back.computer(std::boxed::Box::new(Scripted::of(&replay, first)), &mut out);
    assert!(out.is_empty());
    assert_eq!(back.drain_entries().collect::<Vec<_>>(), answers);
    assert!(back.state().unowned_battle().is_none());
    // Paused there by its host (player 0's seat, taken first), the computer handed over waits;
    // a seat's join resumes the game and the computer answers before the joiner's turn.
    let paused = ServerMessage::Goodbye {
        reason: GoodbyeReason::Paused,
    };
    assert_eq!(
        to(
            &send(&mut s, conn(0), ClientMessage::Pause { seq: 99 }),
            conn(0)
        ),
        [paused]
    );
    s.drain_entries().for_each(drop);
    s.computer(std::boxed::Box::new(Scripted::of(&replay, first)), &mut out);
    assert!(out.is_empty() && s.drain_entries().next().is_none());
    let Actor::Player(fighter) = replay.commands[first - 1].0 else {
        panic!("the battle opens by a player's command");
    };
    let fighter_token: String = token(fighter);
    let out = send(
        &mut s,
        conn(fighter.index()),
        join(fighter.raw(), Some(&fighter_token)),
    );
    let mut expected = vec![GameEntry::Resumed];
    expected.extend(answers);
    assert_eq!(s.drain_entries().collect::<Vec<_>>(), expected);
    let heard = to(&out, conn(fighter.index()));
    assert!(matches!(
        heard[..2],
        [
            ServerMessage::GameJoined { .. },
            ServerMessage::GameSnapshot { .. }
        ]
    ));
    assert!(
        heard[2..]
            .iter()
            .any(|m| matches!(m, ServerMessage::GameEvents { .. }))
    );
    assert!(matches!(heard.last(), Some(ServerMessage::GameTurn { .. })));
    assert_eq!(s.state().state_hash(), back.state().state_hash());
}
