use super::*;
use alloc::format;
use alloc::string::ToString;
use h3_protocol::RECORD_FORMAT_VERSION;
use h3_protocol::wire::{KeyTables, Outcome, RecordHeader, RejectCode, WireU64};

use crate::record::tests::{manifest, observation};

const ADDRESS: &str = "ws://127.0.0.1:7000";
const NONCE: [u8; NONCE_BYTES] = [7; NONCE_BYTES];
const CHALLENGE: [u8; CHALLENGE_BYTES] = [9; CHALLENGE_BYTES];
const KEY: [u8; KEY_BYTES] = [1; KEY_BYTES];

/// A stand-in signature: the key, then the signed bytes folded into 32 by XOR.
fn sign(key: &[u8; KEY_BYTES], message: &[u8]) -> [u8; SIGNATURE_BYTES] {
    let mut signature = [0; SIGNATURE_BYTES];
    signature[..KEY_BYTES].copy_from_slice(key);
    for (i, b) in message.iter().enumerate() {
        signature[KEY_BYTES + i % KEY_BYTES] ^= b;
    }
    signature
}

fn verify(key: &[u8; KEY_BYTES], message: &[u8], signature: &[u8; SIGNATURE_BYTES]) -> bool {
    sign(key, message) == *signature
}

/// The welcome a server with `key` sends at `id` to the hello of [`NONCE`].
fn welcome(id: &str, key: &[u8; KEY_BYTES]) -> ServerMessage {
    let signed = account::welcome_message(id, &NONCE, &CHALLENGE);
    ServerMessage::Welcome {
        protocol: PROTOCOL_VERSION,
        server: "h3-server test".to_string(),
        id: id.to_string(),
        challenge: account::to_hex(&CHALLENGE),
        key: account::to_hex(key),
        signature: account::to_hex(&sign(key, &signed)),
    }
}

/// A client welcomed at [`ADDRESS`] by [`KEY`], pinned before.
fn welcomed() -> Client {
    let mut client = Client::new(ADDRESS, NONCE, Some(KEY));
    client.receive(welcome(ADDRESS, &KEY), verify).unwrap();
    client
}

fn snapshot(n: u32) -> ServerMessage {
    ServerMessage::Snapshot {
        n,
        observation: observation(),
    }
}

fn your_turn(n: u32) -> ServerMessage {
    ServerMessage::YourTurn {
        n,
        legal: vec![Command::Wait {}, Command::Defend {}],
        castable: vec![3],
    }
}

fn header(game: u32, seat: Seat, first: u32) -> RecordHeader {
    RecordHeader {
        format: RECORD_FORMAT_VERSION,
        game,
        seat,
        manifest: manifest(KeyTables::default()),
        first,
    }
}

/// The `Record` message of `header` and `messages`, the last numbered `n`.
fn record(header: &RecordHeader, messages: &[ServerMessage], n: u32) -> ServerMessage {
    let mut text = codec::encode_text(header);
    for m in messages {
        text.push('\n');
        text.push_str(&codec::encode_text(m));
    }
    ServerMessage::Record {
        n,
        seq: 1,
        record: text,
    }
}

fn joined(game: u32, seat: Seat, token: Option<&str>) -> ServerMessage {
    ServerMessage::Joined {
        game,
        seat,
        token: token.map(ToString::to_string),
        manifest: manifest(KeyTables::default()),
    }
}

/// A client that joined game 4 as the attacker by its token and was asked for its record.
fn fetching() -> Client {
    let mut client = welcomed();
    client
        .join(4, Seat::Attacker, Some("t".to_string()))
        .unwrap();
    client
        .receive(joined(4, Seat::Attacker, None), verify)
        .unwrap();
    client
}

#[test]
fn the_hello_names_the_nonce_and_the_address_dialed() {
    let client = Client::new(ADDRESS, NONCE, None);
    assert_eq!(
        client.hello("h3-view", "Ann"),
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            client: "h3-view".to_string(),
            name: "Ann".to_string(),
            nonce: account::to_hex(&NONCE),
            address: ADDRESS.to_string(),
        }
    );
}

