# E08 design note: increment 4

A handover, not binding (`docs/adr/README.md`): the working detail behind increment 4's
amendments of ADR-0013 (point 12; drafted in T24b1) and ADR-0017 (drafted in T24b2), accepted
at ADV-Q16 (the epic's decision 58) and applied by T26a (#325), from the answers at ADV-Q15 and
ADV-Q17 (decisions 55, 60). The task that builds a section deletes it; the file goes when the epic
closes. Facts and sources: survey `docs/rules/surveys/E08-progression.md` (§ numbers are the
survey's unless they name this note).
Increment 3's note was pruned by T24b1: every section was built, its records are in `E08-log.md`
(T18a to T22b) and its last versions in §15 below.

Written in two parts (decision 56): **part A**, the hero side (`h3-heroes`), by T24b1; **part B**,
the game side and the cut, by T24b2 (decision 57).

## Part A — the hero side (`h3-heroes`; T24b1)

Everything in part A is a rule of `h3-heroes`, the hero model beneath the contexts (ADR-0013
point 2): pure functions of a hero's details, the ruleset and explicit inputs (the scenario's
level limit and banned skills, a draw source). Who calls them, when, with which source, and what a
player sees is the game's (part B).

### 1. The data

- **Built by T26b** (`E08-log.md`; `packs/README.md`, "Hero parameters"): `HeroParams` holds the
  experience of levels 1 to `max_level + 1` (the pack's twelve, then each step + step / 5; 75
  entries, the last the bound of §2), `max_level` 74, `wisdom` and `schools` (the forced skills,
  resolved keys in the original's order); `HeroClass::offer_intervals` (`[wisdom, school]`: 3 and
  3 for the eight classes 0x4DAD74 lists, 6 and 4 else); `[[hero_class_override]]`, with A-10's
  data quirk in the `ab` pack (`quirk.content.elementalist_offer_intervals`, `ADV-QUIRK-10`).
  The level of an experience is then a search of `experience` (§2).
- **The adventure part's terrains** (`packs/*/adventure.toml`, adventure-owned; decision 60) gain
  their straight step's cost by Pathfinding level 1–3 beside the cost without it (the original's
  `step_cost_table` 0x63E510: sand and snow 125, 100, 100; swamp 150, 125, 100; rough 100, 100,
  100; the others as without), for the evaluation's shares (§8, §12) now and increment 5's
  movement later; the adventure ruleset part's version moves (T29).
- **Not data**: the scenario's level limit and banned skills are the map's (`RawMap::level_limit`,
  `banned_skills`), inputs from the game (§12).

### 2. to 7. Built by T27

Built by T27 (2026-10-20, `adventure/e08-t27`, decision 62; `E08-log.md`): `h3_heroes`'
`level` (`experience_for`, `experience_bound`, `level_of`, `experience_cap`,
`gained_under_limit`, `pending_levels`; HER-EXP-002, -003), `level_up` (`Draws`, the
`OriginalDraws` generator, `draw_level_up` → `LevelUp` with `Offers`, `Choice`; HER-LVL-001 to
-004) and `HeroChange::GainLevel` with `apply_change_limited` (HER-LVL-005); `HeroDetails.offered`
(`OfferLevels`: the original's +0x90 and +0x3F) and the details' checks (the experience at most
the bound, the level at most the experience's, the offers' levels at most the level, in
`h3-heroes`: decision 62). What the game side needs from it: a level-up is drawn by
`draw_level_up(details, ruleset, banned, draws)` while `pending_levels` > 0, answered by
`LevelUp::change(choice)` (the game checks the answer against its `LevelUp`), gains go through
`apply_change_limited` with the map's limit. One difference from §3: a gain under a limit lowers
the experience no further than the threshold of the hero's level (the details stay valid); the
original keeps such a hero at its level with less experience, a state the game's setup does not
make (D-21).

### 8. The original's choice between two offers (HER-LVL-006; D-20)

- Decision 55: **the computer's evaluation is ported**: `ai_level_up_choice` (0x52BBD0) with
  `ai_secondary_skill_value` (0x524690, 0x684 bytes) and `ai_takes_new_secondary_skill` (0x524DD0,
  0xF3 bytes): both offers known or both new, the higher of the two values, the first on a tie;
  otherwise 0x524DD0 decides whether the new skill is taken (§2.5, §9).
- **Read by T25** (survey §10): no draws; a skill's value (§10.2, the flag-clear column at
  load) from the hero's skills, Wisdom, its primary skills with the worn artifacts (Spell Power
  and Knowledge held to 1–99, Attack to 0–99), two tables (Archery's, Estates'), the army's
  worth A = 1 000 and B = 500 and **the map's terrain shares** (§10.3: water share and the rough
  share per Pathfinding level, over the coast-flagged tiles only: A-11, reproduced in every
  profile, decision 60); a new skill against an advance by 0x524DD0's ranking (§10.5). E3-6's four
  choices follow from it (§10.6).
