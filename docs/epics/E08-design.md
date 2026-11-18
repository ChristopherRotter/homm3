# E08 design note

Increment 13's note: creature banks (E08 decision 333). Increment 12's note (garrisons and the
bonus visitables) is in `E08-log.md` ("Increment 12's design note at its closure", moved by T117),
increments 9's to 11's beside it (moved by T87c, T97c and T107); the code's comments cite these
notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 13's
amendments, drafted in the lane's `adr-drafts/` and applied to the ADRs by the round's `shared/`
task with batch E13's outcome (the drafts then deleted: applied by T122 to ADR-0016 points 1, 4,
6, ADR-0017 points 1, 2, 4, 6, 9 to 11, ADR-0022 points 2, 3, 6, 7, 8 and ADR-0013 point 5); written from T118a's survey and the
answers at ADV-Q52 and ADV-Q53 (the epic's decisions 333 to 339). A building task adds an *As
built* line to the section it builds; the note moves to `E08-log.md` when the increment closes.
Facts and sources: survey `docs/rules/surveys/E08-banks.md` (§ numbers are the survey's unless
they name this note); the proposed rule ids are the survey's §12.

Written in two parts by side (decision 340): **T118b1** parts A to C, the map side: **part A** the
content (`h3-adventure`'s kind, `h3-pack-adventure`, the packs), **part B** the map (`h3-map`,
`h3-adventure`), **part C** the looted searches' once-marks (`h3-adventure`, `h3-heroes`'
inputs); **T118b2** part D, the game side (`h3-game`, the battle seam, the env, replays, batch E13,
the cut and the exit criteria) and the checkpoint. ADR-0015 needs no amendment (no crate; the
import draws nothing, the build draws as for increment 7's objects); ADR-0013 none either (part
C). *Agent's call (change freely)* marks a choice the user could change; the checkpoint after
T118b2 accepts the note and the drafts.

**The checkpoints' answers** this note builds on: the content as the sketch's item 13 (decision
333); the survey whole (335); D-65, D-66 and D-68 as the original (336: the original's table, the
Naga Bank's sulfur and the Imp Cache's 2 Mercury; the Crypt's and the sea banks' question first,
the team's mark after a yes; the artifacts relics first); D-67 as the original (337: a won
bank's reward before the winner's levels), **replaced by decision 342** after T118b2's re-reading
(the original gives a human's levels inside `adventure_battle` before the reward: the levels
first, as GAM-DEC-007 point 3; survey §6's correction); the bank battle the adventure lane's,
`cross-lane`, after a `shared/` change of `BattleSetup` (338); batch E13 before the build (339);
this note's cut by side (340); T118b2 whole (341).

**A correction to the survey** (T118b1; survey §1, §4 and §10 corrected in the same change): the
banks' **visit modes are the original's class lists**, not the templates' blocked entrance the
survey read them from, as batch E12 found for increment 12's fifteen (ADV-Q51, decision 317; the
experiments README's objects note). `object_classes_not_entered` 0x63AA58 and
`object_classes_from_above` 0x63AB00 (`docs/exe/map.toml`) list the Shipwreck (85) and none of the
four others. So the **Shipwreck is visited from a neighbour** (and from the three tiles above its
entrance), and class 16's seven kinds, the Derelict Ship (24), Dragon Utopia (25) and the Crypt
(84) are **entered**: the hero steps onto the entrance and stands there for the visit, the
question, the battle and after a declined question. VCMI 1.7.5 agrees (`creatureBanks.json`:
`blockedVisitable` and `coastVisitable` on the Shipwreck alone). Batch E13 records where the hero
stands (part D's batch section).

## Part A — the content (the adventure part, its packs)

### 1. The bank kind (`h3-adventure`'s `ObjectKind`; ADV-BNK-001)

- **`bank { question, levels, looted, deck, coast }`**, one kind for the eleven (survey §1, §2):
  - **`question`**: `attack` (class 16's seven and Dragon Utopia: `visit_creature_bank`
    0x4A15A0, `visit_dragon_utopia` 0x4A2140) or `search` (the Crypt and the two sea banks:
    `visit_search_bank` 0x4AC490). The two flows differ in the question's moment and the team's
    mark (§6).
  - **`levels`**: one to four (the original's record holds four), in CrBanks.txt's order, each
    with **`chance`** (percent; the levels' chances sum to 100), **`guards`** (one to four
    creature and count pairs in the table's order, each count at least 1, distinct creatures, no
    war machine), **`upgrade`** (percent, 0 to 100), **`resources`** (by resource, each amount at
    least 1), **`creature`** (optional: the reward's creature and its count, 1 to 127, the
    original's signed byte) and **`artifacts`** (a count per class, treasure, minor, major and
    relic, each 0 to 255, the original's bytes).
  - **`looted`** (a `search` kind's only, optional): the once-bonus a looted one's search gives,
    `{ morale = -1, mark = m }` (part C).
  - **`deck`**: its battle is fought on the sea terrain's deck (the original's ship flag +0x53C6,
    set by class for 24 and 85; survey §5): the two sea banks.
  - **`coast`** (a neighbour-visited kind's only): its water entrance is reached from land and
    crossed by a flight as an empty boat's tile (§9; the original's class 85 tests): the Shipwreck.
- **Visit modes** by the class lists (the correction above): the Shipwreck `visit = "neighbour"`,
  `from_above = true`; the ten others `visit = "enter"`, `from_above = false`. Not ownable, not
  weekly, no kind mark (the original's `kinds_visited` holds no bank; the hint reads the tile's
  team bits, §10).
- **The split and the upgrade's slot are rules in code** (§5; ADV-BNK-002), the kinds' numbers
  data: the pack mirrors CrBanks.txt's unsplit counts, which the original-data check compares
  directly, as the monsters' split is code and their counts data (ADV-MON-012). *Agent's call:* not
  chosen, VCMI's form (each level's stacks already split, an upgrade slot per level), which moves
  the rule into the check alone.
- *Agent's call:* one kind with the question as a parameter, not two kinds: the levels, the draw,
  the battle and the reward are the same; only §6's order differs.
- Checked at the compile: the chances' sum, one to four levels and guards, the counts' bounds, a
  known creature and no war machine, `looted` only with `search`, `coast` only with `neighbour`.
  Hash tag 44; adventure ruleset part v21.

*As built* (T123a, `adventure/e08-t123a`; decision 351): `ObjectKind::Bank(Bank)` and
`ObjectKindDef::Bank(BankDef)` in `h3-adventure`'s `ruleset/banks.rs` (`BankQuestion`,
`BankLevel` with `artifacts: [u8; 4]` by `BANK_ARTIFACT_CLASSES`, treasure to relic; `BANK_LEVELS`,
`BANK_GUARDS`, `MAX_BANK_REWARD`), no effect (a step refused as any unbuilt visit's until T124b).
The compile's texts as listed, `check_chances` shared with the rewards; *agent's calls:* each
guard count at most the part's `max_stack_count` and every level's stacks after the split
(`bank_stacks`: five for one to three named stacks, else the four) within the army's slots,
checked once every pack is in (`check_bank`); the reward's creature no war machine either. The
pack's syntax a sub-table `[object_type.bank]` after the type's keys, each level an inline table
on one line (`{ chance, guards = [{ creature, count }], upgrade, resources, creature = { creature,
count }, artifacts = { treasure, minor, major, relic } }`; upgrade, resources, creature and
artifacts optional), `looted`, `deck`, `coast` (both false if left out).

### 2. The packs and the check against the original data

- **All eleven in the base pack** (every kind lies on RoE-format maps, survey §1), ids
  `cyclops_stockpile`, `dwarven_treasury`, `griffin_conservatory`, `imp_cache`, `medusa_stores`,
  `naga_bank`, `dragon_fly_hive` (class 16 subclass 0 to 6, a `# original class 16 subclass N`
  line each, as the dwellings'), `shipwreck` (85), `derelict_ship` (24), `crypt` (84),
  `dragon_utopia` (25); names CrBanks.txt's (each kind's first record, column 0).
- **The data** are survey §2's table (D-65 the original: the Naga Bank's sulfur, the Imp Cache's
  level 1 with its 2 Mercury), the creatures the executable's tables' (`bank_guard_creatures`
  0x6702A0, five per kind, −1 ending; `bank_reward_creatures` 0x67037C, by kind) by the base
  pack's ids as survey §2 lists them (the Dragon Fly Hive's guards `dragon_fly`, the upgraded
  fly). Every chance column 30, 30, 30, 10.
- **The check** (`h3-pack-adventure`'s `original_data` example, a seventh check after T113a's
  sixth): each kind (class 16's subclasses 0 to 6, then 85, 24, 84, 25 as the original's kinds 7
  to 10) has exactly one `bank` type in `base`, named its CrBanks.txt name; its levels equal
  CrBanks.txt's four records (the chance, the four counts, the upgrade chance, the seven
  resources, the reward's count, the four artifact counts; a count of 0 no guard stack, a reward
  count of 0 no creature); its guards' creatures the executable's five by position and its
  reward's the executable's (−1 with a count of 0), through the core packs' original creature
  ids; its visit and from-above as the class lists say; `question = "search"` exactly for 24, 84
  and 85, `deck` exactly for 24 and 85, `coast` exactly for 85 (the visit routines and the class
  tests, survey §4, §5). `CrBanks.txt` from AB's extracted `H3ab_bmp` (H3bitmap's is the same
  file, survey §2). Fails with the mismatches, as the six before it.
