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
  movement later. Built by T29 (adventure ruleset part v9; §11 and §12).
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

### 8. Built by T28

Built by T28 (2026-10-21, `adventure/e08-t28`; `E08-log.md`): `h3_heroes::original_choice(details,
ruleset, level_up, worth, shares) -> Choice` (HER-LVL-006; the flag-clear path of 0x52BBD0,
0x524690, 0x524DD0), `ArmyWorth` (`SETUP`: 1 000 and 500) and `TerrainShares { tiles, water,
rough }` (the map's counts; the evaluation divides them in the original's f64 through
`h3_kernel::arith::X87`, 0 without tiles); `ADV-QUIRK-14`. What the game side needs from it:
**the shares** are counted once from the static map at the build (§12: over the coast-flagged
tiles that are not rock, the water tiles and the tiles whose straight step costs more than 100
by Pathfinding level 0–2, from the terrains' costs by Pathfinding level of §1) and passed in;
the choice takes the hero before the level's answer and the drawn `LevelUp` (it raises the
level's primary skill itself). The flag-set path (E10's computer players) reads
computer-player state: the army's AI values, the effectiveness words, `ai_school_value`
(survey §10.4).

- **The levels at load** seen from the hero (the game's sequence is GAM-SET-002, §12): after the
  custom skills, army, spells and artifacts, the experience set to 0 and the custom experience
  gained (§3), then each pending level drawn with the original's source and S (§4), two offers
  chosen by `original_choice` with `ArmyWorth::SETUP` and the map's shares, one learned, none
  the primary only, each applied as `GainLevel` (§7); the spell points after (HER-MAN-001). In
  `strict-fixed` the owner's stream draws (D-14's flag).

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
  §10.3): the original's in every profile, no flag (decision 60), written by T28.
  The two flags are registered (T26b): 27 `quirk.adventure.level_up_from_seed`, 28
  `quirk.content.elementalist_offer_intervals`; `ADV-QUIRK-10`'s entry is written (its data
  quirk is complete), `ADV-QUIRK-9`'s is written with its rule (GAM-HER-002, T29).

### 10. The hero side's shared and `cross-lane` needs (for the cut, §17)

Built by T26b (`shared/e08-t26b`, `cross-lane`): the data model and the flags in `h3-content`,
the packs and the harvester's gap-fill, both lanes' replay headers. What remains is the lane's own
(`h3-heroes`, T27 and T28; the game state hash with the details, §15).

## Part B — the game side (T24b2)

Rules of `h3-game` (and the import's, the env's) around part A's: what the game keeps, when it
calls `h3-heroes`, who answers, what a player sees. ADR-0017's draft holds the decisions.

### 11. and 12. Built by T29