- **Its place**: an `h3-heroes` rule (the hero model beneath the contexts), so the game's levels at
  load, the system actor's default (§13) and later E10's computer players call the same function.
  **Its inputs**: the hero's details and class, the two offers, the army's worth (the setup's
  1 000 and 500; E10 passes its own with the flag-set path) and the map's shares, which the game
  computes once from the static map at the build (§12) and passes in. Only the flag-clear path is
  built in increment 4; the flag-set one reads E10's computer-player state (§10.4).
- **The shares' arithmetic** is the original's f64 (counts to f64, a division, × A, / 4 or / 2,
  truncated; a NaN, on a map without a coast, gives 0): through `kernel::num`, or an integer form
  a test shows equal for every count the maps can have.
- **The levels at load** seen from the hero (the game's sequence is GAM-SET-002, §12): after the
  custom skills, army, spells and artifacts, the experience set to 0 and the custom experience
  gained (§3), then each pending level drawn with the original's source and S (§4), two offers
  chosen by this rule with the setup's defaults, one learned, none the primary only, each applied
  as `GainLevel` (§7); the spell points after (HER-MAN-001). In `strict-fixed` the owner's stream
  draws (D-14's flag).

### 9. Rule ids and quirks proposed

- Rules (survey §8, ids written when built): HER-EXP-002 (§2), HER-EXP-003 (§3), HER-LVL-001
  (§5), HER-LVL-002 (§4), HER-LVL-003 and -004 (§6), HER-LVL-005 (§7), and **HER-LVL-006** (new,
  §8: the original's choice between two offers). HER-DET-001 and HER-CHG-001 gain the checks of
  §7. The game's GAM-HER-002, GAM-DEC-002, GAM-SET-002, GAM-BAT-008: part B.
- Quirks for `docs/rules/quirks-adventure.md` (confirmed by the second reading, §9), each written
  with its rule (`ADV-QUIRK-11` to `-13` by T27): **ADV-QUIRK-9** a level's draws follow the hero's seed byte and the level (D-14;
  fix-optional, its flag the game's: §11); **ADV-QUIRK-10** the Elementalist's might intervals
  (A-10; fix-optional, the data quirk `quirk.content.elementalist_offer_intervals`, §1);
  **ADV-QUIRK-11** the levels past 74 wrap (A-7), **ADV-QUIRK-12** a limit above 75 caps at a
  wrapped threshold (A-8), **ADV-QUIRK-13** a primary skill wraps at 128 (A-9): fixed, no flag;
  **ADV-QUIRK-14** the computer's terrain shares count only coast-flagged tiles (A-11, survey
  §10.3): the original's in every profile, no flag (decision 60), written with T28.
  The two flags are registered (T26b): 27 `quirk.adventure.level_up_from_seed`, 28
  `quirk.content.elementalist_offer_intervals`; `ADV-QUIRK-10`'s entry is written (its data
  quirk is complete), `ADV-QUIRK-9`'s waits for its rule (T29).

### 10. The hero side's shared and `cross-lane` needs (for the cut, §17)

Built by T26b (`shared/e08-t26b`, `cross-lane`): the data model and the flags in `h3-content`,
the packs and the harvester's gap-fill, both lanes' replay headers. What remains is the lane's own
(`h3-heroes`, T27 and T28; the game state hash with the details, §15).

## Part B — the game side (T24b2)

Rules of `h3-game` (and the import's, the env's) around part A's: what the game keeps, when it
calls `h3-heroes`, who answers, what a player sees. ADR-0017's draft holds the decisions.

### 11. The level seed and the streams (GAM-HER-002; D-14, `ADV-QUIRK-9`)

- **The level seed**: when the store is built (`GameState::new`), each hero's seed byte is drawn
  from the setup stream, `range_inclusive(1, 255)` as the original's `random_between(1, 255)`
  (0x4BFEC2), in hero id order after the hero's own defaults (the experience 40–90 when not
  custom), before the levels at load (§12); the setup stream's later draws move with it (the
  replays are re-recorded once, §14). Kept beside the details in an `Arc<[u8]>` by
  `HeroId` (never written after the build, so a fork shares it), in the state hash, in no
  observation or event. Not in `HeroDetails` (part A §7).
