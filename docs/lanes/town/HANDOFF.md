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
  end teach; no version, no replay moved) and **T49a1** (decision 192, T49a cut in two,
  `cross-lane`: the night's refill, TWN-GLD-008, and the Mana Vortex with its week, TWN-GLD-009,
  at the visit's end, the builds and the hire; `h3-map`'s three oracles taught; no version, no
  replay moved) and **T49a2** (decision 193, `cross-lane`: `TownCommand::Open`, TWN-GLD-006,
  listed only where it gives something; protocol v79, replay format v24, encoding v18;
  `legal-towns-36` re-baselined at the rule's own cost) done. **Next:** **T49b**,
  `BuySpellBook` (`QUEUE.md`). Waiting
  elsewhere: `TWN-STUB-8` (the battle lane's inbox), `-13` (Visions' reach), `TWN-STUB-2` (the
  victory conditions), `TWN-STUB-16` (E10's loss condition), `-17` (increment 7), `-18` (E13).
  **Found, pre-existing:** a hygiene case of `h3-map`'s `a_viewer_learns_nothing_it_cannot_see`
  that fails on `master` too (a captured town's garrison seen; the lane's inbox).
- **Open pull requests:** T49a2's #1128 (`town/e09-t49a2`, `cross-lane`), opened at the session's end;
  watch it until it merges (T49a1's #1124 merged 2026-11-22).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure
  lane's (with T47a2: game state hash v38, map import v28; adventure part v24, adventure state hash
  v27; replay format v24, T49a2), the env's (v18, T49a2) and the protocol's (v79, T49a2) are bumped
  `cross-lane` (each task's record in `E09-log.md`; increment 5's plan in design §96). Shared: core
  ruleset hash v53, pack format v53 (E08 T124a), quirk flags 51 (E08 T140; `TWN-QUIRK-14`'s at T45).
- **Ids** (next): questions `TWN-Q34`; decisions 194; quirks `TWN-QUIRK-15` (`-14` entered by T45,
  taught by T48a's Aurora); stubs `TWN-STUB-21`; experiment batch `TWN-6`; disputes `TD-28`;
  quirk candidates `T-19`. ADR-0028 is the lane's; no drafts open; TWN-GLD-001 to -003, -004's
  Aurora set, TWN-SET-012, TWN-VEW-004's guild, TWN-GLD-004's learning and -005's town triggers built
  (T46 to T48a), -005's visit, capture and siege (T48b), -008 and -009 (T49a1), -006 (T49a2);
  planned -007.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-22 — T49a2: the town screen's opening, `TownCommand::Open`, `cross-lane`

Baseline e478f97 (#1124 merged; the setup's full baseline passed every step up to coverage,
stopped there). The user's computer was connected at the start only: 71 files staged, its shell
down; SHA-256 checked. The agent recommended a cut in two; the user kept T49a2 whole (decision
193). Built: the command, `guild::open` and `opens`, `h3_towns::opening_gives` with the part's
derived `GuildRoles`, one entrance lookup a town in the listing; the wire, the word, `TOWN_OPEN`
(103); the plans. `legal-towns-36` +49 % at the first build (no opening listed), +5.3 % after
the design pass (`TRAPS.md`), re-baselined. The 111 goldens' `format` line only.

### 2026-11-21 — T49a1: the night's refill and the Mana Vortex, `cross-lane`

Baseline a9d593d (#1120 merged; the setup's full baseline passed every step up to coverage,
stopped there). The user's computer was connected at the start only: 71 files staged, its shell
down; SHA-256 checked. The user cut T49a in two (decision 192). Built: `h3_towns::refills`,
`vortex`; the game's `town::guild::refilled` (read by `regain_mana`: a separate pass failed the
bench gate), `vortex_to`, `vortex`, `visit`
(the visit's end, the builds, the hire), the week's flags; five game tests, two pure ones; three
`h3-map` oracles taught. The 111 goldens unchanged.

### 2026-11-21 — T48b: the visit's end and a siege's end teach the guild's spells, `cross-lane`

Baseline 35bedb9 (#1115 merged; CI green: the setup's light gates only, `check-exe-map` red with
the executable staged: E31's addresses, the AI lane's inbox). The user's computer was connected at
the start only: 71 files staged, its shell down; SHA-256 checked. The draft was measured first (the
111 goldens unchanged), then the user kept T48b whole (decision 191). Built:
`AdventureOutcome::town_entered` set by `visit_town`, `learn` after `settle_outcome`'s capture and
in `end_battle` for a siege's town before the levels; three tests, three adventure tests amended.
