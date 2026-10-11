# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018). M1's checkpoint passed (SRV-Q12, 2026-10-11): M2 as cut below, each task about half a context (T1–T5b done 2026-10-11); `signal` is added for T5, the journal keeps the token itself (T2). Work under the autonomy contract (`HANDOFF.md`).

T5 as cut at M1's checkpoint was more than one task; split on 2026-10-11 (the agent, under the autonomy contract) into T5a–T5c, and T5b again the same day into T5b (done) and T5c, the old T5c becoming T5d; in this order.

### E16 M2 T5c — Memory bounded in bytes

- **Scope** (from the M1 review, "major"): a viewer's record bounded in bytes (it grows with every numbered message, queries, rejections and re-takes' snapshots included, and `FetchRecord` sends it whole); the outbound queue bounded in bytes, not only in frames; one `FetchRecord` in flight per connection; the security doc's "Not covered yet" brought up to date; the workload re-measured (the session core changes).
- **Acceptance:** each bound tested in the session core or end to end (a client that queries or re-joins without end holds a bounded record; a slow reader of large frames is dropped by bytes); the hygiene properties unchanged; the workload's drift named.
- **Depends on:** T5b (done 2026-10-11).
- **Status:** ready.

### E16 M2 T5d — The fuzz harness, the replay download; M2's closure

- **Scope:** the fuzz harness on the decoder (`cargo-fuzz`, `05-testing.md`); the replay download of finished games; the client's content check (`E16-design.md` §3) handed to the client lane (an inbox line in its `QUEUE.md`); the inbox's open M2 items decided (kept for later or built); M2's exit criteria written; the reviewer agent; the user's M2 checkpoint.
- **Acceptance:** M2's exit (epic file, written in this task).
- **Depends on:** T5c.
- **Status:** ready once T5c is done.

## Inbox

- 2026-10-11 (M2 T2): after a restore a viewer's numbers and record start afresh (they are not journaled), so a record fetched after a restart begins at the re-take; keep the records across a restart (`server-data/records/`, design note §9) or document it for the client lane, with T5.
- 2026-10-11 (M2 T2): the ids of games that ended or went void may be handed out again after a restart (the lobby continues after the restored games only); a client that re-joins an old id with its token gets `BadToken`. Persist the last id (a line in the data directory) if the ladders (M4) need ids unique for a server's life.
- 2026-10-11 (M2 T3): `GameEnded` goes out before the verification, so a player of a game that turns out void learns it only from `FetchReplay` (`Unverified`); when results count (M4), verify before the end is told, or tell a void game's players.
- 2026-10-11 (M2 T5b): a separate rate for automated seats (T3's note) waits for M3's bot accounts: before accounts no client can be known as an agent (an operator raises `messages_per_second` and `messages_per_address` meanwhile).
- 2026-10-11 (the user, SRV-Q13): for M4's refinement: ranked games one at a time per player (confirm the reading), a pause only by agreement, its length negotiated within the ladder's bounds, the absent player forfeits at the due time.
