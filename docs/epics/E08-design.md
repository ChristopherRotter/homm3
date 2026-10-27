# E08 design note: increment 7, rewards on the map

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 7's
amendments of ADR-0013 (points 1, 2, 15), ADR-0016 (points 1, 3, 4, 6) and ADR-0017 (points 1,
4, 6, 8–11), applied by T59 from the drafts T55b wrote
from T55a's survey and the answers at ADV-Q28 (the epic's decisions 129–136), accepted with the
cut at ADV-Q29 (decisions 138–141; decision 138's typed answers written into §14). The task that
builds a section deletes it; the file goes when the epic closes. Facts and sources: survey `docs/rules/surveys/E08-rewards.md` (§ numbers are the
survey's unless they name this note); the proposed rule ids are the survey's §12, with the
additions of §6 and §17 here.

Increment 6's note is in `E08-log.md` ("Increment 6's design note at its closure", moved by T53).

Written whole (decision 137): **part A**, the hero side (`h3-heroes`, the core content); **part
B**, the map (`h3-map`, `h3-adventure`); **part C**, the game, the env, batch E7 and the cut.

## Part A — the hero side (`h3-heroes`, the core content)

### 1, 2. The data and the hero changes

Built by T60a (`E08-log.md`, "T60a"): `HeroDetails::bonuses` (`BattleBonuses`), the changes
`GainPrimary`, `RaiseSkill`, `Bonus`, `ClearBonuses`, the details' hash in the game state hash
v15. Already there for §3 and §4: the artifact classes (`ArtifactClass`, Special never placed at
random), the spells' level, schools and `granted_by_name_only` (Titan's Lightning Bolt, flag
0x2000), the classes' level-up weights (the University's slots), the Learning factor
(HER-EXP-001), the learnable level (HER-SPL-005), the backpack's most (64, HER-ART-002), the most
skills (8), `HeroParams::mana_ceiling` (999, T59). Not the details': the once-per-hero marks of
objects (the adventure's, by hero id, §10) and the players' knowledge (§10).

### 3. The reward record's hero half (HER-CHG-003; ADR-0013 point 15.2)

`h3_heroes::grant(details, ruleset, reward: &HeroReward) -> Granted` where `HeroReward` holds the
record's experience, primary skills, skills, mana, morale, luck, artifacts and spells (resources
and creatures stay the adventure's), and `Granted { changes: Vec<HeroChange>, experience: u32,
lost: Vec<ArtifactTypeId>, skipped: Vec<SpellId> }`. It works on a scratch copy so each part is
checked against the details as the earlier parts leave them (the original's order, §7.4):

1. the experience amount: `trunc(f32(amount) × learning factor)` with the Learning **before** the
   record's skills (D-39), only for an amount above 0 (the map's u32 read as signed: 2³¹ and
   above give none); returned in `experience`, **not** applied (the game gives it last, §14);
2. `GainPrimary` with each value above 0;
3. `RaiseSkill` per pair (D-37: raised, never lowered; a new one only below the most; no class or
   ban test);
4. `SetSpellPoints(clamp(mana + delta, 0, mana_ceiling))` (D-38: a hero above the ceiling falls
   to it; a negative delta stops at 0);
5. `Bonus { morale, luck, None }` when either is not 0;
6. (resources: the adventure's);
7. `AddArtifact` per artifact that `can_receive_artifact` admits (§4: A-25's flag), else `lost`;
8. `LearnSpell` per spell with a worn spell book (step 7's counts), of a level at most the
   learnable level (step 3's Wisdom counts), not known; else `skipped`; the map's bans not tested;
9. (creatures: the adventure's; the army dialog: the game's);
10. (the experience: the game's, after the army dialog).

A combination completed by a received part asks nothing (§8.3): the player assembles with
`Manage::Assemble`. The function is pure and total; `Granted` is empty for an empty record.

### 4. What a teacher may teach (ADR-0013 point 15.3–15.4)

| Function | Rule | Survey |
|---|---|---|
| `can_receive_artifact(details, ruleset, artifact)` | `FULL_BACKPACK_REFUSES` on (`faithful`): the backpack's count below its most, whatever the free positions; off: a position that takes it (HER-CHG-002) or the backpack below its most (A-25, decision 145; ADR-0013 point 15.4) | §3.2, §8.2, §15 |
| `learns_spell(details, ruleset, spell, known_counts)` | a worn spell book, level ≤ learnable, and (a Shrine, a Scholar, a box) not known; the Pyramid passes `known_counts = false` (relearning changes nothing) | §6.5, §6.7, §7.5 |
| `hut_teaches(details, ruleset, skill)` | not known, below the most skills; no class test | §6.6 |
| `university_slot(details, ruleset, skill) -> SlotState` | `Known`, `NoRoom` (eight), `Barred` (the class weighs it 0), else `Open` | §6.8 |
| `scholar_reward(details, ruleset, kind, id, fallback) -> HeroChange` | a spell as `learns_spell`; a skill: known below Expert a level up, new at Basic below the most; else +1 to the fallback primary | §6.7 |
| `library_admits(details, ruleset, level)` | level + 2 × Diplomacy level ≥ the type's level (10) | §6.2 |
| `learning_experience(details, ruleset, amount)` | `trunc(f32(amount) × learning factor)`: the Learning Stone (1 000), a chest (gold − 500) | §2.2, §6.4 |
| `next_level_experience(details, ruleset)` | the gap to the next level's threshold, no Learning: the Tree | §6.4 |

### 5. The bonuses in battle

Built by T60a (HER-ADV-008: `derive_battle_hero` folds the bonuses into the own side's morale and
luck sums). Left for T63: the game applies `ClearBonuses` to every hero of a battle when it ends
(§15, GAM-BAT-014); its test: a hero with a −3 bonus and Leadership Basic gets −2 morale on its
stacks; after any battle, 0.

### 6. Rule ids, quirks, shared needs (the hero side)

HER-CHG-003 (§3) as survey §12 (T60b); HER-ADV-008 and HER-CHG-001's summary built by T60a. No quirk of the hero side but A-25's (`FULL_BACKPACK_REFUSES`, flag 38, T59: a full
backpack refuses an artifact whatever the free positions in `faithful`; ADR-0013 point 15.4; the
wraps are A-9's class). Everything else is `h3-heroes`' (the lane's).

## Part B — the map (`h3-map`, `h3-adventure`)

### 7. The import (`h3-map`; map import v12)

- **Classes to types** (`codes.rs`): the 31 classes of ADR-0016 point 4, each to an
  object type key of the packs' `adventure.toml`; 65 to 69 to one "random artifact" type each
  (their classes are the type's parameter); 88 to 90 to three Shrine types (the level the type's).
- **Bodies** (all already read, survey §10): an artifact's message → `Guards` with `question:
  true` whenever it has stacks (D-29: the guarded artifact always asks); a scroll's → `Guards` with
  `question` = the text is not empty (§4), and its spell; a Shrine's spell (255: drawn); a Witch
  Hut's mask (`None` on an RoE map, which has none in the file: A-21 is the build's, §9); a
  Scholar's kind and id (`0xFF` random); `Rewards` → the object's **reward record** (below); an
  event's colours, computer flag and removal flag; the monster's treasure artifact
  (`Unsupported::MonsterArtifact` goes: the 8 maps move on).
- **The reward record** (`RewardRecord` in `h3-adventure`'s setup, one per box or event, A-19 not
  reproduced): experience (the map's u32; 2³¹ and above none, §7.4's signed test), mana (i32),
  morale, luck (i8), resources (i32 each, by resource; negative a loss), primary skills (the
  bytes as i8; only positive values given), skills (the ruleset's, level 1–3), artifacts, spells,
  creatures (stacks by slot). **Refused** (`Unsupported`, A-24): more than seven creature stacks;
  a random creature code (as the guards'). The guards as a guarded pile's `Guards` (a box's
  `question: false`: announced, never asked).
- **The bans:** the map's banned spells and artifacts (AB and SoD headers; RoE none) into
  `GameSetup` beside `banned_skills` (GAM-SET-003, §17); the game passes all three to the
  adventure's build.
- **Still blockers:** the classes of later items (banks, Refugee Camp, Seer's Huts, quest guards,
  boats, mines, dwellings…). The census of what imports whole after T61 is its own measurement
  (`--with-original-data`).

### 8. The kinds (`h3-adventure`'s ruleset part v12; `adventure.toml`)

Closed variants of `ObjectKind`, the type's parameters in the packs (data, ADR-0006), the
instance's on `MapObject` (static). Names are the build's (change freely); the shapes:

| Kind | Classes | Type parameters | Instance (map, then drawn) |
|---|---|---|---|
| `Chest` | 101 | gold amounts and chances (32/32/31 %: 1 000, 1 500, 2 000), the artifact's chance (5 %) and class (treasure), a full backpack's gold (1 000), the experience offset (500) | drawn: gold or an artifact |
| `Artifact` | 5, 65–69 | for a random type its classes (65: treasure, minor, major) | the artifact (the template's subclass) or drawn; guards |
| `Scroll` | 93 | — | the spell; guards |
| `Gift` | 4, 23, 32, 41, 47, 51, 61, 100, 102, 107 | once per hero; what it gives: primary gains, a choice of two primary skills with its amount, experience × Learning, one level; a price (or drawn prices: the Tree's free, 2 000 gold, 10 gems); a condition (the Library's level 10, 2 per Diplomacy level); whether one may leave (the Schools, not the Arena) | the Tree's price (drawn) |
| `Shrine` | 88–90 | the spell level | the spell (map or drawn) |
| `WitchHut` | 113 | the RoE masks: `faithful` 0xEFBF, `strict-fixed` all but Leadership and Necromancy (A-21) | the mask (map's or the RoE default), the skill (drawn) |
| `Scholar` | 81 | — | the kind and id (map or drawn), the fallback primary (drawn) |
| `University` | 104 | offers (4), price (2 000 gold) | the four skills (drawn) |
| `Find` | 22, 39, 105, 108 | its draw table (§6.9: the Corpse 20 % treasure-or-minor; the Wagon 10 % empty, 40 % treasure-or-minor, 50 % 2–5 of wood to gems; the Lean To 1–5 of wood to gems; the Tomb by class 30/50/15/5), a full backpack's turn (gold 1 000, the resources, or the artifact stays), a question (the Tomb), a bonus with its once-mark (the Tomb: −3 morale, mark 0) | the contents (drawn) |
| `PandoraBox` | 6 | — | the reward record, guards |
| `Event` | 26 | — | the reward record, guards, colours, computer flag, removal flag |
| `Pyramid` | 63 | the guards (40 Gold Golems split; 20 Diamond Golems in 2 stacks), the spell level (5), the looted bonus (−2 luck, mark 1) | the spell (drawn; none: A-20) |
| `Monster` (amended) | 54, 71–75, 162–164 | — | the treasure's artifact (map) |

`ObjectKind::effect` gains an `Effect` per kind (the one place a visit is wired, T13b's rule).
Events are not visited through it (§12).

### 9. The draws when the game is built (ADR-0016 point 4; GAM-SET-001's stream)

From the setup stream, after today's draws (piles, campfires, monsters), in two passes in object
order (the original resolves the random artifacts first, then walks the rest, §3.1):

1. **The random artifacts** (65–69): uniform among the ruleset's artifacts of the type's classes,
   never Special, banned, placed (a class-5 object) or drawn before; none left, the marks restart
   from the bans; still none, any class (D-31, D-32).
2. **Every other draw**, each object's in its own shape: a chest (r in 0–99 by the type's
   chances; an artifact from the same pool and marks, class treasure); the Tree's price (one of
   three, uniform); a Shrine without a spell (uniform among its level's hero spells with a school,
   not banned and not marked; the marks start as the bans and clear per level when the level is
   used up: D-34); a Witch Hut (the r-th of its mask less the bans, uniform; none: refuses every
   visit); a Scholar (kind 0–2 uniform when random; its id among the unbanned; a fallback primary
   0–3 for kinds 1 and 2); a University (four distinct unbanned skills, Necromancy included: D-36;
   fewer left, all of them: A-22 fixed); a Find (its table, the artifact from the pool); a Pyramid
   (uniform among the level-5 hero spells with a school, not banned, not `granted_by_name_only`;
   none: no spell, A-20 fixed: it fights and gives nothing).

The draws' order is the engine's (decision 17), so the original's dropped draws (a placed
artifact's, §3.1) have no counterpart. The tutorial is not modelled (decision 134).

### 10. The state (ADR-0016 point 1; decision 133)

- **The visits' block** (`Option<Arc<Visits>>`, none until the first visit that writes one):
  - `marks: BTreeMap<HeroId, BitGrid>` — a hero's once-per-hero marks, a bit per object index
    (the grid of `Occupancy::removed`'s shape: 32 objects a row);
  - `known: Vec<BitGrid>` — per player, a bit per object index: the player's knowledge (§13);
  - `emptied: BitGrid` — a find searched, a Pyramid looted.
- **The army left** becomes a two-level table of leaves (64 × 64 entries a mid table, leaves of
  16 shared armies), so a write copies at most a top table, a mid table and a leaf; its fork cost
  stays one reference count.
- **Removals** stay `Occupancy`'s (chests, artifacts, scrolls, Scholars, boxes, events with the
  flag).
- **Sizes** (the XL map's two levels, 41 472 tiles, one object a tile; T55b's measurement, a
  throwaway callgrind program on `BitGrid` and stand-in tables, release build, not committed):

| Candidate | A fork | Its first write |
|---|---|---|
| marks in one map by hero id (8 / 48 / 156 heroes, 30 marks each) | 19 Ir | 2 727 / 7 973 / 26 233 Ir |
| marks on each hero (8 / 48 heroes) | 453 / 1 781 Ir | 1 909 / 3 210 Ir |
| knowledge, 8 players, every object known | 20 Ir | 2 238 Ir |
| an object bit grid (`emptied`) | 40 Ir | 898 Ir |
| army left, today's one map (30 / 1 000 / 41 472 entries) | 19 Ir | 11 086 / 388 877 / 16 459 140 Ir |
| army left, two levels, leaves of 64 armies (30 / 41 472) | 16 Ir | 16 498 / 32 016 Ir |
| the same, armies shared, leaves of 64 / 16 (41 472) | 16 Ir | 4 652 / 4 197 Ir |

  (A write includes about 700 Ir of the value's own allocation.) Today's env fork on the XL map
  is about 3 560 Ir (`fork-xl-144`): the block adds one reference count (about 0.5 %, inside the
  gate's 2 %); the marks on each hero would add 13 %. The build measures again with the real
  types (`fork-xl-144`, and a fork with every tile a box, §16).
- **The hash** walks the block in canonical order (hero ids, players, object indices); the
  adventure state hash's version moves.

### 11. The visits (`h3-adventure`)

Each kind's visit, in the original's order (survey §2 to §7), returns its outcome (ADR-0017
point 10); a refusal is an outcome. Where the original records the visitor's team mark the
visitor's owner's knowledge bit is set at that point (§13); the hero's once-per-hero mark is set
only when it gives.

- **Chest** (ADV-PIK-005): an artifact with room (§4) → `AddArtifact`, removed; else (gold, or an
  artifact with a full backpack as 1 000 gold) a **choice** decision: gold to the owner or
  `learning_experience(gold − 500)` to the hero; removed either way (the removal with the visit,
  the choice's answer gives).
- **Artifact** (ADV-PIK-006, -007): a full backpack refuses (nothing changes); guards → the
  `Fight` offer, a win's rest picks it up; else `AddArtifact`, removed.
- **Scroll** (ADV-PIK-008): as the artifact, its guards asking only with a text (else a battle at
  once), the Spell Scroll with its spell.
- **Monster's treasure** (ADV-MON-011): after a won battle or a join, the artifact to the hero
  with room, else lost; then the resources (today's).
- **Gifts** (ADV-TCH-001 to -003): a hero's mark set → refused; the condition (the Library) or
  the price unpaid (the Schools, the Tree's, before asking) → refused; a question or choice when
  the type has one; then the gain, the mark. The Learning Stone `learning_experience(1 000)`; the
  Tree `next_level_experience`.
- **Shrine, Witch Hut** (ADV-TCH-004, -005): knowledge first; refused by §4's tests (or no skill
  drawn); else `LearnSpell` / `RaiseSkill { level: 1 }`; no mark, every visit.
- **Scholar** (ADV-TCH-006): `scholar_reward`, removed.
- **University** (ADV-TCH-007): opens the decision (§14) over its four skills; knowledge set.
- **Finds** (ADV-PIK-009 to -012): emptied → nothing (the Tomb's bonus still applies); else the
  contents with §8.2's full-backpack turns; the Tomb asks first and adds `Bonus { morale: −3,
  mark: Some(0) }` whether or not it held something; emptied.
- **Pandora's Box** (ADV-PIK-013): the offer; accepted, guards → a battle (announced, unsplit),
  its rest after the levels; then the record (ADV-RWD-001, §11.1); removed.
- **Pyramid** (ADV-TCH-008): the offer at every visit; knowledge after it; looted → `Bonus { luck:
  −2, mark: Some(1) }`; else the monster battle (§11.2), its rest after the levels: looted (emptied),
  the spell by `learns_spell(known_counts = false)` or lost.

**11.1 The record** (ADV-RWD-001, the adventure's half): `grant` (§3) for the hero; the resources
(gains, and losses that floor at 0); the creatures slot by slot onto the army (a stack of its
creature, else the first empty slot, the cap of ADV-ARM-007), what finds no room into **one army
dialog** after the last stack; the experience (`Granted::experience`) is carried in the outcome
for the game to give last (§14). The outcome reports whether anything was given (an empty box's
message).

**11.2 The monster battle with a second creature** (ADV-MON-012, D-40): the Pyramid's guards
through `split` with the first creature's count, n less the second creature's stacks (at least
1), and the order table 0x63DF94's row for n (slot by slot: n = 1 D D – G – – –; 2 D D G – G – –;
3 D G D G – G –; 4 G D G D G – G; 5 G G D G D G G); the split's draws as a monster's
(`ADV-QUIRK-6`). Losses are not kept (no army left).

### 12. Events and the move (ADV-OBJ-007, ADV-MOV-011; A-23)

- **Static:** an event is a `MapObject` without a visitable mask in the movement's sense: the
  static map keeps a separate bit per tile ("an event lies here") and the event's index, so the
  movement rules, the occupancy and the observation see nothing unless a rule reads that bit.
- **The step:** after a step onto a tile with a live event whose colours hold the mover's colour
  (a non-human player only with the computer flag, T54's `human`), the event **fires**: the move
  ends on the tile with the points left after the step (the rest of the path dropped; the
  outcome says so), the guards fight at once (a guarded pile's battle, its rest after the levels),
  else the record (§11.1); removed with its flag. A move is never rejected because of an event; a
  hero that starts its move on an event does not fire it; a non-firing event changes nothing.
- **`Reach`** never reads the bit, in every profile: batch E7's E7-23 showed a path crossing a
  hidden event and the re-read found an event's tile no stop for the original's pathfinder (its
  entrance bit cleared at the placement), so A-23 is withdrawn (decision 131; T58, survey
  `E08-rewards.md` §15). No flag, no `ADV-QUIRK-22`.
- **Security:** nothing leaks through the legal moves, so `docs/06-security.md` lists nothing new
  for events. A property test: two games that differ by a hidden event give every player the same
  observation and legal moves until it fires.
- **Open point** (T62's start): the order of an event and a monster guarding its tile on the same
  step (the original's `hero_move_step` tests the event at 0x4811A3 and fires it at 0x481741; the
  guard's attack was not placed in that order).

### 13. Information (ADV-VIS-006, ADV-FOG-007)

- **Object views:** unchanged in what is public (type, anchor, masks): chests, scrolls and boxes
  look alike; events are never in a view; a removal is `ObjectRemoved` where seen.
- **The viewer's knowledge** (a player's `known` bit): its view of the object adds what it learned
  (a Shrine's spell, a hut's skill, a Tree's price, a University's skills, a find emptied, a
  Pyramid looted). Recorded where the original records the team's mark: the Shrines, the Witch
  Hut, the Schools and the Tree before their tests, the Corpse, the Wagon and the Lean To at their
  visit, the Pyramid and the Tomb after their question (the Tomb's order not read: as the
  Pyramid's), the University when it opens. Allies' knowledge comes with teams (E10).
- **Own heroes:** each hero's once-per-hero marks, named by type and anchor as the views name
  objects (a marked object stood on a tile the hero explored, so nothing hidden shows), and its
  bonuses; the open decision's view (§14).
- **Events to viewers:** a visit's gains and changes to the owner alone; an event's firing (its
  record's parts given) to its visitor's owner alone; other players see the steps they see.

## Part C — the game, the env, batch E7 and the cut

### 14. The decisions and their answers (`h3-game`; GAM-DEC-005, -006, -004 amended)

- **Typed answers** (decision 138; ADR-0017 point 6): `Answer` becomes an enum by
  decision kind, each variant carrying its kind's own type; every decision's answers move,
  increments 2 to 6's with increment 7's, in one replay format bump (v7):

  | Variant | Its type | For | Default |
  |---|---|---|---|
  | `Offer(Consent)` | `Accept`, `Decline` | every yes-or-no offer: the Cartographer, the border guard, a monster's join, join for gold, pursuit, a guarded object's `Fight`; new: `OpenBox`, `SearchPyramid`, `SearchTomb`, `PayTree { price }` | `Decline` |
  | `LevelUp(SkillOffer)` | `First`, `Second` | a level-up with two offers | the original's choice |
  | `Chest(ChestChoice)` | `Gold`, `Experience` | a treasure chest's gold | `Gold` |
  | `Arena(ArenaChoice)` | `Attack`, `Defense` | the Arena (no leaving) | `Attack` |
  | `School(SchoolChoice)` | `First`, `Second`, `Leave` (the School's two skills by its type's order) | a School of Magic or War | `Leave` |
  | `University(UniversityAnswer)` | `Buy(slot)`, `Leave` | the University (open until `Leave`) | `Leave` |
  | `Close` | — | a meeting, an army dialog (increment 6's `Decline`) | `Close` |

  `WrongAnswer` stays for a variant that is not the open decision's kind; a University's `Buy` of
  a closed slot or one the treasury cannot pay is rejected (`SlotClosed`, `CannotPay`) and changes
  nothing. The system actor's `answer_default` (its closed command set, ADR-0017 point 2) gives
  the default per kind.
- **`Decision`** gains: `Choice { owner, hero, object, kind, amounts }` (the chest: gold and
  experience; the Arena; a School: its two skills and price); `University { owner, hero, object,
  skills, price }`; the new `OfferKind`s beside increment 3's `Fight`; `ArmyDialog` gains the
  group of up to seven stacks and a **rest** (`Option<Rest>`).
- **The rest** (GAM-DEC-007, new; decision 141): what a visit owes after a decision closes or a
  battle's levels are given: the experience (and its level-ups), the pickup or the record after a
  won guards' battle, the Pyramid's spell. The game holds it beside the decisions (the hero, the
  object, what remains) and runs it when the hero has no level-up pending and no army dialog open
  (at once when none). Its owner sees nothing of it but the decisions it leads to.
- **Views** (`DecisionView`): the choice's amounts; the offer's price (the Tree); the University's
  skills and slot states; the dialog's group.
- **`legal_commands`:** each decision's answers: both of a choice (and `Leave` for a School), one
  `Buy` per open slot the treasury can pay and `Leave` for a University, `Close` for a meeting or
  an army dialog.

### 15. The game's application

- The adventure's outcome's hero changes applied in order through `h3-heroes` (as today); the
  resources lost floor at 0 (GAM-RES); `Granted::experience` given after the army dialog or at
  once without one, then its level-ups (increment 4's decisions).
- **After every battle** (GAM-BAT-014, new): `ClearBonuses` for each hero of the battle,
  winner and loser, before the winner's level-ups (the original's `battle_end` clears first).
- The scenario's bans to the adventure's build (§7).

### 16. The env, replays, fixtures, properties, benchmarks

- **The env** (the AI lane's): its action mapping (`h3-sim`'s `adventure/action.rs`, which builds
  `Answer::Accept` and the others from its entries) moves onto the typed answers in T63
  (`cross-lane`, decision 138's consent): its accept, decline, first and second entries onto
  `Offer`, `LevelUp`, `Close` and the new two-way choices; its masks allow at least one answer per
  new decision (the defaults of §14); a University's purchase head is the AI lane's call. Its
  inbox is told (T55b).
- **Replays:** format v7 (the typed answers); the twelve goldens re-recorded; new: random play on a
  rewards fixture `fixtures/maps/e08-rewards-*.h3m` (every class of increment 7, written by
  `h3_map::writer`), and scripted replays of the order-dependent paths: a box with creatures and
  no room (the dialog, then the experience), an event that stops a move, a University's purchases,
  a won guarded box whose level-up gives Wisdom before its spells.
- **Properties** (`h3-map`'s `play.rs`): the plans visit every kind, answer every decision, check
  the invariants after every command (marks only grow, an emptied find gives nothing, a removed
  object never returns); a rejected command leaves the hash; the hidden-event property (§12); **decision 133's test**: a 144 × 144 two-level map with a box on every free tile
  (and one with a Learning Stone on every free tile) builds, plays a few hundred steps with
  visits, and forks within the gate's cost.
- **Benchmarks:** `random-play` on the rewards map (a new workload); `fork-xl-144` with the block
  live (+ about 20 Ir); a fork of the all-boxes map (a new workload); the reach workloads
  unchanged (events are blind to `Reach`, §12).

### 17. Versions and the remaining rule ids

| Version | Change | Task |
|---|---|---|
| core ruleset hash v49, pack format v48 | the mana ceiling; A-21's and A-25's flags (A-23 withdrawn, T58) | T59 (done) |
| details' hash, game state hash v15 | the battle bonuses | T60a (done) |
| adventure ruleset part v12 | the kinds | T61 |
| map import v12 | the classes, bodies, records, bans, the monster's artifact | T61 |
| adventure state hash v13 | the visits' block, the army left's table, the draws on objects | T61, T62 |
| game state hash (next) | the decisions, the rest, the bans | T63 |
| adventure replay format v7 | the typed answers (every decision's) | T63 |

Ids beyond survey §12: **GAM-SET-003** (the scenario's spell and artifact bans as setup inputs),
**GAM-DEC-007** (the rest), **GAM-BAT-014** (the bonuses cleared at a battle's end). Quirks (T59,
in `docs/rules/quirks-adventure.md`): A-21 → `ADV-QUIRK-21`, `quirk.adventure.roe_witch_hut_mask`
(flag 37, `ROE_WITCH_HUT_MASK`); A-25 (decision 145) → `ADV-QUIRK-22`,
`quirk.adventure.full_backpack_refuses` (flag 38, `FULL_BACKPACK_REFUSES`): `faithful` the count
test, `strict-fixed` the placement first (survey §15); A-23 withdrawn (T58: no quirk, no flag);
A-17 to A-20, A-22 and A-24 recorded in the ledger as not reproduced or fixed
(no flags).

### 18. Batch E7

Survey §11's 23 rows in two runs (decision 132), on the harness (`clients/capture/`, common) in SoD
on Easy, hot seat, from maps `h3-experiment` writes; the runs E7a (rows 1–11; row 7 on a map of its
own in the RoE format) and E7b (rows 12–23).

- **The writer** (`h3_map::writer`, this lane's): the object bodies of classes 5, 6, 22, 26, 63,
  65, 88, 93, 100, 101, 104, 105, 108 and 113 (a box's and an event's record, guards, text,
  colours and flags; an artifact's guards and text), a monster's treasure artifact, and an RoE
  map (format 0x0E: its header, no masks).
- **The spec** (`h3-experiment`, battle-owned: `cross-lane`): the rows' objects, heroes' artifacts
  and backpacks (64), skills, spell books, gold; the two maps of E7a and E7b's; the results
  templates `experiments/results/E08-batch-7a.md`, `-7b.md`.
- **The harness** (common; CLT-Q18: this lane builds it with its check and tells the client and
  map-client lanes' inboxes): reads of an object's word on its tile, the game's universities
  (+0x4E3CC), a hero's experience, primary and secondary skills, spell points, bonuses (+0x11A,
  +0x11B) and status flags, the backpack's count, the treasury; the two-picture choice's items
  (0x7809, 0x780A); the University dialog's slots and its purchase button; the path marks (batch
  E1a's) for E7-23.
- **The scoring:** survey §15 (new), each row against §11's prediction; decision 131's outcome
  for A-23 (E7-23); a dispute a row contradicts reverts to the other side (decision 129) or
  becomes an `ADV-Q`; the consensus rows' sources `verified`.

### 19. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** T59's `shared/` task (the ADRs applied with batch E7's outcome, the
mana ceiling, the quirk flags, `docs/03-domain.md`'s terms: reward record, once-per-hero mark,
knowledge, battle bonuses, event); batch E7's spec (`h3-experiment`, `cross-lane`); the harness
(common). Everything else is the lane's own.

**The building tasks**, in rounds of one or two (ids continue the epic's; each task's cut
confirmed at its start). **Batch E7 runs before the build** (decision 132) **and before the shared
task**, so the ADRs are applied once with E7-23's outcome and the disputes' verdicts (in
increment 6 the shared task came first; here the batch decides an ADR clause, A-23):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T56** | Batch E7's maps: the writer's bodies and the RoE format (§18), the spec's fields (`cross-lane`), the two specs and templates. Likely cut in two at its start (the writer; the spec) |
| 2 | **T57** + **T58** | T57: the harness's reads and clicks, the recipes, the runs E7a and E7b (as T47: cut at its start). T58: scored (survey §15); A-23 settled (decision 131); an `ADV-Q` if a row contradicts the reading |
| 3 | **T59** | `shared/`: the three ADRs amended (with T58's outcome: A-23 withdrawn, events blind to `Reach`), the mana ceiling, A-21's and A-25's flags, the glossary |
| 4 | **T60** | `h3-heroes`: part A (§1–§6): the bonuses, the changes, `grant`, the teachers' functions; the details' hash |
| 5 | **T61** | `h3-map` and `h3-adventure`: the import (§7), the kinds (§8), the draws (§9), the state (§10); map import v12, ruleset part v12, adventure state hash v13. Likely cut in two (the import and kinds; the draws and state) |
| 6 | **T62** | `h3-adventure`: the visits (§11), events and the move (§12), information (§13). Likely cut in two (pickups, teachers, finds; boxes, events, the Pyramid) |
| 7 | **T63** | `h3-game`: the decisions, their typed answers (every decision's, migrated; `h3-sim`'s action mapping `cross-lane`) and the rest (§14), the application (§15), replay format v7. Likely cut in two (the typed answers' migration; increment 7's decisions) |
| 8 | **T64** + **T65** | T64: replays, fixtures, properties, benchmarks, `--with-original-data` (§16). T65: the audit against the exit criteria, increment 7 closed, increment 8 refined |

**Increment 7's exit criteria:**
1. Treasure chests, placed and random artifacts (guarded too), Spell Scrolls and the monsters'
   treasure artifacts are taken as the original: the chest's choice, a full backpack refusing,
   turning or losing as survey §8.2; the 8 maps refused for a monster's artifact import further
   (ADV-PIK-005 to -008, ADV-OBJ-006, ADV-MON-011).
2. The twenty classes that teach or hold a find give, ask, charge and refuse as the original;
   once-per-hero marks and the finds' emptying have no bound (ADV-TCH-001 to -007, ADV-PIK-009 to
   -012).
3. Pandora's Box and events give the reward record in the original's order (D-37 to D-39 as batch
   E7 decided); their guards fight as a guarded pile's; an event fires for its colours and stops
   the move; leftover creatures go through the army dialog and the experience comes after it
   (ADV-PIK-013, ADV-OBJ-007, ADV-RWD-001, HER-CHG-003, GAM-DEC-004).
4. The Pyramid draws its spell, asks at every visit, fights with a second creature split as the
   original, teaches or loses its spell and costs a looted visitor −2 luck once (ADV-TCH-008,
   ADV-MON-012).
5. The morale and luck bonuses reach the hero's next battle and end with it (HER-ADV-008,
   GAM-BAT-014); a won guards' battle's levels come before the visit's rest (GAM-DEC-007).
6. Contents and draws stay hidden until a visit; the visitor's owner keeps what it learned; events
   are never observed and blind to `Reach` (A-23 withdrawn by batch E7; ADV-VIS-006, ADV-FOG-007, ADV-MOV-011); a map with
   a box on every tile builds, plays and forks (decision 133).
7. Every new rule has its id, source and tests; A-21 and A-25 in the ledger with their flags
   (A-23 withdrawn, T58), A-17 to A-20, A-22, A-24 recorded; the gates pass (coverage, determinism with the
   re-recorded and new replays, the benchmarks at their baselines and the new ones baselined); the
   env plays on through every new decision.
8. Batch E7 run and scored; D-29 to D-31, D-33, D-34, D-36 to D-40 and the consensus rows
   `verified` or reverted.

### 20. Risks and open points

- **The typed answers' migration** (decision 138) touches every caller that builds an answer:
  the AI lane's action mapping (`cross-lane`), the CLI's replay files, the properties' plans and
  the tests; done first in T63, with the twelve goldens re-recorded, so increment 7's decisions
  land on the new types.
- **The rest across decisions** (§14) is the increment's new mechanism: a visit that spans a
  battle, its level-ups and an army dialog. The scripted replays (§16) pin its order; a forgotten
  path would give a box's experience before its dialog (VCMI's order, D-39).
- **The import's reach:** typing 31 classes makes more maps import whole only where nothing else
  refuses them (towns, random heroes: 128 and 18 maps); the census after T61 says how many.
- **Batch E7's size:** 23 rows, two maps for E7a, a harness grown by about ten reads; the cut of
  T56 and T57 at their starts (as T47's three parts).
- ~~**A-23**~~ withdrawn by T58 (survey `E08-rewards.md` §15): the move's stop at a firing event
  is the rule (§12) and `Reach`'s callers (the env's masks) do not change.
- **The army left's table** (§10) is a change of storage under increment 3's monsters: their
  replays must not move (the hash walks content, ADR-0016 point 2).
- **Not in increment 7** (decisions 127, 136): the creature banks, the Refugee Camp, Seer's Huts
  and quest guards, the map's timed events, the computer's answers and the end-of-game tests after
  an event (E10), casting the adventure spells scrolls hold, teams (allies' knowledge, E10).
