//! Tests of `h3 ai ceiling` (E25 T42): without a budget the improved seat plays the fair actor's
//! games; a run is the same on any threads and continues from its games file; a decision repeats,
//! keeps the fair actor's command among its candidates, and on the side's worlds reads nothing
//! the side may not know.

use super::*;

fn settings(set: &str, budget: u32, max_steps: u32) -> Settings {
    Settings {
        set: set.to_owned(),
        episodes: 1,
        opponents: vec!["system".to_owned()],
        budget,
        worlds: Worlds::Side,
        max_steps,
        seed: 0,
        identity: String::new(),
    }
}

fn options(settings: Settings, threads: usize, out: Option<PathBuf>) -> CeilingOptions {
    CeilingOptions {
        settings,
        threads,
        out,
    }
}

fn quiet(_: &str) {}

/// Two records of the same games: equal but for the part's seconds.
fn same(a: &CeilingReport, b: &CeilingReport) {
    assert_eq!(a.settings, b.settings);
    assert_eq!(a.reports, b.reports);
    assert_eq!(a.decisions, b.decisions);
    assert_eq!(a.games, b.games);
}

#[test]
fn without_a_budget_the_improved_seat_plays_the_fair_actors_games() {
    // Budget 0: the seat plays the fair actor's command, drawn from its stream as the fair actor
    // draws it, so every game is `h3 ai eval`'s game of the fair actor against itself.
    let sampler = bench::sampler().unwrap();
    for set in ["report-v1", "neutral-v1"] {
        let report = run(
            &sampler,
            &options(settings(set, 0, 2000), 2, None),
            &mut quiet,
        )
        .unwrap();
        let neutral = set == "neutral-v1";
        let mut policies = vec![("a".to_owned(), Policy::System)];
        if !neutral {
            policies.push(("b".to_owned(), Policy::System));
        }
        let mirror = eval::run(
            &sampler,
            &eval::EvalOptions {
                set: set.to_owned(),
                policies,
                defender: neutral.then(|| ("b".to_owned(), Policy::System)),
                strata: if neutral {
                    Stratum::neutral_v1(1)
                } else {
                    Stratum::report_v1(1)
                }
                .unwrap(),
                max_steps: 2000,
            },
        )
        .unwrap();
        let pairs = &report.reports[0].pairs;
        let other = if neutral { (1, 1) } else { (1, 0) };
        assert_eq!(pairs[0][1], mirror.pairs[0][1]);
        assert_eq!(pairs[other.0][other.1], mirror.pairs[other.0][other.1]);
        assert!(pairs[0][1].iter().all(|p| p.episodes == 1));
        let sum = &report.decisions[0];
        assert!(sum.decisions.iter().sum::<u32>() > 0);
        assert_eq!((sum.differ, sum.rollouts, sum.refused), ([0; 8], 0, 0));
        assert!(sum.advantage.iter().all(|&a| a == 0.0));
        let text = report.to_text();
        assert!(CATEGORIES.iter().all(|c| text.contains(c)), "{text}");
        assert!(report.to_json().contains("\"tool\": \"h3 ai ceiling\""));
    }
}

