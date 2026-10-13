//! The server's key and the signed welcome over loopback (E16 M6 T1; ADR-0021 point 1): every
//! welcome verifies by the server's key (the test client checks each hello's); the key survives a
//! restart, another data directory is another identity, which a client that pinned the first
//! refuses, and a key file that does not parse stops the start; a client that dialed another of the
//! server's addresses gets it as the id and logs in there, with a login signed for it.

use h3_protocol::account;

use super::*;
use crate::client::Key;
use crate::key::KEY_FILE;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_server_key_signs_every_welcome_and_survives_a_restart() {
    let first = config(Limits::default());
    let running = start(&first).await.unwrap();
    let client = greeted(&running).await;
    let key = client.server_key().unwrap();
    let file = std::fs::read_to_string(first.server.data.join(KEY_FILE)).unwrap();
    client.close().await;
    running.shutdown().await;
    // The same data directory: the same key, which a client that pinned it accepts.
    let running = start(&first).await.unwrap();
    let mut pinned = Client::connect(running.address).await.unwrap();
    pinned.pin(key);
    let welcome = pinned.hello("test").await.unwrap();
    assert!(matches!(welcome, Some(ServerMessage::Welcome { .. })));
    let again = std::fs::read_to_string(first.server.data.join(KEY_FILE)).unwrap();
    assert_eq!(again, file, "never written again");
    running.shutdown().await;
    // Another data directory, another identity: the pinning client refuses it.
    let other = config(Limits::default());
    let running = start(&other).await.unwrap();
    let mut pinned = Client::connect(running.address).await.unwrap();
    pinned.pin(key);
    let error = pinned.hello("test").await.unwrap_err();
    assert!(error.0.contains("the server's key changed"), "{error}");
    running.shutdown().await;
    // A key file that does not parse stops the start, and stays as it was.
    std::fs::write(other.server.data.join(KEY_FILE), "not a key\n").unwrap();
    let error = start(&other).await.unwrap_err();
    assert!(error.0.contains("not a key"), "{error}");
    let kept = std::fs::read_to_string(other.server.data.join(KEY_FILE)).unwrap();
    assert_eq!(kept, "not a key\n");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_at_another_of_the_servers_addresses_logs_in_there() {
    let mut config = config(Limits::default());
    let public = "wss://h3.example.org";
    let lan = "ws://192.168.1.5:7878";
    config.server.public_url = Some(String::from(public));
    config.server.addresses = vec![String::from(lan)];
    let running = start(&config).await.unwrap();
    for (n, dialed, id) in [
        (1, lan, lan),
        (2, public, public),
        (3, "ws://h3.example.net", public),
    ] {
        let mut client = Client::connect(running.address).await.unwrap();
        client.dialed(dialed);
        client.hello("test").await.unwrap();
        assert_eq!(client.welcome().unwrap().0, id, "{dialed}");
        let answer = client
            .login(
                1,
                &format!("ann{n}"),
                &Key::from_seed([n; 32]),
                Some(&format!("Ann {n}")),
            )
            .await
            .unwrap();
        assert!(
            matches!(answer, Some(ServerMessage::Account { .. })),
            "{dialed}: {answer:?}"
        );
        client.close().await;
    }
    // At the LAN address, a login signed for the public one is refused.
    let mut client = Client::connect(running.address).await.unwrap();
    client.dialed(lan);
    client.hello("test").await.unwrap();
    let (_, challenge) = client.welcome().unwrap();
    let signed = account::login_message(public, challenge, "ann1", None);
    let key = Key::from_seed([1; 32]);
    client
        .send(&ClientMessage::Login {
            seq: 2,
            login: String::from("ann1"),
            key: account::to_hex(&key.public()),
            signature: account::to_hex(&key.sign(&signed)),
            display: None,
        })
        .await
        .unwrap();
    assert_eq!(
        client.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::BadLogin
        })
    );
    running.shutdown().await;
}
