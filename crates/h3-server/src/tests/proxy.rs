//! Behind a reverse proxy over loopback (E16 M3 T4b): clients behind a trusted proxy count as
//! their own addresses, an IPv6 client by its /64; a forwarded header from an untrusted peer is
//! ignored.

use std::net::IpAddr;

use super::*;

/// A configuration that trusts `proxies` and takes one connection per address.
fn behind(proxies: Vec<IpAddr>) -> Config {
    let mut config = config(Limits {
        connections_per_address: 1,
        ..fast()
    });
    config.server.trusted_proxies = proxies;
    config
}

/// A client connected for `client` as a proxy connects (its `X-Forwarded-For`), greeted; `None`
/// if the server closed the connection instead.
async fn through(running: &Running, client: Option<&str>) -> Option<Client> {
    let mut connected = Client::connect_forwarded(running.address, client)
        .await
        .ok()?;
    match connected.hello("test").await {
        Ok(Some(ServerMessage::Welcome { .. })) => Some(connected),
        _ => None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clients_behind_a_trusted_proxy_count_as_their_own_addresses() {
    let loopback = IpAddr::from([127, 0, 0, 1]);
    let running = start(&behind(vec![loopback])).await.unwrap();
    let first = through(&running, Some("203.0.113.1")).await;
    assert!(first.is_some(), "a place per client");
    let second = through(&running, Some("203.0.113.2")).await;
    assert!(second.is_some(), "another client, another place");
    let again = through(&running, Some("198.51.100.7, 203.0.113.1")).await;
    assert!(
        again.is_none(),
        "the last entry is the client, whose place is taken"
    );
    // An IPv6 client counts by its /64.
    let six = through(&running, Some("2001:db8::1")).await;
    assert!(six.is_some());
    assert!(
        through(&running, Some("2001:db8::2")).await.is_none(),
        "the same /64"
    );
    let other = through(&running, Some("2001:db8:0:1::1")).await;
    assert!(other.is_some(), "another /64");
    // Without a header that reads, the proxy's own address counts.
    let proxy = through(&running, None).await;
    assert!(proxy.is_some());
    assert!(through(&running, Some("unknown")).await.is_none());
    // A client's place is free again once its connection closed.
    first.unwrap().close().await;
    let mut attempts = 0;
    while through(&running, Some("203.0.113.1")).await.is_none() {
        attempts += 1;
        assert!(attempts < 100, "the place is never freed");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    for client in [second, six, other, proxy].into_iter().flatten() {
        client.close().await;
    }
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_forwarded_header_from_an_untrusted_peer_is_ignored() {
    let running = start(&behind(vec![])).await.unwrap();
    let first = through(&running, Some("203.0.113.1")).await;
    assert!(first.is_some());
    let second = through(&running, Some("203.0.113.2")).await;
    assert!(second.is_none(), "both count as the peer's own address");
    first.unwrap().close().await;
    running.shutdown().await;
}
