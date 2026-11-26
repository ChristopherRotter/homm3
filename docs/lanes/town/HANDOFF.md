# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 5 closed (T10, T20, T30, T41, T51; `E09-archive.md`, design
  notes Parts A to G). **Increment 6** (the town's trades; decisions 196–219): T52 the survey
  `docs/rules/surveys/E09-trades.md` (`TWN-Q35`); T53 the drafts and the design note's Part H §101
  to §120 (`TWN-Q36`, `TWN-Q37`); T54 batch TWN-6 (survey §14: every row the code reading); **T55**
  `shared/`: the drafts in ADR-0028, -0017 and -0016, the glossary, `docs/06-security.md`'s "The
  town's trades"; `TWN-QUIRK-15`'s and `-16`'s flags (`quirk.town.visitor_only_trades` 54,
  `quirk.town.university_ignores_map_bans` 55, on in `faithful`; read from T58a and T59b); the
  combinations' costs (HER-ART-008, `h3-content`); **T56** the town part v10 (TWN-TRD-001: the
  five trade roles, `war_machine`, the trades' parameters, `[town_transformer]`; the harvest
  `cross-lane`). **Next:** **T57**, the stock (`cross-lane`), then T58 to T61 (`QUEUE.md`). In the inbox for review: E08 T141a's note (2) (the Thieves' Guild's best
  hero row may follow `AdventureState::hero_list`); the AI lane's E31 note (a town's creature
  upgrade: where planned?). Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `-13`
  (Visions' reach), `TWN-STUB-2` (the victory conditions), `-16` (E10's loss condition), `-17`
  (increment 8), `-18` (E13).
- **Open pull requests:** T56's **#1189** (`town/e09-t56`, `cross-lane`); watch it until it
  merges (T55's #1181 merged 2026-11-25).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v10: T56; the goldens' town hash
  0xc830faa47b612bfc), both in `h3-towns`; the adventure
  lane's (on `master` e2e15c7: game state hash v39, map import v29, adventure part v26, adventure
  state hash v29, replay format v26), the env's (v20) and the protocol's (v83) are bumped
  `cross-lane` (each task's record in `E09-log.md`). Shared: core ruleset hash v53 (its encoding
  kept; T55's base `faithful` hash 0x3f1f68842dab4e0c, `strict-fixed` 0x0bd901a144728cc7), pack
  format v53 (E08 T124a), quirk flags 55 (T55b; 52 and 53 E14 T2c's).
- **Ids** (next): questions `TWN-Q38`; decisions 221; quirks `TWN-QUIRK-17` (planned `-17` to
  `-23`, design §116); stubs `TWN-STUB-21` (planned for T58a); experiment batch `TWN-7`; disputes
  `TD-32`; quirk candidates `T-29`. ADR-0028 is the lane's; no drafts open; every TWN-GLD rule
  `verified` but TWN-GLD-007.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-26 — T56: the town part v10 (`cross-lane`)

Baseline b1e7a44 (#1182; the setup's light gates, CI green on `master`); 5 files staged. The user
kept T56 whole (decision 220). The part v10 in `h3-towns` and `h3-pack-towns`, two executable
tables read in `h3-formats`, the harvest re-run (only the `towns.toml` changed), the 118 goldens'
town hash replaced (0xc830faa47b612bfc; the rest unchanged). No sub-agents. The coverage run hit
a random case of `h3-map`'s view property failing on `master` too: the adventure lane's inbox.

### 2026-11-25 — T55: increment 6's ADRs, the two flags, the combinations' costs (`shared/`)

Baseline 8abf1f4 (#1173 merged; the setup's full baseline passed, CI green on `master`); 5 files
staged at the end for the original-data step. The user chose T55 whole over the recommended T55a
alone, with sub-agents (decision 219): one applied the three drafts, one re-hashed the 118 golden
adventure replays (only `ruleset_hash`), the server's literal and the bot reports, one reviewed
the ADRs against the drafts (five wordings fixed). E14 T2c (#1174) merged meanwhile with flags 52
and 53: the town's moved to 54 and 55, re-hashed again. The inbox's CI note done (`TRAPS.md`).

### 2026-11-25 — T54: batch TWN-6, run and scored

Baseline 6498273 (#1168 merged; CI green on `master`; the setup's baseline lost to a VM restart).
The user kept T54 whole, `cross-lane` consented, sub-agents where useful (decision 218). The
user's computer was offline for the first hour (specs, recipes and harness code written meanwhile),
then 17 files staged. One sub-agent did the live harness work (the University's dialog, the
town's boat payment, the arrows) and the eight runs; the game's disk check needed a ballast file
outside the repo (`clients/capture/README.md`, "Traps"). Every row the code reading; the agent's
own TWN-6-8 prediction slipped (2.5 for 5 times the cost), noted in §14.
