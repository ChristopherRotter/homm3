use std::path::PathBuf;

use h3_adventure::Pos;
use h3_content::CreatureTypeId;
use h3_contracts::{HeroId, PlayerId};
use h3_game::{Answer, Consent, HireAnswer, UniversityAnswer};
use h3_sim::adventure::replay::{RecordError, ReplayError};

use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const DUEL: &str = "fixtures/maps/e08-duel-36.h3m";
/// E08 T52's map: a meeting and an army dialog one step away.
const ARMY: &str = "fixtures/maps/e08-army-12.h3m";
/// The rewards map (E08 T64a): every object class of increment 7.
const REWARDS: &str = "fixtures/maps/e08-rewards-36.h3m";
/// Decision 133's maps (E08 T64b): 144 × 144 with an underground, a Pandora's Box or a Learning
/// Stone on every free tile.
const FULL: [&str; 2] = [
    "fixtures/maps/e08-boxes-144.h3m",
    "fixtures/maps/e08-stones-144.h3m",
];
/// The sea XL map (E08 T75b): 144 × 144 of sea, sixteen islands, fifteen boats.
const SEA_XL: &str = "fixtures/maps/e08-sea-144.h3m";
/// The economy map (E08 T86a) and the economy XL map (E08 T86b2): increment 9's objects.
const ECONOMY: &str = "fixtures/maps/e08-economy-36.h3m";
const ECONOMY_XL: &str = "fixtures/maps/e08-economy-144.h3m";
/// The dwellings XL map (E08 T96b2): increment 10's dwellings, random dwellings and camps.
const DWELLINGS_XL: &str = "fixtures/maps/e08-dwellings-144.h3m";
/// The town map (E09 T9a): eight towns of every kind the import reads.
const TOWNS: &str = "fixtures/maps/e09-towns-36.h3m";
/// The gates XL map (E08 T104): a Subterranean Gate on every fifth tile of both levels.
const GATES_XL: &str = "fixtures/maps/e08-gates-144.h3m";

fn options(days: u32) -> RunOptions {
    RunOptions {
        map: DUEL.to_owned(),
        profile: DEFAULT_PROFILE.to_owned(),
        difficulty: h3_game::DEFAULT_DIFFICULTY,
        record: RecordOptions {
            seed: 4,
            agent_seed: 6,
            max_days: days,
            max_steps: 0,
            checkpoint_every: 5,
        },
    }
}

/// The command of `answer`.
const fn answer(answer: Answer) -> GameCommand {
    GameCommand::Answer(answer)
}

