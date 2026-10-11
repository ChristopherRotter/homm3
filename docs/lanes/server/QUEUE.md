# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018). M1's checkpoint passed (SRV-Q12, 2026-10-11): M2 as cut below, each task about half a context (T1–T5a done 2026-10-11); `signal` is added for T5, the journal keeps the token itself (T2). Work under the autonomy contract (`HANDOFF.md`).

T5 as cut at M1's checkpoint was more than one task; split on 2026-10-11 (the agent, under the autonomy contract) into T5a–T5c, in this order.

### E16 M2 T5b — Hardening

- **Scope:** the rate limits hardened (per address across its connections; a separate limit for automated seats, T3's note); from the M1 review: memory bounded in bytes (a viewer's record, which `FetchRecord` sends whole and which grows with every query and rejection; the outbound queue; one `FetchRecord` in flight), a short deadline for the hello and the join (a connection that never joins holds its place until its idle timeout), WebSocket pings counted in the rate; the inbox's unbind on a refused join; the workload updated if the session core changed.
- **Acceptance:** each bound tested end to end or in the session core; the hygiene properties unchanged.
- **Depends on:** T5a (done 2026-10-11).
- **Status:** ready.

### E16 M2 T5c — The fuzz harness, the replay download; M2's closure

- **Scope:** the fuzz harness on the decoder (`cargo-fuzz`, `05-testing.md`); the replay download of finished games; the client's content check (`E16-design.md` §3) handed to the client lane (an inbox line in its `QUEUE.md`); the inbox's open M2 items decided (kept for later or built); M2's exit criteria written; the reviewer agent; the user's M2 checkpoint.
- **Acceptance:** M2's exit (epic file, written in this task).
- **Depends on:** T5b.
- **Status:** ready once T5b is done.

## Inbox

- 2026-10-11 (M2 T1): a connection whose join a game refused stays bound to it, and keeps a waiting game listed until it leaves or idles out; unbind on a refusal (the game would tell the task) with T5's hardening.
- 2026-10-11 (M2 T2): after a restore a viewer's numbers and record start afresh (they are not journaled), so a record fetched after a restart begins at the re-take; keep the records across a restart (`server-data/records/`, design note §9) or document it for the client lane, with T5.
- 2026-10-11 (M2 T2): the ids of games that ended or went void may be handed out again after a restart (the lobby continues after the restored games only); a client that re-joins an old id with its token gets `BadToken`. Persist the last id (a line in the data directory) if the ladders (M4) need ids unique for a server's life.
- 2026-10-11 (M2 T3): `GameEnded` goes out before the verification, so a player of a game that turns out void learns it only from `FetchReplay` (`Unverified`); when results count (M4), verify before the end is told, or tell a void game's players.
