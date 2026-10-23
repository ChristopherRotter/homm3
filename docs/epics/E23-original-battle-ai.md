# E23 — The original's battle AI

Status: **in progress**, refined 2026-10-19 (BAT-Q52); T1a1 to T1a4 done (the cuts confirmed
2026-10-20), T1c done (BAT-Q53, BAT-Q54); T1a5 done as cut (part C, §7.42 to §7.45); T1a5e (the spell evaluators) done (§7.46); T1b done, ADR-0027 accepted (BAT-Q55); T2 done (T2a the crate and the plain shooters, T2b the area value, the wall shot, the benchmark), the budget
set; T3 done (cut in four: T3-0, T3a, T3b, T3c); T4 done (T4a, T4b in two, T4c, T4d); T1a6 done; slip 25 done; T5 cut in two, T5a done; T5b cut in two, T5b1 done; T5b2 done (in two); T6 next · Follows E05 (the
AI lane, done) · Followed by E06 (training: this policy holds the neutral seat; AI-Q1) ·
The battle lane's, later the AI lane's (BACKLOG).

## Goal

The faithful policy of a battle side that the original's computer plays: the unowned side of a
battle (wandering monsters, an object's guards, a neutral town's garrison) and, later, a computer
player's battles (ADV-Q13; E08 decision 27). It chooses each command as the original's computer
does (shots, melee attacks, moves, waits, defends, creature casts, its hero's spells, retreats
and surrenders), read from the executable. It lives outside the battle's state as the system
actor's policy (ADR-0017 point 4, ADR-0025 point 5): one pure, deterministic function
`fn(&BattleState, &BattleRuleset, BattleSide, &mut Rng) -> Command` (`E05-log.md` §2, the design note at closure), which
the AI lane's batch (`Policy::System`) and the adventure's neutral seat call. It may read the
whole battle state, as the original's does, and draws only from the stream it is given.

## Decisions at the refinement (2026-10-19, the user, BAT-Q52)

1. **The neutral side first.** After the reading, the policy is built first for a side without a
   hero: shots, melee attacks and moves, waits and defends, creature casts. With that part done,
   the AI lane can let the neutral share into training (its A12, `E05-log.md` §4.4). The hero's parts
   follow: its spells, retreat and surrender, the tactics phase, its war machines (the Catapult's
   aim, the First Aid Tent's patient).
2. **A crate of its own** in the battle lane, not a module of `h3-battle`. `h3-battle` keeps
   the computer's numbers its rules already use (`aim.rs`: the automatic turrets, the automatic
   Ballista, the moat avoidance) and gives the new crate a public surface for them. T1b's ADR
   draft names the crate and that surface (`h3-battle-ai` proposed). The crate's registration
   (the workspace, `arch.toml`, `lanes.toml`) is a `shared/` pull request that merges first.
3. **Reading first, spot checks.** The code reading is the main source. A batch row confirms
   only a reading at risk (Q57, `docs/04-rules-and-sources.md` §2: against the wiki or VCMI, at a
   float boundary, on run-time `.TXT` values). No decision-by-decision capture of whole battles.
4. **T1 in two parts.** T1a maps the computer's battle turn in the executable; T1b writes the
   survey and the ADR draft and stops for the user's review.

Unchanged: increments with a user checkpoint each, only the current one refined (Q41's pattern);
the user confirms seams and ADRs; a code reading is insight only, nothing copied (Q29); token
efficiency beats speed.

## Starting point (from E04c and E05)

- **The computer's numbers in `h3-battle`** (`aim.rs`; `installation.md` §7.31, §7.33): the
  battle context 0x435EC0 (the least attack and defense; the desperate mode), the threat records
  0x422B20 (a pathfinder to 127 steps per stack), a creature's worth 0x442A50 and 0x442FD0, a
  shot's value 0x435B90 and 0x435CB0, the shot's damage estimate 0x4359B0 and 0x442780, the
  turret's routine 0x41E190 and the shooter's target 0x41EB80. All are the original's x87
  arithmetic through `kernel::num` (ADR-0003). They decide the automatic turrets (BAT-SIE-020 to
  BAT-SIE-023) and the automatic Ballista (BAT-WAR-008; batch 3h confirmed its target).
- **Named but not read:** the computer's action 0x477EE0 and its choice for the active stack
  0x4221F0; the retreat test 0x41E570; a melee estimate's second half (0x4359B0: Fire Shield, the
  retaliation); the area estimate 0x41EDA0 (Magogs, Liches); the wall shot 0x41EEA0 (Cyclops);
  0x41F580; the AI pathfinder 0x4B3400 (a berserk stack's target, a walker's path among equally
  short ones; BAT-MOV-003). The battle queue's inbox, "For E23", has the leads.
