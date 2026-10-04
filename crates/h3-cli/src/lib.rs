//! Library side of the `h3` tool: loading packs and scenarios from disk, running random battles,
//! and the replay file format (TOML). The binary (`main.rs`) only parses arguments.
//!
//! Replay file format v3 (`fixtures/replays/*.toml` are examples):
//!
//! ```toml
//! [replay]
//! format = 3
//! engine = "0.1.0"
//! scenario = "skirmish"
//! seed = "7"                      # u64 as a decimal string
//! packs = [["synthetic", "0.1.0"]]
//! profile = "synthetic:faithful"
//! ruleset_hash = "0x…"
//! rng = "xoshiro256starstar+splitmix64"
//! rng_version = 1
//! hash = "fnv1a64-le"
//! hash_version = 1
//! battle_state_hash_version = 2
//! checkpoint_every = 25
//!
//! [setup]                         # the rest of the setup (fields default when absent)
//! obstacles = [40, 41]            # blocked hexes
//! terrain = "base:grass"          # the battlefield's terrain (omitted when none)
//! attacker_war_machines = ["base:ballista"]  # war machines per side (omitted when none)
//! tight_formation = ["defender"]  # sides in tight formation (omitted when none)
//!
//! [setup.attacker_hero]           # the battle hero per side (format 3; omitted when none):
//! hero_type = "base:orrin"        #   the fields of `h3_pack::HeroEntry`
//! attack = 2
//! defense = 2
//! power = 1
//! knowledge = 1
//! tactics = 0
//! mana = 10
//! max_mana = 10
//!
//! [[attacker]]                    # the setup, by creature key
//! slot = 0
//! creature = "synthetic:spearman"
//! count = 30
//!
//! commands = ["A move 45", "D attack 3 57", "A shoot 4 120", "A wait", "D defend"]
//! checkpoints = ["0x…"]           # state hash after every `checkpoint_every` commands
//!
//! [result]
//! winner = "attacker"             # attacker | defender | draw | none (undecided)
//! final_hash = "0x…"
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

use h3_battle::{Command, Hex, StackId};
use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::{Army, ArmyStack, BattleHero, BattleResult, BattleSetup, BattleSide};
use h3_kernel::{Rng, StreamId};
use h3_pack::{HeroEntry, PackError, ScenarioDef};
use h3_sim::{Replay, ReplayError, ReplayHeader};
use serde::{Deserialize, Serialize};

/// Errors of the CLI library.
#[derive(Debug)]
pub enum CliError {
    /// File system error.
    Io(PathBuf, std::io::Error),
    /// Pack or scenario error.
    Pack(PathBuf, PackError),
    /// The replay file is malformed.
    Format(String),
    /// The replay does not verify.
    Replay(ReplayError),
    /// The setup does not start a battle.
    Setup(h3_battle::SetupError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(p, e) => write!(f, "{}: {e}", p.display()),
            Self::Pack(p, e) => write!(f, "{}: {e}", p.display()),
            Self::Format(e) => write!(f, "replay format: {e}"),
            Self::Replay(e) => write!(f, "{e}"),
            Self::Setup(e) => write!(f, "setup: {e}"),
        }
    }
}

impl std::error::Error for CliError {}

fn read(path: &Path) -> Result<String, CliError> {
    std::fs::read_to_string(path).map_err(|e| CliError::Io(path.to_path_buf(), e))
}

/// Loads `<packs_dir>/<id>/pack.toml` for each id and compiles them with `profile`. Ids come
/// from scenario and replay files, so they must be valid pack ids (no paths).
pub fn load_ruleset(packs_dir: &Path, ids: &[String], profile: &str) -> Result<Ruleset, CliError> {
    let mut texts = Vec::new();
    for id in ids {
        h3_content::ContentKey::new(id, "pack")
            .map_err(|_| CliError::Format(format!("invalid pack id `{id}`")))?;
        let path = packs_dir.join(id).join("pack.toml");
        texts.push(read(&path)?);
    }
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    h3_pack::compile(&refs, profile).map_err(|e| CliError::Pack(packs_dir.to_path_buf(), e))
}