- RoE's own `CrBanks.txt` is not read (no RoE installation before E11's T1): the base pack takes
  SoD Complete's table for RoE maps too (part D's risks).

*As built* (T123a): the eleven in `packs/base/adventure.toml` after the Watering Hole, in kind
order, written from `CrBanks.txt` and the two tables (the marks 14 Shipwreck, 15 Crypt, 16 Derelict
Ship); the check (`banks`, check 7) also wants a looted bonus exactly on the three search kinds. It
passes on the staged installation ("11 bank types match CrBanks.txt's 11 kinds' four levels, …")
and failed as it should on four perturbations (the Naga Bank's sulfur as gems, a deck on the Crypt,
the Shipwreck not from above, a renamed Griffin Conservatory with a Dragon Fly Hive's guard
changed).

## Part B — the map (`h3-map`, `h3-adventure`)

### 3. The import and the writer (`h3-map`; map import v24)

- **`codes::object_type`** maps class 16 by subclass 0 to 6 to the seven types and 85, 24, 84 and
  25 at subclass 0 to theirs; another subclass is one its class does not define
  (`ImportError::Subclass`; the original indexes its kinds' table by 16's subclass without a test,
  no shipped map has one past 6, survey §1). **No body** to read (`Body::None`; HotA's bank body
  is outside our formats), so nothing is held (`Holds::None`) and nothing drawn at the import
  (ADR-0015).
- **The writer** (`h3_map::writer`) writes the five classes by template, without a body: batch
  E13's maps and part D's bank maps (the spec's fields in `h3-experiment` are part D's
  `cross-lane` work).
- On the 148 shipped maps with banks the 2 137 objects stop being blockers. A test that used one
  of the five classes as a plain blocker breaks (`TRAPS.md`); the shipped-map workloads move once
  (part D).

