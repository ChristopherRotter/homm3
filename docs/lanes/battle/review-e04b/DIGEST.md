# E04b review: digest of the findings the fix lists do not cover

Eleven reports, 87 findings: 32 covered in full by `FIX-a.md` / `FIX-b.md`, F-sec-1 covered in part
(part b), 54 open: 55 open entries below. Suggested: ASK-USER 19, DOCS 17, TEST 10, DEFER 5, FIX-NOW 4.

Legend. "exe": the reviewer's own re-read of the original executable at the addresses given (the
deciding ones; the reports list more). §7.x: `docs/formats/installation.md`. "conf.", "sev.": the
reviewer's confidence and severity. "n/a": the original is not concerned. A1..A11: `FIX-a.md`;
B1..B10: `FIX-b.md`. Nothing here was checked against the code or the executable again.
To decide quickly: the table, then the 19 ASK-USER and 4 FIX-NOW entries; the DOCS and TEST entries are short.

## 1. Coverage table

| id | title | covered by, or open: my suggestion |
|---|---|---|
| F-r1-1 | Chain Lightning searches the attacker's stacks first | open: ASK-USER |
| F-r1-2 | Q-0005 spell damage: `f32` stores before truncating | B6 |
| F-r1-3 | A no-target spell that would affect nobody is refused | open: FIX-NOW |
| F-r1-4 | Unicorn aura: Unicorn's controller against the stack's real side | A10 |
| F-r1-5 | `spell_damage` can overflow `u128` under Q-0005 | B6 |
| F-r1-6 | Chain tests later candidates as cast by the opposite side | open: DEFER |
| F-r1-7 | BAT-MAG-004's "at least 1" reads as a quotient of at least 1 | open: DOCS |
| F-r1-8 | Two summaries describe delivered work as still to come | open: DOCS |
| F-r1-9 | "Once per round" cited from batch 2a R6, which never tries it | open: DOCS |
| F-r1-10 | Small test gaps (mana equal to cost, harvested replays) | open: TEST |
| F-r1-11 | VCMI differs from the chain rule in three points, unrecorded | open: DOCS |
| F-r1-12 | A creature's area spell or chain takes the hero's damage bonuses | open: ASK-USER |
| F-r2a-1 | Morale and luck spells added after a ±3 clamp the original lacks | B4 |
| F-r2a-2 | Expert Dispel passes over hypnotized stacks under Q-0012 | A11 |
| F-r2a-3 | A dead stack loses Disrupting Ray and Acid Breath | B2 |
| F-r2a-4 | Defend bonus recomputed at each strike; the original fixes it | open: ASK-USER |
| F-r2a-5 | BAT-MAG-014's VCMI note is wrong (VCMI agrees) | open: DOCS |
| F-r2a-6 | Dispel below Expert can be aimed at a stack with only Acid Breath | open: DEFER |
| F-r2b-stack-spells-1 | Q-0005: the Behemoths' reduction truncates the defense left | B5 |
| F-r2b-stack-spells-2 | Retaliation at a striker killed by Fire Shield's reflection | open: ASK-USER |
| F-r2b-stack-spells-3 | A Master Genie's draw can put Anti-Magic on a clone | B8 |
| F-r2b-stack-spells-4 | BAT-MAG-019's VCMI source misquoted (Frenzy's end) | open: DOCS |
| F-r2b-stack-spells-5 | Wrong VCMI file, missing `quirks` fields, no VCMI source on Blind | open: DOCS |
| F-r2b-stack-spells-6 | Five clauses no test pins | open: TEST; item 1 comes with B5 |
| F-r2b-stack-spells-7 | Slayer's `i32` addition (point 1); a permanent turn-stopping spell (point 2) | A3, A4 |
| F-r3-control-battlefield-1 | A hypnotized stack stays its owner's to other stacks | A10 |
| F-r3-control-battlefield-2 | A berserk stack cannot be attacked by its own side | A10 |
| F-r3-control-battlefield-3 | Expert Dispel reaches hypnotized stacks | A11 |
| F-r3-control-battlefield-4 | BAT-QUIRK-2: the original never offers an attack from a seen pit | open: ASK-USER |
| F-r3-control-battlefield-5 | A two-hex walker leaving a pit along its row is stopped again | open: ASK-USER |
| F-r3-control-battlefield-6 | The strike-and-return flight enters no hex | B9 |
| F-r3-control-battlefield-7 | Remove Obstacle is listed castable with no target | A2 |
| F-r3-control-battlefield-8 | Pit and mine draws avoid dead stacks that left no corpse | open: FIX-NOW |
| F-r3-control-battlefield-9 | BAT-QUIRK-1 and BAT-MAG-031 misdescribe VCMI | open: ASK-USER |
| F-r3-control-battlefield-10 | No harvested replay of batch 2c; two boundaries unpinned | open: TEST |
| F-r3-control-battlefield-11 | A berserk stack's turn follows its side's hidden pits | open: ASK-USER |
| F-r4a-1 | BAT-QUIRK-5: Sacrifice's raised stack must pass Sacrifice's own test | B7 |
| F-r4a-2 | A dead stack's round state is frozen until raised and a new round | open: ASK-USER |
| F-r4a-3 | BAT-QUIRK-7: a side's last stack vanishes at a round start, no end | A6 |
| F-r4a-4 | An expired created stack's record is never reused | open: FIX-NOW |
| F-r4a-5 | A raise aimed at a two-hex corpse raises its position hex's top corpse | open: FIX-NOW |
| F-r4a-6 | Surrender price counts creatures raised for the battle only | open: ASK-USER |
| F-r4a-7 | Sacrifice's victim test is the original's, not an engine choice | open: DOCS |
| F-r4a-8 | A clone of a raised stack keeps the trait RAISED | B10 |
| F-r4a-9 | Surrender price halves before the discount (1 gold) | open: DEFER |
| F-r4a-10 | Test gaps (raise boundary, summon row, clone place, batch 2d numbers) | open: TEST |
| F-r4b-strike-magic-1 | A dead stack loses Acid Breath's defense loss | B2 |
| F-r4b-strike-magic-2 | A Phoenix that dies aged is reborn at half | B1 |
| F-r4b-strike-magic-3 | The Harpies' return does not enter its landing hexes | B9 |
| F-r4b-strike-magic-4 | Defend bonus inside the defense field (Acid Breath) | open: ASK-USER |
| F-r4b-strike-magic-5 | BAT-MAG-040..047 on the reading alone; VCMI contradicts ten points | open: ASK-USER |
| F-r4b-strike-magic-6 | Test gaps (Q-0005 in a battle, shot at petrified, Bind's expiry) | open: TEST |
| F-r4b-strike-magic-7 | Rule texts and two missing ledger entries | open: DOCS |
| F-r4b-strike-magic-8 part a | A bolt of 0 damage still emits events | A9 |
| F-r4b-strike-magic-8 part b | No `Poisoned` event when Age ends at the same round start | B3 |
| F-r5a-1 | Magic Mirror never redirects a creature's spell (BAT-QUIRK-9) | open: ASK-USER |
| F-r5a-2 | F key: the Faerie Dragon aims an area spell at any hex | open: ASK-USER |
| F-r5a-3 | A creature's area spell or chain takes the hero's damage bonuses | open: ASK-USER |
| F-r5a-4 | The F-key path has no "magic allowed" test (Orb of Inhibition) | open: DEFER |
| F-r5a-5 | VCMI misquoted in two entries; its contradictions unrecorded | open: DOCS |
| F-r5a-6 | BAT-QUIRK-10 is `preserve` without a flag | open: DOCS |
| F-r5a-7 | Two boundary clauses untested (Archangel `<=`, area rolls) | open: TEST |
| F-r5b-mana-defences-1 | A Phoenix that dies under Age is reborn at half | B1 |
| F-r5b-mana-defences-2 | A dead stack loses the permanent defense losses | B2 |
| F-r5b-mana-defences-3 | Bad morale: a side no human commands escapes one freeze in four | open: ASK-USER |
| F-r5b-mana-defences-4 | The wait phase orders every pending stack by the phase | open: ASK-USER |
| F-r5b-mana-defences-5 | A freeze counts as "its side acted last" in the engine only | open: ASK-USER |
| F-r5b-mana-defences-6 | BAT-MAG-059/060 cite the reading alone; VCMI differs in five points | open: DOCS |
| F-r6-limits-opening-1 | Unicorn aura by real side, not by controller | A10 |
| F-r6-limits-opening-2 | Aura under the Orb of Vulnerability: reading against a weak observation | open: DEFER |
| F-r6-limits-opening-3 | VCMI lets the Orb lift artifact immunities; catalog says it agrees | open: DOCS |
| F-r6-limits-opening-4 | Thunderbirds' bolt is no level-0 ability (batch 2e DF note) | open: DOCS |
| F-r6-limits-opening-5 | Q-0005 changes Eagle Eye's chance: not in `quirks`, ledger, tests | open: TEST |
| F-r6-limits-opening-6 | Two timing clauses of the opening spells untested | open: TEST |
| F-r6-limits-opening-7 | No test of a strike's casting side for the negation | open: TEST |
| F-r6-limits-opening-8 | Two comments still say the Faerie Dragon ignores the Orb of Inhibition | open: DOCS |
| F-r6-limits-opening-9 | Exit criterion 3 and `docs/04` §4 against four flagless quirks | open: DOCS |
| F-robust-1 | Automatic Ballista's target shows a hidden pit; the property fails in 1.2 % | A1 |
| F-robust-2 | BAT-QUIRK-3: Remove Obstacle castable with no target | A2 |
| F-robust-3 | `spell_damage` overflows `u128` under Q-0005 | B6 |
| F-robust-4 | A permanent turn-losing spell compiles; a command never returns | A4 |
| F-robust-5 | An obstacle type with more than eight blocked hexes compiles | A5 |
| F-robust-6 | Slayer's specialist bonus overflows `i32` | A3 |
| F-sec-1 part a | A berserk walker's turn shows its own side's hidden pits | open: ASK-USER |
| F-sec-1 part b | The automatic Ballista's target shows the other side's hidden pits | A1 |
| F-sec-2 | The sim's `step` tells the other side a Faerie Dragon's drawn spell | A8 |
| F-sec-3 | Tactics phase: Retreat and Surrender act for the active stack's side | A7 |
| F-sec-4 | Properties never play Berserk, Hypnotize, hidden hero magic, BAT-QUIRK-2 | open: TEST |
| F-sec-5 | Hidden draws share the public rolls' stream | open: DOCS |

## 2. Open findings

**Report r1-casting-core**

### F-r1-1: Chain Lightning searches the attacker's stacks first
- Class: BEHAVIOUR
- Original: exe 0x5A61F0 (0x5A6223–0x5A622F, 0x5A6328–0x5A6331): the attacker's side is searched first, the earlier of equals kept (0x5A62F7–0x5A62FA). Conf. high for the code, but alone against the wiki ("randomly chosen") and VCMI (a positional tie-break).
- Engine: `magic.rs:2106` `next_in_chain`: defender first.
- In play: sev. medium; a distance tie between a stack of each side: the two halves swap.
- Fix needs: swap the order; four doc places; a test; an experiment "should confirm it".
- Reviewer proposes: swap, docs, test.
- Suggested: ASK-USER: one reading against wiki and VCMI; the change is local.

### F-r1-3: A no-target spell that would affect nobody is refused
- Class: BEHAVIOUR
- Original: exe 0x59ED5A–0x59EE03: before anything is paid, some stack must have a factor above zero (0x5A39C0); hypnotized stacks count. Conf. high for the path, medium that nothing else gates the cast. No second source.
- Engine: `no_target` (`magic.rs:1645–1660`) asks `reachable` only, which drops hypnotized stacks.
- In play: sev. low; an enemy hero with 100 % resistance: the cast wastes mana and the round's cast.
- Fix needs: `no_target`: `any(in_reach && accepts)`; BAT-MAG-003's summary. Else not stated.
- Reviewer proposes: that.
- Suggested: FIX-NOW: local, clear reading, no source against it; after A11.

### F-r1-6: Chain tests later candidates as cast by the opposite side
- Class: BEHAVIOUR (a question)
- Original: exe 0x5A61F0, 0x5A6269: each chain candidate is tested with the opposite side as caster, so both heroes' Orbs of Vulnerability are asked. Conf. medium (BAT-Q21: readings about that Orb need an experiment).
- Engine: `next_in_chain` → `Guard::of(state, caster, s)`: the real caster's hero and the controller's.
- In play: sev. low; the defender's hero wears the Orb: the attacker's chain strikes his own Black Dragons (reading) or not (engine).
- Fix needs: a note in §7.8; "decide with BAT-MAG-066"; an experiment is implied.
- Reviewer proposes: the note.
- Suggested: DEFER (the experiment batch on the Orb).

### F-r1-7: BAT-MAG-004's "at least 1" reads as a quotient of at least 1
- Class: DOC
- Original: no floor on the quotient (exe 0x4E608D–0x4E609D; batch 2a R4: 115, not 119).
- Engine: right.
- In play: none.
- Fix needs: BAT-MAG-004's summary.
- Reviewer proposes: "(hero level div the stack's creature level, which is at least 1)".
- Suggested: DOCS.

### F-r1-8: Two summaries describe delivered work as still to come
- Class: DOC
- Original: n/a.
- Engine: all delivered.
- In play: none.
- Fix needs: the summaries of BAT-MAG-005 and BAT-MAG-001 ("join with T8", "other effects with their tasks").
- Reviewer proposes: state the present rule; history to the log.
- Suggested: DOCS.

### F-r1-9: "Once per round" cited from batch 2a R6, which never tries it
- Class: DOC
- Original: the limit holds (wiki, VCMI, the cast mark +0x54B4).
- Engine: right.
- In play: none.
- Fix needs: BAT-MAG-001's observation ref.
- Reviewer proposes: drop "and once per round" there, or cite the code reading.
- Suggested: DOCS.

### F-r1-10: Small test gaps (mana equal to cost, harvested replays)
- Class: TEST
- Original: n/a.
- Engine: untested: a cast with mana equal to the cost (`magic.rs:632`); batch 2a R1/R2 and R6's Ballista turn on the harvested packs; a tie between sides (F-r1-1).
- In play: none.
- Fix needs: tests only.
- Reviewer proposes: a third cast (15 → 0); a harvested replay of R1.
- Suggested: TEST.

### F-r1-11: VCMI differs from the chain rule in three points, unrecorded
- Class: DOC
- Original: exe 0x5A61F0 supports the rule. VCMI: resistance drawn once per cast, a skipped unit out for good, the next target closest to the skipped unit (`UnitEffect.cpp` L227, L246–253).
- Engine: follows the rule.
- In play: none for the engine (Dwarves beside the first target: struck about 99 % by the rule, 80 % in VCMI).
- Fix needs: BAT-MAG-006's VCMI note; an experiment "if wanted".
- Reviewer proposes: extend the note.
- Suggested: DOCS.

### F-r1-12: A creature's area spell or chain takes the hero's damage bonuses
- Class: BEHAVIOUR (a question)
- Original: exe: the area handler 0x5A4970, Chain Lightning 0x5A6360 and Armageddon 0x5A4BC0 fetch the acting side's hero whatever the caster (a creature's bolts get none). Conf. high for the code. VCMI contradicts (`CUnitState.cpp` L423–427); the wiki is not checked.
- Engine: `plan_creature_cast`: `SpellBonuses::default()` (`magic.rs:1467`).
- In play: sev. medium; Faerie Dragons under a hero with Sorcery, an Orb or the specialty: 46 of 100 draws.
- Fix needs: nothing before the sources are compared; then `plan_creature_cast`, or §7.21 says why not. Experiment: Fireball under the Orb of Tempestuous Fire, 80 or 120.
- Reviewer proposes: an OPEN-QUESTIONS entry with the experiment.
- Suggested: ASK-USER: reading against VCMI; same point as F-r5a-3.

