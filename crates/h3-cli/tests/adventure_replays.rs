//! Golden adventure replays (E08 T9b, increment 1's exit criteria): every replay under
//! `fixtures/replays/adventure/` must reproduce its recorded state hashes on this platform and
//! build configuration. `cargo xtask determinism` runs this test in debug, release and release
//! with `-C target-cpu=native`; running it on another OS is the cross-platform check. The random
//! replays play the generated duel map `fixtures/maps/e08-duel-36.h3m` (re-record them with
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
//! taken inside a battle, `duel-s2-d7` none. E08 T51b (game state hash v13: the meetings and the
//! army dialogs) re-recorded all ten with their options, their commands unchanged (no game meets
//! an own hero or a join without room): every hash moved. E08 T54 (the battle seam's facts: game
//! state hash v14, adventure state hash v12, map import v11; the duel's players a human may play
//! count as human, so the bad-morale let-off no longer reaches their sides in faithful, which no
//! game's battle met) re-recorded all twelve with their options, their commands unchanged: every
//! hash moved. E08 T60a (game state hash v15: a hero's battle bonuses, none of which a game here
//! gives) re-recorded all twelve with their options, their commands unchanged: every hash moved.
//! E08 T61a (adventure state hash v13, ruleset part v12, map import v12: increment 7's kinds and
//! what objects hold, none of which either map has) re-recorded all twelve with their options,
//! their commands unchanged: every hash moved. E08 T61b (adventure state hash v14, map import v13:
//! the draws when the game is built, the visits' block, the scenario's bans; neither map has an
//! object that draws, nor bans) re-recorded all twelve with their options, their commands
//! unchanged: every hash moved. E08 T62a (game state hash v16: a gift's price and a find's
//! question among the offer kinds, neither map has one) re-recorded all twelve with their options,
//! their commands unchanged: every hash moved. E08 T62b1 (game state hash v17: Pandora's Box's and
//! the Pyramid's questions among the offer kinds, neither map has one) re-recorded all twelve with
//! their options, their commands unchanged: every hash moved. E08 T62b2 (adventure state hash
//! v15: each player's colour and human flag) re-recorded all twelve with their options, their
//! commands and days unchanged: every hash moved. E08 T63a (adventure replay format
//! v7: the answers typed by the decision's kind, `answer offer accept`, `answer level-up first`,
//! `answer close` for a meeting or an army dialog; game state hash v18: the choices and the
//! Universities among the decisions, neither map has one) re-recorded all twelve with their
//! options (the scripted two from their commands in v7's words), their commands unchanged but for
//! their words: every hash moved. E08 T63b (game state hash v19: the rests, GAM-DEC-007, the
//! guard's wait no longer in a decision; a won guards' battle's pickup after the winner's levels)
//! re-recorded all twelve with their options, their commands unchanged: every hash moved.
//!
//! **Scripted** (E08 T52, increment 6: the random agent manages nothing), on the generated
//! `fixtures/maps/e08-army-12.h3m` (red's Valeska, hero 0, with a full army and twelve artifacts
//! in her backpack; her mate Caitlin, hero 2, east of her with expert Scholar, advanced Wisdom,
//! six spells and Elixir of Life's parts worn; six compliant goblins west of her), recorded with
//! `h3 adventure run --map fixtures/maps/e08-army-12.h3m --seed 1 --checkpoint-every 5 --script
//! <script> --record <replay>`; a replay file is its own script (`--script <replay>` re-records
//! its commands when a hash moves):
//!
//! - `army-meeting-s1`: Valeska steps onto Caitlin: Scholar teaches her the five spells of level 2
//!   at most she lacks (HER-SPL-007), the meeting opens (GAM-MET-001, GAM-DEC-003); her Wood
//!   Elves go into Caitlin's empty slot, three Archers onto Caitlin's by a split, Caitlin's
//!   Pikemen and her Swordsmen swap, she dismisses her Monks (ADV-ARM-001 to -004); Centaur's
//!   Axe goes from her backpack to Caitlin's right hand, the Ring of Vitality to her ring and
//!   back (HER-ART-005); Caitlin assembles Elixir of Life, takes it apart and assembles it again
//!   (HER-ART-006); a formation and a flag are set (ADV-ARM-006); the meeting closes (`decline`)
//!   and Valeska walks on; both players end their turns.
//! - `army-dialog-s1`: Valeska steps onto the goblins, whose offer to join she accepts with no
//!   room: the army dialog opens with the six (ADV-MON-008, GAM-DEC-004); she dismisses her Wood
//!   Elves, takes four goblins by a split from the group into the freed slot and moves her Monks
//!   into the group; closed, the group's two goblins and the Monks are lost; both players end
//!   their turns.
//!
//! **Rewards** (E08 T64a, increment 7), on `fixtures/maps/e08-rewards-36.h3m` (every object class
//! of increment 7, written by `h3_map::writer`; `fixtures/maps/README.md`): red's Orrin (hero 0,
//! a full army), Adela (hero 1, basic Wisdom alone, a Spell Book, 950 experience) and Mephala
//! (hero 2), blue's two heroes. Random play, recorded with `--map
//! fixtures/maps/e08-rewards-36.h3m --seed N --agent-seed N --days 28`: the three of a search
//! over the seeds 1 to 400 (a scratch probe noting the objects each game's moves end on and the
//! decisions it opens, not committed; one game visits 3 to 9 of the 31 classes, every class in
//! some game) whose union meets the most:
//!
//! - `rewards-s162-d28`: a Learning Stone's level-up, the box east of Orrin accepted without room
//!   (its army dialog closed, the wood elves lost, then its experience and level-up), a Witch Hut,
//!   a chest's gold, a meeting, Marletto Tower, a School of Magic paid, a random major and minor
//!   artifact picked up, a guard's battle won, blue's event with guards (its battle won, then its
//!   artifact).
//! - `rewards-s243-d28`: the event east of Mephala, a Star Axis, a Spell Scroll, a Shrine, the
//!   pursuit of the fleeing gnolls with an artifact declined (they leave), the event for everyone (morale and
//!   luck), a Warrior's Tomb searched (its artifact and −3 morale), a School of War left, the
//!   Pyramid declined.
//! - `rewards-s88-d28`: a Scholar, a guard's battle and the guarded event after it, Adela on the
//!   guarded Learning Stone (its level-up, then the guard's battle), a School of Magic paid, the
//!   University's four purchases, a chest's experience and its level-up, a Witch Hut.
//!
//! **Scripted** (E08 T64a: the order-dependent paths of design §16), on the same map, recorded with
//! `h3 adventure run --map fixtures/maps/e08-rewards-36.h3m --seed 1 --checkpoint-every 5 --script
//! <script> --record <replay>` (the battles' commands from a scratch driver that attacks or shoots
//! when it can, else defends; not committed); each ends with both players' turns:
//!
//! - `rewards-box-dialog-s1`: Orrin opens the box east of him (ADV-PIK-013) with no room: the army
//!   dialog opens with the ten wood elves (GAM-DEC-004); he dismisses his cavaliers and takes the
//!   elves; only once it is closed does the box's experience come, with its level-up (ADV-RWD-001,
//!   GAM-DEC-007, D-39).
//! - `rewards-event-stop-s1`: Mephala's move east stops on red's event (ADV-MOV-011, ADV-OBJ-007),
//!   which fires (500 gold) and is removed; her next move crosses its tile.
//! - `rewards-university-s1`: Mephala reaches the University in two moves (the second past her
//!   first reveal), buys two of its four skills and leaves; the next day she enters again and only
//!   the other two are offered; she buys one (ADV-TCH-007, GAM-DEC-006).
//! - `rewards-guarded-box-s1`: Adela opens the box on (31, 4); its ten pikemen fight at once, she
//!   wins, and her level-up (advanced Wisdom first) comes before the box's spells: Bless and
//!   Prayer, a level-4 spell her Wisdom now allows (GAM-DEC-007, HER-SPL-005).
//! - `rewards-event-guard-s1`: Orrin steps onto the event on (6, 8): the gnolls guarding it fight
//!   first, then the event fires for him (five crystals; `ADV-QUIRK-29`).
//! - `rewards-stone-guard-s1`: Orrin enters the Learning Stone on (3, 8): its 1 000 experience and
//!   level-up first; only once he answers do the gnolls guarding it fight (ADV-MON-006,
//!   GAM-DEC-007).

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
        files.len() >= 21,
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
