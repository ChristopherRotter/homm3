# E08 design note: increment 6

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 6's
amendments of ADR-0013 (points 1, 3, 14), ADR-0016 (points 3, 4, 6) and ADR-0017 (points 1, 6, 8,
9, 11), drafted in T44b from the answers at ADV-Q24 (the epic's decisions 93–100), accepted at
ADV-Q25 with the cut (decisions 102–105; decision 104's expansions written in) and applied by T45.
The task that builds a section deletes it; the file goes when the epic closes. Facts and sources:
survey `docs/rules/surveys/E08-army.md` (§ numbers are the survey's unless they name this note).

Increment 5's note is in `E08-log.md` ("Increment 5's design note at its closure", moved by T43);
T45 repointed the two code comments that cited it.

Written whole (decision 101): **part A**, the hero side (`h3-heroes` and the core content); **part
B**, the map, the game, the batch and the cut.

## Part A — the hero side (`h3-heroes`, the core content)

Everything in part A is core content (shared: `h3-content`, `h3-pack`, the packs) or a rule of
`h3-heroes`: pure functions of one or two heroes' details and the ruleset (ADR-0013 point 14 of
the draft). When the game applies them is part B's.

### 1. The data

**Built in T45:** the kinds `meeting_spells { level }` (Scholar: 2, 3, 4) and `gives_spell_book`
(Titan's Thunder) in `h3-content` and `h3-pack` (codes 51, 52), their data in the packs; pack
format v47, core ruleset hash v48. §4 and §5 read them (T49): `gives_spell_book` puts the
ruleset's artifact with the `spell_book` effect into the empty spell book position.

**No data for the positions**: they follow from the slots' order and counts (§2). **The expansions**
are the game's configuration (part B §10; decision 104).

### 2. Positions (HER-ART-001 amended; HER-CHG-002, HER-ART-004, HER-WAR-002 restated)

- `HeroDetails.worn` becomes `Vec<Option<ArtifactInstance>>`, one entry per position:
  `h3_heroes::positions(ruleset)` lists each position's slot, the ruleset's slots in their order,
  each `count` times (19 in SoD: head 0, shoulders 1, neck 2, right hand 3, left hand 4, torso 5,
  rings 6–7, feet 8, misc 9–13, war machines 14–17, spell book 18; the original numbers the fifth
  misc 18 after the book, which no walk can tell apart, ADR-0013 point 1 of the draft). A
  `Position` is an index into it; validation checks the length, each artifact's slot against its
  position's, and that every slot's free positions less the worn combinations' locks stay ≥ 0
  (today's check, counted the same way).
- **The first position that takes it** (`first_position(details, ruleset, artifact)`): the lowest
  empty position of the artifact's slot whose lock check passes with the artifact there (the
  original's `hero_equip_artifact` with -1). HER-CHG-002's received artifact uses it (else the
  backpack's end, as today).
- **The capture** (HER-ART-004): the worn artifacts in position order, then the backpack from its
  end (unchanged in words; the order is now the original's). **A lost war machine** (HER-WAR-002):
  the first position holding the artifact that places it.
- `ArtifactPlace::Worn(Position)` names a position; `HeroChange::AddArtifact` keeps its meaning;
  a new `HeroChange::PlaceArtifact { artifact, at: ArtifactPlace }` puts one at a given empty
  position or backpack index (a move's target), and `RemoveArtifact` at a position leaves it
  empty (the backpack's later entries still move up).
- **The import** (`h3-map`): the map's worn array is by position (`docs/formats/h3m.md`): position
  i of the map's 18 (before SoD) or 19 entries maps to the content's position (the fifth misc,
  the map's 18, to 13). Today's import keeps them as a list in that order: the order is kept, the
  places become positions.
- **A constructor for callers that list artifacts** (`HeroDetails::wearing(ruleset, list)`, each
  placed at its first position): every test and experiment that builds details today (the battle
  lane's `h3-pack-battle` tests and `h3-experiment`, the AI lane's `h3-sim` encounters, the client
  lane's `h3-view` situations; §6) changes one line.

### 3. Moves (HER-ART-005)

`h3_heroes::check_move(from: (&HeroDetails, ArtifactPlace), to: (&HeroDetails, Target), same,
ruleset)` and `move_changes` give the hero changes of a move for one hero (`same`) or two. A target
is a position or a backpack index (0 to the backpack's length; an insertion). The rules (survey
§5.1, §5.2):
- the source holds an artifact; never the Spell Book's position nor the Catapult's;
- a position target: of the artifact's slot, the locks fitting with the artifact there and the
  target's artifact gone; an occupied target's artifact goes back to the source's place (a
  position: the same lock check; a backpack index: inserted there). The original's swap through
  the cursor, as one command;
- a backpack target: the artifact's slot is carried (`backpack = true`: not the war machines, the
  Catapult, the spell book) or it has none (the Grail); the receiving backpack holds fewer than
  the ruleset's 64 (a move within one backpack does not count itself);
- the war machines therefore move between machine positions only (their own slot), the Grail
  between backpacks only;
- a move into a position applies `gives_spell_book` (§1) when the hero has no book; so does every
  other equip (`AddArtifact`, an assembly);
- a scroll keeps its spell (HER-ART-003).

The backpack's arrows rotate the stored array in the original (survey §5.1): a rotation is a set of
moves within the backpack; no command of its own.

### 4. Combinations and Titan's Thunder (HER-ART-006, HER-ART-007)

- **Assembly** at a worn part's position p (`check_assemble`, `assemble_changes`): the artifact at p is a part of combination c (the ruleset's
  `parts`); every part of c is worn (each type once at least). Changes: remove p, then for each
  other part one copy from the positions in descending order (the original walks 18 down to 0;
  within a slot that is the descending index, and parts of one slot are different types), then
  add c at its first position (§2). Its effects are its own and its parts' (as today's
  derivations read a worn combination).
- **Disassembly** of the worn combination at p: remove it; each part in the ruleset's artifact
  order to its first position (the locks freed make room; a part that found none would be lost:
  none can). Never in the backpack (D-28).
- **Titan's Thunder's book** (HER-ART-007): `gives_spell_book` on equip, kept on removal; the book
  never moves (§3), so a hero keeps it. Every hero that equips the Thunder gains one (a meeting
  passes it on): the original's rule (survey §5.4), not a quirk.
- The original's questions (once per player after an equip, at a received part) are the
  assembly command, legal whenever its check passes (part B §10); a computer player's build at
  once is E10's.

### 5. Scholar (HER-SPL-007)

`h3_heroes::meeting_spells(a, b, ruleset) -> (Vec<SpellId>, Vec<SpellId>)`: the spells a and b
learn. s = the greater `meeting_spells` level of the two (§1; none: nothing); either without a
worn spell book (HER-BAT-011's book: the Spell Book worn, or an effect that gives one): nothing;
else a's limit = min(`learn_level(a)`, s) (HER-SPL-005: 2 + Wisdom, at most 5), b's alike; a learns
each spell of b's learned spells (`HeroDetails.spells`) it has not learned, of a level at most a's
limit, in spell order; b alike. The game applies them as `LearnSpell` changes (part B §8). The
original's order (the 70 spells by id, each direction checked per spell) gives the same sets.

### 6. Rule ids and quirks (the hero side); its shared and `cross-lane` needs

| Id | Rule | § |
|---|---|---|
| HER-ART-001 (amended) | worn artifacts on positions; the locks counted per slot | 2 |
| HER-CHG-002 (amended) | a received artifact at its first position, else the backpack's end | 2 |
| HER-ART-004, HER-WAR-002 (restated) | the capture and a lost machine's artifact by position | 2 |
| HER-ART-005 | moves within a hero and between two | 3 |
| HER-ART-006 | assembly and disassembly (the combinations the ruleset has) | 4 |
| HER-ART-007 | Titan's Thunder's Spell Book | 4 |
| HER-SPL-007 | Scholar at a meeting | 5 |

No quirk on the hero side (D-28 the original's; Titan's book a rule). The shared part (the two
kinds, the packs, the replay headers) was T45's. **`cross-lane` in the positions task:** the call sites of the battle, AI and client
lanes that build `HeroDetails` (§2's constructor), as their lanes' inbox notes say.

## Part B — the map, the game, the batch and the cut

### 7. A hero's army, formation and flag (ADV-ARM-001 to -006; ADR-0016 point 3 of the draft)

`h3-adventure` gains the army commands, validated against the trusted player and applied
whole or not at all:
- **The armies**: `ArmyRef::Hero(HeroId)` or `ArmyRef::Group` (the army dialog's, §9); a command
  names a source (army, slot) and a target (army, slot); both armies the player's: the same hero,
  the two heroes of its open meeting, or the hero and the group of its open army dialog.
- **Move** (ADV-ARM-001, -002): the target of the source's creature takes the stack whole (the
  counts added, saturating at the counts' bound); an empty target takes it; any other swaps.
  Between two armies the source's last stack never merges away nor goes to an empty slot (a swap
  is allowed); the group has no protected stack. Counts keep their width (A-15 fixed: the
  original's `army_swap_slots` moves one through 16 bits; ADR-0016 point 3 of the draft).
- **Split** (ADV-ARM-003): `count`, the target's new count, the target empty or of the source's
  creature, t the two counts: any count in [1, t − 1], so both stacks stay. The original's dialog
  (survey §3.3) allows [m₂, t − m₁]; its m₁ and m₂ only ever refuse 0 or t, which leave one of the
  two slots empty: a move (or a merge back), which the engine's move allows exactly where the
  original's click does (§7's move; A-16's refused full split is the click's allowed move, survey
  §3.2). So [1, t − 1] with the move gives the original's outcomes, A-16's included (decision 98:
  the original's bounds, no flag). `h3_adventure::split_range` gives the range for a source and a
  target (the query of §10).
- **Dismiss** (ADV-ARM-004): the stack leaves; refused for a hero's last stack; the group's are
  always allowed.
- **Formation and flag** (ADV-ARM-006): `SetFormation { hero, formation }`, `SetTactics { hero,
  on }`; `HeroOnMap.tactics: bool`, true at the build (the original's 2), in the state hash
  (adventure state hash v11). The map's formation stays the setup's (`ADV-QUIRK-8` as is).
- **ADV-ARM-005**: a move or swap keeps both counts whole (the A-15 regression test).

### 8. The meeting (GAM-MET-001; GAM-DEC-003)

- **The visit** (GAM-MET-001; ADR-0016 point 4 of the draft): `MoveHero`'s last step toward a hero
  of the same player visits it from the neighbouring tile at the step's price (ADV-OBJ-003's
  neighbour visit, as GAM-BAT-004's): the move ends there, the outcome reports
  `Meeting { visitor, visited }` (today's refusal goes: `a_hero_visit_ends_the_move_and_an_own_
  hero_blocks` changes). The objects under the visited hero and the tops not visited from above
  stay GAM-BAT-004's cases. An own hero's tile is still never entered or passed.
- **Scholar** (HER-SPL-007): the game applies both heroes' learned spells as `LearnSpell`
  changes, visitor first, then opens the decision.
- **The decision** (GAM-DEC-003): `Decision::Meeting { owner, heroes: [HeroId; 2] }`; while open
  its owner's legal commands are the management commands on either hero (§10) and `Decline`,
  which closes it; no `EndTurn`, no move; it holds no one else. The game state hash covers it
  (v12).

### 9. The army dialog (ADV-MON-008 amended; GAM-DEC-004; `ADV-STUB-2` closed)

- **The offer**: `answer_to` no longer skips a join without room (the stub's `has_room` test goes):
  the offer to join (free or for gold) is made as for any join.
- **Accepted**: the price paid, the treasure given and the monster removed as today; when
  `army_add_stack`'s rule (a stack of the creature, else a free slot) finds no room, the outcome
  reports `NoRoom { hero, stack }` instead of `CreaturesJoined`, and the game opens
  `Decision::ArmyDialog { owner, hero, group: [Option<ArmyStack>; 7] }` with the whole group in the
  group's first slot (the original's local army).
- **Open**: its owner's legal commands are the army commands between the hero and the group and on
  the hero alone, the hero's formation, flag and artifact commands (the original's portrait opens
  the hero screen), and `Decline`, which closes it: the group's rest is gone (an event to the owner
  with what was lost).
- The order of the original (the dialog, then the treasure, then the removal) shows only in the
  events: no other actor can act in between. The system actor's default answer is `Decline` (as
  for every offer).

### 10. Commands, `legal_commands`, events, the observation, the expansions (GAM-MAN-001, -002)

- **The command** (GAM-MAN-001): `GameCommand::Manage(Manage)` with `Manage::{Army(h3_adventure::
  ArmyCommand), Formation, Tactics, Artifact { from: (HeroId, ArtifactPlace), to: (HeroId,
  Target) }, Assemble { hero, at }, Disassemble { hero, at }}`. Legal in the actor's turn, on its
  heroes on the map; a command between two heroes needs their open meeting; with a meeting or an
  army dialog open, only on its heroes (and the group). The game checks the adventure's or
  `h3-heroes`' rules on every hero it touches, then applies. Rejections are typed
  (`GameCommandError::Manage(..)`), state untouched.
- **`legal_commands`** (GAM-MAN-002): lists every management command of the actor but the
  splits, whose counts `split_range` gives per source and target (a client enumerates them; the
  AI lane's encoding decides how agents choose). The list grows with a meeting (two armies, two
  heroes' artifacts: a few thousand commands at most, bounded by 7 slots, 19 positions and 64
  places); measured in T52 (§12).
- **Events**, every one seen by the owner alone (ADR-0016 point 6 of the draft): `ArmyChanged {
  player, hero, army }` after an army command (the group's own after a group's change), `HeroChanged`
  per hero change (existing), `FormationSet`, `TacticsSet`, `MeetingOpened { heroes, learned }` and
  `DecisionClosed` for the meeting, `ArmyDialogOpened { hero, stack }` and `GroupLost { stacks }`.
  A meeting's step is a step (its viewers as today).
- **The observation** (the owner's): each own hero's tactics flag beside its formation; the open
  meeting (the two heroes) or army dialog (the hero, the group's stacks) as the decision's view.
- **The expansions** (decision 104, the user at ADV-Q25: "it must depend on the game configuration
  that is used to play, i.e. the enabled expansions"): the packs the game's ruleset is compiled
  from. The combinations (the SoD pack's artifacts with `parts`) and the fifth misc position (the
  SoD pack's slot override) follow them, so no setup field and no map version reach the rules: a
  map of RoE played with SoD enabled has both (the original follows the map's `mapKind`). A rule
  the packs cannot express yet (the moat, E09) gains data or a configuration input then.

### 11. The tactics flag in battle (GAM-BAT-009; ADR-0017 point 11 of the draft)

The game's battle setup (GAM-BAT-001): with `BattleHero.tactics` derived as today, when the side
with the greater level is a hero on the map whose flag is off, **both** sides' `tactics` are 0. So
equal levels still cancel, the difference still decides (BAT-TAC-001; the experiments), and a
flag off where the original's would clear the phase clears it (survey §4). A side without a hero
on the map (a monster, a guard) has no flag. Every player's hero's flag counts (the engine has no
computer player, decision 16; the original reads a human side's only). No seam changes; the
battle's observation shows both levels as 0 in such a battle.

### 12. The env, the encoding's move, replays, fixtures, properties, benchmarks

- **The env is the AI lane's** since T45 (decisions 99, 103; ADR-0017 point 8); the replay model
  (`replay.rs`), `h3-cli`'s adventure commands, the golden replays and `benches/adventure/` stay
  this lane's. Increment 6 changes only the env's masks: `Decline` while a meeting or an army dialog is open
  (§8, §9), so the rollout and the replays' recorder never stall in a meeting; the masks are built
  from `legal_commands`, so this may need only the AI lane's test (its inbox; this lane tells it
  when T51 lands).
- **Replays** (format v6): the management commands (`Manage`); two new golden replays, scripted
  (the rollout does not manage): a meeting with Scholar and exchanges of stacks and artifacts, and
  a join into a full army through the army dialog; the ten re-recorded (the hashes move).
- **Fixtures and plans** (`h3-map`'s `play.rs`): two heroes of one player side by side, a full
  army next to a friendly group, artifacts in a backpack, a combination's parts, Scholar and a
  book; the plans meet, exchange, split, dismiss, assemble and fill an army; `check_progression`
  and a new invariant check (every army 1–7 stacks, counts > 0; every hero's details valid) after
  every command.
- **Properties**: a random sequence of legal management commands keeps the invariants and never
  changes another player's view; a rejected command leaves the state's hash; a move and its
  inverse restore the hero (where the original allows the inverse).
- **Benchmarks**: `fork-xl-144` with an open meeting (no new copy per fork beyond the decision's);
  `legal_commands` during a meeting (a new workload, T52); the four gated workloads at their
  baselines (the walker's search untouched).

### 13. Versions

| Version | Change | Task |
|---|---|---|
| core ruleset hash v48, pack format v47 | two effect kinds, the packs' data | T45 (done) |
| details (in the game state hash) | positions | T49 |
| adventure state hash v11 | the tactics flag | T50 |
| game state hash v12 | the meeting and army dialog decisions | T51 |
| map import v9 | positions | T49 |
| adventure replay format v6 | `Manage` | T52 |
| action encoding | unchanged (v4); the AI lane's from the move on | — |

### 14. Batch E6

All eleven rows of survey §9 (decision 100), on the harness (`clients/capture/`: a common path
since CLT-Q18) in SoD on Easy, hot seat, from a map `h3-experiment` writes (battle-owned:
`cross-lane`). The spec (the rows' setups, `experiments/specs/E08-batch-6*.toml`) and the
results template are T46; the harness's new clicks and reads (survey §9's list), the recipes and
the run T47 (as T36 and T37 for batch E5); the scoring T48 (an ADV-Q if a
row contradicts the reading; A-15's E6-5 and A-16's E6-4 decide only the ledger's text, the
rules being fixed by decision 98).

### 15. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** round 1's `shared/` task done (T45: the ADRs, the two kinds, the
packs, the replay headers, the encoding's move); batch E6's spec (`h3-experiment`, `cross-lane`);
the positions' call sites in other lanes' code (T49, `cross-lane`). The harness is common. Everything
else is the lane's own.

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint). As in increment 5, **batch E6 runs before the build**: it tests the
original only, and its answers (A-15's and A-16's texts, the assembly's questions, the tactics
flag) land once:

| Round | Tasks | Content |
|---|---|---|
| 1 | **T45** + **T46** | T45 (done): the shared groundwork (`shared/`, `cross-lane`): ADR-0013, -0016, -0017 amended; `meeting_spells`, `gives_spell_book`; the packs; core ruleset hash v48; the encoding's move to the AI lane (`lanes.toml`, the versions table, the global HANDOFF, the AI lane's inbox). T46: batch E6's spec and results template (§14; `h3-experiment` `cross-lane`) |
| 2 | **T47** + **T48** | T47: the harness's clicks and reads (the hero, exchange, split, creature and army dialogs; the spells, backpack, flags and tactics reads), the recipes and the run. T48: scored; the survey's §8 and the rules' sources; an ADV-Q if a row contradicts the reading |
| 3 | **T49** | `h3-heroes`: positions (§2: the model, validation, the first position, the capture, the import v9, the constructor and the other lanes' call sites, `cross-lane`), moves (§3), combinations and Titan's book (§4), Scholar (§5). The largest task: cut at its start if needed (positions alone; then the rest) |
| 4 | **T50** + **T51** | T50: `h3-adventure`: the army commands, the split's bounds, dismissal, the formation and the flag, the meeting's visit, the join without room's report (§7, §8 visit, §9 offer); adventure state hash v11. T51: `h3-game`: `Manage`, the meeting and army dialog decisions, Scholar's application, `legal_commands` and `split_range`, events, the observation, the tactics flag in battle (§8–§11); game state hash v12; `ADV-STUB-2` closed |
| 5 | **T52** + **T53** | T52: the env's masks (if still this lane's), replays v6 and the two new goldens, fixtures, plans, properties, benchmarks, `--with-original-data` (§12). T53: the audit against the exit criteria, increment 6 closed, increment 7 refined |

**Increment 6's exit criteria:**
1. A player rearranges each hero's army (move, merge, swap, split within the original's bounds,
   dismiss), never leaving a hero without a stack, counts whole (A-15 fixed); sets each hero's
   formation and tactics flag (ADV-ARM-001 to -006).
2. A step toward an own hero meets it at the step's price: Scholar's spells are learned, and
   until the owner closes the meeting both heroes' stacks and artifacts move between them
   (GAM-MET-001, GAM-DEC-003, HER-SPL-007).
3. A join that finds no room opens the army dialog; what the player does not take is lost
   (GAM-DEC-004, ADV-MON-008; `ADV-STUB-2` closed).
4. Worn artifacts sit on the original's positions: moves, the first position that takes an
   artifact, the capture and a lost machine's artifact follow it; the Spell Book and the Catapult
   never move, the machines only between machine positions, the backpack holds 64 in order;
   combinations assemble and disassemble while worn (where the game's configuration enables them), Titan's
   Thunder gives its book (HER-ART-001 to -007).
5. A hero whose flag is off denies the tactics phase it would get, and the two heroes' Tactics
   still cancel as before (GAM-BAT-009).
6. Every new rule has its id, source and tests; A-15 is in the ledger as fixed; the gates pass
   (coverage, determinism with the re-recorded and the two new replays, the benchmarks at their
   baselines and the new one baselined); the env plays on through meetings and army dialogs; the
   encoding is the AI lane's in `lanes.toml`.
7. Batch E6 run and scored; D-27, D-28, A-15, A-16 and the consensus rows `verified` or reopened.

### 16. Risks and open points

- **The positions' ripple**: `HeroDetails.worn` is built in about forty files, a dozen of them
  other lanes' (the battle lane's `h3-pack-battle` tests and `h3-experiment`, the AI lane's
  `h3-sim` encounter, the client lane's `h3-view`). The constructor keeps each one line; T49 is
  `cross-lane` for them and tells their inboxes. The details' hash changes every replay's
  checkpoints (re-recorded with their options, as before).
- **`legal_commands`' size** during a meeting (§10): bounded, but a few thousand entries per call
  would slow a client that lists them every step; T52 measures it, and a per-hero query (the
  original's screens are per hero) replaces the flat list if it hurts.
- **The handover of the env**: the AI lane takes an env whose encoding lacks increment 6's
  commands; agents trained before E13's macro-actions manage nothing. The masks' `Decline` keeps
  the env playable; the encoding's growth is the AI lane's call (its inbox).
- **A-16 and the split's bounds** rest on the code reading; E6-4 checks them. The engine's
  [1, t − 1] with the move reproduces the original's reachable outcomes (§7): a row that showed
  the dialog allowing what the click refuses would reopen it.
- **The expansions** (decision 104): a game's packs decide; the import's map version does not
  reach the rules. The moat (E09) is the next rule that differs by expansion.
- **Not in increment 6** (survey §12 point 9, decision 100): allied heroes' meetings and one-way
  trades (teams, E10), the computer player's join without room and meetings (E10), hero
  dismissal and the creature dialog's upgrade (E09), the backpack's rotation as a command of its
  own, the other callers of the army dialog (rewards, increment 7).
