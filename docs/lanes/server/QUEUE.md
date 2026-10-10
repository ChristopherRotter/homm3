# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 T1 to T3 done 2026-10-10, ADR-0018). M1 = T1–T4 ends at a checkpoint with the user, after a fresh reviewer agent. Work under the autonomy contract (`HANDOFF.md`).

### E16 M1 T4 — Closure

- **Scope:** the epic's M1 T4: the `benches/server/` workload (random play through the session core: decode, validate, project, encode) and its baseline; the `docs/06-security.md` additions (the queue's inbox item: the secret seed and the obstacle seed; a `shared/` pull request, which also brings the global HANDOFF's server row up to date: it still says "T2 next"); the lane HANDOFF and the epic log; a fresh reviewer agent; the user's checkpoint with the review list; the cut of M2.
- **Acceptance:** M1's exit criteria (epic file).
- **Depends on:** T3 (done 2026-10-10: `h3-server` hosts the battle; the workload drives `h3_protocol::session` directly, the end-to-end tests are in `crates/h3-server/src/tests.rs`).
- **Status:** ready once T3 is merged.

## Inbox

- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server. *Decided in ADR-0018, point 8 (a 64-bit OS seed, obstacles from a separate throwaway seed); the `06-security.md` text is M1 T4 (queued above).*
