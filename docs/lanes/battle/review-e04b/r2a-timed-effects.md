# Review r2a-timed-effects: BAT-MAG-009..017, Q-0007, Q-0008, Q-0012, BAT-DMG-002 (E04b part)

Method: every rule's implementing functions read completely (`effects.rs`, `damage.rs`,
`arith.rs` `X87`, the strike path of `combat.rs`, `turn.rs` `start_round`, `morale.rs`
`materialize`, and in `magic.rs` `plan_cast`, `reach_stacks`, `affect`, `protected`,
`spell_damage`, `reachable`/`skips_hypnotized`/`accepts`); the executable re-read with
`scripts/exe.py` at the addresses listed per rule; VCMI 1.7.5 opened where the catalog leans on it;
batch 2b (R1–R6) and 2c (H1, C1) read. No cargo run. For BAT-DMG-002 I ported `X87` and
`Strike::staged` by hand to Python integers and compared them with a model of the executable's
stages in IEEE doubles and `f32` stores (what I read at 0x443F40, 0x443320, 0x443B90): the 2b and
1a fixtures and 300 000 random strikes, 0 mismatches; the fixtures also recomputed in exact
rationals (53 bits: 195/390, 64 bits: 194/389; 121/40/189; 619/154).

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-MAG-009 | agrees (counters, re-cast, Weakness min, Prayer, Disrupting Ray floor) | 0x4448F0 (fixed durations, the already-active path, cases 27–34, 41–46, 48–56, 58–60), 0x5A1A53–0x5A1AD5, 0x5A00B2–0x5A00C4; the mastery array +0x2DC has no reader but 0x4448F0 itself; VCMI `config/spells/timed.json` counters (10 entries, as cited); 2b R5 | pinned (`timed.rs`), numbers differ from R5 but same behaviour | ok (Defend interplay: F-4) |
| BAT-MAG-010 | agrees except a dead stack's permanent effects | 0x447120 (whole), 0x475ED0 around 0x4760F8, 0x444510 (refusal of 47, cases 45, 46, 48, 53, 54), 0x444120 (to 0x444259), 0x5A7560, 0x43EA70; 2b R2 | durations, minimum, death of a timed effect pinned; a dead stack's permanent effect not | F-3 |
| BAT-MAG-011 | attack, speed, Bless/Curse agree; morale/luck clamp and Defend order do not match the executable | 0x442410, 0x442590, 0x448CD0, 0x443160 (all whole), 0x43DF20, 0x43E000, 0x44B2D0 (whole), 0x44AE60 (start, end), 0x4E36C0/0x4E39B0 (ends), 0x469750, roll sites 0x43F928, 0x44180A, 0x46499D, 0x46DD92, the Defend action 0x47974A–0x4798D7, 0x446E30 start; VCMI `lib/BasicTypes.cpp` L81–127, `DamageCalculator.cpp` L79–121, `BattleActionProcessor.cpp` L160–198; 2b R5 (does not pin Slow's rounding: 4 × 0.75 is exact) | the morale test pins the wrong number (F-1) | F-1, F-4 |
| BAT-MAG-012 | agrees | 0x443F40 (whole: Shield +0x4B8 on melee, Air Shield +0x4BC on shots, by duration; one product with 1 − Armorer, one truncation), 0x4E4310; 2b R4 | `protect.rs`, `rules.rs` (121, 40, 189) | ok |
| BAT-MAG-013 | agrees | 0x5A7BB0 (whole: earth, air, fire, water; `f32` damage × `f32` factor, truncated), 0x5A78E0 start (after the creature factor); VCMI `CSpellHandler.cpp` L378–394, L125–137 (first school with a protection, one only) | three profiles | ok |
| BAT-MAG-014 | agrees | 0x442410 (reads the stored mastery +0x48C: flag 0x80 at any, 0x100 from 2, 0x200 from 3; +8 constant), 0x444D50 (stored at the first cast only); VCMI `DamageCalculator.cpp` L156–186, `config/creatures/*.json` KING, `timed.json` slayer; pack king flags (16 base, 6 ab creatures) | by mastery and king level; first-cast mastery pinned through Forgetfulness (`actions.rs`) | F-5 (source note only) |
| BAT-MAG-015 | agrees | 0x444D75 (v stored, added to +0x454), 0x447120 (reset 1/2/5000, + v while the duration is not 0, then the countdown), 0x444510 (nothing given back), 0x441D9C–0x441E65; 2b R2 | both profiles, on the counter | ok |
| BAT-MAG-016 | agrees for aimed Dispel and Cure's list; Expert Dispel differs on hypnotized stacks; a stack with only Acid Breath | 0x5A15CE–0x5A1779 (whole case), 0x5A8090 start to 0x5A8231 and its switch (no case for 35/37 beyond the early Dispel block), 0x59EE7A–0x59EEE2, 0x444510, 0x5A66D0 (whole), 0x5A1E70–0x5A1F84; VCMI `config/spells/other.json` dispel | no Expert Dispel × hypnotized test | F-2, F-6 |
| BAT-MAG-017 | agrees | 0x5A17E6–0x5A18DC (whole case), 0x446500 (whole: 14 ends in the pack's order, then lost hit points − value, at least 0); VCMI `other.json` cure; 2c H1 (5 → 35 at power 1 Expert) | order, cap, healthy target, mass | ok |
| BAT-DMG-002 (E04b) | agrees | 0x443F40, 0x443B90 (whole), 0x443320 (all but the message code), 0x443840 (to the death blow), 0x4E4310, 0x4E42B0, 0x4E4160; 2b R1, R3, R4, R6; Python port against IEEE doubles | fixtures of 1a and 2b, stage truncations | ok |
| Q-0007 | both variants, test fails for the other | 0x5A7BB0 re-read (second look); VCMI applies one protection too (its own order); HotA changelog as cited | `a_spell_of_two_schools_meets_one_protection` | ok |
| Q-0008 | both variants, test fails for the other | 0x44717C–0x4471E4 re-read (retaliations first, then the countdown); 2b R2 | `counterstrike_adds_retaliations_now_and_at_each_round_start` | ok |
| Q-0012 | the quirk itself as described; its flag also gates Expert Dispel | 0x5A186E–0x5A1877 re-read (Cure's Expert loop skips a stack with Hypnotize's duration set); 2c H1, C1 | Cure in both profiles | F-2 |

## Findings

### F-r2a-1: Mirth, Sorrow, Fortune and Misfortune are added after a ±3 clamp of the base that the original does not make
- rules: BAT-MAG-011 (clause "the changes on the side summed and clamped to ±3, then Mirth's … added"); `installation.md` §7.9; survey §14
- kind: behaviour-bug (and source: the reading the clause rests on is wrong)
- severity: high (any army whose base morale or luck is beyond ±3 — Expert Leadership or Luck plus one more point — meets Sorrow, Misfortune, Mirth or Fortune with a wrong result; Armor of the Damned casts Expert Misfortune on every enemy stack)
- confidence: high. An experiment would close it: a hero with base morale +5 (Expert Leadership, one alignment, Angels), Expert Sorrow on a stack, then count extra turns or read the stack window.
- evidence:
  - Code: `crates/h3-battle/src/morale.rs:314-318`: `sum = evaluate(0, adds).clamp(low, high)`, then `evaluate(sum + spells, caps and minimums)`. Test `tests/timed.rs:453-456` pins it ("Morale 4 is clamped to 3 before Sorrow's −1: 2").
  - Executable: the stack's luck routine (0x43DF20) calls the base luck routine 0x44B2D0 with its last two arguments 0 (the two zero pushes at 0x43DF6D and 0x43DF6F). 0x44B2D0 clamps to ±3 only when its last argument is set (the test at 0x44B380 skips the clamp block for 0), and it calls the hero's luck 0x4E36C0 with the clamp flag 0 as well (0x44B31A–0x44B31F; that routine's clamp at 0x4E3977 is conditional in the same way). The stack's morale routine (0x43E000) likewise passes 0 as the last argument of 0x44AE60 (the first push at 0x43E038, the register is zeroed at 0x43E007); 0x44AE60 clamps only when that argument is set (0x44B0B0) and calls the hero's morale 0x4E39B0 with flag 0 (0x44AE8C–0x44AE8F; conditional clamp at 0x4E3C10). So +0x4EC and +0x4E8 hold unclamped base + Fortune − Misfortune and base + Mirth − Sorrow (then the Halfling and Minotaur minimums, Spirit of Oppression, Hourglass). The only clamps on the stack path are at the uses: luck 0x43F936 (shots) and 0x441818 (melee) take min(luck, 3), the good-morale roll 0x4649AA clamps to ±3, and one more reader (0x46DD92, not identified) clamps the same way. +0x4E8 and +0x4EC are written only by these two routines (0x43DFD2, 0x43DFDF, 0x43E0F2, 0x43E0FE; besides a whole-record copy at 0x437EB0).
  - `installation.md` §7.9 lines 468–472 says "the base luck, clamped to ±3 by its own function" and "the base morale clamped to ±3": the callee can clamp, but this caller asks it not to.
  - VCMI 1.7.5 `lib/BasicTypes.cpp` L100–102 and L124–126: the total of all MORALE (LUCK) bonuses, spells included, clamped once. BAT-MAG-011's VCMI source does not cover this clause; BAT-MOR-001's does and agrees with the executable as re-read.
  - Scenario: base morale +5, Expert Sorrow (−2): original 5 − 2 = 3 → rolls at +3; engine clamp(5) − 2 = +1. Base luck +5, Expert Misfortune: original +3, engine +1. Base morale −5, Expert Mirth: original −3, engine −1.
- fix or test sketch: drop the first clamp (`evaluate(spells, adds ∪ caps ∪ minimums)`, then the final clamp, which stays); `timed.rs:456` becomes 3; correct §7.9, the survey and the summary ("clamped once, at the roll").

### F-r2a-2: with quirk Q-0012 on (faithful) Expert Dispel passes over hypnotized stacks; the original's Expert Dispel reaches them
- rules: BAT-MAG-016 ("at Expert (mass) it reaches every living stack of both sides"), Q-0012, BAT-MAG-026/003 (the sentence "Mass versions pass over hypnotized stacks")
- kind: behaviour-bug
- severity: high (in the default profile Expert Dispel no longer ends Hypnotize, nor any other effect on a hypnotized stack of either side)
- confidence: high on the code and on the executable; an experiment would confirm (hypnotize a stack, its owner casts Expert Dispel, is Hypnotize still listed).
- evidence:
  - Code: `magic.rs:854-874` `skips_hypnotized` returns `quirk || !ends_it` for every mass version; Dispel at Expert is a mass version (`mass = "expert"`), so with `quirk.battle.mass_cure_skips_hypnotized` on, `reachable` (`magic.rs:775-786`) is false for every hypnotized stack and the mass loop (`magic.rs:2381-2392`) never calls `affect` on it. Without the quirk Dispel reaches it (its dispel ends Hypnotize). `content/src/quirks.rs:114-117` documents the flag as "every mass version skips hypnotized stacks".
  - Executable: Dispel has its own case (0x5A15CE, switch value 35). Without a target stack it loops over both sides' stack records (0x5A160F–0x5A16D0): for each it calls the target test 0x5A8090 and, when the result is above 0, ends every spell id but 71. There is no test of Hypnotize's duration (+0x288) in that loop. The test for spell 35 (0x5A8138–0x5A8231) returns 1 at Expert unless the controlling hero wears the Sphere. The two places that do skip hypnotized stacks are the general mass handler (0x5A6713) and Cure's Expert loop (0x5A186E), as §7.14 says; §7.14 and BAT-MAG-026's source do not mention Dispel's loop.
  - VCMI: `config/spells/other.json` dispel at Expert has range X on every unit, no exception.
  - Scenario: the attacker hypnotizes a defender stack; the defender's hero (Expert Water) casts Dispel. Original: Hypnotize ends, the stack is the defender's again. Engine, faithful: the stack is skipped and stays hypnotized.
- fix or test sketch: Dispel's mass version never skips (the skip belongs to mass versions that go through the timed handler, and to Cure under Q-0012); the ledger's "as the mass handler of every other mass version does" and BAT-MAG-026's "Mass versions pass over hypnotized stacks" need the exception. Test (needs-run), in `tests/control.rs`, expected to fail today for `t:hypnosis`:
  ```rust
  for profile in ["t:faithful", "t:hypnosis"] {
      let r = ruleset_with(profile, |_| {});
      let mut s = arena(&r, [None, Some(adept(&r, &["dispel"], 3))],
          &[("fixed", 5, 2, 5)], &[("fixed", 5, 9, 5), ("fixed", 5, 10, 7)], 1);
      give(&mut s, &r, (1, "hypnotize"), 1);
      activate(&mut s, 2);
      act(&mut s, &r, cast(&r, "dispel", CastTarget::None)).unwrap();
      assert!(!has(&s, 1, Status::HYPNOTIZED), "{profile}");
  }
  ```

### F-r2a-3: a stack that dies loses Disrupting Ray (and Acid Breath); in the original both survive its death and a raise
- rules: BAT-MAG-010 ("A stack that dies loses its effects" against "a permanent spell lasts the rest of the battle"), BAT-MAG-009 (Disrupting Ray), BAT-MAG-035/059 (raise, rebirth), BAT-MAG-047 (Acid Breath)
- kind: behaviour-bug
- severity: medium (needs a stack with the lowered defense to be killed and then raised or reborn)
- confidence: medium-high; not observed. Experiment: two Disrupting Rays on a stack, kill it, Resurrection, right-click (defense, spell list).
- evidence:
  - Code: `effects.rs:927-945` `remove_dead` drops every entry of a dead stack, permanent ones included; `raise.rs:269-277` `revive` recomputes from an empty list, so the stack returns with its full defense. `tests/timed.rs:338-360` pins the removal for a timed effect only.
  - Executable: the death routine ends the active spells by calling the end routine for each (the loop 0x444228–0x444244); the end routine returns at once for spell 47 (0x44452D–0x444530), so the duration and the lowered +0xCC stay on the dead record. The raise 0x5A7560 only recomputes count and hit points, rebuilds the Unicorn aura lists (0x43EA70) and clears the dead bit (0x5A7867); nothing resets +0xCC. Acid Breath (0x5A1E70) lowers +0xCC with no duration at all, so death cannot touch it either.
  - VCMI `lib/battle/BattleInfo.cpp` L767–775: the comment at the kill/resurrect clean-up says "Special case: DISRUPTING_RAY is absolutely permanent" (its selector looks inverted in 1.7.5, so VCMI's behaviour is no witness, only its stated intent).
  - Scenario: Expert Disrupting Ray twice on Dread Knights (defense −10), they die, Animate Dead: original defense still 10 lower and the Ray listed; engine: full defense, no Ray.
- fix or test sketch: keep entries with `rounds == None` on dead stacks (or re-apply at a raise); test (needs-run): cast `ray`, set the stack dead, end a command, `raise::raise(…)`, assert the entry and `defense_now()` are those before the death.

### F-r2a-4: the Defend bonus is recomputed from the defense at each strike; the original fixes it when the stack defends
- rules: BAT-MAG-011 ("before the Defend bonus, which adds its share of a defense of at least 0"), BAT-MAG-009 (Disrupting Ray's floor at 0), BAT-TRN-004 ("spell bonuses join the base with E04b")
- kind: behaviour-bug; a conflict of sources for the user (the executable against VCMI's model)
- severity: medium (one or two defense points whenever a defense effect starts or ends between a stack's Defend and its next activation; common with mass Stone Skin/Prayer and with effects running out at a round start)
- confidence: medium-high on the reading; experiment: Defend, then own Stone Skin, right-click the defense.
- evidence:
  - Code: `damage.rs:295-302`: while `DEFENDING`, `defense_now() + max(⌊max(defense_now, 0) × 20 %⌋, 1)`, computed when struck. `effects.rs:372-375, 495-499`: Disrupting Ray floors `defense_now()` (without the Defend bonus) at 0.
  - Executable: the Defend action computes max(1, +0xCC × 20 / 100) once (0x47974A–0x479791), stores it at +0x4DC and adds it to +0xCC (0x4798C9–0x4798D7); the stack's next turn start subtracts the stored amount (0x446E74–0x446E87). Stone Skin, Prayer and Disrupting Ray change +0xCC directly (0x444BCE, 0x444C08, 0x5A1A8C–0x5A1ACB), so the stored bonus does not follow them, and the Ray's floor at 0 applies to the defense with the bonus in it. `docs/exe/structs.toml` already names +0xCC "defense (Defend bonus included)".
  - VCMI `server/battles/BattleActionProcessor.cpp` L167–198: a +20 % `PERCENT_TO_ALL` bonus, so dynamic like the engine (with a TODO to leave spell boosts out); the wiki line quoted in BAT-TRN-004 ("after bonuses and spell effects") fits both.
  - Scenarios. (a) Defense 12, Defend (bonus 2), then Expert Stone Skin (+6): original 12 + 2 + 6 = 20, engine 18 + 3 = 21. (b) Defense 14 with Stone Skin (+6), Defend (bonus 4, total 24), Stone Skin runs out at the next round start: original 18, engine 14 + 2 = 16. (c) Defense 2, Defend (total 3), Expert Disrupting Ray (−5): original 0 while defending and −1 for the rest of the battle after its next turn; engine 1, then 0.
- fix or test sketch: store the bonus at Defend (a field or an effect-like entry) and add it as a constant; or record the conflict in OPEN-QUESTIONS for the user with the experiment.

### F-r2a-5: BAT-MAG-014's VCMI note is wrong: VCMI agrees that Slayer at mastery none counts against king-1 creatures
- rules: BAT-MAG-014 (source `vcmi`)
- kind: source
- severity: low
- confidence: high
- evidence: the note says "VCMI's KING bonuses compare it, so at mastery none Slayer affects no creature; the original affects king 1". VCMI 1.7.5: `config/creatures/dungeon.json:474-478` `"KING_1" : // Will be affected by Slayer with no expertise`, `val 0` (2 for Angels and Devils, 3 for Giants and Titans); `config/spells/timed.json:1079-1084` Slayer at none `val 0`; `lib/battle/DamageCalculator.cpp:165-170` `isAffected = spLevel >= slayerAffected`. So 0 ≥ 0: king 1 at none, as the executable (0x442479–0x4424A4). The rule is implemented on a reading alone; this agreement is the consensus `docs/04` §2 asks for, and VCMI's specialist bonus max(5 − tier, 0) (L176–180) matches the table 4, 3, 2, 1 too.
- fix or test sketch: correct the note to "VCMI agrees".

### F-r2a-6: Dispel below Expert may be aimed at a stack whose only effect is Acid Breath; the original's test does not see Acid Breath
- rules: BAT-MAG-016 ("below Expert only at a stack with an active timed effect"), BAT-MAG-047
- kind: behaviour-bug
- severity: medium by the scale (a rare setup: Rust Dragons), with a small effect: a useless cast is legal and the action mask differs
- confidence: high on the code, medium-high on the executable
- evidence: `magic.rs:901` accepts any entry of the effect list; the engine keeps Acid Breath as a permanent cumulative entry (`packs/base/pack.toml:4130-4143`, applied through `after_strike::apply`). The original's test looks for a duration above 0 among spell ids 10–80 (0x5A81A2–0x5A81B9), and Acid Breath's handler (0x5A1E70–0x5A1F84) changes +0xCC without calling the apply routine, so it sets none. Disrupting Ray does set one (255), so a stack with only the Ray is a target in both, as tested.
- fix or test sketch: let `needs_effect` count only entries of spells that the original lists (a spell flag), or accept the difference in the rule text.

## Wiki checks wanted
- Morale, Luck (and Sorrow, Misfortune, Mirth, Fortune): does morale or luck beyond ±3 absorb these spells (F-r2a-1)?
- Dispel, Hypnotize: does Expert Dispel end Hypnotize, any note on hypnotized stacks being skipped (F-r2a-2)?
- Disrupting Ray, Rust Dragon/Acid Breath, Resurrection: does the lowered defense stay on a stack that dies and is raised (F-r2a-3)?
- Combat/Defend, Stone Skin, Prayer: is the +20 % fixed when the stack defends (F-r2a-4)?
- Slayer: the list at no skill (F-r2a-5).

## Not reviewed, left open
- The specialist parts inside my rules' functions (0x4E5FF0, BAT-MAG-020) and the wall and distance routines of stage 2 (0x467510, 0x467600) were not re-read; the middle of 0x44AE60 (alignments, terrain) neither.
- The Rust `X87` itself was not run; the check above is a hand port.
- Stage 1's hero step: the executable runs it for any hero, also with a rate of 0, which rounds the bonus sum through `f32` (0x44371C–0x443759); `damage.rs:261-267` skips it at rate 0. They differ only for a bonus sum above 2^24.
- 2b R5 shows Slow's speed 3 of 4 (exact); the truncation of BAT-MAG-011's slowed speed rests on the reading (0x448CFA–0x448D12, re-read) and a unit test.
- R2 is reproduced on the retaliation counter, not by two attacks that are struck back.
- Outside my slice, met while checking the morale readers and not traced in the Rust code: the bad-morale check 0x464B40 (the map says "not read in detail") clamps the stored morale to ±3, draws random(1, 12) against −morale and then, for a side whose local-human byte +0x54A4 is 0, lets the stack off when a further random(1, 4) gives 1 (0x464BDB–0x464BF8). Whoever holds BAT-MOR-004 should look at it.
- No wiki access.