#[test]
fn the_welcome_pins_the_key_at_first_contact_and_then_holds_it_to_it() {
    let mut first = Client::new(ADDRESS, NONCE, None);
    assert_eq!(
        first.receive(welcome(ADDRESS, &KEY), verify),
        Ok(vec![
            Note::Pinned(KEY),
            Note::Welcomed("h3-server test".to_string())
        ])
    );
    let mut again = Client::new(ADDRESS, NONCE, Some(KEY));
    assert_eq!(
        again.receive(welcome(ADDRESS, &KEY), verify),
        Ok(vec![Note::Welcomed("h3-server test".to_string())])
    );
    // Another key at the address: a server that is not the one pinned, signing correctly.
    let mut other = Client::new(ADDRESS, NONCE, Some([2; KEY_BYTES]));
    assert_eq!(
        other.receive(welcome(ADDRESS, &KEY), verify),
        Err(Fault::KeyChanged {
            pinned: [2; KEY_BYTES],
            offered: KEY
        })
    );
}

#[test]
fn a_welcome_that_does_not_hold_ends_the_connection() {
    let fails = |message: ServerMessage| Client::new(ADDRESS, NONCE, None).receive(message, verify);
    let mut old = welcome(ADDRESS, &KEY);
    if let ServerMessage::Welcome { protocol, .. } = &mut old {
        *protocol = 1;
    }
    assert_eq!(fails(old), Err(Fault::Protocol(1)));
    // Signed correctly, but for another address: a login signed for it could be relayed.
    let elsewhere = "ws://10.0.0.1:7000";
    assert_eq!(
        fails(welcome(elsewhere, &KEY)),
        Err(Fault::NotDialed(elsewhere.to_string()))
    );
    for field in 0..3 {
        let mut bad = welcome(ADDRESS, &KEY);
        if let ServerMessage::Welcome {
            challenge,
            key,
            signature,
            ..
        } = &mut bad
        {
            [challenge, key, signature][field].push('0');
        }
        assert_eq!(fails(bad), Err(Fault::Malformed));
    }
    let mut forged = welcome(ADDRESS, &KEY);
    if let ServerMessage::Welcome { signature, .. } = &mut forged {
        *signature = account::to_hex(&[0; SIGNATURE_BYTES]);
    }
    assert_eq!(fails(forged), Err(Fault::Signature));
    // Anything but the welcome or a goodbye first.
    assert_eq!(
        fails(snapshot(1)),
        Err(Fault::OutOfPlace("Snapshot".to_string()))
    );
}

#[test]
fn a_goodbye_closes_the_connection_in_any_state() {
    let mut client = Client::new(ADDRESS, NONCE, None);
    let bye = ServerMessage::Goodbye {
        reason: GoodbyeReason::Protocol,
    };
    assert_eq!(
        client.receive(bye.clone(), verify),
        Ok(vec![Note::Goodbye(GoodbyeReason::Protocol)])
    );
    assert!(client.closed());
    assert_eq!(
        client.receive(welcome(ADDRESS, &KEY), verify),
        Err(Fault::OutOfPlace("Welcome".to_string()))
    );
    let mut playing = welcomed();
    assert!(!playing.closed());
    assert_eq!(
        playing.receive(bye, verify),
        Ok(vec![Note::Goodbye(GoodbyeReason::Protocol)])
    );
    assert!(playing.closed() && playing.join(1, Seat::Spectator, None).is_none());
}

