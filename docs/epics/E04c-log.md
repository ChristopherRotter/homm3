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
  C ABI v19, protocol v28 (the wire's observation and key tables; `cross-lane`). The golden
  replays play the same commands; a ninth, `field-s5`, plays the synthetic `hallow`.
- **Other lanes:** the client's asset tables moved the SoD obstacle types to `sod`; inbox lines
  in the adventure, client and server queues.
- **The review** (BAT-Q38, the user, after T2 was built): the draft accepted as ADR-0022 (through
  the seam's pull request); the battlefield stays optional, `None` the terrain's ordinary one;
  D-1 as the code with quirk candidate S-28 (the intent may have been only non-positive morale,
  `fix-optional`, to be investigated with T3's row); D-2 as the code with the alignment
  distinction noted (BAT-TER-003); the cut as proposed, both pull requests `cross-lane`. T1
  merged as #164 meanwhile. No code changed after the answers but the doc comments.

## T3 — the battlefield experiments (2026-10-13, increment 2)

- **The user's cut** (after #170, #171): T3's preparation, then T4; the writer's change from the
  battle lane as `cross-lane`; the batch played by an agent on the client lane's capture harness
  (the first battle batch so). CLT-Q1 had been built meanwhile by another session (#169, wire 29).
- **Writer** (`h3-map`): `MapObject::Ground`, the special grounds (classes 21, 46, 222–231)
  without a body, written first; the coast flag `0x40` on land next to water (every one of the 164
  shipped maps has it on exactly those tiles; without it batch 3b's first run had no shore).
  **Compiler** (`h3-experiment`): `[[map.ground]]` (picture, bottom-right tile; every tile of the
  pictures is drawn, their `.msk`), water patches, `--only <id>` (one encounter's map in its place).
- **The harness** (`experiments/agent/opening.py`): opens each map's battle under Wine and reads
  the opening from memory: morale +0x4E8 and luck +0x4EC (0x469750 fills them at the start and
  after every action), speed, native flag, the field +0x53C0, the terrain, the hero's worn
  artifacts. Cross-check: batch 1a's R1 replayed (`crosscheck_r1.py`) gave the user's 105 to 630.
  Trap: the single scenario's random starting bonus gives the hero an artifact a third of the
  time (the first TM run read morale +2); such a battle is played again.
- **3a** (predictions: the engine at 3a4d9ff, as the code): all as predicted. Cursed Ground:
  morale and luck 0 for every stack, the army's −3 included, no native bonus (D-1; S-28 observed,
  not a quirk: the user at BAT-Q39); the SoD pictures 223 and 230 fight on Cursed Ground and Magic
  Plains (D-4); Clover Field +2 luck for Goblins, Gnolls, Pixies and none for Peasants (D-2, VCMI
  refuted); Holy Ground +1 Castle, −1 Inferno; Magic Plains and Lucid Pools as fields 1 and 6.
- **3b:** the defender's tile decides: a monster on Clover Field (TM) or next to water (SHM) gives
  the field, the hero's tile (TH, SHH) does not. For the adventure lane (its queue's inbox).
- **Docs:** BAT-TER-003 and BAT-TER-006 `verified`, BAT-TER-002 an observation; the survey's D-1,
  D-2, D-4 settled and §4's tile; `installation.md` §7.26 (where morale and luck are stored);
  `structs.toml` (the stack's luck, the hero's worn artifacts); `h3m.md` (the coast flag).

## T4 — the siege survey and seam (2026-10-13, increment 2)

- **Read** (`docs/formats/installation.md` §7.27; the wiki with the user's approval): the town is
  stored for every battle at a town, the fort level beside it (1 Fort, 2 Citadel, 3 Castle); the
  moat flags (a Citadel or Castle, no Tower, a Stronghold only on SoD maps) and the moat's hexes
  and damage (only on SoD maps); the attacker's Catapult and no defender's Ballista at a fort; the
  turrets as Arrow Tower stacks whose count is 4 + the town's counted buildings (a tower half of it);
  the defending hero's town bonuses for the battle, restored at its end with the mana capped; the
  defenders' morale from the Tavern and the Brotherhood of the Sword; surrender never and retreat
  only with the Escape Tunnel for the besieged; +500 experience for the attacker; no natural
  obstacles in a siege but on an older map's Stronghold.
- **Survey** (`docs/rules/surveys/E04c-sieges.md`): the parts by fort level, the rule areas against
  `BALLIST.TXT`, `CRTRAITS.TXT`, VCMI and the wiki; D-5 (a tower's count for an odd N: the code
  and the wiki against VCMI), D-6 (Expert Ballistics: the table against the wiki's "always hits"),
  D-7 (a town without a fort is still a town battle), S-29 (the map format's branch, not a bug).
- **Draft:** `docs/lanes/battle/adr-drafts/sieges.md`: `BattleSetup.siege: Option<Siege>` with
  derived facts, the hero's bonuses before the seam, no outcome field. The review is BAT-Q40.