#[test]
fn a_recorded_game_round_trips_through_its_file_and_verifies() {
    let (replay, text) = run_random(&root().join("packs"), &root(), &options(2)).unwrap();
    assert_eq!(replay.final_day, 3);
    assert!(replay.commands.len() > 4);
    assert_eq!(parse_replay(&text).unwrap(), replay);
    let dir = std::env::temp_dir().join(format!("h3-adventure-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("r.toml");
    std::fs::write(&file, &text).unwrap();
    let verified = verify_file(&file, &root().join("packs"), &root()).unwrap();
    assert_eq!(
        verified,
        Verified {
            map: DUEL.to_owned(),
            commands: replay.commands.len(),
            day: 3,
            final_hash: replay.final_hash,
        }
    );
    // Tampering is caught: a command, an unknown key, a hash.
    let tampered = text.replacen("end-turn", "move 1 0,0,0", 1);
    std::fs::write(&file, &tampered).unwrap();
    let e = verify_file(&file, &root().join("packs"), &root()).unwrap_err();
    assert!(matches!(e, AdventureCliError::Replay(_)), "{e}");
    std::fs::write(&file, format!("{text}\n[extra]\nx = 1\n")).unwrap();
    let e = verify_file(&file, &root().join("packs"), &root()).unwrap_err();
    assert!(matches!(e, AdventureCliError::Format(_)), "{e}");
    let e = parse_replay(&text.replacen("final_hash = \"0x", "final_hash = \"0y", 1)).unwrap_err();
    assert!(e.to_string().contains("bad hash"), "{e}");
    let e = parse_replay(&text.replacen("seed = \"4\"", "seed = \"-4\"", 1)).unwrap_err();
    assert!(e.to_string().contains("bad seed"), "{e}");
    // The map is not where the replay says, or is another file.
    std::fs::write(&file, &text).unwrap();
    let e = verify_file(&file, &root().join("packs"), &dir).unwrap_err();
    assert!(matches!(e, AdventureCliError::Io(..)), "{e}");
    std::fs::create_dir_all(dir.join("fixtures/maps")).unwrap();
    std::fs::write(dir.join(DUEL), b"not a map").unwrap();
    // Another file is refused by its hash before it is parsed; read for a run, it does not parse.
    let e = verify_file(&file, &root().join("packs"), &dir).unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Replay(ReplayError::Incompatible("map.hash"))
        ),
        "{e}"
    );
    let e = run_random(&root().join("packs"), &dir, &options(1)).unwrap_err();
    assert!(matches!(e, AdventureCliError::Map(..)), "{e}");
    // A file over the reader's limit is refused before it is read (a sparse file).
    let big = std::fs::File::create(dir.join(DUEL)).unwrap();
    let max = h3_map::Limits::default().max_file_bytes;
    big.set_len(u64::try_from(max).unwrap() + 1).unwrap();
    let e = verify_file(&file, &root().join("packs"), &dir).unwrap_err();
    assert!(e.to_string().contains("over the limit"), "{e}");
    // A link below the root that leads outside it is refused.
    #[cfg(unix)]
    {
        std::fs::remove_file(dir.join(DUEL)).unwrap();
        std::os::unix::fs::symlink(root().join(DUEL), dir.join(DUEL)).unwrap();
        let e = verify_file(&file, &root().join("packs"), &dir).unwrap_err();
        assert!(e.to_string().contains("leads outside"), "{e}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Since format v2: the game's difficulty in the header, verified at it; answers round-trip through
/// the file; a v1 file (no difficulty) reads as normal and is refused by its version; a difficulty
/// the rules do not have is the setup's error.
#[test]
fn the_difficulty_and_the_answers_round_trip_through_the_file() {
    let (packs, dir) = (root().join("packs"), root());
    let easy = RunOptions {
        difficulty: 0,
        ..options(1)
    };
    let (replay, text) = run_random(&packs, &dir, &easy).unwrap();
    assert_eq!(replay.difficulty, 0);
    assert!(text.contains("format = 10\n"), "{text}");
    assert!(text.contains("difficulty = 0\n"), "{text}");
    let (normal, _) = run_random(&packs, &dir, &options(1)).unwrap();
    assert_eq!(normal.difficulty, h3_game::DEFAULT_DIFFICULTY);
    assert_eq!(normal.commands, replay.commands);
    assert_ne!(normal.final_hash, replay.final_hash);
    let tmp = std::env::temp_dir().join(format!("h3-adventure-v2-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("r.toml");
    let verify = |text: &str| {
        std::fs::write(&file, text).unwrap();
        verify_file(&file, &packs, &dir)
    };
    assert_eq!(verify(&text).unwrap().final_hash, replay.final_hash);
    // At another difficulty the recorded hashes are another game's.
    let e = verify(&text.replacen("difficulty = 0", "difficulty = 1", 1)).unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Replay(ReplayError::HashMismatch { .. })
        ),
        "{e}"
    );
    let e = verify(&text.replacen("difficulty = 0", "difficulty = 9", 1)).unwrap_err();
    assert!(e.to_string().contains("no difficulty 9"), "{e}");
    let e = verify(&text.replacen("difficulty = 0", "difficulty = 300", 1)).unwrap_err();
    assert!(matches!(e, AdventureCliError::Format(_)), "{e}");
    // A v1 file: no difficulty, refused by its version before its map is read.
    let v1 = text
        .replacen("format = 10", "format = 1", 1)
        .replacen("difficulty = 0\n", "", 1);
    assert_eq!(
        parse_replay(&v1).unwrap().difficulty,
        h3_game::DEFAULT_DIFFICULTY
    );
    let e = verify(&v1.replacen("e08-duel-36", "nosuch", 1)).unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Replay(ReplayError::Incompatible("format_version"))
        ),
        "{e}"
    );
    // A later file without its difficulty is malformed, not a game at normal difficulty.
    let e = verify(&text.replacen("difficulty = 0\n", "", 1)).unwrap_err();
    assert!(matches!(e, AdventureCliError::Format(_)), "{e}");
    assert!(
        e.to_string().contains("format 10 needs a difficulty"),
        "{e}"
    );
    // A v8 file has no town part's hash (format v9, E09 T5a2): it reads, and its version refuses
    // it; a v9 file without it is refused by the hash.
    let line = text
        .lines()
        .find(|l| l.starts_with("town_ruleset_hash = \"0x"))
        .unwrap();
    let townless = text.replacen(&format!("{line}\n"), "", 1);
    assert_eq!(parse_replay(&townless).unwrap().header.town_ruleset_hash, 0);
    let e = verify(&townless.replacen("format = 10", "format = 8", 1)).unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Replay(ReplayError::Incompatible("format_version"))
        ),
        "{e}"
    );
    let e = verify(&townless).unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Replay(ReplayError::Incompatible("town_ruleset_hash"))
        ),
        "{e}"
    );
    std::fs::remove_dir_all(&tmp).unwrap();
    // Answers round-trip through the file.
    let mut answered = replay.clone();
    answered.commands.push((
        Actor::Player(PlayerId::new(1)),
        answer(Answer::Offer(Consent::Accept)),
    ));
    answered.commands.push((
        Actor::Player(PlayerId::new(0)),
        answer(Answer::University(UniversityAnswer::Buy(2))),
    ));
    answered.commands.push((
        Actor::Player(PlayerId::new(0)),
        answer(Answer::Hire(HireAnswer::Buy {
            creature: CreatureTypeId::from_raw(3),
            amount: 7,
        })),
    ));
    answered.commands.push((
        Actor::Player(PlayerId::new(0)),
        answer(Answer::Hire(HireAnswer::Close)),
    ));
    let text = replay_to_toml(&answered).unwrap();
    assert!(text.contains("\"0 answer hire buy 3 7\""), "{text}");
    assert!(text.contains("\"0 answer hire close\""), "{text}");
    assert!(text.contains("\"1 answer offer accept\""), "{text}");
    assert_eq!(parse_replay(&text).unwrap(), answered);
    // A difficulty the rules do not have is the setup's error.
    let e = run_random(
        &packs,
        &dir,
        &RunOptions {
            difficulty: 5,
            ..options(1)
        },
    )
    .unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Record(RecordError::Setup(h3_game::GameSetupError::Difficulty(5)))
        ),
        "{e}"
    );
}

