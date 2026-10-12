# Docs notes of worker `d` (E04b T11b, docs list D)

Worktree `/home/claude/wt-d`, branch `battle/e04b-t11b-d` (base `64fe5cf`). Files: the rule catalog,
the quirks ledger, `docs/formats/installation.md`, `docs/exe/map.toml`, `docs/exe/structs.toml`.
Checks at the base: `check-rules` ok (183 rules), `check-docs` ok, `check-exe-map` ok (341 entries).
The reading tool runs from `<SCRATCH>/wd/exe-root` (a copy of `scripts/exe.py`, links to the
worktree's `docs` and `crates` and to `/home/claude/homm3/original-data`), so it reads the
worktree's map; nothing is added to the worktree.

## Part 1: the 21 fixes' texts

### List A: done, commit `7962b1e` (pushed)

- A1: BAT-WAR-002, BAT-MAG-031 as proposed.
- A2, A3, A8: no text (as the notes say).
- A4: BAT-MAG-060, BAT-MAG-024, BAT-MAG-010 as proposed; BAT-MAG-043 got one sentence that points
  to BAT-MAG-024.
- A5: BAT-GEO-004 as proposed.
- A6: BAT-END-001, BAT-MAG-038, ledger BAT-QUIRK-7 (test name). BAT-END-001's sentence on the
  original is mine (no report id in a summary; "at the latest after the other side's next
  action", and the battle loop's test 0x4743B8 "read only around that call", as r4a F-3 has it).
- A7: BAT-FLE-001 says "when a stack it commands is active ..., in a tactics phase too" (the
  notes' "could say").
- A9: BAT-MAG-041 (summary and its code-reading source), `installation.md` §7.20. I read
  0x4411FD (the result is compared with 0 right after the modifier's call and the case leaves
  when it is not above 0): the notes' worker had not.
- A10: BAT-MAG-025, 026, 008, BAT-WAR-002/003/004/007, BAT-RNG-001, BAT-ATK-003; §7.14 (Berserk,
  Hypnotize); map 0x442880, 0x43EA70, 0x469600, 0x476490; new map entries 0x473EA0 (the automatic
  tent, read by me: it matches r3's description), 0x523B20 (read) and 0x523BB0 (inferred: only
  its call of 0x442880 at 0x523D53). I read 0x442880 and 0x469600 again: as r3 F-1 says.
- A11: BAT-MAG-026, 003, 016, ledger Q-0012 (also BAT-MAG-016 in its rules), §7.14, §7.10.
- VCMI sources added or corrected with list A (all opened in `/home/claude/vcmi`, 1.7.5):
  BAT-WAR-002 (the function is `tryMakeAutomaticActionOfRangedUnit`, L457-L560; the old ref named
  `tryMakeAutomaticActionOfBallistaOrTowers`, which 1.7.5 does not have), BAT-WAR-004
  (`unitHasAmmoCart`), BAT-MAG-025 (new), BAT-MAG-026 (the Hypnotize limit corrected: r3 F-9's
  part; a second source for the relations), BAT-RNG-001 (new), BAT-ATK-003 (new).

### List B: done, commit `972e9d1` (pushed)

- B1: BAT-MAG-059 (summary, source), §7.21.
- B2: BAT-MAG-010, 035, 044, 046, 037 (summaries; the sources of 010 and 035), §7.9, §7.18; map
  0x444120. (The HANDOFF trap and the survey line the notes name are not mine.)
- B3: BAT-MAG-044.
- B4: BAT-MAG-011 (summary, source; a new VCMI source, `lib/BasicTypes.cpp` L81-L127: one clamp
  of the total, agrees), BAT-MOR-001, BAT-LCK-001, §7.9; map 0x43DF20, 0x43E000, new 0x44B2D0 and
  0x44AE60.
- B5: BAT-DMG-006 (summary; `quirks = ["Q-0005", "Q-0009"]`, which r2b F-1 asks for; a new
  code-reading source: the rule had none; the VCMI source now says that VCMI's formula is the
  executable's number under Q-0005), ledger Q-0005, §7.11; map 0x442590, new 0x63B8AC (the two
  constants; 0.8 is also the Unicorn aura's factor at 0x5A85C3). I read 0x4425C8 to 0x4425F5:
  as r2b F-1 says (the product is subtracted from the single D, then truncated).
- B6: BAT-MAG-004 (summary, source), BAT-MAG-018, BAT-MAG-028 ("none if that side has no hero"),
  ledger Q-0005, §7.8; map 0x4E5760, 0x5A78E0. BAT-MAG-033 already points to Fire Wall's bonuses.
- B7: BAT-MAG-036 (summary, source), ledger BAT-QUIRK-5, §7.18; map 0x5A8523.
- B8: BAT-MAG-021, BAT-MAG-050.
- B9: BAT-ATK-004 (with the coordinator's sentence: the return is a part of the strike's action,
  the battle's end is checked once, after it), BAT-MAG-028, 033, 045, §7.15, §7.20; map 0x445950,
  0x478900.
- B10: BAT-MAG-038 (the optional clause).

## Part 2: the review's doc findings

Done, commit `c7b0e26` (pushed), except F-r4a-7, which went in with list B (`972e9d1`).

- F-r1-7: BAT-MAG-004 "(hero level div the stack's creature level, which is at least 1; the
  quotient may be 0)".
- F-r1-8: BAT-MAG-001 and BAT-MAG-005 state the delivered rules (no "join with T8", no "other
  effects with their tasks"; the Sphere's sentence points to BAT-MAG-063).
- F-r1-9: BAT-MAG-001's observation no longer claims the limit of one cast per round (R6 tried
  no second cast); a new code-reading source carries it: the cast mark +0x54B4 (set 0x59FFF6,
  tested 0x59E948 and 0x59EC76, cleared 0x4760EE).
- F-r1-11: BAT-MAG-006's VCMI source names the three differences and that they are not followed
  (BAT-Q28). §7.8 gets its sentence with part 3's paragraph on the chain.
- F-r2a-5: BAT-MAG-014's VCMI source: VCMI agrees (`DamageCalculator.cpp` L156-L186).
- F-r2b-stack-spells-4: BAT-MAG-019's VCMI source (`UNTIL_ATTACK`, `timed.json` L1112-L1161; not
  followed: BAT-Q28) and one clause in §7.11.
- F-r2b-stack-spells-5: BAT-MAG-018's and the ledger's Q-0010's VCMI file
  (`server/battles/BattleActionProcessor.cpp` L1603-L1616; the Fire Shield's damage base at
  L1183-L1206); `quirks` on BAT-MAG-018 (Q-0010, Q-0005), 019 (Q-0009, Q-0005), 022
  (BAT-QUIRK-9); a VCMI source on BAT-MAG-024 (Blind).
- F-r3-control-battlefield-9: the ledger's BAT-QUIRK-1 ("intended behaviour", sources, one dated
  review sentence in "Decided by"), §7.16, BAT-MAG-031's VCMI source (`battleHasNativeStack`
  counts dead natives, `CBattleInfoEssentials.cpp` L107-L126). BAT-MAG-026's VCMI source went in
  with list A.
- F-r4a-7: done with list B (`972e9d1`): BAT-MAG-036 (summary, both sources), §7.18, map
  0x59F700 and new 0x5A2D00, 0x5A2E90. I read both cursors: as the report says.
- F-r4b-strike-magic-7: BAT-MAG-040 names the Zombies' Disease; BAT-MAG-042's last sentence
  points to the Sphere's rule (BAT-MAG-063); BAT-MAG-043 "shots too"; BAT-MAG-041's bolt of 0
  went in with A9; two new ledger entries, **BAT-QUIRK-20** (the bound Harpy, BAT-Q9) and
  **BAT-QUIRK-21** (the binder on a stack its strike killed, BAT-Q8), class `exploit-fix`, no
  flag, and their ids in the `quirks` of BAT-ATK-004 and BAT-MAG-045.
- F-r4b-strike-magic-5: a VCMI source on each of BAT-MAG-040 to 047 (046: VCMI agrees), each
  with the difference and "not followed, the code reading decides: BAT-Q28"; §7.20 has a closing
  paragraph "The other sources" with the ten points.
- F-r5a-5: the VCMI sources of BAT-MAG-048 (three differences), 049 (the Faerie Dragon's own
  block in `neutral.json`; `RANDOM_SPELLCASTER` is the Genie's), 050 (new: four differences), 051
  (new: agrees), 053 (six `ENCHANTER` spells); §7.21 has a closing paragraph "The other sources
  for the casting creatures".
- F-r5b-mana-defences-6: the VCMI sources of BAT-MAG-055 (the gain computed before the effect),
  059 (new: three differences), 060 (new: two differences).
- F-r6-limits-opening-3: the VCMI sources of BAT-MAG-063, 064 and 066 say that VCMI's negation
  lifts the Pendants', Armageddon's Blade's and the Dragon Father's immunities (BAT-Q28); §7.23
  has it as a sixth point.
- F-r6-limits-opening-4: BAT-MAG-064's batch 2e observation and §7.23: the bolt is the strike's
  Lightning Bolt, level 2; for level 0 only Poison counts (0 of 4 at 30 %: 24 %).
- F-r6-limits-opening-5 (the three text lines): `quirks = ["Q-0005"]` on BAT-MAG-069 and
  HER-SPL-004; Eagle Eye in the ledger's Q-0005 (rules, flag text, regression test).
- F-r6-limits-opening-2, its doc part ("the split is DOCS"): BAT-MAG-066's batch 2e observation
  is now two sources, OV `high` and UA `low` (12 rolls, no control; BAT-Q27, BAT-Q28).
- "Left over from covered findings" (digest part 4): every rule text, ledger, §7.x and map
  correction of the two notes files is in with part 1. Not mine: F-r2b-stack-spells-1's "tell
  the user" (see "For the coordinator").

## Part 3: the readings the engine does not follow yet

(not started)

## Proposals changed (the code won)

None so far where the code contradicted a proposal. Wording changed for other reasons:

- No report ids in rule summaries ("r4a F-3", "review r1 F-2"): the reports are not merged.
- A6, BAT-END-001: "there the other side still acts once before the end" became "there the
  battle ends at the latest after the other side's next action" (r4a F-3 says the battle loop asks
  for the winner too, read only around the call).
- B5, BAT-DMG-006: "VCMI's ... is the same number" got "for a defense above 0" (VCMI's formula
  differs for a negative defense; the engine mirrors the sign, its own choice).
- B8, BAT-MAG-021: I left out "a mass version passes over one": Anti-Magic has no mass version.

## VCMI citations that did not hold

- r3 F-1 says "VCMI 1.7.5 agrees" with the relations of A10. It agrees on who may attack and
  shoot whom (`battleMatchOwner`), the automatic Ballista and tent (`unitOwner`), the Ammo Cart and
  a hypnotized stack's aura. It does **not** agree on three relations the engine now has from the
  code reading (0x442880, 0x43EA70), which no report names:
  - a hypnotized shooter: VCMI's `battleIsUnitBlocked` (`CBattleInfoCallback.cpp` L2143-L2154)
    compares initial owners, so the stacks of the side that controls it block it; the reading and
    the engine: its own side's stacks block it;
  - multi-headed strikes: `getAttackedCreatures` (L1895-L1946) compares both owners through
    hypnosis, so a Hydra spares a stack its own side hypnotized; the reading and the engine: it
    strikes it; and VCMI has no "either is berserk" term there (in the blocking test only for a
    berserk shooter itself);
  - hypnotized Unicorns: `CStack::magicResistance` (`CStack.cpp` L92-L108) compares initial
    owners, so they go on protecting their own side; the reading and the engine: the side that
    controls them.
  Written into the VCMI sources of BAT-MAG-025, 026, BAT-RNG-001, BAT-ATK-003 and §7.14 as "not
  followed, the code reading decides". **No question id exists for these three** (`docs/04` §2
  would make each a conflict of sources): for the coordinator.

