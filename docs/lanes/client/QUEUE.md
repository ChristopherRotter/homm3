# Client lane — QUEUE

Pre-refined tasks of the client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12; ADR-0020). M1 below (T2 and T3 done 2026-10-12: the decoders, the gate's inventory; `docs/epics/E17-design.md`), each task about half a context. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the user's computer must be online); never from the installation. The client's gates: `clients/scripts/check.sh` (the full local check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from `clients/`.

### E17 M1 T1 — Code reading: archives, screen format, keyed colours

- **Scope:** from `Heroes3.exe` (`docs/exe/README.md`, `scripts/exe.py`; every cited address gets a `map.toml` entry): how the game looks up an entry name across the LODs and which archive wins (research item 1); the screen's pixel format (16-bit? 565 or 555?) and how a palette colour becomes a screen pixel (item 2: the user's off-colour pixels); how a DEF frame's indices are drawn: transparent, shadow (the operation on the pixel below, and where two shadows overlap), selection outline, player colours (item 8); how a 16-bit frame is shown on a 32-bit desktop where it matters for capture; the decoders' open points (`E17-design.md` §3: the archive's type against the DEF's, `SGTWMTA`/`SGTWMTB`'s short frame header, `OVSLOT.def`'s canvas, encoding 2's row offsets). Compare with VCMI where it has an answer. Record in `docs/epics/E17-design.md` (§3 onwards).
- **Acceptance:** each question answered with addresses and our own words, or listed as open with what was tried; no code; the design note under 15 KB.
- **Depends on:** nothing.
- **Status:** ready.

### E17 M1 T4 — The asset tables

- **Scope:** the tables' format (*queued for the review*): `clients/assets/{base,ab,sod}.toml` in the packs' dependency graph and keys; entries for creatures (key → DEF, its groups as the battle uses them, missile and offsets from `CRANIM.TXT`), battlefield backgrounds (by terrain and special field), obstacles; the fix entry (`CLT-QUIRK-<n>`, its source, its flag; none yet unless found). A harvesting tool in `clients/` (a bin; reads the user's executable and text tables once; the executable's addresses from `docs/exe/`) generates them. Which expansion each entry belongs to: the AB/SoD archives' split and the packs (research item 6). A loader with the tables' dependency rules (as `h3-pack`'s: unknown keys and fields are errors). The gate checks every name against the archives.
- **Acceptance:** the Definition of Done; the tables load for each expansion selection; every key of the committed packs' creatures, battlefields and obstacles has an entry, or the gap is listed.
- **Depends on:** T1 (precedence); T3 done.
- **Status:** waiting (T1).

### E17 M1 T5 — Code reading: the battle screen

- **Scope:** from the executable (and VCMI, compared): the battlefield's hex geometry in pixels, where a one- and a two-hex stack's sprite is anchored, facing and mirroring, the background and obstacle placement, the draw order (stacks, obstacles, shadows, the selection), the stack count box's position. Record in `E17-design.md`.
- **Acceptance:** as T1.
- **Depends on:** T1.
- **Status:** waiting (T1).

### E17 M1 T6 — The compositor

- **Scope:** `h3-render`: the frame in the screen format T1 found, its size a parameter; blitting an indexed frame with its keyed colours as T1 found (shadow included), player colours, mirroring, clipping; a PNG writer as an adapter (*the `png` crate: queued for the review*). Synthetic tests, 100 %.
- **Acceptance:** the Definition of Done; a test per keyed colour and for overlapping shadows.
- **Depends on:** T1; T3 done.
- **Status:** waiting (T1).

### E17 M1 T7 — A battle still

- **Scope:** the battle's assets loaded as one checkpoint batch (ADR-0020 point 7: an archive set in the adapter, the directories at start, the batch's entries by offset; whole archives read until CLT-Q2's table-only reader lands); `h3-view render`: from a record's `Snapshot` (an `h3-protocol` message; a small adapter makes one by running `h3-protocol`'s session in-process on a scenario file as `h3 run` takes it, as a spectator) to a PNG in `clients/out/`: the background, obstacles, every stack in its standing frame on its hex, facing its side, with T5's geometry and draw order; layouts as data. Pure scene model (snapshot + tables + time 0 → draw list), adapters for files.
- **Acceptance:** the Definition of Done; stills for three scenarios (one- and two-hex stacks, obstacles, both sides) rendered from the user's files and sent to the user.
- **Depends on:** T4, T5, T6.
- **Status:** waiting (T4–T6).

### E17 M1 T8 — M1's closure

- **Scope:** M1's exit criteria checked; the review list (epic file) completed; a fresh reviewer agent over M1; the user compares the stills with the original (the differences become M2/M3 items or fixes); the M2 refinement prepared (the window/audio evaluation's plan, CLT-Q1's state).
- **Acceptance:** M1's exit (epic file).
- **Depends on:** T1–T7.
- **Status:** waiting (T1–T7).

## Inbox

- 2026-10-12 (the user, at E17 M0): the adventure map's object draw order is surprisingly complicated; the user's own engine and VCMI (at the time) did not get it exactly right. Research item 9 (HANDOFF): a code reading of the executable's map drawing (the order of objects, their shadows, heroes and overlapping tiles) when the adventure map viewer is refined, with reference frames from the original to confirm.
- 2026-10-12 (E17 M0 checkpoint, the user): **CLT-Q2**, a table-only LOD reader in `h3-formats` for the battle lane (`docs/OPEN-QUESTIONS.md`). When it lands, the client's archive set switches from whole reads to the directories at start and a batch's entries by offset; `h3-view gate` then measures that.
- 2026-10-09 (server lane, SRV-Q3, the user): the server's first client is its own headless test client (M1); the client lane's viewer comes next. Proposal for the client refinement: the viewer reads `h3-protocol` messages live over the WebSocket or from a *record* file (one viewer's message log, JSON lines) with the same code, so the record format is the viewer's file format and one code path plays a live game and a recorded one. Open for the client ADR: whether the client workspace depends on `crates/h3-protocol` by path. Context: `docs/epics/E16-server.md`, `docs/epics/E16-design.md`.
- 2026-10-11 (server lane, E16 M2 T5e; cross-lane, the user): two things a client of the server must do. (1) **The content check** (`docs/epics/E16-design.md` §3): if the client has the packs and the profile that `Joined`'s manifest (or the lobby's `Offer`) names, it compiles them and compares the ruleset hash; equal means its own previews (damage ranges, reachable hexes) are safe, different or absent means the server's queries only and a visible note. The server never learns the result. (2) **A server restart**: a restored game's seats re-join with their tokens and get a fresh snapshot; the viewer's message numbers and its record start afresh (a `FetchRecord` then begins at the re-join), so a client drops what it numbered before; the whole game is in the replay once it is over (`FetchReplay`, or the public archive: `ListReplays`, `FetchArchived`). Context: `crates/h3-protocol/src/wire.rs`, `docs/06-security.md` ("Hosted games").
- 2026-10-12 (server lane, E16 M3 T5; cross-lane, the user): what a client of a server with accounts must do (wire 13; `docs/epics/E16-design.md` §10, ADR-0019 point 1). (1) **The key**: the login name (3–20 of `a–z 0–9 _ -`, lowercased by the client) and the password derive the account's Ed25519 key: `seed = argon2id(password, salt = "h3-account-v1:" + login, 64 MiB, 3 passes, 1 lane, 32 bytes)`, the seed as Ed25519's secret key; the test vector (login `alice`, password `correct horse battery staple`): seed `a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69`, public key `b17dbb2796b53c1c636e9a5981d6424ac5ee096b89b47223567831f435a8af14`. Derive once per device and cache the key, never the password. (2) **The login**: sign `Login` (`h3_protocol::account::login_message`) only if the `Welcome`'s `id` is the address the client dialed (a hostile server could relay the login otherwise); a registration names a display name. After a password change, `Rekey` with both keys at the next login on each server the client knows. (3) **Seats by account**: a seat taken logged in belongs to the account; its `Joined` carries no token, and the client re-takes it after a drop or a restart by logging in and joining without a token; `GameInfo::seated` names the seats' accounts by display name. (4) **New goodbyes**: `Rekeyed` (the account re-keyed elsewhere: log in again), `CommandLimit` (the game is void), `Abandoned` (a waiting game its players left). The server's test client (`crates/h3-server/src/client.rs`) shows the derivation, the login and the re-key (it signs the welcome's id without that check: a test tool).