- **The seam's inputs exist:** `BattleSetup.difficulty` and `BattleSetup.human` (ADR-0023 point
  7: read only for the computer's choices). E05 plays both sides as a human player's (AI-Q3), so
  the policy treats the side it plays as the computer's, whatever `human` says.
- **The seats:** `h3_sim::batch::Policy` (Random, Aggressor, Greedy) has no `System` yet; the
  neutral kinds weigh 0 until E23 (`E05-log.md` §4.4, A12). The adventure's system actor plays unowned
  battle sides at random until then (ADR-0017 point 4).
- **Experiments:** an agent plays battles in the original under Wine (`experiments/agent/`, the
  client lane's capture harness, `h3mem.py`'s battle reads; batch 3h ran so).
- **Open questions to settle with it:** BAT-Q33 (who commands a side: the computer's escape from a
  bad-morale freeze); BAT-Q51 (the automatic First Aid Tent's patient, an idle machine's Defend).

## Non-goals

The adventure's decisions around a battle (whether a neutral stack fights, flees or joins; a
computer player's strategy: the adventure lane); the learned policy (E06); VCMI's and HotA's AIs,
except as sources; draw-exactness with the original's `rand()` (where the original draws, the
policy draws from its own stream with the same distribution); the server's default for a battle
whose player is gone (the server lane, M5); auto-combat as a feature (the function will serve it).

## Tasks (outline; refine at the start of each increment; keep status current)

**Increment 1:** T1a, split by size (the user, 2026-10-20: small increments; T1a1's map shows
about 45 KB of unread code beyond the dispatch).

- [x] **T1a1 The entries and the map** (2026-10-20) — the hero's cast at each activation
  (0x422DA0), the stack's action with the retreat and surrender (0x477EE0: the computer never
  surrenders), the stack's turn, a shooter's tactics placement, a walker's turn (no Wait below
  Hard), the order of action; the rest mapped into parts S, O, W, P, C, H, A with sizes
  (`installation.md` §7.34; the log).

**Confirmed at the checkpoint** (the user, 2026-10-20: the cut as proposed, T1a2 as queued; from
§7.34's table; the neutral side first, each about one session's reading; the parts' names as
there):

