use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const DUEL: &str = "fixtures/maps/e08-duel-36.h3m";

fn options(days: u32) -> RunOptions {
    RunOptions {
        map: DUEL.to_owned(),
        profile: DEFAULT_PROFILE.to_owned(),
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
    ];
    for (player, command, text) in cases {
        assert_eq!(format_command(player, &command), text);
        assert_eq!(parse_command(text).unwrap(), (player, command));
    }
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
        "--record",
        &out,
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
                workload,
                games: 2,
                days: 2,
                seed: 1,
                forks: 7,
                observe,
                masks,
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
    // The duel's players have two heroes each.
    assert_eq!(reach.operations, 2 * reach.steps);
    let fork = bench(bench::Workload::Fork, false, false);
    assert_eq!((fork.operations, fork.steps), (14, play.steps));
    let dir = root().display().to_string();
    let packs = format!("{dir}/packs");
    let args = |words: &[&str]| words.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
    for workload in ["play", "reach", "fork"] {
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
            "1",
            "--masks",
            "1",
        ]))
        .unwrap();
    }
    for bad in [
        args(&["bench"]),
        args(&["bench", "--map", DUEL, "--workload", "walk"]),
        args(&["bench", "--map", DUEL, "--days", "0"]),
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