**Report r2a-timed-effects**

### F-r2a-4: Defend bonus recomputed at each strike; the original fixes it
- Class: BEHAVIOUR
- Original: exe: Defend computes max(1, defense × 20 / 100) once and adds it to the defense field +0xCC (0x47974A–0x4798D7); the next turn start subtracts it; later defense changes do not move it. Conf. medium-high. VCMI is dynamic like the engine; the wiki fits both.
- Engine: `damage.rs:295-302`: the bonus from `defense_now()` at each strike.
- In play: sev. medium; 1–2 points when a defense effect starts or ends while a stack defends (12, Defend, Expert Stone Skin: 20 against 21).
- Fix needs: a new state field (the bonus stored at Defend), or an OPEN-QUESTIONS entry with an experiment.
- Reviewer proposes: either.
- Suggested: ASK-USER: executable against VCMI; a state field; with F-r4b-strike-magic-4.

### F-r2a-5: BAT-MAG-014's VCMI note is wrong (VCMI agrees)
- Class: DOC
- Original: Slayer at mastery none counts against king 1 (exe 0x442479–0x4424A4); VCMI agrees (`DamageCalculator.cpp:165-170`).
- Engine: right.
- In play: none.
- Fix needs: BAT-MAG-014's VCMI note.
- Reviewer proposes: "VCMI agrees".
- Suggested: DOCS.

