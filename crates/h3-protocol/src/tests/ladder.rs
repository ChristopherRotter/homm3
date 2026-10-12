//! The ladders' queues (E16 M4 T2): who may wait, the matcher's window, a match's answers, and,
//! under random sequences of requests, that no account is matched twice, with itself or outside
//! the window. The bots (E16 M4 T5): a player who waited long enough plays the nearest.

use std::collections::BTreeMap;
use std::format;
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use proptest::prelude::*;

use crate::ladder::{Ladder, LadderBot, Matched, Queues, Told, WIDEN_EVERY_MS, WINDOW, window};
use crate::wire::{LadderInfo, Refusal, ServerMessage, UnqueueReason};

/// The ladder `id`, 10 s to accept, without bots.
fn ladder(id: &str, name: &str) -> Ladder {
    Ladder {
        id: id.to_string(),
        name: name.to_string(),
        season: 1,
        accept_ms: 10_000,
        bots: vec![],
        bot_after_ms: 60_000,
    }
}

/// Two ladders: `arena` and `pit`.
fn queues() -> Queues<u8> {
    Queues::new(vec![ladder("arena", "The Arena"), ladder("pit", "The Pit")])
}

fn queued(ladder: &str) -> ServerMessage {
    ServerMessage::Queued {
        seq: 1,
        ladder: ladder.to_string(),
    }
}

fn refused(reason: Refusal) -> ServerMessage {
    ServerMessage::Refused { reason }
}

/// Connection `conn` as `login`, rated `rating`, waits in `ladder` at `now`.
fn wait(q: &mut Queues<u8>, conn: u8, (login, rating): (&str, i32), ladder: &str, now: u64) {
    let answer = q.queue(conn, (1, ladder), Some(login), (rating, false), now);
    assert_eq!(answer, queued(ladder));
}

/// How many wait in each ladder.
fn waiting(q: &Queues<u8>) -> Vec<u32> {
    let ServerMessage::Ladders { ladders, .. } = q.list(5) else {
        panic!("not a list of ladders")
    };
    ladders.iter().map(|l| l.waiting).collect()
}

fn found(ladder: &str) -> ServerMessage {
    ServerMessage::MatchFound {
        ladder: ladder.to_string(),
        accept_ms: 10_000,
    }
}

#[test]
fn an_account_waits_in_one_queue_once_logged_in() {
    let mut q = queues();
    assert_eq!(
        q.list(5),
        ServerMessage::Ladders {
            seq: 5,
            ladders: vec![
                LadderInfo {
                    id: "arena".to_string(),
                    name: "The Arena".to_string(),
                    season: 1,
                    waiting: 0,
                    bots: vec![],
                },
                LadderInfo {
                    id: "pit".to_string(),
                    name: "The Pit".to_string(),
                    season: 1,
                    waiting: 0,
                    bots: vec![],
                },
            ],
        }
    );
    let ask = |q: &mut Queues<u8>, conn, login, ladder, unfinished| {
        q.queue(conn, (1, ladder), login, (1500, unfinished), 0)
    };
    assert_eq!(
        ask(&mut q, 1, None, "arena", false),
        refused(Refusal::NotLoggedIn)
    );
    assert_eq!(
        ask(&mut q, 1, Some("ann"), "ring", false),
        refused(Refusal::NoSuchLadder)
    );
    assert_eq!(
        ask(&mut q, 1, Some("ann"), "arena", true),
        refused(Refusal::InRankedGame)
    );
    assert_eq!(ask(&mut q, 1, Some("ann"), "arena", false), queued("arena"));
    assert!(q.waits("ann") && !q.waits("bob"));
    assert!(q.holds(1) && !q.holds(2));
    // One queue at a time, from any connection.
    for (conn, ladder) in [(1, "arena"), (2, "pit")] {
        assert_eq!(
            ask(&mut q, conn, Some("ann"), ladder, false),
            refused(Refusal::AlreadyQueued)
        );
    }
    // Another ladder's ranked game does not keep it from this one.
    assert_eq!(ask(&mut q, 2, Some("bob"), "pit", false), queued("pit"));
    assert_eq!(waiting(&q), [1, 1]);
    // It leaves when it asks, or with its connection; not twice.
    let mut told = Told::new();
    assert_eq!(
        q.dequeue(1, 7, &mut told),
        ServerMessage::Dequeued { seq: 7 }
    );
    assert_eq!(q.dequeue(1, 8, &mut told), refused(Refusal::NotQueued));
    q.closed(2, &mut told);
    q.closed(3, &mut told);
    assert_eq!((waiting(&q), told), (vec![0, 0], vec![]));
}

