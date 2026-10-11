//! The hosted games (ADR-0018 points 3, 6, 8 and 12; E16 M2 T1): the offers, compiled once at
//! start; a game prepared from an offer with secrets from the OS and run on its own OS thread. The
//! thread owns the session core; the connection tasks talk to it over bounded channels, and it
//! tells the lobby where it is through the registry ([`crate::registry`]).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use h3_battle::{BattleSide, BattleState, LegalCommands};
use h3_content::Ruleset;
use h3_contracts::BattleSetup;
use h3_kernel::{Rng, StreamId};
use h3_protocol::map;
use h3_protocol::session::{Bot, ConnId, GameConfig, Outbound, SeatKind, Session};
use h3_protocol::wire::{self, ClientMessage, Seat};
use h3_sim::{Replay, ReplayHeader};
use tokio::sync::mpsc;

use crate::Error;
use crate::config::{self, BotSeat};
use crate::registry::{Shared, lock};

/// A frame for a connection task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame {
    /// The message's text.
    pub text: String,
    /// Whether the connection closes after it.
    pub close: bool,
}

/// What a connection task tells a game.
#[derive(Debug)]
pub(crate) enum ToGame {
    /// A connection joins the game; its frames go to `to`.
    Open {
        /// The connection.
        conn: ConnId,
        /// Its outbound queue for this game.
        to: mpsc::Sender<Frame>,
    },
    /// A message the connection's hello gate let through.
    Message {
        /// The connection.
        conn: ConnId,
        /// The message.
        message: ClientMessage,
    },
    /// The connection closed, or left for another game.
    Closed {
        /// The connection.
        conn: ConnId,
    },
}

/// 64 bits of OS randomness (ADR-0018 point 8).
pub(crate) fn os_u64() -> Result<u64, Error> {
    getrandom::u64().map_err(|e| Error(format!("OS randomness: {e}")))
}

