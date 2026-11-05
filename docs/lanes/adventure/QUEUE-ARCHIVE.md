# Adventure lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/adventure/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Completed tasks (the "Tasks" text as it stood on 2026-10-24)

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
