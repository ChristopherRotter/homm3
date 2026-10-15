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
  format v39, battle part hash v4). The battle (`h3_battle::siege`): the Catapult set up for an
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
- **Master merged, the full verify, the pull request** (2026-10-14, sixty-second session). #191
  merged `master` (#192, #194): #194's `raises` took hero-effect code 42 and pack format v38, so
  `ballistics` is code 43 and T6's pack format v39 (labels only). The six adventure replays were
  re-recorded with their options on master's games: only the core ruleset hash changed. The full
  verify `--with-original-data` (Linux sandbox; 164 of 164 maps read) failed on two steps, both
  fixed: coverage (the wall shot's and Earthquake's validated `if let`, whose implicit `else` LLVM
  counted, now `into_iter().for_each`) and deny (rustls 0.23.45's advisories RUSTSEC-2026-0346 to
  0349, new and on `master` too: Cargo.lock takes 0.23.46). The user consented to the `cross-lane`
  pull request as one (2026-10-14).

## T7 — the arrow towers (2026-10-14, increment 5)

- **T7a, the reading** (`installation.md` §7.30; the survey's D-16 to D-19; map entries 0x465AD0
  `turret_shot`, 0x443D90 `stage3_protection_factor`, 0x474BF0): the turrets' stacks (0x463600:
  the keep at 254 with `4 + structures`, with a Castle the towers at 255 and 251 with half,
  rounded up; their records name each turret's stack), the turn (0x464F50: the turrets first,
  then the Catapult), the control (0x474BF0: the Ballista's case, Artillery), the shot (0x465AD0:
  `random(2, 4)` per creature, summed, times the target's stage-3 protection factor 0x443D90,
  truncated, at least 1; nothing else; one shot), what reaches a turret (0x44A4D0: immune to
  every spell; off the field; berserk and Magic Mirror pass it over). The code contradicts the
  wiki's S-06 (Armorer, Air Shield and petrification raise the damage) and the Archery page's
  S-07. VCMI: one uniform draw through the full formula. **BAT-Q44 (the user, with the wiki read
  this session): the code on D-16 to D-18 with confirmation rows; the Ballista's automatic target
  (VCMI) with observation rows (D-19); turrets as off-field stacks; no seam change.**
