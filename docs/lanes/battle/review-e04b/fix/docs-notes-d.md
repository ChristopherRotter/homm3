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

---

# Second session of worker `d` (after the restart of 2026-10-12, 18:00): Part 3

The section "Part 3 ... (not started)" above is the first session's state. This session did Part 3
and the four additions of the takeover brief. Same worktree and branch; start head `c7b0e26`.

## Part 3: state

DONE and pushed (branch `battle/e04b-t11b-d`, head `fc0ad81`, worktree clean):

- `bf3cd64`: addition 2 (BAT-Q28 on the four VCMI sources and in §7.14; two VCMI line ranges).
- `fc0ad81`: the 18 points' re-reads (`installation.md`, the map, the structs, the ledger).

The three checks passed before each commit, on that commit's own state (`check-rules` ok, 183
rules; `check-docs` ok; `check-exe-map` ok, 355 entries after `fc0ad81`). Logs:
`<SCRATCH>/wd/p3-check-*.log`.

Edits applied in the worktree (scripts in `<SCRATCH>/wd/edits/p3_*.py`, applied with `sub.py` and
`mapadd.py`):

- `docs/formats/installation.md`: 19 dated paragraphs "**Review re-read (E04b T11b, 2026-10-12):
  ...**" for the 18 points (the Faerie Dragon's key path has two: its targets, and no "magic
  allowed" test), each with the addresses, what was not read or is unsure, the other sources and
  the closing clause with the question id. Old sentences that the re-reads contradict got a short
  pointer in place instead of a silent rewrite (§7.8 the chain's order; §7.11 "was not read";
  §7.13 where the redirect is drawn; §7.14 the pick; §7.16 "leaves it freely"; §7.18 "stays done";
  §7.19 "(not read)"; §7.20 Acid Breath; §7.21 "no hero bonuses", "local human", "the creature
  caster's button", "never casts at an empty hex", the dragon's own mirror; §7.23 "every cast
  there is", "found no path around 0x41F890", batch 2e's OI row). §7.21's *Magic Mirror* sentence
  of the Faerie Dragon paragraph is rewritten (the brief asks for it): the redirect is drawn and
  dropped; the engine still redirects (BAT-Q29).
- `docs/exe/map.toml`: 21 summaries updated (0x5A61F0, 0x5A02CB, 0x5A098A, 0x59EE7A, 0x59EC50,
  0x448260, 0x464B40 and 0x464F50 now `read`, 0x478D80, 0x4639F0, 0x474D80, 0x465080, 0x446E30,
  0x447120, 0x441CB0, 0x5A1907, 0x441FA0, 0x4B3400, 0x5A4970, 0x5A6360, 0x5A4BC0, 0x4222C0); 6
  new entries: 0x41F580 `walk_towards_hex` (inferred, read at its head), 0x474690
  `cursor_attack_slots`, 0x474A00 `cursor_attack_from_hex`, 0x4782D0 `battle_controls_state`
  (inferred, read in part), 0x4CE940 `player_id_is_human` (H3API `H3Main::IsHuman`), 0x4E4580
  `hero_surrender_factor` (H3API `H3Hero::GetDiplomacyPower`; read whole). `scripts/exe.py check`
  and `cited` pass (650 cited addresses, all covered).
- `docs/exe/structs.toml`: stack +0xF8 `record`, +0x4DC `defend_bonus`, +0x4E8 `morale`, the
  defending bit `0x8000000` in the flags' summary; manager +0x3C to +0x48 (the action and its
  hexes), +0x547C `spell_marks`, +0x54B4 `hero_cast_mark`, +0x132B8/+0x132BC (the stack activated
  last); +0x54A4 corrected (it is `u8[2]`, the player's human flag, not `i32[2]` "local human";
  r5b F-3 asks for it) and +0x54A6 added (inferred). The two aura lists' summaries now say which is
  which (a leftover of fix A10: the old text had "of the same controller").
- `docs/rules/quirks-ledger.md`: one dated "Review (E04b T11b, 2026-10-12)" line each on
  BAT-QUIRK-2 (BAT-Q30), BAT-QUIRK-4 (the berserk walker, BAT-Q31) and BAT-QUIRK-9 (BAT-Q29); the
  entries themselves are untouched. BAT-QUIRK-20 and 21 are the first session's.