### F-r2a-6: Dispel below Expert can be aimed at a stack with only Acid Breath
- Class: BEHAVIOUR
- Original: exe: the aim test wants a duration above 0 (0x5A81A2–0x5A81B9); Acid Breath sets none, Disrupting Ray 255. Conf. high on the code, medium-high on the executable.
- Engine: `magic.rs:901` accepts any effect entry; Acid Breath is one.
- In play: rare (Rust Dragons), small: a useless cast is legal; the action mask differs.
- Fix needs: "a spell flag" (new content data) for `needs_effect`, or the rule text accepts the difference.
- Reviewer proposes: either.
- Suggested: DEFER (battle lane's queue): a content flag for a useless cast.

**Report r2b-stack-spells**

### F-r2b-stack-spells-2: Retaliation at a striker killed by Fire Shield's reflection
- Class: BEHAVIOUR (a question)
- Original: exe 0x441CB0 (0x441DD4–0x441E01): the retaliation does not test the attacker's count, so the struck stack swings at the dead attacker and spends a retaliation. Conf. medium to high. VCMI needs the attacker alive, as the engine.
- Engine: `combat.rs` `retaliate` returns at L793–802 before `retaliations_left` is touched.
- In play: sev. medium, rare: the reflection kills the whole attacker and the shielded stack is attacked again that round.
- Fix needs: a user decision: a new fix-optional quirk flag, or clean in every profile; a ledger entry either way; a hot-seat experiment.
- Reviewer proposes: decide with the user.
- Suggested: ASK-USER: a quirk's class and flag; reading against VCMI.

### F-r2b-stack-spells-4: BAT-MAG-019's VCMI source misquoted (Frenzy's end)
- Class: DOC
- Original: exe (re-read): Frenzy lasts by the stack's turns (0x4448F0, 0x446E30). VCMI: `UNTIL_ATTACK` (`timed.json` L1130): until its first attack or retaliation. No conflict is recorded.
- Engine: follows the reading.
- In play: none unless VCMI is right.
- Fix needs: BAT-MAG-019's VCMI ref; the wiki's Frenzy page; if it is silent, an experiment.
- Reviewer proposes: "VCMI: until its first strike; not followed".
- Suggested: DOCS, with the wiki check.

### F-r2b-stack-spells-5: Wrong VCMI file, missing `quirks` fields, no VCMI source on Blind
- Class: DOC
- Original: n/a.
- Engine: right.
- In play: none.
- Fix needs: the VCMI file cited by BAT-MAG-018 and the ledger's Q-0010 (it is `server/battles/BattleActionProcessor.cpp` L1602–1616); the `quirks` fields of BAT-MAG-018, 019 and 022; a VCMI source for BAT-MAG-024.
- Reviewer proposes: those corrections.
- Suggested: DOCS (BAT-QUIRK-9 on BAT-MAG-022 waits for F-r5a-1).

### F-r2b-stack-spells-6: Five clauses no test pins
- Class: TEST
- Original: n/a.
- Engine: untested: (1) Q-0009 with Q-0005; (2) Frenzy's count for a stack that waits, in play; (3) Magic Mirror's new target among war machines; (4) batch 2b R3's 60 with its numbers; (5) a striker killed by the reflection.
- In play: none.
- Fix needs: tests only.
- Reviewer proposes: one test each.
- Suggested: TEST: (2) to (4); (1) comes with B5; (5) waits for F-r2b-stack-spells-2.

**Report r3-control-battlefield**

### F-r3-control-battlefield-4: BAT-QUIRK-2: the original never offers an attack from a seen pit
- Class: QUIRK
- Original: exe: the cursor's attack slots (0x474690) and the pathfinder's attack marking (0x4B378D-0x4B37D1) leave out positions on a stop: no interface path issues the command the ledger describes ("not read" there). Conf. high for the player's path.
- Engine: `quirk.battle.attack_from_seen_pit` (on in `base:faithful`) adds such attacks (`legal.rs:121-123`).
- In play: sev. medium; `faithful` lists Attack commands that only walk into the pit.
- Fix needs: drop the quirk flag or swap its defaults; the ledger entry to "considered, not a quirk".
- Reviewer proposes: either.
- Suggested: ASK-USER: a quirk's flag and class (the user's decision of 2026-10-07, BAT-Q1).

### F-r3-control-battlefield-5: A two-hex walker leaving a pit along its row is stopped again
- Class: BEHAVIOUR (a question)
- Original: exe walker 0x441FA0 (0x442266-0x442334): each step tests the new head and tail hex for Quicksand with no exemption for the start hexes; along the row the new tail is the old head. Conf. medium; no observation. VCMI's default differs.
- Engine: `movement.rs:109-119` exempts the start hexes for the whole walk.
- In play: sev. medium; Quicksand holds cavalry two turns, not one.
- Fix needs: "if confirmed": the walker's stop rule; a wiki line or one experiment row settles it.
- Reviewer proposes: confirm first.
- Suggested: ASK-USER: reading against VCMI's default.

### F-r3-control-battlefield-8: Pit and mine draws avoid dead stacks that left no corpse
- Class: BEHAVIOUR
- Original: exe: the draw skips a hex with a living stack or a corpse (0x5A03E3-0x5A03F3, 0x5A05FB-0x5A060D). Conf. medium-high; which stacks leave no corpse is the project's own reading (§7.18).
- Engine: `obstacles.rs:388-390` skips every stack's footprint, dead ones with `NO_CORPSE` too.
- In play: sev. low; which hexes random pits and mines can take.
- Fix needs: skip dead stacks with `NO_CORPSE`; one test; BAT-MAG-032's text. Else not stated.
- Reviewer proposes: that.
- Suggested: FIX-NOW: local, no source against it.

