# Quirks ledger: town lane

Known differences between the original game's behaviour and a clean design, in the town lane's rules (a town's record and setup, the derivations over it, the town on the map; E09). Classes, the decision guide and profiles are in `docs/04-rules-and-sources.md` §4; the entry template is in `docs/rules/quirks-ledger.md` (the battle lane's ledger). Ids are `TWN-QUIRK-<n>` (`docs/08-work-process.md`, "Lanes"); 1 to 4 are reserved for the quirk candidates T-1 to T-4 (E09 design note §19), which E09 T7a enters.

A candidate becomes an entry here only once it is confirmed for SoD (by an experiment, by original data, or by a code reading that a second look confirms). Until then it waits in its survey and the engine implements the clean behaviour. `exploit-fix` entries and those not reproduced need no confirmation.

## Entries

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
