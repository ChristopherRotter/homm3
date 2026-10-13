//! A scenario file to a record (E17 M1 T7; ADR-0020 point 8): the battle a scenario file sets up,
//! as `h3 run` reads it (its packs compiled with its profile, obstacles generated from the seed
//! when it asks), hosted by `h3-protocol`'s session in-process, and the record of a spectator who
//! joined it: the manifest and the snapshot at the battle's start. An adapter: it reads files.

use std::fs;
use std::path::Path;

use h3_battle::BattleSetup;
use h3_content_battle::BattleRuleset;
use h3_kernel::{Rng, StreamId};
use h3_protocol::codec;
use h3_protocol::session::{ConnId, GameConfig, SeatKind, Session};
use h3_protocol::wire::{ClientMessage, Command, Seat, ServerMessage};

/// The game's id in the in-process session.
const GAME: u32 = 1;

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The texts of the packs `ids`: `<packs>/<id>/core.toml` and, where it exists, `battle.toml`,
/// as `h3_cli::load_ruleset` reads them.
fn pack_texts(packs: &Path, ids: &[String]) -> Result<Vec<(String, String)>, String> {
    let mut texts = Vec::new();
    for id in ids {
        if id.is_empty() || id.contains(['/', '\\', '.', ':']) {
            return Err(format!("invalid pack id `{id}`"));
        }
        let dir = packs.join(id);
        let battle = dir.join("battle.toml");
        let battle = if battle.is_file() {
            read(&battle)?
        } else {
            String::new()
        };
        texts.push((read(&dir.join("core.toml"))?, battle));
    }
    Ok(texts)
}

/// The battle `scenario` sets up with `seed`, its packs read from `packs`: the ruleset its packs
/// compile to with its profile, and the setup (obstacles generated from the seed when it asks).
pub fn battle(
    scenario: &Path,
    packs: &Path,
    seed: u64,
) -> Result<(BattleRuleset, BattleSetup), String> {
    let def = h3_pack_battle::parse_scenario(&read(scenario)?)
        .map_err(|e| format!("{}: {e}", scenario.display()))?;
    let texts = pack_texts(packs, &def.packs)?;
    let refs: Vec<(&str, &str)> = texts
        .iter()
        .map(|(c, b)| (c.as_str(), b.as_str()))
        .collect();
    let ruleset = h3_pack_battle::compile(&refs, &def.profile)
        .map_err(|e| format!("{}: {e}", packs.display()))?;
    let mut setup = def
        .resolve(&ruleset)
        .map_err(|e| format!("{}: {e}", scenario.display()))?;
    if let (true, Some(terrain)) = (def.random_obstacles, setup.terrain) {
        let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
        setup.obstacles = h3_battle::generate_obstacles(&ruleset, terrain, &mut rng);
    }
    Ok((ruleset, setup))
}

/// What the original's map needs of a scenario's battle (E17 M2 T1, `situation`): its terrain's
/// key, and each side's stacks (slot, the creature's singular name, count) and formation.
#[derive(Debug, PartialEq, Eq)]
pub struct Sides {
    pub terrain: Option<String>,
    pub stacks: [Vec<(u8, String, u32)>; 2],
    pub tight: [bool; 2],
}

/// The [`Sides`] of the battle `scenario` sets up, its packs read from `packs`.
pub fn sides(scenario: &Path, packs: &Path) -> Result<Sides, String> {
    let (ruleset, setup) = battle(scenario, packs, 1)?;
    let terrain = setup
        .terrain
        .and_then(|t| ruleset.terrains().get(t.index()))
        .map(|t| t.key.as_str().to_owned());
    let stacks = [&setup.attacker, &setup.defender].map(|army| {
        army.stacks
            .iter()
            .map(|s| {
                // The core ruleset's creature (its name), not the battle's (its numbers).
                let name = (*ruleset).creature(s.creature).map(|c| c.name.clone());
                (s.slot, name.unwrap_or_default(), s.count)
            })
            .collect()
    });
    let tight = h3_pack_battle::tight_sides(setup.formations);
    Ok(Sides {
        terrain,
        stacks,
        tight: ["attacker", "defender"].map(|side| tight.iter().any(|t| t == side)),
    })
}