#[test]
fn a_join_waits_for_the_welcome_and_a_refusal_lets_the_client_join_again() {
    let mut client = Client::new(ADDRESS, NONCE, None);
    assert_eq!(client.join(4, Seat::Spectator, None), None);
    let mut client = welcomed();
    assert_eq!(client.command(Command::Wait {}), None);
    assert_eq!(
        client.join(4, Seat::Defender, Some("t".to_string())),
        Some(ClientMessage::Join {
            game: 4,
            seat: Seat::Defender,
            token: Some("t".to_string())
        })
    );
    assert_eq!(client.token(), Some("t"));
    assert_eq!(client.join(5, Seat::Defender, None), None);
    // The game's answer for another game or seat is out of place.
    assert_eq!(
        client
            .clone()
            .receive(joined(4, Seat::Attacker, None), verify),
        Err(Fault::OutOfPlace("Joined".to_string()))
    );
    let refused = ServerMessage::Refused {
        reason: Refusal::NoSuchGame,
    };
    assert_eq!(
        client.receive(refused.clone(), verify),
        Ok(vec![Note::Refused(Refusal::NoSuchGame)])
    );
    assert!(client.join(5, Seat::Spectator, None).is_some());
    assert_eq!(client.token(), None);
    // A refusal of anything else keeps the state (a request in the lobby).
    let mut lobby = welcomed();
    assert_eq!(
        lobby.receive(refused, verify),
        Ok(vec![Note::Refused(Refusal::NoSuchGame)])
    );
    assert!(lobby.join(5, Seat::Spectator, None).is_some());
    // No numbered message before a seat is joined.
    assert_eq!(
        welcomed().receive(snapshot(1), verify),
        Err(Fault::OutOfPlace("Snapshot".to_string()))
    );
    assert_eq!(
        welcomed().receive(welcome(ADDRESS, &KEY), verify),
        Err(Fault::OutOfPlace("Welcome".to_string()))
    );
}

#[test]
fn a_game_created_is_joined_and_a_refusal_lets_the_client_try_again() {
    assert_eq!(
        Client::new(ADDRESS, NONCE, None).create("duel", Seat::Attacker),
        None
    );
    let mut client = welcomed();
    assert_eq!(
        client.create("duel", Seat::Attacker),
        Some(ClientMessage::CreateGame {
            seq: 1,
            offer: "duel".to_string(),
            seat: Seat::Attacker,
            spectators: None,
        })
    );
    assert_eq!(client.create("duel", Seat::Attacker), None);
    let refused = ServerMessage::Refused {
        reason: Refusal::NoSuchOffer,
    };
    assert_eq!(
        client.receive(refused, verify),
        Ok(vec![Note::Refused(Refusal::NoSuchOffer)])
    );
    assert!(client.create("duel", Seat::Defender).is_some());
    let created = ServerMessage::GameCreated { seq: 2, game: 9 };
    assert_eq!(
        client.receive(created.clone(), verify),
        Ok(vec![Note::Created(9)])
    );
    // Created once: a second answer is out of place.
    assert_eq!(
        client.clone().receive(created, verify),
        Err(Fault::OutOfPlace("GameCreated".to_string()))
    );
    assert_eq!(
        client.receive(joined(9, Seat::Defender, Some("d")), verify),
        Ok(vec![
            Note::Joined(9, Seat::Defender),
            Note::Send(ClientMessage::FetchRecord { seq: 3 })
        ])
    );
    assert_eq!(client.token(), Some("d"));
}

