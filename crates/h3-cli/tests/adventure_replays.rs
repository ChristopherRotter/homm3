//! Golden adventure replays (E08 T9b, increment 1's exit criteria): every replay under
//! `fixtures/replays/adventure/` must reproduce its recorded state hashes on this platform and
//! build configuration. `cargo xtask determinism` runs this test in debug, release and release
//! with `-C target-cpu=native`; running it on another OS is the cross-platform check. The replays
//! play the generated duel map `fixtures/maps/e08-duel-36.h3m` (re-record them with
//! `h3 adventure run … --record` when a rule or an encoding changes on purpose; the commit says
//! why). They were recorded with `--map fixtures/maps/e08-duel-36.h3m` and: `duel-s1-d28`
//! `--seed 1 --agent-seed 1 --days 28`; `duel-s2-d7` `--seed 2 --agent-seed 3 --days 7`;
//! `duel-s5-strict-d7` `--seed 5 --agent-seed 8 --days 7 --profile base:strict-fixed`;
//! `duel-s3-easy-d7` `--seed 3 --agent-seed 4 --days 7 --difficulty 0` (E08 T14b: the replay's
//! difficulty, which `verify` plays at, shows in the treasuries). All four were re-recorded with
//! the same options in E08 T16a, when the duel got increment 2's typed objects: the 28-day replay
//! picks up a crystal pile and takes the green key, `duel-s2-d7` a campfire and a sulfur pile,
//! the strict one a gold pile and the green key; none of these four meets an offer (random play's
//! moves seldom end on an object: the T16a log). So two more were chosen from a search over
//! seeds (`--seed N --agent-seed N --days 28` for N up to 1 200, E08 T16a): `duel-s892-d28`
//! accepts a Cartographer's offer (paid, a level revealed), takes a key and accepts a border
//! guard's pass (the guard removed), and `duel-s988-d28` declines an offer, accepts one and
//! visits an observatory, so the matrix replays both answers and both offer kinds.

use std::path::PathBuf;

#[test]
fn golden_adventure_replays_reproduce_their_hashes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("fixtures/replays/adventure");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 6,
        "expected the E08 replay fixtures in {}",
        dir.display()
    );
    for file in &files {
        let verified = h3_cli::adventure::verify_file(file, &root.join("packs"), &root);
        assert!(
            verified.is_ok(),
            "{}: {}",
            file.display(),
            verified.unwrap_err()
        );
    }
}
