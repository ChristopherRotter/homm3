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
        commands: String::new(),
        confirm: 0,
    }
}

fn options(settings: Settings, threads: usize, out: Option<PathBuf>) -> CeilingOptions {
    CeilingOptions {
        settings,
        threads,
        out,
        commands: None,
        leave_out: Vec::new(),
    }
}

fn quiet(_: &str) {}

fn plan_of(settings: &Settings) -> Result<Plan, String> {
    Plan::of(settings, None, &[])
}

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
    assert!(plan_of(&settings("v0", 0, 10)).is_err());
    let mut s = settings("neutral-v1", 0, 10);
    s.opponents = vec!["greedy".to_owned()];
    assert!(plan_of(&s).unwrap_err().contains("defender"));
    s.opponents = vec!["nobody".to_owned()];
    assert!(plan_of(&s).is_err());
    let plan = plan_of(&settings("neutral-v1", 0, 10)).unwrap();
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

/// A recording of `policy` playing the external seat of `set`'s batches (as the Python
/// ladder's: per kind and side a batch of the stratum's seed and envs, the opponent the fair
/// actor inside it; `BattleBatch::choice` draws from the external seat's stream), in the format
/// `h3env.train.inspect --commands` writes.
fn recording(sampler: &Sampler, set: &str, policy: Policy, max_steps: u32) -> String {
    use h3_sim::batch::{BatchConfig, BattleBatch, EnvReport, Outputs, Seat};
    use std::sync::Arc;
    let neutral = set == "neutral-v1";
    let strata = if neutral {
        Stratum::neutral_v1(1)
    } else {
        Stratum::report_v1(1)
    }
    .unwrap();
    let mut text = format!(
        "[record]\nnetwork = \"runs/test/best.pt\"\nset = \"{set}\"\nepisodes = 1\nopponent = \
         \"system\"\nmax_steps = {max_steps}\naction_encoding = {ACTION_ENCODING_VERSION}\n"
    );
    let sides: &[u8] = if neutral { &[0] } else { &[0, 1] };
    for s in &strata {
        for &side in sides {
            let mut seats = [Seat::Internal(Policy::System); 2];
            seats[usize::from(side)] = Seat::External;
            let weighted = sampler.clone().with_weights(s.kinds).unwrap();
            let scenario = Scenario::encounters(Arc::new(weighted), 1, Some(seats)).unwrap();
            let config = BatchConfig {
                envs: s.envs,
                threads: 0,
                seed: s.seed,
                max_steps,
                reward: RewardConfig::default(),
            };
            let mut batch = BattleBatch::new(config).unwrap();
            batch.add_scenario(scenario).unwrap();
            let n = s.envs as usize;
            let mut reports = vec![EnvReport::default(); n];
            batch
                .reset(&mut Outputs {
                    reports: Some(&mut reports),
                    ..Outputs::default()
                })
                .unwrap();
            let mut played: Vec<Vec<[u32; 4]>> = vec![Vec::new(); n];
            let (mut actions, mut encoded) = (vec![Action::default(); n], vec![0u8; n]);
            loop {
                let live: Vec<bool> = reports
                    .iter()
                    .map(|r| r.episode == 0 && r.to_move.is_some())
                    .collect();
                if !live.contains(&true) {
                    break;
                }
                batch.choice(policy, &mut actions, &mut encoded).unwrap();
                for e in (0..n).filter(|&e| live[e]) {
                    let a = actions[e];
                    played[e].push([
                        a.entry,
                        u32::from(a.spell),
                        u32::from(a.target),
                        u32::from(a.second),
                    ]);
                }
                batch
                    .step(
                        &actions,
                        &mut Outputs {
                            reports: Some(&mut reports),
                            ..Outputs::default()
                        },
                    )
                    .unwrap();
                for e in (0..n).filter(|&e| live[e] && reports[e].done) {
                    let r = &reports[e];
                    let winner = match r.result.and_then(BattleResult::winner) {
                        Some(BattleSide::Attacker) => 0,
                        Some(BattleSide::Defender) => 1,
                        None => -1,
                    };
                    let actions: Vec<String> = played[e]
                        .iter()
                        .map(|a| format!("[{}, {}, {}, {}]", a[0], a[1], a[2], a[3]))
                        .collect();
                    let _ = write!(
                        text,
                        "\n[[game]]\nkind = \"{}\"\nenv = {e}\nside = {side}\nwinner = {winner}\n\
                         truncated = {}\nreward = [{:?}, {:?}]\nactions = [{}]\n",
                        s.name,
                        r.truncated,
                        f64::from(r.reward[0]),
                        f64::from(r.reward[1]),
                        actions.join(", ")
                    );
                }
            }
        }
    }
    text
}

