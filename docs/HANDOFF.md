# HANDOFF

Read first, then your lane's HANDOFF: the lane index and the few traps that cross lanes (a lane's state and traps: its own HANDOFF and `TRAPS.md`). Rules: `docs/08-work-process.md` ("Lanes", "Keeping the start-of-session reading small"); under 6 000 bytes (`check-docs`; aim for 3 000); changed by `shared/` PRs.

## Lanes

| Lane | Owns (`lanes.toml`) | State, traps, tasks |
|---|---|---|
| battle | `h3-battle`, `h3-content-battle`, `h3-pack-battle`, `h3-cli`, formats/tools crates, packs, fixtures, experiments | [HANDOFF](lanes/battle/HANDOFF.md), [QUEUE](lanes/battle/QUEUE.md) |
| adventure | `h3-adventure`, `h3-game`, `h3-map`, `h3-pack-adventure`, `h3-heroes`; `src/adventure/` in `h3-cli`, its `replay.rs` in `h3-sim`, its replays, `h3m.md` | [HANDOFF](lanes/adventure/HANDOFF.md), [QUEUE](lanes/adventure/QUEUE.md) |
| ai | `h3-sim`, `h3-ffi`, `python/` | [HANDOFF](lanes/ai/HANDOFF.md), [QUEUE](lanes/ai/QUEUE.md) |
| server | `h3-protocol`, `h3-server` | [HANDOFF](lanes/server/HANDOFF.md), [QUEUE](lanes/server/QUEUE.md) |
| client | battle client: `clients/` (own Cargo workspace) | [HANDOFF](lanes/client/HANDOFF.md), [QUEUE](lanes/client/QUEUE.md) |
| map-client | adventure map client: `clients/crates/h3-map…`, `clients/map/` | [HANDOFF](lanes/map-client/HANDOFF.md), [QUEUE](lanes/map-client/QUEUE.md) |

Shared (own PR, merged first): `h3-kernel`, `h3-contracts`, the core `h3-content`, `h3-pack` (R01), workspace and gate files (`Cargo.toml`, `arch.toml`, `lanes.toml`, `xtask`, CI), ADRs, process docs, `BACKLOG.md`, this file. Common: `OPEN-QUESTIONS.md`, `docs/stubs.toml` (a block per lane), both `Cargo.lock`s, `docs/exe/map.toml`, `structs.toml` (a line per entry), the client foundation and harness, each lane's `QUEUE.md` inbox.

- **GitHub and CI:** `ChristopherRotter/homm3` (private, free plan: without minutes or billing runs fail unstarted, the full local verify stands in, `05-testing.md` "CI"). `ci.yml` runs `check-lane` and the full verify for pull requests and `master` (once per code, not for docs only), weekly `cargo deny check advisories`. Changes reach `master` by pull request only, **opened when its branch is ready to merge**; each push to an open one runs CI: merge `master` into it after `master`'s latest run finished. **Going public** is prepared and decided (Q59, `adr/answered-questions.md`): once the user switches the visibility, a `shared/` change does that answer's "after the switch" list (CI then runs free).

## Traps for every lane

- **Battle types on the wire** (ADR-0018 point 5): a new `Command`, `EventKind`, `CommandError`, `CastTarget`, `DamageRoll` or `ObstacleKind` variant, or a field of `Observation`'s structs, breaks `h3-protocol`'s build until mapped (same PR, `cross-lane`, or server lane first).
- **Hero-level properties** (a hero's available spells, a spell's mastery) are derived once in `h3-heroes` (ADR-0013 point 2, BAT-Q19): the adventure map calls them, never a copy.

**Where the other traps live** (open the one your task meets, not all):

| Topic | Read |
|---|---|
| Cloud sandbox, sync, staging original data, disk, background verify, pull requests by REST, killing jobs | `docs/08-work-process.md`, "Cloud sandbox" (steps 1–6) |
| Coverage, clippy, fmt, serde, layers | `docs/05-testing.md`, "Traps in the gates" |
| Wiki, VCMI, experiments, code reading of `Heroes3.exe` | `docs/04-rules-and-sources.md` §7, `docs/exe/README.md` |
| A lane's own area | `docs/lanes/<lane>/TRAPS.md` |

A trap goes where it is used, once: the topic doc above, or the lane's `TRAPS.md`. Never copy it into a HANDOFF or a session entry; this file holds only what breaks a lane other than the one that made the change.
