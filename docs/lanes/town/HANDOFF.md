# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 5 closed (T10, T20, T30, T41, T51; `E09-archive.md`, design
  notes Parts A to G). **Increment 6** (the town's trades; decisions 196–224): T52 the survey
  `docs/rules/surveys/E09-trades.md`; T53 the design note's Part H §101 to §120; T54 batch TWN-6
  (survey §14); T55 `shared/` (the ADRs, `TWN-QUIRK-15`'s and `-16`'s flags 54, 55); T56 the town
  part v10 (TWN-TRD-001); T57 the stock (TWN-STK-001, -002); **T58a1** the market at a town
  (TWN-MKT-001 to -004: `TownCommand::Market` opening E08's market decision with the town as its
  place, `h3_adventure::MarketWindow`, the trader's count, the merchants' purchase; `TWN-QUIRK-17`;
  `TWN-STUB-21`, `-22`; decision 224's cut); **T58a2** the listing (`Market` listed where the
  window opens and offers an answer, the env's `TOWN_MARKET` v22, the plans open towns' windows;
  `TWN-STUB-22` lifted; decision 225). **Next:** **T58b** (the artifact sale), then T59 to T61
  (`QUEUE.md`). In the
  inbox for review: E08 T141a's note (2) (the Thieves' Guild's best hero row may follow
  `AdventureState::hero_list`); the AI lane's E31 note (a town's creature upgrade: where
  planned?). For the next `shared/` pull request: ADR-0028 point 6's `marketplaces(store, part,
  player)` is `(store, part, towns)` as built (design §103's *As built*). Waiting elsewhere:
  `TWN-STUB-8` (the battle lane's inbox), `-13` (Visions' reach), `-2` (the victory conditions),
  `-16` (E10's loss condition), `-17` (increment 8), `-18` (E13), `-21` (E10's teams).
- **Open pull requests:** T58a2's (`town/e09-t58a2`, `cross-lane`; its number in the newest
  session entry); watch it until it merges (T58a1's #1202 merged 2026-11-27). The adventure lane's
  #1205 (E08 T144b: the plans, a new workload `random-play-spells-36`) is open beside it: the
  second to merge merges `master` and re-runs `h3-map`'s tests and that workload.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v10: T56; the goldens' town hash
  0xc830faa47b612bfc), both in `h3-towns`; the adventure lane's (game state hash v40 by T58a1,
  map import v29, adventure part v26, adventure state hash v30, replay format v27 by T58a1), the
  env's action encoding (v22 by T58a2) and the protocol's (v85 by T58a1) are bumped `cross-lane`
  (each task's record in `E09-log.md`). Shared: core ruleset hash v53 (T55's base `faithful` hash 0x3f1f68842dab4e0c,
  `strict-fixed` 0x0bd901a144728cc7), pack format v53, quirk flags 55.
- **Ids** (next): questions `TWN-Q38`; decisions 226; quirks `TWN-QUIRK-18` (planned `-18` to
  `-20`, `-22`, `-23`, design §116); stubs `TWN-STUB-23`; experiment batch `TWN-7`; disputes
  `TD-32`; quirk candidates `T-29`. ADR-0028 is the lane's; no drafts open; every TWN-GLD rule
  `verified` but TWN-GLD-007.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-27 — T58a2: the market window listed (`cross-lane`)

Baseline a7b6752 (#1204; CI green; the local full run stopped at coverage for disk, `master`'s
workloads measured instead); 5 files staged. The user kept T58a2 whole (decision 225). The
listing, `MarketParams`' lot tables, the env's v22, seed 864, the plans; one design pass on
`legal-towns-36` (+21.9 % to +0.02 % on `master`'s games), seven town workloads re-baselined
(their games changed course). A hygiene seed `master` fails too went to the adventure lane's
inbox. No sub-agents.

### 2026-11-26 — T58a1: the market at a town (`cross-lane`)

Baseline dc70cf8 (#1198 merged; the setup's baseline died with a VM restart); 5 files staged.
The user cut T58a in two at its listing (decision 224, as T38a). The rule in `h3-towns`,
`h3-adventure`'s markets (a resolved `MarketWindow`), `h3-game`'s `town/market.rs`, the wire v85,
the word `town market` (format v27); the 118 goldens re-recorded once (hash lines, the format and
the game state hash version v40). The scoped verify `--with-original-data` ran (T57's owed step
too). No sub-agents.

### 2026-11-26 — T57: the Artifact Merchants' stock (`cross-lane`)

`master` 6035ed8 was red (T56's `war_machine` against E31 T2d's new literal): fixed first in
#1193 (decision 222; closed when the AI lane's #1192 fixed it first). The user's computer went offline at the start: nothing staged, the scoped
verify without original data (its `--with-original-data` run is owed). The user kept T57 whole
(decision 221) and chose to fix ADV-OBJ-006's restart by class on the way (decision 223). The
118 goldens re-recorded once (hash lines only). No sub-agents.