#[test]
fn a_run_is_the_same_on_any_threads_and_continues_from_its_games_file() {
    let sampler = bench::sampler().unwrap();
    let s = settings("report-v1", 0, 2000);
    let one = run(&sampler, &options(s.clone(), 1, None), &mut quiet).unwrap();
    let dir = std::env::temp_dir().join(format!("h3-ceiling-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("games.toml");
    let _ = std::fs::remove_file(&path);
    let full = run(
        &sampler,
        &options(s.clone(), 3, Some(path.clone())),
        &mut quiet,
    )
    .unwrap();
    same(&one, &full);
    assert_eq!(full.games, 6);
    // A stopped run leaves the games it finished: cut the file after its second, then continue.
    let text = std::fs::read_to_string(&path).unwrap();
    let (cut, _) = text.match_indices("\n[[game]]").nth(2).unwrap();
    std::fs::write(&path, &text[..cut]).unwrap();
    let mut lines = Vec::new();
    let resumed = run(
        &sampler,
        &options(s.clone(), 2, Some(path.clone())),
        &mut |line| lines.push(line.to_owned()),
    )
    .unwrap();
    assert!(
        lines[0].contains("4 to play") && lines[0].contains("(2 held)"),
        "{lines:?}"
    );
    same(&full, &resumed);
    // A stop during an append leaves the last game cut short: it is dropped and played again.
    let text = std::fs::read_to_string(&path).unwrap();
    let (last, _) = text.match_indices("\n[[game]]").last().unwrap();
    std::fs::write(&path, &text[..last + 30]).unwrap();
    let mut lines = Vec::new();
    let torn = run(
        &sampler,
        &options(s.clone(), 2, Some(path.clone())),
        &mut |line| {
            lines.push(line.to_owned());
        },
    )
    .unwrap();
    assert!(lines[0].contains("1 to play"), "{lines:?}");
    same(&full, &torn);
    // The file holds every game now: a third start plays none.
    let mut lines = Vec::new();
    let again = run(
        &sampler,
        &options(s.clone(), 2, Some(path.clone())),
        &mut |line| {
            lines.push(line.to_owned());
        },
    )
    .unwrap();
    assert!(lines[0].contains("0 to play"), "{lines:?}");
    same(&full, &again);
    let other = Settings { seed: 1, ..s };
    let refused = run(&sampler, &options(other, 1, Some(path.clone())), &mut quiet).unwrap_err();
    assert!(refused.contains("other settings"), "{refused}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_decision_repeats_keeps_the_fair_actors_command_and_reads_nothing_hidden_on_side_worlds() {
    let sampler = bench::sampler().unwrap();
    // A hero battle of the report set, a short step limit so that the rollouts stay short.
    let stratum = &Stratum::report_v1(1).unwrap()[0];
    let source = bench::encounters_of(&sampler, stratum.kinds).unwrap();
    let seed = episode_seed(stratum.seed, 0, 0);
    let env = source.episode(seed, 4).unwrap().env;
    let side = env.to_move().unwrap();
    let mut scratch = Scratch::default();
    let mut rng = Rng::for_stream(seed, StreamId::seat(0));
    let base = Policy::System.choose(&env, side, &mut rng, &mut scratch.legal);
    let s = settings("report-v1", 1, 4);
    let a = decide(&env, side, &sampler, &s, 7, base, &mut scratch).unwrap();
    assert!(a.candidates > 1 && a.rollouts >= a.candidates, "{a:?}");
    assert!(a.advantage >= 0.0 && a.refused == 0, "{a:?}");
    assert_eq!(a.base, base);
    // Every legal command but the escapes is a candidate, the fair actor's too, and each applies.
    let mut commands = Vec::new();
    candidates(&env, side, &mut scratch.legal, &mut commands);
    legal_commands(env.state(), env.ruleset(), side, &mut scratch.legal);
    for c in scratch.legal.iter().filter(|c| !c.leaves_battle()) {
        assert!(commands.contains(c), "{c:?}");
    }
    assert!(commands.contains(&base));
    for &c in &commands {
        rollout::step_command(&mut env.fork(), side, c).unwrap();
    }
    // The battle with the other hero's private part guessed anew (and another stream) decides
    // alike on the side's worlds, with fresh scratch: they read nothing the side may not know,
    // and a decision repeats.
    let view = env.state().hero(side.opponent()).unwrap();
    let siege = env.setup().siege.is_some();
    let mut guesser = Rng::for_stream(99, StreamId::ENCOUNTER);
    let other = sampler
        .guess(&mut guesser, &view, siege, GUESS_CANDIDATES)
        .unwrap();
    let twin = env.world(side, Some(&other), 5).unwrap();
    assert_ne!(twin.state_hash(), env.state_hash());
    let mut fresh = Scratch::default();
    assert_eq!(
        decide(&twin, side, &sampler, &s, 7, base, &mut fresh).unwrap(),
        a
    );
    // A true world is the battle itself on the rollout's stream; a side world is not.
    let truth = world_of(&env, side, &sampler, Worlds::True, 3).unwrap();
    let mut fork = env.fork();
    fork.reseed(3);
    assert_eq!(truth.state_hash(), fork.state_hash());
    let world = world_of(&env, side, &sampler, Worlds::Side, 3).unwrap();
    assert_ne!(world.state_hash(), truth.state_hash());
}

#[test]
fn the_settings_and_the_categories_are_checked() {
    assert_eq!(Worlds::parse("side").unwrap().name(), "side");
    assert_eq!(Worlds::parse("true").unwrap().name(), "true");
    assert!(Worlds::parse("all").is_err());
    assert!(Plan::of(&settings("v0", 0, 10)).is_err());
    let mut s = settings("neutral-v1", 0, 10);
    s.opponents = vec!["greedy".to_owned()];
    assert!(Plan::of(&s).unwrap_err().contains("defender"));
    s.opponents = vec!["nobody".to_owned()];
    assert!(Plan::of(&s).is_err());
    let plan = Plan::of(&settings("neutral-v1", 0, 10)).unwrap();
    assert_eq!(plan.games.len(), 6);
    assert!(
        plan.games
            .iter()
            .all(|g| g.improved == 0 || g.improved == 2)
    );
    let sampler = bench::sampler().unwrap();
    let stratum = &Stratum::report_v1(1).unwrap()[0];
    let env = bench::encounters_of(&sampler, stratum.kinds)
        .unwrap()
        .episode(episode_seed(stratum.seed, 0, 0), 10)
        .unwrap()
        .env;
    let tactics = usize::from(env.state().round() != 0);
    assert_eq!(category(&env, Command::Defend), tactics);
}
