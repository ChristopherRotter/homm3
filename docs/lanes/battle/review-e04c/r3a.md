# Review r3a: the attacker's Catapult, the wall-shot targets, the Catapult's turn, destroyed walls and gate

Slice: BAT-SIE-009, BAT-SIE-010, BAT-SIE-011, BAT-SIE-012, HER-BAT-018 (as far as BAT-SIE-011
reads it). On the way: the shot's roll, miss and re-aim (BAT-SIE-014's code, re-read for D-6,
D-12), because the task prompt asked for them.

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-SIE-009 | matches (`state.rs:383-484`: hero present, fort ≥ Fort, attacker only, first `war_machine`+`catapult` creature, rear hex 119 / front 120, slot 16+3, `InvalidWarMachine` when listed, SHOOTER trait cleared `state.rs:267-272`) | exe 0x463600 (0x46374D–0x4637AA: hero pointer non-null, fort > 0, side 0, creature 0x91 count 1 at hex 0x77, no artifact test; the Ballista's artifact test 0x4637AB comes after); VCMI `lib/battle/BattleInfo.cpp:296-320` (MACH4 only with `wallsHealth > 0`) + `config/gameConfig.json:630` (attacker war-machine hex 120) — the cited `server/battles/BattleProcessor.cpp` holds none of it; batch 3d A/B/E (creature 145 at 119) | `walls.rs:143-241` pins every clause incl. no-room and a ruleset without the creature; `walls.rs:245-291` never shoots stacks | F-r3a-4 (source citation only) |
| BAT-SIE-010 | matches (`siege.rs:302-326`: hp from the town record per target, +castle_bonus on `Wall` kinds only at a Castle, `present` by kind/fort, `stands()` 119-121) | exe 0x4662E3–0x466310 (18 parts copied from the exe table by town), 0x466348–0x466376 (+1 to parts 6, 8, 10, 12 = targets 1, 2, 4, 5 only when fort == 3), 0x476440 (towers fort ≥ 3, keep ≥ 2, hp > 0), 0x63BE60 records (aim, wall index, part: 5,6,8,9,10,12,13,14), 0x63BD00 (index 4 → 78); `BALLIST.TXT` and `WALLS.TXT` extracted from `H3bitmap.lod` and compared with the pack; VCMI `BattleInfo.cpp:178-196`; batch 3d A (2 hp each, absent towers too) | `walls.rs:297-357` pins Fort/Citadel/Castle presence, hp [2,3,3,2,3,3,2,2] at a Castle, kinds | ok |
| BAT-SIE-011 | mostly matches (`turn.rs:59-73` order; `war.rs:51-62, 97-105`; `siege.rs:750-823`; `command.rs:313-327`; `legal.rs:65-73`) — but the engine lists Wait and Defend for the commanded Catapult (F-r3a-2) and does not reproduce the pick's Catapult case 0x4650BB (F-r3a-1); "(it defends)" is not what action 12 does (F-r3a-5) | exe 0x473C00 whole (creature test, fort, the standing loop with fort tests, the hero's +0xD3, 0x46A4A0/0x474BF0 branches, the gate-first branch for Ballistics, the weakest wall of 0x63D54C with `random(1, n)` among the tied in list order, 0x670198 then index order, action 9 at the aim hex); tables 0x63D54C = [1,2,4,5], 0x670198 = [3,7,0,6]; 0x464F50 (0x464F85–0x464FC1: tower 0x95 then Catapult 0x91 before speed; 0x46502A–0x465052 the tie on +0x132B8); 0x465080 (0x46508B–0x4650C5: the Catapult case); 0x479BC2 (action 12); batch 3c, 3d A/B | `walls.rs:245-291, 363-476` pin the order before a faster stack, the aim tables, the commanded shots per level, NoSuchPart, the pass with nothing standing; no test of the tie after the Catapult's turn, none of "no Wait/Defend" (the test pins the opposite) | F-r3a-1, F-r3a-2, F-r3a-5 |
| BAT-SIE-012 | matches (`siege.rs:881-913`: saturating damage, destruction only when present; a wall opens `opens[t]`, the gate sets `gate_open` and `broken`; `walls()` 160, `blocks()` 366-374, `moat()` 150-156, `line_blocked()` 631-637, `close_gate()` 549 never closes a destroyed gate, `entered()` emits no GateOpened) | exe 0x465990 whole (damage ≤ 0 returns; hp − d clamped at 0; at 0: walls clear bit 2 of hex 0x63BD00[index] → 78 for target 2, the gate writes +0x53A4 = 0, towers/keep the turret's dead bit; picture state); VCMI `CBattleInfoCallback.cpp:80-86, 196, 1363-1371, 1474, 1499`; batch 3d A (62 refused, 78 walked; gate 0 to the end; 95/96 passed without stop or damage) | `walls.rs:482-562` pin the wall hex, damage-1 stays, the gate open for good, no MoatDamage, no GateClosed; `town.rs:274-276, 334-340` the line through a destroyed wall (Teleport) | ok |
| HER-BAT-018 | matches (`derive.rs:946-950`: highest `Ballistics { level }` among effects, no growth; validated 1..=3 in `hero.rs:338-344`) | exe 0x445DA3 and 0x473CB4 read +0xD3 (skill 10, Ballistics) of the controlling side's hero; VCMI `config/skills.json` not re-opened (the claim is the same as the code's) | `h3-heroes/src/tests/derive.rs:629-663` pins levels 1–3, max with an artifact | ok |
| (on the way) BAT-SIE-014 roll, miss, re-aim | code matches the exe (`siege.rs:771-790, 838-873`) | exe 0x445D30 whole (level by creature, shots loop, re-aim over 0x63B820 = [0..7] with the tie roll), 0x445EC0 whole (damage then hit, the kind switch 0x445FAC, the miss over 0x63B840 = [1,2,5,4], tie roll ≤ 50 takes the later entry) | not reviewed | F-r3a-3 (text) |

