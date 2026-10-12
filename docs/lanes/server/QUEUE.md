# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, SRV-Q16; M3 done 2026-10-12, its checkpoint passed, SRV-Q18). M4 refined 2026-10-12 (SRV-Q19): ADR-0019 points 4–9 decided; the working detail in `E16-design.md` §11; cut below, each task about half a context, the inbox's M4 items folded in; T1 done 2026-10-12 (`[[ladder]]`, `rating`, the results log); T2 done 2026-10-12 (the queues, the match's game, wire 14; `E16-log.md`); T3a done 2026-10-12 (no host, the grace, forfeits and no-shows, journal format 5, wire 15); T3b done 2026-10-12 (verified before the end, rated and logged, `Rating`, `<data>/next-game`, wire 16). SRV-Q20 done 2026-10-12 (server-side pings; the user); T4 done 2026-10-12 (leaderboards, wire 17); T5 done 2026-10-12 (bots on ladders, wire 18); T6 done 2026-10-12 (the pause by agreement, wire 19, journal format 6); T5b added 2026-10-12 (SRV-Q21) and done the same day (a player's bot opt-out, wire 20); T7 done 2026-10-12 (M4's closure: the reviewer agent, the bar after a match given up; M4 done, its checkpoint passed, SRV-Q22); T8 added by the checkpoint. Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`, each journal change `JOURNAL_FORMAT_VERSION`.

M4's checkpoint passed 2026-10-12 (SRV-Q22): the review list as built; T8 first, then M6's refinement.

### E16 M4 T8 — An agreed pause's hold across a restart (SRV-Q22 (b))

- **Scope:** a seat taken during an agreed pause before its due time holds the other seats' absences until then (`Session::hold`), in memory only: the journal's `Resumed` ends the agreed pause, so a restart before the due time lets an absent player's grace run from the start and forfeit before the pause's due time (M4's review). Journal the hold (an `Entry` with the rest of the pause, `JOURNAL_FORMAT_VERSION` 7) and re-apply it at a restore; propose, *provisional*, when a journaled hold ends so that a later start does not re-apply one that ran out (re-applied whole at each start it would; the downtime never counts against a player). No wire change.
- **Acceptance:** the Definition of Done; the session's tests (a hold journaled at an early resume and re-applied at a restore; one that ended not re-applied); end to end: an agreed pause, one player back early, a restart before the due time, the other back before it plays on (no forfeit), and a player not back by it forfeits after the grace.
- **Depends on:** T6 (done).
- **Status:** ready.

### E16 M6 — Refinement (discovery)

- **Scope:** refine M6 (`docs/epics/E16-server.md`, "Milestones"): the server keypair and the signed welcome, the server card and `Describe`, peers in the configuration, the peer crawl and its bounds, `ListPeers`, signed lists from files, mDNS on the LAN; decide the discovery ADR draft (`docs/lanes/server/adr-drafts/`) with the user; cut the tasks into this queue; fold in the inbox's items that fit.
- **Acceptance:** the user's refinement checkpoint (an `SRV-Q`), the ADR accepted or amended, the tasks queued.
- **Depends on:** T8.
- **Status:** waiting (T8).

## Inbox

- 2026-10-12 (from the battle lane, by the user; **now: the Windows verify is red**): the user's `cargo xtask verify --with-original-data` on Windows (branch `battle/e04b-t11b` with that day's `master`; `h3-server` as on `master`) fails in `h3-server` only. (1) Clippy, with `-D warnings`: `unused variable: new` at `crates/h3-server/src/journal.rs:188` (`open_log`: `new` is read only under `#[cfg(unix)]`). (2) `cargo test -p h3-server --lib`: 97 passed, 2 failed, `ladders::tests::rated_games_are_read_back_bit_for_bit` and `players::tests::the_accounts_are_read_back_and_a_torn_line_is_dropped`; the coverage step fails on the same two. The battle session's reading, not run on Windows: `open_log` opens the log with `append(true)` and cuts a torn last line with `set_len` on that handle, which Windows refuses for an append-only handle (`Journal`'s own recovery cuts through a handle opened with `write(true)`). CI runs on Linux only, so nothing showed there.
- 2026-10-12 (M4's review; **later**): finished, unrated ranked journals that a start re-runs (only after a ladder log's write failed) are rated in the order their threads run, not the order the games ended; Glicko-2 depends on it. Re-run them one after another by id before the others start.
- 2026-10-12 (M4's review; **later**): a game that stops because its journal no longer writes (or whose thread panics) leaves the registry's ranked games while its journal stays: its players may start another match on the ladder, and the next start restores both. Keep the entry until the journal is gone.
- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load. And a rename an old-key session sent before a re-key's commit can still land while its answer is off the task: refuse account requests whose connection's key is no longer the account's.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
- 2026-10-11 (M2 T5e's review; **later**): the registry's `Drain`, `Stop` and `Tick` are `try_send`s, lost when a game's inbox (1 024) is full; a lost `Drain` leaves the drain waiting until the second signal.
