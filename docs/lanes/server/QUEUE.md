# Server lane — QUEUE

Pre-refined tasks of the server lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E16-server.md` (in progress; M0–M4 done 2026-10-10 to 2026-10-12, each checkpoint passed: SRV-Q12, SRV-Q16, SRV-Q18, SRV-Q22; the done tasks in full in `E16-log.md`). Work under the autonomy contract (`HANDOFF.md`).

M4's checkpoint passed 2026-10-12 (SRV-Q22); T8 and M4's review items done the same day (`E16-log.md`). **M6 refined 2026-10-13 (SRV-Q23):** ADR-0021 accepted (in the refinement's pull request, a `shared:` commit, `cross-lane`); the working detail in `E16-design.md` §12; cut below, each task about half a context. Wire changes bump `PROTOCOL_VERSION` (21 now), journal changes `JOURNAL_FORMAT_VERSION` (7; M6 changes no journal).

### E16 M6 T1 — The server key and the signed welcome

- **Scope:** `<data>/server.key` (written at first start from OS randomness, read at every start; a file that does not parse stops the start, never replaced); `h3_protocol` builds `welcome_message(id, nonce, challenge)` (`h3-welcome-v1`); `Hello { …, nonce, address }`; `[server] addresses` (`public_url` read as its first entry when alone); the welcome's `id` is the dialed address when it is the server's, otherwise the first; `Welcome { …, key, signature }`; the test client verifies and pins (a changed key at a pinned address refused); wire 22. `E16-design.md` §12, "Server key", "The signed welcome".
- **Acceptance:** tests: the signed bytes and the hello's bounds (`h3-protocol`, 100 %); end to end, the welcome verifies at each configured address and a login works at a second address, the key survives a restart, a malformed key file stops the start, a changed key refused by the pinning client. Scoped verify without a failing step.
- **Depends on:** the refinement (done).
- **Status:** ready.

### E16 M6 T2 — The card and `Describe`

- **Scope:** `ServerCard`, `SignedCard` (the card's text as signed, `h3-card-v1`) and their bounds in `h3-protocol`; `Describe`/`Described` after the hello, without a login; `[server] name`, `description`, `operator`, `[discovery] card_ms`, `counts`; the card signed when asked and older than a minute; the wall clock behind one local `allow`; card verification in `h3-server` (signature, expiry, skew) for T3 and the test client; `h3-server card`; wire 23. §12, "Card".
- **Acceptance:** tests: a card verifies; a tampered, expired, future-dated or foreign-keyed card fails; the counts follow the configuration; `Describe` before the hello refused. Scoped verify.
- **Depends on:** T1.
- **Status:** ready.

### E16 M6 T3 — Peers, the fetch and `ListPeers`

- **Scope:** a `shared:` commit first (label `cross-lane`): the workspace's entries for `rustls` 0.23 (no default features; `std`, `tls12`), `rustls-graviola` 0.4, `rustls-native-certs` 0.8 and `tokio-tungstenite`'s `rustls-tls-native-roots`, `arch.toml`'s externals for `h3-server` (`cargo deny` must pass unchanged). Then `[[peer]]`, `[discovery] refresh_ms`, `timeout_ms`; one fetch task (at start, each refresh): dial (`wss://` with the server's `ClientConfig` over `rustls-graviola` and the OS roots; `ws://`), `Hello`, the welcome checked against the configured key, `Describe`, the card verified; frames bounded per fetch; a failing peer logged, its last good card kept until expiry; the cache with `via`; `ListPeers`/`Peers` (pages of 50); wire 24. §12, "Peers and the fetch", "`ListPeers`".
- **Acceptance:** tests over loopback: A peers B, A's `ListPeers` serves B's card; a wrong key, a forged or expired card dropped and logged; B down keeps its last good card; a `wss://` fetch against a local TLS endpoint with a test certificate and a test root (no further crate; if one is needed, ask first). Scoped verify; `cargo deny check` passes.
- **Depends on:** T2.
- **Status:** ready.

