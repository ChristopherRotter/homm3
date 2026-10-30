//! A seat played as the window plays it (E17 M5 T6b1), over loopback: `connect`'s host of one
//! game, the other seat played headless by `connect`'s session.

use std::time::Duration;

use h3_protocol::session::SeatKind;
use h3_protocol::wire::{Seat, ServerMessage};
use h3_view::remote::Content;

use super::*;
use crate::connect::tests::{host, player, repo, restartable, scratch, test_roots, whole};
use crate::connect::{self, Options, plain};

/// The window's seat at `address`, its pins in `dir`.
fn server(address: &str, seat: Seat, dir: &Path) -> Server {
    Server {
        address: address.to_owned(),
        game: Game::Join(crate::scenario::GAME),
        seat,
        token: Some(format!("{seat:?}").to_lowercase()),
        name: "window".to_owned(),
        pins: dir.join("pins.txt"),
        drop: None,
        account: None,
    }
}

/// Plays `online`'s seat by the plain player as the window does, a poll at a time, until the
/// game ends (at most a minute); every turn's targets asked first.
fn play(online: &mut Online) {
    // The test's clock: 2 ms a poll.
    let mut now = 0;
    while !online.over() {
        assert!(now < 60_000, "no end: {:?}", online.lines);
        online.poll(now).unwrap();
        if let Some((legal, _)) = online.turn() {
            // The active stack's own cast was asked: the server answered.
            assert!(online.targets(None).is_some());
            let width = online.record().unwrap().header.manifest.field.width;
            let command = plain(legal, online.seat(), width).unwrap();
            online.send(command).unwrap();
            assert_eq!(online.turn(), None);
        }
        std::thread::sleep(Duration::from_millis(2));
        now += 2;
    }
}

#[test]
fn the_window_s_seat_plays_and_re_joins_after_a_drop_keeping_its_record() {
    let (address, _) = host([player("attacker"), player("defender")], false);
    let (attacker, defender) = (scratch("online-a"), scratch("online-d"));
    let options = Options {
        address: address.clone(),
        game: Game::Join(crate::scenario::GAME),
        seat: Seat::Defender,
        token: Some("defender".to_owned()),
        name: "other".to_owned(),
        pins: defender.join("pins.txt"),
        packs: repo().join("packs"),
        out: None,
        drop: None,
        account: None,
    };
    let other = std::thread::Builder::new()
        .spawn(move || connect::session(&options, test_roots, &mut Vec::new()))
        .unwrap();
    let mut seat = server(&address, Seat::Attacker, &attacker);
    seat.drop = Some(1);
    let packs = repo().join("packs");
    let mut online = Online::open(seat, &packs, test_roots, Duration::from_secs(10)).unwrap();
    assert_eq!(online.content(), Content::Same);
    assert!(online.take_rules().is_some());
    assert_eq!(online.take_rules(), None);
    let first = online.record().unwrap().messages.first().cloned();
    assert!(matches!(first, Some(ServerMessage::Snapshot { .. })));
    play(&mut online);
    let d = other.join().unwrap().unwrap();
    // The drop after the first command, the re-join by the token: the record kept its start.
    assert_eq!(online.rejoined, 1);
    let record = online.record().unwrap();
    assert_eq!(record.messages.first().cloned(), first);
    assert_eq!(whole(record), whole(&d.record));
    assert!(
        online
            .lines
            .iter()
            .any(|l| l == "dropped the connection after 1 commands")
    );
    // Nothing more comes once the game ended.
    assert!(!online.poll(u64::MAX).unwrap());
}

#[test]
fn a_refusal_and_a_silent_server_end_the_opening() {
    let (address, _) = host(
        [
            player("attacker"),
            SeatKind::Bot(Box::new(crate::bot::Greedy)),
        ],
        false,
    );
    let dir = scratch("online-refused");
    let packs = repo().join("packs");
    let mut wrong = server(&address, Seat::Attacker, &dir);
    wrong.token = Some("wrong".to_owned());
    let refused = Online::open(wrong, &packs, test_roots, Duration::from_secs(10)).unwrap_err();
    assert!(refused.starts_with("refused"), "{refused}");
    let closed = server("ws://127.0.0.1:1", Seat::Attacker, &dir);
    assert!(Online::open(closed, &packs, test_roots, Duration::from_secs(10)).is_err());
}

#[test]
fn a_restarted_server_s_record_opens_afresh_and_the_seat_plays_on() {
    // E17 M5 T6b2: the host restarts its game after the seat's second command; the window's
    // seat dials again, re-joins by its token, and its record opens afresh at the re-join's
    // snapshot, mid-battle: a new timeline, played to the end.
    let greedy = || SeatKind::Bot(Box::new(crate::bot::Greedy));
    let (address, _, restart) = restartable([player("attacker"), greedy()], false);
    let dir = scratch("online-restart");
    let packs = repo().join("packs");
    let seat = server(&address, Seat::Attacker, &dir);
    let mut online = Online::open(seat, &packs, test_roots, Duration::from_secs(10)).unwrap();
    let opening = online.record().unwrap().messages[0].clone();
    let width = online.record().unwrap().header.manifest.field.width;
    let mut now = 0;
    let mut sent = 0;
    while sent < 2 {
        assert!(now < 60_000, "no turn: {:?}", online.lines);
        online.poll(now).unwrap();
        if let Some((legal, _)) = online.turn() {
            online
                .send(plain(legal, online.seat(), width).unwrap())
                .unwrap();
            sent += 1;
        }
        std::thread::sleep(Duration::from_millis(2));
        now += 2;
    }
    restart.restart([player("attacker"), greedy()]);
    while online.timelines() == 0 {
        assert!(now < 60_000, "no new timeline: {:?}", online.lines);
        online.poll(now).unwrap();
        std::thread::sleep(Duration::from_millis(2));
        now += 2;
    }
    let record = online.record().unwrap();
    assert_eq!(record.header.first, 1);
    let afresh = &record.messages[0];
    assert!(matches!(afresh, ServerMessage::Snapshot { n: 1, .. }));
    assert_ne!(afresh, &opening);
    assert!(online.lines.iter().any(|l| l.contains("opens afresh")));
    play(&mut online);
    assert_eq!(online.timelines(), 1);
    whole(online.record().unwrap());
}