/// Loads a scenario file, its packs and its setup. A scenario with `random_obstacles` gets
/// obstacles generated for its terrain from stream `OBSTACLES` of `seed` (BAT-GEO-004).
pub fn load_scenario(
    path: &Path,
    packs_dir: &Path,
    seed: u64,
) -> Result<(ScenarioDef, Ruleset, BattleSetup), CliError> {
    let def =
        h3_pack::parse_scenario(&read(path)?).map_err(|e| CliError::Pack(path.to_path_buf(), e))?;
    let ruleset = load_ruleset(packs_dir, &def.packs, &def.profile)?;
    let mut setup = def
        .resolve(&ruleset)
        .map_err(|e| CliError::Pack(path.to_path_buf(), e))?;
    if let (true, Some(terrain)) = (def.random_obstacles, setup.terrain) {
        let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
        setup.obstacles = h3_battle::generate_obstacles(&ruleset, terrain, &mut rng);
    }
    Ok((def, ruleset, setup))
}

// ------------------------------------------------------------------ replay file format

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayFile {
    replay: Meta,
    #[serde(default)]
    setup: SetupEntry,
    attacker: Vec<StackEntry>,
    defender: Vec<StackEntry>,
    commands: Vec<String>,
    checkpoints: Vec<String>,
    result: ResultEntry,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Meta {
    format: u32,
    engine: String,
    scenario: String,
    seed: String,
    packs: Vec<[String; 2]>,
    profile: String,
    ruleset_hash: String,
    rng: String,
    rng_version: u32,
    hash: String,
    hash_version: u32,
    battle_state_hash_version: u32,
    checkpoint_every: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupEntry {
    #[serde(default)]
    obstacles: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    terrain: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attacker_war_machines: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    defender_war_machines: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tight_formation: Vec<String>,
    // Tables last: TOML writes values before tables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    attacker_hero: Option<HeroEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    defender_hero: Option<HeroEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StackEntry {
    slot: u8,
    creature: String,
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultEntry {
    winner: String,
    final_hash: String,
}

fn hex64(v: u64) -> String {
    format!("{v:#018x}")
}

fn parse_hex64(s: &str) -> Result<u64, CliError> {
    s.strip_prefix("0x")
        .and_then(|h| u64::from_str_radix(h, 16).ok())
        .ok_or_else(|| CliError::Format(format!("`{s}` is not a 0x-prefixed 64-bit hex value")))
}

fn side_letter(side: BattleSide) -> &'static str {
    match side {
        BattleSide::Attacker => "A",
        BattleSide::Defender => "D",
    }
}

/// Text form of one replay command: `"<A|D> move <hex>"`, `"<A|D> attack <stack> <hex>"`,
/// `"<A|D> wait"`, `"<A|D> defend"`.
pub fn format_command(side: BattleSide, command: Command) -> String {
    let s = side_letter(side);
    match command {
        Command::Move { to } => format!("{s} move {}", to.0),
        Command::Attack { target, from } => format!("{s} attack {} {}", target.0, from.0),
        Command::Shoot { target, at } => format!("{s} shoot {} {}", target.0, at.0),
        Command::Wait => format!("{s} wait"),
        Command::Defend => format!("{s} defend"),
    }
}

/// Parses the text form produced by [`format_command`].
pub fn parse_command(text: &str) -> Result<(BattleSide, Command), CliError> {
    let bad = || CliError::Format(format!("bad command `{text}`"));
    let parts: Vec<&str> = text.split_whitespace().collect();
    let side = match parts.first() {
        Some(&"A") => BattleSide::Attacker,
        Some(&"D") => BattleSide::Defender,
        _ => return Err(bad()),
    };
    let num = |i: usize| {
        parts
            .get(i)
            .and_then(|p| p.parse::<u8>().ok())
            .ok_or_else(bad)
    };
    let command = match (parts.get(1).copied(), parts.len()) {
        (Some("move"), 3) => Command::Move { to: Hex(num(2)?) },
        (Some("attack"), 4) => Command::Attack {
            target: StackId(num(2)?),
            from: Hex(num(3)?),
        },
        (Some("shoot"), 4) => Command::Shoot {
            target: StackId(num(2)?),
            at: Hex(num(3)?),
        },
        (Some("wait"), 2) => Command::Wait,
        (Some("defend"), 2) => Command::Defend,
        _ => return Err(bad()),
    };
    Ok((side, command))
}

/// The `winner` text of a result: the winning side, `draw` or `none` (undecided).
pub fn result_text(r: Option<BattleResult>) -> &'static str {
    match r {
        Some(BattleResult::Victory(BattleSide::Attacker)) => "attacker",
        Some(BattleResult::Victory(BattleSide::Defender)) => "defender",
        Some(BattleResult::Draw) => "draw",
        None => "none",
    }
}

fn parse_result(s: &str) -> Result<Option<BattleResult>, CliError> {
    match s {
        "attacker" => Ok(Some(BattleResult::Victory(BattleSide::Attacker))),
        "defender" => Ok(Some(BattleResult::Victory(BattleSide::Defender))),
        "draw" => Ok(Some(BattleResult::Draw)),
        "none" => Ok(None),
        other => Err(CliError::Format(format!("bad winner `{other}`"))),
    }
}

/// Serialises a replay to TOML.
pub fn replay_to_toml(
    replay: &Replay,
    scenario: &str,
    ruleset: &Ruleset,
) -> Result<String, CliError> {
    let stacks = |army: &Army| -> Vec<StackEntry> {
        army.stacks
            .iter()
            .map(|s| StackEntry {
                slot: s.slot,
                creature: ruleset
                    .creature_key(s.creature)
                    .map(|k| k.as_str().to_owned())
                    .unwrap_or_default(),
                count: s.count,
            })
            .collect()
    };
    let keys = |machines: &[CreatureTypeId]| -> Vec<String> {
        machines
            .iter()
            .map(|&c| {
                ruleset
                    .creature_key(c)
                    .map(|k| k.as_str().to_owned())
                    .unwrap_or_default()
            })
            .collect()
    };
    let hero = |hero: &Option<BattleHero>| -> Result<Option<HeroEntry>, CliError> {
        hero.as_ref()
            .map(|h| HeroEntry::of(h, ruleset))
            .transpose()
            .map_err(|e| CliError::Format(e.to_string()))
    };
    let h = &replay.header;
    let file = ReplayFile {
        replay: Meta {
            format: h.format_version,
            engine: h.engine_version.clone(),
            scenario: scenario.to_owned(),
            seed: replay.seed.to_string(),
            packs: h
                .packs
                .iter()
                .map(|(id, v)| [id.clone(), v.clone()])
                .collect(),
            profile: h.profile.clone(),
            ruleset_hash: hex64(h.ruleset_hash),
            rng: h.rng_algorithm.clone(),
            rng_version: h.rng_version,
            hash: h.hash_algorithm.clone(),
            hash_version: h.hash_version,
            battle_state_hash_version: h.battle_state_hash_version,
            checkpoint_every: replay.checkpoint_every,
        },
        setup: SetupEntry {
            obstacles: replay.setup.obstacles.clone(),
            terrain: replay
                .setup
                .terrain
                .and_then(|t| ruleset.terrains().get(t.index()))
                .map(|t| t.key.as_str().to_owned()),
            attacker_war_machines: keys(&replay.setup.war_machines[0]),
            defender_war_machines: keys(&replay.setup.war_machines[1]),
            tight_formation: h3_pack::tight_sides(replay.setup.formations),
            attacker_hero: hero(&replay.setup.heroes[0])?,
            defender_hero: hero(&replay.setup.heroes[1])?,
        },
        attacker: stacks(&replay.setup.attacker),
        defender: stacks(&replay.setup.defender),
        commands: replay
            .commands
            .iter()
            .map(|(s, c)| format_command(*s, *c))
            .collect(),
        checkpoints: replay.checkpoints.iter().map(|h| hex64(*h)).collect(),
        result: ResultEntry {
            winner: result_text(replay.result).to_owned(),
            final_hash: hex64(replay.final_hash),
        },
    };
    toml::to_string_pretty(&file).map_err(|e| CliError::Format(e.to_string()))
}

/// A replay file parsed from TOML, before its ruleset is loaded.
#[derive(Debug)]
pub struct ReplayDoc {
    file: ReplayFile,
}

impl ReplayDoc {
    /// Parses TOML.
    pub fn parse(text: &str) -> Result<Self, CliError> {
        toml::from_str(text)
            .map(|file| Self { file })
            .map_err(|e| CliError::Format(e.to_string()))
    }

    /// Pack ids the replay needs, in load order.
    pub fn pack_ids(&self) -> Vec<String> {
        self.file
            .replay
            .packs
            .iter()
            .map(|[id, _]| id.clone())
            .collect()
    }

    /// The profile key.
    pub fn profile(&self) -> &str {
        &self.file.replay.profile
    }

    /// The scenario id.
    pub fn scenario(&self) -> &str {
        &self.file.replay.scenario
    }

    /// Converts into a [`Replay`], resolving creature keys against `ruleset`.
    pub fn into_replay(self, ruleset: &Ruleset) -> Result<Replay, CliError> {
        let f = self.file;
        let m = f.replay;
        let army = |entries: &[StackEntry]| -> Result<Army, CliError> {
            let stacks = entries
                .iter()
                .map(|e| {
                    let creature = ruleset.creature_by_key(&e.creature).ok_or_else(|| {
                        CliError::Format(format!("unknown creature `{}`", e.creature))
                    })?;
                    Ok(ArmyStack {
                        slot: e.slot,
                        creature,
                        count: e.count,
                    })
                })
                .collect::<Result<Vec<_>, CliError>>()?;
            Ok(Army { stacks })
        };
        let machines = |keys: &[String]| -> Result<Vec<CreatureTypeId>, CliError> {
            keys.iter()
                .map(|k| {
                    ruleset
                        .creature_by_key(k)
                        .ok_or_else(|| CliError::Format(format!("unknown creature `{k}`")))
                })
                .collect()
        };
        let hero = |entry: Option<&HeroEntry>| -> Result<Option<BattleHero>, CliError> {
            entry
                .map(|e| e.resolve(ruleset))
                .transpose()
                .map_err(|e| CliError::Format(e.to_string()))
        };
        let header = ReplayHeader {
            format_version: m.format,
            engine_version: m.engine,
            packs: m.packs.into_iter().map(|[id, v]| (id, v)).collect(),
            profile: m.profile,
            ruleset_hash: parse_hex64(&m.ruleset_hash)?,
            rng_algorithm: m.rng,
            rng_version: m.rng_version,
            hash_algorithm: m.hash,
            hash_version: m.hash_version,
            battle_state_hash_version: m.battle_state_hash_version,
        };
        Ok(Replay {
            header,
            seed: m
                .seed
                .parse()
                .map_err(|_| CliError::Format(format!("bad seed `{}`", m.seed)))?,
            setup: BattleSetup {
                attacker: army(&f.attacker)?,
                defender: army(&f.defender)?,
                obstacles: f.setup.obstacles,
                heroes: [
                    hero(f.setup.attacker_hero.as_ref())?,
                    hero(f.setup.defender_hero.as_ref())?,
                ],
                terrain: f
                    .setup
                    .terrain
                    .as_ref()
                    .map(|t| {
                        ruleset
                            .terrain_by_key(t)
                            .ok_or_else(|| CliError::Format(format!("unknown terrain `{t}`")))
                    })
                    .transpose()?,
                war_machines: [
                    machines(&f.setup.attacker_war_machines)?,
                    machines(&f.setup.defender_war_machines)?,
                ],
                formations: h3_pack::formations(&f.setup.tight_formation)
                    .map_err(|e| CliError::Format(e.to_string()))?,
            },
            commands: f
                .commands
                .iter()
                .map(|c| parse_command(c))
                .collect::<Result<_, _>>()?,
            checkpoint_every: m.checkpoint_every,
            checkpoints: f
                .checkpoints
                .iter()
                .map(|h| parse_hex64(h))
                .collect::<Result<_, _>>()?,
            final_hash: parse_hex64(&f.result.final_hash)?,
            result: parse_result(&f.result.winner)?,
        })
    }
}

/// Result of verifying a replay file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    /// Scenario id.
    pub scenario: String,
    /// Number of commands.
    pub commands: usize,
    /// Final state hash.
    pub final_hash: u64,
}