#[test]
fn the_join_asks_for_the_record_and_holds_what_comes_meanwhile() {
    let mut client = welcomed();
    client
        .join(4, Seat::Attacker, Some("t".to_string()))
        .unwrap();
    // The seat's token as the game gives it (a seat taken the first time).
    assert_eq!(
        client.receive(joined(4, Seat::Attacker, Some("u")), verify),
        Ok(vec![
            Note::Joined(4, Seat::Attacker),
            Note::Send(ClientMessage::FetchRecord { seq: 1 })
        ])
    );
    assert_eq!(client.token(), Some("u"));
    assert_eq!(client.record(), None);
    assert_eq!(client.turn(), None);
    assert!(!client.over());
    assert_eq!(client.command(Command::Wait {}), None);
    // The join's snapshot and turn, then an event the record does not hold yet.
    let events = ServerMessage::Events {
        n: 4,
        events: Vec::new(),
    };
    for m in [snapshot(2), your_turn(3), events.clone()] {
        assert_eq!(client.receive(m, verify), Ok(Vec::new()));
    }
    // The record holds the messages before the join (a re-join: from 1) up to its turn.
    let messages = [snapshot(1), snapshot(2), your_turn(3)];
    let text = record(&header(4, Seat::Attacker, 1), &messages, 3);
    assert_eq!(client.receive(text, verify), Ok(vec![Note::Opened]));
    let opened = client.record().unwrap();
    assert_eq!(opened.header.first, 1);
    assert_eq!(opened.messages, [&messages[..], &[events][..]].concat());
    // The turn the seat is asked to play stands after the events.
    assert_eq!(
        client.turn(),
        Some((&[Command::Wait {}, Command::Defend {}][..], &[3][..]))
    );
    assert_eq!(
        client.command(Command::Defend {}),
        Some(ClientMessage::Command {
            seq: 2,
            command: Command::Defend {}
        })
    );
    // A rejection keeps the turn; an acceptance ends it.
    let rejected = ServerMessage::Rejected {
        n: 5,
        seq: 2,
        code: RejectCode::NotSeated,
    };
    assert_eq!(client.receive(rejected, verify), Ok(vec![Note::Grew]));
    assert!(client.turn().is_some());
    let accepted = ServerMessage::Accepted {
        n: 6,
        seq: 3,
        command: Command::Defend {},
    };
    assert_eq!(client.receive(accepted, verify), Ok(vec![Note::Grew]));
    assert_eq!(client.turn(), None);
    assert!(!client.over());
    // The rejection of a connection without a seat and a pong are not numbered.
    let unseated = ServerMessage::Rejected {
        n: 0,
        seq: 9,
        code: RejectCode::NotSeated,
    };
    assert_eq!(
        client.receive(unseated.clone(), verify),
        Ok(vec![Note::Other(Box::new(unseated))])
    );
    let pong = ServerMessage::Pong { nonce: 1 };
    assert_eq!(
        client.receive(pong.clone(), verify),
        Ok(vec![Note::Other(Box::new(pong))])
    );
    // The end; a turn before it no longer stands.
    client.receive(your_turn(7), verify).unwrap();
    let ended = ServerMessage::GameEnded {
        n: 8,
        result: Outcome::Draw {},
    };
    assert_eq!(client.receive(ended, verify), Ok(vec![Note::Grew]));
    assert!(client.over());
    assert_eq!(client.turn(), None);
    assert_eq!(client.record().unwrap().messages.len(), 8);
    // A gap, a second record and a second welcome end the connection.
    assert_eq!(
        client.clone().receive(snapshot(10), verify),
        Err(Fault::Gap {
            expected: 9,
            got: 10
        })
    );
    let again = record(&header(4, Seat::Attacker, 1), &messages, 3);
    assert_eq!(
        client.clone().receive(again, verify),
        Err(Fault::OutOfPlace("Record".to_string()))
    );
}

#[test]
fn the_record_opens_at_its_first_snapshot() {
    // A record whose bound let its start go: the events before the snapshot are dropped.
    let mut client = fetching();
    let events = ServerMessage::Events {
        n: 5,
        events: Vec::new(),
    };
    let text = record(
        &header(4, Seat::Attacker, 5),
        &[events, snapshot(6), your_turn(7)],
        7,
    );
    client.receive(text, verify).unwrap();
    let opened = client.record().unwrap();
    assert_eq!(opened.header.first, 6);
    assert_eq!(opened.messages, [snapshot(6), your_turn(7)]);
    assert_eq!(client.receive(snapshot(8), verify), Ok(vec![Note::Grew]));
    // A record without a turn asked (a spectator's): no turn.
    let mut spectator = fetching();
    let text = record(&header(4, Seat::Attacker, 1), &[snapshot(1)], 1);
    spectator.receive(text, verify).unwrap();
    assert_eq!(spectator.turn(), None);
}

