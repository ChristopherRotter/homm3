# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0–M4 done 2026-10-10 to 2026-10-12, each checkpoint passed: SRV-Q12, SRV-Q16, SRV-Q18, SRV-Q22; the done tasks in full in `E16-log.md`). Work under the autonomy contract (`HANDOFF.md`).

M4's checkpoint passed 2026-10-12 (SRV-Q22); T8 and M4's review items done the same day (`E16-log.md`). **M6 refined 2026-10-13 (SRV-Q23):** ADR-0021 accepted (in the refinement's pull request, a `shared:` commit, `cross-lane`); the working detail in `E16-design.md` §12; cut below, each task about half a context; T1 done 2026-10-13 (the server key and the signed welcome, wire 24), T2 the same day (the card and `Describe`, wire 25), T3 the same day (peers, the fetch with its TLS client, `ListPeers`, wire 26), T4 the same day (the chain beyond the peers, `[discovery] depth` and `cards`, no wire change), T5 the same day (lists of servers, `Lists`, the list tools, wire 27; `E16-log.md`), T6 the same day (the closure: `06-security.md` and the client lane's inbox, the reviewer's two major findings and four minor fixed; `E16-log.md`). Wire changes bump `PROTOCOL_VERSION` (29 now: E17's CLT-Q1, a walker's `Moved` hex by hex, the battle lane's `cross-lane` #169), journal changes `JOURNAL_FORMAT_VERSION` (8 since the rear hex, `E16-log.md`; M6's tasks change no journal).

**Waiting:** M6's checkpoint (SRV-Q24, the user's). No task is ready after it: M5 waits for E08's increment 3 (refined, not built), mDNS for the client lane's browser; CLT-Q1, the wire included, is the battle lane's #169.

## Inbox

- 2026-10-13 (the battle lane, E04c T2b; its `cross-lane` pull request): protocol v28 maps the observation's special battlefield and anti-magic flag and lists the battlefields in the key tables (done there). The server draws a hosted scenario's obstacles with `generate_obstacles(terrain)` (`game.rs`); a scenario that names a battlefield needs `h3_battle::generate_setup_obstacles` (none does yet).
- 2026-10-13 (M6 T3; **later**, with 1.0's version rules): a peer that runs another protocol version answers the fetch's hello with `Goodbye { Protocol }` and is skipped, logged; its card says its version, but the server cannot read it. Fetch another version's card once hellos negotiate versions.
- 2026-10-12 (after T8; **later**): a ranked game over whose result did not write (its ladder's log failed) keeps its journal, but its players may play another match on the ladder before the next start rates it, after the newer game. Keep its players out of the ladder until then if logs ever fail in practice.
- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