/// With an account, the window's registration is asked until a login succeeds, never at a
/// re-dial's (E17 M5 review R1-2); the server's display name is kept without its control
/// characters (R1-9).
#[test]
fn the_window_registers_its_account_once_and_re_dials_without_it() {
    let seats = [
        player("attacker"),
        SeatKind::Bot(Box::new(crate::bot::Greedy)),
    ];
    let (address, _, restart) = restartable(seats, false);
    let dir = scratch("online-register");
    let accounts = dir.join("accounts.txt");
    let mut kept = remote::Accounts::default();
    kept.set("alice", [5; 32]);
    connect::write_private(&accounts, &kept.text()).unwrap();
    let mut seat = server(&address, Seat::Attacker, &dir);
    seat.drop = Some(1);
    seat.account = Some(Account {
        login: "alice".to_owned(),
        register: Some("Alice".to_owned()),
        accounts,
    });
    let packs = repo().join("packs");
    let mut online = Online::open(seat, &packs, test_roots, Duration::from_secs(10)).unwrap();
    play(&mut online);
    assert_eq!(online.rejoined, 1);
    assert_eq!(restart.logins(), [Some("Alice".to_owned()), None]);
    let logged: Vec<&String> = (online.lines.iter())
        .filter(|l| l.starts_with("logged in"))
        .collect();
    assert_eq!(logged, ["logged in as Alice", "logged in as Al?[2Jice"]);
}

/// Polls `online` from `now`, 2 ms a poll, until its seat's turn with its targets answered (at
/// most a minute); the clock then.
fn turn(online: &mut Online, mut now: u64) -> u64 {
    let end = now + 60_000;
    while online.turn().is_none() {
        assert!(now < end, "no turn: {:?}", online.lines);
        online.poll(now).unwrap();
        std::thread::sleep(Duration::from_millis(2));
        now += 2;
    }
    now
}

/// E17 M5 review R1-3 and R1-7: a connection silent for longer than `net::SILENT` by the
/// caller's clock counts as dropped, and the seat re-joins; the dials that fail while no server
/// answers count no re-join, the re-join after them one.
#[test]
fn a_silent_connection_is_dialed_again_and_only_re_joins_count() {
    let greedy = SeatKind::Bot(Box::new(crate::bot::Greedy));
    let (address, _) = host([player("attacker"), greedy], false);
    let dir = scratch("online-silent");
    let packs = repo().join("packs");
    let seat = server(&address, Seat::Attacker, &dir);
    let mut online = Online::open(seat, &packs, test_roots, Duration::from_secs(10)).unwrap();
    // The seat's turn: the server waits for it, silent.
    let mut now = turn(&mut online, 0);
    let heard = online.heard.unwrap();
    online.poll(heard + SILENT_MS).unwrap();
    assert!(!online.down, "{:?}", online.lines);
    now = now.max(heard + SILENT_MS + 1);
    online.poll(now).unwrap();
    let silent = "the connection dropped (nothing heard in 60s): re-joining";
    assert!(
        online.lines.iter().any(|l| l == silent),
        "{:?}",
        online.lines
    );
    now = turn(&mut online, now);
    assert_eq!((online.rejoined, online.timelines()), (1, 0));
    // No server at the address for three dials, a second apart.
    online.server.address = "ws://127.0.0.1:1".to_owned();
    online.connection = None;
    for _ in 0..3 {
        now += REDIAL_MS;
        online.poll(now).unwrap();
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(online.rejoined, 1);
    online.server.address = address;
    now += REDIAL_MS;
    turn(&mut online, now);
    assert_eq!((online.rejoined, online.timelines()), (2, 0));
}

/// E17 M5 review R1-5: a poll takes at most its bound of frames, the rest at the next; each
/// growth appends the connection's new message to the record taken, which stays the
/// connection's.
#[test]
fn a_poll_takes_at_most_its_bound_and_a_growth_appends() {
    let greedy = SeatKind::Bot(Box::new(crate::bot::Greedy));
    let (address, _) = host([player("attacker"), greedy], false);
    let dir = scratch("online-bound");
    let packs = repo().join("packs");
    let seat = server(&address, Seat::Attacker, &dir);
    let mut online = Online::open(seat, &packs, test_roots, Duration::from_secs(10)).unwrap();
    let mut now = 0;
    while online.turn().is_none() {
        assert!(now < 60_000, "no turn: {:?}", online.lines);
        // Time for the server's answers to queue up.
        std::thread::sleep(Duration::from_millis(20));
        let before = online.taken();
        online.poll_at_most(now, 1).unwrap();
        assert!(online.taken() <= before + 1);
        now += 20;
    }
    assert_eq!(online.record(), online.client.record());
    assert_eq!(online.copied, online.taken());
}
