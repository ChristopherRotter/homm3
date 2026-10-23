# E11 — Historical rules of RoE and AB

Status: **refined** 2026-10-24 (Q58), not started · Opens after E23's closure and one battle-lane
session on its inbox's leftovers (the user, 2026-10-24), with the user's original media (T1) ·
The battle lane leads (the executable's tools, `installation.md`, `h3-harvest`, the packs); each
difference is built by the lane that owns its rule (BACKLOG).

## Goal

A game plays the rules of the edition its packs make up, as that edition's own executable plays
them: `base` alone plays The Restoration of Erathia's rules at its latest patch, `base` + `ab`
Armageddon's Blade's at its latest patch, all three The Shadow of Death's (Complete 4.0, today's
only source). Content already follows the packs (Q15: historical, latest patches). Until now the
rules came from the SoD executable only, so an RoE-only game had RoE's content but SoD's
rules. E11 reads the two older executables where they differ from the SoD one. It then puts
each difference that matters into the packs as data, so the enabled packs decide it, and builds
it.

## Decisions at the refinement (2026-10-24, the user, Q58)

1. **"AB and SoD completeness" is folded, like E07.** The sketch's content and rules were built
   epic by epic: the `ab` and `sod` packs (E02, E04a, E04c), the AB/SoD creature abilities that
   E03 deferred here (built in E04b), the combinations' battle effects (E04a), RoE/AB/SoD maps
   (E08 increment 1), the expansions as the game's configuration (E08 decision 104). The rule
   from now on: **every epic covers RoE, AB and SoD for what it builds.** The leftovers have
   owners: the combinations' assembly and the fifth misc position (E08 increment 6), Conflux as
   a town and the moat that differs by expansion (E09), the AB/SoD map objects (E12), the AB and
   SoD campaigns (E20). A test matrix over the pack sets (`base`, `base`+`ab`, all three) is a
   shared inbox item (BACKLOG).
2. **E11 is re-scoped to the historical rules** (the user: "historical rules, and we find a way
   to source truthful executables"): an RoE-only or AB-only game reproduces that edition's
   executable, not SoD's engine with fewer packs.
3. **The latest patch of each edition** (matches Q15's packs): two more executables, RoE's and
   AB's at their latest official patches. Earlier patch levels are no profiles of their own.
4. **The edition follows the enabled packs** (E08 decision 104 extended): no separate setting,
   no combination the original never had (SoD content under RoE rules). A difference is data
   that the `ab` and `sod` packs override; a rule reads only the compiled ruleset. The quirk
   profiles (`faithful`, `strict-fixed`) stay orthogonal: each edition's slips get classified
   like SoD's.
5. **The sources are the user's own original media** (Q58): discs or installers the user owns.
   Where an installation is older than the latest patch, the publisher's official patch is
   applied to it. Everything is installed in a folder of its own, never into the GOG
   installation (read-only, AGENTS.md). The copies live in the gitignored `original-data/`;
   nothing original is committed (ADR-0009). A build counts as "truthful" when its version
   resource, link date and size match the edition and patch it claims and its hashes are
   recorded in `installation.md`.
6. **The battle lane leads.** It owns `h3-formats`, `h3-harvest`, the packs and `installation.md`
   (the executable's address tools are shared), so the sourcing, the diff, the survey and the
   ADR draft are its tasks. A difference in another lane's rule goes into that lane's queue as
   a task after T3's checkpoint.

## Starting point

- **One executable today:** `Heroes3.exe` 4.0, SoD (`installation.md` §1; SHA-256 `aac39e8c…`).
  `scripts/exe.py` refuses any other build, and `docs/exe/map.toml` (about 1 150 addresses) names
  SoD's addresses only.
- **Content is historical already:** `base` is RoE at its latest patch, read from the Complete
  edition's tables plus the gap-fill's labelled facts (`crates/h3-harvest/data/gapfill.toml`:
  RoE patch 1.3's Lizardmen and Dragon Flies, SoD's Angel and Cerberus changes). RoE's own
  tables would replace gap-fill facts with original data.
- **Rules are SoD's:** every code reading (`installation.md` §7, the battle and adventure
  catalogs) is SoD's. Experiments run in SoD only (Q28), so no RoE- or AB-only behaviour has been
  observed.
- **Where a pack set reaches the rules today:** the SoD pack's combinations and fifth misc
  position (E08 decision 104), the AB/SoD overrides of creatures (the elementals, Cerberus).

## Scope

- Sourcing and cataloguing the RoE and AB executables and data at their latest patches.
- A function-level diff of each against the SoD executable and its map, and a diff of their
  static tables and text tables.
- A survey of every changed function that touches a rule the engine has (battle, heroes,
  adventure, game), classified as an edition difference, a patch fix, a slip or irrelevant
  (interface, campaign, network). The wiki's "in RoE …" notes are cross-checked against it.
- An ADR draft: how an edition difference is data in the packs and how the rule reads it.
- Re-harvesting `base` and `ab` from their own editions' data where that replaces gap-fill
  facts.
- The differences built, each a rule with the old executable's reading as its source and a test
  per pack set.

**Non-goals:** earlier patch levels; HotA, WoG and HD; a game's map edition as an input
(decision 104); campaigns (E20); the original's random map generator (an AB feature: E14's
refinement decides whether to port it, Q58); interface-only differences.

## Tasks (outline; refined task by task, a user checkpoint at T1, T3 and T4)

- **T1 Sourcing and catalogue** (with the user). The user lists their media. Each edition is
  installed into a folder of its own and patched with official patches to its latest level. The
  executables, the map editor and the data archives are copied into `original-data/roe/` and
  `original-data/ab/`. `installation.md` gains a section per edition: distribution, version
  resource, link date, sizes, SHA-256, archive layout. Each claim is checked against labelled
  public version lists. The user also says whether those editions run on their machine, which
  decides whether experiment batches can be played in RoE and AB as well as SoD (Q28). Stop for
  the user.
- **T2 The diff.** `scripts/exe.py` learns to read the other builds (by hash; a `--build` choice;
  a shared change). A matching of functions from SoD's map to each old build: the same compiler
  family, so normalised instruction sequences and the call graph are expected to match most
  functions. T2 settles the method on a sample first. Output, kept out of git as the listings
  are: unchanged, changed, missing and new functions per build, plus a committed summary by
  address and our names. The executables' static tables (creatures, heroes, artifacts, spells,
  skills) and the text tables are diffed field by field.
