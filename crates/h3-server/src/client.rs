//! A headless client (E16 M1 T3): speaks the protocol over WebSocket, for the end-to-end tests
//! and for tools; a viewer is the client lane's. It derives an account's key as every client does
//! (E16 M3 T2a, [`Key::derive`]). It answers the server's pings only while it reads
//! ([`Client::recv`]): a client that reads nothing for the server's `ping_timeout_ms` is dropped
//! as gone (SRV-Q20). Its hello names a fresh nonce and the address it dialed, and it checks the
//! welcome's signature by the server's key, and the key against one it pinned (E16 M6 T1,
//! ADR-0021 point 1), and the server's card by the welcome's key (E16 M6 T2), and each card the
//! server relays with the chain that vouches for it (E16 M6 T3, T4), or against the signed list it
//! was fetched for (T5). A server fetching its peers' cards dials them through it (`Client::dial`,
//! `wss://` through the TLS client, its reading bounded). Provisional.

use std::net::SocketAddr;
use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::{Signer, SigningKey};
use futures_util::{SinkExt, StreamExt};
use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::account::{self, CHALLENGE_BYTES, KEY_BYTES, NONCE_BYTES, SIGNATURE_BYTES};
use h3_protocol::codec;
use h3_protocol::wire::{
    ClientMessage, RelayedCard, Seat, ServerCard, ServerList, ServerMessage, SignedCard, SignedList,
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

use crate::Error;
use crate::accounts::{Authenticator, Ed25519};
use crate::{card, lists};

/// An account's signing key (E16 M3 T2a; ADR-0019 point 1; `E16-design.md` §10).
#[derive(Debug, Clone)]
pub struct Key(SigningKey);

impl Key {
    /// The key a login name and a password give on every device and every server: a seed of 32
    /// bytes by Argon2id (64 MiB, 3 passes, 1 lane, version 0x13) of the password, salted with
    /// `h3-account-v1:` and the login name, then the Ed25519 key of that seed. Costs a fraction of
    /// a second and 64 MiB: a client derives once and keeps the key.
    pub fn derive(login: &str, password: &str) -> Result<Self, Error> {
        let params =
            Params::new(64 * 1024, 3, 1, Some(32)).map_err(|e| Error(format!("argon2: {e}")))?;
        let mut seed = [0u8; 32];
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(
                password.as_bytes(),
                format!("h3-account-v1:{login}").as_bytes(),
                &mut seed,
            )
            .map_err(|e| Error(format!("argon2: {e}")))?;
        Ok(Self::from_seed(seed))
    }

    /// The Ed25519 key of `seed`.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(SigningKey::from_bytes(&seed))
    }

    /// The public key.
    pub fn public(&self) -> [u8; KEY_BYTES] {
        self.0.verifying_key().to_bytes()
    }

    /// The signature of `message`.
    pub fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES] {
        self.0.sign(message).to_bytes()
    }
}

/// One client connection.
#[derive(Debug)]
pub struct Client {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// The address its hello names as the one it dialed (E16 M6 T1).
    address: String,
    /// The server key it expects, if it pinned one (E16 M6 T1).
    pinned: Option<[u8; KEY_BYTES]>,
    /// The welcome's server id and challenge (E16 M3 T2a) and the server's key (E16 M6 T1), once
    /// the hello was answered.
    welcome: Option<(String, [u8; CHALLENGE_BYTES], [u8; KEY_BYTES])>,
    /// The bytes it reads still, pings included, before it takes the connection for closed (E16
    /// M6 T3: a peer's fetch); unbounded but for a fetch.
    budget: usize,
}

impl Client {
    /// Connects to a server.
    pub async fn connect(address: SocketAddr) -> Result<Self, Error> {
        Self::connect_forwarded(address, None).await
    }

    /// Connects to the server at `address` as a reverse proxy does for the client `forwarded`,
    /// naming it in the handshake's `X-Forwarded-For` header (E16 M3 T4b).
    pub async fn connect_forwarded(
        address: SocketAddr,
        forwarded: Option<&str>,
    ) -> Result<Self, Error> {
        let failed = |e: &dyn std::fmt::Display| Error(format!("connect {address}: {e}"));
        let mut request = format!("ws://{address}")
            .into_client_request()
            .map_err(|e| failed(&e))?;
        if let Some(forwarded) = forwarded {
            let value = HeaderValue::from_str(forwarded).map_err(|e| failed(&e))?;
            request.headers_mut().insert("X-Forwarded-For", value);
        }
        let (socket, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| failed(&e))?;
        Ok(Self {
            socket,
            address: format!("ws://{address}"),
            pinned: None,
            welcome: None,
            budget: usize::MAX,
        })
    }

