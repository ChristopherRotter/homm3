//! The session core's workload (E16 M1 T4, `benches/server/`): scripted clients play hosted
//! battles frame by frame through the connection's hello gate and the session core, as a
//! connection task and the game's thread do, without sockets or threads. `cargo xtask bench-check`
//! counts the instructions inside [`serve_frame`], the server's whole work for one client frame:
//! decode, validate, apply, project, encode. The clients' own work (choosing, encoding a command,
//! finding their turn) stays outside the count.
//!
//! Provisional (E16 M1 T4): the clients' choice (a strike if there is one, as the end-to-end
//! tests play, so that battles end), fixed tokens, no queries or fetches.

use h3_battle::BattleSetup;
use h3_content::Ruleset;
use h3_kernel::{Rng, StreamId};
use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::codec::{self, DEFAULT_MAX_CLIENT_FRAME};
use h3_protocol::connection::{Connection, Received};
use h3_protocol::session::{ConnId, GameConfig, Outbound, SeatKind, Session};
use h3_protocol::wire::{ClientMessage, Command, Seat, ServerMessage};

use crate::Error;

/// The hosted game's id.
const GAME_ID: u32 = 1;

/// The two scripted clients, attacker and defender.
const CONNS: [ConnId; 2] = [ConnId(1), ConnId(2)];

/// The start of a `YourTurn` frame (serde's externally tagged form, ADR-0018 point 4).
const YOUR_TURN: &str = "{\"YourTurn\"";

/// What the workload did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Totals {
    /// Games played.
    pub games: u64,
    /// Games that reached their end within the command limit.
    pub ended: u64,
    /// Frames the clients sent.
    pub frames_in: u64,
    /// Frames the server sent.
    pub frames_out: u64,
    /// Bytes the server sent.
    pub bytes_out: u64,
}

/// One client frame through the server: the connection's hello gate, then the session core; the
/// frames to send go to `out`. Never inlined: `cargo xtask bench-check` counts inside it.
#[inline(never)]
pub fn serve_frame(
    session: &mut Session<'_>,
    connection: &mut Connection,
    conn: ConnId,
    frame: &[u8],
    out: &mut Vec<Outbound>,
) {
    match connection.receive(frame, "h3-server bench") {
        Received::Game(message) => session.handle(conn, message, out),
        Received::Reply { frame, close } => out.push(Outbound {
            to: conn,
            frame,
            close,
        }),
    }
}

/// A scripted client's command among `legal`: a strike if there is one, else one that stays in
/// the battle (a uniform choice rarely ends a battle of the `full` scenario).
pub fn choose(legal: &[Command], rng: &mut Rng) -> Option<Command> {
    let strikes: Vec<Command> = legal
        .iter()
        .copied()
        .filter(|c| matches!(c, Command::Attack { .. } | Command::Shoot { .. }))
        .collect();
    let choices: Vec<Command> = if strikes.is_empty() {
        legal
            .iter()
            .copied()
            .filter(|c| !matches!(c, Command::Retreat {} | Command::Surrender {}))
            .collect()
    } else {
        strikes
    };
    let len = u32::try_from(choices.len()).ok().filter(|&len| len > 0)?;
    choices.get(rng.below(len) as usize).copied()
}

/// The connection told to act in `out` and its legal commands: a `YourTurn` ends what the server
/// sends after a command, so the clients read only that frame.
fn turn(out: &[Outbound]) -> Option<(usize, Vec<Command>)> {
    let last = out.iter().rev().find(|o| o.frame.starts_with(YOUR_TURN))?;
    let seat = CONNS.iter().position(|&c| c == last.to)?;
    match codec::decode_server(last.frame.as_bytes(), usize::MAX) {
        Ok(ServerMessage::YourTurn { legal, .. }) => Some((seat, legal)),
        _ => None,
    }
}

/// Plays one hosted battle of `setup` with `seed`: both clients say hello and take their seats,
/// then answer every `YourTurn`, until the game ends or `max_commands` were sent.
pub fn play_game(
    ruleset: &Ruleset,
    setup: &BattleSetup,
    seed: u64,
    max_commands: u32,
    totals: &mut Totals,
) -> Result<(), Error> {
    let config = GameConfig {
        game: GAME_ID,
        setup: setup.clone(),
        seed,
        checkpoint_every: 25,
        shadow: false,
        spectators: 0,
    };
    let seats = ["bench-attacker", "bench-defender"].map(|token| SeatKind::Player {
        token: token.to_owned(),
    });
    let mut session =
        Session::new(ruleset, config, seats).map_err(|e| Error(format!("setup: {e}")))?;
    let mut connections = [const { Connection::new(DEFAULT_MAX_CLIENT_FRAME) }; 2];
    let mut rng = Rng::for_stream(seed, StreamId::AGENT);
    let mut out = Vec::new();
    let mut send = |session: &mut Session<'_>, seat: usize, message: &ClientMessage| {
        let frame = codec::encode(message);
        out.clear();
        serve_frame(
            session,
            &mut connections[seat],
            CONNS[seat],
            &frame,
            &mut out,
        );
        // The host journals them (E16 M2 T2): file work, outside the count.
        session.drain_entries();
        totals.frames_in += 1;
        totals.frames_out += out.len() as u64;
        totals.bytes_out += out.iter().map(|o| o.frame.len() as u64).sum::<u64>();
        turn(&out)
    };
    let mut next = None;
    for (seat, place) in [Seat::Attacker, Seat::Defender].into_iter().enumerate() {
        let hello = ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            client: String::from("h3-server bench client"),
            name: format!("bench {seat}"),
        };
        send(&mut session, seat, &hello);
        let join = ClientMessage::Join {
            game: GAME_ID,
            seat: place,
            token: None,
        };
        next = send(&mut session, seat, &join);
    }
    for seq in 1..=max_commands {
        let Some((seat, legal)) = next else { break };
        let command = choose(&legal, &mut rng)
            .ok_or_else(|| Error(String::from("a turn without a command")))?;
        next = send(&mut session, seat, &ClientMessage::Command { seq, command });
    }
    totals.games += 1;
    totals.ended += u64::from(session.result().is_some());
    Ok(())
}

/// The workload: `games` hosted battles of `setup`, game `g` with the seed `seed + g`.
pub fn run(
    ruleset: &Ruleset,
    setup: &BattleSetup,
    seed: u64,
    games: u64,
    max_commands: u32,
) -> Result<Totals, Error> {
    let mut totals = Totals::default();
    for g in 0..games {
        play_game(
            ruleset,
            setup,
            seed.wrapping_add(g),
            max_commands,
            &mut totals,
        )?;
    }
    Ok(totals)
}
