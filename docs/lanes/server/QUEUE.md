# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, T1–T5e). M2's checkpoint (SRV-Q16) decides the review list in the epic file and the inbox's placement below. Work under the autonomy contract (`HANDOFF.md`).

### E16 M3 refinement — accounts

- **Scope:** with the user: the identity and ladders ADR draft (`adr-drafts/identity-and-ladders.md`) decided for its M3 part (the password-derived key, the challenge and signature, the `Authenticator` seam, the player log, seats bound to accounts, the anonymous mode, the reverse-proxy setup); M3 cut into tasks of about half a context in this queue, with the inbox lines marked M3.
- **Acceptance:** the user agreed the ADR's M3 part and the cut.
- **Depends on:** the user's M2 checkpoint (SRV-Q16).
- **Status:** waiting (the M2 checkpoint).

## Inbox

Placed in M2 T5e (the user confirms at the M2 checkpoint, SRV-Q16); a viewer's numbers and record after a restart are documented for the client lane (its inbox), not kept across restarts.

- 2026-10-11 (M2 T2; **M4**): the ids of games that ended or went void may be handed out again after a restart (the lobby continues after the restored games only); a client that re-joins an old id with its token gets `BadToken`. Persist the last id (a line in the data directory) when the ladders' results log needs ids unique for a server's life. Replay ids are unique already (a random part).
- 2026-10-11 (M2 T3; **M4**): `GameEnded` goes out before the verification, so a player of a game that turns out void learns it only from `FetchReplay` (`Unverified`); when results count, verify before the end is told, or tell a void game's players.
- 2026-10-11 (M2 T5b; **with the bot accounts**, M3 or M4 as M3's refinement places them): a separate rate for automated seats (T3's note): before accounts no client can be known as an agent (an operator raises `messages_per_second` and `messages_per_address` meanwhile).
- 2026-10-11 (the user, SRV-Q13; **M4's refinement**): ranked games one at a time per player (confirm the reading), a pause only by agreement, its length negotiated within the ladder's bounds, the absent player forfeits at the due time.
- 2026-10-11 (M2 T5c, T5e's review; **M3**, before the server faces the internet): a battle has no round limit, so two players who play on without end grow the game's journal, its memory in the session and its replay without bound (the records and queues are bounded since T5c): a limit of commands per game, the game void beyond it (a wire reason, `[limits]`), sized so a replay fits `outbound_bytes`; M4 decides what it means for a ranked result. Until then an archived replay (`FetchArchived`) goes out beside the outbound queue's byte bound, and a seat's `FetchReplay` of a replay over the bound is dropped `TooSlow`. On loopback and the LAN (until M3) the players are known to the operator.
- 2026-10-11 (M2 T5e's review; **M3**): a waiting game has no deadline, and a bound spectator keeps one listed after its creator left (counted against the creator's address): a waiting game whose seats are all empty leaves, its spectators told, or a deadline for waiting games.
- 2026-10-11 (M2 T5e's review; **M3**, with the reverse-proxy setup): the per-address limits see the proxy's address for every client behind it, and an IPv6 host has many addresses: the forwarded address from a trusted proxy, limits per IPv6 prefix.
- 2026-10-11 (M2 T5e's review; **M4**, with the pause by agreement): a seat that is not the host can repeat `Pause`, and each `PauseAsked` fills every viewer's record window: one ask per seat until the game moves on.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
