# Review r5b-mana-defences: BAT-MAG-054..060, BAT-DMG-010, BAT-MOR-003/004 (E04b part), BAT-QUIRK-15..18

Nothing in the repository was changed, no cargo was run. Every claim about the original below is
from my own read of `Heroes3.exe` with `scripts/exe.py` (addresses given), described in my own
words. Functions read whole: 0x4E5240, 0x44AB80, 0x4D92D0, 0x465080, 0x464F50, 0x464B40, 0x464D40,
0x464920, 0x469440, 0x4687C0, 0x444120, 0x443130, 0x422440, 0x443F40, 0x443B90, 0x4487F0, 0x4CE940,
0x4CE970. Read in the deciding parts: 0x59FE30 (0x59FE30–0x5A0060 cost and payment, 0x5A02CB–0x5A0318
and 0x5A1F89–0x5A2058 the resisted path, 0x5A199C–0x5A1A4E Sacrifice, 0x5A1A53–0x5A1AFC Disrupting
Ray, 0x5A1E70 Acid Breath, 0x5A2058–0x5A2228 tail and channel), 0x465330 (to the Wraith's case),
0x446E30 (head), 0x5A7560, 0x43EA70 (head), 0x444510 (head, switch, case 75), 0x59EC50
(0x59EE7A–0x59EF56), 0x478900, 0x478D80 (0x47927A–0x4792D7, 0x479AFD–0x479B24, 0x479C63–0x479C8E),
0x475ED0 (head, 0x4760AB), 0x462A20 (its end, 0x463238–0x463246), 0x474559–0x474562,
0x47525F–0x475268, 0x4639F0 (head), 0x4782D0 (0x478755–0x478790), 0x447120 (its end); callers of
0x469440, 0x464920, 0x465080, 0x444120, 0x422440, 0x4487F0; the creature table's flags (0x6703B8 +
116·id + 16) of creatures 20, 21, 34, 35, 43, 53, 61, 120, 131, 132, 134, 145–149.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-054 | every clause as stated (`mana::cost_change`, `mana::cost`, `magic::check_spell`) | 0x4E5240 (57 → 0; enemy army null → no change; +2 for 20/21 in the given army; −2 for 34/35 in hero +0x91; floor 1 at 0x4E531C), 0x44AB80 (7 type slots only), 0x59FFCF–0x59FFEB (the army of the side opposite +0x132C0; cost kept for the channel; paid through 0x4D92D0). VCMI `CBattleInfoCallback.cpp` L2111–2136 is as cited (living units, floor 0). Batch 2a R6: 20→15, 10→2, 16→11, 11→6 with the Pegasus dead: as cited | `tests/mana.rs` pins +2, −2, both, once each, the largest reduction, the floor, the refusal, the dead Pegasus. `harvested_mana.rs` replays R6's costs (5, 8, 5, 5) in another cast order (Blue first, so Red goes 20→21→16→11) | ok. Reading-only clauses (not in R6): the floor of 1, dead Mages, created stacks |
| BAT-MAG-055 | as stated (`mana::channel`, `magic::apply_cast`) | 0x5A2058–0x5A2228: caster kind 0 only (kinds 1, 2 skip at 0x5A217B), other side's hero must exist, cost ≥ 5, signed div 5, sum of counts of creature 43 whose controller ≠ acting side, one 16-bit add, no cap. The resisted path 0x5A1F89 falls into the tail, so a resisted cast channels too (the engine does). VCMI `BattleSpellMechanics.cpp` L368–382 agrees on share, largest, once; it computes the gain before the effect (see F-6). R6 row 2: 15→16 after a cost of 8: as cited | share, threshold, largest, after the effect, killed Familiars, own Familiars, no hero, hypnotized, obstacle spell | ok. Reading-only: no cap (R6 never shows mana above the maximum; the harvested test's 20→21 comes from the reading), killed Familiars |
| BAT-MAG-056 | as stated (`mana::drain`, `turn::activation`, `state.wait_phase`) | 0x465330: case 61 at 0x465548 returns while +0x13DE4 is set, hero of the side opposite the controller (0x53D0 − 4·controller), signed word +0x18 > 0, −2, floor 0; no count involved. 0x4652A7 sets the flag, 0x4760AB clears it. VCMI `BattleFlowProcessor.cpp` L950–963, `necropolis.json` L177–181: as cited | normal phase, wait phase, floor, no hero, hypnotized, morale turn, blind, bad morale, tactics, berserk | ok |
| BAT-MAG-057 | as stated (`magic::mirror_chance`, `magic::mirror`) | 0x4487F0 (value while duration +0x228 > 0, creature 134: the larger of it and spell 36's value at no mastery), 0x59EE7A–0x59EEE2 (real side ≠ acting side, not dead, not Dispel, random(1,100) ≤ chance). VCMI `neutral.json` L214–218: 20 | chance, max rule, Hypnotize, Dispel, Genie, 300 casts, BAT-QUIRK-9 | ok |
| BAT-MAG-058 | as stated (`magic::reflection`, `combat::strike`) | 0x443130 (stored f32 while duration +0x20C ≠ 0, else 0.2f for creature 53), 0x422440 (duration or creature 53; fire-immune striker 0x4000 → 0; min(stage 1, hit points); the target's controlling hero as caster of spell 29), 0x443F8D–0x443FC0 (a clone's base is 1). VCMI `inferno.json` L336–340: 20 | own shield, the spell's larger value, Q-0010, shots, fire-immune, hero bonus | ok |
| BAT-MAG-059 | count, draws, cast, clone, acted mark, order before the end check: as stated. Not as the original: the health under Age (F-1), the permanent defense losses (F-2) | 0x469440 whole (both sides; creature 131, dead bit, casts +0xDC > 0, not 0x800000; cast lowered before the draws; +0x60 div 5 plus `rand() mod 5 = 0` per leftover; acted bit only when n > 0; 0x5A7560 with n × +0xC0, permanent). Its 13 callers as the note lists them. 0x47927A–0x4792BC (acted bit, end test, good-morale roll in that order). No VCMI or wiki source in the entry (F-6) | a fifth, draws, retaliation + morale turn, spell, shot, mine, clone, sacrificed, effects, active stack. No test with Age | F-1, F-2, F-6 |
| BAT-MAG-060 | as stated (`morale::frightened`, `turn::activate`, `turn::morale_turn`) | 0x464D40 whole (flag 0x20000 or creature 132 spared; controller from +0xF4 flipped by +0x288; counts of creature 132 in the record array of the opposite side; `rand() mod 10 = 0`; acted bit). 0x465080: asked after bad morale, only with argument 1, outside tactics and the wait phase. 0x463238 (argument 0 at the battle's start), 0x474562, 0x475268, 0x479C73, 0x479C87 (argument 1). No VCMI or wiki source in the entry (F-6) | first activation, immunity, largest chance, wait phase, hypnotized Azure, morale turn, berserk, Tactics | ok in code; F-5, F-6 |
| BAT-DMG-010 | as stated (`combat::strike_inputs`, `damage::Strike::staged`) | 0x443B90 whole: defense factor, then ×0.5 (f64) for striker 120 against stack flag 0x400 and for striker 121 against creature 121 or 83, then the ranged and melee penalties. War machines carry 0x400 in the table (and `mind_immune` in the pack). VCMI `conflux.json`: no halves, as cited | both profiles, the stage, the largest of two, retaliations | ok (wiki check wanted) |
| BAT-MOR-003 | as stated (`morale::good_morale`, `turn::end_turn`, `turn::morale_turn`) | 0x464920 whole: not in tactics, not with 0x1000000, count > 0, morale clamped to ±3, random(1,24) ≤ morale; clears 0x4000000, sets 0x1000000; no human/computer branch. Callers: after a walk, an attack, a shot, a creature's cast | chance, once per round, wait phase, blinded by its own action, Fear | ok |
| BAT-MOR-004 | the E04b changes (first activation, wait-phase gate) as stated (`morale::bad_morale`, `turn::activate_first`) | 0x465080 whole: checks only with argument 1, outside tactics, outside +0x13DE4. 0x464B40 whole (the map has it `inferred`, "not read in detail"): see F-3 | first activation, T11a's `a_stack_that_sees_again_in_the_wait_phase_does_not_freeze` (fails for the old variant: 254/255 freeze) | F-3; F-4, F-5 touch it |
| BAT-QUIRK-15 | rule in every profile, no flag (user, BAT-Q15) | confirmed twice: batch 2a R6 round 3 and my re-read of 0x4E5240, 0x44AB80 | `a_dead_pegasus_still_raises_the_cost` fails for VCMI's variant | ok. §4's guide would give `fix-optional` with a flag for an unknown bug; `preserve` without a flag is the user's decision |
| BAT-QUIRK-16 | always fixed | re-read: the bound +0x54BC of the side opposite the caster is loaded once (0x5A21B2) and used for both sides' record arrays (0x5A21BF–0x5A2215). Records of the opposite side are always walked in full, so only hypnotized Familiars and stale records differ | `a_hypnotized_familiar_channels_for_its_controller_wherever_its_record` fails for the original's loop | ok |
| BAT-QUIRK-17 | always fixed (`raised.corpse == NO_CORPSE`) | re-read: 0x5A199C–0x5A1A4E kills the victim, sets 0x10000000 and raises the target with no call of 0x469440; 0x469440 tests neither the sacrificed bit nor when the stack died; 0x5A7560 and 0x4687C0 place the stack on its hex without looking at what stands there | both tests fail for the original's variant | ok |
| BAT-QUIRK-18 | rule in every profile, no flag (user, BAT-Q18) | re-read of 0x464D40: the frightened stack's side is its controller's, the dragons are counted in the array of the opposite side whoever controls them. This is the second, independent look §4 asks for; VCMI is silent on it | `a_hypnotized_azure_frightens_the_side_that_controls_it` fails for the controller variant | ok |

## Findings

### F-r5b-mana-defences-1: A Phoenix that dies under Age is reborn with the halved health: half as many come back
- rules: BAT-MAG-059 (with BAT-MAG-044)
- kind: behaviour-bug
- severity: medium (Ghost Dragons against Phoenixes: Age lands on 20 % of their strikes and lasts 3 rounds)
- confidence: high for both sides by reading; the sketch below would settle it (`needs-run`)
- evidence:
  - Engine: `raise.rs:313` reads `health = stack.max_hp` first; `raise.rs:325` then calls
    `effects::remove_dead`, whose `recompute` → `vitality::materialize(.., aged = false)`
    (`effects.rs:794`, `vitality.rs:79–96`) gives the dead stack its full `max_hp` back;
    `raise.rs:326` raises `creatures × health` with the stale, halved `health`. Until
    `remove_dead` a dead stack keeps its effects (`effects::damaged` ends only those that damage
    ends), so Age is still on it when `health` is read.
  - Original: the death routine 0x444120 ends every spell with a duration (loop 0x444228–0x444244);
    0x444510 clears the duration before its switch, and its case for spell 75 (0x444740) sets the
    health +0xC0 to `trunc(f32(start health) × poison factor + 0.95)` without the half. The rebirth
    0x469440 runs later in the same action and raises `n × +0xC0` (0x46959E–0x4695AD); 0x5A7560
    turns that into `n` creatures.
  - Scenario (harvested packs): 10 Phoenixes (200 hit points, 100 while aged) die aged. Original:
    10 div 5 = 2 Phoenixes at 200. Engine: 2 × 100 = 200 hit points against a health of 200: 1
    Phoenix. With 5 Phoenixes: the original 1 at 200, the engine 1 with 100 of 200 hit points.
  - It is order-dependent too: if two aged Phoenix stacks die in one action, the first is reborn
    at half, and the second, read after the first's `remove_dead`, in full.
  - Poison is not affected (its tenths stay in both; the health read is the same before and after).
  - The T7c3 review added `remove_dead` before the raise ("a reborn stack kept the effects it died
    with"); the health read stayed in front of it. No test has Age on a reborn stack.
- fix or test sketch: move the read of `max_hp` behind `remove_dead` (or call `remove_dead`
  before the stack's values are read). Test for `tests/rebirth.rs` (`needs-run`; `t:wither` is the
  test pack's Age, the test Phoenix has 10 hit points):
  ```rust
  // covers: BAT-MAG-059, BAT-MAG-044
  #[test]
  fn an_aged_phoenix_comes_back_whole() {
      let r = ruleset();
      let mut s = battle(&r, &[(0, "fixed", 1)], &[(0, "phoenix", 10)], 1);
      let id = spell(&r, "wither");
      let cast = effects::TimedCast { id, spell: r.spell(id).unwrap(), mastery: 0, rounds: 3,
          own: crate::magic::SpellBonus::default() };
      effects::apply(&mut s, &r, cast, StackId(1), &mut Vec::<Event>::new());
      assert_eq!(s.stacks()[1].max_hp, 5);
      kill(&mut s, &r, 1);
      let mut events = Vec::new();
      rebirths(&mut s, &r, &mut events);
      let phoenix = s.stacks()[1];
      assert_eq!((phoenix.count, phoenix.max_hp, phoenix.top_hp), (2, 10, 10)); // today (1, 10, 10)
      assert_eq!(reborn(&events), vec![(1, 2)]);
  }
  ```

### F-r5b-mana-defences-2: A dead stack loses Disrupting Ray's and Acid Breath's defense loss; the original never gives it back, so a reborn or raised stack differs
- rules: BAT-MAG-059 ("without the effects it died with"), BAT-MAG-010 ("a stack that dies loses its effects"), BAT-MAG-035, BAT-MAG-046 (the code is `effects.rs`, outside my slice; I met it through the rebirth)
- kind: behaviour-bug (and rule-text for the three summaries)
- severity: medium (needs a stack hit by Disrupting Ray or by Rust Dragons that dies and comes back: rebirth, Resurrection, Animate Dead, an Archangel)
- confidence: medium to high. Raise it with an experiment: Disrupting Ray on a stack, kill it, resurrect it, read its defense
- evidence:
  - Engine: both spells are `duration = "permanent"`, cumulative `defense_loss` entries
    (`packs/base/pack.toml`: `disrupting_ray`, `acid_breath`). `effects::remove_dead`
    (`effects.rs:927–945`) drops every entry of a dead stack, permanent ones too, and `recompute`
    then returns the defense. It runs at the end of every command (`command.rs:478`) and inside
    the rebirth (`raise.rs:325`).
  - Original: Disrupting Ray lowers the defense field +0xCC directly (0x5A1A89–0x5A1ACB) and Acid
    Breath too (0x5A1E70, no entry at all). 0x444510 returns at once for spell 47 (0x44452D), so
    the death routine neither restores the defense nor even clears the ray's duration. A search
    for every instruction that writes at offset 0xCC finds, in the battle's stack routines, the
    setup (0x43D81E), the apply and end routines of other spells (0x444BDC, 0x444C30, 0x444F13,
    0x4446E7 Prayer, 0x4447BB Stone Skin and Disease), Defend (0x4798D7, 0x446E87) and the two
    handlers above; none in the death 0x444120, the raise 0x5A7560 or 0x43EA70 (the raise's other
    callees, 0x443080, 0x4687C0, 0x5A7320, 0x446720, are not among the writers either).
  - Scenario: Rust Dragons strike 10 Phoenixes (defense 18) twice (12 left), then kill them; two
    are reborn. Original: defense 12. Engine: 18. The same after three Expert Disrupting Rays, a
    death and a Resurrection: −15 in the original, nothing in the engine.
  - No test pins either behaviour (`a_stack_that_dies_loses_its_effects` uses Stone Skin,
    `a_dead_stack_comes_back_on_its_hexes_without_effects_and_keeps_its_round` uses Haste).
- fix or test sketch: let `remove_dead` keep permanent entries (those with `rounds == None`), or
  keep their deltas beside the stack; say so in BAT-MAG-010, 035, 046, 059. Test: as
  `tests/raise.rs`'s dead-stack test with the test pack's `ray` at mastery 3 instead of Haste:
  after the kill and `rise`, `defense_now()` must still be 5 below `defense` (`needs-run`).

### F-r5b-mana-defences-3: The original's bad-morale check lets a side no human commands escape one freeze in four; BAT-MOR-004 rests on a callee nobody had read
- rules: BAT-MOR-004
- kind: question (and source)
- severity: medium (every battle of the original against the computer or neutral stacks with negative morale; the engine has no notion of who commands a side)
- confidence: high for the reading (I read the whole function); what the flag means rests on its setter and H3API's names
- evidence:
  - 0x464B40: morale +0x4E8 clamped to ±3; `random(1, 12)`; no freeze if the draw is above
    −morale (so 1, 2, 3 in 12: the catalog's 2, 4, 6 in 24). Then 0x464BDB reads the manager's byte
    +0x54A4[side] for the stack's real side (the pick pushes +0xF4, 0x465279); if it is 0, a second
    draw `random(1, 4)` follows and a 1 returns without freezing (0x464BE6–0x464BF8). Only after
    that the acted bit is set.
  - 0x4639F0 sets that byte per side from the side's player through 0x4CE940 (the player
    record's byte +0xE2; H3API: `IsHuman`, the manager field `isNotAI`) at 0x463AA8, and to 0 for a
    side without a player (0x463AED). So a computer player's and a neutral side's stacks freeze
    with 3/4 of the chance: 6.25 %, 12.5 %, 18.75 %.
  - Good morale (0x464920) and Fear (0x464D40) have no such branch. For a side a human commands
    the engine's rule is the original's.
  - The engine (`morale.rs:357–365`) rolls once for every side. The catalog's code-reading source
    names 0x464B40 with confidence high, but `docs/exe/map.toml` has it `inferred`, "Not read in
    detail": the failure `docs/exe/README.md` warns of ("an unread callee assumed"). The wiki and
    VCMI (`BattleFlowProcessor.cpp` L378–398) know no such difference, so by `docs/04` §2 this is a
    conflict of sources for the user.
  - On the way: `structs.toml` has +0x54A4 as `local_human`, `i32[2]`; it is two bytes, one per
    side, "a human commands it"; the local-human bytes are +0x54A6 (set from 0x4CE970, which wants
    the record's +0xE2 and +0xE1). `installation.md` line 1323 calls +0x54A4 "a local human".
- fix or test sketch: OPEN-QUESTIONS entry: does the seam say which side a computer commands (or
  a Tier-2 parameter per side), or does the engine keep the human rule for both sides, said in
  BAT-MOR-004? Mark 0x464B40 `read` in the map. An experiment needs a few hundred activations of
  neutral stacks at morale −3 (25 % against 18.75 %).

### F-r5b-mana-defences-4: In the wait phase the engine still orders stacks by their own Wait flag; the original orders every pending stack by the phase (T11a fixed only the bad-morale gate)
- rules: BAT-TRN-002, BAT-MAG-037, BAT-MOR-003 (outside my slice; found comparing the pick 0x465080 with `turn.rs` for T11a's change)
- kind: behaviour-bug
- severity: medium, perhaps high: casting Summon Elemental or Clone in the wait phase is normal play
- confidence: medium. The reading is clear; no observation covers it (batch 2d SU step 4 has one waited stack and the new one only). An experiment: two stacks of speeds 3 and 4 wait; on the slower one's wait-phase turn summon Air Elementals (speed 7); who acts next?
- evidence:
  - Original: when no stack is left for the normal phase the pick sets +0x13DE4 and clears every
    record's waited bit (0x4652A7–0x4652E5); from then on every stack that is not done is a
    candidate alike. The comparison 0x464F50 puts a stack with the had-morale bit first in every
    phase (0x464F60–0x464F82) and, while +0x13DE4 is set, the slower stack first
    (0x464FDB–0x465007). The Wait button (control 0x7D9, whose handler queues action 8,
    0x4751B8) is disabled while +0x13DE4 is set (0x478755–0x478790), for every stack; whether
    the dispatcher would also refuse a Wait there I did not read.
  - Engine: `turn::next_active` (`turn.rs:44–60`) gives a stack without `WAITED` phase 0: before
    every waited stack, fastest first. `command.rs:220` and `legal.rs:84` refuse Wait only to a
    stack with its own `WAITED` flag.
  - Scenario: `slow` (speed 2) and `fixed` (5) of one side wait. In the wait phase `slow` acts
    first; its hero summons `sprites` (7) and `slow` defends. Original: `fixed` (the slower of the
    two pending), then the sprites. Engine: the sprites (never waited), then `fixed`. The same
    for a stack whose blindness ended in the wait phase (T11a's case) when it is faster than a
    pending waited stack.
  - Also: such a stack may Wait in the engine and then acts once more in the same wait phase; the
    original offers no Wait there. And in the normal phase the original puts a pending had-morale
    stack (one that got good morale blind and sees again) before faster stacks; the engine sorts
    it by speed.
- fix or test sketch: order phase by `state.wait_phase` (once it is set every pending stack is in
  the wait order, had-morale stacks first) and refuse Wait while it is set; a test with the
  scenario above.

### F-r5b-mana-defences-5: After a bad-morale or Fear freeze the engine counts the frozen stack's side as the side that acted last; the original records a side only at an activation
- rules: BAT-TRN-001's tie-break as BAT-MOR-004 and BAT-MAG-060 use it (`turn.rs:233–245`; the Fear branch is E04b's)
- kind: question (the reading against VCMI; the wiki's "did not have the latest chance to act" fits both)
- severity: medium (an edge: a freeze followed by a speed tie across the sides)
- confidence: medium
- evidence:
  - Original: the tie-break of 0x464F50 prefers the side that is not +0x132B8 (0x46502A–0x46504C).
    After the battle's setup +0x132B8 is written by the activation (0x465362), by the end of the
    tactics phase (0x474539, 0x475233, 0x475F6D), by the pick's Catapult case (0x4650BB, sieges)
    and, saved and restored, inside the melee routine (0x441D54/0x441D87, 0x441F13/0x441F44);
    nowhere else. A stack that bad morale or Fear freezes goes back to the pick's loop
    (0x465283–0x46529C) without an activation, as a blind stack does.
  - Engine: `turn.rs:245` sets `last_side_acted` after a freeze (not after a blind stack's pass).
    VCMI does the same (`BattleFlowProcessor.cpp` L844–850 makes the frozen stack the active one).
  - Scenario: B0 (speed 9) acts; A1 (6, morale −1) is picked and freezes; A2 and B1 (both 5) are
    left. Original: B acted last, A2 goes. Engine: A "acted" last, B1 goes.
- fix or test sketch: decide with BAT-TRN-001's owner; if the reading stands, skip line 245 for
  the two freezes.

### F-r5b-mana-defences-6: BAT-MAG-059 and BAT-MAG-060 cite the code reading alone; VCMI covers both and differs in five points the entries do not name
- rules: BAT-MAG-059, BAT-MAG-060 (and one wording in BAT-MAG-055's VCMI source)
- kind: source
- severity: low
- confidence: high
- evidence: `docs/04` §2 ("Consensus before trust") wants the wiki and VCMI compared and cited
  before a rule rests on a reading. Both entries have one source. VCMI 1.7.5:
  - Rebirth, `lib/CStack.cpp` L188–222 and `config/creatures/conflux.json` (REBIRTH 20, CASTS 1):
    `floor(base × 20 %)` plus a 20 % draw for each creature left over: agrees with 0x469440. It
    differs in: the cast is used only when a creature comes back (L214–216; the original lowers
    it before the draws, 0x4694B4), the reborn stack loses its retaliations (L221), and the stack
    comes back at the moment of its death, not at the action's end.
  - Fear, `config/creatures/neutral.json` L95–108 and `BattleFlowProcessor.cpp` L358–373,
    L965–975: 10 %, rolled after bad morale, the turn lost: agrees. It differs in: only LIVING
    stacks are frightened, so not the Stone and Obsidian Gargoyles, which have neither flag 0x10
    nor 0x20000 in the original's table and can be frightened by 0x464D4C–0x464D57 (the engine
    follows the original); and it rolls at every activation, the wait phase included.
  - BAT-MAG-055's VCMI source says "after a hero's cast": VCMI computes the gain before the
    effect (`BattleSpellMechanics.cpp` L373–382), so Familiars the cast kills channel there; the
    clause "a stack the cast killed channels nothing" rests on the reading alone.
- fix or test sketch: add the two VCMI sources with these differences; the Gargoyles and the
  wait phase are the points a wiki check or an experiment should settle (below).

## Wiki checks wanted

1. Morale: any word that computer players' or neutral armies freeze less often (F-3).
2. Azure Dragon: who is immune to Fear (non-living creatures, or creatures without morale: the
   Gargoyles decide), its chance, and whether a stack that waited can still be frightened.
3. Phoenix: the rebirth's numbers (a fifth, the chance per creature left over), once per battle,
   and whether a failed rebirth uses it up.
4. Psychic Elemental and Magic Elemental: the half damage (BAT-DMG-010 has no agreeing source
   beside the reading; VCMI lacks it).
5. Disrupting Ray and Rust Dragon: does the defense loss survive the stack's death and
   resurrection (F-2)?
6. Combat: the order in the wait phase of a stack that did not wait (summoned or cloned in it), and
   whether it can wait (F-4); whether a frozen stack counts as its side's chance to act (F-5).
7. Familiar, Mage, Pegasus, Wraith: whether mana can go above the maximum through the channel.

## Not reviewed, left open

- The spell book's own cost test (one of 0x4E5240's 25 callers, not read by anyone): that the
  book refuses with the same cost the cast pays is assumed; R6 shows no refusal.
- Sieges (E04c): which army +0x54C4 names for a town's defender, the moat's rebirth check, arrow
  towers.
- The computer player's paths (its casts are never mirrored; its evaluations ask 0x4487F0 and
  0x443130).
- 0x447A80's usefulness cases for a stack with an innate shield or mirror: checked through the
  tests only.
- Met on the way, not followed: in the dispatcher's own strike case (0x47927A; which action it
  serves I did not check, the move-and-attack routine 0x478900 has the other order) the original
  rolls good morale (0x4792BC) before the stack re-enters its hexes (0x4792D7), while the engine
  re-enters first after every action (`command.rs:420–431`: a stack the fire kills there gets no
  roll); the original calls no turn start (0x446E30) in the wait phase, so a
  stack first picked there (created in it) gets none, while the engine regenerates it and counts
  its own-turn effects (`turn.rs:219–225`); a stack created in the original's wait phase and
  blinded before its turn seems to make the pick 0x465080 loop for ever (+0x4F0 is never set
  there), a soft-lock the engine does not have.
- A creature with `rebirth` that a mine or Fire Wall kills on its way walks on and attacks after
  its rebirth (`obstacles.rs:785`); 0x478900 tests only the dead bit after the move, so the
  original probably does the same. Needs the Orb of Vulnerability for a Phoenix; not followed.
