//! The hosted game: preparing it from the configuration and running it on its own OS thread
//! (ADR-0018 points 3, 6, 8 and 12). The thread owns the ruleset and the session core; the
//! connection tasks talk to it over bounded channels.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::thread;

use h3_battle::{BattleSide, BattleState, LegalCommands};
use h3_content::Ruleset;
use h3_contracts::BattleSetup;
use h3_kernel::{Rng, StreamId};
use h3_protocol::session::{Bot, ConnId, GameConfig, Outbound, SeatKind, Session};
use h3_protocol::wire::ClientMessage;
use h3_sim::{Replay, ReplayHeader};
use tokio::sync::mpsc;

use crate::Error;
use crate::config::{BotSeat, Config};

/// The id clients name the hosted game by (one game per server in M1).
pub const GAME_ID: u32 = 1;

/// A frame for a connection task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame {
    /// The message's text.
    pub text: String,
    /// Whether the connection closes after it.
    pub close: bool,
}

/// What a connection task tells the game.
#[derive(Debug)]
pub(crate) enum ToGame {
    /// A connection opened; its frames go to `to`.
    Open {
        /// The connection.
        conn: ConnId,
        /// Its outbound queue.
        to: mpsc::Sender<Frame>,
    },
    /// A message the connection's hello gate let through.
    Message {
        /// The connection.
        conn: ConnId,
        /// The message.
        message: ClientMessage,
    },
    /// The connection closed.
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

/// A game ready to run: its ruleset, its battle and its secrets.
#[derive(Debug)]
pub struct Prepared {
    /// The server's ruleset.
    pub ruleset: Ruleset,
    /// The scenario's id (the replay file names it).
    pub scenario: String,
    /// The battle, its obstacles placed.
    pub setup: BattleSetup,
    /// The battle's seed.
    pub seed: u64,
    /// The seats: a token per player, `None` for the bot's.
    pub tokens: [Option<String>; 2],
    /// The seed of the bot's choices.
    pub bot_seed: u64,
    /// Where the replay goes at the end.
    pub replay_path: PathBuf,
    /// A state hash in the journal after every this many commands.
    pub checkpoint_every: u32,
}

impl Prepared {
    /// Loads the scenario and draws the secrets from the OS (ADR-0018 point 8): the battle's
    /// seed; the obstacles from a separate seed that is then forgotten, so that the layout every
    /// viewer sees tells nothing about the battle's seed; a token per player seat; the bot's
    /// seed. Checks that the battle starts.
    pub fn new(config: &Config) -> Result<Self, Error> {
        let at = |e: &dyn std::fmt::Display| Error(format!("{}: {e}", config.scenario.display()));
        let text = fs::read_to_string(&config.scenario).map_err(|e| at(&e))?;
        let def = h3_pack::parse_scenario(&text).map_err(|e| at(&e))?;
        let profile = config.profile.as_deref().unwrap_or(&def.profile);
        let ruleset =
            h3_cli::load_ruleset(&config.packs, &def.packs, profile).map_err(|e| at(&e))?;
        let mut setup = def.resolve(&ruleset).map_err(|e| at(&e))?;
        if let (true, Some(terrain)) = (def.random_obstacles, setup.terrain) {
            let mut rng = Rng::for_stream(os_u64()?, StreamId::OBSTACLES);
            setup.obstacles = h3_battle::generate_obstacles(&ruleset, terrain, &mut rng);
        }
        if config.bots.contains(&BotSeat::Attacker) && config.bots.contains(&BotSeat::Defender) {
            return Err(Error(String::from(
                "bots play both seats: a hosted game needs a player",
            )));
        }
        let seed = os_u64()?;
        BattleState::new(&ruleset, &setup, seed).map_err(|e| at(&e))?;
        let mut tokens = [None, None];
        for (side, token_slot) in [BotSeat::Attacker, BotSeat::Defender]
            .into_iter()
            .zip(&mut tokens)
        {
            if !config.bots.contains(&side) {
                *token_slot = Some(token()?);
            }
        }
        let replay_path = config.data.join("replays").join(format!(
            "{}-game{GAME_ID}-{:016x}.toml",
            def.id,
            os_u64()?
        ));
        Ok(Self {
            ruleset,
            scenario: def.id,
            setup,
            seed,
            tokens,
            bot_seed: os_u64()?,
            replay_path,
            checkpoint_every: config.checkpoint_every,
        })
    }

    /// Runs the game on its own OS thread until every sender of `inbox` is gone.
    pub(crate) fn spawn(self, inbox: mpsc::Receiver<ToGame>) -> Result<(), Error> {
        thread::Builder::new()
            .name(format!("game-{GAME_ID}"))
            .spawn(move || self.run(inbox))
            .map(|_| ())
            .map_err(|e| Error(format!("game thread: {e}")))
    }

    /// The game's loop: one message at a time through the session core.
    fn run(self, mut inbox: mpsc::Receiver<ToGame>) {
        let Self {
            ruleset,
            scenario,
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
            game: GAME_ID,
            setup,
            seed,
            checkpoint_every,
        };
        // `Prepared::new` started this battle already.
        let Ok(mut session) = Session::new(&ruleset, config, seats) else {
            return;
        };
        let mut conns = BTreeMap::new();
        let mut out = Vec::new();
        let mut published = false;
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
                let text = replay_text(&ruleset, &scenario, &session);
                write_replay(&replay_path, &text);
                session.publish_replay(text);
            }
        }
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

/// Writes the replay file; a failure is logged, the game goes on.
fn write_replay(path: &std::path::Path, text: &str) {
    let written = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(path, text));
    match written {
        Ok(()) => eprintln!("h3-server: game {GAME_ID} over, replay {}", path.display()),
        Err(e) => eprintln!("h3-server: replay {}: {e}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use h3_protocol::codec;
    use h3_protocol::wire::{Seat, ServerMessage};

    use super::*;

    #[test]
    fn a_full_queue_drops_its_connection_and_frees_the_seat() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let players = ["a", "b"].map(|t| SeatKind::Player {
            token: String::from(t),
        });
        let config = GameConfig {
            game: GAME_ID,
            setup,
            seed: 1,
            checkpoint_every: 0,
        };
        let mut session = Session::new(&r, config, players).unwrap();
        let (to, mut queue) = mpsc::channel(1);
        let mut conns = BTreeMap::from([(ConnId(1), to)]);
        let join = |token: Option<&str>| ClientMessage::Join {
            game: GAME_ID,
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