Quirks: none in this slice (BAT-QUIRK-22 is Earthquake's, r3b).

## Findings

### F-r3a-1: After the Catapult's turn the original hands the equal-speed tie to the attacker; the engine hands it to the defender
- rules: BAT-SIE-011 (the Catapult's turn in the round's order), BAT-TRN-001 (the tie at equal speeds)
- kind: behaviour-bug
- severity: high (every round of every siege with an attacking hero where the two sides' fastest stacks tie in speed: the first regular turn goes to the wrong side)
- confidence: medium-high; a confirmation row would raise it (see "Wiki checks wanted")
- evidence: the pick 0x465080 at 0x46508B–0x4650C5: when the record activated last (+0x132BC ≥ 0) belongs to side +0x132B8 == 0 (attacker) and its creature is 0x91 (the Catapult), it writes +0x132B8 = 1 and +0x132BC = 0 before choosing; the tie-break 0x46502A–0x465052 prefers the stack whose side differs from +0x132B8. `installation.md` §7.21 (line 1958) already lists "the pick's Catapult case (0x4650BB, sieges)" among the writers of +0x132B8, but T6 did not carry it over: `turn.rs:139` and `turn.rs:282` record `last_side_acted = Some(state.at(mover).side)` for the Catapult (attacker) like any stack, and `next_active` (`turn.rs:31-33`) prefers the opponent of `last_side_acted`. Scenario: a Fort siege, attacker with a hero, both sides' fastest stacks of speed 7. Round 1: the Catapult shoots first (both). Original: +0x132B8 becomes 1, the attacker's stack of speed 7 acts next. Engine: `last_side_acted = Attacker`, the defender's stack of speed 7 acts next. With a Citadel (the keep first, defender) the same: original keep → Catapult → attacker; engine keep → Catapult → defender. Holds for the automatic and the commanded Catapult, and for a turn that passes (action 12 is an activation too).
- fix or test sketch: in `turn.rs`, after a turn of the stack in `CATAPULT_SLOT` (both the automatic path at 282 and `end_turn` at 139) set `last_side_acted = Some(BattleSide::Defender)` (the original's "defender's record 0"). Test (`needs-run`): a Fort siege, attacker hero without Ballistics, attacker and defender one stack each of equal speed; after the Catapult's shot `s.active()` is the attacker's stack; the same with a Citadel after the keep's turn. Check the replays (`siege-s7` and the golden sieges) change.

### F-r3a-2: The commanded Catapult may Wait and Defend; the rule says the wall shot is its only command, and the original offers no Defend
- rules: BAT-SIE-011
- kind: behaviour-bug (the rule text and the original against the code and its test)
- severity: medium (a hero with Ballistics: the agent can skip the shots with Defend, or Wait to move the Catapult's shots to the wait phase; a player of the original cannot)
- confidence: medium for Defend (batch 3d B: "Defend does nothing there", the game waited for the aim); low for Wait (not observed; the original's handlers 0x479706/0x479906 were not read for creature 145)
- evidence: `legal.rs:64-73` calls `turn_commands` for the Catapult too, which pushes Wait (`legal.rs:96-98`) and Defend (99) unconditionally; `command.rs:233-240` accepts them; `walls.rs:286-289` asserts the commanded Catapult's commands are among Wait | Defend | ShootWall | Retreat and `walls.rs:457-458` applies `Command::Wait` to it successfully. The summary: "the side then shoots the walls with Command::ShootWall at any standing target (the only legal command besides the side's hero's)". The original's 0x473C00 returns without an action for a player's hero with Ballistics (0x473D02–0x473D09) and the cursor offers the standing targets' aim hexes; batch 3d B records that Defend did nothing.
- fix or test sketch: either exclude the Catapult from Wait/Defend in `turn_commands` and refuse them in `validate` (`CannotWait`/a new error) and change the test, or change the rule text to say the engine allows Wait and Defend "as any commanded war machine" and record it as a deliberate deviation (the user's call). Wiki/experiment check for Wait listed below.

### F-r3a-3: BAT-SIE-014's tie text says "at most 50 for the first"; the code and the engine take the later candidate on ≤ 50
- rules: BAT-SIE-014 (outside the slice; re-read for D-12 as the prompt asked)
- kind: rule-text
- severity: low (the odds are 50/50 either way; only the replay of the original's RNG stream would differ)
- confidence: high
- evidence: 0x445F62–0x445F7A (the miss) and 0x445E3A–0x445E52 (the re-aim): at an equal distance `random(1, 100)`, `cmp eax, 0x32; jg skip` — a roll above 50 keeps the entry found first, a roll ≤ 50 replaces it with the later entry. `siege.rs:782` does the same (`Some((_, b)) if d == b => roll <= 50` takes the new candidate). `installation.md` line 2623 ("a tie replaced with random(1, 100) ≤ 50") and the code comment at `siege.rs:769` say it right; the catalog summary ("a tie decided by random(1, 100) at most 50 for the first") says the opposite or is at best ambiguous.
- fix: "a tie: random(1, 100) at most 50 takes the later entry of the list, above 50 keeps the earlier".

### F-r3a-4: BAT-SIE-009's VCMI source names the wrong file
- rules: BAT-SIE-009
- kind: source
- severity: low
- confidence: high
- evidence: the entry cites `server/battles/BattleProcessor.cpp` for "the attacker's catapult stack in a siege with a fort, placed on hex 120". In VCMI 1.7.5 that file contains no Catapult code (grep: none); the placement is `lib/battle/BattleInfo.cpp:311-320` (`handleWarMachine(ATTACKER, MACH4, warMachineHexes.at(3))` only when `town->fortificationsLevel().wallsHealth > 0`) with the hex from `config/gameConfig.json:630` (`attackerWarMachines: [52, 18, 154, 120]`). The claim itself is true (VCMI agrees: the Catapult at 120, with a fort only, from the hero's MACH4 artifact).
- fix: cite `lib/battle/BattleInfo.cpp` and `config/gameConfig.json`.

### F-r3a-5: "With no standing target its turn passes (it defends)" — the original's action 12 gives no Defend bonus, and neither does the engine
- rules: BAT-SIE-011
- kind: rule-text
- severity: low (the Catapult's defense once every target has fallen and a defender attacks it)
- confidence: high
- evidence: 0x473C53 / 0x473CA2 set action 12; the dispatcher's case at 0x479BC2 only sets bit 0x4000000 on +0x84 (the acted flag) and falls to the winner test; Defend is action 3 with the bonus at +0x4DC (0x479706). The engine (`turn.rs:329-333`) runs `war::act` (nothing to shoot) and sets ACTED, no Defend bonus: it matches the code, the text does not.
- fix: "(its turn passes: no action, no Defend bonus)".

### F-r3a-6: No test pins the order "the turrets, then the Catapult" in one battle, nor the tie after the Catapult's turn
- rules: BAT-SIE-011
- kind: test-gap
- severity: low (the ordering key is pinned piecewise: turrets before a fast stack in `turrets.rs:152-175`, the Catapult before a fast stack in `walls.rs:245-275`; `walls.rs:429` implies the keep acted before the commanded Catapult at a Citadel)
- confidence: high
- test sketch: a Castle with an attacking hero: the round's TurnStarted order begins keep, upper tower, lower tower, Catapult; plus F-r3a-1's tie test.

### F-r3a-7 (question, low): a hypnotized Catapult would shoot with its owner's Ballistics, the original with its controller's
- rules: BAT-SIE-011, BAT-SIE-014
- kind: question
- severity: low (only if Hypnotize can take the Catapult at all)
- confidence: low
- evidence: `siege.rs:724-725, 759-760` read `crate::hero::catapult(&state.heroes, s.side)` (the owner's side), while `legal.rs:46` and `command.rs:205` let the controller command it; 0x445D64–0x445D86 picks the hero of `1 − side` when +0x288 (the hypnotized flag) is set, so the original uses the controller's Ballistics. The check that settles it: whether BAT-MAG-026's targets exclude war machines (if they do, nothing to fix).

## Wiki checks wanted
- Ballistics page: does the player with Ballistics have to shoot (no Wait, no Defend for the Catapult)? Any statement on the Catapult's place in the turn order and on who acts after it at equal speeds (F-r3a-1; better: a confirmation row in a batch — a Fort siege, equal top speeds, attacker's hero, note which stack acts after the Catapult in rounds 1 and 2).
- Catapult page: whether the Catapult can be hypnotized or targeted by spells (F-r3a-7).

## Not reviewed, left open
- BAT-SIE-014's hit and damage tests and its event wording (another reviewer's); only its code against 0x445D30/0x445EC0 was re-read.
- 0x474BF0 (which side is automatic) and 0x46A4A0 (hidden battle): the engine's "the agent's choice" sentence rests on the T6 reading; not re-read.
- Whether good morale after action 9 (0x464920) can give the Catapult a second turn (BAT-SIE-014's "good morale included" against the war machines' NO_MORALE).
- The turrets' half of BAT-SIE-012 (r4).