/// The record of a spectator of the battle `scenario` sets up with `seed`, its packs read from
/// `packs`.
pub fn record(scenario: &Path, packs: &Path, seed: u64) -> Result<String, String> {
    played(scenario, packs, seed, &[])
}

/// The record of a spectator of the battle `scenario` sets up with `seed` when each seat sends
/// its `commands` in order (E17 M2 T4a: the recorded sequences); a rejected command is an error.
pub fn played(
    scenario: &Path,
    packs: &Path,
    seed: u64,
    commands: &[(Seat, Command)],
) -> Result<String, String> {
    let (ruleset, setup) = battle(scenario, packs, seed)?;
    let config = GameConfig {
        game: GAME,
        setup,
        seed,
        checkpoint_every: 0,
        shadow: false,
        spectators: 1,
        watchable: true,
        record_bytes: usize::MAX,
        ranked: None,
    };
    let seats = ["attacker", "defender"].map(|token| SeatKind::Player {
        token: token.to_owned(),
    });
    let mut session =
        Session::new(&ruleset, config, seats).map_err(|e| format!("the setup: {e:?}"))?;
    let (viewer, mut out) = (ConnId(1), Vec::new());
    let join = ClientMessage::Join {
        game: GAME,
        seat: Seat::Spectator,
        token: None,
    };
    session.handle(viewer, join, &mut out);
    if !commands.is_empty() {
        for (conn, seat, token) in [
            (2, Seat::Attacker, "attacker"),
            (3, Seat::Defender, "defender"),
        ] {
            let join = ClientMessage::Join {
                game: GAME,
                seat,
                token: Some(token.to_owned()),
            };
            session.handle(ConnId(conn), join, &mut out);
        }
    }
    for (seq, &(seat, command)) in (1..).zip(commands) {
        let conn = ConnId(if seat == Seat::Attacker { 2 } else { 3 });
        let mut replies = Vec::new();
        session.handle(conn, ClientMessage::Command { seq, command }, &mut replies);
        for frame in &replies {
            if let Ok(ServerMessage::Rejected { code, .. }) =
                codec::decode_server(frame.frame.as_bytes(), usize::MAX)
            {
                return Err(format!("command {seq} ({command:?}): rejected, {code:?}"));
            }
        }
    }
    let mut out = Vec::new();
    session.handle(viewer, ClientMessage::FetchRecord { seq: 1 }, &mut out);
    for frame in out {
        if let Ok(ServerMessage::Record { record, .. }) =
            codec::decode_server(frame.frame.as_bytes(), usize::MAX)
        {
            return Ok(record);
        }
    }
    Err("the session sent no record".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn a_scenario_becomes_a_spectators_record() {
        let scenario = repo().join("packs/synthetic/scenarios/duel.toml");
        let text = record(&scenario, &repo().join("packs"), 1).unwrap();
        let record = h3_view::record::parse(&text).unwrap();
        assert_eq!(record.header.seat, Seat::Spectator);
        let snapshot = record.first_snapshot().unwrap();
        assert!(snapshot.spectator);
        assert!(!snapshot.stacks.is_empty());
        // The duel asks for random obstacles: each placed one has its type.
        let typed = snapshot
            .removable_obstacles
            .iter()
            .all(|o| o.kind.is_some());
        assert!(typed);
    }

    #[test]
    fn a_bad_scenario_is_an_error() {
        let packs = repo().join("packs");
        assert!(record(Path::new("/no/such.toml"), &packs, 1).is_err());
        let dir = std::env::temp_dir().join(format!("h3-view-scenario-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.toml");
        fs::write(
            &bad,
            "[scenario]\nid = \"x\"\npacks = [\"../x\"]\nprofile = \"a:b\"\n",
        )
        .unwrap();
        assert!(
            record(&bad, &packs, 1)
                .unwrap_err()
                .contains("invalid pack id")
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
