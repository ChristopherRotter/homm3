# Town lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md`; for the adventure lane's crates, also the adventure lane's `TRAPS.md` (the group of the area you touch).

## Current state

- **E09** (towns): increments 1 to 3 closed (T10, T20, T30; `E09-archive.md`, design notes Parts A
  to E). **Increment 4** (the Tavern; decisions 98–153, `TWN-Q15` to `-Q26`): T31 to T39a done;
  **T39b cut in two** (decision 151): **T39b1 done** (the Thieves' Guild as a query,
  `h3_game::thieves_guild`; its resource rows the town part's, part v8, decision 152). **Next:**
  T39b2 (the Den of Thieves' visit, `ThievesGuildSeen`, lifting `TWN-STUB-20`; the protocol v74:
  the guild's request and the events; `QUEUE.md`), its cut confirmed at its start. The guild and
  the services are increment 5, travel and the Grail 6. Waiting elsewhere: `TWN-STUB-8` (the
  battle lane's inbox), `TWN-STUB-6`, `-13` and `-15` (increment 5), `TWN-STUB-2` (the victory
  conditions), `TWN-STUB-16` (E10's loss condition), `-17` (increment 6), `-18` (E13).
- **Open pull requests:** T39b1's (`town/e09-t39b1`, `cross-lane`), opened at the session's end;
  watch it until it merges.
- **Ownership** (`lanes.toml`, decision 3): `h3-towns`, `h3-pack-towns`, each pack's `towns.toml`,
  the town modules `src/town*` of `h3-adventure` and `h3-game`, `src/import/town*` and
  `src/writer/town*` of `h3-map`, `town.toml`, `quirks-town.md`, E09's epic, surveys and
  experiments. The rest of those crates is the adventure lane's: a change there is `cross-lane`,
  merged in order with its pull requests.
- **Versions:** the lane's own: the town state hash (`TOWN_STATE_HASH_VERSION`, v3) and the town
  ruleset part (`TOWN_RULESET_HASH_VERSION`, v8: T39b1), both in `h3-towns`; the adventure lane's
  (game state hash v34, the adventure part v22, map import v26 and replay format v19, all by
  T39a; adventure state hash v26 by E08's T124b1), the env's (v13 by T39a) and the protocol's
  (v73 by T39a: T39b takes v74) are bumped `cross-lane` (each task's record in `E09-log.md`; the
  plan in design §77). Shared: core ruleset hash v53, pack format v52 (T24),
  quirk flags 43 and `StreamId::TAVERNS` (8) by T35b.
- **Ids** (next): questions `TWN-Q27`; decisions 154; quirks `TWN-QUIRK-14` (`-12`, `-13` entered
  by T35b; their tests by T38b1, T39b1); stubs `TWN-STUB-21` (`-11`, `-12` planned by §34; `-16`
  by T38b2, `-20` by T39a, `-17`, `-18` by T39b1); rules: TWN-TAV-001 to -009, GAM-MAN-003,
  GAM-DEC-010 and the adventure's ADV-VIS-007, -008 built; experiment batch `TWN-5`; disputes `TD-25`; quirk candidates `T-18`.
  ADR-0028 is the lane's; no drafts open.
- **Working agreement:** a domain lane: a user checkpoint per increment and the cut confirmed at
  each round; the user confirms seams and ADRs; tokens first.

## Standing traps

In `TRAPS.md`.

## Recent sessions

### 2026-11-19 — T39b1: the Thieves' Guild as a query

Baseline 450e47e (#1030 to #1032 merged; CI's runs on `master` standing in: a VM restart killed the
setup's baseline). The user's computer was connected at the start only: 28 files staged (the four
LODs, both executables, the DLLs, both `.snd`, two `.vid`, both tars, the saves, the manuals, two
help files; SHA-256 checked); its tools went away mid-session. The user cut T39b in two and
consented to `cross-lane` (decision 151), then put the guild's resource rows in the town part
(decision 152, over the agent's core order). Built: design §73's As built (`town/thieves.rs`; part
v8 `guild_resources`, harvested; `army_value`, `power`, `primary_skills`, `day_income` made
public; the 98 goldens' `town_ruleset_hash`; nine guild tests). `master`'s CI on 450e47e was red
(an `h3-map` plan property, the adventure lane's: its inbox has the seed).

### 2026-11-18 — T39a: the map's Tavern and Den of Thieves, the tavern decision

Baseline 34dddd8 (#1018 merged; CI's fingerprint on `master`, the setup ran the light gates).
The user's computer was connected at the start only: 28 files staged (the four LODs, both
executables, the DLLs, both `.snd`, two `.vid`, both tars, the saves, the manuals, two help
files; SHA-256 checked). The agent recommended the Tavern alone (the Den to T39b); the user kept
T39a whole as queued and consented to the `cross-lane` PR (decision 149). Built: design §66's
and §74's As built (the two kinds, part v22, import v26; `Decision::Tavern`, game state hash v34;
`map_hire` through the town hire's `place`; protocol v73, env v13, replay format v19; the 98
goldens and the bots tables re-recorded); every workload inside its gate; the scoped verify and
the server scope's coverage clean (a fold test added). PR #1030.

### 2026-11-18 — T38b2: the dismissal

Baseline 8625fed (#1013, #1014 merged; CI's fingerprint on `master`, the setup ran the light
gates). The user's computer was connected at the start only: 16 files staged (the four LODs, both
executables, both tars, the saves, the manuals, two help files; SHA-256 checked). The agent
recommended a cut in two as T38a's; the user kept T38b2 whole (decision 147). Built: design §69's
As built (`Manage::Dismiss` with `NotOnMap`, `InTown`, `LastHero`; `town::leave::dismiss` and
`check_dismissal`; the adventure's `remove_hero`; the listing; replay format v18 `dismiss
<hero>`, protocol v72 with a fold test; the plans dismiss in one `Step::Manage` of sixteen, three
oracles taught); GAM-MAN-003 `verified`, `TWN-STUB-16`; every workload inside its gate.
