# AGENTS.md — read this first

This repository is a **headless, deterministic, high-performance rules engine for Heroes of Might and Magic III** (RoE base game plus the official AB and SoD extensions), written in Rust. Its main purpose is to serve as a fast simulation backend for reinforcement-learning self-play; playable clients come later and live outside the core.

You are one of many agents working on this over time. No single session finishes the project. **Your job is to move the repo forward one coherent increment and leave it in a state the next agent can pick up.**

## Start-of-session checklist

1. Know your lane (`battle`, `adventure`, `ai`, `server`, `client`, `map-client`, or `shared` for a change to shared paths): the task, the branch (`<lane>/<task>`) or the user says which.
2. Read, in this order and nothing more until the task needs it: `docs/HANDOFF.md`, `docs/lanes/<lane>/HANDOFF.md` (state, next step, last sessions) and your task (the lane's `QUEUE.md` entry, or the epic task the user names). Before editing code of an area, read that area's group in `docs/lanes/<lane>/TRAPS.md`.
3. Read the "Open" section of `docs/OPEN-QUESTIONS.md`; answered entries are in the lane's `QUESTIONS-ANSWERED.md` (`docs/adr/answered-questions.md` for `Q<n>`): grep an id when a doc points to it.
4. Read your lane's reading list (below): the open part of the epic file (closed increments and milestones are in `<epic>-archive.md`; the log `<epic>-log.md` only where a task builds on an earlier one) and, from the ADR index (`docs/adr/README.md`, "Read when"), only the ADRs your task touches. ADRs are binding; do not silently deviate.
5. If the lane HANDOFF names an open pull request, check its mergeability with `master` first and merge a conflict away (`docs/08-work-process.md`, "Branches, pull requests and syncing").
6. Confirm the starting point is green **before** you change anything: `cargo xtask verify --scope <lane> --summary` with no failing step (a scoped run exits 2 by design; it is never "green"), and CI's latest run where CI runs (a run that never started for lack of minutes or billing does not count; `docs/05-testing.md`, "CI"). In a cloud sandbox `scripts/sandbox-setup.sh --lane <lane>` prepares it (`docs/08-work-process.md`, "Cloud sandbox").

**Reading discipline.** Never read a log, design note, archive, ledger, survey or long format doc whole: grep for the id or heading a doc points to and read that slice. The files read at every start stay short (sizes, and where each kind of fact lives: `docs/08-work-process.md`, "Keeping the start-of-session reading small").

## Non-negotiables

- **Game logic is pure.** Logic crates have no I/O, no clock, no threads, no globals, no ad-hoc floating point (floats only via the audited `kernel::num` surface, ADR-0003), no `HashMap`/`HashSet`, no sorts without a unique tie-breaker, no graphics/audio/i18n/asset concerns. Enforced by `cargo xtask check-arch` (ADR-0003, ADR-0005).
- **Deterministic.** Same ruleset + initial state + seed + command sequence ⇒ bit-identical state and events on every platform (ADR-0003).
- **Server-authoritative and validated.** Every command is validated against state and the *trusted* actor identity. Never trust payload-supplied ownership, prices or amounts. A rejected command must leave state untouched (ADR-0004, ADR-0008, `docs/06-security.md`).
- **Domain-driven.** Code follows the bounded contexts in `docs/03-domain.md`. Cross-context interaction goes only through the contract types. Use the ubiquitous language from the glossary.
- **Nothing gameplay-relevant is hard-coded** when it can be data (tiers 1–3 in ADR-0006). But **architecture and performance win** over configurability when they conflict; record such a call in an ADR.
- **Every game rule has a rule ID, a source, and a test.** Rule IDs live in `docs/rules/catalog/`. Sources are labelled (wiki pages with URL, original data, observation, VCMI behavior…). See `docs/04-rules-and-sources.md`.
- **Known original bugs** are handled per the quirks ledger and profiles (`docs/rules/quirks-ledger.md`). Exploits and state-corrupting bugs are always fixed.
- **100% coverage** on core crates, with an explicit, reviewed exclusion list. Plus property tests, determinism replays and the other gates in `docs/05-testing.md`.
- **License hygiene.** The project is **GPL-2.0-or-later** (ADR-0009). VCMI (GPL-2+) may be read and its code ported or adapted, with an origin comment and a `kind = "vcmi"` rule source; ported code must still obey our purity/determinism/validation rules. Dependencies must be GPLv2-compatible (`deny.toml`). Do not commit original game files, assets or flavor text (harvested gameplay data and names in `packs/` are the exception, Q13).
- **The original game installation is read-only.** The user's installation (on the user's machine at `D:\Heroes3\HoMM 3 Complete`, connected to sessions as an additional folder) must never be modified: do not edit, create, rename, move or delete anything in it, do not run tools that write into it, and never request delete permission for it. Copy what you need into the repo's `original-data/` directory, which is gitignored; never commit original files (ADR-0009; `check-arch` fails if anything under `original-data/` is tracked). The *harvested* packs under `packs/` (gameplay data and names, no flavor text) are committed by design (Q13).
- **No scope creep into clients, assets, graphics or sound** inside the core.