#[test]
fn the_window_widens_with_the_shorter_wait() {
    assert_eq!(window(0), WINDOW);
    assert_eq!(window(WIDEN_EVERY_MS - 1), 100);
    assert_eq!(window(WIDEN_EVERY_MS), 150);
    assert_eq!(window(5 * WIDEN_EVERY_MS + 7), 350);
    assert_eq!(window(u64::MAX), u32::MAX);
    let mut q = queues();
    wait(&mut q, 1, ("ann", 1500), "arena", 0);
    wait(&mut q, 2, ("bob", 1700), "arena", 0);
    wait(&mut q, 3, ("cid", 1560), "arena", 0);
    wait(&mut q, 4, ("dee", 1900), "arena", 0);
    // Neighbours by rating within 100: Ann and Cid; Bob and Dee are 200 apart.
    let mut told = Told::new();
    q.tick(0, &mut told);
    assert_eq!(told, [(1, found("arena")), (3, found("arena"))]);
    assert_eq!(waiting(&q), [4, 0]);
    assert_eq!(
        (q.accept(1), q.accept(3).map(|m| m.is_some())),
        (Ok(None), Ok(true))
    );
    // 200 needs 60 s of the shorter wait: Eve, new at 50 s and far from both, waits on her own.
    wait(&mut q, 5, ("eve", 1300), "arena", 50_000);
    told.clear();
    q.tick(59_999, &mut told);
    assert_eq!(told, []);
    q.tick(60_000, &mut told);
    assert_eq!(told, [(2, found("arena")), (4, found("arena"))]);
    // Bob (1700) and Eve (1300) would be 400 apart; Eve's wait is the shorter one.
    assert_eq!(waiting(&q), [3, 0]);
}

#[test]
fn both_accept_and_the_host_creates_the_game() {
    let mut q = queues();
    wait(&mut q, 1, ("ann", 1500), "pit", 0);
    wait(&mut q, 2, ("bob", 1520), "pit", 0);
    let mut told = Told::new();
    q.tick(1, &mut told);
    assert_eq!(q.accept(3), Err(Refusal::NoMatch));
    assert_eq!(q.accept(2), Ok(None));
    assert_eq!(q.accept(2), Ok(None), "accepted twice: still waiting");
    assert!(q.holds(1) && q.holds(2), "matched, they still wait");
    let matched = Matched {
        ladder: "pit".to_string(),
        players: [(Some(1), "ann".to_string()), (Some(2), "bob".to_string())],
    };
    assert_eq!(q.accept(1), Ok(Some(matched)));
    assert!(!q.waits("ann") && !q.waits("bob"));
    assert!(!q.holds(1) && !q.holds(2));
    assert_eq!(waiting(&q), [0, 0]);
    assert_eq!(q.accept(1), Err(Refusal::NoMatch));
}

