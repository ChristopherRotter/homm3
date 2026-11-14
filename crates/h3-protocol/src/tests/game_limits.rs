//! The host's bounds on a seat's commands (E16 M5 T18): a turn's map commands rejected past
//! their bound, ending the turn never, each turn counted afresh and a restore counting again; a
//! battle's commands past their bound played by the computer to the battle's end, the seat's own
//! rejected, a battle's commands never counted against the turn, a reopen counting again;
//! `GameTurn` leaving out what a bound refuses.

use std::boxed::Box;
use std::vec::Vec;

use h3_battle::{BattleSide, Command};
use h3_contracts::PlayerId;
use h3_game::{
    Actor, GameCommand as Engine, GameRuleset, Manage, SystemCommand, legal_commands,
    legal_management,
};

use super::game_clock::{Striker, pick};
use super::game_fold::one;
use super::game_seats::{MAP, OPTIONS, join, send, to, token};
use super::game_session::duel;
use crate::codec;
use crate::map::game::{command, manifest};
use crate::session::ConnId;
use crate::session::game::{CommandBounds, GameEntry, GameOptions, GameOutbound, GameSession};
use crate::wire::game::{GameCommand, GameReject};
use crate::wire::{ClientMessage, ServerMessage};

const P0: PlayerId = PlayerId::new(0);
const P1: PlayerId = PlayerId::new(1);

/// Game 9 under the bounds `per_turn` and `per_battle`.
const fn bounded(per_turn: u32, per_battle: u32) -> GameOptions {
    GameOptions {
        commands: CommandBounds {
            per_turn,
            per_battle,
        },
        ..OPTIONS
    }
}

/// A formation of `player`'s first hero (GAM-MAN-001): a map command that costs nothing, as
/// often as the player likes; none without a hero.
fn formation(s: &GameSession<'_>, rules: &GameRuleset, player: PlayerId) -> Option<GameCommand> {
    let state = s.state();
    let hero = (state.adventure().heroes().iter()).find(|h| h.owner == player)?;
    let legal = legal_management(state, rules, Actor::Player(player), hero.id);
    let free = legal
        .into_iter()
        .find(|m| matches!(m, Manage::Formation { .. }))?;
    Some(command(&Engine::Manage(free)))
}

/// Whether `command` is a map command: an adventure's, a management one, a town's.
const fn on_map(command: &GameCommand) -> bool {
    matches!(
        command,
        GameCommand::Adventure { .. } | GameCommand::Manage { .. } | GameCommand::Town { .. }
    )
}

/// The legal commands of each `GameTurn` in `messages`.
fn turns(messages: &[ServerMessage]) -> Vec<&Vec<GameCommand>> {
    let turns = messages.iter().filter_map(|m| match m {
        ServerMessage::GameTurn { legal, .. } => Some(legal),
        _ => None,
    });
    turns.collect()
}

/// `player`'s messages in `out`, decoded.
fn told(out: &[GameOutbound], player: PlayerId) -> Vec<ServerMessage> {
    let frames = out.iter().filter(|o| o.to == player);
    frames
        .map(|o| codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap())
        .collect()
}

#[test]
fn a_turns_map_commands_past_their_bound_are_rejected_but_ending_it_never() {
    let (rules, setup) = duel();
    let mut out = Vec::new();
    let manifest = manifest(rules, MAP);
    let start = |options, entries: &[GameEntry]| {
        let manifest = manifest.clone();
        let mut out = Vec::new();
        GameSession::restore(rules, &setup, 1, manifest, options, entries, &mut out).unwrap()
    };
    let mut s =
        GameSession::start(rules, &setup, 1, manifest.clone(), bounded(3, 1), &mut out).unwrap();
    let free = formation(&s, rules, P0).unwrap();
    for seq in 0..3 {
        out.clear();
        s.play(P0, seq, free.clone(), &mut out).unwrap();
    }
    // Its turn then leaves its map commands out: it may end it.
    let messages = told(&out, P0);
    let legal = turns(&messages);
    assert_eq!(legal.len(), 1);
    assert!(legal[0].contains(&GameCommand::EndTurn {}));
    assert!(!legal[0].iter().any(on_map), "{legal:?}");
    let entries: Vec<GameEntry> = s.drain_entries().collect();
    assert_eq!(entries.len(), 3);
    // The fourth is rejected, told to its player alone; nothing changed or journaled.
    let hash = s.state().state_hash();
    out.clear();
    let refused = s.play(P0, 3, free.clone(), &mut out);
    assert_eq!(refused, Err(GameReject::TurnLimit {}));
    assert_eq!(s.state().state_hash(), hash);
    assert_eq!(s.drain_entries().count(), 0);
    assert!(out.iter().all(|o| o.to == P0));
    assert!(matches!(
        told(&out, P0)[..],
        [ServerMessage::GameRejected {
            seq: 3,
            code: GameReject::TurnLimit {},
            ..
        }]
    ));
    // A player outside the game is the engine's to reject.
    let outside = s.play(PlayerId::new(7), 3, free.clone(), &mut out);
    assert_eq!(outside, Err(GameReject::UnknownPlayer { player: 7 }));
    // A restore counts again; the bounds are the hosting's.
    let mut restored = start(bounded(3, 1), &entries);
    let again = restored.play(P0, 3, free.clone(), &mut Vec::new());
    assert_eq!(again, Err(GameReject::TurnLimit {}));
    let mut unbounded = start(OPTIONS, &entries);
    unbounded.play(P0, 3, free, &mut Vec::new()).unwrap();
    // Ending the turn is never refused; each turn counts afresh, the other player's apart.
    s.play(P0, 4, GameCommand::EndTurn {}, &mut out).unwrap();
    let other = formation(&s, rules, P1).unwrap();
    for seq in 0..3 {
        s.play(P1, seq, other.clone(), &mut out).unwrap();
    }
    let refused = s.play(P1, 3, other, &mut out);
    assert_eq!(refused, Err(GameReject::TurnLimit {}));
    s.play(P1, 4, GameCommand::EndTurn {}, &mut out).unwrap();
    assert!(s.state().is_acting(P0) && s.state().day() == 2);
    let free = formation(&s, rules, P0).unwrap();
    s.play(P0, 5, free, &mut out).unwrap();
}