- `docs/rules/catalog/battle.toml` (addition 2): "the code reading decides (BAT-Q28; ...)" on the
  VCMI sources of BAT-MAG-025, BAT-MAG-026 (the second one), BAT-RNG-001 and BAT-ATK-003; §7.14's
  closing sentence names BAT-Q28 and the four rules. Two VCMI line ranges of the first session
  corrected in BAT-MAG-049's VCMI source (see "VCMI citations", below). No other catalog change:
  the engine is unchanged on all 18 points.

## Every point was opened in the executable (addition 3)

For each of the 18 points I read the deciding addresses with the tool (whole short routines:
0x5A61F0, 0x464F50, 0x4B3290, 0x465080, 0x4E4580, 0x4CE940, 0x4CE970; the deciding parts of the
long ones). The reports' readings hold on all 18. Where my reading adds to a report or words it
differently:

1. **F-r3-5 (two-hex walker).** The report says the walker "tests the new head hex and the new tail
   hex" and "for a step along the row the new tail hex is the old head hex". What the code tests is
   the hex the stack's position (+0x38) moves to and that position's other hex (± 1 by the facing).
   A stack is stopped again only when it walks along its row *with the pit hex leading* (the hex it
   keeps covering is the pit's); walking the other way along the row it is free at once. The
   report's scenario (front hex on the pit, ordered forward) is the stopping case. BAT-Q28 row 5
   and the experiment should say "ordered forward" (the front hex on the pit).
2. **F-r5b-4 (wait phase).** The report did not read the dispatcher. I read the head of its Wait
   case (action 8, 0x479906): no test of the phase; it sets the waited bit, and in the wait phase
   nothing clears that bit before the round ends (the pick's second pass skips waited records and
   then returns "nobody"), so a Wait arriving there would cost the stack its turn, not give it a
   second one. The button's state (0x478755) is as the report says. Also read: the pick runs no
   turn start in the wait phase (0x465211), which the report only "met on the way"; the comparison
   puts arrow towers and the Catapult before the speeds.
3. **F-r2a-4 = F-r4b-4 (Defend).** Added: in the tactics phase and for a war machine the stored
   bonus is 0 and the defending bit is not set (0x479722 to 0x479735, 0x4798C3). r4b lists a war
   machine's Defend bonus under "not reviewed".
4. **F-r5b-3 (bad morale).** The flag's meaning does not rest on H3API alone: the adventure lane's
   own reading (map entry 0x4BADA0, survey E08 §1.4) has the player record's +0xE2 as the human
   flag. H3API calls +0x54A4 `isNotAI` and +0x54A6 `isHuman` and both record bytes (+0xE1, +0xE2)
   human flags; "+0x54A6 is the local human" is the report's guess. BAT-Q33's "the flag's meaning
   is unsure" can be narrowed to that.
5. **F-r5a-1 (mirror).** Added: 0x59EC50 resets the manager's +0x48 to −1 at every entry
   (0x59ECB0), so a redirect drawn and dropped on the key path cannot leak into a later cast.
6. **F-r5a-2.** The existing §7.21 and the map called 0x475CD8 "the creature caster's button"; it
   is the key handler's case for key code 33 (switch 0x4758FB), as the report says. Corrected in
   both. The surrender price and the Wait button are cases of the same handler's control switch
   (0x474E53: controls 2001 and 2009).
7. **F-r3-11.** The brief's closing clause does not fit: by the reading the engine does what the
   original does (it plans with the stack's own side's pits). The paragraph says so. One corner I
   read further than the reports and could not close: the pathfinder's attack marking passes over a
   position on a stop only for a query within one move (the cursor's; r3 F-4 says "when the range
   is within one turn"). The berserk turn asks with range 127 (0x445582, 0x42232C), and there a
   position on a stop does count, one whole move dearer (0x4B378A); the berserk walk attacks only
   if the target's cost is within the stack's speed (0x4223E1), else calls 0x41F580 ("walk
   towards", read only at its head; new map entry, inferred). So a berserk stack whose attack
   position lies on a seen pit does not strike that turn; whether it ends its walk on the pit, as
   the engine's does with BAT-QUIRK-2 on, is not known. BAT-Q30's "It also takes the sharper form
   out of BAT-Q31" and BAT-Q31's "(with BAT-QUIRK-2 on it may stop in the pit: the exact hex)"
   describe the engine; the original may do the same there.
8. **F-r6-2.** Closing clause: "the engine does not follow this reading" (the user's decision
   BAT-Q27 stands), not "yet".
9. **F-r2a-6.** The test wants a duration that is *not 0* (the report: "above 0"); no difference in
   play.

Read by the reports and not re-read by me (the paragraphs say "by the report"): the computer
player's attack query (r3 F-4); the single search of the whole code for the acted bit's masks (r4a
F-2); 0x444297 for "position = the rear hex" (r4a F-5); Ice Bolt's, Lightning Bolt's, Implosion's,
Death Ripple's and Destroy Undead's hero argument (r1 F-12; I read Magic Arrow's); the click
handler 0x476C99 (r5a F-1; not cited, it would need a map entry); "no other test of artifact 126 on
the key path" (r5a F-4; I checked the callers of 0x41F890 and that the three routines push no 126);
good morale's and Fear's lack of a human branch (r5b F-3); the two places that test artifact 93
(r6 F-2).

## Report details against the question entries (addition 1)

Read in `/home/claude/wt-q/docs/OPEN-QUESTIONS.md` (head `aabe205`). No entry contradicts a
report. Points where an entry could be sharper:

- BAT-Q28 row 5: see 1 above ("ordered along the row" should be "ordered forward, its front hex on
  the pit").
- BAT-Q28 row 8: "Wait is not offered" is the button's state; the dispatcher would accept a Wait
  and the stack would lose its turn (2 above).
- BAT-Q33: see 4 above.
- BAT-Q28 rows 4 and 10 cite surveys S-23 and S-24, BAT-Q33 S-25: my paragraphs do not cite them
  (they exist only in worker q's branch); they say "an oddity of the original whose class is the
  user's to decide".
- The § references of BAT-Q29 (§7.21), BAT-Q30 (§7.16), BAT-Q31 (§7.16), BAT-Q32 (§7.21, §7.23)
  and BAT-Q33 (§7.21) are right as they stand. BAT-Q28 names no §; the table at the end has them.

## VCMI citations (second session)

Every VCMI statement in the new paragraphs was opened in `/home/claude/vcmi` (1.7.5):
`doDefendAction` (BattleActionProcessor.cpp L160-L198), the retaliation's `stack->alive()`
(`doAttackAction`, L325-L334), `moveStack`'s start hexes (L758-L763) and `oneHexTriggersObstacles`
(gameConfig.json L613-L614), `nextRound`/`afterNewRound` (BattleInfo.cpp L651-L666, CUnitState.cpp
L913-L927), `getRawSurrenderCost` (Unit.cpp L238-L245), `getSpellBonus` (CUnitState.cpp L423-L427),
`isReflected` (BattleSpellMechanics.cpp L514-L523), `doUnitSpellAction` (BattleActionProcessor.cpp
L489-L546), `tryActivateMoralePenalty` (BattleFlowProcessor.cpp L378-L398), `makeAutomaticAction`
(L843 on), `getClosestTile` (UnitEffect.cpp L253). All hold.

Two line ranges of the first session in BAT-MAG-049's VCMI source did not: `getSpellBonus` is
L423-L427 (was L422-L426) and `isReflected` L514-L523 (was L506-L515, which is the end of the
function before it). Corrected.

## For the coordinator (second session)

- **BAT-Q28's "On record" list**: the three relations of fix A10 and Berserk's term now carry
  "BAT-Q28" in the four VCMI sources and in §7.14, as addition 2 asks; the entry itself is yours.
- **The catalog's code-reading sources are unchanged.** Their `ref` texts still state what the
  earlier readings said, and the summaries state the engine's rule; the question entries carry the
  conflict. Where a source's wording is now contradicted by a re-read in the § it cites:
  BAT-MAG-006 ("defender's stacks first"), BAT-MAG-049 and BAT-MAG-022 and BAT-MAG-057 (the
  redirect reaches a creature's cast; "never at an empty hex"), BAT-MAG-048 ("no hero bonuses"),
  BAT-MAG-032 ("the hexes a stack starts on never stop it"), BAT-MAG-035 ("stays done" for the
  round only), BAT-MAG-066 (the aura), BAT-MOR-004 (its source names 0x464B40 with confidence high;
  the callee had not been read then), BAT-FLE-002 (no code-reading source at all for the price).
  If these should get a note each, say so; I left them because the brief keeps Part 3 to
  `installation.md`, the map and the ledger.
- **BAT-MAG-035 and BAT-FLE-002 are `verified`** and each now has a clause the review's reading
  contradicts (a stack raised in a later round acts; the price counts temporary creatures): add
  them to the first session's list of doubtful `verified` statuses.
- **Not in any report, read on the way** (queue candidates, nothing written into a rule): the
  Defend case gives a war machine no bonus and none in the tactics phase (does the engine?); a
  stack first picked in the wait phase gets no turn start, so a Defend bonus from the round before
  would stay inside its defense and it would not regenerate (r5b lists the missing turn start
  under "met on the way").
- The survey ids S-23 to S-25 (worker q) are not cited by `installation.md`.
- Paragraph length: the brief asks for short paragraphs; they run from 150 to 390 words, because
  each carries its addresses, the unsure parts and the other sources.

## The § of each point in `docs/formats/installation.md`

Each paragraph starts with "**Review re-read (E04b T11b, 2026-10-12): <title>.**"; the title is
given so that the paragraph can be found by a search.

| # | finding | question | § | the paragraph's title |
|---|---|---|---|---|
| 1 | F-r1-1 | BAT-Q28 | §7.8 | the chain's search order |
| 2 | F-r1-6 | BAT-Q28 | §7.8 | which side a chain candidate is tested for |
| 3 | F-r1-12 = F-r5a-3 | BAT-Q28 | §7.21 | the hero's damage bonuses in a creature's area spell and chain (a pointer in §7.8, "Area spells") |
| 4 | F-r2a-4 = F-r4b-strike-magic-4 | BAT-Q28 | §7.9 | the Defend bonus sits inside the defense (a pointer in §7.20, Acid Breath) |
| 5 | F-r2a-6 | BAT-Q28 | §7.10 | what counts as an active spell for Dispel's aim |
| 6 | F-r2b-stack-spells-2 | BAT-Q28 | §7.11 | the retaliation at a striker the reflection killed |
| 7 | F-r3-control-battlefield-5 | BAT-Q28 | §7.16 | a two-hex walker leaving a pit along its row |
| 8 | F-r4a-2 | BAT-Q28 | §7.18 | a dead stack's round state is frozen |
| 9 | F-r4a-5 | BAT-Q28 | §7.18 | the hero's raise spells search twice |
| 10 | F-r4a-6 and F-r4a-9 | BAT-Q28 | §7.19 | the surrender price |
| 11 | F-r5b-mana-defences-4 | BAT-Q28 | §7.21 | the order in the wait phase (a pointer in §7.14, Blind's "No turns") |
| 12 | F-r5b-mana-defences-5 | BAT-Q28 | §7.21 | a freeze and the side that acted last |
| 13 | F-r6-limits-opening-2 | BAT-Q28 | §7.23 | the aura under the Orb of Vulnerability, a second reading |
| 14 | F-r5a-1 | BAT-Q29 | §7.21 | Magic Mirror never redirects a creature's spell (a pointer in §7.13) |
| 15 | F-r3-control-battlefield-4 | BAT-Q30 | §7.16 | no attack is offered from a seen pit |
| 16 | F-r3-control-battlefield-11 = F-sec-1 part a | BAT-Q31 | §7.16 | a berserk stack's turn and its own side's pits |
| 17 | F-r5a-2 and F-r5a-4 | BAT-Q32 | §7.21 | the Faerie Dragon's key path and its targets (F-2); the key path asks no "magic allowed" test (F-4); pointers in §7.23 |
| 18 | F-r5b-mana-defences-3 | BAT-Q33 | §7.21 | bad morale and who commands the side |