- **The source of a level's draws** (part A §4): with the flag `quirk.adventure.level_up_from_seed`
  on (`faithful`) the original's generator from the seed and the new level, at load and in play
  alike; off (`strict-fixed`) the setup stream at load and the owner's stream in play. Registered
  by T26b (position 27, on in `faithful`; no code reads it yet); `ADV-QUIRK-9`'s ledger entry
  with T29.
- **Hygiene and isolation** (the properties, §14): the seeds and a level's offers never reach
  another player's observation or events; in `strict-fixed` a level-up draws only from its
  owner's stream.

### 12. The map's level limit and banned skills; the import; levels at load (GAM-SET-002; D-21)

- **The setup**: `GameSetup` gains `level_limit` (0 none; the map header's byte, AB and SoD) and
  `banned_skills` (the map's set, SoD; `RawMap::banned_skills`); `GameState` keeps both, static,
  in the state hash. The import maps the header's bits to the ruleset's skills by the original's
  ids (the pack order, part A §6).
- **The import** (`h3-map`, `MAP_IMPORT_VERSION` 8): `Unsupported::HeroExperience` goes; a
  placed hero's custom experience passes as it is (`HeroDetailsSetup.experience`), so the
  census's 3 maps more import whole (`Adventures of Jared Haret`, `Faeries`, `Knee Deep in the
  Dead`; 5 in all, survey §6.5). With the next change to `import::details` and `state.rs`, the
  stale 0x486590 citations are re-pointed to 0x4D8B30 (the QUEUE inbox); the import's clamp of
  custom primary skills stays (no shipped map has one above 9; the placed hero's are unclamped,
  0x4D8BF1: the engine's details bound them at `primary_max`).
- **Levels at load** (GAM-SET-002, survey §3.2): in `hero_details`, after the skills, spells,
  artifacts and the war machines' artifacts and before the spell points (HER-MAN-001): a hero
  with a custom experience starts at level 1 and experience 0 and gains its experience with the
  limit (part A §3: capped, D-21); then each pending level is drawn (§11) and given: two offers
  chosen by the original's choice with the setup's defaults (part A §8), one learned, none the
  primary skill only. No events (building a game emits none). A hero without a custom experience
  keeps its default 40–90 at level 1.
- **The map's terrain shares** (survey §10.3, A-11): computed once at the build, before the levels
  at load, from the static map (the coast flag, the terrain, the terrains' costs by Pathfinding
  level): over the coast-flagged tiles that are not rock, the water share and the rough share per
  Pathfinding level 0–2 (cost above 100); an `h3-adventure` query the game calls, the result
  passed to the evaluation (§8). Static, so kept beside the limit (or recomputed: the building
  task measures) and in the state hash if kept.
- **The shipped maps**: `verify --with-original-data` (`crates/h3-map/examples/original_data.rs`)
  imports the 5 maps and builds them, so the 9 heroes' 41 level-ups at load run on real data; the
  census line counts them.

### 13. A level-up as a pending decision (GAM-DEC-002); the battle's levels (GAM-BAT-008)

- **When**: after a change raises a hero's experience above its given level's range, the game
  gives its pending levels (GAM-BAT-008: the winner's after the battle's outcome, Necromancy and
  Eagle Eye included, as the original's human path 0x4AE324; later increments' gains the same
  way). One level at a time: the level is drawn (§11); with two offers a decision opens, owned by
  the hero's owner (`Decision::LevelUp { owner, hero, drawn }`, the drawn level kept for its
  answer); with one or none `GainLevel` applies at once and the next level is drawn. A hero
  without an owner (none in increment 4) levels as at load.
- **The answer**: `GameCommand::Answer(Answer::First | Answer::Second)` (the left and right
  offers), from the owner, in its turn or not; `Accept` and `Decline` are rejected for a level-up
  and the new answers for an offer (`GameCommandError::WrongAnswer`, name the building task's).
  The game checks nothing more (the offers are the drawn level's) and applies `GainLevel`; then the
  next pending level of the hero, if any. A player's queue keeps at most one open decision: a
  level-up opens after the battle closes, and a visit's chain that would follow a battle (none in
  increment 4: a beaten monster's treasure is applied with the outcome) waits behind it.
- **The system actor's default** (ADR-0017 point 6's closed set): the original's choice; its
  flag-set path (the original's timeout) reads E10's computer-player state (survey §10.4), so
  until E10 it chooses with the setup's inputs, as at load (decision 60; an `ADV-STUB-<n>` at the
  call, closed by E10).
- **Events** (to the owner only): `LevelGained { player, hero, level, primary, skill }` for every
  level given in play (with one offer, none or after an answer), and `LevelUpOpened { player,
  hero, level, primary, offers }` when a decision opens (names the building task's); the existing
  `HeroChanged` carries `GainExperience` as before.
- **What a player sees**: `GameObservation.decision` becomes a choice of kinds,
  `Option<DecisionView>` with `Offer(Offer)` and `LevelUp { hero, level, primary, offers }` (the
  owner's only, as the offers are); `legal_commands` lists `First` and `Second` while a level-up is
  open. Another player sees nothing of it (ADV-FOG-006: no level in the view of another's hero).

### 14. The env, replays and properties

- **The env** (`h3-sim/src/adventure/`): action encoding v4 appends `FIRST` (11) and `SECOND`
  (12), each with tile 0 alone, legal only while a level-up is the player's open decision;
  `ACCEPT` and `DECLINE` only while an offer is. `random_rollout` answers a level-up at random.
  The observation is `h3_game`'s view (§13).
- **Replays** (format v5): the new answers in the command list (`<player> answer first|second` in
  the CLI's file, `crates/h3-cli/src/adventure/`); the nine golden replays re-recorded once, with
  their options (`crates/h3-cli/tests/adventure_replays.rs`), now answering the level-ups their
  battles give; one replay at least with a level-up answered each way and a hero levelled at load.
- **Fixtures**: the duel's and the XL map's heroes gain experience near a threshold and one a
  custom experience (levels at load), so the replays, the plans and the benchmarks meet
  level-ups (`h3-map`'s `tests/fixtures.rs` writes them).
- **Properties** (`crates/h3-map/src/tests/play.rs`): the driver answers level-ups; the plans
  check after every command that a hero's level is at most its experience's level and that an
  open level-up's hero has a pending level; the hygiene property covers the seeds and the offers;
  a fixed test per profile for the draws' source.
- **Benchmarks**: the three workloads re-baselined if the fixtures move them; the fork shares
  the seeds (no per-fork copy).

### 15. Versions

Increment 3's last: core ruleset hash v45 (the battle lane's since), game state hash v8,
adventure state hash v10, adventure ruleset part v8, map import v7, action encoding v3, adventure
replay format v4. Increment 4: the core ruleset hash v46 and the pack format v44 (T26b, done:
hero parameters, the class field, the override kind, two flags), the game state hash v9 (T27,
done: the details' counters) and v10 (T29: the seeds, the limit and banned skills), the map
import v8 (T29), the action encoding v4 and the replay format v5 (T30 or T31, with the
answers), the adventure ruleset part v9 (T29: the terrains' costs by Pathfinding level). The
adventure state hash does not change. The others bump once, in the task that first changes them.

### 16. Batch E4

The plan is the survey's §7 (ten rows; E4-1 settles D-20 in SoD, most rows predict from the seed
byte). Its tasks: the **spec** (`h3-experiment`, battle-owned: `cross-lane`) gains the map's level
limit (E4-6) and two grounds on one tile (E4-7, if the writer refuses it); the **recipe**
(`clients/capture/batches/`, client-owned) follows the harness's new reads (the client lane's
inbox since T24a2b2: the seed byte and counters, every level-up dialog, `level_up` as a list,
artifacts, the battlefield); a **scorer** script computes each row's predictions under each
source's rule (kept or thrown away at the scoring). The run is on the harness (decision 50).

### 17. Shared and `cross-lane` work; the building tasks

**Shared and `cross-lane`**: T26's `shared/` pull request (the accepted ADR amendments applied,
`h3-content`'s data model and registry, part A §10), with the packs and the harvester's gap-fill
(battle-owned) as `cross-lane` in it or beside it and the golden replays' headers of both lanes
re-recorded; T32a's spec (`h3-experiment`, `cross-lane`); the client lane's harness and recipe
(its inbox). Everything else is the lane's own (`h3-heroes`, `h3-game`, `h3-map`, the env's and
the CLI's adventure sub-paths, the fixtures, replays and benchmarks).

**The building tasks**, in rounds of one or two (task ids continue the epic's; each round's cut
confirmed at its checkpoint):

| Round | Tasks | Content |
|---|---|---|
| 1 | **T25** + **T26** | Done. Cut (decision 59): **T25** (2026-10-20: survey §10; ADV-Q17, decision 60), **T26a** (2026-10-20, #325: ADR-0013 and -0017 amended), **T26b** (2026-10-20, `shared/e08-t26b`, decision 61: the hero parameters' level data and offered skills, the class's `offer_intervals`, `[[hero_class_override]]`, the two flags, the packs and the gap-fill, the bumps and the replays' headers; part A §1, §10) |
| 2 | **T27** + **T28** | T27 done (2026-10-20, `adventure/e08-t27`, decision 62). T27: `h3-heroes`' progression (part A §2–§7: levels, the bound, the limit's cap, the draw source and the original's generator, the primary skill, the offers, `GainLevel`, the counters, the checks; HER-EXP-002, -003, HER-LVL-001 to -005; `ADV-QUIRK-10` to `-13`). T28: the original's choice ported, its flag-clear path (part A §8, HER-LVL-006; survey §10; `ADV-QUIRK-14`) |
| 3 | **T29** + **T30** | T29: the game's setup and build (§11, §12: the seeds, the limit and banned skills, the terrains' costs by Pathfinding level and the map's shares, levels at load, the import's refusal lifted, the 5 shipped maps; GAM-HER-002, GAM-SET-002; `ADV-QUIRK-9`). T30: the level-up as a decision (§13: the answers, the events, the observation, the system actor's default, the battle's levels; GAM-DEC-002, GAM-BAT-008) |
| 4 | **T31** + **T32a** | T31: the env, replays, fixtures, properties and benchmarks (§14). T32a: batch E4's spec and recipe (§16; `cross-lane`) |
| 5 | **T32b** + **T33** | T32b: batch E4 run on the harness. T33: the scoring (the rules it covers `verified` or reopened), the audit against the exit criteria, increment 4 closed, the next increment refined |

**Increment 4's exit criteria**:
1. A hero's experience gives the original's levels, at most 74, capped by the scenario's limit
   (HER-EXP-002, -003); the import keeps the map's limit and banned skills.
2. A level-up draws the original's primary skill and offers from the seed byte and the level in
   `faithful` (the survey's vectors), from the owner's stream in `strict-fixed`; the
   Elementalist's intervals follow the profile.
3. Two offers are a pending decision of the owner, levels in order, each drawn after the previous
   answer; the battle's winner levels; the env, the replays and the plans play them.
4. Placed heroes with a custom experience get their levels at load, chosen by the ported
   evaluation; 5 shipped maps import whole and their 9 heroes get 41 level-ups at load
   (`verify --with-original-data`).
5. Every new rule has its id, source and tests; `ADV-QUIRK-9` to `-13` are in the ledger; the
   gates pass (coverage, determinism with the re-recorded replays, benchmarks).
6. Batch E4 run and scored; the disputes it covers (D-14 to D-21) `verified` or reopened.