#[test]
fn rules_load_from_the_pack_folders_as_the_embedded_packs_compile() {
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(&root().join("packs"), &ids, DEFAULT_PROFILE).unwrap();
    let (core, part) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, DEFAULT_PROFILE).unwrap();
    assert_eq!(rules.core().content_hash(), core.content_hash());
    assert_eq!(rules.adventure().content_hash(), part.content_hash());
    // The town part from each folder's `towns.toml` (E09 T5a2): the nine factions' records.
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS, DEFAULT_PROFILE).unwrap();
    assert_eq!(rules.towns().content_hash(), towns.content_hash());
    assert_eq!(rules.towns().towns().len(), 9);
    // Pack ids are not paths; a missing pack or profile is refused.
    let e = load_rules(
        &root().join("packs"),
        &["../base".to_owned()],
        DEFAULT_PROFILE,
    )
    .unwrap_err();
    assert!(matches!(e, AdventureCliError::Format(_)), "{e}");
    let e = load_rules(
        &root().join("packs"),
        &["nosuch".to_owned()],
        DEFAULT_PROFILE,
    )
    .unwrap_err();
    assert!(matches!(e, AdventureCliError::Io(..)), "{e}");
    let e = load_rules(&root().join("packs"), &ids, "base:nosuch").unwrap_err();
    assert!(matches!(e, AdventureCliError::Pack(..)), "{e}");
}

