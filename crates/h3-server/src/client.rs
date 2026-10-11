//! A headless client (E16 M1 T3): speaks the protocol over WebSocket, for the end-to-end tests
//! and for tools; a viewer is the client lane's. It derives an account's key as every client does
//! (E16 M3 T2a, [`Key::derive`]). Provisional.

use std::net::SocketAddr;

use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::{Signer, SigningKey};
use futures_util::{SinkExt, StreamExt};
use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::account::{self, CHALLENGE_BYTES, KEY_BYTES, SIGNATURE_BYTES};
use h3_protocol::codec;
use h3_protocol::wire::{ClientMessage, Seat, ServerMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::Error;

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
    /// The welcome's server id and challenge (E16 M3 T2a), once the hello was answered.
    welcome: Option<(String, [u8; CHALLENGE_BYTES])>,
}

impl Client {
    /// Connects to a server.
    pub async fn connect(address: SocketAddr) -> Result<Self, Error> {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .map_err(|e| Error(format!("connect {address}: {e}")))?;
        Ok(Self {
            socket,
            welcome: None,
        })
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

    /// The next message from the server; `None` once the connection closed.
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        loop {
            match self.socket.next().await? {
                Ok(Message::Text(text)) => {
                    return codec::decode_server(text.as_bytes(), usize::MAX).ok();
                }
                Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {}
                Ok(Message::Binary(_) | Message::Close(_)) | Err(_) => return None,
            }
        }
    }

    /// Says hello in this protocol version; the server's answer.
    pub async fn hello(&mut self, name: &str) -> Result<Option<ServerMessage>, Error> {
        self.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            client: crate::server_name().replace("server", "client"),
            name: name.to_owned(),
        })
        .await?;
        let answer = self.recv().await;
        if let Some(ServerMessage::Welcome { id, challenge, .. }) = &answer {
            self.welcome = account::from_hex(challenge).map(|c| (id.clone(), c));
        }
        Ok(answer)
    }

    /// The welcome's server id and challenge (E16 M3 T2a).
    pub fn welcome(&self) -> Option<(&str, &[u8; CHALLENGE_BYTES])> {
        self.welcome.as_ref().map(|(id, c)| (id.as_str(), c))
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

    /// Closes the connection.
    pub async fn close(mut self) {
        let _ = self.socket.close(None).await;
    }
}
