# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, its checkpoint passed, SRV-Q16). M3 refined 2026-10-11 (SRV-Q17): ADR-0019, accepted at the refinement (points 1 and 2 for M3; the design note §10), cut below, each task about half a context; T1 done 2026-10-11 (the login on the wire, wire 12); T2a done 2026-10-11 (logins checked, the player log); T2b done 2026-10-11 (re-keys, renames, the preference, the operator's rebind); T3 done 2026-10-11 (seats bound to accounts, wire 13, journal format 4); T4a done 2026-10-11 (the command limit, a waiting game without a held seat leaves); T4b done 2026-10-11 (trusted proxies, limits by IPv6 /64, the operator's README); T5 done 2026-10-12 but the checkpoint (SRV-Q18). Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`.

(No task: M3 is closed but for the user's checkpoint, SRV-Q18; M4 is refined after it.)

## Inbox

- 2026-10-11 (M2 T2; **M4**): the ids of games that ended or went void may be handed out again after a restart (the lobby continues after the restored games only); a client that re-joins an old id with its token gets `BadToken`. Persist the last id (a line in the data directory) when the ladders' results log needs ids unique for a server's life. Replay ids are unique already (a random part).
- 2026-10-11 (M2 T3; **M4**): `GameEnded` goes out before the verification, so a player of a game that turns out void learns it only from `FetchReplay` (`Unverified`); when results count, verify before the end is told, or tell a void game's players.
- 2026-10-11 (M2 T5b; **M4**, with the bot accounts on ladders): a separate rate for automated seats (T3's note): an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (the user, SRV-Q13; **M4's refinement**): ranked games one at a time per player (confirm the reading), a pause only by agreement, its length negotiated within the ladder's bounds, the absent player forfeits at the due time.
- 2026-10-11 (M2 T5e's review; **M4**, with the pause by agreement): a seat that is not the host can repeat `Pause`, and each `PauseAsked` fills every viewer's record window: one ask per seat until the game moves on.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load. And a rename an old-key session sent before a re-key's commit can still land while its answer is off the task: refuse account requests whose connection's key is no longer the account's.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