#[test]
fn the_command_line_runs_and_verifies() {
    let dir = std::env::temp_dir().join(format!("h3-adventure-main-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("r.toml").display().to_string();
    let root = root().display().to_string();
    let packs = format!("{root}/packs");
    let args = |words: &[&str]| words.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
    cli::main(&args(&[
        "run",
        "--map",
        DUEL,
        "--root",
        &root,
        "--packs-dir",
        &packs,
        "--days",
        "1",
        "--difficulty",
        "0",
        "--record",
        &out,
    ]))
    .unwrap();
    assert!(
        std::fs::read_to_string(&out)
            .unwrap()
            .contains("difficulty = 0\n")
    );
    // Without `--record` the game is played and nothing is written.
    cli::main(&args(&[
        "run",
        "--map",
        DUEL,
        "--root",
        &root,
        "--packs-dir",
        &packs,
        "--max-steps",
        "3",
    ]))
    .unwrap();
    cli::main(&args(&[
        "verify",
        "--root",
        &root,
        "--packs-dir",
        &packs,
        &out,
    ]))
    .unwrap();
    for bad in [
        args(&[]),
        args(&["fly"]),
        args(&["run"]),
        args(&["run", "--map"]),
        args(&["run", "--map", DUEL, "--days", "0"]),
        args(&["run", "--map", DUEL, "--days", "x"]),
        args(&["run", "--map", DUEL, "--difficulty", "x"]),
        args(&["run", "--map", DUEL, "--difficulty", "256"]),
        args(&[
            "run",
            "--map",
            DUEL,
            "--root",
            &root,
            "--packs-dir",
            &packs,
            "--difficulty",
            "5",
        ]),
        args(&["run", "--map", DUEL, "--speed", "1"]),
        args(&[
            "run",
            "--map",
            "../x.h3m",
            "--root",
            &root,
            "--packs-dir",
            &packs,
        ]),
        args(&["verify"]),
        args(&[
            "verify",
            "--root",
            &root,
            "--packs-dir",
            &packs,
            "nosuch.toml",
        ]),
    ] {
        assert!(cli::main(&bad).is_err(), "{bad:?}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// `run --script` (E08 T52): a text script's commands, blank lines and comments skipped, record a
/// replay that verifies and that a replay file's commands re-record identically; a rejected
/// command records nothing; the random agents' options and a malformed line are refused.
#[test]
fn a_script_records_a_replay_that_verifies() {
    let dir = std::env::temp_dir().join(format!("h3-adventure-script-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (root, packs) = (root(), root().join("packs"));
    let script = dir.join("meeting.txt");
    let text = "# Valeska meets Caitlin.\n0 move 0 4,5,0\n\n0 army move 0:6 2:2\n0 answer close\n0 end-turn\n";
    std::fs::write(&script, text).unwrap();
    let options = ScriptOptions {
        map: ARMY.to_owned(),
        profile: DEFAULT_PROFILE.to_owned(),
        difficulty: h3_game::DEFAULT_DIFFICULTY,
        seed: 3,
        checkpoint_every: 2,
    };
    let commands = read_script(&script).unwrap();
    assert_eq!(commands.len(), 4);
    let (replay, toml) = run_script(&packs, &root, &options, commands).unwrap();
    assert_eq!((replay.commands.len(), replay.checkpoints.len()), (4, 2));
    let out = dir.join("meeting.toml");
    std::fs::write(&out, &toml).unwrap();
    assert!(verify_file(&out, &packs, &root).is_ok());
    let again = run_script(&packs, &root, &options, read_script(&out).unwrap()).unwrap();
    assert_eq!(again.1, toml);
    // The second command again, the meeting closed: no army of another hero is together.
    let mut rejected = read_script(&script).unwrap();
    rejected.push(rejected[1].clone());
    let e = run_script(&packs, &root, &options, rejected).unwrap_err();
    assert!(
        matches!(
            e,
            AdventureCliError::Record(RecordError::Rejected { step: 4, .. })
        ),
        "{e}"
    );
    assert!(e.to_string().contains("scripted command 4"), "{e}");
    let bad = dir.join("bad.txt");
    std::fs::write(&bad, "0 fly 3\n").unwrap();
    assert!(read_script(&bad).is_err());
    let args = |words: &[&str]| words.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
    let (root, packs, script) = (
        root.display().to_string(),
        packs.display().to_string(),
        script.display().to_string(),
    );
    let run = |extra: &[&str]| {
        let mut words = vec![
            "run",
            "--map",
            ARMY,
            "--root",
            &root,
            "--packs-dir",
            &packs,
            "--script",
            &script,
        ];
        words.extend(extra);
        cli::main(&args(&words))
    };
    run(&[]).unwrap();
    for extra in [["--days", "1"], ["--agent-seed", "2"], ["--max-steps", "9"]] {
        assert!(run(&extra).is_err(), "{extra:?}");
    }
    assert!(run(&["--seed", "x"]).is_err());
    let missing = dir.join("nosuch.txt").display().to_string();
    assert!(cli::main(&args(&["run", "--map", ARMY, "--script", &missing])).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The env plays on through a meeting and an army dialog (E08 T52, increment 6's exit criterion
/// 6; the env is the AI lane's, ADR-0017 point 8): on the army map red's first hero steps east
/// onto its mate (a meeting) or west onto the goblins and accepts their join without room (the
/// army dialog); the env's random agent then plays every seat on to the day limit, its first
/// action the only one listed, `Decline`, which closes either.
#[test]
fn the_env_plays_on_through_a_meeting_and_an_army_dialog() {
    use h3_sim::adventure::action::{hero_in_slot, tile_index};
    use h3_sim::adventure::{Action, AdventureEnv, random_rollout};
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(&root().join("packs"), &ids, DEFAULT_PROFILE).unwrap();
    let (_, setup) = load_map(&root(), ARMY, &rules).unwrap();
    let red = PlayerId::new(0);
    for (x, answer) in [(4, None), (2, Some(Action::accept()))] {
        let mut env = AdventureEnv::new(rules.clone(), Arc::new(setup.clone()), 1, 3).unwrap();
        assert_eq!(hero_in_slot(env.state(), red, 0), Some(HeroId::new(0)));
        let tile = tile_index(env.state(), Pos::new(x, 5, 0)).unwrap();
        env.step(red, Action::move_to(0, tile)).unwrap();
        if let Some(answer) = answer {
            env.step(red, answer).unwrap();
        }
        let state = env.state();
        let open = (
            state.meeting(red).is_some(),
            state.army_dialog(red).is_some(),
        );
        assert_eq!(open, (answer.is_none(), answer.is_some()));
        let played = random_rollout(&mut env, 5, 0);
        assert!(env.is_done() && played.day == 4, "{played:?}");
        assert!(env.state().meeting(red).is_none() && env.state().army_dialog(red).is_none());
    }
}

#[test]
fn the_benchmark_workloads_run_and_count_what_they_say() {
    let bench = |workload, observe, masks| {
        bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: DUEL.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload,
                games: 2,
                days: 2,
                seed: 1,
                forks: 7,
                observe,
                masks,
                battle: false,
                meeting: false,
                dwelling: false,
                queries: 0,
            },
        )
        .unwrap()
    };
    let play = bench(bench::Workload::Play, false, false);
    assert_eq!(play.operations, play.steps);
    assert!(play.steps > 4);
    // What an agent reads does not change the game: the same steps.
    assert_eq!(bench(bench::Workload::Play, true, true).steps, play.steps);
    let reach = bench(bench::Workload::Reach, false, false);
    assert_eq!(reach.steps, play.steps);
    // The duel's players start with two heroes each: two queries a step at most, fewer once a
    // hero has left the map or while the neutral seat plays a monster's or guards' side (E08
    // T20b3b2: the duel's monsters and guarded piles).
    assert!(
        (reach.steps..=2 * reach.steps).contains(&reach.operations),
        "{reach:?}"
    );
    // E08 T42: the duel's red starts with a flyer and a water walker, blue with two walkers
    // (`h3-map`'s `fixtures.rs` kits).
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(&root().join("packs"), &ids, DEFAULT_PROFILE).unwrap();
    let (_, setup) = load_map(&root(), DUEL, &rules).unwrap();
    let env = h3_sim::adventure::AdventureEnv::new(rules, Arc::new(setup), 1, 2).unwrap();
    let both: [(u8, &[u16], &[u16]); 2] = [(0, &[0, 2], &[]), (1, &[], &[1, 3])];
    for (player, lifted, walkers) in both {
        let player = PlayerId::new(player);
        let ids = |ids: &[u16]| ids.iter().map(|&id| HeroId::new(id)).collect::<Vec<_>>();
        let (sailing, up, down) = bench::heroes_by_lift(&env, player);
        assert_eq!((&up, &down), (&ids(lifted), &ids(walkers)), "{player:?}");
        assert!(sailing.is_empty());
        assert_eq!(bench::reach_queries(&env, player, &down), down.len() as u64);
        assert_eq!(bench::lifted_queries(&env, player, &up), up.len() as u64);
    }
    let fork = bench(bench::Workload::Fork, false, false);
    assert_eq!((fork.operations, fork.steps), (14, play.steps));
    assert_eq!((fork.battles, fork.visits), (0, 0));
    // E08 T64a: on the rewards map the game's visits write the visits block the forks carry (as
    // in `fork-rewards-36`'s game).
    let rewards = bench::run(
        &root().join("packs"),
        &root(),
        &bench::BenchOptions {
            map: REWARDS.to_owned(),
            profile: DEFAULT_PROFILE.to_owned(),
            difficulty: h3_game::DEFAULT_DIFFICULTY,
            workload: bench::Workload::Fork,
            games: 1,
            days: 7,
            seed: 1,
            forks: 3,
            observe: false,
            masks: false,
            battle: false,
            meeting: false,
            dwelling: false,
            queries: 0,
        },
    )
    .unwrap();
    assert_eq!((rewards.operations, rewards.visits), (3, 1), "{rewards:?}");
    // E08 T64b: on decision 133's maps (`fork-boxes-xl-144`, `fork-stones-xl-144`) the boxes
    // opened leave the map (no visits block), the stones entered mark their heroes (the block).
    for (map, block) in FULL.into_iter().zip([0, 1]) {
        let full = bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: map.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload: bench::Workload::Fork,
                games: 1,
                days: 7,
                seed: 1,
                forks: 3,
                observe: false,
                masks: false,
                battle: false,
                meeting: false,
                dwelling: false,
                queries: 0,
            },
        )
        .unwrap();
        assert_eq!(
            (full.operations, full.visits),
            (3, block),
            "{map}: {full:?}"
        );
    }
    // E08 T75b: on the sea XL map the heroes board (`reach-sea-xl-144`'s queries of heroes in
    // boats, `sail_queries`), and the forks carry the boats with a hero aboard
    // (`fork-sea-xl-144`).
    let sea = |workload, days| {
        bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: SEA_XL.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload,
                games: 1,
                days,
                seed: 1,
                forks: 3,
                observe: false,
                masks: false,
                battle: false,
                meeting: false,
                dwelling: false,
                queries: 0,
            },
        )
        .unwrap()
    };
    let fork = sea(bench::Workload::Fork, 7);
    assert_eq!(fork.operations, 3);
    assert!(fork.afloat > 0, "{fork:?}");
    let reach = sea(bench::Workload::Reach, 7);
    // Each player starts with four heroes: four queries a step at most.
    assert!(reach.operations <= 4 * reach.steps, "{reach:?}");
    let ids: Vec<String> = DEFAULT_PACKS.iter().map(|&s| s.to_owned()).collect();
    let rules = load_rules(&root().join("packs"), &ids, DEFAULT_PROFILE).unwrap();
    let (_, setup) = load_map(&root(), SEA_XL, &rules).unwrap();
    let mut env = h3_sim::adventure::AdventureEnv::new(rules, Arc::new(setup), 1, 7).unwrap();
    let _ = h3_sim::adventure::random_rollout(&mut env, 0, 0);
    let sailing: Vec<HeroId> = (0..2)
        .flat_map(|p| bench::heroes_by_lift(&env, PlayerId::new(p)).0)
        .collect();
    assert!(!sailing.is_empty());
    let red = PlayerId::new(0);
    let own = bench::heroes_by_lift(&env, red).0;
    assert_eq!(bench::sail_queries(&env, red, &own), own.len() as u64);
    // E08 T19b1: with `battle` the heroes seek each other and the forks carry the open battle.
    let fight = bench::run(
        &root().join("packs"),
        &root(),
        &bench::BenchOptions {
            map: DUEL.to_owned(),
            profile: DEFAULT_PROFILE.to_owned(),
            difficulty: h3_game::DEFAULT_DIFFICULTY,
            workload: bench::Workload::Fork,
            games: 2,
            days: 7,
            seed: 1,
            forks: 3,
            observe: false,
            masks: false,
            battle: true,
            meeting: false,
            dwelling: false,
            queries: 0,
        },
    )
    .unwrap();
    assert_eq!((fight.operations, fight.battles), (6, 2));
    assert!(fight.steps > 2, "{fight:?}");
    assert_eq!(fight.meetings, 0);
    // E08 T52: with `meeting` a player's heroes seek each other and the forks carry the open
    // meeting; `manage` lists both heroes' management in it, two listings a time. On the army map
    // red's Valeska meets Caitlin at her first step; on the duel the heroes stand on two levels and
    // never meet.
    let meeting = |map: &str, workload, games| {
        bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: map.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload,
                games,
                days: 2,
                seed: 1,
                forks: 3,
                observe: false,
                masks: false,
                battle: false,
                meeting: true,
                dwelling: false,
                queries: 5,
            },
        )
        .unwrap()
    };
    let met = meeting(ARMY, bench::Workload::Fork, 2);
    assert_eq!((met.operations, met.steps, met.meetings), (6, 2, 2));
    let listed = meeting(ARMY, bench::Workload::Manage, 2);
    assert_eq!(
        (listed.operations, listed.steps, listed.meetings),
        (20, 2, 2)
    );
    let apart = meeting(DUEL, bench::Workload::Manage, 1);
    assert_eq!((apart.operations, apart.meetings), (0, 0));
    assert!(apart.steps > 2, "{apart:?}");
    // E08 T86b2: `day-start` counts one step a game, the one that starts the day after the limit:
    // on the economy XL map day 8, a week's start (`day-start-xl-144`); on the economy map day 29,
    // a creature month's start for the seed 3 (as `month-start-xl-144`'s on the XL map).
    let day_start = |map: &str, days, seed, games| {
        bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: map.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload: bench::Workload::DayStart,
                games,
                days,
                seed,
                forks: 0,
                observe: false,
                masks: false,
                battle: false,
                meeting: false,
                dwelling: false,
                queries: 0,
            },
        )
        .unwrap()
    };
    let week = day_start(ECONOMY_XL, 7, 1, 2);
    assert_eq!(
        (week.operations, week.creature_months, week.added),
        (2, 0, 0)
    );
    assert!(week.steps > 2 * 14, "{week:?}");
    let month = day_start(ECONOMY, 28, 3, 1);
    assert_eq!(
        (month.operations, month.creature_months),
        (1, 1),
        "{month:?}"
    );
    assert!(month.added > 0, "{month:?}");
    // E08 T96b2: on the dwellings XL random play seldom reaches a dwelling and the step that
    // starts day 8 refills them all, so its forks carry no dwellings' week block; with `dwelling`
    // the heroes seek the dwellings until a visit writes one, which the forks then carry
    // (`fork-dwellings-xl-144`).
    let fork = |dwelling| {
        bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: DWELLINGS_XL.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload: bench::Workload::Fork,
                games: 1,
                days: 7,
                seed: 1,
                forks: 3,
                observe: false,
                masks: false,
                battle: false,
                meeting: false,
                dwelling,
                queries: 0,
            },
        )
        .unwrap()
    };
    let random = fork(false);
    assert_eq!((random.operations, random.dwellings), (3, 0), "{random:?}");
    let sought = fork(true);
    assert_eq!((sought.operations, sought.dwellings), (3, 1), "{sought:?}");
    assert!(sought.steps < random.steps, "{sought:?}");
    // E09 T9c: `siege` counts one step a game, the one that opens a siege: on the town map the
    // heroes seek the other owners' towns and besiege one on day 1, a garrison alone or the
    // Tower's visitor with the garrison merged (`siege-towns-36`); the duel has no towns, so its
    // games end at the day limit without one.
    let siege = |map: &str, games| {
        bench::run(
            &root().join("packs"),
            &root(),
            &bench::BenchOptions {
                map: map.to_owned(),
                profile: DEFAULT_PROFILE.to_owned(),
                difficulty: h3_game::DEFAULT_DIFFICULTY,
                workload: bench::Workload::Siege,
                games,
                days: 2,
                seed: 1,
                forks: 0,
                observe: false,
                masks: false,
                battle: false,
                meeting: false,
                dwelling: false,
                queries: 0,
            },
        )
        .unwrap()
    };
    let towns = siege(TOWNS, 8);
    assert_eq!((towns.operations, towns.visitors), (8, 3), "{towns:?}");
    assert!(towns.steps >= 8, "{towns:?}");
    let none = siege(DUEL, 1);
    assert_eq!((none.operations, none.visitors), (0, 0), "{none:?}");
    assert!(none.steps >= 4, "{none:?}");
    // E08 T104: `build` counts the game's builds, without play (`build-gates-xl-144` on the gates
    // XL map).
    let built = bench::run(
        &root().join("packs"),
        &root(),
        &bench::BenchOptions {
            map: GATES_XL.to_owned(),
            profile: DEFAULT_PROFILE.to_owned(),
            difficulty: h3_game::DEFAULT_DIFFICULTY,
            workload: bench::Workload::Build,
            games: 2,
            days: 1,
            seed: 1,
            forks: 0,
            observe: false,
            masks: false,
            battle: false,
            meeting: false,
            dwelling: false,
            queries: 0,
        },
    )
    .unwrap();
    assert_eq!((built.operations, built.steps), (2, 0), "{built:?}");
    let dir = root().display().to_string();
    let packs = format!("{dir}/packs");
    let args = |words: &[&str]| words.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
    for (workload, agent) in [
        ("play", "0"),
        ("play", "1"),
        ("reach", "1"),
        ("fork", "1"),
        ("manage", "1"),
        ("day-start", "1"),
        ("siege", "1"),
        ("build", "1"),
    ] {
        cli::main(&args(&[
            "bench",
            "--map",
            DUEL,
            "--root",
            &dir,
            "--packs-dir",
            &packs,
            "--workload",
            workload,
            "--games",
            "1",
            "--days",
            "1",
            "--forks",
            "3",
            "--observe",
            agent,
            "--masks",
            agent,
            "--battle",
            agent,
            "--meeting",
            agent,
            "--dwelling",
            agent,
            "--queries",
            "2",
            "--difficulty",
            "4",
        ]))
        .unwrap();
    }
    for bad in [
        args(&["bench"]),
        args(&["bench", "--map", DUEL, "--workload", "walk"]),
        args(&["bench", "--map", DUEL, "--days", "0"]),
        args(&["bench", "--map", DUEL, "--difficulty", "x"]),
        args(&[
            "bench",
            "--map",
            DUEL,
            "--root",
            &dir,
            "--packs-dir",
            &packs,
            "--difficulty",
            "5",
        ]),
        args(&["bench", "--map", DUEL, "--speed", "1"]),
        // E08 T104: a build the setup refuses.
        args(&[
            "bench",
            "--map",
            DUEL,
            "--root",
            &dir,
            "--packs-dir",
            &packs,
            "--workload",
            "build",
            "--difficulty",
            "5",
        ]),
        args(&[
            "bench",
            "--map",
            "../x.h3m",
            "--root",
            &dir,
            "--packs-dir",
            &packs,
        ]),
    ] {
        assert!(cli::main(&bad).is_err(), "{bad:?}");
    }
}