- **T7b, the battle** (`battle/e04c-t7`, `cross-lane` for `h3-map`, the adventure replays'
  headers, `h3-view`'s table test and the lanes' queues): the Arrow Tower harvested (the gap
  fill's war machines, ability `turret`, pack format v41); the siege field's targets hold their
  turrets' positions and the keep's least count, read from the executable (0x465990's cases,
  0x463600's operands, 0x5BF4F0; `h3_formats::exe::wall_shot_data`; battle part hash v5). The
  battle: the defender's turrets after its war machines (slots 20 to 22; `Stack::is_turret`), the
  targets hold them (`SiegeState::turret`); first in their phase; commanded with Artillery as the
  Ballista (Shoot, Wait, Defend; one shot), else the Ballista's automatic shot; the shot
  (`combat::turret_shot`, the protections only, `Strike` with attack and defense 0); a fallen
  tower's or keep's turret dies without a corpse; `may_attack`, `affects_stack`, berserk's
  candidates and the Magic Mirror's redirect leave turrets out; the outcome and the surrender
  price too. State hash v48, observation encoding v18 (a turret flag per stack), C ABI v22; no
  wire change. The synthetic pack: a turret (spire), the siege scenario's town with three
  structures (a keep of seven): `siege-s7` re-recorded (the attacker still wins), the other
  replays changed only their hashes; the adventure replays only the core ruleset hash. Rules
  BAT-SIE-016 to BAT-SIE-019, 8 tests in `tests/turrets.rs`.
- **T7c, batch 3e run by an agent** (`experiments/specs/E04c-batch-3e.toml`,
  `experiments/agent/turrets.py`, built on `catapult.py`: a turret's shot read as its target's drop
  of hit points while the turret is the last activated stack, +0x132B8/+0x132BC). N = 5 (one
  structure of the writer's towns), towers 3. **K**: the keep, the upper and the lower tower
  first every round, then the Catapult; shots within 2N..4N and 2M..4M. **AS**: Expert Air
  Shield, 9, 7, 7, 7, 8 (N..2N). **AR**: Expert Armorer, 10 to 14, mean 12.3 (× 0.85). **AC**: the
  defender's Expert Archery, 10 to 17, mean 15.2 (unchanged); the keep automatic; destroyed by
  the Catapult in round 10, it shot no more. **AT**: Expert Artillery, the game waited for the
  keep's aim, one shot a turn, a Wait accepted (the keep acted again after every other stack);
  later draws varied between runs with a human command. So S-06 and S-07 are refuted in the
  user's version (not quirks), D-16 to D-18 as the code; BAT-SIE-016 to 018 `verified`.
  **AI (D-19)**: the automatic keep shot the single Marksman before the ten, the wiki's computer
  rule and not the engine's (the automatic Ballista's, VCMI's): BAT-Q45 for the user.
- **Master merged twice; the pull request; the checkpoint.** #205 took pack format v40 and core
  ruleset hash v44, so T7's pack format is v41; the replays re-recorded (only the core ruleset
  hash moved). The full verify with the original data green on b164922 after two coverage
  fixes (a keep or tower without a turret; stale profiles of the previous build); `cross-lane`
  PR #209. The user confirmed T7 and answered BAT-Q45 with (b): the computer's target choice for
  shooting war machines is read next (T7d).

## T7d — the automatic turret's target (2026-10-14, increment 6)

- **The reading** (`installation.md` §7.31; map entries for 0x41E190, 0x465F20, 0x435EC0,
  0x41EAC0, 0x442E60, 0x442FD0, 0x422B20, 0x448840, 0x448BD0, 0x443080 and the general shooter's
  0x41F060, 0x41EB80, 0x435CB0): an automatic arrow tower does not use the computer's shooter but
  0x465F20 → 0x41E190 with the damage `3n` (the mean of its draws; not the wiki's "damage 1").
  Each attacker record scores the kills of that damage (through the target's protection) times
  the computer's worth of a creature (0x442A50: the Fight Value times the root of an offense and a
  defense factor from the bonuses over the field's least, for a war machine or created stack its
  share of the army's hit points), divided by its turns until it can attack (1 to 5, from the
  computer's pathfinder: the moat's stop costs the rest of the turn); ties to the later record; a
  desperate mode (every creature worth 1000) when the side's strength doubled is below the other's
  and the difficulty is above Easy or the side human. The wiki's classes and tie agree; VCMI has
  no turret routine. The automatic Ballista goes through the general shooter (T7e).
- **BAT-Q46 (the user): the turrets fully.** The seam `shared/e04c-t7d-seam` (`cross-lane`):
  `BattleSetup.difficulty`, `BattleSetup.human` (ADR-0023 point 7; the defaults 0 and no human),
  `X87::sqrt` (the runtime's `sqrt` at 53 bits, tested against IEEE doubles), the literals; FYI
  notes in the adventure and server inboxes. A commanded Ballista's Artillery factors need the
  hero's level, which the seam lacks: BAT-STUB-3, with T7e.
- **The battle** (`battle/e04c-t7d`, `cross-lane`): `aim.rs` (the context, the worth, the
  strength, the AI's reach by Dial's buckets without allocation, the turns, the score, the
  target), called by `war::act` for a turret; `Strike::protection` shared with the shot's stage 3;
  the turret's branch of `shot_damage_range` gone (unused). The setup's facts are state (hash
  v49), observed (encoding v19: three globals; C ABI v23), on the wire (protocol v32:
  `Observation.difficulty`, `SideView.human`; `h3-view`'s literal), in scenarios (`difficulty`,
  `human`) and replays (format 28). Golden replays re-recorded: `siege-s7`'s keep aims anew (the
  attacker still wins, in 104 commands), the others' hashes only. Rules BAT-SIE-020 to 023; 9
  tests in `tests/aim.rs` (the test ruleset with Fight Values, `fighting_turret_ruleset`).
- **Batch 3f run by an agent** (`experiments/specs/E04c-batch-3f.toml`, `turrets.py`'s plans; the
  engine's predictions from the base pack): **FV** the Magogs before the Medusas (the Fight Value,
  not the AI value), **TU** Royal Griffins two turns away before a Giant behind the walls (the
  turns), **DE** the Master Gremlins before the Magogs against one Pikeman (desperate) and **DN**
  the Magogs against fifty, **BA** the attacker's Ballista before a hundred Zombies (a war
  machine's share of its army); every round, every encounter as the code. BAT-SIE-020 to 023
  `verified`. Found on the way: the original's records put the siege's Catapult before the hero's
  Ballista (BA), the engine the Ballista first (the battle queue's inbox).


## T8 — the siege's other rules (2026-10-15, increment 7)

- **The user's choice** (2026-10-15, before going offline): T8 before T7e (the besieged side's
  wrong legal commands weigh most for training); the wiki read this session; a dispute decided by
  the code with a batch; the wire `cross-lane` if needed (it was not).
- **The reading** (`installation.md` §7.32; map entries 0x467490 new, 0x469750 read whole, 0x5A3700
  and 0x5A7080 completed): the controls (0x4782D0: no surrender for a town's defender, retreat only
  through a Stronghold's building 17); the town's morale and luck added last to the base values
  for the stacks the defender *controls* (0x469750 recomputes every stack from its controller after
  every action: an inbox item for Hypnotize); the experience (0x46A350: +500 for a town's attacker,
  −500 after a retreat or a surrender; the orchestrator's); the tactics phase tests only the bank
  flag (a siege has one); Teleport (0x5A3700: below Advanced not across the moat, the moat hex
  inside; below Expert the shot's line test); a created stack's place passes the closed gate for
  the acting side, a summon there opens it (0x5A71D7), a clone does not; the spell obstacles'
  tests refuse the wall hexes' blocked bit (the engine let them on walls: a fix); the spell cost
  reads the battle start's armies (nothing siege-specific). D-20: Teleport's tests against VCMI's.
- **The battle** (`battle/e04c-t8`): BAT-SIE-024 (`siege::may_retreat`, `may_surrender`; the
  tunnel in `SiegeState`, state hash v50), BAT-SIE-025 (`morale::town_modifiers`, the pool two
  larger), BAT-SIE-026 (`siege::teleport_allowed`, the second target's test now takes the mastery;
  the masteries in the siege field: `teleport_moat`, `teleport_walls`, harvested from the
  executable's two compares, pack format v42, battle part v6), BAT-SIE-027 (`siege::walled` in
  `obstacles::free` and `random_hexes`), BAT-SIE-028 (`BattleState::blocked_as` with the caster's
  side; the summon calls `siege::entered`). BAT-FLE-001/002, BAT-MAG-027/029/032/038/039,
  BAT-TAC-001 (a town keeps its tactics phase: the catalog said otherwise, the engine never did)
  and BAT-END-003 (the experience at a town) amended; BAT-STUB-2 closed. Tests: `tests/town.rs`
  (9). The siege scenario's town has morale and luck 1: `siege-s7` re-recorded (224 commands, the
  attacker wins); the other battle replays and the two adventure replays with battles (`cross-lane`)
  changed only their hashes. No wire, encoding or observation change: the controls were public
  already, and the tunnel shows in them.
- **Batch 3g run by an agent** (`experiments/specs/E04c-batch-3g.toml`, `experiments/agent/town.py`,
  new: the opening's morale and luck, the battle window's buttons at every commanded turn, a cast
  with its clicks tried in order): **CM** a Castle's defender: Surrender and Retreat disabled at its
  turns, enabled at the attacker's; **ET** a Stronghold's with the Escape Tunnel: Retreat enabled,
  Surrender disabled; **RL** a Rampart's with the Fountain of Fortune: luck +2; **TB** Basic Teleport
  refused the inside hex 64 and the moat hex 61 (VCMI would accept 61: D-20), **TV** Advanced
  refused 64 and landed on 61 without the moat's damage (then 70 for each turn there), **TE** Expert
  reached 64; **FF** Force Field refused 95 (its shape reaches the wall 78) and the gate's 96. Every
  value as the code. **D-21 found:** CM's defenders had morale +1, not +2: read back from the town
  record, the set +0x150 the Tavern's test reads holds the Brotherhood of the Sword *instead* of
  the Tavern (an upgrade replaces its predecessor there, as the Citadel the Fort), so a Castle with
  the Brotherhood gives +2 in all; VCMI agrees (`upgradeReplacesBonuses`), the wiki's "in addition"
  is refuted. A fact for the town context's `Siege.morale` (the adventure lane's inbox; the battle
  adds what it is given). BAT-SIE-024 to 027 `verified`; 028 stays `implemented` (a summon or clone
  at the gate needs a crowded inner yard).
- **The independent check** (a fresh agent over the diff against §7.32): no problem; it found that
  Teleport's fit test let the closed gate pass by the stack's controller, where 0x469A10 reads the
  acting side (a hypnotized stack teleported by its owner): fixed (the caster's side), with a test.

## T7e — the automatic Ballista's target (2026-10-15, increment 8)

- **The choice** (the user, 2026-10-15: "the next sensible increment"): T7e before T9, as T9
  measures the epic's final code; built and confirmed at the checkpoint.
- **The reading** (`installation.md` §7.33; map entries 0x4221F0, 0x477EE0, 0x41E570, 0x4359B0,
  0x442780, 0x41EDA0, 0x41EEA0, 0x63B810 new, 0x41EB80, 0x435B90, 0x443160 completed): a war
  machine acting on its own goes through the computer's action (0x4744C0 → 0x477EE0 → 0x4221F0),
  which sends the Ballista to the computer's shooter 0x41F060: the context and threat records as
  the turret's (§7.31), then 0x41EB80 over the other side's records but the dead and the arrow
  towers: a stack that keeps its turns before one that loses them, the best 0x435CB0 value, ties
  to the later record; no preference for creatures, no controller test. The value: the kills of
  the computer's damage estimate (0x4359B0, 0x442780: n × d from the record's damage, the physical
  damage without draws, at least 1, x2 for a double shot, capped) times the target's worth (1000
  in desperate mode), whole on Easy for a computer side, else / 10 (disabled), / 5 (tent, cart, no
  threat record, more than 5 turns), / turns. The Ballista's worth: doubled above Basic Artillery,
  then x1.0, 1.5, 1.5, 2.0 by level (0x442C0F, 0x63B7F0).
- **The seam** (`shared/e04c-t7e-seam`, #225, `cross-lane`; ADR-0013 point 4): `BallistaControl.level`
  and the `artillery` effect's `level` (core ruleset hash v45, pack format v43, replay format v29,
  protocol v33); the 18 golden replays re-recorded, only hashes and headers moved.
- **The battle** (`battle/e04c-t7e`): `aim::ballista_target`, `shot_value`, `estimate`
  (BAT-WAR-008), called by `war::act` for the automatic Ballista, aimed at the target's position;
  VCMI's `shooting_target`, `value`, `can_attack_next_turn` and `movement::reachable_in_public`
  gone; `combat::shot_damage` (a real shot's stages from a given base; `shot_damage_range` is the
  tests' now); the Ballista's Artillery factors in `aim::worth` (BAT-SIE-021, BAT-STUB-3 closed);
  `hero::check` refuses a level outside 1..=3. State hash v51 (the level hashed), observation
  encoding v20 (the own hero's Artillery level, the last scalar), C ABI v24. BAT-WAR-002 (the
  target moved out), BAT-WAR-005, BAT-SIE-021 to 023, BAT-MAG-031 amended. Tests: 11 in
  `tests/aim.rs`, the level's bounds in `tests/heroes.rs`, a property (the Ballista shoots a stack
  of the other side, one that keeps its turns while there is one) replacing the lazy-ranking one.
  The siege scenario's attacker brings a Ballista she does not command: `siege-s7` re-recorded
  (the attacker wins in 89 commands); the other battle replays and the two adventure replays with
  battles moved only their hashes. bench-check green without a new baseline.
- **Batch 3h run by an agent** (`experiments/specs/E04c-batch-3h.toml`, `experiments/agent/ballista.py`,
  new: the open-map battle on `turrets.py`'s play): **TI** two stacks of ten Archers: the later
  both rounds; **TU** one Archer, then three Behemoths; **DE** a weak human side (desperate): five
  Archers before two Titans; **DN** a strong side: the five Archers too; **AR** Blue's Ballista
  under Expert Artillery before thirty Zombies; **AN** without Artillery the Zombies. **The port's
  mistake found:** the first port estimated a Ballista's shot from the range its hero's attack
  makes of 2..3 (the engine keeps that range on the stack, BAT-WAR-005), and predicted the
  Behemoths in TU and the Titans in DN. The original's record keeps 2..3; its hero's multiplier
  applies only in a strike's base damage (0x443160 at 0x4431AC), and 0x442780 reads the record:
  the estimate of one Ballista starts from 2. Fixed (`aim::damage`, also for the worth's Bless and
  Curse ratio of a war machine), every shot then as the code; VCMI's rule refuted by TI, DE, DN
  and AR. BAT-WAR-008 `verified`; BAT-SIE-021's Artillery factor observed from below (AR, AN).
- **The independent check** (a fresh agent over the diff against §7.33 and the code at 0x41F060,
  0x41EB80, 0x435CB0, 0x435B90, 0x4359B0, 0x442780, 0x442C0F): no faithfulness finding. It found
  that the hero's private Artillery level shows through choices both sides see (BAT-Q47: the user
  applied BAT-Q26's rationale, the level stays private and the other side infers it); that
  `aim::ftol` saturated where the runtime's `_ftol` (0x6179D4, `fistp qword`) keeps the low 32
  bits (fixed, also for the turrets); that a hero's changes to a war machine's damage would have
  been left out of the record's range (fixed: the stack's range less the hero's share); and two
  untested branches (a later disabled stack of a higher value, more than five turns, a clone in
  desperate mode: tests added). Left as is: the desperate rerun recomputes the turns (rare).
- **Not tested:** Easy for a computer side (hot seat makes both sides human; the engine's default
  setup, difficulty 0 and no human, is that case: the Ballista's values are then not divided); a
  stack that loses its turns (/ 10); a hypnotized candidate.


## T9 — the workload and the measurements (2026-10-15, increment 9)

- **The choice** (the user, 2026-10-15: "the next sensible increment"): T9, refined at its start
  (no earlier refinement): T9a the invariants where the epic's battles are fought (exit criterion
  4: until now every generated game of `properties.rs` ran on the ordinary field, without a town),
  T9b the workload and the measurements (exit criterion 5); T10 after the checkpoint.
- **T9a, the places** (`tests/properties.rs`): a generated game's `Ground` holds its natural
  obstacles and its `Place`: two in seven on the terrain's ordinary field, one in seven on one of
  the test ruleset's eight special battlefields, one in seven at an anti-magic garrison, three in
  seven at a town (`TownSpec`: any of the three test siege records, the moat's second row and the
  town's mines among them; no fort, Fort, Citadel or Castle by 1:2:3:3; the moat from a Citadel on;
  0 to 11 structures; the Escape Tunnel; morale 0 to 2 and luck 0 to 1; the attacker's Ballistics
  0 to 3; the defender's Artillery 0 to 3, so that some turrets are commanded). A town with a fort
  has no natural obstacles. The property ruleset (`lively`) gains the town's turret and Fight
  Values from 40 to 78 845, so that the computer's aim for the turrets and the automatic Ballista
  weighs its targets; the heroes' books gain Earthquake.
- **What the properties had to learn** (no engine change): the outcome leaves the turrets out
  (BAT-SIE-016); a turret dies with its tower or keep, and only `WallDestroyed` tells it
  (BAT-SIE-017: "no event but the part's"); the moat's damage explains another side's stack's
  loss (BAT-SIE-005); the automatic Ballista's candidates leave the turrets out (BAT-WAR-008);
  the casting limits read the garrison and the battlefield (BAT-MAG-061, BAT-TER-004: no creature
  casts, raises or periodic casts on Cursed Ground, a spell level limit of 1 there).
- **New property** `walls_and_the_moat_agree_with_movement` (BAT-SIE-002 to 004, 48 cases at a
  town with a fort): at every step no living stack covers a standing wall hex but the gate's and
  none but the defender's own the closed gate's or drawbridge's hex (its clone, summoned or
  teleported stack may, §7.32); no legal move goes there but onto the closed gate for a stack
  commanded for the defender; within a command no `Moved` lands on a standing wall, one onto the
  closed gate or drawbridge is followed by `GateOpened`, and a walker's walk that enters the moat
  outside its start ends there. A throwaway count over one run's games: about 5 800 moves, 52 gate
  openings, 15 walks into the moat, 2 500 legal moves onto the gate's hexes.
- **The hidden siege objects**: the town's mines (`t:necropolis` with a moat) are spell obstacles,
  so `hidden_obstacles_do_not_show` and `observations_do_not_depend_on_hidden_state` now cover
  them in about one game in sixteen.
- **Robustness**: `extreme_heroes_and_counts_play_without_failures` plays half its battles at a
  town with any structures byte (the keep's count is computed in `u32`), and
  `extreme_casting_heroes_play_without_failures` half of its hundred seeds.
- **Stress**: besides the normal runs, three throwaway runs with the case counts raised (every
  earlier property at 1 500 and 600 cases; then, with the garrisons, the Artillery and the new
  property, at 1 000 and 500; then, with the check's additions below, at 700 and 400): no
  failure. Exit criterion 4 is met.
- **The independent check** (a fresh agent over the property changes against the rules and the
  engine's code): no relaxation broader than its rule. It found: (1) the Ballista property handed
  the turrets and the Catapult to `ballista_target`, which the game never does (war.rs sends them
  to their own aims): now the Ballista only, and a turret's own target (BAT-SIE-020) is checked:
  a living stack of the attacker's, none only when none lives; (2) the turret of each part was
  read from the engine's own map: now every step checks that each turret is held by one target,
  lives exactly while it stands and leaves no corpse; (3) **the generated sieges gave the defender
  a Ballista at a fort**, which the original's 0x463600 never sets up (§7.27; VCMI none at any
  town, D-22) and the engine accepted: **BAT-SIE-029**, the battle leaves a listed defender's
  Ballista out at a town with a fort, with a test; **the user (BAT-Q48): a fix-optional quirk**,
  BAT-QUIRK-23 (below); (4) desperate mode (BAT-SIE-023) was never generated: the ground now draws the difficulty
  and the human sides; (5) the walls property left out the outer bridge, which opens the gate for
  a stack commanded for the defender and stops an attacker's walk while the gate is closed, and
  the attack's `from`; both added, and an open gate that stands must have a stack or a corpse on
  its hexes (never failed). Not done: asserting that the moat and gate checks fire in every run
  (the cases are random; `tests/sieges.rs` covers them with fixed games). The full verify before
  these fixes failed only the coverage gate: `Observation::removable_at`'s cleared-hex line, which
  only the properties reached and the new places reached more rarely (the HANDOFF's flaky-line
  trap); `tests/remove.rs` now asks it after a removal.
- **BAT-QUIRK-23** (the user at the checkpoint, 2026-10-15, BAT-Q48: fix-optional, "fixed deploys
  Ballista regardless of town fortification level"): the flag `quirk.battle.no_defender_ballista_at_fort`
  (`shared/e04c-t9`; position 24 first, 25 after master took 24 for the adventure lane's
  ADV-QUIRK-6, #230), on in the faithful profiles (the base pack's through the harvester, the
  synthetic pack's), off in strict-fixed; the test pack's profile `t:fort_ballista`; the unit test
  plays both settings; `every_step_preserves_the_invariants` mixes it into its half with
  BAT-QUIRK-7, the other properties play the fix. The faithful ruleset hashes moved: the ten
  battle replays and the six faithful adventure replays (`cross-lane`) re-recorded with their
  options, only the core ruleset hash in their headers moved. No version moved.
- **T9b, the workload**: `benches/battle/random-rollouts-siege.toml`, 60 random battles of the
  synthetic siege (a Citadel with a damaging moat, the keep of seven, Brea's commanded Catapult
  and Earthquake, her automatic Ballista): 698 902 794 Ir for 7 872 steps (88.8 k Ir per step);
  in every full verify, as `random-rollouts-full`. The criterion benches (`h3-sim/benches/battle.rs`)
  gain `legal_commands`, the step and `observe + encode` on mid-game siege states.
- **Measurements** (cloud sandbox: Intel Xeon @ 2.10 GHz, 2 vCPUs, one used; release; E04b T9b's
  sandbox was a 2.80 GHz Xeon, so wall times compare only within a column; `random-rollouts-full`
  plays the same 7 633 steps as at E04b's end):

  | Measure | E04c T9b |
  |---|---|
  | `h3 bench --games 2000`, `full` | median ≈ 98 k steps/s (eleven runs, 84–104 k); E04b's end (`bcd3a071`, built here) median ≈ 100 k (eight runs, 92–103 k), interleaved: the same within the sandbox's noise |
  | the same with the observation encoded before each step | ≈ 68 k (69.1 k, 67.2 k) |
  | `h3 bench --games 2000`, `siege` | ≈ 73 k steps/s (71.8 k, 74.4 k, 72.2 k); ≈ 63 k with the observation (63.6 k, 62.4 k) |
  | criterion `legal_commands` / `step` / `BattleEnv` step via mask (`full`) | 2.68 / 4.93 / 1.87 µs |
  | criterion `legal_commands` / `step` / `observe + encode` (siege) | 1.56 / 17.4 / 2.64 µs |
  | criterion `observe` / `observe + encode` (`full`) | 0.99 / 3.87 µs |
  | `BattleState` size / clone; `BattleEnv` size / fork | 6 768 bytes / 0.58 µs; 6 800 bytes / 3.40 µs |
  | `state_hash` | 4.88 µs |
  | bench-check `random-rollouts-full` | 592 768 860 Ir for 7 633 steps (77.7 k per step); at E04b's end 609 775 459 for the same steps: E04c costs the field workload nothing (−2.8 %, the footprint's change of 2026-10-13) |
  | bench-check `random-rollouts-siege` | 698 902 794 Ir for 7 872 steps (88.8 k per step), new |

  **Exit criterion 5:** the golden replays `field-s5` and `siege-s7` exist since T2 and T5; the
  100 k steps/s on `full` is at the edge of this slower sandbox, where E04b's end measures the
  same (E04b T9b: 104 k on its faster one); the instruction count says the epic added nothing
  there. A run on the user's Ryzen would settle the wall time.
- **The siege's profile** (callgrind, `--toggle-collect='*random_rollout*'`, exclusive):
  `movement::reachable_within` 26 %, `siege::line_blocked` 24 % (2 442 calls, ≈ 68 k Ir each: the
  shot's wall test steps 17 × the longer difference through software x87 additions; reached from
  the computer's shot estimates for the keep and the automatic Ballista, 18 % inclusive, and from
  real shots, 8 %), `Field::neighbors` 16 %, `Field::attack_positions` 5 %, `aim::turns` 5 %;
  `movement::path` through `travel_through` 14 % inclusive (a damaging moat sends every walk
  there). A cheaper exact line (each axis's positions are `f32` sums of `f32` steps whose exact
  sum needs at most about 38 bits, so the 53-bit rounding never acts and an integer fixed-point
  sum rounded once to 24 bits is the same; testable exhaustively per axis) would take most of
  the 24 %: the queue's inbox, for the user.
- **Verify**: the full local verify with the original data (the 164 maps staged too) on 1f399af
  (the branch with master merged): every step passed; bench-check `random-rollouts-full` +1.22 %
  against its baseline (as on master), `random-rollouts-siege` +0.00 %. Again with BAT-QUIRK-23
  on 207f388 (master and `shared/e04c-t9` merged): every step passed; +1.24 % and +0.02 %. CI
  cannot run.

## T10a — the fold-in and the audit (2026-10-15, increment 10)

- **The cut** (the user, 2026-10-15: the next sensible increment): T10 in three parts as E04b's
  T11: T10a the fold-in and the audit (this), T10b the independent review by fresh agents, T10c
  the closure. The shot's exact line comes before the review (the user), so that the review reads
  the epic's final code, as T7e came before T9.
- **The shot's wall line in integers** (`siege::LineAxis`, BAT-SIE-008 and Teleport's walls,
  BAT-SIE-026): each axis's step is still computed as the original's `f32` (the difference over
  the longer one in [`X87`], stored, divided by 17, stored); each position is then an integer
  numerator over 2^40, the sum rounded once to single precision (`round_single`). The step is at
  least 1/272, so a multiple of 2^-32, and every position below 17 a multiple of 2^-32: the exact
  sum fits 37 bits and the FPU's 53-bit rounding never acts; a position below 0 stays 0 as
  `X87::minus` kept it. `tests::sieges::the_line_s_integer_axis_is_the_x87_s` compares every
  position of every start (0 to 16), difference and longer difference (1 to 16) with the
  step-by-step `X87` computation (904 k steps). The line now tests a hex once where it enters it
  (the answer does not change within the line). No game changed: the siege workload played the
  same 7 872 steps in 585.0 M Ir (−16.3 %) with the integer line alone.
- **The war machines' record order** (`installation.md` §7.27, re-read; survey D-23; BAT-WAR-001):
  0x463600 gives the records after the army to the Catapult, the Ballista, the First Aid Tent,
  the Ammo Cart; the engine numbered them Ballista, Ammo Cart, Tent, Catapult (VCMI's sequence;
  VCMI numbers them before the army). Not only the siege's Catapult (the queue's T7d note): the
  Tent and the Ammo Cart were swapped in every battle. The stack ids now follow the code
  (`state::RECORD_ORDER`); the slots and places are unchanged. Only the computer's ties (the later
  record) and draws over stacks see it. The `full` scenario (Ballista and Tent; Ballista and Ammo
  Cart) keeps its ids and games; `siege-s7` re-recorded (the same 89 commands, new hashes: the
  Catapult is stack 4, the Ballista 5). Six tests named the old order: three in `h3-battle`, and in `h3-pack-battle` the harvested games of batch 1b R8 and batch 2d SU, whose stacks now list the Tent before the Ammo Cart as the original's records do. The user was told at the
  start; an experiment row (a Tent and an Ammo Cart tied in the computer's aim) is in
  `docs/experiments/README.md`.
- **The walls property's reach** (T9's open point): `the_walls_property_s_games_reach_the_gate_the_moat_and_the_breaches`
  draws 96 games as the property's runner does, from proptest's fixed seed, and requires the gate
  opened and closed, a walk into the moat and the moat's damage, the gate, a wall and a turret's
  part destroyed, a destroyed wall's hex crossed and a town mine set off: a generator change that
  loses one fails there.
- **The audit.** Catalog: no E04c rule changes status (the `implemented` ones have no observation
  of what they state: BAT-TER-002's batches 3a/3b observe the derivation, the adventure lane's;
  HER-BAT-018's batch 3d the control, BAT-SIE-011's); stale clauses rewritten in BAT-HER-004,
  BAT-MAG-002, 022, 048 to 053, BAT-SIE-011, HER-SPL-003, HER-WAR-001 (the battlefields' limits
  and raise, the turrets left out of a mirror's draw, the Catapult set up by the battle), and a
  stale comment in `casters::ready` (the Faerie Dragon's `ignores_magic_block` is harvested since
  BAT-Q32 (2)). The disputes D-1 to D-23 are each settled by the code with an observation, by the
  user, or carry their experiment (D-22, D-23: `docs/experiments/README.md`, "From E04c").
- **Exit criteria** (for T10c): (1) ADR-0022 and ADR-0023 accepted. (2) Every E04c rule has its
  sources and tests (`check-rules`); the disputes as above. (3) BAT-QUIRK-22 and 23 in the ledger
  with flags; S-06 and S-07 refuted by batch 3e, S-08 the user's fix (BAT-Q43), S-22 and S-29
  honoured, S-28 not a quirk (BAT-Q39). (4) Met in T9a; the reach test above. (5) `field-s5` and
  `siege-s7`; bench-check re-baselined (below); 100 k steps/s on `full`: 97 k to 104 k here (a
  2.80 GHz Xeon), the user's machine at T10c. (6) The batches 3a to 3h were played by an agent on
  the capture harness, as the user decided at increment 2 (none by the user): met by the user's
  answer at the checkpoint (BAT-Q49). (7) The full verify on Linux with the original data (below); Windows at T10c.
- **The queue's inbox** (31.5 to 22.8 KB): E04c's per-task notes, the E04c bullets of E04b's
  triage and review (done by T2 to T8), the adventure lane's notes for information and three
  E02/E03 notes E04c did (war machines, special battlefields, the obstacle table) replaced by one
  item by destination: what T10c passes on (`h3-heroes`, the game's version, the Catapult's
  artifact row, the original's battle AI to size), battle work without a task, the client lane's
  gate event order. The experiment candidates went to `docs/experiments/README.md`.
- **Measurements** (cloud sandbox: Intel Xeon @ 2.80 GHz, one core, release): bench-check
  `random-rollouts-siege` 577 498 334 Ir for 7 882 steps (73.3 k per step; T9b: 88.8 k, −17.5 %),
  re-baselined; `siege::line_blocked` 7.4 % of it (24 % before; 17 k Ir a call);
  `random-rollouts-full` unchanged (600 181 313 Ir, +1.25 % against its baseline as on master).
  `h3 bench --games 2000`: `siege` ≈ 95 k steps/s (master's code built here ≈ 82 k, noisy),
  `full` ≈ 97 k to 104 k. The architecture doc's E04c column keeps T9b's numbers (a shared path;
  the closure updates it).
- **Verify**: the full local verify with the original data (without the 164 maps; no map code
  changed) on a4e7588 (master merged: `docs/exe/map.toml`'s conflict beside the client lane's
  0x463370, both entries kept): every step passed; bench-check `random-rollouts-full` +1.25 %
  against its baseline (as on master), `random-rollouts-siege` +0.00 % against T10a's. The run
  before found the harvested games' order (`h3-pack-battle`) and rustfmt. CI cannot run.
- **The checkpoint** (the user, 2026-10-15, BAT-Q49): the original's war machine order stays
  (D-23); the batches played by agents count for exit criterion 6, so it is met. Recorded in the
  docs only; no new verify (the user: none needed).

## T10b — the independent review and its fold-in (2026-10-15, increment 10)

- **The review.** Ten fresh reviewer agents, none of which wrote E04c, each with a slice and a
  common brief (rule text against code, code against its sources, the executable re-read where a
  mistake is likeliest or costliest, VCMI 1.7.5 checked where a rule leans on it, the batches'
  rows, the tests): r1 the battlefields, r2a/r2b the walls, gate, moat, mines and shot line,
  r3a/r3b the Catapult, the wall shots, Earthquake, Ballistics and BAT-QUIRK-22, r4 the turrets,
  r5 the computer's aim, r6 the town's other rules and the war machines' record order (D-23
  confirmed by its re-read of 0x463600), a robustness review (111 120 extreme synthetic sieges and
  8 120 sieges on the harvested packs: no panic, overflow or rejected legal command; three bounded
  tests kept) and a security review. 36 rules and two quirks; 39 findings (F-r6-2 is F-sec-1):
  one high, eight medium, the rest low; two questions dropped (F-r3a-7, F-r6-1) and one claim not
  borne out (F-robust-1). Every high and medium one was checked again in the code and the
  executable (by the coordinator or by its fix worker) before it was fixed. The briefs, the reports and the verified list are on the backup branch
  `battle/e04c-t10b-review-reports` (not merged).
- **The user's decisions** (BAT-Q50): the random obstacles as the original's (not VCMI's), the
  computer's moat avoidance as the original's, no Wait or Defend for the commanded Catapult, the
  Escape Tunnel hidden from the attacker, the computer's aim kept reading the other side's
  Armorer and Artillery as the original's computer does. F-r6-1 (BAT-QUIRK-23's class) was not
  put again: BAT-Q48's question already named VCMI's "bug #551".
- **Fixed** (each with a test that failed before; five fix workers in sequence, each checking its
  claims in the code and the executable first):
  - the turn order: after the Catapult's turn the pick 0x465080 makes the defender the last side
    to act, so the attacker wins an equal-speed tie (high; the engine gave it to the defender);
  - the commanded Catapult has no Wait and no Defend (`MustShoot`); a hero's Earthquake that
    brings down the last standing target passes its turn (`turn::catapult_passes`: no command
    would be left; whether the original calls the Catapult's turn again after a cast was not
    read);
  - no shooter but the Catapult shoots the gate while a living stack stands on it (0x4766AA);
  - a dead created stack holds no gate (`siege::lies`; the original adds no corpse for a summoned
    stack, 0x444120);
  - a frenzied striker in the moat adds its defense less 3 (0x442514 → 0x442646);
  - Sacrifice refuses a turret in code (`raise::victim`), not only through the packs' `excludes`;
  - the computer's turns: a two-hex walker pays only the stops it does not cover yet
    (0x4B3701); a computer-controlled walker (`BattleSetup.human` of its controller) keeps out of
    every stop when its hit points are at most 4 × the moat's damage, or 40 × for the defender's
    stacks or a gate that is not closed (0x4B3420–0x4B34B2, read at 0x4B375C; `aim::avoids_stops`);
    the default `human = [false, false]` now plays so; the aim's hit-point sums saturate;
  - the random obstacles: an anchor may lie in the obstacle's own height row (0x46608D), each usual
    type is drawn at most once and a type without a place passes to the next (0x4667F8–0x466878);
    the draw sequence still differs from the original's (the inbox);
  - Magic Plains raise the opening spells to Expert (0x59FE98);
  - the observation: at a town the attacker and a spectator see the defender's `may_retreat` as
    `false`, and the observed `SiegeState` carries the Escape Tunnel for its defender only
    (observation encoding v21, C ABI v25: a change of meaning, the layout unchanged, as the rear
    hex's v14/v18).
- **Not changed:** F-robust-1 (a taken Catapult place stays `NoRoom`, as for every war machine);
  F-r2b-2 (the bow's wall exemption reads the owner's hero, the original the controller's:
  recorded, as the distance penalty's note). **Docs:** BAT-SIE-003, 004, 006, 009, 011, 014, 018,
  019, 022, 024, 028, BAT-GEO-004, BAT-TER-005, BAT-FLE-001, BAT-MAG-019, 068 and BAT-WAR-008's
  texts and sources; "Review re-read (E04c T10b)" notes in `installation.md` §7.26, §7.28, §7.29,
  §7.31; the ledger's note beside BAT-Q31; experiment candidates (the Catapult's Wait, the tie
  after the Catapult, the defender's summon onto the gate). **Tests** for the clauses the reports
  found unpinned, among them batches 3f, 3g and 3h's rows with their own numbers
  (`crates/h3-cli/tests/aim_rows.rs`).
- **Replays and benchmarks:** `siege-s7` re-recorded (the defender now wins, in 146 commands); the
  other battle replays unchanged. The adventure lane's nine (`cross-lane`, master's E08 T20b3b2
  set, merged in: every one fights a monster's battle) re-recorded for the obstacles: four with
  their fixed options, five searched ones replaced from a new search (1 500 faithful and 800
  strict-fixed games) by `duel-s1279-d28`, `duel-s810-d28`, `duel-s328-d28`, `duel-s1122-d28` and
  `duel-s307-strict-d28`; none now declines a pile's question or a Cartographer's offer (the doc
  comment of `adventure_replays.rs` says what each meets).
- **Left** (the queue's inbox, "E04c T10b"): the shared docs' sentences for T10c (ADR-0023,
  `06-security.md`, `07-simulation-and-ai.md`), the server lane's siege frames, and the readings
  the engine does not follow yet.
