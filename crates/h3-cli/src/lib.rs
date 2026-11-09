//! Library side of the `h3` tool: running random battles and the AI's and the adventure's
//! commands. The binary (`main.rs`) only parses arguments. The files the tool reads and writes
//! (packs, scenarios, maps, both replay files) are `h3-files`', re-exported here: the battle's
//! below, the adventure's in [`adventure`] (moved there with the user's file crate, 2026-11-09).

use std::path::Path;

use h3_sim::Replay;

/// The battle's files (`h3-files`): its loaders, the replay file and the commands' text form.
pub use h3_files::{FileError as CliError, *};

pub mod adventure;
pub mod ai;

/// Options for [`run_random`].
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Battle seed.
    pub seed: u64,
    /// Agent seed (random legal agent for both sides).
    pub agent_seed: u64,
    /// Command limit.
    pub max_steps: u32,
    /// Checkpoint interval for the replay.
    pub checkpoint_every: u32,
}

/// Plays a random-vs-random battle on a scenario file and returns the replay (as recorded) and
/// its TOML text.
pub fn run_random(
    scenario_path: &Path,
    packs_dir: &Path,
    options: &RunOptions,
) -> Result<(Replay, String), CliError> {
    let (def, ruleset, setup) = load_scenario(scenario_path, packs_dir, options.seed)?;
    let replay = Replay::record_random(
        &ruleset,
        setup,
        options.seed,
        options.agent_seed,
        options.max_steps,
        options.checkpoint_every,
    )
    .map_err(CliError::Setup)?;
    let text = replay_to_toml(&replay, &def.id, &ruleset)?;
    Ok((replay, text))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use h3_content::ContentKey;

    use super::*;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn run_then_verify_round_trip() {
        let packs = root().join("packs");
        let scenario = packs.join("synthetic/scenarios/skirmish.toml");
        let options = RunOptions {
            seed: 5,
            agent_seed: 6,
            max_steps: 300,
            checkpoint_every: 20,
        };
        let (replay, text) = run_random(&scenario, &packs, &options).unwrap();
        let doc = ReplayDoc::parse(&text).unwrap();
        assert_eq!(doc.scenario(), "skirmish");
        let ruleset = load_ruleset(&packs, &doc.pack_ids(), doc.profile()).unwrap();
        let back = doc.into_replay(&ruleset).unwrap();
        assert_eq!(back, replay);
        assert_eq!(back.verify(&ruleset), Ok(()));
        // The game's difficulty and the human players (format 28, E04c T7d) round-trip; a key
        // left at its default is not written.
        assert!(!text.contains("difficulty") && !text.contains("human"));
        let mut game = replay;
        game.setup.difficulty = 3;
        game.setup.human = [true, false];
        let text = replay_to_toml(&game, "skirmish", &ruleset).unwrap();
        assert!(text.contains("difficulty = 3") && text.contains("human = [\"attacker\"]"));
        let back = ReplayDoc::parse(&text)
            .unwrap()
            .into_replay(&ruleset)
            .unwrap();
        assert_eq!(
            (back.setup.difficulty, back.setup.human),
            (3, [true, false])
        );
        let bad = text.replace("human = [\"attacker\"]", "human = [\"nobody\"]");
        assert!(matches!(
            ReplayDoc::parse(&bad).unwrap().into_replay(&ruleset),
            Err(CliError::Format(_))
        ));
        // The retreat facts and the attacking hero's own ground (format 30, E23 T6) round-trip;
        // none is written while nothing is known.
        assert!(!text.contains("retreat") && !text.contains("attacker_ground"));
        game.setup.retreat[1] = h3_contracts::RetreatFacts {
            experience: 2000,
            artifacts: 0,
            tavern: true,
            wanted: true,
        };
        game.setup.attacker_ground = ContentKey::parse("synthetic:hallow").ok();
        let text = replay_to_toml(&game, "skirmish", &ruleset).unwrap();
        assert!(text.contains("[setup.defender_retreat]") && !text.contains("artifacts"));
        let back = ReplayDoc::parse(&text)
            .unwrap()
            .into_replay(&ruleset)
            .unwrap();
        assert_eq!(
            (back.setup.retreat, back.setup.attacker_ground),
            (game.setup.retreat, game.setup.attacker_ground.clone())
        );
        let bad = text.replace("synthetic:hallow", "hallow");
        assert!(matches!(
            ReplayDoc::parse(&bad).unwrap().into_replay(&ruleset),
            Err(CliError::Format(_))
        ));
        // The viewers' facts (format 31, CLT-Q4, CLT-Q6) round-trip; none is written while not
        // known.
        assert!(!text.contains("background") && !text.contains("colour"));
        game.setup.background = Some(1);
        game.setup.colours = [None, Some(5)];
        let text = replay_to_toml(&game, "skirmish", &ruleset).unwrap();
        assert!(text.contains("defender_colour = 5") && !text.contains("attacker_colour"));
        let back = ReplayDoc::parse(&text)
            .unwrap()
            .into_replay(&ruleset)
            .unwrap();
        assert_eq!(
            (back.setup.background, back.setup.colours),
            (Some(1), [None, Some(5)])
        );
    }
}
