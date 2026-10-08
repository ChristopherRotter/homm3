# 08 — Work process for agents

## Planning levels (just-in-time refinement)

1. **Vision / architecture / ADRs** — stable, changed deliberately (ADR).
2. **Backlog** (`BACKLOG.md`) — coarse epics in rough priority order. Cheap to reorder. One or two lines each until they are close.
3. **Refined epic** (`epics/Exx-*.md`) — only for the next one or two epics: goal, scope, non-goals, exit criteria, task list. Refine an epic when it becomes next, not before.
4. **Task** — sized to fit one focused agent session (a coherent change that can be verified). If it does not fit, split it or deliver partially (below).
5. **HANDOFF** — the living state between sessions, within a size budget ("Keeping the start-of-session reading small"): the global `docs/HANDOFF.md` (lane index, traps of every lane) and one per lane, `docs/lanes/<lane>/HANDOFF.md`.
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

Every session pays for what it reads before it works, and everything read stays in the context for
every later step (user, 2026-10-04: the project docs had grown to about 50k tokens of start-up
reading). So the files every session reads stay short, and detail lives where it is used: rules in
the catalog, quirks in the ledgers, decisions in ADRs, formats in `packs/README.md` and
`docs/formats/`, history in git. Each lane has a reading list in `AGENTS.md` ("Lanes").

- **The global `docs/HANDOFF.md`** is the lane index plus the traps that apply to every lane
  (sandbox and sync, sources, coverage and lints). It has no session entries. `cargo xtask
  check-docs` fails above 6 000 bytes. It is a shared path.
- **A lane's `docs/lanes/<lane>/HANDOFF.md`** has four parts: a short intro; `## Current state`
  (branch, the epic's position and next step, decisions waiting for the user, the versions the
  lane owns; rewritten each session); `## Standing traps` (grouped bullets that still apply to the
  lane: delete a trap when it stops applying, merge duplicates, never copy a trap into a session
  entry); `## Recent sessions` (one `###` entry per session, newest first, about ten lines: what
  was done with commit ids and pointers, user and agent decisions, new traps moved up into
  Standing traps). Keep at most three entries; delete the oldest, git history keeps it.
  `cargo xtask check-docs` fails above 10 000 bytes or three entries per lane: shorten, do not
  raise the limit. The client and server lanes also hold their autonomy contract there.
