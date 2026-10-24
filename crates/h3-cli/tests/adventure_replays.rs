//! Golden adventure replays (E08 T9b, increment 1's exit criteria): every replay under
//! `fixtures/replays/adventure/` must reproduce its recorded state hashes on this platform and
//! build configuration. `cargo xtask determinism` runs this test in debug, release and release
//! with `-C target-cpu=native`; running it on another OS is the cross-platform check. The replays
//! play the generated duel map `fixtures/maps/e08-duel-36.h3m` (re-record them with
//! `h3 adventure run … --record` when a rule or an encoding changes on purpose; the commit says
//! why), recorded with `--map fixtures/maps/e08-duel-36.h3m` and the options below. Since E08
//! T20b3b2 the duel holds increment 3's monsters, guarded piles and special grounds, a monster and
//! a guarded pile beside each hero, so random play meets them in every game, and since E08 T31 its
//! heroes' custom experience brings their first battles to a level-up, and since E08 T42 red's
//! surface hero flies (Angel Wings) and its underground hero walks on water (Boots of Levitation),
//! with the three skills and the other artifacts among the four (`fixtures/maps/README.md`); what
//! each replay meets (from the E08 T42 recording; a flight's or a water walk's pass is a step over
//! a tile its hero may only pass, before the path's last):
//!
//! - `duel-s1-d28` (`--seed 1 --agent-seed 1 --days 28`): a key, a monster's battle won and its
//!   level-up answered with the second offer; flights over tiles objects block, a visit at a
//!   flight's end.
//! - `duel-s2-d7` (`--seed 2 --agent-seed 3 --days 7`): flights over tiles objects block, a water
//!   walk over water; no battle.
//! - `duel-s5-strict-d7` (`--seed 5 --agent-seed 8 --days 7 --profile base:strict-fixed`): in
//!   strict-fixed (the split, the obstacles and the levels from the streams) a flight pursued and
//!   its battle won, the level-up answered with the first offer; a flight over water and over
//!   blocked tiles.
//! - `duel-s3-easy-d7` (`--seed 3 --agent-seed 4 --days 7 --difficulty 0`): at the easy difficulty
//!   (E08 T14b: the replay's difficulty, which `verify` plays at) a monster's battle won and its
//!   level-up answered with the first offer.
//! - `duel-s692-d28` (`--seed 692 --agent-seed 692 --days 28`): a hero battle won with two
//!   artifacts captured, a Cartographer accepted, a declined flight (the monster leaves), two
//!   monsters' battles won, a guarded pile fought at once and picked up, level-ups answered with
//!   both offers; flights over blocked tiles, a water walk, visits at flights' ends.
//! - `duel-s4893-d28` (`--seed 4893 --agent-seed 4893 --days 28`): a border guard's passage
//!   accepted with its key (two keys taken), a declined join and a group joining, a declined
//!   flight and a pursuit won, a pile's question accepted and its guards beaten, level-ups answered
//!   with both offers; five flights over water, water walks.
//! - `duel-s2517-d28` (`--seed 2517 --agent-seed 2517 --days 28`): a key, a Cartographer accepted,
//!   a group joining, two monsters' battles won, a pile's question declined, a pile fought at once
//!   whose guards beat the visitor (their survivors stay, the hero leaves the map), level-ups
//!   answered with both offers; a water walk.
//! - `duel-s3127-d28` (`--seed 3127 --agent-seed 3127 --days 28`): a Cartographer declined, a
//!   declined flight and two pursuits, monsters' battles won and one lost, two piles' questions
//!   accepted and their guards beaten, level-ups answered with both offers; a flight over water.
//! - `duel-s1308-strict-d28` (`--seed 1308 --agent-seed 1308 --days 28 --profile
//!   base:strict-fixed`): in strict-fixed a key, a Cartographer accepted, a declined join and a
//!   declined flight, a pursuit, monsters' battles won, a hero battle lost, the winner capturing
//!   two artifacts, level-ups answered with both offers; flights over water, a water walk.
//! - `duel-s2931-d28` (`--seed 2931 --agent-seed 2931 --days 28`; E08 T42, the tenth): an offer to
//!   join for gold accepted, two groups joining, three monsters' battles won, level-ups answered
//!   with the second offer; a flight over water.
//!
//! In every one red's underground hero gains levels 2 to 4 at load (E08 T31: the duel's custom
//! experience, `fixtures/maps/README.md`; from its level seed in faithful, from the setup stream in
//! strict-fixed). The last six came from a search over seeds (E08 T42, as T31's: `--seed N
//! --agent-seed N --days 28` for N up to 5 000 in each profile, with a scratch probe replaying
//! each game's commands and noting its offers, answers, battles and their ends, its flights and
//! water walks, not committed): of the 5 000 faithful games 83 have a hero battle (81 with a
//! capture), 19 a guard's passage, 143 a pile's guards beating the visitor, 477 a monster's battle
//! lost, one an offer to join for gold accepted (another of the strict ones), 2 758 a flight over
//! water, 1 723 a water walk over water and every game but two a flight over blocked tiles; of the
//! strict-fixed ones 69 a hero battle (63 with a capture). Each stands for one of the five it
//! replaced (`duel-s767-d28`, `duel-s2626-d28`, `duel-s236-d28`, `duel-s12-d28`,
//! `duel-s929-strict-d28`, T31's) with the closest game: `duel-s692-d28` lacks the key of the hero
//! battle's, `duel-s4893-d28` the pile fought at once of the passage's, `duel-s2517-d28` the
//! pursuit, `duel-s3127-d28` the key and the joins, `duel-s1308-strict-d28` the group joining and
//! the monster's battle lost (no strict-fixed game has a lost hero battle with both); the fixed
//! four no longer meet the joins and the strict-fixed declined flights T31's list had, and
//! `duel-s3-easy-d7` no longer its offer to join for gold: `duel-s2931-d28` was added for that.
//! History: E08 T16a re-recorded the first four when the duel got increment 2's objects and added
//! two searched ones, E08 T19b1 two with hero battles (format v3); every version bump since (T20a1,
//! T20a2a, T20b1, `shared/e08-t20b2`, E04c T9, T21a, T29, T30) re-recorded them all with their
//! options, their commands unchanged; a change of the duel changes every command: E08 T20b3b2
//! (monsters, guarded piles, grounds), E04c T10b (the original's obstacles, `cross-lane`) and E08
//! T31 (the heroes' experience) re-recorded the first four with their options and replaced the
//! searched ones, which no longer met what they were chosen for. E08 T38b (ADV-MAP-002's count:
//! every tile no object blocks, not the coast) moved red's levels at load in `duel-s1-d28`,
//! `duel-s3-easy-d7` and T31's `duel-s236-d28`: re-recorded with their options, their commands
//! unchanged, only their checkpoints and final hashes moved. E08 T40 (the heroes' day's points,
//! Pathfinding and radius from their details; adventure ruleset part v10) changed every game:
//! all nine re-recorded with their options, their commands new. E08 T42 (the duel's kits) changed
//! every game again: the first four re-recorded with their options, the searched ones replaced and
//! the tenth added. E23 T5b2b (battle state hash v52, `cross-lane`) re-recorded all ten with their
//! options, their commands unchanged: nine moved checkpoint hashes taken inside a battle,
//! `duel-s2-d7` none; so did E23 T6 twice (battle state hash v53: the tactics phase's rounds,
//! then the retreat facts and the attacking hero's own ground). E08 T49a (the worn artifacts by
//! position: game state hash v12, map import v9, the duel regenerated with its heroes' Spell Books
//! on the map's book entry) re-recorded all ten with their options, their commands unchanged:
//! every checkpoint moved. E08 T49b (map import v10: an equipped Titan's Thunder adds a Spell
//! Book, which the duel's heroes do not wear) moved only the header's import version; every hash
//! stayed. E08 T50 (adventure state hash v11: the tactics flag; adventure ruleset part v11: a
//! stack's most creatures) re-recorded all ten with their options, their commands unchanged:
//! every hash moved. CLT-Q4/Q6 (battle state hash v54: the viewers' facts, `cross-lane`)
//! re-recorded all ten with their options, their commands unchanged: nine moved checkpoint hashes
//! taken inside a battle, `duel-s2-d7` none.

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
        files.len() >= 10,
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
