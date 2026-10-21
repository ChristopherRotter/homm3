//! Golden adventure replays (E08 T9b, increment 1's exit criteria): every replay under
//! `fixtures/replays/adventure/` must reproduce its recorded state hashes on this platform and
//! build configuration. `cargo xtask determinism` runs this test in debug, release and release
//! with `-C target-cpu=native`; running it on another OS is the cross-platform check. The replays
//! play the generated duel map `fixtures/maps/e08-duel-36.h3m` (re-record them with
//! `h3 adventure run … --record` when a rule or an encoding changes on purpose; the commit says
//! why), recorded with `--map fixtures/maps/e08-duel-36.h3m` and the options below. Since E08
//! T20b3b2 the duel holds increment 3's monsters, guarded piles and special grounds, a monster and
//! a guarded pile beside each hero, so random play meets them in every game, and since E08 T31 its
//! heroes' custom experience brings their first battles to a level-up; what each replay meets
//! (from the E08 T31 recording):
//!
//! - `duel-s1-d28` (`--seed 1 --agent-seed 1 --days 28`): a key; twice a declined offer to join
//!   and an accepted one (the group joins), two flights pursued and their battles won, a level-up
//!   answered with the second offer.
//! - `duel-s2-d7` (`--seed 2 --agent-seed 3 --days 7`): a Cartographer's offer accepted, a
//!   monster's battle won and its level-up answered with the second offer.
//! - `duel-s5-strict-d7` (`--seed 5 --agent-seed 8 --days 7 --profile base:strict-fixed`): in
//!   strict-fixed (the split, the obstacles and the levels from the streams) a key and two declined
//!   flights (the monsters leave); no battle.
//! - `duel-s3-easy-d7` (`--seed 3 --agent-seed 4 --days 7 --difficulty 0`): at the easy difficulty
//!   (E08 T14b: the replay's difficulty, which `verify` plays at) an offer to join for gold
//!   accepted, the group joining.
//! - `duel-s767-d28` (`--seed 767 --agent-seed 767 --days 28`): a hero battle won with a capture,
//!   four monsters' battles won, a guarded pile fought at once and picked up, a key, four
//!   level-ups answered with both offers.
//! - `duel-s2626-d28` (`--seed 2626 --agent-seed 2626 --days 28`): a border guard's passage
//!   accepted with its key (the guard removed), a declined join and a group joining, a declined
//!   flight (the monster leaves) and a pursuit won, a pile fought at once and picked up, two
//!   level-ups answered with the second offer.
//! - `duel-s236-d28` (`--seed 236 --agent-seed 236 --days 28`): a Cartographer accepted, a group
//!   joining, a pile's question declined, another pile's guards beating the visitor (their
//!   survivors stay, the hero leaves the map), a pursuit won, level-ups answered with each offer.
//! - `duel-s12-d28` (`--seed 12 --agent-seed 12 --days 28`): a monster's battle lost, a declined
//!   join and a declined flight, a group joining, a pursuit won, a Cartographer declined, a pile's
//!   question accepted and its guards beaten, a key, level-ups answered with each offer.
//! - `duel-s929-strict-d28` (`--seed 929 --agent-seed 929 --days 28 --profile
//!   base:strict-fixed`): in strict-fixed a Cartographer accepted, a group joining, a pursuit won,
//!   monsters' battles won and one lost, a hero battle lost, the winner capturing, three level-ups
//!   answered with each offer.
//!
//! In every one red's underground hero gains levels 2 to 4 at load (E08 T31: the duel's custom
//! experience, `fixtures/maps/README.md`; from its level seed in faithful, from the setup stream in
//! strict-fixed). The last five came from a search over seeds (E08 T31: `--seed N --agent-seed N
//! --days 28` for N up to 5 000 in each profile, with a scratch probe replaying each game's
//! commands and noting its offers, answers, battles and their ends and its levels, not committed):
//! of the 5 000 faithful games 32 have a hero battle (5 with a capture), 11 a guard's passage, 139
//! a pile's guards beating the visitor, 4 725 a level in play and 2 311 answers with both offers;
//! of the strict-fixed ones 34 a hero battle (8 with a capture). Each stands for one of the five
//! it replaced (`duel-s1279-d28`, `duel-s810-d28`, `duel-s328-d28`, `duel-s1122-d28`,
//! `duel-s307-strict-d28`, E04c T10b's) with the closest game: `duel-s767-d28` lacks the declined
//! join and flight of the hero battle's, `duel-s236-d28` the key, `duel-s929-strict-d28` the pile
//! fought at once (none of the five strict-fixed games with a lost hero battle and a capture has
//! one); `duel-s2626-d28` was taken over `duel-s2348-d28`, which has the whole of the passage's
//! set, for its declined join and flight.
//! History: E08 T16a re-recorded the first four when the duel got increment 2's objects and added
//! two searched ones, E08 T19b1 two with hero battles (format v3); every version bump since (T20a1,
//! T20a2a, T20b1, `shared/e08-t20b2`, E04c T9, T21a, T29, T30) re-recorded them all with their
//! options, their commands unchanged; a change of the duel changes every command: E08 T20b3b2
//! (monsters, guarded piles, grounds), E04c T10b (the original's obstacles, `cross-lane`) and E08
//! T31 (the heroes' experience) re-recorded the first four with their options and replaced the
//! searched ones, which no longer met what they were chosen for.

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
