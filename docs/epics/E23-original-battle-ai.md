# E23 — The original's battle AI

Status: **in progress**, refined 2026-10-19 (BAT-Q52); T1a1, T1a2, T1a3a and T1a3b done (the cuts
confirmed 2026-10-20), T1a3c next · Follows E05 (the
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
  left; the hex scores first):
  - [x] **T1a3a The hex scores** (2026-10-20) — the enemies' reach 0x420260 with its
    multi-headed branch, the crowding and breath scores 0x41FB60 with the area threat 0x41F920
    (`installation.md` §7.36; the log).
  - [x] **T1a3b The strike hexes and the order's sort** (2026-10-20) — 0x436300 with the
    shooters blocked, the multi-headed and breath strikes (the jousters' longer run-up at a tie);
    part O, 0x423340, the library sort (ties of the two sides unordered above 16 stacks)
    (`installation.md` §7.37; the log).
  - [ ] **T1a3c The fallbacks and the ends** (about 4 KB) — 0x420760, 0x4208F0, 0x41F3B0,
    0x4B2FF0, the walk 0x41F580's end, the odds' totals (0x420A80, which reaches the adventure
    AI's valuations, part A).
- [ ] **T1a4 The AI pathfinder** — part P (4.6 KB): the walker's path among equally short ones
  against BAT-MOV-003 (S-11, Q53).
- [ ] **T1a5 The creature casts and the spell evaluation** — part C (12.9 KB): the cast entry's
  kinds, the spell context 0x4369C0, the estimates, the predictions; shared with the hero's choice.
- [ ] **T1a6 The hero** (read just before T5) — part H (5.8 KB): the cast choice 0x43C800's candidates, the retreat test
  0x41E570; part A (6.4 KB) only as far as the battle needs it (the adventure lane's to read with
  its AI). The Catapult and the First Aid Tent are read (§7.29; §7.14, BAT-Q51).

**T1b follows T1a5** (the user, 2026-10-20): the neutral side's map is then complete and the
seam's questions are the neutral side's; T1a6 moves to just before T5.

**Increment 2:** T1b, then the user's review.

- [ ] **T1b The survey and the ADR draft** — the survey `docs/rules/surveys/E23-battle-ai.md`
  (the wiki's and VCMI's descriptions against the reading; VCMI's `BattleAI` is its own design);
  the readings at risk, each with a planned batch row; the ADR draft
  (`docs/lanes/battle/adr-drafts/`) for the seam: the crate and its name, the surface `h3-battle`
  exposes for the computer's numbers, the function's signature and stream, how a walker's path
  among equally short ones is chosen (a `Command` carries a destination; 0x4B3400 against
  BAT-MOV-003), the cost budget per decision, the rule ids for the computer's choices (as
  BAT-WAR-008 and BAT-SIE-020 to BAT-SIE-023 for the automatic machines).

**Later increments** (cut at each checkpoint from T1a's map; the neutral side first):

- [ ] **T2 The crate and the shooters** — the `shared/` registration; the function, always a legal
  command; every computer shooter as the automatic Ballista already shoots (0x41F060), with the
  area estimate and the wall shot.
- [ ] **T3 Walkers** — melee targets and moves (0x4221F0's walker branch, the melee estimate, the
  AI pathfinder), waits and defends.
- [ ] **T4 Creature casts** — the abilities a computer stack chooses to use, per T1a's map. With
  T2 to T4 a side without a hero is complete: the AI lane's and the adventure lane's inboxes get
  the function (`Policy::System`, the neutral seat, A12's weights are theirs).
- [ ] **T5 The hero's spells** (T1a6's reading first).
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