#[test]
fn a_record_that_does_not_hold_ends_the_connection() {
    let opens = |message: ServerMessage| fetching().receive(message, verify);
    let bad = ServerMessage::Record {
        n: 1,
        seq: 1,
        record: String::new(),
    };
    assert_eq!(opens(bad), Err(Fault::Record(RecordError::Empty)));
    for (game, seat) in [(5, Seat::Attacker), (4, Seat::Defender)] {
        let other = record(&header(game, seat, 1), &[snapshot(1)], 1);
        assert_eq!(opens(other), Err(Fault::NotTheSeat));
    }
    let empty = record(&header(4, Seat::Attacker, 1), &[], 0);
    assert_eq!(opens(empty), Err(Fault::NoSnapshot));
    // A message held meanwhile that does not follow the record's last.
    let mut client = fetching();
    client.receive(snapshot(3), verify).unwrap();
    let text = record(&header(4, Seat::Attacker, 1), &[snapshot(1)], 1);
    assert_eq!(
        client.receive(text, verify),
        Err(Fault::Gap {
            expected: 2,
            got: 3
        })
    );
}

#[test]
fn a_fault_says_what_ended_the_connection() {
    for (fault, shown) in [
        (Fault::Protocol(1), "the server speaks protocol 1"),
        (
            Fault::NotDialed("ws://x".to_string()),
            "the welcome names ws://x, not the address dialed",
        ),
        (
            Fault::Malformed,
            "the welcome's key, challenge or signature",
        ),
        (Fault::Signature, "the welcome's signature does not verify"),
        (
            Fault::KeyChanged {
                pinned: [0; KEY_BYTES],
                offered: [255; KEY_BYTES],
            },
            "THE SERVER'S KEY CHANGED: pinned 0000",
        ),
        (
            Fault::OutOfPlace("Snapshot".to_string()),
            "a message out of place: Snapshot",
        ),
        (
            Fault::Record(RecordError::Empty),
            "the record: the record is empty",
        ),
        (Fault::NotTheSeat, "the record is not the joined seat's"),
        (Fault::NoSnapshot, "the record holds no snapshot"),
        (
            Fault::Gap {
                expected: 2,
                got: 3,
            },
            "message 3 came where 2 was expected",
        ),
    ] {
        assert!(fault.to_string().starts_with(shown), "{fault}");
    }
}

