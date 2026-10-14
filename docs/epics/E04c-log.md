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
- **The review** (BAT-Q40, the user): the draft accepted; old maps honoured (S-29 not a quirk);
  D-5 and D-6 as the code and `BALLIST.TXT`; the next cut T5 (the seam's `shared/` pull request
  with the ADR, towns on the experiment maps, walls, gate and moat).

## T5 — walls, gate and moat (2026-10-14, increment 3)

- **T5a, the seam** (`shared/siege-seam`, #183): `BattleSetup.siege: Option<Siege>`, `FortLevel`,
  `Moat`; ADR-0023 numbered from the draft. Asked while writing it (BAT-Q41): the faction by handle
  (`FactionId`, as `FactionGroup`); `structures` as the original counts them (0x5BF4F0: built
  buildings 0–43 among the town's buildable mask, the keep 4 + it); and, after the Tower's mines
  were read, the moat by **the game's version** (H3API's `mapKind`), not the map's format: only the
  saved-game loader's setter was found (0x4BCDA0), so how a game gets its version is the adventure
  lane's (an experiment candidate); a Tower's moat follows it like every town's. The battle lane's
  full constructors name `siege: None`; BAT-STUB-2. Full verify `--with-original-data` green.
- **T5b, towns on the experiment maps** (`battle/e04c-t5b`, `cross-lane` for `h3-map`): the
  writer's `Town` (class 98, its entrance two columns west of its position; owner or neutral,
  garrison, formation, standard or custom buildings; no name, events or spells, the owner's
  alignment); `[encounter.opponent.town]` (faction, owner, buildings by name, garrison, tight; the
  entrance north of the hero, an optional visiting hero on it); the building bits as VCMI 1.7.5's
  `gameConfig.json`. Read back by `h3-map`'s reader; the import reports the visitor in the town.
- **T5c, the reading** (`installation.md` §7.28): `WALLS.TXT` gives the 18 parts' hit points (2 for
  the towers, the four walls, the gate and the keep; a Castle's walls 3); the gate's states (3
  closed, 1 open) and its passage for the defender (95 and 94 free of stacks and corpses, 94 in
  every town), opening on a defender's entering 94–96 and closing after every move and action when
  96 and 95 (94 in a Fortress) are free; the moat's hexes, stop, damage on entering and on acting in
  place, −3 defense not floored; the Tower's ten hidden mines (at least 150 or the hero's Land
  Mine under SoD rules); the shot's wall test on a column/row grid line. VCMI agrees but for D-8
  (the open gate) and D-10 (the floor at 0); the wiki sides with the code on D-9. **BAT-Q42 (the
  user): the code on D-8 and D-10.**
