# Review r2a: the siege's setup, the walls, the gate and the moat's stop

## Coverage

| rule | text↔code | sources checked (exe addresses re-read; VCMI files; batch rows) | tests | verdict |
|---|---|---|---|---|
| BAT-SIE-001 | `h3-battle/src/siege.rs` `setup` (238–356), `h3-content-battle/src/siege.rs` `compile_field`/`compile`, `terrain.rs` 110–125, `state.rs` 1082–1092: every clause as stated (UnknownSiege for a missing record or field, InvalidSiege for a moat below a Citadel and for a natural obstacle on walls/moat/mines, no random obstacles from a Fort, the town public) | exe: the gate's set at 0x463C10 (inside 0x4639F0's side setup), the gate's writes (`grep` of +0x53A4: 0x463C10, 0x465A17, 0x467281, 0x4672C4, 0x467413, 0x467447); VCMI `lib/battle/BattleInfo.cpp` not re-read (the moat hexes are in `config/spells/moats.json`, see F-r2a-3) | `sieges.rs` 118–213 pin the errors, the public town, the fort-less town, no obstacles | ok (F-r2a-3 touches its VCMI path only) |
| BAT-SIE-002 | `blocks` 366–374, `walls()` 160–162, `placement_blocked`, `movement.rs` `fits`/`reach` (walkers never path through, flyers never land on a wall hex; the gate's hex handled by BAT-SIE-003; destroyed walls' hexes leave `line` via `broken`) | exe: hex_blocked 0x469A10 whole (the blocked bit 0x2 for every hex but 95/96; the destroyed gate 0x465990 → +0x53A4 = 0, a destroyed wall clears the bit), stack_fits_at_hex 0x446C40 (the hex test's order: hex_blocked, then the living-stack test that excludes the mover) | `the_walls_block_walkers_and_landings`; the walls property (no stack and no legal move on a standing wall hex at every step) | ok |
| BAT-SIE-003 | `blocked_for` 394–402, `blocks`, `lies` 378–384, `entered` 527–540, `close_gate` 546–563, callers in `obstacles.rs` 766/794/838, `command.rs` 520, `turn.rs` 137, `created.rs` 416; the destroyed gate 893 | exe: 0x469A10 (95 and 96 alike: free while +0x53A4 ≠ 3; closed: blocked unless the acting side +0x132C0 is 1 and hex 95's and hex 94's stack ref (record +0x18) and corpse count (+0x1C) are empty, 94 in every town), 0x467130 whole (side flipped by +0x288, fort ≠ 0, gate = 3, hex or the two-hex stack's other hex in 94–96), the walk 0x442161–0x442300 (defender_on_gate_hex on the next hex, then gate_open 0x4671C0 at 0x442230, before the stop test 0x442266 and before the hex is entered), the flight 0x4B4A40–0x4B4AB0 (opens before the landing), gate_close 0x4672E0–0x46735F (only from state 1; 96's and 95's stack and corpse count, 94's only for town type 7), its callers (0x4423F2, 0x479C65 before next_stack_to_act, 0x4B4A29, 0x4B4EF9), 0x465990 (a destroyed gate writes 0: never closes, never blocks); VCMI `server/battles/BattleProcessor.cpp` 225–268, `lib/battle/CBattleInfoCallback.cpp` 398–412, 1320–1334; batch 3c G2, G4, G6 | `sieges.rs` 251–366 (open on the gate's hex, close after the walk, the quiet sink's same state, a stack or a corpse on 94 shuts it for the defender, the Fortress's 94 holds it); the walls property; the reach test | F-r2a-1, F-r2a-3, F-r2a-4, F-r2a-5 |
| BAT-SIE-004 | `stops` 462–470, `moat()` 150–156, `bridges()`, `movement.rs` 94–172 (`enters` on either hex of the footprint, the start's own hexes excepted, a stop position reached but not expanded), `aim.rs` 343–354, the attack refusals `command.rs` 282, `legal.rs` 133 | exe: mark_walk_stops 0x4B3290 whole (flag 0x2 flyers and creatures 54/55 get no stops; the 11 hexes of 0x63BCE8 with +0x53A8, 0x63BCF4 with +0x53A9; 95 (94 with the Fortress flag) cleared while the gate ≠ 3 and, while closed, for the side-flipped-by-+0x288 = 1; the stack's own hex and other hex cleared), the pathfinder 0x4B3680–0x4B3730 (a two-hex stack's step is a stop when the new rear or the new other hex is a stop, hexes it covered at the previous position excepted), is_moat_hex 0x469DC0 whole (the second row by the town type +0x53C8+4 = 7, 95/94 only while +0x53A4 = 3); VCMI `getStoppers` 1485–1506, `config/spells/moats.json`; batch 3c M2, M5 | `the_moat_stops_walkers_but_not_flyers_or_the_defender_on_the_bridges`, the Fortress test 344–366; the walls property (a walk that enters the moat outside its start hexes ends there) | F-r2a-2, F-r2a-4 |

No quirk lies in this slice.

The two questions the task prompt asked, answered from the re-read: (a) the gate opens for a stack commanded for the defender (hypnotize-aware) that begins a walk's step onto 94, 95 or 96, or whose flight lands there, or that is summoned there (0x5A71D7, T8's), with either hex of a two-hex stack; a stack standing in it (living or a corpse, either side) keeps it open; a destroyed gate is state 0: never closed, never blocking. The engine does the same, with the opening reported after the step instead of before (G2's note). (b) The moat stops every walker of either side (a flyer, Devils and Arch Devils never), on the first moat hex it newly enters with either hex of its footprint; the bridges (95, and 94 in the Fortress) stop nobody while the gate is not closed and never a stack commanded for the defender. The engine agrees on every point, including the two-hex stack's rear (`enters` tests the whole footprint; the original's 0x4B36ED–0x4B3725 tests the new rear and the new other hex).

## Findings

### F-r2a-1: a corpse-less stack killed on the gate's hexes keeps the gate open through the killer's good-morale turn

- rules: BAT-SIE-003 (BAT-MAG-037 for the corpse)
- kind: behaviour-bug
- severity: medium (needs a created stack of the defender's on 94–96 and a kill followed by a good-morale turn or an automatic action; then an attacker's stack may walk through the closed-by-rights gate)
- confidence: high that the engine does it (traced below); medium that the original closes before the morale turn (the dispatcher calls gate_close at 0x479C65 right before next_stack_to_act 0x479C73; which path a good-morale turn takes through the dispatcher I did not read)
- evidence:
  - `crates/h3-battle/src/siege.rs:378–384` `lies` counts a stack when `corpse != NO_CORPSE` and its footprint covers the hex: a stack that just died has `corpse == 0` until `raise::mark_corpses` runs (`state.rs:670–674`), so a dead **created** stack (which leaves no corpse, BAT-MAG-037; it gets `NO_CORPSE` only in `mark_corpses`, `raise.rs:42–45`) counts as lying there until then.
  - `crates/h3-battle/src/command.rs:518–524`: `close_gate` runs at line 520 **before** `mark_corpses` at 524. `turn.rs:137` (`end_turn`) runs before the morale check at 141. On a good-morale turn `morale_turn` → `activation` (turn.rs 143–146) runs no `mark_corpses` (its only callers: command.rs:524, turn.rs:249 in `activate_next`, turn.rs:356 after a berserk action). So: the kill → `end_turn` → `close_gate` (the dead elemental still "lies") → good morale → back at 520 `close_gate` (still lies) → 524 `mark_corpses` (now `NO_CORPSE`). The next command is the killer's morale turn with `gate_open == true`; `blocked_for` (`siege.rs:394–402`) then frees 95 and 96 for the attacker, so its walker may walk into the town (or land on 96 and hold the gate open with its own body).
  - Without good morale the gate still closes within the same command (line 520 runs after `activate_next`'s `mark_corpses`), only with `GateClosed` after the next `TurnStarted`: harmless. An automatic action activated in `activate_next` (a berserk attacker's walker) also sees the open gate.
  - The original: 0x4672E0 reads the hex records' stack ref and corpse count (0x2BDC/0x2BE0 for 96, 0x2B6C/0x2B70 for 95); a summoned stack's death adds no corpse to the hex (BAT-MAG-037's source, 0x444120), so at 0x479C65 the gate closes at that action's end. VCMI (`BattleProcessor.cpp:229–231`) also decides on the units present after the action.
  - Scenario: Castle with a Fort; the defender's summoned Air Elementals walked onto the drawbridge 95 (gate open, held by them); an attacker's Archers (hero with +3 morale) shoot them dead and roll good morale → in the engine their morale turn's legal moves include 95 and 96 and a walker's path through them; in the original the gate is closed and 95/96 block.
- fix or test sketch: in `lies`, do not count a dead stack that will leave no corpse: `(s.is_alive() || (corpse != NO_CORPSE && !s.is_created()))` (a sacrificed stack already carries `NO_CORPSE`, `raise.rs:418`; a vanished clone too, `created.rs:498`). Test (`needs-run`): `t:castle`, Fort, `Moat::Absent`; the defender's stack placed on (10,5) with `StackTraits::SUMMONED` set and `siege.gate_open = true`; the attacker's shooter adjacent-not-needed with an attacker hero giving morale +3; loop seeds until the shot's events contain the good-morale event; assert `!open(&t)` and `gate(&events).contains(&false)` and that the attacker's legal `Move` destinations in the morale turn exclude (10,5) and (11,5). The current code fails the assertion on the gate.

### F-r2a-2: BAT-SIE-004 does not say that a walker entering the moat cannot attack from there in the same move

- rules: BAT-SIE-004
- kind: rule-text
- severity: low (the engine and, by the pathfinder reading, the original agree; the text is incomplete)
- confidence: medium (rests on the inferred pathfinder note 0x4B378D–0x4B37D1, the same mechanism BAT-MAG-032 settled with the user, BAT-Q30; no batch row shows a refused attack from a moat hex)
- evidence: `movement.rs:155–157` marks a moat stop position in `reach.stops`; `command.rs:280–284` refuses `Attack { from }` with `CommandError::Unreachable` when `reach.stops_at(from)`, `legal.rs:133` lists no such attack, `aim.rs:354` and `control.rs:133` take no such position for automatic turns. The original marks the moat as stops in the same array as Quicksand (0x4B3290), and the pathfinder offers no attack target from a stop (map entry of 0x4B3400). BAT-MAG-032's summary states the clause for pits; BAT-SIE-004's says only "stops on the first moat hex it enters".
- fix or test sketch: add to the summary "an attack ordered with a move that ends in the moat is not legal (as a seen pit's, BAT-MAG-032)" and a test: the attacker's walker two hexes from a defender's stack standing on a breach (or on 95 with the gate open), the moat hex between; `Attack { from: moat hex }` → `Unreachable`, not listed in `legal_commands`. Also a wiki check (below).

### F-r2a-3: the VCMI notes misstate VCMI 1.7.5 (the corpses, the moat hexes' file)

- rules: BAT-SIE-003 (also BAT-SIE-001, BAT-SIE-004 for the file path)
- kind: source
- severity: low
- confidence: high
- evidence: BAT-SIE-003's VCMI source says "only living stacks keep it open". `server/battles/BattleProcessor.cpp:229–231` reads `battleGetUnitByPos(hex, false)` for 96, 94 and 95, and `lib/battle/CBattleInfoCallback.cpp:398–412` with `onlyAlive == false` returns dead (non-ghost) units too: corpses keep VCMI's gate open and a corpse on 94 makes it `BLOCKED`. So VCMI agrees with the original and the engine; the note records a disagreement that does not exist. BAT-SIE-001's and BAT-SIE-004's VCMI sources cite `config/factions/*.json town.moatHexes`; in 1.7.5 the factions' files carry only `moatAbility`, and the hexes (and the Fortress's double row, which omits 95) are in `config/spells/moats.json` (`castleMoat` line 66, `fortressMoat` line 686).
- fix: reword the notes; keep the medium confidence.

### F-r2a-4: test gaps on the gate's opening by a flight and on the walk-around row of batch M

- rules: BAT-SIE-003, BAT-SIE-004
- kind: test-gap
- severity: low
- confidence: high
- evidence: no unit test lands a defender's flyer on 94/95/96 (the opening path through `travel_through` with `walks == false`, `obstacles.rs:868–874` → `moved` → `entered`); the walls property checks every `Moved` of a defender's stack onto the openers (properties.rs:2206) but the reach test counts only "opened" by any means. Batch M5 (a walker on moat hex 111 sent to 146 goes round 129: 111 → 128 → 145 → 146, no stop before 146) has no unit test with a single-row moat: `the_moat_stops_walkers_...` uses `t:inferno` (double row) and only one-step moves out of the moat. No test refuses an attack from a moat hex (F-r2a-2).
- fix or test sketch: `t:castle` Citadel, Harmless: a walker (speed ≥ 3) on (9,6) sent to (9,8): `destinations` contains (9,8) and `act` arrives there with no stop; a defender's flyer at (13,5) landing on (10,5): `gate(&events) == [true]` and `open`.

### F-r2a-5: the rule text leaves out that the defender's own stack on 94 bars itself from the drawbridge

- rules: BAT-SIE-003
- kind: rule-text
- severity: low
- confidence: high for the engine (`siege.rs:398–399` reads `lies` on 94 with the mover included; its comment says so); medium-high for the original (0x469A10 reads hex 94's record raw, and 0x446C40 calls it before its own living-stack test that excludes the mover; the pathfinder does not call take_stack_off_hexes 0x468730, whose callers are walk_step, the flight animation, 0x4B4F10 and cast_spell)
- evidence: a defender's walker that starts its turn on 94 (any town; the gate closed after its earlier walk out) cannot step 94 → 95 → 96: 95 is blocked because 94 "holds a living stack", its own. It must go round through 77 or 111 (moat hexes: a stop for it) and enter 95 the turn after. The summary says "no living stack … lies on the drawbridge's or the outer bridge's hex", which covers it only by implication. Batch G6 showed the rule with another stack on 94, not the mover.
- fix: one clause in the summary ("the moving stack itself included"); a test: the defender's walker placed on (9,5), gate closed: `destinations` excludes (10,5) and (11,5).

## Wiki checks wanted

- Moat page: does it say a creature that enters the moat cannot attack in the same turn (F-r2a-2)? Any statement on a defender's creature standing on the outer bridge hex blocking its own entry (F-r2a-5)?
- Siege page: whether the drawbridge opens for a defender's creature *flying* onto the bridge hex and whether the gate stays open while a corpse lies in it (both read in the code; the catalog's wiki source quotes only "the attacker must breach the walls or the gate").

## Not reviewed, left open

- The gate during the tactics phase: a tactics placement goes through `obstacles::moved` → `entered` (`command.rs:476`), so a defender's stack placed on 94–96 opens the gate and holds it; whether the original's tactics move reaches 0x442186 and whether the defender's tactics zone includes column 11 at a town I did not read.
- The Teleport spell onto 95/96 for the defender while the gate is closed (no opening in either; then 95 is a moat hex for that stack): BAT-SIE-026's reviewer.
- The property `walls_and_the_moat_agree_with_movement`'s doc comment says a summoned stack "may stand there without opening the gate"; the code (`created.rs:416`) opens it (BAT-SIE-028). A stale comment only, T8's slice.
- The observation's encoding of the walls and the moat (`h3-sim/src/observation.rs:312`): not read beyond the marker.
- The good-morale path through the original's action dispatcher (whether 0x479C65 runs before the morale turn): would raise F-r2a-1's confidence on the original's side; the engine's side needs no further reading.
