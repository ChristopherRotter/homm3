# Server lane — HANDOFF

The server lane's living state; read after the global `docs/HANDOFF.md`. Format and budgets (`check-docs`: 10 000 bytes, three sessions): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Traps of every lane go into the global HANDOFF (a shared change). Tasks waiting: `QUEUE.md`.

## Current state

- **Not started.** The lane owns (`lanes.toml`) the new adapter crates `h3-protocol` and `h3-server` in the engine workspace, `benches/server/` and E16's epic file. Creating the crates touches `Cargo.toml` and `arch.toml`: the crate skeletons are their own `shared/` pull request, merged first.
- **Timing:** refinement and ADR now; implementation waits until the battle actions are mostly stable (E04b).
- **Next:** the refinement session with the user (brief below); it fills `QUEUE.md`. Branches: `server/<task>`.
- **Checkpoints** per milestone, under the autonomy contract below; a fresh reviewer agent checks each milestone before the user does.
- **Ids:** questions `SRV-Q<n>`, quirks `SRV-QUIRK-<n>`, stubs `SRV-STUB-<n>`. ADR drafts go to `docs/lanes/server/adr-drafts/` and get a number when the user accepts them.

## Autonomy contract (draft: confirm in the refinement)

- **Agent decides, marked provisional:** the inner structure of the server crates, names, test layout, error texts, log format; anything a later task can change without touching another crate. Mark it `provisional` (code comment or epic log).
- **Queue for the milestone review:** choices with a lasting cost that stay inside the lane: a light dependency, a message shape inside an accepted ADR, a configuration format. List them in the milestone's review list (epic file) and go on.
- **Stop the line:** seams and contracts (`h3-contracts`, the engine API the server calls), heavy dependencies (async runtime, networking, serialization frameworks), licence questions, changes to the security model (ADR-0008), anything touching another lane's paths. Write the question (`SRV-Q<n>`), stop that task and take the next independent one.

## Brief for the refinement session (E16)

Prepare each point with options and a recommendation; the user decides.

1. **Protocol ADR topics** (one ADR draft, decisions only): the transport (WebSocket, plain TCP, QUIC; HTTP for lobbies?); the async runtime and other heavy dependencies (GPLv2 compatibility, `deny.toml`); the wire format (readable JSON or a binary format, versioning, reuse of `h3-sim`'s encodings or not); sessions and seats with tokens (who holds which side, reconnects, spectators); per-viewer observations and events (`docs/06-security.md`: projection per viewer, visibility scopes); per-viewer replay export (a player's replay shows only what they saw).
2. **What can come before `h3-game` (E10):** a hosted battle: two clients, or a client and a bot, play a `BattleSetup` through the server. Enough to exercise transport, seats, validation against the trusted actor and per-viewer observation on the battle that exists.
3. **Commands the server does not know:** the protocol mapping rejects an unknown command with a typed "unsupported" error instead of an exhaustive match, so a new battle command (the battle lane adds them) does not break the server build; the same for events.
4. **Hidden information:** a server needs a secret, uniformly random seed or an independent obstacle seed (`QUEUE.md`, "Inbox"); write it into `docs/06-security.md` with the server.
5. **The autonomy contract** above: confirm or change its tiers.
6. **Questions for the user:** where the server runs first (local only?), authentication now or later, which clients connect first (the client lane's viewer, a bot, a test harness).

## Standing traps

(None yet; the global HANDOFF's traps apply.)

## Recent sessions

(None yet.)