- **Epic files:** a done task keeps one line in the task list (date, sub-tasks, rule ids,
  versions). Its full record (what was built, refinements, agent decisions marked "change
  freely", measurements) goes into the epic's log, `docs/epics/<epic>-log.md`, which is read only
  when a task builds on it. The design note is pruned as before (`docs/adr/README.md`).
- **OPEN-QUESTIONS.md:** sessions read the "Open" section; an answered entry only when a doc
  points to it.
- **`verify --summary`** keeps the gate's output out of the context: one line per gate plus the
  end of the first failure; the full log goes to `<target>/verify/verify.log`.

## Lanes

Agreed with the user on 2026-10-06/07 (Phase 0). Several agents work in parallel, **one agent at a
time per lane**. A lane owns paths; everything else it may not change in its own pull requests.

**Ownership** (`lanes.toml`; the longest matching pattern decides):

- **battle:** `h3-battle`, `h3-heroes`, `h3-content` and `h3-pack` (until further notice, user
  2026-10-07), `h3-sim`, `h3-ffi` and `h3-cli` (until an AI lane exists at E05), the formats/tools
  crates `h3-formats`, `h3-harvest`, `h3-experiment` and `docs/formats/` (until a formats/tools
  lane opens), packs, fixtures, experiments, `benches/battle/`, the BAT and HER rules, the quirks
  ledger, the surveys, the epics E01–E06. `h3-heroes` moves to adventure with a handover
  checkpoint when hero progression starts (E08–E10).
- **adventure:** the new `h3-adventure` (later the town), the ADV and town rules, its own quirks
  ledger `docs/rules/quirks-adventure.md`, `benches/adventure/`, the epics E07–E10 and E12–E14.
- **server:** the new adapter crates `h3-protocol` and `h3-server`, `benches/server/`, E16.
- **client:** `clients/`, a separate Cargo workspace with its own lockfile and CI job
  (`clients/README.md`), E17.
- **Later lanes:** formats/tools when needed, AI at E05. Opening a lane is a `lanes.toml` change.
- **Shared** (a small, separate pull request that merges first): `h3-kernel`, `h3-contracts`,
  `Cargo.toml`, `arch.toml`, `lanes.toml`, `xtask`, `scripts/`, CI, the toolchain and lint
  configuration, ADRs, `AGENTS.md`, the numbered docs, `BACKLOG.md`, the global HANDOFF, the
  catalog's README and kernel rules. Creating a lane's new crate is a shared change (the
  workspace members and `arch.toml`).
- **Common** (any lane): `docs/OPEN-QUESTIONS.md` and `docs/stubs.toml` (lane ids and one block
  per lane keep concurrent additions apart: add to your lane's block, not at the end), and
  `Cargo.lock` (generated: a lane may add a dependency to its own crate; the full verify's `deny`
  gate checks it on the pull request, and a heavy dependency stops the line under the autonomy
  contract; user, 2026-10-07).

**Branches and pull requests.** A lane works on a branch `<lane>/<task>` per task or increment and
merges it at its checkpoint; a shared change goes on `shared/<topic>`. `cargo xtask check-lane`
(CI: `.github/workflows/ci.yml`, on every pull request) fails when the diff touches paths of
another lane, or a lane's and shared paths together. It takes the lane from `--lane`, the branch
name, or infers it from the diff. A deliberate cross-lane change (rare; the user agrees) carries
the label `cross-lane`. When a lane needs a shared change, it opens the `shared/` pull request
first, waits for its merge and rebases; a seam change (`h3-contracts`) stops the line in every lane
it touches.

**Ids.** New questions, quirks and stubs carry the lane code from `lanes.toml` (BAT, ADV, SRV,
CLT) and count per lane: questions `ADV-Q1`, quirks `ADV-QUIRK-1` (in the lane's ledger), stubs
`ADV-STUB-1`. Existing ids (`Q55`, `Q-0012`, `STUB-0001`) stay. `check-rules` and `check-stubs`
accept both forms. **ADRs** are written as drafts in `docs/lanes/<lane>/adr-drafts/<slug>.md`
and get the next number when the user accepts them (a shared change moves them to `docs/adr/`).

**Versions.** Each version constant has one owning lane; the lane HANDOFF lists its versions. When
two lanes bump a shared version at once (a cross-lane or shared change), the second sees a textual
conflict on rebase and bumps again.

| Version | Where | Owner |
|---|---|---|
| battle state hash | `BATTLE_STATE_HASH_VERSION`, `h3-battle` | battle |
| ruleset hash | `RULESET_HASH_VERSION`, `h3-content` | battle (with `h3-content`) |
| pack format | `packs/README.md` | battle (with `h3-pack`) |
| replay format, action and observation encodings | `h3-sim` | battle (AI lane from E05) |
| C ABI | `H3_ABI_VERSION`, `h3-ffi` | battle (AI lane from E05) |
| kernel RNG and hash | `RNG_VERSION`, `HASH_VERSION`, `h3-kernel` | shared |

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
reading lists, the full verify left to CI.

**Local loop and gates.** `cargo xtask verify --scope <lane>` lints, tests and covers the lane's
crates, runs the lane's own gates (golden replays, the C ABI smoke test, its benchmark workloads)
and `check-lane`; it never counts as green (exit 2 with no failing step is its normal result). On
a `shared/` or cross-lane branch its `check-lane` step fails by design: run the full verify there.
CI runs the full verify on every pull request whose code it has not verified yet (`05-testing.md`,
"CI"). Each benchmark workload has its own baseline file,
`benches/<lane>/<workload>.toml`, so each lane's +2 % gate is its own (`benches/README.md`).

**Checkpoints.** The domain lanes (battle, adventure) close increments at a checkpoint with the
user, as before. The client and server lanes have a checkpoint per milestone, under an autonomy
contract in their HANDOFF with three tiers: *agent decides* (marked provisional), *queue for the
milestone review*, *stop the line* (seams and contracts, heavy dependencies, licence, anything
touching another lane). A fresh reviewer agent checks each milestone before the user does.

## Branches, pull requests and syncing

GitHub (`ChristopherRotter/homm3`) holds the durable copy, and **pull requests are how work
reaches `master`** (Q3, Q49): no bundles or file copies between machines. A lane branch per task
or increment (`<lane>/<task>`, "Lanes"), pushed after every commit (`AGENTS.md`, "Interruptions
and resuming") and merged at its checkpoint once CI is green. **A pull request is opened only when
its branch is ready for merging** (the task or increment done, the scoped verify without a failing
step, the lane docs updated; user, 2026-10-08): pushing a branch runs no CI, while every push to an
open pull request does, and Actions minutes are scarce (`05-testing.md`, "CI"). An open pull
request takes only review fixes and the user's changes. The user's local clone catches up with a
fetch; if the user pushed to the branch in between, their commits win: rebase onto them first.

## Cloud sandbox

Agents usually work in a cloud sandbox cloned from GitHub.

1. **Clone** the working branch: `git clone --depth 1 --branch <branch> <remote> homm3`
   (`git fetch origin master` before pushing, so the push sends a thin pack; `check-lane` and
   doc-drift need `master`).
2. **Original data** (needed for code reading, the harvester and `verify --with-original-data`):
   when the session is linked to the user's computer, get read access to the user's repo folder
   (`D:\Users\Christopher\Source\Repos\homm3`) and stage `original-data/install/Heroes3.exe`,
   `original-data/install/Data/H3bitmap.lod`, `original-data/install/Data/H3ab_bmp.lod` and
   `original-data/extracted/H3bitmap/Objects.txt` (one large LOD per staging call). Without the
   link, the user may upload them; otherwise the session only prepares (Q48).
3. **Setup:** `scripts/sandbox-setup.sh` links the image's `stable` toolchain as the pinned one,
   copies the staged files into `original-data/` and starts the tool installs and the baseline
   verify in the background (about 15 minutes; read the docs meanwhile, then check
   `tail -n 3 target/sandbox/verify.log`). Later runs: `cargo xtask verify --scope <lane>
   --summary` for the lane's loop, or the full
   `nohup scripts/sandbox-verify.sh --with-original-data > /dev/null 2>&1 &`.
4. Packs written by hand in a session without the original data are confirmed with
   `cargo xtask verify --with-original-data` before the merge (Q48).

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
3. When answered: record the decision (ADR for architecture-level; note in the relevant doc otherwise), remove/mark the question answered.

## ADRs

Use `docs/adr/0000-template.md`. One decision per ADR. Statuses: Proposed, Accepted, Superseded by ADR-NNNN. Do not edit an accepted ADR's decision silently; supersede it. A lane writes a new ADR as a draft in `docs/lanes/<lane>/adr-drafts/` (status Proposed, no number); it gets the next number when the user accepts it ("Lanes").

**Size:** ADRs hold decisions only (about 80 lines); working detail goes into the epic's design note (`docs/epics/<epic>-design.md`), which each task prunes as it implements it. Full rule: `docs/adr/README.md`, "Keep ADRs short".

## Branching and commits

Trunk-based: short-lived branches, small commits, `master` always green. Since 2026-10-06 the repo is on GitHub (`ChristopherRotter/homm3`); changes reach `master` through pull requests, and CI (`.github/workflows/ci.yml`, `05-testing.md`, "CI") must be green on them; a pull request is opened only when its branch is ready for merging ("Branches, pull requests and syncing"). Branch names carry the lane (`battle/e04b-t5c`, `shared/<topic>`; "Lanes"). The PR body follows `docs/pull_request_template.md`. Conventional-style prefixes are welcome (`feat(battle): …`, `fix(game): …`, `docs: …`). Attribution trailers follow the session's configured instructions.

## Definition of Done

See `05-testing.md`.

## Backlog hygiene

- New discoveries → the lane's `QUEUE.md` inbox; cross-cutting ones (tooling, gates, kernel) → the inbox at the bottom of `BACKLOG.md` (a shared change).
- Completed epics are marked done with a one-paragraph summary and links to ADRs.
- Keep the backlog short and honest; delete items that stopped being relevant.
