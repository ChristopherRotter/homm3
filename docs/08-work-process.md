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

Agents usually work in a cloud sandbox; the user's repo folder is the only durable copy
(`AGENTS.md`, "Interruptions and resuming"). The device tools are the session's tools for the
user's computer.

1. **On the user's machine:** get delete permission for the repo folder before any git command
   (even `git status` can leave `.git/index.lock` behind). Bundle the working branch and `master`
   (doc-drift needs `master`): `git bundle create "Claude outputs/sync.bundle" <branch> master`.
2. **Stage** the bundle and `original-data/install/Heroes3.exe`,
   `original-data/install/Data/H3bitmap.lod`, `original-data/install/Data/H3ab_bmp.lod` and
   `original-data/extracted/H3bitmap/Objects.txt` (one large LOD per staging call).
3. **In the sandbox:** `git clone -b <branch> <staged bundle> homm3`, then in it
   `git fetch <staged bundle> master:master` and `scripts/sandbox-setup.sh`. The script links
   the image's `stable` toolchain as the pinned one, copies the original data into
   `original-data/` and starts the tool installs and the baseline verify in the background
   (about 15 minutes; read the docs meanwhile, then check `tail -n 3 target/sandbox/verify.log`).
   Later verify runs: `nohup scripts/sandbox-verify.sh --with-original-data > /dev/null 2>&1 &`.
4. **After every commit:** bundle the new commits (`git bundle create <outputs>/sync-<n>.bundle
   <last synced>..<branch>`, a new name each time), write it into `Claude outputs/` on the
   user's machine, there `git -c gc.auto=0 fetch "Claude outputs/sync-<n>.bundle" <branch>` (a
   detached auto-gc would die with the device shell and leave `.git/gc.log.lock`) and
   `git merge --ff-only FETCH_HEAD`, delete the bundle, and check that `git status` is clean and
   no `.git/*.lock` is left. If the user committed in between, their repo wins: bring their
   commits into the sandbox first.
5. The sandbox clone's `origin` is a bundle; after a sync point it at the latest bundle
   (`git remote set-url origin <bundle> && git fetch origin`) so tools do not report unpushed
   commits.

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

## Branching and commits (provisional — OPEN-QUESTIONS Q3)

Trunk-based: short-lived branches, small commits, main always green. Conventional-style prefixes are welcome (`feat(battle): …`, `fix(game): …`, `docs: …`). Attribution trailers follow the session's configured instructions.

## Definition of Done

See `05-testing.md`.

## Backlog hygiene

- New discoveries → coarse items at the bottom or in the right place by priority.
- Completed epics are marked done with a one-paragraph summary and links to ADRs.
- Keep the backlog short and honest; delete items that stopped being relevant.
