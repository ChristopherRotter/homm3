# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 5 closed (T10, T20, T30, T41, T51; `E09-archive.md`, design
  notes Parts A to G). **Increment 6** (the town's trades; decisions 196–227): T52 the survey
  `docs/rules/surveys/E09-trades.md`; T53 the design note's Part H §101 to §120; T54 batch TWN-6
  (survey §14); T55 `shared/` (the ADRs, `TWN-QUIRK-15`'s and `-16`'s flags 54, 55); T56 the town
  part v10 (TWN-TRD-001); T57 the stock (TWN-STK-001, -002); T58 the market at a town
  (TWN-MKT-001 to -005); T59a the war machines and the boat (TWN-SMT-001, TWN-SHY-001, decision
  227); **T59b1** the University (TWN-UNI-001: `University`, T-23 by `TWN-QUIRK-16`, the decision's
  place, decision 228). **Next:** **T59b2** `Transform` (`QUEUE.md`; decision 228's cut), then T60
  (the trades map: the plans meet no boat, no plan town has a spot, and no University), T61. In the inbox for review: E08
  T141a's note (2) (the Thieves' Guild's best hero row may follow `AdventureState::hero_list`);
  the AI lane's E31 note (a town's creature upgrade: where planned?). For the next `shared/` pull
  request: ADR-0028 point 6's `marketplaces(store, part, player)` is `(store, part, towns)` as
  built (design §103's *As built*). For T60: the towns' cover test met one artifact sale (thin).
  Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `-13` (Visions' reach), `-2` (the
  victory conditions), `-16` (E10's loss condition), `-17` (increment 8), `-18` (E13), `-21`
  (E10's teams).
- **Open pull requests:** T59b1's (`town/e09-t59b1`, `cross-lane`; the number in the newest
  session entry); watch it until it merges (T59a's #1221 merged 2026-11-28).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v10: T56; the goldens' town hash
  0xc830faa47b612bfc), both in `h3-towns`; the adventure lane's (game state hash v42 by T59b1,
  map import v30, adventure part v26, adventure state hash v31, replay format v30 by T59b1), the
  env's action encoding (v25 by T59b1) and the protocol's (v88 by T59b1) are bumped `cross-lane`
  (each task's record in `E09-log.md`). Shared: core ruleset hash v53 (T55's base `faithful` hash 0x3f1f68842dab4e0c,
  `strict-fixed` 0x0bd901a144728cc7), pack format v53, quirk flags 55.
- **Ids** (next): questions `TWN-Q38`; decisions 229; quirks planned `TWN-QUIRK-23` (design
  §116), a new one `-24`; stubs `TWN-STUB-23`; experiment batch `TWN-7`; disputes
  `TD-32`; quirk candidates `T-29`. ADR-0028 is the lane's; no drafts open; every TWN-GLD rule
  `verified` but TWN-GLD-007.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-28 — T59b1: the Magic University at a town (`cross-lane`)

Baseline 58142f1 (#1221; the setup's full run stopped for the CPUs once CI's run on `master`
passed its gates, replays and bench); 8f023a7 (#1222) merged in; 7 files staged. The user asked
for a seam in big tasks: T59b cut into T59b1 the University and T59b2 the Transformer (decision
228, as recommended), the map client's two test literals consented. The rule, the place
(`UniversityPlace`, game state hash v42, the 130 goldens re-recorded), the wire v88, the word
format v30, the env's two entries v25, the plans. `legal-towns-36` +0.45 % on `master`'s games
(no University on the town map). No sub-agents. PR and CI: below.

### 2026-11-27 — T59a: the war machines and the boat (`cross-lane`)

Baseline be38c3e (#1216; CI's fingerprint, the setup's light gates); 7 files staged. The user
kept T59a whole (decision 227) over the recommended cut by command. The rules, the game, the
listing (`town/services.rs`), the wire v87, the words (format v29: the goldens' `format` lines),
the env's three entries (v24), the plans (11 machines, no boat: no spot), the town map's seed
128. One design pass on `legal-towns-36` (+2.79 % to +1.98 % on `master`'s games); it and
`week-start-taverns-36` re-baselined (the games changed course). No sub-agents. CI green on #1221
after one re-run (an `h3-server` clock test under coverage).

### 2026-11-27 — T58b: the artifact sale (`cross-lane`)

Baseline 10889f7 (#1211 merged a minute before the start; the setup's run passed its light gates
and tests, then was stopped at coverage for disk; CI's run on `master` failed on an `h3-server`
timing test under coverage, green when re-run); 7 files staged (the harvest now reads
`H3sprite.lod` and `H3ab_spr.lod`: stage them too). The user
kept T58b whole (decision 226) over the recommended cut. The price (0x5ECDC0 read again), the
check, the view, the listing, the wire v86, the word (format v28: the 130 goldens' format lines
only), the env's v23, the plans; a garrison hero's creature sale now tells its town's view (a
T58a1 gap the plans met). Every adventure workload inside its gate, `master` measured the same.
No sub-agents. The scoped verify `--with-original-data` passed (`E09-log.md`).