- **T5d, the battle** (`battle/e04c-t5`, `cross-lane` for `h3-protocol` and the client's
  `h3-view`): the battle part's `siege` field in `[battle]` (each row's wall, moat and second moat
  hex, the gate, the defense penalty, the mines' power) and a `[[siege]]` record per town faction
  (the moat's damage, the second row, the mines' spell), both harvested from the executable's
  tables and five instruction operands (`h3_formats::exe::siege_data`, checked against their
  opcodes); `no_wall_penalty` for the Mage, Arch Mage, Enchanter and Sharpshooter (gap-fill,
  `exe-siege-walls`). The battle (`h3_battle::siege`): setup errors `UnknownSiege` and
  `InvalidSiege`; the walls and the closed gate block (the defender passes while the bridges are
  free of stacks and corpses), the gate opens on a defender's step onto 94–96 and closes after
  every walk, flight, teleporting move, action and turn; the moat's stops (not the bridges for the
  defender), its damage on entering (once per move) and on acting in place, its −3 defense (not
  floored, not for the frenzied); the Tower's ten hidden mines laid after the sight is set; the
  shot's halving on the column/row line in the original's x87 steps (`X87`, 1/17 of a cell), with
  the hero's bow exempt (its `NoRangePenalty`). A recording sink sees the gate open and close
  mid-walk, a quiet one only at the walk's end: the same state (tested). Versions: battle state
  hash v46, battle part hash v3, pack format v37, replay format v26, observation encoding v16
  (four globals, two planes), C ABI v20, protocol v30 (`wire::Siege`, three events). The
  synthetic pack gets the field and a record for the vale (the Fortress's shape); the scenario
  `siege` and the golden replay `siege-s7` (the defender out on both bridges and into the moat,
  the attacker in both rows); the other replays changed only their hashes and versions. Rules
  BAT-SIE-001 to BAT-SIE-008, 13 tests.
- **T5e, batch 3c run by an agent** (#190, `battle/e04c-t5e`, 2026-10-14): the spec
  `experiments/specs/E04c-batch-3c.toml` (predictions: the engine at b63cdb1 in a scratch test with
  the same armies and commands; the engine has no keep and no Catapult yet) and
  `experiments/agent/siege.py`, which plays a map in hot seat with both armies commanded (the town's
  garrison too): a plan per encounter (round, stack, action), every other turn Defend, the clock
  stepped 10 ms and the stacks, the gate, the walls and the mines read after each step; rounds by
  the stacks' turn flags (+0x4F0; counting the Catapult's turns missed some, its active pointer can
  last one frame). **M** (Inferno, Citadel): 90 for Sprites landing on a moat hex and for Magic
  Elementals walking in, 90 again on Defend there, a walk from 111 to 146 around 129 with 90 once,
  the Archangel's strike on the Sprites (defense 2) in the moat **127** (D-10; VCMI 125). **G**
  (Castle, Fort): the gate opens as a defender begins its step onto 96, stays open while it stands
  there, closes after its walk out; a stack on 94 stops a defender's step onto 96; the Titan's shot
  through the closed gate 17 (a quarter), through the open one 46 (half, D-8; VCMI a quarter).
  **T** (Tower, Citadel): ten hidden mines of the defender's, 150 each, on the moat hexes but 95;
  150 on 77, then 450 on the walk 77 → 61 → 44 → 28, which goes on. M and G gave the same values in
  three runs; T was played on its fifth attempt (bonus artifacts). Rules BAT-SIE-003 to 008
  `verified`, BAT-MAG-033 an observation; the survey's D-8 to D-10 confirmed; `installation.md`
  §7.28 "Observed". Seen for later tasks: the keep shoots at a round's start before the Catapult;
  the Catapult (no Ballistics) hit wall parts and not the gate in six shots (the gate is a target
  too, `BALLIST.TXT`; the user expects hits on it: T6 investigates); the original's walks pick other
  shortest paths than ours on ties (M2 along row 5, G4 via 94; no rule depends on it here); the
  gate opens before the step onto 96 where our engine reports it after the step (same state).

## T6 — the Catapult, Ballistics and Earthquake (2026-10-14, increment 4)

- **T6a, the seam** (#191, `shared/catapult-seam`): `CatapultControl { level }` (the hero's
  Ballistics level, 1..=3) in `h3-contracts`; `HeroEffect::Ballistics { level }` (code 42, pack
  kind `ballistics`) in `h3-content`, parsed by `h3-pack`, no ruleset hash bump (as #185's kinds);
  ADR-0023 point 6 amended (the battle sets the attacker's Catapult up itself, 0x463600), ADR-0013
  point 4's history; h3-pack's core test counts the synthetic file's creatures. BAT-Q43 (the
  user): the level in the seam, the code on D-11 to D-15, S-08's fix keeps the absent parts in
  Earthquake's draw and harms no stack, the Cyclops in T6. D-11 was then found to be no dispute
  (the record's index 4 is hex 78, as VCMI; 62 is the aim hex), corrected the same day.
- **T6b, the reading** (`installation.md` §7.29; map entries 0x445D30, 0x445EC0, 0x445FD0,
  0x473C00, 0x474BF0, 0x476440, 0x5A7C80, the tables 0x63B820, 0x63B840, 0x63BE60, 0x63D54C,
  0x670198, [0x679C84]; +0x13F60 the parts' hit points, +0x13FA8 their picture state, +0x53A4 0 a
  destroyed gate): the eight targets, the Catapult's stack and turn, its automatic aim, a wall
  shot (damage roll, then hit roll; a miss on the nearest other wall; re-aim), the parts' damage,
  Earthquake's draw; S-08 confirmed by the code.
- **T6c, the battle** (`battle/e04c-t6`, `cross-lane` for `h3-protocol`, `h3-map` and the
  adventure replays' headers): the siege field's wall shots harvested (`h3_formats::exe::
  wall_shot_data`: the targets, the miss, automatic and fallback lists, the Catapult's hex from an
  operand; `BALLIST.TXT`; each town's hit points from `WALLS.TXT`), the Catapult creature
  harvested, the Cyclops' `wall_shots` (gap-fill), Ballistics' effect, Earthquake's effect (pack
  format v38, battle part hash v4). The battle (`h3_battle::siege`): the Catapult set up for an
  attacker with a hero at a fort (slot 19, hex 119; never listed), first in its phase; automatic
  without Ballistics, commanded with it (`Command::ShootWall`, the Cyclops' too); the shot, the
  parts' hit points, a destroyed wall's hex, a destroyed gate (open for good: passage, no moat on
  the bridges, the line free); Earthquake (a siege with a fort, either side; the code's draw; the
  absent turrets soak hits, BAT-QUIRK-22). Versions: battle state hash v47, replay format v27,
  action encoding v13 (eight wall-shot entries), observation encoding v17 (the gate's destruction
  and the targets' hit points as globals, the Catapult's command), C ABI v21, protocol v31. The
  synthetic pack: a Catapult (sling), a wall shooter (hurler), Quake; the `siege` scenario's
  attacker led by Brea with Advanced Ballistics and Quake; `siege-s7` re-recorded (two walls and
  the keep by the Catapult, the gate by the Hurlers, two Quakes), the other replays changed only
  their hashes and versions; the adventure replays only the core ruleset hash (the base pack's
  Catapult creature); `h3-map`'s creature codes gain 145. Rules BAT-SIE-009 to 015, HER-BAT-018,
  17 tests in `tests/walls.rs`. The Catapult artifact's pending row moved to E08-E10.
- **T6d, batch 3d run by an agent** (`experiments/specs/E04c-batch-3d.toml`,
  `experiments/agent/catapult.py`, built on `siege.py`: plans by round or by state, the Catapult
  aimed under Ballistics, a hero's cast, the parts' hit points +0x13F60; rounds by the acted bits).
  **A** (Castle, Citadel, no Ballistics): nine automatic shots, every landing the weakest standing
  wall or its miss list's nearest, the gate only after the four walls fell, the keep after the gate
  (D-13); a walk onto 62 refused, onto 78 done (D-11); the destroyed gate stayed 0 and a walker
  passed 95 without a stop or damage. **B** (Castle, Fort, Advanced Ballistics): the game waited
  for the aim; misses on the nearest other wall, a second shot after its aim fell on the nearest
  standing target (D-12). **E** (Castle, Citadel, Expert Earthquake by both heroes): four hits a
  cast, the absent towers drawn, a doomed part drawn again (D-14), the defender's cast allowed
  (D-15), and the absent towers' destruction marked the defender's first stack dead (S-08). The
  game's draws repeat exactly from run to run. BAT-SIE-009 to 012, 014 and 015 `verified`.
