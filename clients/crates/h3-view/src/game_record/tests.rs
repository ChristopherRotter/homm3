use super::*;
use alloc::string::{String, ToString};
use alloc::vec;

use h3_protocol::wire::game::{AdventureCommand, GameKeyTables, MapIdentity};
use h3_protocol::wire::{Command, KeyTables, RejectCode, WireU64};

use crate::record::tests::{manifest as battle_manifest, observation};

/// A whole game's manifest with the battle's `battle_manifest` part.
fn game_manifest() -> GameManifest {
    let b = battle_manifest(KeyTables {
        creatures: vec!["base:pikeman".to_string()],
        ..KeyTables::default()
    });
    GameManifest {
        protocol: b.protocol,
        engine: b.engine,
        packs: b.packs,
        profile: b.profile,
        ruleset_hash: WireU64(3),
        battle_ruleset_hash: WireU64(4),
        adventure_ruleset_hash: WireU64(5),
        town_ruleset_hash: WireU64(6),
        rng: b.rng,
        hash: b.hash,
        battle_state_hash_version: 7,
        adventure_state_hash_version: 8,
        game_state_hash_version: 9,
        map: MapIdentity {
            content_hash: WireU64(10),
            import_version: 11,
        },
        field: b.field,
        keys: b.keys,
        game_keys: GameKeyTables::default(),
    }
}

fn header() -> GameRecordHeader {
    GameRecordHeader {
        format: RECORD_FORMAT_VERSION,
        game: 4,
        player: 0,
        manifest: game_manifest(),
        first: 1,
    }
}

fn game(messages: Vec<ServerMessage>) -> GameRecord {
    GameRecord {
        header: header(),
        messages,
    }
}

fn text(header: &GameRecordHeader, messages: &[ServerMessage]) -> String {
    let mut lines = vec![codec::encode_text(header)];
    lines.extend(messages.iter().map(codec::encode_text));
    lines.join("\n")
}

fn events(n: u32, events: Vec<GameEvent>) -> ServerMessage {
    ServerMessage::GameEvents { n, events }
}

fn battle(event: Event) -> GameEvent {
    GameEvent::Battle { event }
}

fn snapshot(n: u32, observation: Observation) -> ServerMessage {
    ServerMessage::Snapshot { n, observation }
}

/// The attacker's view of a battle.
fn attacker() -> Observation {
    Observation {
        spectator: false,
        ..observation()
    }
}

const TILE: Pos = Pos {
    x: 15,
    y: 4,
    level: 0,
};

const WON: Outcome = Outcome::Victory {
    side: Side::Attacker,
};

#[test]
fn a_game_s_record_reads_and_is_told_from_a_battle_s() {
    let messages = [ServerMessage::GameEvents {
        n: 1,
        events: Vec::new(),
    }];
    let good = text(&header(), &messages);
    assert!(is_game(&good));
    assert_eq!(parse(&good), Ok(game(messages.to_vec())));
    // A battle's own record, no text and a line that is no message are not a game's.
    let battle = codec::encode_text(&RecordHeader {
        format: RECORD_FORMAT_VERSION,
        game: 1,
        seat: Seat::Spectator,
        manifest: battle_manifest(KeyTables::default()),
        first: 1,
    });
    assert!(!is_game(&battle));
    assert!(matches!(parse(&battle), Err(RecordError::Line(1, _))));
    assert!(!is_game("\n \n"));
    assert_eq!(parse(" \n"), Err(RecordError::Empty));
    assert!(matches!(
        parse(&alloc::format!("{good}\n{{\"Nonsense\":{{}}}}")),
        Err(RecordError::Line(3, _))
    ));
    // Another format or protocol version is refused.
    let mut old = header();
    old.format = 3;
    assert_eq!(parse(&text(&old, &[])), Err(RecordError::Format(3)));
    let mut other = header();
    other.manifest.protocol = 1;
    assert_eq!(parse(&text(&other, &[])), Err(RecordError::Protocol(1)));
}

