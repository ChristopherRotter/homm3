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
format v47, core ruleset hash v48. T49b reads them (built: §3).

**No data for the positions**: they follow from the slots' order and counts (§2, built). **The expansions**
are the game's configuration (part B §10; decision 104).

### 2. Positions (built in T49a)

Built in T49a (`E08-log.md`, "T49a"): `HeroDetails.worn` by position (`h3_heroes::positions`,
`Position`), the validation with the locks counted per slot, `first_position` (HER-CHG-002),
the capture and a lost machine's artifact in position order, `ArtifactPlace::Worn(Position)`,
`HeroChange::PlaceArtifact`, `wearing` for callers that list artifacts, the import v9.

### 3. Moves, combinations, Titan's book, Scholar (built in T49b)

Built in T49b (`E08-log.md`, "T49b"): `h3_heroes::{ArtifactMove, check_move, move_changes}`
(HER-ART-005), `check_assemble`, `assemble_changes`, `check_disassemble`, `disassemble_changes`
(HER-ART-006), `equip` and the equip in `apply_change` (HER-ART-007), `meeting_spells`
(HER-SPL-007), the import v10 (an equipped Thunder's book). Part B applies them: a move's
changes on both heroes (§8, §10's `Manage`), the assembly and disassembly as commands (§10),
Scholar's spells at the meeting (§8).

### 6. Rule ids and quirks (the hero side); its shared and `cross-lane` needs

| Id | Rule | § |
|---|---|---|
| HER-ART-001 (amended) | worn artifacts on positions; the locks counted per slot | 2 |
| HER-CHG-002 (amended) | a received artifact at its first position, else the backpack's end | 2 |
| HER-ART-004, HER-WAR-002 (restated) | the capture and a lost machine's artifact by position | 2 |
| HER-ART-005 | moves within a hero and between two (built) | 3 |
| HER-ART-006 | assembly and disassembly (the combinations the ruleset has; built) | 3 |
| HER-ART-007 | Titan's Thunder's Spell Book (built) | 3 |
| HER-SPL-007 | Scholar at a meeting (built) | 3 |

No quirk on the hero side (D-28 the original's; Titan's book a rule). The shared part (the two
kinds, the packs, the replay headers) was T45's. **`cross-lane` in the positions task:** the call sites of the battle, AI and client
lanes that build `HeroDetails` (§2's `wearing`), done in T49a with their inboxes told.

## Part B — the map, the game, the batch and the cut

### 7. A hero's army, formation and flag (built in T50)

Built: `E08-log.md`, T50 (ADV-ARM-001 to -007; the cap, decisions 109, 114, 115; adventure state
hash v11, ruleset part v11). For T51: `h3_adventure::{ArmyCommand, ArmyRef, ArmySlot}` and
`AdventureState::{manage_army, split_range, set_formation, set_tactics}` (the group passed in as
`Option<&mut Vec<ArmyStack>>`, by slot like a hero's army); they emit no event: the game tells the
owner (§10).

### 8. The meeting (GAM-MET-001; GAM-DEC-003)

Built: the visit in T50, the decision with Scholar in T51b (`E08-log.md`, T51b): `Decision::Meeting
{ owner, heroes }`, `MeetingOpened` after Scholar's `LearnSpell` changes (the visitor's first),
`Decline` alone listed, `DecisionClosed`; `DecisionView::Meeting`; game state hash v13.

### 9. The army dialog (ADV-MON-008 amended; GAM-DEC-004; `ADV-STUB-2` closed)

Built in T51b (`E08-log.md`, T51b): the join offered whatever room the army has, an accepted join
without room opening `Decision::ArmyDialog { owner, hero, group }` (the group as one stack in slot
0), `ArmyDialogOpened`, the group's slots told as `ArmyRef::Group`, `Decline` closing it with a
`GroupLost` per stack left; `DecisionView::ArmyDialog`.

### 10. Commands, the listing, events, the observation, the expansions (GAM-MAN-001, -002)

Built in T51a and T51b (`E08-log.md`; decisions 116 to 118): `GameCommand::Manage`, the listing
per hero (`legal_management`, `split_range`; ADR-0017 point 6), the events per slot, and with a
meeting or an army dialog open the commands between its two armies (`Together` in `h3-game`'s
`state.rs`). **The expansions** (decision 104, the user at ADV-Q25: "it must depend on the game
configuration that is used to play, i.e. the enabled expansions"): the packs the game's ruleset is
compiled from; the combinations (the SoD pack's artifacts with `parts`) and the fifth misc
position (the SoD pack's slot override) follow them, so no setup field and no map version reach
the rules. A rule the packs cannot express yet (the moat, E09) gains data or a configuration input
then.

### 11. The tactics flag in battle (GAM-BAT-009; ADR-0017 point 11)

Built in T51a (`deny_tactics` in `state/battle.rs`, `Combatant::tactics`): when the side with
the greater Tactics level is a hero whose flag is off, both sides' Tactics go as 0 (equal levels
still cancel, the difference still decides; survey §4). Every player's hero's flag counts (the
original reads a human side's only, decision 16); a side without a hero has no flag.

### 12. The env, the encoding's move, replays, fixtures, properties, benchmarks

*Built in T52* (decision 119; the record in `E08-log.md`): the replays scripted with `h3 adventure
run --script` on a new map, `e08-army-12.h3m`; the plans' fixtures as an `Exchange` of the first
hero; the fork measured as `fork-meeting-xl-144` (+0.17 % over the battle's) and the listing as
`manage-army-12`; the env's test stays the AI lane's (its inbox).

- **The env is the AI lane's** since T45 (decisions 99, 103; ADR-0017 point 8); the replay model
  (`replay.rs`), `h3-cli`'s adventure commands, the golden replays and `benches/adventure/` stay
  this lane's. Increment 6 changes only the env's masks: `Decline` while a meeting or an army dialog is open
  (§8, §9), so the rollout and the replays' recorder never stall in a meeting; the masks are built
  from `legal_commands`, so this may need only the AI lane's test (its inbox; this lane tells it
  when T51 lands).
- **Replays** (format v6, built in T51a): the management commands (`Manage`); two new golden replays, scripted
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
  `legal_management` during a meeting (a new workload, T52); the four gated workloads at their
  baselines (the walker's search untouched).

### 13. Versions

| Version | Change | Task |
|---|---|---|
| core ruleset hash v48, pack format v47 | two effect kinds, the packs' data | T45 (done) |
| game state hash v12 | the details' positions | T49a (done) |
| adventure state hash v11, adventure ruleset part v11 | the tactics flag; a stack's most creatures | T50 (done) |
| game state hash v13 | the meeting and army dialog decisions | T51b (done) |
| map import v9 | positions | T49a (done) |
| map import v10 | Titan's Thunder's book at load | T49b (done) |
| adventure replay format v6 | `Manage` | T51a (done) |
| action encoding | unchanged (v4); the AI lane's from the move on | — |

### 14. Batch E6

Ten of survey §9's eleven rows (decision 100; E6-5 dropped by decision 109), on the harness (`clients/capture/`: a common path
since CLT-Q18) in SoD on Easy, hot seat, from a map `h3-experiment` writes (battle-owned:
`cross-lane`). The spec (the rows' setups, `experiments/specs/E08-batch-6*.toml`) and the
results template are T46; the harness's new clicks and reads (survey §9's list), the recipes and
the run T47 (as T36 and T37 for batch E5); the scoring T48 (an ADV-Q if a
row contradicts the reading; A-16's E6-4 decides only the ledger's text, the rule being fixed by
decision 98). T47 is cut in T47a (the harness's clicks and reads), T47b1 (the recipe, checked
with `fill`) and T47b2 (the run; decisions 108, 110). Scored in T48 (survey §13): every row as
the reading, no question.

### 15. Shared and `cross-lane` work; the building tasks; the exit criteria

**Shared and `cross-lane`:** round 1's `shared/` task done (T45: the ADRs, the two kinds, the
packs, the replay headers, the encoding's move); batch E6's spec (`h3-experiment`, `cross-lane`);
the positions' call sites in other lanes' code (T49, `cross-lane`). The harness is common. Everything
else is the lane's own.

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint). As in increment 5, **batch E6 runs before the build**: it tests the
original only, and its answers (A-16's text, the assembly's questions, the tactics
flag) land once:

| Round | Tasks | Content |
|---|---|---|
| 1 | **T45** + **T46** | T45 (done): the shared groundwork (`shared/`, `cross-lane`): ADR-0013, -0016, -0017 amended; `meeting_spells`, `gives_spell_book`; the packs; core ruleset hash v48; the encoding's move to the AI lane (`lanes.toml`, the versions table, the global HANDOFF, the AI lane's inbox). T46: batch E6's spec and results template (§14; `h3-experiment` `cross-lane`) |
| 2 | **T47** + **T48** | T47: the harness's clicks and reads (the hero, exchange, split, creature and army dialogs; the spells, backpack, flags and tactics reads), the recipes and the run. T48: scored; the survey's §8 and the rules' sources; an ADV-Q if a row contradicts the reading |
| 3 | **T49** | `h3-heroes`: positions (§2: the model, validation, the first position, the capture, the import v9, the constructor and the other lanes' call sites, `cross-lane`), moves, combinations and Titan's book, Scholar (§3). The largest task: cut in T49a (positions, §2; done) and T49b (§3; decision 111; done) |
| 4 | **T50** + **T51** | T50 (done whole, decision 113): `h3-adventure`: the army commands, the split's bounds, dismissal, the formation and the flag, the cap, the meeting's visit, the join without room's report (§7, §8 visit, §9); adventure state hash v11, ruleset part v11. T51 in two (decision 116): T51a (done): `h3-game`: `Manage` on one hero, its per-hero listing and `split_range` (decision 117), events, the tactics flag in battle, replay format v6 (§10, §11); T51b (done whole, decision 118): the meeting and army dialog decisions, Scholar's application, the commands between two armies, the join offered without room (§8 to §10); game state hash v13; `ADV-STUB-2` closed |
| 5 | **T52** + **T53** | T52 (done whole, decision 119): replays v6 and the two new goldens (scripted), fixtures, plans, properties, benchmarks, `--with-original-data` (§12); the env's masks the AI lane's. T53: the audit against the exit criteria, increment 6 closed, increment 7 refined |

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
7. Batch E6 run and scored; D-27, D-28, A-16 and the consensus rows `verified` or reopened.

### 16. Risks and open points

- **The listing's size** (§10): settled at T51's start by listing per hero (decision 117,
  ADR-0017 point 6), so `legal_commands` and the env's rollout stay as they were; a hero with
  seven stacks and a full backpack still lists a few thousand commands, which T52 measures.
  T52: in a meeting a hero with seven stacks and twelve backpack artifacts lists 270 commands,
  its mate 92; both listings and the 196 split ranges of the two armies cost about 800 000
  instructions (`manage-army-12`), about 0.1 ms in release.
- **The handover of the env**: the AI lane takes an env whose encoding lacks increment 6's
  commands; agents trained before E13's macro-actions manage nothing. The masks' `Decline` keeps
  the env playable; the encoding's growth is the AI lane's call (its inbox).
- **A stack's count cap**: settled at T50's start (decisions 109, 114, 115; ADR-0016 point 3) and
  built (ADV-ARM-007); recruiting and growth keep it when they come.
- **A-16 and the split's bounds** rest on the code reading; E6-4 confirmed them (T48: 1 and 9). The engine's
  [1, t − 1] with the move reproduces the original's reachable outcomes (§7): a row that showed
  the dialog allowing what the click refuses would reopen it.
- **The expansions** (decision 104): a game's packs decide; the import's map version does not
  reach the rules. The moat (E09) is the next rule that differs by expansion.
- **Not in increment 6** (survey §12 point 9, decision 100): allied heroes' meetings and one-way
  trades (teams, E10), the computer player's join without room and meetings (E10), hero
  dismissal and the creature dialog's upgrade (E09), the backpack's rotation as a command of its
  own, the other callers of the army dialog (rewards, increment 7).
