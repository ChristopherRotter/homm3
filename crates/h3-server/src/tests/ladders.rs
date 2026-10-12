//! The ladders' configuration and their seasons (E16 M4 T1).

use super::*;
use crate::config::Ladder;

fn arena(profile: Option<&str>) -> Ladder {
    let scenarios = root().join("packs/synthetic/scenarios");
    Ladder {
        id: String::from("arena"),
        name: String::from("Arena"),
        scenarios: vec![scenarios.join("duel.toml")],
        profile: profile.map(String::from),
        ..Ladder::default()
    }
}

#[test]
fn ladders_are_configured_and_checked() {
    let base = root();
    let offer = "[[offer]]\nid = \"a\"\nscenario = \"s.toml\"\n";
    let ladder = "[[ladder]]\nid = \"arena\"\nname = \"The Arena\"\nscenarios = [\"x.toml\"]\n";
    let parse =
        |text: &str| Config::parse(&format!("[content]\npacks = \"p\"\n{offer}{text}"), &base);
    assert!(parse("").unwrap().ladders.is_empty(), "none by default");
    let parsed = parse(ladder).unwrap();
    let expected = Ladder {
        id: String::from("arena"),
        name: String::from("The Arena"),
        scenarios: vec![base.join("x.toml")],
        ..Ladder::default()
    };
    assert_eq!(parsed.ladders, [expected]);
    let defaults = Ladder::default();
    assert_eq!(
        (
            defaults.accept_ms,
            defaults.grace_ms,
            defaults.grace_total_ms
        ),
        (30_000, 300_000, 600_000),
        "the user's 5 and 10 minutes (SRV-Q19)"
    );
    assert_eq!(
        (defaults.pauses, defaults.pause_ms, defaults.spectators),
        (0, 0, true)
    );
    let refused = |text: &str| parse(text).unwrap_err().0;
    for (text, expected) in [
        (format!("{ladder}{ladder}"), "its id is taken"),
        (ladder.replace("arena", "Arena"), "its id"),
        (ladder.replace("arena", "a/b"), "its id"),
        (ladder.replace("arena", &"a".repeat(33)), "its id"),
        (ladder.replace("The Arena", "A"), "not a display name"),
        (ladder.replace("[\"x.toml\"]", "[]"), "no scenarios"),
        (
            format!("{ladder}accept_ms = 999"),
            "accept_ms = 999: outside",
        ),
        (format!("{ladder}grace_ms = 0"), "grace_ms = 0: outside"),
        (
            format!("{ladder}grace_total_ms = 31536000001"),
            "grace_total_ms",
        ),
        (format!("{ladder}pauses = 101"), "pauses = 101: outside"),
        (format!("{ladder}pauses = 1"), "pauses without pause_ms"),
        (format!("{ladder}queue = 1"), "unknown field"),
    ] {
        let error = refused(&text);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    let paused = parse(&format!("{ladder}pauses = 2\npause_ms = 600000")).unwrap();
    assert_eq!(
        (paused.ladders[0].pauses, paused.ladders[0].pause_ms),
        (2, 600_000)
    );
    // The example's ladder loads.
    let example = Config::load(&base.join("crates/h3-server/server.example.toml")).unwrap();
    let ids: Vec<&str> = example.ladders.iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids, ["arena"]);
    assert!(example.ladders[0].scenarios.iter().all(|s| s.is_file()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_ladder_starts_a_season_when_its_content_changes() {
    let config = Config {
        ladders: vec![arena(None)],
        ..config(fast())
    };
    let running = start(&config).await.unwrap();
    assert_eq!(
        (running.season("arena"), running.season("other")),
        (Some(1), None)
    );
    assert_eq!(running.standing("arena", 1, "alice"), None);
    running.shutdown().await;
    // The same content: the same season.
    let running = start(&config).await.unwrap();
    assert_eq!(running.season("arena"), Some(1));
    running.shutdown().await;
    // Another profile: the next season.
    let changed = Config {
        ladders: vec![arena(Some("synthetic:strict-fixed"))],
        ..config.clone()
    };
    let running = start(&changed).await.unwrap();
    assert_eq!(running.season("arena"), Some(2));
    running.shutdown().await;
    // A results line naming an account the player log does not know stops the start.
    let log = config.server.data.join("ladders/arena.log");
    let mut text = std::fs::read_to_string(&log).unwrap();
    text.push_str("{\"NoShow\":{\"game\":1,\"replay\":\"r\",\"accounts\":[\"alice\",\"bob\"]}}\n");
    std::fs::write(&log, text).unwrap();
    let error = start(&changed).await.unwrap_err();
    assert!(
        error.0.contains("arena.log: line 3: no such account"),
        "{error}"
    );
    std::fs::remove_dir_all(&config.server.data).unwrap();
}