Built by T29 (2026-10-21, `adventure/e08-t29`, `cross-lane`, decision 63; `E08-log.md`):
`GameSetup.level_limit` and `.banned_skills` (validated: `GameSetupError::BannedSkill`), kept by
`GameState` with each hero's level seed in one shared `Progression` (`state/levels.rs`; game
state hash v10; `level_limit()`, `banned_skills()`); GAM-HER-002 (the seeds, the draws' source
by `ADV-QUIRK-9`'s flag) and GAM-SET-002 (the limit, the bans, the levels at load through
`original_choice` with `ArmyWorth::SETUP`); the battle's experience under the limit
(`apply_change_limited`); the terrains' costs by Pathfinding level (`TerrainMovementDef::
pathfinding`, `AdventureRuleset::step_cost`; adventure ruleset part v9) and the map's shares
counted once by the static map (`StaticMap::terrain_shares`, ADV-MAP-002, not hashed: they follow
from the tiles); the import (v8: the limit, the bans, the refusal lifted; the five shipped maps'
9 heroes get 41 levels at load). **One change from §11**: the seeds are the setup stream's
*last* draws (after the map objects' contents), not each after its hero's defaults, so no earlier
draw moved and the golden replays kept their commands. What T30 needs: `Progression::draw(core,
hero, details, stream)` draws a level from the seed or from the stream given (the owner's in
play); the shares are `state.adventure().map().terrain_shares()`; `Progression::level_limit()`
for every gain.

### 13. Built by T30: a level-up as a pending decision (GAM-DEC-002); the battle's levels (GAM-BAT-008)

Built by T30 (2026-10-21, `adventure/e08-t30`, decision 64: whole, one pull request, with §14's
env answers and the CLI's words; `E08-log.md`) as planned: the winner's pending levels after the
whole outcome (`end_battle`, GAM-BAT-008), given one at a time (`state/levels.rs`'
`give_levels`): each drawn when given or opened (`Progression::draw` with the owner's stream), one
offer or none at once (`LevelGained`), two a `Decision::LevelUp { owner, view, offered }`
(`LevelUpOpened`); `Answer::First`/`Second`, `GameCommandError::WrongAnswer` (after
`BattleOpen`, before `OfferGone`); the observation's `decision` an `Option<DecisionView>`
(`Offer(Offer)`, `LevelUp(LevelUpView { hero, level, primary, offers })`; `GameState::level_up`);
`legal_commands` lists `First` and `Second`; the events to the owner only (`HeroChanged` tells no
level); the system actor's default `h3_game::default_answer` (a query the hosting layer gives as
the player's answer, the server lane's M5: `Decline` for an offer, the original's choice with
`ArmyWorth::SETUP` and the map's shares for a level-up, `ADV-STUB-8` until E10, none for a
battle); game state hash v11. **An open level-up holds the game** (decision 65, ADR-0017 point 6
amended by `shared/e08-t30-adr`): T30's first build let the other players act on, as the ADR
then said, which allowed a battle against a hero whose level-up was still open; now every
other actor's command is rejected (`WaitingForLevelUp(owner)`, `GameState::check_held`) and
lists no legal command, `GameState::waiting_for` names the owner, and the env's `to_move` yields
the owner alone, in its turn or not (a battle's defender that won answers before the player
whose turn runs goes on). The hot-seat handover is the client's, the paused turn timer, the
owner's time limit and the default answer online the server's (their inboxes). The answer's
next level-up takes the answered decision's place. A hero without an owner does not exist in
increment 4: `give_levels` takes the owner.

### 14. Built by T30 and T31: the env, replays, fixtures, properties and benchmarks

The env's answers (action encoding v4) and the replay format v5 came with T30 (decision 64).
Built by T31 (2026-10-21, `adventure/e08-t31`, decision 66: whole, one pull request; `E08-log.md`)
as planned: the fixtures' heroes near a level and red's underground hero levelled at load, the
nine golden replays re-recorded (the five searched ones replaced; seven answer level-ups, five
with both offers), the plans' level checks after every command (`check_progression`), the
hygiene property's other heroes' experience and whom the game waits for, a fixed test of
another player's level-up, the levels at load by profile through the original packs, two
workloads re-baselined. The draws' source in play per profile stays `h3-game`'s fixed tests
(T30): `h3-map` reaches no level seed.

### 15. Versions

Increment 3's last: core ruleset hash v45 (the battle lane's since), game state hash v8,
adventure state hash v10, adventure ruleset part v8, map import v7, action encoding v3, adventure
replay format v4. Increment 4: the core ruleset hash v46 and the pack format v44 (T26b, done:
hero parameters, the class field, the override kind, two flags), the game state hash v9 (T27,
done: the details' counters) and v10 (T29: the seeds, the limit and banned skills), the map
import v8 (T29), the game state hash v11, the action encoding v4 and the replay format v5 (T30:
the level-up decision, its answers), the adventure ruleset part v9 (T29: the terrains' costs by
Pathfinding level). The
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
| 2 | **T27** + **T28** | Done: T27 (2026-10-20, `adventure/e08-t27`, decision 62), T28 (2026-10-21, `adventure/e08-t28`). T27: `h3-heroes`' progression (part A §2–§7: levels, the bound, the limit's cap, the draw source and the original's generator, the primary skill, the offers, `GainLevel`, the counters, the checks; HER-EXP-002, -003, HER-LVL-001 to -005; `ADV-QUIRK-10` to `-13`). T28: the original's choice ported, its flag-clear path (part A §8, HER-LVL-006; survey §10; `ADV-QUIRK-14`) |
| 3 | **T29** + **T30** | Done: T29 (2026-10-21, `adventure/e08-t29`, `cross-lane`, decision 63: whole, one pull request; §11 and §12 built), T30 (2026-10-21, `adventure/e08-t30`, decision 64: whole, one pull request, with §14's env answers and replay format; §13 built). T29: the game's setup and build (§11, §12: the seeds, the limit and banned skills, the terrains' costs by Pathfinding level and the map's shares, levels at load, the import's refusal lifted, the 5 shipped maps; GAM-HER-002, GAM-SET-002; `ADV-QUIRK-9`). T30: the level-up as a decision (§13: the answers, the events, the observation, the system actor's default, the battle's levels; GAM-DEC-002, GAM-BAT-008) |
| 4 | **T31** + **T32a** | T31 done (2026-10-21, `adventure/e08-t31`, decision 66: whole, one pull request; §14 built): the replays, fixtures, properties and benchmarks. T32a: batch E4's spec and recipe (§16; `cross-lane`; its recipe waits for the harness's new reads) |
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