## When to stop and ask

Stop, write the question in `docs/OPEN-QUESTIONS.md` (context, options, your recommendation), and ask the user interactively if they are present, when you hit any of:

- a change to an ADR-level decision, a seam/contract type, or a public crate boundary,
- license, legal or IP doubts,
- conflicting sources for a rule with real gameplay impact (a code reading counts as one source: compare it with the wiki and VCMI before a rule rests on it, `docs/04-rules-and-sources.md` §2),
- an ambiguous quirk classification (is this bug harmless-and-accepted, or must it be fixed?),
- a dependency that is heavy, not GPLv2-compatible (see `deny.toml`), or unmaintained,
- anything irreversible or that would invalidate a lot of prior work,
- being unsure whether something is "big". When in serious doubt, ask.

If the user is not present, do the safe preparatory work, document the decision needed, and stop at that point rather than guessing.

## Lanes

Several agents work in parallel, **one agent at a time per lane** (`docs/08-work-process.md`, "Lanes": ownership, ids, versions, checkpoints, the runner/worker model; read it when you run a lane, not only a task). `lanes.toml` says which paths each lane owns; `cargo xtask check-lane` enforces it on every pull request.

- A lane's pull request changes only its own paths and the common ones (`docs/OPEN-QUESTIONS.md`, `docs/stubs.toml`, the lockfiles, `docs/exe/map.toml` and `structs.toml`, the client foundation and capture harness, and every lane's `QUEUE.md` for its inbox: something another lane must act on goes straight into that lane's inbox).
- **Shared paths** (`h3-kernel`, `h3-contracts`, `Cargo.toml`, `arch.toml`, `lanes.toml`, `xtask`, CI, ADRs, process docs, `BACKLOG.md`, the global HANDOFF) change only through a small, separate `shared/<topic>` pull request that merges first. A deliberate cross-lane change needs the user's consent and the PR label `cross-lane`.
- New questions, quirks and stubs carry the lane code (`ADV-Q1`, `ADV-QUIRK-1`, `ADV-STUB-1`); ADRs start as drafts in `docs/lanes/<lane>/adr-drafts/`. The client, map-client and server lanes work under the autonomy contract in their HANDOFF, the domain lanes with a user checkpoint per increment.

**Reading lists** (after the two HANDOFFs and your task; open the rest only when the task touches it):

