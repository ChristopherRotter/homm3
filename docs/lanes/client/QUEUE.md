# Client lane — QUEUE

Pre-refined tasks of the client lane, in order (`docs/08-work-process.md`, "Lanes"). A runner hands the first ready task to a fresh worker and keeps only the worker's short report; a worker reads AGENTS.md, the global and lane HANDOFF, the lane's reading list and its one task. A task is about half a context of work and has:

- **Scope:** what to build or decide, with pointers (epic task, ADR, rule ids).
- **Acceptance:** what must hold when it is done (always: the Definition of Done in `docs/05-testing.md`).
- **Depends on:** tasks, questions or user input that must come first.
- **Status:** `ready`, `waiting` (for the user, with what), `blocked` (with the question id; the runner takes the next independent task) or `in progress` (branch).

Done tasks leave the queue (the epic file and its log keep the record). **Inbox:** discovered work, one line each, until a refinement turns it into tasks or a backlog item.

## Tasks

### Asset-notes session

- **Scope:** the user dictates their knowledge of the game's assets and their traps (formats, palettes, animation groups, fonts, sound, anything they know to go wrong); the agent records it in `HANDOFF.md` ("Asset notes") without judging it yet.
- **Acceptance:** the user confirms the notes are complete enough to start the client refinement.
- **Depends on:** Phase 0 merged.
- **Status:** ready (a session with the user).

### Client refinement (E17)

- **Scope:** the refinement session with the user, from the asset notes and the agreed points in `HANDOFF.md`: the client ADR draft (language confirmed or argued against, the renderer, the `clients/` workspace layout), the walking skeleton (a battle replay viewer on the real assets), the autonomy contract, the first milestone and its tasks in this queue.
- **Acceptance:** the user agreed the ADR draft's direction, the autonomy contract and the first milestone.
- **Depends on:** the asset-notes session. Implementation waits until the battle actions are mostly stable (E04b); the user may start the asset and render-to-PNG layer earlier.
- **Status:** waiting (asset notes).

## Inbox

(empty)