- **T3 The survey** (`docs/rules/surveys/E11-editions.md`). Every changed function that a rule of
  the catalogs rests on, or that a planned epic will need, is read and classified (decision 4 for
  slips). The wiki's edition notes and VCMI's (it models SoD only, as far as known) are
  cross-checked. Each difference gets an owner lane and an estimate. Stop for the user: which
  differences to build, and how each slip is classified.
- **T4 ADR draft** in `docs/lanes/battle/adr-drafts/`. It covers: the edition as pack data
  (per-rule data fields or toggles that the `ab`/`sod` packs override, against one edition value
  compiled from the packs), its place in the ruleset hashes, its effect on the import (an RoE
  map played with only `base`) and on the AI lane's encounters (SoD stays their default). It
  amends ADR-0006 (E02's expansions) and ADR-0017 point 1 (decision 104). Stop for the user's
  review.
- **T5 Re-harvest** `base` from RoE's own tables and executable, `ab` from AB's. Gap-fill facts
  that original data now answers are retired, and Q15's version balance is confirmed or
  corrected (a checkpoint if any value changes).
- **T6 onwards: the differences built**, battle first in this lane, the others as queue entries of
  their lanes. Refined after T4.

## Exit criteria

1. Both builds catalogued with hashes; `scripts/exe.py` reads all three.
2. Every changed function the diff found is classified in the survey.
3. The ADR accepted; every difference the user chose to build is a rule with its source (the old
   executable's reading or an observation), built, and tested with each pack set it differs in.
4. `base` and `ab` harvested from their own editions where they have data; the gap-fill's
   remaining facts still labelled.
5. Full `cargo xtask verify` green, including `--with-original-data` for all three editions.

## Risks

- **Sourcing.** The user's media may be at an older patch, and official patches may be hard to
  find. Old installers may not run on Windows 10. T1 may extract files with an archive tool
  instead of running the installer. Only official files are used, never a cracked or modified
  executable.
- **Changed behaviour for `base`-only games.** Tests that compile `base` alone (`h3-ffi`'s
  scenarios, some ruleset tests) will change once a difference is built: each change goes
  through the owning lane's checkpoint. The AI lane trains on SoD, which stays as it is.
- **Cost.** Two more executables multiply the code reading. Decision 3 and T2's diff keep the
  reading to what changed. T3's checkpoint keeps the build to what matters.
