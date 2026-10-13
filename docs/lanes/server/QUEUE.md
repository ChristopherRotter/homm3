# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0 and M1 done 2026-10-10, ADR-0018; M2 done 2026-10-11, SRV-Q16; M3 done 2026-10-12, its checkpoint passed, SRV-Q18). M4 refined 2026-10-12 (SRV-Q19): ADR-0019 points 4–9 decided; the working detail in `E16-design.md` §11; cut below, each task about half a context, the inbox's M4 items folded in; T1 done 2026-10-12 (`[[ladder]]`, `rating`, the results log); T2 done 2026-10-12 (the queues, the match's game, wire 14; `E16-log.md`); T3a done 2026-10-12 (no host, the grace, forfeits and no-shows, journal format 5, wire 15); T3b done 2026-10-12 (verified before the end, rated and logged, `Rating`, `<data>/next-game`, wire 16). SRV-Q20 done 2026-10-12 (server-side pings; the user); T4 done 2026-10-12 (leaderboards, wire 17); T5 done 2026-10-12 (bots on ladders, wire 18); T6 done 2026-10-12 (the pause by agreement, wire 19, journal format 6); T5b added 2026-10-12 (SRV-Q21) and done the same day (a player's bot opt-out, wire 20); T7 done 2026-10-12 (M4's closure: the reviewer agent, the bar after a match given up; M4 done, its checkpoint passed, SRV-Q22); T8 added by the checkpoint and done 2026-10-12 (the hold of a pause resumed early across a restart, journal format 7). Work under the autonomy contract (`HANDOFF.md`). Each wire change bumps `PROTOCOL_VERSION`, each journal change `JOURNAL_FORMAT_VERSION`.

M4's checkpoint passed 2026-10-12 (SRV-Q22): the review list as built; T8 first (done), then M6's refinement. The two review items left for later (the rating order of re-run journals, a stopped match's ranked entry) and two older inbox items (a drain or stop a full inbox missed, a rename racing a re-key) done 2026-10-12 at the user's request (`E16-log.md`, "M4's review items"). The Windows failures the battle lane handed over (the logs' append mode) fixed 2026-10-12 (`E16-log.md`).

### E16 M6 — Refinement (discovery)

- **Scope:** refine M6 (`docs/epics/E16-server.md`, "Milestones"): the server keypair and the signed welcome, the server card and `Describe`, peers in the configuration, the peer crawl and its bounds, `ListPeers`, signed lists from files, mDNS on the LAN; decide the discovery ADR draft (`docs/lanes/server/adr-drafts/`) with the user; cut the tasks into this queue; fold in the inbox's items that fit.
- **Acceptance:** the user's refinement checkpoint (an `SRV-Q`), the ADR accepted or amended, the tasks queued.
- **Depends on:** T8 (done).
- **Status:** ready.

## Inbox

- 2026-10-13 (battle lane, the rear hex as a two-hex stack's position; **next session**): `PROTOCOL_VERSION` is 22. The user decided that a two-hex stack's position is its rear hex, as in the executable (BAT-GEO-002), so `Stack.hex`, `Command::Move`'s `to` and `Command::Attack`'s `from` name a two-hex stack's rear hex; `Shoot`'s `at` is still the target's front hex (or, for area shots, its rear hex). The battle lane changed only the three `wire.rs` comments and the version (no type changed; its pull request is `cross-lane`); the lane's HANDOFF still says 21 since R01. The games are the same (800 random battles compared), so `session-random-play-full`'s baseline holds; whether a journal or record written before needs its own format bump is the lane's call.
- 2026-10-12 (after T8; **later**): a ranked game over whose result did not write (its ladder's log failed) keeps its journal, but its players may play another match on the ladder before the next start rates it, after the newer game. Keep its players out of the ladder until then if logs ever fail in practice.
- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
