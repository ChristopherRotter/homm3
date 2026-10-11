# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

### Asset-notes session

- **Scope:** the user dictates their knowledge of the game's assets and their traps (formats, palettes, animation groups, fonts, sound, anything they know to go wrong); the agent records it in `HANDOFF.md` ("Asset notes") without judging it yet.
- **Acceptance:** the user confirms the notes are complete enough to start the client refinement.
- **Depends on:** Phase 0 merged.
- **Status:** ready (a session with the user).

### Client refinement (E17)

- **Scope:** the refinement session with the user, from the asset notes and the agreed points in `HANDOFF.md`: the client ADR draft (language confirmed or argued against, the renderer, the `clients/` workspace layout), the walking skeleton (a battle replay viewer on the real assets), the autonomy contract, the first milestone and its tasks in this queue.
- **Acceptance:** the user agreed the ADR draft's direction, the autonomy contract and the first milestone.
- **Depends on:** the asset-notes session. Implementation waits until the battle actions are mostly stable (E04b); the user may start the asset and render-to-PNG layer earlier.
- **Status:** waiting (asset notes).

## Inbox

- 2026-10-09 (server lane, SRV-Q3, the user): the server's first client is its own headless test client (M1); the client lane's viewer comes next. Proposal for the client refinement: the viewer reads `h3-protocol` messages live over the WebSocket or from a *record* file (one viewer's message log, JSON lines) with the same code, so the record format is the viewer's file format and one code path plays a live game and a recorded one. Open for the client ADR: whether the client workspace depends on `crates/h3-protocol` by path. Context: `docs/epics/E16-server.md`, `docs/epics/E16-design.md`.
- 2026-10-11 (server lane, E16 M2 T5e; cross-lane, the user): two things a client of the server must do. (1) **The content check** (`docs/epics/E16-design.md` §3): if the client has the packs and the profile that `Joined`'s manifest (or the lobby's `Offer`) names, it compiles them and compares the ruleset hash; equal means its own previews (damage ranges, reachable hexes) are safe, different or absent means the server's queries only and a visible note. The server never learns the result. (2) **A server restart**: a restored game's seats re-join with their tokens and get a fresh snapshot; the viewer's message numbers and its record start afresh (a `FetchRecord` then begins at the re-join), so a client drops what it numbered before; the whole game is in the replay once it is over (`FetchReplay`, or the public archive: `ListReplays`, `FetchArchived`). Context: `crates/h3-protocol/src/wire.rs`, `docs/06-security.md` ("Hosted games").
