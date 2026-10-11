# Review r4a-life-created: BAT-MAG-035 to 039, BAT-QUIRK-5/6/7/8/11, BAT-END-001/003 and BAT-FLE-002 as E04b changed them

Read: the rules (`rules.py`), `installation.md` §7.18, §7.19, §7.21 ("The raisers and the end of the
battle"), the ledger entries, `experiments/results/E04b-batch-2d.md` (whole), the T6a/T6b/T6c log
records; `raise.rs`, `created.rs`, `flee.rs`, `turn.rs`, `vitality.rs` whole; the parts of `magic.rs`,
`state.rs`, `combat.rs`, `damage.rs`, `command.rs`, `effects.rs`, `after_strike.rs`, `obstacles.rs`
that the rules reach; `tests/raise.rs`, `tests/created.rs` whole, `tests/properties.rs` in parts.
No cargo run: every "today:" in a sketch is what I traced by reading and is marked `needs-run`.

Executable, re-read with `scripts/exe.py` (whole unless a range is given): 0x5A7560 raise;
0x59FE30 cast routine at 0x59FE30–0x59FF52, 0x5A0048–0x5A0354 and the handlers 0x5A1907, 0x5A199C;
0x5A3950; 0x5A3CC0; 0x5A3F50; 0x5A39C0 (the target-stack part); 0x5A8090 (whole, all six cases);
0x44A4D0 (first switch, fire immunity, tables); 0x59EC50 at 0x59F196–0x59F300 (Sacrifice) and
0x59F3E0–0x59F620 (Clone, Summon); 0x59F700; 0x5A2D00 and 0x5A2E90 (Sacrifice's two cursors, not in
the map yet); 0x444090; 0x444120; 0x447120; 0x475ED0; 0x465080; 0x478D80 at 0x479B80–0x479CC0;
0x4740D0 at 0x47435B–0x474419 and 0x474500–0x474640; 0x47A100; 0x43D730; 0x5A6C70; 0x446C40;
0x523E80; 0x4E7170; 0x5A7080; 0x465830; 0x4658B0; 0x4762F0; 0x477470 at 0x477470–0x4776B0;
0x474D80 at 0x474FE9–0x47507F (the surrender price, "not read" so far); 0x4E4580; the static spell
records of 34, 38, 39, 40, 53, 65, 66, 69. `SPTRAITS.TXT` rows of the eight spells (power factor and
effects match the pack). Not re-read: 0x4641F0, 0x46A350, 0x43D8B0, 0x43D5C0, 0x4E5FF0, the
samplers 0x50C6E0/0x50C740, 0x464F50.
VCMI 1.7.5: `lib/battle/CUnitState.cpp` (CHealth, `afterNewRound`), `lib/battle/BattleInfo.cpp`
`nextRound`, `lib/spells/effects/Sacrifice.cpp`, `Clone.cpp`, `config/spells/other.json`,
`lib/battle/CBattleInfoCallback.cpp` (`battleIsFinished`, `battleGetSurrenderCost`),
`lib/battle/Unit.cpp` `getRawSurrenderCost`, `server/battles/BattleResultProcessor.cpp`.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-035 | every clause traced; the hit points, `<=` health test, cap, temporary counter and corpse search agree with the code reading | 0x5A1907, 0x5A7560, 0x5A3CC0, 0x5A3F50, 0x5A8090 case 0x5A8467, 0x444120, 0x447120, 0x475ED0, 0x4774E2; VCMI CHealth (lowers its counter: as the entry says); RA1, RA4, RA2, RA3 agree | behaviour pinned with synthetic numbers; gaps in F-10 | F-2, F-5, F-10 |
| BAT-MAG-036 | victim, formula, no corpse, permanence, active victim agree; the target test under the quirk is incomplete | 0x5A199C, 0x5A8090 case 0x5A8523 (both variants), 0x5A39C0, 0x59F196, 0x5A2D00, 0x5A2E90, 0x44A4D0; VCMI Sacrifice.cpp; RA1 | both variants of the flag tested | F-1, F-7 |
| BAT-MAG-037 | limit, set-up, no corpse, no victim, end and outcome agree; slot reuse differs for a stack whose rounds ran out | 0x47A100, 0x43D730, 0x444120, 0x59F447/0x59F563, 0x465830, 0x4658B0; VCMI `battleIsFinished`; SU, CL1 | limit, reuse, draw, price pinned | F-4 |
| BAT-MAG-038 | target test, level, count, rounds, death by any damage, links, place search (order, three skips) agree | 0x5A6C70, 0x446C40, 0x5A8090 case 0x5A841A, 0x444090, 0x444120, 0x447120; VCMI Clone.cpp (tiers 5/5/6/all, current count); CL1, CL2 | place, rounds, damage, original's death pinned | F-3, F-8 (F-4) |
| BAT-MAG-039 | count, column order, uniform row, binding (also without a place), new stack per cast agree | 0x5A7080, 0x59F55D–0x59F5BA, spell flags (no duration bit: no bonus in the count); SU rows | count, column, binding pinned; the row draw is not (F-10) | ok, F-10 |
| BAT-QUIRK-5 | on/off as the ledger says | confirmed by RA1 and by my re-read of 0x5A3950 → 0x5A3CC0 (spell 38 whichever spell asked); the original adds Sacrifice's own test | each variant has a test that fails for the other | F-1 |
| BAT-QUIRK-6 | fixed variant only (exploit-fix) | 0x4774E2 re-read: sums every record with creatures, the last such keeps one; RA2, RA3 | `a_side_left_with_creatures_raised_for_the_battle_only_keeps_one` | ok |
| BAT-QUIRK-7 | on/off as the ledger says | confirmed by CL2 and by my re-read (0x446C40 accepts the stack's own hexes; 0x47A100 refuses an occupied position; 0x5A6C70 then flags the stack on that hex) | each variant has a test that fails for the other; the "on" test ends the battle by hand | F-3 |
| BAT-QUIRK-8 | never implemented (exploit-fix) | re-read: 0x47A100 tests only the position hex, so a two-hex clone can cover the original's hex | unit test in both profiles and the property invariant | ok |
| BAT-QUIRK-11 | as implemented for killed stacks | 0x47A100, 0x43D730; SU a | `the_outcome_reports_each_created_stack_with_its_losses` | F-4 (the entry leaves out "with no creatures") |
| BAT-END-001 | `check_end` agrees with 0x465830/0x4658B0 (real side, dead bit, war machines; hypnotized stacks stay their owner's) | 0x465830, 0x4658B0, 0x4762F0, 0x4743B8, 0x479BDC; VCMI `battleIsFinished` (also leaves summoned and clones out of the winner); SU a–c | draw with only created stacks pinned | F-3 |
| BAT-END-003 | record order, origins, `end`, keep-one, survivors traced | 0x4774E2, 0x4775E1 (killed = start − count per record, no flag test), 0x444120 | unit tests and `outcomes_account_for_every_stack` | ok; F-4 touches the reused record |
| BAT-FLE-002 (E04b part) | created stacks left out: agrees | 0x474FE9–0x47507F read for the first time: summoned records skipped, war machines counted, **temporary creatures subtracted**; VCMI counts army-slot units at full count; CL1, SU | `the_surrender_price_leaves_created_stacks_out` | F-6, F-9 |

## Findings

### F-r4a-1: With BAT-QUIRK-5 on, Sacrifice's raised stack takes only Resurrection's test; the original also applies Sacrifice's own, so fire-immune creatures (Efreet) and summoned stacks are wrongly targets
- rules: BAT-MAG-036, BAT-QUIRK-5 (ledger and `installation.md` §7.18)
- kind: behaviour-bug (and source: §7.18 says "the raised stack takes the immunity factor of spell 38", which is half of it)
- severity: medium
- confidence: high for the code path (three call sites agree); VCMI agrees on the immunity part. An experiment would close it: Sacrifice's first cursor on wounded or dead Efreet with another stack to sacrifice.
- evidence:
  - The original. The target test 0x5A39C0 does two things for a target-stack spell: it finds the stack (0x5A3950: for spell 40 with its fourth argument set, Resurrection's finder 0x5A3CC0, which asks 0x5A8090 about spell 38) and then asks 0x5A8090 again about **the spell itself**, passing that same argument on as 0x5A8090's fifth. All three users pass 1 for the raised stack: the spell book (0x59F1FC), Sacrifice's first cursor (callback 0x5A2D00, call at 0x5A2D7D) and the cast (0x5A02EA, where the aimed stack is the one 0x5A008D found). In 0x5A8090, spell 40 with the fifth argument 1 skips the controller test (0x5A82B9) and takes case 0x5A8523 as "the stack to raise": creature flag alive, **not summoned** (`0x400000`), fewer creatures than at the start; then the common tail 0x44A4D0 with spell 40: no per-spell case (index 23 of the table at 0x44A92C is the default), so the general immunities apply, among them **fire immunity** (creature flag `0x4000` against a spell of the Fire school, 0x44A7BA) and Sacrifice's level 5. With the fifth argument 0 the same case is the victim's test (not undead, creatures left), as §7.18 describes.
  - So the raised stack must pass Resurrection's test (found by 0x5A3CC0) and Sacrifice's own.
  - The engine: `raise::test_for` (raise.rs:83–109) with the quirk returns a `RaiseTest` of the `tests_as` spell alone; `eligible` (raise.rs:116–138) then tests only that spell's `affects`, level and hit points, and has no `is_created` test. `stack_target` (magic.rs:1094–1096) asks nothing else.
  - Scenario (base pack, faithful): Efreet or Efreet Sultans (`alive`, `school_immunity = fire`, packs/base/pack.toml) with losses or dead, a hero with Sacrifice, enough Earth power for one Efreet's health. Original: the first cursor refuses them (factor 0 for spell 40). Engine: they are a target and are raised. In strict-fixed the engine refuses them (Sacrifice's own test), so the two profiles are the wrong way round here. Second case: with BAT-QUIRK-7 a turned original (summoned, living creature, with losses) is a Sacrifice target in the engine and not in the original.
  - VCMI 1.7.5 `lib/spells/effects/Sacrifice.cpp` `applicable`: the healed target goes through `Heal::applicable` with Sacrifice's own immunities (and must be dead, which the original does not ask).
- fix or test sketch: under the quirk, ask both tests (the `tests_as` spell's with its hit points, then the spell's own `affects`/`kept_off` and `!is_created()`); say so in the summary, the ledger and §7.18. Test (`tests/raise.rs`, needs-run), the synthetic `ember` is alive and immune to the fire school, `rise` is an air spell, `offering` a fire one:
  ```rust
  let r = ruleset_with("t:sacrifice", |_| {});
  let mut s = arena(&r, [Some(raiser(&r, 3, 1)), None],
      &[("fixed", 1, 2, 1), ("ember", 3, 5, 5), ("serf", 5, 9, 5)], &[("serf", 1, 14, 5)], 1);
  s.stacks.as_mut_slice()[1].count = 1;
  assert_eq!(targets(&s, &r, "rise"), vec![1]);
  assert_eq!(targets(&s, &r, "offering"), Vec::<u8>::new()); // today: vec![1]
  ```

### F-r4a-2: A dead stack's round state is frozen in the original until it is raised and a new round starts; the engine resets it at every round start, so a stack raised in a later round acts although it had acted before it died
- rules: BAT-MAG-035 ("keeps its round's flags (if it had acted this round it stays done)"), also every raise through 0x5A7560 (BAT-MAG-036, BAT-MAG-051)
- kind: behaviour-bug by the code reading; VCMI does it the engine's way, so by `docs/04` §2 this is a conflict of sources that needs an experiment before the engine changes (question for the user)
- severity: high if the experiment confirms it (raising in round n + 1 a stack that was killed in round n after it had acted is ordinary play)
- confidence: medium. The two facts are plain in the code; what is missing is an observation. Experiment: round 1, Red's stack X acts (any action), then Blue kills it (Implosion); round 2, on another Red stack's turn, Red casts Advanced Resurrection on X: does X get a turn in round 2? Control: Blue kills X in round 1 before X acts; raised in round 2 it should act.
- evidence:
  - The round start 0x475ED0 calls 0x447120 for every record with a creature, and 0x447120 returns at once when the record has no creatures (its first test, 0x44712D). Everything it does is therefore skipped for a dead stack: clearing the flag bits 24–26 (good morale, waited, **acted**, at 0x44713F), the "turn started" byte +0x4F0 and the retaliations +0x454. Besides this, only the good-morale roll (0x4649E6) clears the acted bit of a battle stack (I searched the whole code for the two masks, also in their byte forms; the other matches work on a 48-byte grid cell in 0x53xxxx–0x54xxxx). The raise 0x5A7560 clears only the dead bit (0x5A7867), as §7.18 says; the death 0x444120 sets only the dead bit. The pick 0x465080 passes over a record with the acted bit (0x46511C). (The start of the wait phase clears the waited bit of every record, dead or not, 0x4652CD.)
  - So a stack that had acted in the round it died is done in the round it is raised, whichever round that is, and has the retaliations it died with; it acts again from the next round start.
  - The engine: `turn::start_round` (turn.rs:80–87) resets `ACTED`, `WAITED`, `RETALIATED`, `HAD_MORALE`, `STARTED` and `retaliations_left` of every stack, dead ones included. The T6a record says the intent was "a raised stack keeps its round flags, shots and retaliations (the original clears only its dead bit)"; that holds only within one round. `a_dead_stack_comes_back_..._and_keeps_its_round` tests the same round only.
  - VCMI 1.7.5 `BattleInfo::nextRound` calls `afterNewRound` for every stack, dead or alive (`lib/battle/BattleInfo.cpp` L660–666, `CUnitState.cpp` L913–927): the engine's behaviour. The wiki is not quoted on this.
  - Scenario: round 1, own Swordsmen attack, then die; round 2, first own turn: Expert Resurrection on them. Original (reading): no turn in round 2. Engine: they act in round 2.
- fix or test sketch: in `start_round` leave dead stacks' flags and retaliations alone. Test (needs-run):
  ```rust
  let mut s = arena(&r, [Some(raiser(&r, 3, 0)), None],
      &[("fixed", 1, 2, 1), ("serf", 4, 5, 5)], &[("tank", 1, 14, 5)], 1);
  s.stacks.as_mut_slice()[1].flags.set(crate::StackFlags::ACTED, true);
  kill(&mut s, &r, 1);
  crate::turn::start_round(&mut s, &r, &mut Vec::<Event>::new());
  assert!(s.stacks()[1].flags.contains(crate::StackFlags::ACTED)); // today: cleared
  activate(&mut s, 0);
  cast(&mut s, &r, "rise", (stack(1), CastTarget::None)).unwrap();
  let (round, mut turns) = (s.round(), Vec::new());
  while s.round() == round {                                        // everybody defends to the round's end
      turns.push(s.active().unwrap().0);
      act(&mut s, &r, Command::Defend).unwrap();
  }
  assert!(!turns.contains(&1));                                     // today: the serfs act
  ```

### F-r4a-3: With BAT-QUIRK-7 (on in faithful) a turned original that is its side's last stack vanishes at a round start and the battle never ends
- rules: BAT-MAG-038, BAT-QUIRK-7, BAT-END-001
- kind: behaviour-bug, robustness (a battle without an end: the class `docs/04` §4 always fixes)
- severity: medium (a rare setup, but the default profile, and nothing bounds a battle's rounds)
- confidence: high (traced; the sketch is needs-run)
- evidence:
  - `turn::start_round` (turn.rs:92–94) runs `created::count_down`, which makes the turned original vanish (created.rs:489–503); nothing calls `check_end` there. `check_end` runs only after a strike, a shot, a cast and obstacle damage (combat.rs:773, 859; magic.rs:2233; obstacles.rs:643, 685). `Wait`, `Defend` and `Move` (command.rs:395–410) do not call it. The comment at turn.rs:69–70 ("never their side's last stack: a clone's original lives") is true only without the quirk.
  - After the vanish `next_active` picks the other side's stacks for ever. If that side has no hero that can cast and the vanished side has no war machine to strike (a Ballista's own shot would end it), only Wait, Defend and Move are legal: the battle has no end. With a hero, the next cast ends it; a random or learning agent may never cast.
  - The existing test hides it: `with_quirk_bat_quirk_7_a_crowded_original_becomes_its_own_clone` (tests/created.rs:303–311) calls `start_round` three times and then `check_end` by hand. The property tests discard games that do not end (`prop_assume!(state.is_over())`, properties.rs:904).
  - The original ends such a battle: the action dispatcher asks for the winner after every action whatever its kind (all twelve cases of its switch end at 0x479BD3, the call at 0x479BDC), and the battle loop 0x4740D0 asks too (0x4743B8; I read only around that call, so "on every pass" is likely, not shown); 0x465830 finds no living record of the side. At the latest the other side's next Defend ends it.
- fix or test sketch: `check_end` after `created::count_down` (and no stack active once it is over); a property invariant "while the battle is not over each side has a living stack that is no war machine". Test (`tests/created.rs`, needs-run):
  ```rust
  let r = ruleset_with("t:clone_on_original", |_| {});
  let mut s = walled(&r, 3, None);              // the attacker's only stack; the defender: one serf, no hero
  cast(&mut s, &r, "copy", stack(0)).unwrap();  // TurnedSummoned for 3 rounds
  for _ in 0..12 { if s.is_over() { break; } act(&mut s, &r, Command::Defend).unwrap(); }
  assert_eq!(s.result(), Some(BattleResult::Victory(BattleSide::Defender))); // today: None, for ever
  ```

### F-r4a-4: A created stack whose rounds ran out keeps its creatures in the original, so its record is never reused; the engine reuses its slot
- rules: BAT-MAG-037 ("takes the slot of its side's first dead created stack"), BAT-QUIRK-11, BAT-END-003, BAT-MAG-038
- kind: behaviour-bug (an edge case) and rule-text (the summary and the ledger drop "with no creatures")
- severity: medium (rare: the limit after many expired clones, the turn order of later created stacks)
- confidence: high for the reading; no observation (an experiment: Clone first, then a summon E1, the clone runs out, a second summon E2: with equal speed, which of E1 and E2 acts first? Original: E1, the engine: E2)
- evidence:
  - 0x47A100 takes the first record that is empty, or whose count (+0x4C) is 0 and that has the dead and the summoned bit (0x47A131–0x47A14A): §7.19 has it ("with no creatures"), the T6b log, the BAT-MAG-037 summary and BAT-QUIRK-11 do not. The death 0x444120 does not touch the dying stack's count (§7.21, and BAT-END-003 relies on it: "keeps the creatures it vanished with"), and 0x447120 calls it for a stack whose rounds reach 0 without zeroing the count. So an expired clone, and an expired turned original, stay in their record with their creatures: not reusable, and the side's records in use (+0x54BC) never fall. Only a created stack that was killed (count 0: damage, or a clone whose original died, 0x4444A4) is reused. SU a fits: E1 was killed by Implosion.
  - The engine: `vanish` zeroes the count (created.rs:472–484, the creatures go to `Created::vanished`), and `add` reuses any dead created stack (created.rs:410–413).
  - Effects: (1) the limit: with 7 army stacks, the 14th Clone after 13 clones ran out is refused in the original and allowed in the engine; (2) a later created stack takes the expired stack's id, so it comes earlier in the turn order's tie-break than in the original (new record after every other); (3) the outcome loses the expired stack's record; for a turned original that is the army stack's own record with its losses, which the original's Necromancy still counts (0x4775E1: start − count of every record).
- fix or test sketch: in `add`, reuse only a dead created stack with `vanished == 0`; say "killed" in the summary and the ledger. Test (needs-run):
  ```rust
  let clone = created(&cast(&mut s, &r, "copy", stack(0)).unwrap())[0].0;
  let mut ev = Vec::new();
  for _ in 0..3 { crate::turn::start_round(&mut s, &r, &mut ev); }      // the clone's rounds run out
  let next = created(&cast(&mut s, &r, "sprites", CastTarget::None).unwrap())[0].0;
  assert_ne!(next, clone);                                              // today: equal
  ```

### F-r4a-5: A hero's Resurrection or Animate Dead aimed at a two-hex corpse raises whatever corpse is on top of that corpse's position hex; the engine raises the corpse itself
- rules: BAT-MAG-035 ("on one of its hexes, no corpse above it is also a target")
- kind: behaviour-bug
- severity: medium (a rare setup: a later corpse on a two-hex corpse's rear hex)
- confidence: medium to high; an experiment as RA4 would settle it (Wolf Riders die; a Swordsman dies later on their rear hex; Resurrection aimed at the Wolf Riders' front hex: who comes back?)
- evidence:
  - The cast routine finds the stack at the aimed hex (0x5A008D → 0x5A3950), but the handler 0x5A1907 does not raise that stack: it takes that stack's position (+0x38) and searches again from there (0x5A3CC0 or 0x5A3F50 with the position, 0x5A1917–0x5A1928), and raises what this second search returns. For a two-hex stack +0x38 is the rear hex (0x444297: the other hex is position + 1 for a stack facing right), the engine's tail. Each hex has its own corpse list.
  - So for a two-hex corpse A (tail p, head q) with a later corpse B on p that passes the test: aimed at q the first search finds A (top of q), the second finds B (top of p), and B is raised; aimed at p, B is found. A cannot be raised while B lies there. The rule that follows: a dead stack is raised only if it is the top corpse that passes the test on its position hex.
  - The engine: `on_top` (raise.rs:179) accepts any hex of the footprint, so A is a target through q and A is raised. (Sacrifice raises the found stack directly, 0x5A1A49, and the Archangel too, 0x4485BE: only the hero's two raise spells search twice.)
- fix or test sketch: for the hero's raise spells let `on_top` look at the tail (or only) hex. Test (needs-run): kill the ogres (two-hex), put the serfs on the ogres' tail hex and kill them, then `assert_eq!(targets(&s, &r, "rise"), vec![2])` (today: both).

### F-r4a-6: The surrender price counts creatures raised for the battle only; the original leaves them out
- rules: BAT-FLE-002 (with BAT-MAG-035's temporary creatures, an E04b addition)
- kind: behaviour-bug by the code reading (VCMI counts the full count; the wiki is silent: an experiment candidate by `docs/04` §2)
- severity: medium (Basic Resurrection, then a surrender)
- confidence: high for the reading; experiment: RA2's steps, the surrender dialog before and after the Basic Resurrection (10 Swordsmen at 300: 5 left 750; after the raise to 8 still 750, the engine 1 200)
- evidence:
  - The price is computed in the battle window's button handler, 0x474FE9–0x47507F (so far "not read", §7.19): over the acting side's records 0–19 with a creature and creatures, it skips a record with the summoned bit (0x47501F) and a record with no more creatures than its temporary counter +0x54 (0x47502A), and adds the creature's gold cost (creature record +0x38) × (count − temporary counter). War machines have no test, so they count. Then the sum is halved as an integer and multiplied by the hero's factor 0x4E4580 (1 − discount, at most 0.9 off), truncated.
  - This confirms what batch 2d showed (created stacks out, war machines in) and adds the temporary creatures.
  - The engine: flee.rs:51–52 sums `count × cost` of every living stack that is not created.
  - VCMI: `Unit::getRawSurrenderCost` is cost × count for army-slot units.
- fix or test sketch: `s.count.saturating_sub(temporary)` in `surrender_price`. Test (needs-run): a stack of 10 reduced to 4, the price P4; Basic `rise` brings it to 10 (`temporary` 6); `surrender_price` must still be P4 (today: the price of 10).

### F-r4a-7: The original does compare Sacrifice's victim with its target; the summary and §7.18 call this an engine choice
- rules: BAT-MAG-036 ("Engine choice: the original's code does not compare the victim with the target (its second cursor was not read)"), `installation.md` §7.18 line 1013, map entry 0x59F700
- kind: rule-text, source
- severity: low
- confidence: high
- evidence: Sacrifice's second cursor is the callback 0x5A2E90 (no direct caller, not in the map). On a mouse move it asks the target test with the victim variant (0x5A2F11: 0 where the first cursor passes 1) and then compares the living stack on the hovered hex with the living stack on the first target's hex (+0x44) and refuses when they are the same (0x5A2F3B–0x5A2F63). With a dead first target there is no living stack on its hex, and every victim passes. RA1 observed exactly this. The first cursor is 0x5A2D00 (see F-r4a-1). The engine's `v != s` (magic.rs:1172) is the original's rule.
- fix or test sketch: reword the summary, §7.18 and the map's summary; add 0x5A2D00 and 0x5A2E90 to the map.

### F-r4a-8: A clone of a stack raised from a corpse keeps the trait RAISED: `StackTraits::with(_, false)` clears nothing
- rules: BAT-MAG-038 (with BAT-MAG-052)
- kind: behaviour-bug (observations only)
- severity: low
- confidence: high
- evidence: created.rs:236 writes `.with(StackTraits::RAISED, false)` to drop the trait, but `with` (state.rs:208–210) returns the set unchanged when `on` is false. A clone of a Pit Lord's Demons (level 4: Basic Clone takes them) therefore has SUMMONED, CLONE and RAISED. The outcome is right (`is_created` is asked first, state.rs:1181) and so is the turn order, but the sim's observation (h3-sim observation.rs:394) and the protocol view (h3-protocol map/mod.rs:689) report the clone as raised from a corpse.
- fix or test sketch: clear the bit (a `without`), and in `tests/raisers.rs` clone the Demons and assert `!clone.has(StackTraits::RAISED)`.

### F-r4a-9: The surrender price halves the cost before the discount in the original (outside E04b's change)
- rules: BAT-FLE-002 ("rounded down once")
- kind: behaviour-bug (1 gold)
- severity: low
- confidence: high for the reading; it is E04a's rule, noted here because 0x474FE9 was read for F-r4a-6
- evidence: 0x47506B–0x47507F: the integer sum is halved first (rounded toward 0), turned into a single-precision value, multiplied by the factor and truncated. The engine multiplies cost × 50 × kept and divides once (flee.rs:67). They differ only for an odd sum with some discounts: one Pixie (25) at a 10 % discount is 12 × 0.9 = 10 in the original and 11 in the engine. With the discounts 20, 40 and 60 % and costs in steps of 5 I found no difference under Q-0005.
- fix or test sketch: under Q-0005 compute `(cost / 2) × kept`.

### F-r4a-10: Test gaps that matter in play
- rules: BAT-MAG-035, BAT-MAG-038, BAT-MAG-039
- kind: test-gap
- severity: low
- confidence: high
- evidence:
  - BAT-MAG-035: the health test's boundary is not pinned. The tests have 230 and 190 hit points against a health of 200 (tests/raise.rs:148–161); hit points equal to the health (the original's `<=`, 0x5A850C) would survive a change to `<`.
  - BAT-MAG-039: the row draw is not pinned: the summon test asserts the column only (tests/created.rs:410, 441); always taking the first fitting row would pass. "No place: nothing happens, the side stays bound" has no test.
  - BAT-MAG-038: of the three skips of the place search only "right of the position in ring 1" is pinned (tests/created.rs:185–188); the skip to the left, and from the head towards the tail in rings 1 and 2, are not. No test runs a clone's or a turned original's end through real commands to a round start (F-r4a-3).
  - No test of this slice replays a batch 2d row with the row's own numbers (RA2: 5 Swordsmen with 25 → 8 → 2 → 1 kept; CL1: 800; SU: 910): the unit tests use the synthetic pack. That the temporary counter is never lowered is pinned only by a property (properties.rs:555).
- fix or test sketch: one assertion each; for RA2 a base-pack test if the lane has a place for one.

## Wiki checks wanted

- Resurrection (and Archangel): does a stack raised in a later round than it died get a turn in that round when it had already acted in the round it died? (F-r4a-2; also the HotA changelog for a fix of this.)
- Sacrifice and Efreet / Efreet Sultan / Phoenix: may a creature immune to Fire magic be the stack Sacrifice raises? (F-r4a-1)
- Combat, Surrender: is the price taken from the creatures raised by Basic Resurrection too? (F-r4a-6)
- Clone: do clones that ran out still count towards the 20 stacks of a side? (F-r4a-4)

## Not reviewed, left open

- Not re-read in the executable: the army rebuild 0x4641F0, the experience 0x46A350, the record set-up 0x43D8B0/0x43D5C0 (the start values of +0x24, +0x28, +0x2C), the specialty bonus 0x4E5FF0, the row samplers, the tie-break 0x464F50. Necromancy's loop (0x4775E1) was read only for "killed = start − count of every record".
- BAT-MAG-051/052 (`casters.rs`, `raise_corpse`) only where they share `add`, `on_top` and the outcome.
- The sim's, the protocol's and the C ABI's encodings of these states, apart from the RAISED flag of F-r4a-8.
- Siege branches (0x467130 in the summon is the gate; E04c) and the special terrains (+0x53C0).
- BAT-QUIRK-11 is "preserve, no flag" by the user's decision (BAT-Q13); `docs/04` §4 gives a preserve quirk a flag, so strict-fixed keeps it too. Not a finding; noted for the coordinator.
- Robustness and security of this slice's code: nothing found. Counts and hit points are widened before they are multiplied (`sacrifice`, `resurrect`, the summon's count fits `u32`), ids are validated before `at()`, `add` cannot overfill the stacks while the limit is at most 21, the second target is validated like the first, and the new events and observations carry only public facts.
