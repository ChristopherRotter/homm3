//! E05 T3c: `h3 ai bench` on the synthetic scenarios.

use std::path::PathBuf;

use super::bench::{BenchOptions, run};
use super::main;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn options(scenario: &str, threads: u32) -> BenchOptions {
    BenchOptions {
        scenario: root().join("packs/synthetic/scenarios").join(scenario),
        packs_dir: root().join("packs"),
        envs: 5,
        threads,
        calls: 60,
        seed: 3,
        max_steps: 0,
    }
}

#[test]
fn every_chosen_action_applies_whatever_the_thread_count() {
    for scenario in ["full.toml", "siege.toml"] {
        let one = run(&options(scenario, 0)).unwrap();
        assert_eq!(one.steps, 5 * 60, "{scenario}");
        assert_eq!(one.refused, 0);
        for threads in [1, 3, 8] {
            let many = run(&options(scenario, threads)).unwrap();
            assert_eq!(
                (many.steps, many.episodes),
                (one.steps, one.episodes),
                "{scenario}, {threads} threads"
            );
        }
    }
    // A step limit ends episodes; the batch resets them in the same call.
    let limited = run(&BenchOptions {
        max_steps: 7,
        ..options("full.toml", 2)
    })
    .unwrap();
    assert!(limited.episodes >= 5 * (60 / 7), "{limited:?}");
}

#[test]
fn the_command_line() {
    let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let scenario = root().join("packs/synthetic/scenarios/duel.toml");
    let packs = root().join("packs");
    let ok = args(&[
        "bench",
        "--scenario",
        scenario.to_str().unwrap(),
        "--packs-dir",
        packs.to_str().unwrap(),
        "--envs",
        "3",
        "--threads",
        "2",
        "--calls",
        "5",
    ]);
    main(&ok).unwrap();
    assert!(main(&[]).unwrap_err().starts_with("usage"));
    assert!(
        main(&args(&["fly"]))
            .unwrap_err()
            .contains("unknown ai command")
    );
    assert!(
        main(&args(&["bench", "--envs"]))
            .unwrap_err()
            .contains("needs a value")
    );
    assert!(
        main(&args(&["bench", "--envs", "x"]))
            .unwrap_err()
            .contains("not a valid number")
    );
    assert!(
        main(&args(&["bench", "--fast", "1"]))
            .unwrap_err()
            .contains("unknown option --fast")
    );
    assert!(
        main(&args(&["bench", "now"]))
            .unwrap_err()
            .contains("unexpected argument")
    );
    let missing = args(&["bench", "--scenario", "nowhere.toml"]);
    assert!(main(&missing).is_err());
    let no_envs = [ok.clone(), args(&["--envs", "0"])].concat();
    assert!(main(&no_envs).unwrap_err().contains("at least one env"));
}
