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
- 2026-10-12 (server lane, E16 M3 T5; cross-lane, the user): what a client of a server with accounts must do (wire 13; `docs/epics/E16-design.md` §10, ADR-0019 point 1). (1) **The key**: the login name (3–20 of `a–z 0–9 _ -`, lowercased by the client) and the password derive the account's Ed25519 key: `seed = argon2id(password, salt = "h3-account-v1:" + login, 64 MiB, 3 passes, 1 lane, 32 bytes)`, the seed as Ed25519's secret key; the test vector (login `alice`, password `correct horse battery staple`): seed `a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69`, public key `b17dbb2796b53c1c636e9a5981d6424ac5ee096b89b47223567831f435a8af14`. Derive once per device and cache the key, never the password. (2) **The login**: sign `Login` (`h3_protocol::account::login_message`) only if the `Welcome`'s `id` is the address the client dialed (a hostile server could relay the login otherwise); a registration names a display name. After a password change, `Rekey` with both keys at the next login on each server the client knows. (3) **Seats by account**: a seat taken logged in belongs to the account; its `Joined` carries no token, and the client re-takes it after a drop or a restart by logging in and joining without a token; `GameInfo::seated` names the seats' accounts by display name. (4) **New goodbyes**: `Rekeyed` (the account re-keyed elsewhere: log in again), `CommandLimit` (the game is void), `Abandoned` (a waiting game its players left). The server's test client (`crates/h3-server/src/client.rs`) shows the derivation, the login and the re-key (it signs the welcome's id without that check: a test tool).