- Part 2: every citation of the reports held in substance; the line ranges are as I found them
  in 1.7.5 (r2b F-5: L1603-L1616, not L1602; r3 F-9: `battleHasNativeStack` is L107-L126; r1
  F-11: `BattleSpellMechanics.cpp` L454-L464; r5a F-5: the Faerie Dragon's block is
  `config/creatures/neutral.json` L192-L292, the Enchanter's spells L976-L1021). One point the
  reports do not have: VCMI's Blind (`timed.json` L1390-L1465) names no reduced retaliation for
  the level without mastery, where SPTRAITS.TXT has 50 as at Basic (written into BAT-MAG-024's
  new VCMI source as "not followed, the original's table decides"; no question id).

## Rules whose `verified` status is now doubtful

Each is `verified` and now has a clause that no observation covers (the status is unchanged):

- BAT-MAG-004: the caster's products stored in single precision under Q-0005 (58 against 57 for
  a Sorcery specialist): a code reading only, never observed.
- BAT-MAG-010 and BAT-MAG-035: the permanent effects (Disrupting Ray, Acid Breath, Poison, Age,
  Bind) kept through death and a raise: the reading only.
- BAT-MAG-011: morale and luck clamped once, at the total: the reading and VCMI, no observation.
- BAT-MAG-026: the relations beyond batch 2c H1's remark (who blocks a hypnotized shooter, the
  multi-headed strikes, hypnotized Unicorns): the reading only, and VCMI contradicts three of
  them (see "VCMI citations that did not hold").
