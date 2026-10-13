# E04c — log

The record of E04c's tasks: what each built and decided, beyond the one line in the epic file.

## T1 — the battlefield survey and the ADR draft (2026-10-13, increment 1)

- **The user's refinement** (BAT-Q37): battlefields first, the seam reviewed first, the wiki read.
- **Read** (`docs/formats/installation.md` §7.26): 0x4643B0 sets the terrain, the special field
  (0–9), the ship flag and the anti-magic garrison; morale and luck by field (0x43E000,
  0x43DF20, their only caller 0x469750); no native stack on Cursed Ground (0x43D7C1); the
  mastery raise (0x4E5100 for heroes, 0x59FEBC for the other casts); the obstacle choice by the
  field's bit or the terrain's and the two boats' 32 hexes (0x466290). The executable's obstacle
  tables equal VCMI's `obstacles.json` record by record.
- **Sources compared** (survey `docs/rules/surveys/E04c-battlefields.md`): the game's own texts
  side with the code on Cursed Ground's morale (D-1) and Clover Field (D-2); the wiki with the
  code on Magic Plains' creature casts (D-3); D-4 (the SoD objects 223 and 230) is the adventure
  lane's.
- **Draft:** `docs/lanes/battle/adr-drafts/battlefields.md`; the battlefield crosses the seam by
  key, as an obstacle's type (the second commit, before the review).

## T2 — the battlefields (2026-10-13, increment 1; built before BAT-Q38's answer)

- **T2a, the seam** (`shared/battlefield-seam`): `BattleSetup.battlefield: Option<ContentKey>`
  and `.anti_magic: bool`; the battle lane's full constructors name them (`cross-lane`); BACKLOG.
- **Content** (pack format v36, battle part hash v2): `[[battlefield]]` with one typed field per
  rule (`h3_content_battle::battlefield`), an obstacle type's `battlefields`. **Harvest:** the
  obstacle types from the executable (`h3_formats::exe::obstacle_data`, 91 + 34 records and the
  boats' 32 hexes), the battlefields' rules from the gap-fill file; the 78 existing types are
  unchanged but for their battlefields, 12 types join `base` (the deck's and the shore's) and 35
  `sod` (the SoD fields').
- **Rules:** BAT-TER-002 (the setup's key, public), BAT-TER-003 (morale and luck in the
  stack's own sum; zero replaces everything), BAT-TER-004 (the level limit joins the heroes',
  `no_creature_casts` beside the Orb's ban, the anti-magic garrison as the Orb), BAT-TER-005
  (`magic::mastery` and `magic::other_mastery` at cast time), BAT-TER-006 (`terrain::native_terrain`),
  BAT-GEO-004 (`generate_battlefield_obstacles`, `generate_setup_obstacles`; the old
  `generate_obstacles` stays for the server's caller).
- **Versions:** battle state hash v45, replay format v25, observation encoding v15 (two globals),
  C ABI v19, protocol v27 (the wire's observation and key tables; `cross-lane`). The golden
  replays play the same commands; a ninth, `field-s5`, plays the synthetic `hallow`.
- **Other lanes:** the client's asset tables moved the SoD obstacle types to `sod`; inbox lines
  in the adventure, client and server queues.
