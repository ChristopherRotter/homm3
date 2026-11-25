# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 5 closed (T10, T20, T30, T41, T51; `E09-archive.md`, design
  notes Parts A to G). **Increment 6** (the town's trades; decisions 196–217): T52 the survey
  `docs/rules/surveys/E09-trades.md` (`TWN-Q35`); **T53** the drafts
  `docs/lanes/town/adr-drafts/{0028,0017,0016}-increment-6.md` and the design note's Part H §101 to
  §120, accepted at `TWN-Q37` (`TWN-Q36`: the market and the University as E08's decisions opened
  by town commands, `BuyWarMachine` one-shot, the stock and the pool's marks in the adventure
  state; `TWN-Q37`: the towns' stream for the month's draw, the combinations' costs in the core's
  compile, the env's 42 entries, the cut §118 and the twelve criteria §119 as written). **Next:**
  **T54**, batch TWN-6 (needs the original data and the client lane's harness; cut at its start),
  then T55 to T61 (`QUEUE.md`). In the inbox for review: E08 T141a's note (2) (the Thieves'
  Guild's best hero row may follow `AdventureState::hero_list`); the AI lane's E31 note (a town's
  creature upgrade: where planned?). Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox),
  `-13` (Visions' reach), `TWN-STUB-2` (the victory conditions), `-16` (E10's loss condition),
  `-17` (increment 8), `-18` (E13). For the next `shared/` task (T55): `docs/06-security.md`'s
  ADV-FOG-006 line (T50's record) with increment 6's lines.
- **Open pull requests:** T53's (`town/e09-t53`, docs only); watch it until it merges (T52's
  #1161 merged 2026-11-24).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure
  lane's (on `master` e2e15c7: game state hash v39, map import v29, adventure part v26, adventure
  state hash v29, replay format v26), the env's (v20) and the protocol's (v82) are bumped
  `cross-lane` (each task's record in `E09-log.md`). Shared: core ruleset hash v53, pack format
  v53 (E08 T124a), quirk flags 51 (E08 T140; `TWN-QUIRK-14`'s at T45).
- **Ids** (next): questions `TWN-Q38`; decisions 218; quirks `TWN-QUIRK-15` (planned `-15` to
  `-23`, design §116); stubs `TWN-STUB-21` (planned for T58a); experiment batch `TWN-6` (its eight
  rows, survey §11); disputes `TD-32`; quirk candidates `T-29`. ADR-0028 is the lane's; drafts
  open: increment 6's three (for T55a); every TWN-GLD rule `verified` but TWN-GLD-007.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-25 — T54: batch TWN-6 (in progress)

Kept whole by the user (decision 218), `cross-lane` consented (h3-experiment, its README, the
harness). Done on `town/e09-t54`: the Conflux's two buildings in the spec; the four specs
`E09-batch-6a` to `-6d` and their recipes; the harness's town clicks (`opens`, `pixel`), screen 3,
the stock and the boat spot (unchecked live). Next: stage the original data (the user's computer
was offline), compile the specs, check the harness live (screen 3's vtable 0x643AAC, the arrows,
the University's items), two runs each, scoring.

### 2026-11-25 — T53: increment 6's drafts, Part H, the cut and the checkpoint, docs only

Baseline 4f2cc71 (#1161 merged; CI green on `master`), `master`'s e2e15c7 merged in. No original
data staged. The user kept T53 whole and asked for sub-agents where it makes sense (decision 208):
one reported the E08 paths the trades reuse (which framed `TWN-Q36`'s four ADR-level choices, each
as recommended), one wrote the three drafts from Part H (its four gaps folded in). `TWN-Q37`
accepted the drafts, the month's stream, the combinations' costs, the env's 42 entries and the cut
(decisions 214–217). The lane's E08 market notes archived, four lanes' inboxes told.

### 2026-11-24 — T52: increment 6's survey and its checkpoint TWN-Q35, docs only

Baseline 7782735 (#1149, #1148 merged; the setup's full baseline on `master`'s code passed). The
user's computer was connected: 5 files staged, SHA-256 checked. The user kept T52 whole and asked
for sub-agents (decision 200): seven narrow jobs (four code readings, VCMI, the wiki, the census),
at most two at a time, each writing a notes file the session merged (46 new map entries, 35
changed). The wiki's Freelancer's Guild, Ballista Yard and Magic University pages did not load.
At the checkpoint the user chose three answers of their own (TD-28 `fix-optional`, the count
at least 1, Misc 5 sellable) and had the agent's calls asked one by one.

### 2026-11-24 — T51: the audit, increment 5 closed, increment 6 refined, docs only

Baseline 25397e7 (#1144 merged). The user's computer was connected: 5 files staged, SHA-256
checked. At the start (`TWN-Q34`, with a first look at the audit) the user kept T51 whole and
split the services: 6 the town's trades, 7 the buildings for heroes and the map, 8 travel and
the Grail (decisions 196–198). The twelve criteria hold; the gap: eight rules batch TWN-5
settled were still `implemented`, now `verified`. The VM restarted once during the setup's
baseline; the scoped verify ran again on the branch (the engine's code `master`'s).