#[test]
fn a_decline_an_expiry_or_a_departure_puts_the_other_back() {
    let mut q = queues();
    let mut told = Told::new();
    // Ann and Bob are matched; Cid comes after them.
    wait(&mut q, 1, ("ann", 1500), "arena", 0);
    wait(&mut q, 2, ("bob", 1500), "arena", 0);
    q.tick(0, &mut told);
    wait(&mut q, 3, ("cid", 1500), "arena", 5);
    assert_eq!(q.decline(3, 4, &mut told), refused(Refusal::NoMatch));
    told.clear();
    // Bob declines: he leaves, Ann waits again, ahead of Cid; she and Cid are matched.
    assert_eq!(
        q.decline(2, 4, &mut told),
        ServerMessage::Dequeued { seq: 4 }
    );
    assert_eq!(told, [(1, ServerMessage::MatchCancelled {})]);
    assert!(!q.waits("bob") && q.waits("ann"));
    told.clear();
    q.tick(10, &mut told);
    assert_eq!(told, [(1, found("arena")), (3, found("arena"))]);
    // Cid leaves the match with `Dequeue`: the same.
    told.clear();
    assert_eq!(
        q.dequeue(3, 5, &mut told),
        ServerMessage::Dequeued { seq: 5 }
    );
    assert_eq!(told, [(1, ServerMessage::MatchCancelled {})]);
    // Dee is matched with Ann and goes away: Ann waits again.
    wait(&mut q, 4, ("dee", 1500), "arena", 20);
    told.clear();
    q.tick(20, &mut told);
    assert_eq!(told.len(), 2);
    told.clear();
    q.closed(4, &mut told);
    assert_eq!(told, [(1, ServerMessage::MatchCancelled {})]);
    // Ann accepts a match Eve never answers: at the due time Eve is out, Ann waits again.
    wait(&mut q, 5, ("eve", 1500), "arena", 30);
    q.tick(30, &mut told);
    assert_eq!(q.accept(1), Ok(None));
    told.clear();
    q.tick(10_029, &mut told);
    assert_eq!(told, []);
    q.tick(10_030, &mut told);
    let expired = ServerMessage::Unqueued {
        reason: UnqueueReason::Expired,
    };
    assert_eq!(
        told,
        [(1, ServerMessage::MatchCancelled {}), (5, expired.clone())]
    );
    assert!(q.waits("ann") && !q.waits("eve"));
    // A match neither answers: both are out.
    wait(&mut q, 6, ("fay", 1500), "arena", 10_030);
    told.clear();
    q.tick(10_030, &mut told);
    q.tick(20_030, &mut told);
    assert_eq!(told[2..], [(1, expired.clone()), (6, expired)], "{told:?}");
    assert_eq!(waiting(&q), [0, 0]);
}

#[test]
fn a_drain_empties_the_queues() {
    let mut q = queues();
    let mut told = Told::new();
    wait(&mut q, 1, ("ann", 1500), "arena", 0);
    wait(&mut q, 2, ("bob", 1500), "arena", 0);
    q.tick(0, &mut told);
    wait(&mut q, 3, ("cid", 1500), "pit", 0);
    told.clear();
    q.drain(&mut told);
    let draining = ServerMessage::Unqueued {
        reason: UnqueueReason::Draining,
    };
    assert_eq!(
        told,
        [(3, draining.clone()), (1, draining.clone()), (2, draining)]
    );
    assert_eq!(
        q.queue(1, (2, "arena"), Some("ann"), (1500, false), 1),
        refused(Refusal::Draining)
    );
    assert_eq!(waiting(&q), [0, 0]);
}

/// The ladder `arena` with the bots Randy (1500) and Stan (1700), played after a minute's wait
/// (E16 M4 T5), and `pit` without bots.
fn with_bots() -> Queues<u8> {
    let bot = |login: &str, display: &str, rating| LadderBot {
        login: login.to_string(),
        display: display.to_string(),
        rating,
    };
    let arena = Ladder {
        bots: vec![bot("randy", "Randy", 1500), bot("stan", "Stan", 1700)],
        ..ladder("arena", "The Arena")
    };
    Queues::new(vec![arena, ladder("pit", "The Pit")])
}