    /// Dials the server at `url` (`ws://` or `wss://`) as a server fetching a peer's card does (E16
    /// M6 T3): `wss://` through the TLS client `tls`, reading frames of at most `frame` bytes and
    /// `budget` bytes in all. Its hello names `url` as the address dialed.
    pub(crate) async fn dial(
        url: &str,
        tls: Arc<rustls::ClientConfig>,
        (frame, budget): (usize, usize),
    ) -> Result<Self, Error> {
        let bounds = WebSocketConfig::default()
            .max_message_size(Some(frame))
            .max_frame_size(Some(frame));
        // The connector for `ws://` too: tokio-tungstenite dials that plain.
        let connector = Some(Connector::Rustls(tls));
        let (socket, _) =
            tokio_tungstenite::connect_async_tls_with_config(url, Some(bounds), true, connector)
                .await
                .map_err(|e| Error(format!("connect: {e}")))?;
        Ok(Self {
            socket,
            address: url.to_owned(),
            pinned: None,
            welcome: None,
            budget,
        })
    }

    /// Names `url` in the hello as the address it dialed (E16 M6 T1), as a client that reached the
    /// server through a proxy or by another of its addresses does.
    pub fn dialed(&mut self, url: &str) {
        url.clone_into(&mut self.address);
    }

    /// Expects the server's key to be `key` (E16 M6 T1): a welcome with another key fails.
    pub fn pin(&mut self, key: [u8; KEY_BYTES]) {
        self.pinned = Some(key);
    }

    /// The server's key the welcome carried (E16 M6 T1).
    pub fn server_key(&self) -> Option<[u8; KEY_BYTES]> {
        self.welcome.as_ref().map(|&(_, _, key)| key)
    }

