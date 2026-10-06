# 08 — Work process for agents

## Planning levels (just-in-time refinement)

1. **Vision / architecture / ADRs** — stable, changed deliberately (ADR).
2. **Backlog** (`BACKLOG.md`) — coarse epics in rough priority order. Cheap to reorder. One or two lines each until they are close.
3. **Refined epic** (`epics/Exx-*.md`) — only for the next one or two epics: goal, scope, non-goals, exit criteria, task list. Refine an epic when it becomes next, not before.
4. **Task** — sized to fit one focused agent session (a coherent change that can be verified). If it does not fit, split it or deliver partially (below).
5. **HANDOFF.md** — the living state between sessions, within a size budget ("Keeping the start-of-session reading small").

Do not over-specify. A refined epic is allowed to be wrong after first contact; update it.

## Session protocol

**Start:** follow the checklist in `AGENTS.md`.

**Interruptions:** a session can be stopped at any time and resumed later, possibly by a fresh agent. Follow "Interruptions and resuming" in `AGENTS.md`: sync every commit to the user's repo immediately, keep HANDOFF resumable at task boundaries, checkpoint when the user announces a stop, and rebuild from the user's repo when resuming.

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
the catalog, quirks in the ledger, decisions in ADRs, formats in `packs/README.md` and
`docs/formats/`, history in git.

- **HANDOFF.md** has four parts: a short intro; `## Current state` (branch, verify, the epic's
  position and next step, decisions waiting for the user, versions; rewritten each session);
  `## Standing traps` (grouped bullets that still apply: delete a trap when it stops applying,
  merge duplicates, never copy a trap into a session entry); `## Recent sessions` (one `###`
  entry per session, newest first, about ten lines: what was done with commit ids and pointers,
  user and agent decisions, new traps moved up into Standing traps). Keep at most three entries;
  delete the oldest, git history keeps it. `cargo xtask check-docs` (a verify step) fails above
  10 000 bytes or three entries: shorten, do not raise the limit.
- **Epic files:** a done task keeps one line in the task list (date, sub-tasks, rule ids,
  versions). Its full record (what was built, refinements, agent decisions marked "change
  freely", measurements) goes into the epic's log, `docs/epics/<epic>-log.md`, which is read only
  when a task builds on it. The design note is pruned as before (`docs/adr/README.md`).
- **OPEN-QUESTIONS.md:** sessions read the "Open" section; an answered entry only when a doc
  points to it.

## Cloud sandbox and syncing

Agents usually work in a cloud sandbox cloned from GitHub (`ChristopherRotter/homm3`); the
remote branch is the durable copy (`AGENTS.md`, "Interruptions and resuming"). Q49 (2026-10-06).

1. **Clone** the working branch: `git clone --depth 1 --branch <branch> <remote> homm3`
   (`git fetch origin master` before pushing, so the push sends a thin pack; doc-drift needs
   `master`).
2. **Original data** (needed for code reading, the harvester and `verify --with-original-data`):
   when the session is linked to the user's computer, get read access to the user's repo folder
   (`D:\Users\Christopher\Source\Repos\homm3`) and stage `original-data/install/Heroes3.exe`,
   `original-data/install/Data/H3bitmap.lod`, `original-data/install/Data/H3ab_bmp.lod` and
   `original-data/extracted/H3bitmap/Objects.txt` (one large LOD per staging call). Without the
   link, the user may upload them; otherwise the session only prepares (Q48).
3. **Setup:** `scripts/sandbox-setup.sh` links the image's `stable` toolchain as the pinned one,
   copies the staged files into `original-data/` and starts the tool installs and the baseline
   verify in the background (about 15 minutes; read the docs meanwhile, then check
   `tail -n 3 target/sandbox/verify.log`). Later runs:
   `nohup scripts/sandbox-verify.sh --with-original-data > /dev/null 2>&1 &`.
4. **After every commit:** push to the branch. The user's local clone catches up with a fetch;
   if the user pushed in between, their commits win: rebase onto them first.
5. Packs written by hand in a session without the original data are confirmed with
   `cargo xtask verify --with-original-data` before the merge (Q48).

## Stub registry

Incomplete or deferred code is tracked in `docs/stubs.toml`:

```toml
[[stub]]
id     = "STUB-0001"
where  = "crates/h3-battle/src/spells.rs::cast"
reason = "Spells not part of E01"
epic   = "E03"
```

Code refers to the stub id in a comment (`// stub: STUB-0001`). `xtask check-stubs` fails if logic code contains `todo!`, `unimplemented!`, `#[ignore]` or TODO markers without a registry entry, or if a registry entry has no matching code marker.

## When to interrupt and ask

See `AGENTS.md` for the list. Protocol:

1. Write the question in `OPEN-QUESTIONS.md`: context, options, consequences of each, your recommendation, what is blocked.
2. If the user is present, ask interactively (short, concrete, with options). If not, finish the safe work, record the state in HANDOFF, and stop.
3. When answered: record the decision (ADR for architecture-level; note in the relevant doc otherwise), remove/mark the question answered.

## ADRs

Use `docs/adr/0000-template.md`. One decision per ADR. Statuses: Proposed, Accepted, Superseded by ADR-NNNN. Do not edit an accepted ADR's decision silently; supersede it.

**Size:** ADRs hold decisions only (about 80 lines); working detail goes into the epic's design note (`docs/epics/<epic>-design.md`), which each task prunes as it implements it. Full rule: `docs/adr/README.md`, "Keep ADRs short".

## Branching and commits

Trunk-based: short-lived branches, small commits, `master` always green. Since 2026-10-06 the repo is on GitHub (`ChristopherRotter/homm3`); changes reach `master` through pull requests, and CI (`.github/workflows/ci.yml`, `05-testing.md`, "CI") must be green on them. The PR body follows `docs/pull_request_template.md`. Conventional-style prefixes are welcome (`feat(battle): …`, `fix(game): …`, `docs: …`). Attribution trailers follow the session's configured instructions.

## Definition of Done

See `05-testing.md`.

## Backlog hygiene

- New discoveries → coarse items at the bottom or in the right place by priority.
- Completed epics are marked done with a one-paragraph summary and links to ADRs.
- Keep the backlog short and honest; delete items that stopped being relevant.
