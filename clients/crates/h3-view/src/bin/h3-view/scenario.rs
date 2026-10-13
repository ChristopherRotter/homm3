//! A scenario file to a record (E17 M1 T7; ADR-0020 point 8): the battle a scenario file sets up,
//! as `h3 run` reads it (its packs compiled with its profile, obstacles generated from the seed
//! when it asks), hosted by `h3-protocol`'s session in-process, and the record of a spectator who
//! joined it: the manifest and the snapshot at the battle's start. An adapter: it reads files.

use std::fs;
use std::path::Path;

use h3_kernel::{Rng, StreamId};
use h3_protocol::codec;
use h3_protocol::session::{ConnId, GameConfig, SeatKind, Session};
use h3_protocol::wire::{ClientMessage, Seat, ServerMessage};

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

/// The record of a spectator of the battle `scenario` sets up with `seed`, its packs read from
/// `packs`.
pub fn record(scenario: &Path, packs: &Path, seed: u64) -> Result<String, String> {
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
