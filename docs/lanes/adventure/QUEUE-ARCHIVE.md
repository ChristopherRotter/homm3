# Adventure lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/adventure/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Completed tasks (the "Tasks" text as it stood on 2026-10-24)

*(Moved 2026-11-19, E08 T132a2: done in T132a1's pull request, decision 381; the record in `E08-log.md`; T132b queued.)* **E08 T132a2 and T132b — the kinds, the packs, the import** (round 5; decisions 370, 373, 378,
380; design note §1 to §7, §19, §21; T132a1, the rates, done)

- **Scope:** T132a2 (`h3-adventure`, `h3-pack-adventure`, the packs): the five kinds `market`,
  `altar`, `hill_fort { gold }` (no `roe_except`: decision 378), `war_machine_factory` and the
  Black Market's stock classes, with their hash tags; `[market]` in the part (its definition by
  resource key, compiled into T132a1's `h3_adventure::MarketParams` with `MarketParamsError`
  mapped, hashed); the base and ab packs' types (§6); the eighth original-data check
  (`--with-original-data`: `MarketParams::factor_bits` and the values against the executable's
  tables, the Hill Fort's factors, the upgrades, the census); adventure ruleset part's next version
  (v23 after the town lane's v22), every golden replay's part hash line rehashed
  (`homm3-rerecord-replays`). T132b (`h3-map`): the import of classes 2, 7, 35, 99, 106, 213 and
  221 at subclass 0 (no body, no RoE mark), the writer's classes for fixtures, a step onto one of
  the seven refused as an unbuilt visit's until its task; map import's next (v27); the shipped-map
  census and the counts that move (`TRAPS.md`).
- **Rules:** ADV-MKT-001, ADV-HFT-001's which stacks, ADV-ALT's values, ADV-WMF-001's machines,
  each with its sources (survey §21's rows as `observation` once built); ADV-MKT-002 is T132a1's.
- **Inputs:** design note §1 to §7, §19; ADR-0016 point 4 and ADR-0017 point 9 (T131); survey §4
  to §14; `TRAPS.md` (shared changes, original data, extra target directories).
- **Depends on:** T131 (#1046) merged; T132a1's pull request. **Status:** ready; T132a2 first,
  T132b after it (each confirmed at its start).

*(Moved 2026-11-19, E08 T132a1: the T132 entry as it stood when T132a was cut in two, decision 380; T132a1 done, the record in `E08-log.md`; T132a2 and T132b queued.)* **E08 T132 — the kinds, the packs, the import** (round 5; decisions 370, 373, 378; design note
§1 to §7, §19, §21)

- **Scope:** T132a (`h3-adventure`, `h3-pack-adventure`, the packs): the five kinds `market`,
  `altar`, `hill_fort { gold }` (no `roe_except`: decision 378), `war_machine_factory` and the
  Black Market's stock classes, with their hash tags; `[market]` and the rates' module with its
  tables (`h3_kernel::arith::X87`, survey §4's halves pinned; D-71's cases); the base and ab packs'
  types (§6); the eighth original-data check (`--with-original-data`); adventure ruleset part's
  next version (v23 after the town lane's v22). T132b (`h3-map`): the import of classes 2, 7, 35,
  99, 106, 213 and 221 at subclass 0 (no body, no RoE mark), the writer's classes for fixtures, a
  step onto one of the seven refused as an unbuilt visit's until its task; map import's next (v27);
  the shipped-map census and the counts that move (`TRAPS.md`).
- **Rules:** ADV-MKT-001, -002, ADV-HFT-001's which stacks, ADV-ALT's values, ADV-WMF-001's
  machines, each with its sources (survey §21's rows as `observation` once built).
- **Inputs:** design note §1 to §7, §19; ADR-0016 point 4 and ADR-0017 point 9 as applied (T131);
  survey §4 to §14; `TRAPS.md` (shared changes, original data, extra target directories).
- **Depends on:** T131 (its `shared/` pull request merged first). **Status:** ready; the cut
  confirmed at its start (T132a and T132b as the note has them, one or both in a session).

*(Moved 2026-11-19, E08 T131: done whole, decision 379; the record in `E08-log.md`; T132 queued.)* **E08 T131 — the `shared/` task** (round 4; decisions 370 to 373, 378, 379; as T122 for
increment 13; `shared/e08-t131`, `cross-lane`)

- **Scope:** design note `E08-design.md` §21's shared work. The two accepted drafts applied
  (`adr-drafts/0016-`, `0017-increment-14.md`: ADR-0016 points 1, 4, 6; ADR-0017 points 1, 6, 8
  to 10) with batch E14's outcome (survey `E08-trades.md` §21: the rows cited; the Hill Fort's RoE
  mark withdrawn, the packs deciding, decision 378), then the drafts deleted;
  `docs/03-domain.md`'s terms (market, a market's lot, the Black Market's stock, a transfer, a
  sacrifice, the Hill Fort's upgrade, the War Machine Factory); `docs/06-security.md`'s lines (the
  amounts, the recipient and the price validated; the stock server-only; design §12, §17); the
  index's Read-when cells; the town lane's inbox told the ADRs' new parts.
- **Inputs:** the drafts; design §12, §17, §21; T122's record in `E08-log.md` as the form.
- **Depends on:** T130 (done). **Status:** ready; whole (decision 379, docs only); then T132
  (round 5, its cut at its start).

*(Moved 2026-11-19, E08 T130: done whole, decisions 378, 379; the record in `E08-log.md`; survey `E08-trades.md` §21; T131 queued.)* **E08 T130 — batch E14 scored** (round 3; decisions 368, 373; T129a and T129b the runs, done)

- **Scope:** the survey `docs/rules/surveys/E08-trades.md`'s new §21: each of E14-1 to E14-17
  scored against the reading (§3 to §14) and the specs' predictions (their headers): as predicted,
  or the finding and its effect on the design note `E08-design.md` and the drafts
  `adr-drafts/0016-`, `0017-increment-14.md` (T131 applies them). The results:
  `experiments/results/E08-batch-14.md` (T129b's run, every section) and `-14-roe.md`; T129a's
  run of the markets in `E08-log.md` (the draws differ by start).
- **The finding to settle first:** E14-14: the game's version (game +0x1F698) reads SoD on the
  RoE-format map (as batch E7a's RoE map), so its Air Elementals were offered and upgraded; the
  reading's exclusion (`hill_fort_rows`, survey §13) keys on game version 0, which a new game of
  an RoE map does not take on this executable. Design note §4 and exit criterion 6 ("not an RoE
  map's elementals") rest on it: read where +0x1F698 is set at a new game, compare the wiki and
  VCMI; a question for the user if they disagree (gameplay impact).
- **Inputs:** T129b's record in `E08-log.md` (the raw observations, the notes), the specs'
  headers, survey §18.
- **Then T131** (`shared/`, `cross-lane`).
- **Depends on:** T129b (done). **Status:** ready (docs; the executable for the +0x1F698 reading).

*(Moved 2026-11-18, E08 T126: the "Tasks" text's lead with increment 13's chronicle, and the T126 entry as they stood before T126 closed increment 13; done whole, decision 360; the record in `E08-log.md`; T127a and T127b queued.)* Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T117 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 12 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11, -11-15; increment 12, garrisons and the bonus visitables, by T117's audit). **Increment 13, creature banks**, refined with the user at T117 (ADV-Q52; decisions 333, 334): round 1, the survey (T118a, done 2026-11-15: decisions 335 to 339, its checkpoint ADV-Q53) and the ADR amendment drafts (T118b1 and T118b2 done 2026-11-15, decisions 340 to 342; the checkpoint ADV-Q54, decisions 343 to 345: the note, the drafts and the cut T119 to T126 accepted). Round 2: T119 done 2026-11-15 (`adventure/e08-t119`, `cross-lane`, whole: decision 346; the writer's banks, the spec's pictures and `slots`, batch E13's spec). Round 3: T120 done 2026-11-16 (`adventure/e08-t120`, alone: decision 347; the harness's reads, the recipe, three runs, the template filled); T121 done 2026-11-16 (`adventure/e08-t121`, whole, docs only: survey `E08-banks.md` §14, every row as predicted; the checkpoint ADV-Q55, decisions 348, 349). Round 4: T122 done 2026-11-16 (`shared/e08-t122`, `cross-lane`, whole: decision 350; the three drafts in ADR-0013, -0016, -0017, -0022 with batch E13's rows, the glossary, the security lines, `BattleSetup::creature_bank`). Round 5: T123a done 2026-11-16 (`adventure/e08-t123a`, `cross-lane`, alone: decision 351; the kind, the packs, the seventh check; part v21); T123b done 2026-11-16 (`adventure/e08-t123b`, `cross-lane`, whole: decision 352; the import, the census, map import v24). Round 6: T124a done 2026-11-17 (`adventure/e08-t124a`, `cross-lane`, whole: decision 353; the bank battle in `h3-battle`, the battle pack's `bank_positions`); T124b in two (decision 354): T124b1 done 2026-11-17 (`adventure/e08-t124b1`, `cross-lane`, alone: the state and the draw, adventure state hash v26); T124b2 done 2026-11-17 (`adventure/e08-t124b2`, whole: decision 355; the visit, the battle, the reward and the once-marks). Round 7: T125a done 2026-11-17 (`adventure/e08-t125a`, whole: decision 356; the hint and its events, the Shipwreck's passage); T125b in two (decision 357): T125b1 done 2026-11-18 (`adventure/e08-t125b1`, `cross-lane`: the two banks maps, nine golden replays, the server lane's counts); T125b2 in two (decision 358): T125b2a done 2026-11-18 (`adventure/e08-t125b2a`: the banks in the plans' properties, the views, the cover test); T125b2b done 2026-11-18 (`adventure/e08-t125b2b`, whole: decision 359; the three workloads, the original data); round 7 done. Round 8: T126 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T126 — increment 13's audit** (round 8; design `E08-design.md` §20's exit criteria)

- **Scope:** (`adventure/e08-t126`): the audit against the nine exit criteria with a fresh
  reviewer; increment 13 closed (its scope, decisions 333 to 359 and tasks to `E08-archive.md`,
  the design note to `E08-log.md`; the README's epic table in the session's `shared/` pull
  request); increment 14 refined with the user (the sketch's item 14, quests, or another order).
  Criterion 9's benchmarks: T125b2b baselined the three new workloads and ran every other inside
  its gate (its record in `E08-log.md`).
- **Inputs:** design §20 (the exit criteria), §17's and §9's *As built*; T117's audit
  (`E08-log.md`) as the form; the epic's sketch.
- **Depends on:** T125b2b (`adventure/e08-t125b2b`, merged first). **Status:** ready once
  T125b2b merges (the original data to stage for criterion 1's census and criterion 9's gates);
  its cut confirmed at its start.

*(Moved 2026-11-16, E08 T123b: done whole, decision 352, `cross-lane` for `h3-server`'s `tests::record`; the record in `E08-log.md`; T124 queued.)* **E08 T123b — the import** (round 5; design `E08-design.md` §3, §18; as T113b for increment 12)

- **Scope:** `adventure/e08-t123b`: `codes::object_type`'s class 16 by subclass 0 to 6 to the
  seven bank types and 85, 24, 84, 25 at subclass 0 to theirs (another subclass
  `ImportError::Subclass`; no body, `Holds::None`), the census (`h3-map`'s `original_data`:
  the 2 137 banks of survey §1's table typed), the typed classes' tests (a blocker case of
  another class if one used a bank class), a step onto a bank refused as any unbuilt visit's
  until T124b (a state test, the reach ending before it), `h3m.md`'s import; map import v24 (the
  goldens' `import_version`, checkpoints and final hashes; the server lane's `tests::record` if
  its replay moves, `cross-lane`). The writer already writes the five classes (T119's `BANK_OBJECTS`). The
  rules: ADV-OBJ-002 amended, ADV-BNK-001's import part (`verified` by batch E13 already).
- **Inputs:** design §3; survey `E08-banks.md` §1; T113b's record in `E08-log.md` as the form; the
  town lane's inbox note below (its T37b and T39a come after T123b's map import v24).
- **Depends on:** T123a (`adventure/e08-t123a`). **Status:** ready once T123a merges. The census
  needs `maps.tar` staged.

*(Moved 2026-11-16, E08 T123a: T123a done alone, decision 351, `cross-lane` for the AI lane's two bots tables; the record in `E08-log.md`; T123b queued alone.)* **E08 T123 — the kind, the packs, the check; the import and the writer** (round 5; design
`E08-design.md` §1 to §3, §18; as T113a and T113b for increment 12)

- **Scope:** T123a (`adventure/e08-t123a`, likely `cross-lane` for the base pack's
  `adventure.toml` and `h3-pack-adventure`'s check): the kind `bank { question, levels, looted,
  deck, coast }` in `h3-adventure` (ADV-BNK-001; hash tag 44; the compile's checks, design §1),
  the eleven types in the base pack with survey §2's table (D-65 the original) and the
  executable's creature tables, the seventh original-data check against `CrBanks.txt` and
  0x6702A0 / 0x67037C (design §2; adventure ruleset part v21). T123b (`adventure/e08-t123b`):
  `codes::object_type`'s class 16 by subclass 0 to 6 and 85, 24, 84, 25 (another subclass
  `ImportError::Subclass`, no body), the writer's classes for fixtures, the census; a step onto a
  bank refused as any unbuilt visit's until T124b (design §3; map import v24). The rules'
  `observation` sources from batch E13 (survey §14: ADV-BNK-001 by E13-1, E13-2, E13-11, E13-5,
  E13-12).
- **Inputs:** design §1 to §3, §18; survey `E08-banks.md` §1, §2, §14; T113a's and T113b's records
  in `E08-log.md` as the form; the town lane's inbox note below (its T37b and T39a come after
  T123's map import v24).
- **Depends on:** T122 (merged). **Status:** ready; the cut confirmed at its start (T123a and
  T123b apart, as T113, or whole). The check needs `CrBanks.txt` (AB's extracted `H3ab_bmp`) and
  `Heroes3.exe` staged.

*(Moved 2026-11-16, E08 T122: done whole, decision 350; the record in `E08-log.md`; only two literals were exhaustive, so the AI lane was not told; T123 queued.)* **E08 T122 — the `shared/` task** (round 4; decisions 338, 343, 345, 348; as T112 for
increment 12; `shared/e08-t122`, `cross-lane`)

- **Scope:** design note `E08-design.md` §20's shared work. The three accepted drafts applied
  (`adr-drafts/0016-`, `0017-`, `0022-increment-13.md`: ADR-0016 points 1, 4, 6; ADR-0017 points
  1, 2, 4, 6, 9 to 11; ADR-0022 points 2, 3, 6, 7 and the new point 8, with ADR-0013 point 5's
  line) with batch E13's outcome (survey `E08-banks.md` §14: the rows cited; ADR-0022's
  alternatives gain VCMI's second table for two-hex guards, decision 348), then the drafts
  deleted; the seam's field `BattleSetup::creature_bank: bool` (`h3-contracts`) with every
  exhaustive literal's `false` (mechanical, its own commit; the battle and AI lanes told first);
  `docs/03-domain.md`'s terms (creature bank, a bank's level, looted, bank battle, bank
  positions); `docs/06-security.md`'s lines (design §16).
- **Inputs:** the drafts; design §13, §16, §20; T112's record in `E08-log.md` as the form; the
  town lane's inbox note below (its T35 amends ADR-0016 and -0017 too: whichever merges first,
  the other rebases).
- **Depends on:** T121 (done). **Status:** ready; the cut confirmed at its start (whole as T112,
  or the ADRs and docs first, then the seam's field). Nothing staged.

*(Moved 2026-11-16, E08 T121: done whole, docs only; the record in `E08-log.md`; the survey section is §14, not §15: the survey ended at §13; T122 queued.)* **E08 T121 — batch E13 scored; the checkpoint** (round 3; decisions 339, 345, 347; as T111 for
increment 12; docs only)

- **Scope:** survey `E08-banks.md` §15, each row's verdict against the spec's predictions (the
  spec's `predictions` per encounter; design §19's changes); D-65, D-66, D-68, D-69 and the new
  rows' findings (E13-11, E13-12); the consensus rows' rules toward `verified` at T123 and T124;
  the checkpoint with the user (the drafts' changes for T122, any new dispute or quirk).
- **Inputs:** `experiments/results/E08-batch-13.md` (the third run's template filled; its
  `anything odd` lines), T120's record in `E08-log.md` (the three runs, their draws, the raw
  observations and the harness's fixes between them), T111's record as the form.
- **Depends on:** T120 (done). **Status:** ready (nothing staged: the records are in the repo;
  the executable only for a reading a verdict needs).

*(Moved 2026-11-16, E08 T120: T120 done alone, decision 347; the record in `E08-log.md`, T121 queued alone.)* **E08 T120 + T121 — batch E13's harness, recipe and run; scored** (round 3; decisions 339,
345, 346; as T110 and T111 for increment 12)

- **Scope:** design note `E08-design.md` §19's harness and §20's round 3. **T120** (the capture
  harness, `clients/capture/`, common): the reads batch E13's spec asks for (the banks' records
  at game +0x4E3D8, 0x6C bytes each, and their objects' words; a hero's +0x105 bits 9 to 11 and
  +0x11A, E12's reads may serve; the hint over a tile; the backpack's order; the battle's stacks
  with every hex they cover at the opening, the tactics phase, the obstacles, the war machines,
  the battlefield, a stack's shots before and after a shot; the questions, messages and dialogs
  in order, the army dialog's two bars; Space on a hero's tile), the recipe
  `clients/capture/batches/E08-batch-13.toml` with its choices by the day-1 records (E13-7's
  Utopia, E13-10's Imp Cache) and its battle steps (E13-3's Defends and the Archers' shot,
  E13-9's shot and retreat), the run(s), the template filled into
  `experiments/results/E08-batch-13.md`. **T121** (docs): survey `E08-banks.md` §15, each row's
  verdict against the predictions; D-65, D-66, D-68, D-69 and the new rows' findings; the
  consensus rows' rules toward `verified` at T123 and T124; the checkpoint with the user.
- **Inputs:** `experiments/specs/E08-batch-13.toml` (compiled with `h3-experiment`, stamp in the
  template), T119's record in `E08-log.md`, T110's and T111's records as the form; the client
  lane's `clients/capture/README.md`.
- **Depends on:** T119 (done). **Status:** ready (the original data: the executable, the LODs,
  both tars, the client lane's harness setup; confirm the cut at its start).

*(Moved 2026-11-15, E08 T119: done whole, decision 346; the record in `E08-log.md`, T120 + T121 queued.)* **E08 T119 — batch E13's map: the writer's five classes, the spec's fields, the spec** (round 2;
decisions 339, 343, 345; as T109 for increment 12)

- **Scope:** design note `E08-design.md` §19, §20's round 2: `h3_map::writer` writes the five
  bank classes by template (class 16 by subclass 0 to 6, 85, 24, 84, 25; no body);
  `h3-experiment`'s spec gains them by template and what its adventure hero lacks at the start
  (secondary skills, war machines and artifacts: check the battle hero's fields first), a boat
  for E13-5 (battle-owned: `cross-lane` with the user's consent, label it); the spec and the
  template of run **E13** (survey §11's E13-1 to E13-10 with §19's changes: E13-3 extended,
  E13-6 corrected, E13-11 and E13-12 added; two players, days 1 to 3).
- **Inputs:** survey `docs/rules/surveys/E08-banks.md` §1 (the templates), §11 (the rows); design
  note §19; T109a's and T109b's records in `E08-log.md` as the form.
- **Depends on:** T118b2 (done). **Status:** ready (the original data: the executable and both
  tars, to compile the map and check it in the editor's reader; confirm the cut at its start).

*(Moved 2026-11-15, E08 T118b2: done whole, decision 341; D-67 corrected, decision 342; the checkpoint ADV-Q54, decisions 343–345; the record in `E08-log.md`.)* **E08 T118b2 — the game side of the design note, ADR-0017's draft and the cut** (decisions 334,
340; as T98b2 for increment 11)

- **Scope:** `E08-design.md` part D (from §13): the bank battle's seam (`BattleSetup` in
  `h3-contracts`, a `shared/` change first, decision 338; its ADR draft, likely ADR-0022's, which
  holds the setup's anti-magic flag) and the battle crate's rules (no tactics phase, no
  obstacles, no war machines on the field, the layout by the stacks' ordinals, D-69), set from
  part B §7's `BattleStart::bank`; D-67's exception to GAM-DEC-007 point 3 (the winner's levels
  after the reward and its army dialog, part B §8); GAM-BAT-001 (the deck, §7); the env, replays,
  maps, properties and benchmarks (the shipped-map workloads, §9's hot path, a bank XL); versions;
  batch E13's spec and recipe in the cut (survey §11 with part D's additions listed at the note's
  end); the `shared/` and `cross-lane` work, the building tasks' cut, the exit criteria, the
  risks; `adr-drafts/0017-increment-13.md`; then the checkpoint with the user (the whole note, the
  drafts, the agent's calls of parts A to D, the cut).
- **Inputs:** `E08-design.md` parts A to C and `adr-drafts/0016-increment-13.md` (T118b1); the
  survey `docs/rules/surveys/E08-banks.md` (§5, §6, §9 to §11); decisions 333 to 340; ADR-0017,
  ADR-0013 point 5, ADR-0022, ADR-0023 (the seam's fields), ADR-0027 (the setup's adventure
  facts); the battle lane's inbox note of 2026-11-15 and any answer; T108b's 0017 draft as the
  form (`gh api` at 6276854100).
- **Depends on:** T118b1 (done). **Status:** ready (docs only).

*(Moved 2026-11-15, E08 T117: the "Tasks" text's lead with increment 12's chronicle, and the T117 entry as they stood before T117 closed increment 12; done whole, decision 332; the record in `E08-log.md`.)* Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T107 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 11 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11; increment 11, the teleporters, by T107's audit). **Increment 12, garrisons and the bonus visitables**, refined with the user at T107 (ADV-Q48; decisions 295, 296): round 1 done: T108a in two (decision 297): T108a1 (`adventure/e08-t108a`: survey §0 to §9, the census of all seventeen classes and the garrisons), T108a2 (`adventure/e08-t108a2`, whole: decision 298; survey §10 to §14, the checkpoint ADV-Q49, decisions 299–303); T108b (`adventure/e08-t108b`, whole: decision 304; the design note and the three drafts, the checkpoint ADV-Q50, decisions 305–309, the cut T109 to T117). Round 2: T109 in two at the code's seam (decision 310): T109a done (`adventure/e08-t109a`, `cross-lane`: the writer's seventeen classes, the spec's garrisons; no spell points, decision 311); T109b done (`adventure/e08-t109b`, `cross-lane`, whole: decision 312; the specs E12a and E12b, the School of War in the writer, decision 313; E12-5 on the start's reveal, decision 314); round 2 done. Round 3: T110 done (`adventure/e08-t110`, whole: decision 316; the harness's reads and steps, the recipes `E08-batch-12a` and `-12b`, three runs each, the templates filled); T111 done (`adventure/e08-t111`, whole, docs only: survey §15, every row as read; the visit modes by the class lists, the checkpoint ADV-Q51, decision 317); round 3 done. Round 4: T112 done (`shared/e08-t112`, `cross-lane`, whole: decision 318; the three drafts in ADR-0013, -0016, -0017 with batch E12's rows, the glossary, the security lines, `EffectSource::Visit`'s doc comment, the experiments README's visit modes); round 4 done. Round 5: T113a done (`adventure/e08-t113a`, `cross-lane`, alone: decision 319; the kinds, the packs, `kind_mark`, the check against the original data; part v20); T113b done (`adventure/e08-t113b`, whole: decisions 321, 322; the import, `Holds::Garrison`, the census, the grounds' twins checked; map import v22, adventure state hash v22); round 5 done. Round 6: T114a in two at the exchange (decision 323): T114a1 done (`adventure/e08-t114a1`: the visit, the battle, the capture, anti-magic, the passage; no version moved); T114a2 done (`adventure/e08-t114a2`, `cross-lane`, whole: decision 324; the exchange, the owner rule; game state hash v29, replay format v13, protocol v52); T114b done (`adventure/e08-t114b`, `cross-lane`, whole: decision 325; every viewer's map garrisons, `GarrisonChanged`, the fold; protocol v54); round 6 done. Round 7: T115a done (`adventure/e08-t115a`, `cross-lane`, whole: decision 326; the battle bonuses and timed marks of the heroes on the map, their inputs, the clear and the drops; adventure state hash v23, game state hash v31, protocol v57); T115b done (`adventure/e08-t115b` after `shared/e08-t115b-sirens` #909, `cross-lane`, whole: decision 327; the fifteen's visits, the blocks, the Sirens, the kind marks, the weekday; adventure state hash v24, protocol v61); round 7 done. Round 8: T116 in three (decision 328): T116a done (`adventure/e08-t116a`, `cross-lane`: the garrisons and bonuses maps, fourteen golden replays, the server lane's counts); T116b in two (decision 329): T116b1 done (`adventure/e08-t116b1`: the garrisons in the plans' properties, the hygiene property's two-way monolith); T116b2 done (`adventure/e08-t116b2`, whole: decision 330; the fifteen in the plans' properties, the inbox's case first); T116c done (`adventure/e08-t116c`, whole: decision 331; the garrisons and bonuses XL, the four workloads, the scoped verify with the original data); T117 next. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T117 — increment 12's audit** (round 8; design §19's exit criteria)

- **Scope:** the audit against the ten exit criteria with a fresh reviewer; increment 12 closed
  (its scope, decisions 295 to 331 and tasks to `E08-archive.md`, the design note to
  `E08-log.md`); increment 13 refined with the user (the sketch's item 13, creature banks, or
  another order). Criterion 10's benchmarks: T116c's run found the fork workloads up to
  +1.99 % above their baselines (`fork-rewards-36` +1.75 %, `fork-dwellings-xl-144` +1.99 %, its
  count the same in a `master` c05bccd worktree: the state other lanes added since), so a fork's
  next cost trips the gate: compare with `master` and re-baseline with the reason
  (`homm3-bench-regression`). **Depends on:** T116c
  (done). **Status:** ready (original data to stage for criterion 1's census and criterion 10's
  gates).

*(Moved 2026-11-11, E08 T109b: done whole, decision 312; the School of War in the writer, decision 313; E12-5 on the start's reveal, decision 314; the record in `E08-log.md`, T110 queued.)* **E08 T109b — batch E12's specs and templates: E12a and E12b** (decisions 303, 309, 310)

- **Scope:** as T99b for increment 11 (design note `E08-design.md` §18, §19's round 2): the specs
  and templates of runs **E12a** (E12-1 to E12-6, the garrisons, day 1, two players) and **E12b**
  (E12-7 to E12-17, the bonus visitables, days 1 to 7 and the next week's first, one player), with
  the predictions added in §18; each walk priced on the written map (TRAPS: price an experiment's
  walks; the seventeen classes import as blockers until T113b, so a step onto one is priced by
  hand) and a fresh review, as T99b1's. The writer and the spec's fields are T109a's (done): the
  garrisons in `[[map.ownable]]` with `owner`, `army`, `removable`; the fifteen in `[[map.object]]`
  by picture (`docs/experiments/README.md`, "Increment 12's objects"); a Blue hero is an
  `[[encounter.also]]`; no spell points at the start (decision 311).
- **Inputs:** survey `docs/rules/surveys/E08-garrisons-bonuses.md` §1, §12 (the rows), §3 to §7
  and §10 for the predictions; design note §18; T99b1's and T99b2's records in `E08-log.md` as the
  form; T109a's record.
- **Depends on:** T109a (done). **Status:** ready (the original data: the executable and both
  tars, to compile the maps; confirm the cut at its start: likely in two by spec, E12a then
  E12b, as decision 277).

*(Moved 2026-11-11, E08 T109a: the entry as it stood before T109 was cut in two at the code's seam, decision 310; T109a's record in `E08-log.md`, T109b queued.)* **E08 T109 — batch E12's maps: the writer's seventeen classes, the spec's fields, the specs** (decisions 303, 309)

- **Scope:** as T99 for increment 11 (design note `E08-design.md` §18, §19's round 2): `h3_map::writer`
  writes the garrisons (33, 219 by subclass: the owner, the seven stacks, the removable byte, the
  unused bytes) and the fifteen bonus classes by template (no body); `h3-experiment`'s spec gains
  the garrisons with an owner, an army and the removable byte, the fifteen classes by template, a
  hero's spell points and army at the start, a second player's hero (battle-owned: `cross-lane`
  with the user's consent, label it); the specs and templates of runs **E12a** (E12-1 to E12-6,
  day 1, two players) and **E12b** (E12-7 to E12-17, days 1 to 7 and the next week's first, one
  player), with the predictions added in §18.
- **Inputs:** survey `docs/rules/surveys/E08-garrisons-bonuses.md` §1 (the templates), §2 (the
  garrison's body), §12 (the rows); design note §18; T99's record in `E08-log.md` as the form.
- **Depends on:** T108b (done). **Status:** ready (the original data: the executable and both
  tars, to compile the maps and check them in the editor's reader; confirm the cut at its start).

*(Moved 2026-11-11, E08 T108b: done whole, decision 304; the checkpoint ADV-Q50, decisions 305–309; the record in `E08-log.md`.)* **E08 T108b — the ADR amendment drafts, the design note and the cut** (decision 296)

- **Scope:** as T98b for increment 11: the drafts `adr-drafts/0016-increment-12.md` and
  `0017-increment-12.md` (and `0013-increment-12.md` for the twelve new once-marks and their
  amounts, survey §10.3), increment 12's design note `E08-design.md` (its parts by side), the
  building tasks' cut and the exit criteria; the checkpoint with the user (the note, the drafts,
  the agent's calls, the cut).
- **Inputs:** survey `docs/rules/surveys/E08-garrisons-bonuses.md` (§9.3 and §13 the proposed
  ids, §11 the interfaces' sketch, §10.9 the open points left to T108b: where the Well's day mark
  and the Stables' week mark live, the movement's maximum raised with the points, the shape of the
  once-marks' amounts); decisions 295 to 303 (the checkpoint's answers: the original in D-59 to
  D-64, A-44's `fix-optional` flag and A-45 fixed, the kind marks in, batch E12 before the build);
  ADR-0013 (`BattleBonuses`' eight marks), ADR-0015 to -0017; the client lane's inbox note of
  2026-11-03 (`EffectSource::Visit`); the town lane's two of 2026-11-11 (E09 T20's ask and T21's
  proposal: `ArmyRef::Garrison(town)` and a sibling for a map garrison, one owner rule; answer it
  in the drafts and tell the town lane's inbox; T108a's findings are in its inbox); T98b's drafts as the form.
- **Depends on:** T108a2 (done). **Status:** ready (docs only; the original data not needed).

*(Moved 2026-11-11, E08 T108a2: done whole, decision 298; the checkpoint ADV-Q49, decisions 299–303; the record in `E08-log.md`.)* **E08 T108a2 — the bonus visitables survey; the batch's plan, the ids, the checkpoint** (decisions 296,
297; survey `docs/rules/surveys/E08-garrisons-bonuses.md` §10 to §14)

- **Scope:** the fifteen bonus visitables (11, 14, 28, 30, 31, 38, 48, 49, 52, 56, 64, 92, 94, 96,
  110; census in survey §1.2): each one's gain, its duration (to the next battle, the day's or the
  week's end) and its marks (once per hero, a day, a week; the hero's +0x105 bits and the tiles'
  words), how the bonuses reach the battle (morale, luck) and the movement and spell points, the
  Stables' Cavaliers, the Sirens' losses and experience, the Idol's and the Temple's days, the
  objects with several entrances (48, 56, 110); what a viewer sees (a visit's marks, the hint's
  texts); VCMI's and the wiki's rules; disputes from D-62, quirk candidates from A-44; then for
  both families the interfaces' sketch (commands, decisions, events, the observation, replays,
  benchmarks), batch E12's plan on the client lane's harness (survey §9.4's garrison rows with the
  bonus ones), the proposed rule ids (§9.3's family ADV-GAR with the bonus ones) and the
  checkpoint's questions (D-59 to D-61, §9.2's open points, the bonus visitables'), asked at its
  end.
- **Inputs:** survey §0 to §9 (T108a1); decisions 295 to 297; the handlers measured at T108a's
  start (about 4.9 KB): `visit_object`'s inline cases 11, 14, 28 and 52 and the handlers of 30 to 110
  (`visit_fountain_of_fortune` 0x4A2480 to `visit_watering_hole` 0x4A7EA0 in the address map,
  named from the switch; the Sirens' and the Stables' with three small callees); read before: `hero_morale_description`
  0x4DC320 and `hero_luck_description` 0x4DCAC0 (the visits' bits), `hero_battle_bonuses_clear`
  0x4DA510, `new_day` 0x4C7FE0 and the new week 0x4C8780, `hero_max_land_movement` 0x4E4990 (the
  Stables' bit 1); the battle's morale and luck inputs (HER-BAT-003, HER-ADV-008) and the inbox's
  note of 2026-11-03 (CLT-Q40: the visits by once-mark, `EffectSource::Visit`); increment 7's
  battle bonuses (the Warrior's Tomb, the Pyramid); ADR-0013 (the hero's derived properties);
  ADR-0015 to -0017; the lane's `TRAPS.md`.
- **Depends on:** T108a1 (done). **Status:** ready (the original data staged: `Heroes3.exe` for the
  reading, the shipped maps for the batch's plan).

*(Moved 2026-11-11, E08 T108a1: its entry as it stood before the user cut it in two, decision 297; T108a2 queued in its place; the record in `E08-log.md`.)* **E08 T108a — the garrisons and bonus visitables survey** (decision 296; as T98a for increment 11: survey file
`docs/rules/surveys/E08-garrisons-bonuses.md`, or one per family if T108a is cut by family, the agent's call)

- **Scope:** the census of classes 33 and 219 and of the fifteen bonus visitables (11, 14, 28, 30,
  31, 38, 48, 49, 52, 56, 64, 92, 94, 96, 110) on the 160 shipped maps (by subclass, owner and
  army; T97c's counts in `E08-log.md`, T97c); their `.h3m` bodies and templates
  (`docs/formats/h3m.md`, VCMI 1.7.5's reader); the original's code (`docs/exe/README.md`,
  `scripts/exe.py`): a garrison's owner, army and flag, the passage (the side it is entered from,
  the step through, a hero standing in it), the exchange with a hero of its owner, the battle
  against its army (owned or neutral; 219's without spells), its capture and Leave Guards; each
  bonus visitable's gain, its duration (to the next battle, the day's or the week's end) and its
  marks (once per hero, a day, a week), how the bonuses reach the battle (morale, luck) and the
  movement and spell points, the Stables' Cavaliers, the Sirens' losses and experience, the
  Idol's and the Temple's days; what a viewer sees (a garrison's army, a visit's marks); VCMI's and
  the wiki's rules; disputes `D-<n>` from 59, reading quirk candidates `A-<n>` from 44; the
  proposed rule ids; the batch's plan (E12) on the client lane's harness (decision 296's agent's
  call). Likely in parts by family (the garrisons; the bonus visitables, perhaps in two),
  confirmed with the user at its start.
- **Inputs:** decisions 295, 296; ADR-0015 to -0017 (ADR-0016 points 3, 4 and 6, ADR-0017's
  decisions and rests), ADR-0013 (the hero's derived properties); the battle's morale and luck
  inputs (HER-BAT-003, HER-ADV-008) and the inbox's note of 2026-11-03 (CLT-Q40: the visits by
  once-mark, `EffectSource::Visit`); increment 9's owned objects (flags, Leave Guards: ADV-MIN-002)
  and increment 6's army dialog; a battle without spells is the battle lane's (its queue, with
  the banks' flag of the sketch's item 13); the lane's `TRAPS.md`.
- **Depends on:** T107 (done). **Status:** ready (the original data and the shipped maps are needed:
  staging per `docs/08-work-process.md`, "Cloud sandbox").

*(Moved 2026-11-11, E08 T107: the "Tasks" text's lead with increment 11's chronicle, and the T107 entry as they stood before T107 closed increment 11.)* Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T97c (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 10 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05; increment 10, dwellings and hiring, by T97's audit). **Increment 11, the teleporters**, refined with the user at T97c (ADV-Q44; decisions 260, 261): round 1, the survey and the ADR amendment drafts first: T98a in two (decision 262), both done (T98a2 with the checkpoint ADV-Q45, decisions 263–269); T98b in two by side (decision 270): T98b1 done (`adventure/e08-t98b1`: design parts A and B, ADR-0016's draft), T98b2 done (whole: decision 271; part C, ADR-0017's draft, the checkpoint ADV-Q46, decisions 272–274: the note, both drafts and the cut accepted). Rounds 2 to 8 (design §17): T99 in two at the code's seam (decision 275): T99a done (`adventure/e08-t99a`, `cross-lane`: the writer's five classes, the spec's `level`, the gates' pairs, the boats on Whirlpools' tiles); T99b in two by land and water (decision 276): T99b1 done (`adventure/e08-t99b1`: 11a's land rows), T99b2 in two by spec (decision 277): T99b2a done (`adventure/e08-t99b2a`: 11a's Whirlpool rows), T99b2b done (`adventure/e08-t99b2b`, `cross-lane`: the spec E11b, the batch's README row); round 2 done. Round 3: T100 in two by spec (decision 278): T100a done (`adventure/e08-t100a`: the harness's replay, branch and box reads, the recipe `E08-batch-11a`, three runs, the template filled), T100b done (`adventure/e08-t100b`: the key repeated, the Whirlpools' list and the tiles' reads, the recipe `E08-batch-11b`, three runs, the template filled); T101 done (`claude/epic-brahmagupta-rp3lje`, whole, docs only: survey §15, every row as read but E11-19; the checkpoint ADV-Q47, decisions 279, 280); round 3 done. Round 4: T102 done (`shared/e08-t102`, `cross-lane`, whole: decision 281; the ADRs, the glossary, the security lines, the effect kind `whirlpool_protection`); round 4 done. Round 5: T103 in four (decision 282): T103a done (`adventure/e08-t103a`, `cross-lane`: the kinds, the packs and their check, `whirlpool_protection`; part v18; the bot reports' line, decision 283); T103b done (`adventure/e08-t103b`: the import of the five classes, A-39 refused, `ADV-QUIRK-41`; map import v21, with the inbox's class 220 by the mines' table; the census of the shipped maps); T103c done (`adventure/e08-t103c`, `cross-lane` for the experiments README's row, whole: decision 284; batch E11c, E11-20 to E11-23 every row as read, survey §15.1); T103d done (`adventure/e08-t103d`, whole: the guard first for every entered kind, `Rest::Visit`; no version moved); round 5 done. Round 6: T104 done (`adventure/e08-t104`, whole: decision 285; A-42's refusal, the channels and the gates' pairs, `build-gates-xl-144`; the shipped maps' census brought forward, decision 286); round 6 done. Round 7: T105, T105a in two by land and water (decision 287): T105a1 done (`adventure/e08-t105a1`: the stream in `apply`, the monoliths' and the gates' jumps, `ADV-STUB-16` for an occupied exit; no version moved); T105a2 done (`adventure/e08-t105a2`: the Whirlpool's loss and jump with the boat, `WhirlpoolLoss`; no version moved); T105b in two (decision 288): T105b1 done (`adventure/e08-t105b1`: the battle and the meeting at an exit, `Rest::Jump`; game state hash v27), T105b2 done (`adventure/e08-t105b2`, `cross-lane`, whole: decision 289; `Revisit`, replay format v12, the env's action encoding v8; seven workloads re-baselined); round 7 done. Round 8: T106 in three (decision 290, as T86 and T96): T106a done (`adventure/e08-t106a`: the map `e08-teleporters-36`, three random and the scripted replays); T106b in two (decision 291): T106b1 done (`adventure/e08-t106b1`: the plans' teleporters, `Step::Jump`, the `play.rs` checks, a cover test); T106b2 done (`adventure/e08-t106b2`, whole: decision 292; the `view.rs` checks, `vary`'s teleporters); T106c done (`adventure/e08-t106c`, whole: decision 293; the census checked whole, the XL `e08-teleporters-144`, `reach-teleporters-xl-144` and `random-play-teleporters-36`); round 8 done. T107 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T107 — increment 11's audit** (round 8; design §17's exit criteria)

- **Scope:** the audit against the ten exit criteria with a fresh reviewer; increment 11 closed
  (its scope, decisions 260 to 293 and tasks to `E08-archive.md`); increment 12 refined with the
  user. **Depends on:** T106c (done). **Status:** ready.

*(Moved 2026-11-06, E08 T99a: the entry as it stood before T99 was cut in two at the code's seam, decision 275; T99a's record in `E08-log.md`, T99b queued.)* **E08 T99 — batch E11's maps: the writer's five classes, the spec's fields, the two specs**
(decisions 269, 274; as T89 for batch E10)

- **Scope:** `h3_map::writer` writes classes 43 to 45 (by subclass, the channel), 103 and 111 on
  either level, without a body, as the import reads them (a round trip; its class table checked),
  for the batch and for T106's fixtures; the spec's fields in `h3-experiment` (`cross-lane`, the
  user's consent as T89's): the five classes by picture and subclass (`[[map.object]]`), a hero in
  a boat on or beside a Whirlpool's tile and an empty boat on one (batch E8's boats), the worn
  Hats (batch E8's artifacts), the checks of E11-9's walk order and truncated distances and of
  E11-16's and E11-17's tiles; the specs `experiments/specs/E08-batch-11a.toml` (E11-1 to E11-13,
  E11-18, E11-19; days 1 and 2) and `-11b.toml` (E11-14 to E11-17; one day) with their templates
  and predictions (survey §12's rows, design §16's added row and predictions); each walk priced
  (`TRAPS.md`).
- **Inputs:** `E08-design.md` §16 (and §1, §4 for the classes); the survey
  `docs/rules/surveys/E08-teleporters.md` §12 (the rows, what the spec lacks), §1 to §5; T89a's and
  T89b's records in `E08-log.md` (batch E10's writer and specs); `docs/experiments/README.md`.
- **Depends on:** T98b2 and its checkpoint ADV-Q46 (done). **Status:** ready; its cut confirmed
  with the user at its start. Needs the original data staged for the spec's templates (as T89).

*(Moved 2026-11-06, E08 T98b2: its entry as it stood before it was done whole, decision 271; the record in `E08-log.md`, T99 queued.)* **E08 T98b2 — the game side of the design note, ADR-0017's draft, the cut and the checkpoint**
(decisions 261, 270; as T88b2)

- **Scope:** `E08-design.md` part C (§12 to §18, named by T98b1): the battle and the meeting at an
  exit (GAM-BAT-004, GAM-MET-001 amended: the defender on the exit, where it is fought, the
  radius-1 reveal's place); the jump held as the visit's rest (GAM-DEC-007) and the visitor's
  stream passed to `AdventureState::apply` (ADR-0017 point 4; no draw for one candidate, survey
  §4.1); `Revisit`'s name, validation and the env's action (decision 268, the AI lane's encoding,
  `cross-lane`), with its map-side lines added to `adr-drafts/0016-increment-11.md`; the env,
  replays, fixtures, properties and benchmarks (the reach's and the passage's cost planned against
  the gated workloads, `homm3-bench-regression`; a teleporters' map and workload; the gates'
  pairing cost); versions, rule ids and the quirk entries (design §11's proposed ids); batch E11's
  rows (survey §12, plus a row for design §7's entrance guards) with the spec's and harness's
  needs; the building rounds cut, batch E11's spec and recipe first (decision 269); shared and
  `cross-lane` work (the effect kind `whirlpool_protection`, design §2); the exit criteria and
  risks. The draft `adr-drafts/0017-increment-11.md`. **The checkpoint** (a new `ADV-Q46`): the
  user accepts the note, both drafts and the agent's calls (design §1, §2, §5's A-42, §7's empty
  boat and entrance guards, §10's events) and the cut before anything is built (decision 261).
- **Inputs:** `docs/epics/E08-design.md` parts A and B; `adr-drafts/0016-increment-11.md`; the
  survey `docs/rules/surveys/E08-teleporters.md` (§4.2 to §4.5, §9 to §13); decisions 260, 261,
  264–270; increment 10's part C and ADR-0017 draft as the model (`E08-log.md`, "Increment 10's
  design note at its closure", §12 to §18).
- **Depends on:** T98b1 (done). **Status:** ready; its cut confirmed with the user at its start.
  Needs no original data (the survey holds the readings).

*(Moved 2026-11-06, E08 T98b1: the entry as it stood before T98b was cut in two by side, decision 270; T98b1's record in `E08-log.md`, T98b2 queued.)* **E08 T98b — the ADR amendment drafts, the design note and the cut** (decision 261; as T88b)

- **Scope:** the drafts in `docs/lanes/adventure/adr-drafts/`, `E08-design.md` written for increment
  11 (its pointer replaced; increment 10's note in `E08-log.md` as the model), the reach's and the
  passage's cost planned against the gated workloads (the hot path, `homm3-bench-regression`), the
  building rounds cut, batch E11 first (decision 269: its spec and recipe the first round); the
  draw's stream (ADR-0017's streams; no draw for one candidate, survey §4.1); `Revisit`'s name,
  validation and the env's action (decision 268, the AI lane's encoding, `cross-lane`); A-37's
  battle as a visit's rest (decision 265); the quirk ledger's entries for A-37 to A-41; the
  fixtures' and tests' blockers of the five classes (TRAPS); the checkpoint's question: the user
  accepts the drafts before anything is built (decision 261).
- **Inputs:** the survey `docs/rules/surveys/E08-teleporters.md` (§9's reach, §10's views, §11's
  interfaces' sketch, §12's batch plan, §13's proposed ids, §14's answers); decisions 260, 261,
  264–269.
- **Depends on:** T98a (done). **Status:** ready; its cut confirmed with the user at its start. Needs no original data (the survey holds the readings).

*(Moved 2026-11-06, E08 T98a2: done whole, decision 263; the checkpoint ADV-Q45, decisions 264–269; the record in `E08-log.md`.)* **E08 T98a2 — the teleporters survey, its second part** (decisions 261, 262; survey
`docs/rules/surveys/E08-teleporters.md` §9 to §14)

- **Scope:** the reach and the paths through a teleporter (the original's: `pathfinder_pass_through`
  reads the monolith lists at 0x56AE39 and 0x56AE81, whose paths those are; the engine's reach and
  passage, the hot path); the space key's visit (`key_space_visit` 0x408E65: its full gate, which
  classes, RoE) and whether the increment takes it as a command; what a viewer sees of a hero
  passing (`hero_relocate_inner` 0x49CF50, the network message, the wiki's exit shown in the
  shroud); the interfaces' sketch (commands, decisions, events, the observation, replays,
  benchmarks); batch E11's plan on the client lane's harness (decision 261's agent's call), with
  rows for the points resting on the reading alone (survey §8's open points); the proposed rule ids;
  the checkpoint's questions (D-53 to D-57, A-37's fix, A-38, A-39, A-40), asked at its end.
- **Inputs:** survey §0 to §8 (T98a1); decisions 260 to 262; ADR-0015 to -0017 (ADR-0016 points 3
  and 4, ADR-0017's streams); the lane's `TRAPS.md` (a new read in the reach's or the passage's
  loops; a test's blocker of a class that gets a kind); the inbox's performance note of 2026-10-23
  (a lifted hero's reach); surveys `E08-hero-movement.md` (the reach, flying, A-13) and `E08-sea.md`
  (a boat's entrances).
- **Depends on:** T98a1 (done). **Status:** ready (the original data staged: `Heroes3.exe` for the
  reading, the shipped maps for the batch's plan).

*(Moved 2026-11-05, E08 T98a1: its entry as it stood before the user cut it in two, decision 262; T98a2 queued in its place; the record in `E08-log.md`.)* **E08 T98a — the teleporters survey** (decision 261; as T88a for increment 10: survey file
`docs/rules/surveys/E08-teleporters.md`)

- **Scope:** the census of classes 103, 43 to 45 and 111 on the 160 shipped maps (by subclass and
  level, the gates' pairs, each channel's entrances and exits, the Whirlpools' waters; T97c's counts
  in `E08-log.md`, T97c); their `.h3m` bodies and templates (`docs/formats/h3m.md`, VCMI 1.7.5's
  reader); the original's code (a step onto an entrance: the exit and how it is chosen, a random
  exit's draw and its stream, an exit that is occupied or guarded, the points after the jump,
  whether a move goes on through it, the paths and the reach through a teleporter, a Whirlpool's
  losses and its hero in a boat, what the fog reveals at the exit; `docs/exe/README.md`,
  `scripts/exe.py`), VCMI's and the wiki's rules; disputes `D-<n>` from 53, reading quirk
  candidates `A-<n>` from 37; the proposed rule ids; the batch's plan (E11) on the client lane's
  harness (decision 261's agent's call); what a viewer sees of a hero passing. Likely in two (T98a1
  the objects, the census and the jump; T98a2 the reach and the paths, the views, the batch's plan
  and the checkpoint), confirmed with the user at its start.
- **Inputs:** decisions 260, 261; ADR-0015 to -0017 (ADR-0016 points 3 and 4, ADR-0017's streams);
  the lane's `TRAPS.md` (a new read in the reach's or the passage's loops; a test's blocker of a
  class that gets a kind); the inbox's performance note of 2026-10-23
  (a lifted hero's reach); surveys `E08-hero-movement.md` (the reach, flying, A-13) and `E08-sea.md`
  (a boat's entrances).
- **Depends on:** T97c (done). **Status:** ready (the original data and the shipped maps are needed:
  staging per `docs/08-work-process.md`, "Cloud sandbox").

*(Moved 2026-11-05, E08 T97c: the "Tasks" text's lead with increment 10's chronicle, and the T97 entry as they stood before T97c closed increment 10.)* Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T88a (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 9 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03; increment 9, mines, the week and owned objects, by T87's audit). **Increment 10, dwellings and hiring**, refined with the user at T87 (ADV-Q41; decisions 231, 232): round 1, the survey and the ADR amendment drafts first: T88a done (whole, decision 233; survey `docs/rules/surveys/E08-dwellings.md`, its checkpoint ADV-Q42: decisions 234–237). T88b in two by side (decision 239; D-51 settled as the original, decision 238): T88b1 done (`adventure/e08-t88b1`: `E08-design.md` parts A and B, ADR-0016's draft), T88b2 done whole (decision 240; `adventure/e08-t88b2`: part C, ADR-0017's draft, the cut; its checkpoint ADV-Q43, decisions 241–243). The cut (design §17): rounds 2 to 8, T89 to T97, batch E10 first. T89 in two at the code's seam (decision 244): T89a done (`adventure/e08-t89a`: the writer's classes and teams, the spec's fields, `cross-lane`), T89b done whole (decision 246; `adventure/e08-t89b`, `cross-lane`: the specs `E08-batch-10a` and `-10b`). T90 done (`adventure/e08-t90`: the harness reads and both batches' runs), T91 done whole (decision 247; `adventure/e08-t91`: survey §16, every row as read). T92 done whole (decision 247; `shared/e08-t92`: the ADRs, the glossary, the security lines). Round 5: T93 in two at the design's seam (decision 248): T93a done whole (`adventure/e08-t93a`, `cross-lane`, decision 249: the kinds, the parameters, the table, the packs, the check against the original data; part v16); T93b done whole (decision 250; `adventure/e08-t93b`: the import, map import v16). Round 6: T94 in two at the design's seam (decision 251): T94a done (`adventure/e08-t94a`: the build, the refusals, the resolution, the lists, the census); T94b done whole (decision 252; `adventure/e08-t94b`: the dwellings' and camps' week state, adventure state hash v20). Round 7: T95a in two at the guards (decision 253): T95a1 done (`adventure/e08-t95a1`: the visits without a battle, the hire question, the free creatures, the camp's visit, the writer); T95a2 done whole (decision 254; `adventure/e08-t95a2`: the guards' question, their battle, its end, the flag and the hire question after a win); T95b done whole (decision 255; `adventure/e08-t95b`, `cross-lane`: the purchase, the hire dialog, the env's v7, replay format v10, game state hash v25). Round 8: T96 in three (decision 256, as increment 9's T86): T96a done (`adventure/e08-t96a`: the map `e08-dwellings-36`, its three random and five scripted replays); T96b1 done whole (decision 257; `adventure/e08-t96b1`: the plans' dwellings, their checks and cover test, the three failing `view.rs` seeds fixed in the test); T96b2 done whole (`adventure/e08-t96b2`, `shared/e08-t96b2-benches`: the map `e08-dwellings-144`, its three workloads, the census's entrance check and most per map); T97 in three (decision 258): T97a done (`adventure/e08-t97a`: the audit, its gaps fixed); T97b in two (decision 259): T97b1 done (#742, `adventure/e08-t97b1`: survey §17, the dwellings entered from water imported, a tied random dwelling's checks, map import v19); T97b2 done (`adventure/e08-t97b2`: the plans' dwellings on water, the boat plans' cover and view tests, `check_told`'s heroes stepping into sight); T97c next. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T97 — the audit; increment 10 closed, increment 11 refined** (design §17, the exit criteria; decision 256; in three, decision 258: **T97a** done, the audit, `adventure/e08-t97a`; **T97b** the dwellings entered from water, in two (decision 259): **T97b1** done, `adventure/e08-t97b1`; **T97b2** done, `adventure/e08-t97b2`; **T97c** the closure and the refinement)

- **Scope:** increment 10 against its eight exit criteria (design §17), as T87 audited increment 9: the rules' ids, sources and tests, the quirks and stubs, the gates; **the four shipped maps refused as `OwnedOnWater`** (a dwelling entered from water: Adventures of Jared Haret, Rise of the Phoenix and its Allied version, Serpents Treasure; the inbox's note of 2026-11-04): whether such a dwelling is built is a sourced rule, settled here or carried into increment 11 (the typed refusal stays meanwhile); the increment's scope, decisions and task list moved to `E08-archive.md`; increment 11 refined with the user (the epic's sketch).
- **T97c, the closure and the refinement:** increment 10's scope, decisions and task list to `E08-archive.md`, the design note to the log; increment 11 refined with the user (ADV-Q44; the epic's sketch item 11).
- **Depends on:** T96b2 (done); each part on the one before. **Status:** T97c ready (T97b2's pull request merged first; docs and the user's refinement).

*(Moved 2026-11-03, E08 T89a: the entry as it stood before T89 was cut in two at the code's seam, decision 244; T89a's record in `E08-log.md`, T89b queued.)* **E08 T89 — batch E10's maps and specs** (decisions 237, 242; as T78 for batch E9)

- **Scope:** `h3_map::writer` writes classes 17 and 20 (by subclass, with an owner), 78 and 216
  to 218 (a faction set, levels, a town identifier of 0) as the import reads them (a round trip),
  for the batch and for T96's fixtures; the spec's fields in `h3-experiment` (`cross-lane`, the
  user's consent as T78's): dwellings with an owner, random dwellings, Refugee Camps, a hero's full
  army of named stacks, teams in a hot-seat map (E10-6), a town for each player in E10b; the specs
  `experiments/specs/E08-batch-10a.toml` and `-10b.toml` with their templates and predictions
  (survey §13's rows without E10-2, design §16's two added); each walk priced (`TRAPS.md`).
- **Inputs:** `E08-design.md` §16 (and §1, §4 for the classes); the survey
  `docs/rules/surveys/E08-dwellings.md` §13 (the rows, what the spec lacks), §2–§7; T78's record
  in `E08-log.md` (batch E9's writer and spec); `docs/experiments/README.md`.
- **Depends on:** T88b2 and its checkpoint ADV-Q43 (done). **Status:** ready; its cut confirmed
  with the user at its start.

*(Moved 2026-11-03, E08 T88b2: its entry as it stood before it was done whole, decision 240; the record in `E08-log.md`, T89 queued.)* **E08 T88b2 — the game side, ADR-0017's draft and the cut** (decisions 232, 239; as T77b)

- **Scope:** `E08-design.md` part C (its §12 to §18 as part B's end lists them: the guards' battle,
  the hire dialog as a pending decision with GAM-DEC-008, its commands and events, the env,
  replays, fixtures, properties and benchmarks, versions, batch E10, the shared and `cross-lane`
  work, the building rounds and the exit criteria, risks); ADR-0017's draft in
  `docs/lanes/adventure/adr-drafts/` (points 1, 3, 4, 6, 8, 10, 11 as increment 9's touched: the
  commands, the new week's order, the build's and the week's draws, the dialog, the env, the
  outcome's composition, the guards' battle unowned); the building rounds cut, batch E10 first
  (decision 237: its spec, the harness's reads, its two runs without E10-2, then its scoring); the
  checkpoint's question (ADV-Q43): the user accepts both drafts and the note before anything is
  built (decision 232).
- **Inputs:** `E08-design.md` parts A and B (T88b1), `adr-drafts/0016-increment-10.md`; the survey
  `docs/rules/surveys/E08-dwellings.md` (§5 the dialog, §12 the interfaces' sketch, §13 batch
  E10's plan, §14 the ids); decisions 231–239.
- **Depends on:** T88b1 (done). **Status:** ready; its cut confirmed with the user at its start.

*(Moved 2026-11-03, E08 T88b1: the entry as it stood before T88b was cut in two by side, decision 239; T88b1's record in `E08-log.md`, T88b2 queued.)* **E08 T88b — the ADR amendment drafts, the design note and the cut** (decision 232; as T77b)

- **Scope:** the drafts in `docs/lanes/adventure/adr-drafts/`, `E08-design.md` written for increment
  10 (its pointer replaced; increment 9's note in `E08-log.md` as the model), the building rounds
  cut, batch E10 first (decision 237: its spec, the harness's reads, its two runs, then its
  scoring), the D-51 quirk's flag named (decision 234); the checkpoint's question (ADV-Q43): the
  user accepts the drafts before anything is built (decision 232).
- **Inputs:** the survey `docs/rules/surveys/E08-dwellings.md` (§12 the interfaces' sketch, §13
  batch E10's plan, §14 the proposed ids, §15 the answers); decisions 231–237.
- **Depends on:** T88a (done). **Status:** ready; its cut confirmed with the user at its start.

*(Moved 2026-11-03, E08 T88a: its entry as it stood before it was done whole, decision 233; the record in `E08-log.md`.)* **E08 T88a — the dwellings and hiring survey** (decision 232; as T77a for increment 9: survey file
`docs/rules/surveys/E08-dwellings.md`)

- **Scope:** the census of classes 17, 20, 216 to 218 and 78 on the 160 shipped maps (by subclass,
  the owners at load, a random dwelling's faction set, level range and linked town; T76's counts in
  `E08-log.md`, T76: 216 on 22 maps, 217 on 75, 218 on 1; the town lane's 1 397 tied dwellings on
  87 maps); their `.h3m` bodies (`docs/formats/h3m.md`, VCMI 1.7.5's map reader); the original's
  code (the dwellings' visit and flag, the hire dialog, the weekly population, the Refugee Camp's
  weekly creature, a dwelling's guards; `docs/exe/README.md`, `scripts/exe.py`; the town lane's
  readings in `docs/rules/surveys/E09-towns.md` §1.6, §2.1, §2.3, and `random_dwellings_resolve`
  0x502B60), VCMI's and the wiki's rules; disputes `D-<n>` from 51, reading quirk candidates `A-<n>`
  from 34; the proposed rule ids; the batch's plan on the client lane's harness (decision 232's
  agent's call); what a player sees. Likely in two (T88a1 the objects and the census, T88a2 the
  hiring, the week and the checkpoint), confirmed with the user at its start.
- **Inputs:** decisions 231, 232; the inbox's town-lane notes (2026-10-31 on the hire dialog and the
  weekly growth, 2026-11-02 on the random dwellings); E09 decision 4; ADR-0015 to -0017.
- **Depends on:** T87c (done). **Status:** ready (the original data and the shipped maps are needed:
  staging per `docs/08-work-process.md`, "Cloud sandbox").

*(Moved 2026-11-03, E08 T87c: the "Tasks" text's lead, the rest of increment 9's chronicle, and the T87 entry as they stood before T87c closed increment 9.)* Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T86b1 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 8 are closed. **Increment 9, mines, the week and owned objects** (ADV-Q36; decisions 192, 193): rounds 1 to 7 done, and round 8's T86 (decision 226): T86a (the economy map and its replays), T86b1 (the plans' properties) and T86b2 (decision 228: the economy XL, the day start's workloads, the re-baselines; `adventure/e08-t86b2` with `shared/e08-t86b2`); T87 in three (decision 230): T87a and T87b done, T87c left. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T87 — the audit; increment 9 closed; increment 10 refined** (decision 230: in three, each
branch based on the one before without waiting for its merge; **T87a** done, decision 229's boat
tile, `adventure/e08-t87a` with `shared/e08-t87a`; **T87b** done, the audit, `adventure/e08-t87b`)

- **T87c, the closure and the refinement:** increment 9 closed (its scope, decisions and task
  list to `E08-archive.md`, its design note to `E08-log.md`, a row in the closed increments'
  table); increment 10 refined with the user (the sketch's item 10: dwellings and hiring, the
  Refugee Camp, garrisons; the town lane's inbox notes on the hire dialog and the random
  dwellings). Branch `adventure/e08-t87c` on `adventure/e08-t87b`.
- **Inputs:** `E08-design.md` §18, §19; `E08-log.md` T86a, T86b1, T86b2, T87a, T87b; the epic's
  decisions 192 to 230; the inbox below (the town lane's and the map client's notes).

*(Moved 2026-11-03, E08 T86b2: the "Tasks" text's increment 9 chronicle as it stood before T86b2.)* Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T76 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 8 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30; increment 8, boats and the sea, by T76's audit). **Increment 9, mines, the week and owned objects**, refined with the user at T76 (ADV-Q36; decisions 192, 193): the survey and the ADR amendment drafts first (T77a, T77b), each task's cut confirmed with the user at its start. T77a in two (decision 194): T77a1 done (`adventure/e08-t77a1`: survey `E08-economy.md` §0 to §6), T77a2 done whole (decision 195; `adventure/e08-t77a2`: §7 to §18, the checkpoint's `ADV-Q37`). T77b done whole (decision 200; `adventure/e08-t77b`: the drafts, `E08-design.md`, the cut in rounds 2 to 8, decision 202; the checkpoint's `ADV-Q38`, decisions 201–204). T78 done whole (decision 205; `adventure/e08-t78`, `cross-lane`, decision 206: the writer's ownable objects, weekly objects and timed events, the spec's fields, the specs `E08-batch-9a` and `-9b`). T79 in two by batch (decision 207): T79a
done (`adventure/e08-t79a`: the harness's mine records, tile words, boat dialog, Leave Guards,
battle sides and `auto again`; the recipe `E08-batch-9a` and its runs); T79b done whole (decision
208; `adventure/e08-t79b`: the harness's income, Garden mask, weekly objects' and monsters' words,
end-turn messages; the recipe `E08-batch-9b` and its runs). T80 done (decisions 209, 210: whole
with A-33's re-read; `adventure/e08-t80`, `cross-lane`: batch E9 scored, survey §19, A-33
withdrawn). T81 in two, the ADRs first (decision 211): T81a done (`shared/e08-t81a`: the three
drafts applied, ADR-0015's core row, the index, the glossary); T81b done (decision 212: whole;
`shared/e08-t81b`, `cross-lane`: the four kinds, the two parameters, the creature's abilities
outside battle, the packs; core ruleset hash v51, pack format v50). Round 5: T82 and T83a done
together (decision 213; `adventure/e08-t82-t83a`, `cross-lane`, decision 214: the day's income,
spell points and the Lighthouses' input; the seven kinds and parameters, the import), T83b in two by what reads it (decision 215): T83b1 done
(`adventure/e08-t83b1`, `cross-lane`, decision 217: the owners, guards, weekly contents, draws,
the start's reveal, the flag radius), T83b2 in two, the state first (decision 218): T83b2a done
(`adventure/e08-t83b2a`: the fractions, the added objects, `check_added`, adventure state hash
v19), T83b2b done (decision 219: whole, the `shared/` part first, a mine's guards as the
original's hint; `adventure/e08-t83b2b` with `shared/e08-t83b2b`: the views). Round 6: T84 in two
halves, the mines first (decision 220): T84a done (`adventure/e08-t84a` with `shared/e08-t84a`:
the flags, the mines' visits, Leave Guards, their battles; game state hash v21); T84b in two,
the Lighthouses and the weekly visits first (decision 221): T84b1 done (`adventure/e08-t84b1`
with `shared/e08-t84b1`: the Lighthouses' flag and points, the weekly visits); T84b2 done
(`adventure/e08-t84b2` with `shared/e08-t84b2`: the Shipyards, their boat offer and `BuildBoat`,
adventure replay format v8; the user's decision 222: a Garden marks the team). Round 7: T85 in
two halves, T85a whole (decision 223): T85a done (`adventure/e08-t85a` with `shared/e08-t85a`:
the week's step, a creature month's step, the audit of the static reads). T85b on one branch
over two sessions (decision 224): T85b1 done on `adventure/e08-t85b` (pushed, no pull request);
T85b2 in two (decision 225): T85b2a done on the same branch (the timed events, `events_today`;
the security lines on `shared/e08-t85b`); T85b2b done on it (the env's `BuildBoat`, action
encoding v6, `cross-lane`; the verify with the original data, the bench; the pull requests,
`shared/` first: the lane HANDOFF names them). Round 8: T86 in two (decision 226): T86a done
(`adventure/e08-t86a`: the economy map, its seven replays, the duel replays' lists, four
replaced); T86b in two, the properties first (decision 227): T86b1 done (`adventure/e08-t86b1`:
the plans' economy classes and steps, design §15's checks, the town lane's failing case).
Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`. Increment 1 (T1a–T10c, decisions 1–12) closed on 2026-10-12. **Increment 2** refined with the user the same day (T10c, ADV-Q9; decisions 13–18 in the epic): object types, the treasury, pickups, the Cartographer as the first pending decision, a few visitable objects. The user takes one or two tasks per round, the cut confirmed at each round's checkpoint. T11a, T11b (ADV-Q10 answered, the ADRs amended), T12, T13 and T14 (each in two parts) are done; T15 and T16 (T16a–T16c) are done; **increment 2 closed** on 2026-10-13. **Increment 3** (monsters and battles) refined with the user the same day (decisions 19–21): T17a done (the survey), T17b done (the ADR amendment drafts, the design note, the cut below; accepted with the user's changes at ADV-Q13, decisions 25–27). Round 1 trimmed by the user on 2026-10-14 (decision 28) and done: T18a (#185, `cross-lane`) and T18b (#188, `cross-lane`). Round 2 split by the user the same day (decision 29): T19a1 done (`shared/e08-t19a`, `adventure/e08-t19a1`, `cross-lane`), then `shared/e08-t19a2` alone (decision 31), T19a2 split again (decision 32): T19a2a done (`adventure/e08-t19a2a`, `cross-lane`), T19a2b done (decision 33: `shared/e08-t19a2b` and `adventure/e08-t19a2b`, both `cross-lane`); round 2 closed. Round 3 split by the user (decision 34): T19b1 done (`adventure/e08-t19b1`, `cross-lane`: two shared docs), T19b2 done (decision 35: with T20a1; `shared/e08-t19b2` #213 first, `h3-battle` as `h3-map`'s dev-dependency; #214); round 3 closed. Round 4: T20a split (decision 35), T20a1 done (`adventure/e08-t20a1` #215, `cross-lane`); T20a2 split again (decision 36), T20a2a and T20a2b done (`adventure/e08-t20a2a`, `adventure/e08-t20a2b`); T20b split in three (decision 37), T20b1 and T20b2 done (`adventure/e08-t20b1` #228, `shared/e08-t20b2` #230 `cross-lane`, `adventure/e08-t20b2` #231); T20b3 split (decision 38), T20b3a done (`adventure/e08-t20b3a` #234); T20b3b split (decision 39), T20b3b1 done (`adventure/e08-t20b3b1` #236, decision 40); T21 split and its rule part moved before T20b3b2 (decision 41), T21a done (`adventure/e08-t21a` #239), T20b3b2 done (`adventure/e08-t20b3b2` #240, with guarded piles); round 4 closed; T21b done alone with a shared docs pull request (decision 42: `shared/e08-t21b-docs`, `adventure/e08-t21b`); T22a split in three (decision 43), T22a1 done (`adventure/e08-t22a1`, `cross-lane`: the tools); T22a2 split by rows (decision 44), T22a2a done (`adventure/e08-t22a2a`, `cross-lane`: the monster rows' batches E3a and E3b with their recipes); T22a2b done (decision 45: `adventure/e08-t22a2b`, `cross-lane`: the hero rows' batch E3c with its recipe; the inbox's four rows to increment 4's batch); T22a3 done (decision 46: `adventure/e08-t22a3`, `cross-lane`: E3a, E3b and E3c run on the harness, its auto combat waiting for a stack's turn); T22b done (decision 47: `adventure/e08-t22b`, `cross-lane`: batch E3 scored, E3-9 read, `ADV-QUIRK-8`); **increment 3 closed** on 2026-10-20. **Increment 4** (hero progression only) refined with the user the same day (T23 done, decisions 48–50: `adventure/e08-t23`; the inbox sorted); its first round is T24a, then T24b (decision 49); T24a cut in two by the user (decision 51): T24a1 done (`adventure/e08-t24a1`: survey §1–§2); T24a2 cut again (decision 52): T24a2a done (`adventure/e08-t24a2a`: survey §3–§4); T24a2b cut (decision 53): T24a2b1 done (`adventure/e08-t24a2b1`: survey §5–§6, the wiki and the census), T24a2b2 done (decision 54, `adventure/e08-t24a2b2`: survey §7–§9; the checkpoint's questions ADV-Q15); **T24a done**; ADV-Q15 answered (decision 55), T24b cut by crate (decision 56): T24b1 done (`adventure/e08-t24b1`: ADR-0013's draft, the design note's part A), T24b2 done whole (decision 57, `adventure/e08-t24b2`: ADR-0017's draft, part B, the cut); **T24b done**; ADV-Q16 answered (decision 58: the drafts accepted, the cut confirmed); round 1 cut (decision 59): T25 done (`adventure/e08-t25`: survey §10; ADV-Q17, decision 60), T26a done (`shared/e08-t26a` #325: the ADRs amended); T26b done whole (decision 61: `shared/e08-t26b` #335, `cross-lane`); **round 1 closed**; round 2 cut (decision 62): T27 done whole (`adventure/e08-t27`, `cross-lane`); T28 done (`adventure/e08-t28`); **round 2 closed**; round 3 cut (decision 63): T29 done whole (`adventure/e08-t29`, `cross-lane`); T30 done whole (decision 64, `adventure/e08-t30`: with the env's answers and the replay format from T31); **round 3 closed**; round 4 cut (decision 66): T31 done whole (`adventure/e08-t31`); T32a done whole (decision 67: `shared/e08-t32a` #358, then `adventure/e08-t32a`, `cross-lane`); **round 4 closed**; round 5 cut (decision 68): T32b1 done (`adventure/e08-t32b1`, `cross-lane`: the harness's reads, built by this lane); T32b2 done whole (decision 69, `adventure/e08-t32b2`, `cross-lane`: the recipes, the batches run); T33 done whole (decision 70, `adventure/e08-t33`, `cross-lane`: batch E4 scored, E4-10's rule built; ADV-Q18); **round 5 closed; increment 4 closed** on 2026-10-21. **Increment 5** (the hero's movement and scouting) refined with the user the same day (T33, decisions 71 and 72; ADV-Q19; the inbox re-tagged): its first round is T34a, then T34b; T34a done whole (decisions 73, 74: flying and water walking in; `adventure/e08-t34a`: the survey `E08-hero-movement.md`, ADV-Q20 for the checkpoint, answered: decision 75); T34b done whole (decisions 76, 77: D-26 revised; `adventure/e08-t34b` #375: the three drafts, `E08-design.md` for increment 5, the cut in five rounds with batch E5 first; ADV-Q21 answered: decision 78, all accepted, A-13's caveat restated); round 1 is T35 + T36; T35 cut in two and done whole this session (decision 79): T35a (`shared/e08-t35a` #377: the ADRs), T35b (`shared/e08-t35b` #380, `cross-lane`: the kinds, flags, gap-fill, packs and replay headers); T36 cut by rows (decision 80): T36a done (`adventure/e08-t36a`, `cross-lane`: trees and Border Gates in the writer and spec, batches E5a and E5b); T36b done whole (decision 81, `adventure/e08-t36b`, `cross-lane`: banned skills in the writer and spec, batches E5c to E5e, the scorer); **round 1 closed**; round 2 is T37 + T38; T37 cut in two (decision 82): T37a done (`adventure/e08-t37a`, `cross-lane`: the harness's reads), T37b done whole (decision 83, `adventure/e08-t37b`, `cross-lane`: the recipes, the five batches run); T38 done whole (decisions 84, 85, `adventure/e08-t38`, `cross-lane`: batch E5 scored, the tile bit re-read, ADV-Q22); T38b done (decision 85, `adventure/e08-t38b`: ADV-MAP-002's count); **round 2 closed**; round 3 cut (decision 86): T39 done alone (`adventure/e08-t39`, `cross-lane`: `h3_heroes`' movement derivations, Q-0005's entry); T40 done whole (decision 87, `adventure/e08-t40`: ruleset part v10, the walking price by Pathfinding, Nomads and Cursed Ground, the day's points and reveals from the store, `ADV-QUIRK-18`); **round 3 closed**; round 4: T41 done whole (decision 88, `shared/e08-t41` and `adventure/e08-t41`, `cross-lane`: flag 30 retired; the fly and water-walk prices, the drop, the passage, the reach's pass nodes, `MoveHero`); **round 4 closed**; round 5: T42 done whole (decision 89, `adventure/e08-t42`, `cross-lane`: the plans' kits, the fixtures' flyers and water walkers, the env's test, the replays (six searched, one added), the reach workload split, the Tutorial's check); **T43** done whole (`adventure/e08-t43`: the audit of the six exit criteria, all met; HER-ADV-003 to -006 `verified`; the design note moved into the log); **round 5 closed; increment 5 closed** on 2026-10-23. **Increment 6** (army and hero management) refined with the user the same day (T43, decisions 90 and 91; ADV-Q23; the inbox re-tagged): its first round is T44a, then T44b; T44a done whole (decision 92: survey `docs/rules/surveys/E08-army.md`; its checkpoint ADV-Q24 answered, decisions 93–100); T44b done whole (decision 101: the drafts `docs/lanes/adventure/adr-drafts/*-increment-6.md`, `E08-design.md`, five rounds with batch E6 first; its checkpoint ADV-Q25). ADV-Q25 answered (decisions 102–105: the drafts accepted, the encoding's paths, the expansions as the game's configuration, batch E6 first). Round 1: T45 + T46; T45 done whole (decision 106, `shared/e08-t45`, `cross-lane`: the ADRs amended, the env to the AI lane but `replay.rs`, the two effect kinds and their packs, pack format v47, core ruleset hash v48); T46 done whole (decision 107, `adventure/e08-t46`, `cross-lane`: batch E6's spec and template on one map, E6-5 re-planned); **round 1 closed**; round 2 is T47 + T48; T47 cut in two (decision 108): T47a done (`adventure/e08-t47a`: the harness's clicks and reads, its check); T47b cut in two (decision 110), E6-5 dropped (decision 109: a stack's count cap instead); T47b1 done (`adventure/e08-t47b1`: the spec without E6-5, the recipe checked with `fill`); T47b2 done (`adventure/e08-t47b2`, `cross-lane`: three runs, every field the same; the results filled); T48 done (`adventure/e08-t48`, `cross-lane`: batch E6 scored, survey `E08-army.md` §13: every row as the reading, no question); **round 2 closed**; round 3 is T49, cut in two (decision 111): T49a done whole (the user's choice, `adventure/e08-t49a`, `cross-lane`: the positions, game state hash v12, map import v9, the other lanes' call sites, the fixtures and replays); T49b done whole (decision 112, `adventure/e08-t49b`: the moves, combinations, Titan's book, Scholar, map import v10); **round 3 closed**; round 4 is T50 + T51 (`E08-design.md` §15); T50 done whole (decisions 113–115: the cap of 1 000 000, a join at the cap to a free slot else the army dialog; `shared/e08-t50-cap` #455, then `adventure/e08-t50`: the army commands, the split's range, formation and tactics flag, the cap, the meeting's visit, the join without room's report; adventure state hash v11, ruleset part v11); T51 cut in two (decision 116); T51a done (decision 117: the listing per hero, `shared/e08-t51a-legal` first; `adventure/e08-t51a`, `cross-lane`: `Manage` on one hero, `legal_management`, `split_range`, the events, the tactics flag in battle, replay format v6); T51b done whole (decision 118; `adventure/e08-t51b`: the meeting with Scholar, the army dialog with the join offered without room, the commands between two armies; game state hash v13; `ADV-STUB-2` closed); round 4 closed. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

The text's increment 6 paragraph as it stood on 2026-10-26, before T53 closed the increment:

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T51b (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 5 are closed (2026-10-12, -13, -20, -21, -23). **Increment 6, army and hero management** (decisions 90 to 117; survey `docs/rules/surveys/E08-army.md`, design `E08-design.md`): rounds 1 to 3 done (T44a to T49b); round 4 done (T50, T51a, T51b: the meeting with Scholar, the army dialog, the commands between two armies; game state hash v13; `ADV-STUB-2` closed); round 5: T52 done whole (decision 119: the plans and properties manage, the army map and two scripted golden replays, two new gated benchmarks; `E08-design.md` §12), **T53 next** (the audit against the exit criteria, increment 6 closed, increment 7 refined; §15), its cut confirmed with the user at its start. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

The text's increment 7 paragraph as it stood on 2026-10-28, before T65 closed the increment:

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T51b (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 6 are closed (2026-10-12, -13, -20, -21, -23, -26; increment 6, army and hero management, by T53's audit). **Increment 7, rewards on the map**, refined with the user at T53 (ADV-Q26; decisions 121 to 123): T54 (the battle seam's facts) done; the survey and the ADR amendment drafts (T55a, T55b) done, each task's cut confirmed with the user at its start. T55a is cut in four (decision 125): all four done (survey `docs/rules/surveys/E08-rewards.md`; decisions 126 to 128 the cuts, its checkpoint ADV-Q28 decisions 129 to 136); T55b done (decision 137; its checkpoint ADV-Q29 decisions 138 to 141: the drafts accepted with typed answers, eight rounds with batch E7 first, design §19). T56 done (decision 142: whole, batch E7's maps and specs). Round 2: T57 done (decision 143: whole, the harness's reads, the recipes and the three runs), T58 done (decision 144: scored, A-23 withdrawn). T58b done (decision 144: `h3-map`'s view test's oracle; ADV-Q30, decision 145: A-25 a quirk with a flag). Round 3: T59 done (decision 146: whole, `shared/e08-t59`). Round 4: T60 in two (decision 147): T60a done (`adventure/e08-t60a`, the bonuses and the four changes), T60b done (`adventure/e08-t60b`, `grant` and the teachers). Round 5: T61 in two (decision 148): T61a done (`adventure/e08-t61a`, the import and the kinds), T61b done (`adventure/e08-t61b`, decision 149: whole, the bans, the draws and the state). Round 6: T62 in two (decision 150): T62a done (`adventure/e08-t62a`, the pickups, teachers and finds), T62b in two (decision 151: the rest stays in T63): T62b1 done (`adventure/e08-t62b1`, the record, the box and the Pyramid), T62b2 done (`adventure/e08-t62b2`, decision 152: whole, events and the move). Round 7: T63 in two (decision 153): T63a done (`adventure/e08-t63a`, `cross-lane`: the typed answers, the choices and the University, the env's purchase entries), T63b done (`adventure/e08-t63b`; decision 154, whole: the visit's rests, the guard's wait fixed, `ClearBonuses`, the event after the guard). Round 8: T64 in two (decision 155): T64a done (`adventure/e08-t64a`, the rewards map, its replays and benchmarks), T64b done (`adventure/e08-t64b`; decision 156, whole: the properties, the hidden-event property with a guard, decision 133's 144 × 144 maps and their forks); T65 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

## Notices from other lanes and finished items, newest first

- 2026-11-12 (T112 done, `shared/e08-t112`: decision 318): the town lane's note below moved from the inbox, acted on (ADR-0016 point 3's text before E09's garrison hero paragraph, its clause dropped; ADR-0017 point 3's drops before E09's garrison heroes' turn start).
  - 2026-11-12 (the town lane, E09 T24, branch `shared/e09-t24`, `cross-lane`; **for T112's
    merge order**): T24 moved E09 increment 3's drafts into ADR-0013 (point 16.4), ADR-0016 (points
    1, 3, 4, 6), ADR-0017 (points 1 to 4, 6, 8 to 11), ADR-0023 and ADR-0028, so your T112 is the
    second and rebases (E09 design note §56): put increment 12's text in ADR-0016 point 3 before the
    paragraph "**E09's increment 3, the garrison hero**" (it says "after E08 increment 12's text
    once E08's T112 adds it": drop that clause then), and in ADR-0017 point 3 the turn start's maximum
    and points before E09's garrison heroes' paragraph (the draft's anchor, "the maximum and the
    points left of each of its heroes", was not in the ADR). Shared versions now: core ruleset hash
    v53, pack format v52 (`town_growth`, `owner_town_growth`, unread until E09 T25 and T26), quirk
    flags 41 (`quirk.town.*`, on in `faithful`); your 72 replays kept commands and state hashes,
    only `ruleset_hash` moved.

- 2026-10-19 (T21b's audit): since E04c T10b's re-recording no golden replay declines a guarded pile's question or a Cartographer's offer (the battle lane's note of 2026-10-16 below, (5)); both stay covered by `h3-game`'s unit tests and the property plans. A wider seed search or a fixture of its own would restore them, with the next re-recording. *Done by E08 T86a (2026-11-03): `duel-s4749-d28` and `duel-s5249-d28` decline a pile's question, `duel-s3560-d28` and `duel-s645-d28` a Cartographer's offer.*

- 2026-10-31 (T82 and T83a done together: decision 213, T83 cut in two; ADV-Q39 answered, decision 214; T83b queued): the T82 and T83 entries as they stood in the "Tasks" text:

  **E08 T82 — `h3-heroes`: the day's income, the day's spell points, the Lighthouses' input**

  - **Scope:** design §2 and §3 (round 5; small): `daily_income(details, core)` (ADR-0013 point
    16.1, GAM-RES-004: each worn copy counting, a worn combination's parts too, a skill level's
    amount grown by a skill specialty on it in exact integers, the specialty's; saturating),
    `day_mana(details, core)` (point 16.2, HER-MAN-002: `full_mana`, the highest `mana_regain` else
    `base_mana_regain`, a specialist's growth `+ regain_specialist_bonus` only with the skill, each
    worn type's `extra_mana_regain` once, points above the maximum kept) and `sea_day_points`'
    `lighthouse_points` (point 13.6, HER-ADV-009 amended; the adventure passes 0 until T84). The
    rules in the catalog with T81b's gap-fill sources and batch E9's rows (E9-7, E9-12) as
    observations.
  - **Inputs:** design §2, §3; ADR-0013 points 3, 13.6, 16; survey `E08-economy.md` §8, §12; T81b's
    record in `E08-log.md` (the kinds and parameters as built); T71's record (the sea points'
    pattern).
  - **Depends on:** T81b (`shared/e08-t81b` merged). **Status:** ready after that merge; its cut
    (alone or with T83, design §18's round 5) confirmed with the user at its start.

  **E08 T83 — `h3-map` and `h3-adventure`: the import, the kinds and the part, the state, the views**

  - **Scope:** design §4, §5, §6, §12 (round 5): map import v15 (classes 42, 53, 55, 87, 109, 112,
    220; the owners; the timed events; `ObjectOwner`, `MineResources`, `OwnedOnWater`), the seven
    kinds and the calendar, growth, months and income parameters (adventure ruleset part v14), the
    owners, guards, weekly marks, fractions and added objects in the state (adventure state hash
    v17), the views (ADV-OWN-003, ADV-FOG-006 amended); likely cut in two at its start.
  - **Inputs:** design §4 to §6, §12, §16; ADR-0016 points 1, 3 to 6; survey `E08-economy.md`; T72's
    record in `E08-log.md` (increment 8's import, kinds and state) as the pattern.
  - **Depends on:** T81b (merged). **Status:** ready after that merge; its cut confirmed with the
    user at its start.

- 2026-10-30 (T80 done, whole with A-33's re-read: decisions 209, 210; T81 queued): the T80 entry as it stood in the "Tasks" text:

  **E08 T80 — Batch E9 scored**

  - **Scope:** survey `E08-economy.md` §19 (new): each row of batches E9a and E9b against its
    prediction; D-47 (E9-1), D-49 (E9-9), A-32 (E9-10), A-33 (E9-4) and design §13's call on who plays
    an owned mine's guards (E9-2) settled; a dispute a row contradicts reverts to the other side
    (decision 196) or becomes an `ADV-Q`; the consensus rows' sources `verified`; D-48 and D-50 stay
    as read (decision 199); the README's batch row evaluated.
  - **Inputs:** the results (T79a: `experiments/results/E08-batch-9a.md`, `-run2.md`; T79b:
    `E08-batch-9b.md`, `-run2.md`); the specs' predictions; survey §14, §15; T79a's and T79b's
    records in `E08-log.md` (the runs' draws).
  - **Depends on:** T79b (done). **Status:** ready.

- 2026-10-30 (T77b done, whole: decision 200; ADV-Q38 answered, decisions 201–204; T78 queued): the T77b entry as it stood in the "Tasks" text:

  **E08 T77b — The ADR amendment drafts, the design note and the cut**

  - **Scope:** from T77a's survey and the answers at its checkpoint: the amendment drafts of the
    ADRs it touches (ADR-0016 for owned objects, the day's and the week's changes on the map;
    ADR-0017 for the day's and the week's order in the game and the income into the treasury;
    ADR-0013 if `h3-heroes` gains rules, such as the Lighthouse's sea points), `E08-design.md`
    written for increment 9 (with its exit criteria), the building tasks cut in rounds; the
    checkpoint's questions (an `ADV-Q`). The user accepts the drafts and the cut before anything is
    built (decision 193).
  - **Inputs:** the survey's §13 (the interfaces), §14 (D-49, D-50, A-32; A-31 corrected), §15
    (batch E9's plan, its harness and spec gaps), §16 (the rule ids) and §18; `ADV-Q37`'s answers,
    decisions 196–199 (the scope grown by the heroes' income, the day's spell points and the
    month's monsters; batch E9 whole before the build).
  - **Depends on:** T77a and its checkpoint (both done). **Status:** ready; its cut confirmed at
    its start.

- 2026-10-30 (T77a2 done, whole: decision 195; T77a done): the T77a entry as it stood in the "Tasks" text:

  **E08 T77a — The mines, week and owned objects survey** (in two, decision 194: T77a1 points 1 to 4 and the census done, survey §0 to §6; **T77a2** point 5, the mines' income of point 2, points 6 and 7 next, §7 on)

  - **Scope** (decisions 192, 193): a survey `docs/rules/surveys/E08-economy.md` (as T17a, T24a,
    T34a, T44a, T55a and T66a): the original's rules by code reading (`docs/exe/README.md`; read so
    far: `visit_mine` 0x4A39A0 and the owned objects' code by T77a1 (the survey's Sources), `tile_object_owner` 0x40FD20 with the owned objects'
    record tables, `new_day` 0x4C7FE0 and `new_day_week_events` 0x4C8780 in part,
    `covered_view_new_week` 0x4CA840; `town_gold_income` 0x5BF600 is a town's, E09) with VCMI 1.7.5
    and the wiki, and a census of the 160 shipped maps (T76's counts in `E08-log.md`, T76). (1)
    **Flagging:** which classes a visit flags, what it costs or asks, a guarded owned object, a hero
    standing on an owned object's entrance, an owned object of a player out of the game, the owner
    at load (the map's owner fields), what another player sees (ADR-0016 point 6). (2) **Mines** (53
    subclasses 0 to 6; the abandoned mine 53.7 and 220): their guards, the Leave Guards dialog, the
    abandoned mine's guards and its resource; **the day's income** and its order at the day's
    start. (3) **Lighthouses** (42): the bonus to the owner's heroes at sea, its stacking, when it is
    read (the turn start's sea points, HER-ADV-009). (4) **Shipyards** (87): where the boat appears,
    its price, when it is refused (a boat or a hero on the tile), what the boat's record gets
    (ADV-SEA-001). (5) **The week:** its order (the day, the week, the month), the Windmill (112), the
    Water Wheel (109) and the Mystical Garden (55) (what each gives, who, how often, the reset), the
    monsters' weekly growth (the "does not grow" flag), the map's timed events (decision 127's
    agent's call). (6) The commands, decisions, events, the observation, replays and benchmarks it
    changes; **batch E9's plan** on the client lane's harness (decision 193's agent's call). (7) The
    proposed rule ids, quirk candidates and disputes; a fresh agent's second reading of the deciding
    code; the checkpoint's questions (an `ADV-Q`).
  - **Not in it** (decision 192): dwellings and random dwellings, the Refugee Camp and the hire
    dialog, garrisons (item 10); the teleporters; towns' income and the week's creature (E09); a
    computer player's flags (E10); simultaneous play.
  - **T77a2** (decision 194: what the day's start does): the day's, week's and month's order
    (`new_day` 0x4C7FE0, `new_day_week_events` 0x4C8780, read in part), the mines' income
    (`mine_daily_income` 0x678288, survey §3.5) and its order, point 5 whole, then points 6 and
    7: the survey's §7 on, with T77a1's inputs (§6: D-47, D-48, A-29 to A-31, the open points,
    among them the Shipyard's use without a visit, §5.3, for the second reading) and the
    checkpoint's `ADV-Q37`.
  - **Depends on:** T76 (done). **Status:** T77a2 ready, its cut confirmed at its start; a user
    checkpoint at its end (decision 193).

- 2026-10-30 (T76 done, increment 8 closed; T77a and T77b queued): the chronicle of increment 8's rounds as it stood in the "Tasks" text:

  Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T65 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 7 are closed (2026-10-12, -13, -20, -21, -23, -26, -28; increment 7, rewards on the map, by T65's audit). **Increment 8, boats and the sea**, refined with the user at T65 (ADV-Q31; decisions 158, 159): the survey and the ADR amendment drafts first (T66a, T66b), each task's cut confirmed with the user at its start. T66a in two (decision 160): T66a1 done (`adventure/e08-t66a1`, #550: survey `E08-sea.md` §0 to §6), T66a2 done (`adventure/e08-t66a2`, decision 161: whole; §7 to §14, the checkpoint's ADV-Q32); T66b done (`adventure/e08-t66b`, decision 169: whole; the drafts, `E08-design.md` and the cut in eight rounds, ADV-Q33 accepted: decisions 170–173). Round 2: T67 done (`adventure/e08-t67`, decision 174: whole; the writer's sea objects, the spec's bottles and a monster's own tile, `cross-lane`; the specs E8a, its water map and E8b with their templates). Round 3: T68 in two by batch (decision 175): T68a done (`adventure/e08-t68a`: the harness's boat reads and steps, E8a and its water map run and recorded), T68b done (`adventure/e08-t68b`, decision 176: whole; the sea pickups' reads, E8b run four times, two recorded); T69 done (`adventure/e08-t69`, decision 177: whole with a re-read; batch E8 scored, D-45 reverted, D-46 to batch E8c: decision 179); T69b done (`adventure/e08-t69b`, decision 180: whole; batch E8c run and scored, D-46 confirmed, survey §16). Round 4: T70 done (`shared/e08-t70`, `cross-lane`, decision 181: whole; the three ADRs applied, the effect kinds, the packs and the glossary). Round 5: T71 done (`adventure/e08-t71`, decision 182: with the inbox's property case; HER-ADV-009), T72 done (`adventure/e08-t72`, `cross-lane`, decision 183: whole; the import, kinds, state and views; the AI lane's bots tables' fingerprint). Round 6: T73 in two (decision 184): T73a done (`adventure/e08-t73a`: the boat's moves, prices, conversion and events; ADV-SEA-002 to -005), T73b done (`adventure/e08-t73b`, decision 185: whole; the boat's reach, the walker's boat test, the property; ADV-STUB-11 closed). Round 7: T74 in two (decision 186): T74a done (`adventure/e08-t74a`: the sea's pickups' visits from a boat, ADV-PIK-017, -018; `random-play-duel-36` re-baselined), T74b done (`adventure/e08-t74b`, decision 187: whole, no `cross-lane`; the battle at sea, a beaten hero leaving with its boat, ADV-SEA-006; the env needs no encoding change); the security doc's increment 8 in `shared/e08-security` (decision 188). Round 8: T75 in two (decision 189): T75a done (`adventure/e08-t75a`: the sea map `e08-sea-36.h3m`, three random and eight scripted replays, `random-play-sea-36`; its benches README row in `shared/e08-t75a-benches`), T75b done (`adventure/e08-t75b`, decision 190: whole; the properties at sea, the passage's D-46 fix they found, the sea XL `e08-sea-144.h3m` with `reach-sea-xl-144` and `fork-sea-xl-144`, the census; its benches README rows in `shared/e08-t75b-benches`). Next: **T76** (the audit against increment 8's exit criteria, increment 8 closed, increment 9 refined; design §18), its cut confirmed with the user at its start. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

- 2026-10-29 (T69b done, batch E8c; T70 queued): the T69b entry as it stood:

  **E08 T69b — Batch E8c: a flight over a water tile holding an object (D-46)**

  - **Scope** (decision 179; survey `E08-sea.md` §15, D-46): one row, **E8-17**, on a small map of
    its own (the spec `experiments/specs/E08-batch-8c.toml` and its template; the writer has the
    Flotsam and boats): heroes with Angel Wings on land, each one's only way east across water
    bounded by rock (terrain 9, closed to a flight): (a) one water tile holding a Flotsam, entered
    from land; (b) three water tiles, the Flotsam in the middle, entered from water; controls (c)
    plain water and (d) an empty boat. By the code (a) and (b) give no path to the far side and
    (c), (d) a path over; ADV-MOV-009 and VCMI a path over all four. Record each click's marks and
    the hero's tile; the far sides explored (ADV-MOV-008). The harness recipe
    (`clients/capture/batches/`; its walk and marks reads exist), the run under Wine (the
    installation's files staged as T68a), the record in `experiments/results/`, the score in survey
    §15: confirmed, the drafts and design stay and T73 amends ADV-MOV-009; reverted, the drafts,
    design §7 and §15 go back to the pass and the reading is re-read.
  - **Depends on:** T69 (done). **Status:** ready; its cut confirmed at its start.

- 2026-10-28 (T66b done, the drafts, the design note and the cut; T67 queued): the T66b entry as it stood:

  **E08 T66b — The ADR amendment drafts, the design note and the cut**

  - **Scope:** from T66a's survey and the answers at its checkpoint: the amendment drafts of the
    ADRs it touches (ADR-0016 for boats, the sea and a visit from a boat,
    `docs/lanes/adventure/adr-drafts/`; ADR-0017 for the commands, decisions and battles; ADR-0013
    if `h3-heroes` gains rules, such as the day's points at sea), `E08-design.md` rewritten for
    increment 8 (the hero side, the map, the game, the batch and the cut, with increment 8's exit
    criteria), the building tasks cut in rounds; the checkpoint's questions (an `ADV-Q`). The user
    accepts the drafts and the cut before anything is built (decision 159).
  - **Depends on:** T66a (done) and its checkpoint (ADV-Q32, the survey's §14). **Status:** waiting
    for the user's answers to ADV-Q32; its cut confirmed at its start.

- 2026-10-28 (T66a2 done, the survey `E08-sea.md` complete; T66b waits for ADV-Q32): the T66a entry as it stood:

  **E08 T66a — The boats and sea survey** (in two, decision 160: T66a1 points 1, 2, 3, 6 done, survey `E08-sea.md` §0 to §6; T66a2 points 4, 5, 7, 8 next)

  - **Scope** (decisions 158, 159): a survey `docs/rules/surveys/E08-sea.md` (as T17a, T24a, T34a,
    T44a and T55a): the original's rules for boats and the sea, by code reading
    (`docs/exe/README.md`; what is read already: `hero_boards_boat` 0x4A1010, `hero_lands_from_boat`
    0x49E670 and the boat's points in survey `E08-hero-movement.md` §1.1, §1.4, §2.1, §3.1 and §6,
    the boats' drawing in `docs/epics/E24-design.md` §2.3) with VCMI 1.7.5 and the wiki, and a census
    of the 160 shipped maps. (1) **Boats** (class 8): how many, their subclasses, heroes placed in
    one, what a map leaves reachable only by sea; the body and the import. (2) **Boarding and
    landing:** which steps board and land, the points they cost and leave (the Admiral's Hat), what
    boarding clears (Fly, Water Walk), where the boat stays, a guarded or blocked shore. (3)
    **Sailing:** the step's price at sea and the day's points (Navigation, the Necklace of Ocean
    Guidance, the Sea Captain's Hat; a Lighthouse's bonus stays with item 9), Favorable Winds (225),
    what a hero in a boat visits and how (the sea's objects, another hero, the events on water of
    the inbox's note below), scouting at sea, what another player sees of a boat. (4) **A battle
    with a hero in a boat** (against a hero; the ship's deck and the two boats battlefields,
    GAM-BAT-001; whether anything else fights a hero at sea). (5) **The sea's pickups:** Sea Chest
    (82), Shipwreck Survivor (86), Flotsam (29), Ocean Bottle (59): their contents, draws at a new
    game and visits. (6) **The writer:** an empty boat and a hero standing in one (CLT-Q5). (7) The
    commands, decisions, events, the observation, replays and benchmarks it changes; **batch E8's
    plan** on the client lane's harness (decision 159's agent's call). (8) The proposed rule ids,
    quirk candidates and disputes; a fresh agent's second reading of the code; the checkpoint's
    questions (an `ADV-Q`).
  - **Not in it** (decision 158): Shipyards (87) and Lighthouses (42), item 9; Whirlpools, Buoys,
    Mermaids, Sirens; the Derelict Ship and the Shipwreck (item 10); Summon Boat and Scuttle Boat
    (the adventure spells); a town's shipyard (E09).
  - **Depends on:** T65 (done). **Status:** T66a1 done (2026-10-28: the census, boats, boarding,
    landing, sailing, the events on water, the writer; disputes D-41 to D-44 for the checkpoint);
    **T66a2 ready** (the battle at sea, the four pickups, the commands and the observation, batch
    E8's plan, the rule ids, the second reading of §2 to §4's deciding code, the checkpoint's
    `ADV-Q` with D-41 to D-44), its cut confirmed at its start; a user checkpoint at its end
    (decision 159).

- 2026-10-27 (E08 T62b2's census; **for T66a**, increment 8 builds boats, decision 158): 9 events of
  the shipped maps lie on water. The engine fires an event only for a hero that stands on its tile,
  so these fire once a hero in a boat can step onto them (the original's `hero_move_step` tests
  every tile a step enters, at sea too). Check the boat's step against
  `AdventureState::firing_event` then. *(2026-10-28, T66a1: confirmed, survey `E08-sea.md` §1 and §4.3; the survey carries it to T66b.)*

- 2026-10-28 (T65, increment 7 closed): the entry below moved from the inbox, nothing left to act on (the map client lane's change of the writer's edge bound, merged with its tests).

- 2026-10-27 (the map client lane, E24 M2 T6b1, `map-client/e24-r2-8`, `cross-lane` with the
  user's consent at E24 T0; for your information, nothing to do): **the writer's edge bound one
  tighter** (E24 M1's reviewer, R2-8): `h3_map::writer::write_with` took an object anchored up to
  `side + 7` columns and `side + 5` rows, where an 8-wide or 6-high template has no cell on the
  map; now `side + 6` and `side + 4` (7 and 5 past the last column and row, as its doc comment
  and `docs/formats/h3m.md` already said). `crates/h3-map/src/writer/extras.rs`, its tests moved
  with it.

- 2026-10-26 (T54 done, `adventure/e08-t54`; decisions 123, 124): T54's queue entry and the three battle-lane notes it closed moved here verbatim; the retreat facts stay behind `ADV-STUB-9`.

**E08 T54 — The battle seam's facts from the game** (the queue entry as it stood):

- **Scope** (decision 123; the inbox's three battle-lane notes of 2026-10-23 and -24, which this
  task closes): `h3-game`'s battle setup (`state/battle.rs`, `Battles` when a battle starts on the
  map) fills what the seam gained for battles from the map, each from the game's own state:
  (1) `BattleSetup.difficulty`, the game's difficulty (the setup's, kept in the state if it is
  not); (2) `BattleSetup.human`, per side from a new per-player flag of the game's setup
  (`PlayerSetup`), false for a side without a player; with it BAT-QUIRK-27's let-off (BAT-MOR-004,
  on in `faithful`) no longer reaches a human player's side. Where the flag comes from (the
  session that seats the players, the map's player kinds for the import's default, what an env's
  agent counts as) is the user's question at the task's start; (3) `BattleSetup.colours`, each
  side's player colour (`None` for a monster's or a guard's side); (4) `BattleSetup.background`,
  0x41ADB0's variant on dirt, grass and snow: the tiles within 3 of the battle's whose object
  class is a tree-like, mountain-like or dead vegetation one (`docs/epics/E17-design.md` §5:
  classes 114 to 160; the import may have to keep a decoration's class, a map import version);
  (5) `BattleSetup.attacker_ground`, the special battlefield of the attacking hero's own tile
  (ADV-GRD-001); (6) `BattleHero.sex`, the map's sex code for a placed hero (`h3-map`'s model
  reads it; `None` the hero type's own). The retreat facts (`BattleSetup.retreat`, ADR-0027 point
  14: a Tavern to hire the hero again, E09; the victory condition, E10; read only for a computer
  player's hero, none before E10) stay at their defaults: a stub (`ADV-STUB-9`). Rule ids (a
  `GAM-BAT-*` per derived fact), tests, the ten and two golden replays re-recorded (their hashes
  inside battles move), the versions it moves (game state hash, map import) in the table.
- **Not in it:** the computer player's own choices (E10, E13), the retreat facts' derivation.
- **Depends on:** T53 (done). **Status:** ready; its cut confirmed with the user at its start.

- 2026-10-24 (**T54**, decision 123; the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
  both `cross-lane` with the user's consent; **for you**, when battles start from the adventure
  map): the seam carries three facts for viewers, read by no rule, each defaulting to "not
  known": `BattleSetup.background` (the battlefield picture's variant on dirt, grass and snow:
  0x41ADB0 counts the dead vegetation, mountains and trees within 3 tiles of the battle's tile, 0
  where dead vegetation is the most, 1 where mountains are, 2 otherwise, an empty surrounding
  included; `docs/epics/E17-design.md` §5), `BattleSetup.colours` (each side's player colour by
  `BattleSide::index`, 0 red to 7 pink; `None` for a side without a player, a monster's or a
  neutral town's; one colour for both sides is `SetupError::InvalidColour`) and `BattleHero.sex`
  (`HeroSex`, where the map gave a placed hero one; `None` the hero type's own). `h3-game` should
  set them when it starts a battle. The battle keeps them as hashed state (battle state hash v54)
  and observes them; your ten golden replays were re-recorded with their options, their commands
  unchanged (only checkpoint hashes taken inside a battle moved).

- 2026-10-23 (**T54**, decision 123; the battle lane, E23 T7, `shared/e23-t7-human` #436 and `battle/e23-t7-rest`, both
  `cross-lane` with the user's consent; **for you**): **a battle rule reads `BattleSetup.human`
  now**: with the new fix-optional BAT-QUIRK-27 (`quirk.battle.computer_morale_let_off`, registry
  position 33, on in `faithful`) a frozen stack of a side whose player is not human escapes one
  bad-morale freeze in four (BAT-MOR-004, 0x464BDB; ADR-0023 point 7 widened). `h3-game`'s battle
  setup (`state/battle.rs`) leaves `human` and `difficulty` at their defaults, so in a game's battle
  every side, a human player's too, gets the let-off: set `human` from the side's player (true for
  a human player's hero, false for a computer player's and a neutral side) and `difficulty` from
  the game. In your paths: the base faithful core ruleset hash moved (`0x1ac6c50d158057de` →
  `0x0c7e01b4dffeba0c`), your eight faithful replays' headers patched (`ruleset_hash` only; all ten
  verify). Also BAT-WAR-003: the automatic tent heals the most hurt stack (no draw).

- 2026-10-23 (**T54**, decision 123; the battle lane, E23 T6's last part, `battle/e23-t6`, `cross-lane` with the user's
  consent; **for you**, when battles start from the adventure map): the battle now keeps
  `BattleSetup.retreat` and `attacker_ground` (ADR-0027 point 14) as hashed state, and
  `h3_battle_ai::choose` reads them: a hero's side retreats as the original's computer does
  (BAT-AI-024; `POLICY_VERSION` 18) when the facts allow it, and the odds value an attacking hero's
  spells at its own ground's mastery in `faithful`. The game orchestrator (`h3-game`) should derive
  them: per side the hero's experience, its artifacts' worth by the original's count (per worn or
  carried artifact the larger of the adventure AI's value, 0x433AA0, and half its gold cost), a
  Tavern town of its player to hire it again from (another than the battle's town for its
  defender), whether the scenario's victory condition wants that hero defeated; and the special
  battlefield of the attacking hero's own tile. The defaults never retreat. Your ten golden
  replays were re-recorded again with their options, their commands unchanged (only hash lines;
  `duel-s2-d7` none): the hash covers the new fields.

- 2026-10-26 (T53, increment 6 closed): the three entries below moved from the inbox, nothing left to act on (the map-client lane's edge pieces: nothing to change; T23's renumbering: ADR-0016 amended for army management by T45; the harness's batch features: `clients/capture/README.md` holds them).

- 2026-10-23 (the map-client lane, E24 M1 T1a, `map-client/e24-t1a`; answers your T8b1 note,
  2026-10-11): **which hidden tiles the original draws with an edge piece**
  (`docs/epics/E24-design.md` §1.5, a code reading of 0x412220 and 0x40F8C0). Every tile the
  viewer has not explored and that has an explored tile among its eight neighbours gets a piece of
  `tshre.def` chosen by which of the eight are unexplored (neighbours off the map count as
  unexplored); every other unexplored tile gets the full `tshrc.def`; explored tiles nothing. So
  the half-dark border is exactly ADR-0016's fringe (the eight tiles around an explored one), as
  your T8b1 took it; VCMI agrees on which tiles. The view tests only the viewing player's own bit
  of the tile's vision word (0x69CCBC = 1 << the viewing colour), no team bits. Nothing to change.

- 2026-10-20 (T23, decision 48): what moved from increment 4 to the sketch's increment 5 is still called increment 4 where this docs-only change did not reach: ~~the code comment in `crates/h3-adventure/src/reach.rs` (movement skills)~~ (T33: increment 5); ADR-0016 (army management, increment 6 since decision 90: T44b's draft), with the amendment that builds army management (T24b2 needed none for ADR-0016; ADR-0013's movement and scouting line is in T24b1's draft, point 2). The surveys keep their *(inc. 4)* marks (input, not rules).

- 2026-10-19 (the client lane, E17 H1 and H2; your request of 2026-10-13): **the harness answers questions and plays battles from the map** (`clients/capture/README.md`, "An experiment batch"; design `docs/epics/E17-design.md` §6.3). A recipe now has `difficulty` (checked in memory), a step's `answer` (`yes`/`no`, or per kind: `join`, `join for gold`, `flee`, `other`: the question named by the record its text came from, ADVEVENT 86, 88 or 89 + 90, 91) and `battle` (`auto`, `retreat`, `surrender`, `defender retreat`, `defender surrender`), and records the questions asked, the battle's stacks at the start and the end (and hexes: the formation), its outcome, any hero's experience, level, primary and secondary skills, army and whether it is on the map, every player's resources before and after, and a tile's monster (count, aggression, never flees). Checked on Easy with what h3-experiment places today and, for the offers, on a local map with patched dispositions (join, flee, join then flee): a monster's offers in a spec need T22a's dispositions and "never flees". Two things for the spec: hot seat needs at least two human players, and a player left without heroes and towns is out (with one human left the game ends): keep each player a hero after its battles. H2 the same day: `walk = [x, y, level]` (the underground, E3-4; shown on a shipped map, since `h3-map`'s writer has no underground yet), more than five heroes per player (the list scrolled), `level_up` (`left` or `right`: a level-up's offered skill), the observations `heroes:experience` (the heroes off the map: E3-7) and `date`, and any number of days. Seen on the way (scored with batch E3 at T22b): a map hero with 5 000 experience and 2/3/1/1, basic Diplomacy and advanced Logistics started at level 5 in two runs, its level-ups drawn (3/5/1/2 with both expert, then 3/4/2/2 with Diplomacy advanced) (E3-6); a surrender in hot seat moved its 470 gold to the other player (E3-8); the 147 heroes off the map held 40 to 90 in two runs, 1 and 4 of them at 90 (E3-7).

- 2026-10-23 (T45, for T51; its T49 half done in T49b: `meeting_spells` and `gives_spell_book`
  are read by `h3_heroes::meeting_spells` and `equip`): **ownership.** The env is the AI lane's since T45: when T51 opens a meeting
  or an army dialog, tell its inbox (its masks must allow `Decline` there, `E08-design.md` §12),
  and T52 asks the AI lane, not this one, for the env's test.
  *(Done 2026-10-26, T51b: the AI lane's inbox told; its masks allow `Decline` from `legal_commands` unchanged.)*

- 2026-10-24 (the E11 refinement, `shared/e11-refinement`, Q58; for your information, nothing to
  do now): **E08 decision 104 is extended**: the enabled packs decide the edition's *rules* too,
  each at its latest patch (`base` alone RoE's executable, `base`+`ab` AB's, all three SoD's), read
  from the user's own RoE and AB media (`docs/epics/E11-historical-rules.md`; the battle lane
  leads). Adventure differences reach this queue as tasks after E11 T3's checkpoint; the edition's
  data shape comes from E11 T4's ADR draft (amending ADR-0017 point 1). The former E11 is folded:
  every epic covers RoE, AB and SoD for what it builds, and Conflux as a town and the moat by
  expansion stay E09's, the AB/SoD objects E12's. E14's refinement decides on the original's random
  map generator.

- 2026-10-23 (the map-client lane, E24 M1 T5a, `map-client/e24-t5a`; for information, nothing to
  do now): **a hero the map places on water gets no boat.** Observed in the original under Wine
  (`docs/epics/E24-design.md` §2.8): a map's hero on a water tile starts on horseback on the
  water, its status bit 18 (in a boat) clear and no boat record made; boarding a boat by a move
  takes the day's points (hero_boards_boat 0x4A1010, without the Admiral's Hat) and gives the
  boat the hero's owner as its colour. Every hero and boat faces right (2) at a new game. For the
  importer and the engine's start when a map does this (the editor's own maps were not checked).

- 2026-10-23 (the map-client lane, E24 M1 T4a, `map-client/e24-t4a`; a finding for when the
  engine draws tavern rumours, nothing to do now): **the original's text rumour picks the wrong
  row and can crash.** `rumour_pick_text` (0x4CCF20, from 0x4C8780's roll of 1–100 below 33 at a
  new day's rumour; `docs/exe/map.toml`) counts the unused of 256 flags (game +0x4E546, cleared
  at a new game and when all are used), draws n = random_between(1, that count) and marks the
  n-th unused flag, but copies row n of its table (the first 256 rows of `randtvrn.txt`,
  `rumours_load` 0x4B8410), not that flag's row: row 0 never comes, rows repeat, and with all
  256 unused a draw of 256 reads one entry past the table, a null string, and the game crashes
  at 0x4CCF8C (about once in 800 new games; the map client's harness saw it once at a game's
  start and now starts such a game again). A code reading only, no wiki or VCMI compared: by
  the quirk rules the crash is always fixed; whether the skewed choice is kept is a
  classification for your lane.

- 2026-10-23 (the map-client lane, E24 M1 T3a, `map-client/e24-t3a`, `cross-lane` with the user's
  consent; for your information, nothing to do): **`h3_map::writer` gained a second entry point.**
  `write_with(&Map, &Extras)` writes tiles as given (`GivenTile`: terrain, river, road, their
  pictures and flags; `RiverKind`) and objects of any template but the heroes' (`AnyObject`, with
  the body the editor gives one left as placed, an owner where the body has one, a position up to
  7 columns and 5 rows past the edges) after the map's own. `write` is `write_with` without
  extras: no field, variant or output changed (a test pins a map of every kind by its CRC-32), so
  your literals and matches stay. The new code lives in `writer/extras.rs`; `docs/formats/h3m.md`
  ("The map client's additions") lists the bodies and where they come from.

- 2026-10-23 (the battle lane, E23 T6, `battle/e23-t6`, `cross-lane` with the user's consent; for
  your information, nothing to do): `h3_battle::BATTLE_STATE_HASH_VERSION` is 53 (the tactics
  phase's rounds the computer counts). Your ten golden replays were re-recorded with their
  options, their commands unchanged: nine moved the checkpoint hashes taken inside a battle,
  `duel-s2-d7` none; `adventure_replays.rs`'s history says so. `h3_battle_ai::choose` now plays a
  hero's side's commanded Catapult and First Aid Tent and its tactics phase (`POLICY_VERSION` 17);
  it still never retreats (BAT-STUB-5, T6's last part).

- 2026-10-23 (the map-client lane, E24 M1 T1b, `map-client/e24-t1b`; answers your T14a note,
  2026-10-13; nothing to change): **the order inside a tile's draw list** (`docs/epics/E24-design.md`
  §2.1, a code reading of 0x505060 and 0x505230). Each cell of an object gets a layer 0–6: 0 for a
  flat template, else its row counted from its column's base (the anchor row, or a blocked cell on
  a passable one), a passable cell beside a blocked one taking that cell's layer. A tile's list
  stays ordered by layer; a new entry goes above every lower layer and, at an equal layer, above
  the earlier object unless that one is higher on another tile they share. So 0x505610's
  topmost visitable object is the one with the highest layer on the tile, and at equal layers
  (two entrances on their bottom rows, the usual case) the later in the map: your "last in map
  order" for every shipped map (none stacks visitable tiles). Only an object whose layer on that
  tile is higher (the tile in an upper row of it) would beat a later one.

- 2026-10-23 (the map-client lane, E24 T0, `map-client/e24-t0`; for your information, nothing to
  do now): **the adventure map client's plans that touch your crates** (`docs/epics/E24-map-client.md`).
  (1) **The writer:** the map-client lane adds what its test maps need to `h3_map::writer` in
  additive `cross-lane` pull requests (the user's consent, 2026-10-23; E24 M1 T3a): rivers, a
  tile's pictures and flags as given, objects of any template; every map the writer writes today
  stays byte for byte the same; each pull request gets a note here. (2) **Two of your open points
  get read:** the draw-list order at 0x505230 (your T14a note, 2026-10-13) in E24 M1 T1b, and
  which hidden tiles get an edge piece (your T8b1 note, 2026-10-11) in T1a; the results come here.
  (3) **For E24 M4 only** (a viewer of the engine's games): the viewer takes the static art from
  the raw `.h3m` and the live part from the game's observation, so it will need each `ObjectView`
  matched to the map's object (your T12b note: no index, since it would count unseen objects) and
  a random object's drawn type; refined with you at M4.

- 2026-10-23 (the client lane, E17 M3 T9c, `shared/map-client-lane`; for information): **the
  client lane split** (the user, CLT-Q18). The capture harness `clients/capture/` is a common
  path now: your recipes, reads and steps there need no `cross-lane` label any more (your
  HANDOFF's "Experiments" line still says `cross-lane`); a change that can alter the original's
  side runs the harness check, and a change in behaviour goes into the client and map-client
  lanes' inboxes (`clients/capture/README.md`, "Who changes it"). A new lane, **map-client**
  (E24), builds the adventure map client: what it needs of your crates (`h3-map`'s reader and
  writer, later the game's state for a viewer) comes as entries in this inbox.

- 2026-10-23 (the battle lane, E23 T5b2b, `battle/e23-t5b2b`, `cross-lane` with the user's
  consent; for your information, nothing to do): `h3_battle::BATTLE_STATE_HASH_VERSION` is 52 (each
  stack's last drawn spell, the original's +0x4E0, which the computer's cast entry reads). Your ten
  golden replays (T42's) were re-recorded with their options, their commands unchanged: nine moved
  the checkpoint hashes taken inside a battle, `duel-s2-d7` none; `adventure_replays.rs`'s history
  says so.

- 2026-10-22 (the battle lane, E23 T1a6, `battle/e23-t1a6`; for your information, nothing to do
  now): **the computer's retreat test reads the adventure** (`docs/formats/installation.md`
  §7.55): the side's hero's carried artifacts and experience, the player's gold, its towns with a
  Tavern, the scenario's "defeat a hero" victory condition (game +0x1F89C), and the adventure AI's
  value of each artifact to the player's best hero (0x433AA0 → 0x4339E0 → 0x4336C0; read by its
  cases only: the rest is yours with the adventure AI). No seam carries these; BAT-Q61 asks how
  the battle gets them (a recommendation: a seam input at E23 T6, which your derivation would
  fill).

- 2026-10-21 (the client lane, E17 M3 T7b-1; for your information, nothing to do): E08 T32a's
  `h3_map::writer::Map::level_limit` (#360) left `clients/`'s `h3-view situation` unbuilt on
  `master` (it builds the writer's `Map`); the client's pull request sets it to 0, the byte the
  writer wrote before. A field added to a struct `clients/` builds breaks it: `clients/scripts/
  check.sh` (or `cargo build --manifest-path clients/Cargo.toml`) shows it while CI cannot run.

- 2026-10-21 (the battle lane, E23 T2b, `shared/e23-t2b`, `cross-lane` with the user's consent;
  for your information, nothing to do): BAT-QUIRK-25 (`quirk.battle.original_ai_slips`, registry
  position 29) is on in `base:faithful` (and `synthetic:faithful`), off in strict-fixed: the
  faithful core ruleset hash moved to `0xe6839bfb42ebe118`, so your seven faithful replays in
  `fixtures/replays/adventure/` got it in their headers (commands and state hashes unchanged; the
  two strict ones keep theirs). The flag changes only the original's battle AI's estimates.

- 2026-10-21 (the client lane, E17 M3 T6c-2; for your information, nothing to do): **two harness fixes you asked for or may meet.** `h3adv.value` prints a winner of -1 as `none` (your T22a2b note). `h3lod` now reads the base archive's text tables before the AB ones, as the executable searches them (E17 design §3.1): the AB copies' GENRLTXT differs from record 79 on (ADVEVENT's first 190 records are the same, the base copy has one more), so a dialog identified as `GENRLTXT:<n>` from 79 on names the record the game showed; no recorded batch cites one.

- 2026-10-20 (the battle lane, E23 T1c, `battle/e23-t1c`, `cross-lane` with the user's consent, BAT-Q54; for your information, nothing to do): **the quirk flag `quirk.battle.berserk_walks_to_position_hex`** (BAT-QUIRK-24, registry position 26, `shared/e23-t1c-quirk`) is on in `base:faithful`, off in `base:strict-fixed`, so the faithful core ruleset hash moved: the seven faithful adventure replays keep their commands, only `ruleset_hash` in their headers moved (the two strict ones are unchanged). Also in T1c: a walker's path within its move is the original's pathfinder's (BAT-MOV-003: at a tie, the first position the search took, neighbours from top right clockwise) and the berserk turn follows that search (BAT-MAG-025); the battle's results for the same commands are unchanged in every adventure replay.

- 2026-10-20 (the battle lane, E23 T1a3d, `installation.md` §7.39; for your AI, E10, nothing to do now): **two of the adventure AI's routines are read**, as far as the battle's odds use them. `ai_caster_record` 0x526D40 sets up a hero's caster record (the army's value, Spell Power and Knowledge held to 1–99, the maximum mana by HER-MAN-001's formula, the army's entries sorted by value) and `ai_best_spell_value` 0x5275B0 gives the best value 0x5273D0 finds among the hero's available spells of a mask of AI bits (the spell flags `0x8000`–`0x100000`, §7.6), by kind: 0x5270E0 (damage, a curve of 10 × damage / the army's strength, table 0x640398), 0x5271C0 (damage to every stack), 0x527290 (other battle spells), inline for raised stacks and the map's spells. The mastery reads the field of the hero's own tile (0x4E4FA0, 0x4FCF40). Not read: the seven other callers of 0x526D40 (their map entry: the adventure AI's battle estimate reached from the map objects' visits, the secondary skills' value 0x524690, and others) and the masks they pass. Part A's rest (about 2.4 KB, §7.34) stays yours to read with your AI.

- 2026-10-19 (the battle lane, its inbox's consolidation; for E08 to E10, nothing to do now): what E04b's closure (T11a, 2026-10-11) left for the adventure's use of `BattleOutcome` beyond what `E08-design.md` already holds (Eagle Eye's `learned`, Titan's Thunder's book, Necromancy): a reused created stack's record loses the kills of the stack it replaced (BAT-QUIRK-11), so experience and Necromancy count from the outcome as reported; the experience of killed Demons a Pit Lord raised comes with them; learning outside battle (Mage Guild, scrolls) tests the spells a hero has learned, not the ones available to it. And leads from the battle lane's decompiler trial (2026-10-12, `docs/lanes/battle/exe-decompile-trial.md`, "Extra findings"; unconfirmed) for the part of `hero_max_land_movement` (0x4E4990) your map entry marks not read: land and sea bonuses by artifact and specialty, a Logistics specialist's rate growing with level, and a computer player's hero getting +75 points (+125 with a further player-record condition; H3API's field guesses). (T34a read the function whole, survey `E08-hero-movement.md` §1.1: the artifacts and the Logistics specialist confirmed; the computer's +75 on Expert and Impossible, +50 more for `ai_type` 2, players 0 to 5 only: E10's.)

- 2026-10-15 (the battle lane, E04c T9, `battle/e04c-t9` #233, `cross-lane`; for your information, nothing to do): the quirk flag `quirk.battle.no_defender_ballista_at_fort` (BAT-QUIRK-23, registry position 25, after your ADV-QUIRK-6 at 24) is on in `base:faithful`: a battle at a town with a fort sets up no Ballista for the defender (BAT-SIE-029; strict-fixed sets it up). The six `faithful` adventure replays were re-recorded with their options: only the core ruleset hash in their headers moved. Whoever derives a siege's `BattleSetup` (E09) lists the defending hero's war machines as anywhere else; the battle applies the quirk.

- 2026-10-15 (the battle lane, E04c T7e, `shared/e04c-t7e-seam`, ADR-0013 point 4; for your information, nothing to do): the hero-effect kind `artillery` gains `level` (1 to 3; pack format v43, core ruleset hash v45; the base pack's Artillery gives 1, 2, 3) and `BallistaControl` the same field, the highest of the hero's sources (`h3-heroes`' `machine_control`, HER-BAT-005). The eight adventure replays were re-recorded with their options in that pull request (`cross-lane`): only the core ruleset hash in their headers moved.

- 2026-10-14 (the battle lane, E04c T7, `cross-lane`; for your information): the base pack gains the Arrow Tower creature (149, ability `turret`): `h3-map`'s creature codes gained it, and the six adventure replays were re-recorded with their options (only the core ruleset hash changed). The battle sets a town's turrets up from `Siege.structures` (the keep 4 + structures, BAT-SIE-016): the town context's derivation (E09) gives it as the original's 0x5BF4F0 counts it.

- 2026-10-14 (the battle lane, E04c T6, `cross-lane`; for your information): the battle sets up the attacker's Catapult for any attacking hero at a town with a fort without testing its Catapult artifact (the original's 0x463600, `installation.md` §7.29; BAT-SIE-009); `CatapultControl { level }` (#191) carries the hero's Ballistics level, which `h3-heroes` derives from the new `ballistics` effect (HER-BAT-018). The base pack gains the Catapult creature (145): `h3-map`'s creature codes gained it, and the six adventure replays were re-recorded with their options (only the core ruleset hash in their headers changed). The Catapult artifact's gap-fill pending row is now E08-E10's (the artifact every hero carries). The battle lane may still touch `h3-heroes` in T7 (Artillery's command of the arrow towers).

## Older notes for later work (before 2026-10-19), newest first

The lane's own notes of earlier increments and the cross-lane notes that go with them: read the ones that name the thing a task builds on.

- 2026-10-16 (the battle lane, E04c T10c, the closure; its `cross-lane` pull request): **E04c is closed; what it hands on.** (1) **`h3-heroes` is yours** (ADV-Q13, decision 25): `shared/e04c-closure` moves `crates/h3-heroes/` and `docs/rules/catalog/heroes.toml` to this lane in `lanes.toml`; your HANDOFF's ownership line can drop "as soon as", and the battle lane's changes to them are `cross-lane` from now on. (2) **The game's version** for a battle at a town: the original's `mapKind` (0 RoE, 1 AB, 2 SoD) decides whether a town's moat exists and does damage and the Tower's mines (`Siege.moat`, ADR-0023 point 2, BAT-Q41); the new game's setter was not read (probably the map's format): the town context derives `Siege.moat` from it. (3) **The Catapult artifact:** the battle sets up the attacker's Catapult for any attacking hero at a town with a fort and reads no artifact (BAT-SIE-009, ADR-0023 point 6); the hero keeps the Catapult artifact the original gives every hero (your note of 2026-10-14, T18b, on VCMI's `initHero`). (4) **The original's battle AI**, the neutral seat's faithful policy (decision 27): BACKLOG E23 (the battle lane), sized at the closure; your random stand-in plays until then. (5) **Your nine replays** (T10b, BAT-Q50 (1)): the five searched ones were replaced from a new search (`duel-s1279-d28`, `duel-s810-d28`, `duel-s328-d28`, `duel-s1122-d28`, `duel-s307-strict-d28`), and none now declines a pile's question or a Cartographer's offer: a wider search, or a fixture of its own, would restore them.

- 2026-10-15 (T20a2a): **not built, the grounds' other effects:** ~~Cursed Ground's loss of the native price~~ (built by T40 for both pictures, batch E5's E5-6: `StaticTile::no_native`); Favorable Winds (225) a third of a step's cost at sea, with boats (the sketch's increment 7 since decision 90). The typed grounds give it a reader (`MapObject::lies_on`).

- 2026-10-15 (T20a2a): `Pandora's Box ` no longer imports whole (a monster's artifact, decision 22): 2 shipped maps import whole until rewards on the map (the sketch's increment 7 since decision 90; `Race for Ardintinny`, `Tutorial.tut`; T21b: the 4 counted then held the user's `Untitled` and `experiment`).

- 2026-10-15 (T20a2a, for the client lane's viewers): `ObjectView` shows monsters (creature, the word of the count) and special grounds (`drawn`: the tiles a ground lies on).

- 2026-10-15 (the battle lane, E04c T8, `installation.md` §7.32; for you, with the town context): **a battle at a town and its consequences.** The battle's outcome stays without the town (ADR-0023 point 4); the orchestrator applies, from the setup's `siege`: the experience (0x46A350: +500 for the attacker in a battle at a town, any fort level, beside the +500 for a defeated hero, and 500 less when the battle ended in a retreat or a surrender), the defending hero's mana after the battle (the smaller of the mana left and before; its skills restored, ADR-0023 point 3), and which army defends the town (the battle start's arguments, 0x463A7F: the spell cost's Pegasi and Mages read those armies). The battle itself: the town's defender never surrenders and retreats only through the Escape Tunnel (BAT-SIE-024), the town's morale and luck reach the defender's stacks (BAT-SIE-025). For the derivation (`Siege`, ADR-0023 point 2): `morale` +1 for a Tavern, +2 in all for a Castle with the Brotherhood of the Sword (it replaces the Tavern's, batch 3g CM, survey D-21; the wiki's "in addition" is refuted), `luck` +2 for a Rampart with the Fountain of Fortune, `escape_tunnel` for a Stronghold with building 17. The two replays with battles were re-recorded in the battle's pull request (`cross-lane`): only their hashes moved.

- 2026-10-14 (the battle lane, E04c T7d, `shared/e04c-t7d-seam`, ADR-0023 point 7; for you to map): `BattleSetup` gains `difficulty: u8` (0 Easy to 4 Impossible) and `human: [bool; 2]` (each side's player is human; false for monsters or a neutral town). The battle reads them only for the choices it makes as the original's computer (an automatic turret's desperate aim, `installation.md` §7.31); they decide nothing about who commands. `h3-game`'s battle start (`state/battle.rs`) keeps the defaults (0, no human) until it sets them from the game's difficulty and its players.

- 2026-10-14 (T20a1): HER-MAN-001's open question is closed: `map_hero_setup` equips the map's artifacts before it sets the spell points (0x4866D5, 0x48675C), so +0x479 includes them, as the rule has it.

- 2026-10-14 (T19b2): the plans fight with `Step::Fight` runs and a hero cast in three fighting steps of four; the game's legal commands list no casts (a separate action head), so a client or agent on `h3-game` alone cannot list them: E05/E13's batch API or the server lane's protocol decide whether the game should (the env and the server use `h3-battle` directly). A future monster battle (T20b) is played by the same driver once the neutral seat exists.

- 2026-10-14 (T19b1): **random play seldom fights.** Of 1 500 duel games of 28 days 9 have a battle; on the XL map the heroes never met in 200 days (30 seeds). The fork benchmark lets the heroes seek each other (`h3-cli`'s `bench::seek`), and since T19b2 the plans' driver; an RL agent will need shaping or curriculum to meet the enemy (E05/E06). The random agent plays a battle with the battle's agent, which never retreats or surrenders.

- 2026-10-14 (T19b1): ADR-0017 point 8 has `to_move` yield seats; it yields players until T20b's neutral seat (decision 29). A hero's slot shifts when a battle's loser leaves the map (the 2026-10-11 note below): an agent's slot policy sees it from T19b1 on.

- 2026-10-14 (T19a2b, not built): the Necropolis buildings' +10/+20 % Necromancy (0x4E3CD0 with the building bits 0x66CE40, 0x66CE68: towns, E09); the original's exception to the full army's upgrade for the four elementals (112 to 115 while the game flag +0x1F698 is 0; no hero raises them); its 32-bit product of the hit points and the killed, which wraps above 2^31 (no army reaches it). VCMI's full-army rule differs (any direct or indirect upgrade in the army, at 2/3 of a count computed with that upgrade's hit points): a batch E3 row (a full army with Skeleton Warriors) would settle it; the engine follows the code. Decision 45: the Skeleton Warriors row is not batch E3's; increment 4's batch.

- 2026-10-14 (T19a2a, the reading): not built, each for the hero's artifacts (increment 6 since decision 90, T44a's survey point 5): a received part that completes a combination (SoD: `hero_give_artifact` 0x4E3070 asks the local human, builds at once for a computer player; a pending decision then), the spell book that equipping Titan's Thunder adds (0x4E2A64; a pack field, the harvester's report lists it), the Grail kept out of the capture (0x46A070 skips artifact 2; the packs do not mark it).

- 2026-10-14 (T19a1): a battle that is over at its start (`BattleState::start`: a war machine's turn before the first command) ends through the same path as a command's end; no setup of the tests reaches it.

- 2026-10-14 (T19a1): in simultaneous play a player may hold an offer and a battle at once (another of its group attacks it): its first decision decides its commands (the offer's answers first), as a test builds it; a property with group schedules when simultaneous play is built.

- 2026-10-14 (T18b): VCMI gives every hero a catapult (`initHero`); the original was not read; it matters in sieges only (E04c, E09). VCMI also skips an artifact that does not fit its stored slot and a war machine in the backpack; the import keeps them and the validation refuses them (no shipped map that imports has one).

- 2026-10-13 (the client lane, E17 M2 T6; its `cross-lane` pull request): **agents play the adventure's batches on the harness.** `python3 capture/h3adv.py batch capture/batches/<batch>.toml` (from `clients/`) compiles the spec, plays it in hot seat under Wine and fills the results template (`clients/out/batches/<batch>/<batch>.md`), every record read from the game's memory; E08 batch 2 came out as the user's results (D3a–D3d every field, D5 within ranges; `docs/epics/E17-design.md` §6.3, `docs/experiments/README.md`). A batch needs its recipe in `clients/capture/batches/` (client-owned: the procedure as steps, a hero, a tile clicked twice, the fields each step fills): ask for it in the client lane's inbox when the spec is written. Not built yet: the hero list's scrolling (at most five heroes per player), the underground, Yes/No dialogs, battles started from the map, a second day. Increment 3's batches (monsters and battles) will want the battle part: put what they need into the client lane's inbox when T17a plans them.

- 2026-10-13 (T17b, the user at ADV-Q12): **feature to-do: a joining group that finds no room.** Increment 3 skips the join offer, as declined, when the hero's army has neither a stack of the group's creature nor a free slot (`ADV-STUB-2` at the skip, T20b); the original shows its army dialog (0x5D11D0, not read) so the player can make room (dismiss or merge stacks) and then accept. With army management (increment 6 since decision 90, T44a's survey point 1): read 0x5D11D0, build the dialog as a pending decision, close the stub.

- 2026-10-13 (T17a): **increment 3's batch** (survey §13: E3-1 to E3-9) needs the experiment spec to place monsters (creature, count, disposition, never flees), heroes with custom primary and secondary skills and armies, and to name the game's difficulty (`h3-experiment`, battle-owned: `cross-lane`). On the client lane's harness (the item above) it needs what the harness does not do yet: Yes/No dialogs (the offers), battles started from the map and their stacks' counts, the underground (E3-4), memory reads of a monster's tile word (aggression, E3-3) and of the 156 heroes' experience (E3-7): in the client lane's inbox since T17b (`cross-lane`, the user at ADV-Q12); the recipes follow with the spec (T22a).

- 2026-10-13 (T17a, for the battle lane through T17b): the original and VCMI draw a battle's obstacles from the tile (`srand(110291 x + 167801 y + 81569)` and the C runtime's `rand`, survey §4.2); `h3_battle::generate_setup_obstacles` takes a kernel `Rng`. Whether the engine reproduces the original's obstacles (CLT-Q4's note) is the battle lane's; where the seed comes from: decided (decision 24; the ADR-0017 draft's point 4: the tile's seed for the obstacles, the attacker's stream for the battle).

- 2026-10-13 (T17a): the original's army dialog when a joining group finds no slot (0x5D11D0), the computer player's answers (0x527F60, 0x52BD10) and the battle's later draws after the tile's seed were not read (survey §3.3, §6.3, §4.2). The dialog: T44a's survey point 1 (increment 6).

- 2026-10-13 (the battle lane, E04c T3; its `cross-lane` pull request `battle/e04c-t3`): **the battlefield's tile and the writer.** Batch 3b (run by an agent on the capture harness): the battle's map tile is the **defender's** (the attacked monster's): a monster on a Clover Field or next to water gives the field or the shore, a hero's own tile does not (`docs/rules/surveys/E04c-battlefields.md` §4); batch 3a: the SoD pictures 223 and 230 fight on Cursed Ground and Magic Plains (D-4 settled). `h3-map`'s writer now places the special grounds (`MapObject::Ground`, classes 21, 46, 222–231, no body) and sets the coast flag `0x40` on land next to water (every shipped map has it; without it the game sees no coast: `docs/formats/h3m.md`); the import keeps a ground as an untyped object with no blocked tiles. For agent-run batches: the single scenario's random starting bonus can give the hero an artifact (`docs/experiments/README.md`).

- 2026-10-13 (the battle lane, E04c T2b; its `cross-lane` pull request): **a battle on the map names its battlefield.** `BattleSetup.battlefield` is a battle-part key (`base:shore`, `base:magic_plains`, `base:cursed_ground`, `base:deck`, `base:boats`; `sod:holy_ground`, `sod:evil_fog`, `sod:clover_field`, `sod:lucid_pools`, `sod:fiery_fields`, `sod:rocklands`, `sod:magic_clouds`) and `.anti_magic` marks an anti-magic garrison; the orchestrator derives both in the original's order (`docs/rules/surveys/E04c-battlefields.md` §4, `docs/formats/installation.md` §7.26: a siege; a hero in a boat or a Shipwreck or Derelict Ship: water and the deck, both heroes in boats the boats; underground: subterranean, S-22; a crystal, gold or abandoned mine: subterranean; a tile next to water: sand and the shore; then a special ground object on the tile replaces the field; dispute D-4: the SoD objects 223 and 230 are not in 0x4FCE20's list) and draws the obstacles with `h3_battle::generate_setup_obstacles`.

- 2026-10-13 (T16a, measured): the XL observation's object list costs about 1.6 M Ir per observation with 1 056 typed objects (+543 µs per observed XL step, +256 µs without objects): every object's tiles get the fringe test. Build the object plane once with E13's encoding.

- 2026-10-13 (T16a): random resources are left out of the property plans because the hygiene property would see the drawn type; a hygiene variant with a fixed seed would let the plans draw them (unit tests, the env tests and the duel's replays play them).

- 2026-10-13 (T16a): of the 927 seeds searched for the replays, 2 met a border guard's pass and none crossed a gate (a crossing has no event of its own): an agent that prefers visits, if a later increment wants these in the matrix.

- 2026-10-13 (T16b): the section comment in `crates/h3-map/src/tests/mod.rs` ("the experiment writer's maps (h3-formats, until the writer moves here)") is stale since R01a.

- 2026-10-13 (T14 review; the gate's half closed by T15b): a future entered pickup or removable entered object could leave a hero on a tile whose next live object is neighbour-visited, a state setup refuses; T15 cannot reach it (pickups and guards are visited from a neighbour, entered objects and gates never leave). A key holder stopped on a gate is a legal play state that setup would refuse (no keys at setup).

- 2026-10-13 (T14 review, T15a): `AdventureState::accept_offer` is `pub` and trusts its `Offer` (another type would be marked bought without a reveal; a `Pass` offer removes whatever object it names); only `h3-game` calls it. Narrow it if a second caller appears.

- 2026-10-13 (T14b, T15a): the original's messages have no event: a Cartographer kind already bought, a guard visited without the key, a tent whose key is held (a visit tells nothing).

- 2026-10-13 (T15a): a stale offer (`OfferGone`: the guard left before the answer) needs two players in one turn group, which only `h3-game`'s test `set_schedule` builds (a fixed test covers it): a property with group schedules when simultaneous play is built. The shipped maps have no two typed objects of one type on one anchor (0 of 164; the T14 verify's line closed).

- 2026-10-13 (T14b, measured): fork levers: the heroes' clone is 15 % of `fork-xl-144` (an allocation per army), the schedule's `Vec<Vec<PlayerId>>` 3.6 %.

- 2026-10-13 (T14a): the draw-list order at 0x505230 (equal and lower layers) was not read; it could make the original visit an earlier object on a stacked tile. No shipped map stacks visitable tiles (0 of 101 246).

- 2026-10-13 (T14a): untyped blockers have no from-above setting, so a hero on a town's gate leaves upward: type the town classes with E09.

- 2026-10-13 (T14a): what the original does with a map that puts a hero on a pickup's tile was not read; setup's refusal (`StandError::Visited`) relies on the editor preventing it.

- 2026-10-13 (T14a): random play enters a Cartographer about 3 times per 48 cases (T14b's plans raise offers to 9–16); place heroes beside Cartographers if more is wanted.

- 2026-10-12 (T12b): `ObjectView` has no object index (it would count unseen objects); T14's offer and its events name the object by its visitable tile or anchor, not by index, in what a player sees.

- 2026-10-12 (T12b, not measured): `remove_object` scans every object for each tile it frees (other objects covering it); a per-tile cover count in the static map if pickups show up in a profile.

- 2026-10-12 (T11a): not read, each for the increment that needs it: the computer player's own handling of the Cartographer and observatories (the human path only), how the loader fits a pile amount above 0x7FFFF (T12 bounds it), the game flag +0x1F69D's 50/50 000 start (probably the tutorial), the computer players' daily resource bonus by difficulty (the wiki's Difficulty level; VCMI has a weekly one; with computer players, E10), the flying and boat test before a visit from a neighbour (0x4E5CE0: read in T34a, survey `E08-hero-movement.md` §3.3, A-14), the wiki's "tournament rules" start on impossible (5/2/2 500, probably HD).

- 2026-10-12 (T11a): the original lets a hero visit the object it stands on again (the space bar); a "visit here" command would let a player re-ask a declined Cartographer without stepping off: with an increment that needs it.

- 2026-10-11 (T9b, measured): the reachable-tiles query is 96 % of a random step (≈ 263 k Ir per query on the duel, 342 k on 144 × 144; of a query `Reach::search` 56 % on its own, the step price 28 %, the heap 7 %; zeroing map-sized arrays 5 % of a step): the first lever on the adventure's speed (`benches/adventure/` gates it), for an increment that needs speed or a performance task.

- 2026-10-11 (T9b, measured): an agent reading both mask heads before a step doubles the duel's step (70.9 → 141 µs; ×1.7 on 144 × 144): `legal_mask`, `tile_mask` and the step's decode each run the query. Keep the current state's queries in the env, with E05's batch API (or E13's macro-actions), which owns the mask API.

- 2026-10-11 (T9b, measured): `observe` rebuilds one `TileView` per tile: +15 µs per duel step (+21 %), +99 µs on 144 × 144 (+54 %). Cached static planes with E13's flat encoding. (Closed by the same measurement: the reveal's per-row writes, `fog::reveal` 0.8 % of a step.)

- 2026-10-11 (T9b): the battle workload's `collect = "*random_rollout*"` also names the adventure's `random_rollout`, harmless (the battle's run never calls it): the battle lane may qualify it (`*h3_sim::rollout::random_rollout*`) as hygiene.

- 2026-10-11 (T9a): a player's hero slots are its heroes by id, the first eight; they shift when a hero of a lower id joins or leaves (recruiting, defeat: increments 3 and later). Stable slots (a slot table in the game, as the original's hero list) when an agent needs them (E13), with an encoding version bump.

- 2026-10-11 (T9a, the user): the env's observation is `h3_game`'s view; a flat, versioned encoding comes with E13's macro-actions (or E05's batch API). Episodes end only by the day limit and carry no reward until victory and defeat exist (E10).

- 2026-10-11 (T8b2b): the hygiene test compares two games while the other players' commands have the same outcome in both (about four steps in five over 200 cases); it varies the streams only through the seed (no command draws yet, see the T8b2a note below).

- 2026-10-11 (T8b2a): moves are not listed one by one (the user): `h3_game::legal_moves` gives a hero's `Reach`. T9's env builds its mask from it (a bit per tile and hero; ADR-0017 point 8) and the path only for the action taken.

- 2026-10-11 (T8b2a, the user): clients compute their own paths and `apply` validates them. For the client and server lanes: the pathfinder on a player's observation instead of the state (moves go to explored tiles only, where the observation shows all the query reads: terrain, roads, objects, visible heroes, the own hero's army and points), or a server query for the reach of the player's own hero.

- 2026-10-11 (T8b2a): the play properties (`crates/h3-map/src/tests/play.rs`) take about 6 s for 3 × 48 cases in a debug build (12.6 s since T19b2's fighting runs). `Plan` and `lay_out` are test code; the generated maps are written once by a test and committed (T9a, T9b).

- 2026-10-11 (T8b1): the fringe is the eight tiles around an explored one (ADR-0016's "touching"); how the original draws its half-dark border (which hidden tiles get an edge sprite) was not read: a code reading or a look in SoD with a single explored diagonal, with T10's batch.

- 2026-10-11 (T8b1): of another player's hero the observation shows tile, owner and identity (ADV-FOG-006, the user): what another player's hero shows: survey `E08-battles.md` §9 (T17a). The spectator viewer for the adventure when the server lane hosts adventure games.

- ~~2026-10-11 (T8a): the Scouting specialty multiplies the radius by 1 + 0.05 × level (0x4E4070, truncated), not "+1 every six levels" (survey §2): compare with the wiki and VCMI in T34a's survey (increment 5, decision 71).~~ (T34a, survey `E08-hero-movement.md` §4.1: no SoD hero has the specialty; HotA's "+1 every six levels".)

- 2026-10-11 (T8a): the original's reveal sets the bits of every player of the revealer's team (allies share the map; VCMI too): with teams (E10). ~~The new day reveals again around everything a player owns (0x4C7C50): with the first rule that hides tiles (Cover of Darkness, towns).~~ (T34b: only after a Cover of Darkness hid tiles, 0x4C7BA0; with it, E09.)

- 2026-10-11 (T7b): the step's price leaves out Pathfinding, Nomads, flying, water walking, Favorable Winds and Cursed Ground (0x4B1740 has them): ~~increment 5 (decision 71: T34a's survey; water walking and Favorable Winds with boats), Cursed Ground with objects~~ read in T34a (survey `E08-hero-movement.md` §2): all but Favorable Winds (boats) for increment 5 (decision 74, ADV-Q20).

- 2026-10-10 (T6b review): building a game emits no events (the first `TurnStarted` goes to a `NoopSink`): a replay needs none (T9b: commands and hashes); a client's event stream (server lane) needs a start event or reads the start from `observe`. `GameState` keeps no hero types yet: the game's store of heroes (hero details) comes with `h3-heroes` in increment 3.

- 2026-10-10 (T6b): the original plays every human player before every computer player each day (0x4C6FE0; VCMI `TurnOrderProcessor` too). The engine has no human/computer distinction; decide with the server lane's seats or the turn-group work whether a schedule should order players so (GAM-TRN-001's note).

- 2026-10-10 (T6b): weeks of seven days and months of four (wiki Time; VCMI parameters) come with the first rule that reads them (week events: the week's increment, 6 in the sketch); GAM-CAL-001 counts days only.

- 2026-10-10 (T6b): the starting army's other callers (the tavern, 0x4C86A0 and 0x4D9EC0) give one creature of the first stack; for the towns epic (a hero that lost a battle on the map goes back to the pool through 0x4D9EC0: survey `E08-battles.md` §7.4). The starting army's war machine: survey §6.2.

- 2026-10-10 (T6b): `docs/formats/installation.md` §7.1 says the nine war machines are the heroes' *third* starting creature; the hero table has them second (battle-owned doc; tell the battle lane or fix it with the formats lane).

- 2026-10-10 (T5a/T6a): for E09 (towns): a hero placed on a town's blocked tile is the town's visiting hero (VCMI `CGameState::placeHeroesInTowns` moves it to its own town's entrance, an "H3M format quirk"; 3 shipped maps), and a hero generated at a main town (129 maps); both are `Unsupported` in the import until towns exist.

- 2026-10-10 (T5a/T6a): not checked yet, each for the increment that needs it: a hero type placed twice on one map, the most heroes a player may have on the map (8 in the original; with recruiting); a war machine in a hero's custom army: none in the shipped maps (survey `E08-battles.md` §6.1).

- 2026-10-10 (T4): check `packs/base/adventure.toml`'s land table against the installation's `movement.txt` under `--with-original-data` (an `examples/original_data.rs` in `h3-pack-adventure` needs `h3-formats` as a dev-dependency to read the LOD: a shared `arch.toml` change), or harvest the table when adventure data is harvested.

- 2026-10-10 (T3b): ~~fixtures in `fixtures/maps/` from the user's editor maps, so CI reads real editor output~~ dropped by the user (2026-10-11: too little gain for the time); reconsider only if a reader or import bug shows up that real editor output in CI would have caught. The shipped maps stay the reader's test, locally (`verify --with-original-data`).

- 2026-10-10 (T2): the game context's rule ids use the existing prefix `GAM-` (`check-rules`, catalog file `docs/rules/catalog/game.toml`), not `GAME-` as the T1b survey proposed.

- 2026-10-10 (T2): already allowed by `arch.toml`, so T5–T9 need no shared change: `h3-sim` → `h3-adventure`, `h3-game` (dev: `h3-map`, `h3-pack-adventure`); `h3-cli` → the four crates; `h3-map` dev → `h3-pack`, `h3-pack-adventure`. Golden replays go to `fixtures/replays/adventure/` with the test `crates/h3-cli/tests/adventure_replays.rs` (both adventure-owned): `cargo xtask determinism` runs it once it exists, and `verify --scope adventure` runs the determinism step once the folder has files (T9).

- 2026-10-01 (E02): ~~tight formations~~ (done in E03 T9) / tactics (E04), creature-bank layouts (with creature banks), and other harvested tables (spells, artifacts, heroes, buildings) when their epics come; RoE/AB patch differences outside creatures need the same gap-fill treatment (Q15).

- 2026-10-01 (E04 refinement): the `.h3m` writer built for the experiment pipeline (E04a T2) overlaps with E07 (map reader) and E14 (map generator). ~~Keep it in the formats adapter so both can extend it~~: E08 decision 2 moves it into `h3-map` after E04b closes, beside the reader (2026-10-08).

- 2026-10-03 (E04a T3a): not harvested yet, each for the epic that reads it: spell town chances and hero-class tavern weights and aggression (E09 towns and taverns; RoE lacks the Conflux column, so the `ab` pack needs an override kind or a per-faction table), spell AI values (E05/E06 if wanted), the spell flags and target attributes (E04b T1/T2a, decode the executable's flag bits, installation.md §7.6), hero availability (campaign heroes from the executable's flag byte 2 and VCMI's `special`; Lord Haart the Knight retired by the expansions, flag byte 1; Sir Mullich: VCMI special, executable not campaign; E08–E10, check in the SoD map editor and taverns), specialty names (presentation).

- 2026-10-03 (E04a T3b): artifacts not modelled in E04a, each for its epic (the packs mark them `# pending`, `packs/README.md`): Necromancy artifacts with the Necromancy skill and the raised creature by level (dispute D8; E08–E10), combination costs (blank in the original; artifact merchants, E09), the artifact class of Vial of Dragon Blood and Armageddon's Blade for random placement (D4; E08/E14), backpack capacity (D7; increment 6 since decision 90, T44a's survey point 5).

- 2026-10-01 (E04 refinement): ~~hero progression (experience table, level-up algorithm with the class probabilities, skill offers)~~ (E08 increment 4, T24a) and the hero commands (equip, unequip, learn, trade) are deferred to the adventure and game epics (E08–E10). Equipping, unequipping and the exchange between a player's heroes: increment 6 (decision 90); learning with rewards on the map and towns.

## Inbox notes acted upon (moved 2026-11-03, E08 T86b1)

The town lane's note, verbatim; fixed by T86b1 (the battle oracle's `visited_at`, the case
committed in `crates/h3-map/proptest-regressions/tests/play.txt`):

- 2026-11-03 (the town lane, E09 T3a, PR #659; **a failing property case on `master`'s code**):
  CI's full verify on #659 (run 41541601650, the merge with `master` at 65b5465b; #659 does not
  touch `h3-map`) found a case of `h3-map`'s `tests::play::every_step_keeps_the_commands_invariants`
  that fails: "no monster or guards fought at Pos { x: 1, y: 5, level: 1 }" (play.rs, the battle
  check's `visited_from` / `events_at` search). Reproduced locally in 15 s by appending
  `cc 687f80afb754de647c8cf145b1e0fbbdbb492e3c0aa8b2cb8c6fdbf49d90ad42` to
  `crates/h3-map/proptest-regressions/tests/play.txt` (not committed: it would fail every run until
  fixed). The shrunk plan: RoE, side 7, two levels, nomads `(222, 11)`, one timed event (players
  3, from day 4, every day), 461 `Fight` steps; the full input is in that run's job log. Master's
  own runs passed (random cases). Commit the seed with the fix.

- 2026-11-04 (the town lane, E09 T7b1, `town/e09-t7b1`; **for T94b, when both pull requests are
  open**): `h3-game` gains the hero pool (`GameState::pool`, `town::pool`), game state hash v23,
  and the 39 adventure replays are rehashed; `GameState::new` draws the players' Random choices a
  start reads and the random heroes' types before the starting armies (no fixture map has one, so
  only the hashes moved). Whichever of T94b (adventure state hash v20) and T7b1 merges second
  merges `master` in and rehashes the replays again (skill `homm3-rerecord-replays`).
  `end_battle` now frees the hero types of the heroes it removes from the map.
  (Done by E08 T94b, #716: `master` merged in, the replays rehashed again.)

- 2026-11-04 (the town lane, E09 T7b2a, `town/e09-t7b2a`; **for T95b, the versions**): T7b2a
  takes game state hash v24 (the store of towns, `GameState::towns`, hashed after the pool; your
  HANDOFF's versions line updated), so T95b's is v25. The build now draws each random town's
  faction and each town's garrison and buildings from the setup stream before the starting armies
  (design note E09 §14), so the seed-0 draws after them move on maps with towns (the census's
  guarded dwellings 691, 695 before).
  (Done by E08 T95a2, #724: T95b's queue entry, the HANDOFF and design §15 say v25.)

## Inbox notes acted upon (moved 2026-11-05, E08 T96b1)

The three failing seeds of `h3-map`'s `tests::view::a_viewer_learns_nothing_it_cannot_see`, verbatim;
fixed by T96b1 in the test's `vary` (no engine change), the seeds committed in
`crates/h3-map/proptest-regressions/tests/view.txt`:

- 2026-11-04 (the town lane, E09 T7b2a, `town/e09-t7b2a`; **for the 2026-11-03 note's task**):
  another seed of `h3-map`'s `tests::view::a_viewer_learns_nothing_it_cannot_see` failing on
  `master` ae59f3b at `view.rs:1207` (the viewer's events at day 2's start differ by a
  `ResourceGained` of player 1): `cc e44ebd639ea55b7cce88a74f668c8a98053fa52b72eeaee3c64f471260697aba`
  (an RoE plan, side 15, two levels; not committed).
- 2026-11-04 (the town lane, E09 T7a2, `town/e09-t7a2`; **a failing property case on `master`'s
  import, for a task of yours**): the scoped verify's coverage run found a case of `h3-map`'s
  `tests::view::a_viewer_learns_nothing_it_cannot_see` whose laid-out map the import refuses:
  `start_on` (`tests/play.rs:1478`) unwraps `Err(Unsupported(OwnedOnWater(12)))` (an RoE plan,
  side 15, one level, its third tile water). T7a2 changes neither the import, the writer nor these
  tests, so `master` has it too (CI's runs passed on other random cases). Proptest's persisted
  seed, not committed by T7a2 (it would fail every later run): adding the line
  `cc f1e6f5bccdc743b549410caa31f1089b743afa8f5432e8ffc10073eb14d036fc` to
  `crates/h3-map/proptest-regressions/tests/view.txt` replays it. Likely the plan generator
  placing an ownable object (T93b's dwellings) where it is entered from water.

- 2026-11-03 (the adventure lane, E08 T89a, PR #688; **a failing property case on `master`'s code,
  for a task of its own**): CI's full verify on #688 (run 41694189742) found a case of `h3-map`'s
  `tests::view::a_viewer_learns_nothing_it_cannot_see` that fails at `view.rs:1405`: the viewer's
  events at day 2's start differ between the game and its copy with the unseen changes,
  `ResourceGained { player: PlayerId(2), resource: #5, amount: 1 }` in one and not the other (the
  viewer's own income moved by something it cannot see). Independent of T89a (the test lays out a
  `RawMap`, no writer); reproduced locally in 12 s by appending `cc
  4ceb0bd67ba12482c10781aa43bd0edf62cdcbf92752404a3c05e6a995b42fdf` to
  `crates/h3-map/proptest-regressions/tests/view.txt` (not committed: it would fail every run until
  fixed). The shrunk plan: RoE, side 5, one level, the full input in that run's job log
  (`gh api repos/ChristopherRotter/homm3/actions/jobs/<id>/logs` reads it). Commit the seed with
  the fix.

## Inbox notes acted upon (moved 2026-11-05, E08 T97b1)

- 2026-11-04 (the town lane, E09 T7a1, #710; **increment 10's dwellings on water, for a task of
  yours**): with #710 the import reads the static layer before the players (a main town needs the
  towns), so a refusal the players' `HeroAtMainTown` used to hide now shows: 4 shipped maps are
  refused as `Unsupported::OwnedOnWater` by an object T93b made ownable (`OwnedOnWater(index)`:
  Adventures of Jared Haret 123, Rise of the Phoenix and its Allied version 2268, Serpents
  Treasure 2771; T83a's census found no other ownable entered from water). The variant's doc
  comment still says none of the shipped maps has one. E09's exit criterion 1 holds (a typed
  refusal, not a town's); E08 increment 10 decides whether such a dwelling is built. *Decision 256
  (2026-11-05): settled at T97's audit (or carried into increment 11); the typed refusal stays.* *Done by E08 T97b1 (decision 259, 2026-11-05): such a dwelling is built (survey `E08-dwellings.md` §17, map import v19); all 160 shipped maps import.*

## Inbox notes settled or for information (moved 2026-11-05, E08 T97c)

- 2026-11-02 (the town lane, E09 T2b, `TWN-Q4`; **for your merge order, nothing to build**):
  E09's build bumps your versions `cross-lane` (`docs/epics/E09-design.md` §19, §21): adventure
  replay format v9 (T5a: the town part's hash in the header; `GameRuleset::new` gains the town
  part), map import v16 (T7a), game state hash v23 after your T85b's v22 (T7b), adventure ruleset
  part v16 and state hash v20 (T8a: the kind `Town`, towns ownable); the towns' income joins your
  T85b's day advance (§16). None starts before your T85b's pull requests merge; the second to
  merge rebases and bumps again. *Moved by E08 T97c: settled: its bumps have all landed (replay format v9, map import v16 to v18, game state hash v23 and v24, part v17, state hash v21).*

- 2026-11-02 (the battle lane, E26 T5; for your information, nothing to do now): the fair
  system actor `h3_battle_ai::choose_fair` (ADR-0027 point 4 as amended) is to be the computer's
  default battle policy everywhere, the all-seeing `choose` a game's option, "player option too"
  (the user, AI-Q10). When the game's setup gets options, the sight is one of them; the neutral
  seat (`h3-sim`'s adventure env) is the AI lane's switch. `crates/h3-cli/tests/
  adventure_replays.rs`' uncommitted both-sides battles may use either. *Moved by E08 T97c: for information; the sight joins the game's options when they exist.*

- 2026-11-02 (the town lane, E09 T1a1; **for increment 10's random dwellings**): of the shipped
  maps' random dwellings tied to a town by its identifier (classes 216 and 217), all 1 397 (87
  maps) are tied to a random town (class 77), none to a fixed one; the original resolves the
  random towns' factions as the map's objects are read and the dwellings after them
  (`random_dwellings_resolve` 0x502B60). So these dwellings wait for E09's random towns (increment
  1, survey `docs/rules/surveys/E09-towns.md` §1.6, §2.1, §2.3); 205 take a faction set instead. *Moved by E08 T97c: settled: increment 10 closed; the tied random dwellings wait for E09's random towns as ADV-STUB-13.*

- 2026-10-31 (the client lane, E17 M5 T8b2, review R7-8; for information, nothing to do): the
  harness checks the starting bonus Gold it sets. `h3drive.gold_bonus` (a recipe's `gold = true`,
  `h3adv.to_hot_seat(..., gold=True)`) reads the scenario list's rows in memory
  (`h3mem.starting_bonuses`: H3API's `H3SelectScenarioDialog` pointer 0x69FBE8, `bonusType` +0x6C,
  3 Random, 0 Artifact, 1 Gold; observed) and stops the run unless exactly its players read Gold.
  Recipes without `gold` are unchanged. *Moved by E08 T97c: for information.*

- 2026-10-30 (the map client lane, E24 M3 T6a2b; the user's standing consent for the writer, E24
  T0; for your information, nothing to do): **the writer writes quests**: `Extras::quests` maps
  an index of `Extras::objects` (a Seer's Hut, 83, or a Quest Guard, 215) to a
  `model::Mission`, written in the SoD layout with no deadline, no texts and a seer's reward
  nothing (`writer::extras::write_quest`; refused on another class or with a list past 255).
  A literal `Extras { tiles, objects }` now needs `..Extras::default()`. *Moved by E08 T97c: for information; the quests' increment reads it.*

## Inbox notes settled or for information (moved 2026-11-07, E08 T103b)

- 2026-11-07 (the town lane, E09 T14b, branch `town/e09-t14`; the capture harness, informational):
  `h3adv` plays a walk's `actions` on the town screen when it opens (then closes what they left
  open), with the new actions `hall = true` (a click on the shown hall building, items 10 to 13: the
  hall, vtable 0x6437A0) and `slot = <building id>` (the hall's slot showing it: the building's
  dialog, 0x643944; `click = ["build"]` its Build 30722, `cancel` 30721), and the reads
  `town:x,y:mark`, `hall:slot:<id>`, `build:button`, `build:text`, `screen`, `town_screen_closed`;
  `h3mem`'s town record gains `mark` (+0x2), `h3lod` gains `building_names`. A walk without
  `actions` behaves as before. Learned: the panel's hall picture opens nothing; a map whose players
  are all allies ends at a visit's close (an enemy is needed). The harness check passed after.
  `clients/capture/README.md`, "The hall and the build".
  *Moved by E08 T103b: for information (the capture harness).*
- 2026-11-06 (the town lane, E09 T11, `town/e09-t11`, `cross-lane`; **`ADV-STUB-13` lifted, map
  import v20**): a random dwelling tied to a town is typed with `Holds::LinkedDwelling { town,
  levels }` (the `TownId` of the last town with its identifier; the holds' hash tag 10, which no
  built state holds). `AdventureSetup` has a new field `town_factions` (one faction per town by
  `TownId`, the game's step-2 factions; `SetupError::TownFactions`, checked after the walk, before
  `Garrisons`): a literal with towns sets it. In the placement walk a tied dwelling takes its
  town's faction (no draw) then draws its level (`dwellings::resolve` takes the factions); its
  tiles and names are checked against every table's outcomes. `GameSetupError::LinkedTown`
  refuses a tied dwelling's town that may take a faction without a table. The 55 replays'
  `import_version` moved alone; the property plans generate no tied dwelling (the writer writes
  none: `OwnedBody::RandomDwelling` keeps identifier 0).
  *Moved by E08 T103b: for information; landed (map import v20, the replays' `import_version`); T103b bumped to v21 after it.*
- 2026-11-06 (the town lane, E09 T9b1, `town/e09-t9b1`, `cross-lane`; **for your plans**):
  `h3-map`'s `play.rs` plays towns. `Plan` has a new field `towns` (`TownPlan`, drawn last by
  `towns()`; a plan literal sets `towns: Vec::new()`), laid out by `towns_laid_out` after every
  other object but the boats, each with a template of its own: a town's template is no `KINDS`
  entry, so code that reads a kind from a template skips `Body::Town` first (the town lane's
  `TRAPS.md`). New: `Step::Town` with its runs in `steps()` (as `Step::Hire` on a map without
  towns), `check_towns`, `check_town_battle_end` (`check_unowned_battle_end`'s town arm), the
  entrances in `check_invariants`, the towns' income in `check_income`, a siege's merge in
  `check_isolation` (`told_army`), the cover test `the_plans_capture_and_besiege_towns`;
  `h3-towns` a dev-dependency of `h3-map`. `view.rs` draws its plans without towns (`townless`)
  until E09 T9b2 teaches `vary` and `check_told` the towns.
  *Moved by E08 T103b: for information (the plans' towns, `play.rs`).*
- 2026-11-06 (the town lane, E09 T9a, `town/e09-t9a`, `cross-lane`; **for your replays and the
  writer**): `h3_map::writer`'s `MapObject` has two new variants, `AnyTown` (a town of any kind:
  random with its alignment, a main town in the header, a name, random garrison stacks) and
  `RandomHero` (the code in the town lane's `src/writer/town.rs`; `docs/formats/h3m.md`, "The town
  lane's additions"); a map without them is written as before. The new fixture
  `e09-towns-36.h3m` and its replays `towns-*` make the adventure replays 55 (the skill
  `homm3-rerecord-replays`' count and `adventure_replays.rs`' least): re-record them with yours at
  each bump.
  *Moved by E08 T103b: for information (the writer's two variants; the 55 replays, the skill's count).*
- 2026-11-03 (the town lane, E09 T3b, branch `town/e09-t3b`; the capture harness, informational
  unless you play battles with `attack`): `h3adv` gains the town reads (`town:x,y:owner`,
  `visitor`, `built`, `every`, `garrison`, `slot:N`; `towns:every_vs_built`,
  `towns:owned_garrisons`), `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
  `unknown_screens`, a `new_game = true` step, the town screen (vtable 0x64372C, closed by item
  30720) and the level-up dialog (0x63FE60, `level_up`) as known screens, and a list of creatures
  in the table battle plan's `attack`. Behaviour changes: `play_turns` reads the turn again after
  its 0.5 s wait (an order went into the next stack's turn), a retreat waits out the retreating
  side's Catapult, a `text:` literal ending in `: ` joins the next value without `; `, and `batch
  --map` fills the template beside the map when its stamp matches (no cargo build). A town's
  visitor (+0x10) reads −1 while its hero is the selected one. The harness check passed before and
  after. `clients/capture/README.md`, "Towns (E09 T3b)".
  *Moved by E08 T103b: for information (the capture harness).*
- 2026-11-03 (E08 T87b's audit; **for the next import bump**, increment 10's or the town lane's
  T7a): the parser reads class 220 below subclass 7 as a mine body (`parse/objects.rs`) while the
  class table types every 220 as the abandoned mine (`import/codes.rs`), so a crafted 220.0 is
  refused at the build (`SetupError::Holds`) where the original (220 loaded as 53, the subclass
  kept, `object_class_values` 0x63A6E4) and VCMI make a sawmill. No shipped map has one (all 94
  are 220.7). Type 220 by `MINES` like 53, test 220.0 as a sawmill, amend ADV-MIN-001 and
  ADV-OBJ-002 and `docs/formats/h3m.md`, with the bump that moves `MAP_IMPORT_VERSION` anyway.
  *Moved by E08 T103b: acted upon (class 220 by the mines' table with map import v21; ADV-MIN-001, ADV-OBJ-002, `h3m.md` amended).*
- 2026-11-02 (the town lane, E09 T1b2b, `TWN-Q2`; **nothing to build, for your import and
  setup**): E09's increment 1 builds **the random hero** (class 70, drawn as the main town's hero;
  E09 decision 8) and **the starting-hero pick per colour in the setup** beside the faction
  (decision 9), `cross-lane` in your import and setup, merged in order with yours; leave both to
  it (your 2026-10-20 note's 18 random heroes with a custom experience come with it). An allied
  town is entered and never captured (decision 16), as your owned objects' team test.
  *Moved by E08 T103b: settled (the random hero and the starting-hero pick were built by E09 T7a1 to T7b2b).*

## Inbox notes for information (moved 2026-11-09, E09 T17a, the town lane's own note)

- 2026-11-07 (the town lane, E09 T16a and T16b, `town/e09-t16a`, `town/e09-t16b`, `cross-lane`;
  **informational**): the 55 adventure replays' `town_ruleset_hash` line moved twice (T16a: town
  part v4, the buildings' requirements, costs and the role `shipyard`; T16b: the shipped packs'
  harvested ones), nothing else; the `BuildingDef` literal in `h3-game`'s `src/tests/mod.rs`
  gained `requires` and `cost` (T16a). Nothing to act on.

## Inbox notes acted upon (moved 2026-11-11, E08 T108b: the army reference agreed, decision 305; the town lane's inbox told)

- 2026-11-11 (the town lane, E09 T22a, `town/e09-t22a`; E09 decisions 74 to 78; **growth's
  `cross-lane` plan in your paths, for T108b's versions**): E09 increment 3's growth half is
  drafted (`docs/lanes/town/adr-drafts/*-increment-3.md`; design note `E09-design.md` §37 to §47;
  nothing built yet, the order of the bumps set at the town lane's T22b cut). In your paths it
  plans: `GameState` holding the week's kind beside `MonthKind` and the towns' stream (game state
  hash, the next after v27); `GameEvent::WeekStarted { week, kind }` and `GameObservation::week`;
  GAM-CAL-002 (the week's and the month's town steps in the day's order) and GAM-CAL-003 (after a
  week of a `week_of` building the month is that creature's without the creature's draw, so your
  month step spawns it: `new_month` 0x4C8F63); `GameSetup`'s `roe` flag (map import v22); two
  `h3-heroes` derivations (`town_growth`, `owner_town_growth`) over two new `HeroEffect` kinds
  (pack format v52, `shared/`: if your increment 12 takes v52 or adds effect kinds first, the
  second rebases); `ADV-DWL-003`'s test extended with a week of the hut's creature (your T97a
  note). A neutral town's garrison is written through the town lane's `src/town.rs`. Read it when
  T108b drafts its versions; T22b's cut orders the bumps with your increment 12's.

*(Acted upon by T108b: the design note's §17 takes the versions in merge order with E09 increment 3's; the pack format is not ours to bump in increment 12.)*

- 2026-11-11 (the town lane, E09 T21, `town/e09-t21`; E09 decision 73; **the army reference,
  for T108b, answering the town lane's note below**): the town's two rows read (survey
  `docs/rules/surveys/E09-growth.md` §9, §15): the town screen and your garrison dialog are one
  exchange (`garrison_dialog_create` 0x5CE830, the hover 0x5C7400): two bars, an owner each; only
  the current player's bar's stacks move; into another owner's bar only into an empty slot; a
  hero's last stack stays (in a town, the garrison hero's too, 0x5C7436); a stationary army
  without a hero may be emptied. **Proposed:** a town's upper bar is `ArmyRef::Garrison(town)`
  without a garrison hero and `ArmyRef::Hero(the garrison hero)` with one; a map garrison gets a
  sibling reference by its object (your call, with your store's record of its army, owner and
  removable flag); one owner rule checks both; the exchange's commands are the army dialog's own
  with the second bar named (`ArmyCommandError::Garrison` lifted for an own garrison). Answer in
  the town lane's inbox (or agree it in your T108b's drafts); the town lane's T22 drafts after the
  agreement. Whichever lane builds first, the other reuses it (E09 decision 67).

- 2026-11-11 (the town lane, E09 T20, `town/e09-t20`; E09 decisions 65 and 67; **for T108a's
  garrisons, before T108b drafts**): E09's increment 3 (growth, recruitment and the army rows) makes
  a town's garrison an army of your army commands (`ArmyCommandError::Garrison` lifted), with a
  garrison hero and the heroes' swap between the town's two rows; your increment 12's garrisons
  (33, 219) exchange armies with a hero of their owner (decision 295). Both are the original's army
  exchange with a stationary army, so build it once (as E09 decision 4's hire dialog): the town
  lane's survey T21 reads the town screen's two rows while your T108a reads the garrison's dialog;
  note in the town lane's inbox what T108a finds of the exchange (the dialog, its `ArmyRef` or a
  new one, a garrison's army in the store), and the town lane notes T21's in yours, so that one
  army reference is agreed before either draft (your T108b, the town lane's T22). Whichever lane
  builds it first, the other reuses it.

## Inbox notes acted upon (moved 2026-11-09, E08 T104)

T105's queue entry and design §15's table now give T105b game state hash v27.

- 2026-11-09 (the town lane, E09 T17a, `town/e09-t17a`, `cross-lane`; **for T105b, the
  versions**): T17a takes game state hash v26 (the store of towns' encoding v2: each town's day's
  mark, `h3_towns::Town::built_today`, after its forbidden set; your HANDOFF's versions line
  updated), so T105b's is v27 (your design §15's table and T105's queue entry still say v26). The
  55 adventure replays re-recorded from their own commands: only `game_state_hash_version`, the
  checkpoints and the final hashes moved, in every replay (an empty store's hash moves with its
  version too). Nothing else of yours changed.

## Inbox notes acted upon (moved 2026-11-10, E08 T105b2)

T105b2 took adventure replay format v12.

- 2026-11-09 (the town lane, E09 T18b1, `town/e09-t18b1`, `cross-lane`; **a version and a
  variant, for your bumps and matches**): **adventure replay format v11** is taken (`town build
  <town> <building>`, `GameCommand::Town(TownCommand)`), so T105b's bump is **v12**; the 55
  replays were re-recorded, only their `format` line moved. New in `h3-game`: the variant
  `GameCommand::Town`, `GameCommandError::Town(TownCommandError)`, `GameEvent::TownBuilt`
  (the owner's), the crate-private `GameState::towns_mut`; `advance_day` clears the towns' marks
  right before the income (`Towns::start_day`); `town::capture` takes the players' `human` flags.
  `legal_commands` lists no build before E09 T18b2, so no plan, env or bot report moved; the
  env's `tileless_entry` gives a build no entry until E09 T19. The words are in `h3-files`'
  `src/adventure/` since #808. Nothing to act on but the v12.

## Inbox notes settled or for information (moved 2026-11-11, E08 T107)

- 2026-11-10 (the town lane, E09 T19b, `town/e09-t19b`, `cross-lane`; **nothing to do, for your
  archive**): a gap of the hygiene property that T19b's draws found (`view.rs`' `outcome`):
  another player's hero declined a monster's offer to join, the monster fled in one game (its
  removal seen by the viewer) and asked again in the other; `outcome` compares the objects a
  command removed (`ObjectRemoved`'s kind, beside the guards' hints) now, the seed kept in
  `proptest-regressions/tests/view.txt`. (T19a's counterexample of the plans' gains: your #830's
  fix kept at T19b's merge of `master`.) T19b's town field moves the plans your
  `the_plans_jump_through_teleporters` draws: its 256 cases met no battle at a Whirlpool's exit,
  so it runs 512 now (met twice). *Moved by E08 T107: for information, as the note asks.*

- 2026-11-09 (the server lane, before E16 M5 T3c; **the user's decision**; informational): **the
  adventure's files moved from `h3-cli`'s `src/adventure/` into `h3-files`' `src/adventure/`**, yours
  (`lanes.toml`; ADR-0005's amendment): `load_rules`, `read_map`, `load_map`, the replay file
  format v8 (`replay_to_toml`, `parse_replay`), the commands' text form and `verify_file`, verbatim
  (`AdventureCliError` is `h3_files::adventure::AdventureFileError`). `h3-cli`'s `src/adventure/`
  keeps the runs, the scripts, the benchmarks and the commands and re-exports the rest; a replay
  file format change goes to `h3-files` now. *Moved by E08 T107: for information; the files' owner in `lanes.toml`.*

- 2026-11-09 (the town lane, E09 T18a, `town/e09-t18a`, `cross-lane`; **a kind's field and a
  version, for your matches and bumps**): `ObjectKind::Town` and `ObjectKindDef::Town` carry
  `boat_offsets: Vec<(i8, i8)>` (match `Town { .. }`; the base pack's `town = { boat_offsets =
  [[-1, 2], [1, 2]] }`, optional), **adventure ruleset part v19**; `fix_boat_tiles` fixes each
  town's spot in ADV-SHY-001's step (the town object's `Resolved::BoatTile`, in the static map's
  hash; `StaticMap::town_boat_spot(TownId)`; not in `ObjectView::boat_tile`), and the game forbids
  a town's Shipyard without one (TWN-SET-011). The 55 replays were re-recorded (only hash lines:
  the part's hash, the `towns-*` checkpoints); a pull request of yours that bumps the part again
  merges after it and re-records again. Nothing else moved. *Moved by E08 T107: settled: part v19 landed with its re-recorded replays; T105b2 merged after it.*

- 2026-11-07 (the town lane, E09 T13, `town/e09-t13`, docs only; **the planned `cross-lane` changes
  of E09 increment 2, construction**; design `docs/epics/E09-design.md` §34, §35, the drafts
  `docs/lanes/town/adr-drafts/*-increment-2.md`, accepted at `TWN-Q10`, in ADR-0016 and -0017 since
  E09 T15): in your paths, by task:
  T17 the game state hash (the store of towns' v2 folded in); T18a the kind `Town`'s
  `boat_offsets` in the adventure part (ruleset part bump) and each town's boat spot in
  `fix_boat_tiles` (as the town object's `Resolved::BoatTile`, the encoding unchanged); T18b
  `GameCommand::Town`, its error and events in `h3-game`'s `command.rs`, one call in `state/day.rs`
  at the day's start (the marks), the adventure replay format (the town command); T18c one call in
  `state.rs`' `start_group` after the map's timed events (the town events); T19 the fixtures, the
  golden replays re-recorded, `h3-map`'s plans (`Step::Build`). Your increment 11 plans part v18
  (T103a), game state hash v26, replay format v11 and action encoding v8 (T105b): whichever lane
  merges second bumps again and re-records (the town lane's `TRAPS.md`). Nothing to act on now. *Moved by E08 T107: for information; T17 to T19c1 landed, each bump merged in order.*

- 2026-10-31 (the town lane, E09 T0, `shared/town-lane`; **read before your next task**): the user
  opened a lane `town` (`TWN`) for E09 in parallel with E08 (TWN-Q1). It owns the crate `h3-towns`
  (to come) and the town modules inside your crates by the longest match: `h3-adventure`'s and
  `h3-game`'s `src/town*`, `h3-map`'s `src/import/town*` and `src/writer/town*`; E09's epic,
  surveys and experiments and `docs/rules/catalog/town.toml` moved to it. Your crates' common
  parts (`lib.rs`, the state structs and hashes, the versions, the replays and bot reports) stay
  yours; the town lane changes them in `cross-lane` pull requests merged in order with yours (the
  second rebases and bumps again): name your open pull requests in your HANDOFF as you do. E08's
  increments 10 to 12 stay yours (decision 4): the hire dialog and the weekly growth are built
  once, by whichever lane needs them first; tell the town lane's inbox when yours land. The
  E09 notes of your inbox and archive are gathered in the town queue's inbox; yours can leave
  them as they are. Increment 1 of E09 builds towns on the map (the import's main-town hero and
  `HeroInTown`, entering, capture by siege, the hall's income). *Moved by E08 T107: for information; the ownership in `lanes.toml` and the lanes' HANDOFFs.*

- 2026-10-30 (the client lane, E17 M5 T8b1, reviewer R7's fixes; **your harness changed**):
  after it merges, run `clients/capture/setup.sh` again for every run directory you use: the
  hook DLL's ack has ten fields now, and an older `xdd.dll` stops a run with "stale xdd.dll".
  `H3_RUN` without `WINEPREFIX` is refused (both scripts); the run directory is resolved
  absolute and `Game.start` fails unless the prefix's `d:` points at it. A run also fails when
  the ack's clip-clock field is not 1, when a capture reaches a result window with an empty
  `VIDEO.VID`, and in a sound run when a sample could not be parsed or more than 32 handles
  played. Sound runs: the battle's start sound must be a new sample still playing at the
  opening (else the run raises), loops counted (`sounds_settled` too); a paused sample answers
  Miles' 8; `wineserver -w` waits at most 60 s. Nothing else drives the original otherwise
  (runbook, "Sound", "The clips", "Traps"). *Moved by E08 T107: for information; the harness's runbook (`clients/capture/README.md`) holds it for batch E12.*

## Inbox notes acted upon (moved 2026-11-11, `adventure/srv-q26-reveal`: `h3_game::Sink::battle_ended`, the server lane's inbox told)

- 2026-11-10 (the server lane, E16 M5 T5a, SRV-Q27 (1); **for the whole game on the wire**):
  **a battle a command ended, readable until the next command** (`SRV-STUB-14`). The game
  session (`h3-protocol`'s `session/game.rs`) sends each owner its battle's `Snapshot` when the
  battle opens, but cannot send the last one when it ends, as the battle session does (v38,
  CLT-STUB-3: the own hero's Eagle Eye picks, which no event shows): the ending command closes the
  battle in `h3_game`, so `GameState::battle` has none. Keep the battles the last command ended
  (their `BattleState` and owners, as `OpenBattle`) readable until the next command, for example
  `GameState::ended_battles()`, out of the state hash and the replay; the session then observes
  each owner's side (`h3_battle::observe`) for its last `Snapshot`. Tell the server lane's inbox
  when it lands.

## Inbox notes taken into a task (moved 2026-11-12, E08 T113a: into T113b's scope, decision 320)

- 2026-11-12 (the server lane, E16 M5 T7b's review, branch `claude/keen-tesla-q5uxid`; **a
  question for the object's identity, ADR-0016 point 6**): an object is named to a viewer by
  its type and anchor (`ObjectRef`; `MonsterChanged`, `GuardsChanged`, `ObjectRemoved` and the
  others carry only those), and a view's objects are in tile order by type, anchor and masks
  (E16 M5 T5c). The server lane took it that two objects of one type share an anchor only as two
  special grounds alike in every field (E08 T20a2a), but found no check that a map cannot place
  two objects of one type on one anchor that differ (masks, guards, a monster's count): their
  events would then name both alike, and a client's fold (`GameObservation::fold`) changes the
  first. Either refuse such a map at import (or in the setup's checks), or tell us the reading
  that rules them out; no server change is needed either way.

## Inbox notes acted upon (moved 2026-11-13, E08 T114b: its views and events by type and anchor, the versions after the town lane's)

- 2026-11-13 (the server lane, E16 M5 T13; **the review of your v52 change to its paths**): the
  mapping both ways and the fold are sound. One point against `docs/06-security.md` ("a view
  names an object by its type and anchor, never by its index, which counts the objects before
  it, seen or not"): `ArmyRef::MapGarrison`, `Decision::Exchange` and `GameEvent::ExchangeOpened`
  name the garrison by its object index. Harmless today (objects are only appended, so a
  garrison's index is fixed at the setup and tells only the map's order), and the doc now names
  the exception. Proposal, for your next wire step (T114b's `GarrisonChanged` is one): name it on
  the wire by its `ObjectRef` alone (unique by type and anchor, your T113b answer), the server's
  `command_to_engine` resolving the index; or keep the index and say so in your decisions. No
  server change is needed either way unless you take the proposal.
- 2026-11-13 (the town lane, E09 T26a2, #899, `cross-lane`; **three versions taken after
  yours, for your T114b**): game state hash **v30** (the week's kind and `roe` after the month's
  kind, the towns' stream after the game's; after your T114a2's v29), map import **v23**
  (`GameSetup::roe`) and protocol **v53** (after your v52): T114b's bumps are the next at merge
  (v31, v24, v54), and whichever merges second re-records the 72 replays. In your files:
  `GameEvent::WeekStarted` gained `kind`, `GameObservation` `week`; `h3-map`'s `view.rs`
  `hygienic_town` counts a town visited at load as owned (the town inbox's 2026-11-11 note: the
  plan's fault, its `cc` line committed) and the hygiene property ends at a week's start whose
  kinds differ; `benches/adventure/fork-{economy,meeting,sea}-xl-144` and `fork-rewards-36`
  re-baselined (three new state fields, +0.8 % to +1.1 % against `master`).

## Inbox notes acted upon (moved 2026-11-14, E08 T116b1: the hygiene property's cut fixed, the line committed)

*(Moved verbatim: the property's fault, the movement's change told last after a visit that tells nothing; the record in `E08-log.md`, T116b1.)*

- 2026-11-13 (E08 T114b's scoped verify; **a failing case of the hygiene property, on `master`
  too**): `h3-map`'s `a_viewer_learns_nothing_it_cannot_see` failed with the regression line `cc
  f1ef9d365138e68fe1e92148a08536e583481e4b06db263fa364dc7e91824600` (a 7 × 7 two-level RoE plan;
  red's hero walks to the two-way monolith at (6, 0, 1), whose pair (4, 0, 0) the unseen part's
  teleporters change): one game tells the viewer the walk and `MovementChanged`, the other the walk
  alone, and the `entering && ta != tb` branch (`view.rs`, the teleporters' cut at
  `HeroTeleported`, `LeadsNowhere` or `MeetingOpened`) finds none of its events, so it compares
  the whole lists. Replayed on `master` 8380e4e in a worktree: the same failure, so not T114b's;
  the line is not committed (CI's random cases rarely meet it). Find what the second game told
  instead (a battle at the exit, a hero gone from the map?) and fix the property or the engine;
  then commit the line. T116b meets these properties anyway.

## T116b2 (moved 2026-11-14, done: `adventure/e08-t116b2`, decision 330)

*(Moved verbatim; the record in `E08-log.md`, T116b2.)*

**E08 T116b2 — the bonus visitables' properties** (design note §16; decision 329; then **T116c**)

- **Scope:** `h3-map`'s plans (`play.rs`) with the fifteen bonus visitables (`BONUSES`, appended
  after T116b1's `GARRISONS`: the twelve land kinds by template, near the heroes; the sea's three,
  the Buoy, the Mermaids and the Sirens, on the plan's strait as the Whirlpools), Springs emptied
  and Fountains drawn by the build; `check_bonuses` (a once-mark set once per battle; the amounts
  the visit's; the day's and the week's marks cleared at their steps; the points and the maximum
  raised together; the Sirens' survivors at least one, never more; the kind marks only grow); the
  old oracles met (the visits' events in `check_decisions`, `check_conservation`, the battle
  bonuses' clear in `check_heroes_after_battle`); the views follow (`check_told`'s bonuses and
  kind marks, T115b's, met by the plans' kinds; the hygiene property's own visits of a Fountain or
  a Spring whose drawn value or entrance the two games differ in); a cover test of fixed cases
  that visits every class (as T116b1's `the_plans_cross_fight_and_exchange_at_garrisons`, the
  oracles run inside it). `TRAPS.md`: the plans' strategy (raise the cover tests' cases rather
  than tune the draw). Confirm the cut at its start.
- **Inputs:** design §8 to §10, §16; T116a's and T116b1's records (`E08-log.md`; `bonuses_map.rs`
  shows the kinds' setups), `state/visitables.rs`, `state/bonuses.rs`, `play.rs`, `view.rs`.
- **Depends on:** T116b1 (its pull request). **Status:** ready once T116b1 merges.

## Inbox notes acted upon (moved 2026-11-14, E08 T116b2: the main property's landing check fixed, the line committed)

*(Moved verbatim: the property's gap, a battle lost within the move's command; reproduced on 29f658e; the record in `E08-log.md`, T116b2.)*

- 2026-11-14 (the server lane, CI on its pull request #922, E16 M5 T20; **a failing property case
  for you**): `h3-map`'s `tests::play::every_step_keeps_the_commands_invariants` failed with `cc
  f7f1ca8b35d61464506a3e97c784cd505d8b6f11a7190e3320a98b94a06ba606` (an AB map of 11 × 11, one
  level): after an accepted move the hero is gone (`state.adventure().hero(hero)` is `None`) where
  the walk's landing says `Some(Pos { x: 5, y: 3, level: 0 })` (`play.rs`, the move's position
  check after `landing` and `jumped_to`). It reproduces locally on `master` 29f658e and on 2996b51,
  before the town lane's #920, so it is older than T28a; #922 touches none of the engine's crates.
  To replay it: append the line to `crates/h3-map/proptest-regressions/tests/play.txt` (your path,
  not added by the server lane), then `PROPTEST_CASES=1 cargo test -p h3-map --lib
  tests::play::every_step_keeps_the_commands_invariants`.

## Inbox notes settled or for information (moved 2026-11-15, E08 T117)

*(Moved verbatim at increment 12's closure: the other lanes' notes of changes in this lane's paths, built on by T116a to T116c, their properties and T117's audit; the town lane's plan for T114a, T115a and the order, and CLT-Q40's visits by once-mark, done by T114 and T115a; the server lane's review of the garrison's index, kept as built (T117's agent's call, `E08-log.md`); the harness notes and the server lane's informational decision.)*

- 2026-11-15 (the town lane, E09 T29a, `town/e09-t29a`, `cross-lane` with the user's consent, E09
  decision 96; **your paths changed, for your review; no version moved**): the builds map
  `fixtures/maps/e09-builds-36.h3m` (`h3-map`'s `builds_map.rs`) gains red's Dungeon "Pit" (town
  4), the neutral Rampart "Grove" (town 5), red's Valeska (hero 2) and red's Guardhouse, every
  existing id kept; the six `builds-*` replays re-recorded from their commands (hash lines only)
  and three new scripted ones (`builds-month-s1`, `-rows-s1`, `-siege-s1`; their notes in
  `h3-cli`'s `tests/adventure_replays.rs`); `day-start-builds-36` re-baselined at 858 034 (+51.7 %
  on `master`, no code changed: the map's two towns). `h3-map`'s `original_data` gains the census
  of growth to day 29 (`played_growth`; the whole check 17 s in debug). Nothing to act on; a new map
  object near the Keep or the Pit's paths moves those replays.

- 2026-11-14 (the town lane, E09 T28b2, `town/e09-t28b2`, `cross-lane` with the user's consent, E09
  decision 95; **your paths changed, for your review; protocol v67, no other version**): a
  garrison hero defends its town (TWN-SIE-001's rows 1 and 3; GAM-BAT-001, -003 amended).
  `state.rs`' `end_battle` hands a hero not on the map to the town lane's `end_town_hero_battle`
  (a garrison hero's survivors, an empty army kept for a winner, `TWN-QUIRK-11`; else it leaves
  the game, `HeroRemoved` to its owner), `raise` finds its hero through `hero_or_held_mut` (a
  garrison hero's Necromancy), `state/bonuses.rs`' `change_marks` admits a garrison hero (its
  bonuses cleared after its battle). `h3-game`'s `state/battle.rs` `end_battle` keeps a garrison
  winner's survivors when none is left and tells its town's changed views after the heroes' part;
  `state/manage.rs`' `town_told` tells the town of a garrison hero whose army a command names (a
  town's garrison view is now its upper bar, `AdventureState::upper_bar`). `h3-map`'s `view.rs`:
  `check_town_views` expects the upper bar and the mark (`TownOwnerView::garrison_hero`), the
  `TownChanged` fold reads it. The battle beside a garrison hero is a `BattleStart` without a town.

- 2026-11-14 (the town lane, E09 T28b1, `town/e09-t28b1`, `cross-lane` with the user's consent, E09
  decision 94; **your paths changed, for your review; no version moved**): `state.rs`' `start_turn`
  ends with the town lane's `start_town_heroes` (a garrison hero's points after the heroes on the
  map, TWN-ROW-005, ADV-MOV-002 amended); `state/bonuses.rs`' `drop_timed_marks` changes the marks'
  block by hero id directly (one `make_mut`, the entries left empty dropped), since
  `change_marks` admits only heroes on the map and the garrison heroes' marks did not drop;
  `state/economy.rs`' `income` reads the owned towns' upper bars on a week's first day
  (`AdventureState::upper_bars`, the town lane's `src/town.rs`; GAM-RES-005 amended);
  `h3-game`'s `state/day.rs` `pay_income` pays the garrison heroes' `daily_income` (GAM-RES-004
  amended). `h3-map`'s plans: `check_income` over them, `check_isolation` over another player's
  garrison hero; `view.rs`' checks of `MovementChanged` and the marks' events read `own_view`
  (the hero on the map, else its garrison view: a garrison hero's points and marks change now; CI's
  case `cc 2c6c6565…` added to `proptest-regressions/tests/view.txt`). Nothing else of yours
  changes; a hero leaving the map still leaves its marks.

- 2026-11-14 (the town lane, E09 T28a, `town/e09-t28a`, `cross-lane` with the user's consent, E09
  decision 93; **versions taken and your paths changed, for your review**): adventure state hash
  **v25** (after your #914's v24: the towns' garrison heroes after the heroes on the map,
  `hash_hero` shared), adventure replay format **v16** (`town heroes <town> enter|leave|swap`),
  every adventure replay re-recorded (versions and hash lines only). `h3-adventure`: a state block
  `town_heroes: Option<Arc<TownHeroes>>` (by town; its rules in the town lane's `src/town/rows.rs`:
  `town_hero`, `garrisoned`, `move_town_hero`, `check_town_hero`, `release_town_hero`;
  `town_heroes()` and `rows_mut()` in `state.rs`, `viewers` `pub(crate)`); `state/manage.rs`'
  `held` answers `ArmyRef::Garrison` for an own town without a garrison hero (`Held::Town`) and a
  garrison hero after the heroes on the map (`Held::TownHero`), both `#[cold]`, `set_formation`
  and `set_tactics` through `own_record_mut`; `dwellings.rs`' `Joining::TownHero` (a town's
  purchase joins its garrison hero's army); the events `GarrisonEntered`, `GarrisonLeft`;
  `AdventureObservation::garrison_heroes` (`GarrisonHeroView`, `own_view` shared); `OwnHeroes`
  over the garrison heroes too. A garrison hero's marks stay in your T115a block by hero id (no
  `leave_marks` at `Enter`; `release_town_hero` calls it). `h3-game`: `state/manage.rs`'
  `admits_army` (a town's two bars outside a decision) and `town_told`, `stack_at` and
  `own_details` find a garrison hero, `legal.rs`' `town_partner` and `legal_town_management`,
  `observe.rs`' hero infos with the owner's garrison heroes. `h3-map`'s plans: `town_builds`
  probes the three moves after each town's recruitments, `check_town_heroes` reads their effects,
  `check_invariants` the garrison heroes (none on the map, the town's garrison empty);
  `view.rs`' `check_told` folds the two events. `h3-files`: the `town heroes` line.
  `manage-army-12` +0.55 % against `master` (inside its gate).

- 2026-11-13 (the town lane, E09 T27, `town/e09-t27`, `cross-lane` with the user's consent, E09
  decision 92; **versions taken and your paths changed, for your review**): game state hash
  **v32** (after your T115a's v31: a hire dialog's town target and sources, the target's tag 1,
  the sources' tags 1 and 2), adventure replay format **v15** (`town recruit <town> <building>`,
  after the server's v14), every adventure replay re-recorded (versions and hash lines only);
  `h3-cli`'s adventure tests' format literals. `h3-adventure`: `HireTarget::Garrison(TownId)`,
  `HireSource::Town { town, index }`, `HireSource::Portal { town }`; `hire_stock` gives none for a
  town's source, `hire_room` reads a garrison; `check_join` (a target's room and the price, for a
  town's purchase) and `check_hire` built on it; `HirePurchase` names its army and the map's count
  to lower; `hire` returns the garrison's changed `ArmySlot`. `h3-game`: `state/hire.rs`'s `hire`
  hands a town's source to the town lane's `town::recruit::buy`, `GameState::hire_stock` public
  (any source), `open_hire` `pub(crate)`; `HireSeen` in `observe.rs` replaces `HireView`'s and
  `HireOpened`'s `object_type` and `anchor`; the hash above. `h3-map`'s plans: `hire_stock`,
  `hire_most` and `check_hire` read a town's source and garrison, `town_builds` probes each
  town's recruitments after its builds (`recruited` reads what a building sells apart),
  `check_towns` lets a purchase lower one count or the Portal's; `view.rs`' open dialogs by
  `HireSeen`; `check_legal` lets a move's battle lost within the command remove the mover (CI's
  case on #911, a defender's war machine firing first: a gap of your plans the changed games
  reached; its `cc` line kept). `h3-files`: the `town recruit` line.

- 2026-11-13 (the server lane, E16 M5 T14, `claude/quirky-mccarthy-pgu2o6`, `cross-lane` with
  the user's consent, SRV-Q29, SRV-Q31; **your paths changed, for your review**): `h3-game`
  gains the system actor's own commands, `GameCommand::System(SystemCommand)` (`system.rs`:
  `EndTurn { player }`, `Answer { player }` by `default_answer`, `Battle { player, command }`
  for a timed-out player's side; GAM-SYS-001 to GAM-SYS-003 in `game.toml`), applied as the
  player's own commands; the hold judges them as their player's (`command.rs`' `held_as`: the
  level-up's owner's default answer passes it); `GameCommandError::SystemOnly` for a player's;
  `GameState::check_player`. Not in `legal_commands`. ADR-0017 point 2 amended (its `shared:`
  commit). **Your version: the adventure replay format v14** (`system for <player>
  end-turn|answer default|battle <command>` in `h3-files`' `src/adventure/`): the 72 golden
  replays' `format` line alone moved, no hash; `h3-cli`'s and `h3-sim`'s version tests follow;
  your HANDOFF's and `OVERVIEW.md`'s version lines say v14. A new match on `GameCommand` needs
  the `System` arm (`h3-map`'s `as_move` has it).

- 2026-11-13 (the town lane, E09 T26b, `town/e09-t26b`, `cross-lane`; **in your files, for your
  T114b**): `AdventureOutcome::flagged_dwelling` (`command.rs`; set by `enter_dwelling` when the
  visit flags: the game then draws the player's empty Portals of Summoning, TWN-GRO-007, in
  `settle_outcome` right after a capture); `GameState::town_draws` (`state.rs`: the adventure, the
  store and a town draw's stream), the setup's Portals after the levels at load in
  `GameState::new`, the week's neutral garrisons and Portals after your `start_week` in
  `state/day.rs`, the week's views taken there and told at the towns' day. In `h3-map`'s plans:
  `check_towns` (the Portal's movers, the owner told), the view model's counts and Portal, and the
  hygiene property now ends at **every** week's start (the towns' stream draws the neutral
  garrisons and the Portals after the week's kind). Four replays on the towns map re-recorded
  (hash lines only), `day-start-builds-36` and `day-start-towns-36` re-baselined after your
  T114b's merge. No version of yours moved; protocol v55 is the town lane's (after your v54).
  Informational.

- 2026-11-13 (the server lane, E16 M5 T13; **the review of your v52 change to its paths**): the
  mapping both ways and the fold are sound. One point against `docs/06-security.md` ("a view
  names an object by its type and anchor, never by its index, which counts the objects before
  it, seen or not"): `ArmyRef::MapGarrison`, `Decision::Exchange` and `GameEvent::ExchangeOpened`
  name the garrison by its object index. Harmless today (objects are only appended, so a
  garrison's index is fixed at the setup and tells only the map's order), and the doc now names
  the exception. Proposal, for your next wire step (T114b's `GarrisonChanged` is one): name it on
  the wire by its `ObjectRef` alone (unique by type and anchor, your T113b answer), the server's
  `command_to_engine` resolving the index; or keep the index and say so in your decisions. No
  server change is needed either way unless you take the proposal.

- 2026-11-13 (the client lane, E27 M1 T2, `client/e27-m1-t2`; the capture harness, **rerun
  `setup.sh` in a fresh sandbox as before**): `xdd.dll`'s ack has 13 fields (a stale `xdd.dll` in a
  run directory fails every run: rerun `setup.sh`); the run's `VIDEO.VID` joins `battle-videos.vid`
  with `shell-videos.vid` when both are staged (the start's clips play only with `Show Intro` on,
  which the harness's settings keep off); `SMACKW32.DLL` reads the virtual clock too and
  `BinkGetSummary` is guarded (a full-screen clip's end on a frozen clock). `h3drive.Game` takes
  `frozen` (`H3FROZEN`). New: `h3shell.py`, the shell's screens (the runbook's "The shell's
  screens"). The harness check passed.

- 2026-11-12 (the town lane, E09 T25a, `town/e09-t25a`, `cross-lane`; **a version and your
  replays, for your next bump**): `GAME_STATE_HASH_VERSION` is 28 (the store of towns' v3: each
  town's fourteen available counts and its Portal, `h3_towns::Town::available`, `portal`); the 72
  adventure replays re-recorded from their own commands (only `town_ruleset_hash`, the version,
  the checkpoints and `final_hash` moved). A game state hash bump of yours is v29 (the town lane's
  T26a then takes the next); whichever merges second rebases and re-records again. Your
  `src/tests/mod.rs` town literal gained `roe` and the growth's parameters (`week`,
  `garrison_week`). `h3_towns::Towns::build` and `town_setup` now take the core ruleset (the
  dwellings' first creatures, TWN-GRO-005); their callers are the town lane's `src/town*`.

- 2026-11-12 (the server lane, E16 M5 part 2's refinement, SRV-Q29; **the user's decision,
  informational**): the server lane builds two additive pieces of `h3-game` in `cross-lane` pull
  requests, with the user's consent: in increment 2b the trusted system commands ADR-0017 point 2
  names (a player's turn ended; a player's open decision answered with `h3_game::default_answer`,
  the level-up past its hold included), in 2c `Viewer::Spectator` (public facts only) through
  `h3-game`'s and `h3-adventure`'s projections. Each pull request names its rule ids and tests and
  asks your lane's review; nothing for you to build. 2a (whole games over the network) changes
  none of your paths.

- 2026-11-11 (the town lane, E09 T23, branch `town/e09-t23`, `cross-lane`; **the capture harness
  changed**; `clients/capture/README.md`, "Forced rolls" and "Growth, the garrison hero and a
  town's hire"): xdd.dll's table gains twelve adventure-map sites (four forced: `week_chance`,
  `week_creature`, `month_kind`, `month_creature`; eight logged: the neutral garrisons' and the
  Portal's draws), so `h3drive.ROLLS` and `LOGGED` hold 30 calls: a battle sequence checks that
  xdd.dll redirected all 30, and a stale `xdd.dll` (one built before) fails that check: rebuild it
  with `clients/capture/setup.sh`. `h3adv` gains a recipe's and a step's `rolls`, `roll` and
  `date`, the reads `draws`, `turn_draws`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero`, `hire:creatures`, the town screen actions `hero_in` and
  `building`; `h3mem`'s town record gains `recruits`, `portal`, `dwellings`. Nothing else moved;
  the harness check passed after it (its first run stopped at the known `attack` flake).

- 2026-11-11 (the town lane, E09 T22b, `town/e09-t22b`; E09 decisions 80 to 83, `TWN-Q14`; **the
  town's rows and the garrison hero in your paths, for T114a, T115a and the order**; the town
  epic's design note `docs/epics/E09-design.md` §52, §53, §56): (1) **the town's two rows open no
  decision**: the game's management check admits, with no decision open, an own town's upper bar
  (`ArmyRef::Garrison(town)`, or `ArmyRef::Hero(the garrison hero)`) alone or with its visitor, by
  your one owner rule; the town lane wires a town's garrison's bar owner (its town's owner) into the
  rule's lookup in its T28a (`cross-lane`), so T114a needs only the hook. (2) **The garrison hero's
  record moves whole** from `AdventureState`'s heroes on the map into the town's `Holds::Town`
  (army, points, formation, tactics, and your T115a's bonuses and timed marks kept): `ArmyRef::Hero`
  resolves to it after the heroes on the map; your day's and week's drops of the marks and the
  turn's points (ADV-MOV-002) must reach it too (the original's loops cover every hero record and
  the towns' +0xC heroes): T28b adds those loops (`cross-lane`); if T115a writes the drops as a
  per-record function, T28b only calls it for the garrison heroes. (3) **The plan in your paths**
  (`cross-lane`, each number the next at merge): T27 (`HireSource::Town`, `::Portal`,
  `HireTarget::Garrison` in `command.rs`; the game state hash; the replay format, `town recruit`),
  T28a (`AdventureCommand::TownHeroes`; the adventure state hash; the replay format; management
  lines naming a town's bar), T26a (the game state hash, map import `roe`); T28 waits for your
  T114a and T115a merged.

- 2026-11-03 (the client lane's CLT-Q40, carried by the battle lane, BAT-Q64; `cross-lane`): the
  morale and luck sources ride on the effects (`h3_contracts::EffectSource`). `derive_battle_hero`
  now names the skill and level, the artifact and the specialty (HER-BAT-003) and gives the battle
  bonuses as the temporary source (HER-ADV-008), so battles from the map carry them already. Left
  for you: the visits by once-mark (`EffectSource::Visit { mark }`, the original's +0x105 lines)
  once the derivation knows each mark's amounts (your content), the rest staying temporary; the
  owner's Grail building is the town lane's (its inbox).

## T118a (moved 2026-11-15, done: `adventure/e08-t118a`, decisions 335 to 339)

*(Moved verbatim; the record in `E08-log.md`, T118a.)*

**E08 T118a — the creature banks survey** (decision 334; as T108a for increment 12: survey file
`docs/rules/surveys/E08-banks.md`)

- **Scope:** the census of classes 16 (by subclass), 24, 25, 84 and 85 on the 160 shipped maps
  (T97c's counts in `E08-log.md`, T97c: 2 137 objects on 148 maps); their `.h3m` bodies and
  templates (`docs/formats/h3m.md`, VCMI 1.7.5's reader); the banks' configurations in the
  original data (the text tables the harvest reads: each bank's levels, their chances, guards and
  rewards); the original's code (`docs/exe/README.md`, `scripts/exe.py`): the draw of a bank's
  level, guards and reward at a new game (and at a week's or a month's start, if any), the visit
  (the question, the battle against the guards, the reward: resources, artifacts, creatures
  through the army dialog 0x4ABDC0), the bank emptied, a revisit and a lost battle; the bank
  battle (the battle's bank flag +0x53C5: no war machines, no tactics phase, the guards' own
  layout; the battle lane's: its queue's "War machines" note); what a viewer sees (a bank's
  visited state, its guards' size word); VCMI's and the wiki's rules; disputes `D-<n>` from 65,
  reading quirk candidates `A-<n>` from 46; the proposed rule ids; the batch's plan (E13) on the
  client lane's harness (decision 334's agent's call); what the bank battle needs, told to the
  battle lane's inbox. Likely in parts (the content and the draw; the visit, the battle and the
  reward), confirmed with the user at its start, the reading measured first.
- **Inputs:** decisions 333, 334; ADR-0015 to -0017 (ADR-0016 points 3, 4 and 6, ADR-0017's
  decisions and rests), ADR-0013 (the hero's derived properties); increment 3's monsters and
  battles on the map (the neutral seat), increment 7's rewards (the shared reward record, the
  artifacts), increment 10's hire dialog (creatures joining an army); `E08-archive.md` decision
  127 (T55a3's measure: the five banks about 3 000 instructions and the battle's bank flag, which
  `BattleSetup` lacks); the battle lane's queue ("War machines"); the lane's `TRAPS.md`.
- **Depends on:** T117 (done). **Status:** ready (the original data and the shipped maps:
  staging per `docs/08-work-process.md`, "Cloud sandbox").

## T118b (moved 2026-11-15, its map side done as T118b1: `adventure/e08-t118b1`, decision 340; the rest queued as T118b2)

*(Moved verbatim; the record in `E08-log.md`, T118b1.)*

**E08 T118b — the ADR amendment drafts, the design note and the cut** (decision 334)

- **Scope:** as T108b for increment 12: the drafts `adr-drafts/0016-increment-13.md` and
  `0017-increment-13.md` (and ADR-0013's or ADR-0014's if the survey finds a change of the hero's
  derived properties or of the battle's setup), increment 13's design note `E08-design.md` (its
  parts by side), the building tasks' cut and the exit criteria; the checkpoint with the user (the
  note, the drafts, the agent's calls, the cut).
- **Inputs:** the survey `docs/rules/surveys/E08-banks.md` (§9's open points, §10 the interfaces'
  sketch, §11 batch E13, §12 the proposed ids); decisions 333 to 339 (D-65, D-66, D-68 as the
  original; **D-67 as the original: a won bank's reward before the winner's levels**, an exception
  to GAM-DEC-007 point 3 to design; the bank battle the adventure lane's, `cross-lane`, after a
  `shared/` change of `BattleSetup`; batch E13 before the build, its spec and recipe in the cut);
  ADR-0015 to -0017, ADR-0013 (the three once-marks); the battle lane's inbox note of 2026-11-15
  (the bank battle's needs) and any answer; T108b's drafts as the form.
- **Depends on:** T118a (done). **Status:** ready (docs only).

## T124 (moved 2026-11-17, its battle half done as T124a: `adventure/e08-t124a`, decision 353; the rest queued as T124b)

*(Moved verbatim; the record in `E08-log.md`, T124a.)*

**E08 T124 — the bank battle; the state, the draw, the visit, the battle and the reward** (round 6;
design `E08-design.md` §4 to §8, §12 to §15, §18, §20)

- **Scope:** T124a (`adventure/e08-t124a`, `cross-lane`, first, the battle lane reviewing): the
  bank battle in `h3-battle` with `BattleSetup::creature_bank` (§13: no tactics phase, no
  obstacles and a setup refusing them or a siege beside the flag, no war machines on the field and
  the Ammo Cart's supply kept, the layout by `bank_positions`; BAT-SET-001, BAT-TAC-001,
  BAT-GEO-004, BAT-WAR-001, BAT-WAR-004 amended), the battle pack's `bank_positions` (pack format
  v53, shared: likely a `shared/` part first), battle state hash v56, battle part v8, the battle's
  properties, golden replays and bench. T124b (`adventure/e08-t124b`): the state and the draw (§4,
  §5; adventure state hash v26, ADV-BNK-002), the visit and its offers (§6, ADV-BNK-003, D-66),
  the battle's two halves (§7, §14: `Defender::Bank`, the flag, the deck; ADV-BNK-004,
  GAM-BAT-001), the reward as a won battle's rest (§8, §15; ADV-BNK-005, GAM-DEC-007), the
  once-marks (§12, HER-ADV-008). Each part's cut confirmed at its start; T124a merged before
  T124b's tests.
- **Inputs:** design §4 to §8, §12 to §15, §18's versions (the battle lane's E26 and E11 may have
  taken the battle's numbers: bump past theirs); survey `E08-banks.md` §3 to §6, §14 (batch E13's
  rows); ADR-0022 point 8, ADR-0016 and -0017's increment 13 parts; the battle lane's `TRAPS.md`
  and HANDOFF (its open pull requests) before touching `h3-battle`.
- **Depends on:** T123b (`adventure/e08-t123b`). **Status:** ready once T123b merges; T124a needs
  the battle lane's state checked first (its queue's "War machines", r3).

## T124b (moved 2026-11-17, cut in two by decision 354: its state and draw done as T124b1, `adventure/e08-t124b1`; the rest queued as T124b2)

**E08 T124b — the state, the draw, the visit, the battle and the reward** (round 6; design
`E08-design.md` §4 to §8, §12, §14, §15, §18, §20)

- **Scope:** (`adventure/e08-t124b`): the state and the draw (§4, §5; adventure state hash v26,
  ADV-BNK-002), the visit and its offers (§6, ADV-BNK-003, D-66), the battle's two halves (§7,
  §14: `Defender::Bank`, the flag set in the battle's setup, the deck; ADV-BNK-004, GAM-BAT-001),
  the reward as a won battle's rest (§8, §15; ADV-BNK-005, GAM-DEC-007), the once-marks (§12,
  HER-ADV-008). Likely large: its cut confirmed at its start (a seam: the state, the draw and the
  visit first; the battle, the reward and the once-marks next).
- **Inputs:** design §4 to §8, §12, §14, §15, §18's versions; §13's *As built* (T124a: the bank
  battle as the battle crate runs it, `BattleParams::bank_position`, `SetupError::
  InvalidCreatureBank` for a bank setup with obstacles or a siege, `generate_setup_obstacles`
  giving none); survey `E08-banks.md` §3 to §6, §14 (batch E13's rows); ADR-0016 and -0017's
  increment 13 parts; the lane's `TRAPS.md`.
- **Depends on:** T124a (`adventure/e08-t124a`, merged first). **Status:** ready once T124a
  merges.

## T124b2 (moved 2026-11-17, done whole: `adventure/e08-t124b2`, decision 355)

**E08 T124b2 — the visit, the battle, the reward and the once-marks** (round 6; design
`E08-design.md` §6 to §8, §12, §14, §15, §20; decision 354)

- **Scope:** (`adventure/e08-t124b2`): the visit and its offers (§6, ADV-BNK-003, D-66: an attack
  kind's team marks at once, then its question unless looted; a search kind's question first,
  looted or not, the marks after a yes; `OfferKind::Fight` and `Search` routed by the kind;
  `Revisit` on an entered bank), the battle's two halves (§7, §14: `Defender::Bank` with its arms
  in `h3-game`'s two matches and `Defender::owner`, `start_battle` setting `creature_bank`, the
  deck for a `deck` kind from the coast and from a boat; ADV-BNK-004, GAM-BAT-001), the end
  (`end_unowned_battle`'s bank case: lost or retreated, the survivors stay; a draw empties them,
  not looted), the reward as a won battle's rest after the winner's levels (§8, §15: `give_won`'s
  bank case, the creature joined or the army dialog's group, the artifacts in the record's order,
  the resources, then looted; a yes without guards standing gives it at once; ADV-BNK-005,
  GAM-DEC-007), the looted search's once-marks 14 to 16 (§12, HER-ADV-008). No version planned
  to move (game state hash v32, replay format v16, encoding v8, protocol v69 stay, §18); the
  step onto a bank no longer refused (`h3-map`'s `a_step_onto_a_bank_is_refused_until_t124b2`
  replaced). The hint and its events are T125a's.
- **Inputs:** design §6 to §8, §12, §14, §15; §5's *As built* (T124b1: `Resolved::Bank`'s level
  and artifacts, the guards as drawn in `MapObject::guards`, asking); §13's *As built* (T124a:
  `BattleSetup::creature_bank`, `SetupError::InvalidCreatureBank`); survey `E08-banks.md` §4 to
  §6, §14; ADR-0016 and -0017's increment 13 parts; the lane's `TRAPS.md` (the plans' oracles, a
  command's exact events). The templates in code: the Pyramid's search (`records.rs`'
  `search_pyramid`, `loot_pyramid`), the guarded piles' fight and reward (`piles.rs`), a mine's
  `Fight` routing (`mines.rs`).
- **Depends on:** T124b1 (`adventure/e08-t124b1`, merged first). **Status:** ready once T124b1
  merges; its cut confirmed at its start.

## T125a (moved 2026-11-17, done whole: `adventure/e08-t125a`, decision 356)

**E08 T125a — the hint and its events, the Shipwreck's passage** (round 7; design
`E08-design.md` §9, §10, §16, §20)

- **Scope:** (`adventure/e08-t125a`): what a viewer sees of a bank (§10, ADV-BNK-006, ADV-VIS-006
  and ADV-OWN-003 amended: its type to every viewer; by its team's mark `ObjectView::visited`, and
  `ObjectView::guards` as a `GuardsView` without the army, the creature when every stack is of one
  and the count's word, none when looted or none stand; never the level, the counts, the upgrade,
  the artifacts or the reward); the events (§16): `ObjectMarked` to each player whose mark is set
  anew (T124b2 sets the marks silently: an attack kind's at the visit, a search kind's after the
  yes, `state/banks.rs`), then `GuardsChanged` with the hint to the same players, and after a
  battle's end and the loot to the teams that visited, when the word or the creature changed or it
  was looted; `check_told` and `reveal` see them. The Shipwreck's passage (§9; ADV-STUB-12 lifted,
  ADV-MOV-009, ADV-SEA-002 amended): its water entrance a step from a stop node on land (walking,
  water walking, flying: the visit from the neighbour, the hero on land) and a flight's pass from
  land or water, never a water walk's; `passage`'s water branch, `check_tile`, the reach; the
  Derelict Ship stays D-46's. Measured right after the first build against `master`
  (`reach-xl-144`, `reach-sea-xl-144`, `reach-lifted-xl-144`, the random-play workloads;
  `homm3-bench-regression`). Versions: none planned (the fold maps `ObjectMarked` and
  `GuardsChanged` for any object, §16; protocol v70 stays; check the observation's hash if the
  view's fields move it).
- **Inputs:** design §9, §10, §16; T124b2's *As built* in §6 to §8 and §14 (the marks silent, a
  visited bank in `known` as `Known::Visited`); survey `E08-banks.md` §7, §14 (E13-2, E13-9,
  E13-12); the lane's `TRAPS.md` (the reach's codegen partition and the passage's loops, the
  hygiene property's ends, a command's exact events). The templates in code: a mine's
  `GuardsView` and `GuardsChanged` (`mines.rs`, `observe.rs`), a weekly object's `visited` and
  `ObjectMarked` (`weekly.rs`), the empty boat's tile in `passage`.
- **Depends on:** T124b2 (`adventure/e08-t124b2`, merged first). **Status:** ready once T124b2
  merges; its cut confirmed at its start. T125b (the two maps, the replays, the properties, the
  benchmarks, `--with-original-data`) after it.

## T125b (moved 2026-11-18, cut in two by decision 357: its maps and replays done as T125b1, `adventure/e08-t125b1`; the rest queued as T125b2)

**E08 T125b — the two maps, the replays, the properties, the benchmarks** (round 7; design
`E08-design.md` §17, §19, §20)

- **Scope:** (`adventure/e08-t125b`): the two maps written by `h3_map::writer` (§17):
  `e08-banks-36.h3m` (the eleven kinds beside two players' starts, a coast with a Shipwreck beside
  land, a Derelict Ship off it and a boat, a hero of each player with a full army, one near a
  level) and `e08-banks-144.h3m` (144 × 144 with a sea, the census's most banks on one map, every
  kind, many Shipwrecks beside land, a monster on every hundredth free land tile); the golden
  replays on the 36 map (three random-play seeds per `adventure_replays.rs` and the scripted ones
  of §17: the declined attack and search with both players' hints, a won Griffin Conservatory past
  a level with the army dialog, a lost battle and the revisit, the looted Crypts and Shipwreck's
  once-marks cleared by a battle, the Shipwreck from the coast and the Derelict Ship from the boat,
  a flight across the Shipwreck); the properties (`h3-map`'s `play.rs`: `BANKS` in the plans,
  `Step::Bank`, `check_banks`, `check_told` for the mark's `GuardsChanged`, the hygiene property's
  bank with its hidden draw ending at the viewer's team's mark or battle; fixed cases visiting
  every kind); the benchmarks `reach-banks-xl-144`, `build-banks-xl-144`, `random-play-banks-36`
  baselined (`homm3-bench-regression`); `--with-original-data` (the census, the 160 shipped maps
  built with their banks drawn, the seventh pack check, the bank positions). Versions: none
  planned (adventure replay format v16, env v8 stay).
- **Inputs:** design §17, §19 (batch E13's rows for the scripted replays' expectations), §20's
  exit criteria 1 to 9; T125a's *As built* in §9, §10, §16 (`coast_visit`,
  `StaticMap::coast_entrances`, `bank_view`, `mark_bank`, `tell_hint`); the templates: increment
  12's maps and replays (`e08-garrisons-36`, `e08-bonuses-36`), `play.rs`' `check_garrisons` and
  `check_bonuses`; the lane's `TRAPS.md` (the plans' oracles, the hygiene property, a command's
  exact events); skills `homm3-rerecord-replays`, `homm3-bench-regression`.
- **Depends on:** T125a (`adventure/e08-t125a`, merged first). **Status:** ready once T125a
  merges; its cut confirmed at its start (likely in two: the maps and replays, then the
  properties and benchmarks). T126 (the audit) after it.

## T125b2b (moved 2026-11-18, done whole: `adventure/e08-t125b2b`, decision 359)

**E08 T125b2b — the benchmarks, `--with-original-data`** (round 7; design `E08-design.md` §17,
§20; decisions 357, 358)

- **Scope:** (`adventure/e08-t125b2b`): the benchmarks `reach-banks-xl-144` (walkers and flyers on
  T125b1's `e08-banks-144.h3m`: the coast's Shipwrecks met and crossed), `build-banks-xl-144` (the
  game's build with its draws, as `build-gates-xl-144`) and `random-play-banks-36` (T125b1's
  `e08-banks-36.h3m`) baselined (`homm3-bench-regression`; their README rows in a `shared/` pull
  request, as T116c's); `--with-original-data` (the census, the 160 shipped maps built with their
  banks drawn, the seventh pack check, the bank positions). Versions: none planned.
- **Inputs:** design §17, §20's exit criteria 1, 2, 5, 7 and 9; T125b1's record in `E08-log.md`
  (the maps' layouts); the templates `benches/adventure/`'s `reach-garrisons-xl-144`,
  `build-gates-xl-144` and `random-play-bonuses-36` (T116c's record); the lane's `TRAPS.md` (a
  workload's `collect` pattern, a new workload's first count); skill `homm3-bench-regression`.
- **Depends on:** T125b2a (`adventure/e08-t125b2a`, merged first). **Status:** ready once T125b2a
  merges; its cut confirmed at its start (likely whole, as T116c). T126 (the audit) after it.

## T127a (moved 2026-11-18, cut in two by decision 363: the census and the market done as T127a1, `adventure/e08-t127a1`; the rest queued as T127a2)

**E08 T127a — the trades and services survey** (decision 362; as T118a for increment 13: survey
file `docs/rules/surveys/E08-trades.md`)

- **Scope:** the census of classes 2 (the Altar of Sacrifice), 7 (the Black Market), 35 (the Hill
  Fort), 99 and 221 (the Trading Posts), 106 (the War Machine Factory) and 213 (the Freelancer's
  Guild) on the 160 shipped maps (T97c's counts in `E08-log.md`, T97c); their `.h3m` bodies and
  templates (`docs/formats/h3m.md`, VCMI 1.7.5's reader); their data in the original data (the
  text tables the harvest reads, the executable's tables: the market's rates, the war machines'
  and the upgrades' prices); the original's code (`docs/exe/README.md`, `scripts/exe.py`): each
  object's visit and its dialog's rules (what it sells or buys and at what price; the Black
  Market's artifacts, when they are drawn and whether they are refreshed; the Hill Fort's upgrade
  prices; the Altar's experience for what it takes; the Trading Posts' and the Freelancer's
  Guild's rates against the town's Marketplace, E09 increment 5's), owners if any, a revisit; what
  a viewer sees of each; VCMI's and the wiki's rules; disputes `D-<n>` from 70, reading quirk
  candidates `A-<n>` from 46; the proposed rule ids; the batch's plan (E14) on the client lane's
  harness (decision 362's agent's call); what the market's rates need, told to the town lane's
  inbox. Likely in parts (by object or by kind of trade), confirmed with the user at its start,
  the reading measured first.
- **Inputs:** decisions 361, 362; ADR-0015 to -0017 (ADR-0016 points 3, 4 and 6, ADR-0017's
  decisions and rests), ADR-0013 (the hero's artifacts and experience); increment 6's army
  commands, increment 7's artifacts and rewards, increment 10's hire dialog (a purchase, creatures
  joining an army), increment 13's army dialog for a reward; E09's market (`docs/epics/E09-towns.md`,
  increment 5's Marketplace; the town lane's queue) and its map Tavern (E09 decision 106; the
  kinds `Tavern` and `DenOfThieves` in your adventure part by E09 T39a, the inbox's T33b note);
  the lane's `TRAPS.md`.
- **Depends on:** T126 (done). **Status:** ready (the original data and the shipped maps: staging
  per `docs/08-work-process.md`, "Cloud sandbox").

## T127a2 (moved 2026-11-18, done whole: `adventure/e08-t127a2`, decision 364)

**E08 T127a2 — the trades and services survey, part 2: the services and the checkpoint**
(decisions 362, 363; survey `docs/rules/surveys/E08-trades.md` from §12)

- **Scope:** the code reading of the Altar of Sacrifice (2: its dialog, what it takes by the
  hero's alignment and the experience for each artifact or creature, the team's kind mark k 28),
  the Hill Fort (35: its dialog, which creatures it upgrades and at what price, the mark k 14) and
  the War Machine Factory (106: the question, the hire dialog's war machines, a count of 4000
  each, their prices and what a hero already holding one can buy); `visit_object`'s three cases
  are mapped (§2); the computer's routines not followed. Their VCMI 1.7.5 and wiki comparison,
  disputes from D-72 and reading quirk candidates from A-47; the interfaces' sketch for all seven
  classes (§0 to §11's open points: the window as a decision or as commands beside the object, the
  slider's validation, the rates' representation, what a viewer keeps of a Black Market's stock,
  the transfer's event); batch E14's plan on the client lane's harness (D-71's sale, A-46's
  purchase with a full backpack, the market not restocked after a month, the Trading Post's rates,
  the last creature kept, and the services' rows); the proposed rule ids; the checkpoint's
  questions (D-71, A-46 and part 2's), asked with the user at its end.
- **Inputs:** survey `E08-trades.md` §0 to §12 (T127a1); decisions 361 to 363; ADR-0015 to -0017,
  ADR-0013 (the hero's experience and artifacts); increment 6's army commands, increment 10's hire
  dialog (`recruit_dialog_proc`'s war machine branch, `E08-dwellings.md` §5), increment 13's
  army dialog; `docs/exe/README.md`; the lane's `TRAPS.md`.
- **Depends on:** T127a1 (done). **Status:** ready (the executable and `extracted.tar` staged per
  `docs/08-work-process.md`, "Cloud sandbox"; the census is done).

## T127b (moved 2026-11-18, done whole: `adventure/e08-t127b`, decision 369; the checkpoint ADV-Q58, decisions 370 to 373; T128 queued)

**E08 T127b — the ADR amendment drafts, the design note and the cut** (decision 362)

- **Scope:** as T118b for increment 13: the drafts `adr-drafts/0016-increment-14.md` and
  `0017-increment-14.md` (and ADR-0013's if the survey finds a change of the hero's derived
  properties; the town lane's ADRs where the market's rates are shared, with its consent), increment
  14's design note `E08-design.md` (its parts by side), the building tasks' cut and the exit
  criteria; the checkpoint with the user (the note, the drafts, the agent's calls, the cut).
- **Inputs:** the survey `docs/rules/surveys/E08-trades.md` (its open points §10 and §16, the
  interfaces' sketch §17, batch E14 §18, the proposed ids §19); decisions 361 to 368 (the
  checkpoint's answers 365 to 368: the original's roundings, A-46 and A-48 fixed, the Altar's
  special artifacts as the original, batch E14 before the build: its spec and recipe in the cut); ADR-0015
  to -0017, ADR-0013; the town lane's answer on the market's rates, if any; T118b's drafts as the
  form.
- **Depends on:** T127a2 (done, its checkpoint answered). **Status:** ready (docs only).

## T128 (moved 2026-11-19, done in two parts: `adventure/e08-t128`, decisions 374, 375; T129 queued)

**E08 T128 — batch E14's maps: the writer's seven classes, the spec's fields, the specs** (round 2;
decisions 368, 373; as T119 for increment 13)

- **Scope:** design note `E08-design.md` §20, §21's round 2: `h3_map::writer` writes the seven
  classes by template (2, 7, 35, 99 and 106 at subclass 0; 213 and 221 in the AB and SoD formats
  only; no body); `h3-experiment`'s spec gains them by template and what its adventure hero lacks
  at the start (secondary skills such as Advanced Learning, a hero of the Learning specialty by
  type, war machines, worn artifacts and a full backpack, the map's banned artifacts, Blue's
  starting resources, the RoE format; battle-owned: `cross-lane` with the user's consent, label
  it); the specs `experiments/specs/E08-batch-14.toml` (SoD: survey §18's E14-1 to E14-13 and
  E14-15 with the note's E14-16 and E14-17; Red and Blue; days 1, 2 and 29) and
  `E08-batch-14-roe.toml` (E14-14).
- **Inputs:** survey `docs/rules/surveys/E08-trades.md` §1 (the templates, as corrected: all
  entered), §18 (the rows); design note §20; T119's record in `E08-log.md` as the form.
- **Depends on:** T127b (done). **Status:** ready (the original data: the executable and both
  tars, to compile the maps and check them in the editor's reader; confirm the cut at its start).

## T129b (moved 2026-11-19, done whole: `adventure/e08-t129b`, decision 377; T130 queued)

**E08 T129b — batch E14's services and the whole run** (round 3; decisions 368, 373, 376; T129a
the markets, done)

- **Scope:** design note `E08-design.md` §20's harness and recipe for the services: the capture
  harness's reads (`clients/capture/`, common): the Altar's dialog (`altar_dialog_create` 0x55FDD0:
  +0x78 the total, the places' +0xC0 vector, the slider's most, its two screens, Sacrifice, All,
  the creatures' amounts +0x138), the Hill Fort's rows (`hill_fort_rows` 0x4E7EB0: prices, states,
  the whole +0x298; items 261-267 and 204), the factory through the hire dialog's existing reads
  (GAM-DEC-008), the hero's experience, level, skills, artifacts, army and tile (earlier batches'
  reads), the dialogs' order (E14-17: the level-up over the Altar); the recipe
  `clients/capture/batches/E08-batch-14.toml`: E14-8, E14-9, E14-10, E14-12, E14-15 on Red's day 1
  after E14-5 (its comment marks the place), E14-13 and E14-17 on Blue's (Blue's step marks it),
  as the template's instructions order them; a recipe `E08-batch-14-roe.toml` for E14-14 on the
  RoE map; then one run of each recipe filling the templates `experiments/results/E08-batch-14.md`
  (every section, the market ones again) and `-14-roe.md`.
- **Inputs:** T129a's record in `E08-log.md` (the trade window as the form: a screen kind, its
  globals or fields, items by name, `amount`); the specs' headers; `clients/capture/README.md`
  ("Markets", "Dwellings, the hire dialog ...", "Creature banks": `order`).
- **Then T130:** scored in the survey's new §21.
- **Depends on:** T129a (done). **Status:** ready (the original data: the client lane's staging
  list; compile both maps again: stamps `7945e78d` and `a88dd771`; the maps not yet copied to the
  user's folder).

## T132b (moved 2026-11-19, done whole: `adventure/e08-t132b`, decision 382; T133 queued)

**E08 T132b — the import of the seven trade classes** (round 5; decisions 373, 378, 381; design
note §7, §21; T132a1 and T132a2 done)

- **Scope:** `h3-map`: the import of classes 2, 7, 35, 99, 106, 213 and 221 at subclass 0 as the
  part's six types (221 as `base:trading_post`, the map's class kept for its picture; no body; no
  RoE mark, decision 378; another subclass refused), the writer's classes for fixtures (T128a's
  `TRADE_OBJECTS`), a step onto one of the seven refused as an unbuilt visit's until T133 and
  T134 (`effect` none); map import's next version (v27); the census of the seven classes over the
  shipped maps (survey §1: 1 058 objects on 148 maps, every one at subclass 0) in the map reader's
  original-data check, moved from check 8 (design note §6's *As built*); the shipped-map counts
  that move (`TRAPS.md`); `tests/trades.rs`; the typed classes' test (`tests/import.rs`) taking
  the six into its cases.
- **Inputs:** design note §6, §7; `h3-map`'s `import/codes.rs` (the class table), T123b's record
  in `E08-log.md` as the form; `TRAPS.md` (the shipped maps' counts, the goldens).
- **Depends on:** T132a's pull request. **Status:** ready; its cut confirmed at its start.

## T133 (moved 2026-11-19, cut in three at its start: `adventure/e08-t133a1`, decision 383; T133a1 done, T133a2 and T133b queued)

**E08 T133 — the market** (round 6; decisions 373, 382; design note §8 to §13, §17, §21;
T132b done)

- **Scope (§21's plan):** **T133a** `h3-adventure`: the Black Market's stock in the static map
  (`Resolved::Stock`) and its sold entries in the visits' block (§8; adventure state hash v27),
  the draw when the game is built (§9, ADV-MKT-006: three treasures, three minors, one major from
  the one pool, the map's bans out, never restocked), the visits' outcome of the four markets
  (§10: entered, the window's screens) and the adventure's checks and changes of a trade, a
  transfer, a sale and a purchase (§11; ADV-MKT-003 to -005, -007; A-46 fixed as ADV-QUIRK-48);
  **T133b** `h3-game`: the market's decision (§13, GAM-DEC-011; game state hash v35), what a
  viewer sees (§12, ADV-MKT-008: the stock only in its visitor's open decision) and the events
  and the wire (§17; `cross-lane`, the server lane's `h3-protocol`).
- **Inputs:** design note §8 to §13, §17, §19; survey `E08-trades.md` §2 to §8; `h3-adventure`'s
  `market.rs` (`MarketParams`, T132a1) and `ruleset/trades.rs` (T132a2); T124b1's and T124b2's
  records in `E08-log.md` as the form (a draw in the static map, a visit); `TRAPS.md` (the
  codegen units, the visits' block, the plans' oracles).
- **Depends on:** T132b's pull request. **Status:** ready; its cut confirmed at its start.

## T133a2 (moved 2026-11-20, done whole: `adventure/e08-t133a2`, decision 383; ADV-Q60 for T133b)

**E08 T133a2 — the market's visit and the adventure's checks** (round 6; decisions 373, 383;
design note §10, §11, §21; T133a1 done)

- **Scope (§21's plan, decision 383):** `h3-adventure`: the visits' outcome of the four markets
  (§10: entered, every visit, every player; `AdventureOutcome::service` with
  `ServiceVisit::Market(MarketVisit { hero, index, object_type, anchor })`; the markets' `effect`,
  so a step onto one is no longer refused) and the adventure's checks and changes of a sale and a
  purchase (§11: `check_sale`, `sell`, ADV-MKT-005, the army keeping a creature; `check_buy`,
  `buy`, ADV-MKT-007, the entry drawn and not sold, its price not 0, the hero able to receive the
  artifact: A-46 fixed as ADV-QUIRK-48; the sold entry written by `Visits::sell`). The game opens
  nothing until T133b: `ADV-STUB-18` (registered in `docs/stubs.toml`).
- **Inputs:** design note §8 to §11 (§8 and §9's *As built*: T133a1); survey `E08-trades.md` §2,
  §6, §7.2; `h3-adventure`'s `market.rs` (`MarketParams`: `creature_lot`, `artifact_price`),
  `state/markets.rs` (T133a1), `ruleset/trades.rs`; T124b2's record in `E08-log.md` as the form
  (a visit's outcome); the hire's `check_hire` and `hire` (ADR-0017 point 10's shape);
  `TRAPS.md` (the codegen units, the visits' block, the plans' oracles).
- **Depends on:** T133a1's pull request. **Status:** ready (decision 383).

## Inbox notes settled or for information (moved 2026-11-18, E08 T126)

*(Moved verbatim at increment 13's closure: the other lanes' notes of changes in this lane's paths (the town lane's E09 T29b to T38b1, the map client lane's harness rebuilds), built on by T123b to T125b2b and T126's audit, CI green on each; their lasting rules are one line in `TRAPS.md`, "The town lane's parts of your crates"; the versions they took are the HANDOFF's.)*

- 2026-11-18 (the town lane, E09 T38b1, `town/e09-t38b1`, `cross-lane` with the user's consent;
  **your paths changed, for your review**): `AdventureState::end_battle` returns the heroes that
  left the map (`Vec<HeroOnMap>`, in the order given, as each stood before the end: the army it
  fought with, its formation, tactics flag and points; its marks gone, its boat freed), and
  `release_town_hero` the garrison hero it took (`Option<HeroOnMap>`); `h3-game`'s `end_battle`
  hands them to the town lane's `town::leave::after_battle` **last, after the winner's levels**
  (TWN-TAV-006: a retreated or surrendered hero back into its owner's tavern, its slot drawn
  from its owner's stream, `TavernOffered` told last; every other's type free with one creature
  and its spell points full, untold); `GameState::free_hero_types` is gone. A new way off the map
  (E09 T38b2's dismissal) goes through `town::leave::leave`. The plans' battle oracle
  (`check_heroes_after_battle`) lets a hero that leaves without returning end at its full spell
  points. Golden replays re-recorded (the battles' leaves moved their hashes); no version moved.

- 2026-11-18 (the town lane, E09 T38a2, `town/e09-t38a2`, `cross-lane` with the user's consent;
  **your paths changed, for your review**): `h3_game::legal_commands` lists each own town's hires
  after its recruitments (TWN-TAV-003) and now folds its chain into a list sized 16 (`for_each`,
  `legal.rs`: `collect()` cost `legal-towns-36` +17 % with the longer lists, the town lane's
  `TRAPS.md`); a new part of the listing goes into that fold. The plans hire (`h3-map`:
  `town_builds` probes `Hire`, `check_town_hire`, `Step::Town` hires in one step of eight), and
  four oracles know a hired hero: `check_isolation` (it joins the map), `check_conservation` (the
  price, its army from the slot's view, the refill's record), `view.rs`' told views (`HeroHired`
  shows it on its tile; `check_hired` folds the owner's slot view with `HiredPartSeen`) and the
  hygiene (a hire the viewer is told ends the comparison). Every random game on a map with owned
  towns hires now (env encoding v12): six workloads of yours on the town maps re-baselined
  (`reach-towns-36` +194 %, `day-start-builds-36` +105 %, `month-start-builds-36` +69 %,
  `legal-towns-36` +60 %, `day-start-towns-36` +51 %, `fork-towns-36` +39 %; more heroes, the
  same counted code). No hash, replay or protocol version moved.

- 2026-11-17 (the town lane, E09 T38a1, `town/e09-t38a1`, `cross-lane` with the user's consent;
  **your paths changed, for your review; replay format v17 taken**): the adventure replay format is
  **v17** (`town hire <town> <slot>`, `h3_towns::TownCommand::Hire`; `h3-files`' word and doc,
  `h3-sim`'s `REPLAY_FORMAT_VERSION`, the 89 goldens' `format` line only, `h3-cli`'s and `h3-sim`'s
  literals): your next bump is v18. `h3-game` gained `GameEvent::HeroHired` and `HiredPartSeen`
  (`observe.rs`' `project`, `h3-map`'s `tests/view.rs` told-views arms) and
  `GameState::taverns_for` (`state.rs`); `h3-adventure`'s town module gained `check_tavern_hire`
  and `place_hired` (`src/town/hire.rs`; `rows.rs`' `town_entrance` and `arrive` made
  `pub(super)`). No hash version moved; the hire is not in `legal_commands` until E09 T38a2
  (`TWN-STUB-19`), which also changes `legal.rs` (`cross-lane`).

- 2026-11-17 (the map client lane, E24 M4 T1b1, `map-client/e24-m4-t1b1`; **the capture harness, rebuild `xdd.dll`**): `clients/capture/xdd.c` takes a new command, `<seq> seed <value>` (its 8.: the main thread's C library seed set at its next clock read, so that a computer player's turn draws alike in every run; `Game.seed`, and `h3drive.to_scenario(game, seed=...)` seeds just before Begin), and its acknowledgement has a sixteenth field (the seeds set): `h3drive.ACK_FIELDS` is 16, so a run with an `xdd.dll` built before fails as stale where it reads the acknowledgement whole (a forced roll, a batch's or a sequence's checks, a seed): rebuild it with `clients/capture/setup.sh`. Nothing else of `h3drive`, `h3adv` or `h3mem` changed in behaviour (`Mem.other_turn` is new; `Mem.status_items` also returns kind 5's items, ids 2190 to 2193).

- 2026-11-17 (the town lane, E09 T37b, `town/e09-t37b`, `cross-lane` with the user's consent;
  **your paths changed, for your review**; design note `E09-design.md` §70 As built, decisions
  137, 138): **map import v25** (`h3-map`: `GameSetup::custom_types` from SoD's map-wide entries
  that set a field, a type code past the table refused; `hero_type_of` factored out of `hero`):
  your next import change takes v26. `h3-game`'s `setup.rs`: `GameSetup::custom_types`,
  `HeroDetailsSetup::over`, `GameSetupError::CustomType` and `CustomDetails` (checked after the
  banned sets, before any draw), every hero's details its own over its type's entry (a random
  hero's by its drawn type, a main town's), a record per customised type still free after the
  build's heroes (`town::custom_records`), `Prepared`'s last part each store hero's
  customisations as built; `state.rs`: the level seeds, sexes and levels at load read those (the
  records last); `tests/mod.rs`' `levels` module and `levels::oracle` `pub(crate)` (the town tests
  reuse the oracle). No hash version moved; the game state hash's encoding is unchanged. The 89
  goldens re-recorded: `import_version` 25 in each, and `army-meeting-s1` and `army-dialog-s1`
  moved their checkpoints and final hashes (`e08-army-12`'s engine-written entry of type 3 now
  gives a record at the build); the bot reports unchanged. **A `GameSetup` literal** needs
  `custom_types` (four did: `h3-cli`'s census, `h3-map`'s census example and `tests/rewards.rs`,
  `h3-game`'s `tests::setup`).

- 2026-11-17 (the town lane, E09 T37a, `town/e09-t37a`, `cross-lane` with the user's consent;
  **your paths changed, for your review**; design note `E09-design.md` §61 to §63, §68, §75 As
  built): **game state hash v33** (the taverns' stream after the towns'; the pool's offered status,
  records, away records, factions and tavern slots). `state.rs`: `taverns_rng`, `pool()` and
  `taverns()` (the tavern parts' borrows), the start's fill in `GameState::new` before the first
  group's turn, `town::tavern::start_turn` after the adventure's turn start, the marks' clear at
  `EndTurn`; `state/day.rs`: the week's fill after `grow_week`, before your week step, the marks'
  clear before the towns' day, `regain_mana` skipping a tavern record the pool marks full
  (`Away::full`: `day_mana` never changes points at the maximum); `state/levels.rs`: the seeds and
  sexes a `Vec`, `Progression::record`; `setup.rs`: `hero_details` takes a `HeroDetailsSetup`,
  `starting_army` and `wear_machines` split out, the records and factions kept after the main towns'
  heroes; `command.rs`/`observe.rs`: `GameEvent::TavernOffered`, `TavernPartSeen`,
  `GameObservation::tavern`; `h3-heroes`: `backpack_equipped` (ADR-0013 point 14.5), `first_free`
  and `fits` without allocating the positions; `h3-map`'s told-views property folds the tavern
  heroes. The 89 golden replays re-recorded (only hash lines moved); the week-start benches
  re-baselined (the week's fill). **When you write a tavern hero's record** (none of yours does
  today): clear its `Away::full` and set `Away::stale`, or the day's regain and the turn's points
  skip it.

- 2026-11-17 (the map client lane, E24 M4 T1a, `map-client/e24-m4-t1a`; **the capture harness, rebuild `xdd.dll`**): `clients/capture/xdd.c` forces two more draws (`week_name` 0x4C87A6, the week's name, and `month_name` 0x4C8FCC, a normal month's; `h3drive.ROLLS` grew by two), so a batch or sequence run with an `xdd.dll` built before fails the count of redirected calls: rebuild it with `clients/capture/setup.sh`. Nothing else of `h3adv` or `h3mem` changed in behaviour (`Mem.calendar` is new: the week's and the month's kinds and names).

- 2026-11-17 (the town lane, E09 T36, `town/e09-t36`, `cross-lane` with the user's consent; **your replays' header, mechanical**): the town ruleset part v7 (the records' tavern weights, the hire's price) moved `town_ruleset_hash` 0x0d7be940b64312a6 to 0xabc9756b93a29a55 in all 89 replays (re-recorded from their commands, nothing else moved). As with T35b's note below: a replay recorded on a branch from before the merge carries the old line. `TownRecord::tavern` and `TownParams::hire_price` are read by no rule yet (E09 T37a, T38a).

- 2026-11-17 (the town lane, E09 T35b, `shared/e09-t35b`, `cross-lane` with the user's consent; **your replays' header, mechanical**): two `quirk.town` flags (`QuirkFlag(42)`, `(43)`, on in `faithful`) moved the base faithful ruleset hash 0xb5c01c8736eb73f9 to 0x6ba8109889b34a25; your 86 replays on it keep commands and state hashes, only `ruleset_hash` moved (the three strict-fixed ones unchanged). A replay you record on a branch from before the merge carries the old line: re-record it or patch the line after merging `master`. `StreamId::TAVERNS` (8) is new, no draw yet (E09 T37a draws from it).

- 2026-11-16 (the town lane, E09 T35a, `shared/e09-t35a`, docs only; **ADRs in your paths, for your planning**): E09 increment 4's drafts are in the ADRs, each part after E09 increment 3's (so before your increment 10 to 13 parts where the points interleave): ADR-0016 points 3 (a hire's placement with the game's away record, the leave's hand-back, the dismissal's removal), 4 (the kinds `Tavern` and `DenOfThieves`, after E09's kind `Town`) and 6; ADR-0017 points 1 to 4, 6, 8 to 11; ADR-0013 points 14.5 and 16.5; each Consequences and Alternatives a line. Two decisions at T35's start (`TWN-Q19`, E09 decisions 126, 127), both for the town lane's T37a in your `state/day.rs` (`cross-lane` then): every day's start clears every player's tavern returning marks after the taverns' week (the original's `new_day` 0x4C80CF to 0x4C80FD), and `GameState::regain_mana` stays over every record of the store, tavern heroes and heroes out of the game included (HER-MAN-002 as you built it; the design's planned exception dropped). ADR-0017 point 9 and ADR-0028 point 9 name no adventure state hash bump for increment 4 (design §77: the kinds hold no state). Nothing for you to build now.

- 2026-11-16 (the town lane, E09 T34, `town/e09-t34`, `cross-lane` with the user's consent; **the capture harness and your writer**): `clients/capture/xdd.c` logs three more sites (`tavern_class`, `tavern_type`, `leave_slot`, with their ranges; `h3drive.LOGGED` grew by three), so a sequence or batch run with an `xdd.dll` built before it fails the count of redirected calls: rebuild it with `clients/capture/setup.sh`. Also new and common: the reads and actions of README "Taverns and the Thieves' Guild" (the tavern dialog, the guild window, a town opened from the town list, a step's `resources`, `mana`, `narrow_pool`, a field `<encounter>|<field>`), and `auto_combat` returns when the battle ended before a human stack's turn (a loss to faster monsters kept pressing `a` until the limit). In your `h3_map::writer`: `TAVERN_OBJECT` (95) and `DEN_OF_THIEVES_OBJECT` (97) joined `PLAIN_OBJECTS` (no body; `tests/taverns.rs`): merge in order if you touch the list (the import of both stays the town lane's T39a).

- 2026-11-15 (the town lane, E09 T31, `town/e09-t31`, `cross-lane` with the user's consent, E09
  decision 101; **your paths changed, for your review; no version moved**): `command.rs`'s
  `AdventureEventKind::GarrisonEntered` and `GarrisonLeft` have a field `named: Viewers`, the
  viewers told the move by name (the owner's team and, from the game, a viewer whose view of the
  town is at level 3 by Visions); `observe.rs`' `AdventureEvent::project` tells any other viewer
  `HeroRemoved` at the entrance and `HeroStepped { from: None, to: Some(entrance), .. }` (a hero
  coming into sight) in their place, as a step's unseen ends are left out. `h3-map`'s
  `tests/view.rs`: `views_move_as_the_events_say` checks each viewer's projection of every garrison
  move (`check_garrison_moves`, beside `check_jump_ends`). No hash, replay or format moved (events
  are not hashed). A new constructor of either event passes `named` (`Viewers::NONE` names the
  owner alone: `move_town_hero` adds it).

- 2026-11-15 (the town lane, E09 T29b, `town/e09-t29b`, `cross-lane` with the user's consent, E09
  decision 97; **your paths changed, for your review; no version moved, no engine code**):
  `h3-map`'s `play.rs`: `Step::Town` (`visit_town`) gives a listed `TownCommand::Heroes` move at
  an own town in five steps of eight, so every property's and cover test's games with town runs
  moved (all of `h3-map`'s tests pass, the properties also at 240 cases); `town_events` gives
  65 535 of a level where it drew 20; `check_invariants` bounds the towns' counts;
  `check_towns` takes the command (the Plague, a purchase's amount); a new property
  `a_forks_week_is_its_parents`; `hero_anywhere` (a hero on the map or a garrison hero) in
  `check_battle_end`, `check_heroes_after_battle`, `check_conservation` and `check_progression`,
  and `check_isolation` exempting the battle's garrison hero, which met a garrison hero
  defending, raising and levelling up. `view.rs`' `vary` keeps a map garrison's entrance land (a case
  of `master`'s, `OwnedOnWater`; your E08 T116b1's ownable garrisons were not in its
  `entrances`). A new workload `benches/adventure/month-start-builds-36.toml` (the step that
  starts day 29 on the builds map), its README row by #942.

*(Moved 2026-11-19, E08 T129b: settled by E08 T129a, #1037: `find_gains` shared with `visit_gains`, the seed kept in `proptest-regressions/tests/play.txt`.)*

- 2026-11-19 (the town lane, E09 T39b1, found while confirming its baseline; **`master`'s CI red,
  for you to fix**): CI's run on `master` 450e47e (#1032's merge, run 44482680316) failed in
  `h3-map`'s `tests::play::every_step_keeps_the_commands_invariants` at `play.rs:9615` (a won
  battle against a monster guarding an object: the gains told `[(#4, 3)]` against the oracle's
  `[]`, after `ObjectRemoved { object_type: #30, anchor: (1, 1, 0) }`; an RoE plan of side 8). Its
  seed, `cc 927e83f83344d60c406dca3ea29fb9a601810f09228a5912e618f5408cb989d2`, reproduces it on
  T39b1's branch when appended to `proptest-regressions/tests/play.txt` (not committed: it would
  fail every run until fixed). Nothing of T39b1 is in that path (the guild is a query).

## T134a (moved 2026-11-20, done whole: `adventure/e08-t134a`, decision 385; the places' bitmask, decision 386)

**E08 T134a — the Altar of Sacrifice** (round 7; decisions 366, 367, 373; design note §11's
sacrifices, §14; survey `E08-trades.md` §12, §16)

- **Scope:** `h3-adventure`: `ServiceVisit::Altar` (the visit, the screens by the hero's class's
  faction, the kind mark, ADV-ALT-001, ADV-BON-009), the sacrifices' checks and changes (§11:
  ADV-ALT-002 the artifacts, -003 the creatures; A-47 accepted as ADV-QUIRK-49); `h3-game`:
  `Decision::Altar` (GAM-DEC-012, game state hash v36), `Answer::Altar`, `AltarOpened`, the view,
  the levels inside the Altar and `Rest::Visit` (GAM-DEC-007 amended; A-48 fixed as ADV-QUIRK-50:
  a sacrifice names everything in one answer); the wire (`cross-lane`, protocol v76), the replay
  words (format v21) and the env's `Leave` on the closing entry (encoding v15), as T133b's
  (decision 384: a new answer forces them).
- **Trap:** `h3_game::Answer` is `Copy` (`h3-sim`'s `const fn answer_entry`, the replay's
  `format_answer(*answer)`): §14's `Artifacts(Vec<ArtifactPlace>)` cannot be one; a fixed array
  (at most 23 places with a count) or a decision to drop `Copy` (every lane's matches), asked at
  the start.
- **Depends on:** T133b's pull request. **Status:** ready after it; its cut confirmed at its start.

## Inbox items done at E08 T133b (2026-11-20)

The client lane's format v20 (E27 M4 T3a, CLT-Q156 (a)), merged into T133b's branch, whose market words took v21 (decision 384).

- 2026-11-20 (the client lane, E27 M4 T3a, `client/e27-m4-t3a`, `cross-lane` with the user's
  consent, E27 decision 71; **your replay format moved: increment 14 takes v21**): your inbox's
  format extension of 2026-11-19 (CLT-Q156 (a)) is built. **Adventure replay format v20**:
  `h3_sim::adventure::replay::PlayerChoices` (a player's colour, human flag, `FactionChoice`,
  pick), `Replay::players` (every player's, recorded from the setup by `record_random` and
  `record_commands`; none: the import's), `Replay::setup` (the import at the replay's difficulty
  with the choices set over it; a colour no player has or chosen twice is
  `Incompatible("players")`), `verify` building from it; `h3-files`' `[[players]]` tables
  (`faction` an index or `"random"`, `hero` an index; absent without one). The 104 golden replays
  were recorded again from their commands: only `format = 20` and their `[[players]]` lines moved
  (no hash). Your HANDOFF's versions line says so (replay format v20, increment 14's next v21);
  your design note §19's `answer market …` lines then take v21. A golden of yours recorded at v19
  on a branch open meanwhile needs `format = 20` and its players (record it again).

## T134b (moved 2026-11-20, cut in two: T134b1 done, `adventure/e08-t134b1`, decision 387; T134b2 queued)

**E08 T134b — the Hill Fort and the War Machine Factory** (round 7; decisions 373, 378, 384;
design note §4, §5, §10, §11's upgrade and factory, §15, §16; survey `E08-trades.md` §13, §14)

- **Scope:** `h3-adventure`: `ServiceVisit::HillFort` (the visit after the kind mark k 14,
  ADV-HFT-001, ADV-BON-009), `hill_fort_rows` and `upgrade` (§11, ADV-HFT-002: each slot's row by
  the enabled packs' upgrades, its price the gold difference by level and truncated once, a
  level-1 stack free; D-74 the original's); the factory's hire question and dialog (§10, §16:
  `OfferKind::Hire` with the factory as `HireSource::Object`, `check_hire` and `hire` extended for
  its machines, ADV-WMF-001, ADV-HIR-002 amended: the hero lacking the machine's artifact, the
  artifact given into its position); `h3-game`: `Decision::HillFort` (GAM-DEC-013, the game state hash's
  next), `Answer::HillFort` (`Upgrade(slot)`, `All`, `Leave`), `HillFortOpened`, the view; the
  factory's purchase through the hire dialog (GAM-DEC-008 amended, `HeroChanged`); the wire
  (`cross-lane`), the replay words and the env's `Leave`, as T133b's and T134a's (decisions 384,
  385: a new answer forces them); each version the next free at its task (today: game state hash
  v37, protocol v77, format v23, encoding v16).
- **Depends on:** T134a's pull request. **Status:** ready after it; its cut confirmed at its start
  (the user's working agreement: a seam if it is big, as T134a's offer of T134a1 and T134a2).

## Inbox notes settled at E08 T135b2 (2026-11-21)

- 2026-11-21 (the town lane, E09 T47a2, `town/e09-t47a2`, `cross-lane`; **versions taken and the
  goldens re-recorded, nothing to do now**): map import **v28** (a town's required and banned
  spells into `TownSetup`, TWN-SET-012) and game state hash **v38** (the store of towns v4: the
  guild slots, the Vortex flag, the Aurora set; the guilds drawn at the build as the towns'
  stream's first draws) are taken: your next import change takes v29, your next game hash v39. The
  104 golden adventure replays re-recorded from their commands (the hash lines and the two versions
  only), but `towns-s173-d21`, whose day-16 siege no longer fit (the towns' stream grew the neutral
  garrison otherwise): that battle played again by `h3_battle_ai::choose` for both sides, the rest
  of its commands kept, the same features; your T135b1's seven `trades-*` too (hash lines and
  versions only). `TownSetup` gained two fields: every struct literal
  (`h3-map`'s examples and `tests/towns.rs`) has them. `GameSetupError::TownSpell` is new. *(Settled: the HANDOFF's versions line names v38 and v28 taken, the next v39 and v29.)*

## T136 (moved 2026-11-21, done whole: `adventure/e08-t136`, decision 393; increment 14 closed, increment 15 refined: decisions 394, 395; T137a and T137b queued)

The "Tasks" text as it stood:

Epic: `docs/epics/E08-adventure-map.md`; done tasks in `E08-log.md`; the chronicle of the rounds T1 to T126 (cuts, pull requests, decisions) is in `QUEUE-ARCHIVE.md`. Increments 1 to 13 are closed (2026-10-12, -13, -20, -21, -23, -26, -28, -30, -11-03, -11-05, -11-11, -11-15, -11-18; increment 13, creature banks, by T126's audit). **Increment 14, the trades and services**, refined with the user at T126 (ADV-Q56; decisions 361, 362): round 1 done: the survey (T127a1, T127a2; decision 363) and the drafts with the design note and the cut (T127b, whole: decision 369); the checkpoints ADV-Q57 and ADV-Q58 answered (decisions 365 to 368, 370 to 373). The building tasks T128 to T136 (decision 373, design note §21): T128 done (decision 374), T129a and T129b done (decisions 376, 377), T130 done (batch E14 scored; the checkpoint ADV-Q59: decisions 378, 379), T131 done (the `shared/` task), T132a1 and T132a2 done (the rates, the kinds; decisions 380, 381), T132b done (the import; decision 382), T133 cut in three (decision 383): T133a1 and T133a2 done (the stock and its draw; the visit and the adventure's checks), T133b done (the decision and its wire; decision 384), T134a done (the Altar with its wire; decisions 385, 386), T134b in two (decision 387): T134b1 done (the Hill Fort with its wire), T134b2 done (the factory; decision 388), T135a done (the listing and the env's entries; decisions 389, 390), T135b in two (decision 391): T135b1 done (the two maps and seven golden replays), T135b2 done (whole, decision 392: the plans and oracles, the two workloads, the original data). Round 9: T136 below. Every entry's acceptance includes the Definition of Done (`docs/05-testing.md`).

**E08 T136 — increment 14's audit** (round 9; design `E08-design.md` §21's exit criteria)

- **Scope** (`adventure/e08-t136`): the audit against the ten exit criteria with a fresh reviewer;
  increment 14 closed (its scope, decisions 361 to 392 and tasks to `E08-archive.md`, the design
  note to `E08-log.md`; the README's epic table in the session's `shared/` pull request);
  increment 15 refined with the user (the sketch's item 15, quests, or another order). Criterion
  10's benchmarks: T135b2 baselined the two new workloads and found no other on a map with a
  trade (its record in `E08-log.md`).
- **Inputs:** design §21 (the exit criteria), §18's *As built*; T126's audit (`E08-log.md`) as the
  form; the epic's sketch.
- **Depends on:** T135b2 (`adventure/e08-t135b2`, merged first). **Status:** ready once T135b2
  merges (the original data to stage for criterion 1's census and criterion 10's gates); its cut
  confirmed at its start.

## Inbox notes settled at E08 T136 (2026-11-21)

The town lane's notes of changes in this lane's paths (E09 T33b to T46), built on by T127 to T135b2; their lasting rules one line in `TRAPS.md` ("The town lane's parts of your crates").

- 2026-11-20 (the town lane, E09 T46, `town/e09-t46`, `cross-lane` with the user's consent;
  **your replays' header, mechanical; act only at a merge**): the town ruleset part v9 (the mage
  guild's weights, roles and parameters) moves `town_ruleset_hash` 0xab3ee3f1cb243f30 to
  **0xcf8db1aa2bb768d2** in all 104 golden adventure replays, re-recorded from their commands
  (nothing else moved: no state reads the part's new fields until E09 T47a). A branch of yours
  that re-records goldens across this merge keeps the new line. `h3-game`'s `tests/mod.rs` town
  pack literal gained the new fields (`guild`, `guild_slots`, `spell_book_price`,
  `vortex_factor`).

- 2026-11-20 (the town lane, E09 T45, `shared/e09-t45`, `cross-lane` with the user's consent;
  **your replays' header, mechanical; act only at a merge**): one `quirk.town` flag
  (`QuirkFlag(44)`, `quirk.town.aurora_ignores_map_bans`, `TWN-QUIRK-14`, on in `faithful`) moves
  the base faithful ruleset hash 0x6ba8109889b34a25 to **0x36b98ea152b4db95**: the 101 golden
  adventure replays on it re-recorded from their commands (only `ruleset_hash` moved; the three
  `strict-fixed` ones and the ten battle replays unchanged). A branch of yours that re-records
  goldens across this merge keeps the new hash (re-record after merging `master`). Also: ADR-0016
  point 4's kind `Town` holds E09 increment 5's `AdventureOutcome::town_entered` (T45a).

- 2026-11-20 (the town lane, E09 T44, `town/e09-t44`, `cross-lane`; **a field in your writer and
  the harness's town screen; nothing to do now**): `h3_map::writer::Map` gains `banned_spells:
  Vec<u8>` (the header's nine bytes, refused past 70 and in RoE) beside `banned_skills` and
  `banned_artifacts`; every literal in the workspace and the clients got `banned_spells:
  Vec::new()`, so a new `Map { .. }` literal in an open branch of yours needs it too. The town's
  `AnyTown::spells` (`TownSpells`) is the town lane's. The capture harness: a walk's town screen
  now waits for the screen to settle and passes a message that comes over it (the Mana Vortex's)
  before its `actions`; `hall = true` hovers the hall's box until the status line names it; new
  reads `town:x,y:guild`, `:guild_level`, `:week_flag`, `:standing` (`clients/capture/README.md`,
  "Towns"). The harness's own check passed after.

- 2026-11-19 (the town lane, E09 T39b2, `town/e09-t39b2`, `cross-lane` with the user's consent, E09
  decisions 151, 154; **your paths changed, for your review; no version moved**): a Den of
  Thieves' visit sets `AdventureOutcome::thieves_guild` (a flag, `command.rs`; `state.rs`' `visit`,
  `TWN-STUB-20` lifted; `Effect::DenOfThieves`' doc in `ruleset/objects.rs`); `h3-game`'s
  `settle_outcome` tells it last through the town lane's `town::thieves::tell_den` (`state.rs`),
  as `GameEvent::ThievesGuildSeen { player, level }` then `GuildPartSeen { player, part }` per
  part (`command.rs`; the owner's alone in `observe.rs`' `project`); `h3-map`'s told views
  (`tests/view.rs`) take both as the viewer's own; ADV-VIS-008's summary. Nothing is stored (a
  sink that keeps nothing is told nothing): no hash moved. The census: the 98 goldens' 12 maps
  lay out no Den, nothing re-recorded.

- 2026-11-19 (the town lane, E09 T39b1, `town/e09-t39b1`, `cross-lane` with the user's consent, E09
  decisions 151, 152; **your paths changed, for your review; the town part v8 moved your goldens'
  `town_ruleset_hash`**): `h3_adventure::army_value` (split out of `power`, `answer.rs`) and
  `power` are public, exported from `lib.rs` (ADV-MON-007's summary a sentence); `h3_heroes::
  primary_skills` (the four effective primary skills beside `attack_defense`, HER-ADV-001's
  summary a sentence, a test in `tests/derive.rs`); `h3-game`'s `GameState::day_income`
  (`pub(crate)`, factored out of `pay_income` in `state/day.rs`, the same sums: the Thieves'
  Guild's income row reads its gold). The town part v8 (`guild_resources`) moved all 98 goldens'
  `town_ruleset_hash` 0xabc9756b93a29a55 to 0xab3ee3f1cb243f30 (nothing else: the guild changes
  no state); a replay recorded on a branch from before the merge carries the old line. Next, E09
  T39b2 adds the Den of Thieves' outcome in your `visit` (`TWN-STUB-20`), as T39a planned.

- 2026-11-18 (the town lane, E09 T39a, `town/e09-t39a`, `cross-lane` with the user's consent, E09
  decision 149; **your paths changed, for your review; map import v26, part v22, game state hash
  v34 and replay format v19 taken**): two kinds, `ObjectKind::Tavern` (ADV-VIS-007: visited from
  a neighbour, its visit `AdventureOutcome::tavern`, an `h3_adventure::TavernVisit`) and
  `ObjectKind::DenOfThieves` (ADV-VIS-008: entered, its visit nothing until E09 T39b,
  `TWN-STUB-20` in `state.rs`' `visit`), their `Effect`s, defs (`tavern = {}`,
  `den_of_thieves = {}`), the visit-mode checks and hash tags 45, 46 (`ruleset/objects.rs`,
  **adventure part v22**); the base pack's `base:tavern` and `base:den_of_thieves` appended last
  among base's types, so the AB and SoD types' ids moved by 2 (the duel's and the sea's replays'
  hashes with them); `h3-map`'s `codes.rs` types classes 95 and 97 (**map import v26**; the
  typed-classes test's cases, `tests/taverns.rs`). `h3-game`: `Decision::Tavern` (hash kind 9,
  **game state hash v34**), `Answer::Tavern(TavernAnswer)`, `GameCommandError::Tavern`,
  `GameEvent::TavernOpened`, `DecisionView::Tavern`, `GameState::tavern`, the answer's arm in
  `state.rs` (the hire is the town lane's `town::hire::map_hire`), the listing and the default
  answer in `legal.rs`. `check_stand`'s doc: a map Tavern's hire places a hero on a
  neighbour-visited tile. The replay format **v19** (`answer tavern hire <slot>|close`): the 98
  goldens re-recorded from their commands (only the version lines, the part hash, the
  checkpoints and `final_hash` moved), `h3-cli`'s literals, `h3-server`'s `tests/record.rs`; the
  bots tables' `engine.adventure` line. The plans name the kind in `settle` and `hiring` only.
  Your next bumps: import v27, part v23, game state hash v35, replay format v20.

- 2026-11-18 (the town lane, E09 T38b2, `town/e09-t38b2`, `cross-lane` with the user's consent;
  **your paths changed, for your review; replay format v18 taken**): `h3_game::Manage::Dismiss
  { hero }` (GAM-MAN-003) with `ManageError::{NotOnMap, InTown, LastHero}` (`manage.rs`),
  applied in `state/manage.rs` (`DecisionOpen` unless no decision is open) through the town
  lane's `town::leave::dismiss`; `legal_management` lists it last, outside a decision
  (`legal.rs`), so `tests/manage.rs`' and `tests/exchanges.rs`' candidates hold it.
  `AdventureState::remove_hero((player, hero))` (`state.rs`): `end_battle`'s removal (the marks,
  `HeroRemoved`, the boat) is one private `remove_at` that both use. The adventure replay format
  is **v18** (`dismiss <hero>`, a bare management word; `h3-files`' word and doc, `h3-sim`'s
  `REPLAY_FORMAT_VERSION`, the 98 goldens' `format` line only, `h3-cli`'s and `h3-sim`'s
  literals): your next bump is v19. The plans (`h3-map`'s `tests/play.rs`): `managing` leaves
  the dismissals out, `dismissal` plays one in one `Step::Manage` of sixteen, and
  `check_management`, `check_conservation` and `check_isolation` know it. No hash version moved.

- 2026-11-16 (the town lane, E09 T33b, `town/e09-t33b`, docs only; **planned `cross-lane` changes
  in your paths, for your planning; nothing changed yet**): E09 increment 4's drafts are accepted
  (`TWN-Q18`; `docs/lanes/town/adr-drafts/0016-`, `0017-increment-4.md`; the building tasks T34 to
  T41 in `docs/epics/E09-design.md` §79, the order against your increment 13 there). In your
  paths, each `cross-lane` with the user's consent at its task: T35 amends ADR-0016 and -0017
  (merge order with your T122); T37a the game state hash (the pool's `Offered`, records and away
  records, the players' factions and tavern slots, `StreamId::TAVERNS`) and the week's step (the
  taverns' week after the towns'); T37b `GameSetup`'s customised hero types and the import (after
  your T123's map import v24); T38a the adventure's placement of a hired hero (ADR-0016 point 3:
  a town's entrance or a map Tavern's visited tile, with its away record; `HeroHired`), `legal.rs`,
  the replay format (`town hire`); T38b a hero that leaves the game composed after your battle's
  end in `state/battle.rs` (merge order with your T124b), the dismissal `Manage::Dismiss`
  (`manage dismiss`); T39a the kinds `Tavern` (class 95, visited from a neighbour) and
  `DenOfThieves` (class 97, entered; every visit tells the game's Thieves' Guild) in the adventure
  part, the import and the writer (after your T123); T39b `h3_adventure`'s `answer::power`
  (ADV-MON-007) made public for the guild's army row and best hero, and the day's gold factored
  out of `GameState::pay_income`. T34's fixture needs the writer's classes 95 and 97 (as your
  T119's banks).

## T137a (moved 2026-11-21, done whole: `adventure/e08-t137a`, decision 397; survey `E08-spells.md`)

**E08 T137a — the Prison, the Market of Time and the adventure spells survey** (decision 395; as
T127a for increment 14: survey file `docs/rules/surveys/E08-spells.md`)

- **Scope:** the census of classes 62 (the Prison) and 50 (the Market of Time) on the 160 shipped
  maps (T97c's counts in `E08-log.md`, T97c), their `.h3m` bodies (`docs/formats/h3m.md`: the
  Prison's is a hero's) and the prisoners' heroes against the hero pool (E09's `prisoners`); their
  visits in the original's code (`docs/exe/README.md`, `scripts/exe.py`): the prisoner freed for the
  visitor's player (its limits, its levels at load: the inbox note of 2026-10-20), the Market of
  Time's visit; the adventure spells Fly, Water Walk, Dimension Door, Summon Boat and Scuttle Boat:
  each one's effect by mastery, its spell points, what it refuses, a day's limits, a scroll's cast,
  the boats Summon Boat moves and Scuttle Boat sinks, boarding's clearing of Fly and Water Walk
  (survey `E08-hero-movement.md` §6); flying revisited and **A-13's trade-off** (the inbox note of
  2026-10-22; decisions 75, 78; `ADV-QUIRK-15`); the other adventure spells (Visions, View Earth,
  View Air, Disguise) named with their readings' size for the checkpoint (decision 394's agent's
  call; Visions' parameters are the town lane's, the inbox note of 2026-11-06); what a viewer sees
  of each; VCMI's and the wiki's rules; disputes `D-<n>` from 75, reading quirk candidates `A-<n>`
  from 49; the proposed rule ids; batch E15's plan on the client lane's harness. Likely in parts
  (the Prison and the Market of Time apart from the spells), confirmed with the user at its start,
  the reading measured first.
- **Inputs:** decisions 394, 395; ADR-0015 to -0017 (ADR-0016 point 4's flight and A-13, point 6),
  ADR-0013 (the hero's spells, mastery and spell points: `h3-heroes`, the global HANDOFF's trap);
  increment 5's survey `E08-hero-movement.md` (§2.1, §3, §6; A-13, A-14, D-24), increment 8's
  boats (boarding, landing, the Shipyard's boat), increment 4's levels; E09's spell book and mage
  guild (the town lane's increment 5: which spells a hero holds) and Town Portal's ownership (E09
  decision 6); the inbox notes of 2026-10-20, 2026-10-22 and 2026-11-06; the lane's `TRAPS.md`.
- **Depends on:** T136 (`adventure/e08-t136`, merged first). **Status:** ready once T136 merges
  (the original data and the shipped maps: staging per `docs/08-work-process.md`, "Cloud
  sandbox"); its cut confirmed at its start.

## T137b (moved 2026-11-21, done whole: `adventure/e08-t137b`, decisions 398 to 407, ADV-Q63; design note `E08-design.md`, the three drafts, T138 to T145 cut)

**E08 T137b — the ADR amendment drafts, the design note and the cut** (decision 395)

- **Scope:** as T127b for increment 14: the drafts `adr-drafts/0016-increment-15.md` and
  `0017-increment-15.md` (and ADR-0013's if the survey finds a change of the hero's derived
  properties), increment 15's design note `E08-design.md` (its parts by side), the building tasks'
  cut and the exit criteria; the checkpoint with the user (the note, the drafts, the agent's calls,
  the cut).
- **Inputs:** the survey `docs/rules/surveys/E08-spells.md` (its open points, the interfaces'
  sketch, batch E15, the proposed ids); decisions 394, 395 and the survey checkpoint's; ADR-0013,
  ADR-0015 to -0017; T127b's drafts as the form.
- **Depends on:** T137a (done 2026-11-21, `adventure/e08-t137a`); its questions (survey §17) are
  asked at T137b's start or checkpoint. **Status:** ready (docs only; the executable staged only if
  a reading is reopened).

