# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–195, `TWN-Q30` to `TWN-Q33`;
  survey `E09-guild.md`, design Part G §82 to §100, the cut T44 to T51 and twelve exit criteria):
  T42a to T50 done (each task's record in `E09-log.md`): the survey, the drafts, batch TWN-5, the
  part v9, the guild module, the town spell sets (import v28, store v4, game state hash v38), the
  owner's guild view (protocol v78), learning at every trigger (T48a, T48b), the night's refill
  and the Mana Vortex (T49a1), `TownCommand::Open` (T49a2) and `BuySpellBook` (T49b; protocol v80,
  replay format v25, encoding v19), and **T50** the gates (decision 195, kept whole with the
  inbox's hygiene failure: the property's, fixed): the guild census of the 160 maps, six guild
  properties, the guild map `e09-guild-36` with seven replays (the goldens 118), the workload
  `day-start-guild-36`; no version moved. **Next:** **T51**, the audit against design §99's
  twelve criteria, increment 5 closed, increment 6 (the services) refined (`QUEUE.md`). Waiting
  elsewhere: `TWN-STUB-8` (the battle lane's inbox), `-13` (Visions' reach), `TWN-STUB-2` (the
  victory conditions), `TWN-STUB-16` (E10's loss condition), `-17` (increment 7), `-18` (E13).
  For the next `shared/` task: `docs/06-security.md`'s ADV-FOG-006 line (T50's record).
- **Open pull requests:** T50's (`town/e09-t50`, `cross-lane`), opened at the session's end; watch
  it until it merges (T49b's #1136 merged 2026-11-23).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v4: T47a2) and
  the town ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure
  lane's (with T47a2: game state hash v38, map import v28; adventure part v24, adventure state hash
  v27; replay format v25, T49b), the env's (v19, T49b) and the protocol's (v80, T49b) are bumped
  `cross-lane` (each task's record in `E09-log.md`; increment 5's plan in design §96). Shared: core
  ruleset hash v53, pack format v53 (E08 T124a), quirk flags 51 (E08 T140; `TWN-QUIRK-14`'s at T45).
- **Ids** (next): questions `TWN-Q34`; decisions 196; quirks `TWN-QUIRK-15` (`-14` entered by T45,
  taught by T48a's Aurora); stubs `TWN-STUB-21`; experiment batch `TWN-6`; disputes `TD-28`;
  quirk candidates `T-19`. ADR-0028 is the lane's; no drafts open; every TWN-GLD rule and
  TWN-SET-012 built (T46 to T49b).
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-24 — T50: the guild census, properties, map, replays and workload, `cross-lane`

Baseline d202566 (#1136 merged; the setup's light gates, CI's fingerprint on `master`'s code).
The user's computer was connected: 5 files staged, SHA-256 checked. The agent recommended a cut
in two; the user kept T50 whole with the inbox's hygiene failure and asked for sub-agents
(decision 195): four ran one after another (the failure, the properties, the census, the map).
The failure was the property's (`TRAPS.md`). The census holds on all 160 maps with 25 survey
counts, three mutations caught; the properties cost +1 % (a first draft +29 %). The VM restarted
twice and the server once; each part was pushed as it came, nothing lost.

### 2026-11-23 — T49b: a spell book bought at a town's Mage Guild, `cross-lane`

Baseline f1c4a53 (#1128 merged; the setup's light gates, CI's fingerprint on `master`'s code).
The user's computer was connected and its shell worked: 71 files staged, SHA-256 checked. The
agent recommended a cut in two; the user kept T49b whole (decision 194). Built: the command, its
three refusals, `check_spell_book`, `guild_offers`, the game's `buy_book` and `offers`; the
wire, the word, `SPELL_BOOK` (104); the plans. The town map's seed 2541 lost its won siege: 277
after a search of 3 000. `legal-towns-36` on `master`'s games +5.5 % at the first build, +1.6 %
after the design pass (`TRAPS.md`); `day-start-builds-36` re-baselined (its games bought books).
CI's first run met a gap of T49a1's in `h3-map`'s `check_towns` (a hire's Vortex flag): taught.

### 2026-11-22 — T49a2: the town screen's opening, `TownCommand::Open`, `cross-lane`

Baseline e478f97 (#1124 merged; the setup's full baseline passed every step up to coverage,
stopped there). The user's computer was connected at the start only: 71 files staged, its shell
down; SHA-256 checked. The agent recommended a cut in two; the user kept T49a2 whole (decision
193). Built: the command, `guild::open` and `opens`, `h3_towns::opening_gives` with the part's
derived `GuildRoles`, one entrance lookup a town in the listing; the wire, the word, `TOWN_OPEN`
(103); the plans. `legal-towns-36` +49 % at the first build (no opening listed), +5.3 % after
the design pass (`TRAPS.md`), re-baselined. The 111 goldens' `format` line only.
