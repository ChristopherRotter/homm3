# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M0 done (2026-10-10):** the user accepted the protocol ADR as **ADR-0018** (trimmed to its decisions; the detail stays in the epic and the design note) and the cross-lane label for the skeleton's pull request. `h3-protocol` (adapter, `logic = true`) and `h3-server` (adapter, the binary `h3-server`, stub SRV-STUB-1) exist as skeletons. **Next: M1 T1** (`QUEUE.md`) once that pull request is merged.
- **What M0 prepared for M1.** `arch.toml` already allows every edge M1 needs: `h3-protocol` → kernel, content, contracts, battle, `serde`, `serde_json` (dev: `h3-pack`, `h3-sim`, `proptest`, `postcard`); `h3-server` → those plus `h3-pack`, `h3-sim`, `h3-cli` (provisional, for the pack, scenario and replay-file functions), `tokio`, `tokio-tungstenite`, `futures-util` (`StreamExt`/`SinkExt`), `getrandom`, `toml`. The workspace has their entries; the crates add them when they use them (`Cargo.lock` is common). `cargo deny` passed with all of them in the graph (77 crates under `h3-server`).
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` in `h3-protocol` (0 until T1's first messages, then 1 and bumped on every wire change); `RECORD_FORMAT_VERSION` (T2) and `JOURNAL_FORMAT_VERSION` (`h3-server`, M2).
- The lane owns (`lanes.toml`) `h3-protocol`, `h3-server`, `benches/server/` and E16's files; the workspace files stay shared. **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does. The identity and discovery ADR drafts are decided at the M3 refinement.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them. Branches: `server/<task>`.

## Autonomy contract (confirmed by the user, 2026-10-09)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (beyond the decided `tokio` features and `tokio-tungstenite`: another runtime, an HTTP framework, a TLS stack, the crypto crates, a store), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Standing traps

- `h3-protocol` is `no_std`: take `serde` (`derive`, `alloc`) and `serde_json` (`alloc`) with `default-features = false` in its own manifest, not `workspace = true` (the workspace entries enable `std`).
- `postcard` (and any format that is not self-describing) cannot decode `#[serde(tag = ...)]`, `untagged` or `flatten`: keep the wire types in serde's default enum form (SRV-Q5).
- A 64-bit integer on the wire travels as a string (JavaScript clients lose precision above 2^53).

## Recent sessions

### 2026-10-10 — M0, the shared skeleton

The user accepted the protocol ADR draft with a trim (136 to 85 lines, the decisions unchanged) and the `cross-lane` label for the M0 pull request. ADR-0018 numbered with its index row, the server lane's reading list in `AGENTS.md`; the crate skeletons, the workspace and `arch.toml` entries (above), `benches/server/` (README), the versions table, `BACKLOG.md` and the global HANDOFF's row. `check-arch` accepted a logic crate in the adapter layer without a tool change. Then merged `master`: the adventure lane's E08 T2 had taken ADR-0015 to ADR-0017, so the protocol ADR became **ADR-0018** (only this branch's references renumbered). The two base failures found at the start (the exe map's gap; a random battle property-test failure) were both fixed on `master` meanwhile (`docs/epics/E16-log.md`, M0).

### 2026-10-09 — refinement session, the answers

Took the user through SRV-Q1 to SRV-Q11 in four batches. On SRV-Q5 the user asked for the trade-offs in detail: scratch crates measured threads + `tungstenite` + `serde_json` at 36 crates, the same with `tokio` + `tokio-tungstenite` at 47, a clean build of the dependencies about 24 s either way; a blocking `tungstenite` socket needs one thread for reads and writes, so a thread per connection must poll; JSON is 8–10 times `postcard`'s size (a 20-stack snapshot 4.5 KB against 0.46 KB), and `postcard` refuses internally tagged enums (tested). The recommendation changed to `tokio` for connections and JSON in the default enum form; the user agreed. Spectators need a battle spectator viewer (no correct late-join snapshot otherwise). Drafts corrected, M0 and M1 queued, requests filed in the battle and client queues' inboxes (SRV-Q6 was already in the adventure lane's T1a draft), project status doc updated.

### 2026-10-08 — refinement session, first draft

Read the repo's decisions (ADR-0003/0004/0005/0008/0011, `06-security.md`, the battle's `observe`/`project`/events, `h3-sim`'s replay and encodings, E08's decisions) and wrote the first draft of E16 with no code: two ADR drafts (protocol and hosting; identity, lobbies and ladders), the epic file and a design note. Direction proposed: a sans-I/O `h3-protocol` as a logic crate (published language plus session core), `h3-server` as the shell; WebSocket, std threads, JSON; the protocol's own wire types with a mapping and an "other" fallback; seat tokens, per-viewer projection and sequence numbers; a 64-bit OS seed with separate obstacle seed; replay (post-game) against record (per viewer); one ruleset per game identified by a manifest; keypair identity; ladders with Glicko-2 and file-based persistence. The user asked for readable context and added the two ideas above; the second draft answered their follow-ups: persistence is files with a write-ahead journal per game, restored by a deterministic re-run after a crash or a pause (no state serialization); the per-device keypair gave way to a password-derived keypair so a new device needs only name and password; servers and ladders are independent by design; a corrupted game (a bit flip) is caught by the journal's checkpoint hashes and a re-run before any result counts, with an optional shadow instance; each ADR draft is decided at the milestone that needs it (protocol: M0; accounts and ladders: M3). The user's last thought, decentralized server discovery, became a third ADR draft (`adr-drafts/discovery.md`: server keys, signed cards, a web of trust between servers, signed lists, mDNS on the LAN, a DHT deferred; M6). Open: SRV-Q1 to SRV-Q11 (`docs/OPEN-QUESTIONS.md`).
