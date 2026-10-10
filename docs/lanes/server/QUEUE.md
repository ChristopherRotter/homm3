# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 done 2026-10-10, ADR-0018). M1 = T1–T4 ends at a checkpoint with the user, after a fresh reviewer agent. Work under the autonomy contract (`HANDOFF.md`).

### E16 M1 T1 — Messages and codec

- **Scope:** the epic's M1 T1: the wire types (both directions, struct variants, serde's default enum form, no `tag`/`untagged`/`flatten`, `deny_unknown_fields`, integers only, 64-bit values as strings), the bounded codec (frame size checked before parsing, `serde_json` with `alloc`), the mapping module to and from the battle's commands, legal commands, events and observation (`E16-design.md` §2 and §4), `Unsupported` and `Other { kind }` with the wildcard arms as the lane's one coverage exclusion (queue it for the review), a test listing the mapped kinds; round-trip tests, a round-trip in `postcard` as a dev-dependency (provisional, for the review) so the binary option stays open, and the property that arbitrary bytes never panic.
- **Acceptance:** `h3-protocol` under the 100 % gate; `cargo xtask verify --scope server --summary` with no failing step; the review list in the epic file updated.
- **Depends on:** M0 (merged 2026-10-10).
- **Status:** in progress (claude/jolly-cori-sip168, 2026-10-10T15:04Z).

### E16 M1 T2 — Session core

- **Scope:** the epic's M1 T2: seats, tokens as opaque values the host supplies, join and rejoin, the authoritative loop (command, `apply` with the seat's side, `project` per viewer, per-viewer sequence numbers, typed acknowledgements), the bot hook, the record, the command log with periodic hashes kept in memory; scripted two-client tests; the observation-hygiene property over the wire (a hidden Quicksand placed by the opponent leaves the viewer's bytes unchanged); out-of-turn and wrong-seat commands leave state and log unchanged. No spectators yet (SRV-Q4).
- **Acceptance:** as T1; the security properties hold in tests.
- **Depends on:** T1.
- **Status:** waiting (T1).

### E16 M1 T3 — The server binary

- **Scope:** the epic's M1 T3: `tokio` connection tasks (socket, outbound queue, idle timer), an OS thread per game, bounded channels (a full outbound queue drops the connection), the WebSocket transport behind its trait, a TOML configuration (bind address, loopback by default; packs directory, scenario, profile, bot seats, limits), OS seed and tokens, the random bot from `h3-sim`, the replay written as the CLI's TOML at the end, a headless test client in the crate; end-to-end tests over loopback: two clients, a client against the bot, a reconnect, malformed and oversized input, flooding.
- **Acceptance:** as T1; the end-to-end tests pass; a server replay verifies with `h3 replay verify`.
- **Depends on:** T2.
- **Status:** waiting (T2).

### E16 M1 T4 — Closure

- **Scope:** the epic's M1 T4: the `benches/server/` workload (random play through the session core: decode, validate, project, encode) and its baseline; the `docs/06-security.md` additions (the queue's inbox item: the secret seed and the obstacle seed; a `shared/` pull request); the lane HANDOFF and the epic log; a fresh reviewer agent; the user's checkpoint with the review list; the cut of M2.
- **Acceptance:** M1's exit criteria (epic file).
- **Depends on:** T3.
- **Status:** waiting (T3).

## Inbox

- 2026-10-01 (E03 T9 review): information hiding for server mode (E16): the tools derive the public random obstacles and the hidden battle RNG from the same seed (different streams), so a low-entropy seed can be guessed from the layout. Server mode needs a secret, uniformly random seed (or an independent obstacle seed); write it into docs/06-security.md with the server. *Decided in ADR-0018, point 8 (a 64-bit OS seed, obstacles from a separate throwaway seed); the `06-security.md` text is M1 T4 (queued above).*