| Lane | Read | When the task touches it |
|---|---|---|
| battle | the current epic file (the lane HANDOFF names it), ADR-0014; ADR-0027 while E23 runs | ADR-0003 (numbers), ADR-0012/0013, `docs/04-rules-and-sources.md`, the quirks ledger, `docs/exe/README.md` and `docs/formats/installation.md` §7 (code reading), `docs/experiments/README.md` |
| adventure | `docs/03-domain.md`, the adventure epic file once refined | `docs/02-architecture.md`, ADR-0003/0004/0005/0006, `docs/04-rules-and-sources.md`, `docs/formats/h3m.md`, `docs/exe/README.md` (code reading) |
| ai | `docs/07-simulation-and-ai.md`, the current epic file (`docs/epics/E06-battle-ai.md`), ADR-0026 | ADR-0025 (the env), ADR-0011, ADR-0014 point 9, ADR-0003 (determinism), `docs/06-security.md` (projections), ADR-0023 point 7, `h3-sim`'s encodings |
| server | `docs/06-security.md`, ADR-0004, ADR-0008, ADR-0018 | ADR-0011 (encodings), `docs/02-architecture.md`, `h3-sim`'s action and observation encodings |
| client | `clients/README.md`, ADR-0009, ADR-0020, the asset notes in the lane HANDOFF | `docs/formats/installation.md` (archives), `docs/02-architecture.md` |
| map-client | `clients/README.md`, ADR-0009, ADR-0020, the asset notes in the lane HANDOFF; the E24 epic file once refined | `docs/formats/h3m.md`, `docs/formats/installation.md` (archives), `docs/epics/E17-design.md` (the drawing's readings), `docs/exe/README.md` (code reading), `docs/02-architecture.md` |
| shared | `docs/02-architecture.md`, `docs/05-testing.md`, `lanes.toml` | the ADRs the change touches |

**Long unattended runs.** A thin *runner* session hands each task of the lane's `QUEUE.md` to a fresh *worker* sub-agent and keeps only its short report (`docs/08-work-process.md`, "Queues and long runs"). Workers run `cargo xtask verify --scope <lane> --summary`; the runner runs the full verify while CI cannot.

## Working rules

- Small, coherent commits; keep the main line green. Partial delivery is fine; a red or undocumented repo is not.
- **CI is limited and may be unavailable** (user, 2026-10-12). **It never blocks work:** the full local `cargo xtask verify` stands in, its result recorded in the pull request (`docs/05-testing.md`, "When CI cannot run").
- Incomplete features must be visible: use the stub registry convention in `docs/08-work-process.md` (no silent `todo!()`, no untracked `#[ignore]`).
- Prefer the smallest design that satisfies the current epic. Do not over-specify the future; do not paint the project into a corner either (the guardrails in ADR-0006).
- Keep documentation current in the same change as the code (architecture, rules catalog, quirks ledger, ADRs).
- **Token efficiency beats speed** (user directive, 2026-10-03). Completion speed is not a requirement; token use is. Prefer doing the work yourself, sequentially. Use sub-agents only for narrow, well-bounded jobs (one question, one table, a handful of files or pages) with a concise brief and a short, structured answer; never hand a sub-agent a broad survey whose context grows with every file or page it reads. One worker sub-agent per queued task is fine (the runner/worker model under "Lanes"): it replaces the runner's context instead of adding to it. Do not run several large sub-agents in parallel. Read big files in slices, and summarise into the repo's docs as you go instead of re-reading.
- **Docs stay small by moving, not deleting:** a finished item goes verbatim to the archive next to its file; when a file you touch is past its size aim, trim it in your pull request (`docs/08-work-process.md`, "Keeping the start-of-session reading small").

## Interruptions and resuming

The user may stop a session at any moment and resume later, possibly in a new one. Work so that an unplanned stop costs at most the step in progress:

- **GitHub and the user's repo folder are the durable copies;** a cloud sandbox is temporary. **Push after every commit** to your lane branch (pushing a branch runs no CI; pushing to an open pull request does).
- **Keep the HANDOFF resumable:** at every task boundary, and before anything long-running, refresh a short "in progress" note in the newest "Recent sessions" entry: done, next, half-done and where.
- **When the user says they are about to stop,** finish or cleanly back out of the step, commit, update the HANDOFF, sync, and confirm. On the user's machine get delete permission for the repo folder before running git, never leave a `.git/*.lock`, and start no git operation right before an announced stop.
- **When resuming,** read the HANDOFFs and compare GitHub (and the user's repo) with any sandbox copy. **Their commits win.** Re-create the sandbox as `docs/08-work-process.md` ("Cloud sandbox") says.

## End-of-session checklist

1. `cargo xtask verify --scope <lane> --summary` has no failing step locally (exit 2 is its normal result).
2. **Write each fact once** (the table in `docs/08-work-process.md`, "Keeping the start-of-session reading small"):
   - Lane `HANDOFF.md`: rewrite "Current state" (position, next step, open pull requests, versions, next ids) and add one short entry under "Recent sessions"; at most three, delete the oldest. Nothing else goes there.
   - A lasting trap → the lane's `TRAPS.md` under its area, or the topic doc (one that breaks another lane also into the global HANDOFF, a `shared/` change); never a session entry.
   - The epic file: a done task gets one line, its full record goes into the epic's log; when an increment or milestone closes, move its scope, decisions and task list to `<epic>-archive.md`.
   - The lane's `QUEUE.md`: a done task leaves it; new work goes into its "Inbox" only if someone has to act on it, an informational note to `QUEUE-ARCHIVE.md` or nowhere. `docs/BACKLOG.md` changes status only when an epic starts or ends, in a `shared/` pull request.
3. New questions → `docs/OPEN-QUESTIONS.md` with a lane id (an answered one moves to the lane's `QUESTIONS-ANSWERED.md`); new decisions → an ADR draft in the lane folder.
4. Commit and push. **Open the pull request only when the branch is ready for merging** (steps 1–3 done; user, 2026-10-08): every push to an open pull request runs CI, and Actions minutes are scarce. Use `docs/pull_request_template.md`; a deliberate cross-lane change gets its label `cross-lane` right away. CI must be green on it (or the failure is documented in the lane HANDOFF; while CI cannot run, the full local verify's result stands in); then push only review fixes and what the user asks for. **Watch it until it merges or closes** (user, 2026-10-13): subscribe to its activity where the session can, check its mergeability with `master` at every later step and before ending, merge a conflict away without being told, and name it in the lane HANDOFF if it is still open when the session ends. The shared-file changes of a session go in one `shared/<topic>` pull request.

## Map of the docs

`docs/README.md` indexes everything. Key files: vision (`01`), architecture (`02`), domain (`03`), rules & sources (`04`), testing (`05`), security (`06`), simulation & AI (`07`), work process (`08`), plus `BACKLOG.md`, `HANDOFF.md`, `lanes/` (per lane: `HANDOFF.md`, `TRAPS.md`, `QUEUE.md`, archives), `OPEN-QUESTIONS.md`, `epics/`, `adr/`, `rules/`; `lanes.toml` at the root.