*As built* (T123b, `adventure/e08-t123b`, `cross-lane`; decision 352): `codes::CREATURE_BANKS` for
class 16 by subclass and one arm each for 85, 24, 84 and 25 through the subclass test (past 0
`Subclass`, as the bonus visitables'); no body, so `Holds::Nothing` (the note's `Holds::None`) and
no owner. The census (`original_data`'s `increment_13`) checks every count of survey §1's table
(by version, level and entrance, "beside land" a land tile among the entrance's eight neighbours)
and found them all on its first run. A step onto a bank's entrance is `AdventureCommandError::
Visit` (`visit_at` finds no built effect), the reach ending before it (`h3-map`'s `tests/banks.rs`).
No test used one of the five classes as a plain blocker. Map import v24: the 89 adventure goldens'
`import_version`, checkpoints and final hashes, and `h3-server`'s `tests::record`.

### 4. The state (`h3-adventure`; adventure state hash v26)

**No new block** (ADR-0016 point 1): every part of a bank lives where a kind before it keeps the
same thing.

- **What the build drew**, in the static map: `Resolved::Bank { level, artifacts }`, the level's
  index and the drawn artifacts in the draw's order (relics first, §5). The level's resources and
  creature are read from the kind by the index, never copied. A new `Resolved` hash tag.
- **Its guards as drawn** (split and upgraded, §5) are the object's guards (`MapObject::guards`,
  their stacks by slot, as a guarded pile's), so the static map's hash covers them and
  `guards_left` reads a battle's survivors over them. *Agent's call:* there rather than inside
  `Resolved::Bank`, so the guarded objects' reads serve; the bank's visit, battle end and reward
  are matched by kind before the guarded objects' fallback (§7).
- **A battle's survivors** (none after a draw): the object's army left (increment 7's block), as a
  guarded pile's.
- **Looted**: the object's emptied mark (increment 7's block, as a looted Pyramid's); a bank never
  leaves the map.
- **A team's visit**: the per-player knowledge of objects (increment 7's block), set for every
  player of the visitor's team (the original's bits 5 + p; its owner until E10's teams).
- **The looted searches' once-marks**: the hero on the map's list (part C).

A fork costs what it did; the artifacts' list lives in the static map behind its `Arc`.

### 5. The draw when the game is built (ADV-BNK-002; survey §3)

- **Where:** the static map's build, in `draws::resolve_rewards`' second walk, in object order with
  the chests, the gifts, the Shrines, the finds and the Pyramid, from the setup stream (ADR-0017
  point 4). So a bank's artifacts come from the one pool of the random artifacts, the chests and
  the finds (ADV-OBJ-006: never one banned, placed or drawn before until the pool is used up,
  D-32), as the original's game pool gives them (survey §3 step 2). The original walks the
  entrances by level, row and column; the engine walks the objects, and within a bank the order
  is the engine's (decision 17).
- **Per bank**, three steps with draws and one without:
  1. **The level:** r, a draw in 1 to 100; the first level whose running sum of chances reaches
     r, else the last (the original's subtraction, the same result).
  2. **The artifacts:** the level's counts, **relics first**, then majors, minors and treasures
     (D-68 the original), each one draw from the pool in its class; a draw that finds none gives
     none (the bank gives fewer).
  3. **The split** (no draw) by the number n of the level's guard creatures, each stack in its
     slot as the original's first empty slot puts it:
     - n = 1: c the count; slots 1 to 4 take h₁ = ⌊c / 5⌋, h₂ = ⌊(c − h₁) / 4⌋, h₃ = ⌊(c − h₁ −
       h₂) / 3⌋, h₄ = ⌊(c − h₁ − h₂ − h₃) / 2⌋, slot 0 what is left (every shipped count a
       multiple of 5: five equal stacks); the upgrade's slot 2;
     - n = 2: the second creature's count halved into slot 2 (⌊x / 2⌋ moves, the rest stays in
       slot 1); the first's divided by 3 into slot 3 (⌊x / 3⌋), what is left of it halved into
       slot 4; the upgrade's slot 3 (the Crypt's level 1: 10, 10, 10, 10, 10);
     - n = 3: the second's halved into slot 3, the first's halved into slot 4; the upgrade's slot 0
       (the Crypt's level 2: 13, 10, 5, 10, 12);
     - n = 4: no split, slots 0 to 3; the upgrade's slot 0.
     A quotient of 0 leaves its slot empty (*agent's call*: unreachable with the shipped counts,
     every one at least 10; the original's write of a stack of 0 is unread).
  4. **The upgrade:** when the level's chance is above 0 and the upgrade's slot holds a creature
     with an upgrade in the ruleset (`Ruleset::upgrades`, its first; the original's
     `creature_is_unupgraded` 0x47B120 and `creature_upgrade` 0x47B1A0 agree on every shipped
     guard), r, a draw in 1 to 100; r at most the chance makes the stack its upgrade, the count
     kept. The original draws r for every bank; the engine only where it can change a stack
     (decision 17's form). With the shipped data only class 16 upgrades, one stack of five at
     50 %; the Dragon Fly Hive's Dragon Flies have none, and nor has a base pack's Water
     Elemental (RoE's dead branch, survey §3 step 4, needs no counterpart).
- **Never drawn again:** no week, month or visit touches a bank's draw.
- *As built* (T124b1, decision 354): `state/banks.rs`' `bank_draw` (steps 1, 2, 4) and
  `bank_split` (step 3), called by `draws::resolve_rewards`' second walk (a module of its own for
  the reach's sake: the log's T124b1 measure); the guards ask (`Guards::question` true; §6's visit
  asks by the kind's question); a one-level bank draws its r too; tests in `state/tests/draws.rs`
  (the level and the upgrade against the setup stream's draws, the split's four cases and an empty
  quotient, the relics first from the one pool, none with every artifact banned).

### 6. The visit (ADV-BNK-003; survey §4; D-66 the original)

A bank is visited by a move's last step: onto its entrance for the ten entered kinds (the move
ends there; a guard beside it fights first, decision 279), toward the Shipwreck's from a
neighbour (from land too, §9; from above); and by `Revisit` for a hero standing on an entered
bank's entrance (decision 268). By the kind's question:

- **attack** (class 16, Dragon Utopia):
  1. The team's marks set at once (`ObjectMarked` to each player whose mark is new).
  2. Looted: `ObjectEmpty` to the visitor's owner (the original's ADVEVENT 33, GENRLTXT 425),
     nothing more.
  3. Else the question, a pending offer of the visitor's owner (ADVEVENT 32, 47). Declined:
     nothing (the marks stay; the hero stays on the entrance). Accepted: the battle at once (§7);
     when no guards stand (a draw emptied them), the reward at once instead (§8; *agent's call*:
     the guarded piles' "without guards standing", the original's battle against an empty army
     unread and out of reach of a batch).
- **search** (the Crypt, the Derelict Ship, the Shipwreck):
  1. The question first, looted or not (ADVEVENT 41, 119, 122). Declined: nothing, **no mark**.
  2. Accepted: the team's marks set.
  3. Looted: `ObjectEmpty` (the original's second text), then the kind's `looted` once-bonus by
     its mark (part C): −1 morale until the end of the hero's next battle, while its mark is
     not held.
  4. Else the battle (§7), or with no guards standing the reward (§8).
- **The offers** (*agent's call*): the attack reuses `OfferKind::Fight`, the search
  `OfferKind::Search`, each routed by the object's kind when accepted (as `Fight` already routes a
  mine's); the Pyramid keeps its `SearchPyramid`. No new variant, so nothing changes on the wire,
  in the replay format or in the env's decision kinds (part D confirms). Not chosen: a bank offer
  kind, or two.
- Every player answers as a human (ADR-0017 point 1); the computer's answer (`ai_fight_answer`) is
  E10's.
- *As built* (T124b2, decision 355): `state/banks.rs`' `visit_bank` (`Effect::Bank`, the step's
  and `Revisit`'s) and `enter_bank` (the accepted `Fight` or `Search`, routed by `bank_of` in
  `accept_offer` before the guarded piles' and the finds'); the marks silent (`ObjectMarked` and the
  hint with T125a); the owner's `known` lists a visited bank as `Known::Visited` (*agent's call*:
  `Known::Emptied` would tell looted from a draw's empty guards, which the hint never does); an
  attack kind looted while its question was open told so (*agent's call*). The Shipwreck is visited
  from a boat until T125a's passage (§9).

### 7. The battle, the map's half (ADV-BNK-004; survey §5)

- **The start:** the accepted question gives the outcome's `BattleStart` with
  `Defender::Guards { army }`, the bank's guards as they stand, by slot (an earlier battle's
  survivors, an upgraded stack's as its own creature), unowned and without a hero, at the
  entrance's tile (where the hero stands for the ten entered kinds; the Shipwreck's water tile
  from the coast or a boat).
- **`BattleStart::bank`**, a new field, true for this battle only: the game sets the setup's bank
  battle from it (part D; the seam a `shared/` change, decision 338). *Refined by T118b2* (§14):
  `Defender::Bank` instead, by the same cause, so no `BattleStart` literal changes. **By the
  battle's cause, not by its tile:** the original's flag is `adventure_battle`'s last argument, 1 from
  `bank_battle_reward` alone (a monster's battle passes 0, survey §5), so a hero met on an entered
  bank's entrance fights an ordinary battle, unlike the anti-magic garrison's battle, whose tile
  decides (D-60). Part D settles the other callers' argument (§14: a code reading, every other
  caller passes 0).
- **The battlefield** (`BattleStart::battlefield`, GAM-BAT-001 amended in part D): a `deck`
  kind's battle is on the sea terrain's deck, as with one hero in a boat (`sea_battle_ground`
  counting the bank as one boat at least): the Shipwreck from the coast and from a boat, the
  Derelict Ship from its boat (survey §5, `E08-sea.md` §7.1). A land kind's is the tile's as any
  battle's (its terrain, a special ground on it).
- **The seed:** the original's s = 170 101 x + 223 252 y + 250 037 z + 82 037 of the entrance
  (survey §5; a monster's coefficients differ); the battle's seeds are the game's (ADR-0017 point
  4), part D.
- **The end** (`end_unowned_battle`'s bank case): lost or retreated, the guards' survivors by slot
  stay as its army left, the bank not looted, asked again at the next visit (the hero leaves the
  map as after any lost battle); a draw leaves none standing, not looted (§6's case); won, the
  reward after the winner's levels (§8, as corrected). The hint's change goes to the teams that
  visited it (§10).
- *As built* (T124b2): `bank_battle` (`Defender::Bank` at `pile_tile`, the entrance; a deck kind's
  terrain and battlefield replaced by `sea_battle_ground(…, 1)`, so a hero in a boat gets the deck,
  never the two boats) and `end_bank_battle` (`end_unowned_battle`'s arm before the guarded objects'
  fallback: the survivors, or none won and after a draw, as `army_left`).

### 8. The reward, the map's half (ADV-BNK-005; survey §6; D-67 and D-68 the original)

- *Corrected by T118b2* (decision 342; survey §6's correction): **won, as a won guards' battle's
  rest** (`AdventureOutcome::reward_after`, `Rest::Won` of the entrance, GAM-DEC-007 point 3),
  given after the winner's levels by `give_won`'s bank case, read when given; no exception and no
  rest of its own. *As T118b1 wrote it (replaced):* won, in the battle end's own outcome, not as
  `reward_after` (D-67: before the winner's levels). The record as before (ADV-RWD-001,
  `give_record`'s form):
  - **the creature**, when the level has one: its stack onto the hero's army as `army_add_stack`
    (ADV-ARM-007: the first stack of the creature within a stack's most, else the first free
    slot), else as the outcome's `no_room` group for the army dialog (GAM-DEC-004; what is left
    in it lost);
  - **the artifacts** in the record's order, relics first (D-68: the positions and the backpack
    fill in that order), each the hero cannot receive lost (ADR-0013 point 15.4; SoD's assembly
    is a command, never a question here);
  - **the resources** of the level as the outcome's gains (GAM-RES-001);
  - no experience, skill or spell.
  Then the bank is **looted** (its emptied mark), and stays on the map.
- The record's own order (the hero half, the resources, the creatures) stands against the
  original's (the creatures, the artifacts, the resources): none reads another's (the army, the
  hero's artifacts, the treasury) and only the army dialog is a decision (*agent's call*).
- *As built* (T124b2): `bank_won` (`give_won`'s arm before the beaten objects': nothing once
  looted, the race of another player's yes while the winner's levels are open) and `loot_bank`
  (`bank_record`: the drawn artifacts as the hero half, the level's resources, its creature as one
  stack in slot 0; `give_record`; then the emptied mark); the readers `Resolved::bank` and
  `ObjectKind::bank` in the banks' modules, `state::draws` untouched (T124b1's trap).

### 9. The Shipwreck's passage (ADV-STUB-12 lifted; ADV-MOV-009, ADV-SEA-002 amended)

- The original's step and search take class 85's water entrance as a boat's (`hero_step_allowed`
  0x481D0D, `pathfinder_expand` 0x4B26D6 and 0x4B29AC; `E08-sea.md` §3.1, §4.1, §8.1, §15). So a
  `coast` kind's entrance on water is, for a hero not in a boat, **a step from a stop node on
  land** (walking, water walking or flying: the visit from the neighbour, the hero staying on
  land) and **a flight's pass** from land or from water (as an empty boat's tile, batch E8c's
  E8-17d and E8-17e); never a water walk's pass. A hero in a boat visits it from the water as
  every sea object (ADV-PIK-018).
- In the code: `passage`'s water branch takes it beside the empty boat (its comment's stub goes),
  and so do the move's check (`check_tile`, the 0x481D0D comment) and the reach. A hot path:
  measured against `master` (part D's bench; `homm3-bench-regression`).
- The Derelict Ship stays D-46's: visited only from a boat, never crossed by a flight.
- Untested in the original (`E08-sea.md` §16: a flight over class 85): batch E13 gains a row (part
  D).
- *As built* (T125a, `adventure/e08-t125a`, whole: decision 356): `state/banks.rs`'
  `coast_visit` and `is_coast_entrance` (`#[cold]`, out of line: the live tile visitable and the
  tile's object a bank of a `coast` kind, so another object visited from the tile after it closes
  it, ADV-OBJ-003);
  `check_step`'s water case (an empty boat, else the coast's visit, before the step across a top);
  `passage`'s water branch after the empty boat (a stop node's visit, a flight's pass, a water
  walk's pass refused `Blocked`) and its step up off a closed top (a flight passes it);
  `StaticMap::coast_entrances` (the coast kinds' visitable water tiles, listed with the economy's
  objects) marked in the walking search's `occupied` by the empty boats' loop, chained, so the
  window's rare `AFLOAT` case visits it in `afloat` (a hero first, then an empty boat, then the
  coast's visit with the walker's drop); the layered search through `passage`.
  ADV-STUB-12 lifted; ADV-MOV-009, ADV-MOV-010, ADV-SEA-002 amended. Measured right after the
  first build (`homm3-bench-regression`): a pass of its own before the loop (`#[cold]`, behind an
  emptiness test) cost `Reach::search` +1.9 % on `reach-xl-144` (no Shipwreck) and put
  `random-play-bonuses-36` over its gate; chained into the boats' loop `reach-xl-144`,
  `reach-sea-xl-144` and `reach-lifted-xl-144` are flat against `master` (+0.04 %, −0.79 %,
  −0.49 % to the baselines; `master` +0.05 % on the first), every workload inside its gate, the
  random-play ones +0.0 to +1.6 % against `master` (`Reach::search`'s code with `afloat`'s third
  case, which no shape tried removed: the lane's `TRAPS.md`).

### 10. Information and the events (ADV-BNK-006; ADR-0016 point 6; survey §7)

- **Every viewer that sees the bank** sees its type. Per viewer, by its team's mark
  (`ObjectView::visited`, so far a weekly object's only):
  - not visited: "(Not visited)";
  - visited, looted or none standing: "(Visited)": `ObjectView::guards` none;
  - else `ObjectView::guards` a `GuardsView` without the army: the creature when every stack is
    of one (an upgraded stack beside its base: none, the generic text; `army_description_text`
    0x40ABE0, as a mine's) and the word of their total count (`MonsterParams::count_word`).
  Live: a battle's survivors and the loot change it for every team that visited.
- **Hidden** from every viewer: the level, the counts, the upgrade, the artifacts and the reward,
  until the battle (its stacks and counts) and the win (the reward's events). Never in an
  observation.
- **Events** (no new variant; part D confirms the server's fold): the team's marks `ObjectMarked`
  (each player whose mark is set anew); a looted visit `ObjectEmpty` (the visitor's owner); the
  offer (the owner); the battle's; the reward's (`CreaturesJoined`, the hero's changes, the gains:
  the owner); the once-bonus (`BonusChanged`, `OnceMarkChanged`: the owner); the hint's change
  `GuardsChanged` to the players whose team visited it, after a battle's end or the loot, when its
  creature or word changed or it was looted (`count_word` none).
- So an attack kind's declined visit teaches its team the guards' word without a fight, a search
  kind's only a yes (D-66's form; the Pyramid's knowledge works the same).
- *As built* (T125a, decision 356): `state/banks.rs`' `bank_view` (`object_view` asks it for a
  bank instead of the weekly mark and the mine's guards: the mark `knows`, the hint
  `bank_hint` = `observe::guards_hint`, the mine's and the bank's one reading of
  `army_description_text`, over `guards_left` unless looted), `mark_bank` (the team's marks for
  `allied` players not holding them, `ObjectMarked`, then the hint) replacing `learn` in
  `visit_bank` and `enter_bank`, `tell_hint` from `end_bank_battle` (now with the part, the type
  and the sink). *Agent's call:* no `GuardsChanged` at a mark when no hint shows (looted, none
  standing: the view as before the mark, as a mine's hint that stays), and none at the loot (a
  bank is looted only with none standing, whose hint its battle's end told). ADV-BNK-006
  `verified`; ADV-VIS-006, ADV-OWN-003 amended.

### 11. Ids, quirks, stubs and versions (parts A to C)

- **Rules:** ADV-BNK-001 to -006 (survey §12; texts by §1 to §10 here). Amended: ADV-OBJ-003 (the
  five's entrances end a path: entered, or the Shipwreck's from a stop node), ADV-MOV-009 and
  ADV-SEA-002 (§9), ADV-VIS-006 and ADV-OWN-003 (§10), ADV-RWD-001 (a bank's record, §8),
  HER-ADV-008 (part C). GAM-BAT-001 and GAM-DEC-007 are part D's.
- **Quirks:** none (survey §9). **Stubs:** ADV-STUB-12 lifted (§9). **Disputes:** none new (the
  visit modes apply decision 317).
- **Versions:** adventure ruleset part v21 (the kind), map import v24 (the five classes), adventure
  state hash v26 (`Resolved::Bank`, the banks' guards in the static map). The rest is part D's.

## Part C — the looted searches' once-marks (HER-ADV-008 amended)

### 12. Three once-marks, no change to the hero model

- **Marks 14, 15 and 16** (the Shipwreck, the Crypt, the Derelict Ship), after increment 12's 2 to
  13 and in the order of the original's status bits 9, 10 and 11 (+0x105; survey §4), each
  `looted = { morale = -1, mark = m }`.
- The hero on the map's short list of once-marks (decision 306) takes them as any: added while not
  held, so a second looted Crypt gives nothing and a looted Crypt and a looted Shipwreck −2;
  cleared at the end of its next battle (GAM-BAT-014); in battle each the source
  `EffectSource::Visit { mark }` (HER-ADV-008). Every hero takes them, a computer's too.
- **ADR-0013 needs no amendment:** the marks are content and the list has no fixed count.
- *As built* (T124b2): no code of its own: the packs' `looted` (T123a) given by `enter_bank`'s
  `give_once`; tested with two kinds and the battle's clear (`state/tests/banks.rs`).

## Part D — the game, the env, batch E13 and the cut (`h3-game`; T118b2)

Written by T118b2 (whole, decision 341): the bank battle's seam and rules (§13) with
the draft for ADR-0022 point 8 (applied by T122); the game's battle (§14); the reward after the levels (§15, D-67 corrected,
decision 342); the observation and the wire (§16); the env, replays, maps, properties and
benchmarks (§17); versions and ids (§18); batch E13 (§19); the shared and `cross-lane` work, the
cut and the exit criteria (§20); risks (§21); with the draft for ADR-0017 (applied by T122).

### 13. The bank battle: the seam and the battle's rules (survey §5; ADR-0022 point 8)

- **The seam** (`h3-contracts`, `shared/`, decision 338): **`BattleSetup::creature_bank: bool`**,
  the original's flag (battle +0x53C5, H3API's `creatureBank`), default `false`, public. *Agent's
  call:* one flag, as the original's, not a field per rule: the four rules below come together in
  every bank and never apart, and fields per rule would let a caller pass a mix no battle of the
  original has (ADR-0022's argument against hero-like effects on the setup). Every exhaustive
  `BattleSetup` literal gains `creature_bank: false` in the same change (mechanical: the battle
  lane's crates and tests, `h3-files`, `h3-pack-battle`'s scenario, `h3-sim`).
- **The battle's rules** with the flag (`h3-battle`, `cross-lane`, the battle lane reviews):
  1. **No tactics phase** (BAT-TAC-001 amended, its "no tactics in creature banks" now a rule;
     `battle_start` 0x462C5D): `begin` opens none whatever the heroes' Tactics; the levels stay
     the heroes' (private, as now). The flag is the battle state's, since `setup` and `begin` may
     be apart (CLT-Q69 (a)).
  2. **No obstacles** (BAT-GEO-004 amended; `battle_obstacles` 0x4662B3):
     `generate_setup_obstacles` gives none for such a setup, as for a town with a fort, and the
     setup **refuses** obstacles or a siege beside the flag (a new `SetupError` variant; *agent's
     call*: a caller's mistake refused, not dropped; no bank stands at a town).
  3. **No war machines on the field** (BAT-WAR-001 amended; `battle_war_machine_stacks`
     0x463758): each side's listed machines are checked as now, then neither set up nor in the
     outcome, as a side's without a hero, so the hero keeps them whatever the battle (HER-WAR-002
     loses only a destroyed one). **The Ammo Cart's supply stays** (BAT-WAR-004 amended: the
     original's test at 0x43F9E1 reads the hero's artifact, not the stack; the wiki agrees): a
     side whose listed machines include an ammo supply is supplied for the whole battle as by a
     living cart. Only the bank's case: whether a destroyed cart supplies in other battles stays
     the battle lane's open point (its queue, "War machines", r3).
  4. **The layout** (BAT-SET-001 amended; 0x4636B7, `bank_battle_hexes` 0x63D0E0): each side's
     stacks in slot order take, by their **ordinal among the filled slots**, the battle part's
     **`bank_positions`**, the formation ignored; each its stack's position (a two-hex stack's
     rear hex, its front one column toward the enemy, BAT-GEO-002). The positions are a new
     Tier 2 battle parameter beside the formations (`packs/base/battle.toml`'s `[battle]`), as
     (column, row): the attacker (6, 3), (10, 3), (5, 5), (8, 5), (11, 5), (6, 7), (10, 7), the
     defender (15, 0), (15, 10), (2, 10), (2, 0), (15, 5), (2, 5), (8, 0) (the original's hexes
     57, 61, 90, 93, 96, 125, 129 and 15, 185, 172, 2, 100, 87, 8). The compiler checks
     `army_slots` positions a side, each standable and distinct, and that two-hex footprints on
     all fourteen lie on the field and share no hex, so a bank's placement cannot fail. *Agent's
     call:* content, as the formations, and the table's hex as the place, not a closest-free
     search (no obstacle or machine can stand there). Gap-filled with the code reading as its
     source, or harvested, by the battle lane's form; checked against 0x63D0E0 when the original
     data are staged.
- **D-69** (a two-hex guard at 172, 2 or 87): by BAT-GEO-002 the guard at 172 covers 172 and 171,
  at 2 covers 2 and 1, at 87 covers 87 and 86, the cells VCMI's second table (171, 1, 86) names if
  VCMI's position of a two-hex defender is its front hex (*unverified*). The engine keeps the
  original's one table; batch E13's E13-4 records the Griffins' hexes. *Dissolved* (T121, survey
  §14): the Griffins on 14-15, 184-185, 171-172, 1-2, 99-100 and the attacker's two-hex stacks on
  57-58 …, each the table's hex as its rear; VCMI names the front hex (read at T121), so its wide
  table covers the same defender cells.
- **Unchanged in a bank:** the battlefield (the tile's or the deck, §14), morale and luck, retreat
  (the guards' side has no hero to retreat or surrender), the computer's choices for the guards
  (the neutral seat's `h3_battle_ai::choose`; whether the original's battle AI reads +0x53C5 is
  unread, §21).
- **Visibility** (*agent's call*; the checkpoint's question): the flag is public but does not
  join the battle's observation. Each effect shows in the battle itself (the places, no
  obstacles, no phase, no machines, a supplied shooter's shots not falling), so the observation,
  its flat encoding, the C ABI and the wire stay, unlike ADR-0022 point 6's anti-magic flag,
  which a viewer needs for the refused book.
- **Not in increment 13:** a `creature_bank` key in the battle's scenario files and battle replays
  (no bank battle reaches either; *agent's call*: the battle lane adds both, with the replay
  format's bump, when a scenario needs one; its inbox told).

*As built* (T124a, `adventure/e08-t124a`, `cross-lane`; decision 353): the flag is
`BattleState::creature_bank`, hashed with each side's `bank_supply` after the anti-magic flag
(battle state hash v56); `begin` opens no tactics phase with it; `generate_setup_obstacles` gives
none (the boats' hexes neither) and `setup` refuses obstacles or a siege beside it
(`SetupError::InvalidCreatureBank`); `war_machines` sets up none, a listed ammo supply of a side
with a hero setting its `bank_supply`, which `ammo_supplied` reads beside a living cart; each
stack at `BattleParams::bank_position(side, ordinal)`'s footprint. `bank_positions` is
**harvested** from 0x63D0E0 (`h3_formats::exe::bank_hexes`, `installation.md` §7.62; pack format
v53, battle part v8), not gap-filled; the compiler asks both hexes of each footprint standable.
Tests: `h3-battle`'s `tests/banks.rs` (the layout with two-hex stacks on both sides, the formation
unused; the tactics phase with setup and begin apart; the obstacles, the boats and the refusals;
the machines and the supply, a side without a hero or a cart unsupplied; the hash).

### 14. The game's battle (ADV-BNK-004's game half; GAM-BAT-001 amended)

- **`Defender::Bank { army }`** (*agent's call*, refining §7's `BattleStart::bank`): the bank's
  guards as they stand, by slot, unowned, without a hero, the system actor's side as
  `Defender::Guards`'. Still by the battle's cause (only a bank's accepted question builds it),
  and no `BattleStart` literal changes, so the town lane's siege code and its tests stay
  untouched (T114a1's reason for the anti-magic query). `start_battle` sets `creature_bank` from
  it; its arms in `h3-game`'s two matches and `Defender::owner` are the adventure lane's.
- **The other callers** (settled by a code reading, T118b2; `docs/exe/map.toml`): of the 14 calls
  of `adventure_battle` 0x4AD470, only `bank_battle_reward`'s (0x4ABEDB) pushes the bank flag 1;
  `visit_hero`'s (0x4A2933), `monster_battle`'s, the guarded artifact's, resource's and scroll's,
  Pandora's Box's, `event_fire`'s, the dwelling's, the mine's, `visit_object`'s, `town_visit`'s two
  and one in 0x4ACD70 (0x4ACF47) push 0. So a hero met on an entered bank's entrance fights an
  ordinary battle, as §7 says; part D's planned batch row for it is dropped.
- **The setup** as a guards' battle's: the attacker's army, battle hero with its bonuses, war
  machines (listed, not set up, §13), formation (ignored by the layout), funds, human flag and
  colour; the defender's army the guards. **The battlefield** (GAM-BAT-001 amended): a land kind's
  the tile's, as any guards' battle (its terrain, the coast's shore, a special ground on it); a
  **`deck` kind's** (the Shipwreck, the Derelict Ship) the sea's terrain on the deck,
  `sea_battle_ground` with the boats counted as at least one, from the coast (the hero on land
  beside a Shipwreck) and from a boat; a special ground on the tile replacing the battlefield as
  at sea. The background and the attacker's own ground as any battle's (GAM-BAT-011, -013).
- **Seeds** (ADR-0017 point 4): the battle's own seed the attacking player's stream's next draw,
  as every battle's; the obstacles' seed as now (the tile's under `ADV-QUIRK-5`, else the
  stream's draw after it), the generator giving none (*agent's call*: as a fortified siege, no
  case in the game). The original's s = 170 101 x + … (survey §5) has no counterpart: the battle's
  draws are the engine's own, and the original's use of a battle's stored seed is unread
  (`E08-battles.md` §4.2).
- **The end** (`end_battle`'s unowned branch, `end_unowned_battle`'s bank case): as §7 and §8,
  after the battle's experience, Necromancy and Eagle Eye (`heroes_after_battle`) and the
  bonuses' clear (GAM-BAT-014: a looted search's −1 cleared with them); so Necromancy's raised
  creatures stand in the army before the reward's creature joins, as in the original
  (`battle_end` raises at 0x47776E, the reward follows the battle).
- *As built* (T124b2): `start_battle`'s two matches and `creature_bank: matches!(start.defender,
  Defender::Bank { .. })` (the obstacles' stream draw kept, the generator giving none);
  `h3-game`'s `tests/banks.rs` reads the stacks at the bank positions from the open battle.

### 15. The reward after the winner's levels (D-67 corrected; GAM-DEC-007 amended)

- **The correction** (survey §6's note; the user's decision 342, replacing 337): for a human the
  original gives the winner's levels inside `adventure_battle` (0x4AE324, after the battle's
  screen), before `bank_battle_reward`'s reward; that routine's own `hero_level_up` (0x4AC473)
  matters only after the computer's battle without a screen (0x4270C0, no human on either side;
  E10's). So the original's human order is GAM-DEC-007 point 3's: **no exception.**
- **As built for every guarded object since T63b:** a won bank's end holds its reward as the
  visit's rest (`AdventureOutcome::reward_after` with the entrance, `Rest::Won`, tag 2), given
  after the winner's levels (GAM-BAT-008) by `give_won`'s bank case (§8's record: the creature
  joined or the army dialog's group, the artifacts read when given, so a level's skills cannot
  change what fits but positions taken meanwhile do, the resources), then the loot. The army
  dialog's `Close` loses what is left (GAM-DEC-004). No new rest kind, no game state hash change.
- A draw empties the guards and gives nothing (§7); a yes at a bank without guards standing gives
  the reward at once, with no battle and no levels (§6).
- **GAM-DEC-007 amended:** rest (3) lists a creature bank's reward and loot (ADV-BNK-005).
- *As built* (T124b2): as written; `h3-game`'s `tests/banks.rs` checks the level before the gold
  and the creature, and the army dialog for a creature without room.

### 16. The observation, the events and the wire (ADV-BNK-006's game half)

- **No game-level view:** a bank's hint is the adventure observation's `ObjectView` (`visited`
  and `guards`, §10), which `GameObservation` carries as it is; no new struct field.
- **The events' order at a new mark:** `ObjectMarked`, then the bank's hint as `GuardsChanged`
  (the word and the creature; none when looted or none stand) to the same players, so a team
  learns the word at the mark (§10's declined attack). Then the battle's end and the loot send
  `GuardsChanged` to the teams that visited, as §10 says.
- **The wire** (`h3-protocol`, the server lane's): nothing to map. The events are existing
  variants, the offers `Fight` and `Search` existing kinds, `Defender` not on the wire; the
  fold (`wire/game/fold.rs`) sets `visited` at `ObjectMarked` and `guards` at `GuardsChanged` for
  any object by type and anchor, so it serves banks unchanged (checked at T118b2). Protocol v69
  stays.
- **`docs/06-security.md`** (the `shared/` task): increment 13's lines (§10's hidden facts: the
  level, the counts, the upgrade, the artifacts and the reward until the battle and the win; the
  team's mark and hint; the once-bonus to the owner).
- *As built* (T125a): as written; `GameObservation` carries the adventure's `ObjectView`, no field
  moved, no version (the fold folds both events by type and anchor; `h3-map`'s `check_told` and
  `reveal`'s oracle take them as a mine's). No fixture map holds a bank, so no golden replay and no
  observation hash moved.

### 17. The env, replays, maps, properties, benchmarks

- **The env** (the AI lane's `AdventureEnv`): no new action. The question is answered by the
  offers' answers (`Fight`, `Search`), the battle by its commands, the army dialog by `Close` and
  the management commands, so **action encoding v8 stays**; random play answers them; no flat
  encoding (decision 187). The AI lane's inbox told (nothing moves).
- **Replays:** no new command or reference: **adventure replay format v16 stays.** A new map
  written by `h3_map::writer` (its five classes, T119): **`e08-banks-36.h3m`** (the eleven kinds
  beside two players' starts, the land ones on grass and dirt, a coast with a Shipwreck beside
  land, a Derelict Ship off it and a boat; a hero of each player with a full army, one near a
  level). Golden replays: random play on it (three, seeds per `adventure_replays.rs`) and scripted
  ones: a declined attack and a declined search (the marks and the hints for both players), a won
  Griffin Conservatory with a full army past a level (the level-up, then the army dialog), a lost
  battle and the revisit (the survivors' layout), two looted Crypts and a looted Shipwreck (−1,
  −2, cleared by a battle), the Shipwreck fought from the coast and the Derelict Ship from the
  boat (the deck), a flight across the Shipwreck. Every golden replay's hash lines move with the
  adventure state hash and, from T124a, the battle state hash and the battle part's hash
  (`homm3-rerecord-replays`; the battle lane's replays with theirs).
- **Properties** (`h3-map`'s `play.rs`): `BANKS` in the plans (the eleven near the heroes, the sea
  kinds on the strait, as the Whirlpools'), `Step::Bank` (a move onto or toward one, the offer's
  answer); `check_banks`: a bank's draw and guards never change but by its battle; looted only
  by a won battle (or a yes without guards), never fought again; a won bank's gains, artifacts
  and joined or lost creatures exactly its level's; no `LevelUpOpened` or `LevelGained` of the
  winner after the reward's first event; a bank battle's start has no obstacles, no tactics phase,
  no war machine and the layout's hexes; the once-marks at most one per kind until the next
  battle; `check_told` for the mark's `GuardsChanged`; the hygiene property varies a bank's hidden draw
  and ends at the viewer's team's mark or battle of it. Fixed cases (512) that visit every kind. The
  battle lane's battle properties gain bank setups (T124a, `cross-lane`).
- **Benchmarks** (`homm3-bench-regression`): the Shipwreck's passage in the reach first (T125a,
  right after its first build): `reach-xl-144`, `reach-sea-xl-144` and `reach-lifted-xl-144`
  against `master` (no Shipwreck: flat); a new XL map **`e08-banks-144.h3m`** (144 × 144 with a
  sea: the census's most banks on one map, 106 as *Thousand Islands*, every kind, the sea kinds on
  its sea, many Shipwrecks beside land; a monster on every hundredth free land tile) with
  **`reach-banks-xl-144`** (walkers and flyers, the Shipwrecks met from the coast and crossed),
  **`build-banks-xl-144`** (the game's build with its draws, as `build-gates-xl-144`) and
  **`random-play-banks-36`** on the 36 map (the visits, the bank battles' setup, the rewards).
  No fork workload: a fork carries no new block (§4). The battle lane's workloads against
  `master` for T124a's setup branch (flat expected).
- `--with-original-data`: the census (§1), the import of all 160 shipped maps with the five
  classes typed (2 137 objects on 148 maps: their entrances end paths, so `original_data.rs`'
  shipped-map counts move once), each shipped map built with its banks drawn, the seventh pack
  check (§2), the bank positions against 0x63D0E0.

*As built* (T125b1, `adventure/e08-t125b1`, `cross-lane`; decision 357): **`e08-banks-36.h3m`**
(`h3-map`'s `tests/banks_map.rs`): the land kinds on dirt (columns 0 to 12) and grass, two Crypts,
the Shipwreck's entrance beside the coast with an island beyond it (a flight crosses only towards
land), the Derelict Ship off the coast, a boat; ten heroes, each of red's seven placed for a
scripted replay (a full army without Angels and 900 experience below the Griffin Conservatory).
Nine golden replays: random play `banks-s545-d28`, `-s161-d28`, `-s804-d28` (a probe over the
seeds 1 to 1 500); scripted `banks-declined-s1`, `-griffins-s1`, `-lost-s1`, `-looted-s1` (the
Shipwreck from the coast inside it), `-sea-s1` (the Derelict Ship from the boat on day 2:
boarding ends the move), `-flight-s1`. **`e08-banks-144.h3m`** (`tests/banks_xl_map.rs`): the sea
on columns 96 to 143, the 106 in the census's shares, fourteen coast Shipwrecks each with an
island beyond it, a flyer of each player in front of one. The properties, the three workloads
and `--with-original-data` are T125b2's.

### 18. Versions, rule ids, quirks, stubs

| Version | From | To | By |
|---|---|---|---|
| adventure ruleset part | v20 | **v21** | T123a: the kind `bank` (hash tag 44); *done* |
| map import | v23 | **v24** | T123b: the five classes typed; *done* |
| adventure state hash | v25 | **v26** | T124b1: `Resolved::Bank`, the banks' guards in the static map; *done* |
| battle state hash (the battle lane's) | v55 | **v56** | T124a, `cross-lane`: the flag in the state; *done* |
| battle ruleset part (the battle lane's) | v7 | **v8** | T124a, `cross-lane`: `bank_positions`; *done* |
| pack format (shared, the battle file) | v52 | **v53** | T124a, `cross-lane`: `bank_positions` in `[battle]`; *done* |
| game state hash | v32 | v32 | none (§15: no new rest; the sub-hashes fold in as values) |
| adventure replay format | v16 | v16 | none (§17) |
| env action encoding | v8 | v8 | none (§17) |
| `h3-protocol` | v69 | v69 | none (§16) |
| battle replay format (the AI lane's) | v32 | v32 | none (§13: no key yet) |

The numbers are the next after today's, not reserved: whichever lane merges first takes one and
the other bumps again (the battle lane's E26 and E11 work move the battle's versions too).
**Rules:** ADV-BNK-001 to -006 (parts A to C), GAM-BAT-001 and GAM-DEC-007 amended (§14, §15),
the battle lane's BAT-SET-001, BAT-TAC-001, BAT-GEO-004, BAT-WAR-001 and BAT-WAR-004 amended
(§13, written by T124a with the battle lane's review). **Quirks:** none. **Stubs:** none new
(ADV-STUB-12 lifted, §9). **Disputes:** D-67 dissolved by the correction (decision 342), D-69 for
batch E13 (*dissolved* by it, survey §14). **Decisions:** 341, 342 (next 343).

### 19. Batch E13

Survey §11's ten rows in one run, as planned (SoD on Easy, hot seat, two players, days 1 to 3, a
map `h3-experiment` writes), with this part's changes:

- **E13-3** extended: Red's hero also brings an Ammo Cart and a stack of Archers: after their
  first shot their shots are unchanged (the supply without a cart on the field, §13), beside the
  predicted hexes, no tactics phase, no obstacles, no Ballista.
- **E13-6** predicts the corrected order (survey §11 as corrected): the level-up first, then the
  reward's message and the army dialog with the Angels.
- **E13-11** (added; §6 and the note's correction of the visit modes): Red's hero steps onto a Cyclops
  Stockpile's entrance and declines, then answers yes and wins: where the hero stands at the
  question, after the decline and after the battle (the entrance each time; the move ended
  there); a second hero visits the Shipwreck from the coast (on land throughout).
- **E13-12** (added; §9, `E08-sea.md` §16): a hero with Angel Wings flies across a Shipwreck's water
  tile to the water beyond it (passed, no visit, no question); a hero with Boots of Levitation
  walks the coast toward it (visited from the neighbour, the hero on land) and cannot take a path
  across it.
- **Dropped:** the bank flag at a hero's battle on a bank's entrance (§14: the code reading).
- **The writer and the spec** (T119; the spec `h3-experiment`'s, battle-owned, `cross-lane`; the
  writer this lane's): the five classes by template, no body; what the spec lacks of a hero at the
  start (secondary skills, war machines and artifacts are there for the battle's heroes; the
  adventure hero's checked first), a boat (E13-5). **The harness** (T120; `clients/capture/`,
  common): reads of the banks' records (game +0x4E3DC, 0x6C each), a hero's +0x105 and +0x11A
  (E12's may serve), the battle's stack hexes and shots at the start and after a shot (the battle
  capture's), the dialogs' order, the hero's tile. The verdicts (T121, survey §14) fold into the
  drafts at the `shared/` task (T122).
- *Scored* (T121, survey `E08-banks.md` §14): **every row as predicted** (D-65, D-66, D-68
  confirmed, D-67's correction confirmed, D-69 dissolved); E13-3's hero fought loose (the map's
  tight flag reaches no placed hero, `ADV-QUIRK-8`), so §13's formation ignored rests on the reading
  and VCMI.

### 20. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** the `shared/` task (T122, `cross-lane`): the three drafts applied with
batch E13's outcome (ADR-0016, ADR-0017, ADR-0022 with ADR-0013 point 5's line); the seam's field
`BattleSetup::creature_bank` with every exhaustive literal's `false` (mechanical, its own commit,
the battle and AI lanes told first); `docs/03-domain.md`'s terms (creature bank, a bank's level,
looted, bank battle, bank positions); `docs/06-security.md`'s lines (§16). `cross-lane` elsewhere:
batch E13's spec (`h3-experiment`, T119), the harness (common, T120), the battle crate's rules, the
battle pack and the battle's golden replays (T124a, the battle lane reviewing). **No** `h3-protocol`
change (§16). **The inboxes:** the battle lane's (the seam's shape and the rules, §13; T124a's
plan and versions; D-67's correction for its record; the scenario and replay keys theirs to add),
the AI lane's (nothing moves: v8, no observation flag), the map client lane's (the eleven types
and the hint, T123a and T125a), the client lane's (informational: a bank battle's layout without
war machines and obstacles comes in the battle's observation as any).

*As built* (T122, `shared/e08-t122`, `cross-lane`; decision 350): the three drafts in ADR-0016
points 1, 4, 6, ADR-0017 points 1, 2, 4, 6, 9 to 11 and ADR-0022 points 2, 3, 6, 7, 8 (ADR-0013
point 5's line), each citing batch E13's rows, VCMI's second table among ADR-0022's alternatives;
the glossary's five terms; `docs/06-security.md`'s lines; `BattleSetup::creature_bank` with its
two exhaustive literals (the battle lane's scenario and replay document; every other the
default), the battle lane's inbox told.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut confirmed
at its start). **Batch E13 runs before the build and before the shared task** (decision 339), so
the ADRs are applied once with the rows' verdicts:

| Round | Tasks | Content |
|---|---|---|
| 2 | **T119** | Batch E13's map: the writer's five classes (by template, no body), the spec's fields (`cross-lane`), the spec and the template (§19) |
| 3 | **T120** + **T121** | T120: the harness's reads, the recipe, the run. T121: scored (survey §14) |
| 4 | **T122** | `shared/` (`cross-lane`): the three drafts with T121's outcome, `BattleSetup::creature_bank` and its literals, the glossary, `docs/06-security.md` |
| 5 | **T123** | T123a: the kind, the packs, the seventh original-data check (§1, §2; part v21). T123b: the import and the writer's classes for fixtures (§3; map import v24); a step onto a bank refused as any unbuilt visit's until T124b |
| 6 | **T124** | T124a (`cross-lane`, first): the bank battle in `h3-battle` and the battle pack (§13; battle state hash v56, battle part v8, pack format v53; the battle's properties, golden replays and bench). T124b in two (decision 354): T124b1 the state and the draw (§4, §5; adventure state hash v26); T124b2 the visit and its offers (§6), the battle's two halves (§7, §14: `Defender::Bank`, the flag, the deck), the reward as a won battle's rest (§8, §15), the once-marks (§12) |
| 7 | **T125** | T125a: the hint and its events (§10, §16), the Shipwreck's passage with the reach's measure against `master` (§9; ADV-STUB-12 lifted). T125b: the two maps, the replays, the properties, the benchmarks, `--with-original-data` (§17) |
| 8 | **T126** | The audit against the exit criteria; increment 13 closed; increment 14 refined |

*Agent's call:* the battle crate's half before the map's battle (T124a merged first, so T124b's
tests see the layout), the passage with the views (both small, each measured or told).

**Increment 13's exit criteria:**
1. The five classes import typed by the base pack's eleven kinds, one entrance each (the
   Shipwreck visited from a neighbour and from above, the ten entered); the census holds; the
   seventh pack check passes against CrBanks.txt and the executable's tables (ADV-BNK-001).
2. The game's build draws each bank once from the setup stream in object order: the level by its
   chances, the artifacts relics first from the one pool, the split by the original's rule, the
   upgrade by its chance where it can change a stack; never again (ADV-BNK-002).
3. An attack kind marks the team at once and asks unless looted; a search kind asks first, looted
   or not, and marks after a yes; a looted search gives its −1 morale once per kind until the
   next battle; `Revisit` visits an entered bank again (ADV-BNK-003, HER-ADV-008; D-66).
4. A yes fights at once the guards as they stand, unowned, at the entrance; a hero met on a bank's
   entrance fights an ordinary battle; the sea banks fight on the deck, the Shipwreck from the
   coast too (ADV-BNK-004, GAM-BAT-001).
5. A bank battle has no tactics phase and no obstacles, sets up no war machine and keeps the Ammo
   Cart's supply, and places each side's stacks by their ordinal at the bank positions (BAT-SET-001,
   BAT-TAC-001, BAT-GEO-004, BAT-WAR-001, BAT-WAR-004; D-69 as batch E13 found).
6. Lost or retreated, the survivors stay and the bank is not looted; won, after the winner's
   levels, the creature joins or opens the army dialog, the artifacts come in the record's order
   (relics first), the resources are gained, and the bank stays looted (ADV-BNK-005, GAM-DEC-007;
   D-67 corrected, D-68).
7. The Shipwreck's water entrance is a walker's and water walker's visit from the coast and a
   flight's pass; the reach's workloads hold their baselines (ADV-MOV-009, ADV-SEA-002;
   ADV-STUB-12 lifted).
8. A viewer sees a bank's type and, by its team's mark, the hint (not visited; visited; the word
   and the creature); never the level, the counts, the upgrade, the artifacts or the reward
   before the battle and the win; the events as §10 and §16 (ADV-BNK-006, ADV-VIS-006).
9. Every new rule has its id, source and tests; the gates pass (coverage, determinism with the new
   and rehashed replays of both lanes, the benchmarks at their baselines and the new ones
   baselined); batch E13 run and scored (T121), the consensus rows `verified`.

### 21. Risks and open points

- **Two lanes in `h3-battle`:** the battle lane's E26 and E11 work and T124a's change of its setup,
  versions and golden replays; whichever merges first, the other rebases and bumps (§18). The
  seam's literals (T122) touch the battle and AI lanes' crates in one mechanical commit.
- **RoE unread:** RoE's `CrBanks.txt` and RoE's battle (the flag's sites) are not read (§2): the
  base pack takes SoD Complete's table and rules for RoE maps; E11's T1 may revisit.
- ~~**D-69**~~ dissolved by E13-4 (survey §14): one table, the stack's rear hex its position.
- **The original's battle AI in a bank:** whether the computer's choices read +0x53C5 is unread;
  the guards play by `h3_battle_ai::choose` as any neutral side.
- **The computer's quick battle** (0x4270C0: the reward before the levels) and its answers
  (`ai_fight_answer`, `ai_join_without_room`) are E10's; the engine plays every player as a human.
- **2 137 objects stop being inert** on 148 shipped maps: any test that used one of the five
  classes as a plain blocker breaks (`TRAPS.md`), and the shipped-map counts move once (T123b,
  T125b).
- **The reach's water branch** gains the Shipwreck's case (§9): flat on maps without one, measured
  on `reach-banks-xl-144`.
- **Not read:** network play's branch of `adventure_battle` (0x4AD725 to 0x4AE18F), the stored seed's
  use inside the battle, the reward message's texts and the hint's pictures (the clients').
