# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–189, `TWN-Q30` to `TWN-Q33`;
  survey `E09-guild.md`, design Part G §82 to §100, the cut T44 to T51 and twelve exit criteria):
  T42a, T42b (the survey and its checkpoint), T43 (the drafts, the cut), T44 (batch TWN-5, T-18
  confirmed), T45 (`shared/`: the ADRs, `TWN-QUIRK-14`'s flag), T46 (the town part v9), T47a1
  (`h3-towns`' guild module), T47a2 (decision 188: TWN-SET-012
  with map import v28, the store v4, the guilds drawn at the build, game state hash v38) and
  **T47b** (decision 189, with the inbox's guild fix, `cross-lane`: the owner's
  `TownBuildingsView` gains `guild`, the shown slots as `h3_towns::GuildSlots`, `aurora`, `vortex`;
  protocol v78; `h3-map`'s hygiene oracle follows and ends at a Den's guild) done. **Next:**
  **T48a**, learning: `teaches` with the Aurora and T-18, the town's own triggers (a build, an
  event, a hire, the start, the bars), `TWN-STUB-6` and `-15` lifted (`QUEUE.md`; confirm its
  cut). Waiting
  elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-15` (increment 5), `-13`
  (Visions' reach), `TWN-STUB-2` (the victory conditions), `TWN-STUB-16` (E10's loss condition),
  `-17` (increment 7), `-18` (E13).
- **Open pull requests:** T47b's #1102 (`town/e09-t47b`, `cross-lane`), opened at the session's end;
  watch it until it merges (T47a2's #1097 merged 2026-11-21).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure
  lane's (with T47a2: game state hash v38, map import v28; adventure part v24, adventure state hash
  v27, replay format v23), the env's (v16, E08 T134b1) and the protocol's (v78, T47b) are bumped
  `cross-lane` (each task's record in `E09-log.md`; increment 5's plan in design §96). Shared: core
  ruleset hash v53, pack format v53 (E08 T124a), quirk flags 44 (T45: `TWN-QUIRK-14`'s).
- **Ids** (next): questions `TWN-Q34`; decisions 190; quirks `TWN-QUIRK-15` (`-14` entered by T45,
  its rule's test with T48a); stubs `TWN-STUB-21`; experiment batch `TWN-6`; disputes `TD-28`;
  quirk candidates `T-19`. ADR-0028 is the lane's; no drafts open; TWN-GLD-001 to -003, -004's
  Aurora set, TWN-SET-012 and TWN-VEW-004's guild built (T46, T47a1, T47a2, T47b), planned
  -004's learning, -005 to -009.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-21 — T47b: the guild in the owner's view, protocol v78, the hygiene oracle's guild fix, `cross-lane`

Baseline cfd2f0c (#1097 merged; CI on `master` still running, so the setup's full baseline ran:
its tests passed, stopped at coverage for disk). The user's computer was connected at the start
only: 71 files staged; its shell did not start, so sizes and the LODs' and executables' SHA-256
were compared. The user kept T47b whole with the inbox's guild fix (decision 189). Built: the view's
three fields (`h3_towns::shown_slots`, `has_aurora`; `GuildSlots::with` public), the wire's and
the fold's, protocol v78; the oracle's fields and its break at `ThievesGuildSeen` (the seed fails
without it); `GameEvent` 72 → 104 bytes. The server and adventure lanes' inboxes told.

### 2026-11-21 — T47a2: the town spell sets (import v28), the store v4, the guilds at the build, `cross-lane`

Baseline 87c83f6 (#1092 merged, #1090 before it; the setup's full baseline: its tests passed,
stopped at coverage for disk). The user's computer was connected at the start only: 71 files
staged; its shell did not start, so sizes and the LODs' and executables' SHA-256 were compared.
The user kept T47a2 whole (decision 188). Built: TWN-SET-012, the store v4, the game's draw (game
state hash v38, map import v28); 38 benchmarks inside the gate, none re-baselined; the 104 goldens
re-recorded and E08 T135b1's seven after the merge (`towns-s173-d21`'s siege re-played by the
battle AI), the bot reports unchanged; the
adventure and server lanes' inboxes told.

### 2026-11-20 — T47a1: `h3-towns`' guild module (the draw, what the guild shows, the Aurora set)

Baseline 9413c3f (#1087 merged, #1086 before it; CI on `master` still running, so the setup's full
baseline ran: its tests passed, stopped at coverage for disk). The user's computer was connected
at the start only: 71 files (64 of `original-data/`, the four manuals with three help files)
staged, SHA-256 checked. The user cut T47a in two (decision 186). Built: the `guild` module with
its 13 tests and the fixture's guild (`guild_core`, `guild_keep`), `h3-towns` at 100 %; the
catalog's TWN-GLD-002 to -004, `TWN-QUIRK-14`'s rule test, design §85 to §87 *As built*. Agent's
calls in decision 187 (six slots a level). PR #1090.