fn replay(recorded: &Recorded, budget: u32, confirm: u32, max_steps: u32) -> Settings {
    Settings {
        set: recorded.record.set.clone(),
        commands: recorded.name(),
        confirm,
        ..settings(&recorded.record.set, budget, max_steps)
    }
}

fn replayed(settings: Settings, recorded: &Recorded, out: Option<PathBuf>) -> CeilingOptions {
    CeilingOptions {
        commands: Some(recorded.clone()),
        ..options(settings, 2, out)
    }
}

#[test]
fn a_replay_plays_the_batchs_recorded_games_and_checks_their_ends() {
    // Budget 0: the replay plays the recorded games alone; each must end as the batch's did.
    let sampler = bench::sampler().unwrap();
    for set in ["report-v1", "neutral-v1"] {
        let text = recording(&sampler, set, Policy::Random, 2000);
        let recorded = Recorded::parse(&text).unwrap();
        assert_eq!(recorded.game.len(), if set == "neutral-v1" { 3 } else { 6 });
        let s = replay(&recorded, 0, 0, 2000);
        let report = run(&sampler, &replayed(s.clone(), &recorded, None), &mut quiet).unwrap();
        assert_eq!(report.games, recorded.game.len());
        assert_eq!(report.seat, "runs/test/best.pt");
        assert_eq!(report.reports[0].names[0], "runs/test/best.pt");
        let v = report.decisions[0].valued.as_ref().unwrap();
        assert!(v.net.iter().sum::<u32>() > 0);
        assert_eq!(
            v.net.iter().sum::<u32>(),
            report.decisions[0].decisions.iter().sum::<u32>()
        );
        assert!(report.to_text().contains("replayed: runs/test/best.pt"));
        assert!(report.to_json().contains("\"seat\": \"runs/test/best.pt\""));
        // A recording that does not end as recorded is refused: another reward, an action less.
        let mut wrong = recorded.clone();
        wrong.game[0].reward[0] += 1.0;
        let refused = run(&sampler, &replayed(s.clone(), &wrong, None), &mut quiet).unwrap_err();
        assert!(refused.contains("another engine"), "{refused}");
        let mut short = recorded.clone();
        short.game[0].actions.pop();
        let refused = run(&sampler, &replayed(s.clone(), &short, None), &mut quiet).unwrap_err();
        assert!(refused.contains("end before the battle"), "{refused}");
        let mut long = recorded.clone();
        long.game[0].actions.push([0; 4]);
        let refused = run(&sampler, &replayed(s, &long, None), &mut quiet).unwrap_err();
        assert!(refused.contains("1 recorded actions left"), "{refused}");
    }
}

