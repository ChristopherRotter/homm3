# Docs notes of worker `q` (E04b T11b, docs list Q)

Worktree `/home/claude/wt-q`, branch `battle/e04b-t11b-q` (base `64fe5cf`), pushed. Files: the
battle block under "Open" in `docs/OPEN-QUESTIONS.md`, `docs/experiments/README.md`,
`docs/rules/surveys/E04b-magic.md`, the inbox of `docs/lanes/battle/QUEUE.md`. No other file.
Checks: `check-docs` ok and `check-rules` ok (183 rules) at the base and before each commit
(logs: `<SCRATCH>/wq/check-*.log`).

## State: all four done

| file | what | bytes added | commit |
|---|---|---|---|
| `docs/OPEN-QUESTIONS.md` | BAT-Q28 to BAT-Q33 in the battle block under "Open" | 13 850 (limit 14 000) | `4c14cf9`; wording in `f3996ae`, `0d80034`, `aabe205` |
| `docs/rules/surveys/E04b-magic.md` | §16, the review's quirk candidates S-23 to S-27 | 6 604 | `b0e4329` |
| `docs/experiments/README.md` | "From the E04b review" (candidates by set-up), "Wiki checks the review asks for" | 10 263 | `9447020`; the reading's outcome per row in `0d80034` |
| `docs/lanes/battle/QUEUE.md` | inbox entry "2026-10-12 (E04b T11b, the review)", after the T11a triage entry | 5 141 | `f3996ae` |

Entry sizes in "Open": BAT-Q28 6 815 (the brief: about 6 000; it carries the finding ids, about
250 bytes, and the whole "on record" list, about 1 400: both could be cut if the size matters
more), BAT-Q29 1 468, BAT-Q30 1 282, BAT-Q31 1 387, BAT-Q32 1 430, BAT-Q33 1 489. Branch head:
`aabe205`, pushed; the worktree is clean.

## Where each finding of the list went

- BAT-Q28 rows 1 to 12: r1 F-1; r1 F-12 = r5a F-3; r2a F-4 = r4b F-4; r2b F-2 (survey S-23); r3
  F-5; r4a F-2; r4a F-6 with F-9; r5b F-4; r5b F-5; r4a F-5 (survey S-24); r1 F-6 with r6 F-2;
  r2a F-6. "Row 13" (the paragraph "On record, no decision now"): r4b F-5, r1 F-11, r2b F-4,
  r5a F-5, r5b F-6, r6 F-3.
- BAT-Q29: r5a F-1. BAT-Q30: r3 F-4. BAT-Q31: r3 F-11 = F-sec-1 part a. BAT-Q32: r5a F-2 and
  F-4. BAT-Q33: r5b F-3 (survey S-25).