- [x] **T1a2 The shooters' rest and the walker's choice** (2026-10-20) — part S in full (the area
  value, the area's hexes, the wall shot) and 0x421680 with its branches, draw and helpers by
  role (`installation.md` §7.35; the log).
- **T1a3 The walker's helpers**, cut in three by size (the user, 2026-10-20: about 10.7 KB
  left; the hex scores first), and a fourth for the hero's value in the odds:
  - [x] **T1a3a The hex scores** (2026-10-20) — the enemies' reach 0x420260 with its
    multi-headed branch, the crowding and breath scores 0x41FB60 with the area threat 0x41F920
    (`installation.md` §7.36; the log).
  - [x] **T1a3b The strike hexes and the order's sort** (2026-10-20) — 0x436300 with the
    shooters blocked, the multi-headed and breath strikes (the jousters' longer run-up at a tie);
    part O, 0x423340, the library sort (ties of the two sides unordered above 16 stacks)
    (`installation.md` §7.37; the log).
  - [x] **T1a3c The fallbacks and the ends** (2026-10-20) — the walk 0x41F580 (on Hard and up a
    walker holds back from a worse hex and waits), the two fallbacks, the destination map (as
    Teleport's), the last retaliation's gain, the odds 0x420A80, which compare strength at range
    (§7.35 corrected) (`installation.md` §7.38; the log).
  - [x] **T1a3d The hero's value in the odds** (2026-10-20; the user: all in one part) — the
    caster record 0x526D40 and the best damage spell 0x5275B0 with their helpers: up to four times
    the side's whole strength, from the hero's maximum mana (`installation.md` §7.39; the log).
- **T1a4 The AI pathfinder** — part P (about 4.9 KB with the library's vectors; the walk 0x41F580
  is read), cut in two (the user, 2026-10-20: the search first):
  - [x] **T1a4a The search** (2026-10-20) — 0x4B3400's loop with its queue 0x4B3BB0 and the path's
    trace: a uniform-cost search, first in first out at a tie, directions from top right
    clockwise; batch 3c's M2 and G4 walks follow it, BAT-MOV-003 does not (`installation.md`
    §7.40; the log; the inbox).
  - [x] **T1a4b The head** (2026-10-20) — 0x4B3400's setup, the automatic test 0x474BF0, the
    cells 0x4B1460, the walk's and the cursor's calls; S-11 placed in the attack's end on the
    target's position hex (`installation.md` §7.41; the log).
- [x] **T1c Pathfinding as the original's** (2026-10-20; BAT-Q53; the user: whole, S-11 fix-optional,
  BAT-Q54) — `pathfinder.rs` ports 0x4B3400: the walks' paths (BAT-MOV-003; batch 3c's M2 and G4) and
  the berserk turn's candidates, attack position and walk (BAT-MAG-025), S-11 as BAT-QUIRK-24 (the log).
- [x] **T1a5 The creature casts and the spell evaluation** (2026-10-20, as cut) — part C (12.9 KB,
  34 functions; shared with the hero's choice), cut in four of about 3.2 KB (the user, 2026-10-20:
  the context first):
  - [x] **T1a5a The spell context and the predictions** (2026-10-20) — the context 0x4369C0 (the
    threats each stack meets, the enemy beaten, a second context for the other side) and the
    predictions 0x422A40 (the side's run up to the other side's next stack: shots, strikes,
    retaliations, Fire Shield, breath); the computer's elementals never cast (`installation.md`
    §7.42; the log).
  - [x] **T1a5b The raise and the Faerie Dragon** (2026-10-20) — the raise 0x421000 (it must beat
    the walker's value; doubled when the side is overwhelmingly stronger) and the dragon 0x420F00
    (it casts whenever its spell is worth anything; one target, Chain Lightning, an area, friendly
    damage against it) (`installation.md` §7.43; the log).
  - [x] **T1a5c The Ogre Mage's and the Master Genie's scan, the evaluation** (2026-10-20) — the scan
    0x420D20 (a draw skips three casts in ten when the walker has a value; only targets as strong
    as the caster then), Bloodlust's value, the Genie's mean over the spells it may draw, the
    evaluation 0x43A670 with its "now" for the hero; the evaluators' table 0x43B690
    (`installation.md` §7.44; the log).
  - [x] **T1a5d The estimates** (2026-10-20) — the attack bonus's value 0x437800 (the worth's rise
    for the rounds' share of the battle's expected length); 0x437A00 is the stack record's copy
    (`installation.md` §7.45; the log).
  - [x] **T1a5e The spell evaluators** (2026-10-20; found in T1a5c; the user: in one part, before
    T1b) — the 36 evaluators 0x43B690 hands out and their nine helpers (about 11.6 KB): the
    removals value what they remove by its evaluator, so the Genie needs 34 of the 36, not its 21
    (`installation.md` §7.46; the log).
- [x] **T1a6 The hero** (2026-10-22; the user: whole) — part H in full: the cast choice 0x43C800
  with its spell dispatch 0x43BB20 and the special spells' helpers (a cast only when due now, or
  a damage spell when the side would retreat; never Berserk, Quicksand, Land Mine, Force Field,
  Fire Wall, Remove Obstacle), the retreat test 0x41E570 (the wiki's terms; the share of Fight
  Value); part A's artifact value as far as the battle needs it; slip 26 found, BAT-Q61 (the
  adventure's inputs) (`installation.md` §7.55; the log). The Catapult and the First Aid Tent are
  read (§7.29; §7.14, BAT-Q51).

