//! A headless client (E16 M1 T3): speaks the protocol over WebSocket, for the end-to-end tests
//! and for tools; a viewer is the client lane's. Provisional.

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::codec;
use h3_protocol::wire::{ClientMessage, Seat, ServerMessage};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::Error;

/// One client connection.
#[derive(Debug)]
pub struct Client {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Client {
    /// Connects to a server.
    pub async fn connect(address: SocketAddr) -> Result<Self, Error> {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .map_err(|e| Error(format!("connect {address}: {e}")))?;
        Ok(Self { socket })
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
