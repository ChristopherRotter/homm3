# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

### Server refinement (E16)

- **Scope:** the refinement session with the user; the brief is in `HANDOFF.md` ("Brief for the refinement session"). Output: the protocol ADR draft (`docs/lanes/server/adr-drafts/`), the autonomy contract agreed, questions (`SRV-Q<n>`), the first milestone and its tasks in this queue.
- **Acceptance:** the user agreed the protocol ADR topics' direction, the autonomy contract and the first milestone; this queue holds pre-refined tasks for it.
- **Depends on:** Phase 0 merged.
- **Status:** waiting (the user's answers to SRV-Q1 to SRV-Q11). Drafts done on 2026-10-08: three ADR drafts in `adr-drafts/`, `docs/epics/E16-server.md` (milestones M0–M6, M1's tasks T1–T4, the autonomy contract) and `docs/epics/E16-design.md`. When the answers are in: correct the drafts, move M0 and M1's tasks into this queue as entries, and file SRV-Q6 with the adventure lane if agreed.

## Inbox

- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server. *Direction in the protocol ADR draft, point 8 (a 64-bit OS seed, obstacles from a separate throwaway seed); the `06-security.md` text is M1 T4.*