### E16 M6 T4 — Beyond the peers: the chain

- **Scope:** below `depth`, the peers' `ListPeers` taken into the cache with `via: [P, …]`, each link checked against the cards, duplicates by the shortest chain, at most `cards` (nearer hops first, peers in configured order); never a dial to an address from a card. §12, "The chain".
- **Acceptance:** tests: A → B → C, A serves C via B and never dials C; a card B relays outside its vouching dropped; the depth and count bounds; a client's check of the links (the test client). Scoped verify.
- **Depends on:** T3.
- **Status:** ready.

### E16 M6 T5 — Lists

- **Scope:** `ServerList`, `SignedList` (`h3-list-v1`) and bounds in `h3-protocol`; `<data>/lists/` read at start and each refresh, a list that does not verify or has expired skipped and logged; the listed servers fetched as peers (depth 1, within `cards`, after the peers), their entries marked with the list's key; `Lists`/`ListsAre`; `h3-server list-key` and `sign-list`; wire 25. §12, "Lists".
- **Acceptance:** tests: a signed list's servers appear marked with its key; an expired, forged or malformed list skipped; the tools make a list that verifies. Scoped verify.
- **Depends on:** T3 (T4 for the order of the bound).
- **Status:** ready.

### E16 M6 T6 — Closure

- **Scope:** `docs/06-security.md`'s discovery (the server key as the data directory's secret, cards, peers, the TLS client, lists) and the client lane's inbox (what a server browser needs: the welcome's check and pinning, `Describe`, `ListPeers` and the chain, `Lists`), both `cross-lane` commits; the operator's notes (`crates/h3-server/README.md`: the key, peers, lists, the TLS client's platforms); M6's exit criteria shown (`E16-server.md`); the reviewer agent; the user's checkpoint.
- **Acceptance:** the exit criteria; the reviewer's major findings fixed; the checkpoint (an `SRV-Q`).
- **Depends on:** T1–T5.
- **Status:** ready.

## Inbox

- 2026-10-13 (battle lane, the rear hex as a two-hex stack's position; **next session**): `PROTOCOL_VERSION` is 23 (after #147's v22). The user decided that a two-hex stack's position is its rear hex, as in the executable (BAT-GEO-002), so `Stack.hex`, `Command::Move`'s `to` and `Command::Attack`'s `from` name a two-hex stack's rear hex; `Shoot`'s `at` is still the target's front hex (or, for area shots, its rear hex). The battle lane changed only the three `wire.rs` comments and the version (no type changed; its pull request is `cross-lane`); the lane's HANDOFF names v22 (#147). The games are the same (800 random battles compared), so `session-random-play-full`'s baseline holds; whether a journal or record written before needs its own format bump is the lane's call.
- 2026-10-12 (after T8; **later**): a ranked game over whose result did not write (its ladder's log failed) keeps its journal, but its players may play another match on the ladder before the next start rates it, after the newer game. Keep its players out of the ladder until then if logs ever fail in practice.
- 2026-10-12 (the user, SRV-Q19; **M5**): the turn timer against a connected player who stalls is a game rule (`h3-game`; E08 decision 8's system commands for turn timers); until then a ranked battle can be stalled (the epic's risks).
- 2026-10-11 (M2 T5b; **later**, M4's refinement: the server's own bots send no messages): a separate rate for automated clients: an operator raises `messages_per_second` and `messages_per_address` meanwhile.
- 2026-10-11 (M3's review; **later**): the player log's readers (every login, the games' lookups of display names and preferences) wait behind a write's sync under its one lock; `account_changes_per_address` bounds the writes. Split the lock (a read lock for the accounts, the file's own for the writes) if logins stall under load.
- 2026-10-11 (M3 T3; **later**): the lobby shows a seat's display name as it stood at the seats' last change: a rename during a game shows at the game's next seat change or restart. Refresh the games' names on a rename if players notice.
