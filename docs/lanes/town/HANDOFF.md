# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–188, `TWN-Q30` to `TWN-Q33`;
  survey `E09-guild.md`, design Part G §82 to §100, the cut T44 to T51 and twelve exit criteria):
  T42a, T42b (the survey and its checkpoint), T43 (the drafts, the cut), T44 (batch TWN-5, T-18
  confirmed), T45 (`shared/`: the ADRs, `TWN-QUIRK-14`'s flag), T46 (the town part v9), T47a1
  (`h3-towns`' guild module) and **T47a2** (decision 188, kept whole, `cross-lane`: TWN-SET-012
  with map import v28 and `GameSetupError::TownSpell`; the store v4: `Town::guild`, `vortex`,
  `TownStatic::aurora`, `Towns::start_week`, `take_vortex`; the guilds drawn at the build as the
  towns' stream's first draws, `h3_game`'s `town/guild.rs`; game state hash v38; the 111 goldens
  re-recorded, `towns-s173-d21`'s day-16 siege played again by the battle AI) done. **Next:**
  **T47b**, the owner's view (the shown spells by level, the Aurora mark, the Vortex's week flag;
  TWN-VEW-004, decision 180) and the protocol, `cross-lane` (`QUEUE.md`; confirm its cut). Waiting
  elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and `-15` (increment 5), `-13`
  (Visions' reach), `TWN-STUB-2` (the victory conditions), `TWN-STUB-16` (E10's loss condition),
  `-17` (increment 7), `-18` (E13).
- **Open pull requests:** T47a2's (`town/e09-t47a2`, `cross-lane`), opened at the session's end;
  watch it until it merges (T47a1's #1090 merged 2026-11-21).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure
  lane's (with T47a2: game state hash v38, map import v28; adventure part v24, adventure state hash
  v27, replay format v23), the env's (v16) and the protocol's (v77, both E08 T134b1) are bumped
  `cross-lane` (each task's record in `E09-log.md`; increment 5's plan in design §96). Shared: core
  ruleset hash v53, pack format v53 (E08 T124a), quirk flags 44 (T45: `TWN-QUIRK-14`'s).
- **Ids** (next): questions `TWN-Q34`; decisions 189; quirks `TWN-QUIRK-15` (`-14` entered by T45,
  its rule's test with T48a); stubs `TWN-STUB-21`; experiment batch `TWN-6`; disputes `TD-28`;
  quirk candidates `T-19`. ADR-0028 is the lane's; no drafts open; TWN-GLD-001 to -003, -004's
  Aurora set and TWN-SET-012 built (T46, T47a1, T47a2), planned -004's learning, -005 to -009.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

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

### 2026-11-20 — T46: the town part v9, the guild weights harvested, `cross-lane`

Baseline a8460e1 (#1082 merged, #1080 before it; the setup's light gates on `master`'s code). The
user's computer was connected at the start only: 71 files (64 of `original-data/`, its 40 MP3s
among them, and the four manuals with three help files) staged, SHA-256 checked. The user kept T46 whole
with the `cross-lane` consent (decision 184). Built: `h3-towns`' guild weights, roles and
parameters with their refusals (part v9), `h3-pack-towns`' fields, the harvest of `SPTRAITS.TXT`'s
town columns and the gap-fill's roles; the packs re-harvested; 104 goldens' `town_ruleset_hash`.
Agent's calls in decision 185 (the slots 1 to 5; a role of every faction where its list holds the
building). PR #1086.
