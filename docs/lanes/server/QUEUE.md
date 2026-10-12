# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, SRV-Q16; M3 done 2026-10-12, its checkpoint passed, SRV-Q18). M4 refined 2026-10-12 (SRV-Q19): ADR-0019 points 4–9 decided; the working detail in `E16-design.md` §11; cut below, each task about half a context, the inbox's M4 items folded in; T1 done 2026-10-12 (`[[ladder]]`, `rating`, the results log); T2 done 2026-10-12 (the queues, the match's game, wire 14; `E16-log.md`); T3a done 2026-10-12 (no host, the grace, forfeits and no-shows, journal format 5, wire 15). Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`, each journal change `JOURNAL_FORMAT_VERSION`.

### E16 M4 T3b — The rated end

- **Scope:** a ranked game's journal verified before `GameEnded` (the inbox's M2 item: a void game's players are told, `Unverified`; today the session sends `GameEnded` at the end, and the forfeit's in `Session::elapse`, before the host's `finish` verifies); a verified result with both seats held by their accounts rated (T1's `rating`) and written to the ladder's results log (`Rated`, a forfeit's with `forfeit`; a no-show, `Session::forfeited` both, `NoShow`), then `Rating { ladder, before, after }` to its seats; game ids unique for the server's life (`<data>/next-game`, the inbox's M2 item). (The one-unfinished-game rule over restored games came with T3a: `Registry::reopen`.)
- **Acceptance:** the Definition of Done; end to end: a played and a forfeited game rated as Glicko-2 gives it; the ratings the same after a restart; an unverified game unrated; no game id handed out twice across a restart.
- **Depends on:** T3a (done: `Session::over` and `forfeited`, `Outcome::Forfeit` and `NoShow`, SRV-STUB-4 in `registry::report`).
- **Status:** ready.

### E16 M4 T4 — Leaderboards

- **Scope:** `Leaderboard { seq, ladder, season, page }` (the current season when left out) answered by pages of 50 (rank, display name, rating and deviation as whole numbers, games, wins, losses, draws, the bot flag; by rating, ties by login name; an account listed after its first game of the season); `Standing { seq, ladder }`, the asker's row. Bounded like the replay lists (rate, page size).
- **Acceptance:** the Definition of Done; end to end: pages after rated games, an earlier season's page, one's own standing; a display name changed shows.
- **Depends on:** T3b.
- **Status:** ready once T3b is done.

### E16 M4 T5 — Bots on ladders

- **Scope:** `[[ladder.bot]]` (`login`, `display`, which bot; *provisional*): bot accounts in the player log with no key (never log in), unique names as any account's; each a standing queue entry matched only with a player who waited `bot_after_ms` (60 s), exempt from the one-game rule, any number of games at once; flagged in `Ladders`, the lobby and the leaderboard. The server's bots send no messages, so the inbox's rate for automated seats stays for automated clients.
- **Acceptance:** the Definition of Done; end to end: a lone player is matched with the bot after the wait, the game rated, the bot's row flagged.
- **Depends on:** T3b.
- **Status:** ready once T3b is done.

### E16 M4 T6 — The pause by agreement

- **Scope:** `ProposePause { seq, ms }` within the ladder's `pause_ms` and the seat's remaining `pauses`, one ask per seat until the game moves on (the inbox's M2 T5e item; for unranked games too); `PauseProposed { n, seat, ms }` to the other seat, `AnswerPause { seq, accept }`; agreed, the game pauses with its length in the journal; at its due time the grace runs (from the start for a server that was down); a resumed game goes on. A ranked game's `Pause` is rejected `NotHost` since T3a and nothing pauses one yet: the registry's `paused_game_ms` sweep (`Registry::sweep`) must not void a paused ranked game, and the session's clock (`Session::elapse`, counting from the session's start) must count the grace from the due time.
- **Acceptance:** the Definition of Done; end to end: a proposal declined, one accepted and resumed in time, one whose player is not back after the due time and the grace forfeits; a second ask before the game moved on refused.
- **Depends on:** T3a (done).
- **Status:** ready.

### E16 M4 T7 — M4's closure

- **Scope:** the end-to-end test of the exit criteria (two clients queue and play a rated battle through a restart); `06-security.md`'s ladders (a shared change); the client lane's note (the ladder's messages, the accept step, reconnecting by account, the forfeit; an inbox line, cross-lane); M4's exit criteria checked; a fresh reviewer agent over M4; the user's checkpoint.
- **Acceptance:** M4's exit (epic file).
- **Depends on:** T1–T6.
- **Status:** waiting (T1–T6).

## Inbox

- 2026-10-12 (M4 T3a, the user; **SRV-Q20**, open): eager disconnect detection by server-side pings, wanted eventually; until then a silent drop's grace starts after the idle timeout, and a server-side outage can void ranked games or, with a total nearly spent, cost one a forfeit. Settle before T3b's ratings go live if possible.
- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load. And a rename an old-key session sent before a re-key's commit can still land while its answer is off the task: refuse account requests whose connection's key is no longer the account's.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
