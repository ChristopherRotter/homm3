//! Accounts over loopback (E16 M3 T2a): a client registers with the key its login name and
//! password derive, logs in again after a restart; logins signed for another server, with another
//! connection's challenge or by another key are refused; anonymous play as configured; failed
//! logins close an address's connections beyond their limit.

use h3_protocol::account;

use super::*;
use crate::client::Key;

/// The derivation's test vector, computed independently (Python: `argon2-cffi`'s
/// `hash_secret_raw` with the parameters of `Key::derive`, then PyNaCl's `SigningKey`).
const ALICE_SEED: &str = "a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69";
const ALICE_PUBLIC: &str = "b17dbb2796b53c1c636e9a5981d6424ac5ee096b89b47223567831f435a8af14";

/// A key of a fixed seed (the derivation costs 64 MiB and a moment: once is enough).
fn key(n: u8) -> Key {
    Key::from_seed([n; 32])
}

/// The answer to a request about an account: its display name and listing preference.
fn account(seq: u32, display: &str) -> Option<ServerMessage> {
    Some(ServerMessage::Account {
        seq,
        display: display.to_owned(),
        listed: true,
    })
}

fn refusal(reason: Refusal) -> Option<ServerMessage> {
    Some(ServerMessage::Refused { reason })
}

/// The login a client sends, signed by `by` over `id` and `challenge`.
fn login_signed(
    id: &str,
    challenge: &[u8; account::CHALLENGE_BYTES],
    login: &str,
    key: &Key,
    by: &Key,
) -> ClientMessage {
    let signed = account::login_message(id, challenge, login, None);
    ClientMessage::Login {
        seq: 5,
        login: login.to_owned(),
        key: account::to_hex(&key.public()),
        signature: account::to_hex(&by.sign(&signed)),
        display: None,
    }
}