- Survey S-26 (an obstacle removed early is removed again; r3, "left open"), S-27 (the wait
  phase's soft-lock; r5b, "left open"). The S ids continue `E04-quirks.md` (highest in use: S-22).
- QUEUE: E04c (the reports' siege and special-battlefield points), "Battle, no task yet" (batch
  2g; the eleven behaviour points; the three leftovers of the fixes), "For the server lane" (the
  cast-target query, the sample scenario's profile, the `Bot` trait, F-sec-5's stream).
- Experiments README: every row of BAT-Q28 that has an experiment, BAT-Q29, BAT-Q31 to BAT-Q33,
  S-23 to S-27, the statistics rows, the Orb of Vulnerability rows (with r6 F-3's cursor check);
  the 49 merged wiki checks, each with its rule or question.

## Choices inside the brief's frame

- Finding ids are in the short form of the fix commits ("r4a F-5" for F-r4a-5; "F-sec-1",
  "F-robust-3" as the reports have them), in brackets after the point.
- BAT-Q28 cites `installation.md` §7 without a § per row (worker `d` had not written its
  "Review re-read" paragraphs when this was written). The single-topic entries cite the § where
  the topic is described at `64fe5cf`: BAT-Q29 §7.21, BAT-Q30 §7.16, BAT-Q31 §7.16, BAT-Q32 §7.21
  and §7.23, BAT-Q33 §7.21. **To reconcile with `d`'s placements after the join.**
- BAT-Q28's paragraph "On record, no decision now" calls itself row 13, so that the other docs
  can cite it.
- The suggested classes in the survey: S-23 fix-optional (the reviewer's; the class is the
  user's, BAT-Q28 row 4), S-24 fix-optional (mine: unintended and mild), S-25 "none fits; as a
  quirk, preserve" (mine: the second roll looks deliberate; it hangs on BAT-Q33's seam), S-26 and
  S-27 exploit-fix (the reports').
- BAT-Q32 has an option (b) the brief does not name (the wider targets now, as the reading and
  VCMI agree; the Orb after the session); the recommendation is the brief's.
- BAT-Q33's option (b) names no ADR (the field would be a `BattleSetup` contract change), and
  the entry asks what a self-play agent's side counts as under (b) or (c).
- VCMI citations the entries rest on were opened in `/home/claude/vcmi` (1.7.5) and hold:
  `BattleSpellMechanics.cpp` L520 (only `Mode::HERO` is reflected), `CUnitState.cpp` L423-427 (no
  spell bonus for a unit), `server/battles/BattleActionProcessor.cpp` L326 (the counterattack needs
  `stack->alive()`), L759-763 (start hexes passed; `oneHexTriggersObstacles` false at
  `gameConfig.json` L614, the report says 613), `BattleInfo.cpp` L660-666 (`afterNewRound` for
  every stack), `Unit.cpp` L238-245 (`creatureCost() * getCount()`),
  `BattleActionProcessor.cpp` L167-181 (Defend's +20 % `PERCENT_TO_ALL`),
  `BattleFlowProcessor.cpp` L378-398 (bad morale, no branch by player).
- The engine's side of S-27 was checked in `turn.rs` (`activate` sets `STARTED` and passes a
  `NO_TURNS` stack in every phase).

## Added beyond the brief's lists (say if they should go)

- QUEUE: r6's two notes on `installation.md` §7.23 (the worn-book test of 0x41F890 holds for
  every hero cast: under E04c; Armageddon's Blade's test is listed before the polarity tests:
  a note under "Battle, no task yet"); the `Bot` trait (F-sec-2's "same root") under the server
  lane; the remark that two points of the T11a entry are answered (the surrender price's code,
  the attack from a seen pit).
- Experiments README: a Berserk row for BAT-Q31 (the reading stands alone, so §2 makes it a
  candidate), the Frenzy row (r2b F-4: only if the wiki is silent), the two rows on battles that
  may crash or hang for S-26 and S-27, the Dwarves row for the chain's rolls (r1 F-11).
- BAT-Q28's list says that BAT-Q21's "agreed by all" is wrong for the artifact immunities under
  the Orb of Vulnerability (r6 F-3); the answered entry itself is outside my block.

## Could not place / for the coordinator

1. **r3 F-9's one-line question to the user** (does BAT-QUIRK-1's class and strict-fixed variant
   stand, now that VCMI counts dead natives too and the wiki's "started the battle with" fits the
   original): the brief names no question for it. It is on record only as a clause of the wiki
   check for BAT-QUIRK-1 in the experiments README. The texts are `d`'s.
2. **r5a F-6 = r6 F-9** (`docs/04` §4 and exit criterion 3 against the four flagless quirks
   BAT-QUIRK-10, 11, 15, 18): a sentence in a shared path or in the epic file; none of my files.
3. **F-sec-5's line under "Not covered yet" in `docs/06-security.md`** (a shared path): named as
   the alternative to the keyed stream in the QUEUE's server-lane line, not written.
4. **r2b F-1's "tell the user"** (the decision of 2026-10-07 on Q19 was taken on the misreading;
   the finding is covered by B5): nothing in my files.
5. **The answered entry BAT-Q21** (outside my block): "agreed by all" for the artifact immunities
   (r6 F-3), and point 5's evidence for level 0 (r6 F-4: 4 Poison attacks, 24 %) are unchanged.
6. **The digest's "Not read or not run" list** (part 3: the spell book's dialog 0x59BDF0 and its
   cost test, why the tactics phase allows no cast, the 44 x 42 grid and Q43, helper functions
   taken as the map describes them, the encodings of the raised and created states, the computer
   player's paths, the premises of BAT-QUIRK-13 and 14, the byte +0x13D74, +0xEA at 0x5A829B),
   **[sec]** the alignment of `out` in `h3_env_observe` and the unverified claim in
   `event.rs:312-314`, **[robust]** the two "not findings" (many rounds inside one command; the
   effect and obstacle lists outgrowing their reserved sizes): they describe the review's limits,
   the brief's lists do not name them, and I wrote no registry entry for them (the epic's log?).
7. **Tests** (not mine): the ten TEST findings; r4a F-3's proposed property invariant (not in
   A6); the robustness report's two property tests on the unpushed branch `review/robustness`.
   BAT-Q31 says that the hidden-obstacle property leaves Berserk out and names the entry, as the
   brief has it: that test must exist for the sentence to hold.
8. **`fix-notes-b` B2** names `E04b-magic.md` line 235 ("A stack that dies loses its effects.") as
   history only: unchanged (my brief allows only the new last section).
9. **The T11a inbox entry** still lists "the surrender price's code (unread ...)" and "whether the
   interface offers an attack from a seen pit": my entry says both are answered; I did not edit
   the older entry.
