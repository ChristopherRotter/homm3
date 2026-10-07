# Server lane — QUEUE

Pre-refined tasks of the server lane, in order (`docs/08-work-process.md`, "Lanes"). A runner hands the first ready task to a fresh worker and keeps only the worker's short report; a worker reads AGENTS.md, the global and lane HANDOFF, the lane's reading list and its one task. A task is about half a context of work and has:

- **Scope:** what to build or decide, with pointers (epic task, ADR, rule ids).
- **Acceptance:** what must hold when it is done (always: the Definition of Done in `docs/05-testing.md`).
- **Depends on:** tasks, questions or user input that must come first.
- **Status:** `ready`, `waiting` (for the user, with what), `blocked` (with the question id; the runner takes the next independent task) or `in progress` (branch).

Done tasks leave the queue (the epic file and its log keep the record). **Inbox:** discovered work, one line each, until a refinement turns it into tasks or a backlog item.

## Tasks

### Server refinement (E16)

- **Scope:** the refinement session with the user; the brief is in `HANDOFF.md` ("Brief for the refinement session"). Output: the protocol ADR draft (`docs/lanes/server/adr-drafts/`), the autonomy contract agreed, questions (`SRV-Q<n>`), the first milestone and its tasks in this queue.
- **Acceptance:** the user agreed the protocol ADR topics' direction, the autonomy contract and the first milestone; this queue holds pre-refined tasks for it.
- **Depends on:** Phase 0 merged.
- **Status:** ready (a session with the user, not an unattended worker).

## Inbox

- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server.