**T1b follows T1a5** (the user, 2026-10-20): the neutral side's map is then complete and the
seam's questions are the neutral side's; T1a6 moves to just before T5. The evaluators (T1a5e,
found after that decision) belong to the neutral side's map too and are read (§7.46).

**Increment 2:** T1b, then the user's review.

- [x] **T1b The survey and the ADR draft** (2026-10-20; the user: no split) — the survey
  (`docs/rules/surveys/E23-battle-ai.md`: the reading, the wiki and VCMI, 18 batch rows in 4a
  and 4b, the original's slips) and the ADR draft, accepted as **ADR-0027** (`docs/adr/0027-original-battle-ai.md`:
  `h3-battle-ai` in the application layer, `choose`, a public `computer` module in `h3-battle`,
  `BAT-AI-nnn`, the budget at T2, the slips behind one fix-optional flag, BAT-QUIRK-25); accepted by the user, BAT-Q55 (the log).

**Later increments** (cut at each checkpoint from T1a's map; the neutral side first):

- **T2 The crate and the shooters**, cut in two (the user, 2026-10-20: T2a first, T2b's shape
  decided at its checkpoint):
  - [x] **T2a The crate and the plain shooters** (2026-10-20) — the `shared/` registration
    (#334); `h3-battle`'s public `computer` module; `h3_battle_ai::choose`, always a legal command
    (property tests on the test ruleset and the harvested packs); every computer shooter as the
    automatic Ballista shoots (0x41F060); stand-ins for the rest (BAT-STUB-4 to 6); BAT-AI-001,
    BAT-AI-002; about 6 µs per decision, a shot about 50 µs (the log).
  - [x] **T2b The area value and the wall shot** (2026-10-21; the user: whole) — BAT-QUIRK-25's flag
    (`shared/`, registry position 29), the area value 0x41EDA0 with slip 1 (BAT-AI-003), the wall shot
    0x41EEA0 (BAT-AI-004, the policy's first draw), the gated benchmark `computer-decisions` and the
    budget (ADR-0027 point 11; the log). **The budget** (the user, 2026-10-21): the gated count
    with its +2 %, re-set on purpose at each part of T3 (the walkers cost more), and at most 1 ms
    for the slowest decision on a quiet sandbox, measured and recorded per task; **a decision with
    a hero's cast at most 5 ms** (the user, 2026-10-22, at T5a: the hero's spell context and
    values; the side without a hero keeps 1 ms).
- **T3 Walkers**, cut in four (the user, 2026-10-21: the pack data first, then the plan and the
  attack, the scores, the odds and the ends; each part's missing pieces stand in as stubs):
  - [x] **T3-0 The spells' AI values** (2026-10-21) — each spell's `ai` in the battle pack: the
    executable's AI categories and `SPTRAITS.TXT`'s four AI values (pack format v45, battle part
    hash v7; ADR-0027 point 13; the log).
  - [x] **T3a The plan and the attack** (2026-10-21) — the walker's choice 0x421680 (BAT-AI-005:
    the plan 0x436300 with the blocked shooters, the multi-headed and breath strikes and the
    retaliation's; the melee estimate 0x4359B0, read in full; the Harpies' larger value; the draw
    of 75 to 100 %; the best) and the walk 0x41F580 (BAT-AI-006); Wait on Hard and up, else
    Defend; the hex scores 0 and "the stronger side" absent until T3b and T3c (BAT-STUB-4;
    `installation.md` §7.47; the log).
  - [x] **T3b The hex scores and the hold-back** (2026-10-21; the user: whole) — the Fire Walls,
    the moat under SoD rules, the enemies' reach with speed + 1 and the multi-headed strike, the
    crowding and breath lines with the area threat and the enemy hero's area spell (BAT-AI-007,
    §7.48; slips 3, 18, 19); the walk's hold-back now reads them (the log).
  - [x] **T3c The odds and the ends** (2026-10-21; the user: whole) — the stronger at range
    0x420A80 with the hero's value and the turrets (BAT-AI-008, slip 20; slip 6 a gap, BAT-STUB-7,
    BAT-Q56), the last retaliation's gain 0x41F3B0 with the order of action and the walker's
    side's threat records (BAT-AI-009, slip 4), the fallbacks (BAT-AI-010, slip 21), holding the walls
    (BAT-AI-011, slip 2), the besieger's walk to its wall (BAT-AI-005; §7.49; the log).
