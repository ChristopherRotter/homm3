# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **M0 done and merged; M1 T1 done (2026-10-10)** on `claude/jolly-cori-sip168`, pull request next: `h3-protocol` has `wire` (every M1 message, both directions), `codec` (bounded JSON frames) and `map` (wire ↔ engine; `map::observe` and `map::events` project for the viewer by construction), 100 % covered with no exclusion. `PROTOCOL_VERSION` 1. **Next: M1 T2**, the session core (`QUEUE.md`), once T1 is merged.
- **ADR-0018 point 5 amended (the user, 2026-10-10):** the mapping's matches over the engine's enums are exhaustive, no `Other` fallback; a new battle variant fails `h3-protocol`'s build until mapped (global HANDOFF trap). The amendment and the trap are shared files, so T1's pull request is cross-lane.
- **Prepared by M0:** `arch.toml` allows every edge M1 needs (`h3-server` → `h3-cli`, `tokio`, `tokio-tungstenite`, `futures-util`, `getrandom`, `toml`, `serde_json`; `cargo deny` checked them); the crates declare a dependency when they use it. Review list for the M1 checkpoint: the epic file.
- **Versions** (`docs/08-work-process.md`): `PROTOCOL_VERSION` (`h3-protocol`, bumped on every wire change), `RECORD_FORMAT_VERSION` (T2), `JOURNAL_FORMAT_VERSION` (`h3-server`, M2).
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
- A wildcard arm after every variant of an enum still counts as one uncovered region (llvm-cov), and `#[coverage(off)]` is unstable: no fallback arms in `h3-protocol` (ADR-0018 point 5 as amended).
- `prop_assert!(matches!(x, P { .. }))` without a message fails to compile: the macro formats the condition's text and reads `{ .. }` as a format argument; pass a message.

## Recent sessions

### 2026-10-10 — M1 T1, messages and codec

After M0 merged, restarted the branch from `master` and built T1 (`docs/epics/E16-log.md`, T1). The ADR's wildcard arms to `Other` were measured as uncovered regions that need a coverage exclusion; the user chose exhaustive matches instead, so point 5 was amended (revision history), the global HANDOFF got the trap (its "Phase 0" history line made room) and the `Unsupported`/`Other` wire variants were removed. Design note §2–§5 pruned (now code), §1 aligned with the code (the manifest in `Joined`, `Refused`). 18 tests; `verify --scope server` with no failing step.

### 2026-10-10 — M0, the shared skeleton

The user accepted the protocol ADR draft with a trim (136 to 85 lines, the decisions unchanged) and the `cross-lane` label for the M0 pull request. ADR-0018 numbered with its index row, the server lane's reading list in `AGENTS.md`; the crate skeletons, the workspace and `arch.toml` entries (above), `benches/server/` (README), the versions table, `BACKLOG.md` and the global HANDOFF's row. `check-arch` accepted a logic crate in the adapter layer without a tool change. Then merged `master`: the adventure lane's E08 T2 had taken ADR-0015 to ADR-0017, so the protocol ADR became **ADR-0018** (only this branch's references renumbered). The two base failures found at the start (the exe map's gap; a random battle property-test failure) were both fixed on `master` meanwhile (`docs/epics/E16-log.md`, M0).

### 2026-10-09 — refinement session, the answers

Took the user through SRV-Q1 to SRV-Q11 in four batches. On SRV-Q5 the user asked for the trade-offs in detail: scratch crates measured threads + `tungstenite` + `serde_json` at 36 crates, the same with `tokio` + `tokio-tungstenite` at 47, a clean build of the dependencies about 24 s either way; a blocking `tungstenite` socket needs one thread for reads and writes, so a thread per connection must poll; JSON is 8–10 times `postcard`'s size (a 20-stack snapshot 4.5 KB against 0.46 KB), and `postcard` refuses internally tagged enums (tested). The recommendation changed to `tokio` for connections and JSON in the default enum form; the user agreed. Spectators need a battle spectator viewer (no correct late-join snapshot otherwise). Drafts corrected, M0 and M1 queued, requests filed in the battle and client queues' inboxes (SRV-Q6 was already in the adventure lane's T1a draft), project status doc updated.