/// Player `player`'s connection.
const fn conn(player: u8) -> ConnId {
    ConnId(400 + player as u64)
}

#[test]
fn a_battles_commands_past_their_bound_are_the_computers_to_its_end() {
    // Player 0's hero attacks player 1's at command 0: its turn's one map command.
    let (rules, setup, replay) = one("towns-visitor-s1");
    let manifest = manifest(&rules, MAP);
    let options = bounded(1, 2);
    let mut s = GameSession::new(
        &rules,
        &setup,
        replay.seed,
        manifest.clone(),
        options,
        token,
    )
    .unwrap();
    for player in 0..2 {
        send(&mut s, conn(player), join(player, None));
    }
    s.computer(Box::new(Striker(7)), &mut Vec::new());
    let attack = command(&replay.commands[0].1);
    let message = |seq, command| ClientMessage::GameCommand { seq, command };
    send(&mut s, conn(0), message(0, attack));
    assert_eq!(s.state().battle(P0).unwrap().owners(), [Some(P0), Some(P1)]);
    let mut entries: Vec<GameEntry> = s.drain_entries().collect();
    // Each side plays its two commands (the attacker's turn bound never counts them), then the
    // computer plays it to the battle's end; a seat past its bound is refused its own.
    let (mut seqs, mut spent, mut stream, mut refused) = ([1, 0], [0, 0], 1, 0);
    let mut cut = None;
    while let Some(battle) = s.state().battle(P0) {
        let side = battle.state().side_to_act().unwrap();
        let player = usize::from(side == BattleSide::Defender);
        // The computer plays a side past its bound at once: the side to act is under it.
        assert!(spent[player] < 2);
        let other = 1 - player;
        if spent[other] == 2 {
            let defend = command(&Engine::Battle(Command::Defend));
            let out = send(&mut s, conn(other as u8), message(seqs[other], defend));
            let code = GameReject::BattleLimit {};
            assert!(matches!(
                to(&out, conn(other as u8))[..],
                [ServerMessage::GameRejected { code: c, .. }] if c == code
            ));
            assert_eq!(s.drain_entries().count(), 0);
            refused += 1;
        }
        let actor = Actor::Player(PlayerId::new(player as u8));
        let legal = legal_commands(s.state(), &rules, actor);
        let battle = legal.iter().filter_map(|c| match c {
            Engine::Battle(c) => Some(*c),
            _ => None,
        });
        let choice = command(&Engine::Battle(pick(&mut stream, battle)));
        let out = send(&mut s, conn(player as u8), message(seqs[player], choice));
        seqs[player] += 1;
        spent[player] += 1;
        if spent[player] == 2 {
            // Past its bound its turns leave its battle commands out.
            let messages = to(&out, conn(player as u8));
            let legal = turns(&messages);
            assert!(
                legal
                    .iter()
                    .flat_map(|l| l.iter())
                    .all(|c| !matches!(c, GameCommand::Battle { .. }))
            );
        }
        entries.extend(s.drain_entries());
        if spent[player] == 2 && cut.is_none() && s.state().battle(P0).is_some() {
            cut = Some((entries.len(), s.spent()));
        }
    }
    assert_eq!(spent, [2, 2]);
    assert!(refused > 0);
    let played = entries.iter().filter(|e| {
        matches!(
            e,
            GameEntry::Command(Actor::System, Engine::System(SystemCommand::Battle { .. }))
        )
    });
    assert!(
        played.count() > 0,
        "the sides past their bound played by the computer"
    );
    // The attacker's turn bound, which its battle commands never counted against: its map
    // commands refused, its turn's end accepted.
    let refused = (formation(&s, &rules, P0).into_iter())
        .chain(
            legal_commands(s.state(), &rules, Actor::Player(P0))
                .iter()
                .map(command),
        )
        .find(on_map)
        .unwrap();
    let out = send(&mut s, conn(0), message(seqs[0], refused));
    assert!(matches!(
        to(&out, conn(0))[..],
        [
            ServerMessage::GameRejected {
                code: GameReject::TurnLimit {},
                ..
            },
            ..
        ]
    ));
    send(&mut s, conn(0), message(seqs[0], GameCommand::EndTurn {}));
    assert!(!s.state().is_acting(P0));
    // A reopen inside the battle, once a side spent its bound, counts again.
    let (at, counted) = cut.unwrap();
    let reopened = GameSession::reopen(
        &rules,
        &setup,
        replay.seed,
        manifest,
        options,
        token,
        &entries[..at],
    )
    .unwrap();
    assert!(reopened.state().battle(P0).is_some());
    assert_eq!(reopened.spent(), counted);
    assert_eq!(counted.1, 2);
}
