# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0, M1 T1 and T2 done 2026-10-10, ADR-0018). M1 = T1–T4 ends at a checkpoint with the user, after a fresh reviewer agent. Work under the autonomy contract (`HANDOFF.md`).

### E16 M1 T3 — The server binary

- **Scope:** the epic's M1 T3: `tokio` connection tasks (socket, outbound queue, idle timer), an OS thread per game, bounded channels (a full outbound queue drops the connection), the WebSocket transport behind its trait, a TOML configuration (bind address, loopback by default; packs directory, scenario, profile, bot seats, limits), OS seed and tokens, the random bot from `h3-sim`, the replay written as the CLI's TOML at the end, a headless test client in the crate; end-to-end tests over loopback: two clients, a client against the bot, a reconnect, malformed and oversized input, flooding.
- **Acceptance:** as T1; the end-to-end tests pass; a server replay verifies with `h3 replay verify`.
- **Depends on:** T2 (done 2026-10-10: `connection::Connection` per socket, `session::Session` per game; how the host drives them: `docs/epics/E16-log.md`, T2, "For T3").
- **Status:** ready once T2 is merged.

### E16 M1 T4 — Closure

- **Scope:** the epic's M1 T4: the `benches/server/` workload (random play through the session core: decode, validate, project, encode) and its baseline; the `docs/06-security.md` additions (the queue's inbox item: the secret seed and the obstacle seed; a `shared/` pull request, which also brings the global HANDOFF's server row up to date: it still says "T2 next"); the lane HANDOFF and the epic log; a fresh reviewer agent; the user's checkpoint with the review list; the cut of M2.
- **Acceptance:** M1's exit criteria (epic file).
- **Depends on:** T3.
- **Status:** waiting (T3).

## Inbox

- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server. *Decided in ADR-0018, point 8 (a 64-bit OS seed, obstacles from a separate throwaway seed); the `06-security.md` text is M1 T4 (queued above).*