- BAT-MAG-006: `verified` with "Batch 2a confirmed ... the tie order": R2's tie is between two
  stacks of one side, so the order between the sides is not observed, and the review's re-read
  has the attacker's stacks first (part 3, BAT-Q28).
- BAT-MAG-066: the aura's clause rests on row UA alone (now `low`) against the reading.
- BAT-ATK-004 is `implemented`, not `verified`: nothing to doubt, but its new clauses (the
  return is a move; the battle's end tested once after it) are from the reading alone.

## Could not place / for the coordinator

- **No question id** for the three relations of fix A10 where VCMI contradicts the reading (a
  hypnotized shooter's blockers, multi-headed strikes at a stack one's own side hypnotized,
  hypnotized Unicorns' aura); nor for Berserk's "either is berserk" term in the hostile test.
  The VCMI sources of BAT-MAG-025, 026, BAT-RNG-001 and BAT-ATK-003 say "not followed, the code
  reading decides" without an id.
- **New ledger ids** BAT-QUIRK-20 and BAT-QUIRK-21 (the brief asks for the two entries; the ids
  are the next free ones). BAT-QUIRK-21 has no test of its own second half: nothing raises a
  stack that a binder's strike killed and binds it again (a test gap for the queue).
- F-r2b-stack-spells-1 asks to **tell the user**: the decision of 2026-10-07 on Q19 (the
  Behemoths) was taken on the misreading. Not a doc of mine.
- BAT-QUIRK-9 is in BAT-MAG-022's `quirks` now (F-r2b-5 asks for it; the digest says it could
  wait for F-r5a-1). If BAT-Q29 retires the quirk, the field goes with it.
- BAT-QUIRK-1 has a dated review sentence: at the user's decision (BAT-Q1) the entry described
  VCMI as testing living natives; does the class and the strict-fixed variant stand (r3 F-9)?
- The ledger's Q-0005 now says that Advanced Eagle Eye with the Emblem of Cognizance gives 64 %
  (exact 65 %) "which no test pins yet" (r6 F-5's test gap: for the queue).
- Worker `c`'s three texts (r1 F-3: BAT-MAG-003; r3 F-8: BAT-MAG-032; r4a F-4: BAT-MAG-037 or
  038 and BAT-QUIRK-11) are untouched, as the brief says.
- Named by the fix notes and not mine: the HANDOFF's trap texts (B2, A1), the survey's line on
  the chain's order, the E04b log's lines.