#[test]
fn the_battle_s_manifest_is_the_game_s_battle_part() {
    let game = game_manifest();
    let m = manifest(&game);
    assert_eq!(
        (
            m.ruleset_hash,
            m.battle_ruleset_hash,
            m.battle_state_hash_version
        ),
        (WireU64(3), WireU64(4), 7)
    );
    assert_eq!(
        (m.keys, m.field, m.profile),
        (game.keys, game.field, game.profile)
    );
}

/// The shape of a golden replay's (`sea-deck-monster-s1`, player 0): the adventure before, the
/// battle's start in the command's events, its snapshot, its turns, commands and events, its end
/// with the adventure's events after it, and the adventure again.
#[test]
fn a_battle_comes_out_as_a_seat_s_record_of_it() {
    let turn = Event::TurnStarted { stack: 0 };
    let wait = Event::Waited { stack: 0 };
    let end = Event::BattleEnded { result: WON };
    let wait_command = Command::Wait {};
    let mut later = attacker();
    later.round = 2;
    let adventure = GameEvent::TurnEnded { player: 0 };
    let messages = vec![
        ServerMessage::GameTurn {
            n: 1,
            legal: vec![GameCommand::EndTurn {}],
            castable: Vec::new(),
        },
        // Before the battle: no battle's.
        events(2, vec![battle(wait.clone()), adventure.clone()]),
        events(
            3,
            vec![
                adventure.clone(),
                GameEvent::BattleStarted {
                    heroes: [Some(0), None],
                    tile: TILE,
                },
                battle(turn.clone()),
            ],
        ),
        snapshot(4, attacker()),
        ServerMessage::GameTurn {
            n: 5,
            legal: vec![
                GameCommand::Battle {
                    command: wait_command,
                },
                GameCommand::EndTurn {},
            ],
            castable: vec![7],
        },
        ServerMessage::GameRejected {
            n: 6,
            seq: 1,
            code: GameReject::Battle {
                code: RejectCode::NotYourTurn,
            },
        },
        // Not the battle's: left out.
        ServerMessage::GameRejected {
            n: 7,
            seq: 2,
            code: GameReject::NoBattle {},
        },
        ServerMessage::GameAccepted {
            n: 8,
            seq: 3,
            command: GameCommand::Battle {
                command: wait_command,
            },
        },
        events(9, vec![battle(wait.clone()), battle(turn.clone())]),
        // A turn without a battle command, a message without the battle's events, an adventure
        // command's acceptance: none in the battle's record.
        ServerMessage::GameTurn {
            n: 10,
            legal: vec![GameCommand::EndTurn {}],
            castable: Vec::new(),
        },
        events(11, vec![adventure.clone()]),
        ServerMessage::GameAccepted {
            n: 12,
            seq: 4,
            command: GameCommand::EndTurn {},
        },
        snapshot(13, later.clone()),
        events(
            14,
            vec![
                battle(wait.clone()),
                battle(end.clone()),
                GameEvent::BattleEnded {
                    heroes: [Some(0), None],
                    result: WON,
                },
                adventure.clone(),
            ],
        ),
        // After it: the adventure's, and a turn's battle command (none open) left out.
        ServerMessage::GameTurn {
            n: 15,
            legal: vec![GameCommand::Battle {
                command: wait_command,
            }],
            castable: Vec::new(),
        },
        ServerMessage::GameAccepted {
            n: 16,
            seq: 5,
            command: GameCommand::Battle {
                command: wait_command,
            },
        },
        ServerMessage::GameRejected {
            n: 17,
            seq: 6,
            code: GameReject::Battle {
                code: RejectCode::BattleOver,
            },
        },
        ServerMessage::GameAccepted {
            n: 18,
            seq: 7,
            command: GameCommand::Adventure {
                command: AdventureCommand::Revisit { hero: 0 },
            },
        },
    ];
    let battles = game(messages).battles();
    assert_eq!(battles.len(), 1);
    let GameBattle { record, at } = &battles[0];
    assert_eq!(
        *at,
        InGame {
            heroes: [Some(0), None],
            tile: Some(TILE),
            opened: 4,
            ended: Some((14, WON)),
        }
    );
    assert_eq!(
        record.header,
        RecordHeader {
            format: RECORD_FORMAT_VERSION,
            game: 4,
            seat: Seat::Attacker,
            manifest: manifest(&game_manifest()),
            first: 1,
        }
    );
    let expected = vec![
        snapshot(1, attacker()),
        ServerMessage::Events {
            n: 2,
            events: vec![turn.clone()],
        },
        ServerMessage::YourTurn {
            n: 3,
            legal: vec![wait_command],
            castable: vec![7],
        },
        ServerMessage::Rejected {
            n: 4,
            seq: 1,
            code: RejectCode::NotYourTurn,
        },
        ServerMessage::Accepted {
            n: 5,
            seq: 3,
            command: wait_command,
        },
        ServerMessage::Events {
            n: 6,
            events: vec![wait.clone(), turn.clone()],
        },
        snapshot(7, later),
        ServerMessage::Events {
            n: 8,
            events: vec![wait, end],
        },
        ServerMessage::GameEnded { n: 9, result: WON },
    ];
    assert_eq!(record.messages, expected);
    // As a record of its own: the start's events are its opening, one command after it.
    assert_eq!(record.opening(), Some(&[turn][..]));
    assert_eq!(record.commands().count(), 2);
}