#[test]
fn a_player_who_waited_long_enough_plays_the_nearest_bot() {
    let mut q = with_bots();
    let ServerMessage::Ladders { ladders, .. } = q.list(5) else {
        panic!("not a list of ladders")
    };
    assert_eq!(
        ladders.iter().map(|l| l.bots.clone()).collect::<Vec<_>>(),
        [vec!["Randy".to_string(), "Stan".to_string()], vec![]]
    );
    // Ann (1600) waits alone, as does Gus on the ladder without bots.
    wait(&mut q, 1, ("ann", 1600), "arena", 0);
    wait(&mut q, 7, ("gus", 1600), "pit", 0);
    let mut told = Told::new();
    q.tick(59_999, &mut told);
    assert_eq!(told, []);
    // After a minute she plays the nearest bot: Randy and Stan are as near, Randy comes first.
    q.tick(60_000, &mut told);
    assert_eq!(told, [(1, found("arena"))]);
    assert_eq!(waiting(&q), [1, 1], "a bot is no player waiting");
    // The bot accepted at once: her yes makes the match.
    let matched = Matched {
        ladder: "arena".to_string(),
        players: [(Some(1), "ann".to_string()), (None, "randy".to_string())],
    };
    assert_eq!(q.accept(1), Ok(Some(matched)));
    assert_eq!(waiting(&q), [0, 1]);
    // Stan's rating changes on the arena (not on the pit's ladder): Bob (1600) plays him.
    q.rate("arena", "stan", 1650);
    q.rate("pit", "randy", 1600);
    wait(&mut q, 2, ("bob", 1600), "arena", 60_000);
    told.clear();
    q.tick(120_000, &mut told);
    assert_eq!(told, [(2, found("arena"))]);
    // Bob declines: he leaves, the bot is told nothing.
    told.clear();
    assert_eq!(
        q.decline(2, 3, &mut told),
        ServerMessage::Dequeued { seq: 3 }
    );
    assert_eq!((told.clone(), q.waits("stan")), (vec![], false));
    // Cid does not answer his bot: at the due time he is out; Dee's bot match ends in a drain.
    wait(&mut q, 3, ("cid", 1500), "arena", 120_000);
    q.tick(180_000, &mut told);
    assert_eq!(told, [(3, found("arena"))]);
    told.clear();
    q.tick(190_000, &mut told);
    let expired = ServerMessage::Unqueued {
        reason: UnqueueReason::Expired,
    };
    assert_eq!(told, [(3, expired)]);
    // People first: Eve and Fay, long waiting, play each other, not a bot.
    wait(&mut q, 5, ("eve", 1500), "arena", 190_000);
    wait(&mut q, 6, ("fay", 1500), "arena", 190_000);
    told.clear();
    q.tick(300_000, &mut told);
    assert_eq!(told, [(5, found("arena")), (6, found("arena"))]);
    // Their match expires; Dee, after her minute, plays a bot; a drain tells her and Gus only.
    wait(&mut q, 4, ("dee", 1500), "arena", 300_000);
    told.clear();
    q.tick(360_000, &mut told);
    assert_eq!(told[2..], [(4, found("arena"))], "{told:?}");
    told.clear();
    q.drain(&mut told);
    let draining = ServerMessage::Unqueued {
        reason: UnqueueReason::Draining,
    };
    assert_eq!(told, [(7, draining.clone()), (4, draining)]);
}

/// A request, as the property test makes them.
#[derive(Debug, Clone)]
enum Op {
    Queue { conn: u8, ladder: bool, rating: i32 },
    Dequeue(u8),
    Accept(u8),
    Decline(u8),
    Closed(u8),
    Tick(u64),
}

fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0..6u8, any::<bool>(), 1300..1800i32)
            .prop_map(|(conn, ladder, rating)| Op::Queue { conn, ladder, rating }),
        1 => (0..6u8).prop_map(Op::Dequeue),
        3 => (0..6u8).prop_map(Op::Accept),
        1 => (0..6u8).prop_map(Op::Decline),
        1 => (0..6u8).prop_map(Op::Closed),
        3 => (0..40_000u64).prop_map(Op::Tick),
    ]
}

/// Where a connection is, as the messages it heard say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Where {
    Waiting { rating: i32, since: u64 },
    Matched { rating: i32, since: u64 },
}

/// The connections' accounts: six connections, four accounts.
fn login(conn: u8) -> String {
    format!("p{}", conn % 4)
}