#[test]
fn the_key_derivation_matches_its_test_vector() {
    let derived = Key::derive("alice", "correct horse battery staple").unwrap();
    assert_eq!(account::to_hex(&derived.public()), ALICE_PUBLIC);
    let seed = account::from_hex(ALICE_SEED).unwrap();
    assert_eq!(Key::from_seed(seed).public(), derived.public());
    let other = Key::derive("bob", "correct horse battery staple").unwrap();
    assert_ne!(other.public(), derived.public(), "salted by the login name");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_account_registers_logs_in_and_survives_a_restart() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let alice = Key::derive("alice", "correct horse battery staple").unwrap();
    let mut client = greeted(&running).await;
    // An unknown login name without a display name does not register.
    let unknown = client.login(1, "alice", &alice, None).await.unwrap();
    assert_eq!(unknown, refusal(Refusal::BadLogin));
    let registered = client.login(2, "alice", &alice, Some("Alice")).await;
    assert_eq!(registered.unwrap(), account(2, "Alice"));
    let again = client.login(3, "alice", &alice, None).await.unwrap();
    assert_eq!(again, refusal(Refusal::AlreadyLoggedIn));
    // Another account may not take the display name, in any case; a free one registers.
    let mut other = greeted(&running).await;
    let taken = other.login(1, "bob", &key(2), Some("ALICE")).await.unwrap();
    assert_eq!(taken, refusal(Refusal::NameTaken));
    let bob = other.login(2, "bob", &key(2), Some("Bob")).await.unwrap();
    assert_eq!(bob, account(2, "Bob"));
    // A new connection logs in with the key alone; a display name it names is ignored.
    let mut later = greeted(&running).await;
    let logged = later
        .login(4, "alice", &alice, Some("Other"))
        .await
        .unwrap();
    assert_eq!(logged, account(4, "Alice"));
    for c in [client, other, later] {
        c.close().await;
    }
    running.shutdown().await;
    // The player log keeps the accounts across a restart.
    let again = start(&config).await.unwrap();
    let mut client = greeted(&again).await;
    let restored = client.login(1, "alice", &alice, None).await.unwrap();
    assert_eq!(restored, account(1, "Alice"));
    let log = std::fs::read_to_string(config.server.data.join("players.log")).unwrap();
    assert_eq!(log.lines().count(), 2, "{log}");
    assert!(log.contains(ALICE_PUBLIC), "{log}");
    client.close().await;
    again.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn logins_signed_for_another_server_connection_or_key_are_refused() {
    let mut config = config(fast());
    config.server.public_url = Some(String::from("wss://h3.example"));
    let running = start(&config).await.unwrap();
    let alice = key(1);
    let mut first = greeted(&running).await;
    let (id, challenge) = first.welcome().map(|(i, c)| (i.to_owned(), *c)).unwrap();
    assert_eq!(id, "wss://h3.example", "the public URL is the server's id");
    let mut client = greeted(&running).await;
    let registered = client.login(1, "alice", &alice, Some("Alice")).await;
    assert_eq!(registered.unwrap(), account(1, "Alice"));
    client.close().await;
    let mut client = greeted(&running).await;
    let mine = *client.welcome().unwrap().1;
    assert_ne!(mine, challenge, "a challenge per connection");
    let refused = [
        // Signed for another server.
        login_signed("wss://other.example", &mine, "alice", &alice, &alice),
        // Signed with another connection's challenge.
        login_signed(&id, &challenge, "alice", &alice, &alice),
        // The account's key, signed by another.
        login_signed(&id, &mine, "alice", &alice, &key(2)),
        // Another key, its own signature: the login name is the account's.
        login_signed(&id, &mine, "alice", &key(2), &key(2)),
    ];
    for login in &refused {
        client.send(login).await.unwrap();
        assert_eq!(client.recv().await, refusal(Refusal::BadLogin), "{login:?}");
    }
    // Refused logins changed nothing: the right one holds, and the first connection's too.
    let right = login_signed(&id, &mine, "alice", &alice, &alice);
    client.send(&right).await.unwrap();
    assert_eq!(client.recv().await, account(5, "Alice"));
    let theirs = first.login(6, "alice", &alice, None).await.unwrap();
    assert_eq!(theirs, account(6, "Alice"));
    for c in [first, client] {
        c.close().await;
    }
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn without_anonymous_play_a_game_waits_for_the_login() {
    let mut config = config(fast());
    config.server.anonymous = false;
    let running = start(&config).await.unwrap();
    let mut client = greeted(&running).await;
    client.create("pvp", Seat::Attacker).await.unwrap();
    assert_eq!(client.recv().await, refusal(Refusal::NotLoggedIn));
    client.join(1, Seat::Attacker, None).await.unwrap();
    assert_eq!(client.recv().await, refusal(Refusal::NotLoggedIn));
    // The lobby's lists need no login.
    assert!(listed(&running).await.is_empty());
    let logged = client.login(1, "alice", &key(1), Some("Alice")).await;
    assert_eq!(logged.unwrap(), account(1, "Alice"));
    client.create("pvp", Seat::Attacker).await.unwrap();
    let created = client.recv().await;
    assert!(
        matches!(created, Some(ServerMessage::GameCreated { .. })),
        "{created:?}"
    );
    client.close().await;
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_logins_close_an_addresss_connections_beyond_their_limit() {
    let config = config(Limits {
        failed_logins_per_address: 2,
        ..fast()
    });
    let running = start(&config).await.unwrap();
    let mut client = greeted(&running).await;
    let registered = client.login(1, "alice", &key(1), Some("Alice")).await;
    assert_eq!(registered.unwrap(), account(1, "Alice"));
    // Requests of the wrong form never reach the check and do not count.
    let mut probe = greeted(&running).await;
    for _ in 0..3 {
        let bad = probe.login(1, "No Such Name", &key(1), None).await.unwrap();
        assert_eq!(bad, refusal(Refusal::BadName));
    }
    for seq in 2..4 {
        let wrong = probe.login(seq, "alice", &key(2), None).await.unwrap();
        assert_eq!(wrong, refusal(Refusal::BadLogin));
    }
    // The limit is reached: the next login closes the connection, even the right one, and on
    // another connection of the address too; the address's other requests still pass.
    let right = probe.login(4, "alice", &key(1), None).await.unwrap();
    assert_eq!(
        right,
        Some(ServerMessage::Goodbye {
            reason: GoodbyeReason::RateLimited
        })
    );
    assert_eq!(probe.recv().await, None);
    let mut next = greeted(&running).await;
    assert!(listed(&running).await.is_empty());
    let barred = next.login(1, "alice", &key(1), None).await.unwrap();
    assert_eq!(
        barred,
        Some(ServerMessage::Goodbye {
            reason: GoodbyeReason::RateLimited
        })
    );
    client.close().await;
    running.shutdown().await;
}

/// The answer to a request about an account that opted out of the listing.
fn unlisted(seq: u32, display: &str) -> Option<ServerMessage> {
    Some(ServerMessage::Account {
        seq,
        display: display.to_owned(),
        listed: false,
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rekeyed_account_logs_in_with_its_new_key_only_also_after_a_restart() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (old, new) = (key(1), key(2));
    let mut client = greeted(&running).await;
    // Before a login there is no account to re-key.
    let early = client.rekey(1, "alice", &old, &new).await.unwrap();
    assert_eq!(early, refusal(Refusal::NotLoggedIn));
    let registered = client.login(2, "alice", &old, Some("Alice")).await;
    assert_eq!(registered.unwrap(), account(2, "Alice"));
    // Signed by a key that is not the account's, or for another connection: refused.
    let wrong = client.rekey(3, "alice", &key(3), &new).await.unwrap();
    assert_eq!(wrong, refusal(Refusal::BadLogin));
    let (id, _) = client.welcome().map(|(i, c)| (i.to_owned(), *c)).unwrap();
    let signed =
        account::rekey_message(&id, &[9; account::CHALLENGE_BYTES], "alice", &new.public());
    let elsewhere = ClientMessage::Rekey {
        seq: 4,
        key: account::to_hex(&new.public()),
        by_old: account::to_hex(&old.sign(&signed)),
        by_new: account::to_hex(&new.sign(&signed)),
    };
    client.send(&elsewhere).await.unwrap();
    assert_eq!(client.recv().await, refusal(Refusal::BadLogin));
    let rekeyed = client.rekey(5, "alice", &old, &new).await.unwrap();
    assert_eq!(rekeyed, account(5, "Alice"));
    // The same key again changes nothing and writes no line.
    let again = client.rekey(6, "alice", &new, &new).await.unwrap();
    assert_eq!(again, account(6, "Alice"));
    let mut other = greeted(&running).await;
    let by_old = other.login(1, "alice", &old, None).await.unwrap();
    assert_eq!(by_old, refusal(Refusal::BadLogin));
    let by_new = other.login(2, "alice", &new, None).await.unwrap();
    assert_eq!(by_new, account(2, "Alice"));
    for c in [client, other] {
        c.close().await;
    }
    running.shutdown().await;
    let log = std::fs::read_to_string(config.server.data.join("players.log")).unwrap();
    assert_eq!(log.lines().count(), 2, "{log}");
    let again = start(&config).await.unwrap();
    let mut client = greeted(&again).await;
    let by_old = client.login(1, "alice", &old, None).await.unwrap();
    assert_eq!(by_old, refusal(Refusal::BadLogin));
    let by_new = client.login(2, "alice", &new, None).await.unwrap();
    assert_eq!(by_new, account(2, "Alice"));
    client.close().await;
    again.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn display_names_stay_unique_and_the_preference_is_kept() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let mut alice = greeted(&running).await;
    let early = alice.rename(1, "Alice").await.unwrap();
    assert_eq!(early, refusal(Refusal::NotLoggedIn));
    let early = alice.set_listed(1, false).await.unwrap();
    assert_eq!(early, refusal(Refusal::NotLoggedIn));
    assert_eq!(
        alice
            .login(2, "alice", &key(1), Some("Alice"))
            .await
            .unwrap(),
        account(2, "Alice")
    );
    let mut bob = greeted(&running).await;
    assert_eq!(
        bob.login(1, "bob", &key(2), Some("Bob")).await.unwrap(),
        account(1, "Bob")
    );
    // Another account's name, in any case, is taken; one's own in another case is not.
    assert_eq!(
        bob.rename(2, "ALICE").await.unwrap(),
        refusal(Refusal::NameTaken)
    );
    assert_eq!(alice.rename(3, "aLiCe").await.unwrap(), account(3, "aLiCe"));
    assert_eq!(
        alice.rename(4, "Liddell").await.unwrap(),
        account(4, "Liddell")
    );
    // The old name is free again; a name that breaks the rules never reaches the server's check.
    assert_eq!(bob.rename(3, "Alice").await.unwrap(), account(3, "Alice"));
    assert_eq!(
        bob.rename(4, " x").await.unwrap(),
        refusal(Refusal::BadName)
    );
    assert_eq!(
        alice.set_listed(5, false).await.unwrap(),
        unlisted(5, "Liddell")
    );
    assert_eq!(
        alice.set_listed(6, false).await.unwrap(),
        unlisted(6, "Liddell")
    );
    for c in [alice, bob] {
        c.close().await;
    }
    running.shutdown().await;
    let again = start(&config).await.unwrap();
    let mut alice = greeted(&again).await;
    let back = alice.login(1, "alice", &key(1), None).await.unwrap();
    assert_eq!(back, unlisted(1, "Liddell"));
    let mut carol = greeted(&again).await;
    let taken = carol
        .login(1, "carol", &key(3), Some("alice"))
        .await
        .unwrap();
    assert_eq!(taken, refusal(Refusal::NameTaken), "bob holds it now");
    for c in [alice, carol] {
        c.close().await;
    }
    again.shutdown().await;
    let log = std::fs::read_to_string(config.server.data.join("players.log")).unwrap();
    assert_eq!(
        log.lines().count(),
        6,
        "two registrations, three renames, one preference: {log}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_operator_rebinds_an_account_while_the_server_is_stopped() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let mut client = greeted(&running).await;
    let registered = client.login(1, "alice", &key(1), Some("Alice")).await;
    assert_eq!(registered.unwrap(), account(1, "Alice"));
    client.close().await;
    let new = account::to_hex(&key(2).public());
    // One server or one operator's command at a time on a data directory.
    let busy = crate::rebind(&config, "alice", &new).unwrap_err();
    assert!(busy.0.contains("in use"), "{busy}");
    let second = start(&config).await.unwrap_err();
    assert!(second.0.contains("in use"), "{second}");
    running.shutdown().await;
    let refused = [
        ("bob", new.as_str(), "no account"),
        ("alice", "00", "not a public key"),
        // The identity point: a key no signature holds for.
        (
            "alice",
            &format!("01{}", "00".repeat(31)),
            "not a public key",
        ),
    ];
    for (login, key, why) in refused {
        let error = crate::rebind(&config, login, key).unwrap_err();
        assert!(error.0.contains(why), "{login} {key}: {error}");
    }
    assert_eq!(crate::rebind(&config, "alice", &new).unwrap(), "Alice");
    let log = std::fs::read_to_string(config.server.data.join("players.log")).unwrap();
    assert!(log.ends_with(&format!(
        "{{\"Rebound\":{{\"login\":\"alice\",\"key\":\"{new}\"}}}}\n"
    )));
    let again = start(&config).await.unwrap();
    let mut client = greeted(&again).await;
    let by_old = client.login(1, "alice", &key(1), None).await.unwrap();
    assert_eq!(by_old, refusal(Refusal::BadLogin));
    let by_new = client.login(2, "alice", &key(2), None).await.unwrap();
    assert_eq!(by_new, account(2, "Alice"));
    client.close().await;
    again.shutdown().await;
}