    /// Sends one message.
    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), Error> {
        self.send_raw(Message::text(codec::encode_text(message)))
            .await
    }

    /// Sends one WebSocket message as it is (malformed input in tests).
    pub async fn send_raw(&mut self, message: Message) -> Result<(), Error> {
        self.socket
            .send(message)
            .await
            .map_err(|e| Error(format!("send: {e}")))
    }

    /// The next message from the server; `None` once the connection closed, or once it read its
    /// budget.
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        loop {
            let message = self.socket.next().await?.ok()?;
            self.budget = self.budget.checked_sub(message.len())?;
            match message {
                Message::Text(text) => {
                    return codec::decode_server(text.as_bytes(), usize::MAX).ok();
                }
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
                Message::Binary(_) | Message::Close(_) => return None,
            }
        }
    }

    /// Says hello in this protocol version, with a fresh nonce; the server's answer. A welcome
    /// whose signature does not verify by its key, or whose key is not the one pinned, fails
    /// (E16 M6 T1).
    pub async fn hello(&mut self, name: &str) -> Result<Option<ServerMessage>, Error> {
        let mut nonce = [0u8; NONCE_BYTES];
        getrandom::fill(&mut nonce).map_err(|e| Error(format!("OS randomness: {e}")))?;
        self.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            client: crate::server_name().replace("server", "client"),
            name: name.to_owned(),
            nonce: account::to_hex(&nonce),
            address: self.address.clone(),
        })
        .await?;
        let answer = self.recv().await;
        if let Some(ServerMessage::Welcome {
            id,
            challenge,
            key,
            signature,
            ..
        }) = &answer
        {
            let parsed = (
                account::from_hex(challenge),
                account::from_hex(key),
                account::from_hex(signature),
            );
            let (Some(challenge), Some(key), Some(signature)) = parsed else {
                return Err(Error(String::from("a welcome that does not parse")));
            };
            let signed = account::welcome_message(id, &nonce, &challenge);
            if !Ed25519.verify(&key, &signed, &signature) {
                return Err(Error(String::from(
                    "the welcome's signature does not verify",
                )));
            }
            if self.pinned.is_some_and(|pinned| pinned != key) {
                return Err(Error(format!(
                    "the server's key changed: {}",
                    account::to_hex(&key)
                )));
            }
            self.welcome = Some((id.clone(), challenge, key));
        }
        Ok(answer)
    }

    /// The welcome's server id and challenge (E16 M3 T2a).
    pub fn welcome(&self) -> Option<(&str, &[u8; CHALLENGE_BYTES])> {
        self.welcome.as_ref().map(|(id, c, _)| (id.as_str(), c))
    }

    /// The server's card (E16 M6 T2): asks with `Describe` after the hello, and checks the answer
    /// as a client checks a card ([`card::verify`]: its own key's signature, its times, its bounds)
    /// and that its key is the welcome's. The card, and as it was signed.
    pub async fn describe(&mut self, seq: u32) -> Result<(ServerCard, SignedCard), Error> {
        let key = self
            .server_key()
            .ok_or_else(|| Error(String::from("a card before the welcome")))?;
        self.send(&ClientMessage::Describe { seq }).await?;
        let Some(ServerMessage::Described {
            seq: answered,
            card,
        }) = self.recv().await
        else {
            return Err(Error(String::from("no card")));
        };
        let read = card::verify(&card, card::wall_ms())
            .map_err(|e| Error(format!("the card does not hold: {e:?}")))?;
        if answered != seq || read.key != account::to_hex(&key) {
            return Err(Error(String::from(
                "another card than the welcome's server's",
            )));
        }
        Ok((read, card))
    }

    /// A page of the cards the server relays (E16 M6 T3): asks with `ListPeers` from the `from`th;
    /// the number of cards it relays and the page's, as they came.
    pub async fn list_peers(
        &mut self,
        seq: u32,
        from: u32,
    ) -> Result<(u32, Vec<RelayedCard>), Error> {
        self.send(&ClientMessage::ListPeers { seq, from }).await?;
        match self.recv().await {
            Some(ServerMessage::Peers {
                seq: answered,
                from: page,
                total,
                entries,
            }) if answered == seq && page == from => Ok((total, entries)),
            // Never the message itself: a peer's could fill a log line (E16 M6's review).
            Some(_) => Err(Error(String::from("no page of peers: another answer"))),
            None => Err(Error(String::from(
                "no page of peers: the connection closed",
            ))),
        }
    }

    /// The lists of servers the server serves (E16 M6 T5), each as signed and as it reads now
    /// ([`lists::verify`]); one that does not hold fails.
    pub async fn lists(&mut self, seq: u32) -> Result<Vec<(ServerList, SignedList)>, Error> {
        self.send(&ClientMessage::Lists { seq }).await?;
        let Some(ServerMessage::ListsAre {
            seq: answered,
            lists,
        }) = self.recv().await
        else {
            return Err(Error(String::from("no lists")));
        };
        if answered != seq {
            return Err(Error(String::from("another answer's lists")));
        }
        (lists.into_iter())
            .map(|signed| match lists::verify(&signed, card::wall_ms()) {
                Ok(list) => Ok((list, signed)),
                Err(e) => Err(Error(format!("a list does not hold: {e:?}"))),
            })
            .collect()
    }

    /// Every card the server relays (E16 M6 T3–T5), page by page, each checked as a client checks
    /// it: the card holds ([`card::verify`]); a card relayed for a list (`list`) is named by that
    /// list among the server's lists that hold ([`Client::lists`], [`h3_protocol::list::names`]);
    /// any other, the chain vouches for it from the server's own card `own` (its `Describe`)
    /// through the cards its `via` names, which the server relays too
    /// ([`h3_protocol::card::chain`]).
    pub async fn relayed(
        &mut self,
        own: &ServerCard,
    ) -> Result<Vec<(ServerCard, RelayedCard)>, Error> {
        let lists = self.lists(u32::MAX).await?;
        let mut all = Vec::new();
        loop {
            let from = u32::try_from(all.len()).unwrap_or(u32::MAX);
            let (total, entries) = self.list_peers(from, from).await?;
            let last = entries.is_empty();
            for entry in entries {
                let read = card::verify(&entry.card, card::wall_ms())
                    .map_err(|e| Error(format!("a relayed card does not hold: {e:?}")))?;
                all.push((read, entry));
            }
            if last || all.len() >= usize::try_from(total).unwrap_or(usize::MAX) {
                break;
            }
        }
        let relayed = |key: &String| all.iter().map(|(card, _)| card).find(|c| c.key == *key);
        for (read, entry) in &all {
            if let Some(key) = &entry.list {
                let list = lists.iter().find(|(list, _)| list.key == *key);
                if !list.is_some_and(|(list, _)| h3_protocol::list::names(list, read)) {
                    return Err(Error(format!(
                        "a relayed card its list does not name: {}",
                        read.key
                    )));
                }
                continue;
            }
            let links: Option<Vec<&ServerCard>> = entry.via.iter().map(relayed).collect();
            let chain = links.map(|links| {
                let chain = std::iter::once(own).chain(links).chain([read]);
                chain.collect::<Vec<_>>()
            });
            if !chain.is_some_and(|chain| h3_protocol::card::chain(&chain)) {
                return Err(Error(format!(
                    "a relayed card its chain does not vouch for: {}",
                    read.key
                )));
            }
        }
        Ok(all)
    }

    /// Logs in as `login` by `key`, or registers the account as `display` (E16 M3 T2a): signs the
    /// welcome's server id and challenge; the server's answer. A viewer signs only if the id is
    /// the address it dialed (`E16-design.md` §10); this test client signs what its server names.
    pub async fn login(
        &mut self,
        seq: u32,
        login: &str,
        key: &Key,
        display: Option<&str>,
    ) -> Result<Option<ServerMessage>, Error> {
        let (id, challenge) = self
            .welcome()
            .ok_or_else(|| Error(String::from("login before the welcome")))?;
        let signed = account::login_message(id, challenge, login, display);
        self.send(&ClientMessage::Login {
            seq,
            login: login.to_owned(),
            key: account::to_hex(&key.public()),
            signature: account::to_hex(&key.sign(&signed)),
            display: display.map(str::to_owned),
        })
        .await?;
        Ok(self.recv().await)
    }

    /// Asks for `seat` in game `game`, with its token to re-take it.
    pub async fn join(
        &mut self,
        game: u32,
        seat: Seat,
        token: Option<String>,
    ) -> Result<(), Error> {
        self.send(&ClientMessage::Join { game, seat, token }).await
    }

    /// Creates a game of the server's offer `offer` and takes `seat` in it.
    pub async fn create(&mut self, offer: &str, seat: Seat) -> Result<(), Error> {
        self.send(&ClientMessage::CreateGame {
            seq: 0,
            offer: offer.to_owned(),
            seat,
            spectators: None,
        })
        .await
    }

    /// Re-keys the logged-in account `login` from `old` to `new` (E16 M3 T2b): both sign the
    /// welcome's server id and challenge, the login name and the new key; the server's answer.
    pub async fn rekey(
        &mut self,
        seq: u32,
        login: &str,
        old: &Key,
        new: &Key,
    ) -> Result<Option<ServerMessage>, Error> {
        let (id, challenge) = self
            .welcome()
            .ok_or_else(|| Error(String::from("re-key before the welcome")))?;
        let signed = account::rekey_message(id, challenge, login, &new.public());
        self.send(&ClientMessage::Rekey {
            seq,
            key: account::to_hex(&new.public()),
            by_old: account::to_hex(&old.sign(&signed)),
            by_new: account::to_hex(&new.sign(&signed)),
        })
        .await?;
        Ok(self.recv().await)
    }

    /// Renames the logged-in account to `display` (E16 M3 T2b); the server's answer.
    pub async fn rename(
        &mut self,
        seq: u32,
        display: &str,
    ) -> Result<Option<ServerMessage>, Error> {
        self.send(&ClientMessage::Rename {
            seq,
            display: display.to_owned(),
        })
        .await?;
        Ok(self.recv().await)
    }

    /// Sets whether the logged-in account's finished games' replays may be listed (E16 M3 T2b);
    /// the server's answer.
    pub async fn set_listed(
        &mut self,
        seq: u32,
        listed: bool,
    ) -> Result<Option<ServerMessage>, Error> {
        self.send(&ClientMessage::SetListed { seq, listed }).await?;
        Ok(self.recv().await)
    }

    /// Closes the connection.
    pub async fn close(mut self) {
        let _ = self.socket.close(None).await;
    }
}