/// Follows what the queues told, in order: a connection back in the queue keeps its waiting time;
/// a pair found (two `MatchFound` in a row) is two waiting connections of distinct accounts within
/// the window.
fn follow(model: &mut BTreeMap<u8, Where>, told: &Told<u8>, now: u64) -> Result<(), TestCaseError> {
    let mut first = None;
    for &(conn, ref message) in told {
        match message {
            ServerMessage::MatchFound { .. } => {
                let Some(a) = first.take() else {
                    first = Some(conn);
                    continue;
                };
                let (
                    Some(&Where::Waiting {
                        rating: ra,
                        since: sa,
                    }),
                    Some(&Where::Waiting {
                        rating: rb,
                        since: sb,
                    }),
                ) = (model.get(&a), model.get(&conn))
                else {
                    return Err(TestCaseError::fail(format!(
                        "{a} or {conn} matched while not waiting"
                    )));
                };
                prop_assert_ne!(login(a), login(conn), "an account against itself");
                let waited = now.saturating_sub(sa.max(sb));
                prop_assert!(
                    ra.abs_diff(rb) <= window(waited),
                    "{ra} and {rb} after {waited} ms"
                );
                model.insert(
                    a,
                    Where::Matched {
                        rating: ra,
                        since: sa,
                    },
                );
                model.insert(
                    conn,
                    Where::Matched {
                        rating: rb,
                        since: sb,
                    },
                );
            }
            ServerMessage::MatchCancelled {} => {
                let Some(&Where::Matched { rating, since }) = model.get(&conn) else {
                    return Err(TestCaseError::fail(format!(
                        "{conn} cancelled while not matched"
                    )));
                };
                model.insert(conn, Where::Waiting { rating, since });
            }
            ServerMessage::Unqueued { .. } => {
                prop_assert!(model.remove(&conn).is_some(), "{conn} was not waiting");
            }
            _ => {}
        }
    }
    prop_assert!(first.is_none(), "a match of one");
    Ok(())
}

proptest! {
    #[test]
    fn no_account_is_matched_twice_with_itself_or_outside_the_window(
        ops in proptest::collection::vec(arb_op(), 1..120)
    ) {
        let mut q = queues();
        let mut model = BTreeMap::new();
        let mut now = 0;
        for op in ops {
            let mut told = Told::new();
            match op {
                Op::Queue { conn, ladder, rating } => {
                    let ladder = if ladder { "arena" } else { "pit" };
                    let login = login(conn);
                    let answer = q.queue(conn, (1, ladder), Some(&login), (rating, false), now);
                    if answer == queued(ladder) {
                        prop_assert!(!model.contains_key(&conn));
                        model.insert(conn, Where::Waiting { rating, since: now });
                    }
                }
                Op::Dequeue(conn) | Op::Decline(conn) | Op::Closed(conn) => {
                    let answer = match op {
                        Op::Dequeue(_) => q.dequeue(conn, 1, &mut told),
                        Op::Decline(_) => q.decline(conn, 1, &mut told),
                        _ => {
                            q.closed(conn, &mut told);
                            ServerMessage::Dequeued { seq: 1 }
                        }
                    };
                    if answer == (ServerMessage::Dequeued { seq: 1 }) {
                        model.remove(&conn);
                    }
                }
                Op::Accept(conn) => {
                    if let Ok(Some(Matched { players: [(a, _), (b, _)], .. })) = q.accept(conn) {
                        for player in [a, b].into_iter().flatten() {
                            let matched = model.remove(&player);
                            prop_assert!(
                                matches!(matched, Some(Where::Matched { .. })),
                                "{player} was not matched"
                            );
                        }
                    }
                }
                Op::Tick(dt) => {
                    now += dt;
                    q.tick(now, &mut told);
                }
            }
            follow(&mut model, &told, now)?;
            // The queues and the model agree on who waits, and no account waits twice.
            let ServerMessage::Ladders { ladders, .. } = q.list(0) else { panic!("not a list of ladders") };
            let total: u32 = ladders.iter().map(|l| l.waiting).sum();
            prop_assert_eq!(total as usize, model.len());
            let mut logins: Vec<String> = model.keys().map(|&conn| login(conn)).collect();
            logins.sort();
            let all = logins.len();
            logins.dedup();
            prop_assert_eq!(logins.len(), all, "an account waits twice");
            for conn in 0..6 {
                prop_assert_eq!(q.waits(&login(conn)), logins.contains(&login(conn)));
            }
        }
    }
}