/// A seat token: 128 bits of OS randomness as 32 hex digits (ADR-0018 point 6).
pub(crate) fn token() -> Result<String, Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| Error(format!("OS randomness: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// The sim's random agent as the server's bot (`h3_sim::rollout::choose`).
#[derive(Debug)]
pub(crate) struct RandomBot(pub Rng);

impl Bot for RandomBot {
    fn command(
        &mut self,
        state: &BattleState,
        ruleset: &Ruleset,
        side: BattleSide,
        legal: &LegalCommands,
    ) -> h3_battle::Command {
        h3_sim::rollout::choose(state, ruleset, side, legal, &mut self.0)
    }
}

/// A game configuration the server offers, compiled at start.
#[derive(Debug)]
pub struct Offer {
    /// The offer's id.
    pub id: String,
    /// The scenario's id (the replay file names it).
    pub scenario: String,
    /// The ruleset its games share.
    pub ruleset: Arc<Ruleset>,
    /// The battle, its obstacles not yet placed.
    pub setup: BattleSetup,
    /// Whether each game draws its own obstacles.
    pub random_obstacles: bool,
    /// The seats the server's bot plays.
    pub bots: Vec<BotSeat>,
    /// What the lobby shows of it.
    pub info: wire::Offer,
}

impl Offer {
    /// Loads the offer's scenario on the packs in `packs` and checks that its battle starts.
    pub fn load(config: &config::Offer, packs: &Path) -> Result<Self, Error> {
        let at = |e: &dyn std::fmt::Display| {
            Error(format!(
                "offer `{}`: {}: {e}",
                config.id,
                config.scenario.display()
            ))
        };
        let text = fs::read_to_string(&config.scenario).map_err(|e| at(&e))?;
        let def = h3_pack::parse_scenario(&text).map_err(|e| at(&e))?;
        let profile = config.profile.as_deref().unwrap_or(&def.profile);
        let ruleset = h3_cli::load_ruleset(packs, &def.packs, profile).map_err(|e| at(&e))?;
        let setup = def.resolve(&ruleset).map_err(|e| at(&e))?;
        BattleState::new(&ruleset, &setup, 0).map_err(|e| at(&e))?;
        let seats = config.bots.iter().map(|bot| match bot {
            BotSeat::Attacker => Seat::Attacker,
            BotSeat::Defender => Seat::Defender,
        });
        let info = map::offer(&config.id, &def.id, &ruleset, seats.collect());
        Ok(Self {
            id: config.id.clone(),
            scenario: def.id,
            random_obstacles: def.random_obstacles && setup.terrain.is_some(),
            ruleset: Arc::new(ruleset),
            setup,
            bots: config.bots.clone(),
            info,
        })
    }
}

/// A game ready to run: its offer, its battle and its secrets.
#[derive(Debug)]
pub(crate) struct Prepared {
    /// The game's id.
    game: u32,
    /// What it is made of.
    offer: Arc<Offer>,
    /// The battle, its obstacles placed.
    setup: BattleSetup,
    /// The battle's seed.
    seed: u64,
    /// The seats: a token per player, `None` for the bot's.
    tokens: [Option<String>; 2],
    /// The seed of the bot's choices.
    bot_seed: u64,
    /// Where the replay goes at the end.
    replay_path: PathBuf,
    /// A state hash in the journal after every this many commands.
    checkpoint_every: u32,
}

impl Prepared {
    /// Draws game `game`'s secrets from the OS (ADR-0018 point 8): the battle's seed; the
    /// obstacles from a separate seed that is then forgotten, so that the layout every viewer sees
    /// tells nothing about the battle's seed; a token per player seat; the bot's seed. Checks that
    /// the battle starts.
    pub(crate) fn new(
        offer: Arc<Offer>,
        game: u32,
        server: &config::Server,
    ) -> Result<Self, Error> {
        let mut setup = offer.setup.clone();
        if let (true, Some(terrain)) = (offer.random_obstacles, setup.terrain) {
            let mut rng = Rng::for_stream(os_u64()?, StreamId::OBSTACLES);
            setup.obstacles = h3_battle::generate_obstacles(&offer.ruleset, terrain, &mut rng);
        }
        let seed = os_u64()?;
        BattleState::new(&offer.ruleset, &setup, seed)
            .map_err(|e| Error(format!("offer `{}`, game {game}: {e}", offer.id)))?;
        let mut tokens = [None, None];
        for (side, slot) in [BotSeat::Attacker, BotSeat::Defender]
            .into_iter()
            .zip(&mut tokens)
        {
            if !offer.bots.contains(&side) {
                *slot = Some(token()?);
            }
        }
        let replay_path = server.data.join("replays").join(format!(
            "{}-game{game}-{:016x}.toml",
            offer.id,
            os_u64()?
        ));
        Ok(Self {
            game,
            offer,
            setup,
            seed,
            tokens,
            bot_seed: os_u64()?,
            replay_path,
            checkpoint_every: server.checkpoint_every,
        })
    }

    /// Runs the game on its own OS thread until it leaves the lobby.
    pub(crate) fn spawn(
        self,
        inbox: mpsc::Receiver<ToGame>,
        registry: Shared,
    ) -> Result<(), Error> {
        thread::Builder::new()
            .name(format!("game-{}", self.game))
            .spawn(move || self.run(inbox, &registry))
            .map(|_| ())
            .map_err(|e| Error(format!("game thread: {e}")))
    }

    /// The game's loop: one message at a time through the session core. After each, the lobby
    /// hears where the game is; a game that is not running and that no connection is bound to
    /// leaves the lobby and ends (E16 M2 T1).
    fn run(self, mut inbox: mpsc::Receiver<ToGame>, registry: &Shared) {
        let Self {
            game,
            offer,
            setup,
            seed,
            tokens,
            bot_seed,
            replay_path,
            checkpoint_every,
        } = self;
        let mut bots: u64 = 0;
        let seats = tokens.map(|token| match token {
            Some(token) => SeatKind::Player { token },
            None => {
                bots += 1;
                SeatKind::Bot(Box::new(RandomBot(Rng::for_stream(
                    bot_seed.wrapping_add(bots),
                    StreamId::AGENT,
                ))))
            }
        });
        let config = GameConfig {
            game,
            setup,
            seed,
            checkpoint_every,
        };
        // `Prepared::new` started this battle already.
        let Ok(mut session) = Session::new(&offer.ruleset, config, seats) else {
            return lock(registry).leave(game);
        };
        let mut conns = BTreeMap::new();
        let mut out = Vec::new();
        let mut published = false;
        let mut reported = (session.phase(), session.open_seats());
        while let Some(event) = inbox.blocking_recv() {
            match event {
                ToGame::Open { conn, to } => {
                    conns.insert(conn, to);
                }
                ToGame::Message { conn, message } => session.handle(conn, message, &mut out),
                ToGame::Closed { conn } => {
                    conns.remove(&conn);
                    session.disconnected(conn);
                }
            }
            for outbound in out.drain(..) {
                deliver(&mut conns, &mut session, outbound);
            }
            if !published && session.result().is_some() {
                published = true;
                let text = replay_text(&offer.ruleset, &offer.scenario, &session);
                write_replay(game, &replay_path, &text);
                session.publish_replay(text);
            }
            let now = (session.phase(), session.open_seats());
            let mut registry = lock(registry);
            if now != reported {
                registry.report(game, now.0, now.1.clone());
                reported = now;
            }
            if registry.close_if_idle(game, reported.0) {
                return;
            }
        }
        lock(registry).leave(game);
    }
}

/// Queues a frame for its connection. A full queue drops the connection: the game forgets it,
/// and its task, finding the queue closed, says `TooSlow` (ADR-0018 point 3).
pub(crate) fn deliver(
    conns: &mut BTreeMap<ConnId, mpsc::Sender<Frame>>,
    session: &mut Session<'_>,
    outbound: Outbound,
) {
    let Some(to) = conns.get(&outbound.to) else {
        return;
    };
    let frame = Frame {
        text: outbound.frame,
        close: outbound.close,
    };
    if let Err(mpsc::error::TrySendError::Full(_)) = to.try_send(frame) {
        conns.remove(&outbound.to);
        session.disconnected(outbound.to);
    }
}

/// The finished game's replay as the CLI's file (ADR-0018 point 12).
fn replay_text(ruleset: &Ruleset, scenario: &str, session: &Session<'_>) -> String {
    let journal = session.journal().clone();
    let replay = Replay {
        header: ReplayHeader::current(ruleset),
        seed: journal.seed,
        setup: journal.setup,
        commands: journal.commands,
        checkpoint_every: journal.checkpoint_every,
        checkpoints: journal.checkpoints,
        final_hash: session.state_hash(),
        result: session.result(),
    };
    h3_cli::replay_to_toml(&replay, scenario, ruleset).unwrap_or_else(|e| {
        eprintln!("h3-server: the replay does not render: {e}");
        String::new()
    })
}

/// Writes game `game`'s replay file; a failure is logged, the game goes on.
fn write_replay(game: u32, path: &Path, text: &str) {
    let written = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(path, text));
    match written {
        Ok(()) => eprintln!("h3-server: game {game} over, replay {}", path.display()),
        Err(e) => eprintln!("h3-server: replay {}: {e}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use h3_protocol::codec;
    use h3_protocol::wire::ServerMessage;

    use super::*;

    #[test]
    fn a_full_queue_drops_its_connection_and_frees_the_seat() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let players = ["a", "b"].map(|t| SeatKind::Player {
            token: String::from(t),
        });
        let config = GameConfig {
            game: 1,
            setup,
            seed: 1,
            checkpoint_every: 0,
        };
        let mut session = Session::new(&r, config, players).unwrap();
        let (to, mut queue) = mpsc::channel(1);
        let mut conns = BTreeMap::from([(ConnId(1), to)]);
        let join = |token: Option<&str>| ClientMessage::Join {
            game: 1,
            seat: Seat::Attacker,
            token: token.map(String::from),
        };
        let mut out = Vec::new();
        session.handle(ConnId(1), join(None), &mut out);
        assert_eq!(out.len(), 2, "Joined and Snapshot");
        for outbound in out.drain(..) {
            deliver(&mut conns, &mut session, outbound);
        }
        assert!(conns.is_empty(), "the snapshot found the queue full");
        let joined = queue.try_recv().unwrap();
        assert!(matches!(
            codec::decode_server(joined.text.as_bytes(), usize::MAX),
            Ok(ServerMessage::Joined { .. })
        ));
        // A frame for a connection the game forgot goes nowhere.
        session.handle(ConnId(1), join(None), &mut out);
        let refused = out.pop().unwrap();
        deliver(&mut conns, &mut session, refused);
        // The seat waits for its token.
        session.handle(ConnId(2), join(Some("a")), &mut out);
        assert!(matches!(
            codec::decode_server(out[0].frame.as_bytes(), usize::MAX),
            Ok(ServerMessage::Joined { .. })
        ));
    }
}
