# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–190, `TWN-Q30` to `TWN-Q33`;
  survey `E09-guild.md`, design Part G §82 to §100, the cut T44 to T51 and twelve exit criteria):
  T42a, T42b (the survey and its checkpoint), T43 (the drafts, the cut), T44 (batch TWN-5, T-18
  confirmed), T45 (`shared/`: the ADRs, `TWN-QUIRK-14`'s flag), T46 (the town part v9), T47a1
  (`h3-towns`' guild module), T47a2 (decision 188: TWN-SET-012
  with map import v28, the store v4, the guilds drawn at the build, game state hash v38), T47b
  (decision 189: the owner's `TownBuildingsView` gains `guild`, `aurora`, `vortex`; protocol v78)
  and **T48a** (decision 190, `cross-lane`: `h3_towns::teaches`, TWN-GLD-004 complete; the game's
  `town::guild::learn` at a build, a town event's build, the bars' moves, a hire and the start,
  TWN-GLD-005; `TWN-STUB-6` and `-15` lifted; no version, no replay moved) and **T48b**
  (decision 191, `cross-lane`: `AdventureOutcome::town_entered`, the visit's end and a siege's
  end teach; no version, no replay moved) done. **Next:** **T49a**, the night's refill, the Mana
  Vortex, `Open` (`QUEUE.md`; confirm its cut). Waiting
  elsewhere: `TWN-STUB-8` (the battle lane's inbox), `-13` (Visions' reach), `TWN-STUB-2` (the
  victory conditions), `TWN-STUB-16` (E10's loss condition), `-17` (increment 7), `-18` (E13).
  **Found, pre-existing:** a hygiene case of `h3-map`'s `a_viewer_learns_nothing_it_cannot_see`
  that fails on `master` too (a captured town's garrison seen; the lane's inbox).
- **Open pull requests:** T48b's #1120 (`town/e09-t48b`, `cross-lane`), opened at the session's end;
  watch it until it merges (T48a's #1115 merged 2026-11-21).
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
- **Ids** (next): questions `TWN-Q34`; decisions 192; quirks `TWN-QUIRK-15` (`-14` entered by T45,
  taught by T48a's Aurora); stubs `TWN-STUB-21`; experiment batch `TWN-6`; disputes `TD-28`;
  quirk candidates `T-19`. ADR-0028 is the lane's; no drafts open; TWN-GLD-001 to -003, -004's
  Aurora set, TWN-SET-012, TWN-VEW-004's guild, TWN-GLD-004's learning and -005's town triggers built
  (T46 to T48a), -005's visit, capture and siege (T48b); planned -006 to -009.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-21 — T48b: the visit's end and a siege's end teach the guild's spells, `cross-lane`

Baseline 35bedb9 (#1115 merged; CI green: the setup's light gates only, `check-exe-map` red with
the executable staged: E31's addresses, the AI lane's inbox). The user's computer was connected at
the start only: 71 files staged, its shell down; SHA-256 checked. The draft was measured first (the
111 goldens unchanged), then the user kept T48b whole (decision 191). Built:
`AdventureOutcome::town_entered` set by `visit_town`, `learn` after `settle_outcome`'s capture and
in `end_battle` for a siege's town before the levels; three tests, three adventure tests amended.

### 2026-11-21 — T48a: heroes learn the guild's spells at the town's own triggers, `cross-lane`

Baseline c0ce729 (#1102 merged; CI green: the setup's light gates only). The user's computer was
connected at the start only: 71 files staged, the LODs' and executables' SHA-256 checked. The user
kept T48a whole (decision 190). Built: `h3_towns::teaches` (`h3-towns` now depends on
`h3-heroes`), `town::guild::learn` and `at_start` with their five call sites,
`GameState::change_hero` `pub(crate)`; `h3-map`'s turn-start oracle and two views checks taught the
learning (a hire's hero learns at once). No golden replay moved; the 38 benchmarks inside the gate.
A hygiene seed that fails on `master` too went to the lane's inbox, not to `view.txt`.

### 2026-11-21 — T47b: the guild in the owner's view, protocol v78, the hygiene oracle's guild fix, `cross-lane`

Baseline cfd2f0c (#1097 merged; CI on `master` still running, so the setup's full baseline ran:
its tests passed, stopped at coverage for disk). The user's computer was connected at the start
only: 71 files staged; its shell did not start, so sizes and the LODs' and executables' SHA-256
were compared. The user kept T47b whole with the inbox's guild fix (decision 189). Built: the view's
three fields (`h3_towns::shown_slots`, `has_aurora`; `GuildSlots::with` public), the wire's and
the fold's, protocol v78; the oracle's fields and its break at `ThievesGuildSeen` (the seed fails
without it); `GameEvent` 72 → 104 bytes. The server and adventure lanes' inboxes told.