/// Loads a replay file, its packs from `packs_dir`, and verifies it.
pub fn verify_file(path: &Path, packs_dir: &Path) -> Result<Verified, CliError> {
    let doc = ReplayDoc::parse(&read(path)?)?;
    let ruleset = load_ruleset(packs_dir, &doc.pack_ids(), doc.profile())?;
    let scenario = doc.scenario().to_owned();
    let replay = doc.into_replay(&ruleset)?;
    replay.verify(&ruleset).map_err(CliError::Replay)?;
    Ok(Verified {
        scenario,
        commands: replay.commands.len(),
        final_hash: replay.final_hash,
    })
}

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
    use super::*;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn commands_round_trip() {
        let cases = [
            (
                BattleSide::Attacker,
                Command::Move { to: Hex(45) },
                "A move 45",
            ),
            (
                BattleSide::Defender,
                Command::Attack {
                    target: StackId(3),
                    from: Hex(57),
                },
                "D attack 3 57",
            ),
            (BattleSide::Attacker, Command::Wait, "A wait"),
            (BattleSide::Defender, Command::Defend, "D defend"),
        ];
        for (side, cmd, text) in cases {
            assert_eq!(format_command(side, cmd), text);
            assert_eq!(parse_command(text).unwrap(), (side, cmd));
        }
        for bad in [
            "",
            "X wait",
            "A jump",
            "A move",
            "A move 300",
            "A wait now",
            "A attack 1",
        ] {
            assert!(parse_command(bad).is_err(), "{bad}");
        }
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
    }

    #[test]
    fn format_errors() {
        assert!(matches!(
            ReplayDoc::parse("x = 1"),
            Err(CliError::Format(_))
        ));
        assert!(parse_hex64("12").is_err());
        assert!(parse_hex64("0xzz").is_err());
        assert_eq!(parse_hex64("0x10").unwrap(), 16);
        assert!(parse_result("nobody").is_err());
        assert_eq!(parse_result("none").unwrap(), None);
        for r in [
            None,
            Some(BattleResult::Draw),
            Some(BattleResult::Victory(BattleSide::Attacker)),
            Some(BattleResult::Victory(BattleSide::Defender)),
        ] {
            assert_eq!(parse_result(result_text(r)).unwrap(), r);
        }
        let missing =
            load_ruleset(Path::new("/nonexistent"), &["x".to_owned()], "x:y").unwrap_err();
        assert!(missing.to_string().contains("pack.toml"));
        for id in ["../synthetic", "/etc", "a/b"] {
            let err = load_ruleset(Path::new("packs"), &[id.to_owned()], "x:y").unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("replay format: invalid pack id `{id}`")
            );
        }
    }
}
