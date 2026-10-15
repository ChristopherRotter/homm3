use h3_battle::{CastTarget, Command, Hex, StackId};
use h3_content::SpellId;

use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const DUEL: &str = "fixtures/maps/e08-duel-36.h3m";

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

#[test]
fn commands_round_trip_through_their_text() {
    let hero = HeroId::new(12);
    let cases = [
        (PlayerId::new(0), GameCommand::EndTurn, "0 end-turn"),
        (
            PlayerId::new(2),
            GameCommand::Answer(Answer::Accept),
            "2 accept",
        ),
        (
            PlayerId::new(3),
            GameCommand::Answer(Answer::Decline),
            "3 decline",
        ),
        (
            PlayerId::new(7),
            GameCommand::Adventure(AdventureCommand::MoveHero {
                hero,
                path: vec![Pos::new(3, 4, 0), Pos::new(4, 5, 1)],
            }),
            "7 move 12 3,4,0 4,5,1",
        ),
        (
            PlayerId::new(1),
            GameCommand::Adventure(AdventureCommand::MoveHero {
                hero,
                path: Vec::new(),
            }),
            "1 move 12",
        ),
        // Format v3 (E08 T19b1): a battle command, the battle replays' text without the side.
        (
            PlayerId::new(1),
            GameCommand::Battle(Command::Attack {
                target: StackId(3),
                from: Hex(41),
            }),
            "1 battle attack 3 41",
        ),
        (
            PlayerId::new(0),
            GameCommand::Battle(Command::Cast {
                spell: SpellId::from_raw(15),
                target: CastTarget::Stack(StackId(2)),
                second: CastTarget::Hex(Hex(30)),
            }),
            "0 battle cast 15 stack 2 hex 30",
        ),
        (
            PlayerId::new(0),
            GameCommand::Battle(Command::Wait),
            "0 battle wait",
        ),
    ];
    for (player, command, text) in cases {
        let actor = Actor::Player(player);
        assert_eq!(format_command(actor, &command), text);
        assert_eq!(parse_command(text).unwrap(), (actor, command));
    }
    // Format v4 (E08 T20b2): the system actor, a monster's side's commands.
    let wait = GameCommand::Battle(Command::Wait);
    assert_eq!(format_command(Actor::System, &wait), "system battle wait");
    assert_eq!(
        parse_command("system battle wait").unwrap(),
        (Actor::System, wait)
    );
    for bad in [
        "",
        "x end-turn",
        "256 end-turn",
        "0 jump",
        "0",
        "0 end-turn extra",
        "0 accept now",
        "0 move",
        "0 move h1",
        "0 move 1 3,4",
        "0 move 1 3,4,0,1",
        "0 move 1 3,4,x",
        "0 move 1 70000,4,0",
        "0 battle",
        "0 battle fly 3",
        "0 battle A wait",
        "0 battle wait now",
        "system",
        "systems battle wait",
    ] {
        assert!(
            matches!(parse_command(bad), Err(AdventureCliError::Format(_))),
            "{bad}"
        );
    }
}

#[test]
fn map_paths_stay_below_the_root() {
    for ok in [DUEL, "./a.h3m", "maps/x/y.h3m"] {
        assert!(check_map_path(ok).is_ok(), "{ok}");
    }
    for bad in ["", "../x.h3m", "maps/../../x.h3m", "/etc/passwd"] {
        assert!(check_map_path(bad).is_err(), "{bad}");
    }
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
    assert!(text.contains("format = 4\n"), "{text}");
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
        .replacen("format = 4", "format = 1", 1)
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
    assert!(e.to_string().contains("format 4 needs a difficulty"), "{e}");
    std::fs::remove_dir_all(&tmp).unwrap();
    // Answers round-trip through the file.
    let mut answered = replay.clone();
    answered.commands.push((
        Actor::Player(PlayerId::new(1)),
        GameCommand::Answer(Answer::Accept),
    ));
    answered.commands.push((
        Actor::Player(PlayerId::new(0)),
        GameCommand::Answer(Answer::Decline),
    ));
    let text = replay_to_toml(&answered).unwrap();
    assert!(text.contains("\"1 accept\""), "{text}");
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
    let fork = bench(bench::Workload::Fork, false, false);
    assert_eq!((fork.operations, fork.steps), (14, play.steps));
    assert_eq!(fork.battles, 0);
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
        },
    )
    .unwrap();
    assert_eq!((fight.operations, fight.battles), (6, 2));
    assert!(fight.steps > 2, "{fight:?}");
    let dir = root().display().to_string();
    let packs = format!("{dir}/packs");
    let args = |words: &[&str]| words.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
    for (workload, agent) in [("play", "0"), ("play", "1"), ("reach", "1"), ("fork", "1")] {
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