#[test]
fn numbered_messages_are_the_viewer_s() {
    assert_eq!(number(&snapshot(4)), Some(4));
    assert_eq!(number(&your_turn(5)), Some(5));
    let pause = ServerMessage::PauseAsked {
        n: 6,
        seat: Seat::Defender,
    };
    assert_eq!(number(&pause), Some(6));
    let points = r#"{"rating":1500,"deviation":350}"#;
    for (i, json) in [
        r#"{"Accepted":{"n":1,"seq":1,"command":{"Wait":{}}}}"#.to_string(),
        r#"{"QueryResult":{"n":1,"seq":1,"targets":[]}}"#.to_string(),
        r#"{"Events":{"n":1,"events":[]}}"#.to_string(),
        r#"{"GameEnded":{"n":1,"result":{"Draw":{}}}}"#.to_string(),
        r#"{"PauseProposed":{"n":1,"seat":"Attacker","ms":5}}"#.to_string(),
        r#"{"PauseAnswered":{"n":1,"seat":"Defender","accept":true}}"#.to_string(),
        r#"{"SpectatorsSet":{"n":1,"seq":1,"open":true}}"#.to_string(),
        format!(r#"{{"Rating":{{"n":1,"ladder":"l","before":{points},"after":{points}}}}}"#),
    ]
    .iter()
    .enumerate()
    {
        let message = codec::decode_server(json.as_bytes(), usize::MAX).unwrap();
        assert_eq!(number(&message), Some(1), "{i}");
    }
    let rejected = |n| ServerMessage::Rejected {
        n,
        seq: 1,
        code: RejectCode::NotSeated,
    };
    assert_eq!(number(&rejected(7)), Some(7));
    assert_eq!(number(&rejected(0)), None);
    assert_eq!(number(&ServerMessage::Pong { nonce: 1 }), None);
}

#[test]
fn the_content_check_compares_the_hashes() {
    let game = manifest(KeyTables::default());
    assert_eq!(content(&game, None), Content::Absent);
    let mut own = game.clone();
    own.engine = "another".to_string();
    assert_eq!(content(&game, Some(&own)), Content::Same);
    own.ruleset_hash = WireU64(1);
    assert_eq!(content(&game, Some(&own)), Content::Different);
    let mut battle = game.clone();
    battle.battle_ruleset_hash = WireU64(1);
    assert_eq!(content(&game, Some(&battle)), Content::Different);
}

#[test]
fn pins_read_and_write_a_line_per_address() {
    let (a, b) = ([1; KEY_BYTES], [2; KEY_BYTES]);
    let text = format!(
        "# pinned keys\n\nwss://b.example {}\n  ws://a.example {}  \n",
        account::to_hex(&b),
        account::to_hex(&a)
    );
    let mut pins = Pins::parse(&text).unwrap();
    assert_eq!(pins.key("ws://a.example"), Some(a));
    assert_eq!(pins.key("wss://b.example"), Some(b));
    assert_eq!(pins.key("ws://c.example"), None);
    assert_eq!(
        pins.text(),
        format!(
            "ws://a.example {}\nwss://b.example {}\n",
            account::to_hex(&a),
            account::to_hex(&b)
        )
    );
    pins.pin("ws://a.example", b);
    pins.pin("ws://c.example", a);
    assert_eq!(pins.key("ws://a.example"), Some(b));
    assert_eq!(Pins::parse(&pins.text()), Ok(pins));
    assert_eq!(Pins::parse(""), Ok(Pins::default()));
    let key = account::to_hex(&a);
    for (bad, line) in [
        ("ws://a.example".to_string(), 1),
        (format!("ws://a.example {key} more"), 1),
        ("ws://a.example 00".to_string(), 1),
        (format!("ws://a.example {key}\n#\nws://a.example {key}"), 3),
    ] {
        assert_eq!(Pins::parse(&bad), Err(line), "{bad}");
    }
}

/// The seat's battle on a server (E17 M5 T6b1, `seen`): the active stack and the round by the
/// panel's turns; the hexes by the moves, a teleport and a creation, in order; the counts and the
/// deaths as the timeline shows them; a created stack in the dead one's id; the heroes' mana by
/// the casts' costs, the mana channelled and drained; the gate.
#[test]
fn the_seat_sees_the_moves_deaths_creations_and_mana() {
    use crate::anim::tests::created::{SUMMON, play_created, spell, summoned};
    use crate::anim::tests::{PIKEMAN, SWORDSMAN, at, flags, moved, snapshot, struck};
    use h3_protocol::wire::{Event, Hero, Side, Target};
    let mut opening = snapshot(vec![
        at(0, Side::Attacker, SWORDSMAN, 86, false),
        at(1, Side::Defender, PIKEMAN, 50, false),
        at(2, Side::Defender, PIKEMAN, 120, false),
    ]);
    let hero = |mana| {
        Some(Hero {
            hero_type: 0,
            attack: 1,
            defense: 1,
            power: 1,
            knowledge: 1,
            mana,
            max_mana: 30,
            casts: 0,
            commands_ballista: false,
            commands_tent: false,
            commands_catapult: false,
            spell_book: true,
            sex: None,
        })
    };
    opening.sides[0].hero = hero(20);
    opening.sides[1].hero = hero(5);
    let commands = [
        (
            Some(0),
            vec![Event::TurnStarted { stack: 0 }, moved(0, 86, 87)],
        ),
        (
            None,
            vec![
                Event::TurnStarted { stack: 1 },
                struck(0, 1, 40, flags(false, false, false)),
            ],
        ),
        (
            None,
            vec![
                Event::RoundStarted { round: 2 },
                Event::TurnStarted { stack: 0 },
                spell(SUMMON, Target::None {}),
                summoned(1, 6, 18),
            ],
        ),
        // A second summon takes a new id.
        (
            None,
            vec![
                spell(SUMMON, Target::None {}),
                summoned(3, 4, 30),
                Event::TurnStarted { stack: 2 },
            ],
        ),
    ];
    let timeline = play_created(&opening, &commands);
    let mut events: Vec<Event> = commands.iter().flat_map(|(_, e)| e.clone()).collect();
    events.extend([
        Event::Teleported {
            stack: 2,
            from: 120,
            to: 100,
        },
        Event::ManaChanneled {
            side: Side::Defender,
            mana: 4,
        },
        Event::ManaDrained {
            stack: 0,
            side: Side::Attacker,
            mana: 1,
        },
        Event::GateOpened {},
    ]);
    let cost = |side: Side, spell: u16| {
        if (side, spell) == (Side::Attacker, SUMMON) {
            6
        } else {
            0
        }
    };
    let now = seen(&opening, &events, &timeline, timeline.end, cost);
    assert_eq!((now.active, now.round), (Some(2), 2));
    let stack = |id: u8| now.stacks.iter().find(|s| s.id == id).unwrap();
    assert_eq!(
        (stack(0).hex, stack(0).alive, stack(0).count),
        (87, true, 40)
    );
    // The summoned elementals in the dead Pikemen's id, where they were made.
    assert_eq!(
        (stack(1).hex, stack(1).alive, stack(1).count),
        (18, true, 6)
    );
    assert_eq!(
        stack(1).creature,
        crate::anim::tests::created::FIRE_ELEMENTAL
    );
    assert_eq!((stack(2).hex, stack(2).alive), (100, true));
    assert_eq!((stack(3).hex, stack(3).count, now.stacks.len()), (30, 4, 4));
    let mana = |i: usize| now.sides[i].hero.map(|h| h.mana);
    assert_eq!((mana(0), mana(1)), (Some(7), Some(9)));
    assert!(now.siege.is_none());
    // Before the strike: the Pikemen alive, all 40 of them.
    let early = seen(&opening, &events[..2], &timeline, 0, cost);
    assert_eq!((early.active, early.stacks.len()), (Some(0), 3));
    // A siege's gate; a side without a hero; a timeline without the events' stacks.
    let mut siege = opening.clone();
    siege.siege = Some(h3_protocol::wire::Siege {
        faction: 0,
        fort: 1,
        moat: 0,
        gate_open: false,
        walls: Vec::new(),
        moat_hexes: Vec::new(),
        parts: Vec::new(),
    });
    siege.sides[1].hero = None;
    let gate = [
        Event::GateOpened {},
        Event::GateClosed {},
        Event::GateOpened {},
    ];
    let opened = seen(&siege, &gate, &crate::anim::Timeline::default(), 0, cost);
    assert!(opened.siege.is_some_and(|s| s.gate_open));
    assert_eq!(opened.sides[1].hero, None);
    let closed = seen(
        &siege,
        &gate[..2],
        &crate::anim::Timeline::default(),
        0,
        cost,
    );
    assert!(closed.siege.is_some_and(|s| !s.gate_open));
}

/// A query once the record is open (E17 M5 T6b1): its sequence number, its answer by that number.
#[test]
fn a_query_is_numbered_and_its_answer_kept() {
    let mut client = fetching();
    assert_eq!(client.query(Query::CreatureCastTargets {}), None);
    let text = record(
        &header(4, Seat::Attacker, 1),
        &[snapshot(1), your_turn(2)],
        2,
    );
    client.receive(text, verify).unwrap();
    let (seq, message) = client.query(Query::CastTargets { spell: 3 }).unwrap();
    let asked = ClientMessage::Query {
        seq,
        query: Query::CastTargets { spell: 3 },
    };
    assert_eq!(message, asked);
    assert_eq!(client.answer(seq), None);
    let answer = ServerMessage::QueryResult {
        n: 3,
        seq,
        targets: vec![Target::Stack { stack: 1 }],
    };
    client.receive(answer, verify).unwrap();
    assert_eq!(client.answer(seq), Some(&[Target::Stack { stack: 1 }][..]));
}
