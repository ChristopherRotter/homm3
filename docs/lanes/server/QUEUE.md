# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018). M1's checkpoint passed (SRV-Q12, 2026-10-11): M2 as cut below, each task about half a context (T1–T4 and T4b done 2026-10-11); `signal` is added for T5, the journal keeps the token itself (T2). Work under the autonomy contract (`HANDOFF.md`).

### E16 M2 T4c — The host's spectator setting

- **Scope:** the host (a created game's creator) decides whether its game takes spectators (the user, 2026-10-11; the epic's decision C: "a host picks the game configuration; seats and spectators; the configuration fixed at start"). Today only the operator decides, for every game at once (`[limits] spectators_per_game`, T4). Proposed shape (the agent's, queued for the M2 review): `CreateGame` gains `spectators: bool`; the host changes it while the game waits (`SetSpectators { seq, open }`, refused for another seat and once the game started, as the configuration is fixed at start); closing it drops the spectators already watching (a `Goodbye` reason of its own); `GameInfo` shows whether a game takes spectators, so a lobby does not offer a closed one; the setting journaled (`JOURNAL_FORMAT_VERSION` bumped), so a restored or resumed game keeps it; an offer may fix it (`[[offer]] spectators = false`, the operator's choice, also for the ladders' games, which have no host, M4); `PROTOCOL_VERSION` bumped.
- **Acceptance:** a closed game refuses a spectator (`NoSpectators`), also after a restart and a resume; only the host changes the setting, only before the start; the lobby's list shows it; the hygiene properties unchanged.
- **The user's answers (2026-10-11):** the host changes the setting only before the start (fixed once the game runs); a new game takes spectators unless its creator says otherwise (the wire field optional, open by default).
- **Depends on:** T4 (done 2026-10-11).
- **Status:** ready.

### E16 M2 T5 — Shutdown, drain, hardening; closure

- **Scope:** graceful shutdown on a signal (`tokio`'s `signal` feature, SRV-Q12 (c), the user: ADR-0018 point 3 amended in a `shared/` change first) and a drain mode that refuses new games (it may pause the running ones, T3); a deadline for a game nobody plays: a paused game after `[limits] pause_hours` and a running one whose seats stay empty, both still counted against the game limits (T3: `E16-log.md`); the rate limits hardened (per address across its connections; a separate limit for automated seats, T3's note); from the M1 review: memory bounded in bytes (a viewer's record, which `FetchRecord` sends whole and which grows with every query and rejection; the outbound queue; one `FetchRecord` in flight), a short deadline for the hello and the join (a connection that never joins holds its place until its idle timeout), WebSocket pings counted in the rate; the fuzz harness on the decoder (`cargo-fuzz`, `05-testing.md`); the replay download of finished games; the client's content check (`E16-design.md` §3) handed to the client lane (an inbox line in its `QUEUE.md`); the workload updated if the session core changed; the reviewer agent; the user's M2 checkpoint.
- **Acceptance:** M2's exit (epic file, to be written at the checkpoint).
- **Depends on:** T1–T4b (done), T4c.
- **Status:** ready once T4c is done.

## Inbox

- 2026-10-11 (M2 T1): a connection whose join a game refused stays bound to it, and keeps a waiting game listed until it leaves or idles out; unbind on a refusal (the game would tell the task) with T5's hardening.
- 2026-10-11 (M2 T2): after a restore a viewer's numbers and record start afresh (they are not journaled), so a record fetched after a restart begins at the re-take; keep the records across a restart (`server-data/records/`, design note §9) or document it for the client lane, with T5.
- 2026-10-11 (M2 T2): the ids of games that ended or went void may be handed out again after a restart (the lobby continues after the restored games only); a client that re-joins an old id with its token gets `BadToken`. Persist the last id (a line in the data directory) if the ladders (M4) need ids unique for a server's life.
- 2026-10-11 (M2 T3): `GameEnded` goes out before the verification, so a player of a game that turns out void learns it only from `FetchReplay` (`Unverified`); when results count (M4), verify before the end is told, or tell a void game's players.
