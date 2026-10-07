# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

### Server refinement (E16)

- **Scope:** the refinement session with the user; the brief is in `HANDOFF.md` ("Brief for the refinement session"). Output: the protocol ADR draft (`docs/lanes/server/adr-drafts/`), the autonomy contract agreed, questions (`SRV-Q<n>`), the first milestone and its tasks in this queue.
- **Acceptance:** the user agreed the protocol ADR topics' direction, the autonomy contract and the first milestone; this queue holds pre-refined tasks for it.
- **Depends on:** Phase 0 merged.
- **Status:** ready (a session with the user, not an unattended worker).

## Inbox

- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server.
