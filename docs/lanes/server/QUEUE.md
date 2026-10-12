# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, SRV-Q16; M3 done 2026-10-12, its checkpoint passed, SRV-Q18). M4 refined 2026-10-12 (SRV-Q19): ADR-0019 points 4–9 decided; the working detail in `E16-design.md` §11; cut below, each task about half a context, the inbox's M4 items folded in; T1 done 2026-10-12 (`[[ladder]]`, `rating`, the results log); T2 done 2026-10-12 (the queues, the match's game, wire 14; `E16-log.md`); T3a done 2026-10-12 (no host, the grace, forfeits and no-shows, journal format 5, wire 15); T3b done 2026-10-12 (verified before the end, rated and logged, `Rating`, `<data>/next-game`, wire 16). SRV-Q20 done 2026-10-12 (server-side pings; the user); T4 done 2026-10-12 (leaderboards, wire 17); T5 done 2026-10-12 (bots on ladders, wire 18); T6 done 2026-10-12 (the pause by agreement, wire 19, journal format 6); T5b added 2026-10-12 (SRV-Q21). Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`, each journal change `JOURNAL_FORMAT_VERSION`.

### E16 M4 T5b — A player's bot opt-out (SRV-Q21)

- **Scope:** `Queue { seq, ladder, bots }` (`bots: Option<bool>`, left out or `None`: bots welcome; *provisional*): a player who queued with `bots: false` is never matched with a bot, whatever its wait; `MatchFound` names whether the opponent is a bot (`bot: bool`), so a player who did not opt out may decline knowingly. No stored preference (the client sends it with each `Queue`). The per-ladder choice stays as built (no `[[ladder.bot]]`, no bots).
- **Acceptance:** the Definition of Done; the queue's tests (an opted-out player waits on past `bot_after_ms`, a player who did not is matched with the bot and told so); end to end: `MatchFound { bot: true }` for a bot match, an opted-out lone player never matched.
- **Depends on:** T5 (done).
- **Status:** ready.

### E16 M4 T7 — M4's closure

- **Scope:** the end-to-end test of the exit criteria (two clients queue and play a rated battle through a restart); `06-security.md`'s ladders and the server's pings (SRV-Q20; a shared change); the client lane's note (the ladder's messages, the accept step, reconnecting by account, the forfeit; an inbox line, cross-lane); M4's exit criteria checked; a fresh reviewer agent over M4; the user's checkpoint.
- **Acceptance:** M4's exit (epic file).
- **Depends on:** T1–T6 (done) and T5b.
- **Status:** waiting (T5b).

## Inbox

- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load. And a rename an old-key session sent before a re-key's commit can still land while its answer is off the task: refuse account requests whose connection's key is no longer the account's.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