- **T4 Creature casts** — the abilities a computer stack chooses to use, per T1a's map, cut in
  three (the user, 2026-10-21: the parts without the spell context first; T4c cut again at its
  checkpoint). With T2 to T4 a side without a hero is complete: the AI lane's and the adventure
  lane's inboxes get the function (`Policy::System`, the neutral seat, A12's weights are theirs;
  done with T4c).
  - [x] **T4a The entry, the raise and the Faerie Dragon** (2026-10-21) — the creature cast entry
    0x421280 before the action and after a choice of nothing (BAT-AI-012), the Archangel's and the
    Pit Lord's raise 0x421000 against the walker's value, the Faerie Dragon 0x420F00 with its
    values (BAT-AI-013, slip 7), at the targets the engine accepts (the user; BAT-STUB-8, BAT-Q32);
    §7.50; the log.
  - **T4b The spell context and the Ogre Mage**, cut in two (the user, 2026-10-21: the Ogre Mage
    first; the context's threat tables and its second context, which only the Genie's evaluators
    read on the neutral side, move to T4c; the Ogre Mage is told from the elementals by its spell):
    - [x] **T4b1 The Ogre Mage** (2026-10-21) — the scan 0x420D20 (its draw, the strength floor),
      Bloodlust's value 0x43C330 with its mass version (0x43A910 with the evaluator 0x438100), the
      attack bonus's value 0x437800 and the context's threat records (BAT-AI-014, slip 8; "the
      enemy beaten" and the predictions' draws BAT-STUB-9); §7.51; the log.
    - [x] **T4b2 The predictions** (2026-10-22; the user: whole) — 0x422A40 with the side's run
      0x422880, the predicted shooters' and walkers' turns (the prediction flag through
      `h3-battle`'s estimates and the walker), the melee prediction 0x4227A0 and the predicted
      strike 0x4224E0 with their draws, "the enemy beaten" (BAT-AI-015, BAT-STUB-9 retired); a
      hypnotized walker leaves itself out (the user: BAT-STUB-10, BAT-Q58); §7.52; the log.
  - [x] **T4c The Master Genie** (2026-10-22; the user: whole, no cut; with slip 22, BAT-Q59) —
    the Genie's mean 0x43C4A0 over the evaluators, the context's tables 0x43C040 with the seed
    0x43BF20 (slip 23) and the second context, the removals 0x439A80 (BAT-AI-016; slips 9 to 17,
    8 widened; BAT-STUB-4 retired); the fallbacks after a built context on its last search (slip
    22, BAT-STUB-11); §7.53; the inboxes have the function; the log.
  - [x] **T4d A removal's copy** (2026-10-22; the user: whole, BAT-Q60) — slip 24 under
    BAT-QUIRK-25: the removal's lone copy (0x437A00) beside the battle's records, which Disrupting
    Ray and Slow never meet, Berserk and Hypnotize strike as the stack itself, a two-hex shooter's
    copy blocked by its own record; the copy's turns at its own speed and Hypnotize's shooter
    marks in both profiles; BAT-STUB-12 retired; §7.54; the log.
- [x] **Slip 25** (2026-10-22; the user, BAT-Q57) — in `strict-fixed` the computer's elementals cast
  their protection through the Ogre Mage's scan with the protections' estimates (§7.42, §7.46),
  the shooters among them before their shot, against its value (the user, at the task's start;
  BAT-AI-002, BAT-AI-012, BAT-AI-014; `POLICY_VERSION` 11; the log).
- **T5 The hero's spells** (T1a6's reading, §7.55), cut in two (the user, 2026-10-22; BAT-Q61
  option (c): the retreat test's answer is no until T6, BAT-STUB-13; slip 26 decided at T6):
  - [x] **T5a The entry, the choice, the evaluators, the damage spells and the summons**
    (2026-10-22) — 0x422DA0 before each stack's action, the choice 0x43C800 (× 2.5 or × √casts,
    the 75–100 % draw, "now"), 0x43BB20 but T5b's cases: the evaluators through 0x43A670 and
    0x43A910 with "now" (0x436C60) and Magic Mirror's scaling, the bolts', Dispel's and Clone's
    evaluators, Chain Lightning, the area and mass damage spells, the summons (BAT-AI-017,
    BAT-AI-018; §7.56; slip 27, Expert Dispel, found by the blind check; `POLICY_VERSION` 12;
    the hero budget 5 ms; the log).
  - **T5b The raises, Sacrifice, Earthquake and Teleport**, cut in two (the user, 2026-10-22:
    Teleport on its own):
    - [x] **T5b1 The raises, Sacrifice and Earthquake** (2026-10-22) — 0x43ACA0, 0x43B1E0 with
      0x43AF50, 0x43B8F0 (BAT-AI-019; §7.57; slips 28 and 29 found and built under BAT-QUIRK-25;
      Sacrifice's victim the cast's second target; `POLICY_VERSION` 13; the log).
    - **T5b2 Teleport**, cut in two (the user, 2026-10-22, at its start: the cast entry of a
      caster that is not the active stack on its own):
      - [x] **T5b2a Teleport's value** (2026-10-22) — 0x43AA60 with the destination map 0x4B2FF0
        and the walker's flag-1 turn, a disabled stack's turns, "now" (BAT-AI-020; §7.58; slips 30
        and 31 found and built under BAT-QUIRK-25, strict-fixed without stops on batch 3g TV and
        without a run-up; slip 5 changes no choice; `POLICY_VERSION` 14; the slowest decision 3.5 to 4.3 ms; the log).
      - [x] **T5b2b A caster's cast entry in Teleport's turns** (2026-10-23; the user: whole,
        `cross-lane`) — 0x421280 and its callees read only their caster's record (§7.59): every
        stack's first turn asks its entry at the engine's targets for it
        (`computer::creature_cast_targets_of`), a Faerie Dragon's spell its last activation's
        (`h3-battle` keeps each stack's, battle state hash v52, the golden and the adventure
        lane's replays rehashed); slip 30 widened to a cast entry's spell context; BAT-STUB-14
        retired; `POLICY_VERSION` 15; the log.
- [ ] **T6 The hero's other choices** — retreat and surrender, the tactics phase, the Catapult's
  aim, the First Aid Tent's patient (BAT-Q51); BAT-Q33's rule.
- [ ] **T7 The closure** — the batch rows' leftovers, the benchmarks, the docs.

## Exit criteria

1. The function returns a legal command for every state the engine reaches, for both sides and
   every difficulty (a property over random battles, the special battlefields and sieges
   included), and the same command for the same state and stream.
2. Every branch of T1a's map is modelled, or recorded as a gap with its reason.
3. Every reading at risk has a played batch row that agrees, or the difference is resolved
   (the reading re-read, a quirk, or an open question).
4. The cost per decision is within T1b's budget; a batch benchmark with `System` seats is recorded
   (with the AI lane).
5. The AI lane and the adventure lane have the function in their inboxes.
6. `cargo xtask verify` passes on Linux and Windows, including `--with-original-data`; the new crate
   is a core crate under the coverage gate.

## Risks

- **Size:** the original's AI is large, the hero's spell choice especially; T1a's map may show more
  than the outline's increments. Cut at each checkpoint, the neutral side first.
- **Reading first:** with spot checks only, a misread branch that no batch row covers stays
  wrong. T1b names the readings at risk; a second look at the deciding paths where a branch is
  surprising (Q57's pattern).
- **Floating point:** the computer's numbers are x87 doubles; every new estimate goes through
  `kernel::num` as `aim.rs` does (ADR-0003).
- **Cost:** the threat records run a pathfinder per stack for each decision; a neutral seat's
  decisions are inside the batch's step (E05's throughput).
- **The surface between the crates:** the rules and the policy share the computer's numbers; a
  change to one must not drift from the other (one implementation, in `h3-battle`).
