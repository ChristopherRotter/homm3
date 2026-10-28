use h3_battle::{CastTarget, Command, Hex, StackId};
use h3_content::SpellId;

use super::*;

fn manage(command: Manage) -> GameCommand {
    GameCommand::Manage(command)
}

const fn slot(army: ArmyRef, slot: u8) -> ArmySlot {
    ArmySlot { army, slot }
}

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
fn commands_round_trip_through_their_text() {
    let hero = HeroId::new(12);
    let cases = [
        (PlayerId::new(0), GameCommand::EndTurn, "0 end-turn"),
        // Format v7 (E08 T63a): the answers typed by the decision's kind.
        (
            PlayerId::new(2),
            answer(Answer::Offer(Consent::Accept)),
            "2 answer offer accept",
        ),
        (
            PlayerId::new(3),
            answer(Answer::Offer(Consent::Decline)),
            "3 answer offer decline",
        ),
        (
            PlayerId::new(1),
            answer(Answer::LevelUp(SkillOffer::First)),
            "1 answer level-up first",
        ),
        (
            PlayerId::new(0),
            answer(Answer::LevelUp(SkillOffer::Second)),
            "0 answer level-up second",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Chest(ChestChoice::Gold)),
            "0 answer chest gold",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Chest(ChestChoice::Experience)),
            "0 answer chest experience",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Arena(ArenaChoice::Attack)),
            "0 answer arena attack",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Arena(ArenaChoice::Defense)),
            "0 answer arena defense",
        ),
        (
            PlayerId::new(0),
            answer(Answer::School(SchoolChoice::First)),
            "0 answer school first",
        ),
        (
            PlayerId::new(0),
            answer(Answer::School(SchoolChoice::Second)),
            "0 answer school second",
        ),
        (
            PlayerId::new(0),
            answer(Answer::School(SchoolChoice::Leave)),
            "0 answer school leave",
        ),
        (
            PlayerId::new(0),
            answer(Answer::University(UniversityAnswer::Buy(3))),
            "0 answer university buy 3",
        ),
        (
            PlayerId::new(0),
            answer(Answer::University(UniversityAnswer::Leave)),
            "0 answer university leave",
        ),
        (PlayerId::new(0), answer(Answer::Close), "0 answer close"),
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
        // Format v6 (E08 T51a): the management commands.
        (
            PlayerId::new(0),
            manage(Manage::Army(ArmyCommand::Move {
                from: slot(ArmyRef::Hero(hero), 0),
                to: slot(ArmyRef::Hero(HeroId::new(3)), 6),
            })),
            "0 army move 12:0 3:6",
        ),
        (
            PlayerId::new(1),
            manage(Manage::Army(ArmyCommand::Split {
                from: slot(ArmyRef::Group, 2),
                to: slot(ArmyRef::Hero(hero), 1),
                count: 40000,
            })),
            "1 army split g:2 12:1 40000",
        ),
        (
            PlayerId::new(1),
            manage(Manage::Army(ArmyCommand::Dismiss {
                at: slot(ArmyRef::Hero(hero), 4),
            })),
            "1 army dismiss 12:4",
        ),
        (
            PlayerId::new(2),
            manage(Manage::Formation {
                hero,
                formation: Formation::Tight,
            }),
            "2 formation 12 tight",
        ),
        (
            PlayerId::new(2),
            manage(Manage::Formation {
                hero,
                formation: Formation::Loose,
            }),
            "2 formation 12 loose",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Tactics { hero, on: false }),
            "0 tactics 12 off",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Tactics { hero, on: true }),
            "0 tactics 12 on",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Artifact {
                from: (hero, ArtifactPlace::Backpack(0)),
                to: (HeroId::new(3), ArtifactPlace::Worn(Position(18))),
            }),
            "0 artifact 12:b0 3:w18",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Assemble {
                hero,
                at: Position(6),
            }),
            "0 assemble 12 6",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Disassemble {
                hero,
                at: Position(2),
            }),
            "0 disassemble 12 2",
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
        "0 accept",
        "0 decline",
        "0 answer first",
        "0 answer",
        "0 answer offer",
        "0 answer offer maybe",
        "0 answer offer accept now",
        "0 answer level-up third",
        "0 answer chest silver",
        "0 answer arena power",
        "0 answer school stay",
        "0 answer university",
        "0 answer university buy",
        "0 answer university buy x",
        "0 answer university buy 256",
        "0 answer university buy 1 now",
        "0 answer university stay",
        "0 answer close now",
        "0 answer shrug",
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
        "0 army",
        "0 army jump 1:0",
        "0 army move 1:0",
        "0 army move 1-0 1:1",
        "0 army move x:0 1:1",
        "0 army move 1:0 1:x",
        "0 army split 1:0 1:1",
        "0 army split 1:0 1:1 x",
        "0 army dismiss",
        "0 army dismiss 1:0 now",
        "0 formation 1",
        "0 formation 1 wedge",
        "0 formation x loose",
        "0 tactics 1",
        "0 tactics 1 maybe",
        "0 artifact 1:w0",
        "0 artifact 1:w0 1",
        "0 artifact 1:x0 1:b0",
        "0 artifact 1:w 1:b0",
        "0 artifact 1:b0 1:wx",
        "0 artifact x:b0 1:w0",
        "0 artifact 1: 1:w0",
        "0 assemble 1",
        "0 assemble 1 x",
        "0 disassemble",
        "0 disassemble 1",
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
    assert!(text.contains("format = 7\n"), "{text}");
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
        .replacen("format = 7", "format = 1", 1)
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
    assert!(e.to_string().contains("format 7 needs a difficulty"), "{e}");
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
    let text = replay_to_toml(&answered).unwrap();
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
        let (up, down) = bench::heroes_by_lift(&env, player);
        assert_eq!((&up, &down), (&ids(lifted), &ids(walkers)), "{player:?}");
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
    let dir = root().display().to_string();
    let packs = format!("{dir}/packs");
    let args = |words: &[&str]| words.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
    for (workload, agent) in [
        ("play", "0"),
        ("play", "1"),
        ("reach", "1"),
        ("fork", "1"),
        ("manage", "1"),
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
