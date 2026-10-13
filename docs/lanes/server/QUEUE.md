# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0–M4 done 2026-10-10 to 2026-10-12, each checkpoint passed: SRV-Q12, SRV-Q16, SRV-Q18, SRV-Q22; the done tasks in full in `E16-log.md`). Work under the autonomy contract (`HANDOFF.md`).

M4's checkpoint passed 2026-10-12 (SRV-Q22); T8 and M4's review items done the same day (`E16-log.md`). **M6 refined 2026-10-13 (SRV-Q23):** ADR-0021 accepted (in the refinement's pull request, a `shared:` commit, `cross-lane`); the working detail in `E16-design.md` §12; cut below, each task about half a context; T1 done 2026-10-13 (the server key and the signed welcome, wire 24), T2 the same day (the card and `Describe`, wire 25; `E16-log.md`). Wire changes bump `PROTOCOL_VERSION` (25 now), journal changes `JOURNAL_FORMAT_VERSION` (8 since the rear hex, `E16-log.md`; M6's tasks change no journal).

### E16 M6 T3 — Peers, the fetch and `ListPeers`

- **Scope:** a `shared:` commit first (label `cross-lane`): the workspace's entries for `rustls` 0.23 (no default features; `std`, `tls12`), `rustls-graviola` 0.4, `rustls-native-certs` 0.8 and `tokio-tungstenite`'s `rustls-tls-native-roots`, `arch.toml`'s externals for `h3-server` (`cargo deny` must pass unchanged). Then `[[peer]]`, `[discovery] refresh_ms`, `timeout_ms`; one fetch task (at start, each refresh): dial (`wss://` with the server's `ClientConfig` over `rustls-graviola` and the OS roots; `ws://`), `Hello`, the welcome checked against the configured key, `Describe`, the card verified; frames bounded per fetch; a failing peer logged, its last good card kept until expiry; the cache with `via`; `ListPeers`/`Peers` (pages of 50); a wire bump. §12, "Peers and the fetch", "`ListPeers`".
- **Acceptance:** tests over loopback: A peers B, A's `ListPeers` serves B's card; a wrong key, a forged or expired card dropped and logged; B down keeps its last good card; a `wss://` fetch against a local TLS endpoint with a test certificate and a test root (no further crate; if one is needed, ask first). Scoped verify; `cargo deny check` passes.
- **Depends on:** T2 (done: `card::verify`, `Cards`; the card's `peers` wait for `[[peer]]`).
- **Status:** ready.

### E16 M6 T4 — Beyond the peers: the chain

- **Scope:** below `depth`, the peers' `ListPeers` taken into the cache with `via: [P, …]`, each link checked against the cards, duplicates by the shortest chain, at most `cards` (nearer hops first, peers in configured order); never a dial to an address from a card. §12, "The chain".
- **Acceptance:** tests: A → B → C, A serves C via B and never dials C; a card B relays outside its vouching dropped; the depth and count bounds; a client's check of the links (the test client). Scoped verify.
- **Depends on:** T3.
- **Status:** ready.

### E16 M6 T5 — Lists

- **Scope:** `ServerList`, `SignedList` (`h3-list-v1`) and bounds in `h3-protocol`; `<data>/lists/` read at start and each refresh, a list that does not verify or has expired skipped and logged; the listed servers fetched as peers (depth 1, within `cards`, after the peers), their entries marked with the list's key; `Lists`/`ListsAre`; `h3-server list-key` and `sign-list`; a wire bump. §12, "Lists".
- **Acceptance:** tests: a signed list's servers appear marked with its key; an expired, forged or malformed list skipped; the tools make a list that verifies. Scoped verify.
- **Depends on:** T3 (T4 for the order of the bound).
- **Status:** ready.

### E16 M6 T6 — Closure

- **Scope:** `docs/06-security.md`'s discovery (the server key as the data directory's secret, cards, peers, the TLS client, lists) and the client lane's inbox (what a server browser needs: the welcome's check and pinning, `Describe`, `ListPeers` and the chain, `Lists`), both `cross-lane` commits; the operator's notes (`crates/h3-server/README.md`: the key, peers, lists, the TLS client's platforms); M6's exit criteria shown (`E16-server.md`); the reviewer agent; the user's checkpoint.
- **Acceptance:** the exit criteria; the reviewer's major findings fixed; the checkpoint (an `SRV-Q`).
- **Depends on:** T1–T5.
- **Status:** ready.

## Inbox

- 2026-10-12 (after T8; **later**): a ranked game over whose result did not write (its ladder's log failed) keeps its journal, but its players may play another match on the ladder before the next start rates it, after the newer game. Keep its players out of the ladder until then if logs ever fail in practice.
- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