### F-r3-control-battlefield-9: BAT-QUIRK-1 and BAT-MAG-031 misdescribe VCMI
- Class: QUIRK (its description)
- Original: confirmed (0x46402C). VCMI counts dead natives too (`CBattleInfoEssentials.cpp:107-118`); the wiki's "started the battle with" fits the original.
- Engine: the strict-fixed variant (sight ends when the natives die) follows neither VCMI nor the wiki.
- In play: none in `faithful`.
- Fix needs: the ledger's "intended behaviour", §7.16, the VCMI sources of BAT-MAG-031 and BAT-MAG-026.
- Reviewer proposes: correct; tell the user, "in case that changes the class or the strict-fixed variant".
- Suggested: ASK-USER (one line: does BAT-Q1's class stand); the texts are DOCS.

### F-r3-control-battlefield-10: No harvested replay of batch 2c; two boundaries unpinned
- Class: TEST
- Original: n/a.
- Engine: nothing replays batch 2c's numbers (F1, H1); BAT-MOV-003's tie rule (F2) and Hypnotize's `<=` at the limit are unpinned.
- In play: none.
- Fix needs: tests only.
- Reviewer proposes: `harvested_control_and_field.rs`; a stack of exactly 60 in `control.rs`.
- Suggested: TEST.

### F-r3-control-battlefield-11: A berserk stack's turn follows its side's hidden pits
- Class: HIDDEN-INFO
- Original: does the same (its pathfinder uses the stack's side's stops, 0x4B3384-0x4B33C4); conf. high.
- Engine: `control::berserk` → `reachable_within` (`control.rs:52`); the moves and strikes are public.
- In play: sev. low; Berserk on a stack of the side that owns hidden pits: its detour shows a pit.
- Fix needs: "no clean fix": a ledger note beside BAT-QUIRK-4, or the user decides whether it plans with revealed pits only.
- Reviewer proposes: record it or ask.
- Suggested: ASK-USER: same point as F-sec-1 part a.

**Report r4a-life-created**

### F-r4a-2: A dead stack's round state is frozen until raised and a new round
- Class: BEHAVIOUR
- Original: exe: the round start skips a record without creatures (0x447120, 0x44712D), so a dead stack keeps its acted bit and retaliations; the raise clears only the dead bit. Conf. medium, no observation. VCMI resets every stack, as the engine.
- Engine: `turn::start_round` (turn.rs:80–87) resets dead stacks too.
- In play: sev. high if confirmed: a stack that acted, died and is raised next round gets no turn that round (reading); it acts (engine).
- Fix needs: `start_round` leaves dead stacks alone; a test; an experiment "before the engine changes".
- Reviewer proposes: the experiment, as a question for the user.
- Suggested: ASK-USER: reading against VCMI in ordinary play.

### F-r4a-4: An expired created stack's record is never reused
- Class: BEHAVIOUR
- Original: exe 0x47A100 (0x47A131–0x47A14A) reuses only an empty record or a dead summoned one with count 0; an expired stack keeps its count. §7.19 has it; the summary and BAT-QUIRK-11 drop "with no creatures". Conf. high, no observation.
- Engine: `vanish` zeroes the count; `add` reuses any dead created stack (created.rs:410–413).
- In play: sev. medium, rare: the stack limit after many expired clones; a later created stack's id (turn order); the outcome's record.
- Fix needs: `add` reuses only a dead created stack with `vanished == 0`; summary, ledger; a test.
- Reviewer proposes: that.
- Suggested: FIX-NOW, after B2 (same `created::add`): local, already in §7.19.

### F-r4a-5: A raise aimed at a two-hex corpse raises its position hex's top corpse
- Class: BEHAVIOUR
- Original: exe: the hero's raise handler 0x5A1907 takes the found stack's position (the rear hex) and searches again from there (0x5A1917–0x5A1928). Conf. medium to high; no second source.
- Engine: `on_top` (raise.rs:179) accepts any hex of the footprint and raises the corpse aimed at.
- In play: sev. medium, rare: a later corpse B on two-hex corpse A's rear hex: B comes back; A cannot be raised while B lies there.
- Fix needs: `on_top` looks at the tail hex for the hero's raise spells; a test.
- Reviewer proposes: that.
- Suggested: FIX-NOW: local; tell the user (an oddity that may want a ledger line).

### F-r4a-6: Surrender price counts creatures raised for the battle only
- Class: BEHAVIOUR
- Original: exe 0x474FE9–0x47507F (first read): cost × (count − temporary counter), summoned records skipped. Conf. high for the reading. VCMI: cost × count; the wiki is silent.
- Engine: flee.rs:51–52: `count × cost`.
- In play: sev. medium; Basic Resurrection, then a surrender: 5 of 10 Swordsmen: 750; raised to 8: still 750, in the engine 1 200.
- Fix needs: one line in `surrender_price`; a test; an "experiment candidate".
- Reviewer proposes: that.
- Suggested: ASK-USER: reading against VCMI; one line of code.

### F-r4a-7: Sacrifice's victim test is the original's, not an engine choice
- Class: DOC
- Original: the second cursor 0x5A2E90 refuses the first target's stack; batch 2d RA1 saw it.
- Engine: right (`v != s`).
- In play: none.
- Fix needs: BAT-MAG-036's summary ("Engine choice"), §7.18, the map (add 0x5A2D00, 0x5A2E90).
- Reviewer proposes: reword.
- Suggested: DOCS.

### F-r4a-9: Surrender price halves before the discount (1 gold)
- Class: BEHAVIOUR
- Original: exe 0x47506B–0x47507F: the sum is halved as an integer, then multiplied by the factor and truncated. Conf. high. E04a's rule.
- Engine: flee.rs:67: one division at the end.
- In play: sev. low; one Pixie at 10 %: 10 against 11; none found with the discounts 20, 40, 60 %.
- Fix needs: under Q-0005 `(cost / 2) × kept`. Else not stated.
- Reviewer proposes: that.
- Suggested: DEFER (battle lane's queue, with F-r4a-6).

### F-r4a-10: Test gaps (raise boundary, summon row, clone place, batch 2d numbers)
- Class: TEST
- Original: n/a.
- Engine: untested: BAT-MAG-035's `<=` at the health; BAT-MAG-039's row draw and "no place"; two of BAT-MAG-038's place skips; a created stack's end through real commands; batch 2d's rows with their numbers.
- In play: none.
- Fix needs: tests only.
- Reviewer proposes: one assertion each; a base-pack test for RA2.
- Suggested: TEST (the real-commands item comes with A6).

**Report r4b-strike-magic**

### F-r4b-strike-magic-4: Defend bonus inside the defense field (Acid Breath)
- Class: BEHAVIOUR
- Original: exe: as F-r2a-4; Acid Breath tests the defense field above 0 (0x4407F8) and lowers it by 3 (0x5A1E70–0x5A1E8D), the Defend bonus included. Conf. high. No VCMI or wiki comparison.
- Engine: `spell_after_strike` tests `defense_now() <= 0` without the bonus (`after_strike.rs` L139–L144).
- In play: sev. medium, one point: defense 0, defending: the breath lands and leaves −1 (engine: no breath).
- Fix needs: a new state field ("a stack field or a pseudo-effect"); a test.
- Reviewer proposes: that fix.
- Suggested: ASK-USER: one decision with F-r2a-4.

### F-r4b-strike-magic-5: BAT-MAG-040..047 on the reading alone; VCMI contradicts ten points
- Class: DOC (BEHAVIOUR where VCMI is right)
- Original: exe confirms §7.20; VCMI differs (executable / VCMI): Knights' Curse 25 / 20 %; Mummies' 25 / 50 %; Rust Dragon's extra damage 20 / 30 %; Unicorns' Blind mastery 0 (retaliation at half) / level 3 (none); Dragon Flies' Weakness −6 / −3; Thunderbirds' bolt as Lightning Bolt / a level-0 spell; Stone Gaze resisted / not; Poison a tenth per round, kept / per turn to −30 %, given back; Age 25 → 13 / 12; Bind ends at the binder's move or death / at the bound stack's turn start. Wiki not consulted.
- Engine: the reading on all ten.
- In play: sev. medium "if the other sources are right".
- Fix needs: VCMI and wiki sources on the eight entries; an OPEN-QUESTIONS entry with an experiment batch and the user's decision per point (`docs/04` §2).
- Reviewer proposes: that.
- Suggested: ASK-USER: `docs/04` §2; r4b's wiki checks first.

### F-r4b-strike-magic-6: Test gaps (Q-0005 in a battle, shot at petrified, Bind's expiry)
- Class: TEST
- Original: n/a.
- Engine: untested: Q-0005's Poison and Age variants in a battle; a shot at a petrified stack is halved; Bind's end after its rounds.
- In play: none.
- Fix needs: tests only.
- Reviewer proposes: a 9-hit-point creature poisoned and aged: `max_hp` 5, and 4 with `t:single`.
- Suggested: TEST.

### F-r4b-strike-magic-7: Rule texts and two missing ledger entries
- Class: DOC
- Original: n/a.
- Engine: right.
- In play: none.
- Fix needs: no rule names the Zombies' Disease; BAT-MAG-042's last sentence is stale; BAT-MAG-043: shots are halved too; BAT-MAG-041: a bolt of 0 (A9); no ledger entry for two original bugs the engine leaves out (the bound Harpy, BAT-Q9; the binder on a stack its strike killed, BAT-Q8).
- Reviewer proposes: those corrections.
- Suggested: DOCS.

**Report r5a-creature-casters**

### F-r5a-1: Magic Mirror never redirects a creature's spell (BAT-QUIRK-9)
- Class: BEHAVIOUR (and QUIRK: neither variant of the flag is the original's)
- Original: exe: the redirect is drawn only in the player's targeting 0x59EC50 into +0x48, which a creature's action never passes on (0x47936E–0x479382). Conf. high; VCMI agrees (`BattleSpellMechanics.cpp:520`).
- Engine: `magic.rs:2331-2348` calls `mirror` for every cast with a stack target.
- In play: sev. medium; a Faerie Dragon's bolt or chain at a stack under Magic Mirror, or at enemy Faerie Dragons (innate 20 %: one cast in five), goes to a random own stack.
- Fix needs: `mirror` only for `cast.by.is_none()`; retire the quirk flag `quirk.battle.creature_mirror_untested`; tests, §7.21, map, ledger. Whether a clean profile mirrors creature casts "is the user's call".
- Reviewer proposes: that.
- Suggested: ASK-USER: a quirk flag goes; reading and VCMI agree.

### F-r5a-2: F key: the Faerie Dragon aims an area spell at any hex
- Class: BEHAVIOUR
- Original: exe: key F (0x475C17) opens the hero's targeting for the drawn spell (0x59EC50): any hex outside columns 0 and 16; the mouse path gives the rule's targets. Conf. high for the path; two corners unsettled (the side's hero has cast this round; no hero: "probably crashes"). VCMI aims at a location.
- Engine: `drawn_target` (`magic.rs:1369-1388`): only hexes of affectable stacks.
- In play: sev. medium; 41 of 100 draws: no Fireball between two stacks; a Frost Ring must be centred on a stack.
- Fix needs: a drawn hex spell accepts every standable hex ("at least when the controller has a hero"); the rule text; one observation. No type change stated.
- Reviewer proposes: that.
- Suggested: ASK-USER: when the wider targets apply hangs on the two corners.

### F-r5a-3: A creature's area spell or chain takes the hero's damage bonuses
- Class: BEHAVIOUR
- Original: the reading of F-r1-12; conf. medium to high; VCMI gives none.
- Engine: `magic.rs:1467`.
- In play: two Faerie Dragons' Fireball: 130; 195 with the Orb of Tempestuous Fire, 149 with Expert Sorcery.
- Fix needs: a creature's area or chain cast takes the bonuses of `state.heroes[cast.side]`; "queue the observation first".
- Reviewer proposes: observe, then fix.
- Suggested: ASK-USER: one decision with F-r1-12.

### F-r5a-4: The F-key path has no "magic allowed" test (Orb of Inhibition)
- Class: BEHAVIOUR (a question)
- Original: exe: none of 0x41F890's six callers lies on the F path (0x475C17 → 0x59EC50 → action 10 → 0x448260). Conf. medium to high. The wiki says Faerie Dragons are unaffected by the Orb; batch 2e OI tried only the mouse cursor.
- Engine: `casters.rs:391-399` refuses; no creature has `ignores_magic_block` since batch 2e.
- In play: sev. medium; Faerie Dragons under an Orb of Inhibition.
- Fix needs: "observe first" (OI's setup, press F); then pack data, §7.23, BAT-Q21's record.
- Reviewer proposes: the observation.
- Suggested: DEFER (experiment); it reopens BAT-Q21 point 1.

### F-r5a-5: VCMI misquoted in two entries; its contradictions unrecorded
- Class: DOC
- Original: exe re-reads agree with the catalog.
- Engine: follows the readings.
- In play: none unless VCMI is right.
- Fix needs: the VCMI refs of BAT-MAG-053 (six `ENCHANTER` spells, not eight) and BAT-MAG-049 (`RANDOM_SPELLCASTER` is the Genie's); notes on BAT-MAG-048 (VCMI: Ogre Mage 3 against 6; obstacles act after the cast; every unit rolls resistance) and BAT-MAG-050 (VCMI: no Frenzy, 5 against 6); cite VCMI on BAT-MAG-051.
- Reviewer proposes: those.
- Suggested: DOCS, with r5a's wiki checks.

### F-r5a-6: BAT-QUIRK-10 is `preserve` without a flag
- Class: QUIRK
- Original: confirmed (0x447B20–0x447B82); VCMI differs.
- Engine: one variant in every profile (user, BAT-Q12); `strict-fixed` keeps it.
- In play: none.
- Fix needs: one sentence in `docs/04` §4 (a shared path), or a new quirk flag.
- Reviewer proposes: either.
- Suggested: DOCS, with F-r6-limits-opening-9.

### F-r5a-7: Two boundary clauses untested (Archangel `<=`, area rolls)
- Class: TEST
- Original: n/a.
- Engine: untested: `raise.rs:136` at a health equal to 100 × creatures; a resisting stack in a creature's area; a creature's chain whose first stack does not roll while a later jump does.
- In play: none.
- Fix needs: tests only.
- Reviewer proposes: health 100 a target, 101 none; `novaling` beside a `dwarf` over 40 seeds.
- Suggested: TEST.

**Report r5b-mana-defences**

### F-r5b-mana-defences-3: Bad morale: a side no human commands escapes one freeze in four
- Class: BEHAVIOUR (a question)
- Original: exe 0x464B40 (first full read): for a side no human commands (byte +0x54A4) a second `random(1, 4)` lets the stack off on a 1 (0x464BDB–0x464BF8). Conf. high for the reading; the flag's meaning rests on its setter and H3API's names. Wiki and VCMI know no such rule.
- Engine: `morale.rs:357–365`: one roll for every side.
- In play: sev. medium; computer and neutral sides freeze with 3/4 of the chance (6.25, 12.5, 18.75 %).
- Fix needs: an OPEN-QUESTIONS entry: does the seam say who commands a side (a seam change, or "a Tier-2 parameter per side"), or does the human rule stay; an experiment of a few hundred activations.
- Reviewer proposes: the question.
- Suggested: ASK-USER: a seam question; reading against wiki and VCMI.

### F-r5b-mana-defences-4: The wait phase orders every pending stack by the phase
- Class: BEHAVIOUR
- Original: exe: the wait phase clears every waited bit (0x4652A7–0x4652E5); every stack not done is then a candidate, the slower first (0x464FDB–0x465007); the Wait button is disabled (the dispatcher was not read). Conf. medium; no observation; no VCMI or wiki comparison.
- Engine: `turn::next_active` (`turn.rs:44–60`): a stack without `WAITED` goes before every waited one; Wait is refused only by the stack's own flag.
- In play: sev. "medium, perhaps high": a stack summoned or cloned in the wait phase acts before the pending waited stacks, and may Wait and act again.
- Fix needs: order by `state.wait_phase`; refuse Wait while it is set; a test. An experiment is offered. Outside the slice (BAT-TRN-002).
- Reviewer proposes: that fix.
- Suggested: ASK-USER: turn order and legal commands change on one reading, conf. medium.

### F-r5b-mana-defences-5: A freeze counts as "its side acted last" in the engine only
- Class: BEHAVIOUR (a question)
- Original: exe: the tie-break of 0x464F50 prefers the side that did not act last (+0x132B8), which an activation writes (0x465362) and a freeze does not. Conf. medium. VCMI does as the engine; the wiki fits both.
- Engine: `turn.rs:245` sets `last_side_acted` after a bad-morale or Fear freeze.
- In play: sev. medium, an edge: a freeze, then a speed tie across the sides.
- Fix needs: "decide with BAT-TRN-001's owner"; then skip line 245.
- Reviewer proposes: a decision.
- Suggested: ASK-USER: reading against VCMI; one line either way.

### F-r5b-mana-defences-6: BAT-MAG-059/060 cite the reading alone; VCMI differs in five points
- Class: DOC
- Original: exe as the entries say. VCMI differs: the rebirth's cast is used only if a creature returns; the reborn stack loses its retaliations and returns at its death; only LIVING stacks are frightened (the original frightens Gargoyles, 0x464D4C–0x464D57); Fear rolls in the wait phase too; the channel is computed before the effect (BAT-MAG-055).
- Engine: follows the reading.
- In play: none unless VCMI is right.
- Fix needs: the VCMI sources of BAT-MAG-059, 060 and 055 with these differences.
- Reviewer proposes: that; the Gargoyles and the wait phase to a wiki check.
- Suggested: DOCS.

**Report r6-limits-opening**

### F-r6-limits-opening-2: Aura under the Orb of Vulnerability: reading against a weak observation
- Class: BEHAVIOUR (a question; no code change proposed)
- Original: exe: the aura's 0.8f (0x5A8592–0x5A85CF) has no artifact test: the code keeps the aura under the Orb (a second independent reading). Against it: batch 2e UA, 12 of 12 slowed (6.9 % under the reading), no control; the user decided (BAT-Q27) with the wiki and VCMI.
- Engine: lifts the aura under the Orb.
- In play: sev. medium; Unicorns against an Orb of Vulnerability; conf. "about even" that the engine differs.
- Fix needs: nothing in code; split the observation's confidence (OV high, UA low); re-test UA with a control, at least 30 rolls.
- Reviewer proposes: that.
- Suggested: DEFER (experiment); the split is DOCS.

### F-r6-limits-opening-3: VCMI lets the Orb lift artifact immunities; catalog says it agrees
- Class: DOC
- Original: exe (0x44A5F6 before 0x44A746) and the wiki: no negation lifts the Pendants', Armageddon's Blade's or the Dragon Father's immunity; not observed. VCMI lifts them (`TargetCondition.cpp` L306–330).
- Engine: as the reading.
- In play: none unless VCMI is right.
- Fix needs: the source notes of BAT-MAG-063, 064, 066 and BAT-Q21's list; one cursor check in the next batch.
- Reviewer proposes: that.
- Suggested: DOCS.

### F-r6-limits-opening-4: Thunderbirds' bolt is no level-0 ability (batch 2e DF note)
- Class: DOC
- Original: the bolt is Lightning Bolt, spell 17, level 2 (0x4411D4, 0x4411F5).
- Engine: right.
- In play: none.
- Fix needs: BAT-MAG-064's 2e note and BAT-Q21 point 5: for level 0 only Poison counts (0 of 4 at 30 % = 24 %, not "12 %").
- Reviewer proposes: reword.
- Suggested: DOCS.

### F-r6-limits-opening-5: Q-0005 changes Eagle Eye's chance: not in `quirks`, ledger, tests
- Class: DOC (with a test gap)
- Original: exe 0x4E4420: e.g. Advanced with the Emblem of Cognizance 64 %, a Basic specialist at level 2 43 %.
- Engine: follows the same steps (expected, not run); tests pin only 44 → 43 with a hand-made rate.
- In play: none known.
- Fix needs: "Q-0005" in the `quirks` of BAT-MAG-069 and HER-SPL-004 and Eagle Eye in the ledger's Q-0005 entry; a harvested test.
- Reviewer proposes: Orrin: 64 in `base:faithful`, 65 in `base:strict-fixed`; `base:serena`: 43 and 44.
- Suggested: TEST (plus the three text lines).

### F-r6-limits-opening-6: Two timing clauses of the opening spells untested
- Class: TEST
- Original: the activation comes after bad morale and Fear (0x465283); the opening cast before the Enchanters' count.
- Engine: right; `opening.rs:226` covers only `NO_TURNS`.
- In play: none.
- Fix needs: two tests.
- Reviewer proposes: `BadMorale < TurnStarted < OpeningSpell`; `OpeningSpell` before `PeriodicCast`.
- Suggested: TEST.

### F-r6-limits-opening-7: No test of a strike's casting side for the negation
- Class: TEST
- Original: a strike's spell passes the striker's flipped side (0x44054E–0x44057B).
- Engine: right (`after_strike.rs:146`); `NegateImmunities` is tested only for a hero's cast and a mine.
- In play: none.
- Fix needs: one test in `hero_defences.rs`.
- Reviewer proposes: `blinder` at the mind-immune `statue`, with and without `NegateImmunities`.
- Suggested: TEST.

### F-r6-limits-opening-8: Two comments still say the Faerie Dragon ignores the Orb of Inhibition
- Class: DOC
- Original: batch 2e settled it the other way (this report's view).
- Engine: right; stale comments at `crates/h3-content/src/ability.rs:370–372` and `crates/h3-battle/src/casters.rs:381`.
- In play: none.
- Fix needs: reword both.
- Reviewer proposes: "an ability no harvested creature has; kept for packs".
- Suggested: DOCS, after F-r5a-4 (which doubts batch 2e's row).

### F-r6-limits-opening-9: Exit criterion 3 and `docs/04` §4 against four flagless quirks
- Class: QUIRK
- Original: n/a.
- Engine: BAT-QUIRK-10, 11, 15 and 18 are "preserve, no flag" by user decisions (BAT-Q12, Q13, Q15, Q18); `strict-fixed` keeps them.
- In play: none.
- Fix needs: one sentence in `docs/04` §4 (a shared path) or in the criterion.
- Reviewer proposes: either.
- Suggested: DOCS.

**Report robustness**

No open finding: F-robust-1 to F-robust-6 are all in the fix lists.

**Report security**

### F-sec-1 part a: A berserk walker's turn shows its own side's hidden pits
- Class: HIDDEN-INFO
- Original: "not known" to this reviewer; F-r3-control-battlefield-11 read that it does the same. No recorded quirk.
- Engine: `control.rs:52`: the berserk stack's reach uses the pits its own side sees; target and position follow; the events are public.
- In play: sev. medium (both parts), conf. medium to high (not run); Berserk on a walker of the pits' owner: a detour or a passed turn proves pits; with BAT-QUIRK-2 the walker stops in the pit beside its target (`control.rs:136`): the exact hex.
- Fix needs: "decide first" (a quirk's class): a Q-0002 instance, or a fix (plans use only the pits both sides see; a pit an automatic walk stops in is revealed). Test sketch.
- Reviewer proposes: classify, then fix or record.
- Suggested: ASK-USER: a quirk's class; A1 leaves it to the user.

### F-sec-4: Properties never play Berserk, Hypnotize, hidden hero magic, BAT-QUIRK-2
- Class: TEST
- Original: n/a.
- Engine: `make_setup`'s book has no `t:berserk`, `t:hypnotize`, `t:blind` (`tests/properties.rs:221-257`), so no controller differs from its side; no hero has `NoCasting`, `SpellLevelLimit`, the immunities, `NegateImmunities`, an opening spell or Eagle Eye; `hidden_obstacles_do_not_show` never runs `t:pits` and no `CreatureCast`.
- In play: none; sev. medium (`docs/06-security.md` and ADR-0014 rest on these properties).
- Fix needs: tests only (generators; assertions by controller).
- Reviewer proposes: the three spells, the E04b effects, `t:pits`, a wider `arb_command`.
- Suggested: TEST, after A10; Berserk in the hidden-obstacle property waits for F-sec-1 part a.

### F-sec-5: Hidden draws share the public rolls' stream
- Class: HIDDEN-INFO
- Original: n/a.
- Engine: hidden draws (pit and mine hexes, the Faerie Dragon's spell, Eagle Eye) come from the same `state.rng` as the public rolls; xoshiro256\*\* seeded from 64 bits (`h3-kernel/src/rng.rs:83-97`).
- In play: sev. low, conf. low (an estimate): at most 2^64 offline seed trials against public rolls give every hidden draw.
- Fix needs: none in the battle lane: a line under "Not covered yet" in `docs/06-security.md`, or a keyed stream for hosted games ("ADR-0003 territory", `h3-kernel`).
- Reviewer proposes: one of the two.
- Suggested: DOCS (a `shared/` change); the keyed stream to the server epic.

## 3. Wiki checks wanted; not reviewed, left open

### Wiki checks wanted (merged; reports in brackets)

- Chain Lightning: equal distances beyond "randomly chosen"; are own stacks preferred [r1].
- Sorcery and its specialists: a worked number that tells 58 from 57 [r1].
- Unicorn aura: whose Unicorns protect a hypnotized stack [r1, r6].
- Spell book, Combat: can a mass or battle-wide spell that would affect nobody be cast [r1].
- Faerie Dragon: do the hero's Sorcery, Orbs or specialty raise its area spells and chain [r1, r5a]; does Magic Mirror reflect its spells; can its area spells be aimed at an empty hex; the wording on the Orb of Inhibition, the anti-magic garrison and Cursed Ground [r5a].
- Magic Mirror: "spells cast by creatures are not reflected"? [r5a]; can the reflected spell go to the caster's war machines; are the computer player's spells never reflected [r2b].
- Morale, Luck and their four spells: does a base beyond ±3 absorb the spells [r2a]; do computer or neutral armies freeze less often [r5b].
- Dispel, Hypnotize: does Expert Dispel end Hypnotize, also on the caster's own stack [r2a, r3].
- Disrupting Ray, Rust Dragon, Resurrection: does the defense loss survive death and a raise [r2a, r4b, r5b].
- Defend, Stone Skin, Prayer: is the +20 % fixed when the stack defends [r2a, r4b].
- Slayer: the list at no skill [r2a].
- Behemoth, Ancient Behemoth: the rounding of the defense reduction (defense 10: 5 or 6) [r2b].
- Frenzy: how long it lasts; does a retaliation end it [r2b].
- Fire Shield: a retaliation against an attacker the shield killed; the HotA changelog [r2b].
- Anti-Magic: which active spells the cast removes; are the own side's spells blocked too [r2b].
- Adela (Bless specialty): the levels divided first (VCMI multiplies first) [r2b].
- Hypnotize: who may attack a hypnotized stack; does it block the hypnotizer's shooters; First Aid Tent and Ammo Cart for it [r3].
- Berserk: can a player attack or shoot his own berserk stack [r3].
- Quicksand: is a two-hex creature stopped a second time; can a creature in a pit attack by stepping one hex [r3].
- Quicksand, Land Mine: native terrain: "started the battle with" or living [r3].
- Harpy, Fire Wall: is a Harpy burnt when it returns onto a Fire Wall hex [r3, r4b].
- Remove Obstacle, Dispel, Fire Wall, Force Field: a crash, or a later obstacle vanishing, after a timed obstacle was removed early [r3].
- Resurrection, Archangel: does a stack raised in a later round act if it had acted in the round it died; the HotA changelog [r4a].
- Sacrifice with Efreet, Efreet Sultan, Phoenix: may a fire-immune creature be raised [r4a].
- Surrender: is the price taken from creatures raised by Basic Resurrection [r4a].
- Clone: do expired clones count towards a side's 20 stacks [r4a].
- Black Knight, Dread Knight, Mummy: the chance of Curse (25 %; VCMI 20, 20, 50) [r4b].
- Rust Dragon: the chance of the extra acid damage (20 %; VCMI 30 %) [r4b].
- Unicorn: the level of its Blind (retaliation at half, or none) [r4b].
- Dragon Fly: the level of its Weakness (−3 or −6) [r4b].
- Thunderbird: is the bolt treated as Lightning Bolt (immunities, Protection from Air, Anti-Magic) [r4b].
- Stone Gaze: can Dwarves or a hero's Resistance resist it; the retaliation against the attack that frees it; is a shot halved [r4b].
- Paralyze: the retaliation at 25 % [r4b].
- Poison: per round or per turn, the floor, do the hit points stay lowered, Cure and Dispel [r4b].
- Age: the rounding (25 → 13 or 12) [r4b].
- Bind: what frees the stack [r4b].
- BAT-ATK-004's two wiki quotations (the reviewer could not check them) [r4b].
- Ogre Mage: the duration of its Bloodlust (6; VCMI 3) [r5a].
- Master Genie: the duration (6; VCMI 5), Frenzy in the list, Prayer's and Slayer's conditions [r5a].
- Enchanter: the eight spells and the round of the first cast [r5a].
- Azure Dragon: who is immune to Fear (the Gargoyles decide), its chance, a stack that waited [r5b].
- Phoenix: the rebirth's numbers, once per battle, does a failed rebirth use it up [r5b].
- Psychic and Magic Elemental: the half damage (BAT-DMG-010 has no source beside the reading) [r5b].
- Combat: the wait-phase order of a stack that did not wait, and can it wait; does a frozen stack count as its side's chance to act [r5b].
- Familiar, Mage, Pegasus, Wraith: can mana go above the maximum through the channel [r5b].
- Orb of Vulnerability: the sentence on artifact immunities and on the Unicorns' aura; does either cite a test [r6].
- Recanter's Cloak, Power of the Dragon Father: statements on creature abilities [r6].
- Eagle Eye (SoD): does any page give the odd values 43 %, 64 % [r6].
- Badge of Courage: "immunity to mind spells" for SoD [r6].

### Not reviewed, left open (merged)

Behaviour the reports met and did not follow (candidates for the queue):

- [r3] Ammo Cart: the test is "the hero of the shooter's side wears artifact 5", no living-cart test (0x43F9E1-0x43FA03); BAT-WAR-004 says "a living Ammo Cart".
- [r3] The automatic First Aid Tent heals the stack with the most lost hit points, not a random one (0x473F3C-0x473F67; BAT-WAR-003 says uniformly chosen).
- [r3] Hypnotize ending on the active stack in mid-turn: the original keeps the acting side until the next activation (0x4653AE); the engine hands the turn over at once (`side_to_act`).
- [r3] An original bug not in the ledger: an obstacle removed early is removed again when its rounds run out (0x476162-0x47618F), clearing what was placed since, and calls through a graphic set to 0; a ledger line (exploit-fix) if an experiment or the wiki confirms it.
- [r5b] The dispatcher's strike case (0x47927A) rolls good morale before the stack re-enters its hexes; the engine re-enters first (`command.rs:420–431`).
- [r5b] No turn start (0x446E30) in the original's wait phase: a stack first picked there gets none; the engine regenerates it and counts its own-turn effects (`turn.rs:219–225`).
- [r5b] A stack created in the original's wait phase and blinded before its turn seems to make the pick 0x465080 loop for ever (a soft-lock the engine does not have).
- [r5b] A `rebirth` creature that a mine or Fire Wall kills on its way walks on and attacks after its rebirth (`obstacles.rs:785`); the original "probably does the same".
- [r1] An active Phoenix killed by its own hero's cast and reborn stays the active stack (`command.rs:458`); the original was not read.
- [r2b] An active stack hit by a redirected Blind in its own turn finishes its turn in the engine; no test found in the original (around 0x4745DF).
- [r2b] BAT-MAG-058: the engine takes the larger of a cast Fire Shield and the creature's own, the original the cast one first (no effect with the base values).
- [r2a] Stage 1's hero step runs in the original also at rate 0 (0x44371C–0x443759); the engine skips it: differs only for a bonus sum above 2^24.
- [r1] The original rolls also at a chance of 0 or 100 and walks an area's stacks last to first: only the random stream differs.
- [r6] §7.23, two notes without effect on a rule: Armageddon's Blade's test is listed before the polarity tests (the code has them first, 0x5A82F8); the spell-book test of 0x41F890 is not tied to Cursed Ground.
- [sec, for the server lane] The wire has no query for a creature's cast targets (`session.rs:967-975`, `wire.rs:990-1003`); the sample scenario hosts `synthetic:faithful`, with BAT-QUIRK-4 and BAT-QUIRK-2 on.
- [sec] The alignment of `out` in `h3_env_observe` is left to the caller; `event.rs:312-314`'s claim (the original's log shows the three `SpellDamage` amounts) was not verified.
- [robust] Not findings: one command may run many rounds while no stack can act (253 seen; forced: 15 555 rounds, 4 ms); the effect and obstacle lists outgrow their reserved sizes.

Not read or not run:

- [r1, r5b] The book dialog 0x59BDF0 and the book's own cost test: "less mana than the cost cannot cast" rests on the wiki and VCMI; why the tactics phase allows no cast [r1].
- [r1] The 44 × 42 pixel grid rests on batch 2a; Q43 stays open; the sign of the two-hex shift is not observed.
- [r2a] The Rust `X87` was not run (a hand port to Python); a slowed speed's truncation rests on the reading.
- [r1, r2a, r2b, r3, r4a, r4b, r5a, r6] Helper functions used as the map describes, not re-read; each report lists its own (among them the pathfinder's costs and tie order 0x4B3400, a berserk stack's walk to a far target 0x41F580, the turn-order tie-break 0x464F50, the specialist bonus 0x4E5FF0).
- [r4a] The sim's, the protocol's and the C ABI's encodings of the raised and created states; [r4b] a war machine's Defend bonus; F-r4b-strike-magic-4's wider reach traced, not tested.
- [r2a] Batch 2b R2 is reproduced on the retaliation counter only; [r2b] the wiki quotes of BAT-MAG-023 unchecked; [r4a] BAT-MAG-051/052 only where they share `add`, `on_top` and the outcome, Necromancy's loop 0x4775E1 in part; [r5b] 0x447A80's usefulness cases for an innate shield or mirror through the tests only; [r6] the byte +0xEA at 0x5A829B.
- [r5a] That record 0 is a side's first army stack (BAT-QUIRK-13) and that the manager is created once per session (BAT-QUIRK-14); the byte +0x13D74 and the null-hero corner of the F path.
- [r6] `HeroOutcome.learned` after the battle (adventure lane); the other ledger entries only for "flag and test name exist" (Q-0001, Q-0002 name no test).
- [r3, r4b, r5a, r5b, r6] The computer player's paths: the automatic Ballista part of A10 rests on VCMI [r3]; network, hidden-battle and tactics-phase callers [r5a].
- [r3, r4a, r4b, r5a, r5b, r6] For E04c: sieges (Teleport's limits: VCMI has the moat at Advanced and the wall at Expert, §7.15 the wall first; the moat's rebirth check; arrow towers), Cursed Ground, Magic Plains, the anti-magic garrison, the special terrains.
- [robust] Not exercised: sieges, adventure spells (Earthquake never cast), scenario files and `HeroDetails`, the server, protocol and client crates, other targets, the determinism matrix, battles beyond 450 commands; the sibling `u128` products in `flee.rs`, `war.rs`, `magic.rs` read, not proven.
- [sec] Not read: `h3-server`'s I/O shell and journal, `h3-heroes`, the adventure seam, timing; what the original's berserk AI plans with. No finding in the wire mapping, the flat encoding, the C ABI's `unsafe`, bounded work (bounded by data only: the automatic turns inside one command and the battle's length, no round limit).
- [all rule reviewers] No wiki access; no cargo run (test sketches are `needs-run`).

## 4. Cross-report notes

The same point in two reports:

- F-r1-12 = F-r5a-3 (r1: conf. high for the code; r5a: medium to high; both: VCMI against).
- F-r2a-4 = F-r4b-strike-magic-4: r2a calls it a conflict with VCMI for the user; r4b proposes the fix outright and compares no other source.
- F-r3-control-battlefield-11 = F-sec-1 part a (F-robust-1 asks for "the same look"). They differ on the original (r3: read, does the same; sec: not known) and on the fix (r3: "no clean fix"; sec: plan with the pits both sides see and reveal a pit the walk stops in, which by r3's reading the original does not do).
- F-r5a-6 = F-r6-limits-opening-9 = r4a's note on BAT-QUIRK-11 (r5b's coverage says the same of BAT-QUIRK-15).
- r2a's pointer to the bad-morale check = F-r5b-mana-defences-3 (two independent reads of 0x464BDB–0x464BF8).
- F-r4b-strike-magic-5 (the bolt) and F-r6-limits-opening-4 rest on the same reading (spell 17).

Reports that disagree:

- BAT-QUIRK-9: r1 ("second look done" at 0x5A02CB–0x5A02CF) and r2b ("creature casts skip the roll") take the skip as the quirk's confirmation; F-r5a-1 shows the redirect never reaches a creature's cast.
- Orb of Inhibition: r6 holds BAT-MAG-061 ok and batch 2e as settling it (F-r6-limits-opening-8); F-r5a-4 says the batch tried only the mouse path.
- The byte +0x13D74: r5a "did not identify" it (a corner of F-r5a-2); r1 read that only a menu command toggles it (0x4F4603), "not a rule".
- Orb of Vulnerability: F-r1-6 (which hero's Orb a chain candidate meets) and F-r6-limits-opening-2 (the aura), -3 (artifact immunities), -7 (a strike's casting side) all concern the negation tested at 0x44A746; BAT-Q21 asks for experiments on this Orb: one batch can carry them.

Open findings that touch a fix item:

- A10: F-sec-4 comes after it; r3's pointers on the Ammo Cart and the First Aid Tent touch what A10 changes (A10 keeps "today's test of the cart itself").
- A11: F-r1-3 (hypnotized stacks in `no_target` / `reachable`).
- A1: sec's fix for F-sec-1 part a needs a reach with only the pits both sides see, which A1 adds (worker a's notes: `movement::reachable_in_public`).
- A6 changes `turn::start_round`, where F-r4a-2 sits; F-r5b-mana-defences-4 and -5 are in `turn.rs` too.
- B2: F-r4a-4 (`created::add`), F-r2a-6 (permanent entries), F-r4a-2 (what a dead stack keeps), F-r3-control-battlefield-8.
- B6: F-r1-12 / F-r5a-3 would send a creature's cast through the rewritten stage 1.
- B9 works in `combat::melee`, where F-r2b-stack-spells-2's retaliation sits.
- F-r3-control-battlefield-4 (BAT-QUIRK-2) decides the exact-hex form of F-sec-1 part a and `t:pits` in F-sec-4.

Left over from covered findings (by the lists' wording; the workers' notes agree where they exist):

- Docs: the fix brief forbids changes under `docs/`: every covered finding's rule text, ledger, §7.x and map corrections wait, to be written from `fix-notes-a.md` and `fix-notes-b.md`. F-r2b-stack-spells-1 also asks to "tell the user" (the decision of 2026-10-07 on Q19 was taken on the misreading).
- F-r1-5 / F-robust-3: the other proposal, a bound on the `SpellDamage` rates in `hero::check`, is not in B6.
- F-r2b-stack-spells-7 point 2: a long `fixed` duration on a turn-stopping change still compiles.
- F-r4a-3: the proposed property invariant ("each side has a living stack that is no war machine") is not in A6.
- F-sec-2: A8 takes the first option; "same root, no leak today" stays (`action::encode` reads the active stack's spell, `stack_spell` is public, `spell_obstacles()` returns hidden ones, the server's `Bot` trait gets the whole state).
- The robustness report kept two property tests on the unpushed local branch `review/robustness` (`5ad9475` `extreme_casting_heroes_play_without_failures`, `46fd5b9` `random_legal_sod_heroes_cast_and_play`), to be widened after B6 and A2; no fix item names them (worker a's notes: A2 took the second).

Experiments the reports propose that share a set-up:

- One Faerie Dragon session: F at a stack under Expert Magic Mirror, ten casts (F-r5a-1); F, then click an empty hex (F-r5a-2); Fireball under the Orb of Tempestuous Fire (F-r5a-3, F-r1-12); F under the Orb of Inhibition (F-r5a-4).
- One row each: Defend, then Stone Skin (F-r2a-4, F-r4b-strike-magic-4); a raise in a later round (F-r4a-2); the surrender dialog after Basic Resurrection (F-r4a-6); a summon in the wait phase (F-r5b-mana-defences-4, -5); the chain's tie (F-r1-1); a two-hex walker in a pit (F-r3-control-battlefield-5); a Fire Shield kill (F-r2b-stack-spells-2); UA with a control (F-r6-limits-opening-2). Statistics: F-r4b-strike-magic-5's ten points; neutral bad morale (F-r5b-mana-defences-3).
