# Quirks ledger: town lane

Known differences between the original game's behaviour and a clean design, in the town lane's rules (a town's record and setup, the derivations over it, the town on the map; E09). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template is in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `TWN-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes"); 1 to 4 are the quirk candidates T-1 to T-4 (E09 design note §19): E09 T7a1 entered 1 and 4, 2 and 3 come with T7a2's checks and T7b's pool.

A candidate becomes an entry here only once it is confirmed for SoD (by an experiment, by original data, or by a code reading that a second look confirms). Until then it waits in its survey and the engine implements the clean behaviour. `exploit-fix` entries and those not reproduced need no confirmation.

## Entries

### TWN-QUIRK-1 — A random town's alignment byte past the colours
- Class: fix, no flag (the user, 2026-11-02, TWN-Q2, T-1: refused, typed; the epic's decision 12)
- Context / rules affected: town, the import of a random town (class 77; E09 T7a1), TWN-SET-002 (a random town's faction; E09 T7b)
- Original behavior: `load_town_body` (0x5019F0) reads SoD's alignment byte after the events and, for class 77, takes the chosen faction of the colour it names when that colour plays (0x5020AB). The byte is a signed index into the playable flags: a byte from 8 to 254 reads past the eight colours (up to 127 forward, 128 to 254 backward into other fields) and may take a faction from another table entry.
- Intended/expected behavior: the import refuses such a town with a typed error (`h3_map::ImportError::Alignment`, the object's index); `0xFF` is no alignment and 0 to 7 a colour.
- Impact: none on the shipped maps (the editor writes 0 to 7 or 255; no shipped map has another byte); a hand-edited map is refused rather than read past the table.
- Community status: VCMI 1.7.5's reader warns on 8 to 15 ("not as player", HotA) and on the rest ("corrupted") and keeps the town as the owner's or random (survey §2.5).
- Sources: code reading of `load_town_body` 0x5019F0 (E09 T1a1; survey `docs/rules/surveys/E09-towns.md` §2.3, §3 T-1).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-map/src/tests/towns.rs`, `a_random_towns_alignment_is_a_colour_or_none`
- Decided by / date: the user, 2026-11-02 (TWN-Q2; the epic's decision 12); entered by E09 T7a1, 2026-11-04

### TWN-QUIRK-4 — The start's draws' empty cases
- Class: fix, no flag (the user, 2026-11-02, TWN-Q2, T-4: typed; the epic's decision 12)
- Context / rules affected: town, TWN-SET-001 (the player's main town; E09 T7a1), TWN-SET-007 and TWN-SET-009 (the draw and the main town's hero; E09 T7b)
- Original behavior: `main_town_heroes` (0x4CA040) looks for the town whose position equals the main town's; when none matches, the town pointer stays null and is used. A draw with no free hero at all (`hero_draw_of_faction` 0x4BB400) calls `random_between(1, 0)` and reads below the list. A faction outside 0 to 8 draws from Castle's classes.
- Intended/expected behavior: each case is typed. The import refuses a main town whose position is no town's entrance (`h3_map::ImportError::MainTown`, the colour; E09 T7a1); a draw with nothing free fails typed (E09 T7b); the faction is always one of the ruleset's with a town record.
- Impact: none on the shipped maps (all 553 main towns match a town; 156 heroes); a malformed map or an exhausted pool fails typed instead of reading memory it should not.
- Community status: VCMI 1.7.5 throws when no hero is left (survey §4.7).
- Sources: code reading of `main_town_heroes` 0x4CA040 (read whole) and `hero_draw_of_faction` 0x4BB400 (E09 T1a2; survey `docs/rules/surveys/E09-towns.md` §4.4, §4.5, §6 T-4).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-map/src/tests/towns.rs`, `a_main_town_is_the_town_whose_entrance_the_header_names` (the import's case; T7b adds the draw's)
- Decided by / date: the user, 2026-11-02 (TWN-Q2; the epic's decision 12); entered by E09 T7a1, 2026-11-04

### TWN-QUIRK-5 — A custom set's horde leaves its dwelling standing
- Class: fix, no flag (the user, 2026-11-03, TWN-Q6, T-6; the epic's decision 29)
- Context / rules affected: town, TWN-SET-003 (the buildings at the start; E09 T6c), TWN-SIE-003 (the keep's count of a siege's turrets)
- Original behavior: `town_buildings_setup` (0x5C08C0) builds a custom set's buildings in id order through `town_build` (0x5BE930), which takes out of the standing set only what is standing when it builds. A horde (18 or 24) comes before the dwelling it supersedes (30 + its level, brought in by the horde's supersede set), so the dwelling's later build removes nothing: both stand. The same buildings built in play (the dwelling first, the horde then replacing it) leave the horde alone. The keep (`town_turret_count` 0x5BF4F0) counts one more structure in such a town, and the town screen draws both.
- Intended/expected behavior: the standing set is derived from the buildings ever built (`h3_towns::Town::standing`: those less every building an ever-built one supersedes), so a horde never leaves its dwelling standing, however it came to be built.
- Impact: gameplay-altering (small): a siege's keep has one structure more in a town whose map's custom set builds a horde (T7a's census counts the shipped maps' cases).
- Community status: unknown.
- Sources: code reading of `town_buildings_setup` 0x5C08C0 and `town_build` 0x5BE930, read whole (E09 T5b1; survey `docs/rules/surveys/E09-towns.md` §16.2, §16.5); VCMI 1.7.5 keeps one set of built buildings and no standing set (survey §16.8).
- Flag: none (fixed in every profile)
- Defaults: faithful = fixed, strict-fixed = fixed
- Regression test: `crates/h3-towns/src/setup/tests.rs`, `twn_quirk_5_a_custom_sets_horde_never_leaves_its_dwelling_standing`
- Decided by / date: the user, 2026-11-03 (TWN-Q6; decision 29)