#[test]
fn a_start_is_its_snapshot_s_only_when_right_before_it() {
    // A start the next message is not a snapshot of (another player's battle) is dropped: the
    // later snapshot's battle has no start known. Its viewer the defender.
    let mut defender = attacker();
    defender.viewer = Side::Defender;
    let started = GameEvent::BattleStarted {
        heroes: [Some(1), Some(2)],
        tile: TILE,
    };
    let battles = game(vec![
        events(1, vec![started.clone(), battle(Event::Waited { stack: 1 })]),
        events(2, Vec::new()),
        snapshot(3, defender),
    ])
    .battles();
    assert_eq!(battles.len(), 1);
    assert_eq!(battles[0].record.header.seat, Seat::Defender);
    assert_eq!(
        battles[0].at,
        InGame {
            heroes: [None, None],
            tile: None,
            opened: 3,
            ended: None,
        }
    );
    // Its snapshot alone: a battle the record ends in.
    assert_eq!(battles[0].record.messages.len(), 1);
}

#[test]
fn a_battle_may_end_in_the_message_that_starts_it() {
    // Its end in its start's message: the snapshot after it opens and ends the battle; a
    // spectator's snapshot makes the record a spectator's. An end with no battle (another's)
    // changes nothing.
    let other = GameEvent::BattleEnded {
        heroes: [None, None],
        result: Outcome::Draw {},
    };
    let battles = game(vec![
        events(1, vec![other]),
        events(
            2,
            vec![
                GameEvent::BattleStarted {
                    heroes: [Some(0), None],
                    tile: TILE,
                },
                battle(Event::TurnStarted { stack: 0 }),
                GameEvent::BattleEnded {
                    heroes: [Some(0), None],
                    result: WON,
                },
            ],
        ),
        snapshot(3, observation()),
    ])
    .battles();
    assert_eq!(battles.len(), 1);
    let GameBattle { record, at } = &battles[0];
    assert_eq!(record.header.seat, Seat::Spectator);
    assert_eq!((at.opened, at.ended), (3, Some((2, WON))));
    assert_eq!(record.messages.len(), 3);
    assert!(matches!(
        record.messages[2],
        ServerMessage::GameEnded { n: 3, result: WON }
    ));
}

#[test]
fn a_start_while_a_battle_is_open_ends_that_one_unended() {
    let started = |hero| GameEvent::BattleStarted {
        heroes: [Some(hero), None],
        tile: TILE,
    };
    let battles = game(vec![
        events(1, vec![started(0)]),
        snapshot(2, attacker()),
        events(3, vec![started(1), battle(Event::TurnStarted { stack: 4 })]),
        snapshot(4, attacker()),
    ])
    .battles();
    assert_eq!(battles.len(), 2);
    assert_eq!(
        (battles[0].at.heroes, battles[0].at.ended),
        ([Some(0), None], None)
    );
    assert_eq!(battles[0].record.messages.len(), 1);
    assert_eq!(
        (battles[1].at.heroes, battles[1].at.opened),
        ([Some(1), None], 4)
    );
    assert_eq!(
        battles[1].record.opening(),
        Some(&[Event::TurnStarted { stack: 4 }][..])
    );
}
