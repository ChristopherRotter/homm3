# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018). The M1 checkpoint with the user comes first (SRV-Q12: the review list, this cut of M2, two stop-the-line items). The tasks below are the **proposed** cut of M2, each about half a context; the checkpoint confirms or changes them. Work under the autonomy contract (`HANDOFF.md`).

### E16 M2 T1 — Many games on one server

- **Scope:** the M2 configuration (sections, `E16-design.md` §9: `[server]`, `[limits]`, `[content]` and the game configurations a host may pick); the lobby messages `CreateGame { config }`, `ListGames`, `Games`, `GameCreated` (wire version bump); a sans-I/O lobby in `h3-protocol` (game ids, the open-game limits in total and per connection, the configuration shown before joining); a registry of games in `h3-server`, one OS thread each, `Join` routed by game id; the workload unchanged (one game at a time).
- **Acceptance:** two pairs of clients play two games at once over loopback; the lobby's refusals; the Definition of Done (`05-testing.md`).
- **Depends on:** the M1 checkpoint (SRV-Q12).
- **Status:** waiting (the user's M1 checkpoint).

### E16 M2 T2 — The write-ahead journal and restore

- **Scope:** `JOURNAL_FORMAT_VERSION` 1 and the journal file per game (`E16-design.md` §7): header, seed, setup, seats, one line per accepted command, the checkpoint hashes; each command line synced before its `Accepted` leaves (the session core hands the host the new journal lines with the frames, the host writes before it sends); restore at start (the header checked against the engine and content, the commands re-run, the games reopened with their seats empty); the seats' tokens across a restart (SRV-Q12 (d)), and an operator's copy of them, so that a seat whose `Joined` was lost can be re-taken (the M1 review); the replay as the finished journal.
- **Acceptance:** an end-to-end test stops the server mid-battle and restarts it; the game resumes with the same state hash and the replay verifies; a journal of another engine or content identity ends its game void.
- **Depends on:** T1, SRV-Q12 (d).
- **Status:** waiting (the user's M1 checkpoint).

### E16 M2 T3 — Verification, pause and resume

- **Scope:** every finished game re-run against its hashes before its replay is published (a mismatch marks it corrupt: void, the diagnosis kept, a log line); the ruleset hash recomputed from memory at the end; the shadow-instance option (off by default, `E16-design.md` §8); `Pause` and `Resume` for unranked games (the host or all seats), a paused game as a journal without an engine.
- **Acceptance:** a corrupted journal is caught (tests flip a command and a hash); pause, restart, resume.
- **Depends on:** T2.
- **Status:** waiting (the user's M1 checkpoint).

### E16 M2 T4 — Spectators

- **Scope:** `Seat::Spectator` joins with the battle's spectator viewer (SRV-Q4): its snapshot, its record, a spectator limit per game; the hygiene property extended to spectators.
- **Depends on:** the battle lane's spectator viewer for `observe` and `project` (battle `QUEUE.md` inbox, 2026-10-09); T1.
- **Status:** waiting (the battle lane's spectator viewer; the M1 checkpoint).

### E16 M2 T5 — Shutdown, drain, hardening; closure

- **Scope:** graceful shutdown (SRV-Q12 (c)) and a drain mode that refuses new games; the rate limits hardened (per address across its connections; a separate limit for automated seats, T3's note); from the M1 review: memory bounded in bytes (a viewer's record, which `FetchRecord` sends whole and which grows with every query and rejection; the outbound queue; one `FetchRecord` in flight), a short deadline for the hello and the join (a connection that never joins holds its place until its idle timeout), WebSocket pings counted in the rate; the fuzz harness on the decoder (`cargo-fuzz`, `05-testing.md`); the replay download of finished games; the client's content check (`E16-design.md` §3) handed to the client lane (an inbox line in its `QUEUE.md`); the workload updated if the session core changed; the reviewer agent; the user's M2 checkpoint.
- **Acceptance:** M2's exit (epic file, to be written at the checkpoint).
- **Depends on:** T1–T4, SRV-Q12 (c).
- **Status:** waiting (the user's M1 checkpoint).

## Inbox

(empty)
