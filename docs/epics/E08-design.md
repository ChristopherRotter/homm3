# E08 design note

Increment 13's note: creature banks (E08 decision 333). Increment 12's note (garrisons and the
bonus visitables) is in `E08-log.md` ("Increment 12's design note at its closure", moved by T117),
increments 9's to 11's beside it (moved by T87c, T97c and T107); the code's comments cite these
notes as "increment N's design note §M".

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 13's
amendments, drafted in the lane's `adr-drafts/` and applied to the ADRs by the round's `shared/`
task with batch E13's outcome (the drafts then deleted); written from T118a's survey and the
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
the team's mark after a yes; the artifacts relics first); **D-67 as the original** (337: a won
bank's reward before the winner's levels, an exception to GAM-DEC-007 point 3); the bank battle
the adventure lane's, `cross-lane`, after a `shared/` change of `BattleSetup` (338); batch E13
before the build (339); this note's cut by side (340).

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

### 7. The battle, the map's half (ADV-BNK-004; survey §5)

- **The start:** the accepted question gives the outcome's `BattleStart` with
  `Defender::Guards { army }`, the bank's guards as they stand, by slot (an earlier battle's
  survivors, an upgraded stack's as its own creature), unowned and without a hero, at the
  entrance's tile (where the hero stands for the ten entered kinds; the Shipwreck's water tile
  from the coast or a boat).
- **`BattleStart::bank`**, a new field, true for this battle only: the game sets the setup's bank
  battle from it (part D; the seam a `shared/` change, decision 338). **By the battle's cause, not
  by its tile:** the original's flag is `adventure_battle`'s last argument, 1 from
  `bank_battle_reward` alone (a monster's battle passes 0, survey §5), so a hero met on an entered
  bank's entrance fights an ordinary battle, unlike the anti-magic garrison's battle, whose tile
  decides (D-60). Part D settles the other callers' argument or adds a batch row.
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
  reward at once (§8). The hint's change goes to the teams that visited it (§10).

### 8. The reward, the map's half (ADV-BNK-005; survey §6; D-67 and D-68 the original)

- **Won, in the battle end's own outcome, not as `reward_after`** (D-67: before the winner's
  levels, the exception to GAM-DEC-007 point 3; its game half, where the levels wait for the army
  dialog, is part D's): the bank's record given as a reward record (ADV-RWD-001,
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

### 9. The Shipwreck's passage (ADV-STUB-12 lifted; ADV-MOV-009, ADV-SEA-002 amended)

- The original's step and search take class 85's water entrance as a boat's (`hero_step_allowed`
  0x481D0D, `pathfinder_expand` 0x4B26D6 and 0x4B29AC; `E08-sea.md` §3.1, §4.1, §15). So a
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

## Part D — the game, the env, batch E13 and the cut (`h3-game`; T118b2)

Written by T118b2: the bank battle's seam (`BattleSetup` in `h3-contracts`, a `shared/` change,
and its ADR draft; the battle's rules: no tactics phase, no obstacles, no war machines on the
field, the layout by the stacks' ordinals, D-69); D-67's exception to GAM-DEC-007 point 3 (the
levels after the reward and its army dialog; `BattleStart::bank`, §7); GAM-BAT-001 (the deck,
§7); the env, replays, maps, properties and benchmarks (the shipped-map workloads, §9's hot path,
a bank XL); versions; batch E13 (survey §11's rows with this note's additions: the hero's tile at
an entered bank, a flight over a Shipwreck and a water walker from the coast, the flag at a
hero's battle on a bank's entrance); the shared and `cross-lane` work, the building tasks' cut,
the exit criteria and the risks (RoE's table unread, §2); ADR-0017's draft.
