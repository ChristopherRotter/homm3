//! Golden adventure replays (E08 T9b, increment 1's exit criteria): every replay under
//! `fixtures/replays/adventure/` must reproduce its recorded state hashes on this platform and
//! build configuration. `cargo xtask determinism` runs this test in debug, release and release
//! with `-C target-cpu=native`; running it on another OS is the cross-platform check. The replays
//! play the generated duel map `fixtures/maps/e08-duel-36.h3m` (re-record them with
//! `h3 adventure run … --record` when a rule or an encoding changes on purpose; the commit says
//! why), recorded with `--map fixtures/maps/e08-duel-36.h3m` and the options below. Since E08
//! T20b3b2 the duel holds increment 3's monsters, guarded piles and special grounds, a monster and
//! a guarded pile beside each hero, so random play meets them in every game; what each replay
//! meets (from the T20b3b2 recording):
//!
//! - `duel-s1-d28` (`--seed 1 --agent-seed 1 --days 28`): a key; a declined offer to join, then
//!   the flight pursued and its battle won; another pursuit won.
//! - `duel-s2-d7` (`--seed 2 --agent-seed 3 --days 7`): a Cartographer's offer accepted, a
//!   monster's battle won.
//! - `duel-s5-strict-d7` (`--seed 5 --agent-seed 8 --days 7 --profile base:strict-fixed`): a
//!   monster's battle won in strict-fixed (the split and the obstacles from the stream).
//! - `duel-s3-easy-d7` (`--seed 3 --agent-seed 4 --days 7 --difficulty 0`): two monsters' battles
//!   won at the easy difficulty (E08 T14b: the replay's difficulty, which `verify` plays at).
//! - `duel-s535-d28` (`--seed 535 --agent-seed 535 --days 28`): a Cartographer accepted, a guarded
//!   pile fought at once and picked up, a pile's question declined, a hero battle won with a
//!   capture.
//! - `duel-s1218-d28` (`--seed 1218 --agent-seed 1218 --days 28`): a pursuit won, a key, a pile's
//!   question declined, a border guard's passage accepted (the guard removed).
//! - `duel-s6-d28` (`--seed 6 --agent-seed 6 --days 28`): a pile's question accepted and its
//!   guards beaten, a group joining, a key, a pursuit won, a pile's guards beating the visitor
//!   (their survivors stay, the hero leaves the map).
//! - `duel-s1429-d28` (`--seed 1429 --agent-seed 1429 --days 28`): a declined join and a declined
//!   flight (the monster leaves), a group joining, a monster's battle lost, a Cartographer declined.
//! - `duel-s100-strict-d28` (`--seed 100 --agent-seed 100 --days 28 --profile
//!   base:strict-fixed`): a Cartographer accepted, a monster's battle won, a hero battle lost in
//!   strict-fixed, the winner capturing.
//!
//! The last five came from a search over seeds (`--seed N --agent-seed N --days 28` for N up to
//! 1 500, `strict-fixed` up to 800; E08 T20b3b2): of 1 500 such games 10 have a hero battle, 3 a
//! guard's passage, 58 a pile's guards beating the visitor. History: E08 T16a re-recorded the
//! first four when the duel got increment 2's objects and added two searched ones (a
//! Cartographer's offer, a key and a guard's passage; a declined and an accepted offer), E08 T19b1
//! two with hero battles (format v3), and every later version bump (T20a1, T20a2a, T20b1,
//! `shared/e08-t20b2`, E04c T9, T21a) re-recorded them all with their options, their commands
//! unchanged. E08 T20b3b2 changed the duel, so every command changed: the first four were
//! re-recorded with their options and the four searched ones (`duel-s892-d28`, `duel-s988-d28`,
//! `duel-s792-d28`, `duel-s120-strict-d28`), which no longer met what they were chosen for, were
//! replaced by the five above.

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
        files.len() >= 9,
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
