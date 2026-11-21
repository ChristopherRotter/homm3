# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 4 closed (T10, T20, T30, T41; `E09-archive.md`, design notes
  Parts A to F). **Increment 5** (the mage guild; decisions 161–171, `TWN-Q30`, `TWN-Q31`): its
  survey done in two parts (decision 165): T42a (`E09-guild.md` §0 to §9, what the guild holds)
  and **T42b** (§10 to §20, what a hero gets, with the checkpoint `TWN-Q31`: TD-10, TD-25 the
  original, T-18 `fix-optional`, TD-27 the original's triggers with a free opening command;
  decisions 166–171). **T43** (kept whole, decision 172): the four drafts
  (`adr-drafts/*-increment-5.md`), the design note's Part G (§82 to §100), the cut T44 to T51 and
  twelve exit criteria, accepted at `TWN-Q33` (decisions 173–181; the guilds drawn from the towns'
  stream, decision 179, so T47a re-records the week replays' outcomes). **T44** (kept whole,
  decision 182): batch TWN-5 run twice on the capture harness, every row as the reading (survey
  `E09-guild.md` §21; **T-18 confirmed**); the writer's town spell sets and map-wide bans, the
  spec's fields. **T45** (kept whole, decision 183; `shared/`, `cross-lane`): the four drafts in
  ADR-0013, -0016, -0017 and -0028 with TWN-5's verdicts, the glossary, the security doc's guild
  section; `TWN-QUIRK-14`'s flag (the base faithful hash 0x36b98ea152b4db95, the goldens' headers
  re-recorded). **T46** (kept whole, decision 184; `cross-lane`): the town part v9, the guild
  weights harvested from `SPTRAITS.TXT`, the roles `guild`, `library`, `aurora`, `mana_vortex`,
  the slots, the book's price, the Vortex's factor (TWN-GLD-001; the goldens' town hash
  0xcf8db1aa2bb768d2). **T47a cut in two** (decision 186): **T47a1** (decision 187) `h3-towns`'
  guild module (`draw_guild`, `guild_level`, `shown`, `aurora_set` with `SpellSet`, `GuildSets`,
  `GuildSlots`: six slots a level; TWN-GLD-002, -003 and -004's set; the fixture's `guild_core`,
  `guild_keep`; no version, nothing calls it yet). **Next:** T47a2, the map's town spell sets with
  import v28, the store v4, the game's draw, every golden re-recorded (`QUEUE.md`; confirm its
  cut, `cross-lane`). Waiting elsewhere: `TWN-STUB-8` (the battle lane's inbox), `TWN-STUB-6` and
  `-15` (increment 5), `-13` (Visions' reach), `TWN-STUB-2` (the victory conditions),
  `TWN-STUB-16` (E10's loss condition), `-17` (increment 7), `-18` (E13).
- **Open pull requests:** T47a1's (#1090, `town/e09-t47a1`), opened at the session's end;
  watch it until it merges (T46's #1086 merged 2026-11-20).
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v9: T46), both in `h3-towns`; the adventure lane's
  (on `master` 2026-11-20: game state hash v37, adventure part v24, adventure state hash v27, map
  import v27, replay format v23, by E08 T132b to T134b1), the env's (v16) and the
  protocol's (v77, both E08 T134b1) are bumped `cross-lane` (each task's record in
  `E09-log.md`; increment 5's plan in design §96). Shared: core ruleset hash v53, pack format v53 (E08 T124a; v52 T24),
  quirk flags 44 (T45: `TWN-QUIRK-14`'s; 42, 43 and `StreamId::TAVERNS` (8) by T35b).
- **Ids** (next): questions `TWN-Q34`; decisions 188; quirks `TWN-QUIRK-15` (`-14` entered by T45,
  its rule's test with T48a; `-12`, `-13` by T35b); stubs `TWN-STUB-21` (`-11`, `-12` planned by §34; `-16`
  by T38b2, `-17`, `-18` by T39b1; `-20` lifted by T39b2); rules: TWN-TAV-001 to -009, GAM-MAN-003,
  GAM-DEC-010 and the adventure's ADV-VIS-007, -008 built; experiment batch `TWN-6` (`TWN-5` run
  by T44); disputes `TD-28` (`TD-25`, `-26` by T42a, `-27` by T42b); quirk candidates `T-19` (`T-18`
  by T42a, `fix-optional` by decision 169).
  ADR-0028 is the lane's; no drafts open; TWN-GLD-001 to -003 and -004's Aurora set built (T46,
  T47a1), planned -004's learning, -005 to -009, TWN-SET-012.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

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

### 2026-11-20 — T45: `shared/`, the drafts into the ADRs, the glossary, the security lines, `TWN-QUIRK-14`'s flag

Baseline 4771688 (#1076 merged; the setup's full baseline stopped at coverage for disk, its tests
passed); E08 T134a (#1077) merged meanwhile, the branch rebased onto b0efed6 before T45b. The
user's computer was connected at the start only: 64 files of `original-data/` and the four
manuals with three help files staged, SHA-256 checked. The user kept T45 whole with the
`cross-lane` consent (decision 183). Written: ADR-0013 15.5, 16.6; ADR-0016 point 4; ADR-0017
points 1 to 4, 8 to 10; ADR-0028 points 2, 5, 6, 9 with its Consequences and Alternatives; the
glossary; the security doc's guild section. Built: `quirk.town.aurora_ignores_map_bans` (44), the
profiles, the ledger's `TWN-QUIRK-14`; 101 goldens' `ruleset_hash`, the server literal, the bot
reports. PR #1080.
