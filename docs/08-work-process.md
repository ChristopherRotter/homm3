# 08 — Work process for agents

## Planning levels (just-in-time refinement)

1. **Vision / architecture / ADRs** — stable, changed deliberately (ADR).
2. **Backlog** (`BACKLOG.md`) — coarse epics in rough priority order. Cheap to reorder. One or two lines each until they are close.
3. **Refined epic** (`epics/Exx-*.md`) — only for the next one or two epics: goal, scope, non-goals, exit criteria, task list. Refine an epic when it becomes next, not before.
4. **Task** — sized to fit one focused agent session (a coherent change that can be verified). If it does not fit, split it or deliver partially (below).
5. **HANDOFF** — the living state between sessions, within a size budget ("Keeping the start-of-session reading small"): the global `docs/HANDOFF.md` (lane index, the traps that cross lanes) and one per lane, `docs/lanes/<lane>/HANDOFF.md` (its standing traps in the lane's `TRAPS.md`).
6. **Lane queue** (`docs/lanes/<lane>/QUEUE.md`) — the lane's pre-refined tasks, each about half a context, and an inbox of discovered work ("Lanes").

Do not over-specify. A refined epic is allowed to be wrong after first contact; update it.

## Session protocol

**Start:** follow the checklist in `AGENTS.md`.

**Interruptions:** a session can be stopped at any time and resumed later, possibly by a fresh agent. Follow "Interruptions and resuming" in `AGENTS.md`: push every commit immediately, keep the lane HANDOFF resumable at task boundaries, checkpoint when the user announces a stop, and rebuild from GitHub (and the user's commits) when resuming.

**During:**
- Keep the main line green. Commit small, coherent steps with messages in the ubiquitous language.
- Write the test and the rule entry with the code, not afterwards.
- If you discover work: add a coarse backlog item or a stub registry entry; do not silently expand scope.
- If you hit a decision that might be big: stop and ask (see below).

**End:** follow the checklist in `AGENTS.md` (HANDOFF and the epic file as in "Keeping the start-of-session reading small"). A partial delivery is acceptable if:
- the repo builds and `verify` is green,
- unfinished parts are explicit (stub registry, epic task checklist, HANDOFF "next steps"),
- nothing half-implemented is reachable in a way that could corrupt state (prefer returning a typed `NotImplemented` error behind the validator over partial behavior).

## Keeping the start-of-session reading small

Every session pays for what it reads before it works, and everything read stays in the context for every later step (user, 2026-10-04: the project docs had grown to about 50k tokens of start-up reading; housekeeping of 2026-10-24: the queues, the epic files and the answered questions had grown again). The rule: **what a session must read is short; everything else is found by grep or by the pointer a doc gives, and is never read whole.** History is not deleted: it is moved, verbatim, to an archive next to the file it came from (the code fingerprint ignores `docs/` and `*.md`, so moving text never forces a re-verify), and git keeps the rest.

| Tier | Files (size aim; `check-docs` enforces the HANDOFFs only) | Read |
|---|---|---|
| Always | `AGENTS.md` (10 KB), `docs/HANDOFF.md` (6 000 bytes enforced, aim 3 000), the lane's `HANDOFF.md` (10 000 enforced, aim 5 000) | at the start |
| By task | the lane's `QUEUE.md` (15 KB: open tasks and open inbox items), the epic file (35 KB: the open increment or milestone), the ADRs named by the index's "Read when", the "Open" section of `docs/OPEN-QUESTIONS.md` | the entry or section of the task |
| On demand | `TRAPS.md`, `OVERVIEW.md`, `*-log.md`, `*-design.md`, `*-archive.md`, `QUEUE-ARCHIVE.md`, `QUESTIONS-ANSWERED.md`, `docs/rules/`, `docs/formats/`, the surveys | grep an id; read the section a doc points to |

A file above its size aim is trimmed at the next session's end by moving its finished part to the archive (below), in the same pull request, not in a pull request of its own.

**Write each fact once, where it is read.**

| Fact | Where | Not in |
|---|---|---|
| The lane's position, next step, open pull requests, versions, next ids | the lane `HANDOFF.md`, "Current state" (rewritten each session) | the epic, the queue |
| What the session did | the lane `HANDOFF.md`, "Recent sessions": one `###` entry of about ten lines, newest first, at most three (`check-docs`); delete the oldest | the "Current state", the traps |
| A trap | the lane's `TRAPS.md` under its area; a trap of every lane: the topic doc (sandbox and sync: this file, "Cloud sandbox" step 6; gates: `05-testing.md`; sources: `04-rules-and-sources.md` §7) and, only if it breaks another lane, the global `docs/HANDOFF.md` | a HANDOFF entry, a second doc |
| A done task | one line in the epic's task list (date, sub-tasks, rule ids, versions); the full record (what was built, refinements, agent decisions marked "change freely", measurements) in `docs/epics/<epic>-log.md` | the queue, the HANDOFF |
| A closed increment or milestone | the epic's `<epic>-archive.md` (verbatim: scope, decisions, task lists, review lists), a table row and a pointer left in the epic | the epic file |
| A task waiting; discovered work someone must act on | the lane's `QUEUE.md`, "Tasks" or "Inbox" (another lane's inbox: straight into that lane's) | the HANDOFF |
| A notice nobody has to act on; a finished item | do not write it; or append it to `QUEUE-ARCHIVE.md` at once | the inbox |
| A question | `docs/OPEN-QUESTIONS.md` "Open", under the lane's block; answered: the decision into an ADR or the doc it changes, the entry **moved** to the top of the lane's `docs/lanes/<lane>/QUESTIONS-ANSWERED.md` (numbered `Q<n>`: `docs/adr/answered-questions.md`) | "Open" |
| What the lane has built (a code map by area) | `docs/lanes/<lane>/OVERVIEW.md` (adventure) or the epic log | the HANDOFF |

- **The global `docs/HANDOFF.md`** is the lane index plus the traps that cross lanes and a table saying where the other traps live. It has no session entries and no lane's state (user, 2026-10-11: rows every lane rewrote made most merge conflicts). `cargo xtask check-docs` fails above 6 000 bytes. It is a shared path.
- **A lane's `docs/lanes/<lane>/HANDOFF.md`** has: a short intro; `## Current state` (branch, the epic's position and next step, decisions waiting for the user, the versions and next ids the lane owns); `## Standing traps` (a pointer to `TRAPS.md`: the group names and what each covers); `## Recent sessions`. The client, server and map-client lanes also hold their autonomy contract there. `check-docs` fails above 10 000 bytes or three entries: shorten, do not raise the limit.
- **A lane's `TRAPS.md`** holds the lane's standing traps in groups by area, read for the area the task touches. Delete a trap when it stops applying and merge duplicates.
- **Epic files:** the open increment or milestone, the decisions that apply to the whole epic, the risks. A done task keeps one line; when an increment or milestone closes, the next session moves its scope, decisions and task list into `<epic>-archive.md` (E08, E16, E17 and E24 have one) and leaves a row in a "Closed increments" table. Decision numbers and ids stay as they were: "E08 decision 75" is found by `git grep -n 'decision 75' docs/epics/E08-archive.md`. Design notes are read by section and pruned as their tasks implement them.
- **`QUEUE.md`:** open tasks, then the inbox of items someone must act on. A finished task leaves it with the session that did it; a finished or purely informational inbox item moves to `QUEUE-ARCHIVE.md` in the same session (the lane that writes a notice for another lane writes it only when that lane has to act or decide).
- **`OPEN-QUESTIONS.md`:** only the "Open" section plus a table of where the answered entries are. A lane adds an open question at the end of its block and, when it is answered, moves it to the top of its own `QUESTIONS-ANSWERED.md`, so lanes working at once never edit the same lines. Lane folders are owned by their lanes, so this needs no shared change.
- **Moving text to an archive:** copy it verbatim (a script, not a retyping), keep the ids and the file path of the original, leave a pointer, and check that the two files together hold every line of the old one. Never delete a file that code or docs cite: the references would have to be repointed (and the check `check-exe-map` reads docs too).
- **Bookkeeping rides with the work.** A session's HANDOFF, epic, log, queue and question updates belong in the pull request of its task. Shared files a session must change (process docs, ADRs, `BACKLOG.md`, the global HANDOFF) go in one `shared/<topic>` pull request per session, not one per file; a closure or refinement that only moves docs needs no pull request of its own beyond that.
- **`verify --summary`** keeps the gate's output out of the context: one line per gate plus the end of the first failure; the full log goes to `<target>/verify/verify.log`.

## Lanes

Agreed with the user on 2026-10-06/07 (Phase 0). Several agents work in parallel, **one agent at a
time per lane**. A lane owns paths; everything else it may not change in its own pull requests.

**Ownership** (`lanes.toml`; the longest matching pattern decides):

- **battle:** `h3-battle`, `h3-heroes`, the battle part `h3-content-battle` and its pack adapter
  `h3-pack-battle` (since R01; the core `h3-content` and `h3-pack` are shared), `h3-cli` (its
  `src/ai/` the AI lane's), the formats/tools crates `h3-formats`, `h3-harvest`, `h3-experiment`
  and `docs/formats/` (until a formats/tools lane opens), packs, fixtures, experiments,
  `benches/battle/`, the BAT rules, the quirks ledger, the surveys, the epics E01–E04c and E23.
  `h3-heroes` moved to adventure when E04c closed (ADV-Q13).
- **adventure:** `h3-adventure`, `h3-game`, `h3-map` and `h3-pack-adventure` (E08; later the
  town), adventure-owned sub-paths of the battle lane's crates and folders (`h3-sim`'s and
  `h3-cli`'s `src/adventure/`, `fixtures/replays/adventure/`, the packs' `adventure.toml`; the
  longest match decides), `docs/formats/h3m.md`, the ADV, GAM and town rules, its own quirks
  ledger `docs/rules/quirks-adventure.md`, `benches/adventure/`, the epics E07–E10 and E12–E14.
- **ai** (since E05; the user, 2026-10-18, AI-Q1): `h3-sim` (its `src/adventure/` stays the
  adventure lane's until E13), `h3-ffi`, `h3-cli`'s `src/ai/`, the Python side `python/`,
  `benches/ai/`, the epics E05 and E06. The battle lane still changes rules that the encodings
  show: such a change (a new observation feature, a new action entry) is a `cross-lane` pull
  request from the battle lane, with the AI lane's version bumps in it.
- **server:** the new adapter crates `h3-protocol` and `h3-server`, `benches/server/`, E16.
- **client:** the battle client, E17: `clients/`, a separate Cargo workspace with its own
  lockfile and CI job (`clients/README.md`), but for the map-client lane's paths and the client
  workspace's common ones (below).
- **map-client** (since 2026-10-23; the user split the client lane at E17 M3's closure,
  CLT-Q18): the adventure map client, E24: its crates in the client workspace
  (`clients/crates/h3-map…`) and its data folder `clients/map/`. What it needs of the client
  lane's `h3-view` moves into a common crate (a `lanes.toml` change).
- **Later lanes:** formats/tools when needed. Opening a lane is a `lanes.toml` change, with the
  lane's `HANDOFF.md` and `QUEUE.md` in the same pull request (`check-docs`).
- **Shared** (a small, separate pull request that merges first): `h3-kernel`, `h3-contracts`,
  `Cargo.toml`, `arch.toml`, `lanes.toml`, `xtask`, `scripts/`, CI, the toolchain and lint
  configuration, ADRs, `AGENTS.md`, the numbered docs, `BACKLOG.md`, the global HANDOFF, the
  catalog's README and kernel rules. Creating a lane's new crate is a shared change (the
  workspace members and `arch.toml`).
- **Common** (any lane): `docs/OPEN-QUESTIONS.md` and `docs/stubs.toml` (lane ids and one block
  per lane keep concurrent additions apart: add to your lane's block, not at the end), and
  `Cargo.lock` (generated: a lane may add a dependency to its own crate; the full verify's `deny`
  gate checks it on the pull request, and a heavy dependency stops the line under the autonomy
  contract; user, 2026-10-07), `clients/Cargo.lock` (an engine crate the client builds that
  changes its dependencies updates it too; the verify's `clients-lock` gate; user, 2026-10-14),
  the survey index `docs/rules/surveys/README.md` (one section
  per epic), **the client foundation and the capture harness** (the user, 2026-10-23, CLT-Q18:
  `h3-assets`, `h3-render`, `h3-sdl`, the client workspace's `Cargo.toml`, `deny.toml`,
  `.gitignore`, `README.md`, `scripts/`, the string catalogue `clients/text/`, and
  `clients/capture/`: a client lane changes them in its own pull request, an additive change
  freely, a change in behaviour noted in the other client lane's inbox; `clients/scripts/check.sh`
  runs every client test, a harness change its check; the adventure lane changes the harness
  without the `cross-lane` label), and **every lane's `QUEUE.md`, for its inbox only** (user, 2026-10-19): a lane that
  finds something another lane should learn about or do adds it as a dated inbox entry to that
  lane's queue itself, naming the sender (`2026-10-19 (the client lane, E17 M3 T4b-1): …`), in
  its own pull request; it never edits that queue's tasks or other entries, and leaves nothing
  "to pass on" in its own queue or with the user. The receiving lane triages it like any inbox
  entry.

**Branches and pull requests.** A lane works on a branch `<lane>/<task>` per task or increment and
merges it at its checkpoint; a shared change goes on `shared/<topic>`. `cargo xtask check-lane`
(CI: `.github/workflows/ci.yml`, on every pull request) fails when the diff touches paths of
another lane, or a lane's and shared paths together. It takes the lane from `--lane`, the branch
name, or infers it from the diff. A deliberate cross-lane change (rare; the user agrees) carries
the label `cross-lane`. When a lane needs a shared change, it opens the `shared/` pull request
first, waits for its merge and rebases; a seam change (`h3-contracts`) stops the line in every lane
it touches.

**Ids.** New questions, quirks and stubs carry the lane code from `lanes.toml` (BAT, ADV, SRV,
CLT, MCL, AI) and count per lane: questions `ADV-Q1`, quirks `ADV-QUIRK-1` (in the lane's ledger), stubs
`ADV-STUB-1`. Existing ids (`Q55`, `Q-0012`, `STUB-0001`) stay. `check-rules` and `check-stubs`
accept both forms. **ADRs** are written as drafts in `docs/lanes/<lane>/adr-drafts/<slug>.md`
and get the next number when the user accepts them (a shared change moves them to `docs/adr/`).

**Versions.** Each version constant has one owning lane; the lane HANDOFF lists its versions. When
two lanes bump a shared version at once (a cross-lane or shared change), the second sees a textual
conflict on rebase and bumps again.

| Version | Where | Owner |
|---|---|---|
| battle state hash | `BATTLE_STATE_HASH_VERSION`, `h3-battle` | battle |
| core ruleset hash | `RULESET_HASH_VERSION`, `h3-content` | shared (with `h3-content`, R01) |
| battle ruleset-part hash | `BATTLE_RULESET_HASH_VERSION`, `h3-content-battle` | battle |
| pack format (`core.toml`, `battle.toml`) | `packs/README.md` | shared (with `h3-pack`; the battle file with `h3-pack-battle`) |
| replay format, action and observation encodings | `h3-sim` | ai (battle until E05) |
| C ABI | `H3_ABI_VERSION`, `h3-ffi` | ai (battle until E05) |
| kernel RNG and hash | `RNG_VERSION`, `HASH_VERSION`, `h3-kernel` | shared |
| game state hash | `GAME_STATE_HASH_VERSION`, `h3-game` | adventure |
| adventure state hash, adventure ruleset-part hash | `ADVENTURE_STATE_HASH_VERSION`, `ADVENTURE_RULESET_HASH_VERSION`, `h3-adventure` | adventure |
| map import | `MAP_IMPORT_VERSION`, `h3-map` (a replay names its map by content hash, path and this version; ADR-0015) | adventure |
| adventure replay format | `h3-sim/src/adventure/replay.rs`: `adventure::replay::REPLAY_FORMAT_VERSION` (E08 T9b; the file format in `h3-cli`'s `src/adventure/`) | adventure |
| adventure action and observation encodings | `h3-sim/src/adventure/`: `adventure::ACTION_ENCODING_VERSION` (E08 T9a); no observation encoding before E13 | ai (the adventure lane's until E08 T45, ADR-0017 point 8) |
| wire protocol | `PROTOCOL_VERSION`, `h3-protocol` | server |
| viewer record format | `RECORD_FORMAT_VERSION`, `h3-protocol` (from E16 M1) | server |
| game journal format | `JOURNAL_FORMAT_VERSION`, `h3-server` (from E16 M2) | server |

**Queues and long runs.** Each lane has a `QUEUE.md`: its pre-refined tasks in order, each about
half a context of work, then an **Inbox** of discovered work (one line each, until a refinement
turns it into tasks or a backlog item). A task has **Scope** (what to build or decide, with
pointers: epic task, ADR, rule ids), **Acceptance** (what must hold when it is done; always the
Definition of Done in `05-testing.md`), **Depends on** (tasks, questions or user input that must
come first) and **Status**: `ready`, `waiting` (for the user, with what), `blocked` (with the
question id) or `in progress` (branch, UTC start time). Done tasks leave the queue; the epic file
and its log keep the record. A context fills in one to two hours, so a long unattended run is a
chain of fresh contexts: a thin *runner* session takes the first ready task, hands it to a fresh
*worker* sub-agent (brief: the task entry, the lane, its reading list, `AGENTS.md`) and keeps only
the worker's short structured report (result: done, blocked or failed; branch and commits; the
scoped verify's summary line; question ids; inbox lines; at most five lines for the HANDOFF). The
runner updates `QUEUE.md` and the lane HANDOFF from it and takes the next task. A blocked task
records its question and stays in the queue; the runner takes the next independent task. A
scheduled task restarts the runner as a safety net; it resumes from `QUEUE.md` and the HANDOFF.
**One runner per lane:** before the worker starts, the runner sets the task's status to
`in progress (<branch>, <UTC time>)`, commits and pushes that on the lane branch; a runner that
finds such a mark younger than three hours stops (the previous runner may still be alive), an
older one resumes from that branch. Less context per task: `verify --summary`, the per-lane
reading lists, the full verify left to CI (while CI cannot run, the runner runs it before the pull
request: `05-testing.md`, "CI").

**Local loop and gates.** `cargo xtask verify --scope <lane>` lints, tests and covers the lane's
crates, runs the lane's own gates (golden replays when it owns `h3-cli` or replays of its own
under `fixtures/replays/`, the C ABI smoke test, its benchmark workloads) and `check-lane`; it never counts as green (exit 2 with no failing step is its normal result). On
a `shared/` or cross-lane branch its `check-lane` step fails by design: run the full verify there.
CI runs the full verify on every pull request whose code it has not verified yet (`05-testing.md`,
"CI"). Each benchmark workload has its own baseline file,
`benches/<lane>/<workload>.toml`, so each lane's +2 % gate is its own (`benches/README.md`).

**Checkpoints.** The domain lanes (battle, adventure) close increments at a checkpoint with the
user, as before. The client lanes (client, map-client) and the server lane have a checkpoint per milestone,
under an autonomy contract in their HANDOFF with three tiers: *agent decides* (marked provisional), *queue for the
milestone review*, *stop the line* (seams and contracts, heavy dependencies, licence, anything
touching another lane). A fresh reviewer agent checks each milestone before the user does.

## Branches, pull requests and syncing

GitHub (`ChristopherRotter/homm3`) holds the durable copy, and **pull requests are how work
reaches `master`** (Q3, Q49): no bundles or file copies between machines. A lane branch per task
or increment (`<lane>/<task>`, "Lanes"), pushed after every commit (`AGENTS.md`, "Interruptions
and resuming") and merged at its checkpoint once CI is green, or, while CI cannot run (minutes or
billing), once the full local verify is green and recorded in the pull request (`05-testing.md`,
"CI"; CI never blocks work). **A pull request is opened only when
its branch is ready for merging** (the task or increment done, the scoped verify without a failing
step, the lane docs updated; user, 2026-10-08): pushing a branch runs no CI, while every push to an
open pull request does, and Actions minutes are scarce (`05-testing.md`, "CI"). An open pull
request takes only review fixes and the user's changes. A merge of `master` into an open pull
request (a conflict) waits until `master`'s latest run has finished: until then no run has
verified that code, and the pull request's run cannot skip it (user, 2026-10-11). **An open pull
request stays its author's to keep mergeable** (user, 2026-10-13): after opening it, the session
watches it until it merges or closes. A cloud session subscribes to the pull request's activity
(conflict notices, reviews, CI); every session checks its mergeability against `master`
(`git fetch origin master`, a trial merge) at each later step and before it ends. A conflict is
merged away (merge `master`, resolve, verify, push; the wait for `master`'s run above still holds)
without waiting to be told; a version both sides bumped (`PROTOCOL_VERSION`, a format version)
takes the next free number. A session that ends with its pull request open names it in the newest
entry of the lane HANDOFF, and the lane's next session checks it first. The user's local clone catches up with a
fetch; if the user pushed to the branch in between, their commits win: rebase onto them first.

## Cloud sandbox

Agents usually work in a cloud sandbox cloned from GitHub.

1. **Clone** the working branch: `git clone --depth 1 --branch <branch> <remote> homm3`
   (`git fetch origin master` before pushing, so the push sends a thin pack; `check-lane` and
   doc-drift need `master`). Such a clone tracks only the branch it cloned: a lane branch it
   creates and pushes gets no `origin/<branch>`, and the session's stop hook then reports the
   pushed commits as "unpushed … and no remote branch". Step 3's `sandbox-setup.sh` makes the
   clone track every branch name first, so this does not happen. If it does (a clone set up
   without it), nothing is lost and there is nothing to report to the user:
   `git config --replace-all remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'`, then
   `git fetch origin <branch>`; when in doubt, `git ls-remote origin refs/heads/<branch>` shows
   the pushed commit.
2. **Original data** (needed for code reading, the harvester and `verify --with-original-data`):
   when the session is linked to the user's computer, get read access to the user's repo folder
   (`D:\Users\Christopher\Source\Repos\homm3`) and stage `original-data/install/Heroes3.exe`,
   `original-data/install/Data/H3bitmap.lod`, `original-data/install/Data/H3ab_bmp.lod` and
   `original-data/extracted/H3bitmap/Objects.txt` (one large LOD per staging call); for the
   map reader also the shipped maps as one archive, `original-data/maps.tar` (the 160 maps of
   the installation's `Maps`; untar it into the staged `original-data/install` before step 3;
   2026-10-18: the four maps of the user's own it held before are gone); `extracted.tar` holds
   the text tables, `Objects.txt` among them (untar it into the staged `original-data`). Without
   the link, the user may upload them; otherwise the session only prepares (Q48).
3. **Setup:** `scripts/sandbox-setup.sh --lane <lane>` makes the clone track every branch name
   first, links the image's `stable` toolchain as the pinned one, copies the staged files into
   `original-data/` and starts the tool installs (the apt packages, the client lane's SDL build
   dependencies among them: `target/sandbox/apt.log` ends with `apt: installed`; cargo-deny's
   and cargo-llvm-cov's release binaries; the toolchain's `llvm-tools-preview`, whose
   `llvm-cov` and `llvm-profdata` `scripts/llvm-tools.sh` picks for the coverage gate before
   the apt package `llvm-20`; `capstone` and `pefile` for the code-reading tool
   `scripts/exe.py`, `docs/exe/README.md`) and the baseline in the background through
   `scripts/verify-once.sh`
   (`docs/05-testing.md`, "Each code state once"): the engine's gates, or the client lane's
   `clients/scripts/check.sh`, run in full only when no run has verified that code yet (then
   about 20 minutes on two CPUs; read the docs meanwhile, then check `target/sandbox/setup.log`
   and `tail -n 3` of the log it names). Later runs: `cargo xtask verify --scope <lane>
   --summary` for the lane's loop, or before a pull request
   `CARGO_INCREMENTAL=0 nohup scripts/verify-once.sh engine --base origin/master -- --with-original-data > target/sandbox/once.log 2>&1 &`
   (step 5). The coverage gate runs `cargo llvm-cov clean --workspace` before and after its
   run: test binaries of an earlier build (another branch, or a full run before a scoped one)
   entered the report as hundreds of misses at lines that do not match (E08, 2026-10-14; E04c
   T7: 224 in one file). A bare `cargo llvm-cov` outside the gate still needs that clean first.
4. Packs written by hand in a session without the original data are confirmed with
   `cargo xtask verify --with-original-data` before the merge (Q48).
5. **Disk.** The sandbox's writable disk is a fixed allowance per session (about 44 GB in
   E04c T7's): `df -h .` shows what is left as "Avail"; its size column misleads. A full verify
   built from nothing needs about 20 GB: `target/debug` about 6.5 GB, `target/llvm-cov-target`
   5 to 8 GB while the coverage gate runs (it removes the workspace's own instrumented crates
   afterwards, about half; the third-party ones stay), `clients/target` 1 GB, the release and
   native builds 0.3 GB. `target/debug` grows with every rebuild (stale artifacts: 11 GB late in
   a session), and incremental builds add about 8 GB more (`target/debug/incremental`); a `git
   worktree` with a target directory of its own costs another full build. When the allowance
   runs out, the linker dies with `signal 7 [Bus error]` or rustc reports `No space left on
   device` or `IO failure on output stream`; the steps fail wherever the build was (often
   coverage, then determinism and bench-check), and the agent's own tool output can be lost. So
   build with `CARGO_INCREMENTAL=0` throughout the session (it is part of cargo's artifact hash:
   switching it keeps a second copy of the workspace's crates), and before a full verify with
   less than about 20 GB left, delete `target/debug` and `target/llvm-cov-target` (rebuilt in
   about 15 minutes; deleting works
   even when writing fails). Stop a running verify by its processes' ids (`ps`), never
   `pkill -f`, whose pattern also matches the agent's own shell.

6. **Habits that save a session.**
   - Cloud sessions open pull requests through the REST API (`gh api repos/ChristopherRotter/homm3/pulls`, labels the same way): GraphQL is blocked, so `gh pr create` fails.
   - Git on the user's machine: delete permission for the repo folder first (even `git status` leaves `.git/index.lock`), `-c user.name -c user.email` with the user's GitHub no-reply address (Q59); no GitHub credentials there.
   - The VM can restart (files stay, jobs die): check `ps` before assuming a job still runs.
   - `cargo xtask verify` outlasts a tool call: run `scripts/sandbox-verify.sh` in the background (the coverage tools: `scripts/llvm-tools.sh`) and poll its log; the inner loop is `verify --scope <lane> --quick`.
   - The background verify tests whatever is in the clone: try ad-hoc tests in a `git worktree` with a `CARGO_TARGET_DIR` of its own under `target/` (never shared: `xtask` roots at its build), and remove it afterwards.
   - bench-check at 0 Ir: valgrind 3.22 found no symbols in the lld-linked `h3` (the gate links with `-z norelro`).

## Stub registry

Incomplete or deferred code is tracked in `docs/stubs.toml` (a common file; new ids are numbered
per lane, "Lanes"):

```toml
[[stub]]
id     = "BAT-STUB-1"
where  = "crates/h3-battle/src/spells.rs::cast"
reason = "Spells not part of E01"
epic   = "E03"
```

Code refers to the stub id in a comment (`// stub: BAT-STUB-1`; ids from before the lanes look
like `STUB-0001`). `xtask check-stubs` fails if logic code contains `todo!`, `unimplemented!`, `#[ignore]` or TODO markers without a registry entry, or if a registry entry has no matching code marker.

## When to interrupt and ask

See `AGENTS.md` for the list. Protocol:

1. Write the question in `OPEN-QUESTIONS.md` with a lane id (`ADV-Q1`): context, options, consequences of each, your recommendation, what is blocked.
2. If the user is present, ask interactively (short, concrete, with options). If not, finish the safe work, record the state in HANDOFF, and stop.
3. When answered: record the decision (ADR for architecture-level; note in the relevant doc otherwise), and move the entry to the top of its lane's `QUESTIONS-ANSWERED.md` ("Keeping the start-of-session reading small").

## ADRs

Use `docs/adr/0000-template.md`. One decision per ADR. Statuses: Proposed, Accepted, Superseded by ADR-NNNN. Do not edit an accepted ADR's decision silently; supersede it. A lane writes a new ADR as a draft in `docs/lanes/<lane>/adr-drafts/` (status Proposed, no number); it gets the next number when the user accepts it ("Lanes").

**Size:** ADRs hold decisions only (about 80 lines); working detail goes into the epic's design note (`docs/epics/<epic>-design.md`), which each task prunes as it implements it. Full rule: `docs/adr/README.md`, "Keep ADRs short".

## Branching and commits

Trunk-based: short-lived branches, small commits, `master` always green. Since 2026-10-06 the repo is on GitHub (`ChristopherRotter/homm3`); changes reach `master` through pull requests, and CI (`.github/workflows/ci.yml`, `05-testing.md`, "CI") must be green on them (while CI cannot run, the full local verify stands in); a pull request is opened only when its branch is ready for merging ("Branches, pull requests and syncing"). Branch names carry the lane (`battle/e04b-t5c`, `shared/<topic>`; "Lanes"). The PR body follows `docs/pull_request_template.md`. Conventional-style prefixes are welcome (`feat(battle): …`, `fix(game): …`, `docs: …`). Attribution trailers follow the session's configured instructions.

## Definition of Done

See `05-testing.md`.

## Backlog hygiene

- New discoveries → the lane's `QUEUE.md` inbox, one for another lane straight into that lane's inbox ("Lanes", common paths); cross-cutting ones (tooling, gates, kernel) → the inbox at the bottom of `BACKLOG.md` (a shared change).
- Completed epics are marked done with a one-paragraph summary and links to ADRs.
- Keep the backlog short and honest; delete items that stopped being relevant.
