# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, SRV-Q16; M3 done 2026-10-12, its checkpoint passed, SRV-Q18). M4 refined 2026-10-12 (SRV-Q19): ADR-0019 points 4–9 decided; the working detail in `E16-design.md` §11; cut below, each task about half a context, the inbox's M4 items folded in. Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`, each journal change `JOURNAL_FORMAT_VERSION`.

### E16 M4 T1 — Ladders, ratings and the results log

- **Scope:** `h3-server` only, no wire change. `[[ladder]]` in the configuration (`E16-design.md` §11: `id`, `name`, `scenarios`, `profile`, `accept_ms`, `grace_ms`, `grace_total_ms`, `pauses`, `pause_ms`, `spectators`; the pool compiled at start, one ruleset hash for all its scenarios, ids unique among ladders). `rating`: Glicko-2 as Glickman's paper gives it, one period per game, a draw a half. `<data>/ladders/<id>.log`: `Season`, `Rated`, `NoShow` lines (only `Season` is written in T1; the others are read and checked, so T3b only appends), read back at start into per-season tables (rating, deviation, volatility, games, wins, losses, draws), a torn last line cut, any other bad line stops the start (the error names the line, never its text); a ruleset hash other than the last season's starts the next season (ratings and volatilities kept, deviations 350). The example configuration gains a ladder.
- **Acceptance:** the Definition of Done; Glickman's worked example (1500/200/0.06 against 1400/30, 1550/100, 1700/300: 1464.06/151.52/0.05999) and a draw's update; a table read back from `Rated` lines equals the one the updates built, bit for bit; a content change starts a season, deviations reset; a pool on two rulesets refused; a torn last line cut, a bad line refused.
- **Depends on:** the M4 refinement (SRV-Q19).
- **Status:** ready.

### E16 M4 T2 — The queue and the match

- **Scope:** `h3-protocol` `ladder` (sans-I/O, 100 %): an account's queue entry (one queue at a time; refused while it has an unfinished ranked game on that ladder, which the host tells it, and for an anonymous connection); the matcher (by rating, ties by queue order, the window 100 + 50 per 30 s of the shorter wait; *provisional*); the accept step (`accept_ms`; who did not accept leaves, the other returns with its waiting time); a closed connection leaves its queue. The messages: `ListLadders`/`Ladders`, `Queue`/`Queued`, `Dequeue`, `MatchFound`, `AcceptMatch`, `DeclineMatch`, `MatchReady { game, seat }`, their refusals; `GameInfo::ladder`. `h3-server`: the queues in the registry, the matcher's tick, the accepted match created as a game from a scenario of the pool, the sides at random, both seats bound to their accounts at creation (no token; the players `Join` as their seat). Until T3a the game runs as an unranked one with bound seats; until T3b nothing is rated (an `SRV-STUB` each). Wire 14.
- **Acceptance:** the Definition of Done; properties: the matcher pairs nobody twice, never an account with itself, never outside the window; end to end: two logged-in clients queue, accept, join and play; a decline and an expiry return the other; a second queue and an anonymous queue refused.
- **Depends on:** T1.
- **Status:** ready once T1 is done.

### E16 M4 T3a — A ranked game's seats and forfeits

- **Scope:** a ranked game has no host (`Pause` without agreement and `SetSpectators` refused; spectators by the ladder's setting); `empty_game_ms` and `paused_game_ms` do not apply. Absence: a seat with no connection beyond `grace_ms` in one stretch, or `grace_total_ms` in total, forfeits (a loss; both absent: void, a no-show each), a game not yet started counted from its creation; `Entry::Ranked`, `Entry::Absent(side, ms)`, `Entry::Forfeit(side)` (`JOURNAL_FORMAT_VERSION` 5); a restore keeps the totals and starts a running stretch afresh (the downtime never counts). A forfeit's result on the wire (`GameEnded`); the replay holds the commands up to it (check `h3 replay verify` with an unfinished battle).
- **Acceptance:** the Definition of Done; end to end: a player back within the grace re-takes the seat by account and the game goes on; beyond the grace, and beyond the total in several drops, a forfeit; both away: void; across a restart the totals hold.
- **Depends on:** T2.
- **Status:** ready once T2 is done.

### E16 M4 T3b — The rated end

- **Scope:** a ranked game's journal verified before `GameEnded` (the inbox's M2 item: a void game's players are told, `Unverified`); a verified result with both seats held by their accounts rated (T1's `rating`) and written to the ladder's results log (`Rated`; a no-show `NoShow`), then `Rating { ladder, before, after }` to its seats; the one-unfinished-game rule fed from the registry, restored games included; game ids unique for the server's life (`<data>/next-game`, the inbox's M2 item).
- **Acceptance:** the Definition of Done; end to end: a played and a forfeited game rated as Glicko-2 gives it; the ratings the same after a restart; an unverified game unrated; no game id handed out twice across a restart.
- **Depends on:** T3a.
- **Status:** ready once T3a is done.

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

- **Scope:** `ProposePause { seq, ms }` within the ladder's `pause_ms` and the seat's remaining `pauses`, one ask per seat until the game moves on (the inbox's M2 T5e item; for unranked games too); `PauseProposed { n, seat, ms }` to the other seat, `AnswerPause { seq, accept }`; agreed, the game pauses with its length in the journal; at its due time the grace runs (from the start for a server that was down); a resumed game goes on.
- **Acceptance:** the Definition of Done; end to end: a proposal declined, one accepted and resumed in time, one whose player is not back after the due time and the grace forfeits; a second ask before the game moved on refused.
- **Depends on:** T3a.
- **Status:** ready once T3a is done.

### E16 M4 T7 — M4's closure

- **Scope:** the end-to-end test of the exit criteria (two clients queue and play a rated battle through a restart); `06-security.md`'s ladders (a shared change); the client lane's note (the ladder's messages, the accept step, reconnecting by account, the forfeit; an inbox line, cross-lane); M4's exit criteria checked; a fresh reviewer agent over M4; the user's checkpoint.
- **Acceptance:** M4's exit (epic file).
- **Depends on:** T1–T6.
- **Status:** waiting (T1–T6).

## Inbox

- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load. And a rename an old-key session sent before a re-key's commit can still land while its answer is off the task: refuse account requests whose connection's key is no longer the account's.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