#[test]
fn a_replay_values_the_networks_and_the_fair_actors_commands() {
    // The neutral set's three battles at a short step limit keep the rollouts short.
    let (set, steps) = ("neutral-v1", 6);
    let sampler = bench::sampler().unwrap();
    // The fair actor's own play recorded: its command is the network's at every decision (both
    // draw from the side's seat stream in order), so their regrets are equal.
    let own = Recorded::parse(&recording(&sampler, set, Policy::System, steps)).unwrap();
    let report = run(
        &sampler,
        &replayed(replay(&own, 1, 2, steps), &own, None),
        &mut quiet,
    )
    .unwrap();
    let sum = &report.decisions[0];
    let v = sum.valued.as_ref().unwrap();
    assert_eq!(v.same_base, v.net);
    assert_eq!(v.regret, v.base_regret);
    assert_eq!(sum.differ, [0; 8]);
    assert!(sum.advantage.iter().all(|&a| a == 0.0));
    assert!(sum.rollouts > 0 && v.sample_regret >= 0.0, "{v:?}");
    // Random's play: other commands, valued by the halving with both kept, then confirmed.
    let random = Recorded::parse(&recording(&sampler, set, Policy::Random, steps)).unwrap();
    let s = replay(&random, 1, 2, steps);
    let one = run(&sampler, &replayed(s.clone(), &random, None), &mut quiet).unwrap();
    let sum = &one.decisions[0];
    let v = sum.valued.as_ref().unwrap();
    assert!(sum.differ.iter().sum::<u32>() > 0);
    assert!(v.same_base.iter().sum::<u32>() < v.net.iter().sum::<u32>());
    // The best is the halving's maximum, the network's command always among the last round's.
    // No world refuses a command on the neutral set (no other hero to guess).
    assert!(v.sample_regret >= 0.0);
    assert_eq!((v.refused, sum.refused), ([0; 3], 0));
    assert_eq!(v.best.iter().sum::<u32>(), v.net.iter().sum::<u32>());
    let regret: f64 = v.regret.iter().sum();
    let missed: f64 = v.missed.iter().sum();
    assert!((regret - missed).abs() < 1e-9);
    let text = one.to_text();
    assert!(text.contains("a decision: the network's regret"), "{text}");
    assert!(one.to_json().contains("\"valued\": {\"net\": ["));
    // The same on other threads, and continued from its games file.
    let dir = std::env::temp_dir().join(format!("h3-ceiling-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("games.toml");
    let _ = std::fs::remove_file(&path);
    let mut threads = replayed(s.clone(), &random, Some(path.clone()));
    threads.threads = 3;
    same(&one, &run(&sampler, &threads, &mut quiet).unwrap());
    let text = std::fs::read_to_string(&path).unwrap();
    let (cut, _) = text.match_indices("\n[[game]]").nth(1).unwrap();
    std::fs::write(&path, &text[..cut]).unwrap();
    same(&one, &run(&sampler, &threads, &mut quiet).unwrap());
    // Another recording, or the settings' of another, is refused.
    let refused = run(
        &sampler,
        &replayed(replay(&own, 1, 2, steps), &random, None),
        &mut quiet,
    )
    .unwrap_err();
    assert!(refused.contains("name the recording"), "{refused}");
    let other = Settings {
        episodes: 2,
        ..s.clone()
    };
    let refused = run(&sampler, &replayed(other, &random, None), &mut quiet).unwrap_err();
    assert!(refused.contains("recorded on neutral-v1"), "{refused}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn games_are_left_out_by_their_keys() {
    let sampler = bench::sampler().unwrap();
    let s = settings("report-v1", 0, 2000);
    let leave = |keys: &[&str]| CeilingOptions {
        leave_out: keys.iter().map(|&k| k.to_owned()).collect(),
        ..options(s.clone(), 2, None)
    };
    let report = run(&sampler, &leave(&["siege_hero:0:attacker"]), &mut quiet).unwrap();
    assert_eq!(report.games, 5);
    assert_eq!(report.left_out, ["siege_hero:0:attacker"]);
    assert!(report.to_text().contains("left out: siege_hero:0:attacker"));
    assert!(
        report
            .to_json()
            .contains("\"left_out\": [\"siege_hero:0:attacker\"]")
    );
    for key in [
        "siege_hero:1:attacker",
        "siege_hero:0",
        "dragons:0:attacker",
        "hero:0:both",
    ] {
        assert!(run(&sampler, &leave(&[key]), &mut quiet).is_err(), "{key}");
    }
    // A continued run that leaves out a game its file holds reads the others.
    let dir = std::env::temp_dir().join(format!("h3-ceiling-leave-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("games.toml");
    let _ = std::fs::remove_file(&path);
    let full = CeilingOptions {
        out: Some(path.clone()),
        ..options(s.clone(), 2, None)
    };
    run(&sampler, &full, &mut quiet).unwrap();
    let part = CeilingOptions {
        out: Some(path.clone()),
        ..leave(&["siege_hero:0:attacker"])
    };
    let mut lines = Vec::new();
    let again = run(&sampler, &part, &mut |l| lines.push(l.to_owned())).unwrap();
    assert!(lines[0].contains("0 to play"), "{lines:?}");
    same(&report, &again);
    std::fs::remove_dir_all(&dir).unwrap();
    // The confirmation needs a replay; a recording names its encoding and a plain network.
    let confirm = Settings {
        confirm: 4,
        ..s.clone()
    };
    assert!(plan_of(&confirm).unwrap_err().contains("--commands"));
    let text = recording(&sampler, "neutral-v1", Policy::Random, 20);
    assert!(Recorded::parse(&text.replace("action_encoding = ", "action_encoding = 9")).is_err());
    assert!(Recorded::parse(&text.replace("runs/test", "runs\\\\test")).is_err());
}
